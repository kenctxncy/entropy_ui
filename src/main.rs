//Finished 1st lab, modified for 2nd lab with noise
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
use eframe::{App, Frame, egui};
use entropy_fx::coding::hamming::{
    HammingCode, HammingErrorInfo, add_parity_bit, compute_hamming_n_from_k,
    compute_syndrome_hamming, decode_hamming, encode_hamming, generate_error_multiplicity,
    inject_errors,
};
use entropy_fx::coding::systematic::{
    BinaryMatrix, ErrorInfo, SystematicCode, build_generator_matrix, compute_n_from_k,
    compute_syndrome, correct_error, encode_message, inject_single_error,
};
use entropy_fx::{
    calc_entropy, calculate_average_duration, calculate_capacity_no_noise,
    calculate_capacity_with_noise, calculate_conditional_entropy,
    calculate_information_rate_no_noise, calculate_information_rate_with_noise,
    calculate_joint_probabilities, calculate_mutual_information, calculate_output_probabilities,
    format_rate, generate_error_probability_matrix, generate_probabilities,
    generate_symbol_durations, generate_transition_matrix, max_entropy,
};
use rand::Rng;
use std::fmt::Write;

impl InfoEntropyApp {
    /// Format binary vector as string
    #[must_use]
    fn format_bits(bits: &[bool]) -> String {
        bits.iter().map(|&b| if b { '1' } else { '0' }).collect()
    }

    /// Helper to add spacing and label
    fn add_label(ui: &mut egui::Ui, text: &str) {
        ui.add_space(4.0);
        if !text.is_empty() {
            ui.label(egui::RichText::new(text).strong());
        }
    }

