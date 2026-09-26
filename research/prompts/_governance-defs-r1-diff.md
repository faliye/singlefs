# 附录二：governance-defs-r1 十五份定义改动（`git diff HEAD -- .claude/main-agent.md .claude/agent-common.md .claude/agents/` 原样；基准 HEAD 802fcc1，生成于 2026-09-26 13:48 UTC）

这一轮的改动没有提交点，是工作区对 HEAD（提交 802fcc1，当前 HEAD 25a1abe 只在 802fcc1 之上加了正文与开工快照两个文件，没有碰下面这批路径）的改动。15 份文件的改后全文不附，各腿直接读工作区。

## 一、diff（相对 HEAD 的工作区改动，`.claude/main-agent.md`、`.claude/agent-common.md`、`.claude/agents/` 下 13 份定义，共 15 份）

```diff
diff --git a/.claude/agent-common.md b/.claude/agent-common.md
index 83f3cf8..82d524c 100644
--- a/.claude/agent-common.md
+++ b/.claude/agent-common.md
@@ -25,13 +25,13 @@
   仓里因此有一批引用只写文件名、不写路径，那不是坏链接，是已经归档的东西。
   查法：`git log --all --diff-filter=D --name-only -- "*<文件名>*"` 找到删它的那次提交，
   `git show <提交>^:<路径>` 读当时的内容。**读到的是当时的数，不是今天的结论**——
-  拿它支撑新结论之前先重新跑一遍（`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「所有旧数据都只是参考」）；
+  拿它支撑新结论之前先重新跑一遍（`.claude/singlefs-ai-sop/rules/kb-discipline.md`「2. 每条带出处与状态」一节里「所有旧数据都只是参考」那一条）；
   要推翻早先的结论，按 `.claude/rules/three-way-inference.md` 重走一轮，不是拿旧文件对质。
 
 ## 写
 
 - 只写派发提示与定义「写范围」一节给的路径。写范围闸（项目 settings 里的 hook，表在 `.claude/hooks/agent-write-scope.tsv`）只看 Write / Edit：目标不在表里你那几行模式内的，当场拒掉；被拒之后不换 Bash 或别的办法写过去，写进报告交回主 agent。Bash 里的写闸看不见，全靠你守。
-- tools 里有 Write / Edit 的：手写的改动一律用 Edit（旧串必须在文件里恰好命中一次，与 `replace-once.py` 同义；这一轮自己新建的文件可以 `replace_all`），新建文件用 Write（先 `ls` 确认不存在），让闸看得见；Bash 里只许定义点名的脚本自己写（`relabel-item.py`、`kb-scribe` 的 `replace-batch.py`、门禁阶段的 `--write`、`mutate.sh`、`replay.sh`、cargo、跑产物的重定向）。草稿目录里（仓副本、日志、自己的辅助脚本）用什么写都行，那里只归你。
+- tools 里有 Write / Edit 的：手写的改动一律用 Edit（旧串必须在文件里恰好命中一次，与 `replace-once.py` 同义；这一轮自己新建的文件可以 `replace_all`），新建文件用 Write（先 `ls` 确认不存在），让闸看得见；tools 里只有 Edit、没有 Write 的，新建文件照下一条「tools 只有 Read / Bash 的」排他写；Bash 里只许定义点名的脚本自己写（`relabel-item.py`、`kb-scribe` 的 `replace-batch.py`、门禁阶段的 `--write`、`mutate.sh`、`replay.sh`、cargo、跑产物的重定向）。草稿目录里（仓副本、日志、自己的辅助脚本）用什么写都行，那里只归你；只有一条例外：改一个已存在的 `.sh` / `.py` 照「不做」一节检出 hook 的 ⑦ 换 inode（`replace-once.py`，或写临时文件再 `mv`）。
 - tools 只有 Read / Bash 的：新建文件一律排他，`set -o noclobber` 后 `cat > 文件 <<'EOF'`，文件已存在就停下报告；之后 `>>` 追加。
 - 报告分段写，每一次写进文件的内容不超过 150 行（`.claude/rules/three-way-inference.md`「云端腿的报告要分段落盘」）；表格与代码块整块放进同一段，不从中间切。
 - tools 只有 Read / Bash 的，改已有文件只用定点替换：`research/scripts/replace-once.py` 或 `research/scripts/replace-batch.py`（先 `--dry-run`），不整份重写。
@@ -43,7 +43,7 @@
 - 不建、不改 `.claude/agents/`、`.claude/settings*.json`、`.claude/hooks/`、`.claude/rules/`、`.claude/singlefs-ai-sop/`，也不碰 `~/.claude/`。
 - 不读派发提示给的禁读清单里的文件。
 - 不编译 Rust、不跑 `gate.sh` 全量，除非定义明写要做。跑脚本、跑模型一律加 `nice -n 19`。
-- 重型测试（层 0、QEMU、herd7、crates 变异整表、全量 `cargo test`（`--all` / `--workspace` / 工作区根裸跑）与 `check.sh`、`gate.sh` 整轮、87 号全部实验复跑、E152 装置）只在提交代码时跑一次、或用户要求时跑；子 agent 一律不跑，只跑自己动到的测试二进制（`cargo test -p <crate> --test <自己的目标>`、`--lib`）、fmt / clippy / build 与 `.claude/gate.d/` 下 54、55、57、59、87 之外的阶段。`crash-verifier` 只跑 54、55、57、59 号那几道，`gate-triage` 只跑 `gate.sh` 整轮与 87 号，都在提交时或用户要求时跑、命令带 `SINGLEFS_HEAVY_TESTS=commit`（用户要求时 `=user-request`）。项目 settings 里的 `.claude/hooks/heavy-test-guard.sh` 在执行前拒绝越出这些的命令；被拒了不换写法绕过去，在交回里写明要跑什么、为什么，交主 agent 去问用户。
+- 重型测试（哪些算重型以 `.claude/rules/implementation-workflow.md`「重型测试只在提交时跑」开头那一句为准，逐条的判法在 `.claude/hooks/heavy-test-guard.sh` 文件头）只在提交代码时跑一次、或用户要求时跑；子 agent 一律不跑，只跑自己动到的测试二进制（`cargo test -p <crate> --test <自己的目标>`、`--lib`）、fmt / clippy / build 与 `.claude/gate.d/` 下 54、55、57、59、87 之外的阶段（15、74 号虽然也跑 cargo test，按轻阶段对待）。子 agent 跑 `cargo test` / `cargo run` 与直接执行编出来的二进制，一律经内存包装：`bash research/scripts/run-with-memory-cap.sh <上限> <命令>`（上限照 systemd 写法，派发提示没给的用 `4G`；退出码 250 是撞了这一条的上限，照实报主 agent，不自己调大重跑），不经它的由 `heavy-test-guard.sh` 拒。重型测试里 `crash-verifier` 只跑 54、55、57、59 号那几道，`gate-triage` 只跑 `gate.sh` 整轮与 87 号，都在提交时或用户要求时跑、命令带 `SINGLEFS_HEAVY_TESTS=commit`（用户要求时 `=user-request`）。项目 settings 里的 `.claude/hooks/heavy-test-guard.sh` 在执行前拒绝越出这些的命令；被拒了不换写法绕过去，在交回里写明要跑什么、为什么，交主 agent 去问用户。
 - 写进脚本文件、放进 `python3` 的 subprocess、`make` 这类间接起法同样算跑：自己写的验证链里不许出现重型测试的任何一步，名字含 `layer0` 的测试二进制也不许。
 - 要停自己起的一条链（脚本、后台任务）时，先列出它的整棵子进程（`ps -o pid,ppid,args --ppid <pid>` 逐层往下），从最底层起逐个 `python3 .claude/singlefs-ai-sop/scripts/proc.py stop <pid>`（`proc.py` 只停给定的那一个，不带子进程），停完再列一遍，确认一个不剩；不许留下正在跑的那一步「让它跑完」。
 - 要编译、跑门禁阶段、跑变异或产物的，开跑前 `ps -o pid,args -u "$(id -u)"` 看负载：有性能测量在跑（`qemu-system`、`vm-bench.sh`、`e152-file-system-benchmark`、`fio`）就报「在等 pid …（命令）」停下；只有别的 `cargo`、`gate.sh` 在跑的，加 `nice -n 19` 照常跑（cargo 自己排队拿文件锁），报告里记下 `ps` 看到了什么、等锁等了多久。定义另有更严要求的照定义。
@@ -52,9 +52,9 @@
 - **崩溃点测试不衡量时间成本，也不为省时间缩范围**：要加崩溃点、要多枚举一档，就加。挂钟长不是收窄它的理由，也不用先算它值不值。
 - 禁止自行扩大任务范围，只改自己的部分。一个任务一个出口，达成出口即终止任务，扩大任务必须弹窗。禁止因为产物变化无限继续任务。尤其是门禁检测等任务，不属于自己任务的验证，如果验证为红给其他任务发消息，严禁自行修改扩大任务范围。
 - 本机常有别的会话在同一个仓里干活：只动这一轮自己的文件；看到别人没提交的改动不碰、不修。
-- 长活可以等，不给它设超时、不自己中途杀掉：编译、变异表、复跑这类长活用 Bash 的 `run_in_background` 起，起完结束本轮，完成时会通知你。交给它的命令要一直跑到真正的活结束才退出：不在里面再把活放到后台——不用 `disown`、`coproc`、`setsid -f`、`nohup … &`、`tmux`/`screen` 的分离模式、`systemd-run`，子 shell 或 `$( … )` 里也不写 `&`；`&` 只在同一条命令随后用不带参数的 `wait` 等齐时用。结束本轮就是这一条回复只写一句在等什么、不再调工具；前台命令的 `timeout` 不超过 240000 毫秒。
+- 长活可以等，不给它设超时、不自己中途杀掉：编译、变异表、复跑这类长活用 Bash 的 `run_in_background` 起，起完结束本轮，完成时会通知你。交给它的命令要一直跑到真正的活结束才退出：不在里面再把活放到后台——不用 `disown`、`coproc`、`setsid -f`、`nohup … &`、`tmux`/`screen` 的分离模式、`systemd-run`，子 shell 或 `$( … )` 里也不写 `&`；`&` 只在同一条命令随后逐个 `wait "$pid"` 取回每个作业的退出码时用（不带参数的 `wait` 恒返回 0，会把失败吞掉，`.claude/singlefs-ai-sop/rules/command-safety.md`「并行不许把失败吃掉」）。结束本轮就是这一条回复只写一句在等什么、不再调工具；前台命令的 `timeout` 不超过 240000 毫秒。
   等长活时不起缓存计时器，结束本轮直接等完成通知。
