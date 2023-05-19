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
    input: Vec<TxIn>,
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
struct TxIn {
    previous_output: Outpoint,
    script_bytes: usize,
    signature_script: String,
    sequence: u32,
}

impl TxIn {
    pub fn serialize(&self) -> Result<Vec<u8>, NodoBitcoinError> {
        let mut bytes = Vec::new();
        bytes.write_all(&(self.previous_output.serialize()?)?).map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;
        bytes.write_all(&(self.script_bytes as u32).to_le_bytes()).map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;
        bytes.write_all(self.signature_script.as_bytes()).map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;
        bytes.write_all(&(self.sequence).to_le_bytes()).map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;
        Ok(bytes)
    }

    pub fn deserialize(block_bytes: &[u8]) -> Result<TxIn, NodoBitcoinError> {
        let mut offset = 0;

        let previous_output = Outpoint::deserialize(&block_bytes[offset..])?;
        offset += previous_output.size() as usize;

        let script_bytes = u32::from_le_bytes(block_bytes[offset..offset + 4].try_into().map_err(|_| NodoBitcoinError::NoSePuedeLeerLosBytes)?);
        offset += 4;

        let signature_script = String::from_utf8_lossy(&block_bytes[offset..offset + script_bytes as usize]).into_owned();
        offset += script_bytes as usize;

        let sequence = u32::from_le_bytes(block_bytes[offset..offset + 4].try_into().map_err(|_| NodoBitcoinError::NoSePuedeLeerLosBytes)?);
        offset += 4;

        Ok(TxIn {
            previous_output,
            script_bytes: script_bytes as usize,
            signature_script,
            sequence,
        })
    }
}

/// A struct representing an outpoint from a previous transaction
///
/// # Fields
///
/// * hash - The transaction hash of the previous transaction.
/// * index - The index of the output in the previous transaction.
#[derive(Debug, PartialEq)]
struct Outpoint {
    hash: [u8; 32],
    index: u32,
}
impl Outpoint {
    pub fn serialize(&self) -> Result<Vec<u8>, NodoBitcoinError> {
        let mut bytes = Vec::new();
        bytes.write_all(&self.hash).map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;
        bytes.write_all(&(self.index).to_le_bytes()).map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;
        Ok(bytes)
    }

    pub fn deserialize(block_bytes: &[u8]) -> Result<Outpoint, NodoBitcoinError> {
        let mut offset = 0;

        let mut hash = [0u8; 32];
        hash.copy_from_slice(&block_bytes[offset..offset + 32]);
        offset += 32;

        let index = u32::from_le_bytes(block_bytes[offset..offset + 4].try_into().map_err(|_| NodoBitcoinError::NoSePuedeLeerLosBytes)?);
        offset += 4;

        Ok(Outpoint {
            hash,
            index,
        })
    }
    pub fn size(&self) ->u32{
        self.index
    }
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
    fn test_serialize_outpoint() {
        let outpoint = Outpoint {
            hash: [1u8; 32],
            index: 123,
        };

        let expected_bytes = vec![
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // hash
            123, 0, 0, 0, // index
        ];

        let serialized = outpoint.serialize().unwrap();

        assert_eq!(serialized, expected_bytes);
    }

    #[test]
    fn test_deserialize_outpoint() {
        let bytes = vec![
            2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, // hash
            255, 255, 255, 255, // index
        ];

        let expected_outpoint = Outpoint {
            hash: [2u8; 32],
            index: 4294967295,
        };

        let deserialized = Outpoint::deserialize(&bytes).unwrap();

        assert_eq!(deserialized, expected_outpoint);
    }

    #[test]
    fn test_size_outpoint() {
        let outpoint = Outpoint {
            hash: [3u8; 32],
            index: 456,
        };

        let size = outpoint.size();

        assert_eq!(size, 456);
    }
    #[test]
    fn test_serialize_tx_out() {
        let txout = TxOut {
            value: 123,
            pk_script: vec![1, 2, 3, 4, 5],
        };

        let serialized = txout.serialize().unwrap();

        assert_eq!(serialized, vec![123, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 0, 1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_deserialize_tx_out() {
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

