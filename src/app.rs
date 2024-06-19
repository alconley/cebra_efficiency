use eframe::App;

#[cfg(not(target_arch = "wasm32"))]
use std::fs;
#[cfg(not(target_arch = "wasm32"))]
use std::fs::File;
#[cfg(not(target_arch = "wasm32"))]
use std::io::{Read, Write};

#[cfg(target_arch = "wasm32")]
use std::sync::mpsc::{channel, Receiver, Sender};

use crate::efficiency_fitter::measurements::MeasurementHandler;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct CeBrAEfficiencyApp {
    measurment_handler: MeasurementHandler,
    window: bool,
    show_left_panel: bool,
    show_bottom_panel: bool,
    #[cfg(target_arch = "wasm32")]
    #[serde(skip)]
    file_channel: Option<(Sender<String>, Receiver<String>)>,
    #[cfg(target_arch = "wasm32")]
    #[serde(skip)]
    filename: String,
}

impl Default for CeBrAEfficiencyApp {
    fn default() -> Self {
        Self {
            measurment_handler: MeasurementHandler::new(),
            window: false,
            show_left_panel: true,
            show_bottom_panel: true,
            #[cfg(target_arch = "wasm32")]
            file_channel: None,
            #[cfg(target_arch = "wasm32")]
            filename: String::new(),
        }
    }
}

impl CeBrAEfficiencyApp {
    pub fn new(cc: &eframe::CreationContext<'_>, window: bool) -> Self {
        let mut app = Self {
            measurment_handler: MeasurementHandler::new(),
            window,
            show_left_panel: true,
            show_bottom_panel: true,
            #[cfg(target_arch = "wasm32")]
            file_channel: None,
            #[cfg(target_arch = "wasm32")]
            filename: String::new(),
        };

        if let Some(storage) = cc.storage {
            app = eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        #[cfg(target_arch = "wasm32")]
        if app.file_channel.is_none() {
            app.file_channel = Some(channel());
        }

        app
    }

    #[cfg(not(target_arch = "wasm32"))]
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

    #[cfg(target_arch = "wasm32")]
    fn load_from_file_wasm(&self, ui: &mut egui::Ui) {
        if ui.button("Load").clicked() {
            if let Some((sender, _)) = &self.file_channel {
                let sender = sender.clone();
                let task = rfd::AsyncFileDialog::new()
                    .set_title("Open")
                    .add_filter("YAML", &["yaml", "yml"])
                    .pick_file();

                let ctx = ui.ctx().clone();
                wasm_bindgen_futures::spawn_local(async move {
                    if let Some(file) = task.await {
                        let data = file.read().await;
                        let _ = sender.send(String::from_utf8_lossy(&data).to_string());
                        ctx.request_repaint();
                    } else {
                        eprintln!("No file selected");
                    }
                });
            }
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

    #[cfg(target_arch = "wasm32")]
    fn save_to_file_wasm(&mut self, ui: &mut egui::Ui) {
        use wasm_bindgen_futures::spawn_local;

        ui.horizontal(|ui| {
            if ui.button("Save").clicked() {
                let mut filename = self.filename.clone();
                if filename == "" {
                    filename = "cebra_efficiency".to_string();
                }

                if !filename.ends_with(".yaml") && !filename.ends_with(".yml") {
                    filename.push_str(".yaml");
                }

                let serialized_data =
                    serde_yaml::to_string(self).expect("Failed to serialize data.");
                let task = rfd::AsyncFileDialog::new()
                    .set_title(format!("Save As {}", filename))
                    .set_file_name(filename)
                    .add_filter("YAML", &["yaml", "yml"])
                    .save_file();

                spawn_local(async move {
                    if let Some(file_handle) = task.await {
                        if let Err(e) = file_handle.write(serialized_data.as_bytes()).await {
                            eprintln!("Failed to save file: {}", e);
                        }
                    } else {
                        eprintln!("No file selected for saving");
                    }
                });
            }

            ui.label("Filename:");
            ui.text_edit_singleline(&mut self.filename);
            ui.label(".yaml");
        });
    }

    #[cfg(target_arch = "wasm32")]
    fn handle_loaded_file(&mut self, ui: &mut egui::Ui) {
        // need this otherwise, you can only load something once
        if self.file_channel.is_none() {
            self.file_channel = Some(channel());
        }

        if let Some((_, receiver)) = &self.file_channel {
            if let Ok(data) = receiver.try_recv() {
                if let Ok(result) = serde_yaml::from_str(&data) {
                    self.replace_with(result);
                } else {
                    ui.label("Failed to deserialize data");
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
            self.handle_loaded_file(ui);
            self.load_from_file_wasm(ui);
            self.save_to_file_wasm(ui);
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context) {
        egui::TopBottomPanel::top("cebra_efficiency_top_panel").show_inside(ui, |ui| {
            egui::menu::bar(ui, |ui| {
                egui::global_dark_light_mode_switch(ui);

                ui.separator();

                ui.menu_button("File", |ui| {
                    self.egui_save_and_load_file(ui);
                });

                ui.separator();

                ui.menu_button("Panels", |ui| {
                    ui.checkbox(&mut self.show_left_panel, "Measurement Panel");
                    ui.checkbox(&mut self.show_bottom_panel, "Fitting Panel");
                });
            });
        });

        ui.vertical(|ui| {
            #[cfg(not(target_arch = "wasm32"))]
            ui.horizontal(|ui| {
                ui.label("Previous Measurements");
                if ui.button("REU 2023").clicked() {
                    *self = Self::load_previous_measurements();
                }
            });

            self.measurment_handler
                .ui(ui, self.show_bottom_panel, self.show_left_panel);
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
                self.ui(ui, ctx);
            });
        } else {
            egui::CentralPanel::default().show(ctx, |ui| {
                self.ui(ui, ctx);
            });
        }
    }
}

#[cfg(target_arch = "wasm32")]
trait ReplaceWith {
    fn replace_with(&mut self, other: Self);
}

#[cfg(target_arch = "wasm32")]
impl ReplaceWith for CeBrAEfficiencyApp {
    fn replace_with(&mut self, other: Self) {
        *self = other;
    }
}
