## E180 Acceptance Reading vs Truly Writable Amount —— Not run (phase-one reading rig, 45 unit tests, 22 mutation rows and the sharded long-run script are written; both long reading runs did not finish and there is no artifact; the phase-two pinned-slot rig is not written)

[中文](180-准入读数与真可写量.md) · **English** · [日本語](180-受入読み取り値と真の書き込み可能量.ja.md)

<!-- doc-lint:not-numbers F2 F5 F6 F8 W0 W1 W2 W3 W4 R1 R2 R3 K0 K1 K2 K3 S1 S2 S3 Q1 Q2 Q3 Q4 Q5 Q6 Q7 Q8 Q9 Q10 Q11 Q12 Q13 X1 X2 X3 X4 X5 X6 X7 A1 A3 A4 B1 B2 B3 B4 -->

Pre-run registration `research/prompts/e180-preregistration.md` (two phases: phase one, the reading phase, answers F2, F8 and the half of F5 about "histories with abandoned roots and net release"; phase two, the pinned-slot phase, answers F6 and the half of F5 about "histories of pinned slots"). It answers four rows of the fork sheet `research/prompts/m3-close-admission-and-shadow-ledger-forks.md`: F2 (which formula `df` follows, with live metadata following it: W1 follows the literal `df` of D16 (Publish Semantics) settled item 1; W2 follows the formula of D28 (Mount-Time Commitments) for available, taking the smaller across disks; W3 adds the available of both disks divided by the number of copies; W4 takes W2 and further subtracts the upper bound of the non-data demand of the one write of n units), F5 (where the C356 (Item 9 calculated by difference of statistics, negative on net release without clamp) row goes: R1 pay it off, R2 rewrite the check, R3 keep it), F6 (the pin after a failed raise of F: K1 the reading counts the pin as occupied, K2 release as soon as every disk has a root carrying the current F, K3 push empty publishes that keep F unchanged until every disk has a root carrying F, then release), and F8 (whether `df` reports empty slots closed inside a segment: S1 report, S2 do not report, S3 follow S2 and additionally report an item "available after remount"). F9, F10, F11 and F14 of the same fork sheet are in E181 (Landing Determination and Mount Lower Bound), and B1, B2 and B4 are in E182 (Three Forks for Unreadable Abandoned Roots).

The background is six owed checks: C370 (demand, available, and df have no common unit), C371 (Live Metadata Undefined), C356 (Item 9 calculated by difference of statistics, negative on net release without clamp), C546 (held slots unreturned when raising F rejected), C570 (slots held by raising F reclamation idle in count but allocator does not issue), and C571 (whether df counts empty slots locked in clustered segments, pending pseudo-ENOSPC). The clauses under test are D16 (Publish Semantics) settled item 1 (`df`, acceptance, the sequence that raises F, normal unmount), D28 (Mount-Time Commitments) settled item 1 (the nine-item formula) and settled item 3 (the sentence "`df` reports available"), and D3 (Space Allocation) settled item 12 (the two gates in series); the definitions they cite are D3 (Space Allocation) settled item 8 (a clustered segment is closed to user data) and settled item 9 (the definition of a spurious ENOSPC).

The rig is the checked-in rig `crates/singlefs-checker-tier/src/bin/e180_admission_reading_versus_writable_bytes.rs` (read-only; it only drives and observes today's session publish path, acceptance reading and allocator in `crates/`, and does not change `crates/singlefs-core` or `crates/singlefs-checker`; every quantity that needs a constant from `crates/` uses a local constant whose value is copied from the kb registration site, and at the start of a run it is compared back against the implementation side). This one has phase one only; phase two (F6's cell construction, five-arm retry and crash-point enumeration) is not in it.

**Current state**:

