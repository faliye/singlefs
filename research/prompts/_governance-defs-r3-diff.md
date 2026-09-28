# governance-defs-r3 附录二：这一轮被判的改动全文 diff（基准 1c58cfa，HEAD bfc447e）

范围：7 份定义与共用约束，外加同一批一起改、被定义指着的规则、skill、钩子、门禁与脚本（当背景读）。

````diff
diff --git a/.claude/agent-common.md b/.claude/agent-common.md
index 257018e..d884d38 100644
--- a/.claude/agent-common.md
+++ b/.claude/agent-common.md
@@ -43,7 +43,7 @@
 - 不建、不改 `.claude/agents/`、`.claude/settings*.json`、`.claude/hooks/`、`.claude/rules/`、`.claude/singlefs-ai-sop/`，也不碰 `~/.claude/`。
 - 不读派发提示给的禁读清单里的文件。
 - 不编译 Rust、不跑 `gate.sh` 全量，除非定义明写要做。跑脚本、跑模型一律加 `nice -n 19`。
-- 重型测试（哪些算重型以 `.claude/rules/implementation-workflow.md`「重型测试只在提交时跑」开头那一句为准，逐条的判法在 `.claude/hooks/heavy-test-guard.sh` 文件头）只在提交代码时跑一次、或用户要求时跑；子 agent 一律不跑，只跑自己动到的测试二进制（`cargo test -p <crate> --test <自己的目标>`、`--lib`）、fmt / clippy / build 与 `.claude/gate.d/` 下 54、55、57、59、87 之外的阶段（15、74 号虽然也跑 cargo test，按轻阶段对待）。子 agent 跑 `cargo test` / `cargo run` 与直接执行编出来的二进制，一律经内存包装：`bash research/scripts/run-with-memory-cap.sh <上限> <命令>`（上限照 systemd 写法，派发提示没给的用 `4G`；退出码 250 是撞了这一条的上限，照实报主 agent，不自己调大重跑；251（scope 起不来）与 252（内存不够排不上）是那条命令一行都没跑，照实报、不绕开包装去裸跑；254 是被总上限挤掉、结果不算数，重跑一次，再挤掉就照实报），不经它的由 `heavy-test-guard.sh` 拒。重型测试里 `crash-verifier` 只跑 55、57、59 号那几道（54 号快档在 `gate.sh --staged` 里，全量由主 agent 跑），`gate-triage` 只跑 `gate.sh` 整轮与 87 号，都在提交时或用户要求时跑、命令带 `SINGLEFS_HEAVY_TESTS=commit`（用户要求时 `=user-request`）。项目 settings 里的 `.claude/hooks/heavy-test-guard.sh` 在执行前拒绝越出这些的命令；被拒了不换写法绕过去，在交回里写明要跑什么、为什么，交主 agent 去问用户。
+- 重型测试（哪些算重型以 `.claude/rules/implementation-workflow.md`「重型测试只在提交时跑」开头那一句为准，逐条的判法在 `.claude/hooks/heavy-test-guard.sh` 文件头）只在提交代码时跑一次、或用户要求时跑；子 agent 一律不跑，只跑自己动到的测试二进制（`cargo test -p <crate> --test <自己的目标>`、`--lib`）、fmt / clippy / build 与 `.claude/gate.d/` 下 54、55、57、59、87 之外的阶段（15、74 号虽然也跑 cargo test，按轻阶段对待）。子 agent 跑 `cargo test` / `cargo run` / `cargo bench` 与直接执行编出来的二进制，一律经内存包装：`bash research/scripts/run-with-memory-cap.sh <上限> <命令>`（上限照 systemd 写法，派发提示没给的用 `4G`；退出码 250 是撞了这一条的上限，照实报主 agent，不自己调大重跑；251（scope 起不来）与 252（内存不够排不上）是那条命令一行都没跑，照实报、不绕开包装去裸跑；253 是超过限时（`RUN_WITH_MEMORY_CAP_TIME_LIMIT`），照实报；254 是被总上限挤掉、结果不算数，重跑一次，再挤掉就照实报；2 是包装的用法写错，改写法再跑；退出码表以 `research/scripts/run-with-memory-cap.sh` 文件头为准），不经它的由 `heavy-test-guard.sh` 拒。重型测试里 `crash-verifier` 只跑 55、57、59 号那几道（54 号快档在 `gate.sh --staged` 里，全量由主 agent 跑），`gate-triage` 只跑 `gate.sh` 整轮与 87 号，都在提交时或用户要求时跑、命令带 `SINGLEFS_HEAVY_TESTS=commit`（用户要求时 `=user-request`）。项目 settings 里的 `.claude/hooks/heavy-test-guard.sh` 在执行前拒绝越出这些的命令；被拒了不换写法绕过去，在交回里写明要跑什么、为什么，交主 agent 去问用户。
 - 写进脚本文件、放进 `python3` 的 subprocess、`make` 这类间接起法同样算跑：自己写的验证链里不许出现重型测试的任何一步，名字含 `layer0` 的测试二进制也不许。
 - 要停自己起的一条链（脚本、后台任务）时，先列出它的整棵子进程（`ps -o pid,ppid,args --ppid <pid>` 逐层往下），从最底层起逐个 `python3 .claude/singlefs-ai-sop/scripts/proc.py stop <pid>`（`proc.py` 只停给定的那一个，不带子进程），停完再列一遍，确认一个不剩；不许留下正在跑的那一步「让它跑完」。
 - 要编译、跑门禁阶段、跑变异或产物的，开跑前 `ps -o pid,args -u "$(id -u)"` 看负载：有性能测量在跑（`qemu-system`、`vm-bench.sh`、`e152-file-system-benchmark`、`fio`）就报「在等 pid …（命令）」停下；只有别的 `cargo`、`gate.sh` 在跑的，加 `nice -n 19` 照常跑（cargo 自己排队拿文件锁），报告里记下 `ps` 看到了什么、等锁等了多久。定义另有更严要求的照定义。
diff --git a/.claude/agents/crash-verifier.md b/.claude/agents/crash-verifier.md
index 1aa4cf6..9075233 100644
--- a/.claude/agents/crash-verifier.md
+++ b/.claude/agents/crash-verifier.md
@@ -21,7 +21,7 @@ omitClaudeMd: true
 
 ## 做什么
 
-1. 每个阶段开跑前各照共用约束「不做」一节看一次负载。54 号不归你：快档在 `gate.sh --staged` 的 HEAD + 暂存区树上跑才算得对全绿标记的键（在主工作区跑，工作区与暂存区不同就判红），全量由主 agent 在 worktree 里跑。55、57、59 在主工作区跑，判的是工作区那一份：每个阶段开跑前取那一道自己的输入：55、59 号是 `.claude/gate.d/stage-inputs.tsv` 里它那一行的路径，57 号是 `litmus crates .claude/scripts/lkmm.sh`，三道都另加它自己的阶段脚本 `.claude/gate.d/<文件>`；跑 `git diff --quiet -- <这些路径>`（退 0 才说明工作区与暂存区在这一道的输入上相同）与 `git ls-files --others --exclude-standard -- <这些路径>`（cargo 会把未跟踪的 `src/bin`、`tests` 文件认作目标，要没有输出）。两样有一样不过就不跑那一道，停下报告「工作区与暂存区不同，判的不是要提交的那一批」，原样贴两条命令的输出交主 agent；主 agent 照「一轮怎么开、怎么收」第 9 条找改那几条路径的会话协商，等它们暂存、提交或撤掉再派。
+1. 每个阶段开跑前各照共用约束「不做」一节看一次负载。54 号不归你：快档在 `gate.sh --staged` 的 HEAD + 暂存区树上跑才算得对全绿标记的键（在主工作区跑，工作区与暂存区不同就判红），全量由主 agent 在 worktree 里跑。55、57、59 在主工作区跑，判的是工作区那一份：每个阶段开跑前取那一道自己的输入：55、59 号是 `.claude/gate.d/stage-inputs.tsv` 里它那一行的路径，57 号是 `litmus crates .claude/scripts/lkmm.sh`，三道都另加它自己的阶段脚本 `.claude/gate.d/<文件>`；跑 `git diff --quiet -- <这些路径>`（退 0 才说明工作区与暂存区在这一道的输入上相同）与 `git ls-files --others --exclude-standard -- crates litmus .lkmm-static-only`（cargo 会把未跟踪的 `src/bin`、`tests` 文件认作目标，仓根有 `.lkmm-static-only` 时 57 号只跑静态那几层，要没有输出；别处的未跟踪文件不挡）。两样有一样不过就不跑那一道，停下报告「工作区与暂存区不同，判的不是要提交的那一批」，原样贴两条命令的输出交主 agent；主 agent 照「一轮怎么开、怎么收」第 9 条找改那几条路径的会话协商，等它们暂存、提交或撤掉再派。
 2. 只在提交时（或主 agent 转达用户要求时）跑你那几道，不跑 `gate.sh` 整轮与全量 `cargo test`（`.claude/hooks/heavy-test-guard.sh` 拒）。按共用约束 `.claude/agent-common.md`「门禁」一节从阶段归属表取登记给你的阶段，一次只跑一个；55、57、59 号命令带输入给的那个前缀：`SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/<文件>`，其余登记给你的轻阶段不带；每个记开始与结束时刻（`date -u`）和退出码。单个阶段超过 Bash 单次上限就后台跑，退出码写进文件（`{ SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/<文件>; echo "exit=$?"; } > <草稿目录>/<阶段>.log 2>&1`；退出码只认日志里的 `exit=` 那一行，不认起后台的那条命令自己的 `$?`），结束后读输出。别的会话同时在跑 cargo 时，编译会卡在「Blocking waiting for file lock」，照实记等了多久，不算这个阶段的耗时。
 3. 每个阶段原样抄它的「✓」行，或「✗」行与紧跟的「→」；退出码 77 记「本次未跑」，不记通过。
 4. 计数行原样抄：崩溃状态数、恢复结果、与 E142（第一个事务的干跑） 产物或闭式比对的那几行。输出里读不到判定行的阶段记「作废」，不记通过。
@@ -34,7 +34,7 @@ omitClaudeMd: true
 
 ## 产出
 
-- 一张表：阶段 / 退出码（0、非 0、77）/ 耗时 / 原样的 ✓，或 ✗ 与 → / 计数行；末尾「没做什么」。
+- 一张表：阶段 / 退出码（0、非 0、77）/ 耗时 / 原样的 ✓，或 ✗ 与 → / 计数行 / 日志路径；末尾「没做什么」。每个阶段的整份输出都写进 `<草稿目录>/<阶段>.log`（前台跑也写），59 号那一份是变异分诊员要的输入。
 
 ## 没做什么（固定会有的）
 
