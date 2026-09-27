**出处 `.claude/rules/implementation-workflow.md:6-15`（整段抄，未转述）**

```markdown
## 三步，缺一步就不算做完

| 步 | 做什么 | 谁在判 |
|---|---|---|
| 1 写代码 | `crates/` 下的改动带测试，每条新测试先证明会红（`.claude/singlefs-ai-sop/rules/show-me-test.md`） | show-me-test 阶段判「有没有测试」；会不会红写在 commit message 里 |
| 2 多 agent 正反对抗推理 | 改完的代码走一轮三方：材料带上 diff 与它压着的条款，正推腿核「代码做的是不是条款说的」、反推腿攻「哪一格会错」、本地腿找反例；打中的写回代码，再攻一轮 | 门禁 56 号判形式：这次改动里每个 `crates/*/src/*.rs` 都要在同一次改动带来的 `research/prompts/*-main-verification.md` 里被按路径点名。点名了判得对不对，要人看 |
| 3 checker 测试 | 池级 checker、层 0 崩溃点重放、变异表——三方打中的每一条先做成会红的量再改；每一处改法在 `crates/mutations.tsv` 留一条「改回去它就红」的变异 | 54 号（层 0 全量）、59 号（crates 变异表复跑）、33 号（research 的变异表）、`check.sh` |

**次序不能倒**：三方攻的是写好的代码与它的测试，攻在计划上的那一轮（里程碑那份）替代不了这一轮。

```

**出处 `.claude/rules/implementation-workflow.md:16-23`（整段抄，未转述）**

```markdown
## 改 agent 定义与共用约束，走同一条三步

**定义是工作流，不是说明**：`.claude/agents/`、`.claude/agent-common.md`、`.claude/main-agent.md` 只写怎么做——步骤、约束、判据、交付。为什么与是什么归 kb 自己管（`.claude/kb/`、`records/`），定义里不写、也不为它留解释的尾巴；要用到那些东西时，写成一条指令：「开工先读 `.claude/kb/<某份>` 的〈某节〉」。上游门禁的「规则纪律（项目本地）」阶段判这一条（rules-lint 扫这三处）。

**改它们与改 `crates/` 同规矩**：写完要走一轮三方，判决落 `research/prompts/<轮>-main-verification.md` 并按路径点名改过的每一份定义，门禁 72 号判形式（形态照 56 号）。

**三步在定义上各取什么**：第 1 步不适用——定义是文档，`.claude/singlefs-ai-sop/rules/show-me-test.md`「没有测试的 patch 一律不收」只管 `crates/*/src/`，只改文档和脚本除外；第 2 步就是「改它们与改 `crates/` 同规矩」那一轮三方；第 3 步照 `show-me-test.md`「新增的测试必须先证明它会红」里「这条同样管设计论证」那一句：三方打中的每一条，先在模型里做成一个必须报非 0 的世界，再改定义。

```

**出处 `.claude/rules/implementation-workflow.md:46-69`（整段抄，未转述）**

