use crate::Name;
use crate::errors::BalanceManagerError;
use crate::operations::{Balance, OpKind};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufRead;
use std::path::Path;
use std::{fs, io};

pub trait BalanceManager {
    fn deposit(&mut self, name: &Name, amount: u64) -> Result<(), BalanceManagerError>;
    fn withdraw(&mut self, name: &Name, amount: u64) -> Result<(), BalanceManagerError>;
}

pub struct Storage {
    pub accounts: HashMap<Name, Balance>,
}

impl Default for Storage {
    fn default() -> Self {
        Self::new()
    }
}

impl Storage {
    pub fn new() -> Self {
        Storage {
            accounts: HashMap::new(),
        }
    }

    pub fn add_user(&mut self, name: Name) -> Option<u64> {
        if let std::collections::hash_map::Entry::Vacant(e) = self.accounts.entry(name) {
            e.insert(Balance::new());
            Some(0)
        } else {
            None
        }
    }

    pub fn remove_user(&mut self, name: &Name) -> Option<Balance> {
        self.accounts.remove(name)
    }

    pub fn get_balance(&self, name: &Name) -> Option<Balance> {
        self.accounts.get(name).cloned()
    }

    /// Получает все аккаунты с их балансами
    pub fn get_all(&self) -> Vec<(Name, u64)> {
        self.accounts
            .iter()
            .map(|(n, b)| (n.clone(), b.result))
            .collect()
    }

    /// Загружает данные из CSV-файла или создаёт хранилище с дефолтными пользователями
    pub fn load_data(file: &str) -> Storage {
        let mut storage = Storage::new();

        // Проверяем, существует ли файл
        if Path::new(file).exists() {
            // Открываем файл
            let file = File::open(file).unwrap();

            // Оборачиваем файл в BufReader
            // BufReader читает данные блоками и хранит их в буфере,
            // поэтому построчное чтение (lines()) работает быстрее, чем читать по байту
            let reader = io::BufReader::new(file);

            // Читаем файл построчно
            for line in reader.lines().map_while(Result::ok) {
                // Разделяем строку по запятой: "Name,Balance"
                let parts: Vec<&str> = line.trim().split(',').collect();

                if parts.len() == 2 {
                    let name = parts[0].to_string();
                    // Пробуем преобразовать баланс из строки в число
                    let balance: u64 = parts[1].parse().unwrap_or(0);

                    // Добавляем пользователя и выставляем баланс
                    storage.add_user(name.clone());
                    let _ = storage.deposit(&name, balance);
                }
            }
        } else {
            // если файла нет, создаём пользователей с нуля
            for u in ["John", "Alice", "Bob", "Vasya"] {
                storage.add_user(u.to_string());
            }
        }

        storage
    }

    /// Сохраняет текущее состояние Storage в CSV-файл
    pub fn save(&self, file: &str) {
        let mut data = String::new();

        // Собираем все данные в одну строку формата "Name,Balance"
        for (name, balance) in self.get_all() {
            data.push_str(&format!("{},{}\n", name, balance));
        }

        // Записываем в файл
        // Здесь мы не используем BufWriter, потому что сразу пишем всю строку целиком.
        fs::write(file, data).expect("Не удалось записать файл");
    }

    pub fn process_if_deposit(
        &mut self,
        operations: &[(bool, Name, u64)],
    ) -> Result<(), BalanceManagerError> {
        for (is_deposit, name, sum) in operations {
            if *is_deposit {
                self.deposit(name, *sum)?;
            } else {
                self.withdraw(name, *sum)?;
            }
        }
        Ok(())
    }
}

impl BalanceManager for Storage {
    fn deposit(&mut self, name: &Name, amount: u64) -> Result<(), BalanceManagerError> {
        if let Some(balance) = self.accounts.get_mut(name) {
            let op = OpKind::Deposit(amount as u32);
            let ops_refs = [&op];
            balance.process(&ops_refs);
            Ok(())
        } else {
            Err(BalanceManagerError::UserNotFound(name.clone()))
        }
    }

