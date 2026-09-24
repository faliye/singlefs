1  
yes  
if the test's assertion checked for old content instead of new, then no  
no  
if the test had an assertion checking inode write-time, then yes  
no  
if the test checked back-chain value, then yes  
yes  
if the test's assertion checked for equality, then no  
yes  
if the states_by_publish_text string didn't depend on segment attribution, then no  
no  
if the test had an assertion on tier count, then yes  
no  
if the test's assertion checked for roll-back instead of no roll-back, then yes  

2  
A unknown  
if any test references IMPLEMENTED_INVARIANTS, then yes  
B no  
if a mutation row targeted IMPLEMENTED_INVARIANTS, then yes  
C none  
if B was yes and mutation changed a constant, then constant  
D no  
if the array's sorted order was not documented, then yes  

3  
A no  
if the changed segments were public, then yes  
B yes, two rows targeting versions_applied_only_by_records  
if no rows targeted it, then no  
C behavior  
if mutations only changed constants, then constant  
D no  
if there was an undocumented choice in code, then yes  

4  
A unknown  
if a test calls base_image_tiers_sampled, then yes  
B yes, one row targeting BaseImageTier and newest_root_tree_table_has_no_entries  
if no such row, then no  
C behavior  
if mutation changed constants, then constant  
D unknown  
if there was an undocumented choice, then yes  

5  
A unknown  
if a test uses Layer0PublishOfState, then yes  
B yes, one row targeting Layer0PublishOfState  
if no such row, then no  
C behavior  
if mutation changed constants, then constant  
D unknown  
if undocumented choice, then yes  

6  
A unknown  
if tests reference new items, then yes  
B no  
if any mutation row targeted them, then yes  
C none  
if B was yes, then behavior or constant  
D unknown  

7  
A unknown  
if tests call the changed functions, then yes  
B no  
if any mutation row targeted them, then yes  
C none  

D unknown  

8  
A unknown  
if tests used the removed variant, then yes  
B no  
if mutation row targeted it, then yes  
C none  

D unknown  

9  
A unknown  
if tests call the changed functions, then yes  
B no  

C none  

D unknown  

10  
A unknown  
if tests reference MountError variant or tree_identifier_watermark_of_the_ring, then yes  
B yes, one row mentioning tree_identifier_watermark_of_the_ring in a struct  
if the row didn't mention it, then no  
C behavior  
if mutation changed constants, then constant  
D unknown  

11  
A unknown  
if tests call open_pool_for_read, then yes  
B no  
if mutation row targeted it, then yes  
C none  

D unknown  

12  
A unknown  
if tests call new functions, then yes  
B no  

C none  

D unknown  

13  
A yes  
if the inline test wasn't added, then no  
B no  
if there was a mutation row, then yes  
C none  

D no  
if there was an undocumented choice in parsing code, then yes  

14  
A unknown  
if tests reference new items, then yes  
B yes, one row targeting FileVersionTreeIdentifiers and issued_from_watermark  
if no, then no  
C behavior  
if mutation changed constants, then constant  
D unknown
