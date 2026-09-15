1. For candidate B: Head H (txg 15, previous_snapshot_txg 5 from S2). Create S1 (txg 10), head's previous_snapshot_txg becomes 10. Block b (birth 4) dropped by head, added to deadlist (owner H's tree ID). Destroy S2; next newer side N is S1. Free entries in N's deadlist with birth > 0 (prev(S2).txg=0). Birth 4 > 0, freed. S1 references it (birth 4 <= 10), error.  
For candidate C-interval: Same scenario. Block b has epoch 5. S1's deadlist is [5, 10). Destroy S2; N is S1. Free entries in N's deadlist with birth > 0. Birth 4 > 0, freed. S1 references it, error.  
For candidate C-prime: Block b owner (head tree ID, S2's tree ID). S1's deadlist is (head tree ID, S2's tree ID). Destroy S2; N is S1. Free entries in N's segment with birth > 0. Birth 4 > 0, freed. S1 references it, error.  

2. B: Key size 35 bytes. Tree table uses 8 bytes. Snapshot creation: rewrite all head deadlist entries (O(n)). Destroy cascade: re-key smaller list (O(min(|S|, |N|))). Lookup at drop: O(1) (read head's deadlist number).  
C-interval: Key size 43 bytes. Tree table 0 bytes. Snapshot creation: 0 rewrites. Destroy cascade: 0 rewrites. Lookup at drop: O(1) (read head's previous_snapshot_txg).  
C-prime: Key size 43 bytes. Tree table 0 bytes. Snapshot creation: 0 rewrites. Destroy cascade: re-key all entries in N's deadlist (O(|N|)). Lookup at drop: O(1) (read head's latest snapshot tree ID).  

3. B: Deadlist location (8 bytes) fits. Lineage labels (16 bytes) fits in remaining 16. Livelist counter and inode watermark squeezed out.  
C-interval: Deadlist location squeezed out. Lineage labels (16 bytes) and livelist counter (8 bytes) fit (total 24). Inode watermark squeezed out.  
C-prime: Same as C-interval.  

4. B: Check that for any deadlist entry, if referenced by snapshot S, birth <= prev(S).txg. Turns red when birth > prev(S).txg but referenced. Proven by scenario where birth 4 > 0 (prev(S2).txg) but S1 references it.  
C-interval: Same check. Turns red when birth > prev(S).txg but referenced. Proven by same scenario.  
C-prime: Same check. Turns red when birth > prev(S).txg but referenced. Proven by same scenario.  

5. Pick C-interval. Single observation that would change pick: if the free step condition is corrected to birth > N.txg (next newer side's txg) instead of birth > prev(S).txg, then C-interval remains optimal; otherwise, all candidates are flawed.
