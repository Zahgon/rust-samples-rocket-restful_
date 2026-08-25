//! Health check endpoint.
//!
//! Provides a simple liveness probe so load balancers and monitoring tools
//! can verify that the API process is running and accepting connections.

use crate::state::player_collection::PlayerCollection;
use axum::{Router, http::StatusCode, routing::get};

/// Returns `200 OK` if the service is running.
///
/// ## Axum note: `#[utoipa::path]` vs. route registration
/// Unlike Rocket, Axum does not attach the HTTP method and path to the handler
/// via an attribute macro. The `#[utoipa::path(...)]` attribute here only
/// documents the operation for the generated OpenAPI spec; the method and path
/// are bound separately in [`router`].
#[utoipa::path(
    get,
    path = "/health",
    tag = "Health",
    responses((status = 200, description = "Service is running"))
)]
pub async fn health() -> StatusCode {
    StatusCode::OK
}

/// Builds the health check router.
///
/// Each route module exposes a `router()` returning a `Router<PlayerCollection>`
/// (state not yet applied). The shared connection pool is attached once,
/// centrally, when the routers are merged in [`crate::build_app`]. This keeps
/// route registration modular and makes it easy to add or remove endpoints
/// without touching `main.rs`.
pub fn router() -> Router<PlayerCollection> {
    Router::new().route("/health", get(health))
}
