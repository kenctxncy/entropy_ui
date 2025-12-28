use crate::state::experiments::ExperimentResult;

/// Вычисление среднего значения по результатам экспериментов
#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn calculate_average<F>(results: &[ExperimentResult], f: F) -> f64
where
    F: Fn(&ExperimentResult) -> f64,
{
    if results.is_empty() {
        return 0.0;
    }
    results.iter().map(f).sum::<f64>() / results.len() as f64
}
