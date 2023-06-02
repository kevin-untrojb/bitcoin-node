mod blockchain;
mod common;
mod config;
mod errores;
mod merkle_tree;
mod messages;
mod parse_args;
mod protocol;

use std::{env, println, thread};

use errores::NodoBitcoinError;
use gtk::{
    prelude::{ApplicationExt, ApplicationExtManual},
    traits::{ButtonExt, ContainerExt, WidgetExt},
    Align, Application, ApplicationWindow, Button,
};

use crate::protocol::{connection::connect, initial_block_download::get_full_blockchain};

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

        let button = Button::builder()
            .label("Descargar Bloques")
            .halign(Align::Center)
            .valign(Align::Center)
            .build();

        button.connect_clicked(|_| {
            thread::spawn(move || {
                println!("Descargando!");
                download_blockchain();
            });
        });

        window.set_child(Some(&button));
        window.show_all();
    });

    app.run();
}

fn download_blockchain() {
    let args: Vec<String> = env::args().collect();
    let do_steps = || -> Result<(), NodoBitcoinError> {
        config::inicializar(args)?;
        let admin_connections = connect()?;
        get_full_blockchain(admin_connections)?;

        let nombre_grupo = config::get_valor("NOMBRE_GRUPO".to_string())?;
        println!("Hello, Bitcoin! Somos {}", nombre_grupo);
        Ok(())
    };

    if let Err(e) = do_steps() {
        println!("{}", e);
    }
}
