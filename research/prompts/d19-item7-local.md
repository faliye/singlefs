You are the counterexample leg of a three-way design review for a from-scratch
copy-on-write filesystem. Your only job is to construct concrete counterexamples.
Do not argue which option is better. Do not summarize. Find cases that break things.

Do not use any markdown emphasis in your answer. No bold, no italics, no asterisks.
Write plain sentences and plain numbered lists. Answer in English.

BACKGROUND

Every block pointer in this filesystem has a fixed-width header of 31 bytes today:
MAC 16, nonce 12, algorithm type 1, extent offset 2. All four fields are already
committed by earlier decisions. A pointer also carries N position entries of 14
bytes each; a child pointer inside an index node is 31 + 14 * 2 = 59 bytes.

The open question is whether the pointer header must also carry the birth identity
of the block it points to: the birth tree id (8 bytes) and the birth transaction
group number (8 bytes).

Three candidates:

A. Store both. Header goes 31 to 47. Child pointer 59 to 75. The inode tree
   internal node entry goes 93 to 109. The ledger tree internal entry 81 to 97.
   The root record 194 to 226.
B. Store only the birth txg. Header 31 to 39. Recover the birth tree by walking a
   clone ancestry table together with the birth txg.
C. Store neither. This is today.

Three consumers are said to require this field:

1. The snapshot lifetime decision is documented as O(1) with this pseudocode:
   decide(ptr): birth = ptr.birth (inline pointer field, zero extra I/O);
   p = head.prev_snap_txg (a scalar in the live head, resident in memory);
   if birth > p then release immediately else append to head.deadlist.
2. A settled clause says freeing always goes through a central map, never through
   a hint. The candidate encoding for that map key is the logical identity
   five-tuple plus the birth generation (33 + 8 = 41 bytes), and the tree id
   segment of that five-tuple is the birth tree when a block is shared across
   writable heads.
3. Encryption. Measured result: an AEAD associated-data field that omits the
   snapshot dimension detects 0 of 2 deliberate mismatches, and both mismatches
   silently returned data belonging to another writable head, with the MAC
   passing. A follow-up measurement found that taking the tree dimension from the
   lookup path instead correctly rejects all 3 rehoming attacks but also rejects
   6 of 8 legitimate reads of blocks inherited across a clone point. The only
   form that worked was: the pointer carries a claim of birth (tree, txg), the
   reader first validates that claim against a clone ancestry table (the claimed
   tree must be on the ancestor chain, and the birth txg must not exceed the
   corresponding clone point), and only then feeds the claim into the MAC.
   Taking the tree dimension by reading it back out of the pointer without that
   validation left all 3 rehoming attacks silent.

Two further facts that constrain where these values may come from:

4. The data unit header already contains the five-tuple (33 bytes) and the birth
   generation (8 bytes). But when encryption is on, those identity fields of a
   data unit are stored as ciphertext in place. Also, a standing rule forbids
   taking an associated-data expected value from the object being verified.
5. On one class of edge, the internal node entry of the inode tree already carries
   a 26 byte identity reference laid out as: birth tree 8, packed record type 2,
   container number 8, container birth generation 8. When the type segment is 0,
   meaning the child is a plain index node, the container number and container
   birth generation segments are documented as constant zero. So 16 bytes are
   already spent and forced to zero on that edge.

A NEW MEASUREMENT, RUN TODAY

A pure CPU and memory model of an implicit B-tree, 16384 byte nodes, entries
strided by entry width, binary search inside a node, dependent pointer chasing
between levels. Working set 8388608 keys, which exceeds the 32 MiB L3 cache.
Five rounds. A positive control passed for every arm with ratios from 7.89 to
10.67. A cross-harness gate reproduced an older experiment's numbers at entry
widths 67 and 111 to within 2.4 and 2.6 percent.

Result: at entry width 93 the median was 587.44 ns per lookup; at entry width 109
it was 569.91 ns. So the wider entry was about 3 percent faster, in all 5 rounds.
At entry width 81 the median was 554.37 ns and at 97 it was 563.38 ns, so there
the wider entry was about 1.8 percent slower, in all 5 rounds. Tree depth was 4
for all four of those widths, so no depth change is involved. There is one
unexplained non-monotonicity: 93 bytes was slower than both 109 and 111 bytes.

YOUR TASK

Construct concrete counterexamples, with a specific sequence of operations and
specific numbers, for as many of the following as you can. For each one, either
give the counterexample or say plainly that you could not construct one.

Q1. Construct a case where candidate A is still not enough, that is, where the
    pointer carries both fields and a reader is still fooled or still cannot
    answer the question it needs to answer.

Q2. Construct a case where candidate B gives a wrong answer. Specifically look
    for a case where the mapping from birth txg to birth tree is not injective.
    One documented fact you may use: after a crash the transaction group number
    is reissued.

Q3. There is a fourth path nobody has listed: put the two fields in the referring
    entry rather than in the generic pointer header. On one edge this costs zero
    additional bytes because 16 bytes are already present and forced to zero, see
    fact 5. Construct a case where this fourth path breaks down, in particular on
    the edge where an extent tree leaf refers to a data unit. Be specific about
    what the referring entry looks like there and why the two fields cannot live
    in it.

Q4. Construct a case where the measurement above leads to a wrong conclusion,
    that is, a realistic workload for a filesystem index where going from 93 byte
    entries to 109 byte entries costs significantly more than this measurement
    suggests. Name the resource that the measurement did not consume.

Q5. Construct a case where the claim "the clone ancestry table has no home on
    disk today" is misleading, given that a separate owed item already describes
    the ancestry query as nested interval labels, preorder and postorder, two
    numbers per snapshot, and 16 bytes are already earmarked inside a 32 byte
    reserved area of each tree table entry.

For each answer, state the operation sequence, the concrete values, and what
exactly goes wrong. If you cannot construct a counterexample for a question, say
so in one sentence and move to the next.
