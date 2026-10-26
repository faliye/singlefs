## E109 Runtime Costs of Three Location Authority Arms — Run (2026-09-06, 13 unit tests / 17 mutants caught): **On relocation `ptr` is linear with reference tree count, central mapping constant; on publish central mapping pays extra 66 048-byte fixed quota; on read central mapping doubles, while hybrid form adds only 19–37%**

[中文](109-位置权威三臂的运行时代价.md) · **English** · [日本語](109-位置権威3アームの実行時コスト.ja.md)

**This answers the projection of the same fork across four places**: D21 (Boundary Between Authoritative and Derived States) item "where multiple referrers reside under sharing",
D26 (Background Compaction and Placement Reclamation) settled item 5, C34 (three-way clash on plain text mapping layer states),
and shape of layer after D9 (Encryption) settled item 9 adopts physical keys.
Two rounds of three-way argument independently concluded these four places are the same decision, but **"which one is faster" had zero numbers in the repo** — E109 (Runtime Costs of Three Location Authority Arms) fills that exact number.

### Three Arms

| Arm | Where Location Authority Resides | What Moves to Relocate One Unit | Reading One Unit |
|---|---|---|---|
| `ptr` | Position entry in pointer (literal form of D19 (Block Pointer Structure & Width Budget) settled item 4, **the one settled today**) | COW one path in each reference tree + rewrite each root | 0 extra hops |
| `map` | One central mapping layer "logical identity → physical" | COW one path in mapping tree + 1 root | 1 extra mapping point lookup each time |
| `hybrid` | Position entry as **hint**, central mapping is authoritative | Same as `map` | Hint hit 0 hops, only stale hint pays extra hop |

### Measurement Basis (Locked Before Run, Changing It Means New Experiment)

1. **Geometry all points to settled items**: Data unit 32768 (D4 (Checksum Location) settled items 1 / 5),
   index node 16384 (D8 (Core Index Structure) settled item 2), fanout 119 (D11 (Message Buffers in Index Nodes) settled item 2, ε = 0.65),
   position entry 14 bytes (D19 (Block Pointer Structure & Width Budget) settled item 4).
2. **Mapping entry key takes settled 5-tuple width 33 bytes** (D18 (Information Carried in a Block) settled item 3 settles 5-tuple of data unit self-describing header at 33,
   field table in settled item 7 `105 = 42 + 33 + 8 + 8 + 10 + 4` consistent with it). Remaining three tiers (16 / 24 / 40) kept as sensitivity probes.
   ⚠️ **Round 1 on 2026-09-06 wrote this item as "not settled in repo", which was wrong** — settled item 3 settled it definitively.
3. **One publish** = D25 (Target Workload Priority) primary workload (`N` data leaves + one shared spine 4 index nodes
   + journal 4096 + root slot 512), **byte-for-byte same ruler as E107 (Stripe Table Write Amplification on Primary Workload)**, facilitating cross-experiment comparison.
4. **`K` is count of trees referencing same unit** (multiple writable heads / snapshots, D6 (Snapshot Implementation Model) settled item 1 "each head has its own tree").
5. Read cost only counts **node read count**, not converted to time — wall clock is outside scope.
6. ⚠️ **Round 1 had an even harder flaw, now fixed**: Sample point was only one, and at that point reference tree and mapping tree **happened to share identical height 4**
   ⇒ Deleting mapping tree geometry entirely and replacing with reference tree height yielded **11 unit tests all green, 61-line artifact byte-for-byte unchanged**
   (verified in run on out-of-repo copy by reverse derivation attack leg on 2026-09-06, md5 unchanged).
   Its "positive control" was identical expression on both sides (`X == X`), **never once calling tested `map` arm**.
   ⇒ Now sweeps six points across points where two trees have **different heights**; positive control changed to "changing mapping geometry ⇒ `map` arm must change, `ptr` arm must not change",
   supplementing three mutants specifically targeting this: changing `map` arm height to reference tree height, changing read path mapping height to reference tree height, reverting key width to 32.

