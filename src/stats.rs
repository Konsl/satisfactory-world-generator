use std::collections::HashMap;

use strum::IntoEnumIterator;

use crate::game::{ResourceDescriptor, World};

pub struct Stats(HashMap<(i32, i32, ResourceDescriptor), f32>);

impl Stats {
    pub const CLOCK_SPEEDS: [i32; 2] = [100, 250];
    pub const MINER_MK_MAX: i32 = 3;

    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    fn get_miner_factor(mk: i32) -> f32 {
        2f32.powi(mk - 1)
    }

    pub fn get(&self, clock_speed: i32, miner_mk: i32, resource: ResourceDescriptor) -> f32 {
        self.0
            .get(&(clock_speed, miner_mk, resource))
            .copied()
            .unwrap_or(0.0)
    }

    pub fn compute(&mut self, world: &World) {
        self.clear();

        for resource in ResourceDescriptor::iter() {
            for clock_speed in Self::CLOCK_SPEEDS {
                for miner_mk in 1..=Self::MINER_MK_MAX {
                    self.0.insert(
                        (clock_speed, miner_mk, resource),
                        world.get_extraction_rate(
                            resource,
                            clock_speed as f32 / 100.0,
                            Self::get_miner_factor(miner_mk),
                        ),
                    );
                }
            }
        }
    }
}
