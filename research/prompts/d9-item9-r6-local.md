You are reviewing a design for an encrypted copy-on-write filesystem. Your stance is: find a concrete counterexample. Do not summarize, do not praise. Do not use any markdown emphasis such as bold or italics. Answer in English. Answer all three items, number them 1, 2, 3. Keep each item under 250 words.

Setting, all settled today and not open for debate. The whole volume is encrypted and mounting requires the key. Before locking, the key side signs a lease area: a set of physical slots a later keyless mount may write into. The lease area now carries its own keyless page checksums, so the keyless side can detect media faults in it on its own, and its size is booked as promised reservation rather than as allocated space. The keyless mount may move units but may never free anything. A stripe membership table now exists on disk: it lives in packed record units, is reached through a container index, and its containers are always written as two mirrored copies. The location entry inside a pointer is now only a hint; the authoritative location lives in one central mapping, and every dereference and every release decision goes through that mapping.

The design under review. The lease area may only take slots that pass six conjuncts: the ordinary allocation test, plus not being a live unit, not in the root ring, not in the journal ring, not a reservation, not awaiting reuse inside the replay window, and, new today, not being a pinned cell nor belonging to any stripe that still has a live member. That last conjunct is answered by one point lookup into the stripe membership table.

Your task:

1. The new conjunct needs one point lookup into the stripe membership table, which is reached through a container index. Ask what points at the container index itself. Construct a history where the lease area is being signed and that lookup cannot be completed, and say what the signer does then.

2. Unused lease slots are booked as promised reservation and must return to the free pool at unlock. Construct a crash that happens after unlock has begun and before the return completes, and say what the next mount sees and whether the space is recoverable.

3. A disk dies during the lock period. The keyless mount rebuilds a lost column of a three column stripe from its two surviving siblings and writes the rebuilt column into the lease area. At unlock one page of the lease area fails its signature, so the moves it covered are not accepted. Say what happens to the rebuilt column, and whether anything can tell that the column was a rebuild rather than an ordinary move.

For each item give the history as numbered steps, say which rule it breaks, and give the smallest change that would close it. If you cannot construct one, say so and say exactly which step blocked you.
