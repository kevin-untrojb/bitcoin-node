use std::{fs::{File, OpenOptions}, io::Write};

use crate::{config, errores::NodoBitcoinError};

pub fn escribir_archivo(datos: &[u8]) -> Result<(), NodoBitcoinError> {
    let path = config::get_valor("NOMBRE_ARCHIVO".to_string())?;
    let mut archivo = match OpenOptions::new().create(true).append(true).open(path) {
        Ok(archivo) => archivo,
        Err(_) => return Err(NodoBitcoinError::NoExisteArchivo),
    };

    // Escribe los bytes en el archivo
    match archivo.write_all(datos) {
        Ok(_) => return Ok(()),
        Err(_) => return Err(NodoBitcoinError::NoSePuedeEscribirLosBytes),
    };
}
