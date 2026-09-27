use super::*;
use finance::{InitialPayment,InvoiceInput,PaymentInput};
use patients::{NewPatient,PatientQuery};
use setup::{ActivationPermit,DentistInput,SetupInput};
use tempfile::TempDir;

fn setup_input()->SetupInput {
    SetupInput {clinic_name:"পরীক্ষা ক্লিনিক".into(),address:"Dhaka".into(),phone:"01712345678".into(),
        dentists:vec![DentistInput {name:"Test Clinician".into(),designations:vec!["BDS".into(),"Consultant".into()],registration:String::new(),phone:String::new(),email:String::new()}],
        owner_name:"Test Owner".into(),username:"owner".into(),password:"private fixture phrase".into(),password_confirmation:"private fixture phrase".into(),auto_lock_minutes:10}
}
fn clinic()->(TempDir,App,String) {
    let dir=TempDir::new().unwrap();
    let mut app=App::open(&dir.path().join("clinic.db")).unwrap();
    app.setup(&ActivationPermit::fixture(),setup_input()).unwrap();
    let token=app.login("owner","private fixture phrase").unwrap().token;
    (dir,app,token)
}
fn new_patient()->NewPatient {
    NewPatient {name:"রোগীর নাম".into(),date_of_birth:Some("1995-04-10".into()),gender:"Female".into(),blood_group:"B+".into(),address:"Dhaka".into(),phone:"01712345678".into(),emergency_phone:"01711111111".into(),emergency_name:"Emergency contact".into(),assigned_dentist_id:None}
}
fn invoice(patient:&str,key:&str)->InvoiceInput {
    InvoiceInput {patient_id:patient.into(),items:vec![money::InvoiceLine {description:"Clinician entered service".into(),quantity_milli:1000,unit_price_poisha:125000,discount_poisha:0,tax_basis_points:0}],initial_payment:None,request_key:key.into()}
}
fn payment(patient:&str,invoice:&str,amount:i64)->PaymentInput {
    PaymentInput {patient_id:patient.into(),invoice_id:invoice.into(),amount_poisha:amount,method:"bKash".into(),reference:"reference".into(),notes:String::new(),request_key:uuid::Uuid::new_v4().to_string()}
}
#[test]
fn setup_is_atomic_and_cannot_repeat() {
    let dir=TempDir::new().unwrap();
    let mut app=App::open(&dir.path().join("clinic.db")).unwrap();
    let mut bad=setup_input();bad.password_confirmation="different".into();
    assert!(app.setup(&ActivationPermit::fixture(),bad).is_err());
    assert!(!app.status().unwrap().configured);
    assert_eq!(app.conn.query_row("SELECT COUNT(*) FROM users",[],|r|r.get::<_,i64>(0)).unwrap(),0);
    app.setup(&ActivationPermit::fixture(),setup_input()).unwrap();
    assert_eq!(app.setup(&ActivationPermit::fixture(),setup_input()),Err(error::Error::AlreadyConfigured));
    assert_eq!(app.conn.query_row("SELECT COUNT(*) FROM dentist_designations",[],|r|r.get::<_,i64>(0)).unwrap(),2);
}
#[test]
fn patient_roundtrip_search_permissions_and_lock() {
    let (_dir,mut app,token)=clinic();
    let p=app.create_patient(&token,new_patient()).unwrap();
    assert!(p.code.starts_with("DP-"));
    assert_eq!(app.patient(&token,&p.id).unwrap().name,"রোগীর নাম");
    let page=app.patients(&token,PatientQuery {search:"01711111111".into(),from:None,until:None,limit:20,offset:0}).unwrap();
    assert_eq!(page.total,1);
    assert!(app.patient("forged",&p.id).is_err());
    app.add_medical_note(&token,&p.id,"Allergy","Clinician-entered Bengali: অ্যালার্জি").unwrap();
    app.create_user(&token,auth::NewUser {username:"reception".into(),full_name:"Reception".into(),password:"private fixture reception".into(),role_id:"receptionist".into()}).unwrap();
    app.logout(&token).unwrap();
    let reception=app.login("reception","private fixture reception").unwrap().token;
    assert!(app.patient(&reception,&p.id).is_ok());
    assert_eq!(app.medical_notes(&reception,&p.id).unwrap_err(),error::Error::Forbidden);
    assert_eq!(app.financial_summary(&reception,&p.id).unwrap_err(),error::Error::Forbidden);
    let projected=serde_json::to_string(&app.patient(&reception,&p.id).unwrap()).unwrap();
    assert!(!projected.contains("poisha") && !projected.contains("Allergy"));
    app.lock(&reception).unwrap();
    assert_eq!(app.patient(&reception,&p.id).unwrap_err(),error::Error::Unauthenticated);
}
#[test]
fn invoice_payment_idempotency_and_retained_history() {
    let (_dir,mut app,token)=clinic();
    let p=app.create_patient(&token,new_patient()).unwrap();
    let key=uuid::Uuid::new_v4().to_string();
    let mut input=invoice(&p.id,&key);
    input.initial_payment=Some(InitialPayment {amount_poisha:25000,method:"Cash".into(),reference:String::new()});
    let result=app.create_invoice(&token,input).unwrap();
    let mut same=invoice(&p.id,&key);
    same.initial_payment=Some(InitialPayment {amount_poisha:25000,method:"Cash".into(),reference:String::new()});
    assert_eq!(app.create_invoice(&token,same).unwrap().id,result.id);
    assert_eq!(app.create_invoice(&token,invoice(&p.id,&key)).unwrap_err(),error::Error::Conflict);
    let pay=payment(&p.id,&result.id,50000);
    let pay_key=pay.request_key.clone();
    let pay_id=app.record_payment(&token,pay).unwrap();
    let mut again=payment(&p.id,&result.id,50000);again.request_key=pay_key;
    assert_eq!(app.record_payment(&token,again).unwrap(),pay_id);
    assert!(app.record_payment(&token,payment(&p.id,&result.id,50001)).is_err());
    let balance=app.financial_summary(&token,&p.id).unwrap();
    assert_eq!(balance.total_paid_poisha,75000);
    assert_eq!(balance.outstanding_poisha,50000);
    let other=app.create_patient(&token,new_patient()).unwrap();
    assert!(app.record_payment(&token,payment(&other.id,&result.id,1)).is_err());
    app.archive_patient(&token,&p.id,&p.code).unwrap();
    assert_eq!(app.financial_summary(&token,&p.id).unwrap().outstanding_poisha,50000);
    app.integrity_check(&token).unwrap();
}
#[test]
fn failed_audit_rolls_back_invoice_items_and_initial_payment() {
    let (_dir,mut app,token)=clinic();
    let p=app.create_patient(&token,new_patient()).unwrap();
    app.conn.execute_batch("CREATE TRIGGER reject_invoice_audit BEFORE INSERT ON audit_logs WHEN NEW.action='invoice.created' BEGIN SELECT RAISE(ABORT,'injected failure'); END;").unwrap();
    let mut input=invoice(&p.id,&uuid::Uuid::new_v4().to_string());
    input.initial_payment=Some(InitialPayment {amount_poisha:100,method:"Cash".into(),reference:String::new()});
    assert!(app.create_invoice(&token,input).is_err());
    for table in ["invoices","invoice_items","payments","payment_allocations"] {
        assert_eq!(app.conn.query_row(&format!("SELECT COUNT(*) FROM {table}"),[],|r|r.get::<_,i64>(0)).unwrap(),0);
    }
    let audit_count:i64=app.conn.query_row("SELECT COUNT(*) FROM audit_logs WHERE action IN ('payment.created','invoice.created')",[],|r|r.get(0)).unwrap();
    assert_eq!(audit_count,0);
}
#[test]
fn immutable_finance_audit_and_foreign_keys() {
    let (_dir,mut app,token)=clinic();
    let p=app.create_patient(&token,new_patient()).unwrap();
    app.create_invoice(&token,invoice(&p.id,&uuid::Uuid::new_v4().to_string())).unwrap();
    for sql in ["DELETE FROM invoices","UPDATE invoices SET total_poisha=1","DELETE FROM invoice_items","DELETE FROM audit_logs","UPDATE audit_logs SET summary='changed'","DELETE FROM patients"] {
        assert!(app.conn.execute(sql,[]).is_err(),"{sql}");
    }
    assert_eq!(app.conn.query_row("PRAGMA foreign_keys",[],|r|r.get::<_,i64>(0)).unwrap(),1);
    app.integrity_check(&token).unwrap();
}
#[test]
fn changed_migration_and_future_schema_are_refused_without_reset() {
    let (dir,app,_)=clinic();drop(app);
    let path=dir.path().join("clinic.db");
    let conn=rusqlite::Connection::open(&path).unwrap();
    conn.execute("UPDATE schema_migrations SET checksum='modified'",[]).unwrap();
    assert!(matches!(App::open(&path),Err(error::Error::Schema)));
    conn.pragma_update(None,"user_version",999).unwrap();
    assert!(matches!(App::open(&path),Err(error::Error::Schema)));
    assert_eq!(conn.query_row("SELECT COUNT(*) FROM clinic",[],|r|r.get::<_,i64>(0)).unwrap(),1);
}
#[test]
fn role_revocation_and_idle_expiry_take_effect_natively() {
    let (_dir,mut app,token)=clinic();
    app.conn.execute("DELETE FROM role_permissions WHERE role_id='owner' AND permission_code='patients.create'",[]).unwrap();
    assert_eq!(app.create_patient(&token,new_patient()).unwrap_err(),error::Error::Forbidden);
    let session=app.session.as_mut().unwrap();
    session.activity=std::time::Instant::now()-std::time::Duration::from_secs(601);
    assert!(matches!(app.patients(&token,PatientQuery {search:String::new(),from:None,until:None,limit:10,offset:0}),Err(error::Error::Unauthenticated)));
}
#[test]
fn date_ranges_and_search_metacharacters_are_literal() {
    let (_dir,mut app,token)=clinic();
    app.create_patient(&token,new_patient()).unwrap();
    for search in ["%","_","' OR 1=1 --"] {
        let page=app.patients(&token,PatientQuery {search:search.into(),from:None,until:None,limit:20,offset:0}).unwrap();
        assert_eq!(page.total,0);
    }
    assert!(app.patients(&token,PatientQuery {search:String::new(),from:Some("2026-02-01T00:00:00Z".into()),until:Some("2026-01-01T00:00:00Z".into()),limit:20,offset:0}).is_err());
}
