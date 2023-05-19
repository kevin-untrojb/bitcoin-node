mod blockchain;
mod config;
mod errores;
mod messages;
mod parse_args;
mod protocol;
mod common;

use std::{env, println};

use errores::NodoBitcoinError;

use crate::{
    blockchain::node::Node,
    protocol::{connection::connect, initial_block_download::get_headers},
};

fn main() {
    let args: Vec<String> = env::args().collect();
    let do_steps = || -> Result<(), NodoBitcoinError> {
        config::inicializar(args)?;
        let connections = connect()?;
        let mut node = Node::new();
        get_headers(connections, &mut node)?;

        let nombre_grupo = config::get_valor("NOMBRE_GRUPO".to_string())?;
        println!("Hello, Bitcoin! Somos {}", nombre_grupo);
        Ok(())
    };

    if let Err(e) = do_steps() {
        println!("{}", e);
    }
}
