You are the counterexample leg of a three-way design review for a copy-on-write
filesystem called singlefs. Your job is to attack a candidate conclusion.
Answer in English. Do not use any markdown emphasis.

CONTEXT

The filesystem stores its superblock and its root ring in fixed on-disk structures
that are updated by slot rotation: a small set of slots is written in round robin,
each slot carries a whole-unit checksum and a generation number, and recovery must
verify every candidate slot first and only then pick the newest valid one by
generation. The reading order is a requirement, not an implementation detail.

On zoned block devices, sequential-write-required zones cannot be overwritten in
place. Each such zone has a write pointer (WP) maintained by the device, not by the
filesystem. The filesystem cannot compute or verify the WP.

An open design question asks what role the WP may play. One candidate arm says the
WP is only an addressing hint: it tells you where to write next and where to start
scanning, but it never enters any decision about which slot is newest, which slot is
published, or whether a slot is intact.

That arm was attacked earlier with this argument: on an append-only zone, deciding
where to start scanning is the same thing as deciding which slots are candidates.
So either the WP defines the candidate set, in which case it did enter the newest
selection and the arm contradicts itself, or you scan the whole zone, in which case
you must read the bytes above the WP, which the filesystem never wrote, and those
bytes will fail the whole-unit checksum and be judged corrupt when they are merely
unwritten.

NEW MEASUREMENT

We just measured this on a Linux null_blk memory backed host managed zoned device,
4 zones of 64 MiB, zone 0 conventional and zones 1 to 3 sequential, with a physical
block size of 4096 bytes. Five runs were byte identical. Positive and negative
controls both passed. Results:

- Reading the whole region above the write pointer succeeded, return code 0, and
  returned 67076096 bytes that were all the single byte value 0xff.
- On the same device, an unwritten conventional zone read back as all 0x00.
- After a zone reset, the whole zone read back as all 0xff and the marker bytes
  written before the reset could not be found.
- With the module parameter zone_full set, every zone reports a write pointer at
  the end of the zone while the filesystem has committed zero bytes. So the state
  where the write pointer leads the filesystem commit point is constructible.

CANDIDATE CONCLUSION V TO ATTACK

Candidate V says: the measurement shows the scan the whole zone route survives on
readability grounds, because the bytes above the write pointer are readable. But it
is still not enough to settle the question today, because the rule that separates
unwritten from corrupt takes as its input the byte pattern the device returns for
unwritten space, which is 0xff for sequential zones and 0x00 for conventional zones
on the very same device. That pattern is a device supplied fact, and the project has
never written down the boundary between device maintained state and device probed
geometry.

YOUR TASK

Attack candidate V. For each item below, either give a concrete construction or say
plainly that you cannot construct one. Do not invent a weak objection just to have
output.

1. Construct a case where a genuinely damaged or lost slot reads back as all 0xff,
   so that a rule saying all 0xff means unwritten would silently misclassify real
   damage as absence. Say how likely that case is and what would have to be true.

2. Candidate V may be too cautious. Argue that the question can be settled today.
   The strongest version of this argument would show that the whole unit checksum
   plus the generation number already answer the question without any rule about
   unwritten patterns, because a slot that fails the checksum is simply not a
   candidate, whether it is unwritten or damaged. Attack or defend that reduction.
   If that reduction works, say exactly what is lost by treating damage and absence
   as the same class.

3. Candidate V may be too optimistic. Argue that even readability does not hold in
   general. What do the NVMe Zoned Namespace and SCSI ZBC standards permit a device
   to do when a host reads above the write pointer in a sequential zone. Name the
   specific fields or bits that control this. Say clearly which parts you are sure
   of and which you are not.

4. If the filesystem instead put an explicit record index and record count inside
   each root record it writes, would that remove the dependence on the write pointer
   for defining the candidate set. What new failure mode would that introduce on an
   append only zone, given that the record count is not known when the first record
   in the zone is written.

5. Is there a reason the whole question disappears if the fixed structures simply
   never live in sequential zones. What would that cost, given that the root ring
   regions are placed at r times P times chunk with P equal to 8191, and that this
   product is never a multiple of any power of two zone size larger than the chunk.

OUTPUT

Start with a line saying how many of the five items you could construct something
for. Then take them in order. Be specific and give reasoning that could be checked.
