# d137-ledger-defs-r1 判决（2026-11-02）

被判：`research/prompts/_d137-ledger-defs-r1-body.md` 第一节六条句子改动。腿：正推 `research/prompts/d137-ledger-defs-r1-sonnet-output.md`（sha256 前 8 位 f11399ee），攻方 `research/prompts/d137-ledger-defs-r1-opus-output.md`（43778285），攻方模型 `research/prompts/d137-ledger-defs-r1-opus-model/`。

## 核查怎么做的

- 开工快照倒推：`task-server/task submit --cwd ~/.local/share/singlefs/work/d137-ledger-defs-r1-main -- python3 /home/fy5090/code/singlefs/scripts/agents/snapshot-tree.py d137-ledger-defs-r1 --out ~/.local/share/singlefs/work/d137-ledger-defs-r1-main/d137-ledger-defs-r1-tree --repository /home/fy5090/code/singlefs`（任务 4769）末行「✓ 快照 d137-ledger-defs-r1：清单点名 11 份，倒推出 11 份，sha256 对上 11 份」；11 份此刻工作区那一份与快照相同。
- 引文：`python3 scripts/agents/cite-check.py research/prompts/d137-ledger-defs-r1-sonnet-output.md research/prompts/d137-ledger-defs-r1-opus-output.md --root ~/.local/share/singlefs/work/d137-ledger-defs-r1-main/d137-ledger-defs-r1-tree --background research/prompts/_d137-ledger-defs-r1-body.md` 末行「✓ 核了 49 处引文、99 处裸行号，对上 17 处，没判 131 处」，对不上 0 处；没判的 93 处是快照树之外的脚本与门禁（快照只收被判的 11 份），38 处是没有引文可比的裸行号。这两类逐处人工核：下面「采」的每一条都由主 agent 在主工作区现查过，命令与结果写在那一条里。
- 攻方模型：`task-server/task submit -- bash research/prompts/d137-ledger-defs-r1-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/code/singlefs --work ~/.local/share/singlefs/work/d137-ledger-defs-r1-main/rerun-1`（任务 4770），结局见「攻方模型复跑」一节。

## 逐条判

