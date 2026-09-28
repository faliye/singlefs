# 第四十节「要改定义」的几条落进定义：交回报告（2026-09-26）

## 一、结论一览

| 项 | 做了什么 | 在哪 |
|---|---|---|
| 第 1 行（本地攻方写范围与产出矛盾） | `three-way-local-attack.md`「写范围」加上运行记录与草稿目录；`three-way-local-defense.md` 改读本地攻方「做什么」「写范围」「产出」三节，「文件名形态」一行写明两节里的文件名照它换 | 两份定义；记录第 1 行现状格 |
| 第 29 行 ②（攻方取样与建池） | `three-way-attack.md`「做什么」加第 3b 步（内存稀疏盘、起点只建一次池、先估时长超 40 分钟缩取样、崩溃点逐点穷举不缩、checker 判结束状态、随机跑批小批量限时）；本地攻方不加 | 定义；记录第 29 行现状格 |
| 第 33 行 ①（判决行 false 不报） | `experiment-runner.md`「做什么」加第 4c 步、「产出」加一项：逐行读 `name=verdict`，`false` / `not_run` 逐个点名 | 定义；记录第 33 行现状格 |
| 执行员经内存包装 | `experiment-runner.md`「做什么」加第 1b 步；`agent-common.md` 里没有同类的一句（`grep -n 'run-with-memory-cap\|内存包装\|经包装' .claude/agents/*.md .claude/agent-common.md .claude/main-agent.md` 改前零命中），没改它 | 定义；记录第 30 行现状格（理由见第三节） |
| 97f5904 七份定义逐 hunk 核判决 | 12 个 hunk、拆成 34 块逐块对：有一格专门判过 8 块；完全不在任何一轮 diff 里的 3 块（全在 `main-agent.md`）；腿写了逐项核而没引原文、或腿判了而判决没接的 6 块；只被通查扫过、没有一格判内容的 17 块 | 第五、六节 |
| 第 40 行两种判法 | 参数白名单与阶段自报各写了做法与代价 | 第七节 |
| 门禁 | 47、63、doc-lint、规则纪律（项目本地）绿；**62 号红**，红在 `84-verdict-false-named.sh` 没登记进归属表，不是这一轮的文件 | 第八节 |

改动只在 `.claude/agents/` 四份与记录的四格；`.claude/agent-common.md`、`.claude/main-agent.md`、hooks、SOP 副本、`crates/`、kb 都没碰。

## 二、每份定义改了哪一节

| 文件 | 节 | 改了什么 |
|---|---|---|
| `.claude/agents/three-way-local-attack.md` | 「写范围」 | 加「「产出」一节的运行记录（`research/prompts/<轮>-local-attack-runlog.md`）、草稿目录」。草稿目录是顺带：「输入」一节要主 agent 给草稿目录，而写范围原来写「除此之外不写」，同一类矛盾 |
| `.claude/agents/three-way-local-defense.md` | 开头读取指令、「与本地攻方不同的地方」末条 | 读本地攻方的「做什么」「写范围」「产出」三节（原来两节，读不到运行记录那一项）；「文件名形态（「写范围」「产出」两节里的文件名照这里换）」 |
| `.claude/agents/three-way-attack.md` | 「做什么」新第 3b 步 | 四条取样规矩，照 `research/prompts/_m2-rollback-forward-r3-body.md` 第 58 行「取样」的写法收进来 |
| `.claude/agents/experiment-runner.md` | 「做什么」新第 1b 步、新第 4c 步；「产出」 | 1b 内存包装；4c 判决行点名；「产出」报告项加「判决行的点名（第 4c 步）」 |

写的时候自己加进去、原文没有、要三方判的几处：

1. 第 3b 步「估出来超过 40 分钟，先缩历史、候选与几何的取样」「崩溃点不在缩的范围里」：原文只写「超过 40 分钟的先缩取样」。不这么分，它与 `.claude/agent-common.md` 第 52 行「**崩溃点测试不衡量时间成本，也不为省时间缩范围**」直接冲突。现查有先例正落在这条缝上：`research/prompts/m2-rollback-forward-r2-opus-output.md` 第 284 行「整段子集枚举没做（闭式数 4259850 / 1572877 / 327690，合计按上面的速度估超过 40 分钟）」——腿按 40 分钟缩掉的正是崩溃点的整段子集枚举。「逐点穷举」指每个前缀点、还是连段内子集（层 0 那种）一起，原文没说，我照原文写「逐点穷举」，没替它定。
2. 第 3b 步「checker 判结束状态」照原文字面留着，没解释成「每段历史 / 每个崩溃状态跑完判一次」：原文两种读法都通。
3. 第 3b 步点名的 `SparseBlockDevice`、`SparseDevice`、`MemoryPool`、层 0 那一套都在 `crates/singlefs-harness/src/crash.rs`（`SparseDevice` 第 37 行 derive `Clone`，`SparseBlockDevice` 第 113 行，`MemoryPool` 第 172 行 derive `Clone`，`CrashImage` 第 363 行）。记录第 29 行 ① 的「公用工具」还没做，第 3b 步指的是今天已有的这几样；① 做出来之后要改指它。
4. 第 1b 步「上限取跑前登记或派发提示给的，都没给就取 `replay.sh` 文件头 `REPLAY_MEMORY_CAP` 的默认值」：原文没说上限怎么取，`run-with-memory-cap.sh` 文件头写「默认值由调用方定，写在调用方的脚本头」，我指到 `replay.sh` 第 24 行 `REPLAY_MEMORY_CAP="${REPLAY_MEMORY_CAP:-8G}"`，不在定义里写数。
5. 第 1b 步「`mutate.sh` 与 `replay.sh` 在里面逐条套了，直接跑，外面不再包一层」：套了是现查的（`mutate.sh` 第 44–47 行 `MEMORY_CAP_RUNNER`、`replay.sh` 第 397、442、443、484、504、522 行）。「外面再包会怎样」是推的，没量过：外层 scope 的账上记一笔、里层每条再排队，外层那一笔在里层跑的时候不涨量，按包装文件头「已占」的算法会被重复算进去，可能排到 `RUN_WITH_MEMORY_CAP_WAIT_SECONDS` 退 252。
6. 第 1b 步把 `cargo test` 也写进去：任务单只点了「装置二进制与 cargo run」，而 `.claude/hooks/heavy-test-guard.sh` 第 74–76 行那一道对子 agent 的 cargo test 同样要求经包装（记录第 30 行 ④），执行员第 2 步要跑单测，不写就照定义做会被拒。
7. 第 4c 步「逐字节一致没新存的，读 `research/results/` 里那一份」：重跑已有实验时第 4 步不新存产物，不写这一句就没有产物可读。

