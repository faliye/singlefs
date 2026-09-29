# 附录二：里程碑二收尾定义改动（`git diff` / `git show` 原样；生成于 2026-09-26）

基准：`HEAD` = `73ba4a4c019b9e3fc9c92f3122bfbbdaee93c321`。这一轮是定义轮，不是 `crates/` 代码轮：两段 diff 里都没有新建文件，因此各自都没有「新文件全文」小节。

## 一、工作区相对 HEAD 的改动

命令：`git diff HEAD -- .claude/agents .claude/agent-common.md .claude/main-agent.md .claude/gate.d/stage-owners.tsv`

```diff
diff --git a/.claude/agent-common.md b/.claude/agent-common.md
index f42993c..0fa5282 100644
--- a/.claude/agent-common.md
+++ b/.claude/agent-common.md
@@ -45,7 +45,8 @@
 - 不编译 Rust、不跑 `gate.sh` 全量，除非定义明写要做。跑脚本、跑模型一律加 `nice -n 19`。
 - 重型测试（层 0、QEMU、herd7、crates 变异整表、全量 `cargo test`（`--all` / `--workspace` / 工作区根裸跑）与 `check.sh`、`gate.sh` 整轮、87 号全部实验复跑、E152 装置）只在提交代码时跑一次、或用户要求时跑；子 agent 一律不跑，只跑自己动到的测试二进制（`cargo test -p <crate> --test <自己的目标>`、`--lib`）、fmt / clippy / build 与 `.claude/gate.d/` 下 54、55、57、59、87 之外的阶段。`crash-verifier` 只跑 54、55、57、59 号那几道，`gate-triage` 只跑 `gate.sh` 整轮与 87 号，都在提交时或用户要求时跑、命令带 `SINGLEFS_HEAVY_TESTS=commit`（用户要求时 `=user-request`）。项目 settings 里的 `.claude/hooks/heavy-test-guard.sh` 在执行前拒绝越出这些的命令；被拒了不换写法绕过去，在交回里写明要跑什么、为什么，交主 agent 去问用户。
 - 写进脚本文件、放进 `python3` 的 subprocess、`make` 这类间接起法同样算跑：自己写的验证链里不许出现重型测试的任何一步，名字含 `layer0` 的测试二进制也不许。
-- 要停自己起的一条链（脚本、后台任务）时，先列出它的整棵子进程（`ps -o pid,ppid,args --ppid <pid>` 逐层往下），从最底层起逐个 `scripts/proc.py stop <pid>`（`proc.py` 只停给定的那一个，不带子进程），停完再列一遍，确认一个不剩；不许留下正在跑的那一步「让它跑完」。
+- 跑编译出来的代码经内存包装：`cargo test` / `run` / `bench`（带 `--no-run` 只编不跑的不算）与直接执行 cargo 编出来的二进制（实验装置、测试二进制），一律经 `bash research/scripts/run-with-memory-cap.sh <上限> <命令…>` 跑；写进自己的脚本再跑的，整条脚本经它跑（`bash research/scripts/run-with-memory-cap.sh <上限> bash <脚本>`）。上限取派发提示给的（定义另写了来源的照定义），都没给就取 `research/scripts/replay.sh` 文件头 `REPLAY_MEMORY_CAP` 的默认值。`research/scripts/mutate.sh`、`research/scripts/replay.sh` 与门禁 59 号在里面逐条套了，直接跑，外面不再包一层。退出码 250–254 是包装自己的结局（含义见 `research/scripts/run-with-memory-cap.sh` 文件头「退出码」一段）：那一次的输出不算结果，命令与退出码写进报告，不绕开包装重跑。`cargo build`、`cargo clippy`、`cargo fmt` 不跑编出来的代码，不要求经它。
+- 要停自己起的一条链（脚本、后台任务）时，先列出它的整棵子进程（`ps -o pid,ppid,args --ppid <pid>` 逐层往下），从最底层起逐个 `python3 .claude/singlefs-ai-sop/scripts/proc.py stop <pid>`（`proc.py` 只停给定的那一个，不带子进程），停完再列一遍，确认一个不剩；不许留下正在跑的那一步「让它跑完」。
 - 要编译、跑门禁阶段、跑变异或产物的，开跑前 `ps -o pid,args -u "$(id -u)"` 看负载：有性能测量在跑（`qemu-system`、`vm-bench.sh`、`e152-file-system-benchmark`、`fio`）就报「在等 pid …（命令）」停下；只有别的 `cargo`、`gate.sh` 在跑的，加 `nice -n 19` 照常跑（cargo 自己排队拿文件锁），报告里记下 `ps` 看到了什么、等锁等了多久。定义另有更严要求的照定义。
 - 读大文件（超过 500 行）先 grep 定位、再按行段读（Read 的 offset / limit），不整份读；同一份文件读过一次、之后没改过，就不再整份重读。长命令的输出落进草稿目录的文件，只读汇总行（`tail -n 20`、`grep -c`、`grep ✗`），不把整份日志、整份 diff 读进上下文。
 - 派发提示里有「线程上限：N」的，编译、测试、变异、产物、原型扫描的每条命令都用 `bash research/scripts/capped.sh N <命令>` 起（它把 cargo 与 `crates/singlefs-harness` 各装置读的线程变量一次设成 N）；实验装置自己的线程变量（`KS_THREADS` 这类）同样设成不超过 N。主 agent 发消息改了 N，从下一条命令起照新的。
@@ -54,7 +55,19 @@
 - 本机常有别的会话在同一个仓里干活：只动这一轮自己的文件；看到别人没提交的改动不碰、不修。
 - 长活可以等，不给它设超时、不自己中途杀掉：编译、变异表、复跑这类长活用 Bash 的 `run_in_background` 起，起完结束本轮，完成时会通知你。交给它的命令要一直跑到真正的活结束才退出：不在里面再把活放到后台——不用 `disown`、`coproc`、`setsid -f`、`nohup … &`、`tmux`/`screen` 的分离模式、`systemd-run`，子 shell 或 `$( … )` 里也不写 `&`；`&` 只在同一条命令随后用不带参数的 `wait` 等齐时用。结束本轮就是这一条回复只写一句在等什么、不再调工具；前台命令的 `timeout` 不超过 240000 毫秒。
   等长活时不起缓存计时器，结束本轮直接等完成通知。
-  等的时候看到进程不动、报错、或等的条件不会成立，把看到的命令、输出与时刻写进草稿目录里的进度记录，接着做定义里还能做的；结不结束、怎么处置由主 agent 定。主 agent 的看门狗（`research/scripts/agent-watch.py`）定时复检你；项目 settings 里的 Bash 检出 hook（`.claude/hooks/bash-command-detector.sh`）对五种写法在执行前拒绝：前台没有超时的等待循环、把活放出追踪（`disown`、`nohup … &` 这类）、run_in_background 里以单独的 `&` 收尾而之后同一条命令里没有 `wait` 的作业、用 `>`、不带 `-a` 的 `tee`、`cp` / `mv` / `install` 整份覆盖 `research/results/` 下已存在又没进 git 的产物（要换就按日期另存新文件名，旧的留着）、起看门狗的错误写法（只有主 agent 起看门狗）；run_in_background 里的等待循环与按模式找进程这类命令只记检出，交给主 agent 判断。等日志里的一行字之前，先确认那一行真会写进那个文件；找进程用写死的 pid，不用 `pgrep -f` / `pkill -f`（command-safety.md）。
+  等的时候看到进程不动、报错、或等的条件不会成立，把看到的命令、输出与时刻写进草稿目录里的进度记录，接着做定义里还能做的；结不结束、怎么处置由主 agent 定。主 agent 的看门狗（`research/scripts/agent-watch.py`）定时复检你；前台没有超时的等待循环、把活放出追踪、run_in_background 里后面没有 `wait` 的单独 `&` 在执行前被拒（「执行前拒绝的写法」那一条的 ②③④），run_in_background 里的等待循环只记检出，交给主 agent 判断。等日志里的一行字之前，先确认那一行真会写进那个文件；找进程用写死的 pid，不用 `pgrep -f` / `pkill -f`（command-safety.md）。
+- 执行前拒绝的写法：项目 settings 里的几道 hook 在执行前拒绝这几类写法（退出 2，stderr 写原因与出路），主 agent 与子 agent 一样拒；拒的只是这一次调用，不停已经在跑的东西。被拒了照出路改写法，不换写法绕过去；改不成的写进报告交主 agent。
+  - Bash 检出 hook（`.claude/hooks/bash-command-detector.sh`），④ 只在 run_in_background 里拒，其余前台与 run_in_background 一样拒：
+    ① 起看门狗的错误写法：`research/scripts/watch.sh`、`research/scripts/agent-watch.py watch` 不用 run_in_background 起，或命令里带单独的 `&`、`nohup`、`disown`、`setsid`、把输出丢进 `/dev/null`。看门狗只有主 agent 起。
+    ② 前台没有超时的等待循环：`until` / `while` 里有 `sleep`，外面没套 `timeout`。要等就用 run_in_background 起、结束本轮等完成通知，或 `python3 .claude/singlefs-ai-sop/scripts/proc.py wait <pid> --timeout <秒>`。
+    ③ 把活放出追踪：`disown`、`coproc`、`setsid -f`、`nohup … &`、`tmux` / `screen` 的分离模式、不等到结束的 `systemd-run`。
+    ④ run_in_background 里有作业以单独的 `&` 收尾，之后同一条命令里没有 `wait`。
+    ⑤ 整份覆盖 `research/results/` 下已存在又没进 git 的产物：`>`、`>|`、`&>`、不带 `-a` 的 `tee`、`cp` / `mv` / `install` 的目标、`dd of=`、`truncate`。追加（`>>`、`tee -a`）与新文件名不拦；要换就按日期另存新文件名，旧的留着。
+    ⑥ 终止进程不是点名一个自己起的进程号或任务号：`kill` 的目标带负号、是 0 或 `$PPID`、一次给几个、是命令替换或通配，在循环里逐个发（`proc.py stop` 同样），按名字、按 cgroup 或 `/proc` 挑进程，写死的进程号指到自己会话的祖先、别的会话起的进程或 SSH / VSCode / 本地模型服务，`systemctl` 停 SSH、登录会话、本地模型服务这类单元，关机重启。放行的是点名一个：`kill "$!"`、`kill %1`、单独一条 `python3 .claude/singlefs-ai-sop/scripts/proc.py stop <pid>`。
+    ⑦ 在同一个 inode 上改已经存在的脚本（`.sh`、`.py` 或带执行位的文件，在仓里或 `/tmp/claude-1000/` 下）：`>`、`>|`、`&>`、不带 `-a` 的 `tee`、`cp` 的目标、`dd of=`、`truncate`、python 的 `open(…, 'w')` 这一类。改脚本写到同目录临时文件再 `mv` 换上，或用 `research/scripts/replace-once.py` / `insert-row.py` 定点改；追加与新建不拦。
+  - 上游的 `.claude/singlefs-ai-sop/scripts/claude-hooks/pattern-process-guard.sh`：命令位置上的 `pgrep -f`、`pkill -f`、`killall`。
+  - 重型测试闸（`.claude/hooks/heavy-test-guard.sh`）：越出重型测试那一条的命令（主 agent 不带 `SINGLEFS_HEAVY_TESTS` 前缀也拒）；子 agent 不经内存包装跑编译出来的代码（「跑编译出来的代码经内存包装」那一条）。
+  - 写闸（`.claude/hooks/write-guard.sh`，只看 Write / Edit 工具）：Write 整份覆盖仓里已存在又没进 git 的文件；项目子 agent 写到写范围表（`.claude/hooks/agent-write-scope.tsv`）里自己那几行之外；写进的内容里有撇号类角标（字符与起名的写法见 `.claude/rules/path-moves.md`「变体起新名字，不用角标」）。
 
 ## 门禁
 
diff --git a/.claude/agents/crash-verifier.md b/.claude/agents/crash-verifier.md
index a0983a7..09197a5 100644
--- a/.claude/agents/crash-verifier.md
+++ b/.claude/agents/crash-verifier.md
@@ -22,6 +22,7 @@ omitClaudeMd: true
 ## 做什么
 
 1. 每个阶段开跑前各照共用约束「不做」一节看一次负载；另有别的 `gate.sh` 正在跑 54 号、或有 `54-layer0-replay.sh --full` 在跑时，不起 54 号，等它结束。
+1b. 54、55、57 号整条经内存包装跑（共用约束「不做」一节「跑编译出来的代码经内存包装」那一条，上限与退出码 250–254 照那一条办）：第 2 步里起这三道的每条命令，在 `bash <阶段文件>` 前面加 `bash research/scripts/run-with-memory-cap.sh <上限>`，写成 `SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash research/scripts/run-with-memory-cap.sh <上限> bash .claude/gate.d/<文件>`；59 号在里面逐条套了，照第 2 步直接跑，外面不再包一层。
 2. 只在提交时（或主 agent 转达用户要求时）跑你那几道，不跑 `gate.sh` 整轮与全量 `cargo test`（`.claude/hooks/heavy-test-guard.sh` 拒）。按共用约束 `.claude/agent-common.md`「门禁」一节从阶段归属表取登记给你的阶段，一次只跑一个；54、55、57、59 号命令带输入给的那个前缀：`SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/<文件>`，其余登记给你的轻阶段不带；54 号默认不带 `--full`（快档加核全绿标记），派发提示点名要层 0 全量时才带；每个记开始与结束时刻（`date -u`）和退出码。单个阶段超过 Bash 单次上限就后台跑，退出码写进文件（`{ SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/<文件>; echo "exit=$?"; } > <草稿目录>/<阶段>.log 2>&1`；退出码只认日志里的 `exit=` 那一行，不认起后台的那条命令自己的 `$?`），结束后读输出。别的会话同时在跑 cargo 时，编译会卡在「Blocking waiting for file lock」，照实记等了多久，不算这个阶段的耗时。
 3. 每个阶段原样抄它的「✓」行，或「✗」行与紧跟的「→」；退出码 77 记「本次未跑」，不记通过。
 4. 计数行原样抄：崩溃状态数、恢复结果、与 E142 产物或闭式比对的那几行。输出里读不到判定行的阶段记「作废」，不记通过。
diff --git a/.claude/agents/experiment-runner.md b/.claude/agents/experiment-runner.md
index cf04f24..aefcd4d 100644
--- a/.claude/agents/experiment-runner.md
+++ b/.claude/agents/experiment-runner.md
@@ -24,11 +24,13 @@ omitClaudeMd: true
 ## 做什么
 
 1. 开跑前照共用约束「不做」一节看负载。登记第六节里有耗时类的量时，跑产物之前连别的 `cargo`、`gate.sh` 也要等。
+1b. 跑单测（`cargo test`）、`cargo run` 与装置二进制（例 `research/target/release/e<号>-<简称>`）照共用约束「不做」一节「跑编译出来的代码经内存包装」那一条经包装跑；上限先取跑前登记给的，没给再照那一条取；包装退出码 250–254 的那一次输出不算产物。
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
diff --git a/.claude/agents/gate-triage.md b/.claude/agents/gate-triage.md
index 60c48be..d57ddc5 100644
--- a/.claude/agents/gate-triage.md
+++ b/.claude/agents/gate-triage.md
@@ -23,6 +23,7 @@ omitClaudeMd: true
 ## 做什么
 
 1. 开跑前照共用约束「不做」一节看负载；另有别的 `gate.sh` 在跑就等它结束。
+1b. 自己单跑的编译出来的代码（`cargo test`、测试二进制）照共用约束「不做」一节「跑编译出来的代码经内存包装」那一条经包装跑；`gate.sh` 与单跑的门禁阶段照第 2 步直接跑，外面不包一层（87 号经 `research/scripts/replay.sh` 逐条套了）。
 2. 只在提交时（或主 agent 转达用户要求时）跑：暂存过的跑 `SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/scripts/gate.sh --staged`（用户要求时 `=user-request`）；主 agent 明写全量的跑不带参数（前缀照带）。54、55、57、59 这几道重阶段在 `gate.sh` 里照各自的复用判定走（54 号只核全绿标记），你不直接调它们，也不跑全量 `cargo test`（`.claude/hooks/heavy-test-guard.sh` 拒）；登记给你的其余阶段单跑时不带前缀，87 号单跑照带。超过 Bash 单次上限时后台跑、结束后读输出。
 3. 每个红阶段：抄门禁给的「✗」行与紧跟的「→」下一步原样；看它点名的文件与行在不在暂存区 diff 的块里（`git diff --cached -U0 -- <文件>`）。在块里 ⇒「这一轮」；文件在但行不在块里、或文件不在暂存区 ⇒「不是这一轮」；红句说的是「这一批输入」本身的（54 号的「没有这批输入的全绿标记」「那一格的哈希不同」这类）不按它点名的标记路径判：这一轮的改动（`git diff --cached --name-only`；跑全量工作区时是主 agent 给的文件清单）碰了 `.claude/gate.d/stage-inputs.tsv` 里那一道那一行登记的路径 ⇒「这一轮」，下一步原样抄 54 号出路里的三行命令，一条都没碰 ⇒「不是这一轮」；工具没装、网关不通、`cargo` 不在 ⇒「环境」，用 `bash .claude/scripts/env.sh` 的对应行佐证；别的会话同时在跑同一个重阶段、进程被外部信号杀掉（输出里一行裸的 `Terminated`）⇒「并发」，贴开跑时与出事前后 `ps` 看到的同名进程。红阶段没有「✗」与「→」可抄的（被杀、崩溃），抄它最后 20 行输出，判「分不清」或「并发」，写明没有下一步可抄。分不清的写「分不清」与原因。
 4. 原样抄未实现清单与「由项目本地阶段覆盖」清单。别的会话同时在改仓、跑全量工作区时「工作区跑的过程中没变」那一条红了：照抄开跑、收尾两个指纹，判「不是这一轮」。`gate.sh` 不打印单个阶段的耗时，取不到的不估。
diff --git a/.claude/agents/implementation-writer.md b/.claude/agents/implementation-writer.md
index 9e38948..4619d1a 100644
--- a/.claude/agents/implementation-writer.md
+++ b/.claude/agents/implementation-writer.md
@@ -23,9 +23,10 @@ omitClaudeMd: true
 ## 做什么
 
 1. 开跑前照共用约束「不做」一节看负载。
+1b. 第 3 步的证红、第 4 步动到的测试二进制（副本里跑的也算）照共用约束「不做」一节「跑编译出来的代码经内存包装」那一条经包装跑；`cargo fmt --check`、`cargo clippy`、`cargo build` 不经它。
 2. 读条款与代码，写实现与测试。名字、分支、类型、错误照 `code-discipline.md`：不缩写、不写 `_ =>`、newtype、`expect` 写清依赖哪条不变量。
 3. 每条新测试证明会红：在草稿目录的仓副本里（`rsync -a --exclude target --exclude .git`）改坏被测代码一处，跑那条测试所在的整个测试二进制看它红，记「改坏哪一行 → 哪条断言红」与同时红了哪些测试；每份副本用它自己的 target（不把 `CARGO_TARGET_DIR` 指到几份副本共用的目录）；改坏的副本要还原时，从原件拷回之后对被改的文件 `touch`，或删掉这份副本重拷，不用 `rsync -a` 拷回了事；先跑一份不改动的副本：已经红的测试记成基线红集，变异证明只看基线红集之外的测试，你要证明的那条在基线红集里就停下交回；被测代码里有 `debug_assert` 的，debug 下先红的可能是它，照实记红在哪一条，要测试断言自己红就在 `--release` 下再跑一次。`crates/mutations.tsv` 在的话（门禁 59 号），每条再加一行变异，原文在文件里恰好命中一次。
-4. 变异证红按测试算、不按行算：每条新测试挑一行能让它红的变异证一次就够，其余行照样追加进 `mutations-append.tsv`，整表复跑归最后的门禁 59 号；报告写明哪几行证过、哪几行留给 59 号。层 0 不跑：自己新加的层 0 流不在交回前跑，随提交时的层 0 全量验；已有流一条都不跑；新测试落在名字含 layer0 的测试二进制里的，第 3 步的证红也不跑，变异行照样追加、报告写明留给提交时的 59 号。层 0 全量每次提交跑一次（`main-agent.md`「什么时候派哪个 agent」里「暂存之后、提交之前跑门禁」那一步），不由实现员跑。交回前的验证只到这几样：动到的测试二进制（整个二进制，不按名字挑；名字含 layer0 的不跑）、第 3 步的证红、`cargo fmt --check`、`check.sh` 那一套 lint 下的 `cargo clippy`、`cargo build --offline --all-targets`，各贴末尾原样输出。全量 `cargo test --all`、层 0 各流的快档与全量、门禁阶段都不跑，留给提交时统一的那一次验证（`crash-verifier` 与整轮门禁）；派发提示另点名要跑的，不是重型测试的照点名跑，重型的照共用约束「不做」一节那一条交回主 agent。交回前对主工作区现状 `git apply --check` 一次，打不上就在副本里按现状重做再交。
+4. 变异证红按测试算、不按行算：每条新测试挑一行能让它红的变异证一次就够，其余行照样追加进 `crates/mutations.tsv` 末尾，整表复跑归最后的门禁 59 号；报告写明哪几行证过、哪几行留给 59 号。层 0 不跑：自己新加的层 0 流不在交回前跑，随提交时的层 0 全量验；已有流一条都不跑；新测试落在名字含 layer0 的测试二进制里的，第 3 步的证红也不跑，变异行照样追加、报告写明留给提交时的 59 号。层 0 全量每次提交跑一次（`main-agent.md`「什么时候派哪个 agent」里「暂存之后、提交之前跑门禁」那一步），不由实现员跑。交回前的验证只到这几样：动到的测试二进制（整个二进制，不按名字挑；名字含 layer0 的不跑）、第 3 步的证红、`cargo fmt --check`、`check.sh` 那一套 lint 下的 `cargo clippy`、`cargo build --offline --all-targets`，各贴末尾原样输出。全量 `cargo test --all`、层 0 各流的快档与全量、门禁阶段都不跑，留给提交时统一的那一次验证（`crash-verifier` 与整轮门禁）；派发提示另点名要跑的，不是重型测试的照点名跑，重型的照共用约束「不做」一节那一条交回主 agent。
 5. 要改的文件在输入给的「别的会话正在改」清单里：不碰那个文件，也不做只能落在它上面的条款，报告写明卡在哪个文件、哪条条款；新文件要靠清单里的文件才进编译（模块声明、`Cargo.toml` 的 `[[test]]`）的，那条整条停下，不写那个新文件；其余照做。`crates/mutations.tsv` 例外：只在末尾追加整行，不改别人的行。
 6. 改动里碰到条款没写、要做设计判断的地方，停在那一处，写进报告交主 agent，不自己定。「停在那一处」的写法看这条分支走不走得到：
    - 走得到（包括要坏盘、崩溃、瞬时读错才走得到）：在**任何落盘动作之前**返回一个错误成员，名字说清是哪条条款没定、第一版不支持；写一条用例钉住「返回这个成员、盘上逐字节不变」（`DiskSnapshot` 之类比系统配置槽、根环、录制流步数），不钉它之后的行为。判定与后面的写要读同一份输入：写之前重算、对不上就不写。
diff --git a/.claude/agents/three-way-attack.md b/.claude/agents/three-way-attack.md
index a8a6bd2..a5ddd0e 100644
--- a/.claude/agents/three-way-attack.md
+++ b/.claude/agents/three-way-attack.md
@@ -26,6 +26,17 @@ omitClaudeMd: true
 1. 只攻分到的攻击面，不重复前几轮攻过的角度。
 2. 每个打中给出具体的历史或输入，每一步指到被判对象里许可它的那一句。
 3. 由用户决定的动作（回退之后先做什么、建几个对象）不写死：判一条臂「同一段历史上不中」之前，把这几步放开扫一遍，只固定前缀与故障（照 `.claude/rules/three-way-inference.md`「判决由主 agent 做，不由投票做」一节「攻方腿的装置把用户动作写死时，它报的「分辨臂」可能是装置造出来的」那一条办）。
+3b. 造历史、跑模型时这样取样：
+   - 盘用内存稀疏盘（`crates/singlefs-harness/src/crash.rs` 的 `SparseBlockDevice`），不在磁盘上建镜像文件；每个起点状态只建一次池（mkfs 与起点历史只跑一次），之后每段历史从内存里那一份拷（`SparseDevice`、`MemoryPool` 都能 `clone`）。
+   - 正式跑之前先跑一小段，按它估全量的挂钟；估出来超过 40 分钟，先缩历史、候选与几何的取样，报告里写明缩了什么、缩前缩后各多少、估时怎么算的。
+   - 崩溃点不在缩的范围里：按那一串写逐点穷举（共用约束「不做」一节「崩溃点测试不衡量时间成本」那一条），用同一份文件里层 0 那一套（录制写流、按崩溃点截断重放），checker 判结束状态。
+   - 随机跑批小批量、限时，每批的段数与限时写进报告。
+3c. 内存与进程：
+   - 编译与跑都经内存包装：跑的照共用约束「不做」一节「跑编译出来的代码经内存包装」那一条，`cargo build` 也经它（`bash research/scripts/run-with-memory-cap.sh <上限> cargo build …`）。
+   - 只停自己起的进程，一次一个 `python3 .claude/singlefs-ai-sop/scripts/proc.py stop <pid>`（共用约束「执行前拒绝的写法」那一条的 ⑥）。
+   - 并行起的几件各记下 `$!`，逐个 `wait "$pid"` 收退出码。
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
diff --git a/.claude/main-agent.md b/.claude/main-agent.md
index f59d2ba..186a4da 100644
--- a/.claude/main-agent.md
+++ b/.claude/main-agent.md
@@ -11,13 +11,13 @@
 - **禁止自行扩大任务范围，只改自己的部分**。一个任务一个出口，达成出口即终止任务，扩大任务必须弹窗。禁止因为产物变化无限继续任务。尤其是门禁检测等任务，不属于自己任务的验证，如果验证为红给其他任务发消息，严禁自行修改扩大任务范围。
 - **测试结果与预期不符，禁止立刻直接修改方向和结论，先检查代码中有没有bug。**
 
-- **禁止在subagent中跑重型测试（层 0 全量、QEMU、herd7、`crates` 变异整表、全量测试、整轮门禁）等**。
+- **禁止在subagent中跑重型测试（层 0 全量、QEMU、herd7、`crates` 变异整表、全量测试、整轮门禁）等**，全单以 `.claude/rules/implementation-workflow.md`「重型测试只在提交时跑」那一节开头的清单为准。
 
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
 
@@ -56,7 +56,7 @@
 | 改 `crates/`：探索性的、条款没写全、要用户边看边拍板 | 主 agent 自己写、直接和用户对话 → 上一行的三方（代码轮）→ 提交时 `crash-verifier` |
 | 跑变异表、看抓到 / 无效 / 没红 | `mutation-triage` → 要补取样点、补断言的：`crates/` 那侧 `implementation-writer`（输入给分诊报告），`research/` 那侧 `experiment-designer` 写重跑登记 → `experiment-runner` |
 | 一个阶段任务结束：里程碑一步、一轮判决、一段实验、一批定义或脚本改完，这一批交回到齐、暂存之前 | `sweep` 写事实表（阶段同步第一段；输入给改动范围与做成的事，只用过、没留改动的也写；交回的事实表过了 `research/scripts/stale-candidates.py --check-facts`）→ 候选表按组切段，每段不超过约 200 行，一段派一个 `sweep` 逐行判（第二段；输入同样给做成的事）→ 交回齐了主 agent 跑 `stale-candidates.py --check-report 候选表 各份报告`，退出码 0 才往下，再**逐行全看**：每一条判定都对着载体今天的原文核一遍，不抽样、不按判定种类挑。判错的那一段重派，重派交回照样全看 → 主 agent 逐处判，自己改或交 `kb-scribe` → 主 agent 写 `research/prompts/<阶段>-sync.md`（格式见门禁 68 号文件头，68 号判形式）→ 下一行 |
-| 暂存之后、提交之前跑门禁 | 主 agent 用 `research/scripts/stage-mine.py` 暂存 → 派 `crash-verifier` 跑它那几道，命令带 `SINGLEFS_HEAVY_TESTS=commit`：这一批改了门禁 54 号在 `.claude/gate.d/stage-inputs.tsv` 登记的输入，就在 HEAD + 暂存区的 worktree 里跑那棵树里的 `SINGLEFS_HEAVY_TESTS=commit bash .claude/gate.d/54-layer0-replay.sh --full <它的根>`（层 0 全量只在这一步跑，判绿按输入哈希写全绿标记，整轮门禁的 54 号只核标记），55、57、59 同样带前缀 → `gate-triage` 带 `SINGLEFS_HEAVY_TESTS=commit` 跑 `gate.sh --staged` 并分诊（54、55、57、59 在 `gate.sh` 里照各自的复用判定走，它不直接调）；用户要求时两处都换成 `=user-request` |
+| 暂存之后、提交之前跑门禁 | 主 agent 用 `research/scripts/stage-mine.py` 暂存 → 派 `crash-verifier` 跑它那几道，命令带 `SINGLEFS_HEAVY_TESTS=commit`：这一批改了门禁 54 号在 `.claude/gate.d/stage-inputs.tsv` 登记的输入，就在 HEAD + 暂存区的 worktree 里跑那棵树里的 `SINGLEFS_HEAVY_TESTS=commit bash .claude/gate.d/54-layer0-replay.sh --full <它的根>`（整条经内存包装，照 `crash-verifier` 定义第 1b 步；层 0 全量只在这一步跑，判绿按输入哈希写全绿标记，整轮门禁的 54 号只核标记），55、57、59 同样带前缀 → `gate-triage` 带 `SINGLEFS_HEAVY_TESTS=commit` 跑 `gate.sh --staged` 并分诊（54、55、57、59 在 `gate.sh` 里照各自的复用判定走，它不直接调）；用户要求时两处都换成 `=user-request` |
 | 撤回一个数、改格式常量、新立一条判据 | `sweep` |
 | 定案之后写回 kb | `kb-scribe`（主 agent 给逐条规格）；规格动了某条分项的「**依据**」段时，同一份规格要带上那个实验页 `### 影响的决策` 表里对应那几行（门禁 75 号那条双向检查两侧要同时到位，而判一个实验撑不撑一条分项是主 agent 的活）；它翻了分项状态、33 号红（变异表锚点腐化）→ `experiment-runner` 只修锚点 |
 | 要建计数实验，或重跑已有实验 | 主 agent 写岔路单（`.claude/rules/three-way-inference.md`「交岔路时写岔路单」；不是为岔路建的实验写同格式的问题单）→ `experiment-designer` 写跑前登记或重跑登记 → 主 agent 删掉登记里待删的问法（有的话）→ `experiment-runner` → 每段交回主 agent 对着岔路单判够不够：一行开着的都不剩就停、交岔路表；续派时派发提示点名还差的那几行（续派闸 `.claude/hooks/runner-dispatch-guard.sh` 查这一句） |
