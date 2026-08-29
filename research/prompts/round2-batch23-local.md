You are the counterexample hunter on a three way review of a copy on write filesystem design. Your only job is to find concrete counterexamples and errors. Do not use any markdown emphasis such as asterisks. Write plain sentences. Be short and specific. Answer every question separately.

There are three independent claims. Do not assume they are related.

CLAIM ONE. The index uses keys of the form (locality id, inode, offset). That key layout was settled on one date. Three days later a separate decision settled that the filesystem will support multiple writable heads, meaning several branches can each be written to independently, like writable snapshots. The claim is that the three part key is now incomplete, because two writable heads must each be able to hold their own version of the same (inode, offset), and with a three part key both versions land on the same key slot.

Question 1a. Is there any way to support multiple writable heads without adding a snapshot dimension to the key. Name a concrete mechanism if you can think of one.

Question 1b. Give a counterexample where two writable heads can safely share one key slot.

CLAIM TWO. Two different key orderings are being used in the same design. Ordering P puts the snapshot identifier at the highest sort weight, so all keys belonging to one snapshot are contiguous, which makes deleting a snapshot a contiguous range operation. Ordering L puts the snapshot identifier at the lowest sort weight, so all versions of one (inode, offset) are contiguous, which makes point lookup and ancestor filtering cheap. The claim is that P and L are mutually exclusive uses of the same sort weight and cannot both hold, and that one decision document cites the benefit of P while its own chosen candidate uses L.

Question 2a. Is the mutual exclusion real. If you think both can hold at once, describe the encoding that achieves it.

Question 2b. Give a counterexample: any encoding, any auxiliary structure, or any operation ordering that recovers most of the benefit of P while keeping L.

CLAIM THREE. A journal is a fixed length ring. Slot position is determined by the record sequence number modulo the number of slots. Recovery scans the whole ring, verifies each record, and accepts the longest prefix where each record sequence number equals exactly the expected next number, stopping at the first mismatch. After a crash that lost one record but landed later ones, recovery stops before the lost one, and the system then resumes writing from the sequence number where the prefix ended. The claim is that the records the discarded timeline left behind, which sit at later slots with valid checksums and sequence numbers that exactly equal what recovery will expect next, get replayed after a second crash, and that nothing in the record header can tell the two timelines apart.

Question 3a. Give at least one concrete mechanism already common in real journaling filesystems that would prevent this without adding a new header field.

Question 3b. The journal tail only advances after a checkpoint becomes durable. Does tail advancement remove the leftover records from consideration. Explain precisely when it does and when it does not.

Question 3c. Suppose every replay is idempotent, meaning applying a record twice equals applying it once. Does idempotent replay make replaying a discarded timeline record harmless. Answer yes or no and explain.

Finally list any statement above that you think is wrong or unverified, and say why.
