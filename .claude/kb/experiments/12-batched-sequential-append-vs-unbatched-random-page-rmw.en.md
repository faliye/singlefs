## E12 Batched Sequential Append vs Unbatched Random Page RMW — Run (2026-08-26)

[中文](12-攒批的顺序追加vs不攒批的随机页读改写.md) · **English** · [日本語](12-バッチシーケンシャル追記-vs-非バッチランダムページRMW.ja.md)

**Background**: [decisions.md](../decisions.md) D5 (Snapshot / Space Accounting Mechanism) uses "birth txg + chronology" to deduce block lifecycles, where deletion verdict reads two scalars with **zero additional device I/O**; its cost is excluding reflink / dedup (one premise being "each block has at most one live reference in live trees").
Another path is **one reference count per block**, capable of supporting arbitrary numbers of live references, at the cost of reading-modifying-writing a counter upon each dereference.

When D5 (Snapshot / Space Accounting Mechanism) was initiated, it cited figures from btrfs qgroup (commit time +76%, transaction wait +1347%, write throughput −25%).
⚠️ **That measurement basis measured qgroup going through reverse index, not pure reference counts**, and using it to answer this question exaggerates.
**E12 (Batched Sequential Append vs Unbatched Random Page RMW) measures directly.**

### Measurement Basis: The Sole Independent Variable Separating the Two is "Counter Table Working Set vs Cache Budget"

- When **counter table fits in cache**, reference count read-modify-write stays entirely in memory, with device I/O close to 0—both may exhibit no distinguishable difference, **and that is a valid result**.
- When **it does not fit**, every deletion can trigger a random read (4 KiB page) + a writeback.
  **The D5 (Snapshot / Space Accounting Mechanism) side does not vary with working set**, because it has no counters at all.

### Three Tested Paths

| Arm | Action on Dereferencing One Reference | Expected Device I/O |
|---|---|---|
| **A: D5 (Snapshot / Space Accounting Mechanism) Chronology** | Compare two scalars (`ptr.birth` already in hand, `previous_snapshot_txg` resident in memory) → append one fixed-length record to deadlist buffer, sequentially flush when 1 MiB full | **Only sequential writes**, zero random reads |
| **B: Reference Counting** | Locate counter page (`blk / entries_per_page`) → if not in cache, O_DIRECT read 4 KiB (LRU eviction, dirty page writeback) → decrement → mark dirty | Random reads + random writes, growing with working set / cache ratio |
| **C: D5 (Snapshot / Space Accounting Mechanism) + Sparse Side Table (ZFS BRT Form)** | Probe sparse table once to verify "block has not been cloned"; on miss take A, on hit take B | Depends on **whether probe can avoid touching device** |

### Independent Variables

Counter table entry count, taking three tiers: **fits in cache / 2× cache / 8× cache**.
Cache budget is fixed (hardcoded page count), random deletion sequence uses **fixed seeds** (reproducibility is a hard requirement).
N≥5 rounds per tier.

### Criteria

1. **Primary criterion**: How Arm B's "device I/O count per deletion" scales with working set / cache ratio, and whether Arm A is **flat**.
2. **Crucial question for Arm C**: **For uncloned blocks, can probe achieve truly zero device I/O**.
   If not, the entire selling point of the "D5 (Snapshot / Space Accounting Mechanism) primary + sparse side table secondary" combination collapses.
3. Report **device I/O counts and bytes**, not just wallclock—wallclock is influenced by VM and host, whereas I/O count is a structural quantity.

### What Counts as Failure (Two Control Groups, Judged Separately)

