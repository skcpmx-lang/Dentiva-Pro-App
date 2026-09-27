# Defect and regression ledger

## FIN-001 — Posted invoices accepted an appended zero-price item
- Discovery: source/persistence self-review before release, 2026-09-27.
- Root cause: aggregate-total trigger prevented additional monetary value, but a zero-price line could still change the historical document through a direct persistence write.
- Fix: immutable `invoice_postings` seals an invoice only after its complete line sum matches the header. All item inserts after seal are rejected, including zero-price lines. Allocations reference the posting as well as the invoice/patient composite key. Posting is created inside the invoice transaction before initial payment.
- Regression: `test_zero_value_line_cannot_be_appended_after_posting`, `test_posting_rejects_missing_or_incomplete_items`, existing immutable/over-allocation/rollback tests. Actual SQLite migration tested by Python; native invoice workflow rerun through CI.
- Release disposition: fixed in foundation; this is not certification of unimplemented full billing scope.

## PAT-001 — Patient list returned a generic storage error
- Discovery: first native CI execution on both Linux and Windows, 13/15 tests passed, 2 patient-list tests failed.
- Root cause: ordinary Rust string escaping consumed the backslash in SQLite `ESCAPE '\'`, producing an empty escape expression. Demographic detail reads succeeded; list searches did not.
- Fix: raw Rust SQL string preserves SQLite's one-character escape. Search inputs still escape backslash, percent and underscore as literal characters and bind parameters.
- Regression: real native patient roundtrip/emergency-phone search and metacharacter search tests; added literal backslash alongside percent, underscore and SQL-injection-shaped text.
- Evidence: failure captured in GitHub Checks API `Native test evidence (Linux)` for run 36298965278. Both platforms must pass the rerun; not waived because Python-only schema tests passed.
