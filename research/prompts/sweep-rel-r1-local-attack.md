This is a self-contained task. Do not assume any context beyond what is given below. Do not use any markdown emphasis (no bold, no italics, no headers with asterisks or underscores) anywhere in your answer. Write plain text. Number your answers. Do not write any source-code line numbers or file line numbers anywhere in your answer; if you need to point at something, point at it by its row label (L1 through L10) or by the section/entry name already given to you below (for example "entry C22" or "settled item 8"), never by a line number you invent yourself.

BACKGROUND

A software project keeps a set of text files that record open debts, decisions, and status notes about an unfinished storage-engine implementation. Periodically, a "stage" of work finishes and a set of new facts becomes true about the codebase. There is a rule (given in full below, call it "Step 9") for judging, one text passage at a time, whether an existing sentence in the knowledge base still holds up against a stated list of new facts, or needs to change, or is not affected at all.

Below you get: (1) a list of new facts that just became true, (2) the full literal text of Step 9, the rule you must apply, and (3) ten numbered rows (L1 through L10), each one an existing passage from the knowledge base, quoted verbatim (an ellipsis marked [...] means a middle portion of that same sentence was left out because it was not relevant to the fact being tracked; everything before and after the [...] is verbatim). Your job is to apply Step 9 literally to each of L1 through L10, one at a time, and answer the questions listed under TASK below.

NEW FACTS (these are the only facts you should treat as newly established; do not assume any other change happened)

Fact 1: As of 2026-09-17, layer 0's crash-point-replay workload consists of two streams: the first transaction, and the fixed script running through publish E, which includes overwrite, release, reopening for writes (the second writable instance), warm-up, rollback, raising F, and reuse.

Fact 2: As of 2026-09-18, the pool-level checker judges 29 items.

Fact 3: As of 2026-09-16, a tree-table entry is 200 bytes, 81 trees per layer, with 76 bytes reserved.

