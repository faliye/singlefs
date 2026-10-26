## E71 Entry Count and Width of Accounting Keys — Run (2026-09-01, Pure Arithmetic, 12 Unit Tests / All 8 Mutations Caught; Run 1 Re-run on 2026-10-20 Recomputed Criterion 2 with Each Root Holding Only One Generation): **What Stretches to Bursting First Is the Product `Tree Count × Device Count`, and Only Under the "Dimensions Fully Utilized" Interpretation**

[中文](71-记账key的条目数与宽度.md) · **English** · [日本語](71-記帳キーのエントリ数と幅.ja.md)

<!-- doc-lint:not-numbers Q3 FC1 FC2 FC3 FC4 FC5 V1 V2 V3 V4 S1 S2 S3 EV1 EV2 EV3 EV4 EV5 EV6 EV7 EV8 EV9 EV10 -->

⚠️ **Criterion 2 has been re-run once per "each root holds only one generation, write-only without deletion" (2026-10-20)**; results reside in section "Run 1 Re-run (2026-10-20): Criterion 2 per Each Root Holding Only One Generation, Write-Only Without Deletion"; the subsection "Criterion 2: Accounting Write Bytes per Checkpoint vs 1% of `T_dirty`" on this page presents numbers under the legacy measurement basis (one write + one delete × 2).

D5 (Snapshot / Space Accounting Mechanism) settled item 1 settled key format as `(statistic, tree_id, device, generation)`.
The formula `key count = statistic count × K` previously given by E54 (How Many Generations to Retain in Accounting) **lacked the two parts for dimensional combinations**.
Absolute values could not be calculated previously because statistic count had not yet been settled; after D5 (Snapshot / Space Accounting Mechanism) settled item 4 (user settlement)
settled **nine statistics**, it became calculable.

⚠️ **Range Note (2026-09-03): E71 (Entry Count and Width of Accounting Keys) ran with s = 9, whereas D5 (Snapshot / Space Accounting Mechanism) settled item 4
reopened for the first time on the 3rd of that month, expanding list to eleven.** Binary and artifact remain completely untouched (`replay.sh` requires byte-for-byte comparison);
recalculating per s = 11 is arithmetic, not a re-run: upper bound arm `s × t × d × K` in the t=64 / d=8 / K=25 cell changes from **115 200 to 140 800**;
narrow interpretation arm changes from **4075 to 4300** (neither newly added item is split by tree — fragmentation runs lacks dimensions ⇒ +K=25,
fully empty cluster extent count carries device dimension ⇒ +d×K=200). **Neither conclusion changes**: what stretches to bursting first remains product `tree count × device count`,
with upper bound overestimation factor relative to narrow interpretation shifting from 28.3 to 32.7.

### Criteria (Hardcoded Before Run, Unchanged After Run)

1. **Absolute value assertion**: Entry count must exactly equal `s × t × d × K`, forbidden to perform only inter-arm comparison.
2. Accounting write bytes for a single checkpoint ≤ 1% of `T_dirty` from D16 (Publish Semantics) settled item 5,
   otherwise judge that "accounting itself consumes the dirty data budget".
3. Four independent variables each monotonic: doubling any one must exactly double the entry count.

### Failure Clauses (Hardcoded Before Run)

| Clause | Verdict |
|---|---|
| **Positive control**: Dropping device dimension, entry count must **drop by factor of d**; failure to drop indicates dimension never entered key, invalidating entire round | **Passed** (upper bound arm 115200 → 14400, exactly factor of 8) |
| If true value of s unobtainable, results must carry measurement basis "calculated per s = N", forbidden to write as absolute conclusion | **No longer applicable**: s = 9 settled |

### ⚠️ Formula Stated in Criteria 1 and 3 Is an Upper Bound, Not Entry Count

