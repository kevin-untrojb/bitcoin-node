use crate::blockchain::transaction::Transaction;
use crate::common::uint256::Uint256;
use crate::errores::NodoBitcoinError;
use std::collections::HashMap;
use std::fmt;

#[derive(Debug)]
pub struct Utxo {
    pub tx_id: Uint256,
    pub output_index: u32,
    pub amount: u64,
    pub account: Vec<u8>,
}

impl fmt::Display for Utxo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            f,
            "tx_id: {:?}\namount: {:?}\naccount: {:?}",
            self.tx_id, self.amount, self.account
        )
    }
}

#[derive(Debug)]
pub struct UTXOSet {
    pub utxos: HashMap<Uint256, Vec<Utxo>>,
}

impl fmt::Display for UTXOSet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut utxos = Vec::new();
        for (key, utxos_for_tx) in &self.utxos {
            let format_string = format!("key: {:?}\nutxos_for_tx: {:?}", key, utxos_for_tx);
            utxos.push(format_string);
        }
        write!(f, "UTXOSet\n{:?}]", utxos)
    }
}

impl UTXOSet {
    pub fn new() -> Self {
        UTXOSet {
            utxos: HashMap::new(),
        }
    }

    pub fn build_from_transactions(
        &mut self,
        transactions: Vec<Transaction>,
        accounts: Vec<String>,
    ) -> Result<(), NodoBitcoinError> {
        let mut spent_outputs: HashMap<Uint256, Vec<u32>> = HashMap::new();

        for transaction in transactions.iter().rev() {
            for tx_in in &transaction.input {
                let previous_tx_id = Uint256::from_be_bytes(tx_in.previous_output.hash);
                let output_index = tx_in.previous_output.index;

                let spent_outputs_for_tx =
                    spent_outputs.entry(previous_tx_id).or_insert(Vec::new());
                spent_outputs_for_tx.push(output_index);
            }
        }

        for transaction in transactions {
            let tx_id = transaction.txid()?;
            for (output_index, tx_out) in transaction.output.iter().enumerate() {
                let mut is_user_account_output = false;
                for account in accounts.iter() {
                    if tx_out.is_user_account_output(account) {
                        is_user_account_output = true;
                        continue;
                    }
                }

                if !is_user_account_output {
                    // si no es una de las cuentas, no me importa
                    continue;
                }

                if spent_outputs.contains_key(&tx_id)
                    && spent_outputs[&tx_id].contains(&(output_index as u32))
                {
                    // si la salida de transacción está gastada no es un UXTO
                    continue;
                }

                let utxo = Utxo {
                    tx_id,
                    output_index: output_index as u32,
                    amount: tx_out.value,
                    account: tx_out.pk_script.clone(),
                };

                let utxos_for_tx = self.utxos.entry(tx_id).or_insert(Vec::new());
                utxos_for_tx.push(utxo);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        blockchain::transaction::{Outpoint, TxIn, TxOut},
        common::decoder::{decode_base58, p2pkh_script_serialized},
    };

    use super::*;

    fn get_pk_script_from_account(account: &str) -> Vec<u8> {
        let script = match decode_base58(account) {
            Ok(script) => script,
            Err(e) => return vec![],
        };

        let p2pkh_script = match p2pkh_script_serialized(&script) {
            Ok(p2pkh_script) => return p2pkh_script,
            Err(e) => return vec![],
        };
    }

    #[test]
    fn test_build_from_transactions() {
        let account = "mnJvq7mbGiPNNhUne4FAqq27Q8xZrAsVun";
        let p2pkh_script = get_pk_script_from_account(account);

        let tx_out1 = TxOut {
            value: 100,
            pk_len: 0,
            pk_script: p2pkh_script.clone(),
            pk_len_bytes: 0,
        };
        let tx_out2 = TxOut {
            value: 200,
            pk_len: 0,
            pk_script: p2pkh_script,
            pk_len_bytes: 0,
        };

        let tx_in1 = TxIn {
            previous_output: Outpoint {
                hash: [0; 32],
                index: 0,
            },
            script_bytes: 0,
            signature_script: vec![],
            sequence: 0,
            script_bytes_amount: 0,
        };
        let tx_in2 = TxIn {
            previous_output: Outpoint {
                hash: [0; 32],
                index: 0,
            },
            script_bytes: 0,
            signature_script: vec![],
            sequence: 0,
            script_bytes_amount: 0,
        };

        let transaction1 = Transaction {
            version: 1,
            input: vec![tx_in1.clone()],
            output: vec![tx_out1.clone()],
            lock_time: 0,
            tx_in_count: 1,
            tx_out_count: 1,
        };
        let transaction2 = Transaction {
            version: 1,
            input: vec![tx_in2.clone()],
            output: vec![tx_out2.clone()],
            lock_time: 0,
            tx_in_count: 1,
            tx_out_count: 1,
        };
        let transactions = vec![transaction1.clone(), transaction2.clone()];

        let mut utxo_set = UTXOSet::new();
        let result = utxo_set.build_from_transactions(transactions, vec![account.to_string()]);
        assert!(result.is_ok());

        assert_eq!(utxo_set.utxos.len(), 2);
        assert!(utxo_set.utxos.contains_key(&transaction1.txid().unwrap()));
        assert!(utxo_set.utxos.contains_key(&transaction2.txid().unwrap()));

        let utxos_tx1 = utxo_set.utxos.get(&transaction1.txid().unwrap()).unwrap();
        assert_eq!(utxos_tx1.len(), 1);
        assert_eq!(utxos_tx1[0].amount, tx_out1.value);

        let utxos_tx2 = utxo_set.utxos.get(&transaction2.txid().unwrap()).unwrap();
        assert_eq!(utxos_tx2.len(), 1);
        assert_eq!(utxos_tx2[0].amount, tx_out2.value);
    }

    #[test]
    fn test_build_from_transactions_for_spent_outputs() {
        let account = "mnJvq7mbGiPNNhUne4FAqq27Q8xZrAsVun";
        let p2pkh_script = get_pk_script_from_account(account);

        let mut utxo_set = UTXOSet::new();

        let transaction1 = Transaction {
            input: vec![],
            output: vec![TxOut {
                value: 5,
                pk_script: p2pkh_script,
                pk_len: 0,
                pk_len_bytes: 0,
            }],
            lock_time: 0,
            tx_in_count: 0,
            tx_out_count: 1,
            version: 1,
        };

        println!("tx1 Id: {:?}", transaction1.txid().unwrap().get_bytes());

        let transaction2 = Transaction {
            input: vec![TxIn {
                previous_output: Outpoint {
                    hash: transaction1.txid().unwrap().get_bytes(),
                    index: 0,
                },
                signature_script: vec![],
                script_bytes: 0,
                script_bytes_amount: 0,
                sequence: 0,
            }],
            output: vec![TxOut {
                value: 5,
                pk_script: vec![],
                pk_len: 0,
                pk_len_bytes: 0,
            }],
            lock_time: 0,
            tx_in_count: 1,
            tx_out_count: 1,
            version: 1,
        };

        utxo_set
            .build_from_transactions(vec![transaction1, transaction2], vec![account.to_string()])
            .unwrap();
        assert_eq!(utxo_set.utxos.len(), 0);
    }
}
