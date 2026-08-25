//! Sample REST API with Rust and Axum
//!
//! This is the application entry point that initializes the Axum web framework,
//! initializes player data, and serves all API routes.
//!
//! The application follows a modular architecture:
//! - `models`: Data structures and conversions
//! - `state`: Thread-safe application state management
//! - `services`: Pure business logic functions
//! - `routes`: HTTP endpoint handlers
//!
//! For more details, see the project README.

use rust_samples_rocket_restful::build_app;
use rust_samples_rocket_restful::state::player_collection::initialize_database;

/// Configures and launches the Axum web server.
///
/// ## Rust note: `#[tokio::main]` macro
/// `#[tokio::main]` is a procedural macro that sets up the Tokio async runtime
/// and rewrites this async `main` into a synchronous entry point. It replaces
/// Rocket's `#[launch]` macro: instead of returning a `Rocket<Build>` instance
/// for the framework to launch, we build the router with [`build_app`] and drive
/// the server ourselves via `axum::serve`.
///
/// The listener is bound to `0.0.0.0:9000` in code — the port and address that
/// previously lived in `Rocket.toml`.
#[tokio::main]
async fn main() {
    let database = initialize_database();
    let app = build_app(database);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:9000")
        .await
        .expect("Failed to bind to 0.0.0.0:9000");
    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}
