## E95 Aging Node Layout Arm — Run (2026-09-09, Pure Arithmetic, 20 Unit Tests / 20 Mutations All Caught)

[中文](95-老化的节点布局臂.md) · **English** · [日本語](95-老化したノードレイアウトアーム.ja.md)

**Problem**: D26 (Background Compaction and Placement Reclamation) settled item 4 asks "what form structural anti-aging should take", whereas E93 (Placement and Fragmentation Under Aging) only has policy arms (clustered placement, neighbor rewriting, compaction), **lacking any arm that modifies node layout** — therefore "whether policy forms are sufficient" cannot be compared at the model tier, and the byte verdict of settled item 4 (bytes altering new-pool file creation: yes) could not be flipped (2026-09-03 three-way argument, forward leg and backward leg arrived at the same verdict). E95 (Aging Node Layout Arm) is precisely to supplement that arm.

### Model

Inheriting the object / slot model and two workload sets of E93 (Placement and Fragmentation Under Aging), adding two **format-level** candidate arms to compare against policy arms under the identical workload and extra write budget sets:

| Arm | Which Part of Format Modified | Mechanism |
|---|---|---|
| Node group | Node header carries group identity (G key-adjacent 16 KiB nodes constitute a placement and rewrite unit) | Rewriting operates in units of groups: any dirty node in group triggers re-placement of entire group across a segment — explicit manifestation of BetrFS large node effect; node size remains 16 KiB of D8 (Core Index Structure) settled item 2 |
| Generational separation | Node header carries aging generation (tiering checkpoint count since last rewrite) | Placement partitioned by generation: hot and cold do not mix segments, compaction only cleans hot regions — age separation in log-structured cleaning |
| Policy arm (control) | No format changes | bump_compact(B) and bump_neighbor(R) of E93 (Placement and Fragmentation Under Aging) reused as-is |

**The format arm must be given its most favorable implementation** (`.claude/singlefs-ai-sop/rules/evidence-discipline.md` strawman clause): group identity and aging generation are treated as zero-cost readable, and allocator makes optimal use of them.

### Criteria (Hardcoded Before Run 2026-09-03, Unchanged After Run)

1. **Manual calculation anchor**: 8 objects, group size 2: dirtying one object re-places entire group, runs pinned per manual calculation.
2. **Conservation**: Identical to E93 (Placement and Fragmentation Under Aging) (incremental runs == audit, occupied constant equality, defer slots non-reusable).
3. **Positive control passes gate for every arm**: Under forced placement switch, fully isolated layout reports runs == L, contiguous layout reports runs == 1.
4. **Equal budget comparison**: Extra write multiples take three tiers 1.5× / 3× / 5×, each tier and arm reports median runs (5 seeds).
5. **One question to answer**: Can the format arm under its most favorable implementation compress runs to less than half of policy arm on **any** equal-budget cell? Can ⇒ "policy is sufficient" is overthrown, format portion of D26 (Background Compaction and Placement Reclamation) settled item 4 must settle true form; Cannot ⇒ model tier supports "no format changes", adding a link to evidence chain for byte verdict flip (latency axis remains with E10 (Aging: Structural Resistance vs Post-hoc Compaction)).
6. Does not replace E10 (Aging: Structural Resistance vs Post-hoc Compaction); degradation criteria still belong to it.

### Failure Clauses (Hardcoded Before Run)

| Clause | Verdict |
|---|---|
| Criterion 3 misses on any arm | Whole round voided |
| Format arm inferior to policy arm across all cells | **Legitimate result, faithfully recorded** — that is evidence for "no format changes", not implementation breakage |
| Format arm sweeps all cells | **Legitimate result, faithfully recorded** — that is evidence that D26 (Background Compaction and Placement Reclamation) settled item 4 must settle true form |

### Measurement Basis

Code `scripts/experiment-apparatus/e95_node_layout_arms.rs` (`cargo run --release --bin e95-node-layout-arms`, no arguments, single run emits all 86 lines), raw output `e95-node-layout-arms-2026-09-09.out`. Mutation table `scripts/mutation/tables/e95_node_layout_arms.tsv`. Re-run:

