## E87 Placement of Fixed Structures — Run (2026-09-02, Arithmetic Exhaustion, 8 Unit Tests / 5 Mutations All Caught): Single Copy of System Configuration = Disk 0 Loss Makes Entire Pool Unmountable; One Copy per Disk + Journal Mirroring ⇒ All 8 Cells Mountable, Zero Window Loss; Incidental Discovery: "mkfs Must Seed Generation 0 Root into All Regions"

[中文](87-固定结构的放置.md) · **English** · [日本語](87-固定構造の配置.ja.md)

Decision material for C79 (Nobody Has Settled the Placement of the System Configuration and the Journal Ring): on the 2-disk initial version, 2x2 combinations of system configuration copies x journal ring
copies, exhaustively checking single-disk failure across every cell. The root ring half is modeled according to settled clauses
(R=3 round-robin region assignment, per-region identity storage, per-publish counting).

⚠️ **An existing rule never previously directed at the journal**: hard lower bound of D2 (RAID Stripe Policy) settled item 6
"w >= 2——zero-redundancy stripes forbidden from issuance". Journal records are also writes——a single-disk journal ring is a write path constantly w=1,
which the literal text of the rule should have barred, yet no one in the repository had juxtaposed the two statements.

### Criteria (Written Hard Before Run; mkfs Seeding Clause Supplemented in Model During Initial Trial, Revision Recorded in Source Header Prior to Any Official Repository Commitment)

1. Single copy of system configuration losing Disk 0 must be unmountable (single-point-of-failure machine check form); one copy per disk leaves all 8 cells mountable.
2. Root rollback worst-case nailed down by round-robin arithmetic (losing Disk 1 worst-case 1 generation, losing Disk 0 worst-case 2 generations——on 2 disks R=3 inevitably assigns
   one disk to carry two regions).
3. Single copy of journal losing Disk 0 loses replay window (upper bounded by one T_time window); mirrored across all cells loses nothing, with each record constantly 2x writes.
4. Selection of cell belongs to C79 (Nobody Has Settled the Placement of the System Configuration and the Journal Ring) closeout decision, delivering table.

### Empirical Measurements (Full Table 8 Cells in Artifact; Four Decisional Cells)

| Cell | Disk 0 Lost | Disk 1 Lost |
|---|---|---|
| Single copy of system configuration + single copy of journal | **Unmountable** (data, root, and journal surviving is of no avail) | Mountable, root rollback <=1, window preserved |
| **System configuration per disk + mirrored journal** | **Mountable**, root rollback <=2, **window preserved** | Mountable, root rollback <=1, window preserved |

Steady-state cost of mirrored journal: 2 writes per record——precisely the w=2 lower bound that D2 (RAID Stripe Policy) settled item 6
inherently demands of any write; updates of one system configuration copy per disk follow >=2 slot alternation isomorphic to
D23 (Journal Role & Format) settled item 3 (update protocol belongs to closeout decision).

**Prerequisite discovered during trial run**: When the model does not seed the mkfs initial root, the first publish lands on Disk 1, and losing Disk 1 ⇒ **total destruction**
(no root available to roll back to)——⇒ **mkfs must seed Generation 0 root into all regions** (covering the cell of "fresh mkfs, zero publishes,
loss of any single disk"), modeled as seeds and pinning the unseeded arm hole with a unit test.

### Failure Clauses (Written Hard Before Run)

Criteria 1-3 all hit. No invalidation triggered.

### What It Cannot Answer

Pure arithmetic, 0 file operations. Update protocol of system configuration itself (slot alternation fields and identifiers) not modeled;
disks lost and regained belongs to C88 (Timeline discrimination for root ring unimplemented); slot count S per region simplified to 1
(more slots only deepen intra-region rollback, without changing inter-region outcomes).

### Measurement Basis and Replay

- Code `scripts/experiment-apparatus/e87_fixed_placement.rs`
  (`cargo run --release --bin e87-fixed-placement`),
  raw output `e87-fixed-placement-2026-09-02.out` (10 lines, closing line `emitted=10`).
- Pure arithmetic exhaustion, identical md5 across 5 rounds. **8 unit tests, all 5 mutations caught one by one, 0 blind spots**,
  mutation table `scripts/mutation/tables/e87_fixed_placement.tsv`
  (`bash scripts/mutation/mutate.sh e87-fixed-placement scripts/experiment-apparatus/e87_fixed_placement.rs scripts/mutation/tables/e87_fixed_placement.tsv`).

### Affected Decisions

| Decision Branch | Relationship | Review |
|---|---|---|
| D22 (Synthesizing Unit Atomicity) settled item 8 | Supports | 2026-09-19 Unaffected: single copy of system configuration losing Disk 0 makes entire pool unmountable, one copy per disk plus journal mirroring leaves all 8 cells mountable with zero window loss, discovering mkfs must seed Generation 0 root into all regions |
| D22 (Synthesizing Unit Atomicity) settled item 14 | Supports | 2026-09-19 Unaffected: on 2 disks R = 3 losing Disk 1 worst-case 1 generation, Disk 0 worst-case 2 generations, which are the two figures for disk count degradation when devs < R |
| D22 (Synthesizing Unit Atomicity) settled item 16 | Supports | 2026-09-19 Unaffected: on two disks R = 3 one disk inevitably carries two regions, making ownership 0 / 1 / 0 the sole normative specification |
| D2 (RAID Stripe Policy) settled item 7 | Supports | 2026-09-20 Unaffected: on two disks R = 3 one disk inevitably carries two regions, combined with pigeonhole lower bound yielding ownership 0 / 1 / 0 as sole normative specification, serving as rationale for hardcoding three regions 0 / 1 / 0 in first version; journal mirroring writing each record twice is precisely settled item 6 w >= 2 lower bound, E87 (Placement of Fixed Structures) merely directs it at journal |
| D23 (Journal Role & Format) | Does not affect | 2026-09-19 Unaffected: only cites >=2 slot alternation isomorphic to settled item 3 |

## Historical Versions

Historical entries for E87 (Placement of Fixed Structures) are centralized in [experiments-history.md](../experiments-history.md).
