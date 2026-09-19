# 运行记录：m2-supp3-item1-code-r2 本地攻方（Y5 算术半 / Y6 日志读法半）

提示文件：`research/prompts/m2-supp3-item1-code-r2-local-attack.md`
核对表：`research/prompts/m2-supp3-item1-code-r2-local-attack-translation-audit.md`
调用：`nice -n 19 bash research/scripts/ask-local.sh research/prompts/m2-supp3-item1-code-r2-local-attack.md`，前台跑，未用 `setsid` / `&` / `disown`。

| 调用序号 | 输出文件 | 退出码 | corruption-check + oov-check 判定 | 生词清单 | 词数（`wc -w`） | 状态 |
|---|---|---|---|---|---|---|
| 1 | `m2-supp3-item1-code-r2-local-attack-output-s1.md` | 0 | 绿（`生词=0 拼接=0`） | 无 | 353 | 干净 |
| 2 | `m2-supp3-item1-code-r2-local-attack-output-s2.md` | 0 | 绿（`生词=0 拼接=0`） | 无 | 290 | 干净 |

两次调用都是退出码 0、`oov-check.py` 判绿、生词清单为空；另外通读一遍两份样本全文，句子完整、编号 Q1–Q18 齐全，没有看到缺词、断句或孤立标点这类损坏。两份都记「干净」，无作废（void）副本、无带损坏样本。已达到「至少两份干净样本」，未触发「连续五次调用拿不到两份干净的」停止条款。
