# 账本清欠这一轮的治理文件改动：治理文件轮第一轮（ledger-topdown-governance-r1，2026-11-18）

这一轮攻的是调度记录 `records/2026-11-17-subagent账本清欠调度记录.md` 记的这一轮里，主 agent 与它派的工具员、写回员、执行员改过的治理文件：定义与共用约束、钩子、门禁与样本、`scripts/` 下的脚本、统一任务服务。一共 52 份（47 份此刻在工作区里没提交，5 份已随 2026-11-17 的提交进了 HEAD）。方案按 `crates/` 与这些治理文件今天的实现来谈。

## 问题

这 52 份改动各自关的是现行账本 `records/2026-11-17-subagent账本自上而下清欠.md` 的某一节（表里「账本节」一列）。要判两件事：

1. 每一处改动做的是不是它关的那一节说的事，有没有多做、少做、做偏（正推腿）。
2. 哪一处会错：误拒、漏放、自证罩不住、几份文件之间说法对不上、留了指向封存账本的指针、弄坏开关不红在点名的格（攻方腿）。

## 实现今天的样子

主 agent 的观测：

- 这一轮没有为治理文件改 `crates/` 下的代码；`crates/` 那一侧的改动走的是另一题的代码轮（`research/prompts/judging-closure-and-real-card-instance-code-r1-main-verification.md`）。与这一轮有交点的只有一处：`.claude/gate.d/crash-point-replay.sh` 文件头第 314–318 行的注释说的是 `crates/singlefs-verification/src/judging_units_entries.rs:1178` 那一批「不沿它收」的登记。
- 被判的文件与每一处改在哪一行、依据、验证过的任务号与结局原句，逐份列在「范围」一节的表里。表由事务员对着调度记录、各草稿目录的报告与 `git diff HEAD` 做成（`~/.local/share/singlefs/work/ledger-topdown-governance-round-facts/result.md`），对账：工作区里这几类路径 92 条，47 条归这一轮、45 条不归（别的会话的，或没有任何记录认领），对不上 0 份。
- 13 份是混改：同一份文件里还有别的会话没提交的改动（表里「此刻 diff」一列标「混」）。这一轮只判表里「改了什么」一列点到的那几处，别的会话的改动不判。
- 表里「依据」一列的「调度记录:N」是调度记录第 N 行；`<目录>/result.md:N` 的目录在 `~/.local/share/singlefs/work/` 下，省去 `ledger-topdown-` 前缀。「验证」一列是调度记录里的原句，任务号是当时量的。
- 不在这一轮里的：`task-server/README.en.md`、`task-server/README.ja.md`、`task-server/task` 三份说明里也多了 `gpu-cases-not-run` 与种类数，调度记录与各份报告都没有把它们记成这一轮的改动，没放进范围；`prune-server/src/` 与 `scripts/crash/` 下的改动是别的会话的。

## 范围

列：路径｜这一轮改了什么（此刻行号）｜依据｜现行账本哪一节｜验证｜此刻 diff（增 / 删）。

### 一、定义、共用约束与主 agent 流程

| 路径 | 改了什么 | 依据 | 账本节 | 验证 | 此刻 diff |
|---|---|---|---|---|---|
| `.claude/agent-common.md` | 第 44 行私有值那句加「仓根四份被忽略的私有清场脚本 `multi-host-quiesce-*.sh`」；第 100 行写闸第十一道加「两个服务 `src/` 下已存在的文件」与规格生成工具出路。（第 4 行指现行账本、第 83 行 ⑧ 加 `perl -i`、第 123 行报告括注已在 HEAD 里，c57f4ddce，不在 diff） | 调度记录:232（第 44 行）；`swap-followups/result.md:13-33`「改过的文件（主工作区）」表（第 100 行）；调度记录:55、65、83（在 HEAD 的三处） | 第 44 行没写；第 100 行二百三十六 ⑥（`swap-followups/result.md:5-11`） | 没有 | +2 -2 |
| `.claude/agents/implementation-writer.md` | 第 32 行 1b 步写明 clippy、build 点名 `-p <包>`；第 44、48 行证红命令 `--cwd` 从副本改成仓根 | 调度记录:232 | 二百五十一 ①②（调度记录:232 写「账本第二百五十一节 ①②」） | 没有 | +3 -3 |
| `.claude/agents/tooling-writer.md` | 第 31 行第 3 步规格改用 `make-swap-spec.py` 生成、两个服务 `src/` 走仓副本与补丁；第 35 行第 7 步写开关证红只写一条 `break-switch-replay --item`、键算不出的脚本登不进开关复跑表。（第 15、20 行等前半段改动在 HEAD） | 调度记录:91、111（第 35 行）；`swap-followups/result.md:13-33`（第 31 行）；`whole-script-swap/result.md:108`「改法三」 | 二百三十一（`whole-script-swap/result.md:1`）、二百三十六 ⑥ | 调度记录:91「任务 19056 退 0（第 2 回），「复跑了 108 条开关（1 份）…按条复用：现判 109、复用 3、拒算 0」」 | +2 -2 |
| `.claude/main-agent.md` | 第 99 行派发表工具员那一行，换上改写成经 `scripts/gate/hook-swap.py` 一次换上（脚本与钩子都算）、两个服务 `src/` 打补丁 | 调度记录:232 | 没写 | 没有 | +1 -1 |

