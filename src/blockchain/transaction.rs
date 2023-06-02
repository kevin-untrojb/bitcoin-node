use crate::common::uint256::Uint256;
use crate::common::utils_bytes;
use crate::errores::NodoBitcoinError;
use bitcoin_hashes::{sha256d, Hash};
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
#[warn(dead_code)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Transaction {
    pub version: u32,
    pub input: Vec<TxIn>,
    pub output: Vec<TxOut>,
    pub lock_time: u32,
    pub tx_in_count: usize,
    pub tx_out_count: usize,
}

impl Transaction {
    #[warn(dead_code)]
    pub fn _serialize(&self) -> Result<Vec<u8>, NodoBitcoinError> {
        let mut bytes = Vec::new();
        bytes
            .write_all(&(self.version).to_le_bytes())
            .map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;

        let tx_in_count_prefix = utils_bytes::_from_amount_bytes_to_prefix(self.tx_in_count);
        bytes
            .write_all(&(utils_bytes::_build_varint_bytes(tx_in_count_prefix, self.input.len())?))
            .map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;

        for tx_in in &self.input {
            bytes
                .write_all(&tx_in._serialize()?)
                .map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;
        }

        let tx_out_count_prefix = utils_bytes::_from_amount_bytes_to_prefix(self.tx_out_count);
        bytes
            .write_all(&(utils_bytes::_build_varint_bytes(tx_out_count_prefix, self.output.len())?))
            .map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;
        for tx_out in &self.output {
            bytes
                .write_all(&tx_out._serialize()?)
                .map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;
        }
        bytes
            .write_all(&self.lock_time.to_le_bytes())
            .map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;
        Ok(bytes)
    }
    pub fn deserialize(block_bytes: &[u8]) -> Result<Transaction, NodoBitcoinError> {
        let mut offset = 0;
        let version = u32::from_le_bytes(
            block_bytes[offset..offset + 4]
                .try_into()
                .map_err(|_| NodoBitcoinError::NoSePuedeLeerLosBytes)?,
        );
        offset += 4;
        let (tx_in_count, tx_in_amount) = utils_bytes::parse_varint(&block_bytes[offset..]);
        offset += tx_in_count;

        let mut input = Vec::new();
        for _v in 0..tx_in_amount {
            let tx_in = TxIn::deserialize(&block_bytes[offset..])?;
            offset += tx_in.size();
            input.push(tx_in);
        }

        let (tx_out_count, tx_out_amount) = utils_bytes::parse_varint(&block_bytes[offset..]);
        offset += tx_out_count;

        let mut output = Vec::new();
        for _v in 0..tx_out_amount {
            let tx_out = TxOut::deserialize(&block_bytes[offset..])?;
            offset += tx_out.size();
            output.push(tx_out);
        }

        let lock_time = u32::from_le_bytes(
            block_bytes[offset..offset + 4]
                .try_into()
                .map_err(|_| NodoBitcoinError::NoSePuedeLeerLosBytes)?,
        );
        //offset += 4;
        Ok(Transaction {
            version,
            input,
            output,
            lock_time,
            tx_in_count,
            tx_out_count,
        })
    }
    pub fn size(&self) -> usize {
        let input_size = self.input.iter().map(|tx_in| tx_in.size()).sum::<usize>();
        let output_size = self
            .output
            .iter()
            .map(|tx_out| tx_out.size())
            .sum::<usize>();
        8 + input_size + output_size + self.tx_in_count + self.tx_out_count
    }
    pub fn _txid(&self) -> Result<Uint256, NodoBitcoinError> {
        let bytes = self._serialize()?;
        let hash = sha256d::Hash::hash(&bytes);
        let u256 = Uint256::_from_be_bytes(*hash.as_byte_array());
        Ok(u256)
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
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TxIn {
    pub previous_output: Outpoint,
    pub script_bytes: usize,
    pub signature_script: Vec<u8>,
    pub sequence: u32,
    pub script_bytes_amount: usize,
}

impl TxIn {
    pub fn _serialize(&self) -> Result<Vec<u8>, NodoBitcoinError> {
        let mut bytes = Vec::new();
        bytes
            .write_all(&(self.previous_output._serialize()?))
            .map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;

        let script_bytes_prefix =
            utils_bytes::_from_amount_bytes_to_prefix(self.script_bytes_amount);
        bytes
            .write_all(&(utils_bytes::_build_varint_bytes(script_bytes_prefix, self.script_bytes)?))
            .map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;
        bytes
            .write_all(&self.signature_script)
            .map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;
        bytes
            .write_all(&(self.sequence).to_le_bytes())
            .map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;
        Ok(bytes)
    }

    pub fn deserialize(block_bytes: &[u8]) -> Result<TxIn, NodoBitcoinError> {
        let mut offset = 0;

        let previous_output = Outpoint::deserialize(&block_bytes[offset..offset + 36])?;
        offset += 36;

        let (script_bytes_amount, script_bytes) = utils_bytes::parse_varint(&block_bytes[offset..]);
        offset += script_bytes_amount;

        let mut signature_script = vec![0u8; script_bytes];
        signature_script.copy_from_slice(&block_bytes[offset..offset + script_bytes]);
        offset += script_bytes;

        let sequence = u32::from_le_bytes(
            block_bytes[offset..offset + 4]
                .try_into()
                .map_err(|_| NodoBitcoinError::NoSePuedeLeerLosBytes)?,
        );
        //offset += 4;

        Ok(TxIn {
            previous_output,
            script_bytes,
            signature_script,
            sequence,
            script_bytes_amount,
        })
    }
    pub fn size(&self) -> usize {
        40 + self.script_bytes_amount + self.signature_script.len()
    }
}

/// A struct representing an outpoint from a previous transaction
///
/// # Fields
///
/// * hash - The transaction hash of the previous transaction.
/// * index - The index of the output in the previous transaction.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Outpoint {
    pub hash: [u8; 32],
    pub index: u32,
}
impl Outpoint {
    pub fn _serialize(&self) -> Result<Vec<u8>, NodoBitcoinError> {
        let mut bytes = Vec::new();
        bytes
            .write_all(&self.hash)
            .map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;
        bytes
            .write_all(&(self.index).to_le_bytes())
            .map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;
        Ok(bytes)
    }

