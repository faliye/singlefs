# defs-gate54-tiering-r1 云端正推（Sonnet）报告

立场：从用户定案（`records/2026-09-19-里程碑二遗留收拢.md:147`）与已有规则推，K1–K3 三格逐格判「改动是不是它们的直接后果」。全部引文已用 `grep -nF` 在被引文件里核过一次，行号是核出来的行号。

## K1　实验执行员的写范围加 `crates/mutations.tsv`

**改动**：`.claude/agents/experiment-runner.md:39`（写范围段）与 `.claude/hooks/agent-write-scope.tsv:13` 新增一行。

**是不是用户 2026-09-19 定案（Q8）的直接后果**：不是。`records/2026-09-19-里程碑二遗留收拢.md:147` 原话：

> 「| 8 | 全量测挂在哪个节点 | 原话：「每次主 agent 执行完任务后统一执行」 | 收口表第 41 行：层 0 全量不再每次门禁都跑，改在主 agent 一个任务收尾时统一跑一次；门禁 54 号怎么分档要改（快档每次、全量在收尾），改之前先写清「任务收尾」是哪一步（阶段同步那一步） |」

这一整行只谈层 0 全量测（门禁 54 号）挂在哪个节点，一个字没提实验执行员、`crates/mutations.tsv` 或写范围表。**缺哪一句**：定案里没有一句能推出「实验执行员该不该写 `crates/mutations.tsv`」。正文一「门禁 54 号 2026-09-23 照这句分了档，连带三处定义要改」这句「连带」在 K1 这一格站不住——K1 与 54 号分档在内容上互不相干，只是被同一批三方一起判。

**是不是规则的直接后果**：这条改动的直接依据是 `.claude/hooks/agent-write-scope.tsv:13` 自己写的事件记录：

> 「experiment-runner	crates/mutations.tsv	入库装置的变异行只追加进末尾（experiment-runner.md 第 2 步与「写范围」；2026-09-23 E158 撞上：闸拒了那次追加，三行落进 research/mutations/ 下、门禁 59 号复跑不到）」

这不是用户定案，也不是三方论证正文点名的既有规则，是一次实现事故（E158）的补丁；把它纳入这一轮三方，唯一站得住的规则依据是 `.claude/rules/implementation-workflow.md:16`「改 agent 定义与共用约束，走同一条三步」（凡改 `.claude/agents/`、`.claude/agent-common.md`、`.claude/main-agent.md` 都要走三方），这条只管「要不要走三方」，不管「这处改动本身对不对」。

**验证过的既有闸管不到「只追加」**：`.claude/hooks/write-guard.sh:93` 的 `decide_scope` 只按 `target = absolute if pattern.startswith("/") else relative` 取路径、再用 `glob_to_regex(pattern).match(target)`（同文件第 94 行）整段匹配，不区分 Write 整份覆盖与 Edit 只追加最后一行——定义里写的「只追加进末尾，不改别人的行」这句约束，闸判不出来（这是我现读代码核实的，不是转述背景材料）。`.claude/gate.d/63-agent-write-scope.sh:7` 判的只是「这个 agent 在表里有没有至少一条模式」，同样判不出「只追加」。

结论：**这一格判「规则没说」**——没有一句用户定案覆盖它；规则层面唯一的直接依据是「改定义要走三方」这条元规则（管走不走三方，不管对不对），它是否该被采纳要看 K1 那格的攻防（本地攻方腿的活），不由这份报告判。

什么现象会推翻这条判断：如果日后在 `records/` 或别的用户定案里找到一句明确提到「实验执行员」或「`crates/mutations.tsv` 写范围」，就要改判「是」。

## K2　崩溃验证员跑 54 号带 `--full`

**改动**：`.claude/agents/crash-verifier.md:24`：

> 「2. 按共用约束 `.claude/agent-common.md`「门禁」一节从阶段归属表取登记给你的阶段，一次只跑一个：`nice -n 19 bash .claude/gate.d/<文件>`，54 号带 `--full`（不带只跑快档）；……」

**与用户定案冲突**：Q8 的收口结论是「层 0 全量不再每次门禁都跑，改在主 agent 一个任务收尾时统一跑一次」（引文同 K1，`records:147`）。但崩溃验证员按 `.claude/main-agent.md:40` 的派发次序：

> 「| 改 `crates/`：条款已定、验收标准写得清、改动有界（照已定分项实现、补测试与变异、修原因已知的红） | `implementation-writer`（后台派，主 agent 审 diff）→ 上一行的三方（代码轮）→ `crash-verifier`（层 0、QEMU、herd7、crates 变异表） |」

