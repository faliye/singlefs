1 column i: yes; column ii: no; column iii: yes; column iv: F7; F8; F6  
Would be overturned by: If the tally fields were merged in an order other than ascending slice-number, or if the tally was not included in the render text, or if the merge process did not use F7 and F8.  

2 column i: yes; column ii: no; column iii: yes; column iv: F8; F4; F6  
Would be overturned by: If the known_red_hits list was not built in ascending slice-number order, or if it was not included in the render text, or if the build process did not use F8, F4, or F6.  

3 column i: yes; column ii: no; column iii: yes; column iv: F8; F6  
Would be overturned by: If the new_findings list was not built in ascending slice-number order, or if it was not included in the render text, or if the build process did not use F8 or F6.  

4 column i: no; column ii: yes; column iii: no; column iv: F4  
Would be overturned by: If the count after "finished_slices=" was not printed in arrival order, or if it was part of the render text, or if the printing process did not use F4.  

5 column i: no; column ii: yes; column iii: no; column iv: F4  
Would be overturned by: If the slice number in CRASH_INJECTION_PROGRESS lines was not printed in arrival order, or if it was part of the render text, or if the printing process did not use F4.  

6 column i: no; column ii: no; column iii: no; column iv: none  
Would be overturned by: If the "elapsed_seconds=" number was printed or included in the render text, or if any fact indicated its existence.  

7 R4  
Would be overturned by: If R4's column ii was no or column iii was yes.  

8 the count printed after "finished_slices=" in CRASH_INJECTION_PROGRESS lines  
Would be overturned by: If the count after "finished_slices=" was identical between worker_threads=1 and worker_threads=32 runs despite different slice_count, or if the render text differed.  

9 wanted: 4; slice_count: 4; spawned_worker_threads: 1  
Would be overturned by: If the actual values differed from these calculations using F1 and F2.  

10 wanted: 16; slice_count: 8; spawned_worker_threads: 4  
Would be overturned by: If the actual values differed from these calculations using F1 and F2.  

11 wanted: 128; slice_count: 8; spawned_worker_threads: 8  
Would be overturned by: If the actual values differed from these calculations using F1 and F2.  

12 wanted: 4; slice_count: 4; spawned_worker_threads: 1  
Would be overturned by: If the actual values differed from these calculations using F1 and F2.  

13 wanted: 16; slice_count: 16; spawned_worker_threads: 4  
Would be overturned by: If the actual values differed from these calculations using F1 and F2.  

14 wanted: 128; slice_count: 128; spawned_worker_threads: 32  
Would be overturned by: If the actual values differed from these calculations using F1 and F2.  

15 no  
Would be overturned by: If slice_count and spawned_worker_threads were identical for worker_threads=4 and worker_threads=32 at seed_count=8.  

16 no  
Would be overturned by: If slice_count and spawned_worker_threads were identical for worker_threads=4 and worker_threads=32 at seed_count=4096.  

17 yes  
Would be overturned by: If the slice_count and spawned_worker_threads comparisons between worker_threads=1 and 4 at seed_count=8 were identical to those between worker_threads=4 and 32 at seed_count=8.  

18 column i: not checked by this function; column ii: nothing additional; column iii: nothing additional  
Would be overturned by: If only_allocated_statistic_above_walked, form 0, or form 1 checked the position field.  

19 column i: not checked by this function; column ii: nothing additional; column iii: RaiseRollbackFloor  
Would be overturned by: If form 1's match rule did not require operation_kind to be RaiseRollbackFloor.  

20 column i: non-empty list where each entry names I-3.1 and detail text has recorded allocated > walked; column ii: non-empty list where each entry names I-3.1 and detail text has recorded allocated > walked; column iii: non-empty list where each entry names I-3.1 and detail text has recorded allocated > walked  
Would be overturned by: If only_allocated_statistic_above_walked, form 0, or form 1 did not require violations to be non-empty with allocated > walked.  

21 column i: None; column ii: None; column iii: None  
Would be overturned by: If only_allocated_statistic_above_walked, form 0, or form 1 allowed panic to be non-None.  

22 column i: not checked by this function; column ii: Some and >= root_ring_slot_count; column iii: None or Some value less than root_ring_slot_count  
Would be overturned by: If root_ring_has_turned did not require newest_ring_root_txg to be Some and >= root_ring_slot_count for form 0, or if form 1 did not require it to be None or < root_ring_slot_count.  

23 column i: not checked by this function; column ii: Some and <= newest_ring_root_txg; column iii: None or Some value greater than newest_ring_root_txg  
Would be overturned by: If root_ring_has_turned did not require root_ring_slot_count to be Some and <= newest_ring_root_txg for form 0, or if form 1 did not require it to be None or > newest_ring_root_txg.  

24 column i: None; column ii: None; column iii: None  
Would be overturned by: If only_allocated_statistic_above_walked, form 0, or form 1 allowed harness_judgement to be non-None.  

25 column i: None; column ii: None; column iii: None  
Would be overturned by: If only_allocated_statistic_above_walked, form 0, or form 1 allowed model_disagreement to be non-None.  

26 column i: not checked by this function; column ii: nothing additional; column iii: true  
Would be overturned by: If form 1's match rule did not require raised_floor_lands_only_on_abandoned_roots to be true.  

27 KnownRed with form number 0  
Would be overturned by: If the root_ring_has_turned or only_allocated_statistic_above_walked conditions did not hold for U-A.  

28 KnownRed with form number 0  
Would be overturned by: If the root_ring_has_turned or only_allocated_statistic_above_walked conditions did not hold for U-B.  

29 KnownRed with form number 0  
Would be overturned by: If the root_ring_has_turned or only_allocated_statistic_above_walked conditions did not hold for U-C.  

30 same result; fields that differ: position; operation_kind; newest_ring_root_txg; violations detail; position is not read by matcher; operation_kind is not read by matcher; newest_ring_root_txg is read but all values satisfy condition; violations detail is read but all satisfy allocated > walked  
Would be overturned by: If any of U-A, U-B, or U-C had different classification results.  

31 no; raised_floor_lands_only_on_abandoned_roots; None  
Would be overturned by: If a crash-injection failure could be classified as form 1 despite raised_floor_lands_only_on_abandoned_roots being None.  

32 no  
Would be overturned by: If any field in FailureObservation read by the matcher functions could distinguish between failures with different mechanisms but same form 0 shape.  

33 change the template used for generating I-3.1 detail text in violations to include a mechanism identifier  
Would be overturned by: If changing the template did not allow distinguishing between different mechanisms for form 0 failures.
