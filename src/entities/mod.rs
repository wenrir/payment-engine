pub(crate) mod account;
pub(crate) mod channel;
pub(crate) mod transaction;

#[derive(Debug)]
/// An enum that defines the different types of events that can be utilized the channel.
pub enum EngineEvent {
    /// Transaction Events
    Tx(transaction::Transaction),
    /// Report Events
    Report(),
}
