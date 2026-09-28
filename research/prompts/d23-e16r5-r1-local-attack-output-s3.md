1 four_gibibyte_file|LOW|CONV1|same|same|F1 F2|irrelevant|if F1 or F2 had settles_holds=false
2 four_gibibyte_file|MID|CONV1|same|same|F3 F4|irrelevant|if F3 or F4 had settles_holds=false
3 four_gibibyte_file|HIGH|CONV1|same|same|F5 F6|irrelevant|if F5 or F6 had settles_holds=false
4 one_tebibyte_file|LOW|RISECONV|same|same|F7 F8|irrelevant|if F7 or F8 had settles_holds=false or peak_batch_low=1 for F7
5 one_tebibyte_file|MID|CONV1|same|same|F9 F10|irrelevant|if F9 or F10 had settles_holds=false
6 one_tebibyte_file|HIGH|CONV1|same|same|F11 F12|irrelevant|if F11 or F12 had settles_holds=false
7 sixteen_tebibyte_file|LOW|CONV1|RISECONV|mixed|F13 F14|irrelevant|if F13 had settles_holds=false or F14 had peak_batch_low=1
8 sixteen_tebibyte_file|MID|RISECONV|CONV1|mixed|F15 F16|irrelevant|if F15 had settles_holds=false or F16 had peak_batch_low=1
9 sixteen_tebibyte_file|HIGH|CONV1|same|same|F17 F18|irrelevant|if F17 or F18 had settles_holds=false
10 full_root_level_four|LOW|RISECONV|same|same|F19 F20|irrelevant|if F19 or F20 had settles_holds=false or peak_batch_low=1
11 full_root_level_four|MID|RISECONV|same|same|F21 F22|irrelevant|if F21 or F22 had settles_holds=false or peak_batch_low=1
12 full_root_level_four|HIGH|CONV1|same|same|F23 F24|irrelevant|if F23 or F24 had settles_holds=false
13 two_fifty_six_mebibyte_file|LOW|CONV1|same|same|F25 F26|irrelevant|if F25 or F26 had settles_holds=false
14 two_fifty_six_mebibyte_file|MID|CONV1|same|same|F27 F28|irrelevant|if F27 or F28 had settles_holds=false
15 two_fifty_six_mebibyte_file|HIGH|CONV1|NOCONV|mixed|F29 F30|irrelevant|if F29 had settles_holds=false or F30 had settles_holds=true
16 small_fanout_one_tebibyte_file|LOW|RISECONV|same|same|F31 F32|irrelevant|if F31 or F32 had settles_holds=false or peak_batch_low=1
17 small_fanout_one_tebibyte_file|MID|RISECONV|CONV1|mixed|F33 F34|irrelevant|if F33 had settles_holds=false or F34 had peak_batch_low=1
18 small_fanout_one_tebibyte_file|HIGH|CONV1|same|same|F35 F36|irrelevant|if F35 or F36 had settles_holds=false
19 large_fanout_one_tebibyte_file|LOW|CONV1|same|same|F37 F38|irrelevant|if F37 or F38 had settles_holds=false
20 large_fanout_one_tebibyte_file|MID|CONV1|same|same|F39 F40|irrelevant|if F39 or F40 had settles_holds=false
21 large_fanout_one_tebibyte_file|HIGH|CONV1|same|same|F41 F42|irrelevant|if F41 or F42 had settles_holds=false
22 F30's NOCONV does not contradict the word bounded in G1b because the ratio remains finite (1.977) even though it does not converge to 1, so boundedness in terms of not growing indefinitely still holds
23 F8 alone is not enough to check G1c's numbers because the fact table does not include ratio values at batch=50 and batch=200 for streams=16; it only provides final_ratio at batch=16384 and peak_batch_low, so the required data for batch=50 and 200 is missing
24 No answer in the table can change G23.2's status to answered-for-丙 because all data is about arm 甲 vs 乙, not 丙; this would require a different experiment measuring arm 丙. The observation that would overturn this is if arm 丙 was measured and the data showed a gap between 甲 and 丙
