# VM harness: how to run an experiment on a real block device

[中文](vm-harness.md) · **English** · [日本語](vm-harness.ja.md)

**Why a standalone article**: Block device behavior is the tested subject of several experiments, yet "how to send an experiment into a VM"
was re-derived every time, twice getting stuck in the same places (kernel image unreadable, binary not static).
This document covers only the current status, not history.

## In short

```bash
cd scripts/experiment-apparatus
cargo build --release --target x86_64-unknown-linux-musl --bin <experiment_bin_name>
bash ../devices/vm-bench.sh target/x86_64-unknown-linux-musl/release/<name> /dev/vda <args...>
```

## How many disks to attach

`VM_DISKS` (default 1). When >1, guest sees `/dev/vda`, `/dev/vdb`, …
**All device paths are passed in order as prefix arguments to the binary**, followed by the binary's own arguments.

```bash
VM_DISKS=4 VM_DISK_MB=64 bash scripts/devices/vm-bench.sh <static_binary>
```

Multiple disks are required for E53 (Can Root Ring Mount After Losing an Entire Disk): "whether it can still mount after losing an entire disk"
can only be asked when there are truly multiple disks. After modifying the harness, **run `--selftest` first**; it counts only if it passes.

⚠️ **Minimal initramfs lacks `dm-error` / `mdadm`**:
Cannot simulate "reads return EIO" or "striped array"; can only simulate "truly zeroing out the content of a disk".

## Three preconditions, each with an established resolution

| Precondition | How to check | What to do if unmet |
|---|---|---|
| **KVM readable and writable** | `id -nG \| grep -w kvm`, then test `[ -r /dev/kvm ] && [ -w /dev/kvm ]` | Local machine acquires permissions via **`kvm` group membership**. If unmet, run `sudo usermod -aG kvm $USER` (password is `SUDO_PASS_A` in `.env`); in current shell obtain immediately with `sg kvm -c '<command>'`, new sessions inherit automatically |
| **Kernel image readable** | `bash scripts/devices/vm-kernel.sh --check` | Run `bash scripts/devices/vm-kernel.sh`, which prints an available path; if necessary copy from `/boot` to `TMPDIR` with password in `.env` and chown. To measure time, run `bash scripts/devices/vm-kernel.sh --release $(uname -r)`: it only recognizes `/boot/vmlinuz-<version>`, copying to `singlefs-vmlinuz-<version>` under `TMPDIR`, avoiding hitting lockdep first |
| **Binary is static** | `file <binary>` | Compile experiment binary with `--target x86_64-unknown-linux-musl` (musl target installed via `rustup target list --installed`); checker tier binaries with RocksDB are compiled with `--target x86_64-unknown-linux-gnu --config 'target.x86_64-unknown-linux-gnu.rustflags=["-C","target-feature=+crt-static"]'` into static-pie (`.claude/gate.d/checker-tier-qemu-device-streams.sh`) |

