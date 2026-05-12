use std::collections::{HashMap, HashSet};

use egui_plot::PlotMemory;
use strum::IntoEnumIterator;

use crate::game::{ResourceDescriptor, ResourcePurity};

pub struct ViewOptions {
    geysers_visible: bool,
    /// impure, normal, pure, fracking
    visible_items: HashMap<ResourceDescriptor, [bool; 4]>,
}

impl ViewOptions {
    const ALL_VISIBLE: [bool; 4] = [true; 4];
    const NONE_VISIBLE: [bool; 4] = [false; 4];

    const FRACKING_INDEX: usize = 3;

    pub fn new() -> Self {
        Self {
            geysers_visible: true,
            visible_items: ResourceDescriptor::iter()
                .map(|r| (r, Self::ALL_VISIBLE))
                .collect(),
        }
    }

    fn get_purity_index(purity: ResourcePurity) -> usize {
        match purity {
            ResourcePurity::Impure => 0,
            ResourcePurity::Normal => 1,
            ResourcePurity::Pure => 2,
        }
    }

    pub fn get_hidden_items(&self) -> HashSet<egui::Id> {
        ResourceDescriptor::iter()
            .filter(|&r| !self.is_resource_visible(r))
            .map(|r| egui::Id::new(r.to_string()))
            .chain((!self.geysers_visible).then(|| egui::Id::new("Geyser")))
            .collect()
    }

    pub fn apply_legend_interaction(&mut self, egui_context: &egui::Context, plot_id: egui::Id) {
        let Some(mem) = PlotMemory::load(egui_context, plot_id) else {
            return;
        };

        self.geysers_visible = !mem.hidden_items.contains(&egui::Id::new("Geyser"));

        for resource in ResourceDescriptor::iter() {
            self.enable_resource(
                resource,
                !mem.hidden_items
                    .contains(&egui::Id::new(resource.to_string())),
            );
        }
    }

    pub fn should_display_geysers(&self) -> bool {
        self.geysers_visible
    }

    pub fn should_display_nodes(
        &self,
        resource: ResourceDescriptor,
        purity: ResourcePurity,
    ) -> bool {
        self.visible_items
            .get(&resource)
            .is_some_and(|v| v[Self::get_purity_index(purity)])
    }

    pub fn should_display_fracking(&self, resource: ResourceDescriptor) -> bool {
        self.visible_items
            .get(&resource)
            .is_some_and(|v| v[Self::FRACKING_INDEX])
    }

    pub fn is_resource_visible(&self, resource: ResourceDescriptor) -> bool {
        self.visible_items
            .get(&resource)
            .is_some_and(|v| v.iter().any(|&v| v))
    }

    pub fn is_resource_partial(&self, resource: ResourceDescriptor) -> bool {
        self.visible_items
            .get(&resource)
            .is_some_and(|v| !v.iter().all(|&v| v))
    }

    pub fn geysers_visible_mut(&mut self) -> &mut bool {
        &mut self.geysers_visible
    }

    pub fn enable_resource(&mut self, resource: ResourceDescriptor, enabled: bool) {
        if enabled {
            self.visible_items
                .entry(resource)
                .or_insert(Self::ALL_VISIBLE);
        } else {
            self.visible_items.remove(&resource);
        }
    }

    pub fn enable_resource_purity(
        &mut self,
        resource: ResourceDescriptor,
        purity: ResourcePurity,
        enabled: bool,
    ) {
        if !self.visible_items.contains_key(&resource) && !enabled {
            return;
        }

        let visibility = self
            .visible_items
            .entry(resource)
            .or_insert(Self::NONE_VISIBLE);

        visibility[Self::get_purity_index(purity)] = enabled;
    }

    pub fn enable_resource_fracking(&mut self, resource: ResourceDescriptor, enabled: bool) {
        if !self.visible_items.contains_key(&resource) && !enabled {
            return;
        }

        let visibility = self
            .visible_items
            .entry(resource)
            .or_insert(Self::NONE_VISIBLE);

        visibility[Self::FRACKING_INDEX] = enabled;
    }
}
