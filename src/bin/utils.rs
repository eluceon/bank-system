use bank_system::{BalanceManager, Name, Storage};
use std::io::{self, BufRead, Write};

fn main() {
    let mut storage = Storage::load_data("balance.csv");

    println!("=== Bank CLI Utils ===");
    println!("Команды:");
    println!("  add <name> <balance>      - добавить пользователя");
    println!("  remove <name>             - удалить пользователя");
    println!("  deposit <name> <amount>   - пополнить баланс");
    println!("  withdraw <name> <amount>  - снять со счёта");
    println!("  balance <name>            - показать баланс");
    println!("  exit                      - выйти");

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        print!("> ");
        stdout.flush().unwrap(); // показываем приглашение

        let mut input = String::new();
        if stdin.lock().read_line(&mut input).unwrap() == 0 {
            break; // EOF
        }

        let args: Vec<&str> = input.split_whitespace().collect();
        if args.is_empty() {
            continue;
        }

        match args[0] {
            "add" => {
                if args.len() != 3 {
                    println!("Пример: add John 100");
                    continue;
                }
                let name: Name = args[1].to_string();
                let balance: u64 = match args[2].parse() {
                    Ok(b) => b,
                    Err(_) => {
                        println!("Сумма должна быть числом");
                        continue;
                    }
                };
                if storage.add_user(name.clone()).is_some() {
                    let _ = storage.deposit(&name, balance);
                    println!("Пользователь {} добавлен с балансом {}", name, balance);
                    storage.save("balance.csv");
                } else {
                    println!("Пользователь {} уже существует", name);
                }
            }
            "remove" => {
                if args.len() != 2 {
                    println!("Пример: remove John");
                    continue;
                }
                let name = args[1];
                if storage.remove_user(&name.to_string()).is_some() {
                    println!("Пользователь {} удалён", name);
                    storage.save("balance.csv");
                } else {
                    println!("Пользователь {} не найден", name);
                }
            }
            "deposit" => {
                if args.len() != 3 {
                    println!("Пример: deposit John 100");
                    continue;
                }
                let name = args[1].to_string();
                let amount: u64 = match args[2].parse() {
                    Ok(a) => a,
                    Err(_) => {
                        println!("Сумма должна быть числом");
                        continue;
                    }
                };
                match storage.deposit(&name, amount) {
                    Ok(_) => {
                        println!("Баланс пользователя {} увеличен на {}", name, amount);
                        storage.save("balance.csv");
                    }
                    Err(e) => println!("Ошибка: {}", e),
                }
            }
            "withdraw" => {
                if args.len() != 3 {
                    println!("Пример: withdraw John 100");
                    continue;
                }
                let name = args[1].to_string();
                let amount: u64 = match args[2].parse() {
                    Ok(a) => a,
                    Err(_) => {
                        println!("Сумма должна быть числом");
                        continue;
                    }
                };
                match storage.withdraw(&name, amount) {
                    Ok(_) => {
                        println!("С баланса пользователя {} снято {}", name, amount);
                        storage.save("balance.csv");
                    }
                    Err(e) => println!("Ошибка: {}", e),
                }
            }
            "balance" => {
                if args.len() != 2 {
                    println!("Пример: balance John");
                    continue;
                }
                let name: Name = args[1].to_string();
                match storage.get_balance(&name) {
                    Some(b) => println!("Баланс {}: {}", name, b.result),
                    None => println!("Пользователь {} не найден", name),
                }
            }
            "exit" => break,
            _ => println!("Неизвестная команда"),
        }
    }

    println!("Выход из CLI, все изменения сохранены.");
}
