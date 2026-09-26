# 治理文档核对第 3 轮的十五份定义改动：第一轮判决（2026-09-26）

正文 `research/prompts/_governance-defs-r1-body.md`；背景材料 `research/prompts/_governance-defs-r1-background.md`；附录二 `research/prompts/_governance-defs-r1-diff.md`；开工快照 `research/prompts/governance-defs-r1-snapshot/sha256sums.txt`（31 行，腿交齐后 `sha256sum -c` 全部对得上，腿跑着的时候没有文件被改）。

被判的 15 份：`.claude/main-agent.md`、`.claude/agent-common.md`、`.claude/agents/crash-verifier.md`、`.claude/agents/experiment-designer.md`、`.claude/agents/experiment-runner.md`、`.claude/agents/gate-triage.md`、`.claude/agents/implementation-writer.md`、`.claude/agents/kb-scribe.md`、`.claude/agents/mutation-triage.md`、`.claude/agents/sweep.md`、`.claude/agents/three-way-attack.md`、`.claude/agents/three-way-local-attack.md`、`.claude/agents/three-way-local-defense.md`、`.claude/agents/three-way-materials.md`、`.claude/agents/three-way-verifier.md`。

## 一、这一轮交了什么

| 腿 | 报告 | sha256 |
|---|---|---|
| 云端正推（Sonnet） | `research/prompts/governance-defs-r1-sonnet-output.md`（259 行） | df556f7e53926d4bbc6580d668bb4cee12a7e442688890f30817fc60471a26fb |
| 云端攻方（Opus） | `research/prompts/governance-defs-r1-opus-output.md`（364 行），模型 `research/prompts/governance-defs-r1-opus-model/`，复跑 `bash run-all.sh <草稿目录>` | 81bf7dde9941e9cd12064852fb216fbde225cca53b502e69063614cb10663302 |
| 核查员 | `research/prompts/governance-defs-r1-verifier-output.md`（153 行） | c405700c84ab796bad1dc6357976a000b4ee5ab82cd14c55f6b314b6879a37e5 |
| 本地攻方 / 本地辩方 | **缺席**：这个容器里 `~/code/ai-center` 不存在，`curl localhost:8200` 退出码 7；用户知情后选「走三方（缺本地腿）」 | — |

核查员：攻方腿 41 处引用与复跑 41 ✓；模型目录拷出后复跑，输出与 `run-all.out` 去掉 `/tmp` 路径后逐字节一致。正推腿 37 处 32 ✓、5 ✗，5 处都是行号写错（内容在主树别的行找得到），不影响它的判定方向。

## 二、跑前条款，各触发没触发

| 条款 | 触发没触发 |
|---|---|
| 一格判「站得住」⇒ 照留 | G4 以外每一格都有攻方打中，不整格照留；未被打中的那几处照留 |
| 一格判「站不住」⇒ 改那一处，判决标「被攻过零轮」 | 触发：A1、A4、A6、A9、A10 只在改后的写法上中；A3、A5、A7、A8 改前改后都中（不分辨），按正文第五节另记，但它们都是描述与行为不符、改法不改变行为，一并写回 |
| 删改丢了判据 ⇒ 补回 | 没触发：正推逐处看了 diff 的删除半句，没有丢今天仍成立的判据 |

## 三、逐条判决（主 agent 逐条现查过）

