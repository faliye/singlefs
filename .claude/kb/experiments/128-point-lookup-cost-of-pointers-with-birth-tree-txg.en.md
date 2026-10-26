## E128 Point Lookup Cost of Pointers with Birth (tree, txg) —— Run (2026-09-10, 15 unit tests / 14 mutations caught; **accounting pair was directional across four rounds, inode pair unstable across four rounds**)

[中文](128-指针带出生树txg之后的点查代价.md) · **English** · [日本語](128-出生-ツリー-txg-付与ポインタのポイント検索コスト.ja.md)

**What it answers is the debt owed by D19 (Block Pointer Structure & Width Budget) settled item 7**: Candidate A expands pointer header 31 → 47,
concomitantly widening inode tree internal entries 93 → 109 and accounting tree internal entries 81 → 97; **how much additional CPU time per point lookup does this step incur**.
User settlement judged that cell as "leave unsettled for now, undergo another round", rationale verbatim:
"one cost of A (E20 (CPU Cache Cost of Widening Pointers) measured 67-byte entry on 16 KiB node at 2.22×)
**has never entered the materials of any leg**".
E128 (Point Lookup Cost of Pointers with Birth (tree, txg)) **only provides costs, not judgments** —— judgments must undergo tripartite review
(`.claude/rules/three-way-inference.md`).

### Why Establishing a New Number Instead of Adding Arms to E20

Criteria of E20 (CPU Cache Cost of Widening Pointers) were hardcoded on 2026-08-29, artifact committed, `replay.sh` registered;
identical rationale to establishing E127 (Whether to Store Group Identity Under Splits and Merges) when E95 (Aging Node Layout Arm) lacked arms.

**Furthermore, E20 (CPU Cache Cost of Widening Pointers) did not measure this exact item**: its tiers were 16 / 24 / 32 / 40 / 67 / 111,
whereas this project's real pairs are 93 → 109 and 81 → 97, **neither tier being present among them**.
Substituting those ratios of 67 and 111 amounts to answering this question with figures of a different quantity.

### Criteria, Thresholds, Invalidation Clauses (Written Before Any Measurement Code)

Original document `e128-preregistration.md`, unchanged by a single word after run. Four clauses:
① Positive controls for each of the six arms (L1 tier faster than DRAM tier by ≥ 2×), failure on any cell invalidates entire round;
② **Cross-apparatus gate** —— `xdev_67` and `xdev_111` must land back on the two numbers of E20 (CPU Cache Cost of Widening Pointers) committed artifact (difference ≤ 15%);
③ Absolute value assertions, pinning slot counts and tree depths tier by tier to prevent "all six arms erring together";
④ Run 5 rounds; asserting "A is slower" requires it to be slower across all 5 rounds, asserting "not slower" requires not slower across all 5 rounds; if neither holds, record "unstable".

⚠️ **Explicitly did not establish an 'acceptable slowdown' threshold before run**: that number belongs to judgment, not experiment
(`.claude/singlefs-ai-sop/rules/test-discipline.md` "Failure clauses must not make conclusions unfalsifiable").
⚠️ **Explicitly stated acceptance of opposite outcome before run**: original verbatim states "If `inode_jia` is **faster** than `inode_bing`, or difference is **< 2%**:
I will record that '**this cache cost of A does not hold on 16 KiB nodes**'... **this is an outcome I accept, not an outcome to avoid**".
**The actual run landed precisely in this cell.**

### Six Arms

| Arm | Entry Width | What It Is |
|---|---|---|
| `inode_bing` | 93 | Candidate C = today: separator key 8 + identity reference 26 + child pointer 59 (`format-const: INODE_INTERNAL_ENTRY`) |
| `inode_jia` | 109 | Candidate A: child pointer 59 → 75 |
| `ledger_bing` | 81 | Today: accounting key 22 + child pointer 59 |
| `ledger_jia` | 97 | Candidate A |
| `xdev_67` | 67 | Cross-apparatus control, measured by E20 (CPU Cache Cost of Widening Pointers) |
| `xdev_111` | 111 | Cross-apparatus control, measured by E20 (CPU Cache Cost of Widening Pointers) |

Node header constant taken as 64, identical figure to E20 (CPU Cache Cost of Widening Pointers); it simultaneously reproduces the two registered fanouts of this project
((16384 − 64) / 93 = 175, (16384 − 64) / 81 = 201), pinned by unit test
`published_fanouts_come_out_of_the_same_header_constant`.

