#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::app::run_app;

mod app;
mod game;
mod random_stream;
mod randomization;
mod stats;

fn main() {
    run_app().unwrap();
}
