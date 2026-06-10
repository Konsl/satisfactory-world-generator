use crate::{game::ResourceDescriptor, include_texture};

mod macros {
    #[macro_export]
    macro_rules! include_texture {
        ($file:expr) => {
            include_bytes!(concat!("../resources/textures/", $file, ".png"))
        };
    }
}

pub fn get_resource_texture(resource: ResourceDescriptor) -> &'static [u8] {
    match resource {
        ResourceDescriptor::OreIron => include_texture!("Desc_OreIron_C"),
        ResourceDescriptor::Coal => include_texture!("Desc_Coal_C"),
        ResourceDescriptor::OreCopper => include_texture!("Desc_OreCopper_C"),
        ResourceDescriptor::Stone => include_texture!("Desc_Stone_C"),
        ResourceDescriptor::RawQuartz => include_texture!("Desc_RawQuartz_C"),
        ResourceDescriptor::LiquidOil => include_texture!("Desc_LiquidOil_C"),
        ResourceDescriptor::Water => include_texture!("Desc_Water_C"),
        ResourceDescriptor::SAM => include_texture!("Desc_SAM_C"),
        ResourceDescriptor::NitrogenGas => include_texture!("Desc_NitrogenGas_C"),
        ResourceDescriptor::OreBauxite => include_texture!("Desc_OreBauxite_C"),
        ResourceDescriptor::OreGold => include_texture!("Desc_OreGold_C"),
        ResourceDescriptor::Sulfur => include_texture!("Desc_Sulfur_C"),
        ResourceDescriptor::OreUranium => include_texture!("Desc_OreUranium_C"),
    }
}

pub fn get_geyser_texture() -> &'static [u8] {
    include_texture!("Desc_GeneratorGeoThermal_C")
}

pub fn load_texture(
    ctx: &egui::Context,
    name: impl Into<String>,
    data: &[u8],
) -> Option<egui::TextureHandle> {
    let image = image::load_from_memory(data).ok()?.to_rgba8();

    let size = [image.width() as usize, image.height() as usize];
    let pixels = image.into_raw();

    Some(ctx.load_texture(
        name,
        egui::ColorImage::from_rgba_unmultiplied(size, &pixels),
        Default::default(),
    ))
}
