1 column i: yes; column ii: no; column iii: yes; column iv: F7, F8, F6  
Would be overturned by: the numeric fields of the report's tally being merged in an order other than ascending slice-number order, or the render function including elements beyond tally, known_red_hits, and new_findings.  

2 column i: yes; column ii: no; column iii: yes; column iv: F4, F8, F6  
Would be overturned by: the known_red_hits list being ordered differently than ascending slice-number order, or the render function excluding known_red_hits from its output.  

3 column i: yes; column ii: no; column iii: yes; column iv: F4, F8, F6  
Would be overturned by: the new_findings list being ordered differently than ascending slice-number order, or the render function excluding new_findings from its output.  

4 column i: no; column ii: yes; column iii: no; column iv: F4, F6  
Would be overturned by: the count printed after "finished_slices=" not being printed in arrival order, or the progress lines being included in the render function's output.  

5 column i: no; column ii: yes; column iii: no; column iv: F4, F6  
Would be overturned by: the number printed after "slice=" not being printed in arrival order, or the progress lines being included in the render function's output.  

6 column i: no; column ii: yes; column iii: no; column iv: F4, F6  
Would be overturned by: the number printed after "elapsed_seconds=" not being printed in arrival order, or the progress lines being included in the render function's output.  

7 R4  
Would be overturned by: any row other than R4, R5, or R6 having column ii yes and column iii no.  

8 the count printed after "finished_slices=" inside each CRASH_INJECTION_PROGRESS line  
Would be overturned by: the count printed after "finished_slices=" being identical between worker_threads=1 and worker_threads=32 runs despite differing slice_count.  

9 wanted: 4; slice_count: 4; spawned_worker_threads: 1  
Would be overturned by: the calculation of wanted, slice_count, or spawned_worker_threads for seed_count 8 and worker_threads 1 differing from these values.  

10 wanted: 16; slice_count: 8; spawned_worker_threads: 4  
Would be overturned by: the calculation of wanted, slice_count, or spawned_worker_threads for seed_count 8 and worker_threads 4 differing from these values.  

11 wanted: 128; slice_count: 8; spawned_worker_threads: 8  
Would be overturned by: the calculation of wanted, slice_count, or spawned_worker_threads for seed_count 8 and worker_threads 32 differing from these values.  

12 wanted: 4; slice_count: 4; spawned_worker_threads: 1  
Would be overturned by: the calculation of wanted, slice_count, or spawned_worker_threads for seed_count 4096 and worker_threads 1 differing from these values.  

13 wanted: 16; slice_count: 16; spawned_worker_threads: 4  
Would be overturned by: the calculation of wanted, slice_count, or spawned_worker_threads for seed_count 4096 and worker_threads 4 differing from these values.  

14 wanted: 128; slice_count: 128; spawned_worker_threads: 32  
Would be overturned by: the calculation of wanted, slice_count, or spawned_worker_threads for seed_count 4096 and worker_threads 32 differing from these values.  

15 no  
Would be overturned by: slice_count or spawned_worker_threads being identical for worker_threads=4 and worker_threads=32 when seed_count is 8.  

16 yes  
Would be overturned by: slice_count or spawned_worker_threads being identical for worker_threads=4 and worker_threads=32 when seed_count is 4096.  

17 yes  
Would be overturned by: seed_count 8 never causing slice_count or spawned_worker_threads to differ between worker_threads=4 and worker_threads=32 comparisons versus worker_threads=1 and worker_threads=4 comparisons.  

18 column i: not checked by this function; column ii: nothing additional; column iii: nothing additional  
Would be overturned by: position being checked by only_allocated_statistic_above_walked, form 0, or form 1.  

19 column i: not checked by this function; column ii: nothing additional; column iii: RaiseRollbackFloor  
Would be overturned by: operation_kind being checked by only_allocated_statistic_above_walked or form 0, or form 1 not requiring RaiseRollbackFloor.  

20 column i: list with at least one entry where invariant name is I-3.1 and recorded allocated > walked; column ii: same as column i; column iii: same as column i  
Would be overturned by: violations not requiring I-3.1 entries with recorded allocated > walked for only_allocated_statistic_above_walked, form 0, or form 1.  

21 column i: None; column ii: None; column iii: None  
Would be overturned by: panic not requiring None for only_allocated_statistic_above_walked, form 0, or form 1.  

22 column i: not checked by this function; column ii: Some value greater than or equal to root_ring_slot_count; column iii: Some value less than root_ring_slot_count if root_ring_slot_count is present; otherwise any value  
Would be overturned by: newest_ring_root_txg not requiring specific values for root_ring_has_turned in form 0 or form 1.  

23 column i: not checked by this function; column ii: Some value less than or equal to newest_ring_root_txg; column iii: Some value greater than newest_ring_root_txg if newest_ring_root_txg is present; otherwise any value  
Would be overturned by: root_ring_slot_count not requiring specific values for root_ring_has_turned in form 0 or form 1.  

24 column i: None; column ii: None; column iii: None  
Would be overturned by: harness_judgement not requiring None for only_allocated_statistic_above_walked, form 0, or form 1.  

25 column i: None; column ii: None; column iii: None  
Would be overturned by: model_disagreement not requiring None for only_allocated_statistic_above_walked, form 0, or form 1.  

26 column i: not checked by this function; column ii: nothing additional; column iii: true  
Would be overturned by: raised_floor_lands_only_on_abandoned_roots being checked by only_allocated_statistic_above_walked or form 0, or form 1 not requiring true.  

27 KnownRed with form number 0  
Would be overturned by: classify_failure returning anything other than KnownRed with form number 0 for row U-A.  

28 KnownRed with form number 0  
Would be overturned by: classify_failure returning anything other than KnownRed with form number 0 for row U-B.  

29 KnownRed with form number 0  
Would be overturned by: classify_failure returning anything other than KnownRed with form number 0 for row U-C.  

30 same result; position; operation_kind; newest_ring_root_txg; violations detail text; position is not read by matcher; operation_kind is not read by matcher; newest_ring_root_txg is read by matcher; violations detail text is read by matcher  
Would be overturned by: any field differing between U-A, U-B, U-C being read by the matcher function and causing different results.  

31 no; raised_floor_lands_only_on_abandoned_roots; None  
Would be overturned by: a crash-injection failure being classified as known red form 1 despite raised_floor_lands_only_on_abandoned_roots being None.  

32 no  
Would be overturned by: any field in FailureObservation carrying information that distinguishes between two failures with the same known red form 0 shape but different mechanisms.  

33 change the detail text template for I-3.1 to include the mechanism type  
Would be overturned by: changing the detail text template for I-3.1 not enabling distinction between different mechanisms for known red form 0 failures.
