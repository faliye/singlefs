Q1 TABLE TEN  
TODAY a: 5 [FACT 8]  
TODAY b: 2 + 8 = 10 [FACT 9,10]  
TODAY c: 0 [FACT 9,3]  
TODAY d: 2 + 8 = 10 [FACT 4,5]  
TODAY e: 0 [FACT 6,9]  
TODAY f: 10 [from b and c]  
TODAY g: 163840 [FACT 10]  
This would be refuted by: if the whole device-0 I-3.1 difference is not 163840 bytes.  

JIA a: 5 [FACT 8]  
JIA b: 2 + 8 = 10 [FACT 9,10]  
JIA c: 10 [FACT 9,3,8]  
JIA d: 2 + 8 = 10 [FACT 4,5]  
JIA e: 0 [FACT 6,9]  
JIA f: 0 [from b and c]  
JIA g: 0 [FACT 10,8,9,3]  
This would be refuted by: if the whole device-0 I-3.1 difference is not 0 bytes.  

YI1 a: 6 [FACT 8,5]  
YI1 b: 0 [FACT 5,9]  
YI1 c: 0 [FACT 9,3]  
YI1 d: 0 [FACT 5,4]  
YI1 e: 0 [FACT 6,9]  
YI1 f: 0 [from b and c]  
YI1 g: 0 [FACT 10,5,9]  
This would be refuted by: if the whole device-0 I-3.1 difference is not 0 bytes.  

YI2 a: not determinable [missing fact about F_eff before raise]  
YI2 b: 10 [FACT 7]  
YI2 c: 10 [FACT 7,3]  
YI2 d: 10 [FACT 4,5,7]  
YI2 e: 0 [FACT 6,9]  
YI2 f: 0 [from b and c]  
YI2 g: 0 [FACT 7]  
This would be refuted by: if before the raise attempt the I-3.1 difference was not 0 bytes.  

BING a: 5 [FACT 8]  
BING b: 0 [FACT 5,9]  
BING c: 0 [FACT 9,3]  
BING d: 0 [FACT 5,4]  
BING e: 0 [FACT 6,9]  
BING f: 0 [from b and c]  
BING g: 0 [FACT 10,5,9]  
This would be refuted by: if the whole device-0 I-3.1 difference is not 0 bytes.  

DING1 a: 5 [FACT 8]  
DING1 b: 2 + 8 = 10 [FACT 9,10]  
DING1 c: 0 [FACT 9,3]  
DING1 d: 2 + 8 = 10 [FACT 4,5]  
DING1 e: 0 [FACT 6,9]  
DING1 f: 10 [from b and c]  
DING1 g: 163840 [FACT 10]  
This would be refuted by: if the whole device-0 I-3.1 difference is not 163840 bytes.  

DING2 a: 5 [FACT 8]  
DING2 b: 2 + 8 = 10 [FACT 9,10]  
DING2 c: 0 [FACT 9,3]  
DING2 d: 2 + 8 = 10 [FACT 4,5]  
DING2 e: 0 [FACT 6,9]  
DING2 f: 10 [from b and c]  
DING2 g: 163840 [FACT 10]  
This would be refuted by: if the whole device-0 I-3.1 difference is not 163840 bytes.  

Q2 For YI1, F lands on 6. The root at txg 6 is not abandoned, as per FACT 8 which states "txg 6, instance 2, F 0, not abandoned, not below the floor".  

Q3 For DING2 in Q1, the reading used for "release records below F" is records with release generation < F. Under the other reading (records referenced only by roots below F), (c) would still be 0 and (f) would still be 10 for the ten slots, as the ten slots are not referenced only by roots below F due to reference by (1,5) which is not below F.  

Q4 For BING, the shadow ledger's rule forbids reusing isolated slots. The ten slots are not referenced only by abandoned roots (they are referenced by non-abandoned roots like txg 3 and 4), so they are not isolated, and none are forbidden from reuse. Count: 0.  

Q5 From FACT 10, ledger allocated minus walk is 163840 bytes, which is 10 slots, matching TODAY f. TODAY (e) is 0 for isolation bit set, which does not change the checker's walk under FACT 1 and FACT 3 because the walk is defined by valid roots only, and isolation bit is separate from the walk computation.  

Q6 TABLE SEGMENTS  
S1 a: 3308 [FACT 17]  
S1 b: not determinable [missing fact about total segments or segment counts]  
S1 c: not determinable [from b]  
S1 d: 50432, 0 segments above 50432 [FACT 17]  
This would be refuted by: if the ledger empty segment count is not 3308 or lowest empty segment is not 50432 or not 0 segments above 50432.  

S2 a: 3308 [FACT 17]  
S2 b: not determinable [missing fact about total segments or segment counts]  
S2 c: not determinable [from b]  
S2 d: 50496, 1 segment above 50432 [FACT 17]  
This would be refuted by: if the lowest empty segment is not 50496 or not 1 segment above 50432.  

S3 a: 3308 [FACT 17]  
S3 b: not determinable [missing fact about total segments or segment counts]  
S3 c: not determinable [from b]  
S3 d: 50496, 1 segment above 50432 [FACT 17]  
This would be refuted by: if the lowest empty segment is not 50496 or not 1 segment above 50432.  

S4 a: 3308 [FACT 17]  
S4 b: not determinable [missing fact about total segments or segment counts]  
S4 c: not determinable [from b]  
S4 d: 50432, 0 segments above 50432 [FACT 17]  
This would be refuted by: if the ledger empty segment count is not 3308 or lowest empty segment is not 50432 or not 0 segments above 50432.  

Q7 By at most 1 segment the difference alone can make the two counts differ. The facts do not tell if device 0 has a last partial segment.  

Q8 For S3, under fix (i) ledger row would be not determinable; under fix (ii) ledger row would be 3308 and allocator caliber would be not determinable; under fix (iii) ledger row would be 3308.  

Q9 FACT 14 jump: 50368 - 50304 = 64 slots, 1 segment; FACT 17 jump: 50496 - 50432 = 64 slots, 1 segment; both have same size.
