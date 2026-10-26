## E89 Nested Interval Labeling and Deletion Frontier — Run (2026-09-03, Pure Arithmetic, 13 Unit Tests / 11 Mutations All Caught): All Three Criteria Met——Candidate Arm Erroneous Release and Leaked Free Constantly 0 Across All Cells, Reverse Lookup Constantly 0 (Baseline 8->252 Scaling with N in Same Cells), Degenerates to D5 (Snapshot / Space Accounting Mechanism) Constant Form Under Linear; Frontier Upper Bound Scales with Branch Count

[中文](89-嵌套区间标号与删除前沿.md) · **English** · [日本語](89-入れ子区間ラベリングと削除フロンティア.ja.md)

**It is the modeling verification singled out by C35 (Multi-Writable-Head O(N) Unoptimized)**:
After D6 (Snapshot Implementation Model) adopted multi-writable heads, deleting a snapshot incurred O(N) reverse index lookups
(measured in E26 (Lookup Cost and Accuracy of Three Accounting Models) as 213 lookups at 8192 extents, log-log slope 1.32).
D6 (Snapshot Implementation Model) text left an optimization awaiting testing——nested interval labeling + deletion frontier——can it bring this back to O(branch count)?

**The prerequisite "tree or DAG" has been resolved**: D6 (Snapshot Implementation Model) settled item 1 settled that each head has its own tree,
cloned from a single origin snapshot; no settled decision contains an operation merging two heads
⇒ The genealogy of snapshots and writable heads is a **tree**, where nested interval labeling (preorder intervals) applies, avoiding the difficulty tier of DAGs.

### What to Test

Counting model (pure arithmetic, zero I/O), world generation shares identical semantics with E26 (Lookup Cost and Accuracy of Three Accounting Models):
snapshot tree (Linear / MultiHead topologies), each snapshot creating eps extents,
child snapshots inheriting extents referenced by parent per sharing rate. The only difference is: **accounting on the spot when inheritance fails to occur**——
when a child does not inherit an extent referenced by parent, record child into that extent's **deletion frontier**
(available incrementally at commit time, matching "accounting is a transaction byproduct"; requiring no post-hoc scanning).

Deleting the intermediate snapshot, both arms judge "which extents can be freed", each compared against independent ground truth:

| Arm | Mechanism | Accounting Basis |
|---|---|---|
| `paylookup` (real baseline) | D6 (Snapshot Implementation Model) active Option B: isomorphic to deadlist arm of E26 (Lookup Cost and Accuracy of Three Accounting Models)——falls back to reverse index lookup on lateral reference, +1 each time | Reverse index lookup count |
| `frontier` (tested candidate) | Each extent stores (birth node, deletion frontier); active snapshot set maintains a table independent of extent count via preorder labeling; "any other active references remaining" = active snapshots in birth subtree - Σ active snapshots in frontier subtrees, pure interval arithmetic | Reverse index lookups constantly 0; snapshot table probes and frontier sizes reported separately |

**Ground truth = enumerate all active snapshots; an extent referenced by any active snapshot is active** (same as E26 (Lookup Cost and Accuracy of Three Accounting Models), sharing zero code with both arms).
The candidate arm's function signature **cannot obtain the refs set** (machine-first: using types to make illegal states unrepresentable)——
it receives only each extent's (birth node, frontier), failing compilation if attempting to copy ground truth answers.

### Accounting Measurement Basis (Written Hard Before Run)

- **"Lookup" = reverse index query**, sharing the measurement basis with E26 (Lookup Cost and Accuracy of Three Accounting Models): scale of reverse index grows with total extents,
  querying it on real disk involves a btree descent.
- **Querying snapshot table is not counted as a lookup**: snapshot table size = snapshot count (order of branch count), resident in memory——
  equivalent to free reading of snapshot txg and alive bits in deadlist arm of E26 (Lookup Cost and Accuracy of Three Accounting Models). However, candidate arm snapshot table probe counts are **reported separately**,
  not hidden within free items.
- Enumerating "which extents were referenced by deleted snapshot" is free for both arms: D6 (Snapshot Implementation Model) settled item 1 gives each head its own tree,
  and deleting a head enumerates its own tree.

### Criteria (Written Hard Before Run, Derived from C35 (Multi-Writable-Head O(N) Unoptimized), Neither May Be Omitted)

