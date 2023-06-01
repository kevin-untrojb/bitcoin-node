use crate::blockchain::block::SerializedBlock;
use crate::blockchain::blockheader::BlockHeader;
use crate::blockchain::file::{
    _leer_ultimo_header, escribir_archivo, escribir_archivo_bloque, existe_archivo_headers,
    reset_files,
};
use crate::common::utils_timestamp::{_timestamp_to_datetime, obtener_timestamp_dia};
use crate::config;
use crate::errores::NodoBitcoinError;
use crate::messages::getdata::GetDataMessage;
use crate::messages::getheaders::GetHeadersMessage;
use crate::messages::headers::{deserealize, deserealize_sin_guardar};
use crate::messages::messages_header::check_header;
use std::hash::Hash;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::{cmp, println, thread, vec};

use super::admin_connections::{AdminConnections, Connection};
use super::connection::connect;

pub const GENESIS_BLOCK: [u8; 32] = [
    0x00, 0x00, 0x00, 0x00, 0x09, 0x33, 0xea, 0x01, 0xad, 0x0e, 0xe9, 0x84, 0x20, 0x97, 0x79, 0xba,
    0xae, 0xc3, 0xce, 0xd9, 0x0f, 0xa3, 0xf4, 0x08, 0x71, 0x95, 0x26, 0xf8, 0xd7, 0x7f, 0x49, 0x43,
];

pub fn _version() -> Result<u32, NodoBitcoinError> {
    let version = match (config::get_valor("VERSION".to_string())?).parse::<u32>() {
        Ok(res) => res,
        Err(_) => return Err(NodoBitcoinError::NoSePuedeLeerValorDeArchivoConfig),
    };
    Ok(version)
}

fn start_block() -> Result<[u8; 32], NodoBitcoinError> {
    let existe_header = existe_archivo_headers();
    let start_block = match existe_header {
        true => {
            let last_file_header = _leer_ultimo_header()?;
            let header_serialized = BlockHeader::deserialize(&last_file_header)?;
            header_serialized.hash()?
        }
        false => GENESIS_BLOCK,
    };
    Ok(start_block)
}

fn get_headers_message() -> Result<Vec<u8>, NodoBitcoinError> {
    let version = _version()?;
    let start_block = start_block()?;
    let get_headers = GetHeadersMessage::new(version, 1, start_block, [0; 32]);
    let mut get_headers_message = get_headers.serialize()?;
    Ok(get_headers_message)
}

fn write_header_message_new_connection(
    mut admin_connections: AdminConnections,
) -> Result<(Connection, i32), NodoBitcoinError> {
    let mut get_headers_message = get_headers_message()?;
    let (mut connection, mut id) = admin_connections.find_free_connection()?;
    connection.write_message(&get_headers_message)?;
    Ok((connection, id))
}

fn write_header_message_old_connection(connection: &Connection) -> Result<(), NodoBitcoinError> {
    let mut get_headers_message = get_headers_message()?;
    connection.write_message(&get_headers_message)
}

const DEFAULT_TOTAL_REINTEGROS: usize = 5;

fn total_reintentos() -> usize {
    let total_reintentos_config = config::get_valor("REINTENTOS_DESCARGA_BLOQUES".to_string());
    if let Ok(valor_string) = total_reintentos_config {
        if let Ok(value) = valor_string.parse::<usize>() {
            return value;
        };
    };
    DEFAULT_TOTAL_REINTEGROS
}

fn buscar_conexion_libre_o_nuevas_conexiones(
    mut admin_connections: AdminConnections,
) -> Result<(AdminConnections, (Connection, i32)), NodoBitcoinError> {
    let result = match admin_connections.find_free_connection() {
        Ok(res) => res,
        Err(_) => {
            println!("No se encuentra conexion libre");
            admin_connections = connect()?; // actualizo la lista de conexiones
            admin_connections.find_free_connection()?
        }
    };
    Ok((admin_connections, result))
}

fn read_bytes_header(
    connection: &Connection,
    mut admin_connections: AdminConnections,
    intento: usize,
) -> Result<[u8; 24], NodoBitcoinError> {
    let mut buffer = [0u8; 24];
    let bytes_read_option = connection.read_message(&mut buffer)?;
    match bytes_read_option {
        Some(read_bytes) => {
            if read_bytes > 0 {
                return Ok(buffer);
            } else {
                if intento < total_reintentos() {
                    println!("Reintentando leer header");
                    let (admin_connections, (connection, id)) =
                        buscar_conexion_libre_o_nuevas_conexiones(admin_connections)?;
                    return read_bytes_header(&connection, admin_connections, intento + 1);
                } else {
                    println!("Máximo de reintentos alcanzado ... probá más tarde ...");
                    return Err(NodoBitcoinError::NoSePuedeLeerLosBytes);
                }
            }
        }
        None => return read_bytes_header(connection, admin_connections, intento + 1),
    }
}

