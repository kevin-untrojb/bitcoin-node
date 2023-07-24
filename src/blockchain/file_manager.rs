use std::sync::mpsc::{channel, Sender};
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use crate::log::{log_error_message, log_info_message, LogMessages};
use crate::{config, errores::NodoBitcoinError};
use crate::blockchain::file::{leer_ultimo_header,escribir_archivo_bloque,escribir_archivo,leer_todos_blocks};

#[derive(Clone)]
pub struct FileManager {
    headers_file_name: String,
    block_file_name: String,
    logger: Sender<LogMessages>,
}

pub enum FileMessages {
    ReadAllBlocks(Sender<Result<Vec<Vec<u8>>, NodoBitcoinError>>),
    WriteHeadersFile((Vec<u8>, Sender<Result<(), NodoBitcoinError>>)),
    WriteBlockFile ((Vec<u8>, Sender<Result<(), NodoBitcoinError>>)),
    ReadLastHeader(Sender<Result<Vec<u8>, NodoBitcoinError>>),
    ShutDown()
}


impl FileManager {
    pub fn new(logger: Sender<LogMessages>) ->Result<Sender<FileMessages>, NodoBitcoinError>{
        let headers_file_name = config::get_valor("NOMBRE_ARCHIVO_HEADERS".to_string())?;
        let block_file_name = config::get_valor("NOMBRE_ARCHIVO_BLOQUES".to_string())?;

        let (sender, receiver) = channel();

        let file_manager = Arc::new(Mutex::new(FileManager {
            logger,
            headers_file_name,
            block_file_name
        }));

        thread::spawn(move || {
            let fm = file_manager.clone();
            while let Ok(message) = receiver.recv() {
                let mut manager = match fm.lock() {
                    Ok(manager) => manager,
                    Err(_) => continue,
                };
                manager.handle_message(message);
            }
        });

        Ok(sender)
    }


    fn handle_message(&mut self, message: FileMessages){
        match message{
            FileMessages::ReadAllBlocks(result) =>{
                result.send(leer_todos_blocks());
            },
            FileMessages::WriteHeadersFile((data, result)) =>{
                result.send(escribir_archivo(&data));
            },
            FileMessages::WriteBlockFile(( data, result)) =>{
                result.send(escribir_archivo_bloque(&data));
            },
            FileMessages::ReadLastHeader(result) =>{
                result.send(leer_ultimo_header());
            },
            FileMessages::ShutDown() => {
                return;
            }
        }
    }
}

pub fn read_all_blocks(file_manager: Sender<FileMessages>) -> Result<Vec<Vec<u8>>, NodoBitcoinError> {
    let (result_sender, result_receiver) = channel();
    _ = file_manager.send(FileMessages::ReadAllBlocks(result_sender));

    match result_receiver.recv() {
        Ok(result) => result ,
        Err(_) => {
            // todo log error
            // handle error
            Err(NodoBitcoinError::InvalidAccount)
        }
    }
}
/*
pub fn _update_from_transactions(
    logger: Sender<LogMessages>,
    manager: Sender<TransactionMessages>,
    blocks: Vec<SerializedBlock>,
    accounts: Vec<Account>,
) -> Result<(), NodoBitcoinError> {
    let (sender, receiver) = channel();
    _ = manager.send(FileMessages::((
        , sender,
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


 */