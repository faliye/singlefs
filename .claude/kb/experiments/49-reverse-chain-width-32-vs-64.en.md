## E49 Reverse Chain Width: 32 vs 64 — Run (2026-08-30, N=5 rounds byte-identical, 11 unit tests / 7 mutations all caught; record header 307 → 311 rerun 2026-09-25; 311 → 487 rerun 2026-10-04): **64 is superior to 32, but only in the role of record checker**; ⚠️ **Cost-side conclusion no longer applies since 2026-09-07**—it held at base = 95, whereas current record header excluding reverse chain is 483

[中文](49-反向链宽度32还是64.md) · **English** · [日本語](49-逆方向チェーン幅-32か64か.ja.md)

**It answers the remaining half of D23 (Journal Role & Format) settled item 8.** The three-way argument judged
"16 bits is unacceptable", based on the **probability of at least one false acceptance in a lifetime**; applying the same ruler one step higher: **what does 64 bits buy over 32 bits.**

⚠️ **Cost-side conclusion no longer applies to the current header width (annotated 2026-09-07; conclusion itself not altered, its premise changed)**:
Conclusion 3 of E49 (Reverse Chain Width: 32 vs 64) verbatim states "cost is zero at base = 95", where base = 95 = **then-current** record header 86 + transaction fields 9; **the current record header excluding reverse chain is 492** (D23 (Journal Role & Format) settled item 4: header 496 contains reverse chain 4). **In the table of E49 (Reverse Chain Width: 32 vs 64) itself, base = 78 has 31 divergence points for 2 vs 4 and is not free** ⇒ cost side must be recalculated according to current header width, **prohibiting citing "both sides are free" as a conclusion**; recalculation per 483 is detailed in section "2026-10-04: Cost-Side Recalculation for base=483/307 Under Record Header 487": free under reading B, not free for 4 vs 8 under reading A. Benefit side (64 superior to 32 only on the record checker side) remains unaffected. Placement in [checks-owed.md](../checks-owed.md) C200 (Cost Recalculation After Record Header Widening Not Written Back to Clause).

⚠️ **This owed check deepened on 2026-09-24 and remains unsettled**: D23 (Journal Role & Format) settled item 4 on that day widened the record header from 307 to 311 (adding 4 bytes for "sequence number within this publish"), making the current value excluding reverse chain **307** (not 303 as written earlier in this paragraph), with the four tiers currently 308 / 309 / 311 / 315 bytes (`e43-r1-prereg.md` appendix "Line-by-line verdict of issue sheet row 7", line 5 judged "participates in criteria, requires rerun"). That header 311 rerun did not bundle this cost-side recalculation—the repayment plan for C200 (Cost Recalculation After Record Header Widening Not Written Back to Clause) with its hardcoded "base = 78 and base = 91" no longer corresponded to any current measurement basis, requiring human re-establishment of base measurement basis before scheduling recalculation rather than a simple substitution of numbers. **On 2026-09-25 supplementary runs were completed for base under header 311 (307) and the tier prior to header 311 (303), see section "2026-09-25: Cost-Side Recalculation for base=307/303 Under Record Header 311"; whether to roll 78/91 in C200 (Cost Recalculation After Record Header Widening Not Written Back to Clause) and the sentence "current value excluding reverse chain is 307" to 307/303 is left to the main agent.** On 2026-10-04 the record header was changed again from 311 to 487 (new root segment 188 → 364), making the current value excluding reverse chain **483**; supplementary runs for 483 and 307 were performed, see section "2026-10-04: Cost-Side Recalculation for base=483/307 Under Record Header 487".

⚠️ **E49 (Reverse Chain Width: 32 vs 64) does not reopen the "whether to add" half** (settled: add). Nor does it **resolve** the two prerequisites
(which fields `previous_hash` covers, and whether the chain has been rendered redundant by instance generation)—those two are format requirements, not arithmetic.

### What to Test

**Sweep the domain on both sides without taking sample points.**

| Side | What to Sweep |
|---|---|
| **Cost** | Total record header bytes `base` ∈ all candidates appearing in repo × chain width ∈ {2,4,8} × named item count **0..2000** × atomic unit ∈ {512, 4096} ⇒ judge cell by cell: "is unit count on disk identical" |
| **Benefit** | Compute expected false acceptances and probability of "at least once in lifetime" across two **distinct denominators** |

**The two denominators must be separated; they differ by several orders of magnitude**:

