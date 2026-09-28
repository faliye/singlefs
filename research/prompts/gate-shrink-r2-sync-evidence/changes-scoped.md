# 变更清单 HEAD..工作区

## 变更记录（事实表的「出处」列要罩住每一个 H 编号）

| 编号 | 文件 | 标题 |
|---|---|---|
| H1 | .claude/kb/layout/01-first-txn.md | 2026-09-28（八节回退段序列手算钉住） |
| H2 | .claude/kb/layout/01-first-txn.md | 2026-09-28（八节段序列跟上发布末尾屏障） |
| H3 | .claude/kb/milestone/03-third-txn.md | 2026-09-28 |

## 现状载体里改掉的片段

### .claude/agent-common.md

- 旧：cipline.md`「说人话」一节；说外部状态之前现
  新：cipline.md`「文风要简单自然」一节；说外部状态之前现
- 旧：- 本机时钟是 UTC，人在东京（JST，UTC+9）；报告里的时刻写清是哪个时区。
  新：同一个仓里干活：遇到同时需要修改的内容协商处理。
- 旧：，UTC+9）；报告里的时刻写清是哪个时区。
  新：干活：遇到同时需要修改的内容协商处理。
- 旧：02BA）；全仓由门禁 12 号判，写法见 `.cl
  新：02BA）；全仓由门禁 doc-text 的 prime-mar
- 旧：上一次那批，本轮的留着（`.claude/gate.
  新：上一次那批，本轮的留着（门禁 doc-experiment
- 旧：那批，本轮的留着（`.claude/gate.d/91-
  新：本轮的留着（门禁 doc-experiments
- 旧：的留着（`.claude/gate.d/91-arch
  新：的留着（门禁 doc-experiments 的 archive
- 旧：`.claude/gate.d/91-archive-past
  新：c-experiments 的 archive-past
- 旧：-past-rounds.sh` 判这一条）。
  新：-past-rounds 格判这一条）。
- 旧：gate.sh` 整轮、87 号全部实验复跑、E152
  新：gate.sh` 整轮、全部实验复跑（门禁 ch
- 旧：ippy / build 与 `.claude/gate.d/` 下 54、55、57、59、87 之外的阶段（15、74 号虽然也跑
  新：ippy / build（逐条的判法在 `.cl
- 旧：59、87 之外的阶段（15、74 号虽然也跑 `cargo test`，按轻阶段对待；逐条的判法在 `.cla
  新：ppy / build（逐条的判法在 `.cla
- 旧：fier` 只跑 54、55、57、59 号那几道，
  新：er0-replay`、checker-tier-qemu-device-streams、checker-tie
- 旧：r` 只跑 54、55、57、59 号那几道，`ga
  新：ice-streams、checker-tier-lkmm、checker-tie
- 旧：只跑 54、55、57、59 号那几道，`gate-
  新：r-tier-lkmm、checker-tier-crates-mutation-replay 这四道，`gate-t
- 旧：54、55、57、59 号那几道，`gate-tria
  新：tion-replay 这四道，`gate-tria
- 旧：ate.sh` 整轮与 87 号，都在提交时或用户要
  新：ate.sh` 整轮与 experiment-replay 格，都在提交时或用户要
- 旧：lay.sh` 与门禁 59 号在里面逐条套了，门禁
  新：lay.sh` 与门禁 checker-tier-crates-mutation-replay 在里面逐条套了，门禁
- 旧：号在里面逐条套了，门禁 15、74 号在阶段里面经包装（派
  新：在里面逐条套了，门禁 checker-tier-research-build-and-replay 的 research-
- 旧：设进阶段文件头写的变量：15 号 `GATE_RES
  新：设进阶段文件头写的变量：research-unit-tests 格 `GATE_RES
- 旧：MEMORY_MAX`，74 号 `GATE_MODEL
  新：MEMORY_MAX`，harness-model-differential-and-scenarios `GATE_MODEL
- 旧：的会话在同一个仓里干活：只动这一轮自己的文件；看到别人没提交的
  新：的会话在同一个仓里干活：遇到同时需要修改的内容协商处理。
- 旧：里干活：只动这一轮自己的文件；看到别人没提交的改动不碰、不修。
  新：干活：遇到同时需要修改的内容协商处理。
- 旧：成立，把看到的命令、输出与时刻写进草稿目录里的进度记录
  新：成立，把看到的命令与输出写进草稿目录里的进度记录
- 旧：- 哪个门禁阶段该由谁在干完自己
  新：这一轮提交之后由下一次提交删掉上一次那批，本轮的留着（门禁 doc-exper
- 旧：- 哪个门禁阶段该由谁在干完自己的活之后先跑，登记在 `.claude/ga
  新：次那批，本轮的留着（门禁 doc-experim
- 旧：己的活之后先跑，登记在 `.claude/gate.
  新：那批，本轮的留着（门禁 doc-experiment
- 旧：之后先跑，登记在 `.claude/gate.d/sta
  新：本轮的留着（门禁 doc-experiments
- 旧：登记在 `.claude/gate.d/stage-owners.tsv`（第二列
  新：的留着（门禁 doc-experiments 的 a
- 旧：/stage-owners.tsv`（第二列是 agent 名，逗号分隔）。
  新：门禁 doc-experiments 的 archi
- 旧：（第二列是 agent 名，逗号分隔）。列出登记给你的：
  新：experiments 的 archive-past-rounds 格判这一条）。
- 旧：ent 名，逗号分隔）。列出登记给你的：
  新：unds 格判这一条）。
- 旧：`awk -F'\t' -v me=<你的名字> '$1 !~ /^#/ && NF == 3 { n = split($2, owners, ","); for (i = 1; i <= n; i++) if (owners[i] == me) print $1 }' .claude/gate
  新：给人看的文字照 `.claude/sing
- 旧：ate.d/stage-owners.tsv`
  新：rify-before-claiming.md` 开头一节）
- 旧：.d/stage-owners.tsv`
  新：fore-claiming.md` 开头一节）。
- 旧：tage-owners.tsv`
  新：re-claiming.md` 开头一节）。
- 旧：逐个 `nice -n 19
  新：02BA）；全仓由门禁 doc-text 的 prime-marks 格判，写法见 `.claude/ru
- 旧：逐个 `nice -n 19 bash .claude/gate
  新：rks 格判，写法见 `.claude/rule
- 旧：laude/gate.d/<文件>`，贴每个的原样末行与退
  新：ath-moves.md`「变体起新名字，不用角
- 旧：gate.d/<文件>`，贴每个的原样末行与退出码。其中重型的那几道（54、55、57、59、87）照「不做」一节重型测试那一
  新：th-moves.md`「变体起新名字，不用角标
- 旧：、57、59、87）照「不做」一节重型测试那一条跑：只在提交时或用户要求时、命令带前缀；定义另有写法的照定义（`gate-triage` 登记的阶段都在整轮门禁里跑过，不单跑）。
  新：h-moves.md`「变体起新名字，不用角标」。
- 旧：段都在整轮门禁里跑过，不单跑）。
  新：md`「变体起新名字，不用角标」。
- 旧：- 提交前的整轮门禁归 `gate-tr
  新：交删掉上一次那批，本轮的留着（门禁 doc-exper
- 旧：- 提交前的整轮门禁归 `gate-triage`，
  新：那批，本轮的留着（门禁 doc-experiments 的 archive-past
- 旧：门禁归 `gate-triage`，不归你；表里没登记给你的阶段不用跑。
  新：chive-past-rounds 格判这一条）。
- 旧：、还来不来得及拷。门禁 69 号判这一条的形式：装置
  新：、还来不来得及拷。门禁 doc-experiments 的 evidence-

### .claude/agents/crash-verifier.md

- 旧：不算做完」第 3 步与「重型测试只在提交时跑」里最重的那
  新：不算做完」第 3 步与「checker 档只在提交时跑，harne
- 旧：重型测试只在提交时跑」里最重的那几道。
  新：arness 随时跑」里提交时跑的这四道门禁（文件是 `
- 旧：试只在提交时跑」里最重的那几道。
  新：s 随时跑」里提交时跑的这四道门禁（文件是 `.cl
- 旧：- 54、55、57 号各自的内存上限
  新：er0-replay`、checker-tier-qemu-device-streams、checker-tie
- 旧：- 54、55、57 号各自的内存上限（第
  新：ice-streams、checker-tier-lkmm 各自的内存上限（第 1
- 旧：上限（例 16G）：54、57 号的写明是量过的（峰值
  新：限（例 16G）：`54-layer0-replay` 与 checker-t
- 旧：的线程数相同）还是推的；55 号的不小于同时起的虚机
  新：的线程数相同）还是推的；checker-tier-qemu-device-streams 的不小于同时起的虚机数
- 旧：aude/gate.d/55-qemu-device
  新：aude/gate.d/checker-tier-qemu-device
- 旧：起 54 号，等它结束。55、57、59 在主工作区
  新：eplay`，等它结束。checker-tier-qemu-device-streams、checker-tie
- 旧：4 号，等它结束。55、57、59 在主工作区跑，判
  新：ice-streams、checker-tier-lkmm、checker-tie
- 旧：，等它结束。55、57、59 在主工作区跑，判的是工
  新：r-tier-lkmm、checker-tier-crates-mutation-replay 在主工作区跑，判的是工
- 旧：要跑的再取它自己的输入（55、59 号是 `.cla
  新：要跑的再取它自己的输入（checker-tier-qemu-device-streams、checker-tie
- 旧：再取它自己的输入（55、59 号是 `.claude
  新：ice-streams、checker-tier-crates-mutation-replay 是 `.claude/
- 旧：v` 里它那一行的路径，57 号是 `litmus
  新：v` 里它那一行的路径，checker-tier-lkmm 是 `litmus c
- 旧：跟踪文件要查的路径>`（55、59 号查 `crat
  新：跟踪文件要查的路径>`（checker-tier-qemu-device-streams、checker-tie
- 旧：件要查的路径>`（55、59 号查 `crates`
  新：ice-streams、checker-tier-crates-mutation-replay 查 `crates`，
- 旧：号查 `crates`，55 号按名字读的 `res
  新：查 `crates`，checker-tier-qemu-device-streams 按名字读的 `rese
- 旧：里不查，由整轮门禁里的 87 号兜；57 号查 `c
  新：nd-replay 的 experiment-replay 格兜；checker-
- 旧：轮门禁里的 87 号兜；57 号查 `crates
  新：t-replay 格兜；checker-tier-lkmm 查 `crates l
- 旧：tic-only` 时 57 号只跑静态那几层；要没
  新：tic-only` 时 checker-tier-lkmm 只跑静态那几层；要没有
- 旧：，别处的未跟踪文件不挡；57 号读的 `.claud
  新：，别处的未跟踪文件不挡；checker-tier-lkmm 读的 `.claude
- 旧：1b. 54、55、57 号整条经内存包装
  新：er0-replay`、checker-tier-qemu-device-streams、checker-tie
- 旧：1b. 54、55、57 号整条经内存包装跑（共
  新：ice-streams、checker-tier-lkmm 整条经内存包装跑（共用
- 旧：ate.d/<文件>`；59 号在里面逐条套了，74
  新：ate.d/<文件>`；checker-tier-crates-mutation-replay 在里面逐条套了，照第
- 旧：59 号在里面逐条套了，74 号在阶段里面经包装，这两道照第 2 步直接跑，外面
  新：lay 在里面逐条套了，照第 2 步直接跑，外面
- 旧：转达用户要求时）跑你那几道，不跑 `gate.sh
  新：之后、提交之前跑门禁」那一行点名的相同，不从阶段归属表取；不跑
- 旧：uard.sh` 拒）。按共用约束 `.claude/ag
  新：uard.sh` 拒）。checker-tier-qemu-device-streams、checker-tier-lkmm 与 `54-layer0-
- 旧：` 拒）。按共用约束 `.claude/agent-
  新：ier-lkmm 与 `54-layer0-repla
- 旧：。按共用约束 `.claude/agent-comm
  新：kmm 与 `54-layer0-replay`
- 旧：束 `.claude/agent-common.md`「门禁」一节从阶段归属
  新：layer0-replay` 并行起（各自后台、各
- 旧：t-common.md`「门禁」一节从阶段归属表取登记给你的阶段：55、57 号与 54 号并行起（
  新：yer0-replay` 并行起（各自后台、各自
- 旧：给你的阶段：55、57 号与 54 号并行起（各自后台、各自内
  新：er0-replay` 并行起（各自后台、各自内
- 旧：上限，照第 1b 步），59 号等 54 号跑完再起
  新：上限，照第 1b 步），checker-tier-crates-mutation-replay 等 `54-layer
- 旧：层 0 全量争 CPU），其余轻阶段一次一个；54、55、57、59
  新：层 0 全量争 CPU）；四道的命令都带输入给的
- 旧：），其余轻阶段一次一个；54、55、57、59 号命令带输入给的那个前缀：
  新：0 全量争 CPU）；四道的命令都带输入给的那个前缀
- 旧：gate.d/<文件>`，其余登记给你的轻阶段不带；54 号默认跑快档（`
  新：gate.d/<文件>`；`54-layer0-
- 旧：force` 删；每个记开始与结束时刻（`date
  新：force` 删；每个记耗时与退出码。单个阶段超过
- 旧：ce` 删；每个记开始与结束时刻（`date -u`）和退出码。单个阶段超过 B
  新：ce` 删；每个记耗时与退出码。单个阶段超过 B
- 旧：用的临时目录、编译产物（59 号的 `GATE_MU
  新：用的临时目录、编译产物（checker-tier-crates-mutation-replay 的 `GATE_MUT
- 旧：log`（前台跑也写），59 号那一份是变异分诊员要
  新：log`（前台跑也写），checker-tier-crates-mutation-replay 那一份是变异分诊员要的

### .claude/agents/experiment-designer.md

