You are the counterexample leg of a three-way design review for a from-scratch
copy-on-write filesystem. Your only job is to construct concrete counterexamples.
Do not argue which option is better. Do not summarize.

Do not use any markdown emphasis in your answer. No bold, no italics, no asterisks.
Write plain sentences and plain numbered lists. Answer in English.

FACTS YOU CAN RELY ON

F1. User decision: every pointer to a unit carries the map key of that unit except the
    class tag. For a data unit: birth tree 8, birth txg 8, write sequence 10. For an
    index node or packed record unit: birth tree 8, birth txg 8, instance id 4, birth
    sequence 4. All pointers go through a central map keyed by these values.
F2. User decision: pointers are fixed width everywhere.
F3. Three consumers need birth information from the referencing side: a snapshot
    free rule needs birth txg; the map key needs birth tree and birth txg; an
    encryption associated-data check needs birth tree and birth txg, and those values
    must come from the referencing side, never read back from the unit itself.
F4. An inode tree internal entry is: separator key 8 + identity reference 26 (birth tree
    8, record type 2, container id 8, container birth generation 8) + child pointer.
    Birth tree is carried per entry because clone heads share the origin's leaves, so
    one internal node can hold entries with different birth trees. Record type 0 means
    the child is an index node (container id and generation are 0); 2 means the child
    is a packed container.
F5. An invariant already requires: for record type 2, the child header's birth tree,
    record type, container id and container birth generation equal the entry's
    identity reference byte for byte.
F6. With F1, the inode internal entry holds birth tree twice: in the identity
    reference and in the child pointer.

OPTIONS FOR THE DUPLICATE (F6)

A. Keep both copies and add an invariant: child pointer birth tree equals identity
   reference birth tree equals child header birth tree.
B. Drop birth tree from the identity reference (26 becomes 18); only the pointer has it.
C. Control: in this entry only, the pointer omits birth tree (breaks F2).

WHAT TO PRODUCE

1. Find a layout where no pointer carries birth tree and birth txg, yet every map key
   component in F1 and every consumer in F3 still gets its value from the referencing
   side. If none exists, say why.
2. For A: a crash, torn write, single-copy corruption or clone-sharing sequence where
   the two copies (or a copy and the child header) disagree and no check reports it,
   or where the wrong copy is trusted.
3. For B: a sequence where dropping birth tree from the identity reference breaks the
   F5 check, clone sharing, scan rebuild, or the F3 consumers.
4. For C: confirm or refute that it breaks F2, and anything else it breaks.
If you find nothing for an item, say so plainly and list what you tried.
