# Background: unit-header field table and whitelist closure (C68/C69), 2026-09-02

All facts below were verified against the singlefs kb on 2026-09-02 by the main agent.
F12 is the proposal under attack, not a fact.

F1. Invariant I-6.2 (invariants.md, settled wording): "when encryption is enabled, any
    METADATA block's plaintext header field set must be a subset of the whitelist
    {magic, format version, flags, nonce epoch, length, MAC}". Its literal scope is
    metadata blocks only; nothing in the repo says whether it extends to data units.

F2. D18 item 3 (settled, user decision): every DATA unit carries a PLAINTEXT
    self-description header with the five-tuple (unit type tag, tree id, object id,
    object birth generation, anchor offset). Rationale: the same information already
    lives in plaintext elsewhere (D9 item 5 settled a plaintext logical-to-physical
    mapping layer plus plaintext reverse index), so this adds no new leak surface.
    Forbidden in the header: physical location / device id; filenames / separator keys.
    Note: "object birth generation" is the inode generation, NOT the block's birth txg.

F3. C69 (checks-owed): invariant I-1.2 requires "block header's recorded generation
    <= superblock's current generation"; I-1.4 requires "block header's fsid == superblock
    fsid". The five-tuple contains neither field, so both settled invariants currently
    have no judgment field in the first version.

F4. D18 open-item-7 skeleton (rulings already recorded there): fsid was RULED OUT of the
    block header (D9 item 8 settled fsid goes into the KDF; a header fsid would become
    permanently redundant once encryption is on; the unprotected pre-encryption window
    is registered as C74 and is to be covered by a mount-time check). Still listed as
    pending there: block birth generation (named by I-1.2 and by the incremental-scrub
    watermark) and a header checksum (during scan-rebuild there is no parent pointer,
    so header integrity currently has zero protection).

F5. D5 (settled): a block's birth txg is stored INLINE IN THE POINTER that references it
    (parent side), granularity = publication. D4 (settled): the child's checksum/MAC also
    lives in the parent pointer (Merkle). So for any REACHABLE block, generation and
    integrity credentials are on the parent side already.

F6. D18 item 1 (settled): the "lineage rewrite sequence number" candidate was REJECTED;
    distinguishing stale copies from current ones during scan-rebuild is bought by
    D3 item 1's per-drop-point predicate (the allocation-record tree answers "is this
    drop point part of the current version"), measured in E59 (adding the predicate
    drove all four message-class divergences to zero).

F7. D9 item 6 (settled) lists as a day-1 format requirement: "a btree NODE's logical
    identity needs a definition (candidate: tree id + level + lineage sequence)" - note
    the lineage sequence was later rejected (F6). Also: the AAD field list only covers
    data extents. D18 item 2 (settled): accounting-tree nodes carry their key range
    [min_key, max_key] in the node header.

F8. E85 (2026-09-02, requirements matrix, mutation-proofed): the settled minimal set
    (five-tuple + declared length, 35 bytes under stated width assumptions) cannot judge
    I-1.2, I-1.4, header self-integrity, or the scanner magic probe. The literal
    whitelist set cannot even support scan-rebuild (no five-tuple). A repair set adding
    magic/version/flags/fsid/block-birth/header-csum (71 bytes, 0.22% of a 32 KiB unit)
    covers everything except "self logical address", whose meaning per unit class is
    itself unsettled. Conflicts between the repair set and the F1 whitelist: exactly
    eight fields.

F9. E86 (2026-09-02): the scan-rebuild scanner steps at the pool's smallest unit size and
    relies on a magic probe plus header checksum to reject interior probes of data-unit
    payloads (one interior probe per data unit at 16 KiB stepping).

F10. D21 (settled): scan-rebuild is the ONLY backward-compatibility path of the design;
     when encryption is on, self-description lives in ciphertext, so keyless mode cannot
     rebuild, only scrub. That is the context in which the F1 whitelist was written.

F11. D4 item 6 (settled): the extent declared length (2 bytes) lives in the unit
     self-description header, because I-2.3 (padding must be zero) must be judgable
     during scan with no index available.

F12. PROPOSAL UNDER ATTACK (assembled by the main agent from the above; approved by the
     user contingent on this adversarial round):
     (a) v1 unit header field table = { magic 4, format version 2, flags 2,
         five-tuple 33, declared length 2, header checksum 4 } = 47 bytes under E85's
         width assumptions; initial values, widths adjustable at implementation time.
     (b) fsid: NOT in the header. I-1.4 is rescoped: fsid binding is judged at mount
         time against the superblock (pre-encryption, per C74) and by the KDF once
         encryption is on; the invariant's wording must change accordingly.
     (c) block birth: NOT in the header. I-1.2 is rescoped: the generation judgment
         fields are the parent pointer's birth txg (F5) for reachable blocks, plus the
         root record's txg; scan-rebuild's stale-vs-current question is answered by the
         allocation-record predicate (F6), not by a header field. The incremental-scrub
         watermark walks pointers anyway.
     (d) self logical address (I-1.1): defined per unit class - for data units it IS
         the five-tuple (tree, object, anchor offset); for index nodes it is
         (tree id, level, key range) with the key range field already settled for the
         accounting tree by D18 item 2; I-1.1's wording must change accordingly.
     (e) I-6.2 is rewritten as TWO whitelists by unit class: with encryption on,
         a METADATA block's plaintext header stays the original six
         {magic, version, flags, nonce epoch, length, MAC} (five-tuple and key range go
         into ciphertext for metadata; keyless rebuild is already impossible per F10);
         a DATA unit's plaintext header = (a)'s set plus nonce epoch and MAC
         (the plaintext five-tuple is D18 item 3's settled, informed choice).

QUESTION: does F12 survive? Each leg should attack or derive per its stance, with
special attention to: whether dropping block-birth breaks orphan-block handling or
scrub in ways F12(c) misses; whether the two-whitelist split in (e) breaks any settled
encryption reasoning (D9 items 5/6, A6: AAD expectations never come from the plaintext
side); whether (d) leaves any unit class without a defined identity (journal records
and root slots are self-certifying units with their own headers per D22/D23 - are they
exempt from I-1 class or do they need rows); and whether any settled text contradicts
the rescoping of I-1.2/I-1.4.
