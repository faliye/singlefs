Task: fill in numeric tables using only the quoted rule text given below. Do not use any code, do not run anything, just compute by hand from the definitions given.

Format rules for your answer:
1. Number your answers 1, 2, 3, ... matching the question numbers below.
2. Do not use any markdown emphasis at all: no bold, no italics, no heading markers (no asterisks, no pound signs used as headings). Plain sentences and plain pipe-delimited tables only.
3. After every answer, add one line starting with "Would be overturned by:" that states what observation would prove that answer wrong.
4. Do not cite any code line number or file line number. If you need to point at something, name the function or refer to a table row number given below (for example "row 3 of table B").
5. Do not use the Rust path separator (two colons) anywhere in your answer.
6. Do not use any Chinese characters or Chinese-derived variable names anywhere in your answer.
7. If you find yourself about to write a number that looks like a source-code line number or a file line number, stop and replace it with a function name or a table row number instead, per rule 4.

Background: this is about a filesystem design. All quoted rule text below is a faithful English translation of settled design-decision documents. Each quote is tagged with its source file and line number so you can tell which document it came from; these citations were checked against the source file before this prompt was written, so you may treat them as accurate. Two kinds of numeric facts are being tested: (A) two arithmetic formulas for fixed byte-capacity limits, and (B) five worked data tables about a "root ring" (a small history log of on-disk checkpoints), for which you must compute two derived numbers per table using an explicit computation rule that is given to you below (the computation rule is a plain-language restatement written for this exercise, kept separate from the verbatim quoted rule text).

Section 1: quoted rule text (verbatim translations, use these to answer "what does the rule text say" parts of the questions)

Quote Q1 (source: decisions/04-checksum-placement.md, settled item 5): "The unit is always 32768 bytes; the self-describing header, the declared length, the payload, and the padding are all within these 32768 bytes."

Quote Q2 (source: decisions/18-what-a-block-carries.md, settled item 16, table row for code 1): "Code 1 (the data unit): field offsets are 42 tag copy, 43 tree id, 51 object id, 59 object birth generation, 67 anchor offset, 75 birth generation number, 83 fsid, 91 write sequence, 101 payload CRC, 105 reserved area of width 29; therefore the plaintext header is 105 bytes and the header including the reserved area is 134 bytes."

Quote Q3 (source: decisions/18-what-a-block-carries.md, settled item 16, general clause): "The 29-byte reserved area shared by all three unit types (nonce 12 plus MAC 16 plus algorithm-type 1) is counted into the header immediately after the type-identity segment, for all three types: the code 1 header is 134, the code 3 header is 136, the code 2 header equals 86 plus 2 times the key width plus 29, which equals 115 plus 2 times the key width. The origin point of the declared length is the end of the header including the reserved area; the ceiling is 32768 minus 134 for code 1, and 32768 minus 136 for code 3."

Quote Q4 (source: decisions/08-core-index-structure.md, settled item 11): "The code 2 node header is self-describing: after a 2-byte reserved field come a 2-byte entry count and a 2-byte entry width; the 1-byte key width is placed before the key interval. The base byte count goes from 81 to 86. The header including the 29-byte reserved area is 115 plus 2 times the key width."

Quote Q5 (source: decisions/03-space-allocation.md, settled item 11): "Put the span field into the value: key equals (device identity 4 bytes, 16 KiB slot number 6 bytes) totaling 10 bytes; value equals (span field 2 bytes, allocation generation or release generation 8 bytes) totaling 10 bytes; the released flag still occupies the highest bit of the span field. The code 2 header of the allocation-record tree is computed using a key width of 10."

Quote Q6 (source: decisions/18-what-a-block-carries.md, settled item 9): "Each unit pays for one self-describing header; the scanner steps by the smallest unit size present in the pool. In the first version three unit types coexist: the index node, which is code 2, is 16 KiB; the data unit, which is code 1, and the packed-record unit, which is code 3, are each 32 KiB (decision D4 settled item 1; the unit size and alignment of the three types is governed by the registry table in settled item 11); so the scanner step size is taken to be 16384."

Quote Q7 (source: decisions/16-publish-semantics.md, settled item 1, table row "ceiling for raising F"): "Ceiling for raising F: the minimum of (the newest persistent valid root on each disk) and (the 4th newest non-empty persistent valid root); when there are fewer than 4 non-empty valid roots, take the oldest valid root instead. The target of one reclamation pass equals the minimum of (the release generation being released this time) and (the 4th newest non-empty root)."