`s × t × d × K` multiplies all nine statistics **indiscriminately** by tree and device dimensions,
whereas D5 (Snapshot / Space Accounting Mechanism) settled item 4 verbatim stated that "**key dimension tuple is not fully utilized by every statistic**;
items not partitioned by tree use a reserved tree ID (taken as 0) as 'whole-pool / unassigned' row".

Criteria were drafted **prior to** settling D5 (Snapshot / Space Accounting Mechanism) settled item 4
(body of E71 (Entry Count and Width of Accounting Keys) verbatim stated at the time "absolute values cannot be calculated today")
⇒ **Upper bound arm runs as original, with criteria 1 / 3 and positive control all resting on it without changing a word**;
additionally, two arms accounting for dimensions per item were added to measure the delta.

### ⚠️ One Thing Unsettled by D5 (Snapshot / Space Accounting Mechanism) Settled Item 4, E71 (Entry Count and Width of Accounting Keys) Does Not Settle on Its Behalf

The table **did not list** "which statistics **carry tree dimension**" — it only listed whether device dimension was carried.
⇒ Ran one arm for each of two interpretations, **without picking one as answer**:

| Interpretation | Items Partitioned by Tree | Items Carrying Device Dimension (D5 (Snapshot / Space Accounting Mechanism) settled item 4 verbatim) |
|---|---|---|
| **Narrow interpretation** | Only items 8 (per-tree exclusive) and 9 (per-tree shared) | Items 1, 2, 3, 5 (4 items in total) |
| **Wide interpretation** | All nine items | Items 1, 2, 3, 5 (4 items in total) |

### Measurement

#### Entry Counts Across Three Arms (t=64 Trees, d=8 Disks, K=25)

| Arm | Entry Count for One Generation | Total Entry Count (× K) | Relative to Upper Bound |
|---|---|---|---|
| **Upper bound (dimensions fully utilized)** | 4608 | **115 200** | 1.00× |
| Wide interpretation (9 items by tree) | 2368 | 59 200 | 0.51× |
| **Narrow interpretation (2 items by tree)** | **163** | **4 075** | **0.035×** |

⇒ **Upper bound is 28× larger than narrow interpretation.** Difference originates entirely from the "dimensions fully utilized" reading,
which was verbatim rejected by D5 (Snapshot / Space Accounting Mechanism) settled item 4
⇒ **Using `s × t × d × K` to estimate accounting tree size overestimates by one to two orders of magnitude.**

#### Criterion 3: Doubling Experiment

| Arm | Variable | Baseline | Doubled | Exactly 2× |
|---|---|---|---|---|
| Upper bound | t / d / K | 115 200 | 230 400 | **Yes (all three variables)** |
| **Narrow interpretation** | t | 4075 | 7275 | **No, 1.785×** |

⇒ **Criterion 3 holds on upper bound arm, fails on real keys**, with the latter being the actual target for production landing.
**This is a result, not a bug**: only 2/9 items partition by tree; doubling trees amplifies only those 2 items.

#### Positive Control: Dropping Device Dimension

| Arm | With Device Dimension | Dropped | Factor |
|---|---|---|---|
| **Upper bound** | 115 200 | 14 400 | **Exactly 8× (= d) ✅** |
| Wide interpretation | 59 200 | 14 400 | 4.11× |
| Narrow interpretation | 4075 | 3375 | 1.21× |

⇒ Positive control **holds exactly** on the arm it was designed to verify, proving device dimension indeed entered key;
the other two arms drop less precisely because only 4/9 items carry device dimension.

#### Criterion 2: Accounting Write Bytes per Checkpoint vs 1% of `T_dirty`

⚠️ This subsection reflects legacy measurement basis from 2026-09-01 (one write + one delete, write factor 2). New basis (factor 1) after D5 (Snapshot / Space Accounting Mechanism) settled item 2 changed to each root holding only one generation appears in "Run 1 Re-run (2026-10-20): Criterion 2 per Each Root Holding Only One Generation, Write-Only Without Deletion", with both bases juxtaposed cell-by-cell in the artifact of that section.

