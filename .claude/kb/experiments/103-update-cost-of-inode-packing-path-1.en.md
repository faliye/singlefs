## E103 Update Cost of Inode Packing Path 1 — Run (2026-09-04, pure arithmetic, 13 unit tests / 11 mutants caught; 2026-09-05 added third arm "leaves as containers"): **Single update on packing side is 1.25–1.5x of residing index leaves (both sides count dirty leaves plus all ancestors), batch dispersed 1.3–1.82x, full run rewrite slightly cheaper (0.989); stat read 2.25–2.5x bytes; blast radius 233 vs 116; round 2 forward leg's "≈ 3x" only counted leaves, invalidated. Third arm (inode tree leaves are code 3 containers, internal nodes are container index, fanout 136 at entry 120, header 58): write k = 1 identical per cell with path 1, k ≥ 10 extra is internal level nodes exceeding container index, k = 1000 cell exactly 2 nodes (1e6: extra 65536 bytes, 0.11%), read only 1.25–1.5x**

[中文](103-inode打包路的更新代价.md) · **English** · [日本語](103-inodeパッキング経路1の更新コスト.ja.md)

**This is the measurement owed by the comparison table in D8 (Core Index Structure) settled item 6**: how much path 1 (inode records packed into code 3 containers, inode tree references by identity, container index holds physical pointers)
differs in updates and reads from the ruled-out "residing index leaves" form. When D18 (Information Carried in a Block) settled item 11 landed, it only wrote the estimate from the round 2 forward derivation leg; the user requested actual measurement.

### Cited Items (Copied Verbatim into Source Comments)

| Item | The Exact Sentence Used |
|---|---|
| D23 (Journal Role & Format) settled item 1 | Take Option A: "Each fsync writes dirty leaves + all ancestors + root slot + one record. Ancestors not delayed" ⇒ tree height must be counted into update cost |
| D18 (Information Carried in a Block) settled item 11 | Packed record unit header 107 (2026-09-12 added birth seq 4; before 2026-09-05 C113 (No Input to Current-Version Determination of Multi-Version Units in Scan Rebuild) finalized adding write seq 10, it was 93, 233 records per leaf unchanged); container index is sole holder of physical pointers; other trees reference via (birth tree 8 + packed record type 2 + container number 8 + container birth generation 8) |
| D8 (Core Index Structure) settled item 2 / D4 (Checksum Location) settled item 7 / D2 (RAID Stripe Policy) settled item 9 | Node 16384, unit 32768, w = 2 |
| D19 (Block Pointer Structure & Width Budget) settled items 4 / 7 / 8 / 11 / E98 (Geometry of Inode Records and Inode Trees) | Child pointer 86 (header 50 + position entry 14 × 2 + instance generation 4 + birth seq 4; header changed from 47 to 50 on 2026-09-14 following D19 (Block Pointer Structure & Width Budget) settled item 7, the extra 3 bytes are compression algorithm code 1 + compressed length 2); inode record 140, leaf fanout 116 when residing in index leaf, packed 233 records/container; internal fanout is 173 with child pointer 86 (E98 (Geometry of Inode Records and Inode Trees) was 243 with 59) |
| D23 (Journal Role & Format) settled item 12 / E79 (Capacity of Root Record) | Constant portion of each publish: journal record 4096 + root slot 512, identical on both sides |

### Criteria (Locked Before Run, No Modification After Run)

