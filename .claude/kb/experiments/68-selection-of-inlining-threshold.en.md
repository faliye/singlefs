## E68 Selection of Inlining Threshold — Run (2026-08-31, N=5 Seeds; Expanded to 8 Unit Tests on 2026-09-03, All 9 Mutations Caught): **Primary Criterion Failed to Bracket the Interval; True Constraint Is Leaf Record Density**

[中文](68-内联阈值取多少.md) · **English** · [日本語](68-インライン化閾値の選定.ja.md)

D14 (Dual-Track: Large/Small Files & Persistent/Transient) settled item 3 asked verbatim "what inlining threshold to select".
The 3584 B used in E8 (Different Write Paths for Large vs Small Files) **was postulated, not measured**; E68 (Selection of Inlining Threshold) fills this slot.

### Criteria (Hardcoded Before Run, Unchanged After Run)

1. **Primary criterion shares dimension**: Curve of total device bytes per file (data + metadata) against threshold, taking minimum point `T*`.
   ⚠️ Do not subtract read counts from write amplification — those are two different dimensions; this repository stumbled on that exact error on 2026-08-31 regarding D2 (RAID Stripe Policy) width upper bound.
2. `T*` falling on scan interval endpoint ⇒ verdict "interval not bracketed", faithfully recorded, **no extrapolation**.
3. If read count at `T*` exceeds 2× that of no-inlining, faithfully juxtaposed.

### Failure Clauses (Hardcoded Before Run)

| Clause | Verdict |
|---|---|
| **Positive control, run for every threshold tier**: `T=0` must be cell-by-cell identical to "never inlined" | **Passed** |
| **Negative control**: 1 MiB files cell-by-cell identical under any threshold | **Passed** |
| 5 seeds consistent in direction | **Consistent** |

### Measurement (16 KiB Node, 4 KiB Block, 100k Files, 80% Small Files + 20% Large Files)

| Threshold | Total bytes per file | Metadata per file | **Number of leaves (relative to no inlining)** | Coverage of inlining |
|---|---|---|---|---|
| 0 (no inlining) | 108247 | 64 | 391 (1.00×) | 0% |
| 256 | 108051 | 70 | 430 (**1.10×**) | 4.9% |
| **512** | 107866 | 89 | 548 (**1.40×**) | 9.8% |
| 1024 | 107533 | 166 | 1017 (**2.61×**) | 19.8% |
| 2048 | 107018 | 476 | 2906 (7.36×) | 39.7% |
| 3072 | 106708 | 991 | 6053 (15.37×) | 59.6% |
| 3584 (used by E8 (Different Write Paths for Large vs Small Files)) | 106632 | 1324 | 8086 (**20.61×**) | 69.8% |
| 4096 | 106606 | 1710 | 10442 (**26.62×**) | 79.8% |

### Criterion-by-Criterion Verdict

| # | Verdict |
|---|---|
| 1 | Total bytes **monotonically decreases**, minimum point falls on right endpoint 4096 of scan interval |
| 2 | **Triggered**: interval not bracketed ⇒ **primary criterion failed to provide a threshold**, faithfully recorded, no extrapolation |
| 3 | **Could not judge in round 1, supplemented in round 2**: read side gained discriminative power after adding cache model; see "Round 2" section |

### Why the Total Bytes Curve Is Doomed to Be Monotonic

What inlining saves is **intra-block fragmentation** (a 100 B file without inlining consumes a full 4096 block, saving 3996 bytes);
what it pays is **record expansion** (+100 bytes). What is saved is perpetually greater than what is paid, **until file size approaches block size** ⇒
in terms of bytes, inlining up to block size is always advantageous. **The primary criterion asked the wrong question.**

⇒ **The true constraint lies not in bytes, but in leaf record density**: without inlining, one record is 64 bytes ⇒ one 16 KiB leaf holds 256 records;
inlining swells average records, ballooning leaf counts per the table in "Measurement", and the 6.2× read-side cost measured in E8 (Different Write Paths for Large vs Small Files) stems precisely from here.

