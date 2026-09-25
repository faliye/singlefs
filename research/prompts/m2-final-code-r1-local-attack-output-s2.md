1  
369 rows: 369 divided by 369 equals 1.0, ceiling is 1. Matches fact 1d.  
Falsification note: If rows_per_page were 368 instead of 369.  

370 rows: 370 divided by 369 equals approximately 1.0027, ceiling is 2. Matches fact 1d.  
Falsification note: If rows_per_page were 370 instead of 369.  

738 rows: 738 divided by 369 equals 2.0, ceiling is 2. Matches fact 1d.  
Falsification note: If rows_per_page were 368 instead of 369.  

739 rows: 739 divided by 369 equals approximately 2.0027, ceiling is 3. Matches fact 1d.  
Falsification note: If rows_per_page were 370 instead of 369.  

24354 rows: 24354 divided by 369 equals 66.0, ceiling is 66.  
Falsification note: If rows_per_page were 368 instead of 369.  

24355 rows: 24355 divided by 369 equals approximately 66.0027, ceiling is 67.  
Falsification note: If rows_per_page were 370 instead of 369.  

2  
4096 minus 311 equals 3785. 3785 divided by 56 equals 67 with remainder 33. Remainder is discarded.  
Falsification note: If JOURNAL_NAMED_ENTRY_BYTES were 55 instead of 56.  

3a  
P + 1 where P is the instance table page count.  

3b  
P + 1 > 67 → P > 66. Smallest P is 67.  
Falsification note: If JOURNAL_NAMED_ENTRIES_PER_RECORD were 68 instead of 67.  

3c  
24354 divided by 369 equals 66 exactly, so 24355 rows gives ceiling 67. Smallest row count is 24355.  
Falsification note: If rows_per_page were 368 instead of 369.  

3d  
Yes for page count 67. Yes for row count 24355.  
Falsification note: If computed page count in 3b was not 67 or computed row count in 3c was not 24355.  

4  
Scenario 1:  
i. 1  
ii. 0  
iii. 0  
Falsification note: If JOURNAL_NAMED_ENTRIES_PER_RECORD were 66 instead of 67.  

Scenario 2:  
i. 2  
ii. 0, 0  
iii. 1  
Falsification note: If JOURNAL_NAMED_ENTRIES_PER_RECORD were 68 instead of 67.  

Scenario 3:  
i. 2  
ii. 0, 0  
iii. 1  
Falsification note: If JOURNAL_NAMED_ENTRIES_PER_RECORD were 134 instead of 67.  

Scenario 4:  
i. 3  
ii. 0, 0, 0  
iii. 2  
Falsification note: If JOURNAL_NAMED_ENTRIES_PER_RECORD were 68 instead of 67.  

Scenario 5:  
i. 3  
ii. 0, 1, 2  
iii. 2  
Falsification note: If m were 68 instead of 67.  

Scenario 6:  
i. 4  
ii. 0, 1, 2, 2  
iii. 3  
Falsification note: If m were 67 instead of 68.  

Scenario 7:  
i. 4  
ii. 0, 1, 2, 2  
iii. 3  
Falsification note: If JOURNAL_NAMED_ENTRIES_PER_RECORD were 134 instead of 67.  

Scenario 8:  
i. 5  
ii. 0, 1, 2, 2, 2  
iii. 4  
Falsification note: If JOURNAL_NAMED_ENTRIES_PER_RECORD were 68 instead of 67.  

5a  
record flags offset: 7 (cumulative: magic 4, record type 2, algorithm type 1)  
ordinal-within-publish offset: 87 (cumulative: magic 4, record type 2, algorithm type 1, record flags 1, record length 4, named-item count 4, instance number 4, counter 6, checkpoint txg 8, nonce 12, header checksum 32, transaction number 8, commit flag 1)  
back-chain offset: 91 (cumulative: same as above plus ordinal-within-publish 4)  
payload checksum offset: 95 (cumulative: same as above plus back-chain 4)  

5b  
record flags offset 7: inside covered range, outside checksum span.  
ordinal-within-publish offset 87: inside covered range, outside checksum span.  
back-chain offset 91: inside covered range, outside checksum span.  
payload checksum offset 95: inside covered range, outside checksum span.  
Falsification note: If header checksum field width was different, the checksum span would change.
