1 Using Table 2 Row 1's sizes list, the first three segments (2,2,1) yield (2^2-1)+(2^2-1)+(2^1-1)=3+3+1=7. The last segment (2) yields 2^2-1=3. The list has 55 entries summing to 425. The grand total is 5575606380, matching the quoted value. Falsified by: if the sum of sizes is not 425 or the computed total does not equal 5575606380.

2 Table 2 Row 2's sizes list has 54 entries summing to 423. The computed closed-form total is 5575802973, matching the quoted value. Falsified by: if the sum of sizes is not 423 or the computed total does not equal 5575802973.

3 The two lists are not identical. Row 1 has 55 entries summing to 425, Row 2 has 54 entries summing to 423. At position 45 (1-based), Row 1 has 4 and Row 2 has 18. The closed-form totals (5575606380 and 5575802973) are not numerically equal. Falsified by: if the lists are identical or the closed-form totals match.

4 The number 5575606380 matches Table 2 Row 1's reconstruction report, not Row 2's direct measurement. Calling it "measured" without qualification is inaccurate because a same-day direct measurement produced a different number. Falsified by: if 5575606380 matches Row 2's value or Row 1's closed-form does not match it.

5 4194320 / 302.2 ≈ 13879.9, which rounds to "about 13880". The two conditions are: (1) Opus attacker competing for CPU (would push rate higher if removed), (2) 16 threads (would push rate higher if increased). Falsified by: if the computed rate does not round to 13880 or the conditions' effects are incorrect.

6 13880 * 2 = 27760 states/sec. 5575606380 / 27760 / 86400 ≈ 2.324 days (rounds to 2.3). Using 5575802973: 5575802973 / 27760 / 86400 ≈ 2.325 days (difference < 0.1 day). Falsified by: if the computed days do not round to 2.3 or the difference exceeds 0.1.

7 1. Raw rate measurement (4194320 states in 302.2s): direct observation. 2. Compute 13880 states/sec: deterministic arithmetic. 3. Double to 32 threads: extrapolation (assumes perfect linear scaling of threads). 4. Divide second stream's state count: extrapolation (assumes rate from first stream applies to second stream). 5. Convert seconds to days: deterministic arithmetic. Falsified by: if any step's classification is incorrect.

8 The number 210 was produced by a segment-by-segment formula using each segment's write counts, not by running candidate A2's enumerator. Table 3 Row 6 states "the candidate-A2 enumerator itself was never actually run to produce this number" and "computed via the closed-form", confirming it is formula-derived. Falsified by: if the enumerator was run or the source does not state it was not run.

9 2^(2+16)=2^18=262144, matching the total-statesstates figure. 32768/262144=1/8=1/(2^3). Falsified by: if 2^18 does not equal 262144 or the fraction is not 1/8.

10
5575606380: deterministic arithmetic derivation from segment sizes.
13880 per second: deterministic arithmetic derivation from direct observation.
2.3 days: extrapolation (assumes linear thread scaling and rate applies to second stream).
210: deterministic arithmetic derivation from segment write counts.
262144 and 32768: 262144 is deterministic arithmetic derivation from segment writes; 32768 is directly observed count.
Falsified by: if any figure's classification is incorrect or assumptions for extrapolations are invalid.