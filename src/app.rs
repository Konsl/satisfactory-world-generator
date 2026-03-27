use std::time::{Duration, Instant};

use egui::{Color32, Layout};
use egui_plot::{MarkerShape, PlotItem, PlotPoint, Points};
use strum::IntoEnumIterator;

use crate::{
    game::{ResourceDescriptor, ResourcePurity, World},
    randomization::{NodePuritySettings, NodeRandomizationMode, apply_randomization_settings},
};

pub fn run() {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([400.0, 300.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Satisfactory World Generator",
        native_options,
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    ).unwrap()
}

pub struct App {
    seed: Option<i32>,
    randomization_mode: NodeRandomizationMode,
    purity_settings: NodePuritySettings,

    world: Option<World>,
    last_calc_duration: Duration,
}

impl Default for App {
    fn default() -> Self {
        Self {
            seed: None,
            randomization_mode: NodeRandomizationMode::None,
            purity_settings: NodePuritySettings::NoChange,

            world: None,
            last_calc_duration: Duration::ZERO,
        }
    }
}

impl App {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Default::default()
    }

    fn get_resource_color(resource: ResourceDescriptor) -> Color32 {
        Color32::from_hex(resource.get_color())
        .unwrap()
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.style_mut(|style| style.interaction.selectable_labels = false);

        egui::SidePanel::right("settings_panel")
            .resizable(true)
            .min_width(350.0)
            .show(ctx, |ui| {
                egui::CentralPanel::default().show_inside(ui, |ui| {
                    ui.heading("Randomization Settings");

                    egui::ScrollArea::vertical().show(ui, |ui| {
                        egui::Grid::new("settings_grid")
                            .num_columns(2)
                            .spacing([40.0, 4.0])
                            .striped(true)
                            .show(ui, |ui| {
                                ui.label("Seed");

                                let mut seed_text =
                                    self.seed.map(|seed| seed.to_string()).unwrap_or_default();
                                if ui
                                    .add(egui::TextEdit::singleline(&mut seed_text).hint_text("0"))
                                    .changed()
                                {
                                    self.world = None;
                                }

                                if seed_text.is_empty() {
                                    self.seed = None;
                                } else if let Ok(seed) = seed_text.parse::<i32>() {
                                    self.seed = Some(seed);
                                }

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
            let start_time = Instant::now();

            let mut world: World =
                serde_json::from_str(include_str!("default-world.json")).unwrap();

            apply_randomization_settings(
                &mut world,
                self.seed.unwrap_or_default(),
                self.randomization_mode,
                self.purity_settings,
            );

            self.last_calc_duration = start_time.elapsed();
            world
        });

        egui::TopBottomPanel::bottom("status_panel").show(ctx, |ui| {
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

        egui::CentralPanel::default().show(ctx, |ui| {
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

                for resource in ResourceDescriptor::iter() {
                    let color = Self::get_resource_color(resource);

                    // resource nodes
                    for purity in ResourcePurity::iter() {
                        let points = Points::new(
                            format!("{} ({:?})", resource, purity),
                            world
                                .resource_nodes
                                .iter()
                                .filter(|n| n.resource == resource && n.purity == purity)
                                .map(|n| [n.location[0] as f64, n.location[1] as f64])
                                .collect::<Vec<_>>(),
                        )
                        .color(color)
                        .radius(base_size)
                        .filled(true)
                        .shape(match purity {
                            ResourcePurity::Impure => MarkerShape::Up,
                            ResourcePurity::Normal => MarkerShape::Diamond,
                            ResourcePurity::Pure => MarkerShape::Circle,
                        });

                        plot_ui.points(points);
                    }

                    // fracking cores
                    {
                        let points = Points::new(
                            format!("{} (Resource Well)", resource),
                            world
                                .fracking_cores
                                .iter()
                                .filter(|c| c.resource == resource)
                                .map(|n| [n.location[0] as f64, n.location[1] as f64])
                                .collect::<Vec<_>>(),
                        )
                        .color(color)
                        .filled(false)
                        .radius(1.5 * base_size)
                        .shape(MarkerShape::Circle);

                        plot_ui.points(points);
                    }

                    // fracking satellites
                    for purity in ResourcePurity::iter() {
                        let points = Points::new(
                            format!("{} (Resource Well)", resource),
                            world
                                .fracking_cores
                                .iter()
                                .filter(|c| c.resource == resource)
                                .flat_map(|c| c.satellites.iter())
                                .filter(|s| s.purity == purity)
                                .map(|n| [n.location[0] as f64, n.location[1] as f64])
                                .collect::<Vec<_>>(),
                        )
                        .color(color)
                        .radius(0.75 * base_size)
                        .filled(false)
                        .shape(match purity {
                            ResourcePurity::Impure => MarkerShape::Up,
                            ResourcePurity::Normal => MarkerShape::Diamond,
                            ResourcePurity::Pure => MarkerShape::Circle,
                        });

                        plot_ui.points(points);
                    }
                }

                // geysers
                {
                    let points = Points::new(
                        "Geyser",
                        world
                            .geysers
                            .iter()
                            .map(|n| [n.location[0] as f64, n.location[1] as f64])
                            .collect::<Vec<_>>(),
                    )
                    .color(Self::get_resource_color(ResourceDescriptor::Water))
                    .filled(false)
                    .radius(base_size)
                    .shape(MarkerShape::Asterisk);

                    plot_ui.points(points);
                }
            });
        });
    }
}
