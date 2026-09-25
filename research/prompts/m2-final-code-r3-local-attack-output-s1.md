1  
16384 minus 135 equals 16249  
16249 divided by 20 equals 812.45  
rounded down to 812  
This would be refuted by: using a header width different from fact 2 or not flooring the division result.  

2  
16384 minus 135 equals 16249  
16249 divided by 96 equals 169.26041666666666  
rounded down to 169  
this equals 169  
This would be refuted by: using a header width different from fact 2 or a leaf entry width different from fact 5.  

3  
preparation:  
configuration a: total slot count 240, unit area slot count 240  
configuration b: device byte size 4294967296, 4294967296 divided by 16384 equals 262144, unit area slot count 262144 minus 50176 equals 211968, both devices same count  
configuration c: device byte size 1099511627776, 1099511627776 divided by 16384 equals 67108864, unit area slot count 67108864 minus 50176 equals 67058688, both devices same count  
main part:  
configuration a:  
r=1: 812 times 169 to the 0 equals 812, ceil(240/812) equals 1, sum equals 1 <=169, r=1  
configuration b total slot count:  
r=1: 812 times 169 to the 0 equals 812, ceil(262144/812) equals 323, sum equals 646 >169  
r=2: 812 times 169 equals 137228, ceil(262144/137228) equals 2, sum equals 4 <=169, r=2  
configuration b unit area slot count:  
r=1: 812 times 169 to the 0 equals 812, ceil(211968/812) equals 261, sum equals 522 >169  
r=2: 812 times 169 equals 137228, ceil(211968/137228) equals 2, sum equals 4 <=169, r=2  
configuration c total slot count:  
r=1: 812 times 169 to the 0 equals 812, ceil(67108864/812) equals 82647, sum equals 165294 >169  
r=2: 812 times 169 equals 137228, ceil(67108864/137228) equals 490, sum equals 980 >169  
r=3: 812 times 169 squared equals 23191532, ceil(67108864/23191532) equals 3, sum equals 6 <=169, r=3  
configuration c unit area slot count:  
r=1: 812 times 169 to the 0 equals 812, ceil(67058688/812) equals 82600, sum equals 165200 >169  
r=2: 812 times 169 equals 137228, ceil(67058688/137228) equals 489, sum equals 978 >169  
r=3: 812 times 169 squared equals 23191532, ceil(67058688/23191532) equals 3, sum equals 6 <=169, r=3  
configuration b: same r=2  
configuration c: same r=3  
This would be refuted by: using an incorrect w from item 1 or misapplying the ceiling division in the rule.  

4  
(a) 16384 minus 163 equals 16221, 16221 divided by 113 equals 143.54867256637168, rounded down to 143  
(b) 16384 minus 163 equals 16221, 16221 divided by 112 equals 144.83035714285714, rounded down to 144  
(c) 16384 minus 163 equals 16221, 16221 divided by 110 equals 147.46363636363637, rounded down to 147  
This would be refuted by: using a header width different from fact 3 or entry widths from incorrect facts.  

5  
span at level 0: 144  
span at level 1: 144 times 147 equals 21168  
root level is 1, height is 2  
This would be refuted by: using incorrect leaf capacity or internal fanout from item 4 or miscalculating span levels.
