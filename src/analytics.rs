use crate::operations::OpKind;
use crate::storage::Storage;

fn find_best(storage: &Storage) -> Option<(&str, f32)> {
    if storage.accounts.is_empty() {
        return None;
    }
    let mut best_factor = f32::MIN;
    let mut best_name = "";
    for (name, balance) in &storage.accounts {
        let mut all_positive = 0;
        for op in &balance.last_ops {
            if let OpKind::Deposit(value) = op {
                all_positive += *value as u64
            }
        }
        // почти то же самое на итераторах!
        let all_negative: u64 = balance
            .last_ops
            .iter()
            .filter_map(|op| match op {
                OpKind::Withdraw(value) => Some(*value as u64),
                _ => None,
            })
            .sum();
        let factor = all_positive as f32 / all_negative as f32;
        if factor > best_factor {
            best_factor = factor;
            best_name = name;
        }
    }
    Some((best_name, best_factor))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::operations::{Balance, OpKind};
    use crate::storage::Storage;
    use std::collections::HashMap;

    #[test]
    fn find_best_empty_storage() {
        let storage = Storage {
            accounts: HashMap::new(),
        };
        assert_eq!(find_best(&storage), None);
    }

    #[test]
    fn find_best_single_account() {
        let mut accounts = HashMap::new();
        let mut balance = Balance::new();
        balance.last_ops = vec![OpKind::Deposit(1000), OpKind::Withdraw(500)];
        accounts.insert("Alice".to_string(), balance);

        let storage = Storage { accounts };
        let result = find_best(&storage);

        assert!(result.is_some());
        let (name, factor) = result.unwrap();
        assert_eq!(name, "Alice");
        assert!((factor - 2.0).abs() < 0.01); // 1000/500 = 2.0
    }

    #[test]
    fn find_best_multiple_accounts() {
        let mut accounts = HashMap::new();

        let mut dad_balance = Balance::new();
        dad_balance.last_ops = vec![OpKind::Deposit(200000), OpKind::Withdraw(100000)];

        let mut mom_balance = Balance::new();
        mom_balance.last_ops = vec![
            OpKind::Deposit(120000),
            OpKind::Withdraw(50000),
            OpKind::Withdraw(20000),
        ];

        let mut son_balance = Balance::new();
        son_balance.last_ops = vec![
            OpKind::Deposit(5000),
            OpKind::Withdraw(500),
            OpKind::Withdraw(1000),
            OpKind::Withdraw(700),
        ];

        accounts.insert("Dad".to_string(), dad_balance);
        accounts.insert("Mom".to_string(), mom_balance);
        accounts.insert("Son".to_string(), son_balance);

        let storage = Storage { accounts };
        let result = find_best(&storage);

        assert!(result.is_some());
        let (name, factor) = result.unwrap();

        // Dad: 200000/100000 = 2.0
        // Mom: 120000/70000 ≈ 1.71
        // Son: 5000/2200 ≈ 2.27
        // Son должен быть лучшим
        assert_eq!(name, "Son");
        assert!((factor - 2.27).abs() < 0.01);
    }
}
