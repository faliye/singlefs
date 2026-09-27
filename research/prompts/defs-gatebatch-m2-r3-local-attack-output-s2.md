1.  
Row A: count-line=LAYER0; yes; layer0_enumerates_every_crash_state_of_the_settled_stream_with_zero_violations; N/A  
Row B: count-line=CHECKER; yes; checker_line; N/A  
Row C: exhaustive=LAYER0; yes; yes  
Row D: threads=LAYER0; yes; yes  
Row E: shard=across-machines; yes; no; N/A  
What observation would overturn: If the test function layer0_enumerates_every_crash_state_of_the_settled_stream_with_zero_violations does not print a line starting with "LAYER0 ".

2.  
Row A: count-line=LAYER0B; yes; full_enumeration_of_the_fixed_script_stream_is_exhaustive_and_clean; N/A  
Row B: exhaustive=LAYER0B; yes; yes  
Row C: threads=LAYER0B; yes; yes  
Row D: shard=across-machines; yes; no; N/A  
Row E: count-line=CHECKER registered? no; produces CHECKER line? no  
What observation would overturn: If the test function full_enumeration_of_the_fixed_script_stream_is_exhaustive_and_clean prints a line starting with "CHECKER ".

3.  
Row A: count-line=LAYER0_PARALLEL_FINISHED; yes; enumerate_layer0_in_state_slices; N/A  
Row B: threads=LAYER0_PARALLEL_FINISHED; yes; yes  
Row C: exhaustive=LAYER0_PARALLEL_FINISHED registered? no; contains "exhaustive"? no  
Row D: shard=across-machines registered? no; argument passed? Layer0Resume::NoProgressFile  
What observation would overturn: If the test function crash_states_inside_the_floor_raise_pushed_by_the_session_recover_and_remount_with_the_checker_green passes Layer0Resume::from_environment(...) to enumerate_layer0_in_state_slices.

4.  
Row A: count-line=C561_SIGMA_FULL; yes; count_line helper; N/A  
Row B: exhaustive=C561_SIGMA_FULL; yes; yes  
Row C: threads=C561_SIGMA_FULL; yes; yes  
Row D: shard=across-machines registered? no; mentions from_environment? not mentioned in the evidence given  
What observation would overturn: If the evidence shows the test function every_crash_state_of_sigma_leaves_every_unit_of_the_claimed_publishes_present calls Layer0Resume::from_environment(...).

5.  
Row A: count-line=LAYER0_PARALLEL_LINE_ONE; yes; full_enumeration_of_the_parallel_line_one_stream_is_exhaustive_and_clean; N/A  
Row B: exhaustive=LAYER0_PARALLEL_LINE_ONE; yes; yes  
Row C: threads=LAYER0_PARALLEL_LINE_ONE; yes; yes  
Row D: shard=across-machines; yes; no; N/A  
What observation would overturn: If the test function full_enumeration_of_the_parallel_line_one_stream_is_exhaustive_and_clean does not print a line starting with "LAYER0_PARALLEL_LINE_ONE ".

6.  
Row 1: prefix LAYER0_TREE_SPLIT_CENTRAL_MAPPING_ROOT_SPLIT; count-line yes; exhaustive yes; threads yes  
Row 2: prefix LAYER0_TREE_SPLIT_CENTRAL_MAPPING_LEAF_SPLIT; count-line yes; exhaustive yes; threads yes  
Row 3: prefix LAYER0_TREE_SPLIT_CENTRAL_MAPPING_TWO_LEVELS_SPLIT_IN_A_ROW; count-line yes; exhaustive yes; threads yes  
Row 4: prefix LAYER0_TREE_SPLIT_CENTRAL_MAPPING_EMPTY_LEAF_DROPPED; count-line yes; exhaustive yes; threads yes  
Row 5: prefix LAYER0_TREE_SPLIT_CENTRAL_MAPPING_ROOT_LOWERED; count-line yes; exhaustive yes; threads yes  
Row 6: prefix LAYER0_TREE_SPLIT_ACCOUNTING_ROOT_SPLIT; count-line yes; exhaustive yes; threads yes  
Row 7: prefix LAYER0_TREE_SPLIT_ACCOUNTING_LEAF_SPLIT; count-line yes; exhaustive yes; threads yes  
The absence of shard=across-machines is consistent because the test always uses Layer0Resume::NoProgressFile, which does not read the sharding environment variable. What observation would overturn: If the test function full_enumeration_of_every_tree_split_stream_is_exhaustive_and_clean calls Layer0Resume::from_environment(...) instead of Layer0Resume::NoProgressFile.

7.  
Row A: count-line=CRASH_INJECTION_FINISHED; yes; crash_injection.rs helper; N/A  
Row B: exhaustive=CRASH_INJECTION_FINISHED registered? no; contains "exhaustive"? no  
Row C: threads=CRASH_INJECTION_FINISHED registered? no; contains worker_threads= field? yes; would admission.py find it? no, because admission.py's threads= judging searches for "LAYER0_PARALLEL_FINISHED " not "CRASH_INJECTION_FINISHED "  
What observation would overturn: If admission.py's threads= judging logic was modified to search for "CRASH_INJECTION_FINISHED " lines instead of "LAYER0_PARALLEL_FINISHED ".

8.  
crash-case:crash-injection-fast-tier; produces LAYER0_PARALLEL_FINISHED or worker-thread line? prints "CRASH_INJECTION_FINISHED" with worker_threads= but admission.py's threads= judging does not use this line for threads= checks  
What observation would overturn: If admission.py's threads= judging logic was modified to recognize "CRASH_INJECTION_FINISHED " lines as valid for threads= conditions.

9.  
The quick tier does not evaluate any of the four field kinds because all seven test functions are marked #[ignore] and the quick-tier gate script only parses cargo's test result summary without invoking admission.py or checking any conditions. Section 4 explicitly states the quick tier skips #[ignore] tests and does not call admission.py functions. What observation would overturn: If the quick-tier gate script invoked admission.py's judge_crash_case_log or similar functions when running #[ignore] tests.
