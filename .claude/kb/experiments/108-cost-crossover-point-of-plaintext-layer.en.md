## E108 Cost Crossover Point of Plaintext Layer — Run (2026-09-06, pure arithmetic, 7 unit tests / 6 mutants caught): **Logical key (mapping table) and physical key (issued table) have different cost shapes, crossing over on "how long the disk-locking cycle is" — at 16 TiB / `w = 2` it is 333 578 publishes ≈ 19 days (`w = 4` 46 days, 256 TiB 257 days): disk-locking cycle longer than this makes physical keys more economical; shorter makes logical keys more economical. Mapping table resides permanently at 2582 ppm (5164 at `w = 4`), writing 163 840 extra bytes per publish (hot path); three issued tables 12.72 GiB, single lock + unlock 54.65 GB (cold path, 0 at runtime)**

[中文](108-明文层的代价交叉点.md) · **English** · [日本語](108-平文層のコスト交差点.ja.md)

**This answers the user-conditional finalization in D9 (Encryption) settled item 9**: "If physical keys offer high cost-performance and efficiency then adopt physical keys; otherwise adopt logical keys."
The costs of the two paths have **different shapes** and cannot be compared by a single number:

| Path | Resident Space | Per Publish | Per Lock / Unlock |
|---|---|---|---|
| Logical key (mapping table) | Entry 47 bytes × identity count × `w` | **Must maintain** (modify row whenever identity's placement changes) | 0 (table always present) |
| Physical key (issued table) | 0 (table exists only during locked period) | **0** (table derived from key-side structures) | Write out / read back three tables |

### Cited Items (Copied Verbatim into Source Comments)

| Item | The Exact Sentence Used |
|---|---|
| D19 (Block Pointer Structure & Width Budget) settled item 4 | Position entry 14 = device 4 + 16 KiB slot number 6 + ciphertext checksum 4; mapping table entry = logical identity 32 + replica sequence 1 + 14 |
| D18 (Information Carried in a Block) settled item 9 / settled item 11 | Slot 16384, data unit 32768, code 3 container header 107 |
| D16 (Publish Semantics) settled item 5 | `T_time = 5 s` ⇒ publish rate |
| D25 (Target Workload Priority) | One fsync carries 8 leaves |
| D2 (RAID Stripe Policy) settled item 6 / settled item 10 | `w ∈ {2,3,4}`, position entry one per replica ⇒ both sides must multiply by `w` |

### Criteria (Locked Before Run, Unchanged After Run)

1. Three issued tables calculated by geometry, nailing absolute values per table; live slot table takes **bitmap** (1 bit per slot), not one entry per slot.
2. Extra bytes written per publish for mapping table calculated by "containers touched + container index tree path", same algorithm as E107 (Stripe Table Write Amplification on Primary Workload).
3. **Crossover point**: Solve for `N` such that `N ×` bytes per publish `=` bytes for one lock + one unlock; report absolute value and converted days.
4. **Negative control**: Capacity → 0 yields → 0 on both sides.
5. **Positive control**: Three table widths zeroed ⇒ crossover point at 0 publishes.
6. **Capability difference not counted in cost**: Logical key cannot relocate code 2 / code 3 (identities enter ciphertext); that is coverage rather than cost, E108 (Cost Crossover Point of Plaintext Layer) reports without counting.

### Measurements (2026-09-06, artifact `e108-plaintext-layer-cost-2026-09-06.out`, 9 lines, `emitted=9`)

| Capacity | `w` | Three Issued Tables | Single Lock+Unlock | Mapping Table Resident | ppm | Per Publish | **Crossover Point** |
|---|---|---|---|---|---|---|---|
| 1 TiB | 2 | 853.9 MB | 3.42 GB | 2.84 GB | 2582 | 163 840 | 20 848 publishes ≈ **1 day** |
| **16 TiB** | **2** | **12.72 GiB** | **54.65 GB** | **45.42 GB** | **2582** | **163 840** | **333 578 publishes ≈ 19 days** |
| 16 TiB | 4 | 30.72 GiB | 263.9 GB | 90.84 GB | 5164 | 327 680 | 805 437 publishes ≈ **46 days** |
| 256 TiB | 2 | 203.6 GiB | 874.5 GB | 726.7 GB | 2582 | 196 608 | 4 447 709 publishes ≈ **257 days** |

**How to read this table**: The crossover point is "how long the disk-locking cycle must be for both paths to cost the same".
**Longer than this ⇒ physical keys more economical** (volumes online year-round, locking once amortized over months); **shorter than this ⇒ logical keys more economical** (laptops locked daily).
Larger capacity, larger `w`, pushes crossover point further back ⇒ **larger pools favor physical keys**.

⚠️ **Two factors not in this table that decide trade-offs**:
1. **Logical keys cannot relocate code 2 / code 3** (their identities enter ciphertext, keys unexpressible), while mechanistic rationale in D9 (Encryption) settled item 5 verbatim states the bottleneck **is metadata**
   ⇒ It loses on coverage first; cost comparison is meaningful only under assumption "it can be repaired".
2. **Costs fall on different paths**: The 163 840 bytes of logical keys are hot-path write amplification **per publish**,
   while the 54.65 GB of physical keys is a one-time cold-path cost **per lock**. Same byte count, amortized differently.

### Mutation Testing: 6 Mutants Caught

"Live slot table calculated as one entry per slot", "member table does not scale with width", "locking writes only does not read", "mapping table misses replica dimension", "per publish misses tree path", "fill rate treated as full".

### Assertions Nailing Absolute Values

`signed_tables_are_pinned` (slots 1 073 741 824, bitmap 134 217 728, live slots 966 367 641, three tables total 13 663 364 702),
`map_costs_are_pinned` (identities 483 183 820, 695 entries per container, containers 1 390 458, internal fanout 191, tree height 4, per publish 163 840),
`crossover_is_pinned` (333 578 publishes, 19 days), `member_snapshot_scales_with_width` (`w=4` is 3x of `w=2`).

### What It Does Not Answer

Wall clock; real locking frequency (**crossover point is a ruler, not a conclusion** — applying it to specific machine requires knowing how often that machine locks);
how many authoritative pointers unlock replay must modify (tier of E94 (Touch Surface of Moving Shared Placement Points), zero measurement in this project);
what verifies the three tables themselves (original question in D9 (Encryption) settled item 9, **unanswered** by physical key path);
mapping table tree internal node overhead only calculated along single path, splits not modeled.

### Rerun

```bash
bash scripts/experiment-apparatus/replay.sh E108
cd scripts/experiment-apparatus && bash ../mutation/mutate.sh e108-plaintext-layer-cost \
  scripts/experiment-apparatus/e108_plaintext_layer_cost.rs mutations/e108_plaintext_layer_cost.tsv
```

### Decisions Affected

| Decision Item | Relationship | Review |
|---|---|---|
| D9 (Encryption) settled item 9 | Preparation | 2026-09-20 unaffected: E108 (Cost Crossover Point of Plaintext Layer) quantified the user-conditional finalization before its withdrawal "adopt physical keys if cost-performance is high, otherwise logical keys" into numbers (16 TiB / `w = 2` crosses over at 19 days); after settled item 5 changed to only scrub, that item was withdrawn, its scope verbatim stating that round's numbers are archived with no further consumers; 2026-10-11 unaffected: script path migration only (`../mutate.sh` moved to `../mutation/mutate.sh`), conclusions, numbers, and artifact unchanged |
| D2 (RAID Stripe Policy) | Does not affect | 2026-09-20 unaffected: input only — `w ∈ {2,3,4}` and "position entry one per replica" taken from settled items 6 / 10, multiplied by same `w` on both sides; 2026-10-11 unaffected: script path migration only (`../mutate.sh` moved to `../mutation/mutate.sh`), conclusions, numbers, and artifact unchanged |
| D16 (Publish Semantics) | Does not affect | 2026-09-20 unaffected: input only — `T_time = 5 s` in settled item 5 only used to convert crossover publishes to days; 2026-10-11 unaffected: script path migration only (`../mutate.sh` moved to `../mutation/mutate.sh`), conclusions, numbers, and artifact unchanged |
| D18 (Information Carried in a Block) | Does not affect | 2026-09-20 unaffected: input only — slot 16384, data unit 32768, code 3 container header 107 taken from settled items 9 / 11, geometry of three issued tables and mapping table calculated per them; 2026-10-11 unaffected: script path migration only (`../mutate.sh` moved to `../mutation/mutate.sh`), conclusions, numbers, and artifact unchanged |
| D19 (Block Pointer Structure & Width Budget) | Does not affect | 2026-09-20 unaffected: input only — position entry 14 bytes (device 4 + slot 6 + ciphertext checksum 4) taken from settled item 4, mapping table entry width derived from it plus logical identity 32 and replica sequence 1; 2026-10-11 unaffected: script path migration only (`../mutate.sh` moved to `../mutation/mutate.sh`), conclusions, numbers, and artifact unchanged |
| D25 (Target Workload Priority) settled item 1 | Does not affect | 2026-09-20 unaffected: input only — coarse-grained finalization of one fsync carrying 8 leaves fixed how many containers touched per publish; 2026-10-11 unaffected: script path migration only (`../mutate.sh` moved to `../mutation/mutate.sh`), conclusions, numbers, and artifact unchanged |

## Historical Versions

None (initial output).

