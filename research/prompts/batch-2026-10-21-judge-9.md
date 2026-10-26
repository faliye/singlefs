# 阶段同步第二段逐行判：batch-2026-10-21，组 F73-F85

基准 1bb3a24dc，结束「工作区」（HEAD 此刻是 841b3beab）。候选表 /home/fy5090/.local/share/singlefs/work/batch-2026-10-21-sync-facts/candidates.tsv（sha256 开头 a2913e39，与派发提示一致）。
分到的行：`awk -F'\t' '$1 ~ /^F(7[3-9]|8[0-5])$/' candidates.tsv | wc -l` 得 193（F73=1、F74=40、F75=21、F76=4、F77=47、F79=29、F80=1、F82=31、F83=9、F84=9、F85=1）；F78、F81 是新立行，候选表里没有。

## 判定一览
- 要改 3 行：F79#3（05-fifth-txn.md:24）、F79#7（05-fifth-txn.md:111）、F80#1（CLAUDE.md:113）
- 要补 0 行；要人看 0 行
- 事件句不改 105 行；不相干 85 行；合计 193 行（`grep -c '^| F' ` 数下面的表得 193）
- 要补为 0 的依据见表后「反向问一遍」。

## 三处要改，要点
1. CLAUDE.md:113「崩溃放量的 8 项」：现查 `bash scripts/crash/crash-amplification.sh --list-items 2>/dev/null` 数输出行得 24；`wc -l .claude/gate.d/crash-point-replay-items.tsv` 得 44（基准 1bb3a24dc 时 25 行）。改成「崩溃放量的 24 项」并给数法（`--list-items` 输出行数）。
2. 05-fifth-txn.md:24（入·4 现查格）与 :111：里程碑四的整理接口已在 master。现查：`git worktree list` 与 `git branch -a` 都没有 m4-predev，目录 ~/.local/share/singlefs/work/m4-predev 不存在；`git cat-file -e HEAD:crates/singlefs-core/src/region_scan.rs`（同样 compaction_plan.rs、logged_operations.rs、crates/singlefs-replay-prefix/src/compaction_plan_proofs.rs）都成功；`git log --format=%h -- crates/singlefs-core/src/region_scan.rs` 只有 0b926bd3f（2026-10-21），基准 1bb3a24dc 里没有这个文件；表里八行的落点逐个在 HEAD 找得到，incompat 位 2 的常量 `INCOMPAT_INODE_RECORD_TRUNCATION_FIELDS_BIT = 0x04` 已写（表里「代码还没写」过时）。

