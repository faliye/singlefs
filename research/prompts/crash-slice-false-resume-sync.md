<!-- knowledge-sync -->
# crash-slice-false-resume 阶段同步

范围：基准 `59ce5a865`（HEAD）到只含这一批改动的提交对象 `7ac20c0204a7587b580a8eb2d78c7efead4e873f`（主工作区混着别的会话的改动，照 `scripts/kb/stale-candidates.py` 文件头的出路做的；做法在调度记录「批次 12 派发」一节）。调度记录 `records/2026-10-28-双机崩溃枚举用例键变后切片假续跑.md`。

触发文件：crates/singlefs-checker-tier/src/crash_amplification.rs、crates/singlefs-checker-tier/src/crash_state_enumeration.rs、crates/singlefs-checker-tier/src/verdict_store.rs、crates/singlefs-checker-tier/src/crash_point_replay_provenance.rs、crates/singlefs-checker-tier/src/judging_units_entries.rs、scripts/gate/admission.py、scripts/gate/lib_function_units.py、scripts/hosts/multi-host-run.sh、scripts/hosts/run-with-memory-cap.sh、scripts/hosts/detached-run.sh、scripts/hosts/README.md、.claude/gate.d/stage-inputs.tsv、.claude/gate.d/crash-point-replay-items.tsv、.claude/gate.d/crates-mutation-light-rows.tsv、.claude/gate.d/break-switch-replay-files.tsv、.claude/rules/implementation-workflow.md、.claude/rules/verification.md

## 搜索

- `python3 scripts/kb/stale-candidates.py --changes --base 59ce5a865 --target 7ac20c0204a7587b580a8eb2d78c7efead4e873f --out changes.md` → 变更记录 0 条（结束提交对象里现状载体没有删改的片段）
- `python3 scripts/kb/stale-candidates.py --check-facts facts.tsv --base 59ce5a865 --target 7ac20c02…` → 「✓ 事实表罩全了：0 条变更记录都有出处，24 行事实的检索词都在基准那一版的现状句里命中（新立事项除外、它们在基准零命中）」。回扫员交的是 25 行，F10 依据的是主 agent 派发提示里写错的「改名自」，主 agent 删掉后重核。
- `python3 scripts/kb/stale-candidates.py --facts facts.tsv --base 59ce5a865 --target 7ac20c02… --out candidates.tsv` → 「✓ 候选表 …：640 行（585 个不同的行）」
- `python3 scripts/kb/stale-candidates.py --check-report candidates.tsv judge-a.md judge-b.md judge-c.md judge-d.md` → 「✓ 报告判全了：640 行候选都有逐行判定；crates/ 下判要改要补的 0 行都在「交实现员」一节里（不变量清单的 .md 除外）」。四段合计：事件句不改 498、不相干 125、要改 20、要补 7、要人看 3。
- 抽查：种子 20261030，五种判定各抽 5 行共 25 行，逐行对着结束提交里的原文现查，0 行判错。
- 反向问出的漏记：四段都没漏「假续跑根因、四条新用例、55 行变异、`--peer` 拒带库命令、C924、C925、拆分提案三节」的记载；漏的是里程碑四「3 覆盖」那一格、`scripts/hosts/README.md` 那一行与三份定义的三句（定义的登进拆分提案第一百五十三节）。
- 回扫员顺带提到、不是这一阶段引起的两处，记在调度记录、不改：C914 还写查库「一条接一条」；`.claude/kb/milestone/03-third-txn.md:226` 说 `gpu_judge_units_fingerprint_of_this_process` 不带特性时没人调。

## 命中处置

