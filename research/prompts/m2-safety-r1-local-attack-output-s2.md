### Item 1: 256-slot pool at k = k_max = 6 (last overwrite before mount 2 refused)

**Row A1:** 256  
**Row A2:** 63 (estimate: Fact 15 states ~13 slots after first file, each overwrite adds ~10 slots; 5 prior overwrites: 13 + 5×10 = 63)  
**Row A3:** 0 (Fact 4: non-zoned → unreclaimable=0)  
**Row A4:** 0 (Fact 4: no deletions → deferred_pending_release=0)  
**Row A5:** 68 (rows0=0; pages_of_chain=ceiling((0+3)/369)=1; chain_rewrite=1×2=2; warm_up=5×3=15; one_switch=2+15=17; instance_switch_reserve=17×4=68)  
**Row A6:** 0 (Fact 5: no rollbacks → abandoned_root_exclusive=0)  
**Row A7:** 0 (Fact 4: pending_delete_occupancy=0)  
**Row A8:** 0 (Fact 4: committed_reservation=0)  
**Row A9:** 5 (allocation-record height=2; central-mapping height=1; accounting node count=1; tree table=1; 2+1+1+1=5; central-mapping and accounting trees unchanged as entries fit in single leaves per Fact 13)  
**Row A10:** 5 (Fact 6: checkpoint_reserve_pool/replicas=ckpt_cost)  
**Row A11:** 120 (256 - 63 - 0 - 0 - 68 - 0 - 0 - 0 - 5)  
**Row A12:** 2 (Fact 10: data unit=2 slots; no tree node growth per Fact 13)  
**Row A13:** yes, 118 (120 ≥ 2)  

**Row B1:** 256  
**Row B2:** 73 (assumption: Fact 15 growth rate applies; 63 + 10 = 73; demand(d) for overwrite becomes real allocated growth)  
**Row B3:** 0  
**Row B4:** 0  
**Row B5:** 68 (rows0=1; pages_of_chain=ceiling((1+3)/369)=1; chain_rewrite=2; warm_up=5×3=15; one_switch=17; instance_switch_reserve=17×4=68; ckpt_cost unchanged per Fact 13)  
**Row B6:** 0  
**Row B7:** 0  
**Row B8:** 0  
**Row B9:** 5  
**Row B10:** 5  
**Row B11:** 110 (256 - 73 - 0 - 0 - 68 - 0 - 0 - 0 - 5)  
**Row B12:** 0 (Fact 17: mount-time demand(d)=0)  
**Row B13:** yes, 110 (110 ≥ 0)  

Fact 16 states mount 2 is refused, but calculation shows available(d)=110 ≥ demand(d)=0. This would be refuted by: if the allocated(d) were 184 instead of 73 (requiring ~31 slots per overwrite growth), which would make available(d)=256-184-68-5=-1, but no fact describes such growth rate.

### Item 2: 384-slot pool at k = k_max - 1 = 9

**Row A1:** 384  
**Row A2:** 103 (estimate: Fact 15; 13 + 9×10 = 103)  
**Row A3:** 0  
**Row A4:** 0  
**Row A5:** 68 (rows0=0; pages_of_chain=1; chain_rewrite=2; warm_up=5×3=15; one_switch=17; instance_switch_reserve=68)  
**Row A6:** 0  
**Row A7:** 0  
**Row A8:** 0  
**Row A9:** 5 (same as Item 1; trees unchanged)  
**Row A10:** 5  
**Row A11:** 208 (384 - 103 - 68 - 5)  
**Row A12:** 2  
**Row A13:** yes, 206  

**Row B1:** 384  
**Row B2:** 113 (103 + 10)  
**Row B3:** 0  
**Row B4:** 0  
**Row B5:** 68 (rows0=1; same as Item 1)  
**Row B6:** 0  
**Row B7:** 0  
**Row B8:** 0  
**Row B9:** 5  
**Row B10:** 5  
**Row B11:** 198 (384 - 113 - 68 - 5)  
**Row B12:** 0  
**Row B13:** yes, 198  

**Row C2:** 130 (B2=113 + one_switch=17; Fact 8: real mount cost for m≥2 is one_switch)  
**Row C5:** 68 (rows0=2; pages_of_chain=ceiling((2+3)/369)=1; chain_rewrite=2; warm_up=15; one_switch=17; instance_switch_reserve=68)  
**Row C9:** 5  
**Row C10:** 5  
**Row C11:** 181 (384 - 130 - 68 - 5)  
**Row C12:** 0  
**Row C13:** yes, 181 (181 ≥ 0)  