## 逐行判定
| 组 | 载体 | 判定 | 改后的句子或理由 |
|---|---|---|---|
| F73 | records/2026-10-19-文档普查报告核实与修复.md:33 | 事件句不改 | 「有继任的旧路径五处」：这是 2026-10-19 乙交回的复述，把 scripts/agents/oov-check.py 当作「有继任的旧路径」的反例，句子自己写明它「从没存在过」；换成 scripts/kb/oov-check.py 后变成「scripts/kb/oov-check.py 从没存在过、真身搬到 scripts/kb/」，不再是真话，所以不能按 F73 统一改。现查：ls scripts/agents/oov-check.py scripts/kb/oov-check.py 两个都报 No such file or directory；git log --all --oneline -- scripts/agents/oov-check.py 数行得 0 行（该路径从没有过提交）；git log --all --name-status -- scripts/kb/oov-check.py 显示 503b80e2a（2026-10-11）A、1b6f98657（2026-10-14）D。顺带：「真身 2026-10-11 搬到 scripts/kb/」在 10-19 写下时那份已在 10-14 删掉，这是乙当时的报法，不是这一阶段造成的；全仓 grep scripts/agents/oov-check 除 research/prompts 冻结证据外只剩这一处和记本批的 records/2026-10-21-doc门禁违规修复.md:118。 |
| F74 | .claude/agent-common.md:57 | 不相干 | 「重型测试就是 checker 档那一批」：说的是轻型三条要带 quick 声明、写法照 run-scope-guard.sh 文件头；新增的崩溃放量入口与驱动脚本逐行判都不改它，对文件头核过，分句仍真。 |
| F74 | .claude/agent-common.md:59 | 不相干 | 「跑编译出来的代码经内存包装」：这一条本身就是新口径：声明写在 capped.sh、run-with-memory-cap.sh 两层包装前面，分句仍真。 |
| F74 | .claude/agent-common.md:88 | 不相干 | 「范围声明闸（.claude/hooks/」：已是新口径：崩溃放量三份入口同样要声明、整轮门禁与 cargo test 整包整工作区都是 whole、驱动脚本读进去逐行判；对 run-scope-guard.sh 文件头逐项核过一致。 |
| F74 | .claude/agents/experiment-runner.md:31 | 不相干 | 「1b. scripts/experime」：讲装置单测与运行的声明片段、完整写法声明在前包装在后，与新口径一致；没有断言声明闸只管哪几类命令。 |
| F74 | .claude/agents/implementation-writer.md:34 | 不相干 | 「1c. 动到的测试二进制跑红了」：讲 rerun-failed-tests.py 打出的命令开头已带 target 声明、包装插在声明与 cargo 之间，与新口径一致。 |
| F74 | .claude/agents/implementation-writer.md:35 | 不相干 | 「1d. 交回前只跑前台跑得完的轻型」：讲交回前形式化轻型的 quick 声明写法，仍真，不涉及声明闸管辖范围的变化。 |
| F74 | .claude/agents/implementation-writer.md:40 | 不相干 | 「4. 变异证红按测试算」：讲交回前验证合成一条命令、各段各带自己的声明，与「驱动脚本里每条命令各自带声明」一致。 |
| F74 | .claude/agents/kb-scribe.md:32 | 不相干 | 「2. 决策变更史」：kb-scribe 规格改状态时跑 decision-items-sync 已带 SINGLEFS_RUN_SCOPE 声明，就是这件事实的新写法。 |
| F74 | .claude/agents/kb-scribe.md:33 | 不相干 | 「3. 分项翻状态」：分项翻状态后跑 decision-items-sync 已带 items 声明，是新写法，仍真。 |
| F74 | .claude/agents/kb-scribe.md:35 | 不相干 | 「新立分项、改索引行这类不翻状态的规格不走第 3 步」：新立分项的收尾命令已带 items 声明，是新写法，仍真。 |
| F74 | .claude/agents/three-way-attack.md:53 | 不相干 | 「报告文件、模型目录」：讲三方攻方腿在副本里编译、跑的写法，例子只写声明片段、完整拼法照共用约束，与新口径一致。 |
| F74 | .claude/kb/experiments/186-unknown-trees-accounting-dimension-rederived.en.md:26 | 不相干 | 「SINGLEFS_RUN_SCOPE='」：E186 页复核单测命令已带 target 声明，用法正确。 |
| F74 | .claude/kb/experiments/186-未知のツリー・記帳次元の再導出.ja.md:26 | 不相干 | 「SINGLEFS_RUN_SCOPE='」：E186 页日文译文的复核单测命令已带 target 声明，同中文原文。 |
| F74 | .claude/kb/experiments/186-不认识的树-记账维重推.md:26 | 不相干 | 「SINGLEFS_RUN_SCOPE='」：E186 页中文原文的复核单测命令已带 target 声明，用法正确。 |
| F74 | .claude/main-agent.md:29 | 不相干 | 「10. **改」：并行写回时由主 agent 在全部交回后跑一次 decision-items-sync，命令已带 items 声明，是新写法。 |
| F74 | .claude/main-agent.md:52 | 不相干 | 「1. 主 agent 跑 bash sc」：列的是哪些起验证命令要带声明（这一条、轻型三条、cargo test 点名的包与目标、gate-staged.sh），是举例不是穷举；仍真，与新增入口不冲突。 |
| F74 | .claude/main-agent.md:98 | 不相干 | 「暂存之后、提交之前跑门禁」：用户要求时跑 --full 的命令已带种类 whole 声明，与新口径一致。 |
| F74 | .claude/main-agent.md:100 | 不相干 | 「定案之后写回 kb」：写回员命令已带 items 声明，是新写法。 |
| F74 | .claude/rules/path-moves.md:33 | 不相干 | 「门禁 doc-decisions 的 k」：路径搬家表里重新生成决策分项清单的命令已带 items 声明，是新写法。 |
| F74 | .claude/rules/verification.md:59 | 不相干 | 「起任何验证（轻型三条」：括号里是「任何验证」的举例，崩溃放量入口属其中的重型；「子 agent 起 whole 一律拒」仍真，没有断言只管括号里这几类。 |
| F74 | .claude/skills/trilingual/SKILL.md:33 | 不相干 | 「扫仓的门禁只在提交时跑」：讲暂存后单看三语四格要带前缀与范围声明，已是新写法。 |
| F74 | .claude/skills/trilingual/SKILL.md:36 | 不相干 | 「SINGLEFS_RUN_SCOPE='」：三语四格命令已带 items 声明，写法正确。 |
| F74 | records/2026-10-13-代码欠账批.md:212 | 事件句不改 | 「2026-10-18 m3-b44-co」：说的是 2026-10-18 那次复跑：副本里给逐条 cargo 补了范围声明，是那一次发生的事。 |
| F74 | records/2026-10-13-复用键与全仓改名.md:690 | 事件句不改 | 「用户补三句：「跑脚本的时候 给agent」：用户补三句与 4-3l 钩子当时的口径（SINGLEFS_RUN_SCOPE=whole 一类），是 10-13 的经过，现行口径在 run-scope-guard.sh 文件头。 |
| F74 | records/2026-10-13-复用键与全仓改名.md:691 | 事件句不改 | 「用户再加一句」：4-3l 口径「再改」的经过，说的是那一天的决定，不是现状。 |
| F74 | records/2026-10-13-复用键与全仓改名.md:699 | 事件句不改 | 「去重收尾：E165 装置 e165-re」：去重收尾与 4-3k 拆两位派发的经过，范围声明钩子尾活是当时派给 B 的活。 |
| F74 | records/2026-10-13-复用键与全仓改名.md:710 | 事件句不改 | 「4-3k-B 工具员交回」：4-3k-B 工具员当次交回清单，列了当时加的声明，是交付记录。 |
| F74 | records/2026-10-13-复用键与全仓改名.md:716 | 事件句不改 | 两份技能的范围声明改法用 replace-batch.py 打上，那一次的操作。 |
| F74 | records/2026-10-13-复用键与全仓改名.md:721 | 事件句不改 | 「.claude/rules/verifi」：verification.md 加两条，是 4-3k 那一次的改动记录。 |
| F74 | records/2026-10-13-复用键与全仓改名.md:774 | 事件句不改 | 「用户停了第二台的 TTS」：第四轮 B 合入后验证那一天的情况，每样带范围声明是当时的做法。 |
| F74 | records/2026-10-13-里程碑四预开发.md:278 | 事件句不改 | 「看门狗（修复批四」：看门狗到点退出的记录，一次范围声明与命令点名数对不上被拒，是那一次的事。 |
| F74 | records/2026-10-13-里程碑四预开发.md:301 | 事件句不改 | 「看门狗（截断设计轮）到 60 分钟定时回报退出」：看门狗到点退出的记录，两条腿补声明重跑，是那一次的事。 |
| F74 | records/2026-10-17-全仓知识腐烂治理.md:75 | 事件句不改 | 批 6 交回里列的当批改动（轻型三条带范围声明、执行前拒绝清单补范围声明闸），是那一批做了什么。 |
| F74 | records/2026-10-17-全仓知识腐烂治理.md:82 | 事件句不改 | rot-defs-r1 那一轮打中的清单，「范围声明只补了轻型三条」是当时的判决，现状在 agent-common.md 与 run-scope-guard.sh。 |
| F74 | records/2026-10-17-全仓知识腐烂治理.md:100 | 事件句不改 | 「下轮开（tooling-writer」：工具缺口表里「下轮开」的排期清单，是 2026-10-18 的待办记录。 |
| F74 | records/2026-10-18-里程碑三收尾与近期欠账.md:98 | 事件句不改 | 「合入后验证起了」：那一次合入后验证「带批 3 四份补丁目录（带范围声明 whole）」，是 10-18 的操作。 |
| F74 | records/2026-10-18-里程碑三收尾与近期欠账.md:109 | 事件句不改 | 批 4 工具员交回，交 rot-defs-r2 判的清单，是那一轮的交办。 |
| F74 | records/2026-10-18-里程碑三收尾与近期欠账.md:530 | 事件句不改 | 「它跑那一格被重型闸拒」：那一次被重型闸拒后主 agent 带范围声明跑 doc-registries 四格，是 10-18 的操作。 |
| F74 | records/2026-10-21-doc门禁违规修复.md:110 | 事件句不改 | 「tooling-writer 乙交回」：乙交回 two-logs 的改动清单，lib_heavy_tests.py 的 command_under_two_logs 让包装起的命令过范围声明闸，是那一次的交付。 |
| F74 | records/2026-10-21-doc门禁违规修复.md:114 | 事件句不改 | 主 agent 失手的经过，「不在这一趟的范围声明里」说的是那一趟的声明范围。 |
| F75 | .claude/agents/experiment-designer.md:34 | 不相干 | 「3c. 「## 十四」：已是新写法：「## 十四、并行与挂钟」一节写每段跑法的预计挂钟与预计在用核数，与新事实一致。 |
| F75 | .claude/agents/experiment-designer.md:56 | 不相干 | 固定节名表里「## 十四、并行与挂钟」一行写五个标签行、由 preregistration-parallelism 格判，就是新事实本身。 |
| F75 | .claude/agents/experiment-runner.md:37 | 不相干 | 「2c. 装置照跑前登记「## 十四」：已是新写法：装置照「## 十四、并行与挂钟」一节实现，串行段当场往 SINGLEFS_SERIAL_DECLARATIONS 声明。 |
| F75 | .claude/agents/mutation-triage.md:31 | 不相干 | 「2. 先逐条做子串计数」：已是新说法：只把原文不是恰好一次的「那几条」打 ⚓、不跑，别的条照跑；与 mutation-sampling.md 第七类「按条判」一致。 |
| F75 | .claude/kb/checks-owed.en.md:43 | 不相干 | 「mutation table没存」：讲 code-source-discipline 的 mutation-tables 格按子串计数静态判锚点唯一命中，是门禁一侧的职责，与 mutation-triage 怎么处理锚点失效的行无关；该格文件头仍有 mutation-tables 一格。 |
| F75 | .claude/kb/checks-owed.en.md:666 | 事件句不改 | 「mutation table的锚点腐化没」：C327 行写的是 2026-09-16 那天给 mutation-tables 格加子串计数、修 4 条腐化锚点，是那一次做的事。 |
| F75 | .claude/kb/checks-owed.ja.md:43 | 不相干 | 「ミューテーション表没存」：讲 code-source-discipline 的 mutation-tables 格按子串计数静态判锚点，是门禁一侧的职责，与 mutation-triage 的处理方式无关（译文行，同中文原文）。 |
| F75 | .claude/kb/checks-owed.ja.md:666 | 事件句不改 | 「ミューテーション表的锚点腐化没有会红的检」：C327 行写的是 2026-09-16 那天加子串计数、修 4 条腐化锚点，是那一次做的事（译文行）。 |
| F75 | .claude/kb/checks-owed.md:43 | 不相干 | 「**已有（2026-09-03 对抗测试轮）**」：讲 mutation-tables 格按子串计数静态判锚点，是门禁一侧的职责，与 mutation-triage 的处理方式无关。 |
| F75 | .claude/kb/checks-owed.md:667 | 事件句不改 | 「变异表的锚点腐化没有会红的检查」：C327 行写的是 2026-09-16 那天给 mutation-tables 格加子串计数、修 4 条腐化锚点，是那一次做的事。 |
| F75 | .claude/kb/experiments/155-每次持久化的写量三种fsync形态与反事实上界.md:1 | 事件句不改 | 「E155 每次持久化的写量」：E155 页状态行里的「第四次跑第一段 2026-09-21」跑到 31/34 因锚点不再唯一命中而中止（第七类），是那一次跑发生的事。 |
| F75 | .claude/kb/experiments/155-每次持久化的写量三种fsync形态与反事实上界.md:122 | 事件句不改 | 「**第四次跑第一段」：E155 页执行员记的那一次：锚点从命中 1 次变 2 次，mutate.sh 整体中止，第 33、34 条没跑到，是那一次的事。 |
| F75 | .claude/rules/mutation-sampling.md:53 | 不相干 | 「第七类：变异压根没跑——锚点在源码里不再」：这是第七类的标题，名字本身与 mutation-triage 怎么处理无关；正文已写「按条判、不让整张表退出」，与新说法一致。 |
| F75 | .claude/rules/mutation-sampling.md:63 | 不相干 | 「门禁形态：不跑变异」：门禁形态那句说的是 mutation-tables 格不跑变异、只做子串计数，格的判法没变，与 mutation-triage 第 2 步的写法无关。 |
| F75 | records/2026-09-16-subagent拆分提案.md:1275 | 事件句不改 | 拆分提案里「拦截（工具实现员）」是那一次交付：核数告警共用判法与 preregistration-parallelism 新格，是做成了什么的记录。 |
| F75 | records/2026-09-27-同步远端冲突块.md:247 | 事件句不改 | 「1b. 跑单测」：同步远端冲突块里抄下的当时 experiment-runner 定义文本（e<号>-<简称> 的写法），冲突块记录不回改，现行写法在 experiment-runner.md 第 4 步。 |
| F75 | records/2026-09-27-同步远端冲突块.md:251 | 事件句不改 | 「4. 跑产物进 research/results/」：同上，冲突块里抄下的第 4 步旧文本，说的是当时的产物命名，不是现状。 |
| F75 | records/2026-10-13-复用键与全仓改名.md:681 | 事件句不改 | 4-3j 派出两位工具员的派发内容（含「规则第七类同改」），是 10-13 那一次的派工。 |
| F75 | records/2026-10-13-复用键与全仓改名.md:697 | 事件句不改 | 「4-3j 工具员交回」：4-3j 交回清单，「规则第七类与符号表…改成现状」是那一次做的改动。 |
| F75 | records/2026-10-19-里程碑四GPU垃圾回收开工.md:55 | 事件句不改 | 「第八位执行员交回」：第八位执行员的交回：「登记 12.10 与『并行与挂钟』一节」是那一次交回的内容。 |
| F75 | records/2026-10-19-里程碑四GPU垃圾回收开工.md:59 | 事件句不改 | 「工具实现员交回」：工具实现员的交回：核数告警接进三处、新格 preregistration-parallelism，是那一次的交付记录。 |
| F76 | .claude/main-agent.md:7 | 不相干 | 「主 agent 只调度和判断」：括号里「补丁只改 checker 档包时测试本次未跑」「挑包」已是新写法；「打全部补丁」说的是不带 --skip-apply 时脚本自己打，--skip-apply 先核、分两批的做法写在 main-agent.md 第 52 行，两处不冲突（merge-and-verify.sh 文件头同写「--skip-apply 先核补丁打过没有」）。 |
| F76 | .claude/main-agent.md:94 | 不相干 | 「一批实现员交回到齐」：这一行是「合入之前」的第一次合入：「打全部补丁（apply-writer-patch.py，核不过整批停）」对应文件头「每个补丁目录经 apply-writer-patch.py 打（退 2 就整批停）」；挑包说法已是新写法，--skip-apply 与 --peer 另在第 52 行。 |
| F76 | records/2026-09-28-agent体系用量时间与调度分析.md:159 | 事件句不改 | 「scripts/gate/merge-a」：拆分提案里 merge-and-verify.sh 当时的立项表（「经 apply-writer-patch.py 打全部补丁」），是 2026-09-28 的记录，后来脚本改成只跑改到的包。 |
| F76 | records/2026-10-12-里程碑三工具与治理批.md:68 | 事件句不改 | 批 2 工具甲交回里「五份交回后主 agent 建集成副本打全部补丁」是那一次的处置。 |
| F77 | .claude/kb/checks-owed.en.md:399 | 不相干 | 「长跑的项不按wall clock打进度」：说的是 pinned-red-examples-replay 格经 e165-pin-red-examples.py 跑、每 300 秒出一行进度；replay 格仍在门禁里（checker-tier-formal-verification.sh 文件头有 gate-cell: pinned-red-examples-replay），被取代的是 complete 格，本行没提（译文行）。 |
| F77 | .claude/kb/checks-owed.en.md:414 | 不相干 | 「内存排不上时形式化那一道最坏约 12 小」：讲整道「只排一次」与 pinned-red-examples-replay 格读 VERUS_PROOF_CHECK_QUEUE_REFUSED_FILE；现查 grep -c 该变量在 checker-tier-formal-verification.sh 里得 2，与 complete 换成 table 无关（译文行）。 |
| F77 | .claude/kb/checks-owed.en.md:415 | 不相干 | 「形式化那一道被信号停了的归类与出路文字没」：讲 pinned-red-examples-replay-killed-red 样本核「被信号停了」那句出路，replay 格没变，与核心部分取哪几项无关（译文行）。 |
| F77 | .claude/kb/checks-owed.en.md:750 | 不相干 | 「recovery replaypin t」：C627 已写新格名 pinned-red-examples-table 与 replay，就是新说法（译文行）。 |
| F77 | .claude/kb/checks-owed.ja.md:399 | 不相干 | 「长跑的项不按壁時計打进度」：说的是 pinned-red-examples-replay 格每 300 秒出一行进度，replay 格仍在，被取代的是 complete 格，本行没提（译文行）。 |
| F77 | .claude/kb/checks-owed.ja.md:414 | 不相干 | 「内存排不上时形式化那一道最坏约 12 小」：讲整道「只排一次」与 replay 格读传文件这一步，现查 VERUS_PROOF_CHECK_QUEUE_REFUSED_FILE 在阶段脚本里仍有，与钉表格更名无关（译文行）。 |
| F77 | .claude/kb/checks-owed.ja.md:415 | 不相干 | 「形式化那一道被信号停了的归类与出路文字没」：讲 replay 格的 killed-red 样本核汇总出路，replay 格没变（译文行）。 |
| F77 | .claude/kb/checks-owed.ja.md:750 | 不相干 | 「リカバリ再生固定表整行抄只落了装置那一半」：C627 已写新格名 pinned-red-examples-table 与 replay，就是新说法（译文行）。 |
| F77 | .claude/kb/checks-owed.md:396 | 不相干 | 「长跑的项不按挂钟打进度」：说的是 pinned-red-examples-replay 格每 300 秒出一行进度，replay 格仍在，被取代的是 complete 格，本行没提。 |
| F77 | .claude/kb/checks-owed.md:411 | 不相干 | 「内存排不上时形式化那一道最坏约 12 小」：讲整道「只排一次」与 replay 格读传文件这一步，现查 grep -c VERUS_PROOF_CHECK_QUEUE_REFUSED_FILE 阶段脚本得 2，与钉表格更名无关。 |
| F77 | .claude/kb/checks-owed.md:412 | 不相干 | 「形式化那一道被信号停了的归类与出路文字没」：讲 replay 格的 killed-red 样本核「被信号停了」那句出路，replay 格没变。 |
| F77 | .claude/kb/checks-owed.md:751 | 不相干 | 「恢复重放钉表整行抄只落了装置那一半」：C627 已写新格名 pinned-red-examples-table 与 replay，就是新说法。 |
| F77 | .claude/kb/milestone/03-third-txn.en.md:206 | 事件句不改 | 「Archive commit 31128」：里程碑三坑表第 24 行写的是归档提交 31128273 删红例账、complete 格「从此每次都红」（2026-10-10 起），是当时的事；末格已写「已关」（译文行）。 |
| F77 | .claude/kb/milestone/03-third-txn.ja.md:206 | 事件句不改 | 「アーカイブコミット 31128273 が」：坑表第 24 行写的是归档提交 31128273 删红例账、complete 格以降毎回赤となり（2026-10-10 以降），是当时的事，末格已「クローズ済み」（译文行）。 |
| F77 | .claude/kb/milestone/03-third-txn.md:206 | 事件句不改 | 「归档提交 31128273 删掉了 .c」：坑表第 24 行写的是归档提交 31128273 删红例账、complete 格「从此每次都红」（2026-10-10 起），是当时的事；末格已写「已关（2026-10-12 现查）」。 |
| F77 | .claude/rules/verification.md:58 | 不相干 | 「谁跑得了哪一档由 .claude/hoo」：讲 heavy-test-guard 认轻型的判法在 lib_heavy_tests.py 的 light_tier_command 以及 *_BREAK 变量不算轻型；现查该函数仍在（grep -c 'def light_tier_command' 得 1），与核心部分取哪几项无关。 |
| F77 | .claude/rules/verification.md:126 | 不相干 | 「形式化验证：恢复重放取前缀」：已是新说法：红例钉表两格是 pinned-red-examples-table（只判钉表自己）与 pinned-red-examples-replay（逐行重放）。 |
| F77 | records/2026-09-30-里程碑三开工第一轮.md:920 | 事件句不改 | 「形式化验证真起 Verus 验收」：2026-09-30 合入验证里四趟都退 1、「pinned-red-examples-complete 红在作废的…」是那几趟的结果。 |
| F77 | records/2026-09-30-里程碑三开工第一轮.md:925 | 事件句不改 | 「形式化验证第二趟验收全绿」：2026-09-30 自证与整道全现判/全复用的数（「pinned-red-examples-replay：现判 3、复用 357」），是那几趟的输出。 |
| F77 | records/2026-09-30-里程碑三开工第一轮.md:969 | 事件句不改 | 「合并实现工具 B 因 API 登录失效中途断」：合并实现工具 B 中途断、新派 tooling-writer 接着做的经过，「按工具 A 的 light_tier_command 形态做参数」是当时的派工。 |
| F77 | records/2026-09-30-里程碑三开工第一轮.md:974 | 事件句不改 | 「V、G 两份补丁在副本里带前缀跑」：派工具 B3 的经过：「看门狗按 light_tier_command 整棵不判」是当时的派工内容。 |
| F77 | records/2026-09-30-里程碑三开工第一轮.md:975 | 事件句不改 | 「G2 交回（implementation」：工具 B3 交回的清单与主 agent 重跑的结果，是那一次的交付。 |
| F77 | records/2026-09-30-里程碑三开工第一轮.md:985 | 事件句不改 | 「形式化轻型 checker-tier-f」：主工作区跑形式化轻型当天的输出（「pinned-red-examples-complete 绿」），是那一趟的结果。 |
| F77 | records/2026-09-30-里程碑三开工第一轮.md:1015 | 事件句不改 | 「实现 E7 交回」：E7 报形式化轻型红在 pinned-red-examples-complete、登记的账被归档提交删了，是 2026-09-30 的经过。 |
| F77 | records/2026-09-30-里程碑三开工第一轮.md:1033 | 事件句不改 | 「主工作区轻型形式化与变异」：那一趟主工作区轻型：形式化 rc 1「只红 pinned-red-examples-complete」，是当时的结果。 |
| F77 | records/2026-10-01-删除与并发事务.md:1488 | 事件句不改 | 2026-10-10 形式化轻型在主工作区也红，是当天的记录。 |
| F77 | records/2026-10-01-删除与并发事务.md:1538 | 事件句不改 | 第 60 批乙交回里「pinned-red-examples-complete 红是已知的那份账缺失」，是当时的处置。 |
| F77 | records/2026-10-01-对接上游测试脚本规范.md:240 | 事件句不改 | 「调查员交回（/tmp/claude-10」：调查员交回时列「进度间隔还缺」的几处，是 2026-10-01 的调查结果，「形式化验证 pinned-red-examples-replay 格（capture_output）」是当时缺口。 |
| F77 | records/2026-10-01-对接上游测试脚本规范.md:245 | 事件句不改 | E165 装置那一件交回，「打上之后钉表补全之前 replay 格会红」是当时对第二波的排期。 |
| F77 | records/2026-10-01-对接上游测试脚本规范.md:519 | 事件句不改 | 「上游三仓发版推远端」：当时现查「本仓 find … 现有 7 个」空目录，是那一天的数。 |
| F77 | records/2026-10-01-里程碑三形式化接入.md:149 | 事件句不改 | 「3-3 交回」：新门禁六格的建档交回，列了当时的六格名（含 complete），是那一次的交付。 |
| F77 | records/2026-10-01-里程碑三形式化接入.md:418 | 事件句不改 | 7-5b 交回：「钉表追加与 --check 同一套配对」「pinned-red-examples-replay-killed-red」是那一次接手重核的结果。 |
| F77 | records/2026-10-11-里程碑三待优化上下游协调.md:82 | 事件句不改 | invariants 补丁副本里形式化轻型的输出（complete 红、与补丁无关），是当时的结果。 |
| F77 | records/2026-10-11-里程碑三知识腐烂治理.md:129 | 事件句不改 | 「形式化钉表的 complete 格在 H」：2026-10-11 知识腐烂治理里「complete 格在 HEAD ce1457c3 上红」是那一天现查的结果。 |
| F77 | records/2026-10-13-代码欠账批.md:198 | 事件句不改 | 2026-10-18 形式化轻型输出原样（「3 格里 3 格绿…pinned-red-examples-complete」），是那一趟的结果。 |
| F77 | records/2026-10-13-代码欠账批.md:223 | 事件句不改 | 2026-10-18 形式化轻型输出原样（「3 格里 3 格绿」），是那一趟的结果。 |
| F77 | records/2026-10-13-复用键与全仓改名.md:74 | 事件句不改 | 「用户（原话）「这么说我也有问题 那么把影」：改名后验证：「pinned-red-examples-complete 格 316 行都钉着、绿」是那一次的结果。 |
| F77 | records/2026-10-13-里程碑四预开发.md:206 | 事件句不改 | 加宽参考实现交回时转述的「副本里 complete 两格红」，是第二位当时的报告。 |
| F77 | records/2026-10-13-里程碑四预开发.md:361 | 事件句不改 | 「实现员拷的仓副本不带 research/」：「副本里恒红」是截乙、截戊当时报的情况，已由换成 table 格解决，说的是当时。 |
| F77 | records/2026-10-18-审查报告核实与修复.md:61 | 事件句不改 | 2026-10-18 主工作区形式化轻型「pinned-red-examples-complete 红」，是那一趟的结果。 |
| F77 | records/2026-10-18-审查报告核实与修复.md:81 | 事件句不改 | 「新发现：形式化 pinned-red-e」：审查报告核实表里「新发现」一行，是当时的发现与处置（交用户）。 |
| F77 | records/2026-10-18-审查报告核实与修复.md:86 | 事件句不改 | 「用户：「形式化门禁里的 pinned-r」：用户 2026-10-18 的原话，说的是当时的问题，原话不改。 |
| F77 | records/2026-10-18-审查报告核实与修复.md:88 | 事件句不改 | 「已知一处：门禁 checker-tier」：2026-10-18 记的「已知一处」：complete 格调 check() 默认 glob 产物，是当时的现查，修法见下一条甲簇交回。 |
| F77 | records/2026-10-18-审查报告核实与修复.md:123 | 事件句不改 | 「甲簇交回（Opus」：甲簇交回：「complete 换成 table」是那一次做成的事（交付记录）。 |
| F77 | records/2026-10-18-审查报告核实与修复.md:124 | 事件句不改 | 「验证段待改再加」：验证段待改清单（④⑤⑥⑦）是 2026-10-18 的待办；现查 grep 现行载体（stage-inputs.tsv、commit-stages.md、checks-owed.md、verification.md）已无 pinned-red-examples-complete。 |
| F77 | records/2026-10-18-审查报告核实与修复.md:128 | 事件句不改 | 「主 agent 自己改」：10-18 的改动清单（C627 旧格名改成 pinned-red-examples-table 等），是那一次做了什么。 |
| F77 | records/2026-10-18-里程碑三收尾与近期欠账.md:85 | 事件句不改 | 「批 3 实例表清扫交回」：实现员交回时报「pinned-red-examples-complete 在副本里红（副本没有 research/）」，是当时的情况。 |
| F79 | .claude/kb/milestone/05-fifth-txn.md:15 | 不相干 | 「分两档：硬入口全部满足之前不开段 2」：说的是入口分硬入口、软入口两档的规则（硬入口满足之前不开段 2，软入口只限不改 crates 的几项），是里程碑五自己的设计，不受里程碑四合进 master 影响。 |
| F79 | .claude/kb/milestone/05-fifth-txn.md:17 | 不相干 | 是「硬入口」小节的标题，没有现状断言。 |
| F79 | .claude/kb/milestone/05-fifth-txn.md:24 | 要改 | 入·4 的现查格是现状句，删掉日期读起来仍是「未满足，只在 m4-predev 工作树里有」，此刻不是真话。现查：git worktree list 与 git branch -a 都没有 m4-predev，目录 ~/.local/share/singlefs/work/m4-predev 不存在；git cat-file -e HEAD:路径 对 region_scan.rs、compaction_plan.rs、compaction_plan_proofs.rs、logged_operations.rs 都成功（提交 0b926bd3f，2026-10-21）；git show HEAD:transaction.rs 数 fn publish_relocation_batch 得 1，mounted_session.rs 数 pub fn run_compaction 得 2；unit.rs 只有 UNIT_CLASS_DATA=1、INDEX_NODE=2、PACKED=3；logged_operations.rs 种类码 1 整理 2 截断；system_configuration.rs 有 INCOMPAT_INODE_RECORD_TRUNCATION_FIELDS_BIT = 0x04 且在 SUPPORTED_INCOMPAT_BITS 里。改后的现查格：满足（2026-10-21 现查，提交 0b926bd3f）：表里八行的落点在 master 的 HEAD 都在，其中 incompat 位 2 的「代码还没写」已过时（常量已写）；同表的入·2、入·3 现查格与「里程碑四交出的整理接口」表的小标题下那句、二·1 一行同样写着 m4-predev，要一并改（见报告候选表之外的发现）。 |
| F79 | .claude/kb/milestone/05-fifth-txn.md:29 | 不相干 | 是「软入口」小节的标题，没有现状断言。 |
| F79 | .claude/kb/milestone/05-fifth-txn.md:39 | 不相干 | 是「里程碑四交出的整理接口」小节的标题，没有现状断言；标题下的现查句另见本组 F79#3 的说明。 |
| F79 | .claude/kb/milestone/05-fifth-txn.md:74 | 不相干 | 「「入口」一节的硬入口逐条判」：段 0 的做法写「『里程碑四交出的整理接口』表在 master 上逐行现查」是要做的事，现查还没由段 0 正式做，这一格「未开」仍真；软入口几项可在里程碑四收尾的等待里先做，不受影响。 |
| F79 | .claude/kb/milestone/05-fifth-txn.md:111 | 要改 | 「整理器与意图机制在里程碑四落地：2026-10-17 只在 m4-predev 工作树里有」删日期读来是现状句，此刻不真：整理器的落点已在 master（提交 0b926bd3f，现查命令与输出见 F79#3）。改后：整理器与意图机制在里程碑四落地，已合进 master（提交 0b926bd3f），落点见「里程碑四交出的整理接口」表；开工前在 master 上再查一次。 |
| F79 | .claude/kb/milestone/06-sixth-txn.ja.md:67 | 不相干 | 「**リポジトリ内の既存要素**: E15」：里程碑六页写的是里程碑八候选出口的内容，指向 08-eighth-txn.md「出口（未定）」；该节标题现仍是「出口（未定）」，与里程碑五出口细化无关（日文译文行）。 |
| F79 | .claude/kb/milestone/06-sixth-txn.md:68 | 不相干 | 「**仓里已有的**」：里程碑六页写的是里程碑八候选出口的内容，指向 08-eighth-txn.md「出口（未定）」；该节现仍是未定，与里程碑五出口细化无关。 |
| F79 | .claude/kb/milestone/07-seventh-txn.md:11 | 不相干 | 是里程碑七自己的「出口（未定）」小节标题，不是里程碑五的出口，没有被这批改动。 |
| F79 | .claude/kb/milestone/07-seventh-txn.md:15 | 不相干 | 是里程碑七候选出口表的表头，不是里程碑五的出口表。 |
| F79 | .claude/kb/milestone/08-eighth-txn.md:15 | 不相干 | 是里程碑八自己的「出口（未定）」小节标题，不是里程碑五的出口。 |
| F79 | .claude/kb/milestone/08-eighth-txn.md:19 | 不相干 | 是里程碑八候选出口表的表头，不是里程碑五的出口表。 |
| F79 | .claude/kb/milestone/09-ninth-txn.md:13 | 不相干 | 是里程碑九自己的「出口（未定）」小节标题，不是里程碑五的出口。 |
| F79 | .claude/kb/milestone/09-ninth-txn.md:17 | 不相干 | 是里程碑九候选出口表的表头，不是里程碑五的出口表。 |
| F79 | .claude/kb/milestone/11-eleventh-txn.md:11 | 不相干 | 是里程碑十一自己的「出口（未定）」小节标题，不是里程碑五的出口。 |
| F79 | .claude/kb/milestone/11-eleventh-txn.md:15 | 不相干 | 是里程碑十一候选出口表的表头，不是里程碑五的出口表。 |
| F79 | .claude/kb/milestone/12-twelfth-txn.md:23 | 不相干 | 是里程碑十二候选出口表的表头，不是里程碑五的出口表。 |
| F79 | .claude/kb/milestone/13-thirteenth-txn.md:11 | 不相干 | 是里程碑十三自己的「出口（未定）」小节标题，不是里程碑五的出口。 |
| F79 | .claude/kb/milestone/13-thirteenth-txn.md:15 | 不相干 | 是里程碑十三候选出口表的表头，不是里程碑五的出口表。 |
| F79 | .claude/kb/milestone/14-fourteenth-txn.md:11 | 不相干 | 是里程碑十四自己的「出口（未定）」小节标题，不是里程碑五的出口。 |
| F79 | .claude/kb/milestone/14-fourteenth-txn.md:15 | 不相干 | 是里程碑十四候选出口表的表头，不是里程碑五的出口表。 |
| F79 | .claude/kb/milestone/15-fifteenth-txn.md:11 | 不相干 | 是里程碑十五自己的「出口（未定）」小节标题，不是里程碑五的出口。 |
| F79 | .claude/kb/milestone/15-fifteenth-txn.md:15 | 不相干 | 是里程碑十五候选出口表的表头，不是里程碑五的出口表。 |
| F79 | .claude/kb/milestone/16-sixteenth-txn.md:11 | 不相干 | 是里程碑十六自己的「出口（未定）」小节标题，不是里程碑五的出口。 |
| F79 | .claude/kb/milestone/16-sixteenth-txn.md:15 | 不相干 | 是里程碑十六候选出口表的表头，不是里程碑五的出口表。 |
| F79 | .claude/kb/milestone/17-seventeenth-txn.md:11 | 不相干 | 是里程碑十七自己的「出口（未定）」小节标题，不是里程碑五的出口。 |
| F79 | .claude/kb/milestone/17-seventeenth-txn.md:15 | 不相干 | 是里程碑十七候选出口表的表头，不是里程碑五的出口表。 |
| F79 | records/2026-10-13-里程碑四预开发.md:325 | 事件句不改 | 「用户 2026-10-17」：2026-10-17 用户定长期老化挪进里程碑九、里程碑五入口出口细化「一并改在这棵工作树里」「先误改了主仓那份」，是那一天的经过。 |
| F80 | CLAUDE.md:113 | 要改 | 「加崩溃放量的 8 项」：「崩溃放量的 8 项」是现状句（数法没给），此刻不对：现查 bash scripts/crash/crash-amplification.sh --list-items 2>/dev/null 数输出行得 24，每行一个分项名（全量，不在快档）；崩溃点重放登记表 .claude/gate.d/crash-point-replay-items.tsv 现 44 行（基准 1bb3a24dc 时 25 行）。改后：崩溃点重放…的负载是 stage-inputs.tsv 登记的崩溃枚举用例（grep -c '^crash-case:' 数，含那两条流）加崩溃放量的 24 项（数它的命令：bash scripts/crash/crash-amplification.sh --list-items 数输出行）。 |
| F82 | .claude/kb/checks-owed.en.md:400 | 不相干 | 「变异self-test把包装的排队时间算」：讲 mutate.sh 限时交包装、包装的 252（排不上）与 254（被总上限挤掉）各自归类并让整轮判失败；252 的含义没变，新增的是包装排队时现查第二台并写告警记录，不改 mutate.sh 一侧的分类（译文行）。 |
| F82 | .claude/kb/checks-owed.en.md:414 | 不相干 | 「内存排不上时形式化那一道最坏约 12 小」：讲形式化一道「等满 3600 秒再退 252」与「只排一次」；退 252 的条件没变，新增的第二台现查与告警记录不改整道最坏等几次（译文行）。 |
| F82 | .claude/kb/checks-owed.ja.md:400 | 不相干 | 「变异自己検証把包装的排队时间算进 2 秒」：讲 mutate.sh 限时交包装、252 与 254 各自分类并让整轮判失败；252 的含义没变，新增的第二台现查不改 mutate.sh 一侧的分类（译文行）。 |
| F82 | .claude/kb/checks-owed.ja.md:414 | 不相干 | 「内存排不上时形式化那一道最坏约 12 小」：讲形式化一道「等满 3600 秒再退 252」与「只排一次」；退 252 的条件没变，新增的第二台现查不改整道最坏等几次（译文行）。 |
| F82 | .claude/kb/checks-owed.md:397 | 不相干 | 「变异自证把包装的排队时间算进 2 秒限时」：讲 mutate.sh 限时交包装、252 与 254 各自分类并让整轮判失败；252 的含义没变，新增的第二台现查不改 mutate.sh 一侧的分类。 |
| F82 | .claude/kb/checks-owed.md:411 | 不相干 | 「内存排不上时形式化那一道最坏约 12 小」：讲形式化一道「等满 3600 秒再退 252」与「只排一次」；退 252 的条件没变，新增的第二台现查与告警记录不改整道最坏等几次。 |
| F82 | .claude/rules/implementation-workflow.md:66 | 不相干 | 「**双机**」：讲崩溃枚举用例双机分片，第二台由双机驱动经 multi-host-run.sh --peer 起、内存上限照 PEER_MEMORY_CAP；与本机内存包装排不上时改用 --peer 的提示是两回事，仍真。 |
| F82 | records/2026-09-16-subagent拆分提案.md:941 | 事件句不改 | 「派发重活之前不判内存量」：拆分提案 2026-09-16 那一轮的欠账（「mutate.sh 每条变异外面套 timeout，排队的时间会算进 120 秒」）是当时的欠账，后来由 C612 把限时交给包装。 |
| F82 | records/2026-09-16-subagent拆分提案.md:949 | 事件句不改 | 「scripts/hosts/run-wi」：拆分提案里那一轮的自证与欠账记录（「包装被 KILL 时排队那一步照样成孤儿」），是当时的情况。 |
| F82 | records/2026-09-16-subagent拆分提案.md:1292 | 事件句不改 | 「欠：merge-and-verify.s」：拆分提案第七十五节写的欠账（merge-and-verify.sh 加交给 --peer 的选项）是当时的状态；后面几条记了派 tooling-writer 做成，现查 merge-and-verify.sh 文件头有 --peer 一段。 |
| F82 | records/2026-09-16-subagent拆分提案.md:1294 | 事件句不改 | 「卡在哪（主 agent 现查）」是那一天现查的占用数（排队那一刻回收不掉的用量 5.4 GiB 等），是当时的事。 |
| F82 | records/2026-09-16-subagent拆分提案.md:1295 | 事件句不改 | 「去向：派工具实现员」：「去向」是当时对工具实现员的派工内容，做成的结果记在下一条。 |
| F82 | records/2026-09-16-subagent拆分提案.md:1296 | 事件句不改 | 「去向（tooling-writer）」记的是那次派工与交付内容（run-with-memory-cap.sh 排队时现查第二台、merge-and-verify.sh 加 --peer），是做成了什么的记录。 |
| F82 | records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md:54 | 事件句不改 | 「复用验证过：同库同代码重跑一个状态都没再核」：2026-09-28 证红时第一次退 252 是因给了 --memory 40G，slice 总上限 40.1 GiB 排不进，是那一次的经过。 |
| F82 | records/2026-09-30-里程碑三开工第一轮.md:392 | 事件句不改 | I-8.5 合入验证第一趟排队 40 分钟超时退 252，是 2026-09-30 的经过。 |
| F82 | records/2026-09-30-里程碑三开工第一轮.md:414 | 事件句不改 | 「用户弹窗定 m3-broad-gate-」：用户弹窗定判决第四节三条（升档放不下就等或 252），是当时的决定。 |
| F82 | records/2026-09-30-里程碑三开工第一轮.md:726 | 事件句不改 | 资源限制清单交回：管内存的只有 run-with-memory-cap.sh 一处、等满退 252 等，是 2026-09-30 的盘点。 |
| F82 | records/2026-10-01-里程碑三形式化接入.md:414 | 事件句不改 | 「7-6 交回」：C631 当时的现查：一次等待上限 3600 秒、等满退 252、最坏 16 轮，是 2026-10-01 的记录。 |
| F82 | records/2026-10-01-里程碑三形式化接入.md:950 | 事件句不改 | b24 攻方复跑里 harness 那一段内存排队退 252，是那一次的经过。 |
| F82 | records/2026-10-13-代码欠账批.md:204 | 事件句不改 | 「2026-10-18 崩溃轻型单机那一趟」：2026-10-18 崩溃轻型排队 3600 秒没排上、退 252，是那一次的经过。 |
| F82 | records/2026-10-13-里程碑四预开发.md:780 | 事件句不改 | 「GPU 垃圾回收会话应这边的请求停了它第」：GPU 垃圾回收会话停第二台变异轻型腾额度，是 2026-10-13 的操作。 |
| F82 | records/2026-10-18-里程碑三收尾与近期欠账.md:230 | 事件句不改 | E180 读法长跑 9 片没做完（本机退 252 排不上）的经过，是 2026-10-18 的记录。 |
| F82 | records/2026-10-18-里程碑三收尾与近期欠账.md:360 | 事件句不改 | 「E71 第一次重跑执行员停下交回」：E71 第一次重跑执行员单测经 8G 包装排队退 252，是那一次的经过。 |
| F82 | records/2026-10-18-里程碑三收尾与近期欠账.md:361 | 事件句不改 | 「E97 第五次重跑执行员同样停在单测之前交回」：E97 第五次重跑 8G 排队退 252（账上 36 / 40.1 GiB），是那一次的经过。 |
| F82 | records/2026-10-18-里程碑三收尾与近期欠账.md:362 | 事件句不改 | C656 第二次抽样模型复跑 10G 排队退 252，是那一次的经过。 |
| F82 | records/2026-10-18-里程碑三收尾与近期欠账.md:366 | 事件句不改 | 「E186 第一段执行员停在单测之前交回」：E186 第一段执行员 8G 两次排队退 252，是那一次的经过。 |
| F82 | records/2026-10-18-里程碑三收尾与近期欠账.md:394 | 事件句不改 | 「C693 W2 调查交回」：调查员用 2G 包装（8G 那次排队超时退 252），是那次调查的经过。 |
| F82 | records/2026-10-19-里程碑四GPU垃圾回收开工.md:28 | 事件句不改 | 第三位执行员交回，「撞顶或排不上就改到第二台」是当时的打算。 |
| F82 | records/2026-10-19-里程碑四GPU垃圾回收开工.md:76 | 事件句不改 | 合入后验证收尾 cargo test 排队 3600 秒退 252、没编没跑，是 2026-10-19 的经过。 |
| F82 | records/2026-10-19-里程碑四GPU垃圾回收开工.md:95 | 事件句不改 | 「第二台的变异轻型第二趟也停掉」：第二台变异轻型第二趟停掉让给崩溃轻型，「排不上就退 252」是当时的考虑。 |
| F82 | records/2026-10-21-doc门禁违规修复.md:114 | 事件句不改 | 主 agent 失手那一趟：看门狗报内存排队后逐个停掉进程，是 2026-10-21 的经过。 |
| F83 | records/2026-09-16-subagent拆分提案.md:939 | 事件句不改 | 「变异跑道与实验进程没有内存上限」：拆分提案里「门禁 59 号收住了工作进程数（2026-09-25）」四项最小值是当时改成什么，磁盘项是后来换进去的，把旧值换成新值这句就不是那一天的真话了。 |
| F83 | records/2026-09-16-subagent拆分提案.md:1354 | 事件句不改 | 「磁盘余量成了工作进程数的第四项」是拆分提案当时记的改动，上限默认 64G 的说明写的是那一次的口径。 |
| F83 | records/2026-10-11-崩溃放量两段与脚本审计.md:83 | 事件句不改 | 2026-10-12 的现象记录（n 的 s2 后半程叠加 16 个工作进程）和主 agent 当时的判断，是那一天的事。 |
| F83 | records/2026-10-12-里程碑三工具与治理批.md:39 | 事件句不改 | 「用户指出变异门禁只开 16 个工作进程不对」：2026-10-12 用户指出只开 16 个工作进程，现查 worker_count_for 取 min(16, 核数 // 2)，是当时的现查。 |
| F83 | records/2026-10-19-里程碑四GPU垃圾回收开工.md:105 | 事件句不改 | 第二台变异轻型第三趟开出 9 个工作进程磁盘写满的经过，是 2026-10-19 的事。 |
| F83 | records/2026-10-19-里程碑四GPU垃圾回收开工.md:142 | 事件句不改 | 变异轻型第二台跑带 GATE_MUTATION_WORKERS=2（盘只剩 146G），是那一次的命令。 |
| F83 | records/2026-10-19-里程碑四GPU垃圾回收开工.md:152 | 事件句不改 | 「工具缺陷，进下一批」是 2026-10-19 的当日诊断，写的是那一天的缺陷与改法选项。 |
| F83 | records/2026-10-19-里程碑四GPU垃圾回收开工.md:157 | 事件句不改 | 「清盘（第二台」：清盘记录（第二台 246G 到 462G）是那一天的操作，补的一句也是对上一条缺陷的补充。 |
| F83 | records/2026-10-19-里程碑四GPU垃圾回收开工.md:162 | 事件句不改 | 「用户叫停：「新停在这里 不再派新任务了」」：用户叫停的原话和当时还开着的清单，是 2026-10-19 的状态。 |
| F84 | .claude/agents/experiment-runner.md:32 | 不相干 | 已是新写法：入库装置那一支的编译与单测归主 agent，执行员不对它起 cargo test，要主 agent 跑的命令写进报告。 |
| F84 | .claude/agents/experiment-runner.md:37 | 不相干 | 「2c. 装置照跑前登记「## 十四」：已是新写法：串行的几步跑到时由装置往 SINGLEFS_SERIAL_DECLARATIONS 声明，看门狗读它。 |
| F84 | .claude/agents/experiment-runner.md:38 | 不相干 | 「3. 入库装置那一支不写 researc」：入库装置那一支不写变异表、不跑 mutate.sh，research 那一支在 scripts/experiment-apparatus 下跑整张表，与新事实无冲突，仍真。 |
| F84 | .claude/agents/experiment-runner.md:40 | 不相干 | 「4. 跑产物进 research/results/」：已是新写法：新实验没有旧名时按 e<号>-<英文名里的下划线换成连字符>.out 算，入库装置用主 agent 编好的那一份。 |
| F84 | records/2026-09-16-subagent拆分提案.md:1275 | 事件句不改 | 拆分提案里「拦截（工具实现员）」是那一次交付的记录，运行时核数告警与串行声明是做成了什么。 |
| F84 | records/2026-09-27-同步远端冲突块.md:234 | 事件句不改 | 「3. 入库装置那一支不写 researc」：同步远端冲突块里抄下的当时 experiment-runner 定义第 3 步，冲突块记录不回改，现行写法在 experiment-runner.md 第 3 步。 |
| F84 | records/2026-09-27-同步远端冲突块.md:236 | 事件句不改 | 「4. 跑产物进 research/results/」：同步远端冲突块里抄下的当时第 4 步，说的是当时的产物与编译目录写法，不是现状。 |
| F84 | records/2026-10-13-里程碑四预开发.md:818 | 事件句不改 | 「看门狗报集成副本上 GPU 对拍 cra」：看门狗报低核 3.8 核、「工具欠账：GPU 等卡段没有写串行声明」是 2026-10-13 的现象与当日欠账。 |
| F84 | records/2026-10-19-里程碑四GPU垃圾回收开工.md:59 | 事件句不改 | 工具实现员交回：核数告警接进三处、串行段经 SINGLEFS_SERIAL_DECLARATIONS 声明，是那一次的交付记录。 |
| F85 | .claude/agents/three-way-attack.md:36 | 不相干 | 「在自己的原型里这样调枚举函数跑小流不算重型」：已是新写法：调枚举函数的原型只 cargo build，经内存包装直接执行编出来的二进制，不 cargo run、不 cargo test，与新事实一致。 |

## 候选表之外的发现（同一件事实，检索词没搜到的句子）
这些不在候选表里，不进上面的表；只判不改，交主 agent 或写回员定。
1. `.claude/kb/milestone/05-fifth-txn.md`（及 `.en.md`、`.ja.md` 同一批行：入·2、入·3、入·4、二·1、整理接口表小标题下那句，英文版行号 22-24、37、41，日文版同）：与 F79#3、F79#7 同根，仍写着里程碑四的代码在 m4-predev 工作树。
   - 入·3（:23）「未满足：m4-predev 工作树 `git status --short` 285 条，全部未提交」：m4-predev 工作树与分支已不存在（`git worktree list`、`git branch -a` 现查），里程碑四核心层已在 master 的提交 0b926bd3f 里；入·3 的判法写「`git log master` 里有里程碑四的提交」，此刻 `git log --oneline -- crates/singlefs-core/src/region_scan.rs` 有 0b926bd3f。
   - 入·2（:22）「未满足：代码在工作树 `~/.local/share/singlefs/work/m4-predev`（分支 `m4-predev`）；截断还差一批实现…」：「代码在工作树 m4-predev」过时；`crates/singlefs-core/src/truncate.rs` 已在 0b926bd3f 里，「截断还差一批实现」「验证段没跑完」「崩溃流全域没跑」我没逐项核，要人看；`ls .claude/kb/layout` 此刻仍只有 01-03，「没有 04- 开头的那份」仍真。
   - :41「2026-10-17 在 m4-predev 工作树里现查（未合入 master，不算数）」与二·1（:37）「整理接口按 m4-predev 工作树谈…合入之后对着 master 复核一次」：前者是带日期的现查句、后者是条件句，合入已发生，二·1 的复核条件已满足。
2. `.claude/kb/milestone/04-fourth-txn.md:131`（及译文 :130）「实现进度：写段在 m4-predev 工作树里做」：m4-predev 已不存在，同上；这一句是现状还是当时的记录，我没核里程碑四的整体进度，要人看。
3. `.claude/kb/decisions.md:33`、`:82`（及译文）：示例命令 `bash .claude/gate.d/doc-decisions.sh --write` 与 `--check decision-items-sync` 没带 `SINGLEFS_RUN_SCOPE` 声明；run-scope-guard.sh 文件头把门禁阶段不带 `--check`/`--item` 的跑法算 whole，`--check` 点名一格算 items，都要声明（kb-scribe.md、main-agent.md、path-moves.md 已带）。这是示例而不是断言，要不要补声明交主 agent 判。

## 反向问一遍（第 10 步）
本组做成的事在该记的载体里都有一处记着，所以不加 M 行：
- F76 合入后验证（只给自己那一批、`--skip-apply` 先核、`--peer`）：`.claude/main-agent.md:52`，脚本文件头 `scripts/gate/merge-and-verify.sh`。
- F77 核心部分取什么：`.claude/rules/verification.md:52`；钉表格取代 complete 格：同文件 :126 与 `.claude/gate.d/checker-tier-formal-verification.sh`（`grep -c pinned-red-examples-table` 得 9，`pinned-red-examples-complete` 得 0）。
- F78 看门狗长跑在用核数过少：`.claude/main-agent.md:68`、`scripts/hosts/core_usage.py`、`scripts/agents/agent-watch.py`、`.claude/agents/experiment-runner.md`。
- F81 十三个新门禁格：每个格名都在对应门禁脚本里（现查：field-frequency-sizes、invariant-prerequisites 在 doc-registries.sh；account-quota、out-of-project-names 在 doc-text.sh；preregistration-parallelism 在 doc-experiments.sh；crash-flow-coverage、crash-ignored-nodes、crash-knobs、invariant-zero-evaluations、last-full-judgement、crash-case-must-red 在 checker-tier-crash-point-replay.sh；pinned-red-examples-table 在 checker-tier-formal-verification.sh；committed-file-size 在 code-tooling.sh），CLAUDE.md 与 README 里没有按门禁格数写的数，不用改数。
- F82 内存包装排不上改用 `--peer`：`scripts/hosts/run-with-memory-cap.sh`、`scripts/agents/agent-watch.py`（`memory-queue-peer-fits`）。
- F83 变异整表工作进程与编译目录上限：`.claude/gate.d/checker-tier-crates-mutation-replay.sh`、`scripts/mutation/crates-mutation-rows.py`（`GATE_MUTATION_TARGET_DIR_CAP`）。
- F84、F85：`.claude/agents/experiment-runner.md` 第 1b、2c、3、4 步，`.claude/agents/three-way-attack.md:36`。
- F79 里程碑五入口出口细化：`05-fifth-txn.md`，其中的现查格已过时见上。

## --check-report
命令：`python3 scripts/kb/stale-candidates.py --check-report /home/fy5090/.local/share/singlefs/work/batch-2026-10-21-sync-facts/candidates.tsv /home/fy5090/.local/share/singlefs/work/batch-2026-10-21-sync-judge-9/report.md --groups F73-F77,F79-F80,F82-F85`
末行：
  ✓ 报告判全了：193 行候选都有逐行判定

## 没做什么
- 没改仓里任何文件，没改 stale=；草稿（判定数据 j1-j4.tsv、生成脚本）在 /home/fy5090/.local/share/singlefs/work/batch-2026-10-21-sync-judge-9/，没建仓副本或编译目录。
- 行里的 4 字以上引文大半是自动取的行首片段（候选表里的原文最长 500 字，行尾看不到），理由正文是逐行写的；事件句不改的 105 行里，凡写「是那一次的经过」的，我读了该行上下文但没逐条回到会话记录核当时的事。
- 候选表没罩到的句子（上面「候选表之外的发现」）只列了我顺着 F79、F80、F74 看到的几处，没有对 F73-F85 之外的事实表行再搜。
- F73 的现查只证了路径不存在与历史（`ls`、`git log --all`）；乙当时报的「真身 2026-10-11 搬到 scripts/kb/」其实 10-14 已删（git log 看 1b6f98657 为 D），这是 10-19 记录当时的报法，没有改。
- 推翻条件：F79#3、F79#7 的「要改」会被推翻，如果 `git worktree list` 又出现 m4-predev 且里程碑四整理接口的落点只在那棵树里；F80#1 的「24」会被推翻，如果 `crash-amplification.sh --list-items` 的输出行数不再是 24。