| 编号 | 格 | 打中了什么 | 现查 | 判决与改法 |
|---|---|---|---|---|
| A1 | G1 | 崩溃验证员在主工作区跑 54 号快档，工作区与暂存区在 `crates/ Cargo.toml Cargo.lock` 上不同时算出的键找不到 worktree 里写的全绿标记，判红，而同一批 `gate.sh --staged` 判绿 | 核查员复跑模型 8 种状态属实 | 站不住。崩溃验证员不再跑 54 号；54 号快档只在 `gate.sh --staged`（HEAD + 暂存区的树）里跑；`.claude/gate.d/stage-owners.tsv` 里 54 号改归门禁分诊员；crash-verifier.md、main-agent.md、agent-common.md、`.claude/rules/implementation-workflow.md` 同步，`heavy-test-guard.sh` 拒绝时的出路同步 |
| A2 | G1 | 全绿标记的键还含 54 号脚本自己的 sha256 与 `cargo -V`、`rustc -V`，主 agent 起全量的条件只看登记路径 | `54-layer0-replay.sh:25、176、191` 属实 | 改：main-agent.md 的触发条件补「54 号脚本本身或工具链」 |
| A3 | G2 | 实验执行员第 5 步 `bash research/scripts/replay.sh E<号>` 被 heavy-test-guard 拒 | 主 agent 用合成 JSON 喂钩子：旧写法退 2，经 `run-with-memory-cap.sh 4G` 的写法退 0 | 改：第 5 步写成经内存包装的命令 |
| A4 | G2 | 共用约束只写了内存包装退 250 怎么办 | `run-with-memory-cap.sh` 文件头退出码表属实 | 改：补 251、252、254 各怎么办 |
| A5 | G3 | 样本号已被占时 `>` 会整份盖掉干净样本 | 属实（改前也中） | 改：第 3 步先 `ls` 取下一个没用过的号 |
| A6 | G3 | 「共用约束开着 noclobber」与实测不符 | 属实 | 改：删掉这句理由，写「`>|` 在开没开 noclobber 时都写得进」 |
| A7 | G5 | 照 `✅ [抓到]` 字面数，真实输出上得 0；`[ ]` 里是变异名；💥、⚠️ 两种结局没处放 | `mutate.sh:473、483、504、514、534、536、538` 属实 | 改：mutation-triage.md、`.claude/rules/mutation-sampling.md` 改成按行首符号数，列全七种结局；experiment-runner.md 报告指到那一段 |
| A8 | G4 | 核查员第 2 步没有「文件不在快照清单里」这一支 | 核查员自己确认同一批 8 个文件 | 改：补这一支（对主树核、用 git log / status 现查） |
| A9 | G6 | relabel 改写 `crates/` 下的源码后 `crates/mutations.tsv` 锚点失效、33 号红，main-agent.md 把它派给只许追加的实验执行员 | 核查员复跑属实 | 改：33 号红在 research 表的派实验执行员，在 `crates/mutations.tsv` 的派实现员走代码轮 |
| A10 | G6 | 检出钩子拒绝时的出路仍教「`a & b & wait`」 | `bash-command-detector.sh:2366、2377` 属实；另查到：两个 `&` 后逐个 `wait "$pid"` 会被这个钩子记成没等齐（检出记录实测），只认不带参数的 `wait` | 改：共用约束与钩子两句出路统一成「每个作业把退出码写进自己的文件、不带参数的 `wait` 等齐、逐个读」，这种写法钩子放行且不记检出（实测）；钩子对逐个 `wait "$pid"` 的误记属钩子判法，交用户 |
| 正推开放点 | G1 | main-agent.md 抄了一份 54 号出路里的命令（`16G`），两处各存一份 | 属实 | 改：main-agent.md 只指向 `print_staged_worktree_full_commands`，不抄命令 |

写回之后：`heavy-test-guard.sh --selftest` 通过（562 种），`bash-command-detector.sh --selftest` 通过（430 种），62 号绿。**写回的每一处都被攻过零轮**，由用户要的两轮严查接着攻。

## 四、交用户的

| 项 | 为什么交用户 |
|---|---|
| `heavy-test-guard.sh` 仍放行崩溃验证员跑层 0 | 按定义它已不跑 54 号；收紧要改钩子行为与 5 条以上自证用例，超出这一轮「只改描述」 |
| `bash-command-detector.sh` 把「起了不止一个 `&`、逐个 `wait "$pid"`」记成没等齐 | 与共享规则 `command-safety.md`「并行不许把失败吃掉」推荐的写法相反，改要动钩子判法 |
| 草稿目录写死 `/tmp/claude-1000/` | 用户已定改成按 uid 取，另起第 4 轮 |

## 回看决策

不涉及决策：这一轮判的是 agent 定义、共用约束与两个钩子的出路文字，没有动任何一条决策分项的定案、射程或依据，也没有新增或撤回实验结论。
