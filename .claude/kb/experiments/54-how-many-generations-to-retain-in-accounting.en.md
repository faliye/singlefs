## E54 How Many Generations to Retain in Accounting — Run (2026-08-30, N=5 Rounds Byte-Identical, 8 Unit Tests / 6 Mutation Rules Caught): **Plan A and Plan C Are Not Two Candidates**, All That Needs Setting Is K

[中文](54-记账的代要保留几代.md) · **English** · [日本語](54-記帳の「世代」は何世代保持すべきか.ja.md)

**After the user settlement "generations are needed", D5 (Snapshot / Space Accounting Mechanism) settled item 2 leaves only Plan A and Plan C**:

| Candidate | What "Generation" Represents |
|---|---|
| Plan A | Checkpoint number (txg of D16 (Publish Semantics)), **never discarded** |
| Plan C | Retain only the most recent K generations, K taking root ring depth |

E54 (How Many Generations to Retain in Accounting) measures two things: **how key counts grow for both**, and **whether discarding in Plan C requires traversal**—
whether Plan C discard requires traversal is a life-or-death criterion: `.claude/rules/fs-design.md` rule 1 commands
"runtime decision paths must not traverse, and costs must not scale with disk capacity".

### What to Measure

| # | Quantity |
|---|---|
| 1 | Key count growth with checkpoint count (Plan A vs Plan C), scanning metrics × K × checkpoint count |
| 2 | **How many keys touched when discarding one generation**, and **whether it depends on disk capacity** |
| 3 | Accounting write amplification per checkpoint (Plan A writes only; Plan C writes + deletes)|
| 4 | **Whether Plan A and Plan C are the same path**: when Plan A is equipped with an incremental discard rule, is its key count cell-by-cell identical to Plan C |

### Criteria (Hardcoded Before Run)

| # | Criterion |
|---|---|
| 1 | **Key count of Plan C is independent of checkpoint count** (slope 0); **slope of Plan A = metric count** |
| 2 | **Discard cost of Plan C is independent of disk capacity**—criterion is whether capacity appears in cost formula; appearance judged as violating `.claude/rules/fs-design.md` rule 1 |
| 3 | Plan A + incremental discard and Plan C have **cell-by-cell identical** key counts ⇒ record "they are two expressions of the same path"; if different ⇒ record where they diverge |
| 4 | Lower bound of K is given by **maximum generations root can roll back**: E53 (Can Root Ring Mount After Losing an Entire Disk) measured worst-case rollback as **1**, E50 (Upper and Lower Bounds on Root Ring Slot Count) gives total slot count as N ⇒ calculate both candidate lower bounds |

### What Constitutes Failure (Hardcoded Before Run, Forbidden to Change After Run)

| Clause | Judgment When Triggered |
|---|---|
| **Positive control**: When K = ∞, Plan C must degenerate into Plan A (slope becomes metric count)| Does not degenerate ⇒ model not computing by K, **entire round aborted** |
| **Negative control**: Metric count = 0 ⇒ key count of both constantly 0 | Non-zero ⇒ **entire round aborted** |
| **Plan C discard requires traversal** | **Faithfully record and eliminate Plan C**—violates rule 1 regardless of attractive key counts |
| Both identical cell by cell | **Faithfully record "not two candidates"**, forbidden to fabricate differences |

### The Half It Cannot Answer

1. **This is a counting model, not implementation**, same limitation as E50 (Upper and Lower Bounds on Root Ring Slot Count) / E52 (Operational Costs of Two Multi-Writable-Head Mechanisms): 0 file operations.
2. **How many metrics exist** is an input, not measurement—this project does not yet have a metric inventory (D5 (Snapshot / Space Accounting Mechanism) settled item 1 remains open).
3. **Does not model write buffer coalescence**: multiple updates to same metric within one checkpoint coalesce in buffer;
   E54 (How Many Generations to Retain in Accounting) computes as "one per metric per checkpoint", **which is an upper bound**.

### Prerequisites

None. Pure counting model.

### Measurement Basis and Reproduction

- Code `scripts/experiment-apparatus/e54_accounting_generations.rs`
  (`cargo run --release --bin e54_accounting_generations`),
  raw output `e54-accounting-gen-2026-08-30.out`.
- Mutation table `scripts/mutation/tables/e54_accounting_generations.tsv`.

### Actual Measurement (2026-08-30, N=5 Rounds Byte-Identical)

#### Result 1: **Plan A + One Incremental Discard Rule ≡ Plan C, Cell-by-Cell Identical**

Scanned across 4 metric counts × 5 checkpoint counts × 3 K values (60 cells):
`a_with_discard` and `scheme_c` are **equal in every cell** (`a_and_c_identical_with_discard=1`).

