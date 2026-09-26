# governance-defs-r2 附录二：这一轮被判的改动全文 diff（基准 71f0cbc，工作区现状）

范围：10 份定义与共用约束，外加同一批一起改、被定义指着的钩子、门禁、规则与 skill（当背景读）。

````diff
diff --git a/.claude/agent-common.md b/.claude/agent-common.md
index 9bc512a..257018e 100644
--- a/.claude/agent-common.md
+++ b/.claude/agent-common.md
@@ -31,7 +31,7 @@
 ## 写
 
 - 只写派发提示与定义「写范围」一节给的路径。写范围闸（项目 settings 里的 hook，表在 `.claude/hooks/agent-write-scope.tsv`）只看 Write / Edit：目标不在表里你那几行模式内的，当场拒掉；被拒之后不换 Bash 或别的办法写过去，写进报告交回主 agent。Bash 里的写闸看不见，全靠你守。
-- tools 里有 Write / Edit 的：手写的改动一律用 Edit（旧串必须在文件里恰好命中一次，与 `replace-once.py` 同义；这一轮自己新建的文件可以 `replace_all`），新建文件用 Write（先 `ls` 确认不存在），让闸看得见；tools 里只有 Edit、没有 Write 的，新建文件照下一条「tools 只有 Read / Bash 的」排他写；Bash 里只许定义点名的脚本自己写（`relabel-item.py`、`kb-scribe` 的 `replace-batch.py`、门禁阶段的 `--write`、`mutate.sh`、`replay.sh`、cargo、跑产物的重定向）。草稿目录里（仓副本、日志、自己的辅助脚本）用什么写都行，那里只归你；只有一条例外：改一个已存在的 `.sh` / `.py` 照「不做」一节检出 hook 的 ⑦ 换 inode（`replace-once.py`，或写临时文件再 `mv`）。
+- tools 里有 Write / Edit 的：手写的改动一律用 Edit（旧串必须在文件里恰好命中一次，与 `replace-once.py` 同义；这一轮自己新建的文件可以 `replace_all`），新建文件用 Write（先 `ls` 确认不存在），让闸看得见；tools 里只有 Edit、没有 Write 的，新建文件照下一条「tools 只有 Read / Bash 的」排他写；除了这种排他新建，Bash 里只许定义点名的脚本自己写（`relabel-item.py`、`kb-scribe` 的 `replace-batch.py`、门禁阶段的 `--write`、`mutate.sh`、`replay.sh`、cargo、跑产物的重定向）。草稿目录里（仓副本、日志、自己的辅助脚本）用什么写都行，那里只归你；只有一条例外：改一个已存在的 `.sh` / `.py` 照「不做」一节检出 hook 的 ⑦ 换 inode（`replace-once.py`，或写临时文件再 `mv`）。
 - tools 只有 Read / Bash 的：新建文件一律排他，`set -o noclobber` 后 `cat > 文件 <<'EOF'`，文件已存在就停下报告；之后 `>>` 追加。
 - 报告分段写，每一次写进文件的内容不超过 150 行（`.claude/rules/three-way-inference.md`「云端腿的报告要分段落盘」）；表格与代码块整块放进同一段，不从中间切。
 - tools 只有 Read / Bash 的，改已有文件只用定点替换：`research/scripts/replace-once.py` 或 `research/scripts/replace-batch.py`（先 `--dry-run`），不整份重写。
@@ -60,7 +60,7 @@
 
 - 哪个门禁阶段该由谁在干完自己的活之后先跑，登记在 `.claude/gate.d/stage-owners.tsv`（第二列是 agent 名，逗号分隔）。列出登记给你的：
   `awk -F'\t' -v me=<你的名字> '$1 !~ /^#/ && NF == 3 { n = split($2, owners, ","); for (i = 1; i <= n; i++) if (owners[i] == me) print $1 }' .claude/gate.d/stage-owners.tsv`
-  逐个 `nice -n 19 bash .claude/gate.d/<文件>`，贴每个的原样末行与退出码。
+  逐个 `nice -n 19 bash .claude/gate.d/<文件>`，贴每个的原样末行与退出码。其中重型的那几道（54、55、57、59、87）照「不做」一节重型测试那一条跑：只在提交时或用户要求时、命令带前缀；定义另有写法的照定义（`gate-triage` 登记的阶段都在整轮门禁里跑过，不单跑）。
 - 退出码 77 是「本次未跑」，不是通过。红了先看它点名的文件与行在不在这一轮的改动里；不在的不修，照写。
 - 提交前的整轮门禁归 `gate-triage`，不归你；表里没登记给你的阶段不用跑。
 
diff --git a/.claude/agents/crash-verifier.md b/.claude/agents/crash-verifier.md
index a1d08c3..1947421 100644
--- a/.claude/agents/crash-verifier.md
+++ b/.claude/agents/crash-verifier.md
@@ -21,7 +21,7 @@ omitClaudeMd: true
 
 ## 做什么
 
-1. 每个阶段开跑前各照共用约束「不做」一节看一次负载。54 号不归你：快档在 `gate.sh --staged` 的 HEAD + 暂存区树上跑才算得对全绿标记的键（在主工作区跑，工作区与暂存区不同就判红），全量由主 agent 在 worktree 里跑。
+1. 每个阶段开跑前各照共用约束「不做」一节看一次负载。54 号不归你：快档在 `gate.sh --staged` 的 HEAD + 暂存区树上跑才算得对全绿标记的键（在主工作区跑，工作区与暂存区不同就判红），全量由主 agent 在 worktree 里跑。55、57、59 在主工作区跑，判的是工作区那一份：每个阶段开跑前跑 `git diff --quiet -- crates Cargo.toml Cargo.lock litmus research/scripts research/results`（退 0 才说明工作区与暂存区在这几道的输入上相同），不为 0 就不跑那一道，停下报告「工作区与暂存区不同，判的不是要提交的那一批」交主 agent；同一批路径下的未跟踪文件（`git ls-files --others --exclude-standard -- <同一批路径>`）原样列进报告。
 2. 只在提交时（或主 agent 转达用户要求时）跑你那几道，不跑 `gate.sh` 整轮与全量 `cargo test`（`.claude/hooks/heavy-test-guard.sh` 拒）。按共用约束 `.claude/agent-common.md`「门禁」一节从阶段归属表取登记给你的阶段，一次只跑一个；55、57、59 号命令带输入给的那个前缀：`SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/<文件>`，其余登记给你的轻阶段不带；每个记开始与结束时刻（`date -u`）和退出码。单个阶段超过 Bash 单次上限就后台跑，退出码写进文件（`{ SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/<文件>; echo "exit=$?"; } > <草稿目录>/<阶段>.log 2>&1`；退出码只认日志里的 `exit=` 那一行，不认起后台的那条命令自己的 `$?`），结束后读输出。别的会话同时在跑 cargo 时，编译会卡在「Blocking waiting for file lock」，照实记等了多久，不算这个阶段的耗时。
 3. 每个阶段原样抄它的「✓」行，或「✗」行与紧跟的「→」；退出码 77 记「本次未跑」，不记通过。
 4. 计数行原样抄：崩溃状态数、恢复结果、与 E142（第一个事务的干跑） 产物或闭式比对的那几行。输出里读不到判定行的阶段记「作废」，不记通过。
diff --git a/.claude/agents/experiment-designer.md b/.claude/agents/experiment-designer.md
index 73a98f8..d7dd46b 100644
--- a/.claude/agents/experiment-designer.md
+++ b/.claude/agents/experiment-designer.md
@@ -19,13 +19,13 @@ omitClaudeMd: true
 - 要回答的问题（一种读法；有两种读法的先交回主 agent 定问法。两种读法都报、又不改变任何判据时，可以两种都登记，文件头挂一句「问法待主 agent 在装置写之前删一种」；主 agent 删完再派执行员，执行员见到这句不开工）。
 - 重跑已有实验的：实验号与第几次。这时不占号，写重跑登记 `research/prompts/e<号>-r<n>-prereg.md`，节名照「登记的固定节名」那一节。
 - 被测条款：kb 文件路径与小节标题。
-- 实验简称（或由你按问题起一个，交主 agent 认）；另在登记里定一个英文名（小写字母与数字，词之间下划线），源文件、变异表与 `[[bin]]` 名用它，中文简称只用在 kb 页。
+- 实验简称（或由你按问题起一个，交主 agent 认）。
 - 草稿目录。
 
 ## 做什么
 
 1. 先读被测条款与它的定义逐字（定义不是答案），判问法有没有两种读法；读 `crates/` 里对应的实现。要交回主 agent 定问法的，这时候交回，不占号。重跑的不跑 `claim-experiment.sh`：`set -o noclobber` 后建重跑登记，文件已存在就停下报告；原登记要读（判据怎么定的），实验页与产物照样不读结果与结论。
