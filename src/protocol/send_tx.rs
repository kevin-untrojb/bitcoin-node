use std::sync::mpsc;
use crate::{log::{LogMessages, log_error_message}, messages::messages_header::make_header, errores::NodoBitcoinError, common::uint256::Uint256, blockchain::transaction::{TxIn, TxOut, Transaction}};
use super::admin_connections::AdminConnections;

pub fn send_tx(mut admin_connections: AdminConnections, logger: mpsc::Sender<LogMessages>) -> Result<(), NodoBitcoinError>{
    let previous_tx_id_bytes = [134, 46, 136, 248, 208, 37, 180, 182, 25, 67, 20, 53, 244, 
        66, 208, 74, 237, 139, 218, 2, 27, 116, 240, 156, 232, 77, 21, 1, 12, 206, 51, 97
    ];
    let previous_tx_id = Uint256::from_be_bytes(previous_tx_id_bytes);
    let previous_tx = Transaction::get_tx_from_file(previous_tx_id)?;

    let prev_index: usize = 1;

    let private_key_wif = "cU7dbzeBRgMEZ5BUst2CFydGRm9gt8uQbNoojWPRRuHb2xk5q5h2";

    let target_address = "mnJvq7mbGiPNNhUne4FAqq27Q8xZrAsVun";
    let target_amount: usize = 100000;

    let change_address = "mtm4vS3WH7pg13pjFEmqGq2TSPDcUN6k7a";
    let change_amount: usize = 600000;

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

    let tx_obj_bytes = tx_obj.serialize()?;
    println!("tx_obj_bytes: {:02X?}", tx_obj_bytes);

    let payload: Vec<u8> = tx_obj_bytes.clone();
    let header = make_header("tx".to_string(), &payload)?;
    let mut tx_msg = Vec::new();
    tx_msg.extend_from_slice(&header);
    tx_msg.extend_from_slice(&payload);

    for connection in admin_connections.get_connections() {
        if connection.write_message(&tx_msg).is_err() {
            log_error_message(logger, "Error al enviar la nueva transacción a un peer.".to_string());
            return Err(NodoBitcoinError::NoSePuedeEscribirLosBytes);
        }
    }

    Ok(())
}