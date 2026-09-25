# 运行记录：m2-final-code-r3-local-attack（2026-09-25 08:55 JST / 2026-09-24 23:55 UTC）

提示文件：`research/prompts/m2-final-code-r3-local-attack.md`（英文，五道算术题，覆盖派发任务给
本地攻方的攻击面：① 分配记录树叶宽 W、② 分配记录树内部扇出、③ 根层公式在 240 槽单盘 / 4 GiB × 2 /
1 TiB × 2 三种盘配置上各取几（并要求「总槽数」与「单元区槽数」两种读法都算、报告是否一致）、④
extent 树上段叶 / 下段叶 / 内部扇出三个宽度、⑤ 145 个单元的文件下段几层；不用任何 markdown 强调、
不用 pipe 表格，答案按 5 个 item 编号，指路只用 fact 编号（1 至 13）或常量 / 函数名，明令不写文件名
与行号）。转述核对表：`research/prompts/m2-final-code-r3-local-attack-translation-audit.md`。

事实表来源全部现查冻结副本 `/tmp/claude-1000/m2-final-code-r3/tree/crates/`（`singlefs-format/src/
lib.rs`、`singlefs-core/src/allocator.rs`、`singlefs-core/src/allocation_record_tree.rs`、
`singlefs-core/src/extent_tree.rs`）与 kb 快照 `/tmp/claude-1000/m2-final-code-r3/kb-snapshot/`
（`.claude/kb/decisions/08-核心索引结构.md`），行号在核对表里逐条写死；事实表里**不含**任何一处
五道题要求模型独立算出的答案（W=812、内部扇出 169/147、上段叶 143、下段叶 144、根层与树高的具体
数值、145 单元文件的层级），详见核对表「有意省略的清单」一节。

调用方式：`nice -n 19 bash research/scripts/ask-local.sh <提示文件> > <前缀>-output-sN.md`，前台跑，
全程未用 `setsid` / `&` / `disown`。跑之前 `ps -o pid,etimes,pcpu,args -u "$(id -u)"` 查过
`qemu-system`、`vm-bench.sh`、`e152-file-system-benchmark`、`fio`：零命中，不是性能测量在跑。

## 逐次调用

| 次序 | 目标文件 | 退出码 | 词数（`wc -w`） | oov-check.py | corruption-check.py | 通读复核 | 结论 |
|---|---|---|---|---|---|---|---|
| 1 | `m2-final-code-r3-local-attack-output-s1.md` | 0 | 467 | 绿，生词=1（miscalculating，正常英文词） 拼接=0 | 绿，`cjk=0 words=325 fffd=0` 各项复读 / 落单 / 粘连 / 自复读计数均为 0 | 全文逐行读过：五个 item 编号齐全，每个 item 都有算式与「This would be refuted by:」收尾句，无缺词、无断句、无孤立标点、无粘连词；配置 A 按提示要求原样报「240」两遍（total 与 unit area 相同），配置 B / C 的 total 与 unit area 两条路都各自走了一遍除法与求和 | 干净 |
| 2 | `m2-final-code-r3-local-attack-output-s2.md` | 0 | 333 | 绿，生词=2（同一个词 miscalculating 在正文里出现两次，`dict.fromkeys` 去重后打印的仍是这一个词，不是第二个不同的生词） 拼接=0 | 绿，`cjk=0 words=212 fffd=0` 各项复读 / 落单 / 粘连 / 自复读计数均为 0 | 全文逐行读过：同上结构完整，无缺词、无断句、无孤立标点、无粘连词；配置 C 的 unit area 一支答复写「same as total → R=3」而不是重新列一遍除法步骤，格式与 s1 不同但同样完整、不构成损坏 | 干净 |

## 干净样本清点

- 干净：s1、s2，共 2 份，第一次与第二次调用即达到「至少两份干净样本」的门槛，到此停止抽样。
- 全部调用累计 2 次，未撞「连续五次调用拿不到两份干净的才停」的上限；退出码分布：0 出现 2 次，没有
  退出码 5（判红作废）、没有非 0 非 5 的退出码、没有网关不通，本地腿全程未缺席，无作废副本
  （`-output-void*.md`）产生。
- 目录核对：`ls research/prompts/ | grep m2-final-code-r3-local-attack` 显示提示文件 1 份、核对表
  1 份、运行记录 1 份（本文件）、`-output-s1.md`、`-output-s2.md` 共 2 份，与本记录逐行一致，无
  `-output-void*.md`。

## 没做什么

- 不解读、不总结、不采纳两份样本里 item 1 至 item 5 的具体数值答复，不判两次抽样之间的算式或结果
  是否一致——那是主 agent 的事，本报告按定义只写退出码、词数、生词清单、干净 / 带损坏 / 作废与核对
  表路径。
- 未跑云端攻方腿（Opus，Z13/Z16/Z17）、云端正推腿（Sonnet，Z14/Z15/Z18）或核实员，不在这条腿的
  任务范围内。
- 派发任务未给「草稿目录」与「这一轮的禁读清单」两项输入：本轮未使用会话共用暂存目录存放任何草稿
  （直接现查、直接落盘到最终路径，未产生需要暂存的中间文件），也没有可套用的禁读清单——按共用约束
  「输入缺一样就不开工」的精神，这里如实记录这两项未被给出，未自行假设其内容；后续如需要，由主
  agent 补给。
- 未读这一轮别的腿的提示与产出、主 agent 这一轮的核实、同时在飞的别的轮次。
