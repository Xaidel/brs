# Phase 1 Spike Results Log

Owning phase: [Phase 1 — Technical Feasibility Spike & Offline Licensing / Key-Management Core](tdd.phase-1.tech-spike-and-licensing.md)

Status: Non-normative evidence record. This file has no authority over product or
technical scope — it records raw outcomes of the mandatory spike defined in
[Appendix A §4](../appendix-a-technical-architecture-direction.md#4-mandatory-technical-spike-phase-1)
and required as evidence by §6 and §14 of the owning TDD. Fill in each section as the
spike is actually executed; do not pre-fill results.

---

## S1 — Static-linked SQLCipher + FTS5 compiles cleanly

- Date run: 2026-08-06
- Toolchain (Rust version, target triple): `rustc 1.96.0 (ac68faa20 2026-05-25)`,
  `x86_64-pc-windows-msvc` — verified locally. `.github/workflows/ci.yml`'s `backend`
  job runs the same commands on a `windows-latest`/`macos-latest`/`ubuntu-latest`
  matrix; all three legs observed passing in CI run
  [31079970716](https://github.com/Xaidel/BMSIm/actions/runs/31079970716) (commit
  `5977fc9`) — `fmt`, `clippy`, `build`, `test`, and the release-binary size-proxy
  step all succeeded on every OS.
- `rusqlite` feature flags used: `bundled-sqlcipher-vendored-openssl` (`rusqlite`
  0.40.1, resolving `libsqlite3-sys` 0.38.1 + vendored `openssl-src`/`openssl-sys`) —
  already declared in `src-tauri/Cargo.toml`, no change needed for this spike.
- Build command: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
  `cargo build --all-targets`, `cargo test`, all run from `src-tauri/` (a Cargo
  workspace member; build output lands in the workspace-root `target/`).
- Outcome (pass/fail): **pass** on Windows (local) and on Windows, macOS, and Linux in
  CI (run 31079970716). All four commands succeed with zero warnings/errors on all
  three OSes. `src-tauri/src/sqlcipher.rs`'s three tests
  (`round_trip_open_write_read_close`, `fts5_virtual_table_create_and_query`,
  `wrong_key_is_rejected`) all pass on every matrix OS, proving the DB opens/writes/
  reads/closes correctly and FTS5 is functional (see FTS5 note below).
- Notes / linker errors if failed: FTS5 requires no separate `rusqlite` feature flag —
  confirmed by inspecting `libsqlite3-sys-0.38.1`'s `build.rs`, which passes
  `-DSQLITE_ENABLE_FTS5` unconditionally inside the same `bundled` build path used by
  `bundled-sqlcipher-vendored-openssl`; confirmed functionally by
  `fts5_virtual_table_create_and_query`, which creates a `USING fts5(body)` virtual
  table, seeds two rows, and asserts a `MATCH` query returns the expected row.
  Local-environment wrinkle (not a SQLCipher/OpenSSL/rusqlite issue): the first local
  build attempt failed because Git Bash's bundled MSYS Perl lacks the
  `Locale::Maketext::Simple` CPAN module that `openssl-src`'s `Configure` step needs;
  resolved by prioritizing the machine's separately installed Strawberry Perl
  (`C:\Strawberry\perl\bin`) on `PATH`. Confirmed via CI run 31079970716: this did not
  reproduce on `windows-latest` (ships a complete Strawberry Perl by default), nor on
  `macos-latest`/`ubuntu-latest`.

## S2 — FTS5 performance holds with both features enabled

- Date run: not run — intentionally deferred.
- Deferred to RES-B2 per FOUND-B3 (issue #4) scope: this task proves FTS5 is compiled
  in and functional via a create+query test (`fts5_virtual_table_create_and_query`),
  not FTS5 query performance under a seeded 50,000-record dataset, which does not
  exist until RES-B2's resident search work. Leave S2 unexecuted here; do not read the
  unchecked box below as a failure.
- Seeded dataset size (target: up to 50,000 Resident-equivalent records per NFR-02): n/a — deferred.
- Benchmark methodology: n/a — deferred.
- Measured search latency: n/a — deferred.
- Target: sub-200ms per NFR-02 (referenced by Appendix A, not restated as a Phase 1
  functional requirement — Resident search itself is Phase 2 scope): n/a — deferred.
- Outcome (pass/fail): deferred (not executed).

## S3 — Installer size stays under the NFR-03 budget

- Date run: 2026-08-06
- Artifact measured (full installer vs. representative proxy — state which):
  representative proxy — raw `cargo build --release` binary
  (`target/release/bms.exe`), not a packaged Tauri installer. Final installer-size
  sign-off against NFR-03 is REL-1's scope, not this spike's.
- Measurement method: `cargo build --release` from `src-tauri/`, then inspect the
  resulting binary at the workspace-root `target/release/bms.exe` (local Windows run).
  `.github/workflows/ci.yml`'s `backend` job runs the equivalent build and reports
  size via `$GITHUB_STEP_SUMMARY` on all three matrix OSes.
- Measured size: 14,832,640 bytes (~14.14 MB) on `x86_64-pc-windows-msvc` (local run).
  CI run 31079970716's "Report binary size" step completed successfully (binary built
  and `stat`'d without error) on `windows-latest`, `macos-latest`, and `ubuntu-latest`;
  the exact byte counts for the macOS/Linux legs are recorded in that run's per-job
  step summary (`$GITHUB_STEP_SUMMARY`) on GitHub, not reproduced verbatim here — the
  step only reports a number, it does not fail the build if a size threshold is
  crossed, so passing CI confirms the binary built and was measured on all three OSes
  but does not by itself confirm the macOS/Linux numbers are under budget. Given the
  Windows proxy has ~16MB of headroom and macOS/Linux Rust release binaries for this
  dependency set are not expected to diverge by that much, this is not flagged as a
  risk, but treat the macOS/Linux figures as unconfirmed against the 30MB budget until
  someone reads them off the CI run and records them here.
- Budget: under 30MB (NFR-03): yes on the Windows proxy measurement (~16MB headroom);
  macOS/Linux not numerically confirmed here (see note above).
- Outcome (pass/fail): **pass** (Windows proxy measured; macOS/Linux proxies built and
  measured successfully in CI but exact figures not transcribed into this doc). This
  is a build-artifact sanity check only, not the final packaged-installer validation
  required by REL-1.

---

## Overall Verdict

- [x] S1 pass (Windows verified locally; Windows/macOS/Linux all green in CI run
      31079970716)
- [ ] S2 pass — intentionally deferred to RES-B2, not failed (see S2 notes above)
- [x] S3 pass (Windows proxy measured locally with ~16MB headroom; macOS/Linux proxies
      built and measured successfully in CI run 31079970716, exact byte counts not yet
      transcribed into this doc — see S3 note above)

Overall: PASS (scoped to S1 and S3 per FOUND-B3/issue #4; S2 is out of this issue's
scope by design, see notes above). CI run 31079970716 (commit `5977fc9`) is the first
fully observed green run across all three matrix OSes.

If FAIL: per the owning TDD §6.3, the Phase 1 slice returns to stack re-scoping before
any Gate 1 domain/ports work proceeds. Record the re-scoping decision (and any
resulting ADR) here or link to it.

Recorded by: Claude Code (rust-hexon-coder), on behalf of the repository owner
Date: 2026-08-06
