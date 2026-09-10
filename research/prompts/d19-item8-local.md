You are the counterexample leg of a three-way design review for a from-scratch
copy-on-write filesystem. Your only job is to construct concrete counterexamples.
Do not argue which option is better. Do not summarize. Find cases that break things.

Do not use any markdown emphasis in your answer. No bold, no italics, no asterisks.
Write plain sentences and plain numbered lists. Answer in English.

BACKGROUND

A settled clause says a central map from logical identity to physical location is
the only entry point for dereference and for the free decision. Freeing always
goes through the map, never through a hint carried in a tree pointer.

The candidate map key is six segments:
  1. unit type tag, 1 byte
  2. tree id, 8 bytes
  3. object id, 8 bytes
  4. object birth generation, 8 bytes
  5. anchor offset, 8 bytes
  6. block birth generation, 8 bytes
Segments 1 to 5 are a settled 33 byte five-tuple. Segment 6 is the version
dimension, needed because deadlists and snapshots reference OLD versions while
the five-tuple names only a logical position.

The open question is: at the moment of dereference or free, where does the
REFERRING side get each of these six segments?

What is known today:
- Segment 1 is inferable from the lookup path.
- Segments 3 and 5 come from the extent tree's key, which is settled as
  (locality_id, inode, offset).
- Segment 2 must be the BIRTH tree, not the referring tree, when a block is
  shared across writable heads after a clone. The lookup path only gives the
  referring tree.
- Segment 6 is not carried anywhere on the referring side today.
- Segment 4, object birth generation, is NOT in the extent tree key, and none of
  the three candidates for the pointer header provide it.

Four draft candidates for how to supply the missing segments:
  A. Carry all six on the referring side; pointers or entries get wider.
  B. Carry only the three missing ones (birth tree, object birth generation,
     block birth generation), infer the rest from the lookup path.
  C. Surrogate key: the referring side carries one opaque number unrelated to
     logical identity, and the map is keyed by that number.
  D. Do not require the referring side to assemble the key; answer from somewhere
     else. Note this collides with the settled clause that freeing always goes
     through the map.

Other relevant settled facts:
- The map must be rebuildable by scanning units on disk, because it is derived
  state, not authoritative state.
- Each data unit's own header carries the five-tuple and the block birth
  generation, but when encryption is on those identity fields are ciphertext, and
  a standing rule forbids taking an authentication expected value from the object
  being verified.
- Snapshot count has no upper bound.
- A crash reissues transaction group numbers.

YOUR TASK

Construct concrete counterexamples, with a specific sequence of operations and
specific numbers.

Q1. Construct a case where segment 4, the object birth generation, is genuinely
    needed at free time and cannot be recovered from anything the referring side
    holds. If you think it is never needed, construct instead a case proving that,
    and say which settled clause becomes redundant.

Q2. Construct a case where candidate C, the surrogate key, cannot rebuild the map
    by scanning units on disk. Be specific about what is on the unit and what the
    scanner would have to invent.

Q3. Construct a case where candidate B is not actually cheaper than candidate A,
    that is, where inferring the remaining segments from the lookup path costs
    more than carrying them.

Q4. Construct a fifth candidate that none of A to D covers, and then break it.

Q5. Construct a case where the claim "this question is the common upstream of the
    two downstream open questions" is false, that is, where settling this one
    leaves both downstream questions exactly as stuck as before.

For each answer, state the operation sequence, the concrete values, and what
exactly goes wrong. If you cannot construct a counterexample for a question, say
so in one sentence and move to the next.