| Denominator | What It Is | Values |
|---|---|---|
| **D-Incident** | How many times in a lifetime the incident sequence "hole → recovery → resumed write → second crash" occurs | **36,500** (crashed 10 times daily × 10 years)—**this is the scenario point already adopted when sizing instance generation in D23 (Journal Role & Format) settled item 9**, reused by E49 (Reverse Chain Width: 32 vs 64) without invention |
| **D-Traversal** | How many adjacent record pairs record checker compares per full ring run × runs per lifetime | Ring ∈ {10 MiB (XFS lower bound), 100 MiB, 2 GiB (XFS upper bound)} ÷ on-disk record occupancy ∈ {512, 4096} × frequency ∈ {weekly, daily} × 10 years |

**D-Traversal is the denominator for the second role identified by the backward leg**: In addition to distinguishing timelines, reverse chain buys **traversal independence** documented in D13 (Verification Route)—record checker walks the chain to verify prefix boundaries, **with each pair comparison representing a collision opportunity**.

### Acceptance Criteria (Written Before Run)

| # | Criterion |
|---|---|
| 1 | **"n bits is sufficient"**: Probability of "at least once in lifetime" under that denominator is **< 1%** |
| 2 | **"n bits is free compared to m bits"**: On-disk occupancy is identical across **all** scanned `(base, items, unit)` cells |
| 3 | Conclusions must be presented **separately** per denominator, **not merged into one** |

### Failure Conditions (Written Before Run, Immutable Afterward)

| Clause | Verdict when Triggered |
|---|---|
| **Positive control**: 8 bits under D-Incident must have "at least once in lifetime" ≈ 100% | Not so ⇒ arithmetic not computing probability, **entire run voided** |
| **Positive control 2**: 2 and 4 bytes at `base = 93` must **diverge** (settled by E39 (Does Reverse Chain Block Remnant Records) re-computation: 19 points under 512 unit, minimum items=44) | Judged non-divergent ⇒ on-disk occupancy computation erroneous, **entire run voided** |
| 32 and 64 are **both sufficient** under both denominators with identical cost | **Record faithfully: "both sufficient; wider brings no additional benefit nor cost"**, no tweaking criteria to force a direction |
| 64 bits occupies more units than 32 bits on any base | **Record faithfully**—in that case "going wider" pays real cost |

### What It Cannot Answer

1. **Two prerequisites** (which fields `previous_hash` covers, and whether the chain has been rendered redundant by 32-bit instance generation)—**arithmetic cannot touch them**.
   ⚠️ **When prerequisite 1 is unsatisfied, both tables of E49 (Reverse Chain Width: 32 vs 64) are completely invalidated**: If hashed inputs of two competing records are constructively equal,
   false acceptance rate is **100%, regardless of bit width** (recorded in D23 (Journal Role & Format) settled item 8).
2. **Values of D-Incident and D-Traversal are scenario points, not measurements**: This repository has neither crash telemetry nor a record checker.
3. **Hash algorithm** is unsettled ⇒ "false acceptance = 2⁻ⁿ" is conditional on "uniform chain values". E40 (Ciphertext Checksum Bit Width) empirically demonstrated
   that **truncated CRC is not a shorter CRC**—if chain values take truncated CRC, this curve does not apply.

### Prerequisites

None. Pure arithmetic.

### Measurement Basis and Reproduction

- Code `scripts/experiment-apparatus/e49_chain_width.rs`
  (`cargo run --release --bin e49_chain_width`),
  raw output `e49-chain-width-2026-10-04-h487.out` (`e49-chain-width-2026-09-25.out` from header 311 and the 2026-08-30 copy were archived out of workspace per "delete prior experiment output on commit"; header 311 copy deleted in commit 31128273, see section "2026-09-25").
- Mutation table `scripts/mutation/tables/e49_chain_width.tsv` (14 entries; 12 caught in header 311 run, M13/M14 added in header 487 run; entire table rerun after header 487 caught 14/14, 0 ineffective, 0 un-red, log `e49-mutate-2026-10-04-h487.log`, see section "2026-10-04").

### Empirical Measurements (2026-08-30, N=5 rounds byte-identical, emitted trailing line in artifact)

#### Cost: "Free" Is an Accident of base mod 8, on Both Sides

Swept across 6 bases × 2 unit tiers × item count 0..2000 (12 cells):

