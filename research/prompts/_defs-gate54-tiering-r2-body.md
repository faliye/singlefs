# 三处定义改动与门禁 54 号分档：第二轮正文（2026-09-24）

<!-- doc-lint:not-numbers K1 K2 K3 S1 S2 S3 S4 S5 V1 V2 V3 V4 V5 -->

## 一、这一轮要判什么

第一轮判决 `research/prompts/defs-gate54-tiering-r1-main-verification.md`：K1 照留；K2、K3 站不住，改。改法 f2、f3、f4a、f5 是第一轮攻方在自己的模型上提、量过的，**被攻过零轮**；实现员落进真文件时又在 S3a 那一格换了删标记的时机（第二、三两轮报告 `research/prompts/defs-gate54-tiering-implementer/r2-report.md`、`r3-report.md`）。这一轮攻的是**改后的形态**，并派一条辩方腿复核第一轮的判决。

被判的文件（门禁 72 号按路径点名）：`.claude/gate.d/54-layer0-replay.sh`、`.claude/gate.d/stage-inputs.tsv`、`.claude/main-agent.md`（「暂存之后、提交之前跑门禁」那一行）、`.claude/agents/crash-verifier.md`（「做什么」第 1、2 条）、`.claude/agents/experiment-runner.md`（写范围一节与第 2 步）、`.claude/hooks/agent-write-scope.tsv`、`.claude/rules/implementation-workflow.md`（「复用上一次全量门禁的判定」一节与「门禁管哪一半」一段）、`README.md`（门禁那一段）。

| 格 | 改后的形态 | 在哪 |
|---|---|---|
| V1 | f5：全量挪到暂存之后，在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑 `--full <它的根>`；快档红时出路句给三行能照跑的命令 | `.claude/main-agent.md`；54 号出路句 |
| V2 | f3 与 r3 的删法：标记 `<common-dir>/singlefs-layer0-full-green.<输入哈希>`，一批输入一格；`--full` 开跑不删任何一格，这一趟没写成标记就退出时（EXIT trap）才删这批输入那一格；旧格只增不减 | 54 号 |
| V3 | f4a：快档另核标记里 `LAYER0`、`LAYER0B` 两行都是 `exhaustive=true` | 54 号 |
| V4 | f2：「这次改动碰没碰输入」读 `stage-inputs.tsv` 里 54 号登记的三条（`crates/ Cargo.toml Cargo.lock`）；登记表里没有这一行判红 | 54 号；`stage-inputs.tsv` |
| V5 | 第一轮判决本身：K1 照留、K2 改（崩溃验证员默认不带 `--full`）、K3 改 | 判决 |

## 二、实现今天的样子（主 agent 的观测，2026-09-24 现查）

- 54 号真文件 397 行，sha256 `232c8c0de11bf6561f9ca1736652745f387d05832ad0a5c99a08bc80c0386bf7`（第三轮报告）；`bash -n`、gate-lint、shell-lint、doc-lint 都绿，rules-lint 按 `gate.sh`「规则纪律（项目本地）」的调法重跑绿。
- 真仓上只跑过快档一次（第二轮报告：两条流快档绿、没有这批输入那一格、退 1，出路是 f5 那一句）；**`--full` 在入库装置上一次都没真跑过**。自证都在合成仓 + 假 cargo 上（驱动脚本 `research/prompts/defs-gate54-tiering-implementer/selftest-driver.sh` 与三份报告的附录）。
- 实现员自报的边角：SIGKILL 时这一趟的 trap 不跑，同一批输入上一趟写下的绿格留着；跑的过程中输入变了判红时，trap 删的是开跑那一批输入的格；f5 的出路命令在 `$TMPDIR` 下留一个装 `staged.patch` 的目录；f4a 只挡 `exhaustive`，线程数与 `CHECKER` 行被改坏快档看不出。
- `crates/` 这一轮不改；两条流的用例编译期与运行期都不读 `.claude/kb/layout/` 与 `research/results/`（第一轮正文第二节已现查，`stage-inputs.tsv` 第 11 行注释同）。

## 三、五格要答的

### V1　f5 的 worktree 与暂存区

