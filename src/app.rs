use eframe::App;

use std::fs;
use std::fs::File;
use std::io::{Read, Write};

use crate::efficiency_fitter::measurements::MeasurementHandler;

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct CeBrAEfficiencyApp {
    measurment_handler: MeasurementHandler,
    window: bool,
}

impl CeBrAEfficiencyApp {
    pub fn new(cc: &eframe::CreationContext<'_>, window: bool) -> Self {
        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Self {
            measurment_handler: MeasurementHandler::new(),
            window,
        }
    }

    fn load_previous_measurements() -> Self {
        if let Ok(data) = fs::read_to_string("etc/REU_2023.yaml") {
            match serde_yaml::from_str(&data) {
                Ok(result) => result,
                Err(err) => {
                    eprintln!("Failed to deserialize data: {}", err);
                    Self::default()
                }
            }
        } else {
            Self::default()
        }
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
                    if let Err(err) = file.read_to_string(&mut data) {
                        eprintln!("Failed to read data from file: {}", err);
                        return Self::default();
                    }
                    match serde_yaml::from_str(&data) {
                        Ok(result) => result,
                        Err(err) => {
                            eprintln!("Failed to deserialize data: {}", err);
                            Self::default()
                        }
                    }
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

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label("Previous Measurements");
                if ui.button("REU 2023").clicked() {
                    *self = Self::load_previous_measurements();
                }
            });

            self.measurment_handler.ui(ui);
        });
    }
}

impl App for CeBrAEfficiencyApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

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