## 三、记录第四十节改了哪几格

用 `research/scripts/replace-once.py` 定点改，四格都「命中 1 次，已替换…并回读确认」；日期后来统一改成 `2026-09-26`（同样四次定点替换）。

| 行 | 原来 | 改成（要点） |
|---|---|---|
| 第 1 行 | 「没改。改定义要走一轮三方（门禁 72 号）」 | 「定义已改、待定义三方（2026-09-26）」＋两份文件各改了哪一节 |
| 第 29 行 ② | 「…——改定义走 72 号那一轮三方，与 ① 一起做。」 | 「…——定义已改、待定义三方（2026-09-26，先于 ① 做）」＋第 3b 步、指今天已有的装置、缩取样不缩崩溃点、本地攻方不加 |
| 第 33 行 ① | 「（改定义走门禁 72 号那一轮三方）」 | 「（定义已改、待定义三方（2026-09-26）：`experiment-runner.md`「做什么」加第 4c 步、「产出」一节加判决行的点名，`false` 之外连 `not_run` 一起点名）」 |
| 第 30 行「欠」 | 「欠：崩溃验证员、门禁分诊员、实现员、执行员的定义里还没写经包装跑（…）」 | 执行员拆出来写「定义已改、待定义三方」＋第 1b 步；崩溃验证员、门禁分诊员、实现员三份照旧欠 |

**执行员内存包装落在第 30 行、没落第 35 行**：「定义里还没写经包装跑」这笔欠账写在第 30 行现状格；第 35 行说的是 `replay.sh` 驱动函数，现状格是「已改（2026-09-26）…」，没有定义那一半可改。另外，派发提示说「E142 第十七次跑与第十八次跑的执行员都被拒过（第 35 行一带有记录）」：第 35 行只记了第十七次跑（「2026-09-26 E142 第十七次跑的执行员跑 `replay.sh E142` 被 …拒」）；`grep -c '第十八次' records/2026-09-16-subagent拆分提案.md` 是 0，第十八次跑被拒这件事这份记录里没有。

第 30 行还欠的三份（`crash-verifier.md`、`gate-triage.md`、`implementation-writer.md`）不在这一次的派发里，没动。实现员第 4 步要跑「动到的测试二进制」，照今天的定义去跑会被 `heavy-test-guard.sh` 拒；要不要并进这一轮定义三方，主 agent 定。攻方腿同样：`_m2-rollback-forward-r3-body.md` 第 59 行「**内存与进程**：编译与跑一律经 `bash research/scripts/run-with-memory-cap.sh <上限> <命令…>`…」这一条，派发正文逐轮写着，没进 `three-way-attack.md`（任务单只点了「取样」那一条）。

## 四、定义 diff（`git diff -- .claude/agents/`，原样）