-2. `bash research/scripts/claim-experiment.sh --next` 取号，`bash research/scripts/claim-experiment.sh E<号> <简称>` 占住（它建 `research/prompts/e<号>-preregistration.md`）。不读 `.claude/kb/experiments/` 下已有实验的结果与结论小节，不读 `research/results/`；全仓搜条款与用词时加 `--exclude-dir=experiments --exclude-dir=results --exclude-dir=prompts --exclude=experiments.md --exclude=experiments-history.md`（实验索引的结论列与实验变更史是文件，`--exclude-dir` 排不掉）；`records/`、`.claude/kb/decisions-history/`、`research/perf-by-milestone.md` 里命中的结果行排不干净，读到了照下一句列进「跑之前已经存在的数」。`mutation-sampling.md` 第五类要查「这个量仓里有没有人算过、分母是不是同一个」：只读别的实验怎么算（口径、分母那几行），不读它的结果与结论。读过的每个文件列进登记，grep 命中行也算读过；还是读到了已有的数、或判问法时自己算出了答案，列进登记固定的一节「跑之前已经存在的数」，不删。
+2. `bash research/scripts/claim-experiment.sh --next` 取号，`bash research/scripts/claim-experiment.sh E<号> <简称>` 占住（它建 `research/prompts/e<号>-preregistration.md`）；占号之后定一个英文名（小写字母与数字，词之间下划线），写在登记 `## 一、问题` 的第一行「英文名：…」，源文件、变异表与 bin 名用它，中文简称只用在 kb 页；重跑登记照抄原登记的英文名。不读 `.claude/kb/experiments/` 下已有实验的结果与结论小节，不读 `research/results/`；全仓搜条款与用词时加 `--exclude-dir=experiments --exclude-dir=results --exclude-dir=prompts --exclude=experiments.md --exclude=experiments-history.md`（实验索引的结论列与实验变更史是文件，`--exclude-dir` 排不掉）；`records/`、`.claude/kb/decisions-history/`、`research/perf-by-milestone.md` 里命中的结果行排不干净，读到了照下一句列进「跑之前已经存在的数」。`mutation-sampling.md` 第五类要查「这个量仓里有没有人算过、分母是不是同一个」：只读别的实验怎么算（口径、分母那几行），不读它的结果与结论。读过的每个文件列进登记，grep 命中行也算读过；还是读到了已有的数、或判问法时自己算出了答案，列进登记固定的一节「跑之前已经存在的数」，不删。
 3. 登记写明：问题，连同岔路单逐行抄进来，第六节每个量写明它对应岔路单的哪一行、取什么值会让那一行翻面——对不上任何一行的量不登记，确要顺带量的标「附带，够判后不跑」；每条臂的定义（「怎么做」与「做完会怎样」要推得出，对面那条臂写成支持它的人认的样子）；阳性对照（每条臂都跑）与真实基线；每个量各占一行的判据与门槛（门槛不能从臂的定义直接推出）；钉绝对值的断言；被谓词消费的量报峰值、为正轮数、期末值；几何敏感性那一行（至少一个方向相反的取样点）；失败条款，每条紧跟「什么观测会让它触发」；作废条款；装置与 `crates/` 实现对不上时的停机条款（`implementation-first.md` 第 4 条：两边都查，既不作废也不当结果）。纯算术题里不适用的格（没有臂、没有轮次）写明为什么不适用，不硬凑。钉绝对值的锚点分两类写：出自被测条款本身的（不符时走「条款可能错」的失败条款）与独立算出、用命令核过的（不符才作废）。给执行员列的每条变异，写明它在哪个取样点上改变输出。**装置写在哪要在登记里写死一句**：默认走 `research/e7-index-bench/src/bin/`（独立手写模型，不与实现共用代码，`implementation-first.md` 第 4 条）；**只有当这条岔路问的就是「`crates/` 今天这份代码的性质」、另写一份独立模型答不了时**，才写死成「入库装置」（执行员改写 `crates/singlefs-harness/src/bin/`）。写死成入库装置的，登记里同时写明这条实验哪些量要读 `crates/` 的哪些常量——执行员按它逐个改成本地常量加回比断言，不许直接引。⚠️ 登记里没写死这一句的，执行员一律走 `research/` 那一支，不自己判。计时类的量写明在哪个进程里、用谁的时钟计：在被测进程里自己计时并写进结果行，或先把子进程输出整份读完再转打；不拿「读一行、打时间戳、转打、再读下一行」那个循环里行到达的时刻当分段挂钟（照 `.claude/kb/vm-harness.md`「计时不在转发输出的循环里打时间戳」一节办）。
 3b. 估一下实现量：历史族 × 几何格 × 臂 × 崩溃支线 × 坏镜像，加上要另写的子系统（检查器、触发式重算、崩溃枚举）。按岔路排序：能最先让岔路单里某一行够判的量放在最前，连同停机条款（与 `crates/` 的逐项对拍）与钉绝对值的锚点；一个执行员一次做不完的，在第五节末尾分段写「第一段 / 第二段 / …」，每段写明它补岔路单哪一行、补完那一行够不够判。分段不是「全部做完」的计划：每段交回之后主 agent 对着岔路单判续不续，一行开着的都不剩就停，后面的段不跑。
 
@@ -35,7 +35,7 @@ omitClaudeMd: true
 
 | 节名 | 写什么 |
 |---|---|
-| `## 一、问题` | 主 agent 给的问题逐字；读法写死（取值范围、分母、口径）；岔路单（或问题单）逐行抄进来 |
+| `## 一、问题` | 第一行「英文名：…」（第 2 步定）；主 agent 给的问题逐字；读法写死（取值范围、分母、口径）；岔路单（或问题单）逐行抄进来 |
 | `## 二、被测条款与它引的定义` | `research/scripts/quote-kb.py` 整段抄进草稿目录里的出口文件，回读一致后 `>>` 追加进登记；出口不指向登记本身，追加完回读登记，占号时写的文件头在不在看文件，不看退出码 |
 | `## 三、实现今天的样子` | `crates/` 里对应实现，文件加行号；没有就写 grep 命令与零命中 |
 | `## 四、跑之前已经存在的数` | 读条款、判问法时已经知道或算出的数，照实列，写它们对判据的影响 |
diff --git a/.claude/agents/experiment-runner.md b/.claude/agents/experiment-runner.md
index 989d8f3..99c8dd4 100644
--- a/.claude/agents/experiment-runner.md
+++ b/.claude/agents/experiment-runner.md
@@ -24,7 +24,7 @@ omitClaudeMd: true
 ## 做什么
 
 1. 开跑前照共用约束「不做」一节看负载。登记第六节里有耗时类的量时，跑产物之前连别的 `cargo`、`gate.sh` 也要等。
-2. 写 `research/e7-index-bench/src/bin/e<号>_<英文名>.rs`（源文件、变异表、`[[bin]]` 的 `name` 一律用跑前登记里定的英文名：源文件与变异表写蛇形 `e<号>_<英文名>`，`name` 写连字符 `e<号>-<英文名>`；中文简称只用在 kb 页），在 `research/e7-index-bench/Cargo.toml` 末尾追加它的 `[[bin]]`（⚠️ **跑前登记把装置写死成「入库装置」时改写 `crates/singlefs-harness/src/bin/e<号>_<英文名>.rs`**：岔路问的是「`crates/` 今天这份代码的性质」时，另写一份独立模型答不了，写范围闸按 `crates/singlefs-harness/src/bin/e*.rs` 放行。那种 bin 只读、只驱动与观测，不许改 `crates/singlefs-core` 与 `crates/singlefs-checker`；变异表改追加进 `crates/mutations.tsv` 末尾、归门禁 59 号跑。**另有三条只对入库装置生效**：① **不许从 `crates/` 里引决定工作量、几何或判据门槛的常量**——这类参数在装置里写本地常量，值抄自 kb 的登记位并注明文件与行号，另加一条断言拿它与实现侧同名常量回比；引了就等于装置与实现共用一个数，两边一起错而逐格相等。这一条挡不住「kb 与实现一起取错」。② **写了入库装置就要让它进代码轮判决的点名清单**：门禁 56 号的正则罩得到 `crates/*/src/bin/*.rs`，不点名那一道会红，而那条义务落在跑整轮门禁的一方身上、不落在你身上——交回时写明你新建了哪个 bin。③ **什么算「入库装置」以跑前登记里写死的那一句为准**；登记里没写死就走 `research/` 那一支，别自己判）（`name` 用连字符，与 `replay.sh` 登记行里的 bin 名一致；只追加，不改别人的）：纯算术的计数模型、单测、钉绝对值的断言；取样点覆盖登记里每一个会跑到的取值。写完跑 `bash .claude/scripts/naming-lint.sh`，新文件里的缩写与单字母名在这一步改完再往下走，不留给后面的门禁阶段。
+2. 写 `research/e7-index-bench/src/bin/e<号>_<英文名>.rs`（源文件、变异表与 bin 名一律用跑前登记 `## 一、问题` 第一行定的英文名：源文件与变异表写蛇形 `e<号>_<英文名>`；`research/e7-index-bench` 的 `[[bin]]` 的 `name` 写连字符 `e<号>-<英文名>`，入库装置不写 `[[bin]]`、bin 名就是文件名 `e<号>_<英文名>`；中文简称只用在 kb 页），在 `research/e7-index-bench/Cargo.toml` 末尾追加它的 `[[bin]]`（⚠️ **跑前登记把装置写死成「入库装置」时改写 `crates/singlefs-harness/src/bin/e<号>_<英文名>.rs`**：岔路问的是「`crates/` 今天这份代码的性质」时，另写一份独立模型答不了，写范围闸按 `crates/singlefs-harness/src/bin/e*.rs` 放行。那种 bin 只读、只驱动与观测，不许改 `crates/singlefs-core` 与 `crates/singlefs-checker`；变异表改追加进 `crates/mutations.tsv` 末尾、归门禁 59 号跑。**另有三条只对入库装置生效**：① **不许从 `crates/` 里引决定工作量、几何或判据门槛的常量**——这类参数在装置里写本地常量，值抄自 kb 的登记位并注明文件与行号，另加一条断言拿它与实现侧同名常量回比；引了就等于装置与实现共用一个数，两边一起错而逐格相等。这一条挡不住「kb 与实现一起取错」。② **写了入库装置就要让它进代码轮判决的点名清单**：门禁 56 号的正则罩得到 `crates/*/src/bin/*.rs`，不点名那一道会红，而那条义务落在跑整轮门禁的一方身上、不落在你身上——交回时写明你新建了哪个 bin。③ **什么算「入库装置」以跑前登记里写死的那一句为准**；登记里没写死就走 `research/` 那一支，别自己判）（`research/e7-index-bench` 的 `name` 用连字符，与 `replay.sh` 登记行里的 bin 名一致；只追加，不改别人的）：纯算术的计数模型、单测、钉绝对值的断言；取样点覆盖登记里每一个会跑到的取值。写完跑 `bash .claude/scripts/naming-lint.sh`，新文件里的缩写与单字母名在这一步改完再往下走，不留给后面的门禁阶段。
 3. 入库装置那一支不写 research 的变异表、不跑 `mutate.sh`（它只编得到 `research/` 下的 bin，整表跑 `crates/mutations.tsv` 又是重型）：变异行照 `crates/mutations.tsv` 第 1 行表头的六段格式追加，报告里列出追加的变异名、三个数写「待提交时 59 号」。`research/` 那一支写变异表 `research/mutations/e<号>_<英文名>.tsv`，在 `research/` 下跑完整张表：`cd research && nice -n 19 bash scripts/mutate.sh <bin 名> e7-index-bench/src/bin/<源文件> mutations/<变异表>`（两个路径都相对 `research/`；bin 名以 `research/e7-index-bench/Cargo.toml` 里 `[[bin]]` 的 `name` 为准，没有显式声明的取文件名去掉 `.rs`；报「基线就是红的」时先查是不是在仓根下跑、bin 名是不是写错）（收尾要有「已还原，基线仍全绿」）；没红的逐条按 `mutation-sampling.md` 分类。
 3b. 登记里的停机条款（第十一节里标「停机」的，通常是装置与 `crates/` 实现逐项对拍）在跑产物之前做，照原文真跑，不许只推；做不了就停下交回，不跑产物、不写实验页。登记的量一次做不完的，做完的那一段照常交，没做的逐条列进报告，实验页与索引行的状态写「部分已跑」，不写「已跑」。岔路单每一行都够判了（登记第十一节的够判停机），剩下的量不跑：实验页逐条标「够判后未跑」、写明各自对应哪一行，状态写「已跑（够判）」；够不够判由你按登记写的够判条件报，续不续由主 agent 定。
 4. 跑产物进 `research/results/`；跑前删旧输出，跑完认本轮才有的完成标记。重跑已有实验时，`research/results/` 里已有的产物不覆盖：与它逐字节一致就不新存；对不上就按今天的日期另存一份，实验页两份都点名、写明对不上，交主 agent 定哪份承重。生成产物用的二进制编在 `research/target/` 下，用最后一次源码改动编出来；生成产物与第 5 步跑 `replay.sh` 时都不设 `CARGO_TARGET_DIR`；草稿目录里的 target 只拿来迭代。