`T_dirty` = 2 GiB (D16 (Publish Semantics) settled item 5) ⇒ budget line **21 474 836 bytes**.
A single checkpoint touches all entries of **one generation**, writing each once and deleting each once
(D5 (Snapshot / Space Accounting Mechanism) settled item 2 verbatim: write count changes from `statistic count` to `statistic count × 2`) ⇒ K does not enter this formula.

**Out of 90 cells, 9 exceeded budget, all on upper bound arm; narrow interpretation arm exceeded zero cells.**

| Cell | Upper Bound Arm | Narrow Interpretation Arm | Budget |
|---|---|---|---|
| t=1024, d=64, key+val=30 B | **35 389 440 (1.65× over)** | 138 420 | 21 474 836 |
| t=8192, d=8, key+val=30 B | **35 389 440 (1.65× over)** | — | 21 474 836 |
| t=8192, d=64, key+val=50 B | **471 859 200 (22× over)** | — | 21 474 836 |
| t=1024, d=64, key+val=22 B | **25 952 256 (1.21× over)** | — | 21 474 836 |

⇒ **What stretches to bursting first is not any single independent variable, but product `t × d`**:
the two overflow points (1024×64 and 8192×8) **both have product 65 536**, with byte counts verbatim identical.
⇒ Bursting condition is `t × d ≳ 65536` (under 30-byte width, upper bound interpretation).

⚠️ **K is not in this condition** — it amplifies only total tree size, not write volume of a single checkpoint.
⇒ **K concerned by E54 (How Many Generations to Retain in Accounting) and quantity concerned by Criterion 2 are two distinct matters, forbidden to conflate into one.**

#### Range of K (D5 (Snapshot / Space Accounting Mechanism) Settled Item 2 + D22 (Synthesizing Unit Atomicity) Settled Item 2)

`K = total root ring slots + 1 = R × S + 1`, R = 3, S ranges between 1..16:

| Slots per Region S | Total Slots | Conservative K |
|---|---|---|
| 1 | 3 | 4 |
| 8 | 24 | 25 |
| 16 | 48 | 49 |

Worst-case rollback measured on real devices in E53 (Can Root Ring Mount After Losing an Entire Disk) is 1 ⇒ alternative basis **K = 2**.
The two bases differ by 24.5×, yet **K only linearly amplifies total entry count**, leaving Criterion 2 unaffected.

### Run 1 Re-run (2026-10-20): Criterion 2 per Each Root Holding Only One Generation, Write-Only Without Deletion

**Status: Run (Sufficient to Judge)**, pure arithmetic, byte-for-byte identical across 5 rounds. Pre-registered in `research/prompts/e71-r1-prereg.md` (Q3 in issue ledger `research/prompts/c693-forks.md`, the E71 (Entry Count and Width of Accounting Keys) half; owed check C693 (Arguments to redo after K generations was withdrawn)).

**Only one measurement basis modified**: accounting write bytes per publish = write count factor × single-generation entry count × entry width, with factor changing from 2 (one write + one delete) to 1 — D5 (Snapshot / Space Accounting Mechanism) settled item 2 settled "accounting tree of each root holds only rows for that root's generation, not retaining multiple generations", scope "each publish writes all rows of that generation once". Left unchanged per original harness: the 2026-09-01 nine-statistic table (4 items carrying device dimension, narrow interpretation partitioning only 2 items by tree, wide interpretation partitioning all 9 by tree), three width tiers 22 / 30 / 50 bytes, t ∈ {1, 8, 64, 1024, 8192}, d ∈ {1, 8, 64}, threshold 21 474 836 bytes (1% of `T_dirty` = 2 GiB from D16 (Publish Semantics) settled item 5, strictly greater judged as exceeding budget). **Numbers in this section consistently reflect "the 2026-09-01 nine-item table, new row-count basis"**: D5 (Snapshot / Space Accounting Mechanism) settled item 4 currently uses twelve items, which was not swapped this time.

