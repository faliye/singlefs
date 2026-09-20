1
4
812
equal to 812
lets it proceed past this check
no, because Rule R1 states records_added is rewritten_roles multiplied by devices regardless of reuse
no
Would be overturned by: running_total_after_this_publish being greater than 812

2
4
813
greater than 812
returns the error AllocationRecordsExceedOneNode
no, because Rule R1 states records_added is rewritten_roles multiplied by devices regardless of reuse
yes
Would be overturned by: running_total_after_this_publish being less than or equal to 812

3
4
811
less than 812
lets it proceed past this check
no, because Rule R1 states records_added is rewritten_roles multiplied by devices regardless of reuse
no
Would be overturned by: running_total_after_this_publish being greater than 812

4
4
812
equal to 812
lets it proceed past this check
no, because Rule R1 states records_added is rewritten_roles multiplied by devices regardless of reuse
no
Would be overturned by: running_total_after_this_publish being greater than 812

5
16
812
equal to 812
lets it proceed past this check
no, because Rule R1 states records_added is rewritten_roles multiplied by devices regardless of reuse
no
Would be overturned by: running_total_after_this_publish being greater than 812

6
16
813
greater than 812
returns the error AllocationRecordsExceedOneNode
no, because Rule R1 states records_added is rewritten_roles multiplied by devices regardless of reuse
yes
Would be overturned by: running_total_after_this_publish being less than or equal to 812

7
12
812
equal to 812
lets it proceed past this check
no, because Rule R1 states records_added is rewritten_roles multiplied by devices regardless of reuse
no
Would be overturned by: running_total_after_this_publish being greater than 812

8
cannot be computed
cannot be determined
cannot be determined
cannot be determined
no, because Rule R1 states records_added is rewritten_roles multiplied by devices regardless of reuse
no
Would be overturned by: base_count_on_mirror being countable and running_total_after_this_publish greater than 812

9
2
The definition states "no readable root slot for that (instance, txg) pair in the root ring" and candidate 2 states "rollback target is not in the root ring" with both referencing decision D23 settled item 14
Would be overturned by: candidate 2's definition not mentioning the root ring or the target not being in it

10
4
The definition states "txg is below the effective rollback floor F" and candidate 4 states "rollback target's txg is below F effective" with both referencing decision D16 settled item 1
Would be overturned by: candidate 4's definition not mentioning the txg being below F effective

11
1
The definition states "target's txg is greater than Ti" and candidate 1 states "rollback target is on an abandoned timeline" with both referencing a row (i, Ti, Wi) where T greater than Ti
Would be overturned by: candidate 1's definition not mentioning the instance table row with T greater than Ti

12
3
The definition states "tree table has 0 entries" and candidate 3 states "tree table has 0 entries" with both stating the first version does not support rolling back to such a version
Would be overturned by: candidate 3's definition not mentioning the tree table having 0 entries

13
no match
The definition states "no placement that satisfies policy on any device" but candidate 5 states "unit area cannot hold it" with no matching words for "no placement on any device" versus "unit area"
Would be overturned by: candidate 5's definition matching "no placement that satisfies policy on any device"

14
no match
The definition states "some devices have no placement that satisfies policy while other devices still do" but candidate 5 states "unit area cannot hold it" with no matching words for device-specific issues versus unit area
Would be overturned by: candidate 5's definition matching "some devices have no placement that satisfies policy while other devices still do"

15
no match
The definition states "user-data placement each device works out from its own free-space map differs across devices" but candidate 5 states "unit area cannot hold it" with no matching words for differing placements versus unit area
Would be overturned by: candidate 5's definition matching "user-data placement each device works out from its own free-space map differs across devices"

16
no match
The definition states "devices are given different destinations... cluster segments on each device need to stay aligned" but candidate 5 states "unit area cannot hold it" with no matching words for segment alignment versus unit area
Would be overturned by: candidate 5's definition matching "devices are given different destinations... cluster segments on each device need to stay aligned"
