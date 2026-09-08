Step 1: Journal size is 2 entries. Initial state:
- Offset 0: Record A (instance-id=1, counter=0, checkpoint_txg=1, prev_hash=0)
- Offset 1: Record B (instance-id=1, counter=1, checkpoint_txg=2, prev_hash=CRC32C of Record A's header)
- Root slot points to Record B (instance-id=1, checkpoint_txg=2)

Step 2: New instance (instance-id=2) starts. Journal wraps:
- Write Record C at offset 0: instance-id=2, counter=0, checkpoint_txg=0, prev_hash=CRC32C of Record B's header
- Journal now: offset 0=Record C, offset 1=Record B (unchanged)

Step 3: Crash occurs before updating root slot to point to Record C

Step 4: Recovery:
- Root slot still points to Record B (instance-id=1, checkpoint_txg=2)
- Replay path checks records where (instance-id, checkpoint_txg) > (1, 2)
- Record C has instance-id=2 > 1 → passes lower-bound filter
- Back-chain check: Record C's prev_hash matches Record B's header → passes
- Record C is applied despite being from a different timeline (instance-id=2) during instance-id=1 recovery

This violates the requirement because a record from an abandoned timeline (instance-id=2) is applied when recovering instance-id=1, even though the generation number (instance-id) is part of the comparison. The lower-bound filter incorrectly allows records from higher instance-ids to be applied across timelines without validating timeline consistency. The back-chain check passes because it only verifies cryptographicord C's prev_hash against the physical previous record (Record B), not whether Record B belongs to the same timeline.
