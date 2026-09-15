1. Correctness:  
   Candidate A: Incorrecting a snapshot requires re-keying all deadlist entries to the new snapshot's tree ID. When destroying a snapshot S, entries in S's deadlist have birth ≤ prev(S).txg, so none are freed prematurely. Clones referencing blocks in S's deadlist remain valid as birth ≤ clone's txg. Correct.  
   Candidate B: Snapshot creation reassigns deadlist number without re-keying entries. Destroying S merges entries into next side with re-keying. Birth ≤ prev(S).txg for all entries in S's deadlist, so no premature frees. Clones handled correctly. Correct.  
   Candidate C: Key = (head tree ID, epoch txg, mapping key). Snapshot creation changes head's previous_snapshot_txg, so new drops use new epoch. Destroying S merges range (head tree ID, prev(S).txg) into next side. Birth ≤ prev(S).txg for all entries in the range, so no premature frees. Clones use their own tree ID and epoch, unaffecteding references valid. Correct.  

2. Cost:  
   Candidate A: 35 bytes per entry. Snapshot creation rekeys all n entries (O(n)). Destroy cascade rekeys all entries in S's deadlist (O(n)).  
   Candidate B: 35 bytes per entry. Snapshot creation O(1). Destroy cascade rekeys all entries in S's deadlist (O(n)).  
   Candidate C: 43 bytes per entry. Snapshot creation O(1). Destroy cascade rekeys all entries in S's deadlist (O(n)).  

3. Alignment:  
   Candidate A: Key matches livelist shape (head tree ID + mapping key) without event type. Yes.  
   Candidate B: Deadlist number stored in reserved tree table bytes. If inode counter needs 8 bytes, reserved bytes (24) cannot fit deadlist number (8), lineage (16), and inode counter (8) totalinging 32 > 24. No.  
   Candidate C: Key does not use reserved bytes for deadlist number. Reserved bytes can fit lineage (16) and inode counter (8) within 24. Yes.  

4. Bucketing and value:  
   Bucketing (i): No extra bucket. Birth is inline in the central mapping key (F5), so no need for separate birth field.  
   Value: Empty. Location is derived from central mapping (F10), so no need to store size.  

5. Candidate C. Single observation that would change pick: If key size must be ≤36 bytes (to livelist), candidate C's 43-byte key would be too large.
