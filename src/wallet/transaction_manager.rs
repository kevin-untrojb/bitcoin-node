use std::collections::HashMap;
use std::sync::mpsc::{channel, Sender};
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

use crate::app_manager::ApplicationManagerMessages;
use crate::blockchain::block::SerializedBlock;
use crate::blockchain::transaction::{create_tx_to_send, Transaction};
use crate::common::uint256::Uint256;
use crate::errores::NodoBitcoinError;
use crate::log::{log_error_message, log_info_message, LogMessages};
use crate::protocol::admin_connections::AdminConnections;
use crate::protocol::block_broadcasting::{init_block_broadcasting, BlockBroadcastingMessages};
use crate::protocol::send_tx::send_tx;
use crate::wallet::uxto_set::UTXOSet;

use super::user::Account;

#[derive(Clone)]
pub struct TransactionManager {
    pub utxos: UTXOSet,
    tx_pendings: HashMap<Uint256, Transaction>,
    accounts: Vec<Account>,
    sender_app_manager: Sender<ApplicationManagerMessages>,
    sender_block_broadcasting: Option<Sender<BlockBroadcastingMessages>>,
    admin_connections: Option<AdminConnections>,
    // TODO guardar hilos abiertos para despues cerrarlos (block broadcasting)
}

pub enum TransactionMessages {
    GetAvailableAndPending(String),
    GetTxReportByAccount(String),
    _UpdateFromBlocks(
        (
            Vec<SerializedBlock>,
            Vec<Account>,
            Sender<Result<(), NodoBitcoinError>>,
        ),
    ),
    AddAccount(Vec<Account>, Sender<LogMessages>),
    InitBlockBroadcasting(
        (
            AdminConnections,
            Sender<LogMessages>,
            Sender<TransactionMessages>,
        ),
    ),
    SendTx(Account, String, u64, u64, Sender<LogMessages>),
    NewBlock(SerializedBlock),
    NewTx(Transaction),
    SenderBlockBroadcasting(Sender<BlockBroadcastingMessages>),
    LoadSavedUTXOS,
    ShutDown,
    Shutdowned,
}

impl TransactionManager {
    fn handle_message(&mut self, message: TransactionMessages) {
        match message {
            TransactionMessages::GetAvailableAndPending(account) => {
                let available_amount = match self.utxos.get_available(account) {
                    Ok(available_amount) => available_amount,
                    Err(_) => 0,
                };

                let pending_amount = 0; //todo!

                self.sender_app_manager
                    .send(ApplicationManagerMessages::GetAmountsByAccount(
                        available_amount,
                        pending_amount,
                    ));
            }
            TransactionMessages::GetTxReportByAccount(account) => {
                let tx_by_account = match self.utxos.tx_report_by_accounts.get(&account) {
                    Some(tx) => tx.clone(),
                    None => Vec::new(),
                };
                self.sender_app_manager
                    .send(ApplicationManagerMessages::GetTxReportByAccount(
                        tx_by_account.clone(),
                    ));
            }
            TransactionMessages::_UpdateFromBlocks((blocks, accounts, result)) => {
                result.send(self.utxos.update_from_blocks(blocks, accounts));
                self.sender_app_manager
                    .send(ApplicationManagerMessages::TransactionManagerUpdate);
            }
            TransactionMessages::AddAccount(accounts, logger) => {
                self.accounts = accounts;
                self.utxos.last_timestamp = 0;
                let utxos_updated = match update_utxos_from_file(
                    logger.clone(),
                    self.utxos.clone(),
                    self.accounts.clone(),
                ) {
                    Ok(uxtos) => uxtos,
                    Err(_) => {
                        log_error_message(logger.clone(), "Error al inicializar UTXOS".to_string());
                        return;
                    }
                };
                self.utxos = utxos_updated;
                log_info_message(logger.clone(), "UTXOS actualizadas".to_string());
                self.sender_app_manager
                    .send(ApplicationManagerMessages::TransactionManagerUpdate);
            }
            TransactionMessages::InitBlockBroadcasting((
                admin_connections,
                logger,
                sender_tx_manager,
            )) => {
                let utxos_updated = match update_utxos_from_file(
                    logger.clone(),
                    self.utxos.clone(),
                    self.accounts.clone(),
                ) {
                    Ok(uxtos) => uxtos,
                    Err(_) => {
                        log_error_message(logger.clone(), "Error al inicializar UTXOS".to_string());
                        return;
                    }
                };
                self.utxos = utxos_updated;
                log_info_message(logger.clone(), "UTXOS actualizadas".to_string());
                self.admin_connections = Some(admin_connections.clone());
                log_info_message(logger.clone(), "Inicio del block broadcasting.".to_string());
                thread::spawn(move || {
                    init_block_broadcasting(logger, admin_connections, sender_tx_manager);
                });
                self.sender_app_manager
                    .send(ApplicationManagerMessages::TransactionManagerUpdate);
            }
            TransactionMessages::NewBlock(block) => {
                let txns = block.txns.clone();
                let _ = self
                    .utxos
                    .update_from_blocks(vec![block], self.accounts.clone());
                for tx in txns {
                    self.update_pendings(tx.txid().unwrap());
                }
                self.sender_app_manager.send(ApplicationManagerMessages::TransactionManagerUpdate);
                self.sender_app_manager.send(ApplicationManagerMessages::NewBlock);
            }
            TransactionMessages::NewTx(tx) => {
                self.tx_pendings.insert(tx.txid().unwrap(), tx);
                self.sender_app_manager.send(ApplicationManagerMessages::NewTx);
            }
            TransactionMessages::SenderBlockBroadcasting(sender_block_broadcasting) => {
                self.sender_block_broadcasting = Some(sender_block_broadcasting);
            }
            TransactionMessages::SendTx(account, target_address, target_amount, fee, logger) => {
                let utxos = self.utxos.clone();
                let admin_connections = self.admin_connections.clone();
                let _ = send_new_tx(
                    account,
                    target_address,
                    target_amount,
                    fee,
                    utxos,
                    admin_connections,
                    logger,
                );
            }
            TransactionMessages::LoadSavedUTXOS => {
                // cargar los utxos guardados en el archivo
                let _ = self.utxos.load();
            }
            TransactionMessages::ShutDown => {
                // guardar utxos en archivo
                let _ = self.utxos.save();
                let _ = match &self.sender_block_broadcasting {
                    Some(sender) => {
                        sender.send(BlockBroadcastingMessages::ShutDown);
                        return;
                    }
                    None => {
                        return;
                    }
                };
            }
            TransactionMessages::Shutdowned => {
                self.sender_app_manager
                    .send(ApplicationManagerMessages::ShutDowned);
            }
        }
    }

