## E119 Padding Cost of Fixed-Width Slots —— Run (2026-09-08, pure arithmetic, 11 unit tests / 12 mutations caught)

[中文](119-槽定宽的补齐代价.md) · **English** · [日本語](119-スロット固定幅化のパディングコスト.ja.md)

Answers how much padding cost must be paid for D27 (Small-Data Packing Container) settled item 8 ④ (fixed-width slots), and **how finely tiers should be partitioned**.

### Why It Exists

D27 (Small-Data Packing Container) settled item 8 ④ (2026-09-08 user settlement) chooses fixed-width slots: a container declares a slot width `W`,
all slots within the container are of identical width, and objects are padded to `W`. ⇒ Padding waste returns according to tier width.
**Coarsely partitioned tiers result in high waste, while finely partitioned tiers lead to many concurrently open containers**, and this curve was never measured before E119 (Padding Cost of Fixed-Width Slots).

### Six Arms × Three Distributions

Arms: `var` (variable-length slots greedily packed in ascending size order, **not a candidate** —— settled item 8 ④ already settled not taking variable length, it serves as a baseline),
`gran64` (64-byte granularity, 64 tiers), `gran256` (256-byte granularity, 16 tiers),
`pow2` (powers of 2, 7 tiers), `tier4` (512 / 1024 / 2048 / 4096, 4 tiers), `single` (single tier 4096).

Distributions: `uniform` (1..4096 equal weights), `logunif` (weight ∝ 1/s, more small files),
`discrete` (only at three points 512 / 1024 / 4096, aligned with sampling points of E114 (Overall Accounting of Small-Data Packing Containers) / E116 (Packed Container Balance: Metadata Writes and Compaction Strategy)).
⚠️ **The repository has zero measured object size distributions** (three columns in smallfile row of D25 (Target Workload Priority) are all "unmeasured",
C174 (Three columns of smallfile row unmeasured)) ⇒ report all three distributions, **without picking one as representative**.

### Conclusion 1: Fixed Width Itself Is Not Costly, What Is Costly Is Coarse Partitioning

Loss relative to `var` (N = 100 000):

| Arm | Tiers | `uniform` | `logunif` | `discrete` |
|---|---|---|---|---|
| `gran64` | 64 | **+1.76%** | **+9.61%** | +0.02% |
| `gran256` | 16 | +6.69% | +37.31% | +0.02% |
| `pow2` | 7 | +38.86% | +31.85% | +0.02% |
| `tier4` | 4 | +39.77% | **+90.83%** | +0.02% |
| `single` | 1 | +109.63% | +797.36% | +121.56% |

⇒ **One reverse clause hardcoded before run triggered**: "If `tier4` loss > 30% on any distribution ⇒ four tiers are too coarse, tiers must be finer",
on `logunif` it is 90.83%, on `uniform` 39.77%, **both crossing the line**.

**Four clauses not triggered**: `gran64` loss < 1% holds only on `discrete` (1.76% / 9.61% on the other two distributions) ⇒
cannot write "fine tiers are equivalent to variable length"; `single` gain is constant 7.00, not lower than 2;
every fixed-width arm having loss > 30% on every distribution does not hold (`gran64` all three cells below 10%) ⇒
**D27 (Small-Data Packing Container) settled item 8 ④ does not need reopening**; any tier `cap` < 4 does not hold (smallest tier is 4096, `cap` = 7).

### Conclusion 2: Tiers Must Be Dense at the Small End, Not Equidistant

On `logunif`, **`pow2` (7 tiers) beats `gran256` (16 tiers)**: 31.85% vs 37.31%.
Fewer tiers turn out better because `pow2` has 64 / 128 / 256 three tiers at the small end where mass actually concentrates;
the finest tier of `gran256` is 256, padding all small objects to 256.
⇒ **Tier partitioning must follow the mass distribution of the distribution, not equidistant steps.** Assertion `main_absolute_logunif_and_discrete` guards this.

