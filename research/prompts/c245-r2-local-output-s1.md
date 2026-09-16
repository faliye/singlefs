1. Under the first reading (prefix end as counter covered by R_old), no. Under the second reading (prefix end as the previous instance's last counter), yes.

2. The strongest recovery uses F2 and F3: after selecting the newest readable root (instance i, txg T), it looks for records with instance i and checkpoint_txg T+1 where the previous_hash matches the root's header CRC. For the first F9 finding (rollback record overwrites last covered record), it correctly stops because the next record has a different instance. For the second F9 finding (last covered record unreadable), it fails to find the correct chain head because the next record's previous_hash is invalid, causing the chain to break and losing durable records.

3. No. Two byte-for-byte identical disk states: one where the journal has a valid root for txg T followed by a torn record for txg T+1 and a durable record for txg T+1 (where the durable record's previous_hash is invalid due to the torn record), and another where the journal has a root for txg T+1 with the same bytes (but no prior records). The extra quantity needed is the last covered counter stored in the root record.

4. Yes. Shortest sequence: root record (instance 1, counter 2, txg 0), followed by a record (instance 2, counter 3, txg 1). F10 sorts records above the root's watermark (counter > 2) by (instance, counter), starting with (2,3). F4 forbids applying records with a different instance, but F10 applies it.

5. Add the last covered counter in the root record. If the observation is that the root record is always the last record of its transaction, then the root's counter alone suffices and no extra quantity is needed.
