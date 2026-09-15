1.  
1. A-R4: Front-end flushes key X to leaf with value A. Non-front-end update writes X to root buffer with value B (older than A). Read returns B from buffer before leaf, stale value.  
2. B-D: No sequence found causing stale read or accounting disagreement. Shapes tried: node splits during push down, cross-publish updates, K>1, partial fsync, crash during drain. All cases handled correctly by buffer replacement.  
3. B-mono: No sequence found causing stale read or accounting disagreement. Global seq ensures correct ordering across publishes. Shapes tried: same as B-D.  
4. C: No sequence found causing stale read or accounting disagreement. No node buffers eliminate buffer-related staleness. Shapes tried: node splits, K>1, partial fsync, crash during drain. All cases correct due to direct leaf writes.  

2.  
1. A-R4: Node layout adds non-front-end buffers; seq remains 4 bytes in value (used for same-window landing); tree axis unchanged (buffers only for non-front-end updates); write amplification lower for front-end (direct leaf writes), higher for non-front-end (buffer push down); read amplification higher due to multiple checks (front end, buffers).  
2. B-D: Node layout holds one message per key per buffer; seq not used (width unchanged but redundant); tree axis unchanged (buffers per F5); write amplification higher (level-by-level push down); read amplification higher (check all buffer levels).  
3. B-mono: Node layout same as B-D; seq becomes pool-wide monotonic counter (width unchanged); tree axis unchanged; write/read amplification same as B-D.  
4. C: Node layout removes all buffer fields; seq width unchanged but only used for front-end sorting (no buffer usage); tree axis all trees epsilon 0 (no buffers); write amplification lowest (batched direct leaf writes); read amplification lowest (direct leaf access).  

3.  
1. A-R4: Check that buffer entries for a key never have value older than leaf value. Turns red if non-front-end update writes older value to buffer after front-end flush. Prove by validating each buffer write against leaf seq.  
2. B-D: Check that buffer replacements always use newer position (not seq). Turns red if older update replaces newer in buffer. Prove by tracking update timestamps across levels.  
3. B-mono: Check that global seq strictly increases for each update. Turns red if seq decreases for same key. Prove by validating seq monotcrement on each write.  
4. C: For front-end trees, verify all writes originate from front end. Turns red if direct write to front-end tree bypasses front end. Prove by tagging writes with front-end flag and checking at tree entry point.  

4.  
Pick C. Single observation changing pick: If inode tree requires buffers despite being "cannot" (F5), but F5 explicitly states "cannot" trees must have epsilon 0. If evidence shows "cannot" trees need buffers for performance or correctness, C would be invalid. However, F5's definition of "cannot" implies no buffers, so this observation is unlikely.
