1. For candidate C: Publish 1 (txg=1, root R1 on device 0, inode counter=10, journal J1), publish 2 (txg=2, root R2 on device 1, inode counter=20, journal J2). Rollback to R1. New object created with inode=10. Published inode 10 is reused. Faults: none.  
   For candidate C+CJ: Same as C. Publish 1 and 2 as above. Rollback to R1. New object inode=10 reused. Faults: none.  
   For candidate C+CJ2: Same as C. Publish 1 and 2 as above. Rollback to R1. New object inode=10 reused. Faults: none.  
   For candidate G: Journal ring length 2. Publish 1 (txg=1, root R1 on device 0, journal J1 with pool inode counter=10), publish 2 (txg=2, root R2 on device 1, journal J2 with counter=20), publish 3 (txg=3, root R3 on device 0, journal J3 with counter=30, overwriting J1). Device 0 fails making R3 and J3 unreadable. Rollback to R1. Max readable pool inode counter is 20 (from J2). Head counter set to 20. New object inode=20 reused. Faults: device 0 failure.  

2. No consumer relies on the inode number alone without rolling back with the root; all structures using inode numbers are part of the versioned root data.  

3. For C: 0 extra bytes, first-transaction bytes unchanged, rollback reads accounting tree from R_old. For C+CJ: same as C. For C+CJ2: same as C. For G: +8 bytes per root record and journal header, first-transaction bytes increase by 8, rollback and mount read all readable root records and journal records to compute max pool-wide inode counter.  

4. For C: Check that no two objects share the same inode number. Turns red if reuse occurs. Proof: reuse history in question 1 shows two objects with same inode number. For C+CJ: same as C. For C+CJ2: same as C. For G: Check that current head inode counter >= max pool inode counter from readable records. Turns red if counter is too low. Proof: G reuse history shows counter set to 20 but inode 20 was previously published.  

5. Pick G. Single observation that would change the pick: if journal records can be permanently lost due to hardware failures, then G may fail to prevent reuse; if journal records are guaranteed durable and readable, G is safe.