-  等的时候看到进程不动、报错、或等的条件不会成立，把看到的命令、输出与时刻写进草稿目录里的进度记录，接着做定义里还能做的；结不结束、怎么处置由主 agent 定。主 agent 的看门狗（`research/scripts/agent-watch.py`）定时复检你；项目 settings 里的 Bash 检出 hook（`.claude/hooks/bash-command-detector.sh`）对七种写法在执行前拒绝（逐条的判据写在它的文件头「拒绝七种」一节）：前台没有超时的等待循环、把活放出追踪（`disown`、`nohup … &` 这类）、run_in_background 里以单独的 `&` 收尾而之后同一条命令里没有 `wait` 的作业、用 `>`、不带 `-a` 的 `tee`、`cp` / `mv` / `install` 整份覆盖 `research/results/` 下已存在又没进 git 的产物（要换就按日期另存新文件名，旧的留着）、起看门狗的错误写法（只有主 agent 起看门狗）、终止进程时不是一次点名一个自己起的进程号或任务号（按名字挑、按进程组或 cgroup 批量、在循环里逐个发信号、停 ssh 与会话相关的服务，都拒；要停就照「要停自己起的一条链」那一条逐个 `proc.py stop`）、在同一个 inode 上改一个已存在的脚本（`>`、不带 `-a` 的 `tee`、`cp` 覆盖、python 里 `open(…, "w")` 改 `.sh` / `.py`；改用 `research/scripts/replace-once.py`，或写临时文件再 `mv` 换上）；run_in_background 里的等待循环与按模式找进程这类命令只记检出，交给主 agent 判断。等日志里的一行字之前，先确认那一行真会写进那个文件；找进程用写死的 pid，不用 `pgrep -f` / `pkill -f`（command-safety.md）。
+  等的时候看到进程不动、报错、或等的条件不会成立，把看到的命令、输出与时刻写进草稿目录里的 `progress.md`，接着做定义里还能做的；结不结束、怎么处置由主 agent 定。主 agent 的看门狗（`research/scripts/agent-watch.py`）定时复检你；项目 settings 里的 Bash 检出 hook（`.claude/hooks/bash-command-detector.sh`）对七种写法在执行前拒绝（逐条的判据写在它的文件头「拒绝七种」一节）：前台没有超时的等待循环、把活放出追踪（`disown`、`nohup … &` 这类）、run_in_background 里以单独的 `&` 收尾而之后同一条命令里没有 `wait` 的作业、用 `>`、不带 `-a` 的 `tee`、`cp` / `mv` / `install` 整份覆盖 `research/results/` 下已存在又没进 git 的产物（要换就按日期另存新文件名，旧的留着）、起看门狗的错误写法（只有主 agent 起看门狗）、终止进程时不是一次点名一个自己起的进程号或任务号（按名字挑、按进程组或 cgroup 批量、在循环里逐个发信号、停 ssh 与会话相关的服务，都拒；要停就照「要停自己起的一条链」那一条逐个 `proc.py stop`）、在同一个 inode 上改一个已存在的脚本（`>`、不带 `-a` 的 `tee`、`cp` 覆盖、python 里 `open(…, "w")` 改 `.sh` / `.py`；改用 `research/scripts/replace-once.py`，或写临时文件再 `mv` 换上）；run_in_background 里的等待循环与按模式找进程这类命令只记检出，交给主 agent 判断。等日志里的一行字之前，先确认那一行真会写进那个文件；找进程用写死的 pid，不用 `pgrep -f` / `pkill -f`（command-safety.md）。
 
 ## 门禁
 
diff --git a/.claude/agents/crash-verifier.md b/.claude/agents/crash-verifier.md
index a0983a7..0cfcaa3 100644
--- a/.claude/agents/crash-verifier.md
+++ b/.claude/agents/crash-verifier.md
@@ -22,9 +22,9 @@ omitClaudeMd: true
 ## 做什么
 
 1. 每个阶段开跑前各照共用约束「不做」一节看一次负载；另有别的 `gate.sh` 正在跑 54 号、或有 `54-layer0-replay.sh --full` 在跑时，不起 54 号，等它结束。
-2. 只在提交时（或主 agent 转达用户要求时）跑你那几道，不跑 `gate.sh` 整轮与全量 `cargo test`（`.claude/hooks/heavy-test-guard.sh` 拒）。按共用约束 `.claude/agent-common.md`「门禁」一节从阶段归属表取登记给你的阶段，一次只跑一个；54、55、57、59 号命令带输入给的那个前缀：`SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/<文件>`，其余登记给你的轻阶段不带；54 号默认不带 `--full`（快档加核全绿标记），派发提示点名要层 0 全量时才带；每个记开始与结束时刻（`date -u`）和退出码。单个阶段超过 Bash 单次上限就后台跑，退出码写进文件（`{ SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/<文件>; echo "exit=$?"; } > <草稿目录>/<阶段>.log 2>&1`；退出码只认日志里的 `exit=` 那一行，不认起后台的那条命令自己的 `$?`），结束后读输出。别的会话同时在跑 cargo 时，编译会卡在「Blocking waiting for file lock」，照实记等了多久，不算这个阶段的耗时。
+2. 只在提交时（或主 agent 转达用户要求时）跑你那几道，不跑 `gate.sh` 整轮与全量 `cargo test`（`.claude/hooks/heavy-test-guard.sh` 拒）。按共用约束 `.claude/agent-common.md`「门禁」一节从阶段归属表取登记给你的阶段，一次只跑一个；54、55、57、59 号命令带输入给的那个前缀：`SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/<文件>`，其余登记给你的轻阶段不带；54 号只跑快档（不带 `--full`，快档加核全绿标记）；层 0 全量由主 agent 在 HEAD + 暂存区的 worktree 里自己跑，不归你；每个记开始与结束时刻（`date -u`）和退出码。单个阶段超过 Bash 单次上限就后台跑，退出码写进文件（`{ SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/<文件>; echo "exit=$?"; } > <草稿目录>/<阶段>.log 2>&1`；退出码只认日志里的 `exit=` 那一行，不认起后台的那条命令自己的 `$?`），结束后读输出。别的会话同时在跑 cargo 时，编译会卡在「Blocking waiting for file lock」，照实记等了多久，不算这个阶段的耗时。
 3. 每个阶段原样抄它的「✓」行，或「✗」行与紧跟的「→」；退出码 77 记「本次未跑」，不记通过。
-4. 计数行原样抄：崩溃状态数、恢复结果、与 E142 产物或闭式比对的那几行。输出里读不到判定行的阶段记「作废」，不记通过。
+4. 计数行原样抄：崩溃状态数、恢复结果、与 E142（第一个事务的干跑） 产物或闭式比对的那几行。输出里读不到判定行的阶段记「作废」，不记通过。
 5. 缺 herd7、缺 KVM、虚机起不来这一类判「环境」，以阶段自己的报错为准，另贴 `command -v herd7`、`ls -l /dev/kvm`、`command -v qemu-system-x86_64` 的原样输出作旁证。这三条命令不单独下判断。
 6. 每个阶段开跑与结束时各记一次指纹，开跑那次与起阶段写在同一条 Bash 命令里：`git rev-parse HEAD`、`git diff HEAD -- crates litmus | sha256sum`（暂存与没暂存的都算）、`git ls-files --others --exclude-standard -z -- crates litmus | xargs -0 -r sha256sum | sha256sum`（未跟踪文件按内容，不只按名单），前后或阶段之间不同，就写明「这几个绿对应的不是同一份源码」、列出哪几个阶段之间变了。
 
diff --git a/.claude/agents/experiment-designer.md b/.claude/agents/experiment-designer.md
index 59d600a..73a98f8 100644
--- a/.claude/agents/experiment-designer.md
+++ b/.claude/agents/experiment-designer.md
@@ -19,13 +19,13 @@ omitClaudeMd: true
 - 要回答的问题（一种读法；有两种读法的先交回主 agent 定问法。两种读法都报、又不改变任何判据时，可以两种都登记，文件头挂一句「问法待主 agent 在装置写之前删一种」；主 agent 删完再派执行员，执行员见到这句不开工）。
 - 重跑已有实验的：实验号与第几次。这时不占号，写重跑登记 `research/prompts/e<号>-r<n>-prereg.md`，节名照「登记的固定节名」那一节。
 - 被测条款：kb 文件路径与小节标题。
-- 实验简称（或由你按问题起一个，交主 agent 认）。
+- 实验简称（或由你按问题起一个，交主 agent 认）；另在登记里定一个英文名（小写字母与数字，词之间下划线），源文件、变异表与 `[[bin]]` 名用它，中文简称只用在 kb 页。
 - 草稿目录。
 
 ## 做什么
 
 1. 先读被测条款与它的定义逐字（定义不是答案），判问法有没有两种读法；读 `crates/` 里对应的实现。要交回主 agent 定问法的，这时候交回，不占号。重跑的不跑 `claim-experiment.sh`：`set -o noclobber` 后建重跑登记，文件已存在就停下报告；原登记要读（判据怎么定的），实验页与产物照样不读结果与结论。
