use std::sync::mpsc;
use crate::{log::{LogMessages, log_error_message}, messages::messages_header::make_header, errores::NodoBitcoinError, common::uint256::Uint256, blockchain::transaction::{TxIn, TxOut, Transaction}};
use super::admin_connections::AdminConnections;

pub fn send_tx(mut admin_connections: AdminConnections, logger: mpsc::Sender<LogMessages>) -> Result<(), NodoBitcoinError>{
    let previous_tx_id_bytes = [
            4, 231, 18, 2, 177, 152, 255, 108, 11, 117, 224, 66, 60, 155, 18, 22, 191, 172, 128,
            111, 212, 221, 10, 170, 184, 104, 181, 127, 64, 0, 251, 27,
    ];
    let previous_tx_id = Uint256::from_be_bytes(previous_tx_id_bytes);
    let previous_tx = Transaction::get_tx_from_file(previous_tx_id)?;

    let prev_index: usize = 0;

    let private_key_wif = "cU7dbzeBRgMEZ5BUst2CFydGRm9gt8uQbNoojWPRRuHb2xk5q5h2";

    let target_address = "mnJvq7mbGiPNNhUne4FAqq27Q8xZrAsVun";
    let target_amount: usize = 100000;

    let change_address = "mtm4vS3WH7pg13pjFEmqGq2TSPDcUN6k7a";
    let change_amount: usize = 800000;

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

    for connection in admin_connections.get_connections() {
        let payload: Vec<u8> = tx_obj_bytes.clone();
        let tx_msg = make_header("tx".to_string(), &payload)?;

        if connection.write_message(&tx_msg).is_err() {
            log_error_message(logger, "Error al enviar la nueva transacción a un peer.".to_string());
            return Err(NodoBitcoinError::NoSePuedeEscribirLosBytes);
        }
    }

    Ok(())
}