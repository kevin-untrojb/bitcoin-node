use std::{
    sync::{
        mpsc::{self, Sender, channel},
        Arc, Mutex,
    },
    thread::{self},
};

use crate::{
    blockchain::transaction::Transaction,
    config,
    errores::{InterfaceError, NodoBitcoinError, InterfaceMessage},
    interface::{
        public::{end_loading, start_loading},
        view::ViewObject,
    },
    log::{create_logger_actor, log_info_message,log_error_message, LogMessages},
    protocol::{
        admin_connections::AdminConnections, connection::connect,
        initial_block_download::get_full_blockchain,
    },
    wallet::{
        transaction_manager::{
            create_transaction_manager, get_available, get_txs_by_account, TransactionMessages,
        },
        user::Account,
        uxto_set::TxReport,
    },
};

#[derive(Clone)]
pub struct ApplicationManager {
    pub current_account: Option<Account>,
    pub accounts: Vec<Account>,
    pub tx_manager: mpsc::Sender<TransactionMessages>,
    sender_frontend: glib::Sender<ViewObject>,
    logger: mpsc::Sender<LogMessages>,
    sender_app_manager: Sender<ApplicationManagerMessages>
}

pub enum ApplicationManagerMessages {
    ShutDowned(Sender<Result<(), NodoBitcoinError>>),
    TransactionManagerUpdate,
    NewBlock,
    NewTx
}

impl ApplicationManager {
    pub fn new(sender_frontend: glib::Sender<ViewObject>) -> Self {
        let accounts = match Account::get_all_accounts() {
            Ok(accounts) => accounts,
            Err(_) => Vec::new(),
        };
        let (sender_app_manager, receiver_app_manager) = channel();
        let tx_manager = create_transaction_manager(accounts.clone(), sender_app_manager.clone());
        let logger = create_logger_actor(config::get_valor("LOG_FILE".to_string()));
        let mut app_manager = ApplicationManager {
            current_account: None,
            sender_app_manager,
            accounts,
            sender_frontend,
            logger,
            tx_manager,
        };
        app_manager.thread_download_blockchain();
        let ret_value = app_manager.clone();

        let app_manager_mutex = Arc::new(Mutex::new(app_manager));
        thread::spawn(move || {
            let ap = app_manager_mutex.clone();
            while let Ok(message) = receiver_app_manager.recv() {
                let mut manager = ap.lock().unwrap();
                manager.handle_message(message);
            }
        });

        ret_value
    }
    fn handle_message(&mut self, message: ApplicationManagerMessages) {
        match message {
            ApplicationManagerMessages::ShutDowned(shut_down_sender) => {
                _ = self
                    .tx_manager
                    .send(TransactionMessages::ShutDown);
                shut_down_sender.send(Ok(()));
                return;
            }
            ApplicationManagerMessages::TransactionManagerUpdate => {
                let txs_current_account = match self.get_txs_by_account() {
                    Ok(txs) => txs,
                    Err(_) => {
                        return;
                    }
                };

                let _ = self
                    .sender_frontend
                    .send(ViewObject::UploadTransactions(txs_current_account));
            }
            ApplicationManagerMessages::NewTx => {
                let _ = self
                    .sender_frontend
                    .send(ViewObject::NewTx("Nueva transaccion recibida. Podras ver mas detalles en la pestaña 'Transactions'.".to_string()));
            }
            ApplicationManagerMessages::NewBlock => {
                let _ = self
                    .sender_frontend
                    .send(ViewObject::NewBlock("Nuevo bloque recibido.".to_string()));
            }
        }
    }

    pub fn send_transaction(
        &self,
        target_address: String,
        target_amount_string: String,
        fee_string: String,
    ) -> Result<(), NodoBitcoinError> {
        let target_amount = match target_amount_string.parse::<f64>() {
            Ok(target_amount) => (target_amount * 100_000_000.0) as u64,
            Err(_) => {
                _ = self
                    .sender_frontend
                    .send(ViewObject::Error(InterfaceError::TargetAmountNotValid));
                return Err(NodoBitcoinError::NoSePuedeEnviarTransaccion);
            }
        };
        let fee: u64 = match fee_string.parse::<f64>() {
            Ok(fee) => (fee * 100_000_000.0) as u64,
            Err(_) => {
                _ = self
                    .sender_frontend
                    .send(ViewObject::Error(InterfaceError::FeeNotValid));
                return Err(NodoBitcoinError::NoSePuedeEnviarTransaccion);
            }
        };

        let account = self.get_current_account()?;
        let logger = self.logger.clone();

        let message = format!(
            "Enviando tx a {:?}. Monto: {:?}. Fee: {:?} ...",
            target_address, target_amount, fee
        );
        log_info_message(self.logger.clone(), message);

        if self
            .tx_manager
            .send(TransactionMessages::SendTx(
                account,
                target_address,
                target_amount,
                fee,
                logger,
            ))
            .is_err()
        {
            _ = self
                    .sender_frontend
                    .send(ViewObject::Error(InterfaceError::TransactionNotSent));
            return Err(NodoBitcoinError::NoSePuedeEnviarTransaccion);
        }

        _ = self
        .sender_frontend
        .send(ViewObject::Message(InterfaceMessage::TransactionSent));
        Ok(())
    }

    pub fn close(&self) -> Result<(), NodoBitcoinError> {
        // TODO: cerrar los threads abiertos
        log_info_message(self.logger.clone(), "Cerrando aplicación...".to_string());
        println!("Close");
        _ = Account::save_all_accounts(self.accounts.clone());

        // cerrar todos los threads abiertos
        let (sender_shutdown, receiver_shutdown) = channel();
        self.sender_app_manager.send(ApplicationManagerMessages::ShutDowned(sender_shutdown));
        match receiver_shutdown.recv(){
            Ok(_) => {},
            Err(_) => {
                // todo log error
                // handle error
                log_error_message(self.logger.clone(), "".to_string());
                return Err(NodoBitcoinError::InvalidAccount);
            }
        }

        log_info_message(
            self.logger.clone(),
            "Aplicación cerrada exitosamente.".to_string(),
        );
        Ok(())
    }

    fn get_current_account(&self) -> Result<Account, NodoBitcoinError> {
        let option_current_account = self.current_account.clone();
        let current_account = match option_current_account {
            Some(account) => account,
            None => return Err(NodoBitcoinError::NoHayCuentaSeleccionada),
        };
        Ok(current_account)
    }

    fn get_available_amount(&self) -> Result<u64, NodoBitcoinError> {
        let current_account = self.get_current_account()?;
        let public_key = current_account.public_key.clone();
        let logger = self.logger.clone();
        let tx_manager = self.tx_manager.clone();

        get_available(logger, tx_manager, public_key.to_string())
    }

    fn get_txs_by_account(&self) -> Result<Vec<TxReport>, NodoBitcoinError> {
        let current_account = self.get_current_account()?;
        let public_key = current_account.public_key.clone();
        let logger = self.logger.clone();
        let tx_manager = self.tx_manager.clone();

        Ok(get_txs_by_account(
            logger,
            tx_manager,
            public_key.to_string(),
        ))
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

        let _ = self.tx_manager.send(TransactionMessages::AddAccount(
            self.accounts.clone(),
            self.logger.clone(),
        ));

        _ = self
        .sender_frontend
        .send(ViewObject::Message(InterfaceMessage::CreateAccount));

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
        let txs_current_account = self.get_txs_by_account()?;

        println!("txs_current_account: {:?}", txs_current_account);

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
