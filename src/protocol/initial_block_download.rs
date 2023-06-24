use super::admin_connections::{AdminConnections, Connection};
use super::connection::connect;
use crate::blockchain::block::SerializedBlock;
use crate::blockchain::blockheader::BlockHeader;
use crate::blockchain::file::{
    escribir_archivo, escribir_archivo_bloque, existe_archivo_headers, leer_ultimo_header,
};
use crate::common::utils_timestamp::{timestamp_to_datetime, obtener_timestamp_dia};
use crate::config;
use crate::errores::NodoBitcoinError;
use crate::log::{log_error_message, log_info_message, LogMessages};
use crate::messages::getdata::GetDataMessage;
use crate::messages::getheaders::GetHeadersMessage;
use crate::messages::headers::deserealize_sin_guardar;
use crate::messages::messages_header::check_header;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::{cmp, thread, vec};

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
            let last_file_header = leer_ultimo_header()?;
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
    let get_headers_message = get_headers.serialize()?;
    Ok(get_headers_message)
}

fn write_header_message_new_connection(
    mut admin_connections: AdminConnections,
) -> Result<(Connection, i32), NodoBitcoinError> {
    let get_headers_message = get_headers_message()?;
    let (connection, id) = admin_connections.find_free_connection()?;
    connection.write_message(&get_headers_message)?;
    Ok((connection, id))
}

fn write_header_message_old_connection(connection: &Connection) -> Result<(), NodoBitcoinError> {
    let get_headers_message = get_headers_message()?;
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
    logger: Sender<LogMessages>,
    mut admin_connections: AdminConnections,
) -> Result<(AdminConnections, (Connection, i32)), NodoBitcoinError> {
    let result = match admin_connections.find_free_connection() {
        Ok(res) => res,
        Err(_) => {
            log_error_message(logger.clone(), "No se encuentra conexion libre".to_string());
            admin_connections = connect(logger)?; // actualizo la lista de conexiones
            admin_connections.find_free_connection()?
        }
    };
    Ok((admin_connections, result))
}

