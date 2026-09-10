You are the counterexample leg of a three-way design review for a from-scratch
copy-on-write filesystem. Your only job is to construct concrete counterexamples.
Do not argue which option is better. Do not summarize. Find cases that break things.

Do not use any markdown emphasis in your answer. No bold, no italics, no asterisks.
Write plain sentences and plain numbered lists. Answer in English.

BACKGROUND

A settled clause says: a central map from logical identity to physical location is
the only entry point for dereference and for the free decision. The position
entries carried inside tree pointers are demoted to hints; they may be stale.
Freeing always goes through the map, never through a hint.

What is not settled is how one map entry is encoded. Two candidates:

Candidate ONE. The key is the logical identity five-tuple (33 bytes) plus the
birth generation number (8 bytes), so 41 bytes. Entries are fixed width.
Publishing a new version is a pure insert.

Candidate TWO. The key stays 33 bytes, the five-tuple alone, and the value holds
a version chain. The chain length equals the number of live snapshots plus one.
The number of live snapshots has no upper bound in this design. So entry width has
no upper bound, and publishing changes from a pure insert into a read-modify-write.

Why the version dimension is needed at all: deadlists and snapshot trees reference
OLD versions, while the five-tuple only names a logical position, not a version.
Overwriting the same file offset once produces a new unit with an identical
five-tuple. So a key made of the five-tuple alone can only point at the current
version.

Measured costs of candidate ONE, from an existing arithmetic model:
map entry 61 to 69 bytes; entries per leaf 267 to 236; tree height stays 4, so a
separate measured publish overhead of 66048 bytes per publish does not change by
one byte; a packing container's per-slot self-description grows 47 to 55 bytes,
which moves the 512 byte size class capacity from 58 to 57 slots and leaves the
1 KiB, 4 KiB and 8 KiB classes unchanged.

A separate project principle, settled 2026-08-28: never read-modify-write, and
that principle is explicitly extended to the device side, meaning never issue a
write smaller than the device's minimum io size, because the device would then
perform an internal read-modify-write and a power cut could damage neighbouring
data already persisted in the same mapping unit.

A coupled open question: whether the block pointer header carries the birth tree
id and birth transaction group number. Today it carries neither. Candidate ONE's
key requires the dereferencing side to assemble the five-tuple plus the birth
generation BEFORE it can look up the map.

YOUR TASK

Construct concrete counterexamples, with a specific sequence of operations and
specific numbers, for as many of the following as you can. For each one, either
give the counterexample or say plainly that you could not construct one.

Q1. Construct a case where candidate ONE gives a wrong answer or cannot answer.
    Be specific about which operation, which key, and what goes wrong.

Q2. Construct a case where the claimed disqualifier of candidate TWO does not
    actually bite. That is: show a realistic workload or configuration where the
    unbounded entry width and the read-modify-write publishing are harmless, and
    say exactly why.

Q3. Construct a third encoding that is neither ONE nor TWO and that beats both on
    at least one axis. State its key, its value, its width, and what it costs.

Q4. Construct a case where the cost list of candidate ONE given above is
    incomplete, that is, name a fifth cost that the list misses, with the
    mechanism by which it arises.

Q5. Construct a case where choosing candidate ONE forces a particular answer to
    the coupled open question about the pointer header, in a way that would be
    wrong to decide implicitly rather than explicitly.

For each answer, state the operation sequence, the concrete values, and what
exactly goes wrong. If you cannot construct a counterexample for a question, say
so in one sentence and move to the next.
