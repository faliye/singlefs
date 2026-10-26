## E6 Encryption Algorithm Selection —— Run (2026-08-31 Retested two CPU feature tiers, N=5; 2026-08-28 multicore scaling tier)

[中文](06-加密算法选型.md) · **English** · [日本語](06-暗号化アルゴリズム選定.ja.md)

**Conclusion**: **There is no single winner; speed reverses with AES-NI, and by a substantial margin.**
With AES-NI, AES-256-GCM is **2.59–3.07× faster**; when AES-NI is disabled, ChaCha20-Poly1305 is **4.88–6.47× faster**.
⇒ The answer for [decisions.md](../decisions.md) D9 (Encryption) settled item 3 is **select at runtime based on CPU features**, not pinning a single choice in the format.

**Measurement Basis**: QEMU + KVM, host CPU exposing `aes`; guest side tested at two tiers: `-cpu host` and `-cpu host,-aes,-vaes`.
At each tier, each algorithm encrypts 256 MiB for each unit size, taking the best value across 5 rounds (inter-round dispersion ≤0.47%).
Single-core, **in-place encryption (`encrypt_in_place_detached`), buffers preallocated and warmed up page-by-page**, without allocations inside the loop—
otherwise the benchmark measures the allocator and page faults, not the algorithm. `RustCrypto`'s `aes-gcm` 0.11.1 and `chacha20poly1305` 0.11.0.
Code in `scripts/experiment-apparatus/e6_units.rs` (7 unit tests, 9 mutations demonstrated one-by-one to turn red),
raw output `e6-units-2026-08-31.out`.
Rerun: `bash scripts/experiment-apparatus/e6-units.sh 5` (each tier runs the VM once, cross-tier criteria judged by script).

### Measured (Single-Core Throughput, MiB/s, 2026-08-31, N=5 Best Value)

| Unit | AES with NI | AES Disabled | ChaCha with NI | ChaCha Disabled |
|---|---|---|---|---|
| 4 KiB | **3656.13** | **244.74** | 1192.00 | 1195.08 |
| 16 KiB | 3842.83 | 248.23 | 1483.29 | 1492.12 |
| 64 KiB | 4117.34 | 249.31 | 1582.12 | 1585.14 |
| 256 KiB | 4209.66 | 249.57 | 1608.29 | 1614.11 |

**Winner**: With AES-NI → AES is 2.59× faster (16K) to 3.07× faster (4K);
Without AES-NI → **ChaCha is 4.88× faster (4K) to 6.47× faster (256K)**.

### ⚠️ Masking only `aes` without masking `vaes` measures a set of **miscalculated** numbers

`aes` 0.9.3 uses `cpufeatures` to **separately** probe the two CPUID bits `aes` and `vaes`
(the `features_vaes256, "vaes"` section in `src/lib.rs`, inspected from the host registry on 2026-08-31).
`-cpu host,-aes` only strips the former ⇒ the guest still sees VAES:

| Observation | `-cpu host,-aes` | `-cpu host,-aes,-vaes` |
|---|---|---|
| AES 4 KiB throughput | **3747.63** (differs by 0.5% from NI tier, masking invisible) | **244.74** (drops 14.94×) |
| Known-answer tag for AES | **`2112edc9…`, mismatches external implementation** | `62d27233…`, matches |
| `tagchk` vs with-NI tier | **All 8 cells different** | **All 8 cells bit-for-bit identical** |

⇒ **That tier ran blazing fast while not actually computing AES-256-GCM**; not a single word of its throughput numbers can be used,
even though it looked entirely normal in isolation—two controls (known answer, `tagchk` identical across tiers) independently caught it.

**Expected known-answer values come from an external implementation**: `python3 -c` invoking `cryptography` 49.0.0 (OpenSSL backend),
sharing zero lines of code with this binary. This is the sole check in E6 (Encryption Algorithm Selection) that does not rely on self-consistency—
other checks can only say "mutually consistent", unable to say "which side is correct".

