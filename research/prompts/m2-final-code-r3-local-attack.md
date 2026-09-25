SINGLEFS INDEX TREE GEOMETRY ARITHMETIC - LOCAL ATTACK LEG - ROUND m2-final-code-r3

You are one of several independent reviewers examining freshly written code for
singlefs, a copy on write filesystem being designed and implemented from
scratch. Your only job in this document is arithmetic: given a fixed set of
numbered facts below, taken directly from the project's own source code and
decision records, work out five numbered items, each asking you to compute one
or more specific numbers by a specific, fully stated procedure. This is not a
design review and you are not being asked whether any of these numbers are
good choices; you are only being asked to compute them correctly from the
facts given, and to say plainly if any two facts, or any fact and any item's
stated procedure, do not fit together.

Do not assume anything not stated in the facts below. Read every fact
carefully; every qualifier is load bearing. Do not use any markdown emphasis
anywhere in your answer: no bold text, no italic text, no backtick code
formatting, no asterisk bullets, no pipe tables. Number your answers to match
the five items given at the end, and within each answer, number your
sub-answers to match the sub-parts asked for in that item. For every number
you report, show the arithmetic step that produced it (the specific
subtraction, division, rounding, exponentiation, or summation), not just the
final value. For every numbered item, end with one sentence starting with
"This would be refuted by:" describing the specific arithmetic mistake or the
specific fact you would have had to misread for your answer to that item to be
wrong.

When your answer needs to point at where a number comes from, point at it only
by the fact number given below (fact 1 through fact 13), or, if you need to
name a specific piece of source code, name it only by the name of the constant
or function itself (for example ALLOCATION_RECORD_BYTES, or
lower_root_level_for). Do not write any source file name together with a line
number, and do not write any line number at all anywhere in your answer,
because the underlying files may have moved to different line numbers by the
time anyone reads your answer.

Two vocabulary notes before the facts. First, this project calls its on disk
placement granularity a "slot"; every slot is the same fixed number of bytes,
stated in fact 1 below. Second, this project stores several of its internal
lookup structures as B-tree-like structures called "index trees" here; two
specific index trees are named throughout the facts below: the "allocation
record tree", which records, keyed by absolute on-disk slot number, which
slots on which physical devices are currently allocated; and the "extent
tree", which maps a file's byte ranges to the physical data units holding
those bytes, and which is itself split into two segments, an "upper segment"
keyed by inode number and a "lower segment" (present only for files spanning
more than one data unit) keyed by data unit number within one file.

Fact 1. Every index tree node in this project is a fixed size on disk: 16384
bytes, regardless of which index tree the node belongs to. Separately, the
project's placement granularity, the "slot" defined above, is also fixed at
16384 bytes; every index tree node occupies exactly one slot. These are two
separately named quantities in the source code that happen to carry the same
numeric value.

Fact 2. Every node belonging to the allocation record tree carries a fixed
on-disk header before its entry area begins. That header's total width,
including a trailing 29 byte area reserved for a cryptographic nonce, a
message authentication code, and an algorithm-type byte, is 135 bytes. This
135 byte figure is the same for every node in this tree, leaf or internal.

Fact 3. Every node belonging to the extent tree, in either segment, carries a
fixed on-disk header before its entry area begins. That header's total width,
including the same kind of trailing 29 byte reserved area described in fact 2,
is 163 bytes. This 163 byte figure is the same for every node in this tree,
in either segment, leaf or internal.

Fact 4. One entry inside a leaf node of the allocation record tree occupies
20 bytes on disk.

Fact 5. One entry inside an internal, non-leaf node of the allocation record
tree occupies 96 bytes on disk.

Fact 6. One entry inside a leaf node of the extent tree's upper segment
occupies 113 bytes on disk.

Fact 7. One entry inside a leaf node of the extent tree's lower segment
occupies 112 bytes on disk.

Fact 8. One entry inside an internal, non-leaf node of the extent tree
occupies 110 bytes on disk; this one figure is the same for internal nodes in
both the upper segment and the lower segment.

Fact 9. Every physical device (disk) in the pool reserves its lowest
numbered slots, from slot number 0 up to but not including slot number 50176,
for fixed, pool wide structures unrelated to the two index trees above. The
region of a device starting at slot number 50176 and running to the end of
the device is called that device's "unit area"; it is the only region where
allocation record tree leaves and extent tree data unit pointers actually
live. Separately, the project defines a small helper computation, called here
"unit area slot count", whose entire procedure is: take the device's total
size in bytes, divide by 16384 (the slot size from fact 1), then subtract
50176 from that quotient. The result is that device's unit area slot count.

Fact 10. The project's own test suite directly asserts, for a device whose
total size is exactly 4 gibibytes (4 times 2 to the 30th power bytes, that is
4294967296 bytes), that this device's unit area slot count, computed exactly
as fact 9 describes, equals 211968.

Fact 11. The allocation record tree's own geometry-building procedure, the
one that decides how many layers the tree needs, takes as its input, for each
device in the pool, a single number that its own source comment describes as
"each device's absolute slot count, computed as the device's byte size
divided by 16384" - that is, fact 1's slot size applied to the device's whole
size, with no subtraction of fact 9's 50176 boundary at all. Separately, the
one place in the source code that feeds this geometry-building procedure from
the live, already-running allocator's state builds that same per-device
number by taking fact 9's unit area slot count for that device and then
adding fact 9's 50176 boundary back onto it. Adding that boundary back on
exactly cancels the subtraction that fact 9's own helper procedure had just
performed, so the number that this live call site actually passes onward is
the device's full, undiminished total slot count (byte size divided by
16384), not the smaller unit-area-only slot count from facts 9 and 10.

