# Find a counterexample: deriving the birth tree from a clone ancestry chain

Answer in English. Do not use any markdown emphasis of any kind in your answer.

This is a from-scratch copy-on-write filesystem in the format design phase. No code exists yet.
Every fact below was read out of the repository today by the main agent.

## Settled facts you must take as given

1. Snapshots and clones. The design settled on multiple writable heads, and the mechanism is
   one btree per head. A new head is created by cloning an existing head at some publish counter
   value. The publish counter, called txg here, is a single global monotone counter for the whole
   pool: every publish event increments it once, and there is exactly one such counter.

2. Every data unit carries a birth txg, which is the txg of the publish event that created it.
   Compaction, which moves a unit from one physical location to another, does not change the
   birth txg, because moving is not publishing a new version.

3. A cloned head shares the units that existed at its clone point. Units born in the parent
   before the clone point remain reachable from the child. Units the child writes after the
   clone point are born in the child.

4. Every unit has an on disk header holding a five field logical identity: unit class tag,
   tree id, object id, object birth generation, anchor offset. For a unit that is shared across
   heads, the tree id in that header is the birth tree, which differs from the tree that
   references it.

5. There is a central map that is the single entry point for dereference and for free decisions.
   Its key must identify a version, and the leading segment of that key is the same five field
   identity, so the key contains the birth tree id.

6. A tree pointer today carries no tree id and no birth txg. Its header is 31 bytes and
   decomposes exactly into MAC 16, nonce 12, algorithm type 1, extent offset 2.

7. Tree ids come from a monotone watermark and are never reused.

## The candidate under attack

Candidate B says: put only the birth txg into the tree pointer, 8 bytes, and do not store the
birth tree id anywhere in the pointer. Instead derive the birth tree at runtime from two inputs
that the reader already has:

- the clone ancestry chain of the head the reader is currently in, which is a list of ancestor
  tree ids together with the txg at which each clone was taken, and
- the birth txg read out of the pointer.

The derivation claims: along the chain, the clone point txgs are strictly increasing, so the
birth txg falls into exactly one interval, and the tree that owns that interval is the birth tree.
Example. Origin is cloned into A at txg 100, and A is cloned into B at txg 250. A reader in B
that sees a pointer with birth txg 40 concludes the unit was born in Origin, birth txg 180
concludes A, birth txg 900 concludes B.

## Your task

Your job is to break Candidate B. Construct a concrete scenario in which the derivation returns
the wrong tree id, or returns no answer, or is ambiguous. For each construction give:

- the sequence of operations, with txg numbers,
- what the derivation computes,
- what the true birth tree is,
- and what goes wrong as a consequence, for example a wrong central map key, a wrong deadlist
  entry, a leaked unit, or a unit freed while still referenced.

Directions worth probing, not a closed list. You are expected to find things not on this list.

- Two heads cloned from the same parent at the same or at different txgs, and whether a reader in
  one of them can ever see a unit born in the other.
- A head that is cloned, then the parent is destroyed, and what happens to the chain.
- A chain that is rebuilt after a crash, where the last publish was rolled back and the txg is
  reissued, so two different units can end up claiming the same birth txg.
- A clone taken from a read only snapshot rather than from a live head.
- Any feature that creates a second live reference to a unit that is not a clone relationship,
  for example cross tree reflink or deduplication, and whether the derivation survives it.
- The very first head, which has no ancestor, and the boundary txg exactly equal to a clone point.
- A unit that is moved by compaction long after birth, and whether anything about the derivation
  changes.

If after honest effort you find that a direction does not break the derivation, say so plainly and
say what would have to be true for it to break. Do not invent a break that you cannot exhibit with
concrete txg numbers.

Finally, answer one separate question. Suppose the derivation is correct in all cases. It still
costs a walk over the ancestry chain on every free decision. The repository has a settled proof
that the free decision is O(1) and reads only two inputs, a pointer field and one scalar in
memory. Is a walk over the ancestry chain compatible with that proof, and if not, what exactly
breaks. Answer this even if you found no counterexample.
