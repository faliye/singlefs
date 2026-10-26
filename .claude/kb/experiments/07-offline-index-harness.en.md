## E7 Offline Index Harness —— First Round Run (2026-08-27)

[中文](07-离线索引harness.md) · **English** · [日本語](07-オフラインインデックスharness.ja.md)

**Background**: E1 (Node Size vs Recoverability) / E2 (Key Layout Impact on Recursive Traversal) / E4 (Crossover of Range Rebuild vs Item-by-item Modification) all stated "depends on D8 (Core Index Structure) settlement and checker", while parameter selection for D8 (Core Index Structure) waited on these experiments,
forming a deadlock. The way out is to acknowledge one fact: **selecting an index structure does not require a complete file system**,
only an independent index engine runnable on real block devices and a workload replayer.

**What to Do**: Write an **index harness** without VFS, without POSIX semantics, and without crash recovery—
containing only "maintaining a tree / a KV store on a block device, supporting insert / point query / range scan / range deletion".
Implement each candidate structure atop it, comparing them under the same workload replay.

**Candidate Structures** (based on [prior-art.md](../prior-art.md) 7.3):

| Candidate | Things to Measure |
|---|---|
| B+tree (baseline) | Read cost upper bound at optimal point lookup |
| B+tree with log-structured nodes (D8 (Core Index Structure) current direction) | Actual gain of in-node append + periodic reshuffle vs reshuffle cost |
| Bε-tree (internal nodes with message buffer) | Small-write absorption capacity, point lookup degradation |
| Bε-tree + size-tiering + filter (SplinterDB form) | Write amplification and point lookup difference relative to pure Bε-tree |
| Any of remaining 4 candidates + key-value separation (WiscKey form) | After large values move out of index, reduction in compaction / reshuffle cost vs range scan degradation |

**What to Measure**: Write amplification, read amplification, point lookup latency distribution, range scan bandwidth, space amplification, CPU utilization.
**Every metric must be split into "cold cache" and "hot cache" groups**—under hot cache, all structures appear roughly identical.

**Criteria**: Produce a table of "workload shape × structure", each cell showing the six metrics from "What to Measure".
**This table itself is the output**, with no requirement to pick a single winner—D8 (Core Index Structure) needs "how to set parameters", not "which tree to pick".

**The baseline must be the D8 (Core Index Structure) current direction, not a textbook B+tree.**
The settled form of D8 (Core Index Structure) is "log-structured nodes + write buffer frontend", **which itself is a mechanism for absorbing small writes**.
Using a naive sorted-node B+tree as baseline merely measures "whether batching exists", not "what message buffers buy over write buffer".
Literature numbers are all relative to naive baselines: BetrFS compares against ext4/btrfs/XFS/ZFS, SplinterDB compares against RocksDB,
**and not a single control group carries a write buffer frontend**—the marginal benefit of message buffers over D8 (Core Index Structure) current direction has zero numbers in existing literature.

**What Counts as Failure** (split into two classes, no mixing allowed):

| Symptom | Verdict | Reason |
|---|---|---|
| All candidates on all workloads show differences smaller than measurement noise | **Entire round invalidated**; enlarge dataset, clear cache, and rerun | Likely dataset was small enough to fit completely into page cache |
| Bε is not significantly superior to control on **no-batching control group** (naive sorted-node B+tree) | **Implementation faulty, entire round invalidated** | Positive control: effect is established in literature; failure to measure means implementation error (buffer ratio set to 0, every insertion triggering flush, etc.) |
| Bε has no advantage relative to **D8 (Core Index Structure) current direction baseline** | **This is a valid result, record truthfully** | Indicates message buffer bought nothing beyond what write buffer already bought—the exact question E7 (Offline Index Harness) was intended to answer |

⚠️ **The no-batching control group must be retained**, otherwise "no advantage" cannot distinguish between "such is the conclusion" vs "implementation broken".
Running both control groups simultaneously provides both discriminative power and falsifiability.

