use serde_json;
use std::collections::HashMap;
use std::sync::{Arc, RwLock, Once};
use std::error::Error;
use std::fmt;
use simulacrum::Simulacrum;

use crate::sim_handlers;
use crate::sim_handlers::register_handlers;

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
