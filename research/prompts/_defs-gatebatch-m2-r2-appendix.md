**出处 `.claude/rules/implementation-workflow.md:1-5`（整段抄，未转述）**

```markdown
# 实现改动的流程：写代码 → 三方对抗 → checker，重型测试只在提交时跑

**这是 singlefs 的项目本地规则**，不在共享 SOP 里：它压在本机的三方论证（`.claude/rules/three-way-inference.md`）与
本工程接管的 herd7 / QEMU 装置上，别的项目没有这两样。共享规则在 `.claude/singlefs-ai-sop/rules/`。

```

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

**出处 `.claude/rules/implementation-workflow.md:24-29`（整段抄，未转述）**

```markdown
## 代码轮派腿之前记一份开工快照

派三方的腿之前，把这一轮被判的文件与材料点名的 kb 文件记一份 `sha256sum` 快照（放这一轮的材料目录），交核查员当输入；腿跑着的时候主 agent 不改这些文件，要改的等判决时一起改。

**别的会话改了快照里的文件**：主 agent 管不住别的会话。腿交齐之后、派核查员之前拿快照 `sha256sum -c` 一遍；对不上的，倒推出快照时的原样再交核查员——HEAD 之后没被这一轮碰过的取 HEAD，被别的会话定点替换过的把那几处替换反着做一遍，副本的 sha256 与快照相同才算数；倒推用的改动清单存进这一轮的证据，判决开头写明哪个文件、几点、被改了几处、腿引的行落没落在那几处。

```

**出处 `.claude/rules/implementation-workflow.md:30-45`（整段抄，未转述）**

```markdown
## 复用上一次全量门禁的判定：按「那一道读的输入变没变」判，不按「改了哪个目录」判

改动之后要不要重跑某一道，判据是**那一道读的全部输入自上次跑绿以来变没变**，不是「我改的是不是 `crates/`」。一道读的输入常常不止源码：herd7 读 `litmus/` 与代码里的锚点，QEMU 读装置二进制；每一道读哪些路径，以 `.claude/gate.d/stage-inputs.tsv` 里它那一行为准。

⇒ 要复用一道的判定，三样都要拿出来，缺一样就重跑：

| 要拿出什么 | 怎么算数 |
|---|---|
| 那一次跑的日志，且那一道在里面判绿 | 引它的原样判定行，不转述 |
| 那一次跑的索引与现在的索引，在**那一道读的每个路径**上逐字相同 | 登记进 `.claude/gate.d/stage-inputs.tsv` 的阶段由 `research/scripts/stage-must-run.sh` 自动比对（`refs/sop/staged-green` 那棵树与这一次的暂存树），不必再手工逐个路径现查；没登记的阶段仍要手工核，输出不截断，说不全那一道读什么就没有复用的资格 |
| 那一次之后的改动一条都碰不到那些路径 | 把改动清单与输入清单并排列出来 |

复用要在收尾报告里写明：复用了哪几道、引的是哪一次跑、比对了哪些路径。不写的按没跑算（`.claude/singlefs-ai-sop/rules/show-me-test.md`「门禁不许假装通过」）。

⚠️ **不许拿「我只改了文档」当理由。** 「我只动了一条」是需要被证明的断言，不是事实（`.claude/rules/fs-design.md`「门禁的范围必须可判定」）。

```

**出处 `.claude/rules/implementation-workflow.md:46-81`（整段抄，未转述）**

