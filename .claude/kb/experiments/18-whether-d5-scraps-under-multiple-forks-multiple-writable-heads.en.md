## E18 Whether D5 Scraps Under Multiple Forks / Multiple Writable Heads — Run (2026-08-28, After Round 2 Corrections)

[中文](18-D5在多分叉-多可写头下是否报废.md) · **English** · [日本語](18-マルチ分岐・マルチ書き込みヘッド下でD5が破綻するか.ja.md)

**Conclusion: D5 (Snapshot / Space Accounting Mechanism) breaks under multiple writable heads, but the failure mode is "unbounded leak + zero reclamation rate", not data loss.**

According to the true `death` rule defined in the body of [decisions.md](../decisions.md) D5 (Snapshot / Space Accounting Mechanism) ("the checkpoint number where the **last active reference was unlinked** and that unlinking was published; ∞ if still in an active tree"):

| History Topology | False Negative (Data Loss) | False Positive (Leak) | Leak with "Never Recording Death" |
|---|---|---|---|
| Linear (positive control) | **0** | **0** | — |
| Single fork | **0** | 8.19% | 8.44% |
| 8 writable heads | **0** | **29.40%** | **29.40% (bit-for-bit identical)** |

**Two conclusions**:

1. **The data loss direction is structurally zero.** truth is true ⇒ block is not dead on the chain of that root ⇒ does not satisfy "deleted on all branches" ⇒ `death = ∞` ⇒ interval verdict must be true. **This is not merely empirical; it is provable.**
2. **The price paid is zero reclamation capability.** In the 8 writable heads profile, the correct rule is **bit-for-bit identical** to "**recording no death whatsoever**" (both fp are 29.40%)—it trades "reclaiming zero blocks" for zero data loss.
   And maintaining "whether it is the last active reference" requires branch coverage information for each block = **reference counting**, **precisely the thing that D5 (Snapshot / Space Accounting Mechanism) sought to eliminate**.

⇒ **What D5 (Snapshot / Space Accounting Mechanism) loses under forks is its O(1) property, not correctness.**

### The 08-28 Round Overturned the Conclusions of Round 1 of E18 (Whether D5 Scraps Under Multiple Forks / Multiple Writable Heads)

Round 1 reported "1.73% data loss on single fork, 5.80% **data loss** on 8 writable heads"; that conclusion **does not hold**, with both root causes lying within E18 (Whether D5 Scraps Under Multiple Forks / Multiple Writable Heads) itself:

| # | Where It Went Wrong | Consequence |
|---|---|---|
| 1 | **Neither of the two `death` rules tested (earliest / latest) was the one defined in D5 (Snapshot / Space Accounting Mechanism) body.** D5 (Snapshot / Space Accounting Mechanism) line 206 states "last active reference was unlinked", fully deterministic under forks | Entire data loss column was counterfeit |
| 2 | **Positive control was run only on the `earliest` arm.** The `latest` arm that yielded "5.80 → 4.00" **never passed the gate**—after a supplementary run, its fp=8 on linear history | By E18 (Whether D5 Scraps Under Multiple Forks / Multiple Writable Heads) own invalidation clause, that arm should have been invalidated in its entirety back then |
| 3 | Generator allowed **deleting the same block twice on the same branch** (does not exist in a real file system), which was the exact cause of fp=8 | After fixing this, linear positive controls for all three rules were 0/0 |

⚠️ **Lesson inscribed into discipline**: **Positive controls must be run on every tested arm.**
Running only one is equivalent to the other never passing the gate, when it happened to be the very arm generating the conclusion.

### Measurement Basis and Controls

**Measurement basis**: Pure logic experiment, no devices. Shared prefix of 40 txg, 20 txg per branch,
3 blocks created and 1 prefix block deleted per txg (with **no duplicate deletions on the same branch**). Every txg is a retained root.
Code `scripts/experiment-apparatus/e18_branch.rs` (`cargo run --release --bin e18-branch`),
raw output `e18-branch-2026-08-28.out`.

| Control | Assertion | Result |
|---|---|---|
| **Positive** | Linear history must have zero error across **all three rules** | earliest / latest / conservative all fp=0 fn=0 ✓ |
| **Discriminative power** | Fork profile must truly construct cross-branch references | 7860–24420, non-zero ✓ |
| **Degradation lower bound** | Add a "never record death" arm. If correct rule is bit-for-bit identical to it, it reclaims nothing | heads8 profile is **bit-for-bit identical**—this was the weightiest finding of the 08-28 round |

⚠️ **Specific values of fp depend on root density and deletion patterns**, and vary with pattern changes.
What does not depend on pattern are the two qualitative conclusions: **fn ≡ 0 (provable)**, and **conservative degrades to never under multiple heads**.

### What This Means for D5 (Snapshot / Space Accounting Mechanism) / D6 (Snapshot Implementation Model)

D5 (Snapshot / Space Accounting Mechanism) is marked "settled (**conditional upon** history being unidirectionally linear)". E18 (Whether D5 Scraps Under Multiple Forks / Multiple Writable Heads) pins down the **nature** of that condition:
**It is not a correctness condition, but an O(1) condition.**
⇒ If D6 (Snapshot Implementation Model) selects a multi-writable-head candidate, D5 (Snapshot / Space Accounting Mechanism) failure mode is **unbounded leak + accounting degrades to reference counting**.
**Leaks can be detected and reclaimed by [invariants/05-空间记账.md](../../../crates/singlefs-checker/invariants/05-空间记账.md) I-3.1 (Allocated Statistics Match), and have bounded damage; data loss is irreversible**—
The two differ by an order of magnitude in weight in candidate evaluation for D6 (Snapshot Implementation Model), **a crucial re-characterization**.

### Affected Decisions

| Decision Item | Relationship | Review |
|---|---|---|
| D6 (Snapshot Implementation Model) settled item 5 | Supports | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-13 unaffected: renaming affects phrasing only, conclusion, numbers, and artifacts unchanged |
| D6 (Snapshot Implementation Model) settled item 6 | Supports | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-13 unaffected: renaming affects phrasing only, conclusion, numbers, and artifacts unchanged |
| D6 (Snapshot Implementation Model) settled item 8 | Supports | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-13 unaffected: renaming affects phrasing only, conclusion, numbers, and artifacts unchanged |
| D5 (Snapshot / Space Accounting Mechanism) settled item 14 | Supports | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-13 unaffected: renaming affects phrasing only, conclusion, numbers, and artifacts unchanged |
| D5 (Snapshot / Space Accounting Mechanism) settled item 17 | Supports | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-13 unaffected: renaming affects phrasing only, conclusion, numbers, and artifacts unchanged |

## Historical Versions

Historical entries for E18 (Whether D5 Scraps Under Multiple Forks / Multiple Writable Heads) are gathered in [experiments-history.md](../experiments-history.md).