### 二、钩子（`.claude/hooks/`）

| 路径 | 改了什么 | 依据 | 账本节 | 验证 | 此刻 diff |
|---|---|---|---|---|---|
| `.claude/hooks/write-guard.sh` | 第十一道罩两个服务 `src/`（第 693、694 行 `SWAP_ONLY_EXCLUDED` 与 `SWAP_ONLY_SOURCE_DIRECTORIES`）、第 696 行 `MAKE_SWAP_SPEC_SCRIPT`、第 706 行 `decide_script_swap_only` 的源码分支、拒绝说明点名规格工具；第 88 行指现行账本 | 调度记录:131、135、136、242；`swap-followups/result.md:5-11`；`sweep-apply-tooling/result.md:33-40` | 二百三十六 ⑥；账本指针一处没写 | 调度记录:136「写闸的演练 19203、真换 19204、换上后自证 19205「查了 341 种情形」」；调度记录:242「写闸 19712…写闸开关复跑 19717「复跑了 2 条开关…逐格相同 2 条」」 | +60 -17 |
| `.claude/hooks/bash-command-detector.sh` | 文件头 509、559、790、795、796、799、801、805、807、901 行十处「（照判法推的格（还没复跑））」改成现状；第 7500 行拒绝说明里账本路径指现行账本。（⑧ 认 `perl -i` 与 python 脚本正文那一版在 HEAD，调度记录:80 换上） | 调度记录:238、242；`sweep-apply-tooling/result.md:60-72`「### 2」 | 二百二十二（尾巴，调度记录:98）；账本指针没写 | 调度记录:242「检出钩子自证 19709」等「都退 0」；调度记录:81「检出钩子自证 19007「自检通过（查了 1188 种）」」 | +11 -11 |
| `.claude/hooks/lib_project_code_runs.py` | 第 1103、1105 行账本路径指现行账本；第 1107 行 `TEXT_TOOLS_RUN_DIRECTLY` 加 `scripts/gate/make-swap-spec.py`。（`active-sessions.py` 登 `MAIN_AGENT_TOOLS_RUN_DIRECTLY`、第 1113 行 `ACTIVE_SESSIONS_SCRIPT` 在 HEAD）。混：第 187、1127-1129、3551 行 `gpu-verification-server/control serves` 是别的会话的 | 调度记录:95、103、238、242；`sweep-apply-tooling/result.md:114-130` | 二百二十二（调度记录:95）、二百三十六尾巴（调度记录:242） | 调度记录:242「`lib_project_code_runs.py` 自证 19721 与开关复跑 19722「复跑了 72 条开关…逐格相同 72 条」」 | +9 -5（混） |
| `.claude/hooks/project-code-guard.sh` | 第 206、1717 行账本路径；第 196-198 行 `TEXT_TOOLS_RUN_DIRECTLY` 名单抄写换成指向名单本身；第 506-509、2724-2726、3033-3036 行 perl 单独写 `-i`/`-l`/`-0` 与 `active-sessions-unlisted` 的开关说明和自证格。混：第 255-258、1756-1790、1875-1900 行 `copy-without-full-gate-flag`、`copy-test-names-dropped` 与第 1745 行模板是别的会话的（调度记录:110） | 调度记录:95、103、108、110；`active-sessions-registration/result.md:6-31`；`sweep-apply-tooling/result.md:114-130` | 二百二十二；二百二十九 尾巴（别的会话，调度记录:110） | 调度记录:242「项目代码闸自证 19716「查了 1473 种，其中该拒 332 种」」；调度记录:108、110「`project-code-guard.sh` 不设开关的基准趟就没过…这之前…复跑过不了基准趟是已知的」（开关复跑没判） | +139 -40（混） |
| `.claude/hooks/handback-guard.sh` | 第 43 行账本路径指现行账本 | 调度记录:238；`sweep-apply-tooling/result.md:60-72` | 没写 | 调度记录:242「交回闸 19711」「都退 0」 | +1 -1 |
| `.claude/hooks/lib_run_scope.py` | 第 714 行账本路径 | 调度记录:238 | 没写 | 没有（调度记录:242 的换上后自证里「范围声明闸 19713」「都退 0」） | +1 -1 |
| `.claude/hooks/run-scope-guard.sh` | 第 31、70 行账本路径 | 调度记录:238 | 没写 | 调度记录:242「范围声明闸 19713」「都退 0」 | +2 -2 |
| `.claude/hooks/runner-dispatch-guard.sh` | 第 183、193、2492、2839 行账本路径（2839 是拒绝说明串） | 调度记录:238；`sweep-apply-tooling/result.md:60-72` | 没写 | 调度记录:242「派发闸 19715」「都退 0」 | +4 -4 |
| `.claude/hooks/selftest-registry.tsv` | 第 103 行加 `e56-vm-bench-run.sh`、第 130 行加 `scripts/gate/make-swap-spec.py`（`active-sessions.py` 第 23 行在 HEAD）。混：第 52 行 `crash-library-totals.sh` 是别的会话的（`swap-followups/result.md:19`）；第 24 行 `agent-watch.py` 那一行差别在行内 | 调度记录:122；`swap-followups/result.md:13-33`；`e56-vm-bench-entry/result.md:9-21` | 一百七十六 ①、二百三十六 ④ | 调度记录:145「开关复跑格点它 19222 退 0：「复跑了 12 条开关（1 份）…拒算 0」」（e56） | +4 -1（混） |