1. **Absolute value of single update**: A single publish modifies only one record, each side calculates "written nodes / units count × size × w", per cell for N ∈ {1e4, 1e6, 1e8}, node header 58 / 67 / 76; tree height calculated separately from leaf fanout and internal fanout tiers.
2. **Batch update**: k ∈ {1, 10, 100, 1000} records dispersed uniformly, nodes touched per level calculated by L × (1 − (1 − 1/L)^k); also report clustered cell where all fall in the same container / same leaf.
3. **Read cost of stat**: Residing index leaves = tree height node reads; packing = inode tree height + container index tree height node reads + 1 unit read of 32 KiB. **Third arm "leaves as containers" (2026-09-05, added after D8 (Core Index Structure) settled item 6 round 1 argumentation)**: Inode tree leaves are code 3 containers, internal nodes are container index, entry = split key 8 + identity reference 26 (birth tree 8 + packed record type 2 + container number 8 + container birth generation 8, same segment as other trees referencing container) + child pointer 86 = 120 (fanout 136 at header 58, 135 from header 65; base header three tiers 58 / 67 / 76 are E73 (Fanout Cost of Node Key Interval) without lower bound of interval, with 16-byte key interval they become 74 / 83 / 92; D8 (Core Index Structure) settled item 6 verbatim today's tier is header 131 with ⌊(16384 − 131) / 120⌋ = 135, dropping to 134 only at header 185; before 2026-09-12 with child pointer 59 it was 93, fanout 175; before 2026-09-14 with 83 it was 117, header 58 fanout 139) ⇒ write = containers touched + internal level nodes touched, read = internal level count + 1 unit read of 32 KiB.
4. **Blast radius**: Follow E98 (Geometry of Inode Records and Inode Trees) measurement basis, one number for each side.
5. **Verdict on prior estimate**: "≈ 3x" verified against criterion 1 single update ratio; holds if ≥ 3, invalidated and truthfully recorded if < 3.

### Failure Clauses (Locked Before Run)

| Clause | Judgment on Trigger |
|---|---|
| Positive control: packing side container takes 16384 and ignores index, must have same cost as leaf level of residing index leaf | Not equal ⇒ two arms not on the same ruler, whole round invalidated |
| Negative control: k = 0 ⇒ structural bytes on both sides exactly 0 | Not 0 ⇒ error on trivial input, whole round invalidated |
| Saturation control: k much greater than nodes in level ⇒ touched nodes converge to node count of that level (diff < 0.5%) | Not converging ⇒ batch model incorrect, whole round invalidated |
| **Reverse acceptance clause**: Single update ratio < 3 | Conclusion is "round 2 forward derivation leg's 3x estimate invalidated", sentence in D8 (Core Index Structure) settled item 6 changed to measured numbers |

### Measurements (2026-09-04, pure arithmetic, N=5 rounds byte-for-byte identical, 13 unit tests / 11 mutants caught)

**Measurement basis and rerun**: Code `scripts/experiment-apparatus/e103_inode_update_cost.rs` (`cargo run --release --bin e103-inode-update-cost`),
artifact `e103-inode-update-cost-2026-09-14-round2.out` (131 lines, ending `emitted=131`; version with child pointer 86, internal entry 120. Differs by **49 lines** from previous version `e103-inode-update-cost-2026-09-12-ptr83.out` (child pointer 83, entry 117): `name=config` 1 line, `name=geom` 12 lines (level count vectors), `name=write` 36 lines; `name=stat`, `name=write_clustered`, blast radius lines **verbatim unchanged**. Prior version `e103-inode-update-cost-2026-09-12.out` was child pointer 59, packing header 107; earlier `e103-inode-update-cost-2026-09-07.out` and `e103-inode-update-cost-2026-09-05.out` also retained untouched; mutation rerun `e103_inode_update_cost-mutate-2026-09-14-round2.log` caught all 11 mutants, prior was `e103-mutate-2026-09-12-ptr83.log`),
mutation table `scripts/mutation/tables/e103_inode_update_cost.tsv`
(`bash scripts/mutation/mutate.sh e103-inode-update-cost scripts/experiment-apparatus/e103_inode_update_cost.rs scripts/mutation/tables/e103_inode_update_cost.tsv`,
log `e103-mutate-2026-09-05.log`), hooked into `scripts/experiment-apparatus/replay.sh`.

**Geometry (header 58, 1e6 inodes)**: Residing index leaf 8621 leaves → 50 → 1 (height 3); packing 4292 containers, container index 30 leaves → 1 (height 2; entry 26 + 86 = 112, fanout 145),
inode tree by identity reference has leaf fanout 466 (entry 8 + 27), height 3.

| Criterion | Numbers (Header 58) |
|---|---|
| 1 Single update | 1e4: 65536 vs 98304 (**1.5x**); 1e6: 98304 vs 131072 (**1.33x**); 1e8: 131072 vs 163840 (**1.25x**). Residing index leaf side is "tree height nodes", packing side is "one 32 KiB container + container index tree height nodes", packing side index tree is one to two levels shorter |
| 2 Batch dispersed | 1e6: k = 100 is 1.58x, k = 1000 is 1.82x (marginal cost per update on packing side tends toward one 32 KiB container, residing index leaf tends toward one 16 KiB leaf); 1e4 k = 1000 enters full run rewrite range, **0.99x** — both sides store the same set of 140-byte records, bytes identical on full rewrite; clustered cell (k records all fall in same container / same leaf) identically equals single update ratio |
| 3 stat read | 1e4: 2 reads vs 4 reads (2.5x bytes); 1e6: 3 reads vs 6 reads (49152 vs 114688, **2.33x**); 1e8: 4 reads vs 8 reads (2.25x). Extra comes from container index tree height node reads plus one 32 KiB unit read |
| 4 Blast radius | 116 vs 233 (E98 (Geometry of Inode Records and Inode Trees) measurement basis unchanged) |
| 5 Prior estimate | "≈ 3x" = (32768 + 16384) / 16384, **only counted leaves**; measured single update 1.25–1.5 ⇒ **invalidated**. Three node header tiers trend identically, ratios all fall within [1.25, 2) |
| Third arm Leaves as containers | Internal levels 1e4 / 1e6 / 1e8 = 1 / 2 / 3 levels ([1] / [32, 1] / [3156, 24, 1]), write k = 1 **identical per cell** with path 1 (98304 / 131072 / 163840); at k ≥ 10 third arm slightly more expensive, extra is internal level nodes exceeding container index ([30, 1]): k = 1000 (saturated cell) difference exactly 2 nodes — artifact line 13 `c_bytes=59548673` vs `b_bytes=59483137` (1e6, difference 65536 = 2 × 16384 × 2, ratio 1.826 vs 1.824); k = 10 / 100 difference 2586 / 54838 bytes (lines 11 / 12), approaching 2 nodes with batch size; stat read = internal levels + 1 unit read: 49152 / 65536 / 81920 bytes, **1.5 / 1.33 / 1.25x** (path 1 is 2.5 / 2.33 / 2.25); blast radius same as path 1 at 233 |

**What it says to D8 (Core Index Structure) settled item 6**: Path 1 in writes is not 3x but 1.25–1.82x (depending on N and batch size),
in reads is 2.25–2.5x bytes, one extra unit read; container index tree is shorter than inode tree, offsetting part of the cost where containers are larger than leaves.
After third arm merges container index into inode tree's own internal nodes, single write is unchanged, batch write is under 1% more (in artifact third arm exceeds path 1 by at most 0.92%, at 1e8 / k = 100 cell; child pointer 59 version was at most 1.33%, the "under 0.2%" written then does not hold on k = 10 / 100 cells), read drops to 1.25–1.5x — the form taken since D8 (Core Index Structure) settled item 6 version 3.
Container count ⌈n/233⌉ is value for newly created tree; when not merged after deletions, fill factor has no lower bound (1e6 sequential creation then deleting 90% without merge = 9.98x space amplification). Extra 16 KiB in single stat multiplies across path depth: naming tree value holds only inode number (D8 (Core Index Structure) settled item 3), each component of path queries inode tree once, d levels deep path pays d extra times.
These numbers only answer cost, not modality or the recovery half (judged in three-way argument).

### What It Cannot Answer

Pure arithmetic: no btree implementation, no I/O, no caching, no crash-point replay, 0 file operations. Does not answer wall clock.
Uniformly dispersed update is a selected scenario point; real workload locality is only given an upper bound by clustered cells; the cost of packing side containers being rewritten once per publish (C105 (write amplification of packed containers rewritten per publish frequency))
is reflected here as "how many containers k updates fall into", without separate modeling.

⚠️ **Rerun on 2026-09-07 under header widths holding that day** (step requested by user before finalizing D18 (Information Carried in a Block) settled item 14).
Before 2026-09-07, the three basic node header tiers in source code were **58 / 67 / 76**, which were lower bounds on the day E73 (Fanout Cost of Node Key Interval)
was run; subsequent two widenings were never reflected in source code, added together here:
write seq 10 (D18 (Information Carried in a Block) settled item 7 supplementary note) + reserved bits 28
(D18 (Information Carried in a Block) settled item 7 verbatim "v1 reserves nonce code and MAC field bits",
12 for nonce code per settled item 14 Arm A and 16 for MAC per D9 (Encryption) settled item 2) ⇒ **96 / 105 / 114**.
Both header sets remain in source code running per cell; the older set is where this conclusion stood before 2026-09-07.
**Conclusion not overturned**, only intermediate quantities changed. Artifact replaced with rerun from same day.

**Intermediate quantities**: Internal fanout 243 → **242**, container index fanout 192 → **191**; while `b_over_a`, tree height, container count **unchanged per cell** — absorbed by upward ceiling of tree height (⌈8621 / 243⌉ = ⌈8621 / 242⌉ = 36). ⚠️ This is not "header width does not affect conclusion", intermediate quantities did change, unit test `geometry_at_today_header_is_absolute` nails it down.

### Decisions Affected

| Decision Item | Relationship | Review |
|---|---|---|
| D8 (Core Index Structure) settled item 6 | Supports | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D18 (Information Carried in a Block) settled item 11 | Does not affect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D19 (Block Pointer Structure & Width Budget) | Does not affect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D2 (RAID Stripe Policy) settled item 9 | Does not affect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D4 (Checksum Location) settled item 7 | Does not affect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D9 (Encryption) settled item 2 | Does not affect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D23 (Journal Role & Format) settled item 1 | Does not affect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |

## Historical Versions

Historical entries for E103 (Update Cost of Inode Packing Path 1) are collected in [experiments-history.md](../experiments-history.md).
