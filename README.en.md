# singlefs

[中文](README.md) · **English** · [日本語](README.ja.md)

---
The big disks belong to the company; the good sleep belongs to you.
---
**A copy-on-write (COW) file system designed from scratch, written in Rust.**

## Why build another one

I have read the code of many file systems, and I think they are all excellent. But the hard years they grew up in left them too frugal and too tightly coupled, so now that times are rich they cannot enjoy it.

So I decided we will enjoy it. singlefs is allowed to be ugly, allowed to waste storage, and allowed to carry many bloated implementation branches side by side.

I hope that many years from now, when someone describes this code, they will not say it is one big pile of crap, but that it is many piles of crap.

In practice, changing one place may mean changing many branches together. That is fine: we can easily add another, different branch, even if it stinks in its own special way.

Well, I am still verifying all this, but I reckon it should work.
waaagh!

Design problems that led singlefs to start with its format and architecture, and the directions chosen: these are design goals; implementation boundaries are listed under "Current status".

| Pain point | Root cause | Choice made here |
|---|---|---|
| Absurd ENOSPC (space is free but writes fail; deleting a file also reports no space) | Two-level allocation plus rigid chunk classes | No data/metadata split; one global allocator |
| RAID5/6 write hole | Fixed-width stripes need read-modify-write | Variable stripe width, never read-modify-write |
| Quota accounting that is slow and wrong | Accounting is an after-the-fact traversal that also walks the reverse index | Accounting is a by-product of the transaction, maintained incrementally at commit |
| The reverse index became a complexity black hole | It grew later; it was not designed | Designed into the format from day one |
| Nobody dares to run the repair tool | Rebuildability was never a design goal | Self-describing metadata blocks, plus a checker that evolves with the format |

Decisions and their grounds are in [`.claude/kb/decisions.md`](.claude/kb/decisions.md); the pitfall list is in [`.claude/kb/pitfalls.md`](.claude/kb/pitfalls.md).

### Capacity and overhead data

Existing experiments include measurements and arithmetic models; each value has its own measurement basis: device, workload and format assumptions. They are not throughput, latency or general performance promises for the current complete file system; the comparison table records the full results and limits of historical versions.

- Capacity ceiling: in the 16 GiB-per-disk configuration of E152 (Per-Milestone File Performance Against Six File Systems), the unit area occupies 95.2% of one disk; the mirrored pool ceiling is 47.6% of the two disks combined. Trees and accounting consume additional space, so actual usable capacity is lower; this does not imply single-disk mkfs support. Comparison: [`scripts/experiment-apparatus/perf-by-milestone.md`](scripts/experiment-apparatus/perf-by-milestone.md).
- Small files: each data unit occupies 32 KiB on disk; after the 105-byte header and 29 reserved bytes, its current payload limit is 32634 bytes. Unused space is zero-padded and every unit takes its own space (E66 (How Small Files Land After Padding); implementation: [data-unit construction](crates/singlefs-core/src/unit.rs)). D27 (Small-Data Packing Container) is settled but unimplemented; the 11.1% in E120 (Choosing the Common Ratio of a Geometric Size-Class Table) bounds padding waste for the selected classes, not measured overhead of the current implementation or a complete container.
- Allocation record tree: the arithmetic model with 32 KiB placements and 90% fill uses about 0.055% of capacity (E74 (Allocation Record Entry Count and Compaction Cost)); this counts that tree alone, excluding other indexes, the journal, snapshots and reserved space.

## Current status

The current milestone is "sharing and reclamation" ([milestone four](.claude/kb/milestone/04-fourth-txn.md)). A readable and writable two-disk file-system library prototype exists; having core mechanisms does not complete every milestone acceptance condition. The previous stage's handoff is in [milestone three](.claude/kb/milestone/03-third-txn.md).