| base | Source | 2 vs 4 Bytes | 4 vs 8 Bytes |
|---|---|---|---|
| 78 | 84 − `tail_lsn` + 2 | **31 divergence points** (512 unit, first items=26) | 0 |
| 84 | Original table of E23 (Journal Geometry) | 0 | **31 points** (first items=35) |
| 86 | Header after widening `jsn` | **31 points** (first 35) | 0 |
| 93 | Used by E39 (Does Reverse Chain Block Remnant Records) (84 + transaction fields 9) | **31 points** (first 44) | 0 |
| **95** | Current value during run (86 + transaction fields 9). ⚠️ **86 was header width before 2026-08-30**; current header without reverse chain is 492 (D23 (Journal Role & Format) settled item 4: header 496 contains reverse chain 4) ⇒ this tier is not current value | **0** | **0** |
| 99 | Adding 4 more bytes | 0 | **31 points** (first 53) |

**Across 12 cells, 2v4 is free in 6 cells, 4v8 free in 8 cells.**
⇒ **"64-bit is also free" is not a structural property, but an accident of header byte count in this revision**—
`gcd(56, 512) = 8`, and freeness depends strictly on `total record header bytes mod 8`.
⚠️ **Only base = 95 is free on both sides simultaneously** (current during run, superseded later),
the sole instance among six candidates. **Shifting header by a few bytes may cause both sides to diverge again.**

#### Benefit 1: D-Incident (36,500 Opportunities)—Both 32 and 64 Far Exceed Requirements

| Bit Width | Expected False Acceptances | At Least Once in Lifetime | Sufficient (< 1%) |
|---|---|---|---|
| 8 | 142.6 | **100%** | No |
| 16 | 0.557 | **42.7%** | No |
| **32** | 8.50×10⁻⁶ | **0.00085%** | **Yes** |
| 64 | 1.98×10⁻¹⁵ | ~0 | Yes |

⇒ **64-bit buys nothing in the role of "distinguishing timelines"**: 32 bits is already three orders of magnitude wider than criteria.

#### Benefit 2: D-Traversal (Record Checker)—32-Bit Insufficient in 8 of 12 Tiers

⚠️ **The denominator measurement basis in this section was judged overcounted by E51 (Number of Reverse Chain Collision Opportunities) (same day 2026-08-30)**:
It counted **every adjacent comparison** as a collision opportunity, whereas **chain values in lineage comparisons are constructively equal and contain no randomness**
⇒ The correct denominator is **seam count**, empirically almost constantly equal to incident count (3.65×10⁴) across 12 tiers,
making **32-bit sufficient in every tier**. ⇒ **The 12-tier table of "Benefit 2: D-Traversal (Record Checker)—32-Bit Insufficient in 8 of 12 Tiers" is retained under reading A, but is no longer the basis for judging width.**

Denominator = record pairs compared per pass × lifetime passes (3 ring tiers × 2 on-disk occupancy tiers × 2 frequency tiers = 12 tiers):

| Bit Width | Tiers Sufficient out of 12 |
|---|---|
| 8 / 16 | **0** |
| **32** | **4** |
| **64** | **12** |

The two decisive tiers:

| Tier | Comparisons per Pass | Lifetime Opportunities | 32-bit Expected / At Least Once | 64-bit |
|---|---|---|---|---|
| Lightest: 10 MiB ring / 512 B per record / weekly | 20,480 | 1.07×10⁷ | 0.00249 / **0.249% (sufficient)** | 5.8×10⁻¹³ |
| **Heaviest: 2 GiB ring / 512 B per record / daily** | 4,194,304 | 1.53×10¹⁰ | **3.56 times / 97.2% (insufficient)** | 8.3×10⁻¹⁰ |

⇒ **Everything 64-bit buys is on this side**: Each pair compared by record checker is a collision opportunity,
with its denominator exceeding the incident denominator by **3 to 6 orders of magnitude**.

#### Item-by-Item Verdicts for Controls and Failure Clauses

| Clause | Verdict |
|---|---|
| **Positive control 1**: 8-bit under incident denominator ≈ 100% | **Pass** (`at_least_once=1.000000`) |
| **Positive control 2**: 2 and 4 at base=93 must diverge, first items=44 | **Pass** (`div=31 first=44`) |
| 32 and 64 both sufficient under both denominators with identical cost ⇒ record "both sufficient" | **Did not occur**: 32-bit insufficient in 8/12 tiers on traversal denominator |
| Mutation testing | **7 mutations all caught** (including "loosening sufficiency criterion to pass all" and "divergence sweep missing one tier") |