STEP 9, THE RULE YOU MUST APPLY (translated in full from the project's internal agent definition; every clause below is load-bearing, do not drop any of them)

9a. Row-by-row judgment: judge every row given to you; do not wave through a whole group, and do not sample.

9b. If the end point of the change range is a commit, read that version's context with the appropriate command; if the end point is the working tree, read the file. (For this task, treat the ten rows below as already being read from the correct end-point version; you do not need to run any command yourself.)

9c. Judgment criterion: for every clause in a row that describes the current state of affairs (clauses such as "already has X", "does not yet have X", "only has X", "N items", "not yet started", "prerequisite: X", "still owed: X", "waiting on X"), judge whether that specific clause is still true given the new facts above. You are judging the clause itself, not whether the overall debt item or decision it belongs to has been resolved: even if the debt item is still open, if the prerequisite or reason that clause states has become outdated because of a new fact, that clause is still judged "needs change".

9d. For a clause that carries a date: it counts as an "event clause" only if what follows the date describes something that happened on that specific day (for example: was changed to, was decided, was run once). If instead what follows the date describes the state of affairs as of that day (for example, the second half of a sentence like "as of 2026-09-14, X already exists; layer 0 only has the first transaction"), then judge it as a "current-state clause", not an event clause.

9e. When it is unclear from the passage itself what the up-to-date fact is, defer to whatever authoritative source describes the end-point version (for this task, defer only to the three New Facts given above; do not invent any other fact).

9f. Verdict: pick exactly one of these five labels for the row: "needs-change" (the current-state clause is now false because of a new fact; give the revised sentence), "needs-addition" (this stage produced something new that should have been recorded in this passage but was not), "event-clause-no-change" (the clause describes something that happened on a specific occasion, not a state of affairs, so a later fact cannot make it false), "irrelevant" (the passage's words overlap with a new fact's topic, but the passage is not actually talking about the same fact), "needs-human-review" (you cannot tell which of the above applies).

9g. Whatever verdict you give, write out either the revised sentence (for needs-change or needs-addition) or your reasoning (for the other three labels): state what the row is actually saying, and why it is, or is not, affected by the new facts.

THE TEN ROWS

Row L1. Carrier: file .claude/kb/checks-owed.md, entry C22, its "Prerequisite" column.
Quoted text: "Transaction layer, allocator, root ring, crash-point-replay harness (as of 2026-09-14 all four already exist: crates/singlefs-core, gate 54; layer 0's workload only has the first transaction, without release or reuse)."

Row L2. Carrier: file .claude/kb/checks-owed.md, entry C13, its "Prerequisite" column.
Quoted text: "Layer-1 harness, a checker for each layout. Warning: as of 2026-09-14 the first layout's form exists: the pool-level checker judges 23 items, each one paired with a bad-mirror image [...]; the layer-1-harness half is still owed."

Row L3. Carrier: file .claude/kb/decisions/16 (release semantics, D16), its "settled item 8" section.
Quoted text: "Warning: progress on the prerequisite (2026-09-13, E150): [...] the change of checkpoint_txg from 1 to 3 for the first transaction has already landed (2026-09-13) [...]; the crash-point-replay harness check item is still owed."

Row L4. Carrier: file .claude/kb/checks-owed.md, entry C333, its "Prerequisite" column.
Quoted text: "The implementation of row reclamation; the multi-mount recording stream."

Row L5. Carrier: file .claude/kb/decisions/08 (core index structure, D8), its "settled item 8" section.
Quoted text: "Each entry is 18 bytes (length 2 plus 16 reserved) [...] (using the 2026-09-06 entry widths: inode tree 175 to 171 [...])."

Row L6. Carrier: file .claude/kb/verification-build.md, its "crash-point replay, first version" section.
Quoted text: "2026-09-14 status: [...] the overwrite, release, multi-mount, and rollback that the remaining seven items need are not yet in layer 0's workload."

Row L7. Carrier: file .claude/kb/milestone/02-second-txn.md, its "step 6" section.
Quoted text: "Status (partially landed as of 2026-09-17): [...] each state gets two recovery passes, plus a checker for 26 invariants, plus a record cross-checker."

Row L8. Carrier: file records/2026-09-13-full-audit.md, under the section heading "11.9, C329 and C330: three rounds of three-way argumentation, user ruling, and write-back (2026-09-14)".
Quoted text: "Not yet done: the multi-mount recording stream (the checks for C329, C330, and five newly opened items are all waiting on it)."
Note: the date 2026-09-14 appears only in the section heading shown above, not inside the quoted sentence itself.

Row L9. Carrier: file .claude/kb/decisions/26 (background compaction and placement reclaim, D26), section "Argumentation material for the six undecided items", the row labeled "5, accounting interaction" in a decision table from an earlier round (2026-09-03) of three-way argumentation.
Context, quoted verbatim from that section's own heading note (this sentence is part of the original source file, immediately under the section title, not something added for this task): "Warning: this is that round's record, not the current state (annotated 2026-09-13): sub-items 1, 2, 3, and 6 have already been settled by user ruling; the 'three-way choice among location-authority architectures' inside sub-item 5 has already been settled as the hybrid form by D19 (block pointer structure and width budget), settled item 5; the current state is governed by the 'Settled Items' table and the 'Undecided Items' table instead."
Quoted text (the table row itself): "[from the 'three-leg verdict' cell of that row] Local: what a deadlist entry holds -- D5 has not decided this. [...] [from the 'disposition candidate' cell of the same row] [...] the content of a deadlist entry will be decided together with the location authority."

Row L10. Carrier: file .claude/kb/checks-owed.md, entry C157, its "Check" column.
Quoted text: "This half of the kb has been closed out as of 2026-09-13: D22 (how unit atomicity is composed) settled item 7 has been unified to compute 112 trees per layer using a 145-byte entry."

TASK

For each of the ten rows L1 through L10, in order, answer all of the following. Do not answer with only yes or no anywhere; every answer must be filled in with actual content. Do not skip a row and do not merge two rows into one answer.

1. List every clause in that row's quoted text that describes a current state of affairs (an "already has", "does not yet have", "only has", a count, "not yet started", "prerequisite: X", "still owed: X", "waiting on X", or similar). Quote each such clause verbatim from the text given to you above; do not paraphrase it. If a row has more than one such clause, list each one separately. If a row has none, say so explicitly and explain what kind of clause it is instead.

2. For each clause you listed in question 1: does it carry a date? Answer yes or no for each clause. If yes, say explicitly whether what follows the date describes something that happened on that specific day (an event), or describes the state of affairs as of that day (a status). Justify your answer for each clause using the wording of the clause itself, per rule 9d above.

3. Give the row's verdict: exactly one of the five labels from rule 9f (needs-change, needs-addition, event-clause-no-change, irrelevant, needs-human-review). State which clause (quote it) and which of the New Facts (Fact 1, Fact 2, or Fact 3, or none) drove this verdict. If different clauses in the same row would individually get different verdicts, say so explicitly, explain the conflict, and state which verdict you are giving the row as a whole and why. Per rule 9g, write out the revised sentence if your verdict is needs-change or needs-addition, or your reasoning otherwise.

4. State what observation would overturn the verdict you gave in question 3. Be concrete: name the specific new fact, document, or number that, if it turned out to say something different from what you assumed, would flip your verdict to a different one of the five labels.

Answer row by row, L1 first through L10 last. Use plain numbered text, no markdown emphasis, no tables required (plain numbered lists are fine), no file or code line numbers anywhere in your answer.