### Results (2026-09-06, artifact `e109-position-authority-2026-09-06.out`, 67 lines, `emitted=67`)

**Relocating one unit** (4 million leaves reference tree, height 4; mapping tree fanout 267, height 4):

| `K` | `ptr` Bytes | `map` Bytes | Ratio |
|---|---|---|---|
| 1 | 66 048 | 66 048 | 1.00× |
| 2 | 132 096 | 66 048 | 2.00× |
| 4 | 264 192 | 66 048 | 4.00× |
| 8 | 528 384 | 66 048 | 8.00× |
| 16 | 1 056 768 | 66 048 | 16.00× |

⇒ **Ratio exactly equals `K`**: `ptr` scales linearly with reference tree count, `map` independent of `K`.

**One publish**: `map` pays **66 048 bytes more than `ptr`, and this number is independent of leaf count** (mapping tree leaf node 1 + internal 3 + one root slot):

| Leaf Count | `ptr` | `map` | Ratio |
|---|---|---|---|
| 1 | 102 912 | 168 960 | 1.642× |
| **8 (primary workload)** | **332 288** | **398 336** | **1.199×** |
| 64 | 2 167 296 | 2 233 344 | 1.031× |

**Crossover point**: At `K = 1` (zero sharing) `map` saves nothing ⇒ **never breaks even** (reports −1);
**at `K ≥ 2` relocating just 1 shared unit per publish breaks even for `map`** (`K = 2` exactly ties, larger `K` saves more).

**Read path** (node count per thousand reads):

| Hint Stale Rate | `ptr` | `map` | `hybrid` |
|---|---|---|---|
| 0‰ | 4000 | 8000 | **4000** |
| 190‰ (E96 (Consistency of Hybrid Architecture) round-end lower bound) | 4000 | 8000 | **4760 (+19%)** |
| 370‰ (E96 (Consistency of Hybrid Architecture) round-end upper bound) | 4000 | 8000 | **5480 (+37%)** |
| 1000‰ | 4000 | 8000 | **8000** |

⇒ **Read cost of `map` doubles, while `hybrid` on E96 (Consistency of Hybrid Architecture) measured stale rate range adds only 19–37%.**

**Controls**: Negative control (zero publish ⇒ all three arms 0) pass;
positive control (relocation bytes must equal `ptr` arm at `K = 1` when mapping tree geometry matches reference tree) pass,
with the discriminative power self-test in unit tests (swapping to fanout 20 mapping geometry must not equal);
**hardest discriminatory test**: `hybrid` must match `ptr` cell-for-cell when hint stale rate is 0, must match `map` cell-for-cell at 1000‰,
and intermediate values must fall strictly between both ends — failure indicates `hybrid` was never built as a true intermediate arm.

**Discriminative power (newly added after fix)**: Three of six sample points have two trees with **different heights**;
`map` arm bytes follow mapping tree geometry (pool 1e7 / 4.83e8 / 1e10 ⇒ 49 664 / 66 048 / 82 432),
while `ptr` arm remains completely motionless against pool scale (constantly 66 048).
⚠️ **Main sample point (4 million leaves / full pool) is a degenerate point**: there `map` and `ptr` (`K = 1`) are both 66 048,
**because both trees happen to have height 4, not because they are inherently identical** — citing this 66 048 must state the sample point.

**Mutations**: `scripts/mutation/tables/e109_position_authority.tsv` **17 items, all 17 caught**, baseline green before and after run.
⚠️ **Round 1 had 14 items, none of which distinguished "map arm height calculated from mapping tree" vs "from reference tree"**;
of the three added items, "read path mapping height uses reference tree height" was still a blind spot on first run,
caught only after adding an assertion testing read path at points where the two trees differ in height.
⚠️ **This is a deterministic model**: running same binary N times is necessarily byte-for-byte identical,
so evidence strength **does not come from round count**, but from those 17 mutants and assertions nailing absolute values
(scope note for "single observation does not count" in `.claude/singlefs-ai-sop/rules/test-discipline.md`).

**Rerun** (criterion `exact`, should be byte-for-byte identical):

