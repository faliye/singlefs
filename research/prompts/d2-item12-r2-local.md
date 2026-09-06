You are reviewing a design argument for a copy-on-write filesystem. Your stance is: find a fourth way out. Do not summarize, do not praise. Do not use any markdown emphasis such as bold or italics. Answer in English. Answer all three items, number them 1, 2, 3. Keep each item under 250 words.

Setting. A stripe of width w holds w minus 1 data grids and one parity grid. A grid is one unit, either 16384 or 32768 bytes. A block pointer records one location per replica, device id 4 bytes plus 16 KiB slot number 6 bytes plus checksum 4 bytes, and it records only its own column. For w at least 3 nothing on disk says which grids belong to one stripe or where the parity grid is. That table is needed at three moments: when a read fails and the lost grid must be rebuilt; when a location is released and the allocator must refuse to reuse the other grids of that stripe until repair; and after a total index loss when everything is rebuilt by scanning the devices.

A first round of argument produced three boundaries, each from a settled rule of the project:

Boundary one. The member table cannot live in any unit header. The settled rule forbids physical location and device id in a block header, because physical position never enters the authenticated data of the cipher, so such a field would be unauthenticated and silently forgeable.

Boundary two. The member table cannot live only in pointers or index nodes. Indexes are derived state and are exactly what a scan rebuild has lost.

Boundary three. The member table cannot live in ciphertext. A keyless mount must still be able to move data and repair, and the plaintext exemption list has only two entries: the superblock, and one plaintext layer that maps logical identity to physical address plus a plaintext reverse index.

The first round concluded that the only remaining home is that plaintext mapping layer, whose field list and state class are both undecided, and therefore this question is blocked.

Also settled: parity is computed over ciphertext; two replicas must be byte identical, that is the same ciphertext written twice; a unit header starts at byte zero of the unit and the scanner steps by 16384; the checksum in a pointer covers the whole unit including its header; a released location keeps its allocation record, rewritten as released with a release generation; allocation records are derived state, rebuildable from units plus accounting plus roots.

Your task:

1. Find a fourth home for the member table that none of the three boundaries excludes. Say exactly which bytes it occupies, who writes them and when, how a lookup from one location to the member table works without scanning, and how a scan rebuild recovers it. If your answer is a structure that already exists in the list above, say which one and why it is not excluded.

2. Attack boundary three from a different direction: show that the keyless side does not in fact need the member table, by naming what a keyless mount is allowed to do. If a keyless mount may not publish a new stripe at all, then repair is not a keyless operation and boundary three collapses. Say what would have to be true for that argument to hold, and whether anything in the setting above contradicts it.

3. Consider abolishing the problem: cap w at 2 forever, so every stripe is a mirror, the second grid is a full self describing replica, and the pointer already lists both locations. State the strongest argument against this that does not appeal to space efficiency, since this project rules that saving space is never a criterion.

For each item, if you cannot construct an answer, say so and say which step blocked you.
