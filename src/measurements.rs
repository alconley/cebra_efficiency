use super::detector::{Detector, DetectorLine};
use super::exp_fitter::Fitter;
use super::gamma_source::GammaSource;

use std::collections::{HashMap, HashSet};

use egui_plot::{Legend, Line, Plot, PlotPoints, Points, MarkerShape};

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct Measurement {
    pub gamma_source: GammaSource,
    pub detectors: Vec<Detector>,
}

impl Measurement {
    pub fn new(source: Option<GammaSource>) -> Self {
        Self {
            gamma_source: source.unwrap_or(GammaSource::new()),
            detectors: vec![],
        }
    }

    pub fn measurement_ui(&mut self, ui: &mut egui::Ui) {
        ui.collapsing("Measurement", |ui: &mut egui::Ui| {
            // ensure that there are gamma lines to display
            if self.gamma_source.gamma_lines.is_empty() {
                ui.label("No gamma lines added to source");
                return;
            }

            let mut index_to_remove = None;

            for (index, detector) in &mut self.detectors.iter_mut().enumerate() {
                detector.ui(ui, &self.gamma_source);

                if detector.to_remove == Some(true) {
                    index_to_remove = Some(index);
                }
            }

            ui.separator();

            if ui.button("Add Detector").clicked() {
                self.detectors.push(Detector::default());
            }

            if let Some(index) = index_to_remove {
                self.detectors.remove(index);
            }

            ui.separator();
        });
    }

    pub fn update_ui(&mut self, ui: &mut egui::Ui) {
        ui.collapsing(format!("{} Measurement", self.gamma_source.name), |ui| {
            self.gamma_source.source_ui(ui);
            self.measurement_ui(ui);
        });
    }

