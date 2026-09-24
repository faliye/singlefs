SINGLEFS TEST COVERAGE SCOPE - LOCAL ATTACK LEG - ROUND m2-lines234-code-r1

You are one of three independent reviewers checking a batch of code changes
to singlefs, a copy on write filesystem being designed from scratch. You are
given a self contained set of facts below about 22 specific source files.
Do not assume anything not stated here, and do not look at any other
document. Read every fact carefully, every qualifier is load bearing, a
definition with one word removed can flip an answer. Do not use any
markdown emphasis anywhere in your answer: no bold text, no italic text, no
backtick code formatting, no asterisk bullets, no pipe tables. Number your
answers as Row 1 through Row 22, one row per file, in the same order the
facts are given below. This is a separate, independent document from other
documents in the same review round that ask about other parts of the same
batch of changes; you are not expected to see those other documents and
should not assume anything about their content. Your own scope in this
round is only the test coverage of these 22 files; you are not asked to
judge the correctness of any behavior, and you should not try to.

For each of the 22 files you will be given four fixed facts, labeled fact a,
fact b, and fact c, plus one open question labeled question d. Facts a, b,
and c have already been established by direct inspection of the repository;
copy each one into your answer exactly as given, do not change it, do not
re-derive it, and do not second guess it. Only question d asks for your own
independent judgment.

Fact a for each file states whether any test names that file today, and if
so, the name of the test file. A file counts as named by a test today if
either an integration test file under a tests directory imports a path from
that file's own module, or the file itself contains its own internal
cfg(test) module testing its own contents; if only the second is true, the
name given for the test is the source file's own name, because in that case
the test lives inside the same file.

Fact b for each file states whether that file appears in the project's
mutation table, a checked in file named crates/mutations.tsv that records,
for regression purposes, a deliberate one line code change together with
the test that must fail when that change is applied. If the file appears in
that table, fact b also gives the exact row numbers, counted from the top
of that same checked in file, that name this source file in that row's
second column. These row numbers are fixed reference numbers taken from a
checked in ledger file, not line numbers inside the source file under
review; copy them exactly as given, and do not invent any additional line
number of any kind anywhere in your own answer, for any file, for any
reason. If you need to refer to a specific piece of code in your answer to
question d, refer to it only by the function name that contains it, never
by a line number.

Fact c for each file states, for each row of the mutation table that names
this file, whether the deliberate change described by that row alters the
program's behavior, meaning it changes which decision the code reaches or
which value it computes from real inputs, or whether that row only changes
the value of one already named constant without altering any surrounding
decision logic. Fact c also quotes, in English translation, the short
title the mutation table itself gives that row, describing in the project's
own words what the deliberate change does and why it matters. Where a file
has more than one row and the rows do not all agree on this behavior versus
constant distinction, fact c says so explicitly and gives the count on each
side; in that case the single label given for the file as a whole is
behavior whenever at least one row changes behavior, and is constant only
when every single row for that file only changes a named constant's value.
A file with zero rows in the mutation table has fact c stated as none.

Question d asks, for that same file: for a part of the code whose own
behavior cannot be derived from any already decided design item, does the
file contain a place where the code had to choose one specific behavior on
behalf of a requirement that was never written down anywhere, such that a
different, equally defensible implementation could have chosen
differently, and the two
choices can be told apart by looking at bytes on disk or at a reachable
sequence of operations. A place counts toward a yes answer only when you
can state two different choices that are each individually defensible, and
state how the two choices differ in disk bytes or in reachable history; if
you cannot tell the two choices apart in either of those ways, this
question does not apply to that place, and it should not count. If a row's
own quoted title in fact c already states, in the project's own words, that
some specific choice was made about a case with no written requirement,
you may cite that row's own words as your evidence for a yes answer, naming
the function the row's own original or replacement code snippet belongs to.
You have not been given the full source code of these 22 files. If the
facts given to you do not let you answer question d with a yes or a no you
can support, answer unknown; do not guess, and do not invent a function
name, a field name, or a mechanism you cannot point to inside the facts
given below. A yes answer must give a function name. Every answer to
question d, whether yes, no, or unknown, must end with one sentence
starting with "This would be refuted by:" describing the specific
observation that would prove that answer wrong, and one further sentence
naming which fact letter, a, b, or c, or which row's own quoted title, your
answer to d rests on; an answer to d that cannot name which fact or which
row's quoted title it rests on makes the whole row for that file void, and
you should say so instead of guessing.

