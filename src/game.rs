use std::fmt::Display;

use serde::Deserialize;
use strum::EnumIter;

#[derive(Clone, Copy, Debug)]
pub enum GameplayTag {
    Basic,
    Advanced,
    FossilFuel,
}

#[derive(PartialEq, Eq, Hash, Deserialize, Clone, Copy, Debug, EnumIter)]
#[allow(clippy::upper_case_acronyms)]
pub enum ResourceDescriptor {
    #[serde(rename = "Desc_OreIron_C")]
    OreIron,
    #[serde(rename = "Desc_Coal_C")]
    Coal,
    #[serde(rename = "Desc_OreCopper_C")]
    OreCopper,
    #[serde(rename = "Desc_Stone_C")]
    Stone,
    #[serde(rename = "Desc_RawQuartz_C")]
    RawQuartz,
    #[serde(rename = "Desc_LiquidOil_C")]
    LiquidOil,
    #[serde(rename = "Desc_Water_C")]
    Water,
    #[serde(rename = "Desc_SAM_C")]
    SAM,
    #[serde(rename = "Desc_NitrogenGas_C")]
    NitrogenGas,
    #[serde(rename = "Desc_OreBauxite_C")]
    OreBauxite,
    #[serde(rename = "Desc_OreGold_C")]
    OreGold,
    #[serde(rename = "Desc_Sulfur_C")]
    Sulfur,
    #[serde(rename = "Desc_OreUranium_C")]
    OreUranium,
}

impl ResourceDescriptor {
    pub fn get_internal_name(&self) -> &'static str {
        match self {
            Self::OreIron => "Desc_OreIron_C",
            Self::Coal => "Desc_Coal_C",
            Self::OreCopper => "Desc_OreCopper_C",
            Self::Stone => "Desc_Stone_C",
            Self::RawQuartz => "Desc_RawQuartz_C",
            Self::LiquidOil => "Desc_LiquidOil_C",
            Self::Water => "Desc_Water_C",
            Self::SAM => "Desc_SAM_C",
            Self::NitrogenGas => "Desc_NitrogenGas_C",
            Self::OreBauxite => "Desc_OreBauxite_C",
            Self::OreGold => "Desc_OreGold_C",
            Self::Sulfur => "Desc_Sulfur_C",
            Self::OreUranium => "Desc_OreUranium_C",
        }
    }
    
    pub fn get_color(&self) -> &'static str {
        match self {
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
    }
}

impl Display for ResourceDescriptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::OreIron => "Iron",
            Self::Coal => "Coal",
            Self::OreCopper => "Copper",
            Self::Stone => "Limestone",
            Self::RawQuartz => "Quartz",
            Self::LiquidOil => "Crude Oil",
            Self::Water => "Water",
            Self::SAM => "SAM",
            Self::NitrogenGas => "Nitrogen Gas",
            Self::OreBauxite => "Bauxite",
            Self::OreGold => "Caterium",
            Self::Sulfur => "Sulfur",
            Self::OreUranium => "Uranium",
        })
    }
}

impl ResourceDescriptor {
    pub fn has_tag(&self, tag: GameplayTag) -> bool {
        match tag {
            GameplayTag::Basic => matches!(
                self,
                Self::OreIron | Self::Coal | Self::OreCopper | Self::Stone
            ),
            GameplayTag::Advanced => matches!(
                self,
                Self::RawQuartz
                    | Self::SAM
                    | Self::OreBauxite
                    | Self::OreGold
                    | Self::Sulfur
                    | Self::OreUranium
            ),
            GameplayTag::FossilFuel => matches!(self, Self::Coal | Self::LiquidOil | Self::Sulfur),
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Deserialize, Clone, Copy, Debug, EnumIter)]
pub enum ResourcePurity {
    #[serde(rename = "RP_Inpure")]
    Impure = 1,
    #[serde(rename = "RP_Normal")]
    Normal = 2,
    #[serde(rename = "RP_Pure")]
    Pure = 4,
}

pub type Vector = [f32; 3];

#[derive(Deserialize, Clone, Debug)]
pub struct ResourceNode {
    pub name: String,
    pub location: Vector,
    pub resource: ResourceDescriptor,
    pub purity: ResourcePurity,
}

#[derive(Deserialize, Clone, Debug)]
pub struct GeyserNode {
    pub name: String,
    pub location: Vector,
    pub purity: ResourcePurity,
}

#[derive(Deserialize, Clone, Debug)]
pub struct FrackingCore {
    pub name: String,
    pub location: Vector,
    pub resource: ResourceDescriptor,
    pub satellites: Vec<FrackingSatellite>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct FrackingSatellite {
    pub name: String,
    pub location: Vector,
    pub purity: ResourcePurity,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct World {
    pub game_version: String,
    pub resource_nodes: Vec<ResourceNode>,
    pub geysers: Vec<GeyserNode>,
    pub fracking_cores: Vec<FrackingCore>,
}