diff --git a/.claude/agents/experiment-runner.md b/.claude/agents/experiment-runner.md
index 99c8dd4..33c5b48 100644
--- a/.claude/agents/experiment-runner.md
+++ b/.claude/agents/experiment-runner.md
@@ -24,12 +24,12 @@ omitClaudeMd: true
 ## 做什么
 
 1. 开跑前照共用约束「不做」一节看负载。登记第六节里有耗时类的量时，跑产物之前连别的 `cargo`、`gate.sh` 也要等。
-2. 写 `research/e7-index-bench/src/bin/e<号>_<英文名>.rs`（源文件、变异表与 bin 名一律用跑前登记 `## 一、问题` 第一行定的英文名：源文件与变异表写蛇形 `e<号>_<英文名>`；`research/e7-index-bench` 的 `[[bin]]` 的 `name` 写连字符 `e<号>-<英文名>`，入库装置不写 `[[bin]]`、bin 名就是文件名 `e<号>_<英文名>`；中文简称只用在 kb 页），在 `research/e7-index-bench/Cargo.toml` 末尾追加它的 `[[bin]]`（⚠️ **跑前登记把装置写死成「入库装置」时改写 `crates/singlefs-harness/src/bin/e<号>_<英文名>.rs`**：岔路问的是「`crates/` 今天这份代码的性质」时，另写一份独立模型答不了，写范围闸按 `crates/singlefs-harness/src/bin/e*.rs` 放行。那种 bin 只读、只驱动与观测，不许改 `crates/singlefs-core` 与 `crates/singlefs-checker`；变异表改追加进 `crates/mutations.tsv` 末尾、归门禁 59 号跑。**另有三条只对入库装置生效**：① **不许从 `crates/` 里引决定工作量、几何或判据门槛的常量**——这类参数在装置里写本地常量，值抄自 kb 的登记位并注明文件与行号，另加一条断言拿它与实现侧同名常量回比；引了就等于装置与实现共用一个数，两边一起错而逐格相等。这一条挡不住「kb 与实现一起取错」。② **写了入库装置就要让它进代码轮判决的点名清单**：门禁 56 号的正则罩得到 `crates/*/src/bin/*.rs`，不点名那一道会红，而那条义务落在跑整轮门禁的一方身上、不落在你身上——交回时写明你新建了哪个 bin。③ **什么算「入库装置」以跑前登记里写死的那一句为准**；登记里没写死就走 `research/` 那一支，别自己判）（`research/e7-index-bench` 的 `name` 用连字符，与 `replay.sh` 登记行里的 bin 名一致；只追加，不改别人的）：纯算术的计数模型、单测、钉绝对值的断言；取样点覆盖登记里每一个会跑到的取值。写完跑 `bash .claude/scripts/naming-lint.sh`，新文件里的缩写与单字母名在这一步改完再往下走，不留给后面的门禁阶段。
+2. 写 `research/e7-index-bench/src/bin/e<号>_<英文名>.rs`（源文件、变异表与 bin 名一律用跑前登记 `## 一、问题` 第一行定的英文名：源文件与变异表写蛇形 `e<号>_<英文名>`；`research/e7-index-bench` 的 `[[bin]]` 的 `name` 全写连字符 `e<号>-<英文名里的下划线换成连字符>`，入库装置不写 `[[bin]]`、bin 名就是文件名 `e<号>_<英文名>`；中文简称只用在 kb 页），在 `research/e7-index-bench/Cargo.toml` 末尾追加它的 `[[bin]]`（⚠️ **跑前登记把装置写死成「入库装置」时改写 `crates/singlefs-harness/src/bin/e<号>_<英文名>.rs`**：岔路问的是「`crates/` 今天这份代码的性质」时，另写一份独立模型答不了，写范围闸按 `crates/singlefs-harness/src/bin/e*.rs` 放行。那种 bin 只读、只驱动与观测，不许改 `crates/singlefs-core` 与 `crates/singlefs-checker`；变异表改追加进 `crates/mutations.tsv` 末尾、归门禁 59 号跑。**另有三条只对入库装置生效**：① **不许从 `crates/` 里引决定工作量、几何或判据门槛的常量**——这类参数在装置里写本地常量，值抄自 kb 的登记位并注明文件与行号，另加一条断言拿它与实现侧同名常量回比；引了就等于装置与实现共用一个数，两边一起错而逐格相等。这一条挡不住「kb 与实现一起取错」。② **写了入库装置就要让它进代码轮判决的点名清单**：门禁 56 号的正则罩得到 `crates/*/src/bin/*.rs`，不点名那一道会红，而那条义务落在跑整轮门禁的一方身上、不落在你身上——交回时写明你新建了哪个 bin。③ **什么算「入库装置」以跑前登记里写死的那一句为准**；登记里没写死就走 `research/` 那一支，别自己判）（`research/e7-index-bench` 的 `name` 用连字符，与 `replay.sh` 登记行里的 bin 名一致；只追加，不改别人的）：纯算术的计数模型、单测、钉绝对值的断言；取样点覆盖登记里每一个会跑到的取值。写完跑 `bash .claude/scripts/naming-lint.sh`，新文件里的缩写与单字母名在这一步改完再往下走，不留给后面的门禁阶段。
 3. 入库装置那一支不写 research 的变异表、不跑 `mutate.sh`（它只编得到 `research/` 下的 bin，整表跑 `crates/mutations.tsv` 又是重型）：变异行照 `crates/mutations.tsv` 第 1 行表头的六段格式追加，报告里列出追加的变异名、三个数写「待提交时 59 号」。`research/` 那一支写变异表 `research/mutations/e<号>_<英文名>.tsv`，在 `research/` 下跑完整张表：`cd research && nice -n 19 bash scripts/mutate.sh <bin 名> e7-index-bench/src/bin/<源文件> mutations/<变异表>`（两个路径都相对 `research/`；bin 名以 `research/e7-index-bench/Cargo.toml` 里 `[[bin]]` 的 `name` 为准，没有显式声明的取文件名去掉 `.rs`；报「基线就是红的」时先查是不是在仓根下跑、bin 名是不是写错）（收尾要有「已还原，基线仍全绿」）；没红的逐条按 `mutation-sampling.md` 分类。
 3b. 登记里的停机条款（第十一节里标「停机」的，通常是装置与 `crates/` 实现逐项对拍）在跑产物之前做，照原文真跑，不许只推；做不了就停下交回，不跑产物、不写实验页。登记的量一次做不完的，做完的那一段照常交，没做的逐条列进报告，实验页与索引行的状态写「部分已跑」，不写「已跑」。岔路单每一行都够判了（登记第十一节的够判停机），剩下的量不跑：实验页逐条标「够判后未跑」、写明各自对应哪一行，状态写「已跑（够判）」；够不够判由你按登记写的够判条件报，续不续由主 agent 定。
-4. 跑产物进 `research/results/`；跑前删旧输出，跑完认本轮才有的完成标记。重跑已有实验时，`research/results/` 里已有的产物不覆盖：与它逐字节一致就不新存；对不上就按今天的日期另存一份，实验页两份都点名、写明对不上，交主 agent 定哪份承重。生成产物用的二进制编在 `research/target/` 下，用最后一次源码改动编出来；生成产物与第 5 步跑 `replay.sh` 时都不设 `CARGO_TARGET_DIR`；草稿目录里的 target 只拿来迭代。
+4. 跑产物进 `research/results/`；跑前删旧输出，跑完认本轮才有的完成标记。重跑已有实验时，`research/results/` 里已有的产物不覆盖：与它逐字节一致就不新存；对不上就按今天的日期另存一份，实验页两份都点名、写明对不上，交主 agent 定哪份承重。生成产物用的二进制编在 `research/target/` 下（入库装置那一支编在仓根的 `target/` 下），用最后一次源码改动编出来；生成产物与第 5 步跑 `replay.sh` 时都不设 `CARGO_TARGET_DIR`；草稿目录里的 target 只拿来迭代。
 4b. 产物出来之后先自查齐不齐：按登记的「臂 × 历史 × 几何」数一遍产物里每一类结果行的条数与字段，缺行、缺字段（例如某一族没有臂字段）、某一族一次都没造出被测形状，都列进报告，不许只看末行完成标记。报告里每个判据按每条臂全列判定，不挑「突出发现」报——对某条候选不利的那一半同样要写。分族、分臂的计数用一条命令从产物里数，报告与实验页贴那条命令和它的原样输出，不用文字重写族名单；给「这几族为什么是 0」写解释之前，先把那几族的产物行贴出来，确认它们真是 0。
-5. 在 `research/scripts/replay.sh` 里登记复跑，跑一次 `bash research/scripts/run-with-memory-cap.sh 4G bash research/scripts/replay.sh E<号>`（`replay.sh` 里执行编出来的二进制，不经内存包装会被 `heavy-test-guard.sh` 拒） 确认逐字节一致。送进虚机跑的实验（`vm-bench.sh`，含 `--selftest`）与 E152（按里程碑对比六家文件系统的文件性能） 装置是重型测试，你不跑：装置、登记与复跑命令写好就停，在交回里写明要跑什么、为什么，由主 agent 问用户（`.claude/kb/vm-harness.md`；共用约束「不做」一节里重型测试那一条）。装置要起子进程、读它的输出时，先把输出读完再转打，计时按登记写的进程与时钟取（`.claude/kb/vm-harness.md`「计时不在转发输出的循环里打时间戳」）；共享 `gate.sh` 的「转发计时」阶段查读子进程输出的循环里一边取时间一边输出。
+5. 在 `research/scripts/replay.sh` 里登记复跑（`research/` 那一支写一行登记；入库装置照 E156、E158 的先例另写一个 `driver_e<号>` 驱动函数，登记行写 `E<号>|@driver_e<号>||<产物>|exact`），跑一次 `bash research/scripts/run-with-memory-cap.sh 4G bash research/scripts/replay.sh E<号>`（`replay.sh` 里执行编出来的二进制，不经内存包装会被 `heavy-test-guard.sh` 拒） 确认逐字节一致。送进虚机跑的实验（`vm-bench.sh`，含 `--selftest`）与 E152（按里程碑对比六家文件系统的文件性能） 装置是重型测试，你不跑：装置、登记与复跑命令写好就停，在交回里写明要跑什么、为什么，由主 agent 问用户（`.claude/kb/vm-harness.md`；共用约束「不做」一节里重型测试那一条）。装置要起子进程、读它的输出时，先把输出读完再转打，计时按登记写的进程与时钟取（`.claude/kb/vm-harness.md`「计时不在转发输出的循环里打时间戳」）；共享 `gate.sh` 的「转发计时」阶段查读子进程输出的循环里一边取时间一边输出。
 6. 跑阶段归属表登记给你的阶段（共用约束 `.claude/agent-common.md`「门禁」一节）；其中要编译或复跑整张表的，开跑前照共用约束看负载。15 号（编整个 research 工作区）与 87 号（复跑全部已入库实验）不归你，收尾由 `gate-triage` 统一跑；你只跑第 5 步自己那一个实验的 `replay.sh`。
 7. 主 agent 要写实验页时：`.claude/kb/experiments/<号>-<简称>.md`，结果整行抄自产物并带文件名，写复跑命令、口径、它答不了的；索引行进 `.claude/kb/experiments.md`。
 7b. 实验页的 `### 影响的决策` 一节照第 7 步写实验页的规则写，形态与判据不在这份定义里复述。这一步你要做的三件：① 要写支撑或推翻时，先 `grep -n 'E<号>（' .claude/kb/decisions/<那条决策的文件>` 看那条分项的「**依据**」段引没引这个实验——引了才写支撑或推翻，没引就写备料，并把「这一格该不该升成支撑」列进报告交主 agent，**不自己去改决策文件**（写范围闸也不放行）。② 那条决策还在 `.claude/decision-links-pending` 里时（那份清单只减不增；清单里一条都没有时 ② 不适用，① ③ 照做）不查依据段（门禁 75 号对清单里的决策整段跳过双向检查，那些决策还没有依据段），写决策级一行，并把它列进报告。③ 重跑已有实验、换了产物、加了历史条目之后，表里每一行都重新回看一遍。
