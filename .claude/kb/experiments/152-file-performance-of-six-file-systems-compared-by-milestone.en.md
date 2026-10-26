## E152 Per-Milestone File Performance Against Six File Systems — run (seven formal runs, seventh 2026-09-29: two-disk mirror seven systems + singlefs 5 rounds each, 40-for-40 on first pass; see “Runs (newest first)” for each run; 26 unit tests / 26 mutations all caught)

[中文](152-按里程碑对比六家文件系统的文件性能.md) · **English** · [日本語](152-マイルストーン別に-6-ファイルシステムのファイル性能を比較.ja.md)

The question is where XFS, ext4, F2FS, Bcachefs, Btrfs, OpenZFS, and singlefs stand in file performance on the same VM, virtual disks, and workloads, and where singlefs falls behind. The user specified large-file sequential writes and reads, 4K random reads and writes, and metadata / small files; the main agent added small-file persistence, cold mount, and formatting / usable capacity. The milestone comparison is in `scripts/experiment-apparatus/perf-by-milestone.md`. **Run only when the user requests it** (registered in `scripts/experiment-apparatus/on-request-experiments.tsv`, user decision 2026-09-19): do not rerun when the apparatus changes; the doc-experiments evidence-in-repo gate instead checks whether this page says “no formal run.” Report observations only; “the gap means singlefs should be changed in a particular way” is an inference requiring separate three-way argument.

**Background material:** Supports no decision directly. It is used on the D23 (Journal Role & Format) side to judge whether and how to reduce cold-recovery reads, and on the D17 (Implementation Layering & Third-Party Pipeline) settled item 5 side to judge when to add FUSE: at the new-pool file creation milestone only three of eight dimensions can be compared.

**Subjects and parameters:** The six systems use their defaults, except ext4's `noinit_itable` mount option, with one 16 GiB virtio disk each; a raw-disk row provides the ceiling. At the “new-pool file creation” milestone singlefs has no file interface (D17 settled item 5: the first version is a pure user-space library, not mounted), so only the checker-tier-qemu-device-streams real-device binary can run, using two disks (D2 RAID striping strategy, settled item 9). Versions, commands, and ordering are registered verbatim before the run in `e152-preregistration.md`.

**Test bench:** Local QEMU 8.2.2 + KVM, 4 vCPUs and 4 GiB; guest kernel 6.17.0-1028-oem (same as host, without lockdep, obtained with `scripts/devices/vm-kernel.sh --release`). Disks are fallocate-preallocated raw files in TMPDIR, with `cache=none,aio=native`. Measurements show relative positions on this bench; absolute values are not compared with external published numbers.

**Reproduction commands:**

```bash
bash scripts/devices/e152-run.sh research/results/e152-file-system-benchmark-<date>.out   # Single disk: eight configurations interleaved for five rounds
E152_CONFIGURATIONS="raw-md ext4-md xfs-md f2fs-md btrfs-raid1 bcachefs-replicas2 zfs-mirror singlefs" \
  bash scripts/devices/e152-run.sh research/results/e152-file-system-benchmark-mirror-<date>.out   # Two-disk mirrors
E152_ROUNDS=1 E152_CONFIGURATIONS="ext4" bash scripts/devices/e152-run.sh <artifact>              # Smoke test
E152_CONFIGURATIONS=singlefs bash scripts/devices/e152-run.sh research/results/e152-file-system-benchmark-file-overwrite-<date>.out   # After milestone, rerun singlefs arm only
```


A formal run of eight configurations over five rounds takes about 45 minutes. During that time do not compile or run gates on the host: checker-tier-crash-point-replay, checker-tier-qemu-device-streams, and checker-tier-research-build-and-replay consume CPU or start VMs and would contaminate timings.

Code: `scripts/experiment-apparatus/e152_file_system_benchmark.rs` (26 unit tests; guest single-round workload and host summary modes); mutation table `scripts/mutation/tables/e152_file_system_benchmark.tsv` (all 26 caught); additional root `scripts/devices/e152-stage-root.sh`; driver `scripts/devices/e152-run.sh`; tables `scripts/experiment-apparatus/e152-tables.py` (`--selftest`). Original outputs: `e152-file-system-benchmark-2026-09-15.out` (single disk, 890 lines, final line `emitted=174`) and `e152-file-system-benchmark-mirror-2026-09-15.out` (two-disk mirror, 941 lines, final line `emitted=174`). Preregistration `e152-preregistration.md` has six revisions and four smoke tests before the formal run in section 10.

### Runs, newest first

| Date | Scope | Rounds / VM runs | Artifact (`research/results/`) | Results |
|---|---|---|---|---|
| 2026-09-29 | After the crash amplification full run and the quiesce, seven two-disk mirrors + singlefs | 5 / 40, all `vm_exit=0` | `e152-file-system-benchmark-mirror-2026-09-29.out` | “2026-09-29: seven two-disk mirrors + singlefs”; cell-by-cell comparison with 2026-09-28 in `scripts/experiment-apparatus/perf-by-milestone.md` III.4 |
| 2026-09-28 | Milestone two wrap-up step 8, seven two-disk mirrors + singlefs | 5 / 40, all `vm_exit=0` | `e152-file-system-benchmark-mirror-2026-09-28.out` | “2026-09-28: seven two-disk mirrors + singlefs”; singlefs compared row by row with 2026-09-17 in `scripts/experiment-apparatus/perf-by-milestone.md` III.3 |
| 2026-09-19 | Singlefs arm only, after apparatus changed to finish reading before forwarding; separate P1 recalculation and G1 single-vCPU, 3 rounds | 5 + 3 | `e152-file-system-benchmark-second-transaction-relay-timing-2026-09-19.out`, etc. | “2026-09-19: singlefs arm after reading before forwarding” |
| 2026-09-17 wrap-up | Singlefs arm only (at milestone wrap-up) | 5 | `e152-file-system-benchmark-second-transaction-final-2026-09-17.out` | “2026-09-17 wrap-up” section; comparison with six mirrored systems in “What these numbers mean (overwrite, release, rollback, and reuse, compared with the six two-disk mirrored systems)” section |
| 2026-09-17 step 7 | Singlefs arm only (`file-overwrite` mode) | 5 | `e152-file-system-benchmark-second-transaction-2026-09-17.out` | “Step 7” within “2026-09-17 wrap-up” |
| 2026-09-15 mirrors | Seven mirrored configurations + singlefs | 5 / 41 (one retry) | `e152-file-system-benchmark-mirror-2026-09-15.out` (archived) | “2026-09-15 two-disk mirrors”; `scripts/experiment-apparatus/perf-by-milestone.md` II.2 |
| 2026-09-15 single disk | Eight configurations, one disk each | 5 / 40 | `e152-file-system-benchmark-2026-09-15.out` (archived) | “Single disk” within “2026-09-15 two-disk mirrors”; `scripts/experiment-apparatus/perf-by-milestone.md` II |

### 2026-09-29: seven two-disk mirrors + singlefs

After the crash amplification full run the hosts were quiesced (inference and speech services on both machines stopped, both GPU cards idle); no other compilation or tests were running on this host. All 40 VM runs had `vm_exit=0`, with no retries. Host one-minute load was 0.50–9.61 (higher values occurred during OpenZFS rounds). The singlefs arm used the same commit as 2026-09-28 (`dbcc0d2f`). Its written and read bytes and requests were identical to 2026-09-28 (805,959,680 bytes and 1,598 writes per disk; cold recovery read 806,371,328 bytes and 196,874 requests per disk). The first-stage median changed 1,874 → 348 ms, while five cold-recovery rounds were 9,886–9,985 ms with no outlier, reflecting host conditions. Of 207 cells in the two tables, 13 changed by more than 15%; most were sequential writes marked unstable on both days (cell-by-cell comparison in `scripts/experiment-apparatus/perf-by-milestone.md` III.4).

The six dimension tables and singlefs table below are the first seven tables reproduced from `python3 scripts/experiment-apparatus/e152-tables.py e152-file-system-benchmark-mirror-2026-09-29.out`; the block-layer amplification and control tables are in the same output.

#### Large-file sequential writes and reads (MiB/s; higher is better)

| Configuration | Sequential write | Sequential read |
|---|---|---|
| Raw md raid1 (mirror ceiling) | 2 823 ⚠42% | 5 278 |
| ext4 (md raid1) | 1 524 ⚠166% | 5 272 |
| XFS (md raid1) | 2 091 ⚠120% | 5 299 |
| F2FS (md raid1) | 1 853 ⚠98% | 5 238 |
| Btrfs raid1 | 1 287 ⚠37% | 4 947 |
| Bcachefs two replicas | 1 736 ⚠95% | 4 836 |
| OpenZFS mirror (vdev is a partition) | 406 ⚠108% | 1 959 |
| singlefs (two disks) | — | — |

#### 4K random reads and writes (operations/s: higher is better; p99 completion latency in µs: lower is better)

| Configuration | Random read ops/s | Random read p99 | Random write ops/s | Random write p99 |
|---|---|---|---|---|
| Raw md raid1 (mirror ceiling) | 63 319 | 522 | 37 538 | 888 |
| ext4 (md raid1) | 59 637 | 552 | 33 444 ⚠23% | 1 056 ⚠28% |
| XFS (md raid1) | 59 653 | 552 | 34 434 ⚠35% | 1 036 ⚠100% |
| F2FS (md raid1) | 59 651 | 552 | 34 509 | 1 056 |
| Btrfs raid1 | 50 963 | 651 | 11 218 | 3 162 |
| Bcachefs two replicas | 34 775 | 1 204 | 30 667 ⚠58% | 1 613 ⚠148% |
| OpenZFS mirror (vdev is a partition) | 3 076 | 10 944 | 4 718 ⚠91% | 17 694 ⚠4394% |
| singlefs (two disks) | — | — | — | — |

#### Metadata / small files (operations/s; higher is better)

| Configuration | Create empty file | stat | Delete file | 4 KiB write + fsync |
|---|---|---|---|---|
| ext4 (md raid1) | 243 902 | 909 090 | 303 030 | 821 ⚠22% |
| XFS (md raid1) | 227 272 | 909 090 | 196 078 | 910 |
| F2FS (md raid1) | 188 679 | 909 090 | 256 410 | 1 220 |
| Btrfs raid1 | 238 095 | 909 090 | 131 578 | 691 ⚠23% |
| Bcachefs two replicas | 142 857 ⚠60% | 833 333 | 135 135 | 732 |
| OpenZFS mirror (vdev is a partition) | 126 582 | 666 666 | 88 495 | 1 640 ⚠60% |
| singlefs (two disks) | — | — | — | — |