```markdown
## 重型测试只在提交时跑

**重型测试**，与 `.claude/hooks/heavy-test-guard.sh` 拒的逐类相同（判定在 `.claude/hooks/lib_heavy_tests.py` 的 `classify`）：

- 层 0 全量：`.claude/gate.d/54-layer0-replay.sh`；会跑到名字含 `layer0` 的测试二进制的 `cargo test`——`--test` 的名字含 `layer0` 或通配命中它，或者不挑目标（不带 `--test` / `--lib` / `--bin` 这类、或带 `--tests` / `--all-targets`）而包里有这种测试目标（例 `cargo test -p singlefs-harness`）；直接执行名字含 `layer0` 的测试二进制（`<target 目录>/<profile>/deps/<名字>-<16 位十六进制哈希>`）。
- QEMU：55 号、`qemu-system-*`、`research/scripts/vm-bench.sh`（`--selftest` 也算）。
- herd7：57 号、`.claude/scripts/lkmm.sh`、`herd7`；这两样带什么参数都算，只取版本号的也算。
- `crates` 变异整表：59 号、参数里有 `crates/mutations.tsv` 的 `research/scripts/mutate.sh`。
- 全量 `cargo test`（`cargo t` 同）：带 `--all` / `--workspace`；在工作区根（仓根与 `research/`）上不带 `-p` 也不带 `--test` / `--lib` / `--bin` 这类挑目标选项的；不挑目标而包的范围是工作区全部成员的（在 `research/e7-index-bench/` 里裸跑也算）；`.claude/scripts/check.sh`。
- 整轮门禁：`gate.sh`（带什么参数都算）、`research/scripts/gate-staged.sh`（`--selftest` 不算）。
- 全部实验复跑：87 号。
- E152 装置：`e152-file-system-benchmark`（直接起、或 `cargo run --bin` 它）、`research/scripts/e152-run.sh`。
- 崩溃枚举用例：`.claude/gate.d/stage-inputs.tsv` 里键是 `crash-case:` 的那几条用例的测试目标（`cargo test` 点名它、`--test` 的通配命中它、或不挑目标而包里有它；直接执行它的测试二进制），而下面任一条成立的：libtest 参数带 `--ignored` 或 `--include-ignored`（`cargo nextest run` 是 `--run-ignored only` / `all`）；`--config` 或环境变量里定了测试二进制的 runner，或子命令是 `--config` / `CARGO_ALIAS_` 定的别名；登记的用例函数没标 `#[ignore]`。另外任何命令带 `--ignored` / `--include-ignored` 又点名登记的用例函数的也算（`grep`、`git` 这类按文本处理参数的除外）。不带这两个参数、用例函数标了 `#[ignore]`、只跑那个测试目标里快用例的不算。

只认命令位置；`bash -c`、`capped.sh`、`run-with-memory-cap.sh`、`nice`、`timeout`、`env` 这类包装里面的同样算，`/usr/bin/time`、`flock`、`rustup run`、`chrt`、`prlimit`、`systemd-run`、`strace`、`perf stat|record|trace` 包在外面的剥掉照算。重型测试清单各类里的 `cargo test` 同样指 `cargo nextest run`、`cargo miri test`、`cargo llvm-cov`、`cargo hack test`、`cargo mutants` 与别名；libtest 参数带 `--list`（只列用例、一条都不跑）的哪一类都不算。命令位置上执行的脚本（`bash x.sh`、`./x.sh`、`source x`）闸读进去，逐行照同一个 `classify` 判，里面有一样就整条算（例：`.claude/scripts/fetch-deps.sh --check` 第 81 行调 `herd7 -version`，整条算 herd7）；门禁阶段与 `lib_heavy_tests.py` 的 `KNOWN_SCRIPT_LOCATIONS` 登记的仓内脚本按名字判、不读正文。`.claude/gate.d/` 下 54、55、57、59、87 之外的阶段不是重型，谁都能跑。

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

**出处 `.claude/rules/implementation-workflow.md:82-90`（整段抄，未转述）**

