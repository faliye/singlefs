1. Correctness:  
Candidate A: Incorrect. When destroying snapshot S, re-keying entries to next side's tree ID does not ensure correct freeing. Blocks with birth > prev(S).txg may be freed, but if birth <= prev(S).txg, they remain, but some may still be referenced by newer snapshots. Example: A block with birth=3 and death=7, prev(S).txg=5. When S is destroyed, birth=3 <=5, so not freed, but it should be freed if no snapshots reference it (since death=7 < newer snapshot txg).  
Candidate B: Incorrect. Similar to A. Deadlist number storage does not resolve the issue. Blocks with birth <= prev(S).txg are never freed, even if no snapshots reference them. Example: Birth=3, prev(S).txg=5. After merging, birth=3 <=5, so not freed, but should be.  
Candidate C: Correct. Key is (head tree ID, epoch txg, mapping key), where epoch txg is head's previous_snapshot_txg at drop time. S's deadlist range is (head tree ID, prev(S).txg). Destroying S frees entries with birth > prev(S).txg. Blocks with birth > prev(S).txg were dropped after prev(S) was created, so S is the latest snapshot referencing them. Freeing them is safe. Blocks with birth <= prev(S).txg were dropped before prev(S) and referenced by older snapshots, so not freed. Clones work correctly because head tree ID is unique and epoch txg is per drop time.  

2. Cost:  
Candidate A: Snapshot creation re-keys all deadlist entries (O(N) per creation). Destroy cascade re-keys all entries to next side's tree ID (O(N) per destroy). Total per entry: 35 bytes.  
Candidate B: Snapshot creation: no re-keying (assign deadlist number, O(1)). Destroy cascade re-keys all entries to next side's deadlist number (O(N)). Total per entry: 35 bytes.  
Candidate C: Snapshot creation: no re-keying (epoch txg changes automatically, O(1)). Destroy cascade re-keys head tree ID part of entries (O(N)). Total per entry: 43 bytes.  

3. Alignment:  
Candidate A: Key (35 bytes) does not match livelist (36 bytes), but similar. Tree table reserved bytes: 24 bytes, but deadlist number not stored there. Inode counter would require 8 bytes, leaving 16 for lineage (possible).  
Candidate B: Key (35 bytes) matches livelist shape (36 bytes with event type). Tree table reserved bytes: 8 bytes for deadlist number, 16 for lineage. If inode counter needs 8 bytes, deadlist number would need to be stored elsewhere, conflicting with lineage.  
Candidate C: Key (43 bytes) differs from livelist (36 bytes) but includes birth via mapping key. Tree table reserved bytes not used for deadlist key; reserved bytes for lineage or inode counter remain unaffectedable.  

4. Bucketing and value:  
Option (i) no extra bucket: Birth is already in the central mapping key (27 bytes), so no need for extra birth field. Value: Empty. Size and location come from central mapping key. This aligns with F5 (birth inline for O(1) destroy filter) and minimizes entry size.  

5. Candidate C is picked. The single observation that would change the pick: If head tree ID is not unique across the pool (e.g., reused), which would cause key collisions. However, head tree IDs are unique by design, so this is not a concern.
