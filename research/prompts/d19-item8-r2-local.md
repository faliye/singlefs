You are the counterexample leg of a three-way design review for a from-scratch
copy-on-write filesystem. Your only job is to construct concrete counterexamples.
Do not argue which option is better. Do not summarize.

Do not use any markdown emphasis in your answer. No bold, no italics, no asterisks.
Write plain sentences and plain numbered lists. Answer in English.

FACTS YOU CAN RELY ON

F1. Every block records its birth txg. Block b is referenced by snapshot S if and only
    if birth(b) <= S.txg < death(b). birth(b) is the checkpoint number in which the
    block was published. Moving a block to another physical location is not a new
    publication and does not change birth, because its content is byte-identical.
F2. When a block's last live reference is removed, it is either freed at once or put
    on a deadlist if an older snapshot still references it. Destroying a snapshot
    processes deadlists only; there is no full-disk scan. A clone head keeps a livelist
    that records FREE events when it kills a block.
F3. Existing direction for deadlist entries: record a block by its version identity
    and free it through a central map, never by a stored location, because moving a
    block would otherwise force rewriting every deadlist entry that names it. The same
    direction was proposed for livelist entries (still undecided).
F4. The central map translates a version key into the current physical location.
    Arm A: types 1, 2 and 3 are all in the map (type 2 is an index node, type 3 a
    packed record unit, type 1 data). Arm B: only type 1 is in the map; for types 2
    and 3 the location stored in the parent pointer is authoritative.
F5. Each snapshot and each clone is its own tree. A clone shares unmodified nodes and
    leaves of its origin. Snapshots are read-only.
F6. In arm B, moving a type 2 or type 3 unit means writing it at the new location and
    rewriting every parent node up to the root in every tree that references it,
    including read-only snapshot trees. In arm A, moving only rewrites its map entry.
F7. Background compaction moves units as ordinary transactions. Rebuilding the copy
    lost with a failed disk onto another disk has no settled form yet.
F8. Index nodes (type 2) are not restored from their own headers during scan rebuild;
    trees are rebuilt from their leaves.

WHAT TO PRODUCE

1. Stale holder. For arm B, give a concrete sequence (create, snapshot, delete, move,
   destroy snapshot) in which a type 2 or type 3 unit sits on a deadlist or livelist,
   is then moved, and the deadlist or livelist entry later causes a wrong free or a
   leak. Then try the same sequence under arm A and say whether it also fails.
2. Snapshot tree move. For arm B, move one type 2 node that a read-only snapshot S
   references. List every block written, with its birth. Check each against F1 and
   F2 at the moment S is later destroyed. Either show a violation (leak, wrong free,
   a block S references but F1 says it does not) or show a form that satisfies F1.
3. Rebuild in A. Construct a scan-rebuild path in arm A that must read a key
   component from an old type 2 header. If F8 makes this impossible, say so.
4. Fix for B. Suppose arm B adds the rule: a type 2 or type 3 unit referenced by any
   snapshot is never moved. Construct a case where this rule blocks disk-loss rebuild
   or compaction, or leaves a unit that can never be freed.
If you find nothing for an item, say so plainly and list what you tried.
