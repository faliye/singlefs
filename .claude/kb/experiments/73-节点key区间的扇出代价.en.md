## E73 Fanout Cost of Node Key Interval — Run (2026-09-01, Pure Arithmetic, 8 Unit Tests / All 8 Mutations Caught): **Fanout Drops by at Most 1 Cell (0.33%), Tree Height Rises Zero Levels**

[中文](73-节点key区间的扇出代价.md) · **English** · [日本語](73-节点key区间的扇出代价.ja.md)

D18 (Information Carried in a Block) settled item 2 settled that nodes of accounting tree carry key intervals, while acknowledging
"how much fanout drops and whether write amplification shifted has never been measured once; the 16 KiB of D8 (Core Index Structure) settled item 2 was chosen under the premise of **lacking** these two fields".

### Criteria (Hardcoded Before Run, Unchanged After Run)

1. **Absolute value assertion**: Fanout must exactly equal `(node_size − node_header) / entry_width`, calculable by each of the two arms independently,
   forbidden to compare merely the ratio between the two arms.
2. If tree height increases by one level as a result, judge that "this 16 KiB choice must be reopened alongside it".
3. What key interval buys (a node alone picked up can self-prove which range it covers) must have a **check capable of failing**:
   given an isolated node to judge what range it should cover, the arm without intervals must fail to determine it.

### Failure Clauses (Hardcoded Before Run)

| Clause | Verdict |
|---|---|
| **Positive control**: Ballooning key interval fields to half of node size, fanout must collapse | **Passed** (301 → 150) |
| Tests accounting tree only, conclusions carry measurement basis "holds only for accounting tree", **forbidden to extrapolate** | Carried |

### Measurement

#### Criterion 1: Fanout Drops by at Most 1 Cell

Accounting tree key = `(statistic, tree_id, device, generation)` = **22 bytes**; entry = key + child pointer 32 = **54 bytes**.

⚠️ **Range Note (2026-09-03, E97 (Entry Encoding of Accounting and Allocation Records) review): That 32-byte child pointer is not a settled value.**
Recalculating per actual width of **53 bytes** for tree table unit pointer from D22 (How to Synthesize Unit Atomicity) settled item 7,
internal fanout for the identical 22-byte key is **217** rather than 301.
**The conclusion of E73 (Fanout Cost of Node Key Interval) remains unaffected** — E97 (Entry Encoding of Accounting and Allocation Records)
recalculated across four key tiers 22 / 19 / 14 / 7 and two child pointer widths; accounting tree height remains **invariably 2**,
so 16 KiB of D8 (Core Index Structure) settled item 2 still does not need reopening. **What must change is merely that absolute fanout values of those three tiers may not be cited externally.**
Node header with interval adds two key fields (`min_key` + `max_key`) = **+44 bytes**.

| Base Node Header | Header with Interval | Fanout with Interval | Fanout without Interval | Delta |
|---|---|---|---|---|
| 32 | 76 | **302** | **302** | **0** |
| 64 | 108 | **301** | 302 | **1** |
| 128 | 172 | **300** | 301 | **1** |

⇒ **Cost is at most 1 cell, relative 0.33%.** At base header 32 tier, **cost is exactly 0** —
all 44 bytes absorbed completely by integer division remainder.

⚠️ **Scope of the three tiers narrowed by D18 (Information Carried in a Block) settled item 7 (settled 2026-09-02, note added 2026-09-03)**:
that settlement defined field set for index node headers — common plaintext prefix 42 bytes (magic 4 + format version 2 + flags 2 +
declared length 2 + header checksum 32) + class identity segment (tree ID + level + key interval settled for this tree +
birth generation 8 + fsid 8). ⇒ **Base node header ≥ 58 bytes (42 + 8 + 8, widths of tree ID and level remain implementation-dependent)**
⇒ **32 tier is unreachable**, "cost exactly 0 at 32 tier" points to a tier non-existent following that settlement.
**Conclusion unchanged**: recalculating under three lower bounds 58 / 67 / 76, fanout without interval is invariably 302, with interval invariably 301,
delta remains 1 cell; tree height still does not grow ⇒ 16 KiB of D8 (Core Index Structure) settled item 2 not reopened.
⚠️ **What remains unsettled is width, not fields**: the statement during E73 (Fanout Cost of Node Key Interval) run that "base bytes of node header never settled anywhere in repo"
no longer holds for the field set half; width half remains unsettled (tree ID, level, key segments),
sharing the exact same owed check with E71 (Entry Count and Width of Accounting Keys) / E74 (Allocation Record Entry Count and Compaction Cost).

#### Criterion 2: Tree Height Rises Zero Levels ⇒ 16 KiB of D8 (Core Index Structure) Settled Item 2 **Does Not Need Reopening**

Entry counts taken from accounting tree scales measured by E71 (Entry Count and Width of Accounting Keys), across 3 base headers × 4 entry count tiers for 12 cells in total:

| Entry Count | Tree Height with Interval | Tree Height without Interval |
|---|---|---|
| 4 075 (E71 (Entry Count and Width of Accounting Keys) narrow interpretation arm) | 2 | 2 |
| 59 200 (wide interpretation arm) | 3 | 3 |
| 115 200 (upper bound arm) | 3 | 3 |
| 1 000 000 | 3 | 3 |

**All 12 cells yielded `grew=0`.** ⇒ Reopening condition of Criterion 2 **was not triggered**.

#### Write Amplification: Unmoved

Under COW, number of nodes rewritten to modify one key = tree height, cell-by-cell identical across both arms (2 / 3 / 3).
⚠️ This is a **structural measurement basis**, not byte throughput — byte throughput requires real devices.

#### Criterion 3: Isolated Node Self-Proves Covered Interval