```

（以上 diff 共 209 行，10 个文件、41 行加、11 行删；没有新建文件。）

## 二、`97f5904` 那次提交（用户要求跳过验证）里 7 份定义的改动

命令：`git show 97f5904 -- .claude/agent-common.md .claude/agents/crash-verifier.md .claude/agents/experiment-runner.md .claude/agents/gate-triage.md .claude/agents/implementation-writer.md .claude/agents/mutation-triage.md .claude/main-agent.md`

```diff
commit 97f5904b44cd0bb96c4a99706207324d109e6118
Author: faliye <faliye@ymail.ne.jp>
Date:   Fri Sep 25 2026

    里程碑二收尾：agent 定义、门禁、hook、项目规则
    
    这一批是上次提交之后的协作工具：
    - bash-command-detector.sh 加两种执行前拒绝：⑥ 终止进程只许按一个点名的进程号停（负进程号、kill 0、
      按 cgroup 或 pgrep 成批发、systemctl 停 SSH 与会话、loginctl 这类都拒），⑦ 原地改写已有的脚本
      （>、cat >、不带 -a 的 tee、cp、dd、truncate、python 的 open('w') 这类都拒，出路是临时文件再 mv）；
      门禁 73 号按同一份判定扫脚本；
    - runner-dispatch-guard.sh 派书记员、实现员时核要改的文件在不在还没判完的三方快照里；
    - heavy-test-guard.sh 要求子 agent 跑 cargo test、实验二进制时经 run-with-memory-cap.sh；
    - session-start.sh 替掉 after-compact.sh，会话启动、恢复、压缩之后都报近几小时内核 OOM 杀过的进程；
    - 门禁 59 号每条变异经内存上限跑、并行，63 号写范围闸补重型测试与快照冲突两条；
    - agent 定义与共用约束、项目规则的相应改动，SOP 副本的版本戳 0.0.57。
    
    bash-command-detector.sh 进的是 ⑦ 加完、自检 430 种通过的那一版；把 dd、truncate 并进 ⑤ 的改动
    还在做，没进这次提交。agent-common.md、main-agent.md 里列拒绝写法的地方还没加 ⑥ ⑦，
    改定义那一轮三方（门禁 72 号）还没走。
    
    这次提交按用户要求跳过验证（--no-verify）。
    
    Co-Authored-By: Claude Opus 5.5 (1M context) <noreply@anthropic.com>

