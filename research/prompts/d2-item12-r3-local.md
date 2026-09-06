You are reviewing a design proposal for a copy-on-write filesystem. Your stance is: find a concrete counterexample. Do not summarize, do not praise. Do not use any markdown emphasis such as bold or italics. Answer in English. Answer all three items, number them 1, 2, 3. Keep each item under 250 words.

Setting, all settled in this project. A stripe of width w holds w minus 1 data grids and one parity grid; one grid is one unit, either 16384 or 32768 bytes, and all grids of one stripe are the same size. Width w is clamp(batch size plus one, 2, 4), then clamped by the number of writable devices. When w is 2 the second grid is a byte identical replica, not a computed parity, and a block pointer lists one location per replica, so a mirror needs no extra bookkeeping. For w at least 3 nothing on disk says which grids belong to one stripe or where the parity grid is; that table is needed when a lost grid must be rebuilt, when a location is released and the allocator must refuse to reuse the other grids until repair completes, and after a total index loss when everything is rebuilt by scanning the devices.

The proposal, third version. The stripe member table is authoritative state, so it lives in the payload of units, not in an index and not in a block header. The carrier is a packed record of a new type inside a class 3 container; one record per member location, each record carrying the whole member table of its stripe; the containers live in one pool wide keyspace. Three rules were added after two rounds of attack:

R1. The whole chain from the root record down to the containers always uses w equal 2, that is mirroring. Three reasons: a w equal 2 stripe writes no member records at all, so the dependency graph has no edges and the bootstrap problem disappears; writing a container no longer adds to any batch size, so the width formula stops being a fixed point; and the two mirror copies are byte identical, so a scan does not see them as two versions. A new allocator invariant says the two copies must land on different devices.

R2. A record is never deleted; it is rewritten as retired with a retirement generation, and one byte of state carries retired, pinned, and repairing bits.

R3. Every parity grid gets its own allocation record entry, so garbage sweep uses the has-an-entry branch and never needs to read the parity grid's header, which does not exist.

R4. Publish order is fixed in three segments: first all w grids of a stripe are written successfully, then the w member records are written by copy on write of their containers, then the root is published.

R5. When rebalancing or repair moves a stripe: write the new records, then change the pointers, then mark the old records retired. On interruption the pointer decides which record is current, and one location may have at most one non retired record.

Your task:

1. Attack R1. Construct a history in which the mirrored chain still cannot be recovered after one device failure. Note that the two copies must be on different devices, and that the container index above the containers is itself a tree whose internal nodes are ordinary units. Give device numbers and slot numbers.

2. Attack R3 together with R5. Construct a history in which a parity grid's allocation record entry says one thing and the member record says another, and show which rule then gives a wrong answer. Consider that allocation records are derived state, rebuildable from units plus accounting plus roots, while a parity grid has no header at all.

3. Attack R4. Construct a history where a crash between the second and the third segment leaves member records on disk describing a stripe whose root was never published, and say how a later scan tells those records apart from records of a stripe that was published.

For each item give the history as numbered steps, the rule text it breaks, and the smallest change that would close it. If you cannot construct one, say so and say which step blocked you.
