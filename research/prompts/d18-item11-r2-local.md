You are one of three independent reviewers of a filesystem on-disk format proposal, second
round. Your assigned stance is: find counterexamples. Do not summarise. Do not agree politely.

Answer in English. Do not use any markdown emphasis: no asterisks, no bold, no italics,
no headings. Plain sentences and plain numbered lists only.

Read this carefully before answering. Every field named below is present in the design.
Nothing is being removed. Where the text says a value is decided elsewhere, it means that
number is settled in another document, not that the field disappears.

Background facts, all already settled and not open for debate.
A storage unit is written once, copy on write, never overwritten in place. Every unit header
begins with a common plaintext prefix of 42 bytes: magic 4, format version 2, flags 2, declared
length 2, header checksum 32. The header checksum covers only the header. Whole unit checksums
live in the parent pointer that references the unit; a rebuild scanner that walks the device
after all index trees are lost has no parent pointers. A data unit is 32768 bytes and its class
identity segment is a five tuple (class tag, tree id, object id, object birth generation, anchor
offset) plus birth checkpoint plus filesystem id; the five tuple is plaintext even when the
volume is encrypted, because the same information already sits in a plaintext reverse index
that answers which objects reference a physical unit. Encryption seals every unit with an AEAD
whose associated data is a fixed order encoding of the unit's logical identity; the composition
of that associated data is a day one permanent contract; expected values for it may only come
from the reader's lookup path or from ciphertext fields authenticated one level up, never from
the plaintext header and never read back from the pointer. Every publish, including one
triggered by fsync, increments a 64 bit checkpoint number; a snapshot generation spans many
publishes. Deletions write tombstone records; tombstones are packed many per shared 32768 byte
unit, one generation per unit, reclaimed per whole unit. Rebuild after index loss reads
tombstone units to learn which keys died; without them every deleted key resurrects. Feature
bits have three tiers: incompat refuses mount, compat_ro allows read only mount, compat is
ignored. Earlier experiments assumed a 91 byte header for packed units and computed 583
tombstone records of 56 bytes or 233 inode records of 140 bytes per unit.

The proposal under review, second version.
One. The unit class tag is one byte in the low byte of the flags field, and it is the first
byte of the associated data for every class. Registry: 0 invalid, 1 data unit, 2 index node
(accounting tree nodes are code 2, their role given by the explicit tree id field in the
header), 3 packed record unit, 4 to 15 reserved content classes, 16 root record, 17 journal
record, 18 superblock slot as a placeholder with no associated data contract yet. An
unregistered class code means the unit is rejected and adding a content class costs one
incompat bit.
Two. The packed record unit, code 3, is its own class with a 48 byte identity segment, so the
header is 90 bytes: tree id 8, packed record type 2, container number 8, container birth
generation 8, record count 2, birth checkpoint 8, filesystem id 8, payload checksum 4 (CRC32C
over the record area to the end of the unit including zero padding). The container number is
allocated monotonically per tree and per record type and never changes when the container is
rewritten; the container birth generation is the checkpoint number of the publish that
allocated the number. Together they are the container's stable logical identity, mirroring the
data unit's object id and object birth generation. The birth checkpoint field records only this
particular write and changes on every rewrite. The associated data for code 3 is (tag, tree id,
packed record type, container number, container birth generation). Records inside carry no
per record type, length, checksum or liveness bit; record width comes from a second level
registry keyed by packed record type. The condition record count times record width at most
declared length minus 90 is checked when the header is claimed.
Three. Loading rules: one packed record type per container, one snapshot generation per
container (the generation is not stored; it is derived as the snapshot interval that contains
the container birth generation, and the container is closed at a generation boundary), one tree
per container, fixed width records.
Four. An open container is rewritten, copy on write, at every publish that needs its records
durable; each rewrite is a new unit with a new birth checkpoint and the same identity. A
container index is the only holder of physical pointers to packed units; other trees reference
a container by its identity. Updating a container changes one entry in the container index, not
one pointer per record. The superseded old unit is freed through the normal copy on write free
path and no tombstone is written for it because container index entries are pointers to
physical containers, not logical keys. If a rebuild scan finds both the old and the new version
of a container, tombstone records apply idempotently and for the same identity the higher birth
checkpoint wins.
Five. Second level registry of packed record types: 0 invalid, 1 tombstone range record, 2 inode
record reserved, 3 to 65535 reserved. Adding a packed record type costs one compat_ro bit and is
only possible for fixed width types; a reader that does not know a type verifies the container
after decryption and MAC check, using the type code from the lookup key to build the associated
data, then skips the whole container by declared length and must not reclaim it. A variable
width record type cannot be expressed in code 3 and would need a new unit class.
Six. With the 90 byte header the earlier numbers 583 and 233 are unchanged; they hold for any
header width from 65 to 120 bytes and from 9 to 148 bytes respectively.

Your task. Give at most seven counterexamples. For each one state the concrete situation,
which specific settled fact or number above it contradicts, and what an implementer would
observe going wrong. Concentrate on cases the designers are least likely to have walked: the
container number allocator itself after a crash or after a rebuild from scan (who remembers
the next number), two rewrites of the same container inside one publish, a container whose
records span a snapshot boundary because the snapshot was taken between the last record and
the close, a rebuild scan that finds three or more versions of the same container, an
implementation that reclaims the old version of a container before the root that references
the new version is durable, the derivation of the generation from the snapshot table when the
snapshot table itself is what is being rebuilt, a reader with the key but an older software
version meeting packed record type 2, a 4 byte CRC32C as the only payload integrity evidence on
the scan path, and whether tombstone idempotence still holds when a tombstone record for a key
is followed by a re creation of that key and a second deletion inside the same container's
lifetime. If a part of the proposal is simply correct, say so in one sentence and move on rather
than inventing an objection.
