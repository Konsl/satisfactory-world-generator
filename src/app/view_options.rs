use std::collections::{HashMap, HashSet};

use egui_plot::PlotMemory;
use strum::IntoEnumIterator;

use crate::game::{ResourceDescriptor, ResourcePurity, World};

#[derive(Clone, Copy)]
pub enum ViewOptionsTarget {
    All,
    Geysers,

    Resource(ResourceDescriptor),
    Purity(ResourcePurity),
    FrackingNodes,

    ResourceWithPurity(ResourceDescriptor, ResourcePurity),
    ResourceFrackingNodes(ResourceDescriptor),
}

pub struct ViewOptions {
    world_outline_visible: bool,
    geysers_visible: bool,

    /// impure, normal, pure, fracking
    /// see [ViewOptions::get_purity_index] and [ViewOptions::FRACKING_INDEX]
    visible_items: HashMap<ResourceDescriptor, [bool; 4]>,

    /// resource types where there exist resource nodes of that type
    existing_node_resources: HashSet<ResourceDescriptor>,
    /// resource types where there exist fracking nodes of that type
    existing_fracking_resources: HashSet<ResourceDescriptor>,
}

impl ViewOptions {
    const ALL_VISIBLE: [bool; 4] = [true; 4];
    const NONE_VISIBLE: [bool; 4] = [false; 4];

    const FRACKING_INDEX: usize = 3;

    pub fn new() -> Self {
        Self {
            world_outline_visible: true,
            geysers_visible: true,

            visible_items: ResourceDescriptor::iter()
                .map(|r| (r, Self::ALL_VISIBLE))
                .collect(),

            existing_node_resources: HashSet::new(),
            existing_fracking_resources: HashSet::new(),
        }
    }

    fn get_purity_index(purity: ResourcePurity) -> usize {
        match purity {
            ResourcePurity::Impure => 0,
            ResourcePurity::Normal => 1,
            ResourcePurity::Pure => 2,
        }
    }

    /// create a list of hidden plot item ids for initializing the plot legend
    /// this is used to sync the `ViewOptions` state to the plot widget
    pub fn get_hidden_items(&self) -> HashSet<egui::Id> {
        ResourceDescriptor::iter()
            .filter(|&r| {
                !self
                    .is_target_visible(ViewOptionsTarget::Resource(r))
                    .unwrap_or(true)
            })
            .map(|r| egui::Id::new(r.to_string()))
            .chain((!self.geysers_visible).then(|| egui::Id::new("Geyser")))
            .chain((!self.world_outline_visible).then(|| egui::Id::new("World Outline")))
            .collect()
    }

    /// read `PlotMemory` and apply the changes
    /// this is used to sync the plot widget state to the `ViewOptions`
    pub fn apply_legend_interaction(&mut self, egui_context: &egui::Context, plot_id: egui::Id) {
        let Some(mem) = PlotMemory::load(egui_context, plot_id) else {
            return;
        };

        self.geysers_visible = !mem.hidden_items.contains(&egui::Id::new("Geyser"));
        self.world_outline_visible = !mem.hidden_items.contains(&egui::Id::new("World Outline"));

        for resource in ResourceDescriptor::iter() {
            self.set_target_visible(
                ViewOptionsTarget::Resource(resource),
                !mem.hidden_items
                    .contains(&egui::Id::new(resource.to_string())),
            );
        }
    }

    pub fn get_existing_nodes(&mut self, world: &World) {
        self.existing_node_resources = world
            .resource_nodes
            .iter()
            .map(|n| n.resource)
            .collect::<HashSet<_>>();

        self.existing_fracking_resources = world
            .fracking_cores
            .iter()
            .map(|c| c.resource)
            .collect::<HashSet<_>>();
    }

    /// only [ViewOptionsTarget::ResourceWithPurity] and [ViewOptionsTarget::ResourceFrackingNodes] can not exist
    /// note that this doesn't actually filter by purity
    pub fn target_exists(&self, target: ViewOptionsTarget) -> bool {
        match target {
            ViewOptionsTarget::ResourceWithPurity(resource, _) => {
                self.existing_node_resources.contains(&resource)
            }
            ViewOptionsTarget::ResourceFrackingNodes(resource) => {
                self.existing_fracking_resources.contains(&resource)
            }
            _ => true,
        }
    }

    fn is_node_type_visible(
        &self,
        type_index: usize,
        relevant_resources: &HashSet<ResourceDescriptor>,
    ) -> Option<bool> {
        let mut all_visible = true;
        let mut all_hidden = true;

        for resource in ResourceDescriptor::iter() {
            if !relevant_resources.contains(&resource) {
                continue;
            }

            let Some(resource_entry) = self.visible_items.get(&resource) else {
                all_visible = false;
                continue;
            };

            if resource_entry[type_index] {
                all_hidden = false;
            } else {
                all_visible = false;
            }
        }

        if all_visible {
            Some(true)
        } else if all_hidden {
            Some(false)
        } else {
            None
        }
    }

    fn is_resource_with_type_visible(
        &self,
        resource: ResourceDescriptor,
        type_index: usize,
    ) -> bool {
        self.visible_items
            .get(&resource)
            .is_some_and(|v| v[type_index])
    }

