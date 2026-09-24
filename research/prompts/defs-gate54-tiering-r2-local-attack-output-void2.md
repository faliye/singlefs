V3LAYER0
TRACE step 4 count is 3 (LAYER0, CHECKER, LAYER0B lines present), step 5 count is 1 (only LAYER0B line has exhaustive equals true as LAYER0 line now says false)
VERDICT fails at step 5
This would be refuted by: if the CHECKER line contained the phrase exhaustive equals true, then step 5 count would be 2, but fact 1 states the CHECKER line contains only outcome counts and violations, not exhaustive status

V3LAYER0B
TRACE step 4 count is 3 (LAYER0, CHECKER, LAYER0B lines present), step 5 count is 1 (only LAYER0 line has exhaustive equals true as LAYER0B line now says false)
VERDICT fails at step 5
This would be refuted by: if the CHECKER line contained the phrase exhaustive equals true, then step 5 count would be 2, but fact 1 states the CHECKER line contains only outcome counts and violations, not exhaustive status

V3DUPLICATE
TRACE step 4 count is 3 (two LAYER0 lines and one CHECKER line), step 5 count is 2 (both LAYER0 lines have exhaustive equals true)
VERDICT success, but the LAYER0B line is no longer present while all other marker information matches the original state except for the LAYER0B line being missing
This would be refuted by: if the two LAYER0 lines were not both counted in step 4 or if step 5 count was not 2, but fact 2 specifies any line starting with LAYER0 followed by space is counted and both LAYER0 lines have exhaustive equals true

V3CHECKER
TRACE no steps affected (step 4 count remains 3, step 5 count remains 2)
VERDICT success, but the CHECKER line's detailed counts are altered while all other marker information matches the original state
This would be refuted by: if the CHECKER line's content was used in step 4 or 5, but fact 2 explicitly states detailed counts of CHECKER are never used for success or failure decisions

V3THREAD
TRACE no steps affected (step 4 count remains 3, step 5 count remains 2)
VERDICT success, but the worker thread line is altered while all other marker information matches the original state
This would be refuted by: if the worker thread line was used in any step, but fact 4 states the worker thread line is never read back by quick tier for any purpose

V3HASH
TRACE step 3 fails (marker's input_hash does not match freshly computed hash)
VERDICT fails at step 3
This would be refuted by: if the freshly computed hash matched the edited input_hash, but fact 2 states step 3 compares the marker's input_hash against the fresh hash and fails if they differ

V3INPUTFILE
TRACE no steps affected (step 3 succeeds, step 4 count remains 3, step 5 count remains 2)
VERDICT success, but the individual input_file lines are altered while all other marker information matches the original state
This would be refuted by: if the input_file lines were used in step 3, 4, or 5, but fact 2 explicitly states they are only read after failure has already been decided

V3FINISHED
TRACE no steps affected (step 3 succeeds, step 4 count remains 3, step 5 count remains 2)
VERDICT success, the finish time is used only as informational text when reporting success and is not used for any success or failure decisions
This would be refuted by: if the finish time was used in step 3, 4, or 5, but facts 1 and 2 state it is only displayed for informational purposes

V3SCOPE
fact 2's steps 4 and 5 are not capable of catching every tampering of LAYER0 and LAYER0B lines because row V3DUPLICATE shows that deleting LAYER0B and duplicating LAYER0 (with exhaustive equals true) results in step 4 count 3 and step 5 count 2, passing the checks despite the marker structure being incorrect
This would be refuted by: if V3DUPLICATE caused a failure in step 4 or 5, but step 4 count is 3 and step 5 count is 2 as described

V4NARROW
REPLAYRUN does not stop at fact 5 (line exists), reaches fact 8's first question which answers must run (no prior saved state), reaches fact 10's check where real files exist under valid paths so proceeds
REUSEDECISION must run because the shared table file has changed due to the edit
AGREE agree
LATERBATCH reuse helper decides skip because the path touched is not in the combined path list and no differences exist in the checked paths; gate stage 54 stops immediately due to reuse helper's skip outcome
This would be refuted by: if the narrowed paths included invalid paths causing fact 10 to fail or if the shared table file change was not detected by reuse helper, but fact 9 and 6 specify path behavior and table comparison

V4DELETE
REPLAYRUN stops immediately at fact 5's missing row behavior
REUSEDECISION must run because the shared table file has changed due to the edit
AGREE disagree, gate stage 54 fails while reuse helper says must run
This would be refuted by: if gate stage 54 did not stop at fact 5 or reuse helper did not detect the table change, but fact 5 and 6 specify these behaviors

V4WRONGPATH
REPLAYRUN sub case one: does not stop at fact 5 (line exists), reaches fact 8's first question which answers must run (no prior saved state), reaches fact 10's check where valid paths have files so proceeds; sub case two: does not stop at fact 5 (line exists), reaches fact 10's check where real files are empty so stops with its own failure message
REUSEDECISION sub case one: must run because the shared table file has changed; sub case two: must run because the shared table file has changed
AGREE sub case one: agree; sub case two: disagree, gate stage 54 fails while reuse helper says must run
LATERBATCH reuse helper decides skip because the misspelled path matches zero files and the touched path is not in the checked paths; gate stage 54 stops immediately due to reuse helper's skip outcome
This would be refuted by: if the misspelled path was detected as invalid in REPLAYRUN or reuse helper did not detect table changes, but fact 9 and 6 specify path behavior and table comparison

V4SELFCHANGE
REPLAYRUN does not stop at fact 5 (line exists), reaches fact 8's first question which answers must run (no prior saved state), proceeds to fact 10's check where real files exist so proceeds
REUSEDECISION must run because the gate stage 54 script file has changed
AGREE agree
This would be refuted by: if gate stage 54 stopped at fact 5 or reuse helper did not detect script change, but fact 5 and 7 specify table lookup and script inclusion in reuse helper comparison