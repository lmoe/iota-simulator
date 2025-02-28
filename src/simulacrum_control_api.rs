use crate::consts::get_control_binding_ip;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use iota_types::crypto::AuthorityStrongQuorumSignInfo;
use iota_types::messages_checkpoint::CheckpointSummary;
use serde::{Deserialize, Serialize};
use simulacrum::Simulacrum;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use axum::response::IntoResponse;

async fn health() -> &'static str {
    "OK"
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Checkpoint {
    pub summary: CheckpointSummary,
    pub authority: AuthorityStrongQuorumSignInfo,
}

async fn create_checkpoint(
    State(state): State<Arc<RwLock<Simulacrum>>>, // Use the trait object
) -> Result<Json<Checkpoint>, StatusCode> {
    let mut s = state.write().unwrap();
    let c = s.create_checkpoint().clone();

    Ok(Json(Checkpoint {
        authority: c.auth_sig().clone(),
        summary: c.data().clone(),
    }))
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AdvanceClockRequest {
    pub duration: u32,
}

async fn advance_clock(
    State(state): State<Arc<RwLock<Simulacrum>>>,
    Json(payload): Json<AdvanceClockRequest>,
) -> impl IntoResponse {
    let mut s = state.write().unwrap();
    s.advance_clock(Duration::from_millis(payload.duration as u64));
    ()
}

async fn advance_epoch(State(state): State<Arc<RwLock<Simulacrum>>>) -> impl IntoResponse {
    let mut s = state.write().unwrap();
    s.advance_epoch()
}

async fn get_checkpoint(State(state): State<Arc<RwLock<Simulacrum>>>) -> Result<Json<Checkpoint>, StatusCode> {
    let mut s = state.write().unwrap();
    let checkpoint = s.store().get_highest_checkpoint().clone().unwrap();
    Ok(Json(Checkpoint {
        authority: checkpoint.auth_sig().clone(),
        summary: checkpoint.data().clone(),
    }))
}

pub async fn start_control_api(sim: Arc<RwLock<Simulacrum>>) -> std::io::Result<()> {
    let app = Router::new()
        .route("/", get(health))
        .route("/checkpoint", get(get_checkpoint))
        .route("/create_checkpoint", post(create_checkpoint))
        .route("/advance_clock", post(advance_clock))
        .route("/advance_epoch", post(advance_epoch))
        .with_state(sim);

    let listener = tokio::net::TcpListener::bind(get_control_binding_ip())
        .await
        .unwrap();

    axum::serve(listener, app).await
}
