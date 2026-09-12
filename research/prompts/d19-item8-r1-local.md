You are the counterexample leg of a three-way design review for a from-scratch
copy-on-write filesystem. Your only job is to construct concrete counterexamples
against the candidate arms below. Do not argue which option is better. Do not summarize.

Do not use any markdown emphasis in your answer. No bold, no italics, no asterisks.
Write plain sentences and plain numbered lists. Answer in English.

FACTS YOU CAN RELY ON

F1. Units come in classes. Type 1 is a data unit, type 2 an index node, type 3 a packed
    record unit. Every unit header records its birth tree (for type 2 only a tree id
    field exists), its birth txg, and a write sequence (type 1: instance id 4 bytes plus
    transaction number 6 bytes; type 2: instance id 4 bytes only). Types 2 and 3 also
    carry a 4-byte birth sequence right after the write sequence.
F2. A central map translates a version key into the current physical location. User
    decision: the map is the only entry point for dereference and for free decisions.
    Location entries stored in parent pointers are only hints and can be stale. Hard
    rule: a free is always decided through the map, never through a hint. An earlier
    simulation measured about 5000 wrong frees per 200 rounds when freeing by hint.
F3. Map key for type 1 = class tag 1 + birth tree 8 + birth txg 8 + write sequence 10.
    Map key for types 2 and 3 = class tag 1 + birth tree 8 + birth txg 8 + instance id 4
    + birth sequence 4. The class tag comes from the lookup path. The birth sequence is
    counted per (tree, checkpoint number, instance id); an experiment used start 0,
    step 1 per key, reset at every checkpoint, and a rewritten node gets a new number.
    Every writable mount takes a new instance id.
F4. Every pointer, in every arm below, already carries birth tree 8 and birth txg 8
    (this is assumed from a separate open question and is not under review here).
F5. Some units can never go through the map because the map itself must be found
    first: the map tree's own nodes, the tree table unit, and the instance table unit.
    They need an explicit bootstrap exemption in every arm.
F6. Moving a node in copy-on-write means writing a new copy and rewriting every parent
    up to each root that references it. When K trees share a node, all K parent paths
    are rewritten. Through the map, a move rewrites only one map entry.
F7. An open debt: for type 2 nodes, which header field supplies the birth tree is not
    written anywhere; the header has a tree id, and a rule says a type 2 header's tree id
    equals the tree that references it. Whether type 2 nodes can be shared across
    snapshot heads is not settled.
F8. An open debt: two versions of the same type 2 or type 3 unit written inside one
    checkpoint's commit fixpoint have identical write sequences, so scan rebuild cannot
    tell which one is current.

ARMS

A. All pointers go through the map. Pointers to type 1 carry birth tree, birth txg and
   write sequence. Pointers to types 2 and 3 carry birth tree, birth txg, instance id
   and birth sequence. Map holds types 1, 2 and 3 except the exempt units of F5.
B. Only type 1 goes through the map. Pointers to type 1 as in A. Pointers to types 2
   and 3 carry only birth tree and birth txg, and their location entry is authoritative,
   not a hint. The 4-byte birth sequence in type 2 and 3 headers then has no consumer.
C. Control. The write sequence is not stored in pointers to type 1; at free time it is
   read from the target unit's header.

WHAT TO PRODUCE

1. Key assembly. For each arm and each class, construct a free or dereference where
   some key component cannot be obtained from the referencing side at that moment,
   especially when the location hint is stale. Say what is read and from where.
2. Stale authority in B. Construct any path that changes the physical location of a
   type 2 or type 3 unit without rewriting its parents (background compaction, scrub
   repair of a bad copy, device removal or replacement, crash recovery, replay). If
   one exists, show how arm B then frees or reads the wrong place.
3. Scan rebuild of the map in A. Construct a case where a type 2 or type 3 map entry
   cannot be rebuilt from unit headers alone, using F7 or F8 or anything else. Say
   whether the same case also hurts arm B.
4. Key collision in A. Construct two live type 2 or type 3 units that get the same map
   key under F3, across crashes, replays, instance switches or fixpoint iterations.
If you find nothing for an item, say so plainly and list what you tried.
