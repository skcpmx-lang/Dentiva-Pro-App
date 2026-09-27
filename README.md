# Dentiva Pro

Offline-first Windows dental practice management for Bangladesh. Target architecture: Tauri 2, React/TypeScript, Rust and SQLite. Currency: BDT. English interface with Bengali clinical text support.

**Engineering in progress — NOT a completed product or commercial release. No installer or desktop UI is currently available.**

## Present in this checkout
- Pre-implementation architecture, complete requirement traceability and acceptance/release specifications.
- Native Rust domain/persistence foundation: guarded authentication and roles, patient demographics/clinical-note separation, atomic invoices/payments and protected audit records.
- Real relational SQLite migration with foreign keys, immutable posting seals, same-patient allocation and overpayment constraints.
- Unit/integration/regression checks on Linux and Windows GitHub Actions, plus a fail-closed release-evidence validator.

This is only part of the requested product. Clinical modules, the Tauri/React desktop, full setup, encryption, attachments, backup/restore, print/PDF/Bengali shaping, installer and other mandatory features remain release-blocking. See the checkpoint instead of interpreting source files as completed functionality.

## Engineering documents
- [Build status and exact resume point](docs/BUILD_STATUS.md)
- [Architecture](docs/ARCHITECTURE.md) · [all 147 requirements](docs/REQUIREMENTS_TRACEABILITY.md)
- [Development / test commands](docs/DEVELOPMENT.md)
- [QA report](docs/QA_REPORT.md) · [regression ledger](docs/REGRESSIONS.md)
- [Acceptance matrix](docs/ACCEPTANCE_MATRIX.md) · [release gates](docs/RELEASE_CHECKLIST.md)

Developer/Creator: **Shohan Khan** · helloiamshohan@gmail.com