- 旧：实验页建起来之前，门禁 86 号会判这个实验号「有
  新：实验页建起来之前，门禁 doc-experiments 的 experimen

### .claude/agents/experiment-runner.md

- 旧：（书记员翻分项状态之后 33 号红）：给表名、源文件
  新：记员翻分项状态之后门禁 code-source-discipline 的 mutation-
- 旧：号红）：给表名、源文件与 33 号原样输出。这时不要跑前登
  新：格红）：给表名、源文件与那一格的原样输出。这时不要跑前登
- 旧：tsv` 末尾、归门禁 59 号跑。**另有四条只对
  新：tsv` 末尾、归门禁 checker-tier-crates-mutation-replay 跑。**另有四条只对入
- 旧：决的点名清单**：门禁 56 号的正则罩得到 `cr
  新：决的点名清单**：门禁 doc-process-records 的 crates-ad
- 旧：记里每一个会跑到的取值。写完跑 `bash .claude/scripts/naming-lint.sh`，新文件里的缩写与单字母名在这一步改完再往下走，不留给后面的门禁阶段。
  新：记里每一个会跑到的取值。
- 旧：名、三个数写「待提交时 59 号」。`research/
  新：名、三个数写「待提交时 checker-tier-crates-mutation-replay」。`research/
- 旧：源码改动编出来；生成产物与第 5 步跑 `replay.sh` 时都不设 `CARGO_
  新：源码改动编出来；生成产物时不设 `CARGO_T
- 旧：sh 会静默盖掉前一个），跑一次 `bash research/scripts/replay.sh E<号>` 确认逐字节一致。登记行改指第 4 步的
  新：sh 会静默盖掉前一个）。登记行改指第 4 步的
- 旧：字节一致也点一行；门禁 40 号按文件名查）；这一次
  新：字节一致也点一行；门禁 doc-experiments 的 results-c
- 旧：6. 跑阶段归属表登记给你的阶段（共用约束 `.c
  新：6. 门禁阶段交回前不跑，由提交前 `gate-triage` 跑的 `research/
- 旧：跑阶段归属表登记给你的阶段（共用约束 `.clau
  新：e-triage` 跑的 `research/scripts/gate-staged.sh`（它起的整轮 `gate.sh`）判（共用约束 `.clau
- 旧：n.md`「门禁」一节），读实验页或实验索引的那几道除外（27、34、40、69、75、84、85、86、88、99 号）：它们放到第 7 步写完实验页之后跑；这一次不写实验页的，在这一步最后跑，红了照写，产物照第 4
  新：n.md`「门禁」一节）。整轮门禁判红派回给你修时，产物照第
- 旧：页的，在这一步最后跑，红了照写，产物照第 4 步一个不
  新：禁」一节）。整轮门禁判红派回给你修时，产物照第 4 步一个不
- 旧：物照第 4 步一个不删（86 号出路里「把 rese
  新：物照第 4 步一个不删（doc-experiments 的 experimen
- 旧：报告交主 agent）。其中要编译或复跑整张表的，开跑前照共用约束看负载。15 号（编整个 research 工作区）与 87 号（复跑全部已入库实验）不归你，收尾由 `gate-triage` 统一跑；你只跑第 5 步自己那一个实验的 `replay.sh`。
  新：报告交主 agent）。
- 旧：riments.md`。写完（连同第 7b 步）跑第 6 步留下的那几道，各贴原样末行与退出码。
  新：riments.md`。
- 旧：照做）不查依据段（门禁 75 号对清单里的决策整段跳
  新：照做）不查依据段（门禁 doc-experiments 的 decision-
- 旧：experiments-history.md` 的条目，对这两
  新：riments/<号>-<简称>.md`，结果整行抄自产
- 旧：istory.md` 的条目，对这两份跑一次 `bash .clau
  新：跑命令、口径、它答不了的；索引行进 `.claude/kb
- 旧：条目，对这两份跑一次 `bash .claude/sing
  新：它答不了的；索引行进 `.claude/kb/e
- 旧：ash .claude/singlefs-ai-sop/s
  新：行进 `.claude/kb/experiments.
- 旧：laude/singlefs-ai-sop/scripts/do
  新：.claude/kb/experiments.md
- 旧：glefs-ai-sop/scripts/doc-li
  新：laude/kb/experiments.md`。
- 旧：op/scripts/doc-lint.sh .`，贴末行；红在这一次写
  新：periments.md`。
- 旧：c-lint.sh .`，贴末行；红在这一次写的句子上的改到绿，红在别处的照写不修。
  新：eriments.md`。
- 旧：一格耗时、下一格预计多久，时刻写 JST），主 agent 读文
  新：一格耗时、下一格预计多久），主 agent 读文
- 旧：的点名（第 4c 步）、`replay.sh` 结果、实验页路径；登记修订了什
  新：的点名（第 4c 步）、实验页路径；登记修订了什
- 旧：论，要走三方）；没跑门禁全量；没提交。
  新：论，要走三方）；没跑门禁阶段；没提交。

### .claude/agents/gate-triage.md

- 旧：步直接跑，外面不包一层（87 号经 `researc
  新：步直接跑，外面不包一层（checker-tier-research-build-and-replay 的 experimen
- 旧：ay.sh` 逐条套了，15、74 号在阶段里面经包装）。
  新：ay.sh` 逐条套了，同一道的 research-un
- 旧：een`，下一次 54、55、57、59、74、87
  新：er0-replay`、checker-tier-qemu-device-streams、checker-tie
- 旧：`，下一次 54、55、57、59、74、87 号才
  新：ice-streams、checker-tier-lkmm、checker-tie
- 旧：一次 54、55、57、59、74、87 号才复用得
  新：r-tier-lkmm、checker-tier-crates-mutation-replay、harness-mod
- 旧：54、55、57、59、74、87 号才复用得上，判据在
  新：tion-replay、harness-model-differential-and-scenarios 与 experimen
- 旧：参数（前缀照带）。54、55、57、59 这几道重阶
  新：er0-replay`、checker-tier-qemu-device-streams、checker-tie
- 旧：前缀照带）。54、55、57、59 这几道重阶段在
  新：ice-streams、checker-tier-lkmm、checker-tie
- 旧：带）。54、55、57、59 这几道重阶段在 `ga
  新：r-tier-lkmm、checker-tier-crates-mutation-replay 这几道重阶段在 `ga
- 旧：uard.sh` 拒）；登记给你的其余阶段单跑时不带前缀，87 号
  新：uard.sh` 拒）；单跑某一道或某一格（`-
- 旧：；登记给你的其余阶段单跑时不带前缀，87 号单跑照带
  新：rd.sh` 拒）；单跑某一道或某一格（`--check <格名>`）也带同一个前缀，不带的 `
- 旧：其余阶段单跑时不带前缀，87 号单跑照带。超过 Ba
  新：名>`）也带同一个前缀，不带的 `.claude/ho
- 旧：段单跑时不带前缀，87 号单跑照带。超过 Bash
  新：也带同一个前缀，不带的 `.claude/hooks/heavy-test-guard.sh` 拒（提交时才跑的检查一类）。超过 B
- 旧：时不带前缀，87 号单跑照带。超过 Bash 单次上
  新：.sh` 拒（提交时才跑的检查一类）。超过 Bash 单次上
- 旧：内存包装退 25x 的（15、74 号的「内存包装 res
  新：内存包装退 25x 的（research-unit-tests 格与 harness-

### .claude/agents/implementation-writer.md

- 旧：档测试文件第一行声明模块、耗时用例由 `research/scripts/harness-test-timing.py` 按耗时表标，不手标）。
  新：档测试文件第一行声明模块）。
- 旧：es/` 下的源码之后 33 号红）：给 33 号原
  新：/` 下的源码之后门禁 code-source-discipline 的 mutation-
- 旧：码之后 33 号红）：给 33 号原样输出与被改写的源文件
  新：tables 格红）：给那一格的原样输出与被改写的源文件
- 旧：的原文改到源码今天的写法，改完跑 33 号。
  新：的原文改到源码今天的写法。
- 旧：build` 不经它。登记给你的 74 号在阶段里面经包装，照跑，外面不再包一层。
  新：build` 不经它。
- 旧：tsv` 在的话（门禁 59 号），每条再加一行变异，原
  新：tsv` 在的话（门禁 checker-tier-crates-mutation-replay），每条再加一行变异，原
- 旧：，整表复跑归最后的门禁 59 号；报告写明哪几行证过、哪
  新：，整表复跑归最后的门禁 checker-tier-crates-mutation-replay；报告写明哪几行证过、哪
- 旧：明哪几行证过、哪几行留给 59 号。checker 档不跑
  新：明哪几行证过、哪几行留给它。checker 档不跑
- 旧：、报告写明留给提交时的 59 号。checker 档由提
  新：、报告写明留给提交时的 checker-tier-crates-mutation-replay。checker 档由提
- 旧：all-targets`、阶段归属表登记给你的门禁阶段（共用约束「门禁」一节，74 号在内），各贴末尾原样输出。其余
  新：all-targets`，各贴末尾原样输出。门禁
- 旧：内），各贴末尾原样输出。其余门禁阶段、全量 `car
  新：s`，各贴末尾原样输出。门禁阶段、全量 `car
- 旧：变；层 0 快档是重型（整轮门禁的 54 号），集成
  新：变；层 0 快档是重型（门禁 `54-layer

### .claude/agents/investigator.md

- 旧：行、种子、跑出现象的命令与时刻（写清时区）。
  新：行、种子、跑出现象的命令。
- 旧：0 的测试二进制、54、55、57、59、87 号、
  新：er0-replay`、checker-tier-qemu-device-streams、checker-tie
- 旧：测试二进制、54、55、57、59、87 号、全量
  新：ice-streams、checker-tier-lkmm、checker-tie
- 旧：进制、54、55、57、59、87 号、全量 car
  新：r-tier-lkmm、checker-tier-crates-mutation-replay、checker-tie
- 旧：54、55、57、59、87 号、全量 cargo
  新：tion-replay、checker-tier-research-build-and-replay 的 experimen

### .claude/agents/kb-scribe.md

- 旧：更史、分项状态、欠账表）并跑 kb 门禁阶段。只在主 agent 点
  新：更史、分项状态、欠账表）。只在主 agent 点
- 旧：）」那一段由你在写之前照当月文件现取，其余逐字照给）。
  新：）」那一段由你在写之前照 `decisions-history.md` 对应决策节里那个日期块现取，其余逐字照给）。
- 旧：**状态：未定。**」（20 号），翻成未定的判两句
  新：**状态：未定。**」（门禁 doc-decisio
- 旧：号），翻成未定的判两句（31 号两把尺都要，缺一句就
  新：格），翻成未定的判两句（门禁 doc-decisio
- 旧：数字或一到十的汉字数字，20、75 号都认）；`.clau
  新：数字或一到十的汉字数字，doc-decisions 的 kb-shape
- 旧：状态列由第 3 步的 `21-decision-it
  新：状态列由第 3 步的 `doc-decisions.s
- 旧：`21-decision-items-sync.sh --
  新：doc-decisions.sh --write
- 旧：cision-items-sync.sh --write`
  新：oc-decisions.sh --write`
- 旧：，一条分项一行）：门禁 75 号那条双向检查两侧要同
  新：，一条分项一行）：门禁 doc-experiments 的 decision-
- 旧：开工时先记下规格点名的文件、当月变更史文件、`.claude/
  新：开工时先记下规格点名的文件、`.claude/
- 旧：. 决策变更史：原文写进当月的 `.claude/kb/decisions-history/<年-月>.md`，标题下两行快查照规格；新条目放在哪、同一天多条怎么编「（其N）」，照当月文件与 `.claude/kb
  新：. 决策变更史：原文写进 `.claude/kb
- 旧：istory.md` 的文件头，不另定。然后先跑不带 `--write` 的
  新：ry.md` 对应决策的 `## D<n>（简称
- 旧：不另定。然后先跑不带 `--write` 的 `bash .c
  新：.md` 对应决策的 `## D<n>（简称）` 节里，按 `.cla
- 旧：aude/gate.d/49-history-bri
  新：aude/gate.d/doc-decisions.s
- 旧：e.d/49-history-brief.sh`：它列的「快查·
  新：/doc-decisions.sh --write`
- 旧：ry-brief.sh`：它列的「快查·… 还没写」里
  新：.sh --write` 刷新那一节顶上的「**现状**：」行（
- 旧：ief.sh`：它列的「快查·… 还没写」里有不是这一轮的条目，
  新：e` 刷新那一节顶上的「**现状**：」行（这一步只改那一行，
- 旧：列的「快查·… 还没写」里有不是这一轮的条目，就停在 `
  新：顶上的「**现状**：」行（这一步只改那一行，不碰历
- 旧：… 还没写」里有不是这一轮的条目，就停在 `--wr
  新：**现状**：」行（这一步只改那一行，不碰历史条目；派发提示写「并行写
- 旧：」里有不是这一轮的条目，就停在 `--write` 之
  新：发提示写「并行写回」的，这个 `--write` 由
- 旧：在 `--write` 之前交回，不让 `--write` 往那些条目里写「（待补）」；都是这一轮的才跑 `--write`。派发提示写「并行写回」（主 agent 同时派了几个书记员）的，21、49 号的 `--write` 都不跑，由主 agent 在全部
  新：个 `--write` 由主 agent 在全部
- 旧：nt 在全部交回后跑一次；变更史条目的「（其N）」照派发提示给的写。跑
  新：nt 在全部交回后跑一次）。跑前跑后各 `git
- 旧：；变更史条目的「（其N）」照派发提示给的写。跑前跑后各 `git
  新：t 在全部交回后跑一次）。跑前跑后各 `git
