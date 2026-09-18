# 运行记录：里程碑「第二个事务」步 4 / 步 5 代码三方第一轮，本地辩方

提示：`research/prompts/m2-step45-code-r1-local-defense.md`；核对表：`research/prompts/m2-step45-code-r1-local-defense-translation-audit.md`。全部调用前台跑、加 `nice -n 19`，不 `setsid`。

## 逐次调用

| 次序 | 命令产出文件 | 退出码 | 判定 | 词数（`wc -w`）| 备注 |
|---|---|---|---|---|---|
| 1 | `m2-step45-code-r1-local-defense-output-void1.md` | 5 | 作废：`corruption-check.py` 判红，实词自复读 `reclaimed`（`红 … 实词自复读=1 …`；`绿` 那一行是提示文件自身的干净基线，不是本次输出） | 1364 | `ask-local.sh` 自动另存为 void1；本次 `> …-output-s1.md` 重定向落了空文件（0 字节），已删除后原名重跑 |
| 2 | `m2-step45-code-r1-local-defense-output-void2.md` | 5 | 作废：`oov-check.py` 判红，拼接词 `visiblevisible`（=visible+visible）| 2149 | 同上，空的 `-output-s1.md` 先删再重跑 |
| 3 | `m2-step45-code-r1-local-defense-output-void3.md` | 5 | 作废：`oov-check.py` 判红，拼接词 `visibleontent`（=visible+(c)ontent）| 1821 | 同上 |
| 4 | `m2-step45-code-r1-local-defense-output-s1.md` | 0 | 干净（闸内判绿，无损坏输出打印） | 1052 | 计作第 1 份样本 |
| 5 | `m2-step45-code-r1-local-defense-output-s2.md` | 0 | 干净（闸内判绿，无损坏输出打印） | 1249 | 计作第 2 份样本；一次成功，无作废轮 |

五次调用里三次坏、两次干净，与 `.claude/agents/three-way-local-defense.md`「实测（2026-09-16 第一轮）：辩方提示在本地模型上五次坏三次」相符（不同轮次，独立复现同一比例）。

## 补跑 oov-check.py（判绿之后仍要看生词表）

| 文件 | 退出码 | 生词表 | 判定 |
|---|---|---|---|
| `m2-step45-code-r1-local-defense-output-s1.md` | 0 | `strongest rollback's unreleasable NoPublishedVersion` | 无缺头粘连词，逐个都是提示原文里的正常英文词或标识符（`NoPublishedVersion` 是提示里给的错误名字），计干净样本 |
| `m2-step45-code-r1-local-defense-output-s2.md` | 0 | `strongest rollback's hardcodes unreleasable NoPublishedVersion` | 同上，计干净样本 |

两份样本都完整回答了 8 个编号问题、每题都有 (a)(b)(c) 三段，没有截断迹象（`wc -l` s1 = 15 行、s2 = 31 行，行尾都在第 8 题的 (c) 收尾）。

## 没做什么（固定会有的）

- 不解读、不总结、不采纳本地模型的答复；两份样本对第 1、2、4、8 项给出的结论不完全一致（例如第 4 项 s1 判「objection holds」、s2 判「does not hold」），判它站不站得住是主 agent 的事，这里只如实记录差异存在。
- 未额外抽第三次样：两次干净样本已到手，按定义「攒到至少两份干净样本为止」收尾。
