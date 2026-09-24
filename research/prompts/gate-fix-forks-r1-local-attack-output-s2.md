1
B1: experiment binary; justification: it is described as measuring costs for four forks (multiple arms) and is run by driver_e156 in the replay script, so it is part of the experiment process; would it fail red: no; exemption row: no exemption row needed
B2: apparatus and tooling component; justification: the binary's own doc comment describes it as meant to be run by hand, taking two log file paths and several device size numbers as command line arguments, and it is not run by any driver function, so it is a manual tool not part of the automated gate process; would it fail red: yes; exemption row: needs exemption row
B3: experiment binary; justification: it runs the entire write path on two real virtio disks and recovers, which is a full experiment test, and it is run by VM scripts, so it is part of the experiment process; would it fail red: no; exemption row: no exemption row needed
B4: apparatus and tooling component; justification: it is a dry run for measurement that prints bytes to regions and its output is fed to another binary in the research directory that does the actual checking, so it is a tooling component that supports the experiment but does not itself perform the comparison; would it fail red: yes; exemption row: needs exemption row
This would be refuted by: if B2 was run by a driver function in the replay script or if B4's output was not passed to another binary in the research directory.

2
2
This would be refuted by: if either B2 or B4 had a non-zero assertion count as per fact 4-5.

3
The facts decide it for both B2 and B4. For B2: the exact phrase from fact 4-4 is "no driver function in that script names this binary anywhere; the binary's own doc comment describes it as meant to be run by hand, taking two log file paths and several device size numbers as command line arguments". For B4: the exact phrase from fact 4-4 is "a function named driver_e142 runs this binary directly through cargo, captures its stdout into a temporary file, then passes that temporary file's path as the first command line argument to a second, separately compiled binary that lives under research/e7-index-bench/src/bin".
This would be refuted by: if the driver function for B2 was found in the replay script or if B4's output was not passed to another binary in the research directory.

4
R1: MISSING-FROM-REPO; justification: the script is in the repository as stated in the quote "it lives under .claude/scripts/ in this same repository, right alongside the gate stages"; current exit code consistent with fact 7-1: consistent
R2: MISSING-FROM-REPO; justification: the script is in the repository as stated in the quote "it lives under .claude/scripts/ in this same repository, right alongside the gate stages"; current exit code consistent with fact 7-1: conflicting
R3: MISSING-FROM-REPO; justification: the quote says "it travels with the repository (under .claude/scripts/)"; current exit code consistent with fact 7-1: consistent
R4: MISSING-FROM-REPO; justification: the quote says "if it was never written, write it first, because without it not one citation has ever actually been checked" implying it should be in the repo; current exit code consistent with fact 7-1: consistent
R5: MISSING-FROM-REPO; justification: the quote says "the shared copy of project rules is not installed here, fetch it and run its own install script" implying it should be in the repo; current exit code consistent with fact 7-1: consistent
R6: MISSING-FROM-REPO; justification: the quote says "it travels with the repository, its absence means it was deleted or moved"; current exit code consistent with fact 7-1: consistent
R7: MISSING-FROM-REPO; justification: the script is under research/scripts/ which is part of the repository, and the stage's checking logic lives inside this script; current exit code consistent with fact 7-1: conflicting
R8: NOTHING-TO-JUDGE; justification: the quote says "can't find $IDX, this stage has nothing to judge"; current exit code consistent with fact 7-1: not applicable
R9: NOTHING-TO-JUDGE; justification: the quote says "can't find $DEC, this stage is skipped"; current exit code consistent with fact 7-1: not applicable
R10: NOTHING-TO-JUDGE; justification: the quote says "can't find kb, this stage is skipped"; current exit code consistent with fact 7-1: not applicable
R11: NOTHING-TO-JUDGE; justification: the quote says "can't find $IDX or $EXP, this stage is skipped"; current exit code consistent with fact 7-1: not applicable
R12: NOTHING-TO-JUDGE; justification: the quote says "can't find $IDX, this stage is skipped"; current exit code consistent with fact 7-1: not applicable
R13: NOTHING-TO-JUDGE; justification: the quote says "can't find the experiment body, this stage is skipped"; current exit code consistent with fact 7-1: not applicable
R14: NOTHING-TO-JUDGE; justification: the quote says "can't find $FTL / $MS / $DEC, this stage is skipped"; current exit code consistent with fact 7-1: not applicable
R15: NOTHING-TO-JUDGE; justification: the quote says "can't find $L1 / $L2 / $MS, this stage has nothing to judge"; current exit code consistent with fact 7-1: not applicable
R16: NOTHING-TO-JUDGE; justification: the quote says "can't find $EXP_DIR, this stage is skipped"; current exit code consistent with fact 7-1: not applicable
R17: NOTHING-TO-JUDGE; justification: the quote says "can't find $EXP_DIR, this stage is skipped"; current exit code consistent with fact 7-1: not applicable
R18: NOTHING-TO-JUDGE; justification: the quote says "can't find .claude/kb, this stage has nothing to judge"; current exit code consistent with fact 7-1: not applicable
R19: NOTHING-TO-JUDGE; justification: the quote says "not inside a git repository, this stage is skipped"; current exit code consistent with fact 7-1: not applicable
R20: NOTHING-TO-JUDGE; justification: the quote says "not inside a git repository, this stage is skipped"; current exit code consistent with fact 7-1: not applicable
This would be refuted by: if the script for R1 or R2 was not part of the repository or if the kb missing quotes indicated something else.

5
18; for R2: consistent; for R7: consistent
This would be refuted by: if R2 or R7 had a current exit code of 1 or if fact 7-1 did not require failing red.