| Phenomenon | Verdict | Rationale |
|---|---|---|
| In "8× cache" tier, Arm B shows **no distinguishable extra I/O** relative to Arm A | **Implementation defect, entire round voided** | This is a **positive control**: working set being 8× cache without generating random reads indicates cache boundary failed or access sequence insufficiently random |
| In "fits in cache" tier, Arm B and Arm A show no distinguishable difference | **Valid result, record faithfully** | It demonstrates that "refcounting is nearly free when counters fit", which is one of the questions E12 (Batched Sequential Append vs Unbatched Random Page RMW) aims to answer, **not a bug** |
| Arm A shows distinguishable differences across the three tiers | **Implementation defect, entire round voided** | Arm A contains no counters and should not vary with counter table size |

⚠️ Retaining positive controls is a hard requirement (`.claude/singlefs-ai-sop/rules/test-discipline.md`: failure clauses of an experiment must not make conclusions unfalsifiable)—**without them, "no difference" cannot be distinguished between expected finding or broken implementation.**

### What E12 (Batched Sequential Append vs Unbatched Random Page RMW) Does Not Answer

- **Does not answer** "whether reflink is worth doing"—that is a feature trade-off, not decidable by performance figures.
- **Does not answer** crash consistency and checker costs. Reference counting is delta-form,
  and its checker verification has a class of blind spots (see [checks-owed.md](../checks-owed.md) C12 (shared delta semantics));
  **that balance is outside the range of E12 (Batched Sequential Append vs Unbatched Random Page RMW), and figures from E12 (Batched Sequential Append vs Unbatched Random Page RMW) must not be used to cover it**.
- **Does not answer** distribution under real workloads. Deletion sequences in E12 (Batched Sequential Append vs Unbatched Random Page RMW) are uniform random;
  real workloads have locality that flatters Arm B. **Extrapolations must state this.**

**Prerequisites**: VM harness of E7 (Offline Index Harness) (**ready and self-test passed**), block device, Rust toolchain. **No file system required.**

⚠️ **Raw output not retained**: E12 (Batched Sequential Append vs Unbatched Random Page RMW) requires a real block device (`e12-lifecycle <block-device>`) rather than a pure in-memory model,
and results were not persisted to disk at the time. "What E12 (Batched Sequential Append vs Unbatched Random Page RMW) does not answer" figures can only be checked by re-runs, which require VM and block device.

### What E12 (Batched Sequential Append vs Unbatched Random Page RMW) Actually Measured (Must Read Before Reading Results)

**It measures the I/O comparison between "batched sequential append" and "unbatched random page read-modify-write",
plus a self-test of the measurement apparatus. It is not a mechanism comparison between "D5 (Snapshot / Space Accounting Mechanism) and its competitors".**

Actual content of Arm A: `previous_snapshot_txg` is a hardcoded constant (**no snapshot was created or destroyed during runtime**);
`birth` is a freshly generated random number, not read from any pointer; records declare 16 bytes but only write 8 bytes, **block pointers not included**;
**zero reads across the entire segment**. ⇒ **Replacing criterion with coin flips yields identical results.**
That "2" = ⌈1.6 MB / 1 MiB⌉ + final flush, **unrelated to birth txg, deadlist semantics, or snapshots**.

Arm B **does not batch at all**: one random page read-modify-write per operation.
⇒ **Amortization architectures of both arms are asymmetric; E12 (Batched Sequential Append vs Unbatched Random Page RMW) cannot separate contributions of mechanism versus batching.**
E12 (Batched Sequential Append vs Unbatched Random Page RMW)'s own `sequential` injection reveals the order of magnitude: 174,998 → **391**,
**batching + sorting alone consumes two and a half out of three orders of magnitude.**

Approximately 100k operations judged "immediate release" skip at zero cost in Arm A,
whereas [decisions.md](../decisions.md) D5 (Snapshot / Space Accounting Mechanism) pseudocode requires them **to enter defer queue**—
that is a second append log that must be persisted, unmeasured by E12 (Batched Sequential Append vs Unbatched Random Page RMW).

