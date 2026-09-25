Item 1. The 256-slot pool of Fact 16, at k = k_max = 6 (the case where the very next mount is already refused).

Row A1. 256
Row A2. 63 (estimate: 13 + 5*10 = 63, Fact 15 approximate growth rate)
Row A3. 0
Row A4. 0
Row A5. rows0=0, pages_of_chain=ceiling((0+3)/369)=1, chain_rewrite=2, warm_up=5*3=15, one_switch=17, instance_switch_reserve=17*4=68
Row A6. 0
Row A7. 0
Row A8. 0
Row A9. allocation-record tree height=2, central-mapping tree height=1, accounting tree node count=1, tree table=1; total 2+1+1+1=5; central-mapping height and accounting node count unchanged because 10 entries fit in one leaf (Fact 13: 294 max entries per leaf), and 15 rows fit in one node (Fact 14)
Row A10. 5
Row A11. 256 - 63 - 0 - 0 - 68 - 0 - 0 - 0 - 5 = 120
Row A12. 2 (data unit) + 1 (extent-tree node below root) + 1 (extent-tree root) + 2 (inode leaf container) + 1 (inode-tree root) = 7
Row A13. yes, 113

Row B1. 256
Row B2. 73 (assumption: 63 + 10, Fact 15 growth rate per overwrite)
Row B3. 0
Row B4. 0
Row B5. rows0=1, pages_of_chain=ceiling((1+3)/369)=1, chain_rewrite=2, warm_up=5*3=15, one_switch=17, instance_switch_reserve=17*4=68; ckpt_cost unchanged (Fact 13: trees not grown after 6 overwrites)
Row B6. 0
Row B7. 0
Row B8. 0
Row B9. 5
Row B10. 5
Row B11. 256 - 73 - 0 - 0 - 68 - 0 - 0 - 0 - 5 = 110
Row B12. 0 (Fact 17: mount-time demand is always 0)
Row B13. yes, 110

The calculation shows available(d) of 110 slots, which is sufficient for demand 0, but Fact 16 states refusal. The smallest additional real consumption needed is 111 slots added to allocated(d) (B2), which is not accounted for by any fact in the task, as the observed growth rate is only ~10 slots per overwrite.

This would be refuted by: a computation where allocated(d) after 6 overwrites is 184, causing available(d) to be negative.

Item 2. The 384-slot pool of Fact 16, at k = k_max - 1 = 9 (the case where the next mount is admitted but the one after that, with nothing written in between, is refused).

Row A1. 384
Row A2. 103 (estimate: 13 + 9*10 = 103, Fact 15 approximate growth rate)
Row A3. 0
Row A4. 0
Row A5. rows0=0, pages_of_chain=ceiling((0+3)/369)=1, chain_rewrite=2, warm_up=5*3=15, one_switch=17, instance_switch_reserve=17*4=68
Row A6. 0
Row A7. 0
Row A8. 0
Row A9. allocation-record tree height=2, central-mapping tree height=1, accounting tree node count=1, tree table=1; total 2+1+1+1=5; central-mapping height and accounting node count unchanged (Fact 13: 294 max entries per leaf, 15 rows fit in one node)
Row A10. 5
Row A11. 384 - 103 - 0 - 0 - 68 - 0 - 0 - 0 - 5 = 208
Row A12. 7
Row A13. yes, 201

Row B1. 384
Row B2. 113 (103 + 10, Fact 15 growth rate)
Row B3. 0
Row B4. 0
Row B5. rows0=1, pages_of_chain=ceiling((1+3)/369)=1, chain_rewrite=2, warm_up=5*3=15, one_switch=17, instance_switch_reserve=17*4=68; ckpt_cost unchanged
Row B6. 0
Row B7. 0
Row B8. 0
Row B9. 5
Row B10. 5
Row B11. 384 - 113 - 0 - 0 - 68 - 0 - 0 - 0 - 5 = 198
Row B12. 0
Row B13. yes, 198