At the end of your answer, after all 22 rows, add one final count: state
how many of your 22 answers to question d were unknown.

File 1. crates/singlefs-core/src/inode_tree.rs. This is a brand new file,
626 lines, added the same day as the batch under review. Its own stated
role is to compute, after this release, which leaf containers exist in the
tree, which records each one holds, and which ones need to be rewritten; it
issues no writes of its own and does not touch the allocator.
Fact a: yes, a test names it today. The integration test file
crates/singlefs-harness/tests/first_transaction_step_five_publish.rs
imports items from the inode_tree module of the singlefs_core crate directly. The file itself
also contains its own internal cfg(test) module.
Fact b: yes, it appears in crates/mutations.tsv, at rows 230, 231, 232,
233, 234, and 235, six rows total.
Fact c: behavior, all six rows change behavior. Row 230's own title:
parallel line three C116, the right half container produced by a split is
not given the inode number of the new record that triggered the split. Row
231's own title: parallel line three C116, split one record early, leaving
only 232 records in the left half, the split point is not taken at the
very end. Row 232's own title: parallel line three C116, a record whose
number already exists in the tree is not used to replace that existing
entry, and is instead appended as one more record. Row 233's own title:
parallel line three, a mid sequence insertion, a split that is not at the
very end, a case with no written requirement, is no longer rejected. Row
234's own title: parallel line three, records within the same write are
accepted even when they are not in ascending inode number order. Row 235's
own title: parallel line three, a leaf container count exceeding the 135
pieces that one code two root can hold, the tree needs to grow taller, a
case with no written requirement, is no longer rejected.

File 2. crates/singlefs-core/src/mounted_read.rs. This is a brand new
file, 993 lines, added the same day as the batch under review. Its own
stated role covers three functions: open_pool_for_read, mount_read_only,
and data_unit_span_covering.
Fact a: yes, a test names it today. The integration test file
crates/singlefs-harness/tests/second_transaction_parallel_line_two_mounted_read.rs
imports items from the mounted_read module of the singlefs_core crate directly. The file itself
also contains its own internal cfg(test) module.
Fact b: yes, it appears in crates/mutations.tsv, at rows 248, 249, 258,
259, 260, 261, and 290, seven rows total.
Fact c: behavior, all seven rows change behavior. Row 248's own title:
parallel line two, when a stale location hint falls back to looking up the
central mapping, the multi hop counter is not incremented. Row 249's own
title: parallel line two, the step where a mounted read verifies the whole
unit's checksum against the location entry is changed to always pass. Row
258's own title: parallel line two, the device level read count that the
read path reports about itself is not incremented. Row 259's own title:
parallel line two, the count of how many units were read this time is not
incremented. Row 260's own title: parallel line two, a single read only
dereferences the first unit, a page spanning multiple units loses its
second half. Row 261's own title: parallel line two, a central mapping
root that is an internal node is accepted anyway. Row 290's own title:
general survey R2, the mounted read path does not check the mapping
entry's width.

File 3. crates/singlefs-core/src/write_request_split.rs. This is a brand
new file, 247 lines, added the same day as the batch under review. Its own
stated role is to split one write request across however many data units
it spans.
Fact a: yes, a test names it today. The integration test file
crates/singlefs-harness/tests/parallel_line_one_sequential_write.rs
imports items from the write_request_split module of the singlefs_core crate directly. The file
itself also contains its own internal cfg(test) module.
Fact b: no, it does not appear in crates/mutations.tsv, zero rows.
Fact c: none, because fact b is no.

