# d137-ledger-defs-r1：账本逐节现查那一批的定义与规则改动——第一轮（2026-11-02）

## 一、题面

subagent 账本逐节现查（`records/2026-11-01-subagent账本逐节现查.md`）判「仍欠」的条目，2026-11-02 按文件聚簇改完；脚本与门禁那一侧改完之后，定义、共用约束、规则与 skill 跟着改了一批句子，一轮三方都没攻过，立这一题。

用户定的口径：
- 2026-11-01「subagent不得有欠账 明白吗」「要全部清掉 这次」。
- 2026-11-02 对几处放宽口子（`SINGLEFS_EXPERIMENT_RERUN_REASON`、`apply-writer-patch.py --snapshot-ruled`、`kb-spec-check.py --allow-uncommitted`、`stale-candidates.py --allow-dirty-worktree`）答「现在已经统一交给了 Task-server 需要删就删」。
- 做法本身（子 agent 不交任务、经统一任务服务交、判定库只有一份）不重议，攻的是这一批写出来的句子有没有漏洞、与脚本和门禁此刻的做法对不对得上。

这一批改了哪些句子：
- 工具员改的整份 diff：`~/.local/share/singlefs/work/d137-b19-definitions/this-batch.diff`（134 行，sha256 前 8 位 2a04697b），逐条说明在同目录 `result.md` 的「逐条」一节；
- 主 agent 照它给的旧串新串改 `commit-stages.md` 两处：`~/.local/share/singlefs/work/d137-b19-definitions/commit-stages-main.diff`。

条目（行号指 `records/2026-11-01-subagent账本逐节现查.md`）：
1. 一百二十六 ①②（第 136–137 行）：`experiment-designer.md` 写明走崩溃判段的装置起法带 `--features gpu`、判据里写 `grep -c NOT_RUN` 得 0；`experiment-runner.md` 交回前核同一样。
2. 一百三十二（第 149 行）：`implementation-writer.md` 第 7a 步列全五处登记，删指向不存在小节的说法。
3. 一百三十（第 144 行）：`verification.md`、`implementation-workflow.md`、`commit-stages.md` 快档字句补 product-gpu-real-card-tests 一格。
4. 一百六十六 ⑨ 计数栏（第 359 行）：`mutation-triage.md` 第 32、41 行与 `.claude/rules/mutation-sampling.md` 同一套。
5. `agent-common.md` 第 98 行一带去掉「起准入之前清掉 `SINGLEFS_EXPERIMENT_RERUN_REASON`」。
6. 一百六十六 ②（第 357 行）：四份 read-once 理念规则由定义按路径点名——`engineering-philosophy.md`、`machine-first.md` 加在 `implementation-writer.md`、`tooling-writer.md`，`pushback-discipline.md` 加在 `three-way-attack.md`，`design-doc-discipline.md` 加在 `clerk.md`（「改 `records/` 下的文件时另读」）。

被判的文件（快照 `research/prompts/d137-ledger-defs-r1-snapshot/files-sha256.txt`）：
- 定义与共用约束：`.claude/agent-common.md`，`.claude/agents/` 下 clerk、experiment-designer、experiment-runner、implementation-writer、mutation-triage、three-way-attack、tooling-writer 共 7 份。
- 规则：`.claude/rules/verification.md`、`.claude/rules/implementation-workflow.md`。
- skill：`.claude/skills/crash-test/commit-stages.md`。
- 对照（不判，只读来核「写的与此刻做的对不对得上」）：`scripts/gate/admission.py`、`scripts/gate/merge-and-verify.sh`、`.claude/gate.d/doc-experiments.sh`（preregistration-judge-features 格）、`.claude/gate.d/code-tooling.sh`（rules-named-by-definitions 格）、`scripts/crash/crash-case-check.py`、`scripts/mutation/mutate.sh`、`.claude/gate.d/checker-tier-crash-point-replay.sh`、`.claude/rules/mutation-sampling.md`、`CLAUDE.md`。

## 二、实现今天的样子（主 agent 的观测，2026-11-02）

- 这一批只改定义、规则与 skill 的句子，不碰 `crates/`；`crates/` 里与这几条相关的只有被判句子引到的 `crates/singlefs-checker-tier/Cargo.toml` 的 `[features]`（工具员报此刻只剩 `gpu`，账本原写的 `verdict-store,gpu` 没照抄），腿现查。
- 被判文件相对 HEAD：`git diff HEAD --stat` 末行「11 files changed, 178 insertions(+), 179 deletions(-)」，其中别的会话已暂存、没审过的改动也在，这一轮只判第一节点名的那几处句子。
- 对照的脚本与门禁这一批改过、主 agent 经服务复跑过：`admission.py` 自证（任务 4532，退 0）；`merge-and-verify.sh` 自证（任务 4395，退 0）与开关 `product-gpu-cell-dropped`（任务 4397，红在点名格）；doc-experiments 的 preregistration-judge-features 格真仓（任务 4446，绿）；code-tooling 的 rules-named-by-definitions 格真仓（任务 4436 那一趟只红别的格 agent-write-scope）；`crash-case-check.py` 自证与 provenance（任务 4414、4420）。
- 经服务跑过的两条 lint：rules-lint 只红 `CLAUDE.md:111` 两处（HEAD 里就在）；doc-text 落在被判文件里的只有 `.claude/agent-common.md:93`（HEAD 里就在）。

## 三、格

### F　句子与脚本、门禁此刻做的对不对得上（正推腿）
逐条核第一节六条：每一句说的起法、判据、登记处、计数栏、点名的规则路径，与对照文件此刻的文字逐字比；照这一句做，会不会撞拒、漏判、交出与门禁判据不同的东西；同一件事在几份定义、规则、skill 里写的是不是同一个意思；删掉的那半句删完之后上下文还通不通、有没有别处还在教旧做法。

### A　找漏洞（攻方腿）
做法本身不攻。攻这一批写出来的句子：照字面做会出错的边角（例：不走崩溃判段的装置被要求带 gpu 特性、`grep -c NOT_RUN` 在产物里数到无关的行、五处登记少了一处或多了一处、计数栏与 `mutate.sh` 收尾那行对不上、点名的规则路径不在、某份定义点名了不该它读的规则）；这一批该改而没改的句子（全仓定义、规则、skill 里还在说旧做法的地方）。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 正推腿 | F | 从句子原文推到「照做会发生什么」，与对照文件逐处对 |
| 攻方腿 | A | 找漏洞；不重复正推腿的逐份对照，不攻做法本身 |

## 五、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引定义、规则、脚本、门禁写主工作区的路径与行号，同时写那份文件此刻的 sha256 前 8 位；快照里的 sha256 与此刻对不上的，写明是哪一份。
- 子 agent 不往统一任务服务交任务：要跑的命令（喂钩子、自证、门禁单格、探针）拼成 `task-server/task submit -- <命令>` 的整条，写进报告「要主 agent 跑的命令」一节，由主 agent 交。
- 不造新的绕过钩子的写法。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句；早点开写。草稿与临时文件放 `~/.local/share/singlefs/work/<腿名>/`。