```markdown
## 测试与崩溃检测优先多线程

- **写法**：彼此独立的单位按区间切片，用 `std::thread::scope` 并行，不为这个加依赖。每片各自建状态（`SharedStream` 这类 `Rc` 不能跨线程）。线程数从环境变量取，没设就取 `std::thread::available_parallelism`。
- **合并要确定**：计数按片的次序相加，「第一处」取序号最小的。输出与线程数 = 1 时逐字相同，并且在同一份代码上核过一次。
- **跑的过程中报进度**：每片报区间与已跑的数，长用例的日志一直在涨。
- **不能并行的写明为什么**：共享状态、次序本身就是被测对象，写在那条用例的注释里。

**门禁管哪一半**：崩溃点重放由门禁 54 号判：整轮门禁跑快档，再逐条崩溃枚举用例（`.claude/gate.d/stage-inputs.tsv` 的 `crash-case:` 行）按它这批输入的指纹核那一格全绿标记（两条流的层 0 全量都要 `exhaustive=true`）；全量在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑 `bash <它的根>/.claude/gate.d/54-layer0-replay.sh --full <它的根>`，逐条按输入复用、只重跑输入变了的；这一趟跑了至少两片、没有显式把线程数设成 1、机器多于 1 核却只用了 1 个线程，判红（判法在 `research/scripts/admission.py` 的 `judge_worker_threads`，由它的 `--selftest` 拿合成日志核）。别的测试是不是能并行而没并行，门禁看不出，靠实现员与代码三方。

```

**出处 `.claude/agents/crash-verifier.md:1-16`（整段抄，未转述）**

```markdown
---
name: crash-verifier
description: 崩溃一致性验证员：提交代码时（或用户要求时）逐个跑层 0 崩溃点重放与登记的崩溃枚举用例（逐条按输入复用）、QEMU 真设备、herd7 内存序、crates 变异表这几道重阶段，原样交判定与计数。只在主 agent 点名派发、并给出这次改动的范围时用；不要自动派发。
tools: Read, Bash
model: opus
effort: high
omitClaudeMd: true
---

# 崩溃一致性验证员（crash-verifier）

开工先读 `.claude/agent-common.md`；这份定义开了 `omitClaudeMd`，不继承项目 CLAUDE.md 与它 `@` 的规则，要用的规则照共用约束「规则怎么读」一节读。

只跑、只报，不修。你跑的是 `.claude/rules/implementation-workflow.md`「三步，缺一步就不算做完」第 3 步与「重型测试只在提交时跑」里最重的那几道。
开工先读：`.claude/singlefs-ai-sop/skills/crash-test/SKILL.md`「判读纪律」；`.claude/singlefs-ai-sop/rules/test-discipline.md`「崩溃一致性只能靠崩溃点重放验证」「阴性结果要能和「代码没跑到」分开」；`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「文件系统特有的反推缺口」。

```

**出处 `.claude/agents/crash-verifier.md:17-24`（整段抄，未转述）**

```markdown
## 输入（主 agent 必须给）

- 这次改动的范围：`git diff --stat -- crates litmus` 原样，或提交区间。
- 这一次是提交时跑还是用户要求时跑：决定命令带 `SINGLEFS_HEAVY_TESTS=commit` 还是 `=user-request`。
- 54、55、57 号各自的内存上限（第 1b 步用），每道一个带单位的上限（例 16G）：54、57 号的写明是量过的（峰值表 `research/scripts/memory-peaks.tsv` 里那一道整条经包装跑出的峰值，且那一行「线程数」一列与这一次跑的线程数相同）还是推的；55 号的不小于同时起的虚机数乘每台的内存（`.claude/gate.d/55-qemu-first-transaction.sh` 的 `MODES` 档数 × `research/scripts/vm-bench.sh` 的 `VM_MEM`）。
- 54 号 `--full` 在哪棵 worktree 里跑：HEAD + 暂存区的 worktree 路径，或写明由你照 54 号快档出路里那三行建。只核标记、不跑全量的，写明「54 号不带 --full」；要丢掉续跑的进度文件从头跑的，写明「--start-over」。
- 报告路径与草稿目录。

```

**出处 `.claude/agents/crash-verifier.md:25-34`（整段抄，未转述）**

```markdown
## 做什么

