use crate::{
    db,
    error::{invalid, Error, Result},
    now, validation, App,
};
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NewPatient {
    pub name: String,
    pub date_of_birth: Option<String>,
    pub gender: String,
    pub blood_group: String,
    pub address: String,
    pub phone: String,
    pub emergency_phone: String,
    pub emergency_name: String,
    pub assigned_dentist_id: Option<String>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Patient {
    pub id: String,
    pub code: String,
    pub name: String,
    pub date_of_birth: Option<String>,
    pub gender: String,
    pub blood_group: String,
    pub address: String,
    pub status: String,
    pub registered_at: String,
    pub phone: String,
    pub emergency_phone: String,
    pub emergency_name: String,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PatientQuery {
    pub search: String,
    pub from: Option<String>,
    pub until: Option<String>,
    pub limit: u32,
    pub offset: u32,
}
#[derive(Debug, Serialize)]
pub struct PatientPage {
    pub items: Vec<Patient>,
    pub total: i64,
}
#[derive(Debug, Serialize)]
pub struct MedicalNote {
    pub id: String,
    pub kind: String,
    pub body: String,
    pub created_at: String,
}

const PROJECTION: &str = "SELECT p.id,p.code,p.name,p.date_of_birth,p.gender,p.blood_group,p.address,p.status,p.registered_at,
COALESCE((SELECT phone FROM patient_contacts WHERE patient_id=p.id AND kind='Primary'),''),
COALESCE((SELECT phone FROM patient_contacts WHERE patient_id=p.id AND kind='Emergency'),''),
COALESCE((SELECT name FROM patient_contacts WHERE patient_id=p.id AND kind='Emergency'),'') FROM patients p";
fn map_patient(r: &rusqlite::Row<'_>) -> rusqlite::Result<Patient> {
    Ok(Patient {
        id: r.get(0)?,
        code: r.get(1)?,
        name: r.get(2)?,
        date_of_birth: r.get(3)?,
        gender: r.get(4)?,
        blood_group: r.get(5)?,
        address: r.get(6)?,
        status: r.get(7)?,
        registered_at: r.get(8)?,
        phone: r.get(9)?,
        emergency_phone: r.get(10)?,
        emergency_name: r.get(11)?,
    })
}
impl App {
    pub fn create_patient(&mut self, token: &str, input: NewPatient) -> Result<Patient> {
        let actor = self.authorize(token, Some("patients.create"))?;
        let name = validation::text(&input.name, 200, true)?;
        let address = validation::text(&input.address, 2000, false)?;
        let phone = validation::phone(&input.phone)?;
        let emergency_phone = validation::phone(&input.emergency_phone)?;
        let emergency_name = validation::text(&input.emergency_name, 200, false)?;
        validation::birth_date(&input.date_of_birth)?;
        if !["Female", "Male", "Other", "Not specified"].contains(&input.gender.as_str()) {
            return Err(invalid("Select a supported gender value."));
        }
        if !["", "A+", "A-", "B+", "B-", "AB+", "AB-", "O+", "O-"]
            .contains(&input.blood_group.as_str())
        {
            return Err(invalid("Select a valid blood group."));
        }
        let id = uuid::Uuid::new_v4().to_string();
        let tx = self
            .conn
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        if let Some(dentist) = &input.assigned_dentist_id {
            let valid: bool = tx.query_row(
                "SELECT EXISTS(SELECT 1 FROM dentists WHERE id=?1 AND active=1)",
                [dentist],
                |r| r.get(0),
            )?;
            if !valid {
                return Err(invalid("Select an active dentist."));
            }
        }
        let code = db::next_number(&tx, "patient", "DP")?;
        let timestamp = now();
        tx.execute("INSERT INTO patients(id,code,name,date_of_birth,gender,blood_group,address,assigned_dentist_id,registered_at,registered_by,updated_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?9)",
            params![id,code,name,input.date_of_birth,input.gender,input.blood_group,address,input.assigned_dentist_id,timestamp,actor])?;
        for (kind, number, contact) in [
            ("Primary", &phone, ""),
            ("Emergency", &emergency_phone, emergency_name.as_str()),
        ] {
            if !number.is_empty() || !contact.is_empty() {
                tx.execute(
                    "INSERT INTO patient_contacts VALUES(?1,?2,?3,?4,?5)",
                    params![uuid::Uuid::new_v4().to_string(), id, kind, contact, number],
                )?;
            }
        }
        db::audit(
            &tx,
            Some(&actor),
            "patient.created",
            "patient",
            Some(&id),
            "Demographic record created",
        )?;
        tx.commit()?;
        // The create result contains only values this operation was permitted to write.
        Ok(Patient {
            id,
            code,
            name,
            date_of_birth: input.date_of_birth,
            gender: input.gender,
            blood_group: input.blood_group,
            address,
            status: "Active".into(),
            registered_at: timestamp,
            phone,
            emergency_phone,
            emergency_name,
        })
    }
    pub fn patient(&mut self, token: &str, id: &str) -> Result<Patient> {
        self.authorize(token, Some("patients.view"))?;
        Ok(self
            .conn
            .query_row(&format!("{PROJECTION} WHERE p.id=?1"), [id], map_patient)?)
    }
    pub fn patients(&mut self, token: &str, input: PatientQuery) -> Result<PatientPage> {
        self.authorize(token, Some("patients.view"))?;
        if !(1..=100).contains(&input.limit) || input.offset > 1_000_000 {
            return Err(invalid("Request a page of 1–100 patients."));
        }
        let search = validation::text(&input.search, 200, false)?;
        for date in [&input.from, &input.until].into_iter().flatten() {
            let parsed = chrono::DateTime::parse_from_rfc3339(date)
                .map_err(|_| invalid("Choose a valid date range."))?;
            if parsed.offset().local_minus_utc() != 0 {
                return Err(invalid("Date boundaries must use UTC."));
            }
        }
        let from = input.from.as_ref().map(|d| {
            chrono::DateTime::parse_from_rfc3339(d)
                .expect("validated")
                .with_timezone(&chrono::Utc)
                .to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
        });
        let until = input.until.as_ref().map(|d| {
            chrono::DateTime::parse_from_rfc3339(d)
                .expect("validated")
                .with_timezone(&chrono::Utc)
                .to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
        });
        if matches!((&from,&until),(Some(a),Some(b)) if a>=b) {
            return Err(invalid("The end date must follow the start date."));
        }
        let escaped = search
            .replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_");
        let pattern = format!("%{escaped}%");
        let filter="WHERE p.status='Active' AND (?1='' OR p.name LIKE ?2 ESCAPE '\' OR p.code LIKE ?2 ESCAPE '\' OR p.address LIKE ?2 ESCAPE '\' OR EXISTS(SELECT 1 FROM patient_contacts c WHERE c.patient_id=p.id AND c.phone LIKE ?2 ESCAPE '\')) AND (?3 IS NULL OR p.registered_at>=?3) AND (?4 IS NULL OR p.registered_at<?4)";
        let total = self.conn.query_row(
            &format!("SELECT COUNT(*) FROM patients p {filter}"),
            params![search, pattern, from, until],
            |r| r.get(0),
        )?;
        let mut stmt = self.conn.prepare(&format!(
            "{PROJECTION} {filter} ORDER BY p.registered_at DESC,p.id DESC LIMIT ?5 OFFSET ?6"
        ))?;
        let items = stmt
            .query_map(
                params![search, pattern, from, until, input.limit, input.offset],
                map_patient,
            )?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(PatientPage { items, total })
    }
    pub fn archive_patient(
        &mut self,
        token: &str,
        id: &str,
        confirmation_code: &str,
    ) -> Result<()> {
        let actor = self.authorize(token, Some("patients.archive"))?;
        let tx = self.conn.transaction()?;
        let code: String = tx.query_row(
            "SELECT code FROM patients WHERE id=?1 AND status='Active'",
            [id],
            |r| r.get(0),
        )?;
        if code != confirmation_code {
            return Err(invalid("Type the exact patient code to confirm archiving."));
        }
        tx.execute(
            "UPDATE patients SET status='Archived',updated_at=?1 WHERE id=?2",
            params![now(), id],
        )?;
        db::audit(
            &tx,
            Some(&actor),
            "patient.archived",
            "patient",
            Some(id),
            "Archived; clinical and financial history retained",
        )?;
        tx.commit()?;
        Ok(())
    }
    pub fn add_medical_note(
        &mut self,
        token: &str,
        patient_id: &str,
        kind: &str,
        body: &str,
    ) -> Result<String> {
        self.authorize(token, Some("patients.view"))?;
        let actor = self.authorize(token, Some("clinical.edit"))?;
        if !["Complaint", "Medical", "Dental", "Allergy", "Additional"].contains(&kind) {
            return Err(invalid("Select a valid clinical note type."));
        }
        let body = validation::text(body, 20000, true)?;
        let id = uuid::Uuid::new_v4().to_string();
        let tx = self.conn.transaction()?;
        let active: Option<i64> = tx
            .query_row(
                "SELECT 1 FROM patients WHERE id=?1 AND status='Active'",
                [patient_id],
                |r| r.get(0),
            )
            .optional()?;
        if active.is_none() {
            return Err(Error::NotFound);
        }
        tx.execute(
            "INSERT INTO patient_medical_notes VALUES(?1,?2,?3,?4,?5,?6)",
            params![id, patient_id, kind, body, now(), actor],
        )?;
        db::audit(
            &tx,
            Some(&actor),
            "clinical.note_created",
            "patient",
            Some(patient_id),
            "Clinical note added",
        )?;
        tx.commit()?;
        Ok(id)
    }
    pub fn medical_notes(&mut self, token: &str, patient_id: &str) -> Result<Vec<MedicalNote>> {
        self.authorize(token, Some("patients.view"))?;
        self.authorize(token, Some("clinical.view"))?;
        let mut stmt=self.conn.prepare("SELECT id,kind,body,created_at FROM patient_medical_notes WHERE patient_id=?1 ORDER BY created_at DESC,id DESC LIMIT 100")?;
        let items = stmt
            .query_map([patient_id], |r| {
                Ok(MedicalNote {
                    id: r.get(0)?,
                    kind: r.get(1)?,
                    body: r.get(2)?,
                    created_at: r.get(3)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(items)
    }
}
