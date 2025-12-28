/// Добавить отступ и метку
pub fn add_label(ui: &mut egui::Ui, text: &str) {
    ui.add_space(4.0);
    if !text.is_empty() {
        ui.label(egui::RichText::new(text).strong());
    }
}

/// Форматирование вероятностей в строку с разбиением на строки
#[must_use]
pub fn format_probabilities_row(probs: &[f64], prefix: &str, items_per_row: usize) -> Vec<String> {
    probs
        .chunks(items_per_row)
        .map(|chunk| {
            chunk
                .iter()
                .enumerate()
                .map(|(i, &p)| format!("{}{:>2} = {:>8.5}", prefix, i + 1, p))
                .collect::<Vec<_>>()
                .join("   ")
        })
        .collect()
}
