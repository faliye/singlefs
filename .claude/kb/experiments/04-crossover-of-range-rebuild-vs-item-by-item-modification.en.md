## E4 Crossover of Range Rebuild vs Item-by-item Modification —— Partially Run (Write amplification half cleared by E30 (Write Amplification Half of the Crossover Point); elapsed time half still owes QEMU and two implementations)

[中文](04-范围重建vs逐项修改的交叉点.md) · **English** · [日本語](04-範囲再構築-vs-個別変更の交差点.ja.md)

**Artifacts**: The raw output of the write amplification half is in `e30-range-rebuild-2026-08-29.out`,
rerun `cargo run --release --bin e30_range_rebuild` (body text in E30 (Write Amplification Half of the Crossover Point)).
⚠️ **The elapsed time half has no artifacts, because it has never been run**—prerequisites are QEMU media simulation and two implementations.


**Background**: Switching criteria for two unbounded operation implementations, see [decisions.md](../decisions.md) D8 (Core Index Structure).
The numbers written in the documentation, "spinning disks approx 1%, SSD approx 10%", are **reasoning, not measurement**, and must be measured directly.

**Independent Variables**: Affected ratio K/N (0.1% / 1% / 5% / 10% / 30% / 100%) ×
Medium (QEMU simulated spinning disk / SSD) × key continuity (contiguous range / randomly scattered).

**What to Measure**: Elapsed time and write amplification to complete the same operation, running both implementations once each.

**Criteria**: Find the crossover point of the two curves under each "medium × continuity" combination,
and formulate it as a computable expression placed into code, not into comments.

**What Counts as Failure**:
- If the crossover points of the "contiguous" and "randomly scattered" groups show no evident difference,
  it indicates that range rebuild did not actually take the path of whole-node discarding. **The implementation is faulty, and the whole round is invalidated.**
  This is where the discriminative power of E4 (Crossover of Range Rebuild vs Item-by-item Modification) lies—the two groups must be distinguishable.
- If the two curves do not intersect over the entire interval, it indicates that one implementation is strictly worse from start to finish;
  in that case, do not create a branch, directly eliminate the worse one (`.claude/rules/fs-design.md`: if criteria cannot be formulated, implement only one).

### Impacted Decisions

| Decision Item | Relationship | Review |
|---|---|---|
| D8 (Core Index Structure) settled item 5 | Preparation | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |

## Historical Versions

E4 (Crossover of Range Rebuild vs Item-by-item Modification) history entries are consolidated in [experiments-history.md](../experiments-history.md).