Quote Q8 (source: decisions/16-publish-semantics.md, settled item 1, table row "oldest valid root in the ring"): "The oldest valid root in the ring: the minimum checkpoint_txg among the roots, across all root slots on disk, that are self-certifying legal and that the instance table judges still valid; a slot whose write failed is counted by its old content; an in-flight publish that has not been persisted does not count."

Quote Q9 (source: decisions/16-publish-semantics.md, settled item 1, table row "takes effect"): "Takes effect: it takes effect only once every surviving disk has a persistent root carrying the new F; the value that takes effect after recovery equals the minimum, across surviving disks, of the maximum F carried by each disk."

Quote Q10 (source: decisions/16-publish-semantics.md, settled item 1, warning paragraph attached to the "takes effect" row, dated 2026-09-17): "Warning: the user sent the 'takes effect' row back on 2026-09-17 for renewed discussion, calling it a design question rather than a code question: F lives only in the root record, and root slots are not mirrored, so once raising F takes effect each surviving disk may have only a single persistent root carrying the new F. Blocks reclaimed after it takes effect are legitimately reused; if that one root later has a single byte go bad, then under 'the value that takes effect after recovery equals the minimum, across surviving disks, of the maximum F carried by each disk', F falls back to its old value, the new instance writes the fallen-back value to disk, and earlier roots re-enter the rollback candidate set even though the blocks they reference have already been reused. What is being redebated includes: what keeps F from falling back once it has taken effect; which F a later consistency check should use; whether F carried by a root on an abandoned timeline counts toward this computation; and what to do when an empty publish issued to raise F cannot be assigned to a fixed point. Before this question is settled, the implementation follows the literal wording of the 'takes effect' row only as an anticipated, interim reading."

Quote Q11 (source: decisions/16-publish-semantics.md, settled item 1, warning paragraph attached to the "non-empty" definition, dated 2026-09-17): "A valid root counts as non-empty if and only if the root pointers of the user-visible trees, meaning the inode tree and the extent tree, in its tree table differ from those in its immediately preceding valid root, ordered by txg; here valid means judged still valid by the instance table AND txg is greater than or equal to the current F."

Quote Q12 (source: implementer report research/prompts/m2-supp3-item2-implementer-report.md, table row about F effective, marked "anticipated" in the report): "F effective equals the minimum, across disks, of the maximum F carried by that disk; a root on an abandoned timeline still counts toward this maximum. This is an anticipated reading, tied to open item 2 in the closing table."


Section 2: table A, data unit content capacity. Fill in every blank cell using quotes Q1, Q2, Q3.

Table A
row 1 | quantity: total unit size in bytes | value: 32768 | source: Q1
row 2 | quantity: plaintext header size for a code 1 (data) unit, in bytes | value: 105 | source: Q2
row 3 | quantity: reserved area size shared by all unit types, in bytes | value: 29 | source: Q3
row 4 | quantity: header size for a code 1 unit including the reserved area, in bytes | value: BLANK, you compute this as row 2 plus row 3 | source: Q3 (states this total directly as 134; check your sum against it)
row 5 | quantity: maximum content (payload) length that a code 1 unit can hold, in bytes | value: BLANK, you compute this as row 1 minus row 2 minus row 3 | source: derived
row 6 | quantity: is content of exactly the row 5 length accepted | value: BLANK, answer yes or no | source: derived
row 7 | quantity: is content of one byte more than row 5 accepted | value: BLANK, answer yes or no | source: derived

Section 3: table B, allocation-record tree node capacity. Fill in every blank cell using quotes Q3, Q4, Q5, Q6.

Table B
row 1 | quantity: index node (code 2) unit size in bytes | value: 16384 | source: Q6
row 2 | quantity: key width for the allocation-record tree, in bytes | value: 10 | source: Q5
row 3 | quantity: entry width for the allocation-record tree, in bytes | value: 20 (10 bytes key plus 10 bytes value, per Q5) | source: Q5
row 4 | quantity: plaintext code 2 header size in bytes, using row 2's key width, from the formula "86 plus 2 times key width" | value: BLANK, you compute this | source: Q4
row 5 | quantity: code 2 header size in bytes including the reserved area, using row 2's key width, from the formula "115 plus 2 times key width" | value: BLANK, you compute this | source: Q3, Q4
row 6 | quantity: bytes available for entries in one node, computed as row 1 minus row 5 | value: BLANK, you compute this | source: derived
row 7 | quantity: maximum number of entries that fit in one node, computed as row 6 divided by row 3, rounded down to a whole number | value: BLANK, you compute this | source: derived

