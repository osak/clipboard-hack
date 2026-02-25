use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver};

use arboard::Clipboard;
use egui::{Color32, Key, Modifiers, RichText, ScrollArea, Ui};

use crate::clipboard_backend;
use crate::history::ClipboardHistory;
use crate::hotkey::{hotkey_display, start_hotkey_listener};
use crate::interpreter::{get_interpreters, Interpreter, InterpretItem};

/// Touching this file signals the app to capture the clipboard.
/// Useful for wiring a Wayland compositor hotkey:
///   e.g. bind = CTRL+SHIFT+H, exec, touch /tmp/clipboard-hack-trigger
const TRIGGER_FILE: &str = "/tmp/clipboard-hack-trigger";

pub struct App {
    history: ClipboardHistory,
    history_path: PathBuf,
    selected_index: Option<usize>,
    rx: Receiver<()>,
    clipboard: Option<Clipboard>,
    interpreters: Vec<Box<dyn Interpreter>>,
    status_message: String,
    trigger_path: PathBuf,
}

/// Search common system font paths for a file that supports Japanese,
/// load its bytes, and register it as an egui fallback font.
fn setup_japanese_font(ctx: &egui::Context) {
    // Candidates in priority order.  TTC index 2 = NotoSansCJK JP face.
    let candidates: &[(&str, u32)] = &[
        // Linux – Noto CJK (JP face is index 2 in the standard TTC)
        ("/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc", 2),
        ("/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc", 2),
        ("/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc", 2),
        ("/usr/share/fonts/google-noto-cjk/NotoSansCJK-Regular.ttc", 2),
        // Linux – BIZ UDP Gothic (single TTF, no index needed)
        ("/home/osak/.local/share/fonts/b/BIZUDPGothic_Regular.ttf", 0),
        // Linux – IPA / VL Gothic
        ("/usr/share/fonts/opentype/ipagothic/ipagothic.ttf", 0),
        ("/usr/share/fonts/truetype/vlgothic/VL-Gothic-Regular.ttf", 0),
        // macOS – Hiragino
        ("/System/Library/Fonts/ヒラギノ角ゴシック W3.ttc", 0),
        ("/System/Library/Fonts/Hiragino Sans GB.ttc", 0),
    ];

    for (path, index) in candidates {
        if let Ok(bytes) = std::fs::read(path) {
            let mut fonts = egui::FontDefinitions::default();
            fonts.font_data.insert(
                "cjk_font".to_owned(),
                egui::FontData {
                    font: bytes.into(),
                    index: *index,
                    tweak: Default::default(),
                },
            );
            // Register as fallback so the built-in Latin font still renders ASCII.
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .push("cjk_font".to_owned());
            fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .push("cjk_font".to_owned());
            ctx.set_fonts(fonts);
            eprintln!("[font] Loaded {path} (index {index})");
            return;
        }
    }
    eprintln!("[font] No Japanese font found; CJK characters may not render.");
}

/// Returns the path where history is persisted.
/// Linux/others: $XDG_DATA_HOME/clipboard-hack/history.json
/// macOS:        ~/Library/Application Support/clipboard-hack/history.json
fn history_file_path() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME").unwrap_or_default();
        PathBuf::from(home)
            .join("Library")
            .join("Application Support")
            .join("clipboard-hack")
            .join("history.json")
    }
    #[cfg(not(target_os = "macos"))]
    {
        let base = std::env::var("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                let home = std::env::var("HOME").unwrap_or_default();
                PathBuf::from(home).join(".local").join("share")
            });
        base.join("clipboard-hack").join("history.json")
    }
}

impl App {
    pub fn new(cc: &eframe::CreationContext) -> Self {
        setup_japanese_font(&cc.egui_ctx);

        let (tx, rx) = mpsc::channel();
        start_hotkey_listener(tx);

        let clipboard = Clipboard::new().ok();

        let history_path = history_file_path();
        let history = ClipboardHistory::load(&history_path, 50);
        eprintln!("[history] Loaded {} entries from {}", history.len(), history_path.display());

        let is_wayland = std::env::var("WAYLAND_DISPLAY").is_ok();
        let status = if is_wayland {
            format!(
                "Wayland detected. In-app hotkey: {}  |  Global: touch {}",
                hotkey_display(),
                TRIGGER_FILE
            )
        } else {
            format!("Ready. Hotkey: {}", hotkey_display())
        };

        Self {
            history,
            history_path,
            selected_index: None,
            rx,
            clipboard,
            interpreters: get_interpreters(),
            status_message: status,
            trigger_path: PathBuf::from(TRIGGER_FILE),
        }
    }