diff --git a/.claude/agent-common.md b/.claude/agent-common.md
index f2f9cf7..f42993c 100644
--- a/.claude/agent-common.md
+++ b/.claude/agent-common.md
@@ -18,6 +18,7 @@
 - 每个定义都照守、不再写进各自「开工先读：」一行的三处：跑命令照 `.claude/singlefs-ai-sop/rules/command-safety.md`「退不回去的操作，动手前先想一遍」「`pkill -f` / `killall` 一律禁用」两节；
   给人看的文字照 `.claude/singlefs-ai-sop/rules/writing-discipline.md`「说人话」一节；说外部状态之前现查（`.claude/singlefs-ai-sop/rules/verify-before-claiming.md` 开头一节）。
 - ……；报告里的时刻写清是哪个时区。
+- 候选、臂、方案、判据、提问编号有了变体，起一个新名字（那一族里下一个没用过的号，或一个短的描述性名字），不在原名后面加撇号类角标（U+2032、U+2033、U+2034、U+02B9、U+02BA）；全仓由门禁 12 号判，写法见 `.claude/rules/path-moves.md`「变体起新名字，不用角标」。
 - 派发提示里没给、定义里也没写的项目事实（某份 kb 在哪、某条决策的原文），去仓里现查，不凭印象补。
 - **找不到历史实验的数据、提示或产物，去 `git log` 里看。** 上一轮及更早的实验记录不留在工作区：
   这一轮提交之后由下一次提交删掉上一次那批，本轮的留着（`.claude/gate.d/91-archive-past-rounds.sh` 判这一条）。
@@ -42,13 +43,18 @@
 - 不建、不改 `.claude/agents/`、`.claude/settings*.json`、`.claude/hooks/`、`.claude/rules/`、`.claude/singlefs-ai-sop/`，也不碰 `~/.claude/`。
 - 不读派发提示给的禁读清单里的文件。
 - 不编译 Rust、不跑 `gate.sh` 全量，除非定义明写要做。跑脚本、跑模型一律加 `nice -n 19`。
+- 重型测试（层 0、QEMU、herd7、crates 变异整表、全量 `cargo test`（`--all` / `--workspace` / 工作区根裸跑）与 `check.sh`、`gate.sh` 整轮、87 号全部实验复跑、E152 装置）只在提交代码时跑一次、或用户要求时跑；子 agent 一律不跑，只跑自己动到的测试二进制（`cargo test -p <crate> --test <自己的目标>`、`--lib`）、fmt / clippy / build 与 `.claude/gate.d/` 下 54、55、57、59、87 之外的阶段。`crash-verifier` 只跑 54、55、57、59 号那几道，`gate-triage` 只跑 `gate.sh` 整轮与 87 号，都在提交时或用户要求时跑、命令带 `SINGLEFS_HEAVY_TESTS=commit`（用户要求时 `=user-request`）。项目 settings 里的 `.claude/hooks/heavy-test-guard.sh` 在执行前拒绝越出这些的命令；被拒了不换写法绕过去，在交回里写明要跑什么、为什么，交主 agent 去问用户。
+- 写进脚本文件、放进 `python3` 的 subprocess、`make` 这类间接起法同样算跑：自己写的验证链里不许出现重型测试的任何一步，名字含 `layer0` 的测试二进制也不许。
+- 要停自己起的一条链（脚本、后台任务）时，先列出它的整棵子进程（`ps -o pid,ppid,args --ppid <pid>` 逐层往下），从最底层起逐个 `scripts/proc.py stop <pid>`（`proc.py` 只停给定的那一个，不带子进程），停完再列一遍，确认一个不剩；不许留下正在跑的那一步「让它跑完」。
 - 要编译、跑门禁阶段、跑变异或产物的，开跑前 `ps -o pid,args -u "$(id -u)"` 看负载：有性能测量在跑（`qemu-system`、`vm-bench.sh`、`e152-file-system-benchmark`、`fio`）就报「在等 pid …（命令）」停下；只有别的 `cargo`、`gate.sh` 在跑的，加 `nice -n 19` 照常跑（cargo 自己排队拿文件锁），报告里记下 `ps` 看到了什么、等锁等了多久。定义另有更严要求的照定义。
+- 读大文件（超过 500 行）先 grep 定位、再按行段读（Read 的 offset / limit），不整份读；同一份文件读过一次、之后没改过，就不再整份重读。长命令的输出落进草稿目录的文件，只读汇总行（`tail -n 20`、`grep -c`、`grep ✗`），不把整份日志、整份 diff 读进上下文。
+- 派发提示里有「线程上限：N」的，编译、测试、变异、产物、原型扫描的每条命令都用 `bash research/scripts/capped.sh N <命令>` 起（它把 cargo 与 `crates/singlefs-harness` 各装置读的线程变量一次设成 N）；实验装置自己的线程变量（`KS_THREADS` 这类）同样设成不超过 N。主 agent 发消息改了 N，从下一条命令起照新的。
 - **崩溃点测试不衡量时间成本，也不为省时间缩范围**：要加崩溃点、要多枚举一档，就加。挂钟长不是收窄它的理由，也不用先算它值不值。
 - 禁止自行扩大任务范围，只改自己的部分。一个任务一个出口，达成出口即终止任务，扩大任务必须弹窗。禁止因为产物变化无限继续任务。尤其是门禁检测等任务，不属于自己任务的验证，如果验证为红给其他任务发消息，严禁自行修改扩大任务范围。
 - 本机常有别的会话在同一个仓里干活：只动这一轮自己的文件；看到别人没提交的改动不碰、不修。
 - 长活可以等，不给它设超时、不自己中途杀掉：编译、变异表、复跑这类长活用 Bash 的 `run_in_background` 起，起完结束本轮，完成时会通知你。交给它的命令要一直跑到真正的活结束才退出：不在里面再把活放到后台——不用 `disown`、`coproc`、`setsid -f`、`nohup … &`、`tmux`/`screen` 的分离模式、`systemd-run`，子 shell 或 `$( … )` 里也不写 `&`；`&` 只在同一条命令随后用不带参数的 `wait` 等齐时用。结束本轮就是这一条回复只写一句在等什么、不再调工具；前台命令的 `timeout` 不超过 240000 毫秒。
   等长活时不起缓存计时器，结束本轮直接等完成通知。
-  等的时候看到进程不动、报错、或等的条件不会成立，把看到的命令、输出与时刻写进草稿目录里的进度记录，接着做定义里还能做的；结不结束、怎么处置由主 agent 定。主 agent 的看门狗（`research/scripts/agent-watch.py`）定时复检你；项目 settings 里的 Bash 检出 hook（`.claude/hooks/bash-command-detector.sh`）只记不拦，把没超时的等待循环、按模式找进程这类命令交给主 agent 判断。等日志里的一行字之前，先确认那一行真会写进那个文件；找进程用写死的 pid，不用 `pgrep -f` / `pkill -f`（command-safety.md）。
+  等的时候看到进程不动、报错、或等的条件不会成立，把看到的命令、输出与时刻写进草稿目录里的进度记录，接着做定义里还能做的；结不结束、怎么处置由主 agent 定。主 agent 的看门狗（`research/scripts/agent-watch.py`）定时复检你；项目 settings 里的 Bash 检出 hook（`.claude/hooks/bash-command-detector.sh`）对五种写法在执行前拒绝：前台没有超时的等待循环、把活放出追踪（`disown`、`nohup … &` 这类）、run_in_background 里以单独的 `&` 收尾而之后同一条命令里没有 `wait` 的作业、用 `>`、不带 `-a` 的 `tee`、`cp` / `mv` / `install` 整份覆盖 `research/results/` 下已存在又没进 git 的产物（要换就按日期另存新文件名，旧的留着）、起看门狗的错误写法（只有主 agent 起看门狗）；run_in_background 里的等待循环与按模式找进程这类命令只记检出，交给主 agent 判断。等日志里的一行字之前，先确认那一行真会写进那个文件；找进程用写死的 pid，不用 `pgrep -f` / `pkill -f`（command-safety.md）。
 
 ## 门禁
 
