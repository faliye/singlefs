## E129 How Writes Below io_min Fail on Real Devices —— Run (2026-09-10, three stages of real devices, 5 rounds cell-by-cell identical, 9 mutations caught)

[中文](129-小于iomin的写在真设备上怎么出事.md) · **English** · [日本語](129-io_min-未満の書き込みの実デバイス上での障害形態.ja.md)

**It answers two questions**, with criteria, thresholds, and invalidation clauses set down in `e129-preregistration.md` prior to any harness code:

1. At the end of D2 (RAID Stripe Policy) section "Write Granularity", verbatim: "⚠️ Local machine cannot falsify via fault injection (`dm-flakey` cannot construct tears); per `.claude/singlefs-ai-sop/rules/evidence-discipline.md` recorded as **mechanism derivation**, falsification awaits crash-point replay harness." —— **whether this sentence still holds**.
2. The specific justification owed by Candidate ② of C212 (fixed-structure writes smaller than io_min): "that requires showing why fixed structures do not fear intra-device RMW, whereas the real device round of E34 (Root Ring Slot Geometry) **only proved the single mechanism of RAID5 write hole**" —— **whether a second mechanism exists beyond write hole**.

### Conclusion 1: Position-Deterministic Tear Injector Can Be Constructed, 5/5 Rounds

The construction uses `dm-linear` to concatenate one device into two segments: the front segment sits on normal loop, while the rear segment is `dm-error`;
for an O_DIRECT write **straddling the boundary**, three things hold simultaneously. Artifact `e129-tear-injector-2026-09-10.out` verbatim:

```
E7RESULT name=span round=1 write_rc=1 prefix_persisted=1 suffix_absent=1
E7RESULT name=poscontrol round=1 write_rc=0 all_persisted=1
E7RESULT name=negcontrol round=1 write_rc=1 nothing_persisted=1
E7RESULT name=verdict poscontrol_bad=0 negcontrol_bad=0 span_torn_rounds=5/5
```

⇒ **The sentence in D2 (RAID Stripe Policy) stating "local machine cannot falsify via fault injection" does not hold**: changing construction allows it to be built,
and the tear point is **hardcoded in dm table**, fully reviewable.

⚠️ **Scope must be pinned, this point only overturns that meta-assertion**: the **substantive** claim of D2 (RAID Stripe Policy) settled item 16 is
"devices perform read-modify-write internally; power failure may corrupt already persisted neighbor data within the same mapping unit",
which the tear injector cannot construct ("intra-device RMW") —— that is answered by Conclusion 2 and Conclusion 3.
⚠️ **The half-sentence regarding `dm-flakey` remains correct**: its `corrupt_bio_byte` / `random_write_corrupt` constructs
**byte corruption**, not tearing (prefix persisted, suffix unpersisted). Neither may substitute for the other.

### Conclusion 2: Non-RAID Real Devices with `io_min > physical_block_size` Exist, and a 512-Byte Write Amplifies into a Whole Block on Them

`dm-thin`, block 64 KiB. When a block is **shared** (has snapshot), a write to any 512 bytes within block triggers whole-block COW.
Artifact `e129-thin-rmw-2026-09-10.out` verbatim:

```
E7RESULT name=cell round=1 shared=1 host_sectors=1 data_dev_sectors=129 thin_io_min=65536 thin_pbs=512
E7RESULT name=cell round=1 shared=0 host_sectors=1 data_dev_sectors=1 thin_io_min=65536 thin_pbs=512
E7RESULT name=verdict shared_not_amplified=0 exclusive_amplified=0 cells=10 cells_with_declared_iomin=10 declared_block_bytes=65536
```

Three facts emerge together: ① Host issues 1 sector, data device writes **129** (whole block 128 + that one);
② When block is exclusive it writes only 1, **a 129× difference between cells**, control clearly distinguishes difference;
③ `thin_io_min = 65536` whereas `thin_pbs = 512` ⇒ **this is a real device where `io_min` is 128× larger than `physical_block_size`,
and it is not RAID**. Antecedent of C212 (fixed-structure writes smaller than io_min) was only measured on arrays prior to E129 (How Writes Below io_min Fail on Real Devices) (E34 (Root Ring Slot Geometry));
this finding extends it to non-arrays.

⇒ **What D2 (RAID Stripe Policy) settled item 16 describes, "issuing smaller writes to a device physical mapping unit causes internal read-modify-write",
is measured on real devices, representing a second mechanism** —— distinct family from RAID5 write hole: no parity, no cross-leg dependencies.

### Conclusion 3: When This RMW Is Accurately Interrupted, Neighbors in the Same Mapping Unit Are Byte-for-Byte Intact Across 5/5 Rounds

Padding tear injector of Conclusion 1 underneath pool's **data device**, preventing second half of COW from landing.
Artifact `e129-thin-neighbour-2026-09-10.out` verbatim:

```
E7RESULT name=neighbour round=1 inject=1 write_rc=1 read_rc=0 verdict=intact first_diff=-1 cow_sectors=64 pool_data_before=1/256 pool_data_after=2/256
E7RESULT name=neighbour round=1 inject=0 write_rc=0 read_rc=0 verdict=intact first_diff=-1 cow_sectors=129 pool_data_before=1/256 pool_data_after=2/256
E7RESULT name=verdict inject_not_torn=0 negcontrol_damaged=0
```