-2. `bash research/scripts/claim-experiment.sh --next` 取号，`bash research/scripts/claim-experiment.sh E<号> <简称>` 占住（它建 `research/prompts/e<号>-preregistration.md`）。不读 `.claude/kb/experiments/` 下已有实验的结果与结论小节，不读 `research/results/`；全仓搜条款与用词时加 `--exclude-dir=experiments --exclude-dir=results --exclude-dir=prompts`。`mutation-sampling.md` 第五类要查「这个量仓里有没有人算过、分母是不是同一个」：只读别的实验怎么算（口径、分母那几行），不读它的结果与结论。读过的每个文件列进登记，grep 命中行也算读过；还是读到了已有的数、或判问法时自己算出了答案，列进登记固定的一节「跑之前已经存在的数」，不删。
+2. `bash research/scripts/claim-experiment.sh --next` 取号，`bash research/scripts/claim-experiment.sh E<号> <简称>` 占住（它建 `research/prompts/e<号>-preregistration.md`）。不读 `.claude/kb/experiments/` 下已有实验的结果与结论小节，不读 `research/results/`；全仓搜条款与用词时加 `--exclude-dir=experiments --exclude-dir=results --exclude-dir=prompts --exclude=experiments.md --exclude=experiments-history.md`（实验索引的结论列与实验变更史是文件，`--exclude-dir` 排不掉）；`records/`、`.claude/kb/decisions-history/`、`research/perf-by-milestone.md` 里命中的结果行排不干净，读到了照下一句列进「跑之前已经存在的数」。`mutation-sampling.md` 第五类要查「这个量仓里有没有人算过、分母是不是同一个」：只读别的实验怎么算（口径、分母那几行），不读它的结果与结论。读过的每个文件列进登记，grep 命中行也算读过；还是读到了已有的数、或判问法时自己算出了答案，列进登记固定的一节「跑之前已经存在的数」，不删。
 3. 登记写明：问题，连同岔路单逐行抄进来，第六节每个量写明它对应岔路单的哪一行、取什么值会让那一行翻面——对不上任何一行的量不登记，确要顺带量的标「附带，够判后不跑」；每条臂的定义（「怎么做」与「做完会怎样」要推得出，对面那条臂写成支持它的人认的样子）；阳性对照（每条臂都跑）与真实基线；每个量各占一行的判据与门槛（门槛不能从臂的定义直接推出）；钉绝对值的断言；被谓词消费的量报峰值、为正轮数、期末值；几何敏感性那一行（至少一个方向相反的取样点）；失败条款，每条紧跟「什么观测会让它触发」；作废条款；装置与 `crates/` 实现对不上时的停机条款（`implementation-first.md` 第 4 条：两边都查，既不作废也不当结果）。纯算术题里不适用的格（没有臂、没有轮次）写明为什么不适用，不硬凑。钉绝对值的锚点分两类写：出自被测条款本身的（不符时走「条款可能错」的失败条款）与独立算出、用命令核过的（不符才作废）。给执行员列的每条变异，写明它在哪个取样点上改变输出。**装置写在哪要在登记里写死一句**：默认走 `research/e7-index-bench/src/bin/`（独立手写模型，不与实现共用代码，`implementation-first.md` 第 4 条）；**只有当这条岔路问的就是「`crates/` 今天这份代码的性质」、另写一份独立模型答不了时**，才写死成「入库装置」（执行员改写 `crates/singlefs-harness/src/bin/`）。写死成入库装置的，登记里同时写明这条实验哪些量要读 `crates/` 的哪些常量——执行员按它逐个改成本地常量加回比断言，不许直接引。⚠️ 登记里没写死这一句的，执行员一律走 `research/` 那一支，不自己判。计时类的量写明在哪个进程里、用谁的时钟计：在被测进程里自己计时并写进结果行，或先把子进程输出整份读完再转打；不拿「读一行、打时间戳、转打、再读下一行」那个循环里行到达的时刻当分段挂钟（照 `.claude/kb/vm-harness.md`「计时不在转发输出的循环里打时间戳」一节办）。
 3b. 估一下实现量：历史族 × 几何格 × 臂 × 崩溃支线 × 坏镜像，加上要另写的子系统（检查器、触发式重算、崩溃枚举）。按岔路排序：能最先让岔路单里某一行够判的量放在最前，连同停机条款（与 `crates/` 的逐项对拍）与钉绝对值的锚点；一个执行员一次做不完的，在第五节末尾分段写「第一段 / 第二段 / …」，每段写明它补岔路单哪一行、补完那一行够不够判。分段不是「全部做完」的计划：每段交回之后主 agent 对着岔路单判续不续，一行开着的都不剩就停，后面的段不跑。
 
diff --git a/.claude/agents/experiment-runner.md b/.claude/agents/experiment-runner.md
index f4916bf..f6e589b 100644
--- a/.claude/agents/experiment-runner.md
+++ b/.claude/agents/experiment-runner.md
@@ -24,15 +24,15 @@ omitClaudeMd: true
 ## 做什么
 
 1. 开跑前照共用约束「不做」一节看负载。登记第六节里有耗时类的量时，跑产物之前连别的 `cargo`、`gate.sh` 也要等。
-2. 写 `research/e7-index-bench/src/bin/e<号>_<简称>.rs`，在 `research/e7-index-bench/Cargo.toml` 末尾追加它的 `[[bin]]`（⚠️ **跑前登记把装置写死成「入库装置」时改写 `crates/singlefs-harness/src/bin/e<号>_<简称>.rs`**：岔路问的是「`crates/` 今天这份代码的性质」时，另写一份独立模型答不了，写范围闸按 `crates/singlefs-harness/src/bin/e*.rs` 放行。那种 bin 只读、只驱动与观测，不许改 `crates/singlefs-core` 与 `crates/singlefs-checker`；变异表改追加进 `crates/mutations.tsv` 末尾、归门禁 59 号跑。**另有三条只对入库装置生效**：① **不许从 `crates/` 里引决定工作量、几何或判据门槛的常量**——这类参数在装置里写本地常量，值抄自 kb 的登记位并注明文件与行号，另加一条断言拿它与实现侧同名常量回比；引了就等于装置与实现共用一个数，两边一起错而逐格相等。这一条挡不住「kb 与实现一起取错」。② **写了入库装置就要让它进代码轮判决的点名清单**：门禁 56 号的正则罩得到 `crates/*/src/bin/*.rs`，不点名那一道会红，而那条义务落在跑整轮门禁的一方身上、不落在你身上——交回时写明你新建了哪个 bin。③ **什么算「入库装置」以跑前登记里写死的那一句为准**；登记里没写死就走 `research/` 那一支，别自己判）（`name` 用连字符，与 `replay.sh` 登记行里的 bin 名一致；只追加，不改别人的）：纯算术的计数模型、单测、钉绝对值的断言；取样点覆盖登记里每一个会跑到的取值。写完跑 `bash .claude/scripts/naming-lint.sh`，新文件里的缩写与单字母名在这一步改完再往下走，不留给后面的门禁阶段。
-3. 写变异表 `research/mutations/e<号>_<简称>.tsv`，在 `research/` 下跑完整张表：`cd research && nice -n 19 bash scripts/mutate.sh <bin 名> e7-index-bench/src/bin/<源文件> mutations/<变异表>`（两个路径都相对 `research/`；bin 名以 `research/e7-index-bench/Cargo.toml` 里 `[[bin]]` 的 `name` 为准，没有显式声明的取文件名去掉 `.rs`；报「基线就是红的」时先查是不是在仓根下跑、bin 名是不是写错）（收尾要有「已还原，基线仍全绿」）；没红的逐条按 `mutation-sampling.md` 分类。
+2. 写 `research/e7-index-bench/src/bin/e<号>_<英文名>.rs`（源文件、变异表、`[[bin]]` 的 `name` 一律用跑前登记里定的英文名：源文件与变异表写蛇形 `e<号>_<英文名>`，`name` 写连字符 `e<号>-<英文名>`；中文简称只用在 kb 页），在 `research/e7-index-bench/Cargo.toml` 末尾追加它的 `[[bin]]`（⚠️ **跑前登记把装置写死成「入库装置」时改写 `crates/singlefs-harness/src/bin/e<号>_<英文名>.rs`**：岔路问的是「`crates/` 今天这份代码的性质」时，另写一份独立模型答不了，写范围闸按 `crates/singlefs-harness/src/bin/e*.rs` 放行。那种 bin 只读、只驱动与观测，不许改 `crates/singlefs-core` 与 `crates/singlefs-checker`；变异表改追加进 `crates/mutations.tsv` 末尾、归门禁 59 号跑。**另有三条只对入库装置生效**：① **不许从 `crates/` 里引决定工作量、几何或判据门槛的常量**——这类参数在装置里写本地常量，值抄自 kb 的登记位并注明文件与行号，另加一条断言拿它与实现侧同名常量回比；引了就等于装置与实现共用一个数，两边一起错而逐格相等。这一条挡不住「kb 与实现一起取错」。② **写了入库装置就要让它进代码轮判决的点名清单**：门禁 56 号的正则罩得到 `crates/*/src/bin/*.rs`，不点名那一道会红，而那条义务落在跑整轮门禁的一方身上、不落在你身上——交回时写明你新建了哪个 bin。③ **什么算「入库装置」以跑前登记里写死的那一句为准**；登记里没写死就走 `research/` 那一支，别自己判）（`name` 用连字符，与 `replay.sh` 登记行里的 bin 名一致；只追加，不改别人的）：纯算术的计数模型、单测、钉绝对值的断言；取样点覆盖登记里每一个会跑到的取值。写完跑 `bash .claude/scripts/naming-lint.sh`，新文件里的缩写与单字母名在这一步改完再往下走，不留给后面的门禁阶段。
+3. 入库装置那一支不写 research 的变异表、不跑 `mutate.sh`（它只编得到 `research/` 下的 bin，整表跑 `crates/mutations.tsv` 又是重型）：变异行照 `crates/mutations.tsv` 第 1 行表头的六段格式追加，报告里列出追加的变异名、三个数写「待提交时 59 号」。`research/` 那一支写变异表 `research/mutations/e<号>_<英文名>.tsv`，在 `research/` 下跑完整张表：`cd research && nice -n 19 bash scripts/mutate.sh <bin 名> e7-index-bench/src/bin/<源文件> mutations/<变异表>`（两个路径都相对 `research/`；bin 名以 `research/e7-index-bench/Cargo.toml` 里 `[[bin]]` 的 `name` 为准，没有显式声明的取文件名去掉 `.rs`；报「基线就是红的」时先查是不是在仓根下跑、bin 名是不是写错）（收尾要有「已还原，基线仍全绿」）；没红的逐条按 `mutation-sampling.md` 分类。
 3b. 登记里的停机条款（第十一节里标「停机」的，通常是装置与 `crates/` 实现逐项对拍）在跑产物之前做，照原文真跑，不许只推；做不了就停下交回，不跑产物、不写实验页。登记的量一次做不完的，做完的那一段照常交，没做的逐条列进报告，实验页与索引行的状态写「部分已跑」，不写「已跑」。岔路单每一行都够判了（登记第十一节的够判停机），剩下的量不跑：实验页逐条标「够判后未跑」、写明各自对应哪一行，状态写「已跑（够判）」；够不够判由你按登记写的够判条件报，续不续由主 agent 定。
 4. 跑产物进 `research/results/`；跑前删旧输出，跑完认本轮才有的完成标记。重跑已有实验时，`research/results/` 里已有的产物不覆盖：与它逐字节一致就不新存；对不上就按今天的日期另存一份，实验页两份都点名、写明对不上，交主 agent 定哪份承重。生成产物用的二进制编在 `research/target/` 下，用最后一次源码改动编出来；生成产物与第 5 步跑 `replay.sh` 时都不设 `CARGO_TARGET_DIR`；草稿目录里的 target 只拿来迭代。
 4b. 产物出来之后先自查齐不齐：按登记的「臂 × 历史 × 几何」数一遍产物里每一类结果行的条数与字段，缺行、缺字段（例如某一族没有臂字段）、某一族一次都没造出被测形状，都列进报告，不许只看末行完成标记。报告里每个判据按每条臂全列判定，不挑「突出发现」报——对某条候选不利的那一半同样要写。分族、分臂的计数用一条命令从产物里数，报告与实验页贴那条命令和它的原样输出，不用文字重写族名单；给「这几族为什么是 0」写解释之前，先把那几族的产物行贴出来，确认它们真是 0。
