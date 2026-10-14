攻方腿推的结局（没跑：门禁格子 agent 不起，要主 agent 跑 run-gate-samples.sh）；派发闸 ⑥ 对同一行的判定是量过的（probes/r7-abandoned.sh，out/r7-r2.txt）：
样本                 门禁 abandoned-rounds 格（推的）                              派发闸 ⑥（量过）
r7-date-not-iso      退 1，「第 1 行日期不是 YYYY-MM-DD」                          收（rc=0，date-not-iso）
r7-duplicated-rows   退 1，「这些轮次在撂下登记表里登记了不止一行」                收（rc=0，duplicated-rows-second-good）
r7-no-source         退 0，「登记撂下 1 轮」                                       不收（rc=2，no-source）
