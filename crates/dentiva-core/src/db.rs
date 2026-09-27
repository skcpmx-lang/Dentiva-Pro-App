use crate::{
    error::{Error, Result},
    now,
};
use rusqlite::{params, Connection, OpenFlags};
use sha2::{Digest, Sha256};
use std::{path::Path, time::Duration};

const MIGRATION: &str = include_str!("../../../src-tauri/migrations/0001_foundation.sql");
pub const SCHEMA_VERSION: i64 = 1;

pub(crate) fn open(path: &Path) -> Result<Connection> {
    let mut conn = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_WRITE
            | OpenFlags::SQLITE_OPEN_CREATE
            | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;
    conn.busy_timeout(Duration::from_secs(5))?;
    conn.execute_batch(
        "PRAGMA foreign_keys=ON; PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL;",
    )?;
    if conn.query_row("PRAGMA foreign_keys", [], |r| r.get::<_, i64>(0))? != 1 {
        return Err(Error::Storage);
    }
    let version: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
    let checksum = format!("{:x}", Sha256::digest(MIGRATION.as_bytes()));
    match version {
        0 => {
            let tables: i64 = conn.query_row("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'", [], |r| r.get(0))?;
            if tables != 0 {
                return Err(Error::Schema);
            }
            let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
            tx.execute_batch(MIGRATION)?;
            tx.execute(
                "INSERT INTO schema_migrations VALUES(?1,?2,?3)",
                params![SCHEMA_VERSION, checksum, now()],
            )?;
            tx.pragma_update(None, "user_version", SCHEMA_VERSION)?;
            tx.commit()?;
        }
        SCHEMA_VERSION => {
            let stored: String = conn
                .query_row(
                    "SELECT checksum FROM schema_migrations WHERE version=?1",
                    [SCHEMA_VERSION],
                    |r| r.get(0),
                )
                .map_err(|_| Error::Schema)?;
            if stored != checksum {
                return Err(Error::Schema);
            }
        }
        _ => return Err(Error::Schema),
    }
    health(&conn)?;
    Ok(conn)
}

pub(crate) fn health(conn: &Connection) -> Result<()> {
    let check: String = conn.query_row("PRAGMA quick_check", [], |r| r.get(0))?;
    if check != "ok" {
        return Err(Error::Storage);
    }
    let violations: i64 =
        conn.query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |r| {
            r.get(0)
        })?;
    if violations != 0 {
        return Err(Error::Storage);
    }
    Ok(())
}

pub(crate) fn audit(
    conn: &Connection,
    actor: Option<&str>,
    action: &str,
    entity: &str,
    entity_id: Option<&str>,
    summary: &str,
) -> Result<()> {
    conn.execute("INSERT INTO audit_logs(id,timestamp,actor_id,action,entity,entity_id,summary) VALUES(?1,?2,?3,?4,?5,?6,?7)",
        params![uuid::Uuid::new_v4().to_string(), now(),actor,action,entity,entity_id,summary])?;
    Ok(())
}

pub(crate) fn next_number(conn: &Connection, kind: &str, prefix: &str) -> Result<String> {
    use chrono::Datelike;
    let year = (chrono::Utc::now() + chrono::Duration::hours(6)).year();
    conn.execute(
        "INSERT INTO number_sequences(kind,year,next_value) VALUES(?1,?2,1) ON CONFLICT DO NOTHING",
        params![kind, year],
    )?;
    let value: i64 = conn.query_row(
        "SELECT next_value FROM number_sequences WHERE kind=?1 AND year=?2",
        params![kind, year],
        |r| r.get(0),
    )?;
    if value > 999_999_999 {
        return Err(Error::Storage);
    }
    conn.execute(
        "UPDATE number_sequences SET next_value=next_value+1 WHERE kind=?1 AND year=?2",
        params![kind, year],
    )?;
    Ok(format!("{prefix}-{year}-{value:06}"))
}