-5. 在 `research/scripts/replay.sh` 里登记复跑，跑一次 `bash research/scripts/replay.sh E<号>` 确认逐字节一致。送进虚机跑的实验（`vm-bench.sh`，含 `--selftest`）与 E152 装置是重型测试，你不跑：装置、登记与复跑命令写好就停，在交回里写明要跑什么、为什么，由主 agent 问用户（`.claude/kb/vm-harness.md`；共用约束「不做」一节里重型测试那一条）。装置要起子进程、读它的输出时，先把输出读完再转打，计时按登记写的进程与时钟取（`.claude/kb/vm-harness.md`「计时不在转发输出的循环里打时间戳」）；共享 `gate.sh` 的「转发计时」阶段查读子进程输出的循环里一边取时间一边输出。
+5. 在 `research/scripts/replay.sh` 里登记复跑，跑一次 `bash research/scripts/replay.sh E<号>` 确认逐字节一致。送进虚机跑的实验（`vm-bench.sh`，含 `--selftest`）与 E152（按里程碑对比六家文件系统的文件性能） 装置是重型测试，你不跑：装置、登记与复跑命令写好就停，在交回里写明要跑什么、为什么，由主 agent 问用户（`.claude/kb/vm-harness.md`；共用约束「不做」一节里重型测试那一条）。装置要起子进程、读它的输出时，先把输出读完再转打，计时按登记写的进程与时钟取（`.claude/kb/vm-harness.md`「计时不在转发输出的循环里打时间戳」）；共享 `gate.sh` 的「转发计时」阶段查读子进程输出的循环里一边取时间一边输出。
 6. 跑阶段归属表登记给你的阶段（共用约束 `.claude/agent-common.md`「门禁」一节）；其中要编译或复跑整张表的，开跑前照共用约束看负载。15 号（编整个 research 工作区）与 87 号（复跑全部已入库实验）不归你，收尾由 `gate-triage` 统一跑；你只跑第 5 步自己那一个实验的 `replay.sh`。
 7. 主 agent 要写实验页时：`.claude/kb/experiments/<号>-<简称>.md`，结果整行抄自产物并带文件名，写复跑命令、口径、它答不了的；索引行进 `.claude/kb/experiments.md`。
-7b. 实验页的 `### 影响的决策` 一节照第 7 步写实验页的规则写，形态与判据不在这份定义里复述。这一步你要做的三件：① 要写支撑或推翻时，先 `grep -n 'E<号>（' .claude/kb/decisions/<那条决策的文件>` 看那条分项的「**依据**」段引没引这个实验——引了才写支撑或推翻，没引就写备料，并把「这一格该不该升成支撑」列进报告交主 agent，**不自己去改决策文件**（写范围闸也不放行）。② 那条决策还在 `.claude/decision-links-pending` 里时不查依据段（门禁 75 号对清单里的决策整段跳过双向检查，那些决策还没有依据段），写决策级一行，并把它列进报告。③ 重跑已有实验、换了产物、加了历史条目之后，表里每一行都重新回看一遍。
+7b. 实验页的 `### 影响的决策` 一节照第 7 步写实验页的规则写，形态与判据不在这份定义里复述。这一步你要做的三件：① 要写支撑或推翻时，先 `grep -n 'E<号>（' .claude/kb/decisions/<那条决策的文件>` 看那条分项的「**依据**」段引没引这个实验——引了才写支撑或推翻，没引就写备料，并把「这一格该不该升成支撑」列进报告交主 agent，**不自己去改决策文件**（写范围闸也不放行）。② 那条决策还在 `.claude/decision-links-pending` 里时（那份清单只减不增，一条都没有时跳过这一步）不查依据段（门禁 75 号对清单里的决策整段跳过双向检查，那些决策还没有依据段），写决策级一行，并把它列进报告。③ 重跑已有实验、换了产物、加了历史条目之后，表里每一行都重新回看一遍。
 
 ## 写范围
 
@@ -40,7 +40,7 @@ omitClaudeMd: true
 
 ## 产出
 
-- 报告：单测数（一条命令数出来）、变异三个数与分类、产物路径与完成标记、`replay.sh` 结果、实验页路径；登记修订了什么（没有就写没有）。
+- 报告：单测数（一条命令数出来）、变异三个数（抓到 / 无效 / 没红）与分类，另报 `mutate.sh` 收尾的内存撞顶与超时（不为 0 的整轮已判失败）、产物路径与完成标记、`replay.sh` 结果、实验页路径；登记修订了什么（没有就写没有）。
 - 岔路表：登记里岔路单的每一行一行，写「已够判 / 还差什么 / 登记里剩下的量能不能让它翻面」，每格指到产物里算出它的那条命令。主 agent 续派时照这张表点名还差的行；这张表缺了，报告不算交齐。
 
 ## 没做什么（固定会有的）
diff --git a/.claude/agents/gate-triage.md b/.claude/agents/gate-triage.md
index 60c48be..6cc62bc 100644
--- a/.claude/agents/gate-triage.md
+++ b/.claude/agents/gate-triage.md
@@ -23,13 +23,13 @@ omitClaudeMd: true
 ## 做什么
 
 1. 开跑前照共用约束「不做」一节看负载；另有别的 `gate.sh` 在跑就等它结束。
-2. 只在提交时（或主 agent 转达用户要求时）跑：暂存过的跑 `SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/scripts/gate.sh --staged`（用户要求时 `=user-request`）；主 agent 明写全量的跑不带参数（前缀照带）。54、55、57、59 这几道重阶段在 `gate.sh` 里照各自的复用判定走（54 号只核全绿标记），你不直接调它们，也不跑全量 `cargo test`（`.claude/hooks/heavy-test-guard.sh` 拒）；登记给你的其余阶段单跑时不带前缀，87 号单跑照带。超过 Bash 单次上限时后台跑、结束后读输出。
+2. 只在提交时（或主 agent 转达用户要求时）跑：暂存过的跑 `SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/scripts/gate.sh --staged`（用户要求时 `=user-request`）；主 agent 明写全量的跑不带参数（前缀照带）。54、55、57、59 这几道重阶段在 `gate.sh` 里各走各的：54 号跑快档并核全绿标记，55、59 照 `.claude/gate.d/stage-inputs.tsv` 复用上一次全绿判定，57 号没有复用、每次现跑；你不直接调它们，也不跑全量 `cargo test`（`.claude/hooks/heavy-test-guard.sh` 拒）。登记给你的其余阶段已在 `gate.sh` 整轮里跑过，不再单跑；主 agent 点名要单跑某一道时才单跑，轻阶段不带前缀，87 号照带。超过 Bash 单次上限时后台跑、结束后读输出。
 3. 每个红阶段：抄门禁给的「✗」行与紧跟的「→」下一步原样；看它点名的文件与行在不在暂存区 diff 的块里（`git diff --cached -U0 -- <文件>`）。在块里 ⇒「这一轮」；文件在但行不在块里、或文件不在暂存区 ⇒「不是这一轮」；红句说的是「这一批输入」本身的（54 号的「没有这批输入的全绿标记」「那一格的哈希不同」这类）不按它点名的标记路径判：这一轮的改动（`git diff --cached --name-only`；跑全量工作区时是主 agent 给的文件清单）碰了 `.claude/gate.d/stage-inputs.tsv` 里那一道那一行登记的路径 ⇒「这一轮」，下一步原样抄 54 号出路里的三行命令，一条都没碰 ⇒「不是这一轮」；工具没装、网关不通、`cargo` 不在 ⇒「环境」，用 `bash .claude/scripts/env.sh` 的对应行佐证；别的会话同时在跑同一个重阶段、进程被外部信号杀掉（输出里一行裸的 `Terminated`）⇒「并发」，贴开跑时与出事前后 `ps` 看到的同名进程。红阶段没有「✗」与「→」可抄的（被杀、崩溃），抄它最后 20 行输出，判「分不清」或「并发」，写明没有下一步可抄。分不清的写「分不清」与原因。
 4. 原样抄未实现清单与「由项目本地阶段覆盖」清单。别的会话同时在改仓、跑全量工作区时「工作区跑的过程中没变」那一条红了：照抄开跑、收尾两个指纹，判「不是这一轮」。`gate.sh` 不打印单个阶段的耗时，取不到的不估。
 
 ## 写范围
 
-- 报告文件、草稿目录。门禁自己的 git 写（共享 `gate.sh` 全绿时 `update-ref refs/singlefs/gate-ok`、`--staged` 时的临时 worktree）是门禁本身的行为，允许；你自己不做任何 git 写。
+- 报告文件、草稿目录。门禁自己的 git 写（共享 `gate.sh` 不带 `--staged` 全绿时 `update-ref refs/sop/gate-ok`（带 `--staged` 时不写）、`--staged` 时的临时 worktree）是门禁本身的行为，允许；你自己不做任何 git 写。
 
 ## 产出
 
diff --git a/.claude/agents/implementation-writer.md b/.claude/agents/implementation-writer.md
index 9e38948..c26e20d 100644
--- a/.claude/agents/implementation-writer.md
+++ b/.claude/agents/implementation-writer.md
@@ -24,8 +24,8 @@ omitClaudeMd: true
 
 1. 开跑前照共用约束「不做」一节看负载。
 2. 读条款与代码，写实现与测试。名字、分支、类型、错误照 `code-discipline.md`：不缩写、不写 `_ =>`、newtype、`expect` 写清依赖哪条不变量。
