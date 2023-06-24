use std::{error::Error, fmt};

#[derive(Debug, PartialEq)]
pub enum NodoBitcoinError {
    _NoArgument,
    NoExisteArchivo,
    NoExisteClave,
    ConfigLock,
    NoSePuedeLeerValorDeArchivoConfig,

    // conexion
    NoSePudoConectar,
    MagicNumberIncorrecto,
    ErrorEnHandshake,
    NoSeEncuentraConexionLibre,

    // serialize - deserialize
    NoSePuedeLeerLosBytes,
    NoSePuedeEscribirLosBytes,
    NoSePuedeLeerLosBytesHeaderVersionMessage,
    NoSePuedeLeerLosBytesVersionMessage,
    NoSePuedeLeerLosBytesVerackMessage,
    ValorFueraDeRango,

    // merkle_tree
    NoChildren,
    NoSePuedeArmarElArbol,

    // decode base58 error
    DecodeError,

    // transaccion
    NoHaySuficientesUtxos,

    // mensajes
    InvalidAccount,
    NoEsTransaccion,

    // wallet
    NoHayCuentaSeleccionada,
    CuentaNoEncontrada,
    NoSePuedeEnviarTransaccion,
    ErrorAlActualizarUTXOS,
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
            NodoBitcoinError::NoSePudoConectar => {
                write!(f, "ERROR: No se pudo conectar al servidor.")
            }
            NodoBitcoinError::ValorFueraDeRango => {
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
            NodoBitcoinError::NoChildren => {
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
            NodoBitcoinError::NoSePuedeArmarElArbol => {
                write!(f, "ERROR: No se puede crear el merkle tree del bloque.")
            }
            NodoBitcoinError::DecodeError => {
                write!(f, "ERROR: No se pudo decodificar.")
            }
            NodoBitcoinError::InvalidAccount => {
                write!(f, "ERROR: La TxOut no pertenece a la cuenta.")
            }
            NodoBitcoinError::NoEsTransaccion => {
                write!(f, "ERROR: El mensaje no contiene una transacción.")
            }
            NodoBitcoinError::NoHaySuficientesUtxos => {
                write!(
                    f,
                    "ERROR: No hay suficientes UTXO para crear la transacción."
                )
            }
            NodoBitcoinError::NoHayCuentaSeleccionada => {
                write!(f, "ERROR: No hay ninguna cuenta seleccionada.")
            }
            NodoBitcoinError::CuentaNoEncontrada => {
                write!(f, "ERROR: No se encuentra la cuenta.")
            }
            NodoBitcoinError::NoSePuedeEnviarTransaccion => {
                write!(f, "ERROR: No se puede enviar la transacción.")
            }
            NodoBitcoinError::ErrorAlActualizarUTXOS => {
                write!(f, "ERROR: No se puede actualizar las UTXOs.")
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum InterfaceError {
    CreateAccount,
    EmptyFields,
    TargetAmountNotValid,
    FeeNotValid,
    TransactionNotSent
}

impl Error for InterfaceError {}

impl fmt::Display for InterfaceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InterfaceError::CreateAccount => {
                write!(f, "Hubo un error al crear la cuenta. Intente nuevamente.")
            }
            InterfaceError::EmptyFields => {
                write!(f, "Debe completar todos los campos para continuar.")
            }
            InterfaceError::TargetAmountNotValid => {
                write!(f, "El Target Amount debe ser numérico.")
            }
            InterfaceError::FeeNotValid => {
                write!(f, "El Fee debe ser numérico.")
            }
            InterfaceError::TransactionNotSent => {
                write!(f, "Hubo un error al enviar la transaccion. Intente nuevamente.")
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum InterfaceMessage {
    CreateAccount,
    TransactionSent
}

impl fmt::Display for InterfaceMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InterfaceMessage::CreateAccount => {
                write!(f, "Cuenta creada.")
            }
            InterfaceMessage::TransactionSent => {
                write!(f, "Transaccion enviada.")
            }
        }
    }
}