    pub fn deserialize(block_bytes: &[u8]) -> Result<Outpoint, NodoBitcoinError> {
        let mut offset = 0;

        let mut hash = [0u8; 32];
        hash.copy_from_slice(&block_bytes[offset..offset + 32]);
        offset += 32;

        let index = u32::from_le_bytes(
            block_bytes[offset..offset + 4]
                .try_into()
                .map_err(|_| NodoBitcoinError::NoSePuedeLeerLosBytes)?,
        );

        Ok(Outpoint { hash, index })
    }
}

/// A struct representing an output transaction for a Bitcoin transaction
///
/// # Fields
///
/// * value - The value of the output in satoshis.
/// * pk_script - The public key script for the output.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TxOut {
    pub value: u64,
    pub pk_len: usize,
    pub pk_script: Vec<u8>,
    pub pk_len_bytes: usize,
}

impl TxOut {
    pub fn _serialize(&self) -> Result<Vec<u8>, NodoBitcoinError> {
        let mut bytes = Vec::new();
        bytes
            .write_all(&(self.value).to_le_bytes())
            .map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;
        let n_bytes_prefix = utils_bytes::_from_amount_bytes_to_prefix(self.pk_len_bytes);
        bytes
            .write_all(&(utils_bytes::_build_varint_bytes(n_bytes_prefix, self.pk_script.len())?))
            .map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;
        bytes
            .write_all(&self.pk_script)
            .map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;
        Ok(bytes)
    }
    pub fn deserialize(block_bytes: &[u8]) -> Result<TxOut, NodoBitcoinError> {
        let mut offset = 0;

        let value = u64::from_le_bytes(
            block_bytes[offset..offset + 8]
                .try_into()
                .map_err(|_| NodoBitcoinError::NoSePuedeLeerLosBytes)?,
        );
        offset += 8;
        let (pk_len_bytes, pk_len) = utils_bytes::parse_varint(&block_bytes[offset..]);
        offset += pk_len_bytes;

        let mut pk_script = vec![0u8; pk_len];
        pk_script.copy_from_slice(&block_bytes[offset..offset + pk_len]);
        Ok(TxOut {
            value,
            pk_len,
            pk_script,
            pk_len_bytes,
        })
    }
    pub fn size(&self) -> usize {
        8 + self.pk_len_bytes + self.pk_script.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_transaction() {
        let version = 1;
        let input = vec![TxIn {
            previous_output: Outpoint {
                hash: [0; 32],
                index: 4294967295,
            },
            script_bytes: 31,
            script_bytes_amount: 1,
            signature_script: vec![
                4, 231, 190, 191, 79, 1, 34, 23, 47, 80, 50, 83, 72, 47, 73, 99, 101, 73, 114, 111,
                110, 45, 81, 67, 45, 109, 105, 110, 101, 114, 47,
            ],
            sequence: 4294967295,
        }];
        let output = vec![TxOut {
            value: 5000000000,
            pk_len: 35,
            pk_len_bytes: 1,
            pk_script: vec![
                33, 2, 142, 194, 100, 195, 242, 76, 65, 16, 171, 255, 30, 164, 219, 91, 108, 243,
                201, 188, 210, 174, 108, 157, 164, 77, 116, 205, 122, 47, 28, 107, 84, 81, 172,
            ],
        }];
        let lock_time = 0;

        let transaction = Transaction {
            version,
            input,
            output,
            lock_time,
            tx_in_count: 1,
            tx_out_count: 1,
        };

        let expected_bytes = vec![
            //
            1, 0, 0, 0, //version
            1, // n-tin
            // trin
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, //hash
            255, 255, 255, 255, //index
            31,  // script bytes
            4, 231, 190, 191, 79, 1, 34, 23, 47, 80, 50, 83, 72, 47, 73, 99, 101, 73, 114, 111,
            110, 45, 81, 67, 45, 109, 105, 110, 101, 114, 47, // script
            255, 255, 255, 255, // sequene
            1,   // n-trout
            0, 242, 5, 42, 1, 0, 0, 0,  // value
            35, //pk_len
            33, 2, 142, 194, 100, 195, 242, 76, 65, 16, 171, 255, 30, 164, 219, 91, 108, 243, 201,
            188, 210, 174, 108, 157, 164, 77, 116, 205, 122, 47, 28, 107, 84, 81, // trout pk
            172, 0, 0, 0, 0,
        ]; // lock time

        let serialized = transaction._serialize().unwrap();

        assert_eq!(serialized, expected_bytes);
    }

    #[test]
    fn test_deserialize_transaction() {
        let block_bytes: Vec<u8> = vec![
            //
            1, 0, 0, 0, //version
            1, // n-tin
            // trin
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, //hash
            255, 255, 255, 255, //index
            31,  // script bytes
            4, 231, 190, 191, 79, 1, 34, 23, 47, 80, 50, 83, 72, 47, 73, 99, 101, 73, 114, 111,
            110, 45, 81, 67, 45, 109, 105, 110, 101, 114, 47, // script
            255, 255, 255, 255, // sequene
            1,   // n-trout
            0, 242, 5, 42, 1, 0, 0, 0,  // value
            35, //pk_len
            33, 2, 142, 194, 100, 195, 242, 76, 65, 16, 171, 255, 30, 164, 219, 91, 108, 243, 201,
            188, 210, 174, 108, 157, 164, 77, 116, 205, 122, 47, 28, 107, 84, 81,
            172, // trout pk
            0, 0, 0, 0,
        ]; // lock time

        let transaction = Transaction::deserialize(&block_bytes).unwrap();

        assert_eq!(transaction.version, 1);
        assert_eq!(transaction.input.len(), 1);
        assert_eq!(
            transaction.input[0],
            TxIn {
                previous_output: Outpoint {
                    hash: [0; 32],
                    index: 4294967295,
                },
                script_bytes: 31,
                script_bytes_amount: 1,
                signature_script: vec![
                    4, 231, 190, 191, 79, 1, 34, 23, 47, 80, 50, 83, 72, 47, 73, 99, 101, 73, 114,
                    111, 110, 45, 81, 67, 45, 109, 105, 110, 101, 114, 47
                ],
                sequence: 4294967295,
            }
        );
        assert_eq!(transaction.output.len(), 1);
        assert_eq!(
            transaction.output[0],
            TxOut {
                value: 5000000000,
                pk_len: 35,
                pk_len_bytes: 1,
                pk_script: vec![
                    33, 2, 142, 194, 100, 195, 242, 76, 65, 16, 171, 255, 30, 164, 219, 91, 108,
                    243, 201, 188, 210, 174, 108, 157, 164, 77, 116, 205, 122, 47, 28, 107, 84, 81,
                    172
                ],
            }
        );
        assert_eq!(transaction.lock_time, 0);
    }

    #[test]
    fn test_serialize_and_deserialize_transaction() {
        let bytes = [
            1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 255, 255, 255, 255, 14, 4, 134, 231, 73, 77, 1, 81, 6, 47, 80,
            50, 83, 72, 47, 255, 255, 255, 255, 1, 0, 242, 5, 42, 1, 0, 0, 0, 35, 33, 3, 246, 217,
            255, 76, 18, 149, 148, 69, 202, 85, 73, 200, 17, 104, 59, 249, 200, 142, 99, 123, 34,
            45, 210, 224, 49, 17, 84, 196, 200, 92, 244, 35, 172, 0, 0, 0, 0,
        ];

        let tx = Transaction::deserialize(&bytes);
        assert!(tx.is_ok());

        let tx = tx.unwrap();
        let serialized = tx._serialize();
        assert!(serialized.is_ok());

        let serialized = serialized.unwrap();

        assert_eq!(serialized, bytes);
    }

    #[test]
    fn test_transaction_size() {
        let version = 1;

        let input = vec![TxIn {
            previous_output: Outpoint {
                hash: [0; 32],
                index: 4294967295,
            },
            script_bytes: 31,
            script_bytes_amount: 1,
            signature_script: vec![
                4, 231, 190, 191, 79, 1, 34, 23, 47, 80, 50, 83, 72, 47, 73, 99, 101, 73, 114, 111,
                110, 45, 81, 67, 45, 109, 105, 110, 101, 114, 47,
            ],
            sequence: 4294967295,
        }];
        let output = vec![TxOut {
            value: 5000000000,
            pk_len: 35,
            pk_len_bytes: 1,
            pk_script: vec![
                33, 2, 142, 194, 100, 195, 242, 76, 65, 16, 171, 255, 30, 164, 219, 91, 108, 243,
                201, 188, 210, 174, 108, 157, 164, 77, 116, 205, 122, 47, 28, 107, 84, 81, 172,
            ],
        }];
        let lock_time = 0;

        let transaction = Transaction {
            version,
            input,
            output,
            lock_time,
            tx_in_count: 1,
            tx_out_count: 1,
        };

        let expected_bytes = vec![
            1, 0, 0, 0, //version
            1, // n-tin
            // trin
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, //hash
            255, 255, 255, 255, //index
            31,  // script bytes
            4, 231, 190, 191, 79, 1, 34, 23, 47, 80, 50, 83, 72, 47, 73, 99, 101, 73, 114, 111,
            110, 45, 81, 67, 45, 109, 105, 110, 101, 114, 47, // script
            255, 255, 255, 255, // sequene
            1,   // n-trout
            0, 242, 5, 42, 1, 0, 0, 0,  // value
            35, //pk_len
            33, 2, 142, 194, 100, 195, 242, 76, 65, 16, 171, 255, 30, 164, 219, 91, 108, 243, 201,
            188, 210, 174, 108, 157, 164, 77, 116, 205, 122, 47, 28, 107, 84, 81, // trout pk
            172, 0, 0, 0, 0,
        ]; // lock time

        assert_eq!(transaction.size(), expected_bytes.len());
    }

    #[test]
    fn test_serialize_tx_in() {
        let expected_bytes = vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, //hash
            255, 255, 255, 255, //index
            31,  // script bytes
            4, 231, 190, 191, 79, 1, 34, 23, 47, 80, 50, 83, 72, 47, 73, 99, 101, 73, 114, 111,
            110, 45, 81, 67, 45, 109, 105, 110, 101, 114, 47, // script
            255, 255, 255, 255, // sequene
        ];
        let previous_output = Outpoint {
            hash: [0; 32],
            index: 4294967295,
        };
        let script_bytes = 31;
        let signature_script = vec![
            4, 231, 190, 191, 79, 1, 34, 23, 47, 80, 50, 83, 72, 47, 73, 99, 101, 73, 114, 111,
            110, 45, 81, 67, 45, 109, 105, 110, 101, 114, 47,
        ];
        let sequence = 4294967295;

        let tx_in = TxIn {
            previous_output,
            script_bytes,
            script_bytes_amount: 1,
            signature_script: signature_script.clone(),
            sequence,
        };

        let serialized = tx_in._serialize().unwrap();

        assert_eq!(serialized.len(), expected_bytes.len());
        assert_eq!(serialized, expected_bytes);
    }

