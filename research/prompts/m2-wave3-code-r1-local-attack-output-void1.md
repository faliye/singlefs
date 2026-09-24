1
yes. if the test's assertion checked for the old content instead of the new, then the mutation would not cause the test to fail.
no. if the test included an assertion checking the inode's write-time field, then the mutation would cause a failure.
no. if the test included an assertion checking the back-chain field, then the mutation would cause a failure.
yes. if the assertion checked for equality instead of inequality, then the mutation would not cause the test to fail.
yes. if the test's assertion compared against a different string that included the off-by-one error, then the mutation would not cause the test to fail.
no. if the test included an assertion checking the number of baseline tiers, then the mutation would cause a failure.
no. if the test's assertion checked that the instance was rolled back instead of left in place, then the mutation would cause a failure.

2
A: no. if a test directly referenced IMPLEMENTED_INVARIANTS.
B: no. if a mutations.tsv row contained IMPLEMENTED_INVARIANTS or I-3.10.
C: none. if there was a mutation targeting the constant array.
D: no. if the array's order affected on-disk bytes or history acceptance.

3
A: unknown. if the test file name for versions_applied_only_by_records was provided.
B: yes. if no mutations.tsv rows targeted any changed segments.
C: behavior. if the mutation changed a constant or data-only entry.
D: no. if the code choices affected on-disk bytes or history acceptance.

4
A: yes. second_transaction_supplement_three_bad_disk_input.rs. if no test named the segments.
B: yes. if no mutations.tsv rows targeted the new enum or function.
C: behavior. if the mutation changed a constant or data-only entry.
D: no. if the tier sampling choices affected on-disk bytes or history acceptance.

5
A: yes. second_transaction_step_zero_layer0.rs. if no test named the Layer0PublishOfState enum.
B: yes. if no mutations.tsv rows targeted the Layer0PublishOfState enum.
C: behavior. if the mutation changed a constant or data-only entry.
D: no. if the code choices affected on-disk bytes or history acceptance.

6
A: unknown. if a test named any of the new enums or functions.
B: no. if mutations.tsv rows targeted any of the new identifiers.
C: none. if there was a mutation targeting the new code.
D: no. if the code choices affected on-disk bytes or history acceptance.

7
A: unknown. if a test named the changed functions.
B: no. if mutations.tsv rows targeted the changed functions.
C: none. if there was a mutation targeting the changed code.
D: no. if the code choices affected on-disk bytes or history acceptance.

8
A: unknown. if a test named the removed enum variant.
B: no. if mutations.tsv rows targeted the removed variant.
C: none. if there was a mutation targeting the removed variant.
D: no. if the code choices affected on-disk bytes or history acceptance.

9
A: unknown. if a test named the changed functions.
B: no. if mutations.tsv rows targeted the changed functions.
C: none. if there was a mutation targeting the changed code.
D: no. if the code choices affected on-disk bytes or history acceptance.

10
A: unknown. if a test named the new field or function.
B: unknown. if the mutations.tsv row actually targeted the tree_identifier_watermark_of_the_ring field.
C: none. if there was a mutation targeting the new field.
D: yes. if a different choice for computing the watermark would not affect on-disk bytes.

11
A: unknown. if a test named the open_pool_for_read function.
B: no. if mutations.tsv rows targeted the open_pool_for_read change.
C: none. if there was a mutation targeting the change.
D: yes. if a different choice for reading the birth-tree would not affect on-disk bytes.

12
A: unknown. if a test named the new functions.
B: no. if mutations.tsv rows targeted the new functions.
C: none. if there was a mutation targeting the new functions.
D: no. if the code choices affected on-disk bytes or history acceptance.

13
A: yes. root_record.rs. if no inline test named the root record size change.
B: no. if mutations.tsv rows targeted the root record changes.
C: none. if there was a mutation targeting the root record.
D: no. if the size change choice affected on-disk bytes.

14
A: yes. second_transaction_step_one_overwrite.rs. if no test named the FileVersionTreeIdentifiers struct.
B: yes. if no mutations.tsv rows targeted the FileVersionTreeIdentifiers struct.
C: behavior. if the mutation changed a constant or data-only entry.
D: yes. if a different choice for checking tree_table_entries would not affect on-disk bytes.