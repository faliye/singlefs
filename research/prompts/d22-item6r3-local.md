You are the counterexample leg of a design review for a copy-on-write filesystem.
Answer in English. Do not use any markdown emphasis.

CONTEXT

The filesystem stores its root pointer in a ring of root records. Each root record
carries a magic number, a filesystem id, an instance number, a checkpoint counter,
a self covering checksum of 32 bytes, and pointers to other units. A root record is
194 bytes and lives in a slot whose width is the physical block size.

Recovery is required to verify every candidate slot first, and only then pick the
newest valid one by generation. Picking the newest first and verifying afterwards is
forbidden, because a torn newest slot would then fail outright instead of falling
back to the previous generation.

On zoned block devices, sequential write required zones cannot be overwritten in
place. Each has a device maintained write pointer. Reads above the write pointer
return unwritten data. Resetting a zone returns it to a state that is byte identical
and state identical to a zone that was never written.

CANDIDATE R TO ATTACK

Candidate R says: define the candidate set as exactly those slots whose self
covering checksum verifies. Unwritten space fails the checksum automatically, so it
is simply not a candidate. This means no rule is needed to tell unwritten from
damaged, and the write pointer is not needed to bound the candidate set. The write
pointer therefore falls back to pure addressing, entering no judgement at all.

YOUR TASK

Attack candidate R. Give a concrete construction for each item, or say plainly you
cannot. Do not invent a weak objection.

1. Candidate R may be confusing two different things: deciding where to read, and
   deciding whether what you read counts. Membership in the candidate set has to be
   known before verification, because you must know which addresses to read. On an
   append only zone, is there any way to know which addresses to read without the
   write pointer. If there is not, candidate R has not solved anything.

2. Slot rotation assumes the same slot address is overwritten again on the next lap.
   A sequential zone forbids that. So does a fixed set of slot addresses even exist
   on a zoned device. If it does not, then the idea that recovery can simply read a
   known set of positions also fails.

3. The rule that a whole unit checksum plus a generation number are both needed
   exists because a record that was never written can read back as a complete and
   valid older record left at that position from a previous lap. On an append only
   zone that is never overwritten, can that stale record situation still arise. If it
   cannot, what does that imply.

4. Consider instead having each root record carry its own index within the zone and
   a count. Would that remove the dependence on the write pointer. What breaks, given
   that the count is not known when the first record of a zone is written.

5. Candidate R does not detect that a zone reset destroyed a committed generation,
   because reset leaves the zone indistinguishable from never written. Is that
   acceptable, or does it make candidate R unsafe to adopt at all.

OUTPUT

Start with a line saying how many of the five you could construct something for.
Then take them in order.
