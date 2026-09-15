1. For C-interval with upper bound: a block with birth=10 dropped by head H when previous_snapshot_txg=5 is in deadlist. When snapshot S (txg=15) is destroyed, the intent range [15, hi) does not include epoch=5. The block is not processed and remains unfreed despite birth>prev(S).txg=5, violating correctness.  
For B with ordering: if cascade intents run out of order (e.g., destroy S2 before S1 when S2 was written later), rekeying deadlist numbers incorrectly moves entries, causing a referenced block to be freed prematurely.  
For C-prime with reserve: no such scenarios found; birth checks during processing ensure correct freeing.  

2. C-interval: key 43 bytes; tree table bytes 0; each destroy requires scanning all entries in [lo, hi) range to check birth inline.  
B with ordering: key 35 bytes; tree table bytes 8 for deadlist number; each destroy requires scanning all entries in deadlist number to check birth inline.  
C-prime with reserve: key 43 bytes; tree table bytes 8 for tree ID of head's latest snapshot; each destroy requires re-keying entries in next side's deadlist segment.  

3. C-interval: 24 reserved bytes fit children counter (8), origin link (8), inode watermark (8).  
B with ordering: 24 reserved bytes fit deadlist number (8), lineage labels (16); children counter and origin link exceed space.  
C-prime with reserve: 24 reserved bytes fit tree ID segment (8), children counter (8), origin link (8).  

4. C-interval: check that all blocks with birth > prev(S).txg are freed during destroy. If not, the range [lo, hi) does not align with birth, turning red. Proof: birth values are not in key, so range query cannot filter birth directly.  
B with ordering: check that deadlist numbers are swapped only when intents run in order. If out-of-order processing occurs, deadlist numbers become inconsistent, turning red. Proof: deadlist numbers are global and reused, causing conflicts without strict ordering.  
C-prime with reserve: check that re-keyed entries match the next side's previous snapshot tree ID. If not, entries are misplaced, turning red. Proof: tree ID is stored inline and verified during re-keying.  

5. Pick C-prime with reserve. The single observation that would change the pick is if birth must be included in the key to avoid full scans during destroy operations, as current candidates all require scanning entries to check birth inline.
