## E114 Overall Accounting of Small-Data Packing Containers — Run (2026-09-07, pure arithmetic, 14 unit tests / 15 mutants caught)

[中文](114-小数据打包容器的总账.md) · **English** · [日本語](114-小データパッキングコンテナの全体収支.ja.md)

**The question asked**: whether the user-proposed small-data packing container (multiple small objects sharing one 32 KiB unit) can save space,
how much it saves, and the remaining uncalculated quantity after three rounds and nine legs of argumentation —
**whether bytes saved on data side minus bytes extra paid on metadata side is positive, and whether it flips negative under interleaved workloads**.

**Conclusion in one sentence**: Online packing benefits depend entirely on `m` (number of small objects landing in a single publish); when `m = 1` the overall ledger is negative;
**moving packing to background compaction makes benefits independent of `m`**, 52.33x for 512 B tier, 28.40x for 1 KiB tier,
with the cost of 1.7%–3.3% extra write bytes; objects ≥ 16 KiB are pure losses under both forms.

### Four Arms

| Arm | Where Small Objects Reside | Extra Per Slot |
|---|---|---|
| `pad` | Standalone data extent, padding to 32768 (current behavior of D4 (Checksum Location) settled item 2) | — |
| `pack` | Directly enters shared container on write path, each slot carrying full self-describing info | 47 = 5-tuple 33 + write sequence 10 + slot table 4 |
| `packmin` | Directly enters shared container on write path, carrying only slot table entry (**buys no identity, upper bound control, not a candidate**) | 4 |
| `packbg` | **Write path unaltered by a single byte**, objects relocated into containers by resident compaction of D26 (Background Compaction and Placement Reclamation) after writing | 47 |

`packbg` is the fourth arm added midway through execution, added because benefits of `pack` hung on a quantity undefined in this project.

### Cited Items (Pasted Verbatim into Source Comments)

| Item | The Exact Sentence Used |
|---|---|
| D4 (Checksum Location) settled items 5 / 2 | Unit constantly 32768 including header; short extent padding to full unit, "no variable-length unit branch in format" |
| D18 (Information Carried in a Block) settled item 11 | Packed record unit header 107; other trees reference container via 26 bytes (birth tree 8 + type 2 + container number 8 + birth generation 8); **four loading disciplines ② "one container one generation"** |
| D19 (Block Pointer Structure & Width Budget) settled items 4 / 5 | Position entry 14; central mapping key = logical identity 5-tuple 33, value = w copies of position entries |
| D3 (Space Allocation) settled item 7 | Allocation record entry 18, placement granularity 16384 |
| D2 (RAID Stripe Policy) settled item 9 | Version 1 2 disks constantly w = 2 |

### Criteria and Failure Clauses Locked Before Run

1. **Primary criterion**: Total ledger = (`pad` data + metadata) − (`pack` data + metadata), reporting sign and absolute value per cell;
   metadata **counted on both arms** (central mapping, allocation records, container index, live slot accounting), avoiding asymmetric accounting that tripped E66 (How Small Files Land After Padding).
2. **Benefit relation**: Verify "data side benefit = min(`m`, container capacity)".
3. **Overwrite attenuation**: Overwrite ratio `f` sweeps from 0 to 1, reporting break-even point.
4. **Compaction cost**: Bytes of single compaction ÷ bytes saved during workload filling phase.
5. **Fault domain**: One 32 KiB unit unreadable, how many objects lost per arm.

**Failure clauses**: Positive control runs per arm (when container capacity forced to 1, data side of each packing arm must equal `pad` byte-for-byte);
discriminative power control (`pad` data side must be identical on 512 B and 16384 B); negative control (all 0 when N = 0).

### Verdict on Three Controls: All Passed

| Control | Criterion | Measured |
|---|---|---|
| Positive (per arm) | When capacity forced to 1, `pack` / `packbg` data side = `pad` | 5 tiers × 6 `m` = 30 cells all equal |
| Discriminative power | `pad` occupancy identical on 512 B and 16384 B | Both 3 276 800 000 |
| Negative | N = 0 yields all 0 | Data / metadata / write bytes all 0 |

⚠️ **Positive control caught a modeling error on the spot**: `pack` container count was originally calculated as "publish count × containers per publish",
treating final publish with fewer than `m` objects as full, creating 20 extra containers (655 360 bytes) out of thin air at 1 KiB / `m` = 30 cell.
Fixed model, not criterion.

### Criterion 2 Negative Verdict: Analytical Formula Locked Before Run Disproven

Pre-registered `min(m, cap)` gave 30 at 1 KiB / `m` = 100 cell, **measured 25**.
Correct form is `m / ⌈m/cap⌉` — landing 100 objects in single publish opens 4 containers, final container holding only 10.
Discovered formula matches per cell (relative error ≤ 0.0002). **Both formulas kept in source code**, pre-run one not allowed to be retroactively corrected.

### Measurements (N = 100 000, w = 2, f = 0; Ratio = `pad` Ledger ÷ Arm Ledger)

