# singlefs

[中文](README.md) · [English](README.en.md) · [日本語](README.ja.md)

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

Several pain points of existing COW file systems are **format-level**: patches cannot fix them, they can only be avoided at design time:

| Pain point | Root cause | Choice made here |
|---|---|---|
| Absurd ENOSPC (space is free but writes fail; deleting a file also reports no space) | Two-level allocation plus rigid chunk classes | No data/metadata split; one global allocator |
| RAID5/6 write hole | Fixed-width stripes need read-modify-write | Variable stripe width, never read-modify-write |
| Quota accounting that is slow and wrong | Accounting is an after-the-fact traversal that also walks the reverse index | Accounting is a by-product of the transaction, maintained incrementally at commit |
| The reverse index became a complexity black hole | It grew later; it was not designed | Designed into the format from day one |
| Nobody dares to run the repair tool | Rebuildability was never a design goal | Self-describing metadata blocks, plus a checker that evolves with the format |

Decisions and their grounds are in [`.claude/kb/decisions.md`](.claude/kb/decisions.md); the pitfall list is in [`.claude/kb/pitfalls.md`](.claude/kb/pitfalls.md).

### Performance estimates

Every number comes from an experiment that has been run; read each on its own measurement basis and do not convert between them:

- Usable space after formatting: on a 16 GiB disk, 95.2% is usable on a single disk and 47.6% on a two-disk mirror; the default formatting of six other file systems is in the same range (E152 (per-milestone file performance against six file systems), table in [`scripts/experiments/perf-by-milestone.md`](scripts/experiments/perf-by-milestone.md)).
- Small files: any data under 32 KiB is padded to one 32 KiB unit (E66 (how small files land after padding)). Small-object packing has been decided by D27 (small-data packing container) but is not implemented yet; once implemented, the worst-case padding waste is 11.1% (E120 (choosing the common ratio of a geometric size-class table)).
- Metadata overhead: the allocation record tree takes about 0.055% of capacity at 32 KiB placements and 90% fill (E74 (allocation record entry count and compaction cost), arithmetic model).

## Current status

The current milestone is "sharing and reclamation" ([`.claude/kb/milestone/04-fourth-txn.md`](.claude/kb/milestone/04-fourth-txn.md)); the main body of the previous one, "deletion and repeated COW" ([`.claude/kb/milestone/03-third-txn.md`](.claude/kb/milestone/03-third-txn.md)), is complete, and what remains open is segment 4, "optimization" (2·6, 6·4).

What it can do: mkfs on two disks; write a file and commit, filling several data units sequentially and reading back by offset; create several files in one publish (no directories); overwrite, release, and reuse a placement once the rollback floor has been raised past it; close and remount as a second writable instance; perform an administrator rollback while mounted; read back after a cold start; overwrite, sequentially write and delete a file by inode number (deletion publishes the deletion intent first, then reclaims its data units in batches); truncate (shrinking publishes an intent first and then reclaims the data units beyond the size in batches; growing and keeping the same size each take one publish); create and destroy read-only snapshots.

What it cannot do: it cannot be mounted into the system (no FUSE, no kernel module), has no directories or POSIX interface, and has no writable snapshots, encryption or compression; the entry point for compaction (GC) exists, but the batch-size clause is not yet decided, so the product path stops before it signs a compaction intent. **Do not store any data in it.** The disk format is still soft and may be torn down and redone at any time before the first external user appears ([`.claude/rules/format-evolution.md`](.claude/rules/format-evolution.md)).

How far verification reaches:

| Verification | What it judges | Gate |
|---|---|---|
| Crash-point replay | Cut the write-request stream into segments at each disk's barriers, enumerate every persistence state inside each segment, build an image for each, and run recovery plus the pool-level checker | checker-tier-crash-point-replay; the full run happens only on user request |
| Crash amplification | The same enumeration scaled to billions of states: streams are first recorded into the verdict library, then read back and judged; the judge runs on GPUs, sharded across two hosts | `scripts/crash/crash-amplification.sh`, run on user request |
| Pool-level checker | The 102 invariants marked implemented in the invariant list (`crates/singlefs-checker/invariants/`), the same count as the list in the code | Called by the two rows above |
| Model differential testing | Random operation sequences compared step by step with an ideal model that lives only in memory | harness-model-differential-and-scenarios |
| QEMU real device | A fixed workload on two virtio disks; the writes and FLUSHes recorded on the device side are compared item by item with the stream the program recorded; no crash injection | checker-tier-qemu-device-streams |
| Memory ordering | herd7 judges every Never under `litmus/`, each paired with a control that drops the barrier | checker-tier-lkmm |
| Formal verification | Verus verifies the pure-function core of recovery replay that takes the journal prefix, does root selection and the ring scan (`crates/singlefs-replay-prefix`); this same code is what gets compiled into the product | checker-tier-formal-verification |
| Mutation | The crates mutation table breaks one place per row and the named test must go red; broad mutation (cargo-mutants) requires a conclusion for every mutant and none left unkilled | checker-tier-crates-mutation-replay, checker-tier-broad-mutation |

