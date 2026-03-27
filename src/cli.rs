use clap::Parser;
use itertools::Itertools;
use log::info;
use plotlib::page::Page;
use plotlib::repr::Plot;
use plotlib::style::{PointMarker, PointStyle};
use plotlib::view::{ContinuousView};
use crate::game::{ResourceDescriptor, ResourcePurity, World};
use crate::randomization::{apply_randomization_settings, NodePuritySettings, NodeRandomizationMode};

pub fn run() {
    let args = Args::parse();

    let mut world: World = serde_json::from_str(include_str!("default-world.json")).unwrap();
    apply_randomization_settings(&mut world, args.seed, args.random_mode, args.purity_settings);

    if args.generate_svg{
        info!("Generating SVG");
        let mut view = ContinuousView::new();
        let plots: Vec<Plot> = world.resource_nodes.iter()
            .chunk_by(|rn| rn.resource)
            .into_iter()
            .flat_map(|(resource_type, nodes)| {
                nodes.chunk_by(|rn| rn.purity)
                    .into_iter()
                    .map(|(purity, nodes2)| {
                        Plot::new(nodes2.map(|rn| (rn.location[0] as f64, -rn.location[1] as f64)).collect())
                            .point_style(get_point_style(&resource_type, &purity))
                    }).collect::<Vec<_>>()
            }).collect();
        for p in plots {
            view = view.add(p);
        }
        // TODO add geysers and resource wells
        Page::single(&view).dimensions(6000, 4000).save("output-world.svg").unwrap();
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args{
    #[arg(short, long)]
    seed: i32,

    #[arg(value_enum, short, long, default_value_t = NodeRandomizationMode::None)]
    random_mode: NodeRandomizationMode,

    #[arg(value_enum, short, long, default_value_t = NodePuritySettings::NoChange)]
    purity_settings: NodePuritySettings,

    #[arg(long, default_value_t = false)]
    generate_svg: bool,
}

fn get_point_style(resource_descriptor: &ResourceDescriptor, resource_purity: &ResourcePurity) -> PointStyle{
    let color = resource_descriptor.get_color();
    // FIXME figure out how to do custom point markers to allow for more diversity
    let icon = match resource_purity {
        ResourcePurity::Impure => PointMarker::Cross,
        ResourcePurity::Normal => PointMarker::Square,
        ResourcePurity::Pure => PointMarker::Circle,
    };
    PointStyle::new().marker(icon).colour(color).size(10.0)
}
