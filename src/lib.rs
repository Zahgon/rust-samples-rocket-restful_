//! Library crate root for `rust-samples-rocket-restful`.
//!
//! Exposes the application's modules as a library so they can be imported by
//! integration tests in the `tests/` directory without duplicating code.
//!
//! ## Rust note: binary vs library crates
//! Rust allows a project to have both `main.rs` (binary crate — produces an
//! executable) and `lib.rs` (library crate — exposes reusable modules). The
//! binary imports from the library, and integration tests target the library
//! directly. Tests build the application router via [`build_app`] and exercise
//! it in-process with `axum-test`, without starting a real HTTP server.

pub mod models;
pub mod repositories;
pub mod routes;
pub mod schema;
pub mod services;
pub mod state;

use axum::{Json, Router, routing::get};
use state::player_collection::PlayerCollection;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

/// OpenAPI document for the Players REST API.
///
/// ## Axum note: `#[derive(OpenApi)]`
/// This replaces Rocket's `rocket_okapi` merged-docs machinery. Each operation
/// is declared via the `#[utoipa::path(...)]` attribute on its handler and
/// listed under `paths(...)`; request/response bodies are registered under
/// `components(schemas(...))`. The generated document is served verbatim at
/// `/openapi.json` by [`build_app`].
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Players REST API",
        description = "Sample REST API with Rust and Axum",
        version = env!("CARGO_PKG_VERSION")
    ),
    paths(
        routes::health::health,
        routes::players::get_all_players,
        routes::players::get_player_by_id,
        routes::players::get_player_by_squad_number,
        routes::players::create_player,
        routes::players::update_player,
        routes::players::delete_player,
    ),
    components(schemas(
        models::player::PlayerRequest,
        models::player::PlayerResponse,
    )),
    tags(
        (name = "Players", description = "Player management endpoints"),
        (name = "Health", description = "Health check endpoint")
    )
)]
pub struct ApiDoc;

/// Serves the generated OpenAPI document as JSON.
async fn openapi_json() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}

/// Builds the complete application router with all routes and shared state.
///
/// Merges each module's `router()` (health, players), exposes the raw OpenAPI
/// document at `/openapi.json`, attaches the shared connection pool once via
/// `.with_state(...)`, and mounts the Swagger UI at `/swagger-ui`.
///
/// Shared by the binary entry point ([`main`](../rust_samples_rocket_restful/index.html))
/// and the integration tests so both exercise the exact same wiring.
pub fn build_app(state: PlayerCollection) -> Router {
    let app = Router::new()
        .merge(routes::health::router())
        .merge(routes::players::router())
        .route("/openapi.json", get(openapi_json))
        .with_state(state);

    app.merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
}
