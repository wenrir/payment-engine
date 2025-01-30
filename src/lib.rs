//! Payment engine lib

mod engine;
mod entities;
mod errors;
mod filehandler;

use std::io::stdout;

use crate::engine::run;
use crate::entities::channel::{create_engine_channel, Tx};
use crate::entities::transaction::Transaction;
use crate::entities::EngineEvent;
use crate::errors::EngineError;
use crate::filehandler::read_csv;

/// TODO
pub async fn run_from_csv(path: &str) -> Result<(), EngineError> {
    let mut content = read_csv(path)?;
    let (transmit, recv) = create_engine_channel();
    let payment_engine_handler = tokio::spawn(run(recv, stdout()));
    for transaction in content.deserialize::<Transaction>() {
        let tx = transaction?;
        transmit.0.send(EngineEvent::Tx(tx)).await?; // TODO capture this.
    }

    transmit.0.send(EngineEvent::Report()).await?;
    let _ = payment_engine_handler.await?;
    Ok(())
}

/// Starts the payment engine in standalone mode
/// Continously reads for transactions,
/// and returns Tx for user to communicate with engine.
pub async fn run_stand_alone() -> Result<Tx<EngineEvent>, EngineError> {
    println!("standalone");
    let (transmit, recv) = create_engine_channel();
    let _ = tokio::spawn(run(recv, stdout())).await;
    Ok(transmit)
}
