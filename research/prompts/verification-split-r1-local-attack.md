Round: verification-split-r1, local attack leg. This leg's attack surface has two independent parts, given as Part One and Part Two below. Answer both parts. Do not answer yes or no by itself anywhere; every item asks for a letter, a table cell, or a stated reason. Write your whole answer in English. Do not write any file-and-line-number citation and do not write any code line number anywhere in your answer; when you need to point at something, name it by its own label given below (Fact 1, Fact 2, ..., Rule 8, Row 1, Row 2, ...) instead of a line number. For every numbered item, end it with one separate sentence starting with the words "This would be wrong if" that states what observation would show your answer for that item is wrong.

===================================================================
PART ONE. Has an upstream shared specification finished separating itself from one particular downstream project.

Background. A shared specification repository ("the SOP") is meant to be read by any project that adopts it. It exists in three language editions (Chinese, English, Japanese) that are supposed to say the same thing in three languages. One particular downstream project that uses this SOP builds a filesystem, and is itself named "singlefs". The SOP repository is separately supposed to hold no wording that is specific to that one downstream project or to filesystem design in general -- such wording belongs in the downstream project's own local files, not in the shared SOP.

Rule 8, translated from the SOP's own rule file (rules/rules-discipline.md, its own section 8, titled "the body text must not name any one particular consumer's own things"):

"The specification is meant to be read by any consumer. Its body text must not contain the name of the project that uses it, that project's paths, or that project's file names, and must not use that project's decision numbers or experiment numbers as examples. Write examples in an unnamed, generic form: a hookup-path shape like '.claude/gate.d/' is a shared convention and may be written; a concrete file name like '.claude/gate.d/55-xxx.sh' may not be written. The consumer list is registered exactly once, under the key 'consumers=' in a file named I18N, and is not copied anywhere else. A script named scripts/rules-lint.sh judges the name half of this rule automatically; the other half, using a decision number or an experiment number as an example, cannot be told apart mechanically, and relies on human review."

For each of the 14 facts below, decide which of three labels it gets:
Label A: this line is tied to this one downstream project's own verification method or its own name, and should not be sitting in a shared, any-consumer specification.
Label B: this line states a generic testing concept, or gives a generic, unnamed example, of the kind any consumer's specification writer would be free to use, and it is fine for it to stay.
Label C: this line is about how a consuming project hooks itself up (the kind of thing Rule 8 calls "a consumer's own thing"), and whether it should stay depends on Rule 8 above.
For each fact, write only the label and one sentence of reasoning tied to Rule 8's own wording (does the line name the project, a path, a file name, a decision number, or an experiment number, or does it not).

Fact 1. Source: the SOP's own English edition, rules/verify-before-claiming.md, a row of a table titled "what to check right now instead of assuming". Quoted verbatim: "how another filesystem does something | check its documentation or source now, and note in the kb both the source and \"not verified in this project\"". The Chinese and Japanese editions say the same thing in their own languages; all three editions agree.

Fact 2. Source: the SOP's own English edition, rules/session-wrapup.md, one bullet of a two-bullet list about where to make a change. Quoted verbatim: "How the filesystem is designed -> change the project's own kb/, .claude/rules/, or the project's CLAUDE.md". All three language editions agree.

Fact 3. Source: the SOP's own English edition, rules/session-wrapup.md, the other bullet of the same two-bullet list. Quoted verbatim: "Collaboration (evidence, documents, where decisions land, gate feedback) -> change singlefs-ai-sop and bump VERSION (which paths require the bump is defined by GOVERNED in scripts/version-discipline.sh)". All three language editions agree. Note: "singlefs-ai-sop" here is the SOP repository's own name, not the downstream filesystem project's name; the downstream project is named "singlefs" (without the "-ai-sop" suffix).

Fact 4. Source: the SOP's own English edition, rules/code-discipline.md, one line of a glossary code block. Quoted verbatim: "lba  # logical block address: the number the filesystem gives a block". All three language editions agree.

