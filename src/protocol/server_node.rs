use crate::blockchain::file_manager::FileMessages;
use std::{
    io::{ErrorKind, Read, Write},
    net::{TcpListener, TcpStream},
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread::{self},
    time::Duration,
};

use chrono::Utc;

use crate::{
    blockchain::block::SerializedBlock,
    common::utils_bytes::ping_nonce,
    config,
    errores::NodoBitcoinError,
    log::{log_error_message, log_info_message, LogMessages},
    messages::{
        blocks::make_block,
        getdata::GetDataMessage,
        getheaders::GetHeadersMessage,
        headers::make_headers_msg,
        messages_header::{check_header, make_header},
        ping_pong::{make_ping, make_pong},
        version::VersionMessage,
    },
    wallet::transaction_manager::TransactionMessages,
};

const READ_TIMEOUT_SECONDS: u64 = 10;

pub enum ServerNodeMessages {
    GetBlockResponse(Option<SerializedBlock>),
    ShutDown,
}

/// Comienza a correr el servidor en el puerto especificado en la configuración
pub fn init_server(
    logger: Sender<LogMessages>,
    file_manager: Sender<FileMessages>,
    sender_tx_manager: Sender<TransactionMessages>,
) -> Result<(), NodoBitcoinError> {
    let port = config::get_valor("PORT".to_owned())?;

    let address = "127.0.0.1:".to_owned() + &port;
    _ = server_run(&address, file_manager, logger, sender_tx_manager);
    Ok(())
}

/// Crea y devuelve el listener que escucha en la dirección recibida por parámetro
fn crear_listener(
    address: &str,
    logger: Sender<LogMessages>,
) -> Result<TcpListener, NodoBitcoinError> {
    let listener = match TcpListener::bind(address) {
        Ok(res) => res,
        Err(_) => {
            log_error_message(logger, "Error al bindear el socket".to_string());
            return Err(NodoBitcoinError::NoSePudoConectar);
        }
    };

    if listener.local_addr().is_err() {
        log_error_message(logger, "Error al obtener la dirección local".to_string());
        return Err(NodoBitcoinError::NoSePudoConectar);
    }

    match listener.set_nonblocking(true) {
        Ok(_) => {}
        Err(_) => {
            log_error_message(
                logger,
                "Error al setear el socket como non-blocking".to_string(),
            );
            return Err(NodoBitcoinError::NoSePudoConectar);
        }
    }
    Ok(listener)
}