fn read_bytes_header(
    logger: Sender<LogMessages>,
    connection: &Connection,
    admin_connections: AdminConnections,
    intento: usize,
) -> Result<[u8; 24], NodoBitcoinError> {
    let mut buffer = [0u8; 24];
    let bytes_read_option = connection.read_message(&mut buffer)?;
    match bytes_read_option {
        Some(read_bytes) => {
            if read_bytes > 0 {
                return Ok(buffer);
            } else if intento < total_reintentos() {
                log_info_message(logger.clone(), "Reintentando leer header".to_string());
                let (admin_connections, (connection, _id)) =
                    buscar_conexion_libre_o_nuevas_conexiones(logger.clone(), admin_connections)?;
                return read_bytes_header(logger, &connection, admin_connections, intento + 1);
            };
            log_error_message(
                logger,
                "Máximo de reintentos alcanzado ... probá más tarde ...".to_string(),
            );
            Err(NodoBitcoinError::NoSePuedeLeerLosBytes)
        }
        None => read_bytes_header(logger, connection, admin_connections, intento + 1),
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

fn get_headers_filtrados(
    logger: Sender<LogMessages>,
    blockheaders: &Vec<BlockHeader>,
) -> Vec<BlockHeader> {
    if blockheaders.is_empty() {
        log_info_message(logger, "No hay bloques para descargar".to_string());
        return vec![];
    }

    let timestamp_ini = get_timestamp_inicial();
    let headers_filtrados: Vec<_> = blockheaders
        .clone()
        .into_iter()
        .filter(|header| header.time >= timestamp_ini)
        .collect();
    let last_header = blockheaders[blockheaders.len() - 1];
    let datetime = timestamp_to_datetime(last_header.time.into());
    log_info_message(
        logger,
        format!(
            "Descarga de headers. Total: {:?}. Bloques a descargar: {:?}. Ultimo timestamp: {:?}",
            blockheaders.len(),
            headers_filtrados.len(),
            datetime.format("%Y-%m-%d %H:%M").to_string()
        ),
    );
    headers_filtrados
}

const _DEFAULT_TOTAL_THREADS: usize = 5;

fn _get_config_threads() -> usize {
    let total_reintentos_config = config::get_valor("CANTIDAD_THREADS".to_string());
    if let Ok(valor_string) = total_reintentos_config {
        if let Ok(value) = valor_string.parse::<usize>() {
            return value;
        };
    };
    _DEFAULT_TOTAL_THREADS
}

fn headers_by_threads(headers_filtrados: &Vec<BlockHeader>) -> Vec<Vec<BlockHeader>> {
    if headers_filtrados.is_empty() {
        return vec![];
    }

    let n_threads_max = _get_config_threads();
    let len_response = cmp::min(n_threads_max, headers_filtrados.len());

    let n_blockheaders_by_thread =
        (headers_filtrados.len() as f64 / len_response as f64).ceil() as usize;

    let mut response = vec![];
    for i in 0..len_response {
        let start: usize = i * n_blockheaders_by_thread;
        let end: usize = start + n_blockheaders_by_thread;
        if start < headers_filtrados.len() {
            let block_headers_thread =
                headers_filtrados[start..cmp::min(end, headers_filtrados.len())].to_vec();
            response.push(block_headers_thread);
        }
    }
    response
}

fn get_mutex_connection_id(
    logger: Sender<LogMessages>,
    admin_connections: &Arc<Mutex<AdminConnections>>,
) -> Result<(Connection, i32), NodoBitcoinError> {
    match admin_connections.lock() {
        Ok(mut admin) => {
            let (thread_connection, thread_id_connection) = match admin.find_free_connection() {
                Ok((connection, id)) => (connection, id),
                Err(_) => {
                    log_error_message(logger, "Error al buscar conexiones libres.".to_string());
                    return Err(NodoBitcoinError::NoSeEncuentraConexionLibre);
                }
            };
            drop(admin);
            Ok((thread_connection, thread_id_connection))
        }
        Err(_) => {
            log_error_message(logger, "Error al lockear el Mutex.".to_string());
            Err(NodoBitcoinError::NoSeEncuentraConexionLibre)
        }
    }
}

fn change_mutex_connection_id(
    logger: Sender<LogMessages>,
    previous_id: i32,
    admin_connections_mutex_thread: &Arc<Mutex<AdminConnections>>,
    intento: usize,
) -> Result<(Connection, i32), NodoBitcoinError> {
    if intento > total_reintentos() {
        log_info_message(
            logger,
            "Máximo de reintentos alcanzado ... probá más tarde ...".to_string(),
        );
        return Err(NodoBitcoinError::NoSeEncuentraConexionLibre);
    }
    match admin_connections_mutex_thread.lock() {
        Ok(mut admin) => {
            let (thread_connection, thread_id_connection) =
                match admin.change_connection(previous_id) {
                    Ok((connection, id)) => (connection, id),
                    Err(_) => change_mutex_connection_id(
                        logger,
                        previous_id,
                        admin_connections_mutex_thread,
                        intento + 1,
                    )?,
                };
            drop(admin);
            Ok((thread_connection, thread_id_connection))
        }
        Err(_) => {
            log_error_message(logger, "Error al lockear el Mutex.".to_string());
            Err(NodoBitcoinError::NoSeEncuentraConexionLibre)
        }
    }
}

fn write_data_message_new_connection(
    logger: Sender<LogMessages>,
    data_message: &[u8],
    old_connection_id: i32,
    admin_connections_mutex_thread: &Arc<Mutex<AdminConnections>>,
    intento: usize,
) -> Result<(Connection, i32), NodoBitcoinError> {
    let (connection, new_connection_id) = change_mutex_connection_id(
        logger.clone(),
        old_connection_id,
        admin_connections_mutex_thread,
        intento + 1,
    )?;
    write_bytes_data(
        logger,
        data_message,
        connection,
        new_connection_id,
        admin_connections_mutex_thread,
        intento + 1,
    )
}

fn read_bytes_data(
    logger: Sender<LogMessages>,
    connection: &Connection,
) -> Result<[u8; 24], NodoBitcoinError> {
    let mut thread_buffer = [0u8; 24];
    let thread_bytes_read_result = connection.read_message(&mut thread_buffer);
    match thread_bytes_read_result {
        Ok(thread_bytes_read_option) => match thread_bytes_read_option {
            Some(read_bytes) => {
                if read_bytes == 0 {
                    return Err(NodoBitcoinError::NoSePuedeLeerLosBytes);
                }
                Ok(thread_buffer)
            }
            None => Err(NodoBitcoinError::NoSePuedeLeerLosBytes),
        },
        Err(_) => {
            log_error_message(logger, format!("Error al leer mensaje {:?}", thread_buffer));
            Err(NodoBitcoinError::NoSePuedeLeerLosBytes)
        }
    }
}

fn write_bytes_data(
    logger: Sender<LogMessages>,
    data_message: &[u8],
    mut connection: Connection,
    mut connection_id: i32,
    admin_connections: &Arc<Mutex<AdminConnections>>,
    intento: usize,
) -> Result<(Connection, i32), NodoBitcoinError> {
    if intento == total_reintentos() {
        return Err(NodoBitcoinError::NoSeEncuentraConexionLibre);
    }
    let writed_connection = &connection.write_message(data_message);
    if writed_connection.is_err() {
        log_error_message(
            logger.clone(),
            "Error al escribir el mensaje get_data".to_string(),
        );
        (connection, _) = change_mutex_connection_id(
            logger.clone(),
            connection_id,
            admin_connections,
            intento + 1,
        )?;
        (connection, connection_id) = write_bytes_data(
            logger,
            data_message,
            connection,
            connection_id,
            admin_connections,
            intento + 1,
        )?;
    }
    Ok((connection, connection_id))
}

fn get_data_message(header: BlockHeader) -> Result<Vec<u8>, NodoBitcoinError> {
    let hash_header = header.hash()?;
    let get_data = GetDataMessage::new(1, hash_header);
    let get_data_message = get_data.serialize()?;
    Ok(get_data_message)
}

macro_rules! unwrap_or_return {
    ( $e:expr ) => {
        match $e {
            Ok(x) => x,
            Err(_) => return,
        }
    };
}

macro_rules! unwrap_or_continue {
    ( $e:expr ) => {
        match $e {
            Ok(x) => x,
            Err(_) => continue,
        }
    };
}

fn thread_data(
    logger: Sender<LogMessages>,
    shared_blocks: Arc<Mutex<Vec<SerializedBlock>>>,
    admin_connections_mutex_thread: Arc<Mutex<AdminConnections>>,
    block_headers_thread: Vec<BlockHeader>,
    headers_filtrados_len: usize,
) {
    let (mut cloned_connection, mut thread_id_connection) = unwrap_or_return!(
        get_mutex_connection_id(logger.clone(), &admin_connections_mutex_thread)
    );

    for header in block_headers_thread {
        let get_data_message = unwrap_or_continue!(get_data_message(header));
        (cloned_connection, thread_id_connection) = unwrap_or_return!(write_bytes_data(
            logger.clone(),
            &get_data_message,
            cloned_connection,
            thread_id_connection,
            &admin_connections_mutex_thread,
            0
        ));

        loop {
            let thread_buffer_result = read_bytes_data(logger.clone(), &cloned_connection);
            if thread_buffer_result.is_err() {
                (cloned_connection, thread_id_connection) = unwrap_or_return!(write_bytes_data(
                    logger.clone(),
                    &get_data_message,
                    cloned_connection,
                    thread_id_connection,
                    &admin_connections_mutex_thread,
                    0
                ));
                continue;
            }
            let thread_buffer = thread_buffer_result.unwrap();

            let valid_command: bool;
            let (_command, response_get_data) = match check_header(&thread_buffer) {
                Ok((command, payload_len)) => {
                    let mut response_get_data = vec![0u8; payload_len];

                    if cloned_connection
                        .read_exact_message(&mut response_get_data)
                        .is_err()
                    {
                        log_info_message(logger, "Error al leer el mensaje".to_string());
                        return;
                    }
                    valid_command = command == "block";
                    (command, response_get_data)
                }
                Err(_) => {
                    (cloned_connection, thread_id_connection) =
                        unwrap_or_return!(write_data_message_new_connection(
                            logger.clone(),
                            &get_data_message,
                            thread_id_connection,
                            &admin_connections_mutex_thread,
                            0
                        ));
                    continue;
                }
            };

            if valid_command {
                let cloned_result = shared_blocks.lock();
                if cloned_result.is_err() {
                    log_info_message(logger, "Error al lockear el vector de bloques".to_string());
                    return;
                }
                let mut cloned = cloned_result.unwrap();
                let block = match SerializedBlock::deserialize(&response_get_data) {
                    Ok(block) => block,
                    Err(_) => {
                        log_error_message(
                            logger.clone(),
                            format!("Error al deserializar el bloque {:?}", response_get_data),
                        );
                        continue;
                    }
                };
                cloned.push(block);
                progress_bar(headers_filtrados_len, cloned.len());
                drop(cloned);
                break;
            }
        }
    }
    liberar_conexion(logger, thread_id_connection, admin_connections_mutex_thread);
}

pub fn get_full_blockchain(
    logger: mpsc::Sender<LogMessages>,
    admin_connections: AdminConnections,
) -> Result<(), NodoBitcoinError> {
    log_info_message(logger.clone(), "Obteniendo blockchain completa".to_string());
    log_info_message(
        logger.clone(),
        format!(
            "Comienza la descarga a las {}",
            chrono::offset::Local::now().format("%F %T")
        ),
    );

    let (mut connection, mut _id) = write_header_message_new_connection(admin_connections.clone())?;

    let mut reintentos: usize = 0;

    loop {
        let buffer = read_bytes_header(logger.clone(), &connection, admin_connections.clone(), 0)?;

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
                (connection, _id) = write_header_message_new_connection(admin_connections.clone())?;
                continue;
            }
            Err(_) => continue,
        };

        if valid_command {
            let blockheaders = deserealize_sin_guardar(headers)?;
            let headers_filtrados = get_headers_filtrados(logger.clone(), &blockheaders);
            let headers_filtrados_len = headers_filtrados.len();
            let headers_by_threads = headers_by_threads(&headers_filtrados);

            let blocks = Arc::new(Mutex::new(vec![]));
            let mut threads = vec![];

            let admin_connections_mutex = Arc::new(Mutex::new(admin_connections.clone()));

            for block_headers_thread in headers_by_threads {
                let shared_blocks = blocks.clone();
                let admin_connections_mutex_thread = admin_connections_mutex.clone();
                let thread_logger = logger.clone();
                threads.push(thread::spawn(move || {
                    thread_data(
                        thread_logger.clone(),
                        shared_blocks,
                        admin_connections_mutex_thread,
                        block_headers_thread,
                        headers_filtrados_len,
                    );
                }));
            }

            for thread in threads {
                let _ = thread.join();
            }

            // guardar bloques
            blocks_joined_guardar(
                logger.clone(),
                &blocks,
                headers_filtrados_len,
                blockheaders,
                reintentos,
            )?;
            reintentos += 1;
            write_header_message_old_connection(&connection)?;
        }
    }

    let fecha_actual = chrono::offset::Local::now();
    log_info_message(
        logger,
        format!(
            "Finalizada la descarga a las {}",
            fecha_actual.format("%F %T")
        ),
    );

    Ok(())
}

