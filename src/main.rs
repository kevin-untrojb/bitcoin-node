mod errores;
mod messages;
mod protocol;
use std::env;

use crate::protocol::{connection::connect, initial_block_download::get_headers};
use ::los_rustybandidos::inicializar;
use los_rustybandidos::{config, errores::NodoBitcoinError};

fn main() {
    let args: Vec<String> = env::args().collect();
    let do_steps = || -> Result<(), NodoBitcoinError> {
        inicializar(args)?;
        let connections = connect().unwrap();
        get_headers(connections);

        let nombre_grupo = config::get_valor("NOMBRE_GRUPO".to_string())?;
        println!("Hello, Bitcoin! Somos {}", nombre_grupo);
        Ok(())
    };

    if let Err(e) = do_steps() {
        println!("{}", e);
    }
}
