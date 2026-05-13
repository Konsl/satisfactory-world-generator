use std::ops::RangeInclusive;

use egui::{Color32, Shape, Stroke, Ui, lerp};
use egui_plot::{PlotBounds, PlotGeometry, PlotItem, PlotItemBase, PlotPoint, PlotTransform};
use itertools::Itertools;

pub struct WorldOutline {
    lines: Vec<Vec<PlotPoint>>,
    bounds: PlotBounds,
}

impl WorldOutline {
    pub fn new() -> Self {
        const WORLD_REGION: [[f64; 2]; 2] = [[-324698.16, -375000.0], [425301.8, 375000.0]];

        let data: Vec<Vec<[f64; 2]>> =
            serde_json::from_str(include_str!("../world-outline.json")).unwrap();
        let data = data
            .into_iter()
            .map(|line| {
                line.into_iter()
                    .map(|p| {
                        PlotPoint::new(
                            lerp(WORLD_REGION[0][0]..=WORLD_REGION[1][0], p[0]),
                            lerp(WORLD_REGION[0][1]..=WORLD_REGION[1][1], p[1]),
                        )
                    })
                    .collect_vec()
            })
            .collect_vec();

        let mut bounds = PlotBounds::NOTHING;
        for line in data.iter() {
            for point in line.iter() {
                bounds.extend_with(point);
            }
        }

        Self {
            lines: data,
            bounds,
        }
    }

    pub fn plot_item<'a>(&'a self) -> WorldOutlinePlotItem<'a> {
        WorldOutlinePlotItem {
            base: PlotItemBase::new("World Outline".to_owned()),
            data: self,
        }
    }
}

pub struct WorldOutlinePlotItem<'a> {
    base: PlotItemBase,
    data: &'a WorldOutline,
}

impl<'a> PlotItem for WorldOutlinePlotItem<'a> {
    fn shapes(&self, ui: &Ui, transform: &PlotTransform, shapes: &mut Vec<Shape>) {
        let color = ui.visuals().text_color();
        let stroke = Stroke::new(1.5, color);

        for line in self.data.lines.iter() {
            let mut points = line
                .iter()
                .map(|p| transform.position_from_point(p))
                .collect_vec();

            if let Some(first_point) = points.first() {
                points.push(*first_point);
            }

            shapes.push(Shape::line(points, stroke));
        }
    }

    fn initialize(&mut self, _x_range: RangeInclusive<f64>) {}

    fn color(&self) -> Color32 {
        Color32::WHITE
    }

    fn geometry(&self) -> PlotGeometry<'_> {
        PlotGeometry::None
    }

    fn bounds(&self) -> PlotBounds {
        self.data.bounds
    }

    fn base(&self) -> &PlotItemBase {
        &self.base
    }

    fn base_mut(&mut self) -> &mut PlotItemBase {
        &mut self.base
    }

    fn allow_hover(&self) -> bool {
        false
    }
}
