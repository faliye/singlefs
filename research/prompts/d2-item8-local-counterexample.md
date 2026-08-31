You are reviewing a design inference for a from-scratch copy-on-write filesystem called singlefs.
It is in the format-design stage: no code, no on-disk format frozen yet.
Your job is to find counterexamples. Do not use any markdown emphasis in your answer.
Write plain English sentences and plain lists. Keep it under 500 words.

Settled decisions you must not argue with.
S1. Stripe width is variable. Every write is a full stripe write. Read-modify-write never happens.
S2. A pointer lists every physical location one by one, each carrying a device identity.
S3. The stripe width is clamp of batched write size plus one, between two and four. The upper bound
four is a constant declared in the superblock.
S4. Each stripe has exactly one parity cell.
S5. When a device is added, a background resumable rebalance runs. Data is moved.
S6. A pool has eight devices in the measurement below.

Measured, by exhaustive enumeration of all twenty eight pairs of simultaneously failed devices,
five hundred twelve objects, width four:
The expected fraction of data lost is the same for every placement rule, three over twenty eight.
The fraction of objects that lose nothing at all differs a lot.
Pin each object to one group of four devices chosen when the object is created, by picking the four
emptiest devices at that moment: 78.6 percent of objects lose nothing, and device usage spread is
zero under both a uniform object size workload and a skewed one where one object in sixteen is sixty
four times larger.
Rotate the group for every stripe: 14.3 percent under uniform, 74.6 percent under skewed, but device
usage spread becomes 31.9 percent under skewed.
Pick the four emptiest devices for every stripe: 57.1 percent under uniform, 77.2 under skewed,
spread zero in both.
Pin by object number without looking at free space: 78.6 percent in both, but spread 88.7 percent
under skewed.

The inference under attack, call it V: the rule should be, choose the group of devices when the
object is created by taking the emptiest devices at that moment, and keep that group for every
later stripe of that object.

Your task, in this order:

1. Give concrete counterexamples to V. A situation where pinning a group at creation time produces a
worse outcome than choosing per stripe, for reasons of correctness, recovery, capacity, or failure
distribution. Do not argue from throughput or speed; this project forbids speed as a criterion.
2. The group is chosen from free space at creation time. Name every way that choice can become stale
or wrong later, and say what breaks in each case.
3. A background rebalance moves data between devices after a new device is added. Say exactly what
happens to a pinned group when its data is moved, and whether the measured 78.6 percent still holds.
4. Objects are created and deleted continuously. Say whether pinning can drive the pool into a state
where some group of four devices is full while others are empty, and what the allocator must do then.
5. Accept V, reject V, or accept a narrowed version, and give the narrowed version if that is your
answer.
