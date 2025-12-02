use crate::storage::Storage;

#[derive(Debug)]
pub enum TxError {
    InsufficientFunds,
    InvalidAccount,
}

pub trait Transaction {
    fn apply(&self, storage: &mut Storage) -> Result<(), TxError>;
}

pub struct Deposit {
    pub account: String,
    pub amount: u64,
}

impl Transaction for Deposit {
    fn apply(&self, storage: &mut Storage) -> Result<(), TxError> {
        let balance = storage.accounts.entry(self.account.clone()).or_default();
        balance.result += self.amount;
        Ok(())
    }
}

pub struct Transfer {
    pub from: String,
    pub to: String,
    pub amount: u64,
}

impl Transaction for Transfer {
    fn apply(&self, storage: &mut Storage) -> Result<(), TxError> {
        let from_balance = storage
            .accounts
            .get(&self.from)
            .map(|b| b.result)
            .unwrap_or(0);

        if from_balance < self.amount {
            return Err(TxError::InsufficientFunds);
        }

        if let Some(balance) = storage.accounts.get_mut(&self.from) {
            balance.result -= self.amount;
        } else {
            return Err(TxError::InvalidAccount);
        }

        // Зачисляем получателю
        let to_balance = storage.accounts.entry(self.to.clone()).or_default();
        to_balance.result += self.amount;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deposit_creates_account() {
        let mut storage = Storage::new();

        let tx = Deposit {
            account: "Alice".to_string(),
            amount: 100,
        };

        assert!(tx.apply(&mut storage).is_ok());
        assert_eq!(storage.accounts.get("Alice").unwrap().result, 100);
    }

    #[test]
    fn deposit_adds_to_existing() {
        let mut storage = Storage::new();
        storage.add_user("Bob".to_string());

        let tx = Deposit {
            account: "Bob".to_string(),
            amount: 50,
        };

        assert!(tx.apply(&mut storage).is_ok());
        assert_eq!(storage.accounts.get("Bob").unwrap().result, 50);


        let tx2 = Deposit {
            account: "Bob".to_string(),
            amount: 30,
        };
        assert!(tx2.apply(&mut storage).is_ok());
        assert_eq!(storage.accounts.get("Bob").unwrap().result, 80);
    }

    #[test]
    fn transfer_success() {
        let mut storage = Storage::new();
        storage.add_user("Alice".to_string());
        storage.accounts.get_mut("Alice").unwrap().result = 100;
        storage.add_user("Bob".to_string());

        let tx = Transfer {
            from: "Alice".to_string(),
            to: "Bob".to_string(),
            amount: 40,
        };

        assert!(tx.apply(&mut storage).is_ok());
        assert_eq!(storage.accounts.get("Alice").unwrap().result, 60);
        assert_eq!(storage.accounts.get("Bob").unwrap().result, 40);
    }

    #[test]
    fn transfer_insufficient_funds() {
        let mut storage = Storage::new();
        storage.add_user("Alice".to_string());
        storage.accounts.get_mut("Alice").unwrap().result = 30;

        let tx = Transfer {
            from: "Alice".to_string(),
            to: "Bob".to_string(),
            amount: 50,
        };

        let result = tx.apply(&mut storage);
        assert!(matches!(result, Err(TxError::InsufficientFunds)));
    }

    #[test]
    fn transfer_creates_recipient() {
        let mut storage = Storage::new();
        storage.add_user("Alice".to_string());
        storage.accounts.get_mut("Alice").unwrap().result = 100;

        let tx = Transfer {
            from: "Alice".to_string(),
            to: "NewUser".to_string(),
            amount: 25,
        };

        assert!(tx.apply(&mut storage).is_ok());
        assert_eq!(storage.accounts.get("NewUser").unwrap().result, 25);
    }
}