| Capability | Implemented | Current boundary |
|---|---|---|
| Formatting and mounting | Two-disk mkfs, writable mount, remount after closing, cold start reads | mkfs currently requires exactly two disks; arbitrary disk counts, RAID5/6 and hot-add are not delivered |
| Files and content I/O | Create several files in one publish; write by inode number and read by byte offset | No directories; both overwrite and sequential writes replace whole contents from the beginning, with no arbitrary-offset partial-write entry point |
| Deletion and reuse | Publish a deletion intent, then reclaim in batches; reuse placements after the rollback floor passes them | An inode born after the first file has been fully deleted cannot receive data writes; other files already present before deletion can still be written |
| Truncation and extension | Shrinking issues an intent and reclaims in batches; extension and same-size requests each take one publish | Actual size changes support inode 1 only; other live inodes accept same-size requests only |
| Read-only snapshots | Create and destroy snapshots; old blocks retained by snapshots enter the deadlist | At most 10 coexist; the provided snapshot-reading interface reads the first file only; no writable snapshots or clones |
| Rollback and recovery | Administrator rollback while mounted, journal replay and intention continuation after recovery | Validated through supported library calls and test workloads; no complete POSIX use path |
| Transaction publication | User changes publish synchronously; one request can contain several per-unit transactions | Time/dirty-byte automatic checkpoints, cross-request batching and mount-time threshold setting are not connected |
| Compaction and reclamation | Internal paths exist for intentions, region scans, relocation and post-publish discard | The default product path stops before intention issuance because batch size is undecided; test switches driving it do not provide automatic background GC |
| Index scan rebuild | The checker can scan central-mapping, extent and inode tables into memory and compare them | No general tool rebuilds all trees and writes repairs to disk; livelist and sparse side-table entry writers are still absent |

There is no system mounting entry point (FUSE / kernel module), directory or POSIX interface, encryption, compression or small-object packing. The GPU judge serves testing; product GPU GC is unimplemented. Variable-width stripes, automatic compaction and rebuildability in the design table are goals, not evidence of current product capability.

**Do not store any data in it.** The disk format is still soft and may be replaced at any time before the first external user appears ([format evolution rules](.claude/rules/format-evolution.md)). Future code work and acceptance conditions are recorded under pending optimization at the end of milestone four.

For actual library usage, start with the [multi-inode content-write tests](crates/singlefs-harness/tests/writes_by_inode_number_of_several_files.rs) and [truncation-intention and batch tests](crates/singlefs-harness/tests/truncation_intent_batches_and_final_batch.rs). These are development examples, not file-system commands for everyday use.

### Verification mechanisms and evidence scope

The table lists existing mechanisms and entry points; it does not mean the current workspace has passed every gate:

| Verification | What it judges | Gate |
|---|---|---|
| Crash-point replay | Cut the write-request stream into segments at each disk's barriers, enumerate every persistence state inside each segment, build an image for each, and run recovery plus the pool-level checker | checker-tier-crash-point-replay; the full run happens only on user request |
| Crash amplification | The same enumeration scaled to billions of states: streams are first recorded into the verdict library, then read back and judged; the judge runs on GPUs, sharded across two hosts | `scripts/crash/crash-amplification.sh`, run on user request |
| Pool-level checker | The code list `IMPLEMENTED_INVARIANTS` contains 108 entries; applicability depends on the image and prerequisites, so the count does not mean all 108 are fully judged on every image | Called by the two rows above |
| Model differential testing | Random operation sequences compared step by step with an ideal model that lives only in memory | harness-model-differential-and-scenarios |
| QEMU real device | A fixed workload on two virtio disks; the writes and FLUSHes recorded on the device side are compared item by item with the stream the program recorded; no crash injection | checker-tier-qemu-device-streams |
| Memory ordering | herd7 judges every Never under `litmus/`, each paired with a control that drops the barrier | checker-tier-lkmm |
| Formal verification | Verus verifies the pure-function recovery prefix, root-selection and ring-scan core used by the product; separate compaction-plan/abstract-model proofs exclude core relocation execution and crashes and are not an end-to-end file-system proof | checker-tier-formal-verification |
| Mutation | The crates mutation table breaks one place per row and the named test must go red; broad mutation (cargo-mutants) requires a conclusion for every mutant and none left unkilled | checker-tier-crates-mutation-replay, checker-tier-broad-mutation |

