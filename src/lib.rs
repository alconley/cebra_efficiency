#![warn(clippy::all, rust_2018_idioms)]

mod app;
pub use app::CeBrAEfficiencyApp;

mod detector;
mod exp_fitter;
mod gamma_line;
mod gamma_source;
mod measurements;
