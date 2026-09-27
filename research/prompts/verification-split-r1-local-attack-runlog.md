# Run log: verification-split-r1, local attack leg

Prompt: research/prompts/verification-split-r1-local-attack.md
Translation audit: research/prompts/verification-split-r1-local-attack-translation-audit.md
Samples requested via `bash research/scripts/ask-local.sh research/prompts/verification-split-r1-local-attack.md > research/prompts/verification-split-r1-local-attack-output-s<n>.md`, each started with Bash's run_in_background, each ended this turn and waited for the completion notification.

## Sample s1

- Command exit code (from the completion notification): 0.
- File size: 6130 bytes (non-zero).
- Word count (`wc -w`): 1007.
- Corruption gate: `ask-local.sh` itself ran corruption-check.py and oov-check.py in sequence and returned 0 (neither judged red), so no void copy was produced.
- Re-ran `python3 research/scripts/oov-check.py` separately, with the prompt file passed so prompt-proper-noun terms are not flagged: green, 2 out-of-vocabulary words, printed in full ("singlefs's contradicted"), well under the 300-character print cap.
- Manual full read-through for the two damage classes outside the gates' signatures (missing words, broken punctuation such as an orphaned list marker with nothing after it): none found; both the 14-item Part One list and the 26-row Part Two table are structurally complete, no glued or headless words spotted.
- Verdict: clean.
- Part Two answer table compared against the ground-truth table in research/prompts/verification-split-r1-classify-facts.md by a short Python script (parses both markdown tables by row number, no answers printed): mismatched row numbers (arithmetic does not match): 1, 2, 4, 5, 6, 10, 13, 17, 18, 23. All other rows (16 of 26) matched the ground truth. No row's cell contents are reproduced here per the write-scope on this leg.

## Sample s2

- Command exit code (from the completion notification): 0.
- File size: 6232 bytes (non-zero).
- Word count (`wc -w`): 1003.
- Corruption gate: `ask-local.sh` returned 0; no void copy was produced.
- Re-ran `oov-check.py` separately: green, 4 out-of-vocabulary words, printed in full ("singlefs's contradicting contradicted"), under the 300-character cap.
- Manual full read-through: none of the two extra damage classes found; both parts structurally complete.
- Verdict: clean.
- Part Two answer table compared the same way against the same ground-truth table: mismatched row numbers: 1, 2, 4, 5, 6, 10, 13, 17, 18, 22, 23. All other rows (15 of 26) matched the ground truth.

## Void copies

- None produced across the two calls (both calls returned exit code 0 with content).

## Clean-sample count

- 2 of 2 calls produced clean samples on the first two calls; the round's minimum of two clean samples is met without needing a third call.

## Answer-table source for the 5b arithmetic check

- research/prompts/verification-split-r1-classify-facts.md (given by the dispatching agent; not referenced inside the prompt file itself, so the local model could not read it).