The checker returns satisfied, violated, not applicable or partially judged; the latter two are not full satisfaction. Zero violations or zero GPU violation bits do not establish that every applicable clause was fully judged.

Targeted audit record: the workspace audit on 2026-10-23 ran 6 harness targets, with 60 passed and 0 failed, covering multi-inode writes, snapshots, truncation fields/batches and internal compaction behavior. It did not rerun the whole workspace, GPU, Verus, QEMU or domain-wide crash validation and is not overall acceptance; commands, limits and logs are in the [code-first audit report](records/2026-10-23-代码先行的工程进度与决策核对.md).

Historical full-run record (not a full-run conclusion for the current workspace): the full run of 2026-10-16 to 2026-10-18 covered 14 crash enumeration cases; the 12 whose logs print a state count total 5,149,586,107 crash states (about 5.15 billion), every state with a verdict under the same key and 0 violations; verdicts are stored in the crash verdict library and reused by function unit key, and the judge runs on the GPUs of two hosts. Case 6 was not rerun on the final tree, and the quick tier judgement of case 12 was changed on the last day. The six heavy gates had one pass on 2026-10-15: herd7 and QEMU (6 VM runs, 123 checks) green, formal verification and the mutation table ran their light tier (2 of 172 mutation rows not red), and broad mutation had no conclusions for that version of the code; test numbers, measurement basis and omissions are in the "Summary of test data" section of the [2026-10-18 brief](briefs/2026-10-18.md).

**An all-green gate is only evidence of crash consistency at the model level**, not "verified". For how far each item reaches, which items are recorded as not run this time, and what is not covered yet, see the [latest brief (2026-10-25)](briefs/2026-10-25.md) (historical briefs in [`briefs/`](briefs/)).

## System configuration

Mounting reads the **system configuration** from two 4096-byte slots at offsets 0 and 4096 on each disk. The defined structure is 489 bytes and the slots alternate. It supplies the geometry and identity needed by the chain system configuration → root ring → root record → tree table → index trees → data units. Slot redundancy does not provide automatic recovery after all configuration copies are lost.

The format divides fields into four classes; reserved fields do not mean every setting interface or reconstruction mechanism is implemented:

| Format class | Bytes | Current implementation boundary |
|---|---:|---|
| Immutable configuration | 389 | Format, layout and geometry identities; only some values have mkfs parameters, not every field is tunable |
| Mutable configuration | 4 | Index-node size; currently implemented as a 16 KiB constant, without a general product entry point to resize and rebuild indexes |
| Runtime configuration | 36 | Time, dirty-byte and compaction-watermark fields use defaults; mount-time setting and corresponding automatic triggers are absent |
| Runtime quantities | 60 | Generations, watermarks and other state maintained by the file system and updated by the relevant publications; users cannot set them directly |

Self-describing units support scanning and cross-checking; current reconstruction reads some index tables into memory, not all metadata into a repaired on-disk file system. Full fields and compatibility conditions: [byte layout table](.claude/kb/layout/01-first-txn.md).

## Contributor governance

> **Implementation That Favors the AI, Review That Favors the Human (AI-friendly in implementation, human-friendly in review).**
>
> **Make every submitted patch review-worthy (ensure every submitted patch is worth reviewing).**
>
> **Contribution throughput may be unbounded; acceptance throughput is evidence-bound (contribution throughput may be unbounded, while acceptance throughput is constrained by evidence).**

The gate does not exist to keep anyone out. It exists to raise every submission to the line where it is worth a human's time: the mechanical part goes to scripts, and human attention goes to what only a human can judge.