diff --git a/.claude/agents/kb-scribe.md b/.claude/agents/kb-scribe.md
index 4b8e03f..a17372f 100644
--- a/.claude/agents/kb-scribe.md
+++ b/.claude/agents/kb-scribe.md
@@ -17,7 +17,7 @@ omitClaudeMd: true
 
 - 逐条改动规格：文件、旧串（原文整行）、新串、依据（判决或用户定案的出处）。
 - 要不要记决策变更史：标题整行、日期、改前、改后、依据各一句（快查两行由主 agent 给定，不由你概括；标题里「（其N）」那一段由你在写之前照当月文件现取，其余逐字照给）。
-- 分项翻状态的：决策号与分项号；另给这几样，缺了门禁必红而你不能自己补：分项那一行收尾的「**状态：已定。**」或「**状态：未定。**」（20 号），翻成未定的判两句（31 号两把尺都要，缺一句就红）：改不改第一个事务的字节、动不动格式，决策文件标题行的状态改成什么（已定 / 半定（N 项未定）/ 待定（N 项未定），N 写阿拉伯数字或汉字数字都认）；`.claude/kb/decisions.md` 状态列由第 3 步的 `21-decision-items-sync.sh --write` 重生成，规格只给预期的分项计数供回读核对，不手写。变更史标题与快查里的分项标签写翻完之后的：变更史用今天的编号（`.claude/kb/decisions-history.md` 第 8 行），写成翻之前的，第 3 步的 `relabel-item.py` 也会把它改掉。
+- 分项翻状态的：决策号与分项号；另给这几样，缺了门禁必红而你不能自己补：分项那一行收尾的「**状态：已定。**」或「**状态：未定。**」（20 号），翻成未定的判两句（31 号两把尺都要，缺一句就红）：改不改第一个事务的字节、动不动格式，决策文件标题行的状态改成什么（已定 / 半定（N 项未定）/ 待定（N 项未定），N 写阿拉伯数字或一到十的汉字数字，20、75 号都认）；`.claude/kb/decisions.md` 状态列由第 3 步的 `21-decision-items-sync.sh --write` 重生成，规格只给预期的分项计数供回读核对，不手写。变更史标题与快查里的分项标签写翻完之后的：变更史用今天的编号（`.claude/kb/decisions-history.md` 第 8 行），写成翻之前的，第 3 步的 `relabel-item.py` 也会把它改掉。
 - 欠账表要加或还的行：也写成逐条规格（加：新行与插在哪一整行之后；还：被还的那一行，与移进已还清表的新行、插在哪一整行之后）。
 - 写决策正文（新立分项、或改一条分项的定案 / 射程 / 依据）的：规格要按 `.claude/rules/format-evolution.md` 那一节的形态逐块给全（四块、索引行、未定项的字节判决），形态本身照那一节，这份定义不抄第二份；缺哪一块就停下报告，不自己补。
 - **规格改了某条分项的「**依据**」段时，同一份规格还要带上那个实验页 `### 影响的决策` 表里对应那几行的改动**（旧串、新串，一条分项一行）：门禁 75 号那条双向检查两侧要同时到位，而判关系是主 agent 的活、不是你的。规格里没带那几行就停下报告，列出缺哪几行。
@@ -29,7 +29,7 @@ omitClaudeMd: true
 2. 决策变更史：原文写进当月的 `.claude/kb/decisions-history/<年-月>.md`，标题下两行快查照规格；新条目放在哪、同一天多条怎么编「（其N）」，照当月文件与 `.claude/kb/decisions-history.md` 的文件头，不另定。然后先跑不带 `--write` 的 `bash .claude/gate.d/49-history-brief.sh`：它列的「快查·… 还没写」里有不是这一轮的条目，就停在 `--write` 之前交回，不让 `--write` 往那些条目里写「（待补）」；都是这一轮的才跑 `--write`。跑前跑后各 `git diff -- .claude/kb/decisions-history.md` 一次，只核新增的行：每条决策的卡片只留最近 3 次，被挤掉的旧行是按设计；新增的行里有这一轮之外的条目就停下交回，不自己收拾。
 3. 分项翻状态：`python3 research/scripts/relabel-item.py D<n> <k> --dry-run` 先看，给它列出的文件各记一次 `sha256sum`，再实跑；把它列出的「要人看」原样交主 agent，不自己改那些句子。它改写了 `research/**/*.rs` 的，加跑 33 号，并把改到的实验源码逐个列给主 agent；改写了 `crates/**/*.rs` 的（实现的地盘，要走代码轮），同样逐个列给主 agent；预演列出的文件里带别的会话没提交改动的（`git status --porcelain` 里有它），也逐个列给主 agent。然后 `bash .claude/gate.d/21-decision-items-sync.sh --write`。
 3b. 规格里决策那几处与实验页那几行是一批，一起写、一起回读，不分两次。写完跑 75 号：它报「不对称」时看点名的那一对在不在这一份规格里——在，就是规格少给了一边，停下报告并写明缺哪一行；不在，按「不是这一轮的不修」照写。**判一个实验撑不撑一条分项不在你的活里**，规格没写的关系一个字不加。
-每次写入之后，写入后钩子 `.claude/hooks/kb-scribe-followup.sh` 按 `.claude/hooks/kb-scribe-followups.tsv` 跑几道阶段、把红的 ✗ 与 → 交回给你：红的是这个流程下一步会消掉的（49 号在 `--write` 之前、75 号在实验页那几行写之前），照流程往下走；其余的留到第 4 步一起判归属。
+每次写入之后，写入后钩子 `.claude/hooks/kb-scribe-followup.sh` 按 `.claude/hooks/kb-scribe-followups.tsv` 跑几道阶段、把红的 ✗ 与 → 交回给你：红的是这个流程下一步会消掉的（21 号在第 3 步的 `21-decision-items-sync.sh --write` 之前、30 号在变更史条目写之前、49 号在 `--write` 之前、75 号在实验页那几行写之前），照流程往下走；其余的留到第 4 步一起判归属。
 4. 跑阶段归属表登记给你的阶段各一次并贴结果（共用约束 `.claude/agent-common.md`「门禁」一节）；阶段数用那条 `awk` 接 `| wc -l` 数出来贴原样，不手数。红的判它是不是这一轮写出来的（看点名的文件与行在不在这一轮改动里）；不是这一轮的不修，照写。30 号报的改动行数含别的会话没提交的改动，照抄，不据此判归属。另跑一次 `bash .claude/singlefs-ai-sop/scripts/doc-lint.sh .` 贴末行；红在这一轮写的句子上，停下交主 agent 改规格，不自己改句子。
 5. 收尾再记一次每个被改文件的 `sha256sum`，报告里列全部改过的文件与开工时、收尾时两次哈希。
 
diff --git a/.claude/agents/mutation-triage.md b/.claude/agents/mutation-triage.md
index 9b81c19..d680abb 100644
--- a/.claude/agents/mutation-triage.md
+++ b/.claude/agents/mutation-triage.md
@@ -24,8 +24,8 @@ omitClaudeMd: true
 1. 开跑前照共用约束「不做」一节看负载。
 2. 先逐条做子串计数：原文在源文件里不是恰好一次的，列出来（第七类），这张表不跑。
 3. 在 `research/` 下跑：`cd research && nice -n 19 bash scripts/mutate.sh <bin 名> e7-index-bench/src/bin/<源文件> mutations/<变异表>`（路径相对 `research/`；报「基线就是红的」时先查是不是在仓根下跑）；表头写着「被测装置是 shell 探针」的，照表头写的复跑方式逐条跑，判据用表头那句，替换在草稿目录里那份探针的副本上做、跑副本，不动仓里的探针（要 dm / loop 设备才跑得起来的，写明没跑、为什么，交主 agent）；crates 那张表你不跑：整表复跑是重型测试，只在提交时由崩溃验证员跑门禁 59 号（`.claude/rules/implementation-workflow.md`「重型测试只在提交时跑」，`heavy-test-guard.sh` 会拒绝你跑它）；主 agent 给你那一次 59 号的输出路径，你从里面取点名的条目分类。
-4. 确认整张表跑完：`mutate.sh` 收尾要有「已还原，基线仍全绿」；59 号没有这句，看它收尾的「计数：」行里「共 N 条」与各栏加起来对得上，编不过的在「有变异无效」一栏，「没跑到」算进没红；对不上就是中途退出，这一次的数不算，照实报。
-5. 报抓到 / 无效 / 没红三个数（`mutate.sh` 没有三数汇总行，按每条结果行开头的符号数（`[ ]` 里是变异名，不是结局）：`✅` 抓到、`⏭` 无效、`❌` 没红；`💥`（测试进程没跑完）、`⚠️`（报了失败却一个测试名都没抓到）、`⏱`（超时）、`🧱`（内存撞顶）另列，几栏加起来要等于表的条数；`mutate.sh` 收尾那行「计数：」只报内存撞顶与超时两栏；59 号照它自己收尾的「计数：」行，那一行没有 `💥`、`⚠️` 两栏：测试进程没跑完的，输出里有编译错误记「无效」、没有记「没红」，两种都让整道判红），其余几栏不并进三个数：`⚠️`、`⏱`、`🧱` 不为 0 的整轮已判失败，`mutate.sh` 的 `💥` 不判失败，逐条列出交主 agent；与改动前的数比，「无效」变多要单列。
+4. 确认整张表跑完：`mutate.sh` 收尾要有「已还原，基线仍全绿」；59 号没有这句，看它收尾的「计数：」行里「共 N 条」与各栏加起来对得上，编不过的与测试进程被杀的在「无效」一栏（第 5 步那一句分得开两者），「没跑到」算进没红；对不上就是中途退出，这一次的数不算，照实报。
+5. 报抓到 / 无效 / 没红三个数（`mutate.sh` 没有三数汇总行，按每条结果行开头的符号数（`[ ]` 里是变异名，不是结局）：`✅` 抓到、`⏭` 无效、`❌` 没红；`💥`（测试进程没跑完）、`⚠️`（报了失败却一个测试名都没抓到）、`⏱`（超时）、`🧱`（内存撞顶）另列，几栏加起来要等于表的条数；`mutate.sh` 收尾那行「计数：」只报内存撞顶与超时两栏；59 号照它自己收尾的「计数：」行，那一行没有 `💥`、`⚠️` 两栏：测试进程没跑完的，输出里有一行 `error:` 开头（测试二进制被信号杀时 cargo 也打 `error: test failed`）或 `could not compile` 就记「无效」，否则记「没红」，两种都让整道判红；「无效」里尾巴带 `process didn't exit successfully` 的是进程被杀，不是替换文编不过；另外三栏「被总上限挤掉」「排不上没跑」「scope 起不来没跑」不为 0 也让整道判红，照原样另列），其余几栏不并进三个数：`⚠️`、`⏱`、`🧱` 不为 0 的整轮已判失败，`mutate.sh` 的 `💥` 不判失败，逐条列出交主 agent；与改动前的数比，「无效」变多要单列。
 6. 没红与无效的逐条按 `mutation-sampling.md` 分类；判「取样点不敏感」的，解出让两个式子跨过整数边界的那个输入；判「等价」的，写出在所有输入上同值的理由。
 
 ## 写范围
