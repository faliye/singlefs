# 逐行判 S34：全仓知识腐烂治理 2026-10-17（批 4）

- 组：F910、F911、F914、F915、F916、F917，共 249 行（`awk -F'\t' '$1=="F910"||$1=="F911"||$1=="F914"||$1=="F915"||$1=="F916"||$1=="F917"' candidates.tsv | wc -l` = 249）。基准 ce1457c3，结束在工作区（读文件，没有 `git show`）。
- 没建副本，没起编译，没跑重型测试；只读仓与候选表，没改被判的文件。
- 现查的新事实：
  - `ls -d crates/*/` 数出 8 个目录（多出 `crates/singlefs-unit-graph`）；`CLAUDE.md:8`、`.claude/kb/verification-build.md:4` 仍写「七个 crate」。
  - `grep -c '^crash-case:' .claude/gate.d/stage-inputs.tsv` = 14；合表 `.claude/gate.d/crash-point-replay-items.tsv` 去掉注释行 14 项。
  - `crates/singlefs-checker/src/image.rs:112` `IMPLEMENTED_INVARIANTS: [&str; 92]`；`grep -ch '^| I-.*已实现' crates/singlefs-checker/invariants/0*.md | awk '{s+=$1} END{print s}'` = 92。
  - `scripts/agents/ask-local.sh`、`oov-check.py`、`corruption-check.py` 都不存在（`ls` 报无此文件）。
  - `scripts/agents/remind.sh` 在；`.claude/settings.json` 里 `timer-only-remind-guard.sh` 的 matcher 是 `CronCreate|ScheduleWakeup|RemoteTrigger|Skill|Monitor|Bash`，停止钩子 `remind-rearm-stop-guard.sh` 已注册。
  - `crates/singlefs-checker-tier/src/crash_point_replay_progress.rs` 头注：「进度文件与双机账本没有了：分片、导入、续跑都走库」。
- 候选表的行号是出表时的；判之前逐行用原文在文件里定位（候选表原文只截到 500 字，行内靠后的命中在表里看不到，我按文件里那一行判），249 行都在文件里找得到，没有「原文已不在」的。
- 判定里「批次快照」指带日期批次下收拢表、去向表、进度表的那一天的记载。

## 逐行判定

下面按组分六张表（每张表整块一段写入，表头相同；`--check-report` 把「## 逐行判定」标题下的全部表行合起来判）。同一载体行在几个组里命中的，本报告里每组各写一行、判定相同。

### F910