```bash
bash scripts/experiment-apparatus/replay.sh E109
```

### What It Cannot Answer

- **Does not answer wall clock or cache hits**: whether mapping layer fits in memory is another experiment, while E12 (Batched Sequential Append vs Unbatched Random Page RMW)
  measured that inflection point on isomorphic problem is steep (counter table jumping from nearly free to ~1 device I/O per op once exceeding cache)
  ⇒ **`map` read doubling dimension when exceeding cache may far exceed doubling, E109 (Runtime Costs of Three Location Authority Arms) cannot measure that cell**.
- **Does not answer consistency**: E96 (Consistency of Hybrid Architecture) measured silent wrong reads of hybrid form and its two hard rules
  (unit header or hint must carry block birth, release must go through mapping not hint), E109 (Runtime Costs of Three Location Authority Arms) does not re-measure.
- **Does not answer cost after encryption enabled**, nor lock period semantics.
- **Does not answer space efficiency** (`.claude/rules/fs-design.md` "comparing who saves more does not constitute any criterion").
- **Key width dimension separates nothing across the four sampled tiers**: tiers 16 / 24 / 40 only alter mapping tree fanout (370 / 313 / 239),
  while mapping tree height is 4 across all four tiers ⇒ relocation, publish, crossover point three tables show **identical bytes per cell**.
  ⇒ This dimension is not sensitivity evidence, only proving "conclusions do not move across these four tiers"; what actually flips is tree height, dominated by pool scale.
- Six sample points remain **discrete points**, not swept curves; neither real pool scale nor real reference tree scale has measured profile.

### Decisions Affected

| Decision Item | Relationship | Review |
|---|---|---|
| D21 (Boundary Between Authoritative and Derived States) settled item 1 | Supports | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D21 (Boundary Between Authoritative and Derived States) settled item 6 | Supports | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D21 (Boundary Between Authoritative and Derived States) settled item 19 | Supports | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D26 (Background Compaction and Placement Reclamation) settled item 5 | Preparation | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D9 (Encryption) settled item 9 | Preparation | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D18 (Information Carried in a Block) settled item 3 | Does not affect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D19 (Block Pointer Structure & Width Budget) settled item 5 | Supports | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D11 (Message Buffers in Index Nodes) settled item 2 | Does not affect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D6 (Snapshot Implementation Model) settled item 1 | Does not affect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D4 (Checksum Location) | Does not affect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D8 (Core Index Structure) | Does not affect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D25 (Target Workload Priority) settled item 1 | Does not affect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |

## Historical Versions

### 2026-09-06
- Filed, run on same day, rejected for rework on same day.
- **Two points of rejection** (adopted after main agent line-by-line verification from 2026-09-06 D21 (Boundary Between Authoritative and Derived States) multiple referrers round 2 reverse derivation attack leg):
  ① Item 2 of measurement basis "mapping entry key width is not a settled constant" **was wrong** — D18 (Information Carried in a Block) settled item 3
  definitively settled data unit self-describing header 5-tuple at 33 bytes;
  ② **Sample point was only one, and at that point both trees happened to share identical height 4** ⇒ deleting mapping tree geometry entirely and replacing with reference tree height
  left 11 unit tests all green, artifact byte-for-byte unchanged (attack leg ran on out-of-repo copy, md5 `f911674f23d815a9346789bdbe2c4bec` unchanged).
  Its "positive control" `assert_eq!(same_geo_map, ptr_k1)` had identical expression on both sides, **never once calling `Arm::Map`**.
- **Fix**: Key takes 33; sample points expanded to six (including three where trees differ in height); positive control changed to
  "changing mapping geometry ⇒ `map` arm must change, `ptr` arm must not change"; added three mutants targeting mapping geometry;
  added an assertion testing read path at points where trees differ in height, closing "read path mapping height" blind spot. Mutations 14 → 17, unit tests 11 → 13.
- **Structural conclusions unchanged** (relocation `ptr` linear with `K` / `map` independent of `K`, publish fixed quota, read doubling);
  what changed is that **absolute values hold only at stated sample points**, degenerate point noted in body.

