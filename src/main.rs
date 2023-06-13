mod blockchain;
mod common;
mod config;
mod errores;
mod interface;
mod log;
mod merkle_tree;
mod messages;
mod parse_args;
mod protocol;

use std::sync::mpsc;
use std::{env, println, thread};

use errores::NodoBitcoinError;
use interface::view::{ViewObject, self};

use crate::interface::view::{end_loading, start_loading};
use crate::protocol::block_broadcasting::init_block_broadcasting;
use crate::protocol::{connection::connect, initial_block_download::get_full_blockchain};
use log::{create_logger_actor, LogMessages};

fn main() {
    let args: Vec<String> = env::args().collect();
    _ = config::inicializar(args);

    gtk::init().expect("No se pudo inicializar GTK.");
    let sender = view::create_view();

    thread::spawn(move || {
        download_blockchain(
            create_logger_actor(config::get_valor("LOG_FILE".to_string())),
            sender.clone(),
        );
    });

    gtk::main();
}

fn download_blockchain(logger: mpsc::Sender<LogMessages>, sender: glib::Sender<ViewObject>) {
    let do_steps = || -> Result<(), NodoBitcoinError> {
        start_loading(sender.clone(), "Connecting to peers... ".to_string());
        let admin_connections = connect(logger.clone())?;
        end_loading(sender.clone());
        start_loading(sender.clone(), "Obteniendo blockchain... ".to_string());
        get_full_blockchain(logger.clone(), admin_connections.clone())?;
        end_loading(sender.clone());
        init_block_broadcasting(logger.clone(), admin_connections.clone())?;
        let nombre_grupo = config::get_valor("NOMBRE_GRUPO".to_string())?;
        println!("Hello, Bitcoin! Somos {}", nombre_grupo);
        Ok(())
    };

    if let Err(e) = do_steps() {
        println!("{}", e);
    }
}
