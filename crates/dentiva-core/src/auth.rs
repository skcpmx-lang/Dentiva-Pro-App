use crate::{
    db,
    error::{Error, Result},
    now, validation, App,
};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Algorithm, Argon2, Params, Version,
};
use rand::RngCore;
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use zeroize::Zeroizing;

pub const PERMISSIONS: &[&str] = &[
    "patients.view",
    "patients.create",
    "patients.edit",
    "patients.archive",
    "clinical.view",
    "clinical.edit",
    "invoices.view",
    "invoices.create",
    "invoices.discount",
    "payments.view",
    "payments.record",
    "users.manage",
    "roles.manage",
    "audit.view",
];

pub(crate) fn hasher() -> Argon2<'static> {
    Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        Params::new(65_536, 3, 1, None).expect("reviewed Argon2 parameters"),
    )
}
pub(crate) fn hash(password: &str) -> Result<String> {
    hasher()
        .hash_password(password.as_bytes(), &SaltString::generate(&mut OsRng))
        .map(|h| h.to_string())
        .map_err(|_| Error::Storage)
}
pub(crate) fn verify(password: &str, phc: &str) -> bool {
    PasswordHash::new(phc)
        .ok()
        .is_some_and(|p| hasher().verify_password(password.as_bytes(), &p).is_ok())
}

pub(crate) struct Session {
    pub token: Zeroizing<String>,
    pub user_id: String,
    pub auth_version: i64,
    pub created: Instant,
    pub activity: Instant,
    pub idle: Duration,
}

