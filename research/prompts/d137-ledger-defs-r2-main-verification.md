# d137-ledger-defs-r2 判决（2026-11-02）

被判：`research/prompts/_d137-ledger-defs-r2-body.md` 第一节两件修补（第一轮判决 `research/prompts/d137-ledger-defs-r1-main-verification.md` 判「采」的各条）。腿：辩方 `research/prompts/d137-ledger-defs-r2-sonnet-output.md`（67ee4e34），攻方 `research/prompts/d137-ledger-defs-r2-opus-output.md`（5f6a32b5），攻方模型 `research/prompts/d137-ledger-defs-r2-opus-model/`。被判文件：`.claude/agent-common.md`、`.claude/agents/clerk.md`、`.claude/agents/experiment-designer.md`、`.claude/agents/experiment-runner.md`、`.claude/agents/implementation-writer.md`、`.claude/agents/mutation-triage.md`、`.claude/agents/three-way-attack.md`、`.claude/agents/tooling-writer.md`、`.claude/main-agent.md`、`.claude/rules/verification.md`、`.claude/rules/implementation-workflow.md`、`.claude/skills/crash-test/commit-stages.md`、`.claude/skills/crash-test/SKILL.md`、`scripts/hosts/compile-then-swap.py`、`scripts/gate/merge-and-verify.sh`、`.claude/gate.d/doc-experiments.sh`。

## 核查怎么做的

- 开工快照倒推（任务 4929）：清单 16 份，倒推出 15 份、sha256 对上 15 份；`.claude/main-agent.md` 倒推不出——快照之后主 agent 把它第 101 行特性名的反引号换成「」（门禁 code-tooling 的 agent-definition-shape 格红在那一处，任务 4927 复跑绿），两条腿引的是它第 32、55 行，不落在改动的那一行上。
- 引文：两条腿各自跑过 `scripts/agents/cite-check.py`，辩方「核了 27 处引文…对上 27 处」，攻方「核了 39 处引文…对上 39 处」；下面「采」的每一条由主 agent 在主工作区现查，命令与结果写在那一条里。
- 攻方模型：`tools/rerun.sh`（任务 4930）造样本与变异体；`tools/judge-worlds.sh`（任务 4932）三行 rc 为 77、0、1，与 `out/expected.txt` 第二节逐行相同，对照（注释不以反斜杠收尾）判红；`tools/cli-features-probe.py` 照真脚本（任务 4931）与照变异体（任务 4933）的每一行与 `out/expected.txt` 第三、四节逐行相同。

## 逐条判

