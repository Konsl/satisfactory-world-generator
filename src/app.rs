use std::{
    collections::HashMap,
    ops::RangeInclusive,
    time::{Duration, Instant},
};

use egui::{
    Color32, Layout, PopupAnchor, Pos2, RichText, Shape, Stroke, epaint::CircleShape, vec2,
};
use egui_extras::{Column, TableBuilder};
use egui_plot::{
    Cursor, LabelFormatter, MarkerShape, PlotBounds, PlotGeometry, PlotItem, PlotItemBase,
    PlotPoint, PlotTransform,
};
use itertools::Itertools;
use strum::IntoEnumIterator;

use crate::{
    game::{FrackingCore, GeyserNode, ResourceDescriptor, ResourceNode, ResourcePurity, World},
    randomization::{NodePuritySettings, NodeRandomizationMode, apply_randomization_settings},
};

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
                serde_json::from_str(include_str!("default-world.json")).unwrap();

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

fn get_resource_color(resource: ResourceDescriptor) -> Color32 {
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

fn get_purity_marker(purity: ResourcePurity) -> MarkerShape {
    match purity {
        ResourcePurity::Impure => MarkerShape::Up,
        ResourcePurity::Normal => MarkerShape::Diamond,
        ResourcePurity::Pure => MarkerShape::Circle,
    }
}

enum ResourceDisplayContent<'a> {
    ResourceNodes(ResourceDescriptor, Vec<&'a ResourceNode>),
    FrackingNodes(ResourceDescriptor, Vec<&'a FrackingCore>),
    Geysers(Vec<&'a GeyserNode>),
}

impl<'a> ResourceDisplayContent<'a> {
    pub fn get_color(&self) -> Color32 {
        match self {
            Self::ResourceNodes(resource, _) | Self::FrackingNodes(resource, _) => {
                get_resource_color(*resource)
            }
            Self::Geysers(_) => get_resource_color(ResourceDescriptor::Water),
        }
    }

    fn convert_location(location: [f32; 3]) -> PlotPoint {
        PlotPoint::new(location[0] as f64, location[1] as f64)
    }

    pub fn get_points(&self) -> Vec<PlotPoint> {
        match self {
            Self::ResourceNodes(_, nodes) => nodes
                .iter()
                .map(|n| Self::convert_location(n.location))
                .collect(),

            Self::FrackingNodes(_, cores) => cores
                .iter()
                .flat_map(|c| {
                    let mut points = Vec::with_capacity(1 + c.satellites.len());
                    points.push(Self::convert_location(c.location));

                    for s in &c.satellites {
                        points.push(Self::convert_location(s.location));
                    }

                    points
                })
                .collect(),

            Self::Geysers(geysers) => geysers
                .iter()
                .map(|g| Self::convert_location(g.location))
                .collect(),
        }
    }
}

struct ResourceDisplay<'a> {
    base: PlotItemBase,
    geometry_points: Vec<PlotPoint>,

    marker_base_size: f32,
    content: ResourceDisplayContent<'a>,
}

impl<'a> ResourceDisplay<'a> {
    pub fn new(marker_base_size: f32, content: ResourceDisplayContent<'a>) -> Self {
        let name = match content {
            ResourceDisplayContent::ResourceNodes(resource, _)
            | ResourceDisplayContent::FrackingNodes(resource, _) => resource.to_string(),
            ResourceDisplayContent::Geysers(_) => "Geyser".to_owned(),
        };

        Self {
            base: PlotItemBase::new(name),
            geometry_points: content.get_points(),

            marker_base_size,
            content,
        }
    }

    fn marker_shape(
        shape: MarkerShape,
        center: Pos2,
        radius: f32,
        color: Color32,
        filled: bool,
        shapes: &mut Vec<Shape>,
    ) {
        let sqrt_3 = 3_f32.sqrt();
        let frac_sqrt_3_2 = 3_f32.sqrt() / 2.0;

        let (fill, stroke) = if filled {
            (color, Stroke::NONE)
        } else {
            (Color32::TRANSPARENT, Stroke::new(radius / 5.0, color))
        };

        let tf = |dx: f32, dy: f32| -> Pos2 { center + radius * vec2(dx, dy) };

        match shape {
            MarkerShape::Up => {
                let points = vec![tf(0.0, -1.0), tf(0.5 * sqrt_3, 0.5), tf(-0.5 * sqrt_3, 0.5)];
                shapes.push(Shape::convex_polygon(points, fill, stroke));
            }
            MarkerShape::Diamond => {
                let points = vec![
                    tf(0.0, 1.0),  // bottom
                    tf(-1.0, 0.0), // left
                    tf(0.0, -1.0), // top
                    tf(1.0, 0.0),  // right
                ];
                shapes.push(Shape::convex_polygon(points, fill, stroke));
            }
            MarkerShape::Circle => {
                shapes.push(Shape::Circle(CircleShape {
                    center,
                    radius,
                    fill,
                    stroke,
                }));
            }
            MarkerShape::Asterisk => {
                let vertical = [tf(0.0, -1.0), tf(0.0, 1.0)];
                let diagonal1 = [tf(-frac_sqrt_3_2, 0.5), tf(frac_sqrt_3_2, -0.5)];
                let diagonal2 = [tf(-frac_sqrt_3_2, -0.5), tf(frac_sqrt_3_2, 0.5)];
                shapes.push(Shape::line_segment(vertical, stroke));
                shapes.push(Shape::line_segment(diagonal1, stroke));
                shapes.push(Shape::line_segment(diagonal2, stroke));
            }
            _ => (),
        }
    }
}

impl<'a> PlotItem for ResourceDisplay<'a> {
    fn shapes(&self, _ui: &egui::Ui, transform: &PlotTransform, shapes: &mut Vec<Shape>) {
        let scale = if self.highlighted() { 2f32.sqrt() } else { 1.0 };
        let color = self.color();

        match &self.content {
            ResourceDisplayContent::ResourceNodes(_, nodes) => {
                for node in nodes {
                    let center = transform.position_from_point(
                        &ResourceDisplayContent::convert_location(node.location),
                    );
                    Self::marker_shape(
                        get_purity_marker(node.purity),
                        center,
                        self.marker_base_size * scale,
                        color,
                        true,
                        shapes,
                    );
                }
            }

            ResourceDisplayContent::FrackingNodes(_, cores) => {
                for core in cores {
                    let center = transform.position_from_point(
                        &ResourceDisplayContent::convert_location(core.location),
                    );

                    Self::marker_shape(
                        MarkerShape::Circle,
                        center,
                        1.5 * self.marker_base_size * scale,
                        color,
                        false,
                        shapes,
                    );

                    for satellite in &core.satellites {
                        let center = transform.position_from_point(
                            &ResourceDisplayContent::convert_location(satellite.location),
                        );

                        Self::marker_shape(
                            get_purity_marker(satellite.purity),
                            center,
                            0.75 * self.marker_base_size * scale,
                            color,
                            false,
                            shapes,
                        );
                    }
                }
            }

            ResourceDisplayContent::Geysers(geysers) => {
                for geyser in geysers {
                    let center = transform.position_from_point(
                        &ResourceDisplayContent::convert_location(geyser.location),
                    );

                    Self::marker_shape(
                        MarkerShape::Asterisk,
                        center,
                        self.marker_base_size * scale,
                        color,
                        false,
                        shapes,
                    );
                }
            }
        }
    }

    fn initialize(&mut self, _x_range: RangeInclusive<f64>) {}

    fn color(&self) -> Color32 {
        self.content.get_color()
    }

    fn geometry(&self) -> PlotGeometry<'_> {
        PlotGeometry::Points(&self.geometry_points)
    }

    fn bounds(&self) -> PlotBounds {
        let mut bounds = PlotBounds::NOTHING;
        for p in &self.geometry_points {
            bounds.extend_with(p);
        }

        bounds
    }

    fn base(&self) -> &PlotItemBase {
        &self.base
    }

    fn base_mut(&mut self) -> &mut PlotItemBase {
        &mut self.base
    }

    fn on_hover(
        &self,
        plot_area_response: &egui::Response,
        elem: egui_plot::ClosestElem,
        shapes: &mut Vec<Shape>,
        cursors: &mut Vec<egui_plot::Cursor>,
        plot: &egui_plot::PlotConfig<'_>,
        _label_formatter: &Option<LabelFormatter<'_>>,
    ) {
        let line_color = if plot.ui.visuals().dark_mode {
            Color32::from_gray(100).additive()
        } else {
            Color32::from_black_alpha(180)
        };

        let value = self.geometry_points[elem.index];
        let pointer = plot.transform.position_from_point(&value);
        shapes.push(Shape::circle_filled(pointer, 3.0, line_color));

        cursors.push(Cursor::Vertical { x: value.x });
        cursors.push(Cursor::Horizontal { y: value.y });

        let mut tooltip = egui::Tooltip::always_open(
            plot_area_response.ctx.clone(),
            plot_area_response.layer_id,
            plot_area_response.id,
            PopupAnchor::Pointer,
        );

        let tooltip_width = plot_area_response.ctx.global_style().spacing.tooltip_width;

        tooltip.popup = tooltip.popup.width(tooltip_width);

        tooltip.gap(12.0).show(|ui| {
            ui.set_max_width(tooltip_width);

            let location = match &self.content {
                ResourceDisplayContent::ResourceNodes(_, nodes) => {
                    let node = nodes[elem.index];

                    ui.label(format!("{} ({:?})", node.resource, node.purity));
                    node.location
                }
                ResourceDisplayContent::FrackingNodes(_, cores) => {
                    let mut index = elem.index;

                    let mut location = [0f32; 3];
                    for core in cores {
                        if index == 0 {
                            ui.label(format!("{} (Resource Well)", core.resource));

                            location = core.location;
                            break;
                        }

                        index -= 1;

                        if index < core.satellites.len() {
                            let satellite = &core.satellites[index];
                            ui.label(format!(
                                "{} ({:?} Resource Well)",
                                core.resource, satellite.purity
                            ));

                            location = satellite.location;
                            break;
                        }

                        index -= core.satellites.len();
                    }

                    location
                }
                ResourceDisplayContent::Geysers(geysers) => {
                    let geyser = geysers[elem.index];

                    ui.label(format!("Geyser ({:?})", geyser.purity));
                    geyser.location
                }
            };

            ui.label(format!(
                "x = {:.1}\ny = {:.1}\nz = {:.1}",
                location[0], location[1], location[2],
            ));
        });
    }
}
