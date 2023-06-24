use crate::errores::NodoBitcoinError;
use std::{io::Write, mem};

/// Guarda en el archivo recibido el tamaño del string recibido 
/// seguido de ese mismo string, ambos codificados por motivos de seguridad
pub fn save_encoded_len_bytes(file: &mut dyn Write, data: String) -> Result<(), NodoBitcoinError> {
    let encoded = bs58::encode(data.as_bytes()).into_string();
    let len = encoded.len();
    match file.write_all(&len.to_ne_bytes()) {
        Ok(_) => {}
        Err(_) => {
            return Err(NodoBitcoinError::NoSePuedeEscribirLosBytes);
        }
    };
    match file.write_all(encoded.as_bytes()) {
        Ok(_) => {}
        Err(_) => {
            return Err(NodoBitcoinError::NoSePuedeEscribirLosBytes);
        }
    };
    Ok(())
}

/// Lee información codificada de un archivo, la decodifica y devuelve
pub fn read_decoded_string_offset(
    buffer: Vec<u8>,
    offset: u64,
) -> Result<(String, u64), NodoBitcoinError> {
    let sizeof_usize = mem::size_of::<usize>() as u64;
    let len_bytes: [u8; 8] = match leer_bytes(buffer.clone(), offset, sizeof_usize)?
        .as_slice()
        .try_into()
    {
        Ok(bytes) => bytes,
        Err(_) => return Err(NodoBitcoinError::NoSePuedeLeerLosBytes),
    };
    let len_string = usize::from_ne_bytes(len_bytes);
    let string_bytes = leer_bytes(buffer, offset + sizeof_usize, len_string as u64)?;
    let string_readed = String::from_utf8(string_bytes);
    if string_readed.is_err() {
        return Err(NodoBitcoinError::NoSePuedeLeerLosBytes);
    }
    let string_readed = string_readed.unwrap();

    // Decodificar el string codificado
    let decoded = match bs58::decode(&string_readed).into_vec() {
        Ok(bytes) => bytes,
        Err(_) => return Err(NodoBitcoinError::NoSePuedeLeerLosBytes),
    };
    let decoded_string = match String::from_utf8(decoded) {
        Ok(string) => string,
        Err(_) => return Err(NodoBitcoinError::NoSePuedeLeerLosBytes),
    };

    Ok((decoded_string, offset + sizeof_usize + len_string as u64))
}

fn leer_bytes(buffer: Vec<u8>, offset: u64, length: u64) -> Result<Vec<u8>, NodoBitcoinError> {
    let mut bytes = vec![0; length as usize];
    for i in 0..length {
        bytes[i as usize] = buffer[(offset + i) as usize];
    }
    Ok(bytes)
}
