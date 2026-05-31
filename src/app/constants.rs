use egui::Color32;
use egui_plot::MarkerShape;

use crate::game::{ResourceDescriptor, ResourcePurity};

pub fn get_resource_color(resource: ResourceDescriptor, is_dark_mode: bool) -> Color32 {
    Color32::from_hex(if is_dark_mode {
        match resource {
            ResourceDescriptor::OreIron => "#975f6a",
            ResourceDescriptor::Coal => "#15008e",
            ResourceDescriptor::OreCopper => "#9b4c2b",
            ResourceDescriptor::Stone => "#56452d",
            ResourceDescriptor::RawQuartz => "#9f6c99",
            ResourceDescriptor::SAM => "#502e8e",
            ResourceDescriptor::OreBauxite => "#68392d",
            ResourceDescriptor::OreGold => "#af9c72",
            ResourceDescriptor::Sulfur => "#afaa27",
            ResourceDescriptor::OreUranium => "#357336",
            ResourceDescriptor::Water => "#4a88ab",
            ResourceDescriptor::LiquidOil => "#603560",
            ResourceDescriptor::NitrogenGas => "#7d8089",
        }
    } else {
        match resource {
            ResourceDescriptor::OreIron => "#bf7887",
            ResourceDescriptor::Coal => "#1b00b5",
            ResourceDescriptor::OreCopper => "#c46137",
            ResourceDescriptor::Stone => "#6e5839",
            ResourceDescriptor::RawQuartz => "#c787bf",
            ResourceDescriptor::SAM => "#653ab5",
            ResourceDescriptor::OreBauxite => "#854839",
            ResourceDescriptor::OreGold => "#dec590",
            ResourceDescriptor::Sulfur => "#ded831",
            ResourceDescriptor::OreUranium => "#439144",
            ResourceDescriptor::Water => "#5dabd9",
            ResourceDescriptor::LiquidOil => "#7a437a",
            ResourceDescriptor::NitrogenGas => "#9ea2ad",
        }
    })
    .unwrap()
}

pub fn get_purity_marker(purity: ResourcePurity) -> MarkerShape {
    match purity {
        ResourcePurity::Impure => MarkerShape::Up,
        ResourcePurity::Normal => MarkerShape::Diamond,
        ResourcePurity::Pure => MarkerShape::Circle,
    }
}
