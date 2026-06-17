use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use egui::{Align, Align2, Button, Id, Layout, Modal, Rect, RichText, TextureHandle, Vec2, vec2};
use egui_extras::{Column, TableBuilder};
use egui_plot::PlotPoint;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use url::Url;

use crate::{
    app::{
        constants::get_resource_color,
        outline::WorldOutline,
        plot_item::{ResourceDisplay, ResourceDisplayContent},
        textures::{get_geyser_texture, get_resource_texture, load_texture},
        view_options::{ViewOptions, ViewOptionsTarget},
    },
    game::{ResourceDescriptor, World},
    randomization::{NodePuritySettings, NodeRandomizationMode, apply_randomization_settings},
    stats::Stats,
};

#[derive(Serialize, Deserialize)]
struct QueryParams {
    seed: i32,
    mode: NodeRandomizationMode,
    purity: NodePuritySettings,
}

#[derive(PartialEq, Eq, Clone, Copy, strum::EnumIter, strum::Display)]
enum SidePanel {
    #[strum(to_string = "View Options")]
    ViewOptions,
    #[strum(to_string = "Statistics")]
    Stats,
}

pub struct App {
    seed: Option<i32>,
    randomization_mode: NodeRandomizationMode,
    purity_settings: NodePuritySettings,

    side_panel: SidePanel,

    world: Option<World>,
    stats: Stats,
    last_calc_duration: Duration,

    plot_id: egui::Id,
    view_options: ViewOptions,

    outline: WorldOutline,

    resource_rextures: HashMap<ResourceDescriptor, TextureHandle>,
    geyser_texture: Option<TextureHandle>,

    mobile_popup_open: bool,
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

            plot_id: egui::Id::new("map_display_plot"),
            view_options: ViewOptions::new(),

            outline: WorldOutline::new(),

            resource_rextures: HashMap::new(),
            geyser_texture: None,

            mobile_popup_open: false,
        }
    }
}

impl App {
    pub const PUBLIC_URL: Option<&'static str> = option_env!("PUBLIC_URL");

    pub fn new(cc: &eframe::CreationContext<'_>, startup_url: Option<&str>) -> Self {
        let mut app = Self::default();

        if let Some(params) = startup_url
            .and_then(|url| Url::parse(url).ok())
            .and_then(|url| serde_urlencoded::from_str::<QueryParams>(url.query()?).ok())
        {
            app.seed = Some(params.seed);
            app.randomization_mode = params.mode;
            app.purity_settings = params.purity;
        }

        for resource in ResourceDescriptor::iter() {
            let handle = load_texture(
                &cc.egui_ctx,
                resource.get_internal_name(),
                get_resource_texture(resource),
            )
            .expect("could not load bundled texture");

            app.resource_rextures.insert(resource, handle);
        }

        app.geyser_texture = Some(
            load_texture(
                &cc.egui_ctx,
                "Desc_GeneratorGeoThermal_C",
                get_geyser_texture(),
            )
            .expect("could not load bundled texture"),
        );

        app
    }

    pub const fn supports_share_link() -> bool {
        Self::PUBLIC_URL.is_some()
    }

    pub fn create_share_link(&self) -> Option<String> {
        let params = QueryParams {
            seed: self.seed.unwrap_or(0),
            mode: self.randomization_mode,
            purity: self.purity_settings,
        };
        let query_str = serde_urlencoded::to_string(params).ok()?;

        let mut url = Url::parse(Self::PUBLIC_URL?).ok()?;
        url.set_query(Some(&query_str));

        Some(url.to_string())
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

    fn stats_ui(&self, ui: &mut egui::Ui, enable_scrolling: bool) {
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
            .min_scrolled_height(if enable_scrolling {
                0.0
            } else {
                available_height
            })
            .max_scroll_height(available_height);

        table
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.strong("Resource");
                });

