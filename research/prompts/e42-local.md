You are reviewing an experiment result for a from-scratch copy-on-write filesystem called singlefs.
It is in the format-design stage: no code, no on-disk format frozen yet.
Your job is to find counterexamples. Do not use any markdown emphasis in your answer.
Write plain English sentences and plain lists. Keep it under 600 words.

Design context:

1. The filesystem publishes a root after every fsync. Roots are written into a ring of slots that
rotate, so an old root is never overwritten in place. A candidate shape for that ring was proposed
but never verified: 2 regions times 4 slots times 256 bytes, which is 8 slots and 2048 bytes total.
2. An invariant says: blocks referenced by the most recent K generations of roots must not be
reallocated to another object, where K equals the ring depth. So ring depth and block reuse delay
are the same number.
3. Self witnessing units such as root slots have no parent pointer carrying a checksum, so tearing
is detected by a whole unit checksum plus slot rotation. Tearing happens at the granularity of the
device atomic write unit.

Measured, five identical runs, twelve unit tests green, seven mutations all caught:

1. With a root per fsync, one full lap of the ring is 8 operations, so the block reuse delay is 8
operations. With a root per checkpoint, measured elsewhere as 200 roots per 200000 operations, one
lap is 8000 operations.
2. Tearing: a 256 byte slot on a 512 byte atomic unit loses 2 slots per write; on 4096 it loses 16;
on 65536 it loses 256, which is more than the whole ring of 8 slots.
3. Positive control: raising the slot width to 512, which is this machine's probed physical block
size and minimum io size, brings the loss back to 1 slot per write. But a 512 byte slot on a 4096
byte atomic unit again loses 8.
4. Price of raising the slot width to the atomic width: total ring bytes become 4096, 32768 or
524288, that is 2, 16 or 256 times the candidate 2048.
5. There is no number anywhere in the project for the worst case block reuse delay requirement.

The inference under attack, call it T:

T1. The candidate shape is rejected, and what falls is the slot width, not the choice of 2 regions
times 4 slots.
T2. The slot width cannot be a format constant. It must be at least the atomic width probed at
mount, otherwise the filesystem refuses to mount.
T3. Whether one lap is long enough cannot be judged, because the requirement has no source.

Your task, in this order:

1. Give concrete counterexamples to T1. In particular, is there a situation where 8 slots is the
part that fails, independently of slot width?
2. Give concrete counterexamples to T2. In particular, what breaks if the slot width is chosen at
mount rather than fixed at format time? Think about where mkfs placed the ring on disk, and about
an image created on a 512 byte device and later attached to a 4096 byte device.
3. Give concrete counterexamples to T3. Is there any way to derive a worst case block reuse delay
requirement from quantities a filesystem already knows, such as the number of in flight
transactions, the checkpoint interval, or the deferred free window?
4. Say whether a block reuse delay of 8 operations is dangerous, safe, or unknowable, and give the
specific crash and recovery scenario that decides it.
5. Accept T, reject T, or accept a narrowed version, and give the narrowed version if that is your
answer.
