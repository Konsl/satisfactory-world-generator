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

    return MarkerShape::Circle;
    /* recommend all markers be circles and color purity like scim
    match purity {
        ResourcePurity::Impure => MarkerShape::Up,
        ResourcePurity::Normal => MarkerShape::Diamond,
        ResourcePurity::Pure => MarkerShape::Circle,
    }*/
}

pub fn get_purity_color(purity: ResourcePurity) -> Color32 {
    match purity {
        ResourcePurity::Impure => Color32::from_rgb(210, 52, 48),
        ResourcePurity::Normal => Color32::from_rgb(242, 100, 24),
        ResourcePurity::Pure => Color32::from_rgb(128, 177, 57),
    }
}


pub fn get_resource_stroke(resource: ResourceDescriptor) -> Color32 {
    Color32::from_hex(match resource {
        ResourceDescriptor::OreIron => "#CDCDCD",
        ResourceDescriptor::Coal => "#000000",
        ResourceDescriptor::OreCopper => "#B87333",
        ResourceDescriptor::Stone => "#E3DAC9",
        ResourceDescriptor::RawQuartz => "#9F6C99",
        ResourceDescriptor::SAM => "#502E8E",
        ResourceDescriptor::OreBauxite => "#A76E56",
        ResourceDescriptor::OreGold => "#FFD700",
        ResourceDescriptor::Sulfur => "#FFD700",
        ResourceDescriptor::OreUranium => "#357336",
        ResourceDescriptor::Water => "#7AB0D4",
        ResourceDescriptor::LiquidOil => "#000000",
        ResourceDescriptor::NitrogenGas => "#FFFFFF",
    })
        .unwrap()
}