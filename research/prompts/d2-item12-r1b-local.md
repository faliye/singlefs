You are reviewing the first version of a design proposal for a copy-on-write filesystem. Your stance is: find a concrete counterexample. Do not summarize, do not praise. Produce reachable histories (numbered steps, with device numbers, slot numbers, stripe widths and checkpoint numbers) after which a stated rule gives a wrong answer or cannot be evaluated. Do not use any markdown emphasis such as bold or italics. Answer in English. Keep each item under 250 words.

Background, all of it already settled in this project:

Stripes are variable width full stripe writes, never read modify write. Width w is clamp(batch size plus one, 2, 4), then clamped by the number of writable devices. A w wide stripe holds w minus 1 data grids and one parity grid. One grid equals one unit; a unit is never split across columns. There are two unit sizes, 16384 for metadata and 32768 for data. A block pointer carries one location entry per replica, fixed width 14 bytes, being device id 4, 16 KiB slot number 6, ciphertext checksum 4. A location entry records only its own column. Physical positions are always listed explicitly, never computed from a formula; a chunk mapping table was rejected as a second authoritative structure.

Allocation records live in their own keyspace btree, key is device id 4 plus 16 KiB slot number 6, value is allocation or release generation 8 plus a span field 2 whose top bit means released. One record covers one storage unit. The allocation tree is derived state: the whole tree can be rebuilt from units plus accounting plus roots. Authoritative state is units plus accounting plus roots; indexes are derived.

Unit headers have a common 42 byte prefix: magic 4, format version 2, flags 2 where offset 6 is the unit class code, declared length 2, header checksum 32. Class code 1 is a data unit, 32768, header 105. Class 2 is an index node, 16384. Class 3 is a packed record container, 32768, header 103, carrying records of a second level type registry where types 1 to 4 are taken and 5 is free. Codes 4 to 15 are reserved for future content classes and adding one costs an incompatible feature bit. A unit header may not carry physical location or device id, because physical position never enters the authenticated data of the cipher, so such a field would be unauthenticated and silently forgeable.

After a member of a w at least 3 stripe is released, that grid and the whole stripe including parity may not be reused until repair completes. Repair means a background job re lands the survivors as one new stripe and reclaims the old stripe whole. Measured: if reuse is allowed, a single device failure loses on average 9.8 to 13.0 live units with no error reported before the device dies.

The first release runs two devices, so w is always 2 and the parity grid is just the second replica, already discoverable from the pointer. The problem below only exists for w at least 3.

Garbage sweep admission has a conjunct that is common to both of its branches: the location is not in the allocator in flight overlay, and the header is readable.

The proposal, version one:

P0. A lemma. Let S be the grid size. If the parity grid reserves H bytes for a self describing header, then parity covers only S minus H bytes of each member. When one grid is lost, those H bytes need another source. Storing them verbatim needs w minus 1 times H bytes while only H bytes are available, and that holds only when w is at most 2. So for w at least 3 a parity grid cannot carry any self describing byte, unless every member unit reserves an H byte region whose value is known a priori, taken as all zero. That reservation would cost every unit H bytes of payload forever, including in the two device first release where no parity grid exists at all.

P1. Criteria for whatever carries the stripe member table: lookup from a location to the member table must be a table lookup, never a scan, and its cost must not grow with device capacity, because the allocator asks it at the moment a location is released; the carrier must itself be a self describing unit so that a full scan rebuild can recreate the table; it must not change any settled byte; and it must be readable without the encryption key, because a keyless mount is required to be able to move data, and repair is a move.

P2. The proposal: a stripe member record is a packed record of type 5 inside a class 3 container. One record per member location, and each record carries the member table of the whole stripe. The containers live in a new keyspace, keyed by member location, device id 4 plus 16 KiB slot number 6, the same coordinate system as allocation records. The record is fixed at 56 bytes: grid size flag 1, w 1, my column index 1, member table 4 entries of device 4 plus slot 6 equals 40, stripe birth checkpoint 8, padding 5. The parity grid is always the last entry of the member table. Only stripes with w at least 3 write records.

P3. Two patches that must land with it. First, a parity grid has no readable header, so it can never satisfy the sweep admission conjunct and would leak forever; the conjunct becomes header readable or named as the parity grid by some stripe record. Second, an explicit exception to the project rule that every metadata block must be self describing: the parity grid is the only location in this filesystem that is not self describing, and its identity is supplied by the stripe records.

P4. Known weak point: a stripe record container is itself a unit and itself lands in a stripe. If a container is lost with a failed device, rebuilding it needs its own stripe member table, which lives in the records of its sibling members, since every record carries the whole table. So one surviving member record is enough, unless two containers name each other. An alternative patch is to force stripe record containers to always use w equal 2, mirrored and self describing.

P5. All grids of one stripe must have the same size, 16384 or 32768, otherwise the exclusive or is undefined; the allocator batches by grid size.

Your task. Answer all three items. Number them 1, 2, 3. Do not stop after the first one.

1. Attack the release path. Construct a history where the allocator releases a location, asks the stripe table for the member table, and gets an answer that is wrong, stale or missing, so that one grid is reused while a sibling of the same stripe is still live. Note that the stripe table is itself a copy on write tree published with the same checkpoint as the stripe.

2. Attack the rebuild path. After a total index loss a scan finds class 3 containers and rebuilds the stripe table from them. Construct a history where two readable stripe records name the same member location, one from the old stripe and one from the stripe the background repair re landed it into, and the scan cannot decide which is current.

3. Attack the bootstrap. A stripe record container is itself a unit and itself lands in a stripe. Construct the smallest concrete placement, with device numbers and slot numbers, of containers and stripes in which one device failure leaves a set of lost grids that cannot be rebuilt in any order.

For each item give the history as numbered steps, the rule text it breaks, and the smallest change that would close it. If you cannot construct one, say so and say which step blocked you.