diff --git a/.claude/agents/three-way-verifier.md b/.claude/agents/three-way-verifier.md
index aee9446..d1b7d91 100644
--- a/.claude/agents/three-way-verifier.md
+++ b/.claude/agents/three-way-verifier.md
@@ -18,13 +18,13 @@ omitClaudeMd: true
 - 轮名、这一轮全部腿报告的路径（主 agent 确认都已交齐、腿不再写）。
 - 背景材料路径（用来识别误写成背景材料行号的引用）。
 - 云端腿交回里给的报告 `sha256sum`（有就给）。
-- 腿开工那一刻的快照路径：`sha256sum` 清单，罩这一轮被判的文件与材料点名的 kb 文件（`.claude/rules/implementation-workflow.md`「代码轮派腿之前记一份开工快照」；代码轮必给），腿跑着的时候被别的会话改过的，主 agent 另给倒推出的原样副本（`*.at-snapshot`）。没给快照的代码轮，停下要，不对主树核。没给快照的轮（设计轮）对主树核，腿引的行号与主树对不上时记「分不清：文件可能在腿开工之后被改过」（第 6 步的「分不清」一栏），不记 ✗。
+- 腿开工那一刻的快照路径：`sha256sum` 清单，罩这一轮被判的文件与材料点名的 kb 文件（`.claude/rules/implementation-workflow.md`「代码轮派腿之前记一份开工快照」；代码轮必给），腿跑着的时候被别的会话改过的，主 agent 另给倒推出的原样副本（`*.at-snapshot`）。没给快照的代码轮，停下要，不对主树核。没给快照的轮（设计轮）对主树核，腿引的行号与主树对不上时，拿腿开工时刻照第 2 步现查那个文件在腿开工之后有没有被改过：没改过记 ✗，改过或没给开工时刻记「分不清：文件可能在腿开工之后被改过」（第 6 步的「分不清」一栏）。
 - 腿开工时刻：第 2 步现查「快照清单里没有的文件」在腿开工之后有没有被改过要用；没给时，快照清单外的文件内容对不上记「分不清：文件可能在腿开工之后被改过」，不记 ✗。
 - 报告路径（形态 `research/prompts/<轮>-verifier-output.md`）、草稿目录。
 
 ## 做什么
 
-1. 判别力自证，先做：从待核的引用里挑一条，在草稿目录的副本里把它的行号加 1，按第 2 步核它，必须判 ✗；判不出就停下报告「核查方法不分辨」。
+1. 判别力自证，先做：从待核的引用里挑一条，在草稿目录的副本里把它的行号加 1，按第 2 步核它（挑一条文件在快照里、或腿开工之后没被改过的），必须判 ✗；判不出就停下报告「核查方法不分辨」。
 2. 每处「文件:行号 + 抄的原文」：先拿快照清单 `sha256sum -c` 核那个文件；对得上就到主树那一行（区间就取区间）比内容，对不上就只对主 agent 给的倒推副本核，两样都没有的记「分不清：文件可能在腿开工之后被改过」，不记 ✗；给了快照、而这个文件不在清单里的，对主树核，并用 `git log --since=<腿开工时刻> -- <文件>` 与 `git status --short -- <文件>` 现查它在腿开工之后有没有被改过，照实写，不一律记成「被改过」。对不上时再找原文实际在第几行；若那个行号在背景材料里正好是这句，写「误写成背景材料第 N 行，原文件实为第 M 行」，不只打 ✗。
 3. 每行引的产物：在产物文件里逐字找。
 4. 每条复跑命令：把腿的模型目录拷到草稿目录，在副本里跑（加 `nice -n 19`），比输出与报告里抄的、比 sha256；不在腿的原目录里跑。复跑命令带着指向路径的环境变量的，路径一并换到你的草稿目录；输出里嵌着临时路径的，按字段比，不按整份哈希判 ✗。
diff --git a/.claude/gate.d/10-kb-rot.sh b/.claude/gate.d/10-kb-rot.sh
index d1ddc51..8501692 100755
--- a/.claude/gate.d/10-kb-rot.sh
+++ b/.claude/gate.d/10-kb-rot.sh
@@ -22,11 +22,12 @@
 # 每段的成功行都报检查了多少项；本该有对象却一个都没扫到的，判红
 # （.claude/singlefs-ai-sop/rules/show-me-test.md「扫到 0 项也不是通过」）。
 #
-# 判别力：fixtures/10-kb-rot.sh/red 必须判红（悬空的实验号、invariants.md 丢了登记标记、欠账表一行都数不出）；
+# 判别力：fixtures/10-kb-rot.sh/red 必须判红（悬空的实验号、invariants.md 丢了登记标记、欠账表一行都数不出、第 4 段的悬空门禁号、路径与小节）；
 # green 必须判绿。
 set -uo pipefail
-# 门禁调用时把项目根作为 $1 传进来；单独跑时从脚本位置推。
-cd "${1:-$(dirname "$0")/../..}" || exit 2
+# 门禁调用时把项目根作为 $1 传进来；单独跑时从脚本位置推。共用库按脚本自己的目录找，在 cd 之前取成绝对路径。
+GATE_DIRECTORY="$(cd "$(dirname "$0")" && pwd)"
+cd "${1:-$GATE_DIRECTORY/../..}" || exit 2
 KB=.claude/kb
 fail=0
 say() { printf '  %s\n' "$*"; }
@@ -141,7 +142,7 @@ if [[ ! -f "$KB/checks-owed.md" ]]; then
 else
   # 开着与已还清的切法用共用读法 lib-owed.py（67、92、96 号同一份），不在这里再抄一份 awk
   chk_counts=""
