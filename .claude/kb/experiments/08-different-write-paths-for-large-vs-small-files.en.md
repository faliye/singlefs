## E8 Different Write Paths for Large vs Small Files — Tier A Run (2026-08-28), Tier B Not Run

[中文](08-大文件与小文件是否该走不同写路径.md) · **English** · [日本語](08-大容量と小容量ファイルで書き込みパスを分けるべきか.ja.md)

**Conclusions**:

1. **The crossover point is not some natural scale of file size, but the inline threshold itself.**
   Gains are entirely concentrated in tiers ≤ the threshold (taken as 3584 B in that round): at 64 B and 1 KiB, diversion versus non-diversion
   saves 2.9–94.6× write amplification; **at 4 KiB and above, all four arms are cell-by-cell identical (1.00)**, with zero gain in any tier.
2. **The crossover point is insensitive to pointer width.** Widening pointers from 256 bits to 320 bits,
   crossover point shifts across four write patterns × seven size tiers are all **within ±0.5%** (maximum −0.48%).
   ⇒ **The diversion criteria of E8 (Different Write Paths for Large vs Small Files) need not wait for [decisions.md](../decisions.md) D19 (Block Pointer Structure & Width Budget) to settle.**

   ⚠️ **In that round, four pointer arms were actually swept, not two** (`ptr=` four values in artifacts).
   The earlier "insensitive" statement referred to **crossover point location**; **device read count is not insensitive**:

   | Pointer Arm | Width | Device Reads at 1 KiB / Single-shot Write / Non-diversion | Total Table Read Count |
   |---|---|---|---|
   | `ptr256` | 32 B | 47 | 5803 (Baseline) |
   | `ptr320` | 40 B | 49 | 5932 (1.022×) |
   | `ptr536mirror` | 67 B | 55 | 6301 (1.086×) |
   | `ptr888stripe` | **111 B** | **67 (+42.6%)** | 7000 (**1.206×**) |

   ⇒ **The "+42.6% device reads for 32 B → 111 B" cited in [decisions.md](../decisions.md) D19 (Block Pointer Structure & Width Budget) is precisely this cell.**
   ⚠️ It **is not the steady-state cost of wide pointers**: the node cache in E8 (Different Write Paths for Large vs Small Files) was intentionally set extremely small
   (tree of several thousand nodes, cache of 256), which was the experimental condition to make structural differences visible, noted on-site in D19 (Block Pointer Structure & Width Budget).
3. **Append writes cross the threshold mid-operation, hence the threshold is not a qualified branch variable.**
   When a 4 KiB file is appended in 8 increments, cumulative size in the first 7 increments is below the threshold,
   so the inline arm still captures a 6.96× gain—whereas the same 4 KiB file written in a single shot gains exactly 1.00.
   **Same file size, two write patterns, criteria landing on opposite sides of the threshold.**
   ⇒ Directly answers E8 (Different Write Paths for Large vs Small Files) Criterion 3: `.claude/rules/fs-design.md` Hard Requirement 5
   "Branch variables must not change mid-operation" **is violated by this branch**, unless a cross-threshold migration path is separately established and its crash semantics verified independently.
4. **The cost of inlining lies on the read side, and it is substantial.** The 1 KiB tier inlining inflates each inode record to approximately 1152 bytes,
   collapsing the records accommodated per leaf, causing leaf count to skyrocket: device read count surges from 47 to **293 (6.2×)**,
   metadata bytes per file rise from 692 to 1644.
   ⇒ **Write amplification and read count move in opposite directions at this tier**; looking solely at write amplification will choose wrong.
5. **Message-layer arms are uniformly inferior to inline arms on the write side**, because each message must be rewritten at every level before landing in a leaf:
   pushdown bytes account for 20.6% (single-shot write 64 B) to **91.0%** (random small write 1 KiB) of its total metadata.
   In the 4 KiB append tier, it is even worse than non-diversion (0.98×).