```bash
cargo run --release --bin e95-node-layout-arms          # Run under scripts/experiment-apparatus/
bash scripts/experiment-apparatus/replay.sh E95               # Byte-for-byte comparison with admitted artifact
```

Configuration verbatim follows E93 (Placement and Fragmentation Under Aging): `l=8192 s=10240 g=64 d=64 t=2000 seeds=5`.

**Measurement basis of "extra write multiple X" settled during the 2026-09-09 run** (criterion 4 originally only wrote "three tiers 1.5× / 3× / 5×", without specifying how to spend): extra writes allowed per checkpoint = `D × (X − 1)`, all four arms spend the same number round by round, spent amount reported in `budget_spent_pct` column of artifact. 1.5 / 3 / 5 ⇒ extra = 32 / 128 / 256. ⚠️ This basis aligns with E93 (Placement and Fragmentation Under Aging): its bump_cp_b32 / b128 / b256 verbatim reported `write_amp=1.500` / `3.000` / `5.000`.

**Cross-harness gate**: E95 (Aging Node Layout Arm) copied model of E93 (Placement and Fragmentation Under Aging), so assertions pinned 14 `aging_median` values from E93 (Placement and Fragmentation Under Aging) admitted artifact, turning red if mismatched (`.claude/singlefs-ai-sop/rules/show-me-test.md` "trap is in modeling basis, checks must stand across harnesses"). All 14 reproduced one by one without error.

**Deterministic model**: Same binary run four times is byte-identical. Per `.claude/singlefs-ai-sop/rules/test-discipline.md`, this only proves absence of hidden state, **not statistical stability**.

### Results

**Verdict lines copied directly from `e95-node-layout-arms-2026-09-09.out`**:

```
E7RESULT name=verdict budget=32 load=uniform best_policy=7031 best_format=6128 best_format_arm=fmt_group_g2 format_halves_policy=false
E7RESULT name=verdict budget=32 load=runs8 best_policy=5146 best_format=1595 best_format_arm=fmt_group_g8 format_halves_policy=true
E7RESULT name=verdict budget=128 load=uniform best_policy=5463 best_format=3966 best_format_arm=fmt_group_g2 format_halves_policy=false
E7RESULT name=verdict budget=128 load=runs8 best_policy=3922 best_format=904 best_format_arm=fmt_group_g8 format_halves_policy=true
E7RESULT name=verdict budget=256 load=uniform best_policy=4791 best_format=3414 best_format_arm=fmt_group_g2 format_halves_policy=false
E7RESULT name=verdict budget=256 load=runs8 best_policy=1339 best_format=276 best_format_arm=fmt_group_g32 format_halves_policy=true
E7RESULT name=answer any_cell_format_halves_policy=true criterion=E95_5
```

**Literally per criterion 5: Can.** All three runs8 cells hit; none of the three uniform cells hit (1.15 / 1.38 / 1.40×, none reaching 2×).

⚠️ **Yet this "can" does not hold — criterion 5 was not measuring what it was supposed to measure** (2026-09-10 backward leg verified, main agent read source code + **re-ran on scratchpad copy** to substantiate: altering only tag prefix and family label lines while keeping mechanism and RNG source untouched, **48 lines of `runs_median` unchanged word for word, while all six answer lines flipped from `true` to `false`**; probe artifact unarchived). The boundary between "format / policy" in artifact **is a hardcoded label**: `Arm::family()` hardcoded per enum variant (`Arm::family` in `e95_node_layout_arms.rs`), verdict lines filter by tag prefix (`starts_with("pol_")` at `:953` and `starts_with("fmt_")` at `:959`). Backward leg registered the **identical function** `step_node_group` as a policy arm (aligned key-window rewrite, zero on-disk bytes, group size as runtime knob); **numbers across 24 cells remained verbatim identical, yet answer lines flipped on the spot from `true` to `false`, all six cells false**, with 14 cross-harness medians matching perfectly.
=> **The arm judged as "format arm" does not consume any on-disk fields in this model** (verbatim `dirty.iter().map(|&key| key / group_size)` at `:683`), and which family it belongs to depends entirely on what name the submitter assigned to it.

