use std::{
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
};

use crate::{config, errores::NodoBitcoinError};

pub fn escribir_archivo(datos: &[u8]) -> Result<(), NodoBitcoinError> {
    let path = config::get_valor("NOMBRE_ARCHIVO".to_string())?;
    let mut archivo = match OpenOptions::new().create(true).append(true).open(path) {
        Ok(archivo) => archivo,
        Err(_) => return Err(NodoBitcoinError::NoExisteArchivo),
    };

    // Escribe los bytes en el archivo
    archivo
        .write_all(datos)
        .map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;
    Ok(())
}

pub fn _leer_ultimo_header() -> Result<Vec<u8>, NodoBitcoinError> {
    //_leer_header(1)
    let cantidad_headers = _header_count()?;
    _leer_header_desde_archivo(cantidad_headers - 1)
}

fn _get_file_header_size() -> Result<u64, NodoBitcoinError> {
    let path = config::get_valor("NOMBRE_ARCHIVO".to_string())?;
    let file = File::open(path);
    if file.is_err() {
        return Err(NodoBitcoinError::NoExisteArchivo);
    }
    let file = file.unwrap();
    let metadata = file.metadata();
    if metadata.is_err() {
        return Err(NodoBitcoinError::NoSePuedeLeerLosBytes);
    }
    Ok(metadata.unwrap().len())
}

pub fn _header_count() -> Result<u64, NodoBitcoinError> {
    let file_size = _get_file_header_size()?;
    Ok(file_size / 80)
}

pub fn _leer_header_desde_archivo(index: u64) -> Result<Vec<u8>, NodoBitcoinError> {
    //    _leer_header(total_headers)
    let path = config::get_valor("NOMBRE_ARCHIVO".to_string())?;
    let offset = index * 80;
    leer_bytes(path, offset, 80)
}

pub fn _leer_todos_headers() -> Result<Vec<u8>, NodoBitcoinError> {
    let path = config::get_valor("NOMBRE_ARCHIVO".to_string())?;
    let file_size = _get_file_header_size()?;
    leer_bytes(path, 0, file_size)
}

pub fn _leer_primer_header() -> Result<Vec<u8>, NodoBitcoinError> {
    _leer_header_desde_archivo(0)
}

pub fn _leer_headers(ix: u64) -> Result<Vec<u8>, NodoBitcoinError> {
    // devuelve de a 2000 headers
    let offset: u64 = ix * 2000;
    let length: u64 = 2000 * 80;
    let path = config::get_valor("NOMBRE_ARCHIVO".to_string())?;
    leer_bytes(path, offset, length)
}

fn leer_bytes(path: String, offset: u64, length: u64) -> Result<Vec<u8>, NodoBitcoinError> {
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(_) => return Err(NodoBitcoinError::NoExisteArchivo),
    };
    let new_position = file.seek(SeekFrom::Start(offset));
    if new_position.is_err() {
        return Err(NodoBitcoinError::NoSePuedeLeerLosBytes);
    }
    let mut buffer = vec![0; length as usize];
    let _ = match file.read_exact(&mut buffer) {
        Ok(readed) => readed,
        Err(_) => return Err(NodoBitcoinError::NoSePuedeLeerLosBytes),
    };

    Ok(buffer)
}
