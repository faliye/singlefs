K1SCOPEGATE
WHATHAPPENS: When the experiment runner agent writes to crates/mutations.tsv, the write scope gate allows the write because the path matches the new entry in the write scope table (fact 2). The gate cannot distinguish appending from overwriting, but the agent's definition (fact 1) restricts it to appending only, so the write occurs as an append with no modification to existing rows.
GATE: No numbered gate stages would produce a failing result because the state of crates/mutations.tsv after correct appending would satisfy gate 33's checks (fact 4) and gate 59's checks (fact 5), assuming the new rows are correctly formatted.
This would be refuted by gate 33 reporting a failure for a new row where the original text was not found exactly once in the target file.

K1PATCHCOPY
WHATHAPPENS: The new write scope rule does not affect PATCHCOPY because the write operations in PATCHCOPY are performed via shell commands (fact 3), which bypass the write scope gate entirely. The file is updated by applying patches normally, appending new rows without gate interference.
GATE: No numbered gate stages would produce a failing result because the write scope gate is not involved in shell command writes, and gate 33 and gate 59 check row validity regardless of write method.
This would be refuted by gate 33 reporting a failure for a row added via PATCHCOPY due to incorrect row data, which would indicate a problem with the patch application, not the write scope rule.

K1GATE33
WHATHAPPENS: When the experiment runner appends a new row to crates/mutations.tsv, gate 33 checks each row's original text against its target file. If the new row's original text is present exactly once, gate 33 passes; otherwise, it fails. The write scope rule allows the append but does not affect the content validity checks.
GATE: Gate 33 would fail only if the new row's original text is not found exactly once in its target file, but the write scope rule itself does not cause this failure.
This would be refuted by gate 33 reporting a failure for a new row where the original text was found exactly once, indicating the gate 33 check is incorrect.

K1GATE59
WHATHAPPENS: When the experiment runner appends a new row, gate 59 replays the row. If the row's original text is found exactly once and the test fails as expected, gate 59 passes; otherwise, it fails. The write scope rule allows the append but does not affect the replay checks.
GATE: Gate 59 would fail only if the new row's replay conditions are not met, but the write scope rule itself does not cause this failure.
This would be refuted by gate 59 reporting a success for a new row where the original text was not found exactly once in the target file.

K1BATCHRENAME
WHATHAPPENS: The new write scope rule for the experiment runner does not affect BATCHRENAME because BATCHRENAME is performed by a different agent or process. The write scope gate checks the agent performing BATCHRENAME's permissions, which are unrelated to the experiment runner's new rule.
GATE: No numbered gate stages would produce a failing result because the write scope rule's scope is specific to the experiment runner, and BATCHRENAME's agent may have separate permissions.
This would be refuted by gate 33 or gate 59 failing due to incorrect mutation names after BATCHRENAME, which would indicate the rename operation was flawed, not the experiment runner's write scope rule.

K2OWNRULE1
WHATHAPPENS: Crash verifier's changed rule (fact 8) and fact 7 coexist because crash verifier waits for another gate.sh process running stage 54 before starting its own run with the full flag. The two facts can be true simultaneously without conflict.
GATE: No numbered gate stages would flag anything wrong because the waiting behavior prevents concurrent runs, ensuring gate 54 runs sequentially.
This would be refuted by gate 54's script failing due to concurrent runs caused by crash verifier not waiting for another gate.sh process.

K2LONGWAIT
WHATHAPPENS: Crash verifier starts gate stage 54 with full flag as a background job (fact 9), which has no timeout or killing, so the long running work proceeds without interruption.
GATE: No numbered gate stages would flag anything wrong because the background job respects the long running work convention.
This would be refuted by gate 54 being killed due to a timeout during its full run.

K2MAINSYNC
WHATHAPPENS: The main agent and crash verifier both run gate stage 54 with full flag, but crash verifier waits for the main agent's run (fact 7), so they do not run concurrently. The marker file is deleted at start of each run and written only if successful (fact 11), and gate sequence checks occur after runs complete.
GATE: No numbered gate stages would flag anything wrong because gate 54 runs sequentially and the marker is only missing during runs, not during checks.
This would be refuted by gate stage 54 (the marker check) failing because the marker file was missing during its check.

K2REDMARKER
WHATHAPPENS: When crash verifier runs gate stage 54 with full flag, it deletes the marker file at the start (fact 11), runs checks, and only writes a new marker if all checks pass.
GATE: No numbered gate stages would flag anything wrong because the behavior aligns with fact 11's requirements.
This would be refuted by gate stage 54 writing a marker file when a check during its full run failed.
