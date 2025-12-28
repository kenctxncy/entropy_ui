use crate::formatting::{display_binary_matrix_compact, display_binary_matrix_full, format_bits};
use crate::state::code_config::{
    CodeConfig, CodeType, ErrorInfoType, Labs4To6ExperimentResult, SelectedCodeType,
};
use crate::ui::widgets::add_label;
use entropy_fx::coding::cyclic::{
    compute_syndrome_cyclic, decode_cyclic, encode_cyclic, inject_single_error_cyclic,
    polynomial_to_bits,
};
use entropy_fx::coding::hamming::{
    add_parity_bit, compute_syndrome_hamming, decode_hamming, encode_hamming,
    generate_error_multiplicity, inject_errors,
};
use entropy_fx::coding::systematic::{
    compute_syndrome, correct_error, encode_message, inject_single_error,
};
use rand::Rng;

/// Рендеринг UI для Labs 4-6
#[allow(clippy::too_many_lines)]
pub fn render_labs4to6_ui(
    ui: &mut egui::Ui,
    config: &mut CodeConfig,
    results: &mut Vec<Labs4To6ExperimentResult>,
) {
    let title = match config.code_type {
        SelectedCodeType::Hamming => "Лабораторные работы 4-6: Код Хемминга",
        SelectedCodeType::Cyclic => "Лабораторные работы 4-6: Циклический код",
        SelectedCodeType::Systematic => {
            "Лабораторные работы 4-6: Систематический помехоустойчивый код"
        }
    };
    ui.heading(egui::RichText::new(title).size(18.0));

    render_code_type_selector(ui, config);
    render_code_parameters(ui, config);
    config.ensure_code_initialized();

    ui.separator();

    if config.can_run_experiments() && ui.button("Запустить эксперименты").clicked()
    {
        *results = run_experiments(config);
    }

    ui.separator();

    render_code_matrices(ui, config);

    if !results.is_empty() {
        ui.separator();
        render_experiment_results(ui, results, config);
    }
}

/// Рендеринг селектора типа кода
fn render_code_type_selector(ui: &mut egui::Ui, config: &mut CodeConfig) {
    ui.horizontal(|ui| {
        ui.label("Тип кода:");

        let mut systematic_selected = config.code_type == SelectedCodeType::Systematic;
        let mut hamming_selected = config.code_type == SelectedCodeType::Hamming;
        let mut cyclic_selected = config.code_type == SelectedCodeType::Cyclic;

        let systematic_clicked = ui
            .checkbox(&mut systematic_selected, "Систематический код")
            .clicked();
        let hamming_clicked = ui.checkbox(&mut hamming_selected, "Код Хемминга").clicked();
        let cyclic_clicked = ui
            .checkbox(&mut cyclic_selected, "Циклический код")
            .clicked();

        // Обработка кликов - только один тип может быть выбран
        if systematic_clicked && systematic_selected {
            config.set_code_type(SelectedCodeType::Systematic);
        } else if hamming_clicked && hamming_selected {
            config.set_code_type(SelectedCodeType::Hamming);
        } else if cyclic_clicked && cyclic_selected {
            config.set_code_type(SelectedCodeType::Cyclic);
        } else if !systematic_selected && !hamming_selected && !cyclic_selected {
            // Если все сняты, выбираем систематический код по умолчанию
            config.set_code_type(SelectedCodeType::Systematic);
        }
    });
}

/// Рендеринг параметров кода
fn render_code_parameters(ui: &mut egui::Ui, config: &mut CodeConfig) {
    ui.horizontal(|ui| {
        ui.label("k (длина сообщения):");
        let response = ui.add(egui::DragValue::new(&mut config.k).range(1..=200));
        if response.changed() {
            config.update_n_and_p();
        }
    });

    ui.horizontal(|ui| {
        ui.label(format!("n (длина кодового слова): {}", config.n));
        ui.label(format!("p (число проверочных разрядов): {}", config.p));
    });

    ui.horizontal(|ui| {
        ui.label("Количество экспериментов:");
        ui.add(egui::DragValue::new(&mut config.experiments).range(1..=100));
    });

    if config.code_type != SelectedCodeType::Hamming {
        ui.horizontal(|ui| {
            ui.label("Вероятность ошибки:");
            ui.add(
                egui::DragValue::new(&mut config.error_probability)
                    .range(0.0..=1.0)
                    .speed(0.01),
            );
        });

        if config.code_type == SelectedCodeType::Systematic {
            ui.horizontal(|ui| {
                ui.checkbox(&mut config.compact_view, "Сокращенный вид матриц");
            });
        }
    }
}

