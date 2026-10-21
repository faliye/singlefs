# Milestone "Multi-Disk Striping and Device Set Changes": Tasks, Acceptance, and Ordering

[中文](13-thirteenth-txn.md) · **English** · [日本語](13-thirteenth-txn.ja.md)

**Measurement basis**: Planning document, pre-filed on 2026-10-12, serving as a draft. The user settled Milestones 7 through 10, asking "what might you have missed, how do you think milestones should be ordered", and the main agent suggested placing 11 through 15 sequentially, based on the principle that changes to on-disk formats are scheduled before format freeze, and freeze before external availability (`records/2026-10-11-里程碑五打包与决策进度列.md` Section 7); on the same day the user requested "write it into /home/fy5090/code/singlefs/.claude/kb/milestone, pre-fill it", settling this order: "this is acceptable, adjustments needed during execution". **It does not require correctness**: revise before starting each item. The prerequisite is that all exit criteria for Milestones 3 through 12 are fully met; if any item is missing when previous milestones close out, reschedule this document according to what is missing.

**How to read**: Three fields per item—"Existing in repo" lists which clauses, owed checks, and code this item bears down on in the repository (inspected on 2026-10-12, with sources); "Tasks" is a table: number, what to do, clauses and owed checks borne down upon, acceptance (prove each goes red first), dependencies; "To be settled before starting" lists open questions that must undergo three-way arbitration or ask the user before work begins. Anything labeled "inferred" is a judgment by the main agent without measurement. Task numbering is written as "Item·Point" (I·1, II·3), used solely within this document. Owed check numbers were all open on the day of filing, to be checked individually before starting work.

**Two new transaction types added**: Geometric re-evaluation (D2 (RAID Stripe Policy) settled items 3, 4) and rebalancing relocation; rebalancing follows the intent mechanism of Milestone 4 GC, opening no second publish path.

## Exit Criteria (Unsettled)

Exit criteria, how many disks in version 1, and parity cell count are left to user decision in Phase 0. Main agent's inferred candidates (inferred, unmeasured):

| # | Candidate Exit Criteria | Belongs to Which Item |
|---|---|---|
| 1 | `w ≥ 3` striping lands: membership table, parity column, stripe table tree, on-disk patterns registered in `.claude/kb/layout/` with filename starting with `13-` | I |
| 2 | Device set changes: adding disks, draining, geometric re-evaluation, rebalancing, volume remains mounted | II |
| 3 | Degradation on disk loss: when dropping to devs < R, degraded mount per D22 (Synthesizing Unit Atomicity) settled item 14; resynchronization upon disk return | II |
| 4 | Whole-domain 0 red for crash streams of striping and device changes | III |
| 5 | After enabling parity, measure write amplification against Milestone 9 apparatus, writing numbers back (added per user 2026-10-12 decision) | III |

## Grouping and Ordering

| Phase | What to Do | Why Sequenced Here | Status |
|---|---|---|---|
| 0 Handover | Check current status of two-disk mirror; settle exit criteria and parity cell count (D2 (RAID Stripe Policy) unsettled item 21) | Geometry settled only after parity cell count is fixed | Unstarted |
| 1 Striping | I·1 → I·2 | Membership table precedes lifecycle | Unstarted |
| 2 Device Changes | II·1 → II·2; II·3 | Adding disks and rebalancing require striping in place | Unstarted |
| 3 Coverage | III·1 ∥ III·2 | Both depend on I, II | Unstarted |

**Estimation**: Not estimated. Record `records/2026-10-11-里程碑五打包与决策进度列.md` Section 3 inferred 4–6 weeks (inferred, unmeasured).

## I. Striping

- **Existing in repo**: Version 1 is strictly two-disk mirror `w = 2` (scope sentence of D2 (RAID Stripe Policy) unsettled item 21). Unimplemented items: D2 (RAID Stripe Policy) settled item 8 "when creating objects select a group from the g emptiest blocks, thereafter allocate exclusively from the w emptiest blocks within the group"; settled item 11 "partial stripes pinned, background lazy reallocation; member cells cannot be reused before repair, w = 2 stripe members reclaimed entirely once one dies"; settled item 12 "membership table fixed 56 bytes residing in Code 3 packed record, identified by container index, container chain strictly `w = 2` mirrored"; settled item 14 "acquire placement (device identity 4 + 16 KiB slot number 6), sharing identical coordinate system with allocation record tree of D3 (space allocation) settled item 3". Owed checks: C103 (container index has no placement), C154 (stripe membership table has not registered packed record type), C81 (no rule for partial stripe release), C90 (ENOSPC ratchet under pinned stripes), C127 (pinned state has no placement in allocation records), C128 (sweep acceptance cannot judge either branch of parity cells), C181 (in-pool heterogeneity criterion missed bitwise XOR stripe axis).
- **Tasks**:

