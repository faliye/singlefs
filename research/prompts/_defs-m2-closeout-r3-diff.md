# 附录二：defs-m2-closeout-r3 的两份 diff（原样放；生成于 2026-09-26，HEAD 73ba4a4）

## 一、`/tmp/claude-1000/defs-closeout-r2-fixes/my-changes-final.diff`（整份原样；比的是开工时 `cp -p` 的备份，不是 git 基准）

```diff
--- a/.claude/agents/three-way-attack.md
+++ b/.claude/agents/three-way-attack.md
@@ -30,12 +30,12 @@
    - 盘用内存稀疏盘（`crates/singlefs-harness/src/crash.rs` 的 `SparseBlockDevice`），不在磁盘上建镜像文件；每个起点状态只建一次池（mkfs 与起点历史只跑一次），之后每段历史从内存里那一份拷（`SparseDevice`、`MemoryPool` 都能 `clone`）。
    - 正式跑之前先跑一小段，按它估全量的挂钟；估出来超过 40 分钟，先缩历史、候选与几何的取样，报告里写明缩了什么、缩前缩后各多少、估时怎么算的。
    - 崩溃状态不在缩的范围里（共用约束「不做」一节「崩溃点测试不衡量时间成本」那一条），照层 0 的枚举域取：调同一份文件里每一段都展开的 `enumerate_layer0` / `enumerate_layer0_versions`（录制写流，屏障与 FUA 切段，段内写的整写子集逐个枚举，不是只截前缀），每个崩溃状态恢复之后都跑 checker。缩只缩历史条数与长度、候选与几何的取样；留下的每段历史，它的崩溃状态一个不落。
-   - 在自己的原型里这样调枚举函数跑小流不算重型：测试目标的名字不带 `layer0`（带了 `heavy-test-guard.sh` 按名字拒）；每次跑之前用同一份文件的 `closed_form_state_count` 算出这一次全量的状态数，不超过约 10⁶ 个（几段历史合起来超过的，分几次跑）；一段历史自己就超过的，那段不跑，写进报告交主 agent。
+   - 在自己的原型里这样调枚举函数跑小流不算重型：原型里的流只许在原型里自己造，不从名字带 `layer0` 的用例里拷（拷文件、拷代码段、`include!`、`mod` 引进来都算拷）；测试目标的名字不带 `layer0`（带了 `heavy-test-guard.sh` 按名字拒）；每次跑之前用同一份文件的 `closed_form_state_count` 算出这一次全量的状态数，不超过约 10⁶ 个（几段历史合起来超过的，分几次跑）；这一轮全部原型跑的全量合起来不超过约 10⁷ 个状态。一段历史自己就超过约 10⁶ 的，那段不跑；再跑就要越过约 10⁷ 的，剩下的不跑；两样都写进报告交主 agent。
    - 随机跑批分小批：每批的段数按先跑一小段估出的时长定，不给批设限时（包装的 `RUN_WITH_MEMORY_CAP_TIME_LIMIT`、外面套 `timeout` 都不用）；每批的段数与估时写进报告。
 3c. 内存与进程：
    - 编译与跑都经内存包装：跑的照共用约束「不做」一节「跑编译出来的代码经内存包装」那一条，`cargo build` 也经它（`bash research/scripts/run-with-memory-cap.sh <上限> cargo build …`）。
    - 只停自己起的进程，一次一个 `python3 .claude/singlefs-ai-sop/scripts/proc.py stop <pid>`（共用约束「执行前拒绝的写法」那一条的 ⑥）。
-   - 并行起的几件，每件把退出码写进自己的文件（`{ <命令>; echo "$?" > <草稿目录>/<名字>.rc; } &`），最后单独一个不带参数的 `wait` 等齐，再按起的次序逐个读 `.rc` 文件（共用约束「执行前拒绝的写法」那一条的 ④）；不用 `wait "$pid"` 收退出码。
+   - 并行起的几件，每件把退出码写进自己的文件，文件名带批号与件号（`{ <命令>; echo "$?" > <草稿目录>/b<批号>-<件号>.rc; } &`），每批开跑前先删掉这一批的 `.rc`（`rm -f <草稿目录>/b<批号>-*.rc`）；最后单独一个不带参数的 `wait` 等齐，数一遍这一批的 `.rc`，文件数与派出去的件数对不上，整批作废；对得上再按起的次序逐个读（共用约束「执行前拒绝的写法」那一条的 ④）；不用 `wait "$pid"` 收退出码。
    - 等一行字之前先确认那一行真会写进那个文件。
    - 不改正在跑的脚本，要改的写同目录临时文件再 `mv` 换上（共用约束「执行前拒绝的写法」那一条的 ⑦）。
 4. 打中之后先答四句：分不分辨臂；被判的系统当时看不看得到判别它的东西；满足的是判据字面的哪一个分句；跑前条款给的每个改法在打中的那几格上还中不中。
--- a/.claude/agents/three-way-local-defense.md
+++ b/.claude/agents/three-way-local-defense.md
@@ -16,7 +16,7 @@
 - 立场是辩方：替被判出局、被攻、或被计划排除的一方，找它站得住的理由，写出具体机制（哪个文件、哪一步）；站不住就说站不住、为什么。辩护落得成数的（这一方在某段历史上是几、判红没有），同样按攻方第 1 步给事实表、要模型逐格填。提示里先替每一条写一个攻方问题（它被质疑的那一句，整行抄出处），再让本地模型逐条辩护；攻方问题同样要逐句核转述。
 - 输入里的「攻击面」换成「要辩护的一方」（主 agent 给：被复核的判决或被排除的候选，原文路径）。
 - 行号同攻方第 1 步：提示里明令答复不写行号，样本自带的行号在运行记录里标「模型自给、未核」。
-- 文件名形态（攻方定义里出现的 `-local-attack` 文件名，不论在哪一节，一律照这里换）：提示 `research/prompts/<轮>-local-defense.md`，核对表 `research/prompts/<轮>-local-defense-translation-audit.md`，样本 `research/prompts/<轮>-local-defense-output-s<n>.md`，运行记录 `research/prompts/<轮>-local-defense-runlog.md`。
+- 文件名形态（攻方定义里出现的 `-local-attack` 文件名，不论在哪一节，一律照这里换）：提示 `research/prompts/<轮>-local-defense.md`，核对表 `research/prompts/<轮>-local-defense-translation-audit.md`，运行记录 `research/prompts/<轮>-local-defense-runlog.md`。样本照攻方写成 `<前缀>-output-s<n>.md`，前缀取派发提示给的（与攻方「输入」那一项同），不写死。
 
 ## 没做什么（固定会有的）
 
--- a/.claude/agents/experiment-runner.md
+++ b/.claude/agents/experiment-runner.md
@@ -28,12 +28,12 @@
 2. 写 `research/e7-index-bench/src/bin/e<号>_<简称>.rs`，在 `research/e7-index-bench/Cargo.toml` 末尾追加它的 `[[bin]]`（⚠️ **跑前登记把装置写死成「入库装置」时改写 `crates/singlefs-harness/src/bin/e<号>_<简称>.rs`**：岔路问的是「`crates/` 今天这份代码的性质」时，另写一份独立模型答不了，写范围闸按 `crates/singlefs-harness/src/bin/e*.rs` 放行。那种 bin 只读、只驱动与观测，不许改 `crates/singlefs-core` 与 `crates/singlefs-checker`；变异表改追加进 `crates/mutations.tsv` 末尾、归门禁 59 号跑。**另有三条只对入库装置生效**：① **不许从 `crates/` 里引决定工作量、几何或判据门槛的常量**——这类参数在装置里写本地常量，值抄自 kb 的登记位并注明文件与行号，另加一条断言拿它与实现侧同名常量回比；引了就等于装置与实现共用一个数，两边一起错而逐格相等。这一条挡不住「kb 与实现一起取错」。② **写了入库装置就要让它进代码轮判决的点名清单**：门禁 56 号的正则罩得到 `crates/*/src/bin/*.rs`，不点名那一道会红，而那条义务落在跑整轮门禁的一方身上、不落在你身上——交回时写明你新建了哪个 bin。③ **什么算「入库装置」以跑前登记里写死的那一句为准**；登记里没写死就走 `research/` 那一支，别自己判）（`name` 用连字符，与 `replay.sh` 登记行里的 bin 名一致；只追加，不改别人的）：纯算术的计数模型、单测、钉绝对值的断言；取样点覆盖登记里每一个会跑到的取值。写完跑 `bash .claude/scripts/naming-lint.sh`，新文件里的缩写与单字母名在这一步改完再往下走，不留给后面的门禁阶段。
 3. 写变异表 `research/mutations/e<号>_<简称>.tsv`，在 `research/` 下跑完整张表：`cd research && nice -n 19 bash scripts/mutate.sh <bin 名> e7-index-bench/src/bin/<源文件> mutations/<变异表>`（两个路径都相对 `research/`；bin 名以 `research/e7-index-bench/Cargo.toml` 里 `[[bin]]` 的 `name` 为准，没有显式声明的取文件名去掉 `.rs`；报「基线就是红的」时先查是不是在仓根下跑、bin 名是不是写错）（收尾要有「已还原，基线仍全绿」）；没红的逐条按 `mutation-sampling.md` 分类。
 3b. 登记里的停机条款（第十一节里标「停机」的，通常是装置与 `crates/` 实现逐项对拍）在跑产物之前做，照原文真跑，不许只推；做不了就停下交回，不跑产物、不写实验页。登记的量一次做不完的，做完的那一段照常交，没做的逐条列进报告，实验页与索引行的状态写「部分已跑」，不写「已跑」。岔路单每一行都够判了（登记第十一节的够判停机），剩下的量不跑：实验页逐条标「够判后未跑」、写明各自对应哪一行，状态写「已跑（够判）」；够不够判由你按登记写的够判条件报，续不续由主 agent 定。
-4. 跑产物进 `research/results/`。那里已有的产物一个都不删、不挪、不覆盖（共用约束「执行前拒绝的写法」那一条的 ⑤）：这一次的输出写一个新文件名（带今天的日期或 `rN`，写之前 `ls` 确认没有同名的），跑完按这个新文件名认本轮才有的完成标记。重跑已有实验时拿新文件与已有的那份比：逐字节一致就删掉这一次的新文件、不新存；对不上就两份都留，实验页两份都点名、写明对不上，交主 agent 定哪份承重。生成产物用的二进制编在 `research/target/` 下，用最后一次源码改动编出来；生成产物与第 5 步跑 `replay.sh` 时都不设 `CARGO_TARGET_DIR`；草稿目录里的 target 只拿来迭代。
+4. 跑产物进 `research/results/`。那里已有的产物一个都不删、不挪、不覆盖（共用约束「执行前拒绝的写法」那一条的 ⑤）：这一次的输出写一个新文件名（带今天的日期或 `rN`，写之前 `ls` 确认没有同名的），跑完按这个新文件名认本轮才有的完成标记。这一次的新文件也不删：重跑已有实验时，第 5 步把 `research/scripts/replay.sh` 里这个实验的登记行改指新文件，旧的留着；拿新文件与已有的那份比，逐字节一致的在报告里写明，对不上的在实验页两份都点名、写明对不上，交主 agent 定哪份承重。生成产物用的二进制编在 `research/target/` 下，用最后一次源码改动编出来；生成产物与第 5 步跑 `replay.sh` 时都不设 `CARGO_TARGET_DIR`；草稿目录里的 target 只拿来迭代。
 4b. 产物出来之后先自查齐不齐：按登记的「臂 × 历史 × 几何」数一遍产物里每一类结果行的条数与字段，缺行、缺字段（例如某一族没有臂字段）、某一族一次都没造出被测形状，都列进报告，不许只看末行完成标记。报告里每个判据按每条臂全列判定，不挑「突出发现」报——对某条候选不利的那一半同样要写。分族、分臂的计数用一条命令从产物里数，报告与实验页贴那条命令和它的原样输出，不用文字重写族名单；给「这几族为什么是 0」写解释之前，先把那几族的产物行贴出来，确认它们真是 0。
-4c. 交回之前逐行读判决行：`grep -n 'name=verdict' <产物>` 列出这一次每份产物里的全部判决行（逐字节一致没新存的，读 `research/results/` 里那一份；输出不截断），任何字段取值是 `false` 或 `not_run`，或字段名表示违例、不匹配、歧义、失败的计数（名字里带 `violation`、`mismatch`、`ambiguous`、`fail` 的，例 `layer0_violations`、`width_mismatches`、`today_ambiguous`）取值不是 0，在报告里逐个点名：产物文件与行号、字段名、取值、为什么是这个值、是不是登记预期内的；一个都没有，也写一句「判决行 N 条，没有 false / not_run 字段，违例、不匹配、歧义、失败类计数都是 0」。
+4c. 交回之前逐行读判决行：`grep -n 'name=verdict' <产物>` 列出这一次每份产物里的全部判决行（输出不截断），在报告里逐个点名其中表示「没过」的字段：取值是布尔 `false` 的；取值是 `not_run` 的；取值是大于 0 的整数、而名字表示违例、不匹配、歧义、失败的计数（例 `layer0_violations`、`width_mismatches`、`today_ambiguous`）。取值是布尔 `true` 的、取值是 `zero` 的（例 `control_violations_ok=true`、`layer0_violations=zero`）与名字不表示「没过」的计数（例 `journal_differing_states`）不点名；拿不准的点名，写「拿不准」。每个点名写：产物文件与行号、字段名、取值、为什么是这个值、是不是登记预期内的；一个都没有，也写一句「判决行 N 条，没有 false / not_run 字段，违例、不匹配、歧义、失败类的整数计数都是 0」。
 5. 在 `research/scripts/replay.sh` 里登记复跑，跑一次 `bash research/scripts/replay.sh E<号>` 确认逐字节一致。送进虚机跑的实验（`vm-bench.sh`，含 `--selftest`）与 E152 装置是重型测试，你不跑：装置、登记与复跑命令写好就停，在交回里写明要跑什么、为什么，由主 agent 问用户（`.claude/kb/vm-harness.md`；共用约束「不做」一节里重型测试那一条）。装置要起子进程、读它的输出时，先把输出读完再转打，计时按登记写的进程与时钟取（`.claude/kb/vm-harness.md`「计时不在转发输出的循环里打时间戳」）；门禁 65 号查读子进程输出的循环里一边取时间一边输出。
-6. 跑阶段归属表登记给你的阶段（共用约束 `.claude/agent-common.md`「门禁」一节），84 号除外：它放到第 7 步写完实验页之后跑；这一次不写实验页的，在这一步最后跑，红了照写。其中要编译或复跑整张表的，开跑前照共用约束看负载。15 号（编整个 research 工作区）与 87 号（复跑全部已入库实验）不归你，收尾由 `gate-triage` 统一跑；你只跑第 5 步自己那一个实验的 `replay.sh`。
-7. 主 agent 要写实验页时：`.claude/kb/experiments/<号>-<简称>.md`，结果整行抄自产物并带文件名，写复跑命令、口径、它答不了的；索引行进 `.claude/kb/experiments.md`。写完（连同第 7b 步）跑 84 号，贴原样末行与退出码。
+6. 跑阶段归属表登记给你的阶段（共用约束 `.claude/agent-common.md`「门禁」一节），读实验页或实验索引的那几道除外（27、34、40、69、75、84、85、86、88、99 号）：它们放到第 7 步写完实验页之后跑；这一次不写实验页的，在这一步最后跑，红了照写，产物照第 4 步一个不删（86 号出路里「把 research 下那些文件删掉」那一句不照做，写进报告交主 agent）。其中要编译或复跑整张表的，开跑前照共用约束看负载。15 号（编整个 research 工作区）与 87 号（复跑全部已入库实验）不归你，收尾由 `gate-triage` 统一跑；你只跑第 5 步自己那一个实验的 `replay.sh`。
+7. 主 agent 要写实验页时：`.claude/kb/experiments/<号>-<简称>.md`，结果整行抄自产物并带文件名，写复跑命令、口径、它答不了的；索引行进 `.claude/kb/experiments.md`。写完（连同第 7b 步）跑第 6 步留下的那几道，各贴原样末行与退出码。
 7b. 实验页的 `### 影响的决策` 一节照第 7 步写实验页的规则写，形态与判据不在这份定义里复述。这一步你要做的三件：① 要写支撑或推翻时，先 `grep -n 'E<号>（' .claude/kb/decisions/<那条决策的文件>` 看那条分项的「**依据**」段引没引这个实验——引了才写支撑或推翻，没引就写备料，并把「这一格该不该升成支撑」列进报告交主 agent，**不自己去改决策文件**（写范围闸也不放行）。② 那条决策还在 `.claude/decision-links-pending` 里时不查依据段（门禁 75 号对清单里的决策整段跳过双向检查，那些决策还没有依据段），写决策级一行，并把它列进报告。③ 重跑已有实验、换了产物、加了历史条目之后，表里每一行都重新回看一遍。
 
 ## 写范围
--- a/.claude/agents/crash-verifier.md
+++ b/.claude/agents/crash-verifier.md
@@ -17,13 +17,13 @@
 
 - 这次改动的范围：`git diff --stat -- crates litmus` 原样，或提交区间。
 - 这一次是提交时跑还是用户要求时跑：决定命令带 `SINGLEFS_HEAVY_TESTS=commit` 还是 `=user-request`。
-- 54、55、57 号各自的内存上限（第 1b 步用）；55 号的不小于同时起的虚机数乘每台的内存（`.claude/gate.d/55-qemu-first-transaction.sh` 的 `MODES` 档数 × `research/scripts/vm-bench.sh` 的 `VM_MEM`）。
+- 54、55、57 号各自的内存上限（第 1b 步用），每道一个数：54、57 号的写明是量过的（峰值表 `research/scripts/memory-peaks.tsv` 里那一道整条经包装跑出的峰值）还是推的；55 号的不小于同时起的虚机数乘每台的内存（`.claude/gate.d/55-qemu-first-transaction.sh` 的 `MODES` 档数 × `research/scripts/vm-bench.sh` 的 `VM_MEM`）。
 - 报告路径与草稿目录。
 
 ## 做什么
 
 1. 每个阶段开跑前各照共用约束「不做」一节看一次负载；另有别的 `gate.sh` 正在跑 54 号、或有 `54-layer0-replay.sh --full` 在跑时，不起 54 号，等它结束。
-1b. 54、55、57 号整条经内存包装跑（共用约束「不做」一节「跑编译出来的代码经内存包装」那一条；上限取输入给的那一道的，退出码 250–254 照那一条办）：第 2 步里起这三道的每条命令，在 `bash <阶段文件>` 前面加 `bash research/scripts/run-with-memory-cap.sh <上限>`，写成 `SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash research/scripts/run-with-memory-cap.sh <上限> bash .claude/gate.d/<文件>`；59 号在里面逐条套了，照第 2 步直接跑，外面不再包一层。
+1b. 54、55、57 号整条经内存包装跑（共用约束「不做」一节「跑编译出来的代码经内存包装」那一条；上限取输入给的那一道的，退出码 250–254 照那一条办）：第 2 步里起这三道的每条命令，在 `bash <阶段文件>` 前面加 `bash research/scripts/run-with-memory-cap.sh <上限>`，写成 `SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash research/scripts/run-with-memory-cap.sh <上限> bash .claude/gate.d/<文件>`；59 号在里面逐条套了，74 号在阶段里面经包装，这两道照第 2 步直接跑，外面不再包一层。
 2. 只在提交时（或主 agent 转达用户要求时）跑你那几道，不跑 `gate.sh` 整轮与全量 `cargo test`（`.claude/hooks/heavy-test-guard.sh` 拒）。按共用约束 `.claude/agent-common.md`「门禁」一节从阶段归属表取登记给你的阶段，一次只跑一个；54、55、57、59 号命令带输入给的那个前缀：`SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/<文件>`，其余登记给你的轻阶段不带；54 号默认不带 `--full`（快档加核全绿标记），派发提示点名要层 0 全量时才带；每个记开始与结束时刻（`date -u`）和退出码。单个阶段超过 Bash 单次上限就后台跑，退出码写进文件（`{ SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/<文件>; echo "exit=$?"; } > <草稿目录>/<阶段>.log 2>&1`；退出码只认日志里的 `exit=` 那一行，不认起后台的那条命令自己的 `$?`），结束后读输出。别的会话同时在跑 cargo 时，编译会卡在「Blocking waiting for file lock」，照实记等了多久，不算这个阶段的耗时。
 3. 每个阶段原样抄它的「✓」行，或「✗」行与紧跟的「→」；退出码 77 记「本次未跑」，不记通过。
 4. 计数行原样抄：崩溃状态数、恢复结果、与 E142 产物或闭式比对的那几行。输出里读不到判定行的阶段记「作废」，不记通过。
--- a/.claude/agents/gate-triage.md
+++ b/.claude/agents/gate-triage.md
@@ -18,14 +18,13 @@
 - 这一轮的改动已经按 `research/scripts/stage-mine.py` 暂存了没有（没暂存就不派你，或者主 agent 明写「跑全量工作区」）。
 - 这一次是提交时跑还是用户要求时跑：决定命令带 `SINGLEFS_HEAVY_TESTS=commit` 还是 `=user-request`。
 - 这一轮暂存区 diff 的范围：`git diff --cached --stat` 原样。主 agent 明写跑全量工作区时，暂存区多半是空的，这时主 agent 另给「这一轮改过的文件清单」，归属按清单判。
-- `gate.sh` 整条经内存包装的上限（第 1b 步用）。
 - 报告路径与草稿目录。
 
 ## 做什么
 
 1. 开跑前照共用约束「不做」一节看负载；另有别的 `gate.sh` 在跑就等它结束。
-1b. 自己单跑的编译出来的代码（`cargo test`、测试二进制）照共用约束「不做」一节「跑编译出来的代码经内存包装」那一条经包装跑。`gate.sh` 整条经内存包装跑，上限取输入给的，退出码 250–254 照那一条办；里面 59 号、87 号逐条再经包装的照常跑（嵌套怎么排队见 `research/scripts/run-with-memory-cap.sh` 文件头「包装里再经包装跑的」那一句）。单跑的门禁阶段照第 2 步直接跑，外面不包一层（87 号经 `research/scripts/replay.sh` 逐条套了）。
-2. 只在提交时（或主 agent 转达用户要求时）跑：暂存过的跑 `SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash research/scripts/run-with-memory-cap.sh <上限> bash .claude/scripts/gate.sh --staged`（用户要求时 `=user-request`）；主 agent 明写全量的跑同一条、去掉 `--staged`（前缀照带）。54、55、57、59 这几道重阶段在 `gate.sh` 里照各自的复用判定走（54 号只核全绿标记），你不直接调它们，也不跑全量 `cargo test`（`.claude/hooks/heavy-test-guard.sh` 拒）；登记给你的其余阶段单跑时不带前缀，87 号单跑照带。超过 Bash 单次上限时后台跑、结束后读输出。
+1b. 自己单跑的编译出来的代码（`cargo test`、测试二进制）照共用约束「不做」一节「跑编译出来的代码经内存包装」那一条经包装跑；`gate.sh` 与单跑的门禁阶段照第 2 步直接跑，外面不包一层（87 号经 `research/scripts/replay.sh` 逐条套了，15、74 号在阶段里面经包装）。
+2. 只在提交时（或主 agent 转达用户要求时）跑：暂存过的跑 `SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/scripts/gate.sh --staged`（用户要求时 `=user-request`）；主 agent 明写全量的跑不带参数（前缀照带）。54、55、57、59 这几道重阶段在 `gate.sh` 里照各自的复用判定走（54 号只核全绿标记），你不直接调它们，也不跑全量 `cargo test`（`.claude/hooks/heavy-test-guard.sh` 拒）；登记给你的其余阶段单跑时不带前缀，87 号单跑照带。超过 Bash 单次上限时后台跑、结束后读输出。
 3. 每个红阶段：抄门禁给的「✗」行与紧跟的「→」下一步原样；看它点名的文件与行在不在暂存区 diff 的块里（`git diff --cached -U0 -- <文件>`）。在块里 ⇒「这一轮」；文件在但行不在块里、或文件不在暂存区 ⇒「不是这一轮」；红句说的是「这一批输入」本身的（54 号的「没有这批输入的全绿标记」「那一格的哈希不同」这类）不按它点名的标记路径判：这一轮的改动（`git diff --cached --name-only`；跑全量工作区时是主 agent 给的文件清单）碰了 `.claude/gate.d/stage-inputs.tsv` 里那一道那一行登记的路径 ⇒「这一轮」，下一步原样抄 54 号出路里的三行命令，一条都没碰 ⇒「不是这一轮」；工具没装、网关不通、`cargo` 不在 ⇒「环境」，用 `bash .claude/scripts/env.sh` 的对应行佐证；别的会话同时在跑同一个重阶段、进程被外部信号杀掉（输出里一行裸的 `Terminated`）⇒「并发」，贴开跑时与出事前后 `ps` 看到的同名进程。红阶段没有「✗」与「→」可抄的（被杀、崩溃），抄它最后 20 行输出，判「分不清」或「并发」，写明没有下一步可抄。分不清的写「分不清」与原因。
 4. 原样抄未实现清单与「由项目本地阶段覆盖」清单。别的会话同时在改仓、跑全量工作区时「工作区跑的过程中没变」那一条红了：照抄开跑、收尾两个指纹，判「不是这一轮」。`gate.sh` 不打印单个阶段的耗时，取不到的不估。
 
--- a/.claude/agents/implementation-writer.md
+++ b/.claude/agents/implementation-writer.md
@@ -23,7 +23,7 @@
 ## 做什么
 
 1. 开跑前照共用约束「不做」一节看负载。
-1b. 第 3 步的证红、第 4 步动到的测试二进制（副本里跑的也算）照共用约束「不做」一节「跑编译出来的代码经内存包装」那一条经包装跑；`cargo fmt --check`、`cargo clippy`、`cargo build` 不经它。
+1b. 第 3 步的证红、第 4 步动到的测试二进制（副本里跑的也算）照共用约束「不做」一节「跑编译出来的代码经内存包装」那一条经包装跑；`cargo fmt --check`、`cargo clippy`、`cargo build` 不经它。登记给你的 74 号在阶段里面经包装，照跑，外面不再包一层。
 2. 读条款与代码，写实现与测试。名字、分支、类型、错误照 `code-discipline.md`：不缩写、不写 `_ =>`、newtype、`expect` 写清依赖哪条不变量。
 3. 每条新测试证明会红：在草稿目录的仓副本里（`rsync -a --exclude target --exclude .git`）改坏被测代码一处，跑那条测试所在的整个测试二进制看它红，记「改坏哪一行 → 哪条断言红」与同时红了哪些测试；每份副本用它自己的 target（不把 `CARGO_TARGET_DIR` 指到几份副本共用的目录）；改坏的副本要还原时，从原件拷回之后对被改的文件 `touch`，或删掉这份副本重拷，不用 `rsync -a` 拷回了事；先跑一份不改动的副本：已经红的测试记成基线红集，变异证明只看基线红集之外的测试，你要证明的那条在基线红集里就停下交回；被测代码里有 `debug_assert` 的，debug 下先红的可能是它，照实记红在哪一条，要测试断言自己红就在 `--release` 下再跑一次。`crates/mutations.tsv` 在的话（门禁 59 号），每条再加一行变异，原文在文件里恰好命中一次。
 4. 变异证红按测试算、不按行算：每条新测试挑一行能让它红的变异证一次就够，其余行照样追加进 `crates/mutations.tsv` 末尾，整表复跑归最后的门禁 59 号；报告写明哪几行证过、哪几行留给 59 号。层 0 不跑：自己新加的层 0 流不在交回前跑，随提交时的层 0 全量验；已有流一条都不跑；新测试落在名字含 layer0 的测试二进制里的，第 3 步的证红也不跑，变异行照样追加、报告写明留给提交时的 59 号。层 0 全量每次提交跑一次（`main-agent.md`「什么时候派哪个 agent」里「暂存之后、提交之前跑门禁」那一步），不由实现员跑。交回前的验证只到这几样：动到的测试二进制（整个二进制，不按名字挑；名字含 layer0 的不跑）、第 3 步的证红、`cargo fmt --check`、`check.sh` 那一套 lint 下的 `cargo clippy`、`cargo build --offline --all-targets`、阶段归属表登记给你的门禁阶段（共用约束「门禁」一节，74 号在内），各贴末尾原样输出。其余门禁阶段、全量 `cargo test --all`、层 0 各流的快档与全量都不跑，留给提交时统一的那一次验证（`crash-verifier` 与整轮门禁）；派发提示另点名要跑的，不是重型测试的照点名跑，重型的照共用约束「不做」一节那一条交回主 agent。
--- a/.claude/agent-common.md
+++ b/.claude/agent-common.md
@@ -45,7 +45,7 @@
 - 不编译 Rust、不跑 `gate.sh` 全量，除非定义明写要做。跑脚本、跑模型一律加 `nice -n 19`。
 - 重型测试（名字含 `layer0` 的测试二进制与 54 号、QEMU、herd7、crates 变异整表、全量 `cargo test`（`--all` / `--workspace` / 工作区根裸跑）与 `check.sh`、`gate.sh` 整轮、87 号全部实验复跑、E152 装置）只在提交代码时跑一次、或用户要求时跑；子 agent 一律不跑，只跑自己动到的测试二进制（`cargo test -p <crate> --test <自己的目标>`、`--lib`）、fmt / clippy / build 与 `.claude/gate.d/` 下 54、55、57、59、87 之外的阶段。`crash-verifier` 只跑 54、55、57、59 号那几道，`gate-triage` 只跑 `gate.sh` 整轮与 87 号，都在提交时或用户要求时跑、命令带 `SINGLEFS_HEAVY_TESTS=commit`（用户要求时 `=user-request`）。项目 settings 里的 `.claude/hooks/heavy-test-guard.sh` 在执行前拒绝越出这些的命令；被拒了不换写法绕过去，在交回里写明要跑什么、为什么，交主 agent 去问用户。
 - 写进脚本文件、放进 `python3` 的 subprocess、`make` 这类间接起法同样算跑：自己写的验证链里不许出现重型测试的任何一步，名字含 `layer0` 的测试二进制也不许。
-- 跑编译出来的代码经内存包装：`cargo test` / `run` / `bench`（带 `--no-run` 只编不跑的不算）与直接执行 cargo 编出来的二进制（实验装置、测试二进制），一律经 `bash research/scripts/run-with-memory-cap.sh <上限> <命令…>` 跑；写进自己的脚本再跑的，整条脚本经它跑（`bash research/scripts/run-with-memory-cap.sh <上限> bash <脚本>`）。上限取派发提示给的（定义另写了来源的照定义），都没给就取 `research/scripts/replay.sh` 文件头 `REPLAY_MEMORY_CAP` 的默认值。`research/scripts/mutate.sh`、`research/scripts/replay.sh` 与门禁 59 号在里面逐条套了，直接跑，外面不再包一层。退出码 250–254 是包装自己的结局（含义见 `research/scripts/run-with-memory-cap.sh` 文件头「退出码」一段）：那一次的输出不算结果，命令与退出码写进报告，不绕开包装重跑。`cargo build`、`cargo clippy`、`cargo fmt` 不跑编出来的代码，不要求经它。
+- 跑编译出来的代码经内存包装：`cargo test` / `run` / `bench`（带 `--no-run` 只编不跑的不算）与直接执行 cargo 编出来的二进制（实验装置、测试二进制），一律经 `bash research/scripts/run-with-memory-cap.sh <上限> <命令…>` 跑；写进自己的脚本再跑的，整条脚本经它跑（`bash research/scripts/run-with-memory-cap.sh <上限> bash <脚本>`）。上限取派发提示给的（定义另写了来源的照定义），都没给就取 `research/scripts/replay.sh` 文件头 `REPLAY_MEMORY_CAP` 的默认值。`research/scripts/mutate.sh`、`research/scripts/replay.sh` 与门禁 59 号在里面逐条套了，门禁 15、74 号在阶段里面经包装（派发提示给了上限的，设进阶段文件头写的变量：15 号 `GATE_RESEARCH_BUILD_MEMORY_MAX`，74 号 `GATE_MODEL_DIFFERENTIAL_MEMORY_MAX`），都直接跑，外面不再包一层。退出码 250–254 是包装自己的结局（含义见 `research/scripts/run-with-memory-cap.sh` 文件头「退出码」一段）：那一次的输出不算结果，命令与退出码写进报告，不绕开包装重跑。`cargo build`、`cargo clippy`、`cargo fmt` 不跑编出来的代码，不要求经它。
 - 要停自己起的一条链（脚本、后台任务）时，先列出它的整棵子进程（`ps -o pid,ppid,args --ppid <pid>` 逐层往下），从最底层起逐个 `python3 .claude/singlefs-ai-sop/scripts/proc.py stop <pid>`（`proc.py` 只停给定的那一个，不带子进程），停完再列一遍，确认一个不剩；不许留下正在跑的那一步「让它跑完」。
 - 要编译、跑门禁阶段、跑变异或产物的，开跑前 `ps -o pid,args -u "$(id -u)"` 看负载：有性能测量在跑（`qemu-system`、`vm-bench.sh`、`e152-file-system-benchmark`、`fio`）就报「在等 pid …（命令）」停下；只有别的 `cargo`、`gate.sh` 在跑的，加 `nice -n 19` 照常跑（cargo 自己排队拿文件锁），报告里记下 `ps` 看到了什么、等锁等了多久。定义另有更严要求的照定义。
 - 读大文件（超过 500 行）先 grep 定位、再按行段读（Read 的 offset / limit），不整份读；同一份文件读过一次、之后没改过，就不再整份重读。长命令的输出落进草稿目录的文件，只读汇总行（`tail -n 20`、`grep -c`、`grep ✗`），不把整份日志、整份 diff 读进上下文。
@@ -61,7 +61,7 @@
     ① 起看门狗的错误写法：`research/scripts/watch.sh`、`research/scripts/agent-watch.py watch` 不用 run_in_background 起，或命令里带单独的 `&`、`nohup`、`disown`、`setsid`、把输出丢进 `/dev/null`。看门狗只有主 agent 起。
     ② 前台没有超时的等待循环：`until` / `while` 里有 `sleep`，外面没套 `timeout`。要等就用 run_in_background 起、结束本轮等完成通知，或 `python3 .claude/singlefs-ai-sop/scripts/proc.py wait <pid> --timeout <秒>`。
     ③ 把活放出追踪：`disown`、`coproc`、`setsid -f`、`nohup … &`、`tmux` / `screen` 的分离模式、不等到结束的 `systemd-run`。
-    ④ run_in_background 里有作业以单独的 `&` 收尾，之后同一条命令里没有 `wait`。要并行就每件把退出码写进自己的文件（`{ <命令>; echo "$?" > <文件>; } &`），最后单独一个不带参数的 `wait` 等齐，再按派活的次序逐个读那几个文件；不用 `wait "$pid"` 收。
+    ④ run_in_background 里有作业以单独的 `&` 收尾，之后同一条命令里没有 `wait`。要并行就每件把退出码写进自己的文件（`{ <命令>; echo "$?" > <文件>; } &`，文件名带批号与件号，每批开跑前先删掉这一批的文件），最后单独一个不带参数的 `wait` 等齐，再按派活的次序逐个读那几个文件；读之前数一遍，文件数与派出去的件数对不上，整批作废；不用 `wait "$pid"` 收。
     ⑤ 整份覆盖 `research/results/` 下已存在又没进 git 的产物：`>`、`>|`、`&>`、不带 `-a` 的 `tee`、`cp` / `mv` / `install` 的目标、`dd of=`、`truncate`。追加（`>>`、`tee -a`）与新文件名不拦；要换就按日期另存新文件名，旧的留着。
     ⑥ 终止进程不是点名一个自己起的进程号或任务号：`kill` 的目标带负号、是 0 或 `$PPID`、一次给几个、是命令替换或通配，在循环里逐个发（`proc.py stop` 同样），按名字、按 cgroup 或 `/proc` 挑进程，写死的进程号指到自己会话的祖先、别的会话起的进程或 SSH / VSCode / 本地模型服务，`systemctl` 停 SSH、登录会话、本地模型服务这类单元，关机重启。放行的是点名一个：`kill "$!"`、`kill %1`、单独一条 `python3 .claude/singlefs-ai-sop/scripts/proc.py stop <pid>`。
     ⑦ 在同一个 inode 上改已经存在的脚本（`.sh`、`.py` 或带执行位的文件，在仓里或 `/tmp/claude-1000/` 下）：`>`、`>|`、`&>`、不带 `-a` 的 `tee`、`cp` 的目标、`dd of=`、`truncate`、python 的 `open(…, 'w')` 这一类。改脚本写到同目录临时文件再 `mv` 换上，或用 `research/scripts/replace-once.py` / `insert-row.py` 定点改；追加与新建不拦。
--- a/.claude/main-agent.md
+++ b/.claude/main-agent.md
@@ -45,7 +45,7 @@
 
 同时在跑的重活（编译、测试、变异、产物、原型扫描）不止一个时，每份派发提示写一行「线程上限：N」，N = 整机核数 ÷ 同时在跑的重活数（向下取整，至少 1）；重活数变了，给在跑的 agent 发消息改 N。
 
-派崩溃验证员时给 54、55、57 号各自的内存上限，55 号的不小于同时起的虚机数乘每台的内存（算法在 `.claude/agents/crash-verifier.md`「输入」一节）；派门禁分诊员时给 `gate.sh` 整条经内存包装的上限。
+派崩溃验证员时给 54、55、57 号各自的内存上限，每道一个数，照 `.claude/agents/crash-verifier.md`「输入」一节给：54、57 号写明是量过的还是推的，55 号的不小于同时起的虚机数乘每台的内存。门禁 15、74 号在阶段里面经内存包装，要换它们的上限就在派发提示里给一个数，没给用 `research/scripts/replay.sh` 的 `REPLAY_MEMORY_CAP` 默认值。
 
 在不影响正确性的前提下写简练：只写对方干这件活要的东西——定义「输入」一节要的项、为哪几行岔路或哪个问题、定义与共用约束里没有的约束、交付什么交到哪；不复述定义与规则里已经写着的做法，不讲来龙去脉。事实、路径、行号、数与判据一个不少，嫌长就指到文件，不删内容。
 
--- a/.claude/hooks/ask-user-claim-guard.sh
+++ b/.claude/hooks/ask-user-claim-guard.sh
@@ -19,8 +19,8 @@
 #   ② 句子里有「推的」「没量过」「推测」「估计」「粗估」之一的放行。
 #   ③ 句子里有出处的放行，出处是下面任一样（在原句上找，引号与反引号里的也算）：
 #      反引号里的路径（带 / 的，或带登记过的扩展名 PATH_EXTENSIONS 的文件名）或命令（全 ASCII、至少两段，第一段是小写命令名、
-#      ./ 开头的路径或 VAR= 赋值）；文件:行号（文件带 / 或带登记过的扩展名，冒号全角半角都认；最后一个 / 之后的文件名里认汉字这类非 ASCII 的字，
-#      / 前面紧挨着的那个字要是 ASCII）；research/results/ 下的文件名；
+#      ./ 开头的路径或 VAR= 赋值）；文件:行号（冒号全角半角都认；文件带 / 的，/ 前面紧挨着的那个字要是 ASCII，最后一个 / 之后要么全是 ASCII，
+#      要么以登记过的扩展名收尾、名字里可以有汉字这类非 ASCII 的字；不带 / 的要以登记过的扩展名收尾）；research/results/ 下的文件名；
 #      name= 开头的产物行；「实测」「量过」「产物」「输出」前后 MEASUREMENT_WINDOW 个字以内带着一个数或路径，
 #      而且那个数或路径与这个词之间没有隔着断言词（「输出一定为 0」里 0 与「输出」之间隔着「一定」，不算）。
 #   ①有、②③都没有的句子逐句列出，退出 2；一句都没有就放行。stdin 读不出 JSON、questions 不是列表的放行。
@@ -46,7 +46,8 @@
 PATH_EXTENSIONS = ("rs", "md", "sh", "py", "tsv", "csv", "out", "txt", "log", "json", "jsonl", "toml", "yaml", "yml",
                    "conf", "litmus", "cat", "diff", "patch", "lock", "img")
 ASCII_PATH_CHARACTER = r"[A-Za-z0-9_.-]"
-# 文件名里的字：ASCII 之外还认汉字这类非 ASCII 的字（\w 在 str 上认 Unicode 字母与数字，不认，。：（）「」这类标点）；目录名只认 ASCII
+# 文件名里的字：ASCII 之外还认汉字这类非 ASCII 的字（\w 在 str 上认 Unicode 字母与数字，不认，。：（）「」这类标点）；目录名只认 ASCII。
+# 带汉字的文件名只在以登记过的扩展名收尾时才算（FILE_AND_LINE）：「O3/O8两格：2」这类「/ 后面是汉字跟冒号与数」的不是文件:行号
 FILE_NAME_CHARACTER = r"[\w.-]"
 EXTENSION_ALTERNATION = "|".join(PATH_EXTENSIONS)
 # 路径：带一个 /，或以登记过的扩展名收尾的文件名
@@ -54,7 +55,7 @@
     rf"{ASCII_PATH_CHARACTER}*[A-Za-z0-9_-]/{ASCII_PATH_CHARACTER}+"
     rf"|(?<![A-Za-z0-9_.-])[A-Za-z0-9_-]{ASCII_PATH_CHARACTER}*\.(?:{EXTENSION_ALTERNATION})(?![A-Za-z0-9])")
 FILE_AND_LINE = re.compile(
-    rf"(?:{ASCII_PATH_CHARACTER}*[A-Za-z0-9_-]/{FILE_NAME_CHARACTER}+"
+    rf"(?:{ASCII_PATH_CHARACTER}*[A-Za-z0-9_-]/(?:{ASCII_PATH_CHARACTER}+|{FILE_NAME_CHARACTER}*\.(?:{EXTENSION_ALTERNATION}))"
     rf"|[\w-]{FILE_NAME_CHARACTER}*\.(?:{EXTENSION_ALTERNATION}))[:：]\d+")
 RESULTS_FILE = re.compile(rf"research/results/{ASCII_PATH_CHARACTER}*[A-Za-z0-9_-]")
 PRODUCT_LINE = re.compile(r"(?<![A-Za-z0-9_])name=[^\s`'\"]")
@@ -202,6 +203,18 @@
          ["拆分提案第四十节：40 行写着它一定要走三方"]),
         ("必拒：斜杠分开的中文词跟冒号与数不是文件:行号", {"questions": [{"question": "抓到/无效/没红：40/0/0，这张表一定全抓了", "options": []}]},
          ["抓到/无效/没红：40/0/0，这张表一定全抓了"]),
+        ("必拒：/ 后面是不带扩展名的中文，跟冒号与数（O3/O8两格：2）", {"questions": [{"question": "O3/O8两格：2 格一定该升成打中", "options": []}]},
+         ["O3/O8两格：2 格一定该升成打中"]),
+        ("必拒：/ 后面是数字跟汉字（54/55号：2）", {"questions": [{"question": "54/55号：2 道一定要带前缀", "options": []}]},
+         ["54/55号：2 道一定要带前缀"]),
+        ("必拒：/ 后面是编号跟汉字（F1/F4两条：2）", {"questions": [{"question": "F1/F4两条：2 处一定都要改", "options": []}]},
+         ["F1/F4两条：2 处一定都要改"]),
+        ("必拒：/ 后面是汉字串（E142/第十六次跑：3）", {"questions": [{"question": "E142/第十六次跑：3 个字段一定是 0", "options": []}]},
+         ["E142/第十六次跑：3 个字段一定是 0"]),
+        ("必拒：ASCII 路径后面紧跟汉字（research/prompts下的判决：3）", {"questions": [{"question": "research/prompts下的判决：3 条一定都对", "options": []}]},
+         ["research/prompts下的判决：3 条一定都对"]),
+        ("必拒：/ 前面是汉字夹数、后面是汉字（抓到40/无效：0）", {"questions": [{"question": "抓到40/无效：0，这张表一定全抓了", "options": []}]},
+         ["抓到40/无效：0，这张表一定全抓了"]),
         ("必放：没有 questions", {}, []),
     ]
     results = []
@@ -225,8 +238,8 @@
         print("    → 看 sentences_of() / without_code_and_quotations() / assertion_spans() / has_provenance() 的判法与 stdin 入口，改完再跑 --selftest")
         return 1
     print(f"  ✓ 自检通过（查了 {len(results)} 种）：带断言词、同一句里没有出处也没写「推的」的句子拒绝，只拒没出处的那一句；"
-          "出处（反引号里的路径或命令、文件:行号（文件名是中文的也认）、research/results/ 下的文件、name= 产物行、实测带数）、「推的，没量过」、"
-          "反引号与「」里的断言词、「不一定」放行；中文文件名没带行号、中文词后面跟冒号与数的照拒；四例走真实的 stdin 入口")
+          "出处（反引号里的路径或命令、文件:行号（带扩展名的中文文件名也认）、research/results/ 下的文件、name= 产物行、实测带数）、「推的，没量过」、"
+          "反引号与「」里的断言词、「不一定」放行；中文文件名没带行号、中文词后面跟冒号与数、/ 后面是不带扩展名的汉字跟冒号与数的照拒；四例走真实的 stdin 入口")
     return 0
 
 def main():
--- a/.claude/gate.d/74-model-differential.sh
+++ b/.claude/gate.d/74-model-differential.sh
@@ -36,17 +36,36 @@
 fi
 
 TEST_BINARY="second_transaction_supplement_three_random_history"
+# 这里的 cargo test 跑编译出来的代码，经 research/scripts/run-with-memory-cap.sh 放进内存上限里跑（.claude/agent-common.md「跑编译出来的代码经内存包装」那一条），
+# 谁跑这一道都一样，外面不再包一层。上限取 GATE_MODEL_DIFFERENTIAL_MEMORY_MAX（派发提示给了上限的，跑这一道的 agent 设进它），
+# 没设取 research/scripts/replay.sh 的 REPLAY_MEMORY_CAP 默认值 8G。包装自己的结局（退出码 250–254）不是测试的判定，单独判红、单独给出路。
+MEMORY_CAP_RUNNER="$(cd "$(dirname "$0")/../.." && pwd)/research/scripts/run-with-memory-cap.sh"
+MODEL_DIFFERENTIAL_MEMORY_MAX="${GATE_MODEL_DIFFERENTIAL_MEMORY_MAX:-8G}"
 SECTIONS=("随机历史快档" "随机历史：偏向抬 F 之后复用的取样点" "随机历史：偏向抬 F 之后回退的取样点" "随机历史：越过原分配记录墙的取样点" "随机历史：小盘上逼近单元区墙的取样点" "随机历史：小盘上逼近单元区墙的取样点（空间准入判着）")
 
 log="$(mktemp)"
 if [[ -f Cargo.toml && -d crates/singlefs-harness ]]; then
-  if ! cargo test --release -p singlefs-harness --test "$TEST_BINARY" -- --nocapture >"$log" 2>&1; then
+  if bash "$MEMORY_CAP_RUNNER" "$MODEL_DIFFERENTIAL_MEMORY_MAX" cargo test --release -p singlefs-harness --test "$TEST_BINARY" -- --nocapture >"$log" 2>&1; then
+    cargo_exit=0
+  else
+    cargo_exit=$?
+  fi
+  if (( cargo_exit != 0 )); then
     tail -40 "$log"
-    echo "  ✗ 随机历史的测试二进制判红（上面是 cargo test 的尾部）"
-    echo "     → 怎么办：单跑看细节（经内存包装，上限取派发提示给的，没给就用 research/scripts/replay.sh 的 REPLAY_MEMORY_CAP 默认值 8G）："
-    echo "                bash research/scripts/run-with-memory-cap.sh 8G cargo test --release -p singlefs-harness --test $TEST_BINARY -- --nocapture"
-    echo "                签名是 ModelDisagreement 的，是实现与理想模型对某一步的结局答得不一样：先看模型那一格引的条款，再判改实现还是改模型。"
     rm -f "$log"
+    case "$cargo_exit" in
+      250|251|252|253|254)
+        echo "  ✗ 随机历史的测试二进制没判出结果：内存包装 research/scripts/run-with-memory-cap.sh 退 $cargo_exit（上限 $MODEL_DIFFERENTIAL_MEMORY_MAX），这一次的输出不算判定"
+        echo "     → 怎么办：照包装文件头「退出码」一段办：250 是这一道要的比上限多，把 GATE_MODEL_DIFFERENTIAL_MEMORY_MAX 设大再跑；"
+        echo "                251、252、254 先跑 bash research/scripts/run-with-memory-cap.sh --status 看 slice 被谁占着，空出来再整道重跑；不拿掉包装跑。"
+        ;;
+      *)
+        echo "  ✗ 随机历史的测试二进制判红（上面是 cargo test 的尾部）"
+        echo "     → 怎么办：单跑看细节（经内存包装，上限同这一道）："
+        echo "                bash research/scripts/run-with-memory-cap.sh $MODEL_DIFFERENTIAL_MEMORY_MAX cargo test --release -p singlefs-harness --test $TEST_BINARY -- --nocapture"
+        echo "                签名是 ModelDisagreement 的，是实现与理想模型对某一步的结局答得不一样：先看模型那一格引的条款，再判改实现还是改模型。"
+        ;;
+    esac
     exit 1
   fi
 elif [[ -f model-differential-cargo-output.log ]]; then
--- a/.claude/gate.d/15-research-build.sh
+++ b/.claude/gate.d/15-research-build.sh
@@ -22,8 +22,22 @@
   echo "               或把已装的 cargo 放进 PATH。跳过这一步等于 research/ 的数字没人验过。"
   exit 1; }
 
-out="$(cd "$R" && cargo test --release 2>&1)"
+# 这里的 cargo test 跑编译出来的代码，经 research/scripts/run-with-memory-cap.sh 放进内存上限里跑（.claude/agent-common.md「跑编译出来的代码经内存包装」那一条），
+# 谁跑这一道都一样，外面不再包一层。上限取 GATE_RESEARCH_BUILD_MEMORY_MAX（派发提示给了上限的，跑这一道的 agent 设进它），
+# 没设取 research/scripts/replay.sh 的 REPLAY_MEMORY_CAP 默认值 8G。峰值表的键写死：光看「cargo test --release」分不出是哪个工作区。
+# 包装自己的结局（退出码 250–254）不是构建与单测的判定，单独判红、单独给出路。
+MEMORY_CAP_RUNNER="$(cd "$(dirname "$0")/../.." && pwd)/research/scripts/run-with-memory-cap.sh"
+RESEARCH_BUILD_MEMORY_MAX="${GATE_RESEARCH_BUILD_MEMORY_MAX:-8G}"
+out="$(cd "$R" && RUN_WITH_MEMORY_CAP_KEY="gate 15-research-build: cargo test --release (research)" \
+  bash "$MEMORY_CAP_RUNNER" "$RESEARCH_BUILD_MEMORY_MAX" cargo test --release 2>&1)"
 rc=$?
+if (( rc >= 250 && rc <= 254 )); then
+  tail -8 <<<"$out" | sed 's/^/     /'
+  echo "  ✗ research 的构建与单测没判出结果：内存包装 research/scripts/run-with-memory-cap.sh 退 $rc（上限 $RESEARCH_BUILD_MEMORY_MAX），这一次的输出不算判定"
+  echo "     → 怎么办：照包装文件头「退出码」一段办：250 是这一道要的比上限多，把 GATE_RESEARCH_BUILD_MEMORY_MAX 设大再跑；"
+  echo "               251、252、254 先跑 bash research/scripts/run-with-memory-cap.sh --status 看 slice 被谁占着，空出来再整道重跑；不拿掉包装跑。"
+  exit 1
+fi
 if (( rc != 0 )); then
   echo "  ✗ research 的构建或单测没过（cargo test 退出码 $rc）"
   grep -E '^error|FAILED|panicked at' <<<"$out" | head -8 | sed 's/^/     /'
```

## 二、`git diff HEAD -- .claude/agents .claude/agent-common.md .claude/main-agent.md .claude/rules/implementation-workflow.md .claude/hooks/ask-user-claim-guard.sh .claude/gate.d/74-model-differential.sh .claude/gate.d/15-research-build.sh .claude/gate.d/stage-inputs.tsv .claude/gate.d/stage-owners.tsv`（另跑一次，原样；相对 HEAD 73ba4a4）

```diff
diff --git a/.claude/agent-common.md b/.claude/agent-common.md
index f42993c..97ede15 100644
--- a/.claude/agent-common.md
+++ b/.claude/agent-common.md
@@ -43,18 +43,36 @@
 - 不建、不改 `.claude/agents/`、`.claude/settings*.json`、`.claude/hooks/`、`.claude/rules/`、`.claude/singlefs-ai-sop/`，也不碰 `~/.claude/`。
 - 不读派发提示给的禁读清单里的文件。
 - 不编译 Rust、不跑 `gate.sh` 全量，除非定义明写要做。跑脚本、跑模型一律加 `nice -n 19`。
-- 重型测试（层 0、QEMU、herd7、crates 变异整表、全量 `cargo test`（`--all` / `--workspace` / 工作区根裸跑）与 `check.sh`、`gate.sh` 整轮、87 号全部实验复跑、E152 装置）只在提交代码时跑一次、或用户要求时跑；子 agent 一律不跑，只跑自己动到的测试二进制（`cargo test -p <crate> --test <自己的目标>`、`--lib`）、fmt / clippy / build 与 `.claude/gate.d/` 下 54、55、57、59、87 之外的阶段。`crash-verifier` 只跑 54、55、57、59 号那几道，`gate-triage` 只跑 `gate.sh` 整轮与 87 号，都在提交时或用户要求时跑、命令带 `SINGLEFS_HEAVY_TESTS=commit`（用户要求时 `=user-request`）。项目 settings 里的 `.claude/hooks/heavy-test-guard.sh` 在执行前拒绝越出这些的命令；被拒了不换写法绕过去，在交回里写明要跑什么、为什么，交主 agent 去问用户。
+- 重型测试（名字含 `layer0` 的测试二进制与 54 号、QEMU、herd7、crates 变异整表、全量 `cargo test`（`--all` / `--workspace` / 工作区根裸跑）与 `check.sh`、`gate.sh` 整轮、87 号全部实验复跑、E152 装置）只在提交代码时跑一次、或用户要求时跑；子 agent 一律不跑，只跑自己动到的测试二进制（`cargo test -p <crate> --test <自己的目标>`、`--lib`）、fmt / clippy / build 与 `.claude/gate.d/` 下 54、55、57、59、87 之外的阶段。`crash-verifier` 只跑 54、55、57、59 号那几道，`gate-triage` 只跑 `gate.sh` 整轮与 87 号，都在提交时或用户要求时跑、命令带 `SINGLEFS_HEAVY_TESTS=commit`（用户要求时 `=user-request`）。项目 settings 里的 `.claude/hooks/heavy-test-guard.sh` 在执行前拒绝越出这些的命令；被拒了不换写法绕过去，在交回里写明要跑什么、为什么，交主 agent 去问用户。
 - 写进脚本文件、放进 `python3` 的 subprocess、`make` 这类间接起法同样算跑：自己写的验证链里不许出现重型测试的任何一步，名字含 `layer0` 的测试二进制也不许。
-- 要停自己起的一条链（脚本、后台任务）时，先列出它的整棵子进程（`ps -o pid,ppid,args --ppid <pid>` 逐层往下），从最底层起逐个 `scripts/proc.py stop <pid>`（`proc.py` 只停给定的那一个，不带子进程），停完再列一遍，确认一个不剩；不许留下正在跑的那一步「让它跑完」。
+- 跑编译出来的代码经内存包装：`cargo test` / `run` / `bench`（带 `--no-run` 只编不跑的不算）与直接执行 cargo 编出来的二进制（实验装置、测试二进制），一律经 `bash research/scripts/run-with-memory-cap.sh <上限> <命令…>` 跑；写进自己的脚本再跑的，整条脚本经它跑（`bash research/scripts/run-with-memory-cap.sh <上限> bash <脚本>`）。上限取派发提示给的（定义另写了来源的照定义），都没给就取 `research/scripts/replay.sh` 文件头 `REPLAY_MEMORY_CAP` 的默认值。`research/scripts/mutate.sh`、`research/scripts/replay.sh` 与门禁 59 号在里面逐条套了，门禁 15、74 号在阶段里面经包装（派发提示给了上限的，设进阶段文件头写的变量：15 号 `GATE_RESEARCH_BUILD_MEMORY_MAX`，74 号 `GATE_MODEL_DIFFERENTIAL_MEMORY_MAX`），都直接跑，外面不再包一层。退出码 250–254 是包装自己的结局（含义见 `research/scripts/run-with-memory-cap.sh` 文件头「退出码」一段）：那一次的输出不算结果，命令与退出码写进报告，不绕开包装重跑。`cargo build`、`cargo clippy`、`cargo fmt` 不跑编出来的代码，不要求经它。
+- 要停自己起的一条链（脚本、后台任务）时，先列出它的整棵子进程（`ps -o pid,ppid,args --ppid <pid>` 逐层往下），从最底层起逐个 `python3 .claude/singlefs-ai-sop/scripts/proc.py stop <pid>`（`proc.py` 只停给定的那一个，不带子进程），停完再列一遍，确认一个不剩；不许留下正在跑的那一步「让它跑完」。
 - 要编译、跑门禁阶段、跑变异或产物的，开跑前 `ps -o pid,args -u "$(id -u)"` 看负载：有性能测量在跑（`qemu-system`、`vm-bench.sh`、`e152-file-system-benchmark`、`fio`）就报「在等 pid …（命令）」停下；只有别的 `cargo`、`gate.sh` 在跑的，加 `nice -n 19` 照常跑（cargo 自己排队拿文件锁），报告里记下 `ps` 看到了什么、等锁等了多久。定义另有更严要求的照定义。
 - 读大文件（超过 500 行）先 grep 定位、再按行段读（Read 的 offset / limit），不整份读；同一份文件读过一次、之后没改过，就不再整份重读。长命令的输出落进草稿目录的文件，只读汇总行（`tail -n 20`、`grep -c`、`grep ✗`），不把整份日志、整份 diff 读进上下文。
 - 派发提示里有「线程上限：N」的，编译、测试、变异、产物、原型扫描的每条命令都用 `bash research/scripts/capped.sh N <命令>` 起（它把 cargo 与 `crates/singlefs-harness` 各装置读的线程变量一次设成 N）；实验装置自己的线程变量（`KS_THREADS` 这类）同样设成不超过 N。主 agent 发消息改了 N，从下一条命令起照新的。
 - **崩溃点测试不衡量时间成本，也不为省时间缩范围**：要加崩溃点、要多枚举一档，就加。挂钟长不是收窄它的理由，也不用先算它值不值。
 - 禁止自行扩大任务范围，只改自己的部分。一个任务一个出口，达成出口即终止任务，扩大任务必须弹窗。禁止因为产物变化无限继续任务。尤其是门禁检测等任务，不属于自己任务的验证，如果验证为红给其他任务发消息，严禁自行修改扩大任务范围。
 - 本机常有别的会话在同一个仓里干活：只动这一轮自己的文件；看到别人没提交的改动不碰、不修。
-- 长活可以等，不给它设超时、不自己中途杀掉：编译、变异表、复跑这类长活用 Bash 的 `run_in_background` 起，起完结束本轮，完成时会通知你。交给它的命令要一直跑到真正的活结束才退出：不在里面再把活放到后台——不用 `disown`、`coproc`、`setsid -f`、`nohup … &`、`tmux`/`screen` 的分离模式、`systemd-run`，子 shell 或 `$( … )` 里也不写 `&`；`&` 只在同一条命令随后用不带参数的 `wait` 等齐时用。结束本轮就是这一条回复只写一句在等什么、不再调工具；前台命令的 `timeout` 不超过 240000 毫秒。
+- 长活可以等，不给它设超时、不自己中途杀掉：编译、变异表、复跑这类长活用 Bash 的 `run_in_background` 起，起完结束本轮，完成时会通知你。交给它的命令要一直跑到真正的活结束才退出：不在里面再把活放到后台——不用 `disown`、`coproc`、`setsid -f`、`nohup … &`、`tmux`/`screen` 的分离模式、`systemd-run`，子 shell 或 `$( … )` 里也不写 `&`；`&` 只在同一条命令随后用不带参数的 `wait` 等齐时用，每件的退出码照「执行前拒绝的写法」那一条 ④ 的写法收。结束本轮就是这一条回复只写一句在等什么、不再调工具；前台命令的 `timeout` 不超过 240000 毫秒。
   等长活时不起缓存计时器，结束本轮直接等完成通知。
-  等的时候看到进程不动、报错、或等的条件不会成立，把看到的命令、输出与时刻写进草稿目录里的进度记录，接着做定义里还能做的；结不结束、怎么处置由主 agent 定。主 agent 的看门狗（`research/scripts/agent-watch.py`）定时复检你；项目 settings 里的 Bash 检出 hook（`.claude/hooks/bash-command-detector.sh`）对五种写法在执行前拒绝：前台没有超时的等待循环、把活放出追踪（`disown`、`nohup … &` 这类）、run_in_background 里以单独的 `&` 收尾而之后同一条命令里没有 `wait` 的作业、用 `>`、不带 `-a` 的 `tee`、`cp` / `mv` / `install` 整份覆盖 `research/results/` 下已存在又没进 git 的产物（要换就按日期另存新文件名，旧的留着）、起看门狗的错误写法（只有主 agent 起看门狗）；run_in_background 里的等待循环与按模式找进程这类命令只记检出，交给主 agent 判断。等日志里的一行字之前，先确认那一行真会写进那个文件；找进程用写死的 pid，不用 `pgrep -f` / `pkill -f`（command-safety.md）。
+  等的时候看到进程不动、报错、或等的条件不会成立，把看到的命令、输出与时刻写进草稿目录里的进度记录，接着做定义里还能做的；结不结束、怎么处置由主 agent 定。主 agent 的看门狗（`research/scripts/agent-watch.py`）定时复检你；前台没有超时的等待循环、把活放出追踪、run_in_background 里后面没有 `wait` 的单独 `&` 在执行前被拒（「执行前拒绝的写法」那一条的 ②③④），run_in_background 里的等待循环只记检出，交给主 agent 判断。等日志里的一行字之前，先确认那一行真会写进那个文件；找进程用写死的 pid，不用 `pgrep -f` / `pkill -f`（command-safety.md）。
+- 执行前拒绝的写法：项目 settings 里注册的这几道 hook 在执行前（收工闸在收工前）拒绝这几类写法（退出 2，stderr 写原因与出路），主 agent 与子 agent 一样拒，只挂在一方才用的工具上的在条目里写明；拒的只是这一次调用，不停已经在跑的东西。被拒了照出路改写法，不换写法绕过去；改不成的写进报告交主 agent。
+  - Bash 检出 hook（`.claude/hooks/bash-command-detector.sh`）：② 只在前台拒（run_in_background 里的等待循环只记检出、不拒），④ 只在 run_in_background 里拒，其余前台与 run_in_background 一样拒：
+    ① 起看门狗的错误写法：`research/scripts/watch.sh`、`research/scripts/agent-watch.py watch` 不用 run_in_background 起，或命令里带单独的 `&`、`nohup`、`disown`、`setsid`、把输出丢进 `/dev/null`。看门狗只有主 agent 起。
+    ② 前台没有超时的等待循环：`until` / `while` 里有 `sleep`，外面没套 `timeout`。要等就用 run_in_background 起、结束本轮等完成通知，或 `python3 .claude/singlefs-ai-sop/scripts/proc.py wait <pid> --timeout <秒>`。
+    ③ 把活放出追踪：`disown`、`coproc`、`setsid -f`、`nohup … &`、`tmux` / `screen` 的分离模式、不等到结束的 `systemd-run`。
+    ④ run_in_background 里有作业以单独的 `&` 收尾，之后同一条命令里没有 `wait`。要并行就每件把退出码写进自己的文件（`{ <命令>; echo "$?" > <文件>; } &`，文件名带批号与件号，每批开跑前先删掉这一批的文件），最后单独一个不带参数的 `wait` 等齐，再按派活的次序逐个读那几个文件；读之前数一遍，文件数与派出去的件数对不上，整批作废；不用 `wait "$pid"` 收。
+    ⑤ 整份覆盖 `research/results/` 下已存在又没进 git 的产物：`>`、`>|`、`&>`、不带 `-a` 的 `tee`、`cp` / `mv` / `install` 的目标、`dd of=`、`truncate`。追加（`>>`、`tee -a`）与新文件名不拦；要换就按日期另存新文件名，旧的留着。
+    ⑥ 终止进程不是点名一个自己起的进程号或任务号：`kill` 的目标带负号、是 0 或 `$PPID`、一次给几个、是命令替换或通配，在循环里逐个发（`proc.py stop` 同样），按名字、按 cgroup 或 `/proc` 挑进程，写死的进程号指到自己会话的祖先、别的会话起的进程或 SSH / VSCode / 本地模型服务，`systemctl` 停 SSH、登录会话、本地模型服务这类单元，关机重启。放行的是点名一个：`kill "$!"`、`kill %1`、单独一条 `python3 .claude/singlefs-ai-sop/scripts/proc.py stop <pid>`。
+    ⑦ 在同一个 inode 上改已经存在的脚本（`.sh`、`.py` 或带执行位的文件，在仓里或 `/tmp/claude-1000/` 下）：`>`、`>|`、`&>`、不带 `-a` 的 `tee`、`cp` 的目标、`dd of=`、`truncate`、python 的 `open(…, 'w')` 这一类。改脚本写到同目录临时文件再 `mv` 换上，或用 `research/scripts/replace-once.py` / `insert-row.py` 定点改；追加与新建不拦。
+  - 上游的 `.claude/singlefs-ai-sop/scripts/claude-hooks/pattern-process-guard.sh`：命令位置上的 `pgrep -f`、`pkill -f`、`killall`。
+  - 重型测试闸（`.claude/hooks/heavy-test-guard.sh`）：越出重型测试那一条的命令（主 agent 不带 `SINGLEFS_HEAVY_TESTS` 前缀也拒）；子 agent 不经内存包装跑编译出来的代码（「跑编译出来的代码经内存包装」那一条）。
+  - 写闸（`.claude/hooks/write-guard.sh`，只看 Write / Edit 工具）：Write 整份覆盖仓里已存在又没进 git 的文件；项目子 agent 写到写范围表（`.claude/hooks/agent-write-scope.tsv`）里自己那几行之外；写进的内容里有撇号类角标（字符与起名的写法见 `.claude/rules/path-moves.md`「变体起新名字，不用角标」）。
+  - 交回闸（上游的 `.claude/singlefs-ai-sop/scripts/claude-hooks/handback-scratch-check.sh`，挂交回工具 SubagentHandback 与 SubagentStop，只判子 agent）：临时目录里自己建的编译目录、工作树与仓副本还在，交回报告里又没逐个写全路径与为什么不删（`.claude/singlefs-ai-sop/rules/session-wrapup.md`「5. 子 agent 交回之前，删掉自己建的编译目录与仓副本」）。
+  - 收工闸（上游的 `.claude/singlefs-ai-sop/scripts/claude-hooks/gate-reuse-check.sh`，挂 Stop 与 SubagentStop）：这个会话新建或改过的门禁与钩子没写 `# gate-similar:` / `# hook-events:`、该点名的已有门禁与钩子没点全、或整段抄了已有的一份（`.claude/singlefs-ai-sop/rules/sop-first.md`「加门禁或钩子之前，先找已有的」）。
+  - 弹窗闸（`.claude/hooks/ask-user-claim-guard.sh`，挂 AskUserQuestion，主 agent 用）：问句或选项说明里一句话带断言词（不可能、造不出、从来不、从来没有、一定、必然、永远不、绝不会、恒为），同一句里没有出处（反引号里的路径或命令、文件:行号、`research/results/` 下的文件、`name=` 开头的产物行、「实测」「量过」「产物」「输出」旁边带数或路径），也没写「推的」「没量过」「推测」「估计」「粗估」之一。
+  - 派发闸（`.claude/hooks/runner-dispatch-guard.sh`，挂 Agent / Task，主 agent 用）：派发提示要子 agent 跑重型测试；派 `experiment-runner` 没写「这一段回答的岔路：…」，续做没写「上一段岔路表里还差：…」或写了一行都不差；派 `kb-scribe`、`implementation-writer` 要改的文件落在还没写判决的三方轮的开工快照里。
+  - 续做闸（`.claude/hooks/continuation-guard.sh`，挂 SendMessage，主 agent 用）：给已经交回过、或最近一次任务通知是 failed / killed 的子 agent 发消息。
 
 ## 门禁
 
diff --git a/.claude/agents/crash-verifier.md b/.claude/agents/crash-verifier.md
index a0983a7..1069a6b 100644
--- a/.claude/agents/crash-verifier.md
+++ b/.claude/agents/crash-verifier.md
@@ -17,11 +17,13 @@ omitClaudeMd: true
 
 - 这次改动的范围：`git diff --stat -- crates litmus` 原样，或提交区间。
 - 这一次是提交时跑还是用户要求时跑：决定命令带 `SINGLEFS_HEAVY_TESTS=commit` 还是 `=user-request`。
+- 54、55、57 号各自的内存上限（第 1b 步用），每道一个数：54、57 号的写明是量过的（峰值表 `research/scripts/memory-peaks.tsv` 里那一道整条经包装跑出的峰值）还是推的；55 号的不小于同时起的虚机数乘每台的内存（`.claude/gate.d/55-qemu-first-transaction.sh` 的 `MODES` 档数 × `research/scripts/vm-bench.sh` 的 `VM_MEM`）。
 - 报告路径与草稿目录。
 
 ## 做什么
 
 1. 每个阶段开跑前各照共用约束「不做」一节看一次负载；另有别的 `gate.sh` 正在跑 54 号、或有 `54-layer0-replay.sh --full` 在跑时，不起 54 号，等它结束。
+1b. 54、55、57 号整条经内存包装跑（共用约束「不做」一节「跑编译出来的代码经内存包装」那一条；上限取输入给的那一道的，退出码 250–254 照那一条办）：第 2 步里起这三道的每条命令，在 `bash <阶段文件>` 前面加 `bash research/scripts/run-with-memory-cap.sh <上限>`，写成 `SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash research/scripts/run-with-memory-cap.sh <上限> bash .claude/gate.d/<文件>`；59 号在里面逐条套了，74 号在阶段里面经包装，这两道照第 2 步直接跑，外面不再包一层。
 2. 只在提交时（或主 agent 转达用户要求时）跑你那几道，不跑 `gate.sh` 整轮与全量 `cargo test`（`.claude/hooks/heavy-test-guard.sh` 拒）。按共用约束 `.claude/agent-common.md`「门禁」一节从阶段归属表取登记给你的阶段，一次只跑一个；54、55、57、59 号命令带输入给的那个前缀：`SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/<文件>`，其余登记给你的轻阶段不带；54 号默认不带 `--full`（快档加核全绿标记），派发提示点名要层 0 全量时才带；每个记开始与结束时刻（`date -u`）和退出码。单个阶段超过 Bash 单次上限就后台跑，退出码写进文件（`{ SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/<文件>; echo "exit=$?"; } > <草稿目录>/<阶段>.log 2>&1`；退出码只认日志里的 `exit=` 那一行，不认起后台的那条命令自己的 `$?`），结束后读输出。别的会话同时在跑 cargo 时，编译会卡在「Blocking waiting for file lock」，照实记等了多久，不算这个阶段的耗时。
 3. 每个阶段原样抄它的「✓」行，或「✗」行与紧跟的「→」；退出码 77 记「本次未跑」，不记通过。
 4. 计数行原样抄：崩溃状态数、恢复结果、与 E142 产物或闭式比对的那几行。输出里读不到判定行的阶段记「作废」，不记通过。
diff --git a/.claude/agents/experiment-runner.md b/.claude/agents/experiment-runner.md
index cf04f24..ae01d28 100644
--- a/.claude/agents/experiment-runner.md
+++ b/.claude/agents/experiment-runner.md
@@ -24,14 +24,16 @@ omitClaudeMd: true
 ## 做什么
 
 1. 开跑前照共用约束「不做」一节看负载。登记第六节里有耗时类的量时，跑产物之前连别的 `cargo`、`gate.sh` 也要等。
+1b. 跑单测（`cargo test`）、`cargo run` 与装置二进制（例 `research/target/release/e<号>-<简称>`）照共用约束「不做」一节「跑编译出来的代码经内存包装」那一条经包装跑；上限先取跑前登记给的，没给再照那一条取；包装退出码 250–254 的那一次输出不算产物。
 2. 写 `research/e7-index-bench/src/bin/e<号>_<简称>.rs`，在 `research/e7-index-bench/Cargo.toml` 末尾追加它的 `[[bin]]`（⚠️ **跑前登记把装置写死成「入库装置」时改写 `crates/singlefs-harness/src/bin/e<号>_<简称>.rs`**：岔路问的是「`crates/` 今天这份代码的性质」时，另写一份独立模型答不了，写范围闸按 `crates/singlefs-harness/src/bin/e*.rs` 放行。那种 bin 只读、只驱动与观测，不许改 `crates/singlefs-core` 与 `crates/singlefs-checker`；变异表改追加进 `crates/mutations.tsv` 末尾、归门禁 59 号跑。**另有三条只对入库装置生效**：① **不许从 `crates/` 里引决定工作量、几何或判据门槛的常量**——这类参数在装置里写本地常量，值抄自 kb 的登记位并注明文件与行号，另加一条断言拿它与实现侧同名常量回比；引了就等于装置与实现共用一个数，两边一起错而逐格相等。这一条挡不住「kb 与实现一起取错」。② **写了入库装置就要让它进代码轮判决的点名清单**：门禁 56 号的正则罩得到 `crates/*/src/bin/*.rs`，不点名那一道会红，而那条义务落在跑整轮门禁的一方身上、不落在你身上——交回时写明你新建了哪个 bin。③ **什么算「入库装置」以跑前登记里写死的那一句为准**；登记里没写死就走 `research/` 那一支，别自己判）（`name` 用连字符，与 `replay.sh` 登记行里的 bin 名一致；只追加，不改别人的）：纯算术的计数模型、单测、钉绝对值的断言；取样点覆盖登记里每一个会跑到的取值。写完跑 `bash .claude/scripts/naming-lint.sh`，新文件里的缩写与单字母名在这一步改完再往下走，不留给后面的门禁阶段。
 3. 写变异表 `research/mutations/e<号>_<简称>.tsv`，在 `research/` 下跑完整张表：`cd research && nice -n 19 bash scripts/mutate.sh <bin 名> e7-index-bench/src/bin/<源文件> mutations/<变异表>`（两个路径都相对 `research/`；bin 名以 `research/e7-index-bench/Cargo.toml` 里 `[[bin]]` 的 `name` 为准，没有显式声明的取文件名去掉 `.rs`；报「基线就是红的」时先查是不是在仓根下跑、bin 名是不是写错）（收尾要有「已还原，基线仍全绿」）；没红的逐条按 `mutation-sampling.md` 分类。
 3b. 登记里的停机条款（第十一节里标「停机」的，通常是装置与 `crates/` 实现逐项对拍）在跑产物之前做，照原文真跑，不许只推；做不了就停下交回，不跑产物、不写实验页。登记的量一次做不完的，做完的那一段照常交，没做的逐条列进报告，实验页与索引行的状态写「部分已跑」，不写「已跑」。岔路单每一行都够判了（登记第十一节的够判停机），剩下的量不跑：实验页逐条标「够判后未跑」、写明各自对应哪一行，状态写「已跑（够判）」；够不够判由你按登记写的够判条件报，续不续由主 agent 定。
-4. 跑产物进 `research/results/`；跑前删旧输出，跑完认本轮才有的完成标记。重跑已有实验时，`research/results/` 里已有的产物不覆盖：与它逐字节一致就不新存；对不上就按今天的日期另存一份，实验页两份都点名、写明对不上，交主 agent 定哪份承重。生成产物用的二进制编在 `research/target/` 下，用最后一次源码改动编出来；生成产物与第 5 步跑 `replay.sh` 时都不设 `CARGO_TARGET_DIR`；草稿目录里的 target 只拿来迭代。
+4. 跑产物进 `research/results/`。那里已有的产物一个都不删、不挪、不覆盖（共用约束「执行前拒绝的写法」那一条的 ⑤）：这一次的输出写一个新文件名（带今天的日期或 `rN`，写之前 `ls` 确认没有同名的），跑完按这个新文件名认本轮才有的完成标记。这一次的新文件也不删：重跑已有实验时，第 5 步把 `research/scripts/replay.sh` 里这个实验的登记行改指新文件，旧的留着；拿新文件与已有的那份比，逐字节一致的在报告里写明，对不上的在实验页两份都点名、写明对不上，交主 agent 定哪份承重。生成产物用的二进制编在 `research/target/` 下，用最后一次源码改动编出来；生成产物与第 5 步跑 `replay.sh` 时都不设 `CARGO_TARGET_DIR`；草稿目录里的 target 只拿来迭代。
 4b. 产物出来之后先自查齐不齐：按登记的「臂 × 历史 × 几何」数一遍产物里每一类结果行的条数与字段，缺行、缺字段（例如某一族没有臂字段）、某一族一次都没造出被测形状，都列进报告，不许只看末行完成标记。报告里每个判据按每条臂全列判定，不挑「突出发现」报——对某条候选不利的那一半同样要写。分族、分臂的计数用一条命令从产物里数，报告与实验页贴那条命令和它的原样输出，不用文字重写族名单；给「这几族为什么是 0」写解释之前，先把那几族的产物行贴出来，确认它们真是 0。
+4c. 交回之前逐行读判决行：`grep -n 'name=verdict' <产物>` 列出这一次每份产物里的全部判决行（输出不截断），在报告里逐个点名其中表示「没过」的字段：取值是布尔 `false` 的；取值是 `not_run` 的；取值是大于 0 的整数、而名字表示违例、不匹配、歧义、失败的计数（例 `layer0_violations`、`width_mismatches`、`today_ambiguous`）。取值是布尔 `true` 的、取值是 `zero` 的（例 `control_violations_ok=true`、`layer0_violations=zero`）与名字不表示「没过」的计数（例 `journal_differing_states`）不点名；拿不准的点名，写「拿不准」。每个点名写：产物文件与行号、字段名、取值、为什么是这个值、是不是登记预期内的；一个都没有，也写一句「判决行 N 条，没有 false / not_run 字段，违例、不匹配、歧义、失败类的整数计数都是 0」。
 5. 在 `research/scripts/replay.sh` 里登记复跑，跑一次 `bash research/scripts/replay.sh E<号>` 确认逐字节一致。送进虚机跑的实验（`vm-bench.sh`，含 `--selftest`）与 E152 装置是重型测试，你不跑：装置、登记与复跑命令写好就停，在交回里写明要跑什么、为什么，由主 agent 问用户（`.claude/kb/vm-harness.md`；共用约束「不做」一节里重型测试那一条）。装置要起子进程、读它的输出时，先把输出读完再转打，计时按登记写的进程与时钟取（`.claude/kb/vm-harness.md`「计时不在转发输出的循环里打时间戳」）；门禁 65 号查读子进程输出的循环里一边取时间一边输出。
-6. 跑阶段归属表登记给你的阶段（共用约束 `.claude/agent-common.md`「门禁」一节）；其中要编译或复跑整张表的，开跑前照共用约束看负载。15 号（编整个 research 工作区）与 87 号（复跑全部已入库实验）不归你，收尾由 `gate-triage` 统一跑；你只跑第 5 步自己那一个实验的 `replay.sh`。
-7. 主 agent 要写实验页时：`.claude/kb/experiments/<号>-<简称>.md`，结果整行抄自产物并带文件名，写复跑命令、口径、它答不了的；索引行进 `.claude/kb/experiments.md`。
+6. 跑阶段归属表登记给你的阶段（共用约束 `.claude/agent-common.md`「门禁」一节），读实验页或实验索引的那几道除外（27、34、40、69、75、84、85、86、88、99 号）：它们放到第 7 步写完实验页之后跑；这一次不写实验页的，在这一步最后跑，红了照写，产物照第 4 步一个不删（86 号出路里「把 research 下那些文件删掉」那一句不照做，写进报告交主 agent）。其中要编译或复跑整张表的，开跑前照共用约束看负载。15 号（编整个 research 工作区）与 87 号（复跑全部已入库实验）不归你，收尾由 `gate-triage` 统一跑；你只跑第 5 步自己那一个实验的 `replay.sh`。
+7. 主 agent 要写实验页时：`.claude/kb/experiments/<号>-<简称>.md`，结果整行抄自产物并带文件名，写复跑命令、口径、它答不了的；索引行进 `.claude/kb/experiments.md`。写完（连同第 7b 步）跑第 6 步留下的那几道，各贴原样末行与退出码。
 7b. 实验页的 `### 影响的决策` 一节照第 7 步写实验页的规则写，形态与判据不在这份定义里复述。这一步你要做的三件：① 要写支撑或推翻时，先 `grep -n 'E<号>（' .claude/kb/decisions/<那条决策的文件>` 看那条分项的「**依据**」段引没引这个实验——引了才写支撑或推翻，没引就写备料，并把「这一格该不该升成支撑」列进报告交主 agent，**不自己去改决策文件**（写范围闸也不放行）。② 那条决策还在 `.claude/decision-links-pending` 里时不查依据段（门禁 75 号对清单里的决策整段跳过双向检查，那些决策还没有依据段），写决策级一行，并把它列进报告。③ 重跑已有实验、换了产物、加了历史条目之后，表里每一行都重新回看一遍。
 
 ## 写范围
@@ -40,7 +42,7 @@ omitClaudeMd: true
 
 ## 产出
 
-- 报告：单测数（一条命令数出来）、变异三个数与分类、产物路径与完成标记、`replay.sh` 结果、实验页路径；登记修订了什么（没有就写没有）。
+- 报告：单测数（一条命令数出来）、变异三个数与分类、产物路径与完成标记、判决行的点名（第 4c 步）、`replay.sh` 结果、实验页路径；登记修订了什么（没有就写没有）。
 - 岔路表：登记里岔路单的每一行一行，写「已够判 / 还差什么 / 登记里剩下的量能不能让它翻面」，每格指到产物里算出它的那条命令。主 agent 续派时照这张表点名还差的行；这张表缺了，报告不算交齐。
 
 ## 没做什么（固定会有的）
diff --git a/.claude/agents/gate-triage.md b/.claude/agents/gate-triage.md
index 60c48be..62f1bb1 100644
--- a/.claude/agents/gate-triage.md
+++ b/.claude/agents/gate-triage.md
@@ -23,6 +23,7 @@ omitClaudeMd: true
 ## 做什么
 
 1. 开跑前照共用约束「不做」一节看负载；另有别的 `gate.sh` 在跑就等它结束。
+1b. 自己单跑的编译出来的代码（`cargo test`、测试二进制）照共用约束「不做」一节「跑编译出来的代码经内存包装」那一条经包装跑；`gate.sh` 与单跑的门禁阶段照第 2 步直接跑，外面不包一层（87 号经 `research/scripts/replay.sh` 逐条套了，15、74 号在阶段里面经包装）。
 2. 只在提交时（或主 agent 转达用户要求时）跑：暂存过的跑 `SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/scripts/gate.sh --staged`（用户要求时 `=user-request`）；主 agent 明写全量的跑不带参数（前缀照带）。54、55、57、59 这几道重阶段在 `gate.sh` 里照各自的复用判定走（54 号只核全绿标记），你不直接调它们，也不跑全量 `cargo test`（`.claude/hooks/heavy-test-guard.sh` 拒）；登记给你的其余阶段单跑时不带前缀，87 号单跑照带。超过 Bash 单次上限时后台跑、结束后读输出。
 3. 每个红阶段：抄门禁给的「✗」行与紧跟的「→」下一步原样；看它点名的文件与行在不在暂存区 diff 的块里（`git diff --cached -U0 -- <文件>`）。在块里 ⇒「这一轮」；文件在但行不在块里、或文件不在暂存区 ⇒「不是这一轮」；红句说的是「这一批输入」本身的（54 号的「没有这批输入的全绿标记」「那一格的哈希不同」这类）不按它点名的标记路径判：这一轮的改动（`git diff --cached --name-only`；跑全量工作区时是主 agent 给的文件清单）碰了 `.claude/gate.d/stage-inputs.tsv` 里那一道那一行登记的路径 ⇒「这一轮」，下一步原样抄 54 号出路里的三行命令，一条都没碰 ⇒「不是这一轮」；工具没装、网关不通、`cargo` 不在 ⇒「环境」，用 `bash .claude/scripts/env.sh` 的对应行佐证；别的会话同时在跑同一个重阶段、进程被外部信号杀掉（输出里一行裸的 `Terminated`）⇒「并发」，贴开跑时与出事前后 `ps` 看到的同名进程。红阶段没有「✗」与「→」可抄的（被杀、崩溃），抄它最后 20 行输出，判「分不清」或「并发」，写明没有下一步可抄。分不清的写「分不清」与原因。
 4. 原样抄未实现清单与「由项目本地阶段覆盖」清单。别的会话同时在改仓、跑全量工作区时「工作区跑的过程中没变」那一条红了：照抄开跑、收尾两个指纹，判「不是这一轮」。`gate.sh` 不打印单个阶段的耗时，取不到的不估。
diff --git a/.claude/agents/implementation-writer.md b/.claude/agents/implementation-writer.md
index 9e38948..15c0cec 100644
--- a/.claude/agents/implementation-writer.md
+++ b/.claude/agents/implementation-writer.md
@@ -23,9 +23,10 @@ omitClaudeMd: true
 ## 做什么
 
 1. 开跑前照共用约束「不做」一节看负载。
+1b. 第 3 步的证红、第 4 步动到的测试二进制（副本里跑的也算）照共用约束「不做」一节「跑编译出来的代码经内存包装」那一条经包装跑；`cargo fmt --check`、`cargo clippy`、`cargo build` 不经它。登记给你的 74 号在阶段里面经包装，照跑，外面不再包一层。
 2. 读条款与代码，写实现与测试。名字、分支、类型、错误照 `code-discipline.md`：不缩写、不写 `_ =>`、newtype、`expect` 写清依赖哪条不变量。
 3. 每条新测试证明会红：在草稿目录的仓副本里（`rsync -a --exclude target --exclude .git`）改坏被测代码一处，跑那条测试所在的整个测试二进制看它红，记「改坏哪一行 → 哪条断言红」与同时红了哪些测试；每份副本用它自己的 target（不把 `CARGO_TARGET_DIR` 指到几份副本共用的目录）；改坏的副本要还原时，从原件拷回之后对被改的文件 `touch`，或删掉这份副本重拷，不用 `rsync -a` 拷回了事；先跑一份不改动的副本：已经红的测试记成基线红集，变异证明只看基线红集之外的测试，你要证明的那条在基线红集里就停下交回；被测代码里有 `debug_assert` 的，debug 下先红的可能是它，照实记红在哪一条，要测试断言自己红就在 `--release` 下再跑一次。`crates/mutations.tsv` 在的话（门禁 59 号），每条再加一行变异，原文在文件里恰好命中一次。
-4. 变异证红按测试算、不按行算：每条新测试挑一行能让它红的变异证一次就够，其余行照样追加进 `mutations-append.tsv`，整表复跑归最后的门禁 59 号；报告写明哪几行证过、哪几行留给 59 号。层 0 不跑：自己新加的层 0 流不在交回前跑，随提交时的层 0 全量验；已有流一条都不跑；新测试落在名字含 layer0 的测试二进制里的，第 3 步的证红也不跑，变异行照样追加、报告写明留给提交时的 59 号。层 0 全量每次提交跑一次（`main-agent.md`「什么时候派哪个 agent」里「暂存之后、提交之前跑门禁」那一步），不由实现员跑。交回前的验证只到这几样：动到的测试二进制（整个二进制，不按名字挑；名字含 layer0 的不跑）、第 3 步的证红、`cargo fmt --check`、`check.sh` 那一套 lint 下的 `cargo clippy`、`cargo build --offline --all-targets`，各贴末尾原样输出。全量 `cargo test --all`、层 0 各流的快档与全量、门禁阶段都不跑，留给提交时统一的那一次验证（`crash-verifier` 与整轮门禁）；派发提示另点名要跑的，不是重型测试的照点名跑，重型的照共用约束「不做」一节那一条交回主 agent。交回前对主工作区现状 `git apply --check` 一次，打不上就在副本里按现状重做再交。
+4. 变异证红按测试算、不按行算：每条新测试挑一行能让它红的变异证一次就够，其余行照样追加进 `crates/mutations.tsv` 末尾，整表复跑归最后的门禁 59 号；报告写明哪几行证过、哪几行留给 59 号。层 0 不跑：自己新加的层 0 流不在交回前跑，随提交时的层 0 全量验；已有流一条都不跑；新测试落在名字含 layer0 的测试二进制里的，第 3 步的证红也不跑，变异行照样追加、报告写明留给提交时的 59 号。层 0 全量每次提交跑一次（`main-agent.md`「什么时候派哪个 agent」里「暂存之后、提交之前跑门禁」那一步），不由实现员跑。交回前的验证只到这几样：动到的测试二进制（整个二进制，不按名字挑；名字含 layer0 的不跑）、第 3 步的证红、`cargo fmt --check`、`check.sh` 那一套 lint 下的 `cargo clippy`、`cargo build --offline --all-targets`、阶段归属表登记给你的门禁阶段（共用约束「门禁」一节，74 号在内），各贴末尾原样输出。其余门禁阶段、全量 `cargo test --all`、层 0 各流的快档与全量都不跑，留给提交时统一的那一次验证（`crash-verifier` 与整轮门禁）；派发提示另点名要跑的，不是重型测试的照点名跑，重型的照共用约束「不做」一节那一条交回主 agent。
 5. 要改的文件在输入给的「别的会话正在改」清单里：不碰那个文件，也不做只能落在它上面的条款，报告写明卡在哪个文件、哪条条款；新文件要靠清单里的文件才进编译（模块声明、`Cargo.toml` 的 `[[test]]`）的，那条整条停下，不写那个新文件；其余照做。`crates/mutations.tsv` 例外：只在末尾追加整行，不改别人的行。
 6. 改动里碰到条款没写、要做设计判断的地方，停在那一处，写进报告交主 agent，不自己定。「停在那一处」的写法看这条分支走不走得到：
    - 走得到（包括要坏盘、崩溃、瞬时读错才走得到）：在**任何落盘动作之前**返回一个错误成员，名字说清是哪条条款没定、第一版不支持；写一条用例钉住「返回这个成员、盘上逐字节不变」（`DiskSnapshot` 之类比系统配置槽、根环、录制流步数），不钉它之后的行为。判定与后面的写要读同一份输入：写之前重算、对不上就不写。
diff --git a/.claude/agents/mutation-triage.md b/.claude/agents/mutation-triage.md
index d755ae9..ab37f4b 100644
--- a/.claude/agents/mutation-triage.md
+++ b/.claude/agents/mutation-triage.md
@@ -15,6 +15,7 @@ omitClaudeMd: true
 ## 输入（主 agent 必须给）
 
 - 变异表：`research/mutations/<名>.tsv`（配 bin 名与源文件）或 `crates/mutations.tsv` 里的条目名。bin 名以 `research/e7-index-bench/Cargo.toml` 里 `[[bin]]` 的 `name` 为准（多是连字符，源文件名是下划线）；主 agent 给的对不上、`mutate.sh` 退出码 6 时，照它报的改法改，报告里写明。
+- 给的是 `crates/mutations.tsv` 里的条目时：崩溃验证员跑的那一次门禁 59 号的输出路径（第 3 步从里面取条目）。
 - 改动前的三个数（有就给）。
 - 报告路径与草稿目录。
 
diff --git a/.claude/agents/three-way-attack.md b/.claude/agents/three-way-attack.md
index a8a6bd2..48fac28 100644
--- a/.claude/agents/three-way-attack.md
+++ b/.claude/agents/three-way-attack.md
@@ -26,6 +26,18 @@ omitClaudeMd: true
 1. 只攻分到的攻击面，不重复前几轮攻过的角度。
 2. 每个打中给出具体的历史或输入，每一步指到被判对象里许可它的那一句。
 3. 由用户决定的动作（回退之后先做什么、建几个对象）不写死：判一条臂「同一段历史上不中」之前，把这几步放开扫一遍，只固定前缀与故障（照 `.claude/rules/three-way-inference.md`「判决由主 agent 做，不由投票做」一节「攻方腿的装置把用户动作写死时，它报的「分辨臂」可能是装置造出来的」那一条办）。
+3b. 造历史、跑模型时这样取样：
+   - 盘用内存稀疏盘（`crates/singlefs-harness/src/crash.rs` 的 `SparseBlockDevice`），不在磁盘上建镜像文件；每个起点状态只建一次池（mkfs 与起点历史只跑一次），之后每段历史从内存里那一份拷（`SparseDevice`、`MemoryPool` 都能 `clone`）。
+   - 正式跑之前先跑一小段，按它估全量的挂钟；估出来超过 40 分钟，先缩历史、候选与几何的取样，报告里写明缩了什么、缩前缩后各多少、估时怎么算的。
+   - 崩溃状态不在缩的范围里（共用约束「不做」一节「崩溃点测试不衡量时间成本」那一条），照层 0 的枚举域取：调同一份文件里每一段都展开的 `enumerate_layer0` / `enumerate_layer0_versions`（录制写流，屏障与 FUA 切段，段内写的整写子集逐个枚举，不是只截前缀），每个崩溃状态恢复之后都跑 checker。缩只缩历史条数与长度、候选与几何的取样；留下的每段历史，它的崩溃状态一个不落。
+   - 在自己的原型里这样调枚举函数跑小流不算重型：原型里的流只许在原型里自己造，不从名字带 `layer0` 的用例里拷（拷文件、拷代码段、`include!`、`mod` 引进来都算拷）；测试目标的名字不带 `layer0`（带了 `heavy-test-guard.sh` 按名字拒）；每次跑之前用同一份文件的 `closed_form_state_count` 算出这一次全量的状态数，不超过约 10⁶ 个（几段历史合起来超过的，分几次跑）；这一轮全部原型跑的全量合起来不超过约 10⁷ 个状态。一段历史自己就超过约 10⁶ 的，那段不跑；再跑就要越过约 10⁷ 的，剩下的不跑；两样都写进报告交主 agent。
+   - 随机跑批分小批：每批的段数按先跑一小段估出的时长定，不给批设限时（包装的 `RUN_WITH_MEMORY_CAP_TIME_LIMIT`、外面套 `timeout` 都不用）；每批的段数与估时写进报告。
+3c. 内存与进程：
+   - 编译与跑都经内存包装：跑的照共用约束「不做」一节「跑编译出来的代码经内存包装」那一条，`cargo build` 也经它（`bash research/scripts/run-with-memory-cap.sh <上限> cargo build …`）。
+   - 只停自己起的进程，一次一个 `python3 .claude/singlefs-ai-sop/scripts/proc.py stop <pid>`（共用约束「执行前拒绝的写法」那一条的 ⑥）。
+   - 并行起的几件，每件把退出码写进自己的文件，文件名带批号与件号（`{ <命令>; echo "$?" > <草稿目录>/b<批号>-<件号>.rc; } &`），每批开跑前先删掉这一批的 `.rc`（`rm -f <草稿目录>/b<批号>-*.rc`）；最后单独一个不带参数的 `wait` 等齐，数一遍这一批的 `.rc`，文件数与派出去的件数对不上，整批作废；对得上再按起的次序逐个读（共用约束「执行前拒绝的写法」那一条的 ④）；不用 `wait "$pid"` 收退出码。
+   - 等一行字之前先确认那一行真会写进那个文件。
+   - 不改正在跑的脚本，要改的写同目录临时文件再 `mv` 换上（共用约束「执行前拒绝的写法」那一条的 ⑦）。
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
index ea58224..5c8776a 100644
--- a/.claude/agents/three-way-local-defense.md
+++ b/.claude/agents/three-way-local-defense.md
@@ -8,7 +8,7 @@ omitClaudeMd: true
 
 # 本地辩方腿（three-way-local-defense）
 
-开工先读 `.claude/agent-common.md`（这份定义开了 `omitClaudeMd`，要用的规则照共用约束「规则怎么读」一节读），再读 `.claude/agents/three-way-local-attack.md` 的「做什么」「写范围」两节：做法与它逐条相同，只有下面几处不同。
+开工先读 `.claude/agent-common.md`（这份定义开了 `omitClaudeMd`，要用的规则照共用约束「规则怎么读」一节读），再读 `.claude/agents/three-way-local-attack.md` 的「输入」「做什么」「写范围」「产出」四节：做法与它逐条相同，只有下面几处不同。
 开工先读：`.claude/agents/three-way-local-attack.md` 里「开工先读：」那一行点名的小节。
 
 ## 与本地攻方不同的地方
@@ -16,7 +16,7 @@ omitClaudeMd: true
 - 立场是辩方：替被判出局、被攻、或被计划排除的一方，找它站得住的理由，写出具体机制（哪个文件、哪一步）；站不住就说站不住、为什么。辩护落得成数的（这一方在某段历史上是几、判红没有），同样按攻方第 1 步给事实表、要模型逐格填。提示里先替每一条写一个攻方问题（它被质疑的那一句，整行抄出处），再让本地模型逐条辩护；攻方问题同样要逐句核转述。
 - 输入里的「攻击面」换成「要辩护的一方」（主 agent 给：被复核的判决或被排除的候选，原文路径）。
 - 行号同攻方第 1 步：提示里明令答复不写行号，样本自带的行号在运行记录里标「模型自给、未核」。
-- 文件名形态：提示 `research/prompts/<轮>-local-defense.md`，核对表 `research/prompts/<轮>-local-defense-translation-audit.md`，样本 `research/prompts/<轮>-local-defense-output-s<n>.md`，运行记录 `research/prompts/<轮>-local-defense-runlog.md`。
+- 文件名形态（攻方定义里出现的 `-local-attack` 文件名，不论在哪一节，一律照这里换）：提示 `research/prompts/<轮>-local-defense.md`，核对表 `research/prompts/<轮>-local-defense-translation-audit.md`，运行记录 `research/prompts/<轮>-local-defense-runlog.md`。样本照攻方写成 `<前缀>-output-s<n>.md`，前缀取派发提示给的（与攻方「输入」那一项同），不写死。
 
 ## 没做什么（固定会有的）
 
diff --git a/.claude/gate.d/15-research-build.sh b/.claude/gate.d/15-research-build.sh
index 01d24ce..7a62dfb 100755
--- a/.claude/gate.d/15-research-build.sh
+++ b/.claude/gate.d/15-research-build.sh
@@ -22,8 +22,22 @@ command -v cargo >/dev/null || {
   echo "               或把已装的 cargo 放进 PATH。跳过这一步等于 research/ 的数字没人验过。"
   exit 1; }
 
-out="$(cd "$R" && cargo test --release 2>&1)"
+# 这里的 cargo test 跑编译出来的代码，经 research/scripts/run-with-memory-cap.sh 放进内存上限里跑（.claude/agent-common.md「跑编译出来的代码经内存包装」那一条），
+# 谁跑这一道都一样，外面不再包一层。上限取 GATE_RESEARCH_BUILD_MEMORY_MAX（派发提示给了上限的，跑这一道的 agent 设进它），
+# 没设取 research/scripts/replay.sh 的 REPLAY_MEMORY_CAP 默认值 8G。峰值表的键写死：光看「cargo test --release」分不出是哪个工作区。
+# 包装自己的结局（退出码 250–254）不是构建与单测的判定，单独判红、单独给出路。
+MEMORY_CAP_RUNNER="$(cd "$(dirname "$0")/../.." && pwd)/research/scripts/run-with-memory-cap.sh"
+RESEARCH_BUILD_MEMORY_MAX="${GATE_RESEARCH_BUILD_MEMORY_MAX:-8G}"
+out="$(cd "$R" && RUN_WITH_MEMORY_CAP_KEY="gate 15-research-build: cargo test --release (research)" \
+  bash "$MEMORY_CAP_RUNNER" "$RESEARCH_BUILD_MEMORY_MAX" cargo test --release 2>&1)"
 rc=$?
+if (( rc >= 250 && rc <= 254 )); then
+  tail -8 <<<"$out" | sed 's/^/     /'
+  echo "  ✗ research 的构建与单测没判出结果：内存包装 research/scripts/run-with-memory-cap.sh 退 $rc（上限 $RESEARCH_BUILD_MEMORY_MAX），这一次的输出不算判定"
+  echo "     → 怎么办：照包装文件头「退出码」一段办：250 是这一道要的比上限多，把 GATE_RESEARCH_BUILD_MEMORY_MAX 设大再跑；"
+  echo "               251、252、254 先跑 bash research/scripts/run-with-memory-cap.sh --status 看 slice 被谁占着，空出来再整道重跑；不拿掉包装跑。"
+  exit 1
+fi
 if (( rc != 0 )); then
   echo "  ✗ research 的构建或单测没过（cargo test 退出码 $rc）"
   grep -E '^error|FAILED|panicked at' <<<"$out" | head -8 | sed 's/^/     /'
diff --git a/.claude/gate.d/74-model-differential.sh b/.claude/gate.d/74-model-differential.sh
index e921b91..760f495 100755
--- a/.claude/gate.d/74-model-differential.sh
+++ b/.claude/gate.d/74-model-differential.sh
@@ -36,16 +36,36 @@ if [[ "$scope_rc" != 0 ]]; then
 fi
 
 TEST_BINARY="second_transaction_supplement_three_random_history"
+# 这里的 cargo test 跑编译出来的代码，经 research/scripts/run-with-memory-cap.sh 放进内存上限里跑（.claude/agent-common.md「跑编译出来的代码经内存包装」那一条），
+# 谁跑这一道都一样，外面不再包一层。上限取 GATE_MODEL_DIFFERENTIAL_MEMORY_MAX（派发提示给了上限的，跑这一道的 agent 设进它），
+# 没设取 research/scripts/replay.sh 的 REPLAY_MEMORY_CAP 默认值 8G。包装自己的结局（退出码 250–254）不是测试的判定，单独判红、单独给出路。
+MEMORY_CAP_RUNNER="$(cd "$(dirname "$0")/../.." && pwd)/research/scripts/run-with-memory-cap.sh"
+MODEL_DIFFERENTIAL_MEMORY_MAX="${GATE_MODEL_DIFFERENTIAL_MEMORY_MAX:-8G}"
 SECTIONS=("随机历史快档" "随机历史：偏向抬 F 之后复用的取样点" "随机历史：偏向抬 F 之后回退的取样点" "随机历史：越过原分配记录墙的取样点" "随机历史：小盘上逼近单元区墙的取样点" "随机历史：小盘上逼近单元区墙的取样点（空间准入判着）")
 
 log="$(mktemp)"
 if [[ -f Cargo.toml && -d crates/singlefs-harness ]]; then
-  if ! cargo test --release -p singlefs-harness --test "$TEST_BINARY" -- --nocapture >"$log" 2>&1; then
+  if bash "$MEMORY_CAP_RUNNER" "$MODEL_DIFFERENTIAL_MEMORY_MAX" cargo test --release -p singlefs-harness --test "$TEST_BINARY" -- --nocapture >"$log" 2>&1; then
+    cargo_exit=0
+  else
+    cargo_exit=$?
+  fi
+  if (( cargo_exit != 0 )); then
     tail -40 "$log"
-    echo "  ✗ 随机历史的测试二进制判红（上面是 cargo test 的尾部）"
-    echo "     → 怎么办：单跑看细节：cargo test --release -p singlefs-harness --test $TEST_BINARY -- --nocapture"
-    echo "                签名是 ModelDisagreement 的，是实现与理想模型对某一步的结局答得不一样：先看模型那一格引的条款，再判改实现还是改模型。"
     rm -f "$log"
+    case "$cargo_exit" in
+      250|251|252|253|254)
+        echo "  ✗ 随机历史的测试二进制没判出结果：内存包装 research/scripts/run-with-memory-cap.sh 退 $cargo_exit（上限 $MODEL_DIFFERENTIAL_MEMORY_MAX），这一次的输出不算判定"
+        echo "     → 怎么办：照包装文件头「退出码」一段办：250 是这一道要的比上限多，把 GATE_MODEL_DIFFERENTIAL_MEMORY_MAX 设大再跑；"
+        echo "                251、252、254 先跑 bash research/scripts/run-with-memory-cap.sh --status 看 slice 被谁占着，空出来再整道重跑；不拿掉包装跑。"
+        ;;
+      *)
+        echo "  ✗ 随机历史的测试二进制判红（上面是 cargo test 的尾部）"
+        echo "     → 怎么办：单跑看细节（经内存包装，上限同这一道）："
+        echo "                bash research/scripts/run-with-memory-cap.sh $MODEL_DIFFERENTIAL_MEMORY_MAX cargo test --release -p singlefs-harness --test $TEST_BINARY -- --nocapture"
+        echo "                签名是 ModelDisagreement 的，是实现与理想模型对某一步的结局答得不一样：先看模型那一格引的条款，再判改实现还是改模型。"
+        ;;
+    esac
     exit 1
   fi
 elif [[ -f model-differential-cargo-output.log ]]; then
diff --git a/.claude/gate.d/stage-inputs.tsv b/.claude/gate.d/stage-inputs.tsv
index bb117d7..c07e478 100644
--- a/.claude/gate.d/stage-inputs.tsv
+++ b/.claude/gate.d/stage-inputs.tsv
@@ -1,15 +1,27 @@
-# 每道阶段读哪几条路径。一行一条：<阶段文件名><制表符><路径，空格分隔><制表符>#<为什么是这几条>
+# 准入登记表：每道门禁阶段与每个接了准入的实验读哪几条路径、开跑前要满足哪些条件（门禁与实验共用这一份，唯一登记位）。一行一条：
+#   <键><制表符><路径，空格分隔>[<制表符><准入条件，空格分隔>]<制表符>#<为什么是这几条>
 #
-# 用途：判「这一道能不能复用上一次的判定」。判据是 research/scripts/stage-must-run.sh：
-#   这几条路径，在 refs/sop/staged-green 那棵树与这一次的暂存树之间，git 说变没变。
+# 判法都在一个模块里：research/scripts/admission.py（文件头写全了各子命令、条件的写法与退出码）。
+#   门禁阶段（键是 .claude/gate.d/ 下的文件名）：判「这一道能不能复用上一次的判定」，入口 research/scripts/stage-must-run.sh：
+#     这几条路径，在 refs/sop/staged-green 那棵树与这一次的暂存树之间，git 说变没变。
+#     第三列（门禁行）：command=<可执行名>、readwrite=<路径>、probe=<仓内脚本>[:<参数>…] 是开跑前要齐的前提，
+#     阶段经 admission.py gate-preconditions 判，没齐判红（不退 77）；environment=<仓内脚本>[:<参数>…] 打出来的东西（工具的版本这类
+#     不在 git 树里的输入）另进复用判定，阶段判绿之后调 admission.py gate-record-environment 记下它。
+#   实验（键是实验号；同一个实验另有一种调用方式的写 E<号>/<方式>）：装置开跑前调 admission.py experiment：
+#     这几条路径下每个文件的 sha256、登记行本身与工具链汇成输入指纹，research/results/ 里有一份产物头上记着同一个键、同一个指纹 ⇒ 拒绝重跑；
+#     第三列的准入条件没齐 ⇒ 拒绝开跑。路径写成 @<别的键> 就是那个键登记的全部路径。
 #
-# ⚠️ 这份清单**自己也进比对**（脚本无条件把它加进路径列表）。少写一条输入，那条输入就永远
-# 不会让这道阶段重跑——而清单本身变了必定重跑，改清单的代价因此是「下一趟全跑一次」，不是零。
+# ⚠️ 这份清单**自己也进门禁的比对**（stage-must-run.sh 无条件把它加进路径列表）。少写一条输入，那条输入就永远
+# 不会让这道阶段重跑——而清单本身变了必定重跑，改清单的代价因此是「下一趟全跑一次」，不是零；加一个实验行也一样。
+# 实验那一侧进指纹的只有它自己那几行（连同 @ 引到的行），改别的行不让那个实验放行。
 #
 # 宁宽勿窄：多写一条路径只会多跑几趟，少写一条会让一次真的改动被跳过。拿不准就写上。
-# 只写从仓库根起的路径；目录写成带斜杠的前缀（git diff 的路径限定就按前缀匹配）。
-54-layer0-replay.sh	crates/ Cargo.toml Cargo.lock	# 跑 crates 的崩溃点重放；两条流的用例编译期与运行期都不读 .claude/kb/layout/ 与 research/results/（段序列与布局表的比对归 52 号）
-55-qemu-first-transaction.sh	crates/ Cargo.toml Cargo.lock research/scripts/ research/results/	# 起虚机跑 crates 的装置，虚机与复跑脚本都在 research/scripts/
-59-crates-mutation-replay.sh	crates/ Cargo.toml Cargo.lock research/scripts/run-with-memory-cap.sh	# 按 crates/mutations.tsv 逐条改坏 crates/ 里的源码再跑点名的测试；每条经 run-with-memory-cap.sh 带内存上限跑，它判不判得出撞顶也是这一道的判据
-74-model-differential.sh	crates/ Cargo.toml Cargo.lock	# 拿 crates 里的实现与只住内存的理想模型对拍
+# 只写从仓库根起的路径；目录写成带斜杠的前缀（git diff 与 git ls-files 的路径限定都按前缀匹配）。
+54-layer0-replay.sh	crates/ Cargo.toml Cargo.lock	command=cargo command=rustc	# 跑 crates 的崩溃点重放；两条流的用例编译期与运行期都不读 .claude/kb/layout/ 与 research/results/（段序列与布局表的比对归 52 号）。前提：cargo 与 rustc（两档都要 cargo -V && rustc -V 算输入哈希，快档与 --full 都起 cargo test）
+55-qemu-first-transaction.sh	crates/ Cargo.toml Cargo.lock research/scripts/ research/results/	command=qemu-system-x86_64 readwrite=/dev/kvm probe=research/scripts/vm-kernel.sh:--check	# 起虚机跑 crates 的装置，虚机与复跑脚本都在 research/scripts/；这几条路径同时是改动范围（change-touches-crates.sh）的前缀。前提是 .claude/kb/vm-harness.md「三个前置」：缺 QEMU 装 qemu-system-x86；/dev/kvm 不可读写查 kvm 组成员身份，不许用 setfacl 补；找不到可读的内核镜像就跑 bash research/scripts/vm-kernel.sh，按它打印的路径设 SINGLEFS_KERNEL
+57-lkmm.sh	litmus/ .claude/scripts/lkmm.sh .claude/scripts/fetch-deps.sh .claude/singlefs-ai-sop/scripts/lib.sh crates/	probe=.claude/scripts/lkmm.sh:--herd7-version environment=.claude/scripts/lkmm.sh:--herd7-version	# 判 litmus/ 下每条 Never 的对照组、代码绑定与 herd7 判定。lkmm.sh 是判法，它 source 的 lib.sh、找不到内核树时调的 fetch-deps.sh 一并登记；crates/ 整个取：litmus 的 singlefs-models 锚点今天指 crates/singlefs-core/src/transaction.rs 与 recovery.rs，读 litmus 文件名的测试在 crates/singlefs-harness/tests/publish_order_matches_litmus.rs，而 lkmm.sh 按文件名在全部 crates/**/*.rs 里找测试、新加一条 litmus 的锚点可以指到 crates/ 任何一处，只登记这三份会让下一条新锚点的改动被跳过（宁宽勿窄）。herd7 的版本不在 git 树里，经第三列 environment= 进复用判定（lkmm.sh --herd7-version 打的那一行，57 号判绿之后记进 git common-dir）；前提是找得到 herd7（probe= 同一个入口，PATH 里没有就试 opam 的环境；缺了 opam install herdtools7）。内核树的 tools/memory-model 不进判定，靠 stage-must-run.sh 的 24 小时复用上限兜
+59-crates-mutation-replay.sh	crates/ Cargo.toml Cargo.lock research/scripts/run-with-memory-cap.sh	command=cargo	# 按 crates/mutations.tsv 逐条改坏 crates/ 里的源码再跑点名的测试；每条经 run-with-memory-cap.sh 带内存上限跑，它判不判得出撞顶也是这一道的判据。前提：cargo（装 Rust 工具链，bash .claude/scripts/env.sh 会报缺什么）
+74-model-differential.sh	crates/ Cargo.toml Cargo.lock research/scripts/run-with-memory-cap.sh research/scripts/capped.sh	# 拿 crates 里的实现与只住内存的理想模型对拍；阶段里的 cargo test 经内存包装（定义收尾第二轮判决 G1），包装脚本改了也要重跑
 87-replay.sh	crates/ Cargo.toml Cargo.lock research/e7-index-bench/ research/scripts/ research/results/	# 复跑入库的实验产物，装置在 research/e7-index-bench/，登记表与脚本在 research/scripts/
