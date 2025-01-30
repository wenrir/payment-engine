//! Transaction related data structs and operations.

use serde::Deserialize;

#[derive(Deserialize, Debug)]
/// Input transactions.
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

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
/// Transaction types
pub(crate) enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}