### First Round Results (2026-08-27, N=5 rounds, 5 different seeds, with block-layer independent readings)

**Device I/O on Update Path (per operation) and Read Count on Point Lookup**

| Arm | Update io/op (median) | Range / Median | Point Lookup Reads/op |
|---|---|---|---|
| **sorted_bplus** (no-batching control, positive control) | 1.9378 | 0.0% | 1.241 |
| **logstruct_wb** (D8 (Core Index Structure) current direction: log-structured nodes + write buffer frontend) | 1.5755 | 0.2% | 1.241 |
| **betree** (internal nodes with message buffer) | **0.4900** | 0.9% | **1.178** |

> **Bε relative to D8 (Core Index Structure) current direction: 3.22×** (update path)
> **D8 (Core Index Structure) current direction relative to no-batching control: only 1.23×**

**This is precisely the number required by [decisions.md](../decisions.md) D11 (Message Buffers in Index Nodes) prerequisite 2.**
Prior to E7 (Offline Index Harness), zero existed in literature—BetrFS compared against ext4/btrfs/XFS/ZFS, SplinterDB compared against RocksDB,
**with no control group carrying a write buffer frontend**.

#### Three Hard Requirements Judged Item by Item

| Requirement | Observation | Verdict |
|---|---|---|
| 1. Each arm genuinely implements its claimed structure | Replacing core criteria of two batching arms with constant (buffer=1): logstruct_wb 0.866 → **1.502** (+73%), betree 0.171 → **1.702** (+897%); sorted_bplus unchanged (natively unbuffered) | **Pass**. Structures are genuinely participating |
| 2. Amortization architectures symmetric | logstruct_wb and betree share identical node cache, identical checkpoint cadence (every 4096 operations). sorted_bplus intentionally writes through—**it is a no-batching control and does not participate in symmetry requirement** | **Pass** |
| 3. Block-layer independent readings + validation path itself has discriminative power | Program counters across 5 rounds and 9 cells match `/sys/block/vda/stat` **round-by-round**; removing `O_DIRECT` zeroes block-layer readings while program counters remain unchanged, wall clock drops 73× | **Pass** |

#### Two Analytic Controls (Third Independent Path)

- `sorted_bplus`: 1 write + `(1 − 64/1024)` reads per op = **1.9375**, measured **1.9378**.
- `logstruct_wb`: 512 random keys fall into approx 400 distinct leaves, 1 read + 1 write per leaf ⇒ approx **1.56**, measured **1.5755**.

⚠️ **Low range is not a determinism artifact**: five rounds had distinct seeds (101/202/303/404/505),
the other two arms having ranges of 0.2% and 0.9%. The 0.0% of `sorted_bplus` comes from exactly 1 write per op,
with read counts being the average miss rate across 200,000 samples.

### Cache-to-Tree Ratio Sweep (2026-08-27, 4 tiers × 5 seeds = 20 rounds, all valid)

**Update Path io/op (median)**

| Cache Pages | % of Tree | No-batching Control | D8 (Core Index Structure) Current Direction | Bε | **Bε / D8 (Core Index Structure)** | D8 (Core Index Structure) / No-batching |
|---|---|---|---|---|---|---|
| 16 | 2% | 1.9844 | 1.5755 | 0.6384 | **2.47×** | 1.26× |
| 64 | 6% | 1.9378 | 1.5755 | 0.4900 | **3.22×** | 1.23× |
| 264 | 25% | 1.7437 | 1.5755 | 0.3176 | **4.96×** | 1.11× |
| 1057 | **100%** | 1.0051 | **0.2515** | 0.1678 | **1.50×** | **4.00×** |

**Point Lookup Reads/op (median)**: Three arms almost identical at each tier (2% tier 1.78/1.78/1.72,
100% tier all 0.053), **Bε slightly superior rather than inferior**—see section on extrapolation boundaries for reason.

Block layer vs program counter: **60/60 cells matched**.

#### Two Conclusions

