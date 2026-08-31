You are reviewing a design inference for a from-scratch copy-on-write filesystem called singlefs.
It is in the format-design stage: no code, no on-disk format frozen yet.
Your job is to find counterexamples. Do not use any markdown emphasis in your answer.
Write plain English sentences and plain lists. Keep it under 600 words.

Settled decisions you must not argue with. Reason on top of them.
S1. Stripe width is variable. Every write is a full stripe write. Read-modify-write never happens.
S2. Writes smaller than the device preferred write granularity are never issued.
S3. Disks in a pool need not be the same size.
S4. When the device set changes, an explicit geometry re-evaluation transaction runs and the volume
stays mounted. Forcing an unmount and remount is rejected because a service interruption is not
acceptable.
S5. A pointer location entry carries a device identity. This was settled on two grounds that have
nothing to do with striping: the cost of carrying a device identity, namely rewriting every tree that
points at a removed device, is already paid because a back pointer index is in the format from day
one; and not carrying a device identity forces a logical to physical mapping table that must
bootstrap itself, the way btrfs carves a system chunk array out of its superblock.
S6. Unit geometry is computed once at mount time and only looked up at run time. That rule explicitly
does not cover stripe width, because stripe width depends on the size of the current write and on the
set of devices writable at that moment, and neither is an input to the mount time computation.

The open question, quoted from the design document: does every single write possibly use a different
stripe width, or is the width fixed inside one allocation group and only changes between groups? The
document states the consequence of each reading. Every write possibly different means every physical
location is listed one by one, each carrying a device identity. Fixed inside an allocation group
means a logical address plus a chunk mapping table can be used, and then pointers do not need a
device identity.

The inference under attack, call it T: the open question should be settled as every write possibly
uses a different width, because the only thing the fixed-inside-a-group reading buys, namely pointers
without a device identity, has already been taken away by S5, which was settled on independent
grounds, while the cost of the fixed reading remains, namely that the width cannot follow the set of
devices writable at that moment.

Important constraint on your answer: this project has never measured any performance effect of stripe
width, and its own rules forbid using the phrase this is faster as a criterion. Do not argue from
performance. Argue from failure modes, from what becomes impossible to express, and from what must be
recomputed or rewritten.

Your task, in this order:

1. Name everything the fixed-inside-a-group reading still buys after S5 is settled. For each item say
whether it survives S5 or dies with it. Be concrete about what code or format structure it affects.
2. Give concrete counterexamples to T. A counterexample is a specific situation where fixing the
width inside an allocation group buys something real that carrying a device identity does not already
provide, or where per write variable width creates a problem that grouped width does not have.
3. State what per write variable width makes impossible or much harder to verify. Consider crash
point replay, checker design, and the requirement that every branch must be forcibly enterable by a
test.
4. Name the inputs to a per write width decision. For each, say whether it can change while a single
write operation is in flight, and what breaks if it does.
5. Accept T, reject T, or accept a narrowed version, and give the narrowed version if that is your
answer.
