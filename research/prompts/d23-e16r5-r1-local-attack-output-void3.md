1 four_gibibyte_file LOW CONV1 CONV1 same F1 F2 irrelevant if either F1 or F2 had settles_holds=false or peak_batch_low >1
2 four_gibibyte_file MID CONV1 CONV1 same F3 F4 irrelevant if either F3 or F4 had settles_holds=false or peak_batch_low >1
3 four_gibibyte_file HIGH CONV1 CONV1 same F5 F6 irrelevant if either F5 or F6 had settles_holds=false or peak_batch_low >1
4 one_tebibyte_file LOW RISECONV RISECONV same F7 F8 irrelevant if either F7 or F8 had settles_holds=false or peak_batch_low mismatch
5 one_tebibyte_file MID CONV1 CONV1 same F9 F10 irrelevant if either F9 or F10 had settles_holds=false or peak_batch_low >1
6 one_tebibyte_file HIGH CONV1 CONV1 same F11 F12 irrelevant if either F11 or F12 had settles_holds=false or peak_batch_low >1
7 sixteen_tebibyte_file LOW CONV1 RISECONV mixed F13 F14 irrelevant if both F13 and F14 had same category
8 sixteen_tebibyte_file MID RISECONV CONV1 mixed F15 F16 irrelevant if both F15 and F16 had same category
9 sixteen_tebibyte_file HIGH CONV1 CONV1 same F17 F18 irrelevant if either F17 or F18 had settles_holds=false or peak_batch_low >1
10 full_root_level_four LOW RISECONV RISECONV same F19 F20 irrelevant if either F19 or F20 had settles_holds=false or peak_batch_low mismatch
11 full_root_level_four MID RISECONV RISECONV same F21 F22 irrelevant if either F21 or F22 had settles_holds=false or peak_batch_low mismatch
12 full_root_level_four HIGH CONV1 CONV1 same F23 F24 irrelevant if either F23 or F24 had settles_holds=false or peak_batch_low >1
13 two_fifty_six_mebibyte_file LOW CONV1 CONV1 same F25 F26 irrelevant if either F25 or F26 had settles_holds=false or peak_batch_low >1
14 two_fifty_six_mebibyte_file MID CONV1 CONV1 same F27 F28 irrelevant if either F27 or F28 had settles_holds=false or peak_batch_low >1
15 two_fifty_six_mebibyte_file HIGH CONV1 NOCONV mixed F29 F30 irrelevant if both F29 and F30 had same category
16 small_fanout_one_tebibyte_file LOW RISECONV RISECONV same F31 F32 irrelevant if either F31 or F32 had settles_holds=false or peak_batch_low mismatch
17 small_fanout_one_tebibyte_file MID RISECONV CONV1 mixed F33 F34 irrelevant if both F33 and F34 had same category
18 small_fanout_one_tebibyte_file HIGH CONV1 CONV1 same F35 F36 irrelevant if either F35 or F36 had settles_holds=false or peak_batch_low >1
19 large_fanout_one_tebibyte_file LOW CONV1 CONV1 same F37 F38 irrelevant if either F37 or F38 had settles_holds=false or peak_batch_low >1
20 large_fanout_one_tebibyte_file MID CONV1 CONV1 same F39 F40 irrelevant if either F39 or F40 had settles_holds=false or peak_batch_low >1
21 large_fanout_one_tebibyte_file HIGH CONV1 CONV1 same F41 F42 irrelevant if either F41 or F42 had settles_holds=false or peak_batch_low >1
22 F30; NOCONV for F30 does not contradict G1b's "bounded" because the ratio remains below 2.0, which is bounded (not diverging to infinity), even though it does not converge to 1
23 Checking G1c's numbers requires data for intermediate batch sizes (50 and 200) for the 16-stream row of root_children=11, which is not present in the current fact table
24 No; all data is about 甲 versus 乙 while G23.2 concerns 甲 versus 丙, which requires measuring arm 丙; the observation that would overturn this is if arm 丙 was measured in this experiment