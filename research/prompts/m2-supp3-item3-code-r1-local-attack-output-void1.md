1 yes no yes F4 F7 F8
Would be overturned by: A scenario where the tally fields are not combined by addition in ascending slice-number order or the render method includes other data.
2 yes no yes F4 F8
Would be overturned by: The known_red_hits list being ordered by arrival time instead of slice number or render not using it.
3 yes no yes F4 F8
Would be overturned by: The new_findings list being ordered by arrival time or render not including it.
4 no yes no F4
Would be overturned by: The "finished_slices=" count being printed in slice-number order or render including progress lines.
5 no yes no F4
Would be overturned by: The "slice=" number being printed in arrival order or render including progress lines.
6 no yes no F4
Would be overturned by: The "elapsed_seconds=" value being printed in slice-number order or render including progress lines.
7 R4
Would be overturned by: R4's column ii being no or column iii being yes.
8 CRASH_INJECTION_PROGRESS lines' "finished_slices=" count
Would be overturned by: The "finished_slices=" count being identical between worker_threads=1 and worker_threads=32 runs while render text differs.
9 4 4 1
Would be overturned by: SLICES_PER_WORKER_THREAD or seed_slices not using max(worker_threads,1)*4 or min(wanted,seed_count).
10 16 8 4
Would be overturned by: SLICES_PER_WORKER_THREAD or seed_slices not using max(worker_threads,1)*4 or min(wanted,seed_count).
11 128 8 8
Would be overturned by: SLICES_PER_WORKER_THREAD or seed_slices not using max(worker_threads,1)*4 or min(wanted,seed_count).
12 4 4 1
Would be overturned by: SLICES_PER_WORKER_THREAD or seed_slices not using max(worker_threads,1)*4 or min(wanted,seed_count).
13 16 16 4
Would be overturned by: SLICES_PER_WORKER_THREAD or seed_slices not using max(worker_threads,1)*4 or min(wanted,seed_count).
14 128 128 32
Would be overturned by: SLICES_PER_WORKER_THREAD or seed_slices not using max(worker_threads,1)*4 or min(wanted,seed_count).
15 no
Would be overturned by: slice_count or spawned_worker_threads being identical for worker_threads=4 and 32 when seed_count=8.
16 no
Would be overturned by: slice_count or spawned_worker_threads being identical for worker_threads=4 and 32 when seed_count=4096.
17 yes
Would be overturned by: seed_count=8 comparisons between worker_threads=4 and 32 having identical slice_count and spawned_worker_threads differences as between 1 and 4.
18 not checked not checked not checked
Would be overturned by: only_allocated_statistic_above_walked, form 0, or form 1 checking position.
19 not checked not checked RaiseRollbackFloor
Would be overturned by: form 1's match rule not requiring operation_kind=RaiseRollbackFloor.
20 list of entries each naming I-3.1 and detail text where recorded allocated > walked nothing additional nothing additional
Would be overturned by: only_allocated_statistic_above_walked not requiring violations to be I-3.1 with allocated > walked.
21 None nothing additional nothing additional
Would be overturned by: only_allocated_statistic_above_walked allowing panic to be Some.
22 not checked Some and value >= root_ring_slot_count None or Some with value < root_ring_slot_count
Would be overturned by: form 0's root_ring_has_turned not requiring newest_ring_root_txg >= root_ring_slot_count or form 1's not requiring <.
23 not checked Some None or Some
Would be overturned by: form 0's root_ring_has_turned not requiring root_ring_slot_count to be Some or form 1's not allowing None.
24 None nothing additional nothing additional
Would be overturned by: only_allocated_statistic_above_walked allowing harness_judgement to be Some.
25 None nothing additional nothing additional
Would be overturned by: only_allocated_statistic_above_walked allowing model_disagreement to be Some.
26 not checked not checked true
Would be overturned by: form 1's match rule not requiring raised_floor_lands_only_on_abandoned_roots=true.
27 KnownRed 0
Would be overturned by: classify_failure returning something other than KnownRed 0 for U-A.
28 KnownRed 0
Would be overturned by: classify_failure returning something other than KnownRed 0 for U-B.
29 KnownRed 0
Would be overturned by: classify_failure returning something other than KnownRed 0 for U-C.
30 same result position operation_kind newest_ring_root_txg violations
Would be overturned by: U-A U-B U-C having different classify_failure results or differing fields not matching table 3's read status.
31 no raised_floor_lands_only_on_abandoned_roots None
Would be overturned by: a crash state having raised_floor_lands_only_on_abandoned_roots=true or classify_failure matching form 1 for crash states.
32 no
Would be overturned by: a FailureObservation field carrying mechanism-specific data for form 0 matches.
33 modify the detail text template in check_pool_image to include mechanism-specific details
Would be overturned by: the current detail text template already capturing mechanism details or another change making mechanism distinction possible.