### Comparison with 2026-08-27 Round: Direction and Magnitude Both Align

| Unit | ChaCha with NI (08-27 → 08-31) | AES with NI (08-27 → 08-31) |
|---|---|---|
| 4 KiB | 1192.00 → **1192.00** (bit-for-bit identical) | 3888.51 → 3656.13 (−6.0%) |
| 16 KiB | 1489.67 → 1483.29 (−0.4%) | 4097.12 → 3842.83 (−6.2%) |
| 64 KiB | 1589.36 → 1582.12 (−0.5%) | 4126.31 → 4117.34 (−0.2%) |
| 256 KiB | 1616.59 → 1608.29 (−0.5%) | 4136.52 → 4209.66 (+1.8%) |

⇒ **The conclusions of the 08-27 table reproduce on 08-31**, and from 08-31 onward it possesses code, artifacts, and rerun commands—
prior to 08-31 none of the three existed (under `research/results/` there was only output from the 08-28 multicore round).
⚠️ The disabled tier on 08-27 recorded 267.55 MiB/s, while 08-31 was 244.74 (−8.5%).
**Whether the 08-27 run also fell into the VAES pitfall cannot be answered**—that round's code is not in the repository.
The two sets of figures are close, but "close" does not constitute evidence that it computed correctly at the time.

### Multicore Scaling Tier (2026-08-28 supplementary run, on host, bypassing VM)

**Why supplemented**: [decisions.md](../decisions.md) D24 (Offloading Heavy Background Work to GPU) once used "single core 3888 MiB/s × 16 cores ≈ 65 GB/s"
to reject GPU offload, while E6 (Encryption Algorithm Selection) prior to the supplementary run explicitly noted "not multicore scaling ratio... unmeasured this round". That extrapolation lacked basis.

**AES-256-GCM saturates at 8 threads and stops increasing** (each thread having a 64 MiB independent buffer ⇒ working set far exceeds L3):

| Threads | 1 | 2 | 4 | 8 | 16 | 32 |
|---|---|---|---|---|---|---|
| AES-256-GCM (Round 1) | 3.6 | 6.3 | 10.8 | **17.4** | 16.3 | 16.5 GB/s |
| AES-256-GCM (Rerun) | 3.5 | 6.8 | 11.0 | 16.6 | 17.1 | **17.2** GB/s |
| Speedup (Rerun) | 1.00 | 1.95 | 3.17 | 4.76 | 4.91 | 4.93 |
| ChaCha20-Poly1305 (Round 1) | 1.2 | 2.4 | 4.4 | 7.5 | 12.8 | **16.2** GB/s |
| Speedup (Round 1) | 1.00 | 1.96 | 3.62 | 6.25 | 10.61 | **13.48** |

⚠️ **Which thread tier peaks between rounds fluctuates** (Round 1 peaked at 8 threads, rerun peaked at 32 threads),
but **the saturation value remains stable at 16.5–17.4 GB/s**. ⇒ When citing, use "**≈17 GB/s saturation**";
do not cite single-point values from a specific thread tier.

**Cache-resident tier (1 MiB per thread) control**: AES 16 threads reaches **40.5 GB/s**, speedup 10.98×
⇒ **The bottleneck is indeed on the memory side, not the algorithm itself.**

**⇒ Three Conclusions**:

1. **The "single core × core count" extrapolation was off by nearly 4×**: linear extrapolation gave ≈ 65 GB/s, while measurement showed **17.4 GB/s**.
2. **AES and ChaCha converge on multicore**: at 32 threads, 16.5 vs 16.2 GB/s,
   ⇒ The single-core advantage in E6 (Encryption Algorithm Selection) of "AES is 2.6–3.3× faster with NI" **substantially disappears once saturated**.
   ⚠️ This does not overturn D9 (Encryption) settled item 3 "select at runtime based on CPU features"—the gap remains on single-core and under low concurrency.
3. **AES peaks at 8 threads and drops slightly at 16/32 threads**, while ChaCha scales all the way to 32 threads
   ⇒ The two encounter different bottlenecks: AES hits memory/execution units early, whereas ChaCha is compute-bound.