/// Realiza todas las acciones para que el servidor esté escuchando y maneje los mensajes que reciba en otro hilo
/// Si recibe un mensaje de ShutDown, cierra todos sus hilos y luego termina, enviando un mensaje para 
/// avisar que ya completó la finalización correctamente.
fn server_run(
    address: &str,
    file_manager: Sender<FileMessages>,
    logger: Sender<LogMessages>,
    sender_tx_manager: Sender<TransactionMessages>,
) -> Result<(), NodoBitcoinError> {
    let listener = crear_listener(address, logger.clone())?;

    let mut threads = vec![];

    let (sender, receiver) = channel();

    if sender_tx_manager
        .send(TransactionMessages::SenderServerNode(sender))
        .is_err()
    {
        return Err(NodoBitcoinError::NoSePudoConectar);
    };

    let senders_threads: Vec<Sender<ServerNodeMessages>> = Vec::new();
    let senders_threads_mutex = Arc::new(Mutex::new(senders_threads));
    let thread_logger_shutdown = logger.clone();

    let address = match listener.local_addr() {
        Ok(address) => address,
        Err(_) => return Err(NodoBitcoinError::ErrorEnLaDireccion),
    };

    log_info_message(
        logger.clone(),
        format!("Escuchando en: {:?}", address),
    );
    loop {
        match listener.accept() {
            Ok((mut stream, socket_addr)) => {
                log_info_message(
                    logger.clone(),
                    format!("Conexión establecida: {:?}", socket_addr),
                );
                let logger_cloned = logger.clone();
                let sender_mutex_connection = senders_threads_mutex.clone();
                let sender_tx_manager_clone = sender_tx_manager.clone();
                let sender_file_manager_clone = file_manager.clone();
                threads.push(thread::spawn(move || {
                    let (sender_thread, receiver_thread) = channel();
                    let mut senders_locked = match sender_mutex_connection.lock() {
                        Ok(senders_locked) => senders_locked,
                        Err(_) => return,
                    };
                    senders_locked.push(sender_thread);
                    drop(senders_locked);

                    handle_message(
                        &mut stream,
                        sender_file_manager_clone.clone(),
                        receiver_thread,
                        sender_tx_manager_clone.clone(),
                        logger_cloned.clone(),
                    );
                }));
            }
            Err(ref e) => {
                if e.kind() != ErrorKind::WouldBlock {
                    log_error_message(logger.clone(), "Error al aceptar la conexión".to_string());
                }
            }
        }

        if let Ok(message) = receiver.try_recv() {
            match message {
                ServerNodeMessages::ShutDown => {
                    let senders_locked = match senders_threads_mutex.lock() {
                        Ok(senders_locked) => senders_locked,
                        Err(_) => break,
                    };
                    log_info_message(
                        thread_logger_shutdown,
                        "Inicio cierre hilos del nodo server.".to_string(),
                    );
                    for sender_client in senders_locked.iter() {
                        if sender_client.send(ServerNodeMessages::ShutDown).is_err() {
                            continue;
                        };
                    }

                    drop(senders_locked);
                    break;
                }
                ServerNodeMessages::GetBlockResponse(_) => {}
            }
        }
    }

    for thread in threads {
        let _ = thread.join();
    }

    log_info_message(
        logger,
        "Todas las conexiones del Nodo server se cerraron satisfactoriamente.".to_string(),
    );

    _ = sender_tx_manager.send(TransactionMessages::ShutdownedServerNode(
        sender_tx_manager.clone(),
    ));

    Ok(())
}

/// Hace el handshake pero al revés, siendo el nodo que recibe el pedido de conexión
/// Si termina satisfactoriamente, la conexión con el nodo cliente está realizada
fn shakehand(stream: &mut TcpStream, logger: Sender<LogMessages>) -> Result<(), NodoBitcoinError> {
    let mut header = [0u8; 24];
    if stream.read_exact(&mut header).is_err() {
        return Err(NodoBitcoinError::ErrorEnHandshake);
    }

    let (command, payload_len) = check_header(&header)?;

    if command != "version" {
        return Err(NodoBitcoinError::ErrorEnHandshake);
    }

    let mut payload = vec![0u8; payload_len];
    if stream.read_exact(&mut payload).is_err() {
        return Err(NodoBitcoinError::ErrorEnHandshake);
    }

    let version = VersionMessage::get_version(&payload);
    let my_version = match (config::get_valor("VERSION".to_string())?).parse::<u32>() {
        Ok(res) => res,
        Err(_) => return Err(NodoBitcoinError::ErrorEnHandshake),
    };

    if version > my_version {
        return Err(NodoBitcoinError::ErrorEnHandshake);
    }

    let timestamp = Utc::now().timestamp() as u64;

    let client_address = match stream.peer_addr() {
        Ok(res) => res,
        Err(_) => {
            log_error_message(
                logger,
                "Error al obtener el address del cliente".to_string(),
            );
            return Err(NodoBitcoinError::ErrorEnLaDireccion);
        }
    };

    let version_message = VersionMessage::new(my_version, timestamp, client_address);
    let mensaje = version_message.serialize()?;
    if stream.write_all(&mensaje).is_err() {
        return Err(NodoBitcoinError::ErrorEnHandshake);
    }

    let verack_msg = make_header("verack".to_string(), &Vec::new())?;
    if stream.write_all(&verack_msg).is_err() {
        return Err(NodoBitcoinError::ErrorEnHandshake);
    }

    let mut verack_resp = vec![0u8; 24];
    if stream.read_exact(&mut verack_resp).is_err() {
        return Err(NodoBitcoinError::NoSePuedeLeerLosBytesVerackMessage);
    }

    let (command, _payload_len) = check_header(&verack_resp)?;

    if command != "verack" {
        return Err(NodoBitcoinError::ErrorEnHandshake);
    }

    Ok(())
}

