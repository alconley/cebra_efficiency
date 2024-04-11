use eframe::App;

use serde_yaml;

use std::fs::File;
use std::io::{Read, Write};

use super::measurements::MeasurementHandler;

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