@@ -32,7 +32,7 @@ omitClaudeMd: true
 5. 在 `research/scripts/replay.sh` 里登记复跑，跑一次 `bash research/scripts/run-with-memory-cap.sh 4G bash research/scripts/replay.sh E<号>`（`replay.sh` 里执行编出来的二进制，不经内存包装会被 `heavy-test-guard.sh` 拒） 确认逐字节一致。送进虚机跑的实验（`vm-bench.sh`，含 `--selftest`）与 E152（按里程碑对比六家文件系统的文件性能） 装置是重型测试，你不跑：装置、登记与复跑命令写好就停，在交回里写明要跑什么、为什么，由主 agent 问用户（`.claude/kb/vm-harness.md`；共用约束「不做」一节里重型测试那一条）。装置要起子进程、读它的输出时，先把输出读完再转打，计时按登记写的进程与时钟取（`.claude/kb/vm-harness.md`「计时不在转发输出的循环里打时间戳」）；共享 `gate.sh` 的「转发计时」阶段查读子进程输出的循环里一边取时间一边输出。
 6. 跑阶段归属表登记给你的阶段（共用约束 `.claude/agent-common.md`「门禁」一节）；其中要编译或复跑整张表的，开跑前照共用约束看负载。15 号（编整个 research 工作区）与 87 号（复跑全部已入库实验）不归你，收尾由 `gate-triage` 统一跑；你只跑第 5 步自己那一个实验的 `replay.sh`。
 7. 主 agent 要写实验页时：`.claude/kb/experiments/<号>-<简称>.md`，结果整行抄自产物并带文件名，写复跑命令、口径、它答不了的；索引行进 `.claude/kb/experiments.md`。
-7b. 实验页的 `### 影响的决策` 一节照第 7 步写实验页的规则写，形态与判据不在这份定义里复述。这一步你要做的三件：① 要写支撑或推翻时，先 `grep -n 'E<号>（' .claude/kb/decisions/<那条决策的文件>` 看那条分项的「**依据**」段引没引这个实验——引了才写支撑或推翻，没引就写备料，并把「这一格该不该升成支撑」列进报告交主 agent，**不自己去改决策文件**（写范围闸也不放行）。② 那条决策还在 `.claude/decision-links-pending` 里时（那份清单只减不增，一条都没有时跳过这一步）不查依据段（门禁 75 号对清单里的决策整段跳过双向检查，那些决策还没有依据段），写决策级一行，并把它列进报告。③ 重跑已有实验、换了产物、加了历史条目之后，表里每一行都重新回看一遍。
+7b. 实验页的 `### 影响的决策` 一节照第 7 步写实验页的规则写，形态与判据不在这份定义里复述。这一步你要做的三件：① 要写支撑或推翻时，先 `grep -n 'E<号>（' .claude/kb/decisions/<那条决策的文件>` 看那条分项的「**依据**」段引没引这个实验——引了才写支撑或推翻，没引就写备料，并把「这一格该不该升成支撑」列进报告交主 agent，**不自己去改决策文件**（写范围闸也不放行）。② 那条决策还在 `.claude/decision-links-pending` 里时（那份清单只减不增；清单里一条都没有时 ② 不适用，① ③ 照做）不查依据段（门禁 75 号对清单里的决策整段跳过双向检查，那些决策还没有依据段），写决策级一行，并把它列进报告。③ 重跑已有实验、换了产物、加了历史条目之后，表里每一行都重新回看一遍。
 
 ## 写范围
 
diff --git a/.claude/agents/gate-triage.md b/.claude/agents/gate-triage.md
index 6cc62bc..8dd1261 100644
--- a/.claude/agents/gate-triage.md
+++ b/.claude/agents/gate-triage.md
@@ -23,13 +23,13 @@ omitClaudeMd: true
 ## 做什么
 
 1. 开跑前照共用约束「不做」一节看负载；另有别的 `gate.sh` 在跑就等它结束。
-2. 只在提交时（或主 agent 转达用户要求时）跑：暂存过的跑 `SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/scripts/gate.sh --staged`（用户要求时 `=user-request`）；主 agent 明写全量的跑不带参数（前缀照带）。54、55、57、59 这几道重阶段在 `gate.sh` 里各走各的：54 号跑快档并核全绿标记，55、59 照 `.claude/gate.d/stage-inputs.tsv` 复用上一次全绿判定，57 号没有复用、每次现跑；你不直接调它们，也不跑全量 `cargo test`（`.claude/hooks/heavy-test-guard.sh` 拒）。登记给你的其余阶段已在 `gate.sh` 整轮里跑过，不再单跑；主 agent 点名要单跑某一道时才单跑，轻阶段不带前缀，87 号照带。超过 Bash 单次上限时后台跑、结束后读输出。
+2. 只在提交时（或主 agent 转达用户要求时）跑：暂存过的跑 `SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash research/scripts/gate-staged.sh`（用户要求时 `=user-request`；它跑 `gate.sh --staged`，整轮全绿才前移 `refs/sop/staged-green`）；主 agent 明写全量的跑 `bash .claude/scripts/gate.sh` 不带参数（前缀照带）。54、55、57、59 这几道重阶段在整轮里各走各的：54、55、59（还有 74）号只在 `gate-staged.sh` 那一趟里能复用上一次整轮全绿的判定（判据在 `research/scripts/stage-must-run.sh` 文件头），要跑时 54 号跑快档并核全绿标记，57 号没有复用、每次现跑；你不直接调它们，也不跑全量 `cargo test`（`.claude/hooks/heavy-test-guard.sh` 拒）。登记给你的其余阶段已在 `gate.sh` 整轮里跑过，不再单跑；主 agent 点名要单跑某一道时才单跑，轻阶段不带前缀，87 号照带。超过 Bash 单次上限时后台跑、结束后读输出。
 3. 每个红阶段：抄门禁给的「✗」行与紧跟的「→」下一步原样；看它点名的文件与行在不在暂存区 diff 的块里（`git diff --cached -U0 -- <文件>`）。在块里 ⇒「这一轮」；文件在但行不在块里、或文件不在暂存区 ⇒「不是这一轮」；红句说的是「这一批输入」本身的（54 号的「没有这批输入的全绿标记」「那一格的哈希不同」这类）不按它点名的标记路径判：这一轮的改动（`git diff --cached --name-only`；跑全量工作区时是主 agent 给的文件清单）碰了 `.claude/gate.d/stage-inputs.tsv` 里那一道那一行登记的路径 ⇒「这一轮」，下一步原样抄 54 号出路里的三行命令，一条都没碰 ⇒「不是这一轮」；工具没装、网关不通、`cargo` 不在 ⇒「环境」，用 `bash .claude/scripts/env.sh` 的对应行佐证；别的会话同时在跑同一个重阶段、进程被外部信号杀掉（输出里一行裸的 `Terminated`）⇒「并发」，贴开跑时与出事前后 `ps` 看到的同名进程。红阶段没有「✗」与「→」可抄的（被杀、崩溃），抄它最后 20 行输出，判「分不清」或「并发」，写明没有下一步可抄。分不清的写「分不清」与原因。
 4. 原样抄未实现清单与「由项目本地阶段覆盖」清单。别的会话同时在改仓、跑全量工作区时「工作区跑的过程中没变」那一条红了：照抄开跑、收尾两个指纹，判「不是这一轮」。`gate.sh` 不打印单个阶段的耗时，取不到的不估。
 
 ## 写范围
 
-- 报告文件、草稿目录。门禁自己的 git 写（共享 `gate.sh` 不带 `--staged` 全绿时 `update-ref refs/sop/gate-ok`（带 `--staged` 时不写）、`--staged` 时的临时 worktree）是门禁本身的行为，允许；你自己不做任何 git 写。
+- 报告文件、草稿目录。门禁自己的 git 写（共享 `gate.sh` 不带 `--staged` 全绿时 `update-ref refs/sop/gate-ok`（带 `--staged` 时不写）、`--staged` 时的临时 worktree、`gate-staged.sh` 整轮全绿时 `update-ref refs/sop/staged-green`）是门禁本身的行为，允许；你自己不做任何 git 写。
 
 ## 产出
 
diff --git a/.claude/agents/implementation-writer.md b/.claude/agents/implementation-writer.md
index c26e20d..d1f87d1 100644
--- a/.claude/agents/implementation-writer.md
+++ b/.claude/agents/implementation-writer.md
@@ -25,7 +25,7 @@ omitClaudeMd: true
 1. 开跑前照共用约束「不做」一节看负载。
 2. 读条款与代码，写实现与测试。名字、分支、类型、错误照 `code-discipline.md`：不缩写、不写 `_ =>`、newtype、`expect` 写清依赖哪条不变量。
 3. 每条新测试证明会红：在草稿目录的仓副本里（`rsync -a --exclude target --exclude .git`）改坏被测代码一处，跑那条测试所在的整个测试二进制看它红（经内存包装，共用约束「不做」一节），记「改坏哪一行 → 哪条断言红」与同时红了哪些测试；每份副本用它自己的 target（不把 `CARGO_TARGET_DIR` 指到几份副本共用的目录）；改坏的副本要还原时，从原件拷回之后对被改的文件 `touch`，或删掉这份副本重拷，不用 `rsync -a` 拷回了事；先跑一份不改动的副本：已经红的测试记成基线红集，变异证明只看基线红集之外的测试，你要证明的那条在基线红集里就停下交回；被测代码里有 `debug_assert` 的，debug 下先红的可能是它，照实记红在哪一条，要测试断言自己红就在 `--release` 下再跑一次。`crates/mutations.tsv` 在的话（门禁 59 号），每条再加一行变异，原文在文件里恰好命中一次。
