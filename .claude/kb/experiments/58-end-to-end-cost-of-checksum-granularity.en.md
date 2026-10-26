## E58 (End-to-End Cost of Checksum Granularity) — Run (2026-08-31, N=5 + 1 Robustness Round)

[中文](58-校验和粒度的端到端代价.md) · **English** · [日本語](58-チェックサム粒度のエンドツーエンドコスト.ja.md)

**Asks directly about the axis itself in D4 (Checksum Location) settled item 1 (tension between checksum granularity and random small reads)**:
Inlined checksums equate the "smallest independently verifiable unit" to one unit G ⇒ reading 4 KiB out of it requires **reading the full G and verifying the full G**.
Sweeping G from 4 KiB to 128 KiB measures the end-to-end cost of reading 4 KiB of user data.

### Why Three Existing Numbers Cannot Answer It

| Experiment | Axis It Measures | Why Not This Axis |
|---|---|---|
| E6 (Encryption Algorithm Selection) | Encryption **throughput** (CPU) | Measures MiB/s, not "how long one must wait to read once"; excludes device |
| E20 (CPU Cache Cost of Widening Pointers) | **CPU cache** (node size 2K–64K) | Measures cache behavior during point lookups; distinct resource from read amplification |
| E7 (Offline Index Harness) | **Device I/O count** under fixed memory budget | Measurement basis is counts and hit rates, not "time paid per random small read" |

E6 (Encryption Algorithm Selection) measures encryption throughput, not end-to-end read latency;
D25 (Target Workload Priority) states "barrier measurement basis and byte measurement basis remain unmeasured to date". E58 (End-to-End Cost of Checksum Granularity) fills the two cells of bytes and time.

### What to Measure

| Arm | Workload | What G Affects |
|---|---|---|
| `rand` | Random 4 KiB point read, QD=1 (single-threaded) | Reads full G each time: device time + verification time |
| `randq` | Random 4 KiB point read, QD=16 (16 threads) | Reads full G each time, but bottleneck shifts from latency to bandwidth |
| `seq` | Sequential read 512 MiB, **fixed 1 MiB I/O**, only chunking verification by G | Leaves only fixed overhead per verification |
| `meta` | Pure arithmetic: 53-byte **parent-node-side** pointer payload per unit (D21 (Boundary Between Authoritative and Derived States) 2-replica measurement basis unpacked: pointer header 31 + position entry 22; unit header resides within unit and is included in reading G) | Dilution ratio 53/G |

⚠️ **`seq` I/O size is intentionally decoupled from G**: real sequential read I/O size is governed by readahead, not checksum granularity.
Allowing I/O to vary with G would credit "larger I/O is faster" to G; those are two decisions, not one.

### Pre-Run Hardcoded Criteria and Failure Clauses

1. **Positive control, run arm-by-arm**: `/proc/self/io` `read_bytes` delta recorded by the kernel must equal `ops × G` (tolerance ±2%).
   ⚠️ **Program-calculated `dev_bytes` does not constitute an observation**—it is derived directly from `ops × G`; comparing it to `ops × G` compares itself to itself.
2. **Discriminative power control**: effective user bandwidth between G=4 KiB and G=128 KiB on the `rand` arm must differ by ≥2×.
3. **Negative control**: after removing `O_DIRECT` and warming the page cache, `read_bytes` must collapse to less than 10% of direct tier,
   while program-calculated bytes remain completely unchanged—otherwise this verification path is an echo.
4. **Unreadable ≠ Read 0**: timing of 0, count of 0, or failure to fetch `read_bytes` scraps the entire round.
5. N=5 rounds; judging "Pass" requires passing all 5 rounds.
6. **Acceptance of reversed outcomes** (written down pre-run): 32 KiB total cost lower by ≥5% ⇒ record "32 superior to 16" and request reopening
   D8 (Core Index Structure) settled item 2; 16 KiB lower by ≥5% ⇒ record "16 retained"; difference <5% ⇒ record "indistinguishable on this axis", forbidding choosing either as better.

