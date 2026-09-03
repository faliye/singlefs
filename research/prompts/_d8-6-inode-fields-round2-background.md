# Background: inode record on disk, ROUND 2 (D8 open-item 6), 2026-09-03

singlefs is a from-scratch COW filesystem in Rust, format-design phase, no code yet.
First runnable goal: "correctly commit one transaction" -- write one small file and
publish once. No directories, no snapshots, no encryption in v1.

**This is the SECOND round.** A first proposal was sent to three legs and DID NOT SURVIVE.
F14-F22 below record what round 1 established; F23 is the revised proposal under scrutiny.
Everything marked F was verified against the repo by the main agent, quoting settled text.

## Settled clauses (unchanged from round 1)

F1. D8 settled item 3: data keyspace keyed `(locality_id, inode, offset)`. Naming lives in
    a SEPARATE dirent tree keyed `(parent, name) -> inode`, whose VALUE is just the inode
    number. `locality_id` is inherited from the parent directory at creation and, quoting
    the settled text, "must NEVER be updated by rename".

F2. D8 settled item 2: index node size 16 KiB, pinned.

F3. D14 settled item 3: inline threshold is 0 -- nothing is inlined into the inode.

F4. D19 settled item 3: pointers are FIXED-WIDTH with reserved bits ("reserve, don't grow
    later"), 53 bytes. Stated reason: a variable-length layout means "one more parser +
    checker + crash-point replay to maintain forever". The project's judging rule forbids
    "it's faster / smaller" as justification for a format branch.

F5. D18 settled items 3 and 7: every DATA unit carries a plaintext five-tuple identity
    (unit type tag, tree id, object id, object BIRTH generation, anchor offset), 33 bytes.
    `locality_id` is NOT among them. Individual widths are NOT settled (E85 assumed 8 bytes
    each and says so explicitly).

F6. D9 settled item 6 / D8 settled item 3: `locality_id` is "a hint, not part of
    correctness -- may be wrong, may be stale".

F8. D5 settled: a block's birth txg is inline in the POINTER, not in the object record.

F9. D6: snapshot direction is multiple writable heads; a candidate puts the snapshot id in
    the LOW BITS OF THE KEY, competing for key space with the encryption nonce carrier.
    Not settled.

F10. rules/fs-design.md: (a) "any metadata block picked up alone must answer who am I,
     whose am I, which generation am I" -- obtainable only at format level; (b) "prefer
     writing it bigger; do not sacrifice self-containment to save space"; and "comparing
     who is more space-efficient is NEVER a valid criterion in this project".

F11. rules/format-evolution.md: format is soft before the first external user, but the
     CONCEPTUAL MODEL must move slowly.

F12. D15 freeze policy: every format-level item must be settled before the first freeze;
     after the freeze "we can add it later" stops being true.

## What round 1 established (verified by the main agent after the legs reported)

F14. **`locality_id` has exactly ONE witness on disk and the reader cannot obtain it.**
     Walk `mv a/f b/f` then `open("b/f")`: the dirent tree yields inode N (F1), but the
     data lives at `(L_a, N, *)` where `L_a` is the locality inherited from `a` at creation
     and never updated (F1). The dirent value carries only the inode number; the data-unit
     five-tuple does not contain `locality_id` (F5); nothing else on disk holds it. So the
     reader has no way to construct the data key except by scanning. Same for rebuild:
     unit headers give `(inode, offset)` but the leading key component must be invented.
     ⇒ **The inode record MUST store `locality_id`.** Round 1's "it's only a hint so don't
     store it" was a misreading of F6: F6 licenses distrusting its MEANING (locality may
     have decayed), not discarding its BITS -- as an address component the value is exact
     and definitional.

F15. **v1 cannot construct a data key at all.** v1 has no directories, and F1 sources
     `locality_id` from the parent at creation. With no parent there is no defined value.
     A v1 default must be settled together with the field.

F16. **`st_blocks` is a recoverability requirement, not a convenience.** With no extent
     pointers in the record, and no stored block count, `size = 4 MiB` with one data unit
     found is indistinguishable from "legitimately sparse" versus "catastrophically lost" --
     the checker has no independent counterpart to compare "sum of units found" against.
     Additionally, in THIS design the number cannot be derived from `size`: D4 settled
     item 2 pads any short extent to a full 32 KiB unit (its text says a 1 KiB file occupies
     32x), so size and occupancy diverge by design.

F17. **The record cannot order two versions of one inode.** F5's generation is a BIRTH
     generation, constant over the object's life, so two records for inode 42 found during
     scan carry the same value. Wall-clock timestamps cannot break the tie either (NTP
     steps, settimeofday, VM clock jumps). Note the asymmetry: pointers are self-dating
     (F8) while the inode record is not.

F18. **"generation" is a homonym in this repo** with three distinct referents: F5's object
     BIRTH generation, an inode-number REUSE counter, and node-level COW generation.
     A field named plain "generation" is ambiguous.

F19. **`st_rdev` is absent from any plan**, so device nodes cannot be represented, and
     xattrs (the only side channel) are deferred.