+E142	research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs research/e7-index-bench/src/lib.rs research/e7-index-bench/Cargo.toml research/Cargo.toml research/Cargo.lock research/mutations/e142_first_transaction_dry_run.tsv research/results/e142-first-txn-dry-run-2026-09-25-r16-arm-n15.out .claude/kb/decisions/08-核心索引结构.md .claude/kb/decisions/13-验证路线.md .claude/kb/decisions/16-发布语义.md .claude/kb/decisions/18-块里携带什么信息.md .claude/kb/decisions/23-journal的角色与格式.md .claude/kb/layout/01-first-txn.md .claude/kb/decisions/22-单元原子性怎么合成.md .claude/kb/decisions/15-格式冻结政策.md .claude/kb/feature-bits.md research/e7-index-bench/src/bin/e142_region_diff_independent.rs research/mutations/e142_region_diff_independent.tsv crates/ Cargo.toml Cargo.lock	# E142（第一个事务的干跑）默认调用（replay.sh 的 driver_e142 那一种）。装置源码、它 use 的 e7_index_bench 库、research 的 Cargo 清单与锁、变异表；driver_e142 传给装置的臂 N15 参照产物；replay.sh 不进（跑完把登记行指到新产物就是改它，进了指纹，新产物一存进来就对不上自己；驱动换了参数而这几条路径没变时要靠强制开关）；跑前登记 research/prompts/e142-r17-prereg.md 第二节被测条款所在的 D8（核心索引结构）、D13（验证路线）、D16（发布语义）、D18（块里携带什么信息）、D23（journal 的角色与格式）五份决策正文，加上宽度对账逐格抄的 layout/01（跑前登记第二节没列它，宁宽勿窄加上）；第十八次跑的跑前登记 research/prompts/e142-r18-prereg.md 第二节另加被测条款 D22（单元原子性怎么合成）、D15（格式冻结政策）与 feature-bits.md，那一次用独立比对 bin e142_region_diff_independent 判改前改后，它与它的变异表一并登记（2026-09-26 主 agent 按设计员报的漏列补上）；crates/ 整个取：driver_e142 编的 e142_first_transaction_write_dump 在 singlefs-harness，它依赖 checker、core、format 另外三个 crate，四个就是整个 crates/，多出来的只有 crates/mutations.tsv；按文件精确取要跟着模块图走，漏一个模块就是该跑的不跑
+E142/layer0	@E142	question-row=research/prompts/m2-keyspace-rerun-questions.md#6:够判[：:][^（(|]*对照(本身)?是好的 product-field=E142:verdict:control_violations_ok=true product-field=E142:verdict:positive_control_main_geometry_ok=true	# E142 第二段（设了 E142_LAYER0_MAIN 的调用：主臂层 0 整轮），输入与默认调用相同。前提照跑前登记 research/prompts/e142-r17-prereg.md 6.2、6.3：问题单第 6 行判成「对照是好的」之后才跑，且 Q142.26 caught = 12（最新产物判决行 positive_control_main_geometry_ok）；control_violations_ok 是第 6 行够判那一格的计数判定
diff --git a/.claude/gate.d/stage-owners.tsv b/.claude/gate.d/stage-owners.tsv
index ab9f4c7..7218728 100644
--- a/.claude/gate.d/stage-owners.tsv
+++ b/.claude/gate.d/stage-owners.tsv
@@ -45,6 +45,7 @@
 62-stage-owners.sh	gate-triage	本表与门禁目录、agent 定义
 70-citations.sh	kb-scribe,prior-art	外部引用还核得动；写进 kb 的是书记员，查源码树的是调研员
 80-absolute-assertions.sh	experiment-runner	实验钉绝对值的断言
