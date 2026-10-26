## D17 (Implementation Layering & Third-Party Pipeline) —— Settled

[中文](17-实现分层与第三方管道.md) · **English** · [日本語](17-実装階層とサードパーティパイプライン.ja.md)

D17 (Implementation Layering & Third-Party Pipeline) governs the line between this project and third-party pipelines: where the waistline is drawn, which side belongs exclusively to this project, which byte slice defines the "data interface", who dictates the failure model axis, how structural equivalence classes are counted, as well as version 1's execution carrier and debts owed by a pipeline. Out of scope: how many media lines are opened, and whoever opens a line maintains it resides in D12 (Target Media); crash state sets and `CommitStep` enumeration are in D13 (Verification Route); how unit atomicity is synthesized is in D20 (Load-Bearing Surface: Unit Atomicity & Self-Containment) and D22 (Synthesizing Unit Atomicity); format freeze layering is in D15 (Format Freeze Policy).

### Settled Items

**D17 (Implementation Layering & Third-Party Pipeline) items follow a single numbering sequence**, continuous with the "Unsettled Items" section, without renumbering.

| # | Item | Settlement |
|---|---|---|
| 1 | **How the failure model axis is written into contract** | Core layer prescribes a single failure model (512-byte sector atomicity, writes may tear to sector granularity, reordering permitted between barriers, reads may be misplaced); pipelines cannot declare exceptions, unachieved capabilities synthesized by core layer **Status: Settled.** **Progress: Unimplemented.** |
| 2 | **Structural equivalence class count requires recalculation** | Grouped by segment sequences of recorded streams; equivalence class count = opened structural equivalence classes × failure models, current value 1; solely registered in [Byte Table](../layout/01-first-txn.md) VIII **Status: Settled.** **Progress: Implemented.** |
| 3 | **Publication format of conformance suite** | Ship source code + judging validator program, omitting containers or standalone image corpuses **Status: Settled.** **Progress: Unimplemented.** |
| 4 | **Executable definition of "Entry Point"** | Entry point = RefFS operational surface, persistence commitment of fsync defined by crash semantics of RefFS, without instantiating separate O3 (independent specification executor); RefFS built anew per kb, accepting whitelisted inputs only **Status: Settled.** **Progress: Implemented.** |
| 5 | **Execution carrier for version 1** | Pure user-space library + block device abstraction (files and `O_DIRECT` raw devices as two backends); no mount in version 1, kernel module deferred for initial years **Status: Settled.** **Progress: Implemented.** |
| 6 | **Where the waistline is drawn** | Below the commit protocol: one pipeline per medium; abstract state machines of transactions, commit step sets, and crash semantics remain above waistline, owned exclusively by this project **Status: Settled.** **Progress: Implemented.** |
| 7 | **Precise meaning of "Data Interface"** | The minimal byte set required to safely reject; machine-checkable form is a bare header parser linking zero pipeline code **Status: Settled.** **Progress: Unimplemented.** |
| 8 | **Five debts owed by a third-party pipeline** | Five items: suite published externally, compliance judged using this project's oracle, system configuration records writer identity, degraded mode of unknown `compat_ro` backed by crash testing, checker forbidden from linking pipeline code **Status: Settled.** **Progress: No code object.** |

#### Settled Item 1: Single Failure Model, Core Layer Synthesis

**Settlement**: Core layer prescribes a single failure model—512-byte sector atomicity, writes may tear to sector granularity, reordering permitted between barriers (empirical measurement basis of D20 (Load-Bearing Surface: Unit Atomicity & Self-Containment) Corollary 3: "device-side atomic width is 512 and cannot be relied upon"), reads may be misplaced (reading back self-verifying content from somewhere else: for a root read from a root ring slot whose txg, per the placement formula, should not live in that slot, every place that reads the root ring treats it as failing self-verification; generation-0 roots are recognized separately by the slot mkfs seeds them in, the same exception as I-7.17 (root slot position equals the position computed from the txg)); pipelines cannot declare exceptions; capabilities unachieved are synthesized by core layer (read-back verification / double-write / logging compensation), synthesis costs borne by core layer, charged per pipeline rather than per user. Isomorphic to `REQ_PREFLUSH` / `REQ_FUA`: "backends forbidden from rejecting, block layer synthesizes internally".

