# Milestone "Scrub, Rescue, and Single-Disk Ditto": Tasks, Acceptance, and Ordering

[中文](16-sixteenth-txn.md) · **English** · [日本語](16-sixteenth-txn.ja.md)

**Measurement basis**: Planning document, pre-filed on 2026-10-12, serving as a draft. The user settled Milestones 7 through 10, asking "what might you have missed, how do you think milestones should be ordered", and the main agent suggested placing 11 through 15 sequentially, based on the principle that changes to on-disk formats are scheduled before format freeze, and freeze before external availability (`records/2026-10-11-里程碑五打包与决策进度列.md` Section 7); on the same day the user requested "write it into /home/fy5090/code/singlefs/.claude/kb/milestone, pre-fill it", settling this order: "this is acceptable, adjustments needed during execution". **It does not require correctness**: revise before starting each item. Scheduled after encryption ([15-fifteenth-txn.md](15-fifteenth-txn.md)) because what the keyless side can perform is settled by encryption (D9 (encryption) settled item 5). The prerequisite is that all exit criteria for Milestones 3 through 15 are fully met, with read-only scan reconstruction in Milestone 4 Section IV ([04-fourth-txn.md](04-fourth-txn.md)); if any item is missing when previous milestones close out, reschedule this document according to what is missing.

**How to read**: Three fields per item—"Existing in repo" lists which clauses, owed checks, and code this item bears down on in the repository (inspected on 2026-10-12, with sources); "Tasks" is a table: number, what to do, clauses and owed checks borne down upon, acceptance (prove each goes red first), dependencies; "To be settled before starting" lists open questions that must undergo three-way arbitration or ask the user before work begins. Anything labeled "inferred" is a judgment by the main agent without measurement. Task numbering is written as "Item·Point" (I·1, II·3), used solely within this document. Owed check numbers were all open on the day of filing, to be checked individually before starting work.

**One new transaction type added**: Rescue writeback (writing indices reconstructed from scanning back to disk), following the same scan reconstruction path settled by D21 (Boundary Between Authoritative and Derived States) settled item 13, opening no second publish path. Scrub is read-only without writing.

## Exit Criteria (Unsettled)

Exit criteria are left to user decision in Phase 0. Main agent's inferred candidates (inferred, unmeasured):

| # | Candidate Exit Criteria | Belongs to Which Item |
|---|---|---|
| 1 | Scrub: both full run and generational incremental, governed by clauses on post-detection action; executable without keys on encrypted volumes | I |
| 2 | Rescue writeback: built atop Milestone 4 read-only scan reconstruction, writing reconstruction results back, mountable, checker all green | II |
| 3 | Single-disk ditto: metadata multi-replica distributed placement, surviving single-disk corruption at one site | III |

## Grouping and Ordering

| Phase | What to Do | Why Sequenced Here | Status |
|---|---|---|---|
| 0 Handover | Check Milestone 4 read-only scan reconstruction and 15 keyless mount; settle exit criteria | Writeback builds on read-only reconstruction | Unstarted |
| 1 Scrub | I·1 → I·2 | Incremental builds on full run | Unstarted |
| 2 Rescue | II·1 | Depends on Phase 0 | Unstarted |
| 3 Ditto | III·1 | Modifies format, scheduled before freeze | Unstarted |

**Estimation**: Not estimated. Record `records/2026-10-11-里程碑五打包与决策进度列.md` Section 3 inferred 3 weeks combining these items with format freeze (inferred, unmeasured); not re-estimated after splitting.

## I. Scrub

- **Existing in repo**: D13 (verification route) settled item 8 "Merkle root chain difference (recursive comparison of roots of two adjacent generations, skipping drill-down where pointers unchanged) and generational incremental scrub (traversing only blocks with birth generation above high-water mark)", unimplemented; D9 (encryption) settled item 5 keyless side performs scrub only. `grep -rli scrub crates/*/src` hits zero. Owed checks: C408 (generational incremental scrub has zero code), C194 (generational incremental scrub blind to relocated replicas), C134 (scrub high-water mark lacks placement and protection), C33 (no clause for what to do after scrub detection), C23 (keyless scrub cannot distinguish live slots from stale slots), C18 (decoupled field pollutes read-only dimension).
- **Tasks**:

