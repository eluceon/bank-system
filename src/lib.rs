pub mod analytics;
pub mod errors;
pub mod operations;
pub mod storage;

pub use analytics::find_best;
pub use errors::BalanceManagerError;
pub use operations::{Balance, OpKind};
pub use storage::{BalanceManager, Storage};

pub type Name = String;