/// Рендеринг матриц кода
fn render_code_matrices(ui: &mut egui::Ui, config: &CodeConfig) {
    if config.code_type == SelectedCodeType::Systematic
        && let Some(ref code) = config.systematic_code
    {
        egui::ScrollArea::vertical()
            .id_salt("labs4to6_matrices_scroll")
            .auto_shrink([false; 2])
            .max_height(ui.available_height() * 0.7)
            .show(ui, |ui| {
                if config.compact_view {
                    display_binary_matrix_compact(
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
                    display_binary_matrix_full(
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

                if config.compact_view {
                    display_binary_matrix_compact(
                        ui,
                        &code.parity_check,
                        &format!("Проверочная матрица H (p={}, n={}):", code.p, code.n),
                        "scroll_parity_check",
                        "grid_parity_check",
                    );
                } else {
                    display_binary_matrix_full(
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

/// Рендеринг результатов экспериментов
#[allow(clippy::too_many_lines)]
fn render_experiment_results(
    ui: &mut egui::Ui,
    results: &[Labs4To6ExperimentResult],
    config: &CodeConfig,
) {
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
            for (i, result) in results.iter().enumerate() {
                ui.collapsing(format!("Эксперимент #{}", i + 1), |ui| {
                    add_label(ui, "Сообщение:");
                    ui.label(format_bits(&result.message));

                    add_label(
                        ui,
                        match result.code_type {
                            CodeType::Hamming => "Код Хемминга (без parity bit):",
                            CodeType::Cyclic => "Циклический код:",
                            CodeType::Systematic => "Кодовое слово:",
                        },
                    );
                    ui.label(format_bits(&result.codeword));

                    if let Some(ref codeword_with_parity) = result.codeword_with_parity {
                        add_label(ui, "Модифицированный код Хемминга (с parity bit):");
                        ui.label(format_bits(codeword_with_parity));
                    }

                    if result.code_type == CodeType::Hamming {
                        add_label(
                            ui,
                            &format!("Кратность ошибки: {}", result.error_multiplicity),
                        );
                    }

                    if !result.error_positions.is_empty() {
                        add_label(
                            ui,
                            &match result.code_type {
                                CodeType::Hamming => format!(
                                    "Ошибки внесены в разряды: {:?}",
                                    result.error_positions
                                ),
                                _ => {
                                    format!("Внесена ошибка в разряд {}", result.error_positions[0])
                                }
                            },
                        );
                    }

                    add_label(ui, "Принятое сообщение:");
                    ui.label(format_bits(&result.received));

                    add_label(ui, "Синдром ошибки:");
                    ui.label(format!("({})", format_bits(&result.syndrome)));

                    // Вывод таблицы синдромов для циклического кода
                    if result.code_type == CodeType::Cyclic
                        && let Some(ref code) = config.cyclic_code
                    {
                        add_label(ui, "Таблица соответствия позиции ошибки к синдрому:");

                        // Оценка ширины одной записи (примерно 150-200 пикселей)
                        let estimated_entry_width = 180.0;
                        let available_width = ui.available_width();
                        let spacing_between_columns = 20.0;

                        // Вычисляем оптимальное количество столбцов
                        // available_width всегда неотрицателен и ограничен разумными значениями,
                        // поэтому приведение безопасно (усечение не произойдет для реальных размеров экрана)
                        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
                        let max_columns = (available_width
                            / (estimated_entry_width + spacing_between_columns))
                            .floor()
                            .max(1.0) as usize;
                        let num_columns = max_columns.min(code.syndrome_table.len()).max(1);

                        // Вычисляем количество записей в каждом столбце
                        let entries_per_column = code.syndrome_table.len().div_ceil(num_columns);

                        egui::ScrollArea::vertical()
                            .max_height(300.0)
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    for col in 0..num_columns {
                                        ui.vertical(|ui| {
                                            let start_idx = col * entries_per_column;
                                            let end_idx = ((col + 1) * entries_per_column)
                                                .min(code.syndrome_table.len());

                                            for i in start_idx..end_idx {
                                                if let Some((syndrome_poly, error_pos)) =
                                                    code.syndrome_table.get(i)
                                                {
                                                    let syndrome_bits =
                                                        polynomial_to_bits(syndrome_poly, code.p);
                                                    ui.horizontal(|ui| {
                                                        ui.label(format!("{}:", error_pos + 1));
                                                        ui.label(format!(
                                                            "({})",
                                                            format_bits(&syndrome_bits)
                                                        ));
                                                    });
                                                }
                                            }
                                        });

                                        if col < num_columns - 1 {
                                            ui.add_space(spacing_between_columns);
                                        }
                                    }
                                });
                            });
                    }

                    if let Some(overall_parity) = result.overall_parity {
                        add_label(
                            ui,
                            &format!(
                                "Общая проверка четности: {}",
                                if overall_parity { "OK" } else { "Ошибка" }
                            ),
                        );
                    }

                    add_label(ui, "");
                    match &result.error_info {
                        ErrorInfoType::Systematic(
                            entropy_fx::coding::systematic::ErrorInfo::NoError,
                        )
                        | ErrorInfoType::Hamming(
                            entropy_fx::coding::hamming::HammingErrorInfo::NoError,
                        )
                        | ErrorInfoType::Cyclic(
                            entropy_fx::coding::cyclic::CyclicErrorInfo::NoError,
                        ) => {
                            ui.label(egui::RichText::new("Сообщение не содержит ошибок!").strong());
                        }
                        ErrorInfoType::Systematic(
                            entropy_fx::coding::systematic::ErrorInfo::SingleError(pos),
                        ) => {
                            ui.label(
                                egui::RichText::new(format!(
                                    "Обнаружена ошибка в разряде {}",
                                    pos + 1
                                ))
                                .strong(),
                            );
                        }
                        ErrorInfoType::Systematic(
                            entropy_fx::coding::systematic::ErrorInfo::Uncorrectable,
                        )
                        | ErrorInfoType::Cyclic(
                            entropy_fx::coding::cyclic::CyclicErrorInfo::Uncorrectable,
                        ) => {
                            ui.label(
                                egui::RichText::new(
                                    "Множественные ошибки или некорректируемая ошибка",
                                )
                                .strong(),
                            );
                        }
                        ErrorInfoType::Hamming(
                            entropy_fx::coding::hamming::HammingErrorInfo::SingleError(pos),
                        ) => {
                            ui.label(
                                egui::RichText::new(format!(
                                    "Обнаружена однократная ошибка в разряде {pos}"
                                ))
                                .strong(),
                            );
                        }
                        ErrorInfoType::Hamming(
                            entropy_fx::coding::hamming::HammingErrorInfo::DoubleError,
                        ) => {
                            ui.label(
                                egui::RichText::new(
                                    "Обнаружена двукратная ошибка (коррекция невозможна)",
                                )
                                .strong(),
                            );
                        }
                        ErrorInfoType::Cyclic(
                            entropy_fx::coding::cyclic::CyclicErrorInfo::SingleError(pos),
                        ) => {
                            ui.label(
                                egui::RichText::new(format!("Обнаружена ошибка в разряде {pos}"))
                                    .strong(),
                            );
                        }
                    }

                    add_label(ui, "Скорректированное кодовое слово:");
                    ui.label(format_bits(&result.corrected));

                    ui.separator();
                });
            }
        });
}

/// Запуск экспериментов
#[allow(clippy::too_many_lines)]
fn run_experiments(config: &CodeConfig) -> Vec<Labs4To6ExperimentResult> {
    let mut results = Vec::new();
    let mut rng = rand::rng();

    for _ in 0..config.experiments {
        match config.code_type {
            SelectedCodeType::Hamming => {
                if let Some(ref code) = config.hamming_code {
                    let message: Vec<bool> =
                        (0..code.k).map(|_| rng.random_range(0..2) == 1).collect();

                    let codeword = encode_hamming(&message, code);
                    let codeword_with_parity = add_parity_bit(&codeword);
                    let error_multiplicity = generate_error_multiplicity();
                    let (received, error_positions) =
                        inject_errors(&codeword_with_parity, error_multiplicity);

                    let codeword_part = &received[..code.n];
                    let received_parity = received[code.n];
                    let (syndrome, computed_parity) = compute_syndrome_hamming(codeword_part, code);
                    let overall_parity = received_parity == computed_parity;

                    let (corrected, error_info) = decode_hamming(&received, code, true);

                    results.push(Labs4To6ExperimentResult {
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
            }
            SelectedCodeType::Systematic => {
                if let Some(ref code) = config.systematic_code {
                    let message: Vec<bool> =
                        (0..code.k).map(|_| rng.random_range(0..2) == 1).collect();

                    let codeword = encode_message(&message, code);
                    let (received, error_position) =
                        inject_single_error(&codeword, config.error_probability);
                    let syndrome = compute_syndrome(&code.parity_check, &received);
                    let (corrected, error_info) = correct_error(&code.parity_check, &received);

                    let error_positions = error_position.map(|p| vec![p + 1]).unwrap_or_default();

                    results.push(Labs4To6ExperimentResult {
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
            SelectedCodeType::Cyclic => {
                if let Some(ref code) = config.cyclic_code {
                    let message: Vec<bool> =
                        (0..code.k).map(|_| rng.random_range(0..2) == 1).collect();

                    let codeword = encode_cyclic(&message, code);
                    let (received, error_position) =
                        inject_single_error_cyclic(&codeword, config.error_probability);
                    let syndrome_poly = compute_syndrome_cyclic(&received, code);
                    let syndrome = polynomial_to_bits(&syndrome_poly, code.p);
                    let (corrected, error_info) = decode_cyclic(&received, code);

                    let error_positions = error_position.map(|p| vec![p + 1]).unwrap_or_default();

                    results.push(Labs4To6ExperimentResult {
                        code_type: CodeType::Cyclic,
                        message,
                        codeword,
                        codeword_with_parity: None,
                        error_multiplicity: usize::from(error_position.is_some()),
                        error_positions,
                        received,
                        syndrome,
                        overall_parity: None,
                        corrected,
                        error_info: ErrorInfoType::Cyclic(error_info),
                    });
                }
            }
        }
    }

    results
}