fn liberar_conexion(
    logger: Sender<LogMessages>,
    thread_id_connection: i32,
    admin_connections_mutex_thread: Arc<Mutex<AdminConnections>>,
) {
    match admin_connections_mutex_thread.lock() {
        Ok(mut admin) => {
            match admin.free_connection(thread_id_connection) {
                Ok(()) => (),
                Err(_) => {
                    log_info_message(logger, "Error al liberar la conexión ...".to_string());
                    return;
                }
            };
            drop(admin);
        }
        Err(_) => {
            log_error_message(logger, "Error al lockear el Mutex.".to_string());
        }
    };
}

fn blocks_joined_guardar(
    logger: Sender<LogMessages>,
    blocks: &Arc<Mutex<Vec<SerializedBlock>>>,
    headers_filtrados_len: usize,
    blockheaders: Vec<BlockHeader>,
    intento: usize,
) -> Result<(), NodoBitcoinError> {
    // guardar bloques
    // Convertir Result<MutexGuard<Vec<SerializedBlock>>, _> a Vec<SerializedBlock>
    let bloques_a_guardar = match blocks.lock() {
        Ok(mutex_guard) => mutex_guard.clone(),
        Err(_) => return Err(NodoBitcoinError::NoSePuedeEscribirLosBytes),
    };

    if !bloques_a_guardar.is_empty() {
        log_info_message(
            logger.clone(),
            format!("Bloques descargados: {:?}", bloques_a_guardar.len()),
        );
    };

    if bloques_a_guardar.len() == headers_filtrados_len {
        // guardo los headers y los bloques
        guardar_headers_y_bloques(logger, bloques_a_guardar, blockheaders)?;
    } else {
        if intento > total_reintentos() {
            log_info_message(logger,"Ya se reintentó muchas veces ... dejamos descarsar un rato que después pruebo otra vez ... ".to_string());
            return Err(NodoBitcoinError::NoSePuedeLeerLosBytes);
        }
        log_info_message(
            logger,
            "No se descargaron todos los bloques, reintentando ...".to_string(),
        );
    }
    Ok(())
}

