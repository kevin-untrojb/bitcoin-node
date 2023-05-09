use std::{error::Error, fmt};

#[derive(Debug, PartialEq)]
pub enum NodoBitcoinError {
    _NoArgument,
    NoExisteArchivo,
    NoExisteClave,
    ConfigLock,
    NoSeEncontroURL,

    // serialize - deserialize
    NoSePuedeLeerLosBytes,
    NoSePuedeEscribirLosBytes,
}

impl Error for NodoBitcoinError {}

impl fmt::Display for NodoBitcoinError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NodoBitcoinError::_NoArgument => {
                write!(f, "ERROR: No se especificó el nombre del archivo.")
            }
            NodoBitcoinError::ConfigLock => {
                write!(f, "ERROR: Error al lockear el config.")
            }
            NodoBitcoinError::NoExisteArchivo => {
                write!(f, "ERROR: Error al leer archivo.")
            }
            NodoBitcoinError::NoExisteClave => {
                write!(f, "ERROR: No existe la clave.")
            }

            NodoBitcoinError::NoSeEncontroURL => {
                write!(f, "ERROR: No se pudo encontrar una URL válida.")

            NodoBitcoinError::NoSePuedeLeerLosBytes => {
                write!(
                    f,
                    "ERROR: No se puede leer correctamente la estructura en bytes."
                )
            }
            NodoBitcoinError::NoSePuedeEscribirLosBytes => {
                write!(
                    f,
                    "ERROR: No se puede escribir correctamente la estructura en bytes."
                )
            }
        }
    }
}