是在**每一次**「改 `crates/`」的代码轮之后被派发，不是「一个任务收尾时」才派发一次；一个阶段任务里可能有多次「改 `crates/`」（多次 implementation-writer → 代码轮 → crash-verifier），K2 让 crash-verifier 每次都带 `--full`，等于把「全量」重新钉回 Q8 明令要撤掉的「每次都跑」。`--full` 这条路径本身不做复用短路——`.claude/gate.d/54-layer0-replay.sh` 里「能不能复用」「碰没碰 crates/」两问都包在 `if [[ "$layer0_tier" == quick ]]` 分支里，`--full` 一律真跑一遍全量，不是空跑。

两边原文并排：

| 用户定案（Q8，records:147） | K2 改动（crash-verifier.md:24 + main-agent.md:40） |
|---|---|
| 层 0 全量不再每次门禁都跑，改在主 agent 一个任务收尾时统一跑一次 | crash-verifier 每次代码轮之后都带 `--full` 真跑一遍全量，不是一个任务收尾时才跑一次 |

结论：**这一格判「冲突」**——K2 改动与它自称要落实的那句用户定案在触发频率上直接矛盾。

**规则层面另一个缺口**：`crash-verifier.md:23`（本次未改，diff 里原样保留）：

> 「1. 每个阶段开跑前各照共用约束「不做」一节看一次负载；另有别的 `gate.sh` 正在跑 54 号时，不起 54 号，等它结束。」

这句碰撞检测只挡得住「另一个 `gate.sh`」，而 K3 让主 agent 在阶段同步那一步直接跑 `bash .claude/gate.d/54-layer0-replay.sh --full`（`.claude/main-agent.md:43`），这条命令不经过 `gate.sh`，字面上不是「别的 `gate.sh`」。我在 `.claude/gate.d/54-layer0-replay.sh` 里 `grep -n '正在跑\|已有\|lock\|flock\|ps -'` 零命中——脚本自己也没有互斥锁。**缺哪一句**：`crash-verifier.md:23` 没有覆盖「主 agent 直接跑 54 号脚本本身（不经 `gate.sh`）」这一种碰撞源，这一半推不出「K2 与 K3 不会撞车」，只能推出「碰撞检测这句话的射程窄于 K3 引入的新触发路径」。

什么现象会推翻「冲突」这条判断：如果查到用户定案或规则明确说「crash-verifier 每次跑的 `--full` 允许被短路成复用」，或者查到 crash-verifier 实际只在「一个阶段任务」里最多被派发一次，这条冲突就不成立。什么现象会推翻「碰撞检测缺口」那句：如果查到 `--full` 内部或 `gate.sh` 之外另有一层互斥（比如文件锁），这条缺口就不成立。

## K3　主 agent 收尾跑 54 号全量

**改动**：`.claude/main-agent.md:43`（「什么时候派哪个 agent」表「一个阶段任务结束」一行）：

> 「……→ 主 agent 写 `research/prompts/<阶段>-sync.md`（格式见门禁 68 号文件头，68 号判形式）→ 这一批碰了 `crates/` 就后台跑 `bash .claude/gate.d/54-layer0-replay.sh --full`（层 0 全量只在这一步跑，判绿写全绿标记，整轮门禁的 54 号只核这个标记）→ 下一行」

**核心落点是不是用户定案的直接后果**：是。Q8 原话「每次主 agent 执行完任务后统一执行」，其收口结论额外写明「改之前先写清『任务收尾』是哪一步（阶段同步那一步）」（引文同 K1，`records:147`）。K3 把触发点放在「一个阶段任务结束」这一行，即正文自己框定的「阶段同步」那一步，与用户定案逐字对得上：**落点一致**。

**触发条件与规则冲突**：K3 的触发条件写的是「这一批碰了 `crates/` 就……」（同上，`main-agent.md:43`），但 54 号实际登记的输入集是三项，`.claude/gate.d/stage-inputs.tsv:11`：

> 「54-layer0-replay.sh	crates/ Cargo.toml Cargo.lock	# 跑 crates 的崩溃点重放；两条流的用例编译期与运行期都不读 .claude/kb/layout/ 与 research/results/（段序列与布局表的比对归 52 号）」

而 `.claude/rules/implementation-workflow.md:32`「复用上一次全量门禁的判定」一节明令：

> 「改动之后要不要重跑某一道，判据是**那一道读的全部输入自上次跑绿以来变没变**，不是「我改的是不是 `crates/`」。」

两边原文并排：

| 规则（implementation-workflow.md:32） | K3 改动（main-agent.md:43） |
|---|---|
| 判据是那一道读的全部输入变没变，不是「我改的是不是 `crates/`」 | 触发条件写成「这一批碰了 `crates/` 就……」，只字未提 `Cargo.toml`／`Cargo.lock` |

