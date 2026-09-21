# 本地攻方运行记录：m2-supp3-item3-code-r2-local-attack（2026-09-21，JST）

提示文件：`research/prompts/m2-supp3-item3-code-r2-local-attack-k4.md`（K4：种子基按周期重抽之后复现还成不成立）、
`research/prompts/m2-supp3-item3-code-r2-local-attack-k5.md`（K5：已知红的判别力挂在哪）。
两份提示都前台跑（无 `setsid`/`&`/`disown`），命令：`bash research/scripts/ask-local.sh <提示文件> > <前缀>-output-s<n>.md`。
每份提示各调用两次（s1、s2），全部退出码 0、无退出码 5 的作废副本、无本地腿缺席。
核对表：`research/prompts/m2-supp3-item3-code-r2-local-attack-translation-audit.md`。

## 调用记录

| 提示 | 样本 | ask-local.sh 退出码 | 词数（`wc -w`） | `oov-check.py` 判定 | 生词清单 | 状态 |
|---|---|---|---|---|---|---|
| K4 | s1 | 0 | 579 | 绿（生词=0，拼接=0） | 无 | 干净 |
| K4 | s2 | 0 | 305 | 绿（生词=0，拼接=0） | 无 | 干净 |
| K5 | s1 | 0 | 461 | 绿（生词=1，拼接=0） | `epsilon's` | 干净（见下方说明） |
| K5 | s2 | 0 | 313 | 绿（生词=0，拼接=0） | 无 | 干净 |

四次调用全部退出码 0，没有一次退出码 5，没有作废副本（`ls research/prompts/m2-supp3-item3-code-r2-local-attack-k4-output-void*.md`
与 `...-k5-output-void*.md` 均无命中）。K4 第一次调用（s1）超过 Bash 工具默认 120 秒超时被自动转后台，本子 agent 没有主动加
`setsid`/`&`/`disown`，等到系统通知完成后原样读取产物；退出码与产物内容与前台直接完成的三次一致。

## K5-s1 的 oov-check 生词说明

`oov-check.py` 把 `epsilon's` 记成生词。核实：这是样本正文里「after epsilon's change」一句，`epsilon` 是本份提示第 29 行
自己定义的假设行标签「row Epsilon」（不在 `crates/mutations.tsv` 里，是本子 agent 为测「只改一个标签措辞」这个场景另写的
第四行），模型只是接了这个专有名词的所有格形式，不是词头脱落的粘连词（不长成 `achievesceives` 那种形态），也不是本份材料
之外造出来的新词。判定：不算带损坏。通读 K5-s1 全文另核过复读、成对标记落单、粘连、缺词断句四类，均未见。

## 干净样本汇总

K4 两份（s1、s2）与 K5 两份（s1、s2）均判干净，达到「至少两份干净样本」的门槛，两个问题各自不需要再抽样、也没有触发
「连续五次调用拿不到两份干净的」停止条款。