#### Small-file persistence: create + write 3000 bytes + fsync file + fsync directory (100 times on a freshly formatted empty file system)

| Configuration | Median latency µs | p99 latency µs | Write requests/op | Bytes written/op | FLUSH/op | First operation bytes | First operation write requests | First operation FLUSH |
|---|---|---|---|---|---|---|---|---|
| ext4 (md raid1) | 1 157 | 1 284 ⚠15% | 8.00 | 72 581 | 4.00 | 73 728 | 8.00 | 4.00 |
| XFS (md raid1) | 1 079 | 1 271 | 6.00 | 19 927 | 4.00 | 13 312 | 6.00 | 4.00 |
| F2FS (md raid1) | 1 048 | 1 138 | 12.0 | 41 123 | 6.00 | 40 960 | 12.0 | 6.00 |
| Btrfs raid1 | 2 555 | 2 868 ⚠16% | 14.0 | 201 195 | 8.00 | 155 648 | 14.0 | 8.00 |
| Bcachefs two replicas | 1 616 | 2 437 ⚠21% | 6.30 | 14 602 | 4.00 | 36 864 | 14.0 | 4.00 |
| OpenZFS mirror (vdev is a partition) | 941 | 1 042 ⚠46% | 4.58 | 9 308 | 2.03 | 31 744 ⚠229% | 18.0 ⚠133% | 2.00 |
| singlefs (two disks) | 19 075 | — | 36.0 | 475 648 | 7.00 | — | — | — |

#### Cold mount (unmount, clear caches, remount)

| Configuration | Wall time ms | Bytes read | Read requests |
|---|---|---|---|
| ext4 (md raid1) | 1.38 | 21 504 | 6.00 |
| XFS (md raid1) | 8.07 ⚠61% | 4 235 776 | 45.0 |
| F2FS (md raid1) | 2.90 ⚠17% | 692 224 | 14.0 |
| Btrfs raid1 | 3.90 | 315 392 | 23.0 |
| Bcachefs two replicas | 28.5 | 7 718 912 ⚠17% | 57.0 |
| OpenZFS mirror (vdev is a partition) | 6 110 | 11 534 336 ⚠17% | 240 |
| singlefs (two disks) | 9 914 | 1 612 087 296 | 393 628 |

#### Formatting and usable capacity (16 GiB disks)

| Configuration | Format ms | Format bytes written | Format FLUSH | Total capacity % | Available % |
|---|---|---|---|---|---|
| ext4 (md raid1) | 90.4 ⚠31% | 143 197 184 | 14.0 | 48.8 | 46.3 |
| XFS (md raid1) | 82.9 | 135 160 832 | 10.0 | 49.8 | 48.7 |
| F2FS (md raid1) | 63.0 ⚠53% | 79 815 680 | 8.00 | 49.9 | 49.0 |
| Btrfs raid1 | 20.0 | 9 216 000 | 23.0 | 50.0 | 49.2 |
| Bcachefs two replicas | 28.4 ⚠53% | 32 768 | 2.00 | 92.0 | 90.6 |
| OpenZFS mirror (vdev is a partition) | 544 | 3 574 272 ⚠111% | 26.0 | 46.9 | 46.9 |
| singlefs (two disks) | — | — | — | — | — |

#### singlefs (two 16 GiB disks, checker-tier-qemu-device-streams real-device binary)

