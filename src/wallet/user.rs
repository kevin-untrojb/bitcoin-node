use std::{
    fs::File,
    io::{Read, Write},
};

use crate::{
    common::utils_file::{read_decoded_string_offset, save_encoded_len_bytes},
    errores::NodoBitcoinError,
};

#[derive(Clone)]
pub struct Account {
    pub secret_key: String,
    pub public_key: String,
    pub wallet_name: String,
}

const ACCOUNT_FILENAME: &str = "accounts.dat";

impl Account {
    pub fn new(secret_key: String, public_key: String, wallet_name: String) -> Account {
        Account {
            secret_key,
            public_key,
            wallet_name,
        }
    }

    pub fn save_all_accounts(accounts: Vec<Account>) -> Result<(), NodoBitcoinError> {
        let file = File::create(ACCOUNT_FILENAME).expect("No se pudo crear el archivo");
        for account in accounts {
            account.save(&mut &file)?;
        }
        Ok(())
    }

    fn save(&self, file: &mut dyn Write) -> Result<(), NodoBitcoinError> {
        save_encoded_len_bytes(file, self.secret_key.clone())?;
        save_encoded_len_bytes(file, self.public_key.clone())?;
        save_encoded_len_bytes(file, self.wallet_name.clone())
    }

    pub fn get_all_accounts() -> Result<Vec<Account>, NodoBitcoinError> {
        let mut file = match File::open(ACCOUNT_FILENAME) {
            Ok(file) => file,
            Err(_) => return Err(NodoBitcoinError::NoExisteArchivo),
        };
        // leer todos los bytes del archivo
        let mut buffer = vec![];
        match file.read_to_end(&mut buffer) {
            Ok(_) => {}
            Err(_) => return Err(NodoBitcoinError::NoSePuedeLeerLosBytes),
        };
        let accounts = Self::parse_accounts(buffer)?;
        Ok(accounts)
    }

    fn parse_accounts(buffer: Vec<u8>) -> Result<Vec<Account>, NodoBitcoinError> {
        let mut todas = vec![];
        let mut offset = 0;
        let buffer_len = buffer.len() as u64;
        while offset < buffer_len {
            let mut account_bytes = vec![];
            for _ in 0..3 {
                let (value, new_offset) = read_decoded_string_offset(buffer.clone(), offset)?;
                account_bytes.push(value);
                offset = new_offset;
            }
            let account = Account::new(
                account_bytes[0].clone(),
                account_bytes[1].clone(),
                account_bytes[2].clone(),
            );
            todas.push(account);
        }
        Ok(todas)
    }
}

#[cfg(test)]
mod tests {
    use crate::wallet::user::Account;

    #[test]
    fn test_one_account_save_read() {
        let account = Account::new(
            "cRJzHMCgDLsvttTH8R8t6LLcZgMDs1WtgwQXxk8bFFk7E2AJp1tw".to_string(),
            "mnJvq7mbGiPNNhUne4FAqq27Q8xZrAsVun".to_string(),
            "wallet1".to_string(),
        );

        let mut mock_write = vec![];

        let save = account.save(&mut mock_write);
        assert!(save.is_ok());

        let read = Account::parse_accounts(mock_write);
        assert!(read.is_ok());
        let accounts = read.unwrap();

        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].secret_key, account.secret_key);
        assert_eq!(accounts[0].public_key, account.public_key);
        assert_eq!(accounts[0].wallet_name, account.wallet_name);
    }

    #[test]
    fn test_two_account_save_read() {
        let account1 = Account::new(
            "cRJzHMCgDLsvttTH8R8t6LLcZgMDs1WtgwQXxk8bFFk7E2AJp1tw".to_string(),
            "mnJvq7mbGiPNNhUne4FAqq27Q8xZrAsVun".to_string(),
            "wallet2".to_string(),
        );

        let mut mock_write = vec![];

        let save = account1.save(&mut mock_write);
        assert!(save.is_ok());

        let read = Account::parse_accounts(mock_write.clone());
        assert!(read.is_ok());
        let accounts = read.unwrap();

        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].secret_key, account1.secret_key);
        assert_eq!(accounts[0].public_key, account1.public_key);
        assert_eq!(accounts[0].wallet_name, account1.wallet_name);

        let account2 = Account::new(
            "cU7dbzeBRgMEZ5BUst2CFydGRm9gt8uQbNoojWPRRuHb2xk5q5h2".to_string(),
            "mtm4vS3WH7pg13pjFEmqGq2TSPDcUN6k7a".to_string(),
            "wallet1".to_string(),
        );

        let save = account2.save(&mut mock_write);
        assert!(save.is_ok());

        let read = Account::parse_accounts(mock_write);
        assert!(read.is_ok());
        let accounts = read.unwrap();

        assert_eq!(accounts.len(), 2);
        assert_eq!(accounts[0].secret_key, account1.secret_key);
        assert_eq!(accounts[0].public_key, account1.public_key);
        assert_eq!(accounts[0].wallet_name, account1.wallet_name);
        assert_eq!(accounts[1].secret_key, account2.secret_key);
        assert_eq!(accounts[1].public_key, account2.public_key);
        assert_eq!(accounts[1].wallet_name, account2.wallet_name);
    }

    // #[test]
    // fn test_read() {
    //     // Para probar si se creo bien desde interfaz
    //     let read = Account::get_all_accounts().unwrap();
    //     println!("{}", read[0].secret_key);
    //     println!("{}", read[0].public_key);
    //     println!("{}", read[0].wallet_name);
    // }

    // #[test]
    // fn test_file() {
    //     let secret_key = "cRJzHMCgDLsvttTH8R8t6LLcZgMDs1WtgwQXxk8bFFk7E2AJp1tw".to_string();
    //     let public_key = "mnJvq7mbGiPNNhUne4FAqq27Q8xZrAsVun".to_string();
    //     let wallet_name = "wallet1".to_string();
    //     let account = Account::new(secret_key.clone(), public_key.clone(), wallet_name.clone());

    //     let accounts = vec![account];

    //     let saved = Account::save_all_accounts(accounts);
    //     assert!(saved.is_ok());

    //     let read = Account::get_all_accounts();
    //     assert!(read.is_ok());

    //     let accounts = read.unwrap();
    //     assert_eq!(accounts.len(), 1);
    //     assert_eq!(accounts[0].secret_key, secret_key);
    //     assert_eq!(accounts[0].public_key, public_key);
    //     assert_eq!(accounts[0].wallet_name, wallet_name);
    // }
}