/// Recibe el stream cliente
/// Se realiza el handshake y si se realiza correctamente, se queda a la espera de mensajes de parte del cliente
fn handle_message(
    stream: &mut TcpStream,
    file_manager: Sender<FileMessages>,
    receiver_thread: Receiver<ServerNodeMessages>,
    tx_sender: Sender<TransactionMessages>,
    logger: Sender<LogMessages>,
) {
    let duration = stream.set_read_timeout(Some(Duration::new(READ_TIMEOUT_SECONDS, 0)));
    if duration.is_err() {
        log_error_message(logger, "Error al setear read timeout.".to_string());
        return;
    }

    if let Ok(()) = shakehand(stream, logger.clone()) {
        log_info_message(
            logger.clone(),
            "Handshake exitoso con el cliente".to_string(),
        );
        thread_connection(stream, file_manager, receiver_thread, tx_sender, logger);
    };
}

/// Escucha por mensajes del cliente y realiza las acciones 
/// necesarias en caso de recibir mensaje getdata, headers o ping
/// También envía el mensaje ping en caso de que pase un determinado 
/// tiempo sin recibir mensajes de parte del cliente
fn thread_connection(
    stream: &mut TcpStream,
    file_manager: Sender<FileMessages>,
    receiver_thread: Receiver<ServerNodeMessages>,
    tx_sender: Sender<TransactionMessages>,
    logger: Sender<LogMessages>,
) {
    let client_address = match stream.peer_addr() {
        Ok(res) => res,
        Err(_) => {
            log_error_message(
                logger,
                "Error al obtener el address del cliente".to_string(),
            );
            return;
        }
    };
    log_info_message(
        logger.clone(),
        format!("Comienzo a escuchar mensajes de: {:?}", client_address),
    );

    let mut time_out_counter = 0;

    loop {
        if let Ok(message) = receiver_thread.try_recv() {
            match message {
                ServerNodeMessages::ShutDown => {
                    log_info_message(
                        logger.clone(),
                        format! {"Cerrando la conexión {} ...", client_address.clone()},
                    );
                    break;
                }
                ServerNodeMessages::GetBlockResponse(_) => {}
            }
        }

        let send_ping_on_timeout = time_out_counter >= max_time_outs();
        time_out_counter += 1;
        if send_ping_on_timeout {
            time_out_counter = 0;
        }

        let (command, message) = match read_message(stream, logger.clone(), send_ping_on_timeout) {
            Ok(option) => {
                if option.is_none() {
                    continue;
                }
                let (command, message) = option.unwrap();
                (command, message)
            }
            Err(_) => break,
        };

        if command == "ping" {
            log_info_message(
                logger.clone(),
                format!("ping recibido de {}", client_address),
            );
            match send_pong(message, stream, logger.clone()) {
                Ok(()) => continue,
                Err(_) => break,
            }
        }

        if command == "getheaders" {
            let getheaders_deserealized = GetHeadersMessage::deserealize(&message);
            if getheaders_deserealized.is_err() {
                log_error_message(
                    logger.clone(),
                    "No se puede deserealizar el mensaje getheaders (nodo servidor)".to_string(),
                );
                continue;
            }

            match make_headers_msg(file_manager.clone(), getheaders_deserealized.unwrap()) {
                Ok(headers_msg) => {
                    if stream.write_all(&headers_msg).is_err() {
                        log_error_message(
                            logger.clone(),
                            "No se puede enviar el mensaje HEADERS".to_string(),
                        );
                        break;
                    }
                    log_info_message(logger.clone(), "HEADERS enviado".to_string());
                }
                Err(_) => {
                    log_error_message(
                        logger.clone(),
                        "Error creando el mensaje HEADERS".to_string(),
                    );
                    continue;
                }
            }
        }
        if command == "getdata" {
            _ = send_block(message, stream, logger.clone(), tx_sender.clone());
            continue;
        }
    }
    log_info_message(
        logger,
        format!("Conexión {} cerrada correctamente.", client_address.clone()),
    );
}