                for mk in Stats::MINER_MK_RANGE {
                    for speed in Stats::CLOCK_SPEEDS {
                        header.col(|ui| {
                            ui.strong(format!("Mk. {}\n{} %", mk, speed));
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
                                    .color(get_resource_color(resource, ui.visuals().dark_mode)),
                            );
                            ui.label(resource.to_string());
                        });

                        for mk in Stats::MINER_MK_RANGE {
                            for speed in Stats::CLOCK_SPEEDS {
                                let amount = self.stats.get(speed, mk, resource);

                                row.col(|ui| {
                                    ui.label(format!("{}", amount));
                                });
                            }
                        }
                    });
                }
            });
    }

    fn side_panel_ui(
        &mut self,
        ui: &mut egui::Ui,
        view_options_highlight: &mut Option<ViewOptionsTarget>,
        enable_scrolling: bool,
    ) {
        ui.horizontal(|ui| {
            SidePanel::iter().for_each(|v| {
                ui.selectable_value(
                    &mut self.side_panel,
                    v,
                    RichText::new(v.to_string()).heading(),
                );
            })
        });
        ui.separator();

        match self.side_panel {
            SidePanel::ViewOptions => {
                self.view_options
                    .ui(ui, view_options_highlight, enable_scrolling);
            }

            SidePanel::Stats => {
                self.stats_ui(ui, enable_scrolling);
            }
        }
    }

    fn randomization_settings_ui(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("settings_grid")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.label("Seed");

                    ui.with_layout(
                        Layout::right_to_left(egui::Align::Center).with_cross_justify(true),
                        |ui| {
                            let randomize_seed = ui.button("\u{1F3B2} random").clicked();
                            let mut seed_text =
                                self.seed.map(|seed| seed.to_string()).unwrap_or_default();
                            if ui
                                .add(
                                    egui::TextEdit::singleline(&mut seed_text)
                                        .hint_text("0")
                                        .desired_width(f32::INFINITY),
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
                    ui.end_row();

                    ui.label("Purity");
                    egui::ComboBox::from_id_salt("purity_setting")
                        .selected_text(self.purity_settings.to_string())
                        .show_ui(ui, |ui| {
                            NodePuritySettings::iter().for_each(|p| {
                                if ui
                                    .selectable_value(&mut self.purity_settings, p, p.to_string())
                                    .changed()
                                {
                                    self.world = None;
                                }
                            });
                        });
                    ui.end_row();
                });
        });

        if Self::supports_share_link() {
            ui.add_space(15.0);

            ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                if ui.button("\u{1F4CB} copy share url").clicked()
                    && let Some(link) = self.create_share_link()
                {
                    ui.copy_text(link);
                }
            });
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.global_style_mut(|style| style.interaction.selectable_labels = false);
        let is_mobile_ui = ui.content_rect().width() < 700.0;

        if !is_mobile_ui {
            egui::Panel::top("top_bar")
                .frame(egui::Frame::side_top_panel(ui.style()).inner_margin(4))
                .show_inside(ui, |ui| {
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        egui::widgets::global_theme_preference_buttons(ui);
                    });
                });
        }

        let mut view_options_highlight = None;

        if !is_mobile_ui {
            egui::Panel::right("sidebar_panel")
                .resizable(true)
                .min_size(400.0)
                .show_inside(ui, |ui| {
                    egui::Panel::bottom("settings_stats_panel")
                        .resizable(true)
                        .min_size(200.0)
                        .default_size(380.0)
                        .show_inside(ui, |ui| {
                            ui.take_available_space();
                            ui.add_space(5.0);

                            self.side_panel_ui(ui, &mut view_options_highlight, true);
                        });

                    egui::CentralPanel::default().show_inside(ui, |ui| {
                        ui.heading("Randomization Settings");
                        ui.add_space(5.0);

                        self.randomization_settings_ui(ui);
                    });
                });
        } else {
            egui::Panel::top("top_panel")
                .frame(egui::Frame::side_top_panel(ui.style()).inner_margin(8.0))
                .show_inside(ui, |ui| {
                    ui.heading("Randomization Settings");
                    ui.add_space(5.0);

                    self.randomization_settings_ui(ui);
                });

            let popup_id = Id::new("modal_test");
            let animation = ui
                .ctx()
                .animate_bool_responsive(popup_id.with("animation"), self.mobile_popup_open);

            if animation > 0.0 {
                let modal = Modal::new(popup_id)
                    .area(
                        Modal::default_area(popup_id)
                            .anchor(Align2::CENTER_BOTTOM, vec2(0.0, 400.0 * (1.0 - animation)))
                            .constrain(false)
                            .fade_in(false)
                            .layout(Layout::default().with_cross_justify(true))
                            .default_size(ui.content_rect().size()),
                    )
                    .show(ui.ctx(), |ui| {
                        ui.multiply_opacity(animation);
                        self.side_panel_ui(ui, &mut view_options_highlight, false);
                    });

                if modal.should_close() {
                    self.mobile_popup_open = false;
                }
            }
        }

        let world = self.world.get_or_insert_with(|| {
            let start_time = Self::get_time();

            let mut world: World =
                serde_json::from_str(include_str!("../resources/default-world.json")).unwrap();

            apply_randomization_settings(
                &mut world,
                self.seed.unwrap_or_default(),
                self.randomization_mode,
                self.purity_settings,
            );
            self.stats.compute(&world);
            self.view_options.get_existing_nodes(&world);

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
                .legend(
                    egui_plot::Legend::default().hidden_items(self.view_options.get_hidden_items()),
                )
                .show_axes(true)
                .show_grid(true)
                .data_aspect(1.0)
                .invert_y(true)
                .id(self.plot_id);

            let is_dark_mode = ui.visuals().dark_mode;

            plot.show(ui, |plot_ui| {
                plot_ui.add(self.outline.plot_item());

                let test_rect = plot_ui
                    .transform()
                    .rect_from_values(&PlotPoint::new(0.0, 0.0), &PlotPoint::new(1.0, 1.0));
                let scale = 5000.0 * (test_rect.width() + test_rect.height()) / 2.0;

                let base_size = match self.view_options.get_node_style() {
                    super::view_options::ResourceNodeStyle::Shapes => scale.clamp(5.0, 20.0),
                    super::view_options::ResourceNodeStyle::IconsPurityColors => {
                        (scale).clamp(15.0, 25.0)
                    }
                };

                // resource nodes
                for (resource, nodes) in &world.resource_nodes.iter().chunk_by(|n| n.resource) {
                    plot_ui.add(ResourceDisplay::new(
                        base_size,
                        ResourceDisplayContent::ResourceNodes(resource, nodes.collect()),
                        &self.view_options,
                        view_options_highlight,
                        self.resource_rextures.get(&resource).map(|h| h.id()),
                        is_dark_mode,
                    ));
                }

                // fracking nodes
                for (resource, cores) in &world.fracking_cores.iter().chunk_by(|c| c.resource) {
                    plot_ui.add(ResourceDisplay::new(
                        base_size,
                        ResourceDisplayContent::FrackingNodes(resource, cores.collect()),
                        &self.view_options,
                        view_options_highlight,
                        self.resource_rextures.get(&resource).map(|h| h.id()),
                        is_dark_mode,
                    ));
                }

                // geysers
                plot_ui.add(ResourceDisplay::new(
                    base_size,
                    ResourceDisplayContent::Geysers(world.geysers.iter().by_ref().collect()),
                    &self.view_options,
                    view_options_highlight,
                    self.geyser_texture.as_ref().map(|h| h.id()),
                    is_dark_mode,
                ));
            });

            self.view_options.apply_legend_interaction(ui, self.plot_id);

            if is_mobile_ui {
                let fab_size = Vec2::splat(50.0);
                let bottom_right = ui.max_rect().max - vec2(10.0, 25.0);
                let fab_rect = Rect::from_min_max(bottom_right - fab_size, bottom_right);

                if ui
                    .put(
                        fab_rect,
                        Button::new(RichText::new("\u{23f6}").size(30.0)).corner_radius(u8::MAX),
                    )
                    .clicked()
                {
                    self.mobile_popup_open = true;
                }
            }
        });
    }
}