| Arm | Can Isolated Node Answer What Range It Should Cover |
|---|---|
| **With interval** | **Can answer** (`100..200`) |
| Without interval | **Cannot answer** (interval exists only in parent node's keypointer; lost if parent lost) |

⇒ What D18 (Information Carried in a Block) settled item 2 verbatim claimed it bought **gained a check capable of failing**,
rather than a mere assertion. This check was proven to turn red by mutation M6 (modifying arm without interval to also self-prove, which immediately judged red).

### Conclusion: This Is Cheap Self-Containment

`.claude/rules/fs-design.md` "Do not sacrifice self-containment to save space" verbatim requires
"When encountering trade-offs like 'can this field be omitted', default answer is **do not omit** — unless empirical measurement proves the saved portion is a bottleneck".
⇒ **Empirical measurement concludes the saved portion is far from a bottleneck**: 0 to 1 cell fanout, tree height unchanged, write amplification unchanged,
while buying "a node picked up alone can self-prove which range it covers".

### What It Cannot Answer

1. Pure arithmetic + isolated node judgment model: no btree implementation, no split/merge, no write buffer,
   0 file operations.
2. **Base byte count of node header was never settled anywhere in repo at run time** ⇒ at run time it was a **parameter**, evaluated across 32 / 64 / 128
   tiers; conclusions consistent across all three tiers (dropping 0 or 1 cell), but **if base header swells to several hundred bytes, this conclusion must be recalculated**.
   ⚠️ **Field set half settled 2026-09-02** (D18 (Information Carried in a Block) settled item 7 ⇒ base header ≥ 58,
   tier 32 unreachable), recalculation detailed in Range Note of Measurement; **width half remains unsettled**.
3. **Segment widths of key likewise never settled** (same owed check as E71 (Entry Count and Width of Accounting Keys)): 22 bytes is an assumption.
   Wider key simultaneously elevates entry width and interval fields in same direction, **net effect un-scanned**.
4. **Holds only for accounting tree.** Whether dirent tree carries it settled affirmatively by D18 (Information Carried in a Block) settled item 2; its fanout cost must be measured separately —
   its separating key is file name with completely different width distribution, **forbidden to extrapolate**.
5. **Value** of "isolated node self-proving alone" unmeasured: what it buys is one fewer step on reconstruction path,
   while reconstruction path itself does not yet exist.

### Prerequisites

None. Pure arithmetic.

### Measurement Basis and Re-run

- Code `scripts/experiments/e73_key_range.rs`
  (`cargo run --release --bin e73-key-range`),
  raw output `e73-key-range-2026-09-07.out` (re-run 2026-09-07 per header widths holding on that day; copy prior to debt repayment was `e73-key-range-2026-09-01.out`, retained in `results/`) (26 lines, ending with line `emitted=26`).
- **Byte-for-byte identical across N=5 rounds** (identical md5). **8 unit tests, all 8 mutations hit one-by-one, 0 blind spots**,
  mutation table `scripts/mutation/tables/e73_key_range.tsv`
  (`bash scripts/mutation/mutate.sh e73-key-range scripts/experiments/e73_key_range.rs scripts/mutation/tables/e73_key_range.tsv`).

⚠️ **Re-run once on 2026-09-07 per header widths established on that day** (step requested by user prior to settling D18 (Information Carried in a Block) settled item 14).
Prior to 2026-09-07, three base node headers in source were **58 / 67 / 76**, which were lower bounds on day E73 (Fanout Cost of Node Key Interval)
ran; two subsequent widenings were never reflected in source, added together this time:
write sequence 10 (D18 (Information Carried in a Block) settled item 7 note) + reserved space 28
(D18 (Information Carried in a Block) settled item 7 verbatim "v1 reserves nonce generation and MAC field space",
per settled item 14 arm A nonce generation 12 and D9 (Encryption) settled item 2 MAC 16) ⇒ **96 / 105 / 114**.
Both sets of headers retained in source running cell-by-cell; older set reflects standing ground of this conclusion prior to 2026-09-07.
**Conclusion did not flip**, only intermediate quantities shifted. Artifact swapped to the one re-run on same day.

**Intermediate quantities**: Fanout without key interval 302 → **301**, with interval 301 → **300**; delta between both remains 1 cell.

### Impacted Decisions

| Decision Item | Relationship | Review |
|---|---|---|
| D5 (Snapshot / Space Accounting Mechanism) settled item 5 | Support | 2026-10-11 unaffected: wording only (temporal terms, positional references, self-referential terms), conclusions, figures, and artifact unchanged |
| D18 (Information Carried in a Block) settled item 2 | Support | 2026-10-11 unaffected: wording only (temporal terms, positional references, self-referential terms), conclusions, figures, and artifact unchanged |
| D18 (Information Carried in a Block) settled item 7 | Does not affect | 2026-10-11 unaffected: wording only (temporal terms, positional references, self-referential terms), conclusions, figures, and artifact unchanged |
| D8 (Core Index Structure) settled item 2 | Support | 2026-10-11 unaffected: wording only (temporal terms, positional references, self-referential terms), conclusions, figures, and artifact unchanged |
| D9 (Encryption) | Does not affect | 2026-10-11 unaffected: wording only (temporal terms, positional references, self-referential terms), conclusions, figures, and artifact unchanged |
| D22 (How to Synthesize Unit Atomicity) settled item 7 | Does not affect | 2026-10-11 unaffected: wording only (temporal terms, positional references, self-referential terms), conclusions, figures, and artifact unchanged |

## Historical Versions

Historical entries for E73 (Fanout Cost of Node Key Interval) are concentrated in [experiments-history.md](../experiments-history.md).
