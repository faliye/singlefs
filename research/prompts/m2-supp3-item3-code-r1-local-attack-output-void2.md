1 column i: yes; column ii: no; column iii: yes; column iv: F7, F8
Would be overturned by: evidence that the tally fields are not combined by addition or that the merge order is not ascending slice-number order.

2 column i: yes; column ii: no; column iii: yes; column iv: F8
Would be overturned by: evidence that known_red_hits are appended in a different order than the finished_slices list or that render does not include known_red_hits.

3 column i: yes; column ii: no; column iii: yes; column iv: F8
Would be overturned by: evidence that new_findings are appended out of ascending slice-number order or that render does not include new_findings.

4 column i: no; column ii: yes; column iii: no; column iv: F4
Would be overturned by: evidence that the finished_slices count in progress lines is not printed in arrival order or that progress lines are included in render text.

5 column i: no; column ii: yes; column iii: no; column iv: F4
Would be overturned by: evidence that the slice number in progress lines is not printed in arrival order or that progress lines are included in render text.

6 column i: no; column ii: yes; column iii: no; column iv: F4, Q1
Would be overturned by: evidence that elapsed_seconds is not printed in arrival order or that progress lines are included in render text.

7 R4
Would be overturned by: evidence that R4's column ii is no or column iii is yes.

8 the count printed after "finished_slices=" in CRASH_INJECTION_PROGRESS lines
Would be overturned by: evidence that the count printed after "finished_slices=" is identical between worker_threads=1 and worker_threads=32 runs despite different slice_count.

9 wanted: 4; slice_count: 4; spawned_worker_threads: 1
Would be overturned by: evidence that wanted is not max(worker_threads,1)*4 or slice_count is not min(wanted, seed_count) or spawned_worker_threads is not min(worker_threads.count(), slice_count).

10 wanted: 16; slice_count: 8; spawned_worker_threads: 4
Would be overturned by: evidence that wanted is not max(worker_threads,1)*4 or slice_count is not min(wanted, seed_count) or spawned_worker_threads is not min(worker_threads.count(), slice_count).

11 wanted: 128; slice_count: 8; spawned_worker_threads: 8
Would be overturned by: evidence that wanted is not max(worker_threads,1)*4 or slice_count is not min(wanted, seed_count) or spawned_worker_threads is not min(worker_threads.count(), slice_count).

12 wanted: 4; slice_count: 4; spawned_worker_threads: 1
Would be overturned by: evidence that wanted is not max(worker_threads,1)*4 or slice_count is not min(wanted, seed_count) or spawned_worker_threads is not min(worker_threads.count(), slice_count).

13 wanted: 16; slice_count: 16; spawned_worker_threads: 4
Would be overturned by: evidence that wanted is not max(worker_threads,1)*4 or slice_count is not min(wanted, seed_count) or spawned_worker_threads is not min(worker_threads.count(), slice_count).

14 wanted: 128; slice_count: 128; spawned_worker_threads: 32
Would be overturned by: evidence that wanted is not max(worker_threads,1)*4 or slice_count is not min(wanted, seed_count) or spawned_worker_threads is not min(worker_threads.count(), slice_count).

15 no
Would be overturned by: evidence that slice_count or spawned_worker_threads is identical for worker_threads=4 and worker_threads=32 when seed_count is 8.

16 no
Would be overturned by: evidence that slice_count or spawned_worker_threads is identical for worker_threads=4 and worker_threads=32 when seed_count is 4096.

17 yes; seed_count 8 with worker_threads=1 and 4 has slice_count 4 vs 8 (different), while worker_threads=4 and 32 has slice_count 8 vs 8 (same), so different slice_count behavior is exercised
Would be overturned by: evidence that seed_count 8 always produces identical slice_count or spawned_worker_threads for all worker_threads comparisons.

18 column i: not checked; column ii: nothing additional; column iii: nothing additional
Would be overturned by: evidence that position is checked by only_allocated_statistic_above_walked or form 0/1 match rules.

19 column i: not checked; column ii: nothing additional; column iii: RaiseRollbackFloor
Would be overturned by: evidence that operation_kind is checked by only_allocated_statistic_above_walked or form 0 match rule, or that form 1 requires something other than RaiseRollbackFloor.

20 column i: non-empty list of I-3.1 entries with recorded > walked; column ii: same as column i; column iii: same as column i
Would be overturned by: evidence that violations are not required to be non-empty or I-3.1 with recorded > walked for only_allocated_statistic_above_walked.

21 column i: None; column ii: nothing additional; column iii: nothing additional
Would be overturned by: evidence that panic is not required to be None for only_allocated_statistic_above_walked.

22 column i: not checked; column ii: Some and value >= root_ring_slot_count; column iii: None or Some value < root_ring_slot_count
Would be overturned by: evidence that newest_ring_root_txg is required for only_allocated_statistic_above_walked or that form 0/1 requirements differ.

23 column i: not checked; column ii: Some and value <= newest_ring_root_txg; column iii: None or Some value > newest_ring_root_txg
Would be overturned by: evidence that root_ring_slot_count is required for only_allocated_statistic_above_walked or that form 0/1 requirements differ.

24 column i: None; column ii: nothing additional; column iii: nothing additional
Would be overturned by: evidence that harness_judgement is not required to be None for only_allocated_statistic_above_walked.

25 column i: None; column ii: nothing additional; column iii: nothing additional
Would be overturned by: evidence that model_disagreement is not required to be None for only_allocated_statistic_above_walked.

26 column i: not checked; column ii: nothing additional; column iii: true
Would be overturned by: evidence that raised_floor_lands_only_on_abandoned_roots is checked by only_allocated_statistic_above_walked or form 0 match rule, or that form 1 requires something other than true.

27 KnownRed form 0
Would be overturned by: evidence that root_ring_has_turned returns false or only_allocated_statistic_above_walked returns false for U-A.

28 KnownRed form 0
Would be overturned by: evidence that root_ring_has_turned returns false or only_allocated_statistic_above_walked returns false for U-B.

29 KnownRed form 0
Would be overturned by: evidence that root_ring_has_turned returns false or only_allocated_statistic_above_walked returns false for U-C.

30 same result; differing fields: position (all differ), operation_kind (all differ), newest_ring_root_txg (U-A and U-B same, U-C differs), violations detail (U-A and U-B same, U-C differs); fields read by matcher: newest_ring_root_txg and violations detail
Would be overturned by: evidence that any of U-A, U-B, or U-C does not match form 0 or that differing fields are not read by the matcher.

31 no; field: raised_floor_lands_only_on_abandoned_roots; value: None
Would be overturned by: evidence that a crash-injection failure can be classified as form 1 despite raised_floor_lands_only_on_abandoned_roots being None.

32 no
Would be overturned by: evidence that any field in FailureObservation carries mechanism information for I-3.1 violations.

33 modify the template in check_pool_image to include the mechanism type in the detail text for I-3.1 violations
Would be overturned by: evidence that modifying the template does not allow distinguishing mechanisms or that another change is required.