fn get_timestamp_inicial() -> u32 {
    let fecha_inicial_result = config::get_valor("DIA_INICIAL".to_string());
    if fecha_inicial_result.is_err() {
        return 0;
    }
    let fecha_inicial = fecha_inicial_result.unwrap();
    obtener_timestamp_dia(fecha_inicial)
}

fn get_headers_filtrados(blockheaders: &Vec<BlockHeader>) -> Vec<BlockHeader> {
    if blockheaders.len() == 0 {
        println!("No hay bloques para descargar");
        return vec![];
    }

    let timestamp_ini = get_timestamp_inicial();
    let headers_filtrados: Vec<_> = blockheaders
        .clone()
        .into_iter()
        .filter(|header| header.time >= timestamp_ini)
        .collect();
    let last_header = headers_filtrados[headers_filtrados.len() - 1];
    let datetime = _timestamp_to_datetime(last_header.time.into());
    println!(
        "Descarga de headers. Total: {:?}. Bloques a descargar: {:?}. Ultimo timestamp: {:?}",
        blockheaders.len(),
        headers_filtrados.len(),
        datetime.format("%Y-%m-%d %H:%M").to_string()
    );
    headers_filtrados
}

const DEFAULT_TOTAL_THREADS: usize = 5;

fn get_config_threads() -> usize {
    let total_reintentos_config = config::get_valor("CANTIDAD_THREADS".to_string());
    if let Ok(valor_string) = total_reintentos_config {
        if let Ok(value) = valor_string.parse::<usize>() {
            return value;
        };
    };
    DEFAULT_TOTAL_THREADS
}

fn headers_by_threads(headers_filtrados: Vec<BlockHeader>) -> Vec<Vec<BlockHeader>> {
    if headers_filtrados.len() == 0 {
        return vec![];
    }

    let n_threads_max = get_config_threads();
    let len_response = cmp::min(n_threads_max, headers_filtrados.len());

    let n_blockheaders_by_thread =
        (headers_filtrados.len() as f64 / len_response as f64).ceil() as usize;

    let mut response = vec![];
    for i in 0..len_response {
        let start: usize = i * n_blockheaders_by_thread;
        let end: usize = start + n_blockheaders_by_thread;
        let block_headers_thread =
            headers_filtrados[start..cmp::min(end, headers_filtrados.len())].to_vec();
        response.push(block_headers_thread);
    }
    response
}