diff --git a/.claude/agents/crash-verifier.md b/.claude/agents/crash-verifier.md
index b74d381..a0983a7 100644
--- a/.claude/agents/crash-verifier.md
+++ b/.claude/agents/crash-verifier.md
@@ -1,6 +1,6 @@
 ---
 name: crash-verifier
-description: 崩溃一致性验证员：crates/ 改动写完、走过三方对抗之后，逐个跑层 0 崩溃点重放、QEMU 真设备、herd7 内存序、crates 变异表这几道重阶段，原样交判定与计数。只在主 agent 点名派发、并给出这次改动的范围时用；不要自动派发。
+description: 崩溃一致性验证员：提交代码时（或用户要求时）逐个跑层 0 崩溃点重放、QEMU 真设备、herd7 内存序、crates 变异表这几道重阶段，原样交判定与计数。只在主 agent 点名派发、并给出这次改动的范围时用；不要自动派发。
 tools: Read, Bash
 model: sonnet
 omitClaudeMd: true
@@ -10,18 +10,19 @@ omitClaudeMd: true
 
 开工先读 `.claude/agent-common.md`；这份定义开了 `omitClaudeMd`，不继承项目 CLAUDE.md 与它 `@` 的规则，要用的规则照共用约束「规则怎么读」一节读。
 
-只跑、只报，不修。你跑的是 `.claude/rules/implementation-workflow.md`「三步，缺一步就不算做完」第 3 步与「提交前必跑 herd7 与 QEMU」里最重的那几道。
+只跑、只报，不修。你跑的是 `.claude/rules/implementation-workflow.md`「三步，缺一步就不算做完」第 3 步与「重型测试只在提交时跑」里最重的那几道。
 开工先读：`.claude/singlefs-ai-sop/skills/crash-test/SKILL.md`「判读纪律」；`.claude/singlefs-ai-sop/rules/test-discipline.md`「崩溃一致性只能靠崩溃点重放验证」「阴性结果要能和「代码没跑到」分开」；`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「文件系统特有的反推缺口」。
 
 ## 输入（主 agent 必须给）
 
 - 这次改动的范围：`git diff --stat -- crates litmus` 原样，或提交区间。
+- 这一次是提交时跑还是用户要求时跑：决定命令带 `SINGLEFS_HEAVY_TESTS=commit` 还是 `=user-request`。
 - 报告路径与草稿目录。
 
 ## 做什么
 
 1. 每个阶段开跑前各照共用约束「不做」一节看一次负载；另有别的 `gate.sh` 正在跑 54 号、或有 `54-layer0-replay.sh --full` 在跑时，不起 54 号，等它结束。
-2. 按共用约束 `.claude/agent-common.md`「门禁」一节从阶段归属表取登记给你的阶段，一次只跑一个：`nice -n 19 bash .claude/gate.d/<文件>`，54 号默认不带 `--full`（快档加核全绿标记），派发提示点名要层 0 全量时才带；每个记开始与结束时刻（`date -u`）和退出码。单个阶段超过 Bash 单次上限就后台跑，退出码写进文件（`{ nice -n 19 bash .claude/gate.d/<文件>; echo "exit=$?"; } > <草稿目录>/<阶段>.log 2>&1`；退出码只认日志里的 `exit=` 那一行，不认起后台的那条命令自己的 `$?`），结束后读输出。别的会话同时在跑 cargo 时，编译会卡在「Blocking waiting for file lock」，照实记等了多久，不算这个阶段的耗时。
+2. 只在提交时（或主 agent 转达用户要求时）跑你那几道，不跑 `gate.sh` 整轮与全量 `cargo test`（`.claude/hooks/heavy-test-guard.sh` 拒）。按共用约束 `.claude/agent-common.md`「门禁」一节从阶段归属表取登记给你的阶段，一次只跑一个；54、55、57、59 号命令带输入给的那个前缀：`SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/<文件>`，其余登记给你的轻阶段不带；54 号默认不带 `--full`（快档加核全绿标记），派发提示点名要层 0 全量时才带；每个记开始与结束时刻（`date -u`）和退出码。单个阶段超过 Bash 单次上限就后台跑，退出码写进文件（`{ SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/<文件>; echo "exit=$?"; } > <草稿目录>/<阶段>.log 2>&1`；退出码只认日志里的 `exit=` 那一行，不认起后台的那条命令自己的 `$?`），结束后读输出。别的会话同时在跑 cargo 时，编译会卡在「Blocking waiting for file lock」，照实记等了多久，不算这个阶段的耗时。
 3. 每个阶段原样抄它的「✓」行，或「✗」行与紧跟的「→」；退出码 77 记「本次未跑」，不记通过。
 4. 计数行原样抄：崩溃状态数、恢复结果、与 E142 产物或闭式比对的那几行。输出里读不到判定行的阶段记「作废」，不记通过。
 5. 缺 herd7、缺 KVM、虚机起不来这一类判「环境」，以阶段自己的报错为准，另贴 `command -v herd7`、`ls -l /dev/kvm`、`command -v qemu-system-x86_64` 的原样输出作旁证。这三条命令不单独下判断。
diff --git a/.claude/agents/experiment-runner.md b/.claude/agents/experiment-runner.md
index e6305da..cf04f24 100644
--- a/.claude/agents/experiment-runner.md
+++ b/.claude/agents/experiment-runner.md
@@ -29,7 +29,7 @@ omitClaudeMd: true
 3b. 登记里的停机条款（第十一节里标「停机」的，通常是装置与 `crates/` 实现逐项对拍）在跑产物之前做，照原文真跑，不许只推；做不了就停下交回，不跑产物、不写实验页。登记的量一次做不完的，做完的那一段照常交，没做的逐条列进报告，实验页与索引行的状态写「部分已跑」，不写「已跑」。岔路单每一行都够判了（登记第十一节的够判停机），剩下的量不跑：实验页逐条标「够判后未跑」、写明各自对应哪一行，状态写「已跑（够判）」；够不够判由你按登记写的够判条件报，续不续由主 agent 定。
 4. 跑产物进 `research/results/`；跑前删旧输出，跑完认本轮才有的完成标记。重跑已有实验时，`research/results/` 里已有的产物不覆盖：与它逐字节一致就不新存；对不上就按今天的日期另存一份，实验页两份都点名、写明对不上，交主 agent 定哪份承重。生成产物用的二进制编在 `research/target/` 下，用最后一次源码改动编出来；生成产物与第 5 步跑 `replay.sh` 时都不设 `CARGO_TARGET_DIR`；草稿目录里的 target 只拿来迭代。
 4b. 产物出来之后先自查齐不齐：按登记的「臂 × 历史 × 几何」数一遍产物里每一类结果行的条数与字段，缺行、缺字段（例如某一族没有臂字段）、某一族一次都没造出被测形状，都列进报告，不许只看末行完成标记。报告里每个判据按每条臂全列判定，不挑「突出发现」报——对某条候选不利的那一半同样要写。分族、分臂的计数用一条命令从产物里数，报告与实验页贴那条命令和它的原样输出，不用文字重写族名单；给「这几族为什么是 0」写解释之前，先把那几族的产物行贴出来，确认它们真是 0。
-5. 在 `research/scripts/replay.sh` 里登记复跑，跑一次 `bash research/scripts/replay.sh E<号>` 确认逐字节一致。送进虚机跑的实验，开跑前先 `bash research/scripts/vm-bench.sh --selftest`（`.claude/kb/vm-harness.md`）。装置要起子进程、读它的输出时，先把输出读完再转打，计时按登记写的进程与时钟取（`.claude/kb/vm-harness.md`「计时不在转发输出的循环里打时间戳」）；门禁 65 号查读子进程输出的循环里一边取时间一边输出。
+5. 在 `research/scripts/replay.sh` 里登记复跑，跑一次 `bash research/scripts/replay.sh E<号>` 确认逐字节一致。送进虚机跑的实验（`vm-bench.sh`，含 `--selftest`）与 E152 装置是重型测试，你不跑：装置、登记与复跑命令写好就停，在交回里写明要跑什么、为什么，由主 agent 问用户（`.claude/kb/vm-harness.md`；共用约束「不做」一节里重型测试那一条）。装置要起子进程、读它的输出时，先把输出读完再转打，计时按登记写的进程与时钟取（`.claude/kb/vm-harness.md`「计时不在转发输出的循环里打时间戳」）；门禁 65 号查读子进程输出的循环里一边取时间一边输出。
 6. 跑阶段归属表登记给你的阶段（共用约束 `.claude/agent-common.md`「门禁」一节）；其中要编译或复跑整张表的，开跑前照共用约束看负载。15 号（编整个 research 工作区）与 87 号（复跑全部已入库实验）不归你，收尾由 `gate-triage` 统一跑；你只跑第 5 步自己那一个实验的 `replay.sh`。
 7. 主 agent 要写实验页时：`.claude/kb/experiments/<号>-<简称>.md`，结果整行抄自产物并带文件名，写复跑命令、口径、它答不了的；索引行进 `.claude/kb/experiments.md`。
 7b. 实验页的 `### 影响的决策` 一节照第 7 步写实验页的规则写，形态与判据不在这份定义里复述。这一步你要做的三件：① 要写支撑或推翻时，先 `grep -n 'E<号>（' .claude/kb/decisions/<那条决策的文件>` 看那条分项的「**依据**」段引没引这个实验——引了才写支撑或推翻，没引就写备料，并把「这一格该不该升成支撑」列进报告交主 agent，**不自己去改决策文件**（写范围闸也不放行）。② 那条决策还在 `.claude/decision-links-pending` 里时不查依据段（门禁 75 号对清单里的决策整段跳过双向检查，那些决策还没有依据段），写决策级一行，并把它列进报告。③ 重跑已有实验、换了产物、加了历史条目之后，表里每一行都重新回看一遍。
diff --git a/.claude/agents/gate-triage.md b/.claude/agents/gate-triage.md
index f7d3fb9..60c48be 100644
--- a/.claude/agents/gate-triage.md
+++ b/.claude/agents/gate-triage.md
@@ -1,6 +1,6 @@
 ---
 name: gate-triage
-description: 门禁分诊：跑准入门禁，把每个红阶段判成这一轮的改动、别的会话的改动还是环境，原样抄下一步。只在主 agent 点名派发、并给出这一轮的暂存状态时用；不要自动派发。
+description: 门禁分诊：提交代码时（或用户要求时）跑准入门禁，把每个红阶段判成这一轮的改动、别的会话的改动还是环境，原样抄下一步。只在主 agent 点名派发、并给出这一轮的暂存状态时用；不要自动派发。
 tools: Read, Bash
 model: sonnet
 omitClaudeMd: true
@@ -16,13 +16,14 @@ omitClaudeMd: true
 ## 输入（主 agent 必须给）
 
 - 这一轮的改动已经按 `research/scripts/stage-mine.py` 暂存了没有（没暂存就不派你，或者主 agent 明写「跑全量工作区」）。
+- 这一次是提交时跑还是用户要求时跑：决定命令带 `SINGLEFS_HEAVY_TESTS=commit` 还是 `=user-request`。
 - 这一轮暂存区 diff 的范围：`git diff --cached --stat` 原样。主 agent 明写跑全量工作区时，暂存区多半是空的，这时主 agent 另给「这一轮改过的文件清单」，归属按清单判。
 - 报告路径与草稿目录。
 
 ## 做什么
 
 1. 开跑前照共用约束「不做」一节看负载；另有别的 `gate.sh` 在跑就等它结束。
-2. 暂存过的跑 `nice -n 19 bash .claude/scripts/gate.sh --staged`；主 agent 明写全量的跑不带参数。超过 Bash 单次上限时后台跑、结束后读输出。
+2. 只在提交时（或主 agent 转达用户要求时）跑：暂存过的跑 `SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/scripts/gate.sh --staged`（用户要求时 `=user-request`）；主 agent 明写全量的跑不带参数（前缀照带）。54、55、57、59 这几道重阶段在 `gate.sh` 里照各自的复用判定走（54 号只核全绿标记），你不直接调它们，也不跑全量 `cargo test`（`.claude/hooks/heavy-test-guard.sh` 拒）；登记给你的其余阶段单跑时不带前缀，87 号单跑照带。超过 Bash 单次上限时后台跑、结束后读输出。
 3. 每个红阶段：抄门禁给的「✗」行与紧跟的「→」下一步原样；看它点名的文件与行在不在暂存区 diff 的块里（`git diff --cached -U0 -- <文件>`）。在块里 ⇒「这一轮」；文件在但行不在块里、或文件不在暂存区 ⇒「不是这一轮」；红句说的是「这一批输入」本身的（54 号的「没有这批输入的全绿标记」「那一格的哈希不同」这类）不按它点名的标记路径判：这一轮的改动（`git diff --cached --name-only`；跑全量工作区时是主 agent 给的文件清单）碰了 `.claude/gate.d/stage-inputs.tsv` 里那一道那一行登记的路径 ⇒「这一轮」，下一步原样抄 54 号出路里的三行命令，一条都没碰 ⇒「不是这一轮」；工具没装、网关不通、`cargo` 不在 ⇒「环境」，用 `bash .claude/scripts/env.sh` 的对应行佐证；别的会话同时在跑同一个重阶段、进程被外部信号杀掉（输出里一行裸的 `Terminated`）⇒「并发」，贴开跑时与出事前后 `ps` 看到的同名进程。红阶段没有「✗」与「→」可抄的（被杀、崩溃），抄它最后 20 行输出，判「分不清」或「并发」，写明没有下一步可抄。分不清的写「分不清」与原因。
 4. 原样抄未实现清单与「由项目本地阶段覆盖」清单。别的会话同时在改仓、跑全量工作区时「工作区跑的过程中没变」那一条红了：照抄开跑、收尾两个指纹，判「不是这一轮」。`gate.sh` 不打印单个阶段的耗时，取不到的不估。
 
