1. For Arm B:  
   - Create file with data block D1 (type 1) and index node I1 (type 2) at location L1 in transaction group 1.  
   - Take snapshot S1 in transaction group 2.  
   - Delete the file; D1 and I1 are added to deadlists. I1's deadlist entry is stored by location L1 (since Arm B lacks type 2 in central map).  
   - Move I1 to location L2; write I1 to L2 but do not update S1's parent pointer (S1 is read-only), so S1 still references L1.  
   - Destroy S1; process deadlist for I1. System frees L1, but I1 is now at L2. L1 is freed incorrectly (may be reused), and L2 remains unfreed, causing a leak.  
   Under Arm A: The same sequence does not fail. Arm A includes all types in the central map. Moving I1 updates the central map entry. When destroying S1, the deadlist entry for I1 resolves to L2 via the central map, which is correctly freed.  

2. For Arm B moving a type 2 node referenced by snapshot S:  
   - Move transaction group 8: write new I1 (type 2) at location L_new, birth 8; new parent D (type 2) at L_D_new, birth 8; new root R (type 2) at L_R_new, birth 8.  
   - When S (txg 5) is destroyed at txg 10:  
     - Old I1 (birth 3) has death 10; 3 ≤ 5 < 10 → valid reference.  
     - Old D (birth 4) has death 10; 4 ≤ 5 < 10 → valid reference.  
     - Old R (birth 2) has death 10; 2 ≤ 5 < 10 → valid reference.  
   - New blocks (birth 8) are not referenced by S (8 > 5), so F1 holds. No violation.  

3. F8 states trees are rebuilt from leaves, not from index node headers. To rebuild the tree, index node contents are required to reconstruct parent-child relationships. However, F8 prohibits reading index node headers during rebuild. Thus, it is impossible to rebuild the tree without reading index node headers, making the scan-rebuild path impossible.  

4. Rule: Type 2 or 3 units referenced by any snapshot are never moved.  
   - Disk-loss rebuild: A disk failure occurs; the failed disk contains a type 2 unit referenced by snapshot S. Rebuild requires copying this unit to a new disk, but the rule blocks moving it, causing data loss.  
   - Compaction: During compaction, type 2 units referenced by snapshots cannot be moved, leading to persistent space fragmentation and inability to reclaim space.  
   - Unit that can never be freed: The rule does not prevent freeing units when snapshots are destroyed. However, if a snapshot is never destroyed, the referenced type 2 unit remains unfreed. This is not caused by the rule itself but by the snapshot's existence.
