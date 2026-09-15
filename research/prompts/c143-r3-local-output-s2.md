1. For each candidate, no reachable history exists where a published identity (inode number, object birth generation) is reused after rollback. Shapes tried include: rollback to R_old with varying root slot states (unreadable, write failures), journal ring wrapping, multiple rollbacks in sequence, crash during rollback retry, destroyed and revived heads, and clone heads combined with rollbacks. In all cases, birth generation (txg) is strictly monotonic per publish, ensuring unique pairs. For candidate G, pool-wide counter prevents inode reuse entirely. For others, birth generation differences prevent pair reuse even if inode numbers repeat.

2. The extent key in the mapping tree (F7) relies on inode number alone without birth generation. This is used in the B-tree structure for locating extents, where any reuse of inode numbers would cause data corruption as old and new objects share the same key.

3. Cost analysis:
   - C: 0 extra bytes. Rollback reads only R_old's accounting tree. Mount reads current root's accounting tree. First-transaction bytes unchanged.
   - C+CJ: 0 extra bytes. Rollback scans all readable journal records for max txg. Mount reads current root. First-transaction bytes unchanged.
   - C+CJ2: 0 extra bytes. Rollback and mount both scan all readable journal records for max txg. First-transaction bytes unchanged.
   - G: 8 bytes per root record and per journal header. Rollback and mount scan all readable root slots and journal records for max inode counter. First-transaction bytes increase by 8 bytes for root and journal headers.

4. Check for each candidate:
   - C: Verify (inode, birth generation) uniqueness. Turns red if duplicate pair exists. Proof: if txg is not incremented (e.g., bug in publish logic), duplicate birth generation could occur with same inode.
   - C+CJ: Verify first new root txg is max of readable roots and journal records +1. Turns red if txg is lower. Proof: if journal records are partially unreadable and max txg is miscomputed, txg could be too low.
   - C+CJ2: Same as C+CJ but for mounts. Turns red if mount txg is not max of all readable records. Proof: same as C+CJ but during mount.
   - G: Verify each head's inode counter ≤ pool-wide max. Turns red if head counter exceeds pool-wide max. Proof: if head increments counter without updating pool-wide max (e.g., crash during publish), counter could be higher.

5. Pick G. The single observation that would change the pick is if the extent key (F7) could be modified to include birth generation. If so, C would suffice as it ensures pair uniqueness without extra bytes, resolving the extent key issue. Since the current design fixes extent key to exclude birth generation, G is necessary to prevent inode reuse.