- The phase-one rig is written; in the source file `grep -c '#\[test\]'` = 45 (first 16 hex digits of sha256 `2c0bc641d1f0a0d5`). The unit-test reading recorded in revision 4 of registration section twelve is for the version moved into the main workspace (sha256 `890dde7b…`): `test result: ok. 45 passed; 0 failed`; afterwards a clippy fix and two rename patches changed the rig again, and this page has no unit-test reading for the current version. Unit tests are counted per package under the checker tier and are run by the main agent with the prefix.
- 22 mutation rows are appended at the end of `crates/mutations.tsv` (names start with lowercase `e180`, `grep -c '^e180 '` = 22): the 7 that phase one covers in registration section nine plus 5 added by the rig itself are the 12 recorded in revision 3.6, and the other 10 are the on-demand tightening of revision 4 (4 rows) and the shard merge (6 rows); the three numbers await the gate checker-tier-crates-mutation-replay at commit time.
- Sharded long-run script `scripts/experiment-apparatus/e180-reading-shards.sh`: phase one is cut into 29 shards (5 geometries × the five families HF, HO, HU, HM and HC, plus HL for the 4 geometries other than G-unequal); when the shards are complete the rig's `merge` subcommand combines them into one artifact; at the start of a run it copies the rig program file and pins it, each shard records the rig sha256 that produced it, and a resume only reuses shards from the same version (the fix for owed check C862 (Long-running sharded resume does not verify which version apparatus is)).
- Both long reading runs did not finish: in the first run 9 shards were not done (the memory wrapper could not be scheduled or hit its cap); in the resumed run the rig binary at the repo root was replaced by another build, so shards from several rig versions would have merged into one artifact, and the main agent stopped it (`records/2026-10-18-里程碑三收尾与近期欠账.md` lines 230 and 310); the shards are outside the repo and not checked in. There is no artifact for E180 (Acceptance Reading vs Truly Writable Amount) under `research/results/`, and the raw output is not retained (not one run finished); `scripts/experiment-apparatus/replay.sh` also has no registration row for it.
- The phase-two (pinned-slot phase) rig is not written. This page has no reading for the four rows F2, F5, F6 and F8, so those four rows of the fork sheet are still open.

### What is measured

- **State**: within one mount, the moment between two user changes of the session (after `publish_user_change` returns); a writable mount that has just been made also counts as one. States are built only through the public entry points of the core layer: mkfs, writable mount, `MountedSession::publish_user_change`, normal unmount, crash built by truncating a recorded stream, and error injection on device calls.
- **T (the largest number of bytes writable at this moment)**: on a copy of this state, `UserChange::SequentialWrite` replaces the first file wholesale with content of n × 32634 bytes; the largest n that succeeds through the session (including the raise of F the session pushes for lack of space), times 32634. T counts only when the reason for refusal is lack of space; a state refused for another reason records T as "unobtainable".
- **Over-report and under-report**: candidate c reports X_c bytes on a state; ⌈X_c ÷ 32634⌉ > T ÷ 32634 is an over-report, with over-report amount X_c − T; under-report is T − X_c (when > 0).
- **Geometry** (default ring 768 MiB, io_min 4096): G-default (two 4 GiB disks, S = 8), G-unequal (4 GiB and 3 GiB + 33 slots), G-narrow (two 3 GiB + 33 slots); for geometry sensitivity G-default-S4 and G-default-S16 are also taken.
- **History families** (each family is one fixed script, with seeds fixed in registration section five): HF fill up, HO overwrite, HU delete then write, HM multiple mounts, HC abandoned roots and net release, HL long mount.

### Criteria (registration section six, fixed before the run)

