1. For candidate A-R4: A non-front-end write for key K with value V_old (seq=1 in publish 1) is placed in a node buffer. A front-end write for K with value V_new (seq=2 in publish 2) is flushed directly to the leaf. A read checks the node buffer before the leaf and returns V_old, which is stale.  
For candidate B-D: A direct write places V_new in the leaf buffer (inserted later). A front-end flush writes V_old to the root buffer (inserted earlier). A read checks the root buffer first and returns V_old, which is stale.  
For candidate B-mono: A direct write places a message with seq=5 in the leaf buffer. A front-end flush writes a message with seq=3 to the root buffer. A read checks the root buffer first and returns seq=3, which is stale.  
For candidate C: An accounting tree update is drained to leaves via the front end. An fsync publishes only the inode tree, leaving the accounting tree root unchanged. After a crash, the system uses the old accounting tree root, which points to outdated leaves; the new leaves are unreachable, resulting in stale data.  

2. A-R4: Node layout includes buffers for non-front-end updates; seq remains 4 bytes stored in value but used only within publish windows; tree axis keeps epsilon 0.65 for non-front-end trees; write amplification increases for non-front-end updates due to buffer push-down; read amplification increases due to checking front end, buffers, then leaf.  
B-D: Node layout holds one message per key; seq is unused (width could be reduced but not required); tree axis keeps epsilon 0.65; write amplification high due to level-by-level push-down; read amplification high due to checking multiple buffer levels.  
B-mono: Node layout same as B-D; seq is global monotonic counter (4 bytes); tree axis same; write amplification high; read amplification high.  
C: Node layout has no buffers for front-end trees (epsilon 0); seq used only for front-end deduplication (4 bytes); tree axis sets all trees to epsilon 0; write amplification low (direct leaf writes); read amplification low (direct leaf reads).  

3. A-R4: Check that node buffers never contain values older than the leaf; red if a node buffer entry is older than the leaf entry for the same key. Proof: Inject a non-front-end write with lower seq into a buffer after a front-end flush to the leaf.  
B-D: Check that higher buffer levels never contain older messages than lower levels; red if a root buffer entry is older than a leaf buffer entry. Proof: Insert an older message into root buffer after a newer message in leaf buffer.  
B-mono: Check that the highest seq for a key is always in the lowest accessible buffer; red if a higher buffer has a lower seq than a lower buffer. Proof: Insert a high-seq message into leaf buffer and a low-seq into root buffer.  
C: Check that all writes to front-end trees go through the front end; red if a direct write occurs. Proof: Attempt a direct write to a front-end tree and verify the check fails.  

4. Pick candidate C. A single observation that would change the pick is if the front-end drain before publish introduces unacceptable latency for high-frequency updates, making buffer-based approaches necessary for performance.
