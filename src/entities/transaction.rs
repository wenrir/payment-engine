//! Transaction related data structs and operations.

use serde::Deserialize;

#[allow(dead_code)]
/// Input transaction.
#[derive(Deserialize, Debug)]
pub struct Transaction {
    #[serde(rename(deserialize = "type"))]
    /// Type of the transaction.
    pub(crate) typename: TransactionType,
    /// The client that performed the transaction
    pub(crate) client: u16,
    /// Unique transaction ID.
    pub(crate) tx: u32,
    /// Optional amount for the transaction.
    pub(crate) amount: Option<f64>,
}

/// Transaction types
#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub(crate) enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}
