mod app;
mod clipboard_backend;
mod history;
mod hotkey;
mod interpreter;
mod window_state;

fn main() -> eframe::Result<()> {
    let ws_path = window_state::window_state_file_path();
    let ws = window_state::load(&ws_path);

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Clipboard Hack")
            .with_inner_size([ws.width, ws.height])
            .with_position([ws.x, ws.y])
            .with_min_inner_size([600.0, 400.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Clipboard Hack",
        options,
        Box::new(|cc| Ok(Box::new(app::App::new(cc)))),
    )
}
