You are one of three independent reviewers of four filesystem design proposals, round three.
Your assigned stance is: find counterexamples. Do not summarise. Do not agree politely.

Answer in English. Do not use any markdown emphasis: no asterisks, no bold, no italics,
no headings. Plain sentences and plain numbered lists only.

Read this carefully before answering. Every field named below is present in the design.
Nothing is being removed. Where the text says a width is decided elsewhere, that number is
settled in another document, not that the field disappears. Where a proposal says a decision is
deferred, the field still exists; only the choice of value is deferred.

Settled background, not open for debate. A storage unit is 32768 bytes; an index node is 16384
bytes. Every unit carries a header whose common prefix is 42 bytes; a data unit header is 91
bytes and carries a five part identity: unit class tag, tree id, object id, object birth
generation, anchor offset. The unit class list today has exactly two content classes, data unit
and index node, plus three self-describing exemptions. A scanner that claims a unit skips its
whole payload and does not probe inside it. Deleted-object records are already packed many to a
shared unit. Feature bits are three bitmaps of 256 bits each, held in the superblock. The
superblock is written per device and updated by rotating slots; one slot is one hardware sector.
The format may be changed freely until the first outside user exists, after which it is frozen
in four layers: superblock and block headers, pointer field layout, per-keyspace key encoding,
and index node internal layout.

Round three state of the four proposals.

One. File metadata records, 140 bytes, packed many to a shared 32768 byte unit, 233 per unit.
Measured: 11 of 14 fields cannot be recomputed by scanning, so this placement keeps them inside
the authoritative state. Two gaps remain. A packed shared unit is not any of the listed unit
classes, and its single object id cannot name 233 different objects. The scanner skips payloads
of claimed units, so the 233 records inside are never independently discovered.

Two. The write buffer sequence number is 4 bytes holding only a counter within one publish
window, because the accounting key already contains the checkpoint number, so entries sharing a
key necessarily belong to one window. Measured: one window can hold 71582788 entries, needing 27
bits. Truncating the checkpoint number inside the key instead causes 757947392 pairs of entries
from different checkpoints to collapse onto one key after ten million publishes at 16 bits.

Three. The superblock moves the device table out into its own unit and keeps a 59 byte pointer,
giving a constant 420 bytes. Measured: with the table inline a 512 byte sector holds only 2
devices. But the first version runs exactly 2 devices, and a 4096 byte sector holds 64 inline,
so the overflow only happens on an all-512-byte-sector pool with seven or more devices. Also,
116 of those 420 bytes are widths nobody has ever decided.

Four. No tag field is added to the index node header. Measured: one candidate anti-aging scheme
needs 49 tag values and fits in one byte, but another candidate has no derivable bound, so no
width can be chosen.

Your task. Give at most six counterexamples aimed at permanence and later evolution. For each
state the concrete future situation, which specific rule or number above it contradicts, and
what an implementer would be forced to do. Concentrate on: adding a new unit class years later,
a scanner from an old version meeting a unit class it does not know, a tree whose key does not
contain a checkpoint number, raising the dirty-data threshold that the 27 bit figure was derived
from, a pool that grows from two devices to twenty, and adding a field to a header that has no
reserved space. If a proposal is simply correct, say so in one sentence and move on.
