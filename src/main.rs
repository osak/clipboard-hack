mod app;
mod clipboard_backend;
mod history;
mod hotkey;
mod interpreter;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Clipboard Hack")
            .with_inner_size([900.0, 600.0])
            .with_min_inner_size([600.0, 400.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Clipboard Hack",
        options,
        Box::new(|cc| Ok(Box::new(app::App::new(cc)))),
    )
}
