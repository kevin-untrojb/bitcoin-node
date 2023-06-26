mod app_manager;
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
mod wallet;

use std::{env, println};

use crate::blockchain::transaction::{Transaction, TxIn, TxOut};
use crate::common::uint256::Uint256;
use crate::protocol::send_tx::_mock_tx_obj;
use crate::protocol::send_tx::send_tx;
use crate::{log::create_logger_actor, protocol::connection::connect};
use errores::NodoBitcoinError;
use interface::view::{self};

fn main() {
    let args: Vec<String> = env::args().collect();
    _ = config::inicializar(args);

    match gtk::init() {
        Ok(_) => {
            _ = view::create_view();
            gtk::main();
        }
        Err(_) => println!("No se pudo inicializar GTK."),
    }
}

fn _send_tx_main() {
    let args: Vec<String> = env::args().collect();
    let do_steps = || -> Result<(), NodoBitcoinError> {
        config::inicializar(args)?;
        let logger = create_logger_actor(config::get_valor("LOG_FILE".to_string()));
        let admin_connections = connect(logger.clone())?;
        let tx_obj = _mock_tx_obj()?;
        send_tx(admin_connections, logger, tx_obj)?;
        let nombre_grupo = config::get_valor("NOMBRE_GRUPO".to_string())?;
        println!("Hello, Bitcoin! Somos {}", nombre_grupo);
        Ok(())
    };

    if let Err(e) = do_steps() {
        println!("{}", e);
    }
}