1. 每个阶段开跑前各照共用约束「不做」一节看一次负载；另有别的 `gate.sh` 正在跑 54 号、或有 `54-layer0-replay.sh --full` 在跑时，不起 54 号，等它结束。
   1b. 54、55、57 号整条经内存包装跑（共用约束「不做」一节「跑编译出来的代码经内存包装」那一条；上限取输入给的那一道的，退出码 250–254 照那一条办）：第 2 步里起这三道的每条命令，在 `bash <阶段文件>` 前面加 `bash research/scripts/run-with-memory-cap.sh <上限>`，写成 `SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash research/scripts/run-with-memory-cap.sh <上限> bash .claude/gate.d/<文件>`；59 号在里面逐条套了，74 号在阶段里面经包装，这两道照第 2 步直接跑，外面不再包一层。
2. 只在提交时（或主 agent 转达用户要求时）跑你那几道，不跑 `gate.sh` 整轮与全量 `cargo test`（`.claude/hooks/heavy-test-guard.sh` 拒）。按共用约束 `.claude/agent-common.md`「门禁」一节从阶段归属表取登记给你的阶段，一次只跑一个；54、55、57、59 号命令带输入给的那个前缀：`SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/<文件>`，其余登记给你的轻阶段不带；54 号默认带 `--full`、在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑（`bash <worktree>/.claude/gate.d/54-layer0-replay.sh --full <worktree>`，经内存包装照第 1b 步）：它逐条跑 `.claude/gate.d/stage-inputs.tsv` 里键是 `crash-case:` 的崩溃枚举用例，这批输入那一格全绿标记在的复用、不在的才跑，不用你挑哪几条；派发提示写明「54 号不带 --full」时才只跑快档（逐条核全绿标记），写明「--start-over」时加它；自己建的 worktree 跑完用 `git worktree remove --force` 删；每个记开始与结束时刻（`date -u`）和退出码。单个阶段超过 Bash 单次上限就后台跑，退出码写进文件（`{ SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/<文件>; echo "exit=$?"; } > <草稿目录>/<阶段>.log 2>&1`；退出码只认日志里的 `exit=` 那一行，不认起后台的那条命令自己的 `$?`），结束后读输出。别的会话同时在跑 cargo 时，编译会卡在「Blocking waiting for file lock」，照实记等了多久，不算这个阶段的耗时。
3. 每个阶段原样抄它的「✓」行，或「✗」行与紧跟的「→」；退出码 77 记「本次未跑」，不记通过。
4. 计数行原样抄：崩溃状态数、恢复结果、与 E142 产物或闭式比对的那几行；54 号 `--full` 逐条用例各抄一行（「复用」「判绿」连同它记下的行、「✗」连同紧跟的「→」），末句的判绿、复用、判红各几条原样抄。输出里读不到判定行的阶段记「作废」，不记通过。
5. 缺 herd7、缺 KVM、虚机起不来这一类判「环境」，以阶段自己的报错为准，另贴 `command -v herd7`、`ls -l /dev/kvm`、`command -v qemu-system-x86_64` 的原样输出作旁证。这三条命令不单独下判断。
6. 每个阶段开跑与结束时各记一次指纹，开跑那次与起阶段写在同一条 Bash 命令里：`git rev-parse HEAD`、`git diff HEAD -- crates litmus | sha256sum`（暂存与没暂存的都算）、`git ls-files --others --exclude-standard -z -- crates litmus | xargs -0 -r sha256sum | sha256sum`（未跟踪文件按内容，不只按名单），前后或阶段之间不同，就写明「这几个绿对应的不是同一份源码」、列出哪几个阶段之间变了。

```

**出处 `.claude/agents/crash-verifier.md:35-38`（整段抄，未转述）**

```markdown
## 写范围

- 报告文件、草稿目录，与为 54 号 `--full` 建的临时 worktree（跑完删）。阶段自己用的临时目录、编译产物（59 号的 `GATE_MUTATION_TARGET_DIR` 等）与 54 号写进 git common-dir 的全绿标记、续跑的进度文件是阶段本身的行为；你不改仓里任何文件。