### 三、门禁（`.claude/gate.d/`）与它们的样本

| 路径 | 改了什么 | 依据 | 账本节 | 验证 | 此刻 diff |
|---|---|---|---|---|---|
| `.claude/gate.d/code-tooling.sh` | 第 2205 行 runner 表加 `make-swap-spec.py --selftest`、第 2228 行加 `e56-vm-bench-run.sh --selftest`、第 9595 行账本路径。混：开关复跑格基准趟与里层线程数、样本清单等 130-706 行一带大段是别的会话的（调度记录:256） | 调度记录:242；`sweep-apply-tooling/result.md:114-130`；`sweep-apply-tooling/result.md:60-72` | 二百三十六尾巴；账本指针没写 | 调度记录:242「runner 表新加两份的自证 19719」「都退 0」 | +136 -39（混） |
| `.claude/gate.d/break-switch-replay-files.tsv` | 第 32 行 `comment-history-check.py` 改「十八条」；第 57 行 E201 `full_table_run.py` 改「十条」；第 61 行写闸；第 62 行 `replay.sh` 改「六十条」（任务 19775）；第 69 行 `lib_project_code_runs.py` 改「七十二条」；第 89 行撤掉 `hook-swap.py` 那一行；第 91 行加 `make-swap-spec.py`；第 89 行另有 `e56-vm-bench-run.sh`。混：`crash-amplification-lib.sh`、`crash-point-replay.sh`、`agent-watch.py`、`project-code-guard.sh`、`task_server_lib.py`、`merge-and-verify.sh`、`crash-library-totals.sh` 等行是别的会话的 | 调度记录:105、119、145、242、250、257；`replay-switch-naming/result.md:33-35`；`sweep-apply-tooling/result.md:100-113` | 二百三十六尾巴；一百七十六 ①（e56 行） | 调度记录:257「开关复跑 19775 退 0「复跑了 60 条开关（1 份），自证红且点名的格与实际红的格逐格相同 60 条…拒算 0」」；调度记录:250「规格工具的开关复跑 19727「复跑了 5 条开关…逐格相同 5 条…拒算 0」」 | +12 -10（混） |
| `.claude/gate.d/comment-history-lag.tsv` | 第 9 行 `doc-process-records.sh` 39 改 38；删 `list_check.py`、`e185_r4_baseline_rejudge.py`、`e203-admission-scan-cost-run.sh` 三行。混：`prune-server/` 一带七行是别的会话的 | 调度记录:133、250；`comment-history-apparatus/result.md:53-66`；`sweep-apply-tooling/result.md:41-48` | 二百零一 ③（调度记录:126）；二百三十七（调度记录:115） | 调度记录:250「注释历史那一格 19726 退 1，红的是别人的 4 行…我改的 doc-process-records.sh 那一行不在红的里面」 | +5 -9（混） |
| `.claude/gate.d/doc-text.sh` | 第 646、656-658、741、760 行文件头开关表加 `prefix-compounds-flagged` 与两份新样本名；第 972、976 行样本说明 | 调度记录:119、121；`comment-history-words/result.md:1-12` | 二百三十七 第 7 件（调度记录:115） | 调度记录:121「样本档 19156「点名的 1 道阶段判得都对（103 个用例）」」「样本档加开关 19157 判错的恰好是两份新样本」 | +8 -3 |
| `.claude/gate.d/crash-point-replay.sh` | 第 314-318 行文件头那句改成现状：行首 `GPU_LIFECYCLE` 的行由登记成不沿它收的三个单元打、改它们不换键；行首 `CRASH_JUDGE_ACCOUNTING_SLICE` 的行由判器入口打、改它的字仍换键。混：第 387-390、769-770、1196-1209 行是别的会话的 | 调度记录:263、266；`code-r1-crash-point-replay-comment/result.md:1-22` | 一百九十四（调度记录:188、263「门禁注释与判器登记矛盾」） | 调度记录:266「演练任务 19849 退 0，真换任务 19850 退 0「换上了 1 份（改名换上新 inode、回读一致）」…样本档任务 19851 退 0「点名的 1 道阶段判得都对（103 个用例）：crash-point-replay」」 | +25 -2（混） |
| `.claude/gate.d/doc-process-records.sh` | 第 457-458 行「封存几份、现行哪份」那句改成现状（去掉用户定的日期引文） | 调度记录:242；`sweep-apply-tooling/result.md:41-48`「### 3」 | 没写（回扫） | 调度记录:242「门禁 doc-process-records 的样本档 19723」「都退 0」 | +2 -2 |
| `.claude/gate.d/stage-owners.tsv` | 第 139 行 `ledger-section-numbers` 那行「今天是」后的路径改指现行账本 | `sweep-apply-tooling/result.md:189`「主工作区里直接改的…`stage-owners.tsv`（第 139 行）」 | 没写（回扫） | 没有 | +1 -1 |
| `.claude/gate.d/replay-comparison-units.tsv` | 删第 18、19 行 e183-r1 两条「不登记」的豁免注释；第 111 行加 E201 比对对象（驱动 `driver_e201_mutations`、产物名带 `-2026-11-18`） | 调度记录:218、243；`e201-rerun-runner-third/result.md:1-3` | 二百三十七（调度记录:204 收拢表） | 调度记录:243「文档门禁七格 19705 红一格 replay-comparison-units（两处点已删的 `research/prompts/e183-r1-prereg.md`…我删了文件头那两行豁免）」（删了之后没记复跑） | +1 -2 |
| `.claude/gate.d/fixtures/doc-text.sh/comment-history-past-red/`（未跟踪，3 个文件：`expect`、`scripts/…`） | 新样本：`.sh` 注释一行「键要以前缀起头」不算、下一行真讲过去的句子判红，只点名那一行 | 调度记录:119、121；`comment-history-words/result.md:52-56`「样本（两份，已写进主工作区）」 | 二百三十七 第 7 件（调度记录:115） | 调度记录:121「样本档加开关 19157 判错的恰好是两份新样本」 | 不在 diff 里（未跟踪，`git diff HEAD` 不含；`git status -uall` 数 3 个文件） |
| `.claude/gate.d/fixtures/doc-text.sh/comment-history-prefix-green/`（未跟踪，3 个文件：`expect`、`crates/…`） | 新样本：`.rs` 注释写「以前缀」「除以前者」「所以前面」「除以前一项」，判绿、命中 0 行 | 同上 | 同上 | 调度记录:121「样本档 19156「点名的 1 道阶段判得都对（103 个用例）」」 | 不在 diff 里（未跟踪；3 个文件） |