How much has been judged: the full run of 2026-10-16 to 2026-10-18 covered 14 crash enumeration cases; the 12 whose logs print a state count total 5,149,586,107 crash states (about 5.15 billion), every state with a verdict under the same key and 0 violations; verdicts are stored in the crash verdict library and reused by function unit key, and the judge runs on the GPUs of two hosts. Case 6 was not rerun on the final tree, and the quick tier judgement of case 12 was changed on the last day. The six heavy gates had one pass on 2026-10-15: herd7 and QEMU (6 VM runs, 123 checks) green, formal verification and the mutation table ran their light tier (2 of 172 mutation rows not red), and broad mutation has no conclusions for this version of the code yet; the numbers of every test are summarized in the "Summary of test data" section of the 2026-10-18 brief.

**An all-green gate is only evidence of crash consistency at the model level**, not "verified". For how far each item reaches, which items are recorded as not run this time, and what is not covered yet, see the latest file in [`briefs/`](briefs/).

## System configuration

Mounting starts by reading two 4096-byte slots at offsets 0 and 4096 of each disk. They hold the **system configuration**: a 489-byte structure, one per disk, alternating between the two slots. It is the start of the whole chain (system configuration → root ring → root record → tree table → each index tree → data units), and also the only small piece on disk that cannot be rescued: everything else can be scanned back from self-describing units, but geometry such as where the journal ring is and how large a unit is has no second copy anywhere.

Its contents fall into four classes: immutable configuration set by the user at mkfs, which needs a new file system to change (389 bytes); mutable configuration whose change only rebuilds indexes without losing data (4 bytes); run configuration that can be reset at every mount (36 bytes); and run quantities the file system maintains itself and the user can never set (60 bytes, changing at every publish, so do not cache them as constants). The field table is in [`.claude/kb/layout/01-first-txn.md`](.claude/kb/layout/01-first-txn.md).

## Contributor governance

> **Implementation That Favors the AI, Review That Favors the Human.**
>
> **Make every submitted patch review-worthy.**
>
> **Contribution throughput may be unbounded; acceptance throughput is evidence-bound.**

The gate does not exist to keep anyone out. It exists to raise every submission to the line where it is worth a human's time: the mechanical part goes to scripts, and human attention goes to what only a human can judge.

**The acceptance criterion is automated verification.** This project does not classify submitters by origin; there is only one distinction: submissions that carry evidence, and those that do not. The evidence criterion is the same ruler for everyone; run the gate before sending and you know where you stand.

> Disclosure: the code, documentation and gate scripts of this project are produced by humans and AI together, and will stay that way. This describes how we work; it is not a classification of submitters.

## Contributing

Three hard requirements:

1. **Any change to `crates/*/src/` must come with tests.** No exceptions, no "the next patch will add them".
2. **Every new test must first prove that it goes red**: break the code under test, confirm the test fails, then restore it, and write in the commit message how you checked. A test that cannot be shown to go red does not count.
3. **Unimplemented verification must not pretend to pass.** The gate lists the stages that are not wired in yet; green only means the implemented part passed.

```bash
bash .claude/scripts/gate.sh                                  # full gate round: shared stages plus the project stages in .claude/gate.d/
cargo test -p singlefs-harness                                # harness tier: unit and integration tests, run any time after a change
bash scripts/hosts/run-with-memory-cap.sh <cap> bash scripts/crash/crash-point-replay.sh --quick   # light-tier crash-point replay
```

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
bash .claude/scripts/gate.sh
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
| [`research/`](research/) | Three-way review material and verdicts (`prompts/`) and retained experiment artifacts (`results/`); both can be deleted wholesale and fetched again |
| [`records/`](records/) | The construction process |
| [`briefs/`](briefs/) | A brief per update, one per date: what that version can do, how far it is verified, and what is not covered yet. Each brief has Chinese (default), English (`.en.md`) and Japanese (`.ja.md`) versions |

## License

Dual-licensed: [Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT), at your option.

Any contribution submitted to and accepted by this project is, as defined by Apache-2.0, licensed under the dual license above, without any additional terms.
