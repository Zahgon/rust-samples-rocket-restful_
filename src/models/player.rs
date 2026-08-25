//! Player domain models and type mappings.
//!
//! This module defines the representations of a player used across the
//! application layers.
//!
//! ## Representations
//!
//! | Type              | Purpose                                         | Layer      |
//! |-------------------|-------------------------------------------------|------------|
//! | [`Player`]        | Diesel Queryable — read from `players` table    | repository |
//! | [`NewPlayer`]     | Diesel Insertable — write to `players` table    | repository |
//! | [`PlayerRequest`] | Inbound API payload (ID excluded)               | routes     |
//! | [`PlayerResponse`]| Outbound API payload (ID included)              | routes     |
//!
//! ## Rust note: derive macros
//!
//! `#[derive(...)]` auto-generates standard trait implementations:
//! - `Debug` — enables `{:?}` formatting for logging and tests
//! - `Clone` — allows creating an independent copy of a value
//! - `Serialize` — converts the struct to JSON (for API responses)
//! - `Deserialize` — populates the struct from JSON (for API requests)
//! - `ToSchema` — generates the OpenAPI schema (utoipa) for this type
//! - `#[serde(rename_all = "camelCase")]` maps snake_case Rust fields to
//!   camelCase JSON keys (e.g. `first_name` → `"firstName"`)

use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

/// Internal Diesel model for reading a row from the `players` table.
///
/// Column order must match the `players` table definition in `schema.rs`.
/// `squad_number` and `starting11` are `i32` because SQLite's `INTEGER` type
/// maps to `i32` in Diesel; conversions to `u32` / `bool` happen in
/// [`PlayerResponse::from`].
#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = crate::schema::players)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Player {
    pub id: String,
    pub first_name: String,
    pub middle_name: String,
    pub last_name: String,
    pub date_of_birth: String,
    pub squad_number: i32,
    pub position: String,
    pub abbr_position: String,
    pub team: String,
    pub league: String,
    pub starting11: i32,
}

/// Internal Diesel model for inserting a new row into the `players` table.
#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::players)]
pub struct NewPlayer {
    pub id: String,
    pub first_name: String,
    pub middle_name: String,
    pub last_name: String,
    pub date_of_birth: String,
    pub squad_number: i32,
    pub position: String,
    pub abbr_position: String,
    pub team: String,
    pub league: String,
    pub starting11: i32,
}

/// Inbound payload for creating or updating a player.
///
/// Received as the JSON request body for `POST /players` and
/// `PUT /players/squadnumber/{squad_number}`.
/// The `id` field is intentionally absent — it is assigned automatically on
/// creation. `squad_number` is the natural key and is immutable once set: on
/// `PUT` the value in the request body is ignored and the existing squad number
/// is preserved.
///
/// ## Rust note: missing `Clone` and `Serialize`
/// This struct only needs `Deserialize` because it is only ever read from JSON.
/// `Clone` and `Serialize` are omitted to keep the type minimal and make
/// incorrect usage a compile-time error.
#[derive(Debug, Deserialize, ToSchema, Validate)]
#[serde(rename_all = "camelCase")]
pub struct PlayerRequest {
    #[validate(length(min = 1))]
    pub first_name: String,
    pub middle_name: String,
    #[validate(length(min = 1))]
    pub last_name: String,
    #[validate(length(min = 1))]
    pub date_of_birth: String,
    #[validate(range(min = 1, max = 99))]
    pub squad_number: u32,
    #[validate(length(min = 1))]
    pub position: String,
    #[validate(length(min = 1))]
    pub abbr_position: String,
    #[validate(length(min = 1))]
    pub team: String,
    #[validate(length(min = 1))]
    pub league: String,
    pub starting11: bool,
}

/// Outbound payload returned in API responses.
///
/// Returned as a JSON object for single-player endpoints, or as an element
/// inside a `Vec<PlayerResponse>` for the list endpoint.
///
/// ## Rust note: missing `Deserialize`
/// This struct only needs `Serialize` because it is only ever written to JSON.
/// `Deserialize` is omitted since we never parse a response back into Rust.
#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PlayerResponse {
    pub id: String,
    pub first_name: String,
    pub middle_name: String,
    pub last_name: String,
    pub date_of_birth: String,
    pub squad_number: u32,
    pub position: String,
    pub abbr_position: String,
    pub team: String,
    pub league: String,
    pub starting11: bool,
}

impl From<Player> for PlayerResponse {
    fn from(p: Player) -> Self {
        PlayerResponse {
            id: p.id,
            first_name: p.first_name,
            middle_name: p.middle_name,
            last_name: p.last_name,
            date_of_birth: p.date_of_birth,
            squad_number: p.squad_number as u32,
            position: p.position,
            abbr_position: p.abbr_position,
            team: p.team,
            league: p.league,
            starting11: p.starting11 != 0,
        }
    }
}
