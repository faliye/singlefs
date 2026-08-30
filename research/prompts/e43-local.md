You are reviewing a design inference for a from-scratch copy-on-write filesystem called singlefs.
It is in the format-design stage: no code, no on-disk format frozen yet.
Your job is to find counterexamples. Do not use any markdown emphasis in your answer.
Write plain English sentences and plain lists. Keep it under 600 words.

Background facts, all verified in the project knowledge base today:

1. A decision called D21 says: each on-disk unit reserves an extension point, a fixed span of
bytes plus an interface, for a third party who ports singlefs to a new storage medium.
The third party may store a pointer there and use it for something singlefs does not understand.
2. The size of the extension point is declared per medium line in the superblock. The third party
picks the value. The upper limit is fixed by the singlefs project as a constant number of bytes,
not as a fraction of the unit size. The value of that limit is undecided. The decision text uses
128 bytes as an illustration and explicitly says it is not decided.
3. Hard constraints: the extension point must be inside the checksum and authentication coverage,
and any pointer inside it may only point into space charged to its own quota, never at a core
object of the filesystem.
4. Per-unit metadata accounting, counting only bytes paid to manage one unit: singlefs 108.0 bytes
with 2 replicas, 152.0 bytes with 4+2 erasure coding. Same accounting for others: ZFS 128.0 bytes,
btrfs 82.5 bytes at a 4 KiB unit and 207.3 bytes at a 128 KiB unit.
5. An experiment computed three bounds. Lower bound 9 bytes: one location entry is device id 1 byte
plus physical offset 6 bytes, plus 2 bytes of length so a checker can verify the quota range is
recorded as allocated. Upper bound A is 19 bytes: the D21 text asserts singlefs is cheaper than ZFS
at a 128 KiB unit, and 108 plus N must stay under 128. Upper bound B is 864 bytes: with 16 million
leaves, 40 byte pointers and a 64 byte node header, a 2 KiB node loses a tree level once the
extension point reaches 865 bytes; the 4 KiB node at 1473, the 16 KiB node at 6081, the 64 KiB
node at 55233.
6. Upper bound A applies only if the extension point bytes are counted as singlefs per-unit
metadata. If they are counted instead as third party payload, bound A does not apply and the
binding bound is B.
7. The D21 text also claims singlefs uses half of what btrfs uses at a 128 KiB unit. Taken
literally that is false even with a zero size extension point: 108 times 2 is 216, which exceeds
207.3. The true ratio is 52.1 percent.

The inference under attack, call it P: D21 cannot decide the byte limit on its own, because the
answer is entirely determined by an accounting convention that has not been decided, namely whether
extension point bytes count as singlefs per-unit metadata. So the correct action is to decide the
convention first, then the number, and to rewrite the half of btrfs claim as about 52 percent.

Your task, in this order:

1. Give concrete counterexamples to P. A counterexample is a specific situation where the byte
limit can be decided without settling the accounting convention, or where the convention does not
change the answer. State each one in one or two sentences.
2. Attack the three bounds themselves. For each of the lower bound of 9, upper bound A of 19, and
upper bound B of 864, say what input would have to be wrong for the bound to be wrong, and whether
that input looks wrong to you.
3. Name any constraint the experiment did not model that could bind tighter than all three bounds.
4. State whether you would accept P, reject P, or accept a narrowed version, and give the narrowed
version if that is your answer.
