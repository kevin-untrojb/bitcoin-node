use std::{sync::mpsc, thread};

use crate::{
    blockchain::transaction::Transaction,
    config,
    errores::NodoBitcoinError,
    interface::view::{end_loading, start_loading, ViewObject},
    log::{create_logger_actor, LogMessages},
    protocol::{connection::connect, initial_block_download::get_full_blockchain},
    wallet::user::Account,
};

#[derive(Clone)]
pub struct ApplicationManager {
    pub current_account: Option<Account>,
    pub accounts: Vec<Account>,
    sender_frontend: glib::Sender<ViewObject>,
    logger: mpsc::Sender<LogMessages>,
}

impl ApplicationManager {
    pub fn new(sender: glib::Sender<ViewObject>) -> Self {
        let accounts = match Account::get_all_accounts() {
            Ok(accounts) => accounts,
            Err(_) => Vec::new(),
        };
        let logger = create_logger_actor(config::get_valor("LOG_FILE".to_string()));
        let mut app_manager = ApplicationManager {
            current_account: None,
            accounts,
            sender_frontend: sender,
            logger: logger.clone(),
        };
        app_manager.thread_download_blockchain();
        app_manager
    }

    pub fn close_threads(&mut self) {
        // TODO: cerrar los threads abiertos
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
}
