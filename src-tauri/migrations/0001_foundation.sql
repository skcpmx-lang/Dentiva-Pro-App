CREATE TABLE schema_migrations (
    version INTEGER PRIMARY KEY,
    checksum TEXT NOT NULL,
    applied_at TEXT NOT NULL
) STRICT;

CREATE TABLE roles (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    protected_owner INTEGER NOT NULL DEFAULT 0 CHECK(protected_owner IN (0,1))
) STRICT;
CREATE TABLE permissions (code TEXT PRIMARY KEY) STRICT;
CREATE TABLE role_permissions (
    role_id TEXT NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    permission_code TEXT NOT NULL REFERENCES permissions(code) ON DELETE RESTRICT,
    PRIMARY KEY(role_id, permission_code)
) STRICT;
CREATE TABLE users (
    id TEXT PRIMARY KEY,
    username TEXT NOT NULL UNIQUE COLLATE NOCASE,
    full_name TEXT NOT NULL,
    password_hash TEXT NOT NULL,
    role_id TEXT NOT NULL REFERENCES roles(id) ON DELETE RESTRICT,
    active INTEGER NOT NULL DEFAULT 1 CHECK(active IN (0,1)),
    failed_attempts INTEGER NOT NULL DEFAULT 0 CHECK(failed_attempts >= 0),
    locked_until INTEGER NOT NULL DEFAULT 0,
    auth_version INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL
) STRICT;
CREATE INDEX idx_users_role ON users(role_id);
CREATE TABLE clinic (
    id INTEGER PRIMARY KEY CHECK(id=1),
    name TEXT NOT NULL CHECK(length(trim(name)) BETWEEN 1 AND 200),
    address TEXT NOT NULL,
    phone TEXT NOT NULL,
    currency TEXT NOT NULL DEFAULT 'BDT' CHECK(currency='BDT'),
    timezone TEXT NOT NULL DEFAULT 'Asia/Dhaka' CHECK(timezone='Asia/Dhaka'),
    auto_lock_minutes INTEGER NOT NULL DEFAULT 10 CHECK(auto_lock_minutes IN (5,10,15,30)),
    created_at TEXT NOT NULL
) STRICT;
CREATE TABLE dentists (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    registration TEXT NOT NULL DEFAULT '',
    phone TEXT NOT NULL DEFAULT '',
    email TEXT NOT NULL DEFAULT '',
    active INTEGER NOT NULL DEFAULT 1 CHECK(active IN (0,1))
) STRICT;
CREATE TABLE dentist_designations (
    dentist_id TEXT NOT NULL REFERENCES dentists(id) ON DELETE CASCADE,
    ordinal INTEGER NOT NULL CHECK(ordinal>=0),
    designation TEXT NOT NULL,
    PRIMARY KEY(dentist_id,ordinal)
) STRICT;
CREATE TABLE number_sequences (
    kind TEXT NOT NULL,
    year INTEGER NOT NULL,
    next_value INTEGER NOT NULL CHECK(next_value>0),
    PRIMARY KEY(kind,year)
) STRICT;
CREATE TABLE patients (
    id TEXT PRIMARY KEY,
    code TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL CHECK(length(trim(name)) BETWEEN 1 AND 200),
    date_of_birth TEXT,
    gender TEXT NOT NULL CHECK(gender IN ('Female','Male','Other','Not specified')),
    blood_group TEXT NOT NULL DEFAULT '' CHECK(blood_group IN ('','A+','A-','B+','B-','AB+','AB-','O+','O-')),
    address TEXT NOT NULL DEFAULT '',
    assigned_dentist_id TEXT REFERENCES dentists(id) ON DELETE RESTRICT,
    status TEXT NOT NULL DEFAULT 'Active' CHECK(status IN ('Active','Archived')),
    registered_at TEXT NOT NULL,
    registered_by TEXT NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    updated_at TEXT NOT NULL
) STRICT;
CREATE INDEX idx_patients_name ON patients(name COLLATE NOCASE,id);
CREATE INDEX idx_patients_registered ON patients(registered_at DESC,id);
CREATE INDEX idx_patients_dentist ON patients(assigned_dentist_id);
CREATE INDEX idx_patients_actor ON patients(registered_by);
CREATE TABLE patient_contacts (
    id TEXT PRIMARY KEY,
    patient_id TEXT NOT NULL REFERENCES patients(id) ON DELETE RESTRICT,
    kind TEXT NOT NULL CHECK(kind IN ('Primary','Emergency')),
    name TEXT NOT NULL DEFAULT '',
    phone TEXT NOT NULL,
    UNIQUE(patient_id,kind)
) STRICT;
CREATE INDEX idx_contacts_phone ON patient_contacts(phone);
CREATE TABLE patient_medical_notes (
    id TEXT PRIMARY KEY,
    patient_id TEXT NOT NULL REFERENCES patients(id) ON DELETE RESTRICT,
    kind TEXT NOT NULL CHECK(kind IN ('Complaint','Medical','Dental','Allergy','Additional')),
    body TEXT NOT NULL,
    created_at TEXT NOT NULL,
    created_by TEXT NOT NULL REFERENCES users(id) ON DELETE RESTRICT
) STRICT;
CREATE INDEX idx_notes_patient ON patient_medical_notes(patient_id,created_at);
CREATE INDEX idx_notes_actor ON patient_medical_notes(created_by);

