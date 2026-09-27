You are checking a gate registration table against the Rust source it points to, in a filesystem project (singlefs). This is a cross-check task, not a code-review task. Do not use markdown emphasis (no bold, no italics, no headers with asterisks) anywhere in your answer. Answer by numbered item, matching the numbering of the questions in section 5 below. For every answer, state what single observation would overturn it. Do not cite source-code line numbers or file line numbers anywhere in your answer; refer to function names, or to the row numbers of the tables below, instead. Fill every cell of every table; do not answer a table with only yes or no, write the specific value or specific name in each cell.

Section 1: what the four registration field kinds mean

This is a project registration file, `.claude/gate.d/stage-inputs.tsv`. Each line whose key starts with `crash-case:` registers one Rust integration test as an exhaustive crash-enumeration case. The third tab-separated column of such a line is a space-separated list of conditions. A gate script (its exact name does not matter here) reads this file and a Python module called admission.py, and uses these conditions to judge a log produced by running the registered test with `--include-ignored --exact` in release mode. The condition kinds are, verbatim from admission.py's own header comment:

count-line=<PREFIX>: the log must contain exactly one line starting with "<PREFIX> " (that literal prefix followed by a space). That whole line is recorded verbatim into a green marker.

exhaustive=<PREFIX>: the line found for count-line=<PREFIX> must contain the field exhaustive=true somewhere in it. A PREFIX can only be used with exhaustive= if it was also registered with count-line= for the same PREFIX.

threads=<PREFIX>: take the states=<N> field off the line found for count-line=<PREFIX>. Then search the whole log for a line starting with "LAYER0_PARALLEL_FINISHED " (that exact literal prefix) whose own states= field equals the same N. Judge worker-thread usage from that LAYER0_PARALLEL_FINISHED line's fields (worker_threads=, slices=, resumed_slices=, freshly_run_slices=). A PREFIX can only be used with threads= if it was also registered with count-line= for the same PREFIX.

shard=across-machines: at most one such condition per line. It means this test case's enumeration recognizes a two-machine sharding environment variable (SINGLEFS_LAYER0_SHARD) and can be split across two machines and merged.

A quick tier run (ordinary `cargo test`, no --full) never evaluates any of count-line=, exhaustive=, threads=, or shard=across-machines. It only runs the test binary, and because every one of these seven test functions is marked #[ignore] in the Rust source, the quick tier does not even execute them; it can only report how many other, non-ignored tests in the same file passed. All four field kinds above are judged only in the --full path (release build, --include-ignored --exact), by admission.py's function judge_crash_case_log (which calls judge_worker_threads for each threads= condition).

Section 2: the seven registration lines, verbatim

Below are the seven lines from `.claude/gate.d/stage-inputs.tsv`, today's lines 36 through 42, given to you verbatim including their tab-separated columns. Column 1 is the registration key. Column 2 is the list of input paths (not relevant to this task). Column 3 is the space-separated list of conditions you must check. Column 4 (after the "#") is a free-text comment written by a human; you may read it for context but the facts you must verify are in column 3 versus the source-code evidence in section 3, not in column 4.

Line 36 (crash-case:layer0-first-stream), column 3:
test=singlefs-harness:first_transaction_step_seven_layer0:layer0_enumerates_every_crash_state_of_the_settled_stream_with_zero_violations count-line=LAYER0 count-line=CHECKER exhaustive=LAYER0 threads=LAYER0 shard=across-machines

Line 37 (crash-case:layer0-second-stream), column 3:
test=singlefs-harness:second_transaction_step_zero_layer0:full_enumeration_of_the_fixed_script_stream_is_exhaustive_and_clean count-line=LAYER0B exhaustive=LAYER0B threads=LAYER0B shard=across-machines

Line 38 (crash-case:floor-raise-pushed-by-the-session), column 3:
test=singlefs-harness:second_transaction_crash_inside_the_floor_raise_pushed_by_the_session:crash_states_inside_the_floor_raise_pushed_by_the_session_recover_and_remount_with_the_checker_green count-line=LAYER0_PARALLEL_FINISHED threads=LAYER0_PARALLEL_FINISHED

