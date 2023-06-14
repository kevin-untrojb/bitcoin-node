use crate::{
    blockchain::{
        block::SerializedBlock,
        blockheader::BlockHeader,
        file::{escribir_archivo, escribir_archivo_bloque},
        proof_of_work::pow_validation, transaction::Transaction,
    },
    errores::NodoBitcoinError,
    log::{log_error_message, log_info_message, LogMessages},
    messages::{
        getdata::GetDataMessage, headers::deserealize_sin_guardar, messages_header::check_header,
        ping_pong::make_pong,
    },
};
use std::sync::mpsc::Sender;
use std::{
    sync::{Arc, Mutex, MutexGuard},
    thread,
};

use super::admin_connections::AdminConnections;

pub fn init_block_broadcasting(
    logger: Sender<LogMessages>,
    mut admin_connections: AdminConnections,
) -> Result<(), NodoBitcoinError> {
    let blocks = Arc::new(Mutex::new(SerializedBlock::read_blocks_from_file()?));
    let mut threads = vec![];
    for connection in admin_connections.get_connections() {
        let socket = connection.clone();
        let thread_logger = logger.clone();
        let shared_blocks = blocks.clone();
        threads.push(thread::spawn(move || loop {
            let mut buffer = [0u8; 24];
            if socket.read_message(&mut buffer).is_err() {
                log_error_message(thread_logger, "Error al leer el header del mensaje en broadcasting".to_string());
                return;
            }
            let (command, header) = match check_header(&buffer) {
                Ok((command, payload_len)) => {
                    let mut header = vec![0u8; payload_len];
                    if socket.read_exact_message(&mut header).is_err() {
                        log_error_message(thread_logger, "Error al leer el mensaje en broadcasting".to_string());
                        return;
                    }
                    (command, header)
                }
                Err(NodoBitcoinError::MagicNumberIncorrecto) => {
                    continue;
                }
                Err(_) => continue,
            };

            if command == "ping" {
                let pong_msg = match make_pong(&header) {
                    Ok(msg) => msg,
                    Err(_) => continue,
                };

                if socket.write_message(&pong_msg).is_err() {
                    log_info_message(thread_logger, "Error al escribir el mensaje pong".to_string());
                    return;
                }
            }

            if command == "inv" {
                let get_data = match GetDataMessage::new_for_tx(&header){
                    Ok(get_data) => get_data,
                    Err(_) => continue,
                };

                let get_data_message = match get_data.serialize() {
                    Ok(res) => res,
                    Err(_) => {
                        log_error_message(
                            thread_logger.clone(),
                            "Error al serializar el get_data.".to_string(),
                        );
                        continue;
                    }
                };

                if socket.write_message(&get_data_message).is_err() {
                    log_error_message(thread_logger, "Error al escribir el mensaje get_data".to_string());
                    return;
                }

                let mut buffer = [0u8; 24];
                if socket.read_exact_message(&mut buffer).is_err() {
                    log_error_message(thread_logger, "Error al leer el header mensaje en broadcasting.".to_string());
                    return;
                }

                let (command, tx_read) = match check_header(&buffer) {
                    Ok((command, payload_len)) => {
                        let mut tx_read = vec![0u8; payload_len];
                        if socket.read_exact_message(&mut tx_read).is_err() {
                            log_error_message(thread_logger, "Error al leer el mensaje en broadcasting.".to_string());
                            return;
                        }
                        (command, tx_read)
                    }
                    Err(NodoBitcoinError::MagicNumberIncorrecto) => {
                        continue;
                    }
                    Err(_) => continue,
                };

                if command == "tx" {
                    let tx = Transaction::deserialize(&tx_read);
                }

            }

            if command == "headers" {
                let header = match deserealize_sin_guardar(header) {
                    Ok(header) => header,
                    Err(_) => continue,
                };
                let hash_header = match header[0].hash() {
                    Ok(res) => res,
                    Err(_) => {
                        log_error_message(
                            thread_logger,
                            "Error al calcular el hash del header.".to_string(),
                        );
                        return;
                    }
                };

                let get_data = GetDataMessage::new(1, hash_header);

                let get_data_message = match get_data.serialize() {
                    Ok(res) => res,
                    Err(_) => {
                        log_error_message(
                            thread_logger.clone(),
                            "Error al serializar el get_data.".to_string(),
                        );
                        continue;
                    }
                };

                if socket.write_message(&get_data_message).is_err() {
                    log_error_message(thread_logger, "Error al escribir el mensaje get data en broadcasting".to_string());
                    return;
                }

                let mut buffer = [0u8; 24];
                if socket.read_exact_message(&mut buffer).is_err() {
                    log_error_message(thread_logger, "Error al leer el header mensaje en broadcasting.".to_string());
                    return;
                }

                let (command, block_read) = match check_header(&buffer) {
                    Ok((command, payload_len)) => {
                        let mut block_read = vec![0u8; payload_len];
                        if socket.read_exact_message(&mut block_read).is_err() {
                            log_error_message(thread_logger, "Error al leer el mensaje en broadcasting.".to_string());
                            return;
                        }
                        (command, block_read)
                    }
                    Err(NodoBitcoinError::MagicNumberIncorrecto) => {
                        continue;
                    }
                    Err(_) => continue,
                };

                if command == "block" {
                    let block = match SerializedBlock::deserialize(&block_read) {
                        Ok(block) => block,
                        Err(_) => continue,
                    };

                    pow_poi_validation(thread_logger.clone(), block.clone());

                    let cloned_result = shared_blocks.lock();
                    if let Ok(cloned) = cloned_result {
                        guardar_header_y_bloque(thread_logger.clone(), block, cloned, header[0]);
                    } else {
                        log_error_message(
                            thread_logger,
                            "Error al lockear el vector de bloques".to_string(),
                        );
                        return;
                    }
                }
            }
        }));
    }

    for thread in threads {
        let _ = thread.join();
    }

    Ok(())
}

