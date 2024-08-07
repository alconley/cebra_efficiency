use super::gamma_source::GammaSource;

use crate::egui_plot_stuff::egui_points::EguiPoints;

#[derive(Default, Clone, serde::Deserialize, serde::Serialize)]
pub struct DetectorLine {
    pub energy: f64,
    pub count: f64,
    pub uncertainty: f64,
    pub intensity: f64,
    pub intensity_uncertainty: f64,
    pub efficiency: f64,
    pub efficiency_uncertainty: f64,
}

impl DetectorLine {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.add(
            egui::DragValue::new(&mut self.count)
                .speed(1.0)
                .clamp_range(0.0..=f64::INFINITY),
        );
        ui.add(
            egui::DragValue::new(&mut self.uncertainty)
                .speed(1.0)
                .clamp_range(0.0..=f64::INFINITY),
        );

        ui.label(format!(
            "{:.3} ± {:.3}%",
            self.efficiency, self.efficiency_uncertainty
        ));
    }

    pub fn draw_uncertainty(
        &self,
        plot_ui: &mut egui_plot::PlotUi,
        color: egui::Color32,
        name: Option<String>,
    ) {
        let points = vec![
            [self.energy, self.efficiency - self.efficiency_uncertainty],
            [self.energy, self.efficiency + self.efficiency_uncertainty],
        ];

        let mut line = egui_plot::Line::new(points).color(color);

        if let Some(name) = name {
            line = line.name(name);
        }

        plot_ui.line(line);
    }
}

#[derive(Default, Clone, serde::Deserialize, serde::Serialize)]
pub struct Detector {
    pub name: String,
    pub source_name: String,
    pub lines: Vec<DetectorLine>,
    pub points: EguiPoints,
    pub to_remove: Option<bool>,
}

impl Detector {
    pub fn ui(&mut self, ui: &mut egui::Ui, gamma_source: &GammaSource) {
        ui.horizontal(|ui| {
            ui.label("Detector Name:");
            ui.text_edit_singleline(&mut self.name);

            if ui.button("X").clicked() {
                self.to_remove = Some(true);
            }
        });

        // ui.collapsing(self.name.to_string(), |ui| {
        egui::CollapsingHeader::new(self.name.to_string())
            .default_open(true)
            .show(ui, |ui| {
                let gamma_lines = gamma_source
                    .gamma_lines
                    .iter()
                    .map(|line| format!("{:.1} keV", line.energy))
                    .collect::<Vec<_>>();

                egui::Grid::new("detector_grid")
                    .striped(false)
                    .num_columns(4)
                    .show(ui, |ui| {
                        ui.label("Energy");
                        ui.label("Counts");
                        ui.label("Uncertainty");
                        ui.end_row();

                        let mut index_to_remove = None;
                        for (index, line) in self.lines.iter_mut().enumerate() {
                            egui::ComboBox::from_id_source(format!("Line {}", index))
                                .selected_text(format!("{:.1} keV", line.energy))
                                .show_ui(ui, |ui| {
                                    for (gamma_index, gamma_line_str) in
                                        gamma_lines.iter().enumerate()
                                    {
                                        if ui
                                            .selectable_label(
                                                line.energy
                                                    == gamma_source.gamma_lines[gamma_index].energy,
                                                gamma_line_str,
                                            )
                                            .clicked()
                                        {
                                            line.energy =
                                                gamma_source.gamma_lines[gamma_index].energy;
                                            line.intensity =
                                                gamma_source.gamma_lines[gamma_index].intensity;
                                            line.intensity_uncertainty = gamma_source.gamma_lines
                                                [gamma_index]
                                                .intensity_uncertainty;
                                        }
                                    }
                                });

                            line.ui(ui);

                            if ui.button("X").clicked() {
                                index_to_remove = Some(index);
                            }

                            ui.end_row();
                        }

                        if let Some(index) = index_to_remove {
                            self.remove_line(index);
                        }
                    });

                ui.horizontal(|ui| {
                    if ui.button("+").clicked() {
                        self.lines.push(DetectorLine::default());
                    }
                });

                for line in &mut self.lines {
                    gamma_source.gamma_line_efficiency_from_source_measurement(line);
                }
            });
    }

    fn remove_line(&mut self, index: usize) {
        self.lines.remove(index);
    }

    fn get_detector_points(&self) -> Vec<[f64; 2]> {
        self.lines
            .iter()
            .map(|line| [line.energy, line.efficiency])
            .collect()
    }

    pub fn draw(&mut self, plot_ui: &mut egui_plot::PlotUi, name: Option<String>) {
        self.points.points = self.get_detector_points();

        if self.points.draw {
            for line in &self.lines {
                line.draw_uncertainty(plot_ui, self.points.color, name.clone());
            }
        }

        self.points.draw(plot_ui);
    }

    pub fn menu_button(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui
            .button("📋")
            .on_hover_text("Copy data to clipboard (CSV format)\nEnergy,Counts,Uncertainty,Intensity,Intensity Uncertainty,Efficiency,Efficiency Uncertainty")
            .clicked()
                {
                    let stat_str = self.lines_csv();
                    ui.output_mut(|o| o.copied_text = stat_str);
                }
            self.points.menu_button(ui);
        });
    }

    pub fn lines_csv(&self) -> String {
        let mut csv = String::new();

        csv.push_str("Energy,Counts,Uncertainty,Intensity,Intensity Uncertainty,Efficiency,Efficiency Uncertainty\n");

        for line in &self.lines {
            csv.push_str(&format!(
                "{},{},{},{},{},{},{}\n",
                line.energy,
                line.count,
                line.uncertainty,
                line.intensity,
                line.intensity_uncertainty,
                line.efficiency,
                line.efficiency_uncertainty
            ));
        }

        csv
    }
}