- 旧：` 一次，只核新增的行：每条决策的卡片只留最近 3 次，被挤掉的旧行是按设计；新增的行里有这一轮之外的
  新：` 一次，只核新增的行：新增的行里有这一轮之外的
- 旧：/**/*.rs` 的，加跑 33 号，并把改到的实验源码逐个列给
  新：/**/*.rs` 的，把改到的实验源码逐个列给
- 旧：aude/gate.d/21-decision-it
  新：aude/gate.d/doc-decisions.s
- 旧：/21-decision-items-sync.sh --
  新：doc-decisions.sh --write
- 旧：cision-items-sync.sh --write`
  新：oc-decisions.sh --write`
- 旧：写、一起回读，不分两次。写完跑 75 号：它报「不对称」时看点名的那一对在不在这一份规格里——在，就是规格少给了一边，停下报告并写明缺哪一行；不在，按「不是这一轮的不修」照写。**判一个实验撑不撑一条
  新：写、一起回读，不分两次。**判一个实验撑不撑一条
- 旧：每次写入之后，写入后钩子 `
  新：c-decisions 的 kb-shape 格），翻成未定的判两句（门禁 doc-decisions 的 blocking-verdict 格两把尺都要，缺一句就红）：改不改新池新建文件的字节、动不动格式，决策文件标题行的状态改成什么（已定 / 半定（N 项未定）/ 待定（N 项未定），N 写阿拉伯数字或一到十的汉
- 旧：每次写入之后，写入后钩子 `.cla
  新：定（N 项未定），N 写阿拉伯数字或一到十的汉字数字，doc-decisio
- 旧：每次写入之后，写入后钩子 `.claude/ho
  新：数字或一到十的汉字数字，doc-decisions 的 kb-shape
- 旧：钩子 `.claude/hooks/kb-scribe-fo
  新：认）；`.claude/kb/decisions
- 旧：e/hooks/kb-scribe-followup
  新：ude/kb/decisions.md` 状态列
- 旧：ooks/kb-scribe-followup.sh`
  新：de/kb/decisions.md` 状态列由
- 旧：kb-scribe-followup.sh` 按 `.cla
  新：e/kb/decisions.md` 状态列由第 3
- 旧：be-followup.sh` 按 `.claude
  新：b/decisions.md` 状态列由第 3 步的
- 旧：up.sh` 按 `.claude/hooks/kb-scribe-followups.tsv` 跑几道阶段、把红的 ✗ 与 → 交回给你：红的是这个流程下一步会消掉的（21 号在 `21-decision-it
  新：由第 3 步的 `doc-decisions.s
- 旧：`21-decision-items-sync.sh --
  新：doc-decisions.sh --write
- 旧：cision-items-sync.sh --write`
  新：oc-decisions.sh --write`
- 旧：sh --write` 之前：翻状态的规格在第 3 步跑它，新
  新：sh --write` 重生成，规格只给预期的分项计数供
- 旧：e` 之前：翻状态的规格在第 3 步跑它，新立分项、改索引行这类不翻状
  新：rite` 重生成，规格只给预期的分项计数供回读核对，不手
- 旧：第 3 步跑它，新立分项、改索引行这类不翻状态的规格不走第 3
  新：生成，规格只给预期的分项计数供回读核对，不手写。变更史标题与快查
- 旧：立分项、改索引行这类不翻状态的规格不走第 3 步，写
  新：题与快查里的分项标签写翻完之后的：变更史用今天的编号（
- 旧：、改索引行这类不翻状态的规格不走第 3 步，写完正文自己跑一次 `bash .claud
  新：里的分项标签写翻完之后的：变更史用今天的编号（`.claude/kb/
- 旧：，写完正文自己跑一次 `bash .claude/gate
  新：：变更史用今天的编号（`.claude/kb/d
- 旧：ash .claude/gate.d/21-decision
  新：编号（`.claude/kb/decisions-h
- 旧：aude/gate.d/21-decision-ite
  新：`.claude/kb/decisions-hi
- 旧：ecision-items-sync.sh --wri
  新：relabel-item.py` 也会把它改掉。
- 旧：ion-items-sync.sh --write` 再往下；30 号在变
  新：abel-item.py` 也会把它改掉。
- 旧：sh --write` 再往下；30 号在变更史条目写之前、49 号在 `--write` 之前、75 号在实验页那几行写之前），照流程往下走；其余的留到第 4 步一起判归属。
  新：el-item.py` 也会把它改掉。
- 旧：4. 跑阶段归属表登记给你的阶段各一次并贴结果（共
  新：4. 门禁阶段交回前不跑，由提交前 `gate-triage` 跑的 `research/
- 旧：跑阶段归属表登记给你的阶段各一次并贴结果（共用约束 `.clau
  新：e-triage` 跑的 `research/scripts/gate-staged.sh`（它起的整轮 `gate.sh`）判（共用约束 `.clau
- 旧：.md`「门禁」一节）；阶段数用那条 `awk` 接 `|
  新：.md`「门禁」一节）；第 2、3 步里的 `--write` 是
- 旧：段数用那条 `awk` 接 `| wc -l` 数出来贴原样，不手数。红的判它是不是这一轮写出来
  新：的 `--write` 是写回流程的生成一步，照跑。
- 旧：出来贴原样，不手数。红的判它是不是这一轮写出来的（看点名的文
  新：rite` 是写回流程的生成一步，照跑。
- 旧：手数。红的判它是不是这一轮写出来的（看点名的文件与行在不在这一轮改动里）；不是这一轮的不修，照写。30 号报的改动
  新：e` 是写回流程的生成一步，照跑。
- 旧：不是这一轮的不修，照写。30 号报的改动行数含别的会话没提交的改动，照抄，不据此判归属。另跑一次 `bash .claude/singlefs-ai-sop/scripts/doc-lint.sh .` 贴末行；红在这一轮写的句子上，停下交主 agent 改规格，不自己改句子。
  新：回流程的生成一步，照跑。
- 旧：el-item.py` 与新建当月变更史文件走 Bash，闸管不到，
  新：el-item.py` 走 Bash，闸管不到，
- 旧：些文件与「要人看」清单、各门禁阶段结果与归属、改过的全部文件与
  新：些文件与「要人看」清单、第 2、3 步 `--write` 的原样末行与退出码、改过的全部文件
- 旧：」清单、各门禁阶段结果与归属、改过的全部文件与前后
  新：rite` 的原样末行与退出码、改过的全部文件与前后

### .claude/agents/kb-spec-drafter.md

- 旧：- 要动的 kb 文件；当月变更史文件（`.claude/kb/
  新：- 要动的 kb 文件；`.claude/kb/
- 旧：ons-history/<年-月>.md`）。
  新：态见 `.claude/rules/changelog-format.md`）
- 旧：-history/<年-月>.md`）。
  新：s/changelog-format.md`）。
- 旧：翻分项状态的，给门禁 20、31 号要的句子（「*
  新：翻分项状态的，给门禁 doc-decisions 的 kb-shape、blocking-ve
- 旧：项状态的，给门禁 20、31 号要的句子（「**状态
  新：的 kb-shape、blocking-verdict 两格要的句子（「**状
- 旧：b；没跑 kb 门禁阶段（书记员写完之后跑）；规格里的判断以判决为准
  新：b；没跑 kb 门禁阶段；规格里的判断以判决为准

### .claude/agents/mutation-triage.md

- 旧：溃验证员跑的那一次门禁 59 号的输出路径（第 3
  新：溃验证员跑的那一次门禁 checker-tier-crates-mutation-replay 的输出路径（第 3 步
- 旧：目时：提交时那一次门禁 59 号的输出路径，即崩溃验
  新：目时：提交时那一次门禁 checker-tier-crates-mutation-replay 的输出路径，即崩溃验证
- 旧：路径，即崩溃验证员报告里 59 号那一行的日志路径（缺它不
  新：路径，即崩溃验证员报告里这一道那一行的日志路径（缺它不
- 旧：交时由崩溃验证员跑门禁 59 号（`.claude/ru
  新：交时由崩溃验证员跑门禁 checker-tier-crates-mutation-replay（`.claude/ru
- 旧：agent 给你那一次 59 号的输出路径，你从里面
  新：agent 给你那一次 checker-tier-crates-mutation-replay 的输出路径，你从里面取
- 旧：「已还原，基线仍全绿」；59 号没有这句，看它收尾有
  新：「已还原，基线仍全绿」；checker-tier-crates-mutation-replay 没有这句，看它收尾有没
- 旧：只报内存撞顶与超时两栏；59 号照它自己收尾的「计数
  新：只报内存撞顶与超时两栏；checker-tier-crates-mutation-replay 照它自己收尾的「计数：
- 旧：里带 `$` 这类字符，59 号按字面去找）；都没有记「
  新：里带 `$` 这类字符，它按字面去找）；都没有记「
- 旧：」与数标记的命令和输出，59 号贴「计数：」行原样）
  新：」与数标记的命令和输出，checker-tier-crates-mutation-replay 贴「计数：」行原样）；

### .claude/agents/prior-art.md

- 旧：0. 要查本机源码树的，先跑阶段归属表登记给你的阶段（共用约束 `.claude/agent-common.md`「门禁」一节）：70 号会报哪些承重引用指向的源码树或文献不在本机了，那几棵树上的事实写「本机核不动」。70 号只核 kb 里登记过的那批断言，不替你回答这一次的问题。
  新：

### .claude/agents/sweep.md

- 旧：aude/gate.d/27-format-cons
  新：aude/gate.d/code-source-disc
- 旧：at-constants.sh` 实际扫描的文件集合（读
  新：at-constants 格实际扫描的文件集合（

### .claude/agents/three-way-materials.md

- 旧：0. 开工先跑阶段归属表登记给你的阶段（共用约束 `.claude/ag
  新：5b. 写开工快照：清单里每个 kb 文件一行进 `research/p
- 旧：的阶段（共用约束 `.claude/agent-commo
  新：一行进 `research/prompts/<轮>
- 旧：约束 `.claude/agent-common.md`「门禁
  新：进 `research/prompts/<轮>-sn
- 旧：e/agent-common.md`「门禁」一节）：58
  新：`（`sha256sum` 原样输出，路径从仓根
- 旧：t-common.md`「门禁」一节）：58 号红、而且点名
  新：（`sha256sum` 原样输出，路径从仓根起），排他新建；这个目录已
- 旧：n.md`「门禁」一节）：58 号红、而且点名的就是这一轮的正文，就不
  新：原样输出，路径从仓根起），排他新建；这个目录已经在的不动，回复里写明。门禁
- 旧：：58 号红、而且点名的就是这一轮的正文，就不开工，回复里原样抄它的
  新：他新建；这个目录已经在的不动，回复里写明。门禁
- 旧：的就是这一轮的正文，就不开工，回复里原样抄它的 ✗
  新：新建；这个目录已经在的不动，回复里写明。门禁 do
- 旧：的正文，就不开工，回复里原样抄它的 ✗ 与 →；点名的是
  新：目录已经在的不动，回复里写明。门禁 doc-process-records 的 implementa
- 旧：开工，回复里原样抄它的 ✗ 与 →；点名的是别的轮次的正文，照写、继续。47 号红时看点名的是哪份脚本：是你要用的 `quote-kb.py`、`k
  新：s-records 的 implementation-premi
- 旧：：是你要用的 `quote-kb.py`、`kb-sections.py`、`c
  新：的 implementation-premise
- 旧：、`kb-sections.py`、`checkli
  新：plementation-premise 格查新轮
- 旧：b-sections.py`、`checklist-spec
  新：ementation-premise 格查新轮次有
- 旧：ons.py`、`checklist-specs.py
  新：entation-premise 格查新轮次有没有
- 旧：y`、`checklist-specs.py`（代码轮还
  新：ation-premise 格查新轮次有没有这个
- 旧：hecklist-specs.py`（代码轮还有 `quote-r
  新：tion-premise 格查新轮次有没有这个目录、清单
- 旧：s.py`（代码轮还有 `quote-rust-items.py`），停下报告；是别的脚本，照写、继续。
  新：没有这个目录、清单里的 kb 文件在不在里面。
- 旧：取」。标题里带冒号的小节（例如带时刻的标题），`@标题` 取法会在冒
  新：取」。标题里带冒号的小节，`@标题` 取法会在冒
- 旧：二）：文件头写基准与生成时刻，输入给的 diff 原
  新：二）：文件头写基准与生成日期，输入给的 diff 原
- 旧：不动，回复里写明。门禁 58 号查新轮次有没有这个目
  新：不动，回复里写明。门禁 doc-process-records 的 implement

### .claude/agents/three-way-verifier.md

- 旧：---
  新：
- 旧：name: three-way-verifier
  新：
- 旧：description: 三方论证的核查员：逐条核腿报告里的原文引用、产物行与复跑命令，交核对表。只在主 agent 点名派发、且这一轮全部腿都已交齐时用；不要自动派发。
  新：
- 旧：tools: Read, Bash
  新：
- 旧：model: sonnet
  新：
- 旧：effort: high
  新：
- 旧：omitClaudeMd: true
  新：
- 旧：required-inputs: 快照, 草稿目录, 报告路径|-verifier-output.md
  新：
- 旧：---
  新：
- 旧：# 核查员（three-way-verifier）
  新：
- 旧：开工先读 `.claude/agent-common.md`；这份定义开了 `omitClaudeMd`，不继承项目 CLAUDE.md 与它 `@` 的规则，要用的规则照共用约束「规则怎么读」一节读。
  新：
- 旧：你交的是观测，不是判决：核对表里的 ✗ 不免除主 agent 对推论的逐条现查，这一句照抄进报告开头。
  新：
- 旧：开工先读：`.claude/rules/three-way-inference.md`「判决由主 agent 做，不由投票做」；`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「校验路径本身也要证明它会红」「引产物就整行抄」。
  新：
- 旧：## 输入（主 agent 必须给）
  新：
