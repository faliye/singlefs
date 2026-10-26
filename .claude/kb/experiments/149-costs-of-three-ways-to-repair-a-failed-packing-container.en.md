## E149 Costs of Three Repair Options for Broken Packed Containers — Run (2026-09-13, counting model, 6 unit tests / all 8 mutations caught)

[中文](149-打包容器坏了靠什么补的三条补法各付什么.md) · **English** · [日本語](149-パッキングコンテナの破損を補う三つの方法のコスト.ja.md)

The question is the one that D27 (Small-Data Packing Container), settled item 1 (reopened), requires to be answered together: how much is lost when a container fails, and how can it be recovered? The text lists three remedies—forcing extra container replicas, lowering the cap, and abandoning packing—and says none had yet been measured.
Using the measurement basis of E114 (Overall Accounting of Small-Data Packing Containers)—container header 107, self-description per slot 47, two replicas, and the post-compaction `packbg` form—we calculate each remedy's space benefit, written-byte ratio, loss radius, and number of replicas that must fail together. We report numbers without choosing a winner.

**Result in one sentence**: An extra replica multiplies the benefit by 2/3 and written bytes by 3/2, leaves the radius unchanged, and changes the loss condition from two replicas failing together to three (at 512 B, 58× → 38.7× and writes 1.017 → 1.526); capping cap at k makes both benefit and radius min(cap, k) (with a ceiling of 8, the 512 B / 1 KiB classes fall from 58 / 30 to 8, while the three classes at 4 KiB and above do not change); abandoning packing gives benefit 1, radius 1, and writes 1. None of the five arms gives a benefit > 1 at 16 KiB (cap = 1 is a hard boundary, as stated in E114 (Overall Accounting of Small-Data Packing Containers)).

**Rerun command** (`exact` mode, byte-for-byte comparison with the retained artifact):

```bash
bash scripts/experiment-apparatus/replay.sh E149
# 直接跑装置：
cd scripts/experiment-apparatus && cargo run --release --bin e149-pack-container-repair-options
```

Code: `scripts/experiment-apparatus/e149_pack_container_repair_options.rs`; raw output: `e149-pack-container-repair-options-2026-09-13.out` (39 lines, last line `emitted=39`);
pre-run registration: `e149-preregistration.md`; mutation table: `scripts/mutation/tables/e149_pack_container_repair_options.tsv` (all 8 caught,
`bash scripts/mutation/mutate.sh e149-pack-container-repair-options scripts/experiment-apparatus/e149_pack_container_repair_options.rs scripts/mutation/tables/e149_pack_container_repair_options.tsv`).

### This is a counting model, not an implementation

There is no I/O or random source ⇒ N runs are necessarily byte-identical; the strength of the evidence comes from 6 unit tests and all 8 mutations caught. In the first mutation round, neither “copy container header as 103” nor “copy the self-describing slot overhead as 43” made any test fail: both measurement bases produced the same caps at the five object sizes, category three of `.claude/rules/mutation-sampling.md` (insensitive sample points). Adding the 6486-byte sample point across an integer boundary (cap 4 at 107 / 47, 5 at 103 or 43) caught both.

### Arms and measurement basis

| Arm | Method | Benefit | Written-byte ratio | Loss radius | Replicas that must fail together |
|---|---|---|---|---|---|
| `none` | E114 (Overall Accounting of Small-Data Packing Containers) `packbg` unchanged, 2 replicas | cap | 1 + 1/cap | cap | 2 |
| `extra_copy` | Force 3 container replicas | cap × 2/3 | (1 + 1/cap) × 3/2 | cap | 3 |
| `cap_2` / `cap_4` / `cap_8` | Cap the capacity | min(cap, ceiling) | 1 + 1/min(cap, ceiling) | min(cap, ceiling) | 2 |
| `no_pack` | Abandon packing (`pad` in E114 (Overall Accounting of Small-Data Packing Containers)) | 1 | 1 | 1 | 2 |

cap = ⌊(32768 − 107) / (size + 47)⌋, giving 58 / 30 / 7 / 3 / 1 for the five classes (positive control: exactly the same as E114 (Overall Accounting of Small-Data Packing Containers), pinned by a unit test). Benefit = objects per unit after compaction divided by the replica-count ratio; the written-byte ratio uses `packbg` as n → ∞ (one independent unit per object + 1/cap containers); values reported in basis points are rounded down.

### Verdict rows copied verbatim from the artifact (512 B and 4 KiB; 1 KiB / 8 KiB / 16 KiB are in the artifact)

