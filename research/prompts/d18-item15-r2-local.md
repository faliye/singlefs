You are the counterexample leg of a three-way design review for a from-scratch
copy-on-write filesystem. Your only job is to construct concrete counterexamples.
Do not argue which option is better. Do not summarize.

Do not use any markdown emphasis in your answer. No bold, no italics, no asterisks.
Write plain sentences and plain numbered lists. Answer in English.

SETTING

Index trees are copy-on-write B-trees of class 2 nodes. A node X that was lost must be
rebuilt, which means choosing, from all node versions found on disk, exactly the child
versions X had at its own birth checkpoint b_X. X's birth and its checksum are read from
the intact pointer that points to X. Old versions stay on disk while any snapshot
references them; freed versions may still be on disk until reused.

COMMON BASE (all options)

B1. Only consider versions judged published by the instance table.
B2. Work top-down from the tree table root; only lost nodes are rebuilt.
B3. For a lost node X, the reference time is b_X, not the root's txg.
B4. Lineage: a class 2 header's tree ID is read as the tree that wrote it; a clone also
    takes versions from its origin with birth <= fork txg. The fork txg has no place on
    disk today.
B5. Each class 2 node is written at most once per instance per checkpoint; two readable
    versions with the same (lineage, level, identity, birth, instance) and different
    payload means corruption, never a choice.
B6 (option C only). The rebuilt node, serialized canonically, must match the checksum
    stored in the intact parent pointer; otherwise stop read-only.

OPTIONS

A-prime. Key ranges are fences: sibling ranges tile the parent range exactly with no
         gaps or overlaps; when a node is emptied and removed, a neighbor is widened to
         cover its range. Children of X = versions at level l-1 in X's lineage whose
         range is inside X's range, birth <= b_X, and not covered by a later version
         (birth still <= b_X) whose range intersects. The result must tile X's range.
B-prime. Add a node number (per tree, monotonic, copied on copy-on-write, new number for
         the right half on split). On merge or removal, write a retirement record
         (tree, level, number, retire txg). Children of X = node numbers not retired
         before b_X whose latest version with birth <= b_X exists and whose actual key
         span is inside X's range.
C. A-prime plus B6.

WHAT TO PRODUCE

1. For A-prime: a sequence of splits, merges, removals and rewrites where the rule picks
   a child set that is not unique or not X's true children at b_X. Attack the fence
   proof: a version never covered but not a child, or a child that is covered.
2. For B-prime: the same. Try a retirement record lost or not yet written, a number
   reused after a crash, a merge where the absorbed side keeps old versions.
3. For all: a picked version that no root ever referenced (intermediate rewrite in the
   same checkpoint, unpublished checkpoint, freed but not reused).
4. For B5: show what breaks if a node is written twice in one checkpoint.
5. For C: how often B6 fails because a child was relocated after b_X and the parent's
   stored location hint is stale; can that be narrowed?
6. A clone case (B4) where all three options cannot decide.
If you find nothing for an item, say so plainly and list what you tried.