| 组 | 载体 | 判定 | 改后的句子或理由 |
|---|---|---|---|
| F910 | .claude/kb/checks-owed.md:369 | 要改 | 原句「断点续跑」已不真：崩溃枚举现在一律经判定库（`crates/singlefs-checker-tier/src/crash_state_enumeration.rs` 的 `enumerate_crash_states_with_every_kind_of_observer` 调 `enumerate_crash_states_through_the_library`），进度文件与双机账本已取消（`crash_point_replay_progress.rs` 只留发现日志与工具链身份），打断重跑只补库里没有的块，四条与别的用例同一条路径（按代码读的，推的，没跑）；另「已登记 13 条」现为 14 条（`grep -c ^crash-case: .claude/gate.d/stage-inputs.tsv` 现数 14）。改后：「门禁 checker-tier-crash-point-replay 的 `--full` 已登记 14 条崩溃枚举用例（`.claude/gate.d/stage-inputs.tsv` 的 `crash-case:` 行，`grep -c ^crash-case:` 现数）。仍欠两样：① 计数行：floor-raise-pushed-by-the-session 与 position-addressed-tree-streams 没登记 `exhaustive=`，门禁判不出它们枚举满了没有；② 判定：（同原句 ③ 不变）」；「怎么拦」一格里「四条补断点续跑（进度文件，判法在 `crates/singlefs-checker-tier/src/crash_point_replay_progress.rs`）；」整句去掉，只留「两条补计数行与 `exhaustive=`；全量跑到每条用例的崩溃状态块都有判定。」。同时 stage-inputs.tsv 里 tree-split-streams 一行的「不留进度文件（没有断点续跑」也是同一处过时，不在候选表里，见「要主 agent 现查」。 |
| F910 | .claude/kb/decisions/13-验证路线.md:261 | 不相干 | 引文「全量带断点续跑」说的是全量能续跑；现在续跑走判定库（`crash_state_enumeration`），进度文件没了、续跑还在，句子仍真，不受这件事实影响。 |
| F910 | .claude/kb/layout/01-first-txn.md:420 | 不相干 | 引文行内相关处说的是全量能续跑；现在续跑走判定库（`crash_state_enumeration`），进度文件没了、续跑还在，句子仍真，不受这件事实影响。（行首「⚠️ 覆盖写固定脚本流（里程碑「覆盖写、释放、回」；候选表原文只截到 500 字，命中处在行内靠后） |
| F910 | .claude/kb/milestone/02-second-txn.md:515 | 事件句不改 | 引文「断点续跑只记进度」带日期「2026-09-28 时」与「2026-09-27 现查」，是立这一项（崩溃放量结果怎么存）时的起点快照，说的是那天仓里有什么，不是今天；要改成今天的样子等于改写这一项的立项背景。 |
| F910 | .claude/kb/term-renames.md:106 | 不相干 | 这是改名登记表的一行：「`layer0_progress(?![A-Za-z0-9_])(?<![A-Za-」 作为旧名列在第一格，登记旧名正是它的用途，不是现状句。 |
| F910 | .claude/main-agent.md:78 | 不相干 | 引文「长活默认分片、能断点续跑、按配置里的机器自适应」是主 agent 对长活跑法的通则，没点名崩溃点重放的进度文件，不受该事实影响。 |
| F910 | .claude/rules/verification.md:13 | 不相干 | 这一行已按新事实写：「`crash_point_replay_progress`」与代码头注一致（`crash_point_replay_progress.rs` 只留发现日志与工具链身份），仍真。 |
| F910 | .claude/rules/verification.md:70 | 不相干 | 这一行只是模块名清单：「ash_state_enumeration、crash_point_replay_progress、crash_injection、writ」列的是 checker 档测试文件头要写的模块名，该模块还在，仍真。 |
| F910 | records/2026-09-16-subagent拆分提案.md:1158 | 不相干 | 标题「五十九、超过几小时的长活没有默认分片、断点续跑、按多机配置自适应」是 2026-10-06 用户对长活通则的批评，与崩溃点重放的进度文件无关。 |
| F910 | records/2026-09-24-里程碑二收尾调度.md:19 | 事件句不改 | 批次快照：「崩溃点重放覆盖写固定脚本流的断点续跑先实现」是 2026-09-24 收尾调度表里「先实现、才开跑」的排序，后来实现了；那一天的排序不改。 |
| F910 | records/2026-09-24-里程碑二收尾调度.md:178 | 事件句不改 | 批次快照：行内相关处记 2026-09-26 主 agent 当天现查「今天不能续跑」，是那一天的事。（行首「崩溃点重放规模与定义两轮提前并行（用户 20」；候选表原文只截到 500 字，命中处在行内靠后） |
| F910 | records/2026-09-24-里程碑二收尾调度.md:180 | 事件句不改 | 批次快照：「断点续跑照做、按攻方打中的四格收严」是 2026-09-26 判完那一轮的结论，说的是当时「照做」。 |
| F910 | records/2026-09-24-里程碑二收尾调度.md:195 | 事件句不改 | 批次快照：这是里程碑二收尾调度表里的交回记录，「lefs-checker-tier/src/crash_point_replay_progress.rs`」写的是那一次派发与交回时的事，不是今天的现状。 |
| F910 | records/2026-09-24-里程碑二收尾调度.md:202 | 事件句不改 | 批次快照：这是里程碑二收尾调度表里的交回记录，行内相关处写的是那一次派发与交回时的事，不是今天的现状。（行首「实八交回（2026-09-26）  接续的」；候选表原文只截到 500 字，命中处在行内靠后） |
| F910 | records/2026-09-24-里程碑二收尾调度.md:207 | 事件句不改 | 批次快照：这是里程碑二收尾调度表里的交回记录，「`crash_point_replay_progress.rs`、`crash.rs`」写的是那一次派发与交回时的事，不是今天的现状。 |
| F910 | records/2026-09-24-里程碑二收尾调度.md:236 | 事件句不改 | 批次快照：这是里程碑二收尾调度表里的交回记录，行内相关处写的是那一次派发与交回时的事，不是今天的现状。（行首「同步远端与五批提交；A4c、B3c-2、材料」；候选表原文只截到 500 字，命中处在行内靠后） |
| F910 | records/2026-09-24-里程碑二收尾调度.md:246 | 事件句不改 | 批次快照：这是里程碑二收尾调度表里的交回记录，「断点续跑与分片」写的是那一次派发与交回时的事，不是今天的现状。 |
| F910 | records/2026-09-24-里程碑二收尾调度.md:256 | 事件句不改 | 批次快照：这是里程碑二收尾调度表里的交回记录，「断点续跑与分片」写的是那一次派发与交回时的事，不是今天的现状。 |
| F910 | records/2026-09-24-里程碑二收尾调度.md:299 | 事件句不改 | 批次快照：这是里程碑二收尾调度表里的交回记录，「.rs`、`device_log.rs`、`crash_point_replay_progress.rs`、`on_device_modes」写的是那一次派发与交回时的事，不是今天的现状。 |
| F910 | records/2026-09-24-里程碑二收尾调度.md:301 | 事件句不改 | 批次快照：这是里程碑二收尾调度表里的交回记录，行内相关处写的是那一次派发与交回时的事，不是今天的现状。（行首「第二轮改法 A 交回：checker 四判、」；候选表原文只截到 500 字，命中处在行内靠后） |
| F910 | records/2026-09-27-代码审阅38条去向.md:13 | 事件句不改 | 批次快照：「`crash_point_replay_progress.rs`」是 2026-09-27 审阅去向表里「实分一在改」的当天状态。 |
| F910 | records/2026-09-27-验证两档拆分.md:20 | 事件句不改 | 「、断点续跑、崩溃注入、故障注入」是 2026-09-27 拆分提案里描述拆分之前 harness 包装着什么，问题陈述，不是今天的样子。 |
| F910 | records/2026-09-27-验证两档拆分.md:31 | 事件句不改 | 「oracle、断点续跑」是那一天拆分计划的第 5 步，计划已执行；记录里的计划句不随后来改。 |
| F910 | records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md:45 | 事件句不改 | 批次快照：「`crash_point_replay_progress.rs`」所在行是 2026-09-28 至 2026-09-29 调度表的当日记录，不是今天。 |
| F910 | records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md:76 | 事件句不改 | 批次快照：「门禁读违例与断点续跑不重不漏、三档入口逐项相同、双机各写一库导入与单机逐」所在行是 2026-09-28 至 2026-09-29 调度表的当日记录，不是今天。 |
| F910 | records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md:89 | 事件句不改 | 批次快照：「录入与核对各自断点续跑、一块不重核不漏核」所在行是 2026-09-28 至 2026-09-29 调度表的当日记录，不是今天。 |
| F910 | records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md:105 | 事件句不改 | 批次快照：行内相关处所在行是 2026-09-28 至 2026-09-29 调度表的当日记录，不是今天。（行首「回扫交回；KV 转来这一轮、第一段派出；门禁」；候选表原文只截到 500 字，命中处在行内靠后） |
| F910 | records/2026-09-28-里程碑三第十项清单.md:341 | 事件句不改 | 批次快照：「四条没有断点续跑」所在是 2026-09-28 第十项清单里的当日去向（「部分已还」「留着」），今天的欠账在 `.claude/kb/checks-owed.md` C564 那一行。 |
| F910 | records/2026-10-01-对接上游测试脚本规范.md:134 | 事件句不改 | 「崩溃点重放断点续跑有关掉再判的对照、在快档」是 2026-10-01 审计结论的记录，说的是当时的快档用例名，事件。 |
| F910 | records/2026-10-01-对接上游测试脚本规范.md:359 | 事件句不改 | 这是把决策 13 的射程原文抄进 2026-10-01 记录里（当时的门禁名 checker-tier-layer0-replay 也是旧名）；抄录是当时的快照，活的正文在 `.claude/kb/decisions/13-验证路线.md`。 |
| F910 | records/2026-10-01-对接上游测试脚本规范.md:363 | 事件句不改 | 这是把决策 13 的射程原文抄进 2026-10-01 记录里（当时的门禁名 checker-tier-layer0-replay 也是旧名）；抄录是当时的快照，活的正文在 `.claude/kb/decisions/13-验证路线.md`。 |
| F910 | records/2026-10-01-对接上游测试脚本规范.md:370 | 事件句不改 | 「崩溃态枚举引擎与记录核对器、断点续跑与双机分片、崩溃注入与坏盘输入两场战役、真设备一侧的」是 2026-10-01 记录里「定案」那一天的措辞，记的是那一天定了什么；现行写法在 `.claude/rules/verification.md`。 |
| F910 | records/2026-10-01-对接上游测试脚本规范.md:374 | 事件句不改 | 「崩溃态枚举引擎与记录核对器、断点续跑与双机分片、崩溃注入与坏盘输入两场战役、真设备一侧的」是 2026-10-01 记录里「定案」那一天的措辞，记的是那一天定了什么；现行写法在 `.claude/rules/verification.md`。 |
| F910 | records/2026-10-01-对接上游测试脚本规范.md:673 | 事件句不改 | 「`layer0_progress.rs`」是 2026-10-01 交回记录里的旧文件名，事件。 |
| F910 | records/2026-10-01-里程碑三形式化接入.md:1281 | 事件句不改 | 行内相关处是 2026-10-06 用户原话与当天治理欠账的记录，事件，且说的是长活通则不是进度文件。（行首「- 用户 2026-10-06：「长时间重型任务」；候选表原文只截到 500 字，命中处在行内靠后） |
| F910 | records/2026-10-11-崩溃放量两段与脚本审计.md:51 | 事件句不改 | 这一行就是这件事实的落地记录（「不再读进度文件与双机账本」），是 2026-10-11 做了什么，事件。（行首「- 2026-10-11：课题 3、4、5、6」；候选表原文只截到 500 字，命中处在行内靠后） |
| F910 | records/2026-10-11-崩溃放量两段与脚本审计.md:59 | 事件句不改 | 「秒——断点续跑实测」是 2026-10-12 全量 i 的实测记录，事件。 |
| F910 | records/2026-10-12-里程碑三工具与治理批.md:44 | 不相干 | 「小时多机分片、断点续跑、难活显式」是长活通则（分片、断点续跑、自适应多机）在私有 memory 与项目约定之间的对照，不涉及崩溃点重放的进度文件。 |
| F910 | records/2026-10-12-里程碑三工具与治理批.md:88 | 不相干 | 「长活默认分片、断点续跑、自适应多机」是长活通则（分片、断点续跑、自适应多机）在私有 memory 与项目约定之间的对照，不涉及崩溃点重放的进度文件。 |
| F910 | records/2026-10-12-里程碑三工具与治理批.md:152 | 事件句不改 | 「h_state_enumeration`、`layer0_progress`」是 2026-10-13 别的会话来信告知改名，事件；改名对照在 `.claude/kb/term-renames.md`。 |
| F910 | records/2026-10-13-代码欠账批.md:36 | 事件句不改 | 「h_state_enumeration`、`layer0_progress`」是 2026-10-13 别的会话来信告知改名，事件；改名对照在 `.claude/kb/term-renames.md`。 |
| F910 | records/2026-10-13-代码欠账批.md:59 | 事件句不改 | 「`crash_point_replay_progress.rs:337`」是 2026-10-15 回扫交回里抽核的一条注释问题，事件。 |
| F910 | records/2026-10-13-代码欠账批.md:104 | 事件句不改 | 「isk_input.rs、crash.rs、crash_point_replay_progress.rs、on_device_modes.r」是 2026-10-15 别的会话回信里划的文件范围，事件。 |
| F910 | records/2026-10-13-复用键与全仓改名.md:21 | 事件句不改 | 「h_state_enumeration`、`layer0_progress`」所在行是 2026-10-13 改名批的对照表，登记旧名到新名，事件。 |

### F911

| 组 | 载体 | 判定 | 改后的句子或理由 |
|---|---|---|---|
| F911 | .claude/kb/milestone/05-fifth-txn.md:69 | 不相干 | 这一行已把合表写进去：「`.claude/gate.d/crash-point-replay-items.tsv`」与 `.claude/gate.d/crash-point-replay-items.tsv` 现状一致，仍真。 |
| F911 | .claude/kb/milestone/11-eleventh-txn.md:57 | 不相干 | 这一行已把合表写进去：「`.claude/gate.d/crash-point-replay-items.tsv`」与 `.claude/gate.d/crash-point-replay-items.tsv` 现状一致，仍真。 |
| F911 | .claude/kb/translation-glossary.md:108 | 不相干 | 这是中日英术语对照的一行（原行「合表 merged table 統合表」）：登记的词仍在用，不是状态句。 |
| F911 | .claude/rules/verification.md:74 | 不相干 | 这一行已按新做法写：「再让崩溃点重放合表」与现状一致（合表一项在第二列点名 `crash-case:`，`crash-case-check.py` 的 placement 判），仍真。 |
| F911 | .claude/rules/verification.md:128 | 不相干 | 这一行已按新做法写：「的登记进崩溃点重放合表」与现状一致（合表一项在第二列点名 `crash-case:`，`crash-case-check.py` 的 placement 判），仍真。 |
| F911 | .claude/skills/crash-test/SKILL.md:34 | 不相干 | 这一行已按新做法写：「再让崩溃点重放合表」与现状一致（合表一项在第二列点名 `crash-case:`，`crash-case-check.py` 的 placement 判），仍真。 |
| F911 | records/2026-09-16-subagent拆分提案.md:923 | 不相干 | 「清单、场合表、谁跑哪部分」所在行是 2026-09-16 拆分提案里「重型测试只在提交时跑」一条的去向，与崩溃点重放合表无关。 |
| F911 | records/2026-09-30-里程碑三开工第一轮.md:973 | 事件句不改 | 「与合表」是 2026-10-10 交回记录，当时合表「13 项」；今天是 14 项（`grep -vc ^# .claude/gate.d/crash-point-replay-items.tsv` 得 14），但这是那天的数。 |
| F911 | records/2026-09-30-里程碑三开工第一轮.md:1007 | 事件句不改 | 「键取合表项名、计数行名跟着项名」是 2026-10-10 改名盘点的记录，当时的「13 条 crash-case」，事件。 |
| F911 | records/2026-10-11-scripts审计.md:51 | 事件句不改 | 「`.claude/gate.d/crash-point-replay-items.tsv`」是 2026-10-11 脚本审计第 21 条的发现，审计表是那一天的快照。 |
| F911 | records/2026-10-11-崩溃放量两段与脚本审计.md:25 | 事件句不改 | 「合表里同一条流只剩一套判法」是 2026-10-11 课题表的一格，事件。 |
| F911 | records/2026-10-11-崩溃放量两段与脚本审计.md:77 | 事件句不改 | 「已登记、合表已点名」是 2026-10-12 全量 n 的实测记录（「登记表里共 13 条」是当时的数），事件。 |
| F911 | records/2026-10-11-崩溃放量两段与脚本审计.md:92 | 事件句不改 | 「而核法要第一列是合表项名」是 2026-10-13 tooling-writer 交回里的阻断记录，事件。 |
| F911 | records/2026-10-11-里程碑三知识腐烂治理.md:24 | 不相干 | 这里的「合表」是事实表的合并（「合表、`--check-facts`」），不是崩溃点重放合表。 |
| F911 | records/2026-10-11-里程碑三知识腐烂治理.md:30 | 事件句不改 | 「合表」是 2026-10-11 主 agent 的现数，带日期的快照（`crash-case:` 13 条、合表 13 项都是那天的数）。 |
| F911 | records/2026-10-11-里程碑三知识腐烂治理.md:31 | 不相干 | 这里的「合表」是 `facts.tsv` 的合并（「合表」），与崩溃点重放合表无关。 |
| F911 | records/2026-10-12-里程碑三工具与治理批.md:36 | 事件句不改 | 「成员数、登记流、分项、合表能静态取」是 2026-10-12 覆盖表调查的输入清单，事件。 |
| F911 | records/2026-10-13-复用键与全仓改名.md:93 | 事件句不改 | 行内相关处是 2026-10-13 批 4-2 实现员交回里提到「要登 stage-inputs.tsv 一行与合表一项」，事件。（行首「- 批次 4-2 实现员乙交回（报告 `~/.l」；候选表原文只截到 500 字，命中处在行内靠后） |
| F911 | records/2026-10-13-复用键与全仓改名.md:98 | 事件句不改 | 「`.claude/gate.d/crash-point-replay-items.tsv`」记 2026-10-13 登记表怎么改的，其中「合表 14 项」就是当天的核；事件。 |
| F911 | records/2026-10-13-复用键与全仓改名.md:662 | 事件句不改 | 行内相关处是 2026-10-13 crates 第二轮交回，事件。（行首「crates 第二轮交回（`~/.local/s」；候选表原文只截到 500 字，命中处在行内靠后） |
| F911 | records/2026-10-13-复用键与全仓改名.md:846 | 事件句不改 | 「`.claude/gate.d/crash-point-replay-items.tsv`」是 2026-10-13 记的欠账现状（「只有四列」），欠账事项本身在 checks-owed；这一句说的是 C789 没立的那道闸，与合表是否存在无关，今天合表仍是四列。 |
| F911 | records/2026-10-17-全仓知识腐烂治理.md:35 | 不相干 | 这里的「合表」是事实表合并（「合表、`--check-facts`」），与崩溃点重放合表无关。 |
| F911 | records/2026-10-17-全仓知识腐烂治理.md:50 | 不相干 | 这里的「合表」是事实表合并（「合表时主」），与崩溃点重放合表无关。 |
| F911 | records/2026-10-17-全仓知识腐烂治理.md:53 | 不相干 | 这里的「合表」是事实表合并（「合表时删掉不出候选」），与崩溃点重放合表无关。 |
| F911 | scripts/crash/README.md:7 | 不相干 | 「崩溃点重放的入口与合表的读法、核法」所在的 scripts/crash/README.md 一行已经把入口与合表的读法、核法写在一起，与现状一致。 |

