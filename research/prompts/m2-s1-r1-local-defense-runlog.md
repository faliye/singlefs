# m2-s1-r1 本地辩方（H7）运行记录

提示文件：`research/prompts/m2-s1-r1-local-defense.md`
核对表：`research/prompts/m2-s1-r1-local-defense-translation-audit.md`
草稿目录：`/tmp/claude-1000/m2-s1-r1-local-defense/`（stderr 落盘：`s1.err`、`s2.err`，两份均为空）

调用命令（两次逐字相同，前台跑，未用 `setsid`/`&`/`disown`）：
`nice -n 19 bash research/scripts/ask-local.sh research/prompts/m2-s1-r1-local-defense.md > research/prompts/m2-s1-r1-local-defense-output-sN.md`

| 样本 | 退出码 | 词数（`wc -w`） | `oov-check.py` 结果 | 判定 |
|---|---|---|---|---|
| s1（`m2-s1-r1-local-defense-output-s1.md`） | 0 | 740 | `生词=0 拼接=0`（绿） | 干净 |
| s2（`m2-s1-r1-local-defense-output-s2.md`） | 0 | 513 | `生词=0 拼接=0`（绿） | 干净 |

两次调用都在退出码 5（字词损坏，脚本内建的 `corruption-check.py` + `oov-check.py` 双闸）之前判绿，`ask-local.sh` 自身 stderr 为空，没有触发作废重跑，没有 `-output-void*.md` 产生。除自动闸之外，两份样本都逐行通读过一遍：没有发现列表整项缺失、断句、孤立标点这类闸查不到的损坏。

两份都是本轮第一次、第二次调用就拿到的干净样本，没有用满「连续五次调用拿不到两份干净的就停」这条上限。

行号说明：样本文件里若出现任何编号（如「k=8」），那是模型自己给的编号，不是文件行号；提示第 10–12 行已明令模型的答案里不写文件或代码行号。