⚠️ **The discriminative power half is load-bearing, stated first**: looking solely at "neighbors intact" cannot distinguish "mechanism is safe" from "injection failed to hit COW".
`cow_sectors=64` (injected cell) vs `129` (control cell) proves **injection truly tore COW on the designed boundary** ——
intact segment 192 sectors = Block 0 full 128 + Block 1 front 64, COW falling precisely into 64 hits error segment;
`pool_data 1/256 → 2/256` grew on both cells, proving both cells truly allocated a second block.
Without these two figures, "intact" on this cell proves nothing.

⇒ **Failure mode is "write fails + old data intact", not "silent neighbor corruption"**:
`dm-thin` intra-block COW is **transactional** —— allocating new block first, switching mapping only after copy completes; when copy tears mapping does not switch,
reads still hit old shared block ⇒ neighbors intact byte for byte, while writer of A **receives error** (`write_rc=1`).

### What These Three Mean Combined for C212 (fixed-structure writes smaller than io_min)

**Provides positive evidence for Candidate ②, but insufficient to establish it.** Stated clearly point by point:

| What It Provides | What It Does Not Provide |
|---|---|
| The universal statement "all intra-device RMW is dangerous" is **weakened**: at least one (`dm-thin` intra-block COW) is transactional, interrupting it leaves neighbors undamaged | It **only covers `dm-thin` this single mechanism**, not generalizing to others |
| The impression that "`io_min > pbs` only appears on arrays" is overturned: non-RAID has it too | Intra-FTL RMW across whole mapping unit on SSD **still cannot be injected** (noted in E34 (Root Ring Slot Geometry), unresolved by E129) |
| Tear states can be constructed on demand ⇒ an entire class of invariants can be verified without awaiting crash-point replay harness | ⚠️ **It does not form an opposing pair with RAID5**: E34 (Root Ring Slot Geometry) real-device text states verbatim "neighbor slot B in same chunk (sector bit 1) **10/10 intact**", fault unit "is same stripe row × same sector bit across legs, **not entire `io_min` unit**" ⇒ **neighbors of both measured mechanisms are intact**; the statement "intra-device RMW corrupts neighbors within same mapping unit" had **zero empirical support** as of 2026-09-10 |

⇒ **Deduction (not reviewed by tripartite, non-binding, shape only)**: Rationale sentence of D2 (RAID Stripe Policy) settled item 16
("devices perform read-modify-write internally; power failure may corrupt already persisted neighbor data within the same mapping unit")
holds or fails **mechanism by mechanism** on real devices, not universally. Modifying that settled clause requires three-way argument.

### Measurement Basis and Replay

All three stages use real devices (loop + dm/md), **touching zero bytes of `nvme0n1` partitions**;
requires `sudo`, reading password from `.env` without echoing; device names taken from variables, torn down one by one upon `EXIT`.

```bash
bash scripts/devices/e129-tear-injector.sh      # Conclusion 1
bash scripts/devices/e129-thin-rmw.sh           # Conclusion 2
bash scripts/devices/e129-thin-neighbour.sh     # Conclusion 3
```

Three raw outputs:
`e129-tear-injector-2026-09-10.out`,
`e129-thin-rmw-2026-09-10.out`,
`e129-thin-neighbour-2026-09-10.out`.

⚠️ **Replay is not byte-for-byte comparison**: device names (`/dev/loopN`) and working directories differ each time.
What pins the conclusions are `name=verdict` lines in three artifacts, along with respective invalidation clauses ——
scripts **report `name=fatal` and exit non-zero themselves** upon any control mismatch.

⚠️ **It creates on-disk states, not real power loss**: loop cannot cut power. States equivalent, paths non-equivalent.
This sentence is the identical constraint to real-device half of E34 (Root Ring Slot Geometry).
⇒ What it buys is "ability to construct tear states to verify an invariant", **not** "how frequently tears occur".

### Mutations

Three tables total 9 entries, all caught: `scripts/mutation/tables/e129_tear_injector.tsv` (3),
`scripts/mutation/tables/e129_thin_rmw.tsv` (3), `scripts/mutation/tables/e129_thin_neighbour.tsv` (3).
Tested apparatuses are shell probes rather than Rust binaries, run methodology written in table headers: substitute "original text" with "replacement text",
run script once, **must produce `name=fatal` with non-zero exit code**.

⚠️ **One judged equivalent mutation and retained in archive**: modifying `dm-thin` block width from 128 sectors to 256 is uncaught,
**and should not be caught** —— `BLK_BYTES` is calculated from `BLK_SECT`, `io_min` assertion follows,
and Conclusion 2 is scale-independent regarding block size ⇒ that is a valid alternative configuration, not a bug.
Replaced with "building snapshot on exclusive cell as well" per `.claude/rules/mutation-sampling.md`, testing identical discriminator from opposite side.

⚠️ **Process of supplementing this assertion stepped on category three**: threshold 128 of `shared_not_amplified` was hardcoded,
**insensitive to upward modifications of block size**, hence requiring an additional pin that "measured `io_min` must equal block width declared in this section".

### Influenced Decisions

| Decision Item | Relationship | Review |
|---|---|---|
| D2 (RAID Stripe Policy) settled item 16 | Support | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D2 (RAID Stripe Policy) settled item 17 | Support | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D2 (RAID Stripe Policy) settled item 19 | Support | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D17 (Implementation Layering & Third-Party Pipeline) settled item 1 | Support | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |

## Historical Versions

Historical entries for E129 (How Writes Below io_min Fail on Real Devices) are consolidated in [experiments-history.md](../experiments-history.md).
