use crate::{db, error::{invalid, Error, Result}, money::{calculate_line, invoice_total, InvoiceLine, MAX_MONEY}, now, validation, App};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const METHODS: &[&str]=&["Cash","Bank","Card","bKash","Nagad","Rocket","Upay","Others"];

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PaymentInput {
    pub patient_id:String,
    pub invoice_id:String,
    pub amount_poisha:i64,
    pub method:String,
    pub reference:String,
    pub notes:String,
    pub request_key:String,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InitialPayment {
    pub amount_poisha:i64,
    pub method:String,
    pub reference:String,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InvoiceInput {
    pub patient_id:String,
    pub items:Vec<InvoiceLine>,
    pub initial_payment:Option<InitialPayment>,
    pub request_key:String,
}
#[derive(Debug, Serialize)]
pub struct InvoiceReceipt {pub id:String,pub number:String,pub total_poisha:i64}
#[derive(Debug, Serialize)]
pub struct FinancialSummary {pub total_invoiced_poisha:i64,pub total_paid_poisha:i64,pub outstanding_poisha:i64}
#[derive(Debug, Serialize)]
pub struct InvoiceSummary {pub id:String,pub number:String,pub issued_at:String,pub total_poisha:i64,pub paid_poisha:i64,pub due_poisha:i64}

fn digest<T:Serialize>(value:&T)->Result<String> {
    let bytes=serde_json::to_vec(value).map_err(|_|Error::Storage)?;
    Ok(format!("{:x}",Sha256::digest(bytes)))
}
fn validate_payment(amount:i64,method:&str,reference:&str,notes:&str)->Result<()> {
    if !(1..=MAX_MONEY).contains(&amount) || !METHODS.contains(&method) {return Err(invalid("Enter a positive payment amount and supported payment method."));}
    validation::text(reference,200,false)?;
    validation::text(notes,2000,false)?;
    Ok(())
}
fn insert_payment(conn:&Connection,actor:&str,input:&PaymentInput)->Result<String> {
    let request_digest=digest(input)?;
    let existing=conn.query_row("SELECT id,request_digest,created_by FROM payments WHERE request_key=?1",[&input.request_key],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?))).optional()?;
    if let Some((id,stored,owner))=existing {
        return if stored==request_digest && owner==actor {Ok(id)} else {Err(Error::Conflict)};
    }
    let (patient,total,paid):(String,i64,i64)=conn.query_row("SELECT patient_id,total_poisha,COALESCE((SELECT SUM(amount_poisha) FROM payment_allocations WHERE invoice_id=i.id),0) FROM invoices i WHERE id=?1",[&input.invoice_id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?)))?;
    if patient!=input.patient_id {return Err(invalid("The invoice does not belong to this patient."));}
    if input.amount_poisha>total-paid {return Err(invalid("The payment exceeds the outstanding invoice balance."));}
    let id=uuid::Uuid::new_v4().to_string();
    conn.execute("INSERT INTO payments(id,patient_id,amount_poisha,method,received_at,reference,notes,request_key,request_digest,created_by) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
        params![id,input.patient_id,input.amount_poisha,input.method,now(),input.reference,input.notes,input.request_key,request_digest,actor])?;
    conn.execute("INSERT INTO payment_allocations VALUES(?1,?2,?3,?4)",params![id,input.invoice_id,input.patient_id,input.amount_poisha])?;
    db::audit(conn,Some(actor),"payment.created","payment",Some(&id),"Payment and invoice allocation posted")?;
    Ok(id)
}
impl App {
    pub fn create_invoice(&mut self,token:&str,input:InvoiceInput)->Result<InvoiceReceipt> {
        let actor=self.authorize(token,Some("invoices.create"))?;
        self.authorize(token,Some("patients.view"))?;
        if input.items.iter().any(|l|l.discount_poisha>0) {self.authorize(token,Some("invoices.discount"))?;}
        if input.initial_payment.is_some() {self.authorize(token,Some("payments.record"))?;}
        validation::request_key(&input.request_key)?;
        let total=invoice_total(&input.items)?;
        if let Some(payment)=&input.initial_payment {
            validate_payment(payment.amount_poisha,&payment.method,&payment.reference,"")?;
            if payment.amount_poisha>total {return Err(invalid("The initial payment exceeds the invoice total."));}
        }
        let request_digest=digest(&input)?;
        let tx=self.conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        let existing=tx.query_row("SELECT id,number,total_poisha,request_digest,created_by FROM invoices WHERE request_key=?1",[&input.request_key],|r|Ok((InvoiceReceipt {id:r.get(0)?,number:r.get(1)?,total_poisha:r.get(2)?},r.get::<_,String>(3)?,r.get::<_,String>(4)?))).optional()?;
        if let Some((receipt,stored,owner))=existing {
            return if stored==request_digest && owner==actor {Ok(receipt)} else {Err(Error::Conflict)};
        }
        let active:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM patients WHERE id=?1 AND status='Active')",[&input.patient_id],|r|r.get(0))?;
        if !active {return Err(Error::NotFound);}
        let id=uuid::Uuid::new_v4().to_string();
        let number=db::next_number(&tx,"invoice","INV")?;
        tx.execute("INSERT INTO invoices(id,number,patient_id,issued_at,total_poisha,request_key,request_digest,created_by) VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",params![id,number,input.patient_id,now(),total,input.request_key,request_digest,actor])?;
        for (ordinal,line) in input.items.iter().enumerate() {
            let amounts=calculate_line(line)?;
            tx.execute("INSERT INTO invoice_items VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)",params![uuid::Uuid::new_v4().to_string(),id,ordinal as i64,line.description,line.quantity_milli,line.unit_price_poisha,line.discount_poisha,line.tax_basis_points,amounts.total_poisha])?;
        }
        if let Some(payment)=input.initial_payment {
            insert_payment(&tx,&actor,&PaymentInput {patient_id:input.patient_id,invoice_id:id.clone(),amount_poisha:payment.amount_poisha,method:payment.method,reference:payment.reference,notes:String::new(),request_key:format!("initial:{}",input.request_key)})?;
        }
        db::audit(&tx,Some(&actor),"invoice.created","invoice",Some(&id),"Invoice and price snapshots posted")?;
        tx.commit()?;
        Ok(InvoiceReceipt {id,number,total_poisha:total})
    }
    pub fn record_payment(&mut self,token:&str,input:PaymentInput)->Result<String> {
        let actor=self.authorize(token,Some("payments.record"))?;
        self.authorize(token,Some("invoices.view"))?;
        validation::request_key(&input.request_key)?;
        validate_payment(input.amount_poisha,&input.method,&input.reference,&input.notes)?;
        let tx=self.conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        let id=insert_payment(&tx,&actor,&input)?;
        tx.commit()?;
        Ok(id)
    }
    pub fn financial_summary(&mut self,token:&str,patient_id:&str)->Result<FinancialSummary> {
        self.authorize(token,Some("patients.view"))?;
        self.authorize(token,Some("invoices.view"))?;
        self.authorize(token,Some("payments.view"))?;
        let total:i64=self.conn.query_row("SELECT COALESCE(SUM(total_poisha),0) FROM invoices WHERE patient_id=?1",[patient_id],|r|r.get(0))?;
        let paid:i64=self.conn.query_row("SELECT COALESCE(SUM(amount_poisha),0) FROM payment_allocations WHERE patient_id=?1",[patient_id],|r|r.get(0))?;
        if total>MAX_MONEY || paid>MAX_MONEY {return Err(invalid("This aggregate exceeds the supported display range. Narrow the report range."));}
        Ok(FinancialSummary {total_invoiced_poisha:total,total_paid_poisha:paid,outstanding_poisha:total-paid})
    }
    pub fn invoices(&mut self,token:&str,patient_id:&str)->Result<Vec<InvoiceSummary>> {
        self.authorize(token,Some("invoices.view"))?;
        self.authorize(token,Some("payments.view"))?;
        let mut stmt=self.conn.prepare("SELECT i.id,i.number,i.issued_at,i.total_poisha,COALESCE(SUM(a.amount_poisha),0) FROM invoices i LEFT JOIN payment_allocations a ON a.invoice_id=i.id WHERE i.patient_id=?1 GROUP BY i.id ORDER BY i.issued_at DESC,i.id DESC LIMIT 100")?;
        let result=stmt.query_map([patient_id],|r|{let total:i64=r.get(3)?;let paid:i64=r.get(4)?;Ok(InvoiceSummary {id:r.get(0)?,number:r.get(1)?,issued_at:r.get(2)?,total_poisha:total,paid_poisha:paid,due_poisha:total-paid})})?.collect::<std::result::Result<Vec<_>,_>>()?;
        Ok(result)
    }
}
