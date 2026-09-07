# Background: should the 6-byte transaction number be cut from class-code-2 unit headers?

This is a from-scratch COW filesystem in the format design phase. No code yet, only
an on-disk format under design. Existing COW filesystems are design input, not a port target.

Do not use any markdown emphasis in your answer. Plain text only. Answer in English.

## The object under question

Every unit written to disk carries a self-describing header. Units are divided into classes:

- code 1: data unit, 32768 bytes on disk
- code 2: index node, 16384 bytes (btree node)
- code 3: packed record container

All three classes carry a field called the write sequence, 10 bytes wide:
(instance id, 4 bytes) + (transaction number, low 48 bits, 6 bytes).

It was added on 2026-09-05 by a ruling that solved this problem: during scan-based rebuild
the rebuilder finds several physical versions carrying the same logical identity and must
decide which one is the current version of which head.

The question: for class code 2 only, should the 6-byte transaction number be removed,
leaving a 4-byte write sequence that carries only the instance id?

## Verbatim from the settled clauses

Every line below was checked against the repository today by the person asking you.

1. Published predicate, invariant I-1.2. For a unit whose header records write sequence
   (i, n) and birth txg b, given an instance table row (i, T_pub, W):
   code 1 requires b <= T_pub OR n <= W. Code 2 and code 3 require b <= T_pub only.
   So the published predicate never reads n for code 2.

2. Invariant I-1.8, total order after merge. It covers code 1 and code 3 only.
   The code 1 key is the write sequence. The code 3 key is
   (birth txg, instance id, transaction number). The invariant text ends with the words
   "code 2 does not enter this one".

3. The C113 ruling, revision 9, rule P3. Code 2 is excluded from the current-version
   selection rule, and the stated reason is that code 2 can be recomputed offline by
   traversal, so rebuild rewrites it as derived state. An earlier revision said code 2
   is derived state, and that reason was struck out because accounting-tree code 2 nodes
   are authoritative state.

4. Decision D18 settled item 7, the header field table. For code 2 it says the write
   sequence buys this: a node belonging to a checkpoint that never got published is
   judged garbage by the published predicate and can be swept.

Consequence: no reader anywhere in the repository consumes the 6-byte transaction number
of a code 2 header today. A repository-wide search found none.

## The cost side, arithmetic run today

Fanout formula, identical in three existing experiment programs: (16384 - header) / entry width.

The code 2 identity segment width varies by tree (some trees carry a key range in the header,
some do not), so the header comes in three brackets: 68, 77, 86 bytes today.
Cutting 6 bytes makes them 62, 71, 80.

Entry widths taken from settled clauses and existing experiment artifacts:
inode tree internal node 93, accounting tree 75, allocation record tree 20,
extent / plaintext mapping tree 85.

Result over 12 combinations of (tree, bracket): 10 show no change in fanout at all.
Two show one extra slot: allocation record tree 815 to 816, extent tree 191 to 192.
Tree height never changes in any combination.

## Two project rules that pull in opposite directions

Rule one, from the design discipline file: do not sacrifice self-containment to save space.
The default answer to "can this field be dropped or narrowed" is no, unless measurement
shows the saved part is a bottleneck. Its scope note says verbatim that it governs whether
a field that has already been argued useful should be cut to save space, and that it does
not govern whether to add a field in the first place, which still needs its own reason.

Rule two, the same file: comparing space consumption against other filesystems is not
admissible as a criterion in this project at all.

Observation that connects them: the justification recorded for the code 2 write sequence
(item 4 above) only supports the 4-byte instance id. The 6-byte transaction number was
never separately argued useful for code 2. It came along because all three classes were
given one field of one width.

## Freeze constraint

The unit header belongs to layer 1 of a four-layer format freeze (superblock plus block
header). Layer 1 is frozen first and every other layer is built on it. After the freeze,
neither adding nor removing these bytes is possible without a format break.

## Existing project policy for reserved bytes

Decision D8 states that flags, padding and reserved segments all share one policy:
always zero, and a reader that sees a non-zero value must fail that record with EIO
(object level, not the whole pool). This is invariant I-9.7. Decision D18 applies the same
policy to the 8 flag bits in the common header prefix: first version always zero, non-zero
rejected, and the project deliberately does not offer an "ignore unknown bits" path.

## The three arms

A. Cut to 4 bytes for code 2. The write sequence field is then class-dependent in width:
   10 bytes for code 1 and code 3, 4 bytes for code 2.

B. Keep 10 bytes and keep writing the real transaction number into code 2 headers.
   This is the status quo.

C. Keep 10 bytes of width, but define the 6 bytes as a reserved segment that must be zero,
   with readers rejecting a non-zero value, following the existing reserved-segment policy.

Note on offsets, which you may or may not find relevant: the write sequence is not at a
common offset across the three classes today, because the identity segments differ
(code 1 carries a 33-byte five-tuple, code 2 carries tree id plus level plus an optional
key range, which is exactly why code 2 has three header brackets).

## Your assignment: hunt for counterexamples

Your job is not to pick an arm and defend it. Your job is to find concrete scenarios,
built out of the settled clauses above, in which one of the three arms produces a wrong
outcome on disk.

Work through at least these attack surfaces, and say explicitly for each one whether you
found a counterexample or not:

1. Crash and reuse of a checkpoint number. The project states that after a crash the same
   checkpoint number gets reissued. Two code 2 nodes can then exist with the same tree id,
   the same level, the same key range and the same birth txg, with different contents,
   both readable on disk. Under arm A, is there any situation where the system must tell
   them apart and now cannot?

2. Sweeping garbage. The stated purpose of the code 2 write sequence is that an unpublished
   checkpoint's nodes are judged garbage and swept. Walk the predicate in item 1 of the
   verbatim section and check whether 4 bytes really suffice for every case, including the
   case where the instance table has no row for that instance, and the case where the
   instance id equals the mounting root's instance.

3. Authoritative code 2 nodes. Revision 9 struck out the claim that code 2 is derived state,
   because accounting tree code 2 nodes are authoritative. If a code 2 node is authoritative
   and cannot be recomputed by traversal, does the exclusion of code 2 from the current
   version rule still hold? If it does not, which arm breaks first?

4. Arm C specifically. A reserved-must-be-zero segment that a future version wants to use:
   trace what an old reader does when it meets a new unit whose reserved bytes are non-zero,
   and compare that against what an old reader does under arm A when it meets a header that
   is 6 bytes longer than it expects.

For every counterexample you claim, give the exact sequence of events and name which
settled clause is violated. If you cannot construct one for a given surface, say so
plainly rather than inventing one.
