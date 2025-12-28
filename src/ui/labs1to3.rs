use crate::formatting::{display_matrix_compact, display_matrix_full};
use crate::state::experiments::{ExperimentResult, Labs1To3State};
use crate::ui::widgets::format_probabilities_row;
use crate::utils::statistics::calculate_average;
use entropy_fx::{
    calc_entropy, calculate_average_duration, calculate_capacity_no_noise,
    calculate_capacity_with_noise, calculate_conditional_entropy,
    calculate_information_rate_no_noise, calculate_information_rate_with_noise,
    calculate_joint_probabilities, calculate_mutual_information, calculate_output_probabilities,
    format_rate, generate_error_probability_matrix, generate_probabilities,
    generate_symbol_durations, generate_transition_matrix, max_entropy,
};

/// Рендеринг UI для Labs 1-3
#[allow(clippy::too_many_lines)]
pub fn render_labs1to3_ui(ui: &mut egui::Ui, state: &mut Labs1To3State) {
    let title = if state.with_duration {
        "Обобщенные характеристики сигналов и каналов"
    } else {
        "Симуляция энтропии дискретных сообщений с учетом помех"
    };
    ui.heading(egui::RichText::new(title).size(18.0));

    render_experiment_parameters(ui, state);
    ui.separator();

    if ui.button("Запустить эксперименты").clicked() {
        run_experiments(state);
    }

    ui.separator();

    if !state.results.is_empty() {
        render_statistics(ui, state);
        ui.separator();
        render_experiment_results(ui, state);
    }
}

/// Рендеринг параметров эксперимента
fn render_experiment_parameters(ui: &mut egui::Ui, state: &mut Labs1To3State) {
    ui.horizontal(|ui| {
        ui.label("Количество экспериментов:");
        ui.add(egui::DragValue::new(&mut state.experiments).range(1..=100));
    });

    ui.horizontal(|ui| {
        ui.label("Количество дискретных сообщений (p_i):");
        ui.add(egui::DragValue::new(&mut state.signals).range(2..=128));
    });

    ui.horizontal(|ui| {
        ui.checkbox(&mut state.with_noise, "Учитывать помехи");
        ui.checkbox(
            &mut state.with_duration,
            "Режим расчета матрицы ошибок (по длительности и кол-ву символов)",
        );
        ui.checkbox(&mut state.compact_view, "Сокращенный вид матриц");
    });

    if state.with_noise && !state.with_duration {
        ui.horizontal(|ui| {
            ui.label("Минимальный порог достоверности:");
            ui.add(
                egui::DragValue::new(&mut state.min_threshold)
                    .range(0.0..=1.0)
                    .speed(0.01),
            );
        });
    }
}

