# 🧪 RESTful API with Rust and Axum

[![Rust CI](https://github.com/nanotaboada/rust-samples-rocket-restful/actions/workflows/rust-ci.yml/badge.svg)](https://github.com/nanotaboada/rust-samples-rocket-restful/actions/workflows/rust-ci.yml)
[![Rust CD](https://github.com/nanotaboada/rust-samples-rocket-restful/actions/workflows/rust-cd.yml/badge.svg)](https://github.com/nanotaboada/rust-samples-rocket-restful/actions/workflows/rust-cd.yml)
[![CodeQL Advanced](https://github.com/nanotaboada/rust-samples-rocket-restful/actions/workflows/codeql.yml/badge.svg)](https://github.com/nanotaboada/rust-samples-rocket-restful/actions/workflows/codeql.yml)
[![codecov](https://codecov.io/gh/nanotaboada/rust-samples-rocket-restful/branch/master/graph/badge.svg)](https://codecov.io/gh/nanotaboada/rust-samples-rocket-restful)
[![License: MIT](https://img.shields.io/badge/License-MIT-3DA639.svg)](https://opensource.org/licenses/MIT)
![Dependabot](https://img.shields.io/badge/Dependabot-contributing-025E8C?logo=dependabot&logoColor=white&labelColor=181818)
![Copilot](https://img.shields.io/badge/Copilot-contributing-8662C5?logo=githubcopilot&logoColor=white&labelColor=181818)
![Claude](https://img.shields.io/badge/Claude-contributing-D97757?logo=claude&logoColor=white&labelColor=181818)
![CodeRabbit](https://img.shields.io/badge/CodeRabbit-reviewing-FF570A?logo=coderabbit&logoColor=white&labelColor=181818)

Proof of Concept for a RESTful Web Service built with **Axum** and **Rust 2024 Edition**. This project demonstrates best practices for building a layered, testable, and maintainable API implementing CRUD operations for a Players resource (Argentina 2022 FIFA World Cup squad).

## Features

- 🏗️ **Layered Architecture** - Modular design with routes, services, repositories, state, and models as distinct packages
- 🔒 **Thread-Safe State** - SQLite access via r2d2 connection pool using Axum's `State<T>`
- ✅ **Type Safety** - Strong Rust type system with Serde for request/response serialization
- 🚦 **Comprehensive Testing** - Integration tests covering all endpoints with real SQLite
- 🐳 **Containerized Deployment** - Multi-stage Docker builds with migration-based database initialization
- 🔄 **Automated Pipeline** - Continuous integration with cargo test, clippy, and GitHub releases

## Tech Stack

| Category | Technology |
| -------- | ---------- |
| **Language** | [Rust 2024 Edition](https://www.rust-lang.org/) |
| **Web Framework** | [Axum 0.8](https://github.com/tokio-rs/axum) |
| **Serialization** | [Serde](https://serde.rs/) |
| **Unique IDs** | [uuid](https://github.com/uuid-rs/uuid) |
| **ORM / Migrations** | [Diesel](https://diesel.rs/) (SQLite + r2d2 features) + [diesel_migrations](https://docs.rs/diesel_migrations) |
| **Containerization** | [Docker](https://github.com/docker) & [Docker Compose](https://github.com/docker/compose) |

## Architecture

Layered architecture with Axum's router state for thread-safe dependency sharing.

```mermaid
%%{init: {
  "theme": "default",
  "themeVariables": {
    "fontFamily": "Fira Code, Consolas, monospace",
    "textColor": "#555",
    "lineColor": "#555",
    "lineWidth": 2,
    "clusterBkg": "#f5f5f5",
    "clusterBorder": "#999"
  }
}}%%

graph RL

    tests[tests]

    main[main]
    routes[routes]
    Axum[Axum]

    services[services]

    repositories[repositories]
    Diesel[Diesel]

    state[state]
    Serde[Serde]

    models[models]

    %% Dependencies

    routes --> main
    Axum --> main

    services --> routes
    state --> routes
    Axum --> routes

    repositories --> services

    state --> repositories
    Diesel --> repositories

    Serde --> models

    %% Soft dependencies

    models -.-> routes
    models -.-> services
    models -.-> repositories

    main -.-> tests

    %% Styling
    classDef core fill:#b3d9ff,stroke:#6db1ff,stroke-width:2px,color:#555,font-family:monospace;
    classDef deps fill:#ffcccc,stroke:#ff8f8f,stroke-width:2px,color:#555,font-family:monospace;
    classDef test fill:#ccffcc,stroke:#53c45e,stroke-width:2px,color:#555,font-family:monospace;

    class main,routes,services,repositories,state,models core
    class Axum,Serde,Diesel deps
    class tests test
```

> *Arrows follow the injection direction (A → B means A is injected into B). Solid = runtime dependency, dotted = structural. Blue = core domain, red = third-party, green = tests.*

## API Reference

| Method | Endpoint | Description | Status |
| ------ | -------- | ----------- | ------ |
| `GET` | `/players` | List all players | `200 OK` |
| `GET` | `/players/:id` | Get player by ID | `200 OK` |
| `GET` | `/players/squadnumber/:squadnumber` | Get player by squad number | `200 OK` |
| `POST` | `/players` | Create new player | `201 Created` |
| `PUT` | `/players/squadnumber/:squadnumber` | Update player by squad number | `204 No Content` |
| `DELETE` | `/players/squadnumber/:squadnumber` | Remove player by squad number | `204 No Content` |
| `GET` | `/health` | Health check | `200 OK` |

Error codes: `404 Not Found` (player not found) · `409 Conflict` (duplicate squad number on `POST`)

Alternatively, use [`rest/players.rest`](rest/players.rest) with the [REST Client](https://marketplace.visualstudio.com/items?itemName=humao.rest-client) extension for VS Code, or the built-in HTTP Client in JetBrains IDEs.

## Prerequisites

Before you begin, ensure you have the following installed:

- **Rust 2024 Edition or higher** (managed via `rust-toolchain.toml`)
- **Cargo** (comes with Rust)
- **Docker & Docker Compose** (optional, for containerized deployment)

## Quick Start

### Clone

```bash
git clone https://github.com/nanotaboada/rust-samples-rocket-restful.git
cd rust-samples-rocket-restful
```

### Build

```bash
cargo build
```

### Run

```bash
cargo run
```

### Access

Once the application is running, you can access:

- **API Server**: `http://localhost:9000`
- **Health Check**: `http://localhost:9000/health`
- **Swagger UI**: `http://localhost:9000/swagger-ui/`

## Containers

### Build and Start

```bash
docker compose up
```

> 💡 **Note:** On first run, the app automatically runs pending Diesel migrations — creating the schema and seeding the full 26-player squad — and stores the database in a persistent volume. On subsequent runs, migrations are skipped and existing data is preserved.

### Stop

```bash
docker compose down
```

### Reset Database

To remove the volume and let migrations reinitialize the database from scratch on next start:

```bash
docker compose down -v
```

### Pull Docker Images

Each release publishes multiple tags for flexibility:

```bash
# By semantic version (recommended for production)
docker pull ghcr.io/nanotaboada/rust-samples-rocket-restful:1.0.0

# By nominee name (memorable alternative)
docker pull ghcr.io/nanotaboada/rust-samples-rocket-restful:benzema

# Latest release
docker pull ghcr.io/nanotaboada/rust-samples-rocket-restful:latest
```

## Environment Variables

```bash
# Database storage path
# Container default (set in Dockerfile): /storage/players-sqlite3.db
# Local dev fallback (no env var set):   storage/players-sqlite3.db
STORAGE_PATH=/storage/players-sqlite3.db
```

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for details on:

- Code of Conduct
- Development workflow and best practices
- Commit message conventions (Conventional Commits)
- Pull request process and requirements

**Key guidelines:**

- Follow [Conventional Commits](https://www.conventionalcommits.org/) for commit messages
- Run `cargo fmt` and `cargo clippy` before committing
- Ensure all tests pass (`cargo test`)
- Keep changes small and focused
- Review `.github/copilot-instructions.md` for architectural patterns

**Testing:**

Run the test suite with Rust's built-in test framework:

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture
```

## Command Summary

| Command | Description |
| ------- | ----------- |
| `cargo run` | Start development server |
| `cargo build` | Build the application |
| `cargo build --release` | Build optimized release version |
| `cargo test` | Run all tests |
| `cargo fmt` | Format code |
| `cargo clippy` | Run linter |
| `cargo clean` | Clean build artifacts |
| `docker compose build` | Build Docker image |
| `docker compose up` | Start Docker container |
| `docker compose down` | Stop Docker container |
| `docker compose down -v` | Stop and remove Docker volume |
| **AI Commands** | |
| `/pre-commit` | Runs linting, tests, and quality checks before committing |
| `/pre-release` | Prepares a release: bumps version, updates CHANGELOG, and opens a PR |

## Legal

This project is provided for educational and demonstration purposes and may be used in production at your own discretion. All trademarks, service marks, product names, company names, and logos referenced herein are the property of their respective owners and are used solely for identification or illustrative purposes.
