## E13 Four Risk Paths of D5 — Partially Run (Paths 1/2/4 Discharged by E27 (Three Modelable Risk Paths of D5); Path 3 Still Has Zero Coverage)

[中文](13-D5的四条风险路径.md) · **English** · [日本語](13-D5の4つのリスクパス.ja.md)

**Artifact**: Raw output for Paths 1 / 2 / 4 resides in `e27-d5-paths-2026-08-29.out`
(code `scripts/experiment-apparatus/e27_snapshot_accounting_risk_paths.rs` (`cargo run --release --bin e27_snapshot_accounting_risk_paths`), three rounds byte-identical).
⚠️ **Path 3 (crash replay) has no artifact, because it was never run once**—
its prerequisites are block layer write log + checker + crash-point replay harness, none of which existed.

**Background**: The primary evidence for D5 (Snapshot / Space Accounting Mechanism) is the "D5 (Snapshot / Space Accounting Mechanism) primary + sparse side table secondary" convergence form, whose lower half remains an O(1) reduction proof (a proof, not an empirical measurement).
[E12 (Batched Sequential Append vs Unbatched Random Page RMW)](#) measured "batched sequential append vs unbatched random page RMW"; **its Arm A was not an implementation of D5 (Snapshot / Space Accounting Mechanism)**,
and cannot be used to support D5 (Snapshot / Space Accounting Mechanism). E13 (Four Risk Paths of D5) supplements that test.

**Why steady-state kill path is not tested**: No one ever doubted that path, and the risks of D5 (Snapshot / Space Accounting Mechanism) do not reside there.

### Four Paths to Cover and Their Criteria

| # | Path | What is Measured | Criterion |
|---|---|---|---|
| 1 | **Snapshot Creation Boundaries** | Construct one block on each of three boundaries: `birth == S.txg`, `death == S.txg`, `birth == death`; after creating snapshot, check whether it exists in snapshot | Cell-by-cell consistent with boundary table of D5 (Snapshot / Space Accounting Mechanism). **Any cell mismatch is judged red**—wrong `birth == S.txg` is **data loss**, wrong `death == S.txg` is **space leak** |
| 2 | **Cascading Merge on Snapshot Destruction** | How merge duration and write volume scale with deadlist entry count and presence/absence of "bucketing by birth" | D5 (Snapshot / Space Accounting Mechanism) claims `O(deadlist entry count + bucket count)`. **If without bucketing it degenerates to full table scan, bucketing is a mandatory format requirement to be written into D5 (Snapshot / Space Accounting Mechanism)** |
| 3 | **Crash Replay** | Truncate at each crash point during cascading merge, replay, run checker; repeat resumption twice | ① checker reports no errors; ② reaches target state; ③ **repeated resumption yields invariant result**. ⚠️ deadlist appends and cascading merges are **not idempotent** (multiset addition: appending same entry twice ⇒ double free on destruction); this item is where E13 (Four Risk Paths of D5) is most expected to turn red |
| 4 | **Persistence Cost of Defer Queue** | Write volume and I/O count of the "immediate release" half entering defer queue | D16 (Publish Semantics) settled item 15 requires releases within checkpoint not enter allocatable set before publish. **E12 (Batched Sequential Append vs Unbatched Random Page RMW) treated this half at zero cost; in reality it is a second append log that must be persisted** |

### What Counts as Failure

| Phenomenon | Verdict |
|---|---|
| All three boundaries consistent, but **none were artificially constructed** (those boundaries never appeared in workload) | **Entire round voided**—"no problems detected" and "never tested" must be separated |
| Path 3 repeated resumption twice yields identical result, but **crash points all fall between batches** | **Entire round voided**; lower truncation granularity to individual write requests |
| Path 2 shows no measurable difference between bucketing and no bucketing | Check first whether dataset was too small; if confirmed not, delete the "must bucket" requirement from D5 (Snapshot / Space Accounting Mechanism) |

**Prerequisites**: Transaction layer, snapshot creation/destruction, deadlist, block layer write log, checker, crash-point replay harness.
**⚠️ This is one of the heaviest prerequisite experiments in this inventory**, but without it, none of the four risk paths of D5 (Snapshot / Space Accounting Mechanism) have ever been verified.

### Affected Decisions

| Decision Item | Relationship | Review |
|---|---|---|
| D5 (Snapshot / Space Accounting Mechanism) settled item 13 | Preparation | 2026-09-20 unaffected: scope of D5 (Snapshot / Space Accounting Mechanism) settled item 13 verbatim names E13 (Four Risk Paths of D5) Path 3, stating crash replay cell still has zero coverage and risk paths must not be declared verified until it runs; Paths 1 / 2 / 4 discharged by E27 (Three Modelable Risk Paths of D5), and settlement of D5 (Snapshot / Space Accounting Mechanism) does not rest on E13 (Four Risk Paths of D5) |
| D16 (Publish Semantics) | No Effect | 2026-09-20 unaffected: E13 (Four Risk Paths of D5) Path 4 only cites D16 (Publish Semantics) settled item 15 as provenance that defer queue must be persisted; criteria and conclusions rest on write volume and I/O count of that queue |

## Historical Versions

Historical entries for E13 (Four Risk Paths of D5) are collected in [experiments-history.md](../experiments-history.md).
