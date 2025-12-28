/// Форматирование вероятности для отображения
///
/// Обрабатывает реальные нули и очень маленькие значения, которые могли быть округлены до нуля
#[must_use]
pub fn format_probability(val: f64, compact: bool) -> String {
    // Проверяем, является ли значение реальным нулем или очень маленьким значением
    // Используем порог больше, чем f64::EPSILON, чтобы поймать значения, округленные до нуля
    const ZERO_THRESHOLD: f64 = 1e-10;

    if val.abs() < ZERO_THRESHOLD {
        // Реальный ноль или значение, округленное до нуля
        if compact {
            "0.000".to_string()
        } else {
            "0.00000".to_string()
        }
    } else if compact {
        if val < 0.001 {
            "<0.001".to_string()
        } else {
            format!("{val:.3}")
        }
    } else if val < 0.00001 {
        "<0.00001".to_string()
    } else {
        format!("{val:.5}")
    }
}
