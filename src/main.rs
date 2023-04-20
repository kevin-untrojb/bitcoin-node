use std::env;

use errores::NodoBitcoinError;

mod config;
mod errores;
mod parse_args;

fn main() {
    let args: Vec<String> = env::args().collect();
    let do_steps = || -> Result<(), NodoBitcoinError> {
        let filename = parse_args::parse_args(args)?;
        config::init_config(filename)?;
        let nombre_grupo = config::get_valor("NOMBRE_GRUPO".to_string())?;
        println!("Hello, Bitcoin! Somos {}", nombre_grupo);
        Ok(())
    };

    if let Err(e) = do_steps() {
        println!("{}", e);
    }
}