| Configuration | mkfs + number acquisition + warm-up + new-pool file creation ms | Overwrite, release, rollback and reuse (overwrite B) ms | Publication B (binary's own timing) ms | Cold recovery ms | Bytes read across both disks | Read requests across both disks | B write requests across both disks | B bytes written across both disks | B barriers across both disks | B FUA writes across both disks |
|---|---|---|---|---|---|---|---|---|---|---|
| singlefs (two disks) | 348 ⚠83% | 19.3 | 19.1 | 9 914 | 1 612 087 296 | 393 628 | 29.0 | 475 648 | 6.00 | 1.00 |

### 2026-09-28: seven two-disk mirrors + singlefs

Milestone two wrap-up step 8: all 40 VM runs returned `vm_exit=0`, without retries. The singlefs first-stage median was 1,874 ms (mkfs actually wrote the whole journal ring, 805,959,680 bytes per disk); publication B took a median 19.4 ms, with 29 writes and six barriers; cold recovery median was 10.0 s (round 3 had a 72.6 s outlier). The row-by-row singlefs comparison with 2026-09-17 is in `scripts/experiment-apparatus/perf-by-milestone.md` III.3.

The following six dimension tables and singlefs table are the first seven tables reproduced from `python3 scripts/experiment-apparatus/e152-tables.py e152-file-system-benchmark-mirror-2026-09-28.out`; block-layer amplification and control tables are in the same output.

#### Large-file sequential writes and reads (MiB/s; higher is better)

| Configuration | Sequential write | Sequential read |
|---|---|---|
| Raw md raid1 (mirror ceiling) | 2 819 | 5 265 |
| ext4 (md raid1) | 2 713 ⚠91% | 5 211 ⚠63% |
| XFS (md raid1) | 2 614 ⚠94% | 5 191 |
| F2FS (md raid1) | 2 707 ⚠89% | 5 265 |
| Btrfs raid1 | 1 375 ⚠94% | 4 905 |
| Bcachefs two replicas | 1 208 ⚠143% | 4 824 |
| OpenZFS mirror (vdev is a partition) | 421 ⚠110% | 1 934 |
| singlefs (two disks) | — | — |

#### 4K random reads and writes (operations/s: higher is better; p99 completion latency in µs: lower is better)

| Configuration | Random read ops/s | Random read p99 | Random write ops/s | Random write p99 |
|---|---|---|---|---|
| Raw md raid1 (mirror ceiling) | 64 564 | 610 | 37 037 ⚠26% | 897 |
| ext4 (md raid1) | 59 591 ⚠29% | 643 ⚠764% | 35 156 ⚠69% | 1 056 ⚠462% |
| XFS (md raid1) | 59 019 | 561 | 34 959 | 1 044 |
| F2FS (md raid1) | 60 200 | 643 | 35 301 | 1 056 |
| Btrfs raid1 | 50 472 | 659 | 11 127 ⚠20% | 3 194 |
| Bcachefs two replicas | 31 571 | 1 400 | 23 586 ⚠119% | 1 810 ⚠6430% |
| OpenZFS mirror (vdev is a partition) | 2 934 | 11 337 | 4 153 ⚠113% | 28 442 ⚠1013% |
| singlefs (two disks) | — | — | — | — |

#### Metadata / small files (operations/s; higher is better)

| Configuration | Create empty file | stat | Delete file | 4 KiB write + fsync |
|---|---|---|---|---|
| ext4 (md raid1) | 243 902 | 909 090 | 303 030 | 815 ⚠79% |
| XFS (md raid1) | 227 272 | 909 090 | 188 679 | 903 |
| F2FS (md raid1) | 185 185 ⚠18% | 909 090 | 256 410 | 1 212 |
| Btrfs raid1 | 238 095 | 909 090 | 128 205 | 680 ⚠66% |
| Bcachefs two replicas | 138 888 ⚠48% | 909 090 | 113 636 ⚠22% | 723 ⚠66% |
| OpenZFS mirror (vdev is a partition) | 128 205 | 666 666 | 87 719 | 1 472 ⚠92% |
| singlefs (two disks) | — | — | — | — |

#### Small-file persistence: create + write 3000 bytes + fsync file + fsync directory (100 times on a freshly formatted empty file system)

| Configuration | Median latency µs | p99 latency µs | Write requests/op | Bytes written/op | FLUSH/op | First operation bytes | First operation write requests | First operation FLUSH |
|---|---|---|---|---|---|---|---|---|
| ext4 (md raid1) | 1 152 | 1 309 ⚠21% | 8.00 | 72 581 | 4.00 | 73 728 | 8.00 | 4.00 |
| XFS (md raid1) | 1 072 | 1 226 | 6.00 | 19 927 | 4.00 | 13 312 | 6.00 | 4.00 |
| F2FS (md raid1) | 1 062 | 1 172 ⚠25% | 12.0 | 41 123 | 6.00 | 40 960 | 12.0 | 6.00 |
| Btrfs raid1 | 2 574 | 2 791 | 14.0 | 201 195 | 8.00 | 155 648 | 14.0 | 8.00 |
| Bcachefs two replicas | 1 688 | 2 402 | 6.30 | 14 632 | 4.00 | 36 864 | 14.0 | 4.00 |
| OpenZFS mirror (vdev is a partition) | 921 | 1 185 ⚠26% | 4.00 | 8 192 | 2.00 | 8 192 ⚠288% | 4.00 ⚠350% | 2.00 |
| singlefs (two disks) | 19 233 ⚠198% | — | 36.0 | 475 648 | 7.00 | — | — | — |

#### Cold mount (unmount, clear caches, remount)

| Configuration | Wall time ms | Bytes read | Read requests |
|---|---|---|---|
| ext4 (md raid1) | 1.39 | 21 504 | 6.00 |
| XFS (md raid1) | 10.5 ⚠46% | 4 235 776 | 45.0 |
| F2FS (md raid1) | 3.08 ⚠16% | 692 224 | 14.0 |
| Btrfs raid1 | 4.23 ⚠165% | 315 392 | 23.0 |
| Bcachefs two replicas | 31.2 | 7 718 912 ⚠31% | 57.0 ⚠16% |
| OpenZFS mirror (vdev is a partition) | 6 107 | 11 207 680 ⚠21% | 245 |
| singlefs (two disks) | 10 017 ⚠626% | 1 612 087 296 | 393 628 |

#### Formatting and usable capacity (16 GiB disks)

| Configuration | Format ms | Format bytes written | Format FLUSH | Total capacity % | Available % |
|---|---|---|---|---|---|
| ext4 (md raid1) | 90.0 ⚠22% | 143 197 184 | 14.0 | 48.8 | 46.3 |
| XFS (md raid1) | 86.7 | 135 160 832 | 10.0 | 49.8 | 48.7 |
| F2FS (md raid1) | 79.1 ⚠35% | 79 815 680 | 8.00 | 49.9 | 49.0 |
| Btrfs raid1 | 21.2 | 9 216 000 | 23.0 | 50.0 | 49.2 |
| Bcachefs two replicas | 40.9 ⚠31% | 32 768 | 2.00 | 92.0 | 90.6 |
| OpenZFS mirror (vdev is a partition) | 543 | 3 586 560 | 26.0 | 46.9 | 46.9 |
| singlefs (two disks) | — | — | — | — | — |

#### singlefs (two 16 GiB disks, checker-tier-qemu-device-streams real-device binary)

| Configuration | mkfs + number acquisition + warm-up + new-pool file creation ms | Overwrite, release, rollback and reuse (overwrite B) ms | Publication B (binary's own timing) ms | Cold recovery ms | Bytes read across both disks | Read requests across both disks | B write requests across both disks | B bytes written across both disks | B barriers across both disks | B FUA writes across both disks |
|---|---|---|---|---|---|---|---|---|---|---|
| singlefs (two disks) | 1 874 ⚠230% | 19.4 ⚠197% | 19.2 ⚠198% | 10 017 ⚠626% | 1 612 087 296 | 393 628 | 29.0 | 475 648 | 6.00 | 1.00 |

### 2026-09-19: singlefs arm after changing the apparatus to finish reading before forwarding (whole lines copied from artifacts; lines from artifacts moved out of the workspace written as one-line pointers)

Rerun preregistration `e152-r5-prereg.md`; answer only Q1–Q4 in `e152-r5-questions.md`, without `nice -n 19` (registration section 5: wall time is being measured). Five rounds succeeded first try (`vm_exit=0`); host one-minute load fell monotonically from 6.88 to 4.15. Artifact `e152-file-system-benchmark-second-transaction-relay-timing-2026-09-19.out` (142 lines, final `emitted=26`):


```text
E7RESULT name=summary configuration=singlefs metric=singlefs_file_overwrite_inner_milliseconds rounds=2 median=6.576 minimum=6.360 maximum=6.792 spread_percent=6.6 stability=stable values=4:6.792,5:6.360
E7RESULT name=summary configuration=singlefs metric=singlefs_file_overwrite_milliseconds rounds=2 median=6.629 minimum=6.409 maximum=6.850 spread_percent=6.7 stability=stable values=4:6.850,5:6.409
Artifact line: git show 0ec0be214cdc:research/results/e152-file-system-benchmark-second-transaction-relay-timing-2026-09-19.out line 139 (name=summary)
Artifact line: git show 0ec0be214cdc:research/results/e152-file-system-benchmark-second-transaction-relay-timing-2026-09-19.out line 140 (name=summary)
E7RESULT name=summary configuration=singlefs metric=singlefs_write_path_milliseconds rounds=5 median=19.071 minimum=18.642 maximum=19.867 spread_percent=6.4 stability=stable values=1:19.019,2:19.071,3:19.218,4:19.867,5:18.642
```

**Q1 (does the outer interval contain the inner interval?) cannot be judged by the “all five rounds have definite values” path, but can be judged by the “more than half uncertain” path.** In rounds 1–3 the `singlefs_timing` line itself was interrupted in the middle by the kernel's `[    8.423276] reboot: Power down` message at VM shutdown (once per round, with different times). Round 1, reproduced verbatim:

```text
E7RESULT name=singlefs_timing configuration=singlefs round=1 write_path_nanoseconds=19018861 file_overwrite_nanoseconds=6424304 recovery_nanoseconds=7797794145 file_overwrite_writes_both_devices=21 file_overwrite_written_bytes_both_devices=344576 file_overwrite_barriers_both_devices=4 file_overwrite_[    8.423276] reboot: Power down
```

The `file_overwrite_outer_contains_inner` field falls after the interruption and is absent (registration section 1 Q1 treats absence as uncertain). Rounds 4 and 5 have complete lines, both `true`. Thus 3/5 uncertain, 0 false, 2 true; with a majority uncertain, registration section 14 requires withholding the entire outer “overwrite, release, rollback and reuse ms” cell. Its summary has `rounds=2`, taking only rounds 4 and 5, as quoted above. A3 independently recalculates each round from the `at_nanoseconds` and `nanoseconds=` of the same-round `inner=file_overwrite` line: outer minus inner is 52,540, 65,020, 65,211, 57,621, and 48,330 ns, all positive. **The physical containment relation holds in all five rounds; the field is unreadable in rounds 1–3 because of truncation, not because containment failed.** This does not enter Q1's formal answer, which accepts only the original `singlefs_timing` field; it is shown to distinguish the cases.

**This truncation had not previously been recorded.** The section “do not timestamp in the output-forwarding loop” in `.claude/kb/vm-harness.md` addresses timestamp backlog from forwarding blocking. After the apparatus changed to read through EOF before forwarding, that source did disappear: neither `E152RUN` times nor `singlefs_inner` lines showed tens-of-milliseconds backlog (see Q4). This truncation has a different source: at VM shutdown (`reboot()`), the guest kernel writes `dmesg` to the same serial port and competes with the last user-space `println!` for the same tty, physically interleaving bytes. **Separating reading and forwarding neither removes nor controls this source.** It appeared in 3/5 rounds and not in 2/5; the sample is too small to establish a host-load relationship (loads 6.88/5.98/5.21 when it appeared and 4.64/4.15 when it did not; within a single monotonically declining run, this could simply coincide with later rounds having lower load).

**Q2 is insufficient for judgment:** the criterion requires `singlefs_file_overwrite_inner_milliseconds` summary `rounds=5`, but it has `rounds=2` because `file_overwrite_inner_nanoseconds` also falls after the same truncation. Only these two rounds can be reported: median 6.576 ms, spread 6.6% (stable), not representative of five rounds.

**Q3 is sufficient for judgment:** the question sheet requires only both `name=file_overwrite` and `name=recover_cold` lines, which come from raw subprocess output and are unaffected. In all five rounds c1 (segment sequence), c7 (root 1:4), c8 (`content_matches=true`), and c9 (eight fields per disk) match all ten rounds from the two 2026-09-17 formal runs verbatim. c3–c5 (21 write requests, 344576 bytes, four barriers) precede truncation in all five and likewise match. c6 (FUA write) is readable only in rounds 4 and 5, with value 1 matching old artifacts; rounds 1–3 lack the field. **c2 (operation count) is 23 in all five, not the anchor 21 quoted in registration section 12 and A-2 of section 6.** This is not new: both 2026-09-17 artifacts (`e152-file-system-benchmark-second-transaction-2026-09-17.out` and `…-final-2026-09-17.out`) also have 23 in every round, already explained under “What these numbers mean” as 21 writes plus two barrier counts, according to the binary's counting convention. `git log --since='2026-09-17' -- crates/singlefs-core/src/transaction.rs crates/singlefs-core/src/allocator.rs` shows one change (`cae5092`), but 23 was already present in the earlier 2026-09-17 wrap-up artifact, so it was not a regression from that change. The main agent must decide whether to amend the registered anchor 21 to reflect implementation 23.

**Q4 is sufficient, and all five rounds satisfy it:** d1 (span from the first `segments` to the `transaction` line, unaffected by truncation) is respectively 81,240, 78,121, 86,351, 86,810, and 80,301 ns, all ≤1 ms. d2 (line count) is 13 in all five, matching the new B-3 registered anchor: old output had eight lines, but the source added `instance_acquisition`/`post_mkfs_stream` and other segment lines plus two `device_calls` between runs; the registration anticipated and accepted 13. d3 (maximum adjacent interval) is 21,200–23,540 ns, consistent with the “24 µs” scale of the section 14 smoke test.

**P1 (recalculation of old artifacts, positive control):** applying the same checks to ten rounds of the two 2026-09-17 formal runs gives nine `file_overwrite_outer_contains_inner=false` rounds, reproducing the 9/10 already recorded in `.claude/kb/vm-harness.md` round by round; that record was correct. d1 >1 ms occurs in 8/10 rounds (threshold ≥6). Both checks thus have discriminating power on old artifacts; passing Q1 and Q4 in 5/5 and 5/5 rounds after the apparatus change is not simply because the checks lack power.

**P2 (put forwarding back inside the read loop, positive control) was infeasible: 0/6 attempts succeeded.** As registered in section 5, adding `println!("relay_control {text}")` inside the `read_lines_with_arrival_times` loop was expected not to be collected by `vm-bench.sh` because the line does not begin with E7RESULT. All 3 rounds × 2 attempts were invalidated by `scripts/devices/vm-bench.sh`'s integrity gate: “result count mismatch: program says 22 emitted, host captured 41” (`vm_exit=1`). Root cause: `grep -ao 'E7RESULT .*'` in `vm-bench.sh` is not anchored to the start of a line (its own comment says so); the subprocess line itself begins `E7RESULT `, so `relay_control E7RESULT name=... ` is still extracted from `E7RESULT` as a separate result and duplicates the same line later forwarded by the wrapper. The registration's assumption about collection differed from the implementation. Artifact `e152-file-system-benchmark-second-transaction-relay-in-loop-control-2026-09-19.out` (257 lines, all invalid) is retained; no e3/e4 data resulted. The injected line was restored and `sha256sum` verified against registration section 3.

**G1 (geometry sensitivity, `VM_CPUS=1`, three rounds) is unstable, consistent with the registered opposite-direction expectation.** Artifact `e152-file-system-benchmark-second-transaction-relay-timing-vcpu1-2026-09-19.out` (83 lines, final `emitted=13`):

```text
E7RESULT name=summary configuration=singlefs metric=singlefs_file_overwrite_inner_milliseconds rounds=3 median=6.260 minimum=6.132 maximum=6.643 spread_percent=8.2 stability=stable values=1:6.132,2:6.260,3:6.643
E7RESULT name=summary configuration=singlefs metric=singlefs_file_overwrite_milliseconds rounds=1 median=6.302 minimum=6.302 maximum=6.302 spread_percent=0.0 stability=stable values=2:6.302
```

Across the three rounds `file_overwrite_outer_contains_inner` is false, true, false. In rounds 1 and 3 `file_overwrite_nanoseconds` is 636,471 and 458,921 ns less than `file_overwrite_inner_nanoseconds`, respectively. With one vCPU the host process reading subprocess output competes with the subprocess for the same core; even when reading finishes before forwarding, scheduling delays can invert the arrival times of `transaction` and `file_overwrite` lines, as happened in 2/3 rounds. d1 spans are 726,152, 70,590 and 559,782 ns, all still ≤1 ms (an order of magnitude larger than at four vCPUs but below threshold). Per registration section 8, **A1 is unstable at both geometries:** four vCPUs produced only 2/5 definite rounds because of truncation, though all independently recalculated true; one vCPU produced 2/3 definitely false. The Q1 statement “outer contains inner after separating reading and forwarding” must be limited to “one source, forwarding blocking, was eliminated,” not generalized to every geometry. d1 triggered at neither geometry (≤1 ms); Q4 therefore did not distinguish triggering from non-triggering at these two points, and its power under more extreme scheduling delays cannot yet be judged.

Under preregistration section 11's “stop when sufficient to judge,” Q1, Q3 and Q4 were sufficient at the return of segment one (Q1 via the majority-uncertain path); Q2 was insufficient (`rounds=2 < 5`). Segment two's P2 and G1 were both run: P2 produced no data because the method was infeasible; G1 produced data confirming instability.

### 2026-09-17 wrap-up: rerun the singlefs arm at completion of “overwrite, release, rollback and reuse”

Whole artifact lines are copied; lines from artifacts moved out of the workspace are one-line pointers. The user requested another run at milestone wrap-up. Since step 7, `crates/` had four changes, individually listed in preregistration section 13; workload and criteria were unchanged. Five rounds succeeded first try, host one-minute load 2.27–2.45, while the checker-tier-crash-point-replay full crash-point replay was running concurrently at `nice -n 19`. Artifact `e152-file-system-benchmark-second-transaction-final-2026-09-17.out` (91 lines, final `emitted=10`).

```text
Artifact line: git show 0ec0be214cdc:research/results/e152-file-system-benchmark-second-transaction-final-2026-09-17.out line 82 (name=summary)
Artifact line: git show 0ec0be214cdc:research/results/e152-file-system-benchmark-second-transaction-final-2026-09-17.out line 83 (name=summary)
Artifact line: git show 0ec0be214cdc:research/results/e152-file-system-benchmark-second-transaction-final-2026-09-17.out line 84 (name=summary)
Artifact line: git show 0ec0be214cdc:research/results/e152-file-system-benchmark-second-transaction-relay-timing-2026-09-19.out line 135 (name=summary)
Artifact line: git show 0ec0be214cdc:research/results/e152-file-system-benchmark-second-transaction-final-2026-09-17.out line 86 (name=summary)
Artifact line: git show 0ec0be214cdc:research/results/e152-file-system-benchmark-second-transaction-final-2026-09-17.out line 87 (name=summary)
Artifact line: git show 0ec0be214cdc:research/results/e152-file-system-benchmark-second-transaction-relay-timing-2026-09-19.out line 139 (name=summary)
Artifact line: git show 0ec0be214cdc:research/results/e152-file-system-benchmark-second-transaction-relay-timing-2026-09-19.out line 140 (name=summary)
Artifact line: git show 0ec0be214cdc:research/results/e152-file-system-benchmark-second-transaction-final-2026-09-17.out line 90 (name=summary)
```

The preregistered section 13 criteria hold individually: in all five rounds B's segment sequence `16+2+1+2`, 21 write requests and 344,576 written bytes across both disks, four barriers, and one FUA match the third run verbatim; all five cold recoveries have `content_matches=true` and root 1:4. Wall times for the two runs are shown side by side in `scripts/experiment-apparatus/perf-by-milestone.md` III.2, “rerun at wrap-up,” without ranking their speed.

**Step 7: singlefs arm for the “overwrite, release, rollback and reuse” milestone** (whole artifact lines copied; lines from artifacts moved out of the workspace are one-line pointers).

Only singlefs ran: two 16 GiB disks in `file-overwrite` mode, overwriting once in the same process after creating a file in a new pool. All five rounds succeeded first try, host one-minute load 1.28–1.69. Another full gate was concurrently running the checker-tier-research-build-and-replay experiment-replay cell on the same machine; preregistration section 7 specified running and recording it anyway. Configuration and criteria are in preregistration section 12; the six baselines were not rerun. Artifact `e152-file-system-benchmark-second-transaction-2026-09-17.out` (91 lines, final `emitted=10`).

```text
Artifact line: git show 0ec0be214cdc:research/results/e152-file-system-benchmark-second-transaction-final-2026-09-17.out line 82 (name=summary)
Artifact line: git show 0ec0be214cdc:research/results/e152-file-system-benchmark-second-transaction-final-2026-09-17.out line 83 (name=summary)
Artifact line: git show 0ec0be214cdc:research/results/e152-file-system-benchmark-second-transaction-2026-09-17.out line 84 (name=summary)
Artifact line: git show 0ec0be214cdc:research/results/e152-file-system-benchmark-second-transaction-relay-timing-2026-09-19.out line 135 (name=summary)
Artifact line: git show 0ec0be214cdc:research/results/e152-file-system-benchmark-second-transaction-final-2026-09-17.out line 86 (name=summary)
Artifact line: git show 0ec0be214cdc:research/results/e152-file-system-benchmark-second-transaction-2026-09-17.out line 87 (name=summary)
Artifact line: git show 0ec0be214cdc:research/results/e152-file-system-benchmark-second-transaction-relay-timing-2026-09-19.out line 139 (name=summary)
Artifact line: git show 0ec0be214cdc:research/results/e152-file-system-benchmark-second-transaction-relay-timing-2026-09-19.out line 140 (name=summary)
Artifact line: git show 0ec0be214cdc:research/results/e152-file-system-benchmark-second-transaction-2026-09-17.out line 90 (name=summary)
```


The preregistered section 12 criteria hold: all five B segment sequences are `16+2+1+2`, operation count 23 (21 writes plus two barrier counts; see the binary for the convention), 21 write requests and 344,576 bytes across both disks, four barriers, and one FUA, matching the registered values. All five cold recoveries have `content_matches=true` and root 1:4. Wall-time spreads are 186% for overwrite/free/rollback/reuse and 156% for the first stage; registration specified reporting them without removing them. Interpretation is in “What these numbers mean (overwrite, release, rollback and reuse, compared with the six two-disk mirrors),” with a reader-facing version in `scripts/experiment-apparatus/perf-by-milestone.md` III.2.

### What these numbers mean (overwrite, release, rollback and reuse, compared with the six two-disk mirrors)

Written 2026-09-17. The singlefs cell uses publication B from the two 2026-09-17 formal runs (one overwrite of an existing file in the same process, second version 4100 bytes). The six competitors use the 2026-09-15 formal two-disk-mirror run: 100 repetitions of “create + write 3000 bytes + fsync file + fsync directory” on a freshly formatted empty file system. The six were not rerun. This section contains observations and arithmetic only; how to narrow a gap is a design inference not subjected to three-way argument.

**1. Ranking the seven systems.** For the six, bytes written, write requests and FLUSH are averages of 100 operations, and wall time is their median. For singlefs the counts are identical in all ten rounds of the two runs; wall-time selection is explained in item 4.

| Dimension | singlefs publication B | Six mirrors: lightest or fastest … heaviest or slowest | singlefs among seven | Multiple of lightest/fastest | Multiple of heaviest/slowest |
|---|---|---|---|---|---|
| Bytes written per operation, both disks | 344 576 | OpenZFS mirror 8 192 … Btrfs raid1 201 195 | Most | 42.1 | 1.71 |
| Write requests per operation, both disks, guest block-layer convention | 26 (binary records 21 write calls; item 5) | OpenZFS mirror 4.00 … Btrfs raid1 14.0 | Most | 6.5 | 1.86 |
| FLUSH per operation, both disks | 5 | OpenZFS mirror 2.00 … Btrfs raid1 8.00 | Middle: fewer than F2FS (md raid1) 6.00 and Btrfs raid1, more than the other four | 2.5 | 0.63 |
| Wall time per operation | 2026-09-17 wrap-up median 6.63 ms over five rounds (spread 11.8%); both runs' ten values 5.95–12.61 ms | OpenZFS mirror 850 µs … Btrfs raid1 2 199 µs | Slowest: fastest of ten, 5.95 ms, is 2.3 times the slowest p99 among the six (Bcachefs two replicas, 2 559 µs) | 7.8 | 3.0 |
| Cold-mount wall time | 7 893 / 7 738 ms (2026-09-17 step 7 / wrap-up medians) | ext4 (md raid1) 1.66 ms … OpenZFS mirror 6 125 ms | Slowest | About 4 700 | 1.26–1.29 |
| Cold-mount bytes read, both disks | 1 610 981 376 | ext4 (md raid1) 21 504 … OpenZFS mirror 11 878 400 | Most | About 75 000 | 136 |
| Dimensions runnable | Three of eight: small-file persistence, cold mount, formatting and capacity | Eight | — | — | — |

**2. Where the gaps are, and how large (judgment based on item 1):**

- Small-file persistence first has a steady-state measurement at the “overwrite, release, rollback and reuse” milestone. Written bytes, write requests and wall time are the most or slowest of seven, worse even than the heaviest or slowest competitor (1.71, 1.86 and 3.0 times); FLUSH is in the middle, not the source of the gap.
- The high write volume is not the cost of “all trees hitting disk for the first time”: publication B and new-pool file creation match item by item in bytes, write calls, barriers and FUA (both segment sequences are 16+2+1+2). The per-persistence cost is independent of whether it is the first.
- Cold mount is the largest gap in absolute terms: slowest of seven, seconds. Of 1 610 981 376 bytes read across both disks, `scan_journal` reads 1 610 612 736 (99.98%) by scanning the entire ring slot by slot; that part depends only on ring size. Reads outside the ring depend on disk contents: overwrite/free/rollback/reuse reads five more requests and 49 152 more bytes than new-pool file creation. This decomposition comes from code: the block layer cannot identify which requests land on the ring, and bytes read do not equal read requests × 4 KiB (393 302 × 4 096 is 16 384 bytes less than measured). Amortizing cold-recovery wall time by request count, including time spent outside reads, singlefs has 393 302 requests at 19.7–20.1 µs each; OpenZFS mirror reads only 260 times at 23.6 ms each. The two slowest have opposite profiles: many reads versus few reads with long amortized time.
- Btrfs, Bcachefs and OpenZFS are also copy-on-write (external fact, not verified in this project). On the same bench the latter two write only 14 592 and 8 192 bytes per operation. This supports the inference that copy-on-write itself does not require hundreds of thousands of bytes per fsync; this inference has not undergone three-way argument. E152 had not measured what the 8–15 KiB writes contain as of 2026-09-17.

**3. Where the gap comes from (arithmetic):** publication B replaces eight units of the first file version (the “freed allocation records” row in [layout/02-second-txn.md](../layout/02-second-txn.md)). Its eight new units have the same kinds and widths as the eight in new-pool file creation (unit table in [layout/01-first-txn.md](../layout/01-first-txn.md)): data unit, extent root/leaf, inode leaf container, inode root, allocation record tree root/leaf, accounting root/leaf, central mapping root/leaf, and tree table unit, one copy on each disk.

| Part | Bytes across both disks | Share of publication B | Compared individually to one whole operation of the six |
|---|---|---|---|
| One data unit (4100 bytes of content; data unit always occupies 32 768 on disk including header, D4 checksum location settled item 5) | 65 536 | 19.0% | Already more than whole operations of OpenZFS mirror, Bcachefs two replicas, XFS (md raid1), and F2FS (md raid1): 8.0× OpenZFS and 3.3× XFS |
| Seven metadata units (inode leaf container 32 768, six others 16 384 each) | 262 144 | 76.1% | More than even the heaviest of the six, Btrfs raid1, at 1.30× |
| Two journal records, one root slot, two system configuration slots | 16 896 | 4.9% | Less than XFS's whole-operation 19 927; not the source of the gap |

- Either component alone constitutes a gap: the data unit alone exceeds four competitors' whole operations; metadata units alone exceed all six.
- Likewise for requests: eight units × two disks = 16 writes, already exceeding the highest competitor, Btrfs raid1 at 14.0 per operation.
- In the two 2026-09-17 runs each tree had one node; each of the seven trees touched by one overwrite rewrote one unit. The runs cannot measure per-write volume after trees grow taller and files multiply.

**4. Timing measurement basis: the summary's overwrite/free/rollback/reuse cell is not the time of publication B** (an apparatus issue, not a design or data issue).

- Summary `singlefs_file_overwrite_milliseconds` (median 5.078 for step 7, 5.529 at 2026-09-17 wrap-up) takes the difference between outer-wrapper timestamps on `name=transaction` and `name=file_overwrite`, as registered in section 12 item 2. The wrapper `run_singlefs` in `scripts/experiment-apparatus/e152_file_system_benchmark.rs` forwards each subprocess line to its own stdout before reading and timestamping the next; the subprocess is not blocked when writing to the pipe. Thus each line's timestamp includes printing backlog from previous lines.
- The binary prints `name=transaction`, starts its own timer, performs publication B, stops timing, then prints `name=file_overwrite`. The actual interval between the outer lines must contain the binary's own timed interval, yet in nine of ten rounds across the two runs the outer value was smaller.
- Backlog magnitude: the binary prints `segments`, `device_calls`, `transaction` consecutively, with only microseconds of in-memory computation between them. In wrap-up round 2 these timestamps spanned 21.7 ms (the following six lines copied in full from `e152-file-system-benchmark-second-transaction-final-2026-09-17.out`: `inner=segments path=mkfs`, two `inner=device_calls`, `inner=transaction`, `inner=file_overwrite`, and `name=singlefs_timing`):

```text
E7RESULT name=singlefs_inner configuration=singlefs round=2 at_nanoseconds=19555763 inner=segments path=mkfs operations=13 segments=4+1+1+1+4 closed_form=34 kinds=[unit_write×4,barrier]|[root_record_fua]|[root_record_fua]|[root_record_fua]|[system_configuration_slot×4,barrier]
E7RESULT name=singlefs_inner configuration=singlefs round=2 at_nanoseconds=35650082 inner=device_calls device=0 path=/dev/vda writes=23 written_bytes=251904 force_unit_access_writes=4 barriers=8 skipped_barriers=0 write_ios=35 write_sectors=492 flush_ios=12
E7RESULT name=singlefs_inner configuration=singlefs round=2 at_nanoseconds=38414092 inner=device_calls device=1 path=/dev/vdb writes=21 written_bytes=250880 force_unit_access_writes=2 barriers=8 skipped_barriers=0 write_ios=31 write_sectors=490 flush_ios=10
E7RESULT name=singlefs_inner configuration=singlefs round=2 at_nanoseconds=41239814 inner=transaction policy_mismatches=0 key_order_mismatches=0 root_txg=3 back_chain=628216162
E7RESULT name=singlefs_inner configuration=singlefs round=2 at_nanoseconds=43231321 inner=file_overwrite root_txg=4 transaction=2 released=8 nanoseconds=6608505 operations=23 segments=16+2+1+2 closed_form=65543 device_0_writes=10 device_0_written_bytes=172032 device_0_force_unit_access_writes=0 device_0_barriers=2 device_1_writes=11 device_1_written_bytes=172544 device_1_force_unit_access_writes=1 device_1_barriers=2
E7RESULT name=singlefs_timing configuration=singlefs round=2 write_path_nanoseconds=41040214 file_overwrite_nanoseconds=1991507 recovery_nanoseconds=7738420894 file_overwrite_writes_both_devices=21 file_overwrite_written_bytes_both_devices=344576 file_overwrite_barriers_both_devices=4 file_overwrite_force_unit_access_writes_both_devices=1 child_exit=0 content_matches=true device_0_read_bytes=805609472 device_0_read_requests=196664 device_0_write_bytes=423936 device_0_write_requests=47 device_0_flush_requests=14 device_1_read_bytes=805371904 device_1_read_requests=196638 device_1_write_bytes=423424 device_1_write_requests=45 device_1_flush_requests=13
```

- The same backlog explains the first-stage wall time's “two clusters”: across 20 rounds of four runs, the span from first `segments` to `transaction` (only printing between them) was 0.71–25.83 ms. In the five roughly 40 ms rounds (2026-09-15 single-disk rounds 1 and 3, 2026-09-17 step 7 round 2, wrap-up rounds 2 and 3), this span was 21.68–25.83 ms. Cold recovery took 7.7–7.9 s, so backlog of this magnitude affects it by less than 0.4%.
- Item 1 therefore uses the binary's own `nanoseconds=` on `name=file_overwrite` (a field preregistered in section 12 item 1): step 7 values 12.611 / 8.576 / 5.948 / 6.135 / 6.093 ms (median 6.135, spread 109%); wrap-up 6.369 / 6.609 / 6.634 / 7.153 / 6.643 ms (median 6.634, spread 11.8%). ⚠️ This selection changed after seeing the artifact, because the outer value was physically impossible, not because it was undesirable. The binary's own timing has only this one measurement path and no independent verification.
- The apparatus was changed (preregistration section 14): the guest program reads subprocess output through EOF before forwarding, and `singlefs_timing` additionally reports the subprocess's own nanoseconds `file_overwrite_inner_nanoseconds` and containment check `file_overwrite_outer_contains_inner`. Rounds where outer does not contain inner are reported as `summary_excluded` and excluded from the median; `singlefs_file_overwrite_inner_milliseconds` was added. A one-round smoke test had nine lines spanning 24 µs, with outer 6 632 915 ns containing inner 6 575 144 ns. The four existing artifacts are not recalculated; how to avoid this pitfall and the enforcing checks are in `.claude/kb/vm-harness.md`, “do not timestamp in the output-forwarding loop.”

**5. Write-request measurement basis: the six use block-layer counts; the singlefs summary uses the binary's write-call count.**

- The six systems' “write requests per operation” are differences in the guest `/sys/block/*/stat` write-request field before and after each operation. Summary `singlefs_file_overwrite_writes_both_devices` counts binary write calls (preregistration section 12 item 1: program counter, not independently measured). The measurement bases differ.
- Block-layer differences are identical in ten rounds of the two 2026-09-17 runs: total-path writes on disks 0 / 1 are 47 / 45 (the `device_*_write_requests=` fields in the `singlefs_timing` line above). After new-pool file creation and before publication B, the binary reads block-layer counts 35 / 31 (`write_ios=` in the two `device_calls` lines above). Cold recovery does not write: the 2026-09-15 two-disk-mirror formal run's direct mode has no publication B, and its total block-layer counts are 35 / 31. Publication B is therefore 12 / 14 block-layer write requests, 172 032 / 172 544 bytes, and 2 / 3 FLUSH. Bytes and FLUSH match the binary's counts item by item, while write requests exceed its 10 / 11 by 2 / 3.
- The extras exactly equal barriers plus FUA writes in all four per-disk readings: publication B disk 0 has two barriers (10 + 2 = 12), disk 1 has two barriers plus one FUA (11 + 3 = 14); before publication B disk 0 has eight barriers plus four FUA (23 + 12 = 35), disk 1 eight barriers plus two FUA (21 + 10 = 31). Why the kernel counts this way was not investigated. To compare like with like, singlefs uses the block-layer count 26.
- New-pool/new-file has the same segment shape as publication B, also 26 under this convention. The multiples in `scripts/experiment-apparatus/perf-by-milestone.md` section III that compare 21 with the six are calculated using 26.

**6. Workload differences**

| Difference | Effect on comparison |
|---|---|
| The six create a file and fsync its directory; singlefs overwrites an existing file and has no directory | Singlefs creation (new-pool file creation, 3000 bytes) and overwrite (publication B, 4100 bytes) match item by item in bytes, write calls, barriers and FUA. Switching to creation leaves the gap unchanged; the six systems' single-overwrite volume was not measured. |
| The six use averages or medians of 100 operations including the first; singlefs has one per round | Approximation from summary medians shows that excluding the first changes bytes per operation for the six by −1.5% to +0.3%. |
| The six are kernel file systems, timed from create through directory fsync return; singlefs is a user-space library using O_DIRECT writes and fdatasync on raw disks, timed only over publication B without allocator reconstruction from allocation records | Both cover “initiation to durable return”; singlefs's 6.6 ms was not split between encoding/checksumming, writing, and flushing. |

### 2026-09-15 two-disk mirrors: seven mirrored configurations + singlefs

The six dimension tables and singlefs table below are the first seven reproduced from `python3 scripts/experiment-apparatus/e152-tables.py research/results/e152-file-system-benchmark-mirror-2026-09-15.out`; the block-layer amplification and control tables are in the same output. The artifact is archived; retrieve it from repository revision `3cff909b^`.

#### Large-file sequential writes and reads (MiB/s; higher is better)

| Configuration | Sequential write | Sequential read |
|---|---|---|
| Raw md raid1 (mirror ceiling) | 2 707 ⚠18% | 5 404 |
| ext4 (md raid1) | 2 725 | 5 404 |
| XFS (md raid1) | 2 684 | 5 368 |
| F2FS (md raid1) | 2 718 ⚠44% | 5 389 |
| Btrfs raid1 | 1 280 ⚠30% | 5 044 |
| Bcachefs two replicas | 2 547 ⚠38% | 4 905 |
| OpenZFS mirror (vdev is a partition) | 500 ⚠41% | 1 982 |
| singlefs (two disks) | — | — |

#### 4K random reads and writes (operations/s: higher is better; p99 completion latency in µs: lower is better)

| Configuration | Random read ops/s | Random read p99 | Random write ops/s | Random write p99 |
|---|---|---|---|---|
| Raw md raid1 (mirror ceiling) | 57 893 | 626 | 33 019 ⚠44% | 1 028 ⚠16% |
| ext4 (md raid1) | 53 276 | 659 | 34 441 ⚠34% | 1 368 ⚠16% |
| XFS (md raid1) | 53 356 | 634 ⚠16% | 33 253 ⚠20% | 1 269 ⚠30% |
| F2FS (md raid1) | 55 985 | 593 ⚠33% | 31 928 ⚠23% | 1 089 ⚠33% |
| Btrfs raid1 | 48 047 | 823 ⚠25% | 10 948 ⚠25% | 3 686 |
| Bcachefs two replicas | 31 599 | 1 286 | 33 924 | 1 843 ⚠16% |
| OpenZFS mirror (vdev is a partition) | 3 122 | 10 682 | 5 237 ⚠22% | 15 925 ⚠77% |
| singlefs (two disks) | — | — | — | — |

#### Metadata / small files (operations/s; higher is better)

| Configuration | Create empty file | stat | Delete file | 4 KiB write + fsync |
|---|---|---|---|---|
| ext4 (md raid1) | 208 333 ⚠17% | 833 333 ⚠49% | 263 157 ⚠19% | 1 002 ⚠23% |
| XFS (md raid1) | 217 391 | 909 090 ⚠69% | 185 185 | 1 061 |
| F2FS (md raid1) | 172 413 | 909 090 ⚠88% | 227 272 ⚠21% | 1 365 |
| Btrfs raid1 | 222 222 ⚠15% | 833 333 ⚠75% | 120 481 | 809 |
| Bcachefs two replicas | 125 000 ⚠32% | 714 285 ⚠101% | 113 636 ⚠18% | 815 |
| OpenZFS mirror (vdev is a partition) | 111 111 ⚠22% | 666 666 ⚠48% | 77 519 ⚠22% | 1 719 ⚠38% |
| singlefs (two disks) | — | — | — | — |

#### Small-file persistence: create + write 3000 bytes + fsync file + fsync directory (100 times on a freshly formatted empty file system)

| Configuration | Median latency µs | p99 latency µs | Write requests/op | Bytes written/op | FLUSH/op | First operation bytes | First operation write requests | First operation FLUSH |
|---|---|---|---|---|---|---|---|---|
| ext4 (md raid1) | 980 | 1 120 | 8.00 | 72 581 | 4.00 | 73 728 | 8.00 | 4.00 |
| XFS (md raid1) | 858 | 1 087 ⚠19% | 6.00 | 19 927 | 4.00 | 13 312 | 6.00 | 4.00 |
| F2FS (md raid1) | 928 | 1 418 ⚠31% | 12.0 | 41 123 | 6.00 | 40 960 | 12.0 | 6.00 |
| Btrfs raid1 | 2 199 | 2 531 | 14.0 | 201 195 | 8.00 | 155 648 | 14.0 | 8.00 |
| Bcachefs two replicas | 1 520 | 2 559 ⚠26% | 6.30 | 14 592 | 4.00 | 36 864 | 14.0 | 4.00 |
| OpenZFS mirror (vdev is a partition) | 850 | 1 121 ⚠29% | 4.00 | 8 192 | 2.00 | 8 192 ⚠288% | 4.00 ⚠350% | 2.00 |
| singlefs (two disks) | — | — | — | — | — | — | — | — |

#### Cold mount (unmount, clear caches, remount)

| Configuration | Wall time ms | Bytes read | Read requests |
|---|---|---|---|
| ext4 (md raid1) | 1.66 ⚠22% | 21 504 | 6.00 |
| XFS (md raid1) | 7.29 ⚠42% | 4 235 776 | 45.0 |
| F2FS (md raid1) | 3.02 | 692 224 | 14.0 |
| Btrfs raid1 | 4.21 ⚠24% | 315 392 | 23.0 |
| Bcachefs two replicas | 29.7 | 7 456 768 | 56.0 |
| OpenZFS mirror (vdev is a partition) | 6 125 | 11 878 400 | 260 ⚠18% |
| singlefs (two disks) | 8 293 | 1 610 932 224 | 393 297 |

#### Formatting and usable capacity (16 GiB disks)

| Configuration | Format ms | Format bytes written | Format FLUSH | Total capacity % | Available % |
|---|---|---|---|---|---|
| ext4 (md raid1) | 79.3 ⚠29% | 143 197 184 | 14.0 | 48.8 | 46.3 |
| XFS (md raid1) | 81.0 ⚠41% | 135 160 832 | 10.0 | 49.8 | 48.7 |
| F2FS (md raid1) | 62.0 ⚠47% | 79 815 680 | 8.00 | 49.9 | 49.0 |
| Btrfs raid1 | 18.2 | 9 216 000 | 23.0 | 50.0 | 49.2 |
| Bcachefs two replicas | 23.4 ⚠43% | 32 768 | 2.00 | 92.0 | 90.6 |
| OpenZFS mirror (vdev is a partition) | 546 | 3 586 048 | 26.0 | 46.9 | 46.9 |
| singlefs (two disks) | — | — | — | — | — |

#### singlefs (two 16 GiB disks, checker-tier-qemu-device-streams real-device binary)

| Configuration | mkfs + number acquisition + warm-up + new-pool file creation ms | Overwrite, release, rollback and reuse (overwrite B) ms | Publication B (binary's own timing) ms | Cold recovery ms | Bytes read across both disks | Read requests across both disks | B write requests across both disks | B bytes written across both disks | B barriers across both disks | B FUA writes across both disks |
|---|---|---|---|---|---|---|---|---|---|---|
| singlefs (two disks) | 17.3 | — | — | 8 293 | 1 610 932 224 | 393 297 | — | — | — | — |

Whole lines are copied from artifacts; lines whose artifacts were moved out of the workspace are represented by one-line pointers.

Starting 2026-09-15, seven mirror configurations plus singlefs each used two 16 GiB disks, interleaved across five rounds (configuration and conventions are in preregistration section 11). One of 41 VM runs failed because another session changed `vm-bench.sh` during the run; a retry with the same parameters succeeded:

```text
E152RUN configuration=ext4-md round=1 attempt=1 host_load1=2.26 vm_exit=2
```

Every row for the seven configurations with controls has `rounds_with_teeth=5`; all 40 `software_raid_state` lines for the four md configurations have `synchronizing=false`:

```text
Artifact line: git show 0ec0be214cdc:research/results/e152-file-system-benchmark-mirror-2026-09-15.out line 768 (name=summary_control)
Artifact line: git show 0ec0be214cdc:research/results/e152-file-system-benchmark-mirror-2026-09-15.out line 769 (name=summary_control)
Artifact line: git show 0ec0be214cdc:research/results/e152-file-system-benchmark-mirror-2026-09-15.out line 770 (name=summary_control)
Artifact line: git show 0ec0be214cdc:research/results/e152-file-system-benchmark-mirror-2026-09-15.out line 771 (name=summary_control)
Artifact line: git show 0ec0be214cdc:research/results/e152-file-system-benchmark-mirror-2026-09-15.out line 772 (name=summary_control)
Artifact line: git show 0ec0be214cdc:research/results/e152-file-system-benchmark-mirror-2026-09-15.out line 773 (name=summary_control)
Artifact line: git show 0ec0be214cdc:research/results/e152-file-system-benchmark-mirror-2026-09-15.out line 774 (name=summary_control)
```

```text
Artifact line: git show 0ec0be214cdc:research/results/e152-file-system-benchmark-mirror-2026-09-15.out line 887 (name=summary)
Artifact line: git show 0ec0be214cdc:research/results/e152-file-system-benchmark-mirror-2026-09-15.out line 888 (name=summary)
Artifact line: git show 0ec0be214cdc:research/results/e152-file-system-benchmark-mirror-2026-09-15.out line 917 (name=summary)
Artifact line: git show 0ec0be214cdc:research/results/e152-file-system-benchmark-mirror-2026-09-15.out line 829 (name=summary)
Artifact line: git show 0ec0be214cdc:research/results/e152-file-system-benchmark-mirror-2026-09-15.out line 901 (name=summary)
Artifact line: git show 0ec0be214cdc:research/results/e152-file-system-benchmark-mirror-2026-09-15.out line 813 (name=summary)
Artifact line: git show 0ec0be214cdc:research/results/e152-file-system-benchmark-mirror-2026-09-15.out line 927 (name=summary)
Artifact line: git show 0ec0be214cdc:research/results/e152-file-system-benchmark-mirror-2026-09-15.out line 776 (name=summary)
Artifact line: git show 0ec0be214cdc:research/results/e152-file-system-benchmark-mirror-2026-09-15.out line 775 (name=summary)
Artifact line: git show 0ec0be214cdc:research/results/e152-file-system-benchmark-mirror-2026-09-15.out line 941 (name=summary_done)
```

Two Bcachefs cells from the single-disk run (used to compare capacity readings under mirroring), copied from `e152-file-system-benchmark-2026-09-15.out`：

```text
Artifact line: git show 0ec0be214cdc:research/results/e152-file-system-benchmark-2026-09-15.out line 725 (name=summary)
Artifact line: git show 0ec0be214cdc:research/results/e152-file-system-benchmark-2026-09-15.out line 724 (name=summary)
```


**Two-disk mirrors: what the numbers mean**

1. **Comparing matching configurations, cold mount remains the largest gap:** singlefs cold recovery 8 293 ms, OpenZFS mirror import 6 125 ms (singlefs 1.35×), ext4 (md raid1) 1.66 ms. Singlefs bytes read on two disks match the single-disk run verbatim, still a full-ring scan.
2. **Small-file persistence:** singlefs new-pool file creation writes 344 576 bytes across both disks, 25.9× XFS's (md raid1) first operation at 13 312 and 2.2× Btrfs raid1's first at 155 648. OpenZFS mirror's cell has 288% spread over five rounds and is not compared.
3. **Bcachefs two-replica capacity cannot be compared:** statfs reports raw capacity of both disks, without halving for two copies. Mirrored total capacity 92.0% and available 90.6% exactly match the single-disk run.
4. Mirrored write amplification near 2.00 in the two write columns is mirroring itself; OpenZFS mirror's 4K random-write amplification is 66.45. Per-cell tables are in `scripts/experiment-apparatus/perf-by-milestone.md` II.2.

**Single disk: eight configurations with one disk each**

The six dimension tables and singlefs table below are the first seven reproduced from `python3 scripts/experiment-apparatus/e152-tables.py research/results/e152-file-system-benchmark-2026-09-15.out`; the block-layer amplification and control tables are in the same output. The artifact is archived; retrieve it from repository revision `3cff909b^`.

#### Large-file sequential writes and reads (MiB/s; higher is better)

| Configuration | Sequential write | Sequential read |
|---|---|---|
| Raw disk (ceiling) | 5 498 ⚠26% | 5 911 |
| ext4 | 5 505 | 5 894 |
| XFS | 5 573 | 5 902 |
| F2FS | 5 491 | 5 894 |
| Btrfs | 1 720 | 5 411 |
| Bcachefs | 5 211 | 5 793 |
| OpenZFS (vdev is a partition) | 736 | 2 216 |
| singlefs (two disks) | — | — |

#### 4K random reads and writes (operations/s: higher is better; p99 completion latency in µs: lower is better)

| Configuration | Random read ops/s | Random read p99 | Random write ops/s | Random write p99 |
|---|---|---|---|---|
| Raw disk (ceiling) | 71 052 | 528 ⚠16% | 71 488 | 485 ⚠21% |
| ext4 | 64 626 | 518 ⚠21% | 52 739 | 684 ⚠23% |
| XFS | 64 742 ⚠15% | 518 ⚠22% | 52 324 | 675 ⚠26% |
| F2FS | 65 139 | 514 ⚠20% | 52 970 | 675 ⚠24% |
| Btrfs | 48 987 | 684 ⚠23% | 14 375 | 2 899 |
| Bcachefs | 33 922 | 1 155 | 51 574 | 978 |
| OpenZFS (vdev is a partition) | 3 213 | 10 420 | 6 151 | 12 517 |
| singlefs (two disks) | — | — | — | — |

#### Metadata / small files (operations/s; higher is better)

| Configuration | Create empty file | stat | Delete file | 4 KiB write + fsync |
|---|---|---|---|---|
| ext4 | 238 095 ⚠25% | 1 000 000 ⚠48% | 303 030 ⚠22% | 708 |
| XFS | 222 222 | 1 250 000 ⚠42% | 192 307 ⚠20% | 706 |
| F2FS | 178 571 | 1 000 000 ⚠52% | 243 902 ⚠25% | 1 371 |
| Btrfs | 232 558 | 1 111 111 ⚠25% | 133 333 | 849 |
| Bcachefs | 140 845 | 1 111 111 ⚠31% | 126 582 ⚠22% | 1 008 |
| OpenZFS (vdev is a partition) | 123 456 | 909 090 ⚠41% | 82 644 ⚠38% | 1 970 ⚠22% |
| singlefs (two disks) | — | — | — | — |

#### Small-file persistence: create + write 3000 bytes + fsync file + fsync directory (100 times on a freshly formatted empty file system)

| Configuration | Median latency µs | p99 latency µs | Write requests/op | Bytes written/op | FLUSH/op | First operation bytes | First operation write requests | First operation FLUSH |
|---|---|---|---|---|---|---|---|---|
| ext4 | 1 395 | 1 567 ⚠52% | 3.00 | 36 290 | 2.00 | 36 864 | 3.00 | 2.00 |
| XFS | 1 161 | 1 296 | 2.00 | 9 963 | 2.00 | 6 656 | 2.00 | 2.00 |
| F2FS | 1 083 | 1 178 | 5.02 | 20 561 | 3.00 | 20 480 | 5.00 | 3.00 |
| Btrfs | 2 183 | 2 467 | 15.8 | 188 907 | 4.00 | 143 360 | 13.0 | 4.00 |
| Bcachefs | 1 461 | 2 077 | 2.15 | 6 656 | 2.00 | 14 848 | — | 2.00 |
| OpenZFS (vdev is a partition) | 853 ⚠22% | 923 ⚠42% | 2.27 | 4 613 | 1.02 | 14 336 ⚠82% | 8.00 ⚠88% | — |
| singlefs (two disks) | — | — | — | — | — | — | — | — |

#### Cold mount (unmount, clear caches, remount)

| Configuration | Wall time ms | Bytes read | Read requests |
|---|---|---|---|
| ext4 | 1.41 ⚠19% | 21 504 | — |
| XFS | 6.76 ⚠31% | 4 235 776 | 45.0 |
| F2FS | 2.89 | 692 224 | 14.0 |
| Btrfs | 4.54 | 339 968 | 23.0 |
| Bcachefs | 26.5 ⚠37% | 5 039 104 | 33.0 |
| OpenZFS (vdev is a partition) | 3 102 | 6 688 768 | 170 |
| singlefs (two disks) | 7 898 | 1 610 932 224 | 393 297 |

#### Formatting and usable capacity (16 GiB disks)

| Configuration | Format ms | Format bytes written | Format FLUSH | Total capacity % | Available % |
|---|---|---|---|---|---|
| ext4 | 5.33 ⚠17% | 138 702 848 | 4.00 | 97.4 | 92.3 |
| XFS | 19.4 | 67 575 808 | 2.00 | 99.6 | 97.5 |
| F2FS | 14.2 | 39 903 232 | — | 100.0 | 98.1 |
| Btrfs | 12.7 | 4 984 832 | 14.0 | 100 | 96.8 |
| Bcachefs | 31.1 ⚠25% | 16 384 | — | 92.0 | 90.6 |
| OpenZFS (vdev is a partition) | 278 | 1 791 488 | 13.0 | 93.8 | 93.8 |
| singlefs (two disks) | — | — | — | — | — |

#### singlefs (two 16 GiB disks, checker-tier-qemu-device-streams real-device binary)

| Configuration | mkfs + number acquisition + warm-up + new-pool file creation ms | Overwrite, release, rollback and reuse (overwrite B) ms | Publication B (binary's own timing) ms | Cold recovery ms | Bytes read across both disks | Read requests across both disks | B write requests across both disks | B bytes written across both disks | B barriers across both disks | B FUA writes across both disks |
|---|---|---|---|---|---|---|---|---|---|---|
| singlefs (two disks) | 17.8 ⚠131% | — | — | 7 898 | 1 610 932 224 | 393 297 | — | — | — | — |

Whole lines are copied from artifacts; lines whose artifacts were moved out of the workspace are represented by one-line pointers.

On 2026-09-15 all 40 VM runs succeeded on the first attempt; no cells were excluded. Every row for the seven configurations with controls has `rounds_with_teeth=5`:

```text
Artifact line: git show 0ec0be214cdc:research/results/e152-file-system-benchmark-2026-09-15.out line 717 (name=summary_control)
Artifact line: git show 0ec0be214cdc:research/results/e152-file-system-benchmark-2026-09-15.out line 718 (name=summary_control)
Artifact line: git show 0ec0be214cdc:research/results/e152-file-system-benchmark-2026-09-15.out line 719 (name=summary_control)
Artifact line: git show 0ec0be214cdc:research/results/e152-file-system-benchmark-2026-09-15.out line 720 (name=summary_control)
Artifact line: git show 0ec0be214cdc:research/results/e152-file-system-benchmark-2026-09-15.out line 721 (name=summary_control)
Artifact line: git show 0ec0be214cdc:research/results/e152-file-system-benchmark-2026-09-15.out line 722 (name=summary_control)
Artifact line: git show 0ec0be214cdc:research/results/e152-file-system-benchmark-2026-09-15.out line 723 (name=summary_control)
```

```text
Artifact line: git show 0ec0be214cdc:research/results/e152-file-system-benchmark-2026-09-15.out line 836 (name=summary)
Artifact line: git show 0ec0be214cdc:research/results/e152-file-system-benchmark-mirror-2026-09-15.out line 885 (name=summary)
Artifact line: git show 0ec0be214cdc:research/results/e152-file-system-benchmark-mirror-2026-09-15.out line 886 (name=summary)
Artifact line: git show 0ec0be214cdc:research/results/e152-file-system-benchmark-2026-09-15.out line 837 (name=summary)
Artifact line: git show 0ec0be214cdc:research/results/e152-file-system-benchmark-2026-09-15.out line 778 (name=summary)
Artifact line: git show 0ec0be214cdc:research/results/e152-file-system-benchmark-2026-09-15.out line 866 (name=summary)
Artifact line: git show 0ec0be214cdc:research/results/e152-file-system-benchmark-2026-09-15.out line 850 (name=summary)
Artifact line: git show 0ec0be214cdc:research/results/e152-file-system-benchmark-2026-09-15.out line 788 (name=summary)
Artifact line: git show 0ec0be214cdc:research/results/e152-file-system-benchmark-2026-09-15.out line 762 (name=summary)
Artifact line: git show 0ec0be214cdc:research/results/e152-file-system-benchmark-2026-09-15.out line 828 (name=summary)
Artifact line: git show 0ec0be214cdc:research/results/e152-file-system-benchmark-2026-09-15.out line 883 (name=summary)
Artifact line: git show 0ec0be214cdc:research/results/e152-file-system-benchmark-2026-09-15.out line 786 (name=summary)
Artifact line: git show 0ec0be214cdc:research/results/e152-file-system-benchmark-mirror-2026-09-15.out line 941 (name=summary_done)
```

The write volume in the singlefs column was not measured; it was calculated from the recorded stream in E142 (New-Pool File Creation Dry Run) artifact `e142-first-txn-dry-run-2026-09-14-round2-slot4096.out`:

```text
Artifact line: git show 0ec0be214cdc:research/results/e142-first-txn-dry-run-2026-09-18-change-count-three.out line 38 (name=write_list)
```


**Single disk: what the numbers mean**

1. **Cold-recovery read volume is constant over five rounds, mostly from scanning the whole ring:** both disks read 1 610 932 224 bytes in 393 297 requests each round (disk 0: 805 568 512 bytes / 196 661 requests; disk 1: 805 363 712 bytes / 196 636). `scan_journal` in `crates/singlefs-core/src/recovery.rs` reads each disk's 768 MiB journal ring slot by slot at 4096 bytes per slot: 196 608 requests and 805 306 368 bytes per disk, 99.98% of total bytes. Block-layer counts are compatible with this method but cannot independently confirm it: the extra 81 requests beyond 2 × 196 608 carry only 319 488 bytes (disk 0: 53 requests, 262 144 bytes; disk 1: 28, 57 344), and read bytes do not equal requests × 4 KiB (393 297 × 4 096 = 1 610 944 512, 12 288 more than measured). The block layer counts only requests and bytes and cannot tell which request hits the ring; separation needs device-side read logs or the binary's own read-call counts. The decomposition comes from code. Wall time 7 898 ms is about 5 600× ext4 cold mount at 1.406 ms and 2.5× OpenZFS at 3 102 ms.
2. **Small-file persistence:** singlefs new-pool file creation writes 172 544 bytes per disk (disk 0: eight unit writes, one journal record, one root slot, one system configuration slot). The six write 6 656 bytes (XFS) to 143 360 (Btrfs) on their first such operation on an empty file system. Singlefs wall time measures only an upper bound for the whole stage; five rounds split into 16.955–17.811 ms and 40.123–40.277 ms clusters, so it is not compared. The extra 24.1 and 25.8 ms in the two 40 ms rounds are outer-wrapper printing backlog (item 4 in “What these numbers mean: overwrite, release, rollback and reuse compared with the six mirrors”).
3. **Among the six:** at default settings OpenZFS 4K random read is 4.5% of raw disk; the block layer actually reads 128 KiB per 4 KiB read, 32× amplification, matching the default 128 KiB recordsize. Bcachefs random-read amplification is 16×; ext4, XFS and F2FS have 1.00 amplification in all four data workloads. Per-cell tables are in `scripts/experiment-apparatus/perf-by-milestone.md` II.
4. These numbers settle no decision: whether and how to reduce cold-recovery reads is a design inference requiring separate three-way argument.

### Measurement conventions

- Five rounds per configuration, each with a new VM and disk, with eight configurations interleaved by round. Report median, minimum, maximum and spread; spread ≤15% is “stable.”
- Validate the path by reading guest `/sys/block/vda/stat` before and after every fio job. If block-layer bytes for a direct read (write) job are less than 90% of fio's reported bytes, mark the cell “replaced by cache” (“write absorbed”) and exclude it from the median. Self-test: perform two non-direct reads each round; the same predicate must classify the second as “replaced by cache,” or that round's read cell is invalid.
- The first singlefs small-file persistence measurement gives only an upper bound: its real-device binary divides wall time into three stages (start to new-pool file creation; new-pool file creation to overwrite/free/rollback/reuse; cold recovery). The first cannot separate mkfs, number acquisition and warm-up. Starting 2026-09-17, the second was the difference between outer-wrapper line arrivals; summaries for the two 2026-09-17 runs include printing backlog and do not give publication B time (item 4 of the comparison section). From preregistration section 14, the apparatus reads to EOF before forwarding and reports a containment check. Per-persistence wall time compared with the six is the binary's own `nanoseconds=` on `name=file_overwrite` (summary metric `singlefs_file_overwrite_inner_milliseconds`), covering publication B only.
- Write requests: the six use guest block-layer counts; the singlefs summary uses binary write calls. The block layer additionally counts each barrier and FUA once; use block-layer counts for like-for-like comparison (comparison section item 5).
- The last row in the six tables is singlefs (`SINGLEFS_FROM_SUMMARY` and `SINGLEFS_FROM_BLOCK_LAYER` in `scripts/experiment-apparatus/e152-tables.py`): fill only comparable quantities, otherwise “—”. Small-file persistence median latency uses the binary's own B time in µs; bytes use B's total across both disks; requests and FLUSH are round-by-round block-layer differences across B on both disks, subtracting pre-B `write_ios`/`flush_ios` in the two `device_calls` lines from whole-path `singlefs_timing` counts; cold recovery does not write. The three cold-mount cells use recovery wall time and both disks' read bytes and requests. Singlefs lacks comparable latency p99, the three “first operation” cells, large-file, 4K random, metadata, formatting and capacity quantities (no POSIX interface; mkfs was not separately timed). Workload differences are in comparison item 6. In the 2026-09-29 run block-layer writes 36 = 29 write calls + 6 barriers + 1 FUA, and FLUSH 7 = 6 barriers + 1 FUA, agreeing with the two paths in item 5.

### What it cannot answer

1. Relative placement on virtual disks, not absolute performance on real NVMe.
2. Mirror implementations differ: ext4 / XFS / F2FS use md raid1 (cold mount excludes array assembly), while Btrfs, Bcachefs and OpenZFS use their own mirrors. Bcachefs statfs does not halve for replicas, so mirrored capacity is not comparable.
3. Defaults only, one size, no aging, no concurrency.
4. At new-pool file creation singlefs can run only small-file persistence, cold mount and formatting dimensions.
5. The metadata stat column is quantized by fio's millisecond runtime: one stat over 10 000 files takes only 8–11 ms at the six medians, so readings can only be “10 000 ÷ integer milliseconds” and cannot distinguish them. Comparison requires more than 100 000 files and a separate formal run.
6. Amplification (block-layer bytes ÷ fio bytes) was calculated after the run from artifact block-layer counts; it was not preregistered as a metric.
7. The breakdown of singlefs's 6.6 ms per persistence among encoding/checksumming, 21 writes and five flushes was not measured.
8. Every singlefs tree had one node in the two 2026-09-17 runs; write volume after trees grow and files multiply was not measured.
9. Whether the six systems' 8–200 KiB per operation writes journal, intent log or trees was not measured; use requires current research, marked unverified by this project.
10. How to narrow the gaps (data-unit granularity, seven trees each rewriting a unit on overwrite, reading the whole ring at cold recovery) is a design inference requiring three-way argument.

### Decisions affected

| Decision item | Relationship | Review |
|---|---|---|
| D23 (Journal Role & Format) | Background material | 2026-10-15 unaffected: whole copied artifact lines were replaced by artifact pointers; numbers, conclusions and artifacts unchanged. |
| D17 (Implementation Layering & Third-Party Pipeline), settled item 5 | Background material | 2026-10-15 unaffected: whole copied artifact lines were replaced by pointers to artifacts; numbers, conclusions, and artifacts unchanged. |
| D2 (RAID Stripe Policy), settled item 9 | No effect | 2026-10-15 unaffected: whole copied artifact lines were replaced by pointers to artifacts; numbers, conclusions, and artifacts unchanged. |
| D4 (Checksum Location), settled item 5 | No effect | 2026-10-15 unaffected: whole copied artifact lines were replaced by pointers to artifacts; numbers, conclusions, and artifacts unchanged. |

## Historical Versions

### 2026-09-29

- Formal run after the quiesce following the crash amplification full run: seven two-disk mirrors + singlefs, five rounds each (`e152-file-system-benchmark-mirror-2026-09-29.out`, all 40 VM runs `vm_exit=0`, host load 0.50–9.61). Tables and cell-by-cell comparison with 2026-09-28 are in `scripts/experiment-apparatus/perf-by-milestone.md` III.4: singlefs read/write bytes and requests were identical in both; first-stage median 1 874 → 348 ms and cold recovery had no outlier, reflecting host conditions. Most competitor cells changing over 15% were sequential writes marked unstable on both days. All four affected-decision rows were reviewed to that date.

### 2026-09-28

- Formal run for milestone two wrap-up step 8: seven two-disk mirrors + singlefs, five rounds each (`e152-file-system-benchmark-mirror-2026-09-28.out`, all 40 VM runs `vm_exit=0`). Tables and row-by-row comparison to the 2026-09-17 singlefs arm are in `scripts/experiment-apparatus/perf-by-milestone.md` III.3: first-stage median 1 874 ms (mkfs actually writes the whole journal ring, 805 959 680 bytes per disk), B 29 writes / six barriers median 19.4 ms, cold recovery median 10.0 s (round 3 outlier 72.6 s).

### 2026-09-19

- Formal run reran only singlefs (`e152-r5-prereg.md`, first formal run after apparatus changed to read to EOF before forwarding, answering the four rows in `e152-r5-questions.md`); added a section copying whole lines. Found a second truncation source the new read method does not remove (guest kernel shutdown dmesg competes with the last user-space line for the same serial port, truncating `singlefs_timing` in 3/5 rounds); the existing mismatch between registered anchor “21 operations” and implementation “23 operations”; P2 positive-control injection infeasible because `vm-bench.sh` is not anchored to line start; and G1 single-vCPU containment remained false in 2/3 rounds. Change history in [experiments-history.md](../experiments-history.md), 2026-09-19.

### 2026-09-17

- Apparatus change (preregistration section 14): previously comparison item 4 said “change the apparatus before the next formal run,” with code and mutation table at 22 unit tests / 16 mutations all caught. Afterwards it documents the change (read through EOF before forwarding, containment check, new metric), one-round smoke readings, 26 tests / 26 mutations all caught, and updates the second-stage measurement convention. Basis: user requested on 2026-09-17 that the timing issue be documented. Change history in [experiments-history.md](../experiments-history.md), 2026-09-17.
- Corrected two cold-recovery read descriptions: previously first-results item 1 said requests were “exactly 2 × 196 608 full-ring 4 KiB reads plus 81” and “block-layer count is an independent second path,” while the comparison said read volume “depends only on journal-ring size.” Now it says block-layer counts are compatible with but cannot prove slotwise full-ring reading (the extra 81 requests carry only 319 488 bytes; bytes do not equal requests × 4 KiB; ring requests cannot be distinguished), the full ring accounts for 99.98% of read bytes, and outside-ring reads depend on contents, with code pointer `scan_journal` (the old line number had drifted). Basis: user noted on 2026-09-17 that the statement did not match the two raw byte/request counts; per-disk calculation is in [experiments-history.md](../experiments-history.md), 2026-09-17.
- Added “What these numbers mean (overwrite, release, rollback and reuse, compared with the six mirrors)”: seven-system rankings for the two 2026-09-17 runs versus mirrors, origin of the gap, and two convention issues (outer printing backlog in summary overwrite wall time, replaced by binary-measured nanoseconds; write requests compared using block-layer 26). Updated two convention bullets, the two clusters in first-results item 2, and limitations 7–10 accordingly. Change history in [experiments-history.md](../experiments-history.md), 2026-09-17.
- 2026-09-17 wrap-up formal run: reran singlefs at milestone completion (preregistration section 13) and added whole-line transcription. Change history in [experiments-history.md](../experiments-history.md), 2026-09-17.
- 2026-09-17 step 7 formal run: reran only singlefs at the overwrite/free/rollback/reuse milestone (`file-overwrite` mode, preregistration section 12); added whole-line transcription, a reproduction command, and changed the timing convention to three stages. Change history in [experiments-history.md](../experiments-history.md), 2026-09-17.

### 2026-09-15

- Created preregistration, apparatus, and first smoke run. Change history in [experiments-history.md](../experiments-history.md), 2026-09-15 (part one).
- Formal five-round run; results section copies artifact lines in full. Change history in [experiments-history.md](../experiments-history.md), 2026-09-15 (part two).
- 2026-09-15 two-disk-mirror formal run; added whole-line transcription. Change history in [experiments-history.md](../experiments-history.md), 2026-09-15 (part three).