-3. 每条新测试证明会红：在草稿目录的仓副本里（`rsync -a --exclude target --exclude .git`）改坏被测代码一处，跑那条测试所在的整个测试二进制看它红，记「改坏哪一行 → 哪条断言红」与同时红了哪些测试；每份副本用它自己的 target（不把 `CARGO_TARGET_DIR` 指到几份副本共用的目录）；改坏的副本要还原时，从原件拷回之后对被改的文件 `touch`，或删掉这份副本重拷，不用 `rsync -a` 拷回了事；先跑一份不改动的副本：已经红的测试记成基线红集，变异证明只看基线红集之外的测试，你要证明的那条在基线红集里就停下交回；被测代码里有 `debug_assert` 的，debug 下先红的可能是它，照实记红在哪一条，要测试断言自己红就在 `--release` 下再跑一次。`crates/mutations.tsv` 在的话（门禁 59 号），每条再加一行变异，原文在文件里恰好命中一次。
-4. 变异证红按测试算、不按行算：每条新测试挑一行能让它红的变异证一次就够，其余行照样追加进 `mutations-append.tsv`，整表复跑归最后的门禁 59 号；报告写明哪几行证过、哪几行留给 59 号。层 0 不跑：自己新加的层 0 流不在交回前跑，随提交时的层 0 全量验；已有流一条都不跑；新测试落在名字含 layer0 的测试二进制里的，第 3 步的证红也不跑，变异行照样追加、报告写明留给提交时的 59 号。层 0 全量每次提交跑一次（`main-agent.md`「什么时候派哪个 agent」里「暂存之后、提交之前跑门禁」那一步），不由实现员跑。交回前的验证只到这几样：动到的测试二进制（整个二进制，不按名字挑；名字含 layer0 的不跑）、第 3 步的证红、`cargo fmt --check`、`check.sh` 那一套 lint 下的 `cargo clippy`、`cargo build --offline --all-targets`，各贴末尾原样输出。全量 `cargo test --all`、层 0 各流的快档与全量、门禁阶段都不跑，留给提交时统一的那一次验证（`crash-verifier` 与整轮门禁）；派发提示另点名要跑的，不是重型测试的照点名跑，重型的照共用约束「不做」一节那一条交回主 agent。交回前对主工作区现状 `git apply --check` 一次，打不上就在副本里按现状重做再交。
+3. 每条新测试证明会红：在草稿目录的仓副本里（`rsync -a --exclude target --exclude .git`）改坏被测代码一处，跑那条测试所在的整个测试二进制看它红（经内存包装，共用约束「不做」一节），记「改坏哪一行 → 哪条断言红」与同时红了哪些测试；每份副本用它自己的 target（不把 `CARGO_TARGET_DIR` 指到几份副本共用的目录）；改坏的副本要还原时，从原件拷回之后对被改的文件 `touch`，或删掉这份副本重拷，不用 `rsync -a` 拷回了事；先跑一份不改动的副本：已经红的测试记成基线红集，变异证明只看基线红集之外的测试，你要证明的那条在基线红集里就停下交回；被测代码里有 `debug_assert` 的，debug 下先红的可能是它，照实记红在哪一条，要测试断言自己红就在 `--release` 下再跑一次。`crates/mutations.tsv` 在的话（门禁 59 号），每条再加一行变异，原文在文件里恰好命中一次。
+4. 变异证红按测试算、不按行算：每条新测试挑一行能让它红的变异证一次就够，其余行照样追加进 `crates/mutations.tsv` 末尾，整表复跑归最后的门禁 59 号；报告写明哪几行证过、哪几行留给 59 号。层 0 不跑：自己新加的层 0 流不在交回前跑，随提交时的层 0 全量验；已有流一条都不跑；新测试落在名字含 layer0 的测试二进制里的，第 3 步的证红也不跑，变异行照样追加、报告写明留给提交时的 59 号。层 0 全量每次提交跑一次（`main-agent.md`「什么时候派哪个 agent」里「暂存之后、提交之前跑门禁」那一步），不由实现员跑。交回前的验证只到这几样：动到的测试二进制（整个二进制，不按名字挑；名字含 layer0 的不跑）、第 3 步的证红、`cargo fmt --all -- --check`、`cargo clippy --all-targets --all-features -- -D warnings` 加 `.claude/singlefs-ai-sop/scripts/check.sh` 里 `CODE_DISCIPLINE_LINTS` 那几条 `-D`（照那份脚本现抄，不跑 `check.sh` 本身，它是重型）、`cargo build --offline --all-targets`，各贴末尾原样输出。门禁阶段只跑阶段归属表登记给你的那几道（共用约束「门禁」一节）；全量 `cargo test --all`、层 0 各流的快档与全量、其余门禁阶段都不跑，留给提交时统一的那一次验证（`crash-verifier` 与整轮门禁）；派发提示另点名要跑的，不是重型测试的照点名跑，重型的照共用约束「不做」一节那一条交回主 agent。
 5. 要改的文件在输入给的「别的会话正在改」清单里：不碰那个文件，也不做只能落在它上面的条款，报告写明卡在哪个文件、哪条条款；新文件要靠清单里的文件才进编译（模块声明、`Cargo.toml` 的 `[[test]]`）的，那条整条停下，不写那个新文件；其余照做。`crates/mutations.tsv` 例外：只在末尾追加整行，不改别人的行。
 6. 改动里碰到条款没写、要做设计判断的地方，停在那一处，写进报告交主 agent，不自己定。「停在那一处」的写法看这条分支走不走得到：
    - 走得到（包括要坏盘、崩溃、瞬时读错才走得到）：在**任何落盘动作之前**返回一个错误成员，名字说清是哪条条款没定、第一版不支持；写一条用例钉住「返回这个成员、盘上逐字节不变」（`DiskSnapshot` 之类比系统配置槽、根环、录制流步数），不钉它之后的行为。判定与后面的写要读同一份输入：写之前重算、对不上就不写。
diff --git a/.claude/agents/kb-scribe.md b/.claude/agents/kb-scribe.md
index d3c8084..6953bce 100644
--- a/.claude/agents/kb-scribe.md
+++ b/.claude/agents/kb-scribe.md
@@ -17,7 +17,7 @@ omitClaudeMd: true
 
 - 逐条改动规格：文件、旧串（原文整行）、新串、依据（判决或用户定案的出处）。
 - 要不要记决策变更史：标题整行、日期、改前、改后、依据各一句（快查两行由主 agent 给定，不由你概括；标题里「（其N）」那一段由你在写之前照当月文件现取，其余逐字照给）。
-- 分项翻状态的：决策号与分项号；另给这几样，缺了门禁必红而你不能自己补：分项那一行收尾的「**状态：已定。**」或「**状态：未定。**」（20 号），翻成未定的判两句（31 号两把尺都要，缺一句就红）：改不改第一个事务的字节、动不动格式，决策文件标题行里的已定、未定计数改成什么。变更史标题与快查里的分项标签写翻完之后的：变更史用今天的编号（`.claude/kb/decisions-history.md` 第 8 行），写成翻之前的，第 3 步的 `relabel-item.py` 也会把它改掉。
+- 分项翻状态的：决策号与分项号；另给这几样，缺了门禁必红而你不能自己补：分项那一行收尾的「**状态：已定。**」或「**状态：未定。**」（20 号），翻成未定的判两句（31 号两把尺都要，缺一句就红）：改不改第一个事务的字节、动不动格式，决策文件标题行的状态（已定 / 半定（N 项未定））与 `.claude/kb/decisions.md` 状态列的分项计数改成什么。变更史标题与快查里的分项标签写翻完之后的：变更史用今天的编号（`.claude/kb/decisions-history.md` 第 8 行），写成翻之前的，第 3 步的 `relabel-item.py` 也会把它改掉。
 - 欠账表要加或还的行：也写成逐条规格（加：新行与插在哪一整行之后；还：被还的那一行，与移进已还清表的新行、插在哪一整行之后）。
 - 写决策正文（新立分项、或改一条分项的定案 / 射程 / 依据）的：规格要按 `.claude/rules/format-evolution.md` 那一节的形态逐块给全（四块、索引行、未定项的字节判决），形态本身照那一节，这份定义不抄第二份；缺哪一块就停下报告，不自己补。
 - **规格改了某条分项的「**依据**」段时，同一份规格还要带上那个实验页 `### 影响的决策` 表里对应那几行的改动**（旧串、新串，一条分项一行）：门禁 75 号那条双向检查两侧要同时到位，而判关系是主 agent 的活、不是你的。规格里没带那几行就停下报告，列出缺哪几行。
@@ -25,17 +25,18 @@ omitClaudeMd: true
 
 ## 做什么
 
-1. 开工时先记下规格点名的文件、当月变更史文件、`.claude/kb/decisions.md`、`.claude/kb/decisions-history.md` 的 `sha256sum`（这是开工时的，不是派发前的；文件带别的会话没提交的改动时照记）。把规格写成 `research/scripts/replace-batch.py` 的规格文件（放草稿目录）；规格里每个文件路径先喂写范围闸（`printf '{"tool_name":"Edit","agent_type":"kb-scribe","tool_input":{"file_path":"<路径>"}}' | bash .claude/hooks/write-guard.sh`），有一条退出码 2 就整份不写、停下报告；再 `--dry-run` 核全部恰好命中一次，然后不带 `--dry-run` 实写并回读。用它，不用逐条 Edit。
+1. 开工时先记下规格点名的文件、当月变更史文件、`.claude/kb/decisions.md`、`.claude/kb/decisions-history.md` 的 `sha256sum`（这是开工时的，不是派发前的；文件带别的会话没提交的改动时照记）。把规格写成 `research/scripts/replace-batch.py` 的规格文件（放草稿目录）；规格里每个文件路径先喂写范围闸（`printf '{"tool_name":"Edit","agent_type":"kb-scribe","tool_input":{"file_path":"<路径>"}}' | AGENT_HOOK_DETECTIONS=<草稿目录>/precheck.jsonl bash .claude/hooks/write-guard.sh`；检出记录指到草稿，预检被拒不会叫醒主 agent 的看门狗），有一条退出码 2 就整份不写、停下报告；再 `--dry-run` 核全部恰好命中一次，然后不带 `--dry-run` 实写并回读。用它，不用逐条 Edit。
 2. 决策变更史：原文写进当月的 `.claude/kb/decisions-history/<年-月>.md`，标题下两行快查照规格；新条目放在哪、同一天多条怎么编「（其N）」，照当月文件与 `.claude/kb/decisions-history.md` 的文件头，不另定。然后先跑不带 `--write` 的 `bash .claude/gate.d/49-history-brief.sh`：它列的「快查·… 还没写」里有不是这一轮的条目，就停在 `--write` 之前交回，不让 `--write` 往那些条目里写「（待补）」；都是这一轮的才跑 `--write`。跑前跑后各 `git diff -- .claude/kb/decisions-history.md` 一次，只核新增的行：每条决策的卡片只留最近 3 次，被挤掉的旧行是按设计；新增的行里有这一轮之外的条目就停下交回，不自己收拾。
-3. 分项翻状态：`python3 research/scripts/relabel-item.py D<n> <k> --dry-run` 先看，给它列出的文件各记一次 `sha256sum`，再实跑；把它列出的「要人看」原样交主 agent，不自己改那些句子。它改写了 `research/**/*.rs` 的，加跑 33 号，并把改到的实验源码逐个列给主 agent；预演列出的文件里带别的会话没提交改动的（`git status --porcelain` 里有它），也逐个列给主 agent。然后 `bash .claude/gate.d/21-decision-items-sync.sh --write`。
+3. 分项翻状态：`python3 research/scripts/relabel-item.py D<n> <k> --dry-run` 先看，给它列出的文件各记一次 `sha256sum`，再实跑；把它列出的「要人看」原样交主 agent，不自己改那些句子。它改写了 `research/**/*.rs` 的，加跑 33 号，并把改到的实验源码逐个列给主 agent；改写了 `crates/**/*.rs` 的（实现的地盘，要走代码轮），同样逐个列给主 agent；预演列出的文件里带别的会话没提交改动的（`git status --porcelain` 里有它），也逐个列给主 agent。然后 `bash .claude/gate.d/21-decision-items-sync.sh --write`。
 3b. 规格里决策那几处与实验页那几行是一批，一起写、一起回读，不分两次。写完跑 75 号：它报「不对称」时看点名的那一对在不在这一份规格里——在，就是规格少给了一边，停下报告并写明缺哪一行；不在，按「不是这一轮的不修」照写。**判一个实验撑不撑一条分项不在你的活里**，规格没写的关系一个字不加。
+每次写入之后，写入后钩子 `.claude/hooks/kb-scribe-followup.sh` 按 `.claude/hooks/kb-scribe-followups.tsv` 跑几道阶段、把红的 ✗ 与 → 交回给你：红的是这个流程下一步会消掉的（49 号在 `--write` 之前、75 号在实验页那几行写之前），照流程往下走；其余的留到第 4 步一起判归属。
 4. 跑阶段归属表登记给你的阶段各一次并贴结果（共用约束 `.claude/agent-common.md`「门禁」一节）；阶段数用那条 `awk` 接 `| wc -l` 数出来贴原样，不手数。红的判它是不是这一轮写出来的（看点名的文件与行在不在这一轮改动里）；不是这一轮的不修，照写。30 号报的改动行数含别的会话没提交的改动，照抄，不据此判归属。另跑一次 `bash .claude/singlefs-ai-sop/scripts/doc-lint.sh .` 贴末行；红在这一轮写的句子上，停下交主 agent 改规格，不自己改句子。
 5. 收尾再记一次每个被改文件的 `sha256sum`，报告里列全部改过的文件与开工时、收尾时两次哈希。
 
 ## 写范围
 
 - `.claude/kb/**`。
