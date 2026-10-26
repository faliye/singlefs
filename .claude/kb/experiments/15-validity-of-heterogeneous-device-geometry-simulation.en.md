## E15 Validity of Heterogeneous Device Geometry Simulation — Run (2026-08-27, Heterogeneous Tier + Negative Control Bidirectionally Passed)

[中文](15-异构设备几何的模拟是否成立.md) · **English** · [日本語](15-異種デバイス幾何シミュレーションの成立性.ja.md)

**Conclusion**: **Valid.** This host has no zoned disks, no 4Kn disks, and no rotational disks, but QEMU successfully creates heterogeneous geometry that the guest **can indeed distinguish**, so [decisions.md](../decisions.md) D12 (Target Media) settled item 3 and [checks-owed.md](../checks-owed.md) C17 (Device Geometry as Global Parameter) do not need to wait for physical hardware.

**Measurement Basis**: QEMU 8.2.2 + kernel 6.17.0-lockdep, `nvme` via modules insmodded in initramfs.
What is measured is the **geometry read by the guest kernel** (`/sys/block/*/queue/`), not QEMU command line arguments.
Re-run: `scripts/devices/vm-geom.sh` (not tracked in git).
⚠️ **Raw output not retained**: E15 (Validity of Heterogeneous Device Geometry Simulation) read `/sys/block/*/queue/` in guest kernel, which was not persisted to disk at the time.
The two geometry tables of E15 (Validity of Heterogeneous Device Geometry Simulation) can therefore only be checked by re-runs, which require QEMU.

### Empirical Measurements

**Heterogeneous tier** (three namespaces with mutually distinct geometries):

| Device | Logical/Physical Sector | zoned | chunk_sectors | zone count | bytes |
|---|---|---|---|---|---|
| nvme0n1 | 4096 / 4096 | host-managed | 131072 | 8 | 536870912 |
| nvme0n2 | 4096 / 4096 | host-managed | **262144** | **4** | 536870912 |
| nvme0n3 | **512 / 512** | **none** | 0 | 0 | 536870912 |

⇒ Guest observes **3 distinct** geometries. Numbers are self-consistent: 131072 sectors × 512 B = 64 MiB, 262144 × 512 = 128 MiB;
512 MiB ÷ 64 MiB = 8 zones, ÷ 128 MiB = 4 zones.

**Negative control** (three namespaces with identical geometries): Guest observes **1** geometry.
⇒ **Read path has discriminative power**, and the "3 kinds" in heterogeneous tier is not an echo of command line arguments.

### Three Configurations Creatable in a Single Command

**Heterogeneous zoned pool** (zone size 64M vs 128M), **mixed 512e and 4Kn**, **mixed zoned and conventional**.
`zoned.zone_size` and `logical_block_size` attach to `nvme-ns` (namespace) rather than the controller,
so multiple namespaces under the same controller can each differ.

### Four Pitfalls Encountered Building This Setup (All Fail Silently)

| Pitfall | Manifestation | Criterion |
|---|---|---|
| `logical_block_size` mismatched with `physical_block_size` | QEMU rejects directly: `logical_block_size > physical_block_size not supported` | Errors out, not silent |
| **Integrity gate lets `0/0` pass** | Guest sees zero devices, gate judges "0 items == claimed 0 items" passing, turning conclusion into "only saw 1 geometry ⇒ simulation invalid" | **Must explicitly block `n < 1`**—reading nothing ≠ reading 0 |
| **Kernel and module tree version mismatch** | Majority of 4 kernels in `/boot` unreadable; "first readable one" might lack matching module tree; vermagic mismatch ⇒ all four insmods fail ⇒ zero devices in guest | When selecting kernel, must simultaneously require `/lib/modules/$v` exists |
| Module tree compression formats vary | Host `-oem` tree is `.ko.zst`, `lockdep` tree is bare `.ko`; hardcoding one fails on the other | Support both |

⚠️ **Stacking the first two pitfalls produces a false conclusion that "looks like a negative result"**: the simulation failed to launch,
yet output shows "guest only sees 1 geometry"—outwardly identical to "geometry simulation invalid".

### What E15 (Validity of Heterogeneous Device Geometry Simulation) Cannot Prove

- **Does not prove rotational disks.** The host lacks rotational disks entirely; QEMU's `rotation_rate` alters only declaration, not latency.
  Rotational media can only be simulated via **modeled devices** such as `dm-delay`, which is a separate matter and requires separate measurement basis documentation.
- **Does not prove file system correctness on heterogeneous pools.** It only proves the step that "guest can distinguish different geometries" holds.

**Prerequisites**: None (already run).

---

### Affected Decisions

| Decision Item | Relationship | Review |
|---|---|---|
| D12 (Target Media) settled item 3 | Preparation | 2026-09-20 unaffected: E15 (Validity of Heterogeneous Device Geometry Simulation) proves QEMU can create heterogeneous geometries that guests can distinguish, clearing the fault injection prerequisite for owed check C17 (Device Geometry as Global Parameter) and the "forced entry" of D12 (Target Media) settled item 7 debt table item 4; evidence settling item 3 (geometry constraining device descriptor table in system configuration) is E72 (Mount-Time Recalculation of Device Descriptor Table), not E15 (Validity of Heterogeneous Device Geometry Simulation) |

## Historical Versions

Historical entries for E15 (Validity of Heterogeneous Device Geometry Simulation) are collected in [experiments-history.md](../experiments-history.md).
