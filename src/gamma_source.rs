use super::detector::DetectorLine;
use egui_plot::MarkerShape;

#[derive(Default, Clone, serde::Deserialize, serde::Serialize)]
pub struct GammaLine {
    pub energy: f64, // keV
    pub intensity: f64,
    pub intensity_uncertainty: f64,
}

impl GammaLine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn gamma_line_ui(&mut self, ui: &mut egui::Ui) {
        ui.add(
            egui::DragValue::new(&mut self.energy)
                .speed(1.0)
                .clamp_range(0.0..=f64::INFINITY)
                .suffix(" keV"),
        );

        ui.add(
            egui::DragValue::new(&mut self.intensity)
                .speed(1)
                .clamp_range(0.0..=100.0)
                .suffix("%"),
        );

        ui.add(
            egui::DragValue::new(&mut self.intensity_uncertainty)
                .speed(0.1)
                .clamp_range(0.0..=100.0)
                .suffix("%"),
        );
    }
}

#[derive(Default, Clone, serde::Deserialize, serde::Serialize)]
pub struct SourceActivity {
    pub activity: f64, // kBq
    pub date: Option<chrono::NaiveDate>,
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct GammaSource {
    pub name: String,
    pub gamma_lines: Vec<GammaLine>,
    pub half_life: f64, // years
    pub source_activity_calibration: SourceActivity,
    pub source_activity_measurement: SourceActivity,
    pub measurement_time: f64, // hours

    pub marker_shape: String,
    pub marker_size: f32,
}

impl GammaSource {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            gamma_lines: Vec::new(),
            half_life: 0.0,
            source_activity_calibration: SourceActivity::default(),
            source_activity_measurement: SourceActivity::default(),
            measurement_time: 0.0,
            marker_shape: "circle".to_string(),
            marker_size: 5.0,
        }
    }

    pub fn fsu_152eu_source(&mut self) {
        self.name = "152Eu".to_string();
        self.half_life = 13.517; // years

        self.source_activity_calibration.activity = 74.370; // kBq
        self.source_activity_calibration.date = chrono::NaiveDate::from_ymd_opt(2017, 3, 17);

        self.add_gamma_line(121.7817, 28.53, 0.16);
        self.add_gamma_line(244.6974, 7.55, 0.04);
        self.add_gamma_line(344.2785, 26.59, 0.20);
        self.add_gamma_line(411.1164, 2.237, 0.013);
        self.add_gamma_line(443.9650, 2.827, 0.014);
        self.add_gamma_line(778.9045, 12.93, 0.08);
        self.add_gamma_line(867.3800, 4.23, 0.03);
        self.add_gamma_line(964.0570, 14.51, 0.07);
        self.add_gamma_line(1085.837, 10.11, 0.05);
        self.add_gamma_line(1112.076, 13.67, 0.08);
        self.add_gamma_line(1408.0130, 20.87, 0.09);
    }

    pub fn fsu_56co_source(&mut self) {
        self.name = "56Co".to_string();

        let co60_halflife_days = 77.236; // days
        self.half_life = co60_halflife_days / 365.25; // years

        self.source_activity_calibration.activity = 108.0; // kBq (arbitrary scaled to match 152Eu)
        self.source_activity_calibration.date = chrono::NaiveDate::from_ymd_opt(2022, 4, 18);

        self.add_gamma_line(846.7638, 99.9399, 0.0023);
        self.add_gamma_line(1037.8333, 14.03, 0.05);
        self.add_gamma_line(1360.196, 4.283, 0.013);
        self.add_gamma_line(2598.438, 16.96, 0.04);
        self.add_gamma_line(3451.119, 0.942, 0.006);
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
        // let activity_uncertainty = source_activity * 0.05; // 5% uncertainty in activity
        let activity_uncertainty = 0.0; // 0% uncertainty in activity

        let run_time = self.measurement_time * 3600.0; // convert hours to seconds
        let intensity = line.intensity;
        let intensity_uncertainty = line.intensity_uncertainty;
        let counts = line.count;
        let count_uncertainity = line.uncertainty;

        let efficiency = counts / (intensity * source_activity * run_time * 0.01) * 100.0; // efficiency in percent
        let efficiency_uncertainty = efficiency
            * ((count_uncertainity / counts).powi(2)
                + (intensity_uncertainty / intensity).powi(2)
                + (activity_uncertainty / source_activity).powi(2))
            .sqrt();

        line.efficiency = efficiency;
        line.efficiency_uncertainty = efficiency_uncertainty;
    }

    pub fn source_ui(&mut self, ui: &mut egui::Ui) {
        ui.collapsing("Source", |ui| {
            ui.horizontal(|ui| {
                ui.label("FSU Sources:");

                if ui.button("152Eu").clicked() {
                    self.fsu_152eu_source();
                }

                if ui.button("56Co").clicked() {
                    self.fsu_56co_source();
                }
            });

            ui.separator();

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

                    ui.label("Marker Shape:");
                    let marker_shape_names = [
                        "Circle", "Diamond", "Square", "Cross", "Plus", "Up", "Down", "Left",
                        "Right", "Asterisk",
                    ];
                    let mut selected_index = marker_shape_names
                        .iter()
                        .position(|&shape| shape == self.marker_shape)
                        .unwrap_or(0);
                    let _marker_shape = egui::ComboBox::from_id_source("marker_shape")
                        .selected_text(&self.marker_shape)
                        .show_ui(ui, |ui| {
                            for (index, name) in marker_shape_names.iter().enumerate() {
                                if ui
                                    .selectable_value(&mut selected_index, index, *name)
                                    .clicked()
                                {
                                    self.marker_shape = name.to_string();
                                }
                            }
                        });

                    ui.add(
                        egui::DragValue::new(&mut self.marker_size)
                            .speed(0.1)
                            .clamp_range(0.0..=f32::INFINITY)
                            .prefix("Size: "),
                    );

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

    pub fn remove_gamma_line(&mut self, index: usize) {
        self.gamma_lines.remove(index);
    }

    pub fn to_egui_marker_shape(&self) -> MarkerShape {
        match self.marker_shape.as_str() {
            "Circle" => MarkerShape::Circle,
            "Diamond" => MarkerShape::Diamond,
            "Square" => MarkerShape::Square,
            "Cross" => MarkerShape::Cross,
            "Plus" => MarkerShape::Plus,
            "Up" => MarkerShape::Up,
            "Down" => MarkerShape::Down,
            "Left" => MarkerShape::Left,
            "Right" => MarkerShape::Right,
            "Asterisk" => MarkerShape::Asterisk,
            _ => panic!("Invalid marker shape: {}", self.marker_shape),
        }
    }
}