```

**出处 `.claude/agents/crash-verifier.md:39-42`（整段抄，未转述）**

```markdown
## 产出

- 一张表：阶段 / 退出码（0、非 0、77）/ 耗时 / 原样的 ✓，或 ✗ 与 → / 计数行；末尾「没做什么」。

```

**出处 `.claude/agents/crash-verifier.md:43-47`（整段抄，未转述）**

```markdown
## 没做什么（固定会有的）

- 没修任何一处红，也不判红是不是这一轮的改动造成的（交主 agent 或 `gate-triage`）。
- 全绿只说明这几道的证据要求满足；层 0 覆盖到哪几条流、跑了哪几条崩溃枚举用例、哪些没进来，看 54 号头部与 `.claude/gate.d/stage-inputs.tsv` 的 `crash-case:` 行，不由你外推。

```

**出处 `research/prompts/defs-gatebatch-m2-r1-main-verification.md:1-4`（整段抄，未转述）**

```markdown
# 门禁批第一轮判决（defs-gatebatch-m2-r1，2026-09-26）

<!-- doc-lint:not-numbers J1 J2 J3 J4 J5 F1 F6 F7 F8 F9 F10 F11 F12 F13 Y1 Y2 Y3 Y4 Y5 Y6 Y7 Y8 -->

```

**出处 `research/prompts/defs-gatebatch-m2-r1-main-verification.md:5-10`（整段抄，未转述）**

```markdown
## 一、这一轮

- 正文 `research/prompts/_defs-gatebatch-m2-r1-body.md`，背景材料 `_defs-gatebatch-m2-r1-background.md`，附录二 `_defs-gatebatch-m2-r1-diff.md`（门禁批 diff 的「改前」一侧是它 `cp -p` 的备份，不是某个 git 提交，射程只到这一批），开工快照 `research/prompts/defs-gatebatch-m2-r1-snapshot/sha256sums.txt`（派核查员前主 agent 核过 11 个全 OK）。
- 腿：云端正推 `defs-gatebatch-m2-r1-sonnet-output.md`（只做文字核对，探针目录空）；云端攻方 `defs-gatebatch-m2-r1-opus-output.md`（模型 `defs-gatebatch-m2-r1-opus-model/`）；本地攻方 `-local-attack-output-s1.md`、`-s2.md` 两份干净（本地攻方的定义被别处把 `model` 改成 `local-model`，换账号后起不来，主 agent 派发时给 `model: sonnet` 覆盖）。
- 核查员 `defs-gatebatch-m2-r1-verifier-output.md`：104 处 ✓101 ✗1 分不清 1。✗ 在正推：它说 `.claude/gate.d/stage-inputs.tsv` 四条崩溃枚举行的 `count-line=` / `exhaustive=` / `threads=` 都齐，实际第 34 行（会话里推抬 F 那条）三项全缺、第 35 行（c561 那条）只有 `count-line=`。分不清的那一处：正推说层 0 规模第三轮判决里没有一处写「覆盖」，核查员现跑是 1 处（第 51 行）——那一行是主 agent 在正推交回之后照它的报告补的（正推报告 13:06:45 UTC，那一行 13:07:43 UTC），正推核的时候确实没有，它判得对。攻方 38 处引用与 12 处复跑全 ✓，J1-a、J1-b、J4-b 三格复跑逐字节一致。

```

**出处 `research/prompts/defs-gatebatch-m2-r1-main-verification.md:11-27`（整段抄，未转述）**

```markdown
## 二、逐格判