```markdown
## 重型测试只在提交时跑

**重型测试**：层 0 全量（`.claude/gate.d/54-layer0-replay.sh` 与 `--test` 目标名含 `layer0` 的测试）、QEMU（55 号、`vm-bench.sh`）、herd7（57 号、`.claude/scripts/lkmm.sh`）、`crates` 变异整表（59 号）、全量 `cargo test`（`--all`、`--workspace`、`check.sh`）、整轮门禁（`gate.sh`、`gate-staged.sh`）、全部实验复跑（87 号）、E152 装置。`.claude/gate.d/` 下 54、55、57、59、87 之外的阶段不是重型，谁都能跑。

| 场合 | 跑不跑 |
|---|---|
| 每次提交代码 | **必须跑**：主 agent 在提交流程里后台起（命令带 `SINGLEFS_HEAVY_TESTS=commit`），看门狗盯；git 的 pre-commit hook 照旧跑整轮门禁 |
| 用户当场要求，或任务确实要跑 | 任务确实要跑时主 agent 先弹窗问用户，用户同意了才跑；命令带 `SINGLEFS_HEAVY_TESTS=user-request` |
| 其余任何时候 | 不跑 |
| 崩溃验证员、门禁分诊员 | 各跑各的那一部分，只在提交时或用户要求时（命令带 `SINGLEFS_HEAVY_TESTS=commit` 或 `=user-request`）：崩溃验证员跑 54、55、57、59 号那几道；门禁分诊员跑门禁其余阶段并分诊，54、55、57、59 靠「输入没变就复用上一次全绿判定」不重跑。谁都不把整轮全量从头跑一遍 |
| 其余子 agent | **一律不跑**，只跑自己动到的测试二进制与 fmt / clippy / build |

由 `.claude/hooks/heavy-test-guard.sh`（Bash 的 PreToolUse）在执行前拒绝：其余子 agent 跑上面任何一样拒绝；崩溃验证员、门禁分诊员跑自己那一部分之外的、或不带那个环境变量前缀的拒绝；主 agent 不带那个前缀也拒绝。派发提示要子 agent 跑重型测试的，由 `.claude/hooks/runner-dispatch-guard.sh` 拒绝。

herd7 / LKMM 与 QEMU 不归上游 singlefs-ai-sop 管：相关的脚本、样本、模板与规则段落都不在 SOP 里，怎么测、怎么验、接不接进门禁由本工程自己定。两样都是本工程自己的阶段：

| 装置 | 阶段 | 判什么 |
|---|---|---|
| herd7 / LKMM | `.claude/gate.d/57-lkmm.sh`（逻辑在 `.claude/scripts/lkmm.sh`） | `litmus/` 下每条 Never 有对照组、绑到代码，herd7 判定与声明相符；缺 herd7 直接红，不静默跳过 |
| QEMU 真设备 | `.claude/gate.d/55-qemu-first-transaction.sh` | 两块 virtio 盘上跑固定负载，设备侧录制与程序录制流逐项比 |

两道都在 `gate.sh` 里，提交时跑整轮门禁就把它们带上了；单跑一道不算跑过门禁。
`--staged` 那条路（几个会话共写一个仓时）同样跑它们。

```

**出处 `.claude/singlefs-ai-sop/rules/command-safety.md:34-59`（整段抄，未转述）**

````markdown
## `pkill -f` / `killall` 一律禁用

那个模式串会出现在 wrapper 自己的命令行里，结果**把自己的 shell 也杀了**。

要停进程就先 `ps` 列出来看清楚，再用**写死的 pid** 另起一条命令去杀；
也可以交给 `scripts/proc.py stop 进程号`：它先发 TERM，过 `--grace` 秒（默认 10）还在再发 KILL，要停的是发出命令的进程自己或它的祖先时拒绝。
统计类的改用 `/proc` 里的结构化信息，并且排掉自己这一支进程树。

**同一个模式串放进等待循环里是另一种形态：不会误杀，会永远空转。**
`until ! pgrep -f "X"; do sleep 5; done` 这种等法，模式串命中的是这条循环自己所在的 shell 命令行，
`pgrep` 永远有命中、循环永远不退出，而且不报错——外面看到的只是「还在等」。
⇒ 等一个进程结束，用它**写死的 pid**：`until ! kill -0 "$pid" 2>/dev/null; do sleep 5; done`；
等自己起的后台任务用 `wait`；不是自己起的（别的脚本拉起来的守护进程之类），让起它的一方把 pid 写进文件再等。
pid 没记下来的，用 `scripts/proc.py find 可执行文件名 [--argument 参数]` 找进程号：它按可执行文件名逐字比，不拿模式串去比整条命令行，也不列发出这条命令的那一支进程；
拿到进程号再 `scripts/proc.py wait 进程号 --timeout 秒` 等它退出，到点还没退出就列出还活着的、退出码 3。
`scripts/shell-lint.sh` 的 S3 那条对脚本里命令位置上的每一处 `pgrep -f` 都红（`if` / `while` / `until` / `!` 后面也算命令位置），
不看同一行还有没有 `kill`：先赋给变量、下一行再 kill，或者只是数一数，模式串照样命中自己的命令行。
手敲的命令由 `scripts/claude-hooks/pattern-process-guard.sh` 在执行前拒绝，判据与 shell-lint 的 S2、S3 是同一份（`scripts/lib.sh` 的 `PATTERN_KILL_RE`、`PATTERN_PGREP_RE`），
拒绝时给出按进程号办事的写法。它是 Claude Code 的 PreToolUse 钩子，项目在 `.claude/settings.json` 的 `hooks.PreToolUse` 里注册一次：

```json
{"matcher": "Bash", "hooks": [{"type": "command", "command": "bash \"$CLAUDE_PROJECT_DIR\"/.claude/singlefs-ai-sop/scripts/claude-hooks/pattern-process-guard.sh"}]}
```