Line 39 (crash-case:c561-sigma-full), column 3:
test=singlefs-harness:record_checker_judges_absence_by_the_persisted_set:every_crash_state_of_sigma_leaves_every_unit_of_the_claimed_publishes_present count-line=C561_SIGMA_FULL exhaustive=C561_SIGMA_FULL threads=C561_SIGMA_FULL

Line 40 (crash-case:layer0-parallel-line-one-stream), column 3:
test=singlefs-harness:second_transaction_parallel_line_one_layer0:full_enumeration_of_the_parallel_line_one_stream_is_exhaustive_and_clean count-line=LAYER0_PARALLEL_LINE_ONE exhaustive=LAYER0_PARALLEL_LINE_ONE threads=LAYER0_PARALLEL_LINE_ONE shard=across-machines

Line 41 (crash-case:layer0-tree-split-streams), column 3:
test=singlefs-harness:second_transaction_supplement_two_tree_split_layer0:full_enumeration_of_every_tree_split_stream_is_exhaustive_and_clean count-line=LAYER0_TREE_SPLIT_CENTRAL_MAPPING_ROOT_SPLIT count-line=LAYER0_TREE_SPLIT_CENTRAL_MAPPING_LEAF_SPLIT count-line=LAYER0_TREE_SPLIT_CENTRAL_MAPPING_TWO_LEVELS_SPLIT_IN_A_ROW count-line=LAYER0_TREE_SPLIT_CENTRAL_MAPPING_EMPTY_LEAF_DROPPED count-line=LAYER0_TREE_SPLIT_CENTRAL_MAPPING_ROOT_LOWERED count-line=LAYER0_TREE_SPLIT_ACCOUNTING_ROOT_SPLIT count-line=LAYER0_TREE_SPLIT_ACCOUNTING_LEAF_SPLIT exhaustive=LAYER0_TREE_SPLIT_CENTRAL_MAPPING_ROOT_SPLIT exhaustive=LAYER0_TREE_SPLIT_CENTRAL_MAPPING_LEAF_SPLIT exhaustive=LAYER0_TREE_SPLIT_CENTRAL_MAPPING_TWO_LEVELS_SPLIT_IN_A_ROW exhaustive=LAYER0_TREE_SPLIT_CENTRAL_MAPPING_EMPTY_LEAF_DROPPED exhaustive=LAYER0_TREE_SPLIT_CENTRAL_MAPPING_ROOT_LOWERED exhaustive=LAYER0_TREE_SPLIT_ACCOUNTING_ROOT_SPLIT exhaustive=LAYER0_TREE_SPLIT_ACCOUNTING_LEAF_SPLIT threads=LAYER0_TREE_SPLIT_CENTRAL_MAPPING_ROOT_SPLIT threads=LAYER0_TREE_SPLIT_CENTRAL_MAPPING_LEAF_SPLIT threads=LAYER0_TREE_SPLIT_CENTRAL_MAPPING_TWO_LEVELS_SPLIT_IN_A_ROW threads=LAYER0_TREE_SPLIT_CENTRAL_MAPPING_EMPTY_LEAF_DROPPED threads=LAYER0_TREE_SPLIT_CENTRAL_MAPPING_ROOT_LOWERED threads=LAYER0_TREE_SPLIT_ACCOUNTING_ROOT_SPLIT threads=LAYER0_TREE_SPLIT_ACCOUNTING_LEAF_SPLIT

Line 42 (crash-case:crash-injection-fast-tier), column 3:
test=singlefs-harness:second_transaction_supplement_three_crash_injection:crash_injection_fast_tier_recovers_only_into_versions_the_model_committed count-line=CRASH_INJECTION_FINISHED

Section 3: verbatim evidence from the Rust source, one block per registration line

