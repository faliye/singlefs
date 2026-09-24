TASK

You are given a fixed set of facts about a change to a software project's own operating rules for one of its automated writer roles. You have no access to the repository, no tools, and no ability to look anything up; every fact you need is stated below. Do not assume anything not stated here. Answer in English only. Do not use any markdown emphasis anywhere in your answer: no bold, no italics, no underline, no emphasized headings. Plain numbered or labeled lines, and plain tables using the vertical bar character, are fine. Never cite, invent, or guess at a code line number or a file line number anywhere in your answer, even for facts given below that do carry a source citation in this prompt; when you need to point at something, point at it by its FACT label (for example FACT-1) or by an ARTIFACT label (for example GATE-59-SCRIPT), both defined below. Do not answer any cell of the table below with a bare word such as yes, no, or a bare name; always add the reasoning, tied to a FACT label, that produced that answer. If you cannot tie an answer to any FACT label given below, write UNKNOWN in that cell and say what kind of fact would be needed instead of guessing one. At the end, count and state how many cells across the whole table you filled with UNKNOWN.

BACKGROUND

An automated writer role in this project, called the runner, used to write its experiment code into one location only. A new branch was added to the runner's own rule document: when a separate planning document (written before the runner starts, by a different role) fixes the experiment harness as what this task calls the in repo harness, the runner writes its experiment binary into a different location instead, one that sits inside the same source tree as the project's real implementation code. That branch also states three restrictions meant to keep this new location safe. This task asks you to judge three questions about that branch, called W1, W2, and W3 below, using only the facts given.

ARTIFACT LABELS

EXPERIMENT-RUNNER-DEF names the runner role's own rule document.
WRITE-SCOPE-TABLE names the table that lists, for every automated writer role, which file path patterns that role is allowed to write.
WRITE-GUARD-HOOK names the program that reads the write scope table and blocks a write attempt in real time, before the write happens, whenever the target path does not match any pattern granted to that role.
GATE-63-SCRIPT, GATE-56-SCRIPT, GATE-59-SCRIPT, GATE-96-SCRIPT, GATE-94-SCRIPT each name one numbered, scheduled check that runs over the whole repository at fixed points (for example before a commit), separate from WRITE-GUARD-HOOK's real time check.
GATE-STAGE-DIRECTORY names the folder holding every one of these numbered, scheduled checks.
EXPERIMENT-DESIGNER-DEF names the rule document for the separate planning role that writes the planning document mentioned above.
BODY-DOC names the document that poses the three questions W1, W2, W3 to you.

FACT TABLE

