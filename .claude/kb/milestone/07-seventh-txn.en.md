# Milestone "Packing Relocation Connecting to Parallel Commits": Work Items, Acceptance, and Ordering

[中文](07-seventh-txn.md) · **English** · [日本語](07-seventh-txn.ja.md)

**Measurement basis**: Planning document, filed ahead of time on 2026-10-12 based on user settlement (verbatim: "06 packing interface as a separate milestone numbered 07, others deferred accordingly."); this is a draft. **It does not require correctness**: revisit and revise before starting work on each item. The precondition is that all exits of Milestones 3 through 6 ([03-third-txn.md](03-third-txn.md), [04-fourth-txn.md](04-fourth-txn.md), [05-fifth-txn.md](05-fifth-txn.md), [06-sixth-txn.md](06-sixth-txn.md)) are achieved; if any item is missing when closing the previous milestones, this document is reordered based on what is missing.

**Reading guide**: Three boxes per item—"Existing in repo" lists which clauses, owed checks, and code this item is backed by in the repository (checked on 2026-10-12, with sources); "Work items" is a table: number, description, backing clauses and owed checks, acceptance (each proven to turn red first), dependencies; "Must be settled before starting work" lists unsettled questions that must go through three-party evaluation or consult the user before starting work. Anything marked "inferred" is an unmeasured judgment by the main agent. Work item numbering is formatted as "item·point" (I·1, II·1), used only within this document.

**No new transaction types added**: Packing relocation is a background compaction transaction implemented under serial commits in Milestone 5 ([05-fifth-txn.md](05-fifth-txn.md)); this document modifies only how it connects under parallel commits in Milestone 6, remaining a single transaction layer (`.claude/rules/fs-design.md` "one transaction layer shared by all structures").

## Exits (One Settled, Remainder Unsettled)

The user settled one exit on 2026-10-22:

| # | Exit | Belongs to which item |
|---|---|---|
| 1 | Crash full run: finish what is new, then catch up on the backlog, at least 10 billion states per milestone (settled by user on 2026-10-22). Run in two stages (recording stage on CPU, judging stage on GPU, all observers attached), without narrowing the enumeration domain, all 0 red. ① First run the crash streams new in this milestone and the blocks whose unit key changed; ② then catch up from the backlog registry (set up in Milestone 4), at least 5 billion states, in registry order, resuming where the previous run stopped. When new and backlog together exceed 10 billion states, this milestone runs at least 10 billion states and the part of the new work that does not fit goes to the end of the backlog registry; when together they come to less than 10 billion, run all of it. On red, pin it back into the quick tier per the "A red in a full run is pinned back into the quick tier" section of `.claude/rules/test-script-discipline.md`, fix it, then rerun. | Crash full run (cross-milestone) |

Exits are left to the user to decide in Stage 0. Candidate exits inferred by the main agent (inferred, unmeasured):

| # | Candidate exit | Belongs to which item |
|---|---|---|
| 1 | How packing relocation performs number acquisition under parallel commits and who waits for whom between it and foreground transactions are settled, written back to D27 (Small-Data Packing Container) and the parallel form settled in Milestone 6 | I |
| 2 | Crash streams where packing relocation is simultaneously in-flight with foreground overwrite, deletion, and GC achieve zero red across the board | II |
| 3 | Foreground fsync latency while packing relocation runs does not fall outside the threshold hardcoded in preregistration | II |
| 4 | GPU packing relocation (added to this document by the user on 2026-10-20): attach a GPU path following the multi-branch mechanism of GPU garbage collection in Milestone 4, item-for-item identical to the CPU path on the same inputs, with containers written by the two modes byte-for-byte identical; mount parameters are the single `background_*` set of D26 (Background Compaction and Placement Reclamation) settled item 13, and with GPU on, packing runs on GPU; task kind and compute backend are each a closed enum, so adding a task kind or a backend only adds one enum member | III |

## Grouping and Ordering

| Stage | Description | Rationale for ordering | Status |
|---|---|---|---|
| 0 Handover | Audit packing relocation landed in Milestone 5 and parallel form settled in Milestone 6; settle exits | Packing relocation connection form can only be known after parallel form is settled | Not started |
| 1 Design & implementation | I·1 → I·2 | | Not started |
| 2 Coverage & measurement | II·1 ∥ II·2 | Both depend on I·2 | Not started |
| GPU packing | III·1 measure → III·2 design → III·3 implement → III·4 coverage ∥ III·5 control group | III·1 waits only for the packing path landed in Milestone 5 and the measurement method of Milestone 4 VII·1; III·2 builds on the control interface settled in Milestone 4 VII·2; III·3 comes after I·2, because GPU batching spans several mapping leaves and needs to know how number acquisition works under parallel commits | Not started |