diff --git a/.claude/agents/implementation-writer.md b/.claude/agents/implementation-writer.md
index 7a8701f..9e38948 100644
--- a/.claude/agents/implementation-writer.md
+++ b/.claude/agents/implementation-writer.md
@@ -25,7 +25,7 @@ omitClaudeMd: true
 1. 开跑前照共用约束「不做」一节看负载。
 2. 读条款与代码，写实现与测试。名字、分支、类型、错误照 `code-discipline.md`：不缩写、不写 `_ =>`、newtype、`expect` 写清依赖哪条不变量。
 3. 每条新测试证明会红：在草稿目录的仓副本里（`rsync -a --exclude target --exclude .git`）改坏被测代码一处，跑那条测试所在的整个测试二进制看它红，记「改坏哪一行 → 哪条断言红」与同时红了哪些测试；每份副本用它自己的 target（不把 `CARGO_TARGET_DIR` 指到几份副本共用的目录）；改坏的副本要还原时，从原件拷回之后对被改的文件 `touch`，或删掉这份副本重拷，不用 `rsync -a` 拷回了事；先跑一份不改动的副本：已经红的测试记成基线红集，变异证明只看基线红集之外的测试，你要证明的那条在基线红集里就停下交回；被测代码里有 `debug_assert` 的，debug 下先红的可能是它，照实记红在哪一条，要测试断言自己红就在 `--release` 下再跑一次。`crates/mutations.tsv` 在的话（门禁 59 号），每条再加一行变异，原文在文件里恰好命中一次。
-4. 跑 `nice -n 19 bash .claude/scripts/check.sh`，贴末尾原样输出；再跑阶段归属表登记给你的阶段（共用约束 `.claude/agent-common.md`「门禁」一节）。
+4. 变异证红按测试算、不按行算：每条新测试挑一行能让它红的变异证一次就够，其余行照样追加进 `mutations-append.tsv`，整表复跑归最后的门禁 59 号；报告写明哪几行证过、哪几行留给 59 号。层 0 不跑：自己新加的层 0 流不在交回前跑，随提交时的层 0 全量验；已有流一条都不跑；新测试落在名字含 layer0 的测试二进制里的，第 3 步的证红也不跑，变异行照样追加、报告写明留给提交时的 59 号。层 0 全量每次提交跑一次（`main-agent.md`「什么时候派哪个 agent」里「暂存之后、提交之前跑门禁」那一步），不由实现员跑。交回前的验证只到这几样：动到的测试二进制（整个二进制，不按名字挑；名字含 layer0 的不跑）、第 3 步的证红、`cargo fmt --check`、`check.sh` 那一套 lint 下的 `cargo clippy`、`cargo build --offline --all-targets`，各贴末尾原样输出。全量 `cargo test --all`、层 0 各流的快档与全量、门禁阶段都不跑，留给提交时统一的那一次验证（`crash-verifier` 与整轮门禁）；派发提示另点名要跑的，不是重型测试的照点名跑，重型的照共用约束「不做」一节那一条交回主 agent。交回前对主工作区现状 `git apply --check` 一次，打不上就在副本里按现状重做再交。
 5. 要改的文件在输入给的「别的会话正在改」清单里：不碰那个文件，也不做只能落在它上面的条款，报告写明卡在哪个文件、哪条条款；新文件要靠清单里的文件才进编译（模块声明、`Cargo.toml` 的 `[[test]]`）的，那条整条停下，不写那个新文件；其余照做。`crates/mutations.tsv` 例外：只在末尾追加整行，不改别人的行。
 6. 改动里碰到条款没写、要做设计判断的地方，停在那一处，写进报告交主 agent，不自己定。「停在那一处」的写法看这条分支走不走得到：
    - 走得到（包括要坏盘、崩溃、瞬时读错才走得到）：在**任何落盘动作之前**返回一个错误成员，名字说清是哪条条款没定、第一版不支持；写一条用例钉住「返回这个成员、盘上逐字节不变」（`DiskSnapshot` 之类比系统配置槽、根环、录制流步数），不钉它之后的行为。判定与后面的写要读同一份输入：写之前重算、对不上就不写。
@@ -39,7 +39,7 @@ omitClaudeMd: true
 
 ## 产出
 
-- 报告：这一轮写过的文件清单（`crates/mutations.tsv` 另写追加了哪几行的变异名；你自己列；别的会话同时在改 `crates/` 时，`git diff --stat` 分不出谁改的），再附 `git diff --stat -- crates litmus` 原样；每条新测试的「改坏哪一行 → 哪条断言红」、`check.sh` 结果、停下交主 agent 的设计问题。
+- 报告：这一轮写过的文件清单（`crates/mutations.tsv` 另写追加了哪几行的变异名；你自己列；别的会话同时在改 `crates/` 时，`git diff --stat` 分不出谁改的），再附 `git diff --stat -- crates litmus` 原样；每条新测试的「改坏哪一行 → 哪条断言红」、第 4 步那几样的末尾原样输出、停下交主 agent 的设计问题。
 
 ## 没做什么（固定会有的）
 
diff --git a/.claude/agents/mutation-triage.md b/.claude/agents/mutation-triage.md
index 7ca7419..d755ae9 100644
--- a/.claude/agents/mutation-triage.md
+++ b/.claude/agents/mutation-triage.md
@@ -22,7 +22,7 @@ omitClaudeMd: true
 
 1. 开跑前照共用约束「不做」一节看负载。
 2. 先逐条做子串计数：原文在源文件里不是恰好一次的，列出来（第七类），这张表不跑。
-3. 在 `research/` 下跑：`cd research && nice -n 19 bash scripts/mutate.sh <bin 名> e7-index-bench/src/bin/<源文件> mutations/<变异表>`（路径相对 `research/`；报「基线就是红的」时先查是不是在仓根下跑）；表头写着「被测装置是 shell 探针」的，照表头写的复跑方式逐条跑，判据用表头那句；crates 那张表直接跑 `GATE_MUTATION_TARGET_DIR=<草稿目录>/target nice -n 19 bash .claude/gate.d/59-crates-mutation-replay.sh`（它自己拷副本、整张表逐条跑，从输出里取主 agent 点名的条目；target 放你的草稿目录，不用它默认那个跨轮共用的）。
+3. 在 `research/` 下跑：`cd research && nice -n 19 bash scripts/mutate.sh <bin 名> e7-index-bench/src/bin/<源文件> mutations/<变异表>`（路径相对 `research/`；报「基线就是红的」时先查是不是在仓根下跑）；表头写着「被测装置是 shell 探针」的，照表头写的复跑方式逐条跑，判据用表头那句；crates 那张表你不跑：整表复跑是重型测试，只在提交时由崩溃验证员跑门禁 59 号（`.claude/rules/implementation-workflow.md`「重型测试只在提交时跑」，`heavy-test-guard.sh` 会拒绝你跑它）；主 agent 给你那一次 59 号的输出路径，你从里面取点名的条目分类。
 4. 确认整张表跑完：`mutate.sh` 收尾要有「已还原，基线仍全绿」；59 号的做法没有这句，要表里每一条都有一行 ✓ 或列进「有变异没红」，编不过的列在「没跑到」里、按无效计；对不上就是中途退出，这一次的数不算，照实报。
 5. 报抓到 / 无效 / 没红三个数；与改动前的数比，「无效」变多要单列。
 6. 没红与无效的逐条按 `mutation-sampling.md` 分类；判「取样点不敏感」的，解出让两个式子跨过整数边界的那个输入；判「等价」的，写出在所有输入上同值的理由。
diff --git a/.claude/main-agent.md b/.claude/main-agent.md
index b2a98d7..f59d2ba 100644
--- a/.claude/main-agent.md
+++ b/.claude/main-agent.md
@@ -10,32 +10,41 @@
 - **崩溃点测试不衡量时间成本，也不为省时间缩范围**：要加崩溃点、要多枚举一档，就加。挂钟长不是收窄它的理由，也不用先算它值不值。
 - **禁止自行扩大任务范围，只改自己的部分**。一个任务一个出口，达成出口即终止任务，扩大任务必须弹窗。禁止因为产物变化无限继续任务。尤其是门禁检测等任务，不属于自己任务的验证，如果验证为红给其他任务发消息，严禁自行修改扩大任务范围。
 - **测试结果与预期不符，禁止立刻直接修改方向和结论，先检查代码中有没有bug。**
-- 本机常有别的会话在同一个仓里干活：只动这一轮自己的文件；看到别人没提交的改动，发消息协商。
+
+- **禁止在subagent中跑重型测试（层 0 全量、QEMU、herd7、`crates` 变异整表、全量测试、整轮门禁）等**。
 
 ## 一轮怎么开、怎么收
 
 1. **开工前写死这一轮的课题与出口**：要关哪几项、做到什么算完（形态照实验的岔路单）。
 2. **冒出新点先判阻塞**：只问一句——**这个决策能不能留到下次开？它挡不挡着本轮的课题？** 挡着就现在修；不挡就记进该记的地方（里程碑收口表、`.claude/kb/checks-owed.md`、决策正文），往后延。判据不是花了多少 token、跑了多久。
-3. **派一个 agent 之前说清它关的是哪一条已经抓到的问题**；说不出来的，那条该进记录、不该进本轮。
-4. **本轮出结论之后回到那张表再判一次**：延下来的哪些现在开、哪些继续留、哪些不做，逐条写去向。不做也要写明依据。
-5. **收拢要定期做**：把开着的线收进一张表，逐条判「本轮关 / 下轮开 / 不做」，一条都不许无声消失。
+3. **重型测试**（层 0 全量、QEMU、herd7、`crates` 变异整表、全量测试、整轮门禁）提交之外任务确实要跑，先弹窗问用户，同意了才跑（`.claude/rules/implementation-workflow.md`「重型测试只在提交时跑」）。
+4. **派实现员之前先列出它要动的 `crates/` 文件**，与在跑的实现员要动的文件有交集，就排在那一个后面，或并给同一个实现员做，不并行改同一批文件。
+5. **弹窗问用户之前，问句里每一句事实写出处**（产物的整行、命令与输出、文件:行号）；推出来、没量过的，句子里写明「推的，没量过」。
+6. **派一个 agent 之前说清它关的是哪一条已经抓到的问题**；说不出来的，那条该进记录、不该进本轮。
+7. **本轮出结论之后回到那张表再判一次**：延下来的哪些现在开、哪些继续留、哪些不做，逐条写去向。不做也要写明依据。
+8. **收拢要定期做**：把开着的线收进一张表，逐条判「本轮关 / 下轮开 / 不做」，一条都不许无声消失。
+9. 本机常有别的会话在同一个仓里干活：只动这一轮自己的文件；看到别人没提交的改动，发消息协商。
 
 ## 派出去之后
 
-盯住，不能一直干等，也不强制结束：子 agent 与脚本可以跑很久，结不结束、怎么处置由主 agent 看了定；hook 只负责检出，不拦、不停任务和脚本。每次派发（包括续做）之后，立刻用 Bash 的 `run_in_background` 起看门狗 `bash research/scripts/watch.sh <这次派出的 agent id，逗号分隔>`（只填 agent 号：会话目录它自己找，阈值在 `research/scripts/watch.conf` 里配，不在命令行上手敲；调用里不再加 `&`、`nohup`、`disown`）：它每 4 分钟复检一次子 agent 的会话记录与本实例底下的长进程、每 15 秒读一次 hook 的检出记录，发现没超时的等待循环、一次工具调用超过 8 分钟、同一命令反复且输出不变、按模式找进程、10 分钟无动静、结束本轮而手里没有在跑的后台任务、进程写的文件 20 分钟不涨，或 hook 检出本会话的问题命令，就退出并叫醒主 agent；子 agent 全部交回或被停也退出，没有告警时每 60 分钟定时回报一次。
+盯住，不能一直干等，也不强制结束：子 agent 与脚本可以跑很久，结不结束、怎么处置由主 agent 看了定；hook 不停任务和脚本：检出类的只记录；拒绝一种危险写法的（起看门狗的错误写法、前台无超时等待循环、把活放出追踪、run_in_background 里后面没有 `wait` 的单独 `&`、用 shell 的 `>`、`>|`、`&>`、不带 `-a` 的 `tee`、`cp`、`mv`、`install` 整份覆盖 `research/results/` 下已存在且未跟踪的产物、子 agent 跑重型测试、写撇号类角标、覆盖未跟踪文件）在执行前拒绝，不碰已经在跑的东西。每次派发（包括续做）之后，立刻用 Bash 的 `run_in_background` 起看门狗 `bash research/scripts/watch.sh <这次派出的 agent id，逗号分隔>`（只填 agent 号：会话目录它自己找，阈值在 `research/scripts/watch.conf` 里配，不在命令行上手敲；调用里不再加 `&`、`nohup`、`disown`）：它每 4 分钟复检一次子 agent 的会话记录与本实例底下的长进程、每 15 秒读一次 hook 的检出记录，发现没超时的等待循环、一次工具调用超过 8 分钟、同一命令反复且输出不变、按模式找进程、10 分钟无动静、结束本轮而手里没有在跑的后台任务、进程写的文件 20 分钟不涨、本实例底下有没带 `SINGLEFS_HEAVY_TESTS` 前缀的重型测试进程（确认接着盯写 `--ack 进程:<pid>`）、子 agent 从派发起的运行时间跨过一个整小时（每个整点报一次，那个整点之后主 agent 给它发过消息就不报；一格多长在 `watch.conf` 的 `ask-every-minutes`），或 hook 检出本会话的问题命令，就退出并叫醒主 agent；子 agent 全部交回或被停也退出，没有告警时每 60 分钟定时回报一次。
+
+改了定义、共用约束或规则之后，当场给每个在跑的子 agent 发消息：写明改了哪一条、它手上哪一步要停掉或改做。
 
