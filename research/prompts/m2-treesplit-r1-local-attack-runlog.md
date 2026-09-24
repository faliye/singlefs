# 运行记录：m2-treesplit-r1-local-attack（2026-09-24 UTC）

提示文件：
- `research/prompts/m2-treesplit-r1-local-attack.md`（52 格，四棵树合在一份提示里；第 1 次调用撞输出
  预算，退出码 3，见下方「原始 52 格文件」一节；未再用这份文件调用）。
- `research/prompts/m2-treesplit-r1-local-attack-ar.md`（分配记录树，13 格：leaf_cap/fanout/h3-h6/
  n3-n6/s_k1/s_k10/s_k100）
- `research/prompts/m2-treesplit-r1-local-attack-ac.md`（记账树，同 13 格标签）
- `research/prompts/m2-treesplit-r1-local-attack-cm.md`（中央映射树，同 13 格标签）
- `research/prompts/m2-treesplit-r1-local-attack-ex.md`（extent 树，同 13 格标签）

四份均不用任何 markdown 强调、不用 pipe 表格，答案按 13 个标签编号，指路只用 FACT 编号（FACT 1 至
FACT 11），明令不写文件行号或代码行号（需要指代码时按 FACT 编号或本文件给的英文名指）。每格要求
写公式、代入数字、中间步骤、最终整数、以及一句「This would be refuted by:」。转述核对表：
`research/prompts/m2-treesplit-r1-local-attack-translation-audit.md`（含 52 格撞预算改拆四份的补记）。

调用方式：`nice -n 19 bash research/scripts/ask-local.sh <提示文件> > <前缀>-output-sN.md`，前台跑，
全程未用 `setsid` / `&` / `disown`（首次调用超过工具 240 秒展示上限后被工具自动转入后台监视，
本次会话未主动加 `&`/`setsid`/`disown`，等的是工具自己发的完成通知，不是自己起的后台）。每次调用
前 `ps -o pid,etimes,pcpu,args -u "$(id -u)"` 查过 `qemu-system`、`vm-bench.sh`、
`e152-file-system-benchmark`、`fio`：全程零命中，不是性能测量在跑。

## 原始 52 格文件（作废，未再使用）

| 次序 | 目标文件 | 退出码 | 结论 |
|---|---|---|---|
| 1 | `m2-treesplit-r1-local-attack-output-s1.md` | 3 | 网关报错（非字词损坏闸判红）：`{"message": "输出预算不足被截断（已用 0 tok）", "type": "truncated", "diagnoses": ["第1次：正文复读跑飞，已掐断上游（判定时正文 6001 字符）", "第2次：...6002 字符", "第3次：...6005 字符"]}`；`$TXT` 未写，输出文件为空（0 行 0 词）；`corruption-check.py`/`oov-check.py` 未跑到（这条路径在 ask-local.sh 里更早退出）。按共用约束「本地腿只问能落成数、能逐格判的题」与既有先例（≤16 格一份），改拆成四份按树分的 13 格文件，见下方四节；这一次失败不计入任何一份新文件各自的「连续五次调用」额度（新文件是另一份自足提示，不是同一份的重跑） |

## AR（分配记录树，13 格）

| 次序 | 目标文件 | 退出码 | 词数（`wc -w`） | oov-check.py | corruption-check.py | 通读复核 | 结论 |
|---|---|---|---|---|---|---|---|
| 1 | `m2-treesplit-r1-local-attack-ar-output-s1.md` | 0 | 351 | 绿，生词=0 拼接=0 | 绿，全部计数为 0（words=300） | 13/13 格齐全、按序、各带公式与「This would be refuted by」，无缺词、无断句、无孤立标点 | 干净 |
| 2 | `m2-treesplit-r1-local-attack-ar-output-s2.md` | 0 | 421 | 绿，生词=0 拼接=0 | 绿，全部计数为 0（words=314） | 13/13 格齐全、按序、各带公式与「This would be refuted by」，无缺词、无断句、无孤立标点 | 干净 |

两份干净样本达标，AR 到此为止，不再抽样。

## AC（记账树，13 格）

| 次序 | 目标文件 | 退出码 | 词数（`wc -w`） | oov-check.py | corruption-check.py | 通读复核 | 结论 |
|---|---|---|---|---|---|---|---|
| 1 | `m2-treesplit-r1-local-attack-ac-output-s1.md` | 0 | 394 | 绿，生词=0 拼接=0 | 绿，全部计数为 0（words=376） | 13/13 格齐全、按序、各带公式与「This would be refuted by」，无缺词、无断句、无孤立标点 | 干净 |
| 2 | `m2-treesplit-r1-local-attack-ac-output-s2.md` | 0 | 386 | 绿，生词=0 拼接=0 | 绿，全部计数为 0（words=252） | 13/13 格齐全、按序、各带公式与「This would be refuted by」，无缺词、无断句、无孤立标点 | 干净 |