没注册的会话里，手敲的命令仍然只靠这一条文字。

````

**出处 `.claude/singlefs-ai-sop/rules/command-safety.md:60-75`（整段抄，未转述）**

```markdown
## 起了后台任务和子 agent，就要定时复检，不强制结束

长活可以等：编译、全量复跑、交给子 agent 的活跑上几个小时都正常。要防的是**没人看着的等待**：
等的条件永远不会成立，外面看到的只是「还在跑」。


所以：

- **起了就要复检。** 后台起的任务、派出去的子 agent，调度的一方定时看：最后一次动作是多久以前、是不是停在等待循环里、
  同一条命令是不是反复跑而输出一模一样、它写的文件还涨不涨。
- **检出和处置分开。** 检出卡住与空转的工具（hook、看门狗脚本）只记录、只报告，不拦命令、不杀进程、不停子 agent；
  结不结束、怎么处置，由调度的一方看了再定。工具替它判「超时就杀」，会把还在正常往前走的活一起杀掉。
  拒绝某一种危险写法的钩子不在此列（`session-wrapup.md` 第 4 条：写入前拒绝覆盖未跟踪的文件）。
- **等日志里的一行字之前，先确认那一行真会写进那个文件。**
- 在 Claude Code 这类会话里，两次模型调用隔得太久，提示缓存会过期、整段上下文要重新写入：

```

**出处 `.claude/singlefs-ai-sop/rules/rules-discipline.md:9-20`（整段抄，未转述）**

```markdown
## 1. 正文只写四样

| 写 | 不写 |
|---|---|
| 怎么做：步骤、次序、判据、阈值 | 为什么这么定：论证、取舍、推导 |
| 做什么、不做什么：射程与例外 | 这条怎么来的：实测、日期、谁定的、踩过的坑 |
| 推荐怎么做：拿不准时的默认选项 | 当时是什么样：旧值、旧做法、改动经过 |
| 什么规则在哪里：指到别的规则、脚本或 kb | 门禁判别力的经过与数 |

规则会被整篇读进每一轮工作的上下文。论证与经过挤在里面，执行的人和模型要先分辨
哪一句是命令、哪一句是背景，分辨错了就照着背景办事。

```

**出处 `.claude/singlefs-ai-sop/rules/rules-discipline.md:21-34`（整段抄，未转述）**

```markdown
## 2. 判据留下，论证搬走

这两样最容易混，分界只有一句：

| | 长什么样 | 处置 |
|---|---|---|
| **判据** | 照它能判出该不该做、做到哪算完 | **留在正文** |
| **论证** | 解释那条判据为什么成立 | **删掉** |

- 「删掉这句，读的人少知道什么？说不上来就删」——判据，留下。
- 「啰嗦的代价不是占地方，是让有用的话被埋掉」——论证，删掉。

同一段里两样都有时，把判据那一句留在正文，其余整段删掉，不留摘要。

```

**出处 `.claude/rules/three-way-inference.md:19-37`（整段抄，未转述）**

```markdown
## 各条腿必须互不重复

「三方」是这套流程沿用的名字；一轮派三条推论腿：云端攻方（Opus）、云端正推或云端辩方（Sonnet，一轮一条）、本地攻方或本地辩方（一轮一条，派这一轮缺的那一侧）。

| 腿 | 用什么 | 怎么调 |
|---|---|---|
| 本地攻方 | Qwen3-Next-80B-A3B-Thinking-AWQ-4bit，经 `~/code/ai-center` 网关（`:8200`，模型 id `local`） | `bash research/scripts/ask-local.sh <提示文件>`，**提示用英文写** |
| 本地辩方 | 同一个本地模型，另一份提示 | 同上 |
| 云端 A | Sonnet | Agent 工具，`model: sonnet` |
| 云端 B | Opus | Agent 工具，`model: opus` |

**「不能重复」指的是各条腿要拿到不同的切入角，不是只换个模型名。**
每条腿要被指定一个不同的立场（例如：正推 / 反推 / 找反例），
对齐 `.claude/singlefs-ai-sop/rules/evidence-discipline.md` 的三步。
这一轮派本地攻方时，它与云端攻方同为攻方，**两条攻方腿的攻击面要在正文的分工表里分开写**；写不出两组不重叠的攻击面，这一轮的正文就还没写完。
本地辩方替被判出局或被攻的一方辩护。本地腿按「一条腿只抽一次样不算一次观测」那一节抽样。

**本地腿只问能落成数、能逐格判的题。** 提示里给写死的事实表（每行在核对表里写来源文件与行号），要模型按表格逐格填，不许只答 yes / no。

```

