## E53 Can Root Ring Mount After Losing an Entire Disk — Run (2026-08-30, VM Real Disks, N=5 Rounds Byte-Identical, 6 Unit Tests / 6 Mutation Rules Caught): **All Criteria Passed, Worst-Case Rollback 1 Generation**

[中文](53-丢一整块盘之后根环还挂不挂得上.md) · **English** · [日本語](53-丸ごと1台のディスク喪失後にルートリングがマウントできるか.ja.md)

**This is the very first failure domain experiment in this project to actually touch physical devices.** E46 (Root Ring Region Spacing and Failure Domains) / E47 (Root Ring Failure Domain Losses) /
E48 (Feasible Points for Root Ring Geometry) / E50 (Upper and Lower Bounds on Root Ring Slot Count) were all pure arithmetic models
(**0 file operations, 0 fault injections**), judging geometry; E53 (Can Root Ring Mount After Losing an Entire Disk) writes roots on **real block devices inside a VM**,
**actually wipes an entire disk**, and executes the root selection path of recovery.

### What It Tests and What It Does Not Test

| Tests | Does Not Test |
|---|---|
| Whether feasible point conclusions of E48 (Feasible Points for Root Ring Geometry) reproduce on real devices: **valid root remains readable after losing an entire disk, worst-case rollback 1 generation** | **Prime step size distributing regions across distinct disks**—VM has 4 independent raw disks without striping, regions mapped 1:1 to disks, exactly the ideal placement prime step size seeks to achieve on striped arrays |
| Root selection algorithm behavior under real I/O (verify all candidates individually first, then choose newest by generation) | **Physical failure domains** (controller / backplane / power supply / firmware batch)—requires real hardware, infeasible on host |
| Whether checksum correctly rejects wiped slots | **EIO-form disk failure**: injection in E53 (Can Root Ring Mount After Losing an Entire Disk) is "entire disk wiped", not "read returns EIO"; minimal initramfs lacks `dm-error` |

### What to Measure

4 disks × 1 region per disk × 2 slots per region = 8 root slots, **cross-region round-robin** (generation t writes to slot `(t / 4) mod 2` in region `t mod 4`).

1. **Phase A**: Write 24 generations of root records (3 laps) using O_DIRECT across 4 real disks, each record containing magic + generation + slot + CRC32.
2. **Phase B (real injection)**: Entirely **zero out** the region hosted on disk k—real write via same O_DIRECT path.
3. **Phase C**: Re-read all 8 slots across 4 disks, verify magic and CRC32 individually, and pick newest by generation among valid slots.

Reports three numbers: `survivors` (count of passing slots), `chosen_gen` (selected generation), and `rollback = latest generation − chosen_gen`.

### Criteria (Hardcoded Before Run)

| # | Criterion |
|---|---|
| 1 | **`survivors ≥ 1` after losing one disk** (can mount)|
| 2 | **`rollback ≤ 1`**—the exact number predicted by E48 (Feasible Points for Root Ring Geometry) model, E53 (Can Root Ring Mount After Losing an Entire Disk) tests if it holds on real devices |
| 3 | Must lose all four disks **one by one**, all four satisfying criteria 1 and 2 (forbidden to conclude from dropping only one disk)|

### What Constitutes Failure (Hardcoded Before Run, Forbidden to Change After Run)

| Clause | Judgment When Triggered |
|---|---|
| **Positive control**: When no injection occurs, `rollback` must be **0** and `survivors` must be **8** | Otherwise ⇒ read or write path flawed, **entire round aborted** |
| **Negative control**: **Wipe all four disks** ⇒ `survivors` must be **0** | Non-zero ⇒ not reading disk (likely reading cache), **entire round aborted** |
| `rollback > 1` | **Faithfully record**—means E48 (Feasible Points for Root Ring Geometry) model failed on real devices, far more valuable than "expected" |
| Cannot read `E7RESULT` closing line in VM | Per `.claude/singlefs-ai-sop/rules/command-safety.md` integrity gate, **entire round aborted**, forbidden to treat as 0 |

### Prerequisites

- VM harness (`scripts/devices/vm-bench.sh`), **E53 (Can Root Ring Mount After Losing an Entire Disk) added multi-disk support for it** (`VM_DISKS=4`);
  run `--selftest` after adding, count only after passing.
- Binaries must be static (musl target).
- ⚠️ **The only readable kernel on this host has lockdep** ⇒ **VM timing cannot be compared to host**;
  E53 (Can Root Ring Mount After Losing an Entire Disk) reports only **counts** (surviving slots, generations), not time, unaffected by this.

### Measurement Basis and Reproduction

- Code `scripts/experiment-apparatus/e53_ring_failure.rs`.
- Reproduction:
  ```
  cd scripts/experiment-apparatus && cargo build --release --target x86_64-unknown-linux-musl --bin e53_ring_failure
  VM_DISKS=4 bash scripts/devices/vm-bench.sh target/x86_64-unknown-linux-musl/release/e53_ring_failure
  ```
- Raw output `e53-ring-failure-2026-08-30.out`.
- Mutation table `scripts/mutation/tables/e53_ring_failure.tsv`.

### Actual Measurement (2026-08-30, QEMU 4 virtio Real Disks, Pure O_DIRECT, N=5 Rounds Byte-Identical)

