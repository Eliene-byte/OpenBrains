mod app;
mod editor_widget;
mod theme;

use app::EditorApp;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(egui::vec2(1280.0, 820.0))
            .with_min_inner_size(egui::vec2(820.0, 520.0)),
        ..Default::default()
    };
    eframe::run_native(
        "OpenBrains",
        options,
        Box::new(|cc| Ok(Box::new(EditorApp::new(cc)))),
    )
}