| 格 | 判 | 依据 |
|---|---|---|
| J1-a 只改登记表或 54 号本身 | **打中（量过），这一批新带进来的（改 54 号本身那一形之前就有）**：范围那一问（`54-layer0-replay.sh:114`）只按 `crates/ Cargo.toml Cargo.lock` 判，只改 `stage-inputs.tsv` 或 54 号的提交快档退 77、一格标记都不核；54 号的自证全带 `SINGLEFS_GATE_FULL=1` 跑所以没抓到；先提交带 ignore 的用例、后提交登记行，新用例一次都不会被核 | 攻方 J1、核查员复跑 |
| J1-b 判法不进指纹 | **打中（量过）**：判日志的代码搬进了 `admission.py`，它不进每条用例的指纹（`admission.py:168`），修前修后指纹都是 `639097891a56ec19…`，旧判法写下的标记照样被复用 | 同上 |
| J1-c 两趟并发 | 低危、推的：后一趟判红会删掉前一趟刚写的绿标记，只会假红 | 攻方 |
| J2-a 排除法漏认 | 打中（量过），今天的仓里没有这种写法：`include!(concat!(…))` 编译期拼文件名、别的包的 `build.rs` 按目录读本包 `tests/`，被减掉的文件其实编进了用例 | 攻方 |
| J2-b 排除法减得太少 | **打中（量过，真仓拷贝）**：只改 `crates/mutations.tsv`、`tests/common_tree_split/mod.rs` 或 harness 的 `src/bin/e142_*.rs`，四条用例的指纹全变，第二条流约 2.3 天（推的）要重跑；与用户「改了只跑改了的部分」相反，而每处改法都要在 `mutations.tsv` 留一行，这种提交最常见 | 攻方 |
| J3-a 指纹漏的构建输入 | 打中（量过），今天走到的可能性低：runner、`RUSTC_WRAPPER` 指的脚本换了内容，配置里 `build.rustc` 指的编译器换了版本，指纹都不变 | 攻方 |
| J3 其余 | `CARGO_BUILD_JOBS` 不进指纹对；主工作区与 worktree 的指纹不同只会偏假红，造不成假复用 | 攻方 |
| J4-a 闸的绕法 | 打中（量过，只喂 JSON）：`cargo nextest`、`cargo mutants`、`--config` 别名与 runner、`CARGO_TARGET_*_RUNNER`、拷走测试二进制再执行、`find -exec`、`/usr/bin/time`、`flock`、`rustup run`、`chrt`、`systemd-run --scope`、`prlimit`；包装这一类以前就有，层 0 同样放行 | 攻方 |
| J4-b 用例的 ignore 没人查 | **打中（量过）**：去掉 c561 那条的 `#[ignore]` 之后，不带 `--ignored` 的命令跑 2^18 个状态，闸放行、`crash-cases` 自查退 0 | 攻方、核查员复跑 |
| J4-c 误拒 | 打中、危害小：`-- --ignored --list`、`--include-ignored --exact <一条快用例>` 被拒 | 攻方 |
| J5 文字 | 几份文字之间没有互相矛盾，都改成了逐条复用；层 0 规模第三轮判决「乙不采纳」与用户 18:5x 定案之间没写覆盖——主 agent 已在那份判决第四节之后补了覆盖记录；正推说的「四行三项都齐」不对（核查员 ✗），见 Y8 | 正推、核查员 |
| 本地 | 两份样本一致：四条用例保留、减去的文件数与四个抽样文件的去留与 `admission.py crash-case-manifest` 现跑对得上 | 本地、核查员 |

```

**出处 `research/prompts/defs-gatebatch-m2-r1-main-verification.md:28-39`（整段抄，未转述）**

```markdown
## 三、改法（被攻过零轮，第二轮攻）

