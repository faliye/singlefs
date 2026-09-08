You are the counterexample leg of a design review for a from-scratch copy on write
filesystem. Find concrete workloads that make a proposal lose. Do not use any markdown
emphasis. No bold, no italics, no asterisks. Answer in English. Number your findings.

SETTING. Every unit on disk is exactly 32768 bytes including its header. A small object
of 512 bytes therefore occupies a whole 32768 byte unit today, wasting 64 times its size.

THE PROPOSAL. A background compactor, running after objects have already been written
normally, moves small objects of at most 4096 bytes into shared 32768 byte containers.
Each slot inside a container carries the object plus 47 bytes of self description.
Container capacity is 58 slots for 512 byte objects, 30 for 1 KiB, 7 for 4 KiB,
3 for 8 KiB, 1 for 16 KiB. A container is immutable once written. Modifying a packed
object writes a new independent unit through the unchanged write path and marks the old
slot dead. The central mapping entry for each object must be rewritten when the object
is relocated, which costs a leaf copy on write of a 16384 byte metadata node.

THE NUMBERS SO FAR, for 100000 objects of 512 bytes each:
- Standing occupancy before: 100000 units, 3.28 GB. After: 1725 containers, 56.5 MB.
  Permanent saving about 3.22 GB.
- One time cost of relocating them: container writes 113 MB, plus central mapping
  rewrites which are 11.7 MiB if the compactor walks in mapping key order, or up to
  3.28 GB if it walks in container fill order flushing one object at a time.
- Reading the objects back to relocate them costs 3.28 GB of reads, never counted.

YOUR TASK. Give concrete workloads and operation sequences, with numbers, not opinions.

Q1. Construct a workload where the compaction never pays for itself, because the objects
do not live long enough. State the object lifetime distribution, the compaction trigger,
and compute the total bytes written with and without the proposal.

Q2. Construct a workload where compaction repeatedly moves the same objects back and
forth: object gets packed, then modified so it is evicted to its own unit, then packed
again. State exactly how many times this happens and the total cost. State what trigger
policy would prevent it and what that policy costs.

Q3. Construct a workload where the containers end up mostly dead slots, so the standing
occupancy after compaction is worse than before compaction. Give the exact fraction of
dead slots at which the proposal loses.

Q4. A single unit becoming unreadable loses one object today, but loses up to 58 objects
once they are packed. Construct the concrete data loss scenario and say what would have
to exist to make that acceptable.

Q5. Name every construction you tried that did not work, and say why it failed.
