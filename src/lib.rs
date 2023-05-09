use errores::NodoBitcoinError;

pub mod config;
pub mod errores;
mod parse_args;
pub mod protocol;
pub mod messages;

use crate::config::init_config;

pub fn inicializar(args: Vec<String>) -> Result<(), NodoBitcoinError> {
    let filename = parse_args::parse_args(args)?;
    init_config(filename)?;
    Ok(())
}