| Scenario | Surviving Slots | Selected Gen | Rollback | Mounts |
|---|---|---|---|---|
| **Positive control: no injection** | **8 / 8** | 23 | **0** | Yes |
| Wipe /dev/vda (Region 0) | 6 | 23 | **0** | Yes |
| Wipe /dev/vdb (Region 1) | 6 | 23 | **0** | Yes |
| Wipe /dev/vdc (Region 2) | 6 | 23 | **0** | Yes |
| **Wipe /dev/vdd (Region 3, hosting latest gen)** | 6 | **22** | **1** | Yes |
| **Negative control: wipe all four** | **0** | NA | NA | No |

**Itemized Criteria Verdicts**:

| # | Criterion | Verdict |
|---|---|---|
| 1 | `survivors ≥ 1` after losing one disk | **Passed** (6 in all four cases)|
| 2 | `rollback ≤ 1` | **Passed** (worst-case 1, occurring when "latest gen falls precisely on wiped region")|
| 3 | Must satisfy across losing all four disks individually | **Passed** (passed all 4)|
| Positive control | No injection ⇒ 8 survivors, 0 rollback | **Passed** |
| Negative control | Wipe all ⇒ 0 survivors | **Passed**—⇒ **truly reading disk, not cache** |

⇒ **Model conclusion of E48 (Feasible Points for Root Ring Geometry) reproduced on real devices**: cross-region round-robin + regions on distinct disks,
always mountable after losing an entire disk, worst-case rollback 1 generation.
**Rollback of 1 generation appears only when "latest generation happens to land on the wiped region"**,
consistent with "taking worst case over phase" in E47 (Root Ring Failure Domain Losses) model.

### Extra Gate Added in 2026-08-30 Run (Revealed by Mutation Testing)

`M3_magic_omitted` **did not turn red in any test during round 1**: wiped slots were all zeroes,
**CRC alone rejected them** ⇒ all-zero test **cannot test the magic gate**.
Added test creating record with "**wrong magic, but self-consistent CRC**"—only magic can reject it; all 6 caught after patch.
⚠️ **Not a theoretical issue**: in real scenarios, unfamiliar blocks (other structures, residue from previous mkfs)
can easily carry a self-consistent checksum; **judgment then relies entirely on magic**.

### Measurement Basis and Known Limitations

- Code `scripts/experiment-apparatus/e53_ring_failure.rs`,
  raw output `e53-ring-failure-2026-08-30.out` (9 records, closing line `emitted=9`).
  Mutation table `scripts/mutation/tables/e53_ring_failure.tsv` (6 rules, all caught).
- Reproduction (E53 (Can Root Ring Mount After Losing an Entire Disk) requires VM, not in quick tier of `replay.sh`):
  ```
  cd scripts/experiment-apparatus && cargo build --release --target x86_64-unknown-linux-musl --bin e53_ring_failure
  VM_DISKS=4 VM_DISK_MB=64 bash scripts/devices/vm-bench.sh target/x86_64-unknown-linux-musl/release/e53_ring_failure
  ```
- ⚠️ **Injection form is "entire content wiped", not "read returns EIO"**. Minimal initramfs lacks `dm-error`,
  and true device removal (`device_del`) was not performed. ⇒ **E53 (Can Root Ring Mount After Losing an Entire Disk) proves "whether root can be selected when content vanishes"**,
  not "what happens to driver layer when device is detached".
- ⚠️ **Regions correspond 1:1 to disks, no striping in VM**: E53 (Can Root Ring Mount After Losing an Entire Disk) tests only consequences "after regions are indeed on distinct disks".
  ⚠️ **That premise was verified on real dm-stripe devices by E55 (Stripe Mapping Arithmetic Validity on Real Devices)**
  (`(p/chunk)%devs` 72/72 cells correct; all six prime step size groups landed on distinct disks;
  non-prime positive controls all collided on same disk) ⇒ **both premise and consequence have real-device evidence.**
- ⚠️ **Physical failure domains** (controller / backplane / power supply / firmware batch) **remain 0 coverage**—requires real hardware.
- 4 disks × 64 MiB per disk; root region starts at 1 MiB, 4096 bytes per slot, 2 slots per region, 24 generations total (3 laps).
- VM kernel has lockdep ⇒ **time cannot be compared to host**; E53 (Can Root Ring Mount After Losing an Entire Disk) reports only counts, unaffected.

### Impacted Decisions

| Decision Item | Relationship | Review |
|---|---|---|
| D5 (Snapshot / Space Accounting Mechanism) settled item 2 | Prepares | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D22 (Synthesizing Unit Atomicity) settled item 2 | Supports | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D16 (Publish Semantics) settled item 6 | Supports | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |

## Historical Versions

### 2026-08-30

- Run completed, all criteria passed. Criteria untouched by a single word. Added multi-disk support (`VM_DISKS`) to `vm-bench.sh`,
  running `--selftest` before testing. Mutation testing found one blind spot (magic gate not tested independently), patched.
- Initiated. Basis: user pointed out "I haven't seen a failure domain test report to date", audit confirmed **failure domain tests did not exist**,
  all four existing experiments being pure arithmetic models. E53 (Can Root Ring Mount After Losing an Entire Disk) fills the first one actually touching devices.
