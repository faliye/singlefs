1
(a) 1 for disk 0, 1 for disk 1, total 2. Step: Favorable case has newest root on each disk with F>0 and next older with F=0. Corrupting the newest slot on disk 0 makes its F sub d 0 (using next older valid slot). Same for disk 1. Total corrupted slots: 1+1=2.
(b) 16 for disk 0, 8 for disk 1, total 24. Step: Unfavorable case has all root slots on disk 0 (16 slots from regions 0 and 2) and disk 1 (8 slots) with same nonzero F. To make F sub d zero, all slots must be corrupted since any remaining slot would have F>0. Total: 16+8=24.
(c) Part (a)'s total answers the question. "Least number that must be corrupted" refers to the smallest sufficient number in some possible history (part a), not the worst-case (part b), because fact 5's MAX formula depends only on current valid roots, and the minimal corruption needed is achievable in the favorable case.
This would be refuted by: misinterpreting the question as asking for the worst-case scenario instead of the minimal sufficient scenario.

2
Design MAX+HOLD's F effective formula is textually identical to design MAX's F effective formula as stated in fact 5: "for the purpose of computing F effective, this design is identical to design MAX".
(a) 1 for disk 0, 1 for disk 1, total 2. Step: Same as item 1(a) because F effective formula is identical and modeling assumption ensures next older root is valid.
(b) 16 for disk 0, 8 for disk 1, total 24. Step: Same as item 1(b) because F effective formula is identical and modeling assumption applies to valid root records.
(c) Part (a)'s total answers the question. Step: Same reasoning as item 1(c).
The numeric answers are identical to item 1 because the F effective formula is the same, and MAX+HOLD's slot holding behavior only affects when freed slots are released, not the current valid root records' F values used in F sub d calculation.
This would be refuted by: incorrectly assuming that MAX+HOLD's slot holding behavior affects F sub d's calculation, but fact 5 states that F effective formula is identical to MAX, and slot holding only affects freed slot release timing.

3
(a) Root ring part:
(a) 1 for disk 0, 1 for disk 1, total 2. Step: Same as item 1(a).
(b) 16 for disk 0, 8 for disk 1, total 24. Step: Same as item 1(b).
(c) Part (a)'s total answers the question for root ring part. Step: Same reasoning as item 1(c).
(b) System configuration part:
(a) Favorable case: 1 for disk 0, 1 for disk 1, total 2. Step: Fact 6 says each disk has two system config slots. Favorable case has selected slot F>0 and other F=0. Corrupting selected slot leaves F=0, so max is 0. Total: 1+1=2.
(b) Unfavorable case: 2 for disk 0, 2 for disk 1, total 4. Step: Both slots on each disk have same nonzero F. Corrupting both slots on each disk is needed to make max zero. Total: 2+2=4.
(c) Smallest combined total: 4 (root ring 2 + system config 2). Largest combined total: 28 (root ring 24 + system config 4). Step: Smallest uses favorable case for both parts; largest uses unfavorable case for both parts.
This would be refuted by: misapplying fact 6's two slots per disk count or misunderstanding the modeling assumption for system config slots.