+84-verdict-false-named.sh	experiment-runner	实验产物判决行里的 false 字段要被实验页点名：判决行由执行员交回之前逐行读、实验页由它写，写完就跑
 85-repro-command.sh	experiment-runner	实验复跑命令
 86-experiment-orphans.sh	experiment-runner	实验号在 kb 里有正文
 87-replay.sh	gate-triage	入库实验复现（全部实验复跑十几分钟，2026-09-17 从 experiment-runner 挪到收尾，理由同 15 号；执行员只跑自己那一个实验的 replay.sh）
diff --git a/.claude/hooks/ask-user-claim-guard.sh b/.claude/hooks/ask-user-claim-guard.sh
index 349219c..5445ecc 100755
--- a/.claude/hooks/ask-user-claim-guard.sh
+++ b/.claude/hooks/ask-user-claim-guard.sh
@@ -19,7 +19,8 @@
 #   ② 句子里有「推的」「没量过」「推测」「估计」「粗估」之一的放行。
 #   ③ 句子里有出处的放行，出处是下面任一样（在原句上找，引号与反引号里的也算）：
 #      反引号里的路径（带 / 的，或带登记过的扩展名 PATH_EXTENSIONS 的文件名）或命令（全 ASCII、至少两段，第一段是小写命令名、
