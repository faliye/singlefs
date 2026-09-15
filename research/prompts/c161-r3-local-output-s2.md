1. For A-R4-T: Testedaded splits and collapses were tested. When a root buffer is pushed down during collapse, seq ordering ensures correct newer value. Rebalance moves buffered messages with keys. Deletions in reverse index and LRU trees are handled correctly via front-end flush removing older messages. Accounting point deletes and monotonic stats (max) work as expected. Rollback to older root causes accounting disagreement (common to all candidates), but no specific stale read from buffer issues. Trees taller than 3 and batches with >2 keys show no failures.  
For B-D-T: During tree collapse, root buffer entry (K:V1) is pushed to child buffer (K:V2 where V2 was inserted later). Position-based ordering appends V1 after V2, making V1 appear newer despite V2 being later. This causes stale read of V1 instead of V2. Rebalance interleaved with partial push-down also fails due to position-based ordering.  
For B-G-T: Global seq counter persisted avoids same-seq issues. Casc However, during rollback to older root, buffers contain newer data which is drained before rebuild, causing accounting disagreement (common to all). Cascaded splits and collapses work due to global seq ordering.  
For C-T6: No node buffers for front-end trees (all writes go through front end, drained before publish). All tested shapes (splits, collapses, deletions, accounting, tall trees) show no failures. Rollback to older root causes accounting disagreement (common), but no buffer-specific stale reads.  

2. A-R4-T: Node layout retains seq in value (4 bytes). seq meaning unchanged (per-publish counter). Write amplification: front-end flush to leaves reduces writes. Read amplification: check front end then buffers.  
B-D-T: Node layout stores seq (4 bytes) unused. Write amplification: level-by-level push-down increases writes. Read amplification: check all buffers for position order.  
B-G-T: seq widened to 8 bytes. Pool-wide monotonic counter persisted. Write amplification: similar to B-D-T but seq ordering avoids position issues. Read amplification: check buffers with seq.  
C-T6: Node layout removes message buffers entirely for front-end trees (epsilon 0). seq remains 4 bytes but only in front end. Write amplification: front end batches and flushes to leaves minimizes writes. Read amplification: only leaves checked.  

3. A-R4-T: Check that for same key in same publish, seq increases monotonically across entries. If two entries have same seq, check fails (red).  
B-D-T: Check that for same key across levels, the entry with highest position is the most recent update. If a lower-position entry is newer than higher-position, check fails (red).  
B-G-T: Check that seq is strictly increasing across all updates globally. If two updates have same seq, check fails (red).  
C-T6: Check that every change to front-end tree leaf matches exactly one front-end entry drained in the publish. If mismatch, check fails (red).  

4. Pick C-T6. Single observation that would change pick: if the write buffer front end cannot handle the throughput of high-frequency small updates, causing significant latency or bottlenecks during flushes.
