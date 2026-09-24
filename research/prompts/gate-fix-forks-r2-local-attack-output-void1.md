T4-Q1 This file is an experiment. The single fact supporting this is the doc comment describing it as "E156 first stage: the cost numbers for the four alloc-basis forks", indicating comparison of multiple configurations. This would be refuted if the doc comment stated it compared against a single external ground truth instead of multiple arms.

T4-Q2 This file is a tool. The single fact supporting this is the doc comment stating it compares device logs "against the recorded stream obtained by replaying the same write path on the host", which is an external ground truth. This would be refuted if the doc comment described comparing the two device logs against each other without reference to a ground truth.

T4-Q3 This file is a tool. The single fact supporting this is the doc comment describing it as "run the whole write path of the first transaction on two real virtio disks, cold-reboot, then recover and read the file back", which verifies correctness against expected behavior. This would be refuted if the doc comment described comparing the behavior of the two disks against each other to determine which is better.

T4-Q4 This file is a tool. The single fact supporting this is the doc comment stating it "prints the bytes written to those 21 regions by the first transaction", which is a direct measurement of system behavior. This would be refuted if the doc comment described comparing bytes written between different configurations or arms.

T4-Q5 0. This would be refuted if any row had a classification that did not match the filename match status, such as a tool classified as having a filename matching e<digits>_ or an experiment classified as not matching.

T4-Q6 A file in crates/some-crate/src/bin/algorithm_comparison.rs with doc comment "Compare performance of sorting algorithms A, B, and C across multiple datasets" would be a multi-arm experiment but excluded due to filename not matching e<digits>_. This would be refuted if stage 80 checked its assertions despite the filename mismatch.

T4-Q7 A file in crates/some-crate/src/bin/e123_log_validator.rs with doc comment "Validate device logs against a known good reference stream" would be a tool but included due to filename matching e<digits>_. This would be refuted if stage 80 did not process it despite the filename match.

T7-Q1 For stage 52, no (row A1). For stage 31, no (row A2). This would be refuted if either row showed exit code 1 or "not found" text.

T7-Q2 For stage 52, no (fixture content fact and rows B1-B2). For stage 31, no (fixture content fact and rows B3-B4). This would be refuted if any B row contained "not found" text.

T7-Q3 No. This would be refuted if the temporary worktree contained the new exit-code-1 behavior despite nothing being staged.

T7-Q4 For D3, correct red because the path calculation failed due to missing directory structure. For D5, correct red because the generator was not found at the calculated path despite existing elsewhere. This would be refuted if D3 showed the file existed at the expected path or D5 showed the generator was accessible via the correct path.

T7-Q5 Yes. Rows D5 and D6. This would be refuted if the old code also failed to find the generator in D6 under the same conditions.

T7-Q6 For D7, no. For D8, no. This would be refuted if either row showed "not found" text for the script or generator.

T7-Q7 2 (rows D3 and D5). This would be refuted if any other row showed "not found" text or exit code 1 for missing scripts.

T7-Q8 No. This would be refuted if any B row showed "not found" text or exit code 1 for missing scripts.

T8-Q1 Row 1: dropped-whole-word-unnecessarily. Row 2: dropped-whole-word-unnecessarily. Row 3: correct. Row 4: dropped-whole-word-unnecessarily. Row 5: dropped-whole-word-unnecessarily. Row 6: dropped-whole-word-unnecessarily. Row 7: dropped-whole-word-unnecessarily. Row 8: dropped-whole-word-unnecessarily. Row 9: dropped-whole-word-unnecessarily. Row 10: dropped-whole-word-unnecessarily. Row 11: dropped-whole-word-unnecessarily. Row 12: half-stripped-word. Row 13: dropped-whole-word-unnecessarily. Row 14: half-stripped-word. Row 15: correct. Row 16: correct. Row 17: correct. Row 18: correct. Row 19: correct. Row 20: correct. Row 21: correct. Row 22: correct.

T8-Q2 correct: 9, dropped-whole-word-unnecessarily: 11, half-stripped-word: 2, left-a-bare-reference: 0.

T8-Q3 Yes, row 12. This would be refuted if row 12's output ended with a whole word or was stripped completely without partial characters.

T8-Q4 Different categories. Row 12: half-stripped-word. Row 13: dropped-whole-word-unnecessarily. This would be refuted if both ended in the same category.

T8-Q5 CRC32C ends with a letter, so the regex pattern does not match it, while SHA256 ends with digits and matches. This would be refuted if CRC32C was stripped and SHA256 was not under identical conditions.

T8-Q6 The parenthetical stripping step removes the D22. This would be refuted if the output retained D22 after the parenthetical removal step.