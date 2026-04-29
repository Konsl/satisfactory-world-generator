use serde::Deserialize;

#[derive(Clone, Copy, Debug)]
pub enum GameplayTag {
    Basic,
    Advanced,
    FossilFuel,
}

#[derive(PartialEq, Eq, Hash, Deserialize, Clone, Copy, Debug, strum::EnumIter, strum::Display)]
#[allow(clippy::upper_case_acronyms)]
pub enum ResourceDescriptor {
    #[serde(rename = "Desc_OreIron_C")]
    #[strum(to_string = "Iron")]
    OreIron,
    #[serde(rename = "Desc_Coal_C")]
    #[strum(to_string = "Coal")]
    Coal,
    #[serde(rename = "Desc_OreCopper_C")]
    #[strum(to_string = "Copper")]
    OreCopper,
    #[serde(rename = "Desc_Stone_C")]
    #[strum(to_string = "Limestone")]
    Stone,
    #[serde(rename = "Desc_RawQuartz_C")]
    #[strum(to_string = "Quartz")]
    RawQuartz,
    #[serde(rename = "Desc_LiquidOil_C")]
    #[strum(to_string = "Crude Oil")]
    LiquidOil,
    #[serde(rename = "Desc_Water_C")]
    #[strum(to_string = "Water")]
    Water,
    #[serde(rename = "Desc_SAM_C")]
    #[strum(to_string = "SAM")]
    SAM,
    #[serde(rename = "Desc_NitrogenGas_C")]
    #[strum(to_string = "Nitrogen Gas")]
    NitrogenGas,
    #[serde(rename = "Desc_OreBauxite_C")]
    #[strum(to_string = "Bauxite")]
    OreBauxite,
    #[serde(rename = "Desc_OreGold_C")]
    #[strum(to_string = "Caterium")]
    OreGold,
    #[serde(rename = "Desc_Sulfur_C")]
    #[strum(to_string = "Sulfur")]
    Sulfur,
    #[serde(rename = "Desc_OreUranium_C")]
    #[strum(to_string = "Uranium")]
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

#[derive(PartialEq, Eq, PartialOrd, Ord, Deserialize, Clone, Copy, Debug, strum::EnumIter)]
pub enum ResourcePurity {
    #[serde(rename = "RP_Inpure")]
    Impure = 1,
    #[serde(rename = "RP_Normal")]
    Normal = 2,
    #[serde(rename = "RP_Pure")]
    Pure = 4,
}

impl ResourcePurity {
    fn get_factor(&self) -> f32 {
        match self {
            Self::Impure => 0.5,
            Self::Normal => 1.0,
            Self::Pure => 2.0,
        }
    }
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

impl World {
    pub fn get_extraction_rate(
        &self,
        resource: ResourceDescriptor,
        global_factor: f32,
        miner_factor: f32,
    ) -> f32 {
        let node_extraction_rate = if resource == ResourceDescriptor::LiquidOil {
            120.0
        } else {
            miner_factor * 60.0
        };

        let nodes_total_rate = node_extraction_rate
            * self
                .resource_nodes
                .iter()
                .filter(|n| n.resource == resource)
                .map(|n| n.purity.get_factor())
                .sum::<f32>();

        let fracking_total_rate = 60.0
            * self
                .fracking_cores
                .iter()
                .filter(|c| c.resource == resource)
                .flat_map(|c| c.satellites.iter().map(|s| s.purity.get_factor()))
                .sum::<f32>();

        global_factor * (nodes_total_rate + fracking_total_rate)
    }
}