**出处 `.claude/rules/three-way-inference.md:147-162`（整段抄，未转述）**

```markdown
## 一条腿只抽一次样不算一次观测——否定结论尤其不算

模型的答复是**有变化的观测**，因此 `.claude/singlefs-ai-sop/rules/test-discipline.md`
「单次观测不算数」那条对它成立：同一份提示、同一个模型，两次可以给出方向相反的答案。

⚠️ **两个方向的门槛不一样**，与那条规则同形：

| 结论 | 采信条件 |
|---|---|
| **打中了**（给出反例、指出矛盾） | 一次就值得去核。它是个线索，真伪由主 agent 现查坐实，抽样次数不改变这一步 |
| **没打中**（「构造不出反例」「没发现问题」） | **一次不算**。它与「这一轮它没想到」分不开，而两者在答复里长得一模一样 |

⇒ **做法**：拿一条腿的「没打中」去支撑任何结论之前，至少再抽一次；两次都没打中才记「没打中」，
两次不一致就照 `.claude/singlefs-ai-sop/rules/test-discipline.md` 记「不稳定」，不下结论。
云端腿同样适用——这条管的是「模型答复」这类观测，不是本机那条腿特有的。

```

**出处 `records/2026-09-16-subagent拆分提案.md:912-912`（整段抄，未转述）**

```markdown
| 1 | `.claude/agents/three-way-local-attack.md` | 「写范围」一节写「提示文件、核对表、样本文件……除此之外不写」，「产出」一节却要它写 `research/prompts/<轮>-local-attack-runlog.md`；两节互相矛盾，照写范围做就交不出运行记录 | 定义已改、待定义三方（2026-09-26 JST）：`three-way-local-attack.md`「写范围」一节加上「产出」一节的运行记录与草稿目录；`three-way-local-defense.md` 开头改成读本地攻方的「做什么」「写范围」「产出」三节，「文件名形态」一行写明两节里的文件名照它换 |
```

**出处 `records/2026-09-16-subagent拆分提案.md:940-940`（整段抄，未转述）**

```markdown
| 29 | 攻方腿的取样规模与建池方式没有规矩，每条腿各写一套、每段历史都在磁盘上重建大镜像 | 2026-09-25 m2-safety-r1 云端攻方腿（JST 13:12–16:47）194 次 Bash、没跑层 0；约 1.5 小时耗在重跑：自己的判别装置错一次、S2/S3 第二遍被磁盘读写拖到每个候选 70 分钟以上（主 agent 同时派了 E142 执行员抢同一块磁盘）。它的用例每段历史都在临时目录新建 4 GiB 稀疏文件、mkfs 清 journal 环（`research/prompts/m2-safety-r1-opus-model/tests/s_opus_s23.rs` 第 51 行注释），改成每个起点只建一次、内存里拷之后每个候选 3.3 分钟；S1 六个候选各在 4 GiB 盘上跑几千段，其中大部分格子与盘大小无关。harness 已有内存稀疏盘与录制写流、按崩溃点截断重放的装置（`crates/singlefs-harness/src/crash.rs` 的 `SparseBlockDevice` 与层 0 用的那一套），攻方腿没用。用户 2026-09-25 JST 19:2x 问「这种攻击方 需要每个在 4 GiB 盘和几种小盘上各跑几千段历史，每段都跑池级 checker 吗」「那么每次都重建镜像吗」 | ① harness 加公用工具：起点历史只建一次池、内存里拷；崩溃点用录制写流截断重放——改 `crates/`，等回退改形态那一轮判完并进实现员；② 攻方腿定义写明：用内存盘与这个工具、先跑一小段估时长超过 40 分钟就缩取样、崩溃点按那一串写穷举、checker 判结束状态、随机跑批小批量限时——定义已改、待定义三方（2026-09-26 JST，先于 ① 做）：`three-way-attack.md`「做什么」加第 3b 步，内存盘与崩溃点指今天已有的 `SparseBlockDevice` 与层 0 那一套，① 做出来之后再改指公用工具；「超过 40 分钟就缩」只缩历史、候选与几何，崩溃点照共用约束不缩；`three-way-local-attack.md` 不加（它不建池、不跑模型）。这一轮（m2-rollback-forward-r1）的攻方腿已在派发提示里写了限时与只建一次池 |
```

