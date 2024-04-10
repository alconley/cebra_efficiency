use nalgebra::DVector;
use varpro::model::builder::SeparableModelBuilder;
use varpro::solvers::levmar::{LevMarProblemBuilder, LevMarSolver};

use egui_plot::{Line, PlotPoint, PlotPoints, PlotUi, Polygon};

#[derive(Default, Clone, serde::Deserialize, serde::Serialize)]
pub struct ExpFitter {
    pub fit_params: Option<Vec<((f64, f64), (f64, f64))>>,
    pub x: Vec<f64>,
    pub y: Vec<f64>,
    pub weights: Vec<f64>,

    #[serde(skip)]
    pub fit_line: Option<Vec<Vec<PlotPoint>>>,

    #[serde(skip)]
    pub fit_uncertainity_lines: Option<Vec<Vec<PlotPoint>>>,

    pub fit_label: String,
}

impl ExpFitter {
    pub fn new(x: Vec<f64>, y: Vec<f64>, weights: Vec<f64>) -> Self {
        Self {
            fit_params: None,
            x,
            y,
            weights,
            fit_line: None,
            fit_uncertainity_lines: None,
            fit_label: "".to_string(),
        }
    }

    fn exponential(x: &DVector<f64>, b: f64) -> DVector<f64> {
        x.map(|x_val| (-x_val / b).exp())
    }

    fn exponential_pd_b(x: &DVector<f64>, b: f64) -> DVector<f64> {
        x.map(|x_val| (x_val / b.powi(2)) * (-x_val / b).exp())
    }

    fn exponential_pd_d(x: &DVector<f64>, d: f64) -> DVector<f64> {
        x.map(|x_val| (x_val / d.powi(2)) * (-x_val / d).exp())
    }

    pub fn single_exp_fit(&mut self, initial_b_guess: f64) {
        self.fit_params = None;
        self.fit_line = None;
        self.fit_uncertainity_lines = None;
        self.fit_label = "".to_string();

        let x_data = DVector::from_vec(self.x.clone());
        let y_data = DVector::from_vec(self.y.clone());
        let weights = DVector::from_vec(self.weights.clone());

        let parameter_names: Vec<String> = vec!["b".to_string()];

        let intitial_parameters = vec![initial_b_guess];

        let builder_proxy = SeparableModelBuilder::<f64>::new(parameter_names)
            .initial_parameters(intitial_parameters)
            .independent_variable(x_data)
            .function(&["b"], Self::exponential)
            .partial_deriv("b", Self::exponential_pd_b);

        let model = match builder_proxy.build() {
            Ok(model) => model,
            Err(err) => {
                log::error!("Error building model: {}", err);
                return;
            }
        };

        let problem = match LevMarProblemBuilder::new(model)
            .observations(y_data)
            .weights(weights)
            .build()
                {
                    Ok(problem) => problem,
                    Err(err) => {
                        log::error!("Error building problem: {}", err);
                        return;
                    }
                };

        if let Ok((fit_result, fit_statistics)) =
            LevMarSolver::default().fit_with_statistics(problem)
        {
            let nonlinear_parameters = fit_result.nonlinear_parameters();
            let nonlinear_variances = fit_statistics.nonlinear_parameters_variance();
            
            let linear_coefficients = fit_result.linear_coefficients();
            
            let linear_coefficients = match linear_coefficients {
                Some(coefficients) => coefficients,
                None => {
                    log::error!("No linear coefficients found");
                    return;
                }
            };
            let linear_variances = fit_statistics.linear_coefficients_variance();

            let parameter_a = linear_coefficients[0];
            let parameter_a_variance = linear_variances[0];
            let parameter_a_uncertainity = parameter_a_variance.sqrt();

            let parameter_b = nonlinear_parameters[0];
            let parameter_b_variance = nonlinear_variances[0];
            let parameter_b_uncertainity = parameter_b_variance.sqrt();

            let fit_string = format!(
                "Y = ({:.2} ± {:.2}) * exp[ -x / ({:.2} ± {:.2}) ]",
                parameter_a, parameter_a_uncertainity, parameter_b, parameter_b_uncertainity
            );
            self.fit_label = fit_string;

            let parameters = vec![(
                (parameter_a, parameter_a_uncertainity),
                (parameter_b, parameter_b_uncertainity),
            )];
            log::info!("parameters: {:?}", parameters);

            self.fit_params = Some(parameters);

            let num_points = 2000;

            // let min_x = self.x.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let max_x = self.x.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

            // let start = min_x - 100.0;
            let start = 1.0;
            let end = max_x + 1000.0;

            let step = (end - start) / num_points as f64;

            let plot_points: Vec<PlotPoint> = (0..=num_points)
                .map(|i| {
                    let x = start + i as f64 * step;
                    let y = parameter_a * (-x / parameter_b).exp();

                    PlotPoint::new(x, y)
                })
                .collect();

            self.fit_line = Some(vec![plot_points]);

            
            let upper_points: Vec<PlotPoint> = (0..=num_points)
                .map(|i| {
                    let x = start + i as f64 * step;
                    let y = (parameter_a + parameter_a_uncertainity) * (-x / (parameter_b + parameter_b_uncertainity)).exp();

                    PlotPoint::new(x, y)
                })
                .collect();

            let lower_points: Vec<PlotPoint> = (0..=num_points)
                .map(|i| {
                    let x = start + i as f64 * step;
                    let y = (parameter_a - parameter_a_uncertainity) * (-x / (parameter_b - parameter_b_uncertainity)).exp();

                    PlotPoint::new(x, y)
                })
                .collect();

            // egui only supports convex polygons so i need to split the polygon into multiple.
            // So each polygon will be the two points in the upper and two in the lower
            // then the next polygon will be the next two points in the upper and lower
            // and so on

            let mut polygons: Vec<Vec<PlotPoint>> = Vec::new();
            for i in 0..num_points {
                let polygon = vec![
                    upper_points[i],
                    upper_points[i + 1],
                    lower_points[i + 1],
                    lower_points[i],
                ];
                polygons.push(polygon);
            }
            
            self.fit_uncertainity_lines = Some(polygons);

        }
    }

