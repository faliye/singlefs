# 工具与治理批的定义与规则改动：第三轮（最终轮）判决（2026-10-13）

<!-- doc-lint:not-numbers T1 T2 T3 T4 T5 T6 P1 P2 P3 W1 A1 Q1 Q2 Q3 Q4 Q5 Q6 K1 K2 K3 K4 K5 K6 K7 K8 K9 K10 K11 G11 H4 H5 U01 U02 U03 U04 U05 U06 U07 U08 W01 W02 W03 W04 W05 W06 W07 W08 W09 W10 W11 W12 -->

正文 `research/prompts/_m3-defs-b3-r3-body.md`；腿：正推 `research/prompts/m3-defs-b3-r3-sonnet-output.md`（sha256 1937f2e6…，模型 `research/prompts/m3-defs-b3-r3-sonnet-model/`）、攻方 `research/prompts/m3-defs-b3-r3-opus-output.md`（sha256 304ddd4f…83b7，模型 `research/prompts/m3-defs-b3-r3-opus-model/`）。前两轮判决 `research/prompts/m3-defs-b3-r1-main-verification.md`、`research/prompts/m3-defs-b3-r2-main-verification.md`。第三轮之后不开第四轮（`.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」）。

被判的定义与规则文件（按路径）：`.claude/main-agent.md`、`.claude/agent-common.md`、`.claude/rules/three-way-inference.md`、`.claude/rules/verification.md`、`.claude/agents/experiment-designer.md`、`.claude/agents/experiment-runner.md`、`.claude/agents/implementation-writer.md`、`.claude/agents/investigator.md`、`.claude/agents/mutation-triage.md`、`.claude/agents/three-way-attack.md`、`.claude/agents/tooling-writer.md`、`.claude/agents/clerk.md`、`.claude/agents/three-way-forward.md`、`.claude/agents/three-way-defense.md`。

## 核查记录（主 agent 做）

- 开工快照：冻结副本 `~/.local/share/singlefs/work/m3-defs-b3-r3/repo/` 对 `research/prompts/m3-defs-b3-r3-snapshot/frozen-copy-sha256.txt` 35 份 `sha256sum -c` 非 OK 0 行。
- `cite-check.py <两份报告> --root <冻结副本> --background <背景材料>`：「核了 54 处引文，5 处对不上，20 处没判」；5 处都是正推腿引主工作区调度记录 `records/2026-10-12-里程碑三工具与治理批.md` 第 73、104、112、115、123 行（冻结副本里那份记录是旧的 72 行）——主 agent 在主工作区逐行 `grep -F` 引文，5 处都命中，按对上记。
- 攻方模型拷进草稿目录经 `run-with-memory-cap.sh 4G` 复跑：八行六 same，`t2_nested-r1`、`t2_nested-r3` 只差拒绝说明按固定字节截断的位置（主 agent 的工作目录路径更长），退出码相同，按相同记。正推模型同样复跑：`count_checks same`。
- W01 主 agent 现跑坐实：集成副本标签 `after-b5-fixup` 上 `SINGLEFS_HEAVY_TESTS=commit GATE_BASE=<基准> bash .claude/gate.d/code-tooling.sh --check heavy-thread-defaults` 退 1，点名 `scripts/agents/dispatch-prompt.py:262  读自己的线程变量 THREAD_LIMIT_REQUIRED_AGENTS`，那一行是 `if "THREAD_LIMIT_REQUIRED_AGENTS" in long_run_table.group(2):`（字符串比较，不是读环境变量）——K5 新判法的误判，批 5 收尾之后主 agent 只复跑了四份自证、没复跑这一格。
- T6 抽核：第一轮正推报告 sha256 现算 `60ed02db…d8a8`（判决一与调度记录写成 `…a8a8`）；第二轮攻方 `out/r5-r2.txt` 里 `p-env-C rc=0`（判决二 R5 写「九种写法…0→2」）；`verify3.summary` 期望 green 16 行里 2 行判错（调度记录写「16 项绿」）；`git diff --shortstat before-b4 after-b4` 是 52 份 +1425/−237（调度记录写 30 份 +1283/−237）。正推腿报的另几处没逐个复算，按它的现算命令记。
- 攻方两条现场观察（工作根下没有 `.writer-lock`；别的会话 merge-and-verify 用 `capped.sh 32`）：主工作区 `.claude/hooks/lib_writer_lock.py` 不在、`scripts/gate/merge-and-verify.sh` 读登记 0 处——批 2 起的补丁还没打进主工作区，预期内。

## 逐格判决

| 格 | 判 | 依据 | 去向 |
|---|---|---|---|
| T1 线程数已定项 | 站得住 | 攻方 A1（`capped.sh 31` 套在 `host-dispatch.py` 外面、派到第二台线程变量带不过去）改前也中、可达性弱（规则与定义里没有一处叫子 agent 用派发器） | A1 另立账 |
| T2 写者锁与看门狗 | 站得住 | 攻方 W1（子目录持锁者还在跑时派父目录照放、Bash 删没有闸）改前逐行同、弱 | W1 另立账 |
| T3 ⑭ 长活三行 | 站不住 | 攻方 P1（量过、主 agent 复跑坐实）：派发提示多机一行与两份实验定义走登记值读法，`lib_long_run.py` 的文件头判法、`detached-run.sh long-run-check` / `start`、`host-dispatch.py` ⓪ 仍要点名 `host-resources.py`；照执行员第 1d 步做 long-run-check 退 5，能过的写法要么现算（违反「机器忙也固定」）、要么声明与脚本不符；第一轮冻结副本上不中，是 G11 带进来、第二轮 R8 没看到的。P2（⑭ 登记值读法按字面找词，「登记值除以此刻在跑的重活数」放行）除法那一句是登记值读法下才放的；P3 改前也中 | 改法 Q1、Q3 零轮交用户；P3 另立账 |
| T4 verification 两句 | 站得住 | 攻方：轻型留存产物现判态数 0–639，单机路径 `--test-threads 1` 位置对；顺手记 K6 新文字（第 59 行 CPU 一侧没设取核数）与第 60 行（用例内部 CPU 线程开满默认线程）对同一类用例说两个数 | K6 文字冲突零轮交用户 |
| T5 清单完整性 | 部分站不住 | 正推：四条腿 94 条打中，75 条去向在三类或改法表，两轮判决都没写去向的归并成 U01–U08，调度记录另有 W01–W12（正推报告第 341–367 行）；W01 主 agent 坐实是真红 | 全部补进下面「另立账与交用户」两张表 |
| T6 数字能不能核 | 部分站不住 | 正推 132 个数：112 对、10 对不上（去重 8 处）、10 复核不了；主 agent 抽核 4 处都坐实 | 判决一、判决二与调度记录各加更正注（不改原句），见「数字更正」 |

## 交用户（零轮：被攻过零轮的形态，第三轮之后不再为它们开轮）

| 编号 | 内容 | 状态 |
|---|---|---|
| K1–K11 | 第二轮改法，已落进冻结副本，集成副本上自证与开关复核过（`verify4.summary`、`verify5-out/`） | 已实现，零轮 |
| W07 | 批 5 额外做的：丙两处判法（`["X"] = 值` 不算读；`;`/`&&`/`||` 之后同一行赋的算局部变量）、乙改 `lib_shell_words.py` 与登记表一行、收尾改 `lib_heavy_tests.py` 文件头 | 已实现，零轮 |
| W06 | 门禁 abandoned-rounds 格对 2026-10-12 之前的 4 行登记不要求出处 | 已实现，零轮 |
| Q1 | `lib_long_run.py` 文件头判法改成登记值读法（点名 `--emit-hosts` 与 `heavy-thread-default`、不许现算词），`detached-run.sh` 与 `host-dispatch.py` ⓪ 的说明跟着改，仓里文件头点名 `host-resources.py` 的现有脚本逐个改写 | 未实现，零轮 |
| Q3 | ⑭ 多机一行只认骨架那一句规范写法，或现算词表加「除以、在跑、nproc、核数、忙闲、负载」 | 未实现，零轮 |
| W01 | heavy-thread-defaults 不把 `in` 比较里的字符串常量当读环境变量（K5 新判法的误判；不改判法就进排除表一行） | 未实现，零轮；不修则提交时门禁红 |
| K6 文字 | `verification.md` 第 59、60 行对 GPU 对拍 CPU 一侧的线程说法统一成实情（读 `SINGLEFS_CRASH_JUDGE_CPU_THREADS`、没设取核数，归代码批） | 未实现，零轮 |
| U05 | 私有值词源加 `~/.config/singlefs/`：整仓新增命中 19844 行 / 2232 份，没并 | 开着 |

## 另立账（改前也中、或不在这一题的射程；不拿来否这次改动）

第一、二轮判决已列的照旧。这一轮新增：A1（`capped.sh` 套在派发器外面线程变量到不了第二台）、W1（父目录后派不看子目录还在跑的持锁者、Bash 删没有闸）、P3（预计挂钟只认开头）；正推 T5 的 U01（子 agent 手起命令 `capped.sh 8` 或不经 capped.sh，闸照放）、U02（虚拟工作区根不带 `-p` 的 `cargo run` 文字没写）、U03（前台派发的子 agent 有没有回执，补进核不动）、U04（门禁不认「别人没审过」，H2 只靠文字）、U06（第二台另起一行的线程上限行不判；E175 装置源码在 `research/results/` 下扫不到）、U07（攻方喂钩子没带主会话记录，⑦ ⑯ 回执没参与）、U08（「没打中」没有第二抽）；W02（research-script-selftest-coverage、stage-owners、layer0 缺红样本三样本来就红，不是这一批的）、W03（变异门禁依赖产物共用的六条岔路等 M1–M3 代价）、W04（admission 合入后崩溃枚举全绿标记全部不作数）、W05（补丁合入主工作区的时点与 H1 替换的提交状态）、W08（等两台 s2 跑完的几样）、W09（子 agent 在副本里证不了红、集成副本要补忽略文件这一流程缺口）、W10（排除表「只缩不涨」合入后补比一次）、W11（私有 memory 改指针）、W12（推迟到代码批的六·1 覆盖脚本与 crates 默认值）。

## 数字更正（原句不改，以这里为准）

| 原处 | 原句 | 现算 |
|---|---|---|
| 判决一第 5 行、调度记录第 104 行 | 正推腿 sha256 `60ed02db…a8a8` | `60ed02db…d8a8` |
| 判决二 R5 格 | `-p` 九种写法改后全拒（0→2） | 八种 0→2；`p-env-C` 改后仍 0（归 C2，批 5 K7 已改） |
| 第二轮正文第 30 行、调度记录第 115 行 | 16 项自证与门禁格、约 113 个弄坏开关 | 判对的 14 项绿、115 个开关红 |
| 第三轮正文第 22 行 | 94 处 / 66 行 / 831 份（heavy-thread-defaults） | 这三个数只对标签 `after-b4-fixup2`；`after-b5-fixup` 上那一格红 1 处（W01） |
| 第三轮正文第 22 行 | records 里能当出处的 504 行 | 能当出处 557 行，其中不带叫停词的 504 行 |
| 调度记录第 73 行 | 41 个文件 +1711/−247 | 含新增文件 68 个 +2550/−247 |
| 调度记录第 112 行 | 30 份 +1283/−237 | 含新增文件 52 个 +1425/−237 |
| 调度记录第 123 行 | 13 份自证 | 判对的自证 10 份 |

## 回看决策

不涉及决策：这一轮判的是 agent 定义、共用约束、项目规则的文字与钩子、门禁格的行为，没有一处引用或改动 `.claude/kb/decisions/` 里的分项。

## 用户定案（2026-10-13）

零轮怎么收：先选「修完四样，零轮随提交」，随即更正为「选错了 开三轮」——Q1、Q3、W01、K6 文字先实现（批 6），再把全部零轮形态（K1–K11、W06、W07、Q1、Q3、W01、K6 文字）另立一题，从头走三轮三方，攻完才合入。