| 条 | 腿 | 判 | 主 agent 现查 |
|---|---|---|---|
| 入库装置经 `compile-then-swap.py` 编、不带特性，判段只打 NOT_RUN（正推 1、攻方 A1） | 两腿都打中 | 采 | `scripts/hosts/compile-then-swap.py:300` 的 cargo 参数是 `cargo build --offline -p <包> --bin <名> <档>`，没有 `--features` |
| 「源文件里有 `crash_amplification::` 或 `crash_state_enumeration::`」比「走到判段」宽，别名导入认不出（A2） | 攻方 | 采 | `grep -lE 'crash_amplification::|crash_state_enumeration::'` 命中 e170、e184 等装置；它们进不进判段没逐个核，条件改成按判段入口认 |
| 跑前登记那一格按单行认命令，反斜杠续行漏判或误红（A3） | 攻方 | 采 | 见「攻方模型复跑」 |
| `grep -c NOT_RUN <产物>` 数不到另写的文件与 `.gz`（A4） | 攻方 | 采 | 见「攻方模型复跑」 |
| 设计员「整条写成 submit」那句没带范围声明（A5） | 攻方 | 采 | 范围声明闸写法照 `.claude/hooks/run-scope-guard.sh` 文件头，`cargo test` 点名的包与目标要带 |
| `implementation-writer.md:47`「五处」：两处有条件、漏 must-red 与崩溃放量 ITEMS（正推 2、攻方 A7） | 两腿都打中 | 采 | 第 47 行照字面列五处、不写条件 |
| `commit-stages.md` 第 19 行一带的出路只重跑 feature-gated-tests 一格（正推 3、攻方 A8） | 两腿都打中 | 采 | 那一句前半说两格要卡、出路只给一格 |
| `.claude/main-agent.md` 第 32 行仍写「crash-case-replay、feature-gated-tests 两格」，第 101 行与 `.claude/skills/crash-test/SKILL.md` 第 21 行「快档两趟」没补那一格（正推 3.2、攻方 A9） | 两腿都打中 | 采 | `sed -n 32p .claude/main-agent.md` 命中「crash-case-replay、feature-gated-tests 两格」 |
| `merge-and-verify.sh` 只看 checker 档包依赖闭包，只改产品 GPU 包的补丁判「没碰」，新那一格恰好不跑（A10） | 攻方 | 采 | `crates/singlefs-checker-tier/Cargo.toml` 里 `singlefs-background-gpu` 0 处；`scripts/gate/merge-and-verify.sh` 第 636–646 行只认 checker 档包与它的闭包 |
| 「没卡报本次未跑」把服务没交卡（判红）与测试找不到显卡（本次未跑）并成一种（A11） | 攻方 | 采 | 照门禁那一格现行判法改字句 |
| 分诊定义只认 `NOT_RUN_NO_GPU_ADAPTER` 一种本次未跑；「六栏」与 `mutate.sh` 收尾「计数：」行九栏对不上（A12；正推 4 判成立） | 两腿分歧 | 采后一半 | `scripts/mutation/mutate.sh` 收尾那行是抓到、没红、无效、没跑完、抓名字有盲区、锚点失效六栏，末尾另接现判、复用、拒算三栏，字句照九栏写清；前一半（另一种本次未跑）在 `scripts/mutation/crates-mutation-rows.py` 里，那份此刻别的会话在改，等它落地再跟 |
| `agent-common.md:98` 删半句后同一句仍写「分两种出路」，派发闸现分三条（正推 5、攻方 A6） | 两腿都打中 | 采 | `sed -n 98p .claude/agent-common.md` 命中「分两种」 |
| 四份 read-once 规则点名与定义相冲：`pushback-discipline` 要写 `.claude/warnings/` 攻方腿写不了；`machine-first` 的兼容条与共用约束「不兼容任何旧东西」相反；`clerk.md` 同一句先说只读三处后说另读（正推 6 核不动、攻方 A13） | 攻方打中、正推核不动 | 采前三样 | 点名改成按各定义的活取用到的那一节（不整份点名），`clerk.md` 那一句理顺；另三份只在共用约束括注里提到的规则不在这一轮射程，记欠账 |
| 正推 1 的小差异（定义写 `NOT_RUN`、门禁出路写 `NOT_RUN_`） | 正推 | 不采 | 定义的写法数得更宽，不漏判 |
| 正推 2 说工具员报告里 `crash-case-check.py:25` 指不存在小节对不上 | 正推 | 不采 | 那三处已由跟随改动那一件改成现存小节名（`~/.local/share/singlefs/work/d137-b19-follow-ups/result.md` 第 7 条） |

## 攻方模型复跑

任务 4770 造出七个世界（日志每个世界一行 `name=world_built … untracked=1`），任务 4870 照攻方模型的 `tools/judge-worlds.sh` 逐个判：七行 rc 依次为 1、0、77、1、1、0、0，与 `research/prompts/d137-ledger-defs-r1-opus-model/out/expected.txt` 第二列逐行相同；两个对照（单行没带 gpu 的红、单行带 gpu 的绿）是 1 与 0，这一批作数。坐实：续行把 `--bin` 拆到下一行时门禁一条都没判、退 77，把 `--features gpu` 拆到下一行时门禁误红（A3）；只取崩溃点、不进判段的装置照定义字面与门禁都要 gpu，`use crash_amplification as …` 之后调判段的门禁当不走判段放过（A2）。A4 的 `.gz` 那一半没进世界：GNU grep 不解压、数到 0，是工具本身的行为，照采。

## 改法与去向

采的各条并成一批交工具员改：定义与共用约束（`.claude/agent-common.md`、`.claude/agents/experiment-designer.md`、`.claude/agents/experiment-runner.md`、`.claude/agents/implementation-writer.md`、`.claude/agents/mutation-triage.md`、`.claude/agents/three-way-attack.md`、`.claude/agents/tooling-writer.md`、`.claude/agents/clerk.md`、`.claude/main-agent.md`），规则（`.claude/rules/verification.md`、`.claude/rules/implementation-workflow.md`），skill（`.claude/skills/crash-test/commit-stages.md`、`.claude/skills/crash-test/SKILL.md`），脚本（`scripts/hosts/compile-then-swap.py` 带登记的特性、`scripts/gate/merge-and-verify.sh` 把产品 GPU 包算进快档射程、`.claude/gate.d/doc-experiments.sh` 那一格认续行）。改完开第二轮，攻第一轮的修补本身。

## 回看决策

不涉及决策：这一轮判的是 agent 定义、规则与 skill 的字句和配套脚本，没有动 `.claude/kb/decisions/` 里的任何分项。