-子 agent 等长活时不续提示缓存，主 agent 也不定时 ping 它。临时派不读共用约束的 agent（general-purpose 这类）干带长等待的活时，派发提示里写上共用约束「长活可以等」那一条里后台命令怎么写。
+子 agent 等长活时不续提示缓存；主 agent 除了看门狗报「跑满 N 小时」时的那一条例行询问，不另外定时 ping 它。临时派不读共用约束的 agent（general-purpose 这类）干带长等待的活时，派发提示里写上共用约束「长活可以等」那一条里后台命令怎么写。
 
-叫醒之后主 agent 判断：只是慢就再起一个看门狗接着盯；是问题就决定发消息让它改、停掉重派、还是报给用户。主 agent 自己起的长命令同样放后台，命令照共用约束「长活可以等」那一条写（不在里面再把活放到后台），由看门狗盯着进程：没有子 agent 在跑时用 `bash research/scripts/watch.sh --processes`。
+叫醒之后主 agent 判断：只是慢就再起一个看门狗接着盯；是问题就决定发消息让它改、停掉重派、还是报给用户。收到「跑满 N 小时要主 agent 问一次」就给它发一条例行询问：在做什么、还差几步、在等哪个进程、预计多久，等的已经结束就接着做或交回，并写明把回答写进它草稿目录的 `progress.md`（子 agent 多半没有 SendMessage）；它结束这一轮之后读那份文件，没写就读它会话记录里最后一段正文；发完再起看门狗。主 agent 自己起的长命令同样放后台，命令照共用约束「长活可以等」那一条写（不在里面再把活放到后台），由看门狗盯着进程：没有子 agent 在跑时用 `bash research/scripts/watch.sh --processes`。
 
 ## 交回怎么读
 
 判决与写回只引产物与代码，agent 的结论句一律当线索：以仓里的产物为准，现查过再写进 kb。
 
-交回按子 agent 的 SubagentHandback 消息判。后台派的子 agent 每结束一轮，harness 发一条 status 为 completed 的通知；结果写「This agent has not reported yet: it is waiting on its own background work」、或 note 写「the result below may be interim」的，是它结束本轮去等自己起的后台任务：不当交回、不当出错，照旧等它的交回消息与看门狗。status 为 completed、没有交回消息、note 与 result 里也没有这两句的，是它结束本轮时手里没有还在跑的后台任务，不会自己醒（看门狗报「结束本轮却不会醒」）：看它最后在等什么，发消息让它接着做或交回，或者停掉。主 agent 用 TaskStop 停掉的子 agent 不再交回，它的看门狗按被停退出。
+交回按子 agent 的 SubagentHandback 消息判。**交回之后不再给它发消息**（续做闸 `.claude/hooks/continuation-guard.sh` 拒绝）：要补的活新派一个同类 agent，派发提示指到它的报告与产物；要它会话里的进度，用 `research/scripts/agent-handover.py` 抽交接摘要给新 agent 读。还没交回、在干活或在等自己后台任务的，照旧能收整点询问与纠正。后台派的子 agent 每结束一轮，harness 发一条 status 为 completed 的通知；结果写「This agent has not reported yet: it is waiting on its own background work」、或 note 写「the result below may be interim」的，是它结束本轮去等自己起的后台任务：不当交回、不当出错，照旧等它的交回消息与看门狗。status 为 completed、没有交回消息、note 与 result 里也没有这两句的，是它结束本轮时手里没有还在跑的后台任务，不会自己醒（看门狗报「结束本轮却不会醒」）：看它最后在等什么，发消息让它接着做或交回，或者停掉。主 agent 用 TaskStop 停掉的子 agent 不再交回，它的看门狗按被停退出。
 
 ## 派发提示怎么写
 
+同时在跑的重活（编译、测试、变异、产物、原型扫描）不止一个时，每份派发提示写一行「线程上限：N」，N = 整机核数 ÷ 同时在跑的重活数（向下取整，至少 1）；重活数变了，给在跑的 agent 发消息改 N。
+
 在不影响正确性的前提下写简练：只写对方干这件活要的东西——定义「输入」一节要的项、为哪几行岔路或哪个问题、定义与共用约束里没有的约束、交付什么交到哪；不复述定义与规则里已经写着的做法，不讲来龙去脉。事实、路径、行号、数与判据一个不少，嫌长就指到文件，不删内容。
 
 ## 什么时候派哪个 agent
@@ -43,11 +52,11 @@
 | 什么时候 | 派谁、按什么次序 |
 |---|---|
 | 要推论：设计判断、取舍，拿依据（包括实验结论）去推翻或确立决策，实现改动的对抗 | 主 agent 写 `research/prompts/_<轮>-body.md` → `three-way-materials` → 同一条消息并行派 `three-way-forward` 或 `three-way-defense`（一轮一条）、`three-way-attack`、`three-way-local-attack` 或 `three-way-local-defense`（一轮一条）→ 全部交齐后，有腿交了模型或产物就派 `three-way-verifier` → 主 agent 写判决；三轮之后停 |
-| 改 `crates/`：条款已定、验收标准写得清、改动有界（照已定分项实现、补测试与变异、修原因已知的红） | `implementation-writer`（后台派，主 agent 审 diff）→ 上一行的三方（代码轮）→ `crash-verifier`（层 0、QEMU、herd7、crates 变异表） |
-| 改 `crates/`：探索性的、条款没写全、要用户边看边拍板 | 主 agent 自己写、直接和用户对话 → 上一行的三方（代码轮）→ `crash-verifier` |
+| 改 `crates/`：条款已定、验收标准写得清、改动有界（照已定分项实现、补测试与变异、修原因已知的红） | `implementation-writer`（后台派，主 agent 审 diff）→ 上一行的三方（代码轮）→ 提交时派 `crash-verifier` 跑层 0、QEMU、herd7、crates 变异表（命令带 `SINGLEFS_HEAVY_TESTS=commit`，见「暂存之后、提交之前跑门禁」那一行；平时不跑） |
+| 改 `crates/`：探索性的、条款没写全、要用户边看边拍板 | 主 agent 自己写、直接和用户对话 → 上一行的三方（代码轮）→ 提交时 `crash-verifier` |
 | 跑变异表、看抓到 / 无效 / 没红 | `mutation-triage` → 要补取样点、补断言的：`crates/` 那侧 `implementation-writer`（输入给分诊报告），`research/` 那侧 `experiment-designer` 写重跑登记 → `experiment-runner` |
 | 一个阶段任务结束：里程碑一步、一轮判决、一段实验、一批定义或脚本改完，这一批交回到齐、暂存之前 | `sweep` 写事实表（阶段同步第一段；输入给改动范围与做成的事，只用过、没留改动的也写；交回的事实表过了 `research/scripts/stale-candidates.py --check-facts`）→ 候选表按组切段，每段不超过约 200 行，一段派一个 `sweep` 逐行判（第二段；输入同样给做成的事）→ 交回齐了主 agent 跑 `stale-candidates.py --check-report 候选表 各份报告`，退出码 0 才往下，再**逐行全看**：每一条判定都对着载体今天的原文核一遍，不抽样、不按判定种类挑。判错的那一段重派，重派交回照样全看 → 主 agent 逐处判，自己改或交 `kb-scribe` → 主 agent 写 `research/prompts/<阶段>-sync.md`（格式见门禁 68 号文件头，68 号判形式）→ 下一行 |
-| 暂存之后、提交之前跑门禁 | 主 agent 用 `research/scripts/stage-mine.py` 暂存 → 这一批改了门禁 54 号在 `.claude/gate.d/stage-inputs.tsv` 登记的输入，就在 HEAD + 暂存区的 worktree 里后台跑那棵树里的 `bash .claude/gate.d/54-layer0-replay.sh --full <它的根>`（层 0 全量只在这一步跑，判绿按输入哈希写全绿标记，整轮门禁的 54 号只核标记）→ `gate-triage` |
+| 暂存之后、提交之前跑门禁 | 主 agent 用 `research/scripts/stage-mine.py` 暂存 → 派 `crash-verifier` 跑它那几道，命令带 `SINGLEFS_HEAVY_TESTS=commit`：这一批改了门禁 54 号在 `.claude/gate.d/stage-inputs.tsv` 登记的输入，就在 HEAD + 暂存区的 worktree 里跑那棵树里的 `SINGLEFS_HEAVY_TESTS=commit bash .claude/gate.d/54-layer0-replay.sh --full <它的根>`（层 0 全量只在这一步跑，判绿按输入哈希写全绿标记，整轮门禁的 54 号只核标记），55、57、59 同样带前缀 → `gate-triage` 带 `SINGLEFS_HEAVY_TESTS=commit` 跑 `gate.sh --staged` 并分诊（54、55、57、59 在 `gate.sh` 里照各自的复用判定走，它不直接调）；用户要求时两处都换成 `=user-request` |
 | 撤回一个数、改格式常量、新立一条判据 | `sweep` |
 | 定案之后写回 kb | `kb-scribe`（主 agent 给逐条规格）；规格动了某条分项的「**依据**」段时，同一份规格要带上那个实验页 `### 影响的决策` 表里对应那几行（门禁 75 号那条双向检查两侧要同时到位，而判一个实验撑不撑一条分项是主 agent 的活）；它翻了分项状态、33 号红（变异表锚点腐化）→ `experiment-runner` 只修锚点 |
 | 要建计数实验，或重跑已有实验 | 主 agent 写岔路单（`.claude/rules/three-way-inference.md`「交岔路时写岔路单」；不是为岔路建的实验写同格式的问题单）→ `experiment-designer` 写跑前登记或重跑登记 → 主 agent 删掉登记里待删的问法（有的话）→ `experiment-runner` → 每段交回主 agent 对着岔路单判够不够：一行开着的都不剩就停、交岔路表；续派时派发提示点名还差的那几行（续派闸 `.claude/hooks/runner-dispatch-guard.sh` 查这一句） |
```

（以上 `git show` 共 236 行，含 commit message；7 个文件都是修改，没有新建文件。）

## 三、两份起草报告的相关小节（原样抄，标明来源）

### 出处：`research/prompts/defs-closeout-draft-tmp-evidence/report.md` 第五节「97f5904 七份定义逐块覆盖表」（第 134-188 行）

```markdown
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

```

### 出处：`research/prompts/defs-closeout-draft-tmp-evidence/report.md` 第六节「没被任何判决判到内容的块，交主 agent 并进这一轮」（第 189-218 行）

```markdown
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

