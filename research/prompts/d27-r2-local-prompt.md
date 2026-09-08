You are the counterexample leg of a three-way design review for a from-scratch
copy-on-write filesystem. Your job is to find concrete constructions that break
the proposal. Do not use any markdown emphasis (no bold, no italics, no asterisks).
Answer in English. Number your findings.

BACKGROUND (all clauses below are already settled in this project unless marked OPEN):

Unit: every unit on disk is exactly 32768 bytes including its header. Short extents
are padded to a whole unit. There is no variable length unit in the format.

Central mapping: it is the single entry point for dereferencing a block and for
deciding whether a block can be freed. Its key is a logical identity five tuple of
33 bytes. Its value is w copies of a location entry. A location entry is 14 bytes:
device 4, physical offset 6 encoded as a 16 KiB slot number, ciphertext checksum 4.
Pointers are fixed width; when encryption is off the crypto bytes are still reserved.
MAC and nonce live in the pointer header, one set per logical extent.

Encryption: nonce must never repeat. The invariant reads: when encryption is on, any
two blocks that have ever been written out, whether or not they are still referenced,
must differ in the pair (key generation, nonce). The AEAD associated data binds five
logical identity fields: unit class tag, tree id, object id, object birth generation,
anchor offset. Physical position never enters the associated data. That five field
table is known to be blind to the snapshot dimension and is owed a reopening.

Packed record units (unit class code 3) already exist for metadata records. Other
trees refer to such a container by a 26 byte identity reference: birth tree 8,
packed record type 2, container number 8, container birth generation 8. There are
four loading disciplines for code 3 containers: one record type per container; one
generation per container, but this one is explicitly scoped to record types that are
reclaimed generation by generation, and record type 2 (inode records) is named as
exempt because its records die one by one and its containers are reclaimed by B tree
split and merge; one tree per container; fixed record width with record count times
record width no greater than the declared length.

THE PROPOSAL UNDER REVIEW (partly settled, partly OPEN):

Small data objects of at most 4 KiB are to be moved into shared 32 KiB containers,
so that a 512 byte object no longer occupies a whole 32768 byte unit. Settled: the
move is done by a background compactor after the object has already been written
normally, so the write path is unchanged. OPEN: which unit class code the container
takes; where the container index lives; where the live slot count lives; when
compaction triggers and in what batch; where the nonce and MAC for a slot live;
which loading disciplines apply on the data side; what the associated data for a
container binds.

TWO NEW CONSTRAINTS FROM THE PROJECT OWNER:

C1. Modifying an object that lives in a slot must not bypass copy on write.
C2. No new kind of pointer may be invented.

THREE DEFECTS ALREADY ESTABLISHED IN THE ARITHMETIC MODEL THAT JUSTIFIED THE PROPOSAL:

D1. The model computed container count as ceil(N / capacity), silently assuming that
no loading discipline applies on the data side. That assumption is exactly one of the
open questions. If one generation per container were required, the container count
becomes a function of how many small objects land in a single publish, and at one
object per publish the whole ledger goes negative.

D2. The model's write ledger counts only data units. No metadata write is counted for
either arm. In particular, when the compactor moves an object, the central mapping
entry for that object must be rewritten, which means a leaf copy on write plus the
path to the root plus a journal record. That second rewrite is not in the ledger.

D3. The model does not include nonce or MAC bytes in a slot, and it models compaction
as pure byte movement. Because of the nonce rule, relocation must decrypt and
re-encrypt, whatever the associated data binds.

YOUR TASK. Give concrete constructions, not opinions. For each, give the exact
sequence of operations, the exact on disk state, and which named clause it breaks.

Q1. A 14 byte location entry addresses at 16 KiB granularity. A slot inside a 32 KiB
container sits at a byte offset. Construct the failure that follows if the central
mapping is left as the single dereference entry point for a packed object. Then
construct a failure for the alternative in which the mapping value becomes either a
location entry or a 26 byte identity reference depending on the object.

Q2. Under constraint C1, an object living in a slot is modified. Enumerate every
shape the modification can take, and for each give a concrete crash point where the
result is neither the old object nor the new object.

Q3. Under the nonce rule, construct the worst case where relocation by the background
compactor produces a state that a scanner reading raw disk cannot resolve, including
the case where the compactor crashes after writing the container but before the
central mapping update is published.

Q4. Suppose one tree per container is required on the data side. Construct a workload
where this makes the proposal lose to the current padding behaviour, with numbers.

Q5. Name any construction you tried and could not make work. Say what you tried.
