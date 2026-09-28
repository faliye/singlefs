1. four_gibibyte_file LOW CONV1 CONV1 CONV1 F1 F2 irrelevant If F1 settles_holds is false or F2 settles_holds is false.
2. four_gibibyte_file MID CONV1 CONV1 CONV1 F3 F4 irrelevant If F3 settles_holds is false or F4 settles_holds is false.
3. four_gibibyte_file HIGH CONV1 CONV1 CONV1 F5 F6 irrelevant If F5 settles_holds is false or F6 settles_holds is false.
4. one_tebibyte_file LOW RISECONV RISECONV RISECONV F7 F8 irrelevant If F7 settles_holds is false or F8 settles_holds is false.
5. one_tebibyte_file MID CONV1 CONV1 CONV1 F9 F10 irrelevant If F9 settles_holds is false or F10 settles_holds is false.
6. one_tebibyte_file HIGH CONV1 CONV1 CONV1 F11 F12 irrelevant If F11 settles_holds is false or F12 settles_holds is false.
7. sixteen_tebibyte_file LOW CONV1 RISECONV mixed F13 F14 irrelevant If F14 peak_batch_low is 1 or F13 peak_batch_low is greater than 1.
8. sixteen_tebibyte_file MID RISECONV CONV1 mixed F15 F16 irrelevant If F15 peak_batch_low is 1 or F16 peak_batch_low is greater than 1.
9. sixteen_tebibyte_file HIGH CONV1 CONV1 CONV1 F17 F18 irrelevant If F17 settles_holds is false or F18 settles_holds is false.
10. full_root_level_four LOW RISECONV RISECONV RISECONV F19 F20 irrelevant If F19 settles_holds is false or F20 settles_holds is false.
11. full_root_level_four MID RISECONV RISECONV RISECONV F21 F22 irrelevant If F21 settles_holds is false or F22 settles_holds is false.
12. full_root_level_four HIGH CONV1 CONV1 CONV1 F23 F24 irrelevant If F23 settles_holds is false or F24 settles_holds is false.
13. two_fifty_six_mebibyte_file LOW CONV1 CONV1 CONV1 F25 F26 irrelevant If F25 settles_holds is false or F26 settles_holds is false.
14. two_fifty_six_mebibyte_file MID CONV1 CONV1 CONV1 F27 F28 irrelevant If F27 settles_holds is false or F28 settles_holds is false.
15. two_fifty_six_mebibyte_file HIGH CONV1 NOCONV mixed F29 F30 irrelevant If F30 settles_holds is true or F29 settles_holds is false.
16. small_fanout_one_tebibyte_file LOW RISECONV RISECONV RISECONV F31 F32 irrelevant If F31 settles_holds is false or F32 settles_holds is false.
17. small_fanout_one_tebibyte_file MID RISECONV CONV1 mixed F33 F34 irrelevant If F33 peak_batch_low is 1 or F34 peak_batch_low is greater than 1.
18. small_fanout_one_tebibyte_file HIGH CONV1 CONV1 CONV1 F35 F36 irrelevant If F35 settles_holds is false or F36 settles_holds is false.
19. large_fanout_one_tebibyte_file LOW CONV1 CONV1 CONV1 F37 F38 irrelevant If F37 settles_holds is false or F38 settles_holds is false.
20. large_fanout_one_tebibyte_file MID CONV1 CONV1 CONV1 F39 F40 irrelevant If F39 settles_holds is false or F40 settles_holds is false.
21. large_fanout_one_tebibyte_file HIGH CONV1 CONV1 CONV1 F41 F42 irrelevant If F41 settles_holds is false or F42 settles_holds is false.
22. F30; NOCONV does not contradict the bounded in G1b because the ratio remains bounded (e.g., around 2) even though it does not converge to 1.
23. Checking G1c's specific numbers for batch 50 and 200 requires data not present in this fact table, which only provides final_ratio at batch=16384 and peak_batch_low but not intermediate batch values.
24. No answer in the table can change G23.2's status because the data is about arm 甲 vs 乙, not arm 丙; changing the status would require measuring arm 丙 directly. The observation that would overturn this is if arm 丙 was measured and the results showed different behavior compared to arm 甲.