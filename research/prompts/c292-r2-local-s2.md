1. A class 2 node referenced by a snapshot whose tree table entry is corrupted during rebuild. The rebuild process cannot determine the node's reference to the snapshot from the tree table alone, making it impossible to decide if the node belongs to part 1 or part 2.

2. Two holders for a class 2 node disagree on birth txg: one holder reports birth 3, another reports birth 5. Rebuild uses R1 and copies birth 3. A later reader accesses the node using the key (class 2, tree X, 3, instance 2, serial 1), but another node with the same key exists and returns incorrect data.

3. A node referenced by a snapshot at txg 5 and a clone origin at txg 5. During rebuild, R1 is impossible so D is used, setting birth to 5. After a crash before publishing, the snapshot at txg 5 treats the block as newer (birth 5 equals snapshot txg 5) and frees it immediately, causing data loss.

4. A node referenced only by the live head was originally written at txg 5. During rebuild, R2 sets its birth to current checkpoint 10. A snapshot taken at txg 7 references it, but birth 10 > 7 causes the block to be freed immediately per F1, losing data needed by the snapshot.

5. For C-prime, there is no way out of read-only state once entered. Destroying the snapshot requires writes, but the pool is read-only, creating a deadlock where the snapshot cannot be destroyed to restore write access.