**1. `Bε / D8` (Core Index Structure) is non-monotonic, peaking in the middle.**
2% → 25% rises steadily to 4.96×, while dropping sharply to **1.50×** when cache fits the entire tree.
⇒ **The advantageous regime of Bε is "tree exceeds cache"**—isomorphic to the advantageous regime of D5 (Snapshot / Space Accounting Mechanism) in E12 (Batched Sequential Append vs Unbatched Random Page RMW)
("counter table exceeds cache"): both substitute random lookups with sequential/batched operations,
a property valuable only when data cannot fit in cache.

**2. D8 (Core Index Structure) current direction io/op is bit-for-bit identical (1.5755) across 2% / 6% / 25% tiers—cache has zero effect on it.**
Mechanism: **After write buffer sorts by leaf, each flush performs a sequential scan spanning the entire leaf space,
and LRU hit rate on "repeated sequential scans larger than cache" is precisely 0.**

Intervention confirmation (holding intra-leaf aggregation constant, shuffling only **group processing order**):

| Cache Tier | Sorted Flush | Shuffled Group Order | |
|---|---|---|---|
| 25% | 157,739 reads | **143,251 reads (−9.2%)** | Ordering is indeed the driver |
| 100% | 1,024 reads | **1,024 reads (exact match)** | **Negative control**: ordering irrelevant when fully resident |

⚠️ That 9.2% is a **lower bound**: shuffle seeds derive from group counts; identical group counts yield identical permutations,
introducing correlation across flushes and damping the effect. **Direction and negative control both hold; magnitude is conservative.**

> **This yields a design requirement: If write buffer sorts, the node cache must be scan-resistant
> (such as 2Q / LRU-K), otherwise sorting destroys locality instead of creating it.**
> This directly impacts [decisions.md](../decisions.md) D8 (Core Index Structure); D8 (Core Index Structure) text on 2026-08-27 merely stated
> "write buffer batches, sorts, deduplicates, then batch inserts", **posing zero requirements on cache eviction policy**.

### Node Size Sweep (2026-08-27, 3 tiers × 5 seeds = 15 rounds, **fixed ~1 MB cache budget**)

**Crux of measurement basis: Fixed memory budget; larger nodes mean fewer resident nodes.** This is the real trade-off.

| Node | Cache Nodes | % of Tree | | No-batching Control | D8 (Core Index Structure) Current Direction | Bε |
|---|---|---|---|---|---|---|
| **4 KiB** | 264 | 25% | io/op | 1.7437 | 1.5755 | **0.3176** |
| | | | Bytes/op | 7,142 | 6,453 | **1,301** |
| | | | Point lookup reads/op | 0.7766 | 0.7766 | 0.7361 |
| **16 KiB** | 66 | 6% | io/op | 1.9359 | 1.5755 | **0.3994** |
| | | | Bytes/op | 31,718 | 25,812 | **6,544** |
| | | | Point lookup reads/op | 1.2249 | 1.2249 | 1.1624 |
| **64 KiB** | 16 | 2% | io/op | 1.9844 | 1.5755 | **0.6384** |
| | | | Bytes/op | 130,050 | 103,249 | **41,841** |
| | | | Point lookup reads/op | 1.7792 | 1.7792 | 1.7175 |

Block layer vs program counter: **45/45 cells matched**.

#### Conclusion: Under Fixed Memory Budget, Large Nodes Are Worse on All Three Axes

I/O count doubles (Bε 0.318 → 0.638), byte volume **32×** (1,301 → 41,841), point lookup read count doubles.
Mechanism: **The larger the node, the fewer nodes fit in memory; hit rate drops faster than "reading more per I/O" earns back.**

`logstruct_wb` io/op is **bit-for-bit identical across all three tiers (1.5755)**—for the same reason (sorted flush ⇒ sequential scan ⇒ hit rate 0),
node size does not alter the number of touched nodes; yet its byte volume surges **16×**.

