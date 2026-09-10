You are the counterexample leg of a design review for a copy-on-write filesystem.
Answer in English. Do not use any markdown emphasis.

CONTEXT

Layout. Each disk holds a superblock at a fixed location, updated by slot rotation.
The superblock records the pool geometry, the device table location, the journal
geometry, and the parameters of a root ring.

The root ring holds root records. Recovery reads all root slots, verifies each one
by its own checksum, then picks the newest valid one by generation.

A root record is small and carries a pointer to a separate tree table unit. The tree
table lists every tree in the filesystem. That indirection is already settled: the
root record always points to a tree table unit, and tree roots never sit directly in
the root slot. The tree table unit is copied on write on every publish.

THE QUESTION

Should the superblock ALSO carry its own pointer to the tree table unit?

Two places in the project say different things. One document lists the superblock as
holding the device table, the tree table, the journal geometry and various
thresholds. Another settled clause gives the tree table pointer to the root record.
A byte budget study counted the tree table pointer, 59 bytes, against the superblock.

CANDIDATE T TO ATTACK: the superblock should NOT carry its own tree table pointer,
because the set of trees is per checkpoint state whose authority is the root record,
and duplicating it would create two authoritative records of one fact.

Note: the project lead already leans toward candidate T. Your job is to push back
against that lean, not to agree with it.

YOUR TASK

Try to show candidate T is wrong, that is, that the superblock really does need its
own tree table pointer. For each item give a concrete construction or say plainly
you cannot.

1. Construct a mount or recovery path where something must know the tree table
   before it can read a root record. Walk it step by step and name what is needed at
   each step. If every path reaches the root record first, say so.

2. Consider the state right after the filesystem is created, before any transaction
   has been published. Is there a root record yet. If not, how does the very first
   mount find the trees.

3. Consider total loss of the root ring, and consider an offline repair tool that
   must work when the root ring is unreadable. Does either of those need a tree table
   pointer somewhere other than the root record. If the repair tool instead rebuilds
   by scanning every unit on the disk, say what that costs and whether it removes the
   need.

4. Consider encryption. Everything except the superblock is encrypted. Without the
   key, a maintenance pass can read the superblock but not the encrypted structures.
   Does that create a need for a tree table pointer in the plaintext superblock, or
   does it not, given that the tree table unit would itself be encrypted.

5. Is there any reader of the filesystem that legitimately starts from the superblock
   and must not go through the root ring at all.

OUTPUT

Start with a line saying how many of the five you could construct something for.
Then take them in order.
