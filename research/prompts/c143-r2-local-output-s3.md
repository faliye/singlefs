1.  
A. Yes. Example: R_old (txg=10) is readable. Roots at txg=20 and txg=30 have unreadable root slots on both devices. Tree table entries for these roots are unread. New head starts with R_old's inode counter (e.g., 100). Later, object with inode 150 (previously published by txg=20) is reused.  
B. Same as A. Unreadable root slots lead to max counter being R_old's value.  
C. Yes. Example: R_old (txg=10) counter=100. New head starts at 100. Publishes object with inode 150. This reuses inode 150 (previously published by txg=20), violating F2.  
AJ. Yes. Journal ring wraps: R_old (txg=10), then txg=11-100 written, then txg=101-200 overwrites txg=11-100. Root slots for txg>10 unreadable. AJ scans journal for txg>10 but records overwritten. Max counter=R_old's value. New head reuses published inode numbers.  
CJ. Yes. CJ sets txg but does not affect inode counter. Example: R_old (txg=10) counter=100. New head starts at 100. Publishes object with inode 150 (previously published), reusing inode number.  

2.  
- Extent key (F7): (locality_id, inode, offset) does not include birth generation. Reused inode numbers cause incorrect data mapping.  
- Reclaim state of deleted inodes (F17): Structures keyed by inode number. Reuse leads to incorrect reclaim state.  
- Externally visible numbers (F14): User-space applications (e.g., ls -i) rely on inode numbers. Reuse causes inconsistent file identification.  

3.  
AJ: At rollback, scan all journal records with txg > R_old. Cost = O(journal ring size).  
CJ: At first new root after rollback, scan entire journal ring for max checkpoint_txg. Cost = O(journal ring size).  

4.  
A: Check = new head's inode counter >= all published inode numbers. Turns red if counter < published max. Proof: Unreadable newer roots cause counter to be R_old's value, which may be less than published max.  
B: Same as A.  
C: Check = (inode, birth generation) pairs unique. Turns red if pair reused. Proof: Inode reuse with different birth generation still violates F2 (inode numbers reused).  
AJ: Check = new head's inode counter >= all published inode numbers. Turns red if counter < published max. Proof: Journal records overwritten and root slots unreadable cause counter to be too low.  
CJ: Check = txg of new root is max of readable roots and journal records. Turns red if txg too low. Proof: Journal records overwritten cause txg to reset to older value.  

5.  
Pick AJ. Single observation that would change pick: If journal records are not reliably durable (e.g., FUA writes can fail or be lost), then AJ's reliance on journal records becomes unsafe and a different approach is needed.