```text
E7RESULT name=config unit_bytes=32768 pack_header=107 slot_self_description=47 replicas=2
E7RESULT name=capacity object_bytes=512 cap=58
E7RESULT name=repair object_bytes=512 arm=none objects_per_container=58 replicas=2 space_benefit_bp=580000 write_ratio_bp=10172 loss_radius=58 copies_that_must_fail=2
E7RESULT name=repair object_bytes=512 arm=extra_copy objects_per_container=58 replicas=3 space_benefit_bp=386666 write_ratio_bp=15258 loss_radius=58 copies_that_must_fail=3
E7RESULT name=repair object_bytes=512 arm=cap_2 objects_per_container=2 replicas=2 space_benefit_bp=20000 write_ratio_bp=15000 loss_radius=2 copies_that_must_fail=2
E7RESULT name=repair object_bytes=512 arm=cap_4 objects_per_container=4 replicas=2 space_benefit_bp=40000 write_ratio_bp=12500 loss_radius=4 copies_that_must_fail=2
E7RESULT name=repair object_bytes=512 arm=cap_8 objects_per_container=8 replicas=2 space_benefit_bp=80000 write_ratio_bp=11250 loss_radius=8 copies_that_must_fail=2
E7RESULT name=repair object_bytes=512 arm=no_pack objects_per_container=1 replicas=2 space_benefit_bp=10000 write_ratio_bp=10000 loss_radius=1 copies_that_must_fail=2
E7RESULT name=capacity object_bytes=4096 cap=7
E7RESULT name=repair object_bytes=4096 arm=none objects_per_container=7 replicas=2 space_benefit_bp=70000 write_ratio_bp=11428 loss_radius=7 copies_that_must_fail=2
E7RESULT name=repair object_bytes=4096 arm=extra_copy objects_per_container=7 replicas=3 space_benefit_bp=46666 write_ratio_bp=17142 loss_radius=7 copies_that_must_fail=3
E7RESULT name=repair object_bytes=4096 arm=cap_2 objects_per_container=2 replicas=2 space_benefit_bp=20000 write_ratio_bp=15000 loss_radius=2 copies_that_must_fail=2
E7RESULT name=repair object_bytes=4096 arm=cap_4 objects_per_container=4 replicas=2 space_benefit_bp=40000 write_ratio_bp=12500 loss_radius=4 copies_that_must_fail=2
E7RESULT name=repair object_bytes=4096 arm=cap_8 objects_per_container=7 replicas=2 space_benefit_bp=70000 write_ratio_bp=11428 loss_radius=7 copies_that_must_fail=2
E7RESULT name=repair object_bytes=4096 arm=no_pack objects_per_container=1 replicas=2 space_benefit_bp=10000 write_ratio_bp=10000 loss_radius=1 copies_that_must_fail=2
E7RESULT name=sensitive cap_8_changes_512=true cap_8_changes_4096=false
E7RESULT name=verdict caps=58/30/7/3/1 any_gain_at_16k=false
```

### What these numbers mean

1. **The three remedies buy different things**: extra replicas require three simultaneous failures for loss, at the cost of one-third less benefit and half again as many written bytes; lowering cap reduces the radius at the cost of an equal reduction in benefit (benefit and radius are the same number, confirming cell by cell D27 (Small-Data Packing Container) settled item 1's statement that the benefit multiple and permanent-data-loss radius multiple are the same); abandoning packing buys radius 1, with benefit also reduced to 1.
2. **A ceiling of 8 affects only 512 B and 1 KiB**: cap is already 7 at 4 KiB and 3 at 8 KiB ⇒ lowering cap has no effect at the 4 KiB boundary of D27 (Small-Data Packing Container) settled item 2; it matters only if the boundary is widened below 1 KiB.
3. **No remedy covers the 16 KiB class**: with cap = 1, extra replicas reduce benefit to 0.67 (more costly than not packing), while all other arms give 1.
4. The choice for D27 (Small-Data Packing Container) settled item 1 must be made together with item 2's boundary: the boundary determines cap, and cap determines radius. On 2026-09-13 the user decided: no additional remedy, rely on two replicas, and use a 4 KiB boundary.

### What it cannot answer

1. The probability of container failure (device failure rate, silent corruption rate): “how many replicas must fail together” is only an exponent, not a probability.
2. The wall-clock cost of compaction (C202 (idle overhead and hysteresis K of packing patrol lack values)) and the coverage denominator (C174 (Three columns of smallfile row unmeasured)).
3. Adding check blocks inside the container (local erasure coding) was not modeled; the text of D27 (Small-Data Packing Container) did not list it.

### Affected decisions

| Decision item | Relationship | Review |
|---|---|---|
| D27 (Small-Data Packing Container) settled item 1 | Supports | 2026-09-19 unaffected: the costs of the three remedies, and the fact that without an additional remedy benefit and loss radius are both cap and loss requires two simultaneous replica failures, support “no additional container remedy” |
| D27 (Small-Data Packing Container) settled item 2 | Supports | 2026-09-19 unaffected: capping cap has no effect at the 4 KiB boundary, and a 2 KiB boundary changes only the 2–4 KiB range to radius 1; this is one reason for choosing the 4 KiB boundary |

## Historical Versions

### 2026-09-13
- Created. The pre-run registration preceded the apparatus; the unit test for the 6486-byte sample point was added after the first mutation round, without changing the criteria.