### Actual Run (Artifact `e128-pointer-birth-cost-2026-09-10.out`, 134 Lines, `emitted=134`)

**Conclusion 1: Inode tree pair (93 → 109) is unstable, E128 cannot answer this cell.**
Ran **four full rounds**: first three judged "A is never slower", fourth judged "A is slower every round", in opposite directions.
**Round-by-round table** (add rows when replay flips direction):

| Round | Artifact | inode Ratio | Verdict |
|---|---|---|---|
| 1 | Unpreserved (source before removing `tree_per_level`) | 0.9686 | `0/5 jia_never_slower` |
| 2 | Unpreserved (`replay.sh` verification round, same source as Round 1) | 0.9655 | `0/5 jia_never_slower` |
| 3 | `e128-pointer-birth-cost-2026-09-10.out` | 0.9764 | `0/5 jia_never_slower` |
| **4** | `e128-pointer-birth-cost-2026-09-10-rerun4.out` | **1.0885** | **`5/5 jia_slower_every_round`** |

⇒ Per `.claude/singlefs-ai-sop/rules/test-discipline.md` "To claim 'pass' it must pass every round, to claim 'fail' it must fail every round... if neither is satisfied, record 'unstable' faithfully without drawing a conclusion" —— **this cell records 'unstable', drawing no conclusion.**

Artifacts verbatim for Round 3 and Round 4:

```
E7RESULT name=verdict pair=inode bing_median=586.89 jia_median=572.95 ratio_median=0.9764 ratio_min=0.9750 ratio_max=0.9771 rounds_jia_slower=0/5 verdict=jia_never_slower
E7RESULT name=verdict pair=inode bing_median=612.09 jia_median=667.30 ratio_median=1.0885 ratio_min=1.0283 ratio_max=1.1017 rounds_jia_slower=5/5 verdict=jia_slower_every_round
```

⚠️ **An unverified candidate explanation, recorded without crediting**: Round 4 whole machine ran slower —— cross-apparatus gate two tiers were 4.9% and 5.9% higher than baseline of E20 (CPU Cache Cost of Widening Pointers)
(first three rounds were −2.4% to +2.6%), while working set of `inode_jia` (929 MB) was larger than `inode_bing` (790 MB),
and the machine had other sessions running builds at the time. **This is merely a candidate**: E128 did not log machine workload at the time,
so the hypothesis that "whole-machine slowdown caused larger working set arm to suffer more" **can neither be proven nor disproven**. Recorded as placement C252 (inode pair of E128 unstable, apparatus did not isolate load).

⚠️ **This reversal was caught by byte-for-byte comparison, not interval assertions**: `strip_timing` in `replay.sh` intentionally **does not erase**
`verdict=` and `rounds_jia_slower=` fields (they are conclusions, not timing) ⇒ verdict flipping judged red at structural comparison stage.
Had they been erased together, Round 4 would have passed green claiming "timing differs only, structure matches, conforming to declaration".

**Conclusion 2: Accounting tree pair (81 → 97) was directional across all four rounds, A slower every round**, ratios 1.0179 / 1.0454 / 1.0210 / 1.0357.
⇒ This point **stands** under the same discipline: all four rounds judged the identical direction. Artifact verbatim (Round 3):

```
E7RESULT name=verdict pair=ledger bing_median=562.78 jia_median=574.61 ratio_median=1.0210 ratio_min=1.0153 ratio_max=1.0389 rounds_jia_slower=5/5 verdict=jia_slower_every_round
```

**Conclusion 3: Cross-apparatus gate passed across all four rounds**, deviations being −2.4% / +2.6%, −1.9% / +3.0%, −0.2% / +1.4%, **+4.9% / +5.9%**.
⚠️ **It passed, but was simultaneously the sole artifact recording 'Round 4 whole-machine slowdown'** —— 15% tolerance was sufficiently wide for this cell,
wide enough to pass a 5–6% whole-machine drift that occurred in the identical round where inode pair flipped direction. Artifact verbatim (Round 3):

```
E7RESULT name=xdev entry_bytes=67 e20_ref=466.99 e128_median=466.02 deviation=0.0021 tol=0.15 ok=true
E7RESULT name=xdev entry_bytes=111 e20_ref=547.50 e128_median=555.26 deviation=0.0142 tol=0.15 ok=true
```