| # | What to Do | Borne Clauses and Owed Checks | Acceptance | Dependencies |
|---|---|---|---|---|
| I·1 | Full run scrub and handling after detection | D9 (encryption) settled item 5; C33 (no clause for what to do after scrub detection), C23 (keyless scrub cannot distinguish live slots from stale slots); C476 (21 reachable panics when on-disk content is controllable) (moved from Milestone 3 on 2026-10-19, attribution is inferred, verify before starting, remaining owed per registry rows); C861 (no one rewrites single bad unit replica released by mount) (registered 2026-10-19, attribution is inferred, verify before starting); C864 (misplaced reads of other self-verifying structures go unchecked) (registered 2026-10-20; the full run scrub reads every copy and checks the position binding along with it; settled by the main agent) | Image with one corrupt unit must be detected; running scrub from keyless side writes zero bytes | Phase 0 |
| I·2 | Merkle root chain difference and generational incremental scrub, high-water mark placement and protection | D13 (verification route) settled item 8; C408 (generational incremental scrub has zero code), C194 (generational incremental scrub blind to relocated replicas), C134 (scrub high-water mark lacks placement and protection), C18 (decoupled field pollutes read-only dimension), C211 (node on-disk pattern lacks body text), C389 (Merkle root chain difference has zero code) (moved from Milestone 3 on 2026-10-19, remaining owed per registry rows) | Incremental scrub and full run detect identical set on same image (differential integrated into quick tier); corrupted GC-relocated replica also detected by incremental scrub | I·1 |

- **Owed checks moved from Milestone 3** (2026-10-18; attribution is inferred, verify before starting): C598 (abandoned root unmountable after leaving root ring when bad sectors are not remapped) belongs to I·1 handling after detection: handling of bad sectors returning to free pool awaits clauses.

## II. Rescue Writeback

- **Existing in repo**: D21 (Boundary Between Authoritative and Derived States) settled item 13 "rescue, tree switching, migration share the same scan reconstruction path; the sole backward compatibility requirement is recognizing all unit formats that ever appeared in history"; D18 (information carried in a block) settled item 5 "reconstruction is a degraded path, categorized into three levels by obtainable witnesses; system configuration `map_provenance` reserved + read-only mount + in-place repair forbidden". Milestone 4 IV·1 performs only read-only comparison without writeback. Owed checks: C130 (scan reconstruction placement dimension lacks any metric), C184 (reconstruction path cannot acquire full nonce).
- **Tasks**:

| # | What to Do | Borne Clauses and Owed Checks | Acceptance | Dependencies |
|---|---|---|---|---|
| II·1 | Reconstructed result writeback: no in-place repair, write as a new generation; degraded mounts for each of three witness levels | D21 (Boundary Between Authoritative and Derived States) settled item 13; D18 (information carried in a block) settled item 5; C130 (scan reconstruction placement dimension lacks any metric), C184 (reconstruction path cannot acquire full nonce) | Image with index tree root removed mounts after rescue, differential against model item-by-item identical; mid-writeback crash whole-domain 0 red | Phase 0 |

- **To be settled before starting**: Whether rescue runs automatically during mount or via a standalone command.

## III. Single-Disk Ditto

- **Existing in repo**: D18 (information carried in a block) settled item 4 "adopts metadata multi-replica + distributed placement (ditto), chaining arm rejected; do not open single-device layout line initially; placement offset forbidden from power of 2". Owed checks: C201 (ditto placement offset forbidden from power of 2), C100 (inode container loss is permanent with no redundancy clause covering it).
- **Tasks**:

| # | What to Do | Borne Clauses and Owed Checks | Acceptance | Dependencies |
|---|---|---|---|---|
| III·1 | Under single disk, write metadata twice with distributed placement; read remaining copy if one copy corrupts | D18 (information carried in a block) settled item 4; C201 (ditto placement offset forbidden from power of 2), C100 (inode container loss is permanent with no redundancy clause covering it) | Mutation setting placement offset to power of 2 turns red; image with one corrupt copy reads back successfully | Phase 0 |

- **To be settled before starting**: Whether to reopen "do not open single-device layout line initially" (whether single-disk volume is a supported configuration in this version, settled by user).

**Not in exit criteria**: Format freeze (Milestone 17 [17-seventeenth-txn.md](17-seventeenth-txn.md)), multi-disk reconstruction and resynchronization (Milestone 13).
