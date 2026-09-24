T4-Q1 experiment. The doc comment mentions "four alloc-basis forks" which indicates comparing multiple arms.
This would be refuted if the doc comment described comparing against an external ground truth instead of multiple arms.

T4-Q2 tool. The doc comment describes comparing device logs against a recorded host replay stream, which is an external ground truth.
This would be refuted if the doc comment described comparing multiple arms against each other.

T4-Q3 tool. The doc comment describes running a write path and recovery process to verify correctness against expected behavior, not comparing multiple arms.
This would be refuted if the doc comment described comparing multiple arms against each other.

T4-Q4 tool. The doc comment describes printing bytes written to regions for measurement, which is generating data rather than comparing multiple arms.
This would be refuted if the doc comment described comparing multiple arms against each other.

T4-Q5 0. All rows agree with their filename match status: row 1 is experiment and matches, rows 2-4 are tool and do not match.
This would be refuted if any row classified as experiment had a filename that did not match e<digits>_ or any tool had a filename that matched.

T4-Q6 The file would be in crates/other/src/bin/my_experiment.rs with filename starting with "my_experiment", doc comment stating it compares four performance arms. This would be refuted if stage 80 checked the file and found absolute assertions.

T4-Q7 The file would be in research/e123_tool/src/bin/e123_tool.rs with filename matching e<digits>_, doc comment stating it compares device logs against host replay stream. This would be refuted if stage 80 did not fail the file for missing absolute assertions.

T7-Q1 no for stage 52 (row A1), no for stage 31 (row A2).
This would be refuted if either row showed exit code 1 with "not found" text.

T7-Q2 no for stage 52 (fixture fact), no for stage 31 (fixture fact).
This would be refuted if any row in occasion B showed "not found" text.

T7-Q3 no. Row C1 states the temporary worktree would be an exact checkout of HEAD, which lacks the exit-1 behavior change.
This would be refuted if row C1 showed the temporary worktree contained the new exit-1 code.

T7-Q4 D3 is correct red because the file genuinely does not exist in the specified root argument directory. D5 is correct red for the same reason.
This would be refuted if the script existed in the root argument directory for either row.

T7-Q5 yes. Rows D4 and D6 show the fallback was used to find the script when the root argument directory contained it.
This would be refuted if rows D4 and D6 showed the fallback was not used.

T7-Q6 no for D7, no for D8.
This would be refuted if either row showed "not found" text for the required script.

T7-Q7 2 (rows D3 and D5).
This would be refuted if any other row showed "not found" text for the required script.

T7-Q8 no.
This would be refuted if any row in occasion B showed "not found" text.

T8-Q1 row 1: dropped-whole-word-unnecessarily. row 2: dropped-whole-word-unnecessarily. row 3: correct. row 4: dropped-whole-word-unnecessarily. row 5: dropped-whole-word-unnecessarily. row 6: dropped-whole-word-unnecessarily. row 7: dropped-whole-word-unnecessarily. row 8: dropped-whole-word-unnecessarily. row 9: dropped-whole-word-unnecessarily. row 10: dropped-whole-word-unnecessarily. row 11: dropped-whole-word-unnecessarily. row 12: half-stripped-word. row 13: dropped-whole-word-unnecessarily. row 14: half-stripped-word. row 15: correct. row 16: correct. row 17: correct. row 18: correct. row 19: correct. row 20: correct. row 21: correct. row 22: correct.

T8-Q2 half-stripped-word: 2, dropped-whole-word-unnecessarily: 11, left-a-bare-reference: 0, correct: 9.

T8-Q3 yes. Row 12 is half-stripped-word and row 14 is half-stripped-word, both end with partial words.
This would be refuted if all rows with partial words were categorized differently.

T8-Q4 row 12: half-stripped-word, row 13: dropped-whole-word-unnecessarily, different categories.

T8-Q5 CRC32C ends with a letter after digits, so it doesn't match the regex pattern requiring digits at the end. SHA256 ends with a digit, so it matches. This would be refuted if CRC32C was stripped or SHA256 was not stripped.

T8-Q6 The parenthetical remark stripping step removes D22 by itself. The t value shows an unbalanced parenthesis with D22 inside, and the output removes the entire parenthetical before id-stripping runs.
This would be refuted if the id-stripping step was responsible for removing D22.
