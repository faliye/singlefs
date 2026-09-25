Task: pure arithmetic and formula-application check on five numeric facts drawn from the singlefs project's Rust source. You are not being asked to judge any design decision; you are being asked to compute exact numbers from formulas given to you below, and to show your derivation for every number.

Ground rules for your answer.
1. Answer every numbered question below, in order, using the number and letter given.
2. For every cell or scenario you fill in, also write one short sentence: what fact, if it were different, would flip this answer. This is your falsification note.
3. Do not cite source-code line numbers or file line numbers anywhere in your answer. Refer to functions or constants by the name given to you in this prompt, and refer to scenarios by the scenario number given in this prompt.
4. If the facts given to you in this prompt are not enough to answer a cell, write unknown, and say what fact is missing. Writing unknown is an acceptable answer, not a failure.
5. Do not use markdown emphasis (no bold, no italics) anywhere in your answer.
6. Answer everything in English.
7. Use only the facts given in this prompt. Do not assume anything about code you have not been shown here.
8. Show your arithmetic step by step for every numeric answer (the subtraction, the division, the rounding direction, whichever applies); do not just state a final number.

Background. All facts below are quoted or closely paraphrased from the singlefs project's Rust source comments and code. Some facts carry a project-internal citation in parentheses (a decision tag such as D18, or an invariant tag such as I-2.4, or a date such as 2026-09-24). You do not need to know what those cited decisions say; treat each such tag as an opaque label for where the number comes from, kept only for traceability.

Section 1. Question 1: how many pages an instance table needs.

Fact 1a. A constant named INSTANCE_TABLE_PAGE_RECORDS equals 370: one page of the instance table holds 370 records in total.

Fact 1b. Of those 370 records in a page, the last one is always a chain-pointer record; the other 369 records each carry one data row of the table (decision D18, settled item 11). A page with zero data rows is still one whole page (the page that mkfs writes out holds only the chain-pointer record, no data rows).

Fact 1c. A function named instance_table_pages_for_rows computes how many pages are needed to hold a given row count, defined as: rows_per_page = INSTANCE_TABLE_PAGE_RECORDS minus 1, that is, 369; pages = ceiling(rows divided by rows_per_page), with a floor of 1 page even when rows = 0. (This is stated to be the same number as a separate decision, D28 settled item 3, whose own formula is written as max(1, ceiling(row count / rows per page)).)

Fact 1d. The project's own test suite asserts these exact input/output pairs for instance_table_pages_for_rows: input 0 rows gives 1 page; input 369 rows gives 1 page; input 370 rows gives 2 pages; input 738 rows gives 2 pages; input 739 rows gives 3 pages. Treat fact 1d as a check on fact 1c, not as an additional rule; if your computed answer for one of these five inputs, using fact 1c, disagrees with fact 1d, say so explicitly rather than silently picking one of the two.

Question 1. Using fact 1c, compute the number of pages for each of the following row counts: 369, 370, 738, 739, 24354, 24355. For each row count, show the division you performed (row count divided by 369) and which way you rounded, then give the final page count. For the first four row counts (369, 370, 738, 739), also state whether your answer matches fact 1d.

Section 2. Question 2: how many named items fit in one journal record.

Fact 2a. A journal record is a fixed 4096 bytes in total (constant JOURNAL_RECORD_BYTES).

Fact 2b. The fixed header portion at the start of every journal record is 311 bytes (constant JOURNAL_HEADER_BYTES).

Fact 2c. One named item inside a record's payload area (the part of the record that comes after the header) is 56 bytes (constant JOURNAL_NAMED_ENTRY_BYTES).

Fact 2d. The maximum number of named items one record can hold is defined in the source as an integer division with the remainder discarded (Rust integer division on unsigned 64-bit integers always truncates toward zero, it never rounds up): (JOURNAL_RECORD_BYTES minus JOURNAL_HEADER_BYTES) divided by JOURNAL_NAMED_ENTRY_BYTES. This defines a constant named JOURNAL_NAMED_ENTRIES_PER_RECORD.

