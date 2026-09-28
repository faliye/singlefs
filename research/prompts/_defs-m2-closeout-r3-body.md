# 里程碑二收尾：定义改动一轮三方，第三轮正文（2026-09-26）

<!-- doc-lint:not-numbers G1 G2 G3 G4 G5 G6 G7 G8 G9 H1 H2 H3 H4 H5 H6 H7 H8 H9 K1 K2 K3 -->

## 一、这一轮要判什么

第三轮，也是最后一轮（`.claude/rules/three-way-inference.md`「多轮」：第三轮之后停）。第二轮判决 `research/prompts/defs-m2-closeout-r2-main-verification.md` 整份进材料；它第三节的改法 G1–G9 已由修定义的 agent 改进文件（被攻过零轮），这一轮只攻 G1–G9 的改后字面，攻击面不重复前两轮。第三轮里新冒出来的改法不再为它开一轮，写进判决的交用户表、标「零轮」。

| 格 | 被攻的 | 问题 |
|---|---|---|
| K1 | **G1**（撤回整条包 `gate.sh`；15、74 号在阶段里面经内存包装，上限变量 `GATE_RESEARCH_BUILD_MEMORY_MAX`、`GATE_MODEL_DIFFERENTIAL_MEMORY_MAX`，没设取 8G）与 **G9**（崩溃验证员 54、57 号的上限每道一个数） | 阶段里面包之后，包装自己的退出码 250–254 与阶段判红分得开吗；实现员、崩溃验证员、门禁分诊员照各自定义跑 15、74 号时各是什么结局；15 号的 8G 没量过，峰值表里没有 research 整工作区 cargo test 的行——撞了是什么样子；`.claude/gate.d/stage-inputs.tsv` 的 74 号那一行主 agent 已补上包装脚本，15 号有没有同样的缺口 |
| K2 | **G2、G5、G7**（执行员：产物一个不删、`replay.sh` 改指新文件；判决行点名的对象；读实验页或索引的十道阶段挪到写完实验页之后） | G7 的十道（27、34、40、69、75、84、85、86、88、99）是修定义的 agent 照「凡是读实验页的」现查扩出来的，比判决括注里的三道多：扩得对不对、有没有漏一道、挪了之后第 6 步还剩什么；G2 与 40 号跳过 `*.r[0-9].out` 的命名规则怎么交叉；G5 的「名字表示违例 / 不匹配 / 歧义 / 失败」在今天全部判决行里各落哪一边 |
| K3 | **G3、G4、G6、G8**（攻方原型的流只许自己造、一轮合计 10⁷；弹窗闸收严；退出码文件带批号件号、数件数；本地辩方样本前缀） | 照字面还有没有绕得过去的写法；G4 收严之后有没有把一个带出处的合法断言误拒；G6 的数件数在检出 hook 的出路里没有写（`bash-command-detector.sh` ④ 的出路只写 `a & b & wait`），照 hook 的出路做会不会又回到吞退出码 |

**共用问句**：照改后的字面干活，哪一步会做错或做不了；举出具体的派发情形或命令。

## 二、实现今天的样子（主 agent 的观测，2026-09-26）

- 被判的改动：修定义的 agent 报告 `research/prompts/defs-closeout-r2-fixes-tmp-evidence/report.md` 与它的 `my-changes-final.diff`（258 行，比的是开工时 `cp -p` 的备份）；材料员把 diff 原样放进附录二，把报告的逐条表、探针与门禁两节抄进附录。开工快照 `research/prompts/defs-m2-closeout-r3-snapshot/sha256sums.txt`。
- 这一轮被改的文件：`.claude/agents/three-way-attack.md`、`three-way-local-defense.md`、`experiment-runner.md`、`crash-verifier.md`、`gate-triage.md`、`implementation-writer.md`，`.claude/agent-common.md`，`.claude/main-agent.md`，`.claude/hooks/ask-user-claim-guard.sh`（自证 30 → 36 格），`.claude/gate.d/74-model-differential.sh`、`.claude/gate.d/15-research-build.sh`（阶段里的 cargo 经包装），另有主 agent 补的 `.claude/gate.d/stage-inputs.tsv` 74 号那一行（加 `research/scripts/run-with-memory-cap.sh` 与 `research/scripts/capped.sh`）。
- 两道阶段在极小的假根上跑过三种结局（默认 8G 绿；200M 上限下分配 400 MiB 包装退 250、走新的红出路；测试 panic 走原来的红出路），251–254 没造出来；15 号没有样本、也没有自证。
- 这一轮不碰 `crates/`。

## 三、条款（材料员整段抄进附录）

- 第二轮背景材料 `research/prompts/_defs-m2-closeout-r2-background.md` 的条款照旧；
- 第二轮判决整份；
- `research/scripts/run-with-memory-cap.sh` 文件头（退出码 250–254 的含义、嵌套那一句）。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | K2 | 逐条核 G2、G5、G7 改后的字面是不是第二轮判决第三节要的：G7 扩成十道扩得有没有依据、有没有做过头 |
| 云端攻方（Opus） | K1、K2、K3 | 造派发情形与命令攻改后的字面；要喂 hook 的只喂 JSON、只看退出码，被判的命令一条都不执行 |
| 本地攻方 | K2 的 G7 | 按事实表逐格填：十道阶段各自读不读实验页或索引（修定义的 agent 报告第一节表下的逐道 grep 行号）、登记给不给执行员（`.claude/gate.d/stage-owners.tsv`）；每行写来源文件与行号，每张表不超过 6 行 |

两条攻方腿不重叠：Opus 造情形攻语义，本地只逐格核 G7 那张名单的字面依据。

## 五、交付

- 腿的报告写 `research/prompts/defs-m2-closeout-r3-<腿名>-output.md`，分段落盘（每次工具调用写进文件不超过 150 行，第一段排他新建）。
- 探针放 `research/prompts/defs-m2-closeout-r3-<腿名>-model/`，带 `SHA256SUMS` 与 `rerun.sh`。
- 引文件写那份文件自己的行号，引的行是那句原文自己所在的行。
- 不跑重型测试；只停自己起的进程，一次一个 `python3 .claude/singlefs-ai-sop/scripts/proc.py stop <pid>`；等后台任务就结束本轮等通知，不用 `true` 或 `sleep` 空转；交回之前后台不许留着跑的东西。
