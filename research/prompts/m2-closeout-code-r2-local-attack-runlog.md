# m2-closeout-code-r2 本地攻方腿 运行记录

提示文件：`research/prompts/m2-closeout-code-r2-local-attack.md`（sha256 9d63627e…d1c81c）
译文核对表：`research/prompts/m2-closeout-code-r2-local-attack-translation-audit.md`（sha256 bb9cc97f…c2412f9）

## 调用记录（按次序，共 4 次调用，作废也计次）

1. `bash research/scripts/ask-local.sh <提示文件> > …-output-s1.md`（首次前台起，跑满 120s 前台上限被工具自动挪进后台，任务 id `b8zme0gs1`）：退出码 0。样本 `m2-closeout-code-r2-local-attack-output-s1.md`，729 词。`oov-check.py`（带提示文件参数）：生词=0，拼接=0，判绿。逐段通读：表格、算式完整，无缺头粘连词、无断句缺词。**判：干净**。
2. 同一提示重跑到 `…-output-s2.md`（任务 id `b25o2cdcv`）：退出码 5（字词损坏，闸判红）。`-output-s2.md` 落成 0 字节（已核实大小为 0），作废副本由脚本写成 `m2-closeout-code-r2-local-attack-output-void1.md`（4156 字节）。**判：作废**，不计入干净样本。
3. 用 `>|` 重定向到同一个号 `…-output-s2.md` 重跑（任务 id `bz5v1cwww`）：退出码 5。`-output-s2.md` 再次落成 0 字节（已核实），作废副本 `m2-closeout-code-r2-local-attack-output-void2.md`（3762 字节）。**判：作废**，不计入干净样本。
4. 再用 `>|` 重定向到同一个号 `…-output-s2.md` 重跑（任务 id `b2sf44s5r`）：退出码 0。样本 `m2-closeout-code-r2-local-attack-output-s2.md`，540 词。`oov-check.py`（带提示文件参数）：生词=0，拼接=0，判绿。逐段通读：表格、算式完整，无缺头粘连词、无断句缺词。**判：干净**。

干净样本数：2（`-output-s1.md`、`-output-s2.md`），达到「至少两份干净样本」的门槛，停止取号。总调用次数 4，未触发「连续五次调用拿不到两份干净的」停线。

## 5b 算术答案表比对

派发提示未给算好的答案表路径（也未提及另有答案表）。**没有答案表，算术没比**。

## 没做什么

- 不解读、不总结、不采纳两份样本的答复内容；样本回答的方向是否一致、算得对不对，不属于本腿判断范围，交主 agent。
- 未跑重型测试。
- `-output-void1.md`、`-output-void2.md` 是脚本判红时自动留存的作废副本，未读取其内容用于任何结论（仅确认了对应 `s2` 曾是 0 字节这一件事实）。

## 交回失误记录（如实报告）

本次任务中途我误调用了一次 SubagentHandback（在样本尚未采集完成、仅起了第一次后台调用时），发出一条占位/不完整的消息。之后会话仍收到了后台任务完成通知并被允许继续工作，遂在同一会话内完成了剩余步骤（第二次采样、判净、写运行记录）。这次误调用本身没有造成文件或产出损坏，但违反了「交回只调一次」的约束，如实记在这里，供主 agent 核查这一条腿的产出时留意：本报告是该次误交回之后、同一会话续做出来的最终结果。
