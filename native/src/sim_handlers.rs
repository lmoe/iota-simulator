use std::sync::{Arc, RwLock};
use iota_types::storage::ReadStore;
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

