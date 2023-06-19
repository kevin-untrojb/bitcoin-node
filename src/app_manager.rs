use std::{
    sync::mpsc::{self},
    thread,
};

use crate::{
    blockchain::transaction::Transaction,
    config,
    errores::NodoBitcoinError,
    interface::view::{end_loading, start_loading, ViewObject},
    log::{create_logger_actor, LogMessages},
    protocol::{connection::connect, initial_block_download::get_full_blockchain},
    wallet::{
        transaction_manager::{create_transaction_manager, get_available, TransactionMessages},
        user::Account,
    },
};

#[derive(Clone)]
pub struct ApplicationManager {
    pub current_account: Option<Account>,
    pub accounts: Vec<Account>,
    pub tx_manager: mpsc::Sender<TransactionMessages>,
    sender_frontend: glib::Sender<ViewObject>,
    logger: mpsc::Sender<LogMessages>,
}

impl ApplicationManager {
    pub fn new(sender: glib::Sender<ViewObject>) -> Self {
        let accounts = match Account::get_all_accounts() {
            Ok(accounts) => accounts,
            Err(_) => Vec::new(),
        };
        let tx_manager = create_transaction_manager();
        let logger = create_logger_actor(config::get_valor("LOG_FILE".to_string()));
        let mut app_manager = ApplicationManager {
            current_account: None,
            accounts,
            sender_frontend: sender,
            logger: logger.clone(),
            tx_manager,
        };
        app_manager.thread_download_blockchain();
        app_manager
    }

    pub fn close(&self) {
        // TODO: cerrar los threads abiertos
        println!("Close");
        _ = Account::save_all_accounts(self.accounts.clone());
        _ = self.tx_manager.send(TransactionMessages::ShutDown);
    }

    pub fn get_available_amount(&self) -> Result<u64, NodoBitcoinError> {
        let option_current_account = self.current_account.clone();
        let current_account = match option_current_account {
            Some(account) => account,
            None => return Err(NodoBitcoinError::NoHayCuentaSeleccionada),
        };
        let public_key = current_account.public_key.clone();
        let logger = self.logger.clone();
        let tx_manager = self.tx_manager.clone();

        get_available(logger, tx_manager, public_key.to_string())
    }

    fn thread_download_blockchain(&mut self) {
        let logger = self.logger.clone();
        let sender_frontend = self.sender_frontend.clone();
        thread::spawn(move || {
            let downloaded =
                ApplicationManager::download_blockchain(sender_frontend.clone(), logger);
            if downloaded.is_err() {
                start_loading(
                    sender_frontend,
                    "Error al descargar la blockchain".to_string(),
                );
            }
        });
    }

    fn download_blockchain(
        sender_frontend: glib::Sender<ViewObject>,
        logger: mpsc::Sender<LogMessages>,
    ) -> Result<(), NodoBitcoinError> {
        start_loading(
            sender_frontend.clone(),
            "Connecting to peers... ".to_string(),
        );
        let admin_connections = connect(logger.clone())?;
        end_loading(sender_frontend.clone());
        start_loading(
            sender_frontend.clone(),
            "Obteniendo blockchain... ".to_string(),
        );
        get_full_blockchain(logger.clone(), admin_connections.clone())?;
        end_loading(sender_frontend.clone());
        Ok(())
    }

    pub fn create_account(
        &mut self,
        key: String,
        address: String,
        name: String,
    ) -> Result<(), NodoBitcoinError> {
        println!("Create account!!!!!!");
        let new_account = Account::new(key, address, name);

        let is_valid =
            ApplicationManager::account_validator(new_account.clone(), self.accounts.clone());
        if !is_valid {
            return Err(NodoBitcoinError::ErrorAlCrearLaCuenta);
        }

        self.accounts.push(new_account.clone());

        // let _ = self
        //     .sender_frontend
        //     .send(ViewObject::NewAccount(new_account));

        Ok(())
    }

    fn account_validator(new_account: Account, accounts: Vec<Account>) -> bool {
        for account in accounts.iter() {
            if account.wallet_name == new_account.wallet_name {
                return false;
            }
        }
        return true;
    }

    pub fn select_current_account(&mut self, name: String) -> Result<(), NodoBitcoinError> {
        let accounts = self.accounts.clone();
        let mut current_account = None;
        for account in accounts.iter() {
            if account.wallet_name == name {
                current_account = Some(account.clone());
                continue;
            }
        }
        // cambio el current_account
        self.current_account = current_account;

        // llamar al tx_manager para que me devuelva un Vec<Transaction> y con eso llamar a la vista
        let txs_current_account = Vec::<Transaction>::new();

        // let _ = self
        //     .sender_frontend
        //     .send(ViewObject::NewAccount(new_account));
        let available_amount = self.get_available_amount()?;

        // pedirle al tx manager los saldos de las utxos del nuevo account seleccionado y con eso llamar a la vista
        // tambien devolver los pending

        // let _ = self
        //     .sender_frontend
        //     .send(ViewObject::NewAccount(new_account));

        let pending_amount: u64 = 0;

        Ok(())
    }
}