File 4. crates/singlefs-core/src/address.rs. This file was changed the
same day as the batch under review, with 13 lines added and none removed.
Fact a: yes, a test names it today. Dozens of integration test files
import items from the address module of the singlefs_core crate directly; one example is
crates/singlefs-harness/tests/first_transaction_region_bytes.rs. The file
itself also contains its own internal cfg(test) module.
Fact b: no, it does not appear in crates/mutations.tsv, zero rows.
Fact c: none, because fact b is no.

File 5. crates/singlefs-core/src/allocator.rs. This file was changed the
same day as the batch under review, with 114 lines added and 14 removed.
Fact a: yes, a test names it today. Many integration test files import a
items from the allocator module of the singlefs_core crate directly; one example is
crates/singlefs-harness/tests/first_transaction_step_five_publish.rs. The
file itself also contains its own internal cfg(test) module.
Fact b: yes, it appears in crates/mutations.tsv, at rows 8, 9, 24, 30, 38,
41, 49, 58, 86, 87, 88, 89, 90, 91, 92, 93, 94, 109, 121, 122, 129, 130,
140, 143, 144, 145, 169, 170, 171, 250, 251, 252, 268, 275, and 296,
thirty five rows total.
Fact c: behavior, all thirty five rows change behavior; none of them only
change a named constant's value. Their own titles, in the same row order
as fact b: free count is not decremented when allocating; the bitmap is
cleared at release time enabling immediate reuse; a step forgets to
rewrite the allocation record; released slots are not placed into the
defer queue when rebuilding the allocator; reclamation ignores the release
generation, forcing the reuse window to zero; reuse appends a new record
instead of rewriting the existing one, leaving two records for the same
slot; generation zero tree table units are not recognized when rebuilding
the allocator; the bump cursor for committing generated blocks does not
skip slots isolated or withheld after the segment opened; user data
placement only takes device 0's own answer; committing a generated block's
segment opening and fallback only takes device 0's own answer; devices are
not rejected even when they disagree on where to place a committed
generated block, with the branch that has no written requirement settled
as simply taking device 0's answer; devices are not rejected even when
they disagree on where user data should be placed; the open segment is
closed before a commit of a generated block is rejected; when the segment
for committing a generated block is exhausted there is no fallback, twice,
in two different circumstances; the fallback only looks at already
allocated bits and hands out a slot that shadow accounting had isolated,
twice, in two different circumstances; user data placement excludes only
the one currently open segment; an already reclaimed record that a new
record now covers is not deleted on reuse; no assertion fires when a new
placement covers a record that was never reclaimed; a random history fast
checkpoint catches the same reuse appending bug as the row for line 41; a
random history sampling point catches the same reclaimed record deletion
bug as the row for line 121; a random history fast checkpoint catches
reuse rewriting an already reclaimed record without updating its
allocation generation; a narrower form of that same bug, confined to the
write path taken during a mounted write and during warm up; the mirror
form of that same bug, where every allocation records its allocation
generation as one less than the current transaction generation; a random
history fast checkpoint also catches the same reclaimed record deletion
bug a third time; the allocator's user data placement reports no device
having an answer as a small device being full; the allocator's user data
placement reports a small device being full as no device having an
answer; the allocator's committing of generated blocks reports no device
having an answer as a small device being full; a branch name is reported
as the production path's own name so you cannot tell at run time which
branch actually ran; the reclamation lower bound falls back to zero and
not a single released placement is reclaimed; the delay window is not
bypassed so reuse can never be produced; the reader of allocation records
does not check the entry's width at one call site; the reader of
allocation records does not check the entry's width at a second, writable
mount side call site; the isolated bit does not block allocation.

