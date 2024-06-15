#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EguiPlotSettings {
    pub legend: bool,
    pub show_x_value: bool,
    pub show_y_value: bool,
    pub center_x_axis: bool,
    pub center_y_axis: bool,
    pub allow_zoom: bool,
    pub allow_boxed_zoom: bool,
    pub allow_drag: bool,
    pub allow_scroll: bool,
    pub clamp_grid: bool,
    pub show_grid: bool,
    pub sharp_grid_lines: bool,
    pub show_background: bool,
}

impl Default for EguiPlotSettings {
    fn default() -> Self {
        EguiPlotSettings {
            legend: true,
            show_x_value: true,
            show_y_value: true,
            center_x_axis: false,
            center_y_axis: false,
            allow_zoom: true,
            allow_boxed_zoom: true,
            allow_drag: true,
            allow_scroll: true,
            clamp_grid: false,
            show_grid: true,
            sharp_grid_lines: true,
            show_background: true,
        }
    }
}

impl EguiPlotSettings {
    pub fn menu_button(&mut self, ui: &mut egui::Ui) {
        ui.menu_button("egui Plot Settings", |ui| {
            ui.vertical(|ui| {
                ui.checkbox(&mut self.legend, "Legend");
                ui.checkbox(&mut self.show_x_value, "Show X Value");
                ui.checkbox(&mut self.show_y_value, "Show Y Value");
                ui.checkbox(&mut self.center_x_axis, "Center X Axis");
                ui.checkbox(&mut self.center_y_axis, "Center Y Axis");
                ui.checkbox(&mut self.allow_zoom, "Allow Zoom");
                ui.checkbox(&mut self.allow_boxed_zoom, "Allow Boxed Zoom");
                ui.checkbox(&mut self.allow_drag, "Allow Drag");
                ui.checkbox(&mut self.allow_scroll, "Allow Scroll");
                ui.checkbox(&mut self.clamp_grid, "Clamp Grid");
                ui.checkbox(&mut self.show_grid, "Show Grid");
                ui.checkbox(&mut self.sharp_grid_lines, "Sharp Grid Lines");
                ui.checkbox(&mut self.show_background, "Show Background");

                ui.separator();

                if ui.button("Reset").clicked() {
                    *self = EguiPlotSettings::default();
                }
            });
        });
    }

    // some function i can call that adds the settings to the plot
    pub fn apply_to_plot(&self, plot: egui_plot::Plot) -> egui_plot::Plot {
        let plot = plot
            .show_x(self.show_x_value)
            .show_y(self.show_y_value)
            .center_x_axis(self.center_x_axis)
            .center_y_axis(self.center_y_axis)
            .allow_zoom(self.allow_zoom)
            .allow_boxed_zoom(self.allow_boxed_zoom)
            .allow_drag(self.allow_drag)
            .allow_scroll(self.allow_scroll)
            .clamp_grid(self.clamp_grid)
            .show_grid(self.show_grid)
            .sharp_grid_lines(self.sharp_grid_lines)
            .show_background(self.show_background)
            .auto_bounds(egui::Vec2b::new(true, true));

        if self.legend {
            plot.legend(egui_plot::Legend::default())
        } else {
            plot
        }
    }
}
