1
B1 classification: experiment binary. Justification: fact 4-3 states "E156 stage one: the cost numbers for the four forks of alloc-basis (pre-registered before the run)" which describes an experiment binary. EXTEND fail red: no. Third part: not outside subject matter.
B2 classification: apparatus and tooling component. Justification: fact 4-3 states "Host side: take the two device side logs that a QEMU tool recorded, and compare them item by item, in device order, against the record stream obtained by replaying the same write path on the host with the same parameters and the same bytes" which describes a tooling component for comparison. EXTEND fail red: yes. Third part: not outside subject matter.
B3 classification: experiment binary. Justification: fact 4-3 states "VM tier: run the entire write path of the first transaction on two real virtio disks, then cold reboot and recover, reading the file back" which describes an experiment binary. EXTEND fail red: no. Third part: not outside subject matter.
B4 classification: experiment binary. Justification: fact 4-3 states "The implementation side of one measurement's dry run: run a fixed scenario function on two in memory disks with the same parameters as the VM tier, and print, as one result line per region, the bytes that the first transaction wrote to 21 fixed regions" which describes an experiment binary. EXTEND fail red: yes. Third part: not outside subject matter.
This would be refuted by: if fact 4-3 for B2 described it as part of an experiment rather than a tooling component, or for B4 as a tooling component.

2
2. Binaries B2 and B4 would make stage 80 fail red today under candidate EXTEND with zero exemption table rows filed.
This would be refuted by: if B2 or B4 had a non-zero assertion count matching the pattern.

3
Facts 4-1 through 4-5 already decide the classification for B2 and B4. For B2, the exact phrase is "Host side: take the two device side logs that a QEMU tool recorded, and compare them item by item, in device order, against the record stream obtained by replaying the same write path on the host with the same parameters and the same bytes" from fact 4-3. For B4, the exact phrase is "The implementation side of one measurement's dry run: run a fixed scenario function on two in memory disks with the same parameters as the VM tier, and print, as one result line per region, the bytes that the first transaction wrote to 21 fixed regions" from fact 4-3.
This would be refuted by: if fact 4-3 for B2 did not describe a comparison tool or for B4 did not describe a dry run measurement.

4
R1 classification: MISSING-FROM-REPO. Justification: the missing Python script lives under .claude/scripts/ in the repository and is required for the stage to function, as stated in the quote "it lives under .claude/scripts/ in this same repository, right alongside the gate stages; without it this stage can't judge anything at all, recover it from git". Current exit code consistent with fact 7-1: yes.
R2 classification: MISSING-FROM-REPO. Justification: the missing Python script is identical to R1 and lives in the repository, as confirmed by fact 7-3 and the quote "it lives under .claude/scripts/ in this same repository, right alongside the gate stages; without it this stage can't judge anything at all, recover it from git". Current exit code consistent with fact 7-1: no.
R3 classification: MISSING-FROM-REPO. Justification: the missing shell script travels with the repository and is required for the stage's memory model checking logic, as stated in the quote "it travels with the repository (under .claude/scripts/); if it was deleted, recover it from git". Current exit code consistent with fact 7-1: yes.
R4 classification: MISSING-FROM-REPO. Justification: the missing shell script is relied upon to check every citation one by one and must be present for the stage to function, as stated in the quote "this stage relies on it to check every citation one by one; if it moved, change the path written into this stage; if it was never written, write it first, because without it not one citation has ever actually been checked". Current exit code consistent with fact 7-1: yes.
R5 classification: MISSING-FROM-REPO. Justification: the shared lint script is required and must be fetched and installed, as stated in the quote "the shared copy of project rules is not installed here, fetch it and run its own install script". Current exit code consistent with fact 7-1: yes.
R6 classification: MISSING-FROM-REPO. Justification: the missing Python script travels with the repository and its absence means it was deleted or moved, as stated in the quote "it travels with the repository, its absence means it was deleted or moved, and nobody is checking whether the machine is clean this round". Current exit code consistent with fact 7-1: yes.
R7 classification: MISSING-FROM-REPO. Justification: the missing Python script is forwarded to by the stage and must be present in the repository, as indicated by the path "research/scripts/check-segment-registry.py" and the quote "this stage is skipped" implying it should be present. Current exit code consistent with fact 7-1: no.
R8 classification: NOTHING-TO-JUDGE. Justification: the missing decisions index file means there is nothing to judge, as stated in the quote "can't find $IDX, this stage has nothing to judge". Current exit code consistent with fact 7-1: not applicable.
R9 classification: NOTHING-TO-JUDGE. Justification: the missing decisions directory means there is nothing to judge, as stated in the quote "can't find $DEC, this stage is skipped". Current exit code consistent with fact 7-1: not applicable.
R10 classification: NOTHING-TO-JUDGE. Justification: the missing .claude/kb directory means there is nothing to judge, as stated in the quote "can't find kb, this stage is skipped". Current exit code consistent with fact 7-1: not applicable.
R11 classification: NOTHING-TO-JUDGE. Justification: the missing index or experiments directory means there is nothing to judge, as stated in the quote "can't find $IDX or $EXP, this stage is skipped". Current exit code consistent with fact 7-1: not applicable.
R12 classification: NOTHING-TO-JUDGE. Justification: the missing decisions index file means there is nothing to judge, as stated in the quote "can't find $IDX, this stage is skipped". Current exit code consistent with fact 7-1: not applicable.
R13 classification: NOTHING-TO-JUDGE. Justification: the missing experiment body file means there is nothing to judge, as stated in the quote "can't find the experiment body, this stage is skipped". Current exit code consistent with fact 7-1: not applicable.
R14 classification: NOTHING-TO-JUDGE. Justification: the missing kb paths mean there is nothing to judge, as stated in the quote "can't find $FTL / $MS / $DEC, this stage is skipped". Current exit code consistent with fact 7-1: not applicable.
R15 classification: NOTHING-TO-JUDGE. Justification: the missing kb paths mean there is nothing to judge, as stated in the quote "can't find $L1 / $L2 / $MS, this stage has nothing to judge". Current exit code consistent with fact 7-1: not applicable.
R16 classification: NOTHING-TO-JUDGE. Justification: the missing experiments directory means there is nothing to judge, as stated in the quote "can't find $EXP_DIR, this stage is skipped". Current exit code consistent with fact 7-1: not applicable.
R17 classification: NOTHING-TO-JUDGE. Justification: the missing experiments directory means there is nothing to judge, as stated in the quote "can't find $EXP_DIR, this stage is skipped". Current exit code consistent with fact 7-1: not applicable.
R18 classification: NOTHING-TO-JUDGE. Justification: the missing .claude/kb directory means there is nothing to judge, as stated in the quote "can't find .claude/kb, this stage has nothing to judge". Current exit code consistent with fact 7-1: not applicable.
R19 classification: NOTHING-TO-JUDGE. Justification: not being inside a git repository means there is nothing to judge, as stated in the quote "not inside a git repository, this stage is skipped". Current exit code consistent with fact 7-1: not applicable.
R20 classification: NOTHING-TO-JUDGE. Justification: not being inside a git repository means there is nothing to judge, as stated in the quote "not inside a git repository, this stage is skipped". Current exit code consistent with fact 7-1: not applicable.
This would be refuted by: if for R4 the quote did not state it relies on the script to check every citation, or for R1 the script was not described as living in the repository.

