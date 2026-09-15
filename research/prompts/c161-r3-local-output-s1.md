1. For A-R4-T: A sequence where publish 1 updates key K with seq 3, then publish 2 updates K with seq 1. During flush, A-R4-T discards publish 2's update (seq 1 < 3), but publish 2 is later, causing stale read.  
For B-D-T: Same cross-publish issue (publish 2 seq 1 discarded as older than publish 1 seq 3). Also, root buffer has K V1 (seq 1), child buffer has K V2 (seq 2); pushing down root buffer replaces child's V2 with V1, causing stale read.  
For B-G-T: No failure case found. Global seq orders across publishes; counter persists correctly.  
For C-T6: Front-end trees drained to leaves before publish, so cross-publish order handled by replay. But inode tree (non-front-end) buffers may have seq per publish window; publish 1 K seq 3, publish 2 K seq 1 could cause stale read if buffers don't track publish number.  

2. A-R4-T: Node layout seq 4 bytes; seq per publish window; no persisted counters; tree axis: front-end trees epsilon 0, others have buffers; write amplification low for front-end updates, medium for others; read amplification medium.  
B-D-T: Node layout seq 4 bytes unused; seq unused; no persisted counters; tree axis: all trees have buffers; write amplification high (root to children); read amplification low.  
B-G-T: Node layout seq 8 bytes; seq pool-wide monotonic counter; counter persisted per publish; tree axis: all trees have buffers; write amplification high; read amplification low.  
C-T6: Node layout front-end trees no buffers; inode tree buffers with seq per publish window; no persisted counters; tree axis: front-end trees epsilon 0, inode tree epsilon 0.65; write amplification low for front-end, medium for inode; read amplification low for front-end, medium for inode.  

3. A-R4-T: Check that for same key, seq in buffer hierarchy strictly increases from leaves to root. Turns red when cross-publish updates (e.g., publish 2 seq 1 < publish 1 seq 3) due to per-publish seq. Proven by simulating cross-publish updates.  
B-D-T: Check that buffer messages order by publish number or seq. Turns red when root buffer older message replaces child buffer newer message. Proven by root buffer V1 (seq 1) vs child V2 (seq 2) push-down.  
B-G-T: Check that seq is globally monot monotonic and persisted. Turns red if counter not persisted before crash (e.g., crash after increment but before write). Proven by simulating crash during counter update.  
C-T6: Check that front-end tree leaf updates match drained entries. Turns red if leaf update not from drained entry. Proven by modifying leaf directly without front-end drain.  

4. Pick B-G-T. Single observation that would change pick: if the 8-byte seq and counter persistence cause unacceptable write latency or storage overhead in practice.