Fact 16 states mount 3 is refused, but calculation shows available(d)=181 ≥ demand(d)=0. This would be refuted by: if the allocated(d) were 216 instead of 130 (requiring ~103 slots for mount 2's establishment), which would make available(d)=384-216-68-5=-5, but no fact describes such additional consumption.

### Item 3: History A (rollback to same target, instance 0)

**t=1:**  
- Backfill: no rows → no additions  
- Deletion: no entries → none removed  
- Count before admission: 0 → admitted  
- New entry: (1,0,0) → end count=1  

**t=2:**  
- Backfill: row at position 0 matches (1,0,0) → no additions  
- Deletion: entry (1,0,0) has [0,1) containing root 0 → not deleted → none removed  
- Count before admission: 1 → admitted  
- New entry: (2,0,0) → end count=2  

**t=3:**  
- Backfill: row at position 0 matches existing entries → no additions  
- Deletion: [0,2) contains root 0 → not deleted → none removed  
- Count before admission: 2 → admitted  
- New entry: (3,0,0) → end count=3  

**t=4:**  
- Backfill: no additions  
- Deletion: [0,3) contains root 0 → not deleted → none removed  
- Count before admission: 3 → admitted  
- New entry: (4,0,0) → end count=4  

**Table for t=1-30:**  
t | count before admission | admitted | end count  
1 | 0 | yes | 1  
2 | 1 | yes | 2  
3 | 2 | yes | 3  
4 | 3 | yes | 4  
5 | 4 | yes | 5  
6 | 5 | yes | 6  
7 | 6 | yes | 7  
8 | 7 | yes | 8  
9 | 8 | yes | 9  
10 | 9 | yes | 10  
11 | 10 | yes | 11  
12 | 11 | yes | 12  
13 | 12 | yes | 13  
14 | 13 | yes | 14  
15 | 14 | yes | 15  
16 | 15 | yes | 16  
17 | 16 | yes | 17  
18 | 17 | yes | 18  
19 | 18 | yes | 19  
20 | 19 | yes | 20  
21 | 20 | yes | 21  
22 | 21 | yes | 22  
23 | 22 | yes | 23  
24 | 23 | no | 23  
25 | 23 | no | 23  
... | ... | no | 23  
30 | 23 | no | 23  

Closed-form rule:  
- count before admission = t-1 for t ≤ 23, 23 for t ≥ 24  
- end count = t for t ≤ 23, 23 for t ≥ 24  
Fact 20 deletes entries when root ring no longer contains instances in [r_old, N), preventing growth beyond 23. This would be refuted by: if generation 0 remained in the root ring after generation 24 was published, rule (i) would not delete entry (1,0,0), allowing count to exceed 23.

### Item 4: History B (rollback to newest root)

**t=1:**  
- Backfill: no rows → no additions  
- Deletion: none → none removed  
- Count before admission: 0 → admitted  
- New entry: (1,0,0) → end count=1  

**t=2:**  
- Backfill: row at position 0 matches (1,0,0) → no additions  
- Deletion: entry (1,0,0) has [0,1) containing root 0 → not deleted; no covered entries → none removed  
- Count before admission: 1 → admitted  
- New entry: (2,1,1) → end count=2  

**t=3:**  
- Backfill: rows at positions 0 and 1 match existing entries → no additions  
- Deletion: (1,0,0) has [0,1) containing root 0; (2,1,1) has [1,2) containing root 1 → not deleted; no covered entries → none removed  
- Count before admission: 2 → admitted  
- New entry: (3,2,2) → end count=3  

**t=4:**  
- Backfill: no additions  
- Deletion: no deletions (all entries have roots in their ranges) → none removed  
- Count before admission: 3 → admitted  
- New entry: (4,3,3) → end count=4  

**Table for t=1-30:**  
t | count before admission | admitted | end count  
1 | 0 | yes | 1  
2 | 1 | yes | 2  
3 | 2 | yes | 3  
4 | 3 | yes | 4  
5 | 4 | yes | 5  
6 | 5 | yes | 6  
7 | 6 | yes | 7  
8 | 7 | yes | 8  
9 | 8 | yes | 9  
10 | 9 | yes | 10  
11 | 10 | yes | 11  
12 | 11 | yes | 12  
13 | 12 | yes | 13  
14 | 13 | yes | 14  
15 | 14 | yes | 15  
16 | 15 | yes | 16  
17 | 16 | yes | 17  
18 | 17 | yes | 18  
19 | 18 | yes | 19  
20 | 19 | yes | 20  
21 | 20 | yes | 21  
22 | 21 | yes | 22  
23 | 22 | yes | 23  
24 | 23 | yes | 2  
25 | 1 | yes | 2  
26 | 1 | yes | 2  
... | 1 | yes | 2  
30 | 1 | yes | 2  

**t=24 details:**  
- Before deletion: 24 entries (1,0,0) to (24,23,23)  
- Deletion: covered-deletion (Fact 21) removes all except (24,23,23) because each (k,k-1,k-1) is covered by (k+1,k,k)  
- Count before admission: 1 (after deletion)  
- Admitted → new entry (25,24,24) → end count=2  
- Root ring at t=24: generations 1-24; generation 0 overwritten by g=24 (Fact 19) → no root 0 → rule (i) does not apply for deletion  

Closed-form rule:  
- count before admission = t-1 for t ≤ 23, 1 for t ≥ 24  
- end count = t for t ≤ 23, 2 for t ≥ 24  
Fact 21 (covered-deletion) removes all but the latest entry starting at t=24, as each entry (k,k-1,k-1) is covered by (k+1,k,k). This would be refuted by: if the txg sequence did not strictly increase, so (rb, Tb) ≤ (ra, Ta) comparisons would fail, preventing covered-deletion and allowing count to grow beyond 23.