-  if chk_counts=$(python3 - "$(cd "$(dirname "$0")" && pwd)/lib-owed.py" "$KB/checks-owed.md" <<'PY_OWED'
+  if chk_counts=$(python3 - "$GATE_DIRECTORY/lib-owed.py" "$KB/checks-owed.md" <<'PY_OWED'
 import importlib.util, sys
 spec = importlib.util.spec_from_file_location("lib_owed", sys.argv[1])
 lib_owed = importlib.util.module_from_spec(spec); spec.loader.exec_module(lib_owed)
@@ -166,12 +167,15 @@ fi
 
 echo "── 4. 治理文档里的指向 ──"
 governance_rc=0
-python3 "$(cd "$(dirname "$0")" && pwd)/lib-governance-refs.py" || governance_rc=$?
+python3 "$GATE_DIRECTORY/lib-governance-refs.py" || governance_rc=$?
 if [[ $governance_rc -eq 0 ]]; then
   ok "治理文档里的门禁号、路径与「小节」都指得到"
-elif [[ $governance_rc -eq 2 ]]; then
+elif [[ $governance_rc -eq 3 ]]; then
   bad "一份治理文档都没扫到——这一段没有对象可判"
   howto "确认门禁是在仓库根上跑的；治理文档搬了家的话，改 lib-governance-refs.py 的 CARRIER_PATTERNS。"
+elif [[ $governance_rc -ne 1 ]]; then
+  bad "lib-governance-refs.py 没跑成（退出码 $governance_rc）——这一段没判"
+  howto "单独跑 python3 $GATE_DIRECTORY/lib-governance-refs.py 看它报什么错；没跑成就是没判，不许当成判过了。"
 else
   bad "治理文档里有指不到的指向（上面逐条列出）"
   howto "门禁号：阶段被删或收归上游的，改成共享 gate.sh 里那一道的名字（「链接指向」这类）或现存的号；" \
diff --git a/.claude/gate.d/20-kb-shape.sh b/.claude/gate.d/20-kb-shape.sh
index 51ed117..12f07c1 100755
--- a/.claude/gate.d/20-kb-shape.sh
+++ b/.claude/gate.d/20-kb-shape.sh
@@ -43,7 +43,8 @@ echo
 echo "── 3. 文件内自指链接 ──"
 # 扫 kb 下每一份 .md：链接目标（去掉 #锚点）按这份文件自己的目录解析之后就是它自己，判红。
 # 历史类文件（`*-history.md`、`decisions-history/` 下的月份文件）整份跳过：它们逐字记着当时的原文，
-# 里面抄录的链接改了就成假话（判据同 `.claude/rules/path-moves.md`「改一个全仓术语：正文之外还有五处会红」里「历史类文件同样换名」那一段：换了就成假话的那一句留原样）。
+# 里面抄录的链接多半是当时的原文，这一道分不出哪一句换了会成假话，所以整份不判；这是这一道的射程，
+# 不是豁免：`.claude/rules/path-moves.md`「改一个全仓术语：正文之外还有五处会红」里「历史类文件同样换名」那一段要逐句判，历史类文件里的自链靠人看。
 self_link_report=$(python3 - "$KB" <<'PY'
 import os, re, sys, glob
 kb = sys.argv[1]
diff --git a/.claude/gate.d/54-layer0-replay.sh b/.claude/gate.d/54-layer0-replay.sh
index 2c76264..cc962a6 100755
--- a/.claude/gate.d/54-layer0-replay.sh
+++ b/.claude/gate.d/54-layer0-replay.sh
@@ -227,10 +227,10 @@ report_manifest_differences() {
 # print_staged_worktree_full_commands：出路里建「HEAD + 暂存区」worktree、在里面用那棵树里的 54 号跑 --full 的命令。
 # 建法与共享 gate.sh --staged 相同：worktree add --detach HEAD，再 apply --index 暂存区的 diff（diff 为空就不套）。
 print_staged_worktree_full_commands() {
-  echo '                在项目根、暂存之后，把下面三行命令放进同一次 Bash 调用（三行共用 layer0_full_base 这个变量，分开跑它就是空的；这次调用的退出码就是 --full 的）：'
-  echo '                layer0_full_base="$(mktemp -d)"; git diff --cached --binary > "$layer0_full_base/staged.patch"'
-  echo '                git worktree add --detach "$layer0_full_base/tree" HEAD && { [ ! -s "$layer0_full_base/staged.patch" ] || git -C "$layer0_full_base/tree" apply --index "$layer0_full_base/staged.patch"; }'
-  echo '                SINGLEFS_HEAVY_TESTS=commit bash research/scripts/run-with-memory-cap.sh 16G bash "$layer0_full_base/tree/.claude/gate.d/54-layer0-replay.sh" --full "$layer0_full_base/tree"; layer0_full_rc=$?; git worktree remove --force "$layer0_full_base/tree"; rm -rf "${layer0_full_base:?}"; ( exit "$layer0_full_rc" )'
+  echo '                在项目根、暂存之后，把下面三行命令放进同一次 Bash 调用（三行共用 layer0_full_base 这个变量，分开跑它就是空的；这次调用的退出码是经内存包装的那条 --full 命令的，250–254 是包装自己的结局）：'
+  echo '                layer0_tree_ready=; layer0_full_base="$(mktemp -d)"; git diff --cached --binary > "$layer0_full_base/staged.patch"'
+  echo '                git worktree add --detach "$layer0_full_base/tree" HEAD && { [ ! -s "$layer0_full_base/staged.patch" ] || git -C "$layer0_full_base/tree" apply --index "$layer0_full_base/staged.patch"; } && layer0_tree_ready=1'
+  echo '                if [ "$layer0_tree_ready" = 1 ]; then SINGLEFS_HEAVY_TESTS=commit bash research/scripts/run-with-memory-cap.sh 16G bash "$layer0_full_base/tree/.claude/gate.d/54-layer0-replay.sh" --full "$layer0_full_base/tree"; layer0_full_rc=$?; else echo "worktree 没建好或暂存区的 diff 套不上，--full 没跑"; layer0_full_rc=1; fi; git worktree remove --force "$layer0_full_base/tree" 2>/dev/null; rm -rf "${layer0_full_base:?}"; ( exit "$layer0_full_rc" )'
 }
 
 # report_newest_other_marker <这一次的清单>：这批输入没有自己那一格时，拿 common-dir 里最近写的一格与这一次比，列出不同的文件；
diff --git a/.claude/gate.d/75-decision-experiment-links.sh b/.claude/gate.d/75-decision-experiment-links.sh
index d3451e4..a91a506 100755
--- a/.claude/gate.d/75-decision-experiment-links.sh
+++ b/.claude/gate.d/75-decision-experiment-links.sh
@@ -223,7 +223,7 @@ for decision, info in sorted(decisions.items()):
         continue
     where = info['path']
     # 半定 / 待定的决策，20 号要标题里写明未定几项（`—— 半定（一项未定）`），只许这一种括注
-    if not re.match(r'^## D\d+ \S.*? —— (已定|半定|待定)(（[0-9一二两三四五六七八九十]+[项条]未定）)?\s*$', info['title']):
+    if not re.match(r'^## D\d+ \S.*? —— (已定|半定|待定)(（[0-9一二两三四五六七八九十]+\s*[项条]未定）)?\s*$', info['title']):
         bad('瘦身形态', f'{where}：首行写成「## D{decision} 简称 —— 状态」，状态后面除了半定 / 待定要写的「（N 项未定）」不带别的括注——「{info["title"][:40]}」')
     for (kind, item_number), heading, labels, cited, basis_text in info['shapes']:
         if DATE.search(heading):
diff --git a/.claude/gate.d/90-term-renames.sh b/.claude/gate.d/90-term-renames.sh
index 6513160..55b8663 100755
--- a/.claude/gate.d/90-term-renames.sh
+++ b/.claude/gate.d/90-term-renames.sh
@@ -19,6 +19,6 @@ echo "               产物分两类，别一把梭：**跑得出来的**（装
 echo "               换完跑 cargo test --workspace 与 bash research/scripts/replay.sh，同一份源码要仍然吐出逐字节相同的产物——"
 echo "               那是重新生成，不是改证据；**跑不出来的**（已归档、要虚机或真设备、别人的产物）一个字节都不许动，"
 echo "               连同引它的正文整段留旧名（evidence-discipline「原样保存的证据不许事后改」）。"
-echo "               确实该留旧名的（别家术语、冻结证据目录、历史类文件里换了就成假话的那一句、引文块、对照表自己），逐文件"
-echo "               登记进 .claude/term-rename-exempt 并写明为什么。"
+echo "               确实该留旧名的（别家术语、冻结证据目录、历史类文件里换了就成假话的那一句、引文块、对照表自己），"
+echo "               登记进 .claude/term-rename-exempt 并写明为什么；历史类文件逐文件登记，不整个目录豁免。"
 exit 1
diff --git a/.claude/gate.d/fixtures/10-kb-rot.sh/green/.claude/agents/sample.md b/.claude/gate.d/fixtures/10-kb-rot.sh/green/.claude/agents/sample.md
index 52c6ecc..b745874 100644
--- a/.claude/gate.d/fixtures/10-kb-rot.sh/green/.claude/agents/sample.md
+++ b/.claude/gate.d/fixtures/10-kb-rot.sh/green/.claude/agents/sample.md
@@ -6,4 +6,5 @@ description: 门禁 10 号第 4 段的判别力样本，不派发。
 
 开工先读：`.claude/kb/invariants.md`「历史版本」；改 `.claude/kb/decisions.md` 之前先看 `kb/invariants.md`。
 门禁 10、12 号判的是引用；`cd research && cat notes/sample.md` 按 cd 之后的目录解析（notes/ 不在另外四个基准下）；已归档的写裸名 `old-round/`。
-这几种不是门禁号、也不是仓内路径，不许误报：提交窗口在 9 月 23 号前后；按第 13 号条款办；分支名 `origin/master`、类型 `text/plain`。
+这几种不是门禁号、也不是仓内路径，不许误报：提交窗口在 9 月 23 号前后；按第 13 号条款办；分支名 `origin/master`、类型 `text/plain`；日期 2026-09-26 号。
+门禁 10 / 12 号也判两个号。这两处判不了、要列进没判的：`.claude/kb/decisions/99-不存在的决策.md`，`nowhere.md`「某一节」。
diff --git a/.claude/gate.d/fixtures/10-kb-rot.sh/green/expect b/.claude/gate.d/fixtures/10-kb-rot.sh/green/expect
index 457369e..e462e88 100644
--- a/.claude/gate.d/fixtures/10-kb-rot.sh/green/expect
+++ b/.claude/gate.d/fixtures/10-kb-rot.sh/green/expect
@@ -3,3 +3,6 @@ want=kb 腐化审计通过
 want=欠检查 1 条、已还清 1 条
 want=不变量条数一致：正文声称 1 条在用，表里在用 1 条
 want=治理文档里的门禁号、路径与「小节」都指得到
+want=扫 1 份治理文档：门禁号 5 处、路径 4 处、小节 1 处；没判 2 处；跳过 2 处（被 .gitignore 挡着 0、不像仓内路径 2）
+want=`.claude/kb/decisions/99-不存在的决策.md` 不全是路径字符
+want=nowhere.md「某一节」 的文件在四个解析基准下都找不到，小节没判
diff --git a/.claude/gate.d/lib-governance-refs.py b/.claude/gate.d/lib-governance-refs.py
index 7a5a079..2d8a7db 100644
--- a/.claude/gate.d/lib-governance-refs.py
+++ b/.claude/gate.d/lib-governance-refs.py
@@ -5,7 +5,8 @@
 .claude/rules/*.md、.claude/skills/*/SKILL.md。三类指向：
 
   ① 门禁号：「门禁 65 号」「54、55、57、59 号」这类两位数加「号」，门禁目录里要有 <号>-*.sh。
-     前面紧挨「第」「月」「日」（可隔空白）的不算门禁号（「第 13 号」「9 月 23 号」）。
+     「、」或「/」连着的每个号都判（「20 / 21 号」判 20 与 21）；前面紧挨「第」「月」「日」（可隔空白）的不算门禁号（「第 13 号」「9 月 23 号」），
+     紧挨「-」「–」的也不算（「2026-09-26 号」；「54–59 号」这种区间写法不判，要判就逐个写出来）。
      门禁目录取本文件所在的目录，不取被扫的仓根：样本目录里没有门禁脚本。
   ② 反引号里的仓内路径：反引号里按空白切开的每个词（命令里的脚本路径也算），ASCII 写成、中间带 `/`、
      而且第一段在解析基准下现存、或末段带常见的文件扩展名（`origin/master`、`text/plain` 两样都不满足，不算仓内路径）。
@@ -14,12 +15,13 @@
      已归档的写裸文件名（`.claude/agent-common.md`「找不到历史实验的数据」那一条），不带目录，不在射程里。
   ③ `文件「小节」`：文件在仓里、而「小节」里的字（去掉空白、反引号、星号、「」之后）在那份文件的全文里一处都找不到。
      判的是「全文任意位置出现」，不只看标题：定义常点正文里的一句；小节改了名而原名的字还散在正文里时抓不到。
+     小节名里嵌套「」的（「「无效」那一栏」），只核到第一个」之前那一截。
 
 判不到、逐条列进「没判的」（不算红）：③ 里文件解析不到的（多半是只写了文件名、住在四个基准之外）；
-② 里带非 ASCII 字符、中间带 `/`、又指不到的记号（悬空的中文文件名路径与 `NN-简称.md` 这类占位分不开；指得到的照常算判过）。
+② 里不全是路径字符（非 ASCII、引号、`$` 这类）、中间带 `/`、又指不到的记号（悬空的中文文件名路径与 `NN-简称.md` 这类占位分不开；指得到的照常算判过）。
 
-输出：每处红一行「文件:行: 说明」；「没判的」逐条一行；末行「扫 N 份治理文档：门禁号 A 处、路径 B 处、小节 C 处；没判 D 处」。
-退出码：0 全指得到；1 有指不到的；2 一份治理文档都没扫到（没有对象，不许当通过）。
+输出：每处红一行「文件:行: 说明」；「没判的」逐条一行；末行「扫 N 份治理文档：门禁号 A 处、路径 B 处、小节 C 处；没判 D 处；跳过 E 处（被 .gitignore 挡着 F、不像仓内路径 G）」。
+退出码：0 全指得到；1 有指不到的；3 一份治理文档都没扫到（没有对象，不许当通过）。python 自己出错退别的码（打不开脚本是 2），调用方当「没跑成」。
 """
 import glob
 import os
@@ -30,7 +32,7 @@ import sys
 GATE_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
 CARRIER_PATTERNS = ["CLAUDE.md", ".claude/main-agent.md", ".claude/agent-common.md",
                     ".claude/agents/*.md", ".claude/rules/*.md", ".claude/skills/*/SKILL.md"]
-GATE_NUMBER = re.compile(r"(?<![0-9A-Za-z.])((?:[0-9]{2}\s*、\s*)*[0-9]{2})\s*号")
+GATE_NUMBER = re.compile(r"(?<![0-9A-Za-z.\-–])((?:[0-9]{2}\s*[、/]\s*)*[0-9]{2})\s*号")
 NOT_A_GATE_BEFORE = re.compile(r"(第|月|日)\s*$")
 BACKTICK = re.compile(r"`([^`\n]+)`")
 REPO_PATH = re.compile(r"^[A-Za-z0-9_.\-/]+$")
@@ -95,10 +97,11 @@ def main():
     if not carriers:
         print("  ✗ 一份治理文档都没扫到（CLAUDE.md、.claude/main-agent.md、.claude/agent-common.md、agents、rules、skills）")
         print("     → 怎么办：在仓库根上跑；治理文档搬了家的话，改本文件的 CARRIER_PATTERNS。")
-        return 2
+        return 3
     gates = existing_gate_numbers()
     problems, unjudged = [], []
     counts = {"gate": 0, "path": 0, "section": 0}
+    skipped = {"ignored": 0, "not_repo_path": 0}
     for carrier in carriers:
         with open(carrier, encoding="utf-8") as handle:
             lines = handle.read().split("\n")
@@ -106,7 +109,7 @@ def main():
             for match in GATE_NUMBER.finditer(line):
                 if NOT_A_GATE_BEFORE.search(line[:match.start()]):
                     continue
-                for gate in re.split(r"\s*、\s*", match.group(1)):
+                for gate in re.split(r"\s*[、/]\s*", match.group(1)):
                     counts["gate"] += 1
                     if int(gate) not in gates:
                         problems.append(f"{carrier}:{number}: 「{gate} 号」在门禁目录里没有 {gate}-*.sh")
@@ -123,9 +126,13 @@ def main():
                         if resolve(path, carrier, changed_directories) is not None:
                             counts["path"] += 1
                         else:
-                            unjudged.append(f"{carrier}:{number}: `{word}` 带非 ASCII 字符、指不到，分不清是占位还是悬空，路径没判")
+                            unjudged.append(f"{carrier}:{number}: `{word}` 不全是路径字符（非 ASCII、引号、`$` 这类）、指不到，分不清是占位还是悬空，路径没判")
                         continue
-                    if not looks_like_repo_path(path, carrier, changed_directories) or ignored_by_git(path):
+                    if not looks_like_repo_path(path, carrier, changed_directories):
+                        skipped["not_repo_path"] += 1
+                        continue
+                    if ignored_by_git(path):
+                        skipped["ignored"] += 1
                         continue
                     counts["path"] += 1
                     if resolve(path, carrier, changed_directories) is None:
@@ -147,7 +154,7 @@ def main():
         print("     → 怎么办：门禁号改成现存的号或共享 gate.sh 里那一道的名字；路径改成现存的，已归档的写裸文件名；小节按那份文件今天的标题改。")
     for item in unjudged:
         print(f"  没判的：{item}")
-    print(f"  扫 {len(carriers)} 份治理文档：门禁号 {counts['gate']} 处、路径 {counts['path']} 处、小节 {counts['section']} 处；没判 {len(unjudged)} 处")
+    print(f"  扫 {len(carriers)} 份治理文档：门禁号 {counts['gate']} 处、路径 {counts['path']} 处、小节 {counts['section']} 处；没判 {len(unjudged)} 处；跳过 {skipped['ignored'] + skipped['not_repo_path']} 处（被 .gitignore 挡着 {skipped['ignored']}、不像仓内路径 {skipped['not_repo_path']}）")
     return 1 if problems else 0
 
 
diff --git a/.claude/hooks/heavy-test-guard.sh b/.claude/hooks/heavy-test-guard.sh
index dc6bd51..a096e2f 100755
--- a/.claude/hooks/heavy-test-guard.sh
+++ b/.claude/hooks/heavy-test-guard.sh
@@ -764,7 +764,7 @@ def selftest(hook_dir):
             ("实现员 heredoc 写出不经内存包装的 cargo run 再起它", writer,
              "cat > gen-unwrapped.sh <<'EOF'\ncargo run --release --bin e160-random-small-read-share\nEOF\nbash gen-unwrapped.sh", 2, 0, 1),
             ("主 agent 不经内存包装跑测试目标：这一道不判主 agent", None, "cargo test -p singlefs-core --lib", 0),
-            ("崩溃验证员带前缀不经内存包装跑层 0 测试目标", crash, commit + "cargo test --release -p singlefs-harness --test first_transaction_step_seven_layer0", 2),
+            ("崩溃验证员带前缀不经内存包装跑测试目标", crash, commit + "cargo test --release -p singlefs-harness --test first_transaction_step_six_recovery", 2),
             ("崩溃验证员带前缀经内存包装跑 54 号全量：层 0 不归它", crash,
              commit + "bash research/scripts/run-with-memory-cap.sh 16G bash .claude/gate.d/54-layer0-replay.sh --full /tmp/wt", 2),
             ("崩溃验证员带前缀经内存包装跑 55 号", crash, commit + "bash research/scripts/run-with-memory-cap.sh 16G bash .claude/gate.d/55-qemu-first-transaction.sh", 0),
diff --git a/.claude/hooks/runner-dispatch-guard.sh b/.claude/hooks/runner-dispatch-guard.sh
index 45effcb..fdf7967 100755
--- a/.claude/hooks/runner-dispatch-guard.sh
+++ b/.claude/hooks/runner-dispatch-guard.sh
@@ -39,7 +39,7 @@
 #   SINGLEFS_HEAVY_TESTS=commit / user-request，由 heavy-test-guard.sh 判）；其余一律拒。
 #   会误拒：名字在前、动词在后、中间没有 TOPIC_GAP_BREAK 的说明句（「54 号在提交时跑全量」「54 号跑全量要四十分钟」），改写成「只在……才跑」或用「」括起来；
 #   否定词在括号外、重型阶段的名字在括号里的（「不跑重型测试（check.sh、`cargo test --workspace`）」）：按 ③ 括号里单独成分句，否定词不在那个分句里，照拒；
-#   否定词与名字写进同一个分句（「不跑 check.sh、`cargo test --workspace`」），或只指清单出处不列名字。
+#   要放行就把否定词与名字写进同一个分句（「不跑 check.sh、`cargo test --workspace`」），或只指清单出处、不列名字。
 #   会漏：同一分句里另有否定词的指令（「不改代码直接跑 54 号」）、宾语是代词的（「跑它」）、名字离动词太远的、没有动词的清单行（「验证负载：……层 0 各流快档」）；
 #   这些执行时 heavy-test-guard.sh 照拒。
 #
@@ -437,8 +437,8 @@ def decide(hook_input, project_root):
                    "否定句、引号里的、「……说，」「报告里写」「原句」之后的转述、重型阶段只是同句另一个名词的，都不拦（判法细节在这个 hook 的文件头）。\n"
                    "→ 规矩：重型测试（层 0、QEMU、herd7、crates 变异整表、全量测试、整轮门禁、全部实验复跑、E152 装置）只在提交代码时、或用户要求时跑；"
                    "子 agent 只跑自己动到的测试二进制、fmt / clippy / build 与 54、55、57、59、87 之外的门禁阶段；crash-verifier 只跑 55、57、59 号那几道，"
-                   "gate-triage 只跑 gate.sh 整轮与 87 号（执行时 .claude/hooks/heavy-test-guard.sh 也拒）。\n"
-                   "→ 怎么办：真要它跑就删掉这一句；不是要它跑的，写成否定句（「不跑层 0」），否定词与名字放在同一个分句里（名字别放进否定词后面的全角括号，括号里单独成分句），转述别人的原话用「」括起来；提交时的重阶段派 crash-verifier、整轮门禁派 gate-triage；"
+                   "gate-triage 只跑整轮门禁（research/scripts/gate-staged.sh）与 87 号（执行时 .claude/hooks/heavy-test-guard.sh 也拒）。\n"
+                   "→ 怎么办：真要它跑就删掉这一句；不是要它跑的，写成否定句（「不跑层 0」），否定词与名字放在同一个分句里（名字别放进否定词后面的全角括号，括号里单独成分句），转述别人的原话用「」括起来；提交时 QEMU、herd7、crates 变异整表派 crash-verifier，层 0 全量由主 agent 自己跑，整轮门禁派 gate-triage；"
                    "提交之外任务确实要跑，先弹窗问用户，用户同意了由主 agent 带 SINGLEFS_HEAVY_TESTS=user-request 跑。")
     snapshot_code, snapshot_message = snapshot_conflict_verdict(subagent_type, prompt, project_root)
     if snapshot_code:
diff --git a/.claude/main-agent.md b/.claude/main-agent.md
index b1b8ac7..8c1390e 100644
--- a/.claude/main-agent.md
+++ b/.claude/main-agent.md
@@ -56,7 +56,7 @@
 | 改 `crates/`：探索性的、条款没写全、要用户边看边拍板 | 主 agent 自己写、直接和用户对话 → 上一行的三方（代码轮）→ 提交时照「暂存之后、提交之前跑门禁」那一行：层 0 全量主 agent 跑，`crash-verifier` 跑 QEMU、herd7、crates 变异表 |
 | 跑变异表、看抓到 / 无效 / 没红 | `mutation-triage` → 要补取样点、补断言的：`crates/` 那侧 `implementation-writer`（输入给分诊报告），`research/` 那侧 `experiment-designer` 写重跑登记 → `experiment-runner` |
 | 一个阶段任务结束：里程碑一步、一轮判决、一段实验、一批定义或脚本改完，这一批交回到齐、暂存之前 | `sweep` 写事实表（阶段同步第一段；输入给改动范围与做成的事，只用过、没留改动的也写；交回的事实表过了 `research/scripts/stale-candidates.py --check-facts`）→ 候选表按组切段，每段不超过约 200 行，一段派一个 `sweep` 逐行判（第二段；输入同样给做成的事）→ 交回齐了主 agent 跑 `stale-candidates.py --check-report 候选表 各份报告`，退出码 0 才往下，再**逐行全看**：每一条判定都对着载体今天的原文核一遍，不抽样、不按判定种类挑。判错的那一段重派，重派交回照样全看 → 主 agent 逐处判，自己改或交 `kb-scribe` → 主 agent 写 `research/prompts/<阶段>-sync.md`（格式见门禁 68 号文件头，68 号判形式）→ 下一行 |
-| 暂存之后、提交之前跑门禁 | 主 agent 用 `research/scripts/stage-mine.py` 暂存 → 这一批改了门禁 54 号在 `.claude/gate.d/stage-inputs.tsv` 登记的输入、54 号脚本本身或工具链（`cargo -V`、`rustc -V`；这三样都进全绿标记的键），主 agent 自己在后台跑 `.claude/gate.d/54-layer0-replay.sh` 里 `print_staged_worktree_full_commands` 打印的那三行命令，三行放进同一次 Bash 调用（它们共用一个 shell 变量；建 HEAD + 暂存区的 worktree、带 `SINGLEFS_HEAVY_TESTS=commit` 经内存包装跑那棵树里的 `--full`、删 worktree；命令只在那一处写，不在这里抄）（层 0 全量只在这一步跑，判绿按输入哈希写全绿标记），看门狗用 `--processes` 盯；那次调用退 0（退出码就是 `--full` 的）、全绿标记写好之后才往下，不然整轮门禁里的 54 号必红 → 派 `crash-verifier` 跑 55、57、59，命令带 `SINGLEFS_HEAVY_TESTS=commit` → `gate-triage` 带 `SINGLEFS_HEAVY_TESTS=commit` 跑 `research/scripts/gate-staged.sh`（它跑 `gate.sh --staged`，整轮全绿才前移 `refs/sop/staged-green`）并分诊：54、55、59、74、87 号只在这一趟里能复用上一次整轮全绿的判定，判据在 `research/scripts/stage-must-run.sh` 文件头；要跑时 54 号跑快档并核全绿标记，57 号没有复用、每次现跑；用户要求时各处都换成 `=user-request` |
+| 暂存之后、提交之前跑门禁 | 主 agent 用 `research/scripts/stage-mine.py` 暂存 → 这一批改了门禁 54 号在 `.claude/gate.d/stage-inputs.tsv` 登记的输入、54 号脚本本身或工具链（`cargo -V`、`rustc -V`；这三样都进全绿标记的键），主 agent 自己在后台跑 `.claude/gate.d/54-layer0-replay.sh` 里 `print_staged_worktree_full_commands` 打印的那三行命令，三行放进同一次 Bash 调用（它们共用一个 shell 变量；建 HEAD + 暂存区的 worktree、带 `SINGLEFS_HEAVY_TESTS=commit` 经内存包装跑那棵树里的 `--full`、删 worktree；命令只在那一处写，不在这里抄）（层 0 全量只在这一步跑，判绿按输入哈希写全绿标记），看门狗用 `--processes` 盯；那次调用退 0（退出码是经内存包装的那条 `--full` 命令的，250–254 是包装自己的结局）、全绿标记写好之后才往下，不然整轮门禁里的 54 号必红 → 派 `crash-verifier` 跑 55、57、59，命令带 `SINGLEFS_HEAVY_TESTS=commit` → `gate-triage` 带 `SINGLEFS_HEAVY_TESTS=commit` 跑 `research/scripts/gate-staged.sh`（它跑 `gate.sh --staged`，整轮全绿才前移 `refs/sop/staged-green`）并分诊：54、55、59、74、87 号只在这一趟里能复用上一次整轮全绿的判定，判据在 `research/scripts/stage-must-run.sh` 文件头；要跑时 54 号跑快档并核全绿标记，57 号没有复用、每次现跑；用户要求时各处都换成 `=user-request` |
 | 改了一个数或格式常量、撤回一条结论、新立一条判据 | `sweep`（四种活与各自要给的输入见它的定义） |
 | 定案之后写回 kb | `kb-scribe`（主 agent 给逐条规格）；规格动了某条分项的「**依据**」段时，同一份规格要带上那个实验页 `### 影响的决策` 表里对应那几行（门禁 75 号那条双向检查两侧要同时到位，而判一个实验撑不撑一条分项是主 agent 的活）；它翻了分项状态、33 号红（变异表锚点腐化）→ 红在 `research/mutations/` 的表：`experiment-runner` 只修锚点；红在 `crates/mutations.tsv`（`relabel-item.py` 改写了 `crates/` 下的源码）：派 `implementation-writer` 修锚点，走代码轮 |
 | 要建计数实验，或重跑已有实验 | 主 agent 写岔路单（`.claude/rules/three-way-inference.md`「交岔路时写岔路单」；不是为岔路建的实验写同格式的问题单）→ `experiment-designer` 写跑前登记或重跑登记 → 主 agent 删掉登记里待删的问法（有的话）→ `experiment-runner` → 每段交回主 agent 对着岔路单判够不够：一行开着的都不剩就停、交岔路表；续派时派发提示点名还差的那几行（续派闸 `.claude/hooks/runner-dispatch-guard.sh` 查这一句） |
diff --git a/.claude/rules/format-evolution.md b/.claude/rules/format-evolution.md
index 07d2173..aa76d5f 100644
--- a/.claude/rules/format-evolution.md
+++ b/.claude/rules/format-evolution.md
@@ -67,7 +67,7 @@
 ⇒ 改一个格式常量时：
 
 1. 动手前按「字面量、消息串、预留宽的读写、下标、倍数、商与平方、测试与门禁里钉的产物值」逐类全仓搜一遍，输出不截断；
-2. 改完把**只可能指旧值**的那几个串登记进它 `format-const` 标记的 `stale=`（`|` 分隔），27 号从此在 kb 正文与 `research/`、`crates/` 下的 `.rs` 里替你盯着；门禁脚本（`.sh`、`.py`）与 `research/results/` 下的产物它不扫，那几类照第 1 步手工搜。
+2. 改完把**只可能指旧值**的那几个串登记进它 `format-const` 标记的 `stale=`（`|` 分隔），27 号从此在 kb 正文与 `research/`、`crates/` 下的 `.rs` 里替你盯着；别处它都不扫：门禁脚本（`.sh`、`.py`）、`research/results/` 下的产物、规则与 agent 定义、`records/`、`research/prompts/`、kb 的历史节与变更史，这几类照第 1 步手工搜。
    裸数字（`148`）不许登记：E145（码 2 自描述头与映射树 key 宽的代价） 的 extent 扇出也是 148，登记了就误拒。登记之前现扫一遍，kb 正文与源码里命中 0 次才登记；
 3. 标记写在表格单元格里的，挪到表后单独一行：`stale=` 的分隔符 `|` 会把表格切断。
 
diff --git a/.claude/rules/implementation-workflow.md b/.claude/rules/implementation-workflow.md
index 6ba5e10..abf62ff 100644
--- a/.claude/rules/implementation-workflow.md
+++ b/.claude/rules/implementation-workflow.md
@@ -49,7 +49,7 @@
 
 | 场合 | 跑不跑 |
 |---|---|
-| 每次提交代码 | **必须跑**：主 agent 在提交流程里后台起（命令带 `SINGLEFS_HEAVY_TESTS=commit`），看门狗盯；整轮门禁由 `gate-triage` 带前缀跑 `research/scripts/gate-staged.sh`（`.claude/main-agent.md`「暂存之后、提交之前跑门禁」那一行） |
+| 每次提交代码 | **必须跑**，命令都带 `SINGLEFS_HEAVY_TESTS=commit`：层 0 全量由主 agent 在提交流程里后台起、看门狗盯；QEMU、herd7、crates 变异整表由 `crash-verifier` 跑；整轮门禁由 `gate-triage` 带前缀跑 `research/scripts/gate-staged.sh`（`.claude/main-agent.md`「暂存之后、提交之前跑门禁」那一行） |
 | 用户当场要求，或任务确实要跑 | 任务确实要跑时主 agent 先弹窗问用户，用户同意了才跑；命令带 `SINGLEFS_HEAVY_TESTS=user-request` |
 | 其余任何时候 | 不跑 |
 | 崩溃验证员、门禁分诊员 | 各跑各的那一部分，只在提交时或用户要求时（命令带 `SINGLEFS_HEAVY_TESTS=commit` 或 `=user-request`）：层 0 全量由主 agent 在 HEAD + 暂存区的 worktree 里跑；崩溃验证员跑 55、57、59 号；门禁分诊员跑 `research/scripts/gate-staged.sh`（它跑 `gate.sh --staged`）并分诊：54、55、59、74、87 号只在这一趟里能复用上一次整轮全绿的判定（判据在 `research/scripts/stage-must-run.sh` 文件头），要跑时 54 号跑快档并核全绿标记，57 号没有复用、照跑 |
diff --git a/.claude/rules/mutation-sampling.md b/.claude/rules/mutation-sampling.md
index 68abe34..f2af961 100644
--- a/.claude/rules/mutation-sampling.md
+++ b/.claude/rules/mutation-sampling.md
@@ -79,7 +79,7 @@
 
 变异表里一条本来好好的条目，会因为**别处改了一个常量**而从「被抓」变成「无效」，而没有任何东西报警。
 
-⇒ **改完格式常量，重跑受影响的每张变异表，比对「抓到 / 无效 / 没红」三个数**，不能只看「没红」是不是 0（`mutate.sh` 没有三数汇总行，按每条结果行开头的符号数（`[ ]` 里是变异名，不是结局）：`✅` 抓到、`⏭` 无效、`❌` 没红；`💥`（测试进程没跑完）、`⚠️`（报了失败却一个测试名都没抓到）、`⏱`（超时）、`🧱`（内存撞顶）另列，几栏加起来要等于表的条数；`⚠️`、`⏱`、`🧱` 不为 0 的整轮已判失败，`💥` 不判失败，照样单列、逐条交出去；`mutate.sh` 收尾那行「计数：」只报内存撞顶与超时两栏；门禁 59 号照它自己收尾的「计数：」行，那一行没有 `💥`、`⚠️` 两栏：测试进程没跑完的，输出里有编译错误记「无效」、没有记「没红」，两种都让整道判红；`crates/mutations.tsv` 整表复跑是重型，由提交时的崩溃验证员跑 59 号）：
+⇒ **改完格式常量，重跑受影响的每张变异表，比对「抓到 / 无效 / 没红」三个数**，不能只看「没红」是不是 0（`mutate.sh` 没有三数汇总行，按每条结果行开头的符号数（`[ ]` 里是变异名，不是结局）：`✅` 抓到、`⏭` 无效、`❌` 没红；`💥`（测试进程没跑完）、`⚠️`（报了失败却一个测试名都没抓到）、`⏱`（超时）、`🧱`（内存撞顶）另列，几栏加起来要等于表的条数；`⚠️`、`⏱`、`🧱` 不为 0 的整轮已判失败，`💥` 不判失败，照样单列、逐条交出去；`mutate.sh` 收尾那行「计数：」只报内存撞顶与超时两栏；门禁 59 号照它自己收尾的「计数：」行，那一行没有 `💥`、`⚠️` 两栏：测试进程没跑完的，输出里有一行 `error:` 开头（测试二进制被信号杀时 cargo 也打 `error: test failed`）或 `could not compile` 就记「无效」，否则记「没红」，两种都让整道判红；「无效」里尾巴带 `process didn't exit successfully` 的是进程被杀，不是替换文编不过；另外三栏「被总上限挤掉」「排不上没跑」「scope 起不来没跑」不为 0 也让整道判红，照原样另列；`crates/mutations.tsv` 整表复跑是重型，由提交时的崩溃验证员跑 59 号）：
 无效那一栏变多，等于有断言被关掉了，而它与「这一条本来就抓不到」在输出里长得一模一样。
 
 ## 判据
diff --git a/.claude/rules/three-way-inference.md b/.claude/rules/three-way-inference.md
index 24951a4..012c5c7 100644
--- a/.claude/rules/three-way-inference.md
+++ b/.claude/rules/three-way-inference.md
@@ -160,7 +160,7 @@
 
 ⇒ **做法**：拿一条腿的「没打中」去支撑任何结论之前，至少再抽一次；两次都没打中才记「没打中」，
 两次不一致就照 `.claude/singlefs-ai-sop/rules/test-discipline.md` 记「不稳定」，不下结论。
-云端腿同样适用——这条管的是「模型答复」这类观测，不是本机那条腿特有的：云端腿报「没打中」而要拿它支撑结论时，主 agent 用同一份提示再派一条同立场腿，报告带 `-s2` 后缀。
+云端腿同样适用——这条管的是「模型答复」这类观测，不是本机那条腿特有的：云端腿报「没打中」而要拿它支撑结论时，主 agent 用同一份提示再派一条同立场腿，报告写成 `<轮>-<模型>-s2-output.md`（例 `-opus-s2-output.md`），模型目录写成 `<轮>-<模型>-s2-model/`，不盖掉第一条腿的。
 
 ## 云端腿的报告要分段落盘
 
diff --git a/.claude/skills/crash-test/SKILL.md b/.claude/skills/crash-test/SKILL.md
index 2901d28..eb20895 100644
--- a/.claude/skills/crash-test/SKILL.md
+++ b/.claude/skills/crash-test/SKILL.md
@@ -9,4 +9,4 @@ description: 跑 singlefs 的验证套件——LKMM 内存序、QEMU/KVM 压测
 
 ## 在本项目里
 
-共享正文不管内存序与虚机（它写明那两样由项目自己定）。本项目的四样各落在一道门禁：崩溃点重放是 `.claude/gate.d/54-layer0-replay.sh`，QEMU 真设备是 `55-qemu-first-transaction.sh`，LKMM 内存序是 `57-lkmm.sh`，模型对拍是 `74-model-differential.sh`。前三样是重型测试，谁在什么时候跑、带什么前缀，照 `.claude/rules/implementation-workflow.md`「重型测试只在提交时跑」与 `.claude/main-agent.md`「暂存之后、提交之前跑门禁」那一行；子 agent 里只有崩溃验证员与门禁分诊员各跑登记给自己的那几道，别的子 agent 不跑，钩子 `.claude/hooks/heavy-test-guard.sh` 会拒。
+共享正文不管内存序与虚机（它写明那两样由项目自己定）。本项目的四样各落在一道门禁：崩溃点重放是 `.claude/gate.d/54-layer0-replay.sh`，QEMU 真设备是 `55-qemu-first-transaction.sh`，LKMM 内存序是 `57-lkmm.sh`，模型对拍是 `74-model-differential.sh`。前三样是重型测试，谁在什么时候跑、带什么前缀，照 `.claude/rules/implementation-workflow.md`「重型测试只在提交时跑」与 `.claude/main-agent.md`「暂存之后、提交之前跑门禁」那一行；子 agent 里只有崩溃验证员（55、57、59 号）与门禁分诊员（整轮门禁 `research/scripts/gate-staged.sh`）各跑自己那一份，别的子 agent 不跑，钩子 `.claude/hooks/heavy-test-guard.sh` 会拒。
diff --git a/.claude/skills/gate/SKILL.md b/.claude/skills/gate/SKILL.md
index 9dd2f98..3691c69 100644
--- a/.claude/skills/gate/SKILL.md
+++ b/.claude/skills/gate/SKILL.md
@@ -9,4 +9,4 @@ description: 跑 singlefs 的准入门禁。提交代码前、判断一个改动
 
 ## 在本项目里
 
-`gate.sh` 与 `check.sh` 在本项目里是重型测试，不是快速反馈：提交时由 `gate-triage` 带 `SINGLEFS_HEAVY_TESTS=commit` 跑 `research/scripts/gate-staged.sh`（它跑 `gate.sh --staged`；`.claude/main-agent.md`「暂存之后、提交之前跑门禁」那一行），其余时候不跑，主 agent 不带前缀跑会被 `.claude/hooks/heavy-test-guard.sh` 拒。平时要快速反馈，单跑 `.claude/gate.d/` 下 54、55、57、59、87 之外的阶段。共享正文的阶段表不全，现有阶段以 `.claude/singlefs-ai-sop/scripts/gate.sh` 里 `run_stage` 那几行与 `.claude/gate.d/` 目录为准。
+`gate.sh` 与 `check.sh` 在本项目里是重型测试，不是快速反馈：提交时由 `gate-triage` 带 `SINGLEFS_HEAVY_TESTS=commit` 跑 `research/scripts/gate-staged.sh`（它跑 `gate.sh --staged`；`.claude/main-agent.md`「暂存之后、提交之前跑门禁」那一行），用户要求时换成 `SINGLEFS_HEAVY_TESTS=user-request`，其余时候不跑，主 agent 不带前缀跑会被 `.claude/hooks/heavy-test-guard.sh` 拒。平时要快速反馈，单跑 `.claude/gate.d/` 下 54、55、57、59、87 之外的阶段。共享正文的阶段表不全，现有阶段以 `.claude/singlefs-ai-sop/scripts/gate.sh` 里 `run_stage` 那几行与 `.claude/gate.d/` 目录为准。
diff --git a/research/scripts/ask-local-selftest.sh b/research/scripts/ask-local-selftest.sh
index 4d907af..a09a78c 100755
--- a/research/scripts/ask-local-selftest.sh
+++ b/research/scripts/ask-local-selftest.sh
@@ -24,7 +24,8 @@ sys.argv[1]
 open(sys.argv[1],"w").write('The allocation table is written once per checkpoint and read again on mount. Each unit carries a header that names the tree, the object and the offset. The scan path walks the device in fixed steps and claims every unit whose magic matches and whose header checksum verifies. A unit that fails either test is skipped and reported to the caller as a bad block. The rebuild path runs only when the index trees are gone, and it accepts a unit only after the tag over the ciphertext verifies. Replicas of one extent hold the same ciphertext, so the payload checksum of two replicas is equal whenever the two units carry the same data. The resetting resetting of the batch is fine. The allocation table is written once per checkpoint and read again on mount. Each unit carries a header that names the tree, the object and the offset. The scan path walks the device in fixed steps and claims every unit whose magic matches and whose header checksum verifies. A unit that fails either test is skipped and reported to the caller as a bad block. The rebuild path runs only when the index trees are gone, and it accepts a unit only after the tag over the ciphertext verifies. Replicas of one extent hold the same ciphertext, so the payload checksum of two replicas is equal whenever the two units carry the same data. ')
 PYEOF
 printf 'ask-local selftest prompt, plain english, no emphasis.\n' > "$D/case1-prompt.md"
-ASK_LOCAL_FAKE_TEXT="$D/corrupt.txt" bash "$ASK_LOCAL" "$D/case1-prompt.md" >"$D/o1" 2>"$D/e1"
+# 调用方环境里带着 VOID_SAVED=1 也要照样留 void：这一格证「运行前清零」那一行里 VOID_SAVED 那一半
+VOID_SAVED=1 ASK_LOCAL_FAKE_TEXT="$D/corrupt.txt" bash "$ASK_LOCAL" "$D/case1-prompt.md" >"$D/o1" 2>"$D/e1"
 rc=$?
 [[ $rc -eq 5 ]] || { say ✗ "损坏正文没判红（退出码 $rc，应为 5）"; fail=1; }
 # 判红那一轮 stdout 必须是空的：此前正文先打到 stdout 再过闸，调用方的重定向文件里就落了一份作废输出，
@@ -43,7 +44,7 @@ import sys
 open(sys.argv[1],"w").write('The allocation table is written once per checkpoint and read again on mount. Each unit carries a header that names the tree, the object and the offset. The scan path walks the device in fixed steps and claims every unit whose magic matches and whose header checksum verifies. A unit that fails either test is skipped and reported to the caller as a bad block. The rebuild path runs only when the index trees are gone, and it accepts a unit only after the tag over the ciphertext verifies. Replicas of one extent hold the same ciphertext, so the payload checksum of two replicas is equal whenever the two units carry the same data. The allocation table is written once per checkpoint and read again on mount. Each unit carries a header that names the tree, the object and the offset. The scan path walks the device in fixed steps and claims every unit whose magic matches and whose header checksum verifies. A unit that fails either test is skipped and reported to the caller as a bad block. The rebuild path runs only when the index trees are gone, and it accepts a unit only after the tag over the ciphertext verifies. Replicas of one extent hold the same ciphertext, so the payload checksum of two replicas is equal whenever the two units carry the same data. ')
 PYEOF
 printf 'ask-local selftest prompt two.\n' > "$D/case2-prompt.md"
-# 调用方环境里带着同名标记（UNCHECKED=1）也不许影响判定：这一格同时证「运行前清零」那一行有用
+# 调用方环境里带着同名标记（UNCHECKED=1）也不许影响判定：这一格证「运行前清零」那一行里 UNCHECKED 那一半
 UNCHECKED=1 ASK_LOCAL_FAKE_TEXT="$D/clean.txt" bash "$ASK_LOCAL" "$D/case2-prompt.md" >"$D/o2" 2>"$D/e2"
 rc=$?
 [[ $rc -eq 0 ]] || { say ✗ "干净正文被判红（退出码 $rc，应为 0）"; fail=1; }
diff --git a/research/scripts/ask-local.sh b/research/scripts/ask-local.sh
index 1450d2a..07aa08d 100755
--- a/research/scripts/ask-local.sh
+++ b/research/scripts/ask-local.sh
@@ -10,7 +10,8 @@
 #
 # 退出码：0 过了字词损坏闸，正文在 stdout；2 取不到网关的 key 或提示为空；3 请求失败、响应不是 JSON、网关报错或没有 choices；
 #   4 正文为空；5 判为字词损坏（没设 ASK_LOCAL_ALLOW_CORRUPT=1 时）；6 损坏检测器没跑成或找不到，这一份没验过。
-#   5 与 6 都不打正文、把正文留成 -output-void<n>.md。
+#   5 与 6 都不打正文、把正文留成 -output-void<n>.md；设了 ASK_LOCAL_ALLOW_CORRUPT=1 时判红也退 0、打正文，照样留 void。
+#   内嵌的 python 自己出错时退 1（不在上面几种里，当没跑成）。
 set -uo pipefail
 
 CENTER="${AI_CENTER_DIR:-$HOME/code/ai-center}"
````
