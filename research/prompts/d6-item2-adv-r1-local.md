You are one leg of a three-way adversarial review for a from-scratch copy-on-write filesystem.
Your assigned stance is adversarial construction against a proposal the owner is leaning toward.
Build concrete counterexamples with specific numbers. Do not use any markdown emphasis in your
answer. Answer in English.

This is round one of three. The attack surface for this round is format and on-disk structure
only. Do not attack runtime behaviour, crash recovery during destruction, or clone-of-clone
semantics: those belong to later rounds.

Settled facts. Treat them as given.

1. The filesystem supports multiple writable heads. Each writable head already has its own data
   btree. This is settled.
2. Every tree is registered in a tree table. One tree table entry is 121 bytes: length 2,
   tree kind 2, flags 2, tree id 8, root pointer 59, prev_snap_txg 8, birth txg 8, reserve 32.
   One tree table unit holds 134 entries per level.
3. Of that 32 byte reserve, 24 bytes are already claimed by two known future needs: the location
   of the head's deadlist, 8 bytes, and a nested interval label for the snapshot lineage, 16
   bytes. So 8 bytes remain spare. The deadlist location and the interval label are themselves
   still undecided items.
4. Tree ids are allocated monotonically and a tree id that has been published is never reused.
   The tree id watermark lives in the root record and is the max over the whole root ring.
5. Every data unit on disk carries a self-describing header whose identity tuple includes the
   tree id of the unit's birth owner.
6. The authoritative state is: units, accounting, and roots. Indexes are derived state and must
   be rebuildable by scanning units.
7. The first transaction of a fresh filesystem has no snapshots and no clones.
8. Each writable head also keeps one deadlist. Where the deadlist lives on disk is undecided.

The proposal under attack, call it PER HEAD:

  Each writable head gets a second tree of its own, its livelist. The livelist is an event log:
  one entry per ALLOC and per FREE of a block born inside that head, entry = type tag 2 +
  location 14 + birth txg 8 = 24 bytes, periodically condensed. The livelist tree is created
  lazily when the head is first cloned into existence, not in the first transaction. The head
  finds its livelist through an 8 byte tree id stored in the head's spare reserve, which uses up
  the last 8 spare bytes. A per-livelist entry counter lives in the livelist tree's own tree
  table entry reserve.

Answer with constructions containing specific numbers.

Q1. The head's last 8 spare reserve bytes go to the livelist pointer. Construct a concrete future
    need, following the pattern of fact 3, that now has nowhere to go. If every plausible need
    already has a home, say so.
Q2. The livelist tree is created lazily at clone time. Construct a crash sequence, with specific
    transaction numbers, where the head tree exists and is published but its livelist tree does
    not, or the reverse. State what a reader sees and whether facts 4 and 6 still hold.
Q3. Fact 6 says indexes must be rebuildable by scanning units. Is the livelist derived state?
    Using fact 5, describe precisely what a scan can rebuild and what it cannot. In particular:
    an event log remembers FREE events for blocks that no longer exist on disk. Can a scan
    rebuild those? If not, what does that make the livelist?
Q4. With two tree table entries per head, the second tree table level starts at 68 heads and the
    third at 8979 heads. Construct a workload where this matters and state the concrete cost.
Q5. The deadlist in fact 8 is also per head and has no location yet. Should it follow the same
    carrier as the livelist? Construct the consequence of choosing differently for the two.

Rules. Every claim needs a construction with numbers. Do not argue from what other filesystems do.
If a question cannot be answered from the facts given, name the missing fact.
