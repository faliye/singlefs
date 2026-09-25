1 capacity(d) 3932160 3932160 3932160
2 allocated(d) 1327104 1327104 1327104 1556480 1556480 1556480
3 unreclaimable(d) 0 0 0 0 0 0
4 deferred_pending_release(d) 1064960 1294336 1064960 1294336 1294336 1294336
5 mount_time_commitment(d) 1114112 1114112 1114112 1114112 1114112 1114112
6 abandoned_root_exclusive(d) 0 0 0 0 0 0
7 pending_delete_occupancy / replicas 0 0 0 0 0 0
8 committed_reservation / replicas 0 0 0 0 0 0
9 checkpoint_reserve_pool / replicas 81920 81920 81920 81920 81920 81920
10 available(d) 344064 114688 1409024 -114688 not determined by the given facts my estimate is -7 - X where X is superseded blocks size for sixth overwrite because the size of superseded blocks for sixth overwrite is not specified in the facts 1179648
11 demand(d) 3 3 3 3 3 3
12 is row 10 >= row 11 yes by 18 slots yes by 4 slots yes by 83 slots no by 10 slots not determined by the given facts yes by 69 slots
checkpoint P candidate A1 leaves outcome admitted checkpoint Q candidate A2 flips outcome from refused to admitted
This would be refuted by: if the demand(d) for checkpoint Q is 73 slots or more under candidate A2, but given the assumption of demand=3 slots, a demand of 73 slots would refuteail the admission despite available(d)=72 slots.

1 at this check the 20 deleted slots are still inside defer window for publish 1 2 3 and reclaimable at publish 4 because Fact 16 states 3 publishes changing user-visible state after delete are required before reclaimable
2 allocated(d) 50 50 50 50 30 30 30 30
3 deferred_pending_release(d) 20 20 20 20 0 0 0 0
4 available(d) -20 -20 -20 -20 20 20 20 20
5 demand(d) 20 20 20 20 20 20 20 20
6 is row 4 >= row 5 no by 40 slots no by 40 slots no by 40 slots no by 40 slots yes by 0 slots yes by 0 slots yes by 0 slots yes by 0 slots
retried write is first admitted at publish 4 for all three readings
candidate A1 has no effect because the retried write is fresh content (does not supersede existing content) so Fact 11 does not apply
This would be refuted by: if the number of publishes required to reclaim the deleted block is not 3 but 2 or 4, changing the publish number at which the retried write is admitted.