```

### 出处：`research/prompts/defs-closeout-draft-tmp-evidence/report.md` 第七节「第 40 行：重型闸的「静态分支」怎么判，两种做法与代价（这一次不改钩子）」（第 219-237 行）

```markdown
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
```

### 出处：`research/prompts/defs-closeout-draft-2-tmp-evidence/report.md`「二、两份 hook 今天实际拒哪些（现查）」（第 18-112 行）

````markdown
## 二、两份 hook 今天实际拒哪些（现查）

做法：往 hook 的标准输入喂 hook JSON，只看退出码（2 = 拒），被判的命令一条都没执行。脚本 `/tmp/claude-1000/defs-closeout-draft-2/probe_hooks.py`、`probe_heavy.py`、`probe_heredoc.py`，原样输出 `/tmp/claude-1000/defs-closeout-draft-2/probe-hooks.log`。另读了两份 hook 的入口：`bash-command-detector.sh` 的 `main()`（第 2339 行起，`return 2` 在第 2386–2453 行的七处）依次调 `watchdog_rejection`、`process_signal_refusal`、`detaching_refusal`、`unwaited_ampersand_refusal`、`wait_loop_refusal`、`results_overwrite_refusal`、`script_in_place_refusal`，命中就 `return 2`，一共七种；`write-guard.sh` 的 `decide()`（第 159–170 行）依次三道。`.claude/settings.json` 里 `write-guard.sh` 挂在 `Write|Edit`（没挂 MultiEdit），`bash-command-detector.sh`、`heavy-test-guard.sh`、`pattern-process-guard.sh` 挂在 `Bash`。

`bash-command-detector.sh`（原样输出，每行末尾是 stderr 第一行的前 120 字）：

```
bash-command-detector	① 看门狗前台起	exit=2	✗ 看门狗起法不对（run_in_background 不是 true；认出的调用：watch.sh abc123）：这样起的看门狗叫不醒主 agent，等于没盯
bash-command-detector	① 看门狗后台加 &	exit=2	✗ 看门狗起法不对（命令里有单独的 &；认出的调用：watch.sh abc123）：这样起的看门狗叫不醒主 agent，等于没盯
bash-command-detector	① 看门狗后台正确起（对照放行）	exit=0	
bash-command-detector	② 前台无超时等待循环	exit=2	✗ 前台的等待循环没有超时（认出的循环：until grep -q x log）：等的条件不成立就一直不返回，这一次调用卡在这里期间，发给你的消息也送不到
bash-command-detector	② 外套 timeout（对照放行）	exit=0	
bash-command-detector	③ disown	exit=2	✗ 把活放出了追踪（认出的写法：disown）：…
bash-command-detector	③ nohup … &	exit=2	✗ 把活放出了追踪（认出的写法：nohup … &）：…
bash-command-detector	③ setsid -f	exit=2	✗ 把活放出了追踪（认出的写法：setsid -f）：…
bash-command-detector	④ 后台里单独 & 无 wait	exit=2	✗ run_in_background 里又把活放到了后台（以单独的 & 收尾、之后同一条命令里没有 wait 的作业：sleep 100）：…
bash-command-detector	④ a & b & wait（对照放行）	exit=0	
bash-command-detector	⑤ > 覆盖未跟踪产物	exit=2	✗ 要整份覆盖 research/results/ 下已存在、又没进 git 的产物（认出的写法与目标：> research/results/e142-first-txn-dry-run-2026-09-25-r17-main.out）：旧
bash-command-detector	⑤ >| 覆盖未跟踪产物	exit=2	（同上形，认出 >|）
bash-command-detector	⑤ &> 覆盖未跟踪产物	exit=2	（认出 &>）
bash-command-detector	⑤ tee 不带 -a	exit=2	（认出 tee）
bash-command-detector	⑤ cp 覆盖	exit=2	（认出 cp）
bash-command-detector	⑤ mv 覆盖	exit=2	（认出 mv）
bash-command-detector	⑤ install 覆盖	exit=2	（认出 install）
bash-command-detector	⑤ dd of=	exit=2	（认出 dd）
bash-command-detector	⑤ truncate	exit=2	（认出 truncate）
bash-command-detector	⑤ >> 追加（对照放行）	exit=0	
bash-command-detector	⑥ kill 负号	exit=2	✗ 终止进程的写法打得到不是自己点名的那一个进程（kill：目标 -1 带负号，是整个进程组（-1 是能发到的全部进程）（kill -9 -1））：…
bash-command-detector	⑥ kill 0	exit=2	✗ …（kill：目标 0 是发这条命令的整个进程组（kill 0））…
bash-command-detector	⑥ kill 两个目标	exit=2	✗ …（kill 一次给了 2 个目标（999991 999992）：一次只停点名的一个…）
bash-command-detector	⑥ kill 命令替换	exit=2	✗ …（kill：目标 $(pgrep sleep) 是命令替换…）
bash-command-detector	⑥ 循环里 kill	exit=2	✗ …（kill 在 for / while / until 循环里逐个发：这是批量停，不是点名停一个…）
bash-command-detector	⑥ proc.py stop 循环	exit=2	✗ …（proc.py stop 在 for / while / until 循环里逐个发…）
bash-command-detector	⑥ systemctl stop ssh	exit=2	✗ …（systemctl stop ssh：这是 SSH、登录会话、用户级 systemd、本地模型服务或它们依赖的系统服务…）
bash-command-detector	⑥ kill "$!"（对照放行）	exit=0	
bash-command-detector	⑥ proc.py stop 单个（对照放行）	exit=0	
bash-command-detector	⑦ > 覆盖已有脚本	exit=2	✗ 要在同一个 inode 上改一个已经存在的脚本（认出的写法与目标：> research/scripts/replay.sh）：…
bash-command-detector	⑦ cp 覆盖已有脚本	exit=2	（认出 cp）
bash-command-detector	⑦ tee 覆盖已有脚本	exit=2	（认出 tee）
bash-command-detector	⑦ python open w	exit=2	（认出 python open(…, 'w')）
bash-command-detector	⑦ mv 换上（对照放行）	exit=0	
bash-command-detector	⑦ >> 追加（对照放行）	exit=0	
bash-command-detector	旁：sed -i 改脚本	exit=0	
```

（这一段为省篇幅把重复的 stderr 尾巴写成了「…」「（认出 X）」，完整原样在 `probe-hooks.log`。）

`write-guard.sh`（原样）：

```
write-guard	一 Write 覆盖未跟踪已有文件	exit=2	✗ 拒绝用 Write 整份覆盖 research/results/e142-first-txn-dry-run-2026-09-25-r17-main.out：它已存在而且没进 git
write-guard	一 Write 新文件（对照放行）	exit=0	
write-guard	二 子 agent 越出写范围	exit=2	✗ experiment-runner 的写范围不含 .claude/main-agent.md
write-guard	二 未登记的项目 agent	exit=2	✗ gate-triage 在写范围表里没有登记，按拒绝处理：README.md
write-guard	三 Edit 写进撇号角标	exit=2	✗ 写进去的内容里有撇号类角标 （U+2032）在「…A…」（/tmp/claude-1000/x.md，这次写的内容里共 1 处）
write-guard	三 Write 写进撇号角标	exit=2	✗ 写进去的内容里有撇号类角标 （U+02B9）在「…B…」（/tmp/claude-1000/probe-new-xyz.md，这次写的内容里共 1 处）
```

（最后两行里的角标字符本身在这份报告里删掉了，原样在 `probe-hooks.log`。）

清单里另收了两道同挂 Bash、同样在执行前拒绝的 hook，也现查了：

```
pattern-process-guard	pgrep -f foo	exit=2
pattern-process-guard	pkill -f foo	exit=2
pattern-process-guard	killall foo	exit=2
pattern-process-guard	grep -n pgrep x.sh	exit=0
heavy-test-guard	crash-verifier	nice -n 19 cargo test -p singlefs-core --lib	exit=2	✗ 不经内存包装跑编译出来的代码被拒：cargo test（crash-verifier 不经 run-with-memory-cap.sh）
heavy-test-guard	gate-triage	nice -n 19 cargo test -p singlefs-core --lib	exit=2	✗ 不经内存包装跑编译出来的代码被拒：cargo test（gate-triage 不经 run-with-memory-cap.sh）
heavy-test-guard	gate-triage	nice -n 19 bash research/scripts/run-with-memory-cap.sh 8G cargo test -p singlefs-core --lib	exit=0	
heavy-test-guard	implementation-writer	nice -n 19 cargo test -p singlefs-core --lib	exit=2	✗ 不经内存包装跑编译出来的代码被拒：cargo test（implementation-writer 不经 run-with-memory-cap.sh）
heavy-test-guard	implementation-writer	nice -n 19 bash research/scripts/run-with-memory-cap.sh 8G cargo test -p singlefs-core --lib	exit=0	
heavy-test-guard	implementation-writer	nice -n 19 cargo build --offline --all-targets	exit=0	
heavy-test-guard	implementation-writer	nice -n 19 cargo clippy --all-targets	exit=0	
heavy-test-guard	experiment-runner	nice -n 19 cargo run --release --bin e142-first-txn-dry-run	exit=2	✗ 不经内存包装跑编译出来的代码被拒：cargo run（experiment-runner 不经 run-with-memory-cap.sh）
heavy-test-guard	experiment-runner	nice -n 19 bash research/scripts/run-with-memory-cap.sh 8G cargo run --release --bin e142-first-txn-dry-run	exit=0	
heavy-test-guard	crash-verifier	SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/54-layer0-replay.sh	exit=0	
heavy-test-guard	crash-verifier	SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash research/scripts/run-with-memory-cap.sh 8G bash .claude/gate.d/54-layer0-replay.sh	exit=0	
heavy-test-guard	crash-verifier	SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash research/scripts/run-with-memory-cap.sh 8G bash .claude/gate.d/55-qemu-first-transaction.sh	exit=0	
heavy-test-guard	crash-verifier	SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash research/scripts/run-with-memory-cap.sh 8G bash .claude/gate.d/57-lkmm.sh	exit=0	
heavy-test-guard	crash-verifier	SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/59-crates-mutation-replay.sh	exit=0	
heavy-test-guard	gate-triage	SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/scripts/gate.sh --staged	exit=0	
heavy-test-guard	gate-triage	SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/87-replay.sh	exit=0	
heavy-test-guard	implementation-writer	'bash research/scripts/run-with-memory-cap.sh 8G bash <<\'X\'\ncargo test -p singlefs-core --lib\nX'	exit=0	
heavy-test-guard	implementation-writer	"bash research/scripts/run-with-memory-cap.sh 8G bash -c 'cargo test -p singlefs-core --lib'"	exit=0	
```

据此写进清单的与没写进的：

- 写进：`bash-command-detector.sh` ①–⑦（每种至少一例拒、一例对照放行现查过；⑤ 的 `>&` 文件、带 fd 号的 `2>`，⑥ 的 pkill / fuser -k / cgroup.kill / loginctl / 关机，⑦ 的 `>|`、`&>`、`dd`、`truncate`、`Path.write_text` 这些没逐个喂，照 hook 文件头第 71–132 行写的归类，没写进逐字清单的用「这一类」收）；`write-guard.sh` 三道（都现查过）；`pattern-process-guard.sh`（现查过）与 `heavy-test-guard.sh` 的两类拒绝（重型测试、子 agent 不经包装跑编译出来的代码，后一类现查过）。
- 收这两道的理由：`agent-common.md` 改前那句写「run_in_background 里的等待循环与按模式找进程这类命令只记检出」，按模式找进程今天由 `pattern-process-guard.sh` 在执行前拒（上面第 1–3 行），那半句是错的；`main-agent.md` 改前的清单里本来就有「子 agent 跑重型测试」一项（`heavy-test-guard.sh`）。
- 没写进：只记检出、不拒的（run_in_background 里的等待循环、起了几个 `&` 而 `wait "$pid"` 只等了其中一个、`start-stop-daemon -b`、⑤ 目标这一刻算不出的）；拒的不是 Bash 命令写法的几道（`continuation-guard.sh`、`runner-dispatch-guard.sh`、`ask-user-claim-guard.sh`、`handback-scratch-check.sh`，各自在 `main-agent.md` 与共用约束别处已经点名）。
- `sed -i` 改已有脚本 ⑦ 不拒（exit=0），清单里没列它。
- `heavy-test-guard.sh` 文件头第 81 行说 `run-with-memory-cap.sh 4G bash <<EOF` 会被误拒，现查退 0（上面倒数第 2 行），所以没把「不写成 heredoc」写进共用约束。hook 文件头与实际对不上这一处，没改 hook，交主 agent。

````

### 出处：`research/prompts/defs-closeout-draft-2-tmp-evidence/report.md`「五、要三方判的点（我自己定的、推的）」（第 238-246 行）

```markdown
## 五、要三方判的点（我自己定的、推的）

1. **崩溃验证员 54、55、57 号的上限**：第 1b 步照记录第 30 行 ④「54、55、57 号不改，整条经包装跑」写；上限照共用约束取派发提示给的、没给取 `replay.sh` 的默认值 `8G`（`research/scripts/replay.sh` 第 24 行 `REPLAY_MEMORY_CAP="${REPLAY_MEMORY_CAP:-8G}"`）。记录同一格写着「三道要多少内存都没量过」，例子用的是 16G：8G 够不够层 0 全量、够不够 55 号的虚机，推的，没量过；不够时撞 250 / 254，照共用约束交回、不绕开重跑，代价是那一趟白跑。要不要在崩溃验证员「输入」里把上限列成必给项，三方定。
2. **门禁分诊员「`gate.sh` 与单跑的门禁阶段直接跑，外面不包一层」**：hook 不要求（第二节 `gate.sh --staged`、87 号退 0）。不包的推断是：`gate.sh` 里 59 号、87 号（经 `replay.sh`）在里面逐条套了，外面再包就是嵌套，按包装文件头「已占」的算法可能重复记账、排到 252（第一批报告第二节第 5 条同一个推断，没量过）。代价是 `gate.sh` 里 `check.sh`、15 号、74 号起的 cargo 不在任何包装里，这正是记录第 30 行现状格里留着的欠账「按名字判的门禁阶段（15、74 号这些）里起的 cargo 不在闸的射程里」，定义这一侧没补它。
3. **攻方第 3c 步「`cargo build` 也经它」**：照 `_m2-rollback-forward-r3-body.md` 第 59 行「编译与跑一律经」，比共用约束严（共用约束写 `cargo build` 不要求）；两处不矛盾，但同一件事两种要求，三方看要不要统一。
4. **「执行前拒绝的写法」一条的射程**：派发只点了两份 hook，我另收了 `pattern-process-guard.sh` 与 `heavy-test-guard.sh`（第二节写了理由与现查）。⑤⑥⑦ 里没逐个喂的形态是照 hook 文件头归类的，写成「这一类」，没写成穷举。
5. **main-agent 第 3 条只指不列**：87 号与 E152 装置靠指过去的那张清单罩着，字面不再出现在第 3 条里；主 agent 的上下文里有 `implementation-workflow.md`（`CLAUDE.md` `@` 了它）。
6. **实现员第 1b 步「副本里跑的也算」**：包装按文件名认（`.claude/hooks/lib_shell_words.py:51` `WRAPPERS_TAKING_ONE_ARGUMENT = {"capped.sh", "run-with-memory-cap.sh"}`），副本里的那一份也认；账与锁在 `${XDG_RUNTIME_DIR}/singlefs-heavy-admission`（`run-with-memory-cap.sh:107`），各副本共用一份。峰值表从不带 `.git` 的副本里跑时落在副本自己的 `research/scripts/memory-peaks.tsv`（`run-with-memory-cap.sh:227–231`），与主仓那份不通；没写进定义，推的影响是峰值表记不到主仓。