### Conclusion 3: Fixed Width Costs Almost Nothing on the Three Settled Sampling Points

The four candidate arms in `discrete` column are all +0.02% —— because points 512 / 1024 / 4096 **happen to be exact tier widths**.
⚠️ **This is precisely the shape of sampling point insensitivity**: looking only at the three sampling points used by
E114 (Overall Accounting of Small-Data Packing Containers) / E116 (Packed Container Balance: Metadata Writes and Compaction Strategy)
would lead to the false conclusion that "fixed width costs nothing".

### Mutation M8 Is a True Blind Spot

M8 degenerates `logunif` distribution into `uniform`, and in the first round **did not turn a single test red** —— all assertions pinning absolute values at the time were only on
`uniform`, with zero assertions on the entire `logunif` distribution, even though the heaviest conclusion figure (`tier4` loss 90.83%) sat right on it.
Triaged per `.claude/rules/mutation-sampling.md`: **not an equivalent mutation, nor sampling point insensitivity, but a true blind spot**.
Caught immediately after adding `main_absolute_logunif_and_discrete`, 12/12 with zero blind spots.

### One Modeling Error Caught on the Spot by Positive Control Hardcoded Before Run

`var` arm in first revision was calculated per "perfect packing" (Σ slot length ÷ container net payload rounded up) —— which equals **allowing a slot to span containers**.
Positive control (when objects exactly equal tier width, fixed width must be byte-for-byte identical to variable length) judged false: at all 512 bytes it gave 1700 containers,
whereas 58 slots × 1725 = 100 050, the correct answer being 1725. Control passed after changing to greedy packing in ascending size order.
Both models are recorded in source code, forbidding altering the registered one retroactively.

### What It Cannot Answer

Pure arithmetic ledger: **no implementation, no I/O, no crash-point replay**. Does not answer wall clock, does not answer cache, does not answer concurrency.
`var` is **one specific packing method** for the variable-length path (ascending size greedy), not the optimal packing method —— bin packing optimality is NP-hard.
**Object size distributions are self-defined**, with zero measured distributions repository-wide.
**Does not answer absolute value of "concurrently open containers"** —— that requires multiplying tree count, and repository lacks upper bound on tree count (analogous to C106 (upper bound on open container count)).
Does not answer "whether tier table enters format" —— that question is answered by D27 (Small-Data Packing Container) settled item 12: does not enter format, format tier only commits container header declaring slot width and slot count.

### Measurement Basis and Artifact

Code `scripts/experiment-apparatus/e119_slot_tiers.rs` (`cargo run --release --bin e119-slot-tiers`),
raw output `e119-slot-tiers-2026-09-12.out` (136 lines, ending line `name=done emitted=136`; revision with container header 107 and net payload 32661, arms in `uniform` column differ by at most 1 container, previous revision `e119-slot-tiers-2026-09-08.out` kept in results/ untouched).
Mutation table `scripts/mutation/tables/e119_slot_tiers.tsv` (all 12 caught, `e119-mutate-2026-09-12.log`).
N = 100 000, unit 32768, container header 107, net payload 32661, per-slot overhead 43 (quintup 33 + write sequence 10,
**no slot directory entry required under fixed width**, 4 less than 47 used in E114 (Overall Accounting of Small-Data Packing Containers) / E116 (Packed Container Balance: Metadata Writes and Compaction Strategy)).
Replay: `bash scripts/experiment-apparatus/replay.sh` with E119 (Padding Cost of Fixed-Width Slots) number.

### Influenced Decisions

| Decision Item | Relationship | Review |
|---|---|---|
| D27 (Small-Data Packing Container) settled item 8 | Support | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D27 (Small-Data Packing Container) settled item 12 | Support | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D27 (Small-Data Packing Container) settled item 13 | Support | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D25 (Target Workload Priority) | No effect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |

## Historical Versions

Historical entries for E119 (Padding Cost of Fixed-Width Slots) are consolidated in [experiments-history.md](../experiments-history.md).