- **Y1**（J1-a、J1-b，攻方 F1，副本上量过）：54 号范围那一问再加 `.claude/gate.d/stage-inputs.tsv`、54 号自己、`research/scripts/admission.py`；`admission.py` 按 `--extra-file` 进每条用例的指纹；54 号自证的 `expected_fingerprint` 跟着改，另补一格「只改登记表的提交不带 `SINGLEFS_GATE_FULL=1` 也判红」。
- **Y2**（J4-b，F6 量过、F9 推的）：`crash-cases` 自查要求登记的用例函数标了 `#[ignore]`，没标就退 2；闸在登记的测试目标被点名、却不带 `--ignored` 时读源码核那条函数标没标 ignore，没标按重型算。
- **Y3**（J4-a，F7、F8 推的）：闸把 nextest（`--run-ignored`）、mutants、miri、llvm-cov、hack 也当跑测试的子命令；`--config` 里有 `alias.` 或 `.runner`、环境里有 `CARGO_TARGET_*_RUNNER` 的当可能带 `--ignored`；前缀表补 `/usr/bin/time`、`flock`、`rustup run`、`chrt`、`prlimit`、`systemd-run`、`strace`、`perf`；任意可执行文件带 `--ignored` / `--include-ignored` 且参数里有登记的用例函数名的按崩溃枚举用例算。闸是执行前的一道，看门狗在进程那一层兜底，两道都留。
- **Y4**（J4-c，F10）：带 `--list` 的不算重型；`--include-ignored --exact <快用例>` 那一种接受误拒，写进闸的文件头。
- **Y5**（J2-a，F11）：包里出现 `include!(concat!(`、或任何一个包的 `build.rs` 读 `tests`，这个包一份都不减；另把这两种写进 `admission.py` 文件头「认不出的」那一段。
- **Y6**（J2-b，F12）：`crates/mutations.tsv` 不进崩溃枚举用例的输入；`src/bin/*.rs` 在没有任何 `.rs` 读 `CARGO_BIN_EXE_` 时不进。共用测试模块（`tests/common_*/`）要顺着模块图才减得准，这一批不做，代价写进文件头。
- **Y7**（J3-a，F13）：环境变量与 cargo 配置里 runner、wrapper 指的可执行文件按内容哈希进指纹，`build.rustc` 与 `rustc` 一类取 `-V`。
- **Y8**（核查员 ✗）：`stage-inputs.tsv` 第 34、35 行两条崩溃枚举用例补齐 `count-line=` / `exhaustive=` / `threads=`；那两条用例今天不打这几行的，写进报告交主 agent（要改 `crates/` 的测试，另派）。
- J1-c 不改，写进 54 号文件头「两趟并发只会假红」。

```

**出处 `research/prompts/defs-gatebatch-m2-r1-main-verification.md:40-43`（整段抄，未转述）**

```markdown
## 四、第二轮

改完之后开第二轮：只攻 Y1–Y8 的改后字面与代码，攻击面不重复第一轮（第一轮攻：范围那一问、判法进不进指纹、排除法漏认与减得太少、构建输入、闸的绕法与误拒、文字矛盾）。

```

**出处 `research/prompts/defs-gatebatch-m2-r1-main-verification.md:44-47`（整段抄，未转述）**

```markdown
## 五、按路径点名被判的定义与文件（门禁 72 号）

被判的：`.claude/gate.d/54-layer0-replay.sh`、`research/scripts/admission.py`、`.claude/gate.d/stage-inputs.tsv`、`.claude/gate.d/stage-owners.tsv`、`.claude/hooks/lib_heavy_tests.py`、`.claude/hooks/heavy-test-guard.sh`、`.claude/agents/crash-verifier.md`、`.claude/rules/implementation-workflow.md`、`research/scripts/check-segment-registry.py`；一并核过的 `.claude/main-agent.md`、`.claude/agents/gate-triage.md`。

```

**出处 `research/prompts/defs-gatebatch-m2-r1-main-verification.md:48-51`（整段抄，未转述）**

```markdown
## 回看决策

不涉及决策：被判的是门禁、研究脚本、hook、agent 定义与规则，不改 `.claude/kb/decisions/` 里任何一条决策。