1. **Both erroneous release and leaked free must be 0** (against identical ground truth): Candidate arm across all cells
   (topology x sharing rate x scale x fine-grained deletion tiers) exhibits `wrong_free == 0 && leaked == 0`.
   Any cell missed ⇒ candidate invalid, faithfully record, C35 (Multi-Writable-Head O(N) Unoptimized) remains owed, implementation proceeds with paid lookups.
2. **Lookup count does not scale with total extents, only scaling with branch count**: Candidate arm reverse index lookups **constantly 0** (nailing absolute values,
   asserted on every scale tier); checks per extent = 1 + frontier size, where frontier is an antichain of snapshot tree nodes
   ⇒ upper bound is snapshot count, independent of total extents (asserting `maximum_frontier <= snapshot_count`; frontier sizes faithfully reported per tier).
   Baseline arm in identical cell must reproduce growth of E26 (Lookup Cost and Accuracy of Three Accounting Models) (scale x16 yields lookups > 4x)——serving as positive control,
   proving the measurement distinguishes "scaling with N" from "not scaling with N"; failing ⇒ flawed implementation, entire round invalidated.
3. **Degenerates to zero-cost form when branch count is 1**: Under Linear topology candidate arm lookups constantly 0 with frontier <= 1 per extent
   ⇒ matching precisely the constant form of D5 (Snapshot / Space Accounting Mechanism) "comparing txg with previous snapshot". Failing ⇒ candidate invalid.

Two additional structural self-proofs (failing invalidates entire round as implementation bug, not conclusion):

- **Frontier must be an antichain**: Any two frontier nodes possess no ancestor relationship; and visible set recomputed from (birth node, frontier)
  must equal generated refs extent by extent——accounting and ground truth yielding identical answers for identical world.
- **Positive controls run on every arm**: Under Linear + zero sharing, both arms must match ground truth completely.

### Failure Clauses (Written Hard Before Run)

- Criteria 1 or 3 missed: **faithfully record candidate invalid**, altering ground truth, world generation, or swapping cells forbidden.
- Criterion 2 positive control missed, or structural self-proof missed: **implementation flawed, entire round invalidated.**
- Deterministic model: 5 rounds of byte-identical md5 proves absence of hidden state, not claiming statistical stability
  (`.claude/singlefs-ai-sop/rules/test-discipline.md`).

### Measurement Basis and Known Limitations

- One **deliberate** difference from E26 (Lookup Cost and Accuracy of Three Accounting Models): Topology and sharing use independent RNG streams (derived from same seed),
  preserving "tree shape" across scale scans——Criterion 2 requires scanning extent counts over fixed trees.
- Frontier size depends on deletion pattern (Limit 1 of D6 (Snapshot Implementation Model) pending optimization): fine-grained deletion tiers (low sharing rate) scanned separately,
  with frontier maximum and mean reported faithfully, reporting only advantageous tiers forbidden.
- **Label maintenance outside scope** (Limit 3 of D6 (Snapshot Implementation Model) pending optimization): model numbers tree only after construction;
  as tree grows, preorder labels require maintenance via order-maintenance / list-labeling, for which mature algorithms exist but remain unverified in this project.
  Before citing E89 (Nested Interval Labeling and Deletion Frontier) as basis for implementation, this layer requires isolated verification.
- Real cost of one snapshot table probe unmeasured (matching E26 (Lookup Cost and Accuracy of Three Accounting Models) where "real I/O cost of one reverse lookup" was likewise unmeasured);
  difference between arms is a **categorical difference** (querying O(extent count) structure vs querying O(snapshot count) table), not a ratio difference.
- Prerequisites: None. Pure arithmetic.

### Empirical Measurements (2026-09-03)

**All three criteria fully hit.**

Criterion 1 (Accuracy against independent ground truth): Candidate arm across all cells——two topologies x sharing rates
{0,10,30,70,90} x three scale tiers x fine-grained deletion tiers——exhibits `wrong_free == 0 && leaked == 0`,
matching accuracy of baseline arm which "pays lookups to achieve zero leak and zero erroneous release".

Criterion 2 (Who scales with N): Fixing identical 64-node tree, scanning total extents from 256 to 8192 (x32):

| Total Extents | Baseline Reverse Lookups | Candidate Reverse Lookups | Candidate Snapshot Table Probes | Candidate Frontier Maximum |
|---|---|---|---|---|
| 256 | 8 | **0** | 58 | 12 |
| 512 | 18 | **0** | 118 | 12 |
| 1024 | 27 | **0** | 172 | 10 |
| 2048 | 68 | **0** | 376 | 16 |
| 4096 | 121 | **0** | 778 | 15 |
| 8192 | **252** | **0** | 1601 | **16** |

