You are the counterexample leg of a three-way design review for a from-scratch
copy-on-write filesystem. Your only job is to construct concrete counterexamples.
Do not argue which option is better. Do not summarize. Find cases that break things.

Do not use any markdown emphasis in your answer. No bold, no italics, no asterisks.
Write plain sentences and plain numbered lists. Answer in English.

BACKGROUND

A settled clause says: a central map is the only entry point for dereference and
for the free decision. The position entries inside tree pointers (device 4 bytes,
16 KiB slot number 6 bytes, ciphertext checksum 4 bytes) are only hints and may be
stale. Freeing always goes through the map, never through a hint. Relocating a unit
only rewrites its one map entry; nobody who references the unit is touched.

The map is not an authoritative structure. It must be rebuildable by scanning the
units on disk. A settled test says: if a value cannot be rebuilt, ask whether a
reader then gets a conservative answer or a wrong answer; a wrong answer means the
value must be evicted from the structure or the structure must become authoritative.

Facts you can rely on:
F1. A parent pointer's checksum or MAC covers the whole child unit, header
    included. So relocation must copy the unit byte for byte; everything in a unit
    header is the value it had at birth.
F2. Today a pointer is a 31 byte head (MAC 16, nonce 12, algorithm 1, extent
    offset 2) plus two 14 byte position entries, 59 bytes total. It carries no
    birth transaction number and no birth tree.
F3. A data unit header carries: a logical five-tuple (type tag 1, tree id 8,
    object id 8, object birth generation 8, anchor offset 8, total 33), a birth
    checkpoint number 8, a filesystem id 8, a write order 10, a payload CRC 4. The
    tree id in the five-tuple is the tree the unit was born in; when a snapshot or
    clone shares the unit it differs from the referencing tree. An index node header
    carries: tree id, level, a key range only for some trees, birth checkpoint
    number, filesystem id, write order 4, payload CRC 4. Index nodes have no
    five-tuple.
F4. The snapshot free decision reads only ptr.birth and head.prev_snap_txg and is
    documented as zero extra I/O. Today ptr.birth does not exist in the pointer.
F5. The birth checkpoint number of a unit is the checkpoint that published it.
    After a crash the same checkpoint number is issued again.
F6. Encryption rule: the expected value of every AAD field must come from the
    reader's lookup path or from an already authenticated ciphertext field, never
    read back from the pointer, never from plaintext. The only measured working
    design needs the pointer to declare birth tree and birth checkpoint number.
F7. Reflink is a day-one promise. Snapshots and writable clones share units.
    Small objects can be packed into shared containers by a background job; each
    slot in a container carries the object's own identity so a reader can find it
    inside the container.

The open question is which key the map uses. Four candidates:

K1 logical identity key. Key = five-tuple 33 + birth checkpoint number 8 = 41 bytes.
   The referencing side must assemble it: type tag from the lookup path, object id
   and anchor offset from the extent tree key, object birth generation from one
   lookup of the object's inode record, birth tree and birth checkpoint number
   from two new 8 byte pointer fields. Index nodes need their own key: either
   (tree id, level, key range lower bound, birth checkpoint number), which requires
   every tree to carry key ranges, or index nodes stay out of the map and use
   pointer authority.

K2 surrogate counter key. Key = a pool wide monotonic 8 byte number issued per
   physical version at write time. The pointer carries it (+8). Every unit header
   carries it (+8) so a scan can rebuild the map. The high water mark lives in the
   root record and is recovered as the max over all root records in the root ring.

K3 birth identity key. Key = birth tree 8 + birth checkpoint number 8 + birth
   sequence 4 = 20 bytes. The birth sequence counts units born in that tree in
   that checkpoint, from 0, issued in memory. The pointer carries all 20 bytes.
   Every unit header adds the 4 byte sequence; birth tree and birth checkpoint
   number are already in every header.

K4 birth location key. Key = birth device 4 + birth slot 6 + birth checkpoint
   number 8 = 18 bytes. The pointer's first position entry is frozen at the birth
   location and never refreshed, so it doubles as the first two key segments; the
   pointer adds the birth checkpoint number 8. Every unit header adds its birth
   location 10 so a scan can rebuild the key after relocation.

YOUR TASK

For each item below, construct a concrete counterexample with a specific sequence
of operations and specific values, or say in one sentence that you could not.
Try hardest against K3.

Q1. Scan rebuild. For each candidate, find a unit class and an operation sequence
    where a scan over unit headers alone cannot rebuild the exact key that the
    referencing pointer holds.

Q2. Key construction. For each candidate, find a path that must enter the map
    (dereference, snapshot free decision, deadlist processing, relocation,
    consistency checker, scan rebuild) but does not hold the full key at that
    moment without one extra unit read or one extra tree lookup.

Q3. Sharing. For each candidate, find a sharing case (snapshot, clone, reflink,
    packed container) where two referrers end up with different keys for the same
    physical version, or one referrer cannot produce the key.

Q4. Crash reuse. For each candidate, find a crash and rollback sequence after
    which one key names two different units that are both on disk and both look
    published.

Q5. Uniqueness under reuse. For each candidate, find a sequence of free, reuse,
    relocation or clone operations after which two live versions have the same key.

Q6. A fifth candidate. Construct a key family that is none of K1 to K4 and that is
    strictly better than all four on at least one of Q1 to Q5 and not worse on the
    others. State its key, its width, who carries it, and what it costs.

For each answer, state the operation sequence, the concrete values, and what
exactly goes wrong.
