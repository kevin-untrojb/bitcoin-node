use std::collections::HashMap;
use std::sync::mpsc::{channel, Sender};
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::app_manager::ApplicationManagerMessages;
use crate::blockchain::block::SerializedBlock;
use crate::blockchain::transaction::{create_tx_to_send, Transaction};
use crate::common::uint256::Uint256;
use crate::errores::NodoBitcoinError;
use crate::log::{log_error_message, log_info_message, LogMessages};
use crate::protocol::admin_connections::AdminConnections;
use crate::protocol::block_broadcasting::{init_block_broadcasting, BlockBroadcastingMessages};
use crate::protocol::send_tx::send_tx;
use crate::wallet::uxto_set::{TxReport, UTXOSet};

use super::user::Account;

#[derive(Clone)]
pub struct TransactionManager {
    pub utxos: UTXOSet,
    tx_pendings: HashMap<Uint256, Transaction>,
    accounts: Vec<Account>,
    logger: Sender<LogMessages>,
    sender_app_manager: Sender<ApplicationManagerMessages>,
    sender_block_broadcasting: Option<Sender<BlockBroadcastingMessages>>,
    admin_connections: Option<AdminConnections>,
    // TODO guardar hilos abiertos para despues cerrarlos (block broadcasting)
}

pub enum TransactionMessages {
    GetAvailableAndPending(String),
    GetTxReportByAccount(String),
    _UpdateFromBlocks(
        (
            Vec<SerializedBlock>,
            Vec<Account>,
            Sender<Result<(), NodoBitcoinError>>,
        ),
    ),
    AddAccount(Vec<Account>, Sender<LogMessages>),
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
    LoadSavedUTXOS,
    ShutDown,
    Shutdowned,
}

// crear un type que cubra esta tupla (Account, u32, Uint256, bool, i128)
pub type PendingByAccount = (Account, u32, Uint256, bool, i128);

