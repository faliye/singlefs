# 严查第二轮·甲组报告（规则与入口）

范围：`git diff 73ba4a4..1c58cfa` 在 CLAUDE.md、.claude/main-agent.md、.claude/agent-common.md、.claude/rules/*.md、.claude/skills/*/SKILL.md 上的改动与这些文件今天的全文。第一轮已处置的（report-A/B/C、r1-sync、defs-r2 判决）不重报。

## 发现

| # | 文件:行 | 类别 | 原句 | 证据 | 建议改法 |
|---|---|---|---|---|---|
| A1 | .claude/main-agent.md:59 | ③ 照做会走错（高） | 「这一批改了门禁 54 号在 `.claude/gate.d/stage-inputs.tsv` 登记的输入、54 号脚本本身或工具链（`cargo -V`、`rustc -V`；这三样都进全绿标记的键），主 agent 自己在后台跑 … 那三行命令」 | 54 号判的不是「这一批改没改」，是「这批输入的哈希有没有全绿标记」（54-layer0-replay.sh:281 `full_green_marker_path="$full_green_marker_prefix.$layer0_input_hash"`，:286 没有就 ✗）；而 stage-must-run.sh 在 `refs/sop/staged-green` 不存在时一律「要跑」（stage-must-run.sh:56 `还没有过整轮全绿的暂存树`）。现查本仓：`ls "$(git rev-parse --git-common-dir)" \| grep -i layer0` 零命中，`git rev-parse -q --verify refs/sop/staged-green` 不存在（新克隆、别的会话提交过没跑全量的 crates 改动、标记被清掉，都落到这一格）。这时一批只改文档的提交照 main-agent 不跑全量 → 整轮 54 号必红；gate-triage.md:27 又按「这一轮的改动碰没碰 54 号登记路径」归属，一条没碰就判「不是这一轮」，提交卡住而出路不指向跑全量 | 触发条件改成 54 号自己的判据：「这批输入没有全绿标记（54 号出路里报的那一句），或这一批改了…」；给主 agent 一条不重型的现查法（例如让 54 号加一个只算哈希、查标记在不在的轻模式），或写明「新克隆 / 没有 `refs/sop/staged-green` 时第一次提交照跑全量」；gate-triage 的归属同步：标记缺失而这一批没碰输入的，归「分不清」并抄三行命令 |
| A2 | .claude/rules/three-way-inference.md:163 | ③ 照做会走错 | 「云端腿报「没打中」而要拿它支撑结论时，主 agent 用同一份提示再派一条同立场腿，报告带 `-s2` 后缀」 | ① 后缀放哪没说：唯一的历史先例是 `sb-treetable-reverse-opus-s2-output.md`（`git log --all --name-only` 现查，`-s2` 在 `-output` 之前），而本地腿的约定是 `<前缀>-output-s<n>.md`（three-way-local-attack.md:27），「后缀」按字面读是后者；② 只给报告改名，模型目录没说：three-way-attack.md:21「模型目录（形态 `research/prompts/<轮>-opus-model/`）」，同一份提示再派一条，第二条腿写同一个模型目录，会盖掉第一条的模型（three-way-forward.md:21 同形）；`grep -rn -- '-s2' .claude/agents .claude/gate.d .claude/hooks research/scripts` 除 replay.sh:464 一处无关注释外零命中，没有别处定义它 | 写死形态：「报告 `<轮>-<opus\|sonnet>-output-s2.md`、模型目录 `<轮>-<opus\|sonnet>-model-s2/`」（或沿用先例 `-opus-s2-output.md`），同样写进 three-way-attack / forward 定义的「输入」一节 |
| A3 | .claude/rules/mutation-sampling.md:82（同句在 .claude/agents/mutation-triage.md:28），连带 mutation-sampling.md:73–74 | ① 与 59 号行为不符（推的，没量过：本容器 run-with-memory-cap.sh 退 251「slice 的总上限设不上」，没能复现） | 82 行「门禁 59 号照它自己收尾的「计数：」行，那一行没有 `💥`、`⚠️` 两栏：测试进程没跑完的，输出里有编译错误记「无效」、没有记「没红」」；73–74 行「跑起来才发现的编不过由门禁 59 号单独报成「无效」…无效要改的是那一行替换文」 | 59-crates-mutation-replay.sh:344 判「无效」的条件是 `re.search(r"^error(\[E\d+\])?: ", output, re.M) or "could not compile" in output`，对的是 cargo 的整份输出。测试二进制被信号杀掉时 cargo 照例在 stderr 打 `error: test failed, to rerun pass …`（推的：cargo 对任何测试失败都打这一行），它命中 `^error: `；同时 libtest 多线程下只在测试结束时打 `test X ... `，崩溃的那一条没有这一行，:342 的 `ran_pattern` 不中。于是「测试进程没跑完」按 59 号今天的代码落「无效」、报成「替换文写进源码之后编不过」，而规则说「没有编译错误记没红」、并让读者去改替换文 | 先照 `.claude/rules/implementation-workflow.md` 第 3 步造一个 abort 的变异核实；属实则 59 号的「无效」只认编译期信号（`could not compile` 或 `error[E`），`error: test failed` 归「没红／没跑完」单列；规则文字改成与改后的判据一致。门禁代码归丙组，文字这一半在甲组 |
| A4 | .claude/rules/format-evolution.md:70 | ③ 射程写窄，照做会漏 | 「27 号从此在 kb 正文与 `research/`、`crates/` 下的 `.rs` 里替你盯着；门禁脚本（`.sh`、`.py`）与 `research/results/` 下的产物它不扫，那几类照第 1 步手工搜」 | 27-format-constants.sh:56–57 只扫 `.claude/kb/**/*.md`（去掉 `-history.md` 与 `decisions-history/`，:50 再截掉历史版本节），:83 只扫 `research/**/*.rs` 与 `crates/**/*.rs`。不扫的除了脚本与产物，还有 `.claude/rules/*.md`、`CLAUDE.md`、`.claude/agents/*.md`、`records/`、`research/prompts/`、`research/` 下的 `.md`、kb 的历史节与变更史。「那几类」只点了两类，读的人会以为其余都被盯着 | 改成列「扫的」而不是列「不扫的」：「它只扫 kb 正文（不含历史节与变更史）和 `research/`、`crates/` 下的 `.rs`；别处（脚本、产物、规则与 agent 定义、`records/`、`research/prompts/`）改完照第 1 步手工搜」 |
| A5 | .claude/skills/crash-test/SKILL.md:12 | ② 与定义、钩子不一致（低） | 「子 agent 里只有崩溃验证员与门禁分诊员各跑登记给自己的那几道，别的子 agent 不跑」 | `.claude/gate.d/stage-owners.tsv` 把 54-layer0-replay.sh 与 87-replay.sh 登记给 gate-triage；heavy-test-guard.sh:40 写「gate-triage：…（54 / 55 / 57 / 59 在整轮门禁里跑，直接调它们拒…）」，合成输入现跑：gate-triage 直接 `bash .claude/gate.d/54-…` 被拒；gate-triage.md:26「登记给你的其余阶段已在 `gate.sh` 整轮里跑过，不再单跑」。门禁分诊员跑的是 `gate-staged.sh`，不是「登记给自己的那几道」 | 改成「崩溃验证员跑 55、57、59 号，门禁分诊员跑 `research/scripts/gate-staged.sh`（54、87 在它的整轮里）」，与 implementation-workflow.md:55 同一说法 |
| A6 | .claude/skills/gate/SKILL.md:12 | ② 与 implementation-workflow.md 不一致（低） | 「提交时由 `gate-triage` 带 `SINGLEFS_HEAVY_TESTS=commit` 跑 `research/scripts/gate-staged.sh`…，其余时候不跑」 | implementation-workflow.md:53「用户当场要求，或任务确实要跑 … 命令带 `SINGLEFS_HEAVY_TESTS=user-request`」；heavy-test-guard.sh 头「都要带环境变量 SINGLEFS_HEAVY_TESTS=commit 或 =user-request」。skill 只写 commit，读的人会以为用户要求时也不许跑 | 「其余时候不跑」改成「用户要求时带 `=user-request`（主 agent 先弹窗问），其余时候不跑」 |
| A7 | .claude/agent-common.md:46 | ③ 退出码没列全、命令没列全（低） | 「子 agent 跑 `cargo test` / `cargo run` 与直接执行编出来的二进制，一律经内存包装 … 退出码 250 … 251 … 252 … 254 …」 | run-with-memory-cap.sh:24「253 超过限时（RUN_WITH_MEMORY_CAP_TIME_LIMIT）」、:27「2 用法错」；钩子拒绝信息自己教子 agent「要限时设 RUN_WITH_MEMORY_CAP_TIME_LIMIT=<秒>」，照做就会碰上 253，而共用约束对它没有出路。合成输入现跑：implementation-writer 的 `cargo bench -p singlefs-core` 同样被拒（「cargo test / cargo run / cargo bench」），共用约束只写了 test / run | 补「253 是撞了自己设的限时，照实报」「2 是用法错」；命令写成「`cargo test` / `run` / `bench`」 |
| A8 | .claude/rules/implementation-workflow.md:52 | ③ 主语不清（低） | 「**必须跑**：主 agent 在提交流程里后台起（命令带 `SINGLEFS_HEAVY_TESTS=commit`），看门狗盯；整轮门禁由 `gate-triage` …」 | 按 main-agent.md:59，主 agent 自己起的只有层 0 全量（且只在 54 号要全量时），55、57、59 由 crash-verifier 跑；这一格读起来是「重型测试都由主 agent 后台起」，与同表 :55 行「崩溃验证员跑 55、57、59 号」各说一半 | 改成「层 0 全量由主 agent 在提交流程里后台起…；55、57、59 由崩溃验证员、整轮门禁由门禁分诊员跑（见 `.claude/main-agent.md`「暂存之后、提交之前跑门禁」那一行）」 |
| A9 | .claude/main-agent.md:59（同一句在 54-layer0-replay.sh:230 的出路里） | ① 措辞与包装的行为不符（低） | 「那次调用退 0（退出码就是 `--full` 的）」 | 第三行命令的 `$?` 取的是 `run-with-memory-cap.sh` 的退出码：0–249 转的是 `--full` 的，250–254 是包装自己的（run-with-memory-cap.sh:18–26）。判据「退 0」本身没错，括注说宽了：退 250 / 251 / 252 / 254 时读的人会去 54 号的输出里找原因 | 括注改成「退出码是内存包装的：0–249 就是 `--full` 的，250–254 照 `.claude/agent-common.md` 那张表读」 |

