use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Fields};
use proc_macro2::TokenStream as TokenStream2;

#[proc_macro]
pub fn say_hello(input: TokenStream) -> TokenStream {
    let msg = parse_macro_input!(input as syn::LitStr); // ожидаем строковый литерал
    let expanded = quote! {
        println!("{}", #msg);
    };
    expanded.into()
}

#[proc_macro_derive(Transaction, attributes(transaction))]
pub fn transaction_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    // По умолчанию — deposit
    let mut kind = "deposit";

    for attr in &input.attrs {
        if attr.path().is_ident("transaction") {
            // Разбираем атрибут как Meta
            if let Ok(meta) = attr.parse_args::<syn::LitStr>() {
                let val = meta.value();
                if val == "withdraw" {
                    kind = "withdraw";
                } else if val == "transfer" {
                    kind = "transfer";
                }
            }
        }
    }

    let body = match kind {
        "deposit" => quote! {
            let balance = storage.accounts.entry(self.account.clone()).or_default();
            balance.result += self.amount;
        },
        "withdraw" => quote! {
            let balance = storage.accounts.entry(self.account.clone()).or_default();
            if balance.result < self.amount {
                return Err(TxError::InsufficientFunds);
            }
            balance.result -= self.amount;
        },
        "transfer" => quote! {
            let from_balance = storage
                .accounts
                .get(&self.from)
                .map(|b| b.result)
                .unwrap_or(0);

            if from_balance < self.amount {
                return Err(TxError::InsufficientFunds);
            }

            if let Some(balance) = storage.accounts.get_mut(&self.from) {
                balance.result -= self.amount;
            } else {
                return Err(TxError::InvalidAccount);
            }

            let to_balance = storage.accounts.entry(self.to.clone()).or_default();
            to_balance.result += self.amount;
        },
        _ => panic!("Unknown transaction kind"),
    };

    let expanded = quote! {
        impl Transaction for #name {
            fn apply(&self, storage: &mut Storage) -> Result<(), TxError> {
                #body
                Ok(())
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(ToSql)]
pub fn to_sql_derive(input: TokenStream) -> TokenStream {
    // Парсим вход в proc_macro2 TokenStream
    let input: DeriveInput = parse_macro_input!(input);
    let name = input.ident;

    let (field_names, field_values): (Vec<_>, Vec<_>) = match input.data {
        Data::Struct(ref data) => match &data.fields {
            Fields::Named(fields) => fields
                .named
                .iter()
                .map(|f| {
                    let ident = f.ident.as_ref().unwrap();
                    (ident, quote! { self.#ident })
                })
                .unzip(),
            _ => panic!("ToSql can only be derived for structs with named fields"),
        },
        _ => panic!("ToSql can only be derived for structs"),
    };

    // Генерация кода с proc_macro2 + quote
    let expanded: TokenStream2 = quote! {
        impl #name {
            pub fn to_sql(&self, table: &str) -> String {
                let columns = vec![#(stringify!(#field_names)),*].join(", ");
                let values = vec![#(format!("'{}'", #field_values)),*].join(", ");
                format!("INSERT INTO {} ({}) VALUES ({});", table, columns, values)
            }
        }
    };

    println!("{expanded}",);

    // Преобразуем proc_macro2::TokenStream обратно в proc_macro::TokenStream
    TokenStream::from(expanded)
}