Section 4: computation rules for the root-ring tables (these are plain-language restatements written for this exercise; they are not verbatim quotes, keep them separate from Section 1 in your answer)

Each root-ring table below lists, for two disks named disk A and disk B, a series of root records. Each record has four fields: txg (an increasing checkpoint number), instance (which instance, meaning which timeline, wrote this root), carried F (the value of F that this particular root carries), and non-empty (given directly as yes or no; you do not need to derive it).

Rule V (validity): first find the highest instance number that appears anywhere in the table (across both disks). A root is valid if and only if its instance number equals that highest instance number. A root whose instance number is lower belongs to an abandoned timeline and is not valid. Every record listed in these tables has already passed self-certification (checksums good); the only thing that can make a record invalid here is belonging to an abandoned instance.

Rule X (per-disk newest valid root): for each disk, find that disk's own valid records (using Rule V) and take the one with the highest txg. X is the minimum of these two per-disk values (the smaller of disk A's newest valid txg and disk B's newest valid txg). If a disk has no valid record at all, that disk is excluded from the minimum.

Rule P (pooled non-empty valid states): take the union, across both disks, of all valid (Rule V) records whose non-empty field is yes; if the same txg appears as a valid non-empty record on both disks, count it once. Sort this pooled set by txg from newest to oldest.

Rule Y (4th newest non-empty valid root, with fallback): if Rule P's pooled set has 4 or more entries, Y is the txg of the 4th entry from the top (the newest is 1st, next is 2nd, and so on). If Rule P's pooled set has fewer than 4 entries, Y is instead the oldest valid root, defined as the minimum txg among all valid (Rule V) records pooled across both disks, regardless of their non-empty field.

Rule C (ceiling for raising F): C is the minimum of X (Rule X) and Y (Rule Y).

Rule F (F effective): for each disk, look at every record listed for that disk, valid or not (an abandoned-instance record still counts here, unlike in Rule V), and take the highest carried-F value among them. F effective is the minimum of these two per-disk maximums.


Section 5: the five root-ring tables

Table 1 (baseline, single instance, both disks in sync)
disk A: txg 1 instance 1 carried-F 0 non-empty yes
disk A: txg 2 instance 1 carried-F 0 non-empty yes
disk A: txg 3 instance 1 carried-F 0 non-empty no
disk A: txg 4 instance 1 carried-F 0 non-empty yes
disk A: txg 5 instance 1 carried-F 0 non-empty yes
disk A: txg 6 instance 1 carried-F 0 non-empty yes
disk A: txg 7 instance 1 carried-F 0 non-empty yes
disk B: txg 1 instance 1 carried-F 0 non-empty yes
disk B: txg 2 instance 1 carried-F 0 non-empty yes
disk B: txg 3 instance 1 carried-F 0 non-empty no
disk B: txg 4 instance 1 carried-F 0 non-empty yes
disk B: txg 5 instance 1 carried-F 0 non-empty yes
disk B: txg 6 instance 1 carried-F 0 non-empty yes
disk B: txg 7 instance 1 carried-F 0 non-empty yes

Table 2 (single instance, both disks in sync, fewer than 4 non-empty records)
disk A: txg 1 instance 1 carried-F 0 non-empty yes
disk A: txg 2 instance 1 carried-F 0 non-empty no
disk A: txg 3 instance 1 carried-F 0 non-empty no
disk A: txg 4 instance 1 carried-F 0 non-empty yes
disk A: txg 5 instance 1 carried-F 0 non-empty no
disk A: txg 6 instance 1 carried-F 0 non-empty yes
disk B: txg 1 instance 1 carried-F 0 non-empty yes
disk B: txg 2 instance 1 carried-F 0 non-empty no
disk B: txg 3 instance 1 carried-F 0 non-empty no
disk B: txg 4 instance 1 carried-F 0 non-empty yes
disk B: txg 5 instance 1 carried-F 0 non-empty no
disk B: txg 6 instance 1 carried-F 0 non-empty yes

Table 3 (single instance, disk B fell behind and stopped receiving publishes after txg 3)
disk A: txg 1 instance 1 carried-F 0 non-empty yes
disk A: txg 2 instance 1 carried-F 0 non-empty yes
disk A: txg 3 instance 1 carried-F 0 non-empty yes
disk A: txg 4 instance 1 carried-F 0 non-empty yes
disk A: txg 5 instance 1 carried-F 0 non-empty yes
disk A: txg 6 instance 1 carried-F 0 non-empty yes
disk A: txg 7 instance 1 carried-F 0 non-empty yes
disk A: txg 8 instance 1 carried-F 0 non-empty yes
disk B: txg 1 instance 1 carried-F 0 non-empty yes
disk B: txg 2 instance 1 carried-F 0 non-empty yes
disk B: txg 3 instance 1 carried-F 0 non-empty yes