File 6. crates/singlefs-core/src/block_device.rs. This file was changed
the same day as the batch under review, with 188 lines added and 2
removed.
Fact a: yes, a test names it today. Many integration test files import a
items from the block_device module of the singlefs_core crate directly; one example is
crates/singlefs-harness/tests/first_transaction_region_bytes.rs. The file
itself also contains its own internal cfg(test) module.
Fact b: yes, it appears in crates/mutations.tsv, at row 190, one row.
Fact c: behavior, one row, changing behavior. Row 190's own title:
supplement 3, item 3, the image file is not created exclusively; colliding
with an existing name simply continues using the old image.

File 7. crates/singlefs-core/src/lib.rs. This file was changed the same
day as the batch under review, with 3 lines added and none removed. It is
the crate's own root file, whose own content is only a list of module
declarations naming every other module in this crate; it defines no
function of its own.
Fact a: no, no test names this file today. Every integration test file
that touches this crate imports paths from the individual modules this
file declares, such as address or allocator, never from lib.rs directly by
name, and this file contains no internal cfg(test) module of its own.
Fact b: no, it does not appear in crates/mutations.tsv, zero rows.
Fact c: none, because fact b is no.

File 8. crates/singlefs-core/src/pointer.rs. This file was changed the
same day as the batch under review, with 28 lines added and none removed.
Fact a: yes, a test names it today. Several integration test files import
items from the pointer module of the singlefs_core crate directly; one example is
crates/singlefs-harness/tests/first_transaction_step_five_publish.rs. The
file itself also contains its own internal cfg(test) module.
Fact b: no, it does not appear in crates/mutations.tsv, zero rows.
Fact c: none, because fact b is no.

File 9. crates/singlefs-core/src/records.rs. This file was changed the
same day as the batch under review, with 114 lines added and 22 removed.
Fact a: yes, a test names it today. Many integration test files import a
items from the records module of the singlefs_core crate directly; one example is
crates/singlefs-harness/tests/first_transaction_step_five_publish.rs. The
file itself also contains its own internal cfg(test) module.
Fact b: yes, it appears in crates/mutations.tsv, at rows 241, 265, 266,
267, 286, 287, and 288, seven rows total.
Fact c: behavior, all seven rows change behavior. Row 241's own title:
parallel line three, the blocks field written back for an inode record is
the constant 64, so an inode that was just created and has never had data
written to it is also reported as occupying one unit. Row 265's own
title: the reader of extent leaf records does not check the entry's
width. Row 266's own title: the reader of inode internal entries does not
check the entry's width. Row 267's own title: the reader of accounting
entries does not check the entry's width. Row 286's own title: general
survey R2, the core layer's mapping entry reader drops the width check
from the field table, inside the unit test's own cell. Row 287's own
title: general survey R2, the same width check is dropped, inside the bad
disk input cell, where the central mapping tree root's entry width
shrinks down to just the key's own width. Row 288's own title: general
survey R2, the mapping entry's field table width check is turned into a
closed interval comparison, so a legitimate entry of exactly 55 bytes is
wrongly rejected.

File 10. crates/singlefs-core/src/root_ring.rs. This file was changed the
same day as the batch under review, with 13 lines added and 2 removed.
Fact a: yes, a test names it today. Several integration test files import
items from the root_ring module of the singlefs_core crate directly; one example is
crates/singlefs-harness/tests/first_transaction_region_bytes.rs. The file
itself also contains its own internal cfg(test) module.
Fact b: yes, it appears in crates/mutations.tsv, at row 284, one row.
Fact c: behavior, one row, changing behavior. Row 284's own title: one
region of the root ring is computed as having one fewer slot than its
real count, so the last root slot is never cleared, and an old pool's
root left in that slot stays alive.