建完 worktree 之后别人又往暂存区放东西（几个会话共写一个仓），这一趟写下的那一格的输入哈希还对得上下一次 `--staged` 吗；出路句那三行命令在「暂存区为空」「暂存区里只有别的会话的东西」「HEAD 变了」三种现场下各给出什么；`$TMPDIR` 留下的目录会不会被下一次同名命令当成现场读进来。

### V2　一批输入一格，失败才删

确定性那条前提（同一批输入跑出同一结论）在什么情形下不成立：54 号自己的判定改了而输入哈希不含 54 号、cargo 或工具链版本变了、线程数不同；这些情形下旧绿格被当成这一批的全绿，是不是一条可达历史；trap 在 SIGKILL、OOM、机器掉电下不跑，留下的是哪一格。

### V3　快档核两行 `exhaustive=true`

标记里还有哪些行被伪造或写坏时快档照样判绿；那一格是不是只挡了第一轮 S4 那一种。

### V4　登记表驱动的「碰没碰输入」

`stage-inputs.tsv` 里 54 号那一行本身被改窄、被删、写错路径时 54 号的行为；与 `research/scripts/stage-must-run.sh` 读同一张表，两处判法对不对得上。

### V5　第一轮判决站不站得住

K1「照留」靠的是 33 号与 59 号兜「只追加、不改别人的行」，K2「默认不带 `--full`」与崩溃验证员定义第 1 条「另一趟全量在跑时不起 54 号」合起来，有没有一条历史让该跑的层 0 全量一次都没跑、而门禁判绿；第一轮打中的 S1–S5 是不是同样打中「分档之前的 54 号」，是就不拿来判分档改动。

## 四、分工（两条攻方腿的攻击面分开写）

| 腿 | 立场 | 格 | 要它交什么 |
|---|---|---|---|
| **云端辩方（Sonnet）** | 替第一轮判决辩护，也替它找够不着的地方 | V5 | 逐条：第一轮每个打中够不够得着改动、是不是也打中改前的做法；K1、K2、K3 的结论推不推得出（引原文整行抄）；推不出的写缺哪一句 |
| **云端攻方（Opus）** | 造工作流历史打改后的形态 | **V1、V2** | 每条历史写成可复现的命令序列，在仓的副本里真跑 54 号的判定段（合成日志与假 cargo，照 `selftest-driver.sh` 的做法，不真跑全量）；给出判错的历史，或写「构造不出」并说清卡在哪一步 |
| **本地攻方** | 逐格填表 | **V3、V4** | 事实表逐格填：标记里每一行（`LAYER0`、`LAYER0B`、`CHECKER`、线程数、`input_hash`、`input_file`、`finished_utc`）被写坏时快档的判定；`stage-inputs.tsv` 那一行的每种坏法下 54 号与 `stage-must-run.sh` 各给什么；不许只答 yes / no |

## 五、跑前条款

| 条款 | 什么观测会让它触发 |
|---|---|
| 一格判「站得住」⇒ 改后的形态照留 | 两条攻方腿在那一格都构造不出判错的历史（本地腿两次抽样一致） |
| 一格判「站不住」⇒ 改 54 号或定义；按「第三轮之后停」，第三轮只攻前两轮都站住的形态 | 攻方给出一条可达历史：该跑全量的没跑、不该红的红了、该红的没红，或 agent 照定义做会覆盖别人的东西 |
| V5 判第一轮某个打中「不分辨」⇒ 那一格另立一笔账，不拿来判分档 | 辩方指出同一段历史在分档之前的 54 号上给出同样的错 |

⚠️ 打中之后先按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判据自己也会写错：打中之后先判是哪一种」那四句过一遍。

## 六、各条腿交什么

- 报告先写进 `research/prompts/defs-gate54-tiering-r2-<腿名>-output.md`，**每次工具调用写进文件的内容不超过 150 行**，分段追加，第一段排他新建，回复只写短句。
- 草稿放腿自己的目录 `/tmp/claude-1000/<腿名>/`；攻方的模型与驱动脚本随报告拷进 `research/prompts/defs-gate54-tiering-r2-opus-model/`。
- 引 kb 与规则条款写那份文件自己的行号，去文件里现查，不从背景材料里数。
- 引产物整行抄，带文件名。
