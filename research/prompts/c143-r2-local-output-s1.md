1. For A: Yes. After publishing txg 100-200, journal ring wraps and overwrites records for txg 100-150. Device 0 fails, making root slots unreadable. Rollback to R_old=100. Max tree table entries from readable roots is 100. New objects reuse inode 150 (published earlier).  
For B: Same as A. Max of R_old and readable roots is 100, reusing inode 150.  
For C: No. Birth generation (txg) is monotonic. New objects have higher txg, so (inode, birth generation) pairs are unique.  
For AJ: Yes. Same scenario as A. Journal records for txg>100 are overwritten, so AJ cannot find higher inode counters. Max is 100, reusing inode 150.  
For CJ: No. CJ sets new txg = max(readable roots txg, journal checkpoint_txg) +1. Birth generation is higher, so (inode, birth generation) is unique.  

2. Yes. Applications relying on externally visible inode numbers (F14) as unique identifiers. Extent key (F7) uses inode number without birth generation, but encryption uses birth generation for security. However, if applications use inode numbers alone, reuse breaks them.  

3. AJ cost at rollback: Scan all journal records with txg > R_old for tree table inode counters and max with readable roots' tree table entries. Cost: O(J), where J is journal ring size.  
CJ cost at rollback: Scan all journal records for max checkpoint_txg and max readable roots' txg. Cost: O(J).  

4. For A: Check that inode counter after rollback ≥ max published inode number. Turns red if journal records overwritten and root slots unreadable, causing counter to be too low. Proof: Overwritten journal records and unreadable roots mean max counter is low, but published inodes exist.  
For B: Same as A.  
For C: No check needed; birth generation ensures uniqueness.  
For AJ: Same check as A. Turns red if journal records for txg>R_old are overwritten and root slots unreadable.  
For CJ: No check needed; txg ensures birth generation uniqueness.  

5. Pick AJ. Single observation changing pick: If journal ring size is too small to retain records for all txg > R_old during rollback window, making AJ ineffective.