⚠️ **Two pre-written expectations rejected by unit tests; arithmetic was fixed, not criteria**:
(1) I initially wrote "4 and 8 do not diverge on any base", but base=84 and 99 each had 31 divergence points ⇒ rewritten as **conditional statement**;
(2) On heaviest tier I wrote 32-bit expectation as 3.56×10⁻², whereas actual was **3.56** (off by two orders of magnitude), with at least once being 97.2% rather than 3.5%.
**Not a single word in criteria ("sufficient" = < 1%, "free" = identical across all cells) was modified.**

### Conclusions

1. **64-bit buys nothing in the role of "distinguishing timelines"**—32 bits already exceeds criteria by three orders of magnitude.
2. **64-bit is significantly superior to 32-bit in "traversal independence for record checker"**:
   Among 12 denominator tiers, 32 bits is sufficient in only 4 tiers, with the heaviest tier having **97.2% lifetime false acceptance**; 64 bits is sufficient in all 12 tiers.
3. **Cost is zero at base = 95, but that is an accident of `mod 8`**, the sole instance among six candidate headers free on both sides simultaneously.
   ⚠️ **The base in the table "Cost: 'Free' Is an Accident of base mod 8, on Both Sides" is header width excluding reverse chain, corresponding currently to 492** (D23 (Journal Role & Format) settled item 4: header 496 contains reverse chain 4), which is not among the six tiers; **in that table, base = 78 has 31 divergence points for 2 vs 4** ⇒ **this cost conclusion has no standing on the current header width** (same liability noted in scope of D23 (Journal Role & Format) settled item 8). **Supplementary runs completed on 2026-10-04 for 483 and the 307 tier from header 311** (supplementary runs for 307 and 303 performed on 2026-09-25), see section "2026-10-04: Cost-Side Recalculation for base=483/307 Under Record Header 487": under reading B (actual fixed-length record format), both bases judge identically and are free; under reading A (original variable-length record model of E49 (Reverse Chain Width: 32 vs 64)), both bases also judge identically—2 vs 4 is free, 4 vs 8 is not free (both 483 and 307 are 3 mod 8); the two readings judge oppositely on 4 vs 8 at 483.
4. ⚠️ **All three conclusions are conditional on a factor untouched by E49 (Reverse Chain Width: 32 vs 64)**: Fields covered by `previous_hash` must contain a
   field "guaranteed distinct across two timelines". **When unsatisfied, false acceptance rate is 100%, regardless of bit width**
   (recorded in D23 (Journal Role & Format) settled item 8). **Settle that requirement before discussing width.**

### Measurement Basis and Known Limitations

- Code `scripts/experiment-apparatus/e49_chain_width.rs`
  (`cargo run --release --bin e49_chain_width`),
  raw output `e49-chain-width-2026-10-04-h487.out`.
  Mutation table `scripts/mutation/tables/e49_chain_width.tsv` (14 entries; 12 caught in header 311 run, 7 original plus 5 new M8–M12 in section "2026-09-25", M13/M14 added in header 487 run).
- **Both denominators are scenario points, not measurements**: D-Incident reuses the "crashed 10 times daily × 10 years" adopted by D23 (Journal Role & Format) settled item 9;
  D-Traversal ring sizes take XFS lower / upper bounds and a middle value, with traversal frequency weekly / daily.
  **This repository has neither crash frequency telemetry nor a record checker.**
- **"False acceptance = 2⁻ⁿ" is conditional on uniform chain values**. Hash algorithm unsettled; E40 (Ciphertext Checksum Bit Width) demonstrated that **truncated CRC is not a shorter CRC**;
  if chain values take truncated CRC, this curve does not apply.
- Record checker **comparisons per pass** takes "how many records in ring", assuming a full ring pass;
  if it only compares incremental parts, denominator must be recalculated.

### 2026-09-25: Cost-Side Recalculation for base=307/303 Under Record Header 311 (Issue List Row 7, Rerun Pre-registration e49-r1-prereg.md)

This repays the "recalculate per current header width" owed by conclusion 3 and C200 (Cost Recalculation After Record Header Widening Not Written Back to Clause). The definition of base is consistent with the original table (total record header bytes excluding reverse chain); today = header 311 (D23 (Journal Role & Format) settled item 4) − chain 4 (D23 (Journal Role & Format) settled item 8) = **307**; before header 311 (current from 2026-09-14 to 2026-09-24) = 307 − 4 = **303**.