5
15 rows change away from their current exit code under candidate UNIFORM-1. The rows classified MISSING-FROM-REPO whose exit code changes are R2 and R7. For both, the new exit code 1 is consistent with fact 7-1.
This would be refuted by: if R2 or R7 did not change to exit code 1, or if exit code 1 was inconsistent with fact 7-1 for them.

6
5 rows change away from their current exit code under candidate UNIFORM-77. The rows classified MISSING-FROM-REPO whose exit code changes are R1, R3, R4, R5, R6. For all of them, the new exit code 77 is in conflict with fact 7-1.
This would be refuted by: if R1, R3, R4, R5, or R6 did not change to exit code 77, or if exit code 77 was consistent with fact 7-1 for them.

7
The text already decides which exit code is correct for R1 and R2. The exact phrase is "with the script or the recorded fallback log both missing: fail red, do not treat it as a skip, because it travels with the repository, and its absence is exactly a case of something that should be in the repository not being there, the same rule as stages 57, 70, and 73" from fact 7-4.
This would be refuted by: if fact 7-4 did not reference stages 57, 70, and 73 having the same rule, or if those stages did not fail red.

8
For fact 8-2 row:
- KEEP: yes. Justification: the fixed clip logic in fact 8-1 detects no real truncation (input under limit) and returns unchanged, preventing character loss.
- VOCAB: no. Justification: the trailing Chinese particle is not a token in the doc-lint marker line (fact 8-4), so VOCAB does not skip stripping it.
For fact 8-3 row:
- KEEP: no. Justification: the character limit cut lands inside the algorithm name, and VOCAB does not affect this step, so loss occurs.
- VOCAB: no. Justification: VOCAB only acts during the alternating patterns step after the first cut, not during the first cut step, so it cannot prevent loss from the initial cut.
This would be refuted by: if for fact 8-2 the did not prevent the loss, or for VOCAB the particle was in the marker tokens.

9
Candidate VOCAB does not act on anything before or during the first step. It only acts during the alternating patterns step that occurs after the first character limit cut and parenthesis balancing, as defined by "when the text clip is about to strip from the end matches one of these listed tokens exactly, skip stripping it, even though real truncation did happen".
This would be refuted by: if the definition of candidate VOCAB included actions during the character limit cut step.

10
This number only tells how many files currently use this marker for the unrelated tool named in fact 8-4, not how completely candidate VOCAB's vocabulary list would cover every domain specific term that could be damaged.
This would be refuted by: if the marker line contained all possible domain specific terms that could be damaged by the failure mode in fact 8-3.

