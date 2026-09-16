Whether the recovery of a copy-on-write file system can tell where a publish ends in its journal, and whether three proposed fixes hold.

Answer in English. Do not use any markdown emphasis: no bold, no italics, no headings with asterisks. Use plain numbered answers.

Facts from the design (treat them as given):

F1. The journal is a fixed ring of 4 KiB records on two mirrored disks. A record carries instance id, counter, checkpoint_txg, transaction number, a commit flag, a back-chain CRC of the previous record of the same instance, and a new-root segment that is the root of the whole publish it belongs to. A transaction may span several records; the commit flag is per transaction.
F2. Every publish increments checkpoint_txg by one. One publish may contain several transactions, so several records share one checkpoint_txg.
F3. Write order of a publish: units, barrier, journal records, barrier, root slot with FUA. Crash states enumerate every whole-write subset inside a segment between barriers.
F4. Recovery applies only records above the chosen root's (instance, checkpoint_txg), strictly consecutive, only transactions whose commit flag is present, after verifying the units each record names. Rule six: the unit of applying is one publish; if the valid prefix stops between two transactions of one publish, that whole publish is not applied.
F5. After a write error inside a mount, the instance switches: a new instance id, the in-flight checkpoint is re-published, transactions already durable keep their data units, the fixed-point units are rewritten, and the failed attempt's allocations do not exist.
F6. The fsync of a publish returns only after its root slot is durable. Stepping back one root inside the same instance must be recoverable through journal replay; the first root of a new instance is a single point, one fault can lose it.
F7. First-round finding under test: at the tail of the chain, nothing in a record says whether its publish is complete, so rule six has no input. If recovery applies the tail, it installs the publish root, and a unit named only by a missing record may be unreadable or reused (the switch in F5 can reuse the old fixed-point slot with no disk fault). If recovery never applies the tail, losing the root slot loses acknowledged data.
F8. Three proposed discriminators: (A) record header fields "index within publish" and "records in publish"; (B) a convention that the publish's fixed-point units are named only in the last record of the publish; (C) at recovery, walk the tree from the new-root segment and require every unit born in that publish to be named by a readable record of that publish.

Questions:
1. Under F3, is the state "publish has two records, only the first is durable" reachable without any disk fault? Under F5, may the re-published fixed-point unit land on the old fixed-point unit's slot?
2. Does F6 say anything about two simultaneous faults (the root slot and the record the chosen root covers both unreadable)? If it does not, say so.
3. For each of A, B, C: give a state where it still decides wrongly, considering a record that holds the discriminator being unreadable, a unit being unreadable, two transactions of one publish interleaved, and two instance switches in one mount.
4. Which of A, B, C would you pick, and what single observation would change your pick? Do not decide by bytes saved.
