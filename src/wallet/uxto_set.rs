use crate::blockchain::transaction::{Transaction, TxOut};
use crate::common::uint256::Uint256;
use crate::errores::NodoBitcoinError;
use std::collections::HashMap;
use std::fmt;

use super::user::Account;

#[derive(Debug, Clone)]
pub struct Utxo {
    pub tx_id: Uint256,
    pub output_index: u32,
    pub tx_out: TxOut,
    pub pk_script: Vec<u8>,
    pub tx: Transaction,
}

impl fmt::Display for Utxo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            f,
            "tx_id: {:?}\namount: {:?}\naccount: {:?}",
            self.tx_id, self.tx_out.value, self.pk_script
        )
    }
}

#[derive(Debug, Clone)]
pub struct UTXOSet {
    pub utxos_for_account: HashMap<String, Vec<Utxo>>,
    pub account_for_txid_index: HashMap<(Uint256, u32), String>,
}

impl fmt::Display for UTXOSet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut utxos = Vec::new();
        for (key, utxos_for_tx) in &self.utxos_for_account {
            let format_string = format!("key: {:?}\nutxos_for_tx: {:?}", key, utxos_for_tx);
            utxos.push(format_string);
        }
        write!(f, "UTXOSet\n{:?}]", utxos)
    }
}

impl UTXOSet {
    pub fn new() -> Self {
        UTXOSet {
            utxos_for_account: HashMap::new(),
            account_for_txid_index: HashMap::new(),
        }
    }

    fn agregar_utxo(
        &mut self,
        current_account: String,
        tx_id: Uint256,
        output_index: u32,
        tx_out: &TxOut,
        tx: &Transaction,
    ) {
        let utxo = Utxo {
            tx_id,
            output_index: output_index,
            tx_out: tx_out.clone(),
            pk_script: tx_out.pk_script.clone(),
            tx: tx.clone(),
        };
        let utxos_for_account = self
            .utxos_for_account
            .entry(current_account.clone())
            .or_insert(Vec::new());
        utxos_for_account.push(utxo);

        self.account_for_txid_index
            .insert((tx_id, output_index), current_account);
    }

    fn eliminar_utxo(&mut self, previous_tx_id: Uint256, output_index: u32, key: (Uint256, u32)) {
        let account = self.account_for_txid_index[&key].clone();
        let utxos_for_account = self.utxos_for_account.entry(account.clone()).or_default();
        utxos_for_account
            .retain(|utxo| !(utxo.tx_id == previous_tx_id && utxo.output_index == output_index));
        self.account_for_txid_index.remove(&key);
    }

    fn validar_output(accounts: Vec<Account>, tx_out: &TxOut) -> Result<Account, NodoBitcoinError> {
        for account in accounts.iter() {
            if tx_out.is_user_account_output(account.clone().public_key) {
                return Ok(account.clone());
            }
        }
        Err(NodoBitcoinError::InvalidAccount)
    }

    pub fn update_from_transactions(
        &mut self,
        transactions: Vec<Transaction>,
        accounts: Vec<Account>,
    ) -> Result<(), NodoBitcoinError> {
        for tx in transactions.iter() {
            let tx_id = tx.txid()?;
            // recorro los output para identificar los que son mios
            for (output_index, tx_out) in tx.output.iter().enumerate() {
                let current_account = match UTXOSet::validar_output(accounts.clone(), tx_out) {
                    Ok(account) => account,
                    Err(_) => continue,
                };

                self.agregar_utxo(
                    current_account.public_key.clone(),
                    tx_id,
                    output_index as u32,
                    tx_out,
                    &tx,
                );
            }

            for tx_in in tx.input.iter() {
                let previous_tx_id = Uint256::from_be_bytes(tx_in.previous_output.hash);
                let output_index = tx_in.previous_output.index;
                let key = (previous_tx_id, output_index);

                if self.account_for_txid_index.contains_key(&key) {
                    self.eliminar_utxo(previous_tx_id, output_index, key);
                }
            }
        }
        Ok(())
    }

    pub fn get_available(&self, account: String) -> Result<u64, NodoBitcoinError> {
        let mut balance = 0;
        if let Some(utxos) = self.utxos_for_account.get(&account) {
            for utxo in utxos.iter() {
                balance += utxo.tx_out.value;
            }
        }
        Ok(balance)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        blockchain::transaction::{Outpoint, TxIn, TxOut},
        common::decoder::{decode_base58, p2pkh_script_serialized},
    };

    use super::*;

    fn get_pk_script_from_account(account: String) -> Vec<u8> {
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
        let private_key = "cRJzHMCgDLsvttTH8R8t6LLcZgMDs1WtgwQXxk8bFFk7E2AJp1tw".to_string();
        let public_key = "mnJvq7mbGiPNNhUne4FAqq27Q8xZrAsVun".to_string();
        let account_name = "test".to_string();
        let account = Account::new(private_key, public_key.clone(), account_name);
        let p2pkh_script = get_pk_script_from_account(public_key.clone());

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
        let result = utxo_set.update_from_transactions(transactions, vec![account]);
        assert!(result.is_ok());

        assert_eq!(utxo_set.utxos_for_account.len(), 1);
        assert!(utxo_set.utxos_for_account.contains_key(&public_key));
        let utxos_for_account = utxo_set.utxos_for_account.get(&public_key).unwrap();
        assert_eq!(utxos_for_account.len(), 2);
        assert!(utxos_for_account[0].tx_id == transaction1.txid().unwrap());
        assert!(utxos_for_account[1].tx_id == transaction2.txid().unwrap());

        let balance = utxo_set.get_available(public_key);
        assert!(balance.is_ok());
        assert_eq!(balance.unwrap(), 300);
    }

    #[test]
    fn test_build_from_transactions_for_spent_outputs() {
        let private_key = "cRJzHMCgDLsvttTH8R8t6LLcZgMDs1WtgwQXxk8bFFk7E2AJp1tw".to_string();
        let public_key = "mnJvq7mbGiPNNhUne4FAqq27Q8xZrAsVun".to_string();
        let account_name = "test".to_string();
        let account = Account::new(private_key, public_key.clone(), account_name);

        let p2pkh_script = get_pk_script_from_account(public_key.clone());

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
            .update_from_transactions(vec![transaction1, transaction2], vec![account.clone()])
            .unwrap();
        assert_eq!(utxo_set.utxos_for_account[&account.public_key].len(), 0);
    }
}