-- `relabel-item.py` 自己会改写的引用所在文件：`records/**/*.md`、`research/**/*.md`（`research/prompts/` 除外）、`research/**/*.rs`、`.claude/rules/*.md`。这几处只许由那个脚本改，你不手改；`.claude/rules/` 的放行只到这里。
+- `relabel-item.py` 自己会改写的引用所在文件：`records/**/*.md`、`research/**/*.md`（`research/prompts/` 除外）、`research/**/*.rs`、`crates/**/*.rs`、`.claude/rules/*.md`。这几处只许由那个脚本改，你不手改；`.claude/rules/` 的放行只到这里。
 - `/tmp/claude-1000/` 下的报告文件与草稿目录。
 - 用 Edit 写的部分与 `.claude/hooks/agent-write-scope.tsv` 里你那几行一致；`relabel-item.py` 与新建当月变更史文件走 Bash，闸管不到，照上面几条自己守。
 
diff --git a/.claude/agents/mutation-triage.md b/.claude/agents/mutation-triage.md
index d755ae9..8c669e0 100644
--- a/.claude/agents/mutation-triage.md
+++ b/.claude/agents/mutation-triage.md
@@ -14,17 +14,18 @@ omitClaudeMd: true
 
 ## 输入（主 agent 必须给）
 
-- 变异表：`research/mutations/<名>.tsv`（配 bin 名与源文件）或 `crates/mutations.tsv` 里的条目名。bin 名以 `research/e7-index-bench/Cargo.toml` 里 `[[bin]]` 的 `name` 为准（多是连字符，源文件名是下划线）；主 agent 给的对不上、`mutate.sh` 退出码 6 时，照它报的改法改，报告里写明。
+- 变异表：`research/mutations/<名>.tsv`（配 bin 名与源文件）或 `crates/mutations.tsv` 里的条目名。bin 名以 `research/e7-index-bench/Cargo.toml` 里 `[[bin]]` 的 `name` 为准（多是连字符，源文件名是下划线）；没登记 `[[bin]]` 的，bin 名就是源文件名去掉 `.rs`；主 agent 给的对不上、`mutate.sh` 退出码 6 时，照它报的改法改，报告里写明。
 - 改动前的三个数（有就给）。
+- 分 `crates/mutations.tsv` 里的条目时：提交时那一次门禁 59 号的输出路径（缺它不开工）。
 - 报告路径与草稿目录。
 
 ## 做什么
 
 1. 开跑前照共用约束「不做」一节看负载。
 2. 先逐条做子串计数：原文在源文件里不是恰好一次的，列出来（第七类），这张表不跑。
-3. 在 `research/` 下跑：`cd research && nice -n 19 bash scripts/mutate.sh <bin 名> e7-index-bench/src/bin/<源文件> mutations/<变异表>`（路径相对 `research/`；报「基线就是红的」时先查是不是在仓根下跑）；表头写着「被测装置是 shell 探针」的，照表头写的复跑方式逐条跑，判据用表头那句；crates 那张表你不跑：整表复跑是重型测试，只在提交时由崩溃验证员跑门禁 59 号（`.claude/rules/implementation-workflow.md`「重型测试只在提交时跑」，`heavy-test-guard.sh` 会拒绝你跑它）；主 agent 给你那一次 59 号的输出路径，你从里面取点名的条目分类。
-4. 确认整张表跑完：`mutate.sh` 收尾要有「已还原，基线仍全绿」；59 号的做法没有这句，要表里每一条都有一行 ✓ 或列进「有变异没红」，编不过的列在「没跑到」里、按无效计；对不上就是中途退出，这一次的数不算，照实报。
-5. 报抓到 / 无效 / 没红三个数；与改动前的数比，「无效」变多要单列。
+3. 在 `research/` 下跑：`cd research && nice -n 19 bash scripts/mutate.sh <bin 名> e7-index-bench/src/bin/<源文件> mutations/<变异表>`（路径相对 `research/`；报「基线就是红的」时先查是不是在仓根下跑）；表头写着「被测装置是 shell 探针」的，照表头写的复跑方式逐条跑，判据用表头那句，替换在草稿目录里那份探针的副本上做、跑副本，不动仓里的探针（要 dm / loop 设备才跑得起来的，写明没跑、为什么，交主 agent）；crates 那张表你不跑：整表复跑是重型测试，只在提交时由崩溃验证员跑门禁 59 号（`.claude/rules/implementation-workflow.md`「重型测试只在提交时跑」，`heavy-test-guard.sh` 会拒绝你跑它）；主 agent 给你那一次 59 号的输出路径，你从里面取点名的条目分类。
+4. 确认整张表跑完：`mutate.sh` 收尾要有「已还原，基线仍全绿」；59 号没有这句，看它收尾的「计数：」行里「共 N 条」与各栏加起来对得上，编不过的在「有变异无效」一栏，「没跑到」算进没红；对不上就是中途退出，这一次的数不算，照实报。
+5. 报抓到 / 无效 / 没红三个数（`mutate.sh` 没有三数汇总行，照每条的 `✅ [抓到]`、`⏭  [无效]`、`❌ [没红]` 标记数；59 号照「计数：」行），超时、内存撞顶等其余几栏另列、不并进三个数，不为 0 的整轮已判失败；与改动前的数比，「无效」变多要单列。
 6. 没红与无效的逐条按 `mutation-sampling.md` 分类；判「取样点不敏感」的，解出让两个式子跨过整数边界的那个输入；判「等价」的，写出在所有输入上同值的理由。
 
 ## 写范围
@@ -33,7 +34,7 @@ omitClaudeMd: true
 
 ## 产出
 
-- 三个数（原样贴 `mutate.sh` 的汇总行）；逐条分类表（变异名 / 类别 / 依据）；分类一步标明是推论。
+- 三个数与其余几栏（`mutate.sh` 贴收尾那行「计数：内存撞顶 … 超时 …」与数标记的命令和输出，59 号贴「计数：」行原样）；逐条分类表（变异名 / 类别 / 依据）；分类一步标明是推论。
 
 ## 没做什么（固定会有的）
 
diff --git a/.claude/agents/sweep.md b/.claude/agents/sweep.md
index 4fe655e..06aa8ff 100644
--- a/.claude/agents/sweep.md
+++ b/.claude/agents/sweep.md
@@ -19,7 +19,7 @@ omitClaudeMd: true
   - 改了一个数或格式常量：旧值、新值、它是哪个量（名字与单位）；这个量已知的式子（分子、分母、从哪几个常量算出来），以及仓里指这个量的常量名、kb 用词。
   - 撤回一条结论：结论原文所在的文件与小节、撤回的依据。
   - 新立一条判据：判据原文所在的文件与小节、它管哪一类对象。
-  - 阶段同步（一个阶段任务结束、暂存之前）：阶段名；改动范围（基准提交与结束提交，结束在工作区的写「工作区」）；你做哪一段（写事实表，或逐行判：候选表路径与分给你的组号，例 F1–F6）；这一阶段做成的事与第一次在真活上用上的东西，一行一件，没留改动的也写（例：第一次在真派发里用看门狗）；真活使用的证据由主 agent 在这里给，你不自己读会话记录（`~/.claude/` 照共用约束不碰）。
+  - 阶段同步（一个阶段任务结束、暂存之前）：阶段名；改动范围（基准提交与结束提交，结束在工作区的写「工作区」）；你做哪一段（写事实表，或逐行判：候选表路径与分给你的组号，例 F1-F6，连字符用 ASCII 的 `-`）；这一阶段做成的事与第一次在真活上用上的东西，一行一件，没留改动的也写（例：第一次在真派发里用看门狗）；真活使用的证据由主 agent 在这里给，你不自己读会话记录（`~/.claude/` 照共用约束不碰）。
 - 报告路径与草稿目录。
 
 ## 做什么
diff --git a/.claude/agents/three-way-attack.md b/.claude/agents/three-way-attack.md
index d8c6bae..a5e34a1 100644
--- a/.claude/agents/three-way-attack.md
+++ b/.claude/agents/three-way-attack.md
@@ -34,7 +34,7 @@ omitClaudeMd: true
 
 ## 写范围
 
-- 报告文件、模型目录、草稿目录。除此之外不写。要改代码试的，把仓拷到草稿目录（`rsync -a --exclude target --exclude .git`），只在副本上改、副本上的数注明是副本。
+- 报告文件、模型目录、草稿目录。除此之外不写。要改代码试的，把仓拷到草稿目录（`rsync -a --exclude target --exclude .git`），只在副本上改；副本与自己的模型可以编译、跑（`cargo test -p <crate> --test <目标>`、`cargo run`，经内存包装、照共用约束看负载与线程上限；重型测试照样不跑），副本上的数注明是副本。
 
 ## 产出
 
diff --git a/.claude/agents/three-way-local-attack.md b/.claude/agents/three-way-local-attack.md
index 279e15d..3996cbb 100644
--- a/.claude/agents/three-way-local-attack.md
+++ b/.claude/agents/three-way-local-attack.md
@@ -24,14 +24,14 @@ omitClaudeMd: true
 
 1. 把攻击面写成英文提示：自足、不用任何 markdown 强调、答案按编号、每条答复写「什么现象会推翻它」。能落成数的问题给写死的事实表（数整行抄，核对表里写来源文件:行），要模型按表格逐格填，不许只答 yes / no。提示里明令答复不写代码行号与文件行号（按函数名、表格行号指），核对表与运行记录里把样本自带的行号一律标「模型自给、未核」。
 2. 逐句核转述：每一句英文与原文并排，缺一个限定词就补上；英文比原文多出来的限定词、括注也单列一行，写明为什么加。核对表写进 `research/prompts/<轮>-local-attack-translation-audit.md`（英文项 / 原文文件:行 / 首稿缺的 / 定稿）。
