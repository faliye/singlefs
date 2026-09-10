You are one leg of a three-way adversarial review for a from-scratch copy-on-write filesystem.
Your assigned stance is adversarial construction against a proposal the owner is leaning toward.
Build concrete counterexamples with specific transaction numbers. Do not use any markdown
emphasis in your answer. Answer in English.

This is round two of three. The attack surface for this round is runtime behaviour and crash
recovery. Do not attack on-disk format, tree table layout, tree id allocation, or whether the
structure is derived state: round one already covered those.

Settled facts. Treat them as given.

1. The filesystem supports multiple writable heads. Each head has its own data tree. A head is
   created by cloning from an origin snapshot.
2. Every block has a birth transaction group number. Transaction groups are totally ordered.
3. Each head keeps a scalar prev_snap_txg, meaning the txg of the most recent snapshot taken of
   that head. At clone time it is initialized to the origin snapshot's txg. When a snapshot of
   the head is taken, prev_snap_txg advances to that snapshot's txg.
4. Deletion rule inside a head: read the block's birth. If birth is greater than prev_snap_txg,
   release the block. Otherwise append it to the head's deadlist. The written rule reads birth
   from an inline field of the parent pointer, and the proof that this is O(1) rests on reading
   only that field and prev_snap_txg.
5. When a snapshot of a head is taken, the head's whole deadlist is handed over to the new
   snapshot, and the head starts a new empty deadlist.
6. When a snapshot is destroyed, its deadlist is merged into the next newer side, and the
   destroyed snapshot's prev is inherited by that side.
7. A snapshot that is a branch point, meaning it has more than one child, may not be destroyed.
8. Long operations run as an intent: the intent is committed first, work proceeds in batches,
   after a crash recovery re-runs the intent and already finished parts are redone idempotently,
   the intent is deleted in the same commit as the last batch, and every intermediate state must
   be a legal filesystem state.
9. Whether the parent pointer actually carries the block's birth txg is an undecided item. One of
   its candidates is to carry nothing.

The proposal under attack, call it PER HEAD LIVELIST:

  To destroy a writable head, its blocks born inside that head must be released. Each head
  gets its own livelist tree: an event log with one entry per ALLOC and per FREE of a block born
  inside that head, entry = type tag + location + birth txg, periodically condensed. The
  livelist tree is created when the head is cloned. A counter of livelist entries lives in the
  livelist tree's own tree table entry. The written release boundary for destroying a head says
  birth greater than the origin txg. A round one finding says that boundary is wrong once the
  head has been snapshotted, and the boundary should be the current prev_snap_txg instead.

Answer with constructions containing specific transaction numbers.

Q1. Clone of a clone. Head H1 is cloned at txg 100. At txg 150 H1 writes block B. At txg 200 a
    snapshot S of H1 is taken. At txg 250 head H2 is cloned from S, so H2 inherits B. At txg 300
    H1 is destroyed. Does destroying H1 release B while H2 still references it? Walk the
    livelist, the deadlist, and the release boundary step by step. If something stops it, name it.
Q2. Snapshot handoff. When a snapshot of a head is taken, what should happen to that head's
    livelist: hand it over to the snapshot, keep it, or clear it? With the boundary changed to the
    current prev_snap_txg, entries with birth at or below the new prev_snap_txg can never be
    released by destroying the head. Construct a sequence showing whether keeping them causes a
    wrong release, a leak, or only wasted space.
Q3. Crash during destruction. The head is being destroyed as an intent, consuming its livelist
    tree batch by batch. Construct a crash point after which the resumed intent either releases a
    block twice or never releases it. Use fact 8 precisely.
Q4. Counter consistency. The entry counter lives in the livelist tree's tree table entry.
    Construct a published state where the counter differs from the true entry count, and state
    which check, if any, could detect it.
Q5. Birth source. Suppose the parent pointer carries no birth txg, per the undecided candidate in
    fact 9. At the moment a FREE is recorded, and at the moment destruction compares birth with the
    boundary, where does the birth txg come from? Construct the cost or the failure.

Rules. Every claim needs a construction with numbers. Do not argue from what other filesystems do.
If a question cannot be answered from the facts given, name the missing fact.
