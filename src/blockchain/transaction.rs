use super::blockheader;
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
#[derive(Clone)]
pub struct Transaction {
    version: u32,
    input: Vec<TxIn>,
    output: Vec<TxOut>,
    lock_time: u64,
}

impl Transaction {
    pub fn serialize(&self) -> Result<Vec<u8>,NodoBitcoinError> {
        let mut bytes = Vec::new();
        bytes.write_all(&(self.version).to_le_bytes()).map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;
        bytes.write_all(&(self.input.len() as u32).to_le_bytes()).map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;
        for tx_in in &self.input{
            bytes.write_all(&tx_in.serialize()?);
        }
        bytes.write_all(&(self.output.len() as u32).to_le_bytes()).map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;
        for tx_out in &self.output {
            bytes.write_all(&tx_out.serialize()?);
        }
        bytes.write_all(&self.lock_time.to_le_bytes()).map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;
        Ok(bytes)
    }
    pub fn deserialize(block_bytes: &[u8]) -> Result<Transaction, NodoBitcoinError> {
        let mut offset = 0;

        let version = u32::from_le_bytes(block_bytes[offset..offset + 4].try_into().map_err(|_| NodoBitcoinError::NoSePuedeLeerLosBytes)?);
        offset += 4;

        let number_tx_in = u32::from_le_bytes(block_bytes[offset..offset + 4].try_into().map_err(|_| NodoBitcoinError::NoSePuedeLeerLosBytes)?);
        offset += 4;

        let mut input = Vec::new();
        for _ in 0..number_tx_in {
            let tx_in = TxIn::deserialize(&block_bytes[offset..])?;
            offset += tx_in.size();
            input.push(tx_in);
        }

        let number_tx_out = u32::from_le_bytes(block_bytes[offset..offset + 4].try_into().map_err(|_| NodoBitcoinError::NoSePuedeLeerLosBytes)?);
        offset += 4;

        let mut output = Vec::new();
        for _ in 0..number_tx_out {
            let tx_out = TxOut::deserialize(&block_bytes[offset..])?;
            offset += tx_out.size();
            output.push(tx_out);
        }

        let lock_time = u64::from_le_bytes(block_bytes[offset..offset + 8].try_into().map_err(|_| NodoBitcoinError::NoSePuedeLeerLosBytes)?);
        offset += 8;

        Ok(Transaction {
            version,
            input,
            output,
            lock_time,
        })
    }
    pub fn size(&self) -> usize{
        let input_size = self.input.iter().map(|tx_in| tx_in.size()).sum::<usize>();
        let output_size = self.output.iter().map(|tx_out| tx_out.size()).sum::<usize>();

        20 + input_size + output_size
    }
}

/// A struct representing an input transaction for a Bitcoin transaction
///
/// # Fields
///
/// * previous_output - The outpoint from the previous transaction that this input is spending.
/// * script_bytes - The number of bytes in the signature script.
/// * signature_script - The signature script for the input.
/// * sequence - The sequence number for the input.
#[derive(Debug, PartialEq, Clone)]
struct TxIn {
    pub previous_output: Outpoint,
    pub script_bytes: u32,
    pub signature_script: Vec<u8>,
    pub sequence: u32,
}

impl TxIn {
    pub fn serialize(&self) -> Result<Vec<u8>, NodoBitcoinError> {
        let mut bytes = Vec::new();
        bytes.write_all(&(self.previous_output.serialize()?)).map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;
        bytes.write_all(&(self.script_bytes as u32).to_le_bytes()).map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;
        bytes.write_all(&self.signature_script).map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;
        bytes.write_all(&(self.sequence).to_le_bytes()).map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;
        Ok(bytes)
    }

    pub fn deserialize(block_bytes: &[u8]) -> Result<TxIn, NodoBitcoinError> {
        let mut offset = 0;

        let previous_output = Outpoint::deserialize(&block_bytes[offset..offset+36])?;
        offset += 36;

        let script_bytes = u32::from_le_bytes(block_bytes[offset..offset + 4].try_into().map_err(|_| NodoBitcoinError::NoSePuedeLeerLosBytes)?);
        offset += 4;

        let size = script_bytes as usize;
        let mut signature_script = vec![0u8; size];
        signature_script.copy_from_slice(&block_bytes[offset..offset + size]);
        offset += size;

        let sequence = u32::from_le_bytes(block_bytes[offset..offset + 4].try_into().map_err(|_| NodoBitcoinError::NoSePuedeLeerLosBytes)?);
        offset += 4;

        Ok(TxIn {
            previous_output,
            script_bytes,
            signature_script,
            sequence,
        })
    }
    pub fn size(&self) -> usize{
        (44 + self.script_bytes) as usize
    }
}

