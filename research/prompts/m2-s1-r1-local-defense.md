You are one of several independent reviewers examining a specific engineering
design question for a copy-on-write filesystem project written in Rust. The
project is still in the format-design stage; no on-disk format is frozen for
the parts discussed below. Your stance is DEFENSE. You are defending an
already-adopted design clause, called candidate JIA, against a rival
candidate called WAL_FULL, which claims a much lower per-fsync write cost.
Use only the facts and tables given below; do not invent facts, and do not
import numbers from outside this prompt. Answer in English. Do not use any
markdown emphasis such as bold or italic, and do not use markdown headers.
You may use plain pipe tables. In your answer, do not cite any file or code
line numbers; refer only to the fact numbers below (F1, F2, ...) and to the
table row labels given (for example: Table A row B, Table B row D N=16).
Every numeric answer must show the formula you used and name exactly which
fact number or table row supplied each number in that formula; a bare number
with no formula and no citation does not count as an answer. End every
numbered part of your answer with a line starting "Refuted if:" naming the
concrete observation that would refute that part's conclusion.

Facts, do not dispute them, cite them by number:

F1. The currently adopted clause states: pick candidate JIA. On every fsync
call, the filesystem writes every dirty leaf node (not only the leaves
touched by the file that called fsync -- one fsync is defined as one full
publish covering the whole pool's currently dirty state), every ancestor of
those dirty leaves up to the root, the root slot, and one journal record.
Ancestors are never deferred to a later point in time.

F2. In full, candidate JIA's per-fsync write set is: the d data units this
fsync touches; every dirty node from leaf to root in both the extent tree
and the inode tree; every dirty node in four other tree structures called
the fixed points (an accounting tree, an allocation-record tree, a central
mapping tree, and a tree-of-trees index that records which physical block
currently holds the root of each of the other trees); one or more journal
records, one for every 67 objects named (each written unit gets one named
entry); one write to the root slot (on one physical device, force-unit-
access); and one write to the superblock slot on each physical device in
the pool.

F3. The rival candidate WAL_FULL is written in the strongest form its own
supporters would recognize. On every fsync, it writes: the d data units
touched; every dirty node from leaf to root in the extent and inode trees
(using the same formula as JIA for this part); and one or more small
journal records naming only the new subtree root(s) touched by this fsync.
It does not write the root slot, does not write the superblock slot, and
does not touch the tree-of-trees index or the other three fixed-point trees
(accounting, allocation-record, mapping) on every fsync. Instead, every N
fsyncs, it performs one checkpoint: a JIA-shaped publish that writes only
the dirty nodes of the three fixed-point trees and the tree-of-trees index
(computed as the net change accumulated over that whole N-fsync window, not
fsync-by-fsync), plus journal records for what it wrote, plus one root-slot
write, plus the superblock write on each device.

F4. Nothing above forbids one additional technique for JIA, without
changing a single word of the adopted clause: group commit. The clause only
requires that by the time any one fsync() call returns to its caller, that
call's own new data is durably reachable from a published root; it does not
require a separate publish operation for every individual fsync() call. So
if k fsync() calls become ready to commit at close to the same time, JIA
may perform one single publish that covers the union of all k calls' dirty
state (all k calls' data units, all their ancestors, all four fixed points,
one shared set of journal records, one root-slot write, and one write to
each device's superblock slot), and let all k callers' fsync() calls return
together once that one publish completes. This is the same technique used
by Linux's own jbd2 journal, whose own source comment says group commit
"Speeds up many-threaded, many-dir operations by 30x or more". (This
project's own prior background material quotes that jbd2 source comment;
that quote is from an earlier, separate round of this project's own
argumentation, not from the current round's decision record.)

F5. A directly on-point, already-recorded finding distinguishes two kinds
of workload. When the k operations gathered into one batch share a single
spine (a spine means the leaf-to-root ancestor path in one of the user-data
trees, extent or inode), group commit CAN help, for the same reason
enlarging the batch size can help. When the k operations' spines do not
intersect at all, group commit CANNOT help there: what group commit saves
is the root-slot write and the journal record, because there is only one
root; the number of distinct spines that must be written does not go down.
Note: this finding was written before the four fixed-point trees existed as
a separately modeled cost in this project's own accounting; it does not by
itself say whether the same "does not go down" conclusion applies to the
four fixed-point trees.

F6. Two of the four fixed-point trees do not scale with pool size and are
always rewritten in full on every single publish, regardless of which
entries actually changed. The accounting tree has exactly 3+6xD entries in
total (3 pool-level entries plus 6 per physical device; D is the number of
physical devices, 2 in this pool, so 15 entries total), and this count
never grows with the number of files or snapshots in the pool; on every
publish, all of its entries are rewritten, so it fits in a single small
node.

F7. The tree-of-trees index holds one entry for each content tree in the
pool; whichever of those entries correspond to a tree that had its own root
change in this publish (extent, inode, allocation-record, accounting) are
treated as one single contiguous group of at most 4 entries in this
publish. Its total entry count is fixed and small, independent of pool
size, so it fits in a single small node regardless of pool size.

F8. The other two fixed-point trees, the central mapping tree and the
allocation-record tree, do scale with pool size: their entry count grows
with the total number of live storage units in the pool. Their own internal
node structure behaves exactly like the extent and inode trees: whether
merging k concurrently-arriving fsyncs' worth of insertions and deletions
into one combined write reduces the total number of distinct nodes written
depends on whether the k fsyncs' touched positions in that tree's own key
ordering are adjacent (so they share ancestor nodes) or scattered across
the tree (so they do not), using the same kind of arithmetic that applies
to the extent and inode trees.

F9. A measured, already-verified identity holds for the cell family=F1,
P=1 (see Table A row A and Table B row A N=1 below): (JIA's own per-fsync
bytes) minus (WAL_FULL's own per-fsync bytes) equals exactly the bytes for
the four fixed-point trees' own dirty-node writes, each mirrored onto both
of the pool's 2 physical devices, plus the one root-slot write, plus the
two superblock-slot writes (one per device); at this cell that difference
is exactly 139776 bytes.

F10. The decision record that adopted "publish a root on every fsync" over
the deferred alternatives lists exactly four supporting reasons, by name
only: (1) without this, the journal ring cannot be truncated; (2) without
this, one more mount-time replay code path is needed; (3) without this,
the journal enters the verification chain that must be independently
checked; (4) choosing the deferred alternative does not save even a single
flush barrier.

F11. An elaboration of the same four reasons is found in this project's own
background material from an earlier, separate round of argumentation (not
the current round's own decision record, and not independently re-verified
here beyond this one citation): "axis one is settled: publish a root on
every fsync. Reason: not publishing a root on every fsync means the ring
cannot be truncated (measured ring peak occupancy was 48 bytes under
publish-every-fsync, versus 10^4 to 10^5 bytes under the alternative); it
also requires one more mount-time replay code path; and it puts the journal
into the verification chain. Meanwhile the number of flush barriers is 2 on
each side -- choosing the alternative does not save even one."

Table A. JIA's own measured per-fsync numbers, no grouping (k=1), main
geometry, sequential placement, all bytes are for both physical devices
combined:

Cell | Family | Pool size P | JIA fsync bytes | JIA write calls | JIA
barriers | JIA force-unit-access writes
A | F1 | 1 | 344576 | 21 | 4 | 1
B | F1 | 10000 | 740712 | 45 | 4 | 1
C | F8A | 1 | 803328 | 35 | 4 | 1
D | F8A | 10000 | 1400224 | 71 | 4 | 1

Table B. WAL_FULL's own measured numbers at checkpoint interval N,
sequential placement, same four cells. WAL_FULL fsync bytes is the constant
per-fsync-only part (does not depend on N); WAL_FULL checkpoint bytes is
the total bytes of one checkpoint at that N; WAL_FULL amortized bytes =
WAL_FULL fsync bytes + WAL_FULL checkpoint bytes / N. WAL_FULL was only
ever measured at N in {1, 16, 256, 4096}; there is no measured value at
N=2 or N=8.

Cell | N | WAL_FULL fsync bytes | WAL_FULL checkpoint bytes at this N |
WAL_FULL amortized bytes at this N
A | 1 | 204800 | 147968 | 352768
A | 16 | 204800 | 147968 | 214048
A | 256 | 204800 | 147968 | 205378
A | 4096 | 204800 | 147968 | 204836.125
B | 1 | 237568 | 511336 | 748904
B | 16 | 237568 | 514724 | 269738.25
B | 256 | 237568 | 568924 | 239790.359375
B | 4096 | 237568 | 1436140 | 237918.6201171875
C | 1 | 663552 | 147968 | 811520
C | 16 | 663552 | 147968 | 672800
C | 256 | 663552 | 147968 | 664130
C | 4096 | 663552 | 147968 | 663588.125
D | 1 | 730692 | 677724 | 1408416
D | 16 | 730692 | 704754 | 774739.125
D | 256 | 730692 | 1137208 | 735134.21875
D | 4096 | 730692 | 8081042 | 732664.9106445313

This round's own working table already uses each cell's N=16 WAL_FULL
amortized bytes value as the standing reference number for "WAL_FULL" at
that cell: A=214048, B=269738.25, C=672800, D=774739.125.

Table C. Unit widths and constants, main geometry.

Item | Value
Number of physical devices in the pool, D | 2
Width of a "code 2" index node (used for extent, inode, allocation-record,
accounting and mapping internal nodes, and most leaf nodes) | 16384 bytes
Width of a "code 3" container (used for the inode tree's leaf containers)
| 32768 bytes
Width of one journal record | 4096 bytes
Width of one superblock slot | 4096 bytes
Width of the root slot | equal to the pool's detected physical block size;
the main geometry used throughout Table A and Table B uses 512 bytes

Task.

For each of the four cells A, B, C, D in Table A, and for k = 1, 2, 8, 16
concurrently-pending fsync() calls merged by JIA into one group-commit
publish as defined in F4, estimate:

(1) group_total_bytes(k): the total bytes that one such group-commit
publish would write, combining all k calls' dirty state into one publish.
Start from JIA's own write set in F2, but for every component where
merging k calls can plausibly change the byte count, replace "one fsync's
worth" of that component with "k fsyncs' worth, merged" (see F5 through F9
for what is already known about which components merge for free and which
do not).

(2) amortized_bytes_per_fsync(k) = group_total_bytes(k) / k.

(3) ratio(k) = amortized_bytes_per_fsync(k) divided by that cell's standing
WAL_FULL reference value (the N=16 row of Table B for that cell, given
again just above Table C).

For k=1, group_total_bytes(1) must equal exactly that cell's Table A JIA
fsync bytes value; this is a check on your own arithmetic, not a new fact
to derive. Report your check.

Fill this table, one row per cell-and-k combination, 16 rows total:

Cell | k | group_total_bytes(k) | amortized_bytes_per_fsync(k) | ratio(k) |
ratio(k) <= 1.2 ?

For every row, show your formula and name exactly which fact number or
table row supplied each number you used in it. If some component's byte
count genuinely cannot be determined from the facts given -- not merely
estimated with lower confidence, but truly undetermined -- say so plainly
for that component and state what you assumed instead, rather than
inventing a number silently.

State plainly whether ratio(k) <= 1.2 holds for some single value of k in
all four cells A, B, C, D at the same time (JIA only wins this comparison
if one common k works for every cell at once); if so, name the smallest
such k.

Then, separately, place each of F10's four reasons -- (1) ring cannot be
truncated, (2) extra mount-time replay path, (3) journal enters
verification chain, (4) no barrier saved -- into exactly one of: HOLDS FOR
JIA-WITH-GROUP-COMMIT ONLY, HOLDS FOR WAL_FULL ONLY, HOLDS FOR BOTH, HOLDS
FOR NEITHER. Fill this table:

Reason | Where it holds | One sentence citing which fact or table row
(1) ring cannot be truncated | |
(2) extra mount-time replay path | |
(3) journal enters verification chain | |
(4) no barrier saved | |

End every numbered part of your answer with a line starting "Refuted if:"
naming the concrete observation that would refute that part's conclusion.