Fact 5. Source: the SOP's own English edition, templates/CLAUDE.project.md, a template paragraph meant to be copied into a new consuming project's own CLAUDE.md. Quoted verbatim: "Rules specific to filesystem design (transactions, crash consistency, on-disk format, that family) go in .claude/rules/ and are @-referenced here as well. They are not upstreamed -- the shared SOP holds collaboration rules only." All three language editions agree.

Fact 6. Source: the SOP's own English edition, CLAUDE.md, its own opening definition sentence. Quoted verbatim: "It governs how a project collaborates with AI, not how a filesystem should be designed. It has exactly one user, named in I18N under consumers=." All three language editions agree.

Fact 7. Source: the SOP's own English edition, CLAUDE.md, one bullet of a list contrasting what the SOP governs against what it does not. Quoted verbatim: "How the filesystem is designed -- transactions, crash consistency, on-disk format, the disciplines specific to this kind of system. -> the project. Design decisions into kb/decisions.md, invariants into kb/invariants.md, project-specific rules into .claude/rules/". All three language editions agree.

Fact 8. Source: a configuration file named I18N, identical text in all three language editions, translated from Chinese. Quoted: "consumers: the projects that use this SOP. The specification's body text must not write out their specific content; this is judged by scripts/rules-lint.sh." followed on the next line by the literal key-value pair: consumers=singlefs

Fact 9. Source: the Chinese edition only, README.md, the closing sentence of its License section, translated from Chinese: "Dual-licensed: Apache-2.0 or MIT, take your pick, matching singlefs (https://github.com/faliye/singlefs)." Both the English edition's License section and the Japanese edition's License section say only "Dual-licensed: Apache-2.0 or MIT, at your option" and stop there -- neither one has any sentence naming "singlefs" or linking to that URL.

Fact 10. Source: a Python script named scripts/relay-timing-lint.py, identical text shared by all three language editions (this file is code, not translated prose), translated from its own Chinese comment: "Why this check exists: E152 (comparing six filesystems' file performance, milestone by milestone) has a guest-side program that, inside a loop reading lines from a child process's output, first timestamps a line and then relays it out, using the gap between line-arrival timestamps as a segment's wall-clock time. Inside the virtual machine, standard output is a serial port; one relay call blocks for a few milliseconds, while the child process writing to its pipe is not blocked, so the timestamps end up mixed with a backlog of printing from earlier lines: several lines printed back to back, with only a few microseconds of computation between them, ended up with timestamps spanning 21.7 to 25.8 milliseconds; in 9 out of 10 rounds across two runs, the outer layer's computed wall-clock time for a second transaction came out smaller than the time the child process itself had measured, which is physically impossible (measured on 2026-09-17)."

Fact 11. Source: the same script as Fact 10, identical across all three editions, a short self-test case name, translated from Chinese: "Old way of writing it: read one line, timestamp it, relay it out, then read the next line (modeled on E152's run_singlefs)."

Fact 12. Source: a shell script named scripts/doc-lint.sh, identical across all three editions, two adjacent pieces translated from Chinese. First, a comment explaining a design choice: "In a filesystem project, the word meaning 'that table' nearly always refers to an in-memory table (as in 'the slot table has 4096 entries; that table stays resident in memory'); genuine self-reference to the current document instead uses the word meaning 'this table' -- so the word 'table' is only paired with 'this', not with 'that' (found by measuring false positives)." Second, the line of code this comment justifies, followed by its own comment: a variable holding a list of words excluded from counting as "this document" self-reference because they instead name the system under test; the excluded list is: project, repository, round, machine, filesystem, system, disk, implementation, package, occurrence; its own comment reads: "Anything referring to the system under test, rather than a location within the document, is excluded across the board."

Fact 13. Source: a shell script named scripts/naming-lint.sh, identical across all three editions, one row of an abbreviation-expansion table, translated from Chinese: "ext: extent / extension / external (all three of these expansions are common in filesystem projects)."