```diff
diff --git a/.claude/agents/experiment-runner.md b/.claude/agents/experiment-runner.md
index cf04f24..866a37e 100644
--- a/.claude/agents/experiment-runner.md
+++ b/.claude/agents/experiment-runner.md
@@ -24,11 +24,13 @@ omitClaudeMd: true
 ## 做什么
 
 1. 开跑前照共用约束「不做」一节看负载。登记第六节里有耗时类的量时，跑产物之前连别的 `cargo`、`gate.sh` 也要等。
+1b. 跑单测（`cargo test`）、`cargo run` 与装置二进制（cargo 编出来的可执行文件，例 `research/target/release/e<号>-<简称>`）一律经 `bash research/scripts/run-with-memory-cap.sh <上限> <命令…>`；写进脚本的，整条脚本经它跑（`bash research/scripts/run-with-memory-cap.sh <上限> bash <脚本>`）。上限取跑前登记或派发提示给的，都没给就取 `research/scripts/replay.sh` 文件头 `REPLAY_MEMORY_CAP` 的默认值。`mutate.sh` 与 `replay.sh` 在里面逐条套了，直接跑，外面不再包一层。退出码 250–254 是包装自己的结局（含义见 `research/scripts/run-with-memory-cap.sh` 文件头「退出码」一段）：那一次的输出不算产物，命令与退出码写进报告，不绕开包装重跑。
 2. 写 `research/e7-index-bench/src/bin/e<号>_<简称>.rs`，在 `research/e7-index-bench/Cargo.toml` 末尾追加它的 `[[bin]]`（⚠️ **跑前登记把装置写死成「入库装置」时改写 `crates/singlefs-harness/src/bin/e<号>_<简称>.rs`**：岔路问的是「`crates/` 今天这份代码的性质」时，另写一份独立模型答不了，写范围闸按 `crates/singlefs-harness/src/bin/e*.rs` 放行。那种 bin 只读、只驱动与观测，不许改 `crates/singlefs-core` 与 `crates/singlefs-checker`；变异表改追加进 `crates/mutations.tsv` 末尾、归门禁 59 号跑。**另有三条只对入库装置生效**：① **不许从 `crates/` 里引决定工作量、几何或判据门槛的常量**——这类参数在装置里写本地常量，值抄自 kb 的登记位并注明文件与行号，另加一条断言拿它与实现侧同名常量回比；引了就等于装置与实现共用一个数，两边一起错而逐格相等。这一条挡不住「kb 与实现一起取错」。② **写了入库装置就要让它进代码轮判决的点名清单**：门禁 56 号的正则罩得到 `crates/*/src/bin/*.rs`，不点名那一道会红，而那条义务落在跑整轮门禁的一方身上、不落在你身上——交回时写明你新建了哪个 bin。③ **什么算「入库装置」以跑前登记里写死的那一句为准**；登记里没写死就走 `research/` 那一支，别自己判）（`name` 用连字符，与 `replay.sh` 登记行里的 bin 名一致；只追加，不改别人的）：纯算术的计数模型、单测、钉绝对值的断言；取样点覆盖登记里每一个会跑到的取值。写完跑 `bash .claude/scripts/naming-lint.sh`，新文件里的缩写与单字母名在这一步改完再往下走，不留给后面的门禁阶段。
 3. 写变异表 `research/mutations/e<号>_<简称>.tsv`，在 `research/` 下跑完整张表：`cd research && nice -n 19 bash scripts/mutate.sh <bin 名> e7-index-bench/src/bin/<源文件> mutations/<变异表>`（两个路径都相对 `research/`；bin 名以 `research/e7-index-bench/Cargo.toml` 里 `[[bin]]` 的 `name` 为准，没有显式声明的取文件名去掉 `.rs`；报「基线就是红的」时先查是不是在仓根下跑、bin 名是不是写错）（收尾要有「已还原，基线仍全绿」）；没红的逐条按 `mutation-sampling.md` 分类。
 3b. 登记里的停机条款（第十一节里标「停机」的，通常是装置与 `crates/` 实现逐项对拍）在跑产物之前做，照原文真跑，不许只推；做不了就停下交回，不跑产物、不写实验页。登记的量一次做不完的，做完的那一段照常交，没做的逐条列进报告，实验页与索引行的状态写「部分已跑」，不写「已跑」。岔路单每一行都够判了（登记第十一节的够判停机），剩下的量不跑：实验页逐条标「够判后未跑」、写明各自对应哪一行，状态写「已跑（够判）」；够不够判由你按登记写的够判条件报，续不续由主 agent 定。
 4. 跑产物进 `research/results/`；跑前删旧输出，跑完认本轮才有的完成标记。重跑已有实验时，`research/results/` 里已有的产物不覆盖：与它逐字节一致就不新存；对不上就按今天的日期另存一份，实验页两份都点名、写明对不上，交主 agent 定哪份承重。生成产物用的二进制编在 `research/target/` 下，用最后一次源码改动编出来；生成产物与第 5 步跑 `replay.sh` 时都不设 `CARGO_TARGET_DIR`；草稿目录里的 target 只拿来迭代。
 4b. 产物出来之后先自查齐不齐：按登记的「臂 × 历史 × 几何」数一遍产物里每一类结果行的条数与字段，缺行、缺字段（例如某一族没有臂字段）、某一族一次都没造出被测形状，都列进报告，不许只看末行完成标记。报告里每个判据按每条臂全列判定，不挑「突出发现」报——对某条候选不利的那一半同样要写。分族、分臂的计数用一条命令从产物里数，报告与实验页贴那条命令和它的原样输出，不用文字重写族名单；给「这几族为什么是 0」写解释之前，先把那几族的产物行贴出来，确认它们真是 0。
+4c. 交回之前逐行读判决行：`grep -n 'name=verdict' <产物>` 列出这一次每份产物里的全部判决行（逐字节一致没新存的，读 `research/results/` 里那一份；输出不截断），任何字段取值是 `false` 或 `not_run`，在报告里逐个点名：产物文件与行号、字段名、取值、为什么是这个值、是不是登记预期内的；一个都没有，也写一句「判决行 N 条，没有 false / not_run 字段」。
 5. 在 `research/scripts/replay.sh` 里登记复跑，跑一次 `bash research/scripts/replay.sh E<号>` 确认逐字节一致。送进虚机跑的实验（`vm-bench.sh`，含 `--selftest`）与 E152 装置是重型测试，你不跑：装置、登记与复跑命令写好就停，在交回里写明要跑什么、为什么，由主 agent 问用户（`.claude/kb/vm-harness.md`；共用约束「不做」一节里重型测试那一条）。装置要起子进程、读它的输出时，先把输出读完再转打，计时按登记写的进程与时钟取（`.claude/kb/vm-harness.md`「计时不在转发输出的循环里打时间戳」）；门禁 65 号查读子进程输出的循环里一边取时间一边输出。
 6. 跑阶段归属表登记给你的阶段（共用约束 `.claude/agent-common.md`「门禁」一节）；其中要编译或复跑整张表的，开跑前照共用约束看负载。15 号（编整个 research 工作区）与 87 号（复跑全部已入库实验）不归你，收尾由 `gate-triage` 统一跑；你只跑第 5 步自己那一个实验的 `replay.sh`。
 7. 主 agent 要写实验页时：`.claude/kb/experiments/<号>-<简称>.md`，结果整行抄自产物并带文件名，写复跑命令、口径、它答不了的；索引行进 `.claude/kb/experiments.md`。
@@ -40,7 +42,7 @@ omitClaudeMd: true
 
 ## 产出
 
-- 报告：单测数（一条命令数出来）、变异三个数与分类、产物路径与完成标记、`replay.sh` 结果、实验页路径；登记修订了什么（没有就写没有）。
+- 报告：单测数（一条命令数出来）、变异三个数与分类、产物路径与完成标记、判决行的点名（第 4c 步）、`replay.sh` 结果、实验页路径；登记修订了什么（没有就写没有）。
 - 岔路表：登记里岔路单的每一行一行，写「已够判 / 还差什么 / 登记里剩下的量能不能让它翻面」，每格指到产物里算出它的那条命令。主 agent 续派时照这张表点名还差的行；这张表缺了，报告不算交齐。
 
 ## 没做什么（固定会有的）
diff --git a/.claude/agents/three-way-attack.md b/.claude/agents/three-way-attack.md
index a8a6bd2..a80b51f 100644
--- a/.claude/agents/three-way-attack.md
+++ b/.claude/agents/three-way-attack.md
@@ -26,6 +26,11 @@ omitClaudeMd: true
 1. 只攻分到的攻击面，不重复前几轮攻过的角度。
 2. 每个打中给出具体的历史或输入，每一步指到被判对象里许可它的那一句。
 3. 由用户决定的动作（回退之后先做什么、建几个对象）不写死：判一条臂「同一段历史上不中」之前，把这几步放开扫一遍，只固定前缀与故障（照 `.claude/rules/three-way-inference.md`「判决由主 agent 做，不由投票做」一节「攻方腿的装置把用户动作写死时，它报的「分辨臂」可能是装置造出来的」那一条办）。
+3b. 造历史、跑模型时这样取样：
+   - 盘用内存稀疏盘（`crates/singlefs-harness/src/crash.rs` 的 `SparseBlockDevice`），不在磁盘上建镜像文件；每个起点状态只建一次池（mkfs 与起点历史只跑一次），之后每段历史从内存里那一份拷（`SparseDevice`、`MemoryPool` 都能 `clone`）。
+   - 正式跑之前先跑一小段，按它估全量的挂钟；估出来超过 40 分钟，先缩历史、候选与几何的取样，报告里写明缩了什么、缩前缩后各多少、估时怎么算的。
+   - 崩溃点不在缩的范围里：按那一串写逐点穷举（共用约束「不做」一节「崩溃点测试不衡量时间成本」那一条），用同一份文件里层 0 那一套（录制写流、按崩溃点截断重放），checker 判结束状态。
+   - 随机跑批小批量、限时，每批的段数与限时写进报告。
 4. 打中之后先答四句：分不分辨臂；被判的系统当时看不看得到判别它的东西；满足的是判据字面的哪一个分句；跑前条款给的每个改法在打中的那几格上还中不中。
 5. 自己提的改法写明「只在我的模型上量过、被攻过零轮」。写「几个改法各修哪一格」的表时，每一格标「量过」（贴副本上的原样输出）或「推的」（按代码推、没实现没跑）。
 6. 引文同正推腿：写进报告之前在被引文件里 `grep -nF` 一次，命中 0 次不许写成引文，行号取命中的那一行。
diff --git a/.claude/agents/three-way-local-attack.md b/.claude/agents/three-way-local-attack.md
index 279e15d..c1356a8 100644
--- a/.claude/agents/three-way-local-attack.md
+++ b/.claude/agents/three-way-local-attack.md
@@ -31,7 +31,7 @@ omitClaudeMd: true
 
 ## 写范围
 
-- 提示文件、核对表、样本文件（`<前缀>-output-s<n>.md`，作废副本由脚本自己写）。除此之外不写。
+- 提示文件、核对表、样本文件（`<前缀>-output-s<n>.md`，作废副本由脚本自己写）、「产出」一节的运行记录（`research/prompts/<轮>-local-attack-runlog.md`）、草稿目录。除此之外不写。
 
 ## 产出
 
diff --git a/.claude/agents/three-way-local-defense.md b/.claude/agents/three-way-local-defense.md
index ea58224..d5e2eab 100644
--- a/.claude/agents/three-way-local-defense.md
+++ b/.claude/agents/three-way-local-defense.md
@@ -8,7 +8,7 @@ omitClaudeMd: true
 
 # 本地辩方腿（three-way-local-defense）
 
-开工先读 `.claude/agent-common.md`（这份定义开了 `omitClaudeMd`，要用的规则照共用约束「规则怎么读」一节读），再读 `.claude/agents/three-way-local-attack.md` 的「做什么」「写范围」两节：做法与它逐条相同，只有下面几处不同。
+开工先读 `.claude/agent-common.md`（这份定义开了 `omitClaudeMd`，要用的规则照共用约束「规则怎么读」一节读），再读 `.claude/agents/three-way-local-attack.md` 的「做什么」「写范围」「产出」三节：做法与它逐条相同，只有下面几处不同。
 开工先读：`.claude/agents/three-way-local-attack.md` 里「开工先读：」那一行点名的小节。
 
 ## 与本地攻方不同的地方
@@ -16,7 +16,7 @@ omitClaudeMd: true
 - 立场是辩方：替被判出局、被攻、或被计划排除的一方，找它站得住的理由，写出具体机制（哪个文件、哪一步）；站不住就说站不住、为什么。辩护落得成数的（这一方在某段历史上是几、判红没有），同样按攻方第 1 步给事实表、要模型逐格填。提示里先替每一条写一个攻方问题（它被质疑的那一句，整行抄出处），再让本地模型逐条辩护；攻方问题同样要逐句核转述。
 - 输入里的「攻击面」换成「要辩护的一方」（主 agent 给：被复核的判决或被排除的候选，原文路径）。
 - 行号同攻方第 1 步：提示里明令答复不写行号，样本自带的行号在运行记录里标「模型自给、未核」。
-- 文件名形态：提示 `research/prompts/<轮>-local-defense.md`，核对表 `research/prompts/<轮>-local-defense-translation-audit.md`，样本 `research/prompts/<轮>-local-defense-output-s<n>.md`，运行记录 `research/prompts/<轮>-local-defense-runlog.md`。
+- 文件名形态（「写范围」「产出」两节里的文件名照这里换）：提示 `research/prompts/<轮>-local-defense.md`，核对表 `research/prompts/<轮>-local-defense-translation-audit.md`，样本 `research/prompts/<轮>-local-defense-output-s<n>.md`，运行记录 `research/prompts/<轮>-local-defense-runlog.md`。
 
 ## 没做什么（固定会有的）
 
```

## 五、97f5904 七份定义逐块覆盖表

怎么核的：`git show 97f5904 -- <文件>` 每个 hunk 按改动拆块；每份文件拿改前、改后 blob 的 sha256 对五轮开工快照，再把 `_m2-final-code-r2-diff.md` 第四节（`defs-vs-head.diff`，基准 index 与 97f5904 的改前相同）、`_m2-final-code-r3-diff.md` 第四节（`defs-r2-to-r3.diff`）、`_m2-final-code-r4-diff.md`（`defs-r3-to-r4.diff`）依次打到改前的 `main-agent.md` 上重建各轮冻结版（重建出来的 sha256 前 16 位 3b46ad43dd1813b0 / 6bb8187dc01ee56d / cb89635efb4a13cc，与三轮快照 `defs-sha256.txt` 逐个相同），再与 97f5904 的改后比。

| 文件 | 97f5904 改后 = 哪一轮冻结 | 改前 = |
|---|---|---|
| `agent-common.md` | m2-final-code-r4（e459af63…） | — |
| `main-agent.md` | **哪一轮都不是**：r4 冻结（cb89635e…）之后又改了两处，见第六节 | — |
| `agents/crash-verifier.md` | m2-final-code-r3（cbca0dd2…） | defs-gate54-tiering-r2 快照（3cfbf2e5…） |
| `agents/experiment-runner.md` | m2-final-code-r2（ef74ccd9…） | defs-gate54-tiering-r2 快照（bd6dcd45…） |
| `agents/gate-triage.md`、`implementation-writer.md`、`mutation-triage.md` | m2-final-code-r2（a0b04f47… / 3ce6d977… / e6ab8b41…），与 r2 diff 的 index 逐个相同 | — |

所以 defs-gate54-tiering 两轮判的是 97f5904 的**改前**（它们的快照就是改前），97f5904 新加的内容一块都不在那两轮里；那两轮判过的是改写时原样留下的旧半句（下表注明）。

判法记号：**✓** 有一格专门判了这一块；**◐** m2-final-code-r2 正推腿 Z11「重型测试清单」一格写了「6 份 agent 定义 + `main-agent.md` + `implementation-workflow.md` 与 `heavy-test-guard.sh` 逐项核」（`m2-final-code-r2-sonnet-output.md` 第 86 行），但没引这一块的原文；**△** 只被 Z11 的通查扫过（第 1 条论证句、第 7 条位置指代、照做撞不撞 hook，同一份第 117、127、131 行），没有一格判它说的内容；**✗** 不在任何一轮的 diff 里。

| # | hunk | 块 | 进了哪一轮 diff | 判决与节 | 记号 |
|---|---|---|---|---|---|
| 1 | agent-common `@@ -18,6 +18,7 @@` | 加「变体起新名字，不用角标」一行 | r2 | r2 判决第二节点名（第 55 行）、Z11 通查 | △ |
| 2 | agent-common `@@ -42,13 +43,18 @@` | 加重型测试一行 | r2 | r2 判决 Z11（第 76 行「重型测试清单与 hook 一致」）；腿第 93 行引 `agent-common.md:46` | ✓ |
| 3 | 同上 | 加「写进脚本文件、python3 subprocess、make 同样算跑」一行 | r2 | Z11 通查 | △ |
| 4 | 同上 | 加「停自己起的一条链」一行 | r2 | Z11 通查 | △ |
| 5 | 同上 | 加「读大文件先 grep 再按行段读」一行 | r2 | Z11 通查 | △ |
| 6 | 同上 | 加「线程上限：N」一行 | r2 | Z11 通查 | △ |
| 7 | 同上 | 检出 hook 那一句（只记不拦 → 对五种写法在执行前拒绝） | r2（两种）→ r3（四种）→ r4（五种） | r2 Z11 判「说反话」、改；r3 判决 Z18（第 62 行）、腿第 137 行（问 1）、第 171 行（问 3）判四种一致；r4 判决 Z23（第 51 行）、腿第 114–137 行判 `agent-common.md:57` 与检测器一致。改后逐字等于 r4 冻结 | ✓ |
| 8 | crash-verifier `@@ -1,6 +1,6 @@` | description 改成「提交代码时（或用户要求时）」 | r2 | r2 判决第二节点名（第 56 行）、Z11 通查 | △ |
| 9 | crash-verifier `@@ -10,18 +10,19 @@` | 小节名「提交前必跑 herd7 与 QEMU」→「重型测试只在提交时跑」 | r3 | r3 判决点名（第 48 行）、Z18；腿第 168 行一带把它归「标题改名、纯描述」 | ✓ |
| 10 | 同上 | 「输入」加「提交时跑还是用户要求时跑」 | r2 | Z11 通查 | △ |
| 11 | 同上 | 第 2 步改写（只在提交时跑、带 `SINGLEFS_HEAVY_TESTS` 前缀） | r2 | Z11：腿第 93 行引 `crash-verifier.md:19` 判与 hook 一致，第 133 行判命令在放行表内。原样留下的「54 号默认不带 `--full`」是 defs-gate54-tiering-r1 判决 K2（第 42–46 行）定的 | ✓ |
| 12 | experiment-runner `@@ -29,7 +29,7 @@` | 第 5 步：vm-bench 与 E152 是重型、不跑 | r2 | r2 判决点名（第 57 行）、Z11 重型测试一格 | ◐ |
| 13 | gate-triage `@@ -1,6 +1,6 @@` | description 改成「提交代码时（或用户要求时）」 | r2 | r2 判决点名（第 58 行）、Z11 通查 | △ |
| 14 | gate-triage `@@ -16,13 +16,14 @@` | 「输入」加「提交时跑还是用户要求时跑」 | r2 | Z11 通查 | △ |
| 15 | 同上 | 第 2 步改写（带前缀跑 `gate.sh --staged`，54/55/57/59 走复用） | r2 | Z11：腿第 93 行引 `gate-triage.md:19`、第 133 行判命令在放行表内 | ✓ |
| 16 | implementation-writer `@@ -25,7 +25,7 @@` | 第 4 步里重型那一半（层 0 不跑、全量与门禁留给提交时） | r2 | r2 判决点名（第 59 行）、Z11 重型测试一格 | ◐ |
| 17 | 同上 | 第 4 步其余：变异证红按测试算不按行算、其余行追加进 `mutations-append.tsv`、交回前验证只到这几样、`git apply --check` | r2 | Z11 通查 | △ |
| 18 | implementation-writer `@@ -39,7 +39,7 @@` | 「产出」报告项改成「第 4 步那几样的末尾原样输出」 | r2 | Z11 通查 | △ |
| 19 | mutation-triage `@@ -22,7 +22,7 @@` | 第 3 步：crates 那张表不跑、从 59 号输出里取 | r2 | r2 判决点名（第 60 行）、Z11 重型测试一格；腿第 133 行「不产生新命令」 | ◐ |
| 20 | main-agent `@@ -10,32 +10,41 @@` | 「禁止」节：「本机常有…」挪走，加「禁止在subagent中跑重型测试…等」 | r2 | r2 判决点名（第 61 行）、Z11；腿第 99 行拿「第 14 行…等」作对照 | ✓ |
| 21 | 同上 | 第 3 步「重型测试…提交之外要跑先弹窗」 | r2 | 腿判了：第 99 行、判定一览第 150 行「六类清单…转述判断，不落三种结论」（漏了 87 号全部实验复跑与 E152 装置、又没写「等」）；**r2 判决 Z11 那一格没接这一条**，之后几轮也没再提，今天的原文仍是六类 | ◐ |
| 22 | 同上 | 第 4 步「派实现员之前先列出它要动的 `crates/` 文件」 | r2 | Z11 通查 | △ |
| 23 | 同上 | 第 5 步「弹窗问用户之前，问句里每一句事实写出处…推的，没量过」 | **无**：r4 冻结之后加的 | 五份判决都没有；全仓判决里只有 `m2-safety-r2-main-verification.md` 用到「推的，没量过」这个写法，不是判它 | ✗ |
| 24 | 同上 | 第 6–9 步编号顺延（「本机常有…」挪到第 9 步） | 挪位在 r2（那时是第 8 步）；顺延成 6–9 不在任何一轮 | 随 #23 | ✗ |
| 25 | 同上 | 「派出去之后」第一段：hook 拒绝清单 | r2（五项）→ r3（加 ④）→ r4（加 ⑤，写窄了）→ **r4 冻结之后**再改 | r3 判决 Z18「少了起看门狗的错误写法」、改法 4（第 73 行）；r4 判决 Z23「缺 `install`、「不带 `-a`」「已存在」」（第 51 行）。改后的字面（加「起看门狗的错误写法」「`>|`、`&>`」「不带 `-a` 的」「`install`」「已存在且」）没进过任何一轮；其中 `>|`、`&>` 两项没有哪份判决要求过，`agent-common.md` 第 57 行那一句也没有这两项 | ✗（改法有判决，改后的字面没被判） |
| 26 | 同上 | 「派出去之后」第一段：看门狗叫醒条件加「子 agent 从派发起的运行时间跨过一个整小时」 | r2 | Z11 通查 | △ |
| 27 | 同上 | 同一段：叫醒条件加「本实例底下有没带 `SINGLEFS_HEAVY_TESTS` 前缀的重型测试进程」 | r3 | r3 判决点名；Z18 问 1 只引了同一行的拒绝清单那半句，没判这一条 | △ |
| 28 | 同上 | 新段「改了定义…当场给每个在跑的子 agent 发消息」 | r2（带论证尾巴）→ r3（删尾巴） | r2 判决 Z11 判「说反话」；r3 腿问 2 判删尾巴一致（第 157 行） | ✓ |
| 29 | 同上 | 「子 agent 等长活…除了例行询问不另外 ping」 | r2 | Z11 通查 | △ |
| 30 | 同上 | 「叫醒之后…整点例行询问、写进 `progress.md`」 | r2 | Z11 通查 | △ |
| 31 | 同上 | 「交回之后不再给它发消息」＋交接摘要 | r2 | r2 腿第 101 行起判与续做闸条件 ① 逐字对得上，判决 Z11 接了 | ✓ |
| 32 | 同上 | 新段「线程上限：N」 | r2 | Z11 通查 | △ |
| 33 | main-agent `@@ -43,11 +52,11 @@` | 「改 `crates/`」两行加「提交时派 `crash-verifier`…平时不跑」 | r2 | Z11 重型测试一格 | ◐ |
| 34 | 同上 | 「暂存之后、提交之前跑门禁」一行：派 `crash-verifier` 带 `SINGLEFS_HEAVY_TESTS=commit`、`gate-triage` 带前缀、用户要求时换 `=user-request` | r2 | Z11 重型测试一格。原样留下的「在 HEAD + 暂存区的 worktree 里跑 `--full`」是 defs-gate54-tiering-r1 判决 K3（第 48–62 行）、r2 判决 V1 与 V5（第 36–51、73–77 行）定的 | ◐ |

计数（上表）：✓ 8、◐ 6、△ 17、✗ 3，合计 34 块、12 个 hunk。

## 六、没被任何判决判到内容的块，交主 agent 并进这一轮

行号是今天工作区里的（`grep -n` 现取）。

**一定要并进来的（✗，没进过任何一轮的 diff）**：

| # | 文件:行 | 内容 | 说明 |
|---|---|---|---|
| 23 | `.claude/main-agent.md:22` | 第 5 步「弹窗问用户之前，问句里每一句事实写出处…推的，没量过」 | r4 冻结之后加的；它压着的钩子 `ask-user-claim-guard.sh` 的判据与这一句对不对得上，没人比过 |
| 24 | `.claude/main-agent.md:23-26` | 第 6–9 步编号顺延 | 随 #23，只是编号 |
| 25 | `.claude/main-agent.md:30` | 「派出去之后」第一段的拒绝清单今天的字面 | 改法来自 r3 Z18、r4 Z23 两份判决，改后的字面没被判过；多出来的 `>|`、`&>` 两项没有判决要求，`.claude/agent-common.md:57` 同一张清单里没有，两份又不一致了（`agent-common.md` 写「对五种写法在执行前拒绝」，列的是 `>`；`main-agent.md` 列 `>`、`>|`、`&>`）。检测器认不认 `>|`、`&>` 我没核 |

**腿判了而判决没接的（◑，并进来定个去向）**：#21 `.claude/main-agent.md:20` 第 3 步重型测试只列六类（缺 87 号全部实验复跑、E152 装置），又没写「等」；同一份第 14 行写了「等」。r2 腿判「两种读法都成立，不落结论」，r2 判决 Z11 一格没提，今天原文未变。

**只被通查扫过的（△ 17 块）**，按「这一块说的事与哪个 hook 或脚本的判据对得上」排，前几条最值得有一格专门判：

| # | 文件:行 | 内容 | 要对的东西 |
|---|---|---|---|
| 17 | `.claude/agents/implementation-writer.md:28` | 第 4 步「其余行照样追加进 `mutations-append.tsv`」 | `grep -rn 'mutations-append' .claude/ research/scripts/` 只在这一行命中 1 次；同一份第 3 步写的是「`crates/mutations.tsv` 在的话…每条再加一行变异」。两步说的是不是同一份表、`mutations-append.tsv` 是什么，定义里没写（推的，没核实现员实际怎么交） |
| 3 | `.claude/agent-common.md:47` | 写进脚本、python subprocess、make 同样算跑 | `.claude/hooks/heavy-test-guard.sh` 文件头第 69–73 行「看不见的（照常放行、不记检出）」里有 python subprocess、make、xargs 起的命令：定义说「同样算跑」，闸对这几样看不见，全靠 agent 自己守 |
| 27 | `.claude/main-agent.md:30` | 叫醒条件「没带 `SINGLEFS_HEAVY_TESTS` 前缀的重型测试进程」 | `research/scripts/agent-watch.py` 的判法 |
| 26、30 | `.claude/main-agent.md:30`、`:36` | 整点询问与 `progress.md` | `agent-watch.py` 的 `ask-every-minutes` |
| 6、32 | `.claude/agent-common.md:51`、`.claude/main-agent.md:46` | 线程上限 N 与 `capped.sh` | `research/scripts/capped.sh` 设的变量 |
| 4 | `.claude/agent-common.md:48` | 停一条链的做法 | `proc.py stop` 的行为、`bash-command-detector.sh` 的 ⑥（终止进程只许按点名的进程号，97f5904 提交说明写着 `agent-common.md` 里还没加 ⑥ ⑦） |
| 1 | `.claude/agent-common.md:21` | 变体不用角标 | `write-guard.sh` 第三道、门禁 12 号 |
| 5、22、29 | `.claude/agent-common.md:50`、`.claude/main-agent.md:21`、`:34` | 读大文件、派实现员先列文件、等长活不 ping | 纯做法，没有对应的闸 |
| 8、13、10、14、18 | 两份 description、两份「输入」加的「提交时还是用户要求时」、`implementation-writer.md:42` | 与第 2 步同一件事的另几处写法 | 与各自第 2 步一致即可 |

另外，97f5904 的提交说明自己写着「agent-common.md、main-agent.md 里列拒绝写法的地方还没加 ⑥ ⑦，改定义那一轮三方（门禁 72 号）还没走」：今天 `.claude/agent-common.md:57` 仍是「对五种写法在执行前拒绝」，⑥（终止进程只许按一个点名的进程号）⑦（原地改写已有脚本）两种没写进去。这是 97f5904 那几个 hook 改动留下的定义欠账，不是这张覆盖表里的某一块，一起交主 agent。

## 七、第 40 行：重型闸的「静态分支」怎么判，两种做法与代价（这一次不改钩子）

今天的样子（现查）：
- 闸按阶段文件名判：`.claude/hooks/lib_heavy_tests.py` 第 56 行 `STAGE_KIND = {"54": …, "55": "qemu-stage", "57": "herd7-stage", "59": "crates-mutation-stage", "87": …}`，按名字判的仓内脚本不读正文（`judged_by_name`，第 332 行）。`herd7 -version` 照拒（`heavy-test-guard.sh` 自检第 806 行「实现员 command herd7（不带 -v）照拒」）。
- 静态分支不是靠命令行参数选的，是靠被判根目录里的**标记文件**：55 号第 128–129 行 `[[ -f "$ROOT/.qemu-prerecorded" ]] && prerecorded=1`（预录档不起虚机、不编译，判全过退 3 不退 0）；57 号第 29–30 行 `[[ -f "$ROOT/.lkmm-static-only" ]] && static_only=(--static-only)`。喂样本的是 `.claude/singlefs-ai-sop/scripts/stage-selftest.sh`，把样本目录当根传进去。
- 59 号没有静态分支：它的样本也真跑 cargo 变异，只是把上限压到 512M、限时压到 20 秒（59 号第 64 行）。

| | A 参数白名单（钩子这一侧认） | B 阶段自报（阶段自己声明，钩子读声明或阶段自己守） |
|---|---|---|
| 做法 | `lib_heavy_tests.py` 加一张表：阶段号 → 哪些调用算静态。55：根参数落在 `.claude/gate.d/fixtures/55-*/` 下、且那个根里有 `.qemu-prerecorded`；57：同理认 `.lkmm-static-only`；另列 `herd7 -version` 这类只取版本号的、`--selftest`。钩子执行前读参数、看标记文件，命中就放行 | 每道重阶段在文件头写一行声明（形如 `# heavy-static: marker=.qemu-prerecorded`），钩子从阶段文件头读这一行来判，登记位只在阶段自己；再进一步，阶段在起重的那一步之前自己核 `SINGLEFS_HEAVY_TESTS`，没有前缀就拒，静态分支不核，钩子对这几道放给阶段自己判 |
| 选分支的逻辑有几份 | 两份：阶段按标记选、钩子按表认。阶段改了选法（改标记名、改成参数）钩子不跟，就误放或误拒；要一道门禁核两边一致，做到最后就是 B 的「从阶段头读」 | 一份（阶段的）。新加一道静态分支只改阶段头一行 |
| 要不要改 54、55、57、59 | 不改：不换它们的 sha256，层 0 全绿标记不作废（54 号的 sha256 在层 0 格的键里，defs-gate54-tiering-r2 判决 g1），`stage-must-run.sh` 不因此判它们要重跑 | 要改：至少 55、57 加声明行；走运行期自守的那一种连 54、59 都改。54 一改，现有层 0 全绿标记全部作废，下一次提交重跑层 0 全量；55、57、59 的输入变了，下一次提交按「阶段脚本自己进比对」重跑 |
| 能不能被骗 | 表里认的是「标记在」：标记在而分支里其实起了 qemu / herd7，闸照放。同一条命令里先 `touch` 标记再跑，钩子判的那一刻文件还不在，误拒（与第 18 行「脚本不存在」同形） | 声明是阶段自己说的：声明了静态、分支里其实起了重的，闸照放。要一条会红的检查：PATH 里放假 `qemu-system-*` / `herd7` / `cargo`，跑每个声明过的静态分支，被调用就判红（放进 `stage-selftest.sh` 或 63 号） |
| 拒绝的时机 | 执行前，与今天相同 | 声明那一种仍在执行前；运行期自守那一种挪到阶段里，子 agent 已经起了阶段（55 真档先编装置）才被拒，看门狗「没带前缀的重型测试进程」那一条也要跟着对 |
| `herd7 -version` 这类裸调用 | 表里直接列 | 没有阶段可自报，仍要一张小白名单；B 实际是「阶段自报 + 裸命令白名单」的混合 |
| 59 号的样本 | 管不了：它不是静态，是「压小了的真跑」，要放行得另判上限与限时，那是「判重不重」不是「判静不静」 | 同样管不了，理由相同 |
| 自检要加什么 | 每个静态形态一例放行、换成真根的同一条命令一例照拒；弄坏开关让表恒真时放行例转拒 | 每道阶段的声明行有一例被钩子读到；假二进制那一道要证明会红：把一个声明为静态的分支改成真起 `qemu-system`，必须判红 |