两份干净样本达标，AC 到此为止，不再抽样。

## CM（中央映射树，13 格）

| 次序 | 目标文件 | 退出码 | 词数（`wc -w`） | oov-check.py | corruption-check.py | 通读复核 | 结论 |
|---|---|---|---|---|---|---|---|
| 1 | `m2-treesplit-r1-local-attack-cm-output-s1.md` | 0 | 425 | 绿，生词=0 拼接=0 | 绿，全部计数为 0（words=399） | 13/13 格齐全、按序、各带公式与「This would be refuted by」，无缺词、无断句、无孤立标点 | 干净 |
| 2 | `m2-treesplit-r1-local-attack-cm-output-s2.md` | 0 | 391 | 绿，生词=0 拼接=0 | 绿，全部计数为 0（words=260） | 13/13 格齐全、按序、各带公式与「This would be refuted by」，无缺词、无断句、无孤立标点 | 干净 |

两份干净样本达标，CM 到此为止，不再抽样。

## EX（extent 树，13 格）

| 次序 | 目标文件 | 退出码 | 词数（`wc -w`） | oov-check.py | corruption-check.py | 通读复核 | 结论 |
|---|---|---|---|---|---|---|---|
| 1 | `m2-treesplit-r1-local-attack-ex-output-s1.md` | 0 | 473 | 绿，生词=0 拼接=0 | 绿，全部计数为 0（words=367） | 13/13 格齐全、按序、各带公式与「This would be refuted by」，无缺词、无断句、无孤立标点 | 干净 |
| 2 | `m2-treesplit-r1-local-attack-ex-output-s2.md` | 0 | 295 | 绿，生词=0 拼接=0 | 绿，全部计数为 0（words=294） | 13/13 格齐全、按序、各带公式与「This would be refuted by」，无缺词、无断句、无孤立标点 | 干净 |

两份干净样本达标，EX 到此为止，不再抽样。

## 干净样本清点（合计）

- AR：2 份干净（s1、s2）。AC：2 份干净（s1、s2）。CM：2 份干净（s1、s2）。EX：2 份干净（s1、s2）。
  四棵树合计 8 份干净样本，每棵树各自都在自己的「连续五次调用」额度之内（各只用了 2 次，一次未
  判红作废）。
- 全部调用累计 9 次：52 格原始文件 1 次（退出码 3，网关报错，非字词损坏闸判红，未产生 void 副本，
  该次不计入四份新文件各自的额度）+ 四棵树各 2 次（退出码全部 0）。退出码分布：0 出现 8 次，
  3 出现 1 次；没有退出码 5（字词损坏闸判红），没有 void 副本。
- 目录核对：`ls research/prompts/ | grep "m2-treesplit-r1-local-attack"` 显示 1 份 52 格原始提示、
  1 份它的空输出、4 份按树分的提示（`-ar.md`/`-ac.md`/`-cm.md`/`-ex.md`）、8 份对应的 `-output-sN.md`、
  1 份核对表、本文件，与本记录逐项一致，无遗漏、无多余。
- 本地腿全程未缺席：52 格文件那一次退出码 3 之后，改拆四份 13 格文件立即恢复正常（8/8 次退出码 0），
  不构成「网关不通」——网关期间对四份新文件持续应答成功。

## 没做什么

- 不解读、不总结、不采纳 8 份样本里对 AR/AC/CM/EX 13 格算术的具体数值方向是否一致，不判两次抽样
  之间是否一致——那是主 agent 的事；这份记录只报退出码、词数、生词清单、干净/带损坏/作废，不报
  样本答了什么。
- 未跑云端攻方腿（Opus，T1/T4/T6）或云端正推腿（Sonnet，T2/T3/T5 非算术半 + T1/T4/T6 一句），
  不在这条腿的任务范围内；`research/prompts/m2-treesplit-r1-sonnet-output.md` 已存在但按分工表
  不读、不碰。
- 未碰 T1、T3（非算术半）、T4、T6，按分工表不许碰；T5 的「树高进准入怎么算」这类政策判断本文件
  提示里也明令模型不答，只答算术。
- 未读这一轮的正文/背景材料之外的别的文件（如 `_m2-treesplit-r1-appendix.md`、
  `_m2-treesplit-r1-checklist.md`、`m2-treesplit-r1-snapshot/`），未读主 agent 这一轮的核实、
  同时在飞的别的轮次；本轮派发提示未给出明确的「禁读清单」，本报告据此按「这一轮未特别禁读」处理，
  仅读了完成本任务必需的文件（正文、背景材料、`crates/singlefs-format/src/lib.rs`、
  `.claude/kb/decisions/08-核心索引结构.md` 已定项 11、既有本地攻方提示先例）。