Row C2. 130 (113 + 17, mount 2's real cost one_switch=17)
Row C5. rows0=2, pages_of_chain=ceiling((2+3)/369)=1, chain_rewrite=2, warm_up=5*3=15, one_switch=17, instance_switch_reserve=17*4=68
Row C9. 5
Row C10. 5
Row C11. 384 - 130 - 0 - 0 - 68 - 0 - 0 - 0 - 5 = 181
Row C12. 0
Row C13. yes, 181

The calculation shows available(d) of 181 slots, which is sufficient for demand 0, but Fact 16 states refusal. The smallest additional real consumption needed is 182 slots added to allocated(d) (C2), which is not accounted for by any fact in the task, as the observed growth rate is ~10 slots per overwrite and 17 per mount.

This would be refuted by: a computation where allocated(d) for mount 3 is 312, causing available(d) to be negative.

Item 3. History A (Fact 26): the witness table's entry count after each of the 30 rollbacks.

t=1: instance table has no rows. Backfill adds nothing. Deletion rules remove nothing. Capacity check count=0, admitted. Add (1,0,0). End count=1.
t=2: instance table has one row (position 0, txg 0). Backfill adds nothing (entry (1,0,0) exists). Deletion rules: no deletions. Capacity check count=1, admitted. Add (2,0,0). End count=2.
t=3: instance table has one row. Backfill adds nothing. Deletion rules: no deletions. Capacity check count=2, admitted. Add (3,0,0). End count=3.
t=4: instance table has one row. Backfill adds nothing. Deletion rules: no deletions. Capacity check count=3, admitted. Add (4,0,0). End count=4.

t | count before step | admitted/refused | end count
1 | 0 | admitted | 1
2 | 1 | admitted | 2
3 | 2 | admitted | 3
4 | 3 | admitted | 4
5 | 4 | admitted | 5
6 | 5 | admitted | 6
7 | 6 | admitted | 7
8 | 7 | admitted | 8
9 | 8 | admitted | 9
10 | 9 | admitted | 10
11 | 10 | admitted | 11
12 | 11 | admitted | 12
13 | 12 | admitted | 13
14 | 13 | admitted | 14
15 | 14 | admitted | 15
16 | 15 | admitted | 16
17 | 16 | admitted | 17
18 | 17 | admitted | 18
19 | 18 | admitted | 19
20 | 19 | admitted | 20
21 | 20 | admitted | 21
22 | 21 | admitted | 22
23 | 22 | refused | 22
24 | 22 | refused | 22
25 | 22 | refused | 22
26 | 22 | refused | 22
27 | 22 | refused | 22
28 | 22 | refused | 22
29 | 22 | refused | 22
30 | 22 | refused | 22

Closed-form rule: count before step t = min(t-1, 22); end count = min(t, 22). Fact 22 (capacity check) keeps count from growing to 23. Fact 20 or 21 failing would allow count to grow to 23.

This would be refuted by: a computation where at t=23, the witness table count before step is 22, but deletion rules remove an entry, allowing admission and increasing count to 23.

Item 4. History B (Fact 27): the witness table's entry count after each of the 30 rollbacks.

t=1: instance table has no rows. Backfill adds nothing. Deletion rules remove nothing. Capacity check count=0, admitted. Add (1,0,0). End count=1.
t=2: instance table has one row (position 0, txg 0). Backfill adds nothing (entry (1,0,0) exists). Deletion rules: no deletions. Capacity check count=1, admitted. Add (2,1,1). End count=2.
t=3: instance table has two rows (positions 0 and 1, txg 0 and 1). Backfill adds nothing (entries (1,0,0) and (2,1,1) exist). Deletion rules: no deletions. Capacity check count=2, admitted. Add (3,2,2). End count=3.
t=4: instance table has three rows (positions 0,1,2, txg 0,1,2). Backfill adds nothing. Deletion rules: no deletions. Capacity check count=3, admitted. Add (4,3,3). End count=4.

t=23: instance table has 22 rows (positions 0-21). Backfill adds nothing. Deletion rules: check for covered-deletion. No entry covers another (all (t, t-1, t-1) with strictly increasing t). Capacity check count=22, admitted. Add (23,22,22). End count=23.
t=24: instance table has 23 rows (positions 0-22). Backfill adds nothing. Deletion rules: check Fact 20 for entry (23,22,22). [22,23) range. Root ring has generations 0-23. Generation 22 is in ring (g=22: region 22 mod 3=1, slot 22 div 3 mod 8=7). So root with instance 22 exists in ring. Thus deletion rule (i) does not apply. Covered-deletion: no covering entries. Capacity check count=23, refused. No new entry added. End count=23.
t=25 to 30: same as t=24. Count remains 23, refused.

t | count before step | admitted/refused | end count
1 | 0 | admitted | 1
2 | 1 | admitted | 2
3 | 2 | admitted | 3
4 | 3 | admitted | 4
5 | 4 | admitted | 5
6 | 5 | admitted | 6
7 | 6 | admitted | 7
8 | 7 | admitted | 8
9 | 8 | admitted | 9
10 | 9 | admitted | 10
11 | 10 | admitted | 11
12 | 11 | admitted | 12
13 | 12 | admitted | 13
14 | 13 | admitted | 14
15 | 14 | admitted | 15
16 | 15 | admitted | 16
17 | 16 | admitted | 17
18 | 17 | admitted | 18
19 | 18 | admitted | 19
20 | 19 | admitted | 20
21 | 20 | admitted | 21
22 | 21 | admitted | 22
23 | 22 | admitted | 23
24 | 23 | refused | 23
25 | 23 | refused | 23
26 | 23 | refused | 23
27 | 23 | refused | 23
28 | 23 | refused | 23
29 | 23 | refused | 23
30 | 23 | refused | 23

Closed-form rule: count before step t = t-1 for t <=23, 23 for t >23; end count = t for t <=23, 23 for t >23. Fact 21 never removes any entry because each new entry (t, t-1, t-1) has no other entry (Nb, rb, Tb) with Nb >= t and (rb, Tb) <= (t-1, t-1).

This would be refuted by: a computation where at t=24, a root with instance number 22 is no longer in the root ring, causing deletion rule (i) to apply to entry (23,22,22), reducing count to 22 and allowing admission.
