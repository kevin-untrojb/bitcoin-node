use std::sync::mpsc::{channel, Sender};
use std::thread;
use std::sync::Arc;
use std::sync::Mutex;

use crate::log::{log_error_message,LogMessages};
use crate::wallet::uxto_set::{UtxoSetTrait,Utxo,UTXOSet};
use crate::blockchain::transaction::{Transaction, TxOut};
use crate::common::uint256::Uint256;
use crate::errores::NodoBitcoinError;
use std::collections::HashMap;

pub struct TransactionManager {
    uxtos: dyn UtxoSetTrait
}

enum TransactionMessages {
    GetAvailable(( &str ,Sender<Result<(), NodoBitcoinError>>)),
    UpdateFromTransactions((Vec<Transaction>,Vec<String>, Sender<Result<(), NodoBitcoinError>>)),
    ShutDown,
}

impl TransactionManager {
    fn handle_message(&mut self, message: TransactionMessages) {
        match message {
            TransactionMessages::GetAvailable((account, result)) => {
                result.send(self.uxtos.get_available(account));
            }
            TransactionMessages::UpdateFromTransactions((transactions,accounts, result))=>{
                result.send(self.uxtos.update_from_transactions(transactions,accounts));
            }
            TransactionMessages::ShutDown => {
                return
            }
        }
    }
}

pub fn create_transaction_manager() -> Sender<TransactionMessages> {
    let (sender, receiver) = channel();

    let transaction_manager = Arc::new(Mutex::new(TransactionManager { uxtos: UTXOSet::new() }));

    thread::spawn(move || {
        let tm = transaction_manager.clone();
        while let Ok(message) = receiver.recv() {
            let mut manager = tm.lock().unwrap();
            manager.handle_message(message);
        }
    });

    sender
}

pub fn update_from_transactions(logger: Sender<LogMessages>, manager: Sender<TransactionMessages>, transactions:Vec<Transaction>,accounts:Vec<String>) -> Result<(), NodoBitcoinError> {
    let (sender, receiver)= channel();
    manager.send(TransactionMessages::UpdateFromTransactions((transactions,accounts,sender)));

    match receiver.recv() {
        Ok(result) => match result {
            Ok(()) => {
                result
            }
            Err(error) => {
                error
            }
        }
        Err(_) => {
            // todo log error
            // handle error
            log_error_message(logger,"");
            Err(NodoBitcoinError::InvalidAccount)
        }
    }

}
pub fn get_available(logger: Sender<LogMessages>, manager: Sender<TransactionMessages>,account: &str) ->Result<u64, NodoBitcoinError> {
    let (sender, receiver)= channel();
    manager.send(TransactionMessages::GetAvailable((account,sender)));

    match receiver.recv() {
        Ok(result) => match result {
            Ok((balance)) => {
                OK(balance)
            }
            Err(error) => {
                Err(error)
            }
        }
        Err(_) => {
            // todo log error
            // handle error
            log_error_message(logger,"");
            Err(NodoBitcoinError::InvalidAccount)
        }
    }
}