**The acceptance criterion is automated verification.** This project does not classify submitters by origin; there is only one distinction: submissions that carry evidence, and those that do not. The evidence criterion is the same ruler for everyone; run the gate before sending and you know where you stand.

> Disclosure: the code, documentation and gate scripts of this project are produced by humans and AI together, and will stay that way. This describes how we work; it is not a classification of submitters.

## Contributing

Three hard requirements:

1. **Any change to `crates/*/src/` must come with tests.** No exceptions, no "the next patch will add them".
2. **Every new test must first prove that it goes red**: break the code under test, confirm the test fails, then restore it, and write in the commit message how you checked. A test that cannot be shown to go red does not count.
3. **Unimplemented verification must not pretend to pass.** The gate lists the stages that are not wired in yet; green only means the implemented part passed.

```bash
SINGLEFS_RUN_SCOPE='种类=target; 跑什么=多inode内容写测试; 范围=1 个测试目标; 为什么=验证README的库级使用入口' \
bash scripts/hosts/capped.sh default \
bash scripts/hosts/run-with-memory-cap.sh 12G \
bash scripts/hosts/run-build-directory.sh \
cargo test -p singlefs-harness --test writes_by_inode_number_of_several_files

bash scripts/crash/crash-point-replay.sh --list-items
```

The example runs one harness target. 12G is the example memory cap; the project registry supplies thread counts and the wrapper cleans the temporary build directory. `--list-items` lists registered cases without running validation. Choose other targets by change scope; resource prerequisites, scope declarations and full pre-commit gate commands are in the [commit workflow](.claude/skills/gate/staged-gate.md).

The checker tier (crash enumeration cases, QEMU, herd7, mutation tables, formal verification, broad mutation) runs by default only at commit time, with `SINGLEFS_HEAVY_TESTS=commit` on the command; the full run of crash-point replay and of crash amplification happens only on user request. What counts as heavy, and how the light tier and heavy tier are split, is in [`.claude/rules/verification.md`](.claude/rules/verification.md).

Prerequisites: herd7 needs `opam install herdtools7` and `SINGLEFS_KERNEL_TREE=` pointing at a Linux source tree that has `tools/memory-model`; the QEMU stage needs KVM and a QEMU patched with blklogwrites (build it with `scripts/devices/build-patched-qemu.sh` into a private directory, then write `qemu.env` at the repository root following [`qemu.env.example`](qemu.env.example)); without them it refuses to run rather than falling back to the system QEMU. Prerequisites of the VM harness are in [`.claude/kb/vm-harness.md`](.claude/kb/vm-harness.md).

### Multiple hosts and GPUs (optional)

Write a local `multi-host.env` (git-ignored) from the template [`multi-host.env.example`](multi-host.env.example) at the repository root: two-host sharding, the GPU judge, the order and quota of GPU cards, and the quiesce commands are all there, with each key explained in the template; afterwards run `bash scripts/crash/crash-point-replay-shard-configuration-check.sh` once to check it. Every GPU run quiesces first and does not restore afterwards (`scripts/crash/gpu-quiesce-run.sh`). To move a job whose result is a file onto the second host: `bash scripts/hosts/multi-host-run.sh --peer -- <command>`. Where each script lives is in [`scripts/README.md`](scripts/README.md).

The gate scripts and rules are distributed by singlefs-ai-sop, so every participant runs the same set.

## Getting started

