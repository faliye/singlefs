1  
369 rows: 369 divided by 369 equals 1.0. Ceiling is 1. Matches fact 1d.  
If rows_per_page was 368 instead of 369, the result would be 2.  

370 rows: 370 divided by 369 equals approximately 1.0027. Ceiling is 2. Matches fact 1d.  
If rows_per_page was 370 instead of 369, the result would be 1.  

738 rows: 738 divided by 369 equals 2.0. Ceiling is 2. Matches fact 1d.  
If rows_per_page was 368 instead of 369, the result would be 3.  

739 rows: 739 divided by 369 equals approximately 2.0027. Ceiling is 3. Matches fact 1d.  
If rows_per_page was 370 instead of 369, the result would be 2.  

24354 rows: 24354 divided by 369 equals 66.0. Ceiling is 66.  
If rows_per_page was 368 instead of 369, the result would be 67.  

24355 rows: 24355 divided by 369 equals approximately 66.0027. Ceiling is 67.  
If rows_per_page was 370 instead of 369, the result would be 66.  

2  
4096 minus 311 equals 3785. 3785 divided by 56 equals 67 with remainder 33. Remainder is discarded. Final integer is 67.  
If JOURNAL_HEADER_BYTES was 288 instead of 311, the result would be 68.  

3a  
P + 1  

3b  
P + 1 > 67 → P > 66. Smallest P is 67.  
If JOURNAL_NAMED_ENTRIES_PER_RECORD was 68 instead of 67, the smallest P would be 68.  

3c  
ceiling(rows / 369) = 67 → rows > 66 * 369 = 24354. Smallest row count is 24355.  
If rows_per_page was 370 instead of 369, the smallest row count would be 24355 / 370 → ceil 66 → 24355 would produce 66 pages.  

3d  
Yes, computed threshold page count is 67. Yes, computed threshold row count is 24355.  
If JOURNAL_NAMED_ENTRIES_PER_RECORD was 68 instead of 67, the threshold page count would be 68 instead of 67.  

4  
Scenario 1:  
(i) 1 record  
(ii) Record 0 belongs to transaction 0  
(iii) Record index 0 carries the last-record flag  
If m was 68 instead of 67, the total records would be 2.  

Scenario 2:  
(i) 2 records  
(ii) Record 0 belongs to transaction 0; record 1 belongs to transaction 0  
(iii) Record index 1 carries the last-record flag  
If m was 67 instead of 68, the total records would be 1.  

Scenario 3:  
(i) 2 records  
(ii) Record 0 belongs to transaction 0; record 1 belongs to transaction 0  
(iii) Record index 1 carries the last-record flag  
If m was 133 instead of 134, the total records would be 2 (133/67=1.985 → ceil 2).  

Scenario 4:  
(i) 3 records  
(ii) Record 0 belongs to transaction 0; record 1 belongs to transaction 0; record 2 belongs to transaction 0  
(iii) Record index 2 carries the last-record flag  
If m was 134 instead of 135, the total records would be 2.  

Scenario 5:  
(i) 3 records  
(ii) Record 0 belongs to transaction 0; record 1 belongs to transaction 1; record 2 belongs to transaction 2  
(iii) Record index 2 carries the last-record flag  
If m was 68 instead of 67, the total records would be 4.  

Scenario 6:  
(i) 4 records  
(ii) Record 0 belongs to transaction 0; record 1 belongs to transaction 1; record 2 belongs to transaction 2; record 3 belongs to transaction 2  
(iii) Record index 3 carries the last-record flag  
If m was 67 instead of 68, the total records would be 3.  

Scenario 7:  
(i) 4 records  
(ii) Record 0 belongs to transaction 0; record 1 belongs to transaction 1; record 2 belongs to transaction 2; record 3 belongs to transaction 2  
(iii) Record index 3 carries the last-record flag  
If m was 133 instead of 134, the total records would be 4 (133/67=1.985 → ceil 2 for last transaction, total records 2+2=4).  

Scenario 8:  
(i) 5 records  
(ii) Record 0 belongs to transaction 0; record 1 belongs to transaction 1; record 2 belongs to transaction 2; record 3 belongs to transaction 2; record 4 belongs to transaction 2  
(iii) Record index 4 carries the last-record flag  
If m was 134 instead of 135, the total records would be 4.  

5a  
record flags: cumulative widths up to start = 4 (magic) + 2 (record type) + 1 (algorithm type) = 7  
ordinal-within-publish: cumulative widths up to start = 4 + 2 + 1 + 1 + 4 + 4 + 4 + 6 + 8 + 12 + 32 + 8 + 1 = 87  
back-chain: cumulative widths up to start = 4 + 2 + 1 + 1 + 4 + 4 + 4 + 6 + 8 + 12 + 32 + 8 + 1 + 4 = 91  
payload checksum: cumulative widths up to start = 4 + 2 + 1 + 1 + 4 + 4 + 4 + 6 + 8 + 12 + 32 + 8 + 1 + 4 + 4 = 95  

5b  
record flags: offset 7 is inside covered range but outside checksum field span.  
If the header checksum field started at offset 0, record flags would be inside it.  
ordinal-within-publish: offset 87 is inside covered range but outside checksum field span.  
If the header checksum field extended to offset 87, ordinal-within-publish would be inside it.  
back-chain: offset 91 is inside covered range but outside checksum field span.  
If the header checksum field extended to offset 91, back-chain would be inside it.  
payload checksum: offset 95 is inside covered range but outside checksum field span.  
If the header checksum field extended to offset 95, payload checksum would be inside it.
