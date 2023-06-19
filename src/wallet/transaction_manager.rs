use std::sync::mpsc::{channel, Sender};
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

use crate::blockchain::block::SerializedBlock;
use crate::blockchain::transaction::Transaction;
use crate::errores::NodoBitcoinError;
use crate::log::{log_error_message, LogMessages};
use crate::protocol::admin_connections::{self, AdminConnections};
use crate::protocol::block_broadcasting::init_block_broadcasting;
use crate::wallet::uxto_set::UTXOSet;

use super::user::Account;

#[derive(Clone)]
pub struct TransactionManager {
    uxtos: UTXOSet,
    tx_pendings: Vec<Transaction>,
    accounts: Vec<Account>,
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
    NewBlock(SerializedBlock),
    NewTx(Transaction),
    ShutDown,
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
            TransactionMessages::ShutDown => return,
        }
    }

    fn update_pendings(&mut self, new_tx: Transaction) {
        self.tx_pendings.retain(|tx| tx.txid() != new_tx.txid());
    }
}

pub fn create_transaction_manager(accounts: Vec<Account>) -> Sender<TransactionMessages> {
    let (sender, receiver) = channel();

    let transaction_manager = Arc::new(Mutex::new(TransactionManager {
        uxtos: UTXOSet::new(),
        tx_pendings: Vec::new(),
        accounts,
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