File 11. crates/singlefs-harness/src/bad_disk_input.rs. This is a brand
new file, 2168 lines, added the same day as the batch under review.
Fact a: yes, a test names it today. The integration test file
crates/singlefs-harness/tests/second_transaction_supplement_three_bad_disk_input.rs
imports items from the bad_disk_input module of the singlefs_harness crate directly. The file
itself also contains its own internal cfg(test) module.
Fact b: yes, it appears in crates/mutations.tsv, at rows 243, 244, 245,
246, and 247, five rows total.
Fact c: behavior, at least one of the five rows changes behavior, so the
label for the whole file is behavior; four of the five rows change
behavior and one row only changes the text of a string label used for
grouping fault kinds in a report. Row 243's own title: supplement 3, item
5, all four entry width fault kinds are reported back as not applicable,
so not a single corrupted image belonging to general survey's first
family is actually produced, behavior. Row 244's own title: supplement 3,
item 5, the bit flip fault kind flips no bit at all, so not one byte on
disk changes yet the fault is still reported as applied, behavior. Row
245's own title: supplement 3, item 5, when the pieces are merged back
together the order within one piece is reversed, making the report
depend on the thread count, behavior. Row 246's own title: supplement 3,
item 5, the known panic list identifies a match by comparing the whole
location string for exact equality, equivalent to matching by line
number, behavior. Row 247's own title: supplement 3, item 5, the inode
tree fault kind and the extent fault kind are given the same name, so the
report's own grouping by name merges the two into one cell; this one row
only edits a string label's text and does not change any decision logic,
constant.

File 12. crates/singlefs-harness/src/read_tally.rs. This is a brand new
file, 158 lines, added the same day as the batch under review. Its own
stated role is read counting at the block layer.
Fact a: yes, a test names it today. The integration test file
crates/singlefs-harness/tests/second_transaction_parallel_line_two_mounted_read.rs
imports items from the read_tally module of the singlefs_harness crate directly. The file itself
also contains its own internal cfg(test) module.
Fact b: no, it does not appear in crates/mutations.tsv, zero rows.
Fact c: none, because fact b is no.

File 13. crates/singlefs-harness/src/crash.rs. This file was changed the
same day as the batch under review.
Fact a: yes, a test names it today. Many integration test files import a
items from the crash module of the singlefs_harness crate directly; one example is
crates/singlefs-harness/tests/first_transaction_region_bytes.rs. The file
itself also contains its own internal cfg(test) module.
Fact b: yes, it appears in crates/mutations.tsv, at rows 15, 16, 17, 18,
95, 100, 131, 132, 133, 134, 135, 136, 137, 138, 142, 181, 191, and 274,
eighteen rows total.
Fact c: behavior, all eighteen rows change behavior. Their own titles, in
the same row order as fact b: an oracle compares only the transaction
generation number, not the instance; an oracle looks up the version by
transaction generation number alone; an oracle lets through an updated
root that has no matching version; a recovery pass under an ignore
setting does not go through the oracle at all; the record hints for a
crashed image omit records that live in the base image; layer 0's own
counting does not record a not applicable outcome, so evaluated count
plus not applicable count no longer equals the total state count; the
first slice is dropped during layer 0 parallelism, so the state count
assertion must go red; two adjacent slices overlap during layer 0
parallelism, so the state count assertion must go red; the same overlap
bug caught instead by the slicing unit test; when picking a state by
ordinal the within segment subset mask is inverted; when merging slices
the first violation recorded is taken from the later slice instead; when
merging slices the checker's own first violation for each invariant is
taken from the later slice instead; the SINGLEFS_LAYER0_THREADS
environment variable is never read; setting SINGLEFS_LAYER0_THREADS to
zero silently falls back to a single thread; when merging slices the
first violation from the pass that does not look at the journal is taken
from the later slice instead; a barrier no longer splits a segment, so
one fewer barrier merges two segments into one; the record checker's
second judging criterion does not recognize being overwritten by a later
write in the stream, so a legitimate reuse gets judged as the unit being
missing; a sparse device's whole segment zero fill inserts sectors that
are all zero instead of removing them, which under mkfs clearing a large
ring runs out of memory when multiplied by cloning and parallelism.

