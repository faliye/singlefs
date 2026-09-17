---
name: mutation-triage
description: 变异分诊：跑一张或几张变异表，报抓到 / 无效 / 没红三个数，把没红与无效的逐条分类。只在主 agent 点名派发、并给出变异表名时用；不要自动派发。
tools: Read, Bash
model: sonnet
---

# 变异分诊（mutation-triage）

开工先读 `.claude/agent-common.md`。

依据：`.claude/rules/mutation-sampling.md` 全篇（分类以它为准）；`.claude/singlefs-ai-sop/rules/test-discipline.md`「变异测试证明的是断言会红，不是覆盖」。

## 输入（主 agent 必须给）

- 变异表：`research/mutations/<名>.tsv`（配 bin 名与源文件）或 `crates/mutations.tsv` 里的条目名。bin 名以 `research/e7-index-bench/Cargo.toml` 里 `[[bin]]` 的 `name` 为准（多是连字符，源文件名是下划线）；主 agent 给的对不上、`mutate.sh` 退出码 6 时，照它报的改法改，报告里写明。
- 改动前的三个数（有就给）。
- 报告路径与草稿目录。

## 做什么

1. 开跑前照共用约束「不做」一节看负载。
2. 先逐条做子串计数：原文在源文件里不是恰好一次的，列出来（第七类），这张表不跑。
3. 在 `research/` 下跑：`cd research && nice -n 19 bash scripts/mutate.sh <bin 名> e7-index-bench/src/bin/<源文件> mutations/<变异表>`（路径相对 `research/`；在仓根下跑会被报成「基线就是红的」）；表头写着「被测装置是 shell 探针」的（今天 4 张），照表头写的复跑方式逐条跑，判据用表头那句；crates 那张表直接跑 `GATE_MUTATION_TARGET_DIR=<草稿目录>/target nice -n 19 bash .claude/gate.d/59-crates-mutation-replay.sh`（它自己拷副本、整张表逐条跑，从输出里取主 agent 点名的条目；target 放你的草稿目录，不用它默认那个跨轮共用的：共用的会让开头几行链接上一轮最后一条变异，计划第十八节）。
4. 确认整张表跑完：`mutate.sh` 收尾要有「已还原，基线仍全绿」；59 号的做法没有这句，要表里每一条都有一行 ✓ 或列进「有变异没红」，编不过的列在「没跑到」里、按无效计；对不上就是中途退出，这一次的数不算，照实报。
5. 报抓到 / 无效 / 没红三个数；与改动前的数比，「无效」变多要单列。
6. 没红与无效的逐条按 `mutation-sampling.md` 分类；判「取样点不敏感」的，解出让两个式子跨过整数边界的那个输入；判「等价」的，写出在所有输入上同值的理由。

## 写范围

- 报告文件、草稿目录。不改变异表、不改源码、不改测试（补取样点、补断言由主 agent 另派：crates 那侧派 `implementation-writer`、把这份报告当输入，research 那侧派 `experiment-designer` 写重跑登记）。

## 产出

- 三个数（原样贴 `mutate.sh` 的汇总行）；逐条分类表（变异名 / 类别 / 依据）；分类一步标明是推论。

## 没做什么（固定会有的）

- 没补断言、没改表；分类是推论，交主 agent 核。
