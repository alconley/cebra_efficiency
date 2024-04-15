use egui_plot::{Line, PlotPoint, PlotPoints, PlotUi, Polygon};
use nalgebra::DVector;
use statrs::distribution::ContinuousCDF;
use std::f64::consts::SQRT_2;
use varpro::model::builder::SeparableModelBuilder;
use varpro::solvers::levmar::{LevMarProblemBuilder, LevMarSolver};

#[derive(Default, Clone, serde::Deserialize, serde::Serialize)]
pub struct ExpFitter {
    #[allow(clippy::type_complexity)]
    pub fit_params: Option<Vec<((f64, f64), (f64, f64))>>,
    pub x: Vec<f64>,
    pub y: Vec<f64>,
    pub weights: Vec<f64>,

    pub fit_line_points: Vec<(f64, f64)>,
    pub upper_uncertainity_points: Vec<(f64, f64)>,
    pub lower_uncertainity_points: Vec<(f64, f64)>,

    pub fit_label: String,

    pub color: egui::Color32,
}

impl ExpFitter {
    pub fn new(x: Vec<f64>, y: Vec<f64>, weights: Vec<f64>) -> Self {
        Self {
            fit_params: None,
            x,
            y,
            weights,
            fit_line_points: Vec::new(),
            upper_uncertainity_points: Vec::new(),
            lower_uncertainity_points: Vec::new(),
            fit_label: "".to_string(),
            color: egui::Color32::LIGHT_BLUE,
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

    pub fn color_ui(&mut self, ui: &mut egui::Ui) {
        ui.color_edit_button_srgba(&mut self.color);
    }

    pub fn single_exp_fit(&mut self, initial_b_guess: f64) {
        self.fit_params = None;
        self.fit_line_points = Vec::new();
        self.upper_uncertainity_points = Vec::new();
        self.lower_uncertainity_points = Vec::new();
        self.fit_label = "".to_string();

        let x_data = DVector::from_vec(self.x.clone());
        let y_data = DVector::from_vec(self.y.clone());
        let weights = DVector::from_vec(self.weights.clone());

        let observation_length = x_data.len();

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
            log::info!("fit_result: {:?}\n\n", fit_result);
            log::info!("fit_statistics: {:?}\n\n", fit_statistics);
            log::info!(
                "Weighted residuals: {:?}\n\n",
                fit_statistics.weighted_residuals()
            );
            log::info!(
                "Regression standard error: {:?}\n\n",
                fit_statistics.regression_standard_error()
            );
            log::info!(
                "Covariance matrix: {:?}\n",
                fit_statistics.covariance_matrix()
            );

            let cov = fit_statistics.covariance_matrix();

            let weighted_residuals = fit_statistics.weighted_residuals();

            // Square the weighted residuals
            let squared_weighted_residuals: Vec<f64> = weighted_residuals
                .iter()
                .map(|&residual| residual * residual)
                .collect();

            // Sum up the squared weighted residuals to get the chi-squared value
            let chi_squared: f64 = squared_weighted_residuals.iter().sum();
            let dof = observation_length as f64 - 2.0;
            let rchi2 = chi_squared / dof;

            let sigma = 1.0;

            let prob = statrs::function::erf::erf(sigma / SQRT_2);

            let t_dist = match statrs::distribution::StudentsT::new(0.0, 1.0, dof) {
                Ok(dist) => dist.inverse_cdf((1.0 + prob) / 2.0),
                Err(e) => {
                    log::error!("Error creating StudentsT distribution: {:?}", e);
                    return;
                }
            };

            log::info!("Chi-squared: {:?}\n", chi_squared);
            log::info!("Reduced chi-squared: {:?}\n", rchi2);
            log::info!("Degrees of freedom: {:?}\n", dof);
            log::info!("T-distribution value: {:?}\n", t_dist);

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

            log::info!("fit_string: {:?}\n", fit_string);
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

            let fit_points: Vec<(f64, f64)> = (0..=num_points)
                .map(|i| {
                    let x = start + i as f64 * step;
                    let y = parameter_a * (-x / parameter_b).exp();

                    (x, y)
                })
                .collect();

            let confidence_band: Vec<(f64, f64)> = (0..=num_points)
                .map(|i| {
                    // followed lmfits implementation
                    let x = start + i as f64 * step;

                    let dfda = (-x / (parameter_b)).exp();
                    let dfdb = parameter_a * (x / parameter_b.powi(2)) * (-x / parameter_b).exp();

                    let y = t_dist
                        * (rchi2
                            * (dfda * dfda * cov[0]
                                + dfda * dfdb * cov[1]
                                + dfdb * dfda * cov[2]
                                + dfdb * dfdb * cov[3]))
                            .sqrt();
                    (x, y)
                })
                .collect();

            let lower_points: Vec<(f64, f64)> = fit_points
                .iter()
                .zip(confidence_band.iter())
                .map(|(fit_point, confidence_point)| {
                    (fit_point.0, fit_point.1 - confidence_point.1)
                })
                .collect();

            let upper_points: Vec<(f64, f64)> = fit_points
                .iter()
                .zip(confidence_band.iter())
                .map(|(fit_point, confidence_point)| {
                    (fit_point.0, fit_point.1 + confidence_point.1)
                })
                .collect();

            self.fit_line_points = fit_points;
            self.upper_uncertainity_points = upper_points;
            self.lower_uncertainity_points = lower_points;
        }
    }

    pub fn double_exp_fit(&mut self, initial_b_guess: f64, initial_d_guess: f64) {
        self.fit_params = None;
        self.fit_label = "".to_string();

        let x_data = DVector::from_vec(self.x.clone());
        let y_data = DVector::from_vec(self.y.clone());
        let weights = DVector::from_vec(self.weights.clone());

        let observation_length = x_data.len();

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
            log::info!("fit_result: {:?}\n", fit_result);
            log::info!("fit_statistics: {:?}\n", fit_statistics);
            log::info!(
                "Weighted residuals: {:?}\n",
                fit_statistics.weighted_residuals()
            );
            log::info!(
                "Regression standard error: {:?}\n",
                fit_statistics.regression_standard_error()
            );
            log::info!(
                "Covariance matrix: {:?}\n",
                fit_statistics.covariance_matrix()
            );

            let cov = fit_statistics.covariance_matrix();

            let weighted_residuals = fit_statistics.weighted_residuals();

            // Square the weighted residuals
            let squared_weighted_residuals: Vec<f64> = weighted_residuals
                .iter()
                .map(|&residual| residual * residual)
                .collect();

            // Sum up the squared weighted residuals to get the chi-squared value
            let chi_squared: f64 = squared_weighted_residuals.iter().sum();
            let dof = observation_length as f64 - 4.0;
            let rchi2 = chi_squared / dof;

            let sigma = 1.0;

            let prob = statrs::function::erf::erf(sigma / SQRT_2);

            let t_dist = match statrs::distribution::StudentsT::new(0.0, 1.0, dof) {
                Ok(dist) => dist.inverse_cdf((1.0 + prob) / 2.0),
                Err(e) => {
                    log::error!("Error creating StudentsT distribution: {:?}", e);
                    return;
                }
            };

            log::info!("Chi-squared: {:?}\n", chi_squared);
            log::info!("Reduced chi-squared: {:?}\n", rchi2);
            log::info!("Degrees of freedom: {:?}\n", dof);
            log::info!("T-distribution value: {:?}\n", t_dist);

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
            let end = max_x + 1000.0;

            let step = (end - start) / num_points as f64;

            let fit_points: Vec<(f64, f64)> = (0..=num_points)
                .map(|i| {
                    let x = start + i as f64 * step;
                    let y = parameter_a * (-x / parameter_b).exp()
                        + parameter_c * (-x / parameter_d).exp();

                    (x, y)
                })
                .collect();

            let confidence_band: Vec<(f64, f64)> = (0..=num_points)
                .map(|i| {
                    // followed lmfits implementation
                    let x = start + i as f64 * step;

                    let dfda = (-x / (parameter_b)).exp();
                    let dfdb = (-x / (parameter_d)).exp();
                    let dfdc = parameter_a * (x / parameter_b.powi(2)) * (-x / parameter_b).exp();
                    let dfdd = parameter_c * (x / parameter_d.powi(2)) * (-x / parameter_d).exp();

                    let y = t_dist
                        * (rchi2
                            * (dfda * dfda * cov[0]
                                + dfda * dfdb * cov[1]
                                + dfda * dfdc * cov[2]
                                + dfda * dfdd * cov[3]
                                + dfdb * dfda * cov[4]
                                + dfdb * dfdb * cov[5]
                                + dfdb * dfdc * cov[6]
                                + dfdb * dfdd * cov[7]
                                + dfdc * dfda * cov[8]
                                + dfdc * dfdb * cov[9]
                                + dfdc * dfdc * cov[10]
                                + dfdc * dfdd * cov[11]
                                + dfdd * dfda * cov[12]
                                + dfdd * dfdb * cov[13]
                                + dfdd * dfdc * cov[14]
                                + dfdd * dfdd * cov[15]))
                            .sqrt();

                    (x, y)
                })
                .collect();

            let lower_points: Vec<(f64, f64)> = fit_points
                .iter()
                .zip(confidence_band.iter())
                .map(|(fit_point, confidence_point)| {
                    (fit_point.0, fit_point.1 - confidence_point.1)
                })
                .collect();

            let upper_points: Vec<(f64, f64)> = fit_points
                .iter()
                .zip(confidence_band.iter())
                .map(|(fit_point, confidence_point)| {
                    (fit_point.0, fit_point.1 + confidence_point.1)
                })
                .collect();

            self.fit_line_points = fit_points;
            self.upper_uncertainity_points = upper_points;
            self.lower_uncertainity_points = lower_points;
        }
    }