- 旧：- 轮名、这一轮全部腿报告的路径（主 agent 确认都已交齐、腿不再写）。
  新：
- 旧：- 背景材料路径（用来识别误写成背景材料行号的引用）。
  新：
- 旧：- 云端腿交回里给的报告 `sha256sum`（有就给）。
  新：
- 旧：- 腿开工那一刻的快照路径：`sha256sum` 清单，罩这一轮被判的文件与材料点名的 kb 文件（`.claude/rules/implementation-workflow.md`「代码轮派腿之前记一份开工快照」；代码轮必给），腿跑着的时候被别的会话改过的，主 agent 另给倒推出的原样副本（`*.at-snapshot`）。没给快照的代码轮，停下要，不对主树核。没给快照的轮（设计轮）对主
  新：
- 旧：- 腿开工时刻（UTC）：第 2 步现查「快照清单里没有的文件」在腿开工之后有没有被改过要用；没给时，快照清单外的文件内容对不上记「分不清：文件可能在腿开工之后被改过」，不记 ✗。
  新：
- 旧：- 报告路径（形态 `research/prompts/<轮>-verifier-output.md`）、草稿目录。
  新：
- 旧：## 做什么
  新：
- 旧：1. 判别力自证，先做：从待核的引用里挑一条，在草稿目录的副本里把它的行号加 1，按第 2 步核它，必须判 ✗；判不出就停下报告「核查方法不分辨」。
  新：
- 旧：2. 先跑 `python3 research/scripts/cite-check.py <全部腿报告> --root <快照根> --background <背景材料>`：它判了的（对不上、指到标题行、写的是背景材料的行号）照抄它的判定，贴末行；它列成「没判」的与它认不出写法的引文，再逐处人工核。每处「文件:行号 + 抄的原文」：到快照里取那一行（区间就取区间）比内容（输入给了快照就一律对快照
  新：
- 旧：3. 每行引的产物：在产物文件里逐字找。
  新：
- 旧：4. 每条复跑命令：把腿的模型目录拷到草稿目录，在副本里跑（加 `nice -n 19`），比输出与报告里抄的、比 sha256；不在腿的原目录里跑。复跑命令带着指向路径的环境变量的，路径一并换到你的草稿目录；输出里嵌着临时路径的，按字段比，不按整份哈希判 ✗。
  新：
- 旧：5. 本地腿的转述核对表里每一处「原文文件:行」也逐条核：行号在不在、抄的是不是原文、英文有没有丢限定词，也有没有多加原文没有的限定词或括注。
  新：
- 旧：6. 要编译的在草稿目录的副本里经 `bash research/scripts/run-with-memory-cap.sh 4G …` 跑（共用约束「不做」一节）；包装跑不起来（退 251 这类）、要虚机、要网络的写「核不动」与原因；「核不动」与「分不清」（第 2 步与输入一节那几种，连同原因）各单列一栏，不算进 ✓ 也不算进 ✗；报告文件现在的 sha256 与交回里给的对不上，整份记「分不
  新：
- 旧：## 写范围
  新：
- 旧：- 报告文件、草稿目录。除此之外不写。
  新：
- 旧：- 报告用 Bash 写：`set -o noclobber` 后 `cat > 报告路径 <<'EOF'` 新建，之后 `>>` 分段追加（共用约束「写」一节）。定义里没有 Write 工具不等于写不了文件；报告只交在回复里不算交。
  新：
- 旧：## 产出
  新：
- 旧：- 开头：判别力自证那一条的原样结果。
  新：
- 旧：- 每份腿报告一张表：引用 / 核的结果（✓、✗ 加实际位置、核不动）/ 命令；末尾计数（核了几处、✓ 几处、✗ 几处、核不动几处、分不清几处），再加「没做什么」。
  新：
- 旧：## 没做什么（固定会有的）
  新：
- 旧：- 不判一条打中成不成立、该不该采纳；不核推理本身，只核引用、产物与复跑。
  新：

### .claude/agents/tooling-writer.md

- 旧：条改法先造一个该红的输入证红。只在主 agent 点
  新：条改法先造一个该红的输入。只在主 agent 点
- 旧：脚本加弄坏开关或样本目录，跑它的 `--selftest` 或门禁阶段的样本，贴「弄坏开关 → 判红」的原样行；改完再跑一次贴转绿的原样行。说不出该红的输入，这一
  新：脚本加弄坏开关或样本目录。说不出该红的输入，这一
- 旧：aude/gate.d/63-agent-write
  新：aude/gate.d/code-tooling.sh`
- 旧：-write-scope.sh`；新研究脚本写 `adm
  新：-write-scope 格；新研究脚本写 `adm
- 旧：aude/gate.d/47-research-sc
  新：aude/gate.d/code-tooling.sh`
- 旧：pt-selftests.sh` 的 runner 表。
  新：pt-selftests 格的 runner 表
- 旧：nt 开定义三方（门禁 72 号）。
  新：nt 开定义三方（门禁 doc-process-records 的 agent-def
- 旧：7. 收尾跑并贴末行与退出码：`bash .
  新：4. 新门禁阶段与新钩子写 `# gat
- 旧：7. 收尾跑并贴末行与退出码：`bash .claud
  新：4. 新门禁阶段与新钩子写 `# gate-simi
- 旧：aude/gate.d/47-research-sc
  新：aude/gate.d/code-tooling.sh`
- 旧：pt-selftests.sh`、62、63、73 号，`bash .cl
  新：pt-selftests 格的 runner 表
- 旧：h`、62、63、73 号，`bash .claude/sin
  新：t-selftests 格的 runner 表（`S
- 旧：、73 号，`bash .claude/singlefs
  新：elftests 格的 runner 表（`SEL
- 旧：bash .claude/singlefs-ai-sop/scripts/doc-li
  新：sts 格的 runner 表（`SELFTES
- 旧：s-ai-sop/scripts/doc-lint.sh .`，规则纪律项目本地
  新：ts 格的 runner 表（`SELFTEST
- 旧：doc-lint.sh .`，规则纪律项目本地那一道（`RULES_LINT
  新：s 格的 runner 表（`SELFTEST_R
- 旧：本地那一道（`RULES_LINT_DIR=.claude/rules RULES_LINT_FILES="CLAUDE.md .claude/agents/*.md .claude/agent-common.md .claude/main-agent.md" bash .claude/singlefs-ai-sop/scripts/rules-lint.sh .`），`GATE_LIN
  新：TEST_RUNNERS`）。
- 旧：-lint.sh .`），`GATE_LINT_DIR=.claude/gate.d` 的 gate-lint 与 `SHELL_LINT_DIR=.claude/gate.d` 的 shell-lint，`python3 .claude/singlefs-ai-sop/scripts/preflight-lint.py`，`python3 .claude/singlefs-ai-sop/scri
  新：ST_RUNNERS`）。
- 旧：段归属表登记给你的阶段。54、55、57、59、87 号不真跑，只跑它们的 `--selftest` 或静态分支。红了先看点名的文件在不在这一轮的改动里；不在的不修，照写。
  新：T_RUNNERS`）。
- 旧：过 600k：停在最近一个自证全绿的点，报告写做完的条、做
  新：过 600k：停在最近一条改法写完的点，报告写做完的条、做
- 旧：` 原样；每条改法的判红原样行与转绿原样行；第 7 步
  新：原样；每条改法的样本（红、绿各一份以上）、自证格与弄坏开关；新写的 `#
- 旧：；每条改法的判红原样行与转绿原样行；第 7 步各项的末行与退出码；新写的 `# gate
  新：绿各一份以上）、自证格与弄坏开关；新写的 `# gate
- 旧：- 没跑重型测试（54、55、57、59、87 号本
  新：er0-replay`、checker-tier-qemu-device-streams、checker-tie
- 旧：跑重型测试（54、55、57、59、87 号本身、`
  新：ice-streams、checker-tier-lkmm、checker-tie
- 旧：测试（54、55、57、59、87 号本身、`gat
  新：r-tier-lkmm、checker-tier-crates-mutation-replay、checker-tie
- 旧：54、55、57、59、87 号本身、`gate.s
  新：tion-replay、checker-tier-research-build-and-replay 本身、`gate.sh

### .claude/kb/decisions.md

- 旧：与实验双向登记」，门禁 75 号判。
  新：与实验双向登记」，门禁 doc-experiments 的 decision-
- 旧：文；改前、改后与依据写进当月的 `decisions-history/<年-月>.md`，写法见 [decisions-
  新：文；改前、改后与依据写进 [decisions-
- 旧：-history.md)「怎么加一条」。decisions.m
  新：-history.md) 对应决策的 `## D<n>（简称）` 节里，组织形态见 `.claude/rules/changelog-format.md`。decisions.m
- 旧：aude/gate.d/37-decision-su
  新：aude/gate.d/doc-decisions.s
- 旧：ummary-width.sh`）判红；那个数的权威记录
  新：ummary-width 格）判红；那个数的权威记录
- 旧：aude/gate.d/21-decision-it
  新：aude/gate.d/doc-decisions.s
- 旧：/21-decision-items-sync.sh --
  新：doc-decisions.sh --write
- 旧：cision-items-sync.sh --write
  新：oc-decisions.sh --write
- 旧：aude/gate.d/20-kb-shape.sh
  新：aude/gate.d/doc-decisions.s
- 旧：/20-kb-shape.sh`）第 7 段**自己去数
  新：` 的 kb-shape 格）第 7 段**自己去数
- 旧：aude/gate.d/21-decision-it
  新：aude/gate.d/doc-decisions.s
- 旧：n-items-sync.sh`）逐字比对，
  新：n-items-sync 格）逐字比对，
- 旧：aude/gate.d/21-decision-it
  新：aude/gate.d/doc-decisions.s
- 旧：/21-decision-items-sync.sh --
  新：doc-decisions.sh --write
- 旧：cision-items-sync.sh --write
  新：oc-decisions.sh --write
- 旧：aude/gate.d/21-decision-it
  新：aude/gate.d/doc-decisions.s
- 旧：n-items-sync.sh           #
  新：n-items-sync           #
- 旧：aude/gate.d/22-item-ref-st
  新：aude/gate.d/doc-decisions.s
- 旧：m-ref-status.sh`）判红。
  新：m-ref-status 格）判红。
- 旧：- 10. 层 1 抽样乘的 N 是什么 —— 已
  新：- 10. 真正必须精确覆盖的 N 是什么 —— 已
- 旧：- 11. 层 1 抽样怎么调度 —— 已定
  新：真正必须精确覆盖的 N 是什么 —— 已定
- 旧：11. 层 1 抽样怎么调度 —— 已定
  新：须精确覆盖的 N 是什么 —— 已定

### .claude/kb/experiments.md

- 旧：与实验双向登记」，门禁 75 号判。
  新：与实验双向登记」，门禁 doc-experiments 的 decision-

### .claude/kb/feature-bits.md

- 旧：d`，不改那张表。门禁 93 号（`.claude/
  新：d`，不改那张表。门禁 code-source-discipline 的 feature-b
- 旧：aude/gate.d/93-feature-bit
  新：aude/gate.d/code-source-disc
- 旧：e/gate.d/93-feature-bits.sh`
  新：gate.d/code-source-discipl
- 旧：同一位的「含义」，门禁 93 号按这一条比对。「名称
  新：同一位的「含义」，门禁 code-source-discipline 的 feature-b
- 旧：不是位掩码字面量，门禁 93 号解不出位号，把它列进
  新：不是位掩码字面量，门禁 code-source-discipline 的 feature-b

### .claude/kb/freeze-layer-membership.md

- 旧：aude/gate.d/16-freeze-laye
  新：aude/gate.d/doc-decisions.s
- 旧：r-membership.sh` 判 C45（四层图里每
  新：r-membership 格判 C45（四层图里

### .claude/kb/layout/01-first-txn.md

- 旧：ence` 钉住，门禁 52 号按它比。其余各行的段
  新：ence` 钉住，门禁 doc-registries 的 segment-r
- 旧：物之间的逐字比对由门禁 52 号做（C316（提交步
  新：物之间的逐字比对由门禁 doc-registries 的 segment-r
- 旧：件的干跑） 产物的各行，今天对的是第十六次跑第一段的
  新：件的干跑） 产物的各行，对的是第十九次跑的主产物
- 旧：行，今天对的是第十六次跑第一段的产物** `e142-
  新：的各行，对的是第十九次跑的主产物** `e142
- 旧：6-09-25-r16-combined.out`（`resea
  新：9-main-2026-09-28.out`（`resea
- 旧：那份）：普通发布与整条流两行是分配记录树、exten
  新：字相同；普通发布与整条流里的 12 个单元是分配记录树、exten
- 旧：项 14）之后的 12 个单元，取号与暖机两行与产物逐字相同；`name=width
  新：道（D13（验证路线） 已定项 4）；`name=width
- 旧：ed_form=67108885 skipped=tru
  新：orm=16777240 skipped=tru
- 旧：`\|` 隔开，门禁 52 号逐字比对） | 出处
  新：`\|` 隔开，门禁 doc-registries 的 segment-r
- 旧：e` 钉住录制流，门禁 52 号按它比）。⚠️ **
  新：e` 钉住录制流，门禁 doc-registries 的 segment-r
- 旧：ot×2,barrier]\|[journal_record×2,barrier]\|[root_record_fua]\|[system_configuration_slot×2]` | 零；D16
  新：ot×2,barrier×2]` | 零；D16
- 旧：机，前后都不另接空发布。每段写几个没有用例钉：第二条流里的回退已照这一形改了脚本，那条流钉的段序列、
  新：机，前后都不另接空发布。脚本钉的段序列、写数与闭
- 旧：的回退已照这一形改了脚本，那条流钉的段序列、写数与闭式还
  新：前后都不另接空发布。脚本钉的段序列、写数与闭式已
