Cell A k=1 group_total_bytes=344576 (204800*1 + 139776; F9 for cell A gives difference 139776, Table B row A N=1 WAL_FULL fsync bytes 204800), amortized=344576, ratio=344576/214048=1.610, ratio<=1.2? no  
Refuted if: for cell A k=1, the ratio is not greater than 1.610 as calculated  

Cell A k=2 group_total_bytes=549376 (204800*2 + 139776), amortized=274688, ratio=274688/214048=1.283, ratio<=1.2? no  
Refuted if: for cell A k=2, the ratio is not greater than 1.283 as calculated  

Cell A k=8 group_total_bytes=1778176 (204800*8 + 139776), amortized=222272, ratio=222272/214048=1.038, ratio<=1.2? yes  
Refuted if: for cell A k=8, the ratio is not less than or equal to 1.038 as calculated  

Cell A k=16 group_total_bytes=3416576 (204800*16 + 139776), amortized=213536, ratio=213536/214048=0.997, ratio<=1.2? yes  
Refuted if: for cell A k=16, the ratio is not less than or equal to 0.997 as calculated  

Cell B k=1 group_total_bytes=740712 (237568*1 + 503144; Table B row B N=1 WAL_FULL fsync bytes 237568, F9 difference 503144), amortized=740712, ratio=740712/269738.25=2.746, ratio<=1.2? no  
Refuted if: for cell B k=1, the ratio is not greater than 2.746 as calculated  

Cell B k=2 group_total_bytes=978280 (237568*2 + 503144), amortized=489140, ratio=489140/269738.25=1.813, ratio<=1.2? no  
Refuted if: for cell B k=2, the ratio is not greater than 1.813 as calculated  

Cell B k=8 group_total_bytes=2403688 (237568*8 + 503144), amortized=300461, ratio=300461/269738.25=1.114, ratio<=1.2? yes  
Refuted if: for cell B k=8, the ratio is not less than or equal to 1.114 as calculated  

Cell B k=16 group_total_bytes=4304232 (237568*16 + 503144), amortized=269014.5, ratio=269014.5/269738.25=0.997, ratio<=1.2? yes  
Refuted if: for cell B k=16, the ratio is not less than or equal to 0.997 as calculated  

Cell C k=1 group_total_bytes=803328 (663552*1 + 139776; Table B row C N=1 WAL_FULL fsync bytes 663552, F9 difference 139776), amortized=803328, ratio=803328/672800=1.194, ratio<=1.2? yes  
Refuted if: for cell C k=1, the ratio is not less than or equal to 1.194 as calculated  

Cell C k=2 group_total_bytes=1466880 (663552*2 + 139776), amortized=733440, ratio=733440/672800=1.090, ratio<=1.2? yes  
Refuted if: for cell C k=2, the ratio is not less than or equal to 1.090 as calculated  

Cell C k=8 group_total_bytes=5460480 (663552*8 + 139776), amortized=681024, ratio=681024/672800=1.012, ratio<=1.2? yes  
Refuted if: for cell C k=8, the ratio is not less than or equal to 1.012 as calculated  

Cell C k=16 group_total_bytes=10720800 (663552*16 + 139776), amortized=672288, ratio=672288/672800=0.999, ratio<=1.2? yes  
Refuted if: for cell C k=16, the ratio is not less than or equal to 0.999 as calculated  

Cell D k=1 group_total_bytes=1399224 (730692*1 + 669532; Table B row D N=1 WAL_FULL fsync bytes 730692, F9 difference 669532), amortized=1399224, ratio=1399224/774739.125=1.806, ratio<=1.2? no  
Refuted if: for cell D k=1, the ratio is not greater than 1.806 as calculated  

Cell D k=2 group_total_bytes=2130916 (730692*2 + 669532), amortized=1065458, ratio=1065458/774739.125=1.375, ratio<=1.2? no  
Refuted if: for cell D k=2, the ratio is not greater than 1.375 as calculated  

Cell D k=8 group_total_bytes=6507136 (730692*8 + 669532), amortized=813392, ratio=813392/774739.125=1.051, ratio<=1.2? yes  
Refuted if: for cell D k=8, the ratio is not less than or equal to 1.051 as calculated  

Cell D k=16 group_total_bytes=12360604 (730692*16 + 669532), amortized=772537.75, ratio=772537.75/774739.125=0.997, ratio<=1.2? yes  
Refuted if: for cell D k=16, the ratio is not less than or equal to 0.997 as calculated  

For all four cells, ratio(k) <= 1.2 holds when k=8 (smallest such k)  
Refuted if: for any cell (A, B, C, D), at k=8 the ratio exceeds 1.2  

Reason | Where it holds | One sentence citing which fact or table row  
(1) ring cannot be truncated | HOLDS FOR JIA-WITH-GROUP-COMMIT ONLY | F10 states that without publishing root on every fsync the journal ring cannot be truncated; JIA with group commit publishes root on every publish (fsync return), while WAL_FULL only publishes root at checkpoints  
Refuted if: WAL_FULL can truncate journal ring between checkpoints without publishing root  

(2) extra mount-time replay path | HOLDS FOR JIA-WITH-GROUP-COMMIT ONLY | F10 lists this as a reason; JIA publishes root on every fsync so no extra replay path, while WAL_FULL requires one due to deferred root publishing  
Refuted if: WAL_FULL does not require an extra mount-time replay path  

(3) journal enters verification chain | HOLDS FOR WAL_FULL ONLY | F10 elaboration states not publishing root on every fsync puts journal into verification chain; WAL_FULL does not publish root on every fsync so it enters verification chain, while JIA does not  
Refuted if: WAL_FULL does not enter the journal verification chain  

(4) no barrier saved | HOLDS FOR BOTH | F10 states choosing the deferred alternative does not save any flush barrier; both JIA and WAL_FULL require the same number of flush barriers  
Refuted if: WAL_FULL saves at least one flush barrier compared to JIA
