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

use crate::blockchain::transaction::{Transaction, TxIn, TxOut};
use crate::common::base58::decode_base58;
use crate::common::uint256::Uint256;
use crate::protocol::block_broadcasting::init_block_broadcasting;
use crate::{
    log::{create_logger_actor, LogMessages},
    protocol::{connection::connect, initial_block_download::get_full_blockchain},
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
                new_tx();
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
        get_full_blockchain(logger.clone(), admin_connections.clone())?;
        init_block_broadcasting(logger.clone(), admin_connections)?;
        let nombre_grupo = config::get_valor("NOMBRE_GRUPO".to_string())?;
        println!("Hello, Bitcoin! Somos {}", nombre_grupo);
        Ok(())
    };

    if let Err(e) = do_steps() {
        println!("{}", e);
    }
}

fn new_tx() {
    let args: Vec<String> = env::args().collect();
    let do_steps = || -> Result<(), NodoBitcoinError> {
        config::inicializar(args)?;
        let prev_tx_bytes = [
            0x0d, 0x6f, 0xe5, 0x21, 0x3c, 0x0b, 0x32, 0x91, 0xf2, 0x08, 0xcb, 0xa8, 0xbf, 0xb5,
            0x9b, 0x74, 0x76, 0xdf, 0xfa, 0xcc, 0x4e, 0x5c, 0xb6, 0x6f, 0x6e, 0xb2, 0x0a, 0x08,
            0x08, 0x43, 0xa2, 0x99,
        ];
        let prev_tx = Uint256::from_le_bytes(prev_tx_bytes);
        let prev_index = 13;
        let tx_in = TxIn::new(prev_tx, prev_index);

        let change_amount = 33000000;
        let public_account = "mzx5YhAH9kNHtcN481u6WkjeHjYtVeKVh2";
        let script = decode_base58(public_account)?;
        let txout = TxOut::new(change_amount, script)?;

        let target_amount = 10000000;
        let target_account = "mnrVtF8DWjMu839VW3rBfgYaAfKk8983Xf";
        let target_h160 = decode_base58(target_account)?;
        let tx_out_change = TxOut::new(target_amount, target_h160)?;

        let tx_obj = Transaction::new(1, vec![tx_in], vec![txout, tx_out_change], 0)?;

        let serialize = tx_obj.serialize()?;
        println!("serialize: {:?}", serialize);

        let bytes_serializer_oreilly = [
            0x01, 0x00, 0x00, 0x00, 0x01, 0x99, 0xa2, 0x43, 0x08, 0x08, 0x0a, 0xb2, 0x6e, 0x6f,
            0xb6, 0x5c, 0x4e, 0xcc, 0xfa, 0xdf, 0x76, 0x74, 0x9b, 0xb5, 0xbf, 0xa8, 0xcb, 0x08,
            0xf2, 0x91, 0x32, 0x0b, 0x3c, 0x21, 0xe5, 0x6f, 0x0d, 0x0d, 0x00, 0x00, 0x00, 0x00,
            0xff, 0xff, 0xff, 0xff, 0x02, 0x40, 0x8a, 0xf7, 0x01, 0x00, 0x00, 0x00, 0x00, 0x19,
            0x76, 0xa9, 0x14, 0xd5, 0x2a, 0xd7, 0xca, 0x9b, 0x3d, 0x09, 0x6a, 0x38, 0xe7, 0x52,
            0xc2, 0x01, 0x8e, 0x6f, 0xbc, 0x40, 0xcd, 0xf2, 0x6f, 0x88, 0xac, 0x80, 0x96, 0x98,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x19, 0x76, 0xa9, 0x14, 0x50, 0x7b, 0x27, 0x41, 0x1c,
            0xcf, 0x7f, 0x16, 0xf1, 0x02, 0x97, 0xde, 0x6c, 0xef, 0x3f, 0x29, 0x16, 0x23, 0xed,
            0xdf, 0x88, 0xac, 0x00, 0x00, 0x00, 0x00,
        ];

        let bytes_tx = serialize.as_slice();
        let compare = bytes_tx == bytes_serializer_oreilly;
        println!("compare: {:?}", compare);

        let nombre_grupo = config::get_valor("NOMBRE_GRUPO".to_string())?;
        println!("Hello, Bitcoin! Somos {}", nombre_grupo);
        Ok(())
    };

    if let Err(e) = do_steps() {
        println!("{}", e);
    }
}
