#[macro_export]
macro_rules! tx_chain {
    ( $first:expr $(, $rest:expr )* $(,)? ) => {{
        let tx = $first;
        $(
            let tx = $crate::TxCombinator { t1: tx, t2: $rest };
        )*
        tx
    }};
}
