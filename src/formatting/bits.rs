/// Форматирование битового вектора как строки
#[must_use]
pub fn format_bits(bits: &[bool]) -> String {
    bits.iter().map(|&b| if b { '1' } else { '0' }).collect()
}
