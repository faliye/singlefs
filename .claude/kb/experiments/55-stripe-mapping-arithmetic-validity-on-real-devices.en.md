## E55 Stripe Mapping Arithmetic Validity on Real Devices — Run (2026-08-30, Real dm-stripe Device): **72/72 Cells Correct**, All 6 Prime Step Size Groups Landed on Mutually Distinct Disks

[中文](55-条带映射的算术在真设备上对不对.md) · **English** · [日本語](55-ストライプマッピング算術の実デバイス上での正当性.ja.md)

**It completes the half that E53 (Can Root Ring Mount After Losing an Entire Disk) explicitly noted it did not test.**
All geometric conclusions of E46 (Root Ring Region Spacing and Failure Domains) / E48 (Feasible Points for Root Ring Geometry) rest on one piece of arithmetic:
logical address `p` falls on disk `(p / chunk) % devs`.
Before E55 (Stripe Mapping Arithmetic Validity on Real Devices), **that arithmetic had only been read in Linux source (`raid0.c` / `raid5.c`), never measured on devices**;
and inside the VM of E53 (Can Root Ring Mount After Losing an Entire Disk), there were 4 raw disks **without striping**,
regions placed manually 1:1 ⇒ it tested "consequences", not "premises".

### What to Measure

On a **real striped device** (`dm-stripe` built over 4 loop devices):

1. **Mapping**: Write a unique signature to logical address `k × chunk`, inspect which **backing file** it landed in,
   comparing against `k % devs`. Scans devs ∈ {2,3,4} × chunk ∈ {64 KiB, 512 KiB} × 12 values of `k`.
2. **Prime step size**: Place region r at `r × P × chunk` (P = 13, prime and greater than device count),
   inspecting whether R regions land on R **mutually distinct** disks.

### Criteria (Hardcoded Before Run)

| # | Criterion |
|---|---|
| 1 | **Every cell** `measured disk == k % devs`. Any cell mismatch ⇒ **address arithmetic of E46 (Root Ring Region Spacing and Failure Domains) / E48 (Feasible Points for Root Ring Geometry) invalidated on the spot** |
| 2 | Under prime step size, R regions land on **R mutually distinct** disks (devs ≥ R) |

### What Constitutes Failure (Hardcoded Before Run, Forbidden to Change After Run)

| Clause | Judgment When Triggered |
|---|---|
| **Positive control**: When step size takes **non-prime** 8 (= 2³), four regions on devs=4 must **collide on the same disk** | Do not collide ⇒ measurement cannot distinguish primes from non-primes, **entire round aborted** |
| Any mapping cell mismatches | **Faithfully record and invalidate that arithmetic**—the single most valuable possible outcome of E55 (Stripe Mapping Arithmetic Validity on Real Devices) |
| `dmsetup` fails to build | Record `name=skip`, **forbidden to treat as passed** |

### Actual Measurement (2026-08-30)

| Combination | Matched Mapping Cells | Mutually Distinct Disks Under Prime Step Size |
|---|---|---|
| 2 disks / 64 KiB | **12 / 12** | 2 / 2 |
| 2 disks / 512 KiB | **12 / 12** | 2 / 2 |
| 3 disks / 64 KiB | **12 / 12** | 3 / 3 |
| 3 disks / 512 KiB | **12 / 12** | 3 / 3 |
| 4 disks / 64 KiB | **12 / 12** | **4 / 4** |
| 4 disks / 512 KiB | **12 / 12** | **4 / 4** |
| **Total** | **72 / 72** | All 6 groups all_distinct |

**Positive control**: When step size was 8 (non-prime), `distinct_devices = 1` across four regions on devs=4
——**all collided on the same disk**, across both chunk sizes. ⇒ **Measurement separates primes from non-primes; those 6 "all distinct" results were not given for free.**

### Conclusions

1. **The arithmetic `(p / chunk) % devs` holds on real striped devices** (72 cells all correct)
   ⇒ Address arithmetic of E46 (Root Ring Region Spacing and Failure Domains) / E48 (Feasible Points for Root Ring Geometry) **is no longer merely read from source code**.
2. **Prime step size distributing regions across mutually distinct disks reproduces on real devices**
   ⇒ **The half explicitly left untested in E53 (Can Root Ring Mount After Losing an Entire Disk) is now filled**:
   both premise (regions land on distinct disks) and consequence (mounts after losing 1 disk, rollback ≤ 1) possess real-device evidence.
3. ⚠️ **Still untested**: **physical failure domains** (controller / backplane / power supply / firmware batch)—requires real hardware, infeasible on host.

### Measurement Basis and Known Limitations

- Script `scripts/devices/stripe-map-probe.sh` (`bash scripts/devices/stripe-map-probe.sh`),
  raw output `e55-stripe-map-2026-08-30.out`.
- ⚠️ **Tested `dm-stripe`, not `md/raid0`**. Source code cited in E46 (Root Ring Region Spacing and Failure Domains) was `raid0.c` / `raid5.c`,
  whereas E55 (Stripe Mapping Arithmetic Validity on Real Devices) built device-mapper stripe targets—**both are implementations of the same layout**,
  and E55 (Stripe Mapping Arithmetic Validity on Real Devices) proves dm-stripe matches the formula. **md/raid0 was not tested.**
- ⚠️ **P took 13 instead of 8191 used by E48 (Feasible Points for Root Ring Geometry)**: 8191 × 64 KiB ≈ 536 MB,
  requiring 1.6 GB for four regions, exceeding probe disk capacity. **13 equally satisfies the mechanism requirement "prime and greater than device count"**,
  whereas E48 (Feasible Points for Root Ring Geometry) chose 8191 to satisfy **spacing ≥ `io_min`**—a separate criterion untouched by E55 (Stripe Mapping Arithmetic Validity on Real Devices).
- ⚠️ **Ran over loop + dm, not raw physical disks**: tests **address mapping**, not media behavior.
- Used `sudo -S` via stdin for permissions without printing passwords; ⚠️ **`sudo -S` consumes stdin**,
  so data to write went uniformly through files rather than pipes—the initial probe version tripped here (written marker consumed as password,
  all 12 cells returned "none"). Cleanup handled on EXIT: `dmsetup remove` / `losetup -d` / directory removal, all using literal names.

### Impacted Decisions

| Decision Item | Relationship | Review |
|---|---|---|
| D22 (Synthesizing Unit Atomicity) settled item 2 | Supports | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |

## Historical Versions

### 2026-08-30

- Initiated and completed run. Cause: user asked "what does 4 raw disks without striping in VM mean, does no striping mean everything was inferred"——
  touching the exact scope boundary explicitly stated in E53 (Can Root Ring Mount After Losing an Entire Disk).