    fn save_history(&mut self) {
        if let Err(e) = self.history.save(&self.history_path) {
            eprintln!("[history] Save failed: {e}");
        }
    }

    fn capture_clipboard(&mut self) {
        match clipboard_backend::get_text(&mut self.clipboard) {
            Ok(text) => {
                if self.history.add(text) {
                    self.save_history();
                }
                self.status_message = "Captured.".to_string();
                self.selected_index = Some(0);
            }
            Err(e) => {
                self.status_message = format!("Error: {e}");
            }
        }
    }

    fn draw_toolbar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if ui.button("📋 Capture Now").clicked() {
                self.capture_clipboard();
            }
            if ui.button("🗑 Clear History").clicked() {
                self.history.clear();
                self.save_history();
                self.selected_index = None;
                self.status_message = "History cleared.".to_string();
            }
            ui.separator();
            ui.label(
                RichText::new(format!("Hotkey: {}", hotkey_display()))
                    .color(Color32::GRAY),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    RichText::new(&self.status_message)
                        .color(Color32::from_rgb(180, 180, 180))
                        .italics(),
                );
            });
        });
    }

    fn draw_history_panel(&mut self, ui: &mut Ui) {
        ui.heading("History");
        ui.label(
            RichText::new(format!("{} item(s)", self.history.len()))
                .color(Color32::GRAY)
                .small(),
        );
        ui.separator();

        if self.history.is_empty() {
            ui.colored_label(
                Color32::GRAY,
                "No history yet.\nPress 'Capture Now' or use the hotkey.",
            );
            return;
        }

        let mut to_delete: Option<usize> = None;

        ScrollArea::vertical().show(ui, |ui| {
            let items: Vec<(usize, String, String)> = self
                .history
                .entries()
                .iter()
                .enumerate()
                .map(|(i, e)| (i, e.timestamp_str(), e.preview(45)))
                .collect();

            // Row height: 2 lines of button-style text + vertical padding
            let font_id = egui::TextStyle::Button.resolve(ui.style());
            let line_h = ui.fonts(|f| f.row_height(&font_id));
            let row_h = line_h * 2.0 + ui.spacing().button_padding.y * 2.0;

            for (i, ts, preview) in items {
                let selected = self.selected_index == Some(i);
                let label = format!("{}\n{}", ts, preview);

                let (sel_clicked, del_clicked) = ui.horizontal(|ui| {
                    let avail = ui.available_width();
                    let btn_w = 20.0;
                    let gap = ui.spacing().item_spacing.x;
                    let label_w = (avail - btn_w - gap).max(0.0);

                    // allocate_ui_with_layout で top_down(LEFT) コンテキストを作る。
                    // SelectableLabel はこのコンテキストの h_align() = LEFT を参照して
                    // テキストを左寄せに配置する。
                    let sel = ui.allocate_ui_with_layout(
                        egui::vec2(label_w, row_h),
                        egui::Layout::top_down_justified(egui::Align::LEFT),
                        |ui| ui.selectable_label(selected, &label),
                    ).inner;

                    let del = ui.add_sized([btn_w, row_h], egui::Button::new("×").small());
                    (sel.clicked(), del.clicked())
                }).inner;

                if sel_clicked {
                    self.selected_index = Some(i);
                }
                if del_clicked {
                    to_delete = Some(i);
                }
            }
        });

        if let Some(idx) = to_delete {
            self.delete_history_entry(idx);
        }
    }

    fn delete_history_entry(&mut self, idx: usize) {
        self.history.remove(idx);
        self.save_history();
        self.selected_index = match self.selected_index {
            Some(sel) if sel == idx => None,
            Some(sel) if sel > idx => Some(sel - 1),
            other => other,
        };
    }

    fn draw_detail_panel(&mut self, ui: &mut Ui) {
        if let Some(idx) = self.selected_index {
            if let Some(entry) = self.history.get(idx) {
                let content = entry.content().to_string();
                let captured_at = entry.timestamp_str();

                ui.heading("Content");
                ui.label(
                    RichText::new(format!("Captured at {captured_at}"))
                        .color(Color32::GRAY)
                        .small(),
                );
                ui.separator();

                ScrollArea::vertical()
                    .id_salt("content_scroll")
                    .max_height(120.0)
                    .show(ui, |ui| {
                        ui.code(&content);
                    });

                ui.add_space(8.0);
                ui.separator();
                ui.heading("Interpretations");
                ui.add_space(4.0);

                let results: Vec<(&str, Option<Vec<InterpretItem>>)> = self
                    .interpreters
                    .iter()
                    .map(|interp| {
                        let name = interp.name();
                        let result = interp.interpret(&content).map(|r| r.items);
                        (name, result)
                    })
                    .collect();

                ScrollArea::vertical()
                    .id_salt("interp_scroll")
                    .show(ui, |ui| {
                        for (name, maybe_items) in results {
                            let header_text = if maybe_items.is_some() {
                                RichText::new(name).strong()
                            } else {
                                RichText::new(format!("{name}  (not applicable)"))
                                    .color(Color32::from_rgb(120, 120, 120))
                            };

                            egui::CollapsingHeader::new(header_text)
                                .default_open(maybe_items.is_some())
                                .show(ui, |ui| {
                                    if let Some(items) = maybe_items {
                                        egui::Grid::new(format!("grid_{name}"))
                                            .num_columns(3)
                                            .striped(true)
                                            .spacing([8.0, 4.0])
                                            .show(ui, |ui| {
                                                for item in &items {
                                                    ui.label(
                                                        RichText::new(&item.label)
                                                            .color(Color32::GRAY),
                                                    );
                                                    ui.label(":");
                                                    ui.horizontal(|ui| {
                                                        if let Some(rgba) = item.color {
                                                            let color =
                                                                Color32::from_rgba_unmultiplied(
                                                                    rgba[0], rgba[1], rgba[2],
                                                                    rgba[3],
                                                                );
                                                            let (rect, _) = ui.allocate_exact_size(
                                                                egui::vec2(16.0, 16.0),
                                                                egui::Sense::hover(),
                                                            );
                                                            ui.painter()
                                                                .rect_filled(rect, 3.0, color);
                                                        }
                                                        ui.code(&item.value);
                                                    });
                                                    ui.end_row();
                                                }
                                            });
                                    } else {
                                        ui.colored_label(
                                            Color32::from_rgb(120, 120, 120),
                                            "—",
                                        );
                                    }
                                });
                        }
                    });
            }
        } else {
            ui.centered_and_justified(|ui| {
                ui.colored_label(Color32::GRAY, "Select an item from the history.");
            });
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 1. rdev-based global hotkey (works on X11 / macOS)
        while self.rx.try_recv().is_ok() {
            self.capture_clipboard();
        }

        // 2. In-app keyboard shortcut: Ctrl+Shift+H (works on Wayland when app is focused)
        if ctx.input(|i| {
            i.modifiers == Modifiers::CTRL | Modifiers::SHIFT && i.key_pressed(Key::H)
        }) {
            self.capture_clipboard();
        }

        // 3. File-based trigger: `touch /tmp/clipboard-hack-trigger`
        //    Works with any Wayland compositor hotkey binding.
        if self.trigger_path.exists() {
            let _ = std::fs::remove_file(&self.trigger_path);
            self.capture_clipboard();
        }

        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            self.draw_toolbar(ui);
        });

        egui::SidePanel::left("history_panel")
            .min_width(200.0)
            .default_width(260.0)
            .show(ctx, |ui| {
                self.draw_history_panel(ui);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.draw_detail_panel(ui);
        });

        ctx.request_repaint_after(std::time::Duration::from_millis(50));
    }
}
