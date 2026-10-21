## E20 CPU Cache Cost of Widening Pointers —— Ran (2026-08-28)

[中文](20-指针变宽的CPU缓存代价.md) · **English** · [日本語](20-指针变宽的CPU缓存代价.ja.md)

**Background**: Prior to E20 (CPU Cache Cost of Widening Pointers), [decisions.md](../decisions.md) D19 (Block Pointer Structure & Width Budget) only measured two metrics—internal node **resident memory capacity**
(10.6 → 68.8 GB/PiB) and **tree depth**, both of which calculated "very small cost".
**However, the CPU time of a single point lookup was never measured**: wider entries ⇒ fewer keys fit into a cache line ⇒
each lookup within a node touches more cache lines, and the working set as a whole expands. What is consumed is L1/L2/L3 and DRAM, not cheap disk.

**Conclusions**:

1. **Cost manifests only when working set exceeds L3** (host L3 is 32 MiB), ns per point lookup:

   | Working Set | 4 KiB Node 16B → 111B | 16 KiB Node 16B → 111B |
   |---|---|---|
   | 40 K (within L1) | 59 → 59 (**1.00×**) | 63 → 56 (0.89×) |
   | 528 K (within L2) | 74 → 76 (1.02×) | 74 → 84 (1.13×) |
   | 8.3 M (within L3) | 108 → 122 (1.12×) | 98 → 122 (1.24×) |
   | **134 M (exceeding L3)** | 179 → 467 (**2.61×**) | 206 → 541 (**2.62×**) |

2. **Sweeping all node size tiers reveals a clear minimum point: 16 KiB (supplementary test on 2026-08-29).**

   ⚠️ **The initial version only tested two tiers, 4 KiB and 16 KiB**, leading "16 KiB is better" to be interpreted as "16 KiB is optimal"—
   intermediate and outer points were never tested. Expanding to 6 tiers across three rounds (entry 40 bytes = project pointer head, unit ns):

   | Working Set | 2K | 4K | 8K | **16K** | 32K | 64K |
   |---|---|---|---|---|---|---|
   | 2048 (within L1) | **55** | 57 | 57 | 59 | 61 | 66 |
   | 32768 (within L2) | 77 | 80 | 76 | **75** | 76 | 77 |
   | 524288 (within L3) | 114 | **111** | 117 | 122 | 120 | 114 |
   | **8388608 (exceeding L3)** | 410 | 369 | 480 | **243** | 277 | 489 |

   **Only the tier exceeding L3 displays a distinct minimum point**: 16 KiB is 14% faster than the second-best 32K, 34% faster than 4K, and 50% faster than 64K.
   Under small working sets, 2K/4K is actually fastest. ⚠️ **This observation was once described as "consistent with 'automatic selection based on device characteristics'",
   whereas D8 (Core Index Structure) settled item 2 was settled on 2026-08-30 as "16 KiB, pinned as a constant, not computed by device characteristics"**—
   the exceeding-L3 tier of E20 (CPU Cache Cost of Widening Pointers) serves precisely as the basis side of that settlement;
   the reversal in small working set tiers presents no contradiction, merely demonstrating that 16 KiB is the minimum point on the exceeding-L3 tier.

   Inter-round variance across three rounds was mostly within 2%, maximum 4.6% (`e20-sweep-2026-08-29.out`).

   **Rounds 4 and 5 (2026-08-29, `e20-fanout-2026-08-29-repro.out` and `e20-poscontrol-2026-08-29.out`) reproduced the identical curve shape**,
   ⚠️ Round 5 was run **after each configuration individually passed its positive control**, whereas the first four rounds were not.
   The exceeding-L3 tier 2K/4K/8K/**16K**/32K/64K = 407.6 / 351.5 / 462.7 / **238.5** / 269.2 / 473.2 ns,
   differing tier-by-tier from the first three rounds by 0.6–4.7%. **16 KiB remains the sole minimum point**, and the 8K anomaly was reproduced for the fourth time.

   **Round 6 (2026-08-31, `e20-fanout-2026-08-31-repro.out`, via full run tier of `replay.sh`)**:
   The exceeding-L3 tier 2K/4K/8K/**16K**/32K/64K = 420.83 / 369.22 / 478.06 / **243.50** / 276.72 / 478.08 ns.
   Differing tier-by-tier from Round 5 by 0.2–3.4%, **16 KiB remains the sole minimum point**. Round 6 also ran on configurations that passed the gate.

   ⚠️ **8K is worse than both 4K and 16K, stable across six rounds (480 / 501.78 / 478.06, etc.), which I cannot explain.**
   Tree depth is 4 for both (3 for 16K). Documented faithfully without smoothing over.
   ⚠️ **This is not an inconsequential cell**: The automatic node size selection criterion in D8 (Core Index Structure) must include pointer width
   (D19 (Block Pointer Structure & Width Budget) settled), yet a curve with an **unexplained turning point** cannot support a deterministic selection rule.
   Before it is explained, node size can only be pinned down as a constant, and cannot be phrased as "computed by device characteristics".

