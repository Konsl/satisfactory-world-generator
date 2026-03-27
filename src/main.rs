use crate::app::App;

mod app;
mod game;
mod random_stream;
mod randomization;

fn main() -> eframe::Result {
    env_logger::init();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([400.0, 300.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Satisfactory World Generator",
        native_options,
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}
