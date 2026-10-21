## E130 Does Per-Head Livelist Pass Bounded Destruction — Run (2026-09-10)

[中文](130-每头一份livelist过不过有界销毁.md) · **English** · [日本語](130-每头一份livelist过不过有界销毁.ja.md)

**What is being asked**: Whether Candidate A (per-head livelist) of D6 (Snapshot implementation model) settled item 2 passes
Criterion 5 of the nine criteria in D6 (Snapshot implementation model) — "**Bounded destruction**: batchable, resumable,
worst-case space requirement per batch is bounded and can be pre-reserved before the batch".

**Why it needed to be run**: In that round, tripartite argument eliminated Arm C and Arm B, and re-evaluated Arm D's cost, leaving only Arm A surviving;
yet the backward-inference leg explicitly stated: "My conclusion only goes as far as 'Arm B cannot pass',
**not as far as 'Arm A can pass'**" —
Arm A has an entry count, but the formula for "worst-case space requirement per batch" and the cost of "being condensable" had zero numbers in the warehouse.

**Hardcoded before running**: `e130-preregistration.md` (criteria, thresholds, invalidation clauses,
and "whether I accept the opposite outcome" were all written before the measurement code).

**Re-run command** (registered as `exact` mode, compared byte-for-byte against committed artifacts):

```bash
bash scripts/experiments/replay.sh E130
# Run apparatus directly:
cd scripts/experiments && cargo run --release --bin e130_livelist_bounded_destroy
```

Code: `scripts/experiments/e130_livelist_bounded_destroy.rs`, raw output: `e130-livelist-bounded-destroy-2026-09-10.out` (single run, no parameter sweep).

### This Is a Counting Model, Not an Implementation — It Proves Nothing

No file I/O, no concurrency, no random source ⇒ running the same binary N times is necessarily identical byte-for-byte.
**"Consistent across N rounds" demonstrates the absence of hidden state, not statistical stability**
(`.claude/singlefs-ai-sop/rules/test-discipline.md`).
Evidence strength derives from **22 unit tests + 15 mutations all caught**, not from round counts.

**What it does not answer**: real I/O counts, wall-clock cost of condense itself, crash consistency;
the "resumable" half is not modeled — it relies on the intent mechanism of D8 (Core index structure) settled item 5,
which this apparatus does not verify by even a single word.

### Arms

| Code | Definition |
|---|---|
| `naive` | Appends one entry per ALLOC, appends one entry per FREE, **never condenses** |
| `condense` | Appends one entry per ALLOC, appends one entry per FREE, and cancels out once when structural entries > 2 × net live blocks, retaining only net ALLOC |
| `no_structure` | Arm B form, **control arm** (eliminated on 2026-09-10, included to provide a subtrahend for "what extra Arm A pays") |

⚠️ **The trigger predicate for `condense` was modified once, before any measurement was run**: Version 1 wrote "FREE entry ratio ≥ 1/2",
whereas in this workload family `free_event_count / (allocation_event_count + free_event_count) = c / (1 + 2c)`, which is strictly less than 1/2 for any finite `c`
⇒ antecedent is vacuously false, the arm never triggers once, acting as an exact replica of `naive` —
the **strawman control arm** explicitly called out in `.claude/singlefs-ai-sop/rules/evidence-discipline.md`,
as well as the form in `.claude/singlefs-ai-sop/rules/test-discipline.md` where "antecedents of failure clauses can be written inverted".
It was exposed by this apparatus's unit tests, not artifacts. The inverted version is archived by unit test `the_first_condense_predicate_never_fires`.

### Conclusion: Arm A Passes Criterion 5, with Two Costs

Artifact transcribed line-by-line from `e130-livelist-bounded-destroy-2026-09-10.out`.

