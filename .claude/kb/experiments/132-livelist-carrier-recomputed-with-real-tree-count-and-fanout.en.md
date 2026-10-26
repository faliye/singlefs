## E132 Livelist Carrier: Recomputed with Real Tree Count and Fanout — Run (2026-09-10)

[中文](132-livelist载体按真实树数与内部扇出重算.md) · **English** · [日本語](132-livelist媒体-実ツリー数と内部ファンアウトでの再計算.ja.md)

**What is being asked**: Across the three carriers of D6 (Snapshot Implementation Model) settled item 2 Candidate A — one livelist per head (`per_head_1`), one per data tree (`per_head_t`),
and one shared tree (`shared`) — what the tree table level thresholds and point lookup depths are under **real tree counts** and **real internal fanout**.

**Why establishing a new number**: The figures of E131 (Two Livelist Carriers: Values and Costs) were referenced by the user when expressing preferences, but the backward leg in Round 1 of D6 (Snapshot Implementation Model) settled item 2
three-way adversarial debate hit two modeling errors in it, confirmed by main agent inspection: it counted tree table entries assuming "one data tree per head, no pool-level trees",
and used leaf fanout for internal nodes as well. The criteria of E131 (Two Livelist Carriers: Values and Costs) had been hardcoded and its artifact committed; following the repository's precedent that "experiments with hardcoded criteria and committed artifacts are not retroactively modified, but rerun under a new number", a new number was established.

**Hardcoded before running**: `e132-preregistration.md` (including criteria where triggering one cell yields "`per_head` loses",
and "whether I accept the opposite outcome").

**Re-run command** (`exact` mode, byte-for-byte comparison against committed artifact):

```bash
bash scripts/experiment-apparatus/replay.sh E132
# Run apparatus directly:
cd scripts/experiment-apparatus && cargo run --release --bin e132_livelist_carrier_recount
```

Code: `scripts/experiment-apparatus/e132_livelist_carrier_recount.rs`,
raw output: `e132-livelist-carrier-recount-2026-09-10.out` (single run; swept cells described in "Arms and Measurement Basis").

### This Is a Counting Model, Not an Implementation

No I/O, no random source ⇒ running N times is necessarily byte-for-byte identical; evidence strength derives from **13 unit tests + 14 mutations all caught**.

### Arms and Measurement Basis

| Code | Tree table entries occupied per head | Additional pool-wide footprint |
|---|---|---|
| `per_head_1` | Data trees 2 + livelist 1 = 3 | Pool-level trees 3 |
| `per_head_t` | Data trees 2 + livelist 2 = 4 | Pool-level trees 3 |
| `shared` | Data trees 2 | Pool-level trees 3 + shared livelist 1 |

2 data trees per head and 3 pool-level trees are taken verbatim from "Tree ID Allocation (Envisioned)" in the then-committed `layout/01-first-txn.md` (that line self-labels "envisioned");
81 trees per tree table level (`16253 / 200`: denominator is the sole reading 16384 minus Code 2 header 131 with reserved bits hardcoded in `22-单元原子性怎么合成.md`; entry width is 200 widened by user settlement on 2026-09-16 in D8 (Core Index Structure) settled item 8); child pointer 59 (D22 (Synthesizing Unit Atomicity) settled item 7; ⚠️ pointer width is no longer 59, this cell did not follow suit, see Historical Versions at the end).
Entry width swept {24, 32, 40, 48, 56} — because entry identity must switch to the central mapping key, which remains undecided (C266 (livelist entry identity uses position hint that can expire));
`shared` adds 8 bytes of tree ID to each tier. Head count ∈ {1, 4, 16, 64, 256, 1024, 4096} × live blocks per head ∈ {1024, 65536, 1048576}.

### Conclusion: After Correction, One Tree per Head Still Did Not Lose a Single Cell

Artifact transcribed line-by-line from `e132-livelist-carrier-recount-2026-09-16.out`.