### 四、脚本：门禁与工具（`scripts/gate/`、`scripts/agents/`、`scripts/kb/`、`scripts/mutation/`）

| 路径 | 改了什么 | 依据 | 账本节 | 验证 | 此刻 diff |
|---|---|---|---|---|---|
| `scripts/gate/hook-swap.py` | 第 198 行 `SCRIPT_SPEC_DIRECTORIES` 脚本规格扩到六处目录；第 215 行起退出码 8（中性喂法先喂换上之前那一版，第 602 行 `neutral_feeds_failing_on_base`）；第 216、1728、2758 行起退出码 9（上一趟还在跑）；第 796、856 行 `keyed_stores`、`container_entry` 按字符串键认调用方引到的库；第 26 行文件头指规格工具。（前半段脚本规格那一版在 HEAD） | 调度记录:61、67、92、99、131、136、146、151、154、217；`whole-script-swap/result.md:43-64`；`neutral-feeds-first-fix/result.md:3-31`；`swap-followups/result.md:73-`；`swap-followups-fix/result.md:6-13`；`swap-caller-names-fix/result.md:5-16` | 二百三十一；二百三十六 ①④⑤⑥（调度记录:99、154） | 调度记录:99「演练 19085 退 0；真换 19086 退 0…换上之后自证 19091「自检通过（查了 57 种）」…开关 `neutral-base-skipped` 的 19093 退 1，红的恰好是点名的两格」；调度记录:217「演练 19588、真换 19590 退 0…只核不换 19591 退 0「82 份调用方文件里的 80 处引用，缺 0 处，本次未判 5 处」」；调度记录:217「补完没再复跑这一条」（文件头补三格那一次） | +383 -44 |
| `scripts/gate/make-swap-spec.py`（未跟踪） | 新建（431 行，有执行位）：从基线与新版出规格，正着倒着都核；之后改过的修正版并进 `hook-swap.py` 那份规格换上 | 调度记录:131、151、242、250；`swap-followups/result.md:5-11` | 二百三十六 ④ | 调度记录:151「规格工具自证 19242「6 格都过」」；调度记录:250「规格工具的开关复跑 19727「复跑了 5 条开关…逐格相同 5 条…拒算 0」」 | 不在 diff 里（未跟踪，`git diff HEAD` 不含；431 行，`wc -l`） |
| `scripts/gate/comment-history-check.py` | 第 158-168 行 `BEFORE_PATTERN`：「以前」前紧挨「除」、后紧跟「缀」「者」「面」不算；第 73、140、419、1031 行 `prefix-compounds-flagged` 开关与自证格 | 调度记录:119、121；`comment-history-words/result.md:3-12`「开头：多改了哪几份」 | 二百三十七 第 7 件 | 调度记录:119「演练 19150、真换 19151（验证命令是那份判法脚本的自证，退 0）」；调度记录:121「开关复跑格点那份判法脚本 19160 退 0」 | +25 -8 |
| `scripts/gate/README.md` | 第 20 行「钩子与脚本先验后换」；第 21 行加 `make-swap-spec.py` 一行与它登在直接起的名单里 | `sweep-apply-tooling/result.md:189-200`；`swap-followups/result.md:13-33` | 二百三十六 ④（尾巴） | 没有 | +2 -1 |
| `scripts/agents/agent-watch.py` | 第 591 行 `UNJUDGED_DETECTION_PREFIXES` 加「时刻写法闸没判、照常放行」。混：695 行里其余（定时回报「知识同步提醒」、`repository`、`knowledge` 起头的开关与自证）是别的会话的，调度记录与报告都没提 | 调度记录:93、96 | 二百三十六 ③（调度记录:98「新检出叫醒看门狗」） | 调度记录:96「看门狗前缀表补一项的换上 19070 退 0（验证命令是看门狗自证，结局文件写了完成）」 | +695 -3（混，绝大部分不归这一轮） |
| `scripts/agents/dispatch-prompt.py` | 第 300 行账本路径指现行账本 | 调度记录:238；`sweep-apply-tooling/result.md:60-72` | 没写 | 调度记录:242「四份规格演练与真换都退 0（真换 19692、19697、19699、19702）」 | +1 -1 |
| `scripts/kb/stale-candidates-record-carriers.tsv` | 第 3 行记录载体登记表从封存的旧账本换成现行账本 | 调度记录:219 | 没写 | 没有 | +1 -1 |
| `scripts/mutation/tables/e201_three_oracle_collusion_full_table_on_the_current_tree.tsv` | 第 65 行加注释、第 66 行第 57 行变异换成「判法 E171 目录不进 import 路径」 | 调度记录:227；`e201-rerun-runner-second/result.md:37-47` | 二百三十七（调度记录:204） | 调度记录:237「正式整表重跑 19632 退 0：产物 118 行，「E7RESULT name=verdict mutations=57 caught=57 invalid=0 survived=0 timed_out=0 restored_and_baseline_green=true」」 | +2 -1 |

