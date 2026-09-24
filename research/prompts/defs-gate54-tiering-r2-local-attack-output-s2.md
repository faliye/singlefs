V3LAYER0
TRACE step 4 count is 3 (LAYER0, CHECKER, LAYER0B lines), step 5 count is 1 (only LAYER0B line has exhaustive equals true, LAYER0 line now says false, CHECKER line has no exhaustive statement)
VERDICT failing result at step 5 due to step 5 count not being exactly two
This would be refuted by: if the CHECKER line contained exhaustive equals true, but fact 1 explicitly states CHECKER line holds counts of outcomes checked and violations, not exhaustive status

V3LAYER0B
TRACE step 4 count is 3 (LAYER0, CHECKER, LAYER0B lines), step 5 count is 1 (only LAYER0 line has exhaustive equals true, LAYER0B line now says false, CHECKER line has no exhaustive statement)
VERDICT failing result at step 5 due to step 5 count not being exactly two
This would be refuted by: if the CHECKER line contained exhaustive equals true, but fact 1 explicitly states CHECKER line holds counts of outcomes checked and violations, not exhaustive status

V3DUPLICATE
TRACE step 4 count is 3 (two LAYER0 lines and one CHECKER line), step 5 count is 2 (both LAYER0 lines have exhaustive equals true, CHECKER line has no exhaustive statement)
VERDICT success but the LAYER0B line is missing and the CHECKER line content unchanged, input hash unchanged, input_file lines unchanged, finish time unchanged, worker thread line unchanged, but the plain LAYER0 line now has two entries instead of one
This would be refuted by: if the two LAYER0 lines were counted as a single line but fact 2 counts each line starting with LAYER0 space individually

V3CHECKER
TRACE step 4 count is 3 (LAYER0, CHECKER, LAYER0B lines), step 5 count is 2 (LAYER0 and LAYER0B lines have exhaustive equals true, CHECKER line has no exhaustive statement)
VERDICT success but the CHECKER line content changed to include violations, all other marker information unchanged
This would be refuted by: if the CHECKER line content affected step 5 count but fact 2 states CHECKER line details are never used for success/failure decisions

V3THREAD
TRACE step 4 count is 3 (LAYER0, CHECKER, LAYER0B lines), step 5 count is 2 (LAYER0 and LAYER0B lines have exhaustive equals true)
VERDICT success but the worker thread line content changed, all other marker information unchanged
This would be refuted by: if the worker thread line was read for success/failure but fact 4 explicitly states it is never read back for any purpose

V3HASH
TRACE step 3 fails because marker's input_hash line differs from freshly computed hash
VERDICT failing result at step 3 due to hash mismatch
This would be refuted by: if the freshly computed hash matched the edited input_hash but the row states input_hash is edited to different text

V3INPUTFILE
TRACE step 3 succeeds (input_hash matches fresh hash), step 4 count is 3, step 5 count is 2
VERDICT success but the input_file lines changed, all other marker information unchanged
This would be refuted by: if input_file lines were used for success/failure decisions but fact 2 states they are only read after failure has been decided

V3FINISHED
TRACE step 4 count is 3, step 5 count is 2, step 6 reads and displays finish time
VERDICT success but the finish time line content changed, all other marker information unchanged; the finish time is only used as informational text describing when the underlying full run happened
This would be refuted by: if the finish time was used for success/failure decisions but fact 2 states it is only displayed as informational text after success

V3SCOPE
fact 2's fourth and fifth steps are not capable of catching every way LAYER0 and LAYER0B lines could be wrong; for example, when LAYER0B is deleted and replaced with a duplicate LAYER0 line, step 4 count remains three and step 5 count remains two, allowing success despite missing LAYER0B. This would be refuted by: if the marker had only one LAYER0 line and no LAYER0B line, step 4 count would be two causing failure, but two LAYER0 lines and no LAYER0B line passes step 4 and 5, showing the check is not comprehensive

V4NARROW
REPLAYRUN does not stop at fact 5 missing row behavior; reaches fact 8 first question; assuming no prior saved passing state, reuse helper outcome is must run; reaches fact 10 check; real list of files is non-empty under registered paths so proceeds
REUSEDECISION outcome is must run because shared table file itself was edited
AGREE they agree that the batch must be checked
LATERBATCH reuse helper outcome is skip because the touched path is not in registered paths; gate stage 54 stops immediately with did not run status
This would be refuted by: if the shared table file was unchanged but the row states the table line was edited

V4DELETE
REPLAYRUN stops immediately at fact 5 missing row behavior
REUSEDECISION outcome is must run because reuse helper sees no line for gate stage 54's script in table
AGREE they disagree; gate stage 54 reports failure while reuse helper reports must run
This would be refuted by: if the table still contained a line for gate stage 54's script but the row states the line was removed entirely

V4WRONGPATH
REPLAYRUN sub case one: does not stop at fact 5; reaches fact 8 first question with reuse helper outcome must run; reaches fact 10 check; real list of files non-empty under valid paths so proceeds
REPLAYRUN sub case two: reaches fact 10 check; real list of files empty so stops with failing message
REUSEDECISION sub case one: outcome is must run because shared table file was edited
REUSEDECISION sub case two: outcome is must run because shared table file was edited
AGREE sub case one: they agree that batch must be checked
AGREE sub case two: they disagree; gate stage 54 fails while reuse helper reports must run
LATERBATCH reuse helper outcome is skip; gate stage 54 stops immediately with did not run status
This would be refuted by: if the misspelled path matched files but fact 9 states misspelled paths match zero files

V4SELFCHANGE
REPLAYRUN does not stop at fact 5; reaches fact 8 first question with reuse helper outcome must run; reaches fact 10 check; real list of files non-empty so proceeds
REUSEDECISION outcome is must run because gate stage 54's script file was edited
AGREE they agree that the batch must be checked
This would be refuted by: if gate stage 54's script file was unchanged but the row states it was edited