**Scope**: Cardinality of this axis is strictly 1, not multiplying into equivalence class count (settled item 2). D17 (Implementation Layering & Third-Party Pipeline) settled item 1 settles contract format, committing no bytes before first third-party pipeline appears; prerequisites for C16 (pipeline declares failure model exception) are satisfied. Existing coverage of "failure atomicity width / tearing / reordering" lives in D13 (Verification Route) settled item 4 (crash state set) and D20 (Load-Bearing Surface: Unit Atomicity & Self-Containment) / D22 (Synthesizing Unit Atomicity); C6 (Wrong Block Layer Semantic Assumptions Written) asks "whether this project's assumptions are correct", whereas this asks "what a pipeline's failure model is", falling under different cells. Rejected Option B (allowing pipelines to declare stronger models, dispatching replay per declaration) introduces an extra axis outside `CommitStep`, whose cost requires heavy-tier replay dispatched per declaration.

**Basis**:

- E129 (How Writes Below io_min Fail on Real Devices): torn writes at deterministic offsets can be induced on real devices, with positive and negative controls clean—"writes may tear to sector granularity" is a real behavior to guard against, not a deduction.
- Production control group: `REQ_HARDBARRIER`, which permitted backends to return `-EOPNOTSUPP`, was removed in Linux 2.6.37 and replaced by `REQ_PREFLUSH` / `REQ_FUA` synthesized by block layer without permitting backend rejections, persisting unchanged for 15 years; sole difference is whether backend is allowed to say "unsupported".
- User settlement (2026-09-13 selected Option A), verbatim words in changelog.
- Reads may be misplaced: root ring three-party verdicts over three rounds, `research/prompts/m3-close-rootring-r1-main-verification.md` section 2 X1 (bare formula hit by generation-0 roots, exception added), `research/prompts/m3-close-rootring-r2-main-verification.md` section 2 Y1 (no hit after the exception was added), and the `research/prompts/m3-close-rootring-r3-main-verification.md` section 2 cell C743 (rebuilt allocator recording new class does not verify root occupies slot) (upheld); user settlement 2026-10-20 "cover every place that reads the root ring", verbatim words in changelog. No experiment: numbers measured on the three-party legs' replicas, no experiment number assigned.

**Owed**: C16 (pipeline declares failure model exception).

#### Settled Item 2: Structural Equivalence Classes Grouped by Segment Sequences, Current Value 1

**Settlement**:

1. Structural equivalence classes are grouped by segment sequences of recorded streams: two layout lines are equivalent <=> for every root slot write path (publish, empty publish, mkfs seed, instance switch / administrator rollback—three writer roles listed in D13 (Verification Route) settled item 1 plus empty publish), the segment sequences of recorded streams across both lines are isomorphic: segment boundaries (segments cut by barrier and FUA writes, D13 (Verification Route) settled item 4) share identical locations, and the **set** of step kinds appearing in each segment is identical (seven kinds: whole-segment zeroing / discard / unit write / journal record write / root slot FUA write / system configuration slot in-place overwrite / barrier; canonical order follows this declaration order, matching `StepKind` in `crates/singlefs-harness/src/segments.rs` item by item); repetition counts per step (device count, unit count, replica count) are parameters not entering criteria (D13 (Verification Route) settled item 10). Comparison occurs exclusively between identical path roles across lines; paths within a single line do not compare with one another or count. Kinds are classified by "which I/O action is executed", not "which region on disk this write lands upon": grouping by placement would conflate "whole-segment zeroing of journal ring" and "sequential record writes to journal ring" into the same class, failing to distinguish these two commit protocols when equivalence classes test the set of step kinds per segment.
2. **Equivalence class count = opened structural equivalence classes × failure models, current value 1 × 1 = 1, recalculated whenever a line opens**: failure model count is strictly 1 (settled item 1, synthesis adds ordering points rather than step kinds); opened lines contain pure SSD only (D12 (Target Media) settled item 5). It multiplies against debts 2 and 3 of D12 (Target Media) (running crash-point replay per class, gate wall-clock scaling by equivalence class factor).
3. Sole registry location: [layout/01-first-txn.md](../layout/01-first-txn.md) VIII; segment sequences in the table derive verbatim from `name=segments` lines of E142 (New-Pool File Creation Dry Run) artifacts, **without duplicating strings here** (duplicating creates a second registry location, whereas segment-registry cell of gate doc-registries compares solely layout/01-first-txn VIII against artifacts, unable to validate duplicate copies); instance switch / administrator rollback (including subsequent warm-up) and empty publish raising F are marked projected before writing bytes, rechecked identically upon serialization.
4. Whenever opening a line: run each root slot write path on that line, register its `name=segments` line into that line's byte table, compare path-by-path against existing classes, incrementing by 1 if non-isomorphic. Zoned lines are counted by this rule, not by Option B of D22 (Synthesizing Unit Atomicity) unsettled item 6.
5. Gate = registry table + `name=segments` lines in artifacts (pinned by `replay.sh` exact) + E142 (New-Pool File Creation Dry Run) unit test `registered_segment_sequences_match_every_recorded_path` (turns red if barrier or FUA positions altered in any path, including mkfs).