| # | What to Do | Borne Clauses and Owed Checks | Acceptance | Dependencies |
|---|---|---|---|---|
| I·1 | Membership table, parity column, stripe table, group selection | D2 (RAID Stripe Policy) settled items 8, 12, 14 and unsettled item 21; C103 (container index has no placement), C154 (stripe membership table has not registered packed record type), C181 (in-pool heterogeneity criterion missed bitwise XOR stripe axis); C574 (fit judgment does not check two-slot alignment and same-slot across disks) (moved from Milestone 3 on 2026-10-19, attribution is inferred, verify before starting, remaining owed per registry rows) | New invariants enter pool-level checker, bad images one each; mutation with 1 bit wrong in parity turns red | Phase 0 |
| I·2 | Stripe lifecycle: partial release pinning, lazy reallocation, member cells unreusable before repair | D2 (RAID Stripe Policy) settled item 11; C81 (no rule for partial stripe release), C90 (ENOSPC ratchet under pinned stripes), C127 (pinned state has no placement in allocation records), C128 (sweep acceptance cannot judge either branch of parity cells) | Mutation reusing member cell before repair turns red; full writes under pinned stripes avoid pseudo-ENOSPC | I·1 |

## II. Device Set Changes and Disk Loss

- **Existing in repo**: D2 (RAID Stripe Policy) settled item 3 "execute explicit 'geometric re-evaluation' transaction and publish atomically, volume remains mounted, forced remount forbidden"; settled item 4 "reject adding disk if geometry unqualified, drain before altering geometry, run background resumable rebalance after adding disk, relocation unit is whole object"; settled item 5 "root ring not repositioned; rebalance relocates user data leaving root ring untouched, accepts degradation when disk loss drops devs < R"; D22 (Synthesizing Unit Atomicity) settled item 14 root ring verdict when devs < R (Milestone 9 II·3 implements single disk and two-disk branch first). Owed checks: C17 (device geometry treated as global parameter), C55 (geometry changed before draining is fully clean), C82 (io_min not in mount comparison checklist), C120 (verdict and resynchronization on forked disk return), C126 (worst-case switch reserve has no measurement basis), C346 (background reclustering has no definition).
- **Tasks**:

| # | What to Do | Borne Clauses and Owed Checks | Acceptance | Dependencies |
|---|---|---|---|---|
| II·1 | Adding disks, draining, geometric re-evaluation transactions | D2 (RAID Stripe Policy) settled items 3, 4; C17 (device geometry treated as global parameter), C55 (geometry changed before draining is fully clean), C82 (io_min not in mount comparison checklist), C126 (worst-case switch reserve has no measurement basis); C553 (no clause for disk resizing between mounts) (settle support first, supporting it is a device geometry change, attribution is inferred, verify before starting); C368 (allocator placement checks only disk 0, assertion fails when disks unequal in size), C582 (no clause for which lower bound on mount disk capacity to take) (moved from Milestone 3 on 2026-10-19, attribution is inferred, verify before starting, remaining owed per registry rows) | Mutation changing geometry before draining clean turns red; geometric re-evaluation midpoint crash whole-domain 0 red | I |
| II·2 | Rebalance: background resumable, relocates whole objects, root ring untouched | D2 (RAID Stripe Policy) settled items 4, 5; C346 (background reclustering has no definition) | Differential against model item-by-item identical before and after rebalance | II·1 |
| II·3 | Degraded mode on disk loss and resynchronization upon return | D2 (RAID Stripe Policy) settled item 5; D22 (Synthesizing Unit Atomicity) settled item 14; C120 (verdict and resynchronization on forked disk return); C739 (no clause for which class to take when two replicas prove different classes) (moved from Milestone 3 on 2026-10-19, attribution is inferred, verify before starting, remaining owed per registry rows) | Mutation mounting old timeline when forked disk returns turns red | I |

- **Owed checks moved from Milestone 3** (2026-10-18; attribution is inferred, check individually before starting): C683 (pool-level checker lacks reading method for "one disk dropped") and related C695 (version with 0 tree table entries unreadable on single disk) belong to II·3 disk loss degradation, settling which version single disk survivor selects here; C652 (enabling w >= 3 before redundancy form is settled) belongs to Phase 0: parity cell count and redundancy form must be settled before opening up w ≥ 3.

## III. Crash Coverage

| # | What to Do | Borne Clauses and Owed Checks | Acceptance | Dependencies |
|---|---|---|---|---|
| III·1 | Four streams (stripe write, geometric re-evaluation, rebalance, disk loss) connected to crash amplification, multi-disk geometry also tested in QEMU real devices | D13 (verification route) settled items 9, 15 | Whole-domain 0 red | I, II |
| III·2 | After opening parity, re-measure write amount per fsync using apparatus of Milestone 9 ([09-ninth-txn.md](09-ninth-txn.md)) I·1, comparing against Milestone 9 two-disk mirror measurements; write changed numbers back to D25 (Target Workload Priority) table | Milestone 9 I·1, III·1 | Measure 5 rounds each for identical workload before and after modification, commit artifacts to repo | I |

**Not in exit criteria**: Cloning (Milestone 14 [14-fourteenth-txn.md](14-fourteenth-txn.md)), parity calculated over ciphertext under encryption (D9 (encryption) settled item 17, along with Milestone 15), zoned / FDP.