**On-disk occupancy has two readings, both retained** (settled by main agent 2026-09-24; criterion 2 verdict governed by reading B, reading A computed and reported alongside):

| Reading | Units Occupied per Record |
|---|---|
| A (original model of E49 (Reverse Chain Width: 32 vs 64)) | ⌈(base + chain_width + 56 × item_count) ÷ atomic_unit⌉, not rounded to fixed size |
| B (actual format today, D23 (Journal Role & Format) settled item 12: fixed-length record 4096) | Fits ⌊(4096 − base − chain_width) ÷ 56⌋ items per record; N items requires `max(1, ⌈N ÷ items_per_record⌉)` records |

Artifact `e49-chain-width-2026-09-25.out` (transcribed verbatim) (lines moved out of workspace shown as pointer lines):

```
Line in artifact see git show 5d0b7e7c1667:research/results/e49-chain-width-2026-09-25.out line 15 (name=cost_current)
Line in artifact see git show 5d0b7e7c1667:research/results/e49-chain-width-2026-09-25.out line 16 (name=cost_current)
E7RESULT name=cost_current base=307 unit=512 div_2v4=0 first_2v4=NA div_4v8=31 first_4v8=31 cap_items=3
E7RESULT name=cost_current base=307 unit=4096 div_2v4=0 first_2v4=NA div_4v8=4 first_4v8=287 cap_items=67
Line in artifact see git show 5d0b7e7c1667:research/results/e49-chain-width-2026-09-25.out line 19 (name=cost_fixed_record)
E7RESULT name=cost_fixed_record base=307 div_2v4=0 first_2v4=NA div_4v8=0 first_4v8=NA items_per_record_2=67 items_per_record_4=67 items_per_record_8=67
E7RESULT name=poscontrol_base84_four_versus_eight div=31 first=35 expect_divergent=1
E7RESULT name=poscontrol_fixed_record base=342 pair=2v4 div=455 first=67 expect_divergent=1
E7RESULT name=poscontrol_fixed_record base=340 pair=4v8 div=455 first=67 expect_divergent=1
E7RESULT name=done emitted=196
```

**Verdicts (Q49.2–Q49.4, cell by cell)**:

| base | Reading A: 2 vs 4 | Reading A: 4 vs 8 | Reading B: 2 vs 4 | Reading B: 4 vs 8 |
|---|---|---|---|---|
| 303 (before header 311) | Free | Free | Free | Free |
| 307 (today) | Free | **Not free** (31 points on 512, 4 points on 4096) | Free | Free |

- **Reading B (criterion 2 governed by it) unchanged**: On both 303 and 307 bases, 2 vs 4 and 4 vs 8 are both free, consistent with conclusion 3 original verdict (base=95 free on both sides).
- **Reading A changes**: 4 vs 8 changes from "free" (303) to "not free" (307); 2 vs 4 remains free on both bases.
- **F4 (two readings judge oppositely)**: At base=307 for 4 vs 8, reading A judges "not free" while reading B judges "free", listed side by side.

**Q49.5** (D23 (Journal Role & Format) settled item 8 scope "cost side... depends strictly on total record header bytes mod 8"): Under reading A sweeping base 296..319 (covering each remainder three times) confirmed valid—`base mod 8` falling in {1,2,3,4} only diverges on 4 vs 8 (31 on 512, 4 on 4096), in {5,6} only diverges on 2 vs 4, in {0,7} both free; 307 mod 8 = 3, 303 mod 8 = 7, falling in adjacent groups, explaining why reading A judges them oppositely. This statement holds only for reading A—reading B verdicts are not grouped by `base mod 8` (56 and 8 are not the same modulus); 303 and 307 are both at least 19 bytes away from their nearest flip points (4 vs 8 at base=284/337, 2 vs 4 at base=286/341), crossing neither point, hence both judge "free" (then-unit-test `fixed_record_reading_diverges_only_at_these_bases_in_280_to_345`, renamed to `fixed_record_reading_diverges_only_at_these_bases_in_440_to_520` after header changed to 487 bytes, with range shifted to 440..=520).

**Positive controls**: P1, P2 (original) plus newly added P3 (4 vs 8 at base=84, unit 512: `div=31 first=35`; original experiment only ran positive control for 2 vs 4), P4 (two for reading B: base=342 for 2 vs 4, base=340 for 4 vs 8, both `div=455 first=67`) all passed.