**Scope**: mkfs's own crash states have only one stream in crash-point replay: redoing mkfs over a used disk, enumerating the 27 steps mkfs records (`crash-case:make-filesystem-over-a-used-pool-stream` in `.claude/gate.d/stage-inputs.tsv`); mkfs crash states on other initial on-disk states and device semantics are outside the enumeration (E142 (New-Pool File Creation Dry Run) G19), and on that side only its segment sequence is pinned; "current value 1" is an identity on a singleton set, giving 1 under any criterion; clause weight resides in items 1, 4, and 5.

**Basis**:

- E142 (New-Pool File Creation Dry Run): four `name=segments` lines write segment boundaries and step kinds of each root slot write path into bytes, turning unit tests red if barrier or FUA locations are altered—verdicts are enforceable, registry has unique source.
- Three-way argument: Round 1 `research/prompts/d17-r1-main-verification.md` (all three legs unanimously judged "grouping by `CommitStep` member set" too coarse); Round 2 `research/prompts/d17-r2-main-verification.md` (none counted a second line under candidate criteria; phrasing and gate gaps incorporated into the five settled items).
- Main agent settlement (2026-09-13, per two rounds of three-way argument), verbatim user orientation words in changelog.

**Owed**: C8 (Gate Scope Cannot Be Determined): gate wall-clock multiplies by this number, with nothing checking its expansion.

#### Settled Item 3: Conformance Suite Ships Source Code + Validator Program

**Settlement**: Ship two items: source code of the suite, and a validator program. Omit containers, and do not ship image corpuses alone. Third-party pipelines prove compliance by executing the **identical validator program**, whose judging logic is **readable source code**, not a black box—isomorphic to D13 (Verification Route) settled item 7: "O2 (independent parser + checker) is a unary predicate judging single images"; verdicts must be independently verifiable by third parties, otherwise "compliance" degrades into "compliance because we said so".

**Scope**: Does not answer "whether validator program and in-repo checker share identical code or two separate copies"—if identical, `.claude/rules/fs-design.md` rule "auditor and auditee must not use identical code" must be adjudicated here: third party is auditee, this project is auditor, differing from checker/runtime relationship and preventing uncritical borrowing; placement belongs to settled item 2 and C8 (Gate Scope Cannot Be Determined). Omitting image corpuses from releases is a deliberate choice: corpuses expire with format evolution, whereas source + validator evolves with versions; whether to release a frozen-point corpus follows D15 (Format Freeze Policy).

**Basis**:

- No experiments: what and how many deliverables to release defines external commitments, lacking measurable quantities; benefits (verdicts verifiable by third parties) become measurable only when first third-party pipeline appears.
- User settlement (2026-08-31), verbatim words in changelog.

**Owed**: C8 (Gate Scope Cannot Be Determined).

#### Settled Item 4: Entry Point = RefFS Operational Surface

**Settlement**: Executable definition of "entry point" is provided by RefFS—per D13 (Verification Route) settled item 3 "pushed to the extreme, handling persistence and crash semantics natively", RefFS defines persistence commitment of fsync; RefFS is an independent semantic implementation (written solely per kb text, not extended from O1 (in-memory model differential / HashMap ideal file system), sharing zero crash point enumeration code with implementation); no separate O3 (independent specification executor) is instantiated. RefFS accepts whitelisted inputs exclusively from callers: outcome of each operation (success, rejection prior to writes, error after writes, idempotent reissue), which operation a crash lands within and its attribution rules, rollback target confirmed publish index, contents read upon each recovery, explicit publish points.

**Scope**: The cost is a semantic-level RefFS (D13 (Verification Route) settled item 3: lacks on-disk format; user on 2026-10-02 updated interpretation, no longer categorized as "effort approximating writing another file system"). Rejected Option B (instantiating separate O3 (independent specification executor) defining only fsync commitment, delegating remainder to O1 (in-memory model differential / HashMap ideal file system)) requires reopening D13 (Verification Route) settled item 3.