| Criterion hardcoded before run | Artifact | Verdict |
|---|---|---|
| 1. Head count entering level 2 and level 3 across arms (Red verdict: `per_head` enters level 3 at ≤ 1024 heads) | `per_head_1` verbatim `level=2 first_heads=27`, `level=3 first_heads=2187`; `per_head_t` `20` / `1640`; `shared` `39` / `3279`; both `per_head` arms enter level 3 only after 1024 heads | **Not triggered** |
| 2. Point lookup under real internal fanout (Red verdict: at a certain width tier, `per_head_1` is cell-by-cell not shallower than `shared`) | Five width tiers across 21 cells each: `per_head_1_shallower` sequentially 14 / 14 / 16 / 12 / 12, `per_head_1_deeper` all **0** | **Not triggered** |
| 3. Capacity footprint ratio | `shared_over_per_head_1` sequentially 1.3333 / 1.2500 / 1.2000 / 1.1667 / 1.1429, `judged=no` | **Reported only, not judged** (`.claude/rules/fs-design.md` "competing on economy constitutes no criterion whatsoever") |
| 4. Positive control | Three arms `entries_h1` → `entries_h4096`: 6 → 12291, 7 → 16387, 6 → 8196 | **Does not invalidate** |

⇒ **After correction, the "violent" cost is: tree table level threshold drops to about 2/3 of shared tree** (level 2 at 27 vs 39 heads),
rather than the "halved" (41 vs 81) reported in E131 (Two Livelist Carriers: Values and Costs).
The benefit of one tree per head on point lookup is even slightly larger than reported in E131 (Two Livelist Carriers: Values and Costs) — internal fanout is smaller than leaf fanout, so the extra entries of shared tree must traverse more levels.

### Three Unmodeled Aspects

1. **Snapshots not counted**: D8 (Core Index Structure) settled item 8 states verbatim "snapshots enter tree table"; how many entries a snapshot occupies is undecided today ⇒ tree table threshold will only be reached earlier than reported here.
2. **Entry width can only be swept**: see C266 (livelist entry identity uses position hint that can expire).
3. **Where the counter lives and which tree table entry a "head" corresponds to**: see C267 (a writable head has no object in the tree table that can be pointed to).

### Mutations

14 mutations, **all caught**, table in `scripts/mutation/tables/e132_livelist_carrier_recount.tsv`.
⚠️ In Round 1, M9 (shared arm key omitting tree ID) was not red, classified under `.claude/rules/mutation-sampling.md` as **Category 3 sampling point insensitivity**:
The only cell pinning absolute value happened to be 4 levels under both width 24 and width 32. Supplemented a sampling point spanning a two-level boundary — the entries accommodated across two levels is 680 × 196 = 133280 at width 24,
and 510 × 179 = 91290 at width 32; at 100k entries, one side has 2 levels and the other has 3 levels — catching M9 on the spot.

### Affected Decisions

| Decision item | Relationship | Review |
|---|---|---|
| D6 (Snapshot Implementation Model) settled item 3 | Supports | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D6 (Snapshot Implementation Model) settled item 2 | Supports | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D8 (Core Index Structure) settled item 8 | Preparation | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D22 (Synthesizing Unit Atomicity) settled item 7 | Does not affect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |

## Historical Versions

### 2026-09-16
- Formerly: tree table 134 trees per level (`16284 / 121`), Criterion 1 thresholds `per_head_1` 44 / 5985, `per_head_t` 33 / 4489, `shared` 66 / 8977; conclusion "level 2 at 44 vs 66 heads", E131 (Two Livelist Carriers: Values and Costs) control "68 vs 134", artifact `e132-livelist-carrier-recount-2026-09-10.out`.
- Now: denominator 16253, entry 200, 81 trees per level; thresholds 27 / 2187, 20 / 1640, 39 / 3279; at 4096 heads all three arms enter level 3; Criterion 1 still untriggered, "about 2/3" unchanged; artifacts for Criteria 2 through 4 unchanged by a single row. Artifact replaced by `e132-livelist-carrier-recount-2026-09-16.out`, 14 mutations all caught (replacement values for M4 / M5 adapted to be sensitive at the 200-byte tier).
- Rationale: user pointed out on 2026-09-16 that denominator of E131 (Two Livelist Carriers: Values and Costs) was an old value; E132 (Livelist Carrier: Recomputed with Real Tree Count and Fanout) calculates the same quantity and is the designated current state on the E131 (Two Livelist Carriers: Values and Costs) page; modifying only one would create a new contradiction on this quantity.
- ⚠️ What did not follow suit: child pointer 59 is the 2026-09-10 pointer width (changed after D19 (Block Pointer Structure & Width Budget) settled item 7), which enters internal fanout and Criterion 2 descent levels; that is a separate quantity, left untouched in this round.
