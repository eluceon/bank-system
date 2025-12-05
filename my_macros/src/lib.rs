use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

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
