You are one of three independent reviewers of four filesystem design proposals.
Your assigned stance is: find counterexamples. Do not summarise. Do not agree politely.

Answer in English. Do not use any markdown emphasis: no asterisks, no bold, no italics,
no headings. Plain sentences and plain numbered lists only.

Read this carefully before answering. Every field named below is present in the design.
Nothing is being removed. Where the text says a width or a value is decided elsewhere, it
means that number is settled in another document, not that the field disappears. Where the
text says a field is reserved and always zero in version one, it means the bytes are on disk
and are simply not used yet, not that the field is absent.

Background facts, all already settled and not open for debate.
A storage unit occupies 32768 bytes and starts on a 32768 byte boundary. An index node
occupies 16384 bytes and starts on a 16384 byte boundary. Every unit carries a header whose
common prefix is 42 bytes: magic, format version, flags including a unit class tag, declared
length, and a header checksum. The authoritative on disk state is defined as units plus
accounting plus roots; the index is derived state and can be rebuilt by scanning. A rule says
that if something cannot be recomputed from the authoritative state, then it has in fact been
promoted to authoritative and the definition of authoritative state must be widened.
Checkpoints are published up to 2785 times per second and each publish increments a 64 bit
checkpoint number that never goes backwards.

Proposal one. File metadata records, holding size, mode, owner, link count, timestamps and
a locality prefix, are stored as values inside the leaf nodes of a metadata b-tree. Those leaf
nodes are ordinary units and carry the ordinary unit header, so the scan unit during rebuild
is the node, not the individual record. The claim is that this keeps the definition of
authoritative state unchanged, because the leaf nodes are themselves units. Each record also
carries a change counter whose value is the checkpoint number of the publish that produced it.

Proposal two. A staging structure called the write buffer batches high frequency small updates
and merges them by key before inserting them into their target trees. Each buffer entry carries
a sequence number so that the merge rule last writer wins is deterministic. The proposal is that
this sequence number lives only in the buffer entry, never in the target tree entry, is 8 bytes
wide, and its value is the checkpoint number of the publish that produced it combined with a
counter within that publish window.

Proposal three. The superblock is split into three sections by who is allowed to change them.
Section one is a bootstrap header that never changes: magic, format version, feature bits,
filesystem identifier, this device's identifier, a slot generation number and a whole unit
checksum. Section two is geometry fixed at format time and read only afterwards: the device
table, node size, unit size, allocation granularity, journal geometry, root ring parameters and
the tree table. Section three is tunable values that may change at any time without changing the
format. The superblock is written once per device and updated by rotating among at least two
slots, and one slot is one hardware sector.

Proposal four. To fight long term fragmentation, the policy half is decided now: clustered
placement plus an optional neighbour rewrite radius, both runtime knobs. The format half is only
reserved: a tag field is added to the index node header, always zero in version one, so that a
future aging scheme that groups nodes or separates them by generation can be expressed by
assigning that tag plus one compatible feature bit, without changing the node layout.

Your task. Give at most six counterexamples across the four proposals. For each one state the
concrete situation, which specific rule or number above it contradicts, and what an implementer
would observe going wrong. Concentrate on cases the designers are least likely to have walked:
rebuilding a tree by scanning every unit on a device, crash restart between two of the steps,
two writers holding different versions of the same object, a device whose sector is 512 bytes,
a filesystem that has been running for years, and a later version that must still mount an old
image. If a proposal is simply correct, say so in one sentence and move on rather than inventing
an objection.
