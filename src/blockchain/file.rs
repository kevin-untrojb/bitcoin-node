use crate::blockchain::index::dump_hash_in_the_index;
use std::{
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    mem,
    path::Path,
};

use crate::{config, errores::NodoBitcoinError};

use super::blockheader::BlockHeader;

/// Devuelve el nombre del archivo de heades guardado en el config
/// 
pub fn get_headers_filename() -> Result<String, NodoBitcoinError> {
    config::get_valor("NOMBRE_ARCHIVO_HEADERS".to_string())
}

/// Devuelve el nombre del archivo de bloques guardado en el config
pub fn get_blocks_filename() -> Result<String, NodoBitcoinError> {
    config::get_valor("NOMBRE_ARCHIVO_BLOQUES".to_string())
}

/// Lee todos los bytes de los bloques del archivo y los guarda en un vector
/// Devuelve el vector de bytes de los bloques
pub fn leer_todos_blocks() -> Result<Vec<Vec<u8>>, NodoBitcoinError> {
    let mut todos = vec![];
    let mut offset = 0;
    let block_file_len = get_file_blocks_size()?;
    while offset < block_file_len {
        let (bytes, new_offset) = leer_bloque(offset)?;
        todos.push(bytes);
        offset = new_offset;
    }
    Ok(todos)
}

/// Recibe el path del archivo donde escribir y los datos a escribir
/// Escribe los datos en el archivo
pub fn escribir_archivo(path: String, datos: &[u8]) -> Result<u64, NodoBitcoinError> {
    let mut archivo = match OpenOptions::new()
        .create(true)
        .append(true)
        .open(path.clone())
    {
        Ok(archivo) => archivo,
        Err(error) => {
            println!("Error en avbrir archivo  linea 41 {}", error);
            return Err(NodoBitcoinError::NoExisteArchivo);
        }
    };
    let actual_file_size = get_file_size(path.clone())?;
    archivo
        .write_all(datos)
        .map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;
    Ok(actual_file_size + 1)
}

/// Recibe el path del archivo donde escribir y los datos del bloque a escribir
/// Escribe el bloque en el archivo, primero el tamaño del bloque (porque varía) y luego los bytes del bloque
pub fn escribir_archivo_bloque(path: String, datos: &[u8]) -> Result<(), NodoBitcoinError> {
    let mut archivo = match OpenOptions::new().create(true).append(true).open(path) {
        Ok(archivo) => archivo,
        Err(_) => return Err(NodoBitcoinError::NoExisteArchivo),
    };

    let datos_len = datos.len();
    let datos_len_bytes: [u8; 8] = datos_len.to_ne_bytes();

    let mut datos_con_len = datos_len_bytes.to_vec();
    datos_con_len.extend_from_slice(datos);
    let bytes_para_guardar = datos_con_len.as_slice();

    archivo
        .write_all(bytes_para_guardar)
        .map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;
    Ok(())
}

/// Lee del archivo de headeres el último header guardado y devuelve sus bytes
pub fn leer_ultimo_header() -> Result<Vec<u8>, NodoBitcoinError> {
    let cantidad_headers = header_count()?;
    leer_header_desde_archivo(cantidad_headers - 1)
}

/// Devuelve true si existe el archivo de headers, false si no
pub fn existe_archivo_headers() -> bool {
    let path = match get_headers_filename() {
        Ok(path) => path,
        Err(_) => return false,
    };
    Path::new(&path).exists()
}

/// Recibe el path del archivo a leer, desde donde leer (offser) y cuánto leer (length)
/// Devuelve el vector de los bytes leídos
pub fn leer_bytes(path: String, offset: u64, length: u64) -> Result<Vec<u8>, NodoBitcoinError> {
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(_) => return Err(NodoBitcoinError::NoExisteArchivo),
    };
    let new_position = file.seek(SeekFrom::Start(offset));
    if new_position.is_err() {
        return Err(NodoBitcoinError::NoSePuedeLeerLosBytes);
    }
    let mut buffer = vec![0; length as usize];
    match file.read_exact(&mut buffer) {
        Ok(readed) => readed,
        Err(_) => return Err(NodoBitcoinError::NoSePuedeLeerLosBytes),
    };
    Ok(buffer)
}

