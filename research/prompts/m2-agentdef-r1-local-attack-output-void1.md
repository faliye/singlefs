1 P1 C1 GREEN not a heading line if it were a heading line containing LISHI or BIANGENGSHI
2 P1 C2 GREEN DATE outside quotes but line contains path to records file DATE outside quotes and no path to records or .claude/kb
3 P1 C3 GREEN not the first line of the paragraph if it were the first line of the paragraph
4 P1 C4 GREEN no trigger punctuation followed by group one or two words after a trigger punctuation a group one or two word immediately follows
5 P2 C1 GREEN not a heading line
6 P2 C2 GREEN DATE outside quotes but path exists
7 P2 C3 GREEN line-start is ordinary text
8 P2 C4 GREEN after SEMI WEISHENME followed by non-COLON/WS text
9 P3 C1 GREEN not a heading line
10 P3 C2 GREEN no DATE outside quotes
11 P3 C3 GREEN line-start is ordinary text
12 P3 C4 GREEN no trigger punctuation followed by group words
13 P4 C1 GREEN not a heading line
14 P4 C2 GREEN DATE inside quotes
15 P4 C3 GREEN not first line
16 P4 C4 GREEN after LPAREN DATE followed by text
17 P5 C1 GREEN not a heading line
18 P5 C2 GREEN no DATE outside quotes
19 P5 C3 GREEN line-start is ordinary text
20 P5 C4 GREEN no trigger punctuation followed by group words
21 P6 C1 GREEN not a heading line
22 P6 C2 GREEN no DATE outside quotes
23 P6 C3 GREEN line-start is ordinary text
24 P6 C4 GREEN no trigger punctuation followed by group words
25 P7 C1 GREEN not a heading line
26 P7 C2 GREEN no DATE outside quotes
27 P7 C3 GREEN line-start is ordinary text
28 P7 C4 GREEN after DUN YIJU followed by LPAREN
29 P8 C1 GREEN not a heading line
30 P8 C2 RED DATE outside quotes and no path to records or .claude/kb path to records or .claude/kb exists
31 P8 C3 RED line starts with SHICE followed immediately by LPAREN SHICE followed by non-connector character
32 P8 C4 GREEN no trigger punctuation followed by group words
33 P9 C1 GREEN not a heading line
34 P9 C2 RED DATE outside quotes and no path to records or .claude/kb path to records or .claude/kb exists
35 P9 C3 GREEN line-start is ordinary text
36 P9 C4 RED after LPAREN DATE WS SHICE SHICE not present there or gap had non-WS/DATE
37 P10 C1 GREEN not a heading line
38 P10 C2 RED DATE outside quotes and no path to records or .claude/kb path to records or .claude/kb exists
39 P10 C3 GREEN line-start is ordinary text
40 P10 C4 RED after COMMA DATE WS SHICE SHICE not present there or gap had non-WS/DATE
41 C1 heading line "Historical Overview" literal GREEN intended RED C1 only checks specific tags but heading is explanatory if heading contains LISHI
42 C2 line "Complete by 2026-09-18" literal RED intended GREEN DATE is part of constraint path to records exists
43 C3 first line starts with "For example" literal GREEN intended RED explanatory prose not caught by triggers line starts with trigger word followed by connector
44 C4 line ", SHICE: record" literal RED intended GREEN SHICE is part of step SHICE inside quotes