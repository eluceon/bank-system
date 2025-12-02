#[derive(Debug, Clone, PartialEq)]
pub enum OpKind {
    Deposit(u32),
    Withdraw(u32),
    CloseAccount,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Balance {
    pub result: u64,
    pub last_ops: Vec<OpKind>,
}

impl Balance {
    pub fn new() -> Self {
        Balance {
            result: 0,
            last_ops: Vec::new(),
        }
    }

    pub fn process<'a>(&mut self, ops: &[&'a OpKind]) -> Vec<&'a OpKind> {
        let mut remaining = ops.iter();
        let mut bad_ops = Vec::new();

        for op in &mut remaining {
            match op {
                OpKind::Deposit(value) => {
                    self.result += *value as u64;
                    self.last_ops.push((*op).clone());
                }
                OpKind::Withdraw(value) if self.result >= *value as u64 => {
                    self.result -= *value as u64;
                    self.last_ops.push((*op).clone());
                }
                other => {
                    bad_ops.push(*other);
                    break;
                }
            }
        }

        bad_ops.extend(remaining);
        bad_ops
    }
}

impl Default for Balance {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_balance() {
        let balance = Balance::new();
        assert_eq!(balance.result, 0);
        assert_eq!(balance.last_ops.len(), 0);
    }

    #[test]
    fn process_successful_operations() {
        let mut balance = Balance::new();
        let ops = [
            &OpKind::Deposit(50),
            &OpKind::Withdraw(30),
            &OpKind::Deposit(20),
        ];

        let failed = balance.process(&ops);

        assert_eq!(balance.result, 40);
        assert_eq!(failed.len(), 0);
        assert_eq!(balance.last_ops.len(), 3);
    }

    #[test]
    fn process_insufficient_funds() {
        let mut balance = Balance::new();
        let ops = [
            &OpKind::Deposit(50),
            &OpKind::Withdraw(30),
            &OpKind::Withdraw(30),
            &OpKind::Deposit(100),
        ];

        let failed = balance.process(&ops);

        assert_eq!(balance.result, 20);
        assert_eq!(failed.len(), 2);
        assert_eq!(balance.last_ops.len(), 2);
    }

    #[test]
    fn process_close_account() {
        let mut balance = Balance::new();
        let ops = [
            &OpKind::Deposit(32),
            &OpKind::Withdraw(64),
            &OpKind::CloseAccount,
        ];

        let failed = balance.process(&ops);

        assert_eq!(balance.result, 32);
        assert_eq!(failed.len(), 2);
        assert_eq!(balance.last_ops.len(), 1);
    }
}
