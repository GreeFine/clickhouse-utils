//! Clickhouse Utils
//!
//! This crate provides a set of utilities for working with Clickhouse.
//!
//! # Features
//!
//! - fastnum: Enable support for Fastnum D256.
//!
//! # Usage
//!
//! ## Migration
//!
//! create a migrations folder in your project and add your migrations files.
//! Example:
//! ```text
//! migrations/
//! ├── 0001_create_users_table.sql
//! └── 0002_add_email_column.sql
//! ```
//!
//! Run the migrations:
//! ```rust,no_run
//! #[tokio::main]
//! async fn main() {
//!   let client = clickhouse::Client::default()
//!      .with_user("default")
//!      .with_password("default")
//!      .with_url("http://localhost:8123")
//!      .with_database("default");
//!
//!   clickhouse_utils::migrate(&client).await.expect("failed to run migrations");
//! }
//! ```
//!
//! ## Serde
//!
//! Serialize and Deserialize common types for Clickhouse.
//!
//! - Std Hashmap of <String, String>
//! - Fastnum : decimal D256
//!
//! ## Examples
//!
//! ```rust,no_run
//! use std::collections::HashMap;
//!
//! use serde::{Deserialize, Serialize};
//! use fastnum::decimal::D256;
//!
//! #[derive(Debug, Serialize, Deserialize, clickhouse::Row)]
//! pub struct SomeTable {
//!   #[serde(with = "clickhouse_utils::serde::fastnum_d256")]
//!   pub decimal: D256,
//!   #[serde(with = "clickhouse_utils::serde::map")]
//!   pub attributes: HashMap<String, String>,
//! }
//! ```

mod error;
mod migration;
pub mod serde;

pub use error::ClickhouseUtilsError;
pub use migration::migrate;
