1. For candidate C: Publish txg=1 with root R1 (inode counter=10). Publish txg=2 with corrupted journal record. Recovery sees only R1 (txg=1), sets new txg=2. Accounting tree from R1 sets inode counter=10. New object uses inode 10 with birth generation 2. Previous object with inode 10 existed in txg=2 (but journal corrupted, invisible). Pair (10,2) reused. Faults: journal record corrupted (1 fault).  
For candidate C+CJ: Same as C. Journal record corrupted, max txg from roots is 1, new txg=2. Inode counter reset to 10. Pair (10,2) reused. Faults: 1.  
For candidate C+CJ2: Same as C. Faults: 1.  
For candidate G: Pool-wide inode counter stored in root records. Root R1 has counter=10. Journal record corrupted. Max counter=10. New object uses inode 10 with birth generation 2. Previous object with inode 10 existed in txg=2 (invisible). Pair (10,2) reused. Faults: 1.  

2. Yes. The extent key (F7) is (locality_id, inode, offset), which uses inode number alone without birth generation.  

3. Cost:  
- C: 0 extra bytes. Rollback reads accounting tree from R_old. Mount reads root's accounting tree.  
- C+CJ: Same as C.  
- C+CJ2: Same as C.  
- G: 8 bytes per root record and journal header. Rollback and mount read all root records and journal records to find max inode counter.  

4. For C: Check that all (inode, birth generation) pairs are unique. If duplicate, red.  
For C+CJ: Same as C.  
For C+CJ2: Same as C.  
For G: Check current pool-wide inode counter is >= all used inode numbers. If not, red.  

5. Pick C. Single observation changing the pick: if a consumer relies solely on inode number without birth generation check (e.g., a legacy tool), but the design already uses birth generation in encryption for safety, so C remains best.
