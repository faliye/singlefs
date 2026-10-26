## E10 Aging: Structural Resistance vs Post-hoc Compaction — Partially Run (First Segment Ran 4 of 21 Shards, Readings Serve Only as Leads; Remaining 17 Shards Halted, Whether to Revise Three Registered Items Pending)

[中文](10-老化结构抗老化vs事后整理.md) · **English** · [日本語](10-老化-構造的抗老化-vs-事後デフラグ.ja.md)

**Background**: D3 (Space Allocation) committed to resident background compaction, on the grounds that lack of categorization concentrates fragmentation pressure. FAST 2017 empirical measurements (see [prior-art.md](../prior-art.md) 7.2) offer another path—**aging can be absorbed by the index structure itself**. Whether the two paths are alternative or additive determines the weight of D3 (Space Allocation) settled item 14, and serves as the criterion for Candidate B of D10 (True Answer to Random Writes / Fragmentation).

**Methodology**: Following the methodology of FAST 2017, use **continuous git checkout of Linux kernel source code** as the aging workload, measuring traversal bandwidth and fragmentation degree at fixed round intervals. Three comparison groups: ① Background compaction disabled ② Background compaction enabled ③ Background compaction disabled but index adopts aging-resistant form.

**Criteria**:
- If ③ can reach the level of ②, then "resident background compaction" in D3 (Space Allocation) can be demoted from "commitment" to "optimization".
- The gain of ② relative to ① must be quantifiable, otherwise the commitment in Item 2 of D3 (Space Allocation) lacks data support.

**What counts as failure**: If ① shows no measurable degradation after aging, it indicates the aging workload was insufficiently aggressive or the dataset was too small, **voiding the entire round**—the easiest way an aging experiment deceives itself is mistaking "no degradation measured" for "immune to aging".

**Prerequisites**: File system, background compaction implementation, and a fragmentation measurement tool.

**Measurement basis and artifacts**: The first run follows the pre-run registration `research/prompts/e10-r1-prereg.md` (written 2026-10-13; its question and criteria govern, not the three-group comparison above); apparatus `scripts/experiment-apparatus/e10_aging_compaction.rs`, mutation table `scripts/mutation/tables/e10_aging_compaction.tsv` (21 rows). Four of the 21 shards of the first segment have run; raw outputs are archived and read from git history: `git show e1b1ceb95:research/results/e10-aging-compaction-2026-10-13/main-s4.out`, `git show e1b1ceb95:research/results/e10-aging-compaction-2026-10-13/disk_1gib-s4.out`, `git show e1b1ceb95:research/results/e10-aging-compaction-2026-10-13/disk_1gib-s8.out`, `git show e1b1ceb95:research/results/e10-aging-compaction-2026-10-13/disk_1gib-s16.out`; the other 17 shards are stopped, and the readings are leads only. This registration yields only the compaction floor `N₀`; the product batch size and the three compaction watermarks do not belong to it: line 32 of the registration leaves row 6 of the fork sheet (batch size) outside, and the start watermarks on line 500 are values the registration takes, not quantities it measures.

### Affected Decisions

| Decision Item | Relationship | Review |
|---|---|---|
| D3 (Space Allocation) settled item 14 | Preparation | 2026-10-26 unaffected: raw outputs now cited from git history, and the page states that the product batch size and the three compaction watermarks do not belong to this registration; numbers and conclusions unchanged; prior 2026-10-24 unaffected: product full paths completed with directory prefix; numbers and conclusions unchanged; prior 2026-10-21 unaffected: heading status and index row now say partially run; the preparation relationship and conclusions are unchanged (readings serve only as leads); prior 2026-10-17 unaffected: this round only synchronizes heading parentheticals with index row; conclusions, numbers, and artifacts remain unchanged; prior 2026-10-11 unaffected: only phrasing revised (temporal words, positional references, and self-references); conclusions, numbers, and artifacts remain unchanged |
| D10 (True Answer to Random Writes / Fragmentation) settled item 1 | Preparation | 2026-10-26 unaffected: raw outputs now cited from git history, and the page states that the product batch size and the three compaction watermarks do not belong to this registration; numbers and conclusions unchanged; prior 2026-10-24 unaffected: product full paths completed with directory prefix; numbers and conclusions unchanged; prior 2026-10-21 unaffected: heading status and index row now say partially run; the preparation relationship and conclusions are unchanged (readings serve only as leads); prior 2026-10-17 unaffected: this round only synchronizes heading parentheticals with index row; conclusions, numbers, and artifacts remain unchanged; prior 2026-10-11 unaffected: only phrasing revised (temporal words, positional references, and self-references); conclusions, numbers, and artifacts remain unchanged |

## Historical Versions

Historical entries for E10 (Aging: Structural Resistance vs Post-hoc Compaction) are collected in [experiments-history.md](../experiments-history.md).