**Analytical prediction under byte measurement basis also hardcoded pre-run**: cost per 4 KiB user data
`cost(p, G) = p·(G + 53) + (1−p)·(4096 + 53·4096/G)` (formula corrected on 2026-09-02),
critical random small read share between 16 KiB and 32 KiB **p\* = 0.0404%** (pinned by unit test).

### Verdict on Three Controls: All Passed

| Control | Criterion | Measurement |
|---|---|---|
| Positive | `read_bytes / (ops×G)` ≈ 1 | **90/90 cells exactly 1.0000** |
| Discriminative power | `rand` 4K vs 128K diff ≥2× | QD=1 **2.62×**, QD=16 **9.35×** |
| Negative | `read_bytes` collapses after dropping `O_DIRECT` | Second pass **0.0000**, while program-calculated 67108864 / 134217728 tiers remain bit-identical |

⚠️ **Negative control is the sole element in E58 (End-to-End Cost of Checksum Granularity) proving "verification path is not an echo"**: direct tier shows cell-by-cell agreement between both paths,
and that agreement serves as evidence only after observing "kernel count zeroes while program count holds steady upon dropping direct tier".

### Host Measurements (5-Round Mean, Parentheses Show Coefficient of Variation)

| G | `rand` ns/op | `randq` User MiB/s | `seq` User MiB/s | Verification Share of `rand` Time |
|---|---|---|---|---|
| 4 KiB | 91607.5 (1.2%) | 690.78 (7.6%) | 2024.46 (0.6%) | 1.7% |
| 8 KiB | 147618.0 (0.3%) | 224.39 (0.6%) | 2040.51 (1.1%) | 1.9% |
| **16 KiB** | **154594.4 (0.9%)** | **196.14 (5.3%)** | **2045.53 (0.3%)** | 3.3% |
| **32 KiB** | **171031.9 (0.2%)** | **179.78 (0.5%)** | **2062.60 (0.3%)** | 5.1% |
| 64 KiB | 198904.4 (0.2%) | 145.45 (0.4%) | 2102.34 (0.9%) | 8.0% |
| 128 KiB | 240119.4 (0.3%) | 73.86 (0.4%) | 2123.93 (0.4%) | 12.5% |

**Item-by-item differences between 16 KiB and 32 KiB**:

| Side | 32 KiB Relative to 16 KiB |
|---|---|
| Random small read QD=1 | **10.63% more expensive** |
| Random small read QD=16 | **8.34% less user bandwidth** |
| Sequential read | 0.83% more user bandwidth — **originates entirely from verification, zero from device**, see section "That 0.83% on Sequential Side Belongs to AES Chunking, Not Device" |
| Pointer metadata dilution | Saves **0.16 percentage points** (0.323% → 0.162%, corrected measurement basis 2026-09-02) |

**Robustness** (same day, ops raised 4×, single round): QD=1 is 154878.9 vs 173182.0 ns/op (11.8% more expensive),
QD=16 is 203.183 vs 181.799 MiB/s (10.5% lower). **Neither direction nor magnitude changes with operation count.**

### That 0.83% on Sequential Side Belongs to AES Chunking, Not Device

⚠️ **Hit by backward leg of three-way argument on 2026-08-31; confirmed by main agent independent recomputation.**
`seq` arm I/O is fixed at 1 MiB (designed pre-run); G alters only verification chunking ⇒ **device side is decoupled from G by construction**,
as borne out by measurements:

| G | Total Latency ms | Verification ms | **Device Side ms** | Device Side Relative to 4 KiB |
|---|---|---|---|---|
| 4 KiB | 252.9 | 131.5 | **121.37** | 1.0000 |
| 16 KiB | 250.3 | 128.5 | **121.78** | 1.0033 |
| 32 KiB | 248.2 | 126.9 | **121.37** | 1.0000 |
| 128 KiB | 241.1 | 119.2 | **121.85** | 1.0039 |

Device side across all six tiers falls into 1.0000–1.0049; inter-round range itself is 1–3 ms, same order as inter-tier difference.
⇒ **Sequential read 0.83% is entirely "reduced AES call count", containing zero device contribution.**

