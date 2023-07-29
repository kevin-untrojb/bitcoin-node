use std::{
    io::{ErrorKind, Read, Write},
    net::{TcpListener, TcpStream},
    sync::mpsc::Sender,
    thread::{self},
    time::Duration,
};

use chrono::Utc;

use crate::{
    common::utils_bytes::ping_nonce,
    config,
    errores::NodoBitcoinError,
    log::{log_error_message, log_info_message, LogMessages},
    messages::{
        messages_header::{check_header, make_header},
        ping_pong::{get_nonce, make_ping, make_pong},
        version::VersionMessage,
    },
};

pub fn init_server(logger: Sender<LogMessages>) -> Result<(), NodoBitcoinError> {
    let port = config::get_valor("PORT".to_owned())?;

    let address = "127.0.0.1:".to_owned() + &port;
    server_run(&address, logger).unwrap();
    Ok(())
}

fn server_run(address: &str, logger: Sender<LogMessages>) -> Result<(), NodoBitcoinError> {
    let listener = match TcpListener::bind(address) {
        Ok(res) => res,
        Err(_) => {
            log_error_message(logger.clone(), "Error al bindear el socket".to_string());
            return Err(NodoBitcoinError::NoSePudoConectar);
        }
    };

    if listener.local_addr().is_err() {
        log_error_message(
            logger.clone(),
            "Error al obtener la dirección local".to_string(),
        );
        return Err(NodoBitcoinError::NoSePudoConectar);
    }

    log_info_message(
        logger.clone(),
        format!("Escuchando en: {:?}", listener.local_addr().unwrap()),
    );

    loop {
        match listener.accept() {
            Ok((mut stream, socket_addr)) => {
                log_info_message(
                    logger.clone(),
                    format!("Conexión establecida: {:?}", socket_addr),
                );
                let logger_cloned = logger.clone();
                thread::spawn(move || {
                    handle_message(&mut stream, logger_cloned);
                });
            }
            Err(_) => {
                log_error_message(logger.clone(), "Error al aceptar la conexión".to_string());
                continue;
            }
        }
    }
    Ok(())
}

