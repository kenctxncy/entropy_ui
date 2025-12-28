use entropy_fx::coding::cyclic::CyclicCode;
use entropy_fx::coding::hamming::HammingCode;
use entropy_fx::coding::systematic::SystematicCode;

/// Тип выбранного кода
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum SelectedCodeType {
    Systematic,
    Hamming,
    Cyclic,
}

/// Конфигурация кода для Labs 4-6
pub struct CodeConfig {
    pub code_type: SelectedCodeType,
    pub k: usize,
    pub n: usize,
    pub p: usize,
    pub experiments: usize,
    pub error_probability: f64,
    pub compact_view: bool,
    pub systematic_code: Option<SystematicCode>,
    pub hamming_code: Option<HammingCode>,
    pub cyclic_code: Option<CyclicCode>,
}

impl CodeConfig {
    pub fn new(k: usize) -> Self {
        let (n, p) = entropy_fx::coding::common::compute_n_from_k(k);
        Self {
            code_type: SelectedCodeType::Systematic,
            k,
            n,
            p,
            experiments: 6,
            error_probability: 0.5,
            compact_view: false,
            systematic_code: None,
            hamming_code: None,
            cyclic_code: None,
        }
    }

    /// Установить тип кода и очистить другие типы
    pub fn set_code_type(&mut self, code_type: SelectedCodeType) {
        self.code_type = code_type;
        match code_type {
            SelectedCodeType::Systematic => {
                self.hamming_code = None;
                self.cyclic_code = None;
            }
            SelectedCodeType::Hamming => {
                self.systematic_code = None;
                self.cyclic_code = None;
            }
            SelectedCodeType::Cyclic => {
                self.systematic_code = None;
                self.hamming_code = None;
            }
        }
    }

    /// Проверить, можно ли запустить эксперименты
    pub const fn can_run_experiments(&self) -> bool {
        match self.code_type {
            SelectedCodeType::Systematic => self.systematic_code.is_some(),
            SelectedCodeType::Hamming => self.hamming_code.is_some(),
            SelectedCodeType::Cyclic => self.cyclic_code.is_some(),
        }
    }

    /// Обновить n и p при изменении k
    pub fn update_n_and_p(&mut self) {
        let (n, p) = match self.code_type {
            SelectedCodeType::Hamming => {
                entropy_fx::coding::hamming::compute_hamming_n_from_k(self.k)
            }
            SelectedCodeType::Cyclic | SelectedCodeType::Systematic => {
                entropy_fx::coding::common::compute_n_from_k(self.k)
            }
        };
        self.n = n;
        self.p = p;
        // Инвалидировать коды при изменении параметров
        match self.code_type {
            SelectedCodeType::Systematic => self.systematic_code = None,
            SelectedCodeType::Hamming => self.hamming_code = None,
            SelectedCodeType::Cyclic => self.cyclic_code = None,
        }
    }

    /// Инициализировать код, если он еще не создан
    pub fn ensure_code_initialized(&mut self) {
        match self.code_type {
            SelectedCodeType::Systematic if self.systematic_code.is_none() => {
                self.systematic_code = Some(
                    entropy_fx::coding::systematic::build_generator_matrix(self.k, self.n),
                );
            }
            SelectedCodeType::Hamming if self.hamming_code.is_none() => {
                self.hamming_code = Some(HammingCode {
                    k: self.k,
                    n: self.n,
                    p: self.p,
                });
            }
            SelectedCodeType::Cyclic if self.cyclic_code.is_none() => {
                self.cyclic_code = Some(entropy_fx::coding::cyclic::create_cyclic_code(self.k));
            }
            _ => {}
        }
    }
}

/// Результат эксперимента для Labs 4-6
#[derive(Clone)]
pub struct Labs4To6ExperimentResult {
    pub code_type: CodeType,
    pub message: Vec<bool>,
    pub codeword: Vec<bool>,
    pub codeword_with_parity: Option<Vec<bool>>,
    pub error_multiplicity: usize,
    pub error_positions: Vec<usize>,
    pub received: Vec<bool>,
    pub syndrome: Vec<bool>,
    pub overall_parity: Option<bool>,
    pub corrected: Vec<bool>,
    pub error_info: ErrorInfoType,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum CodeType {
    Systematic,
    Hamming,
    Cyclic,
}

#[derive(Clone)]
pub enum ErrorInfoType {
    Systematic(entropy_fx::coding::systematic::ErrorInfo),
    Hamming(entropy_fx::coding::hamming::HammingErrorInfo),
    Cyclic(entropy_fx::coding::cyclic::CyclicErrorInfo),
}
