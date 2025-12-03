use crate::storage::Storage;
use std::ops::Add;

#[derive(Debug)]
pub enum TxError {
    InsufficientFunds,
    InvalidAccount,
}

pub trait Transaction {
    fn apply(&self, storage: &mut Storage) -> Result<(), TxError>;
}

pub struct TxCombinator<T1, T2> {
    t1: T1,
    t2: T2,
}

impl<T1: Transaction, T2: Transaction> Transaction for TxCombinator<T1, T2> {
    fn apply(&self, storage: &mut Storage) -> Result<(), TxError> {
        self.t1.apply(storage)?;
        self.t2.apply(storage)?;
        Ok(())
    }
}

impl<T1, T2, Rhs: Transaction> Add<Rhs> for TxCombinator<T1, T2> {
    type Output = TxCombinator<TxCombinator<T1, T2>, Rhs>;

    fn add(self, rhs: Rhs) -> Self::Output {
        TxCombinator { t1: self, t2: rhs }
    }
}

pub struct Deposit {
    pub account: String,
    pub amount: u64,
}

impl<T: Transaction> Add<T> for Deposit {
    type Output = TxCombinator<Deposit, T>;

    fn add(self, rhs: T) -> Self::Output {
        TxCombinator { t1: self, t2: rhs }
    }
}

impl Transaction for Deposit {
    fn apply(&self, storage: &mut Storage) -> Result<(), TxError> {
        let balance = storage.accounts.entry(self.account.clone()).or_default();
        balance.result += self.amount;
        Ok(())
    }
}

pub struct Withdraw {
    pub account: String,
    pub amount: u64,
}

impl Transaction for Withdraw {
    fn apply(&self, storage: &mut Storage) -> Result<(), TxError> {
        let balance = storage.accounts.entry(self.account.clone()).or_default();
        if balance.result < self.amount {
            return Err(TxError::InsufficientFunds);
        }
        balance.result -= self.amount;
        Ok(())
    }
}

pub struct Transfer {
    pub from: String,
    pub to: String,
    pub amount: u64,
}

impl<T: Transaction> Add<T> for Transfer {
    type Output = TxCombinator<Transfer, T>;

    fn add(self, rhs: T) -> Self::Output {
        TxCombinator { t1: self, t2: rhs }
    }
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
    fn withdraw_success() {
        let mut storage = Storage::new();
        storage.add_user("Dima".to_string());
        storage.accounts.get_mut("Dima").unwrap().result = 100;

        let tx = Withdraw {
            account: "Dima".to_string(),
            amount: 70,
        };

        assert!(tx.apply(&mut storage).is_ok());
        assert_eq!(storage.accounts.get("Dima").unwrap().result, 30);

        let tx2 = Withdraw {
            account: "Dima".to_string(),
            amount: 30,
        };
        assert!(tx2.apply(&mut storage).is_ok());
        assert_eq!(storage.accounts.get("Dima").unwrap().result, 0);
    }

    #[test]
    fn withdraw_insufficient_funds() {
        let mut storage = Storage::new();
        storage.add_user("Dima".to_string());
        storage.accounts.get_mut("Dima").unwrap().result = 30;

        let tx = Withdraw {
            account: "Dima".to_string(),
            amount: 70,
        };

        let result = tx.apply(&mut storage);
        assert!(matches!(result, Err(TxError::InsufficientFunds)));
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

    #[test]
    fn combined_deposit_and_transfer() {
        let mut storage = Storage::new();

        let tx = Deposit {
            account: "Alice".to_string(),
            amount: 100,
        } + Transfer {
            from: "Alice".to_string(),
            to: "Bob".to_string(),
            amount: 30,
        };

        assert!(tx.apply(&mut storage).is_ok());
        assert_eq!(storage.accounts.get("Alice").unwrap().result, 70);
        assert_eq!(storage.accounts.get("Bob").unwrap().result, 30);
    }

    #[test]
    fn combined_multiple_deposits() {
        let mut storage = Storage::new();

        let tx = Deposit {
            account: "Alice".to_string(),
            amount: 50,
        } + Deposit {
            account: "Bob".to_string(),
            amount: 100,
        } + Deposit {
            account: "Alice".to_string(),
            amount: 25,
        };

        assert!(tx.apply(&mut storage).is_ok());
        assert_eq!(storage.accounts.get("Alice").unwrap().result, 75);
        assert_eq!(storage.accounts.get("Bob").unwrap().result, 100);
    }

    #[test]
    fn combined_fails_on_insufficient_funds() {
        let mut storage = Storage::new();

        let tx = Deposit {
            account: "Alice".to_string(),
            amount: 50,
        } + Transfer {
            from: "Alice".to_string(),
            to: "Bob".to_string(),
            amount: 100,
        };

        let result = tx.apply(&mut storage);
        assert!(matches!(result, Err(TxError::InsufficientFunds)));
        assert_eq!(storage.accounts.get("Alice").unwrap().result, 50);
    }
}