The rules and the gate live in a separate repository, singlefs-ai-sop. The same specification has Chinese, English and Japanese repositories ([`singlefs-ai-sop-zh`](https://github.com/faliye/singlefs-ai-sop-zh), [`-en`](https://github.com/faliye/singlefs-ai-sop-en), `-ja`), and `-en` is authoritative externally; this project uses `-zh`. What the project holds is a **copy**, without the upstream `.git` and `CLAUDE.md`:

```bash
git clone https://github.com/faliye/singlefs.git
git clone https://github.com/faliye/singlefs-ai-sop-zh.git
cd singlefs
rsync -a --exclude '.git' --exclude CLAUDE.md ../singlefs-ai-sop-zh/ .claude/singlefs-ai-sop/
diff -rq --exclude=.git --exclude=CLAUDE.md ../singlefs-ai-sop-zh .claude/singlefs-ai-sop   # no output means the copy is complete
bash .claude/singlefs-ai-sop/install.sh     # refresh the version stamp .singlefs-ai-sop-version
bash scripts/crash/crash-point-replay.sh --list-items
```

After an upstream release, copy again with the same `rsync` and run `install.sh` again. **Rule changes go to the upstream repository**; do not edit `.claude/singlefs-ai-sop/` in place.

## Layout

| Path | Contents |
|---|---|
| [`crates/singlefs-format`](crates/singlefs-format) | Format constants; the value of every width comes from a decision item in the kb |
| [`crates/singlefs-core`](crates/singlefs-core) | mkfs, allocator, transaction layer, writable mount and session, recovery, reads on a mounted pool, the index trees, block device backends |
| [`crates/singlefs-harness`](crates/singlefs-harness) | Harness tier: unit and integration tests, plus scaffolding such as the recorder, memory pool, ideal model, random history and fault injection |
| [`crates/singlefs-checker-tier`](crates/singlefs-checker-tier) | Checker tier: crash-state enumeration, verdict library, crash amplification, GPU judge, real-device fixtures; runs by default only at commit time |
| [`crates/singlefs-checker`](crates/singlefs-checker) | Pool-level checker: the library that judges one image; shares only the format constants with the implementation |
| [`crates/singlefs-replay-prefix`](crates/singlefs-replay-prefix) | The pure-function core of recovery replay that takes the prefix, does root selection and the ring scan; this is what Verus verifies |
| [`crates/singlefs-unit-graph`](crates/singlefs-unit-graph) | Function-unit graph: splits Rust and WGSL source into units, fingerprints them with comments stripped, collects the closure from an entry point and computes the reuse key; depends only on std |
| [`crates/singlefs-reference-filesystem`](crates/singlefs-reference-filesystem) | RefFS: a semantic-level reference file system that walks POSIX operations in memory only |
| [`crates/mutations.tsv`](crates/mutations.tsv) | The mutation table of the crates |
| [`scripts/`](scripts/) | Scripts used again and again, in subdirectories by purpose; see its README for what goes where |
| [`litmus/`](litmus/) | herd7 litmus tests |
| [`.claude/kb/`](.claude/kb/) | Decisions and their change history, experiments, owed checks, byte layout tables, milestone plans, prior-art research and pitfalls |
| [`.claude/rules/`](.claude/rules/) | Project-local rules; the shared rules are in `.claude/singlefs-ai-sop/rules/` |
| [`.claude/gate.d/`](.claude/gate.d/) | Project-local gate stages and the registry files they read |
| [`.claude/agents/`](.claude/agents/), [`.claude/hooks/`](.claude/hooks/), [`.claude/skills/`](.claude/skills/) | Subagent definitions, Claude Code hooks, skills |
| `.claude/singlefs-ai-sop/` | Copy of the shared specification and gate scripts, git-ignored, fetched in "Getting started" |
| [`research/`](research/) | Three-way argument materials and verdicts (`prompts/`), retained experiment artifacts (`results/`), for tracing design and validation evidence |
| [`records/`](records/) | The construction process |
| [`briefs/`](briefs/) | A brief per update, one per date: what that version can do, how far it is verified, and what is not covered yet. Each brief has Chinese (default), English (`.en.md`) and Japanese (`.ja.md`) versions |

## License

Dual-licensed: [Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT), at your option.

Any contribution submitted to and accepted by this project is, as defined by Apache-2.0, licensed under the dual license above, without any additional terms.