=> **E95 (Aging Node Layout Arm) 2026-09-09 run neither overthrows "policy is sufficient" nor establishes it.**
The only format arm truly reading on-disk state is generational separation (reading `last_write` and `object_generation` at `:726`, `:743`), which won only one of six cells and never halved => **looking only at it, criterion 5 = cannot.**

**Generational separation format arm won 1 of 6 cells**: `budget=256 load=uniform` is verbatim
`name=grid budget=256 arm=fmt_gensep_256_1024 load=uniform runs_median=4347`; best policy arm on same cell is `pol_compact ... runs_median=4791` => 9.3% lower, but far from half; the other five cells all lost, closest being `budget=256 arm=fmt_gensep_256_1024 load=runs8 runs_median=1343` against policy arm 1339 (lost by 4). **It never beat node group arm on any cell.**
Per failure clauses, that is a **legitimate result, faithfully recorded**.
⚠️ This section modified once on 2026-09-10: initial draft wrote "lost every cell" and "tied", both conflicting with artifact (`.claude/singlefs-ai-sop/rules/evidence-discipline.md` "quote entire line when citing artifact", hit by forward leg, confirmed by main agent recalculation).

**Fallback rate** (`fallback_pct`, proportion of allocations unable to enter clustered segments): Node group arm is 0.0 on most cells, policy arms 31.4–88.1. Verbatim example: `budget=128 arm=fmt_group_g2 load=runs8 fallback_pct=0.0` vs `budget=128 arm=pol_compact load=runs8 fallback_pct=65.1`.

⚠️ **Scope rewrite (2026-09-13, after E127 (Whether to Store Group Identity Under Splits and Merges))**: The three `fmt_group_gN` "format arms" in artifact do not consume any on-disk fields — E127 (Whether to Store Group Identity Under Splits and Merges) cross-harness gate reproduced their `runs_median` verbatim across 24 cells under degenerate geometry for both "grouping by key range" and "grouping by node count" arms, so verdict line "format arm compresses runs to less than half of policy arm" reads as "**placement policy grouping by key** compresses runs to less than half of bump_compact"; with split and merge, grouping by node count (requiring fields) loses to grouping by key range (no fields) across all six cells. Generational separation arm truly reading on-disk state never halved on any of six cells per criterion 5 (ratio against best policy arm 1.113 / 1.264 / 1.257 / 1.053 / 0.907 / 1.003, `fmt_gensep_*` taking best threshold per cell); looking only at it, criterion 5 = cannot.

### What It Cannot Answer (Hardcoded Before Run 2026-09-03)

1. Counting model: no latency axis, no message buffer batching statistics, no real machine.
2. Two format arms are candidates, not exhaustive: if other node layout forms are proposed, arms must be added and rerun.
3. How many bytes group identity and aging generation occupy in real format, and whether they compete for space with 665 buffer entries of D11 (Message Buffers in Index Nodes), are not calculated by model.

### What It Cannot Answer (Discovered Only After Run, Three on 2026-09-09 + Three on 2026-09-10)

**These items are scope notes, not criteria**: literal conclusion of criterion 5 is unaffected by them, **but "whether this result can be taken to alter decisions" is**.

4. **The winning mechanism requires no format bits in this model.**
   Groups of `fmt_group_gN` are computed via `key / N` — in model, key-to-object is static 1-to-1 without node split or merge, so allocator knows group identity without looking up any on-disk field. Yet the arm registered in model section is verbatim "node header carries group identity (G key-**adjacent 16 KiB nodes** constitute a placement and rewrite unit)": **grouping by node count** requires storing fields; **grouping by key range** does not.
   => What was measured is "fixed-boundary whole-group re-placement beats sliding window and round-robin compaction", **not** "the group identity field in node header is necessary". Disentangling the two requires a model with split and merge.
5. **The win in runs8 column relates to alignment with group size**: Workload rewrite segment length is verbatim 8, and the arm winning most is `fmt_group_g8`; whereas on uniform (7558 / 5797 / 3669) the same arm is suppressed by `fmt_group_g2`.
   => Group size must be picked according to workload locality; picking wrongly yields no benefit.
