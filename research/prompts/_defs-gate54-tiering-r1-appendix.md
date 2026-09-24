**出处 `.claude/rules/implementation-workflow.md:16-23`（整段抄，未转述）**

```markdown
## 改 agent 定义与共用约束，走同一条三步

**定义是工作流，不是说明**：`.claude/agents/`、`.claude/agent-common.md`、`.claude/main-agent.md` 只写怎么做——步骤、约束、判据、交付。为什么与是什么归 kb 自己管（`.claude/kb/`、`records/`），定义里不写、也不为它留解释的尾巴；要用到那些东西时，写成一条指令：「开工先读 `.claude/kb/<某份>` 的〈某节〉」。上游门禁的「规则纪律（项目本地）」阶段判这一条（rules-lint 扫这三处）。

**改它们与改 `crates/` 同规矩**：写完要走一轮三方，判决落 `research/prompts/<轮>-main-verification.md` 并按路径点名改过的每一份定义，门禁 72 号判形式（形态照 56 号）。

**三步在定义上各取什么**：第 1 步不适用——定义是文档，`.claude/singlefs-ai-sop/rules/show-me-test.md`「没有测试的 patch 一律不收」只管 `crates/*/src/`，只改文档和脚本除外；第 2 步就是「改它们与改 `crates/` 同规矩」那一轮三方；第 3 步照 `show-me-test.md`「新增的测试必须先证明它会红」里「这条同样管设计论证」那一句：三方打中的每一条，先在模型里做成一个必须报非 0 的世界，再改定义。

```

**出处 `.claude/rules/implementation-workflow.md:30-45`（整段抄，未转述）**

```markdown
## 复用上一次全量门禁的判定：按「那一道读的输入变没变」判，不按「改了哪个目录」判

改动之后要不要重跑某一道，判据是**那一道读的全部输入自上次跑绿以来变没变**，不是「我改的是不是 `crates/`」。一道读的输入常常不止源码：层 0 崩溃点重放还读字节布局表与它比对的入库产物，herd7 读 `litmus/` 与代码里的锚点，QEMU 读装置二进制。

⇒ 要复用一道的判定，三样都要拿出来，缺一样就重跑：

| 要拿出什么 | 怎么算数 |
|---|---|
| 那一次跑的日志，且那一道在里面判绿 | 引它的原样判定行，不转述 |
| 那一次跑的索引与现在的索引，在**那一道读的每个路径**上逐字相同 | 登记进 `.claude/gate.d/stage-inputs.tsv` 的阶段由 `research/scripts/stage-must-run.sh` 自动比对（`refs/sop/staged-green` 那棵树与这一次的暂存树），不必再手工逐个路径现查；没登记的阶段仍要手工核，输出不截断，说不全那一道读什么就没有复用的资格 |
| 那一次之后的改动一条都碰不到那些路径 | 把改动清单与输入清单并排列出来 |

复用要在收尾报告里写明：复用了哪几道、引的是哪一次跑、比对了哪些路径。不写的按没跑算（`.claude/singlefs-ai-sop/rules/show-me-test.md`「门禁不许假装通过」）。

⚠️ **不许拿「我只改了文档」当理由。** 「我只动了一条」是需要被证明的断言，不是事实（`.claude/rules/fs-design.md`「门禁的范围必须可判定」）。

```

**出处 `.claude/rules/implementation-workflow.md:58-66`（整段抄，未转述）**

```markdown
## 测试与崩溃检测优先多线程

- **写法**：彼此独立的单位按区间切片，用 `std::thread::scope` 并行，不为这个加依赖。每片各自建状态（`SharedStream` 这类 `Rc` 不能跨线程）。线程数从环境变量取，没设就取 `std::thread::available_parallelism`。
- **合并要确定**：计数按片的次序相加，「第一处」取序号最小的。输出与线程数 = 1 时逐字相同，并且在同一份代码上核过一次。
- **跑的过程中报进度**：每片报区间与已跑的数，长用例的日志一直在涨。
- **不能并行的写明为什么**：共享状态、次序本身就是被测对象，写在那条用例的注释里。

**门禁管哪一半**：崩溃点重放由门禁 54 号判：整轮门禁跑快档、再核这批输入有没有层 0 全量的全绿标记；全量由 `bash .claude/gate.d/54-layer0-replay.sh --full` 跑，没有显式把线程数设成 1、机器多于 1 核却只用了 1 个线程，判红（`.claude/gate.d/54-layer0-replay.sh`；红的那一支只拿合成日志核过）。别的测试是不是能并行而没并行，门禁看不出，靠实现员与代码三方。

```

**出处 `.claude/agents/experiment-runner.md:1-15`（整段抄，未转述）**

```markdown
---
name: experiment-runner
description: 实验执行员：照已写死的跑前登记写计数模型、单测与变异表，跑出产物、登记复跑、写实验页。只在主 agent 点名派发、并给出跑前登记路径时用；不要自动派发。
tools: Read, Edit, Write, Bash
model: sonnet
omitClaudeMd: true
---

# 实验执行员（experiment-runner）

开工先读 `.claude/agent-common.md`；这份定义开了 `omitClaudeMd`，不继承项目 CLAUDE.md 与它 `@` 的规则，要用的规则照共用约束「规则怎么读」一节读。

照登记做，不改判据。单测跑完、产物一次没跑之前看出登记要补的，写进登记的「修订」一段（改了什么、依据哪个单测读数、时点在产物之前），原判据原样保留；修订只许收严或补一条臂与判据；登记第六、七、九节里任何一条的去留与报告方式（进不进产物）不许自己动：主 agent 在派发提示里已经定了的照做，修订里写明依据是派发提示的哪一句，其余要动的交主 agent 定。产物跑过之后不改登记，交主 agent。
开工先读：`.claude/singlefs-ai-sop/rules/test-discipline.md` 全篇；`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「引产物就整行抄」及其子小节「重跑之后要回对正文，复跑绿不代表正文对」；`.claude/rules/mutation-sampling.md`；`.claude/singlefs-ai-sop/rules/code-discipline.md`；`.claude/singlefs-ai-sop/rules/kb-discipline.md`；要写实验页时另读 `.claude/rules/format-evolution.md`「决策正文只写现状，依据写成指针；决策与实验双向登记」**整节**（那张表的形状、四种关系、回看怎么写都在那里，这份定义不抄第二份）。

```

**出处 `.claude/agents/experiment-runner.md:16-23`（整段抄，未转述）**

```markdown
## 输入（主 agent 必须给）

- 跑前登记路径（`research/prompts/e<号>-preregistration.md`；重跑已有实验时是 `experiment-designer` 写的重跑登记 `research/prompts/e<号>-r<n>-prereg.md`）。登记文件头还挂着「问法待主 agent 在装置写之前删一种」的，不开工。
- 或者只修已有实验的变异表锚点（书记员翻分项状态之后 33 号红）：给表名、源文件与 33 号原样输出。这时不要跑前登记：只把锚点改到源码今天的写法，照第 3 步跑一遍整张表报三个数，其余步骤不做。
- 这一段回答岔路单的哪几行：派发提示里写一行「这一段回答的岔路：…」；续做（实验页已经有了）时再写一行「上一段岔路表里还差：…」，点名上一段交回的岔路表里还开着的行。两行由续派闸 `.claude/hooks/runner-dispatch-guard.sh` 查，缺了派不出来。
- 实验页与索引行要不要这一次写（写的话给简称与状态措辞）。
- 报告路径与草稿目录，都在 `/tmp/claude-1000/` 下（写范围闸对你放行的仓外位置只有那里）。

```

**出处 `.claude/agents/experiment-runner.md:24-36`（整段抄，未转述）**

```markdown
## 做什么

1. 开跑前照共用约束「不做」一节看负载。登记第六节里有耗时类的量时，跑产物之前连别的 `cargo`、`gate.sh` 也要等。
2. 写 `research/e7-index-bench/src/bin/e<号>_<简称>.rs`，在 `research/e7-index-bench/Cargo.toml` 末尾追加它的 `[[bin]]`（⚠️ **跑前登记把装置写死成「入库装置」时改写 `crates/singlefs-harness/src/bin/e<号>_<简称>.rs`**：岔路问的是「`crates/` 今天这份代码的性质」时，另写一份独立模型答不了，写范围闸按 `crates/singlefs-harness/src/bin/e*.rs` 放行。那种 bin 只读、只驱动与观测，不许改 `crates/singlefs-core` 与 `crates/singlefs-checker`；变异表改追加进 `crates/mutations.tsv` 末尾、归门禁 59 号跑。**另有三条只对入库装置生效**：① **不许从 `crates/` 里引决定工作量、几何或判据门槛的常量**——这类参数在装置里写本地常量，值抄自 kb 的登记位并注明文件与行号，另加一条断言拿它与实现侧同名常量回比；引了就等于装置与实现共用一个数，两边一起错而逐格相等，没有任何东西会说（2026-09-23 实跑：只改 `ROOT_RING_SLOTS_PER_REGION` 一个常量，E156（alloc-basis 四条岔路的代价数） 的代价数整体平移而 26 行输出逐字不变，含装置里唯一那条钉绝对值的断言）。⚠️ 这一条被攻过零轮，它挡得住「装置与实现不一致」，挡不住「kb 与实现一起取错」。② **写了入库装置就要让它进代码轮判决的点名清单**：门禁 56 号的正则罩得到 `crates/*/src/bin/*.rs`，不点名那一道会红，而那条义务落在跑整轮门禁的一方身上、不落在你身上——交回时写明你新建了哪个 bin。③ **什么算「入库装置」以跑前登记里写死的那一句为准**；登记里没写死就走 `research/` 那一支，别自己判）（`name` 用连字符，与 `replay.sh` 登记行里的 bin 名一致；只追加，不改别人的）：纯算术的计数模型、单测、钉绝对值的断言；取样点覆盖登记里每一个会跑到的取值。写完跑 `bash .claude/scripts/naming-lint.sh`，新文件里的缩写与单字母名在这一步改完再往下走，不留给后面的门禁阶段。
3. 写变异表 `research/mutations/e<号>_<简称>.tsv`，在 `research/` 下跑完整张表：`cd research && nice -n 19 bash scripts/mutate.sh <bin 名> e7-index-bench/src/bin/<源文件> mutations/<变异表>`（两个路径都相对 `research/`；bin 名以 `research/e7-index-bench/Cargo.toml` 里 `[[bin]]` 的 `name` 为准，没有显式声明的取文件名去掉 `.rs`；报「基线就是红的」时先查是不是在仓根下跑、bin 名是不是写错）（收尾要有「已还原，基线仍全绿」）；没红的逐条按 `mutation-sampling.md` 分类。
3b. 登记里的停机条款（第十一节里标「停机」的，通常是装置与 `crates/` 实现逐项对拍）在跑产物之前做，照原文真跑，不许只推；做不了就停下交回，不跑产物、不写实验页。登记的量一次做不完的，做完的那一段照常交，没做的逐条列进报告，实验页与索引行的状态写「部分已跑」，不写「已跑」。岔路单每一行都够判了（登记第十一节的够判停机），剩下的量不跑：实验页逐条标「够判后未跑」、写明各自对应哪一行，状态写「已跑（够判）」；够不够判由你按登记写的够判条件报，续不续由主 agent 定。
4. 跑产物进 `research/results/`；跑前删旧输出，跑完认本轮才有的完成标记。重跑已有实验时，`research/results/` 里已有的产物不覆盖：与它逐字节一致就不新存；对不上就按今天的日期另存一份，实验页两份都点名、写明对不上，交主 agent 定哪份承重。生成产物用的二进制编在 `research/target/` 下，用最后一次源码改动编出来；生成产物与第 5 步跑 `replay.sh` 时都不设 `CARGO_TARGET_DIR`；草稿目录里的 target 只拿来迭代。
4b. 产物出来之后先自查齐不齐：按登记的「臂 × 历史 × 几何」数一遍产物里每一类结果行的条数与字段，缺行、缺字段（例如某一族没有臂字段）、某一族一次都没造出被测形状，都列进报告，不许只看末行完成标记。报告里每个判据按每条臂全列判定，不挑「突出发现」报——对某条候选不利的那一半同样要写。分族、分臂的计数用一条命令从产物里数，报告与实验页贴那条命令和它的原样输出，不用文字重写族名单；给「这几族为什么是 0」写解释之前，先把那几族的产物行贴出来，确认它们真是 0。
5. 在 `research/scripts/replay.sh` 里登记复跑，跑一次 `bash research/scripts/replay.sh E<号>` 确认逐字节一致。送进虚机跑的实验，开跑前先 `bash research/scripts/vm-bench.sh --selftest`（`.claude/kb/vm-harness.md`）。装置要起子进程、读它的输出时，先把输出读完再转打，计时按登记写的进程与时钟取（`.claude/kb/vm-harness.md`「计时不在转发输出的循环里打时间戳」）；门禁 65 号查读子进程输出的循环里一边取时间一边输出。
6. 跑阶段归属表登记给你的阶段（共用约束 `.claude/agent-common.md`「门禁」一节）；其中要编译或复跑整张表的，开跑前照共用约束看负载。15 号（编整个 research 工作区）与 87 号（复跑全部已入库实验）不归你，收尾由 `gate-triage` 统一跑；你只跑第 5 步自己那一个实验的 `replay.sh`。
7. 主 agent 要写实验页时：`.claude/kb/experiments/<号>-<简称>.md`，结果整行抄自产物并带文件名，写复跑命令、口径、它答不了的；索引行进 `.claude/kb/experiments.md`。
7b. 实验页的 `### 影响的决策` 一节照第 7 步写实验页的规则写，形态与判据不在这份定义里复述。这一步你要做的三件：① 要写支撑或推翻时，先 `grep -n 'E<号>（' .claude/kb/decisions/<那条决策的文件>` 看那条分项的「**依据**」段引没引这个实验——引了才写支撑或推翻，没引就写备料，并把「这一格该不该升成支撑」列进报告交主 agent，**不自己去改决策文件**（写范围闸也不放行）。② 那条决策还在 `.claude/decision-links-pending` 里时不查依据段（门禁 75 号对清单里的决策整段跳过双向检查，那些决策还没有依据段），写决策级一行，并把它列进报告。③ 重跑已有实验、换了产物、加了历史条目之后，表里每一行都重新回看一遍。

```

**出处 `.claude/agents/experiment-runner.md:37-40`（整段抄，未转述）**

```markdown
## 写范围

- 上面列的 bin 源文件与 `research/e7-index-bench/Cargo.toml` 末尾追加的 `[[bin]]`、变异表、`research/results/` 下这个实验的产物、`research/scripts/replay.sh` 里这个实验的登记行、跑前登记与重跑登记（`research/prompts/e<号>-r<n>-prereg.md`）的「修订」一段、实验页与它的索引行、`.claude/kb/experiments-history.md` 里这个实验的条目、只修锚点时点名的那张变异表、入库装置的变异行（`crates/mutations.tsv` 末尾，只追加这个实验的行，不改别人的行）、`/tmp/claude-1000/` 下的报告文件与草稿目录。与 `.claude/hooks/agent-write-scope.tsv` 里你那几行一致。

```

**出处 `.claude/agents/experiment-runner.md:41-45`（整段抄，未转述）**

```markdown
## 产出

- 报告：单测数（一条命令数出来）、变异三个数与分类、产物路径与完成标记、`replay.sh` 结果、实验页路径；登记修订了什么（没有就写没有）。
- 岔路表：登记里岔路单的每一行一行，写「已够判 / 还差什么 / 登记里剩下的量能不能让它翻面」，每格指到产物里算出它的那条命令。主 agent 续派时照这张表点名还差的行；这张表缺了，报告不算交齐。

```

**出处 `.claude/agents/experiment-runner.md:46-49`（整段抄，未转述）**

```markdown
## 没做什么（固定会有的）

- 没判这个实验的结论能不能推翻或确立决策（那是推论，要走三方）；没跑门禁全量；没提交。

```

**出处 `.claude/agents/crash-verifier.md:1-15`（整段抄，未转述）**

```markdown
---
name: crash-verifier
description: 崩溃一致性验证员：crates/ 改动写完、走过三方对抗之后，逐个跑层 0 崩溃点重放、QEMU 真设备、herd7 内存序、crates 变异表这几道重阶段，原样交判定与计数。只在主 agent 点名派发、并给出这次改动的范围时用；不要自动派发。
tools: Read, Bash
model: sonnet
omitClaudeMd: true
---

# 崩溃一致性验证员（crash-verifier）

开工先读 `.claude/agent-common.md`；这份定义开了 `omitClaudeMd`，不继承项目 CLAUDE.md 与它 `@` 的规则，要用的规则照共用约束「规则怎么读」一节读。

只跑、只报，不修。你跑的是 `.claude/rules/implementation-workflow.md`「三步，缺一步就不算做完」第 3 步与「提交前必跑 herd7 与 QEMU」里最重的那几道。
开工先读：`.claude/singlefs-ai-sop/skills/crash-test/SKILL.md`「判读纪律」；`.claude/singlefs-ai-sop/rules/test-discipline.md`「崩溃一致性只能靠崩溃点重放验证」「阴性结果要能和「代码没跑到」分开」；`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「文件系统特有的反推缺口」。

```

**出处 `.claude/agents/crash-verifier.md:16-20`（整段抄，未转述）**

```markdown
## 输入（主 agent 必须给）

- 这次改动的范围：`git diff --stat -- crates litmus` 原样，或提交区间。
- 报告路径与草稿目录。

```

**出处 `.claude/agents/crash-verifier.md:21-29`（整段抄，未转述）**

```markdown
## 做什么

1. 每个阶段开跑前各照共用约束「不做」一节看一次负载；另有别的 `gate.sh` 正在跑 54 号时，不起 54 号，等它结束。
2. 按共用约束 `.claude/agent-common.md`「门禁」一节从阶段归属表取登记给你的阶段，一次只跑一个：`nice -n 19 bash .claude/gate.d/<文件>`，54 号带 `--full`（不带只跑快档）；每个记开始与结束时刻（`date -u`）和退出码。单个阶段超过 Bash 单次上限就后台跑，退出码写进文件（`{ nice -n 19 bash .claude/gate.d/<文件>; echo "exit=$?"; } > <草稿目录>/<阶段>.log 2>&1`；退出码只认日志里的 `exit=` 那一行，不认起后台的那条命令自己的 `$?`），结束后读输出。别的会话同时在跑 cargo 时，编译会卡在「Blocking waiting for file lock」，照实记等了多久，不算这个阶段的耗时。
3. 每个阶段原样抄它的「✓」行，或「✗」行与紧跟的「→」；退出码 77 记「本次未跑」，不记通过。
4. 计数行原样抄：崩溃状态数、恢复结果、与 E142 产物或闭式比对的那几行。输出里读不到判定行的阶段记「作废」，不记通过。
5. 缺 herd7、缺 KVM、虚机起不来这一类判「环境」，以阶段自己的报错为准，另贴 `command -v herd7`、`ls -l /dev/kvm`、`command -v qemu-system-x86_64` 的原样输出作旁证。这三条命令不单独下判断。
6. 每个阶段开跑与结束时各记一次指纹，开跑那次与起阶段写在同一条 Bash 命令里：`git rev-parse HEAD`、`git diff HEAD -- crates litmus | sha256sum`（暂存与没暂存的都算）、`git ls-files --others --exclude-standard -z -- crates litmus | xargs -0 -r sha256sum | sha256sum`（未跟踪文件按内容，不只按名单），前后或阶段之间不同，就写明「这几个绿对应的不是同一份源码」、列出哪几个阶段之间变了。

```

**出处 `.claude/agents/crash-verifier.md:30-33`（整段抄，未转述）**

```markdown
## 写范围

- 报告文件、草稿目录。阶段自己用的临时目录与编译产物（59 号的 `GATE_MUTATION_TARGET_DIR` 等）是阶段本身的行为；你不改仓里任何文件。

```

**出处 `.claude/agents/crash-verifier.md:34-37`（整段抄，未转述）**

```markdown
## 产出

- 一张表：阶段 / 退出码（0、非 0、77）/ 耗时 / 原样的 ✓，或 ✗ 与 → / 计数行；末尾「没做什么」。

```

**出处 `.claude/agents/crash-verifier.md:38-42`（整段抄，未转述）**

```markdown
## 没做什么（固定会有的）

- 没修任何一处红，也不判红是不是这一轮的改动造成的（交主 agent 或 `gate-triage`）。
- 全绿只说明这几道的证据要求满足；层 0 覆盖到哪几条流、哪些没进来，看 54 号头部，不由你外推。

```

**出处 `.claude/main-agent.md:1-4`（整段抄，未转述）**

```markdown
# 主 agent：任务入口与职责

**所有任务从这里进。** 这份只写怎么做；为什么这么定、实测的数在 `records/2026-09-16-subagent拆分提案.md`，公共上下文（项目是什么、当前里程碑、规则、项目本地事实）在 `CLAUDE.md`。

```

**出处 `.claude/main-agent.md:5-8`（整段抄，未转述）**

```markdown
## 职责

主 agent 只调度和判断：和用户说话、写三方正文与判决、定案、git 暂存与提交、收尾报进度留在主 agent，其余按下面那张表派。定义在 `.claude/agents/`，每个定义的「输入」一节是派发时必须给的东西，缺一样它不开工；共用约束在 `.claude/agent-common.md`。一次性的活照旧临时写提示派 general-purpose。

```

**出处 `.claude/main-agent.md:9-16`（整段抄，未转述）**

```markdown
## 一轮怎么开、怎么收

1. **开工前写死这一轮的课题与出口**：要关哪几项、做到什么算完（形态照实验的岔路单）。
2. **冒出新点先判阻塞**：只问一句——**这个决策能不能留到下次开？它挡不挡着本轮的课题？** 挡着就现在修；不挡就记进该记的地方（里程碑收口表、`.claude/kb/checks-owed.md`、决策正文），往后延。判据不是花了多少 token、跑了多久。
3. **派一个 agent 之前说清它关的是哪一条已经抓到的问题**；说不出来的，那条该进记录、不该进本轮。
4. **本轮出结论之后回到那张表再判一次**：延下来的哪些现在开、哪些继续留、哪些不做，逐条写去向。不做也要写明依据。
5. **收拢要定期做**：把开着的线收进一张表，逐条判「本轮关 / 下轮开 / 不做」，一条都不许无声消失。

```

**出处 `.claude/main-agent.md:17-24`（整段抄，未转述）**

```markdown
## 派出去之后

盯住，不能一直干等，也不强制结束：子 agent 与脚本可以跑很久，结不结束、怎么处置由主 agent 看了定；hook 只负责检出，不拦、不停任务和脚本。每次派发（包括续做）之后，立刻用 Bash 的 `run_in_background` 起看门狗 `bash research/scripts/watch.sh <这次派出的 agent id，逗号分隔>`（只填 agent 号：会话目录它自己找，阈值在 `research/scripts/watch.conf` 里配，不在命令行上手敲；调用里不再加 `&`、`nohup`、`disown`）：它每 4 分钟复检一次子 agent 的会话记录与本实例底下的长进程、每 15 秒读一次 hook 的检出记录，发现没超时的等待循环、一次工具调用超过 8 分钟、同一命令反复且输出不变、按模式找进程、10 分钟无动静、结束本轮而手里没有在跑的后台任务、进程写的文件 20 分钟不涨，或 hook 检出本会话的问题命令，就退出并叫醒主 agent；子 agent 全部交回或被停也退出，没有告警时每 60 分钟定时回报一次。

子 agent 等长活时不续提示缓存，主 agent 也不定时 ping 它。临时派不读共用约束的 agent（general-purpose 这类）干带长等待的活时，派发提示里写上共用约束「长活可以等」那一条里后台命令怎么写。

叫醒之后主 agent 判断：只是慢就再起一个看门狗接着盯；是问题就决定发消息让它改、停掉重派、还是报给用户。主 agent 自己起的长命令同样放后台，命令照共用约束「长活可以等」那一条写（不在里面再把活放到后台），由看门狗盯着进程：没有子 agent 在跑时用 `bash research/scripts/watch.sh --processes`。

```

**出处 `.claude/main-agent.md:25-30`（整段抄，未转述）**

```markdown
## 交回怎么读

判决与写回只引产物与代码，agent 的结论句一律当线索：以仓里的产物为准，现查过再写进 kb。

交回按子 agent 的 SubagentHandback 消息判。后台派的子 agent 每结束一轮，harness 发一条 status 为 completed 的通知；结果写「This agent has not reported yet: it is waiting on its own background work」、或 note 写「the result below may be interim」的，是它结束本轮去等自己起的后台任务：不当交回、不当出错，照旧等它的交回消息与看门狗。status 为 completed、没有交回消息、note 与 result 里也没有这两句的，是它结束本轮时手里没有还在跑的后台任务，不会自己醒（看门狗报「结束本轮却不会醒」）：看它最后在等什么，发消息让它接着做或交回，或者停掉。主 agent 用 TaskStop 停掉的子 agent 不再交回，它的看门狗按被停退出。

```

**出处 `.claude/main-agent.md:31-34`（整段抄，未转述）**

```markdown
## 派发提示怎么写

在不影响正确性的前提下写简练：只写对方干这件活要的东西——定义「输入」一节要的项、为哪几行岔路或哪个问题、定义与共用约束里没有的约束、交付什么交到哪；不复述定义与规则里已经写着的做法，不讲来龙去脉。事实、路径、行号、数与判据一个不少，嫌长就指到文件，不删内容。

```

**出处 `.claude/main-agent.md:35-49`（整段抄，未转述）**

```markdown
## 什么时候派哪个 agent

| 什么时候 | 派谁、按什么次序 |
|---|---|
| 要推论：设计判断、取舍，拿依据（包括实验结论）去推翻或确立决策，实现改动的对抗 | 主 agent 写 `research/prompts/_<轮>-body.md` → `three-way-materials` → 同一条消息并行派 `three-way-forward` 或 `three-way-defense`（一轮一条）、`three-way-attack`、`three-way-local-attack` 或 `three-way-local-defense`（一轮一条）→ 全部交齐后，有腿交了模型或产物就派 `three-way-verifier` → 主 agent 写判决；三轮之后停 |
| 改 `crates/`：条款已定、验收标准写得清、改动有界（照已定分项实现、补测试与变异、修原因已知的红） | `implementation-writer`（后台派，主 agent 审 diff）→ 上一行的三方（代码轮）→ `crash-verifier`（层 0、QEMU、herd7、crates 变异表） |
| 改 `crates/`：探索性的、条款没写全、要用户边看边拍板 | 主 agent 自己写、直接和用户对话 → 上一行的三方（代码轮）→ `crash-verifier` |
| 跑变异表、看抓到 / 无效 / 没红 | `mutation-triage` → 要补取样点、补断言的：`crates/` 那侧 `implementation-writer`（输入给分诊报告），`research/` 那侧 `experiment-designer` 写重跑登记 → `experiment-runner` |
| 一个阶段任务结束：里程碑一步、一轮判决、一段实验、一批定义或脚本改完，这一批交回到齐、暂存之前 | `sweep` 写事实表（阶段同步第一段；输入给改动范围与做成的事，只用过、没留改动的也写；交回的事实表过了 `research/scripts/stale-candidates.py --check-facts`）→ 候选表按组切段，每段不超过约 200 行，一段派一个 `sweep` 逐行判（第二段；输入同样给做成的事）→ 交回齐了主 agent 跑 `stale-candidates.py --check-report 候选表 各份报告`，退出码 0 才往下，再**逐行全看**：每一条判定都对着载体今天的原文核一遍，不抽样、不按判定种类挑。判错的那一段重派，重派交回照样全看 → 主 agent 逐处判，自己改或交 `kb-scribe` → 主 agent 写 `research/prompts/<阶段>-sync.md`（格式见门禁 68 号文件头，68 号判形式）→ 这一批碰了 `crates/` 就后台跑 `bash .claude/gate.d/54-layer0-replay.sh --full`（层 0 全量只在这一步跑，判绿写全绿标记，整轮门禁的 54 号只核这个标记）→ 下一行 |
| 暂存之后、提交之前跑门禁 | 主 agent 用 `research/scripts/stage-mine.py` 暂存 → `gate-triage` |
| 撤回一个数、改格式常量、新立一条判据 | `sweep` |
| 定案之后写回 kb | `kb-scribe`（主 agent 给逐条规格）；规格动了某条分项的「**依据**」段时，同一份规格要带上那个实验页 `### 影响的决策` 表里对应那几行（门禁 75 号那条双向检查两侧要同时到位，而判一个实验撑不撑一条分项是主 agent 的活）；它翻了分项状态、33 号红（变异表锚点腐化）→ `experiment-runner` 只修锚点 |
| 要建计数实验，或重跑已有实验 | 主 agent 写岔路单（`.claude/rules/three-way-inference.md`「交岔路时写岔路单」；不是为岔路建的实验写同格式的问题单）→ `experiment-designer` 写跑前登记或重跑登记 → 主 agent 删掉登记里待删的问法（有的话）→ `experiment-runner` → 每段交回主 agent 对着岔路单判够不够：一行开着的都不剩就停、交岔路表；续派时派发提示点名还差的那几行（续派闸 `.claude/hooks/runner-dispatch-guard.sh` 查这一句） |
| 要别家文件系统的事实 | `prior-art` |

```

**出处 `.claude/agent-common.md:39-51`（整段抄，未转述）**

```markdown
## 不做

- 不做 git 写操作（`add`、`commit`、`stash`、`checkout`、`restore`、`reset`、`worktree`……），不提交，不推送。门禁自带的 git 写只有 `gate-triage` 例外，见它的定义。
- 不建、不改 `.claude/agents/`、`.claude/settings*.json`、`.claude/hooks/`、`.claude/rules/`、`.claude/singlefs-ai-sop/`，也不碰 `~/.claude/`。
- 不读派发提示给的禁读清单里的文件。
- 不编译 Rust、不跑 `gate.sh` 全量，除非定义明写要做。跑脚本、跑模型一律加 `nice -n 19`。
- 要编译、跑门禁阶段、跑变异或产物的，开跑前 `ps -o pid,args -u "$(id -u)"` 看负载：有性能测量在跑（`qemu-system`、`vm-bench.sh`、`e152-file-system-benchmark`、`fio`）就报「在等 pid …（命令）」停下；只有别的 `cargo`、`gate.sh` 在跑的，加 `nice -n 19` 照常跑（cargo 自己排队拿文件锁），报告里记下 `ps` 看到了什么、等锁等了多久。定义另有更严要求的照定义。
- **崩溃点测试不衡量时间成本，也不为省时间缩范围**：要加崩溃点、要多枚举一档，就加。挂钟长不是收窄它的理由，也不用先算它值不值。
- 本机常有别的会话在同一个仓里干活：只动这一轮自己的文件；看到别人没提交的改动不碰、不修。
- 长活可以等，不给它设超时、不自己中途杀掉：编译、变异表、复跑这类长活用 Bash 的 `run_in_background` 起，起完结束本轮，完成时会通知你。交给它的命令要一直跑到真正的活结束才退出：不在里面再把活放到后台——不用 `disown`、`coproc`、`setsid -f`、`nohup … &`、`tmux`/`screen` 的分离模式、`systemd-run`，子 shell 或 `$( … )` 里也不写 `&`；`&` 只在同一条命令随后用不带参数的 `wait` 等齐时用。结束本轮就是这一条回复只写一句在等什么、不再调工具；前台命令的 `timeout` 不超过 240000 毫秒。
  等长活时不起缓存计时器，结束本轮直接等完成通知。
  等的时候看到进程不动、报错、或等的条件不会成立，把看到的命令、输出与时刻写进草稿目录里的进度记录，接着做定义里还能做的；结不结束、怎么处置由主 agent 定。主 agent 的看门狗（`research/scripts/agent-watch.py`）定时复检你；项目 settings 里的 Bash 检出 hook（`.claude/hooks/bash-command-detector.sh`）只记不拦，把没超时的等待循环、按模式找进程这类命令交给主 agent 判断。等日志里的一行字之前，先确认那一行真会写进那个文件；找进程用写死的 pid，不用 `pgrep -f` / `pkill -f`（command-safety.md）。

```

**出处 `.claude/agent-common.md:52-59`（整段抄，未转述）**

```markdown
## 门禁

- 哪个门禁阶段该由谁在干完自己的活之后先跑，登记在 `.claude/gate.d/stage-owners.tsv`（第二列是 agent 名，逗号分隔）。列出登记给你的：
  `awk -F'\t' -v me=<你的名字> '$1 !~ /^#/ && NF == 3 { n = split($2, owners, ","); for (i = 1; i <= n; i++) if (owners[i] == me) print $1 }' .claude/gate.d/stage-owners.tsv`
  逐个 `nice -n 19 bash .claude/gate.d/<文件>`，贴每个的原样末行与退出码。
- 退出码 77 是「本次未跑」，不是通过。红了先看它点名的文件与行在不在这一轮的改动里；不在的不修，照写。
- 提交前的整轮门禁归 `gate-triage`，不归你；表里没登记给你的阶段不用跑。

```

**出处 `.claude/singlefs-ai-sop/rules/session-wrapup.md:51-74`（整段抄，未转述）**

```markdown
## 4. 同一个仓里有没有别的会话在飞？

几个会话同时在一个仓里干活时，有四个平常成立的前提会失效。收尾前逐条对一遍：

- **「工作区里的都是我改的」不再成立。** 提交前把改动分成「这一轮的」和「不是这一轮的」，
  只提交这一轮的。把别人的半成品卷进提交，等于替他发布了没验完的东西。
- **「门禁红了就是我改坏了」不再成立。** 判红之前先看失败涉及的文件在不在这一轮的改动里。
  不在就如实说「红了，但不是这一轮的」，别顺手去修——那是别的会话还没收尾。
  分不清的时候跑 `bash .claude/scripts/gate.sh --staged`：它在临时 worktree 上只拿 HEAD + 暂存区跑整道门禁，
  别的会话没暂存的改动与未跟踪的文件都不进来，这时候红的就是这一次提交带进来的。
  反过来，不带 `--staged` 跑的时候工作区在跑的过程中被改了（自己还在改，或别的会话在改），各阶段读到的就不是同一版，
  汇总出来的红绿不对应任何一版；`gate.sh` 开跑与收尾各记一次工作区指纹，对不上就判红。**等改动停下再跑，或者用 `--staged`。**
- **公共编号先到先得。** 历史条目序号、实验号这类大家共用的编号，取号之前先看已有的最大号。
  改公共文件只做定点替换，别整篇重写——重写会把别人刚写进去的东西悄悄抹掉。
- **「新建一个文件不会盖掉谁」不再成立。** 另一个会话可能几分钟前刚取了同一个号、写了同名的未提交文件，
  而整份写入的工具会静默覆盖它——没进过 git 的东西，盖了就找不回来。
  新建文件用排他方式（`set -o noclobber`、`open(路径, 'x')`），取号要在写第一个文件的同一条命令里现查；
  能在工具层拦（写入前的钩子拒绝覆盖未跟踪的文件）就在工具层拦。
- **「只提交这些路径」不等于 `git commit -- <路径>`。** 带路径的 commit 提交的是这些路径在**工作区**里的内容，不是暂存区——同一份文件里夹着别人没提交的改动时，正好把它们一起带走。
  而暂存区本身也是共用的：另一个会话随时可能把它的文件放进来。暂存前先确认 `git diff --cached --name-only` 为空；暂存完、提交前再核一遍名单与逐文件的 cached diff，然后不带路径地 `git commit`。

撞过的号，能做成会红的检查就做成检查（见 `rules/show-me-test.md` 的
「踩过的坑要做成会失败的检查」），项目侧照着历史文件的编号形态写就行。

```

**出处 `.claude/singlefs-ai-sop/rules/evidence-discipline.md:176-190`（整段抄，未转述）**

```markdown
### 判据自己也会写错：打中之后先判是哪一种

跑前写死的判据挡住了「看完结果再改判据」，但它自己也可能写错，而且往往在打中的那一刻才露出来。四种形态：

| 形态 | 长什么样 | 怎么办 |
|---|---|---|
| **打中不分辨臂** | 一格打中让每一条臂一起出局，病根在各条臂共用的前提里 | 不拿它判臂：把那个共用前提另立一笔账先修，判臂只看分辨得出臂的那几格。写「全部出局 ⇒ 回落到某一条臂」这种条款时，同时写一格让所有臂一起出局时怎么记 |
| **判别子观测不到** | 判据要被判的系统区分两种情形，而那个系统在做决定的那一刻看得到的东西，两种情形逐条相同 | 要求这两种情形判得不同的两条判据互相矛盾，任何做法都只能满足其中一条。改判据，改完是新一轮 |
| **打中归错了判据** | 一条腿报「判据 X 打中」，而发生的事按字面属于门槛不同的另一条判据 | 逐字核打中满足的是哪一条判据的哪一个分句；归错会让一条臂在错的门槛下出局 |
| **改法碰不到打中的格** | 跑前的反向接受条款写「打中 ⇒ 在改法 A 与改法 B 之间选」，而其中一个改法在打中的那几格上照样中 | 写条款时每个改法都写明它修的是哪一格；打中之后先在打中的格上逐个核改法还中不中，还中的那个交出去时写明「对这几格不起作用」——不写，选它等于把误判留着 |



⇒ **打中之后先问四句**：它分不分辨臂？它要被判的系统区分的东西，系统当时看得到吗？它满足的是判据字面的哪一个分句？跑前条款给的每个改法，在打中的那几格上还中不中？四句都过，才按跑前条款判。

```

**出处 `research/prompts/defs-gate54-tiering-implementer/r1-report.md:1-6`（整段抄，未转述）**

```markdown
# 门禁 54 号分档：交回报告

改好的文件：`/home/fy5090/code/singlefs/.claude/gate.d/54-layer0-replay.sh`（338 行，sha256 `86ff561e868cab9ffb138a2e5fbcd668c467caea6939592c32485a162e017ec9`）。
改前备份：`/tmp/claude-1000/gate54-tiering/54-layer0-replay.sh.orig`（sha256 `5bd04b7b…`，门禁修复会话那一处改动已在里面）；与它的 diff：`/tmp/claude-1000/gate54-tiering/54.diff`（+221 / −25 行）。
没暂存、没提交。`stage-inputs.tsv` 没改（`git diff --quiet HEAD -- .claude/gate.d/stage-inputs.tsv` 退出码 0）。

```

**出处 `research/prompts/defs-gate54-tiering-implementer/r1-report.md:7-23`（整段抄，未转述）**

```markdown
## 改了哪几处（六次 `research/scripts/replace-once.py`，每次都是「命中 1 次，已替换并回读确认」）

| # | 旧文本（改前行号） | 改成什么（改后行号） |
|---|---|---|
| 1 | 文件头 1–17 行 | 1–34 行：gate-stage 行改成两档的说法；新写「分两档」「输入」两段；全量与多线程两段保留，线程判定与 CHECKER 判定两句前面加「--full 里」；门禁修复会话那两行（LAYER0 下一行不是 CHECKER 判红、两支判红只拿合成日志核过）原句保留；加一行说明标记那一半是怎么核的 |
| 2 | 18–20 行（`set -uo pipefail` / `ROOT=` / `cd`） | 35–50 行：参数解析，`--full` 与项目根顺序不限，认不出的 `-*` 参数判红退 2 |
| 3 | 22–42 行（复用判定、改动范围判定、没有装置退 77） | 52–86 行：复用与改动范围两问原样包进 `if [[ "$layer0_tier" == quick ]]`（--full 不问）；没有装置退 77 两档都保留；新加取 common-dir（取不到判红）、标记路径、登记表路径、临时目录与 EXIT 清理 |
| 4 | 53–55 行（`run_layer0_test_binary` 头） | 97–102 行：--full 带 `--include-ignored`，快档不带 |
| 5 | 86–87 行（`log="$(mktemp)"` 那两行）之前插入 | 133–250 行：`write_layer0_input_manifest`、`fail_without_input_manifest`、`report_manifest_differences`、`run_quick_tier_of_stream` 四个函数；快档整段（跑两条流、没标记判红、哈希不等判红并列出不同的文件、标记里计数行不是恰好三行判红、判绿报计数并原样带出标记里的三行计数与跑完时刻，`exit 0`）；--full 开跑段（先删旧标记、记开跑时刻与输入清单） |
| 6 | 142 行（最后一句 ✓）之后追加 | 307–338 行：--full 判绿之后再算一次输入，与开跑时不同判红不写标记；相同则先写 `<标记>.partial.<pid>` 再 `mv` 成标记，写不成判红 |

原有几样的去向：线程数判定（`worker_threads_are_acceptable`）、`LAYER0` 下一行不是 `CHECKER` 判红（门禁修复会话那一块，一字未动）、`exhaustive=true` 判定、两条流的 release 全量跑法，都原样留在 --full 那条路上；改动范围没碰输入退 77、`stage-must-run.sh` 复用判定原样留在快档上。

标记：`$(git -C <根> rev-parse --path-format=absolute --git-common-dir)/singlefs-layer0-full-green`，真仓里就是 `/home/fy5090/code/singlefs/.git/singlefs-layer0-full-green`。内容（自证临时仓里写出的一份原样见附录 A 开头）：`input_hash=`、`input_file_count=`、`input_paths=`、`started_utc=`、`finished_utc=`、`judged_root=`、`worker_threads=`、`LAYER0 …`、`CHECKER …`、`LAYER0B …` 三行原样，再逐文件一行 `input_file <sha256>  <路径>`。

输入哈希：路径取 `stage-inputs.tsv` 里 `54-layer0-replay.sh` 那一行（`crates/ Cargo.toml Cargo.lock .claude/kb/layout/ research/results/`）；文件集是 `git ls-files -z --cached --others --exclude-standard -- <这几条>` 里磁盘上真存在的文件，`LC_ALL=C sort -zu` 排序，逐个 `sha256sum`，整张清单再 `sha256sum` 一次。不按 git 对象算；附录 A「--staged 那条路」一段证明工作区跑的 --full 与临时 worktree 里的快档对同一份内容算出同一个哈希。

```

**出处 `research/prompts/defs-gate54-tiering-implementer/r1-report.md:24-35`（整段抄，未转述）**

```markdown
## 四样自证（临时仓 + 打合成日志的假 cargo）

做法：`/tmp/claude-1000/gate54-tiering/selftest-repo`（`git init` 的临时仓，只放 54 号、`stage-inputs.tsv`、`stage-must-run.sh`、`change-touches-crates.sh` 与五个假输入文件），`/tmp/claude-1000/gate54-tiering/fake-bin/cargo` 按 `--include-ignored` 有没有打全量或快档的合成日志，环境变量控制缺 `CHECKER` 行、cargo 判红、第二条流跑时改一个输入。驱动脚本 `research/prompts/defs-gate54-tiering-implementer/selftest-driver.sh`，全部原样输出 `/tmp/claude-1000/gate54-tiering/selftest-output.txt`（194 行，全文在附录 A）。快档用 `SINGLEFS_GATE_FULL=1` 越过「没碰 crates/ 退 77」那一问（临时仓相对 HEAD 没改动）。
不在真仓里做这几样，是因为合成日志跑出来的 --full 会在真仓 common-dir 里留一份假的全绿标记。

1. **标记相等判绿**：附录 A「自证一」，退出码 0，成功句报「第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored」并原样带出标记里的 `LAYER0` / `CHECKER` / `LAYER0B` 三行与 `全量跑完于 2026-09-23T17:13:31Z`。
2. **改一个输入文件一个字节判红**：`cmp` 报 `differ: byte 13, line 1`；附录 A「自证二」，退出码 1，列出「内容不同：crates/singlefs-harness/src/lib.rs」，出路句是「这批输入的层 0 全量没在收尾跑过：跑 `bash .claude/gate.d/54-layer0-replay.sh --full`」。改回那个字节再跑，退出码 0（证明红的就是那一个字节）。
3. **删掉标记判红**：附录 A「自证三」，退出码 1，同一句出路。真仓里也现跑了一次（下一节），真仓本来就没有标记，同样判红。
4. **--full 判红时不写标记**：附录 A「自证四」两支——合成日志里 `LAYER0` 下一行不是 `CHECKER`（门禁修复会话那一支判红，退出码 1）、假 cargo 退 101（「层 0 崩溃点重放的用例判红」，退出码 1）；两次跑完都是「[标记不在]」（开跑前标记在，开跑先删、判红不写）。紧接着跑快档，退出码 1、报没有标记。

外加两样（附录 A）：--full 跑到第二条流时改了一个输入 ⇒ 判红「全量跑的过程中这一道的输入变了」、列出那个文件、不写标记；照 `gate.sh --staged` 的建法（`worktree add --detach HEAD` + `apply --index` 暂存区的 diff）建临时 worktree，工作区跑的 --full（新文件已暂存）⇒ 临时 worktree 里快档读 common-dir 同一份标记判绿；工作区再多一个没暂存的输入文件、重跑 --full ⇒ 临时 worktree 里快档判红、列出「只在标记里：research/results/e999-other-session.out」。认不出的参数 `--fast` ⇒ 退出码 2 并给用法。

```

**出处 `research/prompts/defs-gate54-tiering-implementer/r1-report.md:36-49`（整段抄，未转述）**

````markdown
## 真仓里快档真跑一次

命令：`nice -n 19 bash .claude/gate.d/54-layer0-replay.sh`（不带 `--full`，工作区，没设 `SINGLEFS_STAGED_TREE`，所以复用那一问答「要跑」，改动范围那一问答「碰了」）。开跑 2026-09-23T17:12:35Z，结束 17:16:39Z（挂钟 4 分 4 秒，大半是 release 编译）。ps 看到的 cargo 命令行是 `cargo test --release -p singlefs-harness --test first_transaction_step_seven_layer0 -- --nocapture`（没有 `--include-ignored`）。
全文 `/tmp/claude-1000/gate54-tiering/real-quick-run.log`，其中 `LAYER0_PROGRESS` 转发行 123 行（`grep -c`）；末尾原样：

```
  ✗ 快档绿了（第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 8 条通过、1 条 ignored），但没有层 0 全量的全绿标记（/home/fy5090/code/singlefs/.git/singlefs-layer0-full-green）
     → 怎么办：这批输入的层 0 全量没在收尾跑过：跑 `bash .claude/gate.d/54-layer0-replay.sh --full`
exit=1
finish=2026-09-23T17:16:39Z
```

跑完 `ls` 真仓 common-dir：`ls: cannot access '/home/fy5090/code/singlefs/.git/singlefs-layer0-full-green': No such file or directory`——真仓里没有留下标记。真仓的 --full 没有真跑（按派发，机器被编译占满；两条流合计约 4 小时）。

````

**出处 `research/prompts/defs-gate54-tiering-implementer/r1-report.md:50-65`（整段抄，未转述）**

````markdown
## 两个 lint（改完之后跑，原样）

```
══ 门禁自检（每条拒绝都要给出路） ══

  ✓ 门禁自检通过：83 个脚本（.sh 与 .py）、225 条拒绝都带了出路
gate-lint 退出码 0

══ shell 纪律检查 ══

  ✓ shell 纪律检查通过（共 76 个脚本）
shell-lint 退出码 0
```

改前基线同一对命令是「215 条拒绝都带了出路」与「共 76 个脚本」；多出的 10 条是 54 号新加的 10 处 ✗。`bash -n` 语法检查通过。

````

**出处 `research/prompts/defs-gate54-tiering-implementer/r1-report.md:66-79`（整段抄，未转述）**

```markdown
## 要主 agent 判的几件事（我没改）

1. **54 号的登记输入比它实际读的宽**：两个测试文件与 `crates/singlefs-harness/src/` 里没有读 `research/results/` 或 `.claude/kb/layout/` 的代码（`grep -rn "fs::\|File::\|OpenOptions\|env!(\|include_"` 在两个测试文件、`tests/common/mod.rs`、`src/*.rs` 里只命中删临时文件、写镜像、`include_str!("model.rs")` 与 `write` 录制流）；两处只在注释里引。按登记表算哈希，每出一份新的实验产物（写报告时 `git ls-files --others --exclude-standard -- research/results/` 数到 11 个未跟踪的产物）或改一次布局表，标记就失效、要重跑约 4 小时的全量。收不收窄登记表是主 agent 的事：它同时管 `stage-must-run.sh` 的复用判定，登记表文件头写的是「宁宽勿窄」。
2. **别的会话没提交的输入会让 `--staged` 下判红**：文件集含未跟踪的文件（今天 `crates/` 下有 20 个未跟踪的文件，其中 6 个是 `singlefs-core` 与 `singlefs-harness` 的 `lib.rs` 用 `pub mod` 引着的源文件；只看索引的话 --full 算的哈希会漏掉它们，暂存之后再比就对不上）。代价是工作区里别的会话的未跟踪、未暂存输入会进工作区跑出的哈希，暂存树里没有它们。出路句已写：在只含这一批的 worktree 里跑 `--full <它的根>`（标记在 common-dir，各 worktree 共用）。
3. **哈希里没有 54 号脚本自己和登记表文件**：按派发只算登记的路径。登记表改了会让文件集变、哈希跟着变；54 号脚本改了（比如收严线程判定）不会让标记失效。要不要把脚本自己算进去，主 agent 定。
4. **快档照旧把 `LAYER0_PROGRESS` 转到门禁输出**：真仓一次 123 行。那是给全量看进度用的，快档不需要，但删不删属于行为改动，没动。
5. **跟着要改的定义与文档**（约束里不许我碰，列给主 agent / sweep）：
   - `.claude/agents/crash-verifier.md` 第 23、24 行：照 `stage-owners.tsv` 第 37 行跑 `nice -n 19 bash .claude/gate.d/54-layer0-replay.sh`，现在只跑快档；要它跑全量得写 `--full`。
   - `.claude/main-agent.md`：「任务收尾」是哪一步、在那一步跑 `bash .claude/gate.d/54-layer0-replay.sh --full`，还没写（记录里的要求：`records/2026-09-19-里程碑二遗留收拢.md` 第 147 行「改之前先写清『任务收尾』是哪一步（阶段同步那一步）」）。
   - `README.md` 第 107 行：`bash .claude/gate.d/54-layer0-replay.sh   # 单跑层 0 崩溃点重放全量（release）`，现在这条命令是快档。
   - `.claude/rules/implementation-workflow.md` 第 65 行说 54 号判线程数；这一判现在只在 `--full` 里。
   - `.claude/kb/milestone/02-second-txn.md` 第 379 行（收口表第 41 行）的状态要回写。
6. 文件头「池级 checker（23 条不变量）」是原文照留，没现查它是不是还对。

```

**出处 `research/prompts/defs-gate54-tiering-implementer/r1-report.md:80-86`（整段抄，未转述）**

```markdown
## 没做什么

- 没真跑 `--full`（派发说不用；两条流合计约 4 小时，机器负载开跑时 83）。--full 的判定段、标记写入、跑中输入变化判红，只在临时仓用合成日志走通；真全量跑出的 `LAYER0B` 行长度、跑 4 小时期间别的会话改输入的频率，都没在真仓验过。
- 没做 fixtures 样本（54 号原本就没有，判红要 cargo 真跑）；四样自证的驱动脚本在 `/tmp`，没进仓。
- 没改 `stage-inputs.tsv`、`crates/`、kb、定义文件；没暂存、没提交。
- 没跑整轮门禁，也没跑别的阶段。

```

**出处 `research/prompts/defs-gate54-tiering-implementer/selftest-driver.sh:1-57`（整段抄，未转述）**

```markdown
#!/usr/bin/env bash
# 54 号分档的四样自证（外加两样）：临时仓 + 打合成日志的假 cargo。每一步打原样输出与退出码。
set -uo pipefail
base=/tmp/claude-1000/gate54-tiering
repo="$base/selftest-repo"
export PATH="$base/fake-bin:$PATH"
unset SINGLEFS_LAYER0_THREADS SINGLEFS_STAGED_TREE GATE_BASE
stage=".claude/gate.d/54-layer0-replay.sh"
marker="$(git -C "$repo" rev-parse --path-format=absolute --git-common-dir)/singlefs-layer0-full-green"
run_step() { # <标题> <命令…>
  local title="$1"; shift
  echo "════ $title"
  echo "\$ $*"
  "$@"
  echo "[退出码 $?]"
  if [[ -f "$marker" ]]; then echo "[标记在：$(sed -n 's/^input_hash=//p' "$marker" | cut -c1-16)…，$(wc -l < "$marker") 行]"; else echo "[标记不在]"; fi
  echo
}
cd "$repo" || exit 2
run_step "准备：--full（假 cargo 打全绿的合成日志）写标记" bash "$stage" --full
echo "──── 标记全文 $marker"; cat "$marker"; echo
run_step "自证一：标记与输入相等 ⇒ 快档判绿" env SINGLEFS_GATE_FULL=1 bash "$stage"
cp -p crates/singlefs-harness/src/lib.rs "$base/lib.rs.before-byte-change"
printf 'pub fn marker() {}\n' | sed 's/marker/markes/' > crates/singlefs-harness/src/lib.rs
echo "改一个字节：cmp 原文件与改后："; cmp "$base/lib.rs.before-byte-change" crates/singlefs-harness/src/lib.rs; echo
run_step "自证二：改一个输入文件的一个字节 ⇒ 快档判红" env SINGLEFS_GATE_FULL=1 bash "$stage"
cp -p "$base/lib.rs.before-byte-change" crates/singlefs-harness/src/lib.rs
run_step "改回那个字节 ⇒ 快档又判绿（证明自证二红的就是那一个字节）" env SINGLEFS_GATE_FULL=1 bash "$stage"
rm -f -- "${marker:?}"
run_step "自证三：删掉标记 ⇒ 快档判红" env SINGLEFS_GATE_FULL=1 bash "$stage"
run_step "准备：--full 全绿，标记重新写上" bash "$stage" --full
run_step "自证四：--full 喂 LAYER0 下一行不是 CHECKER 的合成日志 ⇒ 判红、不写标记（旧标记开跑时已删）" env FAKE_CARGO_DROP_CHECKER=1 bash "$stage" --full
run_step "自证四之后：快档判红（没有标记）" env SINGLEFS_GATE_FULL=1 bash "$stage"
run_step "准备：--full 全绿，标记重新写上" bash "$stage" --full
run_step "自证四（另一支）：--full 里 cargo 判红 ⇒ 判红、不写标记" env FAKE_CARGO_FAIL=1 bash "$stage" --full
run_step "准备：--full 全绿，标记重新写上" bash "$stage" --full
run_step "外加：--full 跑到第二条流时改了一个输入 ⇒ 判红、不写标记" env FAKE_CARGO_TOUCH_DURING_SECOND="$repo/research/results/e142-sample.out" bash "$stage" --full
cp -p "$base/lib.rs.before-byte-change" crates/singlefs-harness/src/lib.rs
printf 'result\n' > research/results/e142-sample.out
echo "════ 外加：--staged 那条路（照 gate.sh --staged 的建法：worktree add HEAD + 套暂存区的 diff）"
printf 'pub fn staged_change() {}\n' > crates/singlefs-harness/src/new_module.rs
printf 'pub fn marker() {}\npub mod new_module;\n' > crates/singlefs-harness/src/lib.rs
git add crates/singlefs-harness/src/new_module.rs crates/singlefs-harness/src/lib.rs
git status --short
run_step "工作区里跑 --full（新文件已暂存，工作区与暂存区一致）" bash "$stage" --full
staged_worktree="$base/selftest-staged-worktree"
git diff --cached --binary > "$base/selftest-staged.patch"
git worktree add -q --detach "$staged_worktree" HEAD
git -C "$staged_worktree" apply --index "$base/selftest-staged.patch"
run_step "临时 worktree 里跑快档（读 common-dir 里同一份标记）⇒ 判绿" env SINGLEFS_GATE_FULL=1 bash "$staged_worktree/$stage" "$staged_worktree"
printf 'another session\n' > research/results/e999-other-session.out
run_step "工作区多了别的会话没暂存的一个输入文件，再跑 --full" bash "$stage" --full
run_step "临时 worktree 里跑快档 ⇒ 判红，列出那一个文件" env SINGLEFS_GATE_FULL=1 bash "$staged_worktree/$stage" "$staged_worktree"
git worktree remove --force "$staged_worktree"
echo "════ 参数：认不出的参数判红"
bash "$stage" --fast; echo "[退出码 $?]"

```

**出处 `records/2026-09-19-里程碑二遗留收拢.md:147-147`（整段抄，未转述）**

```markdown
| 8 | 全量测挂在哪个节点 | 原话：「每次主 agent 执行完任务后统一执行」 | 收口表第 41 行：层 0 全量不再每次门禁都跑，改在主 agent 一个任务收尾时统一跑一次；门禁 54 号怎么分档要改（快档每次、全量在收尾），改之前先写清「任务收尾」是哪一步（阶段同步那一步） |
```

**出处 `.claude/kb/checks-owed.md:22-22`（整段抄，未转述）**

```markdown
| C8 | 门禁范围判不出来 | **一次改动的门禁范围判不出来，于是要么全跑（挂钟炸）要么少跑（有布局没被验）** | 门禁按 diff 计算**受影响的布局集合**并打印：diff 只碰某一条布局的专属路径 ⇒ 只跑那一条；碰到共享代码（事务层、`CommitStep`、格式常量、checker 框架）⇒ **强制全跑**。取不到这个集合、或集合为空而 diff 非空，即判红。故障注入自证：造一个只改共享枚举的 diff，门禁必须判为全跑；造一个只改单布局的 diff，必须判为只跑那一条 | 布局清单文件、布局专属路径与共享路径的目录约定、事务层存在（事务层 2026-09-14 已有；布局清单文件 2026-09-21 随 C14（格式变了 checker 没跟） 有了：`.claude/gate.d/layouts.tsv` 一套布局一行，登记位号、格式定义路径与 checker 判定路径，门禁 92 号读它——它给的是「布局 → 路径」这一层映射，C8（门禁范围判不出来） 还欠「专属路径与共享路径怎么分」那一层） | **判据从「总时长」改成「范围可判定」（2026-08-28 用户定案）**：单次改动只动一条分支时代价是 O(1)，语料总量大不构成成本——人只是异步等结果，等的时候在做别的事。⚠️ 但「只动了一条」是**需要被证明的断言，不是事实**：btrfs 的 `btrfs_is_zoned` 散到 93 处，正是一次次「我只是加个 zoned 支持」累积出来的，每一次都以为自己只动了一条。⇒ **让 O(1) 成立的东西是这条检查，不是跑得快。** 与 C19（介质判定渗进事务层） 成对：C19（介质判定渗进事务层） 拦渗透，C8（门禁范围判不出来）拦「渗透了但门禁没扩范围」 ⚠️ **2026-09-21 实测这个前身挡不住同一批改动里的二次重跑**：它判不出「这道阶段上次跑绿之后那些输入有没有再变过」：它按一个 diff 基准判「碰没碰前缀」，而一批改动在暂存区里躺着的时候，相对任何固定基准都一直是「碰了」。一轮里第二、三次跑门禁时 `crates/` 相对基准照旧是「碰了」，于是层 0、QEMU、herd7、59 号又全跑一遍——同一份代码那一轮验了三遍，两遍纯属白跑（约 80 分钟）。`.claude/rules/implementation-workflow.md`「复用上一次全量门禁的判定」写着该怎么复用，而那是一段要人记得去读的文字：主 agent 当轮读过、还引过那一条，照样全跑了两次。⇒ 这一条要的不是更准的 diff 判定，是**每道阶段记住自己上次跑绿时读到的那批输入的指纹**，下次先比指纹、相同就退 77 并打印上一次的判定行。⚠️ **2026-09-20 落了一个粗粒度前身，这一条照旧欠着**：`research/scripts/change-touches-crates.sh` 按 diff 判「碰没碰这道阶段自己的输入前缀」，没碰就让那道阶段退 77（本次未跑，不记通过也不算覆盖），判不出来一律当成碰了；接进门禁 54、55、74、87 号四道重阶段（2026-09-22 起降为第二道闸：这四道先问 `stage-must-run.sh` 按 `.claude/gate.d/stage-inputs.tsv` 登记的具体路径比对，判定要跑时才轮到它再判一次「碰没碰前缀」这个更粗的信号），自证由门禁 47 号复跑。它只摘得掉「零行代码的改动」（实测当天一次 44 个文件、零行 `crates/` 的提交，四道重阶段照样要跑，18 分钟没跑完），**摘不出 C8（门禁范围判不出来）要的那个按布局算的受影响集合**：碰到 `crates/` 里任何东西都当成碰了共享代码、一律全跑，所以「只跑那一条布局」那一半一个字都没做。⚠️ **2026-09-21 给这个前身补了一刀：基准从 `GATE_BASE` 改成恒取 `HEAD`**。`GATE_BASE` 答的是「上次过闸以来有没有人改过」（共享 gate.sh 为 show-me-test 而设，没有 `refs/sop/gate-ok` 时退到 `HEAD~1`；而 `--staged` 跑绿不会让那个 ref 前移，所以几个会话共写的仓里它永远不存在），拿它当基准，**上一个提交已经验过的同一份 `crates/` 改动会重新落进这一轮**：实测那天暂存区零行 `crates/`，相对 `HEAD~1` 却有 5 个 `crates/` 路径，全是上一个提交改的，四道重阶段白跑 40 分钟。改成 `HEAD` 之后这一类摘掉了，判别力自证是第 6 格（把基准改回 `GATE_BASE`，那一格必须判红）。**同一轮里跑第二、三次仍然重跑**：那批暂存改动相对 HEAD 一直是「碰了」，摘掉它要记每道阶段上次跑绿时的输入指纹——**2026-09-22 落了**：`research/scripts/stage-must-run.sh` 拿 `refs/sop/staged-green`（上次整轮全绿那棵暂存树）与这一次的暂存树做 git diff 当指纹，54、55、59、74、87 五道已接上，同一轮里跑第二、三次只要那几条路径没变就直接退 77。这三条拦路的理由，按今天这个做法一条都不成立：指纹是 git 自己算的树、不另外落盘；git 对象不会腐化；新 clone 没有那条 ref 时退回全跑，是安全的方向。**仍欠的是 C8（门禁范围判不出来） 正题那一半**——按布局算受影响集合，这一条给的是按阶段算输入，粗得多 |
```

**出处 `.claude/kb/checks-owed.md:511-511`（整段抄，未转述）**

```markdown
| C327 | 变异表的锚点腐化没有会红的检查 | 2026-09-16 门禁 33 号加子串计数（仍不跑变异）：每张表逐条核「原文」在同名源码里恰好命中一次，命中不是 1 次就判红并列出表名、条目名、表行号与命中数，red / green 两个样本双向证过会红。同日先把 4 条腐化的锚点修到命中 1 次——e143 两条随常量改名跟上、e79 一条随 `LOC_ENTRY` 跟上、e67 一条多带一行上下文变唯一——各跑一次 `mutate.sh` 证明仍被抓（e143 十条全抓、e79 六条全抓、e67 的 M2 被抓），全仓 142 张表 1396 条复查全部命中 1 次 | 2026-09-16 |
```
