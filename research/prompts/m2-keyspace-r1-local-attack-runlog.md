# 运行记录：m2-keyspace-r1-local-attack（S6 算术表，2026-09-24 UTC）

提示文件（四份，按「树 × 候选内部条目宽」拆分，拆分理由见
`research/prompts/m2-keyspace-r1-local-attack-translation-audit.md`「四份文件的拆分理由」一节）：

- `m2-keyspace-r1-local-attack-ar-ptr.md`（分配记录树，候选=仅子指针 86 字节，13 格）
- `m2-keyspace-r1-local-attack-ar-key.md`（分配记录树，候选=key10+子指针86=96 字节，13 格）
- `m2-keyspace-r1-local-attack-ex-ptr.md`（extent 树，候选=仅子指针 86 字节，非已定案，11 格）
- `m2-keyspace-r1-local-attack-ex-key.md`（extent 树，已定案=key24+子指针86=110 字节，11 格）

四份均不用任何 markdown 强调、不用 pipe 表格，答案按各自 11 或 13 个标签编号，指路只用 FACT 编号
（FACT 1 至 FACT 11），明令不写文件行号或代码行号（需要指代码时按 FACT 编号或提示给的英文名指）。
每格要求写公式、代入数字、中间步骤（层链 L(1)…L(h) 或逐层 min(k, L(i))）、最终整数、以及一句
「This would be refuted by:」。转述核对表：
`research/prompts/m2-keyspace-r1-local-attack-translation-audit.md`。

调用方式：`nice -n 19 bash research/scripts/ask-local.sh <提示文件> > <前缀>-output-sN.md`，前台跑，
全程未用 `setsid` / `&` / `disown`（其中一次调用超过工具 120 秒展示上限后被工具自动转入后台监视，
本次会话未主动加 `&`/`setsid`/`disown`，等的是工具自己发的完成通知，不是自己起的后台）。每次调用前
`ps -o pid,etimes,pcpu,args -u "$(id -u)"` 查过 `qemu-system`、`vm-bench.sh`、
`e152-file-system-benchmark`、`fio`：全程零命中，不是性能测量在跑。

## AR-PTR（分配记录树，候选=仅子指针 86，13 格）

| 次序 | 目标文件 | 退出码 | 词数（`wc -w`） | oov-check.py | corruption-check.py | 通读复核 | 结论 |
|---|---|---|---|---|---|---|---|
| 1 | `m2-keyspace-r1-local-attack-ar-ptr-output-s1.md` | 0 | 613 | 绿，生词=0 拼接=0 | 绿，全部计数为 0（words=378） | 13/13 格齐全、按序、各带公式与「This would be refuted by」，无缺词、无断句、无孤立标点 | 干净 |
| 2 | `m2-keyspace-r1-local-attack-ar-ptr-output-s2.md` | 0 | 684 | 绿，生词=0 拼接=0 | 绿，全部计数为 0（words=454） | 13/13 格齐全、按序、各带公式与「This would be refuted by」，无缺词、无断句、无孤立标点 | 干净 |

两份干净样本达标，AR-PTR 到此为止，不再抽样。

## AR-KEY（分配记录树，候选=key10+子指针86=96，13 格）

| 次序 | 目标文件 | 退出码 | 词数（`wc -w`） | oov-check.py | corruption-check.py | 通读复核 | 结论 |
|---|---|---|---|---|---|---|---|
| 1 | `m2-keyspace-r1-local-attack-ar-key-output-s1.md` | 0 | 580 | 绿，生词=0 拼接=0 | 绿，全部计数为 0（words=342） | 13/13 格齐全、按序、各带公式与「This would be refuted by」，无缺词、无断句、无孤立标点 | 干净 |
| 2 | `m2-keyspace-r1-local-attack-ar-key-output-s2.md` | 0 | 549 | 绿，生词=0 拼接=0 | 绿，全部计数为 0（words=349） | 13/13 格齐全、按序、各带公式与「This would be refuted by」，无缺词、无断句、无孤立标点 | 干净 |

两份干净样本达标，AR-KEY 到此为止，不再抽样。第 2 次调用超过工具前台展示的 120 秒上限，
被工具自动转入后台监视（未主动加 `&`/`setsid`/`disown`），收到完成通知后照常核验，退出码 0。

## EX-PTR（extent 树，候选=仅子指针 86，非已定案，11 格）

| 次序 | 目标文件 | 退出码 | 词数（`wc -w`） | oov-check.py | corruption-check.py | 通读复核 | 结论 |
|---|---|---|---|---|---|---|---|
| 1 | `m2-keyspace-r1-local-attack-ex-ptr-output-s1.md` | 0 | 471 | 绿，生词=0 拼接=0 | 绿，全部计数为 0（words=300） | 11/11 格齐全、按序、各带公式与「This would be refuted by」，无缺词、无断句、无孤立标点 | 干净 |
| 2 | `m2-keyspace-r1-local-attack-ex-ptr-output-s2.md` | 0 | 690 | 绿，生词=0 拼接=0 | 绿，全部计数为 0（words=473） | 11/11 格齐全、按序、各带公式与「This would be refuted by」，无缺词、无断句、无孤立标点 | 干净 |