    #[test]
    fn test_deserialize_tx_in() {
        let bytes = vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, //hash
            255, 255, 255, 255, //index
            31,  // script bytes
            4, 231, 190, 191, 79, 1, 34, 23, 47, 80, 50, 83, 72, 47, 73, 99, 101, 73, 114, 111,
            110, 45, 81, 67, 45, 109, 105, 110, 101, 114, 47, // script
            255, 255, 255, 255, // sequene
        ];

        let tx_in = TxIn::deserialize(&bytes).unwrap();

        assert_eq!(tx_in.previous_output.hash, [0u8; 32]);
        assert_eq!(tx_in.previous_output.index, 4294967295);
        assert_eq!(tx_in.script_bytes, 31);
        assert_eq!(
            tx_in.signature_script,
            vec![
                4, 231, 190, 191, 79, 1, 34, 23, 47, 80, 50, 83, 72, 47, 73, 99, 101, 73, 114, 111,
                110, 45, 81, 67, 45, 109, 105, 110, 101, 114, 47
            ]
        );
        assert_eq!(tx_in.sequence, 4294967295);
    }

    #[test]
    fn test_size_tx_in() {
        let previous_output = Outpoint {
            hash: [0; 32],
            index: 0,
        };
        let script_bytes = 10;
        let signature_script = vec![1, 2, 3, 4, 5];
        let sequence = 100;
        let script_bytes_amount = 1;

        let tx_in = TxIn {
            previous_output,
            script_bytes,
            signature_script: signature_script.clone(),
            sequence,
            script_bytes_amount,
        };

        let expected_size = 40 + script_bytes_amount + signature_script.len();
        let actual_size = tx_in.size();

        assert_eq!(expected_size, actual_size);
    }

    #[test]
    fn test_serialize_outpoint() {
        let outpoint = Outpoint {
            hash: [1u8; 32],
            index: 123,
        };

        let expected_bytes = vec![
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, // hash
            123, 0, 0, 0, // index
        ];

        let serialized = outpoint._serialize().unwrap();

        assert_eq!(serialized, expected_bytes);
    }

    #[test]
    fn test_deserialize_outpoint() {
        let bytes = vec![
            2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
            2, 2, 2, // hash
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
    fn test_serialize_tx_out() {
        let expected_bytes = vec![
            0, 242, 5, 42, 1, 0, 0, 0,  // value
            35, //pk_len
            33, 2, 142, 194, 100, 195, 242, 76, 65, 16, 171, 255, 30, 164, 219, 91, 108, 243, 201,
            188, 210, 174, 108, 157, 164, 77, 116, 205, 122, 47, 28, 107, 84, 81,
            172, // trout pk
        ];

        let txout = TxOut {
            value: 5000000000,
            pk_len: 1,
            pk_len_bytes: 35,
            pk_script: vec![
                33, 2, 142, 194, 100, 195, 242, 76, 65, 16, 171, 255, 30, 164, 219, 91, 108, 243,
                201, 188, 210, 174, 108, 157, 164, 77, 116, 205, 122, 47, 28, 107, 84, 81, 172,
            ],
        };

        let bytes = txout._serialize().unwrap();

        assert_eq!(bytes, expected_bytes);
    }

    #[test]
    fn test_deserialize_tx_out() {
        let bytes = vec![
            0, 242, 5, 42, 1, 0, 0, 0,  // value
            35, //pk_len
            33, 2, 142, 194, 100, 195, 242, 76, 65, 16, 171, 255, 30, 164, 219, 91, 108, 243, 201,
            188, 210, 174, 108, 157, 164, 77, 116, 205, 122, 47, 28, 107, 84, 81,
            172, // trout pk
        ];

        let txout = TxOut::deserialize(&bytes).unwrap();
        assert_eq!(txout.value, 5000000000);
        assert_eq!(
            txout.pk_script,
            vec![
                33, 2, 142, 194, 100, 195, 242, 76, 65, 16, 171, 255, 30, 164, 219, 91, 108, 243,
                201, 188, 210, 174, 108, 157, 164, 77, 116, 205, 122, 47, 28, 107, 84, 81, 172
            ]
        );
    }

    #[test]
    fn test_size() {
        let value = 1000;
        let pk_len = 20;
        let pk_script = vec![1, 2, 3, 4, 5];
        let pk_len_bytes = 1;

        let tx_out = TxOut {
            value,
            pk_len,
            pk_script,
            pk_len_bytes,
        };

        let expected_size = 8 + pk_len_bytes + 5;
        let actual_size = tx_out.size();

        assert_eq!(expected_size, actual_size);
    }
}
