use crate::blockchain::file::escribir_archivo;
use crate::blockchain::file::leer_bytes;
use crate::errores::NodoBitcoinError;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;
use std::{
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    mem,
    path::Path,
};

fn create_hash_to_find_index(hash: [u8; 32]) -> usize {
    let mut hasher = DefaultHasher::new();
    hash.hash(&mut hasher);
    let hash_value = hasher.finish();
    let range = 1000;
    let balanced_index = (hash_value % range as u64) as usize;

    balanced_index
}

fn create_path(hash: [u8; 32]) -> String {
    format!(
        "src/indexes/headers/ix-{}.bin",
        create_hash_to_find_index(hash)
    )
}

fn is_hash_searched(vec: Vec<u8>, slice: &[u8; 32]) -> bool {
    vec.iter().eq(slice.iter())
}

pub fn dump_hash_in_the_index(
    path: String,
    hash: [u8; 32],
    real_index: u64,
) -> Result<(), NodoBitcoinError> {
    let index_path = create_path(hash.clone());

    if let Err(error) = escribir_archivo(index_path.clone(), &hash) {
        println!(
            "error path de dumpear archivo linea 40 {} {}",
            index_path.clone(),
            error
        );
    }
    if let Err(error) = escribir_archivo(index_path.clone(), &real_index.to_le_bytes()) {
        println!(
            "error path de dumpear archivo linea 43 {} {}",
            index_path.clone(),
            error
        );
    }

    Ok(())
}

pub fn get_start_index(path: String, hash: [u8; 32]) -> Result<u64, NodoBitcoinError> {
    let index_path = create_path(hash.clone());
    let mut file = match File::open(index_path.clone()) {
        Ok(file) => file,
        Err(_) => return Err(NodoBitcoinError::NoExisteArchivo),
    };
    let mut offset = 0;
    let mut i = 0;
    let size_of_u8 = mem::size_of::<u8>() as u64;
    while offset < file.metadata().unwrap().len() {
        if is_hash_searched(
            leer_bytes(index_path.clone(), offset, size_of_u8 * 32).unwrap(),
            &hash,
        ) {
            offset = offset + size_of_u8 * 32;
            let index_found = leer_bytes(index_path.clone(), offset, size_of_u8 * 8)?;
            let mut array_bytes: [u8; 8] = [0; 8];
            array_bytes.copy_from_slice(&index_found);
            // indice encontrado
            let u64_index = u64::from_le_bytes(array_bytes);
            return Ok(u64_index);
        } else {
            i = i + 1;
            offset = offset + 40;
        }
    }
    Err(NodoBitcoinError::IndexNoEncontrado)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_hash_searched() {
        // Caso de prueba: Vec y slice son iguales
        let vec: Vec<u8> = vec![
            1, 2, 3, 4, 5, 1, 2, 3, 4, 5, 1, 2, 3, 4, 5, 1, 2, 3, 4, 5, 1, 2, 3, 4, 5, 1, 2, 3, 4,
            5, 1, 2,
        ];
        let slice: &[u8; 32] = &[
            1, 2, 3, 4, 5, 1, 2, 3, 4, 5, 1, 2, 3, 4, 5, 1, 2, 3, 4, 5, 1, 2, 3, 4, 5, 1, 2, 3, 4,
            5, 1, 2,
        ];
        assert_eq!(is_hash_searched(vec, slice), true);

        // Caso de prueba: Vec y slice son diferentes
        let vec: Vec<u8> = vec![
            1, 2, 3, 4, 5, 1, 2, 3, 4, 5, 1, 2, 3, 4, 5, 1, 2, 3, 4, 5, 1, 2, 3, 4, 5, 1, 2, 3, 4,
            5, 1, 2,
        ];
        let slice: &[u8; 32] = &[
            1, 1, 1, 1, 1, 1, 1, 1, 4, 5, 1, 2, 3, 4, 5, 1, 2, 3, 4, 5, 1, 2, 3, 4, 5, 1, 2, 3, 4,
            5, 1, 2,
        ];
        assert_eq!(is_hash_searched(vec, slice), false);

        // Caso de prueba: Vec y slice tienen diferentes longitudes
        let vec: Vec<u8> = vec![1, 2, 3];
        let slice: &[u8; 32] = &[
            1, 2, 3, 4, 5, 1, 2, 3, 4, 5, 1, 2, 3, 4, 5, 1, 2, 3, 4, 5, 1, 2, 3, 4, 5, 1, 2, 3, 4,
            5, 1, 2,
        ];
        assert_eq!(is_hash_searched(vec, slice), false);
    }
}
