1. When prefix end is read as the counter covered by R_old, the rollback's record does not overwrite a record the normal recovery would need. When prefix end is read as the prefix end of the normal recovery of that mount, the rollback's record does overwrite a needed record.

2. The strongest recovery scans the ring, validates records, chooses the newest readable root (instance, txg), then for replay looks for the smallest checkpoint_txg > root's txg with the same instance, checks previous_hash matches the prior record's header. For both F9 findings, it does not find the correct chain head.

3. No. Two identical disk states: one where root is (i, T-1) and publish for T has a torn first record and durable second record; another where root is (i, T) and the publish for T has the same torn first and durable second record. The extra quantity is a transaction ID stored in the record header.

4. Yes. Shortest sequence: root record with instance i1 and counter X; next record with instance i2 and counter X+1. F10 applies the i2 record, which F4 forbids.

5. Add a transaction ID in the record header. The observation that changes the choice is whether a single publish can write multiple records.