**Conclusion 4: Derived numbers concomitantly modified by Candidate A, the pure arithmetic half.** Artifact verbatim:

```
E7RESULT name=derived arm=bing child_ptr=59 inode_entry=93 ledger_entry=81 root_record=194 tree_entry=121
E7RESULT name=derived arm=jia child_ptr=75 inode_entry=109 ledger_entry=97 root_record=226 tree_entry=137
```

Positive controls across all six arms passed, ratios 7.81 – 10.45.

### ⚠️ Third Round Artifact Contains a Non-Monotonicity, Exhibited Verbatim Without Explanation

In Round 3, at identical 16 KiB node size and tree depth 4, 93-byte entry (586.89 ns) **was slower than 109-byte (572.95 ns)
and 111-byte (555.26 ns)**; **in Round 4 this non-monotonicity vanished** (612.09 vs 667.30). E20 (CPU Cache Cost of Widening Pointers) column rose monotonically between 67 → 111, whereas here it did not.
An unverified hypothesis is interaction between 16384-byte node alignment, entry stride, and cache associativity.
⚠️ **It does not reproduce across rounds, hence is not of the same nature as the 8 KiB anomaly in E20 (CPU Cache Cost of Widening Pointers)** ——
that one was stable across six rounds, whereas this one appeared three times across four rounds and vanished in Round 4. **Side-by-side citation is forbidden.**

### Explicitly Stated What Cannot Be Measured

| Cannot Measure | Why Written Here |
|---|---|
| Scan, range query, write path | Only tests point lookup |
| Other node tiers | Only tests 16 KiB, the figure pinned by D8 (Core Index Structure) settled item 2 |
| Trees per level in tree table | **E128 does not report this number**, only entry width 121 → 137 (pure field table summation, both measurement bases agree). Denominator has two in repository: D22 (Synthesizing Unit Atomicity) verbatim "(16384−68−32)/67 = 16284/67 = 243" uses 16284, whereas `NODE_BYTES − NODE_HEADER_BYTES` is 16320. ⚠️ **Both yield 134 at 121, differing by 1 only at 137 (118 vs 119)** —— category 3 "sampling point insensitivity" per `.claude/rules/mutation-sampling.md`; matching on baseline prevents anyone auditing the other cell. Unit test `the_two_tree_table_denominators_agree_at_121_and_disagree_at_137` pins this, recorded as placement C148 (Wider tree table entries compress indirection threshold to two layers) (merged from C157 on 2026-10-11) |
| Tree table point lookup | Tree table lookup frequency is orders of magnitude lower than inode tree, lacking timing arms |
| Device I/O | Pure CPU / memory model. **Resident memory of internal nodes is also unmeasured**: fanout 175 → 149 occupying 1.17× resident memory for identical leaf set is arithmetic, not E128 observation |

⇒ **Citing Conclusion 1 requires citing this table together.** Scope of "A is not slower" is "point lookup CPU time on 16 KiB nodes",
not "A has no overhead".

### This Model Is a Hand-Copied Duplicate of E20, Guarded by Cross-Apparatus Gate

The three sections `Tree` / `generate_lookup_keys` / `bench` share origin with `e20_fanout.rs`, without merging into one.
Rationale is mutation table of E20 (CPU Cache Cost of Widening Pointers) modifies its file by **exact text match**;
removing those three sections would cause the 5 mutation proofs committed in E20 to mismatch immediately.
Cost was hand-copying, backstop was cross-apparatus gate. ⚠️ **Gate pins only those two points on DRAM tier**; copying omissions on other tiers remain silent,
recorded as placement C249 (two point lookup models hand-copied same layout code, gate pins only two points).

### Which Statements Among Four Rounds Can Be Cited and Which Cannot

| Can Be Cited | Cannot Be Cited |
|---|---|
| In accounting tree pair A is slower (directional across four rounds, 1.0179 – 1.0454) | **Any direction or ratio for inode tree pair** —— three rounds said faster, one said slower |
| Derived widths concomitantly altered by A (pure arithmetic: 93→109 / 81→97 / 194→226 / 121→137) | The statement "A suffers no penalty on 16 KiB nodes" |
| Cross-apparatus gate passed across all four rounds | "Cross-apparatus gate passed ⇒ numbers from that run can be compared" —— Round 4 also passed, yet its verdict reversed previous three rounds |

