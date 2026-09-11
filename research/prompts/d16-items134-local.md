You are the counterexample leg of a three-way design review for a from-scratch
copy-on-write filesystem. Your only job is to construct concrete counterexamples
against three proposals. Do not argue which option is better. Do not summarize.
Find sequences of operations, images, or rules that break the proposals.

Do not use any markdown emphasis in your answer. No bold, no italics, no asterisks.
Write plain sentences and plain numbered lists. Answer in English.

FACTS YOU CAN RELY ON

Publication and the root ring
F1. Every publication increments a counter called checkpoint_txg by exactly one.
    An fsync-triggered publication counts as a publication. The same counter is
    the root ring rotation key (region = txg mod R), the accounting generation,
    and the journal replay watermark.
F2. The root ring has R = 3 regions and S slots per region. S is a value stored in
    the superblock, between 1 and 16. Ring depth N = 3 * S. Each publication
    writes one root into the next slot, so the ring holds the roots of the last N
    publications. mkfs seeds generation 0 into every slot.
F3. Publish order: new COW units, then a barrier, then the journal record, then a
    barrier, then the root slot written with FUA. fsync returns after the root slot
    is durable.
F4. Recovery: verify every ring slot first, then pick the newest valid root (ties
    broken by the higher instance id). Replay journal records whose
    (instance id, checkpoint_txg) is strictly greater than the chosen root's.
    Every named unit is checksum-verified before its record is applied. On a two
    disk pool, losing disk 0 can force recovery to fall back by 2 generations.
F5. Administrator rollback (a user decision, already settled): the administrator
    may pick any root in the ring that the instance table still considers valid.
    Rollback depth is at most the ring depth. The defer queue, the allocator
    cursor and all accounting values are reloaded from the chosen old root. A new
    instance id is taken and a rollback row is written. The first new root gets
    checkpoint_txg = (maximum txg of all roots in the ring) + 1.

Freeing and reuse
F6. A block freed in checkpoint C must not become allocatable before C is
    published. The free generation f of a block is the checkpoint_txg of the
    publication in which the free takes effect. "Freed" means no root, including
    snapshots, references it any more. Blocks still referenced by a snapshot go to
    a deadlist, not to the defer queue, and are freed only when the snapshot is
    destroyed.
F7. Allocation records are a COW tree under the root. A free rewrites the entry to
    "freed + free generation"; the entry is not deleted. After a crash the
    in-memory defer queue is lost.
F8. Sweeping (overwriting the header of a free unit) is admitted only if the unit
    is freed and f <= checkpoint_txg of the oldest root currently in the ring.
F9. Accounting keeps K = N + 1 generations of every statistic.
F10. Admission control: available = capacity - allocated - unreclaimable -
    defer-pending - pending deletes - committed reservations. Freeing space must
    never itself require allocating space.
F11. Today an invariant says: blocks referenced by the last K valid roots are not
    reallocated, K <= ring depth, K is a runtime policy with lower bound 2.

Freeze policy
F12. Freezing a format component means committing a per-component frozen spec
    file; the gate switches to stricter checks when a commit touches that
    directory. There are four layers in dependency order: layer 1 superblock and
    block header, layer 2 pointer field layout, layer 3 key encodings, layer 4
    index node layout. Two independent components already exist outside the
    layers (the packed inode record format, and the statistic registry). The
    freeze precondition is evaluated per open decision item, not per layer.

Journal
F13. The journal is a fixed-size ring. Record size is 4 KiB and is stored in the
    superblock. The tail pointer lives in a superblock slot. Ring size is not yet
    decided. Record header: magic 4, version and type 2, algorithm 1, padding 1,
    length 4, named item count 4, jsn 10, checkpoint_txg 8, nonce 12, header
    checksum 32, total 78 bytes. Three already decided additions are not yet in
    the 78: transaction id 8 plus commit flag 1, a 4 byte CRC32C back chain over
    the previous record header, and a 4 byte CRC32C payload checksum. Each named
    item is 56 bytes. Only the 56 total is decided; the tentative composition is
    two 14 byte position entries (a pointer layer field, layer 2), tree id 8,
    birth generation 8, unit type tag 1, flags 1, reserved 2, unit size 4,
    payload CRC 4.
F14. A journal record carries pointer-layer target state: which subtree roots
    moved to which blocks, plus the new root. It does not carry logical intent.
    Replay needs no allocator.
F15. A replayed prefix must satisfy five rules: jsn strictly consecutive (stop at
    the first gap); (instance id, checkpoint_txg) greater than the chosen root's;
    only transactions whose commit flag is present are applied; every named unit
    is verified before apply; if the chosen root's instance has a rollback row in
    the instance table, that instance's records are applied only up to the row's
    transaction number W.

Verification
F16. The checker judges one image only. The implementation and the checker share
    only a generated constants module. The two checker directions share only
    format parsing. A reference filesystem (RefFS) handles persistence and crash
    semantics itself and must not share crash point enumeration code with the
    implementation.

THE THREE PROPOSALS

Proposal 1 (defer window). The unit is publications, not seconds. A freed block
may be reallocated if and only if it is freed and its free generation f is less
than or equal to the checkpoint_txg of the oldest root currently in the ring.
This is the same predicate as sweeping in F8. In steady state the window is
exactly N publications. The runtime reuse delay K becomes identical to the ring
depth; the only tuning knob left is S.

Proposal 3 (journal freeze layer). The journal is not placed in the four layer
chain. The journal record format (header with the three additions, named items,
payload checksum, back chain, transaction boundary field) becomes a third
independent freeze component. The journal geometry fields that live in the
superblock (record size, ring size, tail slot) freeze with layer 1. Because each
named item embeds layer 2 position entries, the journal component may freeze no
earlier than layer 2 (same freeze request or later). Its freeze moment is the
commit that adds its frozen spec file, not the first image that writes a record.

Proposal 4 (record checker). A separate record checker takes (pre-crash image,
record stream, post-crash image). It independently implements: record parsing
(header, items, payload checksum, back chain), the five prefix rules, root
selection (verify all candidates, then newest, tie by instance id), and a purely
syntactic application of the named pointer-layer target states onto the chosen
root to obtain the expected post-recovery pointer layer, which it compares with
the implementation's post-recovery image. It does not implement tree semantics,
accounting, or allocation. It shares only generated constants with the
implementation, may share format parsing with the checker, and shares no recovery
code with RefFS. In the collusion test it is a fourth judge; each judge must have
at least one planted bad implementation that only it catches.

WHAT TO PRODUCE

For each proposal, try to construct concrete counterexamples. Number them.
For Proposal 1: a sequence of frees, publications, crashes, disk losses or
administrator rollbacks after which a block is reallocated while some root that
recovery or rollback may still choose references it; or a state where the window
makes a publication impossible for lack of space; or a consumer that needs a root
older than the oldest root in the ring.
For Proposal 3: a journal field whose width or meaning is set by layer 3 or layer
4; a way a crash image written by an older version defeats the component; a freeze
order that the proposal allows but that breaks something.
For Proposal 4: one of the five prefix rules or root selection that needs
information not present in the three inputs, or needs tree semantics, accounting
or allocation; or a bad implementation that all four judges would pass.
If you cannot find a counterexample for a proposal, say so plainly for that
proposal and list what you tried.
