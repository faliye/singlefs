# Background: does extent SPLITTING still exist? (C72 second half), 2026-09-03

All F-facts below were verified against the singlefs kb on 2026-09-03 by the main agent,
quoting the settled text. F11 is the proposal under scrutiny, NOT a fact.

Context: singlefs is a from-scratch COW filesystem, still in format-design (no code).
"Unit" = the fixed on-disk allocation/checksum granularity for data. A "logical extent"
is one contiguous run of user data referenced by one pointer. This question is only about
DATA extents, not index nodes.

F1. D4 settled item 7 (2026-09-01, user decision): a logical extent's ON-DISK length is
    exactly ONE unit = 32768 bytes, never spanning multiple units. Its DECLARED length is
    variable, <= 32768. Two lengths are tracked separately (I-2.3 uses the declared length
    as the zero-padding start).

F2. D4 settled item 3 (2026-08-31): a write that does not fill a whole unit goes
    read-modify-write: read the whole old unit back and verify its MAC, re-checksum the
    WHOLE unit, then write the whole unit to a NEW location. There is no partial in-place
    overwrite of a unit.

F3. D14 settled item 3 (2026-08-31): the inline threshold is 0 -- nothing is inlined into
    the inode. Rationale is atomicity, not performance.

F4. D14 (settled): cross-file tail-packing is REJECTED -- two files sharing one unit means
    writing one forces COW of the whole unit, which forces the other file's pointer to
    change too, breaking D20's unit self-containment. Cited external fact: the only
    tail-packing filesystem in Linux 7.2.0 is erofs, and it is read-only.

F5. Compression: D4's pointer field table reserves 7 bits for "compression algorithm" as a
    day-1 reserved field. There is NO settled decision on how compression works, and (like
    encryption per D9 item 10) it does not enter the first runnable version. So the first
    version writes no compressed data.

F6. D9 day-1 reserved field #2 = "extent offset from original extent start" (this is the
    same field D19 calls extent_off and D4's field table calls "extent distance-from-start
    offset"). Its stated reason for existing: after SPLITTING, the N sub-extents SHARE one
    ciphertext, one nonce, one MAC; each sub-extent pointer uses its offset to locate the
    original nonce. "Storing the nonce is not the same as each sub-extent storing its own."

F7. D9 settled item 4: extent_off MUST NOT go into the AAD, precisely BECAUSE (under
    splitting) N sub-extents share one MAC while their offsets differ -- N AADs against 1
    MAC would make N-1 sub-extents fail verification on every normal read.

F8. D6 (snapshot model): "cross-snapshot partial-overwrite fragmentation" is listed as a
    cost already pre-paid by D9 day-1 reserved field #2 -- i.e. D6 assumes splitting is the
    mechanism that expresses a partial overwrite shared across snapshots.

F9. D19 settled item 3 (2026-08-29): pointers are FIXED-WIDTH, reserving bits even when a
    feature is off ("reserve, don't grow later"). extent_off width budget: 7 bits (crc32
    unit), 9 (crc64), 13 (crc128). These are reserved bits regardless of whether splitting
    happens.

F10. D4 settled item 7's own scope note (2026-09-01): the split question "does NOT change
     the bytes of the first transaction" (the extent_off bits are reserved fixed-width per
     F9, occupied whether used or not); what it changes is "whether the MAC gets copied
     into k copies", plus the wording in F6/F7/F8/F9.

F11. PROPOSAL UNDER SCRUTINY (assembled by the main agent from the above):
     In the first version, SPLITTING DOES NOT HAPPEN. Mechanism: with extent == unit (F1)
     and whole-unit RMW-to-new-location (F2), a partial overwrite -- including reflink and
     cross-snapshot partial overwrite (F8) -- COWs the entire 32768-byte unit to a new
     location as a brand-new unit with a fresh nonce/MAC. No operation produces "one
     physical unit referenced by multiple pointers at different offsets". Therefore
     extent_off is IDENTICALLY 0 and every unit has exactly one pointer, one MAC, one nonce.
     The only conceivable FUTURE source of splitting is compression (F5, a day-1 reserved
     feature not in v1): several compressed extents packed into one unit would revive
     extent_off. So the ruling is: splitting is a RESERVED feature, DISABLED in v1
     (extent_off == 0, MAC never copied), with the reserved bits kept (F9) exactly as
     encryption/compression are reserved-but-off. The four "will split" texts (F6/F7/F8/F9)
     get reworded from "splitting happens" to "splitting is a reserved feature, v1 produces
     none". D9 item 4's ruling (extent_off not in AAD) is KEPT, but its reason is
     re-grounded: not "because splitting makes N-1 fail today" but "extent_off is 0 today
     and, if splitting is ever enabled, it still must not go in AAD for that same reason".

QUESTION: does F11 survive? In particular, each leg with special attention to:
 (a) Is there ANY first-version operation -- reflink, dedup, RAID rebuild/resilver, journal
     replay, snapshot delete, defrag/restripe (D2), tombstone packing (D18) -- that produces
     a single physical unit referenced by multiple pointers at DIFFERENT offsets, i.e. real
     splitting with extent_off != 0? If yes, F11's "v1 has no splitting" is false.
 (b) Does "reserved feature, disabled in v1" contradict any settled text, OR is the
     stronger ruling "splitting never exists, delete the concept" actually forced (e.g. does
     compression as reserved really need sub-unit sharing, or would it too be whole-unit)?
 (c) Does re-grounding D9 item 4's reason (F7) leave any AAD/encryption reasoning broken?
 (d) If extent_off is identically 0 in v1, is keeping it as reserved bits justified, or is
     that dead weight the design should drop?