File 14. crates/singlefs-harness/src/crash_injection.rs. This file was
changed the same day as the batch under review.
Fact a: yes, a test names it today. The integration test file
crates/singlefs-harness/tests/second_transaction_supplement_three_crash_injection.rs
imports items from the crash_injection module of the singlefs_harness crate directly. The file
itself also contains its own internal cfg(test) module.
Fact b: yes, it appears in crates/mutations.tsv, at rows 179, 180, 182,
183, 184, 188, and 189, seven rows total.
Fact c: behavior, at least one of the seven rows changes behavior, so the
label for the whole file is behavior; six of the seven rows change
behavior and one row only changes the numeric value of one named
constant. Row 179's own title: supplement 3, item 3, even when zero crash
points are meant to be drawn one is drawn anyway, behavior. Row 180's own
title: supplement 3, item 3, judging a crash point takes the chosen root
instead of the root actually reached after applying the record prefix,
behavior. Row 182's own title: supplement 3, item 3, within a segment
only a prefix is truncated, no arbitrary proper subset is arranged,
behavior. Row 183's own title: supplement 3, item 3, the record checker
is never run on a crashed state, behavior. Row 184's own title:
supplement 3, item 3, crashed states are again arranged only after the
starting point has finished running, behavior. Row 188's own title:
supplement 3, item 3, on a crashed state whether the floor lands inside a
gap left by a rollback is never computed and is instead hardcoded to a
fixed absent value, behavior. Row 189's own title: supplement 3, item 3,
the seed base for this test cycle, a named constant called
SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE, is swapped for a different number;
this one row only changes that constant's own numeric value and does not
alter any surrounding decision logic, constant.

File 15. crates/singlefs-harness/src/device_log.rs. This file was changed
the same day as the batch under review.
Fact a: yes, but only through its own internal cfg(test) module inside
this same file; no integration test file under any tests directory
imports anything from the device_log module of the singlefs_harness crate anywhere. Because only
the internal test exists, the test file name to give for this row is the
source file's own name, device_log.rs.
Fact b: yes, it appears in crates/mutations.tsv, at row 280, one row.
Fact c: behavior, one row, changing behavior. Row 280's own title: the
device side log, when folding a zero fill segment back down to one step,
does not check whether this entry is all zero, so a write carrying real
content sandwiched inside the segment gets folded the same way.

File 16. crates/singlefs-harness/src/first_transaction_regions.rs. This
file was changed the same day as the batch under review.
Fact a: yes, a test names it today. The integration test file
crates/singlefs-harness/tests/first_transaction_region_bytes.rs imports a
items from the first_transaction_regions module of the singlefs_harness crate directly. The file
itself contains no internal cfg(test) module of its own.
Fact b: yes, it appears in crates/mutations.tsv, at rows 117, 118, and
120, three rows total.
Fact c: behavior, at least one of the three rows changes behavior, so the
label for the whole file is behavior; two of the three rows change which
bytes are read or the order entries appear in, and one row only edits a
numeric count field inside one table entry. Row 117's own title: E142,
measurement 5, the region list's own row order is swapped, behavior. Row
118's own title: E142, measurement 5, the data unit row's own slot count
is undercounted by one; this row only edits the numeric value of one
count field inside a fixed table entry and does not change any
surrounding decision logic, constant. Row 120's own title: E142,
measurement 5, the region's own digest no longer covers its content,
behavior.

File 17. crates/singlefs-harness/src/model_comparison.rs. This file was
changed the same day as the batch under review.
Fact a: yes, but only through its own internal cfg(test) module inside
this same file; no integration test file under any tests directory
imports anything from the model_comparison module of the singlefs_harness crate anywhere. Because
only the internal test exists, the test file name to give for this row is
the source file's own name, model_comparison.rs.
Fact b: yes, it appears in crates/mutations.tsv, at rows 162, 164, and
168, three rows total.
Fact c: behavior, all three rows change behavior. Row 162's own title: a
fix from an earlier verdict, glue code maps a rollback target that was
abandoned into a target below the floor F. Row 164's own title: a fix
from an earlier verdict, glue code also maps a small device being full,
and devices disagreeing on placement, into the unit area wall. Row 168's
own title: a fix from an earlier verdict, glue code maps a tree table
with zero entries and an unsupported first version into a target below
the floor F.