### F914

| 组 | 载体 | 判定 | 改后的句子或理由 |
|---|---|---|---|
| F914 | .claude/kb/checks-owed.md:250 | 不相干 | C342 一行里的「已实现」说的是它自己点名的那条检查或修法已落地（行内相关处），不是池级 checker 一共判几条；条数 92 的新事实不影响它。（行首「C342  树 ID 水位在根读不出时退回」；候选表原文只截到 500 字，命中处在行内靠后） |
| F914 | .claude/kb/checks-owed.md:364 | 不相干 | C554 一行里的「已实现」说的是它自己点名的那条检查或修法已落地（行内相关处），不是池级 checker 一共判几条；条数 92 的新事实不影响它。（行首「C554  崩溃恢复抛弃的根暂时读不出时影」；候选表原文只截到 500 字，命中处在行内靠后） |
| F914 | .claude/kb/checks-owed.md:462 | 不相干 | C695 一行里的「已实现」说的是它自己点名的那条检查或修法已落地（行内相关处），不是池级 checker 一共判几条；条数 92 的新事实不影响它。（行首「C695  树表 0 条那一版在单盘上读不」；候选表原文只截到 500 字，命中处在行内靠后） |
| F914 | .claude/kb/checks-owed.md:529 | 不相干 | C769 一行里的「已实现」说的是它自己点名的那条检查或修法已落地（行内相关处），不是池级 checker 一共判几条；条数 92 的新事实不影响它。（行首「C769  水位台阶行的草案 ③ 判不出历」；候选表原文只截到 500 字，命中处在行内靠后） |
| F914 | .claude/kb/checks-owed.md:558 | 不相干 | C840 一行里的「已实现」说的是它自己点名的那条检查或修法已落地（行内相关处），不是池级 checker 一共判几条；条数 92 的新事实不影响它。（行首「C840  系统配置节点大小字段没人判」；候选表原文只截到 500 字，命中处在行内靠后） |
| F914 | .claude/kb/checks-owed.md:559 | 不相干 | C841 一行里的「已实现」说的是它自己点名的那条检查或修法已落地（行内相关处），不是池级 checker 一共判几条；条数 92 的新事实不影响它。（行首「C841  坏镜像上容器号加一会 pani」；候选表原文只截到 500 字，命中处在行内靠后） |
| F914 | .claude/kb/checks-owed.md:715 | 不相干 | C47 一行里的「已实现」说的是它自己点名的那条检查或修法已落地（行内相关处），不是池级 checker 一共判几条；条数 92 的新事实不影响它。（行首「C47  根槽没按判定宽度对齐  202」；候选表原文只截到 500 字，命中处在行内靠后） |
| F914 | .claude/kb/decisions.md:50 | 不相干 | 这一格的「已实现 N 项」是决策分项的进度计数（「665 条），待下刷消息不能只从权威态重算的树」），不是池级 checker 的不变量条数，量不同。 |
| F914 | .claude/kb/decisions.md:64 | 不相干 | 这一格的「已实现 N 项」是决策分项的进度计数（「1 条共享脊柱上  [25-目标负载优先级.md](decisions/25-目标负载优先级.」），不是池级 checker 的不变量条数，量不同。 |
| F914 | .claude/kb/decisions/08-核心索引结构.md:16 | 不相干 | 「233 条定长 140 字节记录），内部节点码」是决策 8 这一分项自己的进度标记，不是 checker 不变量条数。 |
| F914 | .claude/kb/decisions/18-块里携带什么信息.md:320 | 不相干 | 行内相关处说的是该行点名的那几样（取号逐盘核、改法 C、I-9.30 等）已实现，不是总条数；92 条的新事实不影响。（行首「- **取号之前逐盘核带不带所选那一版**（主」；候选表原文只截到 500 字，命中处在行内靠后） |
| F914 | .claude/kb/decisions/18-块里携带什么信息.md:329 | 不相干 | 行内相关处说的是该行点名的那几样（取号逐盘核、改法 C、I-9.30 等）已实现，不是总条数；92 条的新事实不影响。（行首「**射程**：值域登记表管的是「码 → 类」与各」；候选表原文只截到 500 字，命中处在行内靠后） |
| F914 | .claude/kb/decisions/18-块里携带什么信息.md:580 | 不相干 | 行内相关处说的是该行点名的那几样（取号逐盘核、改法 C、I-9.30 等）已实现，不是总条数；92 条的新事实不影响。（行首「5. **谱系记录**：一棵专用谱系树，day-」；候选表原文只截到 500 字，命中处在行内靠后） |
| F914 | .claude/kb/experiments/154-两道闸串-重判与回收时点的代价.md:154 | 不相干 | 「已实现，但在本模型里是同一条代码路径**（见「这一段做了什么」第」说的是实验 154 模型里 H4a/H4b 两个场景已实现，与池级 checker 条数无关。 |
| F914 | .claude/kb/experiments/154-两道闸串-重判与回收时点的代价.md:174 | 不相干 | 「已实现，但在本模型里是同一条代码路径**（见「这一段做了什么」第」说的是实验 154 模型里 H4a/H4b 两个场景已实现，与池级 checker 条数无关。 |
| F914 | .claude/kb/experiments/154-两道闸串-重判与回收时点的代价.md:178 | 不相干 | 「这次的六项装置-实现对应检查只覆盖了已实现的抬 F/F_生效/区域归属/写行暖机四样」说的是 2026-09-17 实验当时 crates 里有没有两道闸等实现，与 checker 不变量条数无关。 |
| F914 | .claude/kb/milestone/02-second-txn.md:337 | 不相干 | 这一格里的行内相关处一类句子逐条说 I-3.11、I-1.8 等已实现，没有写总条数；总条数 92 不影响它。（行首「②  F 的「生效」事后回落（checke」；候选表原文只截到 500 字，命中处在行内靠后） |
| F914 | .claude/kb/milestone/02-second-txn.md:364 | 不相干 | 这一格说的是项列里 I-1.8、I-7.3、I-8.6、I-1.2、I-4.2 等逐条的实现状态（行内相关处），没有写池级总条数，不受 92 条影响。（行首「26  checker：认「非空持久有效根」；候选表原文只截到 500 字，命中处在行内靠后） |
| F914 | .claude/kb/milestone/02-second-txn.md:366 | 不相干 | 这一格说的是分裂的实现状态（「812 条、记账树 477 条、树表 81 棵」），不是 checker 条数。 |
| F914 | .claude/kb/milestone/03-third-txn.md:38 | 不相干 | 这一格已写「池级 checker 判 92 条」并给了数法，与 `IMPLEMENTED_INVARIANTS` 长度 92 一致；其余逐笔欠账状态与总条数无关。（行首「2 实现  **七·2 到 七·5（M1、」；候选表原文只截到 500 字，命中处在行内靠后） |
| F914 | .claude/kb/milestone/03-third-txn.md:61 | 不相干 | 「92 条不变量（`grep -ch '^ I-.*已实现'」与代码清单 `IMPLEMENTED_INVARIANTS` 长度 92、kb 状态列「已实现」行数 92 一致，仍真。 |
| F914 | .claude/kb/milestone/03-third-txn.md:78 | 不相干 | 这一格里「已实现」都是点名的 I-4.6、I-4.7 等逐条状态（行内相关处），没写池级总条数。（行首「- **仓里已有的**：概念层已定：D8（核心索」；候选表原文只截到 500 字，命中处在行内靠后） |
| F914 | .claude/kb/verification-build.md:138 | 不相干 | 「`IMPLEMENTED_INVARIANTS`」写 92 条并指向 README 的计数口径，与 `IMPLEMENTED_INVARIANTS` 长度 92 一致，仍真。 |
| F914 | CLAUDE.md:112 | 不相干 | 「92 条不变量（数它的命令：`grep -ch」写 92 条，与 `grep -ch` 数状态列「已实现」的行现数 92、`IMPLEMENTED_INVARIANTS` 长度 92 一致，仍真。 |
| F914 | records/2026-09-16-subagent拆分提案.md:1096 | 事件句不改 | 「`IMPLEMENTED_INVARIANTS`」是 2026-09-16 记录里记「往清单追加不变量那一批」引发的教训，事件。 |
| F914 | records/2026-09-16-subagent拆分提案.md:1097 | 事件句不改 | 「`IMPLEMENTED_INVARIANTS`」的那一批」是 2026-09-16 记录里记「往清单追加不变量那一批」引发的教训，事件。 |
| F914 | records/2026-09-19-里程碑二遗留收拢.md:7 | 事件句不改 | 「15 条（报告 e 的 N1–N15）。另外表里写过时的格子有」是 2026-09-19 收口表的合计与「已实现待代码三方」的说明，当天快照。 |
| F914 | records/2026-09-24-里程碑二收尾调度.md:177 | 事件句不改 | 「`IMPLEMENTED_INVARIANTS`」是 2026-09-26 实二交回里「46 → 47」的当日增量，事件。 |
| F914 | records/2026-09-24-里程碑二收尾调度.md:222 | 事件句不改 | 「27、36、37 条与环短于三条）、A2c（allocator」是 2026-09-27 恢复派发的当日记录，事件。 |
| F914 | records/2026-09-24-里程碑二收尾调度.md:301 | 事件句不改 | 「`IMPLEMENTED_INVARIANTS`」是 2026-09-27 改法 A 交回里「48 → 49」的当日增量，事件。 |
| F914 | records/2026-09-27-同步远端冲突块.md:1237 | 事件句不改 | 「46 条不变量（数它的命令：`grep -c」是 2026-09-27 同步远端冲突块里抄的 CLAUDE.md 当时的句子（46 条），冲突块是那天的快照；现行句在 `CLAUDE.md` 已写 92。 |
| F914 | records/2026-09-27-同步远端冲突块.md:1243 | 事件句不改 | 「46 条不变量（数它的命令：`grep -c」是 2026-09-27 同步远端冲突块里抄的 CLAUDE.md 当时的句子（46 条），冲突块是那天的快照；现行句在 `CLAUDE.md` 已写 92。 |
| F914 | records/2026-09-30-里程碑三开工第一轮.md:52 | 事件句不改 | 「49 条（主 agent 现查：工作区与 HEAD」是 2026-09-30 的调查交回与主 agent 当时现查（49 条），事件。 |
| F914 | records/2026-09-30-里程碑三开工第一轮.md:367 | 事件句不改 | 「8 条：第 4 批新立行翻已实现、I-2.4、I-8.1、I-8.5、I-4.1/I」是 2026-09-30 第 5 批、第 9 批写回交回时的当日计数，事件。 |
| F914 | records/2026-09-30-里程碑三开工第一轮.md:369 | 事件句不改 | 「`IMPLEMENTED_INVARIANTS`」是 2026-09-30 第 5 批、第 9 批写回交回时的当日计数，事件。 |
| F914 | records/2026-09-30-里程碑三开工第一轮.md:514 | 事件句不改 | 「翻已实现、E171 页）：报告与 pieces/」是 2026-09-30 第 5 批、第 9 批写回交回时的当日计数，事件。 |
| F914 | records/2026-09-30-里程碑三开工第一轮.md:535 | 事件句不改 | 「11 条过）：主 agent 读过 D16 三行的新串与」是 2026-09-30 各批交回的当日计数（68、69、84 条之类），事件。 |
| F914 | records/2026-09-30-里程碑三开工第一轮.md:539 | 事件句不改 | 「`IMPLEMENTED_INVARIANTS`」是 2026-09-30 各批交回的当日计数（68、69、84 条之类），事件。 |
| F914 | records/2026-09-30-里程碑三开工第一轮.md:554 | 事件句不改 | 「已实现 69 条、C697–C707」是 2026-09-30 各批交回的当日计数（68、69、84 条之类），事件。 |
| F914 | records/2026-09-30-里程碑三开工第一轮.md:704 | 事件句不改 | 「`IMPLEMENTED_INVARIANTS`」是 2026-09-30 各批交回的当日计数（68、69、84 条之类），事件。 |
| F914 | records/2026-10-01-README更新与知识腐烂治理.md:10 | 事件句不改 | 「I-.*已实现' crates/singlefs-checker/invariants/0*.md」是 2026-10-01 回扫表里「48 → 49」做完的记录，事件。 |
| F914 | records/2026-10-01-删除与并发事务.md:729 | 事件句不改 | 「`IMPLEMENTED_INVARIANTS`」是 2026-10-04 至 2026-10-06 各批交回时的当日计数或追加次序，事件。 |
| F914 | records/2026-10-01-删除与并发事务.md:788 | 事件句不改 | 「三格、`IMPLEMENTED_INVARIANTS`」是 2026-10-04 至 2026-10-06 各批交回时的当日计数或追加次序，事件。 |
| F914 | records/2026-10-01-删除与并发事务.md:789 | 事件句不改 | 「`IMPLEMENTED_INVARIANTS`」是 2026-10-04 至 2026-10-06 各批交回时的当日计数或追加次序，事件。 |
| F914 | records/2026-10-01-删除与并发事务.md:814 | 事件句不改 | 「23 条 `kb-spec-check.py`」是 2026-10-04 至 2026-10-06 各批交回时的当日计数或追加次序，事件。 |
| F914 | records/2026-10-01-删除与并发事务.md:924 | 事件句不改 | 行内相关处是 2026-10-04 至 2026-10-06 各批交回时的当日计数或追加次序，事件。（行首「- 2026-10-05 根环线报 checke」；候选表原文只截到 500 字，命中处在行内靠后） |
| F914 | records/2026-10-01-删除与并发事务.md:935 | 事件句不改 | 「`IMPLEMENTED_INVARIANTS`」是 2026-10-04 至 2026-10-06 各批交回时的当日计数或追加次序，事件。 |
| F914 | records/2026-10-01-删除与并发事务.md:955 | 事件句不改 | 「74 条」→ 83，I-9.17–I-9.24、I-9.29」是 2026-10-04 至 2026-10-06 各批交回时的当日计数或追加次序，事件。 |
| F914 | records/2026-10-01-删除与并发事务.md:1098 | 事件句不改 | 「`IMPLEMENTED_INVARIANTS`」是 2026-10-04 至 2026-10-06 各批交回时的当日计数或追加次序，事件。 |
| F914 | records/2026-10-01-删除与并发事务.md:1341 | 事件句不改 | 「已实现 90 条、在用 113 条」是 2026-10-09 第 50 批写回交回时的计数（已实现 90 条、在用 113 条），事件。 |
| F914 | records/2026-10-01-删除与并发事务.md:1424 | 事件句不改 | 「90、`IMPLEMENTED_INVARIANTS`」是 2026-10-10 事实表交回的现数（kb 90、清单 92、README 还写 49），当日快照，现已同为 92。 |
| F914 | records/2026-10-01-里程碑三形式化接入.md:698 | 事件句不改 | 「`IMPLEMENTED_INVARIANTS`」是 2026-10-05 至 2026-10-06 形式化线交回里的当日计数或判断，事件。 |
| F914 | records/2026-10-01-里程碑三形式化接入.md:764 | 事件句不改 | 「0 条那一版上正常卸载不写盘（用户定案原话）、C678」是 2026-10-05 至 2026-10-06 形式化线交回里的当日计数或判断，事件。 |
| F914 | records/2026-10-01-里程碑三形式化接入.md:765 | 事件句不改 | 「0 条那一版答零条根）、18-3 写回员（opus」是 2026-10-05 至 2026-10-06 形式化线交回里的当日计数或判断，事件。 |
| F914 | records/2026-10-01-里程碑三形式化接入.md:769 | 事件句不改 | 「`IMPLEMENTED_INVARIANTS`」是 2026-10-05 至 2026-10-06 形式化线交回里的当日计数或判断，事件。 |
| F914 | records/2026-10-01-里程碑三形式化接入.md:797 | 事件句不改 | 「68 条，已实现的行也是」是 2026-10-05 至 2026-10-06 形式化线交回里的当日计数或判断，事件。 |
| F914 | records/2026-10-01-里程碑三形式化接入.md:817 | 事件句不改 | 「`IMPLEMENTED_INVARIANTS`」是 2026-10-05 至 2026-10-06 形式化线交回里的当日计数或判断，事件。 |
| F914 | records/2026-10-01-里程碑三形式化接入.md:891 | 事件句不改 | 「`IMPLEMENTED_INVARIANTS`」是 2026-10-05 至 2026-10-06 形式化线交回里的当日计数或判断，事件。 |
| F914 | records/2026-10-01-里程碑三形式化接入.md:990 | 事件句不改 | 「IMPLEMENTED_INVARIANTS」是 2026-10-05 至 2026-10-06 形式化线交回里的当日计数或判断，事件。 |
| F914 | records/2026-10-01-里程碑三形式化接入.md:1144 | 事件句不改 | 「`IMPLEMENTED_INVARIANTS`」是 2026-10-05 至 2026-10-06 形式化线交回里的当日计数或判断，事件。 |
| F914 | records/2026-10-11-崩溃放量两段与脚本审计.md:61 | 事件句不改 | 行内相关处是 2026-10-12 全量 j 的实测记录，其中「引擎侧已实现」说的是那一天的段登记，事件。（行首「- 2026-10-12：全量 j 第 4 项」；候选表原文只截到 500 字，命中处在行内靠后） |
| F914 | records/2026-10-11-里程碑三知识腐烂治理.md:30 | 事件句不改 | 「I-.*已实现' .claude/kb/invariants.md`」是 2026-10-11 治理的课题 3 现数（kb 90，清单 92），当日快照，批内已写回。 |
| F914 | records/2026-10-11-里程碑五打包与决策进度列.md:62 | 不相干 | 「353 条分项：已实现」是决策分项进度的合计（353 条分项），不是 checker 不变量条数。 |
| F914 | records/2026-10-12-里程碑三工具与治理批.md:126 | 事件句不改 | 行内相关处是 2026-10-12 第三轮判决交回，事件。（行首「- 第三轮正推腿交回（`research/pro」；候选表原文只截到 500 字，命中处在行内靠后） |
| F914 | records/2026-10-13-代码欠账批.md:56 | 事件句不改 | 行内相关处是 2026-10-14 别的会话来信的记录，事件。（行首「- 2026-10-14 别的会话（用户贴的同一」；候选表原文只截到 500 字，命中处在行内靠后） |
| F914 | records/2026-10-13-复用键与全仓改名.md:35 | 事件句不改 | 行内相关处是 2026-10-13 kb-scribe 交回里「D13 已定 16 项、已实现 12 项」的当日数，事件。（行首「- 2026-10-13：kb-scribe 交」；候选表原文只截到 500 字，命中处在行内靠后） |
| F914 | records/2026-10-13-复用键与全仓改名.md:621 | 事件句不改 | 「36 条全部写回 9 份 kb 文件（含 D44–D47」是 2026-10-13 4-3c 交回，事件。 |
| F914 | crates/singlefs-checker/invariants/01-块自描述.md:15 | 不相干 | I-1.10 一行里的「已实现」是它这一条自己的实现说明（行内相关处），不是池级总条数；总条数 92 与它无关。（行首「I-1.10  码 2 条目宽等于字段表宽」；候选表原文只截到 500 字，命中处在行内靠后） |
| F914 | crates/singlefs-checker/invariants/02-根环.md:11 | 不相干 | I-7.4 一行里的「已实现」是它这一条自己的实现说明（行内相关处），不是池级总条数；总条数 92 与它无关。（行首「I-7.4  近 K 代块未被复用  回」；候选表原文只截到 500 字，命中处在行内靠后） |
| F914 | crates/singlefs-checker/invariants/02-根环.md:12 | 不相干 | I-7.5 一行里的「已实现」是它这一条自己的实现说明（行内相关处），不是池级总条数；总条数 92 与它无关。（行首「I-7.5  根槽按判定宽度对齐  每个」；候选表原文只截到 500 字，命中处在行内靠后） |
| F914 | crates/singlefs-checker/invariants/02-根环.md:15 | 不相干 | I-7.8 一行里的「已实现」是它这一条自己的实现说明（行内相关处），不是池级总条数；总条数 92 与它无关。（行首「I-7.8  根记录树 ID 水位不低于全」；候选表原文只截到 500 字，命中处在行内靠后） |
| F914 | crates/singlefs-checker/invariants/02-根环.md:16 | 不相干 | I-7.9 一行里的「已实现」是它这一条自己的实现说明（行内相关处），不是池级总条数；总条数 92 与它无关。（行首「I-7.9  回退下界 F 不高于抬 F」；候选表原文只截到 500 字，命中处在行内靠后） |
| F914 | crates/singlefs-checker/invariants/02-根环.md:20 | 不相干 | I-7.13 一行里的「已实现」是它这一条自己的实现说明（行内相关处），不是池级总条数；总条数 92 与它无关。（行首「I-7.13  系统配置池级字段在读者收的」；候选表原文只截到 500 字，命中处在行内靠后） |
| F914 | crates/singlefs-checker/invariants/02-根环.md:21 | 不相干 | I-7.14 一行里的「已实现」是它这一条自己的实现说明（行内相关处），不是池级总条数；总条数 92 与它无关。（行首「I-7.14  系统配置本盘设备号等于盘在」；候选表原文只截到 500 字，命中处在行内靠后） |
| F914 | crates/singlefs-checker/invariants/02-根环.md:23 | 不相干 | I-7.16 一行里的「已实现」是它这一条自己的实现说明（行内相关处），不是池级总条数；总条数 92 与它无关。（行首「I-7.16  系统配置槽世代号与槽号的关」；候选表原文只截到 500 字，命中处在行内靠后） |
| F914 | crates/singlefs-checker/invariants/03-journal.md:12 | 不相干 | I-8.5 一行里的「已实现」是它这一条自己的实现说明（行内相关处），不是池级总条数；总条数 92 与它无关。（行首「I-8.5  tail 槽有效集非空」；候选表原文只截到 500 字，命中处在行内靠后） |
| F914 | crates/singlefs-checker/invariants/03-journal.md:17 | 不相干 | I-8.10 一行里的「已实现」是它这一条自己的实现说明（行内相关处），不是池级总条数；总条数 92 与它无关。（行首「I-8.10  记录新根段与同代根相同」；候选表原文只截到 500 字，命中处在行内靠后） |
| F914 | crates/singlefs-checker/invariants/04-校验和.md:9 | 不相干 | I-2.4 一行里的「已实现」是它这一条自己的实现说明（行内相关处），不是池级总条数；总条数 92 与它无关。（行首「I-2.4  头校验和覆盖范围  任一单」；候选表原文只截到 500 字，命中处在行内靠后） |
| F914 | crates/singlefs-checker/invariants/05-空间记账.md:8 | 不相干 | I-3.1 一行里的「已实现」是它这一条自己的实现说明（行内相关处），不是池级总条数；总条数 92 与它无关。（行首「I-3.1  已分配统计对得上  已分配」；候选表原文只截到 500 字，命中处在行内靠后） |
| F914 | crates/singlefs-checker/invariants/05-空间记账.md:11 | 不相干 | I-3.5 一行里的「已实现」是它这一条自己的实现说明（行内相关处），不是池级总条数；总条数 92 与它无关。（行首「I-3.5  引用区间的精确性  **引」；候选表原文只截到 500 字，命中处在行内靠后） |
| F914 | crates/singlefs-checker/invariants/05-空间记账.md:17 | 不相干 | I-3.10 一行里的「已实现」是它这一条自己的实现说明（行内相关处），不是池级总条数；总条数 92 与它无关。（行首「I-3.10  已分配记录的分配代等于它罩」；候选表原文只截到 500 字，命中处在行内靠后） |
| F914 | crates/singlefs-checker/invariants/05-空间记账.md:18 | 不相干 | I-3.11 一行里的「已实现」是它这一条自己的实现说明（行内相关处），不是池级总条数；总条数 92 与它无关。（行首「I-3.11  已分配减 defer 等于」；候选表原文只截到 500 字，命中处在行内靠后） |
| F914 | crates/singlefs-checker/invariants/07-分配器.md:6 | 不相干 | I-5.1 一行里的「已实现」是它这一条自己的实现说明（「已实现（池级 checker `crates/singlefs-checker`」），不是池级总条数；总条数 92 与它无关。 |
| F914 | crates/singlefs-checker/invariants/07-分配器.md:9 | 不相干 | I-5.4 一行里的「已实现」是它这一条自己的实现说明（行内相关处），不是池级总条数；总条数 92 与它无关。（行首「I-5.4  分配记录罩住的槽互不相交」；候选表原文只截到 500 字，命中处在行内靠后） |
| F914 | crates/singlefs-checker/invariants/07-分配器.md:10 | 不相干 | I-5.5 一行里的「已实现」是它这一条自己的实现说明（行内相关处），不是池级总条数；总条数 92 与它无关。（行首「I-5.5  从 mkfs 水位建树那一版」；候选表原文只截到 500 字，命中处在行内靠后） |
| F914 | crates/singlefs-checker/invariants/09-inode树结构.md:14 | 不相干 | I-9.2 一行里的「已实现」是它这一条自己的实现说明（行内相关处），不是池级总条数；总条数 92 与它无关。（行首「I-9.2  条目身份与子头相符  in」；候选表原文只截到 500 字，命中处在行内靠后） |
| F914 | crates/singlefs-checker/invariants/09-inode树结构.md:17 | 不相干 | I-9.5 一行里的「已实现」是它这一条自己的实现说明（行内相关处），不是池级总条数；总条数 92 与它无关。（行首「I-9.5  删除队列里的号仍在树里」；候选表原文只截到 500 字，命中处在行内靠后） |
| F914 | crates/singlefs-checker/invariants/09-inode树结构.md:28 | 不相干 | I-9.16 一行里的「已实现」是它这一条自己的实现说明（行内相关处），不是池级总条数；总条数 92 与它无关。（行首「I-9.16  树表条目按树 ID 严格升」；候选表原文只截到 500 字，命中处在行内靠后） |
| F914 | crates/singlefs-checker/invariants/09-inode树结构.md:29 | 不相干 | I-9.17 一行里的「已实现」是它这一条自己的实现说明（行内相关处），不是池级总条数；总条数 92 与它无关。（行首「I-9.17  谱系树条目身份与子头相符」；候选表原文只截到 500 字，命中处在行内靠后） |
| F914 | crates/singlefs-checker/invariants/09-inode树结构.md:35 | 不相干 | I-9.23 一行里的「已实现」是它这一条自己的实现说明（行内相关处），不是池级总条数；总条数 92 与它无关。（行首「I-9.23  墓碑树条目身份与子头相符」；候选表原文只截到 500 字，命中处在行内靠后） |
| F914 | crates/singlefs-checker/invariants/09-inode树结构.md:36 | 不相干 | I-9.24 一行里的「已实现」是它这一条自己的实现说明（「6 条的增补，用户定案），判决 `research/prompts/m3-blanks-r1-m」），不是池级总条数；总条数 92 与它无关。 |
| F914 | crates/singlefs-checker/invariants/09-inode树结构.md:37 | 不相干 | I-9.25 一行里的「已实现」是它这一条自己的实现说明（「2 条）。  已实现」），不是池级总条数；总条数 92 与它无关。 |
| F914 | crates/singlefs-checker/invariants/09-inode树结构.md:38 | 不相干 | I-9.26 一行里的「已实现」是它这一条自己的实现说明（行内相关处），不是池级总条数；总条数 92 与它无关。（行首「I-9.26  inode 改动计数落在出」；候选表原文只截到 500 字，命中处在行内靠后） |
| F914 | crates/singlefs-checker/invariants/09-inode树结构.md:39 | 不相干 | I-9.27 一行里的「已实现」是它这一条自己的实现说明（「2 条目的树 ID，表里有多于一条种类 2 条目时放宽成「等于其中某一条的树」），不是池级总条数；总条数 92 与它无关。 |
| F914 | crates/singlefs-checker/invariants/09-inode树结构.md:40 | 不相干 | I-9.28 一行里的「已实现」是它这一条自己的实现说明（行内相关处），不是池级总条数；总条数 92 与它无关。（行首「I-9.28  全部可读根里同一树 ID」；候选表原文只截到 500 字，命中处在行内靠后） |
| F914 | crates/singlefs-checker/invariants/09-inode树结构.md:41 | 不相干 | I-9.29 一行里的「已实现」是它这一条自己的实现说明（行内相关处），不是池级总条数；总条数 92 与它无关。（行首「I-9.29  同号只许一个出生代  回」；候选表原文只截到 500 字，命中处在行内靠后） |
| F914 | crates/singlefs-checker/invariants/09-inode树结构.md:42 | 不相干 | I-9.30 一行里的「已实现」是它这一条自己的实现说明（「5 条；正推草案 `research/prompts/m3-w3-r1-sonnet-outpu」），不是池级总条数；总条数 92 与它无关。 |
| F914 | crates/singlefs-checker/invariants/09-inode树结构.md:43 | 不相干 | I-9.31 一行里的「已实现」是它这一条自己的实现说明（「8 条  已实现」），不是池级总条数；总条数 92 与它无关。 |
| F914 | crates/singlefs-checker/invariants/09-inode树结构.md:44 | 不相干 | I-9.32 一行里的「已实现」是它这一条自己的实现说明（「8 条  已实现」），不是池级总条数；总条数 92 与它无关。 |
| F914 | crates/singlefs-checker/invariants/09-inode树结构.md:48 | 不相干 | I-9.36 一行里的「已实现」是它这一条自己的实现说明（「`IMPLEMENTED_INVARIANTS`」），不是池级总条数；总条数 92 与它无关。 |
| F914 | crates/singlefs-checker/invariants/09-inode树结构.md:49 | 不相干 | I-9.37 一行里的「已实现」是它这一条自己的实现说明（「`IMPLEMENTED_INVARIANTS`」），不是池级总条数；总条数 92 与它无关。 |
| F914 | crates/singlefs-checker/invariants/README.md:29 | 不相干 | 「92 条（`crates/singlefs-checker/src/image.rs`」给出的 92 条（清单长度、状态列行数、违例位 92、覆盖下标 92）与现状一致：`IMPLEMENTED_INVARIANTS` 长度 92，`grep -ch` 数状态列「已实现」的行现数 92，仍真。 |
| F914 | crates/singlefs-checker/invariants/README.md:33 | 不相干 | 「`IMPLEMENTED_INVARIANTS`」给出的 92 条（清单长度、状态列行数、违例位 92、覆盖下标 92）与现状一致：`IMPLEMENTED_INVARIANTS` 长度 92，`grep -ch` 数状态列「已实现」的行现数 92，仍真。 |

### F915

| 组 | 载体 | 判定 | 改后的句子或理由 |
|---|---|---|---|
| F915 | .claude/kb/checks-owed.md:830 | 不相干 | 这一行已写到新 crate：「`crates/singlefs-unit-graph`」，并说明 `crates/singlefs-unit-graph` 按函数单元切（Rust、WGSL），与现状一致。 |
| F915 | .claude/kb/milestone/02-second-txn.md:503 | 事件句不改 | 行内相关处是 2026-09-28 立项背景里「GPU 代码住新建的第六个 crate」这一格的判决，说的是当时的决定与当时 crate 数；今天的数在 CLAUDE.md。（行首「- **仓里已有的**：两台主机共 5 张卡，可」；候选表原文只截到 500 字，命中处在行内靠后） |
| F915 | .claude/kb/verification-build.md:4 | 要改 | 原句「**现状**：`crates/` 下七个 crate」已不真：`ls -d crates/*/` 数出 8 个目录，多出 `crates/singlefs-unit-graph`。改后：「**现状**：`crates/` 下八个 crate——`singlefs-format`（格式常量）、`singlefs-core`（…同原句…）、`singlefs-replay-prefix`（…同原句…）、`singlefs-reference-filesystem`（…同原句…）、`singlefs-unit-graph`（函数单元图：Rust 与 WGSL 源码按条目切成单元、取去注释指纹、收调用闭包、算复用键，只用 std、不依赖工作区任何 crate，`singlefs-checker-tier` 的库与 `build.rs` 依赖它，`build.rs` 经它按 `judging_units_entries.rs` 算判器组单元键）、`singlefs-harness`（…同原句…）、`singlefs-checker`（…同原句…）、`singlefs-checker-tier`（…同原句…）」。同一行后半「登记的崩溃枚举用例 13 条」现为 14 条（`grep -c ^crash-case: .claude/gate.d/stage-inputs.tsv` 现数 14），一并改。 |
| F915 | .claude/rules/verification.md:36 | 不相干 | 这一行已提到新 crate：行内相关处，写的是 build.rs 经 `crates/singlefs-unit-graph` 算判器组单元键，与现状一致。（行首「崩溃放量全量  `bash scripts」；候选表原文只截到 500 字，命中处在行内靠后） |
| F915 | CLAUDE.md:8 | 要改 | 原句「代码在 `crates/` 下七个 crate（格式常量、核心、池级 checker、harness 档、checker 档、恢复重放取前缀的核 `singlefs-replay-prefix`、参照文件系统 RefFS；…）」已不真：现有八个。改后：「代码在 `crates/` 下八个 crate（格式常量、核心、池级 checker、harness 档、checker 档、恢复重放取前缀的核 `singlefs-replay-prefix`、参照文件系统 RefFS、函数单元图 `singlefs-unit-graph`；两档怎么分见 `.claude/rules/verification.md`）」。 |
| F915 | records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md:47 | 事件句不改 | 行内相关处是 2026-09-29 当天调试 GPU 判器时的记录（当时五个 crate 都 forbid unsafe），事件。（行首「GPU 判器：三批 49 条全罩、17 条流」；候选表原文只截到 500 字，命中处在行内靠后） |
| F915 | records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md:48 | 事件句不改 | 行内相关处是 2026-09-29 当天调试 GPU 判器时的记录（当时五个 crate 都 forbid unsafe），事件。（行首「GPU 判器：池级 checker 三批写完」；候选表原文只截到 500 字，命中处在行内靠后） |
| F915 | records/2026-10-01-里程碑三形式化接入.md:142 | 事件句不改 | 「ependence、`CLAUDE.md`「五个 crate」」是 2026-10-01 记录里派发重派时列的后续要改的非 crates 文件（含 CLAUDE.md 的「五个 crate」），是待办事件；今天的 CLAUDE.md 现状另行判。 |
| F915 | records/2026-10-11-里程碑三知识腐烂治理.md:52 | 事件句不改 | 「五个 crate」是 2026-10-11 治理那天做了什么（五个 crate 改成七个），事件；当时的数后来又随新 crate 变成八个。 |
| F915 | records/2026-10-13-代码欠账批.md:167 | 事件句不改 | 行内相关处是 2026-10-16 的代码欠账批进展记录，事件。（行首「- 2026-10-16 外部审核报来七条，主」；候选表原文只截到 500 字，命中处在行内靠后） |
| F915 | records/2026-10-13-代码欠账批.md:168 | 事件句不改 | 「`singlefs-unit-graph`」是 2026-10-16 的代码欠账批进展记录，事件。 |
| F915 | records/2026-10-13-代码欠账批.md:169 | 事件句不改 | 「`singlefs-unit-graph`」是 2026-10-16 的代码欠账批进展记录，事件。 |
| F915 | records/2026-10-13-复用键与全仓改名.md:503 | 事件句不改 | 「`singlefs-unit-graph`」是 2026-10-13 至 2026-10-17 复用键与全仓改名批里新 crate 的建立、对齐与审核的过程记录，事件。 |
| F915 | records/2026-10-13-复用键与全仓改名.md:514 | 事件句不改 | 「`singlefs-unit-graph`」是 2026-10-13 至 2026-10-17 复用键与全仓改名批里新 crate 的建立、对齐与审核的过程记录，事件。 |
| F915 | records/2026-10-13-复用键与全仓改名.md:522 | 事件句不改 | 「`crates/singlefs-unit-graph`」是 2026-10-13 至 2026-10-17 复用键与全仓改名批里新 crate 的建立、对齐与审核的过程记录，事件。 |
| F915 | records/2026-10-13-复用键与全仓改名.md:540 | 事件句不改 | 「`singlefs-unit-graph`」是 2026-10-13 至 2026-10-17 复用键与全仓改名批里新 crate 的建立、对齐与审核的过程记录，事件。 |
| F915 | records/2026-10-13-复用键与全仓改名.md:544 | 事件句不改 | 「singlefs-unit-graph`」是 2026-10-13 至 2026-10-17 复用键与全仓改名批里新 crate 的建立、对齐与审核的过程记录，事件。 |
| F915 | records/2026-10-13-复用键与全仓改名.md:574 | 事件句不改 | 「`singlefs-unit-graph`」是 2026-10-13 至 2026-10-17 复用键与全仓改名批里新 crate 的建立、对齐与审核的过程记录，事件。 |
| F915 | records/2026-10-13-复用键与全仓改名.md:693 | 事件句不改 | 「singlefs-unit-graph`」是 2026-10-13 至 2026-10-17 复用键与全仓改名批里新 crate 的建立、对齐与审核的过程记录，事件。 |
| F915 | records/2026-10-13-复用键与全仓改名.md:705 | 事件句不改 | 「singlefs-unit-graph`」是 2026-10-13 至 2026-10-17 复用键与全仓改名批里新 crate 的建立、对齐与审核的过程记录，事件。 |
| F915 | records/2026-10-13-复用键与全仓改名.md:835 | 事件句不改 | 「`crates/singlefs-unit-graph`」是 2026-10-13 至 2026-10-17 复用键与全仓改名批里新 crate 的建立、对齐与审核的过程记录，事件。 |
| F915 | records/2026-10-13-复用键与全仓改名.md:858 | 事件句不改 | 行内相关处是 2026-10-13 至 2026-10-17 复用键与全仓改名批里新 crate 的建立、对齐与审核的过程记录，事件。（行首「- 用户问「确认没有遗漏」，主 agent 把四」；候选表原文只截到 500 字，命中处在行内靠后） |
| F915 | records/2026-10-13-复用键与全仓改名.md:871 | 事件句不改 | 「`crates/singlefs-unit-graph/src/closure.rs`」是 2026-10-13 至 2026-10-17 复用键与全仓改名批里新 crate 的建立、对齐与审核的过程记录，事件。 |
| F915 | records/2026-10-13-复用键与全仓改名.md:874 | 事件句不改 | 「`crates/singlefs-unit-graph`」是 2026-10-13 至 2026-10-17 复用键与全仓改名批里新 crate 的建立、对齐与审核的过程记录，事件。 |
| F915 | records/2026-10-13-复用键与全仓改名.md:882 | 事件句不改 | 「crates/singlefs-unit-graph/src/lib.rs`」是 2026-10-13 至 2026-10-17 复用键与全仓改名批里新 crate 的建立、对齐与审核的过程记录，事件。 |
| F915 | scripts/gate/README.md:10 | 不相干 | 这一行已把 `crates/singlefs-unit-graph` 写进去（「`crates/singlefs-unit-graph`」），与现状一致。 |
| F915 | scripts/mutation/README.md:10 | 不相干 | 这一行已把 `crates/singlefs-unit-graph` 写进去（「`crates/singlefs-unit-graph`」），与现状一致。 |

### F916

| 组 | 载体 | 判定 | 改后的句子或理由 |
|---|---|---|---|
| F916 | .claude/kb/checks-owed.md:820 | 不相干 | 这一行已按新事实写：「`oov-check.py`」，说明本地模型腿不再派、字词损坏检测器 2026-10-11 从仓里删掉，与现状一致。 |
| F916 | .claude/kb/experiments/34-根环槽几何.md:209 | 事件句不改 | 「scripts/agents/ask-local.sh」是实验 34 页里那一次本地腿问答所用的命令（紧接「本地腿那一次问答（材料已过期）」），记当时怎么跑的，不是说脚本今天还在。 |
| F916 | .claude/kb/tooling.md:8 | 不相干 | 这一行已按新事实写：「`ask-local.sh`、`ask-local-selftest.sh`、」已列明这批脚本 2026-10-11 删掉，要取回走 `git log --diff-filter=D`，与现状一致。 |
| F916 | records/2026-09-06-D8项8第二轮对抗.md:10 | 事件句不改 | 「`corruption-check.py`」是 2026-09 的记录里当时对 `corruption-check.py`、`oov-check.py` 的补丁与判绿，事件。 |
| F916 | records/2026-09-09-D22项6项9尝试定案.md:171 | 事件句不改 | 「`oov-check.py`」是 2026-09 的记录里当时对 `corruption-check.py`、`oov-check.py` 的补丁与判绿，事件。 |
| F916 | records/2026-09-09-D22项6项9尝试定案.md:179 | 事件句不改 | 「`corruption-check.py`」是 2026-09 的记录里当时对 `corruption-check.py`、`oov-check.py` 的补丁与判绿，事件。 |
| F916 | records/2026-09-16-subagent拆分提案.md:85 | 事件句不改 | 「`ask-local.sh`」是 2026-09-16 拆分提案里当时对本地攻方、`ask-local.sh`、`oov-check.py` 的设计、缺陷与处理记录，提案是建设过程，不随后来删脚本改写。 |
| F916 | records/2026-09-16-subagent拆分提案.md:185 | 事件句不改 | 「`oov-check.py`」是 2026-09-16 拆分提案里当时对本地攻方、`ask-local.sh`、`oov-check.py` 的设计、缺陷与处理记录，提案是建设过程，不随后来删脚本改写。 |
| F916 | records/2026-09-16-subagent拆分提案.md:235 | 事件句不改 | 「`scripts/agents/oov-check.py`」是 2026-09-16 拆分提案里当时对本地攻方、`ask-local.sh`、`oov-check.py` 的设计、缺陷与处理记录，提案是建设过程，不随后来删脚本改写。 |
| F916 | records/2026-09-16-subagent拆分提案.md:279 | 事件句不改 | 「`ask-local.sh`」是 2026-09-16 拆分提案里当时对本地攻方、`ask-local.sh`、`oov-check.py` 的设计、缺陷与处理记录，提案是建设过程，不随后来删脚本改写。 |
| F916 | records/2026-09-16-subagent拆分提案.md:508 | 事件句不改 | 「`scripts/agents/oov-check.py`」是 2026-09-16 拆分提案里当时对本地攻方、`ask-local.sh`、`oov-check.py` 的设计、缺陷与处理记录，提案是建设过程，不随后来删脚本改写。 |
| F916 | records/2026-09-16-subagent拆分提案.md:816 | 事件句不改 | 「ask-local.sh」是 2026-09-16 拆分提案里当时对本地攻方、`ask-local.sh`、`oov-check.py` 的设计、缺陷与处理记录，提案是建设过程，不随后来删脚本改写。 |
| F916 | records/2026-09-16-subagent拆分提案.md:913 | 事件句不改 | 「`ask-local.sh」是 2026-09-16 拆分提案里当时对本地攻方、`ask-local.sh`、`oov-check.py` 的设计、缺陷与处理记录，提案是建设过程，不随后来删脚本改写。 |
| F916 | records/2026-09-16-subagent拆分提案.md:967 | 事件句不改 | 「ns.sh`、`pdf-text.py`、`ask-local.sh`、`local-repro-tuned.sh`、`」是 2026-09-16 拆分提案里当时对本地攻方、`ask-local.sh`、`oov-check.py` 的设计、缺陷与处理记录，提案是建设过程，不随后来删脚本改写。 |
| F916 | records/2026-09-17-已分配口径三方与两个实验.md:32 | 事件句不改 | 「`scripts/agents/oov-check.py`」是 2026-09 记录里当天的改动与素材，事件。 |
| F916 | records/2026-09-24-里程碑二收尾调度.md:181 | 事件句不改 | 行内相关处是 2026-09 记录里当天的改动与素材，事件。（行首「第十步素材（2026-09-26 起攒，第十」；候选表原文只截到 500 字，命中处在行内靠后） |
| F916 | records/2026-09-27-同步远端冲突块.md:392 | 事件句不改 | 「scripts/agents/ask-local.sh」是 2026-09-27 同步远端冲突块里抄的当时的 agent 定义与改动说明，冲突块是当天快照，本地腿后来已退役。 |
| F916 | records/2026-09-27-同步远端冲突块.md:394 | 事件句不改 | 「scripts/agents/oov-check.py」是 2026-09-27 同步远端冲突块里抄的当时的 agent 定义与改动说明，冲突块是当天快照，本地腿后来已退役。 |
| F916 | records/2026-09-27-同步远端冲突块.md:400 | 事件句不改 | 「scripts/agents/ask-local.sh」是 2026-09-27 同步远端冲突块里抄的当时的 agent 定义与改动说明，冲突块是当天快照，本地腿后来已退役。 |
| F916 | records/2026-09-27-同步远端冲突块.md:402 | 事件句不改 | 「scripts/agents/oov-check.py」是 2026-09-27 同步远端冲突块里抄的当时的 agent 定义与改动说明，冲突块是当天快照，本地腿后来已退役。 |
| F916 | records/2026-09-27-同步远端冲突块.md:1299 | 事件句不改 | 「6、oov-check」是 2026-09-27 同步远端冲突块里抄的当时的 agent 定义与改动说明，冲突块是当天快照，本地腿后来已退役。 |
| F916 | records/2026-09-28-agent体系用量时间与调度分析.md:111 | 事件句不改 | 「`ask-local.sh`、过损坏闸、再抽样」是 2026-09-28 用量分析表里当时的一格，事件。 |
| F916 | records/2026-09-28-agent体系用量时间与调度分析.md:127 | 事件句不改 | 批次快照：「`ask-local.sh`、`corruption-check.py`、`o」写的是 2026-09-29 的状态（那几份当时「留着」，「要不要删归下一次阶段同步」），后来 2026-10-11 已删。 |
| F916 | records/2026-09-28-agent体系用量时间与调度分析.md:128 | 事件句不改 | 「`corruption-check.py`」是 2026-09-28 用量分析里当时的记述，事件。 |
| F916 | records/2026-09-28-agent体系用量时间与调度分析.md:217 | 事件句不改 | 「`ask-local.sh`」是 2026-09-28 用量分析里当时的记述，事件。 |
| F916 | records/2026-09-28-agent体系用量时间与调度分析.md:348 | 事件句不改 | 批次快照：「`ask-local.sh`、两道损坏闸与门禁」是 2026-09-28 记「本地腿删掉之后的残留」时的状态，「归下一次阶段同步判要不要删」后来在 2026-10-11 做了。 |
| F916 | records/2026-10-01-对接上游测试脚本规范.md:177 | 事件句不改 | 「`scripts/agents/corruption-check.py`」是 2026-10-01 对接上游脚本规范时的审计表一格，事件。 |
| F916 | records/2026-10-11-scripts审计-第二轮.md:36 | 事件句不改 | 「`ask-local.sh`、`ask-local-selftest.sh`、」是 2026-10-11 scripts 审计第二轮的审计行（建议退役、API key 放参数里），审计表是当天快照，脚本随后已删。 |
| F916 | records/2026-10-11-scripts审计-第二轮.md:39 | 事件句不改 | 「`scripts/agents/ask-local.sh:40`、`local-repro.sh:26`」是 2026-10-11 scripts 审计第二轮的审计行（建议退役、API key 放参数里），审计表是当天快照，脚本随后已删。 |

### F917

| 组 | 载体 | 判定 | 改后的句子或理由 |
|---|---|---|---|
| F917 | .claude/kb/tooling.md:94 | 不相干 | 这一行已按新做法写：标题「remind.sh」与现状一致。 |
| F917 | .claude/kb/tooling.md:96 | 不相干 | 这一行已按新做法写：「scripts/agents/remind.sh」与 `scripts/agents/remind.sh`、规则、skill 现状一致。 |
| F917 | .claude/rules/timer-only-remind.md:1 | 不相干 | 这一行已按新做法写：「scripts/agents/remind.sh」与现状一致（`remind.sh` 是唯一定时入口）。 |
| F917 | .claude/rules/timer-only-remind.md:5 | 不相干 | 这一行已按新做法写：「scripts/agents/remind.sh`」与现状一致（`remind.sh` 是唯一定时入口）。 |
| F917 | .claude/rules/timer-only-remind.md:9 | 不相干 | 这一行已按新做法写：「scripts/agents/remind.sh」与现状一致（`remind.sh` 是唯一定时入口）。 |
| F917 | .claude/rules/timer-only-remind.md:15 | 不相干 | 这一行是被拒清单：「`CronCreate`、`ScheduleWakeup`、`Remote」，与 `.claude/settings.json` 里钩子 matcher 一致，仍真。 |
| F917 | .claude/rules/timer-only-remind.md:27 | 不相干 | 「CronCreate、ScheduleWakeup、RemoteTrig」与 `.claude/settings.json` 一致：钩子 matcher 为 CronCreate、ScheduleWakeup、RemoteTrigger、Skill、Monitor、Bash，停止钩子也已注册，仍真。 |
| F917 | .claude/skills/remind/SKILL.md:3 | 不相干 | 这一行已按新做法写：「scripts/agents/remind.sh」与 `scripts/agents/remind.sh` 的现行用法一致。 |
| F917 | .claude/skills/remind/SKILL.md:6 | 不相干 | 这一行已按新做法写：「scripts/agents/remind.sh`」与 `scripts/agents/remind.sh` 的现行用法一致。 |
| F917 | .claude/skills/remind/SKILL.md:20 | 不相干 | 这一行已按新做法写：「scripts/agents/remind.sh」与 `scripts/agents/remind.sh` 的现行用法一致。 |
| F917 | .claude/skills/remind/SKILL.md:21 | 不相干 | 这一行已按新做法写：「scripts/agents/remind.sh」与 `scripts/agents/remind.sh` 的现行用法一致。 |
| F917 | .claude/skills/remind/SKILL.md:22 | 不相干 | 这一行已按新做法写：「scripts/agents/remind.sh」与 `scripts/agents/remind.sh` 的现行用法一致。 |
| F917 | .claude/skills/remind/SKILL.md:23 | 不相干 | 这一行已按新做法写：「scripts/agents/remind.sh」与 `scripts/agents/remind.sh` 的现行用法一致。 |
| F917 | .claude/skills/remind/SKILL.md:40 | 不相干 | 这一行是被拒清单：「`CronCreate`、`ScheduleWakeup`、`Remote」，与现状一致，仍真。 |
| F917 | records/2026-09-16-subagent拆分提案.md:252 | 事件句不改 | 「、Workflow、SendMessage、CronCreate、RemoteTrigger、WebFetch、We」是 2026-09-16 嵌套会话探测的实测结果（当时 38 个工具里含 CronCreate），事件。 |
| F917 | records/2026-09-24-里程碑二收尾调度.md:214 | 事件句不改 | 批次快照：「CronCreate」记 2026-09-27 恢复派发用了「CronCreate 一次性任务与后台 sleep」，那一天的做法；今天只用 `remind.sh`。 |
| F917 | records/2026-10-13-复用键与全仓改名.md:125 | 事件句不改 | 「`scripts/agents/remind.sh`、`.claude/hooks/remind-re」是 2026-10-13 工具员交回（新建 remind.sh 与钩子），是做成这件事的记录，事件。 |
| F917 | records/2026-10-13-复用键与全仓改名.md:127 | 事件句不改 | 「里「CronCreate」是 2026-10-13 工具员交回的欠下项，后来 kb-scribe 已把 `tooling.md` 定时一节改掉（见同记录下一条），事件。 |
| F917 | records/2026-10-13-复用键与全仓改名.md:128 | 事件句不改 | 「`remind.sh`」是 2026-10-13 看门狗误报的记录，事件。 |
| F917 | records/2026-10-13-复用键与全仓改名.md:304 | 事件句不改 | 「`remind.sh`」就是这件事实的落地记录（`tooling.md` 定时一节改成只用 remind.sh），事件。 |
| F917 | scripts/agents/README.md:12 | 不相干 | 这一行已按新做法写：「`remind.sh`」与现状一致。 |

## 要主 agent 现查

本组没有判「要人看」的行（五种判定里没有这一种）。下面是做判定时顺带查到的、要主 agent 接手的事，按重要性排。

1. 三行「要改」的改后句子在上面表里（F910 `.claude/kb/checks-owed.md:369`；F915 `.claude/kb/verification-build.md:4` 与 `CLAUDE.md:8`，F915 的前两行同时带 13 条改 14 条）。C564 那一行的依据是读代码得的（崩溃枚举一律经 `enumerate_crash_states_through_the_library`，没跑），改前请主 agent 认一下：「断点续跑」一样整句去掉算还清，还是要挂一条「打断重跑只补没判的块」的用例才算。
2. 候选表没罩到、同一件事实的别处（`stage-inputs.tsv`、README 不在候选表载体里，下面是我现查到的位置）：
   - `.claude/gate.d/stage-inputs.tsv:52`（tree-split-streams 一行）写「不留进度文件（没有断点续跑）」，同文件 :48、:52、:53 都有「不留进度文件」：进度文件取消后这几句字面仍真，「没有断点续跑」一句不真（枚举经判定库，打断重跑复用已判的块）。
   - `.claude/gate.d/crash-point-replay-items.tsv:2` 头注写「stage-inputs.tsv 的 13 条 crash-case:」，现为 14 条（`grep -c '^crash-case:' .claude/gate.d/stage-inputs.tsv` = 14）。
   - 三语 README 的 crate 表（`README.md:130-136`、`README.zh.md:130-136`、`README.ja.md:130-136`，标题「## Layout」下的表）都没有 `crates/singlefs-unit-graph`，三处要同步各加一行（插在 `singlefs-replay-prefix` 那一行之后即可）：
     - README.md：`| [`crates/singlefs-unit-graph`](crates/singlefs-unit-graph) | Function-unit graph: cuts Rust and WGSL source into units, takes comment-free fingerprints, collects call closures and computes reuse keys; std only; the checker tier's build script uses it to key the judge groups |`
     - README.zh.md：`| [`crates/singlefs-unit-graph`](crates/singlefs-unit-graph) | 函数单元图：Rust 与 WGSL 源码按条目切成单元、取去注释指纹、收调用闭包、算复用键，只用 std；checker 档的 build.rs 经它给判器组算单元键 |`
     - README.ja.md：`| [`crates/singlefs-unit-graph`](crates/singlefs-unit-graph) | 関数単位グラフ: Rust と WGSL のソースを項目ごとに単位へ切り、コメントを除いた指紋を取り、呼び出しの閉包を集めて再利用キーを計算する。std のみ。checker 層の build.rs がこれで判定器グループの単位キーを計算する |`
3. F914 的 `.claude/kb/checks-owed.md` 等行里「待提交落地」一类的说法（例：C840、C841 两行）不属于本组事实，没判，只是看到了。

## 汇总

- 候选 249 行，逐行判定表 249 行（`awk '/^## 逐行判定/{i=1} i&&/^\| F9/{n++} END{print n}'` 数得 249）。
- 要改 3 行（F910 一行、F915 两行）；要补 0；事件句不改 142；不相干 104；要人看 0。
- 分组（用 `awk` 按组与判定数出）：F910 44 行（要改 1、事件句不改 34、不相干 9）；F911 25 行（事件句不改 12、不相干 13）；F914 103 行（事件句不改 42、不相干 61）；F915 27 行（要改 2、事件句不改 21、不相干 4）；F916 29 行（事件句不改 27、不相干 2）；F917 21 行（事件句不改 6、不相干 15）。
- 同一载体行在我的几个组里命中的有两处（`records/2026-09-24-里程碑二收尾调度.md:301` 在 F910 与 F914，`records/2026-10-11-里程碑三知识腐烂治理.md:30` 在 F911 与 F914），每组各写一行，判定相同（都是事件句不改）。

## 没做什么

- 没改任何仓内文件；没跑 `--facts`/`--check-facts`，没起编译与重型测试，没建副本。
- C564 的「断点续跑」已还清这一判断来自读代码，没有真中断重跑验证。
- 第 10 步反向核（该记这一阶段新东西而没记的载体）按派发提示这一轮没做。
- F914 里 `.claude/kb/checks-owed.md` 各行逐笔欠账的其他说法（如「待提交落地」）不属本组事实，没判。

## 第 11 步 `--check-report` 输出末行


    ✓ 报告判全了：249 行候选都有逐行判定
