# Background: singlefs D1 settled items 5 and 6, and a proposed unification

All FACTS below were re-read verbatim from the repository on 2026-09-06 by the asking
agent. Quotes marked "verbatim" are translations of the exact Chinese sentence in the
cited file. The PROPOSAL section is NOT verified and is the thing under argument.

singlefs is a from-scratch COW filesystem, still in format design. No code, no disk
format yet. A "unit" is the atomic self-describing storage object. Units are immutable:
changing one byte means writing a new unit.

## FACTS

F1. D1 settled item 3 (2026-09-06, user decision): every unit reserves one customizable
    extension point. Its semantics are two-way: either empty, or exactly one pointer.
    No inline data in the extension point.

F2. D1 settled item 5 (2026-09-06, user decision): a unit's content splits into two
    classes, each carrying exactly one pointer.
    Class A, self-contained immutable data, lives in the self-contained segment; its
    pointer answers "who I belong to".
    Class B, mutable extension data, lives in the extension point; its pointer answers
    "where my previous version is".
    Writing a new version detaches the old one, and the detaching is done by the tree
    changing its pointer, never by modifying the old unit (units are immutable).
    Verbatim: "that pointer does not constitute a reference". Reclaim admission does not
    look at whether some other unit points at a slot. Staleness is detected by carrying
    the target's birth generation in the pointer and comparing on read.

F3. D1 settled item 6 (2026-09-06, user decision): the extension point holds a stack of
    depth 5, the locations of the most recent 5 versions. Writing a new version truncates
    the previous stack to 4 entries, shifts it forward, and pushes the previous version's
    own location into slot 1. Expiry is released in batch by gc. The filesystem does not
    implicitly retain old versions; backup is not the filesystem's job.

F4. D22 settled item 2 (2026-08-31, user decision): the root ring has R = 3 regions
    (R = F + 1 with F = 2). R is a format quantity.

F5. Invariant I-7.4, verbatim: "blocks referenced by the most recent K valid roots have
    their physical ranges neither reallocated to another object nor scrubbed of their
    headers (K is less than or equal to the root ring depth; the depth is a format
    quantity and is the upper bound of K; K itself is a runtime policy with a lower bound
    of 2)". The same entry states that D22 explicitly forbids K from being a format
    constant. It also states: after a crash the defer queue's in-memory state is lost, and
    runtime K is taken over by the root ring depth through the reclaim gate "release
    generation less than or equal to the oldest root txg in the ring".

F6. D3 settled item 7, reclaim admission, verbatim: for a slot that has an allocation
    record entry, admission requires "already released AND release generation less than
    or equal to the checkpoint_txg of the oldest root in the ring". The definition of
    "released" is the moment the allocator puts the slot into the defer queue, which is
    the moment no root at all, snapshots included, references it. Slots referenced by a
    snapshot go to the deadlist rather than the defer queue and are released only when
    that snapshot is destroyed.

F7. D18, verbatim: "any judgment of 'this is the previous version' requires an external
    witness. This is definitional, not an engineering limitation." It reaches this by
    ruling out three paths: a wall-clock timestamp (an old block legitimately carries an
    old timestamp), an incrementing local counter (the expected count is external
    information), and a version-history hash chain (the expected chain length is external
    information). D21 names the witness: "the checkpoint's root is that witness."

F8. D21 constraints on the extension point. It is space reserved for a third party who
    opens a new line of the format, not for this project's own fields.
    Hard constraint 2: the checker must make no judgment whatsoever about its content.
    Hard constraint 4: any target space must come from an explicit quota that goes
    through this project's allocation and accounting.
    Hard constraint 5, self-annotated as the hardest: pointers inside the extension point
    must not point at core-layer objects, only into that extension point's own quota;
    "the extension point is a closed subgraph, not a general reference".
    Lifetime is bound to the host unit: host unit dies, its extension point quota is
    freed. The size is declared per line in the superblock and may be declared 0.
    D21 settled item 4: data units carry an extension point, index nodes carry one
    conditionally, self-attesting units do not carry one.

F9. Byte costs. A pointer is 7 bytes (device 1 + physical offset 6). A birth generation
    is 8 bytes. So a stack entry that carries pointer plus birth generation is 15 bytes.
    Depth 5 costs 75 bytes per unit; depth 3 costs 45 bytes. A data unit header is
    105 bytes. A data unit is 32768 bytes.

F10. Two user decisions given today, 2026-09-06, translated verbatim:
    (a) "Regardless of whether the outside world references it, once its storage time is
        up it should be released. We cannot allow a referenced value to keep occupying
        this space. This place was never for storing data; it is a short-term switchover
        area for avoiding loss."
    (b) "R = 3 and 5 generations conflict. This needs to be unified. Also orphan indexes
        must absolutely be avoided."

F11. Known open items that touch this area. D21 open item 1: the form of the reverse
    index for the multiple-referencer half is still undecided. D26 open item 5: where
    location authority lives is still undecided. C34 is the corresponding owed check.

## PROPOSAL under argument (NOT verified, this is what you must attack or support)

P1. Make the stack depth identical to the root ring depth R, registered as one single
    format constant. Today that is 3, not 5. Two separate numbers are forbidden; writing
    different values in two places must fail the gate.

P2. Expiry is unconditional. Reclaim admission stays exactly the predicate in F6. The
    stack pointer neither pins its target nor has its own life extended by any external
    reference to that target. Stated boundary: old versions referenced by a snapshot are
    held by D5's deadlist, which is a different mechanism holding them, not this chain
    extending anything.

P3. Three gates against orphan entries, in this order.
    Gate 1, the window gate: before dereferencing a stack entry, ask whether it is inside
    the CURRENT runtime K window. If it is not, treat the entry as absent and do not
    dereference it at all. Note K is a runtime policy bounded above by R, so the question
    is not "is it within the stack depth" but "is it within the current K".
    Gate 2, birth generation comparison, as the second gate, catching the case where the
    slot was reclaimed and reused.
    Gate 3, depth and R are the same registered format constant, so the two cannot drift.

P4. Claimed by-product: once depth is bound to the root ring, the root ring itself is the
    external witness that F7 says is definitionally required, so the judgment "this is the
    previous version" is no longer self-attested from inside the unit.
