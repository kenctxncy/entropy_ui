mod app;
mod formatting;
mod state;
mod ui;
mod utils;

use app::InfoEntropyApp;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Info Entropy Simulation")
            .with_inner_size([1200.0, 800.0])
            .with_visible(true)
            .with_resizable(true)
            .with_decorations(true),
        ..Default::default()
    };
    eframe::run_native(
        "Info Entropy Simulation",
        native_options,
        Box::new(|_| Ok(Box::<InfoEntropyApp>::default())),
    )
}
