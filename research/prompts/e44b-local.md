You are reviewing a design inference for a from-scratch copy-on-write filesystem called singlefs.
It is in the format-design stage: no code, no on-disk format frozen yet.
Your job is to find counterexamples. Do not use any markdown emphasis in your answer.
Write plain English sentences and plain lists. Keep it under 600 words.

New premise for this round, decided by the project owner. Do not argue with it. Reason on top of it.
Being cheaper in space than other filesystems is not a criterion for this project. Saving space was
never a goal. A project rule already says the same thing: when asked whether a field can be dropped,
narrowed, or recomputed, the default answer is do not drop it, because what you save is space and
what you lose is whether a single unit picked up alone can still describe itself.

Design facts:

1. Each on-disk unit reserves an extension point: a span of bytes plus an interface, for a third
party who ports the filesystem to a new storage medium. The third party may put a pointer there and
use it for something the core does not understand.
2. The size is declared per medium line in the superblock. The third party picks the value, and may
pick zero. The core project fixes an upper limit as a constant number of bytes. That limit is
undecided.
3. Hard constraints: the extension point must be inside the checksum and authentication coverage;
its size must be declared in the format so generic tools like scrub, scan-rebuild and rescue can
skip it; any pointer inside it may only point into space charged to its own explicit quota, which
is maintained incrementally at commit time and must be included when admission control computes the
worst case before an operation is allowed to start; and it may never point at a core object.
4. Measured, five identical runs: a legal pointer plus its quota range needs between 7 and 13 bytes,
depending on two conventions nobody has written down. A two byte length field spans only 65535
bytes, which cannot cover a 128 KiB unit. If every unit carries the extension point, a 2 KiB index
node loses a tree level once the extension point reaches 865 bytes, a 4 KiB node at 1473, a 16 KiB
node at 6081, a 64 KiB node at 55233. If only data units carry it, no tree level is ever lost.
5. The index is derived state: losing it is only slow, it can be rebuilt by scanning units, because
every unit describes itself. Units, accounting and roots are authoritative state.

The inference under attack, call it Q:

1. The former upper limit of 19 bytes, derived from staying cheaper than ZFS, must be deleted along
with its reasoning, because the whole comparison axis is not a criterion for this project.
2. The comparison table against other filesystems may only be used to refute the claim that this
project is extravagant. It may not be used as an upper limit.
3. The only bound derived from function is the lower bound. The upper limit should be set by asking
what worst case the core must tolerate, and the costs of a larger limit to the core are linear in N
and small, so there is no reason to set the limit narrow.
4. So the open question should be reworded: not how many bytes, but which worst case the limit
protects the core from.

Your task, in this order:

1. Give concrete counterexamples to Q. A counterexample is a specific mechanism, unrelated to
saving space, that forces the upper limit to be narrow. State each in one or two sentences and say
what would fail if the limit were larger.
2. Find any cost to the core that is not linear in N. Step changes matter most: a value of N that
crosses a sector, a page, an atomic write width, a cache line, or a single I/O boundary.
3. Say whether the limit should exist at all, or whether some other constraint should replace it.
4. Accept Q, reject Q, or accept a narrowed version, and give the narrowed version if that is your
answer.
