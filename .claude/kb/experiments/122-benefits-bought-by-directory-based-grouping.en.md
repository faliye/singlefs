## E122 Benefits Bought by Directory-Based Grouping —— Run (2026-09-07, pure counting model, 10 unit tests / 12 mutations caught)

[中文](122-按目录聚在一起买到什么.md) · **English** · [日本語](122-ディレクトリ集約で何が得られるか.ja.md)

<!-- doc-lint:not-numbers M1 M2 M3 M4 M5 M6 M7 M8 M9 M10 M11 M12 -->

**What is asked**: the gain in the row "allocator (placing sub-directory data near parent directory)" within D8 (Core Index Structure) settled item 3 table.
The sole justification for that row is tagged verbatim "⚠️ **Unverified** —— no provenance or measurement basis in kb, must be supplemented before use",
and after three-way arguments on D14 (Dual-Track: Large/Small Files & Persistent/Transient) settled item 2, user settled "**supplement gain data first before deciding**".

**Conclusion in one sentence**: **Gains in both cells hinge on degree of interleaving** —— consecutive run count of `arrival` = total objects ÷ interleaving stride,
when `stride = files_per_directory` (no interleaving), placement sequences of both arms are cell-by-cell identical, **hence both traversal and reclamation metrics are also cell-by-cell identical**.
Only on **interleaved** workloads does `grouped` buy anything: deleting a directory when `stride ≤ 8`,
`grouped` frees up entire segments (2 at segment length 32 slots, 1 at 64 slots, 0 at 128 slots),
whereas `arrival` and `random` cannot free up a single one.

### Clauses on Which the Model Is Built

| Clause | Sentence Used |
|---|---|
| D14 (Dual-Track: Large/Small Files & Persistent/Transient) settled item 3 + D4 (Checksum Location) settled item 2 | Inline threshold 0, small files with padding to 32 KiB ⇒ **each small file occupies an entire unit** ⇒ grouping alters only adjacency **between** units. ⚠️ **Measurement basis (narrowed 2026-09-09)**: D27 (Small-Data Packing Container) settled item 1 / 2 settled that objects ≤ 4 KiB are migrated into shared containers by background compaction after writing ⇒ **in steady state, ≤ 4 KiB tier is not "each occupying an entire unit"**; model of E122 (Benefits Bought by Directory-Based Grouping) holds only for objects **> 4 KiB**, and for objects ≤ 4 KiB during the period **before being migrated by compaction** (duration determined by hysteresis `K` in D27 (Small-Data Packing Container) settled item 6, where `K` lacks numbers, see C202 (idle overhead and hysteresis K of packing patrol lack values)) |
| D3 (Space Allocation) settled item 7 | Placement granularity 16384 ⇒ one 32 KiB data unit occupies **2 slots** |
| D3 (Space Allocation) settled item 5 | Clustered segment is "runtime allocator policy not format bit... no disk field declares it" ⇒ **segment length is parameter, not format constant**, sweeping 32 / 64 / 128 slots |

### Three Arms

| Arm | How Placement Is Determined |
|---|---|
| `grouped` | Objects under same parent directory are contiguous on disk |
| `arrival` | In order of creation arrival; interleaving between directories set by `stride` (place several continuously per directory before switching to next) |
| `random` | Deterministic LCG pseudorandom. **Discriminative power lower bound, not a candidate** |

### Actual Run (16 Directories × 32 Files per Directory = 512 Objects)

| `stride` | `grouped` Total Runs | `arrival` Total Runs | `random` Total Runs |
|---|---|---|---|
| 1 (full round-robin) | 16 | **512** | 484 |
| 2 | 16 | 256 | 484 |
| 4 | 16 | 128 | 484 |
| 8 | 16 | 64 | 484 |
| 32 (= files per directory, no interleaving) | 16 | **16** | 484 |

⇒ **Closed form**: `arrival` total runs = 512 ÷ `stride` (pinned cell-by-cell by source `walk_runs_are_pinned_to_absolute_values`, asserting `runs × stride ≡ 512`).

**Fully emptied segments freed after deleting one directory** (⚠️ **this cell also carries `stride` measurement basis**,
in artifact every row of `arrival` carries `stride`, which this table once stripped and restored 2026-09-09):

| Segment Length (slots) | `grouped` (all `stride`) | `arrival`, `stride ≤ 8` | `arrival`, `stride = 32` (no interleaving) | `random` |
|---|---|---|---|---|
| 32 | **2** | 0 | **2** | 0 |
| 64 | **1** | 0 | **1** | 0 |
| 128 | 0 | 0 | 0 | 0 |

⇒ **Without interleaving, `arrival` and `grouped` are cell-by-cell identical on reclamation as well**, artifact line by line:

```
E7RESULT name=walk arm=grouped stride=32 total_runs=16 worst_dir_runs=1 empty_segs_at_32=2 empty_segs_at_64=1 empty_segs_at_128=0
E7RESULT name=walk arm=arrival stride=32 total_runs=16 worst_dir_runs=1 empty_segs_at_32=2 empty_segs_at_64=1 empty_segs_at_128=0
```

This is identical to the positive control row (at `stride = 32` placement sequences of both arms are **completely identical**) ——
with identical placement sequences, any quantity computed from placements is necessarily identical.
Assertion scope in apparatus was always correct (`empty_segments_are_pinned_to_absolute_values` hardcoded `stride = 2`,
assertion message verbatim "arrival **after interleaving** cannot empty an entire segment (segment length {sg})"),
**what stripped the measurement basis was the prose text**.

### Judgment of Three Controls

