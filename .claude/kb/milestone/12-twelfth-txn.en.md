# Milestone "FUSE Mount": Tasks, Acceptance, and Ordering

[中文](12-twelfth-txn.md) · **English** · [日本語](12-twelfth-txn.ja.md)

**Measurement basis**: Planning document, pre-filed on 2026-10-12 per user settlement (verbatim: "POSIX goes to milestone 9, FUSE goes to milestone 10"; on the same day requested: "write it into /home/fy5090/code/singlefs/.claude/kb/milestone, pre-fill it"), serving as a draft. **It does not require correctness**: revise before starting each item. The prerequisite is that all exit criteria for milestones 3 through 11 are fully met (11 is [11-eleventh-txn.md](11-eleventh-txn.md): what FUSE mounts is a directory tree; without directories there is nothing to mount); if any item is missing when previous milestones close out, reschedule this document according to what is missing.

**How to read**: Three fields per item—"Existing in repo" lists which clauses, owed checks, and code this item bears down on in the repository (inspected on 2026-10-12, with sources); "Tasks" is a table: number, what to do, clauses and owed checks borne down upon, acceptance (prove each goes red first), dependencies; "To be settled before starting" lists open questions that must undergo three-way arbitration or ask the user before work begins. Anything labeled "inferred" is a judgment by the main agent without measurement. Task numbering is written as "Item·Point" (I·1, II·3), used solely within this document. Owed check numbers were all open on the day of filing, to be checked individually before starting work.

**No new transaction types added**: The FUSE daemon translates requests from the kernel into existing transactions; page cache writeback, fsync, flush, and release all map onto the same publish interface (D16 (Publish Semantics) settled item 11 "cannot bypass journal"), opening no second publish path.

**External gate**: `.claude/rules/format-evolution.md` states "before the first external user appears, disk formats are soft", with D15 (Format Freeze Policy) governing freezing. The mount produced here is only for in-repo testing and local machine workloads; it must undergo format freeze before handover to external users (Milestone 17 [17-seventeenth-txn.md](17-seventeenth-txn.md)).

## Exit Criteria (One Settled, Remaining Unsettled)

The user settled one exit criterion on 2026-10-12 (verbatim: "milestone 10 adds an exit criterion: execute experiment 152"):

| # | Exit Criteria | Belongs to Which Item |
|---|---|---|
| 1 | Execute E152 (Per-Milestone File Performance Against Six File Systems): replace singlefs arm with FUSE mount, running the same workload set in the same official run against six systems (two-disk mirror, seven systems + singlefs, 5 rounds each), commit artifacts into repository, add this milestone's section to per-milestone comparison table `scripts/experiment-apparatus/perf-by-milestone.md` | IV |
| 2 | Crash full run: finish what is new, then catch up on the backlog, at least 10 billion states per milestone (settled by user on 2026-10-22). Run in two stages (recording stage on CPU, judging stage on GPU, all observers attached), without narrowing the enumeration domain, all 0 red. ① First run the crash streams new in this milestone and the blocks whose unit key changed; ② then catch up from the backlog registry (set up in Milestone 4), at least 5 billion states, in registry order, resuming where the previous run stopped. When new and backlog together exceed 10 billion states, this milestone runs at least 10 billion states and the part of the new work that does not fit goes to the end of the backlog registry; when together they come to less than 10 billion, run all of it. On red, pin it back into the quick tier per the "A red in a full run is pinned back into the quick tier" section of `.claude/rules/test-script-discipline.md`, fix it, then rerun. | Crash full run (cross-milestone) |

Remaining exit criteria and which xfstests subset to run are left to user decision in Phase 0. Main agent's inferred candidates (inferred, unmeasured):

| # | Candidate Exit Criteria | Belongs to Which Item |
|---|---|---|
| 1 | Formulate and settle decisions on how page cache, fsync, flush, and release map to each step | I |
| 2 | FUSE daemon mounts successfully, fully exercising the settled POSIX operation set from Milestone 11 | II |
| 3 | Settle and pass xfstests subset, logging reasons for any skipped or failed cases individually | III |
| 4 | Crash injection atop FUSE: run real application workloads on QEMU real devices, power cut, remount, reading contents within the set permitted by RefFS | III |

## Grouping and Ordering

| Phase | What to Do | Why Sequenced Here | Status |
|---|---|---|---|
| 0 Handover | Check operations supported by `crates/`; choose FUSE library (adds new dependency, arbitrate third-party code location per D17 (Implementation Layering & Third-Party Pipeline)); settle exit criteria | Library selection dictates how the daemon is written | Unstarted |
| 1 Establish Decisions | I·1 | Write daemon only after mapping is settled | Unstarted |
| 2 Implementation | II·1 → II·2 | | Unstarted |
| 3 Coverage | III·1 ∥ III·2 | Both depend on II | Unstarted |
| 4 Comparison | IV·1 → IV·2 → IV·3 | IV·1 and IV·2 only depend on II·1 mount succeeding, can run in parallel with Phase 3; IV·3 official run requires II·2 working, during which the local host does not compile or run gates | Unstarted |

**Estimation**: Not estimated. Record `records/2026-10-11-里程碑五打包与决策进度列.md` Section 3 inferred 3–4 weeks (inferred, unmeasured).

## I. Mapping Between Page Cache and Publish