fn shakehand(stream: &mut TcpStream) -> Result<(), NodoBitcoinError> {
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

    // chequear que la version sea como la nuestra para mandar el version nuestro, sino abortar
    // despues hago el deserealize del version para esto

    let timestamp = Utc::now().timestamp() as u64;
    let version = match (config::get_valor("VERSION".to_string())?).parse::<u32>() {
        // sacamos del config la version??
        Ok(res) => res,
        Err(_) => return Err(NodoBitcoinError::ErrorEnHandshake),
    };

    let version_message = VersionMessage::new(version, timestamp, stream.peer_addr().unwrap()); //LIMPIAR EL UNWRAP
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

fn handle_message(stream: &mut TcpStream, logger: Sender<LogMessages>) {
    // handshake al revés
    // read version
    // write version
    // read verack
    // write verack

    // si es ok el handshake, se hace el loop de recibir mensajes
    // si no, se cierra el socket
    // loop
    // si el read sale por timeout, enviamos un ping, si no responde pong se cierra el socket
    // si el read no lee bytes, se cierra el socket
    // si el read lee bytes, se hace el parseo del mensaje

    // los mensajes que podemos recibir son:
    // - ping
    // - pong
    // - getheaders
    // - getdata

    // los mensajes que podemos enviar son:
    // - ping
    // - pong
    // - headers
    // - block

    let duration = stream.set_read_timeout(Some(Duration::new(120, 0)));
    if duration.is_err() {
        log_error_message(logger.clone(), "Error al setear read timeout.".to_string());
    }

    match shakehand(stream) {
        Ok(()) => {
            // salio bien el handshake, ponerse a escuchar
            log_info_message(
                logger.clone(),
                "Handshake exitoso con el cliente".to_string(),
            );
            thread_connection(stream, logger.clone());
        }
        Err(_) => return,
    };

    // prueba inicial
    // loop {
    //     let mut buffer_read = [0 as u8; 100];
    //     let leidos = stream.read(&mut buffer_read).unwrap();
    //     if leidos == 0 {
    //         break;
    //     }
    //     println!("Recibido: {:?}", buffer_read);
    // }
}

fn thread_connection(stream: &mut TcpStream, logger: Sender<LogMessages>) {
    let client_address = match stream.peer_addr() {
        Ok(res) => res,
        Err(_) => return,
    };
    log_info_message(
        logger.clone(),
        format!("Comienzo a escuchar mensajes de: {:?}", client_address),
    );

    loop {
        let (command, message) = match read_message(stream, logger.clone()) {
            Ok(option) => {
                if option.is_none() {
                    continue;
                }
                let (command, message) = option.unwrap();
                (command, message)
            }
            Err(_) => return,
        };
        if command == "ping" {
            match send_pong(message, stream, logger.clone()) {
                Ok(()) => continue,
                Err(_) => return,
            }
        }
        if command == "getheaders" {
            log_info_message(logger.clone(), "getheaders recibido".to_string());
        }
        if command == "getdata" {
            log_info_message(logger.clone(), "getdata recibido".to_string());
        }
    }
}

fn send_pong(
    ping_message: Vec<u8>,
    stream: &mut TcpStream,
    logger: Sender<LogMessages>,
) -> Result<(), NodoBitcoinError> {
    let nonce = get_nonce(&ping_message)?;
    let pong_message = make_pong(&nonce)?;
    if stream.write_all(&pong_message).is_err() {
        return Err(NodoBitcoinError::ErrorEnPing);
    }
    log_info_message(logger.clone(), "Pong enviado".to_string());
    Ok(())
}

fn read_message(
    stream: &mut TcpStream,
    logger: Sender<LogMessages>,
) -> Result<Option<(String, Vec<u8>)>, NodoBitcoinError> {
    let mut buffer = [0 as u8; 24];
    let len_bytes = match stream.read(&mut buffer) {
        Ok(res) => res,
        Err(error) => {
            if error.kind() == ErrorKind::WouldBlock {
                // Error de Timeout, enviamos un ping
                log_error_message(
                    logger.clone(),
                    "Error de timeout al leer una solicitud del cliente".to_string(),
                );
                match send_ping_pong_messages(stream, logger.clone()) {
                    Ok(()) => return Ok(None),
                    Err(_) => return Err(NodoBitcoinError::ErrorEnPing),
                }
            } else {
                log_error_message(
                    logger.clone(),
                    "Error al leer una solicitud del cliente".to_string(),
                );
                return Err(NodoBitcoinError::ErrorAlLeerSolicitudDelCliente);
            }
        }
    };
    if len_bytes == 0 {
        log_error_message(
            logger.clone(),
            "Se cierra la conexión al cliente porque se leyó 0 bytes".to_string(),
        );
        return Err(NodoBitcoinError::ErrorAlLeerSolicitudDelCliente);
    }

    // verifico los tipos de mensajes aceptados
    let (command, message) = match check_header(&buffer) {
        Ok((command, payload_len)) => {
            let mut message = vec![0u8; payload_len];
            if stream.read_exact(&mut message).is_err() {
                log_error_message(
                    logger.clone(),
                    "Error al leer el payload del cliente".to_string(),
                );
                return Err(NodoBitcoinError::ErrorAlLeerSolicitudDelCliente);
            }
            (command, message)
        }
        Err(_) => {
            log_error_message(
                logger.clone(),
                "Error al parsear el header del cliente".to_string(),
            );
            return Ok(None);
        }
    };
    log_info_message(logger.clone(), format!("Command recibido: {:?}", command));
    Ok(Some((command, message)))
}

fn send_ping_pong_messages(
    stream: &mut TcpStream,
    logger: Sender<LogMessages>,
) -> Result<(), NodoBitcoinError> {
    let ping_nonce = ping_nonce();
    let ping_msg = match make_ping(&ping_nonce) {
        Ok(msg) => msg,
        Err(_) => return Err(NodoBitcoinError::ErrorEnPing),
    };

    _ = stream.write(&ping_msg.as_slice());

    let (command, message) = match read_message(stream, logger.clone()) {
        Ok(option) => {
            if option.is_none() {
                return Err(NodoBitcoinError::ErrorEnPing);
            }
            let (command, message) = option.unwrap();
            (command, message)
        }
        Err(_) => return Err(NodoBitcoinError::ErrorEnPing),
    };

    if !validar_pong(command, message, ping_nonce, logger) {
        return Err(NodoBitcoinError::ErrorEnPing);
    }
    Ok(())
}

// crear una función que valide el pong
fn validar_pong(
    command: String,
    message: Vec<u8>,
    ping_nonce: [u8; 8],
    logger: Sender<LogMessages>,
) -> bool {
    if command != "pong" {
        log_error_message(logger.clone(), "No se recibió un pong".to_string());
        return false;
    }
    let pong_nonce = match get_nonce(&message) {
        Ok(nonce) => nonce,
        Err(_) => {
            log_error_message(logger.clone(), "No se pudo parsear el pong".to_string());
            return false;
        }
    };
    if pong_nonce != ping_nonce {
        log_error_message(
            logger.clone(),
            "El nonce del pong no coincide con el del ping".to_string(),
        );
        return false;
    }
    true
}

#[cfg(test)]
mod tests {
    use std::net::{Ipv4Addr, SocketAddrV4};

    use crate::{log::create_logger_actor, protocol::connection::handshake};

    use super::*;

    fn init_config() {
        let args: Vec<String> = vec!["app_name".to_string(), "src/nodo.conf".to_string()];
        let init_result = config::inicializar(args);
    }

    fn init_client() -> Result<TcpStream, NodoBitcoinError> {
        let port = match config::get_valor("PORT".to_owned()) {
            Ok(res) => res,
            Err(_) => "18333".to_owned(),
        };

        let address = "127.0.0.1:".to_owned() + &port;
        let mut socket = match TcpStream::connect(address) {
            Ok(res) => res,
            Err(_) => return Err(NodoBitcoinError::NoSePudoConectar),
        };
        Ok(socket)
    }

    #[test]
    fn test_run_server() {
        init_config();
        let logger = create_logger_actor(config::get_valor("LOG_FILE".to_string()));
        init_server(logger);
    }

    #[test]
    fn test_run_client() {
        init_config();
        let socket = init_client();
        assert!(socket.is_ok());

        let socket = socket.unwrap();
        let address = socket.local_addr();
        assert!(address.is_ok());
        let address = address.unwrap();

        let handsahke = handshake(socket, address);
        assert!(handsahke.is_ok());

        let mut socket = handsahke.unwrap();

        let logger = create_logger_actor(config::get_valor("LOG_FILE".to_string()));
        let ping_pong = send_ping_pong_messages(&mut socket, logger);
        assert!(ping_pong.is_ok());
    }
}