3. **Node size determines at which tier cost starts**—this is the most useful takeaway:

   | Entry Width | 4 KiB Node | 16 KiB Node |
   |---|---|---|
   | 40 B (this project's pointer head) | **1.98×** (tree depth 3→4) | **1.14×** (tree depth remains 3) |
   | 67 B | 2.26× | 2.22× |
   | 111 B | 2.61× (depth 5) | 2.62× (depth 4) |

   ⇒ **Large nodes absorb pointer widening.** For the same 40 bytes, a 4 KiB node pays +98%, while a 16 KiB node pays only +14%.

4. ⚠️ **E20 (CPU Cache Cost of Widening Pointers) and [experiments.md](../experiments.md) E7 (Offline Index Harness) point in opposite directions, but they are not contradictory.**
   E7 (Offline Index Harness) measures **device I/O**, concluding "small nodes win on both ends"; E20 (CPU Cache Cost of Widening Pointers) measures **CPU cache**,
   concluding "large nodes are significantly better when pointers are wide". **They measure two different resources.**
   ⇒ The debt recorded in D4 (Checksum Location) (E7 (Offline Index Harness) owes a rerun with final pointer width) **has a second justification**:
   not only does fanout affect device I/O, but node size and pointer width are coupled on the cache side as well.

**Measurement Basis**: Pure CPU/memory experiment. Implicit B-tree laid out in an array, binary search within nodes, entries striding by `entry_bytes`,
with the second 8 bytes of an entry storing child node ID (forming a dependent load chain). **Fixed key count, non-fixed working set**—
entry widens ⇒ working set expands ⇒ evicted from cache, which is the question being asked.
Host per-core L1d 48 KiB, L2 1 MiB, L3 32 MiB, cache line 64 bytes.
Code `scripts/experiments/e20_fanout.rs` (`cargo run --release --bin e20-fanout`),
raw output `e20-fanout-2026-08-28.out` (two-tier initial version) and
`e20-sweep-2026-08-29.out` (six tiers × three rounds).

### ⚠️ Positive control ran only one point before revision, while the group producing conclusions never passed the gate (2026-08-29 adversarial verification revision)

**This is the second repetition of the lesson from E18 (Whether D5 Scraps Under Multiple Forks / Multiple Writable Heads) in this repository.**
`.claude/singlefs-ai-sop/rules/test-discipline.md` verbatim:
"Running positive control on only one arm is equivalent to the other arm never passing the gate, **and it happens to be the very arm producing the conclusion**".

The positive control for E20 (CPU Cache Cost of Widening Pointers) was previously hard-coded on **`entry_bytes=16` / `node_bytes=4096`**, passing after a single run,
while the conclusions were produced by the sweep over **`entry_bytes=40` × six node tiers**—
both conclusions ("16 KiB is the minimum point" and "8 KiB anomaly") stemmed from there, and **the discriminative power of that configuration group was never verified**.
⚠️ In particular for the 8 KiB tier: **for an anomaly on an ungated configuration, one cannot distinguish whether it is a property or a measurement error.**

**Revision**: Positive control changed to sweep 6 node tiers × 3 entry width tiers (16 / 40 / 111) for a total of **18 configurations**,
any single cell with ratio < 2 invalidates the entire round.

**Result: All 18 configurations passed, ratios 2.97–9.81**
(raw output `e20-poscontrol-2026-08-29.out`).

| Node | entry=16 | entry=40 | entry=111 |
|---|---|---|---|
| 2 KiB | 5.24 | 7.58 | 7.49 |
| 4 KiB | 3.07 | 6.59 | 8.24 |
| **8 KiB** | 2.97 | **8.34** | 6.72 |
| 16 KiB | 3.52 | 4.19 | 9.81 |
| 32 KiB | 5.15 | 4.48 | 6.05 |
| 64 KiB | 5.36 | 7.26 | 5.71 |

**Same-round sweep (Round 5) reproduced tier-by-tier** (exceeding-L3 tier, entry=40, unit ns):
2K **406.98** / 4K **362.94** / 8K **501.78** / **16K 249.43** / 32K **276.78** / 64K **479.17**.
⇒ **16 KiB remains the sole minimum point, 8 KiB anomaly reproduced for the fifth time**, and in Round 5 it reproduced on configurations that **passed the gate**.
Round 6 (2026-08-31) achieved the **sixth reproduction** under the same gate, see the paragraph in Conclusion 2.

⇒ **Adding the gate did not overturn any conclusion, but it changed the nature of the anomaly**:
the ratio in cell `8192 / 40` was the **highest in the 40-byte column (8.34)**,
showing that the DRAM tier of this configuration was indeed significantly slowed down—
**the 8 KiB anomaly is therefore more likely a genuine effect rather than a measurement artifact.** It remains unexplained and is faithfully documented.

⚠️ **Adding the gate cannot retroactively imply "rounds prior to 2026-08-29 also passed the gate"**: the gate did not exist until that round,
numbers from previous four rounds were simply **not verified by this gate**, not verified and passed. Do not conflate the two.

### Positive Control Caught Three Versions, Two of Which Were Modeling Errors

**Control**: Under the same entry width, a working set fitting in L1 must be significantly faster than one exceeding L3 (ratio ≥ 2).

| Version | Ratio | Cause |
|---|---|---|
| Version 1 | 1.01 | RNG and modulo operations overwhelmed tree traversal itself (RNG and u64 modulo in timing section) |
| Version 2 | 0.99 | Lookups were independent; **out-of-order execution parallelized dozens of cache misses**—measuring bandwidth rather than latency |
| Version 3 | 0.99 | **Data layout error**: Node 0 keys were 0..251 while lookup keys were uniformly distributed across domain; 99.99% of lookups landed on rightmost leaf at level 1, traversing the same few nodes, all in L1 |
| Version 4 | **3.03** ✓ | Fix: pre-generate keys outside timing section, serial dependencies between lookups, each node takes key bit-slices by layer from same domain |

⚠️ **Without this control, tables from the first three versions would have been published claiming "pointer widening carries no cost".**
Six unit tests proved they go red via targeted breakage (all 5 items caught in 2026-09-03 mutation table `scripts/mutation/tables/e20_fanout.tsv`):
`lookups_spread_across_nodes` specifically catches the layout bug in version 3; two added tests pin blind spots prior to 2026-09-03—
binary search hit position (inverted comparison direction originally did not go red) and child pointers not overlapping with keys (misaligned pointer chase read originally did not go red).

### Impacted Decisions

| Decision Item | Relationship | Review |
|---|---|---|
| D19 (Block Pointer Structure & Width Budget) settled item 1 | Supports | 2026-10-11 Unaffected: wording adjustments only (temporal words, location references, self-references); conclusions, numbers, and artifacts unchanged |
| D8 (Core Index Structure) settled item 2 | Supports | 2026-10-11 Unaffected: wording adjustments only (temporal words, location references, self-references); conclusions, numbers, and artifacts unchanged |
| D4 (Checksum Location) | Does not affect | 2026-10-11 Unaffected: wording adjustments only (temporal words, location references, self-references); conclusions, numbers, and artifacts unchanged |

## Historical Versions

E20 (CPU Cache Cost of Widening Pointers) history entries are consolidated in [experiments-history.md](../experiments-history.md).