    fn update_pendings(&mut self, tx_id: Uint256) {
        self.tx_pendings.remove(&tx_id);
    }
}

fn update_utxos_from_file(
    logger: Sender<LogMessages>,
    utxo_set: UTXOSet,
    accounts: Vec<Account>,
) -> Result<UTXOSet, NodoBitcoinError> {
    log_info_message(logger.clone(), "Actualizando UTXOS ...".to_string());
    let uxos_updated = match initialize_utxos_from_file(utxo_set.clone(), accounts.clone()) {
        Ok(uxtos) => uxtos,
        Err(_) => {
            log_error_message(logger.clone(), "Error al inicializar UTXOS".to_string());
            return Err(NodoBitcoinError::ErrorAlActualizarUTXOS);
        }
    };
    log_info_message(logger.clone(), "UTXOS actualizadas".to_string());
    Ok(uxos_updated)
}

fn send_new_tx(
    account: Account,
    target_address: String,
    target_amount: u64,
    fee: u64,
    utxo_set: UTXOSet,
    admin_connections: Option<AdminConnections>,
    logger: Sender<LogMessages>,
) -> Result<(), NodoBitcoinError> {
    // obtener UTXOS del account
    let public_key = account.public_key.clone();
    let utxos_by_account = match utxo_set.utxos_for_account.get(&public_key) {
        Some(utxos) => utxos.clone(),
        None => return Err(NodoBitcoinError::CuentaNoEncontrada),
    };

    let tx_obj = create_tx_to_send(
        account,
        target_address,
        target_amount,
        fee,
        utxos_by_account,
    )?;

    let admin_connections = match admin_connections {
        Some(admin_connections) => admin_connections,
        None => return Err(NodoBitcoinError::NoSePuedeEnviarTransaccion),
    };

    send_tx(admin_connections, logger, tx_obj)?;

    Ok(())
}

pub fn create_transaction_manager(
    accounts: Vec<Account>,
    app_sender: Sender<ApplicationManagerMessages>,
) -> Sender<TransactionMessages> {
    let (sender, receiver) = channel();

    let transaction_manager = Arc::new(Mutex::new(TransactionManager {
        utxos: UTXOSet::new(),
        tx_pendings: HashMap::new(),
        accounts,
        sender_block_broadcasting: None,
        sender_app_manager: app_sender,
        admin_connections: None,
    }));

    thread::spawn(move || {
        let tm = transaction_manager.clone();
        while let Ok(message) = receiver.recv() {
            let mut manager = tm.lock().unwrap();
            manager.handle_message(message);
        }
    });

    sender
}

fn initialize_utxos_from_file(
    mut utxo_set: UTXOSet,
    accounts: Vec<Account>,
) -> Result<UTXOSet, NodoBitcoinError> {
    let blocks = SerializedBlock::read_blocks_from_file()?;
    // filtrar los bloxks por sólo aquellos que tiene transacciones
    let blocks_with_tx = blocks
        .into_iter()
        .filter(|block| block.txns.len() > 0)
        .collect::<Vec<SerializedBlock>>();

    println!("blocks with tx {:?}", blocks_with_tx.len());
    utxo_set.update_from_blocks(blocks_with_tx, accounts.clone())?;
    Ok(utxo_set)
}

pub fn _update_from_transactions(
    logger: Sender<LogMessages>,
    manager: Sender<TransactionMessages>,
    blocks: Vec<SerializedBlock>,
    accounts: Vec<Account>,
) -> Result<(), NodoBitcoinError> {
    let (sender, receiver) = channel();
    manager.send(TransactionMessages::_UpdateFromBlocks((
        blocks, accounts, sender,
    )));

    match receiver.recv() {
        Ok(result) => result,
        Err(_) => {
            // todo log error
            // handle error
            log_error_message(logger, "".to_string());
            Err(NodoBitcoinError::InvalidAccount)
        }
    }
}
