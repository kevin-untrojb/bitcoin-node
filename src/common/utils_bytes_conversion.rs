use chrono::NaiveDate;

use crate::errores::NodoBitcoinError;

pub fn bytes_to_string(bytes: &[u8]) -> Result<String, NodoBitcoinError> {
    if let Ok(string) = String::from_utf8(bytes.to_vec()) {
        return Ok(string);
    }
    Err(NodoBitcoinError::NoSePuedeLeerLosBytes)
}

pub fn obtener_timestamp_dia(date: String) -> u32 {
    let fecha = NaiveDate::parse_from_str(&date, "%Y-%m-%d").unwrap();
    let fecha_inicio_dia = fecha.and_hms_opt(17,18,31).unwrap(); // Establece la hora a las 00:00:00

    let timestamp = fecha_inicio_dia.timestamp() as u32;
    timestamp
}