我没替它选：两种都要在定义三方那一轮判（记录第 40 行现状格写着「走定义三方」）。上表每一格说的现状都现查过；「会被骗」「时机」「作废标记」三行是按代码推的，没量过。

## 八、门禁判定行（原样）

跑的日期 2026-09-26，`ps` 看负载时本用户没有 `qemu-system`、`vm-bench`、`e152`、`fio`、`cargo`、`gate.sh` 在跑。日志在 `/tmp/claude-1000/defs-closeout-draft/gate-*.log`、`doc-lint*.log`、`rules-lint.log`。

- 47 号 `nice -n 19 bash .claude/gate.d/47-research-script-selftests.sh`，exit=0：
  `  ✓ research 脚本的自证都通过（本阶段跑了 31 条；research/scripts/ 里声称有 --selftest 的 35 份中 34 份有门禁阶段在跑）`
- 62 号 `nice -n 19 bash .claude/gate.d/62-stage-owners.sh`，**exit=1**：
  ```
    ✗ 门禁目录里有这些阶段，表里没有登记——没有哪个 agent 会在自己的活之后先跑它：
       84-verdict-false-named.sh
       → 怎么办：在 .claude/gate.d/stage-owners.tsv 给它加一行，写明哪个 agent 干完活之后该先跑它、为什么；没有合适的 agent 就写 gate-triage。
  ```
  不是这一轮的：`git status --short .claude/gate.d/84-verdict-false-named.sh` 是 `??`（未跟踪），文件 mtime 2026-09-25，是记录第 33 行 ② 那件活建的；这一轮没碰 `.claude/gate.d/`。它判的是「实验页有没有点名 false 字段」，与这一次加的执行员第 4c 步是同一件事的两半，照 `stage-owners.tsv` 的写法多半该登记给 `experiment-runner`（推的），归属表不在这一次的写范围里，交主 agent。