-#      ./ 开头的路径或 VAR= 赋值）；文件:行号（文件带 / 或带登记过的扩展名，冒号全角半角都认）；research/results/ 下的文件名；
+#      ./ 开头的路径或 VAR= 赋值）；文件:行号（冒号全角半角都认；文件带 / 的，/ 前面紧挨着的那个字要是 ASCII，最后一个 / 之后要么全是 ASCII，
+#      要么以登记过的扩展名收尾、名字里可以有汉字这类非 ASCII 的字；不带 / 的要以登记过的扩展名收尾）；research/results/ 下的文件名；
 #      name= 开头的产物行；「实测」「量过」「产物」「输出」前后 MEASUREMENT_WINDOW 个字以内带着一个数或路径，
 #      而且那个数或路径与这个词之间没有隔着断言词（「输出一定为 0」里 0 与「输出」之间隔着「一定」，不算）。
 #   ①有、②③都没有的句子逐句列出，退出 2；一句都没有就放行。stdin 读不出 JSON、questions 不是列表的放行。
@@ -45,14 +46,17 @@ PLACEHOLDER = "□"
 PATH_EXTENSIONS = ("rs", "md", "sh", "py", "tsv", "csv", "out", "txt", "log", "json", "jsonl", "toml", "yaml", "yml",
                    "conf", "litmus", "cat", "diff", "patch", "lock", "img")
 ASCII_PATH_CHARACTER = r"[A-Za-z0-9_.-]"