| Criterion hardcoded before run | Artifact | Verdict |
|---|---|---|
| 1. Worst-case space requirement per batch scales only with batch size | `name=criterion1 arm=naive distinct_worst_batch_bytes=1 value=180224`, `condense` same value, `no_structure` is `81920`; all three arms sweep 24 workload cells, distinct value set size is 1 across each | **Pass**. 180224 = 4096 × (20 + 24) |
| 2. Pre-batch reservation is calculable and does not read the live tree | Unit test `livelist_pre_reserve_does_not_read_the_live_tree`: two cells with same net live blocks but 2× live tree size have equal `pre_reserve_bytes` for the two livelist arms, while `no_structure` has unequal `destroy_reads` | **Pass** |
| 3. Whether overestimate ratio has an upper bound | `naive n=65536 churn=16` verbatim: `entries=2162688 net_alloc=65536 pre_reserve_bytes=36507222016 true_pending_bytes=2147483648 overestimate=17.0000`; `condense` same cell verbatim: `entries=65536 ... overestimate=1.0000` | `naive` diverges linearly with churn (17× at C=16), **`condense` constant at 1.0000** |
| 4. Positive control | `name=positive_control arm=naive n=8192 entries_c0=8192 entries_c16=270336 ratio=33` | **Does not trigger invalidation**, apparatus distinguishes differences |
| 5. Whether it returns to O(net live blocks) after condense | `condense` across all 12 `share=0.0` cells has `entries` cell-by-cell equal to `net_alloc` (1024 / 8192 / 65536), independent of churn | **Pass** |
| 6. Extra prepayment on release path | Both livelist arms cell-by-cell `free_path_extra_bytes=24`, `no_structure` cell-by-cell `0` | **> 0**, falls under C260 (Prepayment for livelist entries was not counted in the quantity to untie itself) |

⇒ **Arm A passes D6 (Snapshot implementation model) Criterion 5**, but **only under the form with condense**:
The entry count of `naive` grows unbounded with write counts — in the `n=65536 churn=16` cell verbatim `entries=2162688`,
while net live blocks are only 65536, a **33×** factor.

### Two Debts to Settle

1. **Arm A must include condense; "condensable" is not optional**. Without it,
   entry count is on the same order as write counts and unrelated to live block count; while Criterion 5 literally passes (upper bound is still calculable),
   acceptance would reject writes while the disk is still largely empty — in the `n=65536 churn=16` cell, the upper bound reports 36507222016 bytes (34 GiB),
   while true pending deletions are only 2147483648 bytes (2 GiB).
2. **The release path prepays an extra 24 bytes per released block** (type tag 2 + position entry 14 + birth txg 8).
   D3 (Space allocation) settled item 17 states verbatim: "A COW system must guarantee that the 'space release' operation itself does not require allocating space",
   and D3 (Space allocation) settled item 2 resolves this as: "tombstones are part of 'untying itself', counted into that quantity",
   prepaid at write time. **For livelist entries to follow the same path, they must be added to the quantity of 'how much is needed to untie itself'**,
   which was settled by user settlement on 2026-08-31 ⇒ falls under C260 (Prepayment for livelist entries was not counted in the quantity to untie itself).

### Mutations

15 mutations, **all caught**, table in `scripts/mutation/tables/e130_livelist_bounded_destroy.tsv`.

⚠️ **One of them was not caught in Round 1; per `.claude/rules/mutation-sampling.md` tripartite division, it was classified as Category 3
"sampling point insensitive", and sample points were supplemented rather than recording it as an equivalent mutation**: M8 (relaxing condense predicate to always trigger)
differs from the original formula only on `raw_entry_count / net_allocated_block_count ∈ (1, 2]`; in this workload family,
`raw_entry_count / net_allocated_block_count = 1 + 2c` only takes values 1, 3, 9, 33, **unable to reach that interval**;
when `c = 0`, `raw_entry_count` equals `net_allocated_block_count`, so assertions cannot observe a difference.
The remediation was extracting the predicate into `condense_entries(raw_entry_count, net_allocated_block_count)` for standalone testing,
supplementing three sample points (150/100, 200/100, 201/100), catching M8 on the spot.
The fact that the workload family cannot reach that interval is itself archived by unit test `the_load_family_cannot_reach_the_sensitive_ratio`.

### Coverage Range: Covers Only Heads That Have Never Been Snapshotted (Added 2026-09-10)

