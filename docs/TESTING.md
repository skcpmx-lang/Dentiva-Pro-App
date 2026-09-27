# Test strategy and evidence policy

A test is PASS only after execution with recorded command, environment, result and artifact. NOT RUN is not PASS; SKIP blocks required release gates. Unit successes do not certify UI, hardware, installers, encryption, Bengali shaping or recovery.

## Pyramid
- Rust unit: checked integer money/rounding, totals/allocation, code generation, roles, dates/validation, stock, activation verifier and metadata parsing.
- Rust integration with isolated on-disk SQLite: migrations/foreign keys/checksums, setup atomicity, login/backoff/session revocation, patient/code race, appointment transition, visit revisions, prescriptions, immutable invoice prices, over/partial payment, same-patient constraint, invoice+initial payment rollback, idempotency, inventory concurrency, accounting reconciliation, file journal, backup/restore rollback and attachments.
- TypeScript unit: date-range/calendar and formatting, input parsing, error mapping, reducer/UI behavior. UI does not retest domain business logic as its authority.
- Browser component/E2E: real renderer accessibility, keyboard/forms/unsaved protection/error states; test doubles only under test directories and never mistaken for desktop persistence evidence.
- Native Windows E2E (Tauri WebDriver where supported): actual installed candidate, native IPC, real DB, all setup/login/clinical/financial/inventory/accounting/backup/lock/RBAC/preview workflows. No browser mock evidence substituted.

## Failure injection
Kill process before/after invoice header, details, allocation, audit and commit; validate either all or none. Kill backup at snapshot/hash/archive/publication, attachment stage/commit/promote and restore every durable phase; restart and verify old or new full generation, not partial. Read-only/no space/USB disconnect; corrupt DB/migration/hash/archive; missing logo/file; invalid printer; malicious CSV/backup paths; duplicate submission/number; concurrent payments and stock issue. Preserve failed artifacts outside release tree with no patient data.

## Print, visual and accessibility
See PRINTING.md. Snapshot each screen/state at 1280×720, 1366×768, 1440×900, 1600×900, 1920×1080, 2560×1440; native Windows 100/125/150/175/200% DPI. Audit focus order, keyboard, labels, contrast, text wrapping and scrolling, max viewport dialogs. Test six-card 3+3 layout and sidebar 264/80. Tooltips are not the sole accessible name. axe automation plus human review. Verify every visible button/tab/menu/shortcut/card action with a real persistence/read/OS effect; maintain a control inventory as screens are implemented.

## Performance
Generate separate temporary fixtures at 1k/10k/50k/100k+ patients and thousands of visits/invoices/payments, plus large managed attachments. Record hardware, cold/warm timings p50/p95/max, peak memory, query plans, DB size and UI thread long tasks. Seed fixtures never enter production DB/resources. Query budgets in ARCHITECTURE.md. Plan/index assertions complement timings, not substitute. Attachments/backups stream with progress; cancel only phases where safe.

## Regression protocol
Every defect: issue ID, reproducer/root cause, minimal fix, failing-before/passing-after test, affected suite and UI review if visual. Don't simply note a known defect and release. Tests assert stored records AND audit/relationships, not only success messages. On unsupported hardware mark NOT RUN and block release; do not fabricate a test result.

## Toolchain / continuous checks
Frontend lockfile npm ci, ESLint, tsc, Vitest, build; Rust locked cargo fmt/check/clippy/test; migration/integrity tests; secret/source audit; dependency SPDX/redistribution inventory; Windows Tauri check/package candidate. Release workflow requires a matching version/tag/source hash and machine-readable signed-off QA evidence for every gate. Third-party notices generated from both lockfiles + fonts/artwork, retaining required copyright files. Workflow execution success must be observed through gh checks/runs, not presumed from YAML.