-4. 变异证红按测试算、不按行算：每条新测试挑一行能让它红的变异证一次就够，其余行照样追加进 `crates/mutations.tsv` 末尾，整表复跑归最后的门禁 59 号；报告写明哪几行证过、哪几行留给 59 号。层 0 不跑：自己新加的层 0 流不在交回前跑，随提交时的层 0 全量验；已有流一条都不跑；新测试落在名字含 layer0 的测试二进制里的，第 3 步的证红也不跑，变异行照样追加、报告写明留给提交时的 59 号。层 0 全量每次提交跑一次（`main-agent.md`「什么时候派哪个 agent」里「暂存之后、提交之前跑门禁」那一步），不由实现员跑。交回前的验证只到这几样：动到的测试二进制（整个二进制，不按名字挑；名字含 layer0 的不跑）、第 3 步的证红、`cargo fmt --all -- --check`、`cargo clippy --all-targets --all-features -- -D warnings` 加 `.claude/singlefs-ai-sop/scripts/check.sh` 里 `CODE_DISCIPLINE_LINTS` 那几条 `-D`（照那份脚本现抄，不跑 `check.sh` 本身，它是重型）、`cargo build --offline --all-targets`，各贴末尾原样输出。门禁阶段只跑阶段归属表登记给你的那几道（共用约束「门禁」一节）；全量 `cargo test --all`、层 0 各流的快档与全量、其余门禁阶段都不跑，留给提交时统一的那一次验证（`crash-verifier` 与整轮门禁）；派发提示另点名要跑的，不是重型测试的照点名跑，重型的照共用约束「不做」一节那一条交回主 agent。
+4. 变异证红按测试算、不按行算：每条新测试挑一行能让它红的变异证一次就够，其余行照样追加进 `crates/mutations.tsv` 末尾，整表复跑归最后的门禁 59 号；报告写明哪几行证过、哪几行留给 59 号。层 0 不跑：自己新加的层 0 流不在交回前跑，随提交时的层 0 全量验；已有流一条都不跑；新测试落在名字含 layer0 的测试二进制里的，第 3 步的证红也不跑，变异行照样追加、报告写明留给提交时的 59 号。层 0 全量每次提交跑一次（`main-agent.md`「什么时候派哪个 agent」里「暂存之后、提交之前跑门禁」那一步），不由实现员跑。交回前的验证只到这几样：动到的测试二进制（整个二进制，不按名字挑；名字含 layer0 的不跑）、第 3 步的证红、`cargo fmt --all -- --check`、`cargo clippy --all-targets --all-features -- -D warnings` 加 `.claude/singlefs-ai-sop/scripts/check.sh` 里 `CODE_DISCIPLINE_LINTS` 那几条 `-D`（照那份脚本现抄，不跑 `check.sh` 本身，它是重型）、`cargo build --offline --all-targets`，各贴末尾原样输出。门禁阶段只跑阶段归属表登记给你的那几道（共用约束「门禁」一节）；全量 `cargo test --all`、层 0 各流的快档与全量、其余门禁阶段都不跑，留给提交时统一的那一次验证（主 agent 的层 0 全量、`crash-verifier` 与整轮门禁）；派发提示另点名要跑的，不是重型测试的照点名跑，重型的照共用约束「不做」一节那一条交回主 agent。
 5. 要改的文件在输入给的「别的会话正在改」清单里：不碰那个文件，也不做只能落在它上面的条款，报告写明卡在哪个文件、哪条条款；新文件要靠清单里的文件才进编译（模块声明、`Cargo.toml` 的 `[[test]]`）的，那条整条停下，不写那个新文件；其余照做。`crates/mutations.tsv` 例外：只在末尾追加整行，不改别人的行。
 6. 改动里碰到条款没写、要做设计判断的地方，停在那一处，写进报告交主 agent，不自己定。「停在那一处」的写法看这条分支走不走得到：
    - 走得到（包括要坏盘、崩溃、瞬时读错才走得到）：在**任何落盘动作之前**返回一个错误成员，名字说清是哪条条款没定、第一版不支持；写一条用例钉住「返回这个成员、盘上逐字节不变」（`DiskSnapshot` 之类比系统配置槽、根环、录制流步数），不钉它之后的行为。判定与后面的写要读同一份输入：写之前重算、对不上就不写。
@@ -43,4 +43,4 @@ omitClaudeMd: true
 
 ## 没做什么（固定会有的）
 
-- 没走三方对抗；层 0、QEMU、herd7 与 crates 变异表归 `crash-verifier`；没提交。
+- 没走三方对抗；层 0 全量归主 agent、快档在整轮门禁里，QEMU、herd7 与 crates 变异表归 `crash-verifier`；没提交。
diff --git a/.claude/agents/kb-scribe.md b/.claude/agents/kb-scribe.md
index 6953bce..4b8e03f 100644
--- a/.claude/agents/kb-scribe.md
+++ b/.claude/agents/kb-scribe.md
@@ -17,7 +17,7 @@ omitClaudeMd: true
 
 - 逐条改动规格：文件、旧串（原文整行）、新串、依据（判决或用户定案的出处）。
 - 要不要记决策变更史：标题整行、日期、改前、改后、依据各一句（快查两行由主 agent 给定，不由你概括；标题里「（其N）」那一段由你在写之前照当月文件现取，其余逐字照给）。
-- 分项翻状态的：决策号与分项号；另给这几样，缺了门禁必红而你不能自己补：分项那一行收尾的「**状态：已定。**」或「**状态：未定。**」（20 号），翻成未定的判两句（31 号两把尺都要，缺一句就红）：改不改第一个事务的字节、动不动格式，决策文件标题行的状态（已定 / 半定（N 项未定））与 `.claude/kb/decisions.md` 状态列的分项计数改成什么。变更史标题与快查里的分项标签写翻完之后的：变更史用今天的编号（`.claude/kb/decisions-history.md` 第 8 行），写成翻之前的，第 3 步的 `relabel-item.py` 也会把它改掉。
+- 分项翻状态的：决策号与分项号；另给这几样，缺了门禁必红而你不能自己补：分项那一行收尾的「**状态：已定。**」或「**状态：未定。**」（20 号），翻成未定的判两句（31 号两把尺都要，缺一句就红）：改不改第一个事务的字节、动不动格式，决策文件标题行的状态改成什么（已定 / 半定（N 项未定）/ 待定（N 项未定），N 写阿拉伯数字或汉字数字都认）；`.claude/kb/decisions.md` 状态列由第 3 步的 `21-decision-items-sync.sh --write` 重生成，规格只给预期的分项计数供回读核对，不手写。变更史标题与快查里的分项标签写翻完之后的：变更史用今天的编号（`.claude/kb/decisions-history.md` 第 8 行），写成翻之前的，第 3 步的 `relabel-item.py` 也会把它改掉。
 - 欠账表要加或还的行：也写成逐条规格（加：新行与插在哪一整行之后；还：被还的那一行，与移进已还清表的新行、插在哪一整行之后）。
 - 写决策正文（新立分项、或改一条分项的定案 / 射程 / 依据）的：规格要按 `.claude/rules/format-evolution.md` 那一节的形态逐块给全（四块、索引行、未定项的字节判决），形态本身照那一节，这份定义不抄第二份；缺哪一块就停下报告，不自己补。
 - **规格改了某条分项的「**依据**」段时，同一份规格还要带上那个实验页 `### 影响的决策` 表里对应那几行的改动**（旧串、新串，一条分项一行）：门禁 75 号那条双向检查两侧要同时到位，而判关系是主 agent 的活、不是你的。规格里没带那几行就停下报告，列出缺哪几行。
diff --git a/.claude/agents/mutation-triage.md b/.claude/agents/mutation-triage.md
index 7facba7..1a642c3 100644
--- a/.claude/agents/mutation-triage.md
+++ b/.claude/agents/mutation-triage.md
@@ -25,7 +25,7 @@ omitClaudeMd: true
 2. 先逐条做子串计数：原文在源文件里不是恰好一次的，列出来（第七类），这张表不跑。
 3. 在 `research/` 下跑：`cd research && nice -n 19 bash scripts/mutate.sh <bin 名> e7-index-bench/src/bin/<源文件> mutations/<变异表>`（路径相对 `research/`；报「基线就是红的」时先查是不是在仓根下跑）；表头写着「被测装置是 shell 探针」的，照表头写的复跑方式逐条跑，判据用表头那句，替换在草稿目录里那份探针的副本上做、跑副本，不动仓里的探针（要 dm / loop 设备才跑得起来的，写明没跑、为什么，交主 agent）；crates 那张表你不跑：整表复跑是重型测试，只在提交时由崩溃验证员跑门禁 59 号（`.claude/rules/implementation-workflow.md`「重型测试只在提交时跑」，`heavy-test-guard.sh` 会拒绝你跑它）；主 agent 给你那一次 59 号的输出路径，你从里面取点名的条目分类。
 4. 确认整张表跑完：`mutate.sh` 收尾要有「已还原，基线仍全绿」；59 号没有这句，看它收尾的「计数：」行里「共 N 条」与各栏加起来对得上，编不过的在「有变异无效」一栏，「没跑到」算进没红；对不上就是中途退出，这一次的数不算，照实报。
-5. 报抓到 / 无效 / 没红三个数（`mutate.sh` 没有三数汇总行，按每条结果行开头的符号数（`[ ]` 里是变异名，不是结局）：`✅` 抓到、`⏭` 无效、`❌` 没红；`💥`（测试进程没跑完）、`⚠️`（报了失败却一个测试名都没抓到）、`⏱`（超时）、`🧱`（内存撞顶）另列，几栏加起来要等于表的条数；59 号照「计数：」行），超时、内存撞顶等其余几栏另列、不并进三个数，不为 0 的整轮已判失败；与改动前的数比，「无效」变多要单列。
+5. 报抓到 / 无效 / 没红三个数（`mutate.sh` 没有三数汇总行，按每条结果行开头的符号数（`[ ]` 里是变异名，不是结局）：`✅` 抓到、`⏭` 无效、`❌` 没红；`💥`（测试进程没跑完）、`⚠️`（报了失败却一个测试名都没抓到）、`⏱`（超时）、`🧱`（内存撞顶）另列，几栏加起来要等于表的条数；`mutate.sh` 收尾那行「计数：」只报内存撞顶与超时两栏；59 号照它自己收尾的「计数：」行，那一行各栏都有），其余几栏不并进三个数：`⚠️`、`⏱`、`🧱` 不为 0 的整轮已判失败，`💥` 不判失败，逐条列出交主 agent；与改动前的数比，「无效」变多要单列。
 6. 没红与无效的逐条按 `mutation-sampling.md` 分类；判「取样点不敏感」的，解出让两个式子跨过整数边界的那个输入；判「等价」的，写出在所有输入上同值的理由。
 
 ## 写范围
diff --git a/.claude/agents/three-way-verifier.md b/.claude/agents/three-way-verifier.md
index 3c7ad1f..583a175 100644
--- a/.claude/agents/three-way-verifier.md
+++ b/.claude/agents/three-way-verifier.md
@@ -18,17 +18,18 @@ omitClaudeMd: true
 - 轮名、这一轮全部腿报告的路径（主 agent 确认都已交齐、腿不再写）。
 - 背景材料路径（用来识别误写成背景材料行号的引用）。
 - 云端腿交回里给的报告 `sha256sum`（有就给）。
