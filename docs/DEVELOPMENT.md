# Development and verification

This checkout currently contains a tested-in-CI native foundation, not a runnable desktop product. Do not store real patient information in engineering builds: Windows encryption, managed files and recovery are not implemented yet.

## Native toolchain
Rust is pinned in `rust-toolchain.toml`; dependencies in `Cargo.lock`.

```sh
cargo metadata --locked --format-version 1 --no-deps
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
python3 -m unittest discover -s tests/integration -v
python3 -m unittest discover -s tests/unit -v
```

The core is the default Cargo workspace member. `src-tauri/migrations` contains the actual migration but no Tauri application shell exists yet. Do not try to run a nonexistent frontend or mistake the Rust test executable for an installer.

## CI and constrained environments
`.github/workflows/native-checks.yml` runs the above on Ubuntu 24.04 and Windows 2025, uses pinned action commits, enforces formatting even after capturing failure evidence, and uploads artifacts. Custom GitHub check runs carry native test output and the rustfmt patch so a workspace with blocked artifact CDN access can still read results:

```sh
gh run list --branch arena/01a0e166-dentiva-pro-app
gh api repos/skcpmx-lang/Dentiva-Pro-App/commits/HEAD_SHA/check-runs
```

Use the actual source SHA and check IDs. Do not substitute the default branch. The initial lockfile/format bootstrap was performed in an explicitly scoped job on this session branch and then removed. Normal CI no longer writes source.

## Scope of current tests
- Native unit and real SQLite domain integration tests live in `crates/dentiva-core/src/workflow_tests.rs` and component test modules. Private setup permit fixtures compile only under `cfg(test)` and are not exposed by production code.
- Python integration tests apply the identical production SQL to temporary on-disk SQLite; they verify storage invariants and forced process termination. A Python backup API test validates SQLite snapshot semantics, NOT an implemented Dentiva backup feature.
- Python release-gate tests use synthetic evidence in automatically cleaned temporary directories; they do NOT manufacture a usable installer.
- No clinical/financial fixtures are bundled into an application database. No test bypass is exported through an application command.

## Release validation
`python3 scripts/release_gate.py docs/qa-report.json` must fail while mandatory scope/evidence is incomplete. It validates evidence identity/checksums, all 147 groups, required gate results, failures/skips, findings and Windows PE artifact structure. An MZ/PE header alone does not prove installation; separate signed-off install/clean-machine evidence is mandatory. Current report is an engineering checkpoint, not a release certificate.

See BUILD_STATUS.md for the exact next task and RELEASE_CHECKLIST.md for every remaining gate.