⚠️ **It cannot justify "layered layout per media class"**: [decisions.md](../decisions.md) D8 (Core Index Structure) Constraint 1 settled node size as
a format parameter written into system configuration at mkfs—**parameters suffice, without needing multiple layout branches**
(layout branch count belongs to D12 (Target Media) settled item 1, uncapped in advance).
E7 (Offline Index Harness) records "one node read" as 1 device I/O, **regardless of node size**;
on VM + host file (approximating SSD), it is **byte-dominated**, making large nodes a net loss.

**Seek-dominated side has been measured (see E7 (Offline Index Harness) "Seek-Dominated Tier"), conclusions being co-directional rather than opposite**:
At the extreme where "each I/O cost is identical, independent of size", large nodes still do not win.
⇒ **Small nodes win at both byte-dominated and seek-dominated ends**, so node size need not take opposite directions per media.

⇒ Regarding [decisions.md](../decisions.md) D8 (Core Index Structure) "do not blindly copy 128K–256K": **supported at both ends**.

### Seek-Dominated Tier (2026-08-27, QEMU throttled to 4000 IOPS, **2 tiers × 5 seeds = 10 rounds**, end-to-end comparison)

**N=5**: 5 seeds each across two configurations (11/22/33/44/55), all 10 rounds completed with completion marks, zero discarded rounds.

**Measurement Basis**: Host lacks spinning disks (`lsblk` confirms nvme0n1 only, ROTA=0), hence this is
**measurement on modeled device, not spinning disk measurement**. Model = QEMU `throttle-group x-iops-total=4000`,
without setting `x-iops-size` ⇒ **each I/O cost is identical, independent of request size**, representing the seek-dominated extreme.
Rerun: `VM_IOPS=4000 bash scripts/devices/vm-bench.sh <musl binary> 11 none <cache> <node_pages>`.
⚠️ **Raw output unarchived**: E7 (Offline Index Harness) ran inside QEMU and did not persist results pulled back from host to disk at that time.

### 2026-08-29 Rerun: Numbers Reproduced, but **Validation Path Did Not Reproduce**

Reran on a **regular host file** with N=5 and five seeds,
artifact `e7-index-2026-08-29-file.out` (first time E7 (Offline Index Harness) had archived artifacts).

| Arm | Current Median io/op | Recorded in Body | Diff |
|---|---|---|---|
| No-batching control (`sorted_bplus`) | 1.9373 | 1.9378 | −0.03% |
| D8 (Core Index Structure) current direction (`logstruct_wb`) | 1.5737 | 1.5755 | −0.11% |
| Bε (`betree`) | 0.4913 | 0.4900 | +0.27% |

⇒ **Bε relative to D8 (Core Index Structure) current direction: 3.20× this run** (recorded 3.22×, diff 0.5%);
**D8 (Core Index Structure) current direction relative to no-batching control: 1.23× this run** (consistent with body text).

⚠️ **When run on regular files, `blkstat=false`—the independent block-layer reading validation path is unavailable.**
Thus that round reproduced the numbers, not the validation path. **Supplemented on the same day in VM**, see section E7 (Offline Index Harness) "Same-Day Rerun in VM: Validation Path Also Reproduced".

### Same-Day Rerun in VM: Validation Path Also Reproduced

Artifact `e7-index-2026-08-29-vm.out`.
Execution follows [vm-harness.md](../vm-harness.md): musl static compile + `scripts/devices/vm-bench.sh` into
QEMU/KVM; guest runs against real virtio disk (`/dev/vda`), with `blkstat=true`.

| Arm | Program Read = Block Read | Program Write = Block Write |
|---|---|---|
| No-batching control (`sorted_bplus`) | 187477 = 187477 ✓ / Query 24873 = 24873 ✓ | 200000 = 200000 ✓ |
| D8 (Core Index Structure) current direction (`logstruct_wb`) | 157592 = 157592 ✓ / Query 24873 = 24873 ✓ | 157592 = 157592 ✓ |
| Bε (`betree`) | 48947 = 48947 ✓ / Query 23890 = 23890 ✓ | 49787 = 49787 ✓ |

