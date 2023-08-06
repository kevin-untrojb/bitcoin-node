use crate::config;

const DEFAULT_TOTAL_REINTEGROS: usize = 5;

pub fn total_reintentos() -> usize {
    let total_reintentos_config = config::get_valor("REINTENTOS_DESCARGA_BLOQUES".to_string());
    if let Ok(valor_string) = total_reintentos_config {
        if let Ok(value) = valor_string.parse::<usize>() {
            return value;
        };
    };
    DEFAULT_TOTAL_REINTEGROS
}