Baseline reproduced growth of E26 (Lookup Cost and Accuracy of Three Accounting Models) (positive control: x32 scale yielded x31.5 lookups),
while candidate reverse lookups remained constantly 0; frontier maximum hovered between 12-16, not tracking N.
On the branch scanning side (snapshots 16->128, extents fixed): frontier maximum 5->22,
snapshot table size = snapshot count——**the scaling dimension is branch count, precisely the direction sought by C35 (Multi-Writable-Head O(N) Unoptimized)**.

Criterion 3 (Degeneration): Linear across all sharing tiers shows frontier <= 1, lookups constantly 0——
falling back precisely to constant form of D5 (Snapshot / Space Accounting Mechanism) "comparing txg with previous snapshot".

Fine-grained deletion tier (faithfully reporting Limit 1): at share=10, global frontier maximum was 10, mean 0.967 per extent;
even the finest tier never exceeded antichain upper bound of snapshot count. Two structural self-proofs (frontier antichain,
visible set recomputed from frontier matching generated refs extent by extent) enforced before each measurement, all green.

**Interpretation and limits**: Candidate arm per-extent evaluation pays 1 + frontier size snapshot table probes,
total probes still tracking "extents referenced by deleted snapshot" (unavoidable in any per-extent evaluation,
read in passing when enumerating own tree upon deleting head); categorical difference lies in **structure touched per probe**——
snapshot table is O(snapshot count) resident in memory, whereas reverse index is O(total extents) on-disk btree.
Label maintenance (order-maintenance during tree growth) is outside scope, requiring isolated verification prior to implementation.

### Measurement Basis and Replay

- Code `scripts/experiment-apparatus/e89_interval_frontier.rs`
  (`cargo run --release --bin e89-interval-frontier`), artifact
  `e89-interval-frontier-2026-09-03.out` (28 lines, closing `emitted=28`).
  Replay passes E89 (Nested Interval Labeling and Deletion Frontier) identifier to `scripts/experiment-apparatus/replay.sh`, byte-for-byte comparison.
- **All 13 unit tests green; all 11 mutations caught** (`scripts/mutation/tables/e89_interval_frontier.tsv`),
  covering six facets: inclusion-exclusion frontier subtraction, failing to exclude victim, interval endpoint open modification, sharing inversion, ground truth treating dead snapshot as alive,
  and baseline omitting lateral lookup charges.
- Deterministic model: 5 rounds of byte-identical md5 (proving absence of hidden state, not claiming statistical stability).

### Affected Decisions

| Decision Branch | Relationship | Review |
|---|---|---|
| D6 (Snapshot Implementation Model) settled item 10 | Supports | 2026-09-20 Unaffected: all three criteria met——candidate arm erroneous release and leaked free constantly 0 across all cells, reverse lookups constantly 0 while paid-lookup baseline scales with size, Linear topology degenerates to D5 (Snapshot / Space Accounting Mechanism) constant form; frontier upper bound tracks branch count, finest deletion tier never exceeds antichain upper bound |
| D6 (Snapshot Implementation Model) settled item 1 | Prepares material | 2026-09-20 Unaffected: "tree or DAG" resolved by each head having own tree, E89 (Nested Interval Labeling and Deletion Frontier) models only trees without touching DAG complexity |
| D5 (Snapshot / Space Accounting Mechanism) | Prepares material | 2026-09-20 Unaffected: "comparing txg with previous snapshot" is interval labeling on chains, E89 (Nested Interval Labeling and Deletion Frontier) establishes its generalization onto trees |

## Historical Versions

### 2026-09-03
- Established and completed on same day. Source: check form text of C35 (Multi-Writable-Head O(N) Unoptimized) and
  D6 (Snapshot Implementation Model) "an optimization awaiting testing"; three criteria copied verbatim from C35 (Multi-Writable-Head O(N) Unoptimized), written hard before run.
  Before run baseline was predicted 0 lookups in hand-built small world, but measured 1——hand calculation missed lateral reference of C,
  and pinned value was corrected to 1 per reality (mechanism itself is "lateral falls back to lookup", reflecting correct implementation and flawed manual arithmetic).
