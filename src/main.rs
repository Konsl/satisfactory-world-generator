mod game;
mod random_stream;
mod randomization;

fn main() {
    let default_world: World = serde_json::from_str(include_str!("default-world.json")).unwrap();

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
    // println!("{:#?}", world);
}