11
S1 with FAIL-LOUD: placed in normal per-decision section for decision 8. Justification: fact 9-3 states such entries are collected into a normal per-decision section for valid decisions.
S1 with NEW-SECTION: placed in normal per-decision section for decision 8. Justification: NEW-SECTION is only defined for S2, so it does not change S1's placement.
S2 with FAIL-LOUD: whole check fails red. Justification: fact 9-3 states such entries end up in neither category and cause failure.
S2 with NEW-SECTION: placed in new section "decisions whose body is no longer under the decisions directory". Justification: NEW-SECTION is defined to place such entries into this new section.
S3 with FAIL-LOUD: placed in "not attached to any decision" section. Justification: fact 9-3 states entries with empty extracted number set are placed in this section.
S3 with NEW-SECTION: placed in "not attached to any decision" section. Justification: NEW-SECTION is only defined for S2, so it does not change S3's placement.
S4 with FAIL-LOUD: placed in "not attached to any decision" section. Justification: fact 9-3 states entries with empty extracted number set are placed in this section.
S4 with NEW-SECTION: placed in "not attached to any decision" section. Justification: NEW-SECTION is only defined for S2, so it does not change S4's placement.
This would be refuted by: if for S2 with NEW-SECTION it was placed in a different section, or for S1 with NEW-SECTION it was not placed in the normal section.

12
No, the generator's logic has no information to tell these two causes apart. The specific fact is "nothing in facts 9-1 through 9-3 reads or consults any change history file, any git log, or any other record of which decision numbers were ever valid in the past" from fact 9-4.
This would be refuted by: if the generator had access to historical decision numbers or git logs.

13
For S1: no. Justification: from table cells in question 11, both candidates place S1 in the normal per-decision section.
For S3: no. Justification: from table cells in question 11, both candidates place S3 in the "not attached to any decision" section.
For S4: no. Justification: from table cells in question 11, both candidates place S4 in the "not attached to any decision" section.
This would be refuted by: if NEW-SECTION changed the placement of S1, S3, or S4.

14
For row 1 (scalar type, expression naming other consts): today: successfully reads and compares the text. With UNIFY: fails to read as integer literal, so comparison cannot be performed.
For row 2 (scalar type, pure arithmetic expression): today: successfully reads and compares the text. With UNIFY: fails to read as integer literal, so comparison cannot be performed.
For row 3 (scalar type, expression naming other consts): today: successfully reads and compares the text. With UNIFY: fails to read as integer literal, so comparison cannot be performed.
For row 4 (scalar type, pure arithmetic expression): today: successfully reads and compares the text. With UNIFY: fails to read as integer literal, so comparison cannot be performed.
For row 5 (scalar type, expression naming other consts): today: successfully reads and compares the text. With UNIFY: fails to read as integer literal, so comparison cannot be performed.
For row 6 (scalar type, pure arithmetic expression): today: successfully reads and compares the text. With UNIFY: fails to read as integer literal, so comparison cannot be performed.
For row 7 (scalar type, expression naming other consts): today: successfully reads and compares the text. With UNIFY: fails to read as integer literal, so comparison cannot be performed.
For row 8 (scalar type, pure arithmetic expression): today: successfully reads and compares the text. With UNIFY: fails to read as integer literal, so comparison cannot be performed.
For row 9 (scalar type, expression naming other consts): today: successfully reads and compares the text. With UNIFY: fails to read as integer literal, so comparison cannot be performed.
For row 10 (scalar type, pure arithmetic expression): today: successfully reads and compares the text. With UNIFY: fails to read as integer literal, so comparison cannot be performed.
For row 11 (scalar type, expression naming other consts): today: successfully reads and compares the text. With UNIFY: fails to read as integer literal, so comparison cannot be performed.
For row 12 (scalar type, pure arithmetic expression): today: successfully reads and compares the text. With UNIFY: fails to read as integer literal, so comparison cannot be performed.
For row 13 (scalar type, expression naming other consts): today: successfully reads and compares the text. With UNIFY: fails to read as integer literal, so comparison cannot be performed.
For row 14 (scalar type, pure arithmetic expression): today: successfully reads and compares the text. With UNIFY: fails to read as integer literal, so comparison cannot be performed.
For row 15 (compound type, array literal): today: successfully reads and compares the text. With UNIFY: fails to read as integer literal, so comparison cannot be performed.
This would be refuted by: if for any row, stage 92 did not successfully read today or did not fail with UNIFY.

15
Yes, stage 92 would fail red purely because of these 15 declarations.
This would be refuted by: if stage 92 handled None values by skipping or defaulting without failing.

16
Stage 92 is best described as a change detector. The exact phrase from fact 10-3 is "this stage only asks whether that text changed, not what it equals".
This would be refuted by: if fact 10-3 stated it compared values for equality.

17
No, from the facts given alone, cannot tell how many of the 15 declarations have a name that currently appears in any marker comment.
This would be refuted by: if fact 10-1 specified which declarations are checked against marker comments.
