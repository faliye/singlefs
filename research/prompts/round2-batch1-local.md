You are the counterexample hunter on a three way review of a copy on write filesystem design. Your only job is to find concrete counterexamples and errors. Do not use any markdown emphasis such as asterisks. Write plain sentences. Be short and specific.

There are two separate claims. Treat them independently.

CLAIM ONE. The design has a per block pointer that is split into two segments. A header segment carries the encryption nonce and the authentication tag, one copy per logical extent. A location segment carries the physical address plus a plain checksum over the ciphertext, one copy per physical location. The stated purpose of that plain ciphertext checksum is that a process without the encryption key can still detect media corruption. The design also states that the pointer as a whole is stored inline in the parent index node, and that every index node payload is encrypted. The design separately lists that same ciphertext checksum among the plaintext side fields. An invariant in the design says: when encryption is on, for any referenced block, the ciphertext checksum recorded in its pointer matches the ciphertext on disk, and checking this does not require the key.

The claim under review is that these statements cannot all be true at once. If the location segment lives inside an encrypted parent node, then a process without the key cannot read the checksum, so the invariant cannot be checked without the key. Therefore the location segment must actually live somewhere outside the encrypted index, which means it is stored twice or stored elsewhere, and the design cost table that counts 108 bytes of per unit metadata, including 22 bytes for two location entries, never accounted for that second copy.

Question 1a. Give a concrete layout that satisfies all the stated statements at once, if one exists. If you think none exists, say which specific statement has to give.

Question 1b. Give at least one counterexample or exception where the reasoning above breaks down. For instance, is there a way for a keyless reader to obtain the checksum without decrypting the parent node.

CLAIM TWO. The design keeps a small ring of root records. Two separate rules forbid reusing recently freed blocks.

Rule one, called the root ring rule: blocks referenced by any of the last K root generations must not be reallocated. In this design one root is published per fsync, so the last K root generations correspond to the last K fsyncs.

Rule two, called the replay window rule: blocks freed anywhere inside the current journal replay window must not be reallocated. The journal tail only advances after a checkpoint is durable, so the replay window covers every fsync since the last checkpoint.

The design contains an argument that K must be a runtime tunable rather than a format constant, and the stated reason is that when the disk is nearly full, the space most needed for rescue is exactly the space held by K, and that lowering K is the only escape hatch from an out of space deadlock.

A model was run. With a checkpoint every 1000 fsyncs and 8 blocks freed per fsync, lowering K from its current value to the floor of 2 frees blocks only during the first K minus 1 phases after a checkpoint completes. With K equal to 4 that is 3 phases out of 1000, and it frees at most 16 blocks, while the replay window rule is holding about 4004 blocks at the same time. A positive control with a checkpoint after every fsync showed that lowering K then does free 16 blocks, so the measurement does respond when K is the binding rule.

Question 2a. Is the phase counting correct. Give your own derivation of how many phases in a checkpoint cycle the root ring rule holds strictly more blocks than the replay window rule.

Question 2b. Give at least one concrete counterexample: a workload, a checkpoint policy, or a device class where lowering K really would free a useful amount of space.

Question 2c. Is there any mechanism by which the journal tail could advance without a checkpoint completing. If yes, describe it, because it would change the answer.

Finally list any statement in the material above that you think is wrong or unverified, and say why.