- 63 号 `nice -n 19 bash .claude/gate.d/63-agent-write-scope.sh`，exit=0：
  `  ✓ 写范围闸、Bash 检出 hook、重型测试闸、续派闸、续做闸与弹窗断言闸注册着、自证通过，…表与定义一致（3 个有 Write 或 Edit 的定义、18 条路径模式），…`（整行在 `gate-63-agent-write-scope.sh.log`）
- doc-lint `nice -n 19 bash .claude/singlefs-ai-sop/scripts/doc-lint.sh .`，两次都 exit=0（第二次在记录日期改过之后）：
  `  ✓ 文档铁律检查通过（检查 485，跳过 0；DOC_LINT_VERBOSE=1 看全部）`
- 规则纪律（项目本地），照 `.claude/singlefs-ai-sop/scripts/gate.sh` 第 414–420 行接法：`GATE_IN_STAGE=1 RULES_LINT_DIR="$PWD/.claude/rules" RULES_LINT_FILES="CLAUDE.md .claude/agents/*.md .claude/agent-common.md .claude/main-agent.md .claude/skills/*/SKILL.md" nice -n 19 bash .claude/singlefs-ai-sop/scripts/rules-lint.sh "$PWD"`，exit=0：
  `  ✓ 规则只写怎么做（扫了 29 份文件 1750 行；没扫 0 个；记录小节 0、论证小节 0、带日期的行 0（另有 8 行的日期只在「」或反引号里）、解释性段落 0、解释性半句 0、没带劝阻句的链接 0；词法说明判了 1627 行（围栏与表格行不判），命中 0；使用者名字这一条无对象可判：被扫的仓没有 I18N 或没登记 consumers=）`
  29 份 = `.claude/rules/` 7 + `.claude/agents/` 16 + skills 3 + `CLAUDE.md`、`agent-common.md`、`main-agent.md` 各 1。

