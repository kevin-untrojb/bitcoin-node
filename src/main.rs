mod connection;

use std::env;

use ::los_rustybandidos::inicializar;
use los_rustybandidos::{config, errores::NodoBitcoinError};

use crate::connection::connection::VersionMessage;

fn main() {
    let args: Vec<String> = env::args().collect();
    let do_steps = || -> Result<(), NodoBitcoinError> {
        inicializar(args)?;
        VersionMessage::connect();

        let nombre_grupo = config::get_valor("NOMBRE_GRUPO".to_string())?;
        println!("Hello, Bitcoin! Somos {}", nombre_grupo);
        Ok(())
    };

    if let Err(e) = do_steps() {
        println!("{}", e);
    }
}
