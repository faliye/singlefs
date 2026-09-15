1. For all candidates, a history where a block with birth 1 is added to a head's deadlist when the head's previous_snapshot_txg is 1, then a snapshot S2 at txg 2 is created, and snapshot S1 (with prev(S1)=0) is destroyed. During destroy cascade, entries in S2's deadlist with birth > 0 are freed, but S2 references the block (birth 1 <= 2), so it is freed incorrectly.

2. Candidate B: key 35 bytes; tree table uses 8 bytes; snapshot creation rewrites 0 entries; destroy cascade rewrites min(N,M) entries; lookup O(1) (previous_snapshot_txg in tree table). Candidate C-interval: key 43 bytes; tree table uses 0 bytes; snapshot creation rewrites 0 entries; destroy cascade rewrites 0 entries; lookup O(1). Candidate C-prime: key 43 bytes; tree table uses 0 bytes; snapshot creation rewrites 0 entries; destroy cascade rewrites all non-freed entries; lookup O(1).

3. Candidate B: deadlist location 8 fits; lineage labels 16, livelist counter 8, inode watermark 8 fit in remaining 16 bytes. Candidate C-interval: all claimants fit in 24 bytes (no deadlist location used). Candidate C-prime: all claimants fit in 24 bytes (no deadlist location used).

4. Check: for any deadlist entry, if the block is referenced by a snapshot, then birth must not be > prev(S).txg for any snapshot S being destroyed where the entry is in the next newer side's deadlist. This check turns red if a referenced block is freed during destroy cascade, proven by the history in question 1.

5. Candidate C-interval. The single observation that would change the pick is if the destroy cascade should free entries with birth > S.txg instead of birth > prev(S).txg, which would resolve the correctness issue for all candidates.
