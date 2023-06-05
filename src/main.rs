mod blockchain;
mod common;
mod config;
mod errores;
mod log;
mod merkle_tree;
mod messages;
mod parse_args;
mod protocol;
use std::sync::mpsc::Sender;
use std::{env, println, thread};

use errores::NodoBitcoinError;
use gtk::{
    prelude::{ApplicationExt, ApplicationExtManual},
    traits::{ButtonExt, ContainerExt, WidgetExt},
    Align, Application, ApplicationWindow, Button,
};

use crate::protocol::block_broadcasting::init_block_broadcasting;
use crate::{
    blockchain::block::SerializedBlock,
    protocol::{connection::connect, initial_block_download::get_full_blockchain},
    log::{create_logger_actor, LogMessages}
};

fn main() {
    let args: Vec<String> = env::args().collect();
    _ = config::inicializar(args);
    let nombre_grupo = match config::get_valor("NOMBRE_GRUPO".to_string()) {
        Ok(valor) => valor,
        Err(e) => {
            println!("{}", e);
            return;
        }
    };

    let title = format!("Nodo Bitcoin - {}", nombre_grupo);
    let app = Application::builder()
        .application_id("nodo_bitcoin")
        .build();

    app.connect_activate(move |app| {
        let window = ApplicationWindow::builder()
            .application(app)
            .default_width(460)
            .default_height(200)
            .title(&title)
            .build();

        let button_download_blockchain = Button::builder()
            .label("Descargar Bloques")
            .halign(Align::Center)
            .valign(Align::Center)
            .build();

        button_download_blockchain.connect_clicked(|_| {
            thread::spawn(move || {
                download_blockchain(create_logger_actor(config::get_valor(
                    "LOG_FILE".to_string(),
                )));
            });
        });

        let button_read_blocks = Button::builder()
            .label("Leer Bloques")
            .halign(Align::Center)
            .valign(Align::Center)
            .build();

        button_read_blocks.connect_clicked(|_| {
            thread::spawn(move || {
                println!("Leyendo!");
                //leer_primer_block();
                block_broadcasting(create_logger_actor(config::get_valor(
                    "LOG_FILE".to_string(),
                )));
            });
        });

        let box_layout = gtk::Box::new(gtk::Orientation::Vertical, 20);
        box_layout.add(&button_download_blockchain);
        box_layout.add(&button_read_blocks);

        window.set_child(Some(&box_layout));
        window.show_all();
    });

    app.run();
}

fn download_blockchain(logger: Sender<LogMessages>) {
    let args: Vec<String> = env::args().collect();
    let do_steps = || -> Result<(), NodoBitcoinError> {
        config::inicializar(args)?;
        let admin_connections = connect(logger.clone())?;
        get_full_blockchain(logger.clone(), admin_connections)?;

        let nombre_grupo = config::get_valor("NOMBRE_GRUPO".to_string())?;
        println!("Hello, Bitcoin! Somos {}", nombre_grupo);
        Ok(())
    };

    if let Err(e) = do_steps() {
        println!("{}", e);
    }
}

fn leer_primer_block() {
    let args: Vec<String> = env::args().collect();
    let do_steps = || -> Result<(), NodoBitcoinError> {
        config::inicializar(args)?;
        let bloques = SerializedBlock::read_blocks_from_file()?;
        println!("Bloques totales: {:?}", bloques.len());

        let nombre_grupo = config::get_valor("NOMBRE_GRUPO".to_string())?;
        println!("Hello, Bitcoin! Somos {}", nombre_grupo);
        Ok(())
    };

    if let Err(e) = do_steps() {
        println!("{}", e);
    }
}

fn block_broadcasting(logger: Sender<LogMessages>){
    let args: Vec<String> = env::args().collect();
    let do_steps = || -> Result<(), NodoBitcoinError> {
        config::inicializar(args)?;
        let admin_connections = connect(logger.clone())?;
        init_block_broadcasting(logger.clone(),admin_connections);

        let nombre_grupo = config::get_valor("NOMBRE_GRUPO".to_string())?;
        println!("Hello, Bitcoin! Somos {}", nombre_grupo);
        Ok(())
    };

    if let Err(e) = do_steps() {
        println!("{}", e);
    }
}
