1 5575606380 is a deterministic arithmetic derivation from the sizes list in Table 2 Row 1. Falsified by: if the sum of (2 raised to each segment's write count minus 1) across all 55 segments plus 1 does not equal 5575606380.

2 5575802973 is a deterministic arithmetic derivation from the sizes list in Table 2 Row 2. Falsified by: if the sum of (2 raised to each segment's write count minus 1) across all 54 segments plus 1 does not equal 5575802973.

3 The two lists are not identical. Table 2 Row 1 has 55 entries summing to 425 writes and 55 segments; Table 2 Row 2 has 54 entries summing to 423 writes and 54 segments. They disagree at position 45: Row 1 has 4, Row 2 has 18. The closed-form totals 5575606380 and 5575802973 are numerically unequal.

4 The number 5575606380 matches the in-memory reconstruction report (Table 2 Row 1) but not today's direct measurement (Table 2 Row 2). Calling it "measured" without qualification is inaccurate because a same-day direct measurement produced a different number.

5 The computed rate is 4194320 divided by 302.2 equals approximately 13880.2 states per second, which rounds to about 13880. The two conditions are: (1) running on 16 threads (higher higher thread count would increase rate), (2) competing with Opus attacker leg (no competition would increase rate).

6 Using 5575606380: 5575606380 divided by (13880 * 2) equals approximately 200850 seconds, which is about 2.32 days (rounds to 2.3 days). Using 5575802973: the result is about 2.325 days, which differs by less than 0.1 day.

7 Steps: (1) compute rate from 4194320/302.2 (direct observation), (2) double rate for 32 threads (extrapolation: assumes linear scaling with threads), (3) divide second stream's state count by 32-thread rate (extrapolation: assumes rate applies to second stream despite being measured on first stream).

8 The number 210 was computed via a segment-by-segment closed-form formula using each segment's write counts, not by running candidate A2's enumerator. Table 3 Row 6 states "candidate A2's figure of 210 was computed from each segment's own write counts; the candidate-A2 enumerator itself was never actually run to produce this number", so it is a formula-derived figure.

9 2 raised to the power of (2 + 16) equals 262144, which matches the total-states figure. The fraction 32768/262144 equals 1/8, which is 1/(2^3), so n=3.

10  
5575606380 deterministic arithmetic derivation  
13880 per second deterministic arithmetic derivation  
2.3 days extrapolation (assumes linear thread scaling and rate consistency across streams)  
210 deterministic arithmetic derivation  
262144 and 32768 directly observed count