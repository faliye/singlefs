1 P1 C1 GREEN Not a heading line If it were a heading line containing LISHI or BIANGENGSHI
2 P1 C2 GREEN Contains path to records file If it contained DATE outside quotes and no path to records or .claude/kb
3 P1 C3 GREEN Not first line of paragraph If it were the first line of its paragraph
4 P1 C4 GREEN No points after punctuation meet criteria If a point after punctuation had SHICE immediately after WS and DATE
5 P2 C1 GREEN Not a heading line If it were a heading line containing LISHI or BIANGENGSHI
6 P2 C2 GREEN Contains path to records file If it contained DATE outside quotes and no path to records or .claude/kb
7 P2 C3 GREEN Line-start after stripping is ordinary text If line-start after stripping was a label word followed by COLON
8 P2 C4 GREEN After SEMI WEISHENME followed by non-COLON/WS If WEISHENME was immediately followed by COLON after SEMI
9 P3 C1 GREEN Not a heading line If it were a heading line containing LISHI or BIANGENGSHI
10 P3 C2 GREEN No DATE outside quotes If DATE existed outside quotes and no path to records or .claude/kb
11 P3 C3 GREEN Line-start after stripping is ordinary text If line-start after stripping was a label word followed by COLON
12 P3 C4 GREEN No points after punctuation meet criteria If a point after punctuation had SHICE immediately after WS and DATE
13 P4 C1 GREEN Not a heading line If it were a heading line containing LISHI or BIANGENGSHI
14 P4 C2 GREEN DATE inside quotes If DATE existed outside quotes and no path to records or .claude/kb
15 P4 C3 GREEN Not first line of paragraph If it were the first line of its paragraph
16 P4 C4 GREEN After LPAREN DATE then text not trigger word If DATE was followed by WS and SHICE after LPAREN
17 P5 C1 GREEN Not a heading line If it were a heading line containing LISHI or BIANGENGSHI
18 P5 C2 GREEN No DATE outside quotes If DATE existed outside quotes and no path to records or .claude/kb
19 P5 C3 GREEN Line-start after stripping is ordinary text If line-start after stripping was a label word followed by COLON
20 P5 C4 GREEN No points after punctuation meet criteria If a point after punctuation had SHICE immediately after WS and DATE
21 P6 C1 GREEN Not a heading line If it were a heading line containing LISHI or BIANGENGSHI
22 P6 C2 GREEN No DATE outside quotes If DATE existed outside quotes and no path to records or .claude/kb
23 P6 C3 GREEN Line-start after stripping is ordinary text If line-start after stripping was a label word followed by COLON
24 P6 C4 GREEN No points after punctuation meet criteria If a point after punctuation had SHICE immediately after WS and DATE
25 P7 C1 GREEN Not a heading line If it were a heading line containing LISHI or BIANGENGSHI
26 P7 C2 GREEN No DATE outside quotes If DATE existed outside quotes and no path to records or .claude/kb
27 P7 C3 GREEN Line-start after stripping is ordinary text If line-start after stripping was a label word followed by COLON
28 P7 C4 GREEN YIJU followed by LPAREN not COLON/WS If YIJU was followed by COLON after DUN
29 P8 C1 GREEN Not a heading line If it were a heading line containing LISHI or BIANGENGSHI
30 P8 C2 RED DATE outside quotes and no path to records or .claude/kb If path to records or .claude/kb existed
31 P8 C3 RED SHICE followed by LPAREN (connector) If SHICE was followed by non-connector
32 P8 C4 GREEN No points after punctuation meet criteria If a point after punctuation had SHICE immediately after WS and DATE
33 P9 C1 GREEN Not a heading line If it were a heading line containing LISHI or BIANGENGSHI
34 P9 C2 RED DATE outside quotes and no path to records or .claude/kb If path to records or .claude/kb existed
35 P9 C3 GREEN Line-start after stripping is ordinary text If line-start after stripping was a label word followed by COLON
36 P9 C4 RED After LPAREN DATE WS SHICE If SHICE was not preceded by DATE WS
37 P10 C1 GREEN Not a heading line If it were a heading line containing LISHI or BIANGENGSHI
38 P10 C2 RED DATE outside quotes and no path to records or .claude/kb If path to records or .claude/kb existed
39 P10 C3 GREEN Line-start after stripping is ordinary text If line-start after stripping was a label word followed by COLON
40 P10 C4 RED After COMMA DATE WS SHICE If SHICE was not preceded by DATE WS
41 C1 Literal RED but intended GREEN: heading line with BIANGENGSHI as valid step title Literal verdict RED due to BIANGENGSHI presence but intended purpose considers it a valid step; overturn if heading lacks BIANGENGSHI
42 C2 Literal GREEN but intended RED: DATE outside quotes with path to records Literal verdict GREEN due to path existence but intended purpose flags all dated records; overturn if no path to records
43 C3 Literal GREEN but intended RED: WEISHENME followed by text without connector Literal verdict GREEN due to no connector after WEISHENME but intended purpose flags explanatory prose; overturn if WEISHENME followed by COLON
44 C4 Literal GREEN but intended RED: after DUN SHICE Literal verdict GREEN due to DUN exemption but intended purpose flags explanatory prose; overturn if trigger punctuation is COMMA instead of DUN
