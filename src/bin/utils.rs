use bank_system::transaction::Withdraw;
use bank_system::{BalanceManager, Deposit, Name, Storage, Transaction, Transfer};
use std::io::{self, BufRead, Write};

fn main() {
    let mut storage = Storage::load_data("balance.csv");

    println!("=== Bank CLI Utils ===");
    println!("Команды:");
    println!("  add <name> <balance>         - добавить пользователя");
    println!("  remove <name>                - удалить пользователя");
    println!("  deposit <name> <amount>      - пополнить баланс (транзакция)");
    println!("  withdraw <name> <amount>     - снять со счёта");
    println!("  transfer <from> <to> <amount> - перевод между счетами");
    println!("  + deposit <name> <amount> transfer <from> <to> <amount>");
    println!("                               - комбинированная транзакция");
    println!("  balance <name>               - показать баланс");
    println!("  exit                         - выйти");

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

                let tx = Deposit {
                    account: name.clone(),
                    amount,
                };

                match tx.apply(&mut storage) {
                    Ok(_) => {
                        println!("Транзакция: депозит {} на {}", name, amount);
                        storage.save("balance.csv");
                    }
                    Err(e) => println!("Ошибка транзакции: {:?}", e),
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

                let tx = Withdraw {
                    account: name.clone(),
                    amount,
                };

                match tx.apply(&mut storage) {
                    Ok(_) => {
                        println!(
                            "Транзакция: с баланса пользователя {} снято {}",
                            name, amount
                        );
                        storage.save("balance.csv");
                    }
                    Err(e) => println!("Ошибка транзакции: {:?}", e),
                }
            }
            "transfer" => {
                if args.len() != 4 {
                    println!("Пример: transfer Alice Bob 100");
                    continue;
                }
                let from = args[1].to_string();
                let to = args[2].to_string();
                let amount: u64 = match args[3].parse() {
                    Ok(a) => a,
                    Err(_) => {
                        println!("Сумма должна быть числом");
                        continue;
                    }
                };

                let tx = Transfer {
                    from: from.clone(),
                    to: to.clone(),
                    amount,
                };

                match tx.apply(&mut storage) {
                    Ok(_) => {
                        println!("Транзакция: перевод {} -> {} на {}", from, to, amount);
                        storage.save("balance.csv");
                    }
                    Err(e) => println!("Ошибка транзакции: {:?}", e),
                }
            }
            "+" => {
                if args.len() != 8 {
                    println!(
                        "Пример: + deposit Alice 100 transfer Alice Bob 30 (получено {} аргументов)",
                        args.len()
                    );
                    continue;
                }

                let deposit = Deposit {
                    account: args[2].to_string(),
                    amount: args[3].parse().unwrap_or(0),
                };

                let transfer = Transfer {
                    from: args[5].to_string(),
                    to: args[6].to_string(),
                    amount: args[7].parse().unwrap_or(0),
                };

                let combined_tx = deposit + transfer;

                match combined_tx.apply(&mut storage) {
                    Ok(_) => {
                        println!("Комбинированная транзакция выполнена!");
                        storage.save("balance.csv");
                    }
                    Err(e) => println!("Ошибка при выполнении: {:?}", e),
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
