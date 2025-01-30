//! Payment Engine
use crate::entities::channel::Rx;
use crate::entities::EngineEvent;
use crate::errors::EngineError;
use crate::filehandler::csv_to_stdout;
use std::io::Write;

use super::entities::{account::Account, transaction::TransactionType};

#[cfg(test)]
use itertools::Itertools;
use std::collections::HashMap;
struct History {
    amount: f64,
    dispute: bool,
}
impl History {
    fn new(_amount: f64, _dispute: bool) -> Self {
        History {
            amount: _amount,
            dispute: _dispute,
        }
    }
}

type Transactions = HashMap<u32, History>; // tx id & History, no need to store the entire transaction.
type Accounts = HashMap<u16, Account>; // tx id & History, no need to store the entire transaction.

struct Engine {
    account: Accounts,
    transaction_history: Transactions,
}

impl Engine {
    pub(crate) fn new() -> Self {
        Engine {
            account: HashMap::new(),
            transaction_history: HashMap::new(),
        }
    }
}
pub(crate) async fn run<S: Write>(
    mut rx: Rx<EngineEvent>,
    report_stream: S,
) -> Result<(), EngineError> {
    let mut engine = Engine::new();
    /// Helper macro for decision logic.
    /// There are 2 transaction categories, [deposit, withdrawal] and [dispute, resolve, chargeback]
    /// Macro created to minimize repetition in code.
    /// Type 1 inputs
    /// + target => Target account
    /// + tx => Transaction id
    /// + amount => Amount
    /// + method => What implemented method to use.
    /// Type 2 inputs
    /// + target => Target account
    /// + tx => Transaction id
    /// + [cond, is_dispute] =>
    /// + method => What implemented method to use.
    macro_rules! process_transaction {
        // Transaction type 1 (deposit/withdrawal)
        (transaction_type_1, $target:expr, $tx:expr, $amount:expr, $method:ident) => {
            if engine.transaction_history.contains_key(&$tx) {
                continue; // Tx already exists, do not re add it to history.
            }
            if let Some(amount) = $amount {
                if let Err(_) = $target.$method(&amount) {
                    continue; // Continue in case of error.
                }
                engine
                    .transaction_history
                    .insert($tx, History::new(amount, false));
            }
        };
        // Transaction type 2 (dispute/resolve/chargeback)
        (transaction_type_2, $target:expr, $tx:expr, [$cond:expr, $is_dispute:expr], $method:ident) => {
            // There are quite many branches here ... probably would be a better idea to break out the macro to functions.
            if let Some(transaction) = engine.transaction_history.get(&$tx) {
                if transaction.dispute != $cond {
                    continue; // Let's not go unnecessarily deep.
                }

                if let Some(amount) = Some(transaction.amount) {
                    if let Err(_) = $target.$method(&amount) {
                        continue; // Don't break on error.
                    }

                    engine
                        .transaction_history
                        .insert($tx, History::new(amount, $is_dispute));
                }
            }
        };
    }

    while let Some(event) = rx.receive.recv().await
    // Blocking recv, could go for polling as well.
    {
        match event {
            EngineEvent::Tx(e) => {
                let target = engine
                    .account
                    .entry(e.client)
                    .or_insert_with(|| Account::new(e.client));
                match (&target.locked, &e.typename) {
                    (true, _) => {
                        continue;
                    }
                    (false, TransactionType::Deposit) => {
                        process_transaction!(
                            transaction_type_1,
                            target,
                            e.tx,
                            e.amount,
                            deposit
                        );
                    }
                    (false, TransactionType::Withdrawal) => {
                        process_transaction!(
                            transaction_type_1,
                            target,
                            e.tx,
                            e.amount,
                            withdrawl
                        );
                    }
                    (false, TransactionType::Dispute) => {
                        process_transaction!(
                            transaction_type_2,
                            target,
                            e.tx,
                            [false, true],
                            dispute
                        );
                    }
                    (false, TransactionType::Resolve) => {
                        process_transaction!(
                            transaction_type_2,
                            target,
                            e.tx,
                            [true, false],
                            resolve
                        );
                    }
                    (false, TransactionType::Chargeback) => {
                        process_transaction!(
                            transaction_type_2,
                            target,
                            e.tx,
                            [true, false],
                            chargeback
                        );
                    }
                }
            }
            EngineEvent::Report() => {
                // Sort keys for testing, avoid overhead for release build.
                #[cfg(not(test))]
                let accounts: Vec<&Account> = engine.account.values().collect();

                #[cfg(test)]
                let accounts: Vec<&Account> = engine
                    .account
                    .iter()
                    .sorted_by_key(|(key, _)| *key) // Sort by key (u16) for tests
                    .map(|(_, a)| a) // Extract account reference
                    .collect();
                csv_to_stdout(accounts, report_stream)?;
                break;
            }
        }
    }
    Ok(())
}
