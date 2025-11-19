use bank_system::{Name, Storage};
use std::env;

fn main() {
    // Загружаем текущее состояние банка из CSV-файла
    // Здесь демонстрация использования BufRead в методе load_data()
    // Файл читается построчно, и каждая строка преобразуется в (Name, Balance)
    let mut storage = Storage::load_data("balance.csv");

    // Получаем аргументы командной строки
    let args: Vec<String> = env::args().collect();

    // Если аргументов недостаточно, показываем справку
    if args.len() < 2 {
        eprintln!("Использование:");
        eprintln!("  add <name> <amount>");
        eprintln!("  withdraw <name> <amount>");
        eprintln!("  balance <name>");
        return;
    }

    // Разбор команды
    match args[1].as_str() {
        "deposit" => {
            // Проверяем, что указан пользователь и сумма
            if args.len() != 4 {
                eprintln!("Пример: add John 200");
                return;
            }
            let name: Name = args[2].clone();
            let amount: i64 = args[3].parse().expect("Сумма должна быть числом");

            // Пытаемся пополнить баланс
            match storage.deposit(&name, amount) {
                Ok(_) => {
                    println!("Пополнено: {} на {}", name, amount);
                    // После изменения баланса сохраняем новое состояние в CSV
                    storage.save("balance.csv");
                }
                Err(e) => println!("Ошибка: {}", e),
            }
        }
        "withdraw" => {
            if args.len() != 4 {
                eprintln!("Пример: withdraw John 100");
                return;
            }
            let name: Name = args[2].clone();
            let amount: i64 = args[3].parse().expect("Сумма должна быть числом");

            // Пытаемся снять деньги
            match storage.withdraw(&name, amount) {
                Ok(_) => {
                    println!("Снято: {} на {}", name, amount);
                    // Сохраняем изменения
                    storage.save("balance.csv");
                }
                Err(e) => println!("Ошибка: {}", e),
            }
        }
        "balance" => {
            if args.len() != 3 {
                eprintln!("Пример: balance John");
                return;
            }
            let name: Name = args[2].clone();

            // Показываем текущий баланс
            match storage.get_balance(&name) {
                Some(b) => println!("Баланс {}: {}", name, b),
                None => println!("Пользователь {} не найден", name),
            }
        }
        _ => {
            eprintln!("Неизвестная команда");
        }
    }
}