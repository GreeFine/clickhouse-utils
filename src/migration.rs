use std::{fs, path::Path, time};

use serde::{Deserialize, Serialize};

struct MigrationFile {
    name: String,
    hash: String,
    content: String,
}

#[derive(Debug, Serialize, Deserialize, clickhouse::Row)]
struct MigrationEntry {
    name: String,
    hash: String,
    created_at: u64,
}

/// Run the migrations in the migrations folder.
///
/// - Migration are run in alphabetical order.
/// - If the migration has changed, will halt with an error.
/// - If the migration has been run before, it will be skipped.
///
///  We MD5 the content of the migration file to detect changes.
///  A table __migrations is created and used to track the migrations that have been run.
pub async fn migrate(client: &clickhouse::Client) -> crate::error::Result<()> {
    let migration_folder = Path::new("migrations");
    if !migration_folder.exists() {
        return Err(crate::error::ClickhouseUtilsError::new(
            "migrations folder does not exist".to_string(),
        ));
    }

    let mut migrations = Vec::new();
    for entry in fs::read_dir(migration_folder)? {
        let entry = entry?;
        let path = entry.path();
        let file_content = fs::read_to_string(&path)?;
        let Some(file_name) = path.file_name().map(|f| f.to_string_lossy().to_string()) else {
            continue;
        };

        if !file_name.ends_with(".sql") {
            tracing::warn!("skipping non-sql file: {}", file_name);
            continue;
        }
        let file_hash = md5::compute(file_content.as_bytes());
        migrations.push(MigrationFile {
            name: file_name,
            hash: format!("{:x}", file_hash),
            content: file_content,
        });
    }
    if migrations.is_empty() {
        tracing::warn!("no migrations to run");
        return Ok(());
    }

    migrations.sort_by_key(|m| m.name.clone());

    client
        .query(
            "CREATE TABLE IF NOT EXISTS __migrations (
                name String,
                hash String,
                created_at DateTime64(0),
                PRIMARY KEY (name)
            )",
        )
        .execute()
        .await?;
    let existing_migrations: Vec<MigrationEntry> = client
        .query("SELECT name, hash, created_at FROM __migrations")
        .fetch_all()
        .await?;

    for migration in migrations {
        if let Some(existing_migration) = existing_migrations
            .iter()
            .find(|m| m.name == migration.name)
        {
            if existing_migration.hash == migration.hash {
                continue;
            }
            return Err(crate::error::ClickhouseUtilsError::new(format!(
                "Migration {} has changed, originally created_at: {}",
                migration.name, existing_migration.created_at
            )));
        }

        let created_at = time::SystemTime::now()
            .duration_since(time::UNIX_EPOCH)?
            .as_secs();
        let mut migration_insert = client.insert("__migrations")?;
        migration_insert
            .write(&MigrationEntry {
                name: migration.name.clone(),
                hash: migration.hash,
                created_at,
            })
            .await?;

        // Run the migration
        client.query(&migration.content).execute().await?;
        tracing::info!("{} - migrated", migration.name);
        // Force the insert to be executed, because the migration was executed
        migration_insert.end().await?;
    }

    Ok(())
}
