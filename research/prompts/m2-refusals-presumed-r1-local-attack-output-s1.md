1
no fact 4
This would be refuted by any evidence that a byte was written to storage before the refusal fires.
no fact 5
This would be refuted by a zero fault scenario where the journal record is invalid without any faults.
yes
This would be refuted by the error name not accurately describing the specific journal record mismatch condition.
new decision clause
This would be refuted by demonstrating that different choices for handling the condition produce identical bytes on disk and reachable histories.

2
no fact 7
This would be refuted by any evidence that a byte was written to storage before the refusal fires.
no fact 8
This would be refuted by a zero fault scenario where the instance number computations disagree without any transient read errors.
yes
This would be refuted by the error name not specifically describing the instance number mismatch condition.
already fully covered by an existing decision clause fact 10
This would be refuted by evidence that the three decision clauses in fact 10 do not include the two-step recompute mechanism described in facts 6 through 8.

3
no fact 14
This would be refuted by any evidence that a byte was written to storage before the refusal fires.
yes
This would be refuted by the inability to reach the refusal condition through ordinary user input without any faults.
yes
This would be refuted by the error name not specifically describing the region assignment mismatch condition.
already fully covered by an existing decision clause fact 15
This would be refuted by evidence that different region assignments produce observable differences in bytes on disk or reachable histories.

4
no fact 18
This would be refuted by any evidence that a byte was written to storage before the refusal fires.
no fact 18
This would be refuted by evidence that the implementation can produce a root meeting the condition without faults.
yes
This would be refuted by the error name not specifically describing the invalid birth transaction number condition.
register as an unsupported case for the first version
This would be refuted by evidence that multiple defensible choices for handling the condition produce different bytes on disk or reachable histories.
