# Engineering QA report — native foundation 0.1.0

**Release decision: BLOCKED. This is not a completed application or release certificate.**

Date: 2026-09-27 UTC. Schema version: 1.
Source tested: `bd19a3412251931f5fd1aa42ffa34a60576c3d8f`.
[Verified GitHub Actions run 36299486349](https://github.com/skcpmx-lang/Dentiva-Pro-App/actions/runs/36299486349).

## Executed engineering checks

| Check | Result | Scope |
|---|---|---|
| Master specification mapping | PASS | All 147 sections mapped before implementation |
| Pinned Rust / Cargo lock resolution | PASS | Rust 1.94.0, checked-in Cargo.lock, Linux and Windows |
| Rust formatting | PASS | `cargo fmt --all -- --check` on both platforms |
| Native compilation / strict lint | PASS | `cargo clippy --locked --all-targets -- -D warnings` on both platforms |
| Native unit/domain integration | 18 passed, 0 failed, 0 ignored **per platform** | Real Rust/SQLite core; money, activation verifier negatives, setup, auth, permission changes/revocation, patient search, finance, transaction rollback, migration refusal |
| Production SQLite migration tests | 15 passed **per platform** | Real on-disk SQLite, constraints, posting seal, WAL snapshot, forced child-process termination, index plan |
| Release validator unit tests | 6 passed **per platform** | Reject missing/stale/tampered evidence, missing scope, skipped tests and text renamed to EXE |
| Local Python rerun | 21 passed | Same 15 migration + 6 gate tests, not additional unique tests |
| SQL-only large-data benchmark | Executed; integrity checks PASS | 1k, 10k, 50k, 100k patients plus equal contact counts; temporary data removed |
| Scoped production-source scan | PASS within narrow scope | Common key markers, unnecessary fixed 16-digit literals and unfinished source markers; not a full secret/security audit |

**39 distinct implemented tests, executed on each CI platform (78 CI test instances).** These counts exclude unimplemented product acceptance groups, which remain NOT RUN—not silently skipped or passed.

At 100k records on this Linux host, measured warm p95 list/count: **13.314ms**; name search **287.708ms**; phone search **258.964ms**. This is database SQL only, not renderer/native/reference-machine/attachment performance certification. Query plans and hardware context are in the evidence file.

## Regression loop completed
- FIN-001: prevented zero-value additions to posted invoices; sealed header/item totals before allocation.
- PAT-001: preserved SQL escape character with a Rust raw string; real native search tests now pass.
- TEST-001: removed Windows monotonic-uptime assumption from expiry fixture; no production bypass.
- PERF-001: matched index direction to deterministic newest-list ordering; regression query-plan assertion.

See [regression ledger](REGRESSIONS.md) for causes, fixes and tests.

## Evidence
- [Machine-readable engineering report](qa-report.json)
- [Native CI job/check transcripts and tested-file hashes](evidence/native-foundation-ci.json)
- [Disposable SQL benchmark measurements and plans](evidence/schema-benchmark.json)

Evidence refers to the exact executable-source commit above. A documentation-only checkpoint commit may follow without changing the tested source files; the per-file hashes make that distinction verifiable.

## Mandatory gates not satisfied
No React/Tauri application, complete clinic workflows, encrypted production storage, activation receipt, managed attachments, backup/restore, PDF/print engine, Bengali shaping, desktop E2E, physical-printer QA, DPI QA, installer or clean-machine application test exists yet. Dependency/advisory/license/third-party-notice and full security audits are outstanding. All corresponding commercial release gates remain blocked.

The Python SQLite snapshot test is NOT a working Dentiva backup system. A Windows Rust test executable is NOT an application installer. Unicode storage tests do NOT prove Bengali font shaping or printed output.

**Production installer:** not built. **GitHub Release:** not published. **/dist installer fallback:** absent because no production installer exists. **Release tag:** none. No artifact has been renamed or fabricated to satisfy delivery requirements.

`python3 scripts/release_gate.py docs/qa-report.json` deliberately rejects this checkpoint as releasable. Next steps and all remaining contractual scope are in [BUILD_STATUS.md](BUILD_STATUS.md).