Table 4 (single instance, both disks in sync, several consecutive empty publishes at the newest end)
disk A: txg 1 instance 1 carried-F 0 non-empty yes
disk A: txg 2 instance 1 carried-F 0 non-empty yes
disk A: txg 3 instance 1 carried-F 0 non-empty yes
disk A: txg 4 instance 1 carried-F 0 non-empty yes
disk A: txg 5 instance 1 carried-F 0 non-empty no
disk A: txg 6 instance 1 carried-F 0 non-empty no
disk A: txg 7 instance 1 carried-F 0 non-empty no
disk A: txg 8 instance 1 carried-F 0 non-empty no
disk A: txg 9 instance 1 carried-F 0 non-empty no
disk A: txg 10 instance 1 carried-F 0 non-empty no
disk B: txg 1 instance 1 carried-F 0 non-empty yes
disk B: txg 2 instance 1 carried-F 0 non-empty yes
disk B: txg 3 instance 1 carried-F 0 non-empty yes
disk B: txg 4 instance 1 carried-F 0 non-empty yes
disk B: txg 5 instance 1 carried-F 0 non-empty no
disk B: txg 6 instance 1 carried-F 0 non-empty no
disk B: txg 7 instance 1 carried-F 0 non-empty no
disk B: txg 8 instance 1 carried-F 0 non-empty no
disk B: txg 9 instance 1 carried-F 0 non-empty no
disk B: txg 10 instance 1 carried-F 0 non-empty no

Table 5 (F was raised, then a rollback created a new instance; disk B lost the roots that carried the raised F)
disk A: txg 1 instance 1 carried-F 0 non-empty yes
disk A: txg 2 instance 1 carried-F 0 non-empty yes
disk A: txg 3 instance 1 carried-F 0 non-empty yes
disk A: txg 4 instance 1 carried-F 0 non-empty yes
disk A: txg 5 instance 1 carried-F 0 non-empty yes
disk A: txg 6 instance 1 carried-F 5 non-empty yes
disk A: txg 7 instance 1 carried-F 5 non-empty yes
disk A: txg 8 instance 1 carried-F 5 non-empty yes
disk A: txg 9 instance 2 carried-F 0 non-empty yes
disk B: txg 1 instance 1 carried-F 0 non-empty yes
disk B: txg 2 instance 1 carried-F 0 non-empty yes
disk B: txg 3 instance 1 carried-F 0 non-empty yes
disk B: txg 4 instance 1 carried-F 0 non-empty yes
disk B: txg 5 instance 1 carried-F 0 non-empty yes
disk B: txg 9 instance 2 carried-F 0 non-empty yes


Section 6: questions, answer every one, numbered 1 to 7

Question 1: Fill in rows 4, 5, 6, 7 of table A. Show the arithmetic for each (which two or three numbers you added or subtracted). State whether your row 4 answer matches the 134 stated directly in Q3.

Question 2: Fill in rows 4, 5, 6, 7 of table B, using key width equal to 10 from row 2 of table B. Show the arithmetic for each row, including the division in row 7 and what the exact remainder is before you round down.

Question 3: Using table 1 and the rules in section 4, state: the highest instance number found, which records (by disk and txg) are valid under Rule V, the value of X under Rule X, the full pooled sorted list under Rule P, the value of Y under Rule Y (state whether you used the 4th-item branch or the fallback branch), the value of C under Rule C, and the value of F effective under Rule F. For each of X, Y, C, and F effective, name which quote from section 1 (Q7, Q8, Q9, Q10, Q11, or Q12) is the closest match to the rule you just applied.

Question 4: Do the same seven things as question 3, but for table 2.

Question 5: Do the same seven things as question 3, but for table 3.

Question 6: Do the same seven things as question 3, but for table 4.

Question 7: Do the same seven things as question 3, but for table 5. In addition, state explicitly: are the disk A records at txg 6, 7, and 8 (instance 1, carried F 5) included when you compute disk A's maximum carried F for Rule F, even though they are not valid under Rule V. Name the quote (Q9, Q10, or Q12) that tells you whether an abandoned-instance record should or should not be included in that maximum.

End of prompt.
