1
50432
53
0
0
not determined by the given facts, my estimate is not possible, because rows0 is not specified in the facts
0
0
0
5
5
not determined by the given facts, because instance_switch_reserve is unknown
10
not determined by the given facts, because available(d) depends on unknown instance_switch_reserve
This would be refuted by: a specific value for rows0 that results in available(d) being less than 10 slots.

2
50560
113
0
0
68
0
0
0
5
5
50374
0
yes, by 50374 slots
This would be refuted by: a calculation showing that demand(d) for the mount is greater than 0 or that available(d) is negative.

3
3
1
1
1
6
98304
Included: allocation-record tree height, central-mapping tree height, accounting tree node count, tree table. Excluded: instance-table chain because Fact 6 explicitly states it is not part of ckpt_cost.
This would be refuted by: evidence that the instance-table chain is included in ckpt_cost or that the tree table contribution is not 1.

4
allocated(d) = 13 + 10 * N; unreclaimable(d), deferred_pending_release(d), abandoned_root_exclusive(d), pending_delete_occupancy/replicas, committed_reservation/replicas all fixed at 0; checkpoint_reserve_pool/replicas fixed at 5; instance_switch_reserve(d) fixed at 68 for N < 365; available(d) = 50560 - (13 + 10 * N) - 68 - 5 = 50474 - 10 * N; smallest N where available(d) < 0 is 5048.
This would be refuted by: a concrete calculation showing available(d) >= 0 for N = 5048.
