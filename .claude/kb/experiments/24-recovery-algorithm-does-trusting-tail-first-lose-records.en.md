## E24 Recovery Algorithm: Does Trusting Tail First Lose Records — Run (2026-08-29)

[中文](24-恢复算法先信tail会不会丢记录.md) · **English** · [日本語](24-リカバリアルゴリズム-tailを先に信用するとレコードを喪失するか.ja.md)

**This is not a performance experiment, but a safety experiment.** Clears [checks-owed.md](../checks-owed.md) C29 (Recovery trusting tail first loses data).
The hard requirement newly added in [decisions.md](../decisions.md) D23 (Journal Role & Format)
"Recovery must first scan the entire ring and verify everything, then select the longest valid prefix, and is forbidden from trusting tail first before proceeding"
was derived from mechanism, and had never been verified prior to this.

### Conclusions

**Red failure conditions were hardcoded upon registration** (`rules/test-discipline.md`: Failure clauses must not make conclusions unfalsifiable):
If "trusting tail first" never misses ⇒ that window does not exist, reasoning was flawed, the entire premise is void;
If "full ring scan" also misses ⇒ "stale tail merely causes extra work" is entirely void.

Ring of 64 blocks, writes 200 records first to wrap around, checkpoint completes but tail is **not** persisted, and within the window `gap` records are written:

| gap | Misses by Trusting Tail First | Misses by Full Ring Scan | **Spurious Replays** by Full Ring Scan |
|---|---|---|---|
| 0 | 0 | 0 | **64** |
| 1 | 1 | 0 | 63 |
| 5 | 5 | 0 | 59 |
| 20 | 20 | 0 | 44 |
| 63 | 63 | 0 | 1 |
| **100 (exceeds ring)** | 100 | **36** | 0 |

1. **Records missed by "trusting tail first" exactly equal the records written within the window**—it cannot recover a single record.
   Mechanism: The slot pointed to by the stale tail has already been overwritten by the new lap, so the very first record has `jsn != expected`, stopping immediately upon sequence break.
   ⇒ **The D23 (Journal Role & Format) hard requirement holds, and the penalty is all window records, not "a portion".**

2. **Full ring scan loses nothing, but it replays every record since the stale tail** (`spurious = ring_slots − gap`).
   This point **was unexpected, discovered on the spot when adding the "spurious replay" dimension**:
   The full ring scan only knows the **stale** persisted tail, and therefore replays everything findable on the ring above that tail.
   ⇒ **The statement "stale tail merely causes extra work" has a prerequisite: replay must be idempotent.**
   Without idempotence, it shifts from "doing extra work" to "reapplying already-effective records".
   That is an additional requirement never settled in this project, now recorded in D23 (Journal Role & Format).

3. **When the window blows out the ring, both arms lose together** (gap=100, ring 64 ⇒ full ring scan misses 36 = 100 − 64).
   ⇒ **Recovery algorithms cannot rescue geometry.** Guaranteeing ring size ≥ window is the responsibility of
   [checks-owed.md](../checks-owed.md) C28 (ring geometry invariant unimplemented), not the recovery algorithm.

### Measurement Basis and Known Limitations

- Pure counting model, no device, no transaction layer. Records contain only two fields: `jsn` and `ckpt`,
  **without modeling checksums, tears, or out-of-order writes**—"verifying a record" in the model simply means "slot is non-empty and jsn matches".
- Models only a single window (checkpoint complete → tail write), **without modeling multiple interleaving checkpoints**.
- The cost of `spurious` was not quantified, only counted in record tally—the actual consequence of replaying an already-effective record depends on idempotence,
  which is unsettled in this project.
- Code `scripts/experiment-apparatus/e24_recovery.rs` (`cargo run --release --bin e24_recovery`),
  raw output `e24-recovery-2026-08-29.out`. Three rounds byte-identical.

### Three Pitfalls in the 08-29 Round, All Caught by Tests Themselves

1. **Failed red on first run**: `gap=100` with ring 64, full ring scan missed 36 records. Not an algorithm issue, but the window blowing out the ring.
   ⇒ Split into two tests, writing the "window blowing out ring" case as an assertion expected to fail red,
   preventing it from being mistaken as a recovery algorithm flaw and patched in the wrong place later.
2. **Testing only stale scenarios ⇒ lower bound became dead code**. When `tail_persisted` is constant 1, the lower bound of full ring scan never activates;
   mutation testing deleted the lower bound completely without a single test failing red. Pinned only after adding healthy scenarios (tail persisted normally).
3. **Healthy scenario caught a bug in the model itself on the spot**: When maximum jsn in the ring is below persisted tail
   (just finished checkpoint, no new records written yet), full ring scan still returned that record, spuriously replaying 1 record.

**Engineering Quality**: 12 unit tests, **all 9 mutations caught**, three rounds byte-identical.

### Impacted Decisions

| Decision Item | Relationship | Review |
|---|---|---|
| D23 (Journal Role & Format) Settled Item 3 | Support | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |

## Historical Versions

Historical entries for E24 (Recovery Algorithm: Does Trusting Tail First Lose Records) are gathered in [experiments-history.md](../experiments-history.md).