**Estimate**: Not estimated.

## I. Interfaces

- **Existing in repo**: Milestone 5 implements D27 (Small-Data Packing Container), where the form is background compaction moving objects ≤ 4 KiB into shared containers without changing a single byte on the write path, and containers are a new unit class (measurement basis of [05-fifth-txn.md](05-fifth-txn.md)). Section "Interfaces connecting Milestone 4" of Milestone 6 only asked how several background transactions—deletion batch, GC, snapshot deletion—perform number acquisition under parallel commits and whether they block foreground fsync; the question regarding packing relocation was transferred to this document.
- **Work items**:

| # | Description | Backing clauses and owed checks | Acceptance | Dependencies |
|---|---|---|---|---|
| I·1 | Design round (three-party): how packing relocation obtains transaction numbers; how to judge when source objects read by relocation are concurrently overwritten or deleted by foreground (candidates taken from the three approaches to deletion resumption in Milestone 6 I·9: re-evaluation at commit, read-set validation, global exclusive); whether it blocks foreground fsync | D27 (Small-Data Packing Container); settled item 2 of D13 (Verification Route); settlements of Milestone 6 I·1, I·4, I·9 | Verdict filed in `research/prompts/`, settlements written back to kb; decisions require experiments | Stage 0 |
| I·2 | Implementation: connect packing relocation into parallel commits according to settlement of I·1 | Settlement of I·1 | Each new test proven to turn red first | I·1 |

- **Must be settled before starting work**: Whether questions in I·1 cover only packing relocation, or unify number acquisition across deletion batch, GC, and snapshot deletion into a single unified answer.

## II. Coverage and Measurement

| # | Description | Backing clauses and owed checks | Acceptance | Dependencies |
|---|---|---|---|---|
| II·1 | Crash streams: packing relocation simultaneously in-flight with foreground overwrite and deletion, connected into crash amplification | Settled items 9 and 15 of D13 (Verification Route) | Zero red across the board; corrupting the number acquisition order causes the judge of that stream to turn red | I·2 |
| II·2 | Measure foreground fsync latency while packing relocation runs, using the apparatus from Milestone 6 Exit 1, with thresholds written into preregistration | `.claude/singlefs-ai-sop/rules/test-discipline.md` "experiments must specify which decision they measure for and what measurement is sufficient" | Commit artifacts into repo | I·2 |

## III. GPU Packing Relocation