⚠️ **Corollary**: real implementation overlaps verification with subsequent I/O (or offloads to other cores).
**After overlapping, 32 KiB gains on sequential side approach zero**, while costs on random side remain unchanged
⇒ Critical random small read share collapses toward 0; **16 KiB saves more across any positive random small read share.**

### Critical Random Small Read Share: Two Time-Basis Numbers Deprecated, Forbidden from Citation

Let p = random small read share in operations, remainder sequential reads; cost computed per 4 KiB user data:

| Measurement Basis | Break-even p | Winner Above It |
|---|---|---|
| Byte (pre-run analytical formula, corrected 2026-09-02) | **0.0404%** | 16 KiB |
| Time, QD=1 | **0.0961%** | 16 KiB |
| Time, QD=16 | **0.8648%** | 16 KiB |

⚠️ **"Two paths arrive at the same destination" is revoked** (backward leg 2026-08-31; confirmed by main agent recomputation).
The two numerators reflect **different mechanisms**: byte side is 53-byte parent-node-side pointer payload dilution per unit (0.16 percentage points),
time side is AES chunking (0.83%); magnitudes differ by 2.5×. **Coinciding closely does not constitute mutual proof.**

⚠️ **Two numbers under time measurement basis must not be written with three significant digits**: round-by-round recomputation yields
**0.0698% / 0.0901% / 0.1214% / 0.0769% / 0.1286%** (QD=1),
**range is 60% of mean**. Meanwhile **80.4%** of the numerator is AES chunking—stripping it causes one round to **flip sign**
(in that round 16 KiB wins across any p).
⇒ **Critical share under time measurement basis is not a citable figure**; it is the "difference between AES and device ledgers",
of which only one ledger truly connects to G.
**"32 KiB requiring p > 1.262% to become 5% more expensive" also hangs on this curve and is downgraded concurrently.**

**Only critical share under byte measurement basis survives** (analytical formula + unit test pinned, corrected 2026-09-02 to **0.0404%**), independent of any timing.

### After Stripping Verification Time, Cost Remains on the Device Side

`verify_ns` is metered separately; subtracting it isolates device side:

| G | Total ns/op | Verification ns/op | Device Side ns/op | Device Side Step Increase |
|---|---|---|---|---|
| 4 KiB | 91607.5 | 1597.3 | 90010.2 | —— |
| 8 KiB | 147618.0 | 2835.4 | 144782.6 | +60.9% |
| **16 KiB** | 154594.4 | 5150.0 | **149444.4** | +3.2% |
| **32 KiB** | 171031.9 | 8710.8 | **162321.1** | **+8.6%** |
| 64 KiB | 198904.4 | 15906.1 | 182998.3 | +12.7% |
| 128 KiB | 240119.4 | 29964.6 | 210154.8 | +14.8% |

⇒ Out of the 10.63% from 16 → 32, **8.6 percentage points are device, 2 percentage points are verification**.
Swapping in a faster disk eliminates only a fraction of the 8.6 device percentage points, incapable of cutting the 2 verification percentage points—verification scales linearly with G and is device-independent.

### The 4 KiB -> 8 KiB Step: Not a Harness Artifact, Falls on Device Side

**On QD=1, 4 KiB → 8 KiB jumps 61%, while 8 KiB → 16 KiB increases by only 4.7%**, thereafter climbing linearly 10–21% per tier.
On 2026-08-31 backward leg independently reproduced on identical file using **fio 3.36** (zero shared lines of code):
Ascending 67.8 → 147.2 μs (+117%), descending 67.4 → 146.6 ⇒ **independent of arm order**;
switching to incompressible data or altering offset alignment leaves it unchanged. Two of three mutually exclusive mechanisms rejected:

| Hypothesis | Discriminative Observation | Verdict |
|---|---|---|
| Host / kernel single-page fast path | Step should persist when working set shrinks to 64 KiB (within device cache) | **No**: 4 KiB 11.9 μs, 8 KiB 12.0 μs, ratio 1.008 |
| **Device-side 4 KiB fast path** (single L2P entry / single ECC codeword) | Step should catch after 4096 and be alignment-independent | **Survives**: `bs=4608` reaches full step; changing `ba` leaves unchanged |
| Harness artifact | Switch tools, order, data contents | **No**: all three fio variants reproduce |