    pub fn double_exp_fit(&mut self, initial_b_guess: f64, initial_d_guess: f64) {
        self.fit_params = None;
        self.fit_line = None;
        self.fit_uncertainity_lines = None;
        self.fit_label = "".to_string();

        let x_data = DVector::from_vec(self.x.clone());
        let y_data = DVector::from_vec(self.y.clone());
        let weights = DVector::from_vec(self.weights.clone());

        let parameter_names: Vec<String> = vec!["b".to_string(), "d".to_string()];

        let initial_parameters = vec![initial_b_guess, initial_d_guess];

        let builder_proxy = SeparableModelBuilder::<f64>::new(parameter_names)
            .initial_parameters(initial_parameters)
            .independent_variable(x_data)
            .function(&["b"], Self::exponential)
            .partial_deriv("b", Self::exponential_pd_b)
            .function(&["d"], Self::exponential)
            .partial_deriv("d", Self::exponential_pd_d);

        let model = match builder_proxy.build() {
            Ok(model) => model,
            Err(err) => {
                log::error!("Error building model: {}", err);
                return;
            }
        };

        let problem = match LevMarProblemBuilder::new(model)
            .observations(y_data)
            .weights(weights)
            .build()
                {
                    Ok(problem) => problem,
                    Err(err) => {
                        log::error!("Error building problem: {}", err);
                        return;
                    }
                };

        if let Ok((fit_result, fit_statistics)) =
            LevMarSolver::default().fit_with_statistics(problem)
        {

            let nonlinear_parameters = fit_result.nonlinear_parameters();
            let nonlinear_variances = fit_statistics.nonlinear_parameters_variance();

            let linear_coefficients = fit_result.linear_coefficients();

            let linear_coefficients = match linear_coefficients {
                Some(coefficients) => coefficients,
                None => {
                    log::error!("No linear coefficients found");
                    return;
                }
            };

            let linear_variances = fit_statistics.linear_coefficients_variance();

            let parameter_a = linear_coefficients[0];
            let parameter_a_variance = linear_variances[0];
            let parameter_a_uncertainity = parameter_a_variance.sqrt();

            let parameter_b = nonlinear_parameters[0];
            let parameter_b_variance = nonlinear_variances[0];
            let parameter_b_uncertainity = parameter_b_variance.sqrt();

            let exp_1 = (
                (parameter_a, parameter_a_uncertainity),
                (parameter_b, parameter_b_uncertainity),
            );

            let parameter_c = linear_coefficients[1];
            let parameter_c_variance = linear_variances[1];
            let parameter_c_uncertainity = parameter_c_variance.sqrt();

            let parameter_d = nonlinear_parameters[1];
            let parameter_d_variance = nonlinear_variances[1];
            let parameter_d_uncertainity = parameter_d_variance.sqrt();

            let exp_2 = (
                (parameter_c, parameter_c_uncertainity),
                (parameter_d, parameter_d_uncertainity),
            );

            let parameters = vec![exp_1, exp_2];

            let fit_string = format!("Y = ({:.2} ± {:.2}) * exp[ -x / ({:.2}±{:.2}) ] + ({:.2} ± {:.2}) * exp[ -x / ({:.2} ± {:.2}) ]",
                parameter_a, parameter_a_uncertainity,
                parameter_b, parameter_b_uncertainity,
                parameter_c, parameter_c_uncertainity,
                parameter_d, parameter_d_uncertainity);
            
            self.fit_label = fit_string;

            self.fit_params = Some(parameters);

            // let min_x = self.x.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let max_x = self.x.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

            let num_points = 1000;

            let start = 0.0;
            let end = max_x + 0.0;
            let end = 7000.0;


            let step = (end - start) / num_points as f64;

            // fit line
            let plot_points: Vec<PlotPoint> = (0..=num_points)
                .map(|i| {
                    let x = start + i as f64 * step;
                    let y = parameter_a * (-x / parameter_b).exp()
                        + parameter_c * (-x / parameter_d).exp();

                    PlotPoint::new(x, y)
                })
                .collect();

            self.fit_line = Some(vec![plot_points]);

            let upper_points: Vec<PlotPoint> = (0..=num_points)
                .map(|i| {
                    let x = start + i as f64 * step;
                    let y = (parameter_a + parameter_a_uncertainity) * (-x / (parameter_b + parameter_b_uncertainity)).exp() + (parameter_c + parameter_c_uncertainity) * (-x / (parameter_d + parameter_d_uncertainity)).exp();

                    PlotPoint::new(x, y)
                })
                .collect();

            let lower_points: Vec<PlotPoint> = (0..=num_points)
                .map(|i| {
                    let x = start + i as f64 * step;
                    let y = (parameter_a - parameter_a_uncertainity) * (-x / (parameter_b - parameter_b_uncertainity)).exp() + (parameter_c - parameter_c_uncertainity) * (-x / (parameter_d - parameter_d_uncertainity)).exp();

                    PlotPoint::new(x, y)
                })
                .collect();

            let mut polygons: Vec<Vec<PlotPoint>> = Vec::new();
            for i in 0..num_points {
                let polygon = vec![
                    upper_points[i],
                    upper_points[i + 1],
                    lower_points[i + 1],
                    lower_points[i],
                ];
                polygons.push(polygon);
            }
            
            self.fit_uncertainity_lines = Some(polygons);
        }
    }
 