⇒ **Nine cells matched cell-by-cell**, identical to the check "block-layer independent readings match round-by-round" in body text.
⇒ Confirms `O_DIRECT` truly bypassed cache, block layer did not merge/split requests, and no readahead occurred.

**Moreover, VM and host `io_per_op` are bit-for-bit identical** (same default seed `0x51561234`):
1.937385 / 1.575920 / 0.493670. **Different medium, different kernel, counting does not differ by a single bit.**

⚠️ **Timing figures are not usable**: Only readable kernel under host `/boot` is `vmlinuz-6.17.0-lockdep`,
carrying heavy lock validation overhead—same tier in VM takes approx **2×** host time (29.7 s vs 14.5 s).
**Count-based metrics unaffected; time-based metrics must not be compared across environments.**

⚠️ **Measurement basis differences must be stated**: Original round ran on QEMU block device with different seeds;
this round ran on ext4 regular file with `O_DIRECT` enabled. Differences across all three arms are within 0.3%.
"Same-day rerun in VM: validation path also reproduced" numbers can **only be verified by rerunning, not by checking archives**—rerun requires QEMU.

**Throttling path self-test goes red**: For identical 300 direct reads, unthrottled takes 0.18 s; throttled to 100 IOPS takes 2.89 s
(≈ 300/100 = 3 s). Skipping this step leaves "no measured difference" indistinguishable from "throttling never took effect".

**Model Self-Test**: Alignment between wall clock and "I/O count ÷ throttle limit" (median, N=5)

| Tier | Arm | io/op Median | Predicted Wall Clock | Measured Median Wall Clock | Error |
|---|---|---|---|---|---|
| 4 KiB | No-batching control | 1.742435 | 87.12 s | 87.13 s | 0.01% |
| 64 KiB | No-batching control | 1.984590 | 99.23 s | 99.18 s | 0.05% |
| 4 KiB | Bε | 0.317940 | 15.90 s | 15.90 s | 0.00% |
| 64 KiB | Bε | 0.639135 | 31.96 s | 31.91 s | 0.15% |
| 4 KiB | D8 (Core Index Structure) current direction | 1.574990 | 78.75 s | 78.76 s | 0.01% |
| 64 KiB | D8 (Core Index Structure) current direction | 1.574990 | 78.75 s | 78.65 s | 0.13% |

⇒ **Byte volume is completely unpriced on this device**: 64 KiB tier reads 130,041 bytes per op, 4 KiB tier reads 7,137 bytes
(**18×**), while wall clock is determined solely by I/O count. The model delivered what it claimed.

**Block-Layer Validation Still Holds**: Throttling inserts a `throttle` block driver in QEMU stack
(`-blockdev driver=throttle` replacing original `-drive`), altering the I/O path,
so block-layer controls cannot be assumed valid by default. Measurement showed **12/12 cells bit-for-bit matched** (two configs × three arms × update/query,
program-counted reads/writes exactly matching guest `/sys/block` `blk_r`/`blk_w`).
This validation path's self-test went red in the 2026-08-27 first round—removing direct I/O zeroed block-layer counts while program counts remained untouched.

**Results**

| Arm | 4 KiB Median Wall Clock | 64 KiB Median Wall Clock | Large Node | Respective Range |
|---|---|---|---|---|
| No-batching control | 87.13 s | 99.18 s | **Diff 13.8%** | 0.06% / 0.32% |
| D8 (Core Index Structure) current direction | 78.76 s | 78.65 s | Parity (−0.14%) | 0.09% / 0.06% |
| Bε | **15.90 s** | 31.91 s | **Diff 100.7%** | 0.72% / 0.33% |

**Range ≤0.72% is not suspicious, but mechanistic**: Wall clock = I/O count ÷ throttle rate,
and I/O count across five seeds differed by only 0.06% (4 KiB no-batching control tier: 1.742510 / 1.741970 /
1.742720 / 1.742435 / 1.741695). **Seeds did vary**—five values all distinct;
merely this workload structure is deterministic, with only the key sequence varying.
The tiny range conversely confirms that "throttling indeed dictates wall clock".

