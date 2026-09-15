How two buffering levels should connect in a copy-on-write B-tree design (a future version, not the first one).

Answer in English. Do not use any markdown emphasis: no bold, no italics, no headings with asterisks. Use plain numbered answers.

Facts from the design (treat them as given):

F1. There is one B-tree implementation, many independent keyspaces (one tree per kind of data), and two front ends. The write buffer front end batches frequent small updates (reverse index, LRU, accounting), sorts and deduplicates them, then inserts them in bulk. It is not implemented in the first version: in the first version accounting goes straight into the accounting tree's leaves.
F2. Three requirements for the write buffer: the node cache must be scan-resistant; entries must be idempotent full values, never deltas; entries must carry a sequence number (seq), because a flush sorts by key and loses time order, so which entry wins must be decided by the entry itself.
F3. seq is 4 bytes, lives in the value (not in the key), and covers the accounting, reverse-index and LRU trees. In the first version seq = how many times the same (tag, tree, device, generation) key was written into the tree within one publish, starting at 1.
F4. Internal index nodes may keep a message buffer. The values 0.65 (buffer fraction), fanout 119 and 665 buffered messages are future values for when the write buffer front end is enabled; in the first version every tree has no node buffers, and a tree of height 1 has a leaf as its root.
F5. Staleness in node buffers is detectable; node buffers are purely derived state that can be recomputed from authoritative state. Per-tree rule: if a tree's pending messages can be recomputed from authoritative state, the tree may keep node buffers; if not, it keeps none (today only the inode tree is in the "cannot" group).
F6. Before any index rebuild all buffers must be drained; the rebuild entry point enforces it.
F7. The internal layout of index nodes, including the message buffer, belongs to freeze layer 4, which is not frozen yet.
F8. If a published root does not include the same batch's accounting or allocation-record updates (because they are still in the in-memory write buffer), a crash back to that root leaves the accounts stale. Nothing in the design says how far the write buffer is drained at a publish point.
F9. Accounting entries are (statistic, dimension tuple, generation) mapped to a full value; non-monotonic statistics merge by last-wins, monotonic ones by maximum; only the most recent K generations are kept.
F10. Applying a journal record during recovery replaces exactly four root fields (tree table pointer, central mapping root pointer, tree ID counter, rollback floor). The journal design never mentions the write buffer.
F11. There must be only one transaction state machine and one publish path shared by all structures; a second publish path for some structure is not allowed.

The open question: after the write buffer front end flushes, where do its entries land (straight into leaves, or into node buffers), in what order do the two levels push down and merge, how far is the front end drained at a publish point, and where do undrained front-end entries come from after a crash?

Candidates:
A. The front end flushes straight into leaves, never through node buffers. Node buffers only receive updates that do not come through the front end. Both levels may exist on one tree, but front-end entries never enter node buffers.
B. The front end flushes into the root node's buffer as messages, which are pushed down level by level ("messages closer to the root are newer"). A read looks in the front end, then the root buffer, then lower buffers, then the leaf, and takes the first match; seq is compared across both levels.
C. Each tree uses only one level: either the front end (then it keeps no node buffers) or node buffers (then it has no front end), fixed per tree kind. Accounting, reverse index and LRU use the front end; other trees follow the per-tree rule in F5.

Questions:
1. Read correctness: when the same key has one entry in the front end, one in some node buffer and one in the leaf, does each candidate return the newest one? Can the seq definition in F3 (the nth write within one publish) still order entries across the two levels and across publishes?
2. Crash: for each candidate, pick crash points before and after a publish. After recovery, do the accounts match the chosen root without a full scan? Where do undrained front-end entries come from, given F10?
3. Cost and format: what does each candidate change in freeze layer 4, in seq, or in the journal record? Give write and read amplification in terms of tree height and buffer sizes. Does any candidate need a second publish path (F11)?
4. For each candidate, one check that would fail if the design were wrong, and how to prove the check can fail.
5. Which candidate would you pick, and what single observation would change your pick?