什么现象会推翻各条：
- A1：本仓 common-dir 里其实有这批输入的全绿标记，或 54 号在没有 `refs/sop/staged-green` 时不查标记、退 77——现跑 `ls "$(git rev-parse --git-common-dir)"` 与读 54-layer0-replay.sh:85–115 可推翻。
- A2：某份定义或脚本里已经写死云端腿第二次抽样的文件名与模型目录（`grep -rn 's2' .claude/agents` 有命中）。
- A3：在有用户级 systemd 的机器上造一条让测试 `abort()` 的变异跑 59 号，结果落「没红」而不是「无效」（即 cargo 不打 `error: test failed`，或 libtest 在崩溃前打出了 `test X ... `）。
- A4：27 号另有一段扫 `.claude/rules/`、`records/` 或 `research/prompts/`（`grep -n glob .claude/gate.d/27-format-constants.sh` 只有 :56、:83 两处）。
- A5：gate-triage 的写法里确有「单跑 54 / 87 号」这一步且钩子放行 54。
- A6：用户要求时跑 gate.sh 另有禁令写在别处。
- A7：run-with-memory-cap.sh 不会退 253，或子 agent 被禁止设限时变量。
- A8、A9：主 agent 在提交流程里另起 55、57、59；或 run-with-memory-cap.sh 在 --full 退 250–254 时改写退出码。