- Q1: for each F2 arm on each geometry cell, the number of over-reporting states; = 0 is judged "no over-report", ≥ 1 is judged "over-report"; Q2 and Q3 report the largest over-report bytes and the upper bound of under-report; Q4 is the number of states where T is unobtainable, and when it exceeds 10% of the states in that cell, Q1–Q3 of that cell are not judged.
- Q5: the number of states where the F5 R2 residual is not 0, split by the pinned-slot family and the abandoned-root and net-release family; when both families are = 0 the verdict is "R2 can be written"; Q6 reports the residual side by side with the number of "pinned and quarantined" slots; Q7: the number of states where the R3 residual is not 0, ≥ 1 is judged "the original formula of the owed check is false on reachable states".
- Q8–Q11 (phase two): for each F6 cell (F in effect, F not in effect), the number of cases in which the one retried write of each arm succeeds; the number of crash states "covered" in the crash window after K0, K2 and K3 release (both disks, only disk 0 left, only disk 1 left reported separately); the number of extra empty publishes K3 pushes; how many cases each of the two kinds of cell produces.
- Q12: for each geometry cell and each disk in F8, the peak, the number of positive states and the end value of "empty slots closed inside a segment − ring window"; peak > 0 is judged "S1 over-reports on reachable states"; Q13 reports the peak of the closed empty slots themselves.
- Positive controls: for each F2 arm, max(X_c, T) + 32634 must be judged an over-report; W0, known to over-report (not dividing by the number of copies, not subtracting the committed amount), must be judged an over-report on states with T > 0; X = 0 must not be judged an over-report. For F5's R2, removing the pinned term and removing the overlap term must each leave a nonzero residual on the corresponding states. F8's counter, in segment [50240, 50304) open on disk 0 after mkfs, must count exactly 52 closed empty slots. If any one fails, that phase is void.
- Revision 1: for a state where T is truncated by the journal record cap (default ring 65536 records), a lower bound L is recorded; a candidate that needs ≤ L units is reported "certainly no over-report" and > L is reported "unsettled", and neither enters the over-report count; revision 4 changed this to tighten the lower bound only on the cells where the judgment could change, and to run phase one in 29 shards and then merge. Not a word of the judgment changes.

### Failure clauses (registration section ten)

- X1: all F2 candidates over-report on the same geometry cell: that cell hits a non-discriminating arm, and is not used to judge arms.
- X2: some F2 arm over-reports on one of two geometry cells in opposite directions and does not on the other: recorded as "unstable".
- X3: Q5 ≥ 1 in either family: the two-term formula of R2 cannot be written.
- X4: after K2 or K3 releases, units referenced by the root recovery picks are covered in the crash window: that arm is out.
- X5: even K0 (release immediately when a failed write is returned) did not cover: F6 is handed in by Q8 and Q10 only.
- X6: the cell where F is not in effect cannot be built: the F6 half is still short, and "unreachable" is not inferred.
- X7: Q12 peak > 0 on any disk of any geometry cell: S1 over-reports on this history.
- The clause "the clause may be wrong" in section ten: when an anchor from the clause under test itself (section seven A1 to A4) disagrees with the implementation, the quantities tied to that anchor stop and are handed back, and are not voided as a broken rig.

### Measurement basis

- Units: slot 16 KiB; one data unit is two slots, one copy on each disk; data unit payload 32634 bytes (32768 − header 134). `df` candidate values are always converted to bytes for reporting.
- T is probed on a copy of the state; the device is a copy-on-write overlay, and when the write of the first data unit arrives it reports a block device error and records pass; this shortcut is used only after it has been judged identically to the whole write item by item on the HF family, and if they differ the whole phase switches to whole writes (registration 5.1, revision 2).
- The session has no entry for an overwrite with an offset, so the "overwrite with offset" in F8's sufficient condition is replaced by HL (long mount with whole-file replacement writes); the holes inside a segment caused by "a local overwrite in a large file, with the other units left in place" are not covered (revision 3.4).
- In the phase-one histories, the sequences that raise F are all issued whole and released whole, so the pinned account H is always 0, and the pinned-slot half belongs to phase two (revision 3.5).

### Rerun

