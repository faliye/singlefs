ITEM 1 OBJECTION:
The attack question asks whether abandoned journal records left in the ring could cause rollback to be undone when the chosen root falls back to instance 1's old root because B's root, C's root, and instance 3's root are all unreadable—specifically whether B's record gets replayed and the rollback is. It also questions if, when the chosen root is (1, 4), the abandoned instance 2's record would be picked up despite P2's jsn choice, and whether jsn = pool-wide maximum + 1 matches the decided rule's "continues writing from end of prefix plus 1" phrasing. It further asks who accounts for abandoned record slot reuse after ring wrap-around, implying that even with P2, the persistence of abandoned records could lead to replay or overwrite issues during ring rotation.

ITEM 1 DEFENSE:
Yes, the fix holds up. The journal replay logic stops scanning forward immediately when a record's instance number differs from the current mounted instance, as specified in the background facts. This check ensures abandoned records from previous instances (e.g., instance 2) are never replayed during recovery, regardless of their jsn or ring location. The P2 choice (max jsn + 1) prevents jsn collision with existing records during rollback, and ring wrap-around reuse of slots is safe because the instance number check blocks replay of abandoned records. The "end of prefix" ambiguity in the decided rule does not impact safety, as P2 avoids the critical overwrite risk of P1.

ITEM 1 REFUTING OBSERVATION:
If the journal replay code fails to check the instance number of each record against the current mounted instance during recovery, and a corrupt root is mounted with an incorrect instance number matching an abandoned record's instance (e.g., a root with instance 1 but incorrectly set to 2), then the abandoned record would be replayed. For example, after a rollback to instance 1's old root, a crash occurs, and during recovery, the root's instance number is corrupted to 2—causing the journal replay to process abandoned instance 2 records, undoing the rollback.

ITEM 2 OBJECTION:
The attack question asks whether the fix squares with the decided rule when the newest root's instance table is unreadable—causing no roots to be isolated by the table-based check, only newly-abandoned roots from a rollback. It also questions the conservative reading's effect: if a slot is referenced by both an abandoned root and R_old, isolating it prevents reuse even after release, causing free count divergence from df reports. It further asks if skipping ledger entries marked is_released is correct for slots referenced by predecessors.

ITEM 2 DEFENSE:
No, the fix does not hold up. The conservative reading isolates slots referenced by valid roots (e.g., R_old), violating the decided rule's "referenced solely by abandoned roots" clause. This causes the allocator to permanently mark such slots as isolated, even after they are released in valid roots' ledgers, leading to underreported free space. For example, a slot used by both an abandoned root and R_old is isolated; when R_old later releases it, the allocator still considers it isolated, so it is never reused despite being free in the live ledger.

ITEM 2 REFUTING OBSERVATION:
A slot X is referenced by an abandoned root (e.g., instance 2) and also by the current root (instance 3). The conservative reading isolates X because it is unreleased in the abandoned root's ledger. When instance 3 releases X in its ledger, the allocator still marks X as isolated. A df-style report would show less free space than actual, as X is counted as allocated but is free in the live ledger.

ITEM 3 OBJECTION:
The attack question asks what happens if F_effective drops due to a corrupt root on one device after reclaiming and reusing units—causing older roots to become candidates again. If a rollback is attempted to such a root, does rebuild_version error when reading overwritten units or panic during a walk? It also asks how invariants I-2.1 and I-3.1 judge during intervals where the newest root's F field and F_effective disagree.

ITEM 3 DEFENSE:
Yes, the fix holds up. If F_effective drops due to a corrupt device, older roots may appear in the candidate set, but rebuild_version will error when attempting to read units that have been reused (overwritten). This error is intentional and safe—it prevents inconsistent state by halting mount operations on invalid roots. The invariants I-2.1 and I-3.1 would detect the error during rebuild, ensuring no unsafe state progresses.

ITEM 3 REFUTING OBSERVATION:
During rollback to a root that references reused units, rebuild_version does not error but proceeds to read an overwritten unit. For example, if a unit was overwritten with new data but the read operation fails to detect corruption (e.g., due to a bug in disk I/O checks), the system continues processing, leading to inconsistent metadata and potential data corruption.

ITEM 4 OBJECTION:
The attack question asks whether the raise-F code path (using current's instance table) and rebuild code path (using newest root's table) can compute different oldest_valid_root values. It also asks which term dominates in reclaim_floor after generation-0 root is overwritten (txg ≥ 25), and whether F_effective or oldest_valid_root arrives first in the maximum.

ITEM 4 DEFENSE:
Yes, the fix holds up. The two code paths may compute different oldest_valid_root values, but reclaim_floor takes the maximum of F_effective and oldest_valid_root. After generation-0 root is overwritten (txg ≥ 25), F_effective typically exceeds the oldest valid root's txg, so reclaim_floor defaults to F_effective. This ensures units are not reclaimed prematurely, as F_effective reflects the highest safe threshold across devices.

ITEM 4 REFUTING OBSERVATION:
If the raise-F path computes an oldest_valid_root higher than the rebuild path's, but F_effective is lower than the rebuild path's oldest_valid_root, reclaim_floor becomes F_effective. If F_effective is lower than the oldest valid root's txg, units referenced by the oldest valid root might be reclaimed prematurely. For example, if the oldest valid root has txg 10 but F_effective is 5, reclaiming units below txg 5 could overwrite data still in use by the root with txg 10.

ITEM 5 OBJECTION:
The attack question asks whether there are other generation-0, span-1 records in the first version of the file system format besides the mkfs-written instance and tree tables. It also questions whether the publish_version path with previous set to none is reachable, noting that a pool with no published files cannot be mounted writable.

ITEM 5 DEFENSE:
Yes, the fix holds up. Mkfs writes exactly two allocation records: instance table (span 2) and tree table (span 1). No other generation-0, span-1 records exist in the initial format. The publish_version path with previous set to none is unreachable in practice (as the pool must have published files to mount writable), but the recognition mechanism is safe because it correctly identifies the tree table unit by generation 0, span 1, unreleased—there are no other candidates to misidentify.

ITEM 5 REFUTING OBSERVATION:
If mkfs accidentally writes an extra allocation record with generation 0, span 1, unreleased (e.g., due to a bug in mkfs), the code would misidentify it as the tree table. During publish_version, the system might incorrectly release the wrong unit, corrupting the filesystem. For example, if the extra record is mistaken for the tree table, the first file version might release the instance table unit instead of the tree table unit, causing immediate metadata corruption.
