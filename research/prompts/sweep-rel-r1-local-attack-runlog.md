# 运行记录：sweep-rel-r1-local-attack

提示：`research/prompts/sweep-rel-r1-local-attack.md`
译文核对表：`research/prompts/sweep-rel-r1-local-attack-translation-audit.md`
攻击面：A1（只限定义第 9 步、只限附录三 L1–L10），本轮抽两次，两次都判红作废次数为 0，两次都是有效、干净样本。

| 次 | 命令 | 退出码 | 词数（`wc -w`） | `oov-check.py` 结果 | 生词清单（模型自给、未核） | 判定 |
|---|---|---|---|---|---|---|
| s1 | `nice -n 19 bash research/scripts/ask-local.sh research/prompts/sweep-rel-r1-local-attack.md > research/prompts/sweep-rel-r1-local-attack-output-s1.md`（前台跑，被工具自身的 120s 超时机制转入后台监控，未使用 setsid/&/disown） | 0 | 1527 | 绿，生词=3，拼接=0 | reevaluation | 干净：`oov-check.py` 判绿；通读全文，十行（L1–L10）逐行都答完，句子完整，没有缺词、断句、孤立标点等第四类损坏的迹象 |
| s2 | `nice -n 19 bash research/scripts/ask-local.sh research/prompts/sweep-rel-r1-local-attack.md > research/prompts/sweep-rel-r1-local-attack-output-s2.md`（同上，前台跑，工具超时转后台监控） | 0 | 1169 | 绿，生词=5，拼接=0 | passage's、contradicted | 干净：`oov-check.py` 判绿；通读全文，十行都答完，句子完整，没有缺词、断句、孤立标点等第四类损坏的迹象 |

两次调用都是判绿、退出码 0 的有效样本，没有出现退出码 5（作废）或非 0 非 5（本地腿缺席）的情况；累计两份干净样本，达到停止条件，不再续抽。

`ask-local.sh` 内部已经串跑过 `corruption-check.py` 与 `oov-check.py`（任一判红即退出码 5），两次调用都以退出码 0 返回，说明两道闸都已过；本记录另外单独重跑 `oov-check.py` 复核（步骤 5 要求），结果与内部闸一致，两次都绿。

