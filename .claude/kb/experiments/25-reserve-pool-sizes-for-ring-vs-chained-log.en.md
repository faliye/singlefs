## E25 Reserve Pool Size for Ring vs Chain — Run (2026-08-29)

[中文](25-环与链各要多大保留池.md) · **English** · [日本語](25-リングと連鎖型で各々必要な予約プールのサイズ.ja.md)

**Clears [checks-owed.md](../checks-owed.md) C30 (chained reserve pool quantification unverified).**
When ruling "adopt fixed ring", [decisions.md](../decisions.md) D23 (Journal Role & Format) used a quantification—
under chained log, the reserve pool must expand from `ckpt_cost` to `ckpt_cost + journal blocks within one checkpoint window`.
**That formula was derived by calculation, never verified.** E25 (Reserve Pool Size for Ring vs Chain) turns it into an executable model and sweeps tiers.

### Conclusions: Deadlock Vanishes Exactly at Predicted Tiers, Both Arms Match Individually

Capacity 100,000 blocks, 20,000 operations, checkpoint every 100 operations, `ckpt_cost = 64`, 1 block per operation
⇒ journal blocks per window = 100. Initial occupancy 99%.

| reserve | Checkpoint Deadlocks on Ring | Checkpoint Deadlocks on Chained |
|---|---|---|
| 0 / 32 | 95 | 120 |
| **64** | **0** | 120 |
| 100 / 128 | 0 | 120 |
| **164** | 0 | **0** |
| 256 / 512 | 0 | 0 |

⇒ **Ring reaches zero at `reserve = 64`** (= `ckpt_cost`);
**Chained reaches zero at `reserve = 164`** (= `ckpt_cost + window journal blocks = 64 + 100`).
Bit-for-bit matches the formula derived in D23 (Journal Role & Format).

**Negative Control**: At 95% initial occupancy (5000 blocks free space), **neither arm deadlocks at any reserve level**
⇒ Effect indeed stems from space pressure, not a modeling artifact.

### A Narrowing of the Conclusion in E19 (Spurious ENOSPC Under Defer Window)

E19 (Spurious ENOSPC Under Defer Window) stated that "reserve pools are pure tax on normal allocations".
**That statement has an implicit prerequisite: the reserve pool is already large enough to avoid deadlock. Outside that prerequisite, its sign flips.**

| Range | reserve | Normal Allocation Refusal Count |
|---|---|---|
| Deadlock regime | 0 | **9340** |
| Non-deadlock regime | 64 | 756 |
| Non-deadlock regime | 512 | 1204 |

After deadlock, space is never reclaimed, and refusal count is dominated by deadlock ⇒ **smaller reserve pools paradoxically reject more**.
This was exposed on the spot when the first version of E25 (Reserve Pool Size for Ring vs Chain) tests was written incorrectly (the original assertion was "larger reserve rejects more", comparing 0 with 512, which reversed). Each regime has a pinned test.

### Measurement Basis and Known Limitations

- Pure counting model, no device, no file system. **Must have net leakage per window** (model assumes 8 blocks),
  otherwise checkpoints return all consumed space, free space perpetually resets to start, and experiment measures nothing—
  initial version was exactly like this, failing three of six tests red. Leakage itself is real:
  D16 (Publish Semantics) settled "releases within checkpoint forbidden from entering allocatable set prior to publish",
  plus D22 (Synthesizing Unit Atomicity) root ring K generations retention.
- **Does not model false ENOSPC**, only modeling checkpoint deadlocks and normal allocation refusals.
  Division of concerns between the two is in E19 (Spurious ENOSPC Under Defer Window).
- Code `scripts/experiment-apparatus/e25_journal_reserve.rs` (`cargo run --release --bin e25_journal_reserve`),
  raw output `e25-journal-reserve-2026-08-29.out`. Three rounds byte-identical.

**Engineering Quality**: 7 unit tests, **all 7 mutations caught**, three rounds byte-identical.

### Impacted Decisions

| Decision Item | Relationship | Review |
|---|---|---|
| D23 (Journal Role & Format) Settled Item 2 | Support | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D23 (Journal Role & Format) Settled Item 24 | Support | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D16 (Publish Semantics) | No impact | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D22 (Synthesizing Unit Atomicity) | No impact | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |

## Historical Versions

Historical entries for E25 (Reserve Pool Size for Ring vs Chain) are gathered in [experiments-history.md](../experiments-history.md).
