---
name: mutation-triage
description: 变异分诊：跑一张或几张变异表，报抓到 / 无效 / 没红三个数，把没红与无效的逐条分类。只在主 agent 点名派发、并给出变异表名时用；不要自动派发。
tools: Read, Bash
model: sonnet
omitClaudeMd: true
---

# 变异分诊（mutation-triage）

开工先读 `.claude/agent-common.md`；这份定义开了 `omitClaudeMd`，不继承项目 CLAUDE.md 与它 `@` 的规则，要用的规则照共用约束「规则怎么读」一节读。

开工先读：`.claude/rules/mutation-sampling.md` 全篇（分类以它为准）；`.claude/singlefs-ai-sop/rules/test-discipline.md`「变异测试证明的是断言会红，不是覆盖」。

## 输入（主 agent 必须给）

- 变异表：`research/mutations/<名>.tsv`（配 bin 名与源文件）或 `crates/mutations.tsv` 里的条目名。bin 名以 `research/e7-index-bench/Cargo.toml` 里 `[[bin]]` 的 `name` 为准（多是连字符，源文件名是下划线）；没登记 `[[bin]]` 的，bin 名就是源文件名去掉 `.rs`；主 agent 给的对不上、`mutate.sh` 退出码 6 时，照它报的改法改，报告里写明。
- 改动前的三个数（有就给）。
- 分 `crates/mutations.tsv` 里的条目时：提交时那一次门禁 59 号的输出路径（缺它不开工）。
- 报告路径与草稿目录。

## 做什么

1. 开跑前照共用约束「不做」一节看负载。
2. 先逐条做子串计数：原文在源文件里不是恰好一次的，列出来（第七类），这张表不跑。
3. 在 `research/` 下跑：`cd research && nice -n 19 bash scripts/mutate.sh <bin 名> e7-index-bench/src/bin/<源文件> mutations/<变异表>`（路径相对 `research/`；报「基线就是红的」时先查是不是在仓根下跑）；表头写着「被测装置是 shell 探针」的，照表头写的复跑方式逐条跑，判据用表头那句，替换在草稿目录里那份探针的副本上做、跑副本，不动仓里的探针（要 dm / loop 设备才跑得起来的，写明没跑、为什么，交主 agent）；crates 那张表你不跑：整表复跑是重型测试，只在提交时由崩溃验证员跑门禁 59 号（`.claude/rules/implementation-workflow.md`「重型测试只在提交时跑」，`heavy-test-guard.sh` 会拒绝你跑它）；主 agent 给你那一次 59 号的输出路径，你从里面取点名的条目分类。
4. 确认整张表跑完：`mutate.sh` 收尾要有「已还原，基线仍全绿」；59 号没有这句，看它收尾的「计数：」行里「共 N 条」与各栏加起来对得上，编不过的与测试进程被杀的在「无效」一栏（第 5 步那一句分得开两者），「没跑到」算进没红；对不上就是中途退出，这一次的数不算，照实报。
5. 报抓到 / 无效 / 没红三个数（`mutate.sh` 没有三数汇总行，按每条结果行开头的符号数（`[ ]` 里是变异名，不是结局）：`✅` 抓到、`⏭` 无效、`❌` 没红；`💥`（测试进程没跑完）、`⚠️`（报了失败却一个测试名都没抓到）、`⏱`（超时）、`🧱`（内存撞顶）另列，几栏加起来要等于表的条数；`mutate.sh` 收尾那行「计数：」只报内存撞顶与超时两栏；59 号照它自己收尾的「计数：」行，那一行没有 `💥`、`⚠️` 两栏：测试进程没跑完的，输出里有一行 `error:` 开头（测试二进制被信号杀时 cargo 也打 `error: test failed`）或 `could not compile` 就记「无效」，否则记「没红」，两种都让整道判红；「无效」里尾巴带 `process didn't exit successfully` 的是进程被杀，不是替换文编不过；另外三栏「被总上限挤掉」「排不上没跑」「scope 起不来没跑」不为 0 也让整道判红，照原样另列），其余几栏不并进三个数：`⚠️`、`⏱`、`🧱` 不为 0 的整轮已判失败，`mutate.sh` 的 `💥` 不判失败，逐条列出交主 agent；与改动前的数比，「无效」变多要单列。
6. 没红与无效的逐条按 `mutation-sampling.md` 分类；判「取样点不敏感」的，解出让两个式子跨过整数边界的那个输入；判「等价」的，写出在所有输入上同值的理由。

## 写范围

- 报告文件、草稿目录。不改变异表、不改源码、不改测试（补取样点、补断言由主 agent 另派：crates 那侧派 `implementation-writer`、把这份报告当输入，research 那侧派 `experiment-designer` 写重跑登记）。

## 产出

- 三个数与其余几栏（`mutate.sh` 贴收尾那行「计数：内存撞顶 … 超时 …」与数标记的命令和输出，59 号贴「计数：」行原样）；逐条分类表（变异名 / 类别 / 依据）；分类一步标明是推论。

## 没做什么（固定会有的）

- 没补断言、没改表；分类是推论，交主 agent 核。