## 核过没问题的

| 载体 | 核了什么 | 怎么核的 |
|---|---|---|
| main-agent.md:59 的三行命令 | 主 agent 带前缀能跑、不带前缀被拒、crash-verifier 跑被拒；bash-command-detector 前台与后台都放行 | 从 54 号 `print_staged_worktree_full_commands` 抽出三行，合成 PreToolUse JSON 喂 heavy-test-guard.sh（rc 0 / 2 / 2）与 bash-command-detector.sh（rc 0） |
| main-agent.md:59「这三样都进全绿标记的键」 | 登记路径、54 号脚本、`cargo -V` / `rustc -V` | 54-layer0-replay.sh:23–25、:176–196 |
| main-agent.md:59 与 implementation-workflow.md:55「54、55、59、74、87 号只在这一趟里能复用」 | 调 stage-must-run.sh 的阶段 | `grep -ln stage-must-run .claude/gate.d/*.sh`：54、55、59、74、87 调它判复用，47 只跑它的 `--selftest` |
| implementation-workflow.md:33「57 号不在那张表里」 | stage-inputs.tsv 的阶段行 | 表里只有 54、55、59、74、87 |
| implementation-workflow.md:48 重型测试清单 | 与 heavy-test-guard.sh「重型测试，按类」逐类一致，15、74 按轻阶段 | 读文件头；合成输入：implementation-writer 跑 15、74 号放行，gate-triage 带前缀跑 87 号与 gate.sh 放行，主 agent 不带前缀跑 gate.sh、check.sh 被拒 |
| agent-common.md:46 内存包装 | 子 agent 裸跑 `cargo test` 被拒、经包装放行 | 合成输入喂 heavy-test-guard.sh |
| agent-common.md:57「七种写法」 | 与 bash-command-detector.sh:31–135「拒绝七种」① 至 ⑦ 一一对上；并行写法 `{ …; echo $? > 甲.rc; } & … & wait` 不记检出、逐个 `wait "$pid"` 记一条检出 | 读文件头；合成输入（检出记录文件现查行数） |
| agent-common.md:34 ⑦ 与 `replace-once.py` | replace-once.py 改名换 inode | replace-once.py:13、:40、:47 |
| agent-common.md:28 引 kb-discipline「2. 每条带出处与状态」里「所有旧数据都只是参考」 | 那一节确有这一条 | 读 kb-discipline.md |
| main-agent.md:30 `gate-overlap.py --list` | 列得出 10 个钩子与各自事件 | 现跑，退 0，「共 106 份：门禁 96、钩子 10」 |
| main-agent.md:30 看门狗告警指到 agent-watch.py 文件头与 watch.conf | 两处都在、`ask-every-minutes=60`、`watch.sh --processes` 存在 | 读文件 |
| CLAUDE.md:16「`.claude/agents/` 下的定义都开了 `omitClaudeMd`」 | 16 份定义 frontmatter 各 1 处 | awk 逐份数 |
| CLAUDE.md:108 46 条与步 0 现状 | `grep -c '^\| I-.*已实现' .claude/kb/invariants.md` = 46；02-second-txn.md 步 0 有「现状」段 | 现跑 |
| CLAUDE.md:22 两道单跑命令 | `stage-selftest.sh`、`number-name-sync.sh` 在、参数形态与 gate.sh:265–286 一致；72 号有 `.claude/agent-def-review-exempt` 豁免 | 读 gate.sh、72 号文件头 |
| format-evolution.md:26 括注 | 20 号 ⑤ 只在标题有「N 项未定」时判条数（20-kb-shape.sh:213–215），75 号判形态（75-…:226） | 读源码 |
| format-evolution.md:53 | 99 号与 `multipath-registry-lag.tsv` 在 | ls |
| fs-design.md:31、57、86、100、137、153、175、214–216 | D5 已定项 4 表第 8、9 行都是「已撤回」；D21 已定项 17 第 5 条原文一致；D15 已定项 4 / 10「三个 bitmap 各 256 位」；D9、D12、D13、D14、D17 与 C8、C19 简称与登记位逐字一致；示意注释与 D15「incompat 位 0 = 第一条纯 SSD 布局线」不再冲突 | 读决策文件与 checks-owed.md |
| mutation-sampling.md:58–64 | mutate.sh 派活前逐条核锚点退 3（mutate.sh:381–387）；「已还原，基线仍全绿」之后源码被改退 5（:663–667）；收尾「计数：」只有内存撞顶与超时（:647） | 读源码 |
| mutation-sampling.md:73 | 33 号查 `\n` 以外的反斜杠（33-mutation-tables.sh:45、:86、:157、:165） | 读源码 |
| path-moves.md:11、26、30–32、38、55 | 「链接指向」「编号与简称」是 gate.sh 的阶段名（:269、:285）；number-name-sync 只登记 E、D（:30）；term-renames.md、90 号、term-rename-exempt 都在；10 号头写第 4 段 | 读源码 |
| three-way-inference.md:20–28、:135、:194–216 | 各腿定义名与 model（opus / sonnet）；核查员按轮派（:135）与 main-agent.md:55 一致；两个检测器分工（corruption-check.py:40、:55 实词自复读，oov-check.py 拼接）；ask-local.sh 退 5 / 6、默认超时 900 秒（ask-local.sh:11–13、:18） | 读源码与 frontmatter |
| three-way-inference.md:125 | c143-r3-main-checks 已归档（删它的提交 3cff909）；agent-common.md「规则怎么读」一节有「找不到历史实验的数据」那一条 | git log、grep |
| decide / crash-test / gate 三份 SKILL 的「在本项目里」 | 共享 crash-test 正文 :37 确写内存序与虚机由项目自定；74 号在；gate.sh 有 `run_stage` / `run_stage_may_skip` | 读共享 skill 与 gate.sh |
| 轻门禁现跑 | 10、20、58、62、63、90 号退 0；68、72 号对 HEAD 退 77，带 `GATE_BASE=73ba4a4` 都退 0；doc-lint 全仓「检查 488，跳过 0」退 0；rules-lint 按 gate.sh 的项目本地接法（29 份）退 0 | 现跑 |

## 没做什么

- 没跑任何重型测试；A3 的复现试过在草稿目录里起一个 abort 的小 crate，`run-with-memory-cap.sh` 在这个容器里退 251（没有用户级 systemd），没跑成，A3 是推的。
- 没核 `.claude/agents/*.md`、`.claude/hooks/`、`.claude/gate.d/` 自己的改动（归乙组、丙组），只在甲组文件指向它们时读到对应那几行。
- `number-name-sync.sh .` 现跑退 1，红在 `crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs:2783、:2850` 与 `records/2026-09-24-里程碑二收尾调度.md:62、:140`，都不在甲组射程里，没判是不是这个分支带进来的。
- 没核 decisions-history 生成、kb 正文里别处对这几份规则的引用。
