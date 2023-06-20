use std::{
    sync::mpsc::{self, channel},
    thread::{self, sleep},
    time::Duration,
};

use crate::{
    blockchain::transaction::Transaction,
    config,
    errores::{InterfaceError, NodoBitcoinError},
    interface::{
        public::{end_loading, start_loading},
        view::ViewObject,
    },
    log::{create_logger_actor, log_info_message, LogMessages},
    protocol::{
        admin_connections::{self, AdminConnections},
        connection::connect,
        initial_block_download::get_full_blockchain,
    },
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

pub enum ApplicationManagerMessages {
    ShutDowned,
}

impl ApplicationManager {
    pub fn new(sender: glib::Sender<ViewObject>) -> Self {
        let accounts = match Account::get_all_accounts() {
            Ok(accounts) => accounts,
            Err(_) => Vec::new(),
        };
        let tx_manager = create_transaction_manager(accounts.clone());
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

    pub fn send_transaction(
        &self,
        target_address: String,
        target_amount_string: String,
        fee_string: String,
    ) {
        let target_amount = match target_amount_string.parse::<f64>() {
            Ok(target_amount) => (target_amount * 100_000_000.0) as u64,
            Err(_) => {
                _ = self
                    .sender_frontend
                    .send(ViewObject::Error(InterfaceError::TargetAmountNotValid));
                return;
            }
        };
        let fee: u64 = match fee_string.parse::<f64>() {
            Ok(fee) => (fee * 100_000_000.0) as u64,
            Err(_) => {
                _ = self
                    .sender_frontend
                    .send(ViewObject::Error(InterfaceError::FeeNotValid));
                return;
            }
        };
        let message = format!(
            "Transacción enviada a {:?}. Monto: {:?}. Fee: {:?}",
            target_address, target_amount, fee
        );
        log_info_message(self.logger.clone(), message);
    }

    pub fn close(&self) {
        // TODO: cerrar los threads abiertos
        start_loading(
            self.sender_frontend.clone(),
            "Closing threads... ".to_string(),
        );

        log_info_message(self.logger.clone(), "Cerrando aplicación...".to_string());
        println!("Close");
        _ = Account::save_all_accounts(self.accounts.clone());

        // cerrar todos los threads abiertos
        let (sender, receiver) = channel();
        _ = self.tx_manager.send(TransactionMessages::ShutDown(sender));

        while let Ok(message) = receiver.recv() {
            match message {
                ApplicationManagerMessages::ShutDowned => {
                    break;
                }
            }
        }

        log_info_message(
            self.logger.clone(),
            "Aplicación cerrada exitosamente.".to_string(),
        );
        end_loading(self.sender_frontend.clone());
    }

    fn get_available_amount(&self) -> Result<u64, NodoBitcoinError> {
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
        let sender_tx_manager = self.tx_manager.clone();
        thread::spawn(move || {
            let admin_connections = match ApplicationManager::download_blockchain(
                sender_frontend.clone(),
                logger.clone(),
            ) {
                Ok(admin_connections) => admin_connections,
                Err(_) => {
                    start_loading(
                        sender_frontend,
                        "Error al descargar la blockchain".to_string(),
                    );
                    return;
                }
            };

            let _ = sender_tx_manager.send(TransactionMessages::InitBlockBroadcasting((
                admin_connections,
                logger,
                sender_tx_manager.clone(),
            )));
        });
    }

    fn download_blockchain(
        sender_frontend: glib::Sender<ViewObject>,
        logger: mpsc::Sender<LogMessages>,
    ) -> Result<AdminConnections, NodoBitcoinError> {
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
        Ok(admin_connections)
    }

    pub fn create_account(&mut self, secret_key: String, address: String, name: String) -> Account {
        let new_account = Account::new(secret_key, address, name);

        let is_valid =
            ApplicationManager::account_validator(new_account.clone(), self.accounts.clone());
        if !is_valid {
            _ = self
                .sender_frontend
                .send(ViewObject::Error(InterfaceError::CreateAccount));
        }

        self.accounts.push(new_account.clone());
        // avisarle al tx_manager que se acaba de crear una cuenta

        new_account.clone()
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
        let _ = self
            .sender_frontend
            .send(ViewObject::UploadTransactions(txs_current_account));

        // pedirle al tx manager los saldos de las utxos del nuevo account seleccionado y con eso llamar a la vista
        // tambien devolver los pending
        let available_amount = self.get_available_amount()?;
        let pending_amount: u64 = 500000;
        let _ = self.sender_frontend.send(ViewObject::UploadAmounts((
            available_amount,
            pending_amount,
        )));

        Ok(())
    }
}