**⇒ At seek-dominated extreme, large nodes still do not win; no reversal occurred.**

Reason: The independent variable is **fixed memory budget**; larger nodes mean fewer fit in memory, miss rate rises,
and **node reads actually increase**—an effect overpowering "reading more per I/O".

### What Seek-Dominated Tier Cannot Prove

- **Not spinning disk measurement.** It is a modeled device.
- **Model omits sequential merging.** On real spinning disks, adjacent small I/Os merge into single seeks,
  whereas QEMU IOPS throttling bills individually ⇒ **this model overestimates small node costs**.
  Even when overestimated, small nodes still win; **thus this limitation solidifies rather than weakens conclusions**.
- **Only two ends tested (4 KiB and 64 KiB), omitting 16 KiB middle tier.** Monotonicity unverified.

### What This Set of Numbers Cannot Prove

- ⚠️ **Tree is tiny**: 1024 leaves + 32 internal + 1 root = 1057 nodes (approx 4.3 MB), cache 64 pages (**6%**).
  Real trees are much larger, and **cache-to-tree ratio is the driving parameter**—altering this ratio changes gaps across all three arms.
- ⚠️ **Workload consists of random updates over existing keyspace, without splits/merges**.
  Splitting behavior is out of range, whereas BetrFS self-reports that worse cases occur precisely when "node splits and merges fall on the critical path".
- ⚠️ **Point lookup column is counterintuitive due to configuration**: Queries use the **same uniform random distribution** as updates,
  so recently updated keys have a high probability of lingering in cached root/internal buffers, saving a leaf read.
  **Queries on cold keys will yield opposite results**—this must be measured separately, which E7 (Offline Index Harness) did not measure.
- ⚠️ Nodes fixed at 4 KiB, values fixed at 8 bytes, whereas D8 (Core Index Structure) settled item 2 selects **16 KiB constant**
  ⇒ **All three arms of E7 (Offline Index Harness) ran on a node size that is no longer this project's chosen value.**
  For weight at 16 KiB, see E56 (Message Buffer Gains vs Epsilon).
- ⚠️ **In Bε arm, buffer capacity and fanout are two unrelated constants** (`message_buffer_capacity = 200`, `FANOUT = 32`,
  internal nodes hold only messages without pivots) ⇒ neither competes for the same node bytes,
  whereas that tension is the entire mechanism of Bε. **E7 (Offline Index Harness) node size sweep therefore simultaneously varied node size and buffer proportion
  (0.78 at 4 KiB, 0.195 at 16 KiB, 0.049 at 64 KiB), rendering the three tiers incomparable.**
  For correction, see E56 (Message Buffer Gains vs Epsilon).

**Measurement Basis**: `e7-index`, musl static, xorshift64\*; fanout 32, 1024 leaves, 64 keys per leaf,
keyspace 65536; node 4 KiB, 255 slots per node; cache 64 pages, write buffer 512 entries,
Bε buffer 200 entries, checkpoint every 4096 operations; each round 200,000 updates + 20,000 point lookups.
VM 2048M / 4 vCPU / 64 MiB virtio disk, `cache=none,aio=native`; host 32 cores / 60 GiB; QEMU 8.2.2 + KVM.

### ⚠️ That Analytic Control Previously Lived Only in Prose (Supplemented 2026-08-29 in adversarial validation)

The third independent validation path listed in E7 (Offline Index Harness) was "two analytic formulas match measurement (1.9375 vs 1.9378, approx 1.56 vs 1.5755)",
yet **zero assertions pinned it in code**—the four unit tests pinned non-overlapping page numbers, keys mapped within bounds, and self-consistent geometry, **all being structural properties**.
⇒ Once geometric constants change, 1.9375 silently invalidates, while the **3.22× cited in D11 (Message Buffers in Index Nodes) prerequisite 2 was derived from these very numbers**.

