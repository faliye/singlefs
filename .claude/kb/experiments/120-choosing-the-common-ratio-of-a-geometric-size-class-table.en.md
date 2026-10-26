## E120 Choosing the Common Ratio of a Geometric Size-Class Table —— Run (2026-09-08, pure arithmetic, 8 unit tests / 13 mutations caught)

[中文](120-档表取等比时公比怎么定.md) · **English** · [日本語](120-等比のサイズクラス表で公比をどう決めるか.ja.md)

Answers "how tiers should be partitioned" following D27 (Small-Data Packing Container) settled item 8 ④, succeeding E119 (Padding Cost of Fixed-Width Slots).

### Why It Exists

E119 (Padding Cost of Fixed-Width Slots) concluded that "four tiers are too coarse, tiers must be dense at the small end", but it compared five **specific** tables,
and those tables were all picked according to certain distributions —— **object size distribution has never been measured repository-wide** (C174 (Three columns of smallfile row unmeasured)).
Picking tables according to unmeasured distributions equals treating unknown quantities as known.

E120 (Choosing the Common Ratio of a Geometric Size-Class Table) takes a different route: **the upper bound on relative waste for geometric progression tier tables is independent of distribution**.
With tier widths `W_k = W_min · r^k` (truncated to ≤ 4096), objects falling into the k-th tier satisfy `W_{k-1} < s ≤ W_k`
⇒ bytes occupied after padding do not exceed `r` times its own size ⇒ **worst-case relative waste = 1 − 1/r, independent of distribution.**
"How tiers should be partitioned" thus collapses into a one-dimensional choice: smaller `r` yields smaller waste and more tiers.

### Conclusion: Balance Sheet of Four Common Ratios

`W_min` = 64, final tier pinned to boundary 4096 (D27 (Small-Data Packing Container) settled item 2).

| Common Ratio `r` | Tiers | Worst Waste Above Floor | Closed Form `1 − 1/r` | `uniform` Actual | `logunif` Actual | `discrete` Actual |
|---|---|---|---|---|---|---|
| 1.125 | 36 | 11.10% | 11.11% | +8.26% | +11.24% | +1.91% |
| 1.25 | 20 | 19.99% | 20.00% | +12.62% | +14.32% | +4.09% |
| 1.5 | 12 | 33.31% | 33.33% | +18.87% | +19.60% | +4.64% |
| 2.0 | 7 | 49.98% | 50.00% | +38.86% | +31.85% | +0.02% |

**Actual loss is container count difference relative to variable-length greedy packing.** Each tier's `cap` is ≥ 4 (coarsest tier 4096 has `cap` = 7),
passing all cells of cost recovery gate in E116 (Packed Container Balance: Metadata Writes and Compaction Strategy).

### Closed Form Registered Before Run Judged False Twice, Neither Being Trivial

1. **Floor is not the common ratio.** The registered expression was "worst-case relative waste = `1 − 1/r`", whereas the first tier covers 1..=`W_min`,
   where a 1-byte object padded to 64 wastes **98.44%**, **a figure completely identical across all four common ratios**.
   ⇒ Closed form holds only above `W_min`; the floor segment must be viewed in **absolute bytes**: each object wastes at most 63 bytes.
2. **Integer rounding makes actual tier ratio slightly exceed `r`** (81 → 92 is 1.1358 > 1.125) ⇒ upper bound must relax to `1 − 1/(r + 1/W_min)`.
   Actual runs in all four cells fell within relaxed bound, and were also slightly lower than closed form itself.

**Forbidding modifying the registered formula retroactively for either clause**, with all three numbers retained side by side in source code.

### The `discrete` Distribution Is Not Neutral

Its three sampling points 512 / 1024 / 4096 **are all powers of 2** ⇒ `r` = 2.0 has only 1 extra container (+0.02%) on it,
whereas `r` = 1.125 has 1.91% more. **Looking only at that distribution would yield the conclusion that "larger common ratio is better", contradicting the other two distributions.**
This is the second occurrence of the sampling point trap noted in E119 (Padding Cost of Fixed-Width Slots), in the opposite direction.

### Comparison Against Equidistant Tables of E119

`gran64` (64-tier equidistant) in E119 (Padding Cost of Fixed-Width Slots) loses only 1.76% on `uniform`, beating every geometric table here.
**However it lacks a distribution-independent upper bound**: equidistant tables can likewise reach 49% relative waste at the small end (65-byte object padded to 128),
it just happened that little mass accumulated on those tiers.
⇒ **When distribution is unmeasured, choose the bounded one**; whenever C174 (Three columns of smallfile row unmeasured) is repaid and distribution measured,
equidistant tables can re-enter comparison.

### What It Cannot Answer

Pure arithmetic ledger: no implementation, no I/O, no crash-point replay. Does not answer wall clock, cache, or concurrency.
`var` is one specific packing method for variable length (ascending size greedy), not the optimal packing method.
**Does not answer absolute value of "concurrently open containers"**: it equals tier count multiplied by tree count processed concurrently by compactor,
and tree count depends on D19 (Block Pointer Structure & Width Budget) settled item 6 (key encoding of central mapping entries) ——
if key is headed by tree ID, traversing in key order touches only one tree at a time and the multiplier is 1; D19 (Block Pointer Structure & Width Budget) settled item 6 chose key headed by birth tree following class tag, and recalculating that multiplier accordingly was not done (whether birth tree is the same thing as the "tree" processed per-tree by compactor was not audited).
Does not answer what `W_min` should be: 64 is a value fixed before run, not swept.

### Measurement Basis and Artifact

Code `scripts/experiment-apparatus/e120_tier_ratio.rs` (`cargo run --release --bin e120-tier-ratio`),
raw output `e120-tier-ratio-2026-09-12.out` (102 lines, ending line `name=done emitted=102`; revision with container header 107 and net payload 32661, previous revision `e120-tier-ratio-2026-09-08.out` kept in results/ untouched).
Mutation table `scripts/mutation/tables/e120_tier_ratio.tsv` (all 13 caught, `e120-mutate-2026-09-12.log`).
N = 100 000, unit 32768, container header 107, net payload 32661, per-slot overhead 43, `W_min` = 64, boundary 4096.
Replay: `bash scripts/experiment-apparatus/replay.sh` with E120 (Choosing the Common Ratio of a Geometric Size-Class Table) number.

### Influenced Decisions

| Decision Item | Relationship | Review |
|---|---|---|
| D27 (Small-Data Packing Container) settled item 13 | Support | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D27 (Small-Data Packing Container) settled item 8 | No effect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D27 (Small-Data Packing Container) settled item 2 | No effect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D19 (Block Pointer Structure & Width Budget) settled item 6 | No effect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |

## Historical Versions

Historical entries for E120 (Choosing the Common Ratio of a Geometric Size-Class Table) are consolidated in [experiments-history.md](../experiments-history.md).
