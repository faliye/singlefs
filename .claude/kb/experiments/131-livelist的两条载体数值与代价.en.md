## E131 Two Livelist Carriers: Values and Costs — Run (2026-09-10)

[中文](131-livelist的两条载体数值与代价.md) · **English** · [日本語](131-livelist的两条载体数值与代价.ja.md)

**What is being asked**: Between the two carriers of D6 (Snapshot implementation model) settled item 2 Candidate A — "one private tree per head" versus "one shared tree with keys prefixed by head tree ID" —
what each pays and what each gains. The form of Arm A was confirmed in Round 2 to preserve the verbatim ZFS original form of D5 (Snapshots / space accounting mechanism); **here we only compare carriers**.

**Why it needed to be run**: The user commented verbatim on "one private tree per head": "**This feels a bit violent. Why don't you look at the numbers and performance?
If it's good, it's not unacceptable**". The form follows the precedent of D3 (Space allocation) settled item 7 (user settlement + E97 (Entry Encoding for Accounting and Allocation Records)).

**Hardcoded before running**: `e131-preregistration.md` (including criteria where triggering one cell yields "`per_head` loses",
and "whether I accept the opposite outcome").

**Re-run command** (`exact` mode, byte-for-byte comparison against committed artifact):

```bash
bash scripts/experiments/replay.sh E131
# Run apparatus directly:
cd scripts/experiments && cargo run --release --bin e131_livelist_carrier
```

Code: `scripts/experiments/e131_livelist_carrier.rs`,
raw output: `e131-livelist-carrier-2026-09-10.out` (single run; swept cells described in "Arms and Measurement Basis").

### This Is a Counting Model, Not an Implementation — It Proves Nothing

No I/O, no concurrency, no cache, no random source ⇒ running N times is necessarily byte-for-byte identical;
"consistent across N rounds" demonstrates the absence of hidden state, not statistical stability. Evidence strength derives from **19 unit tests + 15 mutations all caught**.
**What it does not answer**: condense cost (both arms are event streams, identical item), cache hits, real I/O.

### Arms and Measurement Basis

| Code | Carrier | Entry |
|---|---|---|
| `per_head` | One private tree per head, created alongside head on first clone | Type tag 2 + position entry 14 (D19 (Block Pointer Structure & Width Budget) settled item 4) + birth txg 8 = **24** |
| `shared` | One shared tree, keys prefixed by head tree ID | 24 + tree ID 8 (D8 (Core index structure) settled item 8) = **32** |

Nodes 16384 bytes, node header 64; tree table entry 200 bytes (D8 (Core index structure) settled item 8, widened by user settlement on 2026-09-16);
**denominator for trees per tree table level takes 16253** (the sole reading hardcoded in `22-单元原子性怎么合成.md`: 16384 minus Code 2 header 131 with reserved bits),
⇒ **81** trees per level;
The retired denominator 16284 happens to also give 81 at the 200-byte tier; unit test `the_retired_denominator_agrees_at_200_bytes_and_diverges_at_148` archived this point.
Swept cells: head count ∈ {1, 4, 16, 64, 256, 1024, 4096} × live blocks per head ∈ {1024, 65536, 1048576}.

### Conclusion: `per_head` Did Not Lose a Single Cell Across the Five Hardcoded Cells

Artifact transcribed line-by-line from `e131-livelist-carrier-2026-09-16.out`.

| Criterion hardcoded before run | Artifact | Verdict |
|---|---|---|
| 1. Tree table levels (Red verdict: three levels at `H ≤ 64` ⇒ `per_head` loses) | `name=criterion1 arm=per_head level=2 first_heads=64`, `level=3 first_heads=4096`; precise thresholds pinned by unit tests: level 2 at **41** heads, level 3 at **3281** heads; at H = 64 it is two levels, red condition's three levels not met | **Not triggered** |
| 2. Point lookup descents (Red verdict: cell-by-cell equal between both arms ⇒ `per_head` loses) | `per_head` is shallower across **12 of 21 cells**: shallower by 1 level in 11 cells, and in cell `heads=4096 blocks_per_head=65536` verbatim `per_head_descents=2 shared_descents=4 delta=2`; **0 cells** deeper | **Not triggered**, `per_head` wins |
| 3. Destruction touches | Cell-by-cell equal between both arms (both rows `destroy_touch=65536` at `heads=1024 blocks_per_head=65536`) | Pre-judged before run as "this cell lacks discriminative power" |
| 4. Capacity footprint (Red verdict: `shared` not higher than `per_head`) | Same cell verbatim `per_head ... ppm=92.2`, `shared ... ppm=122.9`, constant **4/3** across all cells | **Not triggered**, `per_head` saves 1/4 |
| 5. Pre-batch reservation (Red verdict: `per_head` also requires range scan) | `per_head ... pre_reserve_range_scan=0`, `shared ... pre_reserve_range_scan=1` | **Not triggered** |
| 6. Positive control | `name=positive_control arm=per_head tt_entries_h1=2 tt_entries_h4096=8192 ratio=4096` | **Does not invalidate** |

### Measuring the Magnitude of That "Violent" Cost

`per_head` occupies **2** tree table entries per head (its data tree + its livelist), whereas `shared` occupies 1 per head plus 1 pool-wide
⇒ **the level thresholds of the tree table are halved**. Calculated at 81 trees per level (arithmetic):

| | Level 2 threshold | Level 3 threshold |
|---|---|---|
| `per_head` | 41 heads (pinned by unit test) | 3281 heads (pinned by unit test) |
| `shared` | 81 heads | 6561 heads |

One extra tree table level = one extra indirection whenever finding a tree's root from the root record (D22 (Synthesis of unit atomicity) settled item 7: root record constantly traverses the tree table unit indirection layer).