### 五、脚本：实验装置（`scripts/experiment-apparatus/`）

| 路径 | 改了什么 | 依据 | 账本节 | 验证 | 此刻 diff |
|---|---|---|---|---|---|
| `scripts/experiment-apparatus/replay.sh` | 四处调用写全路径、执行器 `python3 -B "$@"`（第 3919 行；第 3994 行一带）；按字节截 30 行改按字符截、并行度按这一台能给的内存取（第 516-524、716-719、617 行 `memory-by-task-cap`、`dispatch-task-cap-not-binding`）；内联 python 整段删掉、`replay_row_costs.py read` 报 `host-cap`（第 120、718 行 `host-cap-unreported`）；`OUT_DIR` 拆两步带守卫；第 2928、4044 行加 E201 驱动 `driver_e201_mutations`；第 722 行 `threads-bound-kept` 的样本仓取法设成只读函数。混：调度记录:73、160 记着别的会话的工具员改过 `replay.sh` 的 `reuse-gpu-only-for-feature-rows` 一格 | 调度记录:70、123、158、161、167、218、227、243、255、257；`replay-closure-fix/result.md:11-27`；`byte-cuts-and-full-cores/result.md:35-70`；`replay-inline-removal/result.md:35-79`；`replay-switch-naming/result.md:5-22` | 二百一十四、二百一十八、二百一十九（调度记录:70、115）；E201 二百三十七；`OUT_DIR`、内联 python、`threads-bound-kept` 没写 | 调度记录:257「`replay.sh` 两份规格不带验证连着真换 19770、19771（「✓ 换上了 1 份…」…）；自证 19773「replay.sh 自检通过（查了 111 项…」；开关复跑 19775 退 0「复跑了 60 条开关（1 份），自证红且点名的格与实际红的格逐格相同 60 条…拒算 0」」 | +140 -53（混） |
| `scripts/experiment-apparatus/replay_row_costs.py` | 第 4-8、20-24 行文件头与 `read` 多打一行 `host-cap`；第 140 行 `host_cap_reading`；第 237 行 `--no-host-cap` | 调度记录:167；`replay-inline-removal/result.md:35-59` | 没写（调度记录:161、167） | 调度记录:167「演练 19443、真换 19444（带 `replay.sh` 自证当验证）都退 0，换上后 `replay.sh` sha256 c417e2d6、`replay_row_costs.py` fd2db0e5」 | +25 -7 |
| `scripts/experiment-apparatus/e184-segment-one-long-run.sh` | 第 534、538 行 `cut -c1-400` 改 `clip_text 400`（按字符截） | 调度记录:123；`byte-cuts-and-full-cores/result.md:35-54` | 二百一十八 | 调度记录:141「e184 自证 19206「54 格全绿」」 | +2 -2 |
| `scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/list_check.py` | 第 6、7、307 行文档串「改之前的名单」改「基准名单」、「改之后」改「现行名单」 | 调度记录:130；`comment-history-apparatus/result.md:9-26` | 二百零一 ③（调度记录:126） | 调度记录:133「装置注释那一串：演练 19192、真换 19193（「换上了 4 份」）…注释历史那一格 19194 退 1，红的只剩做全量提交门禁的会话认下的那五行」 | +3 -3 |
| `scripts/experiment-apparatus/e185_r4_baseline_rejudge.py` | 第 535 行文档串「第二轮判决数」改成「r3 第一段产物…unstable 数」 | 同上 | 二百零一 ③ | 同上 | +1 -1 |
| `scripts/experiment-apparatus/e203-admission-scan-cost-run.sh` | 第 63 行「改前的文件级键」改「整份文件进键」 | 同上 | 二百零一 ③ | 调度记录:133「e203 的自证 19195 退 0」 | +1 -1 |
| `scripts/experiment-apparatus/long-run-lib/binary-identity.sh` | 第 3 行文件头加 `e56-vm-bench-run.sh` 这个调用方 | 调度记录:130；`comment-history-apparatus/result.md:9-26` | 二百零一 ③ | 同上（19192、19193、19194） | +1 -1 |
| `scripts/experiment-apparatus/README.md` | 第 11 行 `replay_row_costs.py` 括号补「这一台能给的内存」；第 12 行加 `e56-vm-bench-run.sh` 一条 | `sweep-apply-tooling/result.md:101`；`e56-vm-bench-entry/result.md:9-21` | 一百七十六 ① | 没有 | +2 -1 |
| `scripts/experiment-apparatus/e56-vm-bench-run.sh`（未跟踪） | 新建（391 行，有执行位）：E56 虚机复跑入口，自证改成逐格写字面名字派活（算键库折得出） | 调度记录:122、134、138、145；`e56-vm-bench-entry/result.md:5-22`；`e56-entry-key-fix/result.md:5-23` | 一百七十六 ①（调度记录:115、138） | 调度记录:145「开关复跑格点它 19222 退 0：「复跑了 12 条开关（1 份）…拒算 0」」；调度记录:138「E56 虚机复跑完成…（32 条结果行，与收尾行声称的 32 条相符）」（任务 19164） | 不在 diff 里（未跟踪，`git diff HEAD` 不含；391 行，`wc -l`） |
| `scripts/experiment-apparatus/e201_three_oracle_collusion_full_table_on_the_current_tree/test_full_table_run.py` | 第 401-404 行加 `is_the_repository_root_itself`（解到仓根自身的不要求被 `SYNCED_ENTRIES` 罩住），第 428-436 行仓根引用那条测试跟着改；第 500-533 行加三条用例（仓根本身放行、仓根下文件照报、扫描结果里保留仓根） | 调度记录:164；`e201-rerun-runner/result.md:1-3`（`e201-rerun-runner/landing-checklist.md`） | 二百三十七（调度记录:204） | 调度记录:164「改后六个模块 19424「Ran 124 tests」「OK」；三份手造变异 19427、19428、19429 各退 1」 | +62 -3 |
| `scripts/experiment-apparatus/e201_three_oracle_collusion_full_table_on_the_current_tree/current_tree_row_driver.py` | 第 60-63 行注释写成今天的样子（两条路都让四份解得出），代码不动 | 调度记录:218；`e201-rerun-runner-second/result.md:47` | 二百三十七 | 调度记录:234「改后单测 19631「Ran 125 tests」「OK」」 | +4 -3 |
| `scripts/experiment-apparatus/e201_three_oracle_collusion_full_table_on_the_current_tree/test_current_tree_row_driver.py` | 第 248-273 行 `JudgeImportTests` 加第 57 行等价的留档用例（第 251 行） | 调度记录:218、227；`e201-rerun-runner-second/result.md:46` | 二百三十七 | 调度记录:225「弄坏开关下 19582 退 1、「FAILED (failures=1)」，红的恰是留档的那条等价用例」 | +26 -0 |

