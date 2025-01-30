//! Payment Engine
use crate::entities::channel::Rx;
use crate::entities::EngineEvent;
use crate::errors::EngineError;
use crate::filehandler::csv_to_stdout;
use std::io::Write;
use tracing::{event, span, Level};

use super::entities::{account::Account, transaction::TransactionType};

#[cfg(test)]
use itertools::Itertools;
use std::collections::HashMap;
enum AccountState {
    Open,
    Frozen,
}
struct Engine {
    account: Account,
    transaction_queue: HashMap<u32, f64>, // tx id & amount
    state: AccountState,
}
impl Engine {
    pub(crate) fn new(acc: Account) -> Self {
        Engine {
            account: acc,
            transaction_queue: HashMap::new(),
            state: AccountState::Open,
        }
    }
}
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
    /// ``` ignore
    /// process_transaction!(
    ///     target,
    ///     Some(e.tx),
    ///     e.amount,
    ///     deposit
    /// );
    /// ```
    ///
    /// Expands to:
    /// ``` ignore
    /// if let Some(amount) = e.amount {
    ///     target.account.deposit(&amount);
    ///     Some(e.tx)
    ///         .map(|tx| target.transaction_queue.insert(tx, amount));
    /// }
    /// ```
    macro_rules! process_transaction {
        ($target:expr, $tx:expr, $amount:expr, $method:ident) => {
            if let Some(amount) = $amount {
                let _ = $target.account.$method(&amount);
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
        }
    }
    Ok(())
}
