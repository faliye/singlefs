K1SCOPEGATE
WHATHAPPENS The write scope gate allows any write to crates/mutations.tsv because it only checks the file path and does not inspect content changes. This means both new rows or modifying existing rows are both permitted by the gate. The situation can occur as the gate's behavior is purely path-based and does not enforce the append-only restriction described in fact 2.
GATE Gate stage 33 and gate stage 59 would fail if existing rows are modified. Gate stage 33 checks for exact count of original text in files and would detect incorrect counts due to modifications. Gate stage 59 requires original text to be found exactly once during replay and would fail if modifications alter this condition.
This would be refuted by a write that modifies an existing row in crates/mutations.tsv where the original text count remains exactly one, causing gate stage 33 to pass successfully.

K1PATCHCOPY
WHATHAPPENS When applying patches from separate repository copies, each append operation is permitted by the write scope gate. The crates/mutations.tsv file ends up with all new rows appended at the end in the order of patch application without modifying existing rows.
GATE Gate stage 33 would fail if any two rows share the same mutation name or identical combinations of file path original text replacement text and expected failing test. Gate stage 59 would fail if any row's original text is not found exactly once in its named file during replay.
This would be refuted by a PATCHCOPY scenario where all appended rows have unique mutation names and correct original text counts causing gate stage 33 and gate stage 59 to pass successfully.

K1GATE33
WHATHAPPENS Gate stage 33 scans crates/mutations.tsv and checks each row's original text count. If new rows are correctly appended without modifying existing rows the existing rows' counts remain correct and the new rows' original text is present in their specified files.
GATE Gate stage 33 would fail if the new row has a duplicate mutation name or if the original text count in its specified file is not exactly one.
This would be refuted by a new row appended by the experiment runner that has a unique mutation name and correct original text count causing gate stage 33 to pass successfully.

K1GATE59
WHATHAPPENS Gate stage 59 replays each row of crates/mutations.tsv. If new rows are correctly appended the original text is found exactly once in their specified files and the file paths are within the copied paths.
GATE Gate stage 59 would fail if the new row's original text is not found exactly once in its named file or if the file path falls outside the specific set of paths copied into scratch directories.
This would be refuted by a new row appended that has correct original text and file path causing gate stage 59 to pass successfully.

K1BATCHRENAME
WHATHAPPENS The BATCHRENAME process modifies existing rows in crates/mutations.tsv. The write scope gate allows this if the agent performing the rename has write permission for the file path as per its own write scope entry.
GATE Gate stage 33 would fail if the modifications cause incorrect original text counts in files. Gate stage 59 would fail if the original text is not found exactly once during replay due to the modifications.
This would be refuted by a BATCHRENAME process that modifies rows correctly preserving the original text count for each row causing gate stage 33 and gate stage 59 to pass successfully.

K2OWNRULE1
WHATHAPPENS Crash verifier checks if another gate.sh process is already running gate stage 54 before starting its own run with the full flag. If another run is active crash verifier waits for it to finish before proceeding.
GATE No numbered gate stages would flag anything wrong because the waiting behavior adheres to fact 7 and does not violate any gate stage requirements.
This would be refuted by crash verifier starting gate stage 54 with full flag while another gate.sh is already running stage 54 causing a conflict or error.

K2LONGWAIT
WHATHAPPENS Gate stage 54 with full flag is started as a background job per fact 9. This work is waited on without timeout or killing as required for long running operations.
GATE No numbered gate stages would flag anything wrong because the background launch option ensures no timeout or killing occurs.
This would be refuted by gate stage 54 with full flag being killed or timed out during execution.

K2MAINSYNC
WHATHAPPENS The main agent runs gate stage 54 with full flag at its sync point when crates/ is touched. Crash verifier's changed rule does not affect this behavior. Both can run gate stage 54 with full flag at different times.
GATE Gate stage 54 would fail during the whole gate sequence's check if the marker file was deleted by crash verifier's run before the gate sequence's check verifies as the check only verifies the marker file's presence.
This would be refuted by the main agent and crash verifier both running gate stage 54 with full flag in a sequence where the marker file remains present during the gate sequence's check such as if crash verifier runs after the gate sequence's check.

K2REDMARKER
WHATHAPPENS When gate stage 54 is run with full flag it deletes any existing marker file at the start and only writes a new one at the very end if all checks pass and the hash matches.
GATE Gate stage 54 would fail during the whole gate sequence's check if the run was interrupted or failed leaving no marker file.
This would be refuted by gate stage 54 run with full flag completing successfully resulting in a marker file being written which then passes the gate sequence's check.