Artifact `research/results/e71-accounting-keys-single-generation-2026-10-20.out` (302 lines, ending with line `E7RESULT name=done emitted=302`). One line `name=publish_budget` per cell: four width tiers (original three tiers plus 34 bytes) × 15 pairs of (t, d) × three arms = 180 lines, juxtaposing legacy basis bytes, new basis bytes, both verdicts, and permille of threshold consumed under new basis.

**Judgement cell (upper bound arm, 30 bytes, t=1024, d=64) does not exceed budget under new basis** (line 288 of artifact):

```
E7RESULT name=judgement_cell arm=upper_bound width=middle trees=1024 devices=64 old_bytes=35389440 old_over=1 new_bytes=17694720 new_over=0 budget=21474836
```

New basis consumes 17 694 720 bytes, accounting for 823‰ of threshold. Registered failure clause FC1 ("statement in D5 (Snapshot / Space Accounting Mechanism) settled item 4 basis that 'at 1024 × 64 ... exceeds 1% of `T_dirty`' does not hold on E71 (Entry Count and Width of Accounting Keys) side") **triggered**.

**Over-budget cell counts per arm** (lines 289–294 of artifact; `registered_widths` refers to original three tiers across 45 cells, `current_implementation_width` refers to 15 cells of 34 bytes):

```
E7RESULT name=over_budget_count arm=upper_bound widths=registered_widths cells=45 old_over=9 new_over=5 new_over_cells=narrow:8192:64,middle:8192:64,wide:1024:64,wide:8192:8,wide:8192:64
E7RESULT name=over_budget_count arm=upper_bound widths=current_implementation_width cells=15 old_over=3 new_over=1 new_over_cells=current_implementation:8192:64
E7RESULT name=over_budget_count arm=per_statistic_narrow widths=registered_widths cells=45 old_over=0 new_over=0 new_over_cells=none
E7RESULT name=over_budget_count arm=per_statistic_narrow widths=current_implementation_width cells=15 old_over=0 new_over=0 new_over_cells=none
E7RESULT name=over_budget_count arm=per_statistic_wide widths=registered_widths cells=45 old_over=5 new_over=3 new_over_cells=narrow:8192:64,middle:8192:64,wide:8192:64
E7RESULT name=over_budget_count arm=per_statistic_wide widths=current_implementation_width cells=15 old_over=1 new_over=1 new_over_cells=current_implementation:8192:64
```

| Arm | Legacy Basis Over-Budget out of Original 45 Cells | New Basis Over-Budget | Reading |
|---|---|---|---|
| Upper bound (dimensions fully utilized) | 9 | 5 | Over-budget cells under new basis concentrate at large t × d end: 30-byte tier retains only 8192 × 64, with both 1024 × 64 and 8192 × 8 at 17 694 720, within budget; 50-byte tier has 1024 × 64, 8192 × 8, 8192 × 64 exceeding budget. FC2 (zero over-budget cells) not triggered. Original page claim "bursting condition `t × d ≳ 65536` (30 bytes)" fails under new basis: only t × d = 524 288 exceeds budget among 30-byte samples (no samples between 65 536 and 524 288, flip point unmeasured) |
| Narrow interpretation | 0 | 0 | Exceeds zero cells under both bases. Tightest cell under new basis is 50 bytes, 8192 × 64: `new_bytes=832150`, `new_permille_of_budget=38` (line 144 of artifact). FC3 not triggered |
| Wide interpretation (incidental, original Criterion 2 lacked this arm) | 5 | 3 | Exceeds budget in 8192 × 64 cell across three widths; 30-byte 1024 × 64 cell consumes 8 017 920 (373‰), within budget. D5 (Snapshot / Space Accounting Mechanism) settled item 4 basis wording "wide interpretation arm" lacks support in 1024 × 64 cell; FC5 (zero over-budget cells in wide interpretation arm) not triggered |

