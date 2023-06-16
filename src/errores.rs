use std::{error::Error, fmt};

#[derive(Debug, PartialEq)]
pub enum NodoBitcoinError {
    _NoArgument,
    NoExisteArchivo,
    NoExisteClave,
    ConfigLock,
    NoSePuedeLeerValorDeArchivoConfig,

    // conexion
    _NoSePudoConectar,
    MagicNumberIncorrecto,
    ErrorEnHandshake,
    NoSeEncuentraConexionLibre,

    // serialize - deserialize
    NoSePuedeLeerLosBytes,
    NoSePuedeEscribirLosBytes,
    NoSePuedeLeerLosBytesHeaderVersionMessage,
    NoSePuedeLeerLosBytesVersionMessage,
    NoSePuedeLeerLosBytesVerackMessage,
    _ValorFueraDeRango,

    // merkle_tree
    _NoChildren,
    _NoSePuedeArmarElArbol,

    // decode base58 error
    DecodeError,

    // wallet
    InvalidAccount,
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
            NodoBitcoinError::NoSePuedeLeerValorDeArchivoConfig => {
                write!(f, "ERROR: No se puede leer valor desde el archivo config")
            }
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
            NodoBitcoinError::_NoSePudoConectar => {
                write!(f, "ERROR: No se pudo conectar al servidor.")
            }
            NodoBitcoinError::_ValorFueraDeRango => {
                write!(
                    f,
                    "ERROR: No se puede parsear el valor ya que está fuera de rango."
                )
            }

            NodoBitcoinError::MagicNumberIncorrecto => {
                write!(f, "ERROR: El magic number recibido es incorrecto.")
            }
            NodoBitcoinError::ErrorEnHandshake => {
                write!(f, "ERROR: Hubo un error en el handshake.")
            }
            NodoBitcoinError::_NoChildren => {
                write!(f, "ERROR: No hay TXs para crear el Merkle Tree.")
            }
            NodoBitcoinError::NoSeEncuentraConexionLibre => {
                write!(f, "ERROR: No se encuentra conexion disponible.")
            }
            NodoBitcoinError::NoSePuedeLeerLosBytesHeaderVersionMessage => {
                write!(
                    f,
                    "ERROR: No se puede leer correctamente el header del version message."
                )
            }
            NodoBitcoinError::NoSePuedeLeerLosBytesVersionMessage => {
                write!(
                    f,
                    "ERROR: No se puede leer correctamente el version message."
                )
            }
            NodoBitcoinError::NoSePuedeLeerLosBytesVerackMessage => {
                write!(
                    f,
                    "ERROR: No se puede leer correctamente el verack message."
                )
            }
            NodoBitcoinError::_NoSePuedeArmarElArbol => {
                write!(f, "ERROR: No se puede crear el merkle tree del bloque.")
            }
            NodoBitcoinError::DecodeError => {
                write!(f, "ERROR: No se pudo decodificar.")
            }
            NodoBitcoinError::InvalidAccount => {
                write!(f, "ERROR: La TxOut no pertenece a la cuenta.")
            }
        }
    }
}
