1 capacity(d) 3932160 3932160 3932160 3932160 3932160 3932160
2 allocated(d) 1327104 1327104 1327104 1556480 1556480 1556480
3 unreclaimable(d) 0 0 0 0 0 0
4 deferred_pending_release(d) 1064960 1179648 0 1294336 1638400 0
5 mount_time_commitment(d) 1114112 1114112 1114112 1114112 1114112 1114112
6 abandoned_root_exclusive(d) 0 0 0 0 0 0
7 pending_delete_occupancy/replicas 0 0 0 0 0 0
8 committed_reservation/replicas 0 0 0 0 0 0
9 checkpoint_reserve_pool/replicas 81920 81920 81920 81920 81920 81920
10 available(d) 344064 229376 1409024 -114688 -458752 1179648
11 demand(d) 21 21 21 21 21 21
12 is row10 >= row11? yes 0 14 <21 no 7 -7 <21 no 28 86 >=21 yes 65 72 >=21 yes 51
For checkpoint P candidate A1 flips from admitted to refused. For checkpoint Q candidate A2 flips from refused to admitted. Candidate A2 makes available(d) positive for Q where today's code was negative.
This would be refuted by a demand of 14 bytes for checkpoint P.

1 publish 1 still in defer window publish 2 still in defer window publish 3 still in defer window publish 4 reclaimable
2 allocated(d) 50 50 50 50 50 50 50 50 50 50 50 50
3 deferred_pending_release(d) 20 20 0 20 20 0 20 20 0 0 0 0
4 available(d) -20 -20 0 -20 -20 0 -20 -20 0 20 20 20
5 demand(d) 20 20 20 20 20 20 20 20 20 20 20 20
6 is row4 >= row5? no -40 no -40 yes 0 no -40 no -40 yes 0 no -40 no -40 yes 0 yes 0 yes 0 yes 0
Retried write first admitted at publish 4 for all readings. Candidate A1 has no effect because the retried write does not supersede any content so Fact 11 does not apply.
This would be refuted by a scenario where the retried write is a rewrite of existing content.
