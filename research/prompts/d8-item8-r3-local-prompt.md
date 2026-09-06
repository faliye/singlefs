You are auditing a filesystem mount protocol for a from-scratch copy-on-write filesystem.
Answer in English only. Do not use any markdown emphasis (no bold, no italics, no asterisks).
Plain text and plain numbered lists only.

THE PROTOCOL

Trees are named by an integer identifier. Snapshots are trees. Identifiers must never be reused
for the lifetime of the pool, even across crashes and administrator rollbacks.

The pool has several devices. Each device holds one superblock, and each superblock holds a
field called next_free. There is also a published tree table, reachable from the mounted root,
listing every tree that exists in the mounted checkpoint.

On a writable mount, in this exact order:
  step 1. Exclusively open a strict majority of the devices. Call that set S.
  step 2. Compute base = max( next_free read from each superblock in S,
                              largest identifier in the published tree table, plus one ).
  step 3. Write base + K into the next_free field of every superblock in S. All or nothing:
          if some writes succeed and some fail, roll the successful ones back to their old value.
          If the rollback also fails, mount read only.
  step 4. Only after step 3 completes, begin serving. Identifiers for new trees are handed out
          from the half open range [base, base + K), in increasing order, without touching any
          superblock. When that range is exhausted mid-mount, repeat steps 2 and 3 to reserve
          the next range.

K is a tunable positive integer stored in the superblock. A crash discards whatever remains of
the current range: those identifiers are burned, never handed out again. The project explicitly
accepts burning identifiers; it does not accept reusing them.

A read only mount never writes any superblock and never hands out identifiers.
An administrator rollback selects an older root; it reloads accounting statistics from that
older root but does not touch any superblock field.

QUESTIONS

1. Prove or refute: after any sequence of crashes, rollbacks, and remounts, no identifier is
   ever handed out twice. If you can refute it, give the shortest event sequence that does so,
   naming the step number at which the crash occurs.

2. Step 4 says the range is exhausted mid-mount and steps 2 and 3 repeat. Between the moment
   the range is exhausted and the moment step 3 finishes the second time, what is the state of
   the filesystem, and what must it do with a request to create a tree arriving right then?
   Name every answer that is consistent with the rest of the protocol, and say which ones break it.

3. Step 2 reads the published tree table. Give every circumstance in which that read cannot
   produce an answer, and say what step 2 should do in each. Does any of them make base smaller
   than it would otherwise be? If so, is that safe?

4. Two hosts share the storage. Host A mounts writable, holds a majority, and reserves a range.
   Host A then loses power without releasing anything. Host B mounts writable later and holds a
   different majority, one that overlaps A in exactly one device. Is B guaranteed to compute a
   base at least as large as anything A handed out? Give the reasoning or the counterexample.

5. Name the single strongest objection to this protocol that questions 1 through 4 did not cover.
