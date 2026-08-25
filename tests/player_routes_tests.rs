// Integration tests for player route handlers
// Exercises the full HTTP request/response cycle using axum-test's in-process
// TestServer. Each test gets a fresh application router backed by an in-memory
// SQLite database seeded with the full 26-player Argentina 2022 World Cup squad.

mod common;

use axum::http::StatusCode;
use axum_test::TestServer;
use rust_samples_rocket_restful::{build_app, state::player_collection::initialize_test_database};

// Full 26-player seed — used by all tests except POST creation
fn setup_server() -> TestServer {
    let database = initialize_test_database();
    let app = build_app(database);
    TestServer::new(app)
}

// Standard 26-player seed (squads 1–26) — used by POST creation tests.
// Squad 27 (Lo Celso fixture) is not in the seed, so POST creation succeeds.
fn setup_server_for_post() -> TestServer {
    use rust_samples_rocket_restful::services::player_service;

    let pool = initialize_test_database();
    {
        let mut conn = pool.get().expect("pool connection");
        // Lo Celso is not in the 26-player seed, so this is a no-op —
        // kept for symmetry with previous setup pattern.
        player_service::delete(&mut conn, 27).ok();
    }
    let app = build_app(pool);
    TestServer::new(app)
}

// JSON mirror of common::player_request_for_creation() — Giovani Lo Celso, squad 27
fn player_request_for_creation_json() -> serde_json::Value {
    serde_json::json!({
        "firstName": "Giovani",
        "middleName": "",
        "lastName": "Lo Celso",
        "dateOfBirth": "1996-07-09T00:00:00.000Z",
        "squadNumber": 27,
        "position": "Central Midfield",
        "abbrPosition": "CM",
        "team": "Real Betis Balompié",
        "league": "La Liga",
        "starting11": false
    })
}

// PUT fixture for Emiliano Martínez targeting squad 23.
// squadNumber is deliberately set to 99 (≠ 23) to prove the route param wins.
fn player_request_for_update_json() -> serde_json::Value {
    serde_json::json!({
        "firstName": "Emiliano",
        "middleName": "",
        "lastName": "Martínez",
        "dateOfBirth": "1992-09-02T00:00:00.000Z",
        "squadNumber": 99,
        "position": "Goalkeeper",
        "abbrPosition": "GK",
        "team": "Aston Villa FC",
        "league": "Premier League",
        "starting11": true
    })
}

// GET /openapi.json -----------------------------------------------------------

// GET /openapi.json returns 200 OK with a valid OpenAPI 3 payload
#[tokio::test]
async fn test_request_get_openapi_json_response_status_ok() {
    // Arrange
    let server = setup_server();
    // Act
    let response = server.get("/openapi.json").await;
    // Assert
    assert_eq!(response.status_code(), StatusCode::OK);
    let body = response.text();
    assert!(body.contains("\"openapi\""));
}

// GET /health -----------------------------------------------------------------

// GET /health returns 200 OK
// @coderabbitai: no body assertion — health() returns `StatusCode` only (no
// response body). If the handler is ever updated to return a payload
// (e.g. JSON health object), this test should be extended accordingly.
#[tokio::test]
async fn test_request_get_health_response_status_ok() {
    // Arrange
    let server = setup_server();
    // Act
    let response = server.get("/health").await;
    // Assert
    assert_eq!(response.status_code(), StatusCode::OK);
}

// GET /players ----------------------------------------------------------------

// GET /players returns 200 OK with all 26 players
#[tokio::test]
async fn test_request_get_players_all_response_status_ok() {
    // Arrange
    let server = setup_server();
    // Act
    let response = server.get("/players").await;
    // Assert
    assert_eq!(response.status_code(), StatusCode::OK);
    let body: serde_json::Value = response.json();
    assert_eq!(body.as_array().unwrap().len(), 26);
}

