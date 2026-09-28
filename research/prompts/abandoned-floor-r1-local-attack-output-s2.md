Q1. TODAY a: 5 [FACT 8]  
This would be refuted by: if the F_eff after the raise is not 5.  
TODAY b: 2 + 8 = 10 [FACT 9,10]  
This would be refuted by: if the ledger allocated count for the ten slots is less than 10.  
TODAY c: 0 + 0 = 0 [FACT 3,9]  
This would be refuted by: if the checker walk includes any of the ten slots.  
TODAY d: 2 + 8 = 10 [FACT 4,5,9]  
This would be refuted by: if the ledger defer count for the ten slots is not 10.  
TODAY e: 2 + 8 = 10 [FACT 6,7,9]  
This would be refuted by: if the isolation bit is not set for all ten slots.  
TODAY f: 10  
This would be refuted by: if the difference (b minus c) is not 10 slots.  
TODAY g: 163840 [FACT 10]  
This would be refuted by: if the whole device I-3.1 difference in bytes is not 163840.  

JIA a: 5 [FACT 8]  
This would be refuted by: if the F_eff after the raise is not 5.  
JIA b: 2 + 8 = 10 [FACT 9,10]  
This would be refuted by: if the ledger allocated count for the ten slots is less than 10.  
JIA c: 2 + 8 = 10 [JIA, FACT 9]  
This would be refuted by: if the checker walk does not include the ten slots after JIA change.  
JIA d: 2 + 8 = 10 [FACT 4,5,9]  
This would be refuted by: if the ledger defer count for the ten slots is not 10.  
JIA e: 2 + 8 = 10 [FACT 6,7,9]  
This would be refuted by: if the isolation bit is not set for all ten slots.  
JIA f: 0  
This would be refuted by: if the difference (b minus c) is not 0.  
JIA g: 147456  
This would be refuted by: if the whole device I-3.1 difference in bytes is not 147456.  

YI1 a: 6  
This would be refuted by: if F lands on a txg other than 6.  
YI1 b: 0 + 0 = 0  
This would be refuted by: if the ledger allocated count for the ten slots is not 0 after F raised to 6.  
YI1 c: 0 + 0 = 0  
This would be refuted by: if the checker walk includes any of the ten slots.  
YI1 d: 0 + 0 = 0  
This would be refuted by: if the ledger defer count for the ten slots is not 0.  
YI1 e: 2 + 8 = 10 [FACT 6,7,9]  
This would be refuted by: if the isolation bit is not set for all ten slots after reclaim.  
YI1 f: 0  
This would be refuted by: if the difference (b minus c) is not 0.  
YI1 g: 147456  
This would be refuted by: if the whole device I-3.1 difference in bytes is not 147456.  

YI2 a: not determinable [missing fact: F before raise]  
This would be refuted by: if the F value before the raise is known and the F_eff is not that value.  
YI2 b: not determinable [missing fact: F before raise]  
This would be refuted by: if the F value before the raise is known and the ledger allocated count is not as expected.  
YI2 c: not determinable [missing fact: F before raise]  
This would be refuted by: if the F value before the raise is known and the checker walk count is not as expected.  
YI2 d: not determinable [missing fact: F before raise]  
This would be refuted by: if the F value before the raise is known and the ledger defer count is not as expected.  
YI2 e: 2 + 8 = 10 [FACT 6,7,9]  
This would be refuted by: if the isolation bit is not set for all ten slots.  
YI2 f: not determinable [missing fact: F before raise]  
This would be refuted by: if the F value before the raise is known and the difference is not as expected.  
YI2 g: not determinable [missing fact: F before raise]  
This would be refuted by: if the F value before the raise is known and the whole difference is not as expected.  

BING a: 5 [FACT 8]  
This would be refuted by: if the F_eff after the raise is not 5.  
BING b: 0 + 0 = 0  
This would be refuted by: if the ledger allocated count for the ten slots is not 0.  
BING c: 0 + 0 = 0  
This would be refuted by: if the checker walk includes any of the ten slots.  
BING d: 0 + 0 = 0  
This would be refuted by: if the ledger defer count for the ten slots is not 0.  
BING e: 2 + 8 = 10 [FACT 6,7,9]  
This would be refuted by: if the isolation bit is not set for all ten slots after reclaim.  
BING f: 0  
This would be refuted by: if the difference (b minus c) is not 0.  
BING g: 147456  
This would be refuted by: if the whole device I-3.1 difference in bytes is not 147456.  

DING1 a: 5 [FACT 8]  
This would be refuted by: if the F_eff after the raise is not 5.  
DING1 b: 2 + 8 = 10 [FACT 9,10]  
This would be refuted by: if the ledger allocated count for the ten slots is less than 10.  
DING1 c: 2 + 8 = 10 [DING1, FACT 9]  
This would be refuted by: if the checker walk does not include the shadow ledger isolation set.  
DING1 d: 2 + 8 = 10 [FACT 4,5,9]  
This would be refuted by: if the ledger defer count for the ten slots is not 10.  
DING1 e: 2 + 8 = 10 [FACT 6,7,9]  
This would be refuted by: if the isolation bit is not set for all ten slots.  
DING1 f: 0  
This would be refuted by: if the difference (b minus c) is not 0.  
DING1 g: 0  
This would be refuted by: if the whole device I-3.1 difference in bytes is not 0.  