-- 腿开工那一刻的快照路径：`sha256sum` 清单，罩这一轮被判的文件与材料点名的 kb 文件（`.claude/rules/implementation-workflow.md`「代码轮派腿之前记一份开工快照」；代码轮必给），腿跑着的时候被别的会话改过的，主 agent 另给倒推出的原样副本（`*.at-snapshot`）。没给快照的代码轮，停下要，不对主树核。没给快照的轮（设计轮）对主树核，腿引的行号与主树对不上时记「分不清：文件可能在腿交回之后被改过」（与第 6 步那一类一样单列），不记 ✗。
+- 腿开工那一刻的快照路径：`sha256sum` 清单，罩这一轮被判的文件与材料点名的 kb 文件（`.claude/rules/implementation-workflow.md`「代码轮派腿之前记一份开工快照」；代码轮必给），腿跑着的时候被别的会话改过的，主 agent 另给倒推出的原样副本（`*.at-snapshot`）。没给快照的代码轮，停下要，不对主树核。没给快照的轮（设计轮）对主树核，腿引的行号与主树对不上时记「分不清：文件可能在腿开工之后被改过」（第 6 步的「分不清」一栏），不记 ✗。
+- 腿开工时刻（UTC）：第 2 步现查「快照清单里没有的文件」在腿开工之后有没有被改过要用；没给就不查那一支，照实写没查。
 - 报告路径（形态 `research/prompts/<轮>-verifier-output.md`）、草稿目录。
 
 ## 做什么
 
 1. 判别力自证，先做：从待核的引用里挑一条，在草稿目录的副本里把它的行号加 1，按第 2 步核它，必须判 ✗；判不出就停下报告「核查方法不分辨」。
-2. 每处「文件:行号 + 抄的原文」：先拿快照清单 `sha256sum -c` 核那个文件；对得上就到主树那一行（区间就取区间）比内容，对不上就只对主 agent 给的倒推副本核，两样都没有的记「分不清：文件在腿开工之后被改过」，不记 ✗；文件不在快照清单里的，对主树核，并用 `git log --since=<腿开工时刻> -- <文件>` 与 `git status --short -- <文件>` 现查它在腿开工之后有没有被改过，照实写，不一律记成「被改过」。对不上时再找原文实际在第几行；若那个行号在背景材料里正好是这句，写「误写成背景材料第 N 行，原文件实为第 M 行」，不只打 ✗。
+2. 每处「文件:行号 + 抄的原文」：先拿快照清单 `sha256sum -c` 核那个文件；对得上就到主树那一行（区间就取区间）比内容，对不上就只对主 agent 给的倒推副本核，两样都没有的记「分不清：文件可能在腿开工之后被改过」，不记 ✗；给了快照、而这个文件不在清单里的，对主树核，并用 `git log --since=<腿开工时刻> -- <文件>` 与 `git status --short -- <文件>` 现查它在腿开工之后有没有被改过，照实写，不一律记成「被改过」。对不上时再找原文实际在第几行；若那个行号在背景材料里正好是这句，写「误写成背景材料第 N 行，原文件实为第 M 行」，不只打 ✗。
 3. 每行引的产物：在产物文件里逐字找。
 4. 每条复跑命令：把腿的模型目录拷到草稿目录，在副本里跑（加 `nice -n 19`），比输出与报告里抄的、比 sha256；不在腿的原目录里跑。复跑命令带着指向路径的环境变量的，路径一并换到你的草稿目录；输出里嵌着临时路径的，按字段比，不按整份哈希判 ✗。
 5. 本地腿的转述核对表里每一处「原文文件:行」也逐条核：行号在不在、抄的是不是原文、英文有没有丢限定词，也有没有多加原文没有的限定词或括注。
-6. 核不动的（要编译、要虚机、要网络）写「核不动」与原因；「核不动」与「分不清：文件在腿交回之后被改过」单列，不算进 ✓ 也不算进 ✗；报告文件现在的 sha256 与交回里给的对不上，整份记后一种。
+6. 核不动的（要编译、要虚机、要网络）写「核不动」与原因；「核不动」与「分不清」（第 2 步与输入一节那几种，连同原因）各单列一栏，不算进 ✓ 也不算进 ✗；报告文件现在的 sha256 与交回里给的对不上，整份记「分不清：报告在腿交回之后被改过」。
 
 ## 写范围
 
@@ -38,7 +39,7 @@ omitClaudeMd: true
 ## 产出
 
 - 开头：判别力自证那一条的原样结果。
-- 每份腿报告一张表：引用 / 核的结果（✓、✗ 加实际位置、核不动）/ 命令；末尾计数（核了几处、✓ 几处、✗ 几处），再加「没做什么」。
+- 每份腿报告一张表：引用 / 核的结果（✓、✗ 加实际位置、核不动）/ 命令；末尾计数（核了几处、✓ 几处、✗ 几处、核不动几处、分不清几处），再加「没做什么」。
 
 ## 没做什么（固定会有的）
 
diff --git a/.claude/gate.d/20-kb-shape.sh b/.claude/gate.d/20-kb-shape.sh
index 42145e4..51ed117 100755
--- a/.claude/gate.d/20-kb-shape.sh
+++ b/.claude/gate.d/20-kb-shape.sh
@@ -43,7 +43,7 @@ echo
 echo "── 3. 文件内自指链接 ──"
 # 扫 kb 下每一份 .md：链接目标（去掉 #锚点）按这份文件自己的目录解析之后就是它自己，判红。
 # 历史类文件（`*-history.md`、`decisions-history/` 下的月份文件）整份跳过：它们逐字记着当时的原文，