⚠️ **Judging KVM precondition must check group and test read/write; never check ACL or patch with `setfacl`.**
`/dev/kvm` has `TAG+="uaccess"` (`/usr/lib/udev/rules.d/70-uaccess.rules:48`),
its ACL is issued by logind according to **local seat sessions**; SSH sessions have no seat ⇒ cannot acquire,
and manual `setfacl` rules are wiped upon next session change—**`getfacl` appears correct when set**.
**The experimentally observed failure mode is the worst kind** (2026-09-01): `qemu.sh --selftest` runs VM twice;
the first acquires KVM, the second fails to acquire, outputting "success case → 0, failure case → exit marker unreadable",
**appearing as if harness cannot distinguish failure, when in fact permissions vanished between launches**.
⚠️ **Only one readable kernel exists under local `/boot`: `vmlinuz-6.17.0-lockdep`** (others are `-rw-------`）。
**It carries lockdep**, with massive lock-checking overhead ⇒ **times measured in VM cannot be compared to host**,
though **counting metrics (I/O count, block count) are unaffected**. Citations of times measured in VM must include this statement.
⚠️ **To compare time, use `vm-kernel.sh --release $(uname -r)` to get the OEM kernel currently running on host** (used by E152 (Per-Milestone File Performance Against Six File Systems)):
Without lockdep, times across several VM runs on the same test bench can be compared with each other; they still cannot be compared with times on host (virtualization and `cache=none` are involved).
Times measured with lockdep kernel are unsuitable even for cross-VM comparison: lock checking overhead scales with lock acquisition count, and tested subjects acquire locks differently (conjectured, not measured locally).

## Hygiene check before running

`vm-bench.sh` carries `EXIT` trap to clean working directories, but **still worth checking before launching**
(this cleanup was retrofitted; before retrofitting, 154 directories and 4.5 GB were cleaned at once, see Historical Versions at end):

```bash
ps -eo pid,etimes,args --no-headers | grep '[q]emu-system'   # leftover VMs
ls -d /tmp/singlefs-vmbench.* 2>/dev/null | wc -l            # leftover working dirs
losetup -a                                                    # leftover loops
```

⚠️ **Do not use `pkill -f`** (`.claude/singlefs-ai-sop/rules/command-safety.md`):
Pattern string will match wrapper's own command line. To stop VMs, check with `ps` first, then kill with literal pid.
`VM_KEEP=1` preserves working directory to inspect scene upon failure.

## Harness must self-test first

```bash
bash scripts/devices/vm-bench.sh --selftest
```

It runs three cases: guaranteed success, guaranteed failure, and discovering `/dev/vda`.
**Failure to distinguish the failing case indicates the harness mistakes failure for success**—
identical to `.claude/singlefs-ai-sop/rules/show-me-test.md` "gates must not pretend to pass".

## What is added in VM that is absent on host

| Metric | Running on host (regular file) | Running in VM (`/dev/vda`) |
|---|---|---|
| I/O count, block count | Yes | Yes |
| **Independent block-layer readings** (`/sys/block/vda/stat`) | **No** (`blkstat=false`) | **Yes** |
| Time | Comparable | lockdep kernel: **incomparable**; `--release` OEM kernel: comparable across multiple VM runs on same bench, still incomparable to host |

⚠️ **Independent block-layer reading is the verification path for several experiments**—
E7 (Offline Index Harness) lists "block layer matching program counters cell by cell" as one of its four verification layers;
E12 (Batched Sequential Append vs Unbatched Random Page RMW) relied on it for fault injection verification
(after removing `O_DIRECT`, block-layer reading dropped to zero while program counter remained unchanged).
⇒ **Re-running these experiments on regular files reproduces numbers, not that verification.** When citing, distinguish them.

## Result extraction has an integrity gate

`vm-bench.sh` compares "how many items program claimed to emit" with "how many items host captured"; mismatch invalidates entire round.
Rationale in `.claude/singlefs-ai-sop/rules/command-safety.md`: console line beginnings are overwritten by BIOS escape sequences;
anchoring with `^` silently misses lines. Thus extraction uses `grep -ao` without anchoring line start.

## Masked CPU features tier has another gate: Known-Answer Test (KAT)

`VM_CPU` can mask CPU features from guest (`VM_CPU="host,-aes,-vaes"`).
**Once `-<feature>` appears in `VM_CPU`, `vm-bench.sh` requires tested program to emit at least one line of
`E7RESULT name=kat ... ok=true`**, otherwise entire round is invalidated.

**Why this gate is needed** (experienced on 2026-08-31, E6 (Encryption Algorithm Selection)):
`-cpu host,-aes` only strips CPUID's `aes` bit, while RustCrypto's `aes` 0.9.3
**separately** probes both `aes` and `vaes` bits (`src/lib.rs` snippet `cpufeatures::new!(features_vaes256,`) ⇒ guest still executes AES instructions:

| Observation | `-aes` | `-aes,-vaes` |
|---|---|---|
| AES 4 KiB throughput | 3747.63 MiB/s (0.5% diff from unmasked) | 244.74 MiB/s (14.94× drop) |
| AES-256-GCM tag | **`2112edc9…`, incorrect** | `62d27233…`, correct |

⇒ **That tier ran blazing fast, item counts matched, exit code 0, yet computed the wrong algorithm.**
Looking only at throughput cannot distinguish "masking ineffective" from "masking effective but calculated incorrectly"—both manifest as "numbers unchanged".

**How to write this kat**: Expected value must be provided by **another implementation** (e.g. `python3` calling `cryptography`),
never using tested program's own output on another tier as expected value—that is comparing against oneself.

**This gate proved red itself** (2026-08-31, three cases): masked tier emits no kat ⇒ red;
masked tier emits `ok=false` ⇒ red; unmasked and emits no kat ⇒ pass.

## Independent device-side recording: blklogwrites mode

When `VM_BLKLOGWRITES_DIR=<dir>`, `vm-bench.sh` wraps each disk with QEMU's `blklogwrites` filter node (local QEMU 8.2.2 includes this driver, checked on 2026-09-14).
Every write (with data) and FLUSH emitted to disk by guest is recorded in dm-log-writes format into `<dir>/log<d>.img`; data disks live in the same directory (`disk<d>.img`), preserved after run.
Recording happens outside guest, sharing no code with tested program's own recorder. Parsing and differential logic reside in `crates/singlefs-checker-tier/src/device_log.rs`, criteria in gate checker-tier-qemu-device-streams (`.claude/gate.d/checker-tier-qemu-device-streams.sh`).

⚠️ **blklogwrites branch only uses patched QEMU**: Local system QEMU 8.2.2 (Ubuntu `1:8.2.2+ds-0ubuntu1.18`) `block/blklogwrites.c` appends all data iovecs behind 2 header segments (`qemu_iovec_init(&log_qiov, niov + 2)`); when data segments exceed 1023, log write exceeds IOV_MAX = 1024, host `pwritev` returns EINVAL, guest receives EIO. When mkfs writes zeros in 4 MiB blocks and virtio-blk coalesces adjacent requests into 1024 segments, this hits, presenting as `run_failed step=mkfs cause=BlockDevice(InputOutput(…I/O error))`. 2026-10-01 measurement: 6 VMs in parallel, stock QEMU reported EIO in mkfs 12 / 12 times across two rounds, patched QEMU reported 0 / 12; qemu-io submitting 1023 segments directly failed on stock with `writev failed: Invalid argument`, 1022 passed; upstream master had not modified the same line, no upstream report filed. Patch in `scripts/devices/qemu/blklogwrites-linearize-over-iov-max.patch` (when segments + 2 exceeds IOV_MAX, copy to contiguous buffer before logging); `scripts/devices/build-patched-qemu.sh` fetches same Ubuntu source version, applies patch, installs to private directory (default `~/.local/share/singlefs/qemu`), repo root `qemu.env` (gitignored, template `qemu.env.example`) writes `SINGLEFS_QEMU_HOME=<absolute_path>`. `vm-bench.sh` only reads it on `VM_BLKLOGWRITES_DIR` branch, verifying 1022 / 1023 / 1024 / 2048 segments with `scripts/devices/qemu-blklogwrites-check.sh` before running; refuses to run if configuration missing or verification fails, never falling back to system QEMU; other branches use system QEMU as usual, testing conditions unaffected. Re-run `build-patched-qemu.sh` after changing machine or upgrading system QEMU.

| Phenomenon (2026-09-14 measured, new-pool file creation) | Measurement basis |
|---|---|
| Device-side log end of each disk has one more FLUSH than program | Program's last action on each disk is system configuration slot write without following FLUSH, added upon shutdown; differential allows exactly one extra at end |
| Guest `/sys/block/<name>/stat` write request count = program write count + FLUSH count (disk 0: 23 + 12 = 35) | Differential uses written sectors (= bytes ÷ 512) and FLUSH count; FLUSH count is stat index 15, not 14 (14 is discard latency, reading yields 0) |
| Device-side writes coalesced when using page cache | m1 (32 KiB) and m2 (16 KiB) writeback coalesced into one 48 KiB write; where positive control goes red |

⚠️ **Log disk must accommodate all written data**: `log<d>.img` size is taken from `VM_LOG_MB` (MiB), default in `vm-bench.sh` is 256. After mkfs writes 768 MiB zeros in 4 MiB blocks, actual log usage per disk is ~770 MiB (2026-09-27 measured, VM_DISKS=2, 4096 MiB/disk, all six tiers around 769–771 MiB); at 256 MiB, direct, file-overwrite, second-instance reported `run_failed step=mkfs cause=BlockDevice(InputOutput(…I/O error))` during mkfs; page-cache, skip-new-pool-file-creation-barrier, raise-rollback-floor did not report, reason uninvestigated. Gate checker-tier-qemu-device-streams therefore passes `VM_LOG_MB=4096` by default (size of one data disk; sparse image, only occupying bytes actually written); other blklogwrites runs set `VM_LOG_MB` according to volume of written data. Attributing that EIO to insufficient log disk size remains unverified: same error `run_failed step=mkfs … I/O error` is also symptom of blklogwrites exceeding IOV_MAX, compatible with random collision under concurrency (conjectured, not measured).

⚠️ This tier verifies "writes and FLUSHes emitted by program arrive verbatim with order preserved on virtual device", not real disk durability semantics: under `cache=none`, device-side FLUSH maps to host `fdatasync`, host disk's own cache is not in scope.

## Auxiliary root and disk image preallocation (two switches added for E152 (Per-Milestone File Performance Against Six File Systems))

| Switch | Function | Measurement basis |
|---|---|---|
| `VM_EXTRA_ROOT=<dir>` | Contents in directory merged verbatim into initramfs root | E152 (Per-Milestone File Performance Against Six File Systems) generates it via `scripts/devices/e152-stage-root.sh`: fio, formatting and mounting tools of each file system along with all libraries resolved by ldd, decompressed kernel modules and load orders, total 129 MB; initramfs packaged with gzip -1, adding a few seconds per VM boot |
| `VM_DISK_PREALLOCATE=1` | Disk image preallocated via fallocate instead of truncated into sparse file | Host ext4 does not allocate on the fly while guest writes; run aborted if preallocation fails, never falling back to sparse file |

⚠️ **When placing dynamically linked programs into auxiliary root, libraries (especially `/lib64/ld-linux-x86-64.so.2`) must carry execution bits.**
Kernel executes ELF loader first when running dynamic programs; without execute bit, every dynamic program in guest reports Permission denied, while statically linked programs run normally.
Measured (2026-09-15, E152 (Per-Milestone File Performance Against Six File Systems) first smoke run): libraries installed as 0644, 7 of 8 configurations stuck on first external command, only statically linked singlefs binary succeeded.
`e152-stage-root.sh` installs with 0755 and tests fio with assembled loader and libraries upon completion; auxiliary root is not delivered if this step fails.

## Timing does not log timestamps inside output forwarding loops

Guest often runs tested program as child process (`Stdio::piped()`) via wrapper, reading result lines and re-printing to standard output, captured by host via serial. **Never take line arrival timestamp inside "read line, log timestamp, forward line, read next line" loop as segment wall clock.**
Guest standard output is serial ttyS0 (`scripts/devices/vm-bench.sh` flags `-serial mon:stdio` and `console=ttyS0`), forwarding a line blocks for several milliseconds; child process writes to pipe unblocked, causing timestamps of subsequent lines to carry accumulated forwarding lag of previous lines, fluctuating and mimicking VM scheduling jitter.

Measured (2026-09-17, E152 (Per-Milestone File Performance Against Six File Systems) two formal runs on 2026-09-17): lines printed consecutively by child process with only microseconds of computation had timestamps spanning 21.7–25.8 ms; in 9 of 10 rounds, outer wall clock for overwrite, release, rollback and reuse was smaller than nanoseconds measured by child itself, physically impossible; initial two runs' first-segment wall clock "splitting into two groups" was also this backlog. Round-by-round numbers in E152 (Per-Milestone File Performance Against Six File Systems) experiment page section "What these numbers show (overwrite, release, rollback and reuse compared with two-disk mirror of six systems)" item 4.
After modifying harness to read completely before forwarding, a smoke run on same machine showed 9 lines spanning 24 µs, with outer 6 632 915 ns containing child's self-measured 6 575 144 ns (`e152-preregistration.md` section 14).

| Approach | Example |
|---|---|
| Measure time inside tested process, record nanoseconds in result line | `nanoseconds=` in `name=file_overwrite` line of real device binary; take this number when comparing wall clock per operation with others, covering only that operation |
| If slicing by arrival time is truly necessary, read child output to EOF first, record arrival timestamp per line, forward after reading | `read_lines_with_arrival_times` in E152 (Per-Milestone File Performance Against Six File Systems) harness: signature provides no output handle, preventing forwarding inside read loop |
| When both outer and inner counts exist, report inclusion self-test; rounds where outer fails to contain inner excluded from median | `file_overwrite_outer_contains_inner` in E152 (Per-Milestone File Performance Against Six File Systems), reporting `summary_excluded` for false, NA, or missing fields |

Enforced by: Built-in "relay timing" stage in shared `gate.sh` (merged upstream into `.claude/singlefs-ai-sop/scripts/relay-timing-lint.py`, no longer local gate under `.claude/gate.d/`) scans Rust and Python harnesses under `research/` and `crates/`, going red if timestamp and output are taken simultaneously inside child output read loop; detects read loops moved to another function in same file (one layer); unable to reach outputs hidden in custom functions, cross-file / 2+ layer transfers, channel / struct field transfers, or shell `while read` with `date`, listed in script header. Does not scan `research/prompts/` and `research/results/` (frozen evidence).
⚠️ Inclusion self-test only checks if outer contains inner, unable to detect concurrent drift; nanoseconds measured by child process itself is the only sound path.

## Guest block-layer counts vs program counts: Different conventions for write request counts

Write request count in guest `/sys/block/*/stat` exceeds write calls counted by program by "barrier count + FUA write count": for every barrier (`fdatasync`) and FUA write in program, block layer logs one additional write request; written bytes and FLUSH counts match one-to-one.
Measured (2026-09-17, E152 (Per-Milestone File Performance Against Six File Systems) two formal runs on 2026-09-17, all 4 groups matching per-disk readings): publish B disk 0 had 10 program writes, 2 barriers, block layer 12; disk 1 had 11 program writes, 2 barriers, 1 FUA, block layer 14; prior to publish B, disk 0 had 23 program writes, 8 barriers, 4 FUAs, block layer 35; disk 1 had 21 program writes, 8 barriers, 2 FUAs, block layer 31. Kernel rationale uninvestigated.
⇒ Convert first when comparing write requests between program counts and block-layer counts; when comparing against other file systems (which have only block-layer numbers), use block-layer counts. Block layer counts only requests and bytes without distinguishing categories; using it to corroborate "how read requests are split" can only claim compatibility (E152 (Per-Milestone File Performance Against Six File Systems) experiment page first result item 1).