| 载体 | 原句 | 处置 |
|---|---|---|
| `.claude/kb/checks-owed.md:585` | C910 引 `crash_amplification.rs` 第 20979、20983 行 | 改了：行号改成主工作区现行的第 24181、24185 行（结束提交里是 21034、21038，主工作区混着别的会话的改动，提交以主工作区为准） |
| `.claude/kb/checks-owed.en.md:572` | C910 line 20979、line 20983 | 补了：英语那一行同改成 24181、24185（检索词只有中文，候选没罩到） |
| `.claude/kb/checks-owed.ja.md:572` | C910 第 20979 行、第 20983 行 | 补了：日语那一行同改成 24181、24185 |
| `.claude/rules/implementation-workflow.md:68` | 核它的崩溃状态块在崩溃判定库里都有单元键相同、不红的判定 | 改了：写成判器结论与挂在这个点上的观察者结论都要有，缺任一种、或按用例合计 blocks=0，都不算罩住 |
| `.claude/rules/verification.md:35` | 每个崩溃状态块在崩溃判定库里都有单元键相同、不红的判定才算罩住 | 改了：同上，补观察者结论与 blocks=0 两条 |
| `.claude/kb/milestone/04-fourth-txn.md:38` | 流已入库并登记在 `.claude/gate.d/stage-inputs.tsv` 的 `crash-case:` 行 | 补了：三语这一格末尾补四条新用例已登记、全量一次没跑、假续跑已修、用户定这一阶段不重跑 |
| `.claude/kb/milestone/04-fourth-txn.md:25` | 出口 10：崩溃全量……每个里程碑至少 100 亿态 | 不改：出口 10 写的是要跑的量，不记进度；进度补在「3 覆盖」那一格（同一件事，上一行） |
| `.claude/kb/checks-owed.md:502` | C786 按计划导入几份库同一块各数一次 | 改了：三语都从开着的表挪进「已还清」，写怎么还的（`imported_blocks` 按块键去重、库单测与证过红的变异行） |
| `.claude/kb/checks-owed.en.md:502` | C786 counts same block once per copy | 改了：同上，英语 |
| `.claude/kb/checks-owed.ja.md:502` | C786 同一ブロックを各コピーでカウント | 改了：同上，日语 |
| `.claude/kb/milestone/05-fifth-txn.md:153` | 八·3……双机导入的块只数一次……C786（按计划导入几份库同一块各数一次） | 改了：去掉这一项作业与 C786 那一笔 |
| `.claude/kb/milestone/05-fifth-txn.en.md:153` | VIII·3 … blocks imported across two-host counted only once … C786 | 改了：同上，英语 |
| `.claude/kb/milestone/05-fifth-txn.ja.md:153` | 八·3 … 二台構成でインポートされたブロックを 1 回のみカウント … C786 | 改了：同上，日语 |
| `.claude/kb/checks-owed.md:348` | C564 `--full` 已登记 34 条崩溃枚举用例 | 改了：三语都改成条数用 `grep -c '^crash-case:'` 现数、正文不抄数 |
| `.claude/kb/checks-owed.en.md:348` | C564 has 34 crash enumeration cases registered | 改了：同上，英语 |
| `.claude/kb/checks-owed.ja.md:348` | C564 34 本のクラッシュ列挙のテストケースが登録済み | 改了：同上，日语 |
| `.claude/kb/checks-owed.md:589` | C914 对 36 条登记用例逐条 | 改了：三语都改成对每条登记用例（条数现数，不抄数） |
| `.claude/kb/checks-owed.en.md:589` | C914 each of the 36 registered cases | 改了：同上，英语 |
| `.claude/kb/checks-owed.ja.md:589` | C914 登録された 36 ケース | 改了：同上，日语 |
| `.claude/kb/verification-build.md:303` | 轻型变异的核心部分……现有 166 行（`grep -vcE` 现数 | 改了：三语都改成行数用那条 `grep` 现数、正文不抄数（整表里混着别的会话这一段加的行，不写 298） |
| `.claude/kb/verification-build.en.md:256` | currently 166 rows | 改了：同上，英语 |
| `.claude/kb/verification-build.ja.md:256` | 現在166行 | 改了：同上，日语 |
| `records/2026-10-23-subagent拆分提案.md:1255` | 一百四十八 欠：`multi-host-run.sh --peer` 后面的命令是带判定库的门禁或入口时拒 | 补了：「欠」那一行之后加一行去向：已改（批次 11b），算还清以提交前门禁绿为准 |
| `records/2026-10-23-subagent拆分提案.md:1260` | 一百四十九（形式化门禁只能在本机跑）欠：……改成不给这条出路 | 补了：加一行去向：「不给这条出路」那一半已做成，① 安装脚本与 ② 双机路线照旧欠 |
| `records/2026-10-23-subagent拆分提案.md:705` | 四十三节欠账表第 4 行：两次轮询之间起了又跑完的短命令 watch 不报 | 改了：标成已修（脚本文件头 watch 一段与自证格 Ⓩ），同一格别的几样照旧欠；这一句在基准之前就假了 |
| `records/2026-09-16-subagent拆分提案.md:1001` | 两次轮询之间起了又跑完的短命令 watch 不报 | 不改：那份是拆出来之前的历史、不再追加（CLAUDE.md「经验与约定写在哪」） |
| `.claude/agent-common.md:62` | 就尝试其他可用的机器（`bash scripts/hosts/multi-host-run.sh --peer …` | 不改：改共用约束按批走三方，登 `records/2026-10-23-subagent拆分提案.md` 第一百五十三节 ③ |
| `.claude/agent-common.md:70` | 会话里只用 run_in_background 起 `detached-run.sh watch` 上报还在跑的 | 不改：句子本身没变假；watch 报完即退、要再起一只没写，改共用约束按批走三方，登第一百五十三节 ① |
| `.claude/main-agent.md:72` | 会话里只用 run_in_background 起 `bash scripts/hosts/detached-run.sh watch` 上报还在跑的 | 不改：同上，改定义按批走三方，登第一百五十三节 ① |
| `.claude/main-agent.md:52` | 打过的与没打过的补丁分两条命令、依次跑 | 不改：句子没变假；重叠补丁先合成总补丁这一句改定义按批走三方，登第一百五十三节 ② |
| `scripts/hosts/README.md:11` | 把活挪到第二台跑、两台共用的那几段 | 改了：补 `--peer` 拒 `verdict-library-scripts.tsv` 登记的带判定库的门禁与入口、`--verdict-library-route` 只答带不带判定库 |
| `scripts/gate/lib_function_units.py:77` | 今天三条，都在内存包装里 | 不改：主工作区那一份已写「今天四条」，结束提交是三方合成取了旧文；`REAL_CUT_CELL` 是自证格名，改字要连自证期望一起改，不归这一阶段 |

## 原始证据

候选表副本里引调度记录的几句，照门禁 doc-text 的 session-names 格把会话名换成了按角色的说法（行内替换，行数不变）；未改的原件在 `~/.local/share/singlefs/work/cpr-sync-facts-sweep/candidates.tsv`。

| 材料 | 路径 | 行数 |
|---|---|---|
| 事实表 | research/prompts/crash-slice-false-resume-facts.tsv | 25 |
| 候选表 | research/prompts/crash-slice-false-resume-candidates.tsv | 641 |
| 变更清单 | research/prompts/crash-slice-false-resume-changes.md | 9 |
| 事实表回扫员报告 | research/prompts/crash-slice-false-resume-facts-report.md | 68 |
| 逐行判定报告（第 1 段） | research/prompts/crash-slice-false-resume-judge-1.md | 239 |
| 逐行判定报告（第 2 段） | research/prompts/crash-slice-false-resume-judge-2.md | 208 |
| 逐行判定报告（第 3 段） | research/prompts/crash-slice-false-resume-judge-3.md | 237 |
| 逐行判定报告（第 4 段） | research/prompts/crash-slice-false-resume-judge-4.md | 182 |