FACT-1 (the new branch's trigger condition and what it changes, translated in full, from EXPERIMENT-RUNNER-DEF). When the planning document fixes the harness as the in repo harness, the runner writes crates/singlefs-harness/src/bin/e-number_short-name.rs instead of the other path. This applies when the fork itself is about the nature of the code under crates as it stands today; writing a separate, standalone model elsewhere cannot answer that fork's question. The write-scope gate grants access for files matching crates/singlefs-harness/src/bin/e*.rs.

FACT-2 (the new branch's three restrictions, translated in full, from EXPERIMENT-RUNNER-DEF, the same sentence as FACT-1). That kind of binary must be read-only: it may only drive the real code and observe it. It must not change crates/singlefs-core or crates/singlefs-checker. Its mutation-table rows are appended to the end of crates/mutations.tsv, and GATE-59-SCRIPT runs them.

FACT-3 (the write-scope table's own row for this new location, translated in full, from WRITE-SCOPE-TABLE). The row grants the runner role the pattern crates/singlefs-harness/src/bin/e*.rs. Its note column reads: a read-only binary for when the experiment harness needs to drive real code under crates. The pattern requires the file name to start with the letter e followed by digits, so it cannot reach the first_transaction_* production harness files. This binary must not change core or checker; that rule is enforced by the code three-way review and GATE-56-SCRIPT.

FACT-4 (WRITE-GUARD-HOOK's own stated decision rule, translated in full). If the hook's input carries no agent type field, meaning the write comes from the main agent, the write is allowed. If the agent type has no project definition, meaning it is a built-in agent, the write is allowed. If the agent has a project definition but WRITE-SCOPE-TABLE has no row for it, the write is refused. After the target path is put into a normal form, the write is allowed only when it matches one of that agent's own granted patterns.

FACT-5 (a text search over WRITE-SCOPE-TABLE, its own method and result). A full text search of every row in WRITE-SCOPE-TABLE for one that grants the runner role write access to any path under crates/singlefs-core or crates/singlefs-checker returns zero rows.

FACT-6 (GATE-63-SCRIPT's own stated checks, translated in full, and what was found by reading its full text). GATE-63-SCRIPT requires that every automated role definition whose tools include Write or Edit has at least one path pattern in WRITE-SCOPE-TABLE. Reading GATE-63-SCRIPT's full text line by line for any check on whether a granted pattern's reach is narrow enough, or any mention of the words core or checker, finds none.

FACT-7 (GATE-56-SCRIPT's own stated judging rule and its own file-matching pattern, translated in full). This gate checks form only: every file matching crates then any crate name then src then any path ending in .rs, that this change touches, must be named by its path inside some three-way verdict document written for the first time for this change, a file matching research/prompts then any name then -main-verification.md. Whether that naming was done in a good way or a bad way still needs a human to look. The gate's own file-matching pattern, read from its source code, selects any changed file whose path starts with crates, then one crate name, then src, then any path ending in .rs; that pattern matches files under crates/singlefs-harness/src/bin, and also files under crates/singlefs-core/src or crates/singlefs-checker/src.

FACT-8 (GATE-59-SCRIPT's own stated judging rule, translated in full). For every mutation row in crates/mutations.tsv, giving a file, the original text, the replacement text, the cargo test arguments, and the test name that must turn red, the original text must match one time and no more inside that file. The gate copies the whole repository into a temporary directory, breaks that one spot, runs the named test, and that test must turn red. Then it restores the file and moves to the next row.

FACT-9 (GATE-96-SCRIPT's own stated judging rule and its own hardcoded file pattern, translated in full). This gate's own judging rule states that it scans files matching research/e7-index-bench/src/bin/*.rs. Its source code sets a variable holding that same file pattern as its one and only scan target. No path under crates/singlefs-harness appears in that pattern.

FACT-10 (GATE-94-SCRIPT's own hardcoded crate paths, translated in full). This gate's source code sets three crate paths as the objects it checks: the checker crate at crates/singlefs-checker, the core crate at crates/singlefs-core, and a shared crate at crates/singlefs-format. No path for crates/singlefs-harness appears among them.

FACT-11 (a text search over EXPERIMENT-DESIGNER-DEF and GATE-STAGE-DIRECTORY, its own method and result). A full text search for the exact phrase this project uses for in repo harness finds zero matches anywhere inside EXPERIMENT-DESIGNER-DEF, and zero matches inside every script file inside GATE-STAGE-DIRECTORY.

FACT-12 (EXPERIMENT-DESIGNER-DEF's own fixed list of section names for the planning document, translated in full, in order). 1 Problem. 2 Tested clause and the definitions it cites. 3 What the implementation looks like today. 4 Numbers that already existed before the run. 5 Arms, positive control, real baseline. 6 Which quantities to report and each one's judging rule. 7 Assertions that pin an absolute value. 8 Trajectory and geometry sensitivity. 9 Mutations. 10 Failure clauses. 11 Void clauses and halt clauses. 12 Revisions. 13 Files read and commands run. None of these thirteen section names is a field for declaring that the harness is fixed as the in repo harness.

FACT-13 (BODY-DOC's own statement of the trigger condition and who decides, translated in full). The rule document states the trigger condition as: the planning document fixes the harness as the in repo harness. The choice of when this happens rests with whoever writes that planning document, a role this task calls the designer, the same role named in EXPERIMENT-DESIGNER-DEF.

FACT-14 (BODY-DOC's own description of the risk behind question W1, translated in full). The in repo harness uses the real types and constants from the core crate and the shared format crate. A constant that is written wrong would enter both the real implementation and the harness at the same time, and the two sides would then report the same wrong number, cell for cell, in a way that looks identical on both sides.

FACT-15 (BODY-DOC's own description of the risk behind question W2, translated in full). Before this branch, the runner's write scope held no path under crates. After the pattern crates/singlefs-harness/src/bin/e*.rs is granted, the runner writes inside the same tree as the role that writes the real implementation. The question being judged is whether this widening lets the runner touch something it should not touch, for example changing an implementation constant that it finds awkward to read while writing its own file, and whether WRITE-SCOPE-TABLE's pattern for this row is written no wider than it needs to be.

ROW DEFINITIONS

ROW W1 is: does the new branch's shared-code risk, described in FACT-14, get caught by any of today's scheduled checks, and can each of the three restrictions in FACT-2 be judged by a machine.

ROW W2 is: does the write-scope widening's risk, described in FACT-15, get caught by any of today's scheduled checks or by WRITE-GUARD-HOOK, and can each of the three restrictions in FACT-2 be judged by a machine for this row's own risk.

ROW W3 is: does anything today check whether the trigger condition in FACT-1 and FACT-13 was applied in a clear way when the designer wrote the planning document, and can each of the three restrictions in FACT-2 be judged by a machine for this row's own risk.

TABLE TO FILL

Fill one row for W1, one row for W2, one row for W3, in that order. The table has four columns. Write the table using the vertical bar character, one row per line, and add a short reasoning sentence inside each cell, not a bare word.

Column RED-CHECK: does a check exist today, among GATE-63-SCRIPT, GATE-56-SCRIPT, GATE-59-SCRIPT, GATE-96-SCRIPT, GATE-94-SCRIPT, or WRITE-GUARD-HOOK, or any other artifact named above, that would turn red or refuse a write because of this row's own risk as stated in its ROW DEFINITION above. Write yes or no, name the artifact if yes or write NONE if no, and cite the FACT label that supports the answer.

Column REST-CHECK: for each of the three restrictions in FACT-2, taken in order as REST-1 (read-only, drive-only, observe-only), REST-2 (must not change core or checker), REST-3 (mutation rows go into crates/mutations.tsv and GATE-59-SCRIPT runs them), write one line inside this cell starting with REST-1, REST-2, REST-3 in turn. For each one, write yes or no for whether a machine can judge it today, name the checking artifact and describe in one sentence how it would judge that restriction, or say NONE if no such artifact exists, and cite a FACT label for each of the three lines.

Column FIRST-FINDER: for this row's own risk, name who or what would notice the error first, today, before anyone reads a report by eye. Answer with a role name, an artifact label, or the word NOBODY if nothing today would notice it before a human happens to read something by eye. Cite the FACT label that supports the answer.

Column SHARED-FILE: for each of the other two rows, state yes or no for whether fixing this row's own gap, if one exists, would require changing the same artifact as fixing that other row's gap would; if yes, name the artifact using one of the ARTIFACT labels above, tying it to the FACT label whose source names that artifact. Give two sentences, one comparing to each of the other two rows.

After the table, write three lines, each on its own, labeled W1 overturn condition, W2 overturn condition, W3 overturn condition, each stating one sentence on what observation about the repository would change that row's RED-CHECK or REST-CHECK answer from yes to no or from no to yes.

After the three overturn lines, write one line labeled UNKNOWN COUNT stating the total number of cells across the whole table, counting each of the three REST lines inside a REST-CHECK cell as one cell of its own, that you filled with UNKNOWN.