### Three Unmodeled Aspects to Bear in Mind

1. **`per_head`'s "pre-batch reservation does not require range scanning" is recorded as 0 in the model, but where that counter resides is unmodeled**.
   A btree does not provide its entry count for free: it must either live within the 76-byte reservation of the tree table entry, or establish a separate accounting statistic with a tree dimension —
   establishing a separate accounting statistic with a tree dimension would reopen the 2026-09-06 settlement that withdrew per-tree byte statistics. **If this cell remains undecided, the win on Criterion 5 remains only on paper.**
2. **Destruction**: both arms are equal by entry count; however, `per_head` destruction can drop the entire tree along with its tree table entry,
   while `shared` can only perform range deletion by prefix. The model does not distinguish this — hardcoded before run as "lacking discriminative power", not reversed post hoc.
3. condense, caching, and real I/O are absent from the model.

### Mutations

15 mutations, **all caught**, table in `scripts/mutation/tables/e131_livelist_carrier.tsv`.
⚠️ Two were not red in Round 1, **handled separately per `.claude/rules/mutation-sampling.md`**:
M8 (`per_head` point lookup calculated pool-wide) was a **true blind spot** — both descent assertions at that time only compared arms against each other;
when all arms erred together, relative comparison saw nothing, so absolute-value assertions were added;
M13 (capacity footprint not scaling with entry count) was **Category 3 sampling point insensitivity** — the sole absolute-value assertion was pinned at the baseline entry count of E97 (Entry Encoding for Accounting and Allocation Records),
where the scaling factor happened to be exactly 1, so a sample point deviating from baseline was added.

### This Modeling Was Corrected by E132 (Livelist Carrier: Recomputed with Real Tree Count and Fanout) (2026-09-10)

⚠️ **Figures in the conclusion table are artifacts of this apparatus on an erroneous model; current status is governed by E132 (Livelist Carrier: Recomputed with Real Tree Count and Fanout).**
Errors lay in two places, both struck by the backward-inference leg in Round 1 of D6 (Snapshot implementation model) settled item 2 three-way adversarial debate and verified by main agent inspection:
① **Tree counts** — the `per_head` arm wrote `tree_table_entry_count = heads * 2` (one data tree + one livelist per head) and `shared` wrote `heads + 1`,
neither including pool-level trees; whereas the committed `layout/01-first-txn.md` under "Tree ID Allocation (Envisioned)" envisioned "extent tree 1, inode tree 2, allocation record tree 3, accounting tree 4, central mapping tree 5"
⇒ at least two data trees per head, plus three pool-level trees. After correction, the level 2 threshold is 27 / 39 heads, not 41 / 81 (both calculated at 81 trees per level today).
② **Internal fanout** — `fanout()` used leaf entry width across all levels, whereas internal node entries are keys plus 59-byte child pointers.
③ Additionally, **the criterion itself violated rules**: Criterion 4 framed capacity footprint as winning/losing, whereas `.claude/rules/fs-design.md` explicitly mandates "competing on economy constitutes no criterion whatsoever",
falling under C268 (Criterion table framed economy comparison as win/loss).
⚠️ What the user saw when expressing carrier preference was 68 / 134 from this report; corrected numbers have been submitted to the user for re-judgment.

### Affected Decisions

| Decision item | Relationship | Review |
|---|---|---|
| D6 (Snapshot implementation model) settled item 2 | Preparation | 2026-10-11 unaffected: wording only (temporal terms, positional references, self-references); conclusions, numbers, and artifacts unchanged |
| D5 (Snapshots / space accounting mechanism) | Does not affect | 2026-10-11 unaffected: wording only (temporal terms, positional references, self-references); conclusions, numbers, and artifacts unchanged |
| D3 (Space allocation) settled item 7 | Does not affect | 2026-10-11 unaffected: wording only (temporal terms, positional references, self-references); conclusions, numbers, and artifacts unchanged |
| D8 (Core index structure) | Does not affect | 2026-10-11 unaffected: wording only (temporal terms, positional references, self-references); conclusions, numbers, and artifacts unchanged |
| D19 (Block Pointer Structure & Width Budget) | Does not affect | 2026-10-11 unaffected: wording only (temporal terms, positional references, self-references); conclusions, numbers, and artifacts unchanged |
| D22 (Synthesis of unit atomicity) | Does not affect | 2026-10-11 unaffected: wording only (temporal terms, positional references, self-references); conclusions, numbers, and artifacts unchanged |

## Historical Versions

### 2026-09-16
- Formerly: denominator 16284, tree table entry 121 bytes, 134 trees per level; level 2 / 3 thresholds per_head 68 / 8979 heads, shared 134 / 17956 heads; Criterion 1 artifact `level=2 first_heads=256`, `level=3 first_heads=none_in_sweep`, artifact `e131-livelist-carrier-2026-09-10.out`.
- Now: denominator 16253 (the sole reading hardcoded in D22 (Synthesis of unit atomicity)), entry 200 bytes (widened by user settlement on 2026-09-16 in D8 (Core index structure) settled item 8), 81 trees per level; thresholds per_head 41 / 3281 heads, shared 81 / 6561 heads; Criterion 1 artifact `level=2 first_heads=64`, `level=3 first_heads=4096`, still untriggered; artifact replaced by `e131-livelist-carrier-2026-09-16.out`, 15 mutations all caught (replacement values for M1 / M2 adapted to be sensitive at the 200-byte tier).
- Rationale: user pointed out on 2026-09-16 that denominator was still the old value; entry width changed along with the same quantity; updating only the denominator would yield a mix-and-match model.
