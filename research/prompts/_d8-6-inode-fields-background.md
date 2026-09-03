# Background: what goes into an inode record on disk? (D8 open-item 6), 2026-09-03

singlefs is a from-scratch COW filesystem in Rust, still in format-design phase: no code,
no on-disk format frozen yet. The first runnable goal is "correctly commit one transaction"
-- write one small file and publish once. No directories, no snapshots, no encryption in v1.

Writing that one file requires writing its metadata record, and the repo has NEVER settled
what that record contains. This was found on 2026-09-03 while building a byte-level
inventory of the first transaction, and filed as D8 open-item 6.

All F-facts below were verified against the repo on 2026-09-03 by the main agent,
quoting the settled text. F13 is the proposal under scrutiny, NOT a fact.

F1. D8 settled item 3: the data keyspace is keyed `(locality_id, inode, offset)`.
    Naming lives in a SEPARATE dirent tree keyed `(parent, name) -> inode`. `locality_id`
    is inherited from the parent directory at creation and deliberately NOT updated on
    rename, so rename is O(1) and data keys are decoupled from names.

F2. D8 settled item 2: index node size is 16 KiB, pinned as a constant.

F3. D14 settled item 3: the inline threshold is 0 -- NOTHING is inlined into the inode,
    not even tiny files. Rationale was atomicity, not performance.

F4. D19 settled item 3: pointers are FIXED-WIDTH with reserved bits ("reserve, don't grow
    later"). A pointer costs 53 bytes (31 header + 22 for two location entries). The
    explicit reasoning: a variable-length layout would mean "one more parser + checker +
    crash-point replay to maintain forever", and the judging rule forbids "it's faster /
    smaller" as a justification for a format branch.

F5. D18 settled items 3 and 7: every DATA unit carries a plaintext self-describing header
    whose identity section is a five-tuple: (unit type tag, tree id, OBJECT ID, OBJECT
    BIRTH GENERATION, anchor offset), 33 bytes total. So the object's id and generation
    already appear in a second place on disk. Their individual widths are NOT settled --
    experiment E85 assumed 8 bytes each, and says so explicitly as an assumption.

F6. D9 settled item 6: `locality_id` is a HINT, explicitly "not part of correctness -- it
    may be wrong, it may be stale". It must never be load-bearing for any judgment.

F7. D9 settled item 5: in the dirent tree the separator keys ARE filenames, which is
    settled as sensitive information (must not be exposed in plaintext once encryption is
    on). Encryption is not in v1 (D9 item 10) but the format bits are reserved day-1.

F8. D5 (settled): a block's birth txg is stored inline in the POINTER that references it,
    not in the object's metadata record.

F9. D6 (snapshot model): the direction taken is multiple writable heads; a candidate
    (bcachefs-style) puts the snapshot id in the LOW BITS OF THE KEY. Not fully settled,
    but it competes for key space with encryption's nonce carrier and other users.

F10. rules/fs-design.md, two standing disciplines: (a) "any metadata block picked up alone
     must be able to answer who am I, whose am I, which generation am I" -- this is stated
     as only obtainable at format level, impossible to add later; (b) "prefer writing it
     bigger -- do not sacrifice self-containment to save space; the default answer to
     'can this field be dropped / narrowed / recomputed' is DO NOT DROP, unless measurement
     proves it is the bottleneck". Also: "comparing who is more space-efficient is never
     a valid criterion in this project".

F11. rules/format-evolution.md: before the first external user the on-disk format is soft
     and may be redesigned freely, BUT the conceptual model must move slowly, because a
     wrong mental model silently propagates into hundreds of later decisions. What deserves
     care is the decision text, not the .rs files.

F12. D15 (freeze policy) exists and lists preconditions for the first format freeze; among
     them, every entry marked "format-level" in the decision index must be settled. Once
     frozen, "we can still add it later" stops being true.

F13. PROPOSAL UNDER SCRUTINY (assembled by the main agent):
     (a) SHAPE: a FIXED-LENGTH inode record with reserved trailing bytes, by direct analogy
         to F4's settled "fixed width, reserve, don't grow later". No variable-length TLV
         section in v1.
     (b) IDENTITY IS REPEATED IN THE RECORD: the record stores its own inode number and
         generation (8 bytes each, matching F5's five-tuple widths), even though the inode
         number is already part of the key that located it. Justification: F10(a) --
         a record picked up alone during scan-rebuild must be able to say whose it is.
     (c) NO EXTENT POINTERS IN THE INODE: file data is reached through the data keyspace
         `(locality_id, inode, offset)` (F1), and nothing is inlined (F3). So the inode
         holds no direct block pointers at all -- not even the "first N extents" shortcut
         that ext4/btrfs-style designs use.
     (d) FIELD SET (widths are initial values, adjustable at implementation time):
         inode number 8, generation 8, mode 4 (file type + permission bits), uid 4, gid 4,
         nlink 4, size 8, atime/mtime/ctime/btime 12 each (8-byte seconds + 4-byte
         nanoseconds) = 48, flags 8, reserved 16. Total 112 bytes.
     (e) FULL POSIX FIELD SET ON DAY 1 even though v1 has no directories and no permission
         checking: the same day-1-reservation discipline D9 applies to encryption fields
         (F7/F12) -- fields that will certainly be needed are reserved before the freeze,
         because after the freeze "we can add it later" is false.
     (f) NOT IN THE INODE, explicitly: `locality_id` (it is a hint per F6 and lives in the
         key), the block birth txg (lives in the pointer per F8), xattrs and ACLs (deferred
         to a separate keyspace, not yet designed).

QUESTION: does F13 survive? Points that deserve the most attention:
 (a) Is repeating the inode number and generation inside the record actually justified by
     F10(a), or is it redundancy that F4's own precedent (location entries do NOT repeat
     their key) argues against? What breaks concretely if it is omitted?
 (b) Does "no extent pointers in the inode at all" (c) create a problem the settled
     clauses do not already solve -- e.g. does reading a 1-block file now require an extra
     tree descent that a "first N extents inline" design would avoid, and does any settled
     clause forbid that shortcut?
 (c) Is fixed-length + reserved (a) right for a record that must eventually carry xattrs,
     ACLs, and unknown future POSIX-ish attributes, or does F4's reasoning fail to transfer
     because inodes are far fewer than pointers and grow new attributes over time?
 (d) Do the timestamp widths (48 bytes of the 112) survive F10(b)'s "do not sacrifice
     self-containment to save space" together with the ban on "smaller is better"
     arguments -- i.e. is there any correctness-based (not size-based) reason to change them?
 (e) Does F9 (snapshot id possibly in key low bits) interact with this record in a way
     that must be decided NOW rather than later, given F11 says the conceptual model must
     move slowly?
 (f) Is there any field the first transaction actually needs that F13(d) omits?
