#![warn(clippy::all, rust_2018_idioms)]

mod app;
pub use app::CeBrAEfficiencyApp;

mod efficiency_fitter;
mod egui_plot_stuff;
