1
3932160
3932160
3932160
3932160
3932160
3932160
3932160
3932160

2
1327104
1327104
1327104
1556480
1556480
1556480
1556480
1556480

3
0
0
0
0
0
0
0
0

4
1064960
1294336
0
1294336
1523712
0
1294336
1523712
0
1294336
1523712
0

5
1114112
1114112
1114112
1114112
1114112
1114112
1114112
1114112

6
0
0
0
0
0
0
0
0

7
0
0
0
0
0
0
0
0

8
0
0
0
0
0
0
0
0

9
81920
81920
81920
81920
81920
81920
81920
81920

10
344064
114688
1409024
-114688
-344064
1179648
-114688
-344064
1179648
-114688
-344064
1179648

11
229376
229376
229376
229376
229376
229376
229376
229376

12
yes +114688
no -114688
yes +1179648
no -344064
no -573440
yes +950272
no -344064
no -573440
yes +950272
no -344064
no -573440
yes +950272

For checkpoint P, candidate A1 flips the outcome from admitted to refused. For checkpoint Q, candidate A2 flips the outcome from refused to admitted. Candidate A1 causes available(d) to cross from positive to negative at checkpoint P compared to today's code. Candidate A2 causes available(d) to cross from negative to positive at checkpoint Q compared to today's code.

This would be refuted by a measured value of the superseded blocks for the fifth overwrite being different from 14 slots.

1
still in defer window
still in defer window
still in defer window
still in defer window
still in defer window
still in defer window
still in defer window
still in defer window
still in defer window
reclaimable
reclaimable
reclaimable

2
50
50
50
50
50
50
50
50
50
30
30
30

3
20
20
0
20
20
0
20
20
0
0
0
0

4
-20
-20
0
-20
-20
0
-20
-20
0
20
20
20

5
20
20
20
20
20
20
20
20
20
20
20
20

6
no -40
no -40
no -20
no -40
no -40
no -20
no -40
no -40
no -20
yes 0
yes 0
yes 0

The retried write is first admitted at publish 4 for today's code, candidate A1, and candidate A2. Candidate A1 has no effect because the retried write does not supersede existing content, so Fact 11's rule does not apply.

This would be refuted by evidence that the retried write does supersede existing content.