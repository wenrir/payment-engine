//! This module contains the structures that are used to communicate with the payment engine.
//! The `Tx` and `Rx` structs are used to send and receive events.
//! `EngineEvent` enum is used to define the different types of events.
use super::EngineEvent;
use tokio::sync::mpsc::{channel, Receiver, Sender};

/// An Tx struct is used to send to a channel.
pub struct Tx<E>(pub Sender<E>);
/// An Rx struct is used on the channel to receive.
///
/// It contains a `Receiver` that is used to receive events.
/// The `receive` method is used to receive a message from the channel.
pub(crate) struct Rx<E> {
    /// A `Receiver` that allows receiving of type `E`.
    pub(crate) receive: Receiver<E>,
}

/// Create a new channel for the payment engine.
/// The channel is multi-producer, single-consumer channel.
/// This function is must_use.
#[must_use]
pub(crate) fn create_engine_channel() -> (Tx<EngineEvent>, Rx<EngineEvent>) {
    // Outgoing MQTT queue through multi-producer, single-consumer channel. Many values can be sent.
    let (transmit, recv) = channel(100); // usize ...
    (Tx(transmit), Rx { receive: recv })
}
