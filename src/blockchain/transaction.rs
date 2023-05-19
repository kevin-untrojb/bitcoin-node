use super::header;
use crate::errores::NodoBitcoinError;
use std::io::Write;

/// A struct representing a Bitcoin transaction
/// ### Bitcoin Core References
/// https://developer.bitcoin.org/reference/transactions.html
///
/// # Fields
///
/// * version - The version number of the transaction.
/// * input - The vector of input transactions for the transaction.
/// * output - The vector of output transactions for the transaction.
/// * lock_time - The lock time for the transaction.
pub struct _Transaction {
    version: i32,
    input: Vec<_TxIn>,
    output: Vec<TxOut>,
    lock_time: u64,
}

/// A struct representing an input transaction for a Bitcoin transaction
///
/// # Fields
///
/// * previous_output - The outpoint from the previous transaction that this input is spending.
/// * script_bytes - The number of bytes in the signature script.
/// * signature_script - The signature script for the input.
/// * sequence - The sequence number for the input.
struct _TxIn {
    previous_output: _Outpoint,
    script_bytes: usize,
    signature_script: String,
    sequence: u32,
}

/// A struct representing an outpoint from a previous transaction
///
/// # Fields
///
/// * hash - The transaction hash of the previous transaction.
/// * index - The index of the output in the previous transaction.
struct _Outpoint {
    hash: String,
    index: u32,
}

/// A struct representing an output transaction for a Bitcoin transaction
///
/// # Fields
///
/// * value - The value of the output in satoshis.
/// * pk_script - The public key script for the output.
struct TxOut {
    value: u64,
    pk_script: Vec<u8>,
}

impl TxOut {
    pub fn serialize(&self) -> Result<Vec<u8>, NodoBitcoinError> {
        let mut bytes = Vec::new();
        bytes.write_all(&(self.value).to_le_bytes()).map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;
        bytes.write_all(&(self.pk_script.len() as u32).to_le_bytes()).map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;
        bytes.write_all(&self.pk_script).map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;
        Ok(bytes)
    }

    pub fn deserialize(block_bytes: &[u8]) -> Result<TxOut, NodoBitcoinError> {
        let mut offset = 0;

        let value = u64::from_le_bytes(block_bytes[offset..offset + 8].try_into().map_err(|_| NodoBitcoinError::NoSePuedeLeerLosBytes)?);
        offset += 8;

        let pk_script_length = u32::from_le_bytes(block_bytes[offset..offset + 4].try_into().map_err(|_| NodoBitcoinError::NoSePuedeLeerLosBytes)?);
        offset += 4;

        let mut pk_script = vec![0u8; pk_script_length as usize];
        pk_script.copy_from_slice(&block_bytes[offset..offset + pk_script_length as usize]);

        Ok(TxOut {
            value,
            pk_script,
        })
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_TxOut() {
        let txout = TxOut {
            value: 123,
            pk_script: vec![1, 2, 3, 4, 5],
        };

        let serialized = txout.serialize().unwrap();

        assert_eq!(serialized, vec![123, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 0, 1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_deserialize_TxOut() {
        let serialized = vec![
            123, 0, 0, 0, 0, 0, 0, 0, // Valor
            5, 0, 0, 0, //  pk_script
            1, 2, 3, 4, 5, // pk_script
        ];

        let txout = TxOut::deserialize(&serialized).unwrap();

        assert_eq!(txout.value, 123);
        assert_eq!(txout.pk_script, vec![1, 2, 3, 4, 5]);
    }

}

