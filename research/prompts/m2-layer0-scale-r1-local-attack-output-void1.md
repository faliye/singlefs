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
sum w_i: 423
sum k_i: 91
sum c_i: 332
Falsified by: any position where c_i != w_i - k_i or sum w_i != 423

2
Under candidate A, only in-place overwrite writes (k_i) affect the crash state count because COW writes (c_i) do not change the recovered state regardless of persistence. For a segment, the number of distinct crash states it contributes is (2^k_i - 1). This is because the empty subset of k_i writes (no in-place writes persisted) corresponds to the crash occurring before the segment (counted in the global +1 term), so only non-empty subsets of k_i writes contribute. The COW writes' subsets are irrelevant and thus do not add new states.
Falsified by: a segment where k_i=0 contributes more than 0 or k_i=1 contributes more than 1

3
Per-segment contributions (2^k_i - 1):
Position 1: 3
Position 2: 3
Position 3: 1
Position 4: 3
Position 5: 3
Position 6: 1
Position 54: 3
Sum of all segment contributions: 146
Total crash states under candidate A: 1 + 146 = 147
Falsified by: any segment's contribution not matching (2^k_i - 1) or grand total != 147

4
When c_i=0 for all segments (k_i=w_i), the formula from Question 2 reduces to 1 + sum(2^w_i - 1), which is identical to the baseline closed-form formula. Applied to Table 4 Row 1's w_i values, this reproduces the value 5575802973 exactly as given in Table 4 Row 3.
Falsified by: 1 + sum(2^w_i - 1) for Table 4 Row 1's w_i not equaling 5575802973