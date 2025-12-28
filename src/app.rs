use crate::state::{CodeConfig, Labs1To3State, Labs4To6ExperimentResult};
use crate::ui::{render_labs1to3_ui, render_labs4to6_ui};
use eframe::{App, Frame, egui};

#[derive(PartialEq, Eq, Clone, Copy)]
enum LabMode {
    Labs1To3,
    Labs4To6,
}

pub struct InfoEntropyApp {
    lab_mode: LabMode,
    labs1to3_state: Labs1To3State,
    code_config: CodeConfig,
    labs4to6_results: Vec<Labs4To6ExperimentResult>,
}

impl Default for InfoEntropyApp {
    fn default() -> Self {
        Self {
            lab_mode: LabMode::Labs1To3,
            labs1to3_state: Labs1To3State::default(),
            code_config: CodeConfig::new(60),
            labs4to6_results: vec![],
        }
    }
}

impl App for InfoEntropyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
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

            match self.lab_mode {
                LabMode::Labs1To3 => {
                    render_labs1to3_ui(ui, &mut self.labs1to3_state);
                }
                LabMode::Labs4To6 => {
                    render_labs4to6_ui(ui, &mut self.code_config, &mut self.labs4to6_results);
                }
            }
        });
    }
}
