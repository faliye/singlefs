You are reviewing two competing designs for an encrypted copy-on-write filesystem. Your stance is: find a concrete counterexample. Do not summarize, do not praise. Do not use any markdown emphasis such as bold or italics. Answer in English. Answer all three items, number them 1, 2, 3. Keep each item under 250 words.

Setting, all settled. The whole volume is encrypted; the only plaintext is the superblock plus one layer whose exact shape is the question here. That layer exists so a mount without the key can still move blocks, because continuous background defragmentation is a hard promise. A keyless mount may move blocks but may not free them, and may only allocate inside a lease area signed by the key side. Mounting requires the key. A pointer carries one location entry per replica, and the width floor is two replicas from the first release. Data unit headers carry a plaintext identity; metadata unit identities are ciphertext. Authoritative state is units plus accounting plus roots.

Candidate A, a plaintext mapping table. One row per replica: key is logical identity of thirty two bytes plus a one byte replica index; value is a location entry of device four bytes, slot number six bytes, ciphertext checksum four bytes. A keyless mount that moves a replica rewrites that row. On unlock a checker rebuilds the table from authoritative state and compares row by row.

Candidate B, a plaintext move log. Nothing maps logical identity to location in plaintext at all. A keyless mount that moves a unit writes the new copy, then appends a thirty two byte record: source device and slot, destination device and slot, the first eight bytes of the moved unit's header checksum, and a sequence number. The source is not freed. The log ring and the lease area are both covered by a signature made by the key side. On unlock the key side replays the log in sequence order; for each record it checks that the destination holds a unit whose header checksum matches, that the unit decrypts and its authentication tag verifies, and that its logical identity equals what the authoritative tree says lives at the source; only then does it rewrite the authoritative pointer to the destination and retire the source. A record that fails any check is discarded on its own; the source stays authoritative.

Your task:

1. Attack candidate B's replay. Construct a history where the same source location is moved twice in one lock period, A to B then B to C, and the middle record is discarded. Say what the replay produces and which rule it breaks. Then construct a history where the replay itself is interrupted halfway and the volume is mounted again.

2. Attack candidate B's claim that a keyless mount needs no lookup structure. Name a background job that must run while the volume is locked and that needs to know something about a block other than its physical address. If you cannot name one, say so and say what you checked.

3. Attack both candidates on the same point: a keyless mount moves one replica of a two replica unit, and then the machine crashes before anything else happens. Say, for each candidate, what state the disk is in and whether the next keyless mount can tell what happened.

For each item give the history as numbered steps, the rule text it breaks, and the smallest change that would close it. If you cannot construct one, say so and say which step blocked you.