- **Existing in repo**: D16 (Publish Semantics) settled item 11 "all structures and all operations enter the same journal first, sharing the same checkpoint cadence; fsync merely triggers an early publish"; D16 (Publish Semantics) settled item 17 "after reporting an error, this publish may have succeeded or failed; callers must not assume old memory state can be reused for retry; whether it succeeded is judged by caller reading the active root"; D16 (Publish Semantics) settled item 2 triggering criteria for checkpoint. No decision across the repository governs page cache dirty pages, FUSE flush and release, `O_DIRECT`, or mmap writes. `grep -n 'fuse' crates/*/Cargo.toml Cargo.toml` returns zero hits. Owed checks: C6 (Wrong Block Layer Semantic Assumptions Written).
- **Tasks**:

| # | What to Do | Borne Clauses and Owed Checks | Acceptance | Dependencies |
|---|---|---|---|---|
| I·1 | Formulate decisions: how dirty page writeback, fsync, fdatasync, flush, release, and mmap writes each map to publish steps; what the daemon reports to the kernel after fsync reports an error | D16 (Publish Semantics) settled items 2, 11, 17 | Three-way verdicts land in `research/prompts/`, settled text written back to kb; decisions require experiments | Phase 0 |

- **To be settled before starting**: Whether mmap writes and `O_DIRECT` enter this document.

## II. Daemon

| # | What to Do | Borne Clauses and Owed Checks | Acceptance | Dependencies |
|---|---|---|---|---|
| II·1 | FUSE daemon: mounting, translating requests into transactions, error code mapping | I·1 settled decisions; D17 (Implementation Layering & Third-Party Pipeline); C370 (demand, available, and df have no common unit), C371 (Live Metadata Undefined), C571 (whether df counts empty slots locked in clustered segments, pending pseudo-ENOSPC) (moved from Milestone 3 on 2026-10-19, attribution is inferred, verify before starting, remaining owed per registry rows) | One test per request type, prove red first | I·1 |
| II·2 | Exercise Milestone 11 settled operation set individually on mount; no loss on power cut after fsync returns success | D16 (Publish Semantics) settled item 11 | Same as II·1 | II·1 |

## III. External Testing and Crash Injection

| # | What to Do | Borne Clauses and Owed Checks | Acceptance | Dependencies |
|---|---|---|---|---|
| III·1 | xfstests subset, plus real application workloads (compiling a repo, running a database, settled in Phase 0) | None | All subset tests pass; individually record reasons for any unpassed tests | II |
| III·2 | Crash injection atop FUSE on QEMU real devices: power cut, remount, verify read content falls within set permitted by RefFS | C6 (Wrong Block Layer Semantic Assumptions Written); D13 (Verification Route) settled item 9 | Turn red when intentionally omitting one FUA; whole-domain judge connected to checker tier | II |

## IV. E152 (Per-Milestone File Performance Against Six File Systems) Comparison with Six Systems

- **Existing in repo**: E152 (Per-Milestone File Performance Against Six File Systems) has run seven times, most recently 2026-09-29 (two-disk mirror, seven systems + singlefs); the singlefs arm is the real device binary from gate checker-tier-qemu-device-streams (`new_pool_file_creation_on_device` compiled in `scripts/devices/e152-run.sh`), lacking file interfaces; among eight dimensions only small file persistence and cold mount have numerical results, with large file, 4K random, metadata, formatting, and capacity all "—" (Experiment Page "Questions it cannot answer" item 4); the experiment page registers it to run only when requested by user, and this exit criterion is precisely user requesting it. Pre-run registration `e152-preregistration.md` is not in tree, retrieve from git history.
- **Tasks**:

| # | What to Do | Borne Clauses and Owed Checks | Acceptance | Dependencies |
|---|---|---|---|---|
| IV·1 | Revise pre-run registration: replace singlefs arm from real device binary with FUSE mount, running identical fio workloads across eight dimensions against six file systems; specify how FUSE layer overhead is reported separately from singlefs itself (inferred: add an extra arm "same daemon mounting pass-through dummy implementation" as control) | `.claude/singlefs-ai-sop/rules/test-discipline.md` "Before experiment runs, answers must not already exist", "Positive control must run for every tested arm"; E152 (Per-Milestone File Performance Against Six File Systems) experiment page "Measurement basis" | Revisions written before run; criteria, thresholds, invalidation clauses pinned | II·1 |
| IV·2 | Apparatus: `scripts/devices/e152-stage-root.sh` places daemon into guest, `scripts/devices/e152-run.sh` singlefs configuration changed to mount FUSE running fio; singlefs row in `scripts/experiment-apparatus/e152-tables.py` updated across eight dimensions; unit tests and mutation tables added accordingly | E152 (Per-Milestone File Performance Against Six File Systems) mutation table `scripts/mutation/tables/e152_file_system_benchmark.tsv` | Unit tests prove red first, mutations fully caught; single configuration smoke test runs `vm_exit=0` once | IV·1 |
| IV·3 | Official run: two-disk mirror, seven systems + singlefs, 5 rounds each; commit artifacts to repo, record this run on experiment page, add this milestone section to `scripts/experiment-apparatus/perf-by-milestone.md` (generated by `scripts/experiment-apparatus/e152-tables.py`); concurrently review D17 (Implementation Layering & Third-Party Pipeline) settled item 5 and D23 (Journal Role & Format) two-row preparations | E152 (Per-Milestone File Performance Against Six File Systems) experiment page "Influenced Decisions" | All VM runs report `vm_exit=0`, failed rounds recorded per pre-run registration invalidation clauses; gap explaining how singlefs should change is an inference, requiring separate three-way arbitration | IV·2; II·2 |

- **To be settled before starting**: Run single disk or only two-disk mirror; whether FUSE pass-through control arm is needed (pinned in IV·1).

**Not in exit criteria**: Handover to external users (after format freeze), encryption (Milestone 15), multi-disk striping (Milestone 13).