- 旧：的段序列、写数与闭式还没按它重核（步 6） | D23（
  新：与 C577（系统配置没见证到的最新根，乙罩不到）之前；带 layer0 名字的目标仍没跑过（`research/prompts/m2-impl-merge-verify-1-implementer-report.md` 第 73、222 行），层 0 仍没跑（步 6） | D23（
- 旧：+2+1+2`；整条流里取号的 2 个写与上一次发布的 2 个系统
  新：+2+1+2`；整条流里上一次发布的 2 个系统
- 旧：2 个系统配置槽写合成 4 写一段、末尾的 2 个系统
  新：之间隔着一道屏障、各自成一段，末尾的 2 个系统
- 旧：统配置槽写与暖机第一次的 8 个单元写合成 10 写一段
  新：7），不与暖机第一次的单元写同段（第二条流第
- 旧：机第一次的 8 个单元写合成 10 写一段（第二条流第 13–1
  新：，不与暖机第一次的单元写同段（第二条流第 15–2
- 旧：max = 4；整条流里 8 个单元写与上一次发布的 2 个系统
  新：max = 4；整条流里上一次发布的 2 个系统
- 旧：2+2+1+2+2+1+18+2+1+2`，与新池新
  新：2+2+1+2+2+1+2+24+2+1+2`，与新池新
- 旧：件那条流逐段相同、闭式 262165；树表 0 条的一
  新：件那条流逐段相同、闭式 16777240（同一份
- 旧：槽写与下一次发布的单元写落在同一段（新池新建文件那条
  新：，不与下一次发布的单元写同段；同一块盘上连着的屏障并成
- 旧：一段（新池新建文件那条流里暖机第二次的 2 个系统配置写与 24 个单元写合成 26 个写的一段，取号的 2 个系统配置写与暖机第一次开头的屏障合成开头一段，整条流 `2+2+1+2
  新：一道。新池新建文件那条流是整条流 `2+2+1+2
