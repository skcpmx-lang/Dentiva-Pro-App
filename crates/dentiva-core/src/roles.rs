use crate::{
    auth::PERMISSIONS,
    db,
    error::{invalid, Error, Result},
    validation, App,
};
use rusqlite::params;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Role {
    pub id: String,
    pub name: String,
    pub protected_owner: bool,
    pub permissions: Vec<String>,
}

impl App {
    pub fn roles(&mut self, token: &str) -> Result<Vec<Role>> {
        self.authorize(token, Some("roles.manage"))?;
        let mut statement = self
            .conn
            .prepare("SELECT id,name,protected_owner FROM roles ORDER BY name LIMIT 1000")?;
        let rows = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, bool>(2)?,
                ))
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let mut roles = Vec::new();
        for (id, name, protected_owner) in rows {
            let mut query = self.conn.prepare("SELECT permission_code FROM role_permissions WHERE role_id=?1 ORDER BY permission_code")?;
            let permissions = query
                .query_map([&id], |row| row.get(0))?
                .collect::<std::result::Result<Vec<String>, _>>()?;
            roles.push(Role {
                id,
                name,
                protected_owner,
                permissions,
            });
        }
        Ok(roles)
    }

    pub fn create_role(&mut self, token: &str, name: &str, grants: Vec<String>) -> Result<String> {
        let actor = self.authorize(token, Some("roles.manage"))?;
        let name = validation::text(name, 100, true)?;
        let grants = self.validate_delegation(&actor, grants)?;
        let id = uuid::Uuid::new_v4().to_string();
        let tx = self
            .conn
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        tx.execute(
            "INSERT INTO roles(id,name) VALUES(?1,?2)",
            params![id, name],
        )?;
        for grant in &grants {
            tx.execute(
                "INSERT INTO role_permissions VALUES(?1,?2)",
                params![id, grant],
            )?;
        }
        let summary = serde_json::json!({"before":[],"after":grants}).to_string();
        db::audit(
            &tx,
            Some(&actor),
            "role.created",
            "role",
            Some(&id),
            &summary,
        )?;
        tx.commit()?;
        Ok(id)
    }

    pub fn replace_role_permissions(
        &mut self,
        token: &str,
        role_id: &str,
        grants: Vec<String>,
    ) -> Result<()> {
        let actor = self.authorize(token, Some("roles.manage"))?;
        let grants = self.validate_delegation(&actor, grants)?;
        let actor_grants = self.grants(&actor)?;
        let tx = self
            .conn
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        let protected: bool = tx.query_row(
            "SELECT protected_owner FROM roles WHERE id=?1",
            [role_id],
            |row| row.get(0),
        )?;
        if protected {
            return Err(invalid(
                "Owner permissions cannot be removed. Configure another role instead.",
            ));
        }
        let mut query = tx.prepare("SELECT permission_code FROM role_permissions WHERE role_id=?1 ORDER BY permission_code")?;
        let before = query
            .query_map([role_id], |row| row.get(0))?
            .collect::<std::result::Result<Vec<String>, _>>()?;
        drop(query);
        // Delegators must be able to manage every grant being changed, including removals.
        if before.iter().any(|grant| !actor_grants.contains(grant)) {
            return Err(Error::Forbidden);
        }
        tx.execute("DELETE FROM role_permissions WHERE role_id=?1", [role_id])?;
        for grant in &grants {
            tx.execute(
                "INSERT INTO role_permissions VALUES(?1,?2)",
                params![role_id, grant],
            )?;
        }
        tx.execute(
            "UPDATE users SET auth_version=auth_version+1 WHERE role_id=?1",
            [role_id],
        )?;
        let summary = serde_json::json!({"before":before,"after":grants}).to_string();
        db::audit(
            &tx,
            Some(&actor),
            "role.permissions_changed",
            "role",
            Some(role_id),
            &summary,
        )?;
        tx.commit()?;
        Ok(())
    }

    fn validate_delegation(&self, actor: &str, mut grants: Vec<String>) -> Result<Vec<String>> {
        if grants.len() > PERMISSIONS.len() {
            return Err(invalid("The permission selection is invalid."));
        }
        grants.sort();
        grants.dedup();
        let actor_grants = self.grants(actor)?;
        for grant in &grants {
            if !PERMISSIONS.contains(&grant.as_str()) {
                return Err(invalid("An unrecognized permission was selected."));
            }
            if !actor_grants.contains(grant) {
                return Err(Error::Forbidden);
            }
        }
        Ok(grants)
    }
}
