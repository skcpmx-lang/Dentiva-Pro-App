# Release policy — fail closed

Version target 1.0.0; engineering prereleases use 0.1.0 with explicit nonrelease status. No final release/tag/installer publication until all requirements pass. Candidate packaging is permitted solely for install testing, not final release delivery. The tested source hash and packaging configuration must match final source exactly; only signatures may differ, verified separately.

## Required gates
BUILD, LINT, TYPECHECK, UNIT, INTEGRATION, E2E, DATABASE_INTEGRITY, SECURITY, RBAC, PRINT, PDF, BANGLA_UNICODE, BACKUP, RESTORE, ATTACHMENTS, INSTALLER, CLEAN_MACHINE, PERFORMANCE, HIGH_DPI, LICENSE_AUDIT, UI_UX_AUDIT, NO_FAKE_FEATURES, NO_TODO, SECRET_SCAN, DESTRUCTIVE_ACTIONS, RELEASE_ARTIFACT.

For each: PASS/FAIL/NOT_RUN, tool/environment, UTC timestamp, source commit, evidence artifact SHA-256, test passed/failed/skipped counts where relevant, reviewer. All mandatory rows in ACCEPTANCE_MATRIX.md must be linked to evidence. No critical findings, no unexplained skips. Existence of this checklist is not compliance.

## Candidate and clean machine
1. Complete scope, static checks, tests, security, RBAC, recoverability and print geometry audits.
2. Build nonpublished Windows candidate using pinned Tauri/Rust/npm dependencies and offline-bundled WebView2 installer (no runtime internet download). Validate x64 architecture, name/version/icon/resources, bundle license rights and SHA-256; scan malware where available.
3. Fresh supported Windows 10/11 x64 VM: disconnect all networking before install. Run full master clean-install checklist: installer/start/uninstall/shortcuts/icon, activation/setup/login, patient/visit/prescription/invoice/payment, files, backup+restore, print/PDF. Test reinstall preserves data, standard account Program Files permissions, missing WebView2 installation, disk failure, Windows shutdown. Physical printers and all DPI scales separately. Produce QA evidence.
4. Fix any failure and rerun candidate/relevant regression/full release checks. No declared known critical issue.
5. Final source review, secret scan history/tree, dependency/license notices, no test data/debug artifacts, ensure app/installer/Cargo/npm version consistent.
6. Final signed installer build after pre-release gate. Validate installer hash/signature/version and artifact itself. Tag v1.0.0 on this session branch's verified commit only when authorized and gates pass; no branch switch.
7. GitHub Actions publishes `Dentiva-Pro-Setup-v1.0.0.exe`, optional MSI, SHA256SUMS, QA report, release notes, notices/SBOM. Check actual release assets with gh and download/hash verify.
8. If publishing cannot work, preserve the REAL tested `.exe` in /dist using intentional artifact policy (force-add or release storage with explicit persistence verification); no text file renamed exe. Report fallback and failure accurately. Do not create a fake artifact to pass an existence check.

## Final self-review
Independently inspect permission entry points, same-patient relationships, historical prices, integer overflow, idempotency, recovery phase durability, cleanup safety, font shaping, actual paper geometry, notification leakage, file ACLs, cross-session behavior, unsupported routes/controls, migration behavior, release lockfiles and offline resources. No autodeletion on corruption. Verify installer launch/uninstall and executable Windows icon in each shell surface. Product not complete until artifact tested and available.

## Delivery manifest
Source + migrations + tests + build scripts + GitHub workflow + docs + third-party notices + QA JSON/report + release notes + build/commit metadata + final EXE + release/fallback URL/path and checksums. Never claim GitHub publication or a PASS based on intended behavior.
