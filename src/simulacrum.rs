use std::{
    path::PathBuf,
    sync::{Arc, OnceLock},
};
use std::sync::{RwLock};
use iota_indexer::{
    errors::IndexerError,
    indexer::Indexer,
    store::{PgIndexerStore},
    test_utils::{start_test_indexer, ReaderWriterConfig},
    IndexerConfig,
};

use iota_metrics::init_metrics;
use jsonrpsee::http_client::{HttpClient, HttpClientBuilder};
use simulacrum::{Simulacrum};
use tempfile::tempdir;
use tokio::{runtime::Runtime, task::JoinHandle};
use crate::fake_faucet::start_fake_faucet;
use crate::simulacrum_control_api::start_control_api;
use crate::simulacum_reader_wrapper::SimulacrumReaderWrapper;

// Constants for database and default ports
const POSTGRES_URL: &str = "postgres://postgres:postgrespw@localhost:5432";
const DEFAULT_DB: &str = "iota_indexer";
const DEFAULT_INDEXER_PORT: u16 = 30000;
const DEFAULT_RPC_PORT: u16 = 30001;

/// Functions to define binding and client IPs
fn get_binding_ip(port: u16) -> String {
    format!("0.0.0.0:{port}")
}

fn get_client_ip(port: u16) -> String {
    format!("http://127.0.0.1:{port}")
}

fn get_indexer_binding_ip() -> String {
    get_binding_ip(DEFAULT_INDEXER_PORT)
}

fn get_rpc_binding_ip() -> String {
    get_binding_ip(DEFAULT_RPC_PORT)
}

fn get_indexer_client_ip() -> String {
    get_client_ip(DEFAULT_INDEXER_PORT)
}

fn get_rpc_client_ip() -> String {
    get_client_ip(DEFAULT_RPC_PORT)
}

pub struct SimulacrumTestSetup {
    pub runtime: Runtime,
    pub sim: Arc<RwLock<Simulacrum>>,
    pub store: PgIndexerStore,
    pub client: HttpClient,
}

impl SimulacrumTestSetup {
    pub fn get_or_init<'a>(
        unique_env_name: &str,
        env_initializer: impl Fn(PathBuf) -> Simulacrum,
        initialized_env_container: &'a OnceLock<SimulacrumTestSetup>,
    ) -> &'a SimulacrumTestSetup {
        initialized_env_container.get_or_init(|| {
            let runtime = Runtime::new().expect("Failed to create Tokio runtime");
            let data_ingestion_path = tempdir().expect("Failed to create tempdir").into_path();
            let sim = Arc::new(RwLock::new(env_initializer(data_ingestion_path.clone())));

            let db_name = format!("simulacrum_env_db_{}", unique_env_name);
            let (_, _, store, _, client) = runtime.block_on(start_simulacrum_rest_api_with_read_write_indexer(
                sim.clone(),
                data_ingestion_path,
                Some(&db_name),
            ));

            SimulacrumTestSetup {
                runtime,
                sim,
                store,
                client,
            }
        })
    }
}

fn get_indexer_db_url(database_name: Option<&str>) -> String {
    match database_name {
        Some(db_name) => format!("{POSTGRES_URL}/{db_name}"),
        None => format!("{POSTGRES_URL}/{DEFAULT_DB}"),
    }
}

fn start_indexer_reader(
    data_ingestion_path: PathBuf,
    database_name: Option<&str>,
) -> u16 {
    let db_url = get_indexer_db_url(database_name);
    let config = IndexerConfig {
        db_url: Some(db_url.clone().into()),
        rpc_client_url: get_rpc_client_ip(),
        reset_db: true,
        rpc_server_worker: true,
        rpc_server_port: DEFAULT_INDEXER_PORT,
        data_ingestion_path: Some(data_ingestion_path),
        ..Default::default()
    };

    let registry = prometheus::Registry::default();
    init_metrics(&registry);

    tokio::spawn(async move { Indexer::start_reader(&config, &registry, db_url).await });
    DEFAULT_INDEXER_PORT
}

pub async fn start_simulacrum_rest_api_with_write_indexer(
    sim: Arc<RwLock<Simulacrum>>,
    data_ingestion_path: PathBuf,
    database_name: Option<&str>,
) -> (
    JoinHandle<()>,
    JoinHandle<()>,
    PgIndexerStore,
    JoinHandle<Result<(), IndexerError>>,
) {
    let sim_for_server = Arc::clone(&sim);
    let server_handle = tokio::spawn(async move {
        let sim_wrapper = Arc::new(SimulacrumReaderWrapper { inner: sim_for_server });
        iota_rest_api::RestService::new_without_version(sim_wrapper)
            .start_service(
                get_rpc_binding_ip()
                    .parse()
                    .expect("Invalid server URL"),
            )
            .await;
    });

    let (pg_store, pg_handle) = start_test_indexer(
        Some(get_indexer_db_url(None)),
        get_rpc_client_ip(),
        ReaderWriterConfig::writer_mode(None),
        Some(data_ingestion_path),
        database_name,
    )
        .await;

    let sim_for_faucet = Arc::clone(&sim);
    let faucet_handle = tokio::spawn(async move {
        start_fake_faucet(sim_for_faucet).await;
    });

    let sim_for_ctrl = Arc::clone(&sim);
    tokio::spawn(async move {
        start_control_api(sim_for_ctrl).await;
    });

    (server_handle, faucet_handle, pg_store, pg_handle)
}

pub async fn start_simulacrum_rest_api_with_read_write_indexer(
    sim: Arc<RwLock<Simulacrum>>,
    data_ingestion_path: PathBuf,
    database_name: Option<&str>,
) -> (
    JoinHandle<()>,
    JoinHandle<()>,
    PgIndexerStore,
    JoinHandle<Result<(), IndexerError>>,
    HttpClient,
) {
    let (server_handle, faucet_handle, pg_store, pg_handle) =
        start_simulacrum_rest_api_with_write_indexer(sim, data_ingestion_path.clone(), database_name)
            .await;

    start_indexer_reader(data_ingestion_path, database_name);

    let rpc_client = HttpClientBuilder::default()
        .build(get_rpc_client_ip())
        .expect("Failed to build RPC client");

    println!("Indexer: {}", get_indexer_client_ip());
    println!("RPC URL: {}", get_rpc_client_ip());

    (server_handle, faucet_handle, pg_store, pg_handle, rpc_client)
}