⚠️ **Step magnitude is not a device constant**: 8 KiB / 4 KiB ratio shifts with working set—
footprint 64 KiB yields 1.008, 1 MiB yields 1.19, 64 MiB yields 1.21, 8 GiB yields 1.54–2.15.
The "stable 5-round +60.9%" recorded in E58 (End-to-End Cost of Checksum Granularity) represents figures under **that device state and that working set**.
⚠️ **Does not affect 16 vs 32 comparison**: fio gives 32/16 = **1.090** on the identical host and file,
consistent with 1.086 on device side after stripping verification—the step and 16-vs-32 are mutually independent observations.

### Measurement Basis

Runs directly on host, bypassing VM (⇒ **no block-layer independent readings**, verification path is `/proc/self/io`).
On local NVMe (KINGSTON SNV3S4000G, 3.6 TB, **DRAM-less**) inside an 8 GiB file on ext4,
`O_DIRECT`, filled pre-run with non-zero patterns (sparse holes might avoid touching device entirely).
Verification uses AES-256-GCM (`aes-gcm` 0.11.1, following `encrypt_in_place_detached` from E6 (Encryption Algorithm Selection)), host supports AES-NI.
Randomization is uniformly distributed across full space, without hotspots; single disk, no RAID, no concurrent writes.

Code `scripts/experiment-apparatus/e58_csum_grain.rs` (8 unit tests, 10 mutations each proven to turn red,
`bash scripts/mutation/mutate.sh e58-csum-grain scripts/experiment-apparatus/e58_csum_grain.rs scripts/mutation/tables/e58_csum_grain.tsv`).

**Re-run** (5 rounds = 5 seeds, test file path read from environment variable, forbidden from repository commit):

```bash
cd scripts/experiment-apparatus && cargo build --release --bin e58-csum-grain
for s in 1 2 3 4 5; do
  ./target/release/e58-csum-grain "${E58_IMG:-${TMPDIR:-/tmp}/e58.img}" $s none 4096 8192
done                        # Artifact e58-csum-grain-2026-08-31.out
./target/release/e58-csum-grain "${E58_IMG:-${TMPDIR:-/tmp}/e58.img}" 1 nodirect 4096 8192   # Negative control, run twice
./target/release/e58-csum-grain "${E58_IMG:-${TMPDIR:-/tmp}/e58.img}" 7 none 16384 8192      # Robustness
```

Raw outputs `e58-csum-grain-2026-08-31.out` (5 rounds),
`e58-csum-grain-nodirect-2026-08-31.out` (negative control),
`e58-csum-grain-ops16k-2026-08-31.out` (robustness),
`e58-csum-grain-repro-2026-08-31.out` (single round, run against 2026-08-31 source),
`e58-csum-grain-repro-2026-09-16.out` (single round, for `replay.sh` byte-by-byte differential).
⚠️ **`meta_*` and `model_*` rows in 2026-08-31 outputs computed with 108-byte pointer payload per unit**; source updated 2026-09-02 following
C89 (Leftover Recomputation from Closing the Unit Header) item 1 to 53 (unit header resides within unit and is already inside G; computing with 108 double-counts it); artifacts were not re-run,
so `replay.sh` judged "mismatch" on full run tier, while default tier skips timing experiments and did not observe it. Single round re-run 2026-09-16 packaged as
`-repro-2026-09-16.out` (`ptr_bytes=53`, `meta_g32768 meta_ratio=0.001617`, matching 0.162% in prose);
`replay.sh` row repointed to it, all four interval assertions passed. **Timing rows (`rand` / `randq` / `seq`) exclude pointer payload and remain unaffected by this correction**,
so percentages in timing tables of E58 (End-to-End Cost of Checksum Granularity) stand as before.

