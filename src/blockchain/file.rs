use std::{
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::Path,
};

use crate::{config, errores::NodoBitcoinError};

fn get_headers_filename() -> Result<String, NodoBitcoinError> {
    config::get_valor("NOMBRE_ARCHIVO_HEADERS".to_string())
}

fn get_blocks_filename() -> Result<String, NodoBitcoinError> {
    config::get_valor("NOMBRE_ARCHIVO_BLOQUES".to_string())
}

pub fn _reset_files() -> Result<(), NodoBitcoinError> {
    let path = get_headers_filename()?;
    let _ = std::fs::remove_file(path);
    let path = get_blocks_filename()?;
    let _ = std::fs::remove_file(path);
    Ok(())
}

pub fn existe_archivo_headers() -> bool {
    let path = match get_headers_filename() {
        Ok(path) => path,
        Err(_) => return false,
    };
    Path::new(&path).exists()
}

pub fn escribir_archivo(datos: &[u8]) -> Result<(), NodoBitcoinError> {
    let path = get_headers_filename()?;
    let mut archivo = match OpenOptions::new().create(true).append(true).open(path) {
        Ok(archivo) => archivo,
        Err(_) => return Err(NodoBitcoinError::NoExisteArchivo),
    };

    // Escribe los bytes en el archivo
    archivo
        .write_all(datos)
        .map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;
    Ok(())
}

pub fn escribir_archivo_bloque(datos: &[u8]) -> Result<(), NodoBitcoinError> {
    let path = get_blocks_filename()?;
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

pub fn _leer_ultimo_header() -> Result<Vec<u8>, NodoBitcoinError> {
    //_leer_header(1)
    let cantidad_headers = _header_count()?;
    _leer_header_desde_archivo(cantidad_headers - 1)
}

fn _get_file_header_size() -> Result<u64, NodoBitcoinError> {
    let path = get_headers_filename()?;
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

pub fn _header_count() -> Result<u64, NodoBitcoinError> {
    let file_size = _get_file_header_size()?;
    Ok(file_size / 80)
}

pub fn _leer_header_desde_archivo(index: u64) -> Result<Vec<u8>, NodoBitcoinError> {
    //    _leer_header(total_headers)
    let path = get_headers_filename()?;
    let offset = index * 80;
    leer_bytes(path, offset, 80)
}

pub fn _leer_todos_headers() -> Result<Vec<u8>, NodoBitcoinError> {
    let path = get_headers_filename()?;
    let file_size = _get_file_header_size()?;
    leer_bytes(path, 0, file_size)
}

pub fn _leer_primer_header() -> Result<Vec<u8>, NodoBitcoinError> {
    _leer_header_desde_archivo(0)
}

pub fn _leer_headers(ix: u64) -> Result<Vec<u8>, NodoBitcoinError> {
    // devuelve de a 2000 headers
    let offset: u64 = ix * 2000;
    let length: u64 = 2000 * 80;
    let path = get_headers_filename()?;
    leer_bytes(path, offset, length)
}

fn leer_bytes(path: String, offset: u64, length: u64) -> Result<Vec<u8>, NodoBitcoinError> {
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