pub fn get_full_blockchain(
    mut admin_connections: AdminConnections,
) -> Result<(), NodoBitcoinError> {
    println!("Obteniendo blockchain completa");
    println!(
        "Comienza la descarga a las {}",
        chrono::offset::Local::now().format("%F %T").to_string()
    );

    let (mut connection, mut id) = write_header_message_new_connection(admin_connections.clone())?;

    let mut reintentos: usize = 0;

    loop {
        eprint!("Leyendo siguiente header...");
        let mut buffer = read_bytes_header(&connection, admin_connections.clone(), 0)?;

        let valid_command: bool;
        let (_command, headers) = match check_header(&buffer) {
            Ok((command, payload_len)) => {
                let mut headers = vec![0u8; payload_len];
                connection.read_exact_message(&mut headers)?;
                valid_command = command == "headers";
                if valid_command && payload_len == 1 {
                    break; // llegué al final de los headers
                }
                (command, headers)
            }
            Err(NodoBitcoinError::MagicNumberIncorrecto) => {
                (connection, id) = write_header_message_new_connection(admin_connections.clone())?;
                continue;
            }
            Err(_) => continue,
        };

        if valid_command {
            let blockheaders = deserealize_sin_guardar(headers)?;
            let headers_filtrados = get_headers_filtrados(&blockheaders);
            let headers_filtrados_len = headers_filtrados.len();

            let n_threads_max: usize =
                match (config::get_valor("CANTIDAD_THREADS".to_string())?).parse::<usize>() {
                    Ok(res) => res,
                    Err(_) => return Err(NodoBitcoinError::NoSePuedeLeerValorDeArchivoConfig),
                };
            let n_threads = cmp::min(n_threads_max, headers_filtrados.len());
            let n_blockheaders_thread =
                (headers_filtrados.len() as f64 / n_threads as f64).ceil() as usize;
            let blocks = Arc::new(Mutex::new(vec![]));
            let mut threads = vec![];

            let admin_connections_mutex = Arc::new(Mutex::new(admin_connections.clone()));

            for i in 0..n_threads {
                let start: usize = i * n_blockheaders_thread;
                let end: usize = start + n_blockheaders_thread;
                let block_headers_thread =
                    headers_filtrados[start..cmp::min(end, headers_filtrados.len())].to_vec();
                let shared_blocks = blocks.clone();
                let admin_connections_mutex_thread = admin_connections_mutex.clone();
                threads.push(thread::spawn(move || {
                    let (mut cloned_connection, mut thread_id_connection) =
                        match admin_connections_mutex_thread.lock() {
                            Ok(mut admin) => {
                                let (thread_connection, thread_id_connection) =
                                    match admin.find_free_connection() {
                                        Ok((connection, id)) => (connection, id),
                                        Err(_) => {
                                            println!("Error al buscar conexiones libres.");
                                            return;
                                        }
                                    };
                                drop(admin);
                                (thread_connection, thread_id_connection)
                            }
                            Err(_) => {
                                println!("Error al lockear el Mutex.");
                                return;
                            }
                        };

                    for header in block_headers_thread {
                        let hash_header = match header.hash() {
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

                        for j in 0..total_reintentos() + 1 {
                            let writed_message = cloned_connection.write_message(&get_data_message);
                            if writed_message.is_err() {
                                {
                                    println!("Error al enviar mensaje de get_data");
                                    if j == total_reintentos() {
                                        println!(
                                            "Se realizaron todos los reintentos ... {:?}",
                                            get_data_message
                                        );
                                        return;
                                    }

                                    (cloned_connection, thread_id_connection) =
                                        match admin_connections_mutex_thread.lock() {
                                            Ok(mut admin) => {
                                                let (thread_connection, thread_id_connection) =
                                                    match admin
                                                        .change_connection(thread_id_connection)
                                                    {
                                                        Ok((connection, id)) => (connection, id),
                                                        Err(_) => continue,
                                                    };
                                                drop(admin);
                                                (thread_connection.clone(), thread_id_connection)
                                            }
                                            Err(_) => {
                                                println!("Error al lockear el Mutex.");
                                                return;
                                            }
                                        };
                                    println!("Se cambió la conexión. Reintento {:?} ...", j);
                                    continue;
                                };
                            }
                            break;
                        }
                        let mut intento_actual = 0;
                        loop {
                            let mut change_connection: bool = false;
                            let mut thread_buffer = [0u8; 24];

                            let thread_bytes_read_result =
                                cloned_connection.read_message(&mut thread_buffer);
                            match thread_bytes_read_result {
                                Ok(thread_bytes_read_option) => match thread_bytes_read_option {
                                    Some(read_bytes) => {
                                        if read_bytes == 0 {
                                            intento_actual += 1;
                                            change_connection = true;
                                        } else {
                                            intento_actual = 0;
                                        }
                                    }
                                    None => {
                                        intento_actual += 1;
                                        change_connection = true;
                                    },
                                },
                                Err(_) => {
                                    println!("Error al leer mensaje {:?}", thread_buffer);
                                    return;
                                }
                            }
                            if change_connection {
                                if intento_actual == total_reintentos() {
                                    println!("Se realizaron todos los reintentos ...");
                                    return;
                                }
                                println!("Se va a cambiar la conexión. Reintento {:?} ...", intento_actual);
                                (cloned_connection, thread_id_connection) =
                                    match admin_connections_mutex_thread.lock() {
                                        Ok(mut admin) => {
                                            let (thread_connection, thread_id_connection) =
                                                match admin.change_connection(thread_id_connection)
                                                {
                                                    Ok((connection, id)) => (connection, id),
                                                    Err(_) => continue,
                                                };
                                            drop(admin);
                                            (thread_connection.clone(), thread_id_connection)
                                        }
                                        Err(_) => {
                                            println!("Error al lockear el Mutex.");
                                            return;
                                        }
                                    };
                                if cloned_connection.write_message(&get_data_message).is_err() {
                                    println!("Error al escribir el mensaje");
                                    return;
                                }
                                continue;
                            }

                            let valid_command: bool;
                            let (_command, response_get_data) = match check_header(&thread_buffer) {
                                Ok((command, payload_len)) => {
                                    let mut response_get_data = vec![0u8; payload_len];

                                    if cloned_connection
                                        .read_exact_message(&mut response_get_data)
                                        .is_err()
                                    {
                                        println!("Error al leer el mensaje");
                                        return;
                                    }
                                    valid_command = command == "block";
                                    (command, response_get_data)
                                }
                                Err(NodoBitcoinError::MagicNumberIncorrecto) => {
                                    (cloned_connection, thread_id_connection) =
                                        match admin_connections_mutex_thread.lock() {
                                            Ok(mut admin) => {
                                                let (thread_connection, thread_id_connection) =
                                                    match admin
                                                        .change_connection(thread_id_connection)
                                                    {
                                                        Ok((connection, id)) => (connection, id),
                                                        Err(_) => {
                                                            println!("Error al cambiar de conexión. Reintentando ...");
                                                            continue;
                                                        }
                                                    };
                                                drop(admin);
                                                (thread_connection.clone(), thread_id_connection)
                                            }
                                            Err(_) => {
                                                println!("Error al lockear el Mutex.");
                                                return;
                                            }
                                        };
                                    if cloned_connection.write_message(&get_data_message).is_err() {
                                        println!("Error al escribir el mensaje");
                                        return;
                                    }
                                    continue;
                                }
                                Err(_) => {
                                    println!("Error al chequear el header. Reintentando ...");
                                    continue;
                                }
                            };

                            if valid_command {
                                let cloned_result = shared_blocks.lock();
                                if cloned_result.is_err() {
                                    println!("Error al lockear el vector de bloques");
                                    return;
                                }
                                let mut cloned = cloned_result.unwrap();
                                let block = match SerializedBlock::deserialize(&response_get_data) {
                                    Ok(block) => block,
                                    Err(_) => {
                                        println!(
                                            "Error al deserializar el bloque {:?}",
                                            response_get_data
                                        );
                                        continue;
                                    }
                                };
                                cloned.push(block);
                                let cloned_len = cloned.len();

                                let completado =
                                    ((cloned_len as f32 / headers_filtrados_len as f32) * 50.0)
                                        as usize;
                                let barra_completado = "#".repeat(completado);
                                let barra_no_completado = ".".repeat(50 - completado);
                                eprint!(
                                    "\rDescargando bloques[{}{}] {:?}/{:?}. ",
                                    barra_completado,
                                    barra_no_completado,
                                    cloned_len,
                                    headers_filtrados_len
                                );
                                //println!("Bloque #{} descargado", cloned.len());
                                drop(cloned);
                                break;
                            }
                        }
                    }
                    match admin_connections_mutex_thread.lock() {
                        Ok(mut admin) => {
                            let _ =
                            match admin
                                .free_connection(thread_id_connection)
                            {
                                Ok(()) => (),
                                Err(_) => {
                                    println!("Error al liberar la conexión ...");
                                    return;
                                }
                            };
                        drop(admin);
                        }
                        Err(_) => {
                            println!("Error al lockear el Mutex.");
                            return;
                        }
                    };
                }));
            }

            for thread in threads {
                let _ = thread.join();
            }

            // guardar bloques
            // Convertir Result<MutexGuard<Vec<SerializedBlock>>, _> a Vec<SerializedBlock>
            let mut bloques_a_guardar = match blocks.lock() {
                Ok(mutex_guard) => mutex_guard.clone(),
                Err(_) => return Err(NodoBitcoinError::NoSePuedeEscribirLosBytes),
            };

            if bloques_a_guardar.len() > 0 {
                println!("Bloques descargados: {:?}", bloques_a_guardar.len());
            };

            if bloques_a_guardar.len() == headers_filtrados_len {
                // guardo los headers

                eprint!("Guardando headers...");
                for bh in blockheaders {
                    let bytes = bh.serialize()?;
                    _ = escribir_archivo(&bytes)?;
                }
                println!("Headers guardados");

                // guardo los bloques
                if bloques_a_guardar.len() > 0 {
                    eprint!("Guardando bloques...");
                    bloques_a_guardar.sort();
                    for bloque in bloques_a_guardar {
                        // guardar bloque
                        _ = escribir_archivo_bloque(&bloque.block_bytes)?;
                    }
                    println!("Bloques guardados");
                }
            } else {
                reintentos += 1;
                if reintentos > total_reintentos() {
                    println!("Ya se reintentó muchas veces ... dejamos descarsar un rato que después pruebo otra vez ... ");
                    return Err(NodoBitcoinError::NoSePuedeLeerLosBytes);
                }
                println!("No se descargaron todos los bloques, reintentando ...");
            }
            write_header_message_old_connection(&connection)?;
        }
    }

    let fecha_actual = chrono::offset::Local::now();
    println!(
        "Finalizada la descarga a las {}",
        fecha_actual.format("%F %T").to_string()
    );

    Ok(())
}