**Geometry Sensitivity** (lines 295–300 of artifact, swapping entry width and t × d neighboring cells for upper bound judgement cell):

```
E7RESULT name=geometry_sensitivity point=narrower_entry_width registered_label=S1 arm=upper_bound width=narrow entry_bytes=22 trees=1024 devices=64 new_bytes=12976128 new_over=0 same_new_verdict_as_main_point=1
E7RESULT name=geometry_sensitivity point=main_entry_width registered_label=main arm=upper_bound width=middle entry_bytes=30 trees=1024 devices=64 new_bytes=17694720 new_over=0 same_new_verdict_as_main_point=1
E7RESULT name=geometry_sensitivity point=current_implementation_entry_width registered_label=S2 arm=upper_bound width=current_implementation entry_bytes=34 trees=1024 devices=64 new_bytes=20054016 new_over=0 same_new_verdict_as_main_point=1
E7RESULT name=geometry_sensitivity point=wider_entry_width registered_label=S3 arm=upper_bound width=wide entry_bytes=50 trees=1024 devices=64 new_bytes=29491200 new_over=1 same_new_verdict_as_main_point=0
E7RESULT name=geometry_sensitivity point=neighbor_fewer_devices registered_label=trees_times_devices_neighbor arm=upper_bound width=middle entry_bytes=30 trees=1024 devices=8 new_bytes=2211840 new_over=0 same_new_verdict_as_main_point=1
E7RESULT name=geometry_sensitivity point=neighbor_more_trees registered_label=trees_times_devices_neighbor arm=upper_bound width=middle entry_bytes=30 trees=8192 devices=64 new_bytes=141557760 new_over=1 same_new_verdict_as_main_point=0
```

Verdict for judgement cell **depends on entry width** (FC4 triggered): 22, 30, 34 bytes within budget (34 bytes at 20 054 016, 933‰), 50 bytes exceeds budget, flip point between 34 and 50 bytes. Discriminative power self-test in unit test `moving_the_budget_across_the_judgement_cell_flips_its_verdict`: threshold 17 694 720 is within budget, threshold 17 694 719 exceeds budget.

**Positive Control and Invalidation Clauses** (lines 281–287, 301 of artifact):

```
E7RESULT name=positive_control_write_factor arm=upper_bound cells=60 old_equals_twice_new=60
E7RESULT name=positive_control_write_factor arm=per_statistic_narrow cells=60 old_equals_twice_new=60
E7RESULT name=positive_control_write_factor arm=per_statistic_wide cells=60 old_equals_twice_new=60
E7RESULT name=positive_control_drop_device arm=upper_bound trees=64 devices=8 width=middle with_device_new_bytes=138240 without_device_new_bytes=17280 ratio_x1000=8000 exactly_by_device_count=1 holds=1
E7RESULT name=positive_control_drop_device arm=per_statistic_narrow trees=64 devices=8 width=middle with_device_new_bytes=4890 without_device_new_bytes=4050 ratio_x1000=1207 exactly_by_device_count=0 holds=1
E7RESULT name=positive_control_drop_device arm=per_statistic_wide trees=64 devices=8 width=middle with_device_new_bytes=71040 without_device_new_bytes=17280 ratio_x1000=4111 exactly_by_device_count=0 holds=1
E7RESULT name=arm_ordering groups=60 upper_at_least_wide_at_least_narrow=60
E7RESULT name=verdict cells=180 positive_control_write_factor_holds=true positive_control_drop_device_holds=true arm_ordering_holds=true
```