CREATE TABLE invoices (
    id TEXT PRIMARY KEY,
    number TEXT NOT NULL UNIQUE,
    patient_id TEXT NOT NULL REFERENCES patients(id) ON DELETE RESTRICT,
    issued_at TEXT NOT NULL,
    total_poisha INTEGER NOT NULL CHECK(total_poisha BETWEEN 1 AND 9000000000000),
    state TEXT NOT NULL DEFAULT 'Posted' CHECK(state='Posted'),
    request_key TEXT NOT NULL UNIQUE,
    request_digest TEXT NOT NULL,
    created_by TEXT NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    UNIQUE(id,patient_id)
) STRICT;
CREATE INDEX idx_invoices_patient ON invoices(patient_id,issued_at DESC);
CREATE INDEX idx_invoices_date ON invoices(issued_at DESC);
CREATE INDEX idx_invoices_actor ON invoices(created_by);
CREATE TABLE invoice_items (
    id TEXT PRIMARY KEY,
    invoice_id TEXT NOT NULL REFERENCES invoices(id) ON DELETE RESTRICT,
    ordinal INTEGER NOT NULL CHECK(ordinal>=0),
    description TEXT NOT NULL CHECK(length(trim(description))>0),
    quantity_milli INTEGER NOT NULL CHECK(quantity_milli BETWEEN 1 AND 1000000000),
    unit_price_poisha INTEGER NOT NULL CHECK(unit_price_poisha BETWEEN 0 AND 9000000000000),
    discount_poisha INTEGER NOT NULL CHECK(discount_poisha BETWEEN 0 AND 9000000000000),
    tax_basis_points INTEGER NOT NULL CHECK(tax_basis_points BETWEEN 0 AND 10000),
    total_poisha INTEGER NOT NULL CHECK(total_poisha BETWEEN 0 AND 9000000000000),
    UNIQUE(invoice_id,ordinal)
) STRICT;
CREATE TABLE payments (
    id TEXT PRIMARY KEY,
    patient_id TEXT NOT NULL REFERENCES patients(id) ON DELETE RESTRICT,
    amount_poisha INTEGER NOT NULL CHECK(amount_poisha BETWEEN 1 AND 9000000000000),
    method TEXT NOT NULL CHECK(method IN ('Cash','Bank','Card','bKash','Nagad','Rocket','Upay','Others')),
    received_at TEXT NOT NULL,
    reference TEXT NOT NULL DEFAULT '',
    notes TEXT NOT NULL DEFAULT '',
    request_key TEXT NOT NULL UNIQUE,
    request_digest TEXT NOT NULL,
    created_by TEXT NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    UNIQUE(id,patient_id)
) STRICT;
CREATE INDEX idx_payments_patient ON payments(patient_id,received_at DESC);
CREATE INDEX idx_payments_date ON payments(received_at DESC,method);
CREATE INDEX idx_payments_actor ON payments(created_by);
CREATE TABLE payment_allocations (
    payment_id TEXT NOT NULL,
    invoice_id TEXT NOT NULL,
    patient_id TEXT NOT NULL REFERENCES patients(id) ON DELETE RESTRICT,
    amount_poisha INTEGER NOT NULL CHECK(amount_poisha>0),
    PRIMARY KEY(payment_id,invoice_id),
    FOREIGN KEY(payment_id,patient_id) REFERENCES payments(id,patient_id) ON DELETE RESTRICT,
    FOREIGN KEY(invoice_id,patient_id) REFERENCES invoices(id,patient_id) ON DELETE RESTRICT
) STRICT;
CREATE INDEX idx_allocations_invoice ON payment_allocations(invoice_id);
CREATE INDEX idx_allocations_patient ON payment_allocations(patient_id);
CREATE TRIGGER allocation_limits BEFORE INSERT ON payment_allocations BEGIN
    SELECT CASE WHEN NEW.amount_poisha + COALESCE((SELECT SUM(amount_poisha) FROM payment_allocations WHERE invoice_id=NEW.invoice_id),0) > (SELECT total_poisha FROM invoices WHERE id=NEW.invoice_id)
        THEN RAISE(ABORT,'invoice over-allocation') END;
    SELECT CASE WHEN NEW.amount_poisha + COALESCE((SELECT SUM(amount_poisha) FROM payment_allocations WHERE payment_id=NEW.payment_id),0) > (SELECT amount_poisha FROM payments WHERE id=NEW.payment_id)
        THEN RAISE(ABORT,'payment over-allocation') END;
