SINGLEFS ROLLBACK CANDIDATE DEDUPLICATION ARITHMETIC - LOCAL ATTACK LEG - ROUND m2-rollback-forward-r3 - PART 2 OF 2

You are one of several independent reviewers examining a design proposal for
singlefs, a copy on write filesystem being designed from scratch. Your only
job in this document is arithmetic: given a fixed counting procedure taken
directly from this design round's own working notes, and a small set of
definitions stated below to make three short published-root histories
unambiguous, work out three numbered items, each asking you to apply the
counting procedure, mechanically and step by step, to one of the three
histories. This is not a design review; you are only being asked to execute
the stated procedure correctly, and to say plainly if the procedure and a
history do not fit together.

Do not assume anything not stated in the facts or in the definitions below.
Read every fact carefully; every qualifier is load bearing. Do not use any
markdown emphasis anywhere in your answer: no bold text, no italic text, no
backtick code formatting, no asterisk bullets, no pipe tables. Number your
answers to match the three items given at the end. For every numbered item,
end with one sentence starting with "This would be refuted by:" describing
the specific fact or definition you would have had to misread, or the
specific step of the procedure you would have had to skip or misapply, for
your answer to that item to be wrong.

When your answer needs to point at where a rule comes from, point at it only
by the fact number given below (fact 1 or fact 2). Do not write any source
file name together with a line number, and do not write any line number at
all anywhere in your answer, because the underlying files may have moved to
different line numbers by the time anyone reads your answer.

Background needed to read the facts. Every published root record has a pair
of pointers describing the current user visible filesystem content: an inode
tree root pointer and an extent tree root pointer. Two root records are
considered to carry the identical user visible state exactly when both of
these two pointers match between them; if either pointer differs, the two
root records are considered to carry different user visible states. The
purpose of the counting procedure in fact 1 below is to build a short list of
how many distinct such user visible states remain reachable as rollback
targets, out of a longer history of published root records.

Fact 1. The counting procedure, applied to a history of published root
records ordered from newest to oldest: examine the root records one at a
time, strictly from the newest one to the oldest one. Keep a running list of
the states counted so far (this list starts out empty, before the newest
record is examined). For each root record you examine, count it, and add its
state to the running list, only if its (inode tree root pointer, extent tree
root pointer) pair does not match the pair of any state already on the
running list; if its pair does match some state already on the running list,
do not count it, and do not add anything to the running list for it. Move on
to the next older root record and repeat, until every root record in the
history has been examined. The final size of the running list is the answer
this procedure produces for that history. Source: this design round's own
working notes, the row defining this deduplication procedure.

Fact 2. Separately from fact 1's procedure itself, this is how "does not
match" and "does match" are determined when comparing two root records'
(inode tree root pointer, extent tree root pointer) pairs: the comparison is
of the recorded entries for these two trees, not of the physical on disk
location of the structure that holds those entries. Two root records that
happen to store their tree entries in physically different on disk locations
can still count as carrying the identical pair, and therefore the identical
user visible state, if the recorded entries themselves are equal. Source:
decision record D16 (publish semantics), the definition item explaining how
"non-empty" is determined for a root record.

Definitions for this exercise, stated here only to make the three histories
below unambiguous; these definitions are not quoted from any document, only
the underlying mechanism they describe is.

Definition 1. A history of published root records is written here as a
sequence of capital letters separated by hyphens, listed left to right from
the oldest published root record to the newest published root record (this
is the opposite order from fact 1's procedure, which examines newest to
oldest; you will need to reverse the order before applying fact 1).

Definition 2. Each capital letter names one specific (inode tree root
pointer, extent tree root pointer) pair. Two positions in the same history
are written with the same letter exactly when the root records published at
those two positions carry the identical such pair, by fact 2's comparison
rule; they are written with different letters exactly when the pairs differ.

Definition 3. A root record that changes the user visible content compared
to everything published before it in the same history is written with a
letter that has not appeared anywhere earlier in that same history.

Definition 4. A root record that is a rollback to some earlier target root
record restores the user visible content to be identical to that earlier
target's own (inode tree root pointer, extent tree root pointer) pair, by the
meaning of "rollback" itself (rolling back to an earlier root record makes
the pool's user visible content match that earlier root record again).
Therefore, a rollback root record is written with the same letter as whichever
earlier position in the history already carries the target it is rolling back
to.

Worked example, not one of the three items below: the history written as
"A-D" means two root records were published, oldest first: first one
carrying state A, then one carrying a different state D (a state that had
never appeared before, per definition 3, since it is written with a letter
different from A). Applying fact 1's procedure to this history (examined
newest to oldest, so D first, then A): D is examined first, the running list
is empty, so D does not match anything on it and is counted, running list
becomes "D"; A is examined next, it does not match D, so it is counted too,
running list becomes "D, A"; the final answer for "A-D" is 2.

Item 1. Consider the history written as "A-D-A" (three root records, oldest
to newest: state A, then state D, then a third root record whose (inode tree
root pointer, extent tree root pointer) pair is identical to the very first
one's, per definition 4 - for instance a rollback landing back on the state
that had identity A). First, write out this history's three root records in
newest to oldest order, the order fact 1's procedure actually examines them
in, with each one's letter. Then apply fact 1's procedure to this
newest-to-oldest sequence one root record at a time: for each one, state
whether it is counted or not, and if it is not counted, name the earlier
(meaning: examined earlier in this newest to oldest pass, so actually a newer
root record than the one you are currently looking at) already-counted root
record whose pair it matches. Report the final size of the running list.

Item 2. Consider the history written as "A-D-A-D" (four root records, oldest
to newest: state A, then state D, then a third root record identical to the
first one per definition 4, then a fourth root record identical to the second
one per definition 4). Apply exactly the same procedure as item 1, in the
same level of step by step detail, to this four root record history.

Item 3. Consider the history written as "A-D" itself (the same two root
record history used in the worked example above, which was solved for you
there only as an illustration of the mechanics; solve it here yourself, in
the same level of step by step detail as items 1 and 2, without simply
copying the worked example's stated final answer). Then, once you have your
own answer for "A-D" here and your answers for items 1 and 2, state whether
all three histories ("A-D", "A-D-A", and "A-D-A-D") produce the same final
answer or different final answers; if any two of the three differ, name which
two and by how much.
