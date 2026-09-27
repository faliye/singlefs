1 5575606380
Falsified by: if the sum of (2 raised to each segment size minus 1) plus 1 does not equal 5575606380

2 5575802973
Falsified by: if the computed closed-form total does not equal 5575802973

3 The two lists are not identical. Table 2 Row 1 has 55 entries and sum 425 writes. Table 2 Row 2 has 54 entries and sum 423 writes. At position 45, Row 1 has 4 and Row 2 has 18. The closed-form totals 5575606380 and 5575802973 are not numerically equal.
Falsified by: if the lists are identical or if the closed-form totals are equal

4 5575606380 matches Table 2 Row 1 (in-memory reconstruction). The direct measurement in Table 2 Row 2 produced a different number, so calling it "measured" without qualification is inaccurate.
Falsified by: if the direct measurement in Table 2 Row 2 also yields 5575606380

5 4194320 divided by 302.2 equals approximately 13879.94, which rounds to 13880. The two conditions are: 16 threads and competing with Opus attacker for CPU. A quiet single-task 16-thread run would have a higher rate due to no CPU competition.
Falsified by: if the calculated rate does not round to 13880

6 13880 states per second doubled to 32 threads is 27760 states per second. 5575606380 divided by 27760 equals 200850 seconds, which is 2.324 days (rounds to 2.3 days). Using 5575802973 gives 200857 seconds or 2.325 days, a difference of 0.001 days, which is not more than 0.1 day.
Falsified by: if the days calculation does not round to 2.3 or if using Row 2's total changes the days by more than 0.1

7 Step 1: Rate measurement (direct observation). Step 2: Doubling rate for 32 threads (extrapolation assuming linear scaling of threads). Step 3: Dividing second stream's state count by 32-thread rate (deterministic arithmetic). Step 4: Converting seconds to days (deterministic arithmetic).
Falsified by: if any step's classification is incorrect

8 210 was computed using a segment-by-segment formula based on each segment's write counts. The candidate-A2 enumerator was never run to produce this number. This is a formula-derived figure as stated in Table 3 Row 6: "candidate-A2 enumerator itself was never actually run to produce this number".
Falsified by: if the candidate-A2 enumerator was run to produce 210

9 2 raised to 18 equals 262144. 32768 divided by 262144 equals 1/8, which is 1/(2^3), so n=3.
Falsified by: if 2^18 does not equal 262144 or the fraction is not 1/8

10 
5575606380 deterministic arithmetic derivation from directly observed inputs
13880 per second deterministic arithmetic derivation from directly observed inputs
2.3 days extrapolation (assumes linear scaling of threads)
210 deterministic arithmetic derivation from directly observed inputs
262144 and 32768 262144 is deterministic arithmetic derivation from directly observed inputs; 32768 is directly observed count
Falsified by: for 5575606380 if the inputs are not directly observed; for 13880 if the states and time are not directly observed; for 2.3 days if no linear scaling assumption; for 210 if segment sizes are not directly observed; for the pair if 262144 is not derived from segment size or 32768 is not directly observed