DING2 a: 5 [FACT 8]  
This would be refuted by: if the F_eff after the raise is not 5.  
DING2 b: 2 + 8 = 10 [FACT 9,10]  
This would be refuted by: if the ledger allocated count for the ten slots is less than 10.  
DING2 c: 0 + 0 = 0 [assuming release generation < F]  
This would be refuted by: if the release generation of the ten slots is less than F or if the walk includes them under DING2.  
DING2 d: 2 + 8 = 10 [FACT 4,5,9]  
This would be refuted by: if the ledger defer count for the ten slots is not 10.  
DING2 e: 2 + 8 = 10 [FACT 6,7,9]  
This would be refuted by: if the isolation bit is not set for all ten slots.  
DING2 f: 10  
This would be refuted by: if the difference (b minus c) is not 10.  
DING2 g: 163840 [FACT 10]  
This would be refuted by: if the whole device I-3.1 difference in bytes is not 163840.  

Q2. F lands on 6. The root at txg 6, instance 2 is not abandoned, as per FACT 8.  
This would be refuted by: if the root at txg 6 is abandoned or if F lands on a different txg.  

Q3. I used release generation < F. Under the other reading (referenced only by roots below F), (c) is 0 and (f) is 10.  
This would be refuted by: if under the other reading, (c) or (f) differs from 0 and 10.  

Q4. All ten slots are forbidden, count 10.  
This would be refuted by: if any of the ten slots is not forbidden or the count is not 10.  

Q5. Ledger allocated minus walk is 10 slots, which equals TODAY (f). Yes, all ten slots are counted in (e) for TODAY, but this does not change the checker's walk under FACT 1 and FACT 3.  
This would be refuted by: if the byte difference does not equal 10 slots or if the walk changes due to isolation bit.  

Q6. S1 a: 3308 [FACT 17]  
This would be refuted by: if the ledger empty segment count is not 3308.  
S1 b: not determinable [missing fact: distribution of used, isolated, hold counts]  
This would be refuted by: if the allocator's caliber count is known and differs from the value.  
S1 c: not determinable  
This would be refuted by: if the difference (a minus b) is known and differs from the value.  
S1 d: 50432, 0 [FACT 17]  
This would be refuted by: if the lowest empty segment is not 50432 or the segments above is not 0.  

S2 a: 3308 [FACT 17]  
This would be refuted by: if the ledger empty segment count is not 3308.  
S2 b: not determinable [missing fact: distribution of used, isolated, hold counts]  
This would be refuted by: if the allocator's caliber count is known and differs from the value.  
S2 c: not determinable  
This would be refuted by: if the difference (a minus b) is known and differs from the value.  
S2 d: 50496, 1 [FACT 17]  
This would be refuted by: if the lowest empty segment is not 50496 or the segments above is not 1.  

S3 a: 3308 [FACT 17]  
This would be refuted by: if the ledger empty segment count is not 3308.  
S3 b: not determinable [missing fact: distribution of used, isolated, hold counts]  
This would be refuted by: if the allocator's caliber count is known and differs from the value.  
S3 c: not determinable  
This would be refuted by: if the difference (a minus b) is known and differs from the value.  
S3 d: 50496, 1 [FACT 17]  
This would be refuted by: if the lowest empty segment is not 50496 or the segments above is not 1.  

S4 a: 3308 [FACT 17]  
This would be refuted by: if the ledger empty segment count is not 3308.  
S4 b: not determinable [missing fact: distribution of used, isolated, hold counts]  
This would be refuted by: if the allocator's caliber count is known and differs from the value.  
S4 c: not determinable  
This would be refuted by: if the difference (a minus b) is known and differs from the value.  
S4 d: 50432, 0 [FACT 17]  
This would be refuted by: if the lowest empty segment is not 50432 or the segments above is not 0.  

Q7. At most 1 segment. The facts do not tell if device 0 has a last partial segment.  
This would be refuted by: if the difference is more than 1 or if the facts state whether there is a last partial segment.  

Q8. Under (i): not determinable [missing fact: allocator's caliber count]  
Under (ii): ledger row quantity 3308, allocator caliber quantity not determinable [missing fact: allocator's caliber count]  
Under (iii): 3308  
This would be refuted by: if under (i) the ledger row is not the allocator's caliber count or under (ii) the ledger row is not 3308 or the allocator quantity is known and differs.  

Q9. Both jumps are 64 slots and 1 segment, same size.  
This would be refuted by: if the jumps differ in slots or segments or are not equal.