这处冲突不是假想：一批只改 `Cargo.lock`（比如升级一个依赖版本、不碰任何 `crates/` 下的文件）时，54 号的输入哈希（按 `stage-inputs.tsv:11` 登记的三项算）已经变了、旧的全绿标记对不上新状态，但 K3 这句触发条件不会为它后台跑 `--full`——阶段同步这一步会跳过全量，等到下一次整轮门禁跑快档时才会因为标记的输入哈希不等而判红（`.claude/gate.d/54-layer0-replay.sh:219` 的 `marker_input_hash != layer0_input_hash` 分支）。红本身不算「不该红的红了」，但「该跑全量的没跑」在阶段同步这一步确实成立——这属于正文五「跑前条款」第二款描述的形状（「攻方给出一条可达历史，在上面该跑全量的没跑……」），值得攻方腿（Opus）拿这条具体历史去实测。

结论：**这一格「核心落点」判「一致」，「触发条件」判「冲突」**——落点（阶段同步那一步）与 Q8 逐字对得上；触发条件字面窄于 54 号自己登记的输入集，与 implementation-workflow.md「不按『改了哪个目录』判」这条规则相悖。

什么现象会推翻「落点一致」：如果查到用户后续改口，要求全量测挂在别的节点，这条一致就不成立。什么现象会推翻「触发条件冲突」：如果查到「碰了 `crates/`」在这条工作流里其实是包含 Cargo 文件的简称而非字面路径前缀——但我 `grep -n '碰了.*crates' .claude/main-agent.md` 只命中这一处，没有旁证支持这种读法，暂判冲突成立。

**staged/worktree 与并发覆盖两问，规则没说**：正文三·K3 还问「工作区与暂存区不同时……对不对得上」「几个会话各跑一次会不会互相覆盖」。这两句在用户定案与既有规则里都没有对应的句子：Q8 只定了「挂在哪个节点」；`.claude/singlefs-ai-sop/rules/session-wrapup.md:53`「几个会话同时在一个仓里干活时」整段讲的是暂存区、编号与新建文件的排他写，没有一句谈「git common-dir 里一个跨 worktree 共享的标记文件」这种情形。**缺哪一句**：没有可引的既有规则覆盖「多会话共写同一个 common-dir 标记文件」；这条走不走得通只能靠实测（Opus 攻方腿的活），这份报告在此判「规则没说」。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1 | 规则没说 | Q8 与它无关；唯一依据是「改定义走三方」这条元规则（管走不走三方，不管对不对），闸本身判不出「只追加」 |
| K2 | 冲突 | crash-verifier 每次代码轮后都跑 `--full`，与 Q8「全量不再每次都跑、改一个任务收尾统一跑一次」直接矛盾，且碰撞检测只挡「别的 `gate.sh`」挡不住 K3 直接调用 54 号脚本 |
| K3（落点） | 一致 | 触发点放在「阶段同步」这一步，与 Q8 收口结论逐字对得上 |
| K3（触发条件） | 冲突 | 「碰了 `crates/`」窄于 54 号登记的三项输入，与 implementation-workflow.md「不按改了哪个目录判」相悖 |
| K3（staged/并发） | 规则没说 | 没有既有规则覆盖多会话共享 common-dir 标记这种情形 |

## 没做什么

- 不判 K1、K2 的本地攻方（Qwen）与 K3、K2×K3 撞车的云端攻方（Opus）该判的格，那两条腿的问题不在这份报告里重复判。
- 没有真跑任何门禁阶段、没有真跑 54 号 `--full` 或快档；全部引用的是仓里现有脚本与文档的原文（已用 `grep -nF` 逐句核对行号），没有新造实验或产物。
- 没有替主 agent 采纳或出判决，只交每格的正推结论。
- K2 与 K3 撞在一起那一格（正文分工表点名给 Opus）没有构造可复现的命令序列去实测，只在文本层面指出碰撞检测的射程缺口；实测交攻方腿。
- 没有检查 `crates/mutations.tsv`、`research/mutations/` 目录今天的实际内容（E158 事故是否已经清理），只核了定义与 tsv 文本本身。
- 背景材料「二、实现今天的样子」第 23 行提到的文件名 `.claude/hooks/agent-write-scope.sh` 在仓里已不存在（2026-09-17 与 `refuse-overwrite-untracked.sh` 合并改名为 `.claude/hooks/write-guard.sh`，该文件头 13 行自述此事）；这份报告在 K1 里改引现存的 `write-guard.sh`，没有回改背景材料本身。
