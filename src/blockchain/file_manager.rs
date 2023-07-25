use crate::blockchain::block::SerializedBlock;
use crate::blockchain::file::{
    escribir_archivo, escribir_archivo_bloque, leer_todos_blocks, leer_ultimo_header,
};
use crate::log::{log_error_message, log_info_message, LogMessages};
use crate::{config, errores::NodoBitcoinError};
use std::sync::mpsc::{channel, Sender};
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

#[derive(Clone)]
pub struct FileManager {
    headers_file_name: String,
    block_file_name: String,
    logger: Sender<LogMessages>,
}

pub enum FileMessages {
    ReadAllBlocks(Sender<Result<Vec<Vec<u8>>, NodoBitcoinError>>),
    WriteHeadersFile((Vec<u8>, Sender<Result<(), NodoBitcoinError>>)),
    WriteBlockFile((Vec<u8>, Sender<Result<(), NodoBitcoinError>>)),
    ReadLastHeader(Sender<Result<Vec<u8>, NodoBitcoinError>>),
    ShutDown(),
}

impl FileManager {
    pub fn new(logger: Sender<LogMessages>) -> Sender<FileMessages> {
        let headers_file_name: String;
        let block_file_name: String;

        match config::get_valor("NOMBRE_ARCHIVO_HEADERS".to_string()) {
            Ok(real_headers_file_name) => {
                headers_file_name = real_headers_file_name;
            }
            Err(_) => {
                headers_file_name = "".to_string();
            }
        }

        match config::get_valor("NOMBRE_ARCHIVO_BLOQUES".to_string()) {
            Ok(real_block_file_name) => {
                block_file_name = real_block_file_name;
            }
            Err(_) => {
                block_file_name = "".to_string();
            }
        }

        let (sender, receiver) = channel();

        let file_manager = Arc::new(Mutex::new(FileManager {
            logger,
            headers_file_name,
            block_file_name,
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

        sender
    }

    fn handle_message(&mut self, message: FileMessages) {
        match message {
            FileMessages::ReadAllBlocks(result) => {
                result.send(leer_todos_blocks());
            }
            FileMessages::WriteHeadersFile((data, result)) => {
                result.send(escribir_archivo(&data));
            }
            FileMessages::WriteBlockFile((data, result)) => {
                result.send(escribir_archivo_bloque(&data));
            }
            FileMessages::ReadLastHeader(result) => {
                result.send(leer_ultimo_header());
            }
            FileMessages::ShutDown() => {
                return;
            }
        }
    }
}

pub fn read_blocks_from_file(
    file_manager: Sender<FileMessages>,
) -> Result<Vec<SerializedBlock>, NodoBitcoinError> {
    let (result_sender, result_receiver) = channel();
    _ = file_manager.send(FileMessages::ReadAllBlocks(result_sender));

    match result_receiver.recv() {
        Ok(res) => {
            let block_bytes = res?.to_vec();
            let mut serialized_blocks = vec![];
            for block in &block_bytes {
                let serialized_block = SerializedBlock::deserialize(block)?;
                serialized_blocks.push(serialized_block);
            }
            Ok(serialized_blocks)
        }
        Err(_) => {
            // todo log error
            // handle error
            Err(NodoBitcoinError::InvalidAccount)
        }
    }
}

pub fn shutdown(file_manager: Sender<FileMessages>) {
    file_manager.send(FileMessages::ShutDown());
}