impl TransactionManager {
    fn handle_message(&mut self, message: TransactionMessages) {
        match message {
            TransactionMessages::GetAvailableAndPending(account) => {
                let available_amount = self.utxos.get_available(account.clone()).unwrap_or(0);
                let pending_amount = self.utxos.get_pending(account).unwrap_or(0);

                match self
                    .sender_app_manager
                    .send(ApplicationManagerMessages::GetAmountsByAccount(
                        available_amount,
                        pending_amount,
                    )) {
                    Ok(_) => {}
                    Err(_) => {
                        log_error_message(
                            self.logger.clone(),
                            "Error al enviar mensaje a ApplicationManagerMessages::GetAmountsByAccount".to_string(),
                        );
                        _ = self.sender_app_manager.send(
                            ApplicationManagerMessages::ApplicationError(
                                "Updating wallet data error".to_string(),
                            ),
                        );
                    }
                }
            }
            TransactionMessages::GetTxReportByAccount(account) => {
                let tx_by_account = match self.utxos.tx_report_by_accounts.get(&account) {
                    Some(tx) => tx.clone(),
                    None => Vec::new(),
                };
                let tx_pending_by_account =
                    match self.utxos.tx_report_pending_by_accounts.get(&account) {
                        Some(tx) => tx.clone(),
                        None => Vec::new(),
                    };
                // juntar los dos vectores
                let mut tx_by_account = tx_by_account;
                tx_by_account.extend(tx_pending_by_account);

                // ordernar por timestamp descendente
                tx_by_account.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

                match self.sender_app_manager.send(
                    ApplicationManagerMessages::GetTxReportByAccount(tx_by_account),
                ) {
                    Ok(_) => {}
                    Err(_) => {
                        log_error_message(
                            self.logger.clone(),
                            "Error al enviar mensaje a ApplicationManagerMessages::GetTxReportByAccount".to_string(),
                        );
                        _ = self.sender_app_manager.send(
                            ApplicationManagerMessages::ApplicationError(
                                "Updating wallet data error".to_string(),
                            ),
                        );
                    }
                }
            }
            TransactionMessages::_UpdateFromBlocks((blocks, accounts, result)) => {
                _ = result.send(self.utxos.update_from_blocks(blocks, accounts));
                _ = self
                    .sender_app_manager
                    .send(ApplicationManagerMessages::TransactionManagerUpdate);
            }
            TransactionMessages::AddAccount(accounts, logger) => {
                self.accounts = accounts;
                self.utxos.last_timestamp = 0;
                let utxos_updated = match update_utxos_from_file(
                    logger.clone(),
                    self.utxos.clone(),
                    self.accounts.clone(),
                ) {
                    Ok(uxtos) => uxtos,
                    Err(_) => {
                        log_error_message(logger, "Error al inicializar UTXOS".to_string());
                        return;
                    }
                };
                self.utxos = utxos_updated;
                log_info_message(logger, "UTXOS actualizadas".to_string());
                _ = self
                    .sender_app_manager
                    .send(ApplicationManagerMessages::TransactionManagerUpdate);
            }
            TransactionMessages::InitBlockBroadcasting((
                admin_connections,
                logger,
                sender_tx_manager,
            )) => {
                let utxos_updated = match update_utxos_from_file(
                    logger.clone(),
                    self.utxos.clone(),
                    self.accounts.clone(),
                ) {
                    Ok(uxtos) => uxtos,
                    Err(_) => {
                        log_error_message(logger, "Error al inicializar UTXOS".to_string());
                        return;
                    }
                };
                self.utxos = utxos_updated;
                log_info_message(logger.clone(), "UTXOS actualizadas".to_string());
                self.admin_connections = Some(admin_connections.clone());
                log_info_message(logger.clone(), "Inicio del block broadcasting.".to_string());
                let sender_app_manager_clone = self.sender_app_manager.clone();
                thread::spawn(move || {
                    match init_block_broadcasting(
                        logger.clone(),
                        admin_connections,
                        sender_tx_manager,
                    ) {
                        Ok(_) => todo!(),
                        Err(_) => {
                            log_error_message(
                                logger,
                                "Error al iniciar el block broadcasting.".to_string(),
                            );
                            _ = sender_app_manager_clone
                                .send(ApplicationManagerMessages::BlockBroadcastingError);
                        }
                    };
                });
                _ = self
                    .sender_app_manager
                    .send(ApplicationManagerMessages::TransactionManagerUpdate);
            }
            TransactionMessages::NewBlock(block) => {
                let txns = block.txns.clone();
                let _ = self
                    .utxos
                    .update_from_blocks(vec![block], self.accounts.clone());
                for tx in txns {
                    self.update_pendings(tx.txid().unwrap());
                }
                _ = self
                    .sender_app_manager
                    .send(ApplicationManagerMessages::TransactionManagerUpdate);
                _ = self
                    .sender_app_manager
                    .send(ApplicationManagerMessages::TransactionManagerUpdate);
            }
            TransactionMessages::NewTx(tx) => {
                let tx_id = match tx.txid() {
                    Ok(id) => id,
                    Err(_) => return,
                };
                if self.tx_pendings.contains_key(&tx_id) {
                    return;
                }
                let accounts_to_update = match self.validar_tx_propia(tx.clone()) {
                    Ok(accounts) => accounts,
                    Err(_) => vec![],
                };
                if !accounts_to_update.is_empty() {
                    for (account, index, txid, is_tx_in, value) in accounts_to_update.iter() {
                        // crear una TxReport
                        // agregarla al hashmap del utxoset
                        // enviar mensaje a la app manager

                        // obtener el unixtimestamp actual
                        let unix_timestamp: u32 = match SystemTime::now().duration_since(UNIX_EPOCH)
                        {
                            Ok(n) => n.as_secs() as u32,
                            Err(_) => 0,
                        };

                        let tx_report =
                            TxReport::new(true, unix_timestamp, *txid, *value, *is_tx_in, *index);
                        self.utxos
                            .tx_report_pending_by_accounts
                            .entry(account.public_key.clone())
                            .or_insert(Vec::new())
                            .push(tx_report.clone());
                    }
                }
                self.tx_pendings.insert(tx_id, tx);
                _ = self
                    .sender_app_manager
                    .send(ApplicationManagerMessages::TransactionManagerUpdate);
                _ = self
                    .sender_app_manager
                    .send(ApplicationManagerMessages::TransactionManagerUpdate);
            }
            TransactionMessages::SenderBlockBroadcasting(sender_block_broadcasting) => {
                self.sender_block_broadcasting = Some(sender_block_broadcasting);
            }
            TransactionMessages::SendTx(account, target_address, target_amount, fee, logger) => {
                let utxos = self.utxos.clone();
                let admin_connections = self.admin_connections.clone();
                let _ = send_new_tx(
                    account,
                    target_address,
                    target_amount,
                    fee,
                    utxos,
                    admin_connections,
                    logger,
                );
            }
            TransactionMessages::LoadSavedUTXOS => {
                // cargar los utxos guardados en el archivo
                let _ = self.utxos.load();
            }
            TransactionMessages::ShutDown => {
                // guardar utxos en archivo
                let _ = self.utxos.save();
                match &self.sender_block_broadcasting {
                    Some(sender) => {
                        _ = sender.send(BlockBroadcastingMessages::ShutDown);
                    }
                    None => {
                        _ = self
                            .sender_app_manager
                            .send(ApplicationManagerMessages::ShutDowned);
                    }
                };
            }
            TransactionMessages::Shutdowned => {
                _ = self
                    .sender_app_manager
                    .send(ApplicationManagerMessages::ShutDowned);
            }
        }
    }