    pub fn reu_152eu_measurement(&mut self) {
        let mut source = GammaSource::new();

        source.fsu_152eu_source();
        source.measurement_time = 3.0; // 3 hours
        source.source_activity_measurement.date = chrono::NaiveDate::from_ymd_opt(2023, 6, 26);
        source.source_activity_measurement.activity = 53911.0; // kBq

        source.marker_shape = Some(MarkerShape::Circle);
        source.marker_size = 5.0;

        self.gamma_source = source;

        let mut cebra0 = Detector::default();
        cebra0.name = "Cebra0".to_string();
        cebra0.color = egui::Color32::RED;
        cebra0.lines.push(DetectorLine { energy: 244.6974, count: 324288.0, uncertainty: 1342.0, intensity: 7.55, intensity_uncertainty: 0.04, efficiency: 0.7378096466743758, efficiency_uncertainty: 0.037222437773747,});
        cebra0.lines.push(DetectorLine { energy: 344.2785, count: 989983.0, uncertainty: 1477.0, intensity: 26.59, intensity_uncertainty: 0.2, efficiency: 0.6395430771428688, efficiency_uncertainty: 0.0323510242911418,});
        cebra0.lines.push(DetectorLine { energy: 411.1164, count: 70407.0, uncertainty: 947.0, intensity: 2.237, intensity_uncertainty: 0.013, efficiency: 0.5406425939794314, efficiency_uncertainty: 0.028168901890432262,});
        cebra0.lines.push(DetectorLine { energy: 443.9650, count: 89143.0, uncertainty: 1046.0, intensity: 2.827, intensity_uncertainty: 0.014, efficiency: 0.541653856657235, efficiency_uncertainty: 0.02794750432891119,});
        cebra0.lines.push(DetectorLine { energy: 778.9045, count: 247655.0, uncertainty: 1047.0, intensity: 12.93, intensity_uncertainty: 0.08, efficiency: 0.32900990912509187, efficiency_uncertainty: 0.016634221841624493,});
        cebra0.lines.push(DetectorLine { energy: 867.3800, count: 74800.0, uncertainty: 1363.0, intensity: 4.23, intensity_uncertainty: 0.03, efficiency: 0.303753739389001, efficiency_uncertainty: 0.01630775110370631,});
        cebra0.lines.push(DetectorLine { energy: 964.0570, count: 214595.0, uncertainty: 840.0, intensity: 14.51, intensity_uncertainty: 0.07, efficiency: 0.25404613319294156, efficiency_uncertainty: 0.012799981786185114,});
        cebra0.lines.push(DetectorLine { energy: 1408.0130, count: 247320.0, uncertainty: 512.0, intensity: 20.87, intensity_uncertainty: 0.09, efficiency: 0.20356222599838772, efficiency_uncertainty: 0.010224585482807318,});
        self.detectors.push(cebra0);

        let mut cebra1 = Detector::default();
        cebra1.name = "Cebra1".to_string();
        cebra1.color = egui::Color32::GREEN;
        cebra1.lines.push(DetectorLine { energy: 244.6974, count: 301004.0, uncertainty: 1238.0, intensity: 7.55, intensity_uncertainty: 0.04, efficiency: 0.6848346373827394, efficiency_uncertainty: 0.03454843057390948,});
        cebra1.lines.push(DetectorLine { energy: 344.2785, count: 901978.0, uncertainty: 1372.0, intensity: 26.59, intensity_uncertainty: 0.2, efficiency: 0.5826905973488137, efficiency_uncertainty: 0.02947567088947358,});
        cebra1.lines.push(DetectorLine { energy: 411.1164, count: 66721.0, uncertainty: 1255.0, intensity: 2.237, intensity_uncertainty: 0.013, efficiency: 0.512338467949233, efficiency_uncertainty: 0.02753110493635109,});
        cebra1.lines.push(DetectorLine { energy: 443.9650, count: 79978.0, uncertainty: 1001.0, intensity: 2.827, intensity_uncertainty: 0.014, efficiency: 0.4859651587643712, efficiency_uncertainty: 0.02516330005784486,});
        cebra1.lines.push(DetectorLine { energy: 778.9045, count: 225612.0, uncertainty: 934.0, intensity: 12.93, intensity_uncertainty: 0.08, efficiency: 0.2997257621187952, efficiency_uncertainty: 0.015151483640032281,});
        cebra1.lines.push(DetectorLine { energy: 867.3800, count: 71647.0, uncertainty: 1277.0, intensity: 4.23, intensity_uncertainty: 0.03, efficiency: 0.29094978831555823, efficiency_uncertainty: 0.015581376552683619,});
        cebra1.lines.push(DetectorLine { energy: 964.0570, count: 194008.0, uncertainty: 784.0, intensity: 14.51, intensity_uncertainty: 0.07, efficiency: 0.22967442022645543, efficiency_uncertainty: 0.01157432323241182,});
        cebra1.lines.push(DetectorLine { energy: 1408.0130, count: 227715.0, uncertainty: 492.0, intensity: 20.87, intensity_uncertainty: 0.09, efficiency: 0.18742589476476978, efficiency_uncertainty: 0.009414798496900132,});
        self.detectors.push(cebra1);

        let mut cebra2 = Detector::default();
        cebra2.name = "Cebra2".to_string();
        cebra2.color = egui::Color32::BLUE;
        cebra2.lines.push(DetectorLine { energy: 244.6974, count: 126964.0, uncertainty: 1355.0, intensity: 7.55, intensity_uncertainty: 0.04, efficiency: 0.2888644167541366, efficiency_uncertainty: 0.01484765149215996,});
        cebra2.lines.push(DetectorLine { energy: 344.2785, count: 391036.0, uncertainty: 1088.0, intensity: 26.59, intensity_uncertainty: 0.2, efficiency: 0.252614809257976, efficiency_uncertainty: 0.012792181285746022,});
        cebra2.lines.push(DetectorLine { energy: 411.1164, count: 28952.0, uncertainty: 770.0, intensity: 2.237, intensity_uncertainty: 0.013, efficiency: 0.2223171613744727, efficiency_uncertainty: 0.012656673324939109,});
        cebra2.lines.push(DetectorLine { energy: 443.965, count: 37717.0, uncertainty: 808.0, intensity: 2.827, intensity_uncertainty: 0.014, efficiency: 0.2291773724413687, efficiency_uncertainty: 0.012517903981887422,});
        cebra2.lines.push(DetectorLine { energy: 778.9045, count: 108171.0, uncertainty: 773.0, intensity: 12.93, intensity_uncertainty: 0.08, efficiency: 0.14370527903725067, efficiency_uncertainty: 0.007312534039952126,});
        cebra2.lines.push(DetectorLine { energy: 867.38, count: 31165.0, uncertainty: 655.0, intensity: 4.23, intensity_uncertainty: 0.03, efficiency: 0.12655728994730236, efficiency_uncertainty: 0.0069226034484530415,});
        cebra2.lines.push(DetectorLine { energy: 964.057, count: 95611.0, uncertainty: 635.0, intensity: 14.51, intensity_uncertainty: 0.07, efficiency: 0.1131881210685726, efficiency_uncertainty: 0.0057351684168012685,});
        cebra2.lines.push(DetectorLine { energy: 1408.013, count: 115662.0, uncertainty: 352.0, intensity: 20.87, intensity_uncertainty: 0.09, efficiency: 0.09519818123655799, efficiency_uncertainty: 0.004786356717014694,});
        self.detectors.push(cebra2);

        let mut cebra3 = Detector::default();
        cebra3.name = "Cebra3".to_string();
        cebra3.color = egui::Color32::YELLOW;
        cebra3.lines.push(DetectorLine { energy: 244.6974, count: 297601.0, uncertainty: 1989.0, intensity: 7.55, intensity_uncertainty: 0.04, efficiency: 0.67709224103248, efficiency_uncertainty: 0.03434358011768675,});
        cebra3.lines.push(DetectorLine { energy: 344.2785, count: 875295.0, uncertainty: 2829.0, intensity: 26.59, intensity_uncertainty: 0.2, efficiency: 0.5654530004129035, efficiency_uncertainty: 0.028649115882505353,});
        cebra3.lines.push(DetectorLine { energy: 411.1164, count: 65273.0, uncertainty: 1151.0, intensity: 2.237, intensity_uncertainty: 0.013, efficiency: 0.5012195383529965, efficiency_uncertainty: 0.026732988393020542,});
        cebra3.lines.push(DetectorLine { energy: 443.965, count: 79091.0, uncertainty: 976.0, intensity: 2.827, intensity_uncertainty: 0.014, efficiency: 0.48057553792083924, efficiency_uncertainty: 0.024863947617984397,});
        cebra3.lines.push(DetectorLine { energy: 778.9045, count: 225010.0, uncertainty: 1196.0, intensity: 12.93, intensity_uncertainty: 0.08, efficiency: 0.2989260045314527, efficiency_uncertainty: 0.01514388048418208,});
        cebra3.lines.push(DetectorLine { energy: 867.38, count: 68767.0, uncertainty: 1440.0, intensity: 4.23, intensity_uncertainty: 0.03, efficiency: 0.27925445717330794, efficiency_uncertainty: 0.01526680430463775,});
        cebra3.lines.push(DetectorLine { energy: 964.057, count: 195864.0, uncertainty: 890.0, intensity: 14.51, intensity_uncertainty: 0.07, efficiency: 0.2318716271660677, efficiency_uncertainty: 0.01169497859500245,});
        cebra3.lines.push(DetectorLine { energy: 1408.013, count: 231393.0, uncertainty: 496.0, intensity: 20.87, intensity_uncertainty: 0.09, efficiency: 0.19045315445756483, efficiency_uncertainty: 0.009566724973943784,});
        self.detectors.push(cebra3);

        let mut cebra4 = Detector::default();
        cebra4.name = "Cebra4".to_string();
        cebra4.color = egui::Color32::GOLD;
        cebra4.lines.push(DetectorLine { energy: 244.6974, count: 705870.0, uncertainty: 3232.0, intensity: 7.55, intensity_uncertainty: 0.04, efficiency: 1.60597276278506, efficiency_uncertainty: 0.08108228481683838,});
        cebra4.lines.push(DetectorLine { energy: 344.2785, count: 2310831.0, uncertainty: 2526.0, intensity: 26.59, intensity_uncertainty: 0.2, efficiency: 1.492829643031378, efficiency_uncertainty: 0.07549896053086969,});
        cebra4.lines.push(DetectorLine { energy: 411.1164, count: 163826.0, uncertainty: 1303.0, intensity: 2.237, intensity_uncertainty: 0.013, efficiency: 1.257990165768664, efficiency_uncertainty: 0.06410852872180473,});
        cebra4.lines.push(DetectorLine { energy: 443.965, count: 207139.0, uncertainty: 1345.0, intensity: 2.827, intensity_uncertainty: 0.014, efficiency: 1.258625334733215, efficiency_uncertainty: 0.06376507867703363,});
        cebra4.lines.push(DetectorLine { energy: 778.9045, count: 741055.0, uncertainty: 1702.0, intensity: 12.93, intensity_uncertainty: 0.08, efficiency: 0.9844922905117802, efficiency_uncertainty: 0.04965156757423766,});
        cebra4.lines.push(DetectorLine { energy: 867.38, count: 208354.0, uncertainty: 2030.0, intensity: 4.23, intensity_uncertainty: 0.03, efficiency: 0.846100355837646, efficiency_uncertainty: 0.0435164308126922,});
        cebra4.lines.push(DetectorLine { energy: 964.057, count: 677200.0, uncertainty: 1442.0, intensity: 14.51, intensity_uncertainty: 0.07, efficiency: 0.8016964113714673, efficiency_uncertainty: 0.040307136872024925,});
        cebra4.lines.push(DetectorLine { energy: 1408.013, count: 867281.0, uncertainty: 966.0, intensity: 20.87, intensity_uncertainty: 0.09, efficiency: 0.7138349139823214, efficiency_uncertainty: 0.03583307282097205,});
        self.detectors.push(cebra4);

    }

