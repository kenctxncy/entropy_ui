use super::probability::format_probability;
use entropy_fx::coding::systematic::BinaryMatrix;

/// Отображение бинарной матрицы в компактном виде
pub fn display_binary_matrix_compact(
    ui: &mut egui::Ui,
    matrix: &BinaryMatrix,
    title: &str,
    scroll_id: &str,
    grid_id: &str,
) {
    ui.label(egui::RichText::new(title).strong());

    let n = matrix.len();
    if n == 0 {
        return;
    }
    let m = matrix[0].len();
    let show_rows = (n / 8).max(1);
    let show_cols = (m / 8).max(1);

    egui::ScrollArea::horizontal()
        .id_salt(scroll_id)
        .max_width(ui.available_width())
        .show(ui, |ui| {
            egui::Grid::new(grid_id)
                .num_columns(show_cols * 2 + 3)
                .show(ui, |ui| {
                    ui.label(egui::RichText::new("").strong());
                    for j in 1..=show_cols {
                        ui.label(egui::RichText::new(format!("{j}")).strong());
                    }
                    ui.label(egui::RichText::new("...").strong());
                    for j in (m - show_cols + 1)..=m {
                        ui.label(egui::RichText::new(format!("{j}")).strong());
                    }
                    ui.end_row();

                    for (i, row) in matrix.iter().enumerate() {
                        if i < show_rows || i >= n - show_rows {
                            ui.label(egui::RichText::new(format!("{}", i + 1)).strong());
                            for val in row.iter().take(show_cols) {
                                ui.label(if *val { "1" } else { "0" });
                            }
                            ui.label("...");
                            for val in row.iter().skip(m - show_cols) {
                                ui.label(if *val { "1" } else { "0" });
                            }
                            ui.end_row();
                        } else if i == show_rows {
                            ui.label(egui::RichText::new("...").strong());
                            for _ in 0..show_cols {
                                ui.label("...");
                            }
                            ui.label("...");
                            for _ in (m - show_cols)..m {
                                ui.label("...");
                            }
                            ui.end_row();
                        }
                    }
                });
        });
}

/// Отображение бинарной матрицы в полном виде
pub fn display_binary_matrix_full(
    ui: &mut egui::Ui,
    matrix: &BinaryMatrix,
    title: &str,
    scroll_id: &str,
    grid_id: &str,
) {
    ui.label(egui::RichText::new(title).strong());

    let n = matrix.len();
    if n == 0 {
        return;
    }
    let m = matrix[0].len();

    egui::ScrollArea::horizontal()
        .id_salt(scroll_id)
        .max_width(ui.available_width())
        .show(ui, |ui| {
            egui::Grid::new(grid_id).num_columns(m + 1).show(ui, |ui| {
                // Заголовки столбцов
                ui.label(egui::RichText::new("").strong());
                for j in 1..=m {
                    ui.label(egui::RichText::new(format!("{j}")).strong());
                }
                ui.end_row();

                // Строки матрицы
                for (i, row) in matrix.iter().enumerate() {
                    ui.label(egui::RichText::new(format!("{}", i + 1)).strong());
                    for val in row {
                        ui.label(if *val { "1" } else { "0" });
                    }
                    ui.end_row();
                }
            });
        });
}

/// Отображение матрицы вероятностей в компактном виде
pub fn display_matrix_compact(
    ui: &mut egui::Ui,
    matrix: &[Vec<f64>],
    title: &str,
    scroll_id: &str,
    grid_id: &str,
) {
    ui.label(egui::RichText::new(title).strong());

    let n = matrix.len();
    let show_rows = (n / 8).max(1);
    let show_cols = (n / 8).max(1);

    egui::ScrollArea::horizontal()
        .id_salt(scroll_id)
        .max_width(ui.available_width())
        .show(ui, |ui| {
            egui::Grid::new(grid_id)
                .num_columns(show_cols * 2 + 3)
                .show(ui, |ui| {
                    // Заголовки столбцов
                    ui.label(egui::RichText::new("").strong());

                    // Первые столбцы
                    for j in 1..=show_cols {
                        ui.label(egui::RichText::new(format!("y{j}")).strong());
                    }

                    // Троеточие
                    ui.label(egui::RichText::new("...").strong());

                    // Последние столбцы
                    for j in (n - show_cols + 1)..=n {
                        ui.label(egui::RichText::new(format!("y{j}")).strong());
                    }
                    ui.end_row();

                    // Строки матрицы
                    for (i, row) in matrix.iter().enumerate() {
                        if i < show_rows || i >= n - show_rows {
                            ui.label(egui::RichText::new(format!("x{}", i + 1)).strong());

                            // Первые столбцы
                            for val in row.iter().take(show_cols) {
                                ui.label(format_probability(*val, true));
                            }

                            // Троеточие
                            ui.label("...");

                            // Последние столбцы
                            for val in row.iter().skip(n - show_cols) {
                                ui.label(format_probability(*val, true));
                            }
                            ui.end_row();
                        } else if i == show_rows {
                            // Строка с троеточиями
                            ui.label(egui::RichText::new("...").strong());
                            for _ in 0..show_cols {
                                ui.label("...");
                            }
                            ui.label("...");
                            for _ in (n - show_cols)..n {
                                ui.label("...");
                            }
                            ui.end_row();
                        }
                    }
                });
        });
}

/// Отображение матрицы вероятностей в полном виде
pub fn display_matrix_full(
    ui: &mut egui::Ui,
    matrix: &[Vec<f64>],
    title: &str,
    scroll_id: &str,
    grid_id: &str,
    signals: usize,
) {
    ui.label(egui::RichText::new(title).strong());

    egui::ScrollArea::horizontal()
        .id_salt(scroll_id)
        .max_width(ui.available_width())
        .show(ui, |ui| {
            egui::Grid::new(grid_id)
                .num_columns(signals + 1)
                .show(ui, |ui| {
                    // Заголовки столбцов
                    ui.label(egui::RichText::new("").strong());
                    for j in 0..signals {
                        ui.label(egui::RichText::new(format!("y{}", j + 1)).strong());
                    }
                    ui.end_row();

                    // Строки матрицы
                    for (i, row) in matrix.iter().enumerate() {
                        ui.label(egui::RichText::new(format!("x{}", i + 1)).strong());
                        for val in row {
                            ui.label(format_probability(*val, false));
                        }
                        ui.end_row();
                    }
                });
        });
}
