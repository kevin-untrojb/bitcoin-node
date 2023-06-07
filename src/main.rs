mod blockchain;
mod common;
mod config;
mod errores;
mod merkle_tree;
mod messages;
mod parse_args;
mod protocol;

use std::sync::Arc;
use std::sync::Mutex;
use std::{env, println, thread};

use chrono::Duration;
use errores::NodoBitcoinError;
use glib::PRIORITY_DEFAULT;
use gtk::Label;
use gtk::glib::MainContext;
use gtk::glib::PropertyGet;
use gtk::glib::Sender;
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
    window.set_title(&title);

    let (mut sender, receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

    let label: Label = builder.object("id_prueba").expect("Error: No encuentra objeto 'id_prueba'");
    let button: Button = builder.object("prueba_button").expect("Error: No encuentra objeto 'prueba_button'");

    let sender_clone = sender.clone();

    thread::spawn(move || {

        let data2 = "SIN CLICK!!!!!!"; // Datos a enviar
        thread::sleep(std::time::Duration::from_secs(5));
        let result = format!("Datos enviados: {}", data2);
        sender_clone.send(result).expect("Failed to send data result.");
        
        let data3 = "CAMBIA LABEL!!!!!!"; // Datos a enviar
        thread::sleep(std::time::Duration::from_secs(10));
        let result2 = format!("Datos enviados: {}", data3);
        sender_clone.send(result2).expect("Failed to send data result.");
        });
    
    let data = "CLICK"; // Datos a enviar

    button.connect_clicked(move |_| {
        let sender_clone = sender.clone();

        thread::spawn(move || {
            let result = format!("Datos enviados: {}", data);
            sender_clone.send(result).expect("Failed to send data result.");
        });
    });

    receiver.attach(None, move |result| {
        label.set_text(&result);
        glib::Continue(true)
    });
    

    /*button.connect_clicked(|_| {
        thread::spawn(move || {
            println!("Descargando!");
            download_blockchain();
        });
    });*/

    window.show_all();

    gtk::main();

    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

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
