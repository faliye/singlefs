1 four_gibibyte_file LOW CONV1 CONV1 same F1 F2 irrelevant if F1 or F2 had settles_holds=false or peak_batch_low>1
2 four_gibibyte_file MID CONV1 CONV1 same F3 F4 irrelevant if F3 or F4 had settles_holds=false or peak_batch_low>1
3 four_gibibyte_file HIGH CONV1 CONV1 same F5 F6 irrelevant if F5 or F6 had settles_holds=false or peak_batch_low>1
4 one_tebibyte_file LOW RISECONV RISECONV same F7 F8 irrelevant if F7 or F8 had settles_holds=false or peak_batch_low=1
5 one_tebibyte_file MID CONV1 CONV1 same F9 F10 irrelevant if F9 or F10 had settles_holds=false or peak_batch_low>1
6 one_tebibyte_file HIGH CONV1 CONV1 same F11 F12 irrelevant if F11 or F12 had settles_holds=false or peak_batch_low>1
7 sixteen_tebibyte_file LOW CONV1 RISECONV mixed F13 F14 irrelevant if F13 did not have CONV1 or F14 did not have RISECONV
8 sixteen_tebibyte_file MID RISECONV CONV1 mixed F15 F16 irrelevant if F15 did not have RISECONV or F16 did not have CONV1
9 sixteen_tebibyte_file HIGH CONV1 CONV1 same F17 F18 irrelevant if F17 or F18 had settles_holds=false or peak_batch_low>1
10 full_root_level_four LOW RISECONV RISECONV same F19 F20 irrelevant if F19 or F20 had settles_holds=false or peak_batch_low=1
11 full_root_level_four MID RISECONV RISECONV same F21 F22 irrelevant if F21 or F22 had settles_holds=false or peak_batch_low=1
12 full_root_level_four HIGH CONV1 CONV1 same F23 F24 irrelevant if F23 or F24 had settles_holds=false or peak_batch_low>1
13 two_fifty_six_mebibyte_file LOW CONV1 CONV1 same F25 F26 irrelevant if F25 or F26 had settles_holds=false or peak_batch_low>1
14 two_fifty_six_mebibyte_file MID CONV1 CONV1 same F27 F28 irrelevant if F27 or F28 had settles_holds=false or peak_batch_low>1
15 two_fifty_six_mebibyte_file HIGH CONV1 NOCONV mixed F29 F30 irrelevant if F29 did not have CONV1 or F30 did not have NOCONV
16 small_fanout_one_tebibyte_file LOW RISECONV RISECONV same F31 F32 irrelevant if F31 or F32 had settles_holds=false or peak_batch_low=1
17 small_fanout_one_tebibyte_file MID RISECONV CONV1 mixed F33 F34 irrelevant if F33 did not have RISECONV or F34 did not have CONV1
18 small_fanout_one_tebibyte_file HIGH CONV1 CONV1 same F35 F36 irrelevant if F35 or F36 had settles_holds=false or peak_batch_low>1
19 large_fanout_one_tebibyte_file LOW CONV1 CONV1 same F37 F38 irrelevant if F37 or F38 had settles_holds=false or peak_batch_low>1
20 large_fanout_one_tebibyte_file MID CONV1 CONV1 same F39 F40 irrelevant if F39 or F40 had settles_holds=false or peak_batch_low>1
21 large_fanout_one_tebibyte_file HIGH CONV1 CONV1 same F41 F42 irrelevant if F41 or F42 had settles_holds=false or peak_batch_low>1

22 F30. NOCONV does not contradict the word bounded in G1b because the ratio remains bounded (e.g., approximately 1.98 for F30) even though it does not converge to 1, and bounded refers to the gap being finite rather than diverging to infinity.

23 F8 alone is not enough to check G1c's numbers because the fact table provides only peak_batch_low, settles_holds, and final_ratio without specific ratio values at batch=50 and batch=200 for streams=16; checking requires data at those exact batch sizes which is not present.

24 No, no answer in the 21-row table can change gate G23.2's status from answered-for-乙 to answered-for-丙 because the table is exclusively about arm 甲 versus arm 乙, while G23.2 concerns arm 甲 versus arm 丙, and arm 丙 has never been measured. This would require a different experiment measuring arm 丙. Observation that would overturn: if arm 丙 were measured and the data showed a different gap behavior, but since it is not measured, currently impossible with this data.