    pub fn reu_56co_measurement(&mut self) {
        let mut source = GammaSource::new();

        source.fsu_56co_source();
        source.measurement_time = 15.0; // 3 hours
        source.source_activity_measurement.date = chrono::NaiveDate::from_ymd_opt(2023, 6, 26);
        source.source_activity_measurement.activity = 2199.1397987064543; // kBq

        source.marker_shape = Some(MarkerShape::Cross);
        source.marker_size = 6.0;
        self.gamma_source = source;

        let mut cebra0 = Detector::default();
        cebra0.name = "Cebra0".to_string();
        cebra0.color = egui::Color32::RED;
        cebra0.lines.push(DetectorLine { energy: 846.7638, count: 351162.0, uncertainty: 680.0, intensity: 99.9399, intensity_uncertainty: 0.0023, efficiency: 0.29588435701677696, efficiency_uncertainty: 0.014805310195605834,});
        cebra0.lines.push(DetectorLine { energy: 1037.8333, count: 38292.0, uncertainty: 340.0, intensity: 14.03, intensity_uncertainty: 0.05, efficiency: 0.22982844632438354, efficiency_uncertainty: 0.011699915266117836,});
        cebra0.lines.push(DetectorLine { energy: 1360.196, count: 8724.0, uncertainty: 213.0, intensity: 4.283, intensity_uncertainty: 0.013, efficiency: 0.17152245514112735, efficiency_uncertainty: 0.00955816451809105,});
        cebra0.lines.push(DetectorLine { energy: 2598.438, count: 24091.0, uncertainty: 182.0, intensity: 16.96, intensity_uncertainty: 0.04, efficiency: 0.11961410131023684, efficiency_uncertainty: 0.006055162822645534,});
        cebra0.lines.push(DetectorLine { energy: 3451.119, count: 995.0, uncertainty: 38.0, intensity: 0.942, intensity_uncertainty: 0.006, efficiency: 0.08894582750376995, efficiency_uncertainty: 0.0056248082665797915,});
        self.detectors.push(cebra0);

        let mut cebra1 = Detector::default();
        cebra1.name = "Cebra1".to_string();
        cebra1.color = egui::Color32::GREEN;
        cebra1.lines.push(DetectorLine { energy: 846.7638, count: 335043.0, uncertainty: 664.0, intensity: 99.9399, intensity_uncertainty: 0.0023, efficiency: 0.2823027053837602, efficiency_uncertainty: 0.014126220332089892,});
        cebra1.lines.push(DetectorLine { energy: 1037.8333, count: 35468.0, uncertainty: 327.0, intensity: 14.03, intensity_uncertainty: 0.05, efficiency: 0.2128788084778344, efficiency_uncertainty: 0.010849932369848315,});
        cebra1.lines.push(DetectorLine { energy: 1360.196, count: 10393.0, uncertainty: 220.0, intensity: 4.283, intensity_uncertainty: 0.013, efficiency: 0.2043366433151922, efficiency_uncertainty: 0.011112045683841642,});
        cebra1.lines.push(DetectorLine { energy: 2598.438, count: 22824.0, uncertainty: 179.0, intensity: 16.96, intensity_uncertainty: 0.04, efficiency: 0.1133233260680273, efficiency_uncertainty: 0.005741668310944598,});
        cebra1.lines.push(DetectorLine { energy: 3451.119, count: 953.0, uncertainty: 39.0, intensity: 0.942, intensity_uncertainty: 0.006, efficiency: 0.08519133026240479, efficiency_uncertainty: 0.005531072456500621,});
        self.detectors.push(cebra1);

        let mut cebra2 = Detector::default();
        cebra2.name = "Cebra2".to_string();
        cebra2.color = egui::Color32::BLUE;
        cebra2.lines.push(DetectorLine { energy: 846.7638, count: 164290.0, uncertainty: 491.0, intensity: 99.9399, intensity_uncertainty: 0.0023, efficiency: 0.13842853444930342, efficiency_uncertainty: 0.006933780639742302,});
        cebra2.lines.push(DetectorLine { energy: 1037.8333, count: 17223.0, uncertainty: 259.0, intensity: 14.03, intensity_uncertainty: 0.05, efficiency: 0.10337238407617406, efficiency_uncertainty: 0.0054098858240088235,});
        cebra2.lines.push(DetectorLine { energy: 1360.196, count: 5533.0, uncertainty: 200.0, intensity: 4.283, intensity_uncertainty: 0.013, efficiency: 0.1087842439587182, efficiency_uncertainty: 0.006719838588684058,});
        cebra2.lines.push(DetectorLine { energy: 2598.438, count: 13329.0, uncertainty: 134.0, intensity: 16.96, intensity_uncertainty: 0.04, efficiency: 0.06617974996322888, efficiency_uncertainty: 0.0033788185648905194,});
        cebra2.lines.push(DetectorLine { energy: 3451.119, count: 459.0, uncertainty: 27.0, intensity: 0.942, intensity_uncertainty: 0.006, efficiency: 0.04103129128063357, efficiency_uncertainty: 0.003178475997906439,});
        self.detectors.push(cebra2);

        let mut cebra3 = Detector::default();
        cebra3.name = "Cebra3".to_string();
        cebra3.color = egui::Color32::YELLOW;
        cebra3.lines.push(DetectorLine { energy: 846.7638, count: 323478.0, uncertainty: 668.0, intensity: 99.9399, intensity_uncertainty: 0.0023, efficiency: 0.27255819262640313, efficiency_uncertainty: 0.013639529239796323,});
        cebra3.lines.push(DetectorLine { energy: 1037.8333, count: 36790.0, uncertainty: 359.0, intensity: 14.03, intensity_uncertainty: 0.05, efficiency: 0.2208134477246963, efficiency_uncertainty: 0.011276458426854059,});
        cebra3.lines.push(DetectorLine { energy: 1360.196, count: 6868.0, uncertainty: 217.0, intensity: 4.283, intensity_uncertainty: 0.013, efficiency: 0.13503166230046568, efficiency_uncertainty: 0.007997144350104906,});
        cebra3.lines.push(DetectorLine { energy: 2598.438, count: 22302.0, uncertainty: 176.0, intensity: 16.96, intensity_uncertainty: 0.04, efficiency: 0.11073154652861657, efficiency_uncertainty: 0.00561119579170995,});
        cebra3.lines.push(DetectorLine { energy: 3451.119, count: 874.0, uncertainty: 37.0, intensity: 0.942, intensity_uncertainty: 0.006, efficiency: 0.07812929973697982, efficiency_uncertainty: 0.005142751165036958,});
        self.detectors.push(cebra3);

        let mut cebra4 = Detector::default();
        cebra4.name = "Cebra4".to_string();
        cebra4.color = egui::Color32::GOLD;
        cebra4.lines.push(DetectorLine { energy: 846.7638, count: 1064178.0, uncertainty: 1186.0, intensity: 99.9399, intensity_uncertainty: 0.0023, efficiency: 0.8966620058018796, efficiency_uncertainty: 0.044844240690244806,});
        cebra4.lines.push(DetectorLine { energy: 1037.8333, count: 126858.0, uncertainty: 608.0, intensity: 14.03, intensity_uncertainty: 0.05, efficiency: 0.7614012598928926, efficiency_uncertainty: 0.03834070171286066,});
        cebra4.lines.push(DetectorLine { energy: 1360.196, count: 38511.0, uncertainty: 414.0, intensity: 4.283, intensity_uncertainty: 0.013, efficiency: 0.7571642904562076, efficiency_uncertainty: 0.03879149332523254,});
        cebra4.lines.push(DetectorLine { energy: 2598.438, count: 108362.0, uncertainty: 372.0, intensity: 16.96, intensity_uncertainty: 0.04, efficiency: 0.5380276138881692, efficiency_uncertainty: 0.026994553893109834,});
        cebra4.lines.push(DetectorLine { energy: 3451.119, count: 4920.0, uncertainty: 86.0, intensity: 0.942, intensity_uncertainty: 0.006, efficiency: 0.4398125339884906, efficiency_uncertainty: 0.0234635293089937,});
        self.detectors.push(cebra4);

    }
        
}

