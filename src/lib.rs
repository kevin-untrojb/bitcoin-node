use errores::NodoBitcoinError;

pub mod config;
pub mod errores;
pub mod messages;
mod parse_args;
pub mod protocol;

use crate::config::init_config;

pub fn inicializar(args: Vec<String>) -> Result<(), NodoBitcoinError> {
    let filename = parse_args::parse_args(args)?;
    init_config(filename)?;
    Ok(())
}
