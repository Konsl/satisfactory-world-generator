use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use egui::{Layout, RichText};
use egui_extras::{Column, TableBuilder};
use egui_plot::PlotPoint;
use itertools::Itertools;
use strum::IntoEnumIterator;

use crate::{
    app::{constants::get_resource_color, plot_item::{ResourceDisplay, ResourceDisplayContent}}, game::{ResourceDescriptor, World}, randomization::{NodePuritySettings, NodeRandomizationMode, apply_randomization_settings}
};

mod constants;
mod plot_item;

#[derive(PartialEq, Eq, Clone, Copy, strum::EnumIter, strum::Display)]
enum SidePanel {
    #[strum(to_string = "View Options")]
    ViewOptions,
    #[strum(to_string = "Statistics")]
    Stats,
}

type Stats = HashMap<(u8, u8, ResourceDescriptor), f32>;

pub struct App {
    seed: Option<i32>,
    randomization_mode: NodeRandomizationMode,
    purity_settings: NodePuritySettings,

    side_panel: SidePanel,

    world: Option<World>,
    stats: Stats,
    last_calc_duration: Duration,
}

impl Default for App {
    fn default() -> Self {
        Self {
            seed: None,
            randomization_mode: NodeRandomizationMode::None,
            purity_settings: NodePuritySettings::NoChange,

            side_panel: SidePanel::ViewOptions,

            world: None,
            stats: Stats::new(),
            last_calc_duration: Duration::ZERO,
        }
    }
}

impl App {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Default::default()
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn get_time() -> Instant {
        Instant::now()
    }