    fn display_binary_matrix_compact(
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

    fn display_binary_matrix_full(
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

    fn display_matrix_compact(
        ui: &mut egui::Ui,
        matrix: &[Vec<f64>],
        title: &str,
        scroll_id: &str,
        grid_id: &str,
    ) {
        ui.label(egui::RichText::new(title).strong());

        let n = matrix.len();
        let show_rows = (n / 8).max(1); // Показываем 1/8 строк
        let show_cols = (n / 8).max(1); // Показываем 1/8 столбцов

        egui::ScrollArea::horizontal()
            .id_salt(scroll_id)
            .max_width(ui.available_width())
            .show(ui, |ui| {
                egui::Grid::new(grid_id)
                    .num_columns(show_cols * 2 + 3) // +3 для заголовков и троеточий
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
                                    ui.label(format!("{val:.3}"));
                                }

                                // Троеточие
                                ui.label("...");

                                // Последние столбцы
                                for val in row.iter().skip(n - show_cols) {
                                    ui.label(format!("{val:.3}"));
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
}

#[derive(Clone)]
struct ExperimentResult {
    input_probs: Vec<f64>,
    transition_matrix: Vec<Vec<f64>>,
    output_probs: Vec<f64>,
    joint_probs: Vec<Vec<f64>>,
    input_entropy: f64,
    conditional_entropy: f64,
    mutual_information: f64,
    // Для 3-й лабораторной
    symbol_durations: Vec<f64>,
    avg_duration: f64,
    information_rate_no_noise: f64,
    capacity_no_noise: f64,
    information_rate_with_noise: f64,
    capacity_with_noise: f64,
}

#[derive(Clone)]
enum ErrorInfoType {
    Systematic(ErrorInfo),
    Hamming(HammingErrorInfo),
}

#[derive(Clone)]
struct Labs4To6ExperimentResult {
    code_type: CodeType,
    message: Vec<bool>,
    codeword: Vec<bool>,
    codeword_with_parity: Option<Vec<bool>>, // Only for Hamming code
    error_multiplicity: usize,
    error_positions: Vec<usize>,
    received: Vec<bool>,
    syndrome: Vec<bool>,
    overall_parity: Option<bool>, // Only for Hamming code
    corrected: Vec<bool>,
    error_info: ErrorInfoType,
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum CodeType {
    Systematic,
    Hamming,
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum LabMode {
    Labs1To3,
    Labs4To6,
}

#[allow(clippy::struct_excessive_bools)]
struct InfoEntropyApp {
    lab_mode: LabMode,
    experiments: usize,
    signals: usize,
    with_noise: bool,
    with_duration: bool,
    min_threshold: f64,
    compact_view: bool,
    results: Vec<ExperimentResult>,
    // Labs 4-6 state
    labs4to6_k: usize,
    labs4to6_n: usize,
    labs4to6_p: usize,
    labs4to6_experiments: usize,
    labs4to6_error_probability: f64,
    labs4to6_use_hamming: bool,
    labs4to6_systematic_code: Option<SystematicCode>,
    labs4to6_hamming_code: Option<HammingCode>,
    labs4to6_results: Vec<Labs4To6ExperimentResult>,
}

impl Default for InfoEntropyApp {
    fn default() -> Self {
        let k = 60;
        let (n, p) = compute_n_from_k(k);
        Self {
            lab_mode: LabMode::Labs1To3,
            experiments: 6,
            signals: 9,
            with_noise: false,
            with_duration: false,
            min_threshold: 0.7,
            compact_view: false,
            results: vec![],
            labs4to6_k: k,
            labs4to6_n: n,
            labs4to6_p: p,
            labs4to6_experiments: 6,
            labs4to6_error_probability: 0.5,
            labs4to6_use_hamming: false,
            labs4to6_systematic_code: None,
            labs4to6_hamming_code: None,
            labs4to6_results: vec![],
        }
    }
}

impl App for InfoEntropyApp {
    #[allow(clippy::too_many_lines)]
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        // Request repaint to ensure window renders
        ctx.request_repaint();

        egui::CentralPanel::default().show(ctx, |ui| {
            // Mode selector
            ui.horizontal(|ui| {
                ui.label("Режим работы:");
                ui.radio_value(
                    &mut self.lab_mode,
                    LabMode::Labs1To3,
                    "Лабораторные работы 1-3",
                );
                ui.radio_value(
                    &mut self.lab_mode,
                    LabMode::Labs4To6,
                    "Лабораторные работы 4-6",
                );
            });
            ui.separator();

            match &mut self.lab_mode {
                LabMode::Labs4To6 => {
                    self.render_labs4to6_ui(ui);
                }
                LabMode::Labs1To3 => {
                    self.render_labs1to3_ui(ui);
                }
            }
        });
    }
}

impl InfoEntropyApp {
    #[allow(clippy::too_many_lines)]
    fn render_labs1to3_ui(&mut self, ui: &mut egui::Ui) {
        let title = if self.with_duration {
            "Обобщенные характеристики сигналов и каналов"
        } else {
            "Симуляция энтропии дискретных сообщений с учетом помех"
        };
        ui.heading(egui::RichText::new(title).size(18.0));

        ui.horizontal(|ui| {
            ui.label("Количество экспериментов:");
            ui.add(egui::DragValue::new(&mut self.experiments).range(1..=100));
        });

        ui.horizontal(|ui| {
            ui.label("Количество дискретных сообщений (p_i):");
            ui.add(egui::DragValue::new(&mut self.signals).range(2..=128));
        });

        ui.horizontal(|ui| {
            ui.checkbox(&mut self.with_noise, "Учитывать помехи");
            ui.checkbox(
                &mut self.with_duration,
                "Режим расчета матрицы ошибок (по длительности и кол-ву символов)",
            );
            ui.checkbox(&mut self.compact_view, "Сокращенный вид матриц");
        });

        if self.with_noise && !self.with_duration {
            ui.horizontal(|ui| {
                ui.label("Минимальный порог достоверности:");
                ui.add(
                    egui::DragValue::new(&mut self.min_threshold)
                        .range(0.0..=1.0)
                        .speed(0.01),
                );
            });
        }

        ui.separator();

        // Run experiments button (in separate section for consistent styling)
        if ui.button("Запустить эксперименты").clicked() {
            self.results.clear();
            for _ in 0..self.experiments {
                let input_probs = generate_probabilities(self.signals);
                let input_entropy = calc_entropy(&input_probs);

                // Для 3-й лабораторной: генерируем длительности и рассчитываем среднюю
                let symbol_durations = if self.with_duration {
                    generate_symbol_durations(self.signals)
                } else {
                    vec![]
                };
                let avg_duration = if self.with_duration {
                    calculate_average_duration(&input_probs, &symbol_durations)
                } else {
                    0.0
                };

                // Расчет для канала без помех (3-я лабораторная)
                let information_rate_no_noise = if self.with_duration {
                    calculate_information_rate_no_noise(input_entropy, avg_duration)
                } else {
                    0.0
                };
                let capacity_no_noise = if self.with_duration {
                    calculate_capacity_no_noise(self.signals, avg_duration)
                } else {
                    0.0
                };

                if self.with_noise {
                    // Используем матрицу ошибок для 3-й лабораторной или матрицу переходов для 2-й
                    let transition_matrix = if self.with_duration {
                        generate_error_probability_matrix(self.signals)
                    } else {
                        generate_transition_matrix(self.signals, self.min_threshold)
                    };
                    let output_probs =
                        calculate_output_probabilities(&input_probs, &transition_matrix);
                    let joint_probs = calculate_joint_probabilities(
                        &input_probs,
                        &output_probs,
                        &transition_matrix,
                    );
                    let conditional_entropy =
                        calculate_conditional_entropy(&joint_probs, &transition_matrix);
                    let mutual_information =
                        calculate_mutual_information(input_entropy, conditional_entropy);

                    // Расчет для канала с помехами (3-я лабораторная)
                    let information_rate_with_noise = if self.with_duration {
                        calculate_information_rate_with_noise(
                            input_entropy,
                            conditional_entropy,
                            avg_duration,
                        )
                    } else {
                        0.0
                    };
                    let capacity_with_noise = if self.with_duration {
                        calculate_capacity_with_noise(
                            self.signals,
                            conditional_entropy,
                            avg_duration,
                        )
                    } else {
                        0.0
                    };

                    self.results.push(ExperimentResult {
                        input_probs,
                        transition_matrix,
                        output_probs,
                        joint_probs,
                        input_entropy,
                        conditional_entropy,
                        mutual_information,
                        symbol_durations,
                        avg_duration,
                        information_rate_no_noise,
                        capacity_no_noise,
                        information_rate_with_noise,
                        capacity_with_noise,
                    });
                } else {
                    // Для первой лабораторной - только энтропия
                    self.results.push(ExperimentResult {
                        input_probs,
                        transition_matrix: vec![],
                        output_probs: vec![],
                        joint_probs: vec![],
                        input_entropy,
                        conditional_entropy: 0.0,
                        mutual_information: input_entropy,
                        symbol_durations,
                        avg_duration,
                        information_rate_no_noise,
                        capacity_no_noise,
                        information_rate_with_noise: 0.0,
                        capacity_with_noise: 0.0,
                    });
                }
            }
        }

        ui.separator();

        if !self.results.is_empty() {
            #[allow(clippy::cast_precision_loss)]
            let avg_mutual_info: f64 = self
                .results
                .iter()
                .map(|r| r.mutual_information)
                .sum::<f64>()
                / self.results.len() as f64;

            #[allow(clippy::cast_precision_loss)]
            let avg_input_entropy: f64 = self.results.iter().map(|r| r.input_entropy).sum::<f64>()
                / self.results.len() as f64;

            ui.label(
                egui::RichText::new(format!(
                    "Среднее количество информации: {avg_mutual_info:.5}"
                ))
                .strong(),
            );
            ui.label(
                egui::RichText::new(format!("Средняя энтропия на входе: {avg_input_entropy:.5}"))
                    .strong(),
            );
            ui.label(
                egui::RichText::new(format!(
                    "Максимальная энтропия: {:.5}",
                    max_entropy(self.signals)
                ))
                .strong(),
            );

            if self.with_noise {
                #[allow(clippy::cast_precision_loss)]
                let avg_conditional_entropy: f64 = self
                    .results
                    .iter()
                    .map(|r| r.conditional_entropy)
                    .sum::<f64>()
                    / self.results.len() as f64;
                ui.label(
                    egui::RichText::new(format!(
                        "Средняя условная энтропия: {avg_conditional_entropy:.5}"
                    ))
                    .strong(),
                );
            }

            if self.with_duration {
                #[allow(clippy::cast_precision_loss)]
                let avg_duration: f64 = self.results.iter().map(|r| r.avg_duration).sum::<f64>()
                    / self.results.len() as f64;
                ui.label(
                    egui::RichText::new(format!(
                        "Средняя длительность символа τ: {avg_duration:.5} мкс"
                    ))
                    .strong(),
                );

                #[allow(clippy::cast_precision_loss)]
                let avg_rate_no_noise: f64 = self
                    .results
                    .iter()
                    .map(|r| r.information_rate_no_noise)
                    .sum::<f64>()
                    / self.results.len() as f64;
                ui.label(
                    egui::RichText::new(format!(
                        "Средняя скорость передачи (без помех): {}",
                        format_rate(avg_rate_no_noise)
                    ))
                    .strong(),
                );

                #[allow(clippy::cast_precision_loss)]
                let avg_capacity_no_noise: f64 = self
                    .results
                    .iter()
                    .map(|r| r.capacity_no_noise)
                    .sum::<f64>()
                    / self.results.len() as f64;
                ui.label(
                    egui::RichText::new(format!(
                        "Средняя пропускная способность (без помех): {}",
                        format_rate(avg_capacity_no_noise)
                    ))
                    .strong(),
                );

                if self.with_noise {
                    #[allow(clippy::cast_precision_loss)]
                    let avg_rate_with_noise: f64 = self
                        .results
                        .iter()
                        .map(|r| r.information_rate_with_noise)
                        .sum::<f64>()
                        / self.results.len() as f64;
                    ui.label(
                        egui::RichText::new(format!(
                            "Средняя скорость передачи (с помехами): {}",
                            format_rate(avg_rate_with_noise)
                        ))
                        .strong(),
                    );

                    #[allow(clippy::cast_precision_loss)]
                    let avg_capacity_with_noise: f64 = self
                        .results
                        .iter()
                        .map(|r| r.capacity_with_noise)
                        .sum::<f64>()
                        / self.results.len() as f64;
                    ui.label(
                        egui::RichText::new(format!(
                            "Средняя пропускная способность (с помехами): {}",
                            format_rate(avg_capacity_with_noise)
                        ))
                        .strong(),
                    );
                }
            }

            ui.separator();

            egui::ScrollArea::vertical()
                .id_salt("experiments_scroll")
                .auto_shrink([false; 2])
                .max_height(ui.available_height())
                .show(ui, |ui| {
                    for (i, result) in self.results.iter().enumerate() {
                        ui.collapsing(format!("Эксперимент #{}", i + 1), |ui| {
                            ui.label(
                                egui::RichText::new("Вероятности сообщений на входе (p_i):")
                                    .strong(),
                            );
                            let mut row = String::new();
                            for (j, p) in result.input_probs.iter().enumerate() {
                                write!(row, "x{:>2} = {:>8.5}   ", j + 1, p).unwrap();
                                if (j + 1) % 4 == 0 {
                                    ui.label(row.clone());
                                    row.clear();
                                }
                            }
                            if !row.is_empty() {
                                ui.label(row);
                            }

                            ui.add_space(4.0);
                            ui.label(
                                egui::RichText::new(format!(
                                    "Энтропия на входе H(X): {:.5}",
                                    result.input_entropy
                                ))
                                .strong(),
                            );

                            if self.with_duration {
                                ui.add_space(4.0);
                                ui.label(
                                    egui::RichText::new("Длительности символов (мкс):").strong(),
                                );
                                let mut row = String::new();
                                for (j, t) in result.symbol_durations.iter().enumerate() {
                                    write!(row, "T{:>2} = {:>8.5}   ", j + 1, t).unwrap();
                                    if (j + 1) % 4 == 0 {
                                        ui.label(row.clone());
                                        row.clear();
                                    }
                                }
                                if !row.is_empty() {
                                    ui.label(row);
                                }

                                ui.add_space(4.0);
                                ui.label(
                                    egui::RichText::new(format!(
                                        "Средняя длительность символа τ: {:.5} мкс",
                                        result.avg_duration
                                    ))
                                    .strong(),
                                );

                                ui.add_space(4.0);
                                ui.label(
                                    egui::RichText::new(format!(
                                        "Скорость передачи (без помех): {}",
                                        format_rate(result.information_rate_no_noise)
                                    ))
                                    .strong(),
                                );

                                ui.label(
                                    egui::RichText::new(format!(
                                        "Пропускная способность (без помех): {}",
                                        format_rate(result.capacity_no_noise)
                                    ))
                                    .strong(),
                                );
                            }

                            if self.with_noise {
                                ui.add_space(4.0);
                                ui.label(
                                    egui::RichText::new("Вероятности на выходе (p_y):").strong(),
                                );
                                let mut row = String::new();
                                for (j, p) in result.output_probs.iter().enumerate() {
                                    write!(row, "y{:>2} = {:>8.5}   ", j + 1, p).unwrap();
                                    if (j + 1) % 4 == 0 {
                                        ui.label(row.clone());
                                        row.clear();
                                    }
                                }
                                if !row.is_empty() {
                                    ui.label(row);
                                }

                                ui.add_space(4.0);

                                if self.compact_view {
                                    Self::display_matrix_compact(
                                        ui,
                                        &result.transition_matrix,
                                        "Матрица переходов p(x_i/y_j):",
                                        &format!("scroll_transition_{i}"),
                                        &format!("grid_transition_{i}"),
                                    );
                                } else {
                                    ui.label(
                                        egui::RichText::new("Матрица переходов p(x_i/y_j):")
                                            .strong(),
                                    );

                                    // Создаем прокручиваемую таблицу для матрицы переходов
                                    egui::ScrollArea::horizontal()
                                        .id_salt(format!("scroll_transition_full_{i}"))
                                        .max_width(ui.available_width())
                                        .show(ui, |ui| {
                                            egui::Grid::new(format!("grid_transition_full_{i}"))
                                                .num_columns(self.signals + 1)
                                                .show(ui, |ui| {
                                                    // Заголовки столбцов
                                                    ui.label(egui::RichText::new("").strong());
                                                    for j in 0..self.signals {
                                                        ui.label(
                                                            egui::RichText::new(format!(
                                                                "y{}",
                                                                j + 1
                                                            ))
                                                            .strong(),
                                                        );
                                                    }
                                                    ui.end_row();

                                                    // Строки матрицы
                                                    for (i, row) in
                                                        result.transition_matrix.iter().enumerate()
                                                    {
                                                        ui.label(
                                                            egui::RichText::new(format!(
                                                                "x{}",
                                                                i + 1
                                                            ))
                                                            .strong(),
                                                        );
                                                        for val in row {
                                                            ui.label(format!("{val:.5}"));
                                                        }
                                                        ui.end_row();
                                                    }
                                                });
                                        });
                                }

                                ui.add_space(4.0);

                                if self.compact_view {
                                    Self::display_matrix_compact(
                                        ui,
                                        &result.joint_probs,
                                        "Матрица совместных вероятностей p(x_i,y_j):",
                                        &format!("scroll_joint_{i}"),
                                        &format!("grid_joint_{i}"),
                                    );
                                } else {
                                    ui.label(
                                        egui::RichText::new(
                                            "Матрица совместных вероятностей p(x_i,y_j):",
                                        )
                                        .strong(),
                                    );

                                    // Создаем прокручиваемую таблицу для матрицы совместных вероятностей
                                    egui::ScrollArea::horizontal()
                                        .id_salt(format!("scroll_joint_full_{i}"))
                                        .max_width(ui.available_width())
                                        .show(ui, |ui| {
                                            egui::Grid::new(format!("grid_joint_full_{i}"))
                                                .num_columns(self.signals + 1)
                                                .show(ui, |ui| {
                                                    // Заголовки столбцов
                                                    ui.label(egui::RichText::new("").strong());
                                                    for j in 0..self.signals {
                                                        ui.label(
                                                            egui::RichText::new(format!(
                                                                "y{}",
                                                                j + 1
                                                            ))
                                                            .strong(),
                                                        );
                                                    }
                                                    ui.end_row();

                                                    // Строки матрицы
                                                    for (i, row) in
                                                        result.joint_probs.iter().enumerate()
                                                    {
                                                        ui.label(
                                                            egui::RichText::new(format!(
                                                                "x{}",
                                                                i + 1
                                                            ))
                                                            .strong(),
                                                        );
                                                        for val in row {
                                                            ui.label(format!("{val:.5}"));
                                                        }
                                                        ui.end_row();
                                                    }
                                                });
                                        });
                                }

                                ui.add_space(4.0);
                                ui.label(
                                    egui::RichText::new(format!(
                                        "Условная энтропия H(X/Y): {:.5}",
                                        result.conditional_entropy
                                    ))
                                    .strong(),
                                );

                                if self.with_duration {
                                    ui.add_space(4.0);
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "Скорость передачи (с помехами): {}",
                                            format_rate(result.information_rate_with_noise)
                                        ))
                                        .strong(),
                                    );

                                    ui.label(
                                        egui::RichText::new(format!(
                                            "Пропускная способность (с помехами): {}",
                                            format_rate(result.capacity_with_noise)
                                        ))
                                        .strong(),
                                    );
                                }
                            }

                            ui.add_space(4.0);
                            ui.label(
                                egui::RichText::new(format!(
                                    "Количество информации I(X,Y): {:.5}",
                                    result.mutual_information
                                ))
                                .strong(),
                            );
                            ui.separator();
                        });
                    }
                });
        }
    }

    #[allow(clippy::too_many_lines)]
    fn render_labs4to6_ui(&mut self, ui: &mut egui::Ui) {
        let title = if self.labs4to6_use_hamming {
            "Лабораторные работы 4-6: Код Хемминга"
        } else {
            "Лабораторные работы 4-6: Систематический помехоустойчивый код"
        };
        ui.heading(egui::RichText::new(title).size(18.0));

        // Code type selector
        ui.horizontal(|ui| {
            ui.label("Тип кода:");
            ui.checkbox(&mut self.labs4to6_use_hamming, "Код Хемминга");
            if self.labs4to6_use_hamming {
                // Invalidate codes when switching
                if self.labs4to6_systematic_code.is_some() {
                    self.labs4to6_systematic_code = None;
                }
            } else if self.labs4to6_hamming_code.is_some() {
                self.labs4to6_hamming_code = None;
            }
        });

        // Parameters
        ui.horizontal(|ui| {
            ui.label("k (длина сообщения):");
            let response = ui.add(egui::DragValue::new(&mut self.labs4to6_k).range(1..=200));
            if response.changed() {
                if self.labs4to6_use_hamming {
                    let (n, p) = compute_hamming_n_from_k(self.labs4to6_k);
                    self.labs4to6_n = n;
                    self.labs4to6_p = p;
                    self.labs4to6_hamming_code = None;
                } else {
                    let (n, p) = compute_n_from_k(self.labs4to6_k);
                    self.labs4to6_n = n;
                    self.labs4to6_p = p;
                    self.labs4to6_systematic_code = None;
                }
            }
        });

        ui.horizontal(|ui| {
            ui.label(format!("n (длина кодового слова): {}", self.labs4to6_n));
            ui.label(format!(
                "p (число проверочных разрядов): {}",
                self.labs4to6_p
            ));
        });

        ui.horizontal(|ui| {
            ui.label("Количество экспериментов:");
            ui.add(egui::DragValue::new(&mut self.labs4to6_experiments).range(1..=100));
        });

        if !self.labs4to6_use_hamming {
            ui.horizontal(|ui| {
                ui.label("Вероятность ошибки:");
                ui.add(
                    egui::DragValue::new(&mut self.labs4to6_error_probability)
                        .range(0.0..=1.0)
                        .speed(0.01),
                );
            });

            ui.horizontal(|ui| {
                ui.checkbox(&mut self.compact_view, "Сокращенный вид матриц");
            });
        }

        // Auto-generate code if needed
        if !self.labs4to6_use_hamming && self.labs4to6_systematic_code.is_none() {
            self.labs4to6_systematic_code =
                Some(build_generator_matrix(self.labs4to6_k, self.labs4to6_n));
        }
        if self.labs4to6_use_hamming && self.labs4to6_hamming_code.is_none() {
            self.labs4to6_hamming_code = Some(HammingCode {
                k: self.labs4to6_k,
                n: self.labs4to6_n,
                p: self.labs4to6_p,
            });
        }

        ui.separator();

        // Run experiments button
        let can_run = if self.labs4to6_use_hamming {
            self.labs4to6_hamming_code.is_some()
        } else {
            self.labs4to6_systematic_code.is_some()
        };

        if can_run && ui.button("Запустить эксперименты").clicked() {
            self.labs4to6_results.clear();
            let mut rng = rand::rng();

            for _ in 0..self.labs4to6_experiments {
                if self.labs4to6_use_hamming {
                    if let Some(ref code) = self.labs4to6_hamming_code {
                        // Generate random k-bit message
                        let message: Vec<bool> =
                            (0..code.k).map(|_| rng.random_range(0..2) == 1).collect();

                        // Encode message using Hamming code
                        let codeword = encode_hamming(&message, code);

                        // Add parity bit for double error detection
                        let codeword_with_parity = add_parity_bit(&codeword);

                        // Generate random error multiplicity (0, 1, or 2)
                        let error_multiplicity = generate_error_multiplicity();

                        // Inject errors
                        let (received, error_positions) =
                            inject_errors(&codeword_with_parity, error_multiplicity);

                        // Compute syndrome and overall parity
                        let codeword_part = &received[..code.n];
                        let received_parity = received[code.n];
                        let (syndrome, computed_parity) =
                            compute_syndrome_hamming(codeword_part, code);
                        let overall_parity = received_parity == computed_parity;

                        // Decode and correct errors
                        let (corrected, error_info) = decode_hamming(&received, code, true);

                        self.labs4to6_results.push(Labs4To6ExperimentResult {
                            code_type: CodeType::Hamming,
                            message,
                            codeword,
                            codeword_with_parity: Some(codeword_with_parity),
                            error_multiplicity,
                            error_positions,
                            received,
                            syndrome,
                            overall_parity: Some(overall_parity),
                            corrected,
                            error_info: ErrorInfoType::Hamming(error_info),
                        });
                    }
                } else if let Some(ref code) = self.labs4to6_systematic_code {
                    // Generate random k-bit message
                    let message: Vec<bool> =
                        (0..code.k).map(|_| rng.random_range(0..2) == 1).collect();

                    // Encode message
                    let codeword = encode_message(&message, code);

                    // Inject error
                    let (received, error_position) =
                        inject_single_error(&codeword, self.labs4to6_error_probability);

                    // Compute syndrome
                    let syndrome = compute_syndrome(&code.parity_check, &received);

                    // Correct error
                    let (corrected, error_info) = correct_error(&code.parity_check, &received);

                    let error_positions = error_position.map(|p| vec![p + 1]).unwrap_or_default();

                    self.labs4to6_results.push(Labs4To6ExperimentResult {
                        code_type: CodeType::Systematic,
                        message,
                        codeword,
                        codeword_with_parity: None,
                        error_multiplicity: usize::from(error_position.is_some()),
                        error_positions,
                        received,
                        syndrome,
                        overall_parity: None,
                        corrected,
                        error_info: ErrorInfoType::Systematic(error_info),
                    });
                }
            }
        }

        ui.separator();

        // Display matrices (only for systematic code)
        #[allow(clippy::collapsible_if)]
        if !self.labs4to6_use_hamming {
            if let Some(ref code) = self.labs4to6_systematic_code {
                egui::ScrollArea::vertical()
                    .id_salt("labs4to6_matrices_scroll")
                    .auto_shrink([false; 2])
                    .max_height(ui.available_height() * 0.7)
                    .show(ui, |ui| {
                        if self.compact_view {
                            Self::display_binary_matrix_compact(
                                ui,
                                &code.generator,
                                &format!(
                                    "Производящая матрица P (k={}, n={}, p={}):",
                                    code.k, code.n, code.p
                                ),
                                "scroll_generator",
                                "grid_generator",
                            );
                        } else {
                            Self::display_binary_matrix_full(
                                ui,
                                &code.generator,
                                &format!(
                                    "Производящая матрица P (k={}, n={}, p={}):",
                                    code.k, code.n, code.p
                                ),
                                "scroll_generator",
                                "grid_generator",
                            );
                        }

                        ui.add_space(8.0);

                        if self.compact_view {
                            Self::display_binary_matrix_compact(
                                ui,
                                &code.parity_check,
                                &format!("Проверочная матрица H (p={}, n={}):", code.p, code.n),
                                "scroll_parity_check",
                                "grid_parity_check",
                            );
                        } else {
                            Self::display_binary_matrix_full(
                                ui,
                                &code.parity_check,
                                &format!("Проверочная матрица H (p={}, n={}):", code.p, code.n),
                                "scroll_parity_check",
                                "grid_parity_check",
                            );
                        }
                    });
            }
        }

        // Display results
        if !self.labs4to6_results.is_empty() {
            ui.separator();
            ui.label(
                egui::RichText::new("Результаты экспериментов:")
                    .strong()
                    .size(16.0),
            );

            egui::ScrollArea::vertical()
                .id_salt("labs4to6_experiments_scroll")
                .auto_shrink([false; 2])
                .max_height(ui.available_height())
                .show(ui, |ui| {
                    for (i, result) in self.labs4to6_results.iter().enumerate() {
                        ui.collapsing(format!("Эксперимент #{}", i + 1), |ui| {
                            Self::add_label(ui, "Сообщение:");
                            ui.label(Self::format_bits(&result.message));

                            Self::add_label(
                                ui,
                                if result.code_type == CodeType::Hamming {
                                    "Код Хемминга (без parity bit):"
                                } else {
                                    "Кодовое слово:"
                                },
                            );
                            ui.label(Self::format_bits(&result.codeword));

                            if let Some(ref codeword_with_parity) = result.codeword_with_parity {
                                Self::add_label(
                                    ui,
                                    "Модифицированный код Хемминга (с parity bit):",
                                );
                                ui.label(Self::format_bits(codeword_with_parity));
                            }

                            if result.code_type == CodeType::Hamming {
                                Self::add_label(
                                    ui,
                                    &format!("Кратность ошибки: {}", result.error_multiplicity),
                                );
                            }

                            if !result.error_positions.is_empty() {
                                Self::add_label(
                                    ui,
                                    &if result.code_type == CodeType::Hamming {
                                        format!(
                                            "Ошибки внесены в разряды: {:?}",
                                            result.error_positions
                                        )
                                    } else {
                                        format!(
                                            "Внесена ошибка в разряд {}",
                                            result.error_positions[0]
                                        )
                                    },
                                );
                            }

                            Self::add_label(ui, "Принятое сообщение:");
                            ui.label(Self::format_bits(&result.received));

                            Self::add_label(ui, "Синдром ошибки:");
                            ui.label(format!("({})", Self::format_bits(&result.syndrome)));

                            if let Some(overall_parity) = result.overall_parity {
                                Self::add_label(
                                    ui,
                                    &format!(
                                        "Общая проверка четности: {}",
                                        if overall_parity { "OK" } else { "Ошибка" }
                                    ),
                                );
                            }

                            Self::add_label(ui, "");
                            match &result.error_info {
                                ErrorInfoType::Systematic(ErrorInfo::NoError)
                                | ErrorInfoType::Hamming(HammingErrorInfo::NoError) => {
                                    ui.label(
                                        egui::RichText::new("Сообщение не содержит ошибок!")
                                            .strong(),
                                    );
                                }
                                ErrorInfoType::Systematic(ErrorInfo::SingleError(pos)) => {
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "Обнаружена ошибка в разряде {}",
                                            pos + 1
                                        ))
                                        .strong(),
                                    );
                                }
                                ErrorInfoType::Systematic(ErrorInfo::Uncorrectable) => {
                                    ui.label(
                                        egui::RichText::new(
                                            "Множественные ошибки или некорректируемая ошибка",
                                        )
                                        .strong(),
                                    );
                                }
                                ErrorInfoType::Hamming(HammingErrorInfo::SingleError(pos)) => {
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "Обнаружена однократная ошибка в разряде {pos}"
                                        ))
                                        .strong(),
                                    );
                                }
                                ErrorInfoType::Hamming(HammingErrorInfo::DoubleError) => {
                                    ui.label(
                                        egui::RichText::new(
                                            "Обнаружена двукратная ошибка (коррекция невозможна)",
                                        )
                                        .strong(),
                                    );
                                }
                            }

                            Self::add_label(ui, "Скорректированное кодовое слово:");
                            ui.label(Self::format_bits(&result.corrected));

                            ui.separator();
                        });
                    }
                });
        }
    }
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Info Entropy Simulation")
            .with_inner_size([1200.0, 800.0])
            .with_visible(true)
            .with_resizable(true)
            .with_decorations(true),
        ..Default::default()
    };
    eframe::run_native(
        "Info Entropy Simulation",
        native_options,
        Box::new(|_| Ok(Box::<InfoEntropyApp>::default())),
    )
}
