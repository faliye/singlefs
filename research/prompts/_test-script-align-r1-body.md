# 对接上游测试脚本规范这一批的改动与欠账：三方第一轮正文（2026-10-01）

<!-- doc-lint:not-numbers T1 T2 T3 T4 G3 G6 G7 G8 G9 H4 H6 H9 F2 F4 F7 F8 F12 -->

## 一、这一轮要判什么

用户 2026-10-01 定：这一批的欠账一起走一轮三方。这一批是把本仓对齐上游新规则 `.claude/singlefs-ai-sop/rules/test-script-discipline.md`（副本已同步，版本戳仍 0.0.62）做的改动，调度与经过在 `records/2026-10-01-对接上游测试脚本规范.md`（腿只读，不改）。

被判的文件（工作区相对 HEAD 加暂存区的改动里，属于这一批的那一部分；同文件里别的会话没提交的改动不判，`git diff` 里认不出归属的照调度记录里各件交回那几行认）：

- 规则：`.claude/rules/verification.md`、`.claude/rules/implementation-workflow.md`、`CLAUDE.md`（只判 `@` 新规则那一行与「脚本自证与门禁样本」那一句）。
- 定义与共用约束：`.claude/agent-common.md`（「门禁」一节那一句）、`.claude/agents/tooling-writer.md`（第 7 步）、`.claude/agents/crash-verifier.md`、`.claude/main-agent.md`（「用户要求时」三处）、`.claude/skills/crash-test/SKILL.md`。
- 钩子：`.claude/hooks/lib_heavy_tests.py`（`classify_selftest` 与样本档放行）、`.claude/hooks/heavy-test-guard.sh`、`.claude/hooks/selftest-registry.tsv`。
- 门禁：`.claude/gate.d/checker-tier-layer0-replay.sh`（feature-gated-tests 格与样本）、`.claude/gate.d/checker-tier-crates-mutation-replay.sh`（认 `NOT_RUN_NO_GPU_ADAPTER`、`--show-output`）、`.claude/gate.d/e165-pinned-red-examples.tsv`（文件头）、`.claude/gate.d/stage-owners.tsv`（feature-gated-tests 一行）、`.claude/gate.d/stage-inputs.tsv`（54 号那一行）。
- 研究脚本：`research/scripts/crash-amplification.sh`、`crash-amplification-item.sh`、`crash-amplification-lib.sh`、`crash-amplification-two-hosts.sh`、`mutate.sh`、`crates-mutation-rows.py`、`verus-proof-check.py`、`broad-mutation.py`、`e165-replay-mutation-kill-bounds.sh`、`crash-case-check.py`、`agent-watch.py`、`mutation-shard-run.sh`（后两份只判注释）。
- kb：`.claude/kb/checks-owed.md` 的 C604–C613 与 C620 这十一行；`.claude/kb/decisions/13-验证路线.md`、`06-快照实现模型.md`、`16-发布语义.md` 与 `.claude/kb/decisions-history.md` 里「夜间档」改「全量档」的那几处（D13（验证路线） 已定项 9、15）。

| 格 | 被攻的 | 问题 |
|---|---|---|
| T1 | 规则去重：本仓规则删掉通用写法、改指上游节名；新加进度间隔 5 分钟；删掉「夜间」时机 | 删掉的句子里有没有一句是本仓独有的判据、上游那一节其实没罩住；指向的节名在副本里是不是逐字存在；「用户要求时」之后，还有没有一处规则、定义或门禁出路仍按夜间安排或互相矛盾 |
| T2 | 钩子放行登记为 light 的自证与门禁样本档；定义与共用约束随之改写 | 登记为 light 的哪一份 `--selftest` 或样本档其实会起 cargo、rustc、qemu、herd7、verus（读它的自证代码，或照登记的写法真跑一次看进程树）；放行条件能不能被一条子 agent 会写的命令绕过去跑真重活；定义与共用约束改后的字面，照着干会不会做错 |
| T3 | 入口与门禁的判法：崩溃放量入口（退出码 78 / 247 / 248、复用数与「取不出」、本次未跑、全量展开深度 28、弄坏开关只在自证里认）；54 号带特性那一趟；crates 变异门禁认 NOT_RUN 与 `--show-output`；`mutate.sh` 限时改用包装自己的、排队行转打；几份长跑脚本按挂钟 300 秒打进度 | 哪一格会假绿（该红不红）、哪一格会假「本次未跑」（该判的没判）、哪一格会假红；退出码表与实际出路逐个对不对得上；自证格与弄坏开关是不是分得出差别 |
| T4 | 欠账 C604–C613、C620 的「要拦什么 / 怎么拦」；D13（验证路线） 的「夜间档」改「全量档」 | 每一笔的「怎么拦」能不能落成一个会红的检查（写出会红的输入）；有没有这一批该记而没记的缺口；D13 改写有没有顺带改掉「全量档全绿是合并前的异步准入判据」以外的意思 |

