1. For candidate A: when destroying a snapshot S, the deadlist entries for S are under S's tree ID. During destroy, they are rekeyed to the next side's tree ID. A block with birth=1.5, prev(S).txg=1, and next side's txg=3 is freed because birth > prev(S).txg, but it is still referenced by the next side. This is inherent to F1's check, not candidate-specific. However, candidate A correctly identifies and processes S's deadlist entries without additional issues. For clones, the deadlist entries are correctly scoped by tree ID.  
For candidate B: when destroying S, entries are rekeyed to next side's deadlist number. The same block (birth=1.5, prev(S).txg=1) is freed incorrectly. Additionally, if deadlist numbers are reused (unlikely due to fresh assignment), but more critically, if an inode counter claimant requires 8 reserved bytes, the deadlist number storage conflicts.  
For candidate C: entries are keyed by head tree ID + epoch txg + mapping key. Destroy cascade checks birth > prev(S).txg directly on the range (head tree ID, prev(S).txg). The same block is freed incorrectly due to F1's check, but candidate C correctly isolates S's deadlist entries without rekeying. Clones use their own tree ID and epoch, so no cross-contamination.  

2. Candidate A: 35 bytes per entry. Tree table uses 0 reserved bytes for deadlist. Snapshot creation rewrites all entries (O(N)). Destroy cascade rewrites all entries (O(M)).  
Candidate B: 35 bytes per entry. Tree table uses 8 reserved bytes for deadlist number. Snapshot creation O(1) (no rewrites). Destroy cascade rewrites all entries (O(M)).  
Candidate C: 43 bytes per entry. Tree table uses 0 reserved bytes for deadlist (previous_snapshot_txg is a standard field). Snapshot creation O(1) (no rewrites). Destroy cascade checks entries without rewrites (O(M) for scan, but no rekeying).  

3. Candidate A: key (35 bytes) differs from livelist (36 bytes) by lacking event type, but sparse side table key (27 bytes) is part of it. Tree table reserved bytes free for lineage or inode counter.  
Candidate B: key (35 bytes) aligns with livelist shape. Tree table reserved bytes used for deadlist number, leaving 16 bytes for lineage. If inode counter requires 8 bytes, lineage cannot fit.  
Candidate C: key (43 bytes) differs from livelist but includes birth in mapping key. Tree table reserved bytes fully free for lineage or inode counter.  

4. Bucketing: option (i) is chosen. Birth is already in the mapping key (F6), so no extra 8-byte birth is needed. Sortinging (ii) would duplicate birth data. Value: empty. Size is in central mapping (F10), so deadlist value need not carry size.  

5. Candidate C is picked. It avoids rekeying during snapshot creation and destroy cascade, uses tree table reserved bytes efficiently, and aligns with F5/F7. The single observation that would change the pick is if key size constraints (43 bytes) exceed system limits, forcing a trade-off to candidate A or B despite their rekeying costs.