| Object Size | Container Capacity | Online `m`=1 | Online `m`=8 | Online `m`=30 | **Background Compaction (Any `m`)** | Background Compaction Write Ratio |
|---|---|---|---|---|---|---|
| 512 B | 58 | **0.998× (negative)** | 7.88× | 28.40× | **52.33×** | 1.0172 |
| 1 KiB | 30 | **0.998× (negative)** | 7.88× | 28.40× | **28.40×** | 1.0333 |
| 4 KiB | 7 | **0.998× (negative)** | 3.97× | 5.93× | **6.91×** | 1.1429 |
| 8 KiB | 3 | **0.998× (negative)** | 2.65× | 2.98× | **2.98×** | 1.3333 |
| 16 KiB | 1 | 0.998× (negative) | 1.00× | 1.00× | **1.00× (no benefit)** | 2.0000 |

⚠️ **Background compaction column is conditional; citing it must include this note** (verified in round 2 argumentation on 2026-09-07, three points):

1. **Metadata writes were uncounted by a single byte.** `write_bytes` only counted data units (source `e114_pack_ledger.rs` in `pack_bg_ledger`
   `let write_bytes = (n + containers + overwritten) * UNIT * W;`), `meta_bytes` was resident occupancy not writes.
   During relocation central mapping entries undergo **second overwrite**, while `pad` writes once ⇒ omission asymmetric across arms.
   When patched (central mapping leaf 16384 per D8 (Core Index Structure) settled item 2, entry 61 bytes ⇒ 267 entries per leaf, total 375 leaves):
   when compaction is ordered by container fill and batch B = 1, 512 B / 1 KiB / 4 KiB tiers **all yield negative net benefits**
   (−53.9 MiB / −104.2 MiB / −446.4 MiB, main agent and forward leg independently calculated identical values);
   turning positive requires raising batch beyond 13–15 / 25–27 / 119–120 (critical points differed by 1–2 between parties due to rounding basis, ununified).
   When compaction is ordered by mapping key order, total cost drops to 375 leaves ≈ 11.72 MiB, negligible.
   ⇒ **Benefit multiple is conditioned on D27 (Small-Data Packing Container) settled item 6 (compaction trigger criteria and batching)**; settled: packing proceeds along central mapping key order, precisely satisfying this condition.
2. **`packbg` arm was outside scope of all failure clauses locked before run.** Three reverse acceptance clauses verbatim referred to "Form A",
   while `packbg` was fourth arm added midway through ⇒ arm producing final settlement had no falsifiable clauses,
   violating `.claude/singlefs-ai-sop/rules/test-discipline.md` "experiment failure clauses must not leave conclusion unfalsifiable".
3. **Container count ⌈N/cap⌉ quietly assumed D27 (Small-Data Packing Container) settled item 8 answer is "adopt zero loading disciplines on data side".**
   Line 164 source comment noted only consequence ("independent of m"), without stating which rule it bypassed.
   ⚠️ But **not** in violation of D18 (Information Carried in a Block) settled item 11: discipline ② includes self-scope "generalized to packed record types reclaimed by generation",
   explicitly exempting type 2; and disciplines ①③ were settled verbatim by I-1.7 (packing container validity and judgment order) ("container header declares one packed record type, one birth tree").

**Online form write bytes**: At 1 KiB / `m` = 30 it is 218 497 024 bytes, while `pad` is 6 553 600 000 ⇒ 30x savings.
**Background compaction write bytes**: Extra cost is 1/container capacity — 512 B tier 1.7%, 1 KiB tier 3.3%, 16 KiB tier full 2x (written twice in vain).

**Overwrite break-even point** (`f` = proportion of objects overwritten once, without compaction):

| Object Size | `m`=8 | `m`=30 |
|---|---|---|
| 512 B / 1 KiB | 0.90 | 1.00 |
| 4 KiB | 0.75 | 0.85 |
| 16 KiB | 0.00 | 0.00 |

**Compaction cost**: At 1 KiB / `m` = 30 / `f` = 0.3 single compaction takes 218 497 024 bytes, while same cell saves 2 186 678 084 bytes ⇒ consumes 10%.
**Fault domain**: `pad` loses 1 object, online form loses min(container capacity, `m`), background compaction loses one full container capacity (30 objects in 1 KiB tier).

### Verdict on Three Reverse Acceptance Clauses

| Clause (Locked Before Run) | Verdict |
|---|---|
| Negative ledger on cells where `m` ≥ 8 ⇒ Form A yields no savings, proposal abandoned | **Triggered verbatim**: 16 KiB all cells negative, 512 B / `m`=8 / `f`=0.9 net = −82 885 000. ⚠️ Clause did not clarify "any cell" vs "all cells", flaw in drafting, retrospective changes forbidden ⇒ both readings presented: triggered verbatim, un-triggered on primary cells where `f`=0 and object ≤ 8 KiB |
| Benefit < 1.94x ⇒ inferior to Form B | Triggered when `m` ≤ 2 (Form B 1.88–1.94x, calculated 2026-09-07) |
| Single compaction ≥ savings during fill phase | **Not triggered** (consumes 10%) |

