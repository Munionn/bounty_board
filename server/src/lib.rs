//! Bounty Board HTTP API crate.
//!
//! Layering:
//! - `handlers` / `routes` — HTTP adapters (Axum)
//! - `services` — business logic + database (testable without HTTP)
//! - `auth`, `db`, `storage` — infrastructure
//! - `dto` / `models` — API contracts and database rows

pub mod app;
pub mod auth;
pub mod db;
pub mod dto;
pub mod error;
pub mod handlers;
pub mod models;
pub mod routes;
pub mod services;
pub mod storage;

pub use app::{build_router, init_state, run, AppConfig, AppState};
