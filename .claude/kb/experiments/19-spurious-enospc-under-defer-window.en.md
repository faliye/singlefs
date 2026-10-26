## E19 Spurious ENOSPC Under Defer Window — Modeling Layer Run (2026-08-28), Real-Machine Stress Test Unrun

[中文](19-defer窗口下的假性ENOSPC.md) · **English** · [日本語](19-deferウィンドウ下の疑似ENOSPC.ja.md)

**Conclusions**:

1. **The criterion is computable, and empirically hits 36/36**:

   > **Spurious ENOSPC and checkpoint stall appear ⟺
   > Remaining space ≤ Churn per checkpoint × (delay generations + 1) + Single checkpoint overhead**

   Across 6 fill rate levels (0.90–0.999) × 6 delay levels (1–32) for 36 cells total, predictions and empirical measurements are **cell-by-cell identical**.
   This prediction **uses only workload parameters**, reading no fields whatsoever from the tested model.

2. **All 0 under 90% / 95% fill**—this problem exists only when disk is near full, not in normal states.

   | Fill Rate | Remaining Space | Minimum Delay Exhibiting Issue |
   |---|---|---|
   | 99% | 10000 blocks | **16** (delay 8 clear) |
   | 99.5% | 5000 blocks | **8** |
   | 99.9% | 1000 blocks | **1** (issue appears at minimum delay) |

3. **Spurious ENOSPC rate under 99.9% fill**: delay 1 → 0.375%; delay 8 → 3.875%; delay 16 → **7.875%**.

4. **The two failure modes must be tackled separately, cannot be combined into a single statement.**

   | Failure | What Cures It | Numbers |
   |---|---|---|
   | **Checkpoint stall** | **Reserve pool**, and `reserve ≥ ckpt_cost` suffices | Reserving **64 blocks** (precisely equal to `ckpt_cost`) drives `ckpt_stall` from **199 down to 0** |
   | **Spurious ENOSPC** | **Only acceptance**. Reserve pool aggravates it instead | Reserve 0 → 16384 blocks, spurious ENOSPC rises from 3150 to **7246**, delta per level exactly `reserve / file_blocks` |

   **Mechanism**: The reserve pool creates a floor for checkpoints, acting as pure tax on **normal allocations**.
   And the cause of spurious ENOSPC is "amount withheld by defer/K exceeding remaining space", which is outside the reach of the reserve pool.
   ⇒ Acceptance rules cure spurious ENOSPC: disk must not be filled above
   `Capacity − Churn × (Delay + 1) − Checkpoint Overhead`.

**Measurement basis**: Pure logic modeling, no devices, no file system. Capacity 1 million blocks (4 KiB/block ≈ 3.8 GiB),
4 blocks per temporary file, 200 "delete one + create one" operations per checkpoint window, running 200 checkpoints,
a single checkpoint writes 64 blocks itself. Code `scripts/experiment-apparatus/e19_defer.rs` (`cargo run --release --bin e19-defer`).
Raw output `e19-defer-2026-08-28.out`.

**The two numbers are calculated separately, not derived from the same formula**:
`df_free = Capacity − Active References` (what user sees);
`allocatable = Capacity − Active References − Held Back − Reserve Pool` (what allocator sees).
"Held back" is calculated by a **per-checkpoint accounting queue**.

### The 08-28 Round Caught Two Modeling Bugs of Its Own

| Bug | Symptom | Consequence |
|---|---|---|
| `allocatable()` directly subtracted using `u64` | **Underflowed to astronomical numbers** when held amount exceeded remaining space | Allocations counterintuitively always succeeded, entire table counterfeit |
| Reserve pool implemented as "shrinking capacity", while fill rate calculated against shrunk capacity | Reserve from 0 to 16384 showed **zero difference whatsoever** | Not a finding, but an implementation omission |
| **Stall criterion read wrong view**: written as `allocatable()` (**subtracting reserve pool**) rather than `allocatable_for_checkpoint()` | `allocatable_for_checkpoint()` became dead code, compiler continually warning `never used` | **The conclusion that "reserve pool cannot rescue" was entirely caused by this line.** Once properly wired, `reserve=64` drove stalls from 199 down to 0 |

**The second bug was exposed by results being "too neat"**—six reserve levels yielding identical numbers
is itself a signal that "this independent variable is not wired up".
**The third bug was caught by the backward leg of the three-way argument**: reading the code and discovering that method was never called.
⚠️ Root cause was a `str.replace` **silently failing to take effect** due to indentation mismatch, without read-back confirmation.

### One Gap in Discriminative Power to Note

`analytic_held` (formula estimate) and item-by-item accounting are **cell-by-cell completely equal** (800/800 … 12800/12800).
⚠️ **That "consistency" lacks discriminative power**: the workload in E19 (Spurious ENOSPC Under Defer Window) is completely regular (exactly 200 ops × 4 blocks per checkpoint),
so both paths compute the exact same arithmetic. **What has discriminative power is the acceptance one**—it predicts the binary trouble / no-trouble,
hitting 100% across 36 cells. For `analytic_held` to have discriminative power, workloads must be irregular (variable file lengths, variable rates).

### Real-Machine Stress Test (Unrun)

**Prerequisites**: Transaction layer and allocator. The modeling layer has already supplied the form of acceptance rules; real-machine tests will verify
how to estimate "churn" under real workloads—E19 (Spurious ENOSPC Under Defer Window) treats it as constant, whereas in real workloads it is variable.

### Affected Decisions

| Decision Item | Relationship | Review |
|---|---|---|
| D23 (Journal Role & Format) settled item 23 | Supports | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D23 (Journal Role & Format) settled item 24 | Supports | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D28 (Mount-Time Commitments) settled item 1 | Supports | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D28 (Mount-Time Commitments) settled item 4 | Supports | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |

## Historical Versions

Historical entries for E19 (Spurious ENOSPC Under Defer Window) are gathered in [experiments-history.md](../experiments-history.md).
