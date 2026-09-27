//! Offline domain/persistence foundation. The renderer never receives this connection.
//! Windows encryption, installation receipts and OS integrations live outside this crate.
pub mod activation;
pub mod auth;
mod db;
pub mod error;
pub mod finance;
pub mod money;
pub mod patients;
pub mod setup;
mod validation;

use error::Result;
use rusqlite::Connection;
use std::{path::Path, time::Instant};

pub struct App {
    conn: Connection,
    session: Option<auth::Session>,
    dummy_hash: String,
    login_not_before: Option<Instant>,
}
impl App {
    /// The installation coordinator owns path selection and directory/ACL validation.
    /// An error never deletes or replaces the original database.
    pub fn open(path: &Path) -> Result<Self> {
        Ok(Self {
            conn: db::open(path)?,
            session: None,
            dummy_hash: auth::hash(&uuid::Uuid::new_v4().to_string())?,
            login_not_before: None,
        })
    }
    pub fn integrity_check(&mut self, token: &str) -> Result<()> {
        self.authorize(token, Some("audit.view"))?;
        db::health(&self.conn)
    }
}
pub(crate) fn now() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

#[cfg(test)]
mod workflow_tests;