**共用问句**：照改后的字面干活，哪一步会做错、做不了，或者让一个该红的东西绿着过去；举出具体的命令、输入或派发情形。

## 二、实现今天的样子（主 agent 的观测，2026-10-01）

- `crates/` 这一轮不判：第 2 批的四份 `crates/` 补丁（崩溃放量结果行、GPU 对拍、种子复现、E165 装置，在 `/tmp/claude-1000/impl-*/patch/`）还没打进主工作区，要等别的会话几轮代码三方的判决写完，合入后另开代码轮。腿判 T3 时，入口脚本解析的驱动行字段（`states_reused=`、`exhaustive=`、`expand_up_to=`）今天的 `crates/singlefs-checker-tier/tests/` 里还没有，按调度记录里写的约定判，不按 `crates/` 今天的打印判。
- 主 agent 2026-10-01 带 `SINGLEFS_HEAVY_TESTS=user-request` 跑过：`crash-amplification.sh --selftest`（交回时 21 格、开关轮 16 个开关各红在点名格，由实现它的 tooling-writer 跑）；54 号七份样本全判对、五个开关各让点名样本判错；crates 变异门禁四份样本照预期（red、green、not-run 判对，`notrunasmissed` 下 not-run 判错）；`lib_heavy_tests.py --selftest`、`heavy-test-guard.sh --selftest` 退 0；`mutate.sh --selftest` 不带开关与两份并行都退 0（tooling-writer 跑）；`crash-case-check.py --selftest`、`gate-structure-check.py` 退 0；`admission.py --selftest` 283 格 1 格红，红在暂存区别的改动新加的那一格，不是这一批的。
- 各件的报告（改了什么、文件:行、自证原样输出）：`/tmp/claude-1000/tooling-writer-*/report.md`、`/tmp/claude-1000/investigator-*/report.md`、`/tmp/claude-1000/kb-scribe-no-nightly/report.md`。

## 三、条款（材料员整段抄进附录）

- 上游 `.claude/singlefs-ai-sop/rules/test-script-discipline.md` 全文；
- `.claude/kb/checks-owed.md` 的 C604–C613、C620 十一行；
- `.claude/kb/decisions/13-验证路线.md` 的已定项 9、15 两节；
- 被判的规则、定义与共用约束文件里这一批改过的段落（腿到主仓现读）。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | T1、T4 | 从上游规则与本仓规则逐句推：本仓删改之后，每一条判据还落在哪一处、有没有一句落空；欠账每一笔照它写的「怎么拦」推出会红的输入 |
| 云端攻方（Opus） | T2、T3 | 造输入攻改后的钩子、门禁与脚本：喂钩子只喂 JSON、看退出码；脚本在草稿目录的仓拷贝上用假驱动、假 cargo、假时钟跑，找假绿、假未跑与绕过 |

## 五、交付

- 腿的报告写 `research/prompts/test-script-align-r1-<腿名>-output.md`，分段落盘（每次工具调用写进文件不超过 150 行，第一段排他新建，回复只写短句）。
- 探针放 `research/prompts/test-script-align-r1-<腿名>-model/`，带 `SHA256SUMS` 与 `rerun.sh`（复跑挂钟写在 `rerun.sh` 文件头）；草稿与临时文件放 `/tmp/claude-1000/<腿名>/`。
- 引文件写那份文件自己的行号，行号去被引文件里现查，不从背景材料里数。
- 不跑重型测试；登记为 light 的 `--selftest` 只在草稿目录的仓拷贝上跑；只停自己起的进程。
