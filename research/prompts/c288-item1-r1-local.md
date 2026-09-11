You are the counterexample leg of a three-way design review for a from-scratch
copy-on-write filesystem. Your only job is to construct concrete counterexamples
against the four candidate layouts below. Do not argue which option is better. Do not summarize.

Do not use any markdown emphasis in your answer. No bold, no italics, no asterisks.
Write plain sentences and plain numbered lists. Answer in English.

FACTS YOU CAN RELY ON

F1. Every metadata unit starts with a 42-byte common prefix (offsets 0 to 41). The unit
    type tag sits at offset 6. Type 2 is an index node, type 3 is a packed record unit.
    After the prefix comes a class identity segment whose fields depend on the type.
F2. Type 3 identity segment, every offset already fixed: 42 tag copy (1 byte), 43 birth
    tree (8), 51 packed record type (2), 53 container id (8), 61 container birth
    generation (8), 69 record count (2), 71 record width (2), 73 birth generation (8),
    81 fsid (8), 89 payload checksum (4), 93 write sequence (10, the first 4 bytes are
    the instance id). The header ends at 103. The constant 103 is registered and pinned
    by a gate check against the source of about ten experiment programs.
F3. Type 2 identity segment: only the field order is fixed, not the offsets: tree id,
    level, key range (only on trees that carry one, 16 bytes), birth generation (8),
    fsid (8), write sequence (4, instance id only), payload CRC (4), reserved (2).
F4. The 2 reserved bytes of type 2 must be zero; a nonzero value makes that record an
    I/O error; giving them a meaning requires an incompat feature bit. An open debt
    asks whether to widen this reserved area before the format freezes.
F5. With encryption on, the whole class identity segment of types 2 and 3 is encrypted
    and covered by the MAC. The plaintext whitelist for these types is the common prefix
    plus a 12-byte nonce plus a 16-byte MAC. The nonce and MAC reservation lives in the
    unit's plaintext header; header sizes including it are 131 for type 3 and 96, 105,
    114 for the three type 2 tiers. The byte offset of these 28 bytes inside the header
    is not written down anywhere.
F6. The header checksum covers offset 0 up to the last byte of that type's plaintext
    header.
F7. Type 2 has no copy of the type tag inside the ciphertext (type 3 has one at 42). The
    proposed fix, still open, adds 1 byte to the type 2 identity segment.
F8. User decision: the central mapping key for types 2 and 3 is type tag (1) + birth tree
    (8) + birth txg (8) + instance id (4) + birth sequence (4) = 25 bytes. The unit header
    grows by exactly 4 bytes for types 2 and 3 to hold the birth sequence. Birth tree is
    the identity segment's birth tree (type 3 offset 43), birth txg is the birth
    generation (type 3 offset 73), instance id is the first 4 bytes of the write
    sequence. The birth sequence is only a key component; it is never used to pick the
    newest among several versions.
F9. For the birth sequence only the counter scope is written: (tree, checkpoint number,
    instance id). Initial value, step, and whether a node rewritten twice inside one
    commit fixpoint gets a new number are not written anywhere.
F10. A full-disk scan rebuild must be able to reconstruct each unit's mapping key from
    that unit's header alone.
F11. mkfs writes a type 3 instance table unit and a type 2 tree table unit; the first
    transaction writes a type 2 inode tree root, a type 3 inode leaf container and a
    type 2 extent tree node.

CANDIDATE LAYOUTS FOR THE 4-BYTE BIRTH SEQUENCE

A. After the write sequence. Type 3: offset 103. Type 2: after write sequence, before
   payload CRC; payload CRC and reserved move by 4.
B. Appended at the end. Type 3: offset 103. Type 2: after the reserved 2 bytes, as the
   last field of the identity segment.
C. Before the reserved bytes. Type 3: offset 103. Type 2: after payload CRC, before the
   reserved 2 bytes; only the reserved bytes move by 4.
D. After the birth generation (control arm). Type 3: offset 81; fsid, payload checksum
   and write sequence each move by 4. Type 2: after birth generation.

WHAT TO PRODUCE

1. Encryption. For each layout, lay out the header with encryption on, using F5, and
   say where the nonce and MAC must go. Construct a case where the birth sequence ends
   up in plaintext, or where a plaintext piece sits between two ciphertext pieces, or
   where the header checksum range of F6 no longer ends at the right byte.
2. Scan rebuild. For each layout, construct a case where the five key components of F8
   cannot all be read from the header alone, or where they are read from the wrong
   bytes.
3. Hidden ordering constraints. Find any fact above, or any consequence of it, that
   forces the reserved bytes of type 2 to stay directly after the write sequence or
   the payload CRC, or forces the birth sequence to sit next to the instance id or the
   birth txg. If one exists, say which layouts it rules out.
4. Allocation rule gap. Construct a case where the correct offset depends on the
   missing allocation rule of F9 (initial value, step, renumbering). If none exists,
   say why the offset question is independent of the allocation rule.
5. Future fields. For each layout, list which existing fields would have to move when
   later adding the 1-byte type tag copy of F7 or widening the reserved area of F4.
If you find nothing for an item, say so plainly and list what you tried.
