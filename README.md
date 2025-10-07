Clickhouse Utils

This crate provides a set of utilities for working with Clickhouse.

# Features

- fastnum: Enable support for Fastnum D256.

# Usage

## Migration

create a migrations folder in your project and add your migrations files.
Example:
```text
migrations/
├── 0001_create_users_table.sql
└── 0002_add_email_column.sql
```

Run the migrations:
```rust,no_run
#[tokio::main]
async fn main() {
  let client = clickhouse::Client::default()
    .with_user("default")
    .with_password("default")
    .with_url("http://localhost:8123")
    .with_database("default");

  clickhouse_utils::migrate(&client).await.expect("failed to run migrations");
}
```

## Serde

Serialize and Deserialize common types for Clickhouse.

- Std Hashmap of <String, String>
- Fastnum : any decimals with a fix size scale mandated by the clickhouse schema

## Examples

```rust,no_run
use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use fastnum::decimal::D256;

#[derive(Debug, Serialize, Deserialize, clickhouse::Row)]
pub struct SomeTable {
  #[serde(with = "clickhouse_utils::serde::fastnum::Decimal::<25, {256 / 64}>")]
  pub decimal: D256,
  #[serde(with = "clickhouse_utils::serde::map")]
  pub attributes: HashMap<String, String>,
}
```

For the decimal fastnum::D256, we use:
- Decimal with { 256 / 64 } or Uint<4>
- 25 decimals, that should corelate with you clickhouse schema

The database schema used:
```sql
CREATE TABLE clickhouse_utils (
  decimal Decimal256(25),
  attributes Map(String, String),
) ENGINE = MergeTree()
ORDER BY (decimal)
```