Question 2. Using facts 2a through 2d, compute JOURNAL_NAMED_ENTRIES_PER_RECORD. Show the subtraction (4096 minus 311), then the division of that result by 56, then state explicitly what the remainder of that division is and confirm that the remainder is discarded rather than rounded up, then give the final integer.

Section 3. Question 3: the single-record write-row publish.

Background for this section. In one particular kind of publish, called the write-row publish (used when the file system's tree table has zero entries), the code builds exactly one journal record, never more than one, regardless of how many named items that one record ends up needing.

Fact 3a. In this write-row publish, the list of instance-table-page roles to name is built by a function named instance_table_page_roles_in_bump_order, which, given a page count P, returns exactly P items, one role per page.

Fact 3b. In the same write-row publish, the one record's full list of named items is: those P instance-table-page-role items from fact 3a, followed by exactly one more item for the allocation-record-tree node (decision D23, settled item 17: each rewritten role gets one named item). So the total number of named items in this one record is P + 1, where P is the instance table's page count as computed by fact 1c in Section 1.

Fact 3c. This one record is unconditionally marked, in the code, as carrying ordinal-within-publish number 1 (decision D23 settled item 4: a publish with only one record carries 1) and as being the last record of its own publish (decision D23 settled item 17: being the only record makes it the last one). The code path that builds this record never calls roles_named_by_each_record_of_the_publish (the function described in Section 4 below, which is the one place in the source that splits a record's named-item list across more than one record), and never checks the named-item list's length against any per-record maximum before turning the record into bytes.

Fact 3d. Reuse your own answer to Question 2 (Section 2) as the maximum number of named items that fit in one record.

Question 3a. Using fact 3b, write the formula for the number of named items in the write-row publish's one record, as a function of P, the instance table's page count.

Question 3b. Using your Question 2 answer as the per-record maximum, and your Question 3a formula, find the smallest page count P at which the named-item count first exceeds that per-record maximum. Show the inequality you solved and how you solved it.

Question 3c. Using fact 1c (the same page-count formula from Section 1), find the smallest row count that produces the page count P you found in Question 3b.

Question 3d. A report from this project states the threshold as: page count 67 or more, row count more than 24354. Using only your own answers to Question 3b and Question 3c above, not the report's wording, state: does your computed threshold page count equal 67? Does your computed threshold row count equal 24355, the smallest row count strictly greater than 24354? Answer yes or no to each of these two questions, then give your falsification note: what number, if it came out different in your own computation, would make your answer disagree with the report's claim?

Section 4. Question 4: splitting a publish across multiple records.

Background facts, paraphrased from the doc comments of two functions named roles_named_by_each_record_of_the_publish and transaction_offset_of_each_record_of_the_publish. Decision and invariant tags are kept as opaque labels, as explained above.

Fact 4a. When a publish writes N data units where N is 2 or more, the publish has N transactions (C310, a 2026-09-16 user decision, fixed that a publish's record count equals its data-unit count). Transaction number k, for every k less than N minus 1, gets exactly one journal record, and that record names only that one data unit (decision D23, settled item 17). Which record names the roles this publish rewrites that are not data units (the roles shared by the whole publish, such as commit-generated blocks) used to be an unsettled question in the project's working notes; it was settled on 2026-09-23: those shared roles are named only in the record or records of the last transaction (transaction number N minus 1), together with that last data unit; the earlier records never repeat naming those shared roles.

Fact 4b. When a publish writes one data unit, or writes no data unit at all, the whole publish is a single transaction, and that one transaction names every rewritten role of the publish (this is the shape the project's first transaction takes today).

Fact 4c. In this prompt, let k stand for the total number of transactions in a publish, and let m stand for the number of items the last transaction (or, when k = 1, the only transaction) needs to name in total. By facts 4a and 4b: k = 1 covers both the zero-data-unit case and the one-data-unit case; for every N of 2 or more, k = N.

Fact 4d. When the last transaction's m named items do not all fit in one record, roles_named_by_each_record_of_the_publish packs them into records in a fixed order (the same bump order used elsewhere in the project), filling one record to the per-record maximum before starting the next one (decision D23 settled item 17, a 2026-09-24 user decision); every one of these extra records still belongs to the last transaction. Only the very last of these records, which is also the last record of the entire publish, is marked as the last record of the publish; every record before it is not so marked, no matter which transaction it belongs to.

Fact 4e. transaction_offset_of_each_record_of_the_publish defines which transaction number (counting from 0) each record of the publish (also counting from 0) belongs to, as follows. For k = 1: every record of the publish belongs to transaction number 0. For k = N with N of 2 or more: record index i, for every i from 0 to N minus 2, belongs to transaction number i; every record from index N minus 1 onward (the last transaction's first record and however many more records it spills into, per fact 4d) belongs to transaction number N minus 1.

Fact 4f. Reuse your own Question 2 answer (JOURNAL_NAMED_ENTRIES_PER_RECORD) as the per-record maximum used in fact 4d.

Question 4. For each of the eight scenarios below, give: (i) the total number of records the publish writes; (ii) for every record in order starting from index 0, which transaction number (counting from 0) it belongs to; (iii) which record index (counting from 0) carries the last-record-of-the-publish flag. Show your arithmetic for every scenario, in particular how many records the last transaction's m items split into, using your Question 2 answer as the per-record maximum.

Scenario 1: k = 1, m = 67.
Scenario 2: k = 1, m = 68.
Scenario 3: k = 1, m = 134.
Scenario 4: k = 1, m = 135.
Scenario 5: k = 3, m = 67.
Scenario 6: k = 3, m = 68.
Scenario 7: k = 3, m = 134.
Scenario 8: k = 3, m = 135.

Section 5. Question 5: byte offsets inside a journal record's header, and what the header checksum covers.

Fact 5a. A journal record's bytes are written in this fixed order, each field described by its name and its width in bytes: magic 4 bytes; record type 2 bytes; algorithm type 1 byte; record flags 1 byte; record length 4 bytes; named-item count 4 bytes; instance number 4 bytes; counter 6 bytes; checkpoint txg 8 bytes; nonce 12 bytes; header checksum 32 bytes; transaction number 8 bytes; commit flag 1 byte; ordinal-within-publish 4 bytes; back-chain 4 bytes; payload checksum 4 bytes; new-root segment 188 bytes; filesystem identifier 8 bytes; MAC 16 bytes. After this fixed sequence, which totals 311 bytes (matching JOURNAL_HEADER_BYTES from fact 2b), the record continues with its named items, 56 bytes each (fact 2c).

Fact 5b. Byte offsets in this prompt are counted from 0, so a field described as being at offset X occupies bytes X, X+1, and so on up through its width, and the field immediately after it starts at offset X plus that width.

Fact 5c. The record's header checksum (the 32-byte field listed in fact 5a) is computed, and later re-checked, over the byte range from offset 0 up to but not including offset 4096, that is, the entire record, including both the header and the named-items payload that follows it, with the checksum field's own 32 bytes, wherever they land per fact 5a, temporarily treated as all-zero for the purposes of that one computation only. Those 32 bytes are not excluded from the covered range; they are zeroed while still being inside it (invariant I-2.4: the checksum field counts as zero in its own computation).

Question 5a. Using fact 5a and fact 5b, compute the byte offset of each of these four fields: record flags, ordinal-within-publish, back-chain, payload checksum. Show your running cumulative total of widths up to each of these four fields.

Question 5b. Using fact 5c, for each of the four offsets you computed in Question 5a: is that offset inside the byte range covered by the header checksum? Is it inside the 32-byte span of the header-checksum field itself, the span that gets zeroed for the computation, or is it outside that 32-byte span but still inside the covered range? Answer both parts for each of the four fields.

Final instructions. Give one answer block per question and sub-question: 1, 2, 3a, 3b, 3c, 3d, 4, 5a, 5b. For question 4, give one answer block per scenario (1 through 8), each with its own parts (i), (ii), (iii) as described above. Do not add any other section, summary, or recommendation. Do not restate this prompt.
