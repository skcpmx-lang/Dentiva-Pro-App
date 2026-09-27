# Dentiva Pro — resumable engineering checkpoint

Updated: 2026-09-27 UTC. Branch: `arena/01a0e166-dentiva-pro-app`.
Target commercial release: 1.0.0. **Release readiness: BLOCKED — not a completed product.**

## Completed phases
- Inspected empty repository at 702d220 (README only), confirmed session branch and connected GitHub repository.
- Established Tauri 2 / React + TypeScript / native Rust / relational SQLite architecture and native authorization boundary.
- Wrote architecture, full entity/schema specification, permission matrix, screen/component design, print/PDF, backup/recovery, security, testing, acceptance matrix and release policy BEFORE production implementation.
- Cross-checked all 147 master sections in REQUIREMENTS_TRACEABILITY.md. All functional acceptance groups start NOT RUN.

## Current phase
Implement and verify foundational native persistence/authentication/business invariants before connecting renderer. Do not mistake engineering source/version 0.1.0 for commercial release.

## Remaining mandatory scope
All production implementation and full master acceptance. Track evidence per requirement; no feature removed by being unimplemented. Native activation/ten-step setup, auth/lock, patients/profile/timeline, clinical/chart/referrals/visits, appointments/queue, prescriptions and print/PDF/Bengali, invoices/payments, inventory/accounting, staff/RBAC/settings, search/notifications, files/logos, recovery/backup/scheduler, shell/a11y, audits/reports/export, encryption, Windows offline installer/icon/runtime, all testing/performance/QA and final release publication.

## Environment / limitations
Linux authoring environment initially has Node 22 and Python 3.11, no Rust toolchain or Windows UI/hardware. GitHub CLI repository access confirmed. Rust toolchain installation and portable checks are next. Clean Windows/physical printer/DPI evidence cannot be manufactured from Linux tests.

## Failing tests / known issues
No tests existed at inspection; none executed yet. No installer/release artifact exists. Fixed offline activation and privileged local OS threat limitations documented. Restore multiple-files ambiguity resolved as validate/catalog many, apply one complete snapshot without merging.

## Last successful build / test
None. No executable has been built or claimed. No release tag or publication attempted before gates.

## Resume
Read this file, git status/log, specs and actual test/build outputs. Continue from the exact current phase. Keep this branch, preserve working source, update after meaningful verified milestones. All pending work is release-blocking, not a deferred commercial roadmap.