#[derive(Default, Clone, serde::Deserialize, serde::Serialize)]
pub struct MeasurementHandler {
    pub measurements: Vec<Measurement>,
    pub measurement_exp_fits: HashMap<String, Fitter>
}

impl MeasurementHandler {
    pub fn new() -> Self {
        Self {
            measurements: vec![],
            measurement_exp_fits: HashMap::new(),
        }
    }

    pub fn reu_2023_efficiency(&mut self) {
        self.measurements.clear();

        let mut eu152_measurement = Measurement::new(None);
        eu152_measurement.reu_152eu_measurement();
        self.measurements.push(eu152_measurement);

        let mut co56_measurement = Measurement::new(None);
        co56_measurement.reu_56co_measurement();
        self.measurements.push(co56_measurement);
    }

    fn synchronize_detectors(&mut self) {
        let mut detector_names: HashSet<String> = HashSet::new();
        let mut detector_data: HashMap<String, (Vec<f64>, Vec<f64>, Vec<f64>)> = HashMap::new();
    
        // Collect all detector names from measurements and compute data
        for measurement in &self.measurements {
            for detector in &measurement.detectors {
                let name = &detector.name;
                detector_names.insert(name.clone());
                let data = self.get_detector_data_from_measurements(name.clone());
                detector_data.insert(name.clone(), data);
            }
        }
    
        // Iterate over detector names
        for name in &detector_names {
            // Insert if not exists
            self.measurement_exp_fits.entry(name.clone()).or_insert_with(|| Fitter::default());
    
            // Update Fitter with pre-computed data
            if let Some(fitter) = self.measurement_exp_fits.get_mut(name) {
                if let Some(data) = detector_data.get(name) {
                    fitter.name = name.clone();
                    fitter.data = data.clone();
                }
            }
        }
    
        // Remove entries in the hashmap that don't correspond to any detector in measurements
        let keys: HashSet<String> = self.measurement_exp_fits.keys().cloned().collect();
        for key in keys {
            if !detector_names.contains(&key) {
                self.measurement_exp_fits.remove(&key);
            }
        }
    }
    
