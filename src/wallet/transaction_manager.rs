use std::sync::mpsc::{channel, Sender};
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

use crate::app_manager::ApplicationManagerMessages;
use crate::blockchain::block::SerializedBlock;
use crate::blockchain::transaction::{create_tx_to_send, Transaction};
use crate::errores::NodoBitcoinError;
use crate::log::{log_error_message, log_info_message, LogMessages};
use crate::protocol::admin_connections::{self, AdminConnections};
use crate::protocol::block_broadcasting::{init_block_broadcasting, BlockBroadcastingMessages};
use crate::protocol::send_tx::send_tx;
use crate::wallet::uxto_set::UTXOSet;

use super::user::Account;

#[derive(Clone)]
pub struct TransactionManager {
    pub uxtos: UTXOSet,
    tx_pendings: Vec<Transaction>,
    accounts: Vec<Account>,
    sender_app_manager: Option<Sender<ApplicationManagerMessages>>,
    sender_block_broadcasting: Option<Sender<BlockBroadcastingMessages>>,
    admin_connections: Option<AdminConnections>,
    // TODO guardar hilos abiertos para despues cerrarlos (block broadcasting)
}

pub enum TransactionMessages {
    GetAvailable((String, Sender<Result<u64, NodoBitcoinError>>)),
    UpdateFromTransactions(
        (
            Vec<Transaction>,
            Vec<Account>,
            Sender<Result<(), NodoBitcoinError>>,
        ),
    ),
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
    ShutDown(Sender<ApplicationManagerMessages>),
    Shutdowned(),
}

impl TransactionManager {
    fn handle_message(&mut self, message: TransactionMessages) {
        match message {
            TransactionMessages::GetAvailable((account, result)) => {
                result.send(self.uxtos.get_available(account));
            }
            TransactionMessages::UpdateFromTransactions((transactions, accounts, result)) => {
                result.send(self.uxtos.update_from_transactions(transactions, accounts));
            }
            TransactionMessages::InitBlockBroadcasting((
                admin_connections,
                logger,
                sender_tx_manager,
            )) => {
                log_info_message(logger.clone(), "Actualizando UTXOS ...".to_string());
                let uxos_updated =
                    match initialize_utxos_from_file(self.uxtos.clone(), self.accounts.clone()) {
                        Ok(uxtos) => uxtos,
                        Err(e) => {
                            log_error_message(
                                logger.clone(),
                                "Error al inicializar UTXOS".to_string(),
                            );
                            return;
                        }
                    };
                self.uxtos = uxos_updated;
                log_info_message(logger.clone(), "UTXOS actualizadas".to_string());
                self.admin_connections = Some(admin_connections.clone());
                log_info_message(logger.clone(), "Inicio del block broadcasting.".to_string());
                thread::spawn(move || {
                    init_block_broadcasting(logger, admin_connections, sender_tx_manager);
                });
            }
            TransactionMessages::NewBlock(block) => {
                let txns = block.txns.clone();
                self.uxtos
                    .update_from_transactions(txns.clone(), self.accounts.clone());
                for tx in txns {
                    self.update_pendings(tx);
                }
            }
            TransactionMessages::NewTx(tx) => {
                self.tx_pendings.push(tx);
            }
            TransactionMessages::SenderBlockBroadcasting(sender_block_broadcasting) => {
                self.sender_block_broadcasting = Some(sender_block_broadcasting);
            }
            TransactionMessages::SendTx(account, target_address, target_amount, fee, logger) => {
                let utxos = self.uxtos.clone();
                let admin_connections = self.admin_connections.clone();
                send_new_tx(
                    account,
                    target_address,
                    target_amount,
                    fee,
                    utxos,
                    admin_connections,
                    logger,
                );
            }
            TransactionMessages::ShutDown(sender_app_manager) => {
                self.sender_app_manager = Some(sender_app_manager.clone());
                match &self.sender_block_broadcasting {
                    Some(sender) => sender.send(BlockBroadcastingMessages::ShutDown),
                    None => {
                        sender_app_manager.send(ApplicationManagerMessages::ShutDowned);
                        return;
                    }
                };
            }
            TransactionMessages::Shutdowned() => {
                match &self.sender_app_manager {
                    Some(sender) => sender.send(ApplicationManagerMessages::ShutDowned),
                    None => return,
                };
            }
        }
    }

    fn update_pendings(&mut self, new_tx: Transaction) {
        self.tx_pendings.retain(|tx| tx.txid() != new_tx.txid());
    }
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

pub fn create_transaction_manager(accounts: Vec<Account>) -> Sender<TransactionMessages> {
    let (sender, receiver) = channel();

    let transaction_manager = Arc::new(Mutex::new(TransactionManager {
        uxtos: UTXOSet::new(),
        tx_pendings: Vec::new(),
        accounts,
        sender_block_broadcasting: None,
        sender_app_manager: None,
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
    let txns = blocks
        .iter()
        .flat_map(|bloque| bloque.txns.clone())
        .collect::<Vec<_>>();

    utxo_set.update_from_transactions(txns, accounts.clone())?;
    Ok(utxo_set)
}

pub fn update_from_transactions(
    logger: Sender<LogMessages>,
    manager: Sender<TransactionMessages>,
    transactions: Vec<Transaction>,
    accounts: Vec<Account>,
) -> Result<(), NodoBitcoinError> {
    let (sender, receiver) = channel();
    manager.send(TransactionMessages::UpdateFromTransactions((
        transactions,
        accounts,
        sender,
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
pub fn get_available(
    logger: Sender<LogMessages>,
    manager: Sender<TransactionMessages>,
    account: String,
) -> Result<u64, NodoBitcoinError> {
    let (sender, receiver) = channel();
    manager.send(TransactionMessages::GetAvailable((account, sender)));
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