```

**出处 `research/prompts/_defs-gatebatch-m2-r1-body.md:9-15`（整段抄，未转述）**

```markdown
| 格 | 被攻的 | 问题 |
|---|---|---|
| J1 | **54 号 `--full` 逐条跑、逐条标记、续跑**（`.claude/gate.d/54-layer0-replay.sh`） | 有没有一条路径：某条用例的输入变了而它的旧标记照样作数（假复用）；跑了一部分、被打断或判红之后留下一个作数的标记；续跑的进度目录跨输入串用；`--start-over` 没传到、或别处漏清 `SINGLEFS_LAYER0_START_OVER` |
| J2 | **按排除法算每条用例的输入**（`research/scripts/admission.py` 的崩溃枚举那一节：「别的所有测试目标」自动减掉，在别处 `.rs` 或 `Cargo.toml` 里以整词出现的不减，有 `build.rs` / `[[test]]` / autotests 的包整份不减） | 造一处改动：它改变了某条登记用例的行为，而被算成「与它无关」减掉了（宏、`include!`、`#[path]`、按目录读 `tests/`、拼接出来的名字、共用模块改名）；另看会不会减得太少，让无关改动把 2.8 天的第二条流拖去重跑 |
| J3 | **构建环境进指纹**（`build_environment_lines`；`CARGO_BUILD_JOBS` 故意不进；行名只写类别不写绝对路径） | 还有什么改了会改测试二进制的行为而不进指纹；`CARGO_BUILD_JOBS` 不进会不会让一个真有差别的构建被当成同一个；主工作区与 HEAD + 暂存区的 worktree 算出的指纹是不是一定相同 |
| J4 | **重型测试闸认崩溃枚举用例**（`.claude/hooks/lib_heavy_tests.py` 的 `classify`、`.claude/hooks/heavy-test-guard.sh`） | 子 agent 有没有一种写法绕得过去（脚本里运行时读参数、别名、`cargo nextest`、直接起测试二进制加 `--ignored` 的别的拼法）；有没有把只跑同一目标里快用例的合法命令误拒 |
| J5 | **崩溃验证员定义与相关文字**（`.claude/agents/crash-verifier.md`、`.claude/rules/implementation-workflow.md`、`stage-inputs.tsv` / `stage-owners.tsv`；门禁批没改的 `.claude/main-agent.md` 第 61 行「判绿按输入哈希写全绿标记」与 `.claude/agents/gate-triage.md` 第 29 行） | 照改后的字面，崩溃验证员在提交时做不做得对；几份文字之间有没有互相矛盾、有没有哪一处仍按「整批一格标记」写；用户原话「改了只跑改了的部分」在字面上兑现了没有 |
```

**出处 `records/2026-09-24-里程碑二收尾调度.md:190-190`（整段抄，未转述）**

```markdown
| 崩溃枚举的跑法（用户 2026-09-26 JST 18:5x） | 用户原话：「这次跑可以 下次肯定要接入提交时崩溃验证员， 并且以后跑也不能全量这么跑，改了只跑改了的部分。」起因：实五按主 agent 规格加了一条在会话推的抬 F 那一串上做小枚举的用例（`second_transaction_crash_inside_the_floor_raise_pushed_by_the_session`，20,493 个状态，debug 下单跑约 12 分钟、约 16 核），它是这条新写路径唯一的崩溃证据，层 0 两条流在 4 GiB 盘上碰不到准入被拒（推的）。**定**：① 这一次照跑；实五证红做完后给它加 `#[ignore]`，写明提交时由崩溃验证员按输入哈希跑（release）。② 门禁批（实六之后）把它接进提交时崩溃验证员那一组，在 `.claude/gate.d/stage-inputs.tsv` 登记它读的输入，输入没变复用上一次全绿判定。③ 以后崩溃枚举一律按用例 / 按流各自登记输入、各自复用，改了哪块只重跑读它的那几条——就是层 0 规模第二轮挂起的乙；第二轮打中「按流列的清单漏新加的共用文件」，门禁批照当时的修法做：清单用排除法写（整个 `crates/` 减去别的流自己的用例文件）、指纹补第三轮 U4 那几样；被攻过零轮，改的是门禁与崩溃验证员定义，照「改 agent 定义与共用约束，走同一条三步」走一轮三方 |
```
