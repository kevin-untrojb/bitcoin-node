use crate::blockchain::block::SerializedBlock;
use crate::blockchain::blockheader::BlockHeader;
use crate::blockchain::file::get_file_header_size;
use crate::blockchain::file::leer_bloque;
use crate::blockchain::file::leer_bytes;
use crate::blockchain::file::{
    escribir_archivo, escribir_archivo_bloque, get_blocks_filename, get_headers_filename,
    leer_header_desde_archivo, leer_todos_blocks,
};
use crate::blockchain::index::dump_hash_in_the_index;
use crate::blockchain::index::get_start_index;
use crate::log::{log_error_message, log_info_message, LogMessages};
use crate::protocol::initial_block_download::GENESIS_BLOCK;
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
    WriteHeadersAndBlockFile(
        (
            [u8; 32],
            Vec<u8>,
            [u8; 32],
            Vec<u8>,
            Sender<Result<(), NodoBitcoinError>>,
        ),
    ),
    GetHeaders(([u8; 32], Sender<Result<Vec<u8>, NodoBitcoinError>>)),
    GetHeader(([u8; 32], Sender<Result<Vec<u8>, NodoBitcoinError>>)),
    ShutDown(),
}

impl FileManager {
    pub fn new(logger: Sender<LogMessages>) -> Sender<FileMessages> {
        let headers_file_name: String;
        let block_file_name: String;

        match get_headers_filename() {
            Ok(real_headers_file_name) => {
                headers_file_name = real_headers_file_name;
            }
            Err(_) => {
                headers_file_name = "".to_string();
            }
        }

        match get_blocks_filename() {
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
            FileMessages::WriteHeadersAndBlockFile((
                block_hash,
                block_bytes,
                header_hash,
                header_bytes,
                result,
            )) => {
                log_info_message(
                    self.logger.clone(),
                    "Guardando headers y bloques...".to_string(),
                );
                let index_block =
                    match escribir_archivo_bloque(self.block_file_name.clone(), &block_bytes) {
                        Ok(index) => index,
                        Err(error) => {
                            result.send(Err(error));
                            return;
                        }
                    };
                log_info_message(self.logger.clone(), "Bloque nuevo guardado".to_string());

                match dump_hash_in_the_index(self.block_file_name.clone(), block_hash, index_block)
                {
                    Ok(_) => {}
                    Err(error) => {
                        result.send(Err(error));
                        return;
                    }
                };
                log_info_message(
                    self.logger.clone(),
                    format!("Indice de bloque guardado {}", index_block),
                );

                let index_header =
                    match escribir_archivo(self.headers_file_name.clone(), &header_bytes) {
                        Ok(index) => index,
                        Err(error) => {
                            result.send(Err(error));
                            return;
                        }
                    };
                log_info_message(self.logger.clone(), "Header nuevo guardado".to_string());
                match dump_hash_in_the_index(
                    self.headers_file_name.clone(),
                    header_hash,
                    index_header,
                ) {
                    Ok(_) => {}
                    Err(error) => {
                        result.send(Err(error));
                        return;
                    }
                };
                log_info_message(
                    self.logger.clone(),
                    format!("Indice de header guardado {}", index_header),
                );

                result.send(Ok(()));
            }
            FileMessages::ReadAllBlocks(result) => {
                result.send(leer_todos_blocks());
            }
            FileMessages::ShutDown() => {
                return;
            }

            FileMessages::GetHeaders((hash_id, result)) => {
                let mut header_index;

                if hash_id == GENESIS_BLOCK {
                    header_index = 0;
                } else {
                    header_index = match get_start_index(self.headers_file_name.clone(), hash_id) {
                        Ok(index) => index + 80,
                        Err(error) => {
                            result.send(Err(error));
                            return;
                        }
                    };
                }

                let file_size = match get_file_header_size() {
                    Ok(size) => size,
                    Err(error) => {
                        result.send(Err(error));
                        return;
                    }
                };

                let length = 80 * 2000;
                // valor menor entre leght + offset y file_size
                let length = if length + header_index < file_size {
                    length
                } else {
                    file_size - header_index
                };

                let bytes = match leer_bytes(self.headers_file_name.clone(), header_index, length) {
                    Ok(data) => data,
                    Err(error) => {
                        result.send(Err(error));
                        return;
                    }
                };
                result.send(Ok(bytes));
            }
            FileMessages::GetHeader((hash_id, result)) => {
                let mut header_index;

                header_index = match get_start_index(self.headers_file_name.clone(), hash_id) {
                    Ok(index) => index,
                    Err(error) => {
                        result.send(Err(error));
                        return;
                    }
                };

                let file_size = match get_file_header_size() {
                    Ok(size) => size,
                    Err(error) => {
                        result.send(Err(error));
                        return;
                    }
                };

                let length = 80;
                // valor menor entre leght + offset y file_size
                let length = if length + header_index < file_size {
                    length
                } else {
                    file_size - header_index
                };

                let bytes = match leer_bytes(self.headers_file_name.clone(), header_index, length) {
                    Ok(data) => data,
                    Err(error) => {
                        result.send(Err(error));
                        return;
                    }
                };
                result.send(Ok(bytes));
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

pub fn get_headers_from_file(
    file_manager: Sender<FileMessages>,
    hash_buscado: [u8; 32],
) -> Result<Vec<u8>, NodoBitcoinError> {
    let (result_sender, result_receiver) = channel();
    _ = file_manager.send(FileMessages::GetHeaders((hash_buscado, result_sender)));
    match result_receiver.recv() {
        Ok(result) => result,
        Err(err) => {
            // todo handle
            Err(NodoBitcoinError::InvalidAccount)
        }
    }
}

pub fn get_header_from_file(
    file_manager: Sender<FileMessages>,
    hash_buscado: [u8; 32],
) -> Result<Vec<u8>, NodoBitcoinError> {
    let (result_sender, result_receiver) = channel();
    _ = file_manager.send(FileMessages::GetHeader((hash_buscado, result_sender)));
    match result_receiver.recv() {
        Ok(result) => result,
        Err(err) => {
            // todo handle
            Err(NodoBitcoinError::InvalidAccount)
        }
    }
}

pub fn write_headers_and_block_file(
    file_manager: Sender<FileMessages>,
    block: SerializedBlock,
    block_header: BlockHeader,
) -> Result<(), NodoBitcoinError> {
    let (result_sender, result_receiver) = channel();
    let header_bytes = block_header.serialize()?;
    let block_byes = block.serialize()?;
    let block_hash = block.header.hash()?;
    let header_hash = block_header.hash()?;
    _ = file_manager.send(FileMessages::WriteHeadersAndBlockFile((
        block_hash,
        block_byes,
        header_hash,
        header_bytes,
        result_sender,
    )));

    match result_receiver.recv() {
        Ok(_) => Ok(()),
        Err(err) => {
            // todo handle
            Err(NodoBitcoinError::InvalidAccount)
        }
    }
}

pub fn shutdown(file_manager: Sender<FileMessages>) {
    file_manager.send(FileMessages::ShutDown());
}

#[cfg(test)]
mod tests {
    use crate::{
        blockchain::file_manager::{get_headers_from_file, FileManager},
        config,
        log::create_logger_actor,
        protocol::initial_block_download::GENESIS_BLOCK,
    };

    fn init_config() {
        let args: Vec<String> = vec!["app_name".to_string(), "src/nodo.conf".to_string()];
        _ = config::inicializar(args);
    }

    #[test]
    fn test_get_header() {
        init_config();
        let logger = create_logger_actor(config::get_valor("LOG_FILE".to_string()));
        let file_manager = FileManager::new(logger.clone());
        let hash: [u8; 32] = [
            229, 94, 124, 89, 15, 75, 44, 222, 240, 35, 41, 188, 16, 213, 143, 250, 149, 109, 29,
            10, 111, 146, 99, 54, 138, 72, 107, 37, 0, 0, 0, 0,
        ];

        let genesis = GENESIS_BLOCK;

        let result = get_headers_from_file(file_manager, genesis);
        assert!(result.is_ok());
    }
}
