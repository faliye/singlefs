1  
yes  
if the test's assert checked the old content instead of the new content, the mutation would not fail it.  
no  
if the test checked the inode write-time field directly, the mutation would fail it.  
no  
if the test checked the back-chain field directly, the mutation would fail it.  
yes  
if the assert_ne checked for equality instead of inequality, the mutation would not fail it.  
yes  
if the states_by_publish_text string comparison expected the mutated state breakdown, the mutation would not fail it.  
no  
if the test asserted on the number of baseline tiers or the presence of the "zero tree-table entries" tier, the mutation would fail it.  
yes  
if the assertion on acquired_instances_left expected the old roll-back behavior, the mutation would not fail it.  

2  
A: unknown  
if a test in the 39 harness files directly references IMPLEMENTED_INVARIANTS, the answer would be yes.  
B: no  
if any mutations.tsv row contained IMPLEMENTED_INVARIANTS or "I-3.10", the answer would be yes.  
C: none  
if column B was yes, the answer would depend on whether the mutation changed behavior or a constant.  
D: no  
if the array insertion was not sorted by code logic but manual, the answer would be yes.  

3  
A: no  
if walk.rs had an inline #[test] function that called the new private functions, the answer would be yes.  
B: yes  
if no mutations.tsv rows targeted versions_applied_only_by_records, the answer would be no.  
C: behaviour  
if the mutation only changed a constant like a numeric literal, the answer would be constant.  
D: no  
if the code had undocumented choices for the new functions, the answer would be yes.  

4  
A: unknown  
if any test in the 39 harness files directly calls base_image_tiers_sampled or constructs BaseImageTier, the answer would be yes.  
B: yes  
if no mutations.tsv rows targeted BaseImageTier or newest_root_tree_table_has_no_entries, the answer would be no.  
C: behaviour  
if the mutation only changed a constant like a string literal, the answer would be constant.  
D: unknown  
if the enum BaseImageTier's tier definitions were documented in code comments, the answer would be no.  

5  
A: unknown  
if any test in the 39 harness files directly calls Layer0PublishOfState or states_by_publish_text, the answer would be yes.  
B: yes  
if no mutations.tsv rows targeted Layer0PublishOfState, the answer would be no.  
C: behaviour  
if the mutation only changed a constant like a numeric literal, the answer would be constant.  
D: unknown  
if the code had undocumented choices for publish_of_each_segment, the answer would be yes.  

6  
A: yes  
if no test in the 39 harness files called any new public functions or enums, the answer would be no.  
B: no  
if any mutations.tsv rows targeted AcquiredInstanceLeftAfterAFailedMount or similar, the answer would be yes.  
C: none  
if column B was yes, the answer would depend on mutation type.  
D: unknown  
if the new enum choices were documented in code comments, the answer would be no.  

7  
A: no  
if any test in the 39 harness files directly called publish_error_member or mount_error_member, the answer would be yes.  
B: no  
if any mutations.tsv rows targeted publish_error_member or mount_error_member, the answer would be yes.  
C: none  
if column B was yes, the answer would depend on mutation type.  
D: unknown  
if the match arm changes were undocumented, the answer would be yes.  

8  
A: no  
if any test in the 39 harness files directly referenced RollbackToVersionWithoutFileWhileTheRingStillHoldsAFileVersion, the answer would be yes.  
B: no  
if any mutations.tsv rows targeted RollbackToVersionWithoutFileWhileTheRingStillHoldsAFileVersion, the answer would be yes.  
C: none  
if column B was yes, the answer would depend on mutation type.  
D: no  
if the removal of the variant was undocumented, the answer would be yes.  

9  
A: no  
if any test in the 39 harness files directly called refusal_reason_of_publish_error or similar, the answer would be yes.  
B: no  
if any mutations.tsv rows targeted refusal_reason_of_publish_error or similar, the answer would be yes.  
C: none  
if column B was yes, the answer would depend on mutation type.  
D: no  
if the doc comment changes were undocumented, the answer would be yes.  

10  
A: no  
if any test in the 39 harness files directly called tree_identifier_watermark_of_the_ring, the answer would be yes.  
B: unknown  
if the mutations.tsv row for C378 actually modified tree_identifier_watermark_of_the_ring, the answer would be yes.  
C: none  
if column B was yes, the answer would depend on mutation type.  
D: no  
if the tree_identifier_watermark_of_the_ring field was undocumented, the answer would be yes.  

11  
A: yes  
if no test in the 39 harness files called open_pool_for_read, the answer would be no.  
B: no  
if any mutations.tsv rows targeted open_pool_for_read or TREE_IDENTIFIER_CENTRAL_MAPPING, the answer would be yes.  
C: none  
if column B was yes, the answer would depend on mutation type.  
D: no  
if the birth-tree field usage was undocumented, the answer would be yes.  

12  
A: yes  
if no test in the 39 harness files called highest_tree_identifier_watermark_in_the_ring or tree_table_entry_count, the answer would be no.  
B: no  
if any mutations.tsv rows targeted highest_tree_identifier_watermark_in_the_ring or tree_table_entry_count, the answer would be yes.  
C: none  
if column B was yes, the answer would depend on mutation type.  
D: unknown  
if the logic for the new functions was undocumented, the answer would be yes.  

13  
A: yes  
if the inline test root_record_is_457_bytes_with_the_checksum_at_138_and_round_trips did not exist, the answer would be no.  
B: no  
if any mutations.tsv rows existed for root_record.rs, the answer would be yes.  
C: none  
if column B was yes, the answer would depend on mutation type.  
D: no  
if the size change was undocumented, the answer would be yes.  

14  
A: yes  
if no test in the 39 harness files called publish_first_file or related functions, the answer would be no.  
B: yes  
if no mutations.tsv rows targeted FileVersionTreeIdentifiers or issued_from_watermark, the answer would be no.  
C: behaviour  
if the mutation only changed a constant like a numeric literal, the answer would be constant.  
D: no  
if the tree_table_entries check was undocumented, the answer would be yes.
