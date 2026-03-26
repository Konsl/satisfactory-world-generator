use crate::app::App;

mod app;
mod game;
mod random_stream;
mod randomization;

fn main() -> eframe::Result {
    env_logger::init();

    /* let default_world: World = serde_json::from_str(include_str!("default-world.json")).unwrap();

    let mut world = default_world.clone();
    apply_randomization_settings(
        &mut world,
        NodeRandomizationSettings {
            randomization_mode: NodeRandomizationMode::None,
            purity_settings: NodePuritySettings::AllRandom,
            seed: 1789187456,
        },
    );
    // println!("{:#?}", world);

    let mut world = default_world.clone();
    apply_randomization_settings(
        &mut world,
        NodeRandomizationSettings {
            randomization_mode: NodeRandomizationMode::Strict,
            purity_settings: NodePuritySettings::NoChange,
            seed: 24753513,
        },
    );
    // println!("{:#?}", world);

    let mut world = default_world.clone();
    apply_randomization_settings(
        &mut world,
        NodeRandomizationSettings {
            randomization_mode: NodeRandomizationMode::FossilFuelRich,
            purity_settings: NodePuritySettings::NoChange,
            seed: 553135980, //3040594461u32.cast_signed(),
        },
    );
    // println!("{:#?}", world); */

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
