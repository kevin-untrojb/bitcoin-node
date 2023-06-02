use crate::errores::NodoBitcoinError;

use super::admin_connections::AdminConnections;

pub fn init_block_broadcasting(admin_connections: AdminConnections) -> Result<(), NodoBitcoinError> {
    for connection in admin_connections.clone().get_connections()? {
        let mut socket = connection.clone();
        thread::spawn(move || {
            loop {
                let mut buffer = [u8; 24];
                socket.read_exact_message(&buffer);
                let (_command, headers) = match check_header(&buffer) {
                    Ok((command, payload_len)) => {
                        let mut header = vec![0u8; payload_len];
                        connection.read_exact_message(&mut header)?;

                        (command, header)
                    }
                    Err(NodoBitcoinError::MagicNumberIncorrecto) => {
                        continue;
                    }
                    Err(_) => continue,
                };

                if command == "headers" {
                    let header = deserealize_sin_guardar(header)?;
                    let hash_header = match header.hash() {
                        Ok(res) => res,
                        Err(_) => {
                            println!("Error al calcular el hash del header.");
                            return;
                        }
                    };

                    let get_data = GetDataMessage::new(
                        1,
                        header,
                    );

                    let get_data_message = match get_data.serialize() {
                        Ok(res) => res,
                        Err(_) => {
                            println!("Error al serializar el get_data. Reintentando ...");
                            continue;
                        }
                    };

                    let get_data_message = match get_data.serialize() {
                        Ok(res) => res,
                        Err(_) => {
                            println!("Error al serializar el get_data. Reintentando ...");
                            continue;
                        }
                    };

                    let writed_message = connection.write_message(&get_data_message);
                }
            }
        });
    }

    Ok(())
}