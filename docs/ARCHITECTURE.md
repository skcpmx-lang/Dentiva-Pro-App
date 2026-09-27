# Dentiva Pro — architecture and technical contract

Status: approved engineering baseline, 2026-09-27. Target release: 1.0.0. Implementation status is exclusively in BUILD_STATUS.md; this specification does not assert implemented functionality.

## Repository inspection
Initial commit 702d220 contained only README.md. No application, dependencies, schema, tests, assets, workflow, or installer existed. Work remains on arena/01a0e166-dentiva-pro-app. The authoring environment is Linux; a real Windows runtime and printer laboratory are separate verification environments.

## Scope and boundary
A single-clinic, single-Windows-workstation offline application, supporting unlimited local users/dentists and storage-bounded patients. No network server, telemetry, cloud, subscription, remote font, or runtime CDN. Multi-workstation concurrent database access on a network share is not supported: SQLite and the managed attachment directory must reside on a local filesystem. Backups may target removable/network storage. English interface, unrestricted Bengali Unicode text. BDT is the sole operational currency. This is record management, not diagnosis or prescribing advice. No seeded clinical or financial records.

## Structure
- `src/`: React/TypeScript presentation, feature controllers, typed IPC client, shared design system. No database driver, password verifier, filesystem authority, or authoritative financial logic in JS.
- `crates/dentiva-core/`: Rust domain/application/persistence library, independently testable without WebView. Typed commands, explicit capability checks, validation, transaction and audit boundaries.
- `src-tauri/`: Tauri 2 shell; narrowly scoped command adapters, installation paths, DPAPI, printer and dialog integration. No SQL execution command, arbitrary shell, arbitrary filesystem command, or network listener.
- `src-tauri/migrations/`: immutable numbered SQL files, checksum-verified migration ledger.
- `tests/`: unit/integration/UI fixtures, never bundled patient data.
- `scripts/`, `.github/workflows/`: deterministic checks, license evidence, release gate and Windows packaging.
- `docs/`: specifications, evidence, operations and checkpoint. Production version promotion follows acceptance, not the existence of source files.

## Trust and concurrency
The renderer is untrusted. Every protected command resolves an opaque native session, checks idle/absolute expiry, re-reads active user and current grants, and authorizes both action and returned fields. The UI never supplies trusted actor, role, totals, balance, or file path. A single native application service serializes mutations; blocking DB/filesystem/printing tasks use worker threads. Reads are bounded and cancellable. Transactions use `BEGIN IMMEDIATE` for sequences/allocations. Foreign keys, WAL, busy timeout and durable synchronous settings apply to every connection. A second application process is rejected. No external access to the DB connection.

## Command contract
Structured success or `{code, message, fieldErrors, retryable, correlationId}`; no raw SQL/stack trace/medical data in errors. IDs are generated native UUIDs, timestamps UTC RFC3339; clinic calendar is Asia/Dhaka. IPC money is integer poisha inside checked 64-bit arithmetic and restricted to JS-safe output range. Quantities are fixed-scale integers. Tax/discount rates are integer basis points with a single documented half-up rounding boundary per line. Commands include a native-validated idempotency key for financial mutations. Parameterized SQL only. Read endpoints return field-level DTOs, not rows serialized wholesale.

## Startup / setup / shutdown
Resolve per-user local app data; validate installation receipt; open database without overwriting corruption; verify migration checksums and supported version; perform bounded integrity checks; authenticate. Missing/corrupt databases never trigger a silent reset. Activation precedes the ten-step draft wizard. Drafts are installation-local and not a partially usable clinic. Finalization creates clinic, dentists/designations, owner/grants, settings and audit atomically. An interrupted setup resumes or safely discards the draft. After finish, explicit login is required. Shutdown refuses new mutations, resolves in-flight transaction, flushes durable files, drops connections; checkpoints WAL only when safe.

## Domain rules
Patient code default `DP-{YYYY}-{sequence:6}`, configurable safe prefixes/year/digit count, unique monotonically allocated counter, never count+1. Patient archive is ordinary removal; financial/clinical history restrict hard deletion. Duplicate similarity warnings do not merge records. Patient merge is deliberately NOT included: it is optional in the request and unsafe without a separately validated reconciliation protocol.

Visits and signed prescriptions retain revisions; finalized invoices retain item name/code/price snapshots. Posting an invoice and optional initial payment is one transaction. Payments allocate only to invoices belonging to the same patient, cannot exceed available due, and use a unique external-request key. No editing paid totals; authorized reversal/credit is a new ledger entry. Queue and appointment transitions are explicit state machines; reschedule retains the old event and relation. Inventory stock is a sum of immutable movements, with batch-aware non-negative issue checks. Reports derive from posted records, never dashboard caches. Clinical text is clinician-entered, never inferred from examples in the master request.

## Conflict decisions
- Multi-file restore means catalog/validate many complete snapshots, then choose ONE to apply. Never concatenate/merge backups.
- Clean-machine installer testing precedes *final* packaging: an unsigned nonrelease candidate tests exactly the same sources/config; final reproducibility and signatures are verified afterward.
- Save as PDF through Windows is mandatory. Direct PDF is also in scope via the common render engine, with embedded fonts; no claim of support until tested.
- A Windows administrator or debugger in the same security context cannot be made cryptographically subordinate to a local application's RBAC. Threat boundary, mitigations and residual risk are documented in SECURITY.md.
- No publicly available feature is represented by a dead control. Incomplete scope remains explicit in acceptance status and blocks release; hiding it during engineering does not remove the requirement.

## Performance budgets
On documented reference hardware (4 cores, 8 GB RAM, SSD), 100k patients: first bounded page <500ms warm / <1500ms cold; indexed search <300ms p95 warm; ordinary save <500ms p95; chart interaction <100ms; UI task <50ms. Startup login screen <3s warm. Large file/hash/archive and print work leaves renderer responsive. Measure, do not assume. Maximum list limit 100; keyset paging for long timelines. Stream attachment and backup bytes; do not decode full-resolution photos in tables.