```bash
# 仓根。编与单测（checker 档，主 agent 带前缀跑）
# Repo root. Build and unit tests (checker tier, run by the main agent with the prefix)
cargo build --release -p singlefs-checker-tier --bin e180_admission_reading_versus_writable_bytes
cargo test --release -p singlefs-checker-tier --bin e180_admission_reading_versus_writable_bytes
# 第一段（长跑；分片、续跑与多机都在脚本里，片齐了合并出产物）
# Phase one (long run; sharding, resume and multi-machine are all in the script; merge into the artifact once the shards are complete)
bash scripts/experiment-apparatus/e180-reading-shards.sh
bash scripts/experiment-apparatus/e180-reading-shards.sh --list-items    # 逐行列片名与做没做完 / list shard names line by line and whether each is done
# 装置的子命令（经内存包装：bash scripts/hosts/run-with-memory-cap.sh <上限> target/release/e180_admission_reading_versus_writable_bytes <子命令…>）
# Rig subcommands (via the memory wrapper: bash scripts/hosts/run-with-memory-cap.sh <cap> target/release/e180_admission_reading_versus_writable_bytes <subcommand…>)
#   reading [--geometry <几何名>] [--family <族名>]                    一次跑全部或一格，产物打到标准输出 / run all or one cell at once, artifact printed to standard output
#   reading --geometry <几何名> --family <族名> --shard-output <文件>   只跑一片 / run one shard only
#   list-shards / merge <片目录> / calibrate                           列片、合片、量挂钟外推 / list shards, merge shards, measure wall-clock extrapolation
```

### What it cannot answer

- Phase one has no artifact, so this page has no reading for F2, F5 or F8; the phase-two rig is not written, so there is no rig for F6 or for the pinned-slot half of F5.
- F8 holds only for histories of whole-file replacement writes (revision 3.4); HC builds only one kind of abandoned root, "record durable, root slot unreadable", and whether it is built is judged by the abandoned root slots or quarantined slots the allocator records in mount C.
- Section seven A4 (529 nodes on two 4 GiB disks) checked only the closed form; the function on the implementation side is private and was not checked (revision 3.3).

### Affected decisions

| Decision item | Relationship | Look-back |
|---|---|---|
| D16 (Publish Semantics) settled item 1 | Preparation | 2026-10-21 unaffected: the basis paragraph of this item does not cite E180 (Acceptance Reading vs Truly Writable Amount), the phase-one artifact does not exist, it is only preparation, and the decision text is not changed this time |
| D28 (Mount-Time Commitments) settled item 1 | Preparation | 2026-10-21 unaffected: the basis paragraph of this item does not cite E180 (Acceptance Reading vs Truly Writable Amount), the readings of the nine-item formula only come out in phase one, the artifact does not exist, and the decision text is not changed this time |
| D28 (Mount-Time Commitments) settled item 3 | Preparation | 2026-10-21 unaffected: the basis paragraph of this item does not cite E180 (Acceptance Reading vs Truly Writable Amount), the sentence "`df` reports available" is measured by the W2 arm of F2, the artifact does not exist, and the decision text is not changed this time |
| D3 (Space Allocation) settled item 12 | Preparation | 2026-10-21 unaffected: the basis paragraph of this item does not cite E180 (Acceptance Reading vs Truly Writable Amount), the two gates in series serve only as the clause under test, the artifact does not exist, and the decision text is not changed this time |
| D3 (Space Allocation) settled item 8 | Preparation | 2026-10-21 unaffected: the basis paragraph of this item does not cite E180 (Acceptance Reading vs Truly Writable Amount), the empty slots closed inside a segment are measured by F8, the artifact does not exist, and the decision text is not changed this time |
| D3 (Space Allocation) settled item 9 | Preparation | 2026-10-21 unaffected: the basis paragraph of this item does not cite E180 (Acceptance Reading vs Truly Writable Amount), the definition of a spurious ENOSPC serves only as the way to judge an over-report, the artifact does not exist, and the decision text is not changed this time |

## Historical Versions

The history entries of E180 (Acceptance Reading vs Truly Writable Amount) are collected in [experiments-history.md](../experiments-history.md).
