You are one leg of a three-way design review for a from-scratch copy-on-write filesystem.
Your assigned stance is adversarial construction: build concrete counterexamples with
specific numbers. Do not use any markdown emphasis in your answer. Answer in English.

Background facts. Treat them as given; do not assume anything beyond them.

1. The filesystem supports multiple writable heads. Each head is its own btree. A head is
   created by cloning from an origin snapshot at some transaction group number.
2. Every block carries a birth transaction group number. Transaction groups are totally ordered.
3. Destroying a writable head must release the blocks that were born inside that head, that is
   the blocks whose birth is greater than the origin transaction group. Those blocks are
   scattered across the whole tree, so a dedicated structure is required to find them.
4. The project has one settled clause saying: all indexes use one btree implementation with
   several independent keyspaces, one tree per kind of data, and no heterogeneous structures.
5. The project has another settled clause, applying to trees that go through a write buffer
   front end, saying: entry form must be an idempotent complete value, never an incremental
   delta. The reason recorded is that under the delta form an audit invariant caught zero of
   four injected accounting bugs, because the audit can only replay the same delta log with
   the same accumulate function, so it is comparing against itself.
6. A third settled clause, also for write buffer trees, says every entry must carry a sequence
   number, and that this is a format requirement.
7. The reference design being copied from is the ZFS livelist: it records ALLOC and FREE pairs,
   grouped by birth transaction group, incrementally maintained, and periodically condensed by
   cancelling matching ALLOC and FREE records.
8. A counting model measured: with the never-condensed event log, at 65536 live blocks and 16
   rounds of overwrite, the structure holds 2162688 entries while the net live count is 65536,
   and the pre-reserved pending-delete byte estimate overshoots the true value by 17 times.
   With condensing, entry count equals the net live count exactly and the overshoot is 1.0.
9. An allocation record tree already exists, keyed by device identity 4 bytes plus a 16 KiB
   slot number 6 bytes, with an 8 byte allocation-or-free transaction group as the value, plus
   a 2 byte span field. It has no per-head dimension.

The proposal under attack, call it PROPOSAL S:

  Make the structure a set, not an event log. One independent keyspace btree. The key is
  (head tree id 8 bytes, device identity 4 bytes, 16 KiB slot number 6 bytes). Presence of the
  key means: this slot holds a block born inside that head and still live. Freeing a block
  deletes the key. There are no ALLOC or FREE records. The tree goes through the write buffer,
  so each entry also carries a sequence number.

Your task. Answer these with concrete constructions containing specific numbers.

Q1. Construct a sequence of operations, with specific transaction group numbers and specific
    slot numbers, where the ALLOC and FREE event log can answer a question that the set
    representation cannot. Name who needs that answer. If you cannot construct one, say so and
    say exactly what blocks the construction.
Q2. Consider a slot that is freed inside a head and later reallocated to the same head. Walk
    through both representations step by step with numbers. Does either one become wrong or
    ambiguous?
Q3. Deleting a key is a tree write. Appending a record is also a tree write. For a workload of
    one initial write of 65536 blocks followed by 16 rounds of overwriting all of them,
    count the tree writes each representation performs, including the condense passes the event
    log needs to stay at the same entry count as the set. Show the arithmetic.
Q4. The system crashes in the middle of destroying a head, after some keys were deleted but
    before the operation finished. Compare recovery for both representations. State precisely
    what each one must re-derive and whether re-doing the work is safe to repeat.
Q5. Putting the head tree id first in the key makes destroying a head a contiguous range
    operation. Name the costs of that choice. Consider point lookups, insert locality, and the
    fact that other data may want to control the same sort order.

Rules. Every claim needs a construction with numbers. Do not argue from what other
filesystems do. If a question cannot be answered from the facts given, say which fact is missing.
