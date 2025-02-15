use std::io::stdin;
use std::sync::OnceLock;
use ::simulacrum::Simulacrum;
use iota_json_rpc_api::BridgeReadApiClient;

use iota_types::{
    storage::ReadStore,
};
use axum::{
    response::IntoResponse,
};

mod simulacrum;
mod fake_faucet;
mod simulacum_reader_wrapper;
mod simulacrum_control_api;

static EXTENDED_API_SHARED_SIMULACRUM_INITIALIZED_ENV: OnceLock<simulacrum::SimulacrumTestSetup> =
    OnceLock::new();

fn get_or_init_shared_extended_api_simulacrum_env() -> &'static simulacrum::SimulacrumTestSetup {
    simulacrum::SimulacrumTestSetup::get_or_init(
        "extended_api",
        |data_ingestion_path| {
            let mut sim = Simulacrum::new();

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
        },
        &EXTENDED_API_SHARED_SIMULACRUM_INITIALIZED_ENV,
    )
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



fn main() {
    env_logger::init();
    let sim = get_or_init_shared_extended_api_simulacrum_env();
    println!("Simulacrum Server running!");
    let mut s=String::new();
    stdin().read_line(&mut s).expect("");
}