| Invalidation Clause | Verification Method | Outcome |
|---|---|---|
| V1 Legacy basis reproduces original artifact | 90 lines of `name=ckpt_budget` reprinted by new harness diffed against original artifact `e71-accounting-keys-2026-09-01.out` (archived, extracted via `git show 3cff909b0^:research/results/e71-accounting-keys-2026-09-01.out`, sha256 `9826e85e…`) | 90 lines on each side, `diff` showed 0 lines differing |
| V2 Independent anchors | 48 lines of `e71` anchors in author's `anchors.py` output (sha256 `a3d0bd98…`) checked line-by-line against `name=publish_budget` | `e71_anchor_rows matched=48 differing=0 missing=0` |
| V3 Positive controls | Lines 281–286 of artifact | Across 60 cells per arm legacy = 2 × new; dropping device dimension on upper bound arm yields exact ÷ 8, narrow and wide interpretations strictly decrease by less than 8× |
| V4 Determinism | `cmp` across 5 runs of identical binary | Byte-for-byte identical across 5 runs (sha256 `885c238a…`). Deterministic model, round count merely indicates absence of hidden state |

**Unit Tests and Mutations**: 15 unit tests all green; mutation table `scripts/mutation/tables/e71_accounting_keys_single_generation.tsv` has 10 entries (registered EV1–EV9 plus EV10 "dropping device dimension on wide interpretation arm" added in registration amendment), 10 caught, 0 invalid, 0 non-red; verbatim closing lines of  (3 items reuse / reused)`mutate.sh` (full output in `research/results/e71-accounting-keys-single-generation-mutations-2026-10-20.log`):

```
计数：内存撞顶 0 条（上限 2G）、超时 0 条、排不上 0 条、被总上限挤掉 0 条、显存不够 0 条、锚点失效 0 条、现判 7 条、复用 3 条、拒算 0 条
已还原，基线仍全绿
```

**Registration Amendments** (recorded in pre-registration "Section 12, Amendments" prior to artifact generation): wide interpretation arm changed from "unrun once sufficient to judge" to included in artifact; added mutation EV10; positive control A expanded to four width tiers. **Unrun Once Sufficient to Judge**: incidental quantity "bursting point" registered (minimum t × d exceeding budget under new basis for upper bound arm across widths, corresponding to D5 (Snapshot / Space Accounting Mechanism) settled item 1, not in Q3) and "today's implementation row set 3 + 6d rows × 34 bytes" (provided solely for magnitude reference).

**What It Cannot Answer**: Pure arithmetic, lacking accounting tree, btree, and write buffer; statistics reflect 2026-09-01 nine-item table rather than today's twelve items or today's 3 + 6d rows written per publish; does not model step rows for inode number watermarks (persisting with roots, at most one extra row written per publish); 34 bytes treated solely as geometry sensitivity point; counts entry bytes rather than dirty nodes (which belongs to E97 (Entry Encoding of Accounting and Allocation Records) Criterion 10).

### Paths and Conclusion Registration

| # | Path | Calculation Method | Source Placement | Shared Items Read |
|---|---|---|---|---|
| 1 | New harness two-basis arithmetic | Count single-generation entries per arm, multiply by write count factor (legacy 2, new 1) and entry width, compare against threshold; scan all 180 registered cells (4 width tiers × 15 (t, d) × 3 arms), no sampling | `scripts/experiment-apparatus/e71_accounting_keys_single_generation.rs:219 publish_write_bytes` | Dimension flags of 2026-09-01 nine-item table, three width tiers + 34 bytes, threshold 21 474 836, two write count factors (copied from pre-registration readings 1–3) |
| 2 | Independent anchor script | Compute 48 cells (4 width tiers × 4 groups of (t, d) × 3 arms) solely per formulas in pre-registration readings 2 and 3, without reading harness or `crates/` code; 48 cells selected around judgement and neighboring cells, not full scan | No implementation: script not in repo, resides in author's draft directory `~/.local/share/singlefs/work/m3-close-w8-c693-reruns/anchors.py` (sha256 `1cef0e2b…`), output pinned as 48 constant lines in unit test `independent_anchor_rows_match_on_every_width_arm_and_cell` | Shared nine-item table, widths, threshold, and factors from row 1 (same registration readings) |
| 3 | Original harness legacy basis | Original harness `checkpoint_bytes` (2 × single-generation entries × entry width), yielding 90 cells of `name=ckpt_budget` on 2026-09-01 | `scripts/experiment-apparatus/e71_accounting_keys.rs:128 checkpoint_bytes` | Nine-item table, three width tiers, threshold; legacy factor 2 |