**Basis**:

- No experiments: this cell defines boundaries between two oracles on the verification side without modifying on-disk bytes, lacking measurable quantities.
- User settlement (2026-09-13), 2026-09-08 confirmation "interpreting per settled item 3 constitutes model expansion" codified into clause; verbatim words in changelog.
- User settlement (2026-10-02): RefFS built to semantic level, built anew per kb, isolated into standalone crate, accepting rollback target as input; verbatim words in changelog; "extended from O1 (in-memory model differential / HashMap ideal file system)" rewritten accordingly.
- Three-way arbitration: `research/prompts/m3-formal-r4-main-verification.md` Section 2 independence cell (expanding RefFS directly from O1 violates D13 (Verification Route) settled item 3 regarding non-shared enumeration and semantic level); `research/prompts/m3-formal-r5-main-verification.md` Section 2 standalone crate cell (sentence rewritten combining defense wording and attacker's input whitelist); `research/prompts/m3-formal-r6-main-verification.md` Section 2 whitelist cell (whitelist consolidated into five items).

**Owed**: C447 (Specification Executor Not Connected to Crash-Point Replay and RefFS Differential).

#### Settled Item 5: Version 1 Is Pure User-Space Library, No Mount

**Settlement**: Version 1 is a pure user-space Rust library programmed exclusively against block device abstractions, with two backends: regular files and `O_DIRECT` raw devices. It exposes an operational API (mkfs, open, write extent, publish / fsync, recovery); crash-point replay harness, RefFS differential, and QEMU tier workload binaries all directly drive these operations. Version 1 has no mount: FUSE is a thin adapter added later after directories and POSIX surface emerge; kernel modules will not be built within foreseeable years.

All three verification routes function without mounting: crash-point replay requires intercepting block device primitives (writes, barriers, FUA writes), with recording defined in D13 (Verification Route) settled item 4 positioned at that layer; RefFS differential feeds identical operation sequences to both sides comparing outcomes without touching disk; QEMU tier executes static binaries consuming `/dev/vda`: experiment binaries are musl static, while checker tier real-device binaries with RocksDB compile as static-pie under `x86_64-unknown-linux-gnu` with `+crt-static` ([vm-harness.md](../vm-harness.md)).

**Scope**: Three known corners. ① Recording hooks attach to block device interfaces, not operation APIs: a single "publish / fsync" issues multiple unit writes, two barriers, and one FUA write internally; attaching hooks at API layer swallows intermediate crash points entirely, whereas enumeration domain of D13 (Verification Route) settled item 4 targets those primitives => implementation must report device-level operation counts per API invocation for harness validation, invalidating runs on mismatch. ② Library must reside under `crates/`: gate "Show me test" recognizes implementation code strictly within `crates/*/src/`. ③ Modifies no on-disk bytes, but carrier is not zero-cost—kernel `block/bdev.c` `system_configuration_set_blocksize` requires file systems with blocks larger than `PAGE_SIZE` to carry `FS_LBS` flag with kernel `CONFIG_TRANSPARENT_HUGEPAGE` enabled; singlefs units (32,768) and nodes (16,384) exceed host 4,096, requiring kernel modules to depend on that feature or implement internal sub-page buffering; library and FUSE paths bypass this constraint. Deferring kernel modules is not grounded in D7 (Merging into Linux Mainline)—that clause states only non-submission to mainline, not forbidding out-of-tree modules—the true basis is `.claude/rules/fs-design.md` "build the slow, correct path first", combined with large-block `system_configuration_set_blocksize` kernel constraints.

**Basis**:

- E142 (New-Pool File Creation Dry Run): all fixed structure bytes of new-pool file creation written by a binary programming solely against block device abstractions, matching `crates/` implementation region by region—"correct transaction commit" achievable without mounting.
- Three-way confrontation (2026-09-03): forward leg derived identical carrier from settled clauses highlighting recording hooks and `crates/`; backward leg found no counterexamples, examining host kernel 6.17 source to surface `system_configuration_set_blocksize` constraints; user settlement (2026-09-03), verbatim words in changelog.

**Owed**: C8 (Gate Scope Cannot Be Determined).

#### Settled Item 6: Waistline Drawn Below Commit Protocol

**Settlement**: Waistline is drawn below commit protocol. Each medium maintains an independent pipeline (allocation, placement, layout, index tuning, node size, caching, device handling implemented separately, allowing third-party implementations); however, **abstract state machine of transactions, commit step sets, and crash semantics remain above waistline, owned exclusively by this project**.

**Protocol belongs to this project**: when a pipeline introduces new structural commit steps, it must submit a diff against this repository's shared enum (`CommitStep` in D13 (Verification Route) settled item 10), rather than inventing them within private black boxes.

**Scope**: Waistline constrains structure, not "disk-writing code with identical bytes"—independent implementation pipelines per medium are permitted, but secondary publish paths bypassing state machines are forbidden (`.claude/rules/fs-design.md` "one transaction layer shared across all structures"). Emulating existing templates does not ensure self-enforcing boundaries: even disciplined designs like `iomap` require `validity_cookie` in `iomap.h` because shared layers cannot discern when backend transactions invalidate mappings, forcing file systems to embed sequence numbers. Bytes above waistline belong to freeze Layer 1, held more strictly than subsequent layers (settled item 7).

**Basis**:

- No experiments: waistline placement defines division of labor, lacking measurable quantities; observable costs (pipeline and equivalence class counts) are counted in settled item 2, with anti-leakage efficacy measurable only when first third-party pipeline appears.
- Three-way argument (2026-08-27, unanimous across three legs): backward leg examined kernel 6.17 tree and OpenZFS master, identifying zero cases simultaneously satisfying "third-party backend + crash consistency semantics in contract + non-circumvented contract" (unfound, not non-existent, usable as negative conclusion); sole production COW file system with pluggable backends is ZFS, whose backend signatures embed transaction group numbers (`vdev_asize_func_t` in `include/sys/vdev_impl.h`), backends maintain three per-txg dirty lists, and four methods in `vdev_ops` cross the "data-movement only" boundary; forward leg independently derived identical waistline—shallower placements fail to accommodate zoned structural facts ("root pointers cannot overwrite in-place"), while deeper placements either abandon D12 (Target Media) guarantees ("refuse mount rather than silent misreading") or collapse back to this waistline under analysis.
- Production demonstration (inspected kernel 6.17): btrfs zoned support is not isolated in `zoned.c`; `btrfs_is_zoned` scatters across 22 files at 93 locations, including 5 in `tree-log.c` (transaction logging) and 6 in `space-info.c` (ENOSPC acceptance)—unbounded media branches inevitably leak into transaction layers and ENOSPC acceptance.

**Owed**: C19 (Media Verdict Leaking into Transaction Layer).

#### Settled Item 7: "Data Interface" = Minimal Byte Set Required to Safely Reject

**Settlement**: "Data interface" is not "identical bytes"—D12 (Target Media) settles distinct layouts per medium, where on-disk bytes differ by design. Data interface = **minimal byte set required to safely reject**: the format portion enabling an unfamiliar reader to reach a definitive verdict—mount / read-only mount / reject, one of three, **never a fourth state of "reading seemingly valid bytes under mistaken assumptions"**.

Machine-checkable form: **a bare header parser linking zero pipeline code**, running against arbitrary byte streams claiming to be singlefs, asserting I-1.1 (block header states its logical address) and I-1.4 (block header fsid consistent) extractable and consistent + feature bit encoding + position and width of checksum / MAC fields in pointers.

**Scope**: I-1.2 (block header write order published) and I-1.3 (block header tree ID consistent) fall outside bare header parser scope—I-1.3 requires "matching the tree actively referencing it", necessitating finding parent nodes; I-1.2 requires evaluation via global published predicate settled in P2 of C113 (No Input to Current-Version Determination of Multi-Version Units in Scan Rebuild), taking instance table row (i, T_pub, W) and recovery root txg as inputs; both require secondary inputs, failing as unary header predicates running against arbitrary byte streams => assigned to O2 (independent parser + checker), excluded from data interface.

**Basis**:

- No experiments: this cell defines "which bytes constitute contract", evaluated on failure modes (rejection over silent misreading) rather than numerical ordering; verdict rate against arbitrary byte streams measurable only after bare header parser is built.
- `.claude/rules/fs-design.md` "convert 'used wrongly' into 'fails to mount'" serves as criterion origin; three-way argument (2026-08-27) and 2026-09-14 refinement (moving I-1.2 and I-1.3 out of scope) recorded in changelog.

**Owed**: None. C446 (unimplemented invariants have neither prior registration nor a due date) is paid off; it is judged by the invariant-prerequisites cell of gate doc-registries.

#### Settled Item 8: Five Debts Owed by a Third-Party Pipeline

**Settlement**:

| # | What Is Owed | Why Unavoidable |
|---|---|---|
| 1 | **Three oracles transition from internal tests to externally published conformance suite** | "Whether users use it is up to users" holds only when **users have tooling they can run independently**. Represents explicit scope expansion |
| 2 | **Compliance judged by this project's oracle, not pipeline's proprietary validation code** | `.claude/singlefs-ai-sop/rules/show-me-test.md` "no preferential treatment by origin" applies verbatim: criteria require evidence, not self-certification reports |
| 3 | **System configuration records writer identity day-1** | Blast radius must be attributable. When receiving corrupted images, checker must state "written by pipeline X v2.3", rather than merely "corrupt". Impossible to retrofit |
| 4 | **Degraded mode of unknown `compat_ro` bits requires crash testing safety net** | [pitfalls.md](../pitfalls.md) item 9: btrfs degraded mounts exacerbate corruption, rooted in unengineered degraded semantics |
| 5 | **Checker forbidden from linking pipeline runtime accounting / verification code** | "Auditor and auditee must not use identical code". More vital for third parties than internal components—zero trust exists here |

**Scope**: Debt 3 clauses written, checks owed: bootstrap header of system configuration contains "writer identity 20" = 16-byte ASCII implementation ID zero-padding + 4-byte u32 version, version 1 writing `singlefs-rs` / 1 (D22 (Synthesizing Unit Atomicity) settled item 9), established as invariant I-1.5 (system configuration records pipeline identity), unimplemented in checker. Publication format of Debt 1 settled in item 3; whether validator program and in-repo checker share code remains open. All five debts commit zero bytes before first third-party pipeline appears.

**Basis**:

- No experiments: five debts define "what a pipeline must satisfy upon arrival", lacking measurable quantities when zero third-party pipelines exist; Debt 3 established as on-disk field and invariant, verified by checker rather than experiments.
- [pitfalls.md](../pitfalls.md) item 9 (btrfs degraded mounts) provides origin for Debt 4; Debts 2 and 5 derive from `.claude/singlefs-ai-sop/rules/show-me-test.md` and `.claude/rules/fs-design.md` disciplines; Debt 3 field settled by user (2026-09-14), verbatim words in changelog.

**Owed**: C445 (sampling points of model differential do not grow with write path); C447 (Specification Executor Not Connected to Crash-Point Replay and RefFS Differential); C8 (Gate Scope Cannot Be Determined). Three successor debts (C445 (sampling points of model differential do not grow with write path), C446 (unimplemented invariants have neither prior registration nor a due date), C447 (Specification Executor Not Connected to Crash-Point Replay and RefFS Differential); of these, C446 (unimplemented invariants have neither prior registration nor a due date) is paid off, judged by the invariant-prerequisites cell of gate doc-registries) cover respective verification route gaps, while Debt 1 ("oracle becomes externally published conformance suite") is broader—**that cell lacks an assigned tracking number**.

### Interactions with Other Decisions

| Decision | Interaction Point |
|---|---|
| D12 (Target Media) Target Media | Settled item 6 advances D12 (Target Media) from "multiple format sets" to "multiple implementation sets". D12 (Target Media) settles "open branches per medium, whoever opens maintains", rendering pipeline count an open N—settled item 3 elevates from "unsettled" to "mandatory". Equivalence class count provided by settled item 2: current value 1 (pure SSD opened, failure model count strictly 1), recalculated per segment sequence registry in [layout/01-first-txn.md](../layout/01-first-txn.md) VIII upon opening lines |
| D13 (Verification Route) Verification Route | `CommitStep` closed enum initially capped crash-point replay multipliers; settled item 6 adds **second role: third-party extension point**. Grep gate triggering heavy-tier replay upon modification reused verbatim |
| D15 (Format Freeze Policy) Format Freeze | That small byte slice in settled item 7 belongs to freeze Layer 1, held more strictly than subsequent layers—serving as common contract across all pipelines |
| [pitfalls.md](../pitfalls.md) item 4 | "fsync following specialized log tree = second transaction mechanism". Settled item 6 **forbids** its reintroduction: pipelines vary, but transaction state machine remains singular |

## Historical Versions

Historical entries for D17 (Implementation Layering & Third-Party Pipeline) are consolidated in [decisions-history.md](../decisions-history.md).