| Control | Criterion | Actual Run |
|---|---|---|
| Positive (arm by arm) | Without interleaving `arrival` equals `grouped` cell by cell | At `stride = 32` placement sequences of both arms **completely identical** |
| Negative | 0 files per directory | All three arms give 0 total runs |
| Discriminative power | Three arms differ on at least one cell | `stride = 2`: 16 / 256 / 484 |

### One Parameter Labeled Inversely Before Run, Retained in Archive Unaltered

Criterion 3 registered that "when `stride = 1`, `arrival` must equal `grouped` cell by cell",
whereas semantics of `stride` is "how many continuously per directory before switching to next" ⇒ **`stride = 1` is full round-robin, the most heavily interleaved cell**,
`stride = files_per_directory` is the one without interleaving. The **intent** of the criterion did not change; what was mistaken was the label attached to the parameter by main agent during registration.
Both values remain in source `positive_control_on_every_arm_without_interleave`.

### One True Blind Spot and One Sampling Point Insensitivity Forced by Mutation Testing

| Mutation | Triaging | Disposal |
|---|---|---|
| `M11_swap_directory_and_file_counts` (altered scale in `main`) | **True blind spot**: unit tests hardcoded 16 / 32 individually, nothing pinning the parameter set actually used by artifact | Elevated scale to constants `DIRECTORY_COUNT` / `FILES_PER_DIRECTORY` and pinned in `geometry_and_workload_constants_are_pinned` |
| `M10_random_no_dedup` (short-circuited deduplication to constant true) | **Sampling point insensitivity**: LCG modulus is 2⁶⁴, and 512 is power of 2 ⇒ `x % 512` traverses full period without collision, rendering deduplication dead logic | Added non-power-of-2 sampling point (15 × 32 = 480, colliding 172 times on the spot), see `random_deduplication_needs_a_non_power_of_two_size` |

### What It Says Regarding D14 (Dual-Track: Large/Small Files & Persistent/Transient) Settled Item 2

1. **Traversal gain is not "grouping vs non-grouping", but "interleaved vs non-interleaved"**:
   when workload already finishes writing one directory before moving to next, allocator achieves identical result doing nothing.
   ⇒ Gain in this cell depends on **degree of interleaving in target workload**, a dimension missing from workload table of D25 (Target Workload Priority).
2. **Reclamation gain is also "interleaved vs non-interleaved", not "exclusive to grouping"** (corrected 2026-09-09).
   Before 2026-09-09 this clause claimed "reclamation gain can only be bought by grouping", whereas in the same preserved artifact,
   `empty_segs` for `arm=arrival stride=32` is 2 / 1 / 0, identical cell by cell to `grouped`
   ⇒ original phrasing contradicted artifact, corrected per artifact. Its magnitude also depends on ratio between segment length and directory scale ——
   when segment is 128 slots and directory occupies only 64 slots, gain drops to zero.
   ⚠️ **This contradiction was invisible to gate**: replay of E122 (Benefits Bought by Directory-Based Grouping) matches preserved artifact byte by byte and stays completely green (re-verified 2026-09-09),
   what diverged was **prose text and artifact**, not artifact and artifact.
   Recorded as [checks-owed.md](../checks-owed.md) C233 (Exclusive Gain Named by a Conclusion Has a Counterexample in Its Own Artifact).
3. Both cells **exclude** "suspected short-lived" half: that cell appears 0 times across entire text of D25 (Target Workload Priority), being blank.
4. **Combined**: E122 (Benefits Bought by Directory-Based Grouping) proves that "**on interleaved workloads**, grouping by directory is superior to arrival order in both traversal and reclamation metrics",
   **not** that "grouping by directory has an intrinsic positive gain independent of workload".
   D25 (Target Workload Priority) lacks interleaving dimension (`grep -n "交错"` currently yields 0 hits, 2026-09-09),
   ⚠️ but this statement **cannot be read as "there is no interleaved workload in D25 (Target Workload Priority)"** ——
   its workload table contains the `multistream concurrent multi-stream` row weighted "secondary", which is a substantially interleaved workload.

### What It Cannot Answer

Pure counting model: no real I/O, no seek time, no page cache, no crash-point replay.
"Consecutive runs" is a **proxy metric not wall clock** —— assuming one sequential read is cheaper than two, how much cheaper is unanswered here.
Does not model directory tree shape (only one level), does not model file size distribution (each object constant one unit).
Deterministic model, **running N rounds demonstrates absence of hidden state, not statistical stability**.

### Measurement Basis and Artifact

Raw output `e122-dir-locality-2026-09-07.out` (20 lines, ending line `name=done emitted=20`).
Apparatus `scripts/experiment-apparatus/e122_directory_locality.rs`,
mutation table `scripts/mutation/tables/e122_directory_locality.tsv` (all 12 caught).

```bash
bash scripts/experiment-apparatus/replay.sh E122     # Byte-for-byte comparison with e122-dir-locality-2026-09-07.out
```

### Influenced Decisions

| Decision Item | Relationship | Review |
|---|---|---|
| D14 (Dual-Track: Large/Small Files & Persistent/Transient) settled item 2 | Raw material | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D14 (Dual-Track: Large/Small Files & Persistent/Transient) settled item 3 | No effect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D8 (Core Index Structure) settled item 3 | Raw material | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D4 (Checksum Location) settled item 2 | No effect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D27 (Small-Data Packing Container) settled item 1 | No effect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D3 (Space Allocation) settled item 7 | No effect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D3 (Space Allocation) settled item 5 | No effect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D25 (Target Workload Priority) | No effect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |

## Historical Versions

Historical entries for E122 (Benefits Bought by Directory-Based Grouping) are consolidated in [experiments-history.md](../experiments-history.md).
