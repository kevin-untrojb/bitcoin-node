use crate::blockchain::index::dump_hash_in_the_index;
use std::{
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    mem,
    path::Path,
};

use crate::{config, errores::NodoBitcoinError};

use super::blockheader::BlockHeader;

// usos: initial_block_broadcasting, file_manager
pub fn get_headers_filename() -> Result<String, NodoBitcoinError> {
    config::get_valor("NOMBRE_ARCHIVO_HEADERS".to_string())
}

pub fn get_blocks_filename() -> Result<String, NodoBitcoinError> {
    config::get_valor("NOMBRE_ARCHIVO_BLOQUES".to_string())
}

// block.rs lo utiliza
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

// usos: initial_block_broadcasting, file_manager
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

pub fn escribir_archivo_bloque(path: String, datos: &[u8]) -> Result<(), NodoBitcoinError> {
    let mut archivo = match OpenOptions::new().create(true).append(true).open(path) {
        Ok(archivo) => archivo,
        Err(_) => return Err(NodoBitcoinError::NoExisteArchivo),
    };

    let datos_len = datos.len();
    let datos_len_bytes: [u8; 8] = datos_len.to_ne_bytes();
    // necesito un array de bytes que concatene los datos_len_bytes y los datos
    let mut datos_con_len = datos_len_bytes.to_vec();
    datos_con_len.extend_from_slice(datos);
    let bytes_para_guardar = datos_con_len.as_slice();

    // Escribe los bytes en el archivo
    archivo
        .write_all(bytes_para_guardar)
        .map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;
    Ok(())
}

pub fn leer_ultimo_header() -> Result<Vec<u8>, NodoBitcoinError> {
    let cantidad_headers = header_count()?;
    leer_header_desde_archivo(cantidad_headers - 1)
}

pub fn existe_archivo_headers() -> bool {
    let path = match get_headers_filename() {
        Ok(path) => path,
        Err(_) => return false,
    };
    Path::new(&path).exists()
}

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

///////////////////////////////////////////////////////////////////
// ************************  internas ************************  //
/////////////////////////////////////////////////////////////////

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
fn get_file_size(path: String) -> Result<u64, NodoBitcoinError> {
    // TODO roberto
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

pub fn get_file_header_size() -> Result<u64, NodoBitcoinError> {
    let path = get_headers_filename()?;
    get_file_size(path)
}

fn get_file_blocks_size() -> Result<u64, NodoBitcoinError> {
    let path = get_blocks_filename()?;
    get_file_size(path)
}

pub fn leer_header_desde_archivo(index: u64) -> Result<Vec<u8>, NodoBitcoinError> {
    let path = get_headers_filename()?;
    let offset = index * 80;
    leer_bytes(path, offset, 80)
}

/////////////////////////////////////////////////////////////////
// ************************ Sin Uso ************************  //
///////////////////////////////////////////////////////////////

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

pub fn _leer_primer_block() -> Result<Vec<u8>, NodoBitcoinError> {
    let (bytes, _) = leer_bloque(0)?;
    Ok(bytes)
}

pub fn _reset_files() -> Result<(), NodoBitcoinError> {
    let path = get_headers_filename()?;
    let _ = std::fs::remove_file(path);
    let path = get_blocks_filename()?;
    let _ = std::fs::remove_file(path);
    Ok(())
}

pub fn header_count() -> Result<u64, NodoBitcoinError> {
    let file_size = get_file_header_size()?;
    Ok(file_size / 80)
}

pub fn _leer_todos_headers() -> Result<Vec<u8>, NodoBitcoinError> {
    let path = get_headers_filename()?;
    let file_size = get_file_header_size()?;
    leer_bytes(path, 0, file_size)
}

// leer el archivo de headers de a 80 bytes
pub fn buscar_header(hash_buscado: [u8; 32]) -> Result<Vec<u8>, NodoBitcoinError> {
    let path = get_headers_filename()?;
    let file_size = get_file_header_size()?;
    let mut offset = 0;
    while offset < file_size {
        let bytes = leer_bytes(path.clone(), offset, 80)?;
        let header = BlockHeader::deserialize(bytes.as_slice())?;
        let hash = header.hash()?;
        offset += 80;
        if hash == hash_buscado {
            break;
        }
    }
    // devolver los 80 * 2000 bytes siguientes al offset, o hasta el final del archivo
    let length = 80 * 2000;
    // valor menor entre leght + offset y file_size
    let length = if length + offset < file_size {
        length
    } else {
        file_size - offset
    };
    let bytes = leer_bytes(path, offset, length)?;
    Ok(bytes)
}

pub fn leer_primeros_2mil_headers() -> Result<Vec<u8>, NodoBitcoinError> {
    let path = get_headers_filename()?;
    //let file_size = get_file_header_size()?;
    leer_bytes(path, 0, 2000 * 80)
}

pub fn _leer_primer_header() -> Result<Vec<u8>, NodoBitcoinError> {
    leer_header_desde_archivo(0)
}

pub fn _leer_headers(ix: u64) -> Result<Vec<u8>, NodoBitcoinError> {
    // devuelve de a 2000 headers
    let offset: u64 = ix * 2000;
    let length: u64 = 2000 * 80;
    let path = get_headers_filename()?;
    leer_bytes(path, offset, length)
}

pub fn create_all_indexes() -> Result<(), NodoBitcoinError> {
    let path = get_headers_filename()?;
    let file_size = get_file_header_size()?;
    let mut offset = 0;
    while offset < file_size {
        let bytes = leer_bytes(path.clone(), offset, 80)?;
        let header = BlockHeader::deserialize(bytes.as_slice())?;
        let hash = header.hash()?;

        dump_hash_in_the_index(path.clone(), hash, offset);
        offset += 80;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::config;

    use super::create_all_indexes;

    fn init_config() {
        let args: Vec<String> = vec!["app_name".to_string(), "src/nodo.conf".to_string()];
        _ = config::inicializar(args);
    }

    #[test]
    fn test_create_all_indexes() {
        init_config();
        let _ = create_all_indexes();
    }
}