-# 里面抄录的链接改了就成假话（`.claude/rules/path-moves.md`「历史类文件保留旧名」同一条判据）。
+# 里面抄录的链接改了就成假话（判据同 `.claude/rules/path-moves.md`「改一个全仓术语：正文之外还有五处会红」里「历史类文件同样换名」那一段：换了就成假话的那一句留原样）。
 self_link_report=$(python3 - "$KB" <<'PY'
 import os, re, sys, glob
 kb = sys.argv[1]
diff --git a/.claude/gate.d/54-layer0-replay.sh b/.claude/gate.d/54-layer0-replay.sh
index 1eed270..931b80a 100755
--- a/.claude/gate.d/54-layer0-replay.sh
+++ b/.claude/gate.d/54-layer0-replay.sh
@@ -227,7 +227,8 @@ report_manifest_differences() {
 # print_staged_worktree_full_commands：出路里建「HEAD + 暂存区」worktree、在里面用那棵树里的 54 号跑 --full 的命令。
 # 建法与共享 gate.sh --staged 相同：worktree add --detach HEAD，再 apply --index 暂存区的 diff（diff 为空就不套）。
 print_staged_worktree_full_commands() {
-  echo '                在项目根、暂存之后：layer0_full_base="$(mktemp -d)"; git diff --cached --binary > "$layer0_full_base/staged.patch"'
+  echo '                在项目根、暂存之后，把下面三行命令放进同一次 Bash 调用（三行共用 layer0_full_base 这个变量，分开跑它就是空的）：'
+  echo '                layer0_full_base="$(mktemp -d)"; git diff --cached --binary > "$layer0_full_base/staged.patch"'
   echo '                git worktree add --detach "$layer0_full_base/tree" HEAD && { [ ! -s "$layer0_full_base/staged.patch" ] || git -C "$layer0_full_base/tree" apply --index "$layer0_full_base/staged.patch"; }'
   echo '                SINGLEFS_HEAVY_TESTS=commit bash research/scripts/run-with-memory-cap.sh 16G bash "$layer0_full_base/tree/.claude/gate.d/54-layer0-replay.sh" --full "$layer0_full_base/tree"; git worktree remove --force "$layer0_full_base/tree"'
 }
diff --git a/.claude/gate.d/75-decision-experiment-links.sh b/.claude/gate.d/75-decision-experiment-links.sh
index 44c603c..d3451e4 100755
--- a/.claude/gate.d/75-decision-experiment-links.sh
+++ b/.claude/gate.d/75-decision-experiment-links.sh
@@ -223,7 +223,7 @@ for decision, info in sorted(decisions.items()):
         continue
     where = info['path']
     # 半定 / 待定的决策，20 号要标题里写明未定几项（`—— 半定（一项未定）`），只许这一种括注
-    if not re.match(r'^## D\d+ \S.*? —— (已定|半定|待定)(（[0-9一二三四五六七八九十]+[项条]未定）)?\s*$', info['title']):
+    if not re.match(r'^## D\d+ \S.*? —— (已定|半定|待定)(（[0-9一二两三四五六七八九十]+[项条]未定）)?\s*$', info['title']):
         bad('瘦身形态', f'{where}：首行写成「## D{decision} 简称 —— 状态」，状态后面除了半定 / 待定要写的「（N 项未定）」不带别的括注——「{info["title"][:40]}」')
     for (kind, item_number), heading, labels, cited, basis_text in info['shapes']:
         if DATE.search(heading):
diff --git a/.claude/gate.d/90-term-renames.sh b/.claude/gate.d/90-term-renames.sh
index 21eb099..6513160 100755
--- a/.claude/gate.d/90-term-renames.sh
+++ b/.claude/gate.d/90-term-renames.sh
@@ -19,6 +19,6 @@ echo "               产物分两类，别一把梭：**跑得出来的**（装
 echo "               换完跑 cargo test --workspace 与 bash research/scripts/replay.sh，同一份源码要仍然吐出逐字节相同的产物——"
 echo "               那是重新生成，不是改证据；**跑不出来的**（已归档、要虚机或真设备、别人的产物）一个字节都不许动，"
 echo "               连同引它的正文整段留旧名（evidence-discipline「原样保存的证据不许事后改」）。"
-echo "               确实该留旧名的（别家术语、冻结证据目录、历史类文件、引文块、对照表自己），"
+echo "               确实该留旧名的（别家术语、冻结证据目录、历史类文件里换了就成假话的那一句、引文块、对照表自己），逐文件"
 echo "               登记进 .claude/term-rename-exempt 并写明为什么。"
 exit 1
diff --git a/.claude/hooks/heavy-test-guard.sh b/.claude/hooks/heavy-test-guard.sh
index acc83d8..dc6bd51 100755
--- a/.claude/hooks/heavy-test-guard.sh
+++ b/.claude/hooks/heavy-test-guard.sh
@@ -36,8 +36,8 @@
 #   .claude/gate.d/ 下 54、55、57、59、87 之外的阶段不是重型，谁都能跑、不用带前缀。
 # 谁、带什么才放行（都要带环境变量 SINGLEFS_HEAVY_TESTS=commit 或 =user-request，别的值或没带一律拒）：
 #   主 agent（输入里没有 agent_type）：上面每一类；
-#   crash-verifier：层 0、55 号与 qemu-system-*、herd7、crates 变异整表（vm-bench.sh、全量测试、整轮门禁、E152 拒）；
-#   gate-triage：整轮门禁（gate.sh、gate-staged.sh）与 87 号（gate.sh 里 54 / 55 / 57 / 59 靠「输入没变就复用上一次全绿判定」，直接调它们拒）；
+#   crash-verifier：55 号与 qemu-system-*、herd7、crates 变异整表（层 0 归门禁分诊员的整轮门禁与主 agent，vm-bench.sh、全量测试、整轮门禁、E152 拒）；
+#   gate-triage：整轮门禁（gate-staged.sh、gate.sh）与 87 号（54 / 55 / 57 / 59 在整轮门禁里跑，直接调它们拒；复用上一次整轮全绿判定只在 gate-staged.sh 那一趟里有，判据在 research/scripts/stage-must-run.sh 文件头）；
 #   其余子 agent：一律拒，带不带前缀都拒。
 # 前缀认三种写法：写在命令前（`SINGLEFS_HEAVY_TESTS=commit bash …`）、写进 `env` 的参数、同一行前面的 `export`；
 # 往 `bash -c '…'`、`capped.sh N …`、`nice`、`timeout` 这类包装里面传。git 的 pre-commit hook 由 git 起，不经这道闸。
@@ -126,7 +126,7 @@ SHELL_SCRIPT_EXTENSION = ".sh"  # 直接执行、没有 `#!` 的文件，只有
 
 # 子 agent 自己那一份（kind 见 lib_heavy_tests.KIND_CATEGORY）；不在表里的子 agent 一样也不许
 AGENT_KINDS = {
-    "crash-verifier": {"layer0-stage", "layer0-cargo", "layer0-binary", "qemu-stage", "qemu-system", "herd7-stage", "lkmm", "herd7",
+    "crash-verifier": {"qemu-stage", "qemu-system", "herd7-stage", "lkmm", "herd7",
                        "crates-mutation-stage", "crates-mutation-mutate"},
     "gate-triage": {"gate-sh", "gate-staged", "replay-all-stage"},
 }
@@ -517,7 +517,7 @@ POLICY = ("→ 规矩：重型测试（层 0、QEMU、herd7、crates 变异整
           "子 agent 一律不跑，只跑自己动到的测试二进制（`cargo test -p <crate> --test <自己的目标>`、`--lib`）与 fmt / clippy / build；"
           "主 agent 在提交流程里跑要带 `SINGLEFS_HEAVY_TESTS=commit`，用户要求时带 `SINGLEFS_HEAVY_TESTS=user-request`。\n"
           "→ 各自那一份：crash-verifier 只跑 55、57、59 号与 qemu-system、lkmm.sh / herd7、crates 变异整表（54 号快档在 gate.sh --staged 里，全量由主 agent 跑）；"
-          "gate-triage 只跑 `gate.sh` 整轮与 87 号（54、55、57、59 靠「输入没变就复用上一次全绿判定」）；两个都要带那个前缀，都不跑全量 `cargo test`。"
+          "gate-triage 只跑整轮门禁（`research/scripts/gate-staged.sh`，它跑 `gate.sh --staged`）与 87 号（54、55、57、59 在整轮里跑，复用上一次整轮全绿判定只在 gate-staged.sh 那一趟里有）；两个都要带那个前缀，都不跑全量 `cargo test`。"
           "`.claude/gate.d/` 下其余阶段不是重型，谁都能跑。\n"
           "→ 提交之外任务确实要跑的：主 agent 先弹窗问用户，用户同意了才带 `SINGLEFS_HEAVY_TESTS=user-request` 跑；"
           "子 agent 在交回里写明要跑什么、为什么，交主 agent 去问（派发提示里点名要你跑的也一样，写明被这道闸拒了）。")
@@ -716,10 +716,10 @@ def selftest(hook_dir):
             ("主 agent export 之后跑 54 号全量", None, "export SINGLEFS_HEAVY_TESTS=commit && bash /tmp/wt/.claude/gate.d/54-layer0-replay.sh --full /tmp/wt", 0),
             ("主 agent 前缀往包装里传", None, "nice -n 19 env SINGLEFS_HEAVY_TESTS=commit bash research/scripts/capped.sh 8 bash -c 'cargo test --workspace'", 0),
             ("主 agent 跑一个测试目标不是重型", None, "cargo test -p singlefs-core --test core_contract", 0),
-            ("崩溃验证员带前缀跑 54 号全量", crash, commit + "bash .claude/gate.d/54-layer0-replay.sh --full /tmp/wt", 0),
-            ("崩溃验证员带前缀跑层 0 测试目标", crash,
-             commit + "nice -n 19 bash research/scripts/run-with-memory-cap.sh 16G cargo test --release -p singlefs-harness --test first_transaction_step_seven_layer0", 0),
-            ("崩溃验证员带前缀直接执行层 0 测试二进制", crash, commit + "bash research/scripts/run-with-memory-cap.sh 16G " + layer0_binary, 0),
+            ("崩溃验证员带前缀跑 54 号全量：层 0 不归它", crash, commit + "bash .claude/gate.d/54-layer0-replay.sh --full /tmp/wt", 2),
+            ("崩溃验证员带前缀跑层 0 测试目标：层 0 不归它", crash,
+             commit + "nice -n 19 bash research/scripts/run-with-memory-cap.sh 16G cargo test --release -p singlefs-harness --test first_transaction_step_seven_layer0", 2),
+            ("崩溃验证员带前缀直接执行层 0 测试二进制：层 0 不归它", crash, commit + "bash research/scripts/run-with-memory-cap.sh 16G " + layer0_binary, 2),
             ("崩溃验证员带 =user-request 跑 55 号", crash, request + "bash .claude/gate.d/55-qemu-first-transaction.sh", 0),
             ("崩溃验证员带前缀跑 59 号", crash, "GATE_MUTATION_TARGET_DIR=/tmp/t " + commit + "nice -n 19 bash .claude/gate.d/59-crates-mutation-replay.sh", 0),
             ("门禁分诊带前缀跑 gate.sh --staged", triage, commit + "nice -n 19 bash .claude/scripts/gate.sh --staged", 0),
@@ -765,8 +765,8 @@ def selftest(hook_dir):
              "cat > gen-unwrapped.sh <<'EOF'\ncargo run --release --bin e160-random-small-read-share\nEOF\nbash gen-unwrapped.sh", 2, 0, 1),
             ("主 agent 不经内存包装跑测试目标：这一道不判主 agent", None, "cargo test -p singlefs-core --lib", 0),
             ("崩溃验证员带前缀不经内存包装跑层 0 测试目标", crash, commit + "cargo test --release -p singlefs-harness --test first_transaction_step_seven_layer0", 2),
-            ("崩溃验证员带前缀经内存包装跑 54 号全量", crash,
-             commit + "bash research/scripts/run-with-memory-cap.sh 16G bash .claude/gate.d/54-layer0-replay.sh --full /tmp/wt", 0),
+            ("崩溃验证员带前缀经内存包装跑 54 号全量：层 0 不归它", crash,
+             commit + "bash research/scripts/run-with-memory-cap.sh 16G bash .claude/gate.d/54-layer0-replay.sh --full /tmp/wt", 2),
             ("崩溃验证员带前缀经内存包装跑 55 号", crash, commit + "bash research/scripts/run-with-memory-cap.sh 16G bash .claude/gate.d/55-qemu-first-transaction.sh", 0),
             ("崩溃验证员带前缀经内存包装跑 57 号", crash, commit + "bash research/scripts/run-with-memory-cap.sh 8G bash .claude/gate.d/57-lkmm.sh", 0),
             ("门禁分诊带前缀经内存包装跑 gate.sh --staged", triage, commit + "bash research/scripts/run-with-memory-cap.sh 24G bash .claude/scripts/gate.sh --staged", 0),
diff --git a/.claude/hooks/runner-dispatch-guard.sh b/.claude/hooks/runner-dispatch-guard.sh
index 601f9bb..45effcb 100755
--- a/.claude/hooks/runner-dispatch-guard.sh
+++ b/.claude/hooks/runner-dispatch-guard.sh
@@ -35,7 +35,7 @@
 #   重型阶段的名字：层 0（层 0 / 0 层 / layer0 / 54 号）、QEMU（qemu / vm-bench / 55 号）、herd7（herd7 / lkmm / 57 号）、crates 变异整表
 #   （crates/mutations.tsv / 变异整表 / 59 号）、全量测试（check.sh / 全量测试）、整轮门禁（gate.sh / gate-staged.sh / 整轮门禁）、全部实验复跑（87 号）、E152 装置。
 #   `.claude/gate.d/` 下 54、55、57、59、87 之外的阶段不是重型，提示里要它跑不拦。
-#   crash-verifier 放行层 0、QEMU、herd7、crates 变异整表那几句，gate-triage 放行整轮门禁与全部实验复跑那几句（它们各自那一份，执行时还要带
+#   crash-verifier 放行 QEMU、herd7、crates 变异整表那几句（层 0 不归它：快档在整轮门禁里，全量由主 agent 跑），gate-triage 放行整轮门禁与全部实验复跑那几句（它们各自那一份，执行时还要带
 #   SINGLEFS_HEAVY_TESTS=commit / user-request，由 heavy-test-guard.sh 判）；其余一律拒。
 #   会误拒：名字在前、动词在后、中间没有 TOPIC_GAP_BREAK 的说明句（「54 号在提交时跑全量」「54 号跑全量要四十分钟」），改写成「只在……才跑」或用「」括起来；
 #   否定词在括号外、重型阶段的名字在括号里的（「不跑重型测试（check.sh、`cargo test --workspace`）」）：按 ③ 括号里单独成分句，否定词不在那个分句里，照拒；
@@ -136,7 +136,7 @@ HEAVY_TEST_NAMES = [
 ]
 FULL_TEST_COMMAND = re.compile(r"cargo\s+test\b[^\n]*?--(?:all|workspace)(?![\w-])")
 DISPATCH_HEAVY_OWN_SHARE = {
-    "crash-verifier": {"层 0", "QEMU", "herd7", "crates 变异整表"},
+    "crash-verifier": {"QEMU", "herd7", "crates 变异整表"},
     "gate-triage": {"整轮门禁", "全部实验复跑"},
 }
 
@@ -436,7 +436,7 @@ def decide(hook_input, project_root):
                    "→ 判法：句子里「跑 / 执行 / 复跑 / 重跑 / 运行 / bash / 起」的宾语是重型阶段，或写了「`cargo test --all`」这种命令字面；"
                    "否定句、引号里的、「……说，」「报告里写」「原句」之后的转述、重型阶段只是同句另一个名词的，都不拦（判法细节在这个 hook 的文件头）。\n"
                    "→ 规矩：重型测试（层 0、QEMU、herd7、crates 变异整表、全量测试、整轮门禁、全部实验复跑、E152 装置）只在提交代码时、或用户要求时跑；"
-                   "子 agent 只跑自己动到的测试二进制、fmt / clippy / build 与 54、55、57、59、87 之外的门禁阶段；crash-verifier 只跑 54、55、57、59 号那几道，"
+                   "子 agent 只跑自己动到的测试二进制、fmt / clippy / build 与 54、55、57、59、87 之外的门禁阶段；crash-verifier 只跑 55、57、59 号那几道，"
                    "gate-triage 只跑 gate.sh 整轮与 87 号（执行时 .claude/hooks/heavy-test-guard.sh 也拒）。\n"
                    "→ 怎么办：真要它跑就删掉这一句；不是要它跑的，写成否定句（「不跑层 0」），否定词与名字放在同一个分句里（名字别放进否定词后面的全角括号，括号里单独成分句），转述别人的原话用「」括起来；提交时的重阶段派 crash-verifier、整轮门禁派 gate-triage；"
                    "提交之外任务确实要跑，先弹窗问用户，用户同意了由主 agent 带 SINGLEFS_HEAVY_TESTS=user-request 跑。")
@@ -569,7 +569,8 @@ def selftest(hook_dir):
             case("重型:只提到、没要它跑", "implementation-writer", "层 0 归 crash-verifier；check.sh 那一套 lint 下的 clippy 要过。", 0),
             case("重型:--all-targets 不是 --all", "implementation-writer", "跑 `cargo build --offline --all-targets`。", 0),
             case("重型:轻阶段谁都能跑", "experiment-runner", first + "这一段回答的岔路：岔路 1\n跑完再跑 bash .claude/gate.d/12-no-prime-marks.sh。\n", 0),
-            case("重型:崩溃验证员跑自己那几道", "crash-verifier", "提交流程里跑 54 号 --full、55 号 QEMU、57 号 herd7、59 号变异整表，命令带 SINGLEFS_HEAVY_TESTS=commit。", 0),
+            case("重型:崩溃验证员跑自己那几道", "crash-verifier", "提交流程里跑 55 号 QEMU、57 号 herd7、59 号变异整表，命令带 SINGLEFS_HEAVY_TESTS=commit。", 0),
+            case("重型:崩溃验证员跑层 0 不归它", "crash-verifier", "提交流程里跑 54 号 --full，命令带 SINGLEFS_HEAVY_TESTS=commit。", 2),
             case("重型:门禁分诊跑整轮", "gate-triage", "带 SINGLEFS_HEAVY_TESTS=commit 跑 gate.sh --staged。", 0),
             # 收窄：只在动词的宾语就是重型阶段时才拦。2026-09-25 被误拦的原句逐字，都该放行
             case("收窄:原句一「只在提交时才执行的」是定语", "general-purpose",
diff --git a/.claude/main-agent.md b/.claude/main-agent.md
index cdb1486..4976215 100644
--- a/.claude/main-agent.md
+++ b/.claude/main-agent.md
@@ -53,10 +53,10 @@
 |---|---|
 | 要推论：设计判断、取舍，拿依据（包括实验结论）去推翻或确立决策，实现改动的对抗 | 主 agent 写 `research/prompts/_<轮>-body.md` → `three-way-materials` → 同一条消息并行派 `three-way-forward` 或 `three-way-defense`（一轮一条）、`three-way-attack`、`three-way-local-attack` 或 `three-way-local-defense`（一轮一条）→ 全部交齐后，照 `.claude/rules/three-way-inference.md`「核查员按轮派」派 `three-way-verifier`（有腿交了模型、产物或复跑命令就派；不派的在判决里写明为什么）→ 主 agent 写判决；本地腿派攻方还是辩方由主 agent 定，在正文分工表里写明理由；云端腿报「没打中」而要拿它支撑结论时，主 agent 用同一份提示再派一条同立场腿；三轮之后停 |
 | 改 `crates/`：条款已定、验收标准写得清、改动有界（照已定分项实现、补测试与变异、修原因已知的红） | `implementation-writer`（后台派，主 agent 审 diff）→ 上一行的三方（代码轮）→ 提交时照「暂存之后、提交之前跑门禁」那一行：层 0 全量主 agent 跑，派 `crash-verifier` 跑 QEMU、herd7、crates 变异表（命令带 `SINGLEFS_HEAVY_TESTS=commit`，见「暂存之后、提交之前跑门禁」那一行；平时不跑） |
-| 改 `crates/`：探索性的、条款没写全、要用户边看边拍板 | 主 agent 自己写、直接和用户对话 → 上一行的三方（代码轮）→ 提交时 `crash-verifier` |
+| 改 `crates/`：探索性的、条款没写全、要用户边看边拍板 | 主 agent 自己写、直接和用户对话 → 上一行的三方（代码轮）→ 提交时照「暂存之后、提交之前跑门禁」那一行：层 0 全量主 agent 跑，`crash-verifier` 跑 QEMU、herd7、crates 变异表 |
 | 跑变异表、看抓到 / 无效 / 没红 | `mutation-triage` → 要补取样点、补断言的：`crates/` 那侧 `implementation-writer`（输入给分诊报告），`research/` 那侧 `experiment-designer` 写重跑登记 → `experiment-runner` |
 | 一个阶段任务结束：里程碑一步、一轮判决、一段实验、一批定义或脚本改完，这一批交回到齐、暂存之前 | `sweep` 写事实表（阶段同步第一段；输入给改动范围与做成的事，只用过、没留改动的也写；交回的事实表过了 `research/scripts/stale-candidates.py --check-facts`）→ 候选表按组切段，每段不超过约 200 行，一段派一个 `sweep` 逐行判（第二段；输入同样给做成的事）→ 交回齐了主 agent 跑 `stale-candidates.py --check-report 候选表 各份报告`，退出码 0 才往下，再**逐行全看**：每一条判定都对着载体今天的原文核一遍，不抽样、不按判定种类挑。判错的那一段重派，重派交回照样全看 → 主 agent 逐处判，自己改或交 `kb-scribe` → 主 agent 写 `research/prompts/<阶段>-sync.md`（格式见门禁 68 号文件头，68 号判形式）→ 下一行 |
-| 暂存之后、提交之前跑门禁 | 主 agent 用 `research/scripts/stage-mine.py` 暂存 → 这一批改了门禁 54 号在 `.claude/gate.d/stage-inputs.tsv` 登记的输入、54 号脚本本身或工具链（`cargo -V`、`rustc -V`；这三样都进全绿标记的键），主 agent 自己在后台跑 `.claude/gate.d/54-layer0-replay.sh` 里 `print_staged_worktree_full_commands` 打印的那三行（建 HEAD + 暂存区的 worktree、带 `SINGLEFS_HEAVY_TESTS=commit` 经内存包装跑那棵树里的 `--full`、删 worktree；命令只在那一处写，不在这里抄）（层 0 全量只在这一步跑，判绿按输入哈希写全绿标记），看门狗用 `--processes` 盯 → 派 `crash-verifier` 跑 55、57、59，命令带 `SINGLEFS_HEAVY_TESTS=commit` → `gate-triage` 带 `SINGLEFS_HEAVY_TESTS=commit` 跑 `gate.sh --staged` 并分诊（`gate.sh` 里 54 号跑快档并核全绿标记，55、59 照 `stage-inputs.tsv` 复用上一次全绿判定，57 号没有复用、每次现跑）；用户要求时两处都换成 `=user-request` |
+| 暂存之后、提交之前跑门禁 | 主 agent 用 `research/scripts/stage-mine.py` 暂存 → 这一批改了门禁 54 号在 `.claude/gate.d/stage-inputs.tsv` 登记的输入、54 号脚本本身或工具链（`cargo -V`、`rustc -V`；这三样都进全绿标记的键），主 agent 自己在后台跑 `.claude/gate.d/54-layer0-replay.sh` 里 `print_staged_worktree_full_commands` 打印的那三行命令，三行放进同一次 Bash 调用（它们共用一个 shell 变量；建 HEAD + 暂存区的 worktree、带 `SINGLEFS_HEAVY_TESTS=commit` 经内存包装跑那棵树里的 `--full`、删 worktree；命令只在那一处写，不在这里抄）（层 0 全量只在这一步跑，判绿按输入哈希写全绿标记），看门狗用 `--processes` 盯；全量判绿、全绿标记写好之后才往下，不然整轮门禁里的 54 号必红 → 派 `crash-verifier` 跑 55、57、59，命令带 `SINGLEFS_HEAVY_TESTS=commit` → `gate-triage` 带 `SINGLEFS_HEAVY_TESTS=commit` 跑 `research/scripts/gate-staged.sh`（它跑 `gate.sh --staged`，整轮全绿才前移 `refs/sop/staged-green`）并分诊：54、55、59、74 号只在这一趟里能复用上一次整轮全绿的判定，判据在 `research/scripts/stage-must-run.sh` 文件头；要跑时 54 号跑快档并核全绿标记，57 号没有复用、每次现跑；用户要求时各处都换成 `=user-request` |
 | 改了一个数或格式常量、撤回一条结论、新立一条判据 | `sweep`（四种活与各自要给的输入见它的定义） |
 | 定案之后写回 kb | `kb-scribe`（主 agent 给逐条规格）；规格动了某条分项的「**依据**」段时，同一份规格要带上那个实验页 `### 影响的决策` 表里对应那几行（门禁 75 号那条双向检查两侧要同时到位，而判一个实验撑不撑一条分项是主 agent 的活）；它翻了分项状态、33 号红（变异表锚点腐化）→ 红在 `research/mutations/` 的表：`experiment-runner` 只修锚点；红在 `crates/mutations.tsv`（`relabel-item.py` 改写了 `crates/` 下的源码）：派 `implementation-writer` 修锚点，走代码轮 |
 | 要建计数实验，或重跑已有实验 | 主 agent 写岔路单（`.claude/rules/three-way-inference.md`「交岔路时写岔路单」；不是为岔路建的实验写同格式的问题单）→ `experiment-designer` 写跑前登记或重跑登记 → 主 agent 删掉登记里待删的问法（有的话）→ `experiment-runner` → 每段交回主 agent 对着岔路单判够不够：一行开着的都不剩就停、交岔路表；续派时派发提示点名还差的那几行（续派闸 `.claude/hooks/runner-dispatch-guard.sh` 查这一句） |
diff --git a/.claude/rules/implementation-workflow.md b/.claude/rules/implementation-workflow.md
index 432577e..8243661 100644
--- a/.claude/rules/implementation-workflow.md
+++ b/.claude/rules/implementation-workflow.md
@@ -36,7 +36,7 @@
 | 要拿出什么 | 怎么算数 |
 |---|---|
 | 那一次跑的日志，且那一道在里面判绿 | 引它的原样判定行，不转述 |
-| 那一次跑的索引与现在的索引，在**那一道读的每个路径**上逐字相同 | 登记进 `.claude/gate.d/stage-inputs.tsv` 的阶段由 `research/scripts/stage-must-run.sh` 自动比对（`refs/sop/staged-green` 那棵树与这一次的暂存树），不必再手工逐个路径现查；没登记的阶段仍要手工核，输出不截断，说不全那一道读什么就没有复用的资格 |
+| 那一次跑的索引与现在的索引，在**那一道读的每个路径**上逐字相同 | 登记进 `.claude/gate.d/stage-inputs.tsv` 的阶段由 `research/scripts/stage-must-run.sh` 自动比对（`refs/sop/staged-green` 那棵树与这一次的暂存树；只在 `research/scripts/gate-staged.sh` 起的那一趟里比，直接跑 `gate.sh` 一律照跑），不必再手工逐个路径现查；没登记的阶段仍要手工核，输出不截断，说不全那一道读什么就没有复用的资格 |
 | 那一次之后的改动一条都碰不到那些路径 | 把改动清单与输入清单并排列出来 |
 
 复用要在收尾报告里写明：复用了哪几道、引的是哪一次跑、比对了哪些路径。不写的按没跑算（`.claude/singlefs-ai-sop/rules/show-me-test.md`「门禁不许假装通过」）。
@@ -49,10 +49,10 @@
 
 | 场合 | 跑不跑 |
 |---|---|
-| 每次提交代码 | **必须跑**：主 agent 在提交流程里后台起（命令带 `SINGLEFS_HEAVY_TESTS=commit`），看门狗盯；整轮门禁由 `gate-triage` 带前缀跑 `gate.sh --staged`（`.claude/main-agent.md`「暂存之后、提交之前跑门禁」那一行） |
+| 每次提交代码 | **必须跑**：主 agent 在提交流程里后台起（命令带 `SINGLEFS_HEAVY_TESTS=commit`），看门狗盯；整轮门禁由 `gate-triage` 带前缀跑 `research/scripts/gate-staged.sh`（`.claude/main-agent.md`「暂存之后、提交之前跑门禁」那一行） |
 | 用户当场要求，或任务确实要跑 | 任务确实要跑时主 agent 先弹窗问用户，用户同意了才跑；命令带 `SINGLEFS_HEAVY_TESTS=user-request` |
 | 其余任何时候 | 不跑 |
-| 崩溃验证员、门禁分诊员 | 各跑各的那一部分，只在提交时或用户要求时（命令带 `SINGLEFS_HEAVY_TESTS=commit` 或 `=user-request`）：层 0 全量由主 agent 在 HEAD + 暂存区的 worktree 里跑；崩溃验证员跑 55、57、59 号；门禁分诊员跑 `gate.sh --staged` 并分诊，其中 54 号跑快档并核全绿标记，55、59 靠「输入没变就复用上一次全绿判定」不重跑，57 号没有复用、照跑 |
+| 崩溃验证员、门禁分诊员 | 各跑各的那一部分，只在提交时或用户要求时（命令带 `SINGLEFS_HEAVY_TESTS=commit` 或 `=user-request`）：层 0 全量由主 agent 在 HEAD + 暂存区的 worktree 里跑；崩溃验证员跑 55、57、59 号；门禁分诊员跑 `research/scripts/gate-staged.sh`（它跑 `gate.sh --staged`）并分诊：54、55、59、74 号只在这一趟里能复用上一次整轮全绿的判定（判据在 `research/scripts/stage-must-run.sh` 文件头），要跑时 54 号跑快档并核全绿标记，57 号没有复用、照跑 |
 | 其余子 agent | **一律不跑**，只跑自己动到的测试二进制与 fmt / clippy / build |
 
 由 `.claude/hooks/heavy-test-guard.sh`（Bash 的 PreToolUse）在执行前拒绝：其余子 agent 跑上面任何一样拒绝；崩溃验证员、门禁分诊员跑自己那一部分之外的、或不带那个环境变量前缀的拒绝；主 agent 不带那个前缀也拒绝。派发提示要子 agent 跑重型测试的，由 `.claude/hooks/runner-dispatch-guard.sh` 拒绝。
diff --git a/.claude/rules/mutation-sampling.md b/.claude/rules/mutation-sampling.md
index bf2eb56..307797f 100644
--- a/.claude/rules/mutation-sampling.md
+++ b/.claude/rules/mutation-sampling.md
@@ -79,7 +79,7 @@
 
 变异表里一条本来好好的条目，会因为**别处改了一个常量**而从「被抓」变成「无效」，而没有任何东西报警。
 
-⇒ **改完格式常量，重跑受影响的每张变异表，比对「抓到 / 无效 / 没红」三个数**，不能只看「没红」是不是 0（`mutate.sh` 没有三数汇总行，按每条结果行开头的符号数（`[ ]` 里是变异名，不是结局）：`✅` 抓到、`⏭` 无效、`❌` 没红；`💥`（测试进程没跑完）、`⚠️`（报了失败却一个测试名都没抓到）、`⏱`（超时）、`🧱`（内存撞顶）另列，几栏加起来要等于表的条数；门禁 59 号照收尾的「计数：」行；超时、内存撞顶等其余几栏另列，不为 0 的整轮已判失败；`crates/mutations.tsv` 整表复跑是重型，由提交时的崩溃验证员跑 59 号）：
+⇒ **改完格式常量，重跑受影响的每张变异表，比对「抓到 / 无效 / 没红」三个数**，不能只看「没红」是不是 0（`mutate.sh` 没有三数汇总行，按每条结果行开头的符号数（`[ ]` 里是变异名，不是结局）：`✅` 抓到、`⏭` 无效、`❌` 没红；`💥`（测试进程没跑完）、`⚠️`（报了失败却一个测试名都没抓到）、`⏱`（超时）、`🧱`（内存撞顶）另列，几栏加起来要等于表的条数；`⚠️`、`⏱`、`🧱` 不为 0 的整轮已判失败，`💥` 不判失败，照样单列、逐条交出去；`mutate.sh` 收尾那行「计数：」只报内存撞顶与超时两栏；门禁 59 号照它自己收尾的「计数：」行，那一行各栏都有；`crates/mutations.tsv` 整表复跑是重型，由提交时的崩溃验证员跑 59 号）：
 无效那一栏变多，等于有断言被关掉了，而它与「这一条本来就抓不到」在输出里长得一模一样。
 
 ## 判据
diff --git a/.claude/skills/crash-test/SKILL.md b/.claude/skills/crash-test/SKILL.md
index c286594..2901d28 100644
--- a/.claude/skills/crash-test/SKILL.md
+++ b/.claude/skills/crash-test/SKILL.md
@@ -9,4 +9,4 @@ description: 跑 singlefs 的验证套件——LKMM 内存序、QEMU/KVM 压测
 
 ## 在本项目里
 
-共享正文不管内存序与虚机（它写明那两样由项目自己定）。本项目的四样各落在一道门禁：崩溃点重放是 `.claude/gate.d/54-layer0-replay.sh`，QEMU 真设备是 `55-qemu-first-transaction.sh`，LKMM 内存序是 `57-lkmm.sh`，模型对拍是 `74-model-differential.sh`。前三样是重型测试，谁在什么时候跑、带什么前缀，照 `.claude/rules/implementation-workflow.md`「重型测试只在提交时跑」与 `.claude/main-agent.md`「暂存之后、提交之前跑门禁」那一行；子 agent 不跑，钩子 `.claude/hooks/heavy-test-guard.sh` 会拒。
+共享正文不管内存序与虚机（它写明那两样由项目自己定）。本项目的四样各落在一道门禁：崩溃点重放是 `.claude/gate.d/54-layer0-replay.sh`，QEMU 真设备是 `55-qemu-first-transaction.sh`，LKMM 内存序是 `57-lkmm.sh`，模型对拍是 `74-model-differential.sh`。前三样是重型测试，谁在什么时候跑、带什么前缀，照 `.claude/rules/implementation-workflow.md`「重型测试只在提交时跑」与 `.claude/main-agent.md`「暂存之后、提交之前跑门禁」那一行；子 agent 里只有崩溃验证员与门禁分诊员各跑登记给自己的那几道，别的子 agent 不跑，钩子 `.claude/hooks/heavy-test-guard.sh` 会拒。
diff --git a/.claude/skills/gate/SKILL.md b/.claude/skills/gate/SKILL.md
index 67dc2d8..9dd2f98 100644
--- a/.claude/skills/gate/SKILL.md
+++ b/.claude/skills/gate/SKILL.md
@@ -9,4 +9,4 @@ description: 跑 singlefs 的准入门禁。提交代码前、判断一个改动
 
 ## 在本项目里
 
-`gate.sh` 与 `check.sh` 在本项目里是重型测试，不是快速反馈：提交时由 `gate-triage` 带 `SINGLEFS_HEAVY_TESTS=commit` 跑 `gate.sh --staged`（`.claude/main-agent.md`「暂存之后、提交之前跑门禁」那一行），其余时候不跑，主 agent 不带前缀跑会被 `.claude/hooks/heavy-test-guard.sh` 拒。平时要快速反馈，单跑 `.claude/gate.d/` 下 54、55、57、59、87 之外的阶段。共享正文的阶段表不全，现有阶段以 `.claude/singlefs-ai-sop/scripts/gate.sh` 里 `run_stage` 那几行与 `.claude/gate.d/` 目录为准。
+`gate.sh` 与 `check.sh` 在本项目里是重型测试，不是快速反馈：提交时由 `gate-triage` 带 `SINGLEFS_HEAVY_TESTS=commit` 跑 `research/scripts/gate-staged.sh`（它跑 `gate.sh --staged`；`.claude/main-agent.md`「暂存之后、提交之前跑门禁」那一行），其余时候不跑，主 agent 不带前缀跑会被 `.claude/hooks/heavy-test-guard.sh` 拒。平时要快速反馈，单跑 `.claude/gate.d/` 下 54、55、57、59、87 之外的阶段。共享正文的阶段表不全，现有阶段以 `.claude/singlefs-ai-sop/scripts/gate.sh` 里 `run_stage` 那几行与 `.claude/gate.d/` 目录为准。
````
