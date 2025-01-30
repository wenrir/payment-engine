//! Payment Engine
use crate::entities::channel::Rx;
use crate::entities::EngineEvent;
use crate::errors::EngineError;
use crate::filehandler::csv_to_stdout;
use std::io::Write;
use tracing::{event, span, Level};

#[allow(unused_imports)]
use super::entities::{
    account::Account,
    transaction::{Transaction, TransactionType},
};
#[allow(unused_imports)]
use super::errors::ProcessError;

#[cfg(test)]
use itertools::Itertools;
use std::collections::HashMap;
#[allow(dead_code)]
enum AccountState {
    Open,
    #[allow(dead_code)]
    Frozen,
}
struct Engine {
    #[allow(dead_code)]
    account: Account,
    #[allow(dead_code)]
    transaction_queue: HashMap<u32, f64>, // tx id & amount
    state: AccountState,
}
impl Engine {
    #[allow(unused)]
    pub(crate) fn new(acc: Account) -> Self {
        Engine {
            account: acc,
            transaction_queue: HashMap::new(),
            state: AccountState::Open,
        }
    }
}
#[allow(unused)]
pub(crate) async fn run<S: Write>(
    mut rx: Rx<EngineEvent>,
    report_stream: S,
) -> Result<(), EngineError> {
    let span = span!(Level::INFO, "payment_engine_run");
    let _guard = span.enter();
    let mut engine_map: HashMap<u16, Engine> = HashMap::new();
    /// Helper macro,
    /// This
    ///
    /// ``` rust
    /// process_transaction!(
    ///     target,
    ///     Some(e.tx),
    ///     e.amount,
    ///     deposit
    /// );
    /// ```
    ///
    /// Expands to:
    /// ``` rust
    /// if let Some(amount) = e.amount {
    ///     target.account.deposit(&amount);
    ///     Some(e.tx)
    ///         .map(|tx| target.transaction_queue.insert(tx, amount));
    /// }
    /// ```
    macro_rules! process_transaction {
        ($target:expr, $tx:expr, $amount:expr, $method:ident) => {
            if let Some(amount) = $amount {
                $target.account.$method(&amount);
                $tx.map(|tx| $target.transaction_queue.insert(tx, amount));
            }
        };
    }

    while let Some(event) = rx.receive.recv().await
    // Blocking recv, could go for polling as well.
    {
        match event {
            EngineEvent::Tx(e) => {
                let target = engine_map
                    .entry(e.client)
                    .or_insert_with(|| Engine::new(Account::new(e.client)));
                match (&target.state, &e.typename) {
                    (AccountState::Frozen, _) => {
                        event!(Level::DEBUG, "Account is frozen, do nothing.");
                    }
                    (AccountState::Open, TransactionType::Deposit) => {
                        process_transaction!(
                            target,
                            Some(e.tx),
                            e.amount,
                            deposit
                        );
                    }
                    (AccountState::Open, TransactionType::Withdrawal) => {
                        process_transaction!(
                            target,
                            Some(e.tx),
                            e.amount,
                            withdrawl
                        );
                    }
                    (AccountState::Open, TransactionType::Dispute) => {
                        process_transaction!(
                            target,
                            None,
                            target.transaction_queue.get(&e.tx).copied(),
                            dispute
                        );
                    }
                    (AccountState::Open, TransactionType::Resolve) => {
                        process_transaction!(
                            target,
                            None,
                            target.transaction_queue.get(&e.tx).copied(),
                            resolve
                        );
                    }
                    (AccountState::Open, TransactionType::Chargeback) => {
                        process_transaction!(
                            target,
                            None,
                            target.transaction_queue.get(&e.tx).copied(),
                            chargeback
                        );
                        if target.account.locked {
                            target.state = AccountState::Frozen;
                        }
                    }
                }
            }
            EngineEvent::Report() => {
                // Sort keys for testing,
                // for run I dont wich to introduce the overhead
                // of sorting.
                #[cfg(not(test))]
                let accounts: Vec<&Account> =
                    engine_map.values().map(|engine| &engine.account).collect();
                #[cfg(test)]
                let accounts: Vec<&Account> = engine_map
                    .iter()
                    .sorted_by_key(|(key, _)| *key) // Sort by key (u16)
                    .map(|(_, engine)| &engine.account) // Extract the account reference
                    .collect();
                csv_to_stdout(accounts, report_stream)?;
                break;
            }
            _ => {
                event!(Level::ERROR, "Unsupported event");
                return Err(EngineError::Event("".to_string()));
            }
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    /// Integration tests.
    use super::*;
    use crate::{
        entities::channel::create_engine_channel, filehandler::read_csv,
    };
    macro_rules! test_csv {
        ($fname:expr) => {
            concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/", $fname)
        };
    }
    macro_rules! test_client {
        ($handler:ident, $path:expr, $expected_output: expr) => {
            let (transmit, recv) = create_engine_channel();
            let $handler = tokio::spawn(async move {
                let mut result = vec![];
                let _ = run(recv, &mut result).await;
                assert_eq!(
                    String::from_utf8(result).unwrap(),
                    $expected_output
                );
            });
            let content = read_csv($path);
            assert!(content.is_ok());
            for transaction in content.unwrap().deserialize::<Transaction>() {
                let tx = transaction;
                println!("{:?}", tx);
                assert!(tx.is_ok());
                let t = transmit.0.send(EngineEvent::Tx(tx.unwrap())).await; // TODO capture this.
                assert!(t.is_ok());
            }
            let report_res = transmit.0.send(EngineEvent::Report()).await;
            assert!(report_res.is_ok());
            let handler_res = $handler.await;
            assert!(handler_res.is_ok());
        };
    }

    #[tokio::test]
    async fn test_empty_file() {
        let path = test_csv!("empty_file_test.csv");
        test_client!(handler, path, "".to_string());
    }
    #[tokio::test]
    async fn test_header_only() {
        let path = test_csv!("header_only_test.csv");
        test_client!(handler, path, "".to_string());
    }
    #[tokio::test]
    async fn test_deposit() {
        let path = test_csv!("deposit_test.csv");
        test_client!(handler, path, "client,available,held,total,locked\n1,3,0,3,false\n2,2,0,2,false\n".to_string());
    }
    #[tokio::test]
    async fn test_withdrawl() {
        let path = test_csv!("withdrawl_test.csv");
        test_client!(handler, path, "client,available,held,total,locked\n1,1.5,0,1.5,false\n2,2,0,2,false\n".to_string());
    }
    #[tokio::test]
    async fn test_dispute() {
        let path = test_csv!("dispute_test.csv");
        test_client!(
            handler,
            path,
            "client,available,held,total,locked\n1,0.5,1,1.5,false\n"
                .to_string()
        );
    }
    #[tokio::test]
    async fn test_resolve() {
        let path = test_csv!("resolve_test.csv");
        test_client!(
            handler,
            path,
            "client,available,held,total,locked\n1,1.5,0,1.5,false\n"
                .to_string()
        );
    }
    #[tokio::test]
    async fn test_invalid_resolve() {
        let path = test_csv!("resolve_invalid_test.csv");
        test_client!(
            handler,
            path,
            "client,available,held,total,locked\n1,0.5,1,1.5,false\n"
                .to_string()
        );
    }
    #[tokio::test]
    async fn test_chargeback() {
        let path = test_csv!("chargeback_test.csv");
        test_client!(
            handler,
            path,
            "client,available,held,total,locked\n1,0.5,0,0.5,true\n"
                .to_string()
        );
    }
    #[tokio::test]
    async fn test_invalid_chargeback() {
        let path = test_csv!("chargeback_invalid_test.csv");
        test_client!(
            handler,
            path,
            "client,available,held,total,locked\n1,1.5,0,1.5,false\n"
                .to_string()
        );
    }
}