### What It Says to the Proposal

1. **Benefit denominator of online packing (original form in user proposal) does not exist in this project.** Benefit = `m / ⌈m/cap⌉`,
   while `m` must be defined by target workload, and D25 (Target Workload Priority) settled "one fsync carries 8 leaves" — which are index leaves not small object count,
   and when this experiment ran "small file" appeared 0 times in D25 (Target Workload Priority) (subsequently D25 (Target Workload Priority) added smallfile row, three columns unmeasured, C174 (Three columns of smallfile row unmeasured)).
2. **Background compaction form replaces the denominator**: container count = ⌈N/container capacity⌉, independent of publish rhythm, benefits constant.
   Cost is objects written to disk twice, and **zero space saved before compaction finishes** (peak occupancy = `pad`).
3. **Where boundary should be drawn is quantified**: 8 KiB tier retains 2.98x, 16 KiB tier is pure loss (written twice in vain, zero savings).
4. Solves obstacle in `.claude/rules/fs-design.md` Hard Requirement 5: background form criterion is
   "how large this **already-written** object is", branch variable non-existent during operation. **This is reasoning; E114 (Overall Accounting of Small-Data Packing Containers) did not measure it.**

### What It Cannot Answer

Pure arithmetic ledger: **no implementation, no I/O, no crash-point replay**, zero file operations. Does not answer wall clock, cache, concurrency.
Workload self-defined by E114 (Overall Accounting of Small-Data Packing Containers), **not within definition domain of D25 (Target Workload Priority)**.
Only counts bytes of "single compaction", not how often it runs (packing startup is continuous sweep with rate limiting per D27 (Small-Data Packing Container) settled item 6, idling overhead unmeasured, C202 (idle overhead and hysteresis K of packing patrol lack values)).
**Compaction read bytes** of background compaction form (N × 32768) exist separately, uncounted in write ratio.
Variable-length slot allocation and fragmentation within containers, nonce / MAC carriers for slots, container index placements, all unmodeled.
**Metadata write bytes uncounted on both arms** (see item 1 under measurement table) — `meta_bytes` is resident occupancy, not writes.
**Three format constants lack source citations**, while source comment header verbatim states "format constants all have citations":
`SLOT_FULL = 47`'s 4-byte slot table entry, `SLOT_MIN = 4`, `LIVE_COUNT_ENTRY = 16`;
the last directly corresponds to D27 (Small-Data Packing Container) settled item 5 (where per-container live count resides), which was unsettled when experiment ran ⇒ assumption rather than constant; settled item 5 later settled on no live count, this metadata non-existent in final form.

### Measurement Basis and Artifacts

Raw output `e114-pack-ledger-2026-09-12.out` (229 lines, ending line `name=done emitted=229`; container header 107 version, differs from previous version `e114-pack-ledger-2026-09-07.out` in results/ only in config lines; mutation rerun `e114-mutate-2026-09-12.log`).
Harness `scripts/experiment-apparatus/e114_pack_ledger.rs`, mutation table `scripts/mutation/tables/e114_pack_ledger.tsv` (all 15 caught; from 2026-09-12 onward: prior to that three original lines matched twice in source and never ran green; after adding context targeted only first arm, supplemented three for `packbg` arm, where container index and live count accounting were true blind spots, overwrite was sample insensitive, `packbg_absolute` pinned metadata 6 453 404 bytes and unit count for f = 0.3).
N = 100 000, w = 2, unit 32768, container header 107, full self-describing 47 bytes per slot; `m` takes 1 / 2 / 8 / 30 / 100 / 1000,
object size takes 512 / 1024 / 4096 / 8192 / 16384 B, overwrite ratio `f` takes 0 / 0.1 / 0.3 / 0.5 / 0.9.

```bash
bash scripts/experiment-apparatus/replay.sh E114     # Byte-for-byte comparison against e114-pack-ledger-2026-09-12.out
```

### Decisions Affected

| Decision Item | Relationship | Review |
|---|---|---|
| D27 (Small-Data Packing Container) settled item 1 | Supports | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D27 (Small-Data Packing Container) settled item 2 | Supports | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D27 (Small-Data Packing Container) settled item 6 | Preparation | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D27 (Small-Data Packing Container) settled item 5 | Does not affect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D27 (Small-Data Packing Container) settled item 8 | Does not affect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D25 (Target Workload Priority) | Preparation | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D4 (Checksum Location) settled item 2 | Does not affect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D18 (Information Carried in a Block) settled item 11 | Does not affect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D19 (Block Pointer Structure & Width Budget) settled item 4 | Does not affect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D3 (Space Allocation) settled item 7 | Does not affect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D2 (RAID Stripe Policy) settled item 9 | Does not affect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D26 (Background Compaction and Placement Reclamation) | Does not affect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D8 (Core Index Structure) settled item 2 | Does not affect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |

## Historical Versions

Historical entries for E114 (Overall Accounting of Small-Data Packing Containers) are collected in [experiments-history.md](../experiments-history.md).

