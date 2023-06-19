use super::admin_connections::AdminConnections;
use crate::{
    blockchain::{
        block::SerializedBlock,
        transaction::{create_tx_to_send, Transaction},
    },
    errores::NodoBitcoinError,
    log::{log_error_message, LogMessages},
    messages::messages_header::make_header,
    wallet::{user::Account, uxto_set::UTXOSet},
};
use std::sync::mpsc;

pub fn send_tx(
    mut admin_connections: AdminConnections,
    logger: mpsc::Sender<LogMessages>,
) -> Result<(), NodoBitcoinError> {
    let tx_obj = mock_tx_obj()?;

    let tx_obj_bytes = tx_obj.serialize()?;
    println!("tx_obj_bytes: {:02X?}", tx_obj_bytes);

    let payload: Vec<u8> = tx_obj_bytes.clone();
    let header = make_header("tx".to_string(), &payload)?;
    let mut tx_msg = Vec::new();
    tx_msg.extend_from_slice(&header);
    tx_msg.extend_from_slice(&payload);

    for connection in admin_connections.get_connections() {
        if connection.write_message(&tx_msg).is_err() {
            log_error_message(
                logger,
                "Error al enviar la nueva transacción a un peer.".to_string(),
            );
            return Err(NodoBitcoinError::NoSePuedeEscribirLosBytes);
        }
    }

    Ok(())
}

/*
Cuentas de prueba:
    - Public key: mnJvq7mbGiPNNhUne4FAqq27Q8xZrAsVun
    - Private key: cRJzHMCgDLsvttTH8R8t6LLcZgMDs1WtgwQXxk8bFFk7E2AJp1tw


    - Public key: mtm4vS3WH7pg13pjFEmqGq2TSPDcUN6k7a
    - Private key: cU7dbzeBRgMEZ5BUst2CFydGRm9gt8uQbNoojWPRRuHb2xk5q5h2


    - Public key: mmE8B9CGNBqCP8nDQvKW7Uab3eDGEKmziG
    - Private key: cVcf7ZMBWAanLmWy4QUHpNJEfNvX8n8NowAwzsDA1Qq82mk34drz

 */
fn mock_tx_obj() -> Result<Transaction, NodoBitcoinError> {
    let private_key = "cRJzHMCgDLsvttTH8R8t6LLcZgMDs1WtgwQXxk8bFFk7E2AJp1tw".to_string();
    let public_key = "mnJvq7mbGiPNNhUne4FAqq27Q8xZrAsVun".to_string();
    let account_name = "test".to_string();
    let account = Account::new(private_key, public_key.clone(), account_name);

    let blocks = SerializedBlock::read_blocks_from_file()?;
    let txns = blocks
        .iter()
        .flat_map(|bloque| bloque.txns.clone())
        .collect::<Vec<_>>();

    let mut utxo_set = UTXOSet::new();
    utxo_set.update_from_transactions(txns, vec![account.clone()])?;

    let utxos_by_account = utxo_set.utxos_for_account;
    let utxos = utxos_by_account.get(&public_key).unwrap().clone();

    let target_address = "mtm4vS3WH7pg13pjFEmqGq2TSPDcUN6k7a".to_string();
    let target_amount: u64 = 1700000;

    let fee: u64 = 71052;

    let tx_obj = create_tx_to_send(account, target_address, target_amount, fee, utxos)?;
    Ok(tx_obj)
}
