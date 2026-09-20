1.
quantity: header size for a code 1 unit including the reserved area, in bytes | value: 134
quantity: maximum content (payload) length that a code 1 unit can hold, in bytes | value: 32634
quantity: is content of exactly the row 5 length accepted | value: yes
quantity: is content of one byte more than row 5 accepted | value: no
Would be overturned by: if the sum of plaintext header (105) and reserved area (29) is not 134; if the payload length is not 32768 minus 134; if exactly 32634 bytes of content is rejected; or if 32635 bytes of content is accepted.

2.
quantity: plaintext code 2 header size in bytes, using row 2's key width, from the formula "86 plus 2 times key width" | value: 106
quantity: code 2 header size in bytes including the reserved area, using row 2's key width, from the formula "115 plus 2 times key width" | value: 135
quantity: bytes available for entries in one node, computed as row 1 minus row 5 | value: 16249
quantity: maximum number of entries that fit in one node, computed as row 6 divided by row 3, rounded down to a whole number | value: 812
Would be overturned by: if 86 + 2*10 is not 106; if 115 + 2*10 is not 135; if 16384 - 135 is not 16249; or if 16249 divided by 20 rounded down is not 812 with remainder 9.

3.
quantity: highest instance number found | value: 1
quantity: valid records (disk and txg) | value: disk A txg 1-7; disk B txg 1-7
quantity: value of X under Rule X | value: 7
quantity: full pooled sorted list under Rule P | value: txg 7,6,5,4,2,1
quantity: value of Y under Rule Y | value: 4 (4th newest non-empty valid root)
quantity: value of C under Rule C | value: 4
quantity: value of F effective under Rule F | value: 0
quantity: closest quote for X | value: Q7
quantity: closest quote for Y | value: Q7
quantity: closest quote for C | value: Q7
quantity: closest quote for F effective | value: Q9
Would be overturned by: if highest instance is not 1; if any instance 1 record is invalid; if min of disk A and B newest valid txg is not 7; if pooled non-empty sorted list is not [7,6,5,4,2,1]; if Y is not 4; if C is not 4; if F effective is not 0; if closest quotes are incorrect.

4.
quantity: highest instance number found | value: 1
quantity: valid records (disk and txg) | value: disk A txg 1-6; disk B txg 1-6
quantity: value of X under Rule X | value: 6
quantity: full pooled sorted list under Rule P | value: txg 6,4,1
quantity: value of Y under Rule Y | value: 1 (fallback branch)
quantity: value of C under Rule C | value: 1
quantity: value of F effective under Rule F | value: 0
quantity: closest quote for X | value: Q7
quantity: closest quote for Y | value: Q7
quantity: closest quote for C | value: Q7
quantity: closest quote for F effective | value: Q9
Would be overturned by: if highest instance is not 1; if any instance 1 record is invalid; if X is not 6; if pooled non-empty sorted list is not [6,4,1]; if Y is not 1; if C is not 1; if F effective is not 0; if closest quotes are incorrect.

5.
quantity: highest instance number found | value: 1
quantity: valid records (disk and txg) | value: disk A txg 1-8; disk B txg 1-3
quantity: value of X under Rule X | value: 3
quantity: full pooled sorted list under Rule P | value: txg 8,7,6,5,4,3,2,1
quantity: value of Y under Rule Y | value: 5 (4th newest non-empty valid root)
quantity: value of C under Rule C | value: 3
quantity: value of F effective under Rule F | value: 0
quantity: closest quote for X | value: Q7
quantity: closest quote for Y | value: Q7
quantity: closest quote for C | value: Q7
quantity: closest quote for F effective | value: Q9
Would be overturned by: if highest instance is not 1; if any instance 1 record is invalid; if X is not 3; if pooled non-empty sorted list is not [8,7,6,5,4,3,2,1]; if Y is not 5; if C is not 3; if F effective is not 0; if closest quotes are incorrect.

6.
quantity: highest instance number found | value: 1
quantity: valid records (disk and txg) | value: disk A txg 1-10; disk B txg 1-10
quantity: value of X under Rule X | value: 4
quantity: full pooled sorted list under Rule P | value: txg 4,3,2,1
quantity: value of Y under Rule Y | value: 1 (fallback branch)
quantity: value of C under Rule C | value: 1
quantity: value of F effective under Rule F | value: 0
quantity: closest quote for X | value: Q7
quantity: closest quote for Y | value: Q7
quantity: closest quote for C | value: Q7
quantity: closest quote for F effective | value: Q9
Would be overturned by: if highest instance is not 1; if any instance 1 record is invalid; if X is not 4; if pooled non-empty sorted list is not [4,3,2,1]; if Y is not 1; if C is not 1; if F effective is not 0; if closest quotes are incorrect.

7.
quantity: highest instance number found | value: 2
quantity: valid records (disk and txg) | value: disk A txg 9; disk B txg 9
quantity: value of X under Rule X | value: 9
quantity: full pooled sorted list under Rule P | value: txg 9
quantity: value of Y under Rule Y | value: 9 (fallback branch)
quantity: value of C under Rule C | value: 9
quantity: value of F effective under Rule F | value: 5
quantity: closest quote for X | value: Q7
quantity: closest quote for Y | value: Q7
quantity: closest quote for C | value: Q7
quantity: closest quote for F effective | value: Q12
quantity: are disk A txg 6-8 included in F effective? | value: yes
Would be overturned by: if highest instance is not 2; if valid records are not disk A txg 9 and disk B txg 9; if X is not 9; if pooled non-empty sorted list is not [9]; if Y is not 9; if C is not 9; if F effective is not 5; if closest quotes are incorrect; if abandoned-instance records are excluded from F effective.
