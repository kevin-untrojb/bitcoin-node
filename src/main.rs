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
use gtk::{Spinner, TextView};
use interface::view::ViewObject;
use interface::view::ViewObjectData;
use std::sync::{Arc, Mutex};

use crate::protocol::block_broadcasting::init_block_broadcasting;
use crate::protocol::{connection::connect, initial_block_download::get_full_blockchain};
use log::{create_logger_actor, LogMessages};

use glib::Sender;
use gtk::{
    prelude::*,
    traits::{ButtonExt, WidgetExt},
    Builder, Button, Label, Window,
};

fn main() {
    let args: Vec<String> = env::args().collect();
    _ = config::inicializar(args);

    gtk::init().expect("No se pudo inicializar GTK.");
    let title = format!("Nodo Bitcoin - Los Rustybandidos");
    let glade_src = include_str!("interface/window.glade");

    let (sender, receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

    let builder = Builder::from_string(glade_src);
    let window: Window = builder
        .object("window")
        .expect("Error: No encuentra objeto 'window'");
    window.set_title(&title);
    window.show_all();

    receiver.attach(None, move |view_object: ViewObject| {
        match view_object {
            ViewObject::Label(data) => {
                println!(
                    "RECEIVER LABEL {} {}",
                    &String::from(&data.id),
                    &data.text.to_string()
                );
                let label: Label = builder.object(&String::from(data.id)).expect("error");
                label.set_text(&data.text.to_string());
            }
            ViewObject::Button(data) => {
                println!(
                    "RECEIVER BUTTON {} {}",
                    String::from(&data.id),
                    data.text.to_string()
                );
                let button: Button = builder.object(&String::from(data.id)).expect("error");
                button.set_label(&data.text.to_string());
            }
            ViewObject::Spinner(data) => {
                println!(
                    "RECEIVER SPINNER {} {}",
                    String::from(&data.id),
                    data.active.to_string()
                );
                let button: Spinner = builder.object(&String::from(data.id)).expect("error");
                button.set_active(data.active);
            }
            ViewObject::TextView(data) => {
                let text_view: TextView = builder.object(&String::from(data.id)).expect("error");
                let buffer = text_view.buffer().unwrap();
                buffer.insert_at_cursor(&data.text.to_string());
            }
        }
        glib::Continue(true)
    });

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
        let admin_connections = connect(logger.clone(), sender.clone())?;
        get_full_blockchain(logger.clone(), sender.clone(), admin_connections.clone())?;
        init_block_broadcasting(logger.clone(), admin_connections.clone())?;
        let nombre_grupo = config::get_valor("NOMBRE_GRUPO".to_string())?;
        println!("Hello, Bitcoin! Somos {}", nombre_grupo);
        Ok(())
    };

    if let Err(e) = do_steps() {
        println!("{}", e);
    }
}