END;
CREATE TABLE audit_logs (
    sequence INTEGER PRIMARY KEY,
    id TEXT NOT NULL UNIQUE,
    timestamp TEXT NOT NULL,
    actor_id TEXT REFERENCES users(id) ON DELETE RESTRICT,
    action TEXT NOT NULL,
    entity TEXT NOT NULL,
    entity_id TEXT,
    summary TEXT NOT NULL
) STRICT;
CREATE INDEX idx_audit_time ON audit_logs(timestamp DESC,sequence DESC);
CREATE INDEX idx_audit_actor ON audit_logs(actor_id,timestamp DESC);
CREATE INDEX idx_audit_entity ON audit_logs(entity,entity_id);
CREATE TRIGGER audit_no_update BEFORE UPDATE ON audit_logs BEGIN SELECT RAISE(ABORT,'audit immutable'); END;
CREATE TRIGGER audit_no_delete BEFORE DELETE ON audit_logs BEGIN SELECT RAISE(ABORT,'audit immutable'); END;
CREATE TRIGGER invoice_no_update BEFORE UPDATE ON invoices BEGIN SELECT RAISE(ABORT,'posted invoice immutable'); END;
CREATE TRIGGER invoice_no_delete BEFORE DELETE ON invoices BEGIN SELECT RAISE(ABORT,'posted invoice immutable'); END;
CREATE TRIGGER item_no_update BEFORE UPDATE ON invoice_items BEGIN SELECT RAISE(ABORT,'posted item immutable'); END;
CREATE TRIGGER item_no_delete BEFORE DELETE ON invoice_items BEGIN SELECT RAISE(ABORT,'posted item immutable'); END;
CREATE TRIGGER payment_no_update BEFORE UPDATE ON payments BEGIN SELECT RAISE(ABORT,'payment immutable'); END;
CREATE TRIGGER payment_no_delete BEFORE DELETE ON payments BEGIN SELECT RAISE(ABORT,'payment immutable'); END;
CREATE TRIGGER allocation_no_update BEFORE UPDATE ON payment_allocations BEGIN SELECT RAISE(ABORT,'allocation immutable'); END;
CREATE TRIGGER allocation_no_delete BEFORE DELETE ON payment_allocations BEGIN SELECT RAISE(ABORT,'allocation immutable'); END;
CREATE TRIGGER item_total_limit BEFORE INSERT ON invoice_items BEGIN
    SELECT CASE WHEN NEW.total_poisha + COALESCE((SELECT SUM(total_poisha) FROM invoice_items WHERE invoice_id=NEW.invoice_id),0) > (SELECT total_poisha FROM invoices WHERE id=NEW.invoice_id)
        THEN RAISE(ABORT,'invoice item total exceeded') END;
END;
