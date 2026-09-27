# Defect and regression ledger

## FIN-001 — Posted invoices accepted an appended zero-price item
- Discovery: source/persistence self-review before release, 2026-09-27.
- Root cause: aggregate-total trigger prevented additional monetary value, but a zero-price line could still change the historical document through a direct persistence write.
- Fix: immutable `invoice_postings` seals an invoice only after its complete line sum matches the header. All item inserts after seal are rejected, including zero-price lines. Allocations reference the posting as well as the invoice/patient composite key. Posting is created inside the invoice transaction before initial payment.
- Regression: `test_zero_value_line_cannot_be_appended_after_posting`, `test_posting_rejects_missing_or_incomplete_items`, existing immutable/over-allocation/rollback tests. Actual SQLite migration tested by Python; native invoice workflow rerun through CI.
- Release disposition: fixed in foundation; this is not certification of unimplemented full billing scope.
