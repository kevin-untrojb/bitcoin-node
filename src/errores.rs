use std::{error::Error, fmt};

#[derive(Debug, PartialEq)]
pub enum NodoBitcoinError {
    _NoArgument,
    NoExisteArchivo,
    NoExisteClave,
    ConfigLock,

    // conexion
    NoSePudoConectar,
    MagicNumberIncorrecto,
    ErrorEnHandshake,

    // serialize - deserialize
    NoSePuedeLeerLosBytes,
    NoSePuedeLeerLosBytes1,
    NoSePuedeLeerLosBytes2,
    NoSePuedeLeerLosBytes3,
    NoSePuedeEscribirLosBytes,

    // merkle_tree
    NoChildren,
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
            NodoBitcoinError::NoSePuedeLeerLosBytes => {
                write!(
                    f,
                    "ERROR: No se puede leer correctamente la estructura en bytes."
                )
            }
            NodoBitcoinError::NoSePuedeLeerLosBytes1 => {
                write!(
                    f,
                    "ERROR: No se puede leer correctamente la estructura en bytes1."
                )
            }
            NodoBitcoinError::NoSePuedeLeerLosBytes2 => {
                write!(
                    f,
                    "ERROR: No se puede leer correctamente la estructura en bytes2."
                )
            }
            NodoBitcoinError::NoSePuedeLeerLosBytes3 => {
                write!(
                    f,
                    "ERROR: No se puede leer correctamente la estructura en bytes3."
                )
            }
            NodoBitcoinError::NoSePuedeEscribirLosBytes => {
                write!(
                    f,
                    "ERROR: No se puede escribir correctamente la estructura en bytes."
                )
            }
            NodoBitcoinError::NoSePudoConectar => {
                write!(f, "ERROR: No se pudo conectar al servidor.")
            }

            NodoBitcoinError::MagicNumberIncorrecto => {
                write!(f, "ERROR: El magic number recibido es incorrecto.")
            }
            NodoBitcoinError::ErrorEnHandshake => {
                write!(f, "ERROR: Hubo un error en el handshake.")
            }
            NodoBitcoinError::NoChildren => {
                write!(f, "ERROR: No hay TXs para crear el Merkle Tree.")
            }
        }
    }
}
