//! Player CRUD endpoint handlers.
//!
//! Handles HTTP requests for player management operations, delegating business
//! logic to the player service layer and converting results to HTTP responses.
//!
//! ## Key design
//!
//! | Concern         | Key used        | Notes                              |
//! |-----------------|-----------------|------------------------------------|
//! | Surrogate key   | UUID (`id`)     | Admin route: `GET /players/{uuid}` |
//! | Natural key     | `squad_number`  | All mutation routes use this       |

use crate::models::player::{PlayerRequest, PlayerResponse};
use crate::services::player_service::{self, CreateError, UpdateError};
use crate::state::player_collection::PlayerCollection;
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::get,
};
use validator::Validate;

/// Validates a request payload, mapping any validation failure to `422`.
fn validate_payload<T: Validate>(payload: &T) -> Result<(), StatusCode> {
    payload
        .validate()
        .map_err(|_| StatusCode::UNPROCESSABLE_ENTITY)
}

/// GET /players - Retrieves all players in the collection.
///
/// # Returns
/// * `200 OK` - JSON array of all players
///
/// # Example Response
/// ```json
/// [{"id": "f10f398d-b2ff-40aa-acac-51f58d129bc7", "firstName": "Lionel", "squadNumber": 10, ...}, ...]
/// ```
#[utoipa::path(
    get,
    path = "/players",
    tag = "Players",
    responses(
        (status = 200, description = "JSON array of all players", body = Vec<PlayerResponse>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_all_players(
    State(players): State<PlayerCollection>,
) -> Result<Json<Vec<PlayerResponse>>, StatusCode> {
    let mut connection = players
        .get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    player_service::get_all(&mut connection)
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// GET /players/{id} - Retrieves a specific player by UUID (admin route).
///
/// # Path Parameters
/// * `id` - UUID surrogate key of the player
///
/// # Returns
/// * `200 OK` - JSON object with player data
/// * `404 Not Found` - If no player has that UUID
///
/// # Example
/// `GET /players/f10f398d-b2ff-40aa-acac-51f58d129bc7` returns Messi's data
#[utoipa::path(
    get,
    path = "/players/{id}",
    tag = "Players",
    responses(
        (status = 200, description = "JSON object with player data", body = PlayerResponse),
        (status = 404, description = "No player has that UUID"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_player_by_id(
    State(players): State<PlayerCollection>,
    Path(id): Path<String>,
) -> Result<Json<PlayerResponse>, StatusCode> {
    let mut connection = players
        .get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    player_service::get_by_id(&mut connection, &id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

/// GET /players/squadnumber/{squad_number} - Retrieves a player by squad number.
///
/// # Path Parameters
/// * `squad_number` - Jersey number of the player (e.g., 10 for Messi)
///
/// # Returns
/// * `200 OK` - JSON object with player data
/// * `404 Not Found` - If no player has that squad number
///
/// # Example
/// `GET /players/squadnumber/10` finds the player wearing jersey #10
#[utoipa::path(
    get,
    path = "/players/squadnumber/{squad_number}",
    tag = "Players",
    responses(
        (status = 200, description = "JSON object with player data", body = PlayerResponse),
        (status = 404, description = "No player has that squad number"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_player_by_squad_number(
    State(players): State<PlayerCollection>,
    Path(squad_number): Path<u32>,
) -> Result<Json<PlayerResponse>, StatusCode> {
    let mut connection = players
        .get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    player_service::get_by_squad_number(&mut connection, squad_number)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

/// POST /players - Creates a new player with an auto-generated UUID.
///
/// # Request Body
/// JSON object with player data (ID will be assigned automatically)
///
/// # Returns
/// * `201 Created` - JSON object with the created player including assigned UUID
/// * `409 Conflict` - If squad number is already taken
///
/// # Validation
/// * Squad numbers must be unique across all players
/// * UUID is auto-generated via `uuid::Uuid::new_v4()`
///
/// # Example Request
/// ```json
/// {"firstName": "Diego", "squadNumber": 10, ...}
/// ```
#[utoipa::path(
    post,
    path = "/players",
    tag = "Players",
    request_body = PlayerRequest,
    responses(
        (status = 201, description = "Player created", body = PlayerResponse),
        (status = 409, description = "Squad number already taken"),
        (status = 422, description = "Validation failed"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn create_player(
    State(players): State<PlayerCollection>,
    Json(payload): Json<PlayerRequest>,
) -> Result<(StatusCode, Json<PlayerResponse>), StatusCode> {
    validate_payload(&payload)?;
    let mut connection = players
        .get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match player_service::create(&mut connection, payload) {
        Ok(response) => Ok((StatusCode::CREATED, Json(response))),
        Err(CreateError::DuplicateSquadNumber) => Err(StatusCode::CONFLICT),
        Err(CreateError::Database(_)) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// PUT /players/squadnumber/{squad_number} - Updates an existing player's information.
///
/// Uses `squad_number` as the natural key to look up the player. The squad
/// number and UUID are immutable — they are preserved from the existing record
/// regardless of what the request body contains.
///
/// # Path Parameters
/// * `squad_number` - Squad number (natural key) of the player to update
///
/// # Request Body
/// JSON object with complete player data
///
/// # Returns
/// * `204 No Content` - Player updated successfully, no body
/// * `404 Not Found` - If no player has that squad number
///
/// # Example
/// `PUT /players/squadnumber/10` with JSON body updates the player wearing jersey #10
#[utoipa::path(
    put,
    path = "/players/squadnumber/{squad_number}",
    tag = "Players",
    request_body = PlayerRequest,
    responses(
        (status = 204, description = "Player updated successfully"),
        (status = 404, description = "No player has that squad number"),
        (status = 422, description = "Validation failed"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn update_player(
    State(players): State<PlayerCollection>,
    Path(squad_number): Path<u32>,
    Json(payload): Json<PlayerRequest>,
) -> Result<StatusCode, StatusCode> {
    validate_payload(&payload)?;
    let mut connection = players
        .get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match player_service::update(&mut connection, squad_number, payload) {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(UpdateError::NotFound) => Err(StatusCode::NOT_FOUND),
        Err(UpdateError::Database(_)) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// DELETE /players/squadnumber/{squad_number} - Removes a player from the collection.
///
/// Uses `squad_number` as the natural key to look up the player.
///
/// # Path Parameters
/// * `squad_number` - Squad number (natural key) of the player to delete
///
/// # Returns
/// * `204 No Content` - Player successfully deleted (no response body)
/// * `404 Not Found` - If no player has that squad number
#[utoipa::path(
    delete,
    path = "/players/squadnumber/{squad_number}",
    tag = "Players",
    responses(
        (status = 204, description = "Player deleted"),
        (status = 404, description = "No player has that squad number"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn delete_player(
    State(players): State<PlayerCollection>,
    Path(squad_number): Path<u32>,
) -> Result<StatusCode, StatusCode> {
    let mut connection = players
        .get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match player_service::delete(&mut connection, squad_number) {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Builds the player routes into an Axum router.
///
/// Returns a `Router<PlayerCollection>` (state not yet applied). The shared
/// connection pool is attached centrally when the routers are merged in
/// [`crate::build_app`]. Each path binds its HTTP methods explicitly, replacing
/// Rocket's attribute-driven route mounting.
pub fn router() -> Router<PlayerCollection> {
    Router::new()
        .route("/players", get(get_all_players).post(create_player))
        .route("/players/{id}", get(get_player_by_id))
        .route(
            "/players/squadnumber/{squad_number}",
            get(get_player_by_squad_number)
                .put(update_player)
                .delete(delete_player),
        )
}