### 六、任务服务（`task-server/`）

| 路径 | 改了什么 | 依据 | 账本节 | 验证 | 此刻 diff |
|---|---|---|---|---|---|
| `task-server/task_server.py` | 日志里行首 `NOT_RUN_NO_GPU_ADAPTER` 的行：第 254、891、1116 行 `gpu_not_run_test`、第 1137 行 `task_scope_text`、第 4440 行 `gpu_cases_not_run_items`、第 5535-5599 行读日志、第 6025 行 `record_gpu_not_run_marks`。（任务服务欠账那一批 `task-server-debts` 在 HEAD，3b4259d70）。混：换钩子退 9 的 `previous-swap-running`、提交时只查不跑的写法等是别的会话的 | 调度记录:147、160；`not-run-marker/result.md:12-38` | 二百零五（调度记录:126、172） | 调度记录:160「演练 19230、真换 19231、自证 19233「405 格都对」、开关 19234 红 1 格 `gpu-cases-not-run-reported`」 | +123 -12（混） |
| `task-server/task_server_lib.py` | 第 1507-1508 行开关 `gpu-cases-not-run-unread`、第 1738、1765、1798 行 `gpu-cases-not-run` 进提醒种类与异常表、第 2128 行 `KIND_NAMES`、第 2303 行 `KNOWN_BREAK_ITEMS`。混：`previous-swap-running`、`entry-script-absolute-path-kept`、`commit-gate-query-forms-judged`、`repository_script_word`、`query_only_entry` 是别的会话的 | 调度记录:147、160；`not-run-marker/result.md:22-38` | 二百零五 | 同上；调度记录:172「同一条不带 gpu 特性的点名测试，带 `--nocapture` 的 19471 服务发了 gpu-cases-not-run 提醒，不带的 19467 没有」 | +58 -18（混） |
| `task-server/task_server_selftest.py` | 第 9638-9660 行新格 `gpu-cases-not-run-reported`（`cell_gpu_cases_not_run`）、第 14695 行登记。混：别的会话的格在其余 140 行里 | 调度记录:147；`not-run-marker/result.md:39-53` | 二百零五 | 调度记录:160「自证 19233「405 格都对」」 | +160 -6（混） |
| `task-server/README.md` | 第 149 行提醒种类数与 `gpu-cases-not-run` 的现状。混：第 31、166、190 行是别的会话的（种类数「十七」含它们的） | 调度记录:147；`not-run-marker/result.md:35` | 二百零五 | 没有 | +4 -4（混） |

