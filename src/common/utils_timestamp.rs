use chrono::{DateTime, NaiveDate, TimeZone, Utc};

pub fn obtener_timestamp_dia(date: String) -> u32 {
    let fecha = NaiveDate::parse_from_str(&date, "%Y-%m-%d").unwrap();
    let fecha_inicio_dia = fecha.and_hms_opt(0, 0, 0).unwrap(); // Establece la hora a las 00:00:00
    fecha_inicio_dia.timestamp() as u32
}

pub fn _timestamp_to_datetime(timestamp: i64) -> DateTime<Utc> {
    match Utc.timestamp_opt(timestamp, 0) {
        chrono::offset::LocalResult::Single(date_time) => date_time,
        chrono::offset::LocalResult::None => Utc::now(),
        chrono::offset::LocalResult::Ambiguous(_, _) => Utc::now(),
    }
}
