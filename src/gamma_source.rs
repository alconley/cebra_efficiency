use super::detector::DetectorLine;
use super::gamma_line::GammaLine;

#[derive(Default, Clone, serde::Deserialize, serde::Serialize)]
pub struct SourceActivity {
    pub activity: f64, // kBq
    pub date: Option<chrono::NaiveDate>,
}

#[derive(Default, Clone, serde::Deserialize, serde::Serialize)]
pub struct GammaSource {
    pub name: String,
    pub gamma_lines: Vec<GammaLine>,
    pub half_life: f64, // years
    pub source_activity_calibration: SourceActivity,
    pub source_activity_measurement: SourceActivity,
    pub measurement_time: f64, // hours
}

impl GammaSource {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_gamma_line(&mut self, energy: f64, intensity: f64, intensity_uncertainty: f64) {
        let gamma_line = GammaLine {
            energy,
            intensity,
            intensity_uncertainty,
        };

        self.gamma_lines.push(gamma_line);
    }

    pub fn calculate_source_activity_for_measurement(&mut self) {
        let calibration_date = self.source_activity_calibration.date.unwrap();
        let measurement_date = self.source_activity_measurement.date.unwrap();
        let half_life_years = self.half_life;
        let half_life_days = half_life_years * 365.25; // convert years to days

        let time_difference = measurement_date
            .signed_duration_since(calibration_date)
            .num_days() as f64;
        let decay_constant = 0.693 / half_life_days;
        let source_activity_bq = self.source_activity_calibration.activity * 1000.0; // convert kBq to Bq
        let activity = source_activity_bq * (-decay_constant * time_difference).exp();

        self.source_activity_measurement.activity = activity;
    }

    pub fn gamma_line_efficiency_from_source_measurement(&self, line: &mut DetectorLine) {
        let source_activity = self.source_activity_measurement.activity;
        let activity_uncertainty = source_activity * 0.05; // 5% uncertainty in activity
        // let activity_uncertainty = 0.0; // 0% uncertainty in activity

        let run_time = self.measurement_time * 3600.0; // convert hours to seconds
        let intensity = line.intensity;
        let intensity_uncertainty = line.intensity_uncertainty;
        let counts = line.count;
        let count_uncertainity = line.uncertainty;

        let efficiency = counts / (intensity * source_activity * run_time * 0.01) * 100.0; // efficiency in percent
        let efficiency_uncertainty = efficiency * 
            (     (count_uncertainity / counts).powi(2)
                + (intensity_uncertainty / intensity).powi(2)
                + (activity_uncertainty / source_activity).powi(2)
            )
            .sqrt();

        line.efficiency = efficiency;
        line.efficiency_uncertainty = efficiency_uncertainty;
    }

    pub fn source_ui(&mut self, ui: &mut egui::Ui) {
        ui.collapsing("Source", |ui| {
            egui::Grid::new("source_ui")
                .striped(true)
                .min_col_width(50.0)
                .show(ui, |ui| {
                    ui.label("Source");
                    ui.add(egui::TextEdit::singleline(&mut self.name));

                    ui.label("Half-life:");
                    ui.add(
                        egui::DragValue::new(&mut self.half_life)
                            .speed(0.1)
                            .clamp_range(0.0..=f64::INFINITY)
                            .suffix(" years"),
                    );

                    ui.end_row();

                    ui.label("Calibration");

                    ui.label("Date:");

                    let calibration_date = self
                        .source_activity_calibration
                        .date
                        .get_or_insert_with(|| chrono::offset::Utc::now().date_naive());
                    ui.add(
                        egui_extras::DatePickerButton::new(calibration_date)
                            .id_source("calibration_date")
                            .highlight_weekends(false),
                    );

                    ui.label("Activity:");
                    ui.add(
                        egui::DragValue::new(&mut self.source_activity_calibration.activity)
                            .speed(1.0)
                            .clamp_range(0.0..=f64::INFINITY)
                            .suffix(" kBq"),
                    );

                    ui.end_row();

                    ui.label("Measurement");

                    ui.label("Date:");

                    let measurement_date = self
                        .source_activity_measurement
                        .date
                        .get_or_insert_with(|| chrono::offset::Utc::now().date_naive());
                    ui.add(
                        egui_extras::DatePickerButton::new(measurement_date)
                            .id_source("measurement_date")
                            .highlight_weekends(false),
                    );

                    ui.label("Run Time:");
                    ui.add(
                        egui::DragValue::new(&mut self.measurement_time)
                            .speed(0.5)
                            .clamp_range(0.0..=f64::INFINITY)
                            .suffix(" hours"),
                    );

                    ui.end_row();

                    if ui.button("Calculate Activity").clicked() {
                        self.calculate_source_activity_for_measurement();
                    }

                    ui.label("Activity:");

                    ui.label(&format!(
                        "{:.0} Bq",
                        self.source_activity_measurement.activity
                    ));

                    ui.end_row();
                    ui.label("Energy");
                    ui.label("Intensity");
                    ui.label("");
                    ui.label("Delete");
                    ui.end_row();
                    ui.label("Value");
                    ui.label("Value");
                    ui.label("±");
                    ui.end_row();

                    let mut index_to_remove: Option<usize> = None;

                    for (index, gamma_line) in self.gamma_lines.iter_mut().enumerate() {
                        gamma_line.gamma_line_ui(ui);

                        if ui.button("X").clicked() {
                            index_to_remove = Some(index);
                        }

                        ui.end_row();
                    }

                    if let Some(index) = index_to_remove {
                        self.remove_gamma_line(index);
                    }

                    if ui.button("Add γ Line").clicked() {
                        self.gamma_lines.push(GammaLine::new());
                    }
                });
        });

        ui.separator();
    }

    // Modify other methods to work with the new gamma_lines structure
    pub fn remove_gamma_line(&mut self, index: usize) {
        self.gamma_lines.remove(index);
    }
}
