use std::{
    sync::mpsc::{self},
    thread,
};

use crate::{
    config,
    errores::NodoBitcoinError,
    interface::view::{end_loading, start_loading, ViewObject},
    log::{create_logger_actor, LogMessages},
    protocol::{connection::connect, initial_block_download::get_full_blockchain, admin_connections::{AdminConnections, self}},
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

    pub fn close(&mut self) {
        // TODO: cerrar los threads abiertos
        _ = Account::save_all_accounts(self.accounts.clone());
        _ = self.tx_manager.send(TransactionMessages::ShutDown);
    }

    pub fn get_available_amount(&self) -> Result<u64, NodoBitcoinError> {
        let current_account = match &self.current_account {
            Some(account) => account,
            None => return Err(NodoBitcoinError::NoHayCuentaSeleccionada),
        };
        let public_key = &current_account.public_key;
        let logger = self.logger.clone();
        let tx_manager = self.tx_manager.clone();

        get_available(logger, tx_manager, public_key.to_string())
    }

    fn thread_download_blockchain(&mut self) {
        let logger = self.logger.clone();
        let sender_frontend = self.sender_frontend.clone();
        let sender_tx_manager = self.tx_manager.clone();
        thread::spawn(move || {
            let admin_connections = match ApplicationManager::download_blockchain(sender_frontend.clone(), logger.clone()){
                Ok(admin_connections) => admin_connections,
                Err(_) => {
                    start_loading(sender_frontend,"Error al descargar la blockchain".to_string());
                    return;
                }
            };

            let _ = sender_tx_manager.send(TransactionMessages::InitBlockBroadcasting((admin_connections, logger, sender_tx_manager.clone())));
            
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
}