All of the following are exact excerpts (format strings passed to Rust's println! macro, or exact match-arm string literals) taken directly from the source files by grep. Function names are given so you can refer to them instead of line numbers.

Evidence for line 36 (crash-case:layer0-first-stream). File: crates/singlefs-harness/tests/first_transaction_step_seven_layer0.rs. The test function is named layer0_enumerates_every_crash_state_of_the_settled_stream_with_zero_violations. Inside it, exactly one println! prints this format string:
"LAYER0 states={} closed_form={closed_form} violations={} root_persisted_states={} no_file={} file_read={} failed={} verification_ran={} verification_failed={} journal_differing={} exhaustive={} states_by_publish=[{}]"
Also inside the same test function, exactly one println! prints the result of a helper function named checker_line, which formats as:
"CHECKER record_root_without_record={} record_claimed_state_missing_unit={} {per_invariant}"
The same test function calls (through a local wrapper function named enumerate_counting_allocation_generation_read_sets) the shared library function enumerate_layer0_in_state_slices_or_one_shard, passing it a Layer0Resume::from_environment(...) argument (this is the argument shape that reads the SINGLEFS_LAYER0_SHARD environment variable). That shared library function internally calls another shared function named enumerate_layer0_in_state_slices, which (regardless of caller) always separately prints, once at the start and once at the end, two more lines with these exact format strings:
"LAYER0_PARALLEL_START states={states_of_this_run} slices={} states_per_slice={} worker_threads={spawned_worker_threads} configured_worker_threads={} worker_threads_source={} resumed_slices={restored_slice_count} freshly_run_slices={}{shard_fields}"
"LAYER0_PARALLEL_FINISHED states={states_of_this_run} slices={} worker_threads={spawned_worker_threads} configured_worker_threads={} worker_threads_source={} resumed_slices={restored_slice_count} freshly_run_slices={merged_fresh_slice_count} progress_file_after_completion={} elapsed_seconds={:.1}{shard_fields}"
Note that this LAYER0_PARALLEL_FINISHED format string does not contain the substring "exhaustive" anywhere in it.

Evidence for line 37 (crash-case:layer0-second-stream). File: crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs. The test function is named full_enumeration_of_the_fixed_script_stream_is_exhaustive_and_clean. Inside it, exactly one println! prints this format string:
"LAYER0B states={} closed_form={closed_form} exhaustive={} violations={} root_persisted_states={} no_file={} file_read={} failed={} journal_differing={} verification_ran={} verification_failed={} record_root_without_record={} record_claimed_state_missing_unit={} checker_violations={checker_violations} {per_invariant} states_by_publish=[{}] first_violation={}"
This test function is defined in the same source file as a helper function named checker_line, but that helper function is only defined and only called inside first_transaction_step_seven_layer0.rs (a different file, evidence for line 36 above); grepping the whole second_transaction_step_zero_layer0.rs file for the literal text "CHECKER" or "checker_line(" finds zero matches. This test function calls enumerate_layer0_in_state_slices_or_one_shard the same way as the line-36 test does (Layer0Resume::from_environment(...)), so it also triggers the same LAYER0_PARALLEL_START / LAYER0_PARALLEL_FINISHED lines described above for line 36.

Evidence for line 38 (crash-case:floor-raise-pushed-by-the-session). File: crates/singlefs-harness/tests/second_transaction_crash_inside_the_floor_raise_pushed_by_the_session.rs. The test function is named crash_states_inside_the_floor_raise_pushed_by_the_session_recover_and_remount_with_the_checker_green. Grepping this entire file for the literal text "println!" finds zero matches: this file contains no println! macro calls of its own. Instead, this test function directly imports and calls the shared library function enumerate_layer0_in_state_slices (not the "_or_one_shard" wrapper), passing it the argument &Layer0Resume::NoProgressFile (this is a different argument shape from Layer0Resume::from_environment(...); NoProgressFile does not read the SINGLEFS_LAYER0_SHARD environment variable). Because it calls enumerate_layer0_in_state_slices directly, it triggers exactly the same LAYER0_PARALLEL_START / LAYER0_PARALLEL_FINISHED format strings quoted above for line 36, and nothing else.

Evidence for line 39 (crash-case:c561-sigma-full). File: crates/singlefs-harness/tests/record_checker_judges_absence_by_the_persisted_set.rs. The test function is named every_crash_state_of_sigma_leaves_every_unit_of_the_claimed_publishes_present. Inside it, two println! calls appear one after the other: first the result of a helper function named parallel_finished_line, then the result of a helper function named count_line called with the literal string argument "C561_SIGMA_FULL". The count_line helper formats as:
"{prefix} states={} closed_form_states={} exhaustive={} record_claimed_state_missing_unit={}"
so with prefix="C561_SIGMA_FULL" this produces a line starting with "C561_SIGMA_FULL states=" and it does contain an exhaustive= field. The parallel_finished_line helper (defined in this same test file, not in the shared crash.rs library) formats as:
"LAYER0_PARALLEL_FINISHED states={} slices={} worker_threads={} configured_worker_threads={} worker_threads_source={worker_threads_source} resumed_slices=0 freshly_run_slices={} progress_file_after_completion=none elapsed_seconds={:.1}"
This is a hand-written format string in the test file itself, not a call into the shared enumerate_layer0_in_state_slices function; it always sets resumed_slices to the literal 0. The doc comment directly above parallel_finished_line in the source says, in its own words, that this line is deliberately shaped the same as the one crash.rs's enumerate_layer0_in_state_slices prints, specifically so that admission.py's threads= judging (which greps for a line starting with "LAYER0_PARALLEL_FINISHED ") finds it.

Evidence for line 40 (crash-case:layer0-parallel-line-one-stream). File: crates/singlefs-harness/tests/second_transaction_parallel_line_one_layer0.rs. The test function is named full_enumeration_of_the_parallel_line_one_stream_is_exhaustive_and_clean. Inside it, exactly one println! prints this format string:
"LAYER0_PARALLEL_LINE_ONE states={} closed_form={closed_form} exhaustive={} violations={} states_by_publish=[{}] checker {} first_violation={}"
This test function calls enumerate_layer0_in_state_slices_or_one_shard with a Layer0Resume::from_environment(...) argument, the same shard-aware shape as line 36 and line 37, so it also triggers the LAYER0_PARALLEL_START / LAYER0_PARALLEL_FINISHED lines described above.

Evidence for line 41 (crash-case:layer0-tree-split-streams). File: crates/singlefs-harness/tests/second_transaction_supplement_two_tree_split_layer0.rs. The test function is named full_enumeration_of_every_tree_split_stream_is_exhaustive_and_clean. It loops over seven values of an enum called TreeSplitStream, and for each one calls a local helper function named enumerate, then prints one println! per loop iteration (seven times total) with this format string:
"{prefix} states={} closed_form={closed_form} exhaustive={} violations={} ignored_violations={} failed={} root_persisted_states={} journal_differing_states={} verification_ran_states={} record_root_without_record={} record_claimed_state_missing_unit={} states_by_publish=[{}] checker_by_invariant(evaluated/violated/not_applicable) {per_invariant}"
where prefix comes from calling a method named full_enumeration_count_line_prefix on the enum value. Grepping the enum's match arms for that method's literal string outputs, in TreeSplitStream enum declaration order, gives exactly these seven strings, in this order:
1: LAYER0_TREE_SPLIT_CENTRAL_MAPPING_ROOT_SPLIT
2: LAYER0_TREE_SPLIT_CENTRAL_MAPPING_LEAF_SPLIT
3: LAYER0_TREE_SPLIT_CENTRAL_MAPPING_TWO_LEVELS_SPLIT_IN_A_ROW
4: LAYER0_TREE_SPLIT_CENTRAL_MAPPING_EMPTY_LEAF_DROPPED
5: LAYER0_TREE_SPLIT_CENTRAL_MAPPING_ROOT_LOWERED
6: LAYER0_TREE_SPLIT_ACCOUNTING_ROOT_SPLIT
7: LAYER0_TREE_SPLIT_ACCOUNTING_LEAF_SPLIT
The local helper function enumerate (used by every iteration of this loop) calls the shared library function enumerate_layer0_selecting_versions, which itself, by definition, always calls enumerate_layer0_in_state_slices with the fixed argument &Layer0Resume::NoProgressFile (this is hardcoded inside enumerate_layer0_selecting_versions itself, not chosen by the test). Because NoProgressFile never reads SINGLEFS_LAYER0_SHARD, none of these seven streams can be sharded across two machines through this call path. Each of the seven loop iterations therefore also triggers its own LAYER0_PARALLEL_START / LAYER0_PARALLEL_FINISHED pair (described under line 36 above) inside the same test process, one pair per iteration, in the same loop order 1 through 7 given above.

Evidence for line 42 (crash-case:crash-injection-fast-tier). File: crates/singlefs-harness/tests/second_transaction_supplement_three_crash_injection.rs. The test function is named crash_injection_fast_tier_recovers_only_into_versions_the_model_committed. Grepping this entire file for the literal text "println!" or "CRASH_INJECTION_FINISHED" finds zero matches: this file contains no println! macro calls of its own and no occurrence of that literal string. This test function calls a campaign-running helper that lives in a different source file, crates/singlefs-harness/src/crash_injection.rs; that helper (not the test file itself) contains exactly one println! with this format string:
"CRASH_INJECTION_FINISHED seeds=[{first_seed},{}) slices={} worker_threads={spawned_worker_threads} elapsed_seconds={:.1}"
Note three things about this exact format string: it does not contain the substring "exhaustive" anywhere in it; it does contain a worker_threads= field of its own; and its line-start literal text is "CRASH_INJECTION_FINISHED ", which is a different literal than "LAYER0_PARALLEL_FINISHED ".

Section 4: quick tier versus --full, one more verbatim fact

All seven test functions named above are marked with the Rust attribute #[ignore] in their source files. An ordinary `cargo test` run (the quick tier) skips every #[ignore]-marked test by default. The quick-tier gate script only ever parses lines of the literal form "test result: ok. N passed; 0 failed; M ignored" out of cargo's own output, to report how many tests passed and how many were ignored; it does not call admission.py's judge_crash_case_log, judge_worker_threads, or judge_threads_of_each_shard functions at all, and it does not look for count-line=, exhaustive=, threads=, or shard=across-machines anywhere. Those four functions, and those four field kinds, are read and evaluated only by the --full path.

Section 5: questions, with fact tables to fill

For each table, fill every cell with the specific fact requested (a field name, a function name, "yes" or "no" plus which one, or "none"), not a bare yes or no where a name is asked for. Do not invent facts beyond what section 2, 3 and 4 gave you; if a cell cannot be determined from the evidence given to you, write "not determinable from the evidence given" in that cell and say so in your prose answer too.

Question 1. Table for crash-case:layer0-first-stream (registration line 36). Columns: field kind and prefix; is it registered in line 36 (yes/no); does the section-3 evidence for line 36 show the source producing a line with exactly that prefix (yes/no, name the producing function); if it is exhaustive= or threads=, does the producing line's format string actually contain the field needed (exhaustive= field, or the LAYER0_PARALLEL_FINISHED line's states=/worker_threads=/slices=/resumed_slices=/freshly_run_slices= fields) (yes/no).
Row A: count-line=LAYER0
Row B: count-line=CHECKER
Row C: exhaustive=LAYER0
Row D: threads=LAYER0
Row E: shard=across-machines

