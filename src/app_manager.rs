use std::{sync::mpsc, thread};

use crate::{
    blockchain::{block::SerializedBlock, transaction::Transaction},
    config,
    errores::NodoBitcoinError,
    interface::view::{end_loading, start_loading, ViewObject},
    log::{create_logger_actor, LogMessages},
    protocol::{
        block_broadcasting::init_block_broadcasting, connection::connect,
        initial_block_download::get_full_blockchain,
    },
    wallet::{user::Account, uxto_set::UTXOSet},
};

#[derive(Clone)]
pub struct ApplicationManager {
    pub current_account: Option<Account>,
    blockchain: Vec<SerializedBlock>,
    utxo: UTXOSet,
    pending: Vec<Transaction>,
    sender_frontend: glib::Sender<ViewObject>,
    logger: mpsc::Sender<LogMessages>,
}

impl ApplicationManager {
    pub fn new(sender: glib::Sender<ViewObject>) -> Self {
        let current_account = None;
        let blockchain = Vec::new();
        let utxo = UTXOSet::new();
        let pending = Vec::new();
        let logger = create_logger_actor(config::get_valor("LOG_FILE".to_string()));
        let app_manager = ApplicationManager {
            current_account,
            blockchain,
            utxo,
            pending,
            sender_frontend: sender,
            logger,
        };
        let app_manager_clone = app_manager.clone();
        // crear hilo + channels para descargar blockchain
        thread::spawn(move || {
            _ = app_manager_clone.download_blockchain();
            // TODO: si pincha, avisarle al front que pinchó
        });
        app_manager
    }

    pub fn set_current_account(&mut self, account: Account) {
        self.current_account = Some(account);
    }

    pub fn get_current_account(&self) -> Option<&Account> {
        self.current_account.as_ref()
    }

    // descargar blockchain
    fn download_blockchain(&self) -> Result<(), NodoBitcoinError> {
        start_loading(
            self.sender_frontend.clone(),
            "Connecting to peers... ".to_string(),
        );
        let admin_connections = connect(self.logger.clone())?;
        end_loading(self.sender_frontend.clone());
        start_loading(
            self.sender_frontend.clone(),
            "Obteniendo blockchain... ".to_string(),
        );
        get_full_blockchain(self.logger.clone(), admin_connections.clone())?;
        end_loading(self.sender_frontend.clone());
        Ok(())
    }
}
