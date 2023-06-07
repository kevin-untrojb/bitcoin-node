use crate::{errores::NodoBitcoinError, blockchain::{block::SerializedBlock, proof_of_work::pow_validation}, messages::{messages_header::check_header, headers::deserealize_sin_guardar, getdata::GetDataMessage, ping_pong::make_pong}, log::{log_info_message, LogMessages}};
use std::{thread};
use std::sync::mpsc::Sender;

use super::admin_connections::AdminConnections;

pub fn init_block_broadcasting(logger: Sender<LogMessages>, admin_connections: AdminConnections) -> Result<(), NodoBitcoinError> {
    for connection in admin_connections.clone().get_connections() {
        let socket = connection.clone();
        let thread_logger = logger.clone();
        thread::spawn(move || {
            loop {
                let mut buffer = [0u8; 24];
                if socket.read_exact_message(&mut buffer).is_err(){
                    log_info_message(thread_logger, "Error al leer el mensaje".to_string());
                    return;
                }
                let (command, header) = match check_header(&buffer) {
                    Ok((command, payload_len)) => {
                        let mut header = vec![0u8; payload_len];
                        if socket.read_exact_message(&mut header).is_err(){
                            log_info_message(thread_logger, "Error al leer el mensaje".to_string());
                            return;
                        }
                        (command, header)
                    }
                    Err(NodoBitcoinError::MagicNumberIncorrecto) => {
                        continue;
                    }
                    Err(_) => {
                        println!("ERRROR");
                        continue
                    },
                };

                if command == "ping" {
                    let pong_msg = match make_pong(&header){
                        Ok(msg) => msg,
                        Err(_) => continue,
                    };

                    if socket.write_message(&pong_msg).is_err(){
                        log_info_message(thread_logger, "Error al escribir el mensaje".to_string());
                        return;
                    }

                }

                println!("{:?}", command);
                if command == "headers" {
                    let header = match deserealize_sin_guardar(header){
                        Ok(header) => header,
                        Err(_) => continue
                    };
                    let hash_header = match header[0].hash() {
                        Ok(res) => res,
                        Err(_) => {
                            println!("Error al calcular el hash del header.");
                            return;
                        }
                    };

                    let get_data = GetDataMessage::new(
                        1,
                        hash_header,
                    );

                    let get_data_message = match get_data.serialize() {
                        Ok(res) => res,
                        Err(_) => {
                            println!("Error al serializar el get_data. Reintentando ...");
                            continue;
                        }
                    };

                    if socket.write_message(&get_data_message).is_err(){
                        log_info_message(thread_logger, "Error al escribir el mensaje".to_string());
                        return;
                    }
                    
                    let mut buffer = [0u8; 24];
                    if socket.read_exact_message(&mut buffer).is_err(){
                        log_info_message(thread_logger, "Error al leer el mensaje".to_string());
                        return;
                    }
                    let (command, block_read) = match check_header(&buffer) {
                        Ok((command, payload_len)) => {
                            let mut block_read = vec![0u8; payload_len];
                            if socket.read_exact_message(&mut block_read).is_err(){
                                log_info_message(thread_logger, "Error al leer el mensaje".to_string());
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
                        let block = match SerializedBlock::deserialize(&block_read){
                            Ok(block) => block,
                            Err(_) => continue
                        };

                        let pow = match pow_validation(&block.header){
                            Ok(pow) => pow,
                            Err(_) => continue
                        };

                        let poi = block.is_valid_merkle();
                        if !pow || !poi {
                            continue;
                        }
                    }

                }
            }
        });
    }

    Ok(())
}