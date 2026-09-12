You are the counterexample leg of a three-way design review for a from-scratch
copy-on-write filesystem. Your only job is to construct concrete counterexamples.
Do not argue which option is better. Do not summarize.

Do not use any markdown emphasis in your answer. No bold, no italics, no asterisks.
Write plain sentences and plain numbered lists. Answer in English.

FACTS YOU CAN RELY ON

F1. On-disk state is split into authoritative (units, accounting, roots) and derived
    (index trees). Rule for a value whose bytes cannot be recomputed after a rebuild:
    if a reader that consumes the lost value then gets a conservative answer, the value
    may stay derived; if it gets a wrong answer, the value must be removed from that
    structure or become authoritative. Example of conservative: a free-generation value
    rebuilt as "newest generation" only delays reclaiming by K generations. Example of
    wrong: lost forwarding information makes a reader silently follow a stale pointer.
F2. Space rule: when any input to the free decision is unknown, the default must be
    "append to the deadlist". A leak is detectable, recoverable and bounded; losing data
    is irreversible.
F3. Internal index nodes (class 2 units) of trees whose leaves are authoritative are
    derived: when lost, the tree is rebuilt from its leaves by scanning units. Class 2
    nodes of the accounting tree are authoritative. Three kinds never enter the central
    map: map tree nodes, the tree table unit, the instance table unit.
F4. A class 2 node header carries: tree ID, level, key range (some trees), birth
    generation (the checkpoint txg in which it was written), write sequence 4 bytes
    (instance id only), birth serial 4 bytes, payload CRC, reserved. Birth serial is
    counted per (tree, checkpoint, instance); in the only model that exists it restarts
    at 0 every checkpoint and a rewritten node gets a new serial. None of these bytes can
    be recovered once the node is lost.
F5. Central map key for class 2 = class tag 1 + birth tree 8 + birth txg 8 (the node's
    birth generation) + instance id 4 + birth serial 4. The value is location entries.
    The map is not authoritative: it is rebuilt by scanning unit self-description. All
    pointers go through the map; every pointer to a class 2 node carries these key fields.
    Which header field is the birth tree of a class 2 node is not written down.
F6. Free decision at the moment a block dies: if pointer.birth is greater than
    head.previous_snapshot_txg, free it now; otherwise append it to the head's deadlist.
    With multiple writable heads this fast path is replaced by reverse-index lookups.
    Relocation must never change a unit's birth: reassigning birth on move produced
    exactly one violation per snapshot-referenced moved unit.
F7. Deadlist entries, and livelist entries (a shared tree used when destroying a
    writable head), identify blocks by their central-map key, class 2 nodes included.
    Freeing always goes through the map, never through a location hint.
F8. Not verified: whether class 2 nodes are shared between snapshot trees or clone
    heads. Clone heads are known to share the origin's leaves.
F9. The current written reason says nobody consumes the original birth generation and
    write sequence of a class 2 node, because class 2 is not in the scan-rebuild version
    selection rule and rebuild recreates nodes from leaves. Since all pointers now go
    through the map, that reason is known to be false. The question is whether the
    classification "derived" still holds.

OPTIONS

A. Stay derived. On rebuild, reissue the identity fields by a fixed conservative rule
   (birth generation set to a provable lower bound, write sequence set to the current
   instance, a fresh birth serial) so that every consumer gets a conservative answer.
B. Stay derived. Every place holding the old key (parent pointers, map entries, tree
   table root pointers, deadlist and livelist entries) is rewritten by the same rebuild.
C. Not derived for the part named by authoritative structures: class 2 nodes named by
   deadlist or livelist entries either become authoritative (losing them loses data,
   rebuild may not reissue identity), or those structures stop naming class 2 nodes by
   map key.

WHAT TO PRODUCE

1. List every consumer of each class 2 identity field that the facts imply, one line each.
2. For A: construct a rebuild sequence where some reader gets a wrong answer, not just a
   leak. Try: a map key collision caused by the reissued fields; a block still referenced
   by a snapshot that gets freed; a deadlist or livelist entry whose old key now names a
   different node or no node.
3. For B: find a holder of the old key that the rebuild cannot rewrite, or a rewrite that
   itself reassigns birth in the way F6 forbids.
4. For C: show a case where C still gives a wrong answer, or a case showing C is not needed.
5. For each counterexample, say whether it depends on F8 (class 2 nodes being shared).
If you find nothing for an item, say so plainly and list what you tried.