### 七、这一轮改过、已随 2026-11-17 的提交进了 HEAD 的

HEAD 已带走（2026-11-17 的提交），或此刻与 HEAD 逐字相同。「此刻 diff」一律写不在 diff 里。

| 路径 | 改了什么 | 依据 | 账本节 | 验证 | 此刻 diff |
|---|---|---|---|---|---|
| `.claude/hooks/lib_shell_words.py` | 第 275 行起开关 `perl-bare-options-take-next-word`：perl 单独写的 `-i`、`-l`、`-0` 不再吃下一个词 | 调度记录:95；`active-sessions-registration/result.md:62-71` | 二百二十二（调度记录:95） | 没有（这条开关的点名格写在 `project-code-guard.sh` 文件头第 506-507 行，那份文件的开关复跑没判：调度记录:110「这一轮给它换上的两条新开关（`active-sessions-unlisted`、`perl-bare-options-take-next-word`）与补点名的四条算没判」） | 不在 diff 里：已被别的提交带走（`git log -1 -- .claude/hooks/lib_shell_words.py` 是 001977a00，2026-11-17）；`git show HEAD:` 里 `perl-bare-options-take-next-word` 命中 3 处 |
| `scripts/agents/active-sessions.py` | 新建查活动会话的只读工具（530 行）与自证 17 格 | 调度记录:76、77、81；`detector-and-sessions/result.md:87-117` | 二百二十二 | 调度记录:81「查活动会话工具自证 18992「自检 17 格都过」，六条开关 18993–18998 点名 7、3、3、2、1、2 格，逐格相同」 | 不在 diff 里：已被别的提交带走（f09a3c464，2026-11-17） |
| `scripts/agents/README.md` | 表里加 `active-sessions.py` 一行（第 16 行） | `detector-and-sessions/result.md:6-28` | 二百二十二 | 没有 | 不在 diff 里：已被别的提交带走（f09a3c464） |
| `task-server/control` | `task-server-debts` 那一批改了第二百二十四节两处（规格 2 处） | 调度记录:94、102；`task-server-debts/result.md:30-40` | 二百二十三、二百二十四（调度记录:102） | 调度记录:102「演练 19080、真换 19081、整份自证 19084「403 格都对」、重启后 `control check` 退 0」 | 不在 diff 里：已被别的提交带走（3b4259d70，2026-11-17） |
| `.claude/gate.d/split-proposal-ledgers.tsv` | current 改指新账本、旧账本改 sealed | 调度记录:65 | 换账本（调度记录:64、65） | 没有 | 不在 diff 里：已被别的提交带走（ff07ef7c8，2026-11-17） |