    fn update_pendings(&mut self, tx_id: Uint256) {
        self.tx_pendings.remove(&tx_id);
    }

    fn validar_tx_propia(
        &self,
        tx: Transaction,
    ) -> Result<Vec<PendingByAccount>, NodoBitcoinError> {
        let logger = self.logger.clone();
        let tx_id = tx.txid()?;
        let accounts = self.accounts.clone();
        let utxo_set = self.utxos.clone();
        let mut accounts_tx = vec![];
        let mut accounts_index_is_in = vec![];
        for (index, tx_out) in tx.output.iter().enumerate() {
            let account_ok = UTXOSet::validar_output(accounts.clone(), tx_out);
            if let Ok(account_ok) = account_ok {
                accounts_tx.push(account_ok.clone());
                let item = (
                    account_ok.clone(),
                    index as u32,
                    tx_id,
                    false,
                    tx_out.value as i128,
                );
                accounts_index_is_in.push(item);
                let msg = format!(
                    "Tx {:?} from account: {:?} pending to be mined.",
                    tx.txid().unwrap().to_hexa_le_string(),
                    account_ok.public_key
                );
                log_info_message(logger.clone(), msg);
            }
        }
        for (index, tx_in) in tx.input.iter().enumerate() {
            let account_key_tx_in = utxo_set.validar_input(tx_in.clone());
            if let Ok(account_name) = account_key_tx_in {
                for account in accounts.iter() {
                    if account.public_key == account_name {
                        let previous_tx_id = Uint256::from_be_bytes(tx_in.previous_output.hash);
                        let output_index = tx_in.previous_output.index;
                        let utxos_for_account = utxo_set.utxos_for_account[&account_name].clone();
                        let mut value = 0;
                        for utxo in utxos_for_account.iter() {
                            if utxo.tx_id == previous_tx_id && utxo.output_index == output_index {
                                value = utxo.tx_out.value;
                            }
                        }

                        accounts_tx.push(account.clone());
                        let item = (account.clone(), index as u32, tx_id, true, -(value as i128));
                        accounts_index_is_in.push(item);
                        let msg = format!(
                            "Tx {:?} from account: {:?} pending to be mined.",
                            tx.txid().unwrap().to_hexa_le_string(),
                            account.public_key
                        );
                        log_info_message(logger.clone(), msg);
                    }
                }
            }
        }
        Ok(accounts_index_is_in)
    }
}

fn update_utxos_from_file(
    logger: Sender<LogMessages>,
    utxo_set: UTXOSet,
    accounts: Vec<Account>,
) -> Result<UTXOSet, NodoBitcoinError> {
    log_info_message(logger.clone(), "Actualizando UTXOS ...".to_string());
    let uxos_updated = match initialize_utxos_from_file(utxo_set, accounts) {
        Ok(uxtos) => uxtos,
        Err(_) => {
            log_error_message(logger, "Error al inicializar UTXOS".to_string());
            return Err(NodoBitcoinError::ErrorAlActualizarUTXOS);
        }
    };
    Ok(uxos_updated)
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

pub fn create_transaction_manager(
    accounts: Vec<Account>,
    logger: Sender<LogMessages>,
    app_sender: Sender<ApplicationManagerMessages>,
) -> Sender<TransactionMessages> {
    let (sender, receiver) = channel();

    let transaction_manager = Arc::new(Mutex::new(TransactionManager {
        utxos: UTXOSet::new(),
        tx_pendings: HashMap::new(),
        accounts,
        logger,
        sender_block_broadcasting: None,
        sender_app_manager: app_sender,
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
    // filtrar los bloxks por sólo aquellos que tiene transacciones
    let blocks_with_tx = blocks
        .into_iter()
        .filter(|block| !block.txns.is_empty())
        .collect::<Vec<SerializedBlock>>();

    utxo_set.update_from_blocks(blocks_with_tx, accounts)?;
    Ok(utxo_set)
}

pub fn _update_from_transactions(
    logger: Sender<LogMessages>,
    manager: Sender<TransactionMessages>,
    blocks: Vec<SerializedBlock>,
    accounts: Vec<Account>,
) -> Result<(), NodoBitcoinError> {
    let (sender, receiver) = channel();
    _ = manager.send(TransactionMessages::_UpdateFromBlocks((
        blocks, accounts, sender,
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