+# 文件名里的字：ASCII 之外还认汉字这类非 ASCII 的字（\w 在 str 上认 Unicode 字母与数字，不认，。：（）「」这类标点）；目录名只认 ASCII。
+# 带汉字的文件名只在以登记过的扩展名收尾时才算（FILE_AND_LINE）：「O3/O8两格：2」这类「/ 后面是汉字跟冒号与数」的不是文件:行号
+FILE_NAME_CHARACTER = r"[\w.-]"
 EXTENSION_ALTERNATION = "|".join(PATH_EXTENSIONS)
 # 路径：带一个 /，或以登记过的扩展名收尾的文件名
 PATH_LIKE = re.compile(
     rf"{ASCII_PATH_CHARACTER}*[A-Za-z0-9_-]/{ASCII_PATH_CHARACTER}+"
     rf"|(?<![A-Za-z0-9_.-])[A-Za-z0-9_-]{ASCII_PATH_CHARACTER}*\.(?:{EXTENSION_ALTERNATION})(?![A-Za-z0-9])")
 FILE_AND_LINE = re.compile(
-    rf"(?:{ASCII_PATH_CHARACTER}*[A-Za-z0-9_-]/{ASCII_PATH_CHARACTER}+"
-    rf"|[A-Za-z0-9_-]{ASCII_PATH_CHARACTER}*\.(?:{EXTENSION_ALTERNATION}))[:：]\d+")
+    rf"(?:{ASCII_PATH_CHARACTER}*[A-Za-z0-9_-]/(?:{ASCII_PATH_CHARACTER}+|{FILE_NAME_CHARACTER}*\.(?:{EXTENSION_ALTERNATION}))"
+    rf"|[\w-]{FILE_NAME_CHARACTER}*\.(?:{EXTENSION_ALTERNATION}))[:：]\d+")
 RESULTS_FILE = re.compile(rf"research/results/{ASCII_PATH_CHARACTER}*[A-Za-z0-9_-]")
 PRODUCT_LINE = re.compile(r"(?<![A-Za-z0-9_])name=[^\s`'\"]")
 CODE_SPAN = re.compile(r"`[^`\n]*`")
