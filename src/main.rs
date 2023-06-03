mod blockchain;
mod common;
mod config;
mod errores;
mod merkle_tree;
mod messages;
mod parse_args;
mod protocol;
mod wallet;

use std::{env, println, thread};

use errores::NodoBitcoinError;
use gtk::{
    prelude::{ApplicationExt, ApplicationExtManual},
    traits::{ButtonExt, ContainerExt, WidgetExt},
    Align, Application, ApplicationWindow, Button,
};

use crate::{
    blockchain::node::Node,
    protocol::{connection::connect, initial_block_download::get_headers},
    wallet::{uxto_set},
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

    let title = format!("Nodo Bitcoin - {}", nombre_grupo).to_string();
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
            .label("Descargar Blockchain")
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
        let mut node = Node::new();
        get_headers(admin_connections, &mut node)?;

        let nombre_grupo = config::get_valor("NOMBRE_GRUPO".to_string())?;
        println!("Hello, Bitcoin! Somos {}", nombre_grupo);
        Ok(())
    };

    if let Err(e) = do_steps() {
        println!("{}", e);
    }
}