| 条 | 腿 | 判 | 主 agent 现查 |
|---|---|---|---|
| `experiment-designer.md` 第 31 行末句仍说门禁按 `crash_amplification::` 认判段（攻 B1、辩「跟改漏掉」） | 两腿 | 采 | 那一句此刻在，门禁已改成按 `JUDGE_ENTRIES` 认 |
| 入库装置带 gpu 编好后只跑裸二进制，服务不认它要卡，二进制在没清场时自己照多机配置挑卡（攻 B2；辩 A1「另一半判决没写去向」） | 两腿 | 采，交做统一任务服务的会话 | `crates/singlefs-checker-tier/src/gpu_card_configuration.rs:394` `(None, GpuQuiesce::NotQuiesced) => configured_value(CARDS_BY_PRIORITY_KEY)`；服务按文件头 `# task-server-gpu-arguments:` 与入口名单认要卡，裸二进制不在里面 |
| 执行员第 4c 步在任务日志里数 NOT_RUN 没写 `task log <号> --all`，默认只给最后 200 行；目录产物 `grep -c` 退 2（攻 B3、辩 A4） | 两腿 | 采 | `task-server/task` 第 58 行 `LOG_TAIL_LINES = 200`，第 25 行默认最后那么多行 |
| 续行并接把以反斜杠收尾的注释行与下一条命令并成一条，那一条不判（攻 B4）；围栏外续行不并、`-Fgpu` 与 `--features "verdict-store gpu"` 误红、只点名 `--test` 的 `cargo test` 不判（辩 A3） | 两腿 | 采 | 攻方世界三行 77、0、1 与期望相同 |
| 特性写法三处不一致：门禁认 `--features=`、`-F`、写几次、`--all-features`，`compile-then-swap.py` 前几种退 2、写两次只留后一个（攻 B5、辩 A1） | 两腿 | 采 | 探针 `features-twice-gpu-first rc=0 … gpu_in_every_call=no`，`features-equals-gpu`、`short-flag-gpu`、`all-features` 都 rc=2 |
| `compile-then-swap.py` 新自证项直接调 `swap()`，丢掉命令行 `--features` 的变异体自证照样全过（攻 B6） | 攻方 | 采 | 变异体探针 `features-gpu … gpu_in_every_call=no`、`selftest_in_process rc=0` |
| `merge-and-verify.sh` 读不出产品 GPU 包名照跑的那一支没有自证与开关（攻 B7）；读到的名字不是工作区成员时闭包为空、判「没碰」，与读不出照跑方向相反（辩 A10） | 两腿 | 采 | 照两腿各自引的行改成同一方向：读不出或不是成员都照跑并点名 |
| `.claude/main-agent.md` 第 55 行验证段第 1 步没补产品 GPU 包（攻 B8） | 攻方 | 采 | `sed -n 55p` 里「产品 GPU」「background-gpu」0 处 |
| `.claude/rules/verification.md` 第 63 行末句仍说门禁那一格「里面自己经 `gpu-quiesce-run.sh` 起」（攻 B9）；第 35 行没写产品 GPU 那一格「跑到加复用 0 条也报本次未跑」（辩 A11） | 两腿 | 采 | 第 63 行末句此刻在 |
| `commit-stages.md` 第 23 行靠驱动文件头 `*`，带 `--skip-gpu` 的那一片服务照样清场，与同一行「没卡或不清场的那一台加 `--skip-gpu`」相冲（攻 B10）；第 19 行重跑只列两格，门禁要卡处另有 crash-case-replay 的查库段（辩 5） | 两腿 | 采 | 照门禁 `.claude/gate.d/checker-tier-crash-point-replay.sh` 文件头写的要卡处与服务的认法改字句 |
| `mutation-triage.md` 第 15 行写分类以 `.claude/rules/mutation-sampling.md` 为准，那份第 79 行仍是六栏（攻 B11） | 攻方 | 采 | `grep -c 六栏 .claude/rules/mutation-sampling.md` 得 1 |
| 判段入口表没有一格核它与源码同步（辩 A2） | 辩方 | 采，记欠账 | 入口表此刻 13 个名字对得上源码；新增入口漏登记会漏判，要一格自证或门禁核同步 |
| 设计员范围声明括注没提不点名目标的整包 `cargo test` 也要声明（辩 A5） | 辩方 | 采 | 照 `.claude/hooks/run-scope-guard.sh` 文件头补一句 |
| 判决第一轮第 20 行文字与落地（三处必要加四处有条件）不同（辩） | 辩方 | 不采 | 落地比判决字面更准，判决原样保存，不回改 |
| 第一轮留作欠账的两样当时没登记（辩） | 辩方 | 采，已登记 | 2026-11-02 已用 `scripts/kb/ledger-append-section.py` 登进 `records/2026-10-29-subagent拆分提案.md` 一百八十节 |

## 改法与去向

- 定义与规则一件：`experiment-designer.md` 第 31 行、`experiment-runner.md` 第 40 行（`task log --all`、目录产物逐份数）、`main-agent.md` 第 55 行、`verification.md` 第 35、63 行、`mutation-sampling.md` 第 79 行、设计员范围声明括注。
- 脚本一件：`compile-then-swap.py` 认门禁认的全部特性写法并取并集、自证经命令行起；`doc-experiments.sh` 注释行不并、围栏外续行与 `-F`、带引号多特性照认、只点名 `--test` 的照判；`merge-and-verify.sh` 读不出与不是成员都照跑并点名，配自证与开关；入口表与源码同步那一格（另立，同一件里加）。
- skill 两处由主 agent 改：`commit-stages.md` 第 19、23 行。
- 裸二进制要卡（B2）交做统一任务服务的会话：服务认得出入库装置二进制要卡，或给出这类起法的写法。
- 改完开第三轮（最终轮），只攻前两轮站住的形态。

## 回看决策

不涉及决策：这一轮判的是 agent 定义、规则、skill 的字句与配套脚本，没有动 `.claude/kb/decisions/` 里的任何分项。
