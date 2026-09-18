# bg-notify-r1 本地攻方腿：运行记录

提示文件：`research/prompts/bg-notify-r1-local-attack.md`
核对表：`research/prompts/bg-notify-r1-local-attack-translation-audit.md`
调用方式：`nice -n 19 bash research/scripts/ask-local.sh research/prompts/bg-notify-r1-local-attack.md > <前缀>-output-s<n>.md`，前台跑（s2 那次超过工具自身 120 秒默认超时被 harness 自动挪到后台，不是我自己加 `setsid`/`&`/`disown`；输出仍是完整落盘、非 0 字节）。

| 样本 | 退出码 | 词数（`wc -w`） | `oov-check.py` 判定 | 生词清单 | 通读结果 | 判定 |
|---|---|---|---|---|---|---|
| s1 | 0 | 925 | 绿（生词=16，拼接=0） | overturned disagrees（另 14 个未单列，均为正常英文词） | 14 条编号答复齐全、句子完整，无缺词、无孤立标点、无 markdown 强调、无代码/文件行号 | 干净 |
| s2 | 0 | 884 | 绿（生词=14，拼接=0） | overturned（另 13 个未单列，均为正常英文词） | 14 条编号答复齐全、句子完整，无缺词、无孤立标点、无 markdown 强调、无代码/文件行号 | 干净 |

两份样本均干净，调用两次即达到「至少两份干净样本」，未触发第 6 步的五次上限。没有作废副本（无 `-output-void*.md` 生成）、没有退出码 5 的判红轮。