Question 2. Table for crash-case:layer0-second-stream (registration line 37). Same four columns as question 1.
Row A: count-line=LAYER0B
Row B: exhaustive=LAYER0B
Row C: threads=LAYER0B
Row D: shard=across-machines
Row E: a fifth row you must add yourself: is there a count-line=CHECKER condition registered on line 37 (yes/no), and does the section-3 evidence for line 37 show its test function producing any line starting with "CHECKER " (yes/no, cite which function or "none found")

Question 3. Table for crash-case:floor-raise-pushed-by-the-session (registration line 38). Same four columns as question 1, for these two registered rows, plus two rows asking about conditions that are absent from line 38.
Row A: count-line=LAYER0_PARALLEL_FINISHED
Row B: threads=LAYER0_PARALLEL_FINISHED
Row C: is exhaustive=LAYER0_PARALLEL_FINISHED registered on line 38 (yes/no); does the LAYER0_PARALLEL_FINISHED format string shown in section 3 for line 36 (which line 38 also triggers) contain the substring "exhaustive" (yes/no)
Row D: is shard=across-machines registered on line 38 (yes/no); does the section-3 evidence for line 38 show its test function passing a Layer0Resume::from_environment(...) argument or a Layer0Resume::NoProgressFile argument (name which one) to enumerate_layer0_in_state_slices

