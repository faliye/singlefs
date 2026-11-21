# 旧账本开着项当场修的治理文件改动：治理文件轮第三轮（最后一轮），攻第二轮判决的修补（ledger-open-fixes-governance-r3，2026-11-19）

这一轮攻的是第二轮判决（`research/prompts/ledger-open-fixes-governance-r2-main-verification.md`）「三、改法」六条的修补，都已换上主工作区。这一轮是最后一轮，这一轮的修补在判决里标零轮，不再开第四轮。方案按 `crates/` 今天的实现来谈。前两轮攻过而没打中的角度不重复：同一台判持有者的四样、出口放锁、`load_change_range` 的值与状态、挪场景之后 `s` 的赋值、三份入口只差实验号、入口脚本那一节不该守、两份锁反序互等、`--dry-run` 不占按仓的锁、`_lib_changed_paths_run_git` 成功时照打警告让样本档判错、承接的子 agent 被前一个写坏的行拦。

## 问题

1. 第二轮每一处修补做的是不是判决那一条说的事，修没修到位，有没有带进新的缺口（攻方腿）。
2. 第二轮修补里偏离判决字面的三处，偏得对不对；修补的自证、样本与开关证明的是不是名字说的事（正推腿）。

## 实现今天的样子

主 agent 的观测（读的是换上之后主工作区的文件；读数取自各批草稿目录 `main-agent-logs/` 下的任务日志）：

- 这一轮没有改 `crates/` 下的代码。与 `crates/` 有交点的是 doc-experiments 两格：evidence-in-repo 开始认 `crates/*/src/bin/` 下以 `e<号>_` 起头的装置，pinned-apparatus-revision 从起跑脚本认出 `crates/*/src/bin/` 下的装置源码。
- **换上工具 `.taking`**（判决第 1 条，工具员报告 `~/.local/share/singlefs/work/open-fixes-r2-fix-taking/result.md`）：`.taking` 里写持有者身份（与占用记录同一套写法），判它挡不挡照记录那一套（同一台看进程、另一台看修改时间，过期 360 秒）；摘的那一趟登记进 `HELD_REMOVAL_CLAIMS`，被 TERM 或 HUP 杀掉时由 `release_then_die` 放掉；建 `.taking` 的 `link` 报已有而读回来是自己写的，当放成。新加四格自证（KILL 留下、TERM 当中、link 回复丢了、还占着时点名），四条新开关、七条旧开关的红格名单加了新格。读数：副本里换上前那一版加新自证恰好红新四格；18 条开关减去 16 格副本缺文件的环境格之后逐条红在文件头点名的格；攻方腿探针在新版上 `taking-left-by-kill-repository-lock … next_swap_exit=0 seal_exit=2 … next_killed_holder_then_swap_exit=0`、TERM 臂 `taking_left=False`、回复丢了臂 `first_held=True taking_left=False`，基线版上照旧 `next_swap_exit=10 seal_exit=10`、`taking_left=True`；换上时验证命令 `python3 scripts/gate/hook-swap.py --selftest` 退 0。
- **evidence-in-repo**（判决第 2 条，报告 `~/.local/share/singlefs/work/open-fixes-r2-fix-experiments-gate/result.md`）：有 timing 行的实验，比修改时刻的对象改成每一条 timing 行点名的产物（只存着 `.gz` 的也认），每一份都要在盘、不比装置旧；exact 行的产物与别的 `e<号>` 文件不再作保；`crates/*/src/bin/e<号>_*.rs` 与目录形装置一并认。判决写「那一份」，工具员取「每一条都要」。
- **pinned-apparatus-revision**（判决第 4 条，同一份报告）：触发改成钉值变了或装置变了；登记认脚本里写「跑前登记 <路径>」那一处（三份 E185 起跑脚本是文件头注释那一行）；钉值行允许行尾注释；登记不在树里时取基准那一版，再取 git 历史里它被删前的那一版；装置变了的，第十二节要新加写着新钉值前 12 位的行，且这 12 位基准那一版里没有，旧钉值的行一行不能少。**偏离判决字面**：装置没变、只改钉值的，第十二节里写着新钉值前 12 位即可，不要求是新加的行（主 agent 收了：r4 钉值改回登记第十二节本来写着的值，照字面要为它补一行没有内容的修订）。登记路径写错与历史里从没有过这份文件的，与「已归档」分不开，照旧不判。
- doc-experiments 两格的读数：19 份新样本（`.claude/gate.d/fixtures/doc-experiments.sh/` 下 `evidence-in-repo-crates-bin-*`、`-exact-only-rerun-red`、`-timing-rows-*`、`-unrelated-file-red`，`pinned-apparatus-revision-` 起头的十三份）打上并暂存；关着开关样本档 136 份全对；18 条开关（新加 11 条、名单变了 5 条、共用库两条）逐条判错的样本与文件头点名的相等；攻方腿两份探针九个样本全红；真仓 pinned-apparatus-revision 绿（三份 E185 起跑脚本：「装置没变而钉值改了的 3 份，登记第十二节都新加了写着新钉值的行」，r2、r3 取自 git 历史 `a24379563928` 里被删之前的那一版），evidence-in-repo 红在 E158、E180、E202（装置今天改过、产物没有或实验页缺那一句），三处归改那三份装置的会话，提交前它弄绿。
- **删旧闸**（判决第 3 条）：四份入口 `e07`、`e53`、`e159`、`e56-vm-bench-run.sh` 两条 git 的路径加 `:(literal)`；攻方腿探针在新版上八臂都 `exit=2 untracked_output_kept=true refused_by_delete_gate=true reached_build=false`、提交了的那份原样；开关 `existing-output-deleted-unchecked` 下多红的方括号那一格补进了点名，四份开关复跑 15、16、16、15 条逐格相同。
- **完工清理闸**（判决第 5 条）：下层 `.deliverables` 已声明交接的副本，上层子 agent 交回时算已声明；新加三格 `nested-lower-handover-main-listed`、`nested-undeclared-copy-blocks`、`nested-lower-handover-passes`，补进三条开关的点名。读数：攻方腿探针五臂里上层不再被下层交接副本拦（`nested-upper-blocked-by-lower-handover-copy action=pass`），承接者被前一个写坏的行拦照旧（第二轮不采那一条）；换上后自证「判了 106 格；77 个弄坏开关各红在点名的格」，开关复跑 77 条逐格相同。
- **stage-owners**（判决第 6 条与跟着新判法改的两行）：第 143 行改成封存的说法；第 117、130 行的「为什么」照 doc-experiments 两格新判法改；单格核 stage-owners「17 道门禁、195 格」绿。
- 没修到、不归这一轮的：doc-experiments 真仓 evidence-in-repo 的三处红（归改装置的会话）。

## 范围

这一轮被判的文件（都读主工作区此刻那一版；要看改了哪几行，对着 HEAD 跑 `git diff HEAD -- <文件>`，新样本整份都是改动）：

- `scripts/gate/hook-swap.py`
- `.claude/gate.d/doc-experiments.sh`，与 `.claude/gate.d/fixtures/doc-experiments.sh/` 下这一轮新加的 19 份样本
- `.claude/hooks/completion-cleanup-guard.sh`
- `scripts/experiment-apparatus/e07-vm-bench-run.sh`、`e53-vm-bench-run.sh`、`e159-vm-bench-run.sh`、`e56-vm-bench-run.sh`
- `.claude/gate.d/stage-owners.tsv`（第 117、130、143 行）

## 要腿判的

攻方腿（每问回「成立」「不成立」「推不出」，推不出的算打中；打中的给能照喂的输入或 文件:行）：

1. 换上工具 `.taking`：两趟同时摘同一份已死的 `.taking`（工具员自己列进文件头「留下的缝」那一种）后果多大，有没有一种交错让两趟都以为自己摘成、都去删记录；`release_then_die` 放 `.taking` 与放草稿目录记录的次序，被 TERM 在两者之间时留下什么；另一台起的 `.taking` 过期 360 秒，与记录的过期判法是不是同一个数、同一个时钟。
2. evidence-in-repo「每一条 timing 行都要」：一个实验几条 timing 行、只重跑其中改到的那一段时，出路能不能走通（给得出一条按出路做完就绿的路径，或给不出）；`crates/*/src/bin/` 下目录形装置与 `e<号>_` 起头而不是装置的二进制各判成什么。
3. pinned-apparatus-revision 的放宽：只改钉值时「第十二节里写着新钉值前 12 位即可」，给得出一种钉回一个已被后续修订作废的旧值也放行的写法吗；「登记路径写错」与「已归档」分不开，给一个今天就会被当成「不判」的真实写错；「旧钉值的行一行不能少」与「整行抄一行已有的不算」两条判据之间有没有一种写法两条都过而其实没修订。
4. 删旧闸 `:(literal)`：`git` 的别的路径解法（大小写不敏感的文件系统、`core.ignorecase`、带 `:` 起头的文件名）还能不能让一份没进 git 的输出被当成在 git 里；四份入口这两条之外有没有第三处按 pathspec 解输出路径。
5. 完工清理闸「下层已声明交接的副本，上层交回时算已声明」：下层是别的会话、别的 agent 的目录时上层也认它的声明吗；下层 `.deliverables` 写坏、被删、换成符号链接时上层判什么。

正推腿（逐条回「成立」「不成立」「推不出」）：

6. 第二轮「三、改法」六条，逐条核今天的代码做的是不是那一条说的事，写出代码里的 文件:行 与判决那一句的原文。
7. 偏离判决字面的三处（evidence-in-repo 取「每一条都要」、钉值格只改钉值时的放宽、换上工具的新格名单是按读码推的）：逐处判偏得对不对，偏错的写该照哪个字面。
8. 新加的自证格、样本与开关：每一样红的地方是不是它名字说的那件事（给一份样本或一格自证，说清它红在哪一行判法上）。

## 分工

攻方腿做第 1 到 5 问，正推腿做第 6 到 8 问。两条腿都读「范围」里的文件；只有一条腿读到的范围在报告里写明。

### 小节清单：`.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## E185 GC一批搬迁的挂钟拆分 —— 第 2 次重跑已跑（占比 0.25 的几何改负载后造成了，单点核与第一、二段跑完；问题单四行都够判：第 1、2 行读这一趟，第 3、4 行沿用第 1 次重跑的产物；第三、四段够判后不跑） | 抄 | 顶上标题，与开篇正文合成行区间 |
| （E185 标题之下、第一个下级标题之前的正文：第 2-51 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 几何：f = 0.25 两点造成了（改法甲） | 抄 | 默认全抄（脚本生成） |
| ### 问题单第 1 行够判的集合 S | 抄 | 默认全抄（脚本生成） |
| ### 各量的判定（第 2 次重跑） | 抄 | 默认全抄（脚本生成） |
| ### 沿用第 1 次重跑的量（问题单第 3、4 行；这一趟第三、四段不跑） | 抄 | 默认全抄（脚本生成） |
| ### 阳性对照、自检与失败条款 | 抄 | 默认全抄（脚本生成） |
| ### 校准与机器状态读数（登记第六节新加的量，读数 1–N15） | 抄 | 默认全抄（脚本生成） |
| ### 问题单逐行（岔路表） | 抄 | 默认全抄（脚本生成） |
| ### 做了什么 | 抄 | 默认全抄（脚本生成） |
| ### 复跑（第 2 次重跑；命令照重跑登记第五节「跑法」，起法在 `scripts/experiment-apparatus/e185-r2-segment-run.sh` 文件头） | 抄 | 默认全抄（脚本生成） |
| ### 口径 | 抄 | 默认全抄（脚本生成） |
| ### 它答不了的 | 抄 | 默认全抄（脚本生成） |
| ### 路径与结论登记 | 抄 | 默认全抄（脚本生成） |
| ### 影响的决策 | 抄 | 默认全抄（脚本生成） |
| ## 历史版本 | 不抄 | 历史节，正文只引现状 |

### 小节清单：`.claude/kb/experiments/158-择根与修复四岔路.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## E158 择根与修复四岔路 —— 第 5 次重跑已跑（够判）：四档两臂与一线程对拍已出最终产物，第 4 次跑已交（岔路 1、2 用户已定，3 记欠） | 抄 | 顶上标题，与开篇正文合成行区间 |
| （E158 标题之下、第一个下级标题之前的正文：第 2-26 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 2026-11-12：第 5 次重跑 S4 一档两臂（每臂 8 片加 1 次只取库的合并），四档齐 | 抄 | 默认全抄（脚本生成） |
| ### 2026-11-11：第 5 次重跑（重跑登记 `e158-r5-prereg.md`）：被抛弃根的账读不出，枚举型覆盖补回，G0、S16、小环三档两臂与一线程对拍的最终产物 | 抄 | 默认全抄（脚本生成） |
| ### 2026-11-09：岔路单第 3 行换成如实账本重出，岔路 1 的 q1 三行删除 | 抄 | 默认全抄（脚本生成） |
| ### 2026-09-27：第 4 次跑（重跑登记 `e158-r4-prereg.md`） | 抄 | 默认全抄（脚本生成） |
| ### 2026-09-26：第 2 次跑第一段（重跑登记 `e158-r2-prereg.md`） | 抄 | 默认全抄（脚本生成） |
| ### 这一段做了什么 | 抄 | 默认全抄（脚本生成） |
| ### 2026-09-24：第一段重跑（产物结构性对不上，原因已诊断） | 抄 | 默认全抄（脚本生成） |
| ### 结果整行抄自产物（产物已移出工作区的行写成一行指针） | 抄 | 默认全抄（脚本生成） |
| ### 复跑 | 抄 | 默认全抄（脚本生成） |
| ### 它答不了的 | 抄 | 默认全抄（脚本生成） |
| ### 2026-09-25（续，session s10）：op1 第三种变体（岔路单第 1 行）；`replay.sh` 全量重出；发现另一条并行线正在改的 `crates/` 影响岔路单第 3 行的前提 | 抄 | 默认全抄（脚本生成） |
| ### 影响的决策 | 抄 | 默认全抄（脚本生成） |
| ## 历史版本 | 不抄 | 历史节，正文只引现状 |
| ### 2026-09-23 | 不抄 | 历史节的子节 |
| ### 2026-09-24 | 不抄 | 历史节的子节 |
| ### 2026-09-24（续，session s3，接续一个撞了会话限额而中断的执行员） | 不抄 | 历史节的子节 |
| ### 2026-09-24（续，session s3 第二段，主 agent 消息「接着做，这一段还没交出岔路单要的数，不停」） | 不抄 | 历史节的子节 |
| ### 2026-09-24（续，session s4） | 不抄 | 历史节的子节 |
| ### 2026-09-24（续，session s5，派发要求先做完 PC2 阳性对照才往下） | 不抄 | 历史节的子节 |
| ### 2026-09-24（续，session s6） | 不抄 | 历史节的子节 |
| ### 2026-09-24（续，session s7） | 不抄 | 历史节的子节 |
| ### 2026-09-24（续，session s8） | 不抄 | 历史节的子节 |
| ### 2026-09-26（第 2 次跑第一段） | 不抄 | 历史节的子节 |
| ### 2026-09-27（第 3 次跑第一段与第 4 次跑） | 不抄 | 历史节的子节 |
| ### 2026-11-03 | 不抄 | 历史节的子节 |
| ### 2026-11-09 | 不抄 | 历史节的子节 |
| ### 2026-11-11 | 不抄 | 历史节的子节 |
| ### 2026-11-12 | 不抄 | 历史节的子节 |

### 小节清单：`.claude/kb/experiments/180-准入读数与真可写量.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## E180 准入读数与真可写量 —— 未跑（第一段读数段装置、45 条单测、22 条变异行与分片长跑脚本写成，读数长跑两趟都没跑完、没有产物；第二段扣住位段装置没写） | 抄 | 顶上标题，与开篇正文合成行区间 |
| （E180 标题之下、第一个下级标题之前的正文：第 2-20 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 测什么 | 抄 | 默认全抄（脚本生成） |
| ### 判据（登记第六节，跑前写死） | 抄 | 默认全抄（脚本生成） |
| ### 失败条款（登记第十节） | 抄 | 默认全抄（脚本生成） |
| ### 口径 | 抄 | 默认全抄（脚本生成） |
| ### 复跑 | 抄 | 默认全抄（脚本生成） |
| ### 它答不了的 | 抄 | 默认全抄（脚本生成） |
| ### 影响的决策 | 抄 | 默认全抄（脚本生成） |
| ## 历史版本 | 不抄 | 历史节，正文只引现状 |

### 小节清单：`.claude/kb/experiments/202-GPU与CPU两种模式GC对照组.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## E202 GPU与CPU两种模式GC对照组 —— 跑前登记已写、装置未跑 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （E202 标题之下、第一个下级标题之前的正文：第 2-8 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 现状 | 抄 | 默认全抄（脚本生成） |
| ### 复跑 | 抄 | 默认全抄（脚本生成） |
| ### 口径 | 抄 | 默认全抄（脚本生成） |
| ### 判据与失败条款 | 抄 | 默认全抄（脚本生成） |
| ### 影响的决策 | 抄 | 默认全抄（脚本生成） |
| ## 历史版本 | 不抄 | 历史节，正文只引现状 |
| ### 2026-11-13 | 不抄 | 历史节的子节 |
| ### 2026-11-03 | 不抄 | 历史节的子节 |

### 小节清单：`.claude/kb/tooling.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| # 工具与环境事实 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （工具与环境事实 标题之下、第一个下级标题之前的正文：第 2-7 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ## 本地 LLM（三方论证不派它当腿） | 抄 | 默认全抄（脚本生成） |
| ## 仓里的性能数都只在本机成立，引用要带机器口径 | 抄 | 默认全抄（脚本生成） |
| ## 跑项目代码的命令一律经 task-server 交 | 抄 | 默认全抄（脚本生成） |
| ## 变异工具：并发不超过核数，不限时，卡住由统一任务服务判 | 抄 | 默认全抄（脚本生成） |
| ## 变异条数会随源码漂，不许在 kb 里钉死 | 抄 | 默认全抄（脚本生成） |
| ## 改了引用方，要 grep 全仓改被引用方 | 抄 | 默认全抄（脚本生成） |
| ## 一个数不许从「它自己判为非法」的配置里算出来 | 抄 | 默认全抄（脚本生成） |
| ## 未定项一律写成编号条目，标题一律声明条数 | 抄 | 默认全抄（脚本生成） |
| ## 验证产物放哪：四个位置，各管一段 | 抄 | 默认全抄（脚本生成） |
| ## 内核源码：本机有完整树，不必 WebFetch | 抄 | 默认全抄（脚本生成） |
| ## 到点自己开工：用 remind.sh 登记 | 抄 | 默认全抄（脚本生成） |
| ## Rust 工具链 | 抄 | 默认全抄（脚本生成） |
| ## RocksDB 与 snappy 的预编库：每台主机编一次，取值只走进程环境 | 抄 | 默认全抄（脚本生成） |
| ## 虚机 benchmark harness（E7（离线索引 harness） 用） | 抄 | 默认全抄（脚本生成） |
| ## 造几何量不同的设备：不需要真硬件 | 抄 | 默认全抄（脚本生成） |
| ### QEMU 路线（推荐：无需 root，直接接现有 harness） | 抄 | 随父节带到 |
| ### 宿主侧路线（要 root，胜在不用启虚机） | 抄 | 随父节带到 |
| ## `scripts/mutation/mutate.sh` 的结局分类 | 抄 | 默认全抄（脚本生成） |
| ## 几个会话共用一个仓时怎么暂存与提交 | 抄 | 默认全抄（脚本生成） |
| ## 复跑与变异脚本要在 `scripts/experiment-apparatus/` 下跑 | 抄 | 默认全抄（脚本生成） |
| ## 批量立 / 定分项时门禁会红的形态 | 抄 | 默认全抄（脚本生成） |
| ## 本机 Claude Code 会话里的 `grep` 与 `find` 不是 GNU 那两个 | 抄 | 默认全抄（脚本生成） |

**出处 `.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md:1-51`（整段抄，未转述）**

````markdown
## E185 GC一批搬迁的挂钟拆分 —— 第 2 次重跑已跑（占比 0.25 的几何改负载后造成了，单点核与第一、二段跑完；问题单四行都够判：第 1、2 行读这一趟，第 3、4 行沿用第 1 次重跑的产物；第三、四段够判后不跑）

**中文** · [English](185-wall-clock-breakdown-of-a-batch-of-gc-migrations.en.md) · [日本語](185-GC-一バッチ分の移動の壁時計分解.ja.md)

<!-- doc-lint:not-numbers Q1 Q2 Q3 Q4 Q5 Q6 Q7 Q8 Q9 Q10 Q11 Q12 Q13 Q14 Q15 Q16 Q17 G1 G2 G3 G4 G5 G6 V1 V15 V16 V21 V22 V23 V26 V27 V30 V31 V33 V34 V55 V56 V94 F1 F2 F3 F4 F5 F7 F8 N4 N5 N6 N7 N8 N9 N10 N11 N12 N13 N14 N15 H1 H2 H3 H4 H5 H6 H7 H8 R2 -->

跑前登记 `research/prompts/e185-preregistration.md`（四段：第一段 `wall-clock`、第二段 `stage-split`、第三段 `gpu-submission`、第四段 `foreground-fsync`，第四段附带、够判后不跑）。回答问题单 `research/prompts/m4-gpugc-measure-forks.md` 的四行：一批搬迁的挂钟受盘还是 CPU 限、GPU 上算 GC 的哪几段（甲 只算校验和核对 ／ 乙 加区域扫描里的解头与映射反查 ／ 丙 再加选段打分）、攒批形态（一批一交 ／ 几批攒起来一次交 ／ 放宽 R）、定时兜底间隔与写带宽上限的默认值从什么量取。被测的是 `crates/` 今天的 GC 路径 `MountedSession::run_compaction`；背景是欠账 C412（整理的挂钟代价没量过），被测条款是 D26（后台整理与放置回收） 已定项 13、已定项 1 与 D24（后台重活能不能卸给 GPU） 已定项 3 的判据形式。

正文是第 2 次重跑的结果：重跑登记 `research/prompts/e185-r2-prereg.md`（sha256 前缀 `7d74c089`，写于 2026-10-23，装置改之前、这一趟的产物一份都还没跑）。起因：第 1 次重跑里 f = 0.25 两点几何没造成，问题单第 1、2 行按字面不够判；第一段里还有一段连成一片的慢段（同一份活 CPU 涨到 1.13–2.5 倍，作废后当场重试的那条 2.316 倍被收下，登记第四节第 5 行，第 161 行）。登记改了三样：负载（改法甲：只换造池第 2 步覆盖写的次序，让占比 0.25 的几何造得出）、装置的读数（每条意图前后的校准、线程与机器状态读数，造池之后逐段打成分）、作废与重试的做法（校准比接进「机器被别的活挤」，作废后先等机器回到常态再重试，第十一节修订 R2-1 至 R2-4，原判据原样保留）；另有执行员在单测之前、产物之前写的修订一（R2-5 至 R2-9，只收严或补量，登记第 415–425 行）。Q1–Q17 每一行的算法、门槛、判定照原登记（含第十二节 12.1–12.11）一个字不改。问题单第 3、4 行第 1 次重跑已够判，这一趟不再判，沿用第 1 次重跑的产物（岔路表写明哪几份）；第三、四段这一趟不跑。第 1 次重跑的数与七·6 前后的比法（重跑登记 `research/prompts/e185-r1-prereg.md`）已挪进变更史，正文只写这一趟。

装置是入库装置 `crates/singlefs-implementation-experiments/src/bin/e185_gc_batch_wall_clock_split/main.rs`（一个二进制、五个子命令：`wall-clock`、`stage-split`、`gpu-submission`、`foreground-fsync` 与这一趟新加的 `geometry-check`，只读驱动 `singlefs-core` 的公开入口，不改 `crates/` 里别的文件）。决定工作量、几何与判据门槛的常量都是本地常量，值抄自登记第五节常量表，每个子命令开跑先与 `crates/` 同名常量回比（停机条款第 1 条）；这一趟新加一个本地常量 `LOCAL_ROOT_RING_ENTRIES = 24`，回比 `singlefs_format::ROOT_RING_REGIONS × ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM`（装置第 131 行、第 178–180 行），只用来打机理核对那一栏。

**现状**：

- 装置现在是 sha256 `d312b9aa71d0518976fd383b48300705f4598d2e79d4a3d59488657e684a51eb`，26562 行，`grep -c '#\[test\]'` 对它 = 240；它在登记之后改过，登记第十二节没有修订行；三份起跑脚本的 `registered_apparatus_sha256` 仍各是自己登记第十二节写的值（第 2 次重跑 `0bd22743…`、第 3 次重跑 `4368c0e7…`、第 4 次重跑 `9e813cbd…`），与装置现算的不等：再起第 4 次重跑照停机 K-S6、第 3 次重跑照停机 J-S7 退 3，第 2 次重跑由装置里的停机条款第 7 条（修订 R2-8）退 3。第 2 次重跑那三趟用的是 `0bd2274383f12e88beab3f96f354148698cfcd2043263b5d5709e32e1946784d`，13187 行、118 条（改前基线是第 1 次重跑第三段用的 `eadb929d…`，10226 行、87 条）。三趟产物的 `name=apparatus` 行里，编进二进制的源码哈希与起跑脚本设给装置的登记哈希都是 `0bd22743…`（停机条款第 7 条与修订 R2-8 没触发），单点核两份与第二段同形，第一段如下（第一段第 2 行）：

```text
E7RESULT name=apparatus source_sha256=0bd2274383f12e88beab3f96f354148698cfcd2043263b5d5709e32e1946784d registered_sha256=0bd2274383f12e88beab3f96f354148698cfcd2043263b5d5709e32e1946784d
```

- 单测与变异（登记第十二节第 427–431 行，量的是主 agent 在一次性编译目录里跑的，这一页没重跑）：单测 `118 passed; 0 failed`，clippy 退 0；`crates/mutations.tsv` 里 E185（GC一批搬迁的挂钟拆分） `V` 起头的行现在 94 条（在 `crates/mutations.tsv` 里按该前缀 `grep -c` 输出 94）：V1–V15 照原登记第九节，V16–V55 是第 1 次重跑执行员补的，V56–V94 共 39 行是这一趟补的（V56–V76 登记第九节，V77–V94 修订一 R2-9），这 39 行的轻型变异全部抓到（登记第 431 行：计数「抓到 314 条、没红 0 条、无效 0 条」，39 行都在抓到里）；整表的三个数待提交时门禁 crates-mutation-replay。
- 这一趟的产物（都在 `research/results/`，行数用 `grep -c ''` 数，末行都是 `name=done`）：
  - 单点核：`research/results/e185-gc-batch-wall-clock-split-geometry-check-r2-2026-10-23.out`（125 行，sha256 `c47db8bf3a7841d65aa209c3a3fb4ba941e9c30ec68672cde1b0fc480c206e00`，末行 `E7RESULT name=done emitted=125`），对照 `research/results/e185-gc-batch-wall-clock-split-geometry-check-slot-order-r2-2026-10-23.out`（125 行，sha256 `0add58dbc58f0948d0b115d6575173494aca801550508c2d5d732ad38885a5c2`，末行 `E7RESULT name=done emitted=125`），输入清单 `research/results/e185-r2-inputs-geometry-check-2026-10-23.txt`，清场记录 `research/results/e185-r2-quiesce-geometry-check-2026-10-23.txt`。
  - 第一段：`research/results/e185-gc-batch-wall-clock-split-wall-clock-r2-2026-10-23.out`（2103 行，sha256 `18191ec3a8afbb27405987f99d3e36e5ae2538d3c9ab4f2717738495e767096b`，末行 `E7RESULT name=done emitted=2103`），输入清单 `research/results/e185-r2-inputs-wall-clock-2026-10-23.txt`，清场记录 `research/results/e185-r2-quiesce-wall-clock-2026-10-23.txt`。
  - 第二段：`research/results/e185-gc-batch-wall-clock-split-stage-split-r2-2026-10-23.out`（1803 行，sha256 `f70eb21b8a7081489dd8fc3209080f3ab83f16ac40beee3042c03f4756f1e07a`，末行 `E7RESULT name=done emitted=1803`），输入清单 `research/results/e185-r2-inputs-stage-split-2026-10-23.txt`，清场记录 `research/results/e185-r2-quiesce-stage-split-2026-10-23.txt`。
  - 沿用的（第 3、4 行，不重算）：第三段 `research/results/e185-gc-batch-wall-clock-split-gpu-submission-r1-2026-10-22.out`（sha256 `4041a1cac83e7d62ef8bbe1580468aa6b4e43d69d67e8e9e044dd885ab063408`）与它读的第 1 次重跑第一段 `research/results/e185-gc-batch-wall-clock-split-wall-clock-r1-2026-10-22.out`（sha256 `978ae7b5fe60ab8f5a87338b982e8989c4c0d0ba8f19994bfc8e1c39b3d437fd`），输入清单 `research/results/e185-r1-inputs-gpu-submission-2026-10-22.txt`、`research/results/e185-r1-inputs-2026-10-22.txt`。
  - 存档不引（第 1 次重跑与 2026-10-20 那一趟的产物，数在变更史）：`research/results/e185-gc-batch-wall-clock-split-stage-split-r1-2026-10-22.out`、`research/results/e185-r1-comparison-2026-10-22.out`（比法判决行 `output_void=false`，即那一份没作废）、`research/results/e185-gc-batch-wall-clock-split-wall-clock-2026-10-20.out`、`research/results/e185-gc-batch-wall-clock-split-stage-split-2026-10-20.out`。
  - 2026-10-24 第 3 次与第 4 次重跑产物（三方论证备料，出处见 `research/prompts/e185-r3-prereg.md` 与 `research/prompts/e185-r4-prereg.md`）：`research/results/e185-gc-batch-wall-clock-split-cpu-writings-r4-2026-10-24.out`、`research/results/e185-gc-batch-wall-clock-split-cpu-writings-r4-2026-10-24.out.err`、`research/results/e185-gc-batch-wall-clock-split-function-split-r3-2026-10-24.out`、`research/results/e185-gc-batch-wall-clock-split-function-split-r3-2026-10-24.out.err`、`research/results/e185-gc-batch-wall-clock-split-host-cpu-drift-control-r4-2026-10-24.out`、`research/results/e185-gc-batch-wall-clock-split-host-cpu-drift-control-r4-2026-10-24.out.err`、`research/results/e185-gc-batch-wall-clock-split-host-cpu-per-submission-r3-2026-10-24-2.out`、`research/results/e185-gc-batch-wall-clock-split-host-cpu-per-submission-r3-2026-10-24-2.out.err`、`research/results/e185-gc-batch-wall-clock-split-host-cpu-per-submission-r3-2026-10-24.out`、`research/results/e185-gc-batch-wall-clock-split-host-cpu-per-submission-r3-2026-10-24.out.err`、`research/results/e185-gc-batch-wall-clock-split-recompute-disagreement-r3-2026-10-24.out`、`research/results/e185-gc-batch-wall-clock-split-recompute-disagreement-r3-2026-10-24.out.err`、`research/results/e185-r3-inputs-function-split-2026-10-24.txt`、`research/results/e185-r3-inputs-host-cpu-per-submission-2026-10-24-2.txt`、`research/results/e185-r3-inputs-host-cpu-per-submission-2026-10-24.txt`、`research/results/e185-r3-inputs-recompute-disagreement-2026-10-24.txt`、`research/results/e185-r3-quiesce-host-cpu-per-submission-2026-10-24-2.txt`、`research/results/e185-r3-quiesce-host-cpu-per-submission-2026-10-24.txt`、`research/results/e185-r3-quiesce-recompute-disagreement-2026-10-24.txt`、`research/results/e185-r4-baseline-rejudge-2026-10-24.out.gz`、`research/results/e185-r4-inputs-cpu-writings-2026-10-24.txt`、`research/results/e185-r4-inputs-host-cpu-drift-control-2026-10-24.txt`、`research/results/e185-r4-quiesce-host-cpu-drift-control-2026-10-24.txt`、`research/results/e185-r4-serial-cpu-writings-2026-10-24.txt`、`research/results/e185-r4-serial-host-cpu-drift-control-2026-10-24.txt`。
- 跑在哪：单点核、第一段、第二段在第二台先后跑，不同时跑，真盘臂的镜像放在同一块盘的 ext4 上（输入清单第 22–26 行：型号 CT2000T500SSD8，逻辑块 512，文件系统 ext4；第一段配置行 `image_directory_filesystem=ext4 image_directory_is_cleared_at_boot=false image_device=64512`），不带 `gpu` 特性编（三份输入清单里 `grep -c lookup-work-counters` 都输出 0），外层内存上限 46G、`E185_MEMORY_CAP=40G`、装置线程 31（第一段配置行 `worker_threads=31`）。三份输入清单第 1–11 行（装置、`transaction.rs`、`mounted_session.rs`、`region_scan.rs`、`compaction_plan.rs`、`unit.rs`、`checksum.rs`、`block_device.rs`、`allocator.rs`、`singlefs-format/src/lib.rs`、`Cargo.lock` 的 sha256）逐行相同（`diff` 核过），第 12 行是各自拷树后的提交号。每趟整条命令经 `scripts/crash/gpu-quiesce-run.sh` 起，清场记录原样：

```text
GPU_QUIESCE step=quiesce result=stopped-and-read-back holders=0
GPU_QUIESCE step=command result=exit-0
GPU_QUIESCE step=restore result=not-restored holders=1
GPU_QUIESCE step=exit code=0
```

  单点核那一份 `step=quiesce result=stopped-and-read-back holders=0`、`step=restore result=not-restored holders=0`；第二段那一份第一行是 `step=quiesce result=joined-and-read-back holders=1`（别的会话已登记同用卡，加入而不是新停）；三份都是 `step=command result=exit-0`、`step=exit code=0`。清场回读没过会退 78、命令不起，三趟都没有。
- 判决行（单点核两份、第一段、第二段，整行抄）：

```text
E7RESULT name=verdict segment=geometry_check constant_mismatches=0 overwrite_order=highest_full_segment_last points=2 geometry_formed_points=2
E7RESULT name=verdict segment=geometry_check constant_mismatches=0 overwrite_order=slot_order points=2 geometry_formed_points=1
E7RESULT name=verdict segment=wall_clock constant_or_anchor_mismatches=0 stop_clause_findings=0 positive_control_failures=0 positive_control_failure_items=none discrimination_failures=0 clause_wall_clock_not_rising_with_the_batch_triggered=0 clause_two_cpu_paths_disagree_points=0 gc_to_sequential_write_bandwidth_maximum_ratio=0.015977 gc_to_sequential_write_bandwidth_judgement=single_run_enough stored_intents=140
E7RESULT name=verdict segment=stage_split constant_or_anchor_mismatches=0 stop_clause_findings=0 positive_control_failures=0 positive_control_failure_items=none
```

  四行里取值 `false` 的字段没有，也没有 `not_run`，名字表示违例、不匹配、失败的计数（`constant_mismatches`、`constant_or_anchor_mismatches`、`stop_clause_findings`、`positive_control_failures`、`discrimination_failures`、两条失败条款的触发数）都是 0；大于 0 的整数只有 `stored_intents=140` 与 `points`/`geometry_formed_points`：旧造法对照那一份 `geometry_formed_points=1`（两点里只造成了占比 0.75 那一点）是对照的预期，它重现了第 1 次重跑的「没造成」，改法甲那一份是 `geometry_formed_points=2`。`gc_to_sequential_write_bandwidth_maximum_ratio=0.015977` 是 Q16 的值不是失败计数。
- 起跑脚本：三趟当时跑的是仓根 `e185-r2-tools/` 下的 `run-geometry-check.sh`、`run-wall-clock.sh`、`run-stage-split.sh` 与它们共用的 `common.sh`；之后照用户要求，合成了 `scripts/experiment-apparatus/e185-r2-segment-run.sh`（登记第五节「跑法」已改写，第十二节「起法」一行写明），三趟跑的是合并之前那几份，每段做的事相同（写输入清单 → 在拷过去的树里编一次装置 → 跑这一段 → 产物写进 `research/results/`）。
- `scripts/experiment-apparatus/replay.sh` 还没有 E185（GC一批搬迁的挂钟拆分） 的登记行（在 `scripts/experiment-apparatus/replay.sh` 里按该实验号 `grep -c` 输出 0；照 E184（常规层与大负载层的代价计数） 的先例只登记确定性的一片），三份测量产物都带挂钟字段。

````

**出处 `.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md:52-128`（整段抄，未转述）**

````markdown
### 几何：f = 0.25 两点造成了（改法甲）

改法甲只换造池第 2 步覆盖写的次序（「做了什么」第 2 条），占比 0.75 的次序不变。单点核（登记第五节「单点核」，只核 P = 1024）有两份：改法甲 `research/results/e185-gc-batch-wall-clock-split-geometry-check-r2-2026-10-23.out`，对照 `research/results/e185-gc-batch-wall-clock-split-geometry-check-slot-order-r2-2026-10-23.out`（`--overwrite-order slot-order`，第 1 次重跑那种造法）。登记三格的读数：

| 格 | 读哪 | 读到的 | 判 |
|---|---|---|---|
| 甲造成 | 改法甲那一份第 64 行 `name=geometry`、第 55 行 `name=segment_composition` | `formed=true`；那一段 `live_slots=16 slots_of_this_workload=16 other_slots=0` | 成立，停机条款第 8 条（第十条 H1）不中 |
| 占比 0.75 没被同一棵树里的改动带歪 | 两份第 123 行 `name=pool fraction=0.75 pool_data_units=1024` 的 `digests=` 整值 | 相同（紧接的代码块里 `diff` 退 0） | 成立，停机条款第 9 条不中 |
| 对照：旧造法在今天的树上重现「没造成」 | 对照那一份第 3 行 `name=geometry_attempt`、第 64 行 `name=geometry` | `geometry_formed_segments=0`、`choice=none … formed=false` | 与登记推的 0 相同 |

```text
E7RESULT name=geometry fraction=0.25 pool_data_units=1024 choice=forced_fallback fallback_region_start_slot=53824 formed=true reason=greedy_segment_holds_units_other_than_this_workload_no_registered_workload_change
E7RESULT name=segment_composition fraction=0.25 pool_data_units=1024 overwrite_order=highest_full_segment_last workload_changes=0 region_start_slot=53824 live_slots=16 slots_of_this_workload=16 other_slots=0 formed=true
E7RESULT name=geometry_attempt fraction=0.25 pool_data_units=1024 workload_changes=0 construction_seconds=20.697 overwritten_units=776 filler_units=0 floor_raises_pushed_by_the_session=0 release_rounds=4 every_overwritten_slot_released=true mapped_units=1056 geometry_formed_segments=1 greedy_region_start_slot=51008 greedy_live_slots=1 greedy_slots_of_this_workload=0 greedy_other_slots=1 greedy_formed=false
E7RESULT name=geometry_attempt fraction=0.25 pool_data_units=1024 workload_changes=0 construction_seconds=20.776 overwritten_units=776 filler_units=0 floor_raises_pushed_by_the_session=0 release_rounds=4 every_overwritten_slot_released=true mapped_units=1058 geometry_formed_segments=0 greedy_region_start_slot=50368 greedy_live_slots=1 greedy_slots_of_this_workload=0 greedy_other_slots=1 greedy_formed=false
E7RESULT name=geometry fraction=0.25 pool_data_units=1024 choice=none fallback_region_start_slot=none formed=false reason=greedy_segment_holds_units_other_than_this_workload_no_registered_workload_change
```

改法甲下 greedy 自己选中的那一段（第 3 行 `greedy_region_start_slot=51008 greedy_live_slots=1 greedy_slots_of_this_workload=0 greedy_other_slots=1 greedy_formed=false`）仍是装着别的单元的段，f = 0.25 的点跟占比 0.75 一样走登记好的后备（`choice=forced_fallback`，只供测试的强制选段点名那一个 f 段，第 64 行 `reason=greedy_segment_holds_units_other_than_this_workload_no_registered_workload_change`）。所以第一、二段量的是被强制选定的那一段，不是 greedy 自己选中的段。第一、二段八个 `name=geometry` 行（四个点 × 两段）全是 `choice=forced_fallback … formed=true`：

```text
$ grep -h 'name=geometry ' research/results/e185-gc-batch-wall-clock-split-wall-clock-r2-2026-10-23.out research/results/e185-gc-batch-wall-clock-split-stage-split-r2-2026-10-23.out | grep -c 'choice=forced_fallback .* formed=true'
8
```

```text
E7RESULT name=geometry fraction=0.25 pool_data_units=1024 choice=forced_fallback fallback_region_start_slot=53824 formed=true reason=greedy_segment_holds_units_other_than_this_workload_no_registered_workload_change
E7RESULT name=geometry fraction=0.25 pool_data_units=8192 choice=forced_fallback fallback_region_start_slot=78464 formed=true reason=greedy_segment_holds_units_other_than_this_workload_no_registered_workload_change
```

占比 0.75 的池与第 1 次重跑是不是逐字节相同，登记第五节两道，各报各的：

```text
$ diff <(awk 'NR==237||NR==515' research/results/e185-gc-batch-wall-clock-split-wall-clock-r1-2026-10-22.out | sed -E 's/.* pool_data_units=([0-9]+) .* digests=([^ ]+).*/\1 \2/') <(grep '^E7RESULT name=pool fraction=0.75 ' research/results/e185-gc-batch-wall-clock-split-wall-clock-r2-2026-10-23.out | sed -E 's/.* pool_data_units=([0-9]+) .* digests=([^ ]+).*/\1 \2/'); echo "exit=$?"
1,2c1,2
< 1024 423aa5bc004b3247287ae29cf0bd80de0508b33ec2724f654640e0475fdbc216,ec8a4ecb79383a2f66c6828c4ce64bc0234a708f10c09b7566a5d58b79200ca3
< 8192 e0910aa57c7de2b42dfd1026123435f3e84244105ef0067fb6b0aa44edc3cbb6,d177e4fc925f9c701acdd6e9a879bfff3017615db4c034e1d0c9e55ea6902e26
---
> 1024 a77b38d298bf0f30358b2011867845734bea60928bcb4b1e98b2ab13563d30c3,c43dcaf3fa8b68f1cf7b1d498569f1fa96a6220d07b428d8b5ceecb8b5ebc5d0
> 8192 7d26930931d792bf5859dfc1ceaa83a87ae4a2328ffce19af576986880642c3e,72de4e03f439421f3e56933997b33246b24351994fb457965e6d157778679ede
exit=1
$ diff <(grep '^E7RESULT name=pool fraction=0.75 pool_data_units=1024 ' research/results/e185-gc-batch-wall-clock-split-geometry-check-r2-2026-10-23.out | sed -E 's/.* pool_data_units=([0-9]+) .* digests=([^ ]+).*/\1 \2/') <(grep '^E7RESULT name=pool fraction=0.75 pool_data_units=1024 ' research/results/e185-gc-batch-wall-clock-split-geometry-check-slot-order-r2-2026-10-23.out | sed -E 's/.* pool_data_units=([0-9]+) .* digests=([^ ]+).*/\1 \2/'); echo "exit=$?"
exit=0
$ diff <(grep '^E7RESULT name=pool fraction=0.75 pool_data_units=1024 ' research/results/e185-gc-batch-wall-clock-split-geometry-check-r2-2026-10-23.out | sed -E 's/.* pool_data_units=([0-9]+) .* digests=([^ ]+).*/\1 \2/') <(grep '^E7RESULT name=pool fraction=0.75 pool_data_units=1024 ' research/results/e185-gc-batch-wall-clock-split-wall-clock-r2-2026-10-23.out | sed -E 's/.* pool_data_units=([0-9]+) .* digests=([^ ]+).*/\1 \2/'); echo "exit=$?"
exit=0
```

第 1 道（与第 1 次重跑第一段第 237、515 行比）不同，第 2 道（同一棵树里：单点核两份、再加第一段第 912 行）相同。照登记第五节判法记：占比 0.75 的池与第 1 次重跑不同，同一棵树里改动没有带歪它；差在两棵树之间（核心层五份源文件与 `Cargo.lock` 的 sha256 与第 1 次重跑输入清单不同，登记第三节第 134 行）。不停，占比 0.75 的点照量，这一趟的数不与第 1 次重跑的数混用，所以岔路表与各量的判定只引这一趟。

机理核对（登记第八节，第十条 H2；只报，不是锚点，对不上不作废）：覆盖写副本落进这次造池打出的旧洞的份数，装置逐次数出来的与登记推的 `overwritten_units − 24` 并列，造成段数两条路数并列，紧接的两条命令从三份产物里数：

```text
$ grep -h 'name=overwrite_landing' research/results/e185-gc-batch-wall-clock-split-wall-clock-r2-2026-10-23.out research/results/e185-gc-batch-wall-clock-split-stage-split-r2-2026-10-23.out research/results/e185-gc-batch-wall-clock-split-geometry-check-r2-2026-10-23.out | grep 'status=counted' | sed -E 's/.*fraction=([0-9.]+) pool_data_units=([0-9]+) .*copies_landing_in_holes_of_this_construction=([0-9]+) .*predicted_copies_landing_in_holes=([0-9]+).*/\1 \2 \3 \4/' | sort | uniq -c
      3 0.25 1024 752 752
      2 0.25 8192 6128 6128
      3 0.75 1024 256 256
      2 0.75 8192 2048 2048
$ grep -h 'name=segment_composition_summary' research/results/e185-gc-batch-wall-clock-split-wall-clock-r2-2026-10-23.out research/results/e185-gc-batch-wall-clock-split-stage-split-r2-2026-10-23.out | sed -E 's/.*fraction=([0-9.]+) pool_data_units=([0-9]+) .* formed_segments=([0-9]+) geo.*/\1 \2 \3/' | sort | uniq -c
      2 0.25 1024 1
      2 0.25 8192 1
      2 0.75 1024 3
      2 0.75 8192 3
```

三样都对上：每个点落进旧洞的副本数 = 登记的预测（752、6128、256、2048）；改法甲下 f = 0.25 的 `formed_segments` 是 1（登记预测 1，与 P 无关），f = 0.75 是 3（预测 3）；对照那一份 f = 0.25 的 `geometry_formed_segments=0`（预测 0，上面对照格）。`name=segment_composition_summary` 共 12 行（第一、二段各 4、单点核两份各 2），`counts_agree=true` 12 行、`counts_agree=false` 0 行（`grep -h 'name=segment_composition_summary' <四份产物> | grep -c 'counts_agree=true'` 输出 12，`grep -c 'counts_agree=false'` 输出 0）。记「调查员推的机理在这几个点上对上了」；这一句只说这四个点与单点核那两点，不外推到别的 f、P。

```text
E7RESULT name=segment_composition_summary fraction=0.25 pool_data_units=1024 overwrite_order=highest_full_segment_last workload_changes=0 segments=58 formed_segments=1 geometry_formed_segments_of_the_attempt=1 counts_agree=true
E7RESULT name=overwrite_landing fraction=0.25 pool_data_units=1024 overwrite_order=highest_full_segment_last workload_changes=0 status=counted overwritten_units=776 copies_landing_in_holes_of_this_construction=752 copies_landing_elsewhere=24 predicted_copies_landing_in_holes=752 root_ring_entries=24
E7RESULT name=segment_composition_summary fraction=0.25 pool_data_units=8192 overwrite_order=highest_full_segment_last workload_changes=0 segments=472 formed_segments=1 geometry_formed_segments_of_the_attempt=1 counts_agree=true
E7RESULT name=overwrite_landing fraction=0.25 pool_data_units=8192 overwrite_order=highest_full_segment_last workload_changes=0 status=counted overwritten_units=6152 copies_landing_in_holes_of_this_construction=6128 copies_landing_elsewhere=24 predicted_copies_landing_in_holes=6128 root_ring_entries=24
E7RESULT name=segment_composition_summary fraction=0.75 pool_data_units=1024 overwrite_order=highest_full_segment_last workload_changes=0 segments=53 formed_segments=3 geometry_formed_segments_of_the_attempt=3 counts_agree=true
E7RESULT name=overwrite_landing fraction=0.75 pool_data_units=1024 overwrite_order=highest_full_segment_last workload_changes=0 status=counted overwritten_units=280 copies_landing_in_holes_of_this_construction=256 copies_landing_elsewhere=24 predicted_copies_landing_in_holes=256 root_ring_entries=24
E7RESULT name=segment_composition_summary fraction=0.75 pool_data_units=8192 overwrite_order=highest_full_segment_last workload_changes=0 segments=418 formed_segments=3 geometry_formed_segments_of_the_attempt=3 counts_agree=true
E7RESULT name=overwrite_landing fraction=0.75 pool_data_units=8192 overwrite_order=highest_full_segment_last workload_changes=0 status=counted overwritten_units=2072 copies_landing_in_holes_of_this_construction=2048 copies_landing_elsewhere=24 predicted_copies_landing_in_holes=2048 root_ring_entries=24
```

````

**出处 `.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md:129-139`（整段抄，未转述）**

````markdown
### 问题单第 1 行够判的集合 S

登记第一节「第 1 行够判」的读法（设计员写的，主 agent 认下，调度记录 `records/2026-10-19-里程碑四GPU垃圾回收开工.md` 第 381 行）：S = 第一段里（f、P、K）三者的格，那一点 `name=geometry` 写 `formed=true`，且那一格 `name=q3` 写 `judgement=compatible`；两个 f 在 S 里各有至少两个 K 就够判；Q1、Q2 在 S 的格上报的判定就是第 1 行的读数。

```text
$ grep -c 'name=q3 .*judgement=compatible row_one_judgeable_here=true' research/results/e185-gc-batch-wall-clock-split-wall-clock-r2-2026-10-23.out
12
```

第一段 `name=geometry` 四行全 `formed=true`（上面），`name=q3` 12 格全是 `judgement=compatible row_one_judgeable_here=true`（第 1879、1883、1887、1891、1895、1899、1903、1907、1911、1915、1919、1923 行），所以 S = 全部 12 格：f = 0.25 与 f = 0.75 各 6 格（P = 1024、8192 × K = 4、8、16）。第二段 12 格同一批点。F5 没触发（判决行 `clause_two_cpu_paths_disagree_points=0`），F4 没有（每格 5 轮判定一致）。

````

**出处 `.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md:140-192`（整段抄，未转述）**

````markdown
### 各量的判定（第 2 次重跑）

照原登记第六节各报各的判定，5 轮判定不全相同写「不稳定」。产物：第一段 = `research/results/e185-gc-batch-wall-clock-split-wall-clock-r2-2026-10-23.out`（这一页里行号叫「第一段第 N 行」），第二段 = `research/results/e185-gc-batch-wall-clock-split-stage-split-r2-2026-10-23.out`（「第二段第 N 行」）。区间是对那一组行取最小与最大，两端都是产物里的原值。第一段 Q1、Q2 合计 24 格的判定数：

```text
$ W=research/results/e185-gc-batch-wall-clock-split-wall-clock-r2-2026-10-23.out
$ grep -E 'name=q[12] ' $W | sed -E 's/.* judgement=([a-z_]+)$/\1/' | sort | uniq -c
     12 both
     12 cpu
```

第一段 12 个（f、P、K）格，每格在 `name=q1`、`name=q2`、`name=q3`、`name=q14` 四行里各一行，四行连着排（第 1877–1880 行起，每格 4 行，到第 1924 行）。

| 量 | 读哪 | 这一趟的数 | 判定 |
|---|---|---|---|
| Q1 ρ_真盘 | 第一段 `name=q1` 12 行（第 1877、1881、1885、1889、1893、1897、1901、1905、1909、1913、1917、1921 行） | `rho_median`：P = 1024 六格 1.221978–1.493617，P = 8192 六格 8.441898–9.848870 | P = 1024 六格 `both`，P = 8192 六格 `cpu`；每格 5 轮判定全相同（`judgement_per_round`），没有 `disk`、没有 `unstable`；f = 0.25 的六格与 f = 0.75 的六格同判。随 P 翻面（第 1926 行六对全翻），不随 f（第 1925 行 `flips=0`）、不随 K（第 1927 行 `flips=0`） |
| Q2 挂钟比 | 第一段 `name=q2` 12 行（第 1878、1882、…、1922 行，每隔 4 行） | `intent_wall_ratio_median`：P = 1024 六格 1.723812–1.906832，P = 8192 六格 1.105487–1.118963 | 与 Q1 同判：P = 1024 六格 `both`，P = 8192 六格 `cpu`，每格 5 轮相同；随 P 翻面（第 1929 行 `flips=6`），不随 f（第 1928 行）、不随 K（第 1930 行）`flips=0` |
| Q3 两路对账 | 第一段 `name=q3` 12 行（第 1879、1883、…、1923 行） | 每轮 `reconciliation_per_round` 最大 0.103088（f = 0.75、P = 1024、K = 8 第 5 轮，第 1907 行）；f = 0.25 的格最大 0.055696（P = 1024、K = 16 第 2 轮，第 1887 行） | 12 格全 `compatible`、`row_one_judgeable_here=true`；F5 没触发（判决行 `clause_two_cpu_paths_disagree_points=0`）。离 20% 的门槛最近的一轮是 0.103088 |
| Q4 批大小翻倍 | 第一段 `name=q4` 12 对（第 1933–1944 行） | 每对 5 轮中位 1.015102–1.074115 | 内存臂 6 对、真盘臂 6 对全 `rises`，12 对的 `copy_bytes_double=true`、`clause_wall_clock_not_rising_with_the_batch_triggered=false`；判决行 F1 触发数 0。判据只要 > 1（且每轮 > 1），这一趟涨得不多：批大小翻倍，第一批挂钟涨 1.5%–7.4% |
| Q14 GC 单跑写带宽 | 第一段 `name=q14` 12 行 | `write_bytes_per_second_median` 7229989.048161–50509463.337147 | 不单判 |
| Q15 设备顺序写带宽 | 第一段第 1945 行 | `E7RESULT name=q15 direct_with_fdatasync_bytes_per_second_median=3161349388.1 page_cache_without_sync_bytes_per_second_median=13066869439.3 positive_control_row_four_holds=true` | 不单判 |
| Q16 带宽比 | 第一段第 1958 行 | `maximum_ratio=0.015977`（f = 0.75、P = 1024、K = 16，第 1954 行 `q16_point`） | 「单跑够」；三个旋钮都不翻（第 1959–1961 行 `flips=0`） |
| Q17 前台 fsync 延迟比 | 第四段 | 没跑 | 够判后未跑（第 4 行，Q16 ≤ 10%，原登记第十一节够判停机） |
| Q5 甲段份额 | 第二段 `name=stage` 批行（`kind` 是 `batch` 或 `final_batch`）的 `crc_share`；`name=stage_judgement` 30 行 | f = 0.75 最大 0.020433（第 947 行），f = 0.25 最大 0.006863（第 116 行） | 30 行 `crc_share_judgement=not_worth`，每行 5 轮都是 |
| Q6 乙段增量份额 | 同上的 `scan_increment_share` | f = 0.75 最大 0.006521（第 1065 行），f = 0.25 最大 0.002601（第 216 行） | 30 行 `scan_increment_share_judgement=not_worth`，每行 5 轮都是 |
| Q7 丙段增量份额 | 第二段 `name=choose_judgement` 12 行（第 248–250、869–871、1148–1150、1793–1795 行）的 `choose_share_per_batch_per_round` | f = 0.25 最大 0.028846（第 250 行），f = 0.75 最大 0.014279（第 1150 行）；签发那一次的原值 `choose_share_of_the_issue` 最大 f = 0.25 是 0.03894、f = 0.75 是 0.038388，也都低于 10% | 12 行 `choose_share_judgement=not_worth`，每行 5 轮都是。`name=stage_judgement` 里的丙段那一格 30 行都是 `not_run`，是登记第六节写明的（丙段改读 `name=choose_judgement`），不是漏判 |
| Q8 单线程吞吐 | 第二段批行 `crc_bytes_per_second`、`scan_increment_bytes_per_second`；签发行 `choose_record_bytes_per_second` | 甲 2875331797.740485–3189332554.68769 B/s；乙增量 1046057094.516405–12147544022.242819 B/s；丙 112157777.007656–892197239.242721 记录字节/秒 | 不单判，喂 Q9 |
| 饱和吞吐（Q9 的 CPU 一侧） | 第二段 `name=saturation` 108 行，`distinct_threads_holds=true` 108 行 | 16 线程：甲 48832371460.2–49032300970.9，乙 14532706506.7–37784969513.8，丙 1641866090.6–13741047928.6 B/s | 不单判 |

第二段几条数的命令与输出（`S` 是第二段产物路径）：

```text
$ S=research/results/e185-gc-batch-wall-clock-split-stage-split-r2-2026-10-23.out
$ for f in 0.25 0.75; do for k in crc_share scan_increment_share; do echo -n "batch f=$f $k max: "; grep 'name=stage ' $S | grep -E "kind=(batch|final_batch)" | grep "fraction=$f " | grep -o " $k=[0-9.]*" | sed 's/.*=//' | sort -g | tail -1; done; done
batch f=0.25 crc_share max: 0.006863
batch f=0.25 scan_increment_share max: 0.002601
batch f=0.75 crc_share max: 0.020433
batch f=0.75 scan_increment_share max: 0.006521
$ for f in 0.25 0.75; do echo -n "choose_judgement f=$f max per_round: "; grep 'name=choose_judgement ' $S | grep "fraction=$f " | grep -o 'choose_share_per_batch_per_round=[^ ]*' | sed 's/.*=//' | tr ',' '\n' | sort -g | tail -1; done
choose_judgement f=0.25 max per_round: 0.028846
choose_judgement f=0.75 max per_round: 0.014279
$ grep 'name=stage_judgement ' $S | sed -E 's/.* (crc_share_judgement=[a-z_]+) .* (scan_increment_share_judgement=[a-z_]+) .* (choose_share_judgement=[a-z_]+)$/\1 \2 \3/' | sort | uniq -c
     30 crc_share_judgement=not_worth scan_increment_share_judgement=not_worth choose_share_judgement=not_run
$ grep 'name=choose_judgement ' $S | sed -E 's/.* (choose_share_judgement=[a-z_]+)$/\1/' | sort | uniq -c
     12 choose_share_judgement=not_worth
```

最大的那一行的第二段第 250 行（丙段）整行抄：

```text
E7RESULT name=choose_judgement fraction=0.25 pool_data_units=1024 batch_units=16 rounds=5 choose_share_per_batch_per_round=0.028676,0.028423,0.028614,0.028705,0.028846 choose_share_judgement_per_round=not_worth,not_worth,not_worth,not_worth,not_worth choose_share_judgement=not_worth
```

````

**出处 `.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md:193-205`（整段抄，未转述）**

```markdown
### 沿用第 1 次重跑的量（问题单第 3、4 行；这一趟第三、四段不跑）

数取自第 1 次重跑第三段 `research/results/e185-gc-batch-wall-clock-split-gpu-submission-r1-2026-10-22.out`（这一页叫「r1 第三段第 N 行」）与它读的第 1 次重跑第一段 `research/results/e185-gc-batch-wall-clock-split-wall-clock-r1-2026-10-22.out`。两个范围都含 f = 0.25 的两点，那两点在第 1 次重跑里几何没造成（量的是 greedy 选中的段），这一趟没有重算。

| 量 | 读哪 | 第 1 次重跑的数 | 判定 |
|---|---|---|---|
| Q9 16 线程对 GPU | r1 第三段 `name=q9` 72 行（两张卡 × 三段 × 12 格） | `ratio` 最大 0.007746（卡 0、乙增量、f = 0.75、P = 8192、K = 8） | 72 行 `judgement=gpu_loses`，单线程版 `judgement_one_thread=gpu_loses`，`clause_sixteen_thread_form_misfits_the_gc_triggered=false`（F2 没触发）。这一趟在 f = 0.25 造成了的点上没有 Q9。Q9 算不算问题单第 2 行的翻面量，交主 agent，并进七·2 设计轮定（登记第一节、调度记录第 393 行） |
| Q10 B_批 | r1 第三段 `name=q13` 的 `batch_bytes`（读第 1 次重跑第一段 `name=call`） | f = 0.75：131072–786432 字节；f = 0.25：16384 字节（几何没造成的点） | 不单判 |
| Q11 a、Q12 b | r1 第三段第 54、136 行 | 卡 0 `E7RESULT name=q11_q12 index=0 card_valid=true intercept_seconds=0.004492 slope_seconds_per_byte=4.117289e-10 one_range_median_seconds=0.005244 largest_two_points_slope_seconds_per_byte=4.172332e-10`；卡 1 `E7RESULT name=q11_q12 index=1 card_valid=true intercept_seconds=0.010300 slope_seconds_per_byte=1.266004e-9 one_range_median_seconds=0.005393 largest_two_points_slope_seconds_per_byte=1.191920e-9` | 不单判 |
| Q13 φ 与 n* | r1 第三段 `name=q13` 56 行 | 卡 0 `fixed_overhead_share` 0.932762–0.998500、`batches_to_merge` 125–5993；卡 1 0.911860–0.997990、94–4470 | 56 行 `judgement=merge_batches`（φ ≥ 50%，「要攒批」），两张卡同判（F8 没触发）。判别力第 201 行 `E7RESULT name=discrimination quantity=Q13 lower_value=0.911860 higher_value=0.998500 moved_threshold=0.955180 lower_judgement=one_batch_per_submission higher_judgement=merging_optional holds=true` |

推的（没量）：f = 0.25 造成之后，它的 B_批 ≤ 签发那一次的 262144 字节（原登记第七节乙类第 356 行），比 f = 0.75 的小；φ = a ÷ (a + b·B_批) 随 B_批变小只会变大，翻面要 φ 落到 50% 以下，第 1 次重跑 φ 最小的那一格是 0.911860。

```

**出处 `.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md:206-221`（整段抄，未转述）**

````markdown
### 阳性对照、自检与失败条款

- 第一段第 1 行阳性对照四格（第 1963–1966 行）：盘臂内存盘、真盘两格 `holds=true`，CPU 臂内存盘、真盘两格 `cpu_clause_holds=true write_wait_clause_holds=true`；CPU 臂内存盘那一格原判据那一句 `wait_clause_under_the_original_criterion_holds=false`，照 12.9 只报不判。整行抄：

```text
E7RESULT name=positive_control row=1 arm_of_the_question=disk device=memory injection=sleep_2_milliseconds_per_write baseline_rounds=5 control_rounds=5 write_calls_median=176.0 injected_nanoseconds=352000000.0 cpu_increment_nanoseconds=1795862.0 wait_increment_nanoseconds=362017597.0 write_wait_increment_nanoseconds=362030328.0 cpu_change=0.036631 wait_change=-3020.110262 paired_rounds=5 cpu_increment_paired_median_nanoseconds=1701172.000000 write_wait_increment_paired_median_nanoseconds=362030518.000000 holds=true
E7RESULT name=positive_control row=1 arm_of_the_question=cpu device=memory injection=spin_2_milliseconds_per_write baseline_rounds=5 control_rounds=5 write_calls_median=176.0 injected_nanoseconds=352000000.0 cpu_increment_nanoseconds=352496564.0 wait_increment_nanoseconds=68029.0 write_wait_increment_nanoseconds=67282.0 cpu_change=7.189988 wait_change=-0.567528 cpu_increment_over_injected=1.001411 cpu_clause_holds=true write_wait_increment_over_injected=0.000191 write_wait_clause_holds=true wait_clause_under_the_original_criterion_holds=false original_criterion_is_reported_only=true paired_rounds=5 cpu_increment_paired_median_nanoseconds=352445233.000000 write_wait_increment_paired_median_nanoseconds=67472.000000 holds=true
E7RESULT name=positive_control row=1 arm_of_the_question=disk device=real_disk injection=sleep_2_milliseconds_per_write baseline_rounds=5 control_rounds=5 write_calls_median=176.0 injected_nanoseconds=352000000.0 cpu_increment_nanoseconds=1782904.0 wait_increment_nanoseconds=361903509.0 write_wait_increment_nanoseconds=362318961.0 cpu_change=0.033885 wait_change=7.108353 paired_rounds=5 cpu_increment_paired_median_nanoseconds=1272218.000000 write_wait_increment_paired_median_nanoseconds=362373072.000000 holds=true
E7RESULT name=positive_control row=1 arm_of_the_question=cpu device=real_disk injection=spin_2_milliseconds_per_write baseline_rounds=5 control_rounds=5 write_calls_median=176.0 injected_nanoseconds=352000000.0 cpu_increment_nanoseconds=353173767.0 wait_increment_nanoseconds=4108344.0 write_wait_increment_nanoseconds=1711955.0 cpu_change=6.712162 wait_change=0.080694 cpu_increment_over_injected=1.003335 cpu_clause_holds=true write_wait_increment_over_injected=0.004864 write_wait_clause_holds=true wait_clause_under_the_original_criterion_holds=true original_criterion_is_reported_only=true paired_rounds=5 cpu_increment_paired_median_nanoseconds=352961316.000000 write_wait_increment_paired_median_nanoseconds=1684671.000000 holds=true
```

- 第二段第 2 行阳性对照四格（第 123–126 行）`holds=true`；两段 `name=clock_resolution_check` 都 `holds=true`（两段第 3 行）；第一段 `name=discrimination` 三行（Q1、Q2、Q16，第 1931、1932、1962 行）`holds=true`；两段 `name=anchor` 各 34 行全 `holds=true`（`grep 'name=anchor' <产物> | grep -c 'holds=true'` 两份都输出 34，`grep -vc` 输出 0）。
- 作废：第一段 `name=intent` 140 行全是 `valid=true invalid_reason=none`，第二段 60 行同；两段都没有 `name=round_invalid` 行（`grep -c 'name=round_invalid'` 输出 0），`stored_intents=140`。第 1 次重跑第一段作废过 1 轮，这一趟一轮都没作废、没重试。
- 失败条款：F1 没触发（Q4 12 对全 `rises`）；F2 这一趟不适用（Q9 沿用）；F3 打中：Q1 在 P = 1024 的六格落在 (0.5, 2)（1.221978–1.493617）、Q2 在 P = 1024 的六格落在 (1.5, 3)（1.723812–1.906832），照实记「两边都占」，是正当结果；F4 没有；F5 没触发；F7 打中：第二段 Q5、Q6、Q7 都 < 10%（最大分别 0.020433、0.006521、0.028846），照原登记记「三段都不到一批 CPU 的 10%，CPU 的大头在其余（读这一版、发布）」，交主 agent 判第 2 行的候选是否要补一个「发布那一半」的臂；F6、F8 沿用第 1 次重跑（F6：Q13 没有落在 (10%, 50%) 的格；F8：两张卡同判），这一趟没重算。
- 这一趟新加的失败条款（登记第十条）：H1（甲造不出）没触发；H2（机理对不上）没触发，见「几何」一节；H3、H4 以 F5 打中为前提，没打中，不适用；H5（轮数不足）没有，`name=round_invalid` 0 行；H6（第 1 行仍不够判）没有，S 里 f = 0.25 与 f = 0.75 各 6 格；H7（第 4 行单跑那一半翻面）没触发，`name=q16` 是 `single_run_enough`；H8（校准基线漂移）没触发，两段 `drifted=false`。

````

**出处 `.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md:222-241`（整段抄，未转述）**

````markdown
### 校准与机器状态读数（登记第六节新加的量，读数 1–N15）

校准比（读数 1）是测量线程每条意图前后对一块 16 MiB 缓冲连做 3 遍 CRC-32C 取中位，除以段开头 21 遍的中位；> 1.10 作废（修订 R2-1）；作废后先等机器回到常态再重试（R2-2）。其余读数只报不判。

| 读数 | 第一段 | 第二段 |
|---|---|---|
| 段开头 / 段末基线中位（读数 2） | 5243296.0 / 5227887.0 纳秒，末 ÷ 头 = 0.997061，`drifted=false`（第 5、2099 行） | 5224477.0 / 5222377.0 纳秒，0.999598，`drifted=false`（第 5、1799 行） |
| 校准自己的阳性对照（R2-3，加倍的活） | `ratio=2.003004 crc_holds=true holds=true`（第 6 行） | `ratio=2.003983 crc_holds=true holds=true`（第 6 行） |
| 前后两个校准比里大的那个的最大值（读数 1） | 1.015958；> 1.10 的意图 0 条，> 1.05 的 0 条，> 1.20 的 0 条；等常态 0 次（第 2100 行） | 1.077323；> 1.10 的 0 条，> 1.05 的 3 条，> 1.20 的 0 条；等常态 0 次（第 1800 行） |
| 追指针校准比（N15）的范围 | 1.226963–1.534854，每条意图都高于 1 | 1.279786–1.514878，同 |
| 整机其余部分占的忙比例（N10 `others_share`）最大 | 0.002502（第 4 行，段开头） | 0.001875（第 932 行） |
| 线程读数（读数 3–N9）最大 | 排队等 CPU 7508078 纳秒，非自愿切换 151，`major_faults` 全 0，CPU 号 4 个 | 排队等 CPU 914484 纳秒，非自愿切换 29，CPU 号 2 个 |

```text
E7RESULT name=calibration_trajectory segment=wall_clock intents=140 intents_with_a_readable_ratio=140 larger_ratio_peak=1.015958 intents_above_the_voiding_edge=0 voiding_edge=1.10 intents_above_the_lower_reported_edge=0 lower_reported_edge=1.05 intents_above_the_upper_reported_edge=0 upper_reported_edge=1.20 larger_ratio_final=0.998801 wait_longest_seconds=0 waits=0 wait_total_seconds=0
E7RESULT name=calibration_trajectory segment=stage_split intents=60 intents_with_a_readable_ratio=60 larger_ratio_peak=1.077323 intents_above_the_voiding_edge=0 voiding_edge=1.10 intents_above_the_lower_reported_edge=3 lower_reported_edge=1.05 intents_above_the_upper_reported_edge=0 upper_reported_edge=1.20 larger_ratio_final=1.002871 wait_longest_seconds=0 waits=0 wait_total_seconds=0
```

追指针校准比（N15）全部高于 1（1.23–1.53）是装置报出来的，登记只要求报、不判，这一页没查原因；它没有用来作废任何意图。N14（照原作废规则的并列判定）两段各 12 行，`differs_from_the_registered=true` 0 行（`grep -c 'differs_from_the_registered=true'` 对两份产物都输出 0）：没有一条意图因校准比被作废，R2-1 在这一趟没有动任何一格的进入与否。

````

**出处 `.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md:242-275`（整段抄，未转述）**

````markdown
### 问题单逐行（岔路表）

问题单 `research/prompts/m4-gpugc-measure-forks.md`（四行的「状态」一格已由主 agent 改成「够判」，调度记录 `records/2026-10-19-里程碑四GPU垃圾回收开工.md` 第 394 行）；够判条件照问题单原文，判定照这一趟的产物；S 的读法见「各量的判定」一节。

| 行 | 已够判 / 还差什么 | 这一趟读得出的 | 剩下的量能不能让它翻面 |
|---|---|---|---|
| 1 盘 ／ CPU | **已够判**。问题单够判条件：至少两种批大小、两种段内活槽占比，在内存设备与真盘上各量到一批的挂钟与按阶段拆开的 CPU 时间。f = 0.25 与 f = 0.75 的几何都造成了（第一段 `name=geometry` 四行 `formed=true`），S = 12 格（`grep -c 'name=q3 .*judgement=compatible row_one_judgeable_here=true'` 输出 12），两个 f 各 6 格、K = 4、8、16 都在，Q1、Q2 逐格 5 轮判定一致 | P = 1024 六格 Q1、Q2 都判 `both`（`rho_median` 1.221978–1.493617，`intent_wall_ratio_median` 1.723812–1.906832）；P = 8192 六格都判 `cpu`（8.441898–9.848870；1.105487–1.118963）。判定随 P 翻面（第一段第 1926、1929 行 `flips=6`），不随 f、不随 K 翻（第 1925、1927、1928、1930 行 `flips=0`）；按 f、P 报，不按 K 报。F3 打中（P = 1024 两边都占）。整行见紧接的第一块 | 登记里没有剩下没跑的量。能动它的只有 P 之外的点：装置只收 P = 1024、8192 两个池大小，两点之间与更大的池没量。推的（没量）：翻面轴是 P，池更大时 CPU 一侧只会更重 |
| 2 GPU 算哪几段 | **已够判**（第 1 行已够判）。登记第一节「第 2 行够判」：S 里每一格第二段 `name=stage_judgement` 的甲、乙判定与 `name=choose_judgement` 的丙判定都不是 `not_run`，且 `crc_bytes`、`records_read_per_choice` 大于 0：甲、乙 30 行批判定全 `not_worth`，丙 12 行全 `not_worth`，批行 `crc_bytes` 最小 131072，`records_read_per_choice` 最小 1068（命令见「各量的判定」一节） | 甲、乙、丙三段都 `not_worth`：甲份额最大 0.020433，乙增量最大 0.006521，丙每批最大 0.028846（签发那一次的原值最大 0.03894），都不到一批 CPU 的 10%。原登记 F7 打中：CPU 的大头在其余（读这一版、发布），交主 agent 判第 2 行的候选要不要补「发布那一半」的臂。Q9 沿用第 1 次重跑 72 行 `gpu_loses`，算不算翻面量交主 agent，并进七·2 设计轮定（Q9 是登记第一节写明的一格）。整行见紧接的第二块 | 登记里没有剩下没跑的量。Q9 在 f = 0.25 的点上没有（第三段不跑）；它不论算不算翻面量，都与 Q5–Q7 同向（第 1 次重跑 72 行全 `gpu_loses`，这一趟三段份额没有一格越过 10%） |
| 3 攒批形态 | **已够判**（沿用第 1 次重跑，这一趟第三段不跑）。依据：第 1 次重跑第三段 `research/results/e185-gc-batch-wall-clock-split-gpu-submission-r1-2026-10-22.out`（sha256 `4041a1ca…`）与它读的第 1 次重跑第一段 `research/results/e185-gc-batch-wall-clock-split-wall-clock-r1-2026-10-22.out`（`978ae7b5…`）。第三段装置 `eadb929d…` 比重跑登记写死的 `b3a50d50…` 多 3 行（`quarantine_mark`），主 agent 已认（本次登记第一节派发提示第 5 条，登记第 19 行） | 同第 1 次重跑：Q10 f = 0.75 每批 131072–786432 字节；两张卡 a、b 量到；Q13 56 行全 `merge_batches`，φ 最小 0.911860，两张卡同判：一批一交（甲）出局，选择在乙、丙之间，乙丙之间七·1 只给 n* 与每批字节，R 的语义与批间重扫由七·2 设计轮判（原登记第五节第 3 行臂表） | 没有剩下的量。翻面要 φ 落到 50% 以下，φ 最小的那一格也在 0.91 以上；f = 0.25 造成之后 B_批更小，φ 只会更大（推的，没量） |
| 4 兜底间隔与写带宽上限 | **单跑那一半已够判**（沿用第 1 次重跑 `name=q16`，这一趟第一段并列报、不重判）；第四段与 Q17 够判后未跑 | 第 1 次重跑第一段第 786 行 `E7RESULT name=q16 maximum_ratio=0.011019 judgement=single_run_enough …`；这一趟第一段第 1958 行 `maximum_ratio=0.015977 judgement=single_run_enough`，H7 没触发 | Q17 只在 Q16 > 10% 时跑，这一趟不跑 |

第一块（第 1 行的读数，第一段整行）：

```text
E7RESULT name=q1 fraction=0.25 pool_data_units=1024 batch_units=8 geometry_formed=true rounds=5 rho_per_round=1.456034,1.438745,1.464884,1.404448,1.458415 rho_median=1.456034 judgement_per_round=both,both,both,both,both judgement=both
E7RESULT name=q2 fraction=0.25 pool_data_units=1024 batch_units=8 geometry_formed=true rounds=5 intent_wall_ratio_per_round=1.772596,1.749635,1.720118,1.741935,1.760248 intent_wall_ratio_median=1.749635 per_batch_median_ratio=1.749635 judgement_per_round=both,both,both,both,both judgement=both
E7RESULT name=q3 fraction=0.25 pool_data_units=1024 batch_units=8 rounds=5 reconciliation_per_round=0.050004,0.031328,0.021439,0.016639,0.043347 judgement_per_round=compatible,compatible,compatible,compatible,compatible judgement=compatible row_one_judgeable_here=true
E7RESULT name=q1 fraction=0.75 pool_data_units=8192 batch_units=8 geometry_formed=true rounds=5 rho_per_round=9.819379,9.125596,9.851385,8.752512,9.908914 rho_median=9.819379 judgement_per_round=cpu,cpu,cpu,cpu,cpu judgement=cpu
E7RESULT name=q2 fraction=0.75 pool_data_units=8192 batch_units=8 geometry_formed=true rounds=5 intent_wall_ratio_per_round=1.108970,1.117088,1.108237,1.122220,1.114470 intent_wall_ratio_median=1.114470 per_batch_median_ratio=1.120025,1.116781,1.108746 judgement_per_round=cpu,cpu,cpu,cpu,cpu judgement=cpu
E7RESULT name=q3 fraction=0.75 pool_data_units=8192 batch_units=8 rounds=5 reconciliation_per_round=0.006159,0.006354,0.005763,0.006425,0.012056 judgement_per_round=compatible,compatible,compatible,compatible,compatible judgement=compatible row_one_judgeable_here=true
E7RESULT name=sensitivity quantity=Q1 knob=fraction pairs=6 flips=0 statement=does_not_flip_at_these_points flipped_pairs=none
E7RESULT name=sensitivity quantity=Q1 knob=pool_data_units pairs=6 flips=6 statement=flips_with_this_knob flipped_pairs=0.25:1024:4:both->0.25:8192:4:cpu,0.25:1024:8:both->0.25:8192:8:cpu,0.25:1024:16:both->0.25:8192:16:cpu,0.75:1024:4:both->0.75:8192:4:cpu,0.75:1024:8:both->0.75:8192:8:cpu,0.75:1024:16:both->0.75:8192:16:cpu
E7RESULT name=sensitivity quantity=Q2 knob=pool_data_units pairs=6 flips=6 statement=flips_with_this_knob flipped_pairs=0.25:1024:4:both->0.25:8192:4:cpu,0.25:1024:8:both->0.25:8192:8:cpu,0.25:1024:16:both->0.25:8192:16:cpu,0.75:1024:4:both->0.75:8192:4:cpu,0.75:1024:8:both->0.75:8192:8:cpu,0.75:1024:16:both->0.75:8192:16:cpu
```

第二块（第 2 行的读数，第二段整行：丙段判定、批判定的一行，份额最大的两行批行）：

```text
E7RESULT name=choose_judgement fraction=0.25 pool_data_units=1024 batch_units=4 rounds=5 choose_share_per_batch_per_round=0.014847,0.014637,0.014529,0.014800,0.014522 choose_share_judgement_per_round=not_worth,not_worth,not_worth,not_worth,not_worth choose_share_judgement=not_worth
E7RESULT name=stage_judgement fraction=0.25 pool_data_units=1024 batch_units=4 batch_index=0 rounds=5 crc_share_judgement_per_round=not_worth,not_worth,not_worth,not_worth,not_worth crc_share_judgement=not_worth scan_increment_share_judgement_per_round=not_worth,not_worth,not_worth,not_worth,not_worth scan_increment_share_judgement=not_worth choose_share_judgement_per_round=not_run,not_run,not_run,not_run,not_run choose_share_judgement=not_run
E7RESULT name=stage fraction=0.75 pool_data_units=1024 batch_units=4 round=1 call_index=1 kind=batch call_cpu_nanoseconds=12578975 crc_cpu_nanoseconds=257032.000000 crc_wall_nanoseconds=257271.000000 crc_repetitions=1 scan_cpu_nanoseconds=332311.000000 scan_increment_cpu_nanoseconds=75279.000000 choose_cpu_nanoseconds=none choose_calls=0 crc_bytes=786432 records_read_per_choice=1069 full_segments=3312 crc_share=0.020433 scan_increment_share=0.005985 choose_share_per_batch=none choose_share_of_the_issue=none crc_bytes_per_second=3059665722.555947 scan_increment_bytes_per_second=10446897541.146935 choose_record_bytes_per_second=none choose_records_per_second=none choose_full_segments_per_second=none snapshot_branch_bytes=0 snapshot_branch_note=the_pools_have_no_snapshot release_check_crc_cpu_nanoseconds=84920.000000 release_check_crc_wall_nanoseconds=85020.000000 release_check_crc_repetitions=1 release_check_crc_bytes=262144 crc_with_the_release_check_cpu_nanoseconds=341952.000000 crc_with_the_release_check_bytes=1048576
E7RESULT name=stage fraction=0.75 pool_data_units=1024 batch_units=16 round=3 call_index=1 kind=batch call_cpu_nanoseconds=13777990 crc_cpu_nanoseconds=247692.000000 crc_wall_nanoseconds=247821.000000 crc_repetitions=1 scan_cpu_nanoseconds=337532.000000 scan_increment_cpu_nanoseconds=89840.000000 choose_cpu_nanoseconds=none choose_calls=0 crc_bytes=786432 records_read_per_choice=1069 full_segments=3312 crc_share=0.017977 scan_increment_share=0.006521 choose_share_per_batch=none choose_share_of_the_issue=none crc_bytes_per_second=3175039968.993751 scan_increment_bytes_per_second=8753695458.593056 choose_record_bytes_per_second=none choose_records_per_second=none choose_full_segments_per_second=none snapshot_branch_bytes=0 snapshot_branch_note=the_pools_have_no_snapshot release_check_crc_cpu_nanoseconds=330301.000000 release_check_crc_wall_nanoseconds=330562.000000 release_check_crc_repetitions=1 release_check_crc_bytes=1048576 crc_with_the_release_check_cpu_nanoseconds=577993.000000 crc_with_the_release_check_bytes=1835008
```

````

**出处 `.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md:276-285`（整段抄，未转述）**

```markdown
### 做了什么

1. 计时包装设备：包住内存盘（`SparseBlockDevice`）或真盘（ext4 上的 `O_DIRECT` 镜像文件，`DirectInputOutputBlockDevice`），`BlockDevice` 的每一种调用前后各读一次 `Instant` 与本线程 `clock_gettime(CLOCK_THREAD_CPUTIME_ID)`（CPU 读数包着挂钟读数），按调用种类累计挂钟、CPU、等盘（挂钟减 CPU）、次数与字节；读按「这个计时窗口里第一次写之前 / 之后」分开，第一次写之前的读再分三栏：落在 R 那块盘 R 里的、扣掉搬迁源槽上的再读，归扫描读；这一批搬迁源的槽上（两块盘同号槽）扣掉 R 那块盘每个槽第一次读之后剩下的，归释放前读盘核（`release_check_read_bytes`，登记第十二节 12.5）；其余归别处；写按「落在这一批目的地 / 别的」分开。阳性对照的注入（每次 `write_at` 里睡 2 ms，或按线程 CPU 时钟自旋 2 ms）也在这里。
2. 造池：harness 默认参数 mkfs 两块 4 GiB 内存盘、第一个文件、可写挂载；建 P + 64 个空 inode，前 P 个各顺序写一个数据单元；整段都是这次写的段按槽序每 4 个覆盖写 3 个（f = 0.25）或 1 个（f = 0.75），混住的段这次写的单元全覆盖写掉；用多建的空 inode 把最高那一段补满；再「抬 F 到准入上限、建一个 inode」转到被覆盖写的旧落点都已释放。**覆盖写的次序（改法甲，装置 `overwrite_order_of`，第 2131 行）**：f = 0.75 仍全部按旧槽号升序；f = 0.25 改成三组依次：① 不在「整段 32 个单元都是这次写的」段里的单元（混住段的全部，与配方累计点名的），组内按旧槽号升序；② 整段里要覆盖写的单元，除槽号最大的那一整段之外，按旧槽号升序；③ 槽号最大的那一整段里要覆盖写的 24 个，按旧槽号升序。覆盖写哪些单元、写几次、写的内容不变，只换次序。机理（登记第七节「推的预测」）：每次覆盖写是一次发布、写一条根，根环 24 条根，旧落点在覆盖写那一次发布就记成已释放，覆盖写到第 25 次起每份副本都落进 24 次之前打出的洞，所以最后 24 次覆盖写打出的洞（槽号最大那一整段的 24 个）造完时还空着，那一段剩 16 个活槽、全是这次写的。
3. 几何核：在一份拷贝上可写挂载、照会话签发那一步的选段那一圈重放（`choose_region` 与「先扫」交替），看 greedy 将选哪一段；那一段全是这次写的数据单元、活槽 = 64f 就用 greedy，不是而它全是这次写的数据单元就把它们并进配方重造（至多三次），装着别的单元就走登记好的后备（只供测试的强制选段点名槽号最小的 f 段）。造池之后逐段打成分（`name=segment_composition`，N11）、造成段数两条路数并列（`name=segment_composition_summary` 的 `counts_agree`）、覆盖写副本的落点（`name=overwrite_landing`，N12）；这几行只打每个点头一次造池，第 2–5 轮的重造不打。
4. 量一条意图：可写挂载（`mount_writable_with_compaction_switches`，产品开关只把批大小换成只供测试的 `ForcedForTests(K)`），每次调用之前把会话「已经过去的发布数」设成 1000（发布预算落到 1），一次一次调 `run_compaction` 到最后一批删意图；每次调用前后读 `Instant` 与线程 CPU 时钟（挂钟读数包着 CPU 读数，与设备调用那一层反过来），返回之后在计时窗口外按发成的那一版判是签发、批还是最后一批。每条意图前后还在计时窗口外做一遍校准（读数 1、N15）和线程与机器读数（读数 3–N9，`name=intent_machine_readings`）：校准比 > 1.10 的那一条作废，作废之后先每 10 秒做一次校准、连着两次 ≤ 1.10 才重量，单次至多等 300 秒、一段里累计至多等 600 秒（修订 R2-1、R2-2）；每段开头做校准基线（读数 2，21 遍）与加倍活的自检（R2-3），段末再做一遍基线（R2-4）。段开头、每个取样点开头、段末各一行 `name=machine_state`（N10）。
5. 第二段的重放：调用之前取这一版的分配记录、映射条目（照会话的私有取法另写一份）、钉住与隔离的槽，取扫描读到的单元字节；三个阶段（甲 每个被读单元整单元 CRC-32C（另一个子项是释放前读盘核那几份副本的整单元 CRC-32C，副本字节在调用之前从两块盘同号槽各读一份）、乙 整个 `scan_region`、丙 签发那一步每次 `choose_region`）每次重放先不计时跑一遍热身、再计时跑一遍，5 次重放取中位；饱和吞吐 1、16、N 线程各 2 秒。丙段的判定（改动六）：每轮取那一轮有效意图签发那一行已经算出的 `choose_share_per_batch`，按原门槛（10%）判，五轮收成一行 `name=choose_judgement`（`name=stage_judgement` 行照旧打，它的丙段那一格仍是 `not_run`）。
6. 第三段（沿用，这一趟不跑）：每张卡用造池时写的一个真数据单元（32768 字节）、n 取 1 到 16384 段，`GpuCard::crc32c_of_ranges` 先丢一次再计 5 次取中位，核每段 CRC 与 CPU 相同、每次不快过 PCIe 链路峰值；最小二乘拟合 a、b，用第一段的 B_批 算 φ 与 n*，用第二段的 16 线程饱和吞吐算 D24（后台重活能不能卸给 GPU） 已定项 3 形式的比（Q9）。
7. 单点核（`geometry-check` 子命令，改动二）：只造池、核几何，打 `name=geometry_attempt`、`name=geometry`、`name=segment_composition`、`name=segment_composition_summary`、`name=overwrite_landing`、`name=pool` 几行与末行，不量；`--overwrite-order slot-order` 照第 1 次重跑那一版造（全部按旧槽号升序，不数副本落点）。

```

**出处 `.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md:286-315`（整段抄，未转述）**

````markdown
### 复跑（第 2 次重跑；命令照重跑登记第五节「跑法」，起法在 `scripts/experiment-apparatus/e185-r2-segment-run.sh` 文件头）

```bash
# 仓根。装置在验证包里，编与单测归主 agent 经 task-server 交（主工作区那一版 118 条）
SINGLEFS_RUN_SCOPE='…' task-server/task submit -- cargo test --release -p singlefs-verification --bin e185_gc_batch_wall_clock_split
# 三趟（单点核、第一段、第二段）各一条命令，主 agent 在本机仓根经 task-server 交；派哪台由服务定，两台在同一棵共享的树里跑，产物直接写进这棵树的 `research/results/`、两台都读得到；`--disk-model` 填重跑登记第五节点名的那块盘的型号，派到的那一台没有那块盘时入口退 78、一步都不编不跑；
# <段> 取 geometry-check、wall-clock、stage-split，<日期> 是那一趟起跑当天，产物名见上面「这一趟的产物」
SINGLEFS_RUN_SCOPE='…' task-server/task submit -- env E185_RUN_DATE=<日期> bash scripts/experiment-apparatus/e185-r2-segment-run.sh <段> --disk-model <盘的型号>
```

`e185-r2-segment-run.sh` 在共享的树里、同一个绝对路径上做四步（编译目录取 `CARGO_TARGET_DIR`：派到第二台时是统一任务服务把这一趟派过去时在第二台本地盘上新建、跑完删的那一个（服务不设、只记下它在哪，`task-server/task status` 看得见），没设是树里的 `target`）：写输入清单 `research/results/e185-r2-inputs-<段>-<日期>.txt` → `cargo build --release -p singlefs-verification --bin e185_gc_batch_wall_clock_split`（不带 `gpu` 特性，不带 `--all-targets`、`--tests`，免得并进 `lookup-work-counters`）→ 跑这一段（装置线程不设，取登记值 `.claude/heavy-thread-default`，环境 `E185_REGISTERED_APPARATUS_SHA256`、`E185_MEMORY_CAP=40G`）→ 产物写进 `research/results/`：

```bash
# geometry-check 段（单点核，两条）
${CARGO_TARGET_DIR:-target}/release/e185_gc_batch_wall_clock_split geometry-check --point 0.25:1024 --point 0.75:1024 > research/results/e185-gc-batch-wall-clock-split-geometry-check-r2-<日期>.out
${CARGO_TARGET_DIR:-target}/release/e185_gc_batch_wall_clock_split geometry-check --point 0.25:1024 --point 0.75:1024 --overwrite-order slot-order > research/results/e185-gc-batch-wall-clock-split-geometry-check-slot-order-r2-<日期>.out
# wall-clock 段（第一段；真盘臂的镜像目录取 `E185_IMAGE_DIRECTORY`，没设取 `<MULTI_HOST_RUN_PEER_SCRATCH>/e185-images`：派到第二台时是统一任务服务把这一趟派过去时在第二台本地给的目录，不在 NFS 上、不在开机清空的目录里，跑完连目录删；没有这个变量（在本机跑）取 `${CARGO_TARGET_DIR:-target}/e185-images`；镜像目录要落在 `--disk-model` 点名的那块盘上）
${CARGO_TARGET_DIR:-target}/release/e185_gc_batch_wall_clock_split wall-clock --image-directory <镜像目录> > research/results/e185-gc-batch-wall-clock-split-wall-clock-r2-<日期>.out
# stage-split 段（第二段；只用内存盘，与第一段同一台、先后跑）
${CARGO_TARGET_DIR:-target}/release/e185_gc_batch_wall_clock_split stage-split > research/results/e185-gc-batch-wall-clock-split-stage-split-r2-<日期>.out
```

三趟在第二台先后跑、不同时跑；装置线程不设，取登记值 `.claude/heavy-thread-default`（跑的那一刻是 31）；产物文件名的日期是那一趟起跑当天，同一天再跑加 `-2`、`-3`，不盖已有的。三趟当时跑的是合并之前仓根的 `e185-r2-tools/run-<段>.sh`，每段做的事与上面相同。

第三段与第四段这一趟不跑。主 agent 定要 f = 0.25 造成了的点上的 Q9 时另跑第三段（条件照重跑登记第五节「第三段跑不跑」，三丁换成「主 agent 定要」；命令形态照第 1 次重跑，`--wall-clock-product`、`--stage-split-product` 换成这一趟的两份 `-r2-` 产物，带 `gpu` 特性另编）：

```bash
task-server/task submit -- bash -c 'cargo run --release -p singlefs-verification --features gpu --bin e185_gc_batch_wall_clock_split -- gpu-submission --wall-clock-product research/results/e185-gc-batch-wall-clock-split-wall-clock-r2-2026-10-23.out --stage-split-product research/results/e185-gc-batch-wall-clock-split-stage-split-r2-2026-10-23.out --cards 0,1 > research/results/e185-gc-batch-wall-clock-split-gpu-submission-r2-<日期>.out'
```

````

**出处 `.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md:316-330`（整段抄，未转述）**

```markdown
### 口径

- CPU 时间是本线程 `clock_gettime(CLOCK_THREAD_CPUTIME_ID)` 两次读数之差（登记第一节，第十二节 12.2 修订；原来的 schedstat 读法对正在跑的线程约 1 ms 才前进一次，实测数在 12.2）。`wall-clock`、`stage-split`、`foreground-fsync` 开跑先在本线程上自旋 1 ms、连读这个钟，前进不到 100 次就停机退 3（产物行 `name=clock_resolution_check`）。设备调用那一层 CPU 读数包着挂钟读数，等盘 = 挂钟 − CPU，纯 CPU 的设备调用里可以是负的（差读钟的工夫）；一次 `run_compaction` 调用那一层挂钟读数包着 CPU 读数。这样嵌套，两个钟走得一样快时「挂钟 ≥ CPU + 等盘」不会因为读钟的先后被打破。
- 一批的挂钟是包住那一次 `run_compaction` 调用（含调用末尾的 discard）的 `Instant`；discard、屏障、写零各按调用种类单列。
- 真盘臂：每次测量之前把上一次改过的 4 MiB 块从起始件拷回工作件，再核整份镜像的分块摘要（每 4 MiB 一块的 SHA-256 接起来再算一次 SHA-256）与起始件相同；内存臂每轮按同一份配方重造、核两块盘的分块摘要与第一次造的相同。挂载时 journal 记录偏移的提示照抄单元区之前写过的扇区（同内存盘的答法），不在 `O_DIRECT` 镜像上把整个环逐槽读一遍；计时窗口里不照抄。
- 设备：内存臂是 harness 的稀疏内存盘；真盘臂是两个镜像文件，放在第二台一块 NVMe（型号 CT2000T500SSD8，逻辑块 512，`research/results/e185-r2-inputs-wall-clock-2026-10-23.txt` 第 22–26 行）上的 ext4 里，屏障是 `fdatasync`，结论只说「第二台那块盘上 ext4 的 `O_DIRECT` 镜像文件」（重跑登记第一节「真盘」）；原登记写的本机那块盘没用。
- 批：一次调用里搬了单元的那一次（与会话 `observations.batches_published` 同一口径）；签发单列，不算批。
- Q4 的 `rises` 只要求第一批挂钟的比 > 1（5 轮中位，且每轮 > 1）；这一趟 12 对的中位在 1.015102–1.074115，涨幅小，判据不看涨了多少。
- Q7 的「一批 CPU」取这条意图批 CPU 的中位（装置从原登记起的读法，登记第六节「丙段怎么有读数」写死）；丙段的值是签发那一次 `name=stage` 行的 `choose_share_per_batch`，每轮取那一轮有效意图的一个，所以 `name=choose_judgement` 每格只有 5 个值。
- 第一行阳性对照（登记第五节阳性对照表第 1 行，CPU 臂等盘那一句照第十二节 12.9）：取样点 f = 0.75、P = 1024、K = 8，每次 `write_at` 里睡 2 ms（盘臂）或按线程 CPU 时钟自旋 2 ms（CPU 臂）。增量一律是对照各轮的中位 − 基线各轮的中位。盘臂：整条意图等盘增量落在写调用次数 × 2 ms 的 [1.0, 1.5] 倍里、CPU 变化在 ±10% 以内（没改）。CPU 臂两句各自报：CPU 增量落在写调用次数 × 2 ms 的 ±10% 里（没改）；写调用（注入所在的那一种调用）的等盘增量的绝对值 ≤ 写调用次数 × 2 ms × 10%（12.9 的新判法，分母是注入量）。原判据「整条意图等盘相对基线的变化在 ±10% 以内」照算照报（`wait_change`、`wait_clause_under_the_original_criterion_holds`），不作判定用。对照点每一轮里，每个设备臂挨着量基线、盘臂对照、CPU 臂对照各一次，次序按轮号轮换（第 1、4 轮 基线 → 盘臂 → CPU 臂，第 2、5 轮 盘臂 → CPU 臂 → 基线，第 3 轮 CPU 臂 → 基线 → 盘臂），轮数 5；按轮号配对的增量中位另报、不判。
- 作废与停机（登记第十一节）：原登记八条作废、12.10 第 ③ 条与第 1 次重跑的「计数特性被并进来」照沿用；修订 R2-1 加第三样触发（这条意图前后的校准比有一个 > 1.10，`invalid_reason=calibration_ratio_above_the_registered_edge`），R2-2 作废后先等常态再重试，R2-3 校准自己的阳性对照不过就停（退 4），R2-4 基线漂移只打标记；停机条款第 7 条（装置变了，输入清单第一行与登记的哈希对不上，另有修订 R2-8 在装置里也核一遍）至第 10 条（校准锚点不符）与修订 R2-5、R2-7 把基线读不到、锚点不符的遍数累计超过 3 遍收严到停（退 3）；修订 R2-6 校准读不到那一条意图记 `invalid_reason=calibration_unreadable`。这一趟这些都没触发（两段判决行 `stop_clause_findings=0`，140 + 60 条意图 `invalid_reason=none`）。
- 第三段的 B_批（沿用）取第 1 次重跑第一段内存臂、第 1 轮、`kind` 为 `batch` 或 `final_batch` 那几行的 `scan_read_bytes_in_the_region`；φ 与 n* 按两张卡各自拟合的 a、b 算。
- 产物不带执行那台机器的路径：第一段 `name=configuration` 不打 `--image-directory` 的值，只打 `image_directory_filesystem`（目录所在设备在 `/proc/self/mountinfo` 里的文件系统种类）、`image_directory_is_cleared_at_boot`（tmpfs / ramfs，或真路径落在 `/tmp` 下为 `true`）与 `image_device`（设备号）；镜像建不成的报错只带文件名（登记第十二节 12.6）；输入清单里第二台的目录换成「〔第二台的目录〕」。
- 键名写全名，登记里的提问与条款编号只当索引：第二段甲、乙、丙三段的份额是 `name=stage` 的 `crc_share`、`scan_increment_share`、`choose_share_per_batch`（签发那一次另报 `choose_share_of_the_issue`，丙段判定读 `name=choose_judgement`）；Q16 是第一段判决行的 `gc_to_sequential_write_bandwidth_maximum_ratio` 与 `gc_to_sequential_write_bandwidth_judgement`；F1、F2、F5 是 `clause_wall_clock_not_rising_with_the_batch_triggered`、`clause_sixteen_thread_form_misfits_the_gc_triggered`、`clause_two_cpu_paths_disagree_points`；Q17 是第四段判决行的 `foreground_fsync_latency_judgement`。新旧对照表在登记 12.6。

```

**出处 `.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md:331-348`（整段抄，未转述）**

```markdown
### 它答不了的

- f = 0.25 量到的是被强制选定的那一段，不是 greedy 自己选中的段：改法甲下 greedy 选中的仍是装着别的单元的段（单点核第 3 行 `greedy_formed=false`），f = 0.25 与 f = 0.75 一样走只供测试的强制选段后备。这一页的第 1 行读数只对「被强制选定的、全是这次写的数据单元的段」成立，对 greedy 自己会选中的段没有读数。
- 改法甲造成的几何靠一条机理（根环 24 条根、旧落点在覆盖写那一次发布就记成已释放），机理核对只在四个点（f = 0.25、0.75 × P = 1024、8192）与单点核上对上；别的 f、P、别的覆盖写次序没试，也没试登记里的改法乙（快照留旧单元，装置没实现）。
- 占比 0.75 的池与第 1 次重跑不同（差在两棵树，「几何」一节），所以这一趟的数不与第 1 次重跑的数逐格比；七·6 前后的比法这一趟没有。
- 第 3、4 行沿用第 1 次重跑：第三段的 Q9–Q13 在 f = 0.25 的两点上量的是几何没造成的段（`batch_bytes=16384`、`merged_buffer_bytes=16` 不是显存占用：B_批 16384 字节不满一个 32768 字节的槽，装置把每批段数整除成 0 再算缓冲），f = 0.25 造成之后的 φ 与 n* 没量；第三段装置 `eadb929d…` 与重跑登记写死的 `b3a50d50…` 差的 3 行不在第三段路径上这一句是读 `git diff` 推的，没另量，主 agent 已认。
- Q9 照原登记算、照原样报；今天的 D26（后台整理与放置回收） 已定项 13 第 7 条写 D24（后台重活能不能卸给 GPU） 已定项 3 按挂钟比的判据不管它罩着的后台任务，Q9 在第 2 行还算不算翻面量，这一页不判，交主 agent，并进七·2 设计轮定（重跑登记第六节、第一节）。
- 第 1 行读数的 P 轴只有 1024、8192 两点（装置只收这两个池大小）：翻面发生在两点之间的哪里、更大的池上是什么，没量。
- Q4 的 `rises` 涨幅小（1.5%–7.4%）：「批大小是不是挂钟的主旋钮」这个问法这一页不下判断，F1 没触发只说明每一对都 > 1。
- 校准门槛 1.10 的依据只有本机上一个代用量（登记第四节第 11 行，zlib 的 CRC-32 在本机、负载 18 时的分布），第二台与装置自己的 CRC-32C 没量过；门槛在跑前写死，没看这一趟的数再改。这一趟没有一条意图越过它（最大 1.077323），所以它实际没动任何一格（N14 并列判定 24 行无一差异），也就不能说明门槛放在哪合适。追指针校准比（N15）每条意图都高于 1（1.23–1.53），没查原因。
- 校准只量测量线程此刻跑同一份活要多久，不排除对象自身（GC 路径）在个别意图里变慢；H3、H4 以 F5 打中为前提，这一趟没打中，两条没用上。F5 没触发也不能反推第 1 次重跑的 F5 是机器状态：这一趟的机器状态读数（`others_share` 最大 0.002502）与第 1 次重跑没有同一口径的对照读数。
- 第二段清场记录第一行是 `joined-and-read-back holders=1`：别的会话已登记同用卡，本趟加入而不是新停；计时窗口里别的会话有没有占第二台的 CPU，只能读 `name=machine_state` 的 `others_share`（最大 0.001875）与每条意图的排队等 CPU 读数，那是读数不是隔离。
- 释放前读盘核那一栏（登记第十二节 12.5）只认这一批搬迁源的槽。同一次发布对被换下的旧单元（重写的树节点这类）做的释放前读盘核走同一个函数、也在第一次写之前，落在 R 外，仍记在 `scan_read_bytes_elsewhere`：这一条是读 `transaction.rs` 推的，没量。分栏靠「区域扫描在释放前读盘核之前、扫描对 R 那块盘每个槽只读一遍」这个次序；调查员只在 f = 0.75、P = 128、K = 8 一个点上、只对 R 那块盘抓过调用方，另一块盘那一份是推的，单测端到端那一条在同一个点上钉住两块盘合计 2 × 搬的单元数 × 32768。甲段第二个子项不进 Q5、Q8、Q10 与第三段的 B_批，只报数。
- 几何后备：greedy 将选的段装着节点或打包单元时，登记只举了「多覆盖写那几段、让节点段的旧节点也被引用着」两种改法，装置只实现了前一种；竞争段装着别的单元时直接走后备、不试三次（登记第十二节 12.3 写死了这个读法），`name=geometry` 行写明原因；后一种改法没有试过。
- 第二段的饱和吞吐量的是重放（同一份输入反复跑），不是 GC 自己在多线程上跑：今天的 GC 整条路在调用线程上同步跑。
- 第一行阳性对照照 12.9 判「中位之差」，按轮配对的增量只报不判（这一趟 CPU 臂真盘格 `write_wait_increment_paired_median_nanoseconds=1684671.000000`，内存格 `67472.000000`）；交错之后配对判不判、怎么判没有定。
- 停机条款第 6 条（包含自检反过来）照登记字面判，但两个钟走得不一样快：线程 CPU 时钟不受校时调速，`Instant`（`CLOCK_MONOTONIC`）受。2026-10-19 执行员在本机用 python 量过（忙等、挂钟读数包着 CPU 读数）：200 ms 的窗口里 CPU 读数最多比挂钟多 1526 纳秒（比值最大 1.000007630），`CLOCK_MONOTONIC` 比 `CLOCK_MONOTONIC_RAW` 慢 7.036 ppm；挂钟换成 `CLOCK_MONOTONIC_RAW` 时 CPU 读数没有一次大过挂钟（200 ms 窗口最多 −194 纳秒）。几乎全在 CPU 上、又长到几百毫秒的调用（内存臂）因此可能让第 6 条触发，读钟那几十纳秒的余量盖不住；与线程 CPU 读数配对的挂钟已照 12.4 换成 `CLOCK_MONOTONIC_RAW`，这一趟两段判决行都是 `stop_clause_findings=0`，第 6 条没触发。

```

**出处 `.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md:349-368`（整段抄，未转述）**

```markdown
### 路径与结论登记

同一个量两条或以上互不共享代码的路径各算一遍再比对的，有五样：

| # | 路径 | 怎么算 | 源码落点 | 读了哪些共用项 |
|---|---|---|---|---|
| 1 | 扫描读字节：设备计数 | 计时包装记下的、第一次写之前落在 R 那块盘 R 里的读，扣掉这一批搬迁源槽上的再读（释放前读盘核，登记第十二节 12.5），长度相加 | `crates/singlefs-implementation-experiments/src/bin/e185_gc_batch_wall_clock_split/sampling_point_pool_and_geometry.rs:81 split_the_reads_of_a_call` | 区域 R（会话签发的那一条意图）、槽字节 16384、这一批搬迁源的槽（会话交回的 `units_relocated_by_this_publish` 的 `source`）、读的次序（扫描在释放前读盘核之前） |
| 2 | 扫描读字节：分配记录 | 调用之前那一版 R 那块盘上未释放记录的 Σ 跨度 × 16384 | `crates/singlefs-implementation-experiments/src/bin/e185_gc_batch_wall_clock_split/timed_block_device.rs:160 expected_scan_read_bytes` | 同一个 R、同一个槽字节常量 |
| 3 | 批数：装置分类 | 每次调用发成的那一版里 logged_ops 还有没有意图、搬了几个单元 | `crates/singlefs-implementation-experiments/src/bin/e185_gc_batch_wall_clock_split/timed_block_device.rs:100 kind_of_the_call` | 会话交回的那一版 |
| 4 | 批数：会话计数 | 会话 `observations.batches_published` 的增量 | `crates/singlefs-implementation-experiments/src/bin/e185_gc_batch_wall_clock_split/intent_voiding_and_result_lines.rs:33 stop_findings_of` | 无（会话自己数） |
| 5 | 批数：闭式 | ⌈m ÷ K⌉ | `crates/singlefs-implementation-experiments/src/bin/e185_gc_batch_wall_clock_split/call_kinds_and_byte_anchors.rs:69 batches_of_one_intent` | 活槽占比定的 m |
| 6 | 一批的 CPU：真盘臂线程 CPU 时钟 | 真盘上一批的本线程 CPU | `crates/singlefs-implementation-experiments/src/bin/e185_gc_batch_wall_clock_split/main.rs:759 thread_cpu_nanoseconds` | 本线程 `clock_gettime(CLOCK_THREAD_CPUTIME_ID)`（clock id 3） |
| 7 | 一批的 CPU：内存臂挂钟 | 内存盘上同一批的挂钟（等盘近 0） | `crates/singlefs-implementation-experiments/src/bin/e185_gc_batch_wall_clock_split/timed_block_device.rs:296 cpu_reconciliation` | 同一份配方造的池；`Instant`（`CLOCK_MONOTONIC`），与路径 6 的钟走得不一样快（停机条款第 6 条那一条，口径里） |
| 8 | 造成段数：逐段成分 | 单元区里每一段（有未释放记录的段）逐段数成分，数「造成」为真的段（`name=segment_composition_summary` 的 `formed_segments`） | `crates/singlefs-implementation-experiments/src/bin/e185_gc_batch_wall_clock_split/pool_construction.rs:451 construction_reading_lines` | `is_the_geometry_formed`（活槽 = 64f 且全是这次写的数据单元，第 2784 行）、`composition_of_the_segment`、同一份分配记录 |
| 9 | 造成段数：几何核那条路数 | 对着可写挂载后的那一版，按造池记下的数据单元槽所在的段起点筛「造成」为真的段（`geometry_formed_segments`） | `crates/singlefs-implementation-experiments/src/bin/e185_gc_batch_wall_clock_split/sampling_point_pool_and_geometry.rs:166 predict_the_greedy_choice` | 同路径 8：`is_the_geometry_formed`、`composition_of_the_segment`、同一份分配记录 |
| 10 | 落进旧洞的副本：逐次数 | 造池第 2 步每次覆盖写之后，看这一份新副本的槽是不是前面某次覆盖写空出来的旧槽（`copies_landing_in_holes_of_this_construction`） | `crates/singlefs-implementation-experiments/src/bin/e185_gc_batch_wall_clock_split/pool_construction.rs:134 landings_of_the_copies` | 覆盖写的次序（改法甲的 `overwrite_order_of`）、每次覆盖写前后的槽号 |
| 11 | 落进旧洞的副本：闭式推的预测 | `overwritten_units − 24`（24 = 根环条数） | `crates/singlefs-implementation-experiments/src/bin/e185_gc_batch_wall_clock_split/pool_construction.rs:158 predicted_copies_landing_in_holes` | 本地常量 `LOCAL_ROOT_RING_ENTRIES`（开跑与 `singlefs_format` 的两个常量之积回比）；机理前提：每次覆盖写一次发布、一条根，洞从第 j + 24 次发布起分得到，用户数据取最低的空槽对 |

源码落点的行号按主工作区那一份装置（sha256 前 16 位 `0bd2274383f12e88`）现查。跨路径的比对：1 与 2 由停机条款第 2 条逐批连起来（`stop_findings_of`，对不上就停）；3 与 4 由停机条款第 3 条连起来，3 与 5 只在几何造成了的点上连（同一函数）；6 与 7 是 Q3，> 20% 记第十条 F5，不停；8 与 9 由 `counts_agree` 连起来（`construction_reading_lines` 里 `formed_segments == prediction.geometry_formed_segments.len()`，单测 `construction_reading_lines_compare_the_two_counts_of_formed_segments_and_report_the_landing`，第 13124 行），对不上写 `counts_agree=false`、不停机，8 与 9 共用同一个判成造的函数，这个函数错了两边一起错，由「甲造成」那一格 `live_slots=16 slots_of_this_workload=16 other_slots=0` 的绝对值兜；10 与 11 只在 `name=overwrite_landing` 里并列打出，对不上记第十条 H2、不停机不作废，单测钉了闭式的四个值（第 12344–12347 行），没有一处断言把逐次数的与闭式的连起来，还欠这一条。取值是枚举点（两个 f、两个 P、三个 K、两个设备；8 至 11 是四个 f、P 点），理由是登记第八节只要求这几档；1 与 2 共用同一个 R，R 取错时两边一起错，由第七节「签发那一步扫 R 读到的字节」的绝对值锚点兜。

```

**出处 `.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md:369-377`（整段抄，未转述）**

```markdown
### 影响的决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D26（后台整理与放置回收） 已定项 13 | 支撑 | 2026-11-19 不受影响：只把页首现状那一行的装置 sha256、行数与测试条数改成现值，没有重跑，结论、数与产物不变；此前 2026-11-05 不受影响：改名只动措辞，结论、数与产物不变；此前 2026-11-03 不受影响：只改复跑写法，结论、数与产物都不变；此前 2026-11-02 不受影响：只改复跑一节的写法（派哪台、编译目录与镜像目录照共享的树改写，命令补上 `--disk-model`），结论、数与产物不变；此前 2026-11-01 不受影响：只改复跑命令写法，结论、数与产物不变；此前 2026-10-26 改了：第 3、4 次重跑的读数（一次 GPU 提交的主机 CPU，按 CPU 写法、卡、提交间隔拆开的不赔格数，主 agent 现数的表在 `research/prompts/m4-gpugc-design-r3-main-verification.md` 第二节）进了该分项依据段，撑第 1 条放上 GPU 的三段与射程里快写法下赔的卡与间隔；此前 2026-10-24 不受影响：补上第 3、4 次重跑产物点名，数与结论不变；此前 2026-10-23 不受影响：第 2 次重跑给出的是里程碑四设计轮要的读数（问题单第 1 行：池小（P = 1024）两边都占、池大（P = 8192）CPU 限；第 2 行：甲、乙、丙三段份额都不到 10%，F7 打中；第 3 行沿用第 1 次重跑 Q13 全判要攒批；第 4 行单跑够，比值 0.015977），该分项射程把「GPU 上算 GC 的哪几段」与定时兜底间隔、写带宽上限的默认值留给设计轮与 C412（整理的挂钟代价没量过），依据段是「无实验」、没引 E185（GC一批搬迁的挂钟拆分）；Q9 的用法与第 7 条是否相左，这一趟没重算（第三段沿用），Q9 算不算翻面量交主 agent，并进七·2 设计轮定；决策正文这一次没有改 |
| D26（后台整理与放置回收） 已定项 1 | 备料 | 2026-11-19 不受影响：只把页首现状那一行的装置 sha256、行数与测试条数改成现值，没有重跑，结论、数与产物不变；此前 2026-11-05 不受影响：改名只动措辞，结论、数与产物不变；此前 2026-11-03 不受影响：只改复跑写法，结论、数与产物都不变；此前 2026-11-02 不受影响：只改复跑一节的写法（派哪台、编译目录与镜像目录照共享的树改写，命令补上 `--disk-model`），结论、数与产物不变；此前 2026-11-01 不受影响：只改复跑命令写法，结论、数与产物不变；此前 2026-10-24 不受影响：补上第 3、4 次重跑产物点名，数与结论不变；此前 2026-10-23 不受影响：这一趟装置照旧读它的停机谓词（R 按记录全空）判最后一批，不量三水位阈值；f = 0.25 几何这一趟用改法甲造成了，是负载次序的事，与触发判据无关；该分项依据段没引 E185（GC一批搬迁的挂钟拆分），决策正文这一次没有改 |
| D24（后台重活能不能卸给 GPU） 已定项 3 | 备料 | 2026-11-19 不受影响：只把页首现状那一行的装置 sha256、行数与测试条数改成现值，没有重跑，结论、数与产物不变；此前 2026-11-05 不受影响：改名只动措辞，结论、数与产物不变；此前 2026-11-03 不受影响：只改复跑写法，结论、数与产物都不变；此前 2026-11-02 不受影响：只改复跑一节的写法（派哪台、编译目录与镜像目录照共享的树改写，命令补上 `--disk-model`），结论、数与产物不变；此前 2026-11-01 不受影响：只改复跑命令写法，结论、数与产物不变；此前 2026-10-24 不受影响：补上第 3、4 次重跑产物点名，数与结论不变；此前 2026-10-23 不受影响：Q9 沿用第 1 次重跑第三段（72 行全判 GPU 输、F2 没触发），这一趟没重算；第二段又量了 CPU 一侧的 16 线程饱和吞吐（甲 48832371460.2–49032300970.9 B/s），没有代进该分项的形式；该分项依据段没引 E185（GC一批搬迁的挂钟拆分）；Q9 算不算问题单第 2 行的翻面量交主 agent；决策正文这一次没有改 |
| D24（后台重活能不能卸给 GPU） 已定项 2 | 不影响 | 2026-11-19 不受影响：只把页首现状那一行的装置 sha256、行数与测试条数改成现值，没有重跑，结论、数与产物不变；此前 2026-11-05 不受影响：改名只动措辞，结论、数与产物不变；此前 2026-11-03 不受影响：只改复跑写法，结论、数与产物都不变；此前 2026-11-02 不受影响：只改复跑一节的写法（派哪台、编译目录与镜像目录照共享的树改写，命令补上 `--disk-model`），结论、数与产物不变；此前 2026-11-01 不受影响：只改复跑命令写法，结论、数与产物不变；此前 2026-10-26 不受影响：该分项射程补了一句一次 GPU 提交的实测挂钟量级（第 4 次重跑约 5.3 ms），依据段没引 E185（GC一批搬迁的挂钟拆分），条款判据不改；此前 2026-10-24 不受影响：补上第 3、4 次重跑产物点名，数与结论不变；此前 2026-10-23 不受影响：只作定义引用（GPU 结果要复核、GPU 与 CPU 两份实现）：第三段逐段对拍 CRC 的 32 行 `crcs_agree=true` 是第 1 次重跑第三段的装置作废检查，这一趟第三段不跑，不量运行时两份 GPU 核各算一次比对的复核政策，结论不变 |

```

**出处 `.claude/kb/experiments/158-择根与修复四岔路.md:1-26`（整段抄，未转述）**

```markdown
## E158 择根与修复四岔路 —— 第 5 次重跑已跑（够判）：四档两臂与一线程对拍已出最终产物，第 4 次跑已交（岔路 1、2 用户已定，3 记欠）

**中文** · [English](158-four-forks-in-root-selection-and-repair.en.md) · [日本語](158-ルート選択と修復の四つの分岐-第4回実行を提出済み-分岐1・2はユーザーが決定、3は未完.ja.md)

<!-- doc-lint:not-numbers S1 S2 S3 S4 S5 V1 V2 V3 V4 F1 F2 F3 F4 F5 F6 F7 F8 F9 F10 H1 H2 H3 PC1 PC2 PC3 S16 -->
本文里 S1–S5、V1–V4、F1–F10、H1/H2/H3、Q1-1/Q1-2/Q1-3/Q1-4、Q2-1/Q2-2/Q2-3、Q3-1/Q3-2、Q4-1/Q4-2、PC1/PC2/PC3/PC-Nw、N_w
这些编号与代号都是 `e158-preregistration.md` 跑前登记自己的判据 / 历史族 / 阳性对照代号，登记位在那份文件里，不是 kb 的 D / E / C / I 编号。

<!-- doc-lint:registry name-col=2 -->

| 编号 | 简称 |
|---|---|
| G0 | 主几何点 |
| P331 | 打中判据 |
| P332 | 违例判据 |

G0（主几何点）、P331（打中判据）、P332（违例判据） 是本页自己的本地标签（跑前登记 5.1/5.0 定义：G0（主几何点） = (2, S=8, 环 3 MiB) 这个主要
取样几何点；P331（打中判据） 是岔路 2「打中」的判定过程；P332（违例判据） 是岔路 3「违例」的判定
过程），登记位就在这张表，不在跑前登记文件里另登记第二份。

<!-- doc-lint:not-numbers K0 K1 K2 K3 K4 S0 S6 F11 F12 -->

<!-- doc-lint:not-numbers Q0 Q1 Q2 Q3 Q4 Q5 Q6 Q7 Q8 Q9 L0 L1 L2 L3 L4 L5 L6 F13 F14 F15 F16 F17 F18 F19 S7 S8 S9 V5 C0 M3 PC-22 PC-554 -->

<!-- doc-lint:not-numbers P1 P2 P3 P4 P5 P6 V0 -->

```

**出处 `.claude/kb/experiments/158-择根与修复四岔路.md:27-107`（整段抄，未转述）**

````markdown
### 2026-11-12：第 5 次重跑 S4 一档两臂（每臂 8 片加 1 次只取库的合并），四档齐

**状态**：S4 一档的 today 臂与 skip 臂出了最终产物，第 5 次重跑登记 11.3 的九份产物齐了。按登记的够判条件，问题单 P1、P2、P3、P5 四档都量到，够判；P6 在 2026-11-11 已够判；P4 是读码表（登记 5.9）。够判后未跑（登记 11.3 够判停机）：Q9（k = 2）、5.9 里标「第三段」的几行（只坏第一次读、回退与抬 F 作 op1）、GPU 判段对拍。

**跑法**（登记修订 12.8）：S4 一趟推的挂钟超服务的四小时上限，用户定拆短分片。每臂 8 片（`r5-enum-s4 --shard-index n --shard-count 8`，判定写进判定库），8 片都成后同一模式加 `--reuse-only-from-the-library` 起一次合并，判定全从库取，合并的标准输出就是登记 11.3 第 8、9 行的产物。任务号：today 臂片 14659、14704 至 14710，合并 14900；skip 臂片 14660、14711 至 14717，合并 14789。十八个任务的结局都是 `succeeded`、退 0；14659、14660、14707、14709、14710、14716、14717 的第一次尝试被内存包装退 250，产物是第二次尝试写的。单片挂钟 1032 至 3837 秒，合并 398 秒（skip）与 472 秒（today），取自任务日志里 `r5 library_timing` 行的 `wall_milliseconds`。

**树与二进制**：这一档的快照树不是 2026-11-11 那七份的那一棵（12.8 第 6 条）。快照树取自主工作区，其中装置 `crates/singlefs-implementation-experiments/src/bin/e158_root_choice_repair/main.rs` 的 sha256 是 `c1bf6029288b25a19540575b117a4509c6cbfda5d2e24c19d56c1947a5aefeee`（这一版有分片与合并，另经命名回扫改过名），与此刻主工作区那份相同；树内 11314 个文件的逐文件 sha256 清单自身的 sha256 是 `e6094091ed6fb203892045b88537896468a99602a3dbee149b3a737b5292319a`，跑完后在快照树里 `sha256sum -c --quiet` 退 0。skip 副本对同一份清单只有 `crates/singlefs-core/src/mount.rs` 不同，差的就是臂表两行 `r5 skip：`：副本里 `unreadable += 1;` 下一行是 `continue;`，快照树与主工作区里是 `placements_of_an_abandoned_root_whose_account_is_unreadable(`（V5 不触发）。所以这一档的代码键与另三档不同（`r5_run_parameters` 行的 `code_keys=`）。

**进库的产物与 sha256**（拷自草稿目录，拷后逐份相同）：

```
fd50a60ab9dd3b55a10cdd4b69ab2e5823e0bbf2168b58eef2eb7bae2a2ffe9a  e158-root-choice-repair-2026-11-09-r5-enum-s4-today.out
4cf390bd1028e612acfca13de0b17fa245935c03b1f14b2441c84f1f8633f4e9  e158-root-choice-repair-2026-11-09-r5-enum-s4-skip.out
```

**结果**（整行抄自产物）。S4 today 臂，`e158-root-choice-repair-2026-11-09-r5-enum-s4-today.out` 第 44、47913 至 47925、47932、47933 行：

```
E7RESULT name=r5_run_parameters arm=today mode=r5-enum-s4 slots_per_region=4 ring=3145728 step_limit=4 starting_overwrites=[1, 2, 3] threads=31 cut_depth=3 units=1371 judgement_library=prune-server-apparatus-items code_keys=[abandonment:9de00c703555bf4366cf9617a8afffa01309a438f13f8f6f92ba4017923f7a7c twin:f472b1ef1e51fb4b157a869e7d2c7da74a4b36dcaa2095d7c2ad16a3e039b2af account_unreadable:538fa891eac77e418b9290b070f403faf0ce3208f645e83ca3b933fbccc6f365 root_slot_unreadable:e12de3b8ff2e64ad0b5798020a7fb2d3741187269e47069ea935ea903abcf733] judgements_fresh=1 judgements_reused=526547 assembly=reuse_only_from_the_library
E7RESULT name=r5_q0 arm=today slots_per_region=4 ring=3145728 nodes=15956 pairs=47868 constructed=31912 not_applicable=0 not_constructed=15956 constructed_nodes=15956 multi_root_pairs=0 constructed_without_readable_root=0 depth_zero_unit_first_copy_constructed=3 transition_failures=3053 constructed_by_break=unit=15956,unit_first_copy=15956 constructed_by_last_step=M=2742,R=26422,W=2742,start=6 not_applicable_reasons=- not_constructed_reasons=abandoned_set_empty=15956 verdict=valid
E7RESULT name=r5_q1 arm=today slots_per_region=4 ring=3145728 account_variants=382944 mount_errors=0 members=- both_copies_not_intercepted=0 single_copy_not_intercepted=127648 twin_missing_or_mount_error=0 verdict=holds
E7RESULT name=r5_q2a arm=today slots_per_region=4 ring=3145728 both_copies_judged=127648 count_not_through=0 verdict=holds
E7RESULT name=r5_q2b arm=today slots_per_region=4 ring=3145728 single_copy_intercepted=127648 count_not_zero_intercepted=0 single_copy_not_intercepted=127648 count_not_zero_not_intercepted=0 verdict=holds
E7RESULT name=r5_q2c arm=today slots_per_region=4 ring=3145728 twin_cells=31912 twin_count_not_zero=0 verdict=holds
E7RESULT name=r5_q3a arm=today slots_per_region=4 ring=3145728 unshared_cases=127648 isolated_not_twin=0 verdict=holds
E7RESULT name=r5_q3b arm=today slots_per_region=4 ring=3145728 shared_cases=0 isolated_less_than_twin=0 verdict=holds
E7RESULT name=r5_q3c arm=today slots_per_region=4 ring=3145728 single_copy_cases=255296 isolated_not_twin=0 exclusive_slots_empty_cells=0 verdict=holds
E7RESULT name=r5_q4 arm=today slots_per_region=4 ring=3145728 cases=127648 reuse_more_than_twin=0 variants_with_reuse=0 reuse_peak=0 reuse_final_sum=0 reuse_final_maximum=0 first_positive_step=- verdict=holds
E7RESULT name=r5_q5 arm=today slots_per_region=4 ring=3145728 twin_cells=31912 twin_cells_with_reuse=0 reuse_peak=0 first_positive_step=- verdict=none
E7RESULT name=r5_q6 arm=today slots_per_region=4 ring=3145728 cells=31912 variants=63824 mount_errors=0 count_not_zero=0 shortfall_equals_exclusive=63824 shortfall_below_exclusive=0 shortfall_positive=63824 reuse_more_than_twin=9880 variants_with_reuse=9880 reuse_peak=20 isolated_total_zero=63824 verdict=holds
E7RESULT name=r5_q7 arm=today slots_per_region=4 ring=3145728 isolated_not_twin_unshared=0 reuse_more_than_twin=0 verdict=toothless_on_some
E7RESULT name=r5_anchors arm=today slots_per_region=4 ring=3145728 twin_isolates_fewer_than_exclusive=0 continuation_not_at_the_end_txg=0 verdict=holds
E7RESULT name=verdict arm=today slots_per_region=4 ring=3145728 constructed_nodes_positive=true account_mount_errors=0 count_not_through=0 single_copy_count_not_zero=0 twin_count_not_zero=0 unshared_isolated_not_twin=0 shared_isolated_less_than_twin=0 single_copy_isolated_not_twin=0 reuse_more_than_twin=0 root_slot_count_not_zero=0 twin_isolates_fewer_than_exclusive=0 continuation_not_at_the_end_txg=0 positive_controls_not_pass=0 verdict=fields_independent
E7RESULT name=r5_digest arm=today slots_per_region=4 ring=3145728 sha256=f9b2697d4062e19503d47736805e68798d993308bf586d8fd17572ba29c28a46
```

S4 skip 臂，`e158-root-choice-repair-2026-11-09-r5-enum-s4-skip.out` 第 47918、47921、47924、47926、47932 行（其余汇总行除 `arm=` 外与 today 臂同值，`r5_q0` 行逐字相同）：

```
E7RESULT name=r5_q3a arm=skip slots_per_region=4 ring=3145728 unshared_cases=127648 isolated_not_twin=127648 verdict=violated
E7RESULT name=r5_q4 arm=skip slots_per_region=4 ring=3145728 cases=127648 reuse_more_than_twin=18424 variants_with_reuse=18424 reuse_peak=20 reuse_final_sum=42160 reuse_final_maximum=20 first_positive_step=1 verdict=violated
E7RESULT name=r5_q7 arm=skip slots_per_region=4 ring=3145728 isolated_not_twin_unshared=127648 reuse_more_than_twin=18424 verdict=teeth_on_both
E7RESULT name=r5_positive_control arm=skip slots_per_region=4 ring=3145728 control=PC1-a selected="n1=1 steps=start b=unit a=1.4" tag_test_mount_ok=true tag_test_count_is_through=true tag_test_isolated_equals_twin=false tag_test_twin_count_zero=true tag_test_reuse_equals_twin=false tag_arm_isolated_less_on_every_device=true tag_arm_reuse_more_than_twin=true verdict=pass
E7RESULT name=verdict arm=skip slots_per_region=4 ring=3145728 constructed_nodes_positive=true account_mount_errors=0 count_not_through=0 single_copy_count_not_zero=0 twin_count_not_zero=0 unshared_isolated_not_twin=127648 shared_isolated_less_than_twin=0 single_copy_isolated_not_twin=0 reuse_more_than_twin=18424 root_slot_count_not_zero=0 twin_isolates_fewer_than_exclusive=0 continuation_not_at_the_end_txg=0 positive_controls_not_pass=0 verdict=fields_independent
```

**产物自查与判决行**：

- 两份合并产物末行都是 `name=done emitted=47933`；`name=r5_pair` 各 47868 行、`name=r5_local_constant_check` 各 39 行且全是 `verdict=pass`；非 `E7` 开头的行 0；`name=stop` 0；`NOT_RUN` 在 18 份产物（16 片加 2 份合并）里各 0，18 个任务共 25 次尝试的任务日志逐次数（`task-server/task log <号> --attempt <n> --all | grep -c NOT_RUN`）合计 0（V0 不触发）。各片产物同样末行 `name=done emitted=45`、39 条常量回比 `pass`。
- 片对账（12.8 的 S9，不触发）：每臂 8 片的 `cut_depth=3 units_total=1371` 只有一种；`units_in_shard` 相加 1371；`r5_shard_summary` 的 `pairs=` 相加 47868、`nodes=` 相加 15956，等于合并产物 `r5_q0` 的 `pairs=47868`、`nodes=15956`。合并的 `judgements_fresh=1`：任务日志 `DEVICE_CODE_KEY_SENTINEL` 行的 `forced=` 两次合并都是一条 `items/…` 键（不是 `-`），`hits=526548` = 526547 条复用加这一条；各片的 `forced=-`、`hits=0`。任务日志里 `✗` 行 0。
- 判决行：两份各一条 `name=verdict`（第 47932 行）。today 臂没有 `false`、`not_run` 字段，违例类计数全是 0。skip 臂 `unshared_isolated_not_twin=127648`、`reuse_more_than_twin=18424`，是登记预期的（skip 臂是阳性对照，要把 Q3(a)、Q4 推到大于 0）。skip 臂 PC1-a、PC1-b 行里 `tag_test_isolated_equals_twin=false`、`tag_test_reuse_equals_twin=false` 是 today 臂那一栏的句子，skip 臂那一栏要的是 `tag_arm_*`，两个 `true`，判词 `pass`，与 2026-11-11 G0 skip 臂同形。
- today 臂 `r5_q7` 读 `toothless_on_some`：登记 12.3 写 Q7「判词 `teeth_on_both` 或 `toothless_on_some`；skip 臂上读」，第六节 Q7 的量是「skip 臂每档」，V4 的触发观测是「skip 臂该档对应字段读数 0」。today 臂自己 Q3(a)、Q4 的计数是 0，判词必然如此，不是 V4。
- 作废条款：V1 不触发（`constructed_nodes=15956`）；V2 不触发（`both_copies_not_intercepted=0`，PC-复用检测与 PC1-a、PC1-b 的 [测] 句 `pass`）；V3 只在 G0 判，2026-11-11 已不触发；V4 不触发（skip 臂 `teeth_on_both`）；V5 不触发（上面「树与二进制」的 grep）。停机：A2 节点数 15956 ≤ 松上界 124113；A3 `twin_isolates_fewer_than_exclusive=0`；A4 `depth_zero_unit_first_copy_constructed=3`；Q2(c) 0；续写锚点三行 `pass`；S9 不触发。失败条款 F1 至 F4 都没触发：today 臂 Q1 至 Q4 全 0 且 Q7 有牙，PC1-c `pass`、Q6(a) 为 0，Q5 `twin_cells_with_reuse=0`。

**读数（S4 一档）**：

- Q0：15956 个节点、47868 个（节点, 断链方式）对、31912 个构造成立（unit 与 unit_first_copy 各 15956）、15956 个未造出（全是 b = records，原因 `abandoned_set_empty`）；N_c = 15956；深度 0 的 unit_first_copy 成立 3 个。节点数与 Q3-1 的 S4 档（2026-11-09 一节）相同。
- Q1 至 Q4（today 臂）：账读不出变体 382944 个，挂载报错 0；两份坏的 127648 个里计数不等于 through、隔离不等于孪生、复用多于孪生的都是 0；单份坏的 255296 个里计数非 0、隔离不等于孪生的 0 个（其中 127648 个点名的那份没被读到，单列）；孪生 31912 个计数全 0。Q3(b) 共享单元的样本 0 个，这一格是空的。Q4 轨迹三样都是 0。
- Q5：孪生续写复用 0。
- Q6（只报）：63824 个变体，挂载报错 0，计数非 0 的 0 个，短缺等于 U_a 槽数的 63824 个（全部），隔离总数为 0 的 63824 个；复用多于孪生的变体 9880 个，峰值 20。两臂同值。
- Q7（skip 臂）：Q3(a) 违例是全部 127648 个，Q4 违例 18424 个（峰值 20，期末和 42160），首次为正的步序 1。

**四档对照**（G0、S16、小环的读数见 2026-11-11 一节，那三档在另一棵树上跑）：

| 档 | 节点 | today 臂 Q1 挂载报错 / Q2(a) 计数不等 / Q3(a) 隔离不等 / Q4 复用多于孪生 | today 臂 Q6 复用多于孪生的变体数（峰值） | skip 臂 Q3(a) 违例 | skip 臂 Q4 违例（峰值） | skip 臂 Q7 判词 |
|---|---|---|---|---|---|---|
| G0 | 1371 | 0 / 0 / 0 / 0 | 548（20） | 10968 | 1096（20） | teeth_on_both |
| S16 | 1371 | 0 / 0 / 0 / 0 | 656（24） | 10968 | 1312（24） | teeth_on_both |
| 小环 | 1371 | 0 / 0 / 0 / 0 | 548（20） | 10968 | 1096（20） | teeth_on_both |
| S4 | 15956 | 0 / 0 / 0 / 0 | 9880（20） | 127648 | 18424（20） | teeth_on_both |

四档的判决格方向相同，没有「不稳定」（登记 8.2）。

**判（登记 5.8 第二段）**：P1、P2、P3、P5 四档齐，够判。P1 造得出（四档 N_c > 0）；P2 四件在四档都成立（挂载做成、计数 = through、隔离同孪生、独占槽不复用），且四档 P5 有牙；P3 同现行行为的钉子（计数 0、独占槽掉出隔离），复用发生，只报；P5 四档有牙。结论只限这个枚举域（四档、n1 ≤ 3、步序列 ≤ 3（S4 ≤ 4）、k = 1、账两份或单份读不出、形态两种、续写到 txg_a + 3S − 1），不推到别的 op1 和别的形态。

**判定库读写占的时间**（只在标准错误，不进产物；量的）：`(library_read_thread_milliseconds + library_write_thread_milliseconds) / (wall_milliseconds × threads)` 在 16 片上是 0.01 至 0.24，两次合并 0.60（skip）与 0.72（today）。片里核数上不去不是判定库那把锁；合并几乎全在读库。

**它答不了的**：Q3(b) 四档都无样本；k = 2、只坏第一次读、回退与抬 F 作 op1、GPU 判段对拍够判后未跑；S4 与另三档不在同一棵树上，四档对照是两棵树上的读数并列；16 份片产物留在执行员草稿目录（`~/.local/share/singlefs/work/e158-s4-shards/products/`），没进库。

**复跑**：复跑表不加行，理由同 2026-11-11 一节；`.claude/gate.d/replay-comparison-units.tsv` 里 #8、#9 两行改写为产物已出、仍不登记。手跑：在快照树（today）或 skip 副本里，2026-11-11 一节的 `device-code-key.py run` 命令把模式换成 `r5-enum-s4 --shard-index <n> --shard-count 8`（n 取 0 至 7），8 片都成后再起一次 `r5-enum-s4 --reuse-only-from-the-library`，标准输出就是产物。核对命令（输出见上）：`grep -h name=r5_shard_parameters <片>*.out | grep -o 'cut_depth=[0-9]* units_total=[0-9]*' | sort | uniq -c`；`diff <(grep name=r5_q0 …-s4-today.out | sed 's/arm=today//') <(grep name=r5_q0 …-s4-skip.out | sed 's/arm=skip//')`（无输出）。

````

**出处 `.claude/kb/experiments/158-择根与修复四岔路.md:108-209`（整段抄，未转述）**

````markdown
### 2026-11-11：第 5 次重跑（重跑登记 `e158-r5-prereg.md`）：被抛弃根的账读不出，枚举型覆盖补回，G0、S16、小环三档两臂与一线程对拍的最终产物

**状态**：三档（G0、S16、小环）的 today 臂、skip 臂与 G0 的一线程对拍趟已出最终产物，共七份；S4 一档两臂当时没跑（2026-11-12 按片跑出，见上一节「2026-11-12」），下面所有读数都只是这三个取样点上的读数，不外推到 S4，也不写「从来」「恒」。登记的问题单里 P1、P2、P3、P5 的够判条件是四档齐，所以这四行还差 S4 一档；P6（并行不改产物）够判。

**问的是什么**：岔路单第 1 行（被抛弃根的账读不出时「修复」指什么）早已由用户 2026-09-28 定为「(b)-从映射现算」（C393（被抛弃根的账读不出时修复没有条款））；这一次验的是这个定案在今天 `crates/` 上对枚举出的历史成立不成立。起因是 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 乙合入后随机历史与旧枚举都造不出被抛弃根（C599（随机历史造不出被抛弃的根）），只剩写死的用例罩着。

**做了什么**：入库装置 `crates/singlefs-implementation-experiments/src/bin/e158_root_choice_repair/main.rs` 的 `r5-` 模式，起点 n1 ∈ {1, 2, 3}、步序列 ≤ 3，深度优先枚举 1371 个节点；每个节点按三种断链方式（records、unit、unit_first_copy）各造一次被抛弃步，成立的对各做 12 个账读不出变体、2 个根槽读不出变体、1 个孪生，续写到 txg_a + 3S − 1。today 臂是 `crates/` 原样，skip 臂是副本里把影子账的读不出分支改成只计数、不隔离（阳性对照，不是岔路候选）。判定经 prune-server 的判定库（每个判定单位一条），一线程对拍趟不连库。

**树与二进制**：这批产物都在冻结树上出，不是在别的会话还在改的主工作区上出（上一轮的七份因此作废、不进库）。冻结树取自 HEAD `8ed2ea4845ef0023fcf9a41e3ac09fece4423c0d`，当时主工作区另有 340 个未提交文件改动（`crates`、`scripts`、`Cargo.*`、`.cargo`、`prune-server` 下），快照照工作区拍；树内 10793 个文件的逐文件 sha256 清单自身的 sha256 是 `c65d477da50ed02d22dc8ae9d9271f2adf9a01a64aabb294459addbcaf3309e5`，建完那一刻与主工作区逐文件相同。skip 臂副本取自冻结树，`crates/`、`prune-server/`、`Cargo.*`、`.cargo`、`scripts/` 共 1805 个文件与冻结树逐文件相同，再套臂表 `scripts/mutation/tables/e158_arms.tsv` 的两行 `r5 skip：`。冻结树与副本在产物出完后已删，清单与比对结果留在执行员草稿目录。

**登记里打中的条款**：① 停机 S3 字面触发：臂表第二行（给现算函数加 `#[allow(dead_code)]`）的旧串写的是 `<Device: BlockDevice>`，今天的签名是 `<Reader: PoolReader>`，命中 0 次；用户 2026-11-10 定照今天的签名改臂表这一处，登记修订 12.7 写了，原判据不动。② 影子账分支前多了一支「根环表里记着的落点」（读码，`mount.rs` 的 `PlacementsOfAbandonedRoots::RecordedInTheRootRingTableOtherwiseReadFromTheirAccounts`），登记 12.7 写明它对 Q2 计数的影响在产物里量：today 与 skip 两臂 `count_not_through` 都是 0。③ 登记 11.2 的 S7（第一段挂钟超 2 小时）在上一轮已触发并交主 agent 处理；这一轮单趟最长的是一线程趟 6169 秒（任务 12582）。四个任务（12579、12580、12581、12584）前几次尝试撞内存上限退 250，成的是最后一次尝试，产物是最后一次尝试写的。

**进库的产物与 sha256**（`research/results/` 下共八份：七份枚举产物加 S1 回比；拷自冻结树出的草稿目录，拷后逐份 sha256 相同）：

```
2b33a57764009d064fb89d7ce14d987353938c10e0992239c1aace8c3dbcf095  e158-r5-constants-frozen-tree.txt
40e949e616bdfdbc898085abd4810f64d7311e958c99ffa52453e9861a22ed3c  e158-root-choice-repair-2026-11-09-r5-enum-g0-skip.out
d25a113c9eb7ab99d181ce8738ad3fc6b9453a4437482c559b0c75e5a6d2ff14  e158-root-choice-repair-2026-11-09-r5-enum-g0-today-one-thread.out
e3cee885d2da74062567ad9ed007d64ea7390d5b0ccf7281704156f4fbc4ab5e  e158-root-choice-repair-2026-11-09-r5-enum-g0-today.out
4178aa5ab4105414b9953a2bf1efeb9b976c5219e7024bf8dc2823b3ed240225  e158-root-choice-repair-2026-11-09-r5-enum-s16-skip.out
1f4903eea6c7e541a6bdfbed5a7a315297641316236a654a74add8132ca2a479  e158-root-choice-repair-2026-11-09-r5-enum-s16-today.out
5bf7b60b392bff8eda72f69e84252bd411cfc3feab2c1b7de66402b717421598  e158-root-choice-repair-2026-11-09-r5-enum-small-ring-skip.out
da2931d9545c6fb79e52acc8c9146a49abfb3a2fa2e1782caeddd38196ae8949  e158-root-choice-repair-2026-11-09-r5-enum-small-ring-today.out
```

**结果**（整行抄自产物，行号对七份相同处只写一次；today = `research/results/e158-root-choice-repair-2026-11-09-r5-enum-<档>-today.out`，skip = `…-<档>-skip.out`，一线程 = `…-g0-today-one-thread.out`，`<档>` 取 `g0`、`s16`、`small-ring`）：

G0 today 臂，`e158-root-choice-repair-2026-11-09-r5-enum-g0-today.out` 第 4158 至 4170 行与第 4177、4178 行：

```
E7RESULT name=r5_q0 arm=today slots_per_region=8 ring=3145728 nodes=1371 pairs=4113 constructed=2742 not_applicable=0 not_constructed=1371 constructed_nodes=1371 multi_root_pairs=0 constructed_without_readable_root=0 depth_zero_unit_first_copy_constructed=3 transition_failures=402 constructed_by_break=unit=1371,unit_first_copy=1371 constructed_by_last_step=M=268,R=2200,W=268,start=6 not_applicable_reasons=- not_constructed_reasons=abandoned_set_empty=1371 verdict=valid
E7RESULT name=r5_q1 arm=today slots_per_region=8 ring=3145728 account_variants=32904 mount_errors=0 members=- both_copies_not_intercepted=0 single_copy_not_intercepted=10968 twin_missing_or_mount_error=0 verdict=holds
E7RESULT name=r5_q2a arm=today slots_per_region=8 ring=3145728 both_copies_judged=10968 count_not_through=0 verdict=holds
E7RESULT name=r5_q2b arm=today slots_per_region=8 ring=3145728 single_copy_intercepted=10968 count_not_zero_intercepted=0 single_copy_not_intercepted=10968 count_not_zero_not_intercepted=0 verdict=holds
E7RESULT name=r5_q2c arm=today slots_per_region=8 ring=3145728 twin_cells=2742 twin_count_not_zero=0 verdict=holds
E7RESULT name=r5_q3a arm=today slots_per_region=8 ring=3145728 unshared_cases=10968 isolated_not_twin=0 verdict=holds
E7RESULT name=r5_q3b arm=today slots_per_region=8 ring=3145728 shared_cases=0 isolated_less_than_twin=0 verdict=holds
E7RESULT name=r5_q3c arm=today slots_per_region=8 ring=3145728 single_copy_cases=21936 isolated_not_twin=0 exclusive_slots_empty_cells=0 verdict=holds
E7RESULT name=r5_q4 arm=today slots_per_region=8 ring=3145728 cases=10968 reuse_more_than_twin=0 variants_with_reuse=0 reuse_peak=0 reuse_final_sum=0 reuse_final_maximum=0 first_positive_step=- verdict=holds
E7RESULT name=r5_q5 arm=today slots_per_region=8 ring=3145728 twin_cells=2742 twin_cells_with_reuse=0 reuse_peak=0 first_positive_step=- verdict=none
E7RESULT name=r5_q6 arm=today slots_per_region=8 ring=3145728 cells=2742 variants=5484 mount_errors=0 count_not_zero=0 shortfall_equals_exclusive=5484 shortfall_below_exclusive=0 shortfall_positive=5484 reuse_more_than_twin=548 variants_with_reuse=548 reuse_peak=20 isolated_total_zero=5484 verdict=holds
E7RESULT name=r5_q7 arm=today slots_per_region=8 ring=3145728 isolated_not_twin_unshared=0 reuse_more_than_twin=0 verdict=toothless_on_some
E7RESULT name=r5_anchors arm=today slots_per_region=8 ring=3145728 twin_isolates_fewer_than_exclusive=0 continuation_not_at_the_end_txg=0 verdict=holds
E7RESULT name=verdict arm=today slots_per_region=8 ring=3145728 constructed_nodes_positive=true account_mount_errors=0 count_not_through=0 single_copy_count_not_zero=0 twin_count_not_zero=0 unshared_isolated_not_twin=0 shared_isolated_less_than_twin=0 single_copy_isolated_not_twin=0 reuse_more_than_twin=0 root_slot_count_not_zero=0 twin_isolates_fewer_than_exclusive=0 continuation_not_at_the_end_txg=0 positive_controls_not_pass=0 verdict=fields_independent
E7RESULT name=r5_digest arm=today slots_per_region=8 ring=3145728 sha256=c45a0b205083052a9c80389e353481185df53249b3d3aa1bd21dff889ebd9077
```

另三档的差别（同样整行抄自产物）：S16 today 臂 `…-s16-today.out` 第 4168 行；G0 skip 臂 `…-g0-skip.out` 第 4163、4166、4169 行；S16 skip 臂 `…-s16-skip.out` 第 4166、4169 行；小环 skip 臂 `…-small-ring-skip.out` 第 4167、4170 行；一线程趟 `…-g0-today-one-thread.out` 第 44 行：

```
E7RESULT name=r5_q6 arm=today slots_per_region=16 ring=3145728 cells=2742 variants=5484 mount_errors=0 count_not_zero=0 shortfall_equals_exclusive=5484 shortfall_below_exclusive=0 shortfall_positive=5484 reuse_more_than_twin=656 variants_with_reuse=656 reuse_peak=24 isolated_total_zero=5484 verdict=holds
E7RESULT name=r5_q3a arm=skip slots_per_region=8 ring=3145728 unshared_cases=10968 isolated_not_twin=10968 verdict=violated
E7RESULT name=r5_q4 arm=skip slots_per_region=8 ring=3145728 cases=10968 reuse_more_than_twin=1096 variants_with_reuse=1096 reuse_peak=20 reuse_final_sum=2480 reuse_final_maximum=20 first_positive_step=1 verdict=violated
E7RESULT name=r5_q7 arm=skip slots_per_region=8 ring=3145728 isolated_not_twin_unshared=10968 reuse_more_than_twin=1096 verdict=teeth_on_both
E7RESULT name=r5_q4 arm=skip slots_per_region=16 ring=3145728 cases=10968 reuse_more_than_twin=1312 variants_with_reuse=1312 reuse_peak=24 reuse_final_sum=9232 reuse_final_maximum=24 first_positive_step=1 verdict=violated
E7RESULT name=r5_q7 arm=skip slots_per_region=16 ring=3145728 isolated_not_twin_unshared=10968 reuse_more_than_twin=1312 verdict=teeth_on_both
E7RESULT name=r5_q4 arm=skip slots_per_region=8 ring=1048576 cases=10968 reuse_more_than_twin=1096 variants_with_reuse=1096 reuse_peak=20 reuse_final_sum=2480 reuse_final_maximum=20 first_positive_step=1 verdict=violated
E7RESULT name=r5_q7 arm=skip slots_per_region=8 ring=1048576 isolated_not_twin_unshared=10968 reuse_more_than_twin=1096 verdict=teeth_on_both
E7RESULT name=r5_run_parameters arm=today mode=r5-enum-g0-one-thread slots_per_region=8 ring=3145728 step_limit=3 starting_overwrites=[1, 2, 3] threads=1 cut_depth=1 units=18 judgement_library=none_fresh_for_parity_run judgements_fresh=45243 judgements_reused=0
```

三档两臂的逐档读数（数一遍的命令在下面「复跑」一节；列的是各档产物里同名那一行的值）：

| 档 | today 臂 Q1 挂载报错 / Q2(a) 计数不等 / Q3(a) 隔离不等 / Q4 复用多于孪生 | today 臂 Q6 复用多于孪生的变体数（峰值） | skip 臂 Q3(a) 违例 | skip 臂 Q4 违例（峰值） | skip 臂 Q7 判词 |
|---|---|---|---|---|---|
| G0 | 0 / 0 / 0 / 0 | 548（20） | 10968 | 1096（20） | teeth_on_both |
| S16 | 0 / 0 / 0 / 0 | 656（24） | 10968 | 1312（24） | teeth_on_both |
| 小环 | 0 / 0 / 0 / 0 | 548（20） | 10968 | 1096（20） | teeth_on_both |

**产物自查与判决行**：

- 七份产物末行都是 `name=done emitted=N`（G0 与 S16 五份是 4178，小环两份是 4179）；`name=r5_pair` 各 4113 行、`name=r5_local_constant_check` 各 39 行且全是 `verdict=pass`；非 `E7` 开头的行 0；`name=stop` 0 条；`grep -c NOT_RUN` 七份产物各 0，七个任务（12579、12580、12581、12582、12584、12585、12586）每次尝试的任务日志合计 0（V0 不触发）；连库的六趟日志末段都有 `DEVICE_CODE_KEY_SENTINEL`，每趟 `judgements_fresh=45243 judgements_reused=0`。`e158-r5-constants-frozen-tree.txt` 是 S1 回比（任务 12565）：42 条 `verdict=` 全是 `pass`，`NOT_RUN` 0；装置单测（任务 12564）`158 passed; 0 failed`。
- 判决行逐份读：七份各一条 `name=verdict`（行号 G0 与 S16 的五份是第 4177 行，小环两份是第 4178 行）。today 臂三份与一线程趟没有 `false`、`not_run` 字段，所有违例类计数都是 0。skip 臂三份的 `unshared_isolated_not_twin=10968`，`reuse_more_than_twin` 在 G0 与小环是 1096、在 S16 是 1312：这是登记预期的（skip 臂是阳性对照，要把 Q3(a)、Q4 推到大于 0），不是 today 臂的违例。today 臂 `r5_q7` 判词是 `toothless_on_some`，登记写明这个量在 skip 臂上读，today 臂自己的计数为 0 时判词必然如此。
- 作废条款：V1 不触发（三档 `constructed_nodes=1371`）；V2 不触发（`both_copies_not_intercepted=0`，PC-复用检测与 PC1-a、PC1-b 的 [测] 句都 `pass`）；V3 不触发（G0 的 today 与一线程趟的 `r5_digest` 同为 `c45a0b205083052a9c80389e353481185df53249b3d3aa1bd21dff889ebd9077`，去掉 `r5_run_parameters` 之后逐行相同）；V4 不触发（三档 skip 臂 Q7 都是 `teeth_on_both`）。V5 的 grep 输出这一轮没有留存（副本已删），skip 臂改动的生效由 skip 臂 Q3(a) 全部 10968 个违例间接证明，直接的 grep 欠。失败条款 F1 至 F4 都没触发：today 臂 Q1 至 Q4 全是 0 且 Q7 有牙，PC1-c 三档 `pass`、Q6(a) 为 0，Q5 `twin_cells_with_reuse=0`。

**读数（三档，逐量；全是这三个取样点上的读数）**：

- Q0：三档都是 1371 节点、4113 个（节点, 断链方式）对、2742 个构造成立（unit 与 unit_first_copy 各 1371）、1371 个未造出（全是 b = records，原因 `abandoned_set_empty`）；N_c = 1371 > 0；深度 0 节点里 unit_first_copy 成立 3 个，与锚点一致。三档的 `r5_q0` 行除 `slots_per_region`、`ring` 外逐字相同，today 与 skip 同档的 `r5_q0` 行除 `arm=` 外逐字相同（命令与输出见「复跑」一节）。
- Q1 至 Q4（today 臂，三档同值）：账读不出变体 32904 个，挂载报错 0；两份坏的 10968 个里计数不等于 through 的 0 个、隔离不等于孪生的 0 个、续写后复用数大于孪生的 0 个；单份坏的 21936 个里计数非 0、隔离不等于孪生的 0 个；孪生 2742 个计数全为 0。Q3(b)（被其余可读根共享的单元）三档都是 0 个样本，这一格是空的，不是「判过了」。单份坏的变体里一半（10968 个）点名的那份副本在这次挂载里没被读到，照登记单列、不进判据。Q4 轨迹三样（峰值、为正的变体数、期末值）都是 0。
- Q5：孪生续写复用 0（`twin_cells_with_reuse=0`）。
- Q6（根槽读不出那一形）：5484 个变体，挂载报错 0，计数非 0 的 0 个，短缺等于 U_a 槽数的 5484 个（全部），隔离总数为 0 的 5484 个；复用数大于孪生的变体 G0 与小环 548 个（峰值 20）、S16 656 个（峰值 24）。这些只报，不判。
- Q7（skip 臂）与 PC：三档 skip 臂 Q3(a) 违例都是全部 10968 个，Q4 违例 1096（G0、小环）与 1312（S16），首次为正的步序都是 1；PC1-a、PC1-b 的 [臂] 句成立，PC1-c 在 skip 臂与 today 臂逐项相同（skip 不改根槽读不出那条路径）。
- Q8：G0 的 31 线程趟（1371 个单位）与一线程趟（18 个单位）去掉 `r5_run_parameters` 之后逐行相同，摘要同为上面那个 sha256。两趟 45243 个判定都是现判；一线程趟 6169 秒、平均在用 0.927 核，31 线程的 G0 today 趟最后一次尝试 956 秒。
- 三档对照：G0 与小环的结果行（除 `slots_per_region`、`ring` 外）逐行相同，journal 环小一档在这些量上看不出差别；S16 与 G0 只在 Q6 的复用数（656 对 548，峰值 24 对 20）与 skip 臂的 Q4、Q7（1312 对 1096）不同。

**判（登记 5.8 第二段）**：P6 够判：判定键与产物不随线程数变（G0 全量，31 线程对 1 线程），装置单测绿。P1、P2、P3、P5 三档量到，**按登记的够判条件还差 S4 一档，不够判**；三档上的读数：P1 造得出；P2 四件成立（挂载做成、计数 = through、隔离同孪生、独占槽不复用）；P3 同现行行为的钉子（计数 0、独占槽掉出隔离），复用发生（G0、小环 548 个变体，S16 656 个）；P5 判法在三档都有牙。P4 是读码表（登记 5.9），不靠量，这一轮没动。结论只限这个枚举域（三档、n1 ≤ 3、步序列 ≤ 3、k = 1、账两份或单份读不出、形态两种、续写到 txg_a + 3S − 1）。

**与上一轮非最终产物的关系**：上一轮在别的会话还往主工作区打改动期间出过同名的七份，不进库，仍在执行员工作根下 `e158-r5-segment-two/products-not-final/`。用同一条命令比两批：一线程趟逐字节相同，其余六份去掉 `r5_run_parameters` 一行后逐行相同，七份 `r5_digest` 的 sha256 与这一批相同。就是说冻树重出没有改变任何读数。

**它答不了的**：S4 一档当时没跑（2026-11-12 已跑，见那一节）；Q3(b) 三档无样本；k = 2、只坏第一次读的形态、回退与抬 F 作 op1、GPU 判段对拍都不在第一、二段（登记 5.9 写明不泛化）；钩子次数、两片实例表不泛化；V5 的 grep 输出没留存；根环表那一支对 Q2 计数的影响只在产物里量到「两臂 `count_not_through` 都是 0」，没单独造「根环表有记录而账读不出」的历史。

**复跑**：复跑表（`scripts/experiment-apparatus/replay.sh`）没有 E158 第 5 次重跑的行：连库六趟的 `r5_run_parameters` 行带 `judgements_fresh`、`judgements_reused`，复跑必不同，skip 臂要先建副本套臂改动，一线程趟要不要进复跑表留给用户定；九个比对对象在 `.claude/gate.d/replay-comparison-units.tsv` 里逐个点名为「不登记的比对对象」。手跑的命令（today 臂连库、31 线程，在冻结树或仓根起；二进制由 `device-code-key.py run` 在任务的编译目录里编；输出文件名按跑的那天另起）：

```
SINGLEFS_E158_ARM=today python3 scripts/experiment-apparatus/device-code-key.py run . --unit-graph "$CARGO_TARGET_DIR/release/unit-graph" --package-directory crates/singlefs-verification --binary e158_root_choice_repair --entry singlefs_verification::abandonment_judgement_entry --entry singlefs_verification::twin_judgement_entry --entry singlefs_verification::account_unreadable_judgement_entry --entry singlefs_verification::root_slot_unreadable_judgement_entry -- r5-enum-g0
SINGLEFS_E158_ARM=today cargo run -q --release -p singlefs-verification --bin e158_root_choice_repair -- r5-enum-g0-one-thread
```

S16、小环把最后的 `r5-enum-g0` 换成 `r5-enum-s16`、`r5-enum-small-ring`；skip 臂先 `python3 scripts/hosts/minimal-copy.py <冻结树或仓根> <副本> --with-test-inputs --with-gate-tables --with 'scripts=…' --with '.claude/scripts=…' --with '.claude/singlefs-ai-sop=…'` 建副本，再 `python3 apply-skip-arm.py <副本> <副本>/scripts/mutation/tables/e158_arms.tsv` 套两行 `r5 skip：`（套用脚本在执行员草稿目录，是上一轮 `e158-r5-segment-one` 那份的逐字节拷贝；旧串不是恰好命中一次就退 3），在副本里起第一条（`SINGLEFS_E158_ARM=skip`），副本里起连 prune-server 的命令要显式给 `SINGLEFS_MULTI_HOST_CONFIG`。两条核对命令（输出为 0 行）：`diff <(grep -v name=r5_run_parameters 连库趟.out) <(grep -v name=r5_run_parameters 一线程趟.out) | wc -l`；`diff <(grep name=r5_q0 …-g0-today.out | sed 's/arm=today/arm=X/') <(grep name=r5_q0 …-g0-skip.out | sed 's/arm=skip/arm=X/') | wc -l`（S16、小环同）。

**影响的决策**：见页内「影响的决策」一节；D23（journal 的角色与格式） 已定项 14 的现状与三档读数一致，不改关系；D28（挂载期承诺量） 已定项 1 第九项「被抛弃根独占量」取值口径（账读不出时从映射现算的落点也在内）与三档量到的隔离数一致，写备料。

````

**出处 `.claude/kb/experiments/158-择根与修复四岔路.md:210-287`（整段抄，未转述）**

````markdown
### 2026-11-09：岔路单第 3 行换成如实账本重出，岔路 1 的 q1 三行删除

这一节是 E158（择根与修复四岔路） 岔路单第 3 行（第一段 `all` 里的 H3、Q3-2、Q2-3、PC3，第二段 Q3-1 的四个几何点）的现状，下面各节里这几项的读数与产物行（2026-09-23 至 2026-09-25 的各份）由它取代；岔路 1 的 `q1-g0`、`q1-s16`、`q1-s4` 三行，用户定案删除并标作废。岔路 1、2 的承重读数在第 4 次跑里，由 `r4-compare` 那一行承重（现判 `pass`）；第 2 次跑与第 3 次跑的读数不承重（「2026-09-27：第 4 次跑」一节已写明）。同日另删 7 行，见下面的 ⑥。

**更正**：「2026-09-26（第 2 次跑第一段）」历史条写过「岔路单第 3 行不再量」，与现状不符。岔路单第 3 行这五行（`all` 与 q3-1 四档）一直在 `scripts/experiment-apparatus/replay.sh` 复跑表里，这一次按新账本重出，问的是新判据 `content_differs_from_its_effective_root`。作废的只有 C332（回退实例两个根都读不出时回退被撤销） 正文钉的那句「2 个故障就能撤销回退」（回退改成向前发布之后产品没有这个动作，2026-09-25 session s10 一节已记测不出）与 q1 三行。

**做了什么**：装置账本由「截断时间线」改成如实账本，只改 `crates/singlefs-implementation-experiments/src/bin/e158_root_choice_repair/main.rs`，产品代码 `crates/singlefs-core` 没动；旧账本那条路与两个注入开关（`E158_ROLLBACK_LEDGER`、`E158_INJECT`）删了。五份新产物：`e158-root-choice-repair-2026-11-09-segment1-rerun.out`（85 行，其中 73 行是 `E7RESULT`，另 12 行是 `N1_DONE`）、`e158-root-choice-repair-2026-11-09-q3-1-g0.out`（66 行）、`e158-root-choice-repair-2026-11-09-q3-1-s16.out`（66 行）、`e158-root-choice-repair-2026-11-09-q3-1-small-ring.out`（67 行）、`e158-root-choice-repair-2026-11-09-q3-1-s4.out`（66 行），每份末行 `E7RESULT name=done emitted=` 的数等于该份 `E7RESULT` 行的行数；`replay.sh` 的 E158 五行改指它们，2026-09-25 的各份留在 git 历史里。

**① 11907 条假违例的原因与账本修法**：中间一版装置（回退按「截断时间线」记账，任务 10904 的输出里 `pairs=70470 pairs_with_any_violation=11907`，原始输出未留存于仓内）在 G0（主几何点） 上报 weight=0 的 104 条、weight=1 继承的 1510 条、weight=2 继承的 10293 条违例，合计 11907 条。全是装置账本造成的，产品没读错：`mount_rollback` 先做一次可写挂载，真写下一行与暖机的几条根（内容是回退之前的），旧账本把这几条根记成回退目标的内容，又把时间线截到目标；之后再回退到这几条根之一，产品读回它们的真内容（对的），账上那一段已被截掉，V2 判「不在时间线上」。另一处错：旧账本不记回退那条根，下一步被 S5 拒绝（G0 上 M 被拒 76 次、R 被拒 303 次），枚举域被悄悄剪小（G0 节点 679，如实是 1371）。只按账本规则写的算术模型逐数对上了 G0 的 15 个数，含 104 / 1510 / 10293。修法是如实账本：回退是一次向前发布（D23（journal 的角色与格式） 同一份文件里「管理员回退是挂着时的一次向前发布」），时间线整条留着；写行与暖机的根按回退之前最新根的内容记，回退那条根按目标的内容记并追加；`abandoned` 恒空；目标不在时间线上、或所选根不是时间线尾的挂载，按 S5 拒绝，计入 `e158_ledger_diagnostics_refused`（G0 上 402 次）。

**② 判据预言记输一次**：旧判据 `faulted_cold_content_differs_from_its_root`（读回内容与所选根账上内容比）的预言是 0，G0 上量到 10351、S4 上量到 93530。不是内容读丢，是重放走到了时间线尾：所选根不是尾（`faulted_cold_chosen_root_is_not_the_tip=19610`），journal 重放把内容带回尾。本次产物里的直接读数：152027 次带故障冷读读失败 0、读回内容不是时间线尾 0（带回退与不带回退各 0）；不看 journal 的那一臂 `ignore_policy_content_is_not_the_tip` 是 10351，恰好等于旧判据的 10351 条（关掉重放，这 10351 条读回的就不是尾）。这条预言记输一次。新判据 `content_differs_from_its_effective_root` 拿读回内容与 journal 施加之后实际走的那条根（`effective_root`）的账上内容比：G0 与 S4 都是 0；同组的 `effective_root_off_the_timeline`、`differs_rows_with_nothing_applied`、`ignore_policy_content_differs_from_chosen_root`、`no_fault_read_is_not_the_tip`、`no_fault_read_failed` 也都是 0。这六个量任一非 0，装置打 `name=stop` 退 1，四档都没打。旧判据的数留着当读数，不判红。

**③ 四档 Q3-1 读数**（`pairs_with_any_violation` 四档都是 0，`cold_recover_with_fault_failed` 四档都是 0）：

| 几何点 | nodes_visited / pairs（本次） | nodes_visited / pairs（2026-09-25 产物） | mount_writable_with_fault_failed（本次 / 2026-09-25） | faulted_cold_content_differs_from_its_root（本次） |
|---|---|---|---|---|
| G0（主几何点） | 1371 / 153398 | 1822 / 170719 | 115 / 0 | 10351 |
| S16 | 1371 / 153398 | 1822 / 170719 | 115 / 0 | 10351 |
| 小环 | 1371 / 153398 | 1822 / 170719 | 115 / 0 | 10351 |
| S4 | 15956 / 1258176 | 14252 / 1116897 | 1236 / 239 | 93530 |

`mount_writable_with_fault_failed` 的 115 与 1236 各等于同一份产物里 `effective_root_is_not_the_tip` 的数，为什么相等没单查。S4 的 `cold_recover_with_fault_failed` 旧值是 239，本次 0。小环那一点 `small_ring_search` 找到的环长是 1048576（2026-09-25 产物是 49152），这一档与 G0、S16 的各项计数逐项相同，环长这一维上三点没分开；环长为什么变没单查。

`research/results/e158-root-choice-repair-2026-11-09-q3-1-g0.out` 第 57、63、64、65 行：

```text
E7RESULT name=fault_set_violation_summary geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) overwrites_before_sigma=[1, 2, 3] sigma_length_limit=3 nodes_visited=1371 pairs=153398 pairs_with_any_violation=0 cold_recover_with_fault_failed=0 mount_writable_with_fault_failed=115 skipped_by_the_reduction_rule=132417
E7RESULT name=e158_effective_root_summary reads_checked=152027 effective_root_is_not_the_tip=115 effective_root_off_the_timeline=0 content_differs_from_its_effective_root=0 differs_from_chosen_root_rows=10351 differs_rows_with_nothing_applied=0 ignore_policy_reads_checked=19610 ignore_policy_read_failed=0 ignore_policy_chosen_root_changed=0 ignore_policy_content_differs_from_chosen_root=0 ignore_policy_content_is_not_the_tip=10351
E7RESULT name=e158_ledger_diagnostics_summary no_fault_reads_checked=1371 no_fault_reads_just_after_a_rollback=1100 no_fault_read_failed=0 no_fault_read_is_not_the_tip=0 faulted_cold_reads_checked=152027 faulted_cold_read_failed=0 faulted_cold_chosen_root_is_not_the_tip=19610 faulted_cold_chosen_root_off_the_timeline=0 faulted_cold_content_differs_from_its_root=10351 faulted_cold_content_is_not_the_tip_with_rollback=0 faulted_cold_content_is_not_the_tip_without_rollback=0 mount_then_reads_checked=20866 mount_then_content_is_not_the_tip=0 after_fault_removed_reads_checked=20981 after_fault_removed_content_is_not_the_tip_with_rollback=0 after_fault_removed_content_is_not_the_tip_without_rollback=0
E7RESULT name=e158_ledger_diagnostics_refused step_and_error="R mount_rollback: \"挂着回退：TargetNotACandidate" count=402
```

`research/results/e158-root-choice-repair-2026-11-09-q3-1-s4.out` 第 57、63 行：

```text
E7RESULT name=fault_set_violation_summary geometry=S4(S=4,ring=3MiB) overwrites_before_sigma=[1, 2, 3] sigma_length_limit=4 nodes_visited=15956 pairs=1258176 pairs_with_any_violation=0 cold_recover_with_fault_failed=0 mount_writable_with_fault_failed=1236 skipped_by_the_reduction_rule=1050960
E7RESULT name=e158_effective_root_summary reads_checked=1242220 effective_root_is_not_the_tip=1236 effective_root_off_the_timeline=0 content_differs_from_its_effective_root=0 differs_from_chosen_root_rows=93530 differs_rows_with_nothing_applied=0 ignore_policy_reads_checked=191260 ignore_policy_read_failed=0 ignore_policy_chosen_root_changed=0 ignore_policy_content_differs_from_chosen_root=0 ignore_policy_content_is_not_the_tip=93530
```

**第一段 `all` 与旧产物比（空域）**：账上没有被抛弃的根（`abandoned` 恒空），H3 的 N_w 变 0，PC3 判 `not_applicable`，Q3-2、Q2-3 的算术行按 N_w=0 打。这几行是空域：0 是「没有可数的对象」，不是「打不中」，也不是「候选 2 的见证窄到放得下」。它们仍在 `replay.sh` 的 exact 比对里，复跑逐字节一致，但一致不代表问题还有域。2026-09-24 页上「Q3-2 的 N_w = 4」「PC3 坐实……」两段（「结果整行抄自产物」一节末两段）与那一节抄的 `h3_family_summary` 四行、`peak_live_rollback_events_across_the_family` 行不再承重。

| 项（G0 / S16 / 小环三点同值，S4 另列） | 2026-09-25 产物 | 本次 |
|---|---|---|
| `h3_family_summary` 的 histories（G0 / S4） | 1822 / 14252 | 1371 / 15956 |
| `h3_family_summary` 的 error_count（G0 / S4） | 855 / 10629 | 402 / 3053 |
| `h3_family_summary` 的 peak_events_live（G0 / S4） | 3 / 4 | 0 / 0 |
| `h3_family_summary` 的 peak_roots_live（G0 / S4） | 10 / 10 | 0 / 0 |
| `peak_live_rollback_events_across_the_family` 的 value | 4 | 0 |
| Q3-2 `表1+16*peak` 的 width_bytes / margin_within_512 / 判定 | 65 / -34 / past_512 | 1 / 30 / fits |
| Q3-2 `表1+12*peak` 的 width_bytes / margin_within_512 / 判定 | 49 / -18 / past_512 | 1 / 30 / fits |
| Q2-3 `表1+16*peak` 的 margin_within_512_with_plus8 / 判定 | -42 / past_512_together | 22 / fits |
| Q2-3 `表1+12*peak` 的 margin_within_512_with_plus8 / 判定 | -26 / past_512_together | 22 / fits |
| `pc3_overall_verdict` | fail（`pc3` 一行在） | not_applicable（`pc3` 一行不再打） |

`research/results/e158-root-choice-repair-2026-11-09-segment1-rerun.out` 第 57、75、84 行：

```text
E7RESULT name=h3_family_summary geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) overwrites_before_sigma=[1, 2, 3] sigma_length_limit=3 histories=1371 peak_events_live=0 peak_roots_live=0 steps_with_positive_events_live=0 leaf_count=1237 leaves_with_positive_events_live=0 peak_events_live_at_a_leaf=0 error_count=402
E7RESULT name=peak_live_rollback_events_across_the_family value=0
E7RESULT name=pc3_overall_verdict verdict=not_applicable reason="GEOMETRY_PRIMARY 全族里没有找到 events_live≥1 的节点"
```

五份产物里 `local_constant_check` 的系统配置字段表合计与根记录宽，本地值与 `crates/` 值同步从 481 变 489、从 457 变 466，来自 `crates/` 在 2026-09-25 之后的字段变化，与账本无关。五份产物的判决行：没有 `name=verdict` 行，没有 `verdict=fail`；带 `=false` 的只有 `section_seven_one_anchor` 与 `section_seven_three_row` 的 `checked_in_this_segment=false`（`verdict=deferred`，登记预期内，旧产物里同样）。

**④ q1 三行删除并标作废的依据**：如实账本之后回退是向前发布，不再产生被抛弃的根，H1 家族的 Φ1 故障域（被抛弃根的账）是空的。任务 10974（`q1-g0` 在新账本上跑的一趟，输出未入库）量到：`q1_h1_family_size` 的 nodes 由 2026-09-25 产物的 36 变 27，`rejected_rollback_attempts` 由 0 变 9；`q1_1a_trigger_summary` 的 pairs 由 3942 变 0（trigger_count 由 798 变 0）；`q1_1a_raise_floor_trigger_summary` 的 pairs 是 0；`pc1_a_and_b` 的 `verdict=not_found`。留着这三行重出，只会得到一份空域的 exact 比对，所以删除：`replay.sh` 里 `driver_e158_q1_g0`、`driver_e158_q1_s16`、`driver_e158_q1_s4` 三行与三个驱动函数删了（`grep -c driver_e158_q1 scripts/experiment-apparatus/replay.sh` 为 0）。session s3 至 s10 各节里第一次跑的 Q1-1 至 Q1-4、PC1-a/b、op1 三种变体与 S16、S4 两点 `N_trig=798` 的读数作废，不再承重。另算一个「目标之后那一段」的域（实现员报告的乙走法）没做。

**⑤ `.claude/gate.d/replay-comparison-units.tsv`**：现查没有 E158 的行（`grep -c 'E158\|e158'` 为 0），不需要改；这一次没有新写重跑登记。

**⑥ 同日再删 7 行（用户定「删 7 行并补回枚举」）**：`replay.sh` 复跑表里删 `driver_e158_q2_2a_g0`、`driver_e158_q2_1_pc2`、`driver_e158_q2_1_g0_session_s5`、`driver_e158_q2_1_hc1`、`driver_e158_q2_1_hc1_lower_bound`、`driver_e158_r2_all`、`driver_e158_r3_seg1_today` 七行与只有它们用的驱动函数（`grep -c` 这七个驱动名在 `replay.sh` 里为 0）；留 `driver_e158`、四行 q3-1、`driver_e158_r3_seg1_compare`、`driver_e158_r4_compare`。两个 compare 驱动只按路径读 `research/results/` 下的臂产物（r3 读的 `…-2026-09-27-r3-seg1-today.out` 是被删那一行的旧产物），不调被删的驱动；旧产物一个没删、没挪，留在 git 历史里。
- 删除依据（量的，来自把这七行在今天的 `crates/` 上现跑一遍的判定输出，输出未入库）：差异来自 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 乙合入，乙把「最新根读不出且系统配置见证过更新的版本」的挂载拒成 K3，这七行的「今天」基线写在乙之前。`r2-all`：`r2_q1_0` 的 87 次 `K0` 全变 `K3`，造出被抛弃根 56 → 0，PC1-a、PC1-c 由 `pass` 变 `not_constructible`，PC1-b 不再打。`r3-seg1` 今天：`verdict=fail` 由 2 变 11（PC-O 与八条 PC-N0），`q0_hidden_root_abandoned_cells` 由 32、8 变 0、0。q2 五行里，`q2-1-hc1` 的 `hit=true` 变成 `error="faulted mount_writable: NewerStateStillUnreadableAfterOneReread(PublishWitnessed…)"`（拒因整行在产物里，量的）；`q2-1-g0` session s5 与 hc1 下界的 `k_min=12` 变 `none_found`，按同一历史同一拒因推（推的，没打成员名）。`q2-1-pc2` 只差常量回显 481 → 489、457 → 466，结果行没变。`q2-2a-g0` 只差代价数：分配记录树节点写 14 → 10 次、229376 → 163840 字节、总写 33 → 29；是哪次改动造成的没追到。
- 页面现状由 `r4-compare` 承重（岔路单 `c554-fix-forks.md` 三行的读数在第 4 次跑），这七行回答的是乙合入之前的基线，已不回答现状的问题；留着只会把「乙拦住了」冻成 exact，`crates/` 每变一次就红。
- **更正上一段写进页里的 q1 依据**：q1 三行删除时用的「r2-all、r3 罩着真实路径的 PC1」不成立，r2-all 现跑造出 0 条被抛弃根，r3 的 `q0_hidden_root_abandoned_cells` 也是 0。真正罩着「被抛弃根的账读不出，从映射现算」这条路径的是写死历史的用例，8 条：`crates/singlefs-verification/tests/an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping.rs`（14 条用例）、`unreadable_abandoned_root_slot.rs`（3 条）、`abandoned_root_whose_account_is_unreadable_passes_…`、`raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs`、`reuse_after_raising_the_floor.rs`、`a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable.rs`，以及 checker 侧两个测试文件（`checker_narrow_invariants_and_abandoned_roots.rs`、`crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）；最近读数全绿，有变异行（`crates/mutations.tsv`）。这些用例的造法把系统配置见证藏掉，所以乙挡不住它们。④ 的其余内容（任务 10974 的读数）不变，仍站得住。
- **枚举缺口**：这条路径没有任何枚举型覆盖（多条历史、多个故障组合）；缺口形成的时点是 C554 乙合入（r2 的 Q1-0 曾有 56 个节点，现在 0），不是这次删除。随机历史那一半记在 C599（随机历史造不出被抛弃的根）；补回枚举已由第 5 次重跑补上（见页首「2026-11-12」「2026-11-11：第 5 次重跑」两节：四档已跑）。

**复跑**：`bash scripts/experiment-apparatus/replay.sh E158`，复跑表里 E158 现有 7 行（`driver_e158`、四行 q3-1、`r3_seg1_compare`、`r4_compare`）。

**这一节答不了的**：新判据只判带故障且读成了的冷读；省法省掉的故障集合（G0 132417 个、S4 1050960 个）没有 (b)(c) 读；只在这四个几何点，故障只点名根槽、journal 环完好；重放有没有跨实例边界这里分不出来；回退到以前被剪掉的根现在才第一次走到，产品拒绝的步列在 `e158_ledger_diagnostics_refused`（G0 402、S4 3053），其余读错了严格判据会红，没红。

````

**出处 `.claude/kb/experiments/158-择根与修复四岔路.md:288-622`（整段抄，未转述）**

````markdown
### 2026-09-27：第 4 次跑（重跑登记 `e158-r4-prereg.md`）

「2026-09-27：第 4 次跑」这一节是 E158（择根与修复四岔路） 的现状，回答岔路单 `c554-fix-forks.md`（C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 怎么修）的三行。「2026-09-27：第 3 次跑第一段」一节只当参考；「2026-09-26：第 2 次跑第一段」一节与它之后的各节是更早几次跑的记录，读数不带进第 4 次跑。Q0–Q9、H1d–H1g、H-随、H-随全、实七-甲、实七-乙、实八、L0–L6、PC-N-环、PC-N-盖、PC-O、PC-N0、PC-多拒、PC-多读、PC-22、PC-554、PC-随、F1–F19、S0–S9、V1–V5、N_真、N-配置、N-配置续、N-槽 是第 4 次跑登记（与它照用的第 3 次跑登记 `e158-r3-prereg.md`）自己的代号，登记位在那两份文件里。

**岔路表**（岔路单三行；「产物行」里的行号都指五段合并的 `research/results/e158-root-choice-repair-2026-09-27-r4-compare.out`）：

| # | 岔路 | 状态 | 够判依据 | 产物行 |
|---|---|---|---|---|
| 1 | 挂载时有一条根（或它引用的账）暂时读不出、而它可能比所选根新，可写挂载怎么办 | 用户已定乙，R = 1，判「有一条更新的根读不出」用系统配置记的位置（-配置 一族） | **丢写**：今天、甲-槽、乙-槽、乙-窄读-槽 在 H1d、H1e、H1f、H1g、H-随、实七-甲、实七-乙、实八 上都 > 0（今天 H1d 304 / 1680、H-随 43 / 43），按岔路单出局；-配置 四臂（甲、乙、乙-窄读、丙）在 H1g 上各 16 / 32；-配置续 四臂在 H1f、H1g 与第二段实七-甲、实七-乙、实八 上都是 0，在 H-随 上照登记字面 6 / 37，这 6 格全是 ③c 尾巴（不算 ③c 2 格，Q1 只算环内且不算 ③c 1 格）；H1d、H1e 按登记第六节开头对 -配置 各臂不算判据。**多拒**（L 族，G0（主几何点））：甲、乙、乙-窄读 的 -配置 与 -配置续 各 22（L3 16、L5 6，L5 那 6 格里 3 格是罩住第 22 条那一格的代价），丙 两臂各 16，-槽 三臂各 19；第二段五族每臂 0。**多读**（同成，L 族 G0（主几何点））：乙-配置 每格 +3 或 +4（629 格合计 2472），乙-配置续 每格 +7 或 +8（4988）。**第 22 条那一格**：今天没罩住 3 / 72；候选各臂没罩住 0（甲 三臂的 Q6 按 V1 ④ 作废）。**判据用的那一样今天有没有**：S0 在快照上逐条核过，系统配置槽里记着每次发布写下的 `journal_tail`，但今天的可写挂载只判「落后」、不拿它判「领先」，取号那一写把 tail 写成 0（登记修订第 3 条）；要加的是 N-配置 那一判与取号续写见证值 | 丢写 1546–1554、1567–1575；多拒 634、638；多读见「多读」一段的命令；Q6 1564、1585 |
| 2 | 乙-配置 的判据只认系统配置槽里见证的 tail，没见证的最新根怎么办 | 用户已定乙-配置续；乙-槽 与轮换后加屏障不走 | 两臂只在 H1g 上分开：乙-配置 16 / 32（全在 a = 2：连着两次取号都写 tail = 0，装置解出的 N-配置 为假，被测挂载 K0、回滚），乙-配置续 0 / 32（a = 2 上 N-配置 为真、K3）；H1f、第二段四族（seg2b）两臂逐族相同，H-随 同是 6 / 37；多拒同是 22；同成多读乙-配置续每格比乙-配置多 4 次（L 族 4988 对 2472，同 629 格）；-槽 那一支（乙-槽）丢写全落在读回全零那一种造法上（H1d 288 / 2560） | 1575、1596；1569、1590；634、754 |
| 3 | 乙的读缓存要不要收窄（乙 / 乙-窄读） | 记欠（第四批 kb 规格 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 乙 Q4 那一笔） | 两臂逐格判定**不**全相同：乙-窄读 在读回全零、b = records 的 T(1) 与 O(1 只) 格上拒（PC-N-盖 那三句 [臂] 打不中，乙-窄读 两条 -配置 臂的 Q1–Q3 按 V1 ④ 作废，主 agent 认定这是窄读与乙的差别、照量），拦下栏 H1d 多出 `O=4,T=4`、H1f-cuts 多出 `O=8,T=8`；第二段、第三段的丢写与 L 族多拒两臂逐族相同；同成多读 L 族两臂相同（2472 / 4988），H1d 上乙-配置 +50136（32 格）、乙-窄读-配置 +942（24 格）；**读缓存峰值没量**：第 4 次跑的登记里没有这个量，不为它加臂 | 617、856（H1d 拦下）；624、863（H1f-cuts 拦下）；1775、1782（作废） |

「出局」「选哪个」由主 agent 按岔路单判，第 4 次跑只报数。

**装置与口径**：

- 入库装置 `crates/singlefs-implementation-experiments/src/bin/e158_root_choice_repair/main.rs` 的 `r4-*` 模式（`r4-seg1` 至 `r4-seg5`、`r4-bases`、`r4-compare`），确定性（同一个二进制跑两遍逐字节一致只说明没有隐藏状态，第四段执行员拿今天那一臂重跑一次 `cmp` 过）。装置自己记账、自己解码（根槽、系统配置槽、记录经 `singlefs_checker`），与 `crates/` 交回的逐项对拍（S4，每族 `s4_mismatches=0`）。
- 底座：开跑条件 C0 那份快照（对 `crates` 做一次 `cp -a`，底座认定的四个 `singlefs-core` 文件 sha256 逐个对上；登记修订第 1 条）。**今天那一臂 = 快照原样**；十八条臂各自一份副本，从那一次 `cp -a` 派生，改法是 `scripts/mutation/tables/e158_arms.tsv` 里以 `r4 ` 打头的 18 行。
- 十八条臂：今天；甲-配置、甲-配置续、甲-槽；乙-配置、乙-配置续、乙-槽；乙-窄读-配置、乙-窄读-配置续、乙-窄读-槽；丙-配置、丙-配置续（主族十二条）；乙 三臂与乙-窄读 三臂各一条 R = 3（`-three-rounds`，只跑第五段）。乙-窄读 是 5.5 主 agent 认定第 1 项另立的对照臂：只重读根环槽、系统配置槽与点名单元，把 journal 环范围内读回全零的落点当空槽收进缓存（登记修订第 5 条）。
- 五段：第一段 H1d、H1e 与 PC-N-环、PC-N-盖、PC-O、PC-丢写、PC-N0、PC-多读；第二段实七-甲、实七-乙、实八、H-随（30 段）、H-随全（68 段，只进多拒、多读）与 PC-随；第三段 H1f、H1g；第四段 L0–L6 与 PC-多拒、PC-22、PC-554；第五段几何敏感性（G_S4、G_S16、G_默认、G_小环、k = 2、R = 3）。前四段除第二段外都在 G0（主几何点）上，第二段的历史由 harness 的生成器定几何（4 GiB 盘）。
- 装置 bin 的三版：第一、三、四、五段用 sha256 `37a8a312cf9c2d3bf6bf1a835530e78fa6d9f3a4b36607d3ecb8fd5e5cf1445b`（装置执行员报告 `e158-r4-device-runner-report.md` 第一节），产物第一行（`E7INPUT name=crates_snapshot` 那一行）的指纹是 `sha256=bb5ca9bef0b09eb957906317adf7d2b47567d901ef341e85c14b6fe23a37cf4b`；第二段装置另写之后 bin 是 `12b82f491453e4f598f8262632dc949967508a8a1cae745a1b4db078499fb148`，把 C0 清单里装置那一行换成它之后整份清单的 sha256 是 `ad9f544263a5a61bf0bed305ea6e98701042e63ff2ad5e96f2b66552f360f9dc`（登记修订第 29 条），第二段两次产物与五段合并的 compare 第一行都写这个；seg2b 的 bin 又修了一处（崩溃分支挂载之后的读回改用分支自己的账），源码 sha256 `e71224fc02fde638e92f2548c19c71c2f7032b65ec7056a1bc014bebbac8dba8`，seg2b 产物第一行仍是 `ad9f5442…`（**没把修过的那一版算进指纹**，seg2b 执行员报告 `e158-r4-seg2b-runner-report.md` 第二节）。五段合并的 compare 用 seg2b 今天那一臂的二进制（release 二进制 sha256 `ac25b3fe0c58c26c52100d8e5b1c6851fafc8e3b47a53914df7ad204235dd5d2`）跑。
- 各段报告：`e158-r4-seg1-runner-report.md`、`e158-r4-seg2-runner-report.md`、`e158-r4-seg2b-runner-report.md`、`e158-r4-seg3-runner-report.md`、`e158-r4-seg4-runner-report.md`、`e158-r4-seg5-runner-report.md`；装置 `e158-r4-device-runner-report.md`、`e158-r4-seg2-device-runner-report.md`；撞键调查 `e158-r4-seg2-ledger-investigation-report.md`。
- 跨段的量（多拒、多读、阳性对照的跨臂句子、V1 作废范围）**以五段合并的 `research/results/e158-root-choice-repair-2026-09-27-r4-compare.out` 为准**（第四段之后主 agent 认定第 4 项）；它的 `r4_loss`、`r4_q4`、`r4_q4_after_divergence`、`r4_q5`、`r4_q6` 各行与五份分段 compare（第二段取 seg2b 的 `…-compare-r2.out`）排序后逐行相同，只有 `r4_void` 与跨臂句子多了跨段的那一部分。命令与原样输出：

```text
$ R=research/results/e158-root-choice-repair-2026-09-27-r4; for n in r4_loss r4_q4 r4_q4_after_divergence r4_q5 r4_q6; do m=$(grep "name=$n " $R-compare.out | sort | sha256sum | cut -c1-16); s=$(cat $R-seg1-compare.out $R-seg2b-compare-r2.out $R-seg3-compare.out $R-seg4-compare.out $R-seg5-compare.out | grep "name=$n " | sort | sha256sum | cut -c1-16); echo "$n merged=$(grep -c "name=$n " $R-compare.out) segments=$(cat $R-seg1-compare.out $R-seg2b-compare-r2.out $R-seg3-compare.out $R-seg4-compare.out $R-seg5-compare.out | grep -c "name=$n ") same=$([ $m = $s ] && echo yes || echo no)"; done
r4_loss merged=228 segments=228 same=yes
r4_q4 merged=385 segments=385 same=yes
r4_q4_after_divergence merged=40 segments=40 same=yes
r4_q5 merged=571 segments=571 same=yes
r4_q6 merged=36 segments=36 same=yes
```

**产物**（`research/results/` 下 86 份，文件名 `e158-root-choice-repair-2026-09-27-r4-<段>-<臂>.out`，每段另有 `…-<段>-compare.out`；第二段两次都留：`seg2` 13 份、`seg2b` 14 份，seg2b 的 compare 两份）：

```text
$ cd research/results; for s in seg1 seg2 seg2b seg3 seg4 seg5; do n=$(ls e158-root-choice-repair-2026-09-27-r4-$s-*.out | wc -l); d=$(for f in e158-root-choice-repair-2026-09-27-r4-$s-*.out; do tail -1 $f; done | grep -c '^E7RESULT name=done'); st=$(cat e158-root-choice-repair-2026-09-27-r4-$s-*.out | grep -c 'name=stop'); fp=$(for f in e158-root-choice-repair-2026-09-27-r4-$s-*.out; do head -1 $f | grep -o 'sha256=[0-9a-f]\{8\}\|sha256=unset'; done | sort | uniq -c | tr '\n' ' '); echo "$s files=$n done_last=$d stop_lines=$st first_line=$fp"; done
seg1 files=13 done_last=13 stop_lines=0 first_line=     13 sha256=bb5ca9be 
seg2 files=13 done_last=13 stop_lines=0 first_line=     13 sha256=ad9f5442 
seg2b files=14 done_last=14 stop_lines=0 first_line=     13 sha256=ad9f5442       1 sha256=unset 
seg3 files=13 done_last=13 stop_lines=0 first_line=     13 sha256=bb5ca9be 
seg4 files=13 done_last=13 stop_lines=0 first_line=     13 sha256=bb5ca9be 
seg5 files=19 done_last=19 stop_lines=0 first_line=     19 sha256=bb5ca9be 
```

- 第一段：`e158-root-choice-repair-2026-09-27-r4-seg1-bing-cfg-carry.out`、`e158-root-choice-repair-2026-09-27-r4-seg1-bing-cfg.out`、`e158-root-choice-repair-2026-09-27-r4-seg1-compare.out`、`e158-root-choice-repair-2026-09-27-r4-seg1-jia-cfg-carry.out`、`e158-root-choice-repair-2026-09-27-r4-seg1-jia-cfg.out`、`e158-root-choice-repair-2026-09-27-r4-seg1-jia-slot.out`、`e158-root-choice-repair-2026-09-27-r4-seg1-today.out`、`e158-root-choice-repair-2026-09-27-r4-seg1-yi-cfg-carry.out`、`e158-root-choice-repair-2026-09-27-r4-seg1-yi-cfg.out`、`e158-root-choice-repair-2026-09-27-r4-seg1-yi-narrow-cfg-carry.out`、`e158-root-choice-repair-2026-09-27-r4-seg1-yi-narrow-cfg.out`、`e158-root-choice-repair-2026-09-27-r4-seg1-yi-narrow-slot.out`、`e158-root-choice-repair-2026-09-27-r4-seg1-yi-slot.out`。
- 第二段第一次（装置撞键，只当对照）：`e158-root-choice-repair-2026-09-27-r4-seg2-bing-cfg-carry.out`、`e158-root-choice-repair-2026-09-27-r4-seg2-bing-cfg.out`、`e158-root-choice-repair-2026-09-27-r4-seg2-compare.out`、`e158-root-choice-repair-2026-09-27-r4-seg2-jia-cfg-carry.out`、`e158-root-choice-repair-2026-09-27-r4-seg2-jia-cfg.out`、`e158-root-choice-repair-2026-09-27-r4-seg2-jia-slot.out`、`e158-root-choice-repair-2026-09-27-r4-seg2-today.out`、`e158-root-choice-repair-2026-09-27-r4-seg2-yi-cfg-carry.out`、`e158-root-choice-repair-2026-09-27-r4-seg2-yi-cfg.out`、`e158-root-choice-repair-2026-09-27-r4-seg2-yi-narrow-cfg-carry.out`、`e158-root-choice-repair-2026-09-27-r4-seg2-yi-narrow-cfg.out`、`e158-root-choice-repair-2026-09-27-r4-seg2-yi-narrow-slot.out`、`e158-root-choice-repair-2026-09-27-r4-seg2-yi-slot.out`。
- 第二段重跑（以它为准）：`e158-root-choice-repair-2026-09-27-r4-seg2b-bing-cfg-carry.out`、`e158-root-choice-repair-2026-09-27-r4-seg2b-bing-cfg.out`、`e158-root-choice-repair-2026-09-27-r4-seg2b-compare.out`、`e158-root-choice-repair-2026-09-27-r4-seg2b-compare-r2.out`、`e158-root-choice-repair-2026-09-27-r4-seg2b-jia-cfg-carry.out`、`e158-root-choice-repair-2026-09-27-r4-seg2b-jia-cfg.out`、`e158-root-choice-repair-2026-09-27-r4-seg2b-jia-slot.out`、`e158-root-choice-repair-2026-09-27-r4-seg2b-today.out`、`e158-root-choice-repair-2026-09-27-r4-seg2b-yi-cfg-carry.out`、`e158-root-choice-repair-2026-09-27-r4-seg2b-yi-cfg.out`、`e158-root-choice-repair-2026-09-27-r4-seg2b-yi-narrow-cfg-carry.out`、`e158-root-choice-repair-2026-09-27-r4-seg2b-yi-narrow-cfg.out`、`e158-root-choice-repair-2026-09-27-r4-seg2b-yi-narrow-slot.out`、`e158-root-choice-repair-2026-09-27-r4-seg2b-yi-slot.out`。
- 第三段：`e158-root-choice-repair-2026-09-27-r4-seg3-bing-cfg-carry.out`、`e158-root-choice-repair-2026-09-27-r4-seg3-bing-cfg.out`、`e158-root-choice-repair-2026-09-27-r4-seg3-compare.out`、`e158-root-choice-repair-2026-09-27-r4-seg3-jia-cfg-carry.out`、`e158-root-choice-repair-2026-09-27-r4-seg3-jia-cfg.out`、`e158-root-choice-repair-2026-09-27-r4-seg3-jia-slot.out`、`e158-root-choice-repair-2026-09-27-r4-seg3-today.out`、`e158-root-choice-repair-2026-09-27-r4-seg3-yi-cfg-carry.out`、`e158-root-choice-repair-2026-09-27-r4-seg3-yi-cfg.out`、`e158-root-choice-repair-2026-09-27-r4-seg3-yi-narrow-cfg-carry.out`、`e158-root-choice-repair-2026-09-27-r4-seg3-yi-narrow-cfg.out`、`e158-root-choice-repair-2026-09-27-r4-seg3-yi-narrow-slot.out`、`e158-root-choice-repair-2026-09-27-r4-seg3-yi-slot.out`。
- 第四段：`e158-root-choice-repair-2026-09-27-r4-seg4-bing-cfg-carry.out`、`e158-root-choice-repair-2026-09-27-r4-seg4-bing-cfg.out`、`e158-root-choice-repair-2026-09-27-r4-seg4-compare.out`、`e158-root-choice-repair-2026-09-27-r4-seg4-jia-cfg-carry.out`、`e158-root-choice-repair-2026-09-27-r4-seg4-jia-cfg.out`、`e158-root-choice-repair-2026-09-27-r4-seg4-jia-slot.out`、`e158-root-choice-repair-2026-09-27-r4-seg4-today.out`、`e158-root-choice-repair-2026-09-27-r4-seg4-yi-cfg-carry.out`、`e158-root-choice-repair-2026-09-27-r4-seg4-yi-cfg.out`、`e158-root-choice-repair-2026-09-27-r4-seg4-yi-narrow-cfg-carry.out`、`e158-root-choice-repair-2026-09-27-r4-seg4-yi-narrow-cfg.out`、`e158-root-choice-repair-2026-09-27-r4-seg4-yi-narrow-slot.out`、`e158-root-choice-repair-2026-09-27-r4-seg4-yi-slot.out`。
- 第五段：`e158-root-choice-repair-2026-09-27-r4-seg5-bing-cfg-carry.out`、`e158-root-choice-repair-2026-09-27-r4-seg5-bing-cfg.out`、`e158-root-choice-repair-2026-09-27-r4-seg5-compare.out`、`e158-root-choice-repair-2026-09-27-r4-seg5-jia-cfg-carry.out`、`e158-root-choice-repair-2026-09-27-r4-seg5-jia-cfg.out`、`e158-root-choice-repair-2026-09-27-r4-seg5-jia-slot.out`、`e158-root-choice-repair-2026-09-27-r4-seg5-today.out`、`e158-root-choice-repair-2026-09-27-r4-seg5-yi-cfg-carry.out`、`e158-root-choice-repair-2026-09-27-r4-seg5-yi-cfg-carry-three-rounds.out`、`e158-root-choice-repair-2026-09-27-r4-seg5-yi-cfg.out`、`e158-root-choice-repair-2026-09-27-r4-seg5-yi-cfg-three-rounds.out`、`e158-root-choice-repair-2026-09-27-r4-seg5-yi-narrow-cfg-carry.out`、`e158-root-choice-repair-2026-09-27-r4-seg5-yi-narrow-cfg-carry-three-rounds.out`、`e158-root-choice-repair-2026-09-27-r4-seg5-yi-narrow-cfg.out`、`e158-root-choice-repair-2026-09-27-r4-seg5-yi-narrow-cfg-three-rounds.out`、`e158-root-choice-repair-2026-09-27-r4-seg5-yi-narrow-slot.out`、`e158-root-choice-repair-2026-09-27-r4-seg5-yi-narrow-slot-three-rounds.out`、`e158-root-choice-repair-2026-09-27-r4-seg5-yi-slot.out`、`e158-root-choice-repair-2026-09-27-r4-seg5-yi-slot-three-rounds.out`。
- 五段合并：`e158-root-choice-repair-2026-09-27-r4-compare.out`（1832 行，末行 `E7RESULT name=done emitted=1831`；`replay.sh` 登记的是它）。

**丢写（翻面观测第一样）**：丢写格 = Q1 覆盖格 ∪ Q2 回滚格 ∪ Q3 读回丢写格（含 ③c 尾巴）按格去重，照登记字面算（逐格字段 `lost_write_cell` 不含 Q2，写页以族摘要与 compare 的 `r4_loss` 为准，第五段之后主 agent 认定第 2 项）。每臂每族「丢写格 / 格数」，G0（主几何点）与第二段的 harness 几何：

```text
$ grep 'name=r4_loss ' research/results/e158-root-choice-repair-2026-09-27-r4-compare.out | grep -E 'geometry=(GEOMETRY_PRIMARY|harness-4gib)' | grep -Ev 'family=(h1d-uncut|h1e-k2|h1d-uncut-k2|h1e-r3|h1d-uncut-r3)' | awk '{for(i=1;i<=NF;i++){split($i,kv,"=");v[kv[1]]=substr($i,length(kv[1])+2)}; t[v["arm"]" "v["family"]]=v["lost_write_cells"]"/"v["cells"]} END{for(k in t) print k, t[k]}' | sort | awk '{a[$1]=a[$1]" "$2"="$3} END{for(k in a) print k a[k]}' | sort
bing-cfg-carry eighth-crash=0/2 h1d=0/912 h1e=0/64 h1f-cuts=0/2080 h1f-tail=0/128 h1g=0/32 h-random=6/37 seventh-crash=0/6 seventh-fault=0/10
bing-cfg eighth-crash=0/2 h1d=0/912 h1e=0/64 h1f-cuts=0/2080 h1f-tail=0/128 h1g=16/32 h-random=6/37 seventh-crash=0/6 seventh-fault=0/10
jia-cfg-carry eighth-crash=0/2 h1d=0/48 h1e=0/64 h1f-cuts=0/96 h1f-tail=0/128 h1g=0/32 h-random=6/37 seventh-crash=0/6 seventh-fault=0/10
jia-cfg eighth-crash=0/2 h1d=0/48 h1e=0/64 h1f-cuts=0/96 h1f-tail=0/128 h1g=16/32 h-random=6/37 seventh-crash=0/6 seventh-fault=0/10
jia-slot eighth-crash=3/6 h1d=152/864 h1e=32/64 h1f-cuts=48/2080 h1f-tail=64/128 h1g=16/32 h-random=43/43 seventh-crash=1/5 seventh-fault=10/10
today eighth-crash=3/6 h1d=304/1680 h1e=64/64 h1f-cuts=96/4064 h1f-tail=128/128 h1g=32/32 h-random=43/43 seventh-crash=1/5 seventh-fault=10/10
yi-cfg-carry eighth-crash=0/2 h1d=0/1792 h1e=0/128 h1f-cuts=0/4096 h1f-tail=0/256 h1g=0/32 h-random=6/37 seventh-crash=0/6 seventh-fault=0/10
yi-cfg eighth-crash=0/2 h1d=0/1792 h1e=0/128 h1f-cuts=0/4096 h1f-tail=0/256 h1g=16/32 h-random=6/37 seventh-crash=0/6 seventh-fault=0/10
yi-narrow-cfg-carry eighth-crash=0/2 h1d=0/1360 h1e=0/128 h1f-cuts=0/3104 h1f-tail=0/256 h1g=0/32 h-random=6/37 seventh-crash=0/6 seventh-fault=0/10
yi-narrow-cfg eighth-crash=0/2 h1d=0/1360 h1e=0/128 h1f-cuts=0/3104 h1f-tail=0/256 h1g=16/32 h-random=6/37 seventh-crash=0/6 seventh-fault=0/10
yi-narrow-slot eighth-crash=3/6 h1d=288/2560 h1e=64/128 h1f-cuts=64/6080 h1f-tail=128/256 h1g=16/32 h-random=43/43 seventh-crash=1/5 seventh-fault=10/10
yi-slot eighth-crash=3/6 h1d=288/2560 h1e=64/128 h1f-cuts=64/6080 h1f-tail=128/256 h1g=16/32 h-random=43/43 seventh-crash=1/5 seventh-fault=10/10
```

- 族名对应：`h-random` = H-随，`seventh-fault` = 实七-甲，`seventh-crash` = 实七-乙，`eighth-crash` = 实八；第二段的数取自 seg2b（五段合并的 compare 喂的就是 seg2b）。格数随臂不同：H1d 的断点格按被测挂载的写数立，乙 各臂多一档时长；-配置 各臂在实七-甲 上 3 个注入点 `fault_not_reached`、在实八 上 4 个崩溃状态 `not_constructible=segment_missing`（它们在崩溃恢复那一步就拒了，写序列与今天那一臂不同），所以实八只有主路径 2 格，今天那一臂是 6 格（登记修订第 19 条的记法，不立格）。
- 哪些族对哪些臂算判据（登记第六节开头）：H1d、H1e 对甲-配置、甲-配置续、乙-配置、乙-配置续、丙-配置、丙-配置续 **不算**（N-配置 在那两族的每一格按构造判真，0 由臂的定义推得出；乙-窄读 两条 -配置 臂同一个理由，而且它们的 Q1–Q3 已按 V1 ④ 作废）；这几条臂的丢写判据落在 H1f、H1g、实七-甲、实七-乙、实八、H-随 上。今天与 -槽 各臂全族都算。
- 乙-配置、乙-配置续 在 H1g、H-随、实七-乙 上的原样行（五段合并的 compare 第 1575、1596、1569、1590、1571、1592 行）：

```text
E7RESULT name=r4_loss arm=yi-cfg family=h1g geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=32 lost_write_cells=16 q1_overwrite_cells=16 q2_rollback_cells=16 q3_lost_cells=0 q3c_lost_cells=8 q3_last_confirmed_cells=32
E7RESULT name=r4_loss arm=yi-cfg-carry family=h1g geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=32 lost_write_cells=0 q1_overwrite_cells=0 q2_rollback_cells=0 q3_lost_cells=0 q3c_lost_cells=0 q3_last_confirmed_cells=32
E7RESULT name=r4_loss arm=yi-cfg family=h-random geometry=harness-4gib cells=37 lost_write_cells=6 q1_overwrite_cells=2 q2_rollback_cells=0 q3_lost_cells=0 q3c_lost_cells=6 q3_last_confirmed_cells=37
E7RESULT name=r4_loss arm=yi-cfg-carry family=h-random geometry=harness-4gib cells=37 lost_write_cells=6 q1_overwrite_cells=2 q2_rollback_cells=0 q3_lost_cells=0 q3c_lost_cells=6 q3_last_confirmed_cells=37
E7RESULT name=r4_loss arm=yi-cfg family=seventh-crash geometry=harness-4gib cells=6 lost_write_cells=0 q1_overwrite_cells=0 q2_rollback_cells=0 q3_lost_cells=0 q3c_lost_cells=0 q3_last_confirmed_cells=6
E7RESULT name=r4_loss arm=yi-cfg-carry family=seventh-crash geometry=harness-4gib cells=6 lost_write_cells=0 q1_overwrite_cells=0 q2_rollback_cells=0 q3_lost_cells=0 q3c_lost_cells=0 q3_last_confirmed_cells=6
```

- H1g 按 a 分开：-配置 四臂在 a = 1 上 16 格都拒（K3，不丢写），在 a = 2 上 16 格都 K0、回滚、丢写（装置解出的 N-配置 为假：两次取号都写 tail = 0，系统配置两槽的见证都被盖成 0）；-配置续 四臂 a = 1、a = 2 都 K3（取号续写了见证）。数自第三段臂产物的 `name=r4_cell family=h1g` 行，按臂数 `a`、`a_class`、`device_n_cfg`、`q2_rollback`、`lost_write_cell`（第三段报告 `e158-r4-seg3-runner-report.md` 第四节贴了命令与原样输出，例 `…-r4-seg3-yi-cfg.out` 那一臂 `16 a=1 a_class=K3 device_n_cfg=true …`、`16 a=2 a_class=K0 device_n_cfg=false q2_rollback=true lost_write_cell=true`）。

**H-随 那几格：照登记字面算、并列两栏**（第二段之后主 agent 认定第 3 项）。登记字面的丢写格含 ③c 尾巴（关闭 C → 可写挂载 → 冷启动读回 → 落到被藏那条根之前的旧根时的读回）；另两栏出自各臂产物的 `r4_random_summary`（登记修订第 23 条，只报数、不改判据）：`lost_write_cells_without_q3c`（不算 ③c）与 `lost_write_cells_with_q1_in_ring_only_and_without_q3c`（Q1 只算「还在根环里」那一栏且不算 ③c）：

```text
$ cd research/results; for f in e158-root-choice-repair-2026-09-27-r4-seg2b-*.out; do a=${f#e158-root-choice-repair-2026-09-27-r4-seg2b-}; a=${a%.out}; case $a in compare*) continue;; esac; grep -n 'name=r4_random_summary family=h-random ' $f | sed -E "s/^([0-9]+):.* cells=([0-9]+) lost_write_cells_q1_q2_q3=([0-9]+) .*lost_write_cells_without_q3c=([0-9]+) lost_write_cells_with_q1_in_ring_only_and_without_q3c=([0-9]+) .*q3c_lost_cells=([0-9]+) .*/$a line=\1 cells=\2 literal=\3 without_q3c=\4 in_ring_only_without_q3c=\5 q3c=\6/"; done
bing-cfg-carry line=1197 cells=37 literal=6 without_q3c=2 in_ring_only_without_q3c=1 q3c=6
bing-cfg line=1197 cells=37 literal=6 without_q3c=2 in_ring_only_without_q3c=1 q3c=6
jia-cfg-carry line=1197 cells=37 literal=6 without_q3c=2 in_ring_only_without_q3c=1 q3c=6
jia-cfg line=1197 cells=37 literal=6 without_q3c=2 in_ring_only_without_q3c=1 q3c=6
jia-slot line=1209 cells=43 literal=43 without_q3c=39 in_ring_only_without_q3c=39 q3c=40
today line=1241 cells=43 literal=43 without_q3c=39 in_ring_only_without_q3c=39 q3c=40
yi-cfg-carry line=1197 cells=37 literal=6 without_q3c=2 in_ring_only_without_q3c=1 q3c=6
yi-cfg line=1197 cells=37 literal=6 without_q3c=2 in_ring_only_without_q3c=1 q3c=6
yi-narrow-cfg-carry line=1197 cells=37 literal=6 without_q3c=2 in_ring_only_without_q3c=1 q3c=6
yi-narrow-cfg line=1197 cells=37 literal=6 without_q3c=2 in_ring_only_without_q3c=1 q3c=6
yi-narrow-slot line=1209 cells=43 literal=43 without_q3c=39 in_ring_only_without_q3c=39 q3c=40
yi-slot line=1209 cells=43 literal=43 without_q3c=39 in_ring_only_without_q3c=39 q3c=40
```

| 臂 | H-随 丢写格（登记字面） | 不算 ③c | Q1 只算环内且不算 ③c |
|---|---|---|---|
| -配置、-配置续 八臂（甲、乙、乙-窄读、丙） | 6 / 37 | 2 / 37 | 1 / 37 |
| 今天、甲-槽、乙-槽、乙-窄读-槽 | 43 / 43 | 39 / 43 | 39 / 43 |

-配置 各臂那 6 格的 ③c 尾巴读回 5 格 `failed:NoValidRoot`、1 格 `failed:UnitUnreadable`，Q2 回滚与 Q3 读回丢写都是 0；不算 ③c 仍丢写的 2 格（`…-r4-seg2b-yi-cfg.out` 里种子基 + 23 的 `main:crash_recovery@3` 与种子基 + 79 的 `main:crash_recovery@6` 两格）都是 Q1 覆盖：前一格那一版在历史末尾已不在根环里、第一次被改在第 30 步，那时它之后已有 11 笔确认；后一格那一版还在根环里、第一次被改在第 25 步，之后已有 5 笔确认（`q1_first_change=order25:later_confirmations5:in_ringtrue`）。那一版早已被后来的确认取代时，Q1 与 ③c 也可能为正（登记修订第 23 条，推的，没量过）；这几格算不算丢写交主 agent，第 4 次跑照字面报。

**第二段两次：seg2 与 seg2b**。第一次（seg2）-配置 八臂在实七-乙 上记了 3 / 6 丢写（`crash0`–`crash2` 三个崩溃状态格，读回 `failed:content_differs_from_the_ledger`），今天那一臂实八 `crash1:crash@5` 的读回判定也受影响。调查报告 `e158-r4-seg2-ledger-investigation-report.md` 第一节查明是装置记账撞键：崩溃分支挂载之后的读回拿主路径的账按根的键 `(实例, txg)` 查，两条时间线在同一个键上各有一版；调查员改用分支自己的账之后 -配置 实七-乙 3 / 6 → 0 / 6，故意记错一次 4 格回红（第二节）。按装置 bug 修（分支的读回用分支自己的账），原判据一个字不改，12 条臂整段重跑成 seg2b。**以 seg2b 为准**。两次的差别只在实七-乙、实八两族，H-随、H-随全、实七-甲三族逐字节相同：

```text
$ cd research/results; P=e158-root-choice-repair-2026-09-27-r4; for a in today jia-cfg jia-cfg-carry jia-slot yi-cfg yi-cfg-carry yi-slot yi-narrow-cfg yi-narrow-cfg-carry yi-narrow-slot bing-cfg bing-cfg-carry; do r="$a"; for f in h-random h-random-all seventh-fault seventh-crash eighth-crash; do if cmp -s <(grep "family=$f " $P-seg2-$a.out) <(grep "family=$f " $P-seg2b-$a.out); then r="$r $f:same"; else r="$r $f:differs"; fi; done; echo "$r"; done
today h-random:same h-random-all:same seventh-fault:same seventh-crash:same eighth-crash:differs
jia-cfg h-random:same h-random-all:same seventh-fault:same seventh-crash:differs eighth-crash:same
jia-cfg-carry h-random:same h-random-all:same seventh-fault:same seventh-crash:differs eighth-crash:same
jia-slot h-random:same h-random-all:same seventh-fault:same seventh-crash:same eighth-crash:differs
yi-cfg h-random:same h-random-all:same seventh-fault:same seventh-crash:differs eighth-crash:same
yi-cfg-carry h-random:same h-random-all:same seventh-fault:same seventh-crash:differs eighth-crash:same
yi-slot h-random:same h-random-all:same seventh-fault:same seventh-crash:same eighth-crash:differs
yi-narrow-cfg h-random:same h-random-all:same seventh-fault:same seventh-crash:differs eighth-crash:same
yi-narrow-cfg-carry h-random:same h-random-all:same seventh-fault:same seventh-crash:differs eighth-crash:same
yi-narrow-slot h-random:same h-random-all:same seventh-fault:same seventh-crash:same eighth-crash:differs
bing-cfg h-random:same h-random-all:same seventh-fault:same seventh-crash:differs eighth-crash:same
bing-cfg-carry h-random:same h-random-all:same seventh-fault:same seventh-crash:differs eighth-crash:same
```

今天与 -槽 三臂实八那几行变了、丢写格数没变（3 / 6）。seg2b 的 compare 有两份：`e158-root-choice-repair-2026-09-27-r4-seg2b-compare.out` 跑的时候漏设了快照指纹的环境变量，首行是 `sha256=unset`；另存的 `e158-root-choice-repair-2026-09-27-r4-seg2b-compare-r2.out` 首行是 `ad9f5442…`，两份 `diff` 只差第 1 行，**以 `…-compare-r2.out` 为准**，两份都留着。

**多拒（Q4，翻面观测第二样）**：合法状态（N_真 为假）上臂拒而今天同一格不拒。L 族在 G0（主几何点）上 663 格全是 N_真 为假；乙-配置 与 乙-配置续 的 L 族各行（compare 第 627–641、747–761 行里 G0（主几何点）那几行；L3、L5 两族各另有两个几何点的行）只有 L3、L5 不为 0：

```text
E7RESULT name=r4_q4 arm=yi-cfg family=l3@GEOMETRY_PRIMARY(S=8,ring=3MiB) compared_tested_mounts=24 extra_refusals=16 intercepted=""
E7RESULT name=r4_q4 arm=yi-cfg family=l5@GEOMETRY_PRIMARY(S=8,ring=3MiB) compared_tested_mounts=72 extra_refusals=6 intercepted=""
E7RESULT name=r4_q4 arm=yi-cfg-carry family=l3@GEOMETRY_PRIMARY(S=8,ring=3MiB) compared_tested_mounts=24 extra_refusals=16 intercepted=""
E7RESULT name=r4_q4 arm=yi-cfg-carry family=l5@GEOMETRY_PRIMARY(S=8,ring=3MiB) compared_tested_mounts=72 extra_refusals=6 intercepted=""
```

| 臂 | L 族 G0（主几何点）多拒合计 | 其中 L5（第 22 条那一格） | L5 里罩住第 22 条的代价 | 第二段五族（含 H-随全 68 段） |
|---|---|---|---|---|
| 甲-配置、甲-配置续、乙-配置、乙-配置续、乙-窄读-配置、乙-窄读-配置续 | 22（L3 16） | 6 | 3 | 0 |
| 甲-槽、乙-槽、乙-窄读-槽 | 19（L2 8、L6 4、PC-554 那一格 1） | 6 | 3 | 0 |
| 丙-配置、丙-配置续 | 16（L3 16） | 0 | 0 | 0 |

- l5 那 6 格：甲、乙、乙-窄读 各臂在每份起始镜像的 m = 2、O(2 起) 上拒（`R3NewestInstanceTableUnreadableForTheShadowLedger`），一半在 b = records（今天 `Ok`、隔离 0 = 孪生镜像的 0，今天那一格罩住了），一半在 b = unit_first_copy（今天 `Ok`、隔离 0 < 孪生 28，正是今天没罩住的那 3 格）。后 3 格是「为罩第 22 条多拒」，照第四段之后主 agent 认定第 2 项在「选哪个」里单列、不并进多拒总数；照 Q4 字面它们仍算多拒，合计那一栏含它们。丙 在这 6 格上 `Ok`、隔离 = 孪生（用第一次读出的那张表）。出处：第四段报告 `e158-r4-seg4-runner-report.md` 第五节，产物 `…-r4-seg4-<臂>.out` 的 `name=r4_l_cell family=l5` 行。
- 第二段：`r4_q4` 55 行没有一行 `extra_refusals` 大于 0，走岔之后的 `r4_q4_after_divergence` 40 行没有一行 `refusals_with_truth_newer_false` 大于 0（`grep -c 'name=r4_q4_after_divergence .*refusals_with_truth_newer_false=[1-9]' research/results/e158-root-choice-repair-2026-09-27-r4-compare.out` 输出 `0`）。H-随全 上 -配置 各臂拦下 18 次（N_真 为真时拒），例（compare 第 611、612 行）：

```text
E7RESULT name=r4_q4 arm=yi-cfg family=h-random-all@harness-4gib compared_tested_mounts=293 extra_refusals=0 intercepted="W=18"
E7RESULT name=r4_q4 arm=yi-cfg family=h-random@harness-4gib compared_tested_mounts=64 extra_refusals=0 intercepted="W=30"
```

- 同一种 N 里甲与乙（含窄读）的多拒逐族相等，多拒这一样分不出甲、乙。

**多读（Q5-同成，翻面观测第三样）**：臂与今天那一臂在同一格上都交回 `Ok`、起始镜像逐字节相同的被测挂载，设备一层读调用次数臂减今天。按族归三组汇总（L 族与 PC-554 = 合法状态；随机历史 = 第二段五族；H1 = G0（主几何点）上 H1d–H1g 那几族里两边都挂上的格，乙 的重读在那里起作用）：

```text
$ grep 'name=r4_q5 .*column=both_mounted' research/results/e158-root-choice-repair-2026-09-27-r4-compare.out | grep -E '@(GEOMETRY_PRIMARY|harness-4gib)' | awk '{for(i=1;i<=NF;i++){n=index($i,"=");if(n)v[substr($i,1,n-1)]=substr($i,n+1)}; split(v["family"],fg,"@"); f=fg[1]; g=(f ~ /^(l[0-9]|l1|pc554)/)?"L":(f ~ /^h1/)?"H1":"rand"; k=v["arm"]" "g; c[k]+=v["cells"]; t[k]+=v["total_extra_read_calls"]; if(!(k in mn)||v["minimum_extra_read_calls"]+0<mn[k])mn[k]=v["minimum_extra_read_calls"]+0; if(!(k in mx)||v["maximum_extra_read_calls"]+0>mx[k])mx[k]=v["maximum_extra_read_calls"]+0} END{for(k in c) print k, "cells="c[k], "total="t[k], "min="mn[k], "max="mx[k]}' | sort
bing-cfg-carry H1 cells=62 total=96568 min=1544 max=1614
bing-cfg-carry L cells=635 total=-10831 min=-18 max=-17
bing-cfg-carry rand cells=313 total=-5321 min=-17 max=-17
bing-cfg H1 cells=70 total=96152 min=-21 max=1610
bing-cfg L cells=635 total=-13371 min=-22 max=-21
bing-cfg rand cells=313 total=-6573 min=-21 max=-21
jia-cfg-carry L cells=629 total=5032 min=8 max=8
jia-cfg-carry rand cells=313 total=2504 min=8 max=8
jia-cfg H1 cells=8 total=32 min=4 max=4
jia-cfg L cells=629 total=2516 min=4 max=4
jia-cfg rand cells=313 total=1252 min=4 max=4
jia-slot H1 cells=132 total=0 min=0 max=0
jia-slot L cells=632 total=0 min=0 max=0
jia-slot rand cells=532 total=0 min=0 max=0
yi-cfg-carry H1 cells=124 total=193722 min=1474 max=1643
yi-cfg-carry L cells=629 total=4988 min=7 max=8
yi-cfg-carry rand cells=313 total=2503 min=7 max=8
yi-cfg H1 cells=132 total=193254 min=3 max=1639
yi-cfg L cells=629 total=2472 min=3 max=4
yi-cfg rand cells=313 total=1251 min=3 max=4
yi-narrow-cfg-carry H1 cells=100 total=4120 min=-40 max=120
yi-narrow-cfg-carry L cells=629 total=4988 min=7 max=8
yi-narrow-cfg-carry rand cells=313 total=2503 min=7 max=8
yi-narrow-cfg H1 cells=108 total=3748 min=-44 max=116
yi-narrow-cfg L cells=629 total=2472 min=3 max=4
yi-narrow-cfg rand cells=313 total=1251 min=3 max=4
yi-narrow-slot H1 cells=194 total=1657 min=-52 max=108
yi-narrow-slot L cells=632 total=-40 min=-1 max=0
yi-narrow-slot rand cells=532 total=-73 min=-1 max=0
yi-slot H1 cells=194 total=96053 min=-1 max=1631
yi-slot L cells=632 total=-40 min=-1 max=0
yi-slot rand cells=532 total=-73 min=-1 max=0
```

- 合法状态上：乙-配置 每格 +3 或 +4、乙-配置续 +7 或 +8，甲 两臂 +4 / +8，丙 两臂比今天**少**读（−21 / −22、−17 / −18，每格都比今天少）。格数只在同格数的臂之间可比（-配置 各臂 629、丙 635、-槽 632，拒的格不同）。
- **甲-槽、乙-槽、乙-窄读-槽 的 Q5-同成 按 V1 ④ 作废**（PC-多读 (ii) 登记预言 +3S = 24、量到 0，读代码推的 +0 在装置之前就交了主 agent，认定第 1 项定为走 F18、作废）；表里 -槽 三臂那几行只当明细。
- H1d 上乙 的重读（N_真 为真、故障在 R 轮之内撤掉、重读之后挂上）的代价，整行（compare 第 657、896 行）：

```text
E7RESULT name=r4_q5 arm=yi-cfg column=both_mounted family=h1d@GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=32 positive_cells=32 total_extra_read_calls=50136 total_extra_read_bytes=207872000 minimum_extra_read_calls=1488 maximum_extra_read_calls=1639
E7RESULT name=r4_q5 arm=yi-narrow-cfg column=both_mounted family=h1d@GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=24 positive_cells=23 total_extra_read_calls=942 total_extra_read_bytes=5744640 minimum_extra_read_calls=-38 maximum_extra_read_calls=111
```

**第 22 条那一格（Q6）**：L5 每一格 `Ok` 且隔离总数 < 孪生镜像上不注入时的 ⇒ 没罩住。今天那一臂没罩住 3 / 72（n1 = 1、2、3，b = unit_first_copy，m = 2，O(2 起)），候选各臂 0：

```text
E7RESULT name=r4_q6 arm=today geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=72 not_covered_cells=3 verdict=not_covered
E7RESULT name=r4_q6 arm=yi-cfg geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=72 not_covered_cells=0 verdict=covered
E7RESULT name=r4_q6 arm=yi-cfg-carry geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) cells=72 not_covered_cells=0 verdict=covered
```

Q6 对候选各臂不算判据（「罩住」由臂的定义推得出，登记第六节开头），判别力在今天那一臂：PC-22 的 [今] 句 `pass`。L5 这个构造分不出乙与甲：乙 三臂在 L5 上没有一格是重读之后读出、挂上的（O(m 只) 只打坏两份合计的第 m 次读，另一份读得出，走不到第 22 条那一形；O(m 起) 一直读坏，重读也读坏）。

**阳性对照与 V1 作废范围（以五段合并的 compare 为准）**：跨臂句子按对照、标签、判定数：

```text
$ grep 'name=r4_cross_arm_sentence' research/results/e158-root-choice-repair-2026-09-27-r4-compare.out | awk '{c="";v="";t="";for(i=1;i<=NF;i++){if($i~/^control=/)c=substr($i,9); if($i~/^verdict=/)v=substr($i,9); if($i~/^tag=/)t=substr($i,5)}; print c, t, v}' | sort | uniq -c
      3 PC-22 arm fail
     12 PC-22 arm not_constructible
     17 PC-22 arm pass
      1 PC-22 today pass
     24 PC-extra-reads-ii arm fail
     64 PC-extra-reads-ii arm pass
     10 PC-extra-reads-iii arm fail
     38 PC-extra-reads-iii arm pass
     20 PC-extra-reads-iii measure pass
      6 PC-extra-refusal arm fail
     11 PC-extra-refusal arm pass
     11 PC-extra-refusal measure pass
      6 PC-extra-refusal measure vacuous
     17 PC-extra-refusal today pass
     88 PC-N0 arm pass
      6 PC-N-cover arm pass
      6 PC-N-ring arm pass
      9 PC-random arm pass
```

各臂自己产物里的对照（PC-N-环、PC-N-盖、PC-O、PC-丢写、PC-N0 的逐臂句子、PC-554、PC-随）只有两行 `verdict=fail`：`…-r4-seg1-yi-narrow-cfg.out` 与 `…-r4-seg1-yi-narrow-cfg-carry.out` 各自 `control=PC-N-cover form=reads_zeros duration=t1` 那一行（PC-N-盖，读回全零、时长 T(1) 那一格三句 [臂] 打不中，`missed_routes=F16,F16,F16`）。`r4_void` 126 行里 `voided=true` 26 行，原样：

```text
E7RESULT name=r4_void arm=jia-cfg quantity=Q6 voided=true basis=PC-22:refused_only(arm)
E7RESULT name=r4_void arm=jia-cfg-carry quantity=Q6 voided=true basis=PC-22:refused_only(arm)
E7RESULT name=r4_void arm=jia-slot quantity=Q5-both-mounted voided=true basis=PC-extra-reads-ii:point_value[n1=0 c=C](arm),PC-extra-reads-ii:point_value[n1=0 c=U](arm),PC-extra-reads-ii:point_value[n1=1 c=C](arm),PC-extra-reads-ii:point_value[n1=1 c=U](arm),PC-extra-reads-ii:point_value[n1=2 c=C](arm),PC-extra-reads-ii:point_value[n1=2 c=U](arm),PC-extra-reads-ii:point_value[n1=3 c=C](arm),PC-extra-reads-ii:point_value[n1=3 c=U](arm)
E7RESULT name=r4_void arm=jia-slot quantity=Q6 voided=true basis=PC-22:refused_only(arm)
E7RESULT name=r4_void arm=yi-cfg quantity=Q5-refused voided=true basis=PC-extra-reads-iii:first_pass_reads_equal_jia_total[control=PC-N-ring form=read_fails](arm),PC-extra-reads-iii:first_pass_reads_equal_jia_total[control=PC-N-ring form=reads_zeros](arm)
E7RESULT name=r4_void arm=yi-cfg-carry quantity=Q5-refused voided=true basis=PC-extra-reads-iii:first_pass_reads_equal_jia_total[control=PC-N-ring form=read_fails](arm),PC-extra-reads-iii:first_pass_reads_equal_jia_total[control=PC-N-ring form=reads_zeros](arm)
E7RESULT name=r4_void arm=yi-cfg-carry-three-rounds quantity=Q4 voided=true basis=PC-extra-refusal:arm_refuses(arm)
E7RESULT name=r4_void arm=yi-cfg-carry-three-rounds quantity=Q6 voided=true basis=PC-22:refused_from(arm),PC-22:ok_only_with_twin_isolation(arm)
E7RESULT name=r4_void arm=yi-cfg-three-rounds quantity=Q4 voided=true basis=PC-extra-refusal:arm_refuses(arm)
E7RESULT name=r4_void arm=yi-cfg-three-rounds quantity=Q6 voided=true basis=PC-22:refused_from(arm),PC-22:ok_only_with_twin_isolation(arm)
E7RESULT name=r4_void arm=yi-narrow-cfg quantity=Q1-Q3 voided=true basis=PC-N-cover:destination_chosen_or_applied[form=reads_zeros duration=T1](arm),PC-N-cover:ok_after_one_round_chosen_hidden_no_rollback[form=reads_zeros duration=T1](arm),PC-N-cover:row_publish_txg_is_hidden_txg_plus_one[form=reads_zeros duration=T1](arm)
E7RESULT name=r4_void arm=yi-narrow-cfg quantity=Q5-refused voided=true basis=PC-extra-reads-iii:first_pass_reads_equal_jia_total[control=PC-N-ring form=read_fails](arm),PC-extra-reads-iii:first_pass_reads_equal_jia_total[control=PC-N-ring form=reads_zeros](arm)
E7RESULT name=r4_void arm=yi-narrow-cfg-carry quantity=Q1-Q3 voided=true basis=PC-N-cover:destination_chosen_or_applied[form=reads_zeros duration=T1](arm),PC-N-cover:ok_after_one_round_chosen_hidden_no_rollback[form=reads_zeros duration=T1](arm),PC-N-cover:row_publish_txg_is_hidden_txg_plus_one[form=reads_zeros duration=T1](arm)
E7RESULT name=r4_void arm=yi-narrow-cfg-carry quantity=Q5-refused voided=true basis=PC-extra-reads-iii:first_pass_reads_equal_jia_total[control=PC-N-ring form=read_fails](arm),PC-extra-reads-iii:first_pass_reads_equal_jia_total[control=PC-N-ring form=reads_zeros](arm)
E7RESULT name=r4_void arm=yi-narrow-cfg-carry-three-rounds quantity=Q4 voided=true basis=PC-extra-refusal:arm_refuses(arm)
E7RESULT name=r4_void arm=yi-narrow-cfg-carry-three-rounds quantity=Q6 voided=true basis=PC-22:refused_from(arm),PC-22:ok_only_with_twin_isolation(arm)
E7RESULT name=r4_void arm=yi-narrow-cfg-three-rounds quantity=Q4 voided=true basis=PC-extra-refusal:arm_refuses(arm)
E7RESULT name=r4_void arm=yi-narrow-cfg-three-rounds quantity=Q6 voided=true basis=PC-22:refused_from(arm),PC-22:ok_only_with_twin_isolation(arm)
E7RESULT name=r4_void arm=yi-narrow-slot quantity=Q5-both-mounted voided=true basis=PC-extra-reads-ii:point_value[n1=0 c=C](arm),PC-extra-reads-ii:point_value[n1=0 c=U](arm),PC-extra-reads-ii:point_value[n1=1 c=C](arm),PC-extra-reads-ii:point_value[n1=1 c=U](arm),PC-extra-reads-ii:point_value[n1=2 c=C](arm),PC-extra-reads-ii:point_value[n1=2 c=U](arm),PC-extra-reads-ii:point_value[n1=3 c=C](arm),PC-extra-reads-ii:point_value[n1=3 c=U](arm)
E7RESULT name=r4_void arm=yi-narrow-slot quantity=Q5-refused voided=true basis=PC-extra-reads-iii:first_pass_reads_equal_jia_total[control=PC-N-ring form=read_fails](arm)
E7RESULT name=r4_void arm=yi-narrow-slot-three-rounds quantity=Q4 voided=true basis=PC-extra-refusal:arm_refuses(arm)
E7RESULT name=r4_void arm=yi-narrow-slot-three-rounds quantity=Q6 voided=true basis=PC-22:refused_from(arm),PC-22:ok_only_with_twin_isolation(arm)
E7RESULT name=r4_void arm=yi-slot quantity=Q5-both-mounted voided=true basis=PC-extra-reads-ii:point_value[n1=0 c=C](arm),PC-extra-reads-ii:point_value[n1=0 c=U](arm),PC-extra-reads-ii:point_value[n1=1 c=C](arm),PC-extra-reads-ii:point_value[n1=1 c=U](arm),PC-extra-reads-ii:point_value[n1=2 c=C](arm),PC-extra-reads-ii:point_value[n1=2 c=U](arm),PC-extra-reads-ii:point_value[n1=3 c=C](arm),PC-extra-reads-ii:point_value[n1=3 c=U](arm)
E7RESULT name=r4_void arm=yi-slot quantity=Q5-refused voided=true basis=PC-extra-reads-iii:first_pass_reads_equal_jia_total[control=PC-N-ring form=read_fails](arm)
E7RESULT name=r4_void arm=yi-slot-three-rounds quantity=Q4 voided=true basis=PC-extra-refusal:arm_refuses(arm)
E7RESULT name=r4_void arm=yi-slot-three-rounds quantity=Q6 voided=true basis=PC-22:refused_from(arm),PC-22:ok_only_with_twin_isolation(arm)
```

| 作废的量 | 臂 | 依据的句子（标签） | 怎么处置 |
|---|---|---|---|
| Q1–Q3 | 乙-窄读-配置、乙-窄读-配置续 | PC-N-盖 读回全零、时长 T(1) 那一格三句（[臂]）：窄读把读回全零的记录当 journal 空槽收进缓存，E 那次发布的末条计数器找不到，按 N 为真拒 | 第一段之后主 agent 认定第 2 项：记一次输，这两臂在第 4 次跑里 Q1–Q3 不可判，不改窄读的收法、不另开一次跑（用户已定乙，窄读是对照臂） |
| Q5-同成 | 甲-槽、乙-槽、乙-窄读-槽 | PC-多读 (ii) 点值 +3S = 24、量到 0（[臂]） | 装置之前主 agent 认定第 1 项：走 F18，不改臂行 |
| Q5-拒（附带量） | 六条乙臂（配置、配置续、槽 × 乙、乙-窄读；-槽 两臂只在 `read_fails` 上） | PC-多读 (iii)「乙-X 第一遍的读 = 甲-X 整次」，PC-N-环 上差 1（1573 对 1574、1569 对 1570） | 第一段之后主 agent 认定第 3 项：记一次输；成因（读缓存去重）是执行员推的、没量证，不采信、不查；「选哪个」用同成栏，不受影响 |
| Q6 | 甲-配置、甲-配置续、甲-槽 | PC-22「甲在那个 m 上拒」的 O(m 只) 那一半（`refused_only`，[臂]）：O(2 只) 只打坏一份的一次读、另一份读得出，甲照定义照常挂载、隔离 28 = 孪生 | 第四段之后主 agent 认定第 1 项：记一次输，是登记里这一句的构造写错，不改臂、不改构造 |
| Q4、Q6 | 六条 R = 3 臂 | PC-多拒 `arm_refuses`（[臂]）`fail`、PC-22 两句（[臂]）`not_constructible` | **五段合并之后才出现，各段 compare 里没有**：R = 3 臂只跑第五段的 `h1d-uncut-r3`、`h1e-r3` 两族，没有 L 族；合并之后今天那一臂有了 L 族的格，compare 在这六臂上找不到 L3 那一格，把「没有格」记成没拒、判 `fail`（`r4-compare` 源码里 `arm_cell` 为空时 `refused=false`，读代码核过），PC-22 那两句则记 `not_constructible`。这六臂没有一行 `r4_q4` / `r4_q6`，作废不碰任何数；是 compare 的写法把「构造不出」记成了「打不中」，交主 agent |

- V1 ③「装置缺陷（全部臂一起）」没有出现（compare 里没有 `r4_device_defect` 行）。今天那一臂的 [今] 句全 `pass`（PC-多拒 17 句、PC-22 1 句；各分段 compare 里因为前提在别的段而记 `not_constructible` 的那 12 句，合并之后都有了格、都 `pass`），V1 ⑤ 没有触发任何 F 条款。
- 7.3 与跨臂核对（compare 原样）：

```text
E7RESULT name=r4_section_seven_three row=4 arm=today cells=1896 nonzero_cells=0 verdict=pass
E7RESULT name=r4_section_seven_three row=2 witness=device_n_cfg jia_arm=jia-cfg cells=1856 mismatches=0 verdict=pass
E7RESULT name=r4_section_seven_three row=2 witness=device_n_slot jia_arm=jia-slot cells=1880 mismatches=0 verdict=pass
E7RESULT name=r4_section_seven_three row=3 arm=bing-cfg cells=110 differences=0 verdict=pass
E7RESULT name=r4_section_seven_three row=3 arm=bing-cfg-carry cells=110 differences=0 verdict=pass
E7RESULT name=r4_section_seven_three row=3 arm=jia-cfg cells=110 differences=0 verdict=pass
E7RESULT name=r4_section_seven_three row=3 arm=jia-cfg-carry cells=110 differences=0 verdict=pass
E7RESULT name=r4_section_seven_three row=3 arm=jia-slot cells=110 differences=0 verdict=pass
E7RESULT name=r4_section_seven_three row=3 arm=today cells=110 differences=0 verdict=pass
E7RESULT name=r4_f11 cells=1552 cells_differing_outside_o1_and_cell_22=0 verdict=inference_holds
```

`r4_base_identity` 18 行全 `verdict=pass`、`differing=0`；十二条主族臂各 `prefixes=60`，六条 R = 3 臂 `prefixes=0`（没有跨臂比的前缀，这六行是空判）。

**F / S / V 条款，各段**（数取自各段臂产物的 `r4_family_summary`、`r4_stop_check`、`r4_failure_clause` 等行与五段合并的 compare；逐行原样在各段报告里）：

| 段 | 条款 | 结果 |
|---|---|---|
| 开跑前 | S0 | 不停：快照上逐条重核七件（择根那一遍仍丢掉读不出的根、可写挂载仍只判「落后」、取号仍写 tail = 0、发布仍在根槽 FUA 之后轮换系统配置、`rebuilt_allocator` 仍再择一次根、重放仍在第一份点名单元读不出时停、影子账仍只计数），`first_txg_of_new_instance` 与 `target_for_publish` 两件也没变（登记修订第 3 条，行号在那里） |
| 各段 | V3、V4、S1–S4 | 开跑检查（本地常量回比、臂探针 `r4_arm_code_check`、7.2 锚点）每段每臂全 `pass`；各族 `s4_mismatches=0` |
| 第一段 | F4、F6、F8、F13、F15、F16、F17 | 都没触发：今天那一臂 H1d `f4_failures=0 f6_checked=48 f6_findings=0 f13_checked=48 f13_failures=0 f15_checked=8 f15_findings=0`（`…-r4-seg1-today.out` 的 `r4_family_summary family=h1d` 行）；今天那一臂「环在且被抛弃」∪「根槽被盖」H1d 48、H1e 64（F8 的前件不成立）；PC-N-环、PC-N-盖 今天的 [今] 句全 `pass`；乙-配置、乙-配置续 在 H1d 不断格与 H1e m = 1 格的 T(1)、O(1 只) 上都 K0 且所选根 = 被藏那条根 |
| 第二段 | S5a、S5b、S7、S9、F19 | S9 的种子与写死历史 6 项全 `pass`；装置执行器与 harness 执行器逐步对拍五族 3070 步 `mismatches=0`（S7、S9 不停）；S5：实七-甲、实八两边都复现（`verdict=reproduced`），**实七-乙 harness 与装置两边都复现不出**（`…-r4-seg2b-today.out` 里 `verdict=S5b_not_reproduced_on_the_base` 那一行，S5b：不停、全部臂照跑、标「底座上复现不出」；那一段今天那一臂主路径第 4 步那一格仍是「环在且被抛弃」、丢写 1 格，其余 4 格是崩溃状态格、都读回最后确认那一版）；F19 没触发（第 1239 行 `segments_without_an_abandoned_red=0`）；PC-随 12 臂 36 句与 compare 9 句全 `pass` |
| 第三段 | F7、F16 | F7 没触发：H1g(a = 2) 上甲-配置 16 / 16 交回 K0（推的 ② 成立）；F16 在 H1f 上没触发：乙-配置、乙-配置续 不断的 T(1)、O(1 只) 格全部 `Ok`、所选根就是被藏那条根（各 32 格）；H1g 的取号崩溃核对 72 次全 `holds=true` |
| 第四段 | F1、F2、F5、F14、F11 | PC-554 12 臂全 `pass`（今天那一臂 `a_abandoned_roots_unreadable=0 a_isolated=0`、被抛弃根独占 24 个槽一个没隔离，F1 没触发）；PC-22 今天那一句 `pass`（F2 没触发）；F5（L6 8 格）、F14（每臂 4 次卸载，12 臂 48 行）全 `pass`；F11 见 compare 的 `r4_f11` 行 |
| 第五段 | 8.2 几何敏感性 | 判决格 210 格（丢写 120、多拒 66、Q6 24）逐格比 G0（主几何点）全部「不翻」。k = 2 与 R = 3 两点推动了机制（k = 2 上「根槽被盖」变 0；R = 3 上时长 T(2)、T(3) 从拒变挂上），12 格两点值不同、把门槛挪到两值之间都翻，判别力自证过；**G_S4、G_S16、G_默认、G_小环 四点没把机制推动**（前缀最多 6 次发布，根环与记录都没绕环，H1e@G_小环 64 格 `device_undecidable=false`），这四点上的「不翻」是两点同值，不算判据（第五段之后主 agent 认定第 1 项）。出处：第五段报告 `e158-r4-seg5-runner-report.md` 第四、五节 |

**判决行**：各段臂产物与各 compare 都不打 `name=verdict` 行（`grep -c 'name=verdict'` 在 86 份上都是 0），判定写在各行的 `verdict=` 字段里。五段合并的 compare 按取值数：`pass` 381、`covered` 33、`fail` 43、`not_constructible` 12、`vacuous` 6、`not_covered` 3、`inference_holds` 1；`fail` 43 = PC-多读 (ii) 24 + PC-多读 (iii) 10 + PC-22 甲 3 + R = 3 臂 PC-多拒 6，`not_constructible` 12 与 `vacuous` 6 都在六条 R = 3 臂上，`not_covered` 3 是今天那一臂在 G0（主几何点）、S16、S4 上的 Q6；`r4_missed_arm_and_today_sentences count=61` = 这 55 句加乙-窄读 两臂 PC-N-盖 各 3 句。各段臂产物里只有乙-窄读 两臂第一段第 71 行两个 `verdict=fail`；第二段今天那一臂的 `S5b_not_reproduced_on_the_base` 是登记预先写好的分支。

**它答不了的（第 4 次跑）**：

- **读缓存峰值没量**（岔路单第 3 行的够判条件要它）：第 4 次跑的登记里没有这个量，不为它加臂；乙 与乙-窄读 只报了每次挂载多读的读调用次数。第 3 行记欠，落在第四批 kb 规格里 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 乙 Q4 那一笔。
- **H-随 那 6 格怎么读**：照登记字面全算丢写，全是 ③c 尾巴，其中 2 格同时有 Q1 覆盖、那一版早已被后来的确认取代；这几格该不该算丢写，第 4 次跑只并列两栏，不判。
- **根环大小、环长两条轴上的敏感性没量到**：G_S4、G_S16、G_默认、G_小环 上的前缀最多 6 次发布，根槽按 (txg mod 3, (txg div 3) mod S) 落、txg div 3 ≤ 2，S 取 4、8、16 落点不变，环最小 12 个记录槽也写不满（推的，没量过）。要量得另立更长前缀（让根槽在 S = 4 上回绕、让记录在小环上回绕）的一族，登记外，记作下一次跑的候选。
- **第 22 条那一格上乙与甲分不开**：L5 的两种时长都走不到「第二次读实例表失败、重读读得出」，乙 的重读在那一格上没有被测到。「两份各读坏一次、重读读得出」那种时长这一次不补（补格是事后建模；它只分甲与乙，而用户已定乙），要分就登记进下一次跑。
- **PC-多读 (ii) 丙 那一句的 p**（实例表沿链读的页数）取的是 `crates/` 的沿链读（`instance_table_chain_of_root(E).page_pointers.len()`），装置与 checker 都没有公开的实例表沿链读，这个数不是装置自己解出的。
- **PC-多读 (iii) 差 1 的成因**（乙 第一遍比甲 整次少 1 次读）没量证：执行员推的「读缓存去重」不采信、不查；作废的只是附带的 Q5-拒。
- -配置 各臂在实七-甲 上 3 个注入点 `fault_not_reached`、在实八 上 4 个崩溃状态 `segment_missing`，这几格没立；实八「根还在环里、单元被复用」那一格在 -配置 各臂上只量到主路径。实七-乙 那个签名（崩溃状态读回报 `MappingStillUnreadable`）在底座上两边都复现不出（S5b）。
- 管理员回退的另一种读法（回退交回 `Ok` 也算一笔确认）只报数、不进判据（登记修订第 22 条）：-配置 各臂 H-随 两种读法都是 6 格（`lost_write_cells_rollback_aware=6`），今天那一臂两种读法都是 43 格、Q3 读回丢写 6 对 5。
- **量的是臂副本，不是合进 `crates/` 的实现**：工作树的 `crates/` 在快照之后已合进 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 乙-配置续 的实现（`crates/singlefs-core/src/mount.rs` 里 `PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion` 那一支），它与第 4 次跑的乙-配置续 臂（`scripts/mutation/tables/e158_arms.tsv` 的 `r4` 行）是不是同一个行为，没有对拍。
- 第二段的历史由 harness 的生成器定几何（4 GiB 盘），不做几何敏感性；H1d 除 k = 2 那一点外只藏最新 1 条根。
- seg2b 的产物首行指纹 `ad9f5442…` 没把 seg2b 修过的装置（`e71224fc…`）算进去，第二段两次的首行指纹相同，靠文件名与装置 sha256 分。

**单测与变异**：装置单测 95 条（`grep -c '#\[test\]' crates/singlefs-verification/src/bin/e158_root_choice_repair.rs` 输出 `95`：装置那一趟 82 条、第二段装置加 12 条、seg2b 加 1 条撞键的单测）。第 4 次跑的装置变异在 `crates/mutations.tsv` 里 24 行（`grep -c 'root_choice_repair 第 4 次跑' crates/mutations.tsv` 输出 `24`），归门禁 crates-mutation-replay，三个数待提交时 crates-mutation-replay；各趟执行员在草稿副本上逐条证红：装置那一趟 11 条抓到 11 / 无效 0 / 没红 0、第二段 11 条 11 / 0 / 0、seg2b 在主工作区副本加补丁上 12 行 12 / 0 / 0（含撞键那两条）。臂表变异 `scripts/mutation/tables/e158_r4_arm_mutations.tsv` 8 行：装置那一趟 8 / 0 / 0，第三段另跑了 M3 的臂那一半 1 / 0 / 0（H1g a = 1 上甲-配置 由 K3 变 K0、32 格丢写）。

**复跑**：`bash scripts/experiment-apparatus/replay.sh E158`（择根与修复四岔路） 里的 `driver_e158_r4_compare` 那一行复跑五段合并的 compare（只读 `research/results/` 下 seg1、seg2b、seg3、seg4、seg5 五组臂产物，不碰 `crates/`，产物头的 `E7INPUT` 行比对前删掉）。各段的臂产物不登记：它们要在 C0 快照上套 `scripts/mutation/tables/e158_arms.tsv` 的 `r4` 行编出来的副本上跑，工作树的 `crates/` 已不是今天那一臂。在副本里复跑一条臂的写法（`<段>` 取 `r4-seg1`…`r4-seg5`，第二段写 `r4-seg2`）：`task-server/task submit --cwd <副本根> -- env SINGLEFS_E158_ARM=<臂> SINGLEFS_E158_SNAPSHOT_SHA256=<指纹> SINGLEFS_E158_TODAY_BASES=<今天那一臂先跑 r4-bases 写出的目录> cargo run -q --release -p singlefs-verification --bin e158_root_choice_repair -- <段>`；第二段另要 `SINGLEFS_E158_INVESTIGATOR_REPORT=research/prompts/m2-investigate-gate74-reds-report.md` 与（今天之外的臂）`SINGLEFS_E158_SEGMENT_TWO_TODAY_PRODUCT=<今天那一臂的第二段产物>`，今天那一臂先跑；跨臂的量再跑 `r4-compare <各段产物前缀…>`。

**第 3 次跑第一段（重跑登记 `e158-r3-prereg.md`；只当参考）**

第 3 次跑只跑了第一段（H1d、H1e 与阳性对照，G0（主几何点），九条臂：今天、甲-配置、甲-配置续、甲-槽、乙-配置、乙-配置续、乙-槽、丙-配置、丙-配置续），第二段没跑；第 4 次跑的登记把它原本要跑的全部并进第 4 次跑，第一段量过的也在另一版底座（第 4 次跑的 C0 快照）上重跑（第 4 次跑登记文件头）。第 3 次跑第一段的读数因此只当参考、不当依据。它交回的两处阳性对照没过（执行员报告 `e158-r3-runner-seg1-report.md` 第一节）：PC-N 今天那一臂 `verdict=fail`（b = records 时新实例第一次发布的 txg 等于被藏那条根的 txg，写行那次发布盖掉被藏根的槽），按那次登记的 V1 字面会作废全部臂的 Q1–Q3；PC-多读 乙 三臂 `verdict=fail`。第 4 次跑把 PC-N 拆成 PC-N-环 与 PC-N-盖、把 V1 按句子标签分路，出处是第 4 次跑登记 5.8 与 11.1。

产物（快照指纹 `74912842…a41a`，写在每份第一行）：

- 九条臂与跨臂比：`e158-root-choice-repair-2026-09-27-r3-seg1-bing-cfg-carry.out`、`e158-root-choice-repair-2026-09-27-r3-seg1-bing-cfg.out`、`e158-root-choice-repair-2026-09-27-r3-seg1-compare.out`、`e158-root-choice-repair-2026-09-27-r3-seg1-jia-cfg-carry.out`、`e158-root-choice-repair-2026-09-27-r3-seg1-jia-cfg.out`、`e158-root-choice-repair-2026-09-27-r3-seg1-jia-slot.out`、`e158-root-choice-repair-2026-09-27-r3-seg1-today.out`、`e158-root-choice-repair-2026-09-27-r3-seg1-yi-cfg-carry.out`、`e158-root-choice-repair-2026-09-27-r3-seg1-yi-cfg.out`、`e158-root-choice-repair-2026-09-27-r3-seg1-yi-slot.out`。
- `r2-all` 在那份快照上重出的今天那一臂：`e158-root-choice-repair-2026-09-27-r2-all-today.out`（1055 行，末行 `E7RESULT name=done emitted=1054`，`replay.sh` 的 `driver_e158_r2_all` 那一行改指它）。它与 `…-2026-09-26-r2-all-today.out` 对不上：`r2_h1d_cell` 800 → 832 行、`r2_seventh_batch_crash_finding` 1 → 0 行（第 3 次跑执行员报告第二节）。

````

**出处 `.claude/kb/experiments/158-择根与修复四岔路.md:623-716`（整段抄，未转述）**

````markdown
### 2026-09-26：第 2 次跑第一段（重跑登记 `e158-r2-prereg.md`）

「2026-09-26：第 2 次跑第一段（重跑登记 `e158-r2-prereg.md`）」这一节是第 2 次跑第一段的记录（现状在「2026-09-27：第 4 次跑」一节）。它之后、「影响的决策」之前的各节都是第一次跑（跑前登记 `e158-preregistration.md`）的记录：那次的「被抛弃根」全靠挂载时回退造，今天的代码里没有那条路（D23（journal 的角色与格式） 已定项 14「管理员回退是挂着时的一次向前发布」），那几节的读数不带进第 2 次跑。H1c、H1d、Q1-0、K0–K4、op1 的三种入口调用、S0–S6、F1–F12 是重跑登记自己的代号，登记位在那份文件里。

**做了什么**：入库装置 `crates/singlefs-implementation-experiments/src/bin/e158_root_choice_repair/main.rs` 加了第 2 次跑那一节（模式 `r2-all` 等；第一次跑的各模式与 `replay.sh` 里它们那几行原样留着），在八份副本上各跑一次 `r2-all`：今天那一臂（工作树）、A1（岔路单第 1 行 (a)）、甲-txg、乙F-留环、乙-留环、乙-只配置、丁-留环、丁-只配置（岔路单第 2 行六臂）。七份臂副本都从同一次快照（`git rev-parse HEAD` = 73ba4a4c019b9e3fc9c92f3122bfbbdaee93c321，快照时 `crates/` 与工作树逐文件 sha256 相同）`cp -a` 出来，按 `scripts/mutation/tables/e158_arms.tsv` 逐行套用（每行旧串在副本里恰好命中一次，S2），臂改动之外的 `crates/` 与快照逐文件相同（S4），七份都编得过（S3）。岔路单第 1 行的 (b) 原形、(b) 含链、(c) 不在 `crates/` 里实现（重跑登记第三节末句），从今天那一臂的计数推出（重跑登记 12.1 第 2 条）。开跑前的停机条款 S0、S1 按登记第一节、第三节的行号逐项重读 `crates/singlefs-core`（`first_txg_of_new_instance` 在 `mount.rs:599`、`isolate_slots_referenced_only_by_abandoned_roots` 在 `:684`、账读不出只计数在 `:714`、`MountSpaceAdmission` 在 `:370`、写行之后推在 `:2744`、`unmount` 在 `:1349`），与登记写的一致、行号没漂。登记修订 7 条写在重跑登记 12.1，都在产物之前。

**前提 1–3（问题单第一节）三条都成立**：

| 前提 | 答 | 依据 |
|---|---|---|
| 前提 1（被抛弃根今天只由崩溃恢复造出） | 成立 | Q1-0：H1c 抛弃步（故障一律读错）111 次尝试、87 次挂载做成、56 次造出至少一条被抛弃根，最少 2 个故障（最新根的根槽 1 + 断链发布第一个点名单元的第一份 1）；产物行在「结果整行抄自产物」一节。b = unit（两份都坏）那 25 次第二份一次都没被读到（重放验点名单元是全部份都要过，第一份读错就停），照 V2 作废；只坏第一份就断得了链（重跑登记 12.1 第 1 条） |
| 前提 2（C331（择根倒挂压过已确认的写） 今天仍打得中、乙要重定义） | 成立 | `first_txg_of_new_instance`（`mount.rs`）仍只取可读根与自证记录的 max + 1、不读系统配置的 F；今天那一臂 87 次写行发布的 txg 与装置按这次读得出的根与记录算的 max + 1 逐次相等（`first_txg_rule_of_today_mismatches=0`）。乙族照重跑登记 5.3 分成加 8 字节的乙 / 丁四臂与零字节的乙F-留环；乙 / 丁四臂的副本在 [489, 497) 写出刚落盘那条根的 txg（关闭 U 之后解出 7 = 最新根的 txg，产物行在「结果整行抄自产物」一节） |
| 前提 3（准入改了形态，「被拒」重定义） | 成立 | `MountSpaceAdmission` 四个成员、写行之后推都照旧；装置按 K0–K4 分类，K3 / K4 按这次调用设备一层的写次数分（单测钉住）。G0（主几何点） 的 4 GiB 盘上这一段见到的挂载全是 K0、K3 或注入写失败那几次的 K4，K1、K2 一次没出现——G_小盘这一段没跑，「K1 / K2 走不走得到」还没量 |

```text
E7RESULT name=r2_q1_0_summary arm=today geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) attempted=111 not_applicable=24 void_attempts=25 mounted=87 producing_at_least_one_abandoned_root=56 abandoned_roots_total=202 fewest_faults_that_produced_one=2 first_txg_rule_of_today_mismatches=0 row_publish_txg_at_or_below_the_floor=0
E7RESULT name=r2_decoded_offsets arm=today verdict=pass tail_offset=469 instance_offset=477 floor_offset=481 registered_tail=469 registered_floor=481
E7RESULT name=r2_unmount_check arm=today context="q1-0 n1=1" verdict=pass layout_verdict=pass txg_before_unmount=4 floor_in_newest_system_configuration=[4, 4] unmount_publishes=3 expected_publishes=3 last_nonzero_byte=[481, 481] bytes_489_to_497_as_txg=[0, 0] newest_root_txg=7
```

乙-留环那一份（`e158-root-choice-repair-2026-09-26-r2-all-yi-keep-ring.out`，不在 `replay.sh` 登记里，下同）同一格 `last_nonzero_byte=[489, 489] bytes_489_to_497_as_txg=[7, 7] newest_root_txg=7`：[489, 497) 解出的正是刚落盘那条根的 txg。

Q1-0 在另外几臂上（同一套历史；各臂 `name=r2_q1_0_summary` 那一行的几个字段）：

| 臂（产物） | `producing_at_least_one_abandoned_root` | `abandoned_roots_total` | `fewest_faults_that_produced_one` | `first_txg_rule_of_today_mismatches` | `row_publish_txg_at_or_below_the_floor` |
|---|---|---|---|---|---|
| A1 | 56 | 202 | 2 | 0 | 0 |
| 甲-txg（`…-r2-all-jia-txg.out`） | 0 | 0 | none | 81 | 33 |
| 乙F-留环（`…-r2-all-yif-keep-ring.out`） | 24 | 65 | 4 | 81 | 0 |
| 乙-留环、乙-只配置、丁-留环、丁-只配置（各自那一份） | 62 | 208 | 2 | 6 | 0 |

甲-txg 一条被抛弃根都造不出来：新实例第一次发布的 txg 不看记录，恰好落回被藏那几条根的 txg，写进同一个根槽，把它们盖掉了（例 `n1=1 c1=C k=2 b=records`：所选根 (1, 2)、写行发布 txg = 3）；乙F-留环造出来的 24 次全在关闭 U 的历史上（F 把第一次发布的 txg 顶到被藏的根之上）。这几行是岔路单第 2 行的旁证，不是登记给第 2 行的量。

**H1d（崩溃恢复把最新的根当成读不出、抛弃它、取号，这次可写挂载在写行根落盘之前断；故障撤掉、再挂载）**：操作化见重跑登记 12.1 第 2 条（n1 ∈ {0, 1, 2, 3}、关闭 C、藏最新 1 条根、读错与读回清零两种造法、断链 b ∈ {records, unit, unit_first_copy}，断法取「不断 / 崩在第 j 个写之前 / 第 j 个写报错 / 写行根之前的每一次屏障报错」，j 取 0 到写行根那个写）。每臂 800 格（b = unit 那 8 个基准照 V2 作废、不进这 800 格）。三格：

| 臂 | ① 恢复落到的根引用的单元被覆盖 | ② 再挂载 | ③ 读回的是最后一次确认的那一版 | 与今天那一臂逐格比三格 |
|---|---|---|---|---|
| 今天（= 岔路单第 2 行的丙） | 128 / 800 | K0 680、K3 120（`MountError::Recovery`，冷启动读回 `MappingStillUnreadable`） | 664 / 800（另 120 读失败、12 读回别的版本、4 读回没有文件） | — |
| A1 | 128 / 800 | K0 678、K3 122 | 664 / 800 | 差 2 格：那 2 格的再挂载被拒（`AbandonedRootLedgerUnreadable`），三格里只有 ② 变 |
| (c)（两个指称） | 同今天 | 同今天 | 同今天 | 相同（(c) = 今天的实现加一笔落点未定的记录，不改择根与拒不拒） |
| (b) 原形、(b) 含链（各两个指称） | 同今天，除那 2 格 | 同今天，除那 2 格 | 同今天，除那 2 格 | 那 2 格再挂载时被抛弃根的账读不出计数 = 1，(b) 在那 2 格上重建不重建得出来这一段没算 |
| 甲-txg、乙F-留环、乙-留环、乙-只配置、丁-留环、丁-只配置 | 128 / 800 | K0 680、K3 120 | 664 / 800 | 800 格三格逐格相同 |

① 的 128 格全部落在 n1 = 0（被藏的最新根就是首个文件；站在它前一条根——写行之后的暖机根、树表 0 条——看，首个文件那次发布开的段是全空的，写行那次发布在自己的根落盘之前就把那段重新发出去）；n1 = 1、2、3 的 636 格一格都没盖、再挂载全是 K0、读回全是最后确认那一版。「挂载做完」那 16 格（n1 各 4 格）三格是：① 恢复落到新实例自己写的根、② K0、③ 读回的不是最后确认那一版（n1 ≥ 1 读回前一版的内容 12 格，n1 = 0 读回没有文件 4 格）——被当成读不出的那一版在每一臂上都丢了。按断法分（今天那一臂）：

```text
产物行见 git show 5d0b7e7c1667:research/results/e158-root-choice-repair-2026-09-26-r2-all-today.out 第 869 行（name=r2_h1d_summary）
产物行见 git show 5d0b7e7c1667:research/results/e158-root-choice-repair-2026-09-26-r2-all-today.out 第 870 行（name=r2_h1d_summary_by_cut）
E7RESULT name=r2_h1d_summary_by_cut arm=today cut=crash cases=344 cell1_overwritten_cases=56 cell2_remount_classes={"K0": 292, "K3": 52} cell3_read_back_after_remount={"failed:MappingStillUnreadable": 52, "last_confirmed": 292}
E7RESULT name=r2_h1d_summary_by_cut arm=today cut=none cases=16 cell1_overwritten_cases=0 cell2_remount_classes={"K0": 16} cell3_read_back_after_remount={"no_file": 4, "other_content": 12}
E7RESULT name=r2_h1d_summary_by_cut arm=today cut=write_fails cases=344 cell1_overwritten_cases=56 cell2_remount_classes={"K0": 292, "K3": 52} cell3_read_back_after_remount={"failed:MappingStillUnreadable": 52, "last_confirmed": 292}
E7RESULT name=r2_h1d_fork_one_derived arm=today fork_one_arm=(b)原形 referent=tree_table cases_where_it_differs_from_this_copy=2 cells=same_as_this_copy_except_those
```

a1 那一份（`…-r2-all-a1.out`）的 `name=r2_h1d_summary` 行与今天那一臂的 `name=r2_h1d_summary` 行只差 `cell2_remount_classes={"K0": 678, "K3": 122}` 与 `trigger_positive_cases=0`；岔路单第 2 行六臂那六份的 `name=r2_h1d_summary` 行 `cases`、`cell1_overwritten_cases`、`cell2_remount_classes`、`cell3_read_back_after_remount` 四栏与今天那一行逐字相同。逐格比三格用的命令（每臂一份三格表，与今天那一臂在快照上的那一份 diff；输出原样）：`cells() { grep "name=r2_h1d_cell " "$1" | sed -E 's/.* (n1=[0-9]+) tip=\([0-9, ]+\) (form=[a-z_]+) (break=[a-z_]+) (cut=[a-z_]+) (point=[a-z0-9]+) .*(cell1_overwritten=[a-z]+) .*(cell2_remount_class=K[0-9]) remount_error=([A-Za-z]+) .*(cell3_read_back_after_remount=[^ ]+) .*/\1 \2 \3 \4 \5 \6 \7 \8 \9/'; }`，结果 `a1 three_cells_differing=2`、`jia-txg three_cells_differing=0`、`yif-keep-ring three_cells_differing=0`、`yi-keep-ring three_cells_differing=0`、`yi-config-only three_cells_differing=0`、`ding-keep-ring three_cells_differing=0`、`ding-config-only three_cells_differing=0`。

**实七两段复现历史**（重跑登记 12.1 第 5 条）：今天那一臂两段都复现——故障注入大档种子基 + 110 在第 1 步（崩溃恢复抛弃根）注 `barrier_fails` 之后重开走读失败 `MappingStillUnreadable { slot: SlotNumber(50240) }`，崩溃注入快档种子基 + 16 第 3 步的崩溃状态同一形：

```text
E7RESULT name=r2_seventh_batch_fault_summary arm=today seed=7463871032432355223 drawn=6 new_findings=2 findings_reopening_to_mapping_still_unreadable=2 known_red_hits=0
产物行见 git show 5d0b7e7c1667:research/results/e158-root-choice-repair-2026-09-26-r2-all-today.out 第 1022 行（name=r2_seventh_batch_crash_summary）
```

（快照上的今天那一臂 `…-r2-all-today-snapshot.out` 第二行是 `new_findings=2`、多一条崩溃状态上的 `claimed_state_missing_unit`：快照之后另一条线改了 harness 的记录核对器，工作树上不再报它。两份的其余各行逐字节相同。）乙-留环、乙-只配置两份与快照上的今天那一臂这一段逐行相同，丁两臂只有两处抽到的读序号不同、新发现相同；A1、甲-txg、乙F-留环三臂的历史一开头就走岔了（harness 的模型按今天的规则答第一次发布的 txg，这几臂的挂载与它对不上，抽到的注入点与崩溃点都不再落在那一步），这两段历史在这三臂上没罩住，这三臂的 H1d 只能看「H1d」那张逐臂三格表（① 覆盖、② 再挂载、③ 读回最后确认那一版）。

**PC1-a / PC1-b / PC1-c**（第一段只跑今天那一臂、(c) 与 A1 的 (a) 那一句）：今天那一臂三条都过，A1 的 PC1-a 在任何写之前拒：

```text
E7RESULT name=r2_positive_control arm=today control=PC1-a node="n1=1 c1=C k=1 b=unit_first_copy" abandoned_root=(1, 4) faults=2 unintercepted=[] class=K0 error=none writes=71 abandoned_roots_unreadable=1 expected_count_by_the_device=1 isolated_slots=0 unit=allocation_record_tree_root shared=false expectation=ok_count_equals_the_device_count verdict=pass
E7RESULT name=r2_positive_control arm=today control=PC1-b node="n1=1 c1=C k=1 b=unit_first_copy" abandoned_root=(1, 4) faults=2 unintercepted=[] class=K0 error=none writes=71 abandoned_roots_unreadable=1 expected_count_by_the_device=1 isolated_slots=0 unit=tree_table shared=false expectation=ok_count_equals_the_device_count verdict=pass
E7RESULT name=r2_positive_control arm=today control=PC1-c node="n1=1 c1=C k=1 b=unit_first_copy" abandoned_root=(1, 4) faults=1 unintercepted=[] class=K0 error=none writes=71 abandoned_roots_unreadable=0 expected_count_by_the_device=0 isolated_slots=0 exclusive_placements=24 isolated_slots_without_the_fault=28 expectation=ok_count_zero_isolated_zero verdict=pass
```

A1 那一份（`…-r2-all-a1.out`）的 PC1-a 行：`class=K3 error=AbandonedRootLedgerUnreadable writes=0 … expectation=AbandonedRootLedgerUnreadable_and_zero_writes verdict=pass`，同一段历史、同一个落点（`n1=1 c1=C k=1 b=unit_first_copy`、被抛弃根 (1, 4)、分配记录树节点两份），在任何写之前拒。

PC1-c 那一行就是 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 钉的行为：被抛弃根自己的根槽读不出时计数 0、隔离 0（不注入时这次挂载隔离 28 个槽）。甲-txg 那一份两行 `verdict=fail`（`H1c 里找不到…的历史`）：甲-txg 下抛弃步一条被抛弃根都造不出来（见 Q1-0），PC1 不是岔路单第 2 行各臂的阳性对照，那两行只是照实报「这一臂上 PC1 摆不出来」。

**岔路单第 1、2 行这一段量到哪**：

| 岔路单行 | 这一段量了的 | 还差（登记的量） |
|---|---|---|
| 第 1 行（被抛弃根的账读不出时「修复」指什么） | 前提 1 的装置读数 Q1-0；PC1-a / b / c（今天、(c)、A1 的 PC1-a）；H1d 在 A1 与 (b)(c) 两个指称上的三格（A1 实跑，(b)(c) 由计数推） | Q1-1 至 Q1-5（G0（主几何点） 全族、op1 的三种入口调用、瞬时 / 持续）；(b) 两个指称的 PC 与 H1d 那 2 格上 (b) 重建不重建得出；G_S4、G_S16、G_小盘的敏感性格 |
| 第 2 行（C331（择根倒挂压过已确认的写） 怎么修） | 前提 2；六臂副本编成、臂表重写；H1d 在六臂上的三格；Q1-0 在六臂上的读数（旁证：甲-txg 与乙F-留环（关闭 C 时）让新实例的第一次发布落进被藏根的 txg、盖掉根环里那几条根） | Q2-1（H2、H2c）、PC2 与它的开关（5.3，这一段臂表里没加）、Q2-2a、Q2-2b 的解码核对之外的部分、第八节行 2 的敏感性格 |

**它答不了的（第 2 次跑第一段）**：H1d 的崩溃只取前缀那一种状态（崩溃点重放的段内子集没有枚举）；H1d 只在 G0（主几何点） 一个几何上跑、只藏最新 1 条根、只取关闭 C；岔路单第 2 行各臂的定义都只改新实例第一次发布的 txg，不改择根、也不改「要不要拒」，H1d 的三格在这六臂上相同是这几臂定义的直接后果——用户要改的那一形（崩溃恢复把暂时读不出的最新根当成被抛弃），重跑登记里没有一条臂是「系统配置说有更新的根时拒绝可写挂载」这种形状，这张表答不了那种修法；(b) 在 H1d 那 2 格上的结局没算；实七两段历史在 A1、甲-txg、乙F-留环上没罩住；第一节处理表里的删与改（shim、H3、Q3-1、N_w、PC3、PC-Nw、权重上限 12 与它们的单测、变异行、`replay.sh` 里指向它们的行）这一段没做。

**单测与变异**：装置单测 57 条（`grep -c '#\[test\]' crates/singlefs-verification/src/bin/e158_root_choice_repair.rs`），开工时红的 `mount_writable_trajectory_distinguishes_persistent_from_transient_faults` 照第一节处理表改了取被抛弃根的地方、转绿；`mount_writable_then_raise_floor_reaches_the_injected_fault` 改打分配记录树节点、断言 `RaisedFloor::abandoned_roots_unreadable > 0`。`crates/mutations.tsv` 末尾新加第 2 次跑那一节的 17 行（归门禁 crates-mutation-replay），在草稿副本上逐条改坏、跑点名的单测：17 条全红（其中「根槽写之前的屏障把紧挨在它之后的也算进去」第一遍没红，是取样点不敏感——那两种写法只在「前 0 个写」这一格上不同，补了这一格之后红），沿用的第 551、659 行在新单测上也都红。

**复跑**：（2026-11-09 该行已删，见「2026-11-09」一节 ⑥；下面是当时的复跑说明）`bash scripts/experiment-apparatus/replay.sh E158`（择根与修复四岔路） 里的 `driver_e158_r2_all` 那一行（工作树、今天那一臂：`cargo run -q --release -p singlefs-verification --bin e158_root_choice_repair -- r2-all`，2026-09-27 单独复跑这一行逐字节一致；同一编号下第一次跑的各行这一次复跑全都对不上，差在第一次跑之后 `crates/` 的字段表 481 → 489 与挂载时回退删除，不是这一段改的）。⚠️ 另一条线改了 `crates/singlefs-core/src/mount.rs`、`transaction.rs`（产物出来之后），之后在工作树上重跑 `r2-all` 已与登记的产物对不上：H1d 多出 32 格（写行根之前的屏障每个基准从 6 次变 8 次），`cell1_overwritten_cases=128`、K3 120 不变，读回最后确认那一版从 664 变 696；Q1-0、PC1 那几行不变。登记的产物照旧指改动之前那一版（与各臂副本同一份 `crates/`），要不要按新代码重出交主 agent。另外七臂：对快照 `cp -a` 一份整树，按 `scripts/mutation/tables/e158_arms.tsv` 里说明列以那一臂的臂名起头（几条臂同一行时用「、」连写）的行逐行替换（旧串恰好命中一次），再以 `task-server/task submit --cwd <副本根> -- env SINGLEFS_E158_ARM=<臂名> SINGLEFS_E158_LOCAL_SYSTEM_CONFIGURATION_BYTES=<今天、A1、甲-txg、乙F-留环 取 489；乙 / 丁四臂取 497> cargo run -q --release -p singlefs-verification --bin e158_root_choice_repair -- r2-all` 编并跑。

**产物**：`e158-root-choice-repair-2026-09-26-r2-all-today.out`（工作树，`replay.sh` 登记的那一份）、`e158-root-choice-repair-2026-09-26-r2-all-today-snapshot.out`（快照上的今天那一臂，与各臂比用它）、`e158-root-choice-repair-2026-09-26-r2-all-a1.out`、`e158-root-choice-repair-2026-09-26-r2-all-jia-txg.out`、`e158-root-choice-repair-2026-09-26-r2-all-yif-keep-ring.out`、`e158-root-choice-repair-2026-09-26-r2-all-yi-keep-ring.out`、`e158-root-choice-repair-2026-09-26-r2-all-yi-config-only.out`、`e158-root-choice-repair-2026-09-26-r2-all-ding-keep-ring.out`、`e158-root-choice-repair-2026-09-26-r2-all-ding-config-only.out`。每份末行是 `E7RESULT name=done emitted=N`，N 等于这份里 `E7RESULT` 行数。

**问题**：跑前登记 `e158-preregistration.md`——为里程碑「覆盖写、释放、回退与复用」收口表第 ③、④、7、16 行等「择根与修复」的四条岔路（C393（被抛弃根的账读不出时修复没有条款）、C331（择根倒挂压过已确认的写）的修法、C332（回退实例两个根都读不出时回退被撤销）、C334（切换的所选根没有会红的检查））配代价数。装置放在**入库装置**上：`crates/singlefs-verification` 新加只读 bin `e158_root_choice_repair`，驱动真实 `mount_writable` / `mount_rollback` / `recover`，不改 `crates/singlefs-core`、`crates/singlefs-checker` 的生产代码。第一段（2026-09-23）做跑前登记「5.8」第一段：停机条款 S1–S5 逐项对拍、5.7 常量回比、第七节全部锚点、H3（岔路 3 的历史族）不注入全跑出 N_w 与轨迹、Q3-2、Q2-3 的算术、PC-Nw、PC3——对应岔路单第 3 行（候选 2 那一半）与第 2 行的 ③。**这一次（2026-09-24）**先把第一段在今天的 `crates/` 上重跑、逐格诊断与入库产物的差异（`crates/` 在此期间落地了 C512（树表 0 条的一版上被换下的实例表记在哪没有条款） 定案，拿掉了「回退到无文件那一版」的拒绝），再做跑前登记「5.8」第二段：Q3-1（岔路单第 3 行候选 3 那一半：H3 × Φ3 的 P332（违例判据） 违例比例，今天那一臂）与第八节行 3 的敏感性格。岔路单第 1、2 行（第四、三段）与第 4 行（第五段，前置未满足）这一次没有做，见「它答不了的」。

````

**出处 `.claude/kb/experiments/158-择根与修复四岔路.md:717-729`（整段抄，未转述）**

```markdown
### 这一段做了什么

1. **S1（开跑前重读第三节表里每个函数）**：现查 `choose_root` 比较键 `(checkpoint_txg, instance)`（不读实例表，在 `crates/singlefs-core/src/recovery.rs` 里）、`first_txg_of_new_instance` 取 `max(可读根最大 txg, 环里自证记录最大 txg)+1`、`mount_writable` 的 `next_counter` 取环里最大计数器 + 1（jsn，= 甲-jsn = 今天）、`mount_rollback` 的 `next_counter` 同取环里最大计数器 + 1、`isolate_slots_referenced_only_by_abandoned_roots` 读不出账时只 `unreadable += 1; continue`（不隔离、不拒绝挂载，在 `mount.rs` 里）、`SystemRuntimeQuantities` 恰 3 个字段（`slot_generation`、`journal_tail`、`journal_instance`）。全部与第三节写的相符，S1 不触发。S2/S3（臂副本编译）本段不适用——只跑「今天」这一臂，不建 `e158_arms.tsv` 副本。**S4**：`git rev-parse HEAD` = `3b60f098e97dc4c4f3ed9c6355422b607db1c34c`、`git status --porcelain crates/` 74 行、`sha256sum crates/singlefs-core/src/*.rs crates/singlefs-format/src/*.rs | sha256sum` = `c23b2008600d66067f63373474059f0aa62af1f9ab227a4fac20e8b95e704fc8`，开跑前与交回前两次相同（主工作区在这一段稳定）。**S5**：装置用 `singlefs_checker::image`（另一套独立解析代码）解根环，与 `singlefs_core::recovery::readable_roots` 逐条比对（`name=s5_independent_decode_cross_check verdict=pass readable_root_count=6`），另有单测 `bootstrap_timeline_matches_independent_decode` 做同一件事的最小版本。
2. **5.7 常量回比**：18 个本地常量（根环区域数、每区槽数上下界与 mkfs 默认、journal 环长/记录宽/起点槽、落点粒度、系统配置字段表合计/槽宽/每盘槽数、根记录宽、实例表行宽与一片记录数、系统配置整槽校验和偏移、三个区域住哪块盘）逐条与 `crates::` 同名常量比对，18 条全 `pass`（`name=local_constant_check` × 18）。
3. **第七节全部锚点**：7.2 的 11 条独立算术 + 3 个几何点的根槽总数与 Φ3 故障集合个数 + 3 个环长的记录数/在飞上限，共 20 条全 `pass`；7.1 里能在本段核的 1 条（D22（单元原子性怎么合成） 已定项 9「481/3615/512 没跨」）`pass`，另外 4 条（C332（回退实例两个根都读不出时回退被撤销）「2 个故障」、C393（被抛弃根的账读不出时修复没有条款）「只计数」、D23（journal 的角色与格式） 已定项 14「N_switch=3」、C334（切换的所选根没有会红的检查） 判别力自证）与 7.3 的 4 行都标 `deferred`，留给对应段。
4. **H3 全枚举（不注入）**：四个几何点——`GEOMETRY_PRIMARY`（S=8，环 3 MiB）、`S16`（S=16，环 3 MiB）、`小环`（S=8，环 49152 字节，8.2 的搜法从 48 KiB 起找到的第一个跑得通的值）、`S4`（S=4，环 3 MiB，另跑 L=4）——每个几何点在 overwrites_before_sigma∈{1,2,3} 上穷举 σ∈{W,M,R(j)}^L（R(j) 取那一刻装置自己解出的每一条可读根），逐节点算「还有可读被抛弃根的回退事件数」（events_live）。`GEOMETRY_PRIMARY`/`S16`/`小环` 三个几何点历史数、峰值、正步数逐字段相同（856 段历史，峰值 3），`S4` 因根环容量小（12 槽）在 L=4 下出现峰值 4（7264 段历史）。四个几何点合计 N_w = max(3,3,3,4) = **4**。剪枝分支只有两类（`RollbackTargetNotACandidate`、`RollbackToVersionWithoutFileWhileTheRingStillHoldsAFileVersion`），逐条现查确认是候选集排除与「树表 0 条时环里还有带文件版本的根」两种合法拒绝，不是装置的错。
5. **Q3-2/Q2-3 算术**：单条见证 16/12 字节都放得下（512 以内余量 15/19）；表形态 `1+16*N_w`（65 字节）与 `1+12*N_w`（49 字节）都推过 512（余量 −34/−18），与候选 2 见证同放系统配置槽（沿用 N_w）时余量 −42/−26，同放也推过 512。四种宽度在 4096 以内都放得下。
6. **PC-Nw**：单测 `pc_nw_counts_two_live_rollback_events_from_a_hand_written_ledger`（手写两次回退、两段被抛弃根都读得出的假账，不碰 `crates/`）数出 2；对照单测 `pc_nw_does_not_count_a_rollback_event_whose_abandoned_roots_are_all_unreadable` 数出 0。
7. **PC3**：H3 全族里按固定次序第一个满足 events_live≥1 的历史是 `[Overwrite, Overwrite, Overwrite, RollbackTo(1, 3)]`（`GEOMETRY_PRIMARY`）——三次覆盖写之后回退到根 (1,3)，被抛弃的 (1,4)(1,5)(1,6) 仍可读。F = 回退之后新实例 2 自己的 2 条根槽（把「回退实例」操作化成刚做完 R(j) 建立起来的新实例，按第四节已有用例「回退实例 3 的四条根……四条都读不出时恢复退到被抛弃的根 (2, 8)」的形态；⚠️ **这个操作化没有走 5.6，交回报告点名，请主 agent 核对与 C332（回退实例两个根都读不出时回退被撤销） 攻方腿原文是不是同一个意思**）：检查 (a) 带故障的冷启动 `recover`（走 `PoolReaderWithUnreadableRootRingSlots`）与 (b) 带故障的 `mount_writable`（走 `FaultInjectingBlockDevice` + `SharedFaultPlan`）都选中 (1,6)，与期望「读得出的被抛弃根里 (txg,实例) 最大的那一条」一致，两处都 `pass`。
8. **小环取法的代理探针**：8.2 原文的校准点是「H2 最长模板」（n1=6、mount_writable、n2=2）整段跑得通；H2 不在这一段实现，用 `small_ring_probe` 复刻同样的操作序列（不判是哪一类错，只判有没有错）代替，48 KiB 第一次就通过。⚠️ 这个简化交回报告点名，主 agent 核这个代理够不够格。
9. **变异**：3 条，全部手工验证红→绿→复原绿（本地常量每区槽数 8→9、系统配置槽余量算术 4096-481→4096-480、活回退事件计数只数最近一次）。三行在 `crates/mutations.tsv` 末尾（「E158（择根与修复四岔路） root_choice_repair」开头的三行），归门禁 crates-mutation-replay 跑；门禁 code-source-discipline 的 mutation-tables 格核过原文各命中源码一次。
10. 命名纪律：`naming-lint.sh` 对新文件全部通过（0 处违规；改了 `n1`→`overwrites_before_sigma`、`G0`（主几何点）/`G_S16`/`G_S4`→`GEOMETRY_PRIMARY`/`GEOMETRY_LARGER_ROOT_RING`/`GEOMETRY_SMALLER_ROOT_RING` 等约 15 处单字母/字母加数字的标识符）。

```

**出处 `.claude/kb/experiments/158-择根与修复四岔路.md:730-1438`（整段抄，未转述）**

````markdown
### 2026-09-24：第一段重跑（产物结构性对不上，原因已诊断）

第一段自 2026-09-23 之后没有重跑过；这一次重跑 `bash scripts/experiment-apparatus/replay.sh E158`（择根与修复四岔路），走的是
`driver_e158`（参数 `-- all`），与入库产物 `e158-root-choice-repair-2026-09-23-segment1.out` 比对：
**对不上，50 行不同**。根因是
`crates/singlefs-core/src/mount.rs` 在此期间落地了 C512（树表 0 条的一版上被换下的实例表记在哪没有条款） 定案，
拿掉了 `MountError::RollbackToVersionWithoutFileUnsupported` 与 `InstanceRowsOnVersionWithoutFileUnsupported`
两个拒绝分支（`format_time_allocator` 文档注释里 2026-09-23 追加的一段原话：「这一道今天的依据……它**不再**拦
任何一条正当历史」），H3 的 `R(j)` 分支从此能真的回退到树表 0 条的根（如 `(0,0)`），穷举树因此变大：

| 差异 | 旧（2026-09-23） | 新 | 原因 |
|---|---|---|---|
| `h3_family_summary` 与同一段里另一行结果（`crates/singlefs-implementation-experiments/src/bin/e158_root_choice_repair/main.rs` 里打 `N1_DONE` 的那一行，GEOMETRY_PRIMARY/S16/小环） | histories=856，peak_roots_live=7 | histories=1822，peak_roots_live=10 | 回退到无文件版本的根现在能走到，树变大，派生量（`leaf_count`/`error_count` 等）跟着变 |
| `h3_family_summary`（S4） | histories=7264 | histories=14252 | 与 GEOMETRY_PRIMARY/S16/小环 那一行同一个原因 |
| `h3_family_pruned_branch_category` | `RollbackToVersionWithoutFileWhileTheRingStillHoldsAFileVersion`（`crates::MountError` 的一个已删除变体） | 「现行版本没有文件」（**装置自己** `apply_overwrite` 的守卫，不是 `crates/` 的拒绝） | 拒绝从 `crates/` 移走后，装置在「回退到无文件版本之后再做一次覆盖写」时自己的守卫接手报错，错误分类逻辑按字符串首词抓取字面对上了 |
| `pc3` 的 `path` | `RollbackTo(1, 3)` | `RollbackTo(0, 0)` | PC3 取「按固定次序第一个满足 events_live≥1 的历史」，`(0,0)` 现在合法且排在更前；**核心断言不变**：`expected_root=(1,6)`，`recover`/`mount_writable` 两处仍 `verdict=pass` |

**S1 现查**（`choose_root` 比较键、`first_txg_of_new_instance` 公式、`next_counter`、`isolate_slots_referenced_only_by_abandoned_roots`
只计数不拒绝）全部与第三节写的相符，S1 不触发；今天新增的 `tree_identifier_watermark_of_the_ring` 管的是
D8（核心索引结构） 已定项 8 ② 的树 ID 水位，不是 C331（择根倒挂压过已确认的写） 的择新逻辑，不影响「今天=丙=甲-jsn」这句话。

**第一段承重的量不受影响**：Q3-2 的 N_w=4、witness 宽度 margin（四种宽度的 fits/past_512/past_512_together 判定）、
PC3 的核心断言在新产物里逐字节相同（`path` 字段除外）。**岔路单第 3 行候选 2 那一半、第 2 行 ③ 在今天的代码上
依然成立，不需要重跑。** 新产物另存 `e158-root-choice-repair-2026-09-24-segment1-rerun.out`，
旧产物原样留着，**两份都点名，承重哪一份交主 agent 定**（我的判断是新产物结构性正确反映了今天的 `crates/`，
但按定义不许自己下这个结论）。

**第二段 Q3-1（岔路单第 3 行候选 3 那一半）**

新增内容全部在 `e158_root_choice_repair.rs` 这一份 bin 里（`crates/singlefs-core`、`crates/singlefs-checker`
一行未动）：`SimNode` 加一个只增不删的 `content_by_root: BTreeMap<TimelineRoot, Option<Vec<u8>>>`
（P332（违例判据） 的 V2 唯一需要的新状态，记录每一条根曾经对应的「内容号」），P332（违例判据） 判定（V1=所选根落在被抛弃集合里，
V2=读回的内容号不属于当前时间线上任何一个根）在 (a) 带故障冷启动 recover、(b) 带故障 `mount_writable` 之后
再带同一故障冷启动、(c) 撤故障之后冷启动三处各判一次，`|F|≤2` 的 Φ3 全枚举（含空集），5.1「H3 的省法」
（(a) 所选根与 F=∅ 相同就跳过 (b)(c)）按跑前登记原样实现。新增 7 条单测（12 条全绿，逐条改坏证明会红——
`content_is_off_the_current_timeline` 取反、`is_abandoned` 换成 `.all()`、`combinations_of_size` 的 0 判成 1，
三处改坏后按预期各自只让对应测试红，还原后复跑全绿）；3 条变异追加进 `crates/mutations.tsv` 末尾（只追加，
`git diff` 核过没碰别人的行），门禁 code-source-discipline 的 mutation-tables 格核过原文各命中源码一次。`naming-lint.sh` 全部通过（约 45 处重命名：
`v1`/`v2`→`chosen_root_abandoned`/`content_off_timeline`，检查点 `a`/`b`/`c`→
`cold_recover_with_fault`/`mount_then_recover_with_fault`/`recover_after_fault_removed` 等）。

**开跑之前先做了代价标定**：G0（主几何点） 满深度第一次探测在 100 秒里一行输出都没有；标定发现是 (a) 每个故障集合都
重建一次只读镜像，改成整节点建一次、56 个故障集合共用（不改判定逻辑，只读镜像可以安全复用）之后，
G0（主几何点） 全量从预估「层一单独跑不完」降到 real 2m58s。**这处优化是产物之前发现并修的**，按
`evidence-discipline.md`「产物之前修订」记进跑前登记第十二节，原判据、原臂原样保留。

**结果**（G0（主几何点）/S16/小环三点逐字节相同，整行抄自产物（产物已移出工作区的行写成一行指针））：

```text
产物行见 git show 5d0b7e7c1667:research/results/e158-root-choice-repair-2026-09-24-q3-1-g0.out 第 57 行（name=fault_set_violation_summary）
E7RESULT name=fault_set_violation_by_weight geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) weight=0 pairs=1822 violations=0
E7RESULT name=fault_set_violation_by_weight geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) weight=1 pairs=23752 violations=0
产物行见 git show 5d0b7e7c1667:research/results/e158-root-choice-repair-2026-09-24-q3-1-g0.out 第 60 行（name=fault_set_violation_by_weight）
产物行见 git show 5d0b7e7c1667:research/results/e158-root-choice-repair-2026-09-24-q3-1-g0.out 第 61 行（name=fault_set_violation_by_rollback_presence）
产物行见 git show 5d0b7e7c1667:research/results/e158-root-choice-repair-2026-09-24-q3-1-g0.out 第 62 行（name=fault_set_violation_by_checkpoint_and_predicate）
```

**读法**：N_pairs=170719（1822 个 H3 节点 × 每节点 `|F|∈{0,1,2}` 全部子集）；N_viol=213，比例约 **0.1248%**。
**违例只出现在 `|F|=2` 且历史含至少一次回退**（`|F|∈{0,1}` 与「不含回退」三类各 0 违例）——与 C332（回退实例两个根都读不出时回退被撤销） 正文
「2 个故障」的门槛方向一致（未触发 F2：最小 `|F|` 不是 1、3、4，也不是「≤4 都找不到」）。213 个违例对里
(a) 全部 213 个都 V1，其中 175 个同时 V1+V2；(b)(c) 的 V1 都是 0（`mount_writable` 会把「选中被抛弃的根」
自我修正）但 V2 都是 175（读回的内容仍对不上当前时间线任何一个根，(a)(b)(c) 三处 V2 判定完全一致）。
**这 175 对具体是什么形态、是不是都落在「回退到无文件版本」这个新出现的分支上，这一段没有逐条拆开看**，
是没做完的部分（第「它答不了的」一节点名）。

**第八节敏感性（行 3）**：G_S16、G_小环两点与 G0（主几何点） 逐字节相同；G_S4（`sigma_length_limit=4`，比其余三点
多穷举一层，nodes_visited=14252，7.8 倍于 G0（主几何点）的 nodes_visited）判定方向不翻面——比例、breakdown 整行抄自产物（产物已移出工作区的行写成一行指针）：

```text
产物行见 git show 5d0b7e7c1667:research/results/e158-root-choice-repair-2026-09-24-q3-1-s4.out 第 57 行（name=fault_set_violation_summary）
E7RESULT name=fault_set_violation_by_weight geometry=S4(S=4,ring=3MiB) weight=0 pairs=14252 violations=0
E7RESULT name=fault_set_violation_by_weight geometry=S4(S=4,ring=3MiB) weight=1 pairs=170243 violations=0
产物行见 git show 5d0b7e7c1667:research/results/e158-root-choice-repair-2026-09-24-q3-1-s4.out 第 60 行（name=fault_set_violation_by_weight）
产物行见 git show 5d0b7e7c1667:research/results/e158-root-choice-repair-2026-09-24-q3-1-s4.out 第 61 行（name=fault_set_violation_by_rollback_presence）
产物行见 git show 5d0b7e7c1667:research/results/e158-root-choice-repair-2026-09-24-q3-1-s4.out 第 62 行（name=fault_set_violation_by_checkpoint_and_predicate）
```

比例 1355/1116897 ≈ 0.1213%（G0（主几何点） 是 0.1248%，同一数量级），**同样只在 `|F|=2` 且含回退时出现**，
(a)(b)(c) 三处的 V1/V2 分布形状与 G0（主几何点） 逐比例相同（(a) 全 V1、其中约 86% 同时 V1+V2；(b)(c) 的 V1 全 0、
V2 与 (a) 的 V2 计数相同）——**四个方向相反的取样点里三个（S16、小环、S4）都实测过，判定不翻面**；
第四个（G_默认，768 MiB 环）因 `scan_journal` 按环全部记录槽逐一读、字节数是 3 MiB 环的 256 倍，
按 G0（主几何点） 的 170719 对、real 2m58s 线性外推，**这一段剩下的时间连一次全量都跑不完**，只给出机制论证
（判定只依赖「点名的槽今天存的是哪条根、它的内容号」，不依赖环总槽数——这句话已经在 S=8→S=16、
3 MiB→49152 字节两个方向上被验证），**没有独立产物**，交主 agent 定要不要另开一段跑一个只重放
已知 1355+213 个违例对应 (节点,故障集合) 的缩小版验证。

**（续，session s3）：岔路单第 1 行的 A1 副本建出来了，H1 装置本身没写**

这一段主要做了「另外两件」（`driver_e158` 改指今日承重产物、5 个驱动在主工作区新一批未提交改动之后复核字节一致，
已写进上面「结果整行抄自产物」与「复跑」两节）；在此之外，为岔路单第 1 行（C393（被抛弃根的账读不出时修复没有条款））建了候选 (a) 的 A1 副本：

- 草稿目录 `/tmp/claude-1000/e158-s3/arms/a1/`（只含 `Cargo.toml`/`Cargo.lock`/`.cargo`/`crates`，不含
  `target`），`SNAPSHOT.txt` 记快照时刻、`git rev-parse HEAD`、`git status --porcelain crates/` 行数（92，
  确认 `crates/` 此刻仍在被另一条线 `impl-m2-mountfix` 改动）、`crates/*/src/**/*.rs` 的 sha256 汇总
  （S4 停机条款要求的记录）。
- `crates/singlefs-core/src/mount.rs`：新增 `MountError::AbandonedRootLedgerUnreadable { count: u64 }`，
  在 `mount_writable`/`mount_rollback` 的 `RebuiltAllocator` 解构之后、`establish_instance`（取号，第一次写）
  之前，`abandoned_roots_unreadable > 0` 就提前返回这个错误——按跑前登记 5.2 (a) 逐字实现。
- 收尾两处穷尽 `match` 要跟着补一支才编得过：`crates/singlefs-verification/src/history.rs` 的
  `mount_error_member`、`crates/singlefs-verification/src/model_comparison.rs` 的 `refusal_reason_of_mount_error`
  与 `reported_ceiling_of_mount_error`（新变体归「模型没有理由」的默认档，不改判定逻辑）。
- 四处改动记进 `scripts/mutation/tables/e158_arms.tsv`（新建，四列同 `crates/mutations.tsv`：说明、文件、旧串、
  新串，逐条用 Edit 工具应用、Edit 本身保证旧串在改动那一刻的文件里恰好命中一次）。
- 编译核过：`cargo build --release -p singlefs-verification --bin e158_root_choice_repair`（在 A1 副本目录下）
  `Finished` 无错误无告警。
- **侦察结论**：`crates/singlefs-core::recovery` 里 `tree_table_has_no_entries`、`allocation_records_under_root`
  （都是 `pub fn`）与 `RootRecord.tree_table: NodePointer`、`NodePointer.locations: [LocationEntry; 2]`、
  `LocationEntry{device, slot, ..}`、`SlotNumber::to_device_offset()`（都是 `pub`）合起来就是 `mount.rs` 里
  `isolate_slots_referenced_only_by_abandoned_roots` 判定 `abandoned_roots_unreadable` 时走的同一条路径——
  这条路径**恰好对应跑前登记 5.2 的两个指称**：`tree_table_has_no_entries` 返回 `Err` ⇒ 树表指称的读失败；
  返回 `Ok(false)` 之后 `allocation_records_under_root` 内部再失败 ⇒ 分配记录树指称的读失败（`Ok(true)`，
  即树表 0 条那一版，走 `root.allocation_record_tree_root.locations` 直接判）。

**（续，session s3 第二段）：H1 家族 + Φ1 故障注入 + PC1-a/PC1-b 写出来了，Q1-1 至 Q1-4 有真数了**

写了 H1 全族枚举（`h1_family`：n1∈{1,2,3}、j 取每一条可读根、m∈{0,1,2}，回退到无文件版本之后 `apply_overwrite`
按今天的守卫拒绝、m 停在那一步）、Φ1 故障注入（`attempt_op1`：按 (设备, 偏移) 精确注入，落点由
`locate_allocation_record_tree` 现查，5.2 (a)/(c) 用同一个函数、跨臂只用 `error_member_of_debug` 的字符串
首词识别 A1 副本才有的 `AbandonedRootLedgerUnreadable`，不直接 `match` 枚举名——这份装置源码两边共用一份，
`match` 会让编不过「今天」那份没有这个变体的 `crates/`）、PC1-a/PC1-b（`run_pc1_a_and_b`：按固定次序找第一条
「分配记录树节点不共享」的历史）。**修了一个真 bug**（产物之前发现，见跑前登记「十二、修订」）：最初
`root_record_of` 拿 `RootView::record_bytes`（截到 457 字节）喂给 `RootRecord::parse_slot`，校验和按整槽宽
（512）算的、截短之后对不上，`parse_slot` 恒 `None`——H1 全族「被抛弃可读根」一个都取不到；改成按
`checker_image::root_slot_positions` 自己重新按整槽宽读一遍。新增 4 条单测（16 条全绿，`fault_targets_for`/
`error_member_of_debug` 两条逐条改坏验证会红、还原后 `diff` 逐字节一致），3 条变异（`crates/mutations.tsv`
追加），命名与格式纪律（`naming-lint.sh`、`cargo fmt`、`cargo clippy -D warnings` 加共用约束那批 lint）全过。

**候选 (c)=今天** G0（主几何点） 上跑出的数（产物 `e158-root-choice-repair-2026-09-24-q1-g0-today.out`，
两次独立跑逐字节相同）：

```text
E7RESULT name=q1_h1_family_size geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) nodes=36 rejected_rollback_attempts=0 unresolved_units=0
产物行见 git show 5d0b7e7c1667:research/results/e158-root-choice-repair-2026-09-24-q1-g0-today.out 第 56 行（name=q1_1a_trigger_summary）
E7RESULT name=q1_1a_by_aspect_severity geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) aspect=allocation_record_tree severity=both_copies pairs=657 trigger=399
E7RESULT name=q1_1a_by_aspect_severity geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) aspect=allocation_record_tree severity=disk0_only pairs=657 trigger=0
E7RESULT name=q1_1a_by_aspect_severity geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) aspect=allocation_record_tree severity=disk1_only pairs=657 trigger=0
E7RESULT name=q1_1a_by_aspect_severity geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) aspect=tree_table severity=both_copies pairs=657 trigger=399
E7RESULT name=q1_1a_by_aspect_severity geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) aspect=tree_table severity=disk0_only pairs=657 trigger=0
E7RESULT name=q1_1a_by_aspect_severity geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) aspect=tree_table severity=disk1_only pairs=657 trigger=0
E7RESULT name=q1_1c_error_member geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) member=Recovery count=42
E7RESULT name=q1_1c_error_member geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) member=RollbackTargetNotACandidate count=1296
E7RESULT name=q1_2_allocation_record_tree geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) outcome=can_recompute count=141
E7RESULT name=q1_2_allocation_record_tree geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) outcome=cannot_recompute count=57
E7RESULT name=q1_2_allocation_released_records geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) zero=27 nonzero=114
E7RESULT name=q1_2_tree_table_copy_elsewhere geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) has_copy=9 no_copy=57
E7RESULT name=q1_3_persistence_cost_values geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) values={32768}
```

**候选 (a)=A1 副本**（产物 `e158-root-choice-repair-2026-09-24-q1-g0-a1-arm.out`，
**副本上的数**，只在 `/tmp/claude-1000/e158-s3/arms/a1/` 编出来、`replay.sh` 不覆盖它，见「复跑」）：

```text
产物行见 git show 5d0b7e7c1667:research/results/e158-root-choice-repair-2026-09-24-q1-g0-a1-arm.out 第 48 行（name=q1_1a_trigger_summary）
产物行见 git show 5d0b7e7c1667:research/results/e158-root-choice-repair-2026-09-24-q1-g0-a1-arm.out 第 55 行（name=q1_1c_error_member）
产物行见 git show 5d0b7e7c1667:research/results/e158-root-choice-repair-2026-09-24-q1-g0-a1-arm.out 第 63 行（name=q1_4_a_rejected_device_bytes）
```

**读法**：

- **Q1-1a（N_trig）**：`abandoned_roots_unreadable > 0` 只在「两份都读失败」（`both_copies`，2 个故障）时
  触发（399+399=798 对），只坏一份（`disk0_only`/`disk1_only`，1 个故障）**从不**触发——RAID 镜像的另一份
  顶住了；`empty_baseline`（不注入任何故障，36 个节点各跑一次）**0 次**自然触发，H1 在 n1≤3、m≤2 的规模下
  没有绕环巧合。**N_trig=798>0**：行 1 翻面观测前半成立的方向（(a) 与 (c) 的被拒比例不同）在这批数据上是
  真的——今天那一臂全部 798 个 (Ok, count>0) 对，A1 副本在**同样的 798 对**上全部改判
  `AbandonedRootLedgerUnreadable`（error_member 计数 798 与 (c) 的 trigger_count 798 逐一对应），
  其余 3144 对两臂结局相同（Ok 或同一种候选集拒绝）。
- **Q1-1b（三种被拒比例）**：(c)=今天 N_err=0（这批构造出的 3942 对里没有一个真的入口报错到「今天算法本身
  的缺口」——`Recovery`/`RollbackTargetNotACandidate` 都是候选集本身排除、不是账读不出）；被拒比例
  = 798/3942 ≈ **20.2%**。(a)=A1：被拒比例同样 798/3942（这批数据下 (a) 与 (c) 的「触发即拒绝」严格对应，
  没有观测到 (a) 拒绝了 (c) 没触发的对，也没有 (c) 触发了 (a) 没拒绝的对）。(b) 分配记录树那一档见 Q1-2。
- **Q1-1c（今天报错数分类）**：`Recovery`（42）、`RollbackTargetNotACandidate`（1296）——**F1 不触发**：
  798 个「被抛弃根的单元两份都读失败」的对里，(c) 一次都没有报 `Err`，全部 `Ok(count>0)`，与 C393（被抛弃根的账读不出时修复没有条款） 正文
  「今天只计数、不拒绝挂载」逐条相符（这批数据坐实这句话，不是推断）。
- **Q1-4（(a) 被拒那次写几个字节）**：**798/798 = 100%** 的 `AbandonedRootLedgerUnreadable` 拒绝，设备一层
  写字节都是 **0**（`zero_byte_rejections=798 nonzero_byte_rejections=0`）——「拒绝在任何写之前」这句在
  A1 副本上全量成立，不只是 PC1-a 那一个例子。
- **Q1-2（候选 (b) 分配记录树指称）**：`can_recompute=141`、`cannot_recompute=57`（只统计 op1=mount_writable
  那一档，与 Q1-2 定义一致）——**这些成功大多来自单盘故障**（`disk0_only`/`disk1_only`：走树能用另一份镜像，
  与 Q1-1a 的「1 个故障从不触发」是同一个机制的另一面）；已释放记录条数 `zero=27 nonzero=114`——大多数
  被测的分配记录树节点里混着已释放记录，**只有 27/168 能「按字节原样写回」**（有已释放记录的那 114 个，
  按字节原样重写会把已释放记录也当现行数据写回，候选 (b) 在这些情形下的「就地重建」定义本身要处理已释放
  记录，不只是原样抄一份）。
- **Q1-2（候选 (b) 树表指称）**：`has_copy=9 no_copy=57`——多数被抛弃根的树表节点在孪生镜像别处**没有**
  字节相同的副本（COW 每次发布都重写树表，节点内容随之变化），「有副本」只在少数情形（约 14%）成立；
  「crates 有路」（调 `rebuild_version`/`replay_journal`）**这一段没有测**，`no_copy` 不能读成「无来源」，
  只能读成「只测过的这一支没有」。
- **Q1-3（(b) 的持久化开销）**：两个指称在这批数据里量出的取值集合都是 **{32768}**（一个 16384 字节节点
  × 2 份镜像）——**跑前登记「两个指称的取值集合不同 ⇒ 成立」这一格翻面条件在这个粒度的测法下不成立**：
  树表节点与分配记录树节点在这批被抛弃根上都恰好是单节点（崩溃点重放），开销算式给出同一个数，不是两个指称的
  持久化开销真的相等——粒度更细（比如分配记录树多层、树表跨页）时会不同，这批 H1 历史都没有走到那种规模。
- **PC1-a**：`overwrites_before_rollback=1 rollback_target=(0,0) overwrites_after_rollback=0 abandoned_root=(1,3)`，
  `roots_through_unit=1`。(c)=今天 `Ok(count=1)`；
  (a)=A1 `AbandonedRootLedgerUnreadable{count:1}`、`device_write_bytes=0`——**两条断言都通过**。
  (b) 分配记录树：`outcome=cannot_recompute`——**跑前登记 PC1-a 原文「(b) 分配记录树：装置走树必须成功」
  在这个操作化下没有复现**：候选 (b) 的「就地重建」在这一段被操作化成「用同一组注入目标重新调
  `allocation_records_under_root`」，而这个函数内部本来就要重读**同一个**被注入的节点——两份都读失败时
  它当然也读不出，逻辑上不可能在这个操作化下成功。**这不是这一段算错了 Q1-2 的数**（Q1-2 的
  `can_recompute`/`cannot_recompute` 分类如实反映了这个操作化的行为），**是候选 (b) 支持者原意很可能指
  另一种算法**——从树表出发走它引用的每一棵内容树、按单元指针反推占用集合，完全不重读被注入的那个节点
  本身（跑前登记 5.2 候选 (b) 原文「把它能走到的每棵树的每个单元指针走一遍」）——那种算法要新写一整套
  内容树遍历（inode / 中央映射树 / extent 指针），这一段没有实现。按 `evidence-discipline.md`
  「阳性对照没有过 ⇒ 作废」（跑前登记第十一节 V1），**PC1-a 只按 (a)(c) 两条通过计**，(b) 那一句的
  操作化对不对交主 agent 判断。
- **PC1-b**：同一段历史，改打树表指称：(c) `Ok(count=1)`、(a) `AbandonedRootLedgerUnreadable`——与 PC1-a
  一致；候选 (b) 分配记录树在树表被注入时会不会失败（附带检查）：`walk_fails=true`（这个被抛弃根的树表
  非 0 条，分配记录树指针本身住在树表节点里，树表读不出连指针都拿不到，分配记录树自然也走不通——这是
  跑前登记原文没有单独问的一个附带观测，供参考）；候选 (b) 树表「有副本」：`has_copy_elsewhere=false`。

**（续，session s4）：岔路单第 2 行 ①②——H2 主族穷举下界 + Q2-2a 有真数了**

写了 Q2-1 的新装置代码（`crates/singlefs-implementation-experiments/src/bin/e158_root_choice_repair/main.rs` 新增约 550 行）：
Φ2 证据项枚举（`evidence_items_at`：根环里每个存着自证根的槽，权重 1；journal 环里每一条至少一份
自证的记录，权重 = 自证的份数）、权重从小到大的穷举下界搜索（`candidate_fault_sets_of_weight` +
`search_minimum_weight_that_triggers_rootback`，根槽权重 1、记录权重 2，逐权重把两边组合叉乘）、
H2 主族（`run_rootback_tolerance_family`：mkfs → 可写挂载 → 首个文件 → n1 次覆盖写 → 关闭 →
op2=`mount_writable`（受注入）→ 成功则 n2 次覆盖写 → 关闭 → 判 P331（打中判据） ）、P331（打中判据） 两处判定
（`judge_recovery_outcome` + `evaluate_rootback_predicate`：冷启动 recover 与再做一次不注入的
`mount_writable` 之后冷启动 recover）。另写了 Q2-2a（`run_fixed_publication_byte_script`）：跑前登记
5.8 表的固定脚本（mkfs → 可写挂载 → 首个文件 → 3 次覆盖写 → 关闭 → 可写挂载 → 2 次覆盖写 → 关闭 →
回退到首个文件那条根 → 1 次覆盖写），逐次发布读 `TransactionOutput.writes`/
`VersionWithoutFilePublishOutput.writes`（`WritesByStructureKind`）按结构种类报字节。

**过程中发现并修了一个真 bug（产物之前）**：P331（打中判据） 检查②最初直接套用 `judge_recovery_outcome` 判读回的
所选根「在不在时间线上」，而检查②本身先做了一次**不注入**的 `mount_writable`（推进了状态、新实例的
写行/暖机已经落盘），它的读回所选根天然是这个新实例的根，不在装置只记到 op2 探针为止的
`node.timeline` 里——套用「倒挂」半句会把这次正常前进的 mount_writable 误判成打中，导致 G0（主几何点） 上任何一个
模板在权重 0（零故障）时就报「打中」。改成检查②只比内容号，不再重套「倒挂」判定（跑前登记 5.0 原文
「而它之后冷启动读回的不是当前末端的内容号，记读不回」本来就只问内容，不问根的时间线归属）。这不改变
任何臂或判据——判的还是 P331（打中判据） 定义的那两处，只是修好了检查②本身的实现。

候选 (c)=今天=丙=甲-jsn 与候选甲-txg 两臂都跑了同一份搜索（Φ2 只含根槽 + 记录两类，不含系统配置槽：
这两条臂的 `first_txg_of_new_instance` 都不读系统配置，跳过这一类证据不改变这两条臂的搜索结果）：

```text
E7RESULT name=q2_1_minimum_weight geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) overwrites_before_the_rootback_probe=0 overwrites_after_the_rootback_probe=1 k_min=none_found_within_full_evidence_space maximum_weight_available=10 subsets_tried=128 rootback_probe_failures=0 overwrite_failures=0
E7RESULT name=q2_1_minimum_weight geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) overwrites_before_the_rootback_probe=1 overwrites_after_the_rootback_probe=1 k_min=none_found_within_full_evidence_space maximum_weight_available=13 subsets_tried=512 rootback_probe_failures=0 overwrite_failures=0
E7RESULT name=q2_1_minimum_weight geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) overwrites_before_the_rootback_probe=2 overwrites_after_the_rootback_probe=1 k_min=none_found_within_full_evidence_space maximum_weight_available=16 subsets_tried=2048 rootback_probe_failures=0 overwrite_failures=0
E7RESULT name=q2_1_minimum_weight geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) overwrites_before_the_rootback_probe=3 overwrites_after_the_rootback_probe=1 k_min=none_found_within_full_evidence_space maximum_weight_available=19 subsets_tried=8192 rootback_probe_failures=0 overwrite_failures=0
```

（`research/results/e158-root-choice-repair-2026-09-24-q2-1-g0-today.out`；甲-txg 臂的同一份产物
`e158-root-choice-repair-2026-09-24-q2-1-g0-jia-txg-arm.out` 与它 `diff` 逐字节相同。）

**读法**：n1∈{0,1,2,3}、n2=1、op2=`mount_writable` 这个缩小范围内，**穷举已经走到了这个模板 Φ2
的全部权重**（`subsets_tried` 精确等于该权重下枚举出的全部子集数，没有被 `SUBSET_ENUMERATION_CAP`
截断），**一次都没有打中**——哪怕把根槽 + 记录能藏的全部藏起来（n1=3 时权重上限 19），今天那一臂与
甲-txg 都不会把新实例的写行发布落在一个会被之后择新覆盖的旧位置上。**这与判决 K2 表「今天推的是……丙
12 个」的方向不一致**：19 ≥ 12，而权重 19 已经穷尽了 n1=3 这个模板的全部证据空间还是没打中，说明「丙
12」这个数（或它所在的具体构造）在**这个缩小的评测范围**里复现不出来——按跑前登记 F4「岔路单里推的数
……与跑出来的不同」的触发观测（「今天那一臂（=丙）在崩溃 0 档的区间不含 12」），这一格判据**触发**了，
但按 F4 的处置步骤，触发之后要先让主 agent 对照攻方腿原文核实这里的 n1≤3、op2=`mount_writable`、
Φ2 缺系统配置槽是不是与判决 K2 推「12」时用的同一个构造——**不是同一个构造几乎可以肯定**（K2 的原始
推导细节这一段没有重新读一遍），所以更准确的说法是「这一段缩小范围的搜索没有复现『12』，不能判定
『12』本身错了」，交主 agent 核对判决 K2 原文之后再定是记「推的数不成立」还是「臂不同，推的数无可比」。
两条臂在这个缩小范围内**逐字节相同**：甲-txg（只看根环 txg）与丙（根环 txg 与记录 txg 取 max）在
n1≤3 时从未出现「记录读不出让择新结果不同」的一格——也符合直觉：记录的 txg 不会超过它所属发布的根的
txg，n1 小、根环没有绕环巧合时，两种口径能看到的「最大 txg」应当一致，需要绕环或记录被择新单独依赖
的那种情形（n1 更大、或跨过环容量）才会分化，这一段的 n1≤3 还没有走到那种规模。

**Q2-2a**（固定脚本，`e158-root-choice-repair-2026-09-24-q2-2a-g0-today.out`，
124 行）：`TransactionOutput.writes`/`VersionWithoutFilePublishOutput.writes` 的口径已经是「一次写
调用 = 交给一块盘的一次写，镜像的两份各算一次」（`write_accounting.rs` 顶部注释），与跑前登记「每块盘
收到的写的长度之和」是同一个数，不必另建按 (设备, 偏移) 分类的字节计数器。逐次发布（写行、暖机、
覆盖写、回退写行）按 12 种 `WrittenStructureKind` 分别报字节；甲-txg 臂的同一份产物
（`e158-root-choice-repair-2026-09-24-q2-2a-g0-jia-txg-arm.out`）与今天逐字节
**完全相同**——甲-txg 只改变 `first_txg_of_new_instance` 选中**哪个 txg 值**，不改变任何一次发布要
写哪些结构、写多少字节，这个结果与该臂定义本身直接推得出，是同义反复式的确认（跑前登记 Q2-2b 那一格
「只报数、不作判据」的理由同理适用于这里）：**Q2-2a 在甲-txg 与丙之间的 Δ = 0**（每一种结构、每一种
发布种类都是 0），不支持「行 2 翻面观测后半（每次发布开销不同）」在这一对臂上成立。

**验证**：新增 6 条单测（`candidate_fault_sets_of_weight_mixes_root_slots_and_journal_records_by_weight`、
`candidate_fault_sets_of_weight_at_zero_is_only_the_empty_set`、`judge_recovery_outcome_flags_a_root_off_the_timeline_as_rootback_before_checking_content`、
`judge_recovery_outcome_flags_wrong_content_as_unreadable`、
`judge_recovery_outcome_passes_when_root_on_timeline_and_content_matches`、
`judge_recovery_outcome_passes_on_no_file_when_no_file_is_expected`，22 条全绿）；3 条变异
（`crates/mutations.tsv` 累计 12 条 E158（择根与修复四岔路） 行，逐条手工改坏验证会红、还原后 `diff` 逐字节一致、
门禁 code-source-discipline 的 mutation-tables 格核过原文各命中源码一次）；`cargo fmt`/`clippy -D warnings`/`naming-lint.sh` 对这份 bin
清零。甲-txg 臂：草稿目录 `/tmp/claude-1000/e158-s4/arms/jia-txg/`（`cp -a` 整棵工作树的
`Cargo.toml`/`Cargo.lock`/`.cargo`/`crates`，`SNAPSHOT.txt` 记快照时刻、`git rev-parse HEAD`、
`git status --porcelain crates/` 行数、`crates/*/src/**/*.rs` 的 sha256 汇总），改动记进
`scripts/mutation/tables/e158_arms.tsv`（只改 `crates/singlefs-core/src/mount.rs` 的
`first_txg_of_new_instance` 一处：只取根环 txg，`records` 参数改名 `_records` 保留不读）。

**（续，session s5）：PC2 阳性对照——查出装置故障装配 bug，修好之后甲-txg 的 k_min 与判决 K2 一致**

**派发要求「先做完才往下」的一件**：复现判决 K2（`m2-rootchoice-repair-r1-main-verification.md` 第
47 行）已经判过的两个具体构造——甲「计数器从全环最大 + 1 起」4 个瞬时根槽读失败打中；乙「系统配置带
txg」0 个注入故障、1 个崩溃点打中——不靠穷举搜索，作为「装置看得见判决说存在的打中」的独立验证。

**臂对齐**：K2 表「甲」逐字对应本登记「甲-txg」（`highest_root_txg`，D16（发布语义） 已定项 6 读法），**不是**
5.6 项 2 登记的「甲-jsn」（今天的 `next_counter`）——现查 `mount.rs`：`next_counter` 只喂给下一条
journal 记录的 jsn 分配，`first_txg_of_new_instance`（选根 txg 的唯一入口）从未读它，两个量完全不
相交。详细依据见跑前登记「十二、修订」session s5 条目第 1 条。

**第一次直接测「甲-txg，4 个瞬时根槽读失败」，hit=false**——查出根因：`FaultInjectingBlockDevice` 一
块盘只嵌套一层、一个 `SharedFaultPlan` 一次只装一条计划，`attempt_rootback_probe_and_advance`
（H2 全家族的穷举搜索都靠它装故障）原来用 `.find()` 只挑每块盘上第一个匹配的故障目标去装，只有 2 块
盘时「至少两个目标落在同一块盘」按鸽笼原理在权重 ≥ 2 就可能撞上——**这条 bug 影响 H2 全家族至今用
这套机制跑出的每一个「0 命中」**，包括 session s3/s4 入库的 `q2-1-g0-today.out` 与
`q2-1-g0-jia-txg-arm.out`。修法：新写 `MultiOffsetReadFailingBlockDevice`（本地包装，一次记住一块盘
上全部该装的精确偏移，不改 `fault_injection.rs` 共用基础设施），`attempt_rootback_probe_and_advance`
与新写的 `attempt_a_faulted_mount_writable` 都改用它。

**修好之后，不需要新机制，既有的穷举搜索直接找到 k_min=4**（与专用构造 `attempt_a_faulted_mount_
writable_and_append` 互证；这条专用构造最初是因为怀疑 `effective_root` 截断本身测不出 H-A 才写的，修好根因
一之后证明这层担心不成立——`effective_root` 的位置本来就在截断前的时间线末尾，截断是空操作，见跑前
登记「十二、修订」session s5 条目第 3 条）：

```text
产物行见 git show 5d0b7e7c1667:research/results/e158-root-choice-repair-2026-09-24-pc2-root-ring-txg-only.out 第 46 行（name=q2_1_pc2_root_ring_txg_only）
产物行见 git show 5d0b7e7c1667:research/results/e158-root-choice-repair-2026-09-24-q2-1-g0-root-ring-txg-only-session-s5.out 第 49 行（name=q2_1_minimum_weight）
```
（`e158-root-choice-repair-2026-09-24-pc2-root-ring-txg-only.out`、
`e158-root-choice-repair-2026-09-24-q2-1-g0-root-ring-txg-only-session-s5.out`）

**乙-只配置候选新建了副本、复现 K2「0 个故障、1 个崩溃点」**：新增系统配置字段 `published_txg`
（481→489），`first_txg_of_new_instance` 只信这个字段（`crates/` 现存字段表比对因此要按臂现算，
`local_system_configuration_bytes()` 读环境变量、没设时取 481）。崩溃点用 `acquire_instance`（公开
函数，取号本身只做「写系统配置槽 + 一道屏障」两步，中途无其他写）单独调用后立即收工来还原，不需要
崩溃点重放的崩溃点枚举器：

```text
产物行见 git show 5d0b7e7c1667:research/results/e158-root-choice-repair-2026-09-24-pc2-configuration-published-txg.out 第 47 行（name=q2_1_pc2_configuration_published_txg）
```
（`e158-root-choice-repair-2026-09-24-pc2-configuration-published-txg.out`，跑在
副本上，环境变量 `SINGLEFS_E158_LOCAL_SYSTEM_CONFIGURATION_BYTES=489`）

四组负对照（今天/甲-txg 副本上乙-候选 hit=false；今天副本上甲-候选 hit=false）与两组正对照
（`q2_1_pc2_root_ring_txg_only` hit=true、`q2_1_pc2_configuration_published_txg` hit=true，两条
`E7RESULT` 原样已引在这一节前两段），逐条落在三份产物里：
`research/results/e158-root-choice-repair-2026-09-24-pc2-today.out`（今天，两条候选都
`hit=false`）、`e158-root-choice-repair-2026-09-24-pc2-root-ring-txg-only.out`、
`e158-root-choice-repair-2026-09-24-pc2-configuration-published-txg.out`。

**H2 主族按登记 5.1 补齐 n1∈{0,...,6}**（此前只到 3）：今天/丙在整个范围内仍是穷举完整个可枚举证据
空间都没打中（n1≤3）或撞上 `SUBSET_ENUMERATION_CAP=20000` 提前撞顶（n1∈{4,5,6}，`k_min=capped`）；
**这不构成「丙挡得住」的确认**——判决 K2 给丙的具体历史 H-C1 需要 12 个特定故障（4 根槽 + 4 条记录
各在两块盘上都读不出），这一段没有像甲、乙那样为它单独写一个直接构造，`capped` 只说明穷举没走到那
么远。甲-txg 在 n1=3/4/5/6 各自给出 `k_min=4/4/5/4`（一次 `k_min=5`），与判决 K2「甲 4 个」一致
（`research/results/e158-root-choice-repair-2026-09-24-q2-1-g0-today-session-s5.out`、
`...-root-ring-txg-only-session-s5.out`）。

**验证**：新增 6 条单测（`parse_local_system_configuration_bytes_*` 三条、
`multi_offset_read_failing_block_device_*` 两条、`root_ring_slot_targets_for_finds_exactly_the_
named_roots`，28 条全绿）；3 条变异（`crates/mutations.tsv` 累计 15 条 E158（择根与修复四岔路） 行，逐条手工改坏验证会
红、还原后 `diff` 逐字节一致、门禁 code-source-discipline 的 mutation-tables 格核过原文各命中源码一次）；`cargo fmt`/`clippy -D
warnings`/`naming-lint.sh` 清零；`replay.sh E158`（择根与修复四岔路） 开工与交回前各跑一次。乙-只配置臂：草稿目录
`/tmp/claude-1000/e158-s5/arms/configuration-published-txg/`（同 s4 的整棵工作树 `cp -a` 手法），
改动记进 `scripts/mutation/tables/e158_arms.tsv`（15 行，跨 `singlefs-format/src/lib.rs`、
`singlefs-core/src/{system_configuration,transaction,make_filesystem,mount}.rs`）。

**没做的**（详见「它答不了的」）：乙-留环/丁-留环/丁-只配置三份副本；H2-R、H2c 两个变体；n2=2；
Φ2 系统配置槽证据类（对已跑的两臂不改变结果）；H-C1（丙）的直接构造；Q2-2b。

**（续，session s6）：H-C1 直接构造坐实丙恰好需要 12 个故障；第一行几何敏感性；第二行 ③ 算式**

**开工先做的四处 clippy（session s4/s5 遗留，不是本段新写的代码）**：`items after a test module`
（`mod tests` 挪到文件末尾，纯粹的代码块重排——写脚本切出前后两段互换顺序，用 sha256 核对「把重排
撤销之后与改前的文件逐字节相同」，核对通过：`7a2a46dde4bceb4745dac5bfc7a7245de96708496cc0f67c
91e2440eb8d88df7`，与 session s5 报告记录的改前哈希一致）；`judge_recovery_outcome`/
`evaluate_rootback_predicate` 两处通配臂改成穷举 `RecoveryOutcome` 三个具名变体；
`run_fixed_publication_byte_script` 两处 `shadow_unrelated` 改名（`mounted`→`rollback_mount_
output`、`allocator`→`rollback_allocator`）。四处改完 `cargo clippy --all-targets --all-features`
加共用约束那批 lint 对这份 bin 清零；22 条既有单测保持全绿。

**主工作区在本段期间被合并进「里程碑二收尾」提交系列**（开工 `HEAD`=`3b60f098e97dc4c4f3ed9c6
355422b607db1c34c`、`git status --porcelain crates/` 95 行；中途变成 `HEAD`=`e980a219f1834c638
cd2fae18b25a60525a6bd52`，`crates/*/src/**/*.rs` 净改动 5139 行新增/1185 行删除）。S1 逐一现查本
登记依赖的每个函数与常量（`first_txg_of_new_instance`、两种挂载的 `next_counter`、`choose_root`
比较键、`isolate_slots_referenced_only_by_abandoned_roots` 只计数不拒绝、`SystemRuntimeQuantities`
三字段、五个格式常量）**全部与第三节写的相符，S1 不触发**——这批改动是别处的新增（如
`crates/singlefs-core/src/admission.rs`），不影响 E158（择根与修复四岔路） 已测的任何观测面；装置在新 `HEAD` 上编译、
既有单测全绿。

**H-C1 直接构造（岔路单第 2 行 ①，丙的具体历史，判决 K2「丙……要 12 个」）**：新写
`run_direct_construction_for_the_record_scan_watermark_candidate`（与甲-txg 同一段历史 n1=4，
精确点名顶上 4 条根槽 + `evidence_items_at` 里计数器最大的 4 条记录、两份镜像的 (设备, 偏移) 全拦，
总权重 4×1+4×2=12）与负对照（只挡 4 条根槽，不挡记录）：

```text
E7RESULT name=q2_1_h_c1_record_scan_watermark geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) candidate=record_scan_watermark overwrites_before_the_rootback_probe=4 overwrites_after_the_rootback_probe=1 topmost_four={(1, 4), (1, 5), (1, 6), (1, 7)} record_labels=["record(instance=1,counter=7)", "record(instance=1,counter=6)", "record(instance=1,counter=5)", "record(instance=1,counter=4)"] root_target_count=4 record_target_count=8 fault_target_count=12 fault_weight=12 expected_by_k2_table_fault_count=12 hit=true expected_by_k2_table_hit=true first_check="cold_recover: 读不回 outcome=FileRead { root: (InstanceGeneration(1), CheckpointTxg(7)), content: [150, 30, 161, 41, 172, 52, 183, 63, 194, 74, 205, 85, 216, 96, 227, 107, 238, 118, 249, 129, 9, 140, 20, 151, 31, 162, 42, 173, 53, 184, 64, 195, 75, 206, 86, 217, 97, 228, 108, 239, 119, 250, 130, 10, 141, 21, 152, 32, 163, 43, 174, 54, 185, 65, 196, 76, 207, 87, 218, 98, 229, 109, 240, 120, 0, 131, 11, 142, 22, 153, 33, 164, 44, 175, 55, 186, 66, 197, 77, 208, 88, 219, 99, 230, 110, 241, 121, 1, 132, 12, 143, 23, 154, 34, 165, 45] } expected=Some([96, 227, 107, 238, 118, 249, 129, 9, 140, 20, 151, 31, 162, 42, 173, 53, 184, 64, 195, 75, 206, 86, 217, 97, 228, 108, 239, 119, 250, 130, 10, 141, 21, 152, 32, 163, 43, 174, 54, 185, 65, 196, 76, 207, 87, 218, 98, 229, 109, 240, 120, 0, 131, 11, 142, 22, 153, 33, 164, 44, 175, 55, 186, 66, 197, 77, 208, 88, 219, 99, 230, 110, 241, 121, 1, 132, 12, 143, 23, 154, 34, 165, 45, 176, 56, 187, 67, 198, 78, 209, 89, 220, 100, 231, 111, 242])" second_check="unfaulted_mount_writable_then_recover: 读不回 outcome=FileRead { root: (InstanceGeneration(3), CheckpointTxg(10)), content: [150, 30, 161, 41, 172, 52, 183, 63, 194, 74, 205, 85, 216, 96, 227, 107, 238, 118, 249, 129, 9, 140, 20, 151, 31, 162, 42, 173, 53, 184, 64, 195, 75, 206, 86, 217, 97, 228, 108, 239, 119, 250, 130, 10, 141, 21, 152, 32, 163, 43, 174, 54, 185, 65, 196, 76, 207, 87, 218, 98, 229, 109, 240, 120, 0, 131, 11, 142, 22, 153, 33, 164, 44, 175, 55, 186, 66, 197, 77, 208, 88, 219, 99, 230, 110, 241, 121, 1, 132, 12, 143, 23, 154, 34, 165, 45] } expected=Some([96, 227, 107, 238, 118, 249, 129, 9, 140, 20, 151, 31, 162, 42, 173, 53, 184, 64, 195, 75, 206, 86, 217, 97, 228, 108, 239, 119, 250, 130, 10, 141, 21, 152, 32, 163, 43, 174, 54, 185, 65, 196, 76, 207, 87, 218, 98, 229, 109, 240, 120, 0, 131, 11, 142, 22, 153, 33, 164, 44, 175, 55, 186, 66, 197, 77, 208, 88, 219, 99, 230, 110, 241, 121, 1, 132, 12, 143, 23, 154, 34, 165, 45, 176, 56, 187, 67, 198, 78, 209, 89, 220, 100, 231, 111, 242])"
E7RESULT name=q2_1_h_c1_negative_control_root_slots_only geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) candidate=record_scan_watermark overwrites_before_the_rootback_probe=4 overwrites_after_the_rootback_probe=1 topmost_four={(1, 4), (1, 5), (1, 6), (1, 7)} fault_target_count=4 expected_by_k2_table_fault_count=4 hit=false expected_hit=false first_check="ok" second_check="ok"
```

（`research/results/e158-root-choice-repair-2026-09-24-q2-1-hc1.out`；session s8 门禁 doc-experiments 的 quoted-result-lines 格发现
上一行曾被摘成 `content=[...]`，现按产物整行补全，不再摘录）**直接构造 `hit=true`（4 条根槽 + 4 条
记录都拦下，权重 12），负对照 `hit=false`（只拦根槽挡不住丙）**——与 K2「丙……要 12 个」一致。

**下界探针坐实「恰好 12」，不只是上界**：新写 `run_record_scan_watermark_lower_bound_probe`，复用
既有的 `search_minimum_weight_that_triggers_rootback`，只把它已有的 `subset_enumeration_cap` 参数
从 `SUBSET_ENUMERATION_CAP`=20000（session s5 在这个 n1 上撞了顶）单独调高到 60000（**session s9
把 `subset_enumeration_cap` 参数改名为 `weight_ceiling`、并把「数够就停」的截断机制换成按权重分级
截断，详见 `.claude/kb/experiments/158-择根与修复四岔路.md`「2026-09-24（续，session s9）」一节；
`q2_1_h_c1_lower_bound_probe` 这一行结果是按新字段名重跑之后的产物，`k_min`/`subsets_tried` 等结
论数字与当年逐字相同，只是多了 `full_space_subset_count`）：

```text
E7RESULT name=q2_1_h_c1_lower_bound_probe geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) overwrites_before_the_rootback_probe=4 weight_ceiling=None k_min=12 hit_count_at_that_weight=1 first_hit_combination=["root(region=0,slot=2)", "root(region=1,slot=1)", "root(region=1,slot=2)", "root(region=2,slot=1)", "record(instance=1,counter=4)", "record(instance=1,counter=5)", "record(instance=1,counter=6)", "record(instance=1,counter=7)"] subsets_tried=22558 full_space_subset_count=32768 rootback_probe_failures=29 overwrite_failures=153
```

（`research/results/e158-root-choice-repair-2026-09-24-q2-1-hc1-lower-bound.out`）`subsets_tried=
22558 < 60000`——**没有撞顶**，权重 0 到 11 的全部组合穷举完都没有命中，权重 12 上第一个尝试的组合
（4 根槽 + 4 条记录，与直接构造用的是不同的具体根槽、同样是计数器 4/5/6/7 那 4 条记录）就命中。
按 `.claude/rules/mutation-sampling.md`「构造上界给出的数，穷举在它之下全部权重都没打中才写成
『恰好 k』」，**丙恰好需要 12 个故障，这是穷举下界给出的严格证据，不是撞顶推出的上界**。

⚠️ **判决 K2 原表把丙这一格同时标「攻不动（结构性：每次发布必写记录、记录先于根 ⇒ 水位单调不减）」
与「要 12 个」，这两句字面有张力**：「攻不动」读作「不可能被打中」，「要 12 个」给出一个具体的打中数。这一段两条
独立路径（手写构造、独立穷举）都在权重 12 上给出 `hit=true`——**丙确实可以被打中，且恰好需要 12**，
与后一句吻合、与前一句的字面读法（「不可能」）不吻合。这句话该怎么读（也许「攻不动」原意专指「只靠
压根环这一种简单攻法」）交主 agent 核对攻方腿原文判断，本段不越权下结论。**这也更新了 session s4
的一句旧判断**：session s4 曾报告「今天那一臂（=丙）在崩溃 0 档的区间不含 12」、按 F4 判「触发」
（见「结果整行抄自产物」573 行前后）；那时的搜索范围是 n1≤3（穷尽到 19 都没打中），本段在
n1=4 上用直接构造 + 60000 上限的穷举证明「不含 12」是搜索范围不够大，不是丙真的挡得住——**F4 在丙
这一格上，按这一段的读数应改判「不触发」（与推的数一致）**，具体归类交主 agent 定。

**第八节几何敏感性（岔路单第 1 行判决格 = Q1-1a 的 N_trig）**：新增 `q1-s16`、`q1-s4` 两个模式，
复用既有的 `run_ledger_fault_family`（本就是几何参数化的函数，未改代码）：

```text
产物行见 git show 5d0b7e7c1667:research/results/e158-root-choice-repair-2026-09-24-q1-s16.out 第 56 行（name=q1_1a_trigger_summary）
产物行见 git show 5d0b7e7c1667:research/results/e158-root-choice-repair-2026-09-24-q1-s4.out 第 56 行（name=q1_1a_trigger_summary）
```

（`e158-root-choice-repair-2026-09-24-q1-s16.out`、
`e158-root-choice-repair-2026-09-24-q1-s4.out`；两点与 G0（主几何点）
的 `pairs=3942 trigger_count=798` 逐字节相同，`q1_1a_by_aspect_severity`/`q1_1c_error_member`
各分项计数也一致）**判定不翻面**。判别力自证：两点同值，门槛无处可挪，按 8.2 原文记「两点同值，
自证不适用」。

**岔路单第 2 行 ③ 的字段表宽度算式（甲/丙、乙/丁两组，对候选 2 的两种宽度与两种表形态）**：在
`run_constants_and_anchors` 里补 8 条算术断言（候选 2 表形态 `1+16*N_w`/`1+12*N_w`，N_w=4 取自
Q3-2 已量出的最大值，算出 65/49；甲、丙都不改字段表，与候选 2 同放按「单放」公式算：单条 16/12
余量 15/19（放得下），表形态 65/49 余量 −34/−18（推过 512）；乙、丁都把字段表改成 489，按「与乙
同放」公式算：表形态 65/49 余量 −42/−26，推过 512）：

```text
E7RESULT name=section_seven_two_arithmetic item="候选2表形态 1+16*N_w(N_w=4)" verdict=pass computed=65 registered=65
E7RESULT name=section_seven_two_arithmetic item="候选2表形态 1+12*N_w(N_w=4)" verdict=pass computed=49 registered=49
E7RESULT name=section_seven_two_arithmetic item="甲/丙同放候选2单条16 512-481-16" verdict=pass computed=15 registered=15
E7RESULT name=section_seven_two_arithmetic item="甲/丙同放候选2单条12 512-481-12" verdict=pass computed=19 registered=19
E7RESULT name=section_seven_two_arithmetic item="甲/丙同放候选2表形态65 512-481-65" verdict=pass computed=-34 registered=-34
E7RESULT name=section_seven_two_arithmetic item="甲/丙同放候选2表形态49 512-481-49" verdict=pass computed=-18 registered=-18
E7RESULT name=section_seven_two_arithmetic item="乙/丁同放候选2表形态65 512-489-65" verdict=pass computed=-42 registered=-42
E7RESULT name=section_seven_two_arithmetic item="乙/丁同放候选2表形态49 512-489-49" verdict=pass computed=-26 registered=-26
```

（同一份 `research/results/e158-root-choice-repair-2026-09-24-q2-1-hc1.out` 开头的常量回比段；
这 8 条与其余 local_constant_check/section_seven_two_arithmetic 一起在每次运行开跑时都会打印）
**这只是算术，不需要新建 5.3 表里没有的臂**：甲、丙已经「与今天同一份代码」，乙、丁的字段表改动
就是 5.3 表原文写的那 8 字节，数值早已由 Q2-2b/Q3-2 各自算出，这里只是把「与哪个修法同放」的四种
组合摆全。

**验证**：单测仍是 28 条全绿（本段没有新增单测——新增的两个 PC2 风格驱动函数与算术检查复用的都是
已被单测/变异覆盖过的既有函数：`root_ring_slot_targets_for`、`evidence_items_at`、`evaluate_
rootback_predicate`、`search_minimum_weight_that_triggers_rootback`，与既有的
`run_positive_control_for_the_root_ring_txg_only_candidate` 等驱动函数同一惯例——它们的正确性
靠自己的 E7RESULT 输出核对，不是靠专门的 `#[test]`）；1 条变异追加进 `crates/mutations.tsv`
（改候选 2 表形态算术的 `N_w` 从 4 改成 3，手工验证：改坏 → `constants_and_anchors_all_pass` 红
→ 还原 → 复绿），累计 16 条 E158（择根与修复四岔路） 变异行，门禁 code-source-discipline 的 mutation-tables 格核过原文各命中源码一次；`cargo fmt`/
`clippy -D warnings`（含共用约束那批 lint）/`naming-lint.sh` 对这份 bin 清零（`run_h_c1_lower_
bound_probe` 最初撞了单字母 `h` 违规，改名 `run_record_scan_watermark_lower_bound_probe` 后清零）。
新增 `driver_e158_q1_s16`、`driver_e158_q1_s4`、`driver_e158_q2_1_hc1`、
`driver_e158_q2_1_hc1_lower_bound` 登记进 `replay.sh`。

**没做的**（详见「它答不了的」）：乙-留环、丁-留环、丁-只配置三份副本的 ①②（这三个子臂的系统
不配置层面工程量在 session s5 建乙-只配置时已验证过，这一段的时间全给了 H-C1）；H2-R、H2c 两个
变体；n2=2；Φ2 系统配置槽证据类；Q2-2b。**岔路单第 2 行仍不算够判**——丙的 ① 现在是全实验最强的
一格证据，但乙、丁的多数子臂一格数据都没有。

**（续，session s7）：岔路单第 2 行三份新副本（乙-留环/丁-留环/丁-只配置）①②；岔路单第 1 行候选 (b) 两个子分支**

**开工重核（S1）**：`git rev-parse HEAD`=`e980a219f1834c638cd2fae18b25a60525a6bd52`、`git status
--porcelain crates/` 只剩 E158（择根与修复四岔路） 自己的 bin 与 `mutations.tsv`（2 行），与 session s6 收尾时一致；本
登记依赖的函数与常量再次现查全部相符，S1 不触发。

**岔路单第 2 行三份新副本**：`scripts/mutation/tables/e158_arms.tsv` 追加 47 行（乙-留环 15 行、丁-留环
与丁-只配置各 16 行——比乙族多一个 `published_txg_to_carry_over` 方法 + 取号那两处调用点改成续传）。
三份副本 `cp -a` 自同一次快照（`git_head`/`git_status_crates_lines`/`crates_sha256_of_sha256s` 与
开工时一致），各自 `cargo build --release` 编译通过，跑 `q2-1-g0`（Φ2 穷举下界，n1∈{0,...,6}）、
`q2-2a-g0`（固定脚本设备一层写字节）、`q2-1-pc2`（K2 给甲/乙的具体构造）三个模式（均加
`SINGLEFS_E158_LOCAL_SYSTEM_CONFIGURATION_BYTES=489`）。**结果整行抄自产物（产物已移出工作区的行写成一行指针）**：

```
产物行见 git show 5d0b7e7c1667:research/results/e158-root-choice-repair-2026-09-24-q2-1-g0-yi-ring-retained.out 第 54 行（name=q2_1_minimum_weight）
产物行见 git show 5d0b7e7c1667:research/results/e158-root-choice-repair-2026-09-24-q2-1-g0-yi-ring-retained.out 第 58 行（name=q2_1_minimum_weight）
```
（`e158-root-choice-repair-2026-09-24-q2-1-g0-yi-ring-retained.out`、
`e158-root-choice-repair-2026-09-24-q2-1-g0-ding-ring-retained.out`、
`e158-root-choice-repair-2026-09-24-q2-1-g0-ding-only-configuration.out` 三份
逐字节相同；n1∈{1,2,3,5,6} 各档同一种形状，只贴 n1=0 与 n1=4 两档示例。）

```
E7RESULT name=q2_1_pc2_configuration_published_txg geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) candidate=configuration_published_txg overwrites_before_the_crash=2 acquired_instance_before_the_crash=2 injected_faults=0 crash_points=1 hit=false expected_by_k2_table=true first_check="ok" second_check="ok"
```
（`e158-root-choice-repair-2026-09-24-pc2-yi-ring-retained.out`、
`e158-root-choice-repair-2026-09-24-pc2-ding-ring-retained.out`、
`e158-root-choice-repair-2026-09-24-pc2-ding-only-configuration.out` 三份都是
这一行；乙-只配置在同一个构造上是 `hit=true`，见 session s5。）

```
$ diff <(grep 'q2_2a_publish_writes' e158-root-choice-repair-2026-09-24-q2-2a-g0-today.out) <(grep 'q2_2a_publish_writes' e158-root-choice-repair-2026-09-24-q2-2a-g0-yi-ring-retained.out)
（无输出，rc=0；e158-root-choice-repair-2026-09-24-q2-2a-g0-ding-ring-retained.out、
e158-root-choice-repair-2026-09-24-q2-2a-g0-ding-only-configuration.out 两份同样 rc=0）
```

**读法**：①打中最少几个故障——三份副本在 Φ2（根槽+记录两类故障）穷举下 n1≤3 全空、n1∈{4,5,6} 撞顶
未穷尽；K2 给乙的具体构造（崩在取号之后、写行发布之前）在三份上都不打中——乙-留环因为公式仍读根环
（`max(ring, published_txg)`，崩溃没压低根环给出的值），丁两族因为取号续传（不写 0）。本段没有找到
任何打中这三份副本的构造，① 报告为「本段测过的两类攻法（Φ2 穷举、K2 给乙的构造）都没打中」，不是一
个具体的 k_min 数，也不能推广成「挡得住全部攻法」（Φ2 缺系统配置槽这一类证据）。②每次发布多写几字
节——三份副本与今天逐字节相同（Δ=0，设备一层的写按整槽计，加 8 字节字段不改变槽本身的写字节数）。

**岔路单第 1 行候选 (b) 两个子分支**：新写 `nearest_older_readable_root_on_same_instance`
（同一实例上最近的可读祖先）与 `tree_table_crates_has_a_path`（从祖先出发调 `replay_journal`，
`verify_named_units` 两种都跑，比对交回的 `tree_table` 指针）实现「crates 有路」；新写
`allocation_record_tree_reachable_placements_via_central_mapping`（走中央映射树、不重读被注入的
分配记录树节点）实现「从内容树反推占用集合」的一种操作化。**结果整行抄自产物（产物已移出工作区的行写成一行指针）**：

```
产物行见 git show 5d0b7e7c1667:research/results/e158-root-choice-repair-2026-09-24-q1-g0-today.out 第 69 行（name=q1_2_tree_table_crates_has_a_path）
E7RESULT name=q1_2_tree_table_crates_has_a_path geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) verify_named_units=false outcome=no_older_readable_root_on_timeline count=9
产物行见 git show 5d0b7e7c1667:research/results/e158-root-choice-repair-2026-09-24-q1-g0-today.out 第 71 行（name=q1_2_tree_table_crates_has_a_path）
E7RESULT name=q1_2_tree_table_crates_has_a_path geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) verify_named_units=true outcome=did_not_reach_target count=189
E7RESULT name=q1_2_tree_table_crates_has_a_path geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) verify_named_units=true outcome=no_older_readable_root_on_timeline count=9
E7RESULT name=q1_2_allocation_record_tree_from_content_trees geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) outcome=central_mapping_unreadable count=27
E7RESULT name=q1_2_allocation_record_tree_from_content_trees geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) outcome=unequal count=171
```
（`e158-root-choice-repair-2026-09-24-q1-g0-today.out`，覆盖了旧产物、纯增量，
旧的 76 行原样保留，只多了这 7 行诊断与末行 `emitted` 计数 76→83。）

**读法**：树表指称「crates 有路」——按今天挂载真用的口径（`verify_named_units=true`）198 个触发对
里零格给出匹配指针（189 格没推进到目标、9 格没有更旧的可读祖先），说明 `replay_journal` 在真实验证
口径下没有独立于那次被挡住的读的路径（验证点名单元那一步会去碰同一个物理位置）；放宽到不验证
（`verify_named_units=false`）时 81/189（约 43%）能给出匹配指针——journal 记录自带的新根段字段本
身携带了重建根字段所需的信息，只是「验证」这一步会去碰物理位置。这两个数字合起来：候选 (b) 树表指
称「crates 有路」这句话，在「按今天真实验证口径」下不成立；在「只信记录字段、放弃当场验证」这个更
弱的读法下，对将近一半的触发对成立。哪个读法才是候选 (b) 支持者的原意，交主 agent 判。分配记录树指
称「从内容树反推占用集合」——只走中央映射树的操作化，171 个可比对的对里**全部 `unequal`**（27 个
`central_mapping_unreadable` 是树表 0 条那一版 `mapping_root` 恒空指针，正当拿不到，不是读错）。
这不是候选 (b) 这条思路本身不成立，是「只走中央映射树」这一种操作化本身不完整——中央映射树只记「当
前逻辑地址映射到的数据落点」，分配记录树的「未释放」集合还含各棵树自己的结构节点（树表、8 棵子树各
自的根与内部/叶节点、accounting 树），要让这种操作化真正给出匹配的占用集合，还要另外走这些结构树。

两个新函数各配一条单测（`nearest_older_readable_root_on_same_instance_picks_the_closest_
ancestor_on_the_same_instance`、`allocation_record_tree_reachable_placements_via_central_
mapping_reflects_written_content_and_errors_on_an_empty_pointer`），逐条手工改坏验证会红、还原后
复绿；单测总数 28→30。变异表 `crates/mutations.tsv` 追加 2 行（对应这两条新单测），门禁 code-source-discipline 的 mutation-tables 格核过
原文各命中源码一次（536 条）。

**不改判据、不改臂定义**：`scripts/mutation/tables/e158_arms.tsv` 新增的 47 行是 5.3 表已写死的臂定义的
字面实现；新写的三个函数是跑前登记 5.2 表「② crates 有路」与「(b) 分配记录树：…从账里取的那一份」这
两句此前只有文字描述、现在实现成真跑得出结果的代码，定义原文一个字没有改。第五、六、七节的判据、
5.1–5.4 的臂定义、P331（打中判据）/P332（违例判据） 的定义同样一个字没有改。

**没做**：op1 第三种变体（先不注入 `mount_writable` 再 `raise_rollback_floor`）与持续故障（op1 及其
后两次挂载都带着）本段仍未实现；H2-R、H2c、n2=2、Φ2 系统配置槽证据类本段也仍未做，见「它答不了的」。

**（续，session s8）：岔路单第 2 行 Φ2 系统配置槽这一类证据；岔路单第 1 行候选 (b) 分配记录树指称「走全」**

**开工重核（S1）+ 实做才发现的 S4 变体**：`git rev-parse HEAD` 仍是 `e980a219f1834c638cd2fae18b25a60
525a6bd52`；`git status --porcelain crates/` 从 session s7 收尾时的 2 行涨到 31 行——另一条并行改动
线正在改 `mount.rs`/`recovery.rs`/`checker/walk.rs`。逐一现查本登记依赖的每个函数名，命中数全部 0，
S1 不触发。但把新写的装置源码拷进 session s7 已建好的三份旧副本重新编译时**编不过**：`replay_journal`
的返回类型在两个快照之间变了（旧：裸元组；今天：`Result<(...), _>`），session s7 自己新写的
`tree_table_crates_has_a_path` 调它的 `.expect(...)` 因此报错——这不是第三节任何一条（`replay_
journal` 不在依赖清单里，是 session s7 自己引入的依赖），照实记；处置：今天重新 `cp -a` 一份基准
快照（`git_status_crates_lines=31`、`crates_sha256_of_sha256s=fac44f8b508802d112dbdd4f1b826a54379
c87776d2ce43570f03ff78248798a`），从它派生三份新的乙-留环/丁-留环/丁-只配置副本（`e158_arms.tsv`
已有的 47 行原样重新套用，行数与 s7 一致：15/16/16），s7 的旧副本留作对照、不删除。详见跑前登记
「十二、修订」session s8 条目。

**Φ2 系统配置槽这一类证据**：`evidence_items_at` 新增 `include_system_configuration_slots` 开关；
打开时枚举两块盘各自 2 个自证过的系统配置槽（权重 1）。H-C2 直接构造（机制假说：`choose_system_
configuration` 按 `device_identities()` 顺序取第一块有效盘的值直接当 `chosen`，后面的盘只核对
`filesystem_identifier`/`device_count`——单测 `faulting_the_newer_device_zero_system_
configuration_slot_regresses_the_chosen_generation` 不依赖任何臂改法证明了这个假说对「今天」的
`choose_system_configuration` 成立）只把设备 0 上世代号更高的槽读失败（权重 1），加两个负对照。
**结果整行抄自产物（产物已移出工作区的行写成一行指针）**：

```
产物行见 git show 5d0b7e7c1667:research/results/e158-root-choice-repair-2026-09-24-hc2-yi-ring-retained.out 第 54 行（name=q2_1_h_c2_system_configuration_evidence）
产物行见 git show 5d0b7e7c1667:research/results/e158-root-choice-repair-2026-09-24-hc2-yi-ring-retained.out 第 55 行（name=q2_1_h_c2_negative_control_older_slot）
产物行见 git show 5d0b7e7c1667:research/results/e158-root-choice-repair-2026-09-24-hc2-yi-ring-retained.out 第 56 行（name=q2_1_h_c2_negative_control_other_device）
```
（`e158-root-choice-repair-2026-09-24-hc2-yi-ring-retained.out`、
`e158-root-choice-repair-2026-09-24-hc2-ding-ring-retained.out`、
`e158-root-choice-repair-2026-09-24-hc2-ding-only-configuration.out` 三份逐字节
相同）——**三份副本上都 `hit=false`**：单个系统配置槽故障
（哪怕是精确针对机制假说构造的）不足以打中任何一条，与三条修法各自的设计初衷（乙-留环保留根环回
落、丁族取号续传）一致。

**Q2-1 加 Φ2 系统配置槽的穷举（n1 拆两段跑，见修订条目②）**：三份副本结果逐字节相同——n1=0、1 穷举
完整个证据空间都没打中（不是撞顶）；n1=2...6 全部撞 `SUBSET_ENUMERATION_CAP=20000` 上限、没找到
命中。

⚠️ **这一段原本在这里逐行抄的 `E7RESULT` 产物行，session s9 用新的 `weight_ceiling` 机制重跑
`q2-1-g0-configuration-evidence` 时把
`e158-root-choice-repair-2026-09-24-q2-1-g0-configuration-evidence-yi-ring-retained.out`、
`e158-root-choice-repair-2026-09-24-q2-1-g0-configuration-evidence-ding-ring-retained.out`、
`e158-root-choice-repair-2026-09-24-q2-1-g0-configuration-evidence-ding-only-configuration.out`
三份文件**直接覆盖**了，没有先按「对不上就按今天的日期另存一份」的规矩另存旧产物——这是 session s9
的一处操作失误，如实记在这里：本段引用的这几行 `capped` 产物的原始字节今天已经找不到了，只有这段
文字里留着的数字（`subsets_tried=2048/8192/20000/20000/20000/20000`，n1=0..6 对应
`maximum_weight_available=14/17/…`）还能证明当时测过什么、测到了哪一步。**这几行的结论仍然成
立**（session s9 的新机制在这三份副本上重跑出的结果与这里描述的方向一致：n1=0、1 完整穷举不打中，
n1≥2 此前因为撞旧的计数上限没有搜完——新机制补齐了完整搜索，见「它答不了的」岔路 2 那一条的完整
读法与新产物），只是这一处不再贴可复核的原始行。**读法**：H-C2 与穷举是两条独立代码路径（H-C2 用
`attempt_a_faulted_mount_writable_and_append`，
穷举用 `attempt_rootback_probe_and_advance`），两条路径在这三份副本上给出一致的负结果——本段测过的
两类攻法（单点构造 + 权重递增穷举，穷举在 n1≤1 是完整的、n1≥2 只到 20000 子集为止）都没有找到能
打中乙-留环/丁-留环/丁-只配置的系统配置槽相关故障集合。**没有可报的 k（没有上界，因为没有一个能打
中的构造）**，只能报「测过的攻法都没打中」，不能推广成「挡得住全部攻法」。

**候选 (b) 分配记录树指称「走全」**：新增 `allocation_record_tree_reachable_placements_via_all_
structural_trees`，在只走中央映射树的基础上并入树表自己的节点、`TREE_KIND_INODE`/`_EXTENT`/
`_ACCOUNTING` 三种子树各自的根节点、中央映射树自己的节点（`TREE_KIND_ALLOCATION` 不并入，那是要
验证的账本身；`_LIVELIST`/`_SPARSE_SIDE_TABLE`/`_DEADLIST` 三种跑前登记原文没点名，没有并入）。
**结果整行抄自产物**：

```
E7RESULT name=q1_2_allocation_record_tree_from_content_trees geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) outcome=central_mapping_unreadable count=27
E7RESULT name=q1_2_allocation_record_tree_from_content_trees geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) outcome=unequal count=171
E7RESULT name=q1_2_allocation_record_tree_from_all_structural_trees geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) outcome=structural_trees_unreadable count=27
E7RESULT name=q1_2_allocation_record_tree_from_all_structural_trees geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) outcome=subset_of_pristine count=171
```
（`e158-root-choice-repair-2026-09-24-q1-g0-today.out`；S16、S4 两个几何点同样的
171/27 分布，`e158-root-choice-repair-2026-09-24-q1-s16.out`、`...-q1-s4.out`）。
**读法**：走全版本没有让任何一格从「不等」变成「相等」，但把 171 格从「不等，方向不明」精确到了
「`subset_of_pristine`，走全版本给出的集合仍然是真实未释放集合的真子集」——单测证明走全版本是只走
中央映射版本的严格超集（新增了树表与映射树自己的节点），所以这不是走全反而漏了东西，是**走全之后
仍然不够全**：真实的未释放集合里还有走全版本没覆盖到的落点（候选是 `TREE_KIND_LIVELIST`/`_SPARSE_
SIDE_TABLE`/`_DEADLIST`，或者是别的机制，本段没有再往下查）。这不是候选 (b) 这条思路整体不成立，
是这种操作化目前已知的范围（树表 + inode + extent + accounting + 映射树自己）仍不足以覆盖真实账的
全部内容。

**新增单测 5 条**（`system_configuration_slot_positions_uses_slot_spacing_for_the_second_slot_on_
each_device`、`evidence_items_at_includes_system_configuration_slots_only_when_asked`、
`candidate_fault_sets_of_weight_mixes_root_and_configuration_slots_by_weight`、`faulting_the_
newer_device_zero_system_configuration_slot_regresses_the_chosen_generation`、`allocation_
record_tree_reachable_placements_via_all_structural_trees_is_a_superset_of_the_central_mapping_
only_version`），逐条手工改坏验证会红、还原后复绿；单测总数 30→35。变异表 `crates/mutations.tsv`
追加 5 行（3 条给 Φ2、2 条给「走全」），逐条手工验证会红；改名 `candidate_fault_sets_of_weight` 的
第一个参数导致 session s5 写的一条既有变异锚点漂移，已同步改新变量名。

**不改判据、不改臂定义**：`evidence_items_at`/`run_rootback_tolerance_family`/`search_minimum_
weight_that_triggers_rootback` 新增的开关与参数，既有调用点都传关闭/默认值，逐字节复现之前的行为
（`q1-g0`/`q1-s16`/`q1-s4` 三份产物按纯增量覆盖，旧内容原样保留，各多 2 行诊断）；第五、六、七节的
判据、5.1–5.4 的臂定义、P331（打中判据）/P332（违例判据） 的定义一个字没有改。

**没做**：op1 第三种变体与持续故障仍未实现；H2-R、H2c、n2=2、Q2-2b 仍未做；「走全」仍不是穷举（还差
`_LIVELIST`/`_SPARSE_SIDE_TABLE`/`_DEADLIST` 三种、以及为什么仍是 `subset_of_pristine` 的根因）。

**（续，session s9）：171 格差集的内容与根因；op1 起三步挂载的持续/瞬时轨迹；n1=2…6 撤掉穷举的计数上限；实十九多层树防护**

**171 格差集 = 实例表自己的节点，不是 LIVELIST/SPARSE_SIDE_TABLE/DEADLIST**：新增 `classify_diff_
pair`，对每个 `subset_of_pristine` 格算 `pristine − 走全` 差集、逐个落点分类。**结果整行抄自产物**
（`e158-root-choice-repair-2026-09-24-q1-g0-today.out`）：

```text
E7RESULT name=q1_2_subset_diff_pair geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) device=0 slot=50176 label=instance_table_self_node_not_walked_by_the_all_structural_trees_arm occurrences=171
E7RESULT name=q1_2_subset_diff_pair geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) device=1 slot=50176 label=instance_table_self_node_not_walked_by_the_all_structural_trees_arm occurrences=171
E7RESULT name=q1_2_subset_diff_distinct_pairs geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) distinct_pairs=2
```

`(0, 50176)`/`(1, 50176)` 就是 `UNIT_AREA_START_SLOT`=`INSTANCE_TABLE_SLOT`——`RootRecord.
instance_table` 是与 `tree_table`/`mapping_root` 并列的第三条指针，走全版本没读过它。核过
`transaction.rs` 的 `publish_admitted`：`TREE_KIND_LIVELIST`/`_SPARSE_SIDE_TABLE`/`_DEADLIST` 三
条树表条目的根**恒为** `NodePointer::empty_root()`，是全仓每次发布都这样、不是这批历史的特例——差
集与这三种树无关，候选 (b) 走全版本真正的缺口是实例表。

**op1 起三步挂载的持续/瞬时轨迹**（8.1 要
求）：新写 `mount_writable_trajectory`，复用 PC1-a 已选中的落点：

```text
E7RESULT name=q1_1a_op1_trajectory geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) mode=persistent trajectory=[Some(1), Some(1), Some(1)]
E7RESULT name=q1_1a_op1_trajectory geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) mode=transient trajectory=[Some(1), Some(0), Some(0)]
```

与登记原文逐字吻合。**n1=2…6 撤掉了穷举的计数上限**：`search_minimum_weight_that_triggers_
rootback` 的 `subset_enumeration_cap`（会在权重档中途停手）换成 `weight_ceiling`（只在权重档边界
停，恒报 `full_space_subset_count` 与 `highest_weight_examined`）。今天/丙臂
`q2-1-g0`（`research/results/e158-root-choice-repair-2026-09-24-q2-1-g0-today-session-s5.out`）：

```text
E7RESULT name=q2_1_minimum_weight geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) overwrites_before_the_rootback_probe=4 overwrites_after_the_rootback_probe=1 k_min=12 hit_count_at_that_weight=1 first_hit_combination=["root(region=0,slot=2)", "root(region=1,slot=1)", "root(region=1,slot=2)", "root(region=2,slot=1)", "record(instance=1,counter=4)", "record(instance=1,counter=5)", "record(instance=1,counter=6)", "record(instance=1,counter=7)"] subsets_tried=22558 full_space_subset_count=32768 rootback_probe_failures=29 overwrite_failures=153
E7RESULT name=q2_1_minimum_weight geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) overwrites_before_the_rootback_probe=5 overwrites_after_the_rootback_probe=1 k_min=not_found_up_to_weight_ceiling highest_weight_examined=12 maximum_weight_available=25 full_space_subset_count=131072 subsets_tried=65536 rootback_probe_failures=9 overwrite_failures=120
E7RESULT name=q2_1_minimum_weight geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) overwrites_before_the_rootback_probe=6 overwrites_after_the_rootback_probe=1 k_min=12 hit_count_at_that_weight=1 first_hit_combination=["root(region=0,slot=2)", "root(region=0,slot=3)", "root(region=1,slot=2)", "root(region=2,slot=2)", "record(instance=1,counter=6)", "record(instance=1,counter=7)", "record(instance=1,counter=8)", "record(instance=1,counter=9)"] subsets_tried=173716 full_space_subset_count=524288 rootback_probe_failures=10 overwrite_failures=57
```

**读法**：n1=4、6 此前撞旧计数上限报 `capped`，现在主搜索本身（不靠 H-C1 那种额外直接构造）就在
两处独立给出 `k_min=12`，与 session s6 的 H-C1 直接构造互证；n1=5 完整空间 131072，只搜完一半
（65536，因为按权重分档的组合数在这个空间上关于权重 12 对称）就停在权重 12，如实报「未打中」，不
是「挡得住」——`stopped_by_weight_ceiling` 字段与 `full_space_subset_count`/`highest_weight_
examined` 一起报，不再是一个不说明含义的 `capped`。**乙-留环/丁-留环/丁-只配置三份副本本段中途按
主 agent 要求重跑完，不留成欠账，读法见「它答不了的」岔路 2 那一条**。

**实十九多层树防护**：
`allocation_record_tree_reachable_placements_via_central_mapping` 加了 `level != 0` 检查——手工
构造 `level=1` 的假节点证明改动前会**静默解出一个假的落点集合**（不是报错）；实测这批历史
中央映射树根 `entries.len()` 恒为 6、`level` 恒为 0，远低于 294 的单叶容量，暂时不会撞上，但检查
留在代码里。新增单测 4 条（`classify_diff_pair_identifies_the_instance_table_self_node`、
`mount_writable_trajectory_distinguishes_persistent_from_transient_faults`、`search_minimum_
weight_that_triggers_rootback_honors_an_explicit_weight_ceiling`、`allocation_record_tree_
reachable_placements_via_central_mapping_rejects_a_multi_level_root`），单测 35→39；变异表追加 4
行、修了 1 行既有锚点（门禁 code-source-discipline 的 mutation-tables 格红——`classify_diff_pair` 插入位置挤开了原有一条变异的锚点，已改到
函数体内更窄的一段，重新核对命中一次）。**完整读法见跑前登记「十二、修订」session s9 条目**（8 条，逐条附手工
验证依据）。

````

**出处 `.claude/kb/experiments/158-择根与修复四岔路.md:1439-1465`（整段抄，未转述）**

````markdown
### 结果整行抄自产物（产物已移出工作区的行写成一行指针）

**2026-09-24 主 agent 定案**：`driver_e158`（第一段）承重产物改指
`e158-root-choice-repair-2026-09-24-segment1-rerun.out`（82 行，
`E7RESULT name=done emitted=70`）——这是 `crates/` 落地 C512（树表 0 条的一版上被换下的实例表记在哪没有条款）
定案（拿掉「回退到无文件那一版」的拒绝）**之后**重跑的产物；`scripts/experiment-apparatus/replay.sh` 的
`driver_e158` 行已改指到它。旧产物 `e158-root-choice-repair-2026-09-23-segment1.out`
原样留着，对应 **C512（树表 0 条的一版上被换下的实例表记在哪没有条款） 落地之前**的代码，不再是这一行比对的对象，50 行差异的逐格原因见「2026-09-24：第一段重跑」一节
一节与 `/tmp/claude-1000/e158-s2/report.md` 第 1 节。

```text
E7RESULT name=s5_independent_decode_cross_check verdict=pass readable_root_count=6 readable_roots={(0, 0), (1, 1), (1, 2), (1, 3), (1, 4), (1, 5)}
E7RESULT name=h3_family_summary geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) overwrites_before_sigma=[1, 2, 3] sigma_length_limit=3 histories=1822 peak_events_live=3 peak_roots_live=10 steps_with_positive_events_live=1702 leaf_count=1605 leaves_with_positive_events_live=1524 peak_events_live_at_a_leaf=3 error_count=855
E7RESULT name=h3_family_summary geometry=S16(S=16,ring=3MiB) overwrites_before_sigma=[1, 2, 3] sigma_length_limit=3 histories=1822 peak_events_live=3 peak_roots_live=10 steps_with_positive_events_live=1702 leaf_count=1605 leaves_with_positive_events_live=1524 peak_events_live_at_a_leaf=3 error_count=855
E7RESULT name=h3_family_summary geometry=小环 overwrites_before_sigma=[1, 2, 3] sigma_length_limit=3 histories=1822 peak_events_live=3 peak_roots_live=10 steps_with_positive_events_live=1702 leaf_count=1605 leaves_with_positive_events_live=1524 peak_events_live_at_a_leaf=3 error_count=855
E7RESULT name=h3_family_summary geometry=S4(S=4,ring=3MiB) overwrites_before_sigma=[1, 2, 3] sigma_length_limit=4 histories=14252 peak_events_live=4 peak_roots_live=10 steps_with_positive_events_live=13857 leaf_count=12430 leaves_with_positive_events_live=12155 peak_events_live_at_a_leaf=4 error_count=10629
E7RESULT name=peak_live_rollback_events_across_the_family value=4
产物行见 git show 5d0b7e7c1667:research/results/e158-root-choice-repair-2026-09-24-segment1-rerun.out 第 88 行（name=pc3）
产物行见 git show 5d0b7e7c1667:research/results/e158-root-choice-repair-2026-09-24-segment1-rerun.out 第 89 行（name=pc3_overall_verdict）
```

（`grep -c '^E7RESULT' e158-root-choice-repair-2026-09-24-segment1-rerun.out` → `70`；`grep -c 'verdict=pass'` → `42`、`grep -c 'verdict=fail'` → `0`；分类计数 `grep -oE 'name=[a-z0-9_]+' e158-root-choice-repair-2026-09-24-segment1-rerun.out | sort | uniq -c` 见「复跑」。`pc3` 的 `path` 从旧产物的 `RollbackTo(1, 3)` 变成 `RollbackTo(0, 0)`——PC3 取「按固定次序第一个满足 events_live≥1 的历史」，`(0,0)` 在 2026-09-24 重跑时合法且排在更前，核心断言 `expected_root=(1, 6)` 与两处 `verdict=pass` 不变。）

**岔路单第 2 行 ③（乙与候选 2 抢不抢槽空间）**：Q2-3 在与乙同放的四种宽度里，单条 16/12 都放得下（余量 7/11），两种表形态都同放推过 512（余量 −42/−26）——判定落在候选 2 取哪种宽度形态上，不落在单条上，与跑前登记的预判一致。

**岔路单第 3 行（候选 2 那一半）**：Q3-2 的 N_w = 4（来自 S4/L=4，比 `GEOMETRY_PRIMARY`/S16/小环 三个几何点的 3 更大），候选 2 的表形态见证（1+16*N_w=65、1+12*N_w=49）都推过 512（放得进 4096），单条见证不推过 512；PC3 用已有用例的同一种形态（回退之后再让新实例自己的根全部读不出）坐实了「读得出的被抛弃根里 (txg,实例) 最大的那一条」这个操作化的判定式与实测（recover / mount_writable）逐字一致。

````

**出处 `.claude/kb/experiments/158-择根与修复四岔路.md:1466-1515`（整段抄，未转述）**

````markdown
### 复跑

```
bash scripts/experiment-apparatus/replay.sh E158
```

**第 5 次重跑（2026-11-11）**：复跑表没有第 5 次重跑的行（理由与手跑的命令见页首「2026-11-11：第 5 次重跑」一节「复跑」段）；上面这条命令只复跑复跑表里 E158 已有的行，不碰第 5 次重跑的九份产物（2026-11-11 七份、2026-11-12 S4 两份）。

第一段（`driver_e158`，`-- all`）**2026-09-24 主 agent 定案改指今日承重产物**（见「结果整行抄自产物」与
「第一段重跑」两节：与 2026-09-23 旧产物比 50 行不同，逐格原因已诊断；改指之后 `driver_e158` 与今日承重产物
`e158-root-choice-repair-2026-09-24-segment1-rerun.out` 字节一致），旧产物原样留着、不再是比对对象；
Q3-1 四个几何点各占一行（`driver_e158_q3_1_g0` / `_s16` / `_small_ring` / `_s4`，走同一个二进制的
`q3-1-*` 模式），其中 G0（主几何点） 已现查两次独立跑逐字节相同（确定性核对，`diff` 无输出）。

**2026-09-24 复核**：
主工作区在段-2 之后又打进一批 `crates/` 未提交改动（`git status --porcelain crates/` 87 个文件，覆盖
`mount.rs`/`recovery.rs`/`transaction.rs`/`allocator.rs`/`instance_table.rs`/`root_record.rs`/
`root_ring.rs`/`system_configuration.rs`/`write_accounting.rs`/`singlefs-checker` 的 `image.rs`/`walk.rs`
等），重跑 `bash scripts/experiment-apparatus/replay.sh E158`（择根与修复四岔路） 一次：**5 个驱动全部「字节一致」**（`driver_e158` 与
四个 `q3_1_*`），说明这批改动没有改变 E158（择根与修复四岔路） 已测的任一条观测面；日志 `/tmp/claude-1000/e158-s3/replay-e158-2.log`。

**session s3 第二段**又新增 `driver_e158_q1_g0`（岔路单第 1 行，候选 (c)=今天，只跑 G0（主几何点） ）并登记进
`replay.sh`；重跑 `bash scripts/experiment-apparatus/replay.sh E158`（择根与修复四岔路）：**6 个驱动全部「字节一致」**（日志
`/tmp/claude-1000/e158-s3/replay-e158-3.log`）。候选 (a)=A1 副本的同一份数**不登记进 `replay.sh`**
（产物 `e158-root-choice-repair-2026-09-24-q1-g0-a1-arm.out`，副本上的数；复跑要先按
`scripts/mutation/tables/e158_arms.tsv` 在草稿目录重建 A1 副本、把
`crates/singlefs-implementation-experiments/src/bin/e158_root_choice_repair/main.rs` 原样拷进去、`cargo build --release`、
跑 `-- q1-g0`）。单测：`cargo test -p singlefs-verification --bin e158_root_choice_repair` → `16 passed; 0 failed`
（12 条到 session s3 第一段为止 + 4 条 session s3 第二段新增：`fault_targets_for_both_copies_hits_both_devices`、
`fault_severity_fault_count_matches_the_prereg_table`、`error_member_of_debug_extracts_the_variant_name`、
`location_key_ignores_the_checksum_field`）。变异：9 条手工验证（改坏必红、还原后 `diff` 逐字节一致、
基线重新全绿），登记在 `crates/mutations.tsv` 里 E158（择根与修复四岔路） 的九行（每行变异名都以「root_choice_repair」这个 bin 名起头，前面缀着这个实验的编号），归门禁 crates-mutation-replay 跑；
门禁 code-source-discipline 的 mutation-tables 格核过原文各命中源码一次（2026-09-24 `cargo fmt` 把 `is_abandoned` 的实现拆成三行后，对应那一条的
原文跟着改成三行、用 `\n` 分隔，重新核过命中一次）。运行记录（挂钟、git 状态、sha256）在
`e158-root-choice-repair-2026-09-23-segment1-run-record.out`（只覆盖 2026-09-23 第一段，
不参与 `replay.sh` 的逐字节比对）。

**session s4** 又新增 `driver_e158_q2_1_g0`（岔路单第 2 行 ①，候选 (c)=今天，只跑 G0（主几何点） 、n1∈{0,1,2,3}、
n2=1）与 `driver_e158_q2_2a_g0`（岔路单第 2 行 ②，固定脚本）并登记进 `replay.sh`；重跑
`bash scripts/experiment-apparatus/replay.sh E158`（择根与修复四岔路）：8 个驱动（含新增两个）全部「字节一致」（日志
`/tmp/claude-1000/e158-s4/replay-final.log`）。单测：`cargo test -p singlefs-verification --bin
e158_root_choice_repair` → `22 passed; 0 failed`（新增 6 条，见「session s4」一节）。变异：3 条手工
验证（改坏必红、还原后 `diff` 逐字节一致、基线重新全绿），登记在 `crates/mutations.tsv` 里 E158（择根与修复四岔路）
的十二行（累计，每行变异名都以「root_choice_repair」这个 bin 名起头，前面缀着这个实验的编号），归门禁 crates-mutation-replay 跑；门禁 code-source-discipline 的 mutation-tables 格核过原文各命中源码一次（489 条）。
甲-txg 臂的同一份数**不登记进 `replay.sh`**（产物 `e158-root-choice-repair-2026-09-24-q2-1-g0-jia-txg-arm.out`
与 `e158-root-choice-repair-2026-09-24-q2-2a-g0-jia-txg-arm.out`，副本上的数；复跑要先按
`scripts/mutation/tables/e158_arms.tsv` 甲-txg 那一行在草稿目录重建副本、把
`crates/singlefs-implementation-experiments/src/bin/e158_root_choice_repair/main.rs` 原样拷进去、`cargo build --release`、
分别跑 `-- q2-1-g0`、`-- q2-2a-g0`）。

````

**出处 `.claude/kb/experiments/158-择根与修复四岔路.md:1516-1597`（整段抄，未转述）**

```markdown
### 它答不了的

- （2026-11-09 起作废：岔路 1 的 q1 三行与三个驱动函数已删，session s3 至 s10 各节第一次跑的 Q1-1 至 Q1-4 读数不再承重，岔路 1 以第 4 次跑与第 5 次重跑为准，见「2026-11-09」一节 ④；第 5 次重跑没跑的项见页首「2026-11-12」一节「状态」）**岔路 1（C393（被抛弃根的账读不出时修复没有条款）） Q1-1 至 Q1-4 当时有数了，171 格差集的内容与根因
  session s9 已经查清，但 op1 第三种变体与几何敏感性仍不全**：见「2026-09-24（续，session s3 第二
  段）」「2026-09-24（续，session s6）」「2026-09-24（续，session s9）」三节的完整读法。几何敏感性
  session s6 已补两个方向相反的点（S16、S4，与 G0（主几何点） 逐字节相同，判定不翻面），小环、`G_默认` 两点仍未
  跑。session s7/s8 补了候选 (b) 的三种操作化（「crates 有路」、只走中央映射树、「走全」），171 格
  从「`unequal`，方向不明」变成「`subset_of_pristine`，方向已知但仍不够全」。**session s9 把这 171
  格的差集内容与根因查清了**：差集只有 2 个不同的落点（`(0, 50176)`、`(1, 50176)`，各出现 171
  次），就是 `RootRecord::instance_table` 自己的物理节点（`crates/singlefs-format` 的
  `UNIT_AREA_START_SLOT`=50176，与 `INSTANCE_TABLE_SLOT` 是同一个槽）——它是与 `tree_table`/
  `mapping_root` 并列的第三条独立指针，走全版本从来没有读过它；`TREE_KIND_LIVELIST`/
  `_SPARSE_SIDE_TABLE`/`_DEADLIST` 三种树在 2026-09-24 的 `crates/` **全仓任何一次发布**都恒是
  `NodePointer::empty_root()`（现查 `transaction.rs` 的 `publish_admitted`），不是这批历史的特例，
  也不是候选 (b) 走全版本漏了它们——差集与这三种树无关。**session s9 还把 8.1 要的持续/瞬时故障轨
  迹补上了**：`mount_writable_trajectory` 在 PC1-a 已选中的同一个落点上给出 persistent=
  `[Some(1), Some(1), Some(1)]`、transient=`[Some(1), Some(0), Some(0)]`，与登记原文逐字吻合。
  **当时还差（2026-11-09 起不再补，q1 三行已删）**：op1 第三种变体（先不注入 `mount_writable` 再 `raise_rollback_floor`，见跑前登记「十二、
  修订」session s9 条目第 5 条——`raise_rollback_floor` 的签名理论上接得进 `SimNode.session`，但要
  正确处理它内部推的若干次空发布各自产生的中间根，工程量本段没做完）；小环、`G_默认` 两个几何点
  （S16/S4 已补，dispatch 要求的「至少一个方向相反的几何点」已满足）。这些缺口逐条列在跑前登记
  「十二、修订」2026-09-24（session s3、s7、s8、s9）条目，不是随手漏掉。
- **岔路 2（C331（择根倒挂压过已确认的写） ①②③）session s9 用新机制把乙-留环/丁-留环/丁-只配置三
  份副本补齐到 n1=0..6，这一个切面够判了，但完整的「六臂×两档×两种」矩阵仍不算够判**：见「session
  s6」「session s7」「session s9」三节的完整读法。甲-txg 的 k_min 稳定=4，丙的 k_min 恰好=12（session
  s6 穷举下界坐实、session s9 的 `q2-1-g0` 主搜索本身在 n1=4 与 n1=6 上又各自独立给出 12），都与判
  决 K2 一致；③（四条修法与候选 2 抢不抢槽空间）session s6 已给出算式。**session s9 撤掉了旧的计
  数上限（`SUBSET_ENUMERATION_CAP`，权重档中途停手、不报完整空间多大），换成按权重档边界停的
  `weight_ceiling` 机制，恒报完整空间大小与穷举到的权重**：乙-留环、丁-留环、丁-只配置三份副本从
  实十九落地之后的新快照重建，跑完整个 n1=0..6：

  | n1 | 完整空间 | 结局 | subsets_tried |
  |---|---|---|---|
  | 0 | 2048 | 完整穷举，none_found | 2048 |
  | 1 | 8192 | 完整穷举，none_found | 8192 |
  | 2 | 32768 | 完整穷举，none_found（旧上限下报 capped，这次是真穷举完了） | 32768 |
  | 3 | 131072 | 穷举到权重 12，not_found_up_to_weight_ceiling | 82598 |
  | 4 | 524288 | 穷举到权重 12，not_found_up_to_weight_ceiling | 229720 |
  | 5 | 2097152 | 穷举到权重 12，not_found_up_to_weight_ceiling | 582829 |
  | 6 | 8388608 | 穷举到权重 12，not_found_up_to_weight_ceiling | 1367161 |

  三份副本逐字节相同
  （`e158-root-choice-repair-2026-09-24-q2-1-g0-configuration-evidence-yi-ring-retained.out`、
  `e158-root-choice-repair-2026-09-24-q2-1-g0-configuration-evidence-ding-ring-retained.out`、
  `e158-root-choice-repair-2026-09-24-q2-1-g0-configuration-evidence-ding-only-configuration.out`）。
  **读法**：n1=0..2
  是完整穷举（`subsets_tried` 恰等于完整空间），n1=3..6 都在权重 12 那一档边界上停手，完整空间与
  穷举到的权重如实报出，没有静默截断。跑前登记 5.8 分段表第 777 行「Q2-1 全族里各模板区间…的最小
  者」要求的是**全族**（n1=0 到 6），这次是全族的结果：**乙-留环/丁-留环/丁-只配置在权重≤12、覆
  盖 H2 全族的范围内，一次都没有被今天/丙那个「4 根槽 + 4 条记录」权重 12 的具体构造打中**——它们
  的 k_min（如果存在）这次确立为 **> 12**，与甲-txg 的区间 `[4,4]`、丙的区间 `[12,12]` 都不重叠。
  按第 761 行「行 2 够判，除非各臂的 k_min 区间两两重叠」，**这一个切面（今天/甲-txg/丙 与
  乙-留环/丁-留环/丁-只配置，崩溃档 0，任一种打中）这次够判**——K2 给乙的那个「0 故障+1 崩溃点」
  具体构造（session s5/s7）在乙-留环上仍 `hit=false`（保留根环读取挡住了这个具体构造），在丁-留环、
  丁-只配置上也仍 `hit=false`（取号续传挡住了）；H-C2 直接构造（session s8，精确针对
  `choose_system_configuration` 的设备优先级机制假说）与两负对照在三份副本上仍 `hit=false`。
  **仍不是完整的「六臂×两档×两种」矩阵，标「够判后未跑」，不是本段能补的欠账**：H2-R
  （op2=`mount_rollback`）、H2c（崩溃档 1）两个变体；n2=2（n1 已补到 {0,...,6}）；Q2-2b（乙族字段
  表宽度解码核对，三份副本都只跑了固定脚本按结构种类报字节，没有单独做字段表宽度解码核对）。
- **岔路 4（C334（切换的所选根没有会红的检查）与 Q4-1/Q4-2）** 前置未满足，按登记第五段不开，不是失败也不是
  作废。⚠️ **session s3（2026-09-24 续）更正**：`grep -rn -i 'switch' crates/singlefs-core/src/` **不再是
  0 命中**——新文件 `crates/singlefs-core/src/admission.rs` 有 19 处（`INSTANCE_SWITCHES_ALLOWED_PER_MOUNT`、
  `instance_switch_reserve_on_one_device`），逐条读过确认是 D28（挂载期承诺量） 已定项 3 的**空间预留算术**，
  不是实例切换本身（没有取新号、写行、重发在飞 checkpoint 的代码）；`grep -rn -i 'probe'` 22 处全在
  `block_device.rs` 的 `probe_physical_block_size`（探测物理块大小），与 D23（journal 的角色与格式） 已定项 14 的「固定落点探针写」
  无关。两项前置条件逐条读过后仍都不满足，P4 判定不变，但「0 命中」这句表述本身已经不成立——
  `impl-m2-mountfix` 已经在为实例切换打地基（先落空间预留），下一次核 P4 时留意 `admission.rs`。
- **Q3-1 的 175 个「V1 在 (a) 消失、V2 在 (a)(b)(c) 三处持续为真」的对，具体形态没有拆开看**：不知道它们是不是
  都落在「回退到无文件版本」这个新出现的分支上、内容号具体对应哪个历史节点——这一段只报了汇总计数，没有
  逐条摘出来读；`e158-root-choice-repair-2026-09-24-q3-1-g0.out` 里有全部原始数据，
  逐条拆解留给下一段或三方判决用。
- **G_默认（768 MiB 环）没有独立跑 Q3-1**：`scan_journal` 按环全部记录槽逐一读，768 MiB 环是 3 MiB 环的 256 倍，
  按 G0（主几何点） 的 real 2m58s 线性外推跑不完；只给出机制论证（判定不依赖环总槽数，已在 S=8→S=16、3 MiB→49152 字节
  两个方向验证过），**不是独立观测**。
- **H3 的省法自己没有单独验证过会不会漏判**：5.1 原文要求省法有自检抽样（「每 100 个抽 1 个全做」），Q3-1
  的实现里省法条件是「(a) 所选根与 F=∅ 相同」，逻辑上（b)(c) 在 F=∅ 时必然非违例，但**没有做 5.1 原文那条
  独立抽检**去证明「省掉的那些的确都不会是违例」——这一段直接论证了省法的正确性（见「新增了什么」一节），
  但没有按登记原文再加一层抽样自检，留给下一段判是否需要补。
- **小环取法的代理探针是简化版**：见 2026-09-23 记录，未变。
- **门禁 doc-experiments 的 results-cited 格与当时编号 85、86 的两道阶段的完整核对**：本次修改实验页与索引行、修了 `cargo fmt`/clippy，但没有重新跑一遍
  这三个阶段确认变绿（归属 `gate-triage`）。

```

**出处 `.claude/kb/experiments/158-择根与修复四岔路.md:1598-1680`（整段抄，未转述）**

````markdown
### 2026-09-25（续，session s10）：op1 第三种变体（岔路单第 1 行）；`replay.sh` 全量重出；发现另一条并行线正在改的 `crates/` 影响岔路单第 3 行的前提

**op1 第三种变体做完了**：新增 `FaultedOperationKind::MountWritableThenRaiseFloorTo`（先不注入地
做一次 `mount_writable` 只为量出这一步的 F 与上限，受注入的一步是 `raise_rollback_floor` 本身），
按跑前登记 Q1-1c 原文「抬 F 那一处单独一行」不并入既有的 `pairs`/`trigger_count`。**结果整行抄自
产物**（`research/results/e158-root-choice-repair-2026-09-25-q1-g0-today.out`）：

```text
E7RESULT name=q1_1a_raise_floor_trigger_summary geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) pairs=96 trigger_count=32 history_nodes_without_room=26
E7RESULT name=q1_1a_raise_floor_by_aspect_severity geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) aspect=allocation_record_tree severity=both_copies pairs=16 trigger=16
E7RESULT name=q1_1a_raise_floor_by_aspect_severity geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) aspect=allocation_record_tree severity=disk0_only pairs=16 trigger=0
E7RESULT name=q1_1a_raise_floor_by_aspect_severity geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) aspect=allocation_record_tree severity=disk1_only pairs=16 trigger=0
E7RESULT name=q1_1a_raise_floor_by_aspect_severity geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) aspect=tree_table severity=both_copies pairs=16 trigger=16
E7RESULT name=q1_1a_raise_floor_by_aspect_severity geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) aspect=tree_table severity=disk0_only pairs=16 trigger=0
E7RESULT name=q1_1a_raise_floor_by_aspect_severity geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) aspect=tree_table severity=disk1_only pairs=16 trigger=0
```

**读法**：两份都读失败（`both_copies`）时两个指称各 16/16 全触发，只坏一份（`disk0_only`/
`disk1_only`）时 0/16——与既有 `mount_writable`/`mount_rollback` 那两种 op1 在同一批构造上的模式
一致（另一份仍读得出账）；`history_nodes_without_room=26` 是这个历史节点在这一步没有 `[F+1, 上限]`
余量可抬（不是漏测）。G0（主几何点） 与 S16 两点逐字节相同
（`research/results/e158-root-choice-repair-2026-09-25-q1-s16.out`），S4 不同
（`pairs=132 trigger_count=44 history_nodes_without_room=25`，
`research/results/e158-root-choice-repair-2026-09-25-q1-s4.out`）——几何变了、H1 家族的节点与候选
根都会变，不要求与 G0（主几何点） 相同。单测 39→43（4 条新增：3 条 `floor_targets_between` 边界 + 1 条端到端
「不注入必须成功、注入必须与不注入不同」），变异 30→33（3 条），`cargo test --release -p
singlefs-verification --bin e158_root_choice_repair` 43 passed，`bash .claude/gate.d/code-source-discipline.sh --check mutation-tables` 复跑绿。**没做「把 raise_rollback_floor 里若干次空发布各自的中间根追加进
node.timeline」**——现查发现这一步不需要：`RaisedFloor::abandoned_roots_unreadable` 是从盘上现读
实例表 + 现行根的候选集算出来的，不依赖装置自己的内存时间线（与 `MountRollbackTo` 分支同理，那个
分支交回的中间根同样从不追加进时间线），详见跑前登记「十二、修订」session s10 条目第 2、3 条。

**`replay.sh` 全量重出，发现另一条并行线正在改的 `crates/` 影响岔路单第 3 行的前提**：
`bash scripts/experiment-apparatus/replay.sh E158`（择根与修复四岔路）（14 行）：字节一致 4／对不上 10／跑不了 0。1 行
（`driver_e158_q2_1_g0`）是 session s4 遗留、driver 自己的注释早就说明不再当依据引用，本来就该对
不上；其余 9 行按今天日期另存新产物：
`research/results/e158-root-choice-repair-2026-09-25-segment1-rerun.out`、
`research/results/e158-root-choice-repair-2026-09-25-q3-1-g0.out`、
`research/results/e158-root-choice-repair-2026-09-25-q3-1-s16.out`、
`research/results/e158-root-choice-repair-2026-09-25-q3-1-small-ring.out`、
`research/results/e158-root-choice-repair-2026-09-25-q3-1-s4.out`、
`research/results/e158-root-choice-repair-2026-09-25-q1-g0-today.out`、
`research/results/e158-root-choice-repair-2026-09-25-q2-2a-g0-today.out`、
`research/results/e158-root-choice-repair-2026-09-25-q1-s16.out`、
`research/results/e158-root-choice-repair-2026-09-25-q1-s4.out`；
`…-09-24-…` 原样留着，`replay.sh` 登记表已改指向新文件。逐份现查差在哪（不是只看行数），除了本段新增的 `q1_1a_raise_floor_*` 行之外，全部差异都指
向同一批文件：主 agent 知会另一条并行线（「实二六」）此刻正在改
`crates/singlefs-core/src/mount.rs`、`allocator.rs`、`allocation_record_tree.rs`（抬 F、挂载读、
分配记录树根层，见「重出产物之前」与「交回前」两份 crates/ 快照
`e158-root-choice-repair-2026-09-25-s10-crates-sha256.out`、
`e158-root-choice-repair-2026-09-25-s10-crates-sha256-final.out`）：

- `driver_e158` 的 `pc3` 一行从 `recover_verdict=pass recover_root=Some((1, 6))` 变成
  `recover_verdict=fail recover_root=Some((0, 0))`——与 session s9 报告的同一处翻转方向一致。
- **`q3_1_g0`/`q3_1_s16`/`q3_1_small_ring`（岔路单第 3 行 Q3-1）**：`pairs_with_any_violation`
  从 213 变成 0；**`q3_1_s4`** 除了同一项从 1355 变成 0，`cold_recover_with_fault_failed`/
  `mount_writable_with_fault_failed` 还从 0 变成 239——同一批构造里，239 个从「成功但违例」变成
  「调用直接报错」，失效形态本身变了。**⚠️ 这一条可能动到岔路单第 3 行「候选 3 = 今天」
  这个前提与 F2 的触发条件**：C332（回退实例两个根都读不出时回退被撤销） 正文钉的「2 个故障就能撤
  销回退」在今天这份 `crates/`（含另一条并行线还没提交的改动）上现在测不出来了。是要重新核对 F2、
  还是这一刻的 `crates/` 状态本就不该当「候选 3 = 今天」的代表（等那条并行线落定提交后再复核），
  交主 agent 判断，本段不越权下结论、不改判据或臂定义。
- `q1_2_tree_table_crates_has_a_path`（`verify_named_units=false`）从「`did_not_reach_
  target=108`+`reached_target_with_matching_pointer=81`」变成「`did_not_reach_target=189`」（原
  来能不碰物理拷贝复算出指针的 81 格现在全部走不到）；`pc1_a` 的 `device_write_bytes` 从 157184
  变 353792；`q2_2a_g0` 的 `AllocationRecordTreeNode` 每次发布 `write_calls`/`written_bytes` 全
  面上涨（如 `write_calls=2`→`10`）——三处都与 session s9 报告的方向一致，是既有漂移的重复坐实。
- **没有做第二次确认性复跑**：新存的 9 份文件是这一次真实运行的原始输出，逐字节复跑要再等两个
  `q2-1-g0` 全穷举跑一遍（约 1～2.5 小时），而 crates/ 这一刻仍可能继续变，投入产出比低，续不续跑
  交主 agent 定。

**doc-lint 第 497 行附近的位置指代——现查没有找到**：`bash .claude/scripts/doc-lint.sh` 全仓跑一遍
对本页给出 0 处违规；第 498 行「详见 `.claude/kb/experiments/158-择根与修复四岔路.md`「2026-09-24
（续，session s9）」一节」指向的标题在第 759 行确实存在，是写明文件名与小节标题的有效引用。派发点
名的问题这一刻的正文里没有找到，大概率是主 agent 派发时看到的是 session s9 那两处自指还没修的中间
状态（s9 交回前已改成写死文件名与小节标题、复跑确认过 0 处），不强行改一处现查看不出毛病的文字。

**它答不了的（更新）**：岔路单第 1 行——op1 第三种变体已做完，仍差小环、`G_默认` 两个几何点；岔路
单第 2 行——H2-R、H2c、n2=2、Q2-2b 仍是「够判后未跑」；本段现查到的 `crates/` 波动（`mount.rs`/
`allocator.rs`/`allocation_record_tree.rs`，另一条并行线还没提交）让岔路单第 3 行「候选 3 = 今天」
这个前提本身可能已经不成立，交主 agent 判断续不续核；`replay.sh` 没有做第二次确认性复跑——理由见
「（续，session s10）：op1 第三种变体（岔路单第 1 行）；`replay.sh` 全量重出；发现另一条
并行线正在改的 `crates/` 影响岔路单第 3 行的前提」这一节「`replay.sh` 全量重出……」一段最后一条。

````

**出处 `.claude/kb/experiments/158-择根与修复四岔路.md:1681-1691`（整段抄，未转述）**

```markdown
### 影响的决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D16（发布语义） 已定项 6 | 备料 | 2026-11-17 不受影响：只改过时的现状句与产物指针，结论、数与产物不变；此前 2026-11-13 不受影响：第 5 次重跑只多了 S4 一档的同类读数，这条分项的依据没动，结论不变；此前 2026-11-11 不受影响：第 5 次重跑只量被抛弃根的账读不出时的挂载、隔离与复用，这条分项的依据没动，结论不变；此前 2026-11-09 不受影响：本次只换岔路单第 3 行（Q3-1、H3、PC3）的账本重出并删岔路 1 的 q1 三行，这条分项的依据（第 4 次跑与决策正文）没动，结论不变；此前 2026-11-05 不受影响：改名只动措辞，结论、数与产物不变；此前 2026-11-03 不受影响：只改复跑写法，结论、数与产物都不变；此前 2026-10-24 不受影响：三语段落排版对齐与复跑路径改名，数、结论与产物不变 |
| D23（journal 的角色与格式） 已定项 14 | 支撑 | 2026-11-17 不受影响：只改过时的现状句与产物指针，结论、数与产物不变；此前 2026-11-13 不受影响：S4 一档出齐、四档齐，today 臂 Q1 至 Q4 违例四档都是 0，与这条分项「账读不出时从映射现算」的现状一致，依据与结论不动；此前 2026-11-11 不受影响：第 5 次重跑三档读数与这条分项「账读不出时从映射现算」的现状一致，依据与结论不动；S4 一档没跑，四档齐之前不改依据；此前 2026-11-09 不受影响：本次只换岔路单第 3 行（Q3-1、H3、PC3）的账本重出并删岔路 1 的 q1 三行，这条分项的依据（第 4 次跑与决策正文）没动，结论不变；此前 2026-11-05 不受影响：改名只动措辞，结论、数与产物不变；此前 2026-11-03 不受影响：只改复跑写法，结论、数与产物都不变；此前 2026-10-24 不受影响：三语段落排版对齐与复跑路径改名，数、结论与产物不变 |
| D22（单元原子性怎么合成） 已定项 9 | 备料 | 2026-11-17 不受影响：只改过时的现状句与产物指针，结论、数与产物不变；此前 2026-11-13 不受影响：第 5 次重跑只多了 S4 一档的同类读数，这条分项的依据没动，结论不变；此前 2026-11-11 不受影响：第 5 次重跑只量被抛弃根的账读不出时的挂载、隔离与复用，这条分项的依据没动，结论不变；此前 2026-11-09 不受影响：本次只换岔路单第 3 行（Q3-1、H3、PC3）的账本重出并删岔路 1 的 q1 三行，这条分项的依据（第 4 次跑与决策正文）没动，结论不变；此前 2026-11-05 不受影响：改名只动措辞，结论、数与产物不变；此前 2026-11-03 不受影响：只改复跑写法，结论、数与产物都不变；此前 2026-10-24 不受影响：三语段落排版对齐与复跑路径改名，数、结论与产物不变 |
| D8（核心索引结构） 已定项 8 | 备料 | 2026-11-17 不受影响：只改过时的现状句与产物指针，结论、数与产物不变；此前 2026-11-13 不受影响：第 5 次重跑只多了 S4 一档的同类读数，这条分项的依据没动，结论不变；此前 2026-11-11 不受影响：第 5 次重跑只量被抛弃根的账读不出时的挂载、隔离与复用，这条分项的依据没动，结论不变；此前 2026-11-09 不受影响：本次只换岔路单第 3 行（Q3-1、H3、PC3）的账本重出并删岔路 1 的 q1 三行，这条分项的依据（第 4 次跑与决策正文）没动，结论不变；此前 2026-11-05 不受影响：改名只动措辞，结论、数与产物不变；此前 2026-11-03 不受影响：只改复跑写法，结论、数与产物都不变；此前 2026-10-24 不受影响：三语段落排版对齐与复跑路径改名，数、结论与产物不变 |
| D28（挂载期承诺量） 已定项 3 | 备料 | 2026-11-17 不受影响：只改过时的现状句与产物指针，结论、数与产物不变；此前 2026-11-13 不受影响：第 5 次重跑只多了 S4 一档的同类读数，这条分项的依据没动，结论不变；此前 2026-11-11 不受影响：第 5 次重跑只量被抛弃根的账读不出时的挂载、隔离与复用，这条分项的依据没动，结论不变；此前 2026-11-09 不受影响：本次只换岔路单第 3 行（Q3-1、H3、PC3）的账本重出并删岔路 1 的 q1 三行，这条分项的依据（第 4 次跑与决策正文）没动，结论不变；此前 2026-11-05 不受影响：改名只动措辞，结论、数与产物不变；此前 2026-11-03 不受影响：只改复跑写法，结论、数与产物都不变；此前 2026-10-24 不受影响：三语段落排版对齐与复跑路径改名，数、结论与产物不变 |
| D28（挂载期承诺量） 已定项 1 | 支撑 | 2026-11-17 不受影响：只改过时的现状句与产物指针，结论、数与产物不变；此前 2026-11-13 改了：四档齐，这条分项的依据段引回 E158（择根与修复四岔路）；此前 2026-11-11 不受影响：第 5 次重跑三档量到的隔离数，与第九项「被抛弃根独占量」的取值口径（账读不出时从映射现算的落点也在内）一致；该分项的依据段没有引 E158（择根与修复四岔路），所以写备料，不改结论。 |

```

**出处 `.claude/kb/experiments/180-准入读数与真可写量.md:1-20`（整段抄，未转述）**

```markdown
## E180 准入读数与真可写量 —— 未跑（第一段读数段装置、45 条单测、22 条变异行与分片长跑脚本写成，读数长跑两趟都没跑完、没有产物；第二段扣住位段装置没写）

**中文** · [English](180-acceptance-reading-vs-truly-writable-amount.en.md) · [日本語](180-受入読み取り値と真の書き込み可能量.ja.md)

<!-- doc-lint:not-numbers F2 F5 F6 F8 W0 W1 W2 W3 W4 R1 R2 R3 K0 K1 K2 K3 S1 S2 S3 Q1 Q2 Q3 Q4 Q5 Q6 Q7 Q8 Q9 Q10 Q11 Q12 Q13 X1 X2 X3 X4 X5 X6 X7 A1 A3 A4 B1 B2 B3 B4 -->

跑前登记 `research/prompts/e180-preregistration.md`（两段：第一段读数段答 F2、F8 与 F5 里「带被抛弃根、净释放的历史」那一半；第二段扣住位段答 F6 与 F5 里「扣住位的历史」那一半）。回答的是岔路单 `research/prompts/m3-close-admission-and-shadow-ledger-forks.md` 的四行：F2（`df` 跟哪一条式子、活元数据随它：W1 照 D16（发布语义） 已定项 1 的 `df` 字面，W2 照 D28（挂载期承诺量） 的式子可用、各盘取小，W3 两盘可用相加 ÷ 副本数，W4 在 W2 上再扣写 n 单元那一次的非数据需求上界）、F5（C356（第九项按统计量之差算，净释放时为负且没有钳位） 那一行的去向：R1 还清、R2 改写检查、R3 留着）、F6（抬 F 写错之后的扣住位：K1 读数把扣住算占着、K2 每块盘都有带现行 F 的根就放开、K3 推 F 不变的空发布直到每块盘都有带 F 的根再放开）、F8（`df` 报不报段内关着的空槽：S1 报、S2 不报、S3 照 S2 另报一项「重挂之后可用」）。同一份岔路单的 F9、F10、F11、F14 在 E181（落得下那一判与挂载下界），B1、B2、B4 在 E182（被抛弃根读不出的三条岔路）。

背景是 C370（需求、可用与 df 没有共同单位）、C371（活元数据没有定义）、C356（第九项按统计量之差算，净释放时为负且没有钳位）、C546（抬 F 被拒时扣住的槽不退回）、C570（抬 F 回收扣住的槽计数上空闲但分配器不发）、C571（df 算不算聚簇段内关着的空槽，假性 ENOSPC 待判）六条欠账；被测条款是 D16（发布语义） 已定项 1（`df`、准入、抬 F 那一串、正常卸载）、D28（挂载期承诺量） 已定项 1（九项式子）与已定项 3（「`df` 报可用」那一句）、D3（空间分配） 已定项 12（两道串联的闸），它们引的定义是 D3（空间分配） 已定项 8（聚簇段对用户数据关着）与已定项 9（假性 ENOSPC 的定义）。

装置是入库装置 `crates/singlefs-implementation-experiments/src/bin/e180_admission_reading_versus_writable_bytes/main.rs`（只读、只驱动与观测 `crates/` 今天的会话发布路径、准入读数与分配器，不改 `crates/singlefs-core`、`crates/singlefs-checker`；要读 `crates/` 常量的量都是本地常量，值抄自 kb 的登记位，开跑先与实现侧回比）。这一份只有第一段；第二段（F6 的造格、五条臂重试与崩溃点枚举）不在里面。

**现状**：

- 第一段装置写成，源文件 `grep -c '#\[test\]'` = 45（sha256 前 16 位 `2c0bc641d1f0a0d5`）。登记第十二节修订 4 记的单测读数是换进主工作区那一版（sha256 `890dde7b…`）`test result: ok. 45 passed; 0 failed`；之后 clippy 修补与两次改名补丁又换过装置，现在这一版的单测读数这一页没有。单测在验证包里，归主 agent 经统一任务服务交。
- 变异行 22 条追加在 `crates/mutations.tsv` 末尾（名字以小写 `e180` 起头，`grep -c '^e180 '` = 22）：登记第九节里第一段罩得到的 7 条加装置自己补的 5 条是修订 3.6 记的 12 条，另 10 条打的是修订 4 的按需收紧（4 条）与分片合并（6 条）；三个数待提交时门禁 crates-mutation-replay。
- 分片长跑脚本 `scripts/experiment-apparatus/e180-reading-shards.sh`：第一段切成 29 片（5 个几何 × HF、HO、HU、HM、HC 五族，加 G不等大之外 4 个几何的 HL），片齐了由装置的 `merge` 子命令合成一份产物；开跑时把装置程序文件拷一份钉住，每片记打它的装置 sha256，续跑只沿用同一版的片（欠账 C862（长跑分片续跑不核装置是哪一版） 的修法）。
- 读数长跑两趟都没跑完：第一趟 9 片没做完（内存包装排不上或撞顶）；续跑那一趟里仓根的装置二进制被别的编译换掉，几版装置的片会合进同一份产物，主 agent 停了它（`records/2026-10-18-里程碑三收尾与近期欠账.md` 第 230、310 行），片在仓外、不入库。`research/results/` 下没有 E180（准入读数与真可写量） 的产物，原始输出未留存（一趟都没跑完）；`scripts/experiment-apparatus/replay.sh` 也没有它的登记行。
- 第二段（扣住位段）装置没写。F2、F5、F6、F8 四行这一页都没有读数，岔路单这四行仍开着。

```

**出处 `.claude/kb/experiments/180-准入读数与真可写量.md:21-28`（整段抄，未转述）**

```markdown
### 测什么

- **状态**：一次挂载里，会话两次用户改动之间的那一刻（`publish_user_change` 返回之后）；可写挂载刚做成也算一个。只经核心层的公开入口造：mkfs、可写挂载、`MountedSession::publish_user_change`、正常卸载、按录制流截断造崩溃、在设备调用上注入错误。
- **T（这一刻写得进的最大字节数）**：在这个状态的拷贝上，`UserChange::SequentialWrite` 把第一个文件整份换成 n × 32634 字节的内容，经会话（含会话为空间不够推的抬 F）做得成的最大 n，乘 32634。T 只在被拒的原因是空间不够时算数；被别的原因拒的那个状态 T 记「取不出」。
- **多报与少报**：候选 c 在状态上报 X_c 字节，⌈X_c ÷ 32634⌉ > T ÷ 32634 即多报，多报数 X_c − T；少报是 T − X_c（> 0 时）。
- **几何**（默认环 768 MiB、io_min 4096）：G默认（两块 4 GiB，S = 8）、G不等大（4 GiB 与 3 GiB + 33 槽）、G窄（两块 3 GiB + 33 槽），几何敏感性另取 G默认-S4 与 G默认-S16。
- **历史族**（每族一条确定的脚本，种子写死在登记第五节）：HF 填满、HO 覆盖写、HU 删了再写、HM 多次挂载、HC 被抛弃根与净释放、HL 长挂载。

```

**出处 `.claude/kb/experiments/180-准入读数与真可写量.md:29-37`（整段抄，未转述）**

```markdown
### 判据（登记第六节，跑前写死）

- Q1：每条 F2 臂、每个几何格上多报的状态数，= 0 判「不多报」，≥ 1 判「多报」；Q2、Q3 报多报的最大字节与少报上界；Q4 是 T 取不出的状态数，超过那一格状态数的 10% 时那一格的 Q1–Q3 不判。
- Q5：F5 的 R2 残差不为 0 的状态数，按扣住位族与被抛弃根、净释放族分开，两族都 = 0 判「R2 写得成」；Q6 把残差与「扣住且隔离」的槽数并排报；Q7：R3 残差不为 0 的状态数，≥ 1 判「欠账行原式在可达状态上为假」。
- Q8–Q11（第二段）：F6 每格（F 已生效、F 没生效）每条臂重试的那一次写做成的例数，K0、K2、K3 放开之后崩溃窗口里「被盖」的崩溃状态数（两盘、只剩盘 0、只剩盘 1 分开），K3 多推的空发布次数，两种格各造出几例。
- Q12：F8 每个几何格每块盘「段内关着的空槽 − 环窗口」的峰值、为正的状态数、期末值，峰值 > 0 判「S1 在可达状态上多报」；Q13 报关着的空槽本身的峰值。
- 阳性对照：每条 F2 臂另算 max(X_c, T) + 32634 必须判多报；已知多报的 W0（不除副本数、不扣承诺量）在 T > 0 的状态上必须判多报；X = 0 不许判多报。F5 的 R2 去掉扣住项、去掉重叠项，各自在对应状态上残差必须不为 0。F8 的计数器在 mkfs 之后盘 0 开着的段 [50240, 50304) 里数关着的空槽必须恰是 52。任一条没中，那一段作废。
- 修订 1：T 被 journal 记录上限（默认环 65536 条）截尾的状态记下界 L，候选要的单元数 ≤ L 报「一定不多报」、> L 报「未定」，不进多报数；修订 4 改成只在判定会变的那一格收紧下界、第一段分 29 片跑再合并。判定一个字不变。

```

**出处 `.claude/kb/experiments/180-准入读数与真可写量.md:38-48`（整段抄，未转述）**

```markdown
### 失败条款（登记第十节）

- X1：F2 的候选在同一个几何格上全部多报：那一格打中的不分辨臂，不拿它判臂。
- X2：某条 F2 臂在方向相反的两个几何格上一边多报、一边不多报：记「不稳定」。
- X3：Q5 任一族 ≥ 1：R2 的两项式写不成。
- X4：K2 或 K3 放开之后在崩溃窗口里盖掉恢复择到的根引用的单元：那条臂出局。
- X5：连 K0（写错交回就立刻放开）都没盖：F6 只按 Q8、Q10 交。
- X6：F 没生效的那一格造不出：F6 那一半还差，不推「不可达」。
- X7：Q12 任一几何格任一块盘峰值 > 0：S1 在这段历史上多报。
- 第十节「条款可能错」那一条：出自被测条款本身的锚点（第七节 A1 至 A4）与实现不符时，那一条锚点相关的量停下交回，不当装置坏作废。

```

**出处 `.claude/kb/experiments/180-准入读数与真可写量.md:49-55`（整段抄，未转述）**

```markdown
### 口径

- 单位：槽 16 KiB；一个数据单元两槽、每块盘各落一份；数据单元载荷 32634 字节（32768 − 头 134）。`df` 候选的值一律换成字节报。
- T 的探测在状态的拷贝上做，设备是写时复制的叠层，第一个数据单元的写到达时报块设备错、记放行；这条捷径要先在 HF 族上与整写逐项同判才用，不同就整段改整写（登记 5.1、修订 2）。
- 会话没有带偏移的覆盖写入口，F8 够判条件里的「带偏移覆盖写」用 HL（整份换写的长挂载）代替；「大文件里局部覆盖、其余单元留着」造成的段内空洞罩不到（修订 3.4）。
- 第一段的历史里抬 F 那一串都整串发成、整串放开，扣住账 H 恒 0，扣住位那一半归第二段（修订 3.5）。

```

**出处 `.claude/kb/experiments/180-准入读数与真可写量.md:56-70`（整段抄，未转述）**

````markdown
### 复跑

```bash
# 仓根。编与单测（验证包，主 agent 经统一任务服务交）
cargo build --release -p singlefs-verification --bin e180_admission_reading_versus_writable_bytes
cargo test --release -p singlefs-verification --bin e180_admission_reading_versus_writable_bytes
# 第一段（长跑；分片、续跑与多机都在脚本里，片齐了合并出产物）
bash scripts/experiment-apparatus/e180-reading-shards.sh
bash scripts/experiment-apparatus/e180-reading-shards.sh --list-items    # 逐行列片名与做没做完
# 装置的子命令（经 task-server：task-server/task submit -- cargo run --release -p singlefs-verification --bin e180_admission_reading_versus_writable_bytes -- <子命令…>）
#   reading [--geometry <几何名>] [--family <族名>]                    一次跑全部或一格，产物打到标准输出
#   reading --geometry <几何名> --family <族名> --shard-output <文件>   只跑一片
#   list-shards / merge <片目录> / calibrate                           列片、合片、量挂钟外推
```

````

**出处 `.claude/kb/experiments/180-准入读数与真可写量.md:71-76`（整段抄，未转述）**

```markdown
### 它答不了的

- 第一段没有产物，F2、F5、F8 这一页都没有读数；第二段装置没写，F6 与 F5 的扣住位那一半没有装置。
- F8 只对整份换写的历史成立（修订 3.4）；HC 只造「记录已持久、根槽读不出」一种被抛弃根，造没造出按挂载丙里分配器记的被抛弃根槽或隔离槽判。
- 第七节 A4（两块 4 GiB 盘 529 个节点）只核闭式，实现侧那个函数是私有的，没核（修订 3.3）。

```

**出处 `.claude/kb/experiments/180-准入读数与真可写量.md:77-87`（整段抄，未转述）**

```markdown
### 影响的决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D16（发布语义） 已定项 1 | 备料 | 2026-11-05 不受影响：只改用法与复跑说明（一个门禁、两种跑法；验证包改名），结论、数与产物不变；此前 2026-11-03 不受影响：只改复跑写法，结论、数与产物都不变；此前 2026-11-01 不受影响：只改复跑起法；此前 2026-10-21 不受影响：该分项依据段没引 E180（准入读数与真可写量），第一段产物没有，只备料，决策正文这一次没有改 |
| D28（挂载期承诺量） 已定项 1 | 备料 | 2026-11-05 不受影响：只改用法与复跑说明（一个门禁、两种跑法；验证包改名），结论、数与产物不变；此前 2026-11-03 不受影响：只改复跑写法，结论、数与产物都不变；此前 2026-11-01 不受影响：只改复跑起法；此前 2026-10-21 不受影响：该分项依据段没引 E180（准入读数与真可写量），九项式子的读数第一段才出，产物没有，决策正文这一次没有改 |
| D28（挂载期承诺量） 已定项 3 | 备料 | 2026-11-05 不受影响：只改用法与复跑说明（一个门禁、两种跑法；验证包改名），结论、数与产物不变；此前 2026-11-03 不受影响：只改复跑写法，结论、数与产物都不变；此前 2026-11-01 不受影响：只改复跑起法；此前 2026-10-21 不受影响：该分项依据段没引 E180（准入读数与真可写量），「`df` 报可用」那一句由 F2 的 W2 臂量，产物没有，决策正文这一次没有改 |
| D3（空间分配） 已定项 12 | 备料 | 2026-11-05 不受影响：只改用法与复跑说明（一个门禁、两种跑法；验证包改名），结论、数与产物不变；此前 2026-11-03 不受影响：只改复跑写法，结论、数与产物都不变；此前 2026-11-01 不受影响：只改复跑起法；此前 2026-10-21 不受影响：该分项依据段没引 E180（准入读数与真可写量），两道串联的闸只作被测条款，产物没有，决策正文这一次没有改 |
| D3（空间分配） 已定项 8 | 备料 | 2026-11-05 不受影响：只改用法与复跑说明（一个门禁、两种跑法；验证包改名），结论、数与产物不变；此前 2026-11-03 不受影响：只改复跑写法，结论、数与产物都不变；此前 2026-11-01 不受影响：只改复跑起法；此前 2026-10-21 不受影响：该分项依据段没引 E180（准入读数与真可写量），段内关着的空槽由 F8 量，产物没有，决策正文这一次没有改 |
| D3（空间分配） 已定项 9 | 备料 | 2026-11-05 不受影响：只改用法与复跑说明（一个门禁、两种跑法；验证包改名），结论、数与产物不变；此前 2026-11-03 不受影响：只改复跑写法，结论、数与产物都不变；此前 2026-11-01 不受影响：只改复跑起法；此前 2026-10-21 不受影响：该分项依据段没引 E180（准入读数与真可写量），假性 ENOSPC 的定义只作多报的判法，产物没有，决策正文这一次没有改 |

```

**出处 `.claude/kb/experiments/202-GPU与CPU两种模式GC对照组.md:1-8`（整段抄，未转述）**

```markdown
## E202 GPU与CPU两种模式GC对照组 —— 跑前登记已写、装置未跑

**中文** · [English](202-gpu-vs-cpu-gc-comparison.en.md) · [日本語](202-GPUとCPUの2方式GC対照群.ja.md)

跑前登记 `research/prompts/e202-preregistration.md`。回答问题单 `research/prompts/m4-gpugc-compare-questions.md`：比较同一负载、同一池下 `background_compute=cpu` 与 `background_compute=gpu` 两种模式的 GC 挂钟、占用的主机 CPU 时间与省下的时间、前台 fsync 延迟分布，以及省下的 CPU 时间随盘容量的变化。

装置 `crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison/main.rs`（入库装置，住在产品 GPU 的测试包 background-gpu-tests 里）。**这一页只记跑前登记与装置骨架已写，产物一次没跑。**

```

**出处 `.claude/kb/experiments/202-GPU与CPU两种模式GC对照组.md:9-16`（整段抄，未转述）**

```markdown
### 现状

- 跑前登记已写（2026-11-03），装置骨架已立，产物一次没跑。
- 第一段的几何核起过一次、没跑完、没留产物：造起步池太慢，照量到的速度过不了任务服务的四小时上限。量到的：2288MiB 档顺序写那一半约 34 分钟，交错写那一半约 60 秒做 95 步、共 18304 步；8GiB 档 2100 秒做到顺序写 18292 步、共 65536 步；三档同时造，常驻内存 2100 秒时 9.4 GiB。用户定这一次不跑，等造池提速做完再跑。
- 盘容量采用定占用读法（容量涨，活数据占单元区比例固定 50%）。
- 装置约束：GPU 模式每张卡上限 1 GiB、按卡的顺序填满；一张卡与多张卡各跑；判定逐条与 CPU 参照相同才计一组数。
- 多张卡那一臂用几张卡由 `lockstep --cards <张数>` 定（给 0 或 1、或多于服务交来的张数退 2；不给就用服务交来的全部卡）。产物里有两种记卡的行：`name=every_card_arm_cards` 记 `cards_named`、它是怎么定的（`named_by=cards_option` 或 `every_handed_over_card`）与每张卡的 PCI 地址与型号；`name=one_card_arm_card` 记点名一张卡那条臂那一张卡的 PCI 地址与型号，交替跑与并发跑都打。续跑时这几样与要续的产物里已有的同种行有一处不同，或产物里有这条臂跑出的结果行（`name=run`、`name=concurrent_run`）而没有这种行，打 `name=stopped clause=cards_differ_from_the_product_being_resumed` 退 2；一条结果行都没跑出来的那一趟不挡以后的续跑。

```

**出处 `.claude/kb/experiments/202-GPU与CPU两种模式GC对照组.md:17-23`（整段抄，未转述）**

````markdown
### 复跑

```bash
# 入库装置编译与运行归主 agent；经 task-server 调度
task-server/task submit -- cargo run --release -p singlefs-background-gpu-tests --features tests-on-a-real-gpu --bin e202_gpu_cpu_gc_comparison
```

````

**出处 `.claude/kb/experiments/202-GPU与CPU两种模式GC对照组.md:24-32`（整段抄，未转述）**

```markdown
### 口径

- 原始输出未留存：实验未跑。
- 一次整理：一次跑里 GC 线程从第一次调后台入口到最后一次调用返回为止做的全部整理。
- GC 挂钟：GC 线程上每一次后台入口调用前后读 `std::time::Instant` 之差相加。
- GC 的主机 CPU 时间：每次 GC 调用前后读 `clock_gettime(CLOCK_PROCESS_CPUTIME_ID)` 之差减去同一窗口前台线程增量。
- 省下的 CPU 时间：同一档同一次重复里 CPU 臂的进程口径减 GPU 臂的进程口径。
- 重复次数：每档每臂 5 次，报中位、最小、最大与中位绝对偏差。

```

**出处 `.claude/kb/experiments/202-GPU与CPU两种模式GC对照组.md:33-37`（整段抄，未转述）**

```markdown
### 判据与失败条款

- 判据：每一档容量上「CPU 模式减 GPU 模式」的差有数、有正负、有离散度；阳性对照下省下的 CPU 时间必须少于正常对照。
- 失败条款：若 GPU 模式省下的主机 CPU 时间在各容量档位均恒 ≤ 0，或前台 fsync 延迟劣化超限，则触发失败条款。

```

**出处 `.claude/kb/experiments/202-GPU与CPU两种模式GC对照组.md:38-43`（整段抄，未转述）**

```markdown
### 影响的决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D26（后台整理与放置回收） 已定项 13 | 备料 | 2026-11-18 不受影响：这一次只把多张卡那一条的说法照登记修订第 12 条回写（两种记卡的行、续跑核卡的停机分句名），判据、臂与口径没改，产物仍一次没跑。2026-11-14 那一次改的是装置搬进测试包与起法、多张卡那一臂的张数定法，判据、臂与口径没改，产物一次没跑，仍是跑前登记已写、装置未跑 |

```

**出处 `.claude/kb/tooling.md:1-7`（整段抄，未转述）**

```markdown
# 工具与环境事实

**中文** · [English](tooling.en.md) · [日本語](tooling.ja.md)

**tooling.md 放「本工程用到的工具现在是什么状态、用的时候怎么做」**。
规则不进 tooling.md，进 [CLAUDE.md](../../CLAUDE.md) 或 `.claude/rules/`。

```

**出处 `.claude/kb/tooling.md:8-26`（整段抄，未转述）**

```markdown
## 本地 LLM（三方论证不派它当腿）

三方论证只派两条云端腿加主 agent 判决，不派本地模型的腿。仓里调网关的脚本与字词损坏检测器（`ask-local.sh`、`ask-local-selftest.sh`、`local-repro.sh`、`local-repro-tuned.sh`、`corruption-check.py`、`corruption-summary.py`、`oov-check.py`、`en-words.txt`）2026-10-11 删掉了；要再用，`git log --diff-filter=D -- scripts/agents/ask-local.sh` 找到删它的那次提交，从它的上一版取回。

| 项 | 值 |
|---|---|
| 入口 | 本机的本地模型网关（OpenAI 兼容，地址与所在目录不进仓） |
| 鉴权 | 请求头 `X-Api-Key`，取值在本机私有配置里，不进仓 |
| 模型 | `local` → `Qwen3-Next-80B-A3B-Thinking-AWQ-4bit`，`max_model_len` 262144 |
| 调用脚本 | 仓里没有了（取回办法见本节开头那一段） |
| 预算 | 不传 `max_tokens` 与 `thinking_token_budget` |

用它的时候：

- 提示一律用英文，不用 markdown 强调（`**粗体**:`）；不靠 `presence_penalty`、`repetition_penalty` 修中文复读。
- 它给出的具体字段名、结构名、机制描述一律当待核线索；核不动整条丢掉。
- 提示里把最容易上钩的类别错误写出来并禁止：「同一系统在两种介质上」不许答成「两个不同系统」；「宽度在别处定」写明「推迟的是宽度，不是字段本身」。
- 它答「举不出来」时跑阳性对照：同一份提示只把被问对象换成已知答案是「有」的对象，这次举得出来，那个「举不出来」才算数。

```

**出处 `.claude/kb/tooling.md:27-38`（整段抄，未转述）**

```markdown
## 仓里的性能数都只在本机成立，引用要带机器口径

**射程**：挂钟、吞吐、每次发布多少毫秒、门禁跑多久、冷挂载多少秒、并行加速比，凡是时间与速率。

**怎么做**：

- 引一个性能数时把**机器口径**一起带（哪台机、什么盘、什么负载、空不空闲），不带口径的数不许进判据。
- 拿本机的数去定**阈值、上界、超时**时，写明「这是本机值，别的机器上要重量」。
- 需要一个跨机器成立的数时，要么量两台以上，要么写成条件式（「按 X 算是 Y，那一档本机测不到，仍是假设」）。

门禁那一档整环清零的耗时、冷重开恢复的耗时、崩溃点重放两条流的挂钟、E152（按里程碑对比六家文件系统的文件性能） 与六家的全部对比数都是本机数（C486（整环清零的 55 秒与冷重开的 17 秒都没人管））。

```

**出处 `.claude/kb/tooling.md:39-45`（整段抄，未转述）**

```markdown
## 跑项目代码的命令一律经 task-server 交

- 统一任务服务（`task-server/`）是起跑项目代码的任务（编译、测试、门禁、实验、自证；CPU、GPU 都算）的唯一入口：挑机器、内存额度、线程、排队次序都由服务定，提交时不收。服务只调配 CPU 与内存，不分卡、不领卡、不清场：卡归 GPU 批处理验证服务（`gpu-verification-server/`）。用法、判法与拒绝清单在 `task-server/README.md`，不抄第二份。
- 调用方用客户端 `task-server/task` 的七个子命令：`submit [--cwd <目录>] -- <命令…>` 交、打出任务号；`confirm <号> --items <N> --tasks <M>` 核对确认单之后确认（交了的任务先待确认，确认了才进队，不确认的满 30 分钟服务取消），`confirm <号> --why '<…>'` 确认一条跑满两小时的任务确是要跑这么久；`wait <号>` 等到结局（末行 `TASK_SERVER_RESULT …`，退出码是任务的）；`status [<号>]` 看在排、在跑与排队原因；`log <号>` 看最后一次尝试的日志；`cancel <号>` 停；`clean <号>` 删这条任务留下的编译目录（第二台那一趟的编译目录由 `multi-host-run.sh` 跑完自己删，`clean` 不管）；任务自己在跑的那一次尝试里按需领卡、还卡用 `lease take`、`lease return`（只认文件头声明了 `# task-server-card-lease: on-demand` 的入口），调用方不起。命令里不套包装、不写资源参数，写了当场拒（`task-server/README.md`「提交写法」）。
- 子 agent 一律不交任务：`submit`、`wait`、`cancel`、`clean` 只由主 agent 起，子 agent 要跑的写进报告交主 agent（用户 2026-11-01 定，`records/2026-11-17-subagent账本自上而下清欠.md` 第一百七十四节）。
- 出了问题看哪里：结局看 `wait` 末行的 `type=`（错误类型与退出码的表在 `task-server/task_server_lib.py` 文件头「错误类型」）；排着不起看 `task-server/task status <号>` 的「在等：…」；跑挂了看 `task-server/task log <号>`（`--all` 整份）；`wait` 退 78 是连不上服务，先 `task-server/task status <号>` 看任务还在不在，不重交。

```

**出处 `.claude/kb/tooling.md:46-50`（整段抄，未转述）**

```markdown
## 变异工具：并发不超过核数，不限时，卡住由统一任务服务判

- 并发度乘以一条被测单位自己占的核数，不超过核数。cargo-mutants 跑本仓用 `--jobs 1`。
- `scripts/mutation/mutate.sh`、`scripts/mutation/broad-mutation.py`、`scripts/mutation/mutation-prune.py` 不给一条变异或一步设限时，也没有超时这一种结局：整条命令是统一任务服务（`task-server/`）的一个任务，卡住与死循环由服务判、停掉，记 `stalled`、打回提交它的会话（判法在 `task-server/README.md`）。

```

**出处 `.claude/kb/tooling.md:51-61`（整段抄，未转述）**

````markdown
## 变异条数会随源码漂，不许在 kb 里钉死

正文里写数它的命令，不只写数：

```
cargo mutants --list --package singlefs-checker | wc -l
cargo mutants --list --package singlefs-core | wc -l
```

要引一个具体的数，连同量它的日期写成事件句（「<日期> 数出 N 条」），不写成现状句。

````

**出处 `.claude/kb/tooling.md:62-65`（整段抄，未转述）**

```markdown
## 改了引用方，要 grep 全仓改被引用方

改正一处措辞之后，说「已全部改完」之前，先 `grep -rn` 全仓数一遍。范围含 `kb/` 全部子目录、`research/prompts/`（喂给三方论证的背景材料）、`scripts/` 与 `research/` 下的源码注释、`records/`、门禁脚本。数不清就写「改了这几处，没有全仓核过」。

```

**出处 `.claude/kb/tooling.md:66-70`（整段抄，未转述）**

```markdown
## 一个数不许从「它自己判为非法」的配置里算出来

- 算一个上限或下限之前，先确认那个配置在本工程的判定下是合法的。
- 撤回一个数时，作废理由写成真实的那一条（缺输入、配置非法、口径错），不写成它保留下来的别的档同样有的性质。

```

**出处 `.claude/kb/tooling.md:71-74`（整段抄，未转述）**

```markdown
## 未定项一律写成编号条目，标题一律声明条数

按 `^\d+\.` 提取的脚本只认编号条目；`.claude/gate.d/doc-decisions.sh` 的 kb-shape 格的条数检查只在标题声明了「N 项未定」时启动。

```

**出处 `.claude/kb/tooling.md:75-87`（整段抄，未转述）**

```markdown
## 验证产物放哪：四个位置，各管一段

| 位置 | 放什么 | 谁读 | 进仓库 |
|---|---|---|---|
| `.claude/kb/experiments.md` | 结论、口径、判据、复现命令、**判决性的那几行数字** | 模型检索 | 进 |
| `records/` | 每一轮的叙述：定了什么、推翻了什么、依据 | 人通读 | 进 |
| `scripts/` | 实验代码与脚本 | 跑的人 | 进（`.gitignore` 只排除 `target/` 与个别超大文件） |
| `research/results/<实验>-<日期>.out` | 原始输出全文 | 下一轮复跑时逐行比对 | 进；早于最近一轮的在下一次提交时归档进 git 历史（门禁 doc-experiments 的 archive-past-rounds 格） |

- 判决性的数字贴进 kb 正文。
- 每份原始输出要能自证是产出它的那一次运行留下的：收尾行 `E7RESULT name=done emitted=N` 里的 N 等于文件里 `E7RESULT` 的行数，对不上整轮作废。归档时核一遍：
  `for f in research/results/*.out; do echo "$(grep -c '^E7RESULT' $f) vs $(grep 'name=done' $f)"; done`

```

**出处 `.claude/kb/tooling.md:88-100`（整段抄，未转述）**

```markdown
## 内核源码：本机有完整树，不必 WebFetch

| 项 | 值 |
|---|---|
| 路径 | `~/kbuild/linux-om`（1.8 GB） |
| 版本 | **Linux 7.2.0**（`Makefile` 的 `VERSION = 7` / `PATCHLEVEL = 2` / `SUBLEVEL = 0`） |
| 有什么 | 完整 `.c`/`.h` 正文，含 `fs/jbd2/`、`fs/xfs/`、`fs/btrfs/`、`fs/erofs/`、`fs/ext4/`、`fs/f2fs/`、`drivers/nvme/`、`drivers/md/`、`crypto/`、`Documentation/` |
| **没有什么** | **`fs/bcachefs/` 不在这棵树里**（见 [decisions.md](decisions.md) D7（是否进 Linux 主线））。`/usr/src/linux-headers-6.17.0-1028-oem/fs/bcachefs/` 只有 `Kconfig` + `Makefile`，**没有源码** |

- 查源码用 `~/kbuild/linux-om`，不用 `/usr/src/linux-*-headers-*`（那里只有 `Kconfig` 与 `Makefile`）。
- 引哪棵树就标哪个版本号，两个版本的结论不混着引。kb 里标「现查 Linux 6.17」的引用来自 GitHub 的 `v6.17` tag，本机没有那个版本。
- OpenZFS 本机没有树，现取 `https://raw.githubusercontent.com/openzfs/zfs/master/...`，引用时记下取回日期，行号不是永久的。

```

**出处 `.claude/kb/tooling.md:101-106`（整段抄，未转述）**

```markdown
## 到点自己开工：用 remind.sh 登记

用户要会话到点自己开工时，只用 `bash scripts/agents/remind.sh start --every <分钟> --message <提醒内容> --once`（Bash 的 `run_in_background` 起，到点退出叫醒会话；规则 `.claude/rules/timer-only-remind.md`，写法 `.claude/skills/remind/SKILL.md`）；不用 CronCreate，也不在 Bash 里只为定时而写 `sleep`。提醒内容写开工指令所在的路径，第一步用 `set -o noclobber` 排他建「已开工」标记，标记已在就什么都不做（`remind.sh` 重起时可能重复提醒）。

- 提醒只活在当前会话里，告诉用户会话不能关、机器不能休眠。

```

**出处 `.claude/kb/tooling.md:107-117`（整段抄，未转述）**

```markdown
## Rust 工具链

| 项 | 值 |
|---|---|
| 版本 | rustc 1.98.0 / cargo 1.98.0 |
| 安装方式 | rustup，`--profile minimal --default-toolchain stable`，装在宿主 `~/.cargo` |
| 额外目标 | `x86_64-unknown-linux-musl`，实验二进制（`scripts/experiment-apparatus/` 下的包）编成静态的在虚机的 busybox initramfs 里跑；验证包的真设备二进制 `new_pool_file_creation_on_device` 不走它（验证包默认编 RocksDB 的 C++，本机没有 musl 的 C/C++ 编译器），改用 `--target x86_64-unknown-linux-gnu` 加 `+crt-static` 编成 static-pie，命令在 `.claude/gate.d/qemu-device-streams.sh` |
| 组件 | `rustfmt` + `clippy`（`--profile minimal` 不含它们，要另装） |
| **PATH** | rustup 的 shim 软链在 PATH 上的用户级可执行文件目录里（cargo / rustc / rustup / rustdoc / rustfmt / cargo-fmt / cargo-clippy）；门禁与脚本用 `bash -c` 起，不读 shell 的启动文件，PATH 不靠改 dotfile |
| release 溢出检查 | 树根 `Cargo.toml` 与 `scripts/experiment-apparatus/Cargo.toml` 的 `[profile.release]` 都设了 `overflow-checks = true`；profile 只认工作区根，写在成员 manifest 里不生效 |

```

**出处 `.claude/kb/tooling.md:118-145`（整段抄，未转述）**

```markdown
## RocksDB 与 snappy 的预编库：每台主机编一次，取值只走进程环境

librocksdb-sys 的 C++（`librocksdb.a` 与 `libsnappy.a`）每台主机只编一次，各编译目录经环境变量链这一份，不在每个编译目录里现编。工具是 `scripts/hosts/rocksdb-prebuilt.py`，用法写在它的文件头（它不认 `--help`，退出码 2）。子命令都在仓根跑，`--root R` 不给就是这份脚本所在的仓。

| 子命令 | 做什么 |
|---|---|
| `key` | 打这台主机此刻的键（各组成）与预编库目录，不编 |
| `env` | 预编库在、对得上时打四行 `export`（给 `eval` 用）；不在退 1、标准输出为空 |
| `build` | 预编库不在就编一次，持锁：几个进程同时来只编一次，后来的等锁之后看到编好了直接退 0；起真 cargo，是重活，经 `task-server/task submit -- python3 scripts/hosts/rocksdb-prebuilt.py build` 交（`task-server/README.md`） |
| `install` | `build`，再写这台主机的三处环境来源、改用户级 systemd 管理器此刻的环境，最后跑 `check`；同样是重活 |
| `check` | 判这台主机：两个工作区（`crates/singlefs-verification/Cargo.toml` 与 `scripts/experiment-apparatus/Cargo.toml`）的依赖对得上、预编库在且对得上、三处环境来源与此刻进程环境的取值都与预编库一致、cargo 配置里没写这四个变量 |
| `audit-target <编译目录>…` | 判编译目录：librocksdb-sys 构建脚本那一次拿到的取值是不是这台的预编库、`out/` 里有没有现编过的 C++、同一把键下见过几种取值；一份构建脚本记录都没找到退 77（本次未判） |
| `entry [--cargo C]` | 起 cargo 的入口开头调的轻判（谁调、取值缺了怎么办写在「起 cargo 的入口」那一段） |

**预编库放哪**：`${SINGLEFS_ROCKSDB_PREBUILT_STORE:-~/.local/share/singlefs/rocksdb-prebuilt}/librocksdb-sys-<版本>/<键的前 16 位>/lib/{librocksdb.a,libsnappy.a}`；同目录 `manifest.json` 记键的各组成与两份库的 sha256，`complete` 最后写（有它才算编好）；不放开机清空的目录。`SINGLEFS_ROCKSDB_PREBUILT_STORE` 只给自证换临时目录用。键由两个工作区的 rocksdb 依赖写法、两份 `Cargo.lock` 里相关包的版本与 checksum、`rustc -vV`、c++ / cc / ar 的真实路径与版本、交给 C++ 的指令集、编法组成，哪一样变了键就变、要重编。

**取值从哪来**：四个变量 `ROCKSDB_LIB_DIR`、`ROCKSDB_STATIC`、`SNAPPY_LIB_DIR`、`SNAPPY_STATIC` 的取值写在三处环境来源里，都由 `install` 写：

| 位置 | 作用 |
|---|---|
| `~/.config/singlefs/rocksdb-prebuilt.env` | 唯一的取值来源：四行 `export`，外加 `unset ROCKSDB_COMPILE`、`unset SNAPPY_COMPILE`；别手改，改了重跑 `install` |
| `~/.bashrc` 最前面一段（三行，`# >>> singlefs rocksdb-prebuilt >>>` 到 `# <<< singlefs rocksdb-prebuilt <<<`） | `source` 那份 env 文件；放在交互判断之前，所以交互 shell、从它起的会话与 ssh 起的非交互命令都读得到 |
| `~/.config/environment.d/60-singlefs-rocksdb-prebuilt.conf` | 用户级 systemd 管理器起的单元读（task-server 为每一次尝试起的单元，`task-server/README.md`）；`install` 另跑 `systemctl --user set-environment`，让此刻在跑的管理器当场换上 |

**四个变量只走进程环境**：不写进 `.cargo/config.toml` 或别的哪一层 cargo 配置的 `[env]`（写了 `check` 判红；理由在 `.cargo/config.toml` 开头的注释）。`ROCKSDB_COMPILE`、`SNAPPY_COMPILE` 一律不设，设成 1 就在编译目录里现编。

**起 cargo 的入口**：门禁阶段、崩溃与变异驱动、`merge-and-verify.sh`、`admission.py` 交的命令、第二台那一侧，开头都调 `entry`。它只判三样：这台装了（预编库在、对得上，不算两份库的 sha256）、此刻进程环境的四个取值与预编库一致、没设 `ROCKSDB_COMPILE` 与 `SNAPPY_COMPILE`；不判三处环境来源（那是 `check` 的事）。取值缺了或对不上，入口当场拒、一个 cargo 都不起，逐项写出出路。出路：此刻进程没有取值的，开新会话（`install` 之后起的登录 shell 读 `~/.bashrc` 最前面那一段），或在命令前 `eval "$(python3 scripts/hosts/rocksdb-prebuilt.py env)"`；这台还没装的先交 `task-server/task submit -- python3 scripts/hosts/rocksdb-prebuilt.py install`（`task-server/README.md`）。要编的那棵树里没有依赖 rocksdb 的工作区时（编不到 librocksdb-sys，门禁样本与自证的小仓多是这样），`entry` 不判、退 0。

```

**出处 `.claude/kb/tooling.md:146-160`（整段抄，未转述）**

```markdown
## 虚机 benchmark harness（E7（离线索引 harness） 用）

| 项 | 值 |
|---|---|
| 脚本 | `bash scripts/devices/vm-bench.sh <静态二进制>` / `--selftest` |
| 它是什么 | 项目自己的虚机装置：给虚机挂 virtio 盘（稀疏文件落 `TMPDIR`）并把二进制塞进 initramfs |
| 完整性闸 | 被测程序收尾行报 `name=done emitted=N`，宿主抓到的条数必须等于 N，否则整轮作废 |
| 默认规格 | 2048M 内存 / 4 vCPU / 4096M 盘 / 900s 超时，均可用环境变量覆盖 |

- 抓结果行不锚定行首（控制台首行前面有 BIOS 的转义序列），条数靠完整性闸核。
- 虚机里裸设备的顺序写吞吐不稳定，不作为任何判断的输入。
- 跑批脚本作废一轮时，把该轮输出的尾部一并记下。

---

```

**出处 `.claude/kb/tooling.md:161-197`（整段抄，未转述）**

````markdown
## 造几何量不同的设备：不需要真硬件

本机具备造异构设备池的全部手段，[decisions.md](decisions.md) D12（目标介质） 已定项 3「设备级几何量该放哪、异构池支不支持」可以实测。

### QEMU 路线（推荐：无需 root，直接接现有 harness）

`zoned.zone_size` 与 `logical_block_size` 挂在 **`nvme-ns`（命名空间）** 上，不是挂在控制器上，同一个控制器下的多个命名空间可以有不同的几何量。

已实测通过（QEMU 8.2.2，退出码 0，三个设备全部构造成功）：

```
-device nvme,serial=A,id=nv0
-device nvme-ns,drive=d1,bus=nv0,nsid=1,zoned=true,zoned.zone_size=64M,logical_block_size=4096,physical_block_size=4096
-device nvme-ns,drive=d2,bus=nv0,nsid=2,zoned=true,zoned.zone_size=128M,logical_block_size=4096,physical_block_size=4096
-device nvme-ns,drive=d3,bus=nv0,nsid=3,logical_block_size=512,physical_block_size=512
```

一条命令同时造出：**异构 zoned 池**（64M vs 128M zone）、**512e/4Kn 混用**、**zoned 与常规混用**。

- `logical_block_size` 必须配 `physical_block_size`，只给前者会被拒：`logical_block_size > physical_block_size not supported`。
- 来宾内核看不看得到三份不同的几何没有验过：要进来宾读 `/sys/block/*/queue/{logical_block_size,chunk_sectors}` 与 `blkzone report`。

### 宿主侧路线（要 root，胜在不用启虚机）

| 手段 | 造什么 | 关键参数 |
|---|---|---|
| `null_blk` | zoned 设备，内存后端 | `zoned=1 zone_size=<MB> zone_capacity zone_nr_conv zone_max_open`。模块参数是全局的，**每设备各自的几何要走 configfs**（`/sys/kernel/config` 本机已挂载） |
| `zloop` | **文件后端**的 zoned 设备 | 模块在 `drivers/block/zloop.ko.zst`，本机有 |
| `scsi_debug` | 任意扇区大小 + ZBC | `sector_size=<bytes> zbc=host-managed zone_cap_mb zone_nr_conv` |

运行内核 6.17.0-1028-oem 的相关配置：`CONFIG_BLK_DEV_ZONED=y`、
`CONFIG_BLK_DEV_NULL_BLK=m`、`CONFIG_BLK_DEV_ZONED_LOOP=m`、`CONFIG_SCSI_DEBUG=m`。

⚠️ 本机唯一的真实块设备是 `nvme0n1`（3.6T，512/512，非旋转）——
**没有真的旋转盘、没有真的 4Kn 盘、没有真的 zoned 盘**，所有相关数字都只能是模拟出来的，
引用时必须带这个口径。

````

**出处 `.claude/kb/tooling.md:198-203`（整段抄，未转述）**

```markdown
## `scripts/mutation/mutate.sh` 的结局分类

- 结局：`✅ [抓到]`（红了）/ `❌ [没红]`（测试盲区）/ `⏭ 变异导致编译失败（无效变异，不计入盲区也不算命中）`；另有 `💥`（测试进程没跑完）、`⚠️`（测试进程报了失败，却一个失败的测试名都没抓到：抓名字的规则有盲区）与锚点失效（`⚓ [锚点腐化]`，原文命中不是恰好一次的那一条不跑、不入判定库，别的条照跑，整轮退 3）几类，各自单列；收尾「计数：」一行报抓到、没红、无效、没跑完、抓名字有盲区、锚点失效六栏（全部结局见 `scripts/mutation/mutate.sh` 文件头与 `--selftest`）。
- 上限与限时：每次 cargo test（基线、每条变异、还原之后那一次）直接起，不设内存上限、不限时；整条 `mutate.sh` 交统一任务服务（`task-server/`），一整条一个任务：内存上限由服务按记录给、撞了放大重排；死循环的变异由服务停掉整条任务（用时远超记录而日志又满 10 分钟不涨就停掉、打回）（`scripts/mutation/mutate.sh` 文件头「上限、限时与卡住」）。
- 改了判定工具，拿已知答案的整套语料回归一遍，不只跑新加的那几条。

```

**出处 `.claude/kb/tooling.md:204-217`（整段抄，未转述）**

```markdown
## 几个会话共用一个仓时怎么暂存与提交

门禁红了先分清红的是谁的改动，再动手；不为了让门禁变绿去改别人在飞的文件。规矩在上游 `session-wrapup.md` 第 4 节，本仓的做法：

| 事 | 怎么做 |
|---|---|
| 只暂存自己的块 | `scripts/gate/stage-mine.py --match 我的标记 --foreign 别人的标记 文件…`，先 `--dry-run` 看它挑了哪几块 |
| 暂存区也是共用的 | 暂存前先确认 `git diff --cached --name-only` 为空；暂存完、提交前再核一遍名单与逐文件 cached diff，然后不带路径地 `git commit` |
| 不要 `git commit -- <路径>` | 带路径时 git 提交的是这些路径在工作区里的内容，不是暂存区 |
| 同一行两边都改了 | stage-mine 报「进暂存区的一块里有命中 --foreign 的行」时一行切不开：对方在那份文件里只做了可逆替换（改名）的，拿工作区版本把对方的替换反做一遍、与 `git show HEAD:文件` 比 diff 确认只剩自己的，再 `git hash-object -w` + `git update-index --cacheinfo 100644,<sha>,<路径>` 放进暂存区，工作区不动 |
| 在 worktree 上提交 | 在 worktree（HEAD 上）用脚本重放这次要提交的全部改动 → 与主工作区逐文件比对（不同的部分应当全是对方的；比对时 `git -c core.quotePath=false`，否则中文路径被转义、`cmp` 全报不同）→ 在 worktree 跑整道门禁并提交 → 主仓里 `git update-ref refs/heads/master <新提交> <旧 HEAD>` 再 `git read-tree <新提交>`，工作区不动、对方的改动原样留着 |
| 看清哪些不是这次要提交的 | `git -c core.quotepath=false status --short`，中文路径默认被转义成八进制，grep 中文关键词会漏 |
| 两边改动交织在同一文件 | 一个提交带全部，消息里写明（拆开提交会让门禁 doc-registries 的 experiment-refs 格与门禁 doc-experiments 的 experiment-orphans 格判红） |

```

**出处 `.claude/kb/tooling.md:218-223`（整段抄，未转述）**

```markdown
## 复跑与变异脚本要在 `scripts/experiment-apparatus/` 下跑

- `scripts/mutation/mutate.sh` 在 `scripts/experiment-apparatus/` 下跑：`cd scripts/experiment-apparatus && bash ../mutation/mutate.sh <bin 名> <源文件> ../mutation/tables/<变异表>`。
- `replay.sh` 只认带 E 的实验号（`E` 加数字），裸数字退出码 2。
- `replay.sh` 每一趟编进它在 `${TMPDIR:-/tmp}/singlefs-replay-build-<进程号>-XXXXXX/target/` 自己建的编译目录、跑完删（调用方设了 `CARGO_TARGET_DIR` 的照它编、不删），研究二进制从 `$REPLAY_RELEASE_DIRECTORY`（`$CARGO_TARGET_DIR/release/`）起；`scripts/experiment-apparatus/target/` 不再有复跑编出来的二进制。

```

**出处 `.claude/kb/tooling.md:224-243`（整段抄，未转述）**

```markdown
## 批量立 / 定分项时门禁会红的形态

一次立或定一批分项之后，提交前门禁里这几格会按下列形态红，写的时候照「写法」一列写：

| 门禁 | 会红的形态 | 写法 |
|---|---|---|
| doc-registries 的 first-txn-hooks 格（三份文件挂钩） | 按 `\| N \|` 行切未定项条目，最后一行把后面所有 `#### 未定项 N` 正文吞进自己那一条；正文里写「改新池新建文件的字节：是（」，判「否」的最后一行也被当成「是」 | 索引行用规范形态 `改新池新建文件的字节：是（日期，依据：…）`，正文小节写成 `**改新池新建文件的字节**：是（…）` |
| doc-decisions 的 item-ref-status 格（引用归属） | 行内裸的「已定项 k」归属给同一行最近的 `D<n>（简称）` | 新立分项之后跑 doc-decisions 的 item-ref-status 格，把翻掉的裸引用补上决策名；决策正文里引别的决策的分项，同一行里要重复写自己的决策名 |
| doc-lint | `F` 加数字这类审核编号被当成没有登记位的编号；简称后面加逗号注判「名字不符」 | 写「发现 1」「提议 2」；括注另起一对括号，写成「编号（简称）（条款已写）」，不塞进简称的括号里 |
| doc-registries 的 table-shape 格（欠账表形状） | 新的六列欠账行接在文件末尾，而那里是「已还清」那张 4 列表；欠着那张里一格开头写「已还（…）」或「销账」；欠着那张中间夹一个空行 | 接在第一张表最后一行后面：第一个五竖线的 `\|---\|` 之前、最后一行以 `\| C` 开头且竖线 ≥ 7 的行；还清了就按四列挪进已还清表，只还一部分的开头写「已还一半」「决策已定、检查仍欠」「前置已还」；表中间不留空行 |
| doc-registries 的 segment-registry 格（段序列登记表） | `--selftest` 要「mkfs 种根」那一行的段序列串在整份 `layout/01-first-txn.md` 里只出现一次，同一串在别处再写一遍就红 | 别处引用写成「4 + 1 + 1 + 1 + 4」带空格的写法 |
| 门禁 doc-decisions 的变更史三格（entry-added、shape、history-loss） | 「（其N）」、快查两行与节顶现状行 | 写法与判法在 `.claude/rules/changelog-format.md`；数与编号有没有丢由 history-loss 格判，整理变更史时暂存前自己先跑 `scripts/kb/history-consolidate.py loss-check` |
| code-source-discipline 的 format-constants 格（格式常量） | kb 标记改值后，`scripts/experiment-apparatus/**/*.rs` 与 `crates/**/*.rs` 里每一份同名 `const` 都要跟（`research/` 下只剩冻结证据，扫它但由 `.claude/format-constants-exclude` 逐份放行）；`stale=` 列的旧值串不许再出现在 kb 正文 | 别把仍在别处合法出现的串（`base = 95`）加进 `stale=` |
| clippy `shadow_unrelated` | 同一个测试函数里第二次 `let reader = …` / `let segments = …` 判红，rustfmt 重排之后用正则替换还会漏改后半段 | 每个读者 / 段 / 报告起不同的名字（`extent_reader`、`warm_up_segments`、`control_report`） |
| naming-lint | 「字母 + 数字」当单字母：`read_u48` 判红；结尾的 `a` 与只有一段的 `a` 判红（`side_a`），打头与中间的 `a` 后面还有别的段时当英语冠词、不判（`a_mount_that_stays_short`）；它扫全仓的 .rs，连 `research/prompts` 里云端腿写的模型源码也扫 | 写 `read_six_byte_unsigned`；`research/prompts` 在 `.claude/naming-lint-exclude`（证据目录，不回收），腿的模型写到别的目录照查 |
| crash-point-replay（崩溃点重放） | 崩溃枚举用例住 `crates/singlefs-verification/tests/`、登记成 `.claude/gate.d/stage-inputs.tsv` 的 `crash-case:` 行、不标 `#[ignore]`（D13（验证路线） 已定项 15） | 交门禁 crash-point-replay：按条那一趟经 `--skip` 排掉登记的用例，登记的用例逐条按崩溃状态块的单元键判（块键相同的复用、缺的当场判；登记行第三列带 `separate-run=` 的用例没被点名时不跑、报本次未跑）；在跑时换 `CARGO_TARGET_DIR` 就能同时编别的测试 |
| 变异证明 | rustfmt 之后多行调用的整串匹配「没命中」 | 按「找到含关键串的行、删到以 `);` 收尾的那一行」做；改完复原并 `assert` 文件逐字节等于原文 |
| checker 新判定 | 只在对象已经过了同一关之后才判的判定永远判不红 | 每条新判定配一份坏镜像（`checker_known_bad_images.rs` 的形态），不许只看干净镜像全绿 |
| 批量定点替换 | 逐条写盘会在一处锚点不中时留下半套改动 | 用 `scripts/kb/replace-batch.py`：全部替换先在内存里核「恰好命中 1 次」，全过了才写盘 |

```

**出处 `.claude/kb/tooling.md:244-254`（整段抄，未转述）**

```markdown
## 本机 Claude Code 会话里的 `grep` 与 `find` 不是 GNU 那两个

`type grep`、`type find` 显示两个都是 shell 函数：`grep` 底下是 ugrep，`find` 底下是 bfs（报错分别以 `ugrep: error`、`bfs: error` 开头）。两处行为与 GNU 不同：

| 写法 | 结果 | 改成 |
|---|---|---|
| `grep -o` 带 `.\{0,90\}` 这种有界重复，作用在中文行上 | `exceeds complexity limits`，退出码 2，一行都不输出 | 用 python 按下标切：`line[max(0, i - 90):i + 90]` |
| `find 目录 -newermt '-40 minutes'` | `Invalid timestamp`，不认相对时间 | 写 ISO 日期：`-newermt '<YYYY-MM-DD>'` |

两种失败的样子都是「这条命令没给出结果」，接在管道或 `||` 后面时不要读成「没有命中」。

```