⇒ **They are not two candidates, but two expressions of the same path**:
Plan A says "generation takes checkpoint number", Plan C says "retain only recent K generations"—
yet "what generation represents" and "how many generations to keep" are **two orthogonal questions**,
framed before E54 (How Many Generations to Retain in Accounting) as a binary choice.
⇒ **All that needs setting is K.**

#### Result 2: Cost of Never Discarding Is Unbounded; Cost of Discarding Is Constant

| | Key Count (64 metrics, 1,000,000 checkpoints) | Slope Relative to Checkpoint Count |
|---|---|---|
| Never discard (literal form of Plan A) | **64 000 000** | **= metric count** (verified across 8 / 64 / 512 tiers)|
| Retain recent K = 8 generations | **512** | **0** |

#### Result 3: Discard Is Incremental, **Satisfying `.claude/rules/fs-design.md` Rule 1**

Keys touched when discarding one generation = **metric count** (**64** at K=8, 64 metrics, 1,000,000 checkpoints),
**independent of checkpoint count, independent of disk capacity** (`discard_depends_on_capacity=0`).

Mechanism: in `(metric, dimension_tuple, generation)`, "generation" is a segment of the key; the generation to discard is **computable** (current generation − K),
**no search needed** ⇒ point deletion, not scan.
⇒ **Rule 1 requirement "no traversal, cost independent of disk capacity" is satisfied.**
The cost is that accounting writes per checkpoint change from `metric count` to `metric count × 2` (one write, one delete).

#### Result 4: Lower Bound of K Given by "Maximum Generations Root Can Roll Back"

| Source | Rollback Depth | K Lower Bound | Key Count Under 64 Metrics |
|---|---|---|---|
| E53 (Can Root Ring Mount After Losing an Entire Disk) **Empirical** worst-case rollback | 1 | **2** | **128** |
| E50 (Upper and Lower Bounds on Root Ring Slot Count) Total slots (conservative: any surviving slot selectable) | 8 | **9** | **576** |

⇒ **The two lower bounds differ by 4.5×, but both are constants and five orders of magnitude smaller than never discarding.**

#### Itemized Control Verdicts

| Clause | Verdict |
|---|---|
| **Positive control**: K = ∞ ⇒ Plan C degenerates into Plan A | **Passed** (64,000,000 on both sides)|
| **Negative control**: Metric count 0 ⇒ both constantly 0 | **Passed** |
| Plan C discard requires traversal ⇒ eliminate Plan C | **Did not occur**: discard cost = metric count, formula free of disk capacity |
| Both identical cell by cell ⇒ record "not two candidates" | **Occurred, faithfully recorded** |

### Conclusions

1. **"What generation represents" and "how many generations to retain" are two orthogonal questions**, framed before E54 (How Many Generations to Retain in Accounting) as a binary choice.
   Generation taking checkpoint number + incremental discard rule **is** "retaining only recent K generations".
2. **Discard satisfies rule 1**: point deletion, cost = metric count, independent of disk capacity.
3. **Lower bound of K ties to "maximum generations root can roll back"**: empirical measurement basis K ≥ 2, conservative basis K ≥ total root ring slots + 1.
4. ⚠️ **Absolute key count still depends on number of metrics**, absent in this project
   (D5 (Snapshot / Space Accounting Mechanism) settled item 1 remains open) ⇒ 128 / 576 in K lower bound table are **examples based on 64 metrics**, not forecasts.

### Measurement Basis and Known Limitations

- Code `scripts/experiment-apparatus/e54_accounting_generations.rs`
  (`cargo run --release --bin e54_accounting_generations`),
  raw output `e54-accounting-gen-2026-08-30.out`.
  Mutation table `scripts/mutation/tables/e54_accounting_generations.tsv` (6 rules, all caught).
- ⚠️ **Counting model, not implementation** (0 file operations), same limitation as E50 (Upper and Lower Bounds on Root Ring Slot Count) / E52 (Operational Costs of Two Multi-Writable-Head Mechanisms).
- ⚠️ **Does not model write buffer coalescence**: multiple updates to same metric in same checkpoint coalesce in buffer;
  E54 (How Many Generations to Retain in Accounting) calculates as "one per metric per checkpoint", **giving an upper bound**.
- ⚠️ **Metric count is input, not measurement**: project lacks metric inventory.

### Impacted Decisions

| Decision Item | Relationship | Review |
|---|---|---|
| D5 (Snapshot / Space Accounting Mechanism) settled item 2 | Supports | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D16 (Publish Semantics) | Does not affect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |

## Historical Versions

### 2026-08-30

- Run completed. Criteria untouched by a single word, all 6 mutations caught. **Main conclusion is "this binary choice itself does not hold"**—
  Plan A plus incremental discard rule is cell-by-cell identical to Plan C.
- Initiated. Basis: user settlement "generations are needed" ⇒ Plan B eliminated; difference between Plan A and Plan C is computable;
  and D8 (Core Index Structure) settled item 1 explicitly noted "increase in key count is unmeasured; forbidden to infer space before measuring".
