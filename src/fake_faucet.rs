use std::sync::{Arc, RwLock};
use std::time::Duration;
use axum::extract::State;
use axum::http::StatusCode;
use axum::{Json, Router};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use iota_faucet::{FaucetError, FaucetReceipt, FaucetRequest, FaucetResponse};
use simulacrum::Simulacrum;

async fn health() -> &'static str {
    "OK"
}

async fn request_gas(
    State(state): State<Arc<RwLock<Simulacrum>>>, // Use the trait object
    Json(payload): Json<FaucetRequest>,
) -> impl IntoResponse {
    let result = match payload {
        FaucetRequest::FixedAmountRequest(requests) => {
            let mut s = state.write().unwrap();
            let res = s.request_gas(requests.recipient, 20);
            s.create_checkpoint();
            s.advance_clock(Duration::new(5, 0));
            res
        }
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(FaucetResponse::from(FaucetError::Internal(
                    "Input Error.".to_string(),
                ))),
            );
        }
    };

    match result {
        Ok(o) => {
            println!("{:?}", o.summary_for_debug());
            println!("Request is successfully served");
            (
                StatusCode::CREATED,
                Json(FaucetResponse::from(FaucetReceipt { sent: Vec::new() })),
            )
        }
        Err(err) => {
            println!("Failed to request gas: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(FaucetResponse {
                    error: None,
                    transferred_gas_objects: Vec::new(),
                }),
            )
        }
    }
}

pub async fn start_fake_faucet(sim: Arc<RwLock<Simulacrum>>) {
    let app = Router::new()
        .route("/", get(health))
        .route("/gas", post(request_gas))
        .with_state(sim);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:30002").await.unwrap();
    axum::serve(listener, app).await.expect("TODO: panic message");
}