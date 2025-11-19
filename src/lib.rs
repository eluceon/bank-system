use std::collections::HashMap;

pub mod storage;

pub type Name = String;
pub type Balance = i64;

pub struct Storage {
    accounts: HashMap<Name, Balance>,
}
