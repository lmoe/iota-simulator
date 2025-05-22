use std::sync::{Arc, RwLock};
use fastcrypto::encoding::{Base64, Encoding};
use serde::__private::de::borrow_cow_str;
use serde::de::DeserializeOwned;
use iota_types::storage::ReadStore;
use iota_types::transaction::{SenderSignedData, Transaction, TransactionData, TransactionKind};
use serde::Deserialize;
use simulacrum::Simulacrum;
use crate::registry::{HandlerError, HandlerRegistry};

pub fn register_handlers(registry: &mut HandlerRegistry) {
    registry.register("getLatestCheckpoint", get_latest_checkpoint_handler);
}

pub fn get_latest_checkpoint_handler(
    simulator: &Arc<RwLock<Simulacrum>>,
    _args: &serde_json::Value
) -> Result<Box<dyn erased_serde::Serialize>, HandlerError> {
    let sim = simulator.read().map_err(|_| HandlerError::new("Failed to acquire read lock"))?;

    match sim.get_latest_checkpoint() {
        Ok(checkpoint) => {
            let serializable = checkpoint.serializable();
            Ok(Box::new(serializable))
        },
        Err(e) => Err(HandlerError::new(&format!("Failed to get latest checkpoint: {}", e))),
    }
}

#[derive(Debug, Deserialize)]
pub struct TransactionRequest {
    TxDataBytes: String,
    Signatures: Vec<String>
}

fn to_request(value: serde_json::Value) -> TransactionRequest
{
    serde_json::from_value(value).unwrap()
}

fn deserialize_tx_data<T>(tx_bytes: &str) -> Result<T, HandlerError>
where
    T: DeserializeOwned,
{
    let res = bcs::from_bytes(
        &fastcrypto::encoding::Base64::decode(tx_bytes)
            .map_err(|e| {
                HandlerError::new(&format!(
                    "Unable to deserialize transaction bytes from Base64: {e}"
                ))
            })
            .unwrap(),
    )
    .map_err(|e| {
        HandlerError::new(&format!(
            "Unable to deserialize transaction bytes as BCS: {e}"
        ))
    })?;
  
    match res {
        Ok(tx) => Ok(tx),
        Err(e) => Err(e),
    }
}


pub fn execute_transaction_block(simulator: &Arc<RwLock<Simulacrum>>, args: serde_json::Value) -> Result<Box<dyn erased_serde::Serialize>, HandlerError> {
    let mut sim = simulator.write().map_err(|_| HandlerError::new("Failed to acquire read lock"))?;
    let dec = to_request(args);

    bcs::from_bytes(dec.TxDataBytes.as_slice());

    let txData = TransactionData::new(TransactionKind::ProgrammableTransaction, )
    let tx = sim.execute_transaction(Transaction::new(SenderSignedData::new()));
}