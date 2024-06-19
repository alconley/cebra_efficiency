use super::detector::Detector;
use super::exp_fitter::Fitter;
use super::gamma_source::GammaSource;

use std::collections::{HashMap, HashSet};

use egui_plot::Plot;

use crate::egui_plot_stuff::plot_settings::EguiPlotSettings;

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct Measurement {
    pub gamma_source: GammaSource,
    pub detectors: Vec<Detector>,
}

impl Measurement {
    pub fn new(source: Option<GammaSource>) -> Self {
        Self {
            gamma_source: source.unwrap_or_default(),
            detectors: vec![],
        }
    }

    pub fn measurement_ui(&mut self, ui: &mut egui::Ui) {
        ui.collapsing("Measurement", |ui: &mut egui::Ui| {
            // ensure that there are gamma lines to display
            if self.gamma_source.gamma_lines.is_empty() {
                ui.label("No gamma lines added to source");
                return;
            }

            let mut index_to_remove = None;

            for (index, detector) in &mut self.detectors.iter_mut().enumerate() {
                detector.ui(ui, &self.gamma_source);

                if detector.to_remove == Some(true) {
                    index_to_remove = Some(index);
                }
            }

            ui.separator();

            if ui.button("Add Detector").clicked() {
                self.detectors.push(Detector::default());
            }

            if let Some(index) = index_to_remove {
                self.detectors.remove(index);
            }

            ui.separator();
        });
    }

    pub fn update_ui(&mut self, ui: &mut egui::Ui) {
        ui.collapsing(format!("{} Measurement", self.gamma_source.name), |ui| {
            self.gamma_source.source_ui(ui);
            self.measurement_ui(ui);
        });
    }

    pub fn draw(&mut self, plot_ui: &mut egui_plot::PlotUi) {
        for detector in self.detectors.iter_mut() {
            detector.points.name = format!("{}: {}", detector.name, self.gamma_source.name);
            detector.draw(plot_ui);
        }
    }

    pub fn menu_button(&mut self, ui: &mut egui::Ui) {
        ui.menu_button(format!("{} Measurement", self.gamma_source.name), |ui| {
            for detector in self.detectors.iter_mut() {
                detector.menu_button(ui);
            }
        });
    }
}

#[derive(Default, Clone, serde::Deserialize, serde::Serialize)]
pub struct MeasurementHandler {
    pub measurements: Vec<Measurement>,
    pub measurement_exp_fits: HashMap<String, Fitter>,
    pub plot_settings: EguiPlotSettings,
}

impl MeasurementHandler {
    pub fn new() -> Self {
        Self {
            measurements: vec![],
            measurement_exp_fits: HashMap::new(),
            plot_settings: EguiPlotSettings::default(),
        }
    }

    fn synchronize_detectors(&mut self) {
        let mut detector_names: HashSet<String> = HashSet::new();
        #[allow(clippy::type_complexity)]
        let mut detector_data: HashMap<String, (Vec<f64>, Vec<f64>, Vec<f64>)> = HashMap::new();

        // Collect all detector names from measurements and compute data
        for measurement in &self.measurements {
            for detector in &measurement.detectors {
                let name = &detector.name;
                detector_names.insert(name.clone());
                let data = self.get_detector_data_from_measurements(name.clone());
                detector_data.insert(name.clone(), data);
            }
        }

        // Iterate over detector names
        for name in &detector_names {
            // Insert if not exists
            self.measurement_exp_fits.entry(name.clone()).or_default();

            // Update Fitter with pre-computed data
            if let Some(fitter) = self.measurement_exp_fits.get_mut(name) {
                if let Some(data) = detector_data.get(name) {
                    fitter.name.clone_from(name);
                    fitter.data = data.clone();
                }
            }
        }

        // Remove entries in the hashmap that don't correspond to any detector in measurements
        let keys: HashSet<String> = self.measurement_exp_fits.keys().cloned().collect();
        for key in keys {
            if !detector_names.contains(&key) {
                self.measurement_exp_fits.remove(&key);
            }
        }
    }

