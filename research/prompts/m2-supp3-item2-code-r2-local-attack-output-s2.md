1. records_added_by_this_publish is 4, running_total_after_this_publish is 812, it is equal to 812, the real admission function lets it proceed past this check, yes because if reused_a_recycled_record were yes, records_added would be less, changing the arithmetic, no because running_total is not greater than 812
Would be overturned by: If the allocation record node capacity is not 812, or if a running_total of 812 is considered to exceed the capacity.

2. records_added_by_this_publish is 4, running_total_after_this_publish is 813, it is greater than 812, the real admission function returns the error AllocationRecordsExceedOneNode, yes because if reused_a_recycled_record were yes, records_added would be less, changing the arithmetic, yes because running_total is greater than 812 and base_count can be counted
Would be overturned by: If running_total is not greater than 812, or if base_count cannot be counted but a refusal still occurs.

3. records_added_by_this_publish is 4, running_total_after_this_publish is 811, it is less than 812, the real admission function lets it proceed past this check, yes because if reused_a_recycled_record were yes, records_added would be less, changing the arithmetic, no because running_total is not greater than 812
Would be overturned by: If running_total is greater than 812, or if the real admission function returns an error for a running_total less than or equal to 812.

4. records_added_by_this_publish is 3, running_total_after_this_publish is 811, it is less than 812, the real admission function lets it proceed past this check, yes because if reused_a_recycled_record were no, records_added would be 4, changing the arithmetic, no because running_total is not greater than 812
Would be overturned by: If records_added is not 3 for reused_yes, or if running_total exceeds 812.

5. records_added_by_this_publish is 16, running_total_after_this_publish is 812, it is equal to 812, the real admission function lets it proceed past this check, yes because if reused_a_recycled_record were yes, records_added would be less, changing the arithmetic, no because running_total is not greater than 812
Would be overturned by: If the running_total of 812 is considered exceeding, or if the capacity is not 812.

6. records_added_by_this_publish is 16, running_total_after_this_publish is 813, it is greater than 812, the real admission function returns the error AllocationRecordsExceedOneNode, yes because if reused_a_recycled_record were yes, records_added would be less, changing the arithmetic, yes because running_total is greater than 812 and base_count can be counted
Would be overturned by: If running_total is not greater than 812, or if the real admission function lets it proceed for running_total greater than 812.

7. records_added_by_this_publish is 11, running_total_after_this_publish is 811, it is less than 812, the real admission function lets it proceed past this check, yes because if reused_a_recycled_record were no, records_added would be 12, changing the arithmetic, no because running_total is not greater than 812
Would be overturned by: If records_added is not 11 for reused_yes, or if running_total exceeds 812.

8. records_added_by_this_publish is 4, running_total_after_this_publish cannot be computed, it cannot be determined, the real admission function cannot be determined, yes because if base_count were known, reused would affect records_added, no because base_count cannot be counted on the mirror
Would be overturned by: If base_count can be counted for this row, or if a refusal citing the wall is accepted when base_count cannot be counted.

9. 2, row definition has no readable root slot for that instance txg pair in the root ring candidate definition has rollback target is not in the root ring
Would be overturned by: If the candidate name or definition does not match "not in the root ring".

10. 4, row definition has txg is below the effective rollback floor F candidate definition has rollback target's txg is below F effective
Would be overturned by: If the txg comparison is not consistent with below F effective.

11. 1, row definition has target's txg is greater than Ti candidate definition has there is a row i Ti Wi with T greater than Ti
Would be overturned by: If T in candidate definition does not refer to txg, or if the abandoned timeline condition is not met.

12. 3, row definition has that target version's tree table has 0 entries candidate definition has rolling back to a root whose tree table has 0 entries
Would be overturned by: If the tree table entries condition does not match.

13. 5, row definition has no placement that satisfies policy on any device candidate definition has unit area cannot hold it
Would be overturned by: If no placement on any device is not due to unit area capacity, or if unit area cannot hold it is not the cause.

14. no match, row definition has some devices have no placement that satisfies policy while other devices still do how to choose the currently writable device set once a smaller device is full is undefined device set is simply every device in the pool
Would be overturned by: If candidate 5 definition includes device set selection issues.

15. no match, row definition has user-data placement each device works out from its own free-space map differs across devices placement structure uses the same slot number on both devices publish path's placement entry carries only one slot number
Would be overturned by: If candidate 5 definition includes placement consistency across devices.

16. no match, row definition has after the open segment cannot hold it devices are given different destinations cluster segments on each device need to stay aligned placement structure uses the same slot number
Would be overturned by: If candidate 5 definition includes segment alignment issues.