@@ -164,6 +168,7 @@ def decide(hook_input):
 def selftest(hook_dir):
     original = "可达状态里 defer 一项不可能为 0……重新搭环境也造不出来"
     one_sourced_one_not = "实测 `research/results/e156-r3.out:12` 这一格 defer=0。重新搭环境也造不出别的值"
+    chinese_file_name_sourced = "records/2026-09-16-subagent拆分提案.md:951 写着静态分支一定要走定义三方"
     # (说明, tool_input, 应当被拒的句子——空列表就是应当放行)
     cases = [
         ("必拒：原话那句", {"questions": [{"question": original, "options": [{"label": "采纳", "description": "自证用造的基底"}]}]}, [original]),
@@ -189,6 +194,27 @@ def selftest(hook_dir):
         ("必放：反引号里的路径", {"questions": [{"question": "回收写在 `crates/core/src/mount.rs` 的挂载路径里，defer 一定会被清掉", "options": []}]}, []),
         ("必放：不在反引号里的文件:行号", {"questions": [{"question": "见 transaction.rs：120，回收一定排在提交之后", "options": []}]}, []),
         ("必放：research/results/ 下的文件名", {"questions": [{"question": "research/results/e156-r3.out 那一格必然是 0", "options": []}]}, []),
+        ("必放：不在反引号里、文件名是中文的文件:行号", {"questions": [{"question": chinese_file_name_sourced, "options": []}]}, []),
+        ("必放：多层目录下中文文件名的文件:行号", {"questions": [{"question": ".claude/kb/decisions/08-核心索引结构.md:40 写着这一格一定不变", "options": []}]}, []),
+        ("必放：不带 / 的中文文件名，全角冒号行号", {"questions": [{"question": "见 08-核心索引结构.md：40，这一格一定不变", "options": []}]}, []),
+        ("必拒：中文文件名没带行号", {"questions": [{"question": "records/2026-09-16-subagent拆分提案.md 里写着静态分支一定要走定义三方", "options": []}]},
+         ["records/2026-09-16-subagent拆分提案.md 里写着静态分支一定要走定义三方"]),
+        ("必拒：中文词后面跟冒号与数不是文件:行号", {"questions": [{"question": "拆分提案第四十节：40 行写着它一定要走三方", "options": []}]},
+         ["拆分提案第四十节：40 行写着它一定要走三方"]),
+        ("必拒：斜杠分开的中文词跟冒号与数不是文件:行号", {"questions": [{"question": "抓到/无效/没红：40/0/0，这张表一定全抓了", "options": []}]},
+         ["抓到/无效/没红：40/0/0，这张表一定全抓了"]),
+        ("必拒：/ 后面是不带扩展名的中文，跟冒号与数（O3/O8两格：2）", {"questions": [{"question": "O3/O8两格：2 格一定该升成打中", "options": []}]},
+         ["O3/O8两格：2 格一定该升成打中"]),
+        ("必拒：/ 后面是数字跟汉字（54/55号：2）", {"questions": [{"question": "54/55号：2 道一定要带前缀", "options": []}]},
+         ["54/55号：2 道一定要带前缀"]),
+        ("必拒：/ 后面是编号跟汉字（F1/F4两条：2）", {"questions": [{"question": "F1/F4两条：2 处一定都要改", "options": []}]},
+         ["F1/F4两条：2 处一定都要改"]),
+        ("必拒：/ 后面是汉字串（E142/第十六次跑：3）", {"questions": [{"question": "E142/第十六次跑：3 个字段一定是 0", "options": []}]},
+         ["E142/第十六次跑：3 个字段一定是 0"]),
+        ("必拒：ASCII 路径后面紧跟汉字（research/prompts下的判决：3）", {"questions": [{"question": "research/prompts下的判决：3 条一定都对", "options": []}]},
+         ["research/prompts下的判决：3 条一定都对"]),
+        ("必拒：/ 前面是汉字夹数、后面是汉字（抓到40/无效：0）", {"questions": [{"question": "抓到40/无效：0，这张表一定全抓了", "options": []}]},
+         ["抓到40/无效：0，这张表一定全抓了"]),
         ("必放：没有 questions", {}, []),
     ]
     results = []
