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

fn create_path(path: String, hash: [u8; 32]) -> String {
    format!("indexes/{}/ix-{}", path, create_hash_to_find_index(hash))
}
fn get_key_path(path: String) -> String {
    format!("{}-v", path)
}
fn get_val_path(path: String) -> String {
    format!("{}-k", path)
}

fn is_hash_searched(vec: Vec<u8>, slice: &[u8; 32]) -> bool {
    vec.iter().eq(slice.iter())
}

pub fn dump_hash_in_the_index(
    path: String,
    hash: [u8; 32],
    real_index: u64,
) -> Result<(), NodoBitcoinError> {
    let index_path = create_path(path, hash.clone());
    let mut keys_file = match OpenOptions::new()
        .create(true)
        .append(true)
        .open(get_key_path(index_path.clone()))
    {
        Ok(file) => file,
        Err(_) => return Err(NodoBitcoinError::NoExisteArchivo),
    };
    let mut vals_file = match OpenOptions::new()
        .create(true)
        .append(true)
        .open(get_val_path(index_path))
    {
        Ok(file) => file,
        Err(_) => return Err(NodoBitcoinError::NoExisteArchivo),
    };
    keys_file
        .write_all(&hash)
        .map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;
    vals_file
        .write_all(&real_index.to_le_bytes())
        .map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;

    Ok(())
}

pub fn get_start_index(path: String, hash: [u8; 32]) -> Result<usize, NodoBitcoinError> {
    let keys_path = get_key_path(create_path(path.clone(), hash.clone()));
    let vals_path = get_val_path(create_path(path.clone(), hash.clone()));
    let mut keys_file = match File::open(get_key_path(keys_path.clone())) {
        Ok(file) => file,
        Err(_) => return Err(NodoBitcoinError::NoExisteArchivo),
    };

    let mut offset = 0;
    let mut i = 0;
    let mut is_missing_index = true;
    let size_of_u8 = mem::size_of::<u8>() as u64;
    while offset < keys_file.metadata().unwrap().len() && is_missing_index {
        if is_hash_searched(
            leer_bytes(keys_path.clone(), offset, size_of_u8 * 32).unwrap(),
            &hash,
        ) {
            is_missing_index = false
        } else {
            i = i + 1;
            offset = offset + 32;
        }
    }
    if is_missing_index {
        return Err(NodoBitcoinError::IndexNoEncontrado);
    }

    let sizeof_usize = mem::size_of::<usize>() as u64;
    let index_from_file = leer_bytes(vals_path, sizeof_usize * i, sizeof_usize)?;

    let mut array_index: [u8; 8] = [0; 8];
    array_index.copy_from_slice(&index_from_file);

    let index = usize::from_le_bytes(array_index);

    Ok(index)
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
