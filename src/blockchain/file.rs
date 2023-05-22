use std::{fs::OpenOptions, io::Write};

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