    pub fn draw_fit_line(&self, plot_ui: &mut PlotUi, color: egui::Color32, name: String) {
        if let Some(fit_line) = &self.fit_line {
            for points in fit_line.iter() {
                let line = Line::new(PlotPoints::Owned(points.clone()))
                    .color(color)
                    .stroke(egui::Stroke::new(1.0, color))
                    .name(name.clone());

                plot_ui.line(line);
            }
        }

        if let Some(fit_line_uncertainity) = &self.fit_uncertainity_lines {
            for points in fit_line_uncertainity.iter() {

                let uncertainity_band = Polygon::new(PlotPoints::Owned(points.clone()))
                .stroke(egui::Stroke::new(0.0, color))
                .width(0.0)
                .name(name.clone());

                plot_ui.polygon(uncertainity_band);
            }

        }
    }

}

#[derive(Default, Clone, serde::Deserialize, serde::Serialize)]
pub struct Fitter {
    pub name: String,
    pub data: (Vec<f64>, Vec<f64>, Vec<f64>), // (x_data, y_data, weights)
    pub exp_fitter: Option<ExpFitter>,
    pub initial_b_guess: f64,
    pub initial_d_guess: f64,
}

impl Fitter {

    pub fn ui(&mut self, ui: &mut egui::Ui) {

            ui.label(self.name.to_string());


        ui.horizontal(|ui| {

            ui.add(
                egui::DragValue::new(&mut self.initial_b_guess)
                    .prefix("b: ")
                    .speed(10.0)
                    .clamp_range(0.0..=f64::INFINITY)
            );

            ui.add(
                egui::DragValue::new(&mut self.initial_d_guess)
                    .prefix("d: ")
                    .speed(10.0)
                    .clamp_range(0.0..=f64::INFINITY)
            );
        });

        ui.horizontal(|ui| {

            if ui.button("Single").clicked() {

                let (x_data, y_data, weights) = self.data.clone();
                self.exp_fitter = Some(ExpFitter::new(x_data, y_data, weights));

                if let Some(exp_fitter) = &mut self.exp_fitter {
                    exp_fitter.single_exp_fit(self.initial_b_guess);
                }

            }

            if ui.button("Double").clicked() {
                let (x_data, y_data, weights) = self.data.clone();
                self.exp_fitter = Some(ExpFitter::new(x_data, y_data, weights));

                if let Some(exp_fitter) = &mut self.exp_fitter {
                    exp_fitter.double_exp_fit(self.initial_b_guess, self.initial_d_guess);
                }
            }
        }); 

        ui.label("Parameters:");

        // Display fit parameters
        if let Some(exp_fitter) = &self.exp_fitter {
            if let Some(fit_params) = &exp_fitter.fit_params {
                for ((a, a_uncertainty), (b, b_uncertainty)) in fit_params.iter() {
                    ui.label(format!(
                        "{:.1e} ± {:.1e}",
                        a, a_uncertainty
                    ));

                    ui.label(format!(
                        "{:.1e} ± {:.1e}",
                        b, b_uncertainty
                    ));
                }
            }
        }

    }
}