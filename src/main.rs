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
    prelude::*,
    traits::{ButtonExt, ContainerExt, WidgetExt},
    Align, Application, ApplicationWindow, Button, Window, Builder,
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

    gtk::init().expect("No se pudo inicializar GTK.");

    let glade_src = include_str!("gtk/window.glade");
    let builder = Builder::from_string(glade_src);

    let window: Window = builder.object("window").expect("Error: No encuentra objeto 'window'");

    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    /*button.connect_clicked(|_| {
        thread::spawn(move || {
            println!("Descargando!");
            download_blockchain();
        });
    });*/

    window.show_all();

    gtk::main();

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