/// Запуск экспериментов
fn run_experiments(state: &mut Labs1To3State) {
    state.results.clear();
    for _ in 0..state.experiments {
        let input_probs = generate_probabilities(state.signals);
        let input_entropy = calc_entropy(&input_probs);

        let symbol_durations = if state.with_duration {
            generate_symbol_durations(state.signals)
        } else {
            vec![]
        };
        let avg_duration = if state.with_duration {
            calculate_average_duration(&input_probs, &symbol_durations)
        } else {
            0.0
        };

        let information_rate_no_noise = if state.with_duration {
            calculate_information_rate_no_noise(input_entropy, avg_duration)
        } else {
            0.0
        };
        let capacity_no_noise = if state.with_duration {
            calculate_capacity_no_noise(state.signals, avg_duration)
        } else {
            0.0
        };

        if state.with_noise {
            let transition_matrix = if state.with_duration {
                generate_error_probability_matrix(state.signals)
            } else {
                generate_transition_matrix(state.signals, state.min_threshold)
            };
            let output_probs = calculate_output_probabilities(&input_probs, &transition_matrix);
            let joint_probs =
                calculate_joint_probabilities(&input_probs, &output_probs, &transition_matrix);
            let conditional_entropy =
                calculate_conditional_entropy(&joint_probs, &transition_matrix);
            let mutual_information =
                calculate_mutual_information(input_entropy, conditional_entropy);

            let information_rate_with_noise = if state.with_duration {
                calculate_information_rate_with_noise(
                    input_entropy,
                    conditional_entropy,
                    avg_duration,
                )
            } else {
                0.0
            };
            let capacity_with_noise = if state.with_duration {
                calculate_capacity_with_noise(state.signals, conditional_entropy, avg_duration)
            } else {
                0.0
            };

            state.results.push(ExperimentResult {
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
            state.results.push(ExperimentResult {
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

/// Рендеринг статистики
fn render_statistics(ui: &mut egui::Ui, state: &Labs1To3State) {
    let avg_mutual_info = calculate_average(&state.results, |r| r.mutual_information);
    let avg_input_entropy = calculate_average(&state.results, |r| r.input_entropy);

    ui.label(
        egui::RichText::new(format!(
            "Среднее количество информации: {avg_mutual_info:.5}"
        ))
        .strong(),
    );
    ui.label(
        egui::RichText::new(format!("Средняя энтропия на входе: {avg_input_entropy:.5}")).strong(),
    );
    ui.label(
        egui::RichText::new(format!(
            "Максимальная энтропия: {:.5}",
            max_entropy(state.signals)
        ))
        .strong(),
    );

    if state.with_noise {
        let avg_conditional_entropy = calculate_average(&state.results, |r| r.conditional_entropy);
        ui.label(
            egui::RichText::new(format!(
                "Средняя условная энтропия: {avg_conditional_entropy:.5}"
            ))
            .strong(),
        );
    }

    if state.with_duration {
        let avg_duration = calculate_average(&state.results, |r| r.avg_duration);
        ui.label(
            egui::RichText::new(format!(
                "Средняя длительность символа τ: {avg_duration:.5} мкс"
            ))
            .strong(),
        );

        let avg_rate_no_noise = calculate_average(&state.results, |r| r.information_rate_no_noise);
        ui.label(
            egui::RichText::new(format!(
                "Средняя скорость передачи (без помех): {}",
                format_rate(avg_rate_no_noise)
            ))
            .strong(),
        );

        let avg_capacity_no_noise = calculate_average(&state.results, |r| r.capacity_no_noise);
        ui.label(
            egui::RichText::new(format!(
                "Средняя пропускная способность (без помех): {}",
                format_rate(avg_capacity_no_noise)
            ))
            .strong(),
        );

        if state.with_noise {
            let avg_rate_with_noise =
                calculate_average(&state.results, |r| r.information_rate_with_noise);
            ui.label(
                egui::RichText::new(format!(
                    "Средняя скорость передачи (с помехами): {}",
                    format_rate(avg_rate_with_noise)
                ))
                .strong(),
            );

            let avg_capacity_with_noise =
                calculate_average(&state.results, |r| r.capacity_with_noise);
            ui.label(
                egui::RichText::new(format!(
                    "Средняя пропускная способность (с помехами): {}",
                    format_rate(avg_capacity_with_noise)
                ))
                .strong(),
            );
        }
    }
}

/// Рендеринг результатов экспериментов
fn render_experiment_results(ui: &mut egui::Ui, state: &Labs1To3State) {
    egui::ScrollArea::vertical()
        .id_salt("experiments_scroll")
        .auto_shrink([false; 2])
        .max_height(ui.available_height())
        .show(ui, |ui| {
            for (i, result) in state.results.iter().enumerate() {
                ui.collapsing(format!("Эксперимент #{}", i + 1), |ui| {
                    render_single_experiment(
                        ui,
                        result,
                        i,
                        state.signals,
                        state.compact_view,
                        state.with_duration,
                        state.with_noise,
                    );
                });
            }
        });
}

/// Рендеринг одного эксперимента
#[allow(clippy::too_many_lines)]
fn render_single_experiment(
    ui: &mut egui::Ui,
    result: &ExperimentResult,
    index: usize,
    signals: usize,
    compact_view: bool,
    with_duration: bool,
    with_noise: bool,
) {
    ui.label(egui::RichText::new("Вероятности сообщений на входе (p_i):").strong());
    for row in format_probabilities_row(&result.input_probs, "x", 4) {
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

    if with_duration {
        ui.add_space(4.0);
        ui.label(egui::RichText::new("Длительности символов (мкс):").strong());
        for row in format_probabilities_row(&result.symbol_durations, "T", 4) {
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

    if with_noise {
        ui.add_space(4.0);
        ui.label(egui::RichText::new("Вероятности на выходе (p_y):").strong());
        for row in format_probabilities_row(&result.output_probs, "y", 4) {
            ui.label(row);
        }

        ui.add_space(4.0);

        if compact_view {
            display_matrix_compact(
                ui,
                &result.transition_matrix,
                "Матрица переходов p(x_i/y_j):",
                &format!("scroll_transition_{index}"),
                &format!("grid_transition_{index}"),
            );
        } else {
            display_matrix_full(
                ui,
                &result.transition_matrix,
                "Матрица переходов p(x_i/y_j):",
                &format!("scroll_transition_full_{index}"),
                &format!("grid_transition_full_{index}"),
                signals,
            );
        }

        ui.add_space(4.0);

        if compact_view {
            display_matrix_compact(
                ui,
                &result.joint_probs,
                "Матрица совместных вероятностей p(x_i,y_j):",
                &format!("scroll_joint_{index}"),
                &format!("grid_joint_{index}"),
            );
        } else {
            display_matrix_full(
                ui,
                &result.joint_probs,
                "Матрица совместных вероятностей p(x_i,y_j):",
                &format!("scroll_joint_full_{index}"),
                &format!("grid_joint_full_{index}"),
                signals,
            );
        }

        ui.add_space(4.0);
        ui.label(
            egui::RichText::new(format!(
                "Условная энтропия H(X/Y): {:.5}",
                result.conditional_entropy
            ))
            .strong(),
        );

        if with_duration {
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
}