F20. **The inode tree's own key layout was never settled.** F1's `(locality_id, inode,
     offset)` is the EXTENT tree's key. D8's structural layer settles "one tree per data
     class (extent / inode / dirent / reverse index / free space / accounting)", so an
     inode tree exists, but its key is unwritten.

F21. **v1 `nlink` is undefined and both obvious values break something.** v1 has no dirent
     tree, so the single file has no name. `nlink = 1` violates any "nlink equals dirent
     count" invariant on the very first image; `nlink = 0` invites an orphan-reclaim pass
     to delete the only file. Reachable in the first transaction.

F22. **Any duplicated identity field needs a stated authority and mismatch rule** (key says
     42, record says 43: repair? EIO? trust which?). Without it, redundancy converts key
     corruption into a silently ignored discrepancy, and the kernel path and the checker
     will diverge.

## F23. REVISED PROPOSAL UNDER SCRUTINY

(a) **Shape**: fixed-length record, reserved tail, no variable-length section -- by analogy
    to F4. The overflow route for genuinely open-ended attributes (xattrs, ACLs, quota /
    project id, crypto policy) is a SEPARATE attribute keyspace, plus a presence bit in
    `flags` whose semantics are defined NOW: if the bit is set and the reader does not
    understand the attribute keyspace, it must refuse the operation rather than proceed
    (so a reader that ignores an ACL cannot silently apply mode bits only).

(b) **Field table** (widths are initial values; the identity widths are provisional and
    pinned to D18's five-tuple, which is itself unsettled per F5):

    | field | bytes | why |
    |---|---|---|
    | inode number | 8 | F10(a) self-identification; cross-check against key (F22) |
    | inode_reuse_gen | 8 | F18: named for its purpose, NOT F5's birth generation |
    | locality_id | 8 | **F14** -- without it a renamed file's data is unreachable |
    | change_counter | 8 | **F17** monotone, incremented on every metadata change |
    | size | 8 | logical size |
    | blocks | 8 | **F16** occupancy; cannot be derived from size in this design |
    | rdev | 8 | **F19** device nodes |
    | flags | 8 | includes the attribute-presence bit from (a) |
    | mode / uid / gid / nlink | 4 each = 16 | POSIX core |
    | atime/mtime/ctime/btime seconds | 8 each = 32 | signed, Unix epoch; clears 2038 |
    | atime/mtime/ctime/btime nanos | 4 each = 16 | unsigned 0..999999999 |
    | reserved | 32 | F10(b) generosity; two open dependents (F9, attribute keyspace) |
    **Total 160 bytes.** Seconds and nanos are grouped separately so every field is
    naturally aligned (round 1's interleaved 12-byte timestamps misaligned two fields).

(c) **No extent pointers in the record** (unchanged from round 1, conceded by both cloud
    legs): data is reached through the data keyspace (F1); F3 forecloses inlining; F4's
    "one more parser + checker + crash-point replay forever" applies to a second
    representation of the same relation.

(d) **inode tree key** (F20): `(inode)` alone -- the inode record is located by object id
    only. `locality_id` deliberately does NOT prefix this key, because the record is what
    TELLS you the locality; putting it in the key would recreate F14's circularity.

(e) **v1 defaults** (F15, F21): `locality_id = 0` is reserved to mean "no parent / root
    locality" and is what the first transaction writes; `nlink = 1` with the invariant
    stated as "nlink equals dirent count, and an object with no dirent tree yet is exempt
    until the dirent tree exists" -- the exemption is written into the invariant, not left
    to the orphan-reclaim pass to guess.

(f) **Authority rule** (F22): the KEY is authoritative for identity; the record's copy is a
    cross-check only. On mismatch the checker judges red and the kernel path returns EIO
    for that object; it must never silently repair, because a silent repair would let key
    corruption propagate into the record and destroy the cross-check.

(g) **Not in the record**: block birth txg (lives in the pointer, F8), xattrs / ACLs
    (attribute keyspace per (a)), the snapshot head discriminator (F9 is unsettled; the
    record deliberately does not encode it, and the freeze of this item is declared
    GATED on F9 being resolved at least to "out of v1, N bits reserved in key").

## QUESTION

Does F23 survive? Attack or derive per your stance, with attention to:
 (a) Does storing `locality_id` in the record (F14) actually close the rename and rebuild
     holes, or does it create a NEW one -- e.g. can the record's copy and the key's copy
     disagree, and what happens then given (f)?
 (b) Is `(inode)` alone the right key for the inode tree (d), given F9 may need snapshot
     bits and given scan locality arguments that motivated `locality_id` for extents?
 (c) Does the attribute-keyspace overflow route (a) actually work as an extension
     mechanism before it is designed, or is declaring the presence-bit semantics now
     writing a cheque that cannot be cashed?
 (d) Do the v1 defaults (e) survive -- especially `nlink = 1` with an exemption written
     into the invariant: does that weaken the invariant into something unfalsifiable?
 (e) Is `change_counter` (F17) sufficient to order two versions found by scan, or does it
     need a tie-break rule of its own? What increments it, and what happens on wrap?
 (f) Anything still missing that the FIRST TRANSACTION needs.
