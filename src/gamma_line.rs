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