**Mutations**: Prior to modifying apparatus, original 7 mutations (M1–M7) run on today's source code caught 7/7, 0 ineffective, 0 un-red; after modification, expanded 12-entry table (adding M8–M12) caught 12/12, 0 ineffective, 0 un-red—original 7 states identical across runs (V4 untriggered). All 24 unit tests green (11 original + 13 new), including discriminative power self-test `discriminative_power_self_proof_flips_the_verdict`: shifting Q49.2 cell threshold (base=307, unit 512, 4 vs 8) from `div=0` to `d` (31, divergence count of that cell) flips verdict from "not free" to "free" (V5 untriggered).

**Reproduction**:

```
bash scripts/experiment-apparatus/replay.sh E49
```

Byte-identical `e49-chain-width-2026-09-25.out`.

**Unfinished**: The sentence in D23 (Journal Role & Format) settled item 8 scope "current header without reverse chain is 303" and the C200 (Cost Recalculation After Record Header Widening Not Written Back to Clause) repayment plan (base=78/91) do not match today's measurement basis; whether to update them to 307/303 is left to the main agent.

### 2026-10-04: Cost-Side Recalculation for base=483/307 Under Record Header 487 (Issue List Row 4, Rerun Pre-registration e49-r2-prereg.md)

D23 (Journal Role & Format) settled item 4 changed record header from 311 to 487 (new root segment 188 → 364, items per record 67 → 64), making current base excluding reverse chain = 487 − 4 = **483**; header 311 tier = 311 − 4 = **307**. In the apparatus, `JOURNAL_HEADER_BYTES`, sweep windows for reading A and B, and item upper bound remained at header 311 generation (old sweep see `sweep-311-188-67.md`). Rerun pre-registration hardcoded criteria per header 311 run (`e49-r1-prereg.md`, retaining both readings, criterion 2 governed by reading B), only changing the two judged bases, sweep windows (reading A base 472..495, reading B base 440..520), and item upper bound (67 → 64, module constant `ITEM_UPPER_BOUND_TODAY`). Prior to modifying apparatus, ran current code as baseline (apparatus 311, `crates/` 487, hardcoded pre-registration exception, baseline only, no conclusions), byte-identical to `e49-chain-width-2026-09-25.out`.

Artifact `research/results/e49-chain-width-2026-10-04-h487.out` (211 lines, trailing `emitted=211`; header 311 version had 196 lines, with 15 extra lines from reading B sweep window expanding from 66 → 81 bases). Compared to header 311 version: `cmp` reports differences starting line 15, `diff` counts 259 differing lines, all falling in the five families `cost_current`, `cost_fixed_record`, `base_sweep`, `base_sweep_items`, `done` (command and verbatim output):

```
$ diff e49-chain-width-2026-09-25.out research/results/e49-chain-width-2026-10-04-h487.out | grep '^[<>]' | grep -vE 'name=(cost_current|cost_fixed_record|base_sweep|base_sweep_items|done)( |$)' | wc -l
0
```

Key lines transcribed verbatim from new artifact:

```
E7RESULT name=cost_current base=307 unit=512 div_2v4=0 first_2v4=NA div_4v8=31 first_4v8=31 cap_items=3
E7RESULT name=cost_current base=307 unit=4096 div_2v4=0 first_2v4=NA div_4v8=4 first_4v8=287 cap_items=67
E7RESULT name=cost_current base=483 unit=512 div_2v4=0 first_2v4=NA div_4v8=31 first_4v8=37 cap_items=0
E7RESULT name=cost_current base=483 unit=4096 div_2v4=0 first_2v4=NA div_4v8=4 first_4v8=357 cap_items=64
E7RESULT name=cost_fixed_record base=307 div_2v4=0 first_2v4=NA div_4v8=0 first_4v8=NA items_per_record_2=67 items_per_record_4=67 items_per_record_8=67
E7RESULT name=cost_fixed_record base=483 div_2v4=0 first_2v4=NA div_4v8=0 first_4v8=NA items_per_record_2=64 items_per_record_4=64 items_per_record_8=64
E7RESULT name=poscontrol_8bit_incident at_least_once=1.000000 expect_near_one=1
E7RESULT name=poscontrol_base93_diverges div=31 first=44 expect_first=44
E7RESULT name=poscontrol_base84_four_versus_eight div=31 first=35 expect_divergent=1
E7RESULT name=poscontrol_fixed_record base=342 pair=2v4 div=455 first=67 expect_divergent=1
E7RESULT name=poscontrol_fixed_record base=340 pair=4v8 div=455 first=67 expect_divergent=1
E7RESULT name=base_sweep_items reading=甲 max_items=64 base=307 unit=512 div_2v4=0 div_4v8=1
E7RESULT name=base_sweep_items reading=甲 max_items=64 base=307 unit=4096 div_2v4=0 div_4v8=0
E7RESULT name=base_sweep_items reading=甲 max_items=64 base=483 unit=512 div_2v4=0 div_4v8=1
E7RESULT name=base_sweep_items reading=甲 max_items=64 base=483 unit=4096 div_2v4=0 div_4v8=0
E7RESULT name=done emitted=211
```

