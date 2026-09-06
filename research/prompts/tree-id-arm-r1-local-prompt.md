You are auditing where to store a monotonic counter in a copy-on-write filesystem.
Answer in English only. Do not use any markdown emphasis (no bold, no italics, no asterisks).
Plain text and plain numbered lists only.

SETTING

Trees are named by an integer. Snapshots are trees. An integer must never name two different
trees over the lifetime of the pool, not across crashes and not across administrator rollbacks.
Burning integers without using them is explicitly acceptable. Reusing one is not.

Publishing works like this. A checkpoint gathers changes, writes data blocks, then writes one
root record into a ring of three root slots with a forced unit access. A root record that got
written is published. Anything from a checkpoint whose root never got written is an orphan:
the blocks sit on the medium, nothing points at them, and a later scan can still read their
headers, which name the tree they belonged to.

An administrator rollback picks an older root out of the ring, then starts a new checkpoint
whose sequence number is one more than the largest sequence number found across every root
record in the ring, including roots from abandoned timelines that can no longer be picked.
Rollback can never reach past the ring, so any root it skips over is still in the ring.

THREE PLACES TO PUT THE COUNTER

Arm A: in the superblock, one copy per device. Written once at mount time, before the mount
       touches anything else, reserving a block of K integers. Integers are handed out from
       that block without writing anything. When the block runs out mid mount, another block
       is reserved by writing the superblocks again.

Arm B: in the accounting tree, as one more statistic. Rollback reloads every accounting
       statistic from the older root, so the counter goes backwards on rollback.

Arm C: in the root record, one field. Read back as the largest value across every root record
       in the ring, exactly like the sequence number above. The root record is written on every
       publish anyway, so the field costs nothing extra.

QUESTIONS

1. For arm C, does an administrator rollback ever cause an integer to name two different trees?
   Give the reasoning, or the shortest sequence that breaks it.

2. For arm C, consider a checkpoint that allocated integer 42, wrote blocks whose headers name
   tree 42, and then crashed before its root was written. The counter value for that checkpoint
   is gone. The next mount hands out 42 again. Name every way this can be observed later, and
   for each say whether it is a real defect or a harmless coincidence. Be concrete about who
   is reading and what they conclude.

3. For arm A, the same crash cannot lose the counter, because the superblock was written at
   mount time before anything else. Is that advantage complete, or are there sequences where
   arm A also hands out an integer twice? Consider in particular the case where a mount uses up
   its block of K and reserves a second block, and something fails during that second write.

4. Rank the three arms. State the single property that decides the ranking, and say what would
   have to change for the ranking to flip.

5. Name the strongest objection to arm C that questions 1 through 4 did not cover.