/// A struct representing an outpoint from a previous transaction
///
/// # Fields
///
/// * hash - The transaction hash of the previous transaction.
/// * index - The index of the output in the previous transaction.
#[derive(Debug, PartialEq, Clone)]
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
#[derive(Debug, PartialEq, Clone)]
struct TxOut {
    pub value: u64,
    pub pk_len: u32,
    pub pk_script: Vec<u8>,
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

        let pk_len = u32::from_le_bytes(block_bytes[offset..offset + 4].try_into().map_err(|_| NodoBitcoinError::NoSePuedeLeerLosBytes)?);
        offset += 4;

        let mut pk_script = vec![0u8; pk_len as usize];
        pk_script.copy_from_slice(&block_bytes[offset..offset + pk_len as usize]);

        Ok(TxOut {
            value,
            pk_len,
            pk_script,
        })
    }
    pub fn size(&self) -> usize{
        (12 + self.pk_len) as usize
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_transaction() {
        let version = 1;
        let input = vec![
            TxIn {
                previous_output: Outpoint {
                    hash: [1u8; 32],
                    index: 123,
                },
                script_bytes:4,
                signature_script: vec![128, 0, 0, 0],
                sequence:255,
            }
        ];
        let output = vec![
            TxOut {
                value: 123,
                pk_len:5,
                pk_script: vec![1, 2, 3, 4, 5],
            }
        ];
        let lock_time = 0;

        let transaction = Transaction {
            version,
            input,
            output,
            lock_time,
        };

        let expected_bytes = vec![
            1, 0, 0, 0,  // version
            1, 0, 0, 0,  // number_tx_in
            // Datos de input
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // hash
            123, 0, 0, 0,
            4, 0, 0, 0,
            128, 0, 0, 0,
            255, 0, 0, 0,
            // Datos de número de output y output
            1, 0, 0, 0,  // number_tx_out
            // Datos de output
            123, 0, 0, 0, 0, 0, 0, 0, // Valor
            5, 0, 0, 0, //  pk_len
            1, 2, 3, 4, 5, // pk_script
            // Datos de lock_time
            0, 0, 0, 0, 0, 0, 0, 0,  // lock_time
        ];

        let serialized = transaction.serialize().unwrap();

        assert_eq!(serialized, expected_bytes);
    }

    #[test]
    fn test_deserialize_transaction() {
        let block_bytes = vec![
            1, 0, 0, 0,  // version
            1, 0, 0, 0,  // number_tx_in
            // Datos de input
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // hash
            123, 0, 0, 0,
            4, 0, 0, 0,
            128, 0, 0, 0,
            255, 0, 0, 0,
            1, 0, 0, 0,  // number_tx_out
            // Datos de output
            123, 0, 0, 0, 0, 0, 0, 0, // Valor
            5, 0, 0, 0, //  pk_len
            1, 2, 3, 4, 5, // pk_script
            // Datos de lock_time
            0, 0, 0, 0, 0, 0, 0, 0,  // lock_time
        ];

        let transaction = Transaction::deserialize(&block_bytes).unwrap();

        assert_eq!(transaction.version, 1);
        assert_eq!(transaction.input.len(), 1);
        assert_eq!(transaction.input[0], TxIn {
            previous_output: Outpoint {
                hash: [1u8; 32],
                index: 123,
            },
            script_bytes:4,
            signature_script: vec![128, 0, 0, 0],
            sequence:255,
        });
        assert_eq!(transaction.output.len(), 1);
        assert_eq!(transaction.output[0], TxOut {
            value: 123,
            pk_len:5,
            pk_script: vec![1, 2, 3, 4, 5],
        });
        assert_eq!(transaction.lock_time, 0);
    }

    #[test]
    fn test_serialize_and_deserialize_transaction() {
        let version = 1;
        let input = vec![
            TxIn {
                previous_output: Outpoint {
                    hash: [0u8; 32],
                    index: 0,
                },
                script_bytes: 10,
                signature_script: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
                sequence: 100,
            },
            TxIn {
                previous_output: Outpoint {
                    hash: [255u8; 32],
                    index: 1,
                },
                script_bytes: 5,
                signature_script: vec![11, 12, 13, 14, 15],
                sequence: 200,
            },
        ];
        let output = vec![
            TxOut {
                value: 1000,
                pk_len: 20,
                pk_script: vec![16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35],
            },
            TxOut {
                value: 2000,
                pk_len: 15,
                pk_script: vec![36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50],
            },
        ];
        let lock_time = 12345;

        let transaction = Transaction {
            version,
            input: input.clone(),
            output: output.clone(),
            lock_time,
        };
        let serialized = transaction.serialize().unwrap();

        let deserialized = Transaction::deserialize(&serialized).unwrap();

        assert_eq!(deserialized.version, version);
        assert_eq!(deserialized.input, input);
        assert_eq!(deserialized.output, output);
        assert_eq!(deserialized.lock_time, lock_time);
    }
    #[test]
    fn test_transaction_size() {
        let version = 1;
        let input = vec![
            TxIn {
                previous_output: Outpoint {
                    hash: [1u8; 32],
                    index: 123,
                },
                script_bytes:4,
                signature_script: vec![128, 0, 0, 0],
                sequence:255,
            }
        ];
        let output = vec![
            TxOut {
                value: 123,
                pk_len:5,
                pk_script: vec![1, 2, 3, 4, 5],
            }
        ];
        let lock_time = 0;

        let transaction = Transaction {
            version,
            input,
            output,
            lock_time,
        };
        let expected_bytes = vec![
            1, 0, 0, 0,  // version
            1, 0, 0, 0,  // number_tx_in
            // Datos de input
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // hash
            123, 0, 0, 0,
            4, 0, 0, 0,
            128, 0, 0, 0,
            255, 0, 0, 0,
            // Datos de número de output y output
            1, 0, 0, 0,  // number_tx_out
            // Datos de output
            123, 0, 0, 0, 0, 0, 0, 0, // Valor
            5, 0, 0, 0, //  pk_len
            1, 2, 3, 4, 5, // pk_script
            // Datos de lock_time
            0, 0, 0, 0, 0, 0, 0, 0,  // lock_time
        ];
        assert_eq!(transaction.size(), expected_bytes.len());
    }
    #[test]
    fn test_serialize_tx_in() {
        let previous_output = Outpoint {
            hash: [1; 32],
            index: 123,
        };
        let script_bytes = 4;
        let signature_script = vec![128, 0, 0, 0];
        let sequence = 255;

        let tx_in = TxIn {
            previous_output,
            script_bytes,
            signature_script: signature_script.clone(),
            sequence,
        };

        let serialized = tx_in.serialize().unwrap();

        assert_eq!(serialized.len(), 48);
        assert_eq!(serialized[0..32], [1; 32]);
        assert_eq!(u32::from_le_bytes(serialized[32..36].try_into().unwrap()), 123);
        assert_eq!(u32::from_le_bytes(serialized[36..40].try_into().unwrap()), 4);
        assert_eq!(serialized[40..44], [128, 0, 0, 0]);
        assert_eq!(u32::from_le_bytes(serialized[44..48].try_into().unwrap()), 255);
    }

    #[test]
    fn test_deserialize_tx_in() {
        let bytes: [u8; 48] = [
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // hash
            123, 0, 0, 0,
            4, 0, 0, 0, 128, 0, 0, 0,
            255, 0, 0, 0
        ];

        let tx_in = TxIn::deserialize(&bytes).unwrap();

        assert_eq!(tx_in.previous_output.hash, [1u8; 32]);
        assert_eq!(tx_in.previous_output.index, 123);
        assert_eq!(tx_in.script_bytes, 4);
        assert_eq!(tx_in.signature_script, vec![128, 0, 0, 0]);
        assert_eq!(tx_in.sequence, 255);
    }

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
            pk_len:5,
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