```

## 四、探针原样输出 `/tmp/claude-1000/defs-closeout-draft-2/probe-hooks.log`（整份，64 行）

```
bash-command-detector	① 看门狗前台起	exit=2	✗ 看门狗起法不对（run_in_background 不是 true；认出的调用：watch.sh abc123）：这样起的看门狗叫不醒主 agent，等于没盯
bash-command-detector	① 看门狗后台加 &	exit=2	✗ 看门狗起法不对（命令里有单独的 &；认出的调用：watch.sh abc123）：这样起的看门狗叫不醒主 agent，等于没盯
bash-command-detector	① 看门狗后台正确起（对照放行）	exit=0	
bash-command-detector	② 前台无超时等待循环	exit=2	✗ 前台的等待循环没有超时（认出的循环：until grep -q x log）：等的条件不成立就一直不返回，这一次调用卡在这里期间，发给你的消息也送不到
bash-command-detector	② 外套 timeout（对照放行）	exit=0	
bash-command-detector	③ disown	exit=2	✗ 把活放出了追踪（认出的写法：disown）：进程移出 shell 的作业表或另起会话，随后的 wait 立刻返回、完成通知当场发出，它跑完不会叫醒任何人，成了没人追踪的孤儿
bash-command-detector	③ nohup … &	exit=2	✗ 把活放出了追踪（认出的写法：nohup … &）：进程移出 shell 的作业表或另起会话，随后的 wait 立刻返回、完成通知当场发出，它跑完不会叫醒任何人，成了没人追踪的孤儿
bash-command-detector	③ setsid -f	exit=2	✗ 把活放出了追踪（认出的写法：setsid -f）：进程移出 shell 的作业表或另起会话，随后的 wait 立刻返回、完成通知当场发出，它跑完不会叫醒任何人，成了没人追踪的孤儿
bash-command-detector	④ 后台里单独 & 无 wait	exit=2	✗ run_in_background 里又把活放到了后台（以单独的 & 收尾、之后同一条命令里没有 wait 的作业：sleep 100）：外层 shell 起完它就退出，完成通知当场发出，真跑完的那个进程不会叫醒任何人
bash-command-detector	④ a & b & wait（对照放行）	exit=0	
bash-command-detector	⑤ > 覆盖未跟踪产物	exit=2	✗ 要整份覆盖 research/results/ 下已存在、又没进 git 的产物（认出的写法与目标：> research/results/e142-first-txn-dry-run-2026-09-25-r17-main.out）：旧
bash-command-detector	⑤ >| 覆盖未跟踪产物	exit=2	✗ 要整份覆盖 research/results/ 下已存在、又没进 git 的产物（认出的写法与目标：>| research/results/e142-first-txn-dry-run-2026-09-25-r17-main.out）：
bash-command-detector	⑤ &> 覆盖未跟踪产物	exit=2	✗ 要整份覆盖 research/results/ 下已存在、又没进 git 的产物（认出的写法与目标：&> research/results/e142-first-txn-dry-run-2026-09-25-r17-main.out）：
bash-command-detector	⑤ tee 不带 -a	exit=2	✗ 要整份覆盖 research/results/ 下已存在、又没进 git 的产物（认出的写法与目标：tee research/results/e142-first-txn-dry-run-2026-09-25-r17-main.out）
bash-command-detector	⑤ cp 覆盖	exit=2	✗ 要整份覆盖 research/results/ 下已存在、又没进 git 的产物（认出的写法与目标：cp research/results/e142-first-txn-dry-run-2026-09-25-r17-main.out）：
bash-command-detector	⑤ mv 覆盖	exit=2	✗ 要整份覆盖 research/results/ 下已存在、又没进 git 的产物（认出的写法与目标：mv research/results/e142-first-txn-dry-run-2026-09-25-r17-main.out）：
bash-command-detector	⑤ install 覆盖	exit=2	✗ 要整份覆盖 research/results/ 下已存在、又没进 git 的产物（认出的写法与目标：install research/results/e142-first-txn-dry-run-2026-09-25-r17-main.
bash-command-detector	⑤ dd of=	exit=2	✗ 要整份覆盖 research/results/ 下已存在、又没进 git 的产物（认出的写法与目标：dd research/results/e142-first-txn-dry-run-2026-09-25-r17-main.out）：
bash-command-detector	⑤ truncate	exit=2	✗ 要整份覆盖 research/results/ 下已存在、又没进 git 的产物（认出的写法与目标：truncate research/results/e142-first-txn-dry-run-2026-09-25-r17-main
bash-command-detector	⑤ >> 追加（对照放行）	exit=0	
bash-command-detector	⑥ kill 负号	exit=2	✗ 终止进程的写法打得到不是自己点名的那一个进程（kill：目标 -1 带负号，是整个进程组（-1 是能发到的全部进程）（kill -9 -1））：后面的命令不许停掉前面的任务，不许动 SSH、VSCode 与 Claude 会话，一次只停
bash-command-detector	⑥ kill 0	exit=2	✗ 终止进程的写法打得到不是自己点名的那一个进程（kill：目标 0 是发这条命令的整个进程组（kill 0））：后面的命令不许停掉前面的任务，不许动 SSH、VSCode 与 Claude 会话，一次只停点名的一个
bash-command-detector	⑥ kill 两个目标	exit=2	✗ 终止进程的写法打得到不是自己点名的那一个进程（kill 一次给了 2 个目标（999991 999992）：一次只停点名的一个（kill 999991 999992））：后面的命令不许停掉前面的任务，不许动 SSH、VSCode 与 C
bash-command-detector	⑥ kill 命令替换	exit=2	✗ 终止进程的写法打得到不是自己点名的那一个进程（kill：目标 $(pgrep sleep) 是命令替换：展开出几个进程、是谁起的，这一刻都不知道（kill $(pgrep sleep)）；同一条命令里先从 cgroup.procs、/p
bash-command-detector	⑥ 循环里 kill	exit=2	✗ 终止进程的写法打得到不是自己点名的那一个进程（kill 在 for / while / until 循环里逐个发：这是批量停，不是点名停一个（do kill $p））：后面的命令不许停掉前面的任务，不许动 SSH、VSCode 与 Cl
bash-command-detector	⑥ proc.py stop 循环	exit=2	✗ 终止进程的写法打得到不是自己点名的那一个进程（proc.py stop 在 for / while / until 循环里逐个发：这是批量停，不是点名停一个（do python3 .claude/singlefs-ai-sop/scri
bash-command-detector	⑥ systemctl stop ssh	exit=2	✗ 终止进程的写法打得到不是自己点名的那一个进程（systemctl stop ssh：这是 SSH、登录会话、用户级 systemd、本地模型服务或它们依赖的系统服务（systemctl stop ssh））：后面的命令不许停掉前面的任务
bash-command-detector	⑥ kill "$!"（对照放行）	exit=0	
bash-command-detector	⑥ proc.py stop 单个（对照放行）	exit=0	
bash-command-detector	⑦ > 覆盖已有脚本	exit=2	✗ 要在同一个 inode 上改一个已经存在的脚本（认出的写法与目标：> research/scripts/replay.sh）：正在跑它的 bash 按文件偏移往下读，改完会从新内容的同一偏移接着读，读到的是别的东西
bash-command-detector	⑦ cp 覆盖已有脚本	exit=2	✗ 要在同一个 inode 上改一个已经存在的脚本（认出的写法与目标：cp research/scripts/replay.sh）：正在跑它的 bash 按文件偏移往下读，改完会从新内容的同一偏移接着读，读到的是别的东西
bash-command-detector	⑦ tee 覆盖已有脚本	exit=2	✗ 要在同一个 inode 上改一个已经存在的脚本（认出的写法与目标：tee research/scripts/replay.sh）：正在跑它的 bash 按文件偏移往下读，改完会从新内容的同一偏移接着读，读到的是别的东西
bash-command-detector	⑦ python open w	exit=2	✗ 要在同一个 inode 上改一个已经存在的脚本（认出的写法与目标：python open(…, 'w') research/scripts/replay.sh）：正在跑它的 bash 按文件偏移往下读，改完会从新内容的同一偏移接着读，读
bash-command-detector	⑦ mv 换上（对照放行）	exit=0	
bash-command-detector	⑦ >> 追加（对照放行）	exit=0	
bash-command-detector	旁：sed -i 改脚本	exit=0	
write-guard	一 Write 覆盖未跟踪已有文件	exit=2	✗ 拒绝用 Write 整份覆盖 research/results/e142-first-txn-dry-run-2026-09-25-r17-main.out：它已存在而且没进 git
write-guard	一 Write 新文件（对照放行）	exit=0	
write-guard	二 子 agent 越出写范围	exit=2	✗ experiment-runner 的写范围不含 .claude/main-agent.md
write-guard	二 未登记的项目 agent	exit=2	✗ gate-triage 在写范围表里没有登记，按拒绝处理：README.md
write-guard	三 Edit 写进撇号角标	exit=2	✗ 写进去的内容里有撇号类角标 <U+2032>（U+2032）在「…A<U+2032>…」（/tmp/claude-1000/x.md，这次写的内容里共 1 处）
write-guard	三 Write 写进撇号角标	exit=2	✗ 写进去的内容里有撇号类角标 <U+02B9>（U+02B9）在「…B<U+02B9>…」（/tmp/claude-1000/probe-new-xyz.md，这次写的内容里共 1 处）
pattern-process-guard	pgrep -f foo	exit=2
pattern-process-guard	pkill -f foo	exit=2
pattern-process-guard	killall foo	exit=2
pattern-process-guard	grep -n pgrep x.sh	exit=0
heavy-test-guard	crash-verifier	SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/54-layer0-replay.sh	exit=0	
heavy-test-guard	crash-verifier	SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash research/scripts/run-with-memory-cap.sh 8G bash .claude/gate.d/54-layer0-replay.sh	exit=0	
heavy-test-guard	crash-verifier	SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash research/scripts/run-with-memory-cap.sh 8G bash .claude/gate.d/55-qemu-first-transaction.sh	exit=0	
heavy-test-guard	crash-verifier	SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash research/scripts/run-with-memory-cap.sh 8G bash .claude/gate.d/57-lkmm.sh	exit=0	
heavy-test-guard	crash-verifier	SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/59-crates-mutation-replay.sh	exit=0	
heavy-test-guard	crash-verifier	nice -n 19 cargo test -p singlefs-core --lib	exit=2	✗ 不经内存包装跑编译出来的代码被拒：cargo test（crash-verifier 不经 run-with-memory-cap.sh）
heavy-test-guard	gate-triage	SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/scripts/gate.sh --staged	exit=0	
heavy-test-guard	gate-triage	SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/87-replay.sh	exit=0	
heavy-test-guard	gate-triage	nice -n 19 cargo test -p singlefs-core --lib	exit=2	✗ 不经内存包装跑编译出来的代码被拒：cargo test（gate-triage 不经 run-with-memory-cap.sh）
heavy-test-guard	gate-triage	nice -n 19 bash research/scripts/run-with-memory-cap.sh 8G cargo test -p singlefs-core --lib	exit=0	
heavy-test-guard	implementation-writer	nice -n 19 cargo test -p singlefs-core --lib	exit=2	✗ 不经内存包装跑编译出来的代码被拒：cargo test（implementation-writer 不经 run-with-memory-cap.sh）
heavy-test-guard	implementation-writer	nice -n 19 bash research/scripts/run-with-memory-cap.sh 8G cargo test -p singlefs-core --lib	exit=0	
heavy-test-guard	implementation-writer	nice -n 19 cargo build --offline --all-targets	exit=0	
heavy-test-guard	implementation-writer	nice -n 19 cargo clippy --all-targets	exit=0	
heavy-test-guard	experiment-runner	nice -n 19 cargo run --release --bin e142-first-txn-dry-run	exit=2	✗ 不经内存包装跑编译出来的代码被拒：cargo run（experiment-runner 不经 run-with-memory-cap.sh）
heavy-test-guard	experiment-runner	nice -n 19 bash research/scripts/run-with-memory-cap.sh 8G cargo run --release --bin e142-first-txn-dry-run	exit=0	
heavy-test-guard	implementation-writer	"bash research/scripts/run-with-memory-cap.sh 8G bash <<'X'\ncargo test -p singlefs-core --lib\nX"	exit=0	
heavy-test-guard	implementation-writer	"bash research/scripts/run-with-memory-cap.sh 8G bash -c 'cargo test -p singlefs-core --lib'"	exit=0	
```

## 五、两份 hook 的判定函数位置（材料员现查，2026-09-26，主工作区）

`.claude/hooks/bash-command-detector.sh` 的 `main()` 第 2339 行起。七种拒绝对应的 `return 2` 行号（`awk 'NR>=2339 && NR<=2460 && /return 2$/{print NR}' .claude/hooks/bash-command-detector.sh` 现查）：

- 第 2386 行：`process_signal_refusal`（⑥ 终止进程只许按一个点名的进程号）
- 第 2398 行：`detaching_refusal`（③ 把活放出追踪）
- 第 2411 行：`unwaited_ampersand_refusal`（④ run_in_background 里又把活放到了后台）
- 第 2424 行：`wait_loop_refusal`（② 前台无超时等待循环）
- 第 2436 行：`results_overwrite_refusal`（⑤ 整份覆盖 research/results/ 下未跟踪产物）
- 第 2448 行：`script_in_place_refusal`（⑦ 在同一个 inode 上改已有脚本）
- 第 2453 行：`watchdog_rejection`（① 看门狗起法不对）

（`watchdog_rejection` 在函数体最上面就被调用来算 `reasons, calls`，但它触发 `return 2` 是在其余六种都不命中之后，落在最后一行 2453；另有一处 `return 2` 在第 2351 行，是 `--scan-scripts` 的命令行用法错误分支，不算这七种拒绝之一。）

`.claude/hooks/write-guard.sh` 的 `decide()` 第 159-170 行（`def decide(...)` 到 `return 0, None, None` 加尾随空行），依次调 `decide_overwrite`（一 整份覆盖未跟踪文件）、`decide_scope`（二 越出写范围）、`decide_prime_marks`（三 写进撇号类角标）。

## 六、`crates/singlefs-harness/src/crash.rs` 的 `SparseBlockDevice` 定义（材料员现查，主工作区）

- 第 112 行：文档注释「稀疏设备当一块盘用：宿主上重跑整条路……」
- 第 113-117 行：`pub struct SparseBlockDevice { ... }`（`pub image: SparseDevice`、`size_in_bytes: u64`、`physical_block_size: singlefs_core::block_device::PhysicalBlockSizeInBytes`）
- 第 119 行起：`impl SparseBlockDevice { ... }`
- 第 133 行起：`impl singlefs_core::block_device::BlockDevice for SparseBlockDevice { ... }`
