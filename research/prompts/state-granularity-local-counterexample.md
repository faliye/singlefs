Role: you are the counterexample leg of a three-way design review. Your job is to
construct a counterexample, not to agree. Answer in English. Do not use any markdown
emphasis in your answer.

Context. This is a from-scratch copy-on-write filesystem. On-disk state is split into two
classes. Authoritative state is units plus accounting plus roots; losing it loses data.
Derived state is the index trees; losing it only costs time, because it can be rebuilt.

The test that decides which class something belongs to is quoted verbatim: "can this
content be recomputed from authoritative state alone".

The same document applies that test at three different granularities today:

G1. Whole structure. Quoted: "that whole allocation-record tree can be rebuilt from
authoritative state, it does not enter the authoritative list". This was decided by the
project owner on 2026-09-03.

G2. Node and leaf. Quoted: "the internal nodes of a tree can be recomputed from its
leaves, therefore derived state; if a leaf holds a value that cannot be recomputed,
therefore authoritative state".

G3. Single field. Two instances. First, on the same day as G1, a consequence was recorded:
relocation forwarding information, meaning "where did the block that used to be at this
location move to", is a historical event; a scan sees only current state and cannot
rebuild it; therefore forwarding may not live only in the allocation record. It must
either become authoritative state or not exist. Second, for a message buffer held in
index-tree internal nodes, the test was applied to the individual message in the buffer,
not to the buffer or the node.

Also established: the claim "losing derived state only costs time" has a precondition,
quoted: "corruption of derived state must be detectable". If a format choice degrades
corruption from "detectable unreachability" into "undetectable staleness", that claim
fails for that part of derived state. The recorded instance: if a key's authoritative
value may rest temporarily in an internal node's buffer, then after losing that node, a
full-disk scan rebuilds a tree whose block headers, checksums and Merkle chain all verify,
while the content is stale.

Also established, as a hard constraint: "an index node's extension point may only hold a
derived cache that is recomputable from data-unit extension points alone, and does not
separately count toward out-of-space admission". Its stated reason: putting a value that
cannot be recomputed into a rebuildable container promotes it, in effect, to authoritative
state.

Candidate conclusion under test: the granularity at which the test is applied is the
field. Every clause that classifies a structure as derived state must carry a
per-field recomputation-source list, mapping each field to the group of authoritative
state it is recomputed from. A field present in the structure's field table but absent
from that list is an error. A statement of the form "this whole structure can be rebuilt"
holds only when every field under it appears in the list.

Your task. Find a counterexample. Concretely:

1. Construct a field for which applying the test at field granularity gives an absurd or
   harmful answer, while applying it at node granularity or whole-structure granularity
   gives the right one. Pure caches are the most promising place to look: a fanout hint, a
   cached subtree size, a bloom filter. Work one through concretely.
2. Under the candidate, re-read G1. Does "that whole allocation-record tree can be rebuilt"
   still hold, or does it become false? If it becomes false, the candidate requires
   overturning a decision made by the project owner. Say so explicitly if that is your
   finding.
3. Is "field" itself well defined here? Consider a field holding a composite value, for
   example an interval, or a five-part identity tuple. Does the test need to recurse below
   the field? Where does the recursion stop?
4. Give one concrete observation, doable today on this design, that would falsify the
   candidate.

If you cannot construct a counterexample, say so explicitly and list which classes of
structure and which kinds of field you searched. Do not invent facts not in this prompt.