Fact 14. Source: the same script as Fact 12, its Japanese-specific branch, identical across all three editions (this branch fires only when the document being linted is itself written in Japanese), translated from Chinese: the Japanese-language excluded list mirrors Fact 12's list but is shorter: project, repository, machine, filesystem, system, package -- it does not separately list "disk", "round", "implementation", or "occurrence" the way Fact 12's list does.

Answer Part One as items 1 through 14, one label and one reasoning sentence each, each ending with its own "This would be wrong if" sentence.

===================================================================
PART TWO. Predict a classification from rule text alone, for later comparison against ground truth you are not shown.

Background. A project splits its test commands into six categories: "harness" (may run any time), "checker tier" (by default only at commit time), "full test suite", "layer 0", "crates mutation table", and "not heavy" (none of the other five; always allowed). Below are the only two pieces of rule text that are supposed to define how a command is sorted into one of these six categories. Read only this rule text; do not assume any fact about the project beyond what these two blocks state.

RULE BLOCK ONE. Source: a project rule file named .claude/rules/verification.md, its own "Definitions" section, translated from Chinese:

"harness: crates/singlefs-harness. The layer of unit tests and integration tests, plus the scaffolding library they need (recorder, ideal model, crash-state enumeration engine, injector, scenarios). Runs any time the code changes. Test for membership: a test that finishes in anywhere from a second to one or two minutes, does not do full crash enumeration, and does not depend on a real device or an external tool, is a harness test.

checker: crates/singlefs-checker. The library is the pool-level checker (a one-argument predicate over a single image); its tests/ directory is the checker tier -- test cases that directly call the full crash-enumeration functions (the full-enumeration case is marked #[ignore]; a small-stream quick-tier case in the same file is not marked), plus the sampled tier of injection campaigns. Together with QEMU (gate stage 55), herd7 (stage 57), the full crates mutation table (stage 59), and full replay of all experiments (stage 87), this tier by default runs only at commit time. Test for membership: a test that enumerates crash states, needs a real device or an external tool, or takes on the order of ten minutes to run once, is checker-tier.

Classify a newly written test using these two definitions first, then decide which crate it goes in; when unsure, put it in the checker tier."

RULE BLOCK ONE, continued. Source: the same rule file, its own "the two tiers are split by when they run, not by mechanism" section, translated from Chinese, given as a table with columns tier / lives where / when it runs / who runs it:

Row for harness: lives in crates/singlefs-harness (unit tests and integration tests, together with scaffolding such as the recorder, the ideal model, and scenarios); runs any time the code changes; run, before handing back, by whoever implemented the change, on whichever test binaries they themselves touched (a command of the shape "cargo test -p singlefs-harness --test <target>", or the same with --lib instead of --test), plus fmt / clippy / build.

Row for checker tier: lives in tests inside the crates/singlefs-checker package (crash-enumeration cases, injection campaigns), together with QEMU on a real device (gate stage 55), herd7 (stage 57), the full crates mutation table (stage 59), and full replay of all experiments (stage 87); by default runs only at commit time; to run it on its own, the command must carry the environment variable SINGLEFS_HEAVY_TESTS=user-request; at commit time it is run only by two specific automated roles, each running their own part of it with the command carrying SINGLEFS_HEAVY_TESTS=commit; no other role ever runs it.

Closing sentence of this section: "checker tier" and "pool-level checker" are two different names: the former is this tier's set of tests, the latter is the one-argument predicate function living inside the singlefs-checker library that the checker tier's own tests call at any time.

RULE BLOCK TWO. Source: a shell script named .claude/hooks/heavy-test-guard.sh, its own POLICY message string, translated from Chinese:

"The rule: heavy testing is that whole checker tier (see .claude/rules/verification.md): tests in the singlefs-checker package, layer 0, crash-enumeration cases, QEMU, herd7, the full crates mutation table, full cargo test, a whole pass of the project's gate, full replay of all experiments, and one named benchmark rig -- run only once at commit time, or when the user asks for it; an automated subagent role never runs any of this, and only runs the harness tier -- test binaries in non-checker packages such as singlefs-harness that it itself touched (a command of the shape 'cargo test -p <crate> --test <its own target>', or the same with --lib), plus fmt / clippy / build; when the main coordinating role runs heavy testing as part of the commit flow it must carry SINGLEFS_HEAVY_TESTS=commit, and SINGLEFS_HEAVY_TESTS=user-request when the user asked for it outside of commit.

Each role's own share: one role (call it the crash-verifier role) only runs gate stages 54, 55, 57, 59 and the checker-package tests underneath them (the quick tier and the registered crash-enumeration cases), a QEMU system command, an lkmm/herd7 script, and the full crates mutation table; a second role (call it the gate-triage role) only runs a whole pass of the gate script and gate stage 87 (stages 54, 55, 57, 59 are instead reused from the last all-green verdict as long as their inputs have not changed); both roles must carry the same environment-variable prefix, and neither of them ever runs a full, unrestricted cargo test. Every other stage under the project's gate-stage directory is not heavy, and any role may run it.

Anything outside of commit time that truly needs heavy testing to run: the main coordinating role first asks the human user; only once the user agrees does it run with SINGLEFS_HEAVY_TESTS=user-request; an automated subagent role instead writes down, in its handback, what it needed to run and why, for the main role to go ask the user."

TASK for Part Two (item 15 of your answer): using only Rule Block One and Rule Block Two above, and nothing else, classify each of the 26 commands below into exactly one of the six categories: harness, checker tier, full test suite, layer 0, crates mutation table, not heavy. All 26 commands below are commands that would be typed while standing at the root of the project's own repository. Build one table with 26 rows, one per command number. For every row, name which specific piece of Rule Block One or Rule Block Two (by the block's own paragraph, e.g. "Rule Block One's harness test-for-membership sentence", or "Rule Block Two's first paragraph") led you to that category. If, for a given command, the two rule blocks point you toward two different categories, or a rule block does not clearly say either way, say so explicitly in that row instead of silently picking one; do not resolve the disagreement yourself using outside knowledge of testing tools, only using the wording actually given above. End the whole table with one "This would be wrong if" sentence covering the table as a whole.

