# d137-gov-r1：一百三十七节那一轮的治理文件改动——第一轮（2026-10-29）

## 一、题面

`records/2026-10-23-subagent拆分提案.md` 第一百三十七节 ①–⑲ 那一轮（调度记录 `records/2026-10-29-一百三十七节那一轮调度记录.md`）改了钩子、定义与共用约束、规则与 13 份研究脚本。用户 2026-10-29 弹窗定：钩子批与定义批合成一轮三方，研究脚本一并带进来（「带进去，一轮攻全」）。这一轮是第一轮，照 `.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」的第一轮分工攻：过度外推与越位、内部矛盾与循环依赖、哪一条最脆弱。派法：一条 Sonnet 正推腿、一条 Opus 攻方腿。

用户在这一轮定的口径（逐字见调度记录「用户在弹窗里定的」与其后各行）：
- 钩子判法：认不出的一律拒（偏严）；三种静默漏拒（命令替换里不加括号的 case 模式与注释、`coproc`、位置参数与变量拼 cargo 旗标）改成拒。
- 共用切词偏严三类：cat 不看源文件在不在、重定向目标花括号展开成几个词——照拒、拒绝说明改贴切；函数体外引号里的花括号——放。
- 撤旧根：拆成两样——工作根的过渡期放行撤掉；Claude Code 的会话 scratchpad 另立常设的「会话临时目录」，检出钩子 ⑦、看门狗、完工清理闸照旧管它。
- 显存额度队 `CardMemoryQueue` 不建等卡标记（那份补丁不打）。
- 实验准入按需接：设计员写重跑登记时准入判法退 2、退 3 照拒，别的码判不了放行并记检出。

被判的文件（快照 `research/prompts/d137-gov-r1-snapshot/files-sha256.txt`）：
- 钩子：`.claude/hooks/lib_shell_words.py`、`heavy-test-guard.sh`、`bash-command-detector.sh`、`lib_heavy_tests.py`、`runner-dispatch-guard.sh`、`lib_work_root.py`、`lib_prompt_scratch_line.py`、`lib_write_scope.py`、`write-guard.sh`、`handback-guard.sh`、`completion-cleanup-guard.sh`、`lib_snapshot_conflicts.py`、`agent-write-scope.tsv`。
- 定义与共用约束：`.claude/agent-common.md`、`.claude/main-agent.md`、`.claude/agents/` 下 `tooling-writer.md`、`implementation-writer.md`、`experiment-runner.md`、`investigator.md`、`three-way-forward.md`、`three-way-attack.md`、`three-way-defense.md`、`three-way-materials.md`、`mutation-triage.md`。
- 规则与流程：`.claude/rules/verification.md`、`.claude/skills/crash-test/commit-stages.md`、`CLAUDE.md`（四篇项目本地规则改成开会话读一次）。
- 研究脚本：`scripts/agents/agent-watch.py`、`scripts/agents/watch.sh`、`scripts/agents/copy-originals.py`、`scripts/hosts/run-with-memory-cap.sh`、`scripts/hosts/memory_peaks_lib.py`、`scripts/hosts/detached-run.sh`、`scripts/hosts/core_usage.py`、`scripts/hosts/clean-session-scratch.py`、`scripts/hosts/peer-host-lib.sh`、`scripts/gate/apply-writer-patch.py`、`scripts/mutation/mutation-shard-run.sh`、`scripts/crash/crash-point-replay-shard-run.sh`、`scripts/crash/crash-amplification-item.sh`。

## 二、实现今天的样子（主 agent 的观测，2026-10-29）

- `crates/` 里没有相关实现：`grep -rln 'shell_tokens\|heredoc_feeds_shell\|registered_light_selftest\|claude_session_temporary_root\|CARD_WAIT_MARK' crates/` 零命中。被判的是项目的协作钩子、定义与研究脚本；方案按它们今天的代码来谈。
- 这一轮各件只含自己改动的 diff（主 agent 核过 sha256 前 8 位）：钩子 `~/.local/share/singlefs/work/d137-hooks/own.diff`（c2003ea0）、`d137-hooks-2/own.diff`（da344e8d）、`d137-hooks-fix/own.diff`（9147a5bd）、`d137-legacy-root/own.diff`（f3cfce93）、`d137-legacy-root-2/own.diff`（3cdac62c）、`d137-text/own.diff`（321b9a5c）；定义 `d137-defs/own.diff`（40e19aaa）、`d137-defs-2/own.diff`（d67757ea）、`d137-defs-3/own.diff`（a038a431）、`d137-watch-follow/own.diff`（73a93387）；脚本 `d137-watch/own.diff`（e6176de1）、`d137-memcap/own.diff`（fda02b1f）、`d137-memcap-2/own.diff`（5f254e92）、`d137-memcap-3/own.diff`（daac16d9）、`d137-shard/own.diff`（edf53794）；钩子另两件 `d137-bash-env/own.diff`（4120b485）、`d137-bash-env-2/own.diff`（cfefe0bf）。各件报告是同目录的 `result.md`。这些路径都在 `~/.local/share/singlefs/work/` 下。
- 被判文件里有别的会话没提交的改动（例：一个别的会话改了 `run-with-memory-cap.sh`、`memory_peaks_lib.py` 的峰值口径；另一个会话在 `runner-dispatch-guard.sh`、`write-guard.sh`、`agent-watch.py` 加「弄坏开关只在自证里认」）：只判这一轮的改动，别人的块不判。
- 主 agent 在主工作区现跑的读数（量的）：`run-with-memory-cap.sh --selftest` 查了 154 项都对；`multi-host-run.sh --selftest` 53 格都对；其余各份自证读数在各件报告；四份全量复跑（第三轮正推腿乙与腿甲、第二轮两条腿的装置，以此刻五份钩子为「此刻」那棵树）在 `~/.local/share/singlefs/work/d137-hooks/main-replays-3/`（这一趟在腿跑着时跑，读数出来之前腿不引它）。
- `run-with-memory-cap.sh` 开关表改完，整表读数（基础组补了第二趟）合成整份一趟后 83 条开关全「对」（`d137-memcap/main-runs/final-2-analyze.txt`）；`agent-watch.py` 133 条开关整表复跑归提交前门禁 code-tooling 的 break-switch-replay 格。
- 已知没认、判决之后与这一轮打中的合成一批修的（不用再报这几种本身，报同类的别的写法）：登录 shell 读的 profile 一类、系统级启动文件、别的程序（python、make 这类）里起 shell、`bash x.sh` 起的脚本正文里起 claude（检出钩子 ⑬ 不读脚本正文）。

## 三、格

### G1　照用户口径，钩子判得对不对（正推腿）
被判的钩子此刻的判法，是不是照 bash 的真实行为与用户定的口径判：认不出的一律拒落在了哪几处、边界（读进来的脚本正文里 `"$@"`、`$cmd` 不判；本仓里的脚本与它逐字节相同的拷贝不判；同族脚本逐字节相同的拷贝照本仓那一份认）有没有放过真重型、有没有把正当命令拒掉；偏严三类、撤旧根两样、准入退码各自照定了没有。每条给一个能复跑的喂法与原样输出。

### G2　定义、共用约束、规则与钩子和脚本实际做的一致吗（正推腿）
`.claude/agent-common.md`、`.claude/main-agent.md`、各份定义、`.claude/rules/verification.md`、`commit-stages.md` 这一轮新写的句子，与钩子、看门狗、内存包装此刻的实际判法逐句对；只写怎么做（`.claude/singlefs-ai-sop/rules/rules-discipline.md` 第 1–3、7 条），有没有写进理由、经过；几份文件之间同一件事说法一致不一致。

### G3　攻：过度外推、内部矛盾、最脆弱的一条（攻方腿）
- 过度外推与越位：各件报告与文件头说「判得了」「全对」「只记不叫醒也不漏报」的地方，哪一句超出了它量过的范围；看门狗把一批告警改成只记不叫醒之后，有没有真卡住、真漏跑的情形从此叫不醒主 agent。
- 内部矛盾与循环依赖：钩子之间（重型闸、检出钩子、派发闸、写闸）、钩子与看门狗、包装与峰值库与派发器之间，对同一件事判法不一致的地方；「会话临时目录」与工作根拆开之后，各处认法有没有对不上。
- 最脆弱的一条：这一轮改动里，最容易被一种普通写法绕过、或在并发与负载下判错的那一处（例：认不出一律拒的边界、`scope_result` 读 journal 兜底、等卡标记、`pending_bytes` 跳过标记、脚本正文绕闸那一格）；给出能复跑的反例。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 正推腿（Sonnet） | G1、G2 | 照真 bash 与钩子、脚本此刻的代码逐条对照；自己写判法、自己起真 bash 喂钩子 |
| 攻方腿（Opus） | G3 | 找反例与矛盾；不重复正推腿的逐条对照 |

## 五、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引代码写被判文件在主工作区的路径与行号，同时写那份文件此刻的 sha256 前 8 位；快照里的 sha256 与此刻对不上的，写明是哪一份。
- 喂钩子在自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`）里拷一份钩子喂，cwd 放草稿目录，不在主工作区里改任何文件；喂法与原样输出写进报告。
- 起真 bash 的判法在脚本里用 `setsid --wait` 起每条样本，超时与结束在脚本里杀整组；交回前 `ps` 核自己起的进程一个不剩，留下了写清是哪条样本。
- 不跑重型测试、不起 cargo、不碰 GPU；自证与探针经 `scripts/hosts/run-with-memory-cap.sh` 带不超过 4G 的上限，排不上可以带 `ulimit -v` 与 `nice` 直接跑并写明。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句；早点开写。