6. **Conclusion is non-monotonic on budget axis, and registered three tiers did not cover inflection point.**
   After run, extra = 512 (write_amp 9×) probe was added on scratchpad copy (**artifact unarchived**, not entering criteria, not entering replay): on uniform `pol_compact=1037` overtook `fmt_group_g2=2545`, on runs8 `pol_compact=195` overtook `fmt_group_g8=401` and `fmt_group_g32=230`.
   => **Advantage of format arm falls in 1.5×–5× segment; with larger budget it flips**; inflection point between 5× and 9× was not measured.
   ⚠️ On 2026-09-10 backward leg independently reproduced four numbers of 512 tier (verbatim identical), and extrapolated upward to 1024: `pol_compact` reported 576 / 102, **beating all six format arms**, flipping monotonically deepening with budget.
7. **The three group size sample points (2 / 8 / 32) underestimated node group arm rather than overestimating.**
   Backward leg scanned g ∈ {1,2,4,8,16,32,64} × segment length ∈ {1,2,4,8,16,32}: optimal g **follows budget**, not workload segment length => item 5's assertion "related to alignment with group size" **does not hold in the form written**. Actual gap: true optimum for `budget=128 load=runs8` is g16 = 522, while registered three-point grid reported 904 (1.73× inferior).
8. **`fallback_pct` cannot serve as supporting evidence for win.** (Cited in "Results" section of body)
   Backward leg verified: after aligning fallback counting basis of generational separation arm, fallback rates across 18 cells flipped from 55.7–89.8 to 0.0, while `runs_median` **remained verbatim identical across 18 cells** — because when `run_length_in_slots == 1`, `find_free_run(1)` and `allocate_first_fit` return the identical slot, with only the latter counting. => That column measures "which code path was traversed", not "how good placement is".
9. **Cross-harness gate only covers shared base, cannot cover two newly added arms.**
   `step_node_group` / `step_generation_separation` / `place_generation_run` / `extend_neighbors_budgeted` appear **0 times** in `run_e93_arm` => 14 medians all matching only proves Sim / dirty_set / place_run base shares lineage with E93 (Placement and Fragmentation Under Aging), saying nothing about new arms. `pol_neighbor` is likewise outside gate, and it is precisely the opponent on the decisive `budget=32 load=runs8` cell.

### Affected Decisions

| Decision Item | Relationship | Review |
|---|---|---|
| D26 (Background Compaction and Placement Reclamation) settled item 4 | Support | 2026-10-17 unaffected: this round only synchronized wording, artifact pointers, paths, and numbers, conclusions, numbers, and artifacts unchanged; previously 2026-09-27 unaffected: terminology renaming only touched wording, conclusions, numbers, and artifacts unchanged (prior review 2026-09-20: unaffected: generational separation format arm never halved across six cells per E95 (Aging Node Layout Arm) own criterion 5; its "format / policy" boundary is a hardcoded label, registering identical function as policy arm flips answer lines — combined, two facts provide half the basis for "not entering format"); 2026-10-11 unaffected: only replaced line number references with anchors, conclusions, numbers, and artifacts unchanged |
| D8 (Core Index Structure) settled item 2 | Unaffected | 2026-10-17 unaffected: this round only synchronized wording, artifact pointers, paths, and numbers, conclusions, numbers, and artifacts unchanged; previously 2026-09-27 unaffected: terminology renaming only touched wording, conclusions, numbers, and artifacts unchanged (prior review 2026-09-20: unaffected: node size served only as geometric input, E95 (Aging Node Layout Arm) does not judge it); 2026-10-11 unaffected: only replaced line number references with anchors, conclusions, numbers, and artifacts unchanged |
| D11 (Message Buffers in Index Nodes) | Unaffected | 2026-10-17 unaffected: this round only synchronized wording, artifact pointers, paths, and numbers, conclusions, numbers, and artifacts unchanged; previously 2026-09-27 unaffected: terminology renaming only touched wording, conclusions, numbers, and artifacts unchanged (prior review 2026-09-20: unaffected: E95 (Aging Node Layout Arm) admitted model does not calculate whether group identity and aging generation compete for space with message buffers in real format); 2026-10-11 unaffected: only replaced line number references with anchors, conclusions, numbers, and artifacts unchanged |

## Historical Versions

Historical entries for E95 (Aging Node Layout Arm) are consolidated in [experiments-history.md](../experiments-history.md).

