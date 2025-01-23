use axum::{
    routing::{get, post},
    Json, Router,
};
use leptos::*;
use practical_6::server::set_up_server;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = set_up_server().await;
    Ok(())
}
