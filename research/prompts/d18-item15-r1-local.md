You are the counterexample leg of a three-way design review for a from-scratch
copy-on-write filesystem. Your only job is to construct concrete counterexamples.
Do not argue which option is better. Do not summarize.

Do not use any markdown emphasis in your answer. No bold, no italics, no asterisks.
Write plain sentences and plain numbered lists. Answer in English.

FACTS YOU CAN RELY ON

F1. Index trees are copy-on-write B-trees made of class 2 nodes. Rewriting a node
    writes a new node elsewhere and rewrites every ancestor up to the root; the old node
    stays on disk until it is freed.
F2. A class 2 node header has: tree ID, level, key range (only in two of the trees; the
    others have no key range, and one tree's key range is deliberately deferred because
    its keys are file names), birth generation (the checkpoint txg in which it was
    published), instance id 4 bytes, birth serial 4 bytes, payload CRC. There is no node
    number that stays the same across copy-on-write.
F3. Packed containers (class 3) do have an identity that stays the same across
    copy-on-write, and their version order is (birth generation, instance id,
    transaction number).
F4. Within one checkpoint, publishing runs a fixed-point loop: writing tree nodes
    allocates space, which rewrites allocation-tree nodes, which allocates again, until
    it converges. So the same key range of a tree can be written more than once in one
    checkpoint, with the same birth generation and the same instance id.
F5. Old versions referenced by a snapshot stay on disk until the snapshot is destroyed.
    Blocks referenced by the newest K valid roots are never reused. Freed blocks outside
    that may be reused.
F6. Each snapshot has an entry in the tree table with its root pointer and its txg.
F7. A node written by instance i counts as published only if the instance table says so;
    for class 2 the check is birth <= T_pub of that instance's row.
F8. A block referenced by root R must have birth <= R.txg.

THE QUESTION

Given every class 2 node header found by a full scan, plus the tree table, can a rule
pick, for any root R with txg T and any key range, exactly the node version that R
referenced at T?

OPTIONS

A. No new field. Among class 2 nodes of the same tree and level whose key range covers
   the range, pick the one with the largest birth <= T. Requires every derived tree to
   carry a key range.
B. Add a node number that is unique within the tree, copied on copy-on-write, and newly
   issued on split. Version order is (birth generation, instance id, fixed-point round).
   For root R pick, for that node number, the largest birth <= T.

WHAT TO PRODUCE

1. For A: a set of on-disk versions and a root where the rule picks two candidates or
   the wrong one. Try node splits and merges that change key ranges, and F4.
2. For B: the same. Try splits, merges, F4, and a node number that is reused.
3. For both: a case where the rule picks a version that no root references any more
   (freed but not yet reused, an intermediate fixed-point version, a node from an
   unpublished checkpoint).
4. For both: a conflict with F7 or F8.
If you find nothing for an item, say so plainly and list what you tried.