All three paths share the same nine-item table and threshold: if table itself is not the intended one (twelve items today), all three err together, so numbers in this section are framed strictly as "per 2026-09-01 nine-item table". Comparison between path 1 and path 2 is pinned in unit test (48 lines of `assert_eq!`); comparison between path 1 and path 3 has only 3 lines pinned in unit test `original_format_lines_match_the_original_product_text`; the full run 90-line diff is a `diff` command lacking connecting assertions in source code, an owed check yet to be repaid.

### What It Cannot Answer

1. Pure arithmetic model: no accounting tree implementation, no btree, no write buffer, 0 file operations.
2. **Byte width of key segments has never been settled anywhere in repo** (D5 (Snapshot / Space Accounting Mechanism) settled item 3 settled format only, not width)
   ⇒ Three width tiers (22 / 30 / 50 bytes including value) are **assumptions**, forbidden to treat byte counts of any single tier as conclusions.
   ⚠️ **Hard lower bound already established for generation segment width (2026-09-02)**: D16 (Publish Semantics) settled item 6 settled generations advance with publishes
   (at local rate 2785 publishes/sec, 32-bit wraps in 17.8 days) ⇒ rider clause "generation segment ≥ 48 bits",
   **byte arithmetic of E71 (Entry Count and Width of Accounting Keys) must re-run accordingly** — prior to re-run, these three tiers serve merely as shape references.
   Bursting condition `t × d ≳ 65536` was given under 30-byte tier, must be recalculated if width changes.
3. **No basis for realistic range of tree count t**: D6 (Snapshot Implementation Model) settled multiple writable heads, but how many trees exist in a pool
   has not been written anywhere. Tiers such as t=8192 were scanned, not workload realities.
4. **"Which statistics carry tree dimension" unsettled** (table in D5 (Snapshot / Space Accounting Mechanism) settled item 4 only listed device dimension),
   hence the 14.5× difference between narrow and wide interpretations
   is the **magnitude of divergence** measured by E71 (Entry Count and Width of Accounting Keys), not the answer.

### Prerequisites

None. Pure arithmetic. **Original registered prerequisite "D5 (Snapshot / Space Accounting Mechanism) settled item 4 (which statistics exist)" settled on 2026-09-01.**

### Measurement Basis and Re-run

- Code `scripts/experiment-apparatus/e71_accounting_keys.rs`
  (`cargo run --release --bin e71-accounting-keys`),
  raw output `e71-accounting-keys-2026-09-01.out` (209 lines, ending with line `emitted=209`).
- **Byte-for-byte identical across N=5 rounds** (identical md5). **12 unit tests, 8 mutations hit one-by-one, 0 blind spots**,
  mutation table `scripts/mutation/tables/e71_accounting_keys.tsv`
  (`bash scripts/mutation/mutate.sh e71-accounting-keys scripts/experiment-apparatus/e71_accounting_keys.rs scripts/mutation/tables/e71_accounting_keys.tsv`).
- Run 1 re-run (2026-10-20): code `scripts/experiment-apparatus/e71_accounting_keys_single_generation.rs` (bin `e71-accounting-keys-single-generation`, does not `use` original harness or `crates/`),
  raw output `research/results/e71-accounting-keys-single-generation-2026-10-20.out` (302 lines, ending with line `emitted=302`),
  re-run registered in `scripts/experiment-apparatus/replay.sh` as `E71|e71-accounting-keys-single-generation||e71-accounting-keys-single-generation-2026-10-20.out|exact`.
  Mutation: `cd scripts/experiment-apparatus && bash ../mutation/mutate.sh e71-accounting-keys-single-generation e71_accounting_keys_single_generation.rs ../mutation/tables/e71_accounting_keys_single_generation.tsv` (limit `MUTATE_MEMORY_MAX=2G`).

