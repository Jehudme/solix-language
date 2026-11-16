# Milestone 058: Dynamic Release Benchmarking, Release Notes Comparison & CI/CD Gates

## Document Metadata
* **Milestone ID:** MS-058
* **Target Version:** `v2.0.0`
* **Milestone Branch:** `milestone/058-dynamic-release-benchmarking-and-ci-cd`
* **Status:** COMPLETED
* **Lead Architect:** CI/CD & Performance Engineering Team
* **Target Delivery Date:** 2026-11-16
* **Completed Date:** 2026-11-16
* **Related Specifications:** [GL-0001](../../internals/guidelines/authoring-commits.md), [GL-0008](../../internals/guidelines/authoring-milestones.md), [GL-0011](../../internals/guidelines/language-conventions.md), [GL-0013](../../internals/guidelines/documentation-update-protocol.md), [SPEC-0001](../../spec/README.md)

---

## 1. Executive Summary & Objectives

Milestone 058 replaces static historical benchmark JSON comparisons with a dynamic release-to-release benchmarking pipeline, introduces automated benchmark comparison tables in GitHub release notes, and establishes a mandatory PR gate running `python3 benchmarks/runner.py --verify`.

Key deliverables completed:
1. **Dynamic Release Baseline Pipeline**:
   - During release workflow execution (`.github/workflows/release.yml`), the `benchmark-release` job runs `benchmarks/runner.py --save-baseline staging-benchmarks/release_baseline.json`.
   - The generated `release_baseline.json` is packaged and automatically uploaded as an official GitHub release asset.
2. **Automated Release Notes Comparison Table**:
   - `benchmarks/compare_releases.py` inspects previous release baseline assets via GitHub CLI (`gh release download`) or local fallback baseline archives.
   - Calculates speedup ratios, percentage execution time deltas, and memory peak RSS differences across all 16 benchmark workloads.
   - Injects a formatted GitHub Flavored Markdown comparison table directly into GitHub Release notes (`$RUNNER_TEMP/notes.txt`).
3. **CI/CD Quality Gates**:
   - Integrated `python3 benchmarks/runner.py --verify` as a mandatory blocking quality gate in `.github/workflows/ci.yml`.
   - Enforces a 5% regression threshold via `--fail-on-regression`, ensuring performance regressions block merges.

---

## 2. Priority Blocker Taxonomy & Deliverables

| Priority | Task ID | Subsystem | Description | Release Impact | Status |
|:---|:---|:---|:---|:---|:---|
| **P0** | `MS058-T01` | `ci/cd` | Add `python3 benchmarks/runner.py --verify` as a blocking gate in `.github/workflows/ci.yml`. | Build integrity | COMPLETED |
| **P0** | `MS058-T02` | `benchmarks` | Create `benchmarks/compare_releases.py` generating Markdown diff tables and regression analyses. | Performance reporting | COMPLETED |
| **P1** | `MS058-T03` | `ci/cd` | Update `.github/workflows/release.yml` to fetch preceding baseline, generate comparison table, and embed in release notes. | Release automation | COMPLETED |
| **P2** | `MS058-T04` | `docs` | Update benchmarking documentation and release procedures. | Living documentation | COMPLETED |

---

## 3. Comprehensive Test Matrix

### A. Positive Conformance Tests
1. `test_compare_releases_table_generation`: Running `compare_releases.py` with mock current and previous release baselines produces a valid Markdown table with execution time diffs and memory diffs (`🚀 FASTER`, `⚖️ STABLE`).
2. `test_compare_releases_regression_detection`: Confirmed `--fail-on-regression` exits with non-zero status code when a workload regresses beyond the `--threshold-pct` limit.
3. `test_compare_releases_inject_notes`: Confirmed automated appending and injection of markdown comparison table into existing release notes documents.
4. `test_compare_releases_initial_baseline_mode`: Confirmed fallback to baseline establishment mode when previous release baseline is unavailable.
5. `test_benchmark_runner_verify_command`: Verified all 16 cross-engine workloads pass execution verification across Solix and comparative engines.

---

## 4. Definition of Done (DoD)
1. CI pipeline runs benchmark verification on every pull request.
2. Release workflow verified to attach baseline and embed comparison table in release notes.
3. Conformance integration test suite passing (5/5 tests in `crates/solix-tests/tests/compilation/integration/test_milestone058_benchmarking_and_ci.rs`).
4. Zero clippy warnings, zero documentation drift, and zero GL-0011 violations.