6
5; for R1: conflicting; for R3: conflicting; for R4: conflicting; for R5: conflicting; for R6: conflicting
This would be refuted by: if fact 7-1 allowed skipping for MISSING-FROM-REPO or if the current exit codes for R1 R3 R4 R5 R6 were already 77.

7
The text decides that exit code 1 is correct for this dependency. The exact phrase is "Even when the source tree is missing, this stage still fails red; it must not be treated as a skip".
This would be refuted by: if the script for R1 and R2 was not part of the repository.

8
fact 8-2 row, KEEP: prevented; justification: the new clip does not run the alternating patterns because there was no real truncation, so the character loss is prevented
fact 8-2 row, VOCAB: prevented; justification: the new clip does not run the alternating patterns because there was no real truncation, so VOCAB does not act but the loss is prevented by the same mechanism
fact 8-3 row, KEEP: not prevented; justification: the first step cuts the algorithm name unconditionally, and the alternating patterns do not fix the cut because the resulting string does not match the stripping patterns
fact 8-3 row, VOCAB: not prevented; justification: VOCAB only acts during the alternating patterns step and does not affect the first step, so it cannot prevent the cut in the first step
This would be refuted by: if VOCAB changed the first step of clip or if the character loss for fact 8-2 was not prevented by KEEP new clip.

9
No. It only acts on the alternating patterns that come after it.
This would be refuted by: if VOCAB modified the first step of clip or if the definition of VOCAB included actions before the first step.

10
It only tells you how many files currently use this marker for the unrelated tool named in fact 8-4.
This would be refuted by: if the marker lines in the 21 files contained every possible domain specific term that could be damaged across the whole kb directory.

11
S1, FAIL-LOUD: placed in per-decision section for decision 8; justification: 8 is in today's valid list so it is placed in the normal per-decision section
S1, NEW-SECTION: placed in per-decision section for decision 8; justification: NEW-SECTION only changes behavior for entries with invalid numbers, so S1 is handled the same as FAIL-LOUD
S2, FAIL-LOUD: whole check fails red; justification: 99 is not in today's valid list and there is no new section for it, so it is unplaced causing failure
S2, NEW-SECTION: placed in new section titled "decisions whose body is no longer under the decisions directory"; justification: NEW-SECTION defines this behavior for entries with invalid numbers
S3, FAIL-LOUD: placed in "not attached to any decision" section; justification: extracted number set is empty so it is placed in this section
S3, NEW-SECTION: placed in "not attached to any decision" section; justification: NEW-SECTION only changes behavior for entries with invalid numbers, so S3 is handled the same as FAIL-LOUD
S4, FAIL-LOUD: placed in "not attached to any decision" section; justification: extracted number set is empty so it is placed in this section
S4, NEW-SECTION: placed in "not attached to any decision" section; justification: NEW-SECTION only changes behavior for entries with invalid numbers, so S4 is handled the same as FAIL-LOUD
This would be refuted by: if NEW-SECTION changed behavior for S1 S3 or S4.

12
No. The specific fact whose wording establishes this is fact 9-4: "the generator's own logic has no memory of which decision numbers existed on any day other than today".
This would be refuted by: if the generator consulted a git log or history of decision numbers.

13
S1: no; S3: no; S4: no
This would be refuted by: if NEW-SECTION changed the placement of S1 S3 or S4.

14
Row 1: today: reads the text; with UNIFY: returns None; justification: value text is an arithmetic or naming expression referring to other consts
Row 2: today: reads the text; with UNIFY: returns None; justification: value text is an arithmetic or naming expression referring to other consts
Row 3: today: reads the text; with UNIFY: returns None; justification: value text is an arithmetic or naming expression referring to other consts
Row 4: today: reads the text; with UNIFY: returns None; justification: value text is an arithmetic or naming expression referring to other consts
Row 5: today: reads the text; with UNIFY: returns None; justification: value text is an arithmetic or naming expression referring to other consts
Row 6: today: reads the text; with UNIFY: returns None; justification: value text is an arithmetic or naming expression referring to other consts
Row 7: today: reads the text; with UNIFY: returns None; justification: value text is an arithmetic or naming expression referring to other consts
Row 8: today: reads the text; with UNIFY: returns None; justification: value text is an arithmetic or naming expression referring to other consts
Row 9: today: reads the text; with UNIFY: returns None; justification: value text is an arithmetic or naming expression referring to other consts
Row 10: today: reads the text; with UNIFY: returns None; justification: value text is an arithmetic or naming expression referring to other consts
Row 11: today: reads the text; with UNIFY: returns None; justification: value text is an arithmetic or naming expression referring to other consts
Row 12: today: reads the text; with UNIFY: returns None; justification: value text is an arithmetic or naming expression referring to other consts
Row 13: today: reads the text; with UNIFY: returns None; justification: value text is an arithmetic or naming expression referring to other consts
Row 14: today: reads the text; with UNIFY: returns None; justification: value text is an arithmetic or naming expression referring to other consts
Row 15: today: reads the text; with UNIFY: returns None; justification: value text is a literal compound value such as an array
This would be refuted by: if stage 92 with integer_literal did not return None for any of these declarations.

15
Yes
This would be refuted by: if stage 92 handled None values without failing red.

16
Change detector. The exact phrase from fact 10-3 is "this stage only asks whether that text changed, not what it equals".
This would be refuted by: if stage 92 also checked the value for equality as in stage 27.

17
No. The additional fact needed is a list of marker comments and which declaration names appear in them.
This would be refuted by: if facts 10-1 and 10-4 included information about which declarations have names in marker comments.