### Two Positive Controls, the First Caught a Real Bug

| Control | Assertion | Result |
|---|---|---|
| Single-core reproduction of E6 (Encryption Algorithm Selection) | 4 KiB tier single-core throughput differs by ≤20% from E6 (Encryption Algorithm Selection)'s 3888.51 MiB/s (E6 (Encryption Algorithm Selection) in VM, reproduction on host) | **3462.54 MiB/s, diff 11.0%**, pass |

⚠️ **The baseline 3888.51 used for that control was from the 08-27 round**; retesting on 2026-08-31 yielded 3656.13;
under the new baseline the same observation differs by 5.3%, and **the control still passes**. Multicore artifacts and conclusions remain untouched.
| Compute-bound arm | Fixed total workload, 16-thread speedup ≥4× | **15.31×**, pass |

⚠️ **The first control initially judged red twice (2154 / 2024 MiB/s, diff 45–48%), and the flaw was in the harness, not the algorithm**:
`vec![0u8; n]` provides **lazily mapped zero pages**, while warm-up only touched the first and last two bytes ⇒ **page faulting inside the timed section**.
Changing to page-by-page warmup immediately restored performance to 3462.
⚠️ `RUSTFLAGS="-C target-cpu=native"` was tested in between and **proved useless** (2023 vs 2154, within noise)—
indicating it was not an issue with compile-time target features, eliminating that path.

⇒ **Lesson: A control "reproducing existing measured values" catches harness bugs far better than any self-consistency check.**
It does not judge whether an algorithm is fast or slow; it only judges "whether what I am measuring is the same thing as last time".

**Measurement Basis**: Direct run on host (bypassing VM—only disabling AES-NI requires a VM; multicore scaling does not).
4 KiB units, in-place encryption, buffers preallocated **and warmed up page-by-page**, no allocations in loop.
3 rounds per tier taking the best value. `aes-gcm` 0.11.1 / `chacha20poly1305` 0.11.0, same versions as E6 (Encryption Algorithm Selection),
**and deliberately adhering to the deprecated API `encrypt_in_place_detached` used by E6 (Encryption Algorithm Selection)**—switching APIs switches the tested object.
Code in `scripts/experiment-apparatus/e6_multicore.rs` (4 unit tests, 4 mutations demonstrated one-by-one to turn red),
output `e6-multicore-2026-08-28.out`.

### Unit Size: Gains Substantially Saturate Beyond 16 KiB

Enlarging from 4 KiB to 64 KiB increases throughput by only **12.6% (AES) / 32.7% (ChaCha)**;
enlarging further to 256 KiB yields only 2.2% / 1.7% more (2026-08-31 retest; 08-27 round was 6.1% / 33.3% and 0.2% / 1.7%).

⚠️ **This is one of the directional bases for D4 (Checksum Location) settled item 1 (tension between checksum granularity and random small reads)**:
Enlarging the encryption/checksum unit from 4 KiB to 64 KiB **buys throughput gains from single digits to 30%**,
while the volume forced to be read and verified by random small reads is **16×** ⇒ **large units buy no throughput on the CPU side—holding only for hot buffers**: E6 (Encryption Algorithm Selection) buffers are preallocated and warmed page-by-page; on the random small read path, the buffer is overwritten each time by `O_DIRECT` DMA and remains constantly cold; according to the 5-round average of `dev_bytes ÷ verify_ns` on the `rand` arm in E58 (End-to-End Cost of Checksum Granularity) artifact `e58-csum-grain-2026-08-31.out`, AES throughput from 16 → 32 KiB is 3.18 → 3.76 GB/s (+18.2%); under cold buffers large units buy a modest amount of throughput, yet this is far smaller than device-side cost on the same path (E58 (End-to-End Cost of Checksum Granularity) device-side breakdown table 149444.4 → 162321.1 ns, adding 12.9 μs each time, +8.6%), direction unchanged.
⚠️ **It did not fix the value**: D4 (Checksum Location) settled item 1 takes **32 KiB** by user settlement, in the opposite direction,
with cost measured by E58 (End-to-End Cost of Checksum Granularity) and recorded in the body text of D4 (Checksum Location).
The throughput numbers measured by E6 (Encryption Algorithm Selection) remain valid; what it proves is that "32 KiB buys nothing in CPU throughput" (holding only for hot buffers), not that "32 KiB should not be chosen".

