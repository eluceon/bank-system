pub mod analytics;
pub mod errors;
pub mod operations;
pub mod storage;
pub mod transaction;

pub use analytics::find_best;
pub use errors::BalanceManagerError;
pub use operations::{Balance, OpKind};
pub use storage::{BalanceManager, Storage};
pub use transaction::{Deposit, Transaction, Transfer, TxError};

pub type Name = String;