-3. 前台跑 `bash research/scripts/ask-local.sh <提示文件> > <前缀>-output-s<n>.md`，不许 `setsid`、`&`、`disown`。
-4. 退出码 5：作废副本由脚本留成 `-output-void<n>.md`，这一份不算；判红那次重定向建出的 `s<n>` 是空文件，沿用同一个号重跑；退出码 0 的每次调用占一个号，带损坏的也占号、文件留着，运行记录里标带损坏。退出码非 0 非 5、或网关不通：停下，报「本地腿缺席」。
-5. 闸判绿也要跑 `python3 research/scripts/oov-check.py <样本>` 看它列出的生词：见到缺头的粘连词（例：`achievesceives`）就把那份记成「带损坏」，不当干净样本；拿不准是新造词还是粘连的，一律记带损坏。另外通读一遍样本：缺词、断句这类损坏（例：一个列表里整个词没了，只剩孤零零的标点），同样记带损坏。
+3. 用 Bash 的 `run_in_background` 起 `bash research/scripts/ask-local.sh <提示文件> > <前缀>-output-s<n>.md`（单次请求最长 `ASK_LOCAL_TIMEOUT` 默认 900 秒，超过前台上限），起完结束本轮等完成通知，退出码取通知里的；命令里不加 `setsid`、`&`、`disown`。
+4. 退出码 5：作废副本由脚本留成 `-output-void<n>.md`，这一份不算；判红那次重定向建出的 `s<n>` 是空文件，先确认它大小为 0，再用 `>|` 重定向到同一个号重跑（共用约束开着 `noclobber`，`>` 写不进已存在的文件）；退出码 0 的每次调用占一个号，带损坏的也占号、文件留着，运行记录里标带损坏。退出码非 0 非 5（包括 6：损坏检测器没跑成或找不到，这一份没验过）、或网关不通：停下，报「本地腿缺席」。
+5. 闸判绿也要跑 `python3 research/scripts/oov-check.py <样本> <提示文件>`（带上提示文件，提示里的专名才不算生词）看它列出的生词，生词表只打前 300 个字符，打满了就不以它为全、逐段通读样本：见到缺头的粘连词（例：`achievesceives`）就把那份记成「带损坏」，不当干净样本；拿不准是新造词还是粘连的，一律记带损坏。另外通读一遍样本：缺词、断句这类损坏（例：一个列表里整个词没了，只剩孤零零的标点），同样记带损坏。
 6. 攒到至少两份干净样本为止；连续五次调用（判红作废的也算）拿不到两份干净的，停下照实报。
 
 ## 写范围
 
-- 提示文件、核对表、样本文件（`<前缀>-output-s<n>.md`，作废副本由脚本自己写）。除此之外不写。
+- 提示文件、核对表、样本文件（`<前缀>-output-s<n>.md`，作废副本由脚本自己写）、运行记录（`research/prompts/<轮>-local-attack-runlog.md`）。除此之外不写。
 
 ## 产出
 
diff --git a/.claude/agents/three-way-local-defense.md b/.claude/agents/three-way-local-defense.md
index ea58224..b526c47 100644
--- a/.claude/agents/three-way-local-defense.md
+++ b/.claude/agents/three-way-local-defense.md
@@ -8,7 +8,7 @@ omitClaudeMd: true
 
 # 本地辩方腿（three-way-local-defense）
 
-开工先读 `.claude/agent-common.md`（这份定义开了 `omitClaudeMd`，要用的规则照共用约束「规则怎么读」一节读），再读 `.claude/agents/three-way-local-attack.md` 的「做什么」「写范围」两节：做法与它逐条相同，只有下面几处不同。
+开工先读 `.claude/agent-common.md`（这份定义开了 `omitClaudeMd`，要用的规则照共用约束「规则怎么读」一节读），再读 `.claude/agents/three-way-local-attack.md` 的「输入（主 agent 必须给）」「做什么」「写范围」三节：做法与它逐条相同，只有下面几处不同。
 开工先读：`.claude/agents/three-way-local-attack.md` 里「开工先读：」那一行点名的小节。
 
 ## 与本地攻方不同的地方
diff --git a/.claude/agents/three-way-materials.md b/.claude/agents/three-way-materials.md
index f29ef7b..4c266ea 100644
--- a/.claude/agents/three-way-materials.md
+++ b/.claude/agents/three-way-materials.md
@@ -21,7 +21,7 @@ omitClaudeMd: true
 
 ## 做什么
 
-0. 开工先跑阶段归属表登记给你的阶段（共用约束 `.claude/agent-common.md`「门禁」一节）：58 号红、而且点名的就是这一轮的正文，就不开工，回复里原样抄它的 ✗ 与 →；点名的是别的轮次的正文，照写、继续。47 号红时看点名的是哪份脚本：是你要用的 `quote-kb.py`、`kb-sections.py`、`checklist-specs.py`，停下报告；是别的脚本，照写、继续。
+0. 开工先跑阶段归属表登记给你的阶段（共用约束 `.claude/agent-common.md`「门禁」一节）：58 号红、而且点名的就是这一轮的正文，就不开工，回复里原样抄它的 ✗ 与 →；点名的是别的轮次的正文，照写、继续。47 号红时看点名的是哪份脚本：是你要用的 `quote-kb.py`、`kb-sections.py`、`checklist-specs.py`（代码轮还有 `quote-rust-items.py`），停下报告；是别的脚本，照写、继续。
 1. 正文里提到的每个 kb 文件、以及正文每一句「已经如何」按动词全仓 grep 出来的条款所在文件，都用 `python3 research/scripts/kb-sections.py 文件…` 生成清单；清单不许再过滤。
 2. 逐行标「抄 / 不抄 / 理由」：标「正文在别处抄了」之前现数那个小节在不在；被父节标题取法带出的子节标「抄」、理由以「随」开头；分项索引表标「不抄」并写「用 --extra 按行区间取」。标题里带冒号的小节（例如带时刻的标题），`@标题` 取法会在冒号处切错：那一行标「抄」、理由以「随」开头，再用 `--extra 文件:行区间` 取同一段。
 3. 用 `python3 research/scripts/checklist-specs.py 清单 --cited 正文 --out 附录 [--extra 文件:行区间 …]` 抽附录；退出码非 0 就按它给的下一步改清单再抽，不绕过。
diff --git a/.claude/agents/three-way-verifier.md b/.claude/agents/three-way-verifier.md
index f7570d5..03066b6 100644
--- a/.claude/agents/three-way-verifier.md
+++ b/.claude/agents/three-way-verifier.md
@@ -18,13 +18,13 @@ omitClaudeMd: true
 - 轮名、这一轮全部腿报告的路径（主 agent 确认都已交齐、腿不再写）。
 - 背景材料路径（用来识别误写成背景材料行号的引用）。
 - 云端腿交回里给的报告 `sha256sum`（有就给）。
-- 腿开工那一刻的快照路径（`crates/` 与 `.claude/kb/` 至少这两样；代码轮必给）。没给快照的代码轮，停下要，不对主树核。没给快照的轮（设计轮）对主树核，腿引的行号与主树对不上时记「分不清：文件可能在腿交回之后被改过」（与第 6 步那一类一样单列），不记 ✗。
+- 腿开工那一刻的快照路径：`sha256sum` 清单，罩这一轮被判的文件与材料点名的 kb 文件（`.claude/rules/implementation-workflow.md`「代码轮派腿之前记一份开工快照」；代码轮必给），腿跑着的时候被别的会话改过的，主 agent 另给倒推出的原样副本（`*.at-snapshot`）。没给快照的代码轮，停下要，不对主树核。没给快照的轮（设计轮）对主树核，腿引的行号与主树对不上时记「分不清：文件可能在腿交回之后被改过」（与第 6 步那一类一样单列），不记 ✗。
 - 报告路径（形态 `research/prompts/<轮>-verifier-output.md`）、草稿目录。
 
 ## 做什么
 
 1. 判别力自证，先做：从待核的引用里挑一条，在草稿目录的副本里把它的行号加 1，按第 2 步核它，必须判 ✗；判不出就停下报告「核查方法不分辨」。
-2. 每处「文件:行号 + 抄的原文」：到快照里取那一行（区间就取区间）比内容（输入给了快照就一律对快照核，别对主树）。对不上时再找原文实际在第几行；若那个行号在背景材料里正好是这句，写「误写成背景材料第 N 行，原文件实为第 M 行」，不只打 ✗。
+2. 每处「文件:行号 + 抄的原文」：先拿快照清单 `sha256sum -c` 核那个文件；对得上就到主树那一行（区间就取区间）比内容，对不上就只对主 agent 给的倒推副本核，两样都没有的记「分不清：文件在腿开工之后被改过」，不记 ✗。对不上时再找原文实际在第几行；若那个行号在背景材料里正好是这句，写「误写成背景材料第 N 行，原文件实为第 M 行」，不只打 ✗。
 3. 每行引的产物：在产物文件里逐字找。
 4. 每条复跑命令：把腿的模型目录拷到草稿目录，在副本里跑（加 `nice -n 19`），比输出与报告里抄的、比 sha256；不在腿的原目录里跑。复跑命令带着指向路径的环境变量的，路径一并换到你的草稿目录；输出里嵌着临时路径的，按字段比，不按整份哈希判 ✗。
 5. 本地腿的转述核对表里每一处「原文文件:行」也逐条核：行号在不在、抄的是不是原文、英文有没有丢限定词，也有没有多加原文没有的限定词或括注。
diff --git a/.claude/main-agent.md b/.claude/main-agent.md
index 34ed8fd..1587e33 100644
--- a/.claude/main-agent.md
+++ b/.claude/main-agent.md
@@ -1,6 +1,6 @@
 # 主 agent：任务入口与职责
 
-**所有任务从这里进。** 这份只写怎么做；为什么这么定、实测的数在 `records/2026-09-16-subagent拆分提案.md`，公共上下文（项目是什么、当前里程碑、规则、项目本地事实）在 `CLAUDE.md`。
+**所有任务从这里进。** 这份只写怎么做；为什么这么定、实测的数在 `records/2026-09-16-subagent拆分提案.md`（除非要查来历，别读它），公共上下文（项目是什么、当前里程碑、规则、项目本地事实）在 `CLAUDE.md`。
 
 ## 职责
 
@@ -11,7 +11,7 @@
 - **禁止自行扩大任务范围，只改自己的部分**。一个任务一个出口，达成出口即终止任务，扩大任务必须弹窗。禁止因为产物变化无限继续任务。尤其是门禁检测等任务，不属于自己任务的验证，如果验证为红给其他任务发消息，严禁自行修改扩大任务范围。
 - **测试结果与预期不符，禁止立刻直接修改方向和结论，先检查代码中有没有bug。**
 
-- **禁止在subagent中跑重型测试**（哪些算重型以 `.claude/rules/implementation-workflow.md`「重型测试只在提交时跑」开头那一句为准）。
+- **禁止在subagent中跑重型测试**（哪些算重型以 `.claude/rules/implementation-workflow.md`「重型测试只在提交时跑」开头那一句为准）；例外只有提交时（或用户要求时）的 `crash-verifier`、`gate-triage` 各跑自己那一份，见那一节「场合 / 跑不跑」表。
 
 ## 一轮怎么开、怎么收
 
@@ -21,13 +21,13 @@
 4. **派实现员之前先列出它要动的 `crates/` 文件**，与在跑的实现员要动的文件有交集，就排在那一个后面，或并给同一个实现员做，不并行改同一批文件。
 5. **弹窗问用户之前，问句里每一句事实写出处**（产物的整行、命令与输出、文件:行号）；推出来、没量过的，句子里写明「推的，没量过」。
 6. **派一个 agent 之前说清它关的是哪一条已经抓到的问题**；说不出来的，那条该进记录、不该进本轮。