    pub fn draw_fit_line(&self, plot_ui: &mut PlotUi, name: String) {
        // convert the fit line points to PlotPoints
        let fit_line_plot_points = self
            .fit_line_points
            .iter()
            .map(|(x, y)| PlotPoint::new(*x, *y))
            .collect();

        let line = Line::new(PlotPoints::Owned(fit_line_plot_points))
            .stroke(egui::Stroke::new(1.0, self.color))
            .name(name.clone());

        plot_ui.line(line);

        // convert the upper uncertainity points to PlotPoints
        let upper_uncertainity_plot_points: Vec<PlotPoint> = self
            .upper_uncertainity_points
            .iter()
            .map(|(x, y)| PlotPoint::new(*x, *y))
            .collect();
        let lower_uncertainity_plot_points: Vec<PlotPoint> = self
            .lower_uncertainity_points
            .iter()
            .map(|(x, y)| PlotPoint::new(*x, *y))
            .collect();

        // egui only supports convex polygons so i need to split the polygon into multiple.
        // So each polygon will be the two points in the upper and two in the lower
        // then the next polygon will be the next two points in the upper and lower
        // and so on
        let num_points = upper_uncertainity_plot_points.len() - 1;
        let mut polygons: Vec<Vec<PlotPoint>> = Vec::new();
        for i in 0..num_points {
            let polygon = vec![
                upper_uncertainity_plot_points[i],
                upper_uncertainity_plot_points[i + 1],
                lower_uncertainity_plot_points[i + 1],
                lower_uncertainity_plot_points[i],
            ];
            polygons.push(polygon);
        }

        for points in polygons.iter() {
            let uncertainity_band = Polygon::new(PlotPoints::Owned(points.clone()))
                .stroke(egui::Stroke::new(0.0, self.color))
                .width(0.0)
                .name(name.clone());

            plot_ui.polygon(uncertainity_band);
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
        ui.horizontal(|ui| {
            ui.label(self.name.to_string());

            if self.exp_fitter.is_some() {
                self.exp_fitter.as_mut().unwrap().color_ui(ui);
            }
        });

        ui.horizontal(|ui| {
            ui.add(
                egui::DragValue::new(&mut self.initial_b_guess)
                    .prefix("b: ")
                    .speed(10.0)
                    .clamp_range(0.0..=f64::INFINITY),
            );

            ui.add(
                egui::DragValue::new(&mut self.initial_d_guess)
                    .prefix("d: ")
                    .speed(10.0)
                    .clamp_range(0.0..=f64::INFINITY),
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
                    ui.label(format!("{:.1e} ± {:.1e}", a, a_uncertainty));

                    ui.label(format!("{:.1e} ± {:.1e}", b, b_uncertainty));
                }
            }
        }
    }
}
