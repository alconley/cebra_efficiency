use eframe::App;

use serde_yaml;

use std::fs::File;
use std::io::{Read, Write};

use super::gamma_source::GammaSource;
use super::measurements::{Measurement, MeasurementHandler};

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct CeBrAEfficiencyApp {
    measurment_handler: MeasurementHandler,
    window: bool,
}

impl CeBrAEfficiencyApp {
    pub fn new(_cc: &eframe::CreationContext<'_>, window: bool) -> Self {
        Self {
            measurment_handler: MeasurementHandler::new(),
            window,
        }
    }

    fn get_fsu_152eu_source(&mut self) -> GammaSource {
        let mut gamma_source = GammaSource::new();
        gamma_source.name = "152Eu".to_string();
        gamma_source.half_life = 13.517; // years

        gamma_source.source_activity_calibration.activity = 74.370; // kBq
        gamma_source.source_activity_calibration.date =
            chrono::NaiveDate::from_ymd_opt(2017, 3, 17);

        gamma_source.add_gamma_line(121.7817, 28.53, 0.16);
        gamma_source.add_gamma_line(244.6974, 7.55, 0.04);
        gamma_source.add_gamma_line(344.2785, 26.59, 0.20);
        gamma_source.add_gamma_line(411.1164, 2.237, 0.013);
        gamma_source.add_gamma_line(443.9650, 2.827, 0.014);
        gamma_source.add_gamma_line(778.9045, 12.93, 0.08);
        gamma_source.add_gamma_line(867.3800, 4.23, 0.03);
        gamma_source.add_gamma_line(964.0570, 14.51, 0.07);
        gamma_source.add_gamma_line(1085.837, 10.11, 0.05);
        gamma_source.add_gamma_line(1112.076, 13.67, 0.08);
        gamma_source.add_gamma_line(1408.0130, 20.87, 0.09);

        gamma_source
    }

    fn get_fsu_56co_source(&mut self) -> GammaSource {
        let mut gamma_source = GammaSource::new();
        gamma_source.name = "56Co".to_string();

        let co60_halflife_days = 77.236; // days
        gamma_source.half_life = co60_halflife_days / 365.25; // years

        gamma_source.source_activity_calibration.activity = 108.0; // kBq (arbitrary scaled to match 152Eu)
        gamma_source.source_activity_calibration.date =
            chrono::NaiveDate::from_ymd_opt(2022, 4, 18);

        gamma_source.add_gamma_line(846.7638, 99.9399, 0.0023);
        gamma_source.add_gamma_line(1037.8333, 14.03, 0.05);
        gamma_source.add_gamma_line(1360.196, 4.283, 0.013);
        gamma_source.add_gamma_line(2598.438, 16.96, 0.04);
        gamma_source.add_gamma_line(3451.119, 0.942, 0.006);

        gamma_source
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn load_from_file() -> Self {
        if let Some(path) = rfd::FileDialog::new()
            .set_title("Open")
            .add_filter("YAML", &["yaml", "yml"])
            .pick_file()
        {
            match File::open(path) {
                Ok(mut file) => {
                    let mut data = String::new();
                    file.read_to_string(&mut data)
                        .expect("Failed to read data from file.");
                    serde_yaml::from_str(&data).expect("Failed to deserialize data.")
                }
                Err(e) => {
                    eprintln!("Failed to load file: {}", e);
                    Self::default() // Return default if loading fails
                }
            }
        } else {
            Self::default() // Return default if no file is picked
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn save_to_file(&self) {
        if let Some(path) = rfd::FileDialog::new()
            .set_title("Save As")
            .add_filter("YAML", &["yaml", "yml"])
            .save_file()
        {
            match File::create(path) {
                Ok(mut file) => {
                    let data = serde_yaml::to_string(self).expect("Failed to serialize data.");
                    file.write_all(data.as_bytes())
                        .expect("Failed to write data to file.");
                }
                Err(e) => {
                    eprintln!("Failed to save file: {}", e);
                }
            }
        }
    }

    fn egui_save_and_load_file(&mut self, ui: &mut egui::Ui) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            if ui.button("Save").clicked() {
                self.save_to_file();
                ui.close_menu();
            }

            if ui.button("Load").clicked() {
                *self = Self::load_from_file();
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            // inplement save/load for web
            ui.label("Save/Load not implemented for web as of now.");
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        egui::TopBottomPanel::top("cebra_efficiency_top_panel").show_inside(ui, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    self.egui_save_and_load_file(ui);
                });
            });
        });

        egui::SidePanel::left("cebra_efficiency_left_side_panel").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("FSU's Current Sources:");

                if ui.button("152Eu").clicked() {
                    let eu152 = self.get_fsu_152eu_source();

                    self.measurment_handler
                        .measurements
                        .push(Measurement::new(Some(eu152)));
                }

                if ui.button("56Co").clicked() {
                    let co56 = self.get_fsu_56co_source();

                    self.measurment_handler
                        .measurements
                        .push(Measurement::new(Some(co56)));
                }

                ui.separator();
            });

            self.measurment_handler.sources_ui(ui);
        });

        self.measurment_handler.plot(ui);
    }
}

impl App for CeBrAEfficiencyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.window {
            egui::Window::new("CeBrA Efficiency").show(ctx, |ui| {
                self.ui(ui);
            });
        } else {
            egui::CentralPanel::default().show(ctx, |ui| {
                self.ui(ui);
            });
        }
    }
}
