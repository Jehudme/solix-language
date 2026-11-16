# Milestone 058: Dynamic Release Benchmarking, Release Notes Comparison & CI/CD Gates

## Document Metadata
* **Milestone ID:** MS-058
* **Target Version:** `v2.0.0`
* **Milestone Branch:** `milestone/058-dynamic-release-benchmarking-and-ci-cd`
* **Status:** DRAFT
* **Lead Architect:** CI/CD & Performance Engineering Team
* **Target Delivery Date:** 2026-11-16
* **Related Specifications:** [GL-0001](../../internals/guidelines/authoring-commits.md), [GL-0008](../../internals/guidelines/authoring-milestones.md), [GL-0011](../../internals/guidelines/language-conventions.md), [GL-0013](../../internals/guidelines/documentation-update-protocol.md), [SPEC-0001](../../spec/README.md)

---

## 1. Executive Summary & Objectives

Milestone 058 replaces static historical benchmark JSON comparisons with a dynamic release-to-release benchmarking pipeline, introduces automated benchmark comparison tables in GitHub release notes, and establishes a mandatory PR gate running `python3 benchmarks/runner.py --verify`.

Key objectives:
1. **Dynamic Release Baseline Pipeline**:
   - During release execution (`release.yml`), run `benchmarks/runner.py --json --output benchmarks/baselines/release_baseline.json`.
   - Automatically attach the generated baseline as a GitHub release asset.
2. **Automated Release Notes Comparison Table**:
   - Fetch the preceding release's benchmark JSON via GitHub API / release asset cache.
   - Run `python3 benchmarks/compare_releases.py` to calculate percentage speedups, memory peak differences, and instruction counts across all 16 benchmark suites.
   - Automatically inject a formatted GitHub Flavored Markdown comparison table directly into the release notes body.
3. **CI/CD Quality Gates**:
   - Add `python3 benchmarks/runner.py --verify` as a mandatory blocking gate in `.github/workflows/ci.yml`.
   - On pull requests, compare performance against the preceding release baseline, alerting or failing if performance regresses beyond acceptable thresholds (> 5%).

---

## 2. Priority Blocker Taxonomy & Deliverables

| Priority | Task ID | Subsystem | Description | Release Impact |
|:---|:---|:---|:---|:---|
| **P0** | `MS058-T01` | `ci/cd` | Add `python3 benchmarks/runner.py --verify` as a blocking gate in `.github/workflows/ci.yml`. | Build integrity |
| **P0** | `MS058-T02` | `benchmarks` | Create `benchmarks/compare_releases.py` generating Markdown diff tables and regression analyses. | Performance reporting |
| **P1** | `MS058-T03` | `ci/cd` | Update `.github/workflows/release.yml` to fetch preceding baseline, generate comparison table, and embed in release notes. | Release automation |
| **P2** | `MS058-T04` | `docs` | Update benchmarking documentation and release procedures. | Living documentation |

---

## 3. Comprehensive Test Matrix

### A. Positive Conformance Tests
1. `test_compare_releases_table_generation`: Running `compare_releases.py` with mock current and previous release baselines produces a valid Markdown table with execution time diffs and memory diffs.
2. `test_ci_benchmark_verify_gate`: `python3 benchmarks/runner.py --verify` executes all 16 cross-engine workloads and passes cleanly.
3. `test_release_baseline_json_schema`: Verified JSON schema matches expectations for automated ingestion.

---

## 4. Definition of Done (DoD)
1. CI pipeline runs benchmark verification on every pull request.
2. Release workflow verified to attach baseline and embed comparison table in release notes.
3. Documentation synchronized.
