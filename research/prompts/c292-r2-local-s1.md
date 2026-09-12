1. A node exists in the deadlist but the deadlist file is corrupted during rebuild. The tree table shows references only from the live head, so the system treats it as part 1. However, the deadlist corruption hides the fact that the node is referenced by the deadlist, making its membership impossible to decide from available data.

2. Two holders of a class 2 node disagree on birth txg: one shows birth=3, another shows birth=5. During rebuild, R1 copies the holder with birth=3. A snapshot taken at txg=4 references this node. The system treats birth=3 ≤ 4 as valid, but the node was actually published at txg=5, so it should not exist in the snapshot. Readers see incorrect data.

3. A node is referenced by snapshot S1 at txg=5 and S2 at txg=10. D sets its birth to 5 (smallest txg). The current instance has T_pub=3. Per F5, birth=5 > T_pub=3 means the node is unpublished. After rebuild, the system sweeps it as garbage despite being referenced by snapshots, causing data loss.

4. After rebuild using R2 for a node referenced only by the live head (birth set to current checkpoint txg=10), a clone is created from a snapshot with txg=5. The clone’s previous_snapshot_txg=5. Since the node’s birth=10 > 5, F2 triggers immediate freeing. The clone loses the node data despite needing it.

5. No way out of read-only state once entered for C-prime. Destroying the snapshot requires writes, but the pool is read-only, creating an impossible deadlock.