Fact 12. Two of the project's own tests build a hypothetical two-device pool
by computing each device's slot count directly as that device's byte size
divided by 16384, with no subtraction of fact 9's boundary at all, matching
the undiminished total slot count described in fact 11, not the smaller
unit-area slot count from facts 9 and 10. One of these two tests uses two
devices of 4 gibibytes each; the other uses two devices of 1 tebibyte each (1
tebibyte being 2 to the 40th power bytes, that is 1099511627776 bytes). Both
tests then feed those two devices' slot counts into the same geometry-building
procedure named in fact 11.

Fact 13. The extent tree's lower segment is layered as follows. For a
nonnegative whole number L, define the "lower-segment span at level L" as: at
level 0, the lower-segment leaf capacity (the whole number of data units one
lower-segment leaf node can hold, which you will compute in item 4 below);
at any level L greater than 0, the span at level L minus 1, multiplied once
more by the extent tree's internal fanout (the whole number of children one
internal node can point to, which you will also compute in item 4 below).
For a file holding some whole number of data units, define the "lower-segment
root level" as the smallest such L, starting the search at L equal to 0, at
which the span at level L is greater than or equal to that whole number of
data units. Define the lower segment's "height" as that root level plus 1. A
height of 0 is reserved separately for files with no lower segment at all
(such a file's single data unit's pointer sits directly in the upper segment
instead), and does not arise from this level-search procedure.

Item 1. Compute W, the whole number of allocation record tree leaf entries
that fit inside one leaf node's entry area. The procedure: take the node size
in bytes (fact 1), subtract the allocation record tree's header width (fact
2), divide the result by the allocation record tree's leaf entry width (fact
4), and round down to the nearest whole number. Report the subtraction, the
division as an exact fraction, and the rounded down whole number result.

Item 2. Compute the allocation record tree's internal fanout, the whole
number of children one internal node can point to. The procedure is the same
as item 1 except that you divide by the allocation record tree's internal
entry width (fact 5) instead of its leaf entry width. Report the subtraction,
the division as an exact fraction, and the rounded down whole number result.
State explicitly whether this result equals the number 169 that appears
literally inside item 3's formula below; if it does not, say so as a
discrepancy worth flagging rather than silently using whichever number you
prefer.

Item 3. This item has two parts, a preparation part and a main part.

Preparation part of item 3. For each of the three device configurations
listed below, compute two different slot counts per device, following facts 9
through 12 exactly: (i) the "total slot count" way, dividing the device's
byte size by 16384 with no subtraction of fact 9's 50176 boundary, the way
facts 11 and 12 describe the tree's own geometry procedure actually consuming
its input; and (ii) the "unit area slot count" way, using fact 9's helper
procedure exactly (dividing by 16384 and then subtracting 50176), the way
facts 9 and 10 define it. The three device configurations: configuration A,
a single hypothetical device whose slot count is stipulated directly as 240
for both the total and the unit-area way alike (this number is given to you
directly here; it is not something you derive from a byte size, and you
should not attempt to reverse it into a byte size); configuration B, two
devices of 4 gibibytes each; configuration C, two devices of 1 tebibyte each.
For configuration A, report 240 for both ways as instructed. For
configurations B and C, show the byte size, the division by 16384, and, for
the unit-area way only, the subtraction of 50176, for one device; then state
whether both devices in that configuration share the same slot count (they
do, since both devices in each of configurations B and C are stipulated to be
the same size).

Main part of item 3. The allocation record tree's root layer is chosen by the
following rule, stated here exactly as it appears in the project's own
decision record: the root layer is the smallest whole number R, with R
greater than or equal to 1, such that the sum, taken over every device in the
pool, of the ceiling (round up to the nearest whole number) of that device's
slot count divided by the quantity W times 169 raised to the power of (R
minus 1), is less than or equal to 169. Here W is the value you computed in
item 1, and the two appearances of the number 169 are both given literally as
part of this rule, independent of whatever number you computed in item 2.
Apply this rule to each of the three device configurations from the
preparation part above, and, for configurations B and C, apply it twice: once
using each device's total slot count, and once using each device's unit-area
slot count. For every one of these five applications of the rule (one for
configuration A, two for configuration B, two for configuration C), show, for
increasing values of R starting at R equal to 1, the value of W times 169
raised to the power of (R minus 1), the per-device ceiling division, the sum
over all devices in that configuration, and whether that sum is less than or
equal to 169; stop and report R as soon as you find the smallest R for which
it is. State explicitly, for configuration B and separately for configuration
C, whether the total-slot-count way and the unit-area-slot-count way produce
the same value of R or two different values of R; if they differ, say by how
much.

Item 4. Compute three separate whole numbers for the extent tree, each by the
same kind of procedure as items 1 and 2: subtract the extent tree's header
width (fact 3) from the node size (fact 1), then divide by the relevant entry
width and round down. Compute: (a) the upper-segment leaf capacity, dividing
by the upper-segment leaf entry width (fact 6); (b) the lower-segment leaf
capacity, dividing by the lower-segment leaf entry width (fact 7); (c) the
internal fanout shared by both segments, dividing by the internal entry width
(fact 8). Report, for each of the three, the subtraction, the division as an
exact fraction, and the rounded down whole number result.

Item 5. Using your own answers to item 4 for the lower-segment leaf capacity
and the internal fanout (do not use any other numbers for these two
quantities), and using fact 13's level-search procedure exactly as stated,
determine the lower-segment root level and the lower-segment height, as fact
13 defines those two terms, for a file holding exactly 145 data units. Show
the span at level 0, the span at level 1, and, only if needed, the span at
level 2 and beyond, until you reach a span that is greater than or equal to
145; report which level that is (the root level) and the root level plus 1
(the height).
