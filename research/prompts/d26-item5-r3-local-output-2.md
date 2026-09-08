Clause 1 breaks. Timeline: Scrub watermark set to 100. Unit created at TXG 50 with birth 50. Compaction relocates it to new physical location but birth remains 50. Scrub skips all units with birth ≤ 100, including both the old and new copies. Corrupted checksum in the new copy goes undetected because the scrub never walks it despite it being live.

Clause 2 breaks. Timeline: TXG 10 create unit U (tuple T, birth 10) at (dev1, off1). TXG 15 head prev_snap_txg=15. TXG 20 compaction frees old location; birth 10 ≤ 15 so added to head deadlist. TXG 30 take snapshot S; S's deadlist contains (T, birth 10). TXG 40 compaction moves U to (dev1, off2), updating map to point to off2. TXG 50 destroy S; deadlist entry (T, birth 10) looks up map → off2, then frees off2 which is still live → double free.

Clause 2a breaks. Timeline: Compaction relocates 1,000,000 units. Each relocation requires writing 1KB data, 40B map entry, and 32B allocation records (source and destination). Total transaction size ≈ 1,096,000,000 bytes. System transaction size limit is 1GB; transaction exceeds limit and fails, preventing compaction.

Clause 3 is consistent under second encoding. Deadlist entry (41 bytes: 33-byte tuple + 8-byte birth) requires searching the chain of versions for the birth value. The chain length has no upper bound but background work exemption allows slow lookups. No counterexample exists.

Clause 4 breaks. Timeline: Disk has 1000 blocks total with 990 used. Compaction attempts to relocate 100 units. Destination allocation requires 100 free blocks but only 10 available. Compaction fails due to insufficient space, and no allocation policy exists to handle this scenario.
