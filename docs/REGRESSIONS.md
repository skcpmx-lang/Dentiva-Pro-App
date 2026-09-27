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

## TEST-001 — Windows monotonic-clock underflow in expiry fixture
- Discovery: Windows Actions run 36299210737, 17/18 native tests passed; Linux 18/18 passed.
- Root cause: the fixture subtracted 601 seconds from `Instant::now()`. On a newly booted Windows runner the monotonic clock had less than that uptime, and the test panicked before calling authorization.
- Fix: set the private test session's idle duration to zero, then assert native authorization expires it. Production policy remains validated at 5/10/15/30 minutes; no production clock or authentication bypass was added.
- Regression: same native revocation/expiry test on both actual runner operating systems. Never classify the Windows failure as a skip or a product PASS before rerun.