@@ -199,7 +225,8 @@ def selftest(hook_dir):
     for label, tool_input, want_code, want_in_stderr in (
             ("stdin：原话那句拒绝", cases[0][1], 2, "重新搭环境也造不出来"),
             ("stdin：两句里拒没出处的那句", cases[3][1], 2, "重新搭环境也造不出别的值"),
-            ("stdin：带出处的放行", cases[7][1], 0, "")):
+            ("stdin：带出处的放行", cases[7][1], 0, ""),
+            ("stdin：中文文件名的文件:行号放行", {"questions": [{"question": chinese_file_name_sourced, "options": []}]}, 0, "")):
         completed = subprocess.run(["bash", script], input=json.dumps({"tool_name": "AskUserQuestion", "tool_input": tool_input}),
                                    capture_output=True, text=True)
         stderr_ok = want_in_stderr in completed.stderr if want_in_stderr else completed.stderr == ""
@@ -211,8 +238,8 @@ def selftest(hook_dir):
         print("    → 看 sentences_of() / without_code_and_quotations() / assertion_spans() / has_provenance() 的判法与 stdin 入口，改完再跑 --selftest")
         return 1
     print(f"  ✓ 自检通过（查了 {len(results)} 种）：带断言词、同一句里没有出处也没写「推的」的句子拒绝，只拒没出处的那一句；"
-          "出处（反引号里的路径或命令、文件:行号、research/results/ 下的文件、name= 产物行、实测带数）、「推的，没量过」、"
-          "反引号与「」里的断言词、「不一定」放行；三例走真实的 stdin 入口")
+          "出处（反引号里的路径或命令、文件:行号（带扩展名的中文文件名也认）、research/results/ 下的文件、name= 产物行、实测带数）、「推的，没量过」、"
+          "反引号与「」里的断言词、「不一定」放行；中文文件名没带行号、中文词后面跟冒号与数、/ 后面是不带扩展名的汉字跟冒号与数的照拒；四例走真实的 stdin 入口")
     return 0
 
 def main():
