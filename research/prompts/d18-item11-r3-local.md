You are one of three independent reviewers of a filesystem on-disk format proposal, third
round. Your assigned stance is: find counterexamples in the parts that changed since the
second round. Do not summarise. Do not agree politely.

Answer in English. Do not use any markdown emphasis: no asterisks, no bold, no italics,
no headings. Plain sentences and plain numbered lists only.

Background facts, all already settled and not open for debate.
A storage unit is written once, copy on write. Every unit header begins with a common
plaintext prefix of 42 bytes: magic at offset 0 (4 bytes), format version at 4 (2), flags at 6
(2), declared length at 8 (2), header checksum at 10 (32). After the prefix comes a class
specific identity segment. A data unit's segment is a five tuple (class tag 1, tree id 8,
object id 8, object birth generation 8, anchor offset 8) plus birth checkpoint 8 plus
filesystem id 8, so its header is 91 bytes. Whole unit checksums and MACs live in the parent
pointer; a rebuild scanner has no parent pointers. Declared length counts payload bytes after
the header; padding after the declared length up to the end of the 32768 byte unit must be
zero. When encryption is on, an index node's or packed unit's identity segment is ciphertext
covered by the MAC; a data unit's five tuple stays plaintext. A settled rule says a sequence
number for telling stale copies from current ones was rejected because that ability is
already provided by a per location predicate built from allocation records plus per snapshot
dead lists; rebuilding that predicate from units plus accounting was measured to resolve all
overwrites. Feature bits are three 256 bit bitmaps in the superblock; a bit once used is
never recycled. Every fsync is a publish and increments the checkpoint number. A tombstone
range record is written for every deleted extent; skipping tombstones during rebuild makes
every deleted key resurrect. Earlier experiments pinned 583 tombstone records of 56 bytes or
233 inode records of 140 bytes per 32768 byte packed unit; those numbers hold for any header
from 65 to 120 bytes (tombstones) or 9 to 148 bytes (inodes).

What changed in the third version.
One. The class tag is defined as the byte at offset 6 of the common prefix; the byte at offset
7 holds eight flag bits that are all zero in version one, and a reader that sees any nonzero
flag bit rejects the unit as corrupt; assigning meaning to a flag bit later therefore costs an
incompat feature bit. The two bytes of the flags field are defined independently so no byte
order is involved.
Two. The packed record unit's identity segment is now 51 bytes, header 93: class tag copy 1,
birth tree id 8, packed record type 2, container number 8, container birth generation 8,
record count 2, record width 2, birth checkpoint 8, filesystem id 8, payload CRC32C 4 covering
offset 93 to the end of the unit including zero padding. The header checksum is defined to
cover offsets 0 through the end of the plaintext header of that class (0 to 92 for packed
units), with the checksum field itself taken as zero; when encryption is on and the identity
segment is ciphertext, the header checksum covers only the part that stays plaintext and the
MAC covers the rest. The condition record count times record width at most declared length,
and declared length at most 32768 minus 93, is checked when the header is claimed, before the
payload checksum is computed.
Three. When a rebuild scan finds several versions of a container with the same identity, it
does not pick the highest birth checkpoint. Each version is judged by the per location
predicate (allocation records plus dead lists, rebuilt from units plus accounting) to decide
which head's current state it belongs to; two versions can belong to two different heads at
once, for example an old version still referenced by a snapshot and a new version referenced
by the writable head. Candidates must first pass the rule birth checkpoint at most the root's
checkpoint number.
Four. The container's snapshot generation is not stored. It is derived only at reclaim time
from the live snapshot list as the interval containing the container birth generation; a
rebuild scan never needs it. Before a snapshot is published the currently open container must
be closed and written.
Five. Adding a packed record type is at least compat_ro, with four restrictions: fixed width
only; under encryption the container is verified only after decryption and MAC check; the type
code is a value in the associated data, not part of its composition; and compat_ro is allowed
only for record types whose skipping changes no answer, so any successor of the tombstone type
or of the inode type must be incompat instead. A compat_ro reader must never run a rebuild.
Six. The tombstone range record now carries object birth generation in addition to object id
and range, so that two deaths of a reused object id can be told apart.
Seven. Codes 1, 2 and 3 freeze when the first image is written; codes 16, 17 and 18 exist only
as the first byte of associated data and freeze when encryption is first enabled; codes are
never recycled.

Your task. Give at most seven counterexamples against the changed parts only. For each one
state the concrete situation, which settled fact or number above it contradicts, and what an
implementer would observe going wrong. Concentrate on: a header checksum that covers a field
which is itself a checksum of the payload, and what a scanner sees when the payload is torn
but the header landed; the per location predicate being built from the very units it is
judging; a snapshot published by a crash restart before the open container was closed; a
flag bit policy that rejects units written by a newer version even when that bit was meant to
be harmless; a tombstone record whose object birth generation is older than the data unit it
should kill; and whether defining the header checksum coverage differently for the encrypted
and unencrypted states creates two on disk formats under one format version. If a part is
simply correct, say so in one sentence and move on rather than inventing an objection.
