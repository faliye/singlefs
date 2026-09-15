Two-level buffering in a copy-on-write B-tree filesystem design, round 2.

Answer in English. Do not use any markdown emphasis: no bold, no italics, no headings with asterisks. Use plain numbered answers.

This round is about a future version that turns on a write buffer front end and node message buffers (epsilon > 0). The first version turns both off. Facts from the design (treat them as given):

F1. One B-tree engine with several independent keyspaces and two front ends. The write buffer front end batches high-frequency small updates (reverse index, LRU, accounting), sorts and deduplicates them, then inserts them in bulk.
F2. Write buffer requirements: the node cache must resist scans; entries are idempotent full values, never deltas; every entry carries a sequence number seq, because flushing sorts by key and loses time order, so "later wins" must be carried by the entry itself. This is a format requirement.
F3. seq is 4 bytes, stored in the value, not the key, and covers the accounting, reverse index and LRU trees. In the current definition, seq = how many times the same (tag, tree, device, generation) has been written into the tree within one publish, starting at 1. The width argument assumes that entries waiting for deduplication for the same key always belong to one publish window. A rider says: for the same key, cross-batch insertion order must follow seq.
F4. Index nodes keep a message buffer (future values: epsilon 0.65, fanout 119, 665 buffered messages). The first version uses epsilon 0 for every tree; a tree of height 1 has a root that is a leaf with no buffer.
F5. Staleness is detectable. Buffers are purely derived state, recomputed from authoritative state. Tree axis: if a tree's pending messages can be recomputed from authoritative state, keep buffers (epsilon 0.65); otherwise epsilon 0. Today only the inode tree is on the "cannot" side; the extent, directory entry, reverse index and accounting trees are judged "can".
F6. Before any index rebuild, every buffer level must be drained; the rebuild entry point enforces it.
F7. The internal layout of index nodes, including message buffers, belongs to freeze layer 4.
F8. If a published root does not include the accounting or allocation record updates of the transactions in the same batch (still in the in-memory write buffer), a crash back to that root leaves the accounting stale. How far the write buffer is drained at a publish point is not covered anywhere. An fsync publishes only the fsynced file's dirty leaves and their ancestors, and nobody has defined which dirty state belongs to a publish.
F9. Accounting entry: (statistic, dimension tuple, generation) -> full value. Non-monotonic statistics: later wins, decided by seq. Monotonic statistics: take the maximum. Only the most recent K generations are kept; dropping one is a point delete of generation (current - K), so each checkpoint writes one entry and deletes one.
F10. Applying a journal record swaps four fields of the chosen root (tree table pointer, central mapping root pointer, tree ID counter, rollback lower bound F) for the record's values. The journal design never mentions the write buffer.
F11. Fixed point: each index node and container is written at most once per instance per checkpoint, copied on write all at once at the end of the publish; the first version forbids copy-on-write during inserts.
F12. One transaction may span several journal records; the record header carries transaction boundary fields.
F13. Stale-as-valid risk: if the authoritative value of a key sits in an internal node's buffer and that node is lost, the rebuilt tree has valid headers and checksums but stale content. Detecting it needs a comparable time clue in the format.

Round 1 found: A in its weakest reading reads a stale value (an old message in a node buffer shadows a newer value that the front end wrote straight into the leaf). B as written reads stale values across publishes, because the per-publish seq of F3 cannot order entries from two publishes; on the accounting tree this happens through the point delete of F9, whose delete marker and the deleted row share a key but belong to two publish windows. C was not broken. Draining at every publish, with front-end entries tagged by publish window, is required by every candidate. Out-of-order flush of frozen batches reads stale values unless batches for the same key range enter the tree in arrival order.

Candidates:
A-R4. Front-end flush goes straight to leaves, not through node buffers; node buffers take only updates that do not use the front end. Plus three rules: a flush removes older messages for the same key on its path; landing in a leaf compares arrival order within the same window; a write that does not use the front end evicts the front end's entry for the same key. Reads take the first hit in the order front end, root buffer, ..., leaf.
B-D. Front-end flush goes into the root's buffer and is pushed down level by level. A buffer holds at most one message per key; a new one replaces the old (monotonic statistics take the maximum). Newer versus older is decided by position, across levels and within a level; seq is not read.
B-mono. B as written, with seq changed to a pool-wide monotonic counter.
C. A tree uses only one level: front-end trees have epsilon 0 and all their writes go through the front end; other trees follow F5. The front end is drained to leaves before each publish, and landing in a leaf compares arrival order within the same window.

Questions:
1. Correctness under shapes round 1 did not try: node splits and merges that redistribute buffer contents; an fsync publishing only part of the dirty state (F8); one batch whose keys fall into different subtrees; K > 1; a crash in the middle of draining; staleness detection (F5, F13) disagreeing with a candidate's newer-versus-older rule. For each candidate, give a sequence that reads a stale value, or that makes the accounting disagree with the chosen root after a crash, or list the shapes you tried.
2. Cost and format: what each candidate changes in node layout, in the meaning and width of seq, and in the tree axis of F5; write and read amplification.
3. For each candidate, one check that turns red when it is broken, and how you would prove that the check itself can turn red. For C, how would you make "front-end trees have no second writer" a check that can turn red?
4. Which one do you pick, and what single observation would change your pick?
