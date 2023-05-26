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
    _leer_header(1)
}

pub fn _leer_header(offset: u64) -> Result<Vec<u8>, NodoBitcoinError> {
    let path = config::get_valor("NOMBRE_ARCHIVO".to_string())?;
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(_) => return Err(NodoBitcoinError::NoExisteArchivo),
    };
    let file_size = match file.seek(SeekFrom::End(0)) {
        Ok(file_size) => file_size,
        Err(_) => return Err(NodoBitcoinError::NoSePuedeLeerLosBytes),
    };
    let start_position = if file_size >= 80 {
        file_size - (80 * offset)
    } else {
        0
    };

    let new_position = file.seek(SeekFrom::Start(start_position));
    if new_position.is_err() {
        return Err(NodoBitcoinError::NoSePuedeLeerLosBytes);
    }
    let mut buffer = vec![0; 80];
    let _ = match file.read_exact(&mut buffer) {
        Ok(readed) => readed,
        Err(_) => return Err(NodoBitcoinError::NoSePuedeLeerLosBytes),
    };

    Ok(buffer)
}
