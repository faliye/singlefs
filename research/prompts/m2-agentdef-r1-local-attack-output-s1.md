1 P1 C1 not a heading line, so not flagged. Overturn: if heading line contains LISHI or BIANGENGSHI.
2 P1 C2 line points to records file, so not flagged. Overturn: if DATE outside quotes and no path to records or .claude/kb file.
3 P1 C3 not first line of paragraph, so not flagged. Overturn: if it were the first line of the paragraph.
4 P1 C4 no trigger words after punctuation points, so not flagged. Overturn: if after a punctuation mark, SHICE appears immediately after allowed gap.
5 P2 C1 not a heading line, so not flagged. Overturn: if heading line contains LISHI or BIANGENGSHI.
6 P2 C2 line points to records file, so not flagged. Overturn: if DATE outside quotes and no path to records or .claude/kb file.
7 P2 C3 start token after stripping is ordinary text, not a trigger, so not flagged. Overturn: if start token after stripping is WEISHENME followed immediately by COLON.
8 P2 C4 WEISHENME after SEMI is followed by non-COLON/WS, so not flagged. Overturn: if WEISHENME after SEMI is followed immediately by COLON.
9 P3 C1 not a heading line, so not flagged. Overturn: if heading line contains LISHI or BIANGENGSHI.
10 P3 C2 no DATE outside quotes, so not flagged. Overturn: if DATE outside quotes and no path to records or .claude/kb file.
11 P3 C3 start token after stripping is ordinary text, not a trigger, so not flagged. Overturn: if start token after stripping is YINWEI or ZHISUOYI.
12 P3 C4 no trigger words after punctuation points, so not flagged. Overturn: if after a punctuation mark, SHICE appears immediately after allowed gap.
13 P4 C1 not a heading line, so not flagged. Overturn: if heading line contains LISHI or BIANGENGSHI.
14 P4 C2 DATE inside quotes, so not flagged. Overturn: if DATE outside quotes and no path to records or .claude/kb file.
15 P4 C3 not first line of paragraph, so not flagged. Overturn: if it were the first line of the paragraph.
16 P4 C4 after LPAREN, DATE followed by non-group text, so not flagged. Overturn: if after LPAREN, DATE followed by SHICE.
17 P5 C1 not a heading line, so not flagged. Overturn: if heading line contains LISHI or BIANGENGSHI.
18 P5 C2 no DATE outside quotes, so not flagged. Overturn: if DATE outside quotes and no path to records or .claude/kb file.
19 P5 C3 start token after stripping is ordinary text, not a trigger, so not flagged. Overturn: if start token after stripping is SHICE followed immediately by COLON.
20 P5 C4 no trigger words after punctuation points, so not flagged. Overturn: if after LPAREN, SHICE appears immediately after allowed gap.
21 P6 C1 not a heading line, so not flagged. Overturn: if heading line contains LISHI or BIANGENGSHI.
22 P6 C2 no DATE outside quotes, so not flagged. Overturn: if DATE outside quotes and no path to records or .claude/kb file.
23 P6 C3 start token after stripping is ordinary text, not a trigger, so not flagged. Overturn: if start token after stripping is YINWEI or ZHISUOYI.
24 P6 C4 no trigger words after punctuation points, so not flagged. Overturn: if after SEMI, SHICE appears immediately after allowed gap.
25 P7 C1 not a heading line, so not flagged. Overturn: if heading line contains LISHI or BIANGENGSHI.
26 P7 C2 no DATE outside quotes, so not flagged. Overturn: if DATE outside quotes and no path to records or .claude/kb file.
27 P7 C3 start token after stripping is ordinary text, not a trigger, so not flagged. Overturn: if start token after stripping is YIJU followed immediately by COLON.
28 P7 C4 after DUN mark, exempt from flagging, so not flagged. Overturn: if after COMMA, YIJU followed immediately by COLON.
29 P8 C1 not a heading line, so not flagged. Overturn: if heading line contains LISHI or BIANGENGSHI.
30 P8 C2 DATE outside quotes and no path to records or .claude/kb file, so flagged. Overturn: if line points to records or .claude/kb file.
31 P8 C3 SHICE followed immediately by LPAREN, so flagged. Overturn: if SHICE followed by a character not in connector list.
32 P8 C4 after LPAREN, DATE followed by non-group text, so not flagged. Overturn: if after LPAREN, DATE followed by SHICE.
33 P9 C1 not a heading line, so not flagged. Overturn: if heading line contains LISHI or BIANGENGSHI.
34 P9 C2 DATE outside quotes and no path to records or .claude/kb file, so flagged. Overturn: if line points to records or .claude/kb file.
35 P9 C3 start token after stripping is ordinary text, not a trigger, so not flagged. Overturn: if start token after stripping is YINWEI or ZHISUOYI.
36 P9 C4 after LPAREN, DATE WS SHICE, so flagged. Overturn: if after LPAREN, DATE WS followed by non-group one word.
37 P10 C1 not a heading line, so not flagged. Overturn: if heading line contains LISHI or BIANGENGSHI.
38 P10 C2 DATE outside quotes and no path to records or .claude/kb file, so flagged. Overturn: if line points to records or .claude/kb file.
39 P10 C3 start token after stripping is ordinary text, not a trigger, so not flagged. Overturn: if start token after stripping is SHICE followed immediately by COLON.
40 P10 C4 after COMMA, DATE WS SHICE, so flagged. Overturn: if after COMMA, DATE WS followed by non-group one word.
41 C1 hypothetical heading line "Process Overview". Literal verdict GREEN (no LISHI or BIANGENGSHI), intended RED (process narrative). Divergence: C1 checks only specific tags, not general explanatory prose. Overturn: heading contains LISHI.
42 C2 hypothetical line "2023-01-01: meeting notes" with path to records file. Literal verdict GREEN (points to records), intended RED (dated record). Divergence: C2's path check overrides DATE presence. Overturn: no path to records or .claude/kb file.
43 C3 hypothetical first line "YUANYIN the reason is unclear". Literal verdict GREEN (YUANYIN followed by non-connector), intended RED (explanatory prose). Divergence: C3 requires label word followed by connector. Overturn: YUANYIN followed by COLON.
44 C4 hypothetical line "Check result, SHI_COPULA 5". Literal verdict GREEN (SHI_COPULA not in group), intended RED (explanatory prose). Divergence: C4 doesn't check SHI_COPULA. Overturn: after COMMA, SHICE appears.
