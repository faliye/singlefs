1. Read correctness:  
   Candidate A: Yes, for a single tree, only one buffer level (front end or node buffers) exists. Reads check front end then leaves for front end trees, or node buffers then leaves for node buffer trees. Seq within a publish orders entries; cross-publish ordering relies on publish sequence number stored elsewhere.  
   Candidate B: No, seq per publish cannot order across publishes (e.g., seq 2 from older publish may be chosen over seq 1 from newer publish).  
   Candidate C: Yes, same as A. Each tree uses only one level; seq within publish orders entries, cross-publish ordering uses publish sequence number.  

2. Crash points:  
   Candidate A: Crash before publish: undrained front-end entries lost. Crash after publish (if drained to leaves): accounts match root. Journal recovery replays root fields; undrained entries not recoverable.  
   Candidate B: Crash before publish: undrained entries lost. Crash after publish (if buffers drained to leaves): accounts match root. If not drained, accounts stale. Journal recovery does not restore buffer contents.  
   Candidate C: Same as A. Crash before publish: entries lost. Crash after publish (if drained to leaves): accounts match root. Undrained entries lost; journal recovery does not restore them.  

3. Cost and format:  
   Candidate A: Freeze layer 4 includes message buffers only for node buffer trees. Seq unchanged. Journal record unchanged (F10). Write amplification: front end batches reduce writes; node buffer trees have typical B-tree amplification. Read amplification: front end trees check memory then disk; node buffer trees check memory buffers then disk. No second publish path (F11).  
   Candidate B: Freeze layer 4 requires message buffers in all internal nodes. Seq comparison across levels fails cross-publish ordering. Journal record unchanged. Write amplification higher due to message pushing; read amplification higher due to checking multiple levels. No second publish path.  
   Candidate C: Same as A. Freeze layer 4 includes message buffers only for allowed trees. Seq unchanged. Journal record unchanged. Write amplification similar to A. Read amplification similar to A. No second publish path.  

4. Check that would fail:  
   Candidate A: Check that accounting tree has no node buffers. If node buffers exist for accounting tree, design fails. Prove by verifying tree-specific buffer configuration.  
   Candidate B: Check that a value from a newer publish (seq 1) is chosen over an older publish (seq 2). If older publish value is selected due to higher seq, design fails. Prove by simulating two publishes and verifying read order.  
   Candidate C: Same as A. Check that accounting tree has no node buffers. If node buffers exist, design fails.  

5. Candidate C is picked. Single observation that would change pick: if accounting tree can recomputed from authoritative state (per F5), allowing node buffers for accounting, but current design requires front end for accounting. If node buffers are more efficient for accounting, candidate C would be invalid.
