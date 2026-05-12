use egui::Color32;
use egui_plot::MarkerShape;

use crate::game::{ResourceDescriptor, ResourcePurity};

pub fn get_resource_color(resource: ResourceDescriptor) -> Color32 {
    Color32::from_hex(match resource {
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