**The four premises of D5 (Snapshot / Space Accounting Mechanism) have zero coverage in E12 (Batched Sequential Append vs Unbatched Random Page RMW)**: no snapshots, no clones, no reflink, no crashes, no flushes.
Yet risks of D5 (Snapshot / Space Accounting Mechanism) lie not on steady-state kill paths, but on **snapshot creation boundaries** (`birth == S.txg` taken wrongly is data loss),
**cascading merges on snapshot destruction** (bucketing cost by birth was unmeasured by E12 (Batched Sequential Append vs Unbatched Random Page RMW)), and **crash replay**.

### Three Conclusions Supported by E12 (Batched Sequential Append vs Unbatched Random Page RMW)

1. **Analytical expression for inflection point**: `io/op = 2 × (1 − cache / table)`, measured 0.998 vs analytical 1.000, 1.751 vs 1.750.
2. **The `nodirect` row**: Proves this **measurement apparatus** has discriminative power—unrelated to D5 (Snapshot / Space Accounting Mechanism), but valid for all subsequent block layer experiments.
3. **The `bigcache` / `sequential` rows**: Quantify the individual weights of Arm B's two driving variables (85×, 448×).
   **These two rows represent the most informative part of E12 (Batched Sequential Append vs Unbatched Random Page RMW), and they measure "how fragile Arm B's model is".**

### At Least Three Changes Required to Make It a True Mechanism Comparison

Arm A truly implements D5 (Snapshot / Space Accounting Mechanism) (including snapshot creation handover, destruction cascade, defer queue);
Arm B and Arm A have symmetric amortization architectures (both batched);
Put real competitors (D6 (Snapshot Implementation Model) "snapshot ID into key" + D8 (Core Index Structure) write buffer form) on the field.

### Results (2026-08-26, N=5 Rounds, **Five Distinct Seeds**, with Independent Block Layer Readings)

**Total Device I/O Triggered Across 200,000 Deletions (Median)**

| Arm | Fits in Cache | 2× Cache | 8× Cache |
|---|---|---|---|
| **A: D5 (Snapshot / Space Accounting Mechanism) Chronology** | **2** | **2** | **2** |
| **B: Reference Counting** | 512 | ~200,208 | ~350,042 |
| **C: D5 (Snapshot / Space Accounting Mechanism) + Sparse Side Table** (clone rate 1‰) | 508 | ~1,956 | ~3,466 |

**Median Wallclock (ms)**: A is ~1.7–2.3; B is 53 / 15398 / 26982; C is 40 / 150 / 271.

#### Verification: Independent Block Layer Readings Match In-Program Counters Cell by Cell, Ratio 1.00

Each arm reads `/sys/block/vda/stat` before and after (**counted by kernel block layer itself**,
sharing no code, sampling, or tools with this program's counters, satisfying
verification requirements of `.claude/singlefs-ai-sop/rules/evidence-discipline.md`).
**All nine cells show ratio 1.00.**

This independent path proves three things that were merely assumptions prior to E12 (Batched Sequential Append vs Unbatched Random Page RMW):
**O_DIRECT truly bypassed cache** (no extraneous reads), **block layer neither merged nor split requests**, **no read-ahead**.

#### Dispersion Across Seeds is Real

| Arm · Working Set | Actual Values Across Five Seeds |
|---|---|
| refcount · 2× | 199894 / 200118 / 200208 / 200214 / 200632 |
| refcount · 8× | 349930 / 349980 / 350042 / 350084 / 350156 |
| hybrid · 8× | 3396 / 3420 / 3466 / 3500 / 3522 |

Dispersion is 0.4%–4%. **Whereas D5 (Snapshot / Space Accounting Mechanism) is constantly 2—that invariance is structural (it never queries counters), not artifact of seeds.**

⚠️ Block layer range for `d5 · x8` displaying 50% is because **absolute value is only 2**, where a difference of 1 is 50%.
**Percentages over small bases are meaningless; do not read them.**

### ⚠️ Arm C Lacks Failure Clauses and Positive Controls (Found During 2026-08-29 Adversarial Verification, Unfixed)

