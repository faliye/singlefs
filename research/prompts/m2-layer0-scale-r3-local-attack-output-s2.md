1. The first three segments (2, 2, 1) yield 3 + 3 + 1 = 7. The last segment (2) yields 3. The sizes list has 55 entries summing to 425. The grand total equals 5575606380.
Falsified by: Any discrepancy in the sum of the sizes list not equaling 425, or the computed closed-form total not matching 5575606380.

2. The sizes list has 54 entries summing to 423. The computed closed-form total equals 5575802973.
Falsified by: Any discrepancy in the sum of the sizes list not equaling 423, or the computed closed-form total not matching 5575802973.

3. The two lists are not identical. Row 1 has 55 entries summing to 425. Row 2 has 54 entries summing to 423. They disagree at position 45: Row 1 has 4, Row 2 has 18. The closed-form totals are not numerically equal.
Falsified by: If the lists were identical, or the sum of either list did not match the stated writes count, or the closed-form totals were equal.

4. The number 5575606380 matches Table 2 Row 1's reconstruction report, not Table 2 Row 2's direct measurement. Calling it "measured" without qualification is inaccurate because a same-day direct measurement of the same stream produced a different number.
Falsified by: If 5575606380 matched the direct measurement's closed-form total, or if the direct measurement's closed-form total matched the reconstruction's.

5. 4194320 divided by 302.2 equals approximately 13879.9, which rounds to about 13880. The two conditions are: (1) CPU contention from Opus attacker leg, which would push the rate higher on a quiet run; (2) specific machine hardware configuration, which could cause the rate to be higher or lower depending on specs but direction cannot be determined without additional data.
Falsified by: A computed rate not rounding to 13880, or evidence that the rate was not measured under Opus competition, or the machine hardware not affecting the rate as stated.

6. 13880 per second doubled to 32 threads is 27760 per second. 5575606380 divided by 27760 equals approximately 200850 seconds, or 2.325 days, rounding to about 2.3 days. Using 5575802973 gives 2.325 days, differing by less than 0.1 day.
Falsified by: A computed days value not rounding to about 2.3, or the difference between the two totals exceeding 0.1 day.

7. Steps: (1) Measure rate on 16 threads with Opus competition (direct observation); (2) Double rate to 32 threads (extrapolation: assumes linear scaling with thread count); (3) Divide second stream's state count (from reconstruction) by 32-thread rate (deterministic arithmetic); (4) Convert seconds to days (deterministic arithmetic).
Falsified by: Evidence that the rate does not scale linearly with threads, or the state count used was not from the reconstruction, or the conversion steps were incorrect.

8. The number 210 was produced by a segment-by-segment formula using each segment's write counts, not by running candidate A2's enumerator. Table 3 Row 6 states "the actual candidate-A2 enumerator was never actually run to produce this number" and "the number is still to be handed to the user labeled as derived, computed via the closed-form", confirming it is formula-derived.
Falsified by: Evidence that candidate A2's enumerator was run to produce 210, or the document not stating it was computed via the closed-form.

9. 2 raised to (2 + 16) equals 262144, matching the total-states figure. 32768 divided by 262144 equals 1/8, which is 1/(2^3).
Falsified by: 2^18 not equaling 262144, or 32768/262144 not equaling 1/8.

10.
5575606380 deterministic arithmetic derivation
13880 per second deterministic arithmetic derivation
2.3 days extrapolation assumes rate scales linearly with thread count
210 deterministic arithmetic derivation
262144 and 32768 directly observed count

Falsified by: For 5575606380, if it was not computed from the segment sizes using the formula; for 13880 per second, if not derived from the raw states and time; for 2.3 days, if the linear scaling assumption was invalid; for 210, if not computed via the closed-form formula; for the pair, if not directly observed from enumeration logs.