72 号（定义改了要有判决按路径点名）没跑：这四份定义现在还没有判决，跑了必红，等主 agent 那一轮。

## 九、要三方判的点与顺带看到的

1. 第 3b 步「超过 40 分钟就缩」与共用约束第 52 行「崩溃点测试…也不为省时间缩范围」怎么分界：我写成「只缩历史、候选与几何，崩溃点逐点穷举不缩」。攻方腿 `m2-rollback-forward-r2-opus-output.md` 第 284 行按 40 分钟缩掉的是「整段子集枚举」——段内子集算不算「崩溃点」，定义没说，要判。
2. 第 3b 步「checker 判结束状态」照原文，没定是每段历史一次还是每个崩溃状态一次。
3. 第 1b 步「外面不再包一层」的理由是推的（第二节第 5 条），没量过嵌套包装会不会排到 252。
4. 第 4c 步只管报告；门禁 84 号管实验页那一半，但 84 号不在归属表里（第八节 62 号红），执行员照归属表跑阶段时跑不到它。
5. 本地攻方不加第 29 行的取样规矩：它的「做什么」六步里没有建池、跑模型、枚举崩溃点的一步，唯一的耗时是本地模型调用，已由第 6 步「连续五次调用」封顶。三方要是认为它的事实表也要自己算数（第 1 步「能落成数的问题给写死的事实表」），那一步要不要带取样规矩再判。
6. 记录第 44 行（超过 30 分钟的任务要能中断续跑，用户定「里程碑二收尾之后做」）与第 3b 步的「超过 40 分钟先缩」是两条不同的线：一条是续跑，一条是缩取样。这一次没碰第 44 行。

