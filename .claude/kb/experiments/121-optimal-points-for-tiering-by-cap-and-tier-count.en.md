<!-- doc-lint:not-numbers M10 -->
## E121 Optimal Points for Tiering by Cap and Tier Count —— Run (2026-09-08, pure arithmetic, 12 unit tests / 14 mutations caught)
[中文](121-按cap切档与档数的最优点.md) · **English** · [日本語](121-cap基準分割と段階数の最適点.ja.md)

Answers the "how tiers should be partitioned" cell (registered under D27 (Small-Data Packing Container) settled item 6 when this experiment was run, now belonging to D27 (Small-Data Packing Container) settled item 13 (which table to take for initial tier table)), testing another candidate family succeeding E120 (Choosing the Common Ratio of a Geometric Size-Class Table).

### Why It Exists

The four candidates of E120 (Choosing the Common Ratio of a Geometric Size-Class Table) belong to **the same family**: geometric progression partitioning by slot width `W`.
Yet what determines occupancy is not `W`, but **`cap` = ⌊(32768 − 107) / (W + 43)⌋** —— how many objects a container holds.
If two tiers share identical `cap`, they hold the identical count of objects ⇒ completely equivalent in terms of occupancy. E121 (Optimal Points for Tiering by Cap and Tier Count) switches to partitioning by `cap`, and verifies a falsifiable prediction.

### Conclusion 1: **More Tiers Is Not Always Better**, the Primary Conclusion of E121

More tiers reduce padding, **but also increase unfilled tail containers** —— one per tier.

| Arm | Tiers | Worst Waste Above Floor | `uniform` | `logunif` | `discrete` |
|---|---|---|---|---|---|
| `wgeo` (geometric by `W` with `r` = 1.125) | 36 | 11.10% | +8.26% | **+11.24%** | +1.91% |
| `capall` (one tier per reachable `cap`) | 248 | 11.19% | **+2.41%** | +12.37% | +0.02% |
| `capgeo125` (geometric by `cap` with 1.25) | 15 | 25.29% | +9.41% | +20.48% | +1.88% |
| `capgeo150` | 10 | 44.54% | +20.04% | +20.41% | +7.01% |
| `capgeo200` | 5 | 53.89% | +33.78% | +52.20% | +1.57% |

**248 tiers wins on `uniform` (+2.41%), but loses to 36 tiers on `logunif` (+12.37% vs +11.24%).**
The cause is tail containers: under `logunif` total containers is only 1592, whereas 248 tiers means 248 tail containers, **this item alone accounting for 15%**.
⇒ **Optimal tier count depends on "how many objects can be amortized per tier", that is, on workload scale, not merely on distribution shape.**
Assertion `more_tiers_is_not_always_better` guards this.

### Conclusion 2: Geometric by `cap` Is a Bad Parameterization

The three `capgeo` arms are uniformly inferior: worst-case waste 25%–54%, losing to `wgeo` on both neutral distributions in actual runs.
The only "winning" cell is on `discrete` where `capgeo125` uses 2 fewer containers than `wgeo` (0.03%),
and that distribution is not neutral (three sampling points are all powers of 2, as noted in E120 (Choosing the Common Ratio of a Geometric Size-Class Table)).
⇒ The pre-run clause "If `capgeo` family saves nothing over `wgeo` on all distributions ⇒ partitioning by `cap` offers no benefit"
**did not trigger literally** (missed by those 2 containers), but **triggered in substance** —— recorded faithfully side by side without altering measurement basis.

### Conclusion 3: Pre-Run "Deduplication" Prediction Is Empty at This Common Ratio

The prediction was "container count only decreases or stays equal after deduplicating by `cap`". The direction is sound (merging two tiers with identical `cap` eliminates one tail container),
but **the 36-tier table at `r` = 1.125 remains 36 tiers after deduplication** —— every step crosses a `cap` boundary, having no redundancy originally.
Switching to a table finer than `cap` resolution (`r` = 1.01, 370 tiers), deduplication immediately slashes it to 208 tiers, removing 44%.
⇒ **Deduplication only bites when the table is finer than `cap` resolution**; all three numbers remain in source code.

### Conclusion 4: No Difference in `W_min` Between 32 and 64

On `capgeo150`, `W_min` = 32 and 64 are **byte-for-byte identical** (the finest tier in that table is 65, unreachable by 32);
at 128 tier count drops from 10 to 8, with loss on `logunif` rising from 20.41% to 36.06%.
⇒ Pushing floor below 64 yields no gain, while raising to 128 incurs noticeable cost.

### An Equivalent Mutation, Retained in Archive Rather Than Blind Spot

M10 originally removed `tier_widths.push(LIMIT)` at the end of `capall_tiers`, and not a single test turned red ——
because the maximum slot width calculated for `cap` = 7 tier was already 4096 (`32661/7 − 43 = 4622`, truncated by limit to 4096,
and `objects_per_container_for_slot_width(4096)` happened to still be 7) ⇒ that statement was redundant, yielding identical value across all inputs when removed.
Triaged per `.claude/rules/mutation-sampling.md` as **equivalent mutation**: equivalence documented in
`capall_contains_limit_without_the_explicit_push` in archive, and replaced by one that truly changes behavior (removing limit truncation),
which turned 8 tests red immediately. 14/14 with zero blind spots.

### What It Cannot Answer

Pure arithmetic ledger: no implementation, no I/O, no crash-point replay. Does not answer wall clock, cache, or concurrency.
`var` is one specific packing method for variable length (ascending size greedy), not optimal packing.
**Does not answer absolute value of concurrently open containers** (requires multiplying tree count processed concurrently by compactor; D19 (Block Pointer Structure & Width Budget) settled item 6 chose key headed by birth tree following class tag, and recalculating multiplier accordingly was not done).
**Does not answer "what the optimal tier count is"** —— it varies with workload scale, while the three columns in smallfile row of D25 (Target Workload Priority) are all unmeasured (C174 (Three columns of smallfile row unmeasured)).
E121 (Optimal Points for Tiering by Cap and Tier Count) provides the shape that "tier count has an optimum" along with its two endpoints, not the position of that point.

### Measurement Basis and Artifact

Code `scripts/experiment-apparatus/e121_capacity_tiers.rs` (`cargo run --release --bin e121-capacity-tiers`),
raw output `e121-cap-tiers-2026-09-12.out` (49 lines, ending line `name=done emitted=49`; revision with container header 107 and net payload 32661, previous revision `e121-cap-tiers-2026-09-08.out` kept in results/ untouched).
Mutation table `scripts/mutation/tables/e121_capacity_tiers.tsv` (all 14 caught, `e121-mutate-2026-09-12.log`).
N = 100 000, unit 32768, container header 107, net payload 32661, per-slot overhead 43, limit 4096, `W_min` = 64 (additionally swept 32 / 128).
Replay: `bash scripts/experiment-apparatus/replay.sh` with E121 (Optimal Points for Tiering by Cap and Tier Count) number.

### Influenced Decisions

| Decision Item | Relationship | Review |
|---|---|---|
| D27 (Small-Data Packing Container) settled item 13 | Support | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D27 (Small-Data Packing Container) settled item 12 | Support | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D19 (Block Pointer Structure & Width Budget) settled item 6 | No effect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D25 (Target Workload Priority) | No effect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |

## Historical Versions

Historical entries for E121 (Optimal Points for Tiering by Cap and Tier Count) are consolidated in [experiments-history.md](../experiments-history.md).