### Round 2 (Added 2026-08-31): Read Model with Cache — Round 1 Criterion 3 Lacked This to Judge

Device reads for one random point lookup = leaf miss + (only for non-inlined files) one data block read. Interior nodes assumed resident.

**Metadata working set** (100k files): T=0 → 6.1 MiB (64 B per file); T=512 → **8.6 MiB** (89 B);
T=1024 → 15.9 MiB; T=3584 → 126 MiB; T=4096 → 163 MiB.

| Cache \ Threshold | 0 | 256 | **512** | 1024 | 2048 | 3584 | 4096 |
|---|---|---|---|---|---|---|---|
| 1 MiB | 1.836 | 1.802 | 1.784 | 1.738 | 1.577 | 1.290 | **1.191** |
| 4 MiB | **1.345** | 1.355 | 1.434 | 1.549 | 1.511 | 1.266 | 1.173 |
| 16 MiB | 1.000 | 0.951 | 0.901 | **0.801** | 1.247 | 1.171 | 1.099 |
| 64 MiB | 1.000 | 0.951 | 0.901 | 0.801 | **0.599** | 0.791 | 0.805 |
| 256 MiB | 1.000 | 0.951 | 0.901 | 0.801 | 0.599 | 0.298 | **0.198** |

**There is one rule**: **the best threshold is the tier where "the metadata working set just fits into cache"** —
when it fits, inlining saves one data block read; when it overflows, leaf misses lose back all the savings.
⇒ With 4 MiB cache, no inlining is best; with 16 MiB, T=1024 is best; with 256 MiB, T=4096 is best.

**Position of T=512**: At 16 MiB and above, 10% better than no inlining (0.901 vs 1.000);
**at 4 MiB tier, 6.6% worse than no inlining** (1.434 vs 1.345); at 1 MiB tier, 2.8% better.
⇒ It is **the fixed constant with minimal deviation across the entire cache range**, not the optimum for any single tier.

### What It Cannot Answer

2. Does not model append — that **threshold is not a qualified branching variable** was answered by E8 (Different Write Paths for Large vs Small Files):
   when a 4 KiB file is appended in 8 chunks, the first 7 chunks all fall below threshold.
3. Does not model the crash semantics of the bounded transaction "inline → extent" (D14 (Dual-Track: Large/Small Files & Persistent/Transient) settled item 5; has no code object after settled item 3 settled threshold to 0).
4. Size distribution (80% ≤4 KiB) is postulated, not empirically measured in this project.

### Prerequisites

None. Pure counting model.

### Measurement Basis and Re-run

- Code `scripts/experiment-apparatus/e68_inline_threshold.rs`
  (`cargo run --release --bin e68-inline-threshold`),
  raw output `e68-inline-threshold-2026-08-31.out` (90 lines, ending with line `emitted=90`).
- Mutation table `scripts/mutation/tables/e68_inline_threshold.tsv` (2026-09-03, all 9 caught).
  When writing table, discovered Round 2 cached read model ("optimal threshold = tier where working set fits cache" originated from that code)
  originally had **zero assertions**; added 4 unit tests: inlining count independent recalculation, hit rate endpoints, tree height and fanout arithmetic,
  size distribution two peaks (`size_distribution_has_both_modes` simultaneously pins both peaks and "large file tier non-empty",
  negative control `negative_control_huge_file_same_everywhere` existed before addition) —
  corresponding mutations originally turned no test red.

### Impacted Decisions

| Decision Item | Relationship | Review |
|---|---|---|
| D14 (Dual-Track: Large/Small Files & Persistent/Transient) settled item 3 | Material prep | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D14 (Dual-Track: Large/Small Files & Persistent/Transient) settled item 5 | Does not affect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D2 (RAID Stripe Policy) | Does not affect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |

## Historical Versions

Historical entries for E68 (Selection of Inlining Threshold) are concentrated in [experiments-history.md](../experiments-history.md).