## 十、没做什么

- 没改 `.claude/main-agent.md`、`.claude/agent-common.md`、hooks、`.claude/gate.d/`（含 `stage-owners.tsv`）、SOP 副本、`crates/`、kb。
- 没改 `crash-verifier.md`、`gate-triage.md`、`implementation-writer.md` 的内存包装（记录第 30 行还欠的三份，不在这一次的派发里）。
- 没替第 40 行选判法，没改钩子。
- 没跑 72 号（这四份定义还没有判决，必红）、没跑任何重型测试、没编译。
- 97f5904 覆盖表里「◐ / △」两类是按腿的原文判的：腿写了「逐项核」而没引原文的，我没有替它重判那一块对不对，只标出来。
- 检测器认不认 `>|`、`&>`（第六节 #25）没核。
- 记录第 35 行没改：执行员内存包装的欠账写在第 30 行（第三节）。第十八次跑被拒这件事记录里没有，没补（不在这一次的写范围里说得清出处，主 agent 定要不要记）。
- 草稿目录 `/tmp/claude-1000/defs-closeout-draft/` 里只有文本：`before/`（改前四份定义与记录的拷贝）、`hunks/`（97f5904 逐文件 diff）、`recon/`（`main-agent.md` 各轮冻结版的重建与补丁）、`r2/r3/r4-defs-diff.txt`、门禁日志、`defs.diff` 与这份报告。没建编译目录、没拷仓副本、没起 worktree，没有要删的；后台进程只起过一个（47、62、63 串跑那一条），已经结束。