    fn get_detector_data_from_measurements(&self, name: String) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
        let mut x_data: Vec<f64> = vec![];
        let mut y_data: Vec<f64> = vec![];
        let mut weights: Vec<f64> = vec![];

        for measurement in &self.measurements {
            for detector in &measurement.detectors {
                if detector.name == name {
                    for line in &detector.lines {
                        x_data.push(line.energy);
                        y_data.push(line.efficiency);
                        weights.push(1.0 / line.efficiency_uncertainty);
                    }
                }
            }
        }

        (x_data, y_data, weights)
    }

    fn fit_detectors_ui(&mut self, ui: &mut egui::Ui) {
        
        self.synchronize_detectors(); // Ensure synchronization before fitting UI

        ui.label("Fit Equation: y = a * exp[-x/b] + c * exp[-x/d]");

        egui::ScrollArea::both().show(ui, |ui| {
            
            ui.separator();

            egui::Grid::new("detector_grid")
                .striped(true)
                .show(ui, |ui| {

                    ui.label("Detector Name");

                    ui.label("Initial Guesses");

                    ui.label("Exponential Fitter");

                    ui.label("Results");
                    ui.label("a");
                    ui.label("b");
                    ui.label("c");
                    ui.label("d");

                    ui.end_row();

                    for (name, fitter) in &mut self.measurement_exp_fits {
                        fitter.name = name.clone();
                        fitter.ui(ui);
                        ui.end_row();
                    }

            });

        });

    }

    fn remove_measurement(&mut self, index: usize) {
        self.measurements.remove(index);
    }

    pub fn plot(&mut self, ui: &mut egui::Ui) {
        let plot = Plot::new("Efficiency")
            .legend(Legend::default())
            .min_size(egui::Vec2::new(400.0, 400.0));

        plot.show(ui, |plot_ui| {

            for measurement in self.measurements.iter_mut() {

                let marker_shape = measurement.gamma_source.marker_shape.unwrap();
                let marker_size = measurement.gamma_source.marker_size;

                for detector in measurement.detectors.iter_mut() {

                    let color = detector.color;
                    let name = format!("{}: {}", detector.name, measurement.gamma_source.name);

                    let mut points: Vec<[f64; 2]> = vec![];
                    for detector_line in &detector.lines {
                        points.push([detector_line.energy, detector_line.efficiency]);
                    }

                    let detector_plot_points = PlotPoints::new(points);

                    let detector_points = Points::new(detector_plot_points)
                        .filled(true)
                        .color(color)
                        .shape(marker_shape)
                        .radius(marker_size)
                        .name(name.to_string());

                    plot_ui.points(detector_points);

                    // draw the uncertainity as vertical lines from the efficiency points
                    for detector_line in &detector.lines {
                        let mut y_err_points: Vec<[f64; 2]> = vec![];
                        y_err_points.push([
                            detector_line.energy,
                            detector_line.efficiency - detector_line.efficiency_uncertainty,
                        ]);
                        y_err_points.push([
                            detector_line.energy,
                            detector_line.efficiency + detector_line.efficiency_uncertainty,
                        ]);

                        let y_err_plot_points = PlotPoints::new(y_err_points);

                        // this can be a line with the points at (x, y-yerr) and (x, y+yerr)
                        let y_err_points = Line::new(y_err_plot_points)
                            .color(color)
                            .name(name.to_string());

                        plot_ui.line(y_err_points);
                    }
                }
            }

            for (name, fitter) in self.measurement_exp_fits.iter_mut() {
                // check to see if the exp is not none
                if let Some(exp_fit) = &mut fitter.exp_fitter {
                    exp_fit.draw_fit_line(plot_ui, name.clone());
                }
            }

        });
    }

    pub fn sources_ui(&mut self, ui: &mut egui::Ui) {
        let mut index_to_remove: Option<usize> = None;

        // Previous measurments
        ui.horizontal(|ui| {
            ui.label("Previous Measurements");
            if ui.button("REU 2023").clicked() {
                self.reu_2023_efficiency();
            }
        });

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.collapsing("Sources", |ui| {
                for (index, measurement) in self.measurements.iter_mut().enumerate() {
                    measurement.update_ui(ui);

                    if ui.button("Remove Source").clicked() {
                        index_to_remove = Some(index);
                    }

                    ui.separator();
                }

                if let Some(index) = index_to_remove {
                    self.remove_measurement(index);
                }

                if ui.button("New Source").clicked() {
                    self.measurements.push(Measurement::new(None));
                }

                ui.separator();
            });

            ui.collapsing("Fitter", |ui| {
                self.fit_detectors_ui(ui);
            });

        });
    }

}