Question 4. Table for crash-case:c561-sigma-full (registration line 39). Same four columns as question 1.
Row A: count-line=C561_SIGMA_FULL
Row B: exhaustive=C561_SIGMA_FULL
Row C: threads=C561_SIGMA_FULL
Row D: a fourth row you must add yourself: is shard=across-machines registered on line 39 (yes/no); does the section-3 evidence for line 39 mention any call to a shard-reading Layer0Resume::from_environment(...) argument at all (yes/no, or "not mentioned in the evidence given")

Question 5. Table for crash-case:layer0-parallel-line-one-stream (registration line 40). Same four columns as question 1.
Row A: count-line=LAYER0_PARALLEL_LINE_ONE
Row B: exhaustive=LAYER0_PARALLEL_LINE_ONE
Row C: threads=LAYER0_PARALLEL_LINE_ONE
Row D: shard=across-machines

Question 6. Table for crash-case:layer0-tree-split-streams (registration line 41). This line registers seven count-line= prefixes, the same seven again as exhaustive=, and the same seven again as threads=, and it registers no shard=across-machines at all. Columns: row number (1 through 7, matching the loop order given in section 3); the exact prefix string (copy it from the section-3 numbered list); is this exact string present as one of the seven count-line= values on registration line 41 (yes/no); is this exact string also present as one of the seven exhaustive= values on line 41 (yes/no); is this exact string also present as one of the seven threads= values on line 41 (yes/no).
Row 1 through row 7: one row per prefix in the loop order given in section 3.
After the table, answer in prose: since line 41 registers no shard=across-machines, and section 3 states the shared helper enumerate_layer0_selecting_versions always passes Layer0Resume::NoProgressFile, is the absence of shard=across-machines on line 41 consistent with what the evidence shows about this call path, or not; say what observation would overturn your answer.

