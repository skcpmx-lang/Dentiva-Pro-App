use crate::{auth, db, error::{Error, Result}, now, validation, App};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SetupInput {
    pub clinic_name: String,
    pub address: String,
    pub phone: String,
    pub dentists: Vec<DentistInput>,
    pub owner_name: String,
    pub username: String,
    pub password: String,
    pub password_confirmation: String,
    pub auto_lock_minutes: i64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DentistInput {
    pub name: String,
    pub designations: Vec<String>,
    pub registration: String,
    pub phone: String,
    pub email: String,
}
#[derive(Debug, Serialize)]
pub struct AppStatus {
    pub configured: bool,
    pub clinic_name: Option<String>,
    pub schema_version: i64,
    pub app_version: &'static str,
}

impl App {
    pub fn status(&self) -> Result<AppStatus> {
        use rusqlite::OptionalExtension;
        let clinic_name=self.conn.query_row("SELECT name FROM clinic WHERE id=1",[],|r|r.get(0)).optional()?;
        Ok(AppStatus { configured:clinic_name.is_some(),clinic_name,schema_version:db::SCHEMA_VERSION,app_version:env!("CARGO_PKG_VERSION") })
    }

    /// Called only by the installation coordinator after successful activation verification.
    /// The opaque permit cannot be constructed from a renderer-supplied boolean.
    pub fn setup(&mut self, _permit: &ActivationPermit, input: SetupInput) -> Result<()> {
        if self.status()?.configured { return Err(Error::AlreadyConfigured); }
        let password=Zeroizing::new(input.password);
        let confirmation=Zeroizing::new(input.password_confirmation);
        validation::password(&password)?;
        if password.as_str()!=confirmation.as_str() { return Err(crate::error::invalid("The passwords do not match.")); }
        let clinic=validation::text(&input.clinic_name,200,true)?;
        let address=validation::text(&input.address,2000,true)?;
        let phone=validation::phone(&input.phone)?;
        if phone.is_empty() { return Err(crate::error::invalid("Enter the clinic phone number.")); }
        let username=validation::username(&input.username)?;
        let owner=validation::text(&input.owner_name,200,true)?;
        if ![5,10,15,30].contains(&input.auto_lock_minutes) { return Err(crate::error::invalid("Select a supported automatic lock interval.")); }
        if input.dentists.is_empty() || input.dentists.len()>1000 { return Err(crate::error::invalid("Add at least one dentist; submit up to 1,000 per operation.")); }
        let mut dentists=Vec::new();
        for dentist in input.dentists {
            let name=validation::text(&dentist.name,200,true)?;
            let registration=validation::text(&dentist.registration,200,false)?;
            let phone=validation::phone(&dentist.phone)?;
            let email=validation::text(&dentist.email,254,false)?;
            if dentist.designations.len()>30 { return Err(crate::error::invalid("Use at most 30 designations per dentist.")); }
            let designations=dentist.designations.iter().map(|d|validation::text(d,200,true)).collect::<Result<Vec<_>>>()?;
            dentists.push((name,registration,phone,email,designations));
        }
        let phc=auth::hash(&password)?;
        let tx=self.conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        let existing: bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM clinic)",[],|r|r.get(0))?;
        if existing { return Err(Error::AlreadyConfigured); }
        for (id,name,protected) in [("owner","Owner",1),("administrator","Administrator",0),("dentist","Dentist",0),
            ("receptionist","Receptionist",0),("accountant","Accountant",0),("assistant","Assistant",0),("inventory_manager","Inventory Manager",0)] {
            tx.execute("INSERT INTO roles(id,name,protected_owner) VALUES(?1,?2,?3)",params![id,name,protected])?;
        }
        for permission in auth::PERMISSIONS {
            tx.execute("INSERT INTO permissions(code) VALUES(?1)",[permission])?;
            for role in ["owner","administrator"] {
                tx.execute("INSERT INTO role_permissions VALUES(?1,?2)",params![role,permission])?;
            }
        }
        for (role,grants) in [
            ("dentist",&["patients.view","patients.create","patients.edit","clinical.view","clinical.edit"][..]),
            ("receptionist",&["patients.view","patients.create","patients.edit"][..]),
            ("accountant",&["patients.view","invoices.view","invoices.create","invoices.discount","payments.view","payments.record"][..]),
            ("assistant",&["patients.view"][..]),
        ] {
            for permission in grants { tx.execute("INSERT INTO role_permissions VALUES(?1,?2)",params![role,permission])?; }
        }
        let owner_id=uuid::Uuid::new_v4().to_string();
        tx.execute("INSERT INTO users(id,username,full_name,password_hash,role_id,created_at) VALUES(?1,?2,?3,?4,'owner',?5)",params![owner_id,username,owner,phc,now()])?;
        tx.execute("INSERT INTO clinic(id,name,address,phone,auto_lock_minutes,created_at) VALUES(1,?1,?2,?3,?4,?5)",params![clinic,address,phone,input.auto_lock_minutes,now()])?;
        for (name,registration,phone,email,designations) in dentists {
            let id=uuid::Uuid::new_v4().to_string();
            tx.execute("INSERT INTO dentists(id,name,registration,phone,email) VALUES(?1,?2,?3,?4,?5)",params![id,name,registration,phone,email])?;
            for (ordinal,designation) in designations.iter().enumerate() {
                tx.execute("INSERT INTO dentist_designations VALUES(?1,?2,?3)",params![id,ordinal as i64,designation])?;
            }
        }
        db::audit(&tx,Some(&owner_id),"setup.completed","clinic",Some("1"),"Clinic and owner initialized atomically")?;
        tx.commit()?;
        Ok(())
    }
}

pub struct ActivationPermit { _private: () }
impl ActivationPermit {
    pub fn verify(input: &str) -> Result<Self> {
        if crate::activation::verify_activation(input) { Ok(Self { _private: () }) }
        else { Err(Error::ActivationRequired) }
    }
    #[cfg(test)]
    pub(crate) fn fixture() -> Self { Self { _private: () } }
}