**Verdicts (cell by cell)**:

| base | Reading A: 2 vs 4 | Reading A: 4 vs 8 | Reading B: 2 vs 4 | Reading B: 4 vs 8 |
|---|---|---|---|---|
| 307 (header 311 tier) | Free | **Not free** (31 points on 512, 4 points on 4096) | Free | Free |
| 483 (today) | Free | **Not free** (31 points on 512, 4 points on 4096) | Free | Free |

- **483 and 307 judge identically cell by cell**: Across eight cells in reading A and B, not a single cell flipped. 483 and 307 are congruent (both 3 mod 8); reading A verdicts and divergence counts are identical, changing only first divergence item count (31 → 37 on 512, 287 → 357 on 4096), `cap_items` (3 → 0 on 512, 67 → 64 on 4096), and items per record in reading B (67 → 64).
- **Compared to conclusion 3 (base = 95 free on both sides)**: 2 vs 4 on 483 is free, matching it; 4 vs 8 is not free under reading A and free under reading B.
- **F4 (two readings judge oppositely)**: At base = 483 for 4 vs 8, reading A judges "not free" while reading B judges "free", listed side by side; criterion 2 verdict governed by reading B (settled by main agent 2026-09-24).
- **mod 8 regularity** (D23 (Journal Role & Format) settled item 8 scope "cost side... depends strictly on total record header bytes mod 8"): Under reading A, base 472..495 (24 bases, each remainder three times) grouped by `base mod 8` show only one 4-cell signature per group: remainder 0 and 7 is (0, 0, 0, 0), remainder 1–4 is (0, 31, 0, 4), remainder 5–6 is (31, 0, 4, 0) (the four cells being divergence counts for 2v4 on 512, 4v8 on 512, 2v4 on 4096, 4v8 on 4096)—this holds across these 24 bases, only for reading A. Command and verbatim output in `e49-chain-width-2026-10-04-h487-analysis.out`.
- **Reading B flip points** (same analysis output): Across base 440..520, 2 vs 4 diverges only at 453, 454, 509, 510; 4 vs 8 diverges only at 449, 450, 451, 452, 505, 506, 507, 508; 483 is separated from nearest flip points (4 vs 8 at 452 and 505) by 31 and 22 bytes, 2 vs 4 at 454 and 509 by 29 and 26 bytes, not crossing them; for 307 side see window 280..345 in section "2026-09-25".
- **Item upper bound 64**: Four rows of `name=base_sweep_items`, base 307 and 483 both have one divergence on 512 unit 4 vs 8, remaining three cells non-divergent. Compared to upper bound 2000: 512 unit 4 vs 8 still judges "not free" (1 vs 31 points); 4096 unit 4 vs 8 flips from "not free" (4 points) to "free" (0 points), because first divergence item counts 287 and 357 lie outside 64—reading A verdict on this cell is unstable with respect to item upper bound, remaining three cells unchanged. Sixth section verdict follows domain of upper bound 2000 and is not modified by this row.
- Positive controls (P1–P4) identical line-by-line with header 311 version and passed; P5 (base knob was read): two groups of `name=cost_current` lines differ in `first_4v8=` and `cap_items=` in addition to `base=`.

Family counts (command and verbatim output):

```
$ sed -E 's/^E7RESULT name=([a-z_0-9]+).*/\1/' research/results/e49-chain-width-2026-10-04-h487.out | sort | uniq -c
    129 base_sweep
      4 base_sweep_items
      1 config
     12 cost
      4 cost_current
      2 cost_fixed_record
      1 cost_summary
      1 done
      1 poscontrol_8bit_incident
      1 poscontrol_base84_four_versus_eight
      1 poscontrol_base93_diverges
      2 poscontrol_fixed_record
      4 risk_incident
     48 risk_traversal
$ grep -n 'name=verdict' research/results/e49-chain-width-2026-10-04-h487.out
（无输出，退出码 1：这份产物没有 `name=verdict` 行）
```