fn escribir_header_y_bloque(
    logger: Sender<LogMessages>,
    bloque: SerializedBlock,
    blockheader: BlockHeader,
) -> Result<(), NodoBitcoinError> {
    log_info_message(logger.clone(), "Guardando headers...".to_string());
    let bytes = blockheader.serialize()?;
    escribir_archivo(&bytes)?;

    log_info_message(logger.clone(), "Header nuevo guardado".to_string());

    escribir_archivo_bloque(&bloque.serialize()?)?;

    log_info_message(logger, "Bloque nuevo guardado".to_string());

    Ok(())
}

fn pow_poi_validation(thread_logger: Sender<LogMessages>, block: SerializedBlock) -> bool {
    let pow = match pow_validation(&block.header) {
        Ok(pow) => {
            log_info_message(thread_logger.clone(), "POW nuevo bloque válida".to_string());
            pow
        }
        Err(_) => {
            log_error_message(thread_logger, "POW nuevo bloque inválida".to_string());
            return false;
        }
    };

    let poi = block.is_valid_merkle();
    if poi {
        log_info_message(thread_logger, "POI nuevo bloque válida".to_string());
    } else {
        log_error_message(thread_logger, "POI nuevo bloque inválida".to_string());
    }

    pow && poi
}

fn guardar_header_y_bloque(
    thread_logger: Sender<LogMessages>,
    block: SerializedBlock,
    mut cloned: MutexGuard<Vec<SerializedBlock>>,
    header: BlockHeader,
) {
    if SerializedBlock::contains_block(cloned.to_vec(), block.clone()) {
        log_error_message(thread_logger, "Bloque repetido".to_string());
    } else {
        match escribir_header_y_bloque(thread_logger.clone(), block.clone(), header) {
            Ok(_) => {
                cloned.push(block);
                log_info_message(
                    thread_logger,
                    "Bloque nuevo guardado correctamente".to_string(),
                );
                drop(cloned);
            }
            Err(_) => {
                log_error_message(
                    thread_logger,
                    "Error al guardar el nuevo bloque".to_string(),
                );
            }
        }
    }
}