两份干净样本达标，EX-PTR 到此为止，不再抽样。

## EX-KEY（extent 树，已定案=key24+子指针86=110，11 格）

| 次序 | 目标文件 | 退出码 | 词数（`wc -w`） | oov-check.py | corruption-check.py | 通读复核 | 结论 |
|---|---|---|---|---|---|---|---|
| 1 | `m2-keyspace-r1-local-attack-ex-key-output-s1.md` | 0 | 514 | 绿，生词=0 拼接=0 | 绿，全部计数为 0（words=343） | 11/11 格齐全、按序、各带公式与「This would be refuted by」，无缺词、无断句、无孤立标点 | 干净 |
| 2 | `m2-keyspace-r1-local-attack-ex-key-output-s2.md` | 0 | 424 | 绿，生词=0 拼接=0 | 绿，全部计数为 0（words=342） | 11 个量全部齐全、无缺词、无断句、无孤立标点；格式上把 n_1mib/n_1gib/n_1tib 三个量各自并进了同一段落的 h_1mib/h_1gib/h_1tib 段里作答，未各自另起一个独立编号段落（提示 FORMAT RULES 要求 11 个独立编号答案），但对应数值（1、232、235578）均以完整语句写出，不是缺词或断句 | 干净（内容无损坏；格式上三对量合并成段，如上，照实记录，不判断是否影响可用性——那是主 agent 的事） |

两份干净样本达标，EX-KEY 到此为止，不再抽样。

## 干净样本清点（合计）

- AR-PTR：2 份干净（s1、s2）。AR-KEY：2 份干净（s1、s2）。EX-PTR：2 份干净（s1、s2）。EX-KEY：2 份干净
  （s1、s2）。四份文件合计 8 份干净样本，每份文件各自都在自己的「连续五次调用」额度之内（各只用了
  2 次，一次未判红作废）。
- 全部调用累计 8 次，退出码全部为 0；没有退出码 5（字词损坏闸判红），没有 void 副本，没有非 0 非 5
  的退出码，没有网关不通的情形。
- 目录核对：`ls research/prompts/ | grep "^m2-keyspace-r1-local-attack"` 显示 4 份提示、8 份对应的
  `-output-sN.md`、1 份核对表，与本记录逐项一致，无遗漏、无多余（见上一条命令的原样输出，已在写这份
  记录前跑过）。
- 本地腿全程未缺席：8 次调用全部退出码 0，corruption-check.py 与 oov-check.py 两个检测器全部跑到、
  全部判绿；无需设 `ASK_LOCAL_ALLOW_CORRUPT`。

## 没做什么

- 不解读、不总结、不采纳 8 份样本里对 AR-PTR/AR-KEY/EX-PTR/EX-KEY 各 11 或 13 格算术的具体数值方向
  是否一致，不判两次抽样之间、四份候选之间是否一致——那是主 agent 的事；这份记录只报退出码、词数、
  生词清单、干净/带损坏/作废，不报样本答了什么。
- 未跑云端攻方腿（Opus，S1、S2、S4）或云端正推腿（Sonnet，S3、S5，及 S1/S2/S4 各一句），不在这条腿的
  任务范围内，按分工表不读、不碰。
- 未碰 S3、S5（节点自描述、与别的结构的咬合）；S1、S2、S4（几何设计判断、崩溃历史、写序）按分工表
  归云端攻方与云端正推，本文件提示里也明令模型只答算术，不答设计判断。
- 未读这一轮的正文/背景材料之外的其它文件（如 `_m2-keyspace-r1-appendix.md`、
  `_m2-keyspace-r1-checklist.md`、`m2-keyspace-r1-snapshot/`），未读主 agent 这一轮的核实、同时在飞
  的别的轮次；本轮派发提示未给出明确的「禁读清单」，本报告据此按「这一轮未特别禁读」处理，仅读了
  完成本任务必需的文件（`_m2-keyspace-r1-body.md`、`crates/singlefs-format/src/lib.rs`、
  `crates/singlefs-core/src/unit.rs`、`crates/singlefs-core/src/allocator.rs`、
  `.claude/kb/decisions/08-核心索引结构.md`、既有本地攻方提示先例
  `m2-treesplit-r1-local-attack*.md`）。
- 未通读 `_m2-keyspace-r1-background.md` 全文（1526 行），理由见
  `m2-keyspace-r1-local-attack-translation-audit.md`「没做什么」一节。