**Measurement Basis**: Pure accounting experiment, no device, no VM, no file system. Four arms **share the same ledger and the same index tree**,
differing only in "where data is placed, what records are inserted". Node 16 KiB / block 4 KiB / key 16 B /
inline threshold 3584 B / checkpoint batching 64 / node cache 256. Deterministic, without random seed dependency
(random small write uses fixed seed 7). Code `scripts/experiment-apparatus/e8_split.rs`, `cargo run --release --bin e8-split`,
raw output `e8-split-2026-08-28.out`.

**Why not placed in VM**: Write amplification and metadata bytes per file are arithmetic on format;
placing into a VM would only add noise to deterministic figures. End-to-end throughput/latency is a Tier B matter.

### Two Controls, Both Self-proven to Fail Red

| Control | Assertion | Self-test |
|---|---|---|
| **Positive** | 4096 64 B files on non-diversion arm data bytes == 4096 × 4096 (whole-block allocation cost truly paid), inline arm data bytes == 0 | Changing whole-block rounding to per-byte counting immediately flips `pos_ok` to false and aborts the entire round (measured) |
| **Negative** | 1 GiB file cannot reach inline threshold; inline arm and non-diversion arm `device_bytes_written` / `cache_miss_read_count` / `node_writes` are **bit-for-bit identical** | Changing threshold check to constant true turns this assertion red (measured) |

⚠️ **Positive control asserts mechanism only, not "how many times inline wins".**
That multiple varies with checkpoint batch size (dropping below 2× at batch=1);
writing it into a control would cause a legitimate result to be misjudged as a broken model.

All six unit tests passed directional corruption proving they fail red (2026-09-03 mutation table `scripts/mutation/tables/e8_split.tsv`
catches all five): breaking whole-block rounding / fanout ignoring pointer width / threshold check failure / ledger adding extra exit,
each turning the corresponding test red; an added test pins the inline boundary (≤ changed to < originally had no test landing on 3584).

### A Modeling Flaw Fixed in the 08-28 Tier A Round

In the initial round, `inline` and `msgbuf` columns were **cell-by-cell completely identical**—the message layer exhibited zero behavior distinct from inlining in the model;
that was not a discovery, but a defect. Fix: message insertion does not touch target leaves (this is what the message layer buys),
and during checkpoint, messages are sorted by key and pushed down, accounting for pushdown rewritten bytes by `payload × tree_depth` (this is the cost it pays).
**Benefits and costs modeled in pairs; missing either is bias.**

### A Cell Unmeasurable in Tier A

**The read-side benefit of the message layer in E8 (Different Write Paths for Large vs Small Files) could not be measured.** Its `cache_miss_read_count` across all combinations of cache 4–512 nodes
and batching 1–64 was ≥ inline arm (at cache=4, 621 versus 293).
The mechanism is that this model determines leaf assignment by "sort rank / records per leaf", so batch landing of messages shifts leaf mapping en masse—
**this is a modeling artifact, not a property of the message layer**.
⇒ Readouts in this column do not count; read-side gains of the message layer are left to Tier B.

### Tier B (Not Run)

Conduct end-to-end I/O counts only on tiers identified by Tier A as having crossovers (≤ 4 KiB), across two media types.
**Prerequisites**: `scripts/devices/vm-bench.sh` exists; cross-threshold migration awaits file system and crash-point replay.

### Impacted Decisions

| Decision Item | Relationship | Review |
|---|---|---|
| D14 (Dual-Track: Large/Small Files & Persistent/Transient) Settled Item 3 | Material | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D14 (Dual-Track: Large/Small Files & Persistent/Transient) Settled Item 6 | Support | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D19 (Block Pointer Structure & Width Budget) Settled Item 1 | Material | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |

## Historical Versions

Historical entries for E8 (Different Write Paths for Large vs Small Files) are gathered in [experiments-history.md](../experiments-history.md).
