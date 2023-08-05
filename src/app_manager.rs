use std::{
    sync::{
        mpsc::{self, channel, Sender},
        Arc, Mutex,
    },
    thread::{self},
};

use crate::{
    blockchain::block::{pow_poi_validation, SerializedBlock},
    blockchain::{
        file::header_count,
        file_manager::{FileManager, FileMessages},
    },
    common::utils_data::total_reintentos,
    config,
    errores::{InterfaceError, InterfaceMessage, NodoBitcoinError},
    interface::{
        public::{end_loading, show_message, start_loading},
        view::ViewObject,
    },
    log::{create_logger_actor, log_error_message, log_info_message, LogMessages},
    protocol::{
        admin_connections::AdminConnections, connection::connect,
        initial_block_download::get_full_blockchain,
    },
    wallet::{
        transaction_manager::{create_transaction_manager, TransactionMessages},
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
    sender_app_manager: Sender<ApplicationManagerMessages>,
    file_manager: Sender<FileMessages>,
    shutdown_sent: bool,
}

pub enum ApplicationManagerMessages {
    GetAmountsByAccount(u64, i128),
    GetTxReportByAccount(Vec<TxReport>),
    ShutDowned,
    ShutDown,
    TransactionManagerUpdate,
    _NewBlock,
    _NewTx,
    BlockBroadcastingError,
    _NodoServerError,
    InitialDownloadError(usize),
    ApplicationError(String),
    UpdateProgressBar(usize, usize),
    POIInvalido,
}

impl ApplicationManager {
    pub fn new(sender_frontend: glib::Sender<ViewObject>) -> Self {
        let accounts = match Account::get_all_accounts() {
            Ok(accounts) => accounts,
            Err(_) => Vec::new(),
        };
        let (sender_app_manager, receiver_app_manager) = channel();
        let logger = create_logger_actor(config::get_valor("LOG_FILE".to_string()));

        let file_manager = FileManager::new(logger.clone());
        let tx_manager = create_transaction_manager(
            accounts.clone(),
            logger.clone(),
            sender_app_manager.clone(),
            file_manager.clone(),
        );
        _ = tx_manager.send(TransactionMessages::LoadSavedUTXOS);
        let mut app_manager = ApplicationManager {
            current_account: None,
            sender_app_manager,
            accounts,
            sender_frontend,
            logger,
            tx_manager,
            file_manager,
            shutdown_sent: false,
        };
        app_manager.thread_download_blockchain(0);
        let ret_value = app_manager.clone();

        let app_manager_mutex = Arc::new(Mutex::new(app_manager));
        thread::spawn(move || {
            while let Ok(message) = receiver_app_manager.recv() {
                let mut manager = match app_manager_mutex.lock() {
                    Ok(manager) => manager,
                    Err(_) => {
                        println!("Error al obtener el lock del appmanager");
                        continue;
                    }
                };
                manager.handle_message(message);
            }
        });

        ret_value
    }
    fn handle_message(&mut self, message: ApplicationManagerMessages) {
        match message {
            ApplicationManagerMessages::TransactionManagerUpdate => {
                _ = self.send_messages_to_get_values();
                _ = self
                    .sender_app_manager
                    .send(ApplicationManagerMessages::UpdateProgressBar(0, 0));
            }
            ApplicationManagerMessages::GetAmountsByAccount(available_amount, pending_amount) => {
                //println!("pending_amount: {:?}", pending_amount);
                let _ = self.sender_frontend.send(ViewObject::UploadAmounts((
                    available_amount,
                    pending_amount,
                )));
                end_loading(self.sender_frontend.clone());
            }
            ApplicationManagerMessages::GetTxReportByAccount(tx_reports) => {
                let _ = self
                    .sender_frontend
                    .send(ViewObject::UploadTransactions(tx_reports));
                end_loading(self.sender_frontend.clone());
            }
            ApplicationManagerMessages::ShutDown => {
                self.shutdown_sent = true;
                _ = self.tx_manager.send(TransactionMessages::ShutDown);
            }
            ApplicationManagerMessages::ShutDowned => {
                if !self.shutdown_sent {
                    // no se envió ningun shutdown, hay que reiniciar las conexiones
                    let sender_tx_manager = self.tx_manager.clone();
                    let logger = self.logger.clone();
                    let admin_connections = match connect(logger.clone()) {
                        Ok(admin_connections) => admin_connections,
                        Err(_) => {
                            let _ = self
                                .sender_frontend
                                .send(ViewObject::Error(InterfaceError::BlockBroadcastingError));
                            return;
                        }
                    };
                    let _ = sender_tx_manager.send(TransactionMessages::InitBlockBroadcasting((
                        admin_connections,
                        logger,
                        sender_tx_manager.clone(),
                    )));
                    return;
                }
                log_info_message(
                    self.logger.clone(),
                    "Aplicación cerrada exitosamente.".to_string(),
                );
                let _ = self.sender_frontend.send(ViewObject::CloseApplication);
            }
            ApplicationManagerMessages::_NewBlock => {
                //let _ = self.sender_frontend.send(ViewObject::NewBlock("Nuevo bloque recibido".to_string()));
            }
            ApplicationManagerMessages::_NewTx => {
                //let _ = self.sender_frontend.send(ViewObject::NewTx("Nuevo transaccion recibido".to_string()));
            }
            ApplicationManagerMessages::InitialDownloadError(intento) => {
                let maximo_reintentos = total_reintentos() < intento;
                if maximo_reintentos {
                    log_error_message(
                        self.logger.clone(),
                        "Error al descargar la blockchain".to_string(),
                    );
                    show_message(
                        self.sender_frontend.clone(),
                        "Error al descargar la blockchain. Reintente más tarde. ".to_string(),
                    );
                } else {
                    show_message(
                        self.sender_frontend.clone(),
                        "Error al descargar la blockchain. Reintentando...".to_string(),
                    );
                    self.thread_download_blockchain(intento + 1);
                }
            }
            ApplicationManagerMessages::BlockBroadcastingError => {
                let _ = self
                    .sender_frontend
                    .send(ViewObject::Error(InterfaceError::BlockBroadcastingError));
            }
            ApplicationManagerMessages::_NodoServerError => {
                let _ = self
                    .sender_frontend
                    .send(ViewObject::Error(InterfaceError::NodoServerError));
            }
            ApplicationManagerMessages::ApplicationError(message) => {
                show_message(self.sender_frontend.clone(), message);
            }
            ApplicationManagerMessages::UpdateProgressBar(n_blocks, n_downloaded_blocks) => {
                let n_headers = match header_count() {
                    Ok(n_headers) => n_headers as usize,
                    Err(_) => 0,
                };
                let _ = self.sender_frontend.send(ViewObject::UploadProgressBar((
                    n_headers,
                    n_blocks,
                    n_downloaded_blocks,
                )));
            }
            ApplicationManagerMessages::POIInvalido => {
                log_info_message(
                    self.logger.clone(),
                    "Descargando nuevo bloque luego del POI inválido...".to_string(),
                );
                let sender_app_manager = self.sender_app_manager.clone();
                let last_block = match ApplicationManager::download_blockchain(
                    self.sender_frontend.clone(),
                    sender_app_manager,
                    self.logger.clone(),
                ) {
                    Ok(_) => match SerializedBlock::read_last_block_from_file() {
                        Ok(last_block) => last_block,
                        Err(_) => {
                            show_message(
                                self.sender_frontend.clone(),
                                "Error al obtener el último bloque descargado".to_string(),
                            );
                            return;
                        }
                    },
                    Err(_) => {
                        show_message(
                            self.sender_frontend.clone(),
                            "Error al obtener el último bloque descargado".to_string(),
                        );
                        return;
                    }
                };

                if !pow_poi_validation(self.logger.clone(), last_block.clone()) {
                    _ = self
                        .sender_app_manager
                        .send(ApplicationManagerMessages::POIInvalido);
                } else {
                    _ = self
                        .tx_manager
                        .send(TransactionMessages::NewBlock(last_block));
                    end_loading(self.sender_frontend.clone());
                }
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
            "Enviando tx desde {:?} a {:?}. Monto: {:?}. Fee: {:?} ...",
            account.public_key, target_address, target_amount, fee
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
        start_loading(
            self.sender_frontend.clone(),
            "Closing application... ".to_string(),
        );

        log_info_message(self.logger.clone(), "Cerrando aplicación...".to_string());
        _ = Account::save_all_accounts(self.accounts.clone());
        _ = self
            .sender_app_manager
            .send(ApplicationManagerMessages::ShutDown);

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

    fn thread_download_blockchain(&mut self, intento: usize) {
        let logger = self.logger.clone();
        let sender_frontend = self.sender_frontend.clone();
        let sender_tx_manager = self.tx_manager.clone();
        let sender_app_manager = self.sender_app_manager.clone();
        thread::spawn(move || {
            let admin_connections = match ApplicationManager::download_blockchain(
                sender_frontend.clone(),
                sender_app_manager.clone(),
                logger.clone(),
            ) {
                Ok(admin_connections) => admin_connections,
                Err(_) => {
                    _ = sender_app_manager
                        .send(ApplicationManagerMessages::InitialDownloadError(intento));
                    return;
                }
            };

            let _ = sender_tx_manager.send(TransactionMessages::InitBlockBroadcasting((
                admin_connections,
                logger,
                sender_tx_manager.clone(),
            )));

            let _ = sender_tx_manager.send(TransactionMessages::InitServerNode(
                sender_tx_manager.clone(),
            ));
        });
    }

    fn download_blockchain(
        sender_frontend: glib::Sender<ViewObject>,
        sender_app_manager: Sender<ApplicationManagerMessages>,
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
        get_full_blockchain(
            logger,
            admin_connections.clone(),
            sender_app_manager.clone(),
        )?;
        end_loading(sender_frontend);
        let _ = sender_app_manager.send(ApplicationManagerMessages::UpdateProgressBar(0, 0));
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

        new_account
    }

    fn account_validator(new_account: Account, accounts: Vec<Account>) -> bool {
        for account in accounts.iter() {
            if account.wallet_name == new_account.wallet_name {
                return false;
            }
        }
        true
    }

    fn send_messages_to_get_values(&self) -> Result<(), NodoBitcoinError> {
        let current_account_ok = match self.current_account.clone() {
            Some(account) => account,
            None => return Err(NodoBitcoinError::InvalidAccount),
        };
        let manager = self.tx_manager.clone();
        match manager.send(TransactionMessages::GetTxReportByAccount(
            current_account_ok.public_key.clone(),
        )) {
            Ok(_) => {}
            Err(_) => {
                show_message(
                    self.sender_frontend.clone(),
                    "Updating wallet tx error".to_string(),
                );
                return Err(NodoBitcoinError::InvalidAccount);
            }
        }
        match manager.send(TransactionMessages::GetAvailableAndPending(
            current_account_ok.public_key,
        )) {
            Ok(_) => {}
            Err(_) => {
                show_message(
                    self.sender_frontend.clone(),
                    "Updating wallet data error".to_string(),
                );
                return Err(NodoBitcoinError::InvalidAccount);
            }
        }
        start_loading(
            self.sender_frontend.clone(),
            "Updating wallet data ... ".to_string(),
        );
        Ok(())
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
        self.send_messages_to_get_values()
    }
}