/// Recibe un vector de hash de bloques
/// Devuelve un vector con los bloques correspondientes a esos hashes
fn get_blocks_from_hashes(
    hashes: Vec<Vec<u8>>,
    tx_sender: Sender<TransactionMessages>,
) -> Result<Vec<SerializedBlock>, NodoBitcoinError> {
    let mut blocks: Vec<SerializedBlock> = Vec::new();

    let (sender, receiver) = channel();
    for hash in hashes {
        _ = tx_sender.send(TransactionMessages::GetBlockRequest(hash, sender.clone()));

        if let Ok(message) = receiver.recv() {
            match message {
                ServerNodeMessages::ShutDown => {}
                ServerNodeMessages::GetBlockResponse(block) => {
                    if block.is_none() {
                        continue;
                    }
                    blocks.push(block.unwrap());
                }
            }
        }
    }
    Ok(blocks)
}

/// Se encarga de interpretar el mensaje getdata, buscar los bloques a devolver
/// y enviar el mensaje block con los bloques pedidos por el cliente
fn send_block(
    data_message: Vec<u8>,
    stream: &mut TcpStream,
    logger: Sender<LogMessages>,
    tx_sender: Sender<TransactionMessages>,
) -> Result<(), NodoBitcoinError> {
    let get_data_message = GetDataMessage::deserealize(&data_message)?;
    let hashes = get_data_message.get_hashes();

    let blocks = get_blocks_from_hashes(hashes, tx_sender)?;

    let mut blocks_bytes: Vec<u8> = Vec::new();
    for block in blocks {
        let block_bytes = block.serialize()?;
        blocks_bytes.extend(block_bytes);
    }

    let block_message = make_block(&blocks_bytes)?;

    if stream.write_all(&block_message).is_err() {
        log_error_message(logger, "No se puede enviar el mensaje BLOCK".to_string());
        return Err(NodoBitcoinError::NoSePuedeEscribirLosBytes);
    }
    Ok(())
}

/// Escribe al cliente el mensaje pong a partir del mensaje ping recibido
fn send_pong(
    ping_message: Vec<u8>,
    stream: &mut TcpStream,
    logger: Sender<LogMessages>,
) -> Result<(), NodoBitcoinError> {
    let pong_message = make_pong(&ping_message)?;
    if stream.write_all(&pong_message).is_err() {
        log_error_message(logger, "No se puede enviar el mensaje PONG".to_string());
        return Err(NodoBitcoinError::ErrorEnPing);
    }
    log_info_message(logger, "Pong enviado".to_string());
    Ok(())
}

/// Define la cantidad de timeouts que se permitirá que tenga el servidor al leer
/// para luego enviar el mensaje ping para confirmar si el cliente sigue conectado
fn max_time_outs() -> i32 {
    let ping_frequency_minutes = match config::get_valor("PING_FREQUENCY_MINUTES".to_string()) {
        Ok(res) => res,
        Err(_) => "2".to_string(),
    };

    let ping_frequency_minutes = ping_frequency_minutes.parse::<u64>().unwrap_or(2);
    let max_time_outs = ping_frequency_minutes * (60 / READ_TIMEOUT_SECONDS);
    max_time_outs as i32
}