// GET /players returns a body where every element has the expected fields
#[tokio::test]
async fn test_request_get_players_all_response_body_structure() {
    // Arrange
    let server = setup_server();
    // Act
    let response = server.get("/players").await;
    // Assert
    let body: serde_json::Value = response.json();
    let first = &body.as_array().unwrap()[0];
    assert!(first["id"].is_string());
    assert!(first["firstName"].is_string());
    assert!(first["middleName"].is_string());
    assert!(first["lastName"].is_string());
    assert!(first["dateOfBirth"].is_string());
    assert!(first["squadNumber"].is_number());
    assert!(first["position"].is_string());
    assert!(first["abbrPosition"].is_string());
    assert!(first["team"].is_string());
    assert!(first["league"].is_string());
    assert!(first["starting11"].is_boolean());
}

// GET /players/{id} -----------------------------------------------------------

// GET /players/{uuid} with existing UUID returns 200 OK with full player body
#[tokio::test]
async fn test_request_get_player_by_id_existing_response_status_ok() {
    // Arrange
    let server = setup_server();
    // Act
    let response = server
        .get(&format!("/players/{}", common::EXISTING_PLAYER_ID))
        .await;
    // Assert
    assert_eq!(response.status_code(), StatusCode::OK);
    let body: serde_json::Value = response.json();
    assert_eq!(body["id"], common::EXISTING_PLAYER_ID);
    assert_eq!(body["firstName"], "Lionel");
    assert_eq!(body["middleName"], "Andrés");
    assert_eq!(body["lastName"], "Messi");
    assert_eq!(body["dateOfBirth"], "1987-06-24T00:00:00.000Z");
    assert_eq!(body["squadNumber"], 10);
    assert_eq!(body["position"], "Right Winger");
    assert_eq!(body["abbrPosition"], "RW");
    assert_eq!(body["team"], "Paris Saint-Germain");
    assert_eq!(body["league"], "Ligue 1");
    assert_eq!(body["starting11"], true);
}

