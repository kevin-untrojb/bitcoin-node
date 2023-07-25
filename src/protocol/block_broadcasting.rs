use crate::blockchain::file_manager::{read_blocks_from_file, FileMessages};
use crate::{
    blockchain::{
        block::{SerializedBlock, pow_poi_validation},
        blockheader::BlockHeader,
        file::{escribir_archivo, escribir_archivo_bloque},
        transaction::Transaction,
    },
    errores::NodoBitcoinError,
    log::{log_error_message, log_info_message, LogMessages},
    messages::{
        getdata::GetDataMessage, headers::deserealize_sin_guardar, messages_header::check_header,
        ping_pong::make_pong,
    },
    wallet::transaction_manager::TransactionMessages,
};
use std::sync::mpsc::{channel, Sender};
use std::{
    sync::{Arc, Mutex, MutexGuard},
    thread,
};

use super::admin_connections::AdminConnections;

pub enum BlockBroadcastingMessages {
    ShutDown,
}

/// Escucha "infinitamente" por mensajes de los nodos de la red
/// Recibe nuevos bloques y transacciones y se los envía al Transaction Manager
/// Se cortarán los hilos cuando se reciba un mensaje ShutDown
pub fn init_block_broadcasting(
    logger: Sender<LogMessages>,
    mut admin_connections: AdminConnections,
    sender_tx_manager: Sender<TransactionMessages>,
    file_manager: Sender<FileMessages>,
) -> Result<(), NodoBitcoinError> {
    let blocks = Arc::new(Mutex::new(read_blocks_from_file(file_manager.clone())?));
    let mut threads = vec![];
    let (sender, receiver) = channel();
    if sender_tx_manager
        .send(TransactionMessages::SenderBlockBroadcasting(sender))
        .is_err()
    {
        return Err(NodoBitcoinError::NoSePudoConectar);
    };
    let senders: Vec<Sender<BlockBroadcastingMessages>> = Vec::new();
    let sender_mutex = Arc::new(Mutex::new(senders));

    let thread_logger_shutdown = logger.clone();
    let sender_mutex_clone = sender_mutex.clone();
    thread::spawn(move || {
        if let Ok(message) = receiver.recv() {
            match message {
                BlockBroadcastingMessages::ShutDown => {
                    let senders_locked = match sender_mutex_clone.lock() {
                        Ok(senders_locked) => senders_locked,
                        Err(_) => return,
                    };
                    log_info_message(
                        thread_logger_shutdown.clone(),
                        "Inicio cierre hilos block broadcasting.".to_string(),
                    );
                    for sender in senders_locked.iter() {
                        if sender.send(BlockBroadcastingMessages::ShutDown).is_err() {
                            continue;
                        };
                    }

                    drop(senders_locked);
                }
            }
        }
    });

    for connection in admin_connections.get_connections() {
        let socket = connection.clone();
        let thread_logger = logger.clone();
        let thread_sender_tx_manager = sender_tx_manager.clone();
        let shared_blocks = blocks.clone();
        let sender_mutex_connection = sender_mutex.clone();
        threads.push(thread::spawn(move || {
            let (sender_thread, receiver_thread) = channel();
            let mut senders_locked = match sender_mutex_connection.lock(){
                Ok(senders_locked) => senders_locked,
                Err(_) => return,
            };
            senders_locked.push(sender_thread);
            drop(senders_locked);
            loop {
                if let Ok(message) = receiver_thread.try_recv() {
                    match message {
                        BlockBroadcastingMessages::ShutDown => {
                            log_info_message(
                                thread_logger,
                                format!{"Hilo de conexión {} cerrado correctamente.", socket.id}
                            );
                            return;
                        }
                    }
                }

                let mut buffer = [0u8; 24];
                if socket.read_exact_message(&mut buffer).is_err() {
                    log_error_message(
                        thread_logger.clone(),
                        format!("Error al leer el header del mensaje en broadcasting en conexión {}", socket.id),
                    );
                    return;
                }

                let (command, header) = match check_header(&buffer) {
                    Ok((command, payload_len)) => {
                        let mut header = vec![0u8; payload_len];
                        if socket.read_message(&mut header).is_err() {
                            log_error_message(
                                thread_logger.clone(),
                                format!("Error al leer el mensaje en broadcasting en conexión {}", socket.id),
                            );
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
                        log_error_message(
                            thread_logger.clone(),
                            format!("Error al escribir el mensaje pong en conexión {}", socket.id),
                        );
                        return;
                    }
                }

                if command == "inv" {
                    log_info_message(thread_logger.clone(), format!("Mensaje inv recibido en conexión {}", socket.id));
                    let get_data = match GetDataMessage::new_for_tx(&header) {
                        Ok(get_data) => {
                            get_data
                        },
                        Err(_) => {
                            log_error_message(thread_logger.clone(), format!("Error al crear el get data para el inv en conexión {}", socket.id));
                            continue
                        },
                    };

                    let get_data_message = match get_data.serialize() {
                        Ok(res) => res,
                        Err(_) => {
                            log_error_message(
                                thread_logger.clone(),
                                format!("Error al serializar el get_data en conexión {}.", socket.id),
                            );
                            continue;
                        }
                    };

                    if socket.write_message(&get_data_message).is_err() {
                        log_error_message(
                            thread_logger,
                            format!("Error al escribir el mensaje get_data en conexión {}", socket.id),
                        );
                        return;
                    }

                    let mut buffer = [0u8; 24];
                    if socket.read_exact_message(&mut buffer).is_err() {
                        log_error_message(
                            thread_logger,
                            format!("Error al leer el header mensaje en broadcasting en conexión {}.", socket.id),
                        );
                        return;
                    }

                    let (command, tx_read) = match check_header(&buffer) {
                        Ok((command, payload_len)) => {
                            let mut tx_read = vec![0u8; payload_len];
                            if socket.read_message(&mut tx_read).is_err() {
                                log_error_message(
                                    thread_logger,
                                    format!("Error al leer el mensaje en broadcasting en conexión {}.", socket.id),
                                );
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
                        log_info_message(thread_logger.clone(), format!("Tx recibido en conexión {}.", socket.id));
                        let tx = match Transaction::deserialize(&tx_read){
                            Ok(tx) => {
                                let msj = format!("Transacción nueva descerializada correctamente: {:?}", tx.txid().unwrap().to_hexa_le_string());
                                log_info_message(thread_logger.clone(), msj);
                                tx
                            },
                            Err(_) => {
                                log_error_message(thread_logger.clone(), "No se pudo guardar la nueva transacción recibida en block broadcasting.".to_string());
                                continue;
                            }
                        };

                        if thread_sender_tx_manager.send(TransactionMessages::NewTx(tx)).is_err(){
                            continue;
                        };
                        log_info_message(thread_logger.clone(), "Nueva transacción enviada al manager".to_string());
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
                                format!("Error al calcular el hash del header en conexión {}.", socket.id),
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
                                format!("Error al serializar el get_data en conexión {}.", socket.id),
                            );
                            continue;
                        }
                    };

                    if socket.write_message(&get_data_message).is_err() {
                        log_error_message(
                            thread_logger,
                            format!("Error al escribir el mensaje get data en broadcasting en conexión {}", socket.id),
                        );
                        return;
                    }

                    let mut buffer = [0u8; 24];
                    if socket.read_message(&mut buffer).is_err() {
                        log_error_message(
                            thread_logger,
                            format!("Error al leer el header mensaje en broadcasting en conexión {}.", socket.id),
                        );
                        return;
                    }

                    let (command, block_read) = match check_header(&buffer) {
                        Ok((command, payload_len)) => {
                            let mut block_read = vec![0u8; payload_len];
                            if socket.read_message(&mut block_read).is_err() {
                                log_error_message(
                                    thread_logger,
                                    format!("Error al leer el mensaje en broadcasting en conexión {}.", socket.id),
                                );
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

                        if !pow_poi_validation(thread_logger.clone(), block.clone()) {
                            _ = thread_sender_tx_manager.send(TransactionMessages::POIInvalido);
                            continue;
                        }
                        
                        let cloned_result = shared_blocks.lock();
                        if let Ok(cloned) = cloned_result {
                            guardar_header_y_bloque(thread_logger.clone(), block.clone(), cloned, header[0]);
                            if thread_sender_tx_manager.send(TransactionMessages::NewBlock(block)).is_err(){
                                return;
                            };
                        } else {
                            log_error_message(
                                thread_logger,
                                "Error al lockear el vector de bloques".to_string(),
                            );
                            return;
                        }
                }
                }
            }
        }));
    }

    for thread in threads {
        let _ = thread.join();
    }

    // si llegué porque quise o porque se cerraron todas

    _ = sender_tx_manager.send(TransactionMessages::Shutdowned);

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