    #[cfg(target_arch = "wasm32")]
    fn get_time() -> f64 {
        web_sys::window()
            .expect("no window")
            .performance()
            .expect("no performance")
            .now()
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn get_elapsed_duration(start_time: Instant) -> Duration {
        start_time.elapsed()
    }

    #[cfg(target_arch = "wasm32")]
    fn get_elapsed_duration(start_time: f64) -> Duration {
        Duration::from_secs_f64((Self::get_time() - start_time) / 1000.0)
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.global_style_mut(|style| style.interaction.selectable_labels = false);

        egui::Panel::right("settings_panel")
            .resizable(true)
            .min_size(400.0)
            .show_inside(ui, |ui| {
                egui::Panel::bottom("stats_panel")
                    .resizable(true)
                    .min_size(350.0)
                    .show_inside(ui, |ui| {
                        ui.take_available_space();
                        ui.add_space(5.0);

                        ui.horizontal(|ui| {
                            SidePanel::iter().for_each(|v| {
                                ui.selectable_value(&mut self.side_panel, v, v.to_string());
                            })
                        });
                        ui.separator();

                        match self.side_panel {
                            SidePanel::ViewOptions => {
                                ui.label("view sth or dont idfc");
                            }

                            SidePanel::Stats => {
                                let available_height = ui.available_height();
                                let table = TableBuilder::new(ui)
                                    .striped(true)
                                    .resizable(false)
                                    .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                                    .column(Column::remainder())
                                    .column(Column::auto())
                                    .column(Column::auto())
                                    .column(Column::auto())
                                    .column(Column::auto())
                                    .column(Column::auto())
                                    .column(Column::auto())
                                    .min_scrolled_height(0.0)
                                    .max_scroll_height(available_height);

                                table
                                    .header(20.0, |mut header| {
                                        header.col(|ui| {
                                            ui.strong("Resource");
                                        });

                                        for mk in 1..=3 {
                                            for rate in [100, 250] {
                                                header.col(|ui| {
                                                    ui.strong(format!("Mk. {}\n{} %", mk, rate));
                                                });
                                            }
                                        }
                                    })
                                    .body(|mut body| {
                                        for resource in ResourceDescriptor::iter() {
                                            body.row(18.0, |mut row| {
                                                row.col(|ui| {
                                                    ui.label(
                                                        RichText::new("\u{23FA}")
                                                            .color(get_resource_color(resource)),
                                                    );
                                                    ui.label(resource.to_string());
                                                });

                                                for mk in 1..=3 {
                                                    for rate in [100, 250] {
                                                        let amount = self
                                                            .stats
                                                            .get(&(rate, mk, resource))
                                                            .copied()
                                                            .unwrap_or(0.0);

                                                        row.col(|ui| {
                                                            ui.label(format!("{}", amount));
                                                        });
                                                    }
                                                }
                                            });
                                        }
                                    });
                            }
                        }
                    });

                egui::CentralPanel::default().show_inside(ui, |ui| {
                    ui.heading("Randomization Settings");

                    egui::ScrollArea::vertical().show(ui, |ui| {
                        egui::Grid::new("settings_grid")
                            .num_columns(2)
                            .spacing([40.0, 4.0])
                            .striped(true)
                            .show(ui, |ui| {
                                ui.label("Seed");

                                ui.with_layout(
                                    Layout::right_to_left(egui::Align::Center)
                                        .with_cross_justify(true),
                                    |ui| {
                                        let randomize_seed =
                                            ui.button("\u{1F3B2} random").clicked();
                                        let mut seed_text = self
                                            .seed
                                            .map(|seed| seed.to_string())
                                            .unwrap_or_default();
                                        if ui
                                            .add(
                                                egui::TextEdit::singleline(&mut seed_text)
                                                    .hint_text("0"),
                                            )
                                            .changed()
                                        {
                                            self.world = None;
                                        }

                                        if randomize_seed {
                                            self.seed = Some(rand::random());
                                            self.world = None;
                                        } else if seed_text.is_empty() {
                                            self.seed = None;
                                        } else if let Ok(seed) = seed_text.trim().parse::<i32>() {
                                            self.seed = Some(seed);
                                        }
                                    },
                                );

                                ui.end_row();

                                ui.label("Mode");
                                ui.horizontal(|ui| {
                                    egui::ComboBox::from_id_salt("mode_setting")
                                        .selected_text(self.randomization_mode.to_string())
                                        .show_ui(ui, |ui| {
                                            NodeRandomizationMode::iter().for_each(|m| {
                                                if ui
                                                    .selectable_value(
                                                        &mut self.randomization_mode,
                                                        m,
                                                        m.to_string(),
                                                    )
                                                    .changed()
                                                {
                                                    self.world = None;
                                                }
                                            });
                                        });
                                });
                                ui.end_row();

                                ui.label("Purity");
                                ui.horizontal(|ui| {
                                    egui::ComboBox::from_id_salt("purity_setting")
                                        .selected_text(self.purity_settings.to_string())
                                        .show_ui(ui, |ui| {
                                            NodePuritySettings::iter().for_each(|p| {
                                                if ui
                                                    .selectable_value(
                                                        &mut self.purity_settings,
                                                        p,
                                                        p.to_string(),
                                                    )
                                                    .changed()
                                                {
                                                    self.world = None;
                                                }
                                            });
                                        });
                                });
                                ui.end_row();
                            });
                    });
                });
            });

        let world = self.world.get_or_insert_with(|| {
            let start_time = Self::get_time();

            let mut world: World =
                serde_json::from_str(include_str!("../default-world.json")).unwrap();

            apply_randomization_settings(
                &mut world,
                self.seed.unwrap_or_default(),
                self.randomization_mode,
                self.purity_settings,
            );

            let mut stats = Stats::new();
            for resource in ResourceDescriptor::iter() {
                for factor in [100, 250] {
                    for miner_mk in 1..=3 {
                        stats.insert(
                            (factor, miner_mk, resource),
                            world.get_extraction_rate(
                                resource,
                                factor as f32 / 100.0,
                                2f32.powi(miner_mk as i32 - 1),
                            ),
                        );
                    }
                }
            }
            self.stats = stats;

            self.last_calc_duration = Self::get_elapsed_duration(start_time);
            world
        });

        egui::Panel::bottom("status_panel").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                if !self.last_calc_duration.is_zero() {
                    ui.label(format!(
                        "calculation took {:.2} ms",
                        self.last_calc_duration.as_secs_f64() * 1000.0
                    ));
                }

                ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(world.game_version.clone());
                });
            })
        });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            let plot = egui_plot::Plot::new("main_display_plot")
                .legend(egui_plot::Legend::default())
                .show_axes(true)
                .show_grid(true)
                .data_aspect(1.0)
                .invert_y(true);

            plot.show(ui, |plot_ui| {
                let test_rect = plot_ui
                    .transform()
                    .rect_from_values(&PlotPoint::new(0.0, 0.0), &PlotPoint::new(1.0, 1.0));
                let scale = (test_rect.width() + test_rect.height()) / 2.0;
                let base_size = (5000.0 * scale).clamp(5.0, 20.0);

                // resource nodes
                for (resource, nodes) in &world.resource_nodes.iter().chunk_by(|n| n.resource) {
                    plot_ui.add(ResourceDisplay::new(
                        base_size,
                        ResourceDisplayContent::ResourceNodes(resource, nodes.collect()),
                    ));
                }

                // fracking nodes
                for (resource, cores) in &world.fracking_cores.iter().chunk_by(|c| c.resource) {
                    plot_ui.add(ResourceDisplay::new(
                        base_size,
                        ResourceDisplayContent::FrackingNodes(resource, cores.collect()),
                    ));
                }

                // geysers
                plot_ui.add(ResourceDisplay::new(
                    base_size,
                    ResourceDisplayContent::Geysers(world.geysers.iter().by_ref().collect()),
                ));
            });
        });
    }
}