⚠️ **The apparatus creates no snapshots**: `snap` appears 0 times in source code, workload `Load` has only three quantities: live blocks, overwrite rounds, and inheritance ratio.
⇒ The conclusions that "entry count after condense exactly equals net live blocks" and "pending deletion overestimate is constant at 1.0000" **cover only heads that have never been snapshotted**.
Per the fix of C265 (Release boundary for destroying writable head was written as a fixed clone time), a snapshotted head's livelist retains entries where `birth ≤ previous_snapshot_txg`
that are not released upon destruction; the counter's overestimate of pending deletion occupancy has no upper bound in this case — pointed out by the backward-inference leg in Round 2 of D6 (Snapshot implementation model) settled item 2 three-way adversarial debate: **deduction, not measured**.
Where that batch of entries should land is also undecided; see C269 (Blocks left when destroying writable head have nowhere to go).

### Corrected a Cross-Harness Measurement Basis That Day: The Guard Was Written, but Pinned to the Wrong Source

`ALLOCATION_RECORD_BYTES` was written as **30** in Version 1, with a comment citing D3 (Space allocation) settled item 7,
whereas summing the field table of D3 (Space allocation) settled item 7 segment by segment yields **4 + 6 + 2 + 8 = 20**;
30 is actually the **accounting** entry width (key 22 + value 8) of D5 (Snapshots / space accounting mechanism) settled item 5.
At that time, the cross-harness guard was written as `assert_eq!(ALLOC_REC_BYTES + 0, 22 + 8)` — **it confessed its own origin**,
yet assertions were self-consistent, mutation M2 was caught, and gates were all green, with nothing flagging that this number belonged to another tree.

⇒ This is precisely what `.claude/singlefs-ai-sop/rules/show-me-test.md` described:
"The person patching a pitfall is most likely to stop here: they indeed turned the pitfall into a failing check, **with complete evidence and all gates green**,
and therefore will not reflect once more on how far that check's range reaches". It was caught when the forward-inference leg of the tripartite argument summed the field table of D3 (Space allocation)
segment by segment, not by any assertion.

**Remediation**: Changed constant to 20; changed cross-harness guard from "pinning a total" to **pinning field table segment by segment**
(`ALLOCATION_RECORD_BYTES == device_identity_bytes 4 + placement_slot_bytes 6 + span_segment_bytes 2 + allocation_and_free_generation_bytes 8`),
and added `the_allocation_record_width_is_not_the_ledger_entry_width` to separate and archive the two quantities of the two trees.
**Impact**: Only `worst_batch_bytes` changes (221184 → **180224**, `no_structure` 122880 → **81920**);
entry count, overestimate ratio, and positive control ratio are independent of this constant ⇒ **none of the conclusions for Criteria 1 / 3 / 4 / 5 changed**.
Artifacts were re-run and replaced, passing byte-for-byte verification under `replay.sh`.

### Affected Decisions

| Decision item | Relationship | Review |
|---|---|---|
| D6 (Snapshot implementation model) settled item 3 | Supports | 2026-09-20 unaffected: whether Candidate A (per-head livelist) passes settled item 8 Criterion 5 "bounded destruction" is judged by it, concluding it passes only with condense — that is acceptance evidence for "requiring a second structure, adopting ZFS event stream form". ⚠️ E130 (Does Per-Head Livelist Pass Bounded Destruction) body text wrote "D6 (Snapshot implementation model) settled item 2", which was the numbering on 2026-09-10; destruction structure resides in settled item 3 after 2026-09-13 |
| D3 (Space allocation) settled item 17 | Supports | 2026-09-20 unaffected: the extra bytes prepaid per block released on the release path collide verbatim with the principle that "releasing space does not require allocating space" |
| D3 (Space allocation) settled item 2 | Supports | 2026-09-20 unaffected: for livelist entries to take the tombstone path, they must enter the quantity of "how much is needed to untie itself" — the basis for settled item 2 supplementing the livelist half |
| D5 (Snapshots / space accounting mechanism) settled item 5 | Does not affect | 2026-09-20 unaffected: only borrows accounting entry width as a cross-harness measurement basis control, does not judge accounting key |
| D8 (Core index structure) settled item 5 | Does not affect | 2026-09-20 unaffected: only notes that the "resumable" half relies on intent mechanism, unmodeled by apparatus |

## Historical Versions

None.