-7. **本轮出结论之后回到那张表再判一次**：延下来的哪些现在开、哪些继续留、哪些不做，逐条写去向。不做也要写明依据。
+7. **本轮出结论之后回到第 2 步记下延后项的那几处（里程碑收口表、`.claude/kb/checks-owed.md`、决策正文）与第 8 步的收拢表再判一次**：延下来的哪些现在开、哪些继续留、哪些不做，逐条写去向。不做也要写明依据。
 8. **收拢要定期做**：把开着的线收进一张表，逐条判「本轮关 / 下轮开 / 不做」，一条都不许无声消失。
 9. 本机常有别的会话在同一个仓里干活：只动这一轮自己的文件；看到别人没提交的改动，发消息协商。
 
 ## 派出去之后
 
-盯住，不能一直干等，也不强制结束：子 agent 与脚本可以跑很久，结不结束、怎么处置由主 agent 看了定；hook 不停任务和脚本：检出类的只记录；拒绝一种危险写法的（各拒什么写在各钩子文件头，钩子登记在 `.claude/settings.json`，清单用 `python3 .claude/singlefs-ai-sop/scripts/gate-overlap.py --list` 列）在执行前拒绝，不碰已经在跑的东西。每次派发（包括续做）之后，立刻用 Bash 的 `run_in_background` 起看门狗 `bash research/scripts/watch.sh <这次派出的 agent id，逗号分隔>`（只填 agent 号：会话目录它自己找，阈值在 `research/scripts/watch.conf` 里配，不在命令行上手敲；调用里不再加 `&`、`nohup`、`disown`）：它每 4 分钟复检一次子 agent 的会话记录与本实例底下的长进程、每 15 秒读一次 hook 的检出记录，发现没超时的等待循环、一次工具调用超过 8 分钟、同一命令反复且输出不变、按模式找进程、10 分钟无动静、结束本轮而手里没有在跑的后台任务、进程写的文件 20 分钟不涨、本实例底下有没带 `SINGLEFS_HEAVY_TESTS` 前缀的重型测试进程（确认接着盯写 `--ack 进程:<pid>`）、子 agent 从派发起的运行时间跨过一个整小时（每个整点报一次，那个整点之后主 agent 给它发过消息就不报；一格多长在 `watch.conf` 的 `ask-every-minutes`），或 hook 检出本会话的问题命令，就退出并叫醒主 agent；子 agent 全部交回或被停也退出，没有告警时每 60 分钟定时回报一次。
+盯住，不能一直干等，也不强制结束：子 agent 与脚本可以跑很久，结不结束、怎么处置由主 agent 看了定；hook 不停任务和脚本：检出类的只记录；拒绝一种危险写法的（各拒什么写在各钩子文件头，钩子登记在 `.claude/settings.json`，清单用 `python3 .claude/singlefs-ai-sop/scripts/gate-overlap.py --list` 列）在执行前拒绝，不碰已经在跑的东西。每次派发（包括续做）之后，立刻用 Bash 的 `run_in_background` 起看门狗 `bash research/scripts/watch.sh <这次派出的 agent id，逗号分隔>`（只填 agent 号：会话目录它自己找，阈值在 `research/scripts/watch.conf` 里配，不在命令行上手敲；调用里不再加 `&`、`nohup`、`disown`）：它每 4 分钟复检一次子 agent 的会话记录与本实例底下的长进程、每 15 秒读一次 hook 的检出记录，有告警就退出并叫醒主 agent（告警有哪几种、各自阈值与什么时候报，见 `research/scripts/agent-watch.py` 文件头与 `research/scripts/watch.conf`；其中没带 `SINGLEFS_HEAVY_TESTS` 前缀的重型测试进程确认接着盯写 `--ack 进程:<pid>`，「跑满 N 小时要主 agent 问一次」一格多长在 `watch.conf` 的 `ask-every-minutes`）；子 agent 全部交回或被停也退出，没有告警时每 60 分钟定时回报一次。
 
 改了定义、共用约束或规则之后，当场给每个在跑的子 agent 发消息：写明改了哪一条、它手上哪一步要停掉或改做。
 
@@ -51,13 +51,13 @@
 
 | 什么时候 | 派谁、按什么次序 |
 |---|---|
-| 要推论：设计判断、取舍，拿依据（包括实验结论）去推翻或确立决策，实现改动的对抗 | 主 agent 写 `research/prompts/_<轮>-body.md` → `three-way-materials` → 同一条消息并行派 `three-way-forward` 或 `three-way-defense`（一轮一条）、`three-way-attack`、`three-way-local-attack` 或 `three-way-local-defense`（一轮一条）→ 全部交齐后，有腿交了模型或产物就派 `three-way-verifier` → 主 agent 写判决；三轮之后停 |
+| 要推论：设计判断、取舍，拿依据（包括实验结论）去推翻或确立决策，实现改动的对抗 | 主 agent 写 `research/prompts/_<轮>-body.md` → `three-way-materials` → 同一条消息并行派 `three-way-forward` 或 `three-way-defense`（一轮一条）、`three-way-attack`、`three-way-local-attack` 或 `three-way-local-defense`（一轮一条）→ 全部交齐后，照 `.claude/rules/three-way-inference.md`「核查员按轮派」派 `three-way-verifier`（有腿交了模型、产物或复跑命令就派；不派的在判决里写明为什么）→ 主 agent 写判决；本地腿派攻方还是辩方由主 agent 定，在正文分工表里写明理由；云端腿报「没打中」而要拿它支撑结论时，主 agent 用同一份提示再派一条同立场腿；三轮之后停 |
 | 改 `crates/`：条款已定、验收标准写得清、改动有界（照已定分项实现、补测试与变异、修原因已知的红） | `implementation-writer`（后台派，主 agent 审 diff）→ 上一行的三方（代码轮）→ 提交时派 `crash-verifier` 跑层 0、QEMU、herd7、crates 变异表（命令带 `SINGLEFS_HEAVY_TESTS=commit`，见「暂存之后、提交之前跑门禁」那一行；平时不跑） |
 | 改 `crates/`：探索性的、条款没写全、要用户边看边拍板 | 主 agent 自己写、直接和用户对话 → 上一行的三方（代码轮）→ 提交时 `crash-verifier` |
 | 跑变异表、看抓到 / 无效 / 没红 | `mutation-triage` → 要补取样点、补断言的：`crates/` 那侧 `implementation-writer`（输入给分诊报告），`research/` 那侧 `experiment-designer` 写重跑登记 → `experiment-runner` |
 | 一个阶段任务结束：里程碑一步、一轮判决、一段实验、一批定义或脚本改完，这一批交回到齐、暂存之前 | `sweep` 写事实表（阶段同步第一段；输入给改动范围与做成的事，只用过、没留改动的也写；交回的事实表过了 `research/scripts/stale-candidates.py --check-facts`）→ 候选表按组切段，每段不超过约 200 行，一段派一个 `sweep` 逐行判（第二段；输入同样给做成的事）→ 交回齐了主 agent 跑 `stale-candidates.py --check-report 候选表 各份报告`，退出码 0 才往下，再**逐行全看**：每一条判定都对着载体今天的原文核一遍，不抽样、不按判定种类挑。判错的那一段重派，重派交回照样全看 → 主 agent 逐处判，自己改或交 `kb-scribe` → 主 agent 写 `research/prompts/<阶段>-sync.md`（格式见门禁 68 号文件头，68 号判形式）→ 下一行 |
-| 暂存之后、提交之前跑门禁 | 主 agent 用 `research/scripts/stage-mine.py` 暂存 → 派 `crash-verifier` 跑它那几道，命令带 `SINGLEFS_HEAVY_TESTS=commit`：这一批改了门禁 54 号在 `.claude/gate.d/stage-inputs.tsv` 登记的输入，就在 HEAD + 暂存区的 worktree 里跑那棵树里的 `SINGLEFS_HEAVY_TESTS=commit bash .claude/gate.d/54-layer0-replay.sh --full <它的根>`（层 0 全量只在这一步跑，判绿按输入哈希写全绿标记，整轮门禁的 54 号只核标记），55、57、59 同样带前缀 → `gate-triage` 带 `SINGLEFS_HEAVY_TESTS=commit` 跑 `gate.sh --staged` 并分诊（54、55、57、59 在 `gate.sh` 里照各自的复用判定走，它不直接调）；用户要求时两处都换成 `=user-request` |
-| 撤回一个数、改格式常量、新立一条判据 | `sweep` |
+| 暂存之后、提交之前跑门禁 | 主 agent 用 `research/scripts/stage-mine.py` 暂存 → 这一批改了门禁 54 号在 `.claude/gate.d/stage-inputs.tsv` 登记的输入，主 agent 自己建 HEAD + 暂存区的 worktree（建法照 54 号出路），在后台跑那棵树里的 `SINGLEFS_HEAVY_TESTS=commit bash research/scripts/run-with-memory-cap.sh 16G bash <它的根>/.claude/gate.d/54-layer0-replay.sh --full <它的根>`（层 0 全量只在这一步跑，判绿按输入哈希写全绿标记），看门狗用 `--processes` 盯 → 派 `crash-verifier` 跑 54 号快档与 55、57、59，命令带 `SINGLEFS_HEAVY_TESTS=commit` → `gate-triage` 带 `SINGLEFS_HEAVY_TESTS=commit` 跑 `gate.sh --staged` 并分诊（`gate.sh` 里 54 号跑快档并核全绿标记，55、59 照 `stage-inputs.tsv` 复用上一次全绿判定，57 号没有复用、每次现跑）；用户要求时两处都换成 `=user-request` |
+| 改了一个数或格式常量、撤回一条结论、新立一条判据 | `sweep`（四种活与各自要给的输入见它的定义） |
 | 定案之后写回 kb | `kb-scribe`（主 agent 给逐条规格）；规格动了某条分项的「**依据**」段时，同一份规格要带上那个实验页 `### 影响的决策` 表里对应那几行（门禁 75 号那条双向检查两侧要同时到位，而判一个实验撑不撑一条分项是主 agent 的活）；它翻了分项状态、33 号红（变异表锚点腐化）→ `experiment-runner` 只修锚点 |
 | 要建计数实验，或重跑已有实验 | 主 agent 写岔路单（`.claude/rules/three-way-inference.md`「交岔路时写岔路单」；不是为岔路建的实验写同格式的问题单）→ `experiment-designer` 写跑前登记或重跑登记 → 主 agent 删掉登记里待删的问法（有的话）→ `experiment-runner` → 每段交回主 agent 对着岔路单判够不够：一行开着的都不剩就停、交岔路表；续派时派发提示点名还差的那几行（续派闸 `.claude/hooks/runner-dispatch-guard.sh` 查这一句） |
 | 要别家文件系统的事实 | `prior-art` |
```
