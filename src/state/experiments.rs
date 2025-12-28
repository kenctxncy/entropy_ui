/// Результат эксперимента для Labs 1-3
#[derive(Clone)]
pub struct ExperimentResult {
    pub input_probs: Vec<f64>,
    pub transition_matrix: Vec<Vec<f64>>,
    pub output_probs: Vec<f64>,
    pub joint_probs: Vec<Vec<f64>>,
    pub input_entropy: f64,
    pub conditional_entropy: f64,
    pub mutual_information: f64,
    // Для 3-й лабораторной
    pub symbol_durations: Vec<f64>,
    pub avg_duration: f64,
    pub information_rate_no_noise: f64,
    pub capacity_no_noise: f64,
    pub information_rate_with_noise: f64,
    pub capacity_with_noise: f64,
}

/// Состояние для Labs 1-3
pub struct Labs1To3State {
    pub experiments: usize,
    pub signals: usize,
    pub with_noise: bool,
    pub with_duration: bool,
    pub min_threshold: f64,
    pub compact_view: bool,
    pub results: Vec<ExperimentResult>,
}

impl Default for Labs1To3State {
    fn default() -> Self {
        Self {
            experiments: 6,
            signals: 9,
            with_noise: false,
            with_duration: false,
            min_threshold: 0.7,
            compact_view: false,
            results: vec![],
        }
    }
}
