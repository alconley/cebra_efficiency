use super::detector::Detector;
use super::exp_fitter::Fitter;
use super::gamma_source::GammaSource;

use std::collections::{HashMap, HashSet};

use egui_plot::{Legend, Line, Plot, PlotPoints, Points};

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct Measurement {
    pub gamma_source: GammaSource,
    pub detectors: Vec<Detector>,
}

impl Measurement {
    pub fn new(source: Option<GammaSource>) -> Self {
        Self {
            gamma_source: source.unwrap_or(GammaSource::new()),
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
}

#[derive(Default, Clone, serde::Deserialize, serde::Serialize)]
pub struct MeasurementHandler {
    pub measurements: Vec<Measurement>,
    pub measurement_exp_fits: HashMap<String, Fitter>,
}

impl MeasurementHandler {
    pub fn new() -> Self {
        Self {
            measurements: vec![],
            measurement_exp_fits: HashMap::new(),
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
            self.measurement_exp_fits
                .entry(name.clone())
                .or_insert(Fitter::default());

            // Update Fitter with pre-computed data
            if let Some(fitter) = self.measurement_exp_fits.get_mut(name) {
                if let Some(data) = detector_data.get(name) {
                    fitter.name = name.clone();
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
                        fitter.name = name.clone();
                        fitter.ui(ui);
                        ui.end_row();
                    }
                });
        });
    }

    fn remove_measurement(&mut self, index: usize) {
        self.measurements.remove(index);
    }

    pub fn plot(&mut self, ui: &mut egui::Ui) {
        let plot = Plot::new("Efficiency")
            .legend(Legend::default())
            .min_size(egui::Vec2::new(400.0, 400.0));

        plot.show(ui, |plot_ui| {
            for measurement in self.measurements.iter_mut() {
                let marker_shape = measurement.gamma_source.to_egui_marker_shape();

                let marker_size = measurement.gamma_source.marker_size;

                for detector in measurement.detectors.iter_mut() {
                    let color = detector.color;
                    let name = format!("{}: {}", detector.name, measurement.gamma_source.name);

                    let mut points: Vec<[f64; 2]> = vec![];
                    for detector_line in &detector.lines {
                        points.push([detector_line.energy, detector_line.efficiency]);
                    }

                    let detector_plot_points = PlotPoints::new(points);

                    let detector_points = Points::new(detector_plot_points)
                        .filled(true)
                        .color(color)
                        .shape(marker_shape)
                        .radius(marker_size)
                        .name(name.to_string());

                    plot_ui.points(detector_points);

                    // draw the uncertainity as vertical lines from the efficiency points
                    for detector_line in &detector.lines {
                        let y_err_points: Vec<[f64; 2]> = vec![
                            [
                                detector_line.energy,
                                detector_line.efficiency - detector_line.efficiency_uncertainty,
                            ],
                            [
                                detector_line.energy,
                                detector_line.efficiency + detector_line.efficiency_uncertainty,
                            ],
                        ];

                        let y_err_plot_points = PlotPoints::new(y_err_points);

                        // this can be a line with the points at (x, y-yerr) and (x, y+yerr)
                        let y_err_points = Line::new(y_err_plot_points)
                            .color(color)
                            .name(name.to_string());

                        plot_ui.line(y_err_points);
                    }
                }
            }

            for (name, fitter) in self.measurement_exp_fits.iter_mut() {
                // check to see if the exp is not none
                if let Some(exp_fit) = &mut fitter.exp_fitter {
                    exp_fit.draw_fit_line(plot_ui, name.clone());
                }
            }
        });
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        egui::TopBottomPanel::bottom("efficiency_bottom").show_inside(ui, |ui| {
            self.fit_detectors_ui(ui);
        });

        egui::SidePanel::left("cebra_efficiency_left_side_panel").show_inside(ui, |ui| {
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
        });

        self.plot(ui);
    }
}
