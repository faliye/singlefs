K1SCOPEGATE
WHATHAPPENS: When the experiment runner agent attempts to append a new row to crates/mutations.tsv, the write scope gate checks the target path against the agent's registered patterns. Fact 3 states the gate only checks the file path, not the content change type. Facts 1 and 2 specify the agent's pattern includes crates/mutations.tsv for appending only. Since the path matches, the gate allows the write. The agent's definition enforces appending only, so the write is correct with new rows appended at the end and existing rows unchanged.
GATE: No numbered gate stage would fail because the new rows are correctly appended. Gate 33 (fact 4) checks each row's original text in its specified file; if valid, it passes. Gate 59 (fact 5) replays rows and checks test failures; if valid, it passes. The append follows the rule, so no gate fails.
This would be refuted by: if gate 33 reports a failure for a mutation row that was correctly appended by the experiment runner agent, indicating the original text does not exist exactly once in the target file despite the append being correct.

K1PATCHCOPY
WHATHAPPENS: In PATCHCOPY, multiple people append their own rows to local copies of crates/mutations.tsv and apply patches sequentially. The new write scope rule allows appending appending, so each patch appends to the end. The state of crates/mutations.tsv is all rows appended in order with no modifications to existing rows.
GATE: Gate 33 would fail if two rows share the same mutation name or same file path, original text, replacement text, and expected failing test. Gate 59 would fail if any row's original text is not found exactly once in its target file or if the file is outside copied paths.
This would be refuted by: if gate 33 and gate 59 both pass despite two rows in crates/mutations.tsv having identical mutation names, which violates fact 4's check.

K1GATE33
WHATHAPPENS: Gate 33 checks each row in crates/mutations.tsv. For new rows appended by the experiment runner agent, gate 33 reads the target file specified in each row and counts occurrences of the original text. If valid, it passes; if not, it fails.
GATE: Gate 33 would fail if any new row's original text does not exist exactly once in the target file, or if duplicate mutation names or row combinations exist.
This would be refuted by: if gate 33 passes for a mutation row where the original text is not present in the target file, which would violate fact 4's check.

K1GATE59
WHATHAPPENS: Gate 59 replays each row. For new rows appended by the experiment runner agent, it copies the repository, replaces the original text with the replacement in the target file, runs the test, and checks for failure. If valid, it passes; if not, it fails.
GATE: Gate 59 would fail if any row's original text is not found exactly once in its target file, if the file is outside copied paths, or if the named test does not fail.
This would be refuted by: if gate 59 passes for a mutation row where the original text is not found in the file, which would violate fact 5's check.

K1BATCHRENAME
WHATHAPPENS: BATCHRENAME rewrites rows in crates/mutations.tsv for renaming. If done by the experiment runner agent, the write scope gate rejects the write because modifying existing rows violates the append-only permission. If done via shell command, the gate does not check and the rewrite occurs.
GATE: If the write is rejected by the gate (agent without permission), no gate stage fails. If allowed (shell command), gate 33 would fail for duplicate mutation names or row combinations, and gate 59 would fail for invalid original text or test failures.
This would be refuted by: if the experiment runner agent successfully modified an existing row in crates/mutations.tsv despite the write scope gate blocking it, which would violate fact 3's gate behavior.

K2OWNRULE1
WHATHAPPENS: Crash verifier checks if another gate.sh is running gate stage 54 before starting its own run with the full flag. If another run is active, it waits; otherwise, it proceeds with the full flag.
GATE: No numbered gate stage would flag anything wrong because fact 7 ensures only one gate.sh runs gate stage 54 at a time, and fact 11's marker handling is consistent.
This would be refuted by: if two gate.sh processes run gate stage 54 with the full flag simultaneously, causing the marker file to be deleted by one while the other is running, which would violate fact 11's behavior.

K2LONGWAIT
WHATHAPPENS: Crash verifier starts gate stage 54 with the full flag using the background launch option, waits for completion without timeout, and only reports waiting in its reply.
GATE: No numbered gate stage would flag anything wrong because the long-running job is handled per fact 9, with no timeout or premature termination.
This would be refuted by: if gate stage 54 was killed partway through due to a timeout, violating fact 9's rule.

K2MAINSYNC
WHATHAPPENS: The main agent starts gate stage 54 with the full flag at the end of a crates batch touching crates/. Crash verifier waits for this run to complete before starting its own, as per fact 7. Gate 54 deletes the marker at start and writes it at end if passes.
GATE: No numbered gate stage would flag anything wrong because the marker is properly written and checked, and concurrent runs are handled by fact 7.
This would be refuted by: if gate stage 54's check during the gate sequence fails because the marker file is missing when it should be present, indicating the main agent's run did not complete successfully.

K2REDMARKER
WHATHAPPENS: Crash verifier runs gate stage 54 with the full flag, which deletes the marker file at start and only writes a new one at the end if all checks pass.
GATE: Gate stage 54's check during the gate sequence would fail if the marker file is missing, which is correct when gate 54's full run failed. If the run passes, the marker is present and the check passes.
This would be refuted by: if gate stage 54's check passes despite the marker file being missing, indicating the gate 54 full run did not complete successfully but the marker was present.