// GET /players/{uuid} with nonexistent UUID returns 404 Not Found
#[tokio::test]
async fn test_request_get_player_by_id_nonexistent_response_status_not_found() {
    // Arrange
    let server = setup_server();
    // Act
    let response = server
        .get("/players/00000000-0000-0000-0000-000000000000")
        .await;
    // Assert
    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

// GET /players/{uuid} with unknown UUID (valid format, absent from DB) returns 404 Not Found
#[tokio::test]
async fn test_request_get_player_by_id_unknown_response_status_not_found() {
    // Arrange
    let server = setup_server();
    // Act
    let response = server
        .get(&format!("/players/{}", common::UNKNOWN_PLAYER_ID))
        .await;
    // Assert
    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

// GET /players/squadnumber/{squad_number} -------------------------------------

// GET /players/squadnumber/{squad_number} with existing number returns 200 OK
#[tokio::test]
async fn test_request_get_player_by_squadnumber_existing_response_status_ok() {
    // Arrange
    let server = setup_server();
    // Act
    let response = server.get("/players/squadnumber/10").await;
    // Assert
    assert_eq!(response.status_code(), StatusCode::OK);
    let body: serde_json::Value = response.json();
    assert_eq!(body["squadNumber"], 10);
    assert_eq!(body["firstName"], "Lionel");
    assert_eq!(body["middleName"], "Andrés");
    assert_eq!(body["lastName"], "Messi");
    assert_eq!(body["dateOfBirth"], "1987-06-24T00:00:00.000Z");
    assert_eq!(body["position"], "Right Winger");
    assert_eq!(body["abbrPosition"], "RW");
    assert_eq!(body["team"], "Paris Saint-Germain");
    assert_eq!(body["league"], "Ligue 1");
    assert_eq!(body["starting11"], true);
}

// GET /players/squadnumber/{squad_number} with nonexistent number returns 404
#[tokio::test]
async fn test_request_get_player_by_squadnumber_nonexistent_response_status_not_found() {
    // Arrange
    let server = setup_server();
    // Act
    let response = server.get("/players/squadnumber/99").await;
    // Assert
    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

// POST /players ---------------------------------------------------------------

// POST /players with valid body returns 201 Created with full player response
#[tokio::test]
async fn test_request_post_player_body_valid_response_status_created() {
    // Arrange — 26-player seed (no squad 27), so Lo Celso can be created
    let server = setup_server_for_post();
    let body = player_request_for_creation_json();
    // Act
    let response = server.post("/players").json(&body).await;
    // Assert
    assert_eq!(response.status_code(), StatusCode::CREATED);
    let response_body: serde_json::Value = response.json();
    assert!(!response_body["id"].as_str().unwrap().is_empty());
    assert_eq!(response_body["id"].as_str().unwrap().len(), 36); // UUID v4
    assert_eq!(response_body["firstName"], "Giovani");
    assert_eq!(response_body["middleName"], "");
    assert_eq!(response_body["lastName"], "Lo Celso");
    assert_eq!(response_body["dateOfBirth"], "1996-07-09T00:00:00.000Z");
    assert_eq!(response_body["squadNumber"], 27);
    assert_eq!(response_body["position"], "Central Midfield");
    assert_eq!(response_body["abbrPosition"], "CM");
    assert_eq!(response_body["team"], "Real Betis Balompié");
    assert_eq!(response_body["league"], "La Liga");
    assert_eq!(response_body["starting11"], false);
}

// POST /players with duplicate squad number returns 409 Conflict
#[tokio::test]
async fn test_request_post_player_body_duplicate_response_status_conflict() {
    // Arrange — POST Lo Celso once, then attempt a second creation (squad 27 now exists)
    let server = setup_server();
    let body = player_request_for_creation_json();
    server.post("/players").json(&body).await;
    // Act
    let response = server
        .post("/players")
        .json(&player_request_for_creation_json())
        .await;
    // Assert
    assert_eq!(response.status_code(), StatusCode::CONFLICT);
}

// PUT /players/squadnumber/{squad_number} -------------------------------------

// PUT /players/squadnumber/{squad_number} returns 204 No Content on success
#[tokio::test]
async fn test_request_put_player_squadnumber_existing_response_status_no_content() {
    // Arrange
    let server = setup_server();
    let body = player_request_for_update_json();
    // Act
    let response = server.put("/players/squadnumber/23").json(&body).await;
    // Assert
    assert_eq!(response.status_code(), StatusCode::NO_CONTENT);
    let persisted = server.get("/players/squadnumber/23").await;
    let persisted_body: serde_json::Value = persisted.json();
    assert_eq!(persisted_body["firstName"], "Emiliano");
    assert_eq!(persisted_body["lastName"], "Martínez");
    assert_eq!(persisted_body["team"], "Aston Villa FC");
}

// PUT /players/squadnumber/{squad_number} with nonexistent number returns 404
#[tokio::test]
async fn test_request_put_player_squadnumber_nonexistent_response_status_not_found() {
    // Arrange
    let server = setup_server();
    let body = player_request_for_update_json();
    // Act
    let response = server.put("/players/squadnumber/999").json(&body).await;
    // Assert
    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

// PUT /players/squadnumber/{squad_number} with unknown number (valid format, absent from DB) returns 404
#[tokio::test]
async fn test_request_put_player_squadnumber_unknown_response_status_not_found() {
    // Arrange
    let server = setup_server();
    let body = player_request_for_update_json();
    // Act
    let response = server.put("/players/squadnumber/28").json(&body).await;
    // Assert
    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

// POST /players — validation failures ----------------------------------------

// POST /players with empty required string field returns 422 Unprocessable Entity
#[tokio::test]
async fn test_request_post_players_body_empty_first_name_response_status_unprocessable_entity() {
    // Arrange
    let server = setup_server_for_post();
    let mut body = player_request_for_creation_json();
    body["firstName"] = serde_json::json!("");
    // Act
    let response = server.post("/players").json(&body).await;
    // Assert
    assert_eq!(response.status_code(), StatusCode::UNPROCESSABLE_ENTITY);
}

// POST /players with squad_number = 0 (below minimum) returns 422 Unprocessable Entity
#[tokio::test]
async fn test_request_post_players_body_squad_number_zero_response_status_unprocessable_entity() {
    // Arrange
    let server = setup_server_for_post();
    let mut body = player_request_for_creation_json();
    body["squadNumber"] = serde_json::json!(0);
    // Act
    let response = server.post("/players").json(&body).await;
    // Assert
    assert_eq!(response.status_code(), StatusCode::UNPROCESSABLE_ENTITY);
}

// POST /players with squad_number = 100 (above maximum) returns 422 Unprocessable Entity
#[tokio::test]
async fn test_request_post_players_body_squad_number_above_maximum_response_status_unprocessable_entity()
 {
    // Arrange
    let server = setup_server_for_post();
    let mut body = player_request_for_creation_json();
    body["squadNumber"] = serde_json::json!(100);
    // Act
    let response = server.post("/players").json(&body).await;
    // Assert
    assert_eq!(response.status_code(), StatusCode::UNPROCESSABLE_ENTITY);
}

// PUT /players/squadnumber/{squad_number} — validation failures ---------------

// PUT /players/squadnumber/{squad_number} with empty required string field returns 422
#[tokio::test]
async fn test_request_put_player_squadnumber_body_empty_last_name_response_status_unprocessable_entity()
 {
    // Arrange
    let server = setup_server();
    let mut body = player_request_for_update_json();
    body["lastName"] = serde_json::json!("");
    // Act
    let response = server.put("/players/squadnumber/23").json(&body).await;
    // Assert
    assert_eq!(response.status_code(), StatusCode::UNPROCESSABLE_ENTITY);
}

// PUT /players/squadnumber/{squad_number} with squad_number = 0 returns 422
#[tokio::test]
async fn test_request_put_player_squadnumber_body_squad_number_zero_response_status_unprocessable_entity()
 {
    // Arrange
    let server = setup_server();
    let mut body = player_request_for_update_json();
    body["squadNumber"] = serde_json::json!(0);
    // Act
    let response = server.put("/players/squadnumber/23").json(&body).await;
    // Assert
    assert_eq!(response.status_code(), StatusCode::UNPROCESSABLE_ENTITY);
}

// PUT /players/squadnumber/{squad_number} with squad_number = 100 returns 422
#[tokio::test]
async fn test_request_put_player_squadnumber_body_squad_number_above_maximum_response_status_unprocessable_entity()
 {
    // Arrange
    let server = setup_server();
    let mut body = player_request_for_update_json();
    body["squadNumber"] = serde_json::json!(100);
    // Act
    let response = server.put("/players/squadnumber/23").json(&body).await;
    // Assert
    assert_eq!(response.status_code(), StatusCode::UNPROCESSABLE_ENTITY);
}

// DELETE /players/squadnumber/{squad_number} ----------------------------------

// DELETE /players/squadnumber/{squad_number} with existing number returns 204
#[tokio::test]
async fn test_request_delete_player_squadnumber_existing_response_status_no_content() {
    // Arrange — POST Lo Celso (squad 27) first, then delete by squad number
    let server = setup_server();
    server
        .post("/players")
        .json(&player_request_for_creation_json())
        .await;
    // Act
    let response = server.delete("/players/squadnumber/27").await;
    // Assert
    assert_eq!(response.status_code(), StatusCode::NO_CONTENT);
}

// DELETE /players/squadnumber/{squad_number} with nonexistent number returns 404
#[tokio::test]
async fn test_request_delete_player_squadnumber_nonexistent_response_status_not_found() {
    // Arrange
    let server = setup_server();
    // Act
    let response = server.delete("/players/squadnumber/999").await;
    // Assert
    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

// DELETE /players/squadnumber/{squad_number} with unknown number (valid format, absent from DB) returns 404
#[tokio::test]
async fn test_request_delete_player_squadnumber_unknown_response_status_not_found() {
    // Arrange
    let server = setup_server();
    // Act
    let response = server.delete("/players/squadnumber/28").await;
    // Assert
    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}
