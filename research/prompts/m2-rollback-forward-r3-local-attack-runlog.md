# m2-rollback-forward-r3 本地攻方运行记录

提示文件：`research/prompts/m2-rollback-forward-r3-local-attack-part1.md`（K4 三臂算术）、
`research/prompts/m2-rollback-forward-r3-local-attack-part2.md`（K3 去重算术）。
调用：`bash research/scripts/ask-local.sh <提示文件>`，前台跑，未用 setsid / & / disown。
译文核对表：`research/prompts/m2-rollback-forward-r3-local-attack-translation-audit.md`。

## part1（K4 三臂算术）

| 样本 | 退出码 | 词数（wc -w） | oov-check.py 生词 | 判定 |
|---|---|---|---|---|
| s1 | 0 | 570 | Unfavorable, misinterpreting, unfavorable | 干净（oov-check.py 绿，拼接=0；通读无缺词、无断句、无粘连） |
| s2 | 0 | 348 | misreading, assumption's, misinterpreting, Unfavorable | 干净（oov-check.py 绿，拼接=0；通读无缺词、无断句、无粘连） |

两份均干净，达到停止线（至少两份干净样本），part1 只跑了 2 次，未作废、无缺席。

## part2（K3 去重算术）

| 样本 | 退出码 | 词数（wc -w） | oov-check.py 生词 | 判定 |
|---|---|---|---|---|
| s1 | 0 | 82 | histories | 干净（oov-check.py 绿，拼接=0；通读无缺词、无断句、无粘连；答复比 part2 提示要求的步骤简略，但简略不等于损坏，未记带损坏） |
| s2 | 0 | 166 | histories | 干净（oov-check.py 绿，拼接=0；通读无缺词、无断句、无粘连） |

两份均干净，达到停止线（至少两份干净样本），part2 只跑了 2 次，未作废、无缺席。

## 汇总

- 两份提示均未撞网关报错（退出码非 0 非 5），也没有退出码 5 的判红作废；两份提示都在各自第 2 次调用即拿到 2 份干净样本，均未触及「连续五次调用拿不到两份干净」的停止线。
- 全程 4 次调用，退出码均为 0，无作废副本（`-output-void*.md`）产生。
- 未见任何一份样本出现缺头粘连词、复读、成对标记落单、缺词断句这类损坏形态。
- 样本文件：
  `research/prompts/m2-rollback-forward-r3-local-attack-part1-output-s1.md`、
  `research/prompts/m2-rollback-forward-r3-local-attack-part1-output-s2.md`、
  `research/prompts/m2-rollback-forward-r3-local-attack-part2-output-s1.md`、
  `research/prompts/m2-rollback-forward-r3-local-attack-part2-output-s2.md`。
