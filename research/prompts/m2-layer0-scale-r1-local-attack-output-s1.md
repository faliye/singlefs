1  
Position w_i k_i c_i  
1 2 2 0  
2 2 2 0  
3 1 1 0  
4 2 2 0  
5 2 2 0  
6 1 1 0  
7 26 2 24  
8 2 2 0  
9 1 1 0  
10 26 2 24  
11 2 2 0  
12 1 1 0  
13 4 4 0  
14 18 0 18  
15 2 2 0  
16 1 1 0  
17 18 2 16  
18 2 2 0  
19 1 1 0  
20 18 2 16  
21 2 2 0  
22 1 1 0  
23 26 2 24  
24 2 2 0  
25 1 1 0  
26 4 4 0  
27 22 0 22  
28 2 2 0  
29 1 1 0  
30 18 2 16  
31 2 2 0  
32 1 1 0  
33 30 2 28  
34 2 2 0  
35 1 1 0  
36 30 2 28  
37 2 2 0  
38 1 1 0  
39 30 2 28  
40 2 2 0  
41 1 1 0  
42 30 2 28  
43 2 2 0  
44 1 1 0  
45 18 2 16  
46 2 2 0  
47 1 1 0  
48 18 2 16  
49 2 2 0  
50 1 1 0  
51 30 2 28  
52 2 2 0  
53 1 1 0  
54 2 2 0  
Sum w_i: 423  
Sum k_i: 91  
Sum c_i: 332  
Falsified by: sum of w_i does not equal 423  

2  
The formula for segment i's contribution under candidate A is (2^{k_i} - 1). This is because COW writes (c_i) do not contribute to distinct crash states; any subset of COW writes results in the same state as none persisting. Only in-place overwrite writes (k_i) affect the count, and the number of distinct subsets of k_i writes is 2^{k_i}. The baseline formula subtracts 1 per segment (Table 2 Row 4), so candidate A uses (2^{k_i} - 1). The empty subset of in-place writes (k_i's subset empty) is included in this count: for k_i=0, it contributes 0 (2^0 - 1 = 0), which correctly indicates no variation from COW writes.  
Falsified by: a segment with k_i=2 contributing a value other than 3, or k_i=1 contributing a value other than 1  

3  
Per-segment contributions:  
Position 1: 3  
Position 2: 3  
Position 3: 1  
Position 4: 3  
Position 5: 3  
Position 6: 1  
Position 54: 3  
Grand total: 147  
Falsified by: grand total not equal to 147  

4  
The formula reduces to the baseline when c_i=0 (k_i=w_i for all i). Applying the formula to the actual w_i values in Table 4 Row 1, sum(2^{k_i} - 1) + 1 equals 5575802973, matching Table 4 Row 3. This holds because sum(2^{k_i}) when k_i=w_i is sum(2^w_i), and 1 + sum(2^w_i - 1) = sum(2^w_i) - 53 = 5575802973.  
Falsified by: sum(2^w_i) - 53 not equal to 5575802973