## 要腿判的

逐份读表里点到的行（混改的只读点到的那几处），回「成立」「不成立」或「推不出」，推不出的算打中。重点问这十件：

1. 写闸第十一道（`.claude/hooks/write-guard.sh:693-706`）罩到两个服务的 `src/`：已存在的源文件只许经补丁换，这道闸会不会误拒该放的写法（新建文件、草稿目录里的副本、`src/` 之外的服务文件），会不会漏放（别的写文件的写法绕得过）。它的拒绝说明给的出路（规格工具、打补丁）本身过不过别的闸。
2. 换上工具 `scripts/gate/hook-swap.py`：退 8（中性喂法先喂换上之前那一版）、退 9（上一趟还在跑）、按字符串键认调用方引到的库（第 796、856 行）三处，各自的判据有没有把该换的挡住、把不该换的放过去；「补完没再复跑」的那一次文件头补点名三格，点名与实际红的格对不对得上。
3. 规格工具 `scripts/gate/make-swap-spec.py` 登进了直接起的名单（`.claude/hooks/lib_project_code_runs.py:1107`）：它会写文件（规格），登成「文本工具直接起」之后，子 agent 能不能借它往不该写的地方写。
4. 任务服务认「本次未跑」的记号行（`task-server/task_server.py:5535-5599`、`:6025`，`task_server_lib.py` 的 `gpu-cases-not-run`）：只认日志里行首的记号行。测试不带 `--nocapture` 时这一行被测试框架收走、服务读不到（调度记录:172 量到）；这个缺口在文档与提醒里写没写明，会不会让人把「没提醒」读成「都跑了」。
5. 复跑入口 `scripts/experiment-apparatus/replay.sh`：并行度按这一台能给的内存取（第 516–524、716–719 行）、`replay_row_costs.py read` 报 `host-cap`、样本仓的取法设成只读函数（第 722 行）三处。两台的内存上限不同时判定结局会不会不同；只读函数挡住的那种「被共用库读进的取法盖掉」在别的派行格的样本里还有没有。
6. 注释历史判法 `scripts/gate/comment-history-check.py:158-168`：「以前」前紧挨「除」、后紧跟「缀」「者」「面」不算。这四个例外会不会放过真讲过去的句子（例「所以前面那一版」）。
7. 三份定义与共用约束之间说法对不对得上：`.claude/agents/tooling-writer.md:31,35`、`.claude/main-agent.md:99`、`.claude/agent-common.md:44,100`、`.claude/agents/implementation-writer.md:32,44,48`，与它们说的钩子和脚本今天的行为（写闸、换上工具、`scripts/mutation/prove-red.sh` 要从哪里起）是不是一回事；有没有哪一句说的做法照做会被别的闸拒。
8. 账本指针：这一轮把一批文件里指向封存账本的路径换成了现行账本。全仓这几类路径里还有没有漏换的；换成现行账本之后那一句还是不是真话（那一句说的事在现行账本里找不找得到）。
9. 自证与弄坏开关：这一轮每一处新加的会判红的分支，有没有开关或样本，开关在文件头点名的格与登记表 `.claude/gate.d/break-switch-replay-files.tsv` 里的条数对不对得上；表里「验证」一列写「没有」的那几份（定义、`stage-owners.tsv`、两份 README、`stale-candidates-record-carriers.tsv`），改动有没有该验而没验的。
10. 门禁注释（`.claude/gate.d/crash-point-replay.sh:314-318`）说的与 `crates/singlefs-verification/src/judging_units_entries.rs` 今天的登记、与这道门禁滤日志行的代码是不是一回事。

## 分工

- 正推腿：第 1 件事。按表逐份核「改动做的是不是账本那一节说的」，账本节读 `records/2026-11-17-subagent账本自上而下清欠.md` 对应的节；表里账本节写「没写」的，判这一处改动有没有依据（调度记录那一行说了为什么改）。另判第 7、8、10 问。
- 攻方腿：第 2 件事。攻第 1 到 6 问与第 9 问；每一个打中给出能照跑的输入或 文件:行。要跑的命令照定义写进报告交主 agent。
- 两条腿都读表里全部 52 份点到的行；只有一条腿读到的范围在报告里写明。
