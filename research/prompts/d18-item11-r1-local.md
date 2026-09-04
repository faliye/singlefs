You are one of three independent reviewers of a filesystem on-disk format proposal.
Your assigned stance is: find counterexamples. Do not summarise. Do not agree politely.

Answer in English. Do not use any markdown emphasis: no asterisks, no bold, no italics,
no headings. Plain sentences and plain numbered lists only.

Read this carefully before answering. Every field named below is present in the design.
Nothing is being removed. Where the text says a width or a value is decided elsewhere, it
means that number is settled in another document, not that the field disappears.

Background facts, all already settled and not open for debate.
A storage unit is written once, copy on write, never overwritten in place. A data unit
occupies 32768 bytes on a 32768 byte boundary. An index node occupies 16384 bytes on a
16384 byte boundary. A rebuild scanner steps through the device in 16384 byte strides and
probes each position for a unit header. Every unit header begins with a common plaintext
prefix of 42 bytes: magic 4, format version 2, flags 2, declared length 2, header checksum 32.
After the prefix comes a class specific identity segment. For a data unit that segment is a
five tuple of 33 bytes (unit class tag 1, tree id 8, object id 8, object birth generation 8,
anchor offset 8) plus birth checkpoint number 8 plus filesystem id 8, so the data unit header
is 91 bytes. For an index node the segment is tree id, level, key range, birth checkpoint,
filesystem id. Three self certifying structures, the root slot, the journal record and the
superblock slot, do not use this header table; each has its own magic and its own header.
When encryption is enabled every unit is sealed with an AEAD cipher whose associated data is
a fixed order encoding of the unit's logical identity, and the design states that the unit
class tag must be part of that associated data as a domain separator, because without it two
classes with different field sets could serialise to the same bytes and a type confusion would
pass as a valid authentication. The design also states that the field composition of the
associated data is a day one permanent contract: it must be fixed before encryption is ever
enabled, and changing it later is a format break. Expected values for the associated data may
only come from the reader's lookup path or from ciphertext side fields already authenticated
one level up, never from the plaintext header. After the scanner claims a unit header it skips
the whole payload by declared length, so probes that land inside a payload are ignored; this
claim rule is the only defence against a backup image of the same volume stored as a file.
Deletions write tombstone records. Tombstones are packed many per shared 32768 byte unit,
the unit is closed when a checkpoint generation ends so one unit never mixes generations, and
reclamation happens per whole unit, never per record. The authoritative on disk state is
units plus accounting plus roots; index trees are derived and can be rebuilt by scanning.
Feature bits have three tiers: incompat refuses mount, compat_ro allows read only mount,
compat is ignored. A settled table says a new record type is compat_ro.

The proposal under review.
One. The unit class tag gets a single registry with numeric codes, and four documents that
today each list their own set of class names are rewritten to point at it.
Two. The registry: code 0 invalid, never assigned. Code 1 data unit. Code 2 index node, and
accounting tree nodes are code 2 too, distinguished only by tree id. Code 3 packed record unit,
a new class. Codes 4 to 15 reserved for future content classes; a reader meeting an unregistered
code rejects that unit, and adding a content class costs one incompat feature bit. Codes 16, 17
and 18 are reserved for the root slot, the journal record and the superblock slot, purely so
that their associated data, when they are sealed, begins with a domain separator from the same
namespace; their headers stay their own. Codes 19 to 255 reserved.
Three. The tag is one byte and lives in the low byte of the 2 byte flags field of the common
prefix; the high byte is eight flag bits, all zero in version one. The first byte of the data
unit's five tuple must equal the flags low byte, mismatch means corrupt. In the associated
data the tag code is always the first byte, for every class.
Four. The packed record unit is its own class, not a data unit and not an index node. Its
identity segment is tree id 8, record type 2, sequence within generation 4, record count 2,
record width 2, birth checkpoint 8, filesystem id 8, total 34, so its header is 76 bytes.
Its associated data is tag, tree id, record type, birth checkpoint, sequence. The birth
checkpoint doubles as the loading generation because the unit is closed at generation end.
Records inside carry no per record type, length, checksum or liveness bit: the whole unit is
covered by the unit checksum or MAC, a torn unit is rejected whole, reclamation is per unit,
and records are fixed width by record type. Records are enumerated by parsing the claimed
container, not by probing.
Five. Loading rules: one record type per unit, one generation per unit, one tree per unit,
fixed width records with count times width at most declared length minus 76.
Six. Record types form a second level registry: 0 invalid, 1 tombstone range record, 2 inode
record reserved pending another decision, 3 to 65535 reserved. Adding a record type costs one
compat_ro bit; a reader that does not know a record type can still verify the container by
count and width and skip it, but must not reclaim it. Adding a record type does not change the
composition of the associated data, only the value of one field.
Seven. Neither the class tag byte nor the flags high byte may be borrowed as an aging or node
group tag by a future anti fragmentation scheme.
Eight. With a 76 byte header, a 32768 byte packed unit holds 583 tombstone records of 56 bytes
or 233 inode records of 140 bytes, unchanged from earlier experiments that assumed a 91 byte
header.

Your task. Give at most seven counterexamples. For each one state the concrete situation,
which specific settled fact or number above it contradicts, and what an implementer would
observe going wrong. Concentrate on cases the designers are least likely to have walked:
rebuilding by scanning every unit after all index trees are lost, a crash between writing a
packed unit and publishing the root that references it, a packed unit that is referenced from
two different trees or two different generations, a later software version reading an image
written by an earlier one and the reverse, a hostile actor who can write raw sectors but has no
key, the moment encryption is first enabled on a volume that already has unencrypted packed
units, and whether giving the root slot and journal record codes in this namespace actually
buys anything or creates a trap. If a part of the proposal is simply correct, say so in one
sentence and move on rather than inventing an objection.