1. cargo test -p singlefs-harness
2. cargo test -p singlefs-harness --test second_transaction_step_one_overwrite
3. cargo test -p singlefs-checker
4. cargo test --release -p singlefs-checker --test first_transaction_step_seven_layer0
5. cargo test -p singlefs-checker --lib
6. cargo test -p singlefs-checker --doc
7. cargo test -p singlefs-checker --tests
8. cargo test --package=singlefs-checker
9. cd crates/singlefs-checker && cargo test
10. cd crates/singlefs-harness && cargo test
11. cargo test
12. cargo test --workspace
13. cargo test -p singlefs-core -p singlefs-checker
14. cargo test -p singlefs-checker --test '*'
15. cargo nextest run -p singlefs-checker
16. cargo test --release -p singlefs-checker -- --include-ignored --exact full_enumeration_of_the_fixed_script_stream_is_exhaustive_and_clean
17. ./target/release/deps/first_transaction_step_seven_layer0-0123456789abcdef
18. ./target/debug/deps/second_transaction_step_one_overwrite-0123456789abcdef
19. bash .claude/gate.d/54-layer0-replay.sh
20. bash .claude/gate.d/74-model-differential.sh
21. cargo test -p singlefs-checker -- --list
22. cargo test -p singlefs-format
23. cargo clippy --all-targets
24. bash research/scripts/mutate.sh x y crates/mutations.tsv
25. nice -n 19 bash research/scripts/run-with-memory-cap.sh 16G cargo test --release -p singlefs-checker
26. bash .claude/gate.d/59-crates-mutation-replay.sh

This is the end of the prompt. Answer Part One (items 1 through 14) and then Part Two (item 15, the 26-row table), in that order.
