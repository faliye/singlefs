1 capacity(d) in bytes
- P: 3932160 3932160 3932160
- Q: 3932160 3932160 3932160

2 allocated(d) in bytes
- P: 1327104 1327104 1327104
- Q: 1556480 1556480 1556480

3 unreclaimable(d) in bytes
- P: 0 0 0
- Q: 0 0 0

4 deferred_pending_release(d) in bytes
- P: 1064960 1294336 0
- Q: 1294336 not determined by the given facts, my estimate is unknown, because the size of the superseded blocks for the sixth overwrite is not provided in the given facts 0

5 mount_time_commitment(d) in bytes
- P: 1114112 1114112 1114112
- Q: 1114112 1114112 1114112

6 abandoned_root_exclusive(d) in bytes
- P: 0 0 0
- Q: 0 0 0

7 pending_delete_occupancy / replicas in bytes
- P: 0 0 0
- Q: 0 0 0

8 committed_reservation / replicas in bytes
- P: 0 0 0
- Q: 0 0 0

9 checkpoint_reserve_pool / replicas in bytes
- P: 81920 81920 81920
- Q: 81920 81920 81920

10 available(d) in bytes
- P: 344064 114688 1409024
- Q: -114688 not determined by the given facts, my estimate is unknown, because the size of the superseded blocks for the sixth overwrite is not provided in the given facts 1179648

11 demand(d) in bytes
- P: 81920 81920 81920
- Q: 81920 81920 81920

12 is row 10 >= row 11
- P: yes, slack 262144 yes, slack 32768 yes, slack 1327104
- Q: no, shortfall 196608 not determined yes, slack 1097728

For checkpoint P, candidate A1 leaves it admitted; for checkpoint Q, candidate A2 flips refused to admitted. Candidate A2 at checkpoint Q crosses from negative to non-negative.

This would be refuted by: a measurement showing that the demand(d) for checkpoint Q was 1200000 bytes, which would exceed candidate A2's available(d) of 1179648 bytes, thus refusing the write even under candidate A2.

1 still in defer window or reclaimable
- publish 1: still in defer window
- publish 2: still in defer window
- publish 3: still in defer window
- publish 4: reclaimable set

2 allocated(d) in slots
- publish 1: 50 50 50
- publish 2: 50 50 50
- publish 3: 50 50 50
- publish 4: 30 30 30

3 deferred_pending_release(d) in slots
- publish 1: 20 20 20
- publish 2: 20 20 20
- publish 3: 20 20 20
- publish 4: 0 0 0

4 available(d) in slots
- publish 1: -20 -20 0
- publish 2: -20 -20 0
- publish 3: -20 -20 0
- publish 4: 20 20 20

5 demand(d) in slots
- publish 1: 20 20 20
- publish 2: 20 20 20
- publish 3: 20 20 20
- publish 4: 20 20 20

6 is row 4 >= row 5
- publish 1: no, shortfall 40 no, shortfall 40 no, shortfall 20
- publish 2: no, shortfall 40 no, shortfall 40 no, shortfall 20
- publish 3: no, shortfall 40 no, shortfall 40 no, shortfall 20
- publish 4: yes, slack 0 yes, slack 0 yes, slack 0

First admitted at publish 4 for all readings. Candidate A1 has no effect because the retried write is fresh content and not an overwrite, so Fact 11's change does not apply.

This would be refuted by: a scenario where the retried write was an overwrite, causing candidate A1 to add the superseded blocks into deferred_pending_release(d), which would affect the available(d) calculation, but Fact 18 explicitly states the retried write is fresh content, so this refutation is not applicable to the given scenario.
