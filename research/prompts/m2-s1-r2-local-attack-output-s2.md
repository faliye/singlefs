Section 1  
1. Wait time formula: wait_time = L. Assumption: all k fsync calls arrive at exactly the same instant, so the batch starts immediately without waiting for additional arrivals, and the publish sequence takes duration L regardless of k.  
Falsifier: If a real system shows that the first-arriving fsync’s wait time is greater than L when k > 1, this would contradict the assumption of simultaneous arrival.  

2. At k=1: L; k=8: L; k=16: L. All values are exactly L in units of L.  
Falsifier: If measured wait times differ from L for any k, this would contradict the formula.  

3. Wait time stays flat as k grows. It is linear (slope zero) in k.  
Falsifier: If wait time increases or decreases with k in a real system, this would contradict the flat prediction.  

4. Different. Wait time is a duration (units of L), while s is a dimensionless ratio of byte counts (as defined in FACT TABLE 3, row b).  
Falsifier: If s were measured in time units or wait time in byte units, this would contradict the distinction.  

5. Cannot construct such a scenario. The wait time formula predicts constant L regardless of k, so it cannot increase with k. The amortized-bytes metric s decreases with k (per FACT TABLE 3 row c), but wait time does not change, making the scenario impossible.  
Falsifier: If wait time increased with k in a real system while s decreased, this would contradict the formula, but the formula inherently cannot show this.  

6. Wait time remains bounded at L as k grows without limit. A measured behavior contradicting this would be if the first-arriving fsync’s wait time increases with k (e.g., proportional to k or superlinearly) in a real system.  
Falsifier: If a real system shows wait time growing beyond L for large k (e.g., k=256), this would contradict the bounded prediction.  

Section 2  
1. 135^3 = 135 × 135 × 135 = 2,460,375 (less than 268,435,456). 135^4 = 2,460,375 × 135 = 332,150,625 (greater than 268,435,456). Smallest h is 4.  
Falsifier: If 135^4 were less than 268,435,456 or 135^3 were greater, this would change h.  

2. Leaf count = total data size / data-unit width = 2^40 bytes / 32768 bytes = 1099511627776 / 32768 = 33554432.  
Falsifier: If the total data size or data-unit width differed from the facts, this would change the leaf count.  

3. 135^3 = 2,460,375 (less than 33,554,432). 135^4 = 332,150,625 (greater than 33,554,432). Smallest h is 4.  
Falsifier: If 135^4 were less than 33,554,432 or 135^3 were greater, this would change h.  

4. Ratios for streams=16, tree_height=4:  
batch=1: (1 + 1×4 + 2)/(1+1) = 7/2 = 3.5  
batch=2: (2 + 2×4 + 2)/(2+1) = 12/3 = 4.0  
batch=5: (5 + 5×4 + 2)/(5+1) = 27/6 = 4.5  
batch=10: (10 + 10×4 + 2)/(10+1) = 52/11 ≈ 4.727  
batch=20: (20 + 16×4 + 2)/(20+1) = 86/21 ≈ 4.095  
batch=50: (50 + 16×4 + 2)/(50+1) = 116/51 ≈ 2.275  
batch=100: (100 + 16×4 + 2)/(100+1) = 166/101 ≈ 1.644  
batch=200: (200 + 16×4 + 2)/(200+1) = 266/201 ≈ 1.323  
Peak position: batch=10.  
Falsifier: If any ratio calculation used incorrect tree_height or formula parameters, this would change the values.  

5. Peak position is the same (batch=10). Peak value (≈4.727) is higher than 3.91.  
Falsifier: If the recomputed peak value were ≤3.91 or the peak position changed, this would contradict the comparison.  

6. Recomputed values vs. FACT TABLE 6 measured values:  
streams=2, batch=2: 4.0 > 3.67  
streams=4, batch=2: 4.0 > 3.67  
streams=8, batch=5: 4.5 > 3.83  
streams=32, batch=20: 102/21 ≈ 4.857 > 3.95  
streams=64, batch=50: 252/51 ≈ 4.941 > 3.98  
streams=128, batch=50: 252/51 ≈ 4.941 > 3.98  
All recomputed values are higher.  
Falsifier: If any recomputed value were ≤ the measured value, this would contradict the comparison.  

7. index node. FACT TABLE 5 describes "block width 4 KiB" in the context of tree geometry (e.g., "complete 128-ary tree, 4 internal levels, block width 4 KiB"), which refers to tree-node size. FACT TABLE 9 row b states today’s index-node width is 16384 bytes, and row d states the journal record is 4096 bytes. The "block width" in FACT TABLE 5 is part of the tree structure definition, so it corresponds to the index node size (not the journal record).  
Falsifier: If FACT TABLE 5 explicitly linked "block width" to journal records or FACT TABLE 9 showed index-node width matching 4 KiB, this would change the conclusion.