    fn get_detector_data_from_measurements(&self, name: String) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
        let mut x_data: Vec<f64> = vec![];
        let mut y_data: Vec<f64> = vec![];
        let mut weights: Vec<f64> = vec![];

        for measurement in &self.measurements {
            for detector in &measurement.detectors {
                if detector.name == name {
                    for line in &detector.lines {
                        x_data.push(line.energy);
                        y_data.push(line.efficiency);
                        weights.push(1.0 / line.efficiency_uncertainty);
                    }
                }
            }
        }

        (x_data, y_data, weights)
    }

    fn fit_detectors_ui(&mut self, ui: &mut egui::Ui) {
        self.synchronize_detectors(); // Ensure synchronization before fitting UI

        ui.label("Fit Equation: y = a * exp[-x/b] + c * exp[-x/d]");

        egui::ScrollArea::both().show(ui, |ui| {
            ui.separator();

            egui::Grid::new("detector_grid")
                .striped(true)
                .show(ui, |ui| {
                    ui.label("Detector Name");

                    ui.label("Initial Guesses");

                    ui.label("Exponential Fitter");

                    ui.label("Results");
                    ui.label("a");
                    ui.label("b");
                    ui.label("c");
                    ui.label("d");

                    ui.end_row();

                    for (name, fitter) in &mut self.measurement_exp_fits {
                        fitter.name.clone_from(name);
                        fitter.ui(ui);
                        ui.end_row();
                    }
                });
        });
    }

    fn remove_measurement(&mut self, index: usize) {
        self.measurements.remove(index);
    }

    fn context_menu(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.heading("Context Menu");

            self.plot_settings.menu_button(ui);

            ui.separator();

            for measurement in self.measurements.iter_mut() {
                measurement.menu_button(ui);
            }

            ui.separator();

            for (name, fitter) in self.measurement_exp_fits.iter_mut() {
                ui.collapsing(format!("{} Fitter", name), |ui| {
                    fitter.menu_button(ui);
                });
            }
        });
    }

    fn draw(&mut self, plot_ui: &mut egui_plot::PlotUi) {
        for measurement in self.measurements.iter_mut() {
            measurement.draw(plot_ui);
        }

        for (name, fitter) in self.measurement_exp_fits.iter_mut() {
            fitter.name.clone_from(name);
            fitter.draw(plot_ui);
        }
    }

    pub fn plot(&mut self, ui: &mut egui::Ui) {
        let mut plot = Plot::new("Efficiency")
            .min_size(egui::Vec2::new(400.0, 400.0))
            .auto_bounds(egui::Vec2b::new(true, true));

        plot = self.plot_settings.apply_to_plot(plot);

        plot.show(ui, |plot_ui| {
            self.draw(plot_ui);
        })
        .response
        .context_menu(|ui| {
            self.context_menu(ui);
        });
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, show_bottom_panel: bool, show_left_panel: bool) {
        egui::TopBottomPanel::bottom("efficiency_bottom")
            .resizable(true)
            .show_animated_inside(ui, show_bottom_panel, |ui| {
                self.fit_detectors_ui(ui);
            });

        egui::SidePanel::left("cebra_efficiency_left_side_panel").show_animated_inside(
            ui,
            show_left_panel,
            |ui| {
                let mut index_to_remove: Option<usize> = None;

                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.collapsing("Sources", |ui| {
                        for (index, measurement) in self.measurements.iter_mut().enumerate() {
                            measurement.update_ui(ui);

                            if ui.button("Remove Source").clicked() {
                                index_to_remove = Some(index);
                            }

                            ui.separator();
                        }

                        if let Some(index) = index_to_remove {
                            self.remove_measurement(index);
                        }

                        if ui.button("New Source").clicked() {
                            self.measurements.push(Measurement::new(None));
                        }

                        ui.separator();
                    });
                });
            },
        );

        self.plot(ui);
    }
}