Added `the_no_batching_control_arm_has_an_analytic_io_per_op_of_1_9375`:
`1 + (1 − 64/1024) = 1.9375`, pinning `LEAF_COUNT == 1024` and `FANOUT² == LEAF_COUNT`.
⚠️ **It pins analytic formulas and geometric constants, not measured values**—measurement requires devices and VMs, which cannot run in unit tests,
and comparing the two still relies on rerunning. Mutation table stored at `scripts/mutation/tables/e7_index.tsv`.

### Three Hard Requirements Learned from E12 (Batched Sequential Append vs Unbatched Random Page RMW) (Do not run without satisfying)

The lesson of E12 (Batched Sequential Append vs Unbatched Random Page RMW) was concrete: one of its arms claimed to implement a mechanism but was actually a stub,
and amortization architectures between arms were asymmetric—**as a result, in that 170,000× figure, contributions of mechanism vs batching could not be teased apart**.
E7 (Offline Index Harness) must not repeat this:

1. **Each arm must genuinely implement its claimed structure.**
   Whatever the arm is named, that entity must exist in code.
   ⚠️ **Self-check method**: Replace the core criterion of the arm with an always-true/always-false constant; results **must change significantly**;
   no change indicates that structure never participated, **entire round invalidated**.
   (In E12 (Batched Sequential Append vs Unbatched Random Page RMW), replacing Arm A's criterion with a coin flip gave identical results—the exact pattern this self-check turns red on.)
2. **Amortization architecture across all arms must be symmetric**: Same batching window, same flushing timing, same publish semantics.
   [decisions.md](../decisions.md) D16 (Publish Semantics) settled checkpoints; **all arms run under the same checkpoint cadence**.
   Otherwise what is measured is "who batched", not "whose structure is superior".
3. **Device I/O must have independent block-layer readings** (inside VM `/sys/block/*/stat`), matched cell-by-cell against internal program counters;
   and **this validation path itself must prove discriminative power via fault injection** (removing `O_DIRECT`, block-layer readings must zero out).
   `.claude/singlefs-ai-sop/rules/evidence-discipline.md`: A validation path must also prove it turns red.

### Adding Another Independent Variable: Idempotent Full Values vs Deltas in Buffers

The cost criterion in [decisions.md](../decisions.md) D11 (Message Buffers in Index Nodes) has been rewritten to "whether buffers store **idempotent full values** (merge = latter wins,
checker needs no understanding of value semantics) or **incremental deltas** (merge relies on value semantics)".
**This must be measured as an independent variable**: Write amplification and merge costs under both entry formats on the same arm.
It determines whether Cost 1 of D11 (Message Buffers in Index Nodes) holds, a criterion spanning D8 (Core Index Structure) and D11 (Message Buffers in Index Nodes).

**Prerequisites**: Block device, workload replayer, Rust toolchain. **Requires no file system, no checker, no D8 (Core Index Structure) settlement.**

**What It Unblocks**: Node size of E1 (Node Size vs Recoverability), key layout of E2 (Key Layout Impact on Recursive Traversal), range rebuild crossover of E4 (Crossover of Range Rebuild vs Item-by-item Modification),
large/small file diversion of E8 (Different Write Paths for Large vs Small Files), full path vs inode indexing of E9 (Key Encoding Impact on Traversal Locality)—the main parts of five experiments can be run on this harness first,
leaving only "parts requiring real POSIX semantics and crash recovery" until after the file system exists.

### Impacted Decisions

| Decision Item | Relationship | Review |
|---|---|---|
| D8 (Core Index Structure) settled item 15 | Support | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D8 (Core Index Structure) settled item 4 | Support | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D11 (Message Buffers in Index Nodes) settled item 1 | Preparation | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D12 (Target Media) settled item 1 | Does not affect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D16 (Publish Semantics) | Does not affect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D5 (Snapshot / Space Accounting Mechanism) | Does not affect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |

## Historical Versions

E7 (Offline Index Harness) history entries are consolidated in [experiments-history.md](../experiments-history.md).
