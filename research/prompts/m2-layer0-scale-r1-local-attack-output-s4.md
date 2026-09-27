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
Sum of w_i: 423
Sum of k_i: 91
Sum of c_i: 332
Falsified by: sum of w_i not equal to 423, or sum of k_i + sum of c_i not equal to sum of w_i

2
Each segment contributes (2^{k_i} - 1) states. COW writes do not affect the recovered state, so only subsets of in-place overwrite writes matter. The number of distinct states for a segment is determined solely by the in-place overwrite writes, as any subset of COW writes results in the same state as none persisting. Thus, the contribution is based on the number of subsets of IPO writes (2^{k_i}), but the baseline formula subtracts 1 per segment to avoid double-counting the empty subset across segments. The empty IPO subset is included in the overall +1 term of the total formula.
Falsified by: a segment with k_i=0 contributing more than 0, or a segment with k_i>0 having a contribution not equal to (2^{k_i} - 1)

3
First 6 segments: 3, 3, 1, 3, 3, 1
Last segment (position 54): 3
Grand total: 145
Falsified by: grand total not equal to 145

4
The formula reduces to the baseline when c_i=0 for all segments. sum(2^{k_i}) - 53 equals the baseline value 5575802973, as k_i=w_i in this case. sum(2^{w_i}) - 53 = 5575802973, which matches Table 4 Row 3.
Falsified by: sum(2^{w_i}) - 53 not equal to 5575802973
