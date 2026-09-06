You are auditing a filesystem on-disk format decision. Answer in English only.
Do not use any markdown emphasis (no bold, no italics, no asterisks). Plain text and plain numbered lists only.

CONTEXT

A from-scratch copy-on-write filesystem is choosing where to put a monotonic counter called
the tree-ID counter: the next tree ID to hand out. Tree IDs name btrees; snapshots are trees.

The chosen design: the counter lives in the superblock, replicated one copy per device.
On a writable mount the filesystem computes
  new_value = max(counter in each superblock of the set of devices it exclusively opened,
                  instance id recorded in every root record in the root ring) + 1
and writes that value back into every superblock of that set, all-or-nothing.
A writable mount requires exclusively opening a strict majority of the devices in the pool.
Read-only mounts never take a number.
This exactly copies an existing settled mechanism used for a different counter, the instance id.

The two rejected alternatives:
  R1. Keep the counter in the accounting btree. Rejected because rollback to an older
      checkpoint reloads every accounting statistic from the old accounting tree, so the
      watermark would move backwards and already-issued tree IDs would be handed out again.
  R2. On mount, scan the tree table and take max + 1. Rejected because tree IDs allocated
      inside a checkpoint that was later abandoned do not appear in the published tree table,
      so they would be handed out again.

Other facts you may rely on:
  - The tree table is not stored in the superblock. The superblock holds a 59 byte pointer to
    a tree table unit; the table lives in its own units.
  - The superblock is one sector (512 bytes) per slot and currently occupies 420 bytes.
  - Snapshots are now stored as tree table entries.

QUESTIONS

1. The instance id mechanism was designed for a counter that must increase on every writable
   mount. The tree ID counter increases when a tree is created, which can happen many times
   within one mount. Name every place where that difference makes the copied mechanism behave
   differently from what the tree ID counter needs. For each one, say concretely what goes
   wrong and at which step.

2. Consider this sequence: a pool is mounted writable, creates trees, and crashes without a
   clean unmount. On the next mount, is the tree ID counter in the superblock guaranteed to
   be at least as large as the largest tree ID that any surviving on-disk structure refers to?
   Give the reasoning. If it is not guaranteed, give the shortest sequence of events that
   breaks it.

3. Rejection R2 says abandoned tree IDs are invisible to a tree table scan. Does the chosen
   design actually avoid that problem, or does it only move it? Specifically: if the counter
   is written to superblocks during the mount, but the trees themselves are published later in
   checkpoints, is there any window in which the superblock counter and the published tree
   table disagree in a way that matters after a crash? Say what breaks, or say nothing breaks
   and why.

4. Of the two rejected alternatives, is either rejection reasoning wrong or incomplete?
   Answer for each separately.