**Issue sheet row 4 (E49 (Reverse Chain Width: 32 vs 64)): Artifact changed (259 differing lines), verdicts unchanged**—reading B free on both 483 and 307, reading A judges 483 and 307 identically cell by cell (2 vs 4 free, 4 vs 8 not free), with first divergence item count, `cap_items`, and items per record changed.

All 25 unit tests green (`e49-unit-test-2026-10-04-h487.out`), including discriminative power self-test and assertion pinning first divergence item count (catching mutation changing header width back to 311, since 483 and 307 are congruent with identical verdict cells). Full rerun of 14-entry mutation table run by main agent (run in background, not during artifact run), log `e49-mutate-2026-10-04-h487.log`:

```
$ for s in '✅' '⏭' '❌' '💥' '⚠️' '⏱' '🧱'; do printf '%s=%s ' "$s" "$(grep -c "^$s" e49-mutate-2026-10-04-h487.log)"; done; echo
✅=14 ⏭=0 ❌=0 💥=0 ⚠️=0 ⏱=0 🧱=0
$ grep -E '^(计数：|已还原)' e49-mutate-2026-10-04-h487.log
计数：内存撞顶 0 条（上限 1G）、超时 0 条、排不上 0 条、被总上限挤掉 0 条
已还原，基线仍全绿
```

**Caught 14, ineffective 0, un-red 0**, sum of columns 14 equals table's 14 rows (`scripts/mutation/tables/e49_chain_width.tsv` 14 rows), full table completed. Pre-registration hardcoded "original eleven mutations must have identical states across runs" (V4; eleven refers to M1–M8, M10–M12), not run prior to modifying apparatus (rationale in section 12 of registration); V4 only triggers if any post-modification mutation is not "caught"; M1 through M14 are individually ✅ ⇒ **V4 untriggered**; header 311 run caught 12/12, 0 ineffective, 0 un-red ("Mutations" paragraph), and header 487 run keeps ineffective and un-red at 0. M9 source changed from `= 311;` to `= 487;` and still caught; newly added M13 (header width reverts to 311) and M14 (item upper bound hardcoded to old value) both caught: M13 red on five tests including `base_today_and_base_before_are_derived_correctly`, M14 red on `item_upper_bound_today_is_sixty_four`. Ineffective column did not increase (0).

Reproduction:

```
bash scripts/experiment-apparatus/replay.sh E49
```

Byte-identical `e49-chain-width-2026-10-04-h487.out`.

**Unfinished**: The sentence in D23 (Journal Role & Format) settled item 8 scope "current header without reverse chain is 492 (496 − 4), not among swept tiers ⇒ prior to recalculation, citing 'choosing 16 saves nothing', '64-bit also free', 'neither side diverges' as conclusions is prohibited" and the C200 (Cost Recalculation After Record Header Widening Not Written Back to Clause) repayment plan—whether to rewrite them based on this recalculation is not judged on this page.

### Affected Decisions

| Decision Item | Relationship | Review |
|---|---|---|
| D23 (Journal Role & Format) settled item 8 | Material | 2026-10-22 unaffected: only the format widths in sentences describing the current state were changed; previously, 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-17 Unaffected: only adds unit test names then and now in question 5; numbers, conclusions, and artifacts remain unchanged |
| D13 (Verification Route) | No effect | 2026-10-22 unaffected: only the format widths in sentences describing the current state were changed; previously, 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-17 Unaffected: only adds unit test names then and now in question 5; numbers, conclusions, and artifacts remain unchanged |

## Historical Versions

### 2026-08-30

- Run completed. Criteria unchanged; two pre-written expectations rejected by unit tests (4v8 diverging on base 84/99,
  heaviest tier 32-bit expectation undercounted by two orders of magnitude); arithmetic was fixed.
- Established. Rationale: After three-way argument in D23 (Journal Role & Format) settled item 8 ruled out 16 bits,
  no one had calculated the question one step higher (what 64 buys over 32);
  and cost table of E39 (Does Reverse Chain Block Remnant Records) only took 5 item counts, requiring domain sweep per C49 (Universal Assertions Only Verified at Sample Points).
