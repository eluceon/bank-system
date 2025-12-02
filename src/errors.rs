use crate::Name;
use std::fmt;

#[derive(Debug)]
pub enum BalanceManagerError {
    UserNotFound(Name),
    NotEnoughMoney { required: u64, available: u64 },
}

impl fmt::Display for BalanceManagerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BalanceManagerError::UserNotFound(name) => {
                write!(f, "Пользователь '{}' не найден", name)
            }
            BalanceManagerError::NotEnoughMoney {
                required,
                available,
            } => {
                write!(
                    f,
                    "Недостаточно средств: требуется {}, доступно {}",
                    required, available
                )
            }
        }
    }
}

impl std::error::Error for BalanceManagerError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_not_found_display() {
        let err = BalanceManagerError::UserNotFound("Alice".to_string());
        assert_eq!(format!("{}", err), "Пользователь 'Alice' не найден");
    }

    #[test]
    fn not_enough_money_display() {
        let err = BalanceManagerError::NotEnoughMoney {
            required: 100,
            available: 50,
        };
        assert_eq!(
            format!("{}", err),
            "Недостаточно средств: требуется 100, доступно 50"
        );
    }

    #[test]
    fn error_debug() {
        let err = BalanceManagerError::UserNotFound("Bob".to_string());
        assert!(format!("{:?}", err).contains("UserNotFound"));
    }
}
