1  
16384 - 135 = 16249  
16249 / 20 = 812.45  
812  
This would be refuted by not subtracting fact 2's 135 bytes from fact 1's 16384 bytes before division.  

2  
16384 - 135 = 16249  
16249 / 96 = 169.260416666...  
169  
The result equals 169.  
This would be refuted by miscalculating the division as 16249 / 96 = 170 or not rounding down correctly.  

3  
Preparation:  
Config A: total slot count 240, unit area slot count 240  
Config B:  
Byte size 4294967296  
Total slot count: 4294967296 / 16384 = 262144  
Unit area slot count: 262144 - 50176 = 211968  
Both devices same.  
Config C:  
Byte size 1099511627776  
Total slot count: 1099511627776 / 16384 = 67108864  
Unit area slot count: 67108864 - 50176 = 67058688  
Both devices same.  
Main:  
Config A:  
R=1: 812 * 1 = 812  
ceiling(240 / 812) = 1  
Sum=1 <=169 → R=1  
Config B total:  
R=1: ceiling(262144 / 812)=323, sum=646>169  
R=2: 812*169=137228, ceiling(262144/137228)=2, sum=4<=169 → R=2  
Config B unit area:  
R=1: ceiling(211968/812)=261, sum=522>169  
R=2: ceiling(211968/137228)=2, sum=4<=169 → R=2  
Config C total:  
R=1: ceiling(67108864/812)=82647, sum=165294>169  
R=2: ceiling(67108864/137228)=490, sum=980>169  
R=3: 812*169^2=23191532, ceiling(67108864/23191532)=3, sum=6<=169 → R=3  
Config C unit area:  
R=3: same as total → R=3  
Config B: same R=2 for both ways.  
Config C: same R=3 for both ways.  
This would be refuted by using the wrong slot count (e.g., unit area instead of total) for Config B or C in the rule application.  

4  
(a) 16384 - 163 = 16221  
16221 / 113 = 143.548... → 143  
(b) 16221 / 112 = 144.830... → 144  
(c) 16221 / 110 = 147.463... → 147  
This would be refuted by using the wrong header width (fact 3) or entry widths (facts 6,7,8) in the division.  

5  
Lower-segment leaf capacity=144, internal fanout=147  
Span level 0: 144  
Span level 1: 144 * 147 = 21168  
144 < 145 → check level 1: 21168 >=145 → root level=1, height=2  
This would be refuted by using incorrect leaf capacity or fanout values or miscalculating span multiplication.