**出处 `records/2026-09-16-subagent拆分提案.md:941-941`（整段抄，未转述）**

```markdown
| 30 | 派发重活之前不判内存量：`research/scripts/run-with-memory-cap.sh` 只给每条定上限，门禁 59 号按可用内存收工作进程数，子 agent 直接跑 `cargo test` 没人拦 | 第 28 行之后，每条命令有了自己的上限，几条合起来没人管：各 agent 各经包装、各报各的上限，合起来照样能超整机；59 号「可用内存 ÷ 每条上限」只看得见开跑那一刻，看不见别的重活同时来抢；子 agent 不经包装直接跑 `cargo test`、`cargo run`、实验二进制，执行前没有东西拦。用户 2026-09-25 JST 19:4x：「进程数不是问题 问题是要派发脚本的时候防止过大了」「改进程数 治标不治本 要先判断内存量」，对主 agent 的方案回「对 你这个想法是对的 改吧」 | 已做（2026-09-25，前一个子 agent 做到一半会话断了，接手的子 agent 做完；报告 `research/prompts/process-safety-tmp-evidence/report.md`）：① 总量兜底：包装起的每个 scope 挂进 `singlefs-heavy.slice`，总上限 = MemTotal − 余量（`RUN_WITH_MEMORY_CAP_RESERVE`，默认 20G：那时 slice 外面回收不掉的合计 13.4 GiB，其中 vllm-prod anon + shmem 8.0 GiB、user.slice anon 2.4 GiB，其余是内核；`--status` 现量这几项），本机约 40.1 GiB；set-property / start 失败、cgroup 里读回的 memory.max 不对、余量比整机还大，都退 251、命令不跑，不退回无总上限。② 起跑前判内存量：同一把 flock 里算「slice 回收不掉的用量 + 账上放行了还没涨到量的 + 这一条要的」，不超过总上限才起，否则排队；等满 `RUN_WITH_MEMORY_CAP_WAIT_SECONDS`（默认 3600）退 252、列出占着的，要的量比总上限还大的不等、直接退 252。要的量取峰值表 `research/scripts/memory-peaks.tsv` 里这条命令上一次的峰值（scope 里的外壳在命令退出之后读 memory.peak，含页缓存；一行一条，文件头写口径），表里没有的按它的上限算。这张表不进 git（`.gitignore`）：59 号跑的时候每条都写它，跟踪着会让 gate.sh 开跑与收尾的工作区指纹对不上（`lib.sh` 的 `worktree_fingerprint` 用 `git add -A` 算），数也只在本机成立。slice 的用量减掉 active_file 与 inactive_file：结束了的 scope 的页缓存挂回 slice（实测 scope 里 dd 写 120 MiB 退出之后，slice 的 memory.current 还是 131 MiB）。顺带加两种结局：被总上限挤掉的退 254（scope 自己的 oom 计数是 0、oom_kill 不是 0），`RUN_WITH_MEMORY_CAP_TIME_LIMIT` 用 RuntimeMaxSec 限时、退 253（不算排队的时间）。scope 里的外壳在命令退出之后改成忽略 TERM、只用 bash 内建的 read / printf 写峰值文件：systemd 停 scope 时给 scope 里每个进程发 TERM，外壳那时起的 cut / cat 被杀，峰值没记下，被总上限挤掉的那条就报成撞了自己的上限（门禁 47 号里撞上过一次）；新加一例「scope 里一直收 TERM 时峰值照记」，改前那一版外壳 5 次都红。那一例的第一版在弄坏开关 nocap 下打掉了 SSH 会话，守卫与证法见第 32 行。自检 29 项全过（再加一项：slice 名不是 singlefs 开头退 2，见第 32 行），每一例都在自己的临时 slice 里（总上限 256M 或 512M），跑完不留 slice；弄坏开关（2026-09-25 UTC 12:3x，风暴守卫证过之后跑）各红 nocap 21、fallback 1、exitcode 4、noresult 3、noslice 11、slicefallback 3、nolock 1、noledger 3、nodefault 4、ignoretable 3、noslicehit 1、notimelimit 1、keepstale 2 项；第 28 行那一版包装跑新的自证红 15 项，其中「slice 设不上」「余量比整机还大」「scope 挂在 slice 底下」4 项、「两条同时判放得下」「前一条还没涨到量」2 项、「峰值表里没有这条」1 项。③ 59 号撤掉按内存收进程数（「可用内存按每个 1.5 GiB 估」「⌊(MemAvailable − 余量) ÷ 每条上限⌋」两项与余量），工作进程数回到 min(核数的一半、至多 16，表里的条数)，每条经包装排队；限时改交包装（套在包装外面的 timeout 会把排队的时间算进限时），加「被总上限挤掉」「排不上没跑」两栏。绿样本留一行撤掉的余量写法，要开 2 个、卡在「表里的条数」；经 `stage-selftest.sh` 喂，改前的 59 号在新样本上两个都判错（绿样本开 1 个，找不到 4 条 want），改后红绿都判对（22 秒；2026-09-25 UTC 12:3x 复跑 21 秒，仍判对）。门禁 47 号（UTC 12:4x，150 秒）里包装与 `mutate.sh` 的自证都过，只红 `check-segment-registry.py --selftest`（kb 段序列 `16+2+1+2` 对产物 `24+2+1+2`，是别的会话的 kb 与产物）。④ `.claude/hooks/heavy-test-guard.sh` 加一道：子 agent 跑 cargo test / t / run / r / bench（带 --no-run 的不算）、或直接执行 cargo 编出来的二进制，不经包装的拒，出路写经 `bash research/scripts/run-with-memory-cap.sh <上限> <命令>` 跑；`lib_shell_words.py` 的 `memory_capped` 顺着 bash -c 与它起的脚本往里传，`lib_heavy_tests.runs_compiled_code` 认命令。自检 562 种（该拒 98 种），`HEAVY_TEST_GUARD_IGNORE_MEMORY_CAP=1` 红 14 种；实现员跑 `cargo test -p singlefs-core --lib`，改前那一份 hook 退 0，改后退 2。54、55、57 号不改，整条经包装跑（例 `SINGLEFS_HEAVY_TESTS=commit bash research/scripts/run-with-memory-cap.sh 16G bash <根>/.claude/gate.d/54-layer0-replay.sh --full <根>`），闸放行、自检里有这几例。不改的原因：改 54 号会换它的 sha256，作废层 0 全量的全绿标记；三道要多少内存都没量过（层 0、QEMU、herd7 不能跑），写进脚本就是拍的数。定义已改、待定义三方（2026-09-26 JST）：`agent-common.md`「不做」一节加「跑编译出来的代码经内存包装」一条（`cargo test` / `run` / `bench` 与 cargo 编出来的二进制一律经 `run-with-memory-cap.sh`，写进自己脚本的整条经它；上限取派发提示给的、没给取 `replay.sh` 文件头 `REPLAY_MEMORY_CAP` 的默认值；`mutate.sh`、`replay.sh`、59 号在里面套了、外面不再包；退出码 250–254 的那一次不算结果；`cargo build` / `clippy` / `fmt` 不要求），执行员、崩溃验证员、门禁分诊员、实现员四份定义「做什么」的第 1b 步都指过去：`experiment-runner.md` 第 1b 步改成指到这一条、上限先取跑前登记给的；`crash-verifier.md` 加第 1b 步，54、55、57 号整条经包装跑、59 号直接跑；`gate-triage.md` 加第 1b 步，自己单跑的 `cargo test`、测试二进制经包装，`gate.sh` 与单跑的门禁阶段直接跑；`implementation-writer.md` 加第 1b 步，第 3 步证红与第 4 步动到的测试二进制经包装，fmt、clippy、build 不经。欠：`research/scripts/mutate.sh` 每条变异外面套 `timeout`，排队的时间会算进 120 秒，也没认 252、253、254（归正在改 mutate.sh 的那个 agent）；按名字判的门禁阶段（15、74 号这些）里起的 cargo 不在闸的射程里；slice 外面涨过余量挡不住；装新包装时还在跑的两条老包装（rbf_attack，各 20G）不在 slice 里 |
```

