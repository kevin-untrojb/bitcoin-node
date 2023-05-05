use errores::NodoBitcoinError;

pub mod errores;
pub mod connection;
pub mod config;
mod parse_args;

use crate::config::init_config;

pub fn inicializar(args: Vec<String>) -> Result<(), NodoBitcoinError> {
    let filename = parse_args::parse_args(args)?;
    init_config(filename)?;
    Ok(())
}