- **User's words**: User on 2026-10-20: "In Milestone 4 we will do GPU GC. So discuss whether packing relocation in Milestone 7 can be implemented on the GPU. List this item in Milestone 7, and, following the multi-branch handling mechanism of Milestone 4, reserve other interfaces and fields that can be extended."
- **Existing in repo** (checked 2026-10-20): the GPU switch, card selection, GPU memory limit, error-without-fallback and the re-check that compares two GPU kernels, for background tasks, are settled in D26 (Background Compaction and Placement Reclamation) settled item 13, with one set of parameters governing all background tasks (user settled 2026-10-20, original wording in the change history); Milestone 4 VII·1 is measuring the wall clock of one batch of GC relocation (E185 (Wall-Clock Breakdown of a Batch of GC Migrations), all four rows of the question sheet `research/prompts/m4-gpugc-measure-forks.md` still open), and the control interface of VII·2 is not designed yet. The three hard constraints of D24 (Offloading Heavy Background Work to GPU) settled item 2 govern any work offloaded to the GPU; packing relocation is not among the three candidates of D24 (Offloading Heavy Background Work to GPU) and is governed by D26 (Background Compaction and Placement Reclamation) settled item 13. master has no packing-container code (`grep -rln 'PackingContainer\|packing_container'` over `crates/` has zero hits; `crates/singlefs-core/src/unit.rs` has only codes 1, 2, 3); product code has no GPU dependency (`wgpu` is only in `crates/singlefs-checker-tier/Cargo.toml`). D27 (Small-Data Packing Container) settled item 6 fixes one mapping leaf per transaction, with cost dominated by mapping writes; the 36 size classes of settled item 13 kept open at once total 1.15 MiB of containers; E21 (Net Benefit of GPU Offloading) measured that the transfer floor needs batches of ≥ 64 MiB, so GPU mode must batch across many mapping leaves.
- **Can it go on the GPU** (main agent's judgment, inferred, unmeasured): yes, in the same form as GPU garbage collection — the GPU only computes, mapping-leaf COW and transaction publish stay on the CPU, still one transaction layer (`.claude/rules/fs-design.md` "one transaction layer shared by all structures"). One batch of packing relocation splits into stages:

| Stage | What it does | GPU or not (inferred) |
|---|---|---|
| Source check | Read source objects, verify the checksum of the source unit | Yes: bandwidth-bound, same shape as GC verifying source units; E21 (Net Benefit of GPU Offloading) measured a 0.78× wall-clock net benefit for a pure scan, so it can only pay off when counted as saved host CPU time per D26 (Background Compaction and Placement Reclamation) settled item 13 rule 7 |
| Size-class assignment | Put each object into one of the 36 size classes, fix its slot number | Yes, but compute is small; offloading it alone saves little |
| Container assembly | Padding to slot width, assemble container bytes, compute the whole-container checksum (MAC has an execution path only with encryption on, D27 (Small-Data Packing Container) settled item 7) | Yes: assembled bytes must come back to be written to disk, so it takes the round-trip transfer floor |
| Mapping update and publish | COW mapping leaves in key order, go through transaction publish | No: tree operations and publish are the transaction layer; backend does not change within a batch |
| Re-encryption | With encryption on, moving an object recomputes ciphertext under the new AAD (C240 (uncomputed cost of re-encryption during relocation)) | Reserved: encryption is not in the first runnable version (D9 (Encryption) settled item 10); this stage only keeps a task kind in the interface |

- **Following the multi-branch mechanism of Milestone 4, what room is reserved**:

| Mechanism in Milestone 4 VII·3 | How packing relocation attaches | Extension room reserved |
|---|---|---|
| Compute backend `background_compute` (`cpu` / `gpu`, default `cpu`) | Shares this one parameter: with `gpu` on, packing and GC go to GPU together, no separate packing parameter | Backend is a closed enum with exhaustive `match`, no wildcard arm; adding a backend only adds one member, and the compiler forces every site to be filled in |
| Card selection `background_gpu_devices`, GPU memory limit `background_gpu_memory_limit` | Shared: a card's limit caps all background tasks combined; how packing and GC split it within the limit is settled in III·2 | Parameters exist as one set, not per task; task kind is a closed enum (GC relocation, packing relocation; later the scrub, erasure-code rebuild and bulk compression of D24 (Offloading Heavy Background Work to GPU), and re-encryption once encryption is on); which kinds have a GPU path is decided by the exhaustive `match` over the enum, and adding a task kind only adds one member |
| Switch while mounted, effective from the next batch, never within a batch | Same: one switch flips all background tasks, each taking effect at its own next batch | Control interface has a single switch, not addressed by task kind |
| Error when the GPU is unusable, no fallback | Same: the batch being computed is abandoned, the packing sweep stops and reports | None |
| Two independent GPU kernels each compute once and are compared (one card is enough), confirmed results go to the CPU to update the bitmap, entries that stay uncertain go to the CPU to judge; GPU and CPU are two implementations | Same (D26 (Background Compaction and Placement Reclamation) settled item 13 rule 8, D24 (Offloading Heavy Background Work to GPU) settled item 2 rule 3) | One CPU reference and GPU path pair per task kind; differential tests per kind each go into the quick tier |
| Branch can be forced by tests, runtime shows which branch ran | Same (`.claude/rules/fs-design.md` five hard requirements) | Observation counters recorded per "task kind × backend" |

  Room is reserved in three places only: the task-kind and backend enums, control interface, in-memory observation counters; mount parameters are only the one set of D26 (Background Compaction and Placement Reclamation) settled item 13, none added for packing. Not one bit is added on disk: neither the packing intent nor the container header records which backend ran (D24 (Offloading Heavy Background Work to GPU) settled item 2 rule 1; D26 (Background Compaction and Placement Reclamation) settled item 13 rule 1). No placeholder code: the extension room lives in the shape of the enums and parameters, and code is written only for two task kinds (GC and packing) and two backends (`cpu` and `gpu`) (`.claude/singlefs-ai-sop/rules/code-discipline.md` "delete code that is not used"; same practice as the Milestone 4 section "Optimization items: Not in exit criteria, entry conditions registered").

- **Work items**:

| # | Description | Backing clauses and owed checks | Acceptance (proven red first) | Dependencies |
|---|---|---|---|---|
| III·1 | Measure: on the packing path landed in Milestone 5, measure the wall clock of one batch of packing relocation, split by the four stages source check, size-class assignment, container assembly, mapping update and publish into read/write I/O and CPU shares; reuse the apparatus and measurement basis of E185 (Wall-Clock Breakdown of a Batch of GC Migrations) with a rerun registration, no new measurement method | D27 (Small-Data Packing Container) settled item 6; judgment form of D24 (Offloading Heavy Background Work to GPU) settled item 3; C202 (idle overhead and hysteresis K of packing patrol lack values) | Assertions pin absolute values; doubling the mapping leaves in a batch raises the wall clock accordingly (discriminative power self-test) | Milestone 5 packing path on master; Milestone 4 VII·1 |
| III·2 | Design round (three-party): which stages go on the GPU; how batching (how many mapping leaves per submission) relates to the GPU memory limit; when packing and GC both run on the GPU, how they split a card's limit and who waits for whom; how the task-kind enum plugs into the control interface settled in Milestone 4 VII·2; GPU kernels live in the same optional package as GC's | D26 (Background Compaction and Placement Reclamation) settled item 13; D24 (Offloading Heavy Background Work to GPU) settled item 2; D27 (Small-Data Packing Container) settled items 6, 7, 10, 11; settlement of Milestone 4 VII·2; settlement of I·1 | Verdict filed in `research/prompts/`, settlements written back to kb; decisions require experiments | III·1, I·1, Milestone 4 VII·2 |
| III·3 | Implementation: backend choice, card selection, GPU memory limit and switching while mounted (effective from the next batch) for packing relocation; GPU path with CPU reference; two independent GPU kernels each compute once and are compared, uncertain entries wait for the next round, and entries uncertain for N consecutive rounds go to the CPU to judge (D26 (Background Compaction and Placement Reclamation) settled item 13 rule 8); error without fallback; branches can be forced by tests and the runtime shows which branch ran | Settlement of III·2; `.claude/rules/fs-design.md` five hard requirements | The two paths are item-for-item identical on the same inputs, differential test in the quick tier; injecting a known fault on the GPU side (one object put in the wrong size class) turns the differential test red; misconfigurations each refuse to mount (as in Milestone 4 VII·3); peak GPU memory taken by packing and GC together on each card does not exceed the value `background_gpu_memory_limit` gives that card, with an absolute-value assertion, and injecting a fault that allocates one extra buffer must turn red; injecting a GPU error at runtime abandons that batch, packing stops and reports; model differential testing reads identical content in gpu and cpu mode | III·2, I·2 |
| III·4 | Coverage: packing relocation stream in GPU mode, one stream switching `cpu` ↔ `gpu` between batches, one with GPU packing and GPU GC both on, connected into crash amplification; containers written by the two modes are byte-for-byte identical | D13 (Verification Route) settled items 4, 9, 15 | Zero red across the board; must-red world (GPU side changes one slot number, container byte comparison turns red) | III·3, II·1 |
| III·5 | Control group: same workload, same pool, packing run with `cpu` and with `gpu`, reporting packing wall clock, host CPU time taken by packing (GPU mode includes preparation, submission, waiting for results, comparing the two results and the entries handed to the CPU to judge) and foreground fsync latency; run once per several small-object counts | D26 (Background Compaction and Placement Reclamation) settled item 13 rule 7 | Measurement basis fixed in preregistration; absolute-value assertions; positive control: the GPU side deliberately re-checks once more, and the saved CPU time must drop; artifacts committed | III·3 |

- **Must be settled before starting work**:
  1. Shared or separate parameters: the user settled on 2026-10-20 that they are shared; one `background_*` set governs all background tasks, and turning GPU on turns on every GPU feature (D26 (Background Compaction and Placement Reclamation) settled item 13 rules 1 to 3). Settled.
  2. The worth criterion: the user settled on 2026-10-20 to count saved host CPU time, running faster on large data being only a by-product (D26 (Background Compaction and Placement Reclamation) settled item 13 rule 7). Settled.
  3. Milestone 4 VII·2 must use this one parameter set and make background task kind an enum when designing the control interface, so that III·2 need not go back and change it; that requirement is written in "To be settled before starting" of [04-fourth-txn.md](04-fourth-txn.md) "VII GPU Garbage Collection".

**Not in exits**: Form of packing itself (Milestone 5), parallel commits itself (Milestone 6), write buffer and node cache (Milestone 8 [08-eighth-txn.md](08-eighth-txn.md)).