**出处 `records/2026-09-16-subagent拆分提案.md:944-944`（整段抄，未转述）**

```markdown
| 33 | 实验产物的判决行里有字段判 false，执行员交回时没报、主 agent 读交回时也没看见 | 2026-09-25 E142 第十六次跑第一段的产物 `research/results/e142-first-txn-dry-run-2026-09-25-r16-combined.out` 第 683 行 `E7RESULT name=verdict … control_states_ok=true control_violations_ok=false …`，第 632 行阳性对照 `violations=4092 expected_violations=4080`；执行员的交回与实验页「第十六次跑第一段」那一条都没提，主 agent 据交回判第 4、5 行够判；`layout/01` 规格起草员逐行读产物时才发现。`replay.sh` 的 exact 比对只核产物复跑得出来，不核产物自己的判决字段是不是 true | 欠：① 执行员定义写明交回之前逐行读产物的 `name=verdict` 行，任何字段是 false 都在报告里点名（定义已改、待定义三方（2026-09-26 JST）：`experiment-runner.md`「做什么」加第 4c 步、「产出」一节加判决行的点名，`false` 之外连 `not_run` 一起点名）；② 一道门禁：`replay.sh` 登记的产物里 `name=verdict` 行有字段是 false，而实验页最新那一条没有点名这一行，判红。E142 这一次的去向：问题单 `research/prompts/m2-keyspace-rerun-questions.md` 第 6 行，第十七次跑的登记写进了第 ① 条；② 已做（2026-09-25）：门禁 `.claude/gate.d/84-verdict-false-named.sh`——被扫集合是 `replay.sh` 登记表里每一行登记的入库产物，找 `E7RESULT name=verdict ` 行（现查过全仓产物，只有这一种字面形式），任何字段是 false 就要在对应实验页「## 历史版本」下最新一条或正文里、与字面 `false` 同一行点名字段名，否则判红；真仓上跑：登记 150 行、37 份文件在 research/results 下找到、113 份已归档，4 行判决、1 个 false 字段（E142 的 `control_violations_ok`）未点名，判红退出 1；fixtures 红绿样本判得对，弄坏开关 `GATE_VERDICT_FALSE_SKIP_NAMED_CHECK=1` 下红转绿，证明检查有判别力 |
```