    fn withdraw(&mut self, name: &Name, amount: u64) -> Result<(), BalanceManagerError> {
        if let Some(balance) = self.accounts.get_mut(name) {
            if balance.result >= amount {
                let op = OpKind::Withdraw(amount as u32);
                let ops_refs = [&op];
                balance.process(&ops_refs);
                Ok(())
            } else {
                Err(BalanceManagerError::NotEnoughMoney {
                    required: amount,
                    available: balance.result,
                })
            }
        } else {
            Err(BalanceManagerError::UserNotFound(name.clone()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::{BufReader, BufWriter, Cursor, Write};

    #[test]
    fn new_storage_is_empty() {
        let bank = Storage::new();
        assert_eq!(bank.accounts.len(), 0);
    }

    #[test]
    fn add_user() {
        let mut storage = Storage::new();
        assert_eq!(storage.add_user("Alice".to_string()), Some(0)); // новый пользователь
        assert_eq!(storage.add_user("Alice".to_string()), None); // уже существует
    }

    #[test]
    fn remove_user() {
        let mut storage = Storage::new();
        storage.add_user("Bob".to_string());
        storage.deposit(&"Bob".to_string(), 100).unwrap();

        let removed = storage.remove_user(&"Bob".to_string());
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().result, 100); // удаляем и получаем баланс
        assert_eq!(storage.remove_user(&"Bob".to_string()), None); // второй раз — не найден
    }

    #[test]
    fn nonexistent_user() {
        let mut storage = Storage::new();

        // Депозит несуществующему пользователю
        assert!(storage.deposit(&"Dana".to_string(), 100).is_err());

        // Снятие у несуществующего пользователя
        assert!(storage.withdraw(&"Dana".to_string(), 50).is_err());

        // Баланс у несуществующего пользователя
        assert_eq!(storage.get_balance(&"Dana".to_string()), None);
    }

    #[test]
    fn load_data_existing_file() {
        let file_path = "load.csv";

        let mut file = File::create(file_path).unwrap();
        writeln!(file, "John,100").unwrap();
        writeln!(file, "Alice,200").unwrap();
        writeln!(file, "Bob,50").unwrap();

        let storage = Storage::load_data(file_path);

        assert_eq!(
            storage.get_balance(&"John".to_string()).unwrap().result,
            100
        );
        assert_eq!(
            storage.get_balance(&"Alice".to_string()).unwrap().result,
            200
        );
        assert_eq!(storage.get_balance(&"Bob".to_string()).unwrap().result, 50);
        assert_eq!(storage.get_balance(&"Vasya".to_string()), None);

        fs::remove_file(file_path).unwrap();
    }

    #[test]
    fn save_creates_file_with_correct_data() {
        let file_path = "save.csv";

        let mut storage = Storage::new();
        storage.add_user("John".to_string());
        storage.add_user("Alice".to_string());
        storage.deposit(&"John".to_string(), 150).unwrap();
        storage.deposit(&"Alice".to_string(), 300).unwrap();

        storage.save(file_path);

        let contents = fs::read_to_string(file_path).unwrap();
        let mut lines: Vec<&str> = contents.lines().collect();
        lines.sort();

        assert_eq!(lines, vec!["Alice,300", "John,150"]);

        fs::remove_file(file_path).unwrap();
    }

    #[test]
    fn load_data_existing_cursor() {
        // Создаём данные в памяти, как будто это CSV-файл
        let data = b"John,100\nAlice,200\nBob,50\n";
        let mut cursor = Cursor::new(&data[..]);

        // Читаем данные из Cursor
        let mut storage = Storage::new();
        let reader = BufReader::new(&mut cursor);
        for line in reader.lines() {
            let line = line.unwrap();
            let parts: Vec<&str> = line.trim().split(',').collect();
            if parts.len() == 2 {
                let name = parts[0].to_string();
                let balance: u64 = parts[1].parse().unwrap_or(0);
                storage.add_user(name.clone());
                storage.deposit(&name, balance).unwrap();
            }
        }

        assert_eq!(
            storage.get_balance(&"John".to_string()).unwrap().result,
            100
        );
        assert_eq!(
            storage.get_balance(&"Alice".to_string()).unwrap().result,
            200
        );
        assert_eq!(storage.get_balance(&"Bob".to_string()).unwrap().result, 50);
        assert_eq!(storage.get_balance(&"Vasya".to_string()), None); // нет в данных
    }

    #[test]
    fn save_writes_to_cursor_correctly() {
        let mut storage = Storage::new();
        storage.add_user("John".to_string());
        storage.add_user("Alice".to_string());
        storage.deposit(&"John".to_string(), 150).unwrap();
        storage.deposit(&"Alice".to_string(), 300).unwrap();

        let buffer = Vec::new();
        let mut cursor = Cursor::new(buffer);
        {
            let mut writer = BufWriter::new(&mut cursor);
            for (name, balance) in storage.get_all() {
                writeln!(writer, "{},{}", name, balance).unwrap();
            }
            writer.flush().unwrap();
        }

        cursor.set_position(0);
        let mut lines: Vec<String> = BufReader::new(cursor).lines().map(|l| l.unwrap()).collect();
        lines.sort();

        assert_eq!(lines, vec!["Alice,300", "John,150"]);
    }

    #[test]
    fn process_if_deposit_success() {
        let mut storage = Storage::new();
        storage.add_user("Alice".to_string());
        storage.add_user("Bob".to_string());

        let operations = vec![
            (true, "Alice".to_string(), 100), // deposit
            (true, "Bob".to_string(), 200),   // deposit
            (false, "Alice".to_string(), 50), // withdraw
        ];

        let result = storage.process_if_deposit(&operations);
        assert!(result.is_ok());
        assert_eq!(
            storage.get_balance(&"Alice".to_string()).unwrap().result,
            50
        );
        assert_eq!(storage.get_balance(&"Bob".to_string()).unwrap().result, 200);
    }

    #[test]
    fn process_if_deposit_user_not_found() {
        let mut storage = Storage::new();
        storage.add_user("Alice".to_string());

        let operations = vec![
            (true, "Alice".to_string(), 100),
            (true, "Unknown".to_string(), 50), // пользователь не существует
        ];

        let result = storage.process_if_deposit(&operations);
        assert!(result.is_err());

        match result {
            Err(BalanceManagerError::UserNotFound(name)) => {
                assert_eq!(name, "Unknown");
            }
            _ => panic!("Ожидалась ошибка UserNotFound"),
        }
    }

    #[test]
    fn process_if_deposit_not_enough_money() {
        let mut storage = Storage::new();
        storage.add_user("Alice".to_string());
        storage.deposit(&"Alice".to_string(), 50).unwrap();

        let operations = vec![
            (false, "Alice".to_string(), 100), // снять больше чем есть
        ];

        let result = storage.process_if_deposit(&operations);
        assert!(result.is_err());

        match result {
            Err(BalanceManagerError::NotEnoughMoney {
                required,
                available,
            }) => {
                assert_eq!(required, 100);
                assert_eq!(available, 50);
            }
            _ => panic!("Ожидалась ошибка NotEnoughMoney"),
        }
    }
}
