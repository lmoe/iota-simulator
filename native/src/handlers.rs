use serde::{Serialize, Deserialize};
use serde_json;
use std::collections::HashMap;
use std::sync::{Arc, RwLock, Once};
use std::error::Error;
use std::fmt;
use iota_types::storage::ReadStore;
use simulacrum::Simulacrum;

static mut REGISTRY: Option<HandlerRegistry> = None;
static INIT: Once = Once::new();

pub fn get_registry() -> &'static HandlerRegistry {
    unsafe {
        INIT.call_once(|| {
            let mut registry = HandlerRegistry::new();
            register_handlers(&mut registry);
            REGISTRY = Some(registry);
        });
        REGISTRY.as_ref().unwrap()
    }
}

fn register_handlers(registry: &mut HandlerRegistry) {
    registry.register("getLatestCheckpoint", get_latest_checkpoint_handler);
}

#[derive(Debug)]
pub struct HandlerError {
    message: String,
}

impl HandlerError {
    pub fn new(message: &str) -> Self {
        HandlerError {
            message: message.to_string(),
        }
    }
}

impl fmt::Display for HandlerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Handler error: {}", self.message)
    }
}

impl Error for HandlerError {}

pub type HandlerFn = fn(&Arc<RwLock<Simulacrum>>, &serde_json::Value) -> Result<Box<dyn erased_serde::Serialize>, HandlerError>;

pub struct HandlerRegistry {
    handlers: HashMap<String, HandlerFn>,
}

impl HandlerRegistry {
    pub fn new() -> Self {
        HandlerRegistry {
            handlers: HashMap::new(),
        }
    }

    pub fn register(&mut self, name: &str, handler: HandlerFn) {
        self.handlers.insert(name.to_string(), handler);
    }

    pub fn get(&self, name: &str) -> Option<&HandlerFn> {
        self.handlers.get(name)
    }
}

fn get_latest_checkpoint_handler(
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