/// Recibe desde dónde leer
/// Devuelve los bytes del bloque leído y la cantidad de bytes leídos
pub fn leer_bloque(offset: u64) -> Result<(Vec<u8>, u64), NodoBitcoinError> {
    let path = get_blocks_filename()?;
    let sizeof_usize = mem::size_of::<usize>() as u64;
    let from_file = leer_bytes(path.clone(), offset, sizeof_usize)?;
    let len_bytes: [u8; 8] = match from_file.as_slice().try_into() {
        Ok(bytes) => bytes,
        Err(_) => return Err(NodoBitcoinError::NoSePuedeLeerLosBytes),
    };
    let len_block = usize::from_ne_bytes(len_bytes);
    let block_bytes = leer_bytes(path, offset + sizeof_usize, len_block as u64)?;
    Ok((block_bytes, offset + sizeof_usize + len_block as u64))
}

/// Recibe el path del archivo del cual se quiere saber el tamaño
/// Devuelve, si existe el archivo, el tamaño del mismo
pub fn get_file_size(path: String) -> Result<u64, NodoBitcoinError> {
    let file = File::open(path);
    if file.is_err() {
        return Err(NodoBitcoinError::NoExisteArchivo);
    }
    let file = file.unwrap();
    let metadata = file.metadata();
    if metadata.is_err() {
        return Err(NodoBitcoinError::NoSePuedeLeerLosBytes);
    }
    Ok(metadata.unwrap().len())
}

/// Devuelve el tamaño del archivo de headers
pub fn get_file_header_size() -> Result<u64, NodoBitcoinError> {
    let path = get_headers_filename()?;
    get_file_size(path)
}

/// Devuelve el tamaño del archivo de bloques
fn get_file_blocks_size() -> Result<u64, NodoBitcoinError> {
    let path = get_blocks_filename()?;
    get_file_size(path)
}

/// Recibe el index del header que se quiere leer
/// Devuelve los bytes del header leído
pub fn leer_header_desde_archivo(index: u64) -> Result<Vec<u8>, NodoBitcoinError> {
    let path = get_headers_filename()?;
    let offset = index * 80;
    leer_bytes(path, offset, 80)
}

/// Recibe la cantidad de bloques a leer
/// Devuelve los bytes de los bloques leídos
/// Función usada en tests de bloques
pub fn _leer_algunos_blocks(cantidad: u32) -> Result<Vec<Vec<u8>>, NodoBitcoinError> {
    let mut algunos = vec![];
    let mut offset = 0;
    let block_file_len = get_file_blocks_size()?;
    let mut i = 0;
    while offset < block_file_len && i < cantidad {
        let (bytes, new_offset) = leer_bloque(offset)?;
        algunos.push(bytes);
        offset = new_offset;
        i += 1;
    }
    Ok(algunos)
}

/// Lee el primer bloque del archivo y devuelve sus bytes
/// Función usada en tests
pub fn _leer_primer_block() -> Result<Vec<u8>, NodoBitcoinError> {
    let (bytes, _) = leer_bloque(0)?;
    Ok(bytes)
}

/// Devuelve la cantidad de headers que hay en el archivo
pub fn header_count() -> Result<u64, NodoBitcoinError> {
    let file_size = get_file_header_size()?;
    Ok(file_size / 80)
}

/// Devuelve los bytes de todos los headers del archivo
/// Función usada en tests
pub fn _leer_todos_headers() -> Result<Vec<u8>, NodoBitcoinError> {
    let path = get_headers_filename()?;
    let file_size = get_file_header_size()?;
    leer_bytes(path, 0, file_size)
}

/// Crea los índices del archivo de headers
pub fn _create_all_indexes() -> Result<(), NodoBitcoinError> {
    let path = get_headers_filename()?;
    let file_size = get_file_header_size()?;
    let mut offset = 0;
    while offset < file_size {
        let bytes = leer_bytes(path.clone(), offset, 80)?;
        let header = BlockHeader::deserialize(bytes.as_slice())?;
        let hash = header.hash()?;

        _ = dump_hash_in_the_index(path.clone(), hash, offset);
        offset += 80;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::config;

    use super::_create_all_indexes;

    fn init_config() {
        let args: Vec<String> = vec!["app_name".to_string(), "src/nodo.conf".to_string()];
        _ = config::inicializar(args);
    }

    #[test]
    fn test_create_all_indexes() {
        init_config();
        let _ = _create_all_indexes();
    }
}