**出处 `records/2026-09-16-subagent拆分提案.md:951-951`（整段抄，未转述）**

```markdown
| 40 | `.claude/hooks/heavy-test-guard.sh` | 按阶段文件名拒子 agent 跑 `.claude/gate.d/55-*`、`57-*`、`59-*`，不看调用走的是不是预录档、`--static-only` 这类不起虚机、不起 herd7、不跑变异的静态分支；`herd7 -version` 这种只取版本号的调用同样被拒。2026-09-26 JST 05:xx 迁 54、55、57、59 接准入的 agent 因此改 55、57 号时只能用 diff 证样本分支没动、新判法用桩核，样本的逐字 stdout 留给提交时（第 39 行那件活） | 未改：要改得先定「静态分支」怎么从命令行判（参数白名单还是阶段自报），走定义三方 |
```

**出处 `.claude/kb/checks-owed.md:394-394`（整段抄，未转述）**

```markdown
| C455 | 挂钟最贵的七道阶段没有判别力样本 | **`.claude/gate.d/` 里 7 道没有 `fixtures/<阶段>/{red,green}/`：15、21、47、52、54、70、87**（2026-09-21 起 55 号已配，从八道减到七道：它走的是 57 号那条「预录档 + 只供测试的开关」的路——`.qemu-prerecorded` 标记摆上就不起虚机、只拿同一段判定代码判某一轮真跑留下的输出，而且判全过退 3 不退 0，仓顶层误留这个标记会红、不会假绿。剩下七道里 54 与 87 同样喂得了预录输出，21、52、70 本来就不起重装置） ⇒ 层 0 崩溃点重放（46 分钟）、QEMU 真设备、replay 复跑、research 构建、脚本自证这几道最贵的，从来没人验过「被测对象真坏了它会不会红」（`show-me-test.md`「写完一个检查就问：被测对象真坏了，它会不会红？答不上来就是还没写完」）。门禁 95 号只查「头部声称了样本而目录不存在」，压根不声称也不配的只出一句 warn、阶段照记 PASS | 两步。① 95 号把「没有样本目录」从 warn 升成红，已有的 8 道逐个登记进一张豁免表、写明为什么暂时配不出（要虚机、要真设备这类）；② 逐道补样本：至少一个红样本（造一个该被它抓住的坏输入）与一个绿样本。判别力自证：给某一道的红样本改成合规输入，89 号必须由绿转红 | 无 | 2026-09-21 模拟提交流程的复核，主 agent 逐道现查坐实 |
```