The three failure clauses covered **Arm A and Arm B**:
"Tier 8× Arm B has no extra I/O ⇒ implementation defect", "Fits-in-cache tier shows no difference ⇒ valid result",
"Arm A shows difference across three tiers ⇒ implementation defect". **Arm C has none.**

Yet Arm C is the **sole source** of the conclusion cited by D5 (Snapshot / Space Accounting Mechanism)
("not binary choice; data supports D5 (Snapshot / Space Accounting Mechanism) primary + sparse side table secondary").
The statement in E12 (Batched Sequential Append vs Unbatched Random Page RMW) text "Crucial question for Arm C: For uncloned blocks, can probe achieve truly zero device I/O"
is a criterion, **but it is absent from failure clause table, and nothing proves Arm C's measurement has discriminative power**—
no one performed self-tests like E7 (Offline Index Harness) (replacing arm's core criterion with constant, where results must change significantly).

⚠️ **This shares the same pattern as the lesson from E18 (Whether D5 Scraps Under Multiple Forks / Multiple Writable Heads)**:
Positive controls covered only a subset of arms, **and the uncovered arm happened to be the one yielding conclusions**.

**The owed control form is clear**: change Arm C's sparse table probe to constantly return "miss" (equivalent to degenerating into Arm A);
its I/O count must change significantly; invariance indicates probe never participated, voiding entire round.
⚠️ **That round was not run**—Arm C requires real block device and VM (`e12-lifecycle <block-device>`),
and raw artifacts of E12 (Batched Sequential Append vs Unbatched Random Page RMW) were not retained. **This is recorded; do not pretend it was verified.**

### Point-by-Point Evaluation of Three Failure Clauses

| Clause | Observation | Verdict |
|---|---|---|
| 8× tier Arm B shows no distinguishable extra I/O → implementation defect | 350042 vs 2 | **Not triggered**, positive control has discriminative power |
| Fits-in-cache tier shows no difference → valid result | 512 vs 2, both absolute values minimal (512 = cold start filling 256 pages read+write once) | **Triggered, record faithfully**: reference counting nearly free when counter table fits |
| Arm A shows difference across three tiers → implementation defect | All three tiers **are 2** | **Not triggered** |

### Conclusions

1. **Transition is steep, not gradual.** Once counter table cannot fit in cache, reference counting jumps from 512 to ~200k
   (≈ 1 device I/O per deletion), reaching 350k at 8×. **Whereas D5 (Snapshot / Space Accounting Mechanism) is constantly 2**—it has no counters to query.
2. **Sparse side table reclaims most advantage.** At 1‰ clone rate, 8× tier is 3466,
   compared to pure reference counting's 350042, **~100×**.
3. **Therefore not a binary choice.** Data supports "**D5 (Snapshot / Space Accounting Mechanism) primary + sparse side table secondary**" (ZFS BRT form).

### Mechanism: Why D5 (Snapshot / Space Accounting Mechanism) is Exactly 2 While Reference Counting Explodes with Working Set

**Numbers without explained mechanism are mere observations, not understanding.** Both sides can be derived from first principles,
and **derivation matches empirical measurements to three significant figures**—constituting a third independent verification path
(the other two being block layer readings and direct measurement).

#### Why Arm A is 2

Verdict inputs are only two, **neither requiring querying anything**:
`ptr.birth` (already read from pointer, zero extra I/O) and `head.previous_snapshot_txg` (one scalar resident in memory).
Thus sole device I/O is **sequentially appending** deadlist records out:

- Across 200,000 deletions, ~100k entries with `birth ≤ previous_snapshot_txg` enter deadlist (the other half release directly, without even writing)
- 16 bytes per entry ⇒ ~1.6 MB ⇒ **1.53 buffers of 1 MiB**
- ⇒ 1 full buffer flush + 1 trailing flush = **2 writes, 0 reads**

Derivation 1,600,000 bytes vs measured `bytes_w=1597440`, difference 0.16% (PRNG distribution not exact half).

> **Its I/O count = ⌈total bytes into deadlist / buffer⌉, proportional to "volume of deletions", independent of address space size.**

#### Why Arm B Explodes

It must **query a table by block identity**, and table size is proportional to address space. Under uniform random access + LRU:

```
page_fault_rate = 1 − cache_pages / table_pages
each_fault = one random read + one dirty page eviction writeback = 2 I/Os
⇒ io_per_op = 2 × (1 − cache_pages / table_pages)
```

| Tier | Table Pages | Cache / Table | Fault Rate | **Analytical io/op** | **Measured io/op** |
|---|---|---|---|---|---|
| fits | 256 | 1.000 | 0.000 | 0.000 | 0.003 (one-off cold start filling 256 pages) |
| 2× | 512 | 0.500 | 0.500 | **1.000** | **0.998** |
| 8× | 2048 | 0.125 | 0.875 | **1.750** | **1.751** |

> **Its I/O count is proportional to "address space to cache ratio", independent of deletion volume—paid on every deletion.**

#### Structural Difference in One Sentence

**D5 (Snapshot / Space Accounting Mechanism) converts random lookup into sequential append; reference counting lacks this option**,
because "where this block's counter resides" is determined by **block identity**, which is random.
This is identical in shape to LSM relative to B-tree, occurring at the **accounting layer**.

#### ⚠️ This "2" Has a Premise, and That Premise is D16 (Publish Semantics)

**Those 2 writes hold only on the premise that "appends can batch into bulk flushes".**
If every deletion must persist immediately (e.g. fsync per deletion), buffers cannot accumulate,
and **2 degenerates to ~100,000** (one persistence per record)—**this point is deduction, not measured**.

⇒ **[decisions.md](../decisions.md) D16 (Publish Semantics) designating checkpoint is precisely the premise for D5 (Snapshot / Space Accounting Mechanism)'s "2".**
If publish semantics revert to "new root per transaction", Arm A results of E12 (Batched Sequential Append vs Unbatched Random Page RMW) no longer hold and must be re-tested.

### Fault Injection on Measurement Apparatus: Proving Discriminative Power

**"Ratio 1.00" can mean "all correct", or "verification path is cosmetic"—without fault injection, one cannot tell.**
`.claude/singlefs-ai-sop/rules/test-discipline.md`: after writing a check, ask "if the subject truly broke, would it turn red".

Four test cases (seed 77777, inspecting only refcount 8× tier):

| Injection | Modification | My Counted I/O | **Block Layer Count** | Wallclock | Verdict |
|---|---|---|---|---|---|
| **none** (baseline) | — | 174,998 | 174,998 | 27.2 s | Ratio 1.00 |
| **nodirect** | Strip `O_DIRECT` upon device open | 174,998 | **0** | 0.36 s | **Block layer verification has discriminative power** |
| **bigcache** | Enlarge LRU capacity to fit entire table (table size unchanged) | **2,048** | 2,048 | 0.36 s | **Working set / cache ratio is true driver** |
| **sequential** | Change access sequence to sequential rather than random | **391** | 391 | 0.056 s | **Randomness is true driver** |

**Most critical is the `nodirect` row.** After stripping O_DIRECT, program internal counter remains **completely unchanged**
(counting syscalls it issued), while **block layer reading drops to zero**—page cache absorbed all, wallclock dropped 75×.

> **⇒ If O_DIRECT silently fails one day, the ratio drops to 0 rather than remaining at 1.00. That 1.00 is meaningful.**

`bigcache` and `sequential` prove respectively that the two premises of conclusions are real drivers rather than coincidences:
cache fitting whole table yields 174,998 → 2,048 (**85×**); sequential access yields 174,998 → 391 (**448×**).

⚠️ **The `sequential` row transforms extrapolation boundary from "annotation" into "measured number"**:
Deletion sequence of E12 (Batched Sequential Append vs Unbatched Random Page RMW) is uniform random, whereas **real workloads have locality**—
in the sequential extreme, refcounting requires only 391 I/Os, differing from D5 (Snapshot / Space Accounting Mechanism)'s 2 by 195× rather than 87499×.
**Cost of reference counting depends heavily on access locality; this point must be cited alongside primary conclusions.**

#### A Fault Injection Error I Made Myself

First version of `bigcache` enlarged cache to 2048 pages, **and because tiers were defined relative to cache, table also expanded 8×**,
leaving ratio unchanged and result identical (1.74992)—**measuring nothing**.
It was exposed by `counters=8388608` in config line. Changed to "tiers defined against baseline cache of 256 pages, invariant to faults".

**Lesson: logging all parameters into result lines was the sole reason this was caught.**
Logging only results without config would have caused that round to be accepted as the erroneous conclusion "bigcache had no effect".

### What These Numbers Cannot Prove (Extrapolation Boundaries, Must Be Cited Together)

- ⚠️ **Arm C probing uses an in-memory bitmap of 1 bit per block, which is idealized.** On real file systems ≈ 128 MB/TB,
  **E12 (Batched Sequential Append vs Unbatched Random Page RMW) overestimates side table performance on large disks**. ZFS BRT is a **sparse structure**; substituting it requires re-measurement.
- ⚠️ **Uniform random deletion sequence**; real workloads have locality, **which flatters reference counting**.
- ⚠️ **Does not answer checker costs** (reference counting is delta-form, with blind spots like [checks-owed.md](../checks-owed.md) C12 (shared delta semantics)),
  **nor crash consistency**, nor "whether reflink is worth doing".

**Measurement basis**: `e12-lifecycle`, musl static, xorshift64\* random sequence,
five seed rounds 11111 / 22222 / 33333 / 44444 / 55555 (three arms share identical seed within same round);
cache budget 256 pages (1 MiB), 200,000 deletions per tier, 512 counters per page;
VM 2048M / 4 vCPU / 2 GiB virtio disk, `cache=none,aio=native`,
host 32 cores / 60 GiB; QEMU 8.2.2 + KVM. Block layer readings taken from `/sys/block/vda/stat` inside VM.

### Affected Decisions

| Decision Item | Relationship | Review |
|---|---|---|
| D6 (Snapshot Implementation Model) settled item 1 | Support | 2026-10-11 unaffected: only phrasing revised (temporal words, positional references, and self-references); conclusions, numbers, and artifacts remain unchanged |
| D5 (Snapshot / Space Accounting Mechanism) settled item 13 | Support | 2026-10-11 unaffected: only phrasing revised (temporal words, positional references, and self-references); conclusions, numbers, and artifacts remain unchanged |
| D5 (Snapshot / Space Accounting Mechanism) settled item 19 | Support | 2026-10-11 unaffected: only phrasing revised (temporal words, positional references, and self-references); conclusions, numbers, and artifacts remain unchanged |
| D5 (Snapshot / Space Accounting Mechanism) settled item 17 | Support | 2026-10-11 unaffected: only phrasing revised (temporal words, positional references, and self-references); conclusions, numbers, and artifacts remain unchanged |
| D5 (Snapshot / Space Accounting Mechanism) settled item 6 | Support | 2026-10-11 unaffected: only phrasing revised (temporal words, positional references, and self-references); conclusions, numbers, and artifacts remain unchanged |
| D16 (Publish Semantics) | No Effect | 2026-10-11 unaffected: only phrasing revised (temporal words, positional references, and self-references); conclusions, numbers, and artifacts remain unchanged |
| D8 (Core Index Structure) | No Effect | 2026-10-11 unaffected: only phrasing revised (temporal words, positional references, and self-references); conclusions, numbers, and artifacts remain unchanged |

## Historical Versions

Historical entries for E12 (Batched Sequential Append vs Unbatched Random Page RMW) are collected in [experiments-history.md](../experiments-history.md).
