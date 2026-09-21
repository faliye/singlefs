Section 1  
1. The wait time for the first-arriving fsync is given by the formula: wait_time = L + (k - 1) * Δt, where Δt is the fixed inter-arrival time between consecutive fsync calls in the batch. Assumption: fsync calls arrive sequentially with a constant inter-arrival time Δt, so the first fsync arrives at time 0, the second at Δt, ..., the kth at (k - 1) * Δt; the batch starts immediately when the kth arrives, and the publish sequence takes L time.  
Falsifier: If the measured wait time for the first-arriving fsync is independent of k (e.g., constant regardless of batch size), this would contradict the formula.  

2. The numeric wait time in units of L is 1 for k=1 (since (1-1)*Δt = 0), 1 + (8-1)*(Δt/L) for k=8, and 1 + (16-1)*(Δt/L) for k=16. However, without a known value for Δt/L, exact numeric values cannot be determined from the facts. Given the facts do not specify Δt, the formula depends on an unknown constant, but under the assumption that Δt = 0 (all fsyncs arrive simultaneously), the wait time is L for all k, yielding 1 in units of L for all cases.  
Falsifier: If in a real system, the wait time for the first-arriving fsync is observed to be less than L when k=1 (e.g., 0.5L), this would contradict the formula.  

3. The wait time grows as k increases (linearly in k), since wait_time = L + (k - 1) * Δt. The growth is linear in k.  
Falsifier: If the measured wait time remains constant or decreases as k increases (e.g., for k=16 it is smaller than for k=8), this would contradict the linear growth prediction.  

4. Different. The wait time formula has units of time (duration), while s is a dimensionless ratio of bytes per fsync (amortized byte count divided by amort byte count).  
Falsifier: If s were measured in time units (e.g., seconds) or the wait time were measured in bytes, this would contradict the distinction.  

5. Cannot construct such a scenario. The wait time formula (under the assumption that Δt is constant) increases with k, but the amortized-bytes metric s decreases with k (as per FACT TABLE 3 row c, where fold_point_ten_percent occurs at k=8 for wal_full). However, the wait time formula requires Δt to be positive for growth, but s is defined solely by byte counts (no time dependency). The formula for s does not involve wait time; it is purely a byte metric. Since the wait time and s are fundamentally different quantities (time vs. bytes), no single scenario can simultaneously show wait time increasing while s decreases—these are independent metrics.  
Falsifier: If the measured s and wait time were found to be directly correlated (e.g., both increasing or both decreasing together), this would contradict the independence of the quantities.  

6. The wait time grows unbounded as k increases (wait_time = L + (k - 1) * Δt). Measured behavior that would contradict this is if the wait time for the first-arriving fsync remains bounded (e.g., constant or capped) even as k grows to 4096, such as a maximum wait time of 100ms regardless of batch size.  
Falsifier: If the measured wait time for the first-arriving fsync at k=4096 is less than or equal to the wait time at k=16 (e.g., 10ms vs. 20ms), this would contradict the unbounded growth prediction.  

Section 2  
1. 135^3 = 135 × 135 × 135 = 18,225 × 135 = 2,460,375. 135^4 = 2,460,375 × 135 = 332,150,625. Since 2,460,375 < 268,435,456 < 332,150,625, the smallest h is 4.  
Falsifier: If a measurement showed that 135^4 < 268,435,456, this would contradict h=4.  

2. Leaf count = total data size / data-unit width = 2^40 bytes / 32,768 bytes = 1,099,511,627,776 / 32,768 = 33,554,432.  
Falsifier: If the total data size were measured as different from 2^40 bytes, this would contradict the leaf count calculation.  

3. 135^3 = 2,460,375 < 33,554,432 and 135^4 = 332,150,625 > 33,554,432, so the smallest h is 4.  
Falsifier: If 135^4 were measured as less than 33,554,432, this would contradict h=4.  

4. Using tree height h=4 and streams=16:  
- batch=1: (1 + min(16,1)×4 + 2)/(1+1) = (1+4+2)/2 = 3.5  
- batch=2: (2 + min(16,2)×4 + 2)/3 = (2+8+2)/3 = 4.0  
- batch=5: (5 + min(16,5)×4 + 2)/6 = (5+20+2)/6 = 4.5  
- batch=10: (10 + min(16,10)×4 + 2)/11 = (10+40+2)/11 ≈ 4.727  
- batch=20: (20 + min(16,20)×4 + 2)/21 = (20+64+2)/21 ≈ 4.095  
- batch=50: (50 + min(16,50)×4 + 2)/51 = (50+64+2)/51 ≈ 2.275  
- batch=100: (100 + min(16,100)×4 + 2)/101 = (100+64+2)/101 ≈ 1.644  
- batch=200: (200 + min(16,200)×4 + 2)/201 = (200+64+2)/201 ≈ 1.323  
Peak position is batch=10.  
Falsifier: If the measured ratio at batch=10 were less than 4.727 (e.g., 3.5), this would contradict the recomputed peak position.  

5. The peak position (batch=10) matches FACT TABLE 6, but the peak value (≈4.727) is higher than FACT TABLE 6's value of 3.91.  
Falsifier: If the measured peak value for streams=16 were exactly 4.727, this would confirm the recomputation, but if it were lower than 3.91, it would contradict the comparison.  

6. Recomputed values at original peak positions:  
- streams=2, batch=2: 4.0 > 3.67 (higher)  
- streams=4, batch=2: 4.0 > 3.67 (higher)  
- streams=8, batch=5: 4.5 > 3.83 (higher)  
- streams=32, batch=20: ≈4.857 > 3.95 (higher)  
- streams=64, batch=50: ≈4.941 > 3.98 (higher)  
- streams=128, batch=50: ≈4.941 > 3.98 (higher)  
Falsifier: If any recomputed value were lower than the measured value (e.g., streams=2, batch=2 ratio < 3.67), this would contradict the comparison.  

7. index node. FACT TABLE 5 describes "block width" in the context of tree geometry (e.g., "128-ary tree, 4 internal levels, block width 4 KiB"), which refers to the size of tree nodes (index nodes). FACT TABLE 8 uses "node_bytes" to compute fanout for tree nodes, and FACT TABLE 9 row b specifies "index-node width" as 16384 bytes, confirming that "block width" in tree contexts refers to index nodes. The journal record width (4096 bytes, FACT TABLE 9 row d) is unrelated to tree node geometry.  
Falsifier: If the tree node width in the real system were measured as 4096 bytes, this would contradict the "index node" assignment.
