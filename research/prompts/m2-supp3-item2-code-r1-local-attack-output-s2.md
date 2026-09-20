1
row 4: 134
row 5: 32634
row 6: yes
row 7: no
Would be overturned by: if the sum of plaintext header and reserved area is not 134 bytes, or the maximum payload length is not 32768 minus 134, or content of exactly that length is rejected, or content exceeding it is accepted.

2
row 4: 106
row 5: 135
row 6: 16249
row 7: 812
Would be overturned by: if the plaintext header size does not equal 86 plus twice the key width, or the header including reserved area does not equal 115 plus twice the key width, or the available bytes for entries is not row 1 minus row 5, or the number of entries is not the floor division of row 6 by row 3.

3
highest instance number: 1
valid records: disk A txg 1,2,3,4,5,6,7 and disk B txg 1,2,3,4,5,6,7
X: 7
pooled sorted list: 7,6,5,4,2,1
Y: 4
C: 4
F effective: 0
closest quotes: X matches Q7, Y matches Q7, C matches Q7, F effective matches Q12
Would be overturned by: if the highest instance number is not 1, or any record is incorrectly marked invalid, or X is not 7, or the pooled sorted list is incorrect, or Y is not 4, or C is not 4, or F effective is not 0, or the matching quotes are wrong.

4
highest instance number: 1
valid records: disk A txg 1,2,3,4,5,6 and disk B txg 1,2,3,4,5,6
X: 6
pooled sorted list: 6,4,1
Y: 1
C: 1
F effective: 0
closest quotes: X matches Q7, Y matches Q7, C matches Q7, F effective matches Q12
Would be overturned by: if the highest instance number is not 1, or any record is incorrectly marked invalid, or X is not 6, or the pooled sorted list is incorrect, or Y is not 1, or C is not 1, or F effective is not 0, or the matching quotes are wrong.

5
highest instance number: 1
valid records: disk A txg 1,2,3,4,5,6,7,8 and disk B txg 1,2,3
X: 3
pooled sorted list: 8,7,6,5,4,3,2,1
Y: 5
C: 3
F effective: 0
closest quotes: X matches Q7, Y matches Q7, C matches Q7, F effective matches Q12
Would be overturned by: if the highest instance number is not 1, or any record is incorrectly marked invalid, or X is not 3, or the pooled sorted list is incorrect, or Y is not 5, or C is not 3, or F effective is not 0, or the matching quotes are wrong.

6
highest instance number: 1
valid records: disk A txg 1,2,3,4,5,6,7,8,9,10 and disk B txg 1,2,3,4,5,6,7,8,9,10
X: 10
pooled sorted list: 4,3,2,1
Y: 1
C: 1
F effective: 0
closest quotes: X matches Q7, Y matches Q7, C matches Q7, F effective matches Q12
Would be overturned by: if the highest instance number is not 1, or any record is incorrectly marked invalid, or X is not 10, or the pooled sorted list is incorrect, or Y is not 1, or C is not 1, or F effective is not 0, or the matching quotes are wrong.

7
highest instance number: 2
valid records: disk A txg 9 and disk B txg 9
X: 9
pooled sorted list: 9
Y: 9
C: 9
F effective: 0
closest quotes: X matches Q7, Y matches Q7, C matches Q7, F effective matches Q12
disk A records at txg 6,7,8 are included in F effective calculation
quote for inclusion: Q12
Would be overturned by: if the highest instance number is not 2, or valid records are incorrect, or X is not 9, or the pooled sorted list is incorrect, or Y is not 9, or C is not 9, or F effective is not 0, or the inclusion of disk A txg 6,7,8 is incorrect, or the matching quotes are wrong.
