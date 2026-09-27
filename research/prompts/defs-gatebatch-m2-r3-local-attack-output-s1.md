1  
Row A: yes, yes, layer0_enumerates_every_crash_state_of_the_settled_stream_with_zero_violations, not applicable  
Row B: yes, yes, checker_line, not applicable  
Row C: yes, yes, layer0_enumerates_every_crash_state_of_the_settled_stream_with_zero_violations, yes  
Row D: yes, no, enumerate_layer0_in_state_slices, yes  
Row E: yes, no, not applicable, not applicable  
Observation that would overturn: If the LAYER0_PARALLEL_FINISHED line did not contain worker_threads= or states= fields, or if the count-line=LAYER0 line did not exist.  

2  
Row A: yes, yes, full_enumeration_of_the_fixed_script_stream_is_exhaustive_and_clean, not applicable  
Row B: yes, yes, full_enumeration_of_the_fixed_script_stream_is_exhaustive_and_clean, yes  
Row C: yes, no, enumerate_layer0_in_state_slices, yes  
Row D: yes, no, not applicable, not applicable  
Row E: no, no  
Observation that would overturn: If the second_transaction_step_zero_layer0.rs file contained a CHECKER line or checker_line call.  

3  
Row A: yes, yes, enumerate_layer0_in_state_slices, not applicable  
Row B: yes, yes, enumerate_layer0_in_state_slices, yes  
Row C: no, no  
Row D: no, Layer0Resume::NoProgressFile  
Observation that would overturn: If the test function passed Layer0Resume::from_environment(...) instead of NoProgressFile.  

4  
Row A: yes, yes, count_line helper, not applicable  
Row B: yes, yes, count_line helper, yes  
Row C: yes, no, parallel_finished_line helper, yes  
Row D: no, no  
Observation that would overturn: If the test function called Layer0Resume::from_environment(...) or used SINGLEFS_LAYER0_SHARD.  

5  
Row A: yes, yes, full_enumeration_of_the_parallel_line_one_stream_is_exhaustive_and_clean, not applicable  
Row B: yes, yes, full_enumeration_of_the_parallel_line_one_stream_is_exhaustive_and_clean, yes  
Row C: yes, no, enumerate_layer0_in_state_slices, yes  
Row D: yes, no, not applicable, not applicable  
Observation that would overturn: If the LAYER0_PARALLEL_FINISHED line did not contain worker_threads= or states= fields.  

6  
Row 1: LAYER0_TREE_SPLIT_CENTRAL_MAPPING_ROOT_SPLIT, yes, yes, yes  
Row 2: LAYER0_TREE_SPLIT_CENTRAL_MAPPING_LEAF_SPLIT, yes, yes, yes  
Row 3: LAYER0_TREE_SPLIT_CENTRAL_MAPPING_TWO_LEVELS_SPLIT_IN_A_ROW, yes, yes, yes  
Row 4: LAYER0_TREE_SPLIT_CENTRAL_MAPPING_EMPTY_LEAF_DROPPED, yes, yes, yes  
Row 5: LAYER0_TREE_SPLIT_CENTRAL_MAPPING_ROOT_LOWERED, yes, yes, yes  
Row 6: LAYER0_TREE_SPLIT_ACCOUNTING_ROOT_SPLIT, yes, yes, yes  
Row 7: LAYER0_TREE_SPLIT_ACCOUNTING_LEAF_SPLIT, yes, yes, yes  
The absence of shard=across-machines is consistent because the call path uses NoProgressFile, which does not read SINGLEFS_LAYER0_SHARD. Observation that would overturn: If the test function called Layer0Resume::from_environment(...) instead of NoProgressFile.  

7  
Row A: yes, yes, crash_injection.rs helper, not applicable  
Row B: no, no  
Row C: no, yes, no because admission.py only searches for "LAYER0_PARALLEL_FINISHED " lines  
Observation that would overturn: If admission.py's threads= judging also searched for "CRASH_INJECTION_FINISHED " lines.  

8  
crash-case:crash-injection-fast-tier: prints a worker-thread-count field (worker_threads={spawned_worker_threads}) in "CRASH_INJECTION_FINISHED " line but not "LAYER0_PARALLEL_FINISHED " line  

9  
The quick tier does not evaluate any of the four field kinds because it skips all #[ignore]-marked tests by default, and the gate script for quick tier only parses the test result summary line without calling admission.py or checking the field kinds. Observation that would overturn: If the quick tier ran #[ignore] tests (e.g., via --include-ignored) and the gate script called admission.py during quick tier runs.