### Negative Controls Span the Full Sweep, Not Merely Holding at One Point

On the four unit tiers, ChaCha20-Poly1305 was affected by disabling by **+0.26% / +0.60% / +0.19% / +0.36%**
(2026-08-31). It does not use AES-NI, so it must remain unaffected—
all four tiers holding proves that the CPU flag acted only where it was supposed to act, rather than slowing down the entire machine.

**`tagchk` (sum of first bytes of authentication tags) is bit-for-bit identical across eight cells between tiers**
(AES 9764864 / 1130496 / 397312 / 36864, ChaCha 12124160 / 2818048 / 270336 / 2048)
⇒ Both tiers ran the same computation; the difference was only in speed.
⚠️ **This item is not decorative**: the tier masking only `aes` revealed its flaw right here—all eight cells differed.

### Verdict on Failure Clauses: Passed, Accompanied by a Built-in Negative Control

The failure clause of E6 (Encryption Algorithm Selection) is "masking AES-NI has no distinguishable effect on AES-GCM numbers ⇒ CPU flag did not genuinely take effect, entire round invalidated".
Measurement showed AES-GCM dropped by **14.94 / 15.48 / 16.51 / 16.87×** (four tiers, 2026-08-31), far exceeding any noise.
⚠️ **This clause genuinely triggered a red verdict once on 2026-08-31**, the red round being precisely the one masking only `aes`—
see the section "Masking only `aes` without masking `vaes`". **The clause was enforced as written; the entire round was invalidated without backpedaling on criteria.**

**Meanwhile ChaCha20-Poly1305 remained completely unchanged between tiers (4K tier differed by 0.26%)**—it does not use AES-NI,
so it **must** remain unaffected. This is an unarranged yet valid negative control:
it proves that the CPU flag not only took effect, but also **acted only where it was supposed to act**, rather than slowing down the whole machine.

### Triggered Second Criterion: Encryption Must Be Parallelizable

When E6 (Encryption Algorithm Selection) was initiated, it specified "if single-core throughput of both falls below single-disk sequential bandwidth, encryption must be parallelized—
that would alter write path structure, belonging to results that change what the first segment of code looks like, and must be written back to D9 (Encryption)".

**Without AES-NI, AES-GCM achieves only 244.74 MiB/s (4 KiB units, 2026-08-31)**, while any modern NVMe sequential bandwidth exceeds 1 GB/s
(host nvme0n1 3.6 T, sequential bandwidth unmeasured directly—this statement is deduced from media category, not measured).
⇒ **On machines without AES-NI, single-core encryption is a bottleneck; the encryption path must support parallelism.**

### What E6 (Encryption Algorithm Selection) Cannot Prove

- **Not multicore scaling ratio.** The two items "multicore scaling ratio" and "gate wall-clock increment" listed when initiating E6 (Encryption Algorithm Selection) were unmeasured.
- **Does not cover decryption and authentication failure paths.**

**Prerequisites**: None (fully completed).

### Impacted Decisions

| Decision Item | Relationship | Review |
|---|---|---|
| D4 (Checksum Location) settled item 1 | Support | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D9 (Encryption) settled item 3 | Support | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D21 (Boundary Between Authoritative and Derived States) settled item 18 | Support | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D8 (Core Index Structure) settled item 2 | Support | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D24 (Offloading Heavy Background Work to GPU) | Preparation | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |

## Historical Versions

E6 (Encryption Algorithm Selection) history entries are consolidated in [experiments-history.md](../experiments-history.md).