diff --git a/.claude/main-agent.md b/.claude/main-agent.md
index f59d2ba..789caa1 100644
--- a/.claude/main-agent.md
+++ b/.claude/main-agent.md
@@ -11,13 +11,13 @@
 - **禁止自行扩大任务范围，只改自己的部分**。一个任务一个出口，达成出口即终止任务，扩大任务必须弹窗。禁止因为产物变化无限继续任务。尤其是门禁检测等任务，不属于自己任务的验证，如果验证为红给其他任务发消息，严禁自行修改扩大任务范围。
 - **测试结果与预期不符，禁止立刻直接修改方向和结论，先检查代码中有没有bug。**
 
-- **禁止在subagent中跑重型测试（层 0 全量、QEMU、herd7、`crates` 变异整表、全量测试、整轮门禁）等**。
+- **禁止在subagent中跑重型测试**，完整清单以 `.claude/rules/implementation-workflow.md`「重型测试只在提交时跑」那一节开头的清单为准。
 
 ## 一轮怎么开、怎么收
 
 1. **开工前写死这一轮的课题与出口**：要关哪几项、做到什么算完（形态照实验的岔路单）。
 2. **冒出新点先判阻塞**：只问一句——**这个决策能不能留到下次开？它挡不挡着本轮的课题？** 挡着就现在修；不挡就记进该记的地方（里程碑收口表、`.claude/kb/checks-owed.md`、决策正文），往后延。判据不是花了多少 token、跑了多久。
-3. **重型测试**（层 0 全量、QEMU、herd7、`crates` 变异整表、全量测试、整轮门禁）提交之外任务确实要跑，先弹窗问用户，同意了才跑（`.claude/rules/implementation-workflow.md`「重型测试只在提交时跑」）。
+3. **重型测试**（哪几样以 `.claude/rules/implementation-workflow.md`「重型测试只在提交时跑」那一节开头的清单为准）提交之外任务确实要跑，先弹窗问用户，同意了才跑。
 4. **派实现员之前先列出它要动的 `crates/` 文件**，与在跑的实现员要动的文件有交集，就排在那一个后面，或并给同一个实现员做，不并行改同一批文件。
 5. **弹窗问用户之前，问句里每一句事实写出处**（产物的整行、命令与输出、文件:行号）；推出来、没量过的，句子里写明「推的，没量过」。
 6. **派一个 agent 之前说清它关的是哪一条已经抓到的问题**；说不出来的，那条该进记录、不该进本轮。
@@ -27,7 +27,7 @@
 
 ## 派出去之后
 
-盯住，不能一直干等，也不强制结束：子 agent 与脚本可以跑很久，结不结束、怎么处置由主 agent 看了定；hook 不停任务和脚本：检出类的只记录；拒绝一种危险写法的（起看门狗的错误写法、前台无超时等待循环、把活放出追踪、run_in_background 里后面没有 `wait` 的单独 `&`、用 shell 的 `>`、`>|`、`&>`、不带 `-a` 的 `tee`、`cp`、`mv`、`install` 整份覆盖 `research/results/` 下已存在且未跟踪的产物、子 agent 跑重型测试、写撇号类角标、覆盖未跟踪文件）在执行前拒绝，不碰已经在跑的东西。每次派发（包括续做）之后，立刻用 Bash 的 `run_in_background` 起看门狗 `bash research/scripts/watch.sh <这次派出的 agent id，逗号分隔>`（只填 agent 号：会话目录它自己找，阈值在 `research/scripts/watch.conf` 里配，不在命令行上手敲；调用里不再加 `&`、`nohup`、`disown`）：它每 4 分钟复检一次子 agent 的会话记录与本实例底下的长进程、每 15 秒读一次 hook 的检出记录，发现没超时的等待循环、一次工具调用超过 8 分钟、同一命令反复且输出不变、按模式找进程、10 分钟无动静、结束本轮而手里没有在跑的后台任务、进程写的文件 20 分钟不涨、本实例底下有没带 `SINGLEFS_HEAVY_TESTS` 前缀的重型测试进程（确认接着盯写 `--ack 进程:<pid>`）、子 agent 从派发起的运行时间跨过一个整小时（每个整点报一次，那个整点之后主 agent 给它发过消息就不报；一格多长在 `watch.conf` 的 `ask-every-minutes`），或 hook 检出本会话的问题命令，就退出并叫醒主 agent；子 agent 全部交回或被停也退出，没有告警时每 60 分钟定时回报一次。
+盯住，不能一直干等，也不强制结束：子 agent 与脚本可以跑很久，结不结束、怎么处置由主 agent 看了定；hook 不停任务和脚本：检出类的只记录；拒绝一种危险写法的在执行前拒绝，不碰已经在跑的东西，拒哪几种列在共用约束 `.claude/agent-common.md`「不做」一节「执行前拒绝的写法」那一条（主 agent 同样被拒）。每次派发（包括续做）之后，立刻用 Bash 的 `run_in_background` 起看门狗 `bash research/scripts/watch.sh <这次派出的 agent id，逗号分隔>`（只填 agent 号：会话目录它自己找，阈值在 `research/scripts/watch.conf` 里配，不在命令行上手敲；调用里不再加 `&`、`nohup`、`disown`）：它每 4 分钟复检一次子 agent 的会话记录与本实例底下的长进程、每 15 秒读一次 hook 的检出记录，发现没超时的等待循环、一次工具调用超过 8 分钟、同一命令反复且输出不变、按模式找进程、10 分钟无动静、结束本轮而手里没有在跑的后台任务、进程写的文件 20 分钟不涨、本实例底下有没带 `SINGLEFS_HEAVY_TESTS` 前缀的重型测试进程（确认接着盯写 `--ack 进程:<pid>`）、子 agent 从派发起的运行时间跨过一个整小时（每个整点报一次，那个整点之后主 agent 给它发过消息就不报；一格多长在 `watch.conf` 的 `ask-every-minutes`），或 hook 检出本会话的问题命令，就退出并叫醒主 agent；子 agent 全部交回或被停也退出，没有告警时每 60 分钟定时回报一次。
 
 改了定义、共用约束或规则之后，当场给每个在跑的子 agent 发消息：写明改了哪一条、它手上哪一步要停掉或改做。
 
@@ -45,6 +45,8 @@
 
 同时在跑的重活（编译、测试、变异、产物、原型扫描）不止一个时，每份派发提示写一行「线程上限：N」，N = 整机核数 ÷ 同时在跑的重活数（向下取整，至少 1）；重活数变了，给在跑的 agent 发消息改 N。
 
+派崩溃验证员时给 54、55、57 号各自的内存上限，每道一个数，照 `.claude/agents/crash-verifier.md`「输入」一节给：54、57 号写明是量过的还是推的，55 号的不小于同时起的虚机数乘每台的内存。门禁 15、74 号在阶段里面经内存包装，要换它们的上限就在派发提示里给一个数，没给用 `research/scripts/replay.sh` 的 `REPLAY_MEMORY_CAP` 默认值。
+
 在不影响正确性的前提下写简练：只写对方干这件活要的东西——定义「输入」一节要的项、为哪几行岔路或哪个问题、定义与共用约束里没有的约束、交付什么交到哪；不复述定义与规则里已经写着的做法，不讲来龙去脉。事实、路径、行号、数与判据一个不少，嫌长就指到文件，不删内容。
 
 ## 什么时候派哪个 agent
@@ -56,7 +58,7 @@
 | 改 `crates/`：探索性的、条款没写全、要用户边看边拍板 | 主 agent 自己写、直接和用户对话 → 上一行的三方（代码轮）→ 提交时 `crash-verifier` |
 | 跑变异表、看抓到 / 无效 / 没红 | `mutation-triage` → 要补取样点、补断言的：`crates/` 那侧 `implementation-writer`（输入给分诊报告），`research/` 那侧 `experiment-designer` 写重跑登记 → `experiment-runner` |
 | 一个阶段任务结束：里程碑一步、一轮判决、一段实验、一批定义或脚本改完，这一批交回到齐、暂存之前 | `sweep` 写事实表（阶段同步第一段；输入给改动范围与做成的事，只用过、没留改动的也写；交回的事实表过了 `research/scripts/stale-candidates.py --check-facts`）→ 候选表按组切段，每段不超过约 200 行，一段派一个 `sweep` 逐行判（第二段；输入同样给做成的事）→ 交回齐了主 agent 跑 `stale-candidates.py --check-report 候选表 各份报告`，退出码 0 才往下，再**逐行全看**：每一条判定都对着载体今天的原文核一遍，不抽样、不按判定种类挑。判错的那一段重派，重派交回照样全看 → 主 agent 逐处判，自己改或交 `kb-scribe` → 主 agent 写 `research/prompts/<阶段>-sync.md`（格式见门禁 68 号文件头，68 号判形式）→ 下一行 |
-| 暂存之后、提交之前跑门禁 | 主 agent 用 `research/scripts/stage-mine.py` 暂存 → 派 `crash-verifier` 跑它那几道，命令带 `SINGLEFS_HEAVY_TESTS=commit`：这一批改了门禁 54 号在 `.claude/gate.d/stage-inputs.tsv` 登记的输入，就在 HEAD + 暂存区的 worktree 里跑那棵树里的 `SINGLEFS_HEAVY_TESTS=commit bash .claude/gate.d/54-layer0-replay.sh --full <它的根>`（层 0 全量只在这一步跑，判绿按输入哈希写全绿标记，整轮门禁的 54 号只核标记），55、57、59 同样带前缀 → `gate-triage` 带 `SINGLEFS_HEAVY_TESTS=commit` 跑 `gate.sh --staged` 并分诊（54、55、57、59 在 `gate.sh` 里照各自的复用判定走，它不直接调）；用户要求时两处都换成 `=user-request` |
+| 暂存之后、提交之前跑门禁 | 主 agent 用 `research/scripts/stage-mine.py` 暂存 → 派 `crash-verifier` 跑它那几道，命令带 `SINGLEFS_HEAVY_TESTS=commit`：这一批改了门禁 54 号在 `.claude/gate.d/stage-inputs.tsv` 登记的输入，就在 HEAD + 暂存区的 worktree 里跑那棵树里的 `SINGLEFS_HEAVY_TESTS=commit bash .claude/gate.d/54-layer0-replay.sh --full <它的根>`（整条经内存包装，照 `crash-verifier` 定义第 1b 步；层 0 全量只在这一步跑，判绿按输入哈希写全绿标记，整轮门禁的 54 号只核标记），55、57、59 同样带前缀 → `gate-triage` 带 `SINGLEFS_HEAVY_TESTS=commit` 跑 `gate.sh --staged` 并分诊（54、55、57、59 在 `gate.sh` 里照各自的复用判定走，它不直接调）；用户要求时两处都换成 `=user-request` |
 | 撤回一个数、改格式常量、新立一条判据 | `sweep` |
 | 定案之后写回 kb | `kb-scribe`（主 agent 给逐条规格）；规格动了某条分项的「**依据**」段时，同一份规格要带上那个实验页 `### 影响的决策` 表里对应那几行（门禁 75 号那条双向检查两侧要同时到位，而判一个实验撑不撑一条分项是主 agent 的活）；它翻了分项状态、33 号红（变异表锚点腐化）→ `experiment-runner` 只修锚点 |
 | 要建计数实验，或重跑已有实验 | 主 agent 写岔路单（`.claude/rules/three-way-inference.md`「交岔路时写岔路单」；不是为岔路建的实验写同格式的问题单）→ `experiment-designer` 写跑前登记或重跑登记 → 主 agent 删掉登记里待删的问法（有的话）→ `experiment-runner` → 每段交回主 agent 对着岔路单判够不够：一行开着的都不剩就停、交岔路表；续派时派发提示点名还差的那几行（续派闸 `.claude/hooks/runner-dispatch-guard.sh` 查这一句） |
diff --git a/.claude/rules/implementation-workflow.md b/.claude/rules/implementation-workflow.md
index 6e30a46..e4b85ec 100644
--- a/.claude/rules/implementation-workflow.md
+++ b/.claude/rules/implementation-workflow.md
@@ -45,7 +45,18 @@
 
 ## 重型测试只在提交时跑
 
-**重型测试**：层 0 全量（`.claude/gate.d/54-layer0-replay.sh` 与 `--test` 目标名含 `layer0` 的测试）、QEMU（55 号、`vm-bench.sh`）、herd7（57 号、`.claude/scripts/lkmm.sh`）、`crates` 变异整表（59 号）、全量 `cargo test`（`--all`、`--workspace`、`check.sh`）、整轮门禁（`gate.sh`、`gate-staged.sh`）、全部实验复跑（87 号）、E152 装置。`.claude/gate.d/` 下 54、55、57、59、87 之外的阶段不是重型，谁都能跑。
+**重型测试**，与 `.claude/hooks/heavy-test-guard.sh` 拒的逐类相同（判定在 `.claude/hooks/lib_heavy_tests.py` 的 `classify`）：
+
+- 层 0 全量：`.claude/gate.d/54-layer0-replay.sh`；会跑到名字含 `layer0` 的测试二进制的 `cargo test`——`--test` 的名字含 `layer0` 或通配命中它，或者不挑目标（不带 `--test` / `--lib` / `--bin` 这类、或带 `--tests` / `--all-targets`）而包里有这种测试目标（例 `cargo test -p singlefs-harness`）；直接执行名字含 `layer0` 的测试二进制（`<target 目录>/<profile>/deps/<名字>-<16 位十六进制哈希>`）。
+- QEMU：55 号、`qemu-system-*`、`research/scripts/vm-bench.sh`（`--selftest` 也算）。
+- herd7：57 号、`.claude/scripts/lkmm.sh`、`herd7`；这两样带什么参数都算，只取版本号的也算。
+- `crates` 变异整表：59 号、参数里有 `crates/mutations.tsv` 的 `research/scripts/mutate.sh`。
+- 全量 `cargo test`（`cargo t` 同）：带 `--all` / `--workspace`；在工作区根（仓根与 `research/`）上不带 `-p` 也不带 `--test` / `--lib` / `--bin` 这类挑目标选项的；不挑目标而包的范围是工作区全部成员的（在 `research/e7-index-bench/` 里裸跑也算）；`.claude/scripts/check.sh`。
+- 整轮门禁：`gate.sh`（带什么参数都算）、`research/scripts/gate-staged.sh`（`--selftest` 不算）。
+- 全部实验复跑：87 号。
+- E152 装置：`e152-file-system-benchmark`（直接起、或 `cargo run --bin` 它）、`research/scripts/e152-run.sh`。
+
+只认命令位置；`bash -c`、`capped.sh`、`run-with-memory-cap.sh`、`nice`、`timeout`、`env` 这类包装里面的同样算。命令位置上执行的脚本（`bash x.sh`、`./x.sh`、`source x`）闸读进去，逐行照同一个 `classify` 判，里面有一样就整条算（例：`.claude/scripts/fetch-deps.sh --check` 第 81 行调 `herd7 -version`，整条算 herd7）；门禁阶段与 `lib_heavy_tests.py` 的 `KNOWN_SCRIPT_LOCATIONS` 登记的仓内脚本按名字判、不读正文。`.claude/gate.d/` 下 54、55、57、59、87 之外的阶段不是重型，谁都能跑。
 
 | 场合 | 跑不跑 |
 |---|---|
```
