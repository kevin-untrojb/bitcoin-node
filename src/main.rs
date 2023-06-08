mod blockchain;
mod common;
mod config;
mod errores;
mod merkle_tree;
mod messages;
mod parse_args;
mod protocol;
mod interface;

use std::sync::Arc;
use std::sync::Mutex;
use std::{env, println, thread};

use errores::NodoBitcoinError;
use glib::Sender;
use gtk::Button;
use gtk::Label;
use interface::view::View;
use interface::view::ViewObject;
use interface::view::ViewObjectType;


use crate::protocol::{connection::connect, initial_block_download::get_full_blockchain};

fn main() {
    let args: Vec<String> = env::args().collect();
    _ = config::inicializar(args);

    gtk::init().expect("No se pudo inicializar GTK.");

    let new_view = View::new();

    let view_clone = Arc::clone(&new_view);
    let view_result = view_clone.lock();
    if let Ok(view_guard) = view_result {
        let view_object = ViewObject{ id: "id_prueba".to_string(), text: "bla".to_string() };
        let label = Label::new(None);
        let object = ViewObjectType::Label(label.clone());
        view_guard.sender.send((view_object, object));

        let view_object2 = ViewObject{ id: "prueba_button".to_string(), text: "Cambio".to_string() };
        let button = Button::new();
        let object2 = ViewObjectType::Button(button.clone());
        view_guard.sender.send((view_object2, object2));
        let clone = (view_guard.sender).clone();
        //download_blockchain(clone);
    }

    gtk::main();
}

fn download_blockchain(sender: Sender<ViewObject>) {
    let do_steps = || -> Result<(), NodoBitcoinError> {
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