fn guardar_headers_y_bloques(
    logger: Sender<LogMessages>,
    mut bloques_a_guardar: Vec<SerializedBlock>,
    blockheaders: Vec<BlockHeader>,
) -> Result<(), NodoBitcoinError> {
    log_info_message(logger.clone(), "Guardando headers...".to_string());
    for bh in blockheaders {
        let bytes = bh.serialize()?;
        escribir_archivo(&bytes)?;
    }
    log_info_message(logger.clone(), "Headers guardados".to_string());

    // guardo los bloques
    if !bloques_a_guardar.is_empty() {
        log_info_message(logger.clone(), "Guardando bloques...".to_string());
        bloques_a_guardar.sort();
        for bloque in bloques_a_guardar {
            // guardar bloque
            escribir_archivo_bloque(&bloque.serialize()?)?;
        }
        log_info_message(logger, "Bloques guardados".to_string());
    }
    Ok(())
}

fn progress_bar(total: usize, actual: usize) {
    let completado = ((actual as f32 / total as f32) * 50.0) as usize;
    let barra_completado = "#".repeat(completado);
    let barra_no_completado = ".".repeat(50 - completado);
    eprint!(
        "\rDescargando bloques[{}{}] {:?}/{:?}. ",
        barra_completado, barra_no_completado, actual, total
    );
}
