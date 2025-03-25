pub mod sim_handlers;
pub mod registry;

use ::simulacrum::Simulacrum;
use std::ops::Deref;
use std::{ptr, slice};
use std::sync::{Arc, OnceLock, RwLock};
use iota_types::storage::ReadStore;
use serde::{Deserialize, Serialize};
use tempfile::tempdir;
use crate::registry::get_registry;
use log::{LevelFilter, error, info};

static EXTENDED_API_SHARED_SIMULACRUM_INITIALIZED_ENV: OnceLock<Simulacrum> =
    OnceLock::new();

fn get_or_init_shared_extended_api_simulacrum_env() -> Simulacrum {
    let mut sim = Simulacrum::new();
    let data_ingestion_path = tempdir().expect("Failed to create tempdir").into_path();

    sim.set_data_ingestion_path(data_ingestion_path);

    execute_simulacrum_transactions(&mut sim, 15);
    add_checkpoints(&mut sim, 300);
    sim.advance_epoch();

    execute_simulacrum_transactions(&mut sim, 10);
    add_checkpoints(&mut sim, 300);
    sim.advance_epoch();

    execute_simulacrum_transactions(&mut sim, 5);
    add_checkpoints(&mut sim, 300);

    sim
}

fn execute_simulacrum_transaction(sim: &mut Simulacrum) {
    let transfer_recipient = iota_types::base_types::IotaAddress::random_for_testing_only();
    let (transaction, _) = sim.transfer_txn(transfer_recipient);
    sim.execute_transaction(transaction.clone()).unwrap();
}

fn execute_simulacrum_transactions(sim: &mut Simulacrum, transactions_count: u32) {
    for _ in 0..transactions_count {
        execute_simulacrum_transaction(sim);
    }
}
fn add_checkpoints(sim: &mut Simulacrum, checkpoints_count: i32) {
    // Main use of this function is to create more checkpoints than the current
    // processing batch size, to circumvent the issue described in
    // https://github.com/iotaledger/iota/issues/2197#issuecomment-2376432709
    for _ in 0..checkpoints_count {
        sim.create_checkpoint();
    }
}


// Opaque type to hold the simulator instance
pub struct SimulatorHandle {
    simulator: Arc<RwLock<Simulacrum>>,
}

#[repr(C)]
pub struct ByteArray {
    data: *mut u8,
    length: usize,
}

#[derive(Serialize)]
struct FFIResponse<T: Serialize> {
    success: bool,
    data: Option<T>,
    error_message: Option<String>,
}

#[derive(Deserialize)]
struct FFIRequest {
    method: String,
    #[serde(default)]
    args: serde_json::Value,
}

#[no_mangle]
pub extern "C" fn simulator_create() -> *mut SimulatorHandle {
    env_logger::builder().filter_level(LevelFilter::Debug).init();

    let sim = get_or_init_shared_extended_api_simulacrum_env();
    let handle = Box::new(SimulatorHandle { simulator: Arc::new(RwLock::new(sim)) });

    info!("Created simulator handle");

    Box::into_raw(handle)
}

#[no_mangle]
pub extern "C" fn simulator_destroy(handle: *mut SimulatorHandle) {
    if !handle.is_null() {
        unsafe {
            let _ = Box::from_raw(handle);
        }
    }
}

fn serialize_to_byte_array<T: Serialize>(data: &FFIResponse<T>) -> ByteArray {
    match serde_json::to_vec(data) {
        Ok(json_bytes) => {
            let length = json_bytes.len();

            info!("serialize response: {}", String::from_utf8(json_bytes.clone()).unwrap());

            let mut boxed_bytes = json_bytes.into_boxed_slice().into_vec();
            let data_ptr = boxed_bytes.as_mut_ptr();

            std::mem::forget(boxed_bytes);

            ByteArray { data: data_ptr, length }
        },
        Err(_) => ByteArray { data: ptr::null_mut(), length: 0 },
    }
}

// Helper function to create an error response
fn error_response(message: &str) -> ByteArray {
    serialize_to_byte_array(&FFIResponse::<serde_json::Value> {
        success: false,
        data: None,
        error_message: Some(message.to_string()),
    })
}


#[no_mangle]
pub extern "C" fn simulator_execute(
    handle: *mut SimulatorHandle,
    request_data: *const u8,
    request_len: usize,
) -> ByteArray {
    let result = unsafe {
        if handle.is_null() || request_data.is_null() {
            return error_response("Invalid handle or request data");
        }

        let request_slice = slice::from_raw_parts(request_data, request_len);

        let asStr = std::str::from_utf8_unchecked(request_slice).to_owned();
        info!("Received simulacrum request {}", asStr);

        let request: FFIRequest = match serde_json::from_slice(request_slice) {
            Ok(req) => {
                req
            },
            Err(e) => return error_response(&format!("Failed to parse request: {}", e)),
        };


        let simulator = &(*handle).simulator;

        let registry = get_registry();

        let handler = match registry.get(&request.method) {
            Some(h) => h,
            None => return error_response(&format!("Unknown method: {}", request.method)),
        };

        match handler(simulator, &request.args) {
            Ok(result) => {
                // Use erased_serde to serialize the boxed trait object
                let json_value = match serde_json::to_value(&*result) {
                    Ok(v) => v,
                    Err(e) => return error_response(&format!("Failed to serialize result: {}", e)),
                };

                serialize_to_byte_array(&FFIResponse {
                    success: true,
                    data: Some(json_value),
                    error_message: None,
                })
            },
            Err(e) => error_response(&e.to_string()),
        }
    };

    result
}

#[no_mangle]
pub extern "C" fn simulator_free_byte_array(array: ByteArray) {
    if !array.data.is_null() {
        unsafe {
            let _ = Vec::from_raw_parts(array.data, array.length, array.length);
        }
    }
}