#[derive(Serialize, Debug)]
pub struct LoginResult {
    pub token: String,
    pub full_name: String,
    pub permissions: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NewUser {
    pub username: String,
    pub full_name: String,
    pub password: String,
    pub role_id: String,
}

impl App {
    pub fn login(&mut self, username: &str, password: &str) -> Result<LoginResult> {
        // Rate limit the command globally too, so nonexistent usernames cannot bypass backoff.
        if self
            .login_not_before
            .is_some_and(|until| Instant::now() < until)
        {
            return Err(Error::InvalidCredentials);
        }
        let normalized = validation::username(username).unwrap_or_default();
        let row = self.conn.query_row("SELECT id,full_name,password_hash,active,failed_attempts,locked_until,auth_version FROM users WHERE username=?1",
            [&normalized], |r| Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?,r.get::<_,i64>(3)?,r.get::<_,i64>(4)?,r.get::<_,i64>(5)?,r.get::<_,i64>(6)?))).optional()?;
        let phc = row
            .as_ref()
            .map(|r| r.2.as_str())
            .unwrap_or(&self.dummy_hash);
        let verified = password.len() <= 512 && verify(password, phc);
        let timestamp = chrono::Utc::now().timestamp();
        let valid = row
            .as_ref()
            .is_some_and(|r| verified && r.3 == 1 && r.5 <= timestamp);
        if !valid {
            self.login_not_before = Some(Instant::now() + Duration::from_secs(1));
            let tx = self.conn.transaction()?;
            let actor = row.as_ref().map(|r| r.0.as_str());
            if let Some(r) = &row {
                let failures = (r.4 + 1).min(20);
                let delay = if failures >= 5 {
                    (1_i64 << (failures - 5).min(8)).min(300)
                } else {
                    0
                };
                tx.execute("UPDATE users SET failed_attempts=?1,locked_until=MAX(locked_until,?2) WHERE id=?3",params![failures,timestamp+delay,r.0])?;
            }
            db::audit(&tx, actor, "auth.failed", "user", actor, "Sign-in rejected")?;
            tx.commit()?;
            return Err(Error::InvalidCredentials);
        }
        let (id, name, _, _, _, _, version) = row.ok_or(Error::InvalidCredentials)?;
        let permissions = self.grants(&id)?;
        let idle: u64 =
            self.conn
                .query_row("SELECT auto_lock_minutes FROM clinic WHERE id=1", [], |r| {
                    r.get(0)
                })?;
        let tx = self.conn.transaction()?;
        tx.execute(
            "UPDATE users SET failed_attempts=0,locked_until=0 WHERE id=?1",
            [&id],
        )?;
        db::audit(&tx, Some(&id), "auth.login", "user", Some(&id), "Signed in")?;
        tx.commit()?;
        let mut bytes = [0_u8; 32];
        OsRng.fill_bytes(&mut bytes);
        let token: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
        self.session = Some(Session {
            token: Zeroizing::new(token.clone()),
            user_id: id,
            auth_version: version,
            created: Instant::now(),
            activity: Instant::now(),
            idle: Duration::from_secs(idle * 60),
        });
        self.login_not_before = None;
        Ok(LoginResult {
            token,
            full_name: name,
            permissions,
        })
    }

    pub fn lock(&mut self, token: &str) -> Result<()> {
        let id = self.authorize(token, None)?;
        self.session = None;
        db::audit(
            &self.conn,
            Some(&id),
            "auth.lock",
            "user",
            Some(&id),
            "Session locked",
        )
    }
    pub fn logout(&mut self, token: &str) -> Result<()> {
        let id = self.authorize(token, None)?;
        self.session = None;
        db::audit(
            &self.conn,
            Some(&id),
            "auth.logout",
            "user",
            Some(&id),
            "Signed out",
        )
    }
    pub(crate) fn grants(&self, id: &str) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare("SELECT rp.permission_code FROM role_permissions rp JOIN users u ON u.role_id=rp.role_id WHERE u.id=?1 AND u.active=1 ORDER BY rp.permission_code")?;
        let result = stmt
            .query_map([id], |r| r.get(0))?
            .collect::<std::result::Result<Vec<String>, _>>()?;
        Ok(result)
    }
    pub(crate) fn authorize(&mut self, token: &str, permission: Option<&str>) -> Result<String> {
        let session = self.session.as_ref().ok_or(Error::Unauthenticated)?;
        if session.token.as_str() != token {
            return Err(Error::Unauthenticated);
        }
        if session.activity.elapsed() >= session.idle
            || session.created.elapsed() >= Duration::from_secs(12 * 3600)
        {
            self.session = None;
            return Err(Error::Unauthenticated);
        }
        let id = session.user_id.clone();
        let valid: bool = self.conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM users WHERE id=?1 AND active=1 AND auth_version=?2)",
            params![id, session.auth_version],
            |r| r.get(0),
        )?;
        if !valid {
            self.session = None;
            return Err(Error::Unauthenticated);
        }
        if let Some(permission) = permission {
            if !self.grants(&id)?.iter().any(|p| p == permission) {
                return Err(Error::Forbidden);
            }
        }
        if let Some(session) = &mut self.session {
            session.activity = Instant::now();
        }
        Ok(id)
    }

    pub fn create_user(&mut self, token: &str, input: NewUser) -> Result<String> {
        let secret = Zeroizing::new(input.password);
        let actor = self.authorize(token, Some("users.manage"))?;
        let username = validation::username(&input.username)?;
        let name = validation::text(&input.full_name, 200, true)?;
        validation::password(&secret)?;
        // Only owner can delegate ownership; no actor can delegate grants they don't hold.
        let owner: bool = self.conn.query_row(
            "SELECT r.protected_owner FROM roles r JOIN users u ON u.role_id=r.id WHERE u.id=?1",
            [&actor],
            |r| r.get(0),
        )?;
        let target_owner: bool = self.conn.query_row(
            "SELECT protected_owner FROM roles WHERE id=?1",
            [&input.role_id],
            |r| r.get(0),
        )?;
        if target_owner && !owner {
            return Err(Error::Forbidden);
        }
        let mut stmt = self
            .conn
            .prepare("SELECT permission_code FROM role_permissions WHERE role_id=?1")?;
        let target = stmt
            .query_map([&input.role_id], |r| r.get::<_, String>(0))?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        drop(stmt);
        let grants = self.grants(&actor)?;
        if target.iter().any(|g| !grants.contains(g)) {
            return Err(Error::Forbidden);
        }
        let phc = hash(&secret)?;
        let id = uuid::Uuid::new_v4().to_string();
        let tx = self.conn.transaction()?;
        tx.execute("INSERT INTO users(id,username,full_name,password_hash,role_id,created_at) VALUES(?1,?2,?3,?4,?5,?6)",params![id,username,name,phc,input.role_id,now()])?;
        db::audit(
            &tx,
            Some(&actor),
            "user.created",
            "user",
            Some(&id),
            "Account created",
        )?;
        tx.commit()?;
        Ok(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hashes_salted_argon2id() {
        let a = hash("a private long test phrase").unwrap();
        let b = hash("a private long test phrase").unwrap();
        assert!(a.starts_with("$argon2id$v=19$m=65536,t=3,p=1$"));
        assert_ne!(a, b);
        assert!(verify("a private long test phrase", &a));
        assert!(!verify("incorrect", &a));
    }
}
