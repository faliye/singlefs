## E22 Recording Named Ancestor Placement —— Unrun, and **Ruled Out by Axis 1 of D23 (Journal Role & Format), Relevant Only If Axis 1 Reopens**

[中文](22-记录点名祖先落点.md) · **English** · [日本語](22-レコードで指名した祖先の配置場所.ja.md)

**What it asks**: Can journal records naming ancestor placement avoid replay allocation?

⚠️ **The status of E22 (Recording Named Ancestor Placement) changed on 2026-08-28.** D23 (Journal Role & Format) settled item 1 settled: deferring ancestors ⇒ new root cannot be self-consistent
⇒ impossible to emit a root reflecting this fsync, directly conflicting with settled "emit root on every fsync".
**Option C similarly avoids writing ancestor contents and is thus excluded.** E22 (Recording Named Ancestor Placement) is relevant only if Axis 1 reopens;
it is no longer "the third candidate of Axis 2". The design of E22 (Recording Named Ancestor Placement) is preserved as-is for direct use then.

**Background**: **Axis 2 Candidate Option C** of [decisions.md](../decisions.md) D23 (Journal Role & Format) settled item 1.
It was forced out by the 2026-08-28 three-way argument round: the criterion for rejecting logical intent log's second rationale (placement uncertainty degrades checking, D23 (Journal Role & Format) settled item 5) was phrased as "whether replay runs the allocator",
whereas the real criterion is "**whether records + format can uniquely determine placement**". D23 (Journal Role & Format) settled item 20 settled "allocation precedes journal" ⇒ **ancestor placement is already known at fsync time**, simply omitted from records.

**Form to test**: fsync writes only dirty leaves + one record; the record carries **placement and checksum for the entire ancestor chain**,
omitting ancestor contents. Replay: read old ancestor → apply mutation → write to address named by record.
⇒ Picks no free blocks, strictly deterministic.

**Independent Variables**: Workload × fsync frequency × checkpoint interval × tree height (record size proportional to it).

**What to Measure**:
1. **Record size**: Each record ≈ `48 + (path_length + 1) × (placement + checksum)` bytes,
   compared cell-by-cell against Candidate Option B (naming leaves) and Candidate Option A (no deferral).
2. **Placement consumption when same ancestor repeatedly touched within a window**: whether each touch consumes a new placement, or can be reused.
   **This item is key to whether Option C holds**—if each touch requires a new placement,
   a hot directory will exhaust large volumes of placements within one checkpoint window.
3. **How "allocated but unwritten" placements avoid double-allocation after crash**:
   If the answer is "replay reads records to reconstruct that set" ⇒ no extra persistent structures needed;
   if not ⇒ Option C falls back to Candidate Option B side, and conclusion of E22 (Recording Named Ancestor Placement) is "Option C does not hold".

**Positive Control**: Run Candidates Option A and B on the same harness, reproducing
existing figures from [experiments.md](../experiments.md) E16 (Role of Journal: WAL vs Intent Log) (rand / per fsync: Option A 7 blocks, Option B 2 blocks).
Failure to reproduce indicates harness defect, **invalidating the entire round**—forbidden to record "Option C has no advantage" as conclusion.
⚠️ **Control must be run once for each of the three arms**, not just the first arm.

**What Counts as Failure**: If Option C's record size causes fsync block count to equal Candidate Option A, Option C gains nothing; faithfully record "Option C does not hold".

**Prerequisites**: None. Simply add an arm to E16 (Role of Journal: WAL vs Intent Log) harness (`scripts/experiment-apparatus/e16_journal.rs`).

### Impacted Decisions

| Decision Item | Relationship | Review |
|---|---|---|
| D23 (Journal Role & Format) settled item 1 | Input material | 2026-09-19 Unaffected: E22 (Recording Named Ancestor Placement) unrun, Option C structurally eliminated by Axis 1, design preserved as-is for when Axis 1 reopens |

## Historical Versions

E22 (Recording Named Ancestor Placement) history entries are consolidated in [experiments-history.md](../experiments-history.md).
