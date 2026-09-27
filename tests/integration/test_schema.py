"""Real SQLite tests of the production migration, no substitute app implementation."""
from pathlib import Path
import sqlite3
import tempfile
import unittest

SQL = (Path(__file__).resolve().parents[2] / 'src-tauri/migrations/0001_foundation.sql').read_text()


class SchemaTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.path = Path(self.directory.name) / 'clinic.db'
        self.db = sqlite3.connect(self.path)
        self.db.execute('PRAGMA foreign_keys=ON')
        self.db.execute('PRAGMA journal_mode=WAL')
        self.db.executescript(SQL)
        self.db.execute("INSERT INTO roles VALUES('owner','Owner',1)")
        self.db.execute("INSERT INTO users(id,username,full_name,password_hash,role_id,created_at) VALUES('u','owner','Test','test-fixture-not-a-login-hash','owner','2026-09-27T00:00:00.000Z')")
        self.patient('p', 'DP-1')
        self.patient('q', 'DP-2')
        self.db.commit()

    def tearDown(self):
        self.db.close()
        self.directory.cleanup()

    def patient(self, ident, code):
        self.db.execute("INSERT INTO patients(id,code,name,gender,registered_at,registered_by,updated_at) VALUES(?,?,'রোগীর নাম','Not specified','2026-09-27T00:00:00.000Z','u','2026-09-27T00:00:00.000Z')", (ident, code))

    def invoice(self, ident='i', patient='p', amount=10000):
        self.db.execute("INSERT INTO invoices(id,number,patient_id,issued_at,total_poisha,request_key,request_digest,created_by) VALUES(?,?,?,'2026-09-27T00:00:00.000Z',?,?,?,'u')", (ident, ident, patient, amount, ident, ident))

    def payment(self, ident='pay', patient='p', amount=10000):
        self.db.execute("INSERT INTO payments(id,patient_id,amount_poisha,method,received_at,request_key,request_digest,created_by) VALUES(?,?,?,'Cash','2026-09-27T00:00:00.000Z',?,?,'u')", (ident, patient, amount, ident, ident))

    def test_integrity_and_unicode(self):
        self.assertEqual(self.db.execute('PRAGMA integrity_check').fetchone(), ('ok',))
        self.assertEqual(self.db.execute('PRAGMA foreign_key_check').fetchall(), [])
        self.assertEqual(self.db.execute("SELECT name FROM patients WHERE id='p'").fetchone()[0], 'রোগীর নাম')

    def test_unique_codes_and_foreign_keys(self):
        with self.assertRaises(sqlite3.IntegrityError):
            self.patient('new', 'DP-1')
        with self.assertRaises(sqlite3.IntegrityError):
            self.invoice(patient='unknown')

    def test_same_patient_allocations_enforced(self):
        self.invoice()
        self.payment(patient='q')
        with self.assertRaises(sqlite3.IntegrityError):
            self.db.execute("INSERT INTO payment_allocations VALUES('pay','i','p',1)")
        with self.assertRaises(sqlite3.IntegrityError):
            self.db.execute("INSERT INTO payment_allocations VALUES('pay','i','q',1)")

    def test_invoice_cannot_be_overpaid_by_multiple_payments(self):
        self.invoice()
        self.payment(amount=6000)
        self.payment('pay2', amount=6000)
        self.db.execute("INSERT INTO payment_allocations VALUES('pay','i','p',6000)")
        with self.assertRaises(sqlite3.IntegrityError):
            self.db.execute("INSERT INTO payment_allocations VALUES('pay2','i','p',4001)")
        self.db.execute("INSERT INTO payment_allocations VALUES('pay2','i','p',4000)")

    def test_payment_cannot_be_overallocated_across_invoices(self):
        self.invoice()
        self.invoice('i2')
        self.payment(amount=6000)
        self.db.execute("INSERT INTO payment_allocations VALUES('pay','i','p',5000)")
        with self.assertRaises(sqlite3.IntegrityError):
            self.db.execute("INSERT INTO payment_allocations VALUES('pay','i2','p',1001)")

    def test_money_and_method_constraints(self):
        for amount in [-1, 0, 9000000000001, 1.25]:
            with self.assertRaises(sqlite3.IntegrityError):
                self.invoice(ident=str(amount), amount=amount)
        self.payment()
        with self.assertRaises(sqlite3.IntegrityError):
            self.db.execute("INSERT INTO payments(id,patient_id,amount_poisha,method,received_at,request_key,request_digest,created_by) VALUES('x','p',1,'USD','date','x','x','u')")

    def test_posted_records_and_audit_are_immutable(self):
        self.invoice()
        self.payment()
        self.db.execute("INSERT INTO payment_allocations VALUES('pay','i','p',100)")
        self.db.execute("INSERT INTO audit_logs(id,timestamp,actor_id,action,entity,summary) VALUES('audit','date','u','payment.created','payment','Posted')")
        for table, col in [('invoices', 'total_poisha'), ('payments', 'amount_poisha'), ('payment_allocations', 'amount_poisha')]:
            with self.assertRaises(sqlite3.IntegrityError):
                self.db.execute(f'UPDATE {table} SET {col}=1')
            with self.assertRaises(sqlite3.IntegrityError):
                self.db.execute(f'DELETE FROM {table}')
        with self.assertRaises(sqlite3.IntegrityError):
            self.db.execute("UPDATE audit_logs SET summary='altered'")
        with self.assertRaises(sqlite3.IntegrityError):
            self.db.execute('DELETE FROM audit_logs')
        with self.assertRaises(sqlite3.IntegrityError):
            self.db.execute("DELETE FROM patients WHERE id='p'")

    def test_extra_line_cannot_change_finalized_total(self):
        self.invoice()
        self.db.execute("INSERT INTO invoice_items VALUES('l','i',0,'Saved price',1000,10000,0,0,10000)")
        with self.assertRaises(sqlite3.IntegrityError):
            self.db.execute("INSERT INTO invoice_items VALUES('l2','i',1,'Extra',1000,1,0,0,1)")
        with self.assertRaises(sqlite3.IntegrityError):
            self.db.execute('DELETE FROM invoice_items')

    def test_transaction_failure_rolls_back_all_financial_rows(self):
        try:
            with self.db:
                self.invoice()
                self.payment()
                self.db.execute("INSERT INTO payment_allocations VALUES('pay','i','q',100)")
        except sqlite3.IntegrityError:
            pass
        self.assertEqual(self.db.execute('SELECT COUNT(*) FROM invoices').fetchone()[0], 0)
        self.assertEqual(self.db.execute('SELECT COUNT(*) FROM payments').fetchone()[0], 0)

    def test_process_close_discards_uncommitted_write(self):
        self.invoice()
        self.db.close()
        self.db = sqlite3.connect(self.path)
        self.db.execute('PRAGMA foreign_keys=ON')
        self.assertEqual(self.db.execute('SELECT COUNT(*) FROM invoices').fetchone()[0], 0)
        self.assertEqual(self.db.execute('PRAGMA integrity_check').fetchone()[0], 'ok')

    def test_online_snapshot_includes_committed_wal(self):
        self.invoice()
        self.db.commit()
        snapshot = sqlite3.connect(Path(self.directory.name) / 'snapshot.db')
        try:
            self.db.backup(snapshot)
            self.assertEqual(snapshot.execute('SELECT COUNT(*) FROM invoices').fetchone()[0], 1)
            self.assertEqual(snapshot.execute('PRAGMA integrity_check').fetchone()[0], 'ok')
        finally:
            snapshot.close()


if __name__ == '__main__':
    unittest.main(verbosity=2)
