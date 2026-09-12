1. Rollback sequence: publish generation 1 (R1), publish generation 2 (R2), roll back to R1, dereference a block only present in R1.  
   - Option A: reader follows map root pointer from tree table entry in R1's tree table unit. Entry exists.  
   - Option B: reader follows map root pointer from R1's root record. Entry exists.  
   - Option C: reader follows map root pointer from superblock (latest version), which points to R2's map root. Entry does not exist for R1.  

2. Publication order step for new map root pointer durability:  
   - Option A: durable at units step (tree table unit written).  
   - Option B: durable at root slot step.  
   - Option C: durable at superblock write step (not in publication order steps). Option C needs a fourth ordering point for superblock write.  

3. Mount chain:  
   - Option A:  
     - root record → tree table pointer (authoritative, bootstrap exemption)  
     - tree table unit → map tree entry root pointer (authoritative, map tree nodes exempt)  
     - map root node  
   - Option B:  
     - root record → map pointer (authoritative, map tree nodes exempt)  
     - map root node  
   - Tree kind field (2 bytes) is sufficient to distinguish authoritative hints for map tree vs hints for other trees.  

4. Tree table rules for map entry under A:  
   - Old reader skips unknown kinds: tree kind 5 known, no issue.  
   - Every snapshot is one entry: map tree is pool-wide not snapshot, no issue.  
   - Creating clone writes watermark row: map tree not per-head, no issue.  
   - Destroying head deletes by prefix: map tree not per-head, no issue.  
   - Previous_snapshot_txg per head: map tree pool-wide, previous_snapshot_txg=0, no issue.  
   - Enumerating live snapshots: map tree not a as snapshot, no issue.  
   - No operation sequence breaks rules; tree kind field sufficient.  

5. Tree id under B and C:  
   - Class 2 node headers for map tree carry tree id.  
   - Tree id watermark does not issue it (no tree table entry).  
   - Tree id 0 conflict: instance table unit uses tree id 0 for no owner; map tree cannot use tree id 0. If map tree uses tree id 5, collision possible when new tree gets id 5.  

6. Missing candidates:  
   - No other placements satisfy all constraints. All valid options covered by A B C.
