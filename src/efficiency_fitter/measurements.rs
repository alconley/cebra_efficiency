use super::detector::Detector;
use super::exp_fitter::Fitter;
use super::gamma_source::GammaSource;

use std::collections::{HashMap, HashSet};

use egui_plot::Plot;

use crate::egui_plot_stuff::{egui_line::EguiLine, plot_settings::EguiPlotSettings};

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
        egui::CollapsingHeader::new("Measurement")
            .id_source(format!("{} Measurement", self.gamma_source.name))
            .default_open(true)
            .show(ui, |ui| {
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

    pub fn update_ui(&mut self, ui: &mut egui::Ui, index: usize) {
        egui::CollapsingHeader::new(format!("{} Measurement", self.gamma_source.name))
            .id_source(index)
            .default_open(true)
            .show(ui, |ui| {
                self.gamma_source.source_ui(ui);
                self.measurement_ui(ui);
            });
    }

    pub fn draw(&mut self, plot_ui: &mut egui_plot::PlotUi) {
        for detector in self.detectors.iter_mut() {
            let name = format!("{}: {}", detector.name, self.gamma_source.name);
            detector.points.name.clone_from(&name);
            detector.draw(plot_ui, Some(name));
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

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct SummedEfficiency {
    pub line: EguiLine,
    pub uncertainty: Vec<f64>,
    pub uncertainty_lower_points: Vec<[f64; 2]>,
    pub uncertainty_upper_points: Vec<[f64; 2]>,
    pub max_energy: f64,
}

impl SummedEfficiency {
    pub fn new() -> Self {
        let mut line = EguiLine::new(egui::Color32::RED);
        line.name = "Summed".to_string();

        Self {
            line,
            uncertainty: vec![],
            uncertainty_lower_points: vec![],
            uncertainty_upper_points: vec![],
            max_energy: 0.0,
        }
    }

    pub fn draw(&mut self, plot_ui: &mut egui_plot::PlotUi) {
        self.line.draw(plot_ui);

        if self.line.draw {
            let upper_uncertainity_plot_points: Vec<egui_plot::PlotPoint> = self
                .uncertainty_upper_points
                .iter()
                .map(|[x, y]| egui_plot::PlotPoint::new(*x, *y))
                .collect();
            let lower_uncertainity_plot_points: Vec<egui_plot::PlotPoint> = self
                .uncertainty_lower_points
                .iter()
                .map(|[x, y]| egui_plot::PlotPoint::new(*x, *y))
                .collect();

            // check is number of points is the greater than 4
            if upper_uncertainity_plot_points.len() < 2 {
                return;
            }

            let num_points = upper_uncertainity_plot_points.len() - 1;
            let mut polygons: Vec<Vec<egui_plot::PlotPoint>> = Vec::new();
            for i in 0..num_points {
                let polygon = vec![
                    upper_uncertainity_plot_points[i],
                    upper_uncertainity_plot_points[i + 1],
                    lower_uncertainity_plot_points[i + 1],
                    lower_uncertainity_plot_points[i],
                ];
                polygons.push(polygon);
            }

            for points in polygons.iter() {
                let uncertainity_band =
                    egui_plot::Polygon::new(egui_plot::PlotPoints::Owned(points.clone()))
                        .stroke(egui::Stroke::new(0.0, self.line.color))
                        .highlight(false)
                        .width(0.0)
                        .name(self.line.name.clone());

                plot_ui.polygon(uncertainity_band);
            }
        }
    }

    pub fn csv_points(&self) -> String {
        let mut csv = String::new();

        csv.push_str("Energy, Efficiency, Uncertainity\n");
        for (index, point) in self.line.points.iter().enumerate() {
            csv.push_str(&format!(
                "{}, {}, {}\n",
                point[0], point[1], self.uncertainty[index]
            ));
        }

        csv
    }
}

#[derive(Default, Clone, serde::Deserialize, serde::Serialize)]
pub struct MeasurementHandler {
    pub measurements: Vec<Measurement>,
    pub measurement_exp_fits: HashMap<String, Fitter>,
    pub plot_settings: EguiPlotSettings,
    pub summed_efficiency: Option<SummedEfficiency>,
}

impl MeasurementHandler {
    pub fn new() -> Self {
        Self {
            measurements: vec![],
            measurement_exp_fits: HashMap::new(),
            plot_settings: EguiPlotSettings::default(),
            summed_efficiency: None,
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
            ui.heading("Efficiency Menu");

            self.plot_settings.menu_button(ui);

            ui.separator();

            ui.heading("Measurements");
            for measurement in self.measurements.iter_mut() {
                measurement.menu_button(ui);
            }

            ui.separator();

            ui.heading("Fits");
            for (name, fitter) in self.measurement_exp_fits.iter_mut() {
                ui.collapsing(format!("{} Fitter", name), |ui| {
                    fitter.menu_button(ui);
                });
            }

            ui.separator();

            ui.heading("Summed Efficiency");
            if self.summed_efficiency.is_none() && ui.button("Add Summed Line").clicked() {
                self.summed_efficiency = Some(SummedEfficiency::new());
            }

            if let Some(summed_efficiency) = &mut self.summed_efficiency {
                if ui.button("Sum Efficiency Fits").clicked() {
                    let max_range = summed_efficiency.max_energy;
                    self.get_summed_efficiency(max_range);
                }
            }

            if let Some(summed_efficiency) = &mut self.summed_efficiency {
                ui.add(
                    egui::DragValue::new(&mut summed_efficiency.max_energy)
                        .speed(1.0)
                        .clamp_range(0.0..=10000.0)
                        .prefix("Max Energy: ")
                        .suffix(" keV"),
                );
            }

            if let Some(summed_efficiency) = &mut self.summed_efficiency {
                ui.horizontal(|ui| {
                    if ui
                        .button("📋")
                        .on_hover_text(
                            "Copy data to clipboard (CSV format)\nEnergy, Efficiency, Uncertainty",
                        )
                        .clicked()
                    {
                        let stat_str = summed_efficiency.csv_points();
                        ui.output_mut(|o| o.copied_text = stat_str);
                    }

                    summed_efficiency.line.menu_button(ui);
                });

                if ui.button("Clear").clicked() {
                    self.summed_efficiency = None;
                }
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

        if let Some(summed_efficiency) = &mut self.summed_efficiency {
            summed_efficiency.draw(plot_ui);
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

    pub fn total_efficiency(&mut self, energy: f64) -> (f64, f64) {
        let mut efficiency = 0.0;
        let mut uncertainty_values = Vec::new();

        for fit in self.measurement_exp_fits.values() {
            if let Some(parameters) = &fit.exp_fitter.fit_params {
                if parameters.len() == 1 {
                    let a = parameters[0].0 .0;
                    let b = parameters[0].1 .0;
                    efficiency += a * (-energy / b).exp();
                } else if parameters.len() == 2 {
                    let a = parameters[0].0 .0;
                    let b = parameters[0].1 .0;
                    let c = parameters[1].0 .0;
                    let d = parameters[1].1 .0;
                    efficiency += a * (-energy / b).exp() + c * (-energy / d).exp();
                }
            }

            let uncertainity = fit.exp_fitter.uncertainity(energy, 1.0);
            uncertainty_values.push(uncertainity);
        }

        let total_uncertainty = (uncertainty_values.iter().map(|&x| x * x).sum::<f64>()).sqrt();

        (efficiency, total_uncertainty)
    }

    pub fn get_summed_efficiency(&mut self, max_x: f64) {
        // Ensure `summed_efficiency` is initialized
        if self.summed_efficiency.is_none() {
            self.summed_efficiency = Some(SummedEfficiency::new());
        }

        // Collect efficiency and uncertainty values before mutably borrowing `summed_efficiency`
        let num_points = 1000;
        let start = 0.0;
        let step = (max_x - start) / num_points as f64;

        let mut line_points: Vec<[f64; 2]> = Vec::new();
        let mut uncertainity_values: Vec<f64> = Vec::new();
        let mut uncertainty_lower_points: Vec<[f64; 2]> = Vec::new();
        let mut uncertainty_upper_points: Vec<[f64; 2]> = Vec::new();

        for i in 0..num_points {
            let x = start + i as f64 * step;
            let (efficiency, uncertainty) = self.total_efficiency(x);

            line_points.push([x, efficiency]);
            uncertainity_values.push(uncertainty);
            uncertainty_lower_points.push([x, efficiency - uncertainty]);
            uncertainty_upper_points.push([x, efficiency + uncertainty]);
        }

        // Now update `summed_efficiency` with the collected data
        if let Some(summed_efficiency) = &mut self.summed_efficiency {
            summed_efficiency.line.points = line_points;
            summed_efficiency.uncertainty = uncertainity_values;
            summed_efficiency.uncertainty_lower_points = uncertainty_lower_points;
            summed_efficiency.uncertainty_upper_points = uncertainty_upper_points;
        }
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
                    egui::CollapsingHeader::new("Sources")
                        .default_open(true)
                        .show(ui, |ui| {
                            for (index, measurement) in self.measurements.iter_mut().enumerate() {
                                measurement.update_ui(ui, index);

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
