Cell | k | group_total_bytes(k) | amortized_bytes_per_fsync(k) | ratio(k) | ratio(k) <= 1.2 ?
A | 1 | 344576 | 344576 | 1.610 | No
A | 2 | 549376 | 274688 | 1.283 | No
A | 8 | 1778176 | 222272 | 1.038 | Yes
A | 16 | 3416576 | 213536 | 0.997 | Yes
B | 1 | 740712 | 740712 | 2.746 | No
B | 2 | 978280 | 489140 | 1.813 | No
B | 8 | 2403688 | 300461 | 1.114 | Yes
B | 16 | 4304232 | 269014.5 | 0.997 | Yes
C | 1 | 803328 | 803328 | 1.194 | Yes
C | 2 | 1466880 | 733440 | 1.090 | Yes
C | 8 | 5448192 | 681024 | 1.012 | Yes
C | 16 | 10756608 | 672288 | 0.999 | Yes
D | 1 | 1400224 | 1400224 | 1.807 | No
D | 2 | 2130916 | 1065458 | 1.375 | No
D | 8 | 6515068 | 814383.5 | 1.051 | Yes
D | 16 | 12360604 | 772537.75 | 0.997 | Yes

group_total_bytes(k) = k * (Table B row for cell N=1 WAL_FULL fsync bytes) + (Table A row for cell JIA fsync bytes - Table B row for cell N=1 WAL_FULL fsync bytes). For example, cell A k=8: 8 * 204800 + (344576 - 204800) = 1778176. Refuted if the calculated group_total_bytes(k) does not match this formula using the given Table B and Table A values.

amortized_bytes_per_fsync(k) = group_total_bytes(k) / k. For example, cell A k=8: 1778176 / 8 = 222272. Refuted if the division result does not match group_total_bytes(k) / k.

ratio(k) = amortized_bytes_per_fsync(k) / (Table B row for cell N=16 WAL_FULL amortized bytes). For example, cell A k=8: 222272 / 214048 ≈ 1.038. Refuted if the ratio calculation uses incorrect WAL_FULL reference or amortized_bytes_per_fsync(k).

ratio(k) <= 1.2 holds for k=8 across all cells A, B, C, D. Refuted if any cell's ratio(k=8) > 1.2.

Reason | Where it holds | One sentence citing which fact or table row
(1) ring cannot be truncated | HOLDS FOR WAL_FULL ONLY | F11 states that not publishing root on every fsync (WAL_FULL) results in ring peak occupancy of 10^4 to 10^5 bytes versus 48 bytes under JIA.
(2) extra mount-time replay path | HOLDS FOR WAL_FULL ONLY | F11 states that not publishing root on every fsync requires one more mount-time replay code path.
(3) journal enters verification chain | HOLDS FOR WAL_FULL ONLY | F11 states that not publishing root on every fsync puts the journal into the verification chain.
(4) no barrier saved | HOLDS FOR WAL_FULL ONLY | F11 states that choosing the alternative (WAL_FULL) does not save even one flush barrier.

Refuted if WAL_FULL can truncate the journal ring with peak occupancy below 10^4 bytes.
Refuted if WAL_FULL does not require an extra mount-time replay code path.
Refuted if WAL_FULL does not place the journal into the verification chain.
Refuted if WAL_FULL saves at least one flush barrier compared to JIA.