fn _new_tx() {
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
        let public_account = "mzx5YhAH9kNHtcN481u6WkjeHjYtVeKVh2".to_string();
        let txout = TxOut::new(change_amount, public_account)?;

        let target_amount = 10000000;
        let target_account = "mnrVtF8DWjMu839VW3rBfgYaAfKk8983Xf".to_string();
        let tx_out_change = TxOut::new(target_amount, target_account)?;

        let tx_obj = Transaction::new(vec![tx_in], vec![txout, tx_out_change], 0)?;

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

fn _signature() {
    let args: Vec<String> = env::args().collect();
    let do_steps = || -> Result<(), NodoBitcoinError> {
        config::inicializar(args)?;
        let previous_tx = [
            0x01, 0x00, 0x00, 0x00, 0x02, 0x13, 0x7c, 0x53, 0xf0, 0xfb, 0x48, 0xf8, 0x36, 0x66,
            0xfc, 0xfd, 0x2f, 0xe9, 0xf1, 0x2d, 0x13, 0xe9, 0x4e, 0xe1, 0x09, 0xc5, 0xae, 0xab,
            0xbf, 0xa3, 0x2b, 0xb9, 0xe0, 0x25, 0x38, 0xf4, 0xcb, 0x00, 0x00, 0x00, 0x00, 0x6a,
            0x47, 0x30, 0x44, 0x02, 0x20, 0x7e, 0x60, 0x09, 0xad, 0x86, 0x36, 0x7f, 0xc4, 0xb1,
            0x66, 0xbc, 0x80, 0xbf, 0x10, 0xcf, 0x1e, 0x78, 0x83, 0x2a, 0x01, 0xe9, 0xbb, 0x49,
            0x1c, 0x6d, 0x12, 0x6e, 0xe8, 0xaa, 0x43, 0x6c, 0xb5, 0x02, 0x20, 0x0e, 0x29, 0xe6,
            0xdd, 0x77, 0x08, 0xed, 0x41, 0x9c, 0xd5, 0xba, 0x79, 0x89, 0x81, 0xc9, 0x60, 0xf0,
            0xcc, 0x81, 0x1b, 0x24, 0xe8, 0x94, 0xbf, 0xf0, 0x72, 0xfe, 0xa8, 0x07, 0x4a, 0x7c,
            0x4c, 0x01, 0x21, 0x03, 0xbc, 0x9e, 0x73, 0x97, 0xf7, 0x39, 0xc7, 0x0f, 0x42, 0x4a,
            0xa7, 0xdc, 0xce, 0x9d, 0x2e, 0x52, 0x1e, 0xb2, 0x28, 0xb0, 0xcc, 0xba, 0x61, 0x9c,
            0xd6, 0xa0, 0xb9, 0x69, 0x1d, 0xa7, 0x96, 0xa1, 0xff, 0xff, 0xff, 0xff, 0x51, 0x74,
            0x72, 0xe7, 0x7b, 0xc2, 0x9a, 0xe5, 0x9a, 0x91, 0x4f, 0x55, 0x21, 0x1f, 0x05, 0x02,
            0x45, 0x56, 0x81, 0x2a, 0x2d, 0xd7, 0xd8, 0xdf, 0x29, 0x32, 0x65, 0xac, 0xd8, 0x33,
            0x01, 0x59, 0x01, 0x00, 0x00, 0x00, 0x6b, 0x48, 0x30, 0x45, 0x02, 0x21, 0x00, 0xf4,
            0xbf, 0xdb, 0x0b, 0x31, 0x85, 0xc7, 0x78, 0xcf, 0x28, 0xac, 0xba, 0xf1, 0x15, 0x37,
            0x63, 0x52, 0xf0, 0x91, 0xad, 0x9e, 0x27, 0x22, 0x5e, 0x6f, 0x3f, 0x35, 0x0b, 0x84,
            0x75, 0x79, 0xc7, 0x02, 0x20, 0x0d, 0x69, 0x17, 0x77, 0x73, 0xcd, 0x2b, 0xb9, 0x93,
            0xa8, 0x16, 0xa5, 0xae, 0x08, 0xe7, 0x7a, 0x62, 0x70, 0xcf, 0x46, 0xb3, 0x3f, 0x8f,
            0x79, 0xd4, 0x5b, 0x0c, 0xd1, 0x24, 0x4d, 0x9c, 0x4c, 0x01, 0x21, 0x03, 0x1c, 0x0b,
            0x0b, 0x95, 0xb5, 0x22, 0x80, 0x5e, 0xa9, 0xd0, 0x22, 0x5b, 0x19, 0x46, 0xec, 0xae,
            0xb1, 0x72, 0x7c, 0x0b, 0x36, 0xc7, 0xe3, 0x41, 0x65, 0x76, 0x9f, 0xd8, 0xed, 0x86,
            0x0b, 0xf5, 0xff, 0xff, 0xff, 0xff, 0x02, 0x7a, 0x95, 0x88, 0x02, 0x00, 0x00, 0x00,
            0x00, 0x19, 0x76, 0xa9, 0x14, 0xa8, 0x02, 0xfc, 0x56, 0xc7, 0x04, 0xce, 0x87, 0xc4,
            0x2d, 0x7c, 0x92, 0xeb, 0x75, 0xe7, 0x89, 0x6b, 0xdc, 0x41, 0xae, 0x88, 0xac, 0xa5,
            0x51, 0x5e, 0x00, 0x00, 0x00, 0x00, 0x00, 0x19, 0x76, 0xa9, 0x14, 0xe8, 0x2b, 0xd7,
            0x5c, 0x9c, 0x66, 0x2c, 0x3f, 0x57, 0x00, 0xb3, 0x3f, 0xec, 0x8a, 0x67, 0x6b, 0x6e,
            0x93, 0x91, 0xd5, 0x88, 0xac, 0x00, 0x00, 0x00, 0x00,
        ];
        let previous_tx = Transaction::deserialize(&previous_tx[..])?;

        let tx_bytes = [
            0x01, 0x00, 0x00, 0x00, 0x01, 0x81, 0x3f, 0x79, 0x01, 0x1a, 0xcb, 0x80, 0x92, 0x5d,
            0xfe, 0x69, 0xb3, 0xde, 0xf3, 0x55, 0xfe, 0x91, 0x4b, 0xd1, 0xd9, 0x6a, 0x3f, 0x5f,
            0x71, 0xbf, 0x83, 0x03, 0xc6, 0xa9, 0x89, 0xc7, 0xd1, 0x00, 0x00, 0x00, 0x00, 0x6b,
            0x48, 0x30, 0x45, 0x02, 0x21, 0x00, 0xed, 0x81, 0xff, 0x19, 0x2e, 0x75, 0xa3, 0xfd,
            0x23, 0x04, 0x00, 0x4d, 0xca, 0xdb, 0x74, 0x6f, 0xa5, 0xe2, 0x4c, 0x50, 0x31, 0xcc,
            0xfc, 0xf2, 0x13, 0x20, 0xb0, 0x27, 0x74, 0x57, 0xc9, 0x8f, 0x02, 0x20, 0x7a, 0x98,
            0x6d, 0x95, 0x5c, 0x6e, 0x0c, 0xb3, 0x5d, 0x44, 0x6a, 0x89, 0xd3, 0xf5, 0x61, 0x00,
            0xf4, 0xd7, 0xf6, 0x78, 0x01, 0xc3, 0x19, 0x67, 0x74, 0x3a, 0x9c, 0x8e, 0x10, 0x61,
            0x5b, 0xed, 0x01, 0x21, 0x03, 0x49, 0xfc, 0x4e, 0x63, 0x1e, 0x36, 0x24, 0xa5, 0x45,
            0xde, 0x3f, 0x89, 0xf5, 0xd8, 0x68, 0x4c, 0x7b, 0x81, 0x38, 0xbd, 0x94, 0xbd, 0xd5,
            0x31, 0xd2, 0xe2, 0x13, 0xbf, 0x01, 0x6b, 0x27, 0x8a, 0xfe, 0xff, 0xff, 0xff, 0x02,
            0xa1, 0x35, 0xef, 0x01, 0x00, 0x00, 0x00, 0x00, 0x19, 0x76, 0xa9, 0x14, 0xbc, 0x3b,
            0x65, 0x4d, 0xca, 0x7e, 0x56, 0xb0, 0x4d, 0xca, 0x18, 0xf2, 0x56, 0x6c, 0xda, 0xf0,
            0x2e, 0x8d, 0x9a, 0xda, 0x88, 0xac, 0x99, 0xc3, 0x98, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x19, 0x76, 0xa9, 0x14, 0x1c, 0x4b, 0xc7, 0x62, 0xdd, 0x54, 0x23, 0xe3, 0x32, 0x16,
            0x67, 0x02, 0xcb, 0x75, 0xf4, 0x0d, 0xf7, 0x9f, 0xea, 0x12, 0x88, 0xac, 0x19, 0x43,
            0x06, 0x00,
        ];

        let mut transaction = Transaction::deserialize(&tx_bytes[..])?;

        let private_key_hexa: [u8; 32] = [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x84, 0x5f, 0xed,
        ];

        let input_index = 0;
        transaction.sign_with_hexa_key(input_index, private_key_hexa.to_vec(), previous_tx)?;

        let tx_in_bytes = transaction.input[0].serialize()?;
        println!("TxIn: {:02X?}", tx_in_bytes);

        let tx_bytes = transaction.serialize()?;
        println!("Tx: {:02X?}", tx_bytes);

        let nombre_grupo = config::get_valor("NOMBRE_GRUPO".to_string())?;
        println!("Hello, Bitcoin! Somos {}", nombre_grupo);
        Ok(())
    };

    if let Err(e) = do_steps() {
        println!("{}", e);
    }
}

fn _new_tx_signed() {
    let args: Vec<String> = env::args().collect();
    let do_steps = || -> Result<(), NodoBitcoinError> {
        config::inicializar(args)?;
        let previous_tx_id_bytes = [
            0x85, 0x25, 0xc9, 0xb1, 0x6d, 0x36, 0x3a, 0x81, 0x50, 0x09, 0x86, 0xe5, 0x9f, 0xd7,
            0xdc, 0x63, 0x0b, 0xda, 0x5d, 0x9c, 0x3c, 0x97, 0xa0, 0x34, 0xda, 0x56, 0xab, 0x58,
            0xe7, 0x5f, 0xcd, 0x1a,
        ];

        let previous_tx_id = Uint256::from_le_bytes(previous_tx_id_bytes);
        let previous_tx = Transaction::_get_tx_from_file(previous_tx_id)?;

        let prev_index: usize = 1;

        let private_key_wif = "cRJzHMCgDLsvttTH8R8t6LLcZgMDs1WtgwQXxk8bFFk7E2AJp1tw";

        let target_address = "mtm4vS3WH7pg13pjFEmqGq2TSPDcUN6k7a".to_string();
        let target_amount: u64 = 1000000;

        let change_address = "mnJvq7mbGiPNNhUne4FAqq27Q8xZrAsVun".to_string();
        let change_amount: u64 = 100000;

        let mut tx_ins = vec![];
        let tx_in = TxIn::new(previous_tx.txid()?, prev_index);
        tx_ins.push(tx_in);

        let mut tx_outs = vec![];
        let tx_out = TxOut::new(target_amount, target_address)?;
        tx_outs.push(tx_out);

        let tx_out_change = TxOut::new(change_amount, change_address)?;
        tx_outs.push(tx_out_change);

        let mut tx_obj = Transaction::new(tx_ins, tx_outs, 0)?;

        tx_obj.sign_with_wif_compressed_key(0, private_key_wif, previous_tx)?;

        let _tx_obj_bytes = tx_obj.serialize()?;

        let nombre_grupo = config::get_valor("NOMBRE_GRUPO".to_string())?;
        println!("Hello, Bitcoin! Somos {}", nombre_grupo);
        Ok(())
    };

    if let Err(e) = do_steps() {
        println!("{}", e);
    }
}
