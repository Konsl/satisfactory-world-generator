use std::{collections::HashSet, fmt::Display};

use strum::{EnumIter, IntoEnumIterator};

use crate::{
    game::{FrackingCore, GameplayTag, ResourceDescriptor, ResourceNode, ResourcePurity, World},
    random_stream::RandomStream,
};

#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy, EnumIter)]
pub enum NodeRandomizationMode {
    None,
    Strict,
    BasicRich,
    AdvancedRich,
    FossilFuelRich,
}

impl Display for NodeRandomizationMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::None => "none",
            Self::Strict => "random",
            Self::BasicRich => "more basic nodes",
            Self::AdvancedRich => "more advanced nodes",
            Self::FossilFuelRich => "more fossil fuels",
        })
    }
}

#[derive(Eq, PartialEq, Debug, Clone, Copy, EnumIter)]
pub enum NodePuritySettings {
    NoChange,
    AllImpure,
    Decrease,
    AllNormal,
    Increase,
    AllPure,
    AllRandom,
}

impl Display for NodePuritySettings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::NoChange => "unchanged",
            Self::AllPure => "all pure",
            Self::AllNormal => "all normal",
            Self::AllImpure => "all impure",
            Self::AllRandom => "random",
            Self::Increase => "increase",
            Self::Decrease => "decrease",
        })
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ResourceNodeInfo {
    pub resource: ResourceDescriptor,
    pub purity: Option<ResourcePurity>,
    pub total_throughput: i32,
}

impl PartialOrd for ResourceNodeInfo {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ResourceNodeInfo {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.resource == other.resource {
            if self.purity == other.purity {
                self.total_throughput.cmp(&other.total_throughput)
            } else {
                self.purity.cmp(&other.purity)
            }
        } else {
            self.resource
                .get_internal_name()
                .cmp(other.resource.get_internal_name())
        }
    }
}

impl From<&ResourceNode> for ResourceNodeInfo {
    fn from(value: &ResourceNode) -> Self {
        Self {
            resource: value.resource,
            purity: Some(value.purity),
            total_throughput: 0,
        }
    }
}

impl From<&FrackingCore> for ResourceNodeInfo {
    fn from(value: &FrackingCore) -> Self {
        Self {
            resource: value.resource,
            purity: Some(ResourcePurity::Pure),
            total_throughput: value.satellites.iter().map(|s| s.purity as i32).sum(),
        }
    }
}

pub fn shuffle<T>(rng: &mut RandomStream, node_pool: &mut [T]) {
    let mut i = 0;

    while i < node_pool.len() - 1 {
        let swap_index = rng.frand_range(i as f32..node_pool.len() as f32) as usize;
        node_pool.swap(i, swap_index);

        i += 1;
    }
}

pub fn get_purity_override(
    rng: &mut RandomStream,
    purity: Option<ResourcePurity>,
    purity_settings: NodePuritySettings,
) -> Option<ResourcePurity> {
    match purity_settings {
        NodePuritySettings::NoChange => None,
        NodePuritySettings::AllPure => Some(ResourcePurity::Pure),
        NodePuritySettings::AllNormal => Some(ResourcePurity::Normal),
        NodePuritySettings::AllImpure => Some(ResourcePurity::Impure),
        NodePuritySettings::AllRandom => match rng.frand_range(0.0..3.0) as u32 {
            0 => Some(ResourcePurity::Impure),
            1 => Some(ResourcePurity::Normal),
            2 => Some(ResourcePurity::Pure),
            _ => None,
        },
        NodePuritySettings::Increase => match purity? {
            ResourcePurity::Impure => Some(ResourcePurity::Normal),
            ResourcePurity::Normal | ResourcePurity::Pure => Some(ResourcePurity::Pure),
        },
        NodePuritySettings::Decrease => match purity? {
            ResourcePurity::Impure | ResourcePurity::Normal => Some(ResourcePurity::Impure),
            ResourcePurity::Pure => Some(ResourcePurity::Normal),
        },
    }
}

pub fn modify_node_distribution(
    rng: &mut RandomStream,
    node_pool: &mut [ResourceNodeInfo],
    tag: GameplayTag,
    multiplier: f32,
) {
    assert!(multiplier >= 1.0, "cannot decrease node count");

    let mut matching_node_count = node_pool.iter().filter(|n| n.resource.has_tag(tag)).count();
    let modified_node_count = (matching_node_count as f32 * multiplier).round() as usize;

    let mut resource_options = ResourceDescriptor::iter()
        .filter(|r| r.has_tag(tag))
        .collect::<Vec<_>>();
    resource_options.sort_by_key(|r| r.get_internal_name());

    shuffle(rng, node_pool);

    let mut seen_resources = HashSet::<ResourceDescriptor>::new();
    for n in node_pool.iter_mut() {
        if matching_node_count >= modified_node_count {
            break;
        }

        if n.resource.has_tag(tag) {
            continue;
        }

        if seen_resources.insert(n.resource) {
            continue;
        }

        let new_resource =
            resource_options[rng.frand_range(0.0..resource_options.len() as f32) as usize];
        println!("converting {} to {}", n.resource, new_resource);
        n.resource = new_resource;

        matching_node_count += 1;
    }
}

