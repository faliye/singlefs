## E86 Scan Stride and Two Unit Sizes — Run (2026-09-02, Counting Model, 8 Unit Tests / 6 Mutations All Caught): 16 KiB Stepping Fully Recovers; 32 KiB Stepping Misses Exactly All Odd-Slot Nodes (~Half); "Nodes Monopolize 32K" Fully Recovers but Metadata Space Constantly 2x

[中文](86-扫描步进与两种单元大小.md) · **English** · [日本語](86-スキャンステップと2種類のユニットサイズ.ja.md)

Decision material for C86 (Two Unit Sizes Conflict with Scan Stepping): D18 (Information Carried in a Block) settled item 9 verbatim
"step by 32768", D4 (Checksum Location) settled item 1 verbatim "metadata side 16 KiB"——how much each of three repair options recovers and what each pays.

### Criteria (Written Hard Before Run, Unchanged After Run)

1. 16 KiB stepping recovery rate constantly 100%; 2. 32 KiB stepping + nodes in arbitrary 16K slots miss count **constantly equals** independently tallied
odd-slot node count (the two paths share no stepping logic); 3. Nodes monopolizing 32K slot pairs ⇒ full recovery with node slot occupancy constantly 2x;
4. Internal probe count closed form (16K stepping matches exactly data unit count, 32K stepping constantly 0); 5. Repair choice belongs to C86 (Two Unit Sizes Conflict with Scan Stepping) closeout decision, delivering numbers.

### Empirical Measurements (64 MiB Region = 4096 16-KiB Slots, Node:Data Slot Ratio ~ 1:2, 5 Seeds; seed=11 Row)

| Repair Arm | Nodes Fully Recovered | Data Fully Recovered | Cost |
|---|---|---|---|
| **Stepping 16384, nodes in arbitrary 16K slots** | **998/998** | 1392/1392 | 1 internal probe per data unit (1392 times, distinguished by magic + header checksum, E76 (Discriminative Power of Payload Checksum) measurement basis) |
| Stepping 32768, nodes monopolize 32K slot pairs | 631/631 | 1402/1402 | **Node slot occupancy constantly 2x** (1262 slots hold 631 nodes, wasting 50%) |
| Stepping 32768, nodes in arbitrary 16K slots (combination of existing wording) | **502/998——misses exactly 496 odd-slot nodes** | 1392/1392 | Scan rebuild as sole fallback (D21 (Boundary Between Authoritative and Derived States)) blinded to half of metadata units |

⇒ **One-sentence verdict material**: Stepping takes minimum unit size in pool (16384), data units maintain 32768 alignment,
nodes 16384 alignment——full recovery, cost is merely doubling internal probes (guarded by header checksum);
wording "step by 32768" in D18 (Information Carried in a Block) settled item 9 must be amended. Settlement belongs to C86 (Two Unit Sizes Conflict with Scan Stepping) closeout.

### Failure Clauses (Written Hard Before Run)

Criteria 1-4 all hit (miss count == odd-slot node count cell by cell across seeds; odd-slot nodes under monopolized slot pairs constantly 0). No invalidation triggered.

### What It Cannot Answer

Counting model, 0 file operations. Probability of internal probes deceived by payload pretending to be magic not modeled
(32-byte header checksum compresses it to negligible, isomorphic to E76 (Discriminative Power of Payload Checksum));
colocation ratio 1:2 is a selected scenario point.

### Measurement Basis and Replay

- Code `scripts/experiment-apparatus/e86_scan_step.rs`
  (`cargo run --release --bin e86-scan-step`),
  raw output `e86-scan-step-2026-09-02.out` (17 lines, closing line `emitted=17`).
- N=5 seeds, output deterministic per seed, identical md5 across 5 rounds. **8 unit tests, all 6 mutations caught one by one, 0 blind spots**,
  mutation table `scripts/mutation/tables/e86_scan_step.tsv`
  (`bash scripts/mutation/mutate.sh e86-scan-step scripts/experiment-apparatus/e86_scan_step.rs scripts/mutation/tables/e86_scan_step.tsv`).

### Affected Decisions

| Decision Branch | Relationship | Review |
|---|---|---|
| D18 (Information Carried in a Block) settled item 9 | Supports | 2026-09-19 Unaffected: 16384 stepping fully recovers, 32768 stepping misses exactly all odd-slot nodes——stepping takes minimum unit size in pool, unit start aligned by class |
| D4 (Checksum Location) settled item 1 | Does not affect | 2026-09-19 Unaffected: only takes metadata side 16 KiB and data side 32 KiB as two unit sizes |
| D21 (Boundary Between Authoritative and Derived States) | Does not affect | 2026-09-19 Unaffected: only borrows "scan rebuild is sole fallback" to illustrate consequences of missing half the nodes |

## Historical Versions

Historical entries for E86 (Scan Stride and Two Unit Sizes) are centralized in [experiments-history.md](../experiments-history.md).