Question 7. Table for crash-case:crash-injection-fast-tier (registration line 42). Same four columns as question 1, for the one registered row, plus two rows asking about conditions that are absent.
Row A: count-line=CRASH_INJECTION_FINISHED
Row B: is exhaustive=CRASH_INJECTION_FINISHED registered on line 42 (yes/no); does the CRASH_INJECTION_FINISHED format string shown in section 3 contain the substring "exhaustive" (yes/no)
Row C: is threads=CRASH_INJECTION_FINISHED registered on line 42 (yes/no); does the CRASH_INJECTION_FINISHED format string shown in section 3 contain a worker_threads= field of its own (yes/no); would admission.py's threads= judging (which, per section 1, searches for a line starting with the literal "LAYER0_PARALLEL_FINISHED ", not "CRASH_INJECTION_FINISHED ") find that field even if threads=CRASH_INJECTION_FINISHED were registered (yes/no, and say why in one sentence)

Question 8. Across all seven registration lines (36 through 42), list every one that has no threads= condition registered at all. For each one you list, name it by its crash-case key (for example crash-case:layer0-first-stream), and state, from the evidence given to you, whether that test's producing function ever prints any line starting with "LAYER0_PARALLEL_FINISHED " or with any other worker-thread-count field at all, or not.

Question 9. State, in one paragraph, whether the quick tier (section 4) evaluates any of the four field kinds (count-line=, exhaustive=, threads=, shard=across-machines) for any of these seven test cases, and why or why not, citing which fact in section 4 supports your answer. Then state what observation would overturn your answer.