pub fn distribute_throughput(fracking_core: &mut FrackingCore, throughput: i32) {
    fracking_core
        .satellites
        .iter_mut()
        .for_each(|s| s.purity = ResourcePurity::Pure);

    let mut error =
        fracking_core.satellites.len() as i32 * (ResourcePurity::Pure as i32) - throughput;
    if error < 2 {
        return;
    }

    let convert_count = (error as usize / 2).min(fracking_core.satellites.len());
    fracking_core
        .satellites
        .iter_mut()
        .take(convert_count)
        .for_each(|s| s.purity = ResourcePurity::Normal);
    error += convert_count as i32 * (ResourcePurity::Normal as i32 - ResourcePurity::Pure as i32);

    if error < 1 {
        return;
    }

    if error > 15 {
        panic!();
    }

    fracking_core
        .satellites
        .iter_mut()
        .take(error as usize)
        .for_each(|s| s.purity = ResourcePurity::Impure);
}

pub fn apply_randomization_settings(
    world: &mut World,
    seed: i32,
    randomization_mode: NodeRandomizationMode,
    purity_settings: NodePuritySettings,
) {
    let mut rng = RandomStream::new(seed);

    world.resource_nodes.sort_by_key(|n| n.name.clone());
    world.geysers.sort_by_key(|g| g.name.clone());
    world.fracking_cores.sort_by_key(|c| c.name.clone());
    world
        .fracking_cores
        .iter_mut()
        .for_each(|c| c.satellites.sort_by_key(|s| s.name.clone()));

    if randomization_mode == NodeRandomizationMode::None {
        for n in world.resource_nodes.iter_mut() {
            let Some(new_purity) = get_purity_override(&mut rng, Some(n.purity), purity_settings)
            else {
                continue;
            };
            n.purity = new_purity;
        }
    } else {
        let mut node_pool = world
            .resource_nodes
            .iter()
            .map(ResourceNodeInfo::from)
            .collect::<Vec<_>>();
        node_pool.sort();

        match randomization_mode {
            NodeRandomizationMode::BasicRich => {
                modify_node_distribution(&mut rng, &mut node_pool, GameplayTag::Basic, 1.1)
            }
            NodeRandomizationMode::AdvancedRich => {
                modify_node_distribution(&mut rng, &mut node_pool, GameplayTag::Advanced, 3.0)
            }
            NodeRandomizationMode::FossilFuelRich => {
                modify_node_distribution(&mut rng, &mut node_pool, GameplayTag::FossilFuel, 2.0)
            }
            _ => (),
        }

        for n in world.resource_nodes.iter_mut() {
            let pool_index = rng.frand_range(0.0..node_pool.len() as f32) as usize;
            let node_info = node_pool.remove(pool_index);

            n.resource = node_info.resource;
            let Some(new_purity) = get_purity_override(&mut rng, node_info.purity, purity_settings)
            else {
                continue;
            };
            n.purity = new_purity;
        }

        let mut fracking_node_pool = world
            .fracking_cores
            .iter()
            .map(ResourceNodeInfo::from)
            .collect::<Vec<_>>();
        fracking_node_pool.sort();

        shuffle(&mut rng, &mut fracking_node_pool);

        for core in world.fracking_cores.iter_mut() {
            let pool_index = rng.frand_range(0.0..fracking_node_pool.len() as f32) as usize;
            let node_info = fracking_node_pool.remove(pool_index);

            core.resource = node_info.resource;
            distribute_throughput(core, node_info.total_throughput);
        }
    }

    if purity_settings != NodePuritySettings::NoChange {
        let mut satellites = world
            .fracking_cores
            .iter_mut()
            .flat_map(|c| c.satellites.iter_mut())
            .collect::<Vec<_>>();
        satellites.sort_by_key(|s| s.name.clone());

        for s in satellites {
            let Some(new_purity) = get_purity_override(&mut rng, Some(s.purity), purity_settings)
            else {
                continue;
            };
            s.purity = new_purity;
        }
    }
}
