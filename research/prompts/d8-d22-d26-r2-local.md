You are one of three independent reviewers of four filesystem design proposals, round two.
Your assigned stance is: find counterexamples. Do not summarise. Do not agree politely.

Answer in English. Do not use any markdown emphasis: no asterisks, no bold, no italics,
no headings. Plain sentences and plain numbered lists only.

Read this carefully before answering. Every field named below is present in the design.
Nothing is being removed. Where the text says a width is decided elsewhere, the number of
bytes is settled in another document, not that the field disappears. Where the text says a
field is reserved, the bytes are on disk and simply unused yet.

Settled background, not open for debate. A storage unit is 32768 bytes and starts on a 32768
byte boundary. An index node is 16384 bytes and starts on a 16384 byte boundary. Every unit
has a header; a data unit header is 91 bytes and contains a five part identity: unit class tag,
tree id, object id, object birth generation, anchor offset. The authoritative on disk state is
units plus accounting plus roots; the index is derived and rebuilt by scanning. A settled rule
says that putting a value that cannot be recomputed into a rebuildable container promotes that
value to authoritative. Checkpoints publish up to 2785 times per second; the checkpoint number
is 64 bits and never goes backwards. One superblock slot is one hardware sector, 512 or 4096
bytes. Deleted-object tombstones are already stored as interval records packed into shared units.

Round two proposals, after round one removed the first versions.

One. File metadata records, 140 bytes each, are packed into shared 32768 byte units, 233 per
unit, exactly like the tombstones already are. A separate tree only routes to them. Measurement:
11 of the 14 fields cannot be recomputed by scanning, so packing them into units keeps them
inside the authoritative state, whereas putting them inside index nodes would not.

Two. The write buffer sequence number is 12 bytes: 8 bytes of checkpoint number plus 4 bytes of
counter within the publish window. Measurement: 8 bytes leaves zero bits for the counter, and
one publish window can hold up to 71582788 accounting entries, needing at least 27 bits. Only a
sequence number derived from the checkpoint number picks the right winner across a remount; a
wall clock picks wrong for all 1000 test keys and a per-mount counter leaves all 1000 undefined.

Three. The superblock keeps a fixed 420 bytes in its slot by moving the device table out into a
separate unit and storing only a 59 byte pointer to it. Measurement: with the table inline, a
512 byte slot holds only 2 devices; 8 devices overflows by 41 bytes and creates 2 readable but
inconsistent states after a torn write; 256 devices creates 8190 such states.

Four. A tag field in the index node header is proposed so a future anti-aging scheme can be
expressed without changing node layout. Measurement: one candidate scheme needs 49 distinct tag
values and fits in 1 byte, but the other candidate has no derivable bound on how many values it
needs, so no width can be chosen, so the first transaction still cannot be written. The proposal
now admits it does not unblock anything.

Your task. Give at most six counterexamples. For each state the concrete situation, which
specific number or rule above it contradicts, and what an implementer would observe going wrong.
Concentrate on what a counting model cannot see: rewriting one record inside a shared 32768 byte
unit, what the shared unit's single object id should say when it holds 233 different objects,
crash between writing a unit and writing the tree that points at it, the device table unit's own
address needing to be known before the device table can be read, and a filesystem upgraded years
later. If a proposal is simply correct, say so in one sentence and move on.