    /// returns `Some(true)` if that target is fully visible, `Some(false)` if it is fully hidden,
    /// and `None` otherwise
    pub fn is_target_visible(&self, target: ViewOptionsTarget) -> Option<bool> {
        match target {
            ViewOptionsTarget::All => {
                let mut all_visible = true;
                let mut all_hidden = true;

                for resource in ResourceDescriptor::iter() {
                    match self.is_target_visible(ViewOptionsTarget::Resource(resource)) {
                        Some(true) => all_hidden = false,
                        Some(false) => all_visible = false,
                        None => {
                            all_visible = false;
                            all_hidden = false;
                        }
                    }
                }

                if all_visible {
                    Some(true)
                } else if all_hidden {
                    Some(false)
                } else {
                    None
                }
            }
            ViewOptionsTarget::Geysers => Some(self.geysers_visible),
            ViewOptionsTarget::Resource(resource) => {
                let Some(resource_entry) = self.visible_items.get(&resource) else {
                    return Some(false);
                };

                // "relevant" entries are the entries relevant to this resource type
                // we do this because we want to display a resource that only has fracking nodes
                // as enabled if it has fracking nodes enabled, regardless of whether other
                // types of resource nodes are enabled for that resource
                // (they should not matter because they dont exist)

                let mut relevant_entries: &[bool] = resource_entry;
                if !self.existing_fracking_resources.contains(&resource) {
                    relevant_entries = &relevant_entries[..Self::FRACKING_INDEX];
                }
                if !self.existing_node_resources.contains(&resource) {
                    relevant_entries = &relevant_entries[Self::FRACKING_INDEX..];
                }

                if relevant_entries.iter().all(|v| *v) {
                    Some(true)
                } else if relevant_entries.iter().any(|v| *v) {
                    None
                } else {
                    Some(false)
                }
            }
            ViewOptionsTarget::Purity(purity) => self.is_node_type_visible(
                Self::get_purity_index(purity),
                &self.existing_node_resources,
            ),
            ViewOptionsTarget::FrackingNodes => {
                self.is_node_type_visible(Self::FRACKING_INDEX, &self.existing_fracking_resources)
            }
            ViewOptionsTarget::ResourceWithPurity(resource, purity) => {
                Some(self.is_resource_with_type_visible(resource, Self::get_purity_index(purity)))
            }
            ViewOptionsTarget::ResourceFrackingNodes(resource) => {
                Some(self.is_resource_with_type_visible(resource, Self::FRACKING_INDEX))
            }
        }
    }

    fn set_node_type_visible(&mut self, type_index: usize, visible: bool) {
        if visible {
            if self.visible_items.values().any(|e| e[type_index]) {
                return;
            }

            for resource in ResourceDescriptor::iter() {
                let entry = self
                    .visible_items
                    .entry(resource)
                    .or_insert(Self::NONE_VISIBLE);

                entry[type_index] = visible;
            }
        } else {
            for (_, entry) in self.visible_items.iter_mut() {
                entry[type_index] = false;
            }
        }
    }

    fn set_resource_with_type_visible(
        &mut self,
        resource: ResourceDescriptor,
        type_index: usize,
        visible: bool,
    ) {
        if !self.visible_items.contains_key(&resource) && !visible {
            return;
        }

        let visibility = self
            .visible_items
            .entry(resource)
            .or_insert(Self::NONE_VISIBLE);

        visibility[type_index] = visible;
    }

    pub fn set_target_visible(&mut self, target: ViewOptionsTarget, visible: bool) {
        match target {
            ViewOptionsTarget::All => {
                if visible {
                    if self.visible_items.values().any(|e| e.iter().any(|&v| v)) {
                        return;
                    }

                    for resource in ResourceDescriptor::iter() {
                        self.visible_items.insert(resource, Self::ALL_VISIBLE);
                    }
                } else {
                    self.visible_items.clear();
                }
            }
            ViewOptionsTarget::Geysers => self.geysers_visible = visible,
            ViewOptionsTarget::Resource(resource) => {
                if visible {
                    self.visible_items
                        .entry(resource)
                        .or_insert(Self::ALL_VISIBLE);
                } else {
                    self.visible_items.remove(&resource);
                }
            }
            ViewOptionsTarget::Purity(purity) => {
                self.set_node_type_visible(Self::get_purity_index(purity), visible)
            }
            ViewOptionsTarget::FrackingNodes => {
                self.set_node_type_visible(Self::FRACKING_INDEX, visible)
            }
            ViewOptionsTarget::ResourceWithPurity(resource, purity) => self
                .set_resource_with_type_visible(resource, Self::get_purity_index(purity), visible),
            ViewOptionsTarget::ResourceFrackingNodes(resource) => {
                self.set_resource_with_type_visible(resource, Self::FRACKING_INDEX, visible)
            }
        }
    }

    pub fn world_outline_visible_mut(&mut self) -> &mut bool {
        &mut self.world_outline_visible
    }

    pub fn geysers_visible_mut(&mut self) -> &mut bool {
        &mut self.geysers_visible
    }
}
