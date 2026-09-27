# Dentiva Pro — resumable engineering checkpoint

Updated: 2026-09-27 UTC. Branch: `arena/01a0e166-dentiva-pro-app`.
Target commercial release: 1.0.0. Current native crate: 0.1.0.
**Release readiness: BLOCKED — the requested complete application is NOT delivered.**

## Completed phases
1. Inspected initial repository 702d220 (README only); confirmed branch/GitHub access.
2. Wrote architecture, full entity/schema specification, role matrix, screen/component specification, printing/PDF, backup/recovery, security, test strategy, acceptance and release policy BEFORE implementation.
3. Cross-checked all 147 master sections in REQUIREMENTS_TRACEABILITY.md. Full product acceptance groups remain NOT RUN.
4. Implemented a real Rust/SQLite foundation (not yet a desktop application):
   - Checksum-validated migration v1; foreign keys, WAL/FULL durability, integrity checks, indexed relational tables, refusal of changed/newer schemas.
   - Isolated derived activation verifier. **Device receipt, one-time persistence and rate limiting coordinator remain absent.**
   - Atomic initial clinic/owner/dentists/designations setup primitive with validated Argon2id passwords. **Not the complete ten-step wizard/preferences.**
   - Native in-memory sessions, monotonic idle/absolute expiry, manual lock/logout, persisted user failures/backoff, active-user/grant checks below UI.
   - Demographic patient create/detail/bounded list/UTC filtering/literal search/contacts/archive; separately authorized append-only medical notes.
   - Decimal-safe invoice line calculations, immutable price snapshots, posting seal, atomic optional initial payment, same-patient allocation constraints, overpayment rejection, idempotent financial writes and audit.
   - Custom role delegation, explicit permission sets, owner protection, before/after grant audit and immediate auth-version session invalidation.
   - Fail-closed release-evidence validator that rejects incomplete scope, stale source, skipped tests, changed evidence and nonexistent/non-PE installer files.
5. Pushed source and CI only to the fixed session branch. GitHub-hosted Linux and Windows compile/test execution works; Cargo.lock and rustfmt output retrieved through a controlled same-branch bootstrap commit.

## Current phase
Native foundation checks passed on both platforms at 02f755c. Verify the final index-direction regression/benchmark change, then document the exact final source/results. Then continue production implementation per specs, starting with secure Windows installation/receipt/encryption coordinator and the complete setup/auth Tauri boundary, rather than inventing a browser-only backend.

## Verified checks so far
- Local Python: 15 real migration/SQLite integration tests PASS, including forced child-process termination before commit, immutable posted documents, cross-patient/overpayment/FK rejection, rollback and WAL snapshot.
- Local Python: 6 release-gate validator unit tests PASS.
- Native CI: Rust compilation and clippy `-D warnings` passed on Linux and Windows for prior revisions.
- Native 15-test suite initially 13 PASS / 2 FAIL; diagnosed and fixed Rust/SQLite search escaping. Linux rerun on 2c94b4b: 15 PASS. Expanded suite at 02f755c: Linux 18/18 PASS and Windows 18/18 PASS, with formatting and strict clippy PASS (run 36299362642). Windows expiry fixture corrected; no skipped native tests.
- SQL-only benchmark at 1k/10k/50k/100k patients AND contacts completed with full integrity/foreign-key checks. At 100k: newest page/count warm p95 13.314ms, name search 287.708ms, phone search 258.964ms. Actual Rust query strings extracted; no retained test database. Full native/UI/financial/attachment/reference-hardware performance remains untested.
- No frontend, Tauri shell, installer or actual printer testing has been executed or claimed.

## Defects / fixes
- FIN-001: zero-value invoice line could be appended after posting. Added immutable posting seal, aggregate-sum validation, post-seal insert rejection, allocation FK to posting and regression tests.
- PAT-001: Rust string escaping consumed SQLite ESCAPE backslash. Raw SQL string fix, plain/emergency-phone/literal metacharacter regressions. See REGRESSIONS.md.
- Windows-only expiry fixture subtracted 601s from a fresh monotonic clock; corrected to test the native expiry boundary without uptime assumptions. Formatter patch applied from CI; both-platform rerun PASS at 02f755c.

## Mandatory unimplemented / unverified scope
The following is remaining contractual work, NOT a deferred product roadmap:
- React/TypeScript application, Tauri 2 shell/IPC/capabilities/CSP, all premium UI screens, shared controls and accessibility. No browser demo has been substituted.
- Complete activation receipt/reset/device binding and ten-step setup; SQLCipher/DPAPI/attachment encryption/Windows ACLs; OS suspend/lock integration; policy/session persistence and recovery.
- Complete patient fields/edit/duplicate warnings/profile/unified timeline, visit/treatment/referral/chart, adult/primary dental history, appointments and queue.
- Prescription/medicine catalog, all print engine/templates/profiles/PDF, locally licensed Bengali font shaping and real Windows printer failure support.
- Full billing permissions/discount/tax settings/statuses/reversals, payment module/reports/date ranges, inventory/suppliers/batches/stock alerts, accounting, staff-sensitive fields.
- Full users/roles management UI, per-user grant overrides, field-level permissions across all remaining modules, settings, global search/notifications/dashboard/export/reports/audit viewer.
- Managed files/logo uploads/previews/journal; consistent encrypted attachment-inclusive backup, scheduler, multi-file restore catalog, verified pre-restore backup and crash-safe generation recovery.
- Full migrations for remaining entities; large-data performance, all crashes/destructive/error tests, UI/native E2E, physical printer/PDF/Bengali/DPI/clean-machine verification.
- Dependency/advisory/license inventories + notices, original app icon, offline WebView2 Windows candidate installer, release workflow/signing/version/tag/artifact checks, final GitHub release or REAL /dist fallback.

## Environment / evidence access
Linux authoring environment has Node/Python, but direct Rust/crates/apt download hosts return TLS failures. GitHub push/read and push-triggered Actions work. Workflow dispatch and repository Actions-permission admin API return integration 403; do not ask for tokens/passwords. Artifact/log CDN hosts also return EOF from this sandbox, so native test transcripts and formatter patches are published through GitHub Checks API. Pin toolchain 1.94.0; exact dependencies in Cargo.lock. Local toolchain is still absent; no native local compile claim. Real hosted Windows native compilation and tests do not equal installed-application clean-machine/physical-printer QA.

## Last successful build / test
Native compile/clippy: GitHub Actions on both Linux and Windows, see runs for 2c94b4b and later checkpoint updates. Tests: local Python 21/21; native 18/18 on each OS at 02f755c; final index-only follow-up rerun pending. A core library build is NOT an application/installer build. No release tag, EXE, GitHub release or /dist fallback exists.

## Resume precisely
1. Inspect git status/log/branch and latest Actions runs/check-run output for 525ca59 or later.
2. Confirm the final index-only follow-up CI run is green. If formatter/native checks fail, retrieve the Checks API evidence and fix/retest on this branch.
3. Update QA_REPORT.md and this file with actual green source hash and counts.
4. Continue remaining implementation above in spec order, with regressions and real native tests. Do not restart completed foundation, remove required scope, or publish a final artifact while gates are blocked.