### Measurement Basis and Replay

Pure CPU / memory experiment, implicit B-tree laid out as array, binary search within node, entries strided by entry width,
entry second 8 bytes storing child node index (forming dependent load chain). **Fixed key count, non-fixed working set** ——
wider entries ⇒ larger working set ⇒ evicted from cache, which is precisely the question asked.
Local per-core L1d 48 KiB, L2 1 MiB, L3 32 MiB, cache line 64 bytes.
Working set four tiers 2¹¹ / 2¹⁵ / 2¹⁹ / 2²³ keys, 5 rounds per tier, taking minimum of 3 trials per round, `iters = 2000000`.

Code `scripts/experiment-apparatus/e128_pointer_birth_cost.rs`
(`cargo run --release --bin e128-pointer-birth-cost -- 2000000`),
raw output `e128-pointer-birth-cost-2026-09-10.out` (Round 3, committed copy)
and `e128-pointer-birth-cost-2026-09-10-rerun4.out` (Round 4, reversed copy).
First two rounds ran source prior to removing `tree_per_level`, **artifacts unpreserved**.

```bash
bash scripts/experiment-apparatus/replay.sh E128
```

⚠️ **Replay of E128 will judge red intermittently; this is intentional, not noise.**
`verdict=` of inode pair participates in byte-for-byte structural comparison, and being inherently unstable ⇒ **flips direction and judges red**.
When judging red, the proper response is **not** widening tolerance, but: adding newly run ratio and verdict **into round-by-round table in Conclusion 1 of E128**
(turning four rounds into five, six...), then deciding whether to replace committed artifact.
**Every red is a new observation** —— for an experiment recording 'unstable' in a cell, this is precisely expected replay behavior.
Erasing those two fields to keep it permanently green produces a version that proves nothing.

Interval assertions in `replay.sh` pin **those stable across four rounds**: cross-apparatus gate both cells `ok=true`;
accounting pair ratio falls within (1.00, 1.10). **Inode pair does not pin direction** —— three rounds in one direction, one opposite;
pinning either side writes an unstable observation as an assertion. ⚠️ Structural comparison still compares `verdict=` and
`rounds_jia_slower=` verbatim (`strip_timing` intentionally retains both) ⇒ **verdict flipping again will still judge red on replay.**

### Mutations

`scripts/mutation/tables/e128_pointer_birth_cost.tsv`, all 14 caught, with zero blind spots.
Among them M6 (reverting `inode_jia` arm back to 93) was supplemented: without unit test
`arms_are_the_projects_real_entry_widths`, substituting tested object did not turn a single test red.

### Influenced Decisions

| Decision Item | Relationship | Review |
|---|---|---|
| D19 (Block Pointer Structure & Width Budget) settled item 7 | Support | 2026-10-17 unaffected: only synchronized phrasing, artifact pointers, paths, and numbers; conclusions, numbers, and artifacts unchanged; previously 2026-10-11 unaffected: only phrasing modified (time words, position designations, self-reference); conclusions, numbers, and artifacts unchanged |
| D19 (Block Pointer Structure & Width Budget) settled item 8 | Support | 2026-10-17 unaffected: only synchronized phrasing, artifact pointers, paths, and numbers; conclusions, numbers, and artifacts unchanged; previously 2026-10-11 unaffected: only phrasing modified (time words, position designations, self-reference); conclusions, numbers, and artifacts unchanged |
| D8 (Core Index Structure) | No effect | 2026-10-17 unaffected: only synchronized phrasing, artifact pointers, paths, and numbers; conclusions, numbers, and artifacts unchanged; previously 2026-10-11 unaffected: only phrasing modified (time words, position designations, self-reference); conclusions, numbers, and artifacts unchanged |
| D22 (Synthesizing Unit Atomicity) | No effect | 2026-10-17 unaffected: only synchronized phrasing, artifact pointers, paths, and numbers; conclusions, numbers, and artifacts unchanged; previously 2026-10-11 unaffected: only phrasing modified (time words, position designations, self-reference); conclusions, numbers, and artifacts unchanged |

## Historical Versions

Historical entries for E128 (Point Lookup Cost of Pointers with Birth (tree, txg)) are consolidated in [experiments-history.md](../experiments-history.md).