**Integrated into `scripts/experiment-apparatus/replay.sh`**, criterion in table is `timing`; re-run full table with `bash scripts/experiment-apparatus/replay.sh`.
Because timing experiments cannot reproduce identical bytes upon re-run, four interval assertions pin down figures from kb:
Random small read QD=1 ratio 1.03–1.25, QD=16 bandwidth ratio 1.03–1.25,
**32 KiB read amplification exactly 8.0** (absolute value, computed independently as 32768/4096),
and positive control `read_bytes ÷ (ops×G)` falls within 0.98–1.02.
⚠️ **Last assertion proven to go red**: negative control artifact yields **0.0000** in identical cell, falling outside interval.

### A Harness Defect: N=5 Actually Had Only 3 Seeds

`let mut st = seed | 1` ⇒ **seed 2 and 3 collapse into the same state; seed 4 and 5 collapse into the same**
(confirmed 2026-08-31 by recomputing 1024 offsets for each of 5 seeds, identical one by one).
⇒ Nominal N=5 is actually **3 access patterns ×(1,2,2) repetitions**; **recorded inter-round variation (0.2–7.6%) is systematically understated**.
E65 (Write-Side Cost of Checksum Granularity) and E66 (How Small Files Land After Padding) share identical formulation.
Placement in [checks-owed.md](../checks-owed.md) C59 (Seeds Folding into the Same State).
⚠️ **Modifying it requires re-running and updating artifacts**—`87-replay` does byte-by-byte differential; modifying source leaves it red until re-run.

### What It Cannot Answer

Does not touch crash consistency, write path, or RAID; swapping algorithm (ChaCha20-Poly1305) requires re-running;
**changing devices requires re-running**—curve shape reflects properties of this consumer-grade DRAM-less NVMe; mechanical disks and enterprise SSDs exhibit different fixed overhead shares;
⚠️ **changing working set also requires re-running**: backward leg 2026-08-31 measured 32 KiB / 16 KiB ratio shifting with working set—
8 GiB (used in E58 (End-to-End Cost of Checksum Granularity), coldest) 1.117–1.135, 1 GiB 1.154–1.159,
**16 MiB hotspot set 1.300–1.307** ⇒ **cost of 32 KiB under hotspot workload is roughly 2.5× higher than recorded here**;
⚠️ **`randq` arm includes fixed overhead decoupled from G**: 16 threads each execute `open()` + AES key schedule + spawn,
inferred via fio steady state at roughly **8.6 ms/arm**, taking larger share in small G tiers ⇒ **bias direction flatters 32 KiB**,
deducting it shifts 16→32 from 1.115 to 1.129 (fio independently yields 10.0%);
does not model readahead hits, nor cache hits under skewed distributions;
`meta` arm is arithmetic rather than host measurement, relying on 108-byte measurement basis from D21 (Boundary Between Authoritative and Derived States).

⚠️ **Duplicate accounting in `meta` arm corrected (2026-09-02, discovered during C68 (Invariant Allowlist Conflicts with Settled Clauses) adversarial round,
formula re-pinned same day)**: original formula added entire 108-byte pointer payload outside G, whereas 55-byte unit header resides inside unit
and is already included in reading G—post-correction extra cost is constant 53 bytes on parent node side per unit, **independent of unit header width** (header width only modifies container threshold in E43 (Extension Point Byte Upper Bound),
not entering read amplification). Critical share 0.0823% → **0.0404%**, dilution gain 0.33 → 0.16 percentage points;
8 unit tests re-pinned, all 10 mutations caught again (`scripts/mutation/tables/e58_csum_grain.tsv`).
**Formulas and readings for host measurement arms (rand / randq / seq) remain unaltered by a single character**; replay interval assertions unaffected;
artifacts retain 2026-08-31 version (legacy values in meta rows preserved alongside, faithfully recording pre-correction source).

### Three-Way Argument: All Three Legs Finished Running (2026-08-31)

| Leg | Stance | Conclusion |
|---|---|---|
| Local (Qwen3-Next-80B, English prompt, both corruption detectors judged Green) | Attack | **Failed to hit primary conclusion**; suggested "AES fixed overhead per call" refuted by data |
| Cloud A (Sonnet) | Forward deduction | Primary chain holds; notes **critical share under time measurement basis absent from any script or unit test**, and D4 (Checksum Location) settled item 3 makes no distinction between "first write" and "modifying existing" |
| Cloud B (Opus) | Counterfactual / Find counterexample | **Hit three items**: sequential 0.83% is AES not device, round-by-round range of critical share under time measurement basis is 60%, 11 percentage points of `full` 25.92% in E65 (Write-Side Cost of Checksum Granularity) are AES |