- 旧：+2+1+2`、67108885 个状态，种类 `[sy
  新：+2`、16777240 个状态，种类 `[sy
- 旧：rd×2,barrier]|[root_record_fua]|[unit_write×24,system_c
  新：rd×2,barrier×2]|[root_re
- 旧：unit_write×24,system_configuration_slot×2,barrier]|[journal_record×2,barrier]|[root_reco
  新：×2,barrier×2]|[root_reco
- 旧：）；mkfs 末尾有屏障、空发布开头有屏障，那两道接缝是切开的。段
  新：）。mkfs 末尾有屏障，mkfs 与取号之间的
- 旧：屏障、空发布开头有屏障，那两道接缝是切开的。段序列按整
  新：。mkfs 末尾有屏障，mkfs 与取号之间的接缝是切开的。段序列按整
- 旧：ck_tier=232`；第二条流的段序列 `2+2+1+2+2+1+26+2+1+26+2+1+4+18+2+1+18+2+1+18+2+1+26+2+1+22+2+1+30+2+1+30+2+1+30+2+1+30+2+1+30+2+1+4+16+2+1+18+2+1+30+2+1+4+16+2+1+18+2+1+2`、477 次写、6649413746 个状态（19 条根槽写、61 段，状态
  新：ck_tier=232`，快档 232 个状态（
- 旧：9 条根槽写、61 段，状态数按闭式算）；平时快档按甲二枚举只有 23
  新：k_tier=232`，快档 232 个状态（C
- 旧：态数按闭式算）；平时快档按甲二枚举只有 232 个状态（D13
  新：tier=232`，快档 232 个状态（C57
- 旧：举只有 232 个状态（D13（验证路线） 已定项 9
  新：，快档 232 个状态（C577（系统配置没见证到的最新
- 旧：232 个状态（D13（验证路线） 已定项 9），门禁 54 号 `--full` 在 release 下全量跑那 6649413746 个状态、带断点续跑，52 号核这句与用例里的数组、闭式、写数相符。没有干跑产物，装置钉住：`crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs` 的 `prepa
  新：32 个状态（C577（系统配置没见证到的最新根
- 旧：B 的 2 个系统配置槽写与重开取号的 2 个系统配置槽写合
  新：状态（C577（系统配置没见证到的最新根，乙罩不到） 之
- 旧：系统配置槽写与重开取号的 2 个系统配置槽写合成的 4 写一段（进程退出与重开之间没有屏障，录制流按设备连着记）；
  新：77（系统配置没见证到的最新根，乙罩不到） 之前的数，
- 旧：退出与重开之间没有屏障，录制流按设备连着记）；D 挂着时直接向前发
  新：统配置没见证到的最新根，乙罩不到） 之前的数，重量归提交
- 旧：障，录制流按设备连着记）；D 挂着时直接向前发布、不再重开进程与重新取号，第 26 段是它的 20 个单元写并上 C 的 2 个系统配置槽写合成的一段（22）；D 之后五次覆盖写各把 2
  新：证到的最新根，乙罩不到） 之前的数，重量归提交时
- 旧：成的一段（22）；D 之后五次覆盖写各把 28 个单元写并上前一次的 2 个系统配置
  新：的最新根，乙罩不到） 之前的数，重量归提交时的崩
- 旧：把 28 个单元写并上前一次的 2 个系统配置槽写合
  新：最新根，乙罩不到） 之前的数，重量归提交时的崩溃
- 旧：8 个单元写并上前一次的 2 个系统配置槽写合成一段（30）；抬 F 先写系统配置
  新：新根，乙罩不到） 之前的数，重量归提交时的崩溃验证员）。
- 旧：配置槽写合成一段（30）；抬 F 先写系统配置、与 txg 14 那次轮换合成 4 写一段，两次空发布各 16 个单元写（第一次自成一段、第二次并上轮换成 18）；E 同覆盖写（30）；正常卸载同抬 F 的形状（4、16、18），末尾落在 txg 19 的轮换。
  新：量归提交时的崩溃验证员）。

### .claude/kb/layout/02-second-txn.md

- 旧：，路径不加反引号：门禁 76 号把反引号里以 .rs
  新：，路径不加反引号：门禁 doc-registries 的 second-tx
- 旧：函数名不加反引号：门禁 76 号按反引号里的记号去
  新：函数名不加反引号：门禁 doc-registries 的 second-tx

### .claude/kb/milestone/01-first-txn.md

- 旧：aude/gate.d/53-format-cons
  新：aude/gate.d/doc-registries.
- 旧：placeholders.sh`，样本在 fixture
  新：placeholders 格，样本在 fixture
- 旧：在 fixtures），27 号阶段的常量同步扩到
  新：在 fixtures），code-source-discipline 的 format-co
- 旧：ixtures），27 号阶段的常量同步扩到 `cra
  新：-discipline 的 format-constants 格的常量同步扩到 `cra
- 旧：按段报种类多重集，门禁 52 号拿登记表逐字比对——
  新：按段报种类多重集，门禁 doc-registries 的 segment-r
- 旧：存两个对照都判红，门禁 55 号三次虚机跑合计十几秒
  新：存两个对照都判红，门禁 checker-tier-qemu-device-streams 三次虚机跑合计十几秒。

### .claude/kb/milestone/03-third-txn.md

- 旧：QEMU 真设备（门禁 55 号，只有真实负载、没有崩溃
  新：QEMU 真设备（门禁 checker-tier-qemu-device-streams，只有真实负载、没有崩溃
- 旧：*开工前要先定的**：按剖析的数定先优化哪一段；增量
  新：一侧自己的表示（位图、按段的整数子集编码这类，推的），与第四项 GPU 的数据结构一起设计。第一
- 旧：要先定的**：按剖析的数定先优化哪一段；增量枚举（相邻状态
  新：，与第四项 GPU 的数据结构一起设计。第一轮判决（`
- 旧：：按剖析的数定先优化哪一段；增量枚举（相邻状态只差一次写）、
  新：项 GPU 的数据结构一起设计。第一轮判决（`research/p
- 旧：先优化哪一段；增量枚举（相邻状态只差一次写）、按镜像
  新：构一起设计。第一轮判决（`research/prompts/m3-prune-gpu-r1-main-verification.md`「逐格判决」一节）：状态表示采用位图这一格站
- 旧：一段；增量枚举（相邻状态只差一次写）、按镜像去重各自
  新：「逐格判决」一节）：状态表示采用位图这一格站住（本地攻方两份干
- 旧：增量枚举（相邻状态只差一次写）、按镜像去重各自的收益——按镜像去重对记录核对器不适用（D13（验证路线） 已
  新：）：状态表示采用位图这一格站住（本地攻方两份干净样本量
- 旧：对记录核对器不适用（D13（验证路线） 已定项 7：记录核对器的入参含持久集合）。
  新：状态 8 字节，段长 16–28）。
- 旧：-smi 现查）；门禁 94 号要求 checker
  新：-smi 现查）；门禁 checker-independence-and-sync 的 checker-i
- 旧：7 / LKMM（门禁 57 号）要不要为它加 litm
  新：7 / LKMM（门禁 checker-tier-lkmm）要不要为它加 litm
- 旧：与本里程碑3相关的内容 /home/fy5090/code/singlefs/.claude/kb/c
  新：与本里程碑3相关的内容 .claude/kb/c
- 旧：**开工前要先定的**：选型挪到里程碑二先做（用户 2
  新：**开工前要先定的**：「并发」是真的并行提交；herd7 / LKMM（门禁 checker-tier-lkmm）要不要为它加 litmus。暂定目标、优先级最低，不在出口里，做不到转给里程碑四（用户 2
- 旧：定的**：选型挪到里程碑二先做（用户 2026-09-
  新：出口里，做不到转给里程碑四（用户 2026-09-
- 旧：2026-09-27 定；问题单 `research/prompts/m2-crash-store-r1-forks.md`）；里程碑三接着定块多大
  新：2026-09-27 定）。
- 旧：1-forks.md`）；里程碑三接着定块多大、块键里带哪些指纹；两台机器各写各的库还是一个库，怎么合并；违例表与判定向量表的形态；与第一项、第四项一起定。
  新：026-09-27 定）。

### .claude/kb/prior-art.md

- 旧：`/home/fy5090/code/fs-ref
  新：`~/code/fs-ref
- 旧：aude/gate.d/70-citations.s
  新：aude/gate.d/doc-registries.
- 旧：70-citations.sh`）——**源码树不在也判
  新：的 citations 格）——**源码树不在也判
- 旧：-08 在本机固定点 `/home/fy5090/code/fs-ref
  新：-08 在本机固定点 `~/code/fs-ref

### .claude/kb/tooling.md

- 旧：g-AWQ-4bit`，vLLM 承载，`max_mode
  新：g-AWQ-4bit`，本地模型服务承载，`max_mode
- 旧：通了**：传越界值时上游 vLLM 直接拒绝
  新：通了**：传越界值时上游本地模型服务直接拒绝
- 旧：aude/gate.d/15-research-bu
  新：aude/gate.d/checker-tier-research-bu
- 旧：aude/gate.d/20-kb-shape.sh
  新：aude/gate.d/doc-decisions.s
- 旧：/20-kb-shape.sh` 的条数检查
  新：` 的 kb-shape 格 的条数检查
- 旧：| 路径 | `/home/fy5090/kbuild/linu
  新：| 路径 | `~/kbuild/linu
- 旧：个文件。查源码要用 `/home/fy5090/kbuild/linu
  新：个文件。查源码要用 `~/kbuild/linu
- 旧：e 一次性任务按系统时钟 UTC 写成 `23 22 16
  新：e 一次性任务按系统时钟写成 `23 22 16
- 旧：表里；后台 sleep 在 22:30:00 UTC 按时退出、叫醒了会话。
  新：表里；后台 sleep 按时退出、叫醒了会话。
- 旧：地时区」解释，客户端若按东京时间，那个时刻建任务时已
  新：地时区」解释，客户端若按用户本地时区，那个时刻建任务时已
- 旧：经过去；本机系统时钟是 UTC、人在 JST。
  新：与人所在的时区不同（差 9 小时）。
- 旧：统时钟是 UTC、人在 JST。
  新：所在的时区不同（差 9 小时）。
- 旧：个 Claude 会话在 `/home/fy5090/code/singlefs` 并行工作，
  新：个 Claude 会话在本仓的同一个工作区里并行工作，
- 旧：| 只暂存自己的块 | `research
  新：| 54-layer0-replay（层 0 全量） | 用例标 `#[ig
- 旧：| 只暂存自己的块 | `research/scripts/stage-mine.py --match 我的标记 --foreign
  新：y（层 0 全量） | 用例标 `#[ignore]
- 旧：--match 我的标记 --foreign 别人的标记 文件…`，先 `--dry-run`
  新：0 全量） | 用例标 `#[ignore]`
- 旧：的标记 文件…`，先 `--dry-run` 看它挑了哪几块；`research/scripts/check-staged.sh` 在临时 worktree 上只拿 HEAD + 暂存区跑 kb 阶段。它会把 `.gitignore` 掉的 `
  新：全量） | 用例标 `#[ignore]`、住 `
- 旧：`.gitignore` 掉的 `.claude/si
  新：`#[ignore]`、住 `crates/sin
- 旧：re` 掉的 `.claude/singlefs-a
  新：ore]`、住 `crates/singlefs-
- 旧：inglefs-ai-sop/` 拷进 worktr
  新：ker-tier/tests/`（D13（验证路线）
- 旧：fs-ai-sop/` 拷进 worktree；手工
  新：`（D13（验证路线） 已定项 15），`cargo
- 旧：拷进 worktree；手工建的 worktree 没有这一步，`gat
  新：们 | 门禁在 release 下跑它，每个状态加池
- 旧：建的 worktree 没有这一步，`gate.sh` 退
  新：门禁在 release 下跑它，每个状态加池级 che
- 旧：tree 没有这一步，`gate.sh` 退出码 127、日志
  新：1 s；全量在跑时换 `CARGO_TARGET_DIR` 就能同时编别的测试
- 旧：步，`gate.sh` 退出码 127、日志只有一行 No such file，要先 `rsync -a --exclude .git` 把副本拷进去 |
  新：TARGET_DIR` 就能同时编别的测试 |
- 旧：一个没入库的实验号 ⇒ 10 / 86 号红）⇒ 一
  新：一个没入库的实验号 ⇒ 门禁 doc-registr
- 旧：的实验号 ⇒ 10 / 86 号红）⇒ 一个提交带全
  新：-registries 的 experiment-
- 旧：轮全量门禁。加完先跑 `20 21 22 42 31
  新：轮全量门禁。加完先跑 `doc-decisions doc-registr
- 旧：门禁。加完先跑 `20 21 22 42 31 43 49 52` 与 doc-lint
  新：c-decisions doc-registries` 与 doc-lint
- 旧：| 42 号（三份文件挂钩） |
  新：| doc-registries 的 first-txn
- 旧：| 22 号（引用归属） | 行
  新：| doc-decisions 的 item-ref-
- 旧：` | 新立分项之后跑 22 号，把翻掉的裸引用补上
  新：` | 新立分项之后跑 doc-decisions 的 item-ref-
- 旧：| 43 号（欠账表形状） |
  新：| doc-registries 的 table-sha
- 旧：| 52 号（段序列登记表） |
  新：| doc-registries 的 segment-r
- 旧：| 30 / 32 / 49 号
  新：| 门禁 doc-decisio
- 旧：| 30 / 32 / 49 号（变更史） | `（其四十
  新：c-decisions 的变更史几格（entry-
- 旧：）` 已被别的日期用过；49 号 `--write` 之后
  新：）` 已被别的日期用过；`--write` 之后
- 旧：| 27 号（格式常量） | k
  新：| code-source-discipline 的 format-co
- 旧：对时间 | 写 ISO 时间：`-newermt '
  新：对时间 | 写 ISO 日期：`-newermt '
- 旧：'2026-09-16T12:20:00'`（本机时钟是 UTC
  新：'2026-09-16'` |
- 旧：6T12:20:00'`（本机时钟是 UTC） |
  新：2026-09-16'` |

### .claude/kb/verification-build.md

- 旧：现清单与门禁 54 / 55 号的 `gate-cove
  新：r0-replay 与 checker-tier-qemu-device-streams的 `gate-cove
- 旧：QEMU 真设备在门禁 55 号（只跑新池新建文件）。
  新：QEMU 真设备在门禁 checker-tier-qemu-device-streams（只跑新池新建文件）。
- 旧：了第一个被测对象（门禁 55 号）：新池新建文件在两块真
  新：了第一个被测对象（门禁 checker-tier-qemu-device-streams）：新池新建文件在两块真
- 旧：aude/gate.d/33-mutation-ta
  新：aude/gate.d/code-source-disc
- 旧：ation-tables.sh` | checker 的
  新：ation-tables 格 | checker 的
- 旧：aude/gate.d/27-format-cons
  新：aude/gate.d/code-source-disc
- 旧：at-constants.sh` | 实现与 check
  新：at-constants 格 | 实现与 check
- 旧：| 7.58，内核树 `/home/fy5090/linux-bug-f
  新：| 7.58，内核树 `~/linux-bug-f

### .claude/kb/vm-harness.md

- 旧：g.rs`，判据在门禁 55 号（`.claude/ga
  新：g.rs`，判据在门禁 checker-tier-qemu-device-streams（`.claude/ga
- 旧：aude/gate.d/55-qemu-device
  新：aude/gate.d/checker-tier-qemu-device
- 旧：没报，为什么没查。门禁 55 号因此默认传 `VM_
  新：没报，为什么没查。门禁 checker-tier-qemu-device-streams 因此默认传 `VM_L
- 旧：谁在拦：门禁 65 号（`resea
  新：谁在拦：共享 `gate.sh` 内
- 旧：谁在拦：门禁 65 号（`research
  新：谁在拦：共享 `gate.sh` 内置的「转发计时」阶段
- 旧：65 号（`research/scripts/rel
  新：e/singlefs-ai-sop/scripts/rel

### .claude/main-agent.md

- 旧：「并行写回」：各自不跑 21、49 号 `--write`
  新：「并行写回」：各自不跑 `bash .claude/gat
- 旧：一次 `--write` 与 kb 门禁）。什么时候一起验证：一
  新：一次 `--write`）。什么时候一起验证：一
- 旧：自证过的不重证，整表归 59 号），交回；子 agent
  新：自证过的不重证，整表归 checker-tier-crates-mutation-replay），交回；子 agent
- 旧：，交回；子 agent 各自名下的门禁阶段照定义在交回前跑，整批门禁在合入
  新：，交回；子 agent 交回前、合入后验证都不跑
- 旧：的门禁阶段照定义在交回前跑，整批门禁在合入后验证交回之后跑一次
  新：；子 agent 交回前、合入后验证都不跑门禁阶段
- 旧：，整批门禁在合入后验证交回之后跑一次；代码三方一轮攻这
  新：阶段，扫仓的门禁只在提交前由 `gate-triage` 跑的 `research
- 旧：派崩溃验证员时给 54、55、57 号各自的内存上限
  新：er0-replay`、checker-tier-qemu-device-streams、checker-tie
- 旧：验证员时给 54、55、57 号各自的内存上限，每道
  新：ice-streams、checker-tier-lkmm 各自的内存上限，每道一
- 旧：d`「输入」一节给：54、57 号写明是量过的还是推的
  新：`「输入」一节给：`54-layer0-replay` 与 checker-t
- 旧：号写明是量过的还是推的，55 号的不小于同时起的虚机
  新：写明是量过的还是推的，checker-tier-qemu-device-streams 的不小于同时起的虚机数
- 旧：机数乘每台的内存。门禁 15、74 号在阶段里面经内存包装
  新：机数乘每台的内存。门禁 checker-tier-research-build-and-replay 的 research-
- 旧：rence.md`「核查员按轮派」派 `three-way-verifier`（有腿交了模型、产物或复跑命令就派，输入里给派腿时主 agent 记下的
  新：rence.md`「核查与判决由主 agent 做」：逐
- 旧：给派腿时主 agent 记下的 `date -u`，
  新：与判决由主 agent 做」：逐条核腿报告里的原文引用、产物行与复跑
- 旧：时主 agent 记下的 `date -u`，代码轮另给开工快照；不
  新：做」：逐条核腿报告里的原文引用、产物行与复跑命令，代码轮对着开工快照核腿
- 旧：date -u`，代码轮另给开工快照；不派的在判决里
  新：产物行与复跑命令，代码轮对着开工快照核腿引的行；不派
- 旧：代码轮另给开工快照；不派的在判决里写明为什么）→ 主 agent 写
  新：开工快照核腿引的行；不派核查员）；本地腿派攻方还是辩方
- 旧：派的在判决里写明为什么）→ 主 agent 写判决；本地腿派攻方还是辩方由
  新：核腿引的行；不派核查员）；本地腿派攻方还是辩方由
- 旧：sync.md`（格式见门禁 68 号文件头，68
  新：sync.md`（格式见 `.claude/ga
- 旧：c.md`（格式见门禁 68 号文件头，68 号判形
  新：ync.md`（格式见 `.claude/gate.d/doc-process-records.sh` 文件头 knowled
- 旧：见门禁 68 号文件头，68 号判形式）→ 下一行 |
  新：e-sync 格那一段，那一格判形式）→ 下一行 |
- 旧：-verifier` 跑它那几道，命令带 `SINGL
  新：-verifier` 跑这四道：`54-layer0
- 旧：按输入哈希写全绿标记），55、57、59 同样带前缀、同样按各自输
  新：按输入哈希写全绿标记），其余三道同样带前缀、同样按各自输
- 旧：变了的、判绿写全绿标记（55、57 与 54 号并行
  新：变了的、判绿写全绿标记（checker-tier-qemu-device-streams、checker-tie
- 旧：、判绿写全绿标记（55、57 与 54 号并行，59
  新：ice-streams、checker-tier-lkmm 与 `54-layer
- 旧：57 与 54 号并行，59 号与 54 号串行，内
  新：-replay` 并行，checker-tier-crates-mutation-replay 与 `54-layer
- 旧：-green`）并分诊（54、55、57、59 在 `gate.sh`
  新：-green`）并分诊（这四道在 `gate.sh`
- 旧：表里对应那几行（门禁 75 号那条双向检查两侧要同
  新：表里对应那几行（门禁 doc-experiments 的 decision-
- 旧：的活）；它翻了分项状态、33 号红（变异表锚点腐化）
  新：的活）；它翻了分项状态、门禁 code-source
- 旧：或由用户逐份豁免（门禁 72 号） |
  新：或由用户逐份豁免（门禁 doc-process-records 的 agent-def

### .claude/rules/format-evolution.md

- 旧：- 决策变更**必须记进**决策变更史（原文写进当月的 `.claude/kb
  新：组织形态见 `.claude/ru
- 旧：月的 `.claude/kb/decisions-h
  新：态见 `.claude/rules/changelog-f
- 旧：claude/kb/decisions-history/
  新：rules/changelog-format.md
- 旧：e/kb/decisions-history/<年-
  新：les/changelog-format.md`）
- 旧：b/decisions-history/<年-月>.md
  新：s/changelog-format.md`），含
- 旧：sions-history/<年-月>.md`，标题下写两行快查
  新：hangelog-format.md`），含推翻
- 旧：tory/<年-月>.md`，标题下写两行快查，再跑 49 号 --write 重新生成 `.cl
  新：angelog-format.md`），含推翻依据
- 旧：49 号 --write 重新生成 `.claude/kb/decisions-history.md`），含推翻依据；
  新：gelog-format.md`），含推翻依据；
- 旧：并把推翻依据写进当月的决策变更史。
  新：并把推翻依据写进决策变更史。
- 旧：这一种括注；写了括注时 20 号判条数对不对、75
  新：种括注；写了括注时门禁 doc-decisions 的 kb-shape
- 旧：20 号判条数对不对、75 号判形态，没写括注两道
  新：ape 格判条数对不对、门禁 doc-experim
- 旧：**状态：已定。**」（20 号要的规范标记，不算字
  新：**状态：已定。**」（门禁 doc-decisio
- 旧：，写在表下那一段里门禁 31 号定位不到：「改新池新
  新：，写在表下那一段里门禁 doc-decisions 的 blocking-
- 旧：项就写几行**——门禁 75 号按分项查依据（`de
  新：项就写几行**——门禁 doc-experiments 的 decision-
- 旧：aude/gate.d/75-decision-exp
  新：aude/gate.d/doc-experime
- 旧：ate.d/75-decision-experiment-
  新：e/gate.d/doc-experiments
- 旧：riment-links.sh` 判表的形状、双向对不对
  新：cision-links 格判表的形状、双向对不
- 旧：路径与结论登记」一节由 `.claude/gate.d/99-multipath-re
  新：论登记」一节由同一道的 multipath-re
- 旧：ath-registry.sh` 判形状、源码落点存在、
  新：ath-registry 格判形状、源码落点存在
- 旧：aude/gate.d/27-format-cons
  新：aude/gate.d/code-source-disc
- 旧：at-constants.sh` 只核 `const 名
  新：at-constants 格只核 `const
- 旧：e=`（`|` 分隔），27 号从此在 kb 正文与
  新：e=`（`|` 分隔），format-constants 格从此在 kb 正文与

### .claude/rules/implementation-first.md

- 旧：aude/gate.d/58-implementat
  新：aude/gate.d/doc-process-rec

### .claude/rules/implementation-workflow.md

- 旧：码，再攻一轮 | 门禁 56 号判形式：这次改动里每
  新：码，再攻一轮 | 门禁 doc-process-records 的 crates-ad
- 旧：cation.md`）、59 号（crates 变异表复
  新：cation.md`）、checker-tier-crates-mutation-replay（crates 变异表复
- 旧：ates 变异表复跑）、33 号（research
  新：ates 变异表复跑）、code-source-discipline 的 mutation-
- 旧：改过的每一份定义，门禁 72 号判形式（形态照 56
  新：改过的每一份定义，门禁 doc-process-records 的 agent-def
- 旧：72 号判形式（形态照 56 号）。
  新：判形式（形态照同一道的 crates-adversarial-review 格）。
- 旧：（放这一轮的材料目录），连同派腿的时刻（`date -
  新：（放这一轮的材料目录），主 agent 写判决时拿它核腿引的行；腿跑着的时候主
- 旧：的材料目录），连同派腿的时刻（`date -u`）交核查员当输入；腿跑着的时候主 age
  新：t 写判决时拿它核腿引的行；腿跑着的时候主 age
- 旧：住别的会话。腿交齐之后、派核查员之前拿快照 `sha25
  新：住别的会话。腿交齐之后、写判决之前拿快照 `sha25
- 旧：倒推出快照时的原样再交核查员——HEAD 之后没被这
  新：，倒推出快照时的原样再核——HEAD 之后没被这
- 旧：，判决开头写明哪个文件、几点、被改了几处、腿引的行落没
  新：，判决开头写明哪个文件、被改了几处、腿引的行落没
- 旧：用判定，每次照跑；54、55、57、59 号都登记着
  新：er0-replay`、checker-tier-qemu-device-streams、checker-tie
- 旧：，每次照跑；54、55、57、59 号都登记着。
  新：ice-streams、checker-tier-lkmm、checker-tie
- 旧：照跑；54、55、57、59 号都登记着。
  新：r-tier-lkmm、checker-tier-crates-mutation-replay 都登记着。
- 旧：数的报「本次未跑」）与 55、57、59 号；整轮门
  新：数的报「本次未跑」）与 checker-tier-qemu-device-streams、checker-tie
- 旧：「本次未跑」）与 55、57、59 号；整轮门禁由
  新：ice-streams、checker-tier-lkmm、checker-tie
- 旧：未跑」）与 55、57、59 号；整轮门禁由 `gate
  新：r-tier-lkmm、checker-tier-crates-mutation-replay；整轮门禁由 `gate
- 旧：测试二进制）、54 / 55 / 57 / 59 /
  新：on-replay 与 checker-tier-research-build-and-replay 的 experimen
- 旧：）、54 / 55 / 57 / 59 / 87 号
  新：nd-replay 的 experiment-replay 格、`qemu-sys
- 旧：4 / 55 / 57 / 59 / 87 号、`qemu-syste
  新：ment-replay 格、`qemu-syste
- 旧：aude/gate.d/57-lkmm.sh`（逻辑
  新：aude/gate.d/checker-tier-lkmm.sh`（逻辑
- 旧：aude/gate.d/55-qemu-device
  新：aude/gate.d/checker-tier-qemu-device
- 旧：在本地配置（仓根 `layer0-shard.env`，
  新：本地配置（仓根 `multi-host.env`，模
- 旧：仓根 `layer0-shard.env`，模板 `la
  新：根 `multi-host.env`，模板 `mu
- 旧：d.env`，模板 `layer0-shard.env.e
  新：env`，模板 `multi-host.env.ex
- 旧：模板 `layer0-shard.env.example
  新：板 `multi-host.env.example

### .claude/rules/mutation-sampling.md

- 旧：（`.claude/gate.
  新：（门禁 code-source-
- 旧：（`.claude/gate.d/33
  新：（门禁 code-source-di
- 旧：（`.claude/gate.d/33-mutat
  新：（门禁 code-source-discipline
- 旧：laude/gate.d/33-mutation-tab
  新：ode-source-discipline 的 mutation-tab
- 旧：ation-tables.sh`，C327（变异表的锚点
  新：ation-tables 格，C327（变异表的锚点
- 旧：杠转义这一种成因由门禁 33 号静态判红——两侧变异
  新：杠转义这一种成因由门禁 code-source-discipline 的 mutation-
- 旧：来才发现的编不过由门禁 59 号单独报成「无效」，
  新：来才发现的编不过由门禁 checker-tier-crates-mutation-replay 单独报成「无效」，
- 旧：读 59 号的输出时按两栏分开数
  新：读 checker-tier-crates-mutation-replay 的输出时按两栏分开数，
- 旧：存撞顶与超时两栏；门禁 59 号照它自己收尾的「计数
  新：存撞顶与超时两栏；门禁 checker-tier-crates-mutation-replay 照它自己收尾的「计数：
- 旧：里带 `$` 这类字符，59 号按字面去找）；都没有记「
  新：里带 `$` 这类字符，它按字面去找）；都没有记「
- 旧：由提交时的崩溃验证员跑 59 号）：
  新：由提交时的崩溃验证员跑 checker-tier-crates-mutation-replay）：

### .claude/rules/path-moves.md

- 旧：档反引号里的路径由门禁 10 号第 4 段兜一部分，
  新：档反引号里的路径由门禁 doc-registries 的 governanc
- 旧：号里的路径由门禁 10 号第 4 段兜一部分，`Ca
  新：-registries 的 governance-
- 旧：自检用例一起改成新布局，改完跑一遍自检，每个弄坏开关仍要判红。搬
  新：自检用例一起改成新布局，每个弄坏开关仍要判红。搬
- 旧：mes.md`、由门禁 90 号判全仓不再出现旧名，
  新：mes.md`、由门禁 doc-text 的 term-rena
- 旧：*决策分项清单** | 20 / 21 号 | 清单
  新：*决策分项清单** | 门禁 doc-decisio
- 旧：清单** | 20 / 21 号 | 清单是正文的投
  新：c-decisions 的 kb-shape、de
- 旧：让截断位置变 ⇒ 跑 `21-decision-it
  新：让截断位置变 ⇒ 跑 `bash .claude/gate.d/doc-decisions.s
- 旧：`21-decision-items-sync.sh --
  新：doc-decisions.sh --write
- 旧：cision-items-sync.sh --write`
  新：oc-decisions.sh --write`
- 旧：响的决策」回看** | 75 号 | 它的判据是「正
  新：响的决策」回看** | 门禁 doc-experim
- 旧：一起换掉了的，换回旧名，40 号才在历史里找得到。分
  新：一起换掉了的，换回旧名，门禁 doc-experim
- 旧：物」当场成假话，而门禁 88 号要到下一次跑才说。
  新：物」当场成假话，而门禁 doc-experiments 的 quoted-re
- 旧：st` 名仍成对，门禁 27 号不红；而 `crat
  新：st` 名仍成对，门禁 code-source-discipline 的 format-co
- 旧：门禁 12 号扫全仓（冻结证据也在
  新：门禁 doc-text 的 prime-mar
- 旧：「链接指向」阶段与门禁 10 号第 4 段顺带罩住
  新：「链接指向」阶段与门禁 doc-registries 的 governanc
- 旧：指向」阶段与门禁 10 号第 4 段顺带罩住 Mar
  新：-registries 的 governance-
- 旧：aude/gate.d/90-term-rename
  新：aude/gate.d/doc-text.sh`）的
- 旧：term-renames.sh`，它按 [.claude
  新：term-renames 格，它按 [.claude

### .claude/rules/three-way-inference.md

- 旧：**核查员按轮派。** 这一轮有腿交了模型、产
  新：3. 判决要引的复跑数：把腿
- 旧：**核查员按轮派。** 这一轮有腿交了模型、产物或复跑命
  新：3. 判决要引的复跑数：把腿的模型目录拷到草稿目录
- 旧：按轮派。** 这一轮有腿交了模型、产物或复跑命令，就
  新：判决要引的复跑数：把腿的模型目录拷到草稿目录，在
- 旧：** 这一轮有腿交了模型、产物或复跑命令，就派 `three-w
  新：要引的复跑数：把腿的模型目录拷到草稿目录，在副本里经 `rese
- 旧：了模型、产物或复跑命令，就派 `three-way-
  新：的模型目录拷到草稿目录，在副本里经 `research/s
- 旧：复跑命令，就派 `three-way-verifier`；
  新：n-with-memory-cap.sh` 跑，
- 旧：`three-way-verifier`；这一轮没有任何腿交模
  新：with-memory-cap.sh` 跑，比输出与报告里抄
- 旧：ay-verifier`；这一轮没有任何腿交模型、产物或复跑命令
  新：mory-cap.sh` 跑，比输出与报告里抄的、比 sha256，不在腿的原目录里跑。
- 旧：er`；这一轮没有任何腿交模型、产物或复跑命令的可以不派，判决里
  新：比 sha256，不在腿的原目录里跑。
- 旧：任何腿交模型、产物或复跑命令的可以不派，判决里写明没派、为什么。
  新：56，不在腿的原目录里跑。
- 旧：两次不一致就照 `.claude/si
  新：3. 判决要引的复跑数：把腿的模型目录拷到草稿目录，在副本里经 `research/s
- 旧：两次不一致就照 `.claude/singlefs-ai
  新：本里经 `research/scripts/run
- 旧：`.claude/singlefs-ai-sop/rul
  新：esearch/scripts/run-with-m
- 旧：ude/singlefs-ai-sop/rules/test-
  新：arch/scripts/run-with-me
- 旧：ai-sop/rules/test-discipline.md` 记「不稳定」，不下结
  新：memory-cap.sh` 跑，比输出与报告里抄
- 旧：cipline.md` 记「不稳定」，不下结论。
  新：ory-cap.sh` 跑，比输出与报告里抄的、比 sha256，不在腿的原目录里跑。
- 旧：md` 记「不稳定」，不下结论。
  新：的、比 sha256，不在腿的原目录里跑。
- 旧：**闸判红之后**：那一轮**作废重跑**，不许记成「三方
  新：**闸判红之后**：那一份不算干净样本，照样重跑取下一份，不许记成「
- 旧：**：那一轮**作废重跑**，不许记成「三方不一致」
  新：份不算干净样本，照样重跑取下一份，不许记成「三方不一致」
- 旧：⚠️ **这不是洁癖**：*
  新：4. 本地腿的译文核对表逐条
- 旧：⚠️ **这不是洁癖**：**损坏的
  新：4. 本地腿的译文核对表逐条核：行号在不在、抄的是不是原文、英文丢没丢或多
- 旧：⚠️ **这不是洁癖**：**损坏的不只是词。**
  新：：行号在不在、抄的是不是原文、英文丢没丢或多没多限定词与括注。
- 旧：*：**损坏的不只是词。**
  新：丢或多没多限定词与括注。
- 旧：确有必要采用带损坏的输出时，设
  新：3. 判决要引的复跑数：把腿的模型
- 旧：确有必要采用带损坏的输出时，设 `ASK_
  新：3. 判决要引的复跑数：把腿的模型目录
- 旧：必要采用带损坏的输出时，设 `ASK_LOCAL_ALLOW_CORRUPT=1`，
  新：抄的、比 sha256，不在腿的原目录里跑。
- 旧：并在结论里写明这一票带瑕疵。
  新：. 腿引的每一行产物，在产物文件里逐字找。
- 旧：并在结论里写明这一票带瑕疵。
  新：每一行产物，在产物文件里逐字找。

### .claude/rules/verification.md

- 旧：部用例；连同 QEMU（55 号）、herd7（57
  新：部用例；连同 QEMU（门禁 checker-tie
- 旧：55 号）、herd7（57 号）、crates 变异整
  新：eams）、herd7（checker-tier-lkmm）、crates 变异整
- 旧：crates 变异整表（59 号）、全部实验复跑（87
  新：crates 变异整表（checker-tier-crates-mutation-replay）、全部实验复跑（che
- 旧：9 号）、全部实验复跑（87 号） | 默认只在提交
  新：lay）、全部实验复跑（checker-tier-research-build-and-replay 的 experimen
- 旧：；源码里零处引它；门禁 94 号判）。两档都要用的一
  新：；源码里零处引它；门禁 checker-independence-and-sync 的 checker-i
- 旧：ck.py` 判，门禁 14 号在真仓上跑。
  新：mes 那一样判，门禁 code-source-discipline 的 test-file
- 旧：- 耗时用例：单线程 debug
  新：- 一条用例一个场景：按参数循环
- 旧：- 耗时用例：单线程 debug 下单条跑到
  新：- 一条用例一个场景：按参数循环、每一轮新建一个池（`build_pool`、`build_through_*`、`format_pool`、`MemoryPool::with_devices`）的，拆
- 旧：耗时用例：单线程 debug 下单条跑到 60 秒及以上的
  新：ool::with_devices`）的，拆成一个带参数的函数加每个取值一条 `#[test]`，
- 旧：线程 debug 下单条跑到 60 秒及以上的用例，标 `#[ignore =
  新：参数的函数加每个取值一条 `#[test]`，红
- 旧：及以上的用例，标 `#[ignore = "harness
  新：数加每个取值一条 `#[test]`，红了只重跑那
- 旧：= "harness 耗时用例：…"]`；要跑随时跑，一律经
  新：ne-scenario <理由>`。`research/
- 旧：ss 耗时用例：…"]`；要跑随时跑，一律经 `research/sc
  新：enario <理由>`。`research/sc
- 旧：ch/scripts/run-with-memory-cap
  新：h/scripts/crash-case-check
- 旧：pts/run-with-memory-cap.sh`，线程数
  新：cripts/crash-case-check.
- 旧：h-memory-cap.sh`，线程数按派发提示的上
  新：case-check.py` 的 one-scen
- 旧：mory-cap.sh`，线程数按派发提示的上限。轻重由量出来的数定，不由整份全量日志里 libtest 的「has been
  新：se-check.py` 的 one-scena
- 旧：志里 libtest 的「has been runni
  新：-check.py` 的 one-scenario 那一样
- 旧：ibtest 的「has been running f
  新：.py` 的 one-scenario 那一样判，
- 旧：t 的「has been running for over 60 seconds」定：`python3 research/scripts
  新：` 的 one-scenario 那一样判，门禁
- 旧：thon3 research/scripts/harness
  新：的 one-scenario 那一样判，门禁 h
- 旧：esearch/scripts/harness-test
  新：one-scenario 那一样判，门禁 harness-mode
- 旧：s/harness-test-timing.py m
  新：harness-model-differentia
- 旧：ness-test-timing.py measur
  新：l-differential-and-scenarios
- 旧：s-test-timing.py measure`（只量变了
  新：ferential-and-scenarios 的 on
- 旧：timing.py measure`（只量变了的目标加 `--targets`）把单条耗
  新：tial-and-scenarios 的 one-
- 旧：加 `--targets`）把单条耗时写进 harness 档包根下的
  新：rios 的 one-scenario 格在真仓上跑它
- 旧：）把单条耗时写进 harness 档包根下的耗时表 te
  新：的 one-scenario 格在真仓上跑它。
- 旧：时写进 harness 档包根下的耗时表 test-timing.tsv（第一次量完才有这份文件），再 `apply` 按表统一标上或摘掉，表与标记一起提
  新：ne-scenario 格在真仓上跑它。
- 旧：pply` 按表统一标上或摘掉，表与标记一起提交；不手标、不手摘。门禁 14 号判标记与
  新：enario 格在真仓上跑它。
- 旧：起提交；不手标、不手摘。门禁 14 号判标记与表对得上。
  新：rio 格在真仓上跑它。
- 旧：ck.py` 判，门禁 14 号在真仓上跑它。
  新：rio 那一样判，门禁 harness-model-differential-and-scenarios 的 one-scena
- 旧：55、57、59、87 号照各自的复用判定跑（
  新：- 阶段归属表 `.cla
- 旧：55、57、59、87 号照各自的复用判定跑（`research/sc
  新：- 阶段归属表 `.claude/gat
- 旧：的复用判定跑（`research/scripts/stage-must-
  新：.claude/gate.d/stage-owner
- 旧：ripts/stage-must-run.sh` 文
  新：ate.d/stage-owners.tsv` 一行一个门
- 旧：s/stage-must-run.sh` 文件头）。
  新：age-owners.tsv` 一行一个门禁与格
- 旧：ust-run.sh` 文件头）。
  新：owners.tsv` 一行一个门禁与格名：门禁、格名、判红时派给谁修、为什么。
- 旧：线） 已定项 5，门禁 94 号判）；它今天只依赖
  新：线） 已定项 5，门禁 checker-independence-and-sync 的 checker-i
- 旧：encies]`，这一句 94 号不单判。
  新：encies]`，这一句那一格不单判。
- 旧：规定的方法名不判。门禁 13 号判；还没改完名的文件
  新：规定的方法名不判。门禁 code-source-discipline 的 vague-nam
- 旧：拍跑了、每段都判过 | 74 号；覆盖声明 `# g
  新：拍跑了、每段都判过 | 门禁 harness-mod
- 旧：| 崩溃枚举用例住在 checker
  新：| harness 档一条用例一个场景 | 门禁
- 旧：| 崩溃枚举用例住在 checker 档、标
  新：arness 档一条用例一个场景 | 门禁 harnes
- 旧：崩溃枚举用例住在 checker 档、标了 `#[i
  新：景 | 门禁 harness-model-differential-and
- 旧：用例住在 checker 档、标了 `#[ignore]` 的登记
  新：model-differential-and-scen
- 旧：档、标了 `#[ignore]` 的登记了 `crash
  新：and-scenarios 的 one-scena
- 旧：#[ignore]` 的登记了 `crash-case
  新：-scenarios 的 one-scenari
- 旧：e]` 的登记了 `crash-case:` | `research
  新：的 one-scenario 格（跑 `resear
- 旧：se-check.py`（47 号跑它的自证） |
  新：ne-scenario`） |
- 旧：名不只由空泛词拼成 | 13 号，词表 `.clau
  新：文件不按里程碑起名 | 门禁 code-source
- 旧：由空泛词拼成 | 13 号，词表 `.claude/
  新：按里程碑起名 | 门禁 code-source-discipline：vague-names 格读词表 `.claude/
- 旧：checker 档 | 94 号 |
  新：checker 档 | 门禁 checker-ind
- 旧：| 用例住对档、一条一个场景、测试文件不按里程碑起名、checker 档测试文件声明模块、harness 耗时用例
  新：| harness 档一条用
- 旧：明模块、harness 耗时用例标记与耗时表对得上
  新：| harness 档一条用例一个场景 | 门禁
- 旧：harness 耗时用例标记与耗时表对得上 | 14 号（跑 `r
  新：arness 档一条用例一个场景 | 门禁 harnes
- 旧：标记与耗时表对得上 | 14 号（跑 `resear
  新：档一条用例一个场景 | 门禁 harness-mod
- 旧：check.py` 与 `research/scri
  新：k.py --only one-scenario`）
- 旧：y` 与 `research/scripts/harness
  新：y one-scenario`） |
- 旧：esearch/scripts/harness-test-timing.py check`） |
  新：one-scenario`） |
- 旧：词筛得到点名的测试 | 33 号 |
  新：词筛得到点名的测试 | 门禁 code-source
- 旧：第一个参数当项目根 | 62 号 |
  新：第一个参数当项目根 | 门禁 code-toolin
- 旧：装置二进制的内联测试）、55 / 57 / 59 /
  新：装置二进制的内联测试）、门禁 checker-tie
- 旧：的内联测试）、55 / 57 / 59 / 87 号
  新：tion-replay 与 checker-tie
- 旧：）、55 / 57 / 59 / 87 号、QEMU
  新：-and-replay 的 experiment-
- 旧：/ 57 / 59 / 87 号、QEMU、herd7、
  新：ment-replay 格、QEMU、herd7、
- 旧：*它们管不到的**：耗时表是不是最近量的（表头写着量
  新：*：harness 耗时用例标得对不对、快档抽的取样点够不
- 旧：管不到的**：耗时表是不是最近量的（表头写着量的那次提交）、快档抽的取样点够不够、
  新：ess 耗时用例标得对不对、快档抽的取样点够不够、

### .claude/skills/crash-test/SKILL.md

- 旧：二进制与它们的用例）与 55、57、59、87 号
  新：制与它们的用例）与门禁 checker-tier-qemu-device-streams、checker-tie
- 旧：与它们的用例）与 55、57、59、87 号 | 默
  新：ice-streams、checker-tier-lkmm、checker-tie
- 旧：的用例）与 55、57、59、87 号 | 默认只在
  新：r-tier-lkmm、checker-tier-crates-mutation-replay、checker-tie
- 旧：）与 55、57、59、87 号 | 默认只在提交时
  新：tion-replay、checker-tier-research-build-and-replay 的复跑那一格 | 默认
- 旧：aude/gate.d/74-model-diffe
  新：aude/gate.d/harness-model-diffe
- 旧：rential.sh`（轻阶段，整轮门禁里跑） | `
  新：enarios.sh`（harness 类，整轮门禁里跑） | `
- 旧：aude/gate.d/55-qemu-device
  新：aude/gate.d/checker-tier-qemu-device
- 旧：aude/gate.d/57-lkmm.sh`，逻辑
  新：aude/gate.d/checker-tier-lkmm.sh`，逻辑
- 旧：里只有崩溃验证员（54、55、57、59 号）与门禁
  新：yer0-replay、checker-tier-qemu-device-streams、checker-tie
- 旧：崩溃验证员（54、55、57、59 号）与门禁分诊员
  新：ice-streams、checker-tier-lkmm、checker-tie
- 旧：证员（54、55、57、59 号）与门禁分诊员（整轮门禁
  新：r-tier-lkmm、checker-tier-crates-mutation-replay）与门禁分诊员（整轮门禁

### .claude/skills/decide/SKILL.md

- 旧：ions.md`，变更史按月写进 `.claude/
  新：ions.md`，变更史写进 `.claude/
- 旧：ions-history/<年-月>.md` 再跑 49 号
  新：ions-history.md` 对应决策的节里
- 旧：y/<年-月>.md` 再跑 49 号 `--write`。格
  新：history.md` 对应决策的节里，组织形态见 `.claude/ru
- 旧：` 再跑 49 号 `--write`。格式与写法以
  新：s/changelog-format.md`。格式与

### .claude/skills/gate/SKILL.md

- 旧：ard.sh` 拒。平时要快速反馈，单跑 `.claude/
  新：t 都只在提交时跑，平时不单跑（同一个钩子在执行前
- 旧：拒。平时要快速反馈，单跑 `.claude/gate.d/` 下 54、55、57、59、87 之外的阶段。共享正文的阶段表不全，
  新：只在提交时跑，平时不单跑（同一个钩子在执行前拒）。共享正文的阶段表不全，

### CLAUDE.md

- 旧：大范围。改完记进那份计划，跑相关门禁：47、62、63、
  新：大范围。改完记进那份计划；门禁、样本与脚本自证归提
- 旧：记进那份计划，跑相关门禁：47、62、63、doc-l
  新：。改完记进那份计划；门禁、样本与脚本自证归提交时
- 旧：计划，跑相关门禁：47、62、63、doc-lint，改了定
  新：改完记进那份计划；门禁、样本与脚本自证归提交时，由提交前 `gate-triage` 跑的 `research/scripts/gate-staged.sh` 判，改完一样
- 旧：禁：47、62、63、doc-lint，改了定义与共用约束的走
  新：/gate-staged.sh` 判，改完一样都不跑。改了定
- 旧：轮三方或由用户逐份豁免（72 号判），改了触发文件的
  新：轮三方或由用户逐份豁免（门禁 doc-process
- 旧：发文件的写阶段同步记录（68 号判）；共享的「本地阶
  新：发文件的写阶段同步记录（门禁 doc-process
- 旧：的写阶段同步记录（68 号判）；共享的「本地阶段判别力」「编
  新：的写阶段同步记录（门禁 doc-process-records 的 knowledge-
- 旧：录（68 号判）；共享的「本地阶段判别力」「编号与简称」两道单跑 `bash .clau
  新：ss-records 的 knowledge-s
- 旧：「编号与简称」两道单跑 `bash .claude/singlefs-ai-so
  新：s-records 的 knowledge-syn
- 旧：laude/singlefs-ai-sop/scripts/stage-selftest.
  新：rds 的 knowledge-sync 格判）。
- 旧：scripts/stage-selftest.sh .claude/gate.d` 与 `bash .claude/singlefs-ai-sop/scripts/number-name-sync.sh .`
  新：s 的 knowledge-sync 格判）。
- 旧：er-name-sync.sh .`，不是重型。
  新：owledge-sync 格判）。
- 旧：ame-sync.sh .`，不是重型。
  新：wledge-sync 格判）。
- 旧：| 决策变更史按决策的汇总，由 49 号 `--w
  新：一条决策一节、节内是它的完整历史，组织形态见 `.cla
- 旧：决策变更史按决策的汇总，由 49 号 `--write` 从原文生成；原文按月在 `.claude/kb/decisions-history/<年-月>.md`，怎么写见 `.claude/r
  新：节、节内是它的完整历史，组织形态见 `.claude/r
- 旧：rules/format-evolution.md`「硬约束」 |
  新：gelog-format.md` |
- 旧：volution.md`「硬约束」 |
  新：g-format.md` |
- 旧：定项 4 那张表；门禁 93 号判两处逐位一致 |
  新：定项 4 那张表；门禁 code-source-discipline 的 feature-b
- 旧：新名 / 匹配），门禁 90 号按它查全仓不再出现旧
  新：新名 / 匹配），门禁 doc-text 的 term-rena
- 旧：结、依据哪条分项；门禁 16 号按它判三条（每层、每
  新：结、依据哪条分项；门禁 doc-decisions 的 freeze-la
- 旧：| `research/scripts/check-
  新：.claude/kb/experiments-hist
- 旧：h/scripts/check-staged.sh` | 在
  新：ude/rules/changelog-form
- 旧：check-staged.sh` | 在临时 work
  新：og-format.md` |
- 旧：staged.sh` | 在临时 worktree 上只拿「HEAD + 暂存区」跑 doc-lint 与快的 kb 阶段 |
  新：format.md` |

### README.md

- 旧：| 模型对拍（门禁 74 号） | 随机历史的快档加
  新：| 模型对拍（门禁 harness-model-differential-and-scenarios） | 随机历史的快档加
- 旧：QEMU 真设备（门禁 55 号） | 两块 virti
  新：QEMU 真设备（门禁 checker-tier-qemu-device-streams） | 两块 virti
- 旧：| 内存序（门禁 57 号） | herd7 判
  新：| 内存序（门禁 checker-tier-lkmm） | herd7 判
- 旧：入的最终判据。今天门禁 55 号只接了真实负载与设备
  新：入的最终判据。今天门禁 checker-tier-qemu-device-streams 只接了真实负载与设备侧
- 旧：的理想模型比对。由门禁 74 号跑：随机历史的快档加
  新：的理想模型比对。由门禁 harness-model-differential-and-scenarios 跑：随机历史的快档加五
- 旧：取样点，一共六段（段名以 74 号的 `SECTIONS`
  新：共六段，每段一格（段名以它文件头的格名表为准），每一步拿
- 旧：六段（段名以 74 号的 `SECTIONS` 为准），每一步拿实现的结
  新：段一格（段名以它文件头的格名表为准），每一步拿实现的结
- 旧：aude/gate.d/55-qemu-device
  新：aude/gate.d/checker-tier-qemu-device
- 旧：级到软件模拟**。门禁 55 号要 `qemu-sy
  新：级到软件模拟**。门禁 checker-tier-qemu-device-streams 要 `qemu-sys
- 旧：不分片：没有下面这份配置，或者配置判不过，门禁 54
  新：不分片：没有下面这份配置、配置判不过，或者配置里的
- 旧：${SINGLEFS_LAYER0_SHARD_CONFI
  新：SINGLEFS_MULTI_HOST_CONFIG
- 旧：EFS_LAYER0_SHARD_CONFIG:-<主工
  新：FS_MULTI_HOST_CONFIG:-<主工
- 旧：:-<主工作树的根>/layer0-shard.env}`
  新：<主工作树的根>/multi-host.env}`，
- 旧：的根>/layer0-shard.env}`，已被 gi
  新：根>/multi-host.env}`，已被 gi
- 旧：进仓；从仓根的模板 `layer0-shard.env.e
  新：；从仓根的模板 `multi-host.env.ex
- 旧：模板 `layer0-shard.env.example
  新：板 `multi-host.env.example
- 旧：值不做 shell 展开，七个键都要写：
  新：值不做 shell 展开。前八个键都要写，最后两个开关
- 旧：条必须变红的测试，门禁 59 号复跑 |
  新：条必须变红的测试，门禁 checker-tier-crates-mutation-replay 复跑 |
- 旧：lkmm.sh`（门禁 57 号的逻辑）、`fetc
  新：lkmm.sh`（门禁 checker-tier-lkmm 的逻辑）、`fetch
- 旧：| [`layer0-shard.env.e
  新：| [`multi-host.env.ex
- 旧：| [`layer0-shard.env.example
  新：[`multi-host.env.example
- 旧：.example`](layer0-shard.env.e
  新：xample`](multi-host.env.ex
- 旧：e`](layer0-shard.env.example
  新：`](multi-host.env.example
- 旧：| 层 0 双机分片本地配置的模板，见「层 0 全量分到两
  新：y 的双机分片、跨机脚本 `research/scripts/multi-host-run.sh` 共用；双机与 GPU 两个开关默认关），见「层 0 全量分到两