File 18. crates/singlefs-harness/src/scenario.rs. This file was changed
the same day as the batch under review.
Fact a: yes, a test names it today. Several integration test files import
items from the scenario module of the singlefs_harness crate directly; one example is
crates/singlefs-harness/tests/first_transaction_region_bytes.rs. The file
itself contains no internal cfg(test) module of its own.
Fact b: yes, it appears in crates/mutations.tsv, at row 119, one row.
Fact c: constant, one row, only changing a named constant's value. Row
119's own title: E142, measurement 5, the write time is changed to be
taken from the system clock instead of the named fixed constant
FIXED_WRITE_TIME_SECONDS; what this row pins down is that a specific
constant value must be used for the write time, not any branch of
decision logic.

File 19. crates/singlefs-harness/src/segments.rs. This file was changed
the same day as the batch under review.
Fact a: yes, a test names it today. Several integration test files import
items from the segments module of the singlefs_harness crate directly; one example is
crates/singlefs-harness/tests/first_transaction_step_one_mkfs.rs. The file
itself also contains its own internal cfg(test) module.
Fact b: no, it does not appear in crates/mutations.tsv, zero rows.
Fact c: none, because fact b is no.

File 20. crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs.
This file was changed the same day as the batch under review. It compiles
to its own separate command line program, not a library module.
Fact a: yes, but only through its own internal cfg(test) module inside
this same file; no integration test file under any tests directory
references it in any way at all. Because only the internal test exists,
the test file name to give for this row is the source file's own name,
e156_allocation_basis_counts.rs.
Fact b: yes, it appears in crates/mutations.tsv, at row 228, one row.
Fact c: behavior, one row, changing behavior. Row 228's own title: E156's
own referenced_slots computation drops the fallback addition for the
instance table entirely, rather than changing the value of any named
constant.

File 21. crates/singlefs-harness/src/bin/first_transaction_device_log_check.rs.
This file was changed the same day as the batch under review. It compiles
to its own separate command line program, not a library module.
Fact a: no. No integration test file under any tests directory references
it in any way, and the file itself contains no internal cfg(test) module
of its own. There is no test of any kind for this file today.
Fact b: no, it does not appear in crates/mutations.tsv, zero rows.
Fact c: none, because fact b is no.

File 22. crates/singlefs-harness/src/bin/first_transaction_region_bytes.rs.
This file was changed the same day as the batch under review. It compiles
to its own separate command line program, not a library module.
Fact a: no. The only place this file's own name appears anywhere inside
any test file is inside two comments, not executable code, in
crates/singlefs-harness/tests/first_transaction_region_bytes.rs; that same
test file's own executable code imports items from
the first_transaction_regions and scenario modules of the singlefs_harness
crate instead, and never imports or runs this bin file itself. The file itself
contains no internal cfg(test) module of its own. There is no test of any
kind that actually imports or executes this file today.
Fact b: no, it does not appear in crates/mutations.tsv, zero rows.
Fact c: none, because fact b is no.

End of the 22 files. Fill only the table described below; do not write any
prose outside of it, no summary, no commentary, no discussion. Now produce
your answer as Row 1 through Row 22, one row per file, in this same order.
For each row, state the file's own path exactly as given above, then state
fact a exactly as given, then fact b exactly as given, then fact c exactly
as given, then your own answer to question d with its yes, no, or unknown
verdict, the function name if your verdict is yes, the sentence starting
with "This would be refuted by:", and the sentence naming which fact
letter or which row's own quoted title your verdict rests on. After Row
22, state the final count of how many of your 22 answers to question d
were unknown.
