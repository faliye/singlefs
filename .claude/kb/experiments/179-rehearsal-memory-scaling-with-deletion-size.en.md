## E179 Rehearsal Memory Scaling with Deletion Size —— Not run (pre-run registration written, apparatus not written)

[中文](179-预演内存随删除规模.md) · **English** · [日本語](179-削除規模に伴う演習メモリ.ja.md)

**What is asked**: Before number acquisition, the writable mount rehearses the sequence that raises F, plus the continuation, on an in-memory-only disk. How do the bytes recorded in this pass (primary measurement basis M) grow with the deletion size (U, the number of data units remaining in the one file in the deletion queue), and is that enough to judge the four candidates of code-triad m3-b44-code-r2 fork two (the in-memory-only disk has no upper bound on memory): reject when over a cap, skip the rehearsal when over a cap, measure first then decide, or keep the status quo. On 2026-10-17, on that round's fork sheet, the user chose "measure first then decide".

**Pre-run registration**: `e179-preregistration.md` (the question, the reading, the arms and controls, the sampling points, the criteria, the failure clauses, and the void and halt clauses are all in that one document; revision one is in section twelve). The subject under test is the implementation (`DeviceKeepingWritesInMemory` in `crates/singlefs-core/src/mounted_session.rs` and the rehearsal before number acquisition in `crates/singlefs-core/src/mount.rs`); there is no kb clause.

**Current state**: Only the pre-run registration exists; the apparatus, unit tests, mutation table, and artifacts do not exist yet; raw output is not retained (not run yet).

### What it cannot answer

- There is no reading before the run, so this page gives no conclusion on any quantity.

### Affected decisions

| Decision item | Relationship | Look-back |
|---|---|---|
| D16 (Publish Semantics) settled item 1 | Preparation | 2026-10-21 unaffected: archiving the previous round's record only changed paths pointing to deleted files to bare file names, the conclusion and artifacts are unchanged; before that, 2026-10-18 unaffected: only the page was created to register the pre-run registration, with no reading |

## Historical Versions

The history entries of E179 (Rehearsal Memory Scaling with Deletion Size) are collected in [experiments-history.md](../experiments-history.md).