/// Lee el mensaje recibido del cliente
/// Si el error en la lectura es WouldBlock (timeout), entonces si se llegó a la máxima
/// cantidad de intentos de lectura se envía el mensaje ping para chequear que el
/// cliente siga conectado y devuelve None.
/// Si se lee algo, devuelve el comando del mensaje recibido y el mensaje
fn read_message(
    stream: &mut TcpStream,
    logger: Sender<LogMessages>,
    send_ping_on_timeout: bool,
) -> Result<Option<(String, Vec<u8>)>, NodoBitcoinError> {
    let mut buffer = [0u8; 24];
    let len_bytes = match stream.read(&mut buffer) {
        Ok(res) => res,
        Err(error) => {
            if error.kind() == ErrorKind::WouldBlock {
                if send_ping_on_timeout {
                    match send_ping_pong_messages(stream, logger.clone()) {
                        Ok(()) => {
                            log_info_message(logger, "Ping pong válido".to_string());
                            return Ok(None);
                        }
                        Err(_) => {
                            log_error_message(logger, "Ping pong inválido".to_string());
                            return Err(NodoBitcoinError::ErrorEnPing);
                        }
                    }
                }
                return Ok(None);
            } else {
                log_error_message(
                    logger,
                    "Error al leer una solicitud del cliente".to_string(),
                );
                return Err(NodoBitcoinError::ErrorAlLeerSolicitudDelCliente);
            }
        }
    };
    if len_bytes == 0 {
        log_error_message(
            logger,
            "Se cierra la conexión al cliente porque se leyó 0 bytes".to_string(),
        );
        return Err(NodoBitcoinError::ErrorAlLeerSolicitudDelCliente);
    }

    let (command, message) = match check_header(&buffer) {
        Ok((command, payload_len)) => {
            let mut message = vec![0u8; payload_len];
            if stream.read_exact(&mut message).is_err() {
                log_error_message(logger, "Error al leer el payload del cliente".to_string());
                return Err(NodoBitcoinError::ErrorAlLeerSolicitudDelCliente);
            }
            (command, message)
        }
        Err(_) => {
            log_error_message(logger, "Error al parsear el header del cliente".to_string());
            return Ok(None);
        }
    };
    log_info_message(logger, format!("Command recibido: {:?}", command));
    Ok(Some((command, message)))
}

/// Envía el mensaje ping al cliente 
/// Devuelve error si el mensaje que se lee luego no es un pong
fn send_ping_pong_messages(
    stream: &mut TcpStream,
    logger: Sender<LogMessages>,
) -> Result<(), NodoBitcoinError> {
    let ping_nonce = ping_nonce();
    let ping_msg = match make_ping(&ping_nonce) {
        Ok(msg) => msg,
        Err(_) => return Err(NodoBitcoinError::ErrorEnPing),
    };

    _ = stream.write(ping_msg.as_slice());

    let (command, message) = match read_message(stream, logger.clone(), false) {
        Ok(option) => {
            if option.is_none() {
                return Err(NodoBitcoinError::ErrorEnPing);
            }
            let (command, message) = option.unwrap();
            (command, message)
        }
        Err(_) => return Err(NodoBitcoinError::ErrorEnPing),
    };

    if !validar_pong(command, message, ping_nonce, logger.clone()) {
        log_error_message(
            logger,
            "No se recibió el mismo nonce en el pong".to_string(),
        );
        return Err(NodoBitcoinError::ErrorEnPing);
    }
    Ok(())
}

// Devuelve true si el pong es válido, false si no
fn validar_pong(
    command: String,
    message: Vec<u8>,
    ping_nonce: [u8; 8],
    logger: Sender<LogMessages>,
) -> bool {
    if command != "pong" {
        log_error_message(logger, "No se recibió un pong".to_string());
        return false;
    }
    if message.len() != 8 {
        log_error_message(logger, "El pong no tiene 8 bytes".to_string());
        return false;
    }

    let pong_nonce = &message[0..8];

    if pong_nonce != ping_nonce {
        log_error_message(
            logger,
            "El nonce del pong no coincide con el del ping".to_string(),
        );
        return false;
    }
    log_info_message(logger, "Nonce del ping pong válido".to_string());
    true
}

#[cfg(test)]
mod tests {

    use super::*;

    fn _init_config() {
        let args: Vec<String> = vec!["app_name".to_string(), "src/nodo.conf".to_string()];
        _ = config::inicializar(args);
    }

    fn _init_client() -> Result<TcpStream, NodoBitcoinError> {
        let port = match config::get_valor("PORT".to_owned()) {
            Ok(res) => res,
            Err(_) => "18333".to_owned(),
        };

        let address = "127.0.0.1:".to_owned() + &port;
        let socket = match TcpStream::connect(address) {
            Ok(res) => res,
            Err(_) => return Err(NodoBitcoinError::NoSePudoConectar),
        };
        Ok(socket)
    }
}