**Divergence across three parties adjudicated by main agent per `.claude/rules/three-way-inference.md`, based on recomputation rather than voting**:
Three items raised by backward leg were **fully corroborated** by main agent via independent round-by-round recomputation against raw artifacts, and adopted into prose.
Two points raised by forward leg also accepted: critical share downgraded, and D4 (Checksum Location) settled item 3 gap noted in pending settlements.

**Unchallenged items** (unanimous across three legs and corroborated by independent tooling): QD=1 device-side cost
(fio 3.36 independently yields 1.090, vs 1.086 on device side after stripping verification), negative control, analytical critical point under byte measurement basis.

Four points from local leg judged individually:

| What It Said | Verdict |
|---|---|
| "If random small reads truly stay below 0.1%, 32 KiB will win" | **Holds, and agrees with measurements**—critical point 0.0961% (QD=1) is precisely this figure. Not a rebuttal; it merely restates the applicable boundary of the conclusion |
| "4 KiB→8 KiB step may be AES fixed overhead per invocation" | **Refuted by E58 (End-to-End Cost of Checksum Granularity) numbers**: device side remains **+60.9%** after stripping `verify_ns`, while verification accounts for only 1.7% / 1.9% in 4 KiB / 8 KiB tiers |
| "Isolate device and verification first using unverified raw reads" | **Already done**, see Device Side column in section "After Stripping Verification Time, Cost Remains on the Device Side" |
| "Large granularity wins clearly under media streaming / analytics / backup workloads" | Holds only when p < 0.1%. D25 (Target Workload Priority) classifies rand as **secondary workload rather than zero** ⇒ establishing this requires first proving p < 0.001, for which this project lacks evidence |

### Affected Decisions

| Decision Item | Relationship | Retrospective |
|---|---|---|
| D4 (Checksum Location) settled item 1 | Supports | 2026-10-21 unaffected: only the trilingual switch row was added; previously, 2026-10-11 unaffected: wording only (temporal terms, locational references, self-referential terms), conclusions, figures, and artifacts unchanged |
| D4 (Checksum Location) settled item 8 | Supports | 2026-10-21 unaffected: only the trilingual switch row was added; previously, 2026-10-11 unaffected: wording only (temporal terms, locational references, self-referential terms), conclusions, figures, and artifacts unchanged |
| D4 (Checksum Location) settled item 7 | Supports | 2026-10-21 unaffected: only the trilingual switch row was added; previously, 2026-10-11 unaffected: wording only (temporal terms, locational references, self-referential terms), conclusions, figures, and artifacts unchanged |
| D4 (Checksum Location) settled item 3 | Preparation | 2026-10-21 unaffected: only the trilingual switch row was added; previously, 2026-10-11 unaffected: wording only (temporal terms, locational references, self-referential terms), conclusions, figures, and artifacts unchanged |
| D8 (Core Index Structure) settled item 2 | Does not affect | 2026-10-21 unaffected: only the trilingual switch row was added; previously, 2026-10-11 unaffected: wording only (temporal terms, locational references, self-referential terms), conclusions, figures, and artifacts unchanged |
| D21 (Boundary Between Authoritative and Derived States) | Does not affect | 2026-10-21 unaffected: only the trilingual switch row was added; previously, 2026-10-11 unaffected: wording only (temporal terms, locational references, self-referential terms), conclusions, figures, and artifacts unchanged |
| D25 (Target Workload Priority) | Does not affect | 2026-10-21 unaffected: only the trilingual switch row was added; previously, 2026-10-11 unaffected: wording only (temporal terms, locational references, self-referential terms), conclusions, figures, and artifacts unchanged |

## Historical Versions

Change history of E58 (End-to-End Cost of Checksum Granularity) is consolidated in [experiments-history.md](../experiments-history.md).