### Impacted Decisions

| Decision Item | Relationship | Review |
|---|---|---|
| D5 (Snapshot / Space Accounting Mechanism) settled item 4 | Support | 2026-10-21 unaffected: heading and index row now also state the 2026-10-20 Run 1 re-run; body, numbers, conclusions, and artifacts unchanged; previously, 2026-10-21 unaffected: only the rerun registry sentence in the body was rewritten to include the short name; numbers, conclusions, and artifacts are unchanged; prior 2026-10-20 changed: the basis sentence is rewritten with the figures of Run 1 re-run — the entry-count ratio grows with `tree count × device count`; at `1024 × 64` the wide interpretation arm stays within budget under the new basis for all three widths, the upper bound arm exceeds it only at the 50-byte width, and the narrow interpretation arm exceeds it in 0 of 45 cells (`research/prompts/m3-close-c693-r3-main-verification.md` section 2, cell T3) |
| D5 (Snapshot / Space Accounting Mechanism) settled item 1 | Support | 2026-10-21 unaffected: heading and index row now also state the 2026-10-20 Run 1 re-run; body, numbers, conclusions, and artifacts unchanged; previously, 2026-10-21 unaffected: only the rerun registry sentence in the body was rewritten to include the short name; numbers, conclusions, and artifacts are unchanged; prior 2026-10-20 unaffected: statement in basis that "what stretches budget to bursting first is the product `tree count × device count`" holds as before under new basis — 5 over-budget cells of upper bound arm all sit at large t × d end, narrow interpretation arm has 0 cells; only threshold point changed (30-byte tier 1024 × 64 no longer exceeds budget) |
| D5 (Snapshot / Space Accounting Mechanism) settled item 2 | Does not affect | 2026-10-21 unaffected: heading and index row now also state the 2026-10-20 Run 1 re-run; body, numbers, conclusions, and artifacts unchanged; previously, 2026-10-21 unaffected: only the rerun registry sentence in the body was rewritten to include the short name; numbers, conclusions, and artifacts are unchanged; prior 2026-10-20 unaffected: Run 1 re-run used current settlement of this item (each root holds only one generation, each publish rewrites full generation once) as input basis, neither measuring nor judging it |
| D16 (Publish Semantics) | Does not affect | 2026-10-21 unaffected: heading and index row now also state the 2026-10-20 Run 1 re-run; body, numbers, conclusions, and artifacts unchanged; previously, 2026-10-21 unaffected: only the rerun registry sentence in the body was rewritten to include the short name; numbers, conclusions, and artifacts are unchanged; prior 2026-10-20 unaffected: threshold remains 1% of `T_dirty` = 2 GiB from settled item 5, Run 1 re-run did not alter it |
| D6 (Snapshot Implementation Model) | Does not affect | 2026-10-21 unaffected: heading and index row now also state the 2026-10-20 Run 1 re-run; body, numbers, conclusions, and artifacts unchanged; previously, 2026-10-21 unaffected: only the rerun registry sentence in the body was rewritten to include the short name; numbers, conclusions, and artifacts are unchanged; prior 2026-10-20 unaffected: tree count t remains scanned value, Run 1 re-run added no basis |
| D22 (Synthesizing Unit Atomicity) | Does not affect | 2026-10-21 unaffected: heading and index row now also state the 2026-10-20 Run 1 re-run; body, numbers, conclusions, and artifacts unchanged; previously, 2026-10-21 unaffected: only the rerun registry sentence in the body was rewritten to include the short name; numbers, conclusions, and artifacts are unchanged; prior 2026-10-20 unaffected: value of K enters only total entry count, new basis of Run 1 re-run uses K ≡ 1 and does not utilize it |

## Historical Versions

Historical entries for E71 (Entry Count and Width of Accounting Keys) are concentrated in [experiments-history.md](../experiments-history.md).
