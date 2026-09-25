#!/usr/bin/env bash
# PreToolUse hook（Bash）：检出可能出问题的命令，记下来交给主 agent 判断；起看门狗的错误写法、前台没超时的等待循环、把活放出追踪的写法、run_in_background 里后面没有 wait 的单独 `&`、整份覆盖 `research/results/` 下未跟踪产物的写法、打得到别人进程的终止写法与在同一个 inode 上改已有脚本的写法在执行前拒绝，其余只记不拦，不停任何在跑的命令与脚本。
#
# 按模式找进程（`pgrep -f`、`pkill -f`、`killall`）不在这里判：上游 SOP 的 `.claude/singlefs-ai-sop/scripts/claude-hooks/pattern-process-guard.sh`
# 在执行前拒绝（`.claude/settings.json` 里与本 hook 注册在同一条 Bash matcher 下），判据与 shell-lint 的 S2、S3 同一份、只认命令位置。
# 这里原先自己判，判据是「这几个词出现在哪都算」，把写在 grep 参数里的词也拦了（用户 2026-09-19 定拦截放上游）。
#
# 为什么：2026-09-17 一个实验执行员用 `pgrep -f` 找复跑进程，接着在一个等不到的 `until` 循环里空转，
# 主 agent 查进度才发现（records/2026-09-17-已分配口径三方与两个实验.md 第六节）。用户定：hook 不能终止任务和脚本，
# 只负责检出问题，交给主 agent 去判断；子 agent 与脚本结不结束由主 agent 定。
#
# 检出两种，写进检出记录（默认 /tmp/claude-1000/agent-hook-detections.jsonl，环境变量 AGENT_HOOK_DETECTIONS 可改），每条一行 JSON：
#   ① 没有 `timeout` 的 `until` / `while` 等待循环（里面有 `sleep`）：等的条件不成立时会一直等（前台的在执行前拒绝，见「拒绝六种」②，拒了不再记）；
#   ② `run_in_background` 起的命令里又自己放后台：外层 shell 当场退出（下面列的形态里，命令位置上的 disown、coproc、setsid -f、nohup … &、
#      tmux / screen 的分离模式、systemd-run 在执行前拒绝，见「拒绝六种」③；单独的 `&` 而之后同一层没有 `wait` 的在执行前拒绝，见「拒绝六种」④；
#      拒了不再记。这里剩下的只记：起了几个 `&`、之后的 `wait "$pid"` 只等了其中一个，与 `start-stop-daemon -b`），
#      harness 的完成通知当场发出，真正在跑的东西跑完不再叫醒谁（2026-09-18 一个门禁分诊员这样起门禁与三个缓存计时器，
#      records/2026-09-16-subagent拆分提案.md 第三十三节）。认的形态：`disown`、`coproc`、`setsid -f/--fork`、同一行带单独 `&` 的 `nohup`、
#      `tmux new(-session) … -d`、`screen -dm…`、不带 --wait/--pipe/--pty/--scope 的 `systemd-run`、`start-stop-daemon -b`、
#      子 shell 或命令替换里的 `&`，以及后面没有等它的单独 `&`（不是 `&&`、`>&`、`&>`、`|&`、`<&`）——等它指：之后在命令位置有
#      不带参数的 `wait`，只起了一个 `&` 时带 pid 的 `wait` 也算（`wait -n` 不算），或之后有 `tail --pid` / `while|until … kill -0` 轮询。
#      喂给非 shell 解释器的 heredoc 正文先剥掉再找（Python 的按位与、Rust 的引用不算）。
#      判不到的：前台起的 `… &`（run_in_background 为假，hook 那一刻看不出之后会不会结束本轮去等；由看门狗那一侧的
#      「结束本轮、手里没有在跑的后台任务、也没交回」告警兜）；引号里当数据用的 `&`（sed 替换串、URL 参数）会误记，
#      不能一概跳过引号，`bash -c '… &'` 里的 `&` 是真放后台；`… & (wait)`、`bash -c '… &'; wait` 这种等不到的 `wait` 这里认不出（④ 按层、按圆括号认得出，拒了不再记）。
# 主 agent 的看门狗（research/scripts/agent-watch.py watch）每 15 秒读一次检出记录，读到本会话的检出就退出、叫醒主 agent。
# 两种检出都先剥掉喂给非 shell 命令的 heredoc 正文（往笔记里写一段等待循环不是在跑它；2026-09-18 撞过，
# `records/2026-09-16-subagent拆分提案.md` 第二十五节）；喂给 bash / sh 的 heredoc 正文照查。其余只看顶层命令文本，
# 引号里当数据写的等待循环也会被记一条，主 agent 看了判断即可。
#
# 拒绝七种（退出 2，stderr 写原因与出路；拒了不记检出）。它们拒的是一种危险写法、在执行之前，不停在跑的任务，
# 是 .claude/singlefs-ai-sop/rules/command-safety.md「检出和处置分开」那一条写明的例外（「拒绝某一种危险写法的钩子不在此列」）。
# ① 命令位置上起看门狗——`research/scripts/watch.sh`（带 `--selftest`、`--report`、`--dry-run` 的不盯，不算）
#   或 `research/scripts/agent-watch.py watch`——而 run_in_background 不是 true，或整条命令里有单独的 `&`、`nohup`、`disown`、`setsid`、
#   把输出重定向到 /dev/null：这样起的看门狗叫不醒主 agent，等于没盯。2026-09-24 主 agent 同一个会话里第三次这样起，
#   `main-agent.md`「调用里不再加 `&`、`nohup`、`disown`」靠自觉守不住（records/2026-09-16-subagent拆分提案.md 第四十节那张表第 5 行）。
#   只认命令位置：按 shell 的规矩切词（引号与 # 注释去掉），`grep -n watch.sh`、`cat` / `tail` 读看门狗输出这类把名字当参数的不拦；
#   `bash -c '…'`、`capped.sh N …`、`source`、`python3 脚本` 与 nice / timeout / env / taskset 这类前缀里面的照判，`bash -n`（只查语法）不算，
#   喂给非 shell 命令的 heredoc 正文每一层都先剥掉。
#   命令替换 `$(…)` 与反引号在双引号里外都判（里面的命令另按一条完整的命令判，lib_shell_words.py）。
#   判不到的：变量里拼出来的命令、eval、`<<<` 喂给 shell 的字符串；喂给 shell 的 heredoc 正文里引号没配对时，其后的切词会错。
# ② 前台（run_in_background 不是 true）的命令里有没超时的等待循环：命令位置上的 `until` / `while` 循环，条件或循环体里有一条 `sleep`
#   （剥掉 nice / timeout 这类前缀之后命令词是 sleep），外面又没有 `timeout`。2026-09-25 代码三方攻方腿在前台跑
#   `until grep -q '^rc=' <日志>; do sleep 10; done`，主 agent 发去的纠正消息要等它下一次调用工具才送得到，它却卡在这一次调用里，
#   只好按 pid 从外面停掉那个循环（records/2026-09-16-subagent拆分提案.md 第四十节那张表第 20 行）：只记检出叫得醒主 agent，叫不醒卡住的那一个。
#   主 agent 与子 agent 都拒；run_in_background 里的等待循环照旧只记检出（①）。
#   「外面有 timeout」按包装的层次认：`timeout N bash -c '…'`、`capped.sh N timeout N bash -c '…'`、`timeout N bash <<EOF` 喂进去的正文里的循环算有；
#   只在循环条件或循环体里的 timeout（`until timeout 5 grep …; do sleep 10; done`）不算，循环照样能一直转下去。
#   只认命令位置（按 shell 的规矩切词），引号里当数据写的、写进文件的 heredoc 正文、注释里的循环不拒；
#   `bash -c '…'`、`capped.sh N …`、喂给 shell 的 heredoc 正文、命令替换里面的递归判。多行写的循环（`do` 另起一行）同样认。
#   不带 sleep 的循环（`while read -r 行; do …; done < 文件`）、`for` 循环不拒；带计数上限的 while 循环机器分不出上限，照拒，外面套一层 timeout 就放行。
#   判不到的：变量里拼出来的命令、eval、`bash x.sh` 起的脚本文件里的循环。
# ③ 命令位置上把活放出追踪的写法：`disown`、`coproc`、`setsid -f` / `--fork`、`nohup` 而同一层有单独的 `&`、`tmux new(-session) … -d`、
#   `screen -dm…` / `-d -m`、不带 --wait / --pipe / --pty / --scope 的 `systemd-run`——前台、run_in_background 一样拒，主 agent 与子 agent 都拒。
#   进程移出 shell 的作业表或另起会话之后，随后的 `wait` 立刻返回、harness 的完成通知当场发出，进程成了没人追踪的孤儿接着跑
#   （2026-09-25 一个核查员写 `… & disown 2>/dev/null || true & wait`）；`.claude/agent-common.md`「长活可以等」那一条明令不用这几种，靠自觉守不住。
#   每一层（`bash -c '…'`、喂给 shell 的 heredoc 正文、命令替换）都判，nice / timeout / env 这类前缀、`capped.sh N`、`bash 包装脚本` 剥开往里找；
#   引号里当数据写的、写进文件的 heredoc 正文、注释里的不拒；`nohup` 不带 `&`、裸 `setsid`（不 fork）、`systemd-run --wait` 这类会等到结束的不拒；
#   单独的 `&` 不在这一条：run_in_background 里之后没有 `wait` 的归 ④，其余照检出 ② 只记。
#   判不到的：变量里拼出来的命令、eval、`bash x.sh` 起的脚本文件里的写法、`start-stop-daemon -b`（只在 ② 里记）。
# ④ run_in_background 为 true 的命令里，有一个作业以单独的 `&` 收尾（不是 `&&`、`>&`、`&>`、`|&`），之后同一层、同一对圆括号里再没有 `wait`
#   （不带参数的 `wait`，或带进程号的 `wait "$pid"`；只带选项的 `wait -n` 不算）：外层 shell 起完它就退出，完成通知当场发出，
#   真跑完的那个进程叫不醒任何人。2026-09-25 一天里子 agent 至少三次在 run_in_background 里写 `… > 日志 2>&1 &`
#   （records/2026-09-16-subagent拆分提案.md 第四十节那张表第 23 行）。主 agent 与子 agent 都拒；前台的 `… &` 不拒。
#   按共用模块切出的记号逐层判：`bash -c '…'`（capped.sh N、nice 这类包着的剥开）、喂给 shell 的 heredoc 正文、命令替换各是一层，
#   里面的 `&` 要在同一层里等；圆括号是子 shell，`(… &); wait`、`… & (wait)` 里的 `wait` 等不到那个作业，照拒；
#   `{ …; }`、循环与 if 的身子与外面同一层。
#   `&` 之后按进程号轮询（`tail --pid`、`while kill -0`）不算 wait，照拒。起了几个 `&`、之后的 `wait "$pid"` 只等了其中一个的，
#   机器分不出等全没有，不拒，照检出 ② 记一条。引号里当数据写的、写进文件的 heredoc 正文、注释里的不拒。
#   判不到的：变量里拼出来的命令、eval、`bash x.sh` 起的脚本文件里的写法。
# ⑤ 命令位置上会整份覆盖 `research/results/` 下一个已存在、又没进 git（`git ls-files --error-unmatch` 失败）的文件：重定向 `>`、`>|`、`&>`、`>& 文件`
#   （前面带 fd 号的 `2>` 一样截断，一样算）、不带 `-a` / `--append` 的 `tee` 写的每个文件、`cp` / `mv` / `install` 的目标（`-t 目录`、或目标是已有目录、
#   以 `/` 结尾的，写的是 目录/源的文件名，源里的通配按这一刻的文件展开）。2026-09-25 E158 第九段执行员用 `cp` 把 s8 同名的三份未跟踪产物整份覆盖，
#   旧字节找不回（records/2026-09-16-subagent拆分提案.md 第四十节那张表第 24 行）；write-guard.sh 只拦 Write / Edit 工具，拦不到 shell。
#   前台、run_in_background 一样拒，主 agent 与子 agent 都拒。
#   放行：`>>`、`&>>`、`tee -a` 追加；目标不存在（新文件名）；目标已进 git（`git add` 过就算，git 兜得住）；目标不在仓库根的 `research/results/` 下；
#   `2>&1`、`>&2`、`>&-` 这类 fd 复制与关闭；`cp` / `mv` 带 `-n`、`--no-clobber`、`--update=none`、`-b`、`--backup` 的（已有的目标不被整份盖掉）；`install -d`；
#   同一条命令里排在写之前 `git add` 过、或被 `mv` 挪走的那个文件（拒绝信息给的两条出路写进同一条命令也认）。
#   目标按命令里的 cd / pushd 跟着算；cd 算不出来（`cd "$变量"`、`cd -`、`popd`）或没有 cd 时按仓库根（这个 hook 所在的仓，`.claude/hooks/` 往上两级）解析。
#   这条命令里前面独立赋过值的变量（`out=…`、`export out=…`）在目标里的 `$out`、`${out}` 先代入。
#   目标这一刻算不出来（没赋过值的变量、命令替换、通配、花括号展开）的不拦：算不出的段换成 `*` 去对 research/results/ 下已有的文件，
#   对得上没进 git 的就记一条检出（对不上的一定是新文件名，不记）；git 自己出错、判不了进没进 git 的也只记检出。
#   每一层（`bash -c '…'`、喂给 shell 的 heredoc 正文、命令替换与进程替换）都判，nice / timeout / env / sudo 这类前缀、`capped.sh N` 与 `bash 包装脚本` 剥开往里找；
#   引号里当数据写的、写进文件的 heredoc 正文、注释里的不拒。
#   判不到的：变量里拼出来的命令、eval、`bash x.sh` 起的脚本文件里的写法、xargs 与 `find -exec` 起的 cp、`cp -r` 整目录拷进已有目录时里面逐个文件、
#   `dd of=`、`sed -i`、`sort -o`、`rsync`、`ln -f` 与 python 里 `open(…, 'w')` 这类别的写法；圆括号子 shell 里的 cd 当成对后面的命令也生效；
#   不看 hook 输入里的 cwd（没有 cd 的相对路径一律按仓库根解析）。
# ⑥ 终止进程只许点名一个自己起的进程号或任务号，一次一个。用户 2026-09-25 JST 21:0x–21:1x 原话：「后面的脚本不能终止前面的脚本 这是核心」
#   「另外不能动ssh 这是基本的」「终止要按照任务号 终止 禁止终止所有」。起因：2026-09-25 UTC 11:12 一个子 agent 跑 run-with-memory-cap.sh 的弄坏开关自证，
#   自测代码读自己 cgroup 的 cgroup.procs、给里面每个进程发 TERM，而不开 scope 的那一支里它就在 SSH 会话的 session-1.scope 里：
#   VSCode 服务端、扩展宿主、ptyHost 与 Claude 会话一起断掉（records/2026-09-16-subagent拆分提案.md 第四十节那张表第 32 行）。
#   前台、run_in_background 一样拒，主 agent 与子 agent 都拒。拒的写法：
#   kill 的目标带负号（进程组；-1 是能发到的全部）、是 0（发命令的整个进程组）或 $PPID（会话本身）；一次给了不止一个目标；目标是命令替换
#   （`$(pgrep …)`、`$(jobs -p)`）、数组或位置参数的整批展开、通配；在 for / while / until 循环里逐个发（`kill -0` 探活、`kill -l` 不算发）；
#   `proc.py stop` 同样一次一个、不在循环里；xargs / parallel / find -exec 把一串进程号喂给 kill 或 proc.py；pkill、killall、skill 按名字挑；fuser -k；
#   同一条命令里先从 cgroup.procs、/proc 或 pgrep / pidof 挑进程、再发信号；往 cgroup.kill、cgroup.freeze 写；
#   systemctl 停、杀、重启、改属性（stop、kill、restart 一族、freeze、set-property、disable / mask --now）ssh、sshd、session-*.scope、user@*.service、
#   user.slice、user-*.slice、vllm-prod、systemd-logind、dbus，或单元带通配、是命令替换；systemctl isolate、poweroff、reboot 一类与 --user exit；
#   loginctl terminate-* / kill-*；reboot、poweroff、halt、shutdown（shutdown -c 不算）；
#   目标是写死的进程号时现读进程表：是这条命令所在会话自己的祖先（Claude 会话、VSCode 扩展宿主往上）、pid 1、名叫 sshd / systemd / dbus / logind / claude 的、
#   命令行里带 .vscode-server 或 vllm 的，或它的祖先里有另一个 Claude 会话（别的会话起的）——都拒；读不到那个进程的不拦（kill 自己会报错）。
#   python3 -c 与喂给 python 的 heredoc 里的代码同样判：os.killpg、os.kill 的目标是 0 或负数、在循环或推导式里调 os.kill、同一个函数里读 cgroup.procs 或 /proc 再 os.kill。
#   放行：`kill "$!"`、`kill -TERM "$pid"`、`kill %1`、`kill %%` 这样点名一个的；`proc.py stop <一个 pid>`；
#   systemctl 对任何单元的 status / show / list-units / reset-failed，对自己起的单元（`systemctl --user stop singlefs_memory_selftest_…slice`）的停；
#   只读 cgroup.procs、只跑 pgrep 不发信号的；引号里当数据写的、写进文件的 heredoc 正文、注释里的。
#   每一层（`bash -c '…'`、喂给 shell 的 heredoc 正文、命令替换）都判，nice / timeout / env / sudo / builtin 这类前缀、`capped.sh N`、`run-with-memory-cap.sh <上限>`、
#   `bash 包装脚本` 与 `python3 脚本` 剥开往里找。
#   同一份判定由门禁 73 号拿去扫 research/scripts/、.claude/hooks/、.claude/scripts/ 与 .claude/gate.d/ 下的脚本（本文件 --scan-scripts，按行号报；
#   不判「同一条命令里先挑再发」那一条，脚本里靠循环与命令替换两条）。脚本里确实要按 cgroup 批量发信号的（run-with-memory-cap.sh 的 TERM 风暴自测），
#   先核 cgroup 路径是自己开的 singlefs-memory-cap-*.scope，那一行写 `# process-safety:own-scope <怎么核的>`，往上 40 行里要有核 `singlefs-memory-cap-*.scope` 的那一句；
#   这个标注只放行循环与按 cgroup 挑进程两类，标了而那一行没有要放行的判红。别人正在改、这一轮动不了的脚本登记进 .claude/process-safety-pending
#   （一行一个相对仓根的路径、# 后写为什么；指向不存在的、一处都没排到的判红），成功行逐个列名。
#   判不到的：变量里拼出来的命令、eval、`bash x.sh` 起的脚本文件里的写法（脚本归门禁 73 号）；`kill $pids` 这种不带引号、里面其实放了几个进程号的变量；
#   先把 cgroup.procs 读进文件、下一条命令再读文件 kill；循环里 `bash -c "kill $p"`（里面那一层不知道自己在循环里）；python 里经 subprocess 起的 kill、
#   Popen 的 terminate / kill；进程号写死而那一刻进程表读不到的。
# ⑦ 在同一个 inode 上改一个已经存在的脚本（`.sh`、`.py`，或带执行位的文件；在仓里或 /tmp/claude-1000/ 下）：正在跑它的 bash 按文件偏移往下读，
#   改完会从新内容的同一偏移接着读，读到的是别的东西。2026-09-25 `replace-once.py` 就地改了一个别的 agent 正经 `run-with-memory-cap.sh` 在跑的脚本
#   （records/2026-09-16-subagent拆分提案.md 第四十节那张表第 32 行）；两个工具改成改名换上之后，用户 2026-09-25 JST 22:1x 要给手敲的就地写也加闸。
#   前台、run_in_background 一样拒，主 agent 与子 agent 都拒。拒的写法（每一种都实测过：改之前打开文件的读者接着读到新内容）：
#   重定向 `>`、`>|`、`&>`、`>& 文件`（带 fd 号的 `2>` 一样）、`cat > 旧`；不带 `-a` / `--append` 的 `tee`；`cp` 的目标（`-f`、`-u`、`-a` 一样，写进已有目录的按 目录/源的文件名算）；
#   `dd of=`（带不带 `conv=notrunc` 都算）；`truncate` 的文件；喂给 python 的代码（`python3 -c` 与喂给它的 heredoc）里 `open(旧, 带 w 或 r+ 的模式)`（io / codecs 的 open 一样）、
#   `Path(旧).open(同上)`、`Path(旧).write_text` / `write_bytes`、`shutil.copyfile` / `copy` / `copy2` 的目标。
#   放行：`>>`、`&>>`、`tee -a`、python 的 `'a'` 追加（不动已经读过的偏移之前的字节）；目标不存在（新建）；目标不是脚本；目标在仓外又不在 /tmp/claude-1000/ 下；
#   换 inode 的写法：`mv`、`install`、`cp --remove-destination`、`cp -l` / `-s`、`cp -n` / `-b` / `--update=none`、`os.replace`；
#   同一条命令里排在写之前 `mv` 挪走或 `rm` 掉的那个文件（之后写的是一个新 inode）。
#   目标跟着 cd / pushd 与这条命令里前面赋过值的变量算，与 ⑤ 同一套（overwrite_steps）；python 的相对路径按跑它那一刻的当前目录算。
#   python 的路径只认字符串字面量、整段代码里只赋过一次的名字、Path(…) 与 Path(…) / '…'、os.path.join 的字面量；模式只认字面量。
#   判不到的：变量里拼出来的命令、eval、`bash x.sh` 起的脚本文件里的写法、目标里有这一刻算不出的段（没赋过值的变量、命令替换、通配）、
#   python 里从 sys.argv 或别的运行期值来的路径、`os.open` 与 `os.write`、经 subprocess 起的写法；`rsync --inplace` 这类别的就地写。
#   Edit / Write 工具本来就换 inode，不经这个 hook。
#
# 切词、切简单命令、认命令位置、剥前缀与包装、跟 cd：同目录的 lib_shell_words.py，与 heavy-test-guard.sh 共用一份，按文件路径导入，
# 这里只留检出与拒绝的判定；那几个函数在 .claude/hooks/ 别的文件里再定义一份，门禁 63 号判红。
# 读不到它时这条命令照常执行，stderr 报一句并记一条检出。
#
#   bash-command-detector.sh             # 从 stdin 读 hook 的 JSON；拒绝七种之一退出 2（拒绝），其余退出 0
#   bash-command-detector.sh --scan-scripts <仓根> <目录>…  # ⑥ 的判定扫目录里的 .sh 与 .py（门禁 73 号调）；有红退出 1
#   bash-command-detector.sh --selftest  # 走一遍检出、不检出、拒绝与放行；BASH_COMMAND_DETECTOR_DISABLE_CHECK=1（两种检出、后六种拒绝与 ⑤ 的检出都关）、
#                                        # BASH_COMMAND_DETECTOR_DISABLE_SELF_BACKGROUND=1（只关第三种）或
#                                        # BASH_COMMAND_DETECTOR_KEEP_HEREDOC_BODIES=1（前两种不剥 heredoc 正文）或
#                                        # BASH_COMMAND_DETECTOR_ALLOW_WATCHDOG_MISUSE=1（起看门狗的错误写法也放行）或
#                                        # BASH_COMMAND_DETECTOR_ALLOW_FOREGROUND_WAIT_LOOP=1（前台没超时的等待循环也放行）或
#                                        # BASH_COMMAND_DETECTOR_ALLOW_DETACHING=1（把活放出追踪的写法也放行）或
#                                        # BASH_COMMAND_DETECTOR_ALLOW_UNWAITED_AMPERSAND=1（run_in_background 里后面没有 wait 的单独 & 也放行）或
#                                        # BASH_COMMAND_DETECTOR_ALLOW_RESULTS_OVERWRITE=1（整份覆盖 research/results/ 下未跟踪产物也放行、也不记检出）或
#                                        # BASH_COMMAND_DETECTOR_ALLOW_PROCESS_SIGNALS=1（⑥ 终止进程的写法也放行）或
#                                        # BASH_COMMAND_DETECTOR_ALLOW_SCRIPT_IN_PLACE_WRITE=1（⑦ 在同一个 inode 上改已有脚本也放行）或
#                                        # BASH_COMMAND_DETECTOR_REFUSE_EVERY_WAIT_LOOP=1（外层 timeout 与 run_in_background 都不算，等待循环一律拒）时自检必须判红
set -uo pipefail
HOOK_DIR="$(cd "$(dirname "$0")" && pwd)"
# python 程序从文件描述符 3 读，标准输入留给 hook 的 JSON（与 agent-write-scope.sh 同一个坑）。
python3 /dev/fd/3 "$HOOK_DIR" "$@" 3<<'PY'
import ast, glob, importlib.util, io, json, os, re, shutil, subprocess, sys, tempfile, tokenize
from datetime import datetime, timezone
from typing import NamedTuple

def shell_words_library_path(hook_dir):
    return os.path.join(hook_dir, "lib_shell_words.py")

def load_shell_words(library_path):
    """切词与认命令位置的共用模块，与 heavy-test-guard.sh 同一份；读不到或导入出错就抛异常，由调用方处理。"""
    spec = importlib.util.spec_from_file_location("lib_shell_words", library_path)
    library = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(library)
    return library

try:
    shell_words, shell_words_error = load_shell_words(shell_words_library_path(sys.argv[1])), None
except Exception as error:  # 文件不在、语法错、导入时抛的都算读不到
    shell_words, shell_words_error = None, error

WAIT_LOOP = re.compile(r"\b(until|while)\b[^\n]*?;\s*do\b[\s\S]*?\bsleep\b")
SELF_DETACH = re.compile(
    r"\bsetsid(?:\s+-[-\w]+)*?\s+(?:--fork\b|-[a-eg-z]*f[a-z]*\b)"
    r"|\bdisown\b|\bcoproc\b"
    r"|\bnohup\b[^\n]*(?<![&>|<])&(?![&>])"
    r"|\btmux\b[^\n]*\bnew(?:-session)?\b[^\n]*\s-d\b"
    r"|\bscreen\b[^\n]*\s-(?:[dD]m\w*|d\s+-m)"
    r"|\bsystemd-run\b(?![^\n]*--(?:wait|pipe|pty|scope)\b)"
    r"|\bstart-stop-daemon\b[^\n]*\s(?:-b|--background)\b")
LONE_AMPERSAND = re.compile(r"(?<![&>|<])&(?![&>])")
WAIT_COMMAND = re.compile(r"(?:^|[;&|\n({]|\b(?:then|do|else)\b)\s*wait\b(?![\w.(-])([^;&|\n)}]*)")
SUBSHELL_AMPERSAND = re.compile(r"(?<![&>|<])&\s*\)|\$\([^()]*(?<![&>|<])&(?![&>])[^()]*\)")
POLL_AFTER = re.compile(r"\btail\b[^\n]*--pid\b|\b(?:until|while)\b[^\n]*\bkill\s+-0\b")

def puts_itself_in_background(command):
    command = shell_words.strip_data_heredocs(command)
    if SELF_DETACH.search(command) or SUBSHELL_AMPERSAND.search(command):
        return True
    ampersands = list(LONE_AMPERSAND.finditer(command))
    for match in ampersands:
        rest = command[match.end():]
        if POLL_AFTER.search(rest):
            continue
        waits = [found.group(1).strip() for found in WAIT_COMMAND.finditer(rest)]
        if any(arguments == "" for arguments in waits):
            continue
        if len(ampersands) == 1 and any(arguments and not arguments.startswith("-n") for arguments in waits):
            continue
        return True
    return False
DEFAULT_DETECTIONS = "/tmp/claude-1000/agent-hook-detections.jsonl"

def findings_for(command, run_in_background=False):
    if os.environ.get("BASH_COMMAND_DETECTOR_DISABLE_CHECK") == "1":
        return []
    findings = []
    executed = command if os.environ.get("BASH_COMMAND_DETECTOR_KEEP_HEREDOC_BODIES") == "1" else shell_words.strip_data_heredocs(command)
    if WAIT_LOOP.search(executed) and not re.search(r"\btimeout\b", executed):
        findings.append("没有 timeout 的等待循环")
    if run_in_background and puts_itself_in_background(command) and os.environ.get("BASH_COMMAND_DETECTOR_DISABLE_SELF_BACKGROUND") != "1":
        findings.append("run_in_background 里又自己放后台（nohup / setsid / disown / 后面没有 wait 的 &）：完成通知当场发出，跑完的那个不会叫醒你")
    return findings

def record(hook_input, detections_path, extra_findings=()):
    """检出就追加一行；返回写了几行。永远不拦命令。extra_findings 是入口另外判出、只记不拦的（⑤ 目标算不出、git 判不了的）。"""
    tool_input = hook_input.get("tool_input") or {}
    findings = findings_for(tool_input.get("command") or "", bool(tool_input.get("run_in_background"))) + list(extra_findings)
    if not findings:
        return 0
    append_detection(hook_input, findings, detections_path)
    return 1

def append_detection(hook_input, findings, detections_path):
    tool_input = hook_input.get("tool_input") or {}
    command = tool_input.get("command") or ""
    entry = {
        "time": datetime.now(timezone.utc).isoformat(),
        "session_id": hook_input.get("session_id"),
        "agent_id": hook_input.get("agent_id"),
        "agent_type": hook_input.get("agent_type") or "主 agent",
        "transcript_path": hook_input.get("transcript_path"),
        "command": command[:500],
        "findings": findings,
    }
    os.makedirs(os.path.dirname(detections_path), exist_ok=True)
    with open(detections_path, "a", encoding="utf-8") as handle:
        handle.write(json.dumps(entry, ensure_ascii=False) + "\n")

# 起看门狗的写法：只认命令位置，所以先按 shell 的规矩切词（共用模块），不拿正则在整条文本上找名字。
WATCH_SH_MODES_THAT_DO_NOT_WATCH = {"--selftest", "--report", "--dry-run"}

def watchdog_calls_and_detaching(text):
    """返回 (命令位置上起看门狗的调用, 整条命令里放后台或丢输出的写法)；bash -c、capped.sh 这类包装里的由共用模块递归进去一起交回。"""
    scan = shell_words.commands_at_command_position(text)
    calls, detaching = [], []
    if scan.has_lone_ampersand:
        detaching.append("命令里有单独的 &")
    if "/dev/null" in scan.output_targets:
        detaching.append("输出重定向到了 /dev/null")
    detaching += [f"命令里有 {prefix}" for prefix in ("nohup", "setsid") if prefix in scan.prefix_names]
    for command in scan.commands:
        name, arguments = os.path.basename(command.words[0]), command.words[1:]
        if name == "disown":
            detaching.append("命令里有 disown")
        if name == "watch.sh" and not WATCH_SH_MODES_THAT_DO_NOT_WATCH.intersection(arguments):
            calls.append(" ".join(["watch.sh", *arguments]))
        elif name == "agent-watch.py" and "watch" in arguments and arguments[:1] != ["--selftest"]:
            calls.append(" ".join(["agent-watch.py", *arguments]))
    return calls, detaching

def watchdog_rejection(command, run_in_background):
    """起看门狗的写法不对就返回 (原因, 认出的调用)；写法对、或根本没起看门狗，原因为空。"""
    if os.environ.get("BASH_COMMAND_DETECTOR_ALLOW_WATCHDOG_MISUSE") == "1":
        return [], []
    calls, detaching = watchdog_calls_and_detaching(command)
    if not calls:
        return [], []
    reasons = [] if run_in_background is True else ["run_in_background 不是 true"]
    reasons += list(dict.fromkeys(detaching))
    return reasons, calls

# 前台没超时的等待循环：只认命令位置，按共用模块切出的简单命令与开头的关键字找循环的头尾，不拿正则在整条文本上找
WAIT_LOOP_KEYWORDS = {"while", "until"}
BLOCKS_CLOSED_BY_DONE = {"while", "until", "for", "select"}

def loop_keyword_changes(words):
    """一条简单命令开头的关键字：交回 (开了几层要 done 收的块, 收了几层, 开的是不是 while / until)。"""
    opened, closed, opens_wait_loop = 0, 0, False
    for word in words:
        if word == "done":
            closed += 1
        elif word in BLOCKS_CLOSED_BY_DONE:
            opened, opens_wait_loop = 1, word in WAIT_LOOP_KEYWORDS
            break
        elif word not in shell_words.COMMAND_KEYWORDS:
            break
    return opened, closed, opens_wait_loop

def is_sleep_command(words):
    """剥掉开头的关键字与 nice / timeout 这类前缀之后，命令词是不是 sleep。"""
    skipped = shell_words.skip_prefixes(words)
    return skipped.command_position is not None and os.path.basename(words[skipped.command_position]) == "sleep"

def wrapped_commands(words):
    """一条简单命令一层层剥开前缀、capped.sh N 与 `bash 包装脚本`（bash 起的是脚本时接着往里剥）：
    依次交回 (这一层命令词前面的前缀词, 路过的前缀名, 命令词的名字, 参数)；最后只剩关键字、赋值与前缀的，名字是 None。"""
    remaining = words
    for _ in range(shell_words.MAXIMUM_NESTING):
        skipped = shell_words.skip_prefixes(remaining)
        if skipped.command_position is None:
            yield remaining, skipped.prefix_names, None, []
            return
        prefix_words, remaining = remaining[:skipped.command_position], remaining[skipped.command_position:]
        name, arguments = os.path.basename(remaining[0]), remaining[1:]
        yield prefix_words, skipped.prefix_names, name, arguments
        if name in shell_words.WRAPPERS_TAKING_ONE_ARGUMENT and len(arguments) >= 2 and not arguments[0].startswith("-"):
            remaining = arguments[1:]
            continue
        if name in shell_words.SHELL_NAMES:
            invocation = shell_words.shell_invocation(arguments)
            if invocation.code_string is None and invocation.script_index is not None:
                remaining = arguments[invocation.script_index:]
                continue
        return

def shell_code_and_timeout(words):
    """一条简单命令剥开包装之后：交回 (bash -c 的那段代码或 None, 路过的前缀里有没有 timeout)。"""
    has_timeout = False
    for _, prefix_names, name, arguments in wrapped_commands(words):
        has_timeout = has_timeout or "timeout" in prefix_names
        if name in shell_words.SHELL_NAMES:
            code = shell_words.shell_invocation(arguments).code_string
            if code is not None:
                return code, has_timeout
    return None, has_timeout

class ShellLayer(NamedTuple):
    commands: list             # 这一层的简单命令，每条是词列表（重定向去掉、包装没剥）
    has_lone_ampersand: bool   # 这一层有单独的 &（放后台；不是 &&、>&、&>、|&）
    under_timeout: bool        # 这一层外面有 timeout（挂它的那条命令、或更外面一层带 timeout 前缀）
    tokens: list               # 这一层的记号（共用模块 shell_tokens 切的，喂给 shell 的 heredoc 正文已剥出去另成一层）

def shell_layers(text, under_timeout=False, honor_timeout=True, depth=0):
    """整条命令的每一层：顶层、bash -c 的那段代码、喂给 shell 的 heredoc 正文、命令替换，各交一个 ShellLayer。
    外面那条带 timeout 前缀的，里面各层都算有 timeout；honor_timeout 为假时 timeout 一律不算（自检的弄坏开关用）。"""
    if depth > shell_words.MAXIMUM_NESTING:
        return []
    shell_heredocs = []
    tokens = shell_words.shell_tokens(shell_words.strip_data_heredocs(text, shell_heredocs=shell_heredocs))
    commands, _, substitutions, _ = shell_words.simple_commands(tokens)
    layers = [ShellLayer(commands, (True, "&") in tokens, under_timeout, tokens)]
    for head, body in shell_heredocs:
        head_commands = shell_words.simple_commands(shell_words.shell_tokens(head))[0]
        feeds_under_timeout = bool(head_commands) and shell_code_and_timeout(head_commands[-1])[1] and honor_timeout
        layers += shell_layers(body, under_timeout or feeds_under_timeout, honor_timeout, depth + 1)
    for words, command_substitutions in zip(commands, substitutions):
        for substitution in command_substitutions:
            layers += shell_layers(substitution, under_timeout, honor_timeout, depth + 1)
        code, has_timeout = shell_code_and_timeout(words) if words else (None, False)
        if code is not None:
            layers += shell_layers(code, under_timeout or (has_timeout and honor_timeout), honor_timeout, depth + 1)
    return layers

def wait_loops_in(commands):
    """一层里的 until / while 循环，条件或循环体里有 sleep 的，交回每个循环开头那条简单命令。"""
    found, index = [], 0
    while index < len(commands):
        if not loop_keyword_changes(commands[index])[2]:
            index += 1
            continue
        end, level = index + 1, 1
        while end < len(commands) and level > 0:
            opened, closed, _ = loop_keyword_changes(commands[end])
            level += opened - closed
            end += 1
        if any(is_sleep_command(words) for words in commands[index:end] if words):
            found.append(" ".join(commands[index])[:120])
            index = end
        else:
            index += 1
    return found

def unbounded_wait_loops(text, honor_timeout=True):
    """命令位置上的 until / while 循环里（条件或循环体）有 sleep、外面又没有 timeout 的，交回每个循环开头那条简单命令。"""
    return [loop for layer in shell_layers(text, honor_timeout=honor_timeout) if not layer.under_timeout for loop in wait_loops_in(layer.commands)]

# 把活放出追踪的写法（agent-common.md「长活可以等」那一条列的）：命令词是这几个的
DETACHING_COMMANDS = {"disown", "coproc"}

def detaching_command(name, arguments):
    """命令词与参数是不是 tmux / screen 的分离模式、不等结束的 systemd-run、disown、coproc；是就交回认出的写法，不是交 None。"""
    if name in DETACHING_COMMANDS:
        return name
    if name == "tmux" and any(argument in ("new", "new-session") for argument in arguments) and any(
            argument.startswith("-") and not argument.startswith("--") and "d" in argument[1:] for argument in arguments):
        return "tmux new -d"
    if name == "screen" and (any(re.match(r"^-[dD]m", argument) for argument in arguments)
                             or ({"-d", "-D"} & set(arguments) and "-m" in arguments)):
        return "screen -dm"
    if name == "systemd-run" and not any(argument in ("--wait", "--pipe", "--pty", "--scope", "-P", "-t") for argument in arguments):
        return "systemd-run（不带 --wait / --pipe / --pty / --scope）"
    return None

def detaching_prefixes(prefix_words, has_lone_ampersand):
    """命令词前面那段前缀里的 `setsid -f` 与（这一层有单独的 & 时的）`nohup`。"""
    found = []
    for index, word in enumerate(prefix_words):
        name = os.path.basename(word)
        if name == "setsid":
            options = []
            for option in prefix_words[index + 1:]:
                if not option.startswith("-") or option == "-":
                    break
                options.append(option)
            if any(option == "--fork" or (not option.startswith("--") and "f" in option[1:]) for option in options):
                found.append("setsid -f")
        elif name == "nohup" and has_lone_ampersand:
            found.append("nohup … &")
    return found

def detaching_forms(text):
    """命令位置上把活放出追踪的写法：disown、coproc、setsid -f、nohup … &、tmux / screen 的分离模式、不等结束的 systemd-run。
    每一层（bash -c、喂给 shell 的 heredoc、命令替换）都判，前缀、capped.sh N 与 `bash 包装脚本` 剥开往里找。"""
    found = []
    for layer in shell_layers(text):
        for words in layer.commands:
            for prefix_words, _, name, arguments in wrapped_commands(words):
                found += detaching_prefixes(prefix_words, layer.has_lone_ampersand)
                form = detaching_command(name, arguments) if name else None
                if form:
                    found.append(form)
    return list(dict.fromkeys(found))

def detaching_refusal(command):
    """命令位置上有把活放出追踪的写法就交回认出的写法；否则交 []。前台、run_in_background 一样拒。"""
    if (os.environ.get("BASH_COMMAND_DETECTOR_DISABLE_CHECK") == "1"
            or os.environ.get("BASH_COMMAND_DETECTOR_ALLOW_DETACHING") == "1"):
        return []
    return detaching_forms(command)

def wait_loop_refusal(command, run_in_background):
    """前台（run_in_background 不是 true）的命令里有没超时的等待循环，就交回认出的循环（每个循环开头那条简单命令）；否则交 []。"""
    if (os.environ.get("BASH_COMMAND_DETECTOR_DISABLE_CHECK") == "1"
            or os.environ.get("BASH_COMMAND_DETECTOR_ALLOW_FOREGROUND_WAIT_LOOP") == "1"):
        return []
    refuse_every = os.environ.get("BASH_COMMAND_DETECTOR_REFUSE_EVERY_WAIT_LOOP") == "1"
    if run_in_background is True and not refuse_every:
        return []
    return unbounded_wait_loops(command, honor_timeout=not refuse_every)

# run_in_background 里后面没有 wait 的单独 &：逐层按记号找以单独的 & 收尾的作业，看之后同一层、同一对圆括号里有没有 wait
WORDS_THAT_ONLY_CLOSE_A_COMPOUND = {"done", "fi", "esac", "}"}

def jobs_with_endings(tokens):
    """一层的记号按命令分隔符切成一条条简单命令，交回 [(词, 收尾的分隔符或 None, 所在的圆括号组)]。
    圆括号组是从外到里每一层开括号的序号（顶层是空元组），一对圆括号是一个子 shell；词由共用模块的 simple_commands 取
    （重定向连同目标去掉）。数组赋值 `名字=(…)` 的括号也当一组：括号成对，组的层次不乱，元素里既没有 & 也不会被当成同组的 wait。
    没有词的只留以 & 收尾的那种（`(…) &`）。"""
    jobs, piece, groups, opened_count = [], [], [()], 0
    for is_operator, text in tokens:
        if not (is_operator and text in shell_words.COMMAND_SEPARATORS):
            piece.append((is_operator, text))
            continue
        words = shell_words.simple_commands(piece)[0]
        if words or text == "&":
            jobs.append((words[0] if words else [], text, groups[-1]))
        piece = []
        if text == "(":
            opened_count += 1
            groups.append(groups[-1] + (opened_count,))
        elif text == ")" and len(groups) > 1:
            groups.pop()
    words = shell_words.simple_commands(piece)[0]
    if words:
        jobs.append((words[0], None, groups[-1]))
    return jobs

def is_wait_command(words):
    """剥掉开头的关键字与前缀之后命令词是 wait，且不带参数、或带了进程号（`wait "$pid"`）；只带选项的（`wait -n`）不算。"""
    skipped = shell_words.skip_prefixes(words)
    if skipped.command_position is None or words[skipped.command_position] != "wait":
        return False
    arguments = words[skipped.command_position + 1:]
    return not arguments or any(not argument.startswith("-") for argument in arguments)

def background_job_label(words):
    """拒绝信息里认出的作业：它的词；只剩 done / fi / } 这类收尾词的，是整个复合命令放了后台。"""
    if not [word for word in words if word not in WORDS_THAT_ONLY_CLOSE_A_COMPOUND]:
        return "（前面那个复合命令：圆括号、花括号、循环或 if）"
    return " ".join(words)[:120]

def unwaited_background_jobs(text):
    """整条命令每一层（顶层、bash -c 的代码、喂给 shell 的 heredoc 正文、命令替换）里以单独的 & 收尾、
    之后同一层同一对圆括号里再没有 wait 的作业，交回每个作业的说法。"""
    found = []
    for layer in shell_layers(text):
        jobs = jobs_with_endings(layer.tokens)
        for index, (words, ending, group) in enumerate(jobs):
            if ending != "&":
                continue
            if any(later_group == group and is_wait_command(later_words) for later_words, _, later_group in jobs[index + 1:]):
                continue
            found.append(background_job_label(words))
    return list(dict.fromkeys(found))

def unwaited_ampersand_refusal(command, run_in_background):
    """run_in_background 为 true 的命令里有以单独的 & 收尾、之后同一层没有 wait 的作业，就交回认出的作业；否则交 []。前台不拒。"""
    if (os.environ.get("BASH_COMMAND_DETECTOR_DISABLE_CHECK") == "1"
            or os.environ.get("BASH_COMMAND_DETECTOR_ALLOW_UNWAITED_AMPERSAND") == "1"):
        return []
    if run_in_background is not True:
        return []
    return unwaited_background_jobs(command)

# ⑤ 整份覆盖 research/results/ 下未跟踪的产物：按命令位置认会整份覆盖文件的写法，跟着 cd 与这条命令里赋过的变量算目标
OVERWRITING_REDIRECTS = {">", ">|", "&>", ">&"}   # >> 与 &>> 是追加；>& 后面跟 fd 号或 - 时是复制、关闭 fd，不写文件
FILE_DESCRIPTOR_WORD = re.compile(r"^(?:\d+-?|-)$")
ASSIGNED_VARIABLE = re.compile(r"\$(?:\{([A-Za-z_][A-Za-z0-9_]*)\}|([A-Za-z_][A-Za-z0-9_]*))")
# 目标词里这一刻算不出的段：变量（含 ${…} 的各种展开与特殊参数）、花括号展开；命令替换另由共用模块的 substitution_span 认
UNKNOWN_PIECE = re.compile(r"\$\{[^}]*\}|\$[A-Za-z_][A-Za-z0-9_]*|\$[0-9@*#?$!-]|\{[^{}/]*(?:,|\.\.)[^{}/]*\}")
GLOB_CHARACTERS = set("*?[")
RESULTS_DIRECTORY_WORDS = ("research", "results")
# cp / mv / install：命令名 → 要带一个值的短选项；带值的长选项（不写 = 时值是下一个词）
COPY_SHORT_OPTIONS_WITH_VALUE = {"cp": "tS", "mv": "tS", "install": "tSgmo"}
COPY_LONG_OPTIONS_WITH_VALUE = {"--target-directory", "--suffix", "--no-preserve", "--sparse", "--group", "--mode", "--owner", "--strip-program"}
GIT_GLOBAL_OPTIONS_WITH_VALUE = {"-C", "-c", "--git-dir", "--work-tree", "--namespace", "--config-env"}
GIT_ADD_OPTIONS_THAT_DO_NOT_KEEP_BYTES = {"-n", "--dry-run", "-u", "--update", "-N", "--intent-to-add", "-p", "--patch", "-i", "--interactive", "-e", "--edit", "--refresh"}
RESULTS_PRESERVING_KINDS = {"git add", "mv"}   # rm 掉的旧字节没留住，⑤ 不认
IN_PLACE_ONLY_FORMS = {"dd", "truncate"}       # 只给 ⑦ 认的写法；⑤ 的射程不含它们（文件头 ⑤「判不到」里列着），要不要并进 ⑤ 另定

class OverwriteCandidate(NamedTuple):
    form: str                            # 认出的写法：>、>|、&>、>&、tee、cp、mv、install；⑦ 另认 dd、truncate（⑤ 不判这两种）
    target: str                          # 目标词（这条命令里前面赋过值的变量已代入）
    directory: str | None                # 那一刻的当前目录；cd 算不出来是 None，按仓库根解析
    sources: tuple = ()                  # cp / mv / install 的源
    into_directory: bool | None = False  # True：-t 给的目录；None：看目标是不是已有目录、以 / 结尾；False：目标就是写的那个文件
    replaces_inode: bool = False         # 先删掉目标或换成链接再写（cp --remove-destination、-l、-s）：⑦ 不拦，⑤ 照样算覆盖

class PreservingStep(NamedTuple):
    """同一条命令里让旧字节留住的一步：git add 的路径（进了 git）、mv 挪走的源（换了名字还在）；排在它后面再写这些路径不算覆盖。
    rm 删掉的路径也记一步（kind 为 rm）：只给 ⑦ 用，之后写那个名字是一个新 inode；⑤ 不认它，旧字节并没有留住。"""
    words: tuple
    directory: str | None
    kind: str = "git add"                # git add、mv、rm

class PythonCode(NamedTuple):
    """喂给 python 的一段代码（`python3 -c` 的，或喂给它的 heredoc 正文）与跑它那一刻的当前目录：只给 ⑦ 用。"""
    code: str
    directory: str | None

class CopyArguments(NamedTuple):
    operands: list                 # 非选项参数：源在前，目标在最后（给了 -t 时全是源）
    target_directory: str | None   # -t / --target-directory 给的目录
    treats_target_as_file: bool    # -T / --no-target-directory
    keeps_existing_target: bool    # -n / --no-clobber / --update=none… / -b / --backup：已有的目标不被整份盖掉
    creates_directories: bool      # install -d / --directory：参数全是要建的目录
    replaces_target_inode: bool = False  # cp --remove-destination、-l / --link、-s / --symbolic-link：目标先删掉或换成链接，不在原 inode 上写

def substitute_assigned_variables(word, variables):
    """`$名字`、`${名字}` 换成这条命令里前面赋过的值；没赋过的变量、命令替换、特殊参数照留。"""
    return ASSIGNED_VARIABLE.sub(lambda match: variables.get(match.group(1) or match.group(2), match.group(0)), word)

def tee_files(arguments):
    """tee 的参数：带 -a / --append 的交 []（追加），否则交它写的每个文件（`-` 是标准输出，不算）。"""
    files, options_ended = [], False
    for argument in arguments:
        if options_ended or argument == "-" or not argument.startswith("-"):
            if argument != "-":
                files.append(argument)
        elif argument == "--":
            options_ended = True
        elif argument.startswith("--"):
            if len(argument) >= 3 and "--append".startswith(argument):
                return []
        elif "a" in argument[1:]:
            return []
    return files

def parse_copy_arguments(name, arguments):
    """cp / mv / install 的参数按 GNU 的选项规矩切开（选项可以排在文件名后面，`--` 之后全是文件名）。"""
    operands, target_directory, treats_target_as_file, keeps_existing_target, creates_directories = [], None, False, False, False
    replaces_target_inode = False
    short_options_with_value = COPY_SHORT_OPTIONS_WITH_VALUE[name]
    position = 0
    while position < len(arguments):
        argument = arguments[position]
        position += 1
        if argument == "--":
            operands += arguments[position:]
            break
        if argument == "-" or not argument.startswith("-"):
            operands.append(argument)
            continue
        if argument.startswith("--"):
            option, has_value, value = argument.partition("=")
            if option in COPY_LONG_OPTIONS_WITH_VALUE and not has_value:
                value = arguments[position] if position < len(arguments) else ""
                position += 1
            if option == "--target-directory":
                target_directory = value
            elif option == "--no-target-directory":
                treats_target_as_file = True
            elif option in ("--no-clobber", "--backup") or (option == "--update" and value.startswith("none")):
                keeps_existing_target = True
            elif option == "--directory" and name == "install":
                creates_directories = True
            elif option in ("--remove-destination", "--link", "--symbolic-link") and name == "cp":
                replaces_target_inode = True
            continue
        letters = argument[1:]
        for index, letter in enumerate(letters):
            if letter in short_options_with_value:
                value = letters[index + 1:]
                if not value:
                    value = arguments[position] if position < len(arguments) else ""
                    position += 1
                if letter == "t":
                    target_directory = value
                break
            if letter == "T":
                treats_target_as_file = True
            elif letter in "nb":
                keeps_existing_target = True
            elif letter == "d" and name == "install":
                creates_directories = True
            elif letter in "ls" and name == "cp":
                replaces_target_inode = True
    return CopyArguments(operands, target_directory, treats_target_as_file, keeps_existing_target, creates_directories, replaces_target_inode)

def git_added_paths(arguments):
    """git 的参数：是 `git [全局选项] add …` 且真把内容放进 git（不是 -n、-u、-N、-p 这类）时，交回 (-C 给的目录列表, 加进去的路径)；否则交 None。
    `-A` / `--all` 不带路径时加的是整个工作树，交 ["."]（按 -C 与当前目录算，仓库根上跑时就是整个仓）。"""
    position, change_directories = 0, []
    while position < len(arguments) and arguments[position].startswith("-"):
        option = arguments[position].partition("=")[0]
        has_value_inline = "=" in arguments[position]
        position += 1
        if option in GIT_GLOBAL_OPTIONS_WITH_VALUE and not has_value_inline:
            if option == "-C" and position < len(arguments):
                change_directories.append(arguments[position])
            position += 1
    if arguments[position:position + 1] != ["add"]:
        return None
    paths, adds_everything, options_ended = [], False, False
    for argument in arguments[position + 1:]:
        if options_ended or not argument.startswith("-"):
            paths.append(argument)
        elif argument == "--":
            options_ended = True
        elif argument in GIT_ADD_OPTIONS_THAT_DO_NOT_KEEP_BYTES:
            return None
        elif argument in ("-A", "--all"):
            adds_everything = True
    if not paths and adds_everything:
        paths = ["."]
    return change_directories, paths

def python_code_arguments(arguments, heredoc_bodies):
    """python 的参数：交回它要跑的代码（`-c` 的那一段；没给脚本、脚本是 `-` 或 /dev/fd/N、/dev/stdin 时是喂给它的 heredoc 正文）。
    `-m` 起模块、给了脚本文件的，交 []（脚本文件里的写法这里判不到）。"""
    position = 0
    while position < len(arguments):
        argument = arguments[position]
        if argument == "-c":
            return arguments[position + 1:position + 2]
        if argument == "-m":
            return []
        if argument in PYTHON_OPTIONS_WITH_VALUE:
            position += 2
            continue
        if argument.startswith("-") and argument != "-":
            position += 1
            continue
        if argument != "-" and not argument.startswith(PYTHON_STANDARD_INPUT_SCRIPTS):
            return []
        break
    return [body.text for body in heredoc_bodies]

def truncate_files(arguments):
    """truncate 的参数：交回它改的每个文件（-s / -r / -o 带值的选项跳过）。"""
    files, position, options_ended = [], 0, False
    while position < len(arguments):
        argument = arguments[position]
        position += 1
        if options_ended or not argument.startswith("-") or argument == "-":
            files.append(argument)
        elif argument == "--":
            options_ended = True
        elif argument in ("--size", "--reference"):
            position += 1
        elif not argument.startswith("--"):
            letters = argument[1:]
            value_letter = next((index for index, letter in enumerate(letters) if letter in "sr"), None)
            if value_letter is not None and value_letter == len(letters) - 1:
                position += 1
    return files

def follow_simple_command(words, directory, variables, steps, depth, heredoc_bodies=()):
    """一条简单命令：会整份覆盖文件的写法（tee、cp、mv、install；⑦ 另认 dd、truncate 与喂给 python 的代码）与让旧字节留住的一步
    （git add、mv 挪走源；⑦ 另认 rm）追加进 steps；跟着 cd / pushd 换目录，跟着独立的赋值与 export 记变量（改 variables）；
    bash -c 的那段代码按这一刻的目录与变量递归进去。heredoc_bodies 是喂给这一条的、被当数据剥掉的 heredoc 正文。交回这条之后的当前目录。"""
    skipped = shell_words.skip_prefixes(words)
    if skipped.command_position is None:
        for name, value in skipped.assignments.items():
            variables[name] = substitute_assigned_variables(value, variables)
        return directory
    name, arguments = None, []
    for _, _, name, arguments in wrapped_commands(words):
        pass
    if name is None:
        return directory
    if shell_words.PYTHON_NAME.match(name):
        steps += [PythonCode(code, directory) for code in python_code_arguments(arguments, heredoc_bodies)]
        return directory
    arguments = [substitute_assigned_variables(argument, variables) for argument in arguments]
    if name == "export":
        for argument in arguments:
            assignment = shell_words.ASSIGNMENT.match(argument)
            if assignment:
                variables[assignment.group(1)] = assignment.group(2)
    elif name in ("cd", "pushd"):
        return shell_words.changed_directory(directory, arguments)
    elif name == "popd":
        return None
    elif name == "tee":
        steps += [OverwriteCandidate("tee", file, directory) for file in tee_files(arguments)]
    elif name in COPY_SHORT_OPTIONS_WITH_VALUE:
        parsed = parse_copy_arguments(name, arguments)
        if parsed.creates_directories:
            return directory
        sources = tuple(parsed.operands if parsed.target_directory is not None else parsed.operands[:-1])
        if not parsed.keeps_existing_target:
            if parsed.target_directory is not None:
                steps.append(OverwriteCandidate(name, parsed.target_directory, directory, sources, True, parsed.replaces_target_inode))
            elif len(parsed.operands) >= 2:
                steps.append(OverwriteCandidate(name, parsed.operands[-1], directory, sources, False if parsed.treats_target_as_file else None,
                                                parsed.replaces_target_inode))
        if name == "mv" and not parsed.keeps_existing_target and sources:
            steps.append(PreservingStep(sources, directory, "mv"))
    elif name == "dd":
        steps += [OverwriteCandidate("dd", argument[len("of="):], directory) for argument in arguments if argument.startswith("of=")]
    elif name == "truncate":
        steps += [OverwriteCandidate("truncate", file, directory) for file in truncate_files(arguments)]
    elif name == "rm":
        removed = tuple(argument for argument in arguments if not argument.startswith("-"))
        if removed:
            steps.append(PreservingStep(removed, directory, "rm"))
    elif name == "git":
        added = git_added_paths(arguments)
        if added is not None:
            git_directory = directory
            for change in added[0]:
                git_directory = shell_words.changed_directory(git_directory, [change])
            steps.append(PreservingStep(tuple(added[1]), git_directory))
    elif name in shell_words.SHELL_NAMES:
        code = shell_words.shell_invocation(arguments).code_string
        if code is not None:
            steps += overwrite_steps(code, directory, variables, depth + 1)
    return directory

def command_pieces(tokens):
    """一层的记号按命令分隔符切成一段段：一段至多一条简单命令，连同它的重定向与替换正文；只有重定向、没有词的也成一段
    （`(…) > 文件`、`{ …; } > 文件` 右括号之后那段，共用模块的 simple_commands 不交它）。数组赋值 `名字=(…)` 的括号留在段里。"""
    pieces, piece, position = [], [], 0
    while position < len(tokens):
        is_operator, text = tokens[position]
        if (is_operator is False and shell_words.ARRAY_ASSIGNMENT_START.match(text)
                and tokens[position + 1:position + 2] == [(True, "(")]):
            end = position + 2
            while end < len(tokens) and tokens[end] != (True, ")"):
                end += 1
            piece += tokens[position:end + 1]
            position = end + 1
            continue
        if is_operator and text in shell_words.COMMAND_SEPARATORS:
            if piece:
                pieces.append(piece)
            piece = []
        else:
            piece.append(tokens[position])
        position += 1
    if piece:
        pieces.append(piece)
    return pieces

def overwrite_steps(text, directory, variables=None, depth=0):
    """整条命令每一层（顶层、bash -c 的代码、喂给 shell 的 heredoc 正文、命令替换与进程替换）命令位置上，会整份覆盖文件的写法
    （OverwriteCandidate：重定向、tee、cp、mv、install）与让旧字节留住的一步（PreservingStep），按执行的先后交回。
    喂给 shell 的 heredoc 正文留在原位按行切（共用模块的 strip_data_heredocs 只剥喂给别的命令的）；directory 是开头的当前目录。"""
    if depth > shell_words.MAXIMUM_NESTING:
        return []
    variables = dict(variables or {})
    steps = []
    data_heredocs = []
    for piece in command_pieces(shell_words.shell_tokens(shell_words.strip_data_heredocs(text, stripped_bodies=data_heredocs))):
        for is_operator, substitution in piece:
            if is_operator is None:
                steps += overwrite_steps(substitution, directory, variables, depth + 1)
        for (is_operator, operator), (next_is_operator, target) in zip(piece, piece[1:]):
            if (is_operator and operator in OVERWRITING_REDIRECTS and next_is_operator is False
                    and not (operator == ">&" and FILE_DESCRIPTOR_WORD.match(target))):
                steps.append(OverwriteCandidate(operator, substitute_assigned_variables(target, variables), directory))
        commands, _, _, redirections = shell_words.simple_commands(piece)
        if commands and commands[0]:
            fed_heredocs = [data_heredocs[int(marker.group(1))] for operator, target in redirections[0] if operator in ("<<", "<<-")
                            for marker in [shell_words.HEREDOC_BODY_MARKER.match(target)] if marker and int(marker.group(1)) < len(data_heredocs)]
            directory = follow_simple_command(commands[0], directory, variables, steps, depth, fed_heredocs)
    return steps

def target_pattern(word):
    """目标词里这一刻算不出的段（没赋过值的变量、命令替换、花括号展开）换成 *：交回 (样式, 有没有算不出的段, 有没有通配)。"""
    pieces, position, has_unknown_piece, has_glob = [], 0, False, False
    while position < len(word):
        if word.startswith("$(", position) or word[position] == "`":
            position = shell_words.substitution_span(word, position)[0]
            pieces.append("*")
            has_unknown_piece = True
            continue
        unknown = UNKNOWN_PIECE.match(word, position)
        if unknown:
            pieces.append("*")
            has_unknown_piece = True
            position = unknown.end()
            continue
        has_glob = has_glob or word[position] in GLOB_CHARACTERS
        pieces.append(word[position])
        position += 1
    return "".join(pieces), has_unknown_piece, has_glob

def is_inside(path, directory):
    """path 在 directory 里面（不含 directory 自己）；按字面比、解开符号链接之后比，有一边在里面就算。"""
    return any(os.path.normpath(candidate).startswith(os.path.normpath(parent) + os.sep)
               for candidate, parent in ((path, directory), (os.path.realpath(path), os.path.realpath(directory))))

def destination_words(candidate, base):
    """这一步写到的目标词：重定向与 tee 是目标本身；cp / mv / install 写进目录时是 目录/每个源的文件名（源里的通配按这一刻的文件展开，
    一个都展不开的 cp 会报错、什么都不写；源里有算不出的段，文件名换成 *）。"""
    into_directory = candidate.into_directory
    if into_directory is None:
        pattern, has_unknown_piece, has_glob = target_pattern(os.path.expanduser(candidate.target))
        into_directory = candidate.target.endswith("/") or (
            not has_unknown_piece and not has_glob and os.path.isdir(shell_words.resolve_path(base, pattern)))
    if not into_directory:
        return [candidate.target]
    names = []
    for source in candidate.sources:
        pattern, has_unknown_piece, has_glob = target_pattern(os.path.expanduser(source))
        if has_unknown_piece:
            names.append("*")
        elif has_glob:
            names += [os.path.basename(match.rstrip("/")) for match in sorted(glob.glob(shell_words.resolve_path(base, pattern)))]
        else:
            names.append(os.path.basename(source.rstrip("/")))
    return [f"{candidate.target.rstrip('/')}/{name}" for name in names]

def resolve_destination(word, base, repository_root):
    """目标词按 base 解开：交回 (确定的绝对路径, [])；有算不出的段或通配时交回 (None, 样式在 research/results/ 下对得上的已有文件)，
    样式落不进 research/results/ 的交 (None, [])。`"$变量"/research/results/…` 这类变量在前的，从 research/results/ 那一段起按仓库根接。"""
    results_directory = os.path.join(repository_root, *RESULTS_DIRECTORY_WORDS)
    pattern, has_unknown_piece, has_glob = target_pattern(os.path.expanduser(word))
    path = shell_words.resolve_path(base, pattern)
    if not has_unknown_piece and not has_glob:
        return path, []
    anchor = pattern.find("/".join(RESULTS_DIRECTORY_WORDS) + "/")
    if anchor > 0 and "*" in pattern[:anchor] and not is_inside(path, results_directory):
        path = os.path.join(repository_root, pattern[anchor:])
    if not is_inside(path, results_directory):
        return None, []
    return None, sorted(match for match in glob.glob(path) if os.path.isfile(match))

def preserved_paths(step, base):
    """PreservingStep 里的路径按 base 解开（通配按这一刻的文件展开，有算不出的段的不算）。"""
    paths = []
    for word in step.words:
        pattern, has_unknown_piece, has_glob = target_pattern(os.path.expanduser(word))
        if has_unknown_piece:
            continue
        resolved = shell_words.resolve_path(base, pattern)
        paths += glob.glob(resolved) if has_glob else [resolved]
    return [os.path.normpath(path) for path in paths]

def is_preserved(path, preserved):
    """path 是同一条命令里前面 git add 过、或 mv 挪走了的那个文件（或在那样一个目录里面）。"""
    return any(os.path.normpath(path) == kept or is_inside(path, kept) for kept in preserved)

def git_tracks(repository_root, path):
    """`git ls-files --error-unmatch`：进过 git 交 True，没进交 False，git 自己出错（不是仓、路径在仓外、超时）交 None。"""
    try:
        completed = subprocess.run(["git", "-C", repository_root, "ls-files", "--error-unmatch", "--", path],
                                   capture_output=True, text=True, timeout=20)
    except (OSError, subprocess.TimeoutExpired):
        return None
    return {0: True, 1: False}.get(completed.returncode)

def untracked_among(repository_root, paths):
    """几份已有的文件里没进 git 的，一次 git ls-files 查完；git 自己出错交 None。"""
    if not paths:
        return []
    try:
        completed = subprocess.run(["git", "-C", repository_root, "ls-files", "-z", "--full-name", "--", *paths],
                                   capture_output=True, text=True, timeout=20)
    except (OSError, subprocess.TimeoutExpired):
        return None
    if completed.returncode != 0:
        return None
    top = os.path.realpath(repository_root)
    tracked = {os.path.normpath(os.path.join(top, name)) for name in completed.stdout.split("\0") if name}
    return [path for path in paths if os.path.normpath(os.path.realpath(path)) not in tracked]

def results_overwrite_verdict(command, repository_root):
    """整条命令里会整份覆盖 research/results/ 下文件的每一步：目标确定、已存在、没进 git、前面也没被 git add 或 mv 挪走的，交进要拒的
    （`写法 相对仓库根的路径`）；目标这一刻算不出、样式对得上已有的未跟踪文件的，与 git 判不了的，交进要记检出的。"""
    results_directory = os.path.join(repository_root, *RESULTS_DIRECTORY_WORDS)
    preserved, refused, noted = [], [], []
    for step in overwrite_steps(command, repository_root):
        base = step.directory if step.directory is not None else repository_root
        if isinstance(step, PreservingStep):
            if step.kind in RESULTS_PRESERVING_KINDS:
                preserved += preserved_paths(step, base)
            continue
        if isinstance(step, PythonCode) or step.form in IN_PLACE_ONLY_FORMS:
            continue
        for word in destination_words(step, base):
            exact, matches = resolve_destination(word, base, repository_root)
            if exact is not None:
                if not is_inside(exact, results_directory) or not os.path.isfile(exact) or is_preserved(exact, preserved):
                    continue
                tracked = git_tracks(repository_root, exact)
                relative = os.path.relpath(exact, repository_root)
                if tracked is False:
                    refused.append(f"{step.form} {relative}")
                elif tracked is None:
                    noted.append(f"判不了 {relative} 进没进 git（git ls-files 出错），{step.form} 覆盖它这一步没拦")
                continue
            untracked = untracked_among(repository_root, [match for match in matches if not is_preserved(match, preserved)])
            if untracked is None:
                noted.append(f"判不了 {step.form} {word} 对得上的文件进没进 git（git ls-files 出错），没拦")
            elif untracked:
                shown = "、".join(os.path.relpath(path, repository_root) for path in untracked[:3])
                noted.append(f"{step.form} {word} 的目标这一刻算不出来（变量、命令替换或通配），没拦；"
                             f"它对得上 research/results/ 下 {len(untracked)} 份已有的未跟踪文件：{shown}{' 等' if len(untracked) > 3 else ''}")
    return list(dict.fromkeys(refused)), list(dict.fromkeys(noted))

def results_overwrite_refusal(command, repository_root):
    """交回 (要拒的写法与目标, 要记的检出)；前台、run_in_background 一样判，主 agent 与子 agent 都判。"""
    if (os.environ.get("BASH_COMMAND_DETECTOR_DISABLE_CHECK") == "1"
            or os.environ.get("BASH_COMMAND_DETECTOR_ALLOW_RESULTS_OVERWRITE") == "1"):
        return [], []
    return results_overwrite_verdict(command, repository_root)

def repository_root_of(hook_dir):
    """这个 hook 所在的仓：`.claude/hooks/` 往上两级。"""
    return os.path.realpath(os.path.join(hook_dir, os.pardir, os.pardir))

# ⑦ 在同一个 inode 上改一个已经存在的脚本：shell 那一半沿用 ⑤ 的 overwrite_steps（同一份写法、cd 与变量），另判喂给 python 的代码
SCRIPT_SUFFIXES = (".sh", ".py")
SCRIPT_SCRATCH_ROOT = "/tmp/claude-1000"
INODE_REPLACING_FORMS = {"mv", "install"}               # 实测换 inode：改之前打开文件的读者接着读到旧内容
PYTHON_OPTIONS_WITH_VALUE = {"-W", "-X", "--check-hash-based-pycs"}
PYTHON_STANDARD_INPUT_SCRIPTS = ("/dev/fd/", "/dev/stdin")
PYTHON_OPEN_MODULES = {"io", "builtins", "codecs"}      # 这几个模块的 open 与内建 open 同一个签名：(文件, 模式)
PYTHON_PATH_CONSTRUCTORS = {"Path", "PosixPath", "PurePath", "PurePosixPath"}
PYTHON_PATH_WRITERS = {"write_text", "write_bytes"}
PYTHON_FILE_COPIERS = {"copyfile", "copy", "copy2"}      # shutil 的这三个用 'wb' 打开目标，实测在原 inode 上写
PYTHON_PATH_MAXIMUM_DEPTH = 8

class PythonWrite(NamedTuple):
    form: str            # 认出的写法，报给人看
    path: str            # 目标路径（从字面量算出来的）
    source: str | None   # shutil.copy / copy2 的源：目标是已有目录时写的是 目标/源的文件名

def called_name_and_owner(function):
    """ast 调用对象的 (名字, 属主)：`open` → ("open", None)，`io.open` → ("open", "io")，`os.path.join` → ("join", "path")，属主不是名字或属性时为 None。"""
    if isinstance(function, ast.Name):
        return function.id, None
    if isinstance(function, ast.Attribute):
        receiver = function.value
        owner = receiver.id if isinstance(receiver, ast.Name) else receiver.attr if isinstance(receiver, ast.Attribute) else None
        return function.attr, owner
    return None, None

def single_assignments(tree):
    """整段代码里只绑定过一次、而且是 `名字 = 值` 这一种绑法的名字：交回 {名字: 值的节点}。for、with … as、函数参数、import、
    增量赋值与海象这些别的绑法都算一次绑定，绑过不止一次的不认（运行期拿到的是哪个值，这里说不准）。"""
    bindings, values = {}, {}
    for node in ast.walk(tree):
        bound = []
        if isinstance(node, ast.Name) and isinstance(node.ctx, (ast.Store, ast.Del)):
            bound = [node.id]
        elif isinstance(node, ast.arg):
            bound = [node.arg]
        elif isinstance(node, (ast.Import, ast.ImportFrom)):
            bound = [(alias.asname or alias.name).split(".")[0] for alias in node.names]
        for name in bound:
            bindings[name] = bindings.get(name, 0) + 1
        if isinstance(node, ast.Assign) and len(node.targets) == 1 and isinstance(node.targets[0], ast.Name):
            values[node.targets[0].id] = node.value
    return {name: value for name, value in values.items() if bindings.get(name) == 1}

def python_literal_path(node, assignments, depth=0):
    """路径表达式在这一刻算得出的值：字符串字面量、只赋过一次的名字、Path(…)、Path(…) / '…'、os.path.join(…)；算不出交 None。"""
    if node is None or depth > PYTHON_PATH_MAXIMUM_DEPTH:
        return None
    if isinstance(node, ast.Constant) and isinstance(node.value, str):
        return node.value
    if isinstance(node, ast.Name) and node.id in assignments:
        return python_literal_path(assignments[node.id], assignments, depth + 1)
    if isinstance(node, ast.Call) and node.args and not node.keywords:
        name, owner = called_name_and_owner(node.func)
        if name in PYTHON_PATH_CONSTRUCTORS or (name == "join" and owner == "path"):
            parts = [python_literal_path(argument, assignments, depth + 1) for argument in node.args]
            return None if None in parts else os.path.join(*parts)
    if isinstance(node, ast.BinOp) and isinstance(node.op, ast.Div):
        left, right = python_literal_path(node.left, assignments, depth + 1), python_literal_path(node.right, assignments, depth + 1)
        return None if left is None or right is None else os.path.join(left, right)
    return None

def call_argument(node, position, keyword):
    """调用的第 position 个位置参数，没有就找名为 keyword 的关键字参数；都没有交 None。"""
    if len(node.args) > position:
        return node.args[position]
    return next((item.value for item in node.keywords if item.arg == keyword), None)

def opening_mode(node, default="r"):
    """open 的模式：没给是 default，给的是字符串字面量交它，给的是别的表达式（算不出）交 None。"""
    if node is None:
        return default
    return node.value if isinstance(node, ast.Constant) and isinstance(node.value, str) else None

def mode_writes_in_place(mode):
    """'w' 截断重写、'r+' 从头改写，都在原 inode 上；'a' 追加、'x' 只建新文件，不算。"""
    return mode is not None and ("w" in mode or ("r" in mode and "+" in mode))

def python_in_place_writes(code):
    """python 代码里在原 inode 上写文件的调用：open(路径, 带 w 或 r+ 的模式)、Path(路径).open(同上)、Path(路径).write_text / write_bytes、
    shutil.copyfile / copy / copy2 的目标。路径与模式算不出的不判，解析不了的代码不判。"""
    try:
        tree = ast.parse(code)
    except (SyntaxError, ValueError):
        return []
    assignments = single_assignments(tree)
    writes = []
    for node in ast.walk(tree):
        if not isinstance(node, ast.Call):
            continue
        name, owner = called_name_and_owner(node.func)
        receiver = node.func.value if isinstance(node.func, ast.Attribute) else None
        if name == "open" and (receiver is None or (isinstance(receiver, ast.Name) and receiver.id in PYTHON_OPEN_MODULES)):
            mode = opening_mode(call_argument(node, 1, "mode"))
            path = python_literal_path(call_argument(node, 0, "file"), assignments)
            if path is not None and mode_writes_in_place(mode):
                writes.append(PythonWrite(f"python open(…, '{mode}')", path, None))
        elif name == "open":
            mode = opening_mode(call_argument(node, 0, "mode"))
            path = python_literal_path(receiver, assignments)
            if path is not None and mode_writes_in_place(mode):
                writes.append(PythonWrite(f"python Path.open('{mode}')", path, None))
        elif name in PYTHON_PATH_WRITERS and receiver is not None:
            path = python_literal_path(receiver, assignments)
            if path is not None:
                writes.append(PythonWrite(f"python Path.{name}", path, None))
        elif name in PYTHON_FILE_COPIERS and owner == "shutil":
            path = python_literal_path(call_argument(node, 1, "dst"), assignments)
            source = python_literal_path(call_argument(node, 0, "src"), assignments) if name != "copyfile" else None
            if path is not None:
                writes.append(PythonWrite(f"python shutil.{name}", path, source))
    return writes

def is_existing_script(path, repository_root, scratch_root):
    """path 是仓里或 scratch_root 下一个已经存在的脚本：.sh、.py，或带执行位的普通文件（符号链接按它指向的文件算）。"""
    if not (is_inside(path, repository_root) or is_inside(path, scratch_root)):
        return False
    real = os.path.realpath(path)
    if not os.path.isfile(real):
        return False
    return path.endswith(SCRIPT_SUFFIXES) or real.endswith(SCRIPT_SUFFIXES) or bool(os.stat(real).st_mode & 0o111)

def script_in_place_verdict(command, repository_root, scratch_root=SCRIPT_SCRATCH_ROOT):
    """整条命令里在原 inode 上写一个已有脚本的每一步，交回 `写法 路径`（仓里的写相对仓库根的路径）。目标这一刻算不出（没赋过值的变量、
    命令替换、通配）的不判；同一条命令里排在前面被 mv 挪走或 rm 掉的那个名字，之后写的是一个新 inode，不算。"""
    replaced_names, refused = [], []
    for step in overwrite_steps(command, repository_root):
        base = step.directory if step.directory is not None else repository_root
        if isinstance(step, PreservingStep):
            if step.kind in ("mv", "rm"):
                replaced_names += preserved_paths(step, base)
            continue
        targets = []
        if isinstance(step, PythonCode):
            for write in python_in_place_writes(step.code):
                path = shell_words.resolve_path(base, write.path)
                if write.source is not None and os.path.isdir(path):
                    path = os.path.join(path, os.path.basename(write.source))
                targets.append((write.form, path))
        elif step.form not in INODE_REPLACING_FORMS and not step.replaces_inode:
            for word in destination_words(step, base):
                pattern, has_unknown_piece, has_glob = target_pattern(os.path.expanduser(word))
                if not has_unknown_piece and not has_glob:
                    targets.append((step.form, shell_words.resolve_path(base, pattern)))
        for form, path in targets:
            if not is_preserved(path, replaced_names) and is_existing_script(path, repository_root, scratch_root):
                refused.append(f"{form} {os.path.relpath(path, repository_root) if is_inside(path, repository_root) else path}")
    return list(dict.fromkeys(refused))

def script_in_place_refusal(command, repository_root, scratch_root=SCRIPT_SCRATCH_ROOT):
    """交回要拒的写法与目标；前台、run_in_background 一样判，主 agent 与子 agent 都判。"""
    if (os.environ.get("BASH_COMMAND_DETECTOR_DISABLE_CHECK") == "1"
            or os.environ.get("BASH_COMMAND_DETECTOR_ALLOW_SCRIPT_IN_PLACE_WRITE") == "1"):
        return []
    return script_in_place_verdict(command, repository_root, scratch_root)

# ⑥ 终止进程只许点名一个自己起的进程号或任务号；同一份判定也交给门禁 73 号扫脚本（--scan-scripts），hook 与门禁不各写一份
BY_NAME_KILLERS = {"pkill", "killall", "killall5", "skill"}           # 按名字挑进程：一次命中一批，谁起的都算
FAN_OUT_RUNNERS = {"xargs", "parallel"}                                # 把一串进程号逐个喂给后面那条命令
FIND_EXECUTING_OPTIONS = {"-exec", "-execdir", "-ok", "-okdir"}
TERMINATING_NAMES = {"kill", "proc.py"} | BY_NAME_KILLERS
MACHINE_STOPPING_COMMANDS = {"reboot", "poweroff", "halt", "shutdown", "telinit"}
SYSTEMCTL_STOPPING_VERBS = {"stop", "kill", "restart", "try-restart", "reload-or-restart", "try-reload-or-restart",
                            "condrestart", "force-reload", "freeze", "set-property", "clean"}
SYSTEMCTL_STOPPING_WITH_NOW = {"disable", "mask"}                      # 带 --now 时顺手把单元停掉
SYSTEMCTL_MACHINE_VERBS = {"isolate", "poweroff", "reboot", "halt", "kexec", "suspend", "hibernate", "hybrid-sleep",
                           "suspend-then-hibernate", "rescue", "emergency", "default", "soft-reboot", "exit", "switch-root"}
SYSTEMCTL_OPTIONS_WITH_VALUE = {"-t", "--type", "-s", "--signal", "-p", "--property", "-P", "-H", "--host", "-M", "--machine",
                                "-n", "--lines", "-o", "--output", "--kill-whom", "--kill-value", "--job-mode", "--root", "--image",
                                "--state", "--what", "--timestamp", "--message", "--reboot-argument", "--kill-mode", "--drop-in", "--when"}
# SSH 会话、登录会话、用户级 systemd、本地模型服务与它们依赖的系统服务；systemctl 不写后缀的单元按 .service 补上再比
PROTECTED_UNIT = re.compile(r"^(?:sshd?(?:@[^/]*)?(?:\.service|\.socket)|session-[^/]*\.scope|user@[^/]*\.service|user(?:-[^/]*)?\.slice"
                            r"|-\.slice|init\.scope|vllm-prod\.service|systemd-logind\.service|dbus(?:-broker)?\.(?:service|socket))$")
LOGINCTL_TERMINATING_VERB = re.compile(r"^(?:terminate|kill)-")
CGROUP_CONTROL_FILES = {"cgroup.kill", "cgroup.freeze"}               # 写进去一次停掉（冻住）那个 cgroup 里的全部进程
PROCESS_LIST_READING = re.compile(r"cgroup\.(?:procs|threads)|/proc/(?:\*|\[)|\b(?:ls|find)\s+/proc(?:/|\s|$)")
PROCESS_PICKERS = {"pgrep", "pidof"}
PROTECTED_PROCESS_NAMES = {"systemd", "init", "sshd", "sshd-session", "sshd-auth", "dbus-daemon", "dbus-broker", "systemd-logind", "claude"}
EXPANSION = re.compile(r"\$\{[^}]*\}")
WHOLE_LIST_EXPANSION = re.compile(r"\$\{?[@*]|\$\{[^}]*\[[@*]\][^}]*\}")
BRACE_LIST = re.compile(r"\{[^{}]*(?:,|\.\.)[^{}]*\}")
LITERAL_PROCESS_ID = re.compile(r"^\d+$")
SIGNAL_ZERO_NAMES = {"0", "SIG0"}

class TerminationFinding(NamedTuple):
    words: tuple              # 认出它的那条简单命令（引号去掉）；python 代码里的是空
    problem: str              # 给人看的一句：哪种写法、为什么打得到别人
    kind: str                 # "loop"（循环里逐个发）、"cgroup"（按 cgroup / 进程表挑）、"other"；脚本里的 own-scope 标注只放行前两类
    python_code: str = ""     # python 代码里认出的：那段代码（脚本扫描按它在文件里的位置换算行号）
    python_line: int = 0      # python 代码里认出的：它在那段代码里的行号

def innermost_call(words):
    """一条简单命令剥开前缀、capped.sh N / run-with-memory-cap.sh <上限>、`bash 包装脚本`、builtin、python 解释器之后：交回 (名字, 参数)。
    python -c 那一条交回 python 本身与全部参数，代码由 python 那一侧判。"""
    name, arguments = None, []
    for _, _, name, arguments in wrapped_commands(words):
        pass
    for _ in range(shell_words.MAXIMUM_NESTING):
        if name in ("builtin", "command") and arguments:
            name, arguments = os.path.basename(arguments[0]), arguments[1:]
            continue
        if name and shell_words.PYTHON_NAME.match(name):
            index = 0
            while index < len(arguments) and arguments[index].startswith("-") and arguments[index] != "-":
                if arguments[index] == "-c":
                    return name, arguments
                index += 2 if arguments[index] in ("-m", "-W", "-X") else 1
            if index < len(arguments) and arguments[index] != "-":
                name, arguments = os.path.basename(arguments[index]), arguments[index + 1:]
                continue
        break
    return name, arguments

def kill_arguments(arguments):
    """kill 的参数：交回 (信号, 目标)；只列信号（-l、-L）时交 (None, None)。信号给过之后再出现的 -N 是负的进程号（进程组）。"""
    signal_name, targets, index, options_done = None, [], 0, False
    while index < len(arguments):
        argument = arguments[index]
        if options_done or not argument.startswith("-") or argument == "-":
            targets.append(argument)
        elif argument == "--":
            options_done = True
        elif argument in ("-l", "-L", "--list", "--table"):
            return None, None
        elif argument in ("-s", "-n", "--signal"):
            signal_name = arguments[index + 1] if index + 1 < len(arguments) else ""
            index += 1
        elif argument.startswith("--signal="):
            signal_name = argument.split("=", 1)[1]
        elif argument in ("-q", "--queue"):
            index += 1
        elif argument == "--timeout":
            index += 2
        elif signal_name is None:
            signal_name = argument[1:]
        else:
            targets.append(argument)
        index += 1
    return signal_name, targets

def target_problem(target):
    """一个 kill / proc.py stop 的目标打得到不止一个进程、或打到谁这一刻说不清，就交回原因；点名一个的交 None。"""
    if target.startswith("-") and target != "-":
        return f"目标 {target} 带负号，是整个进程组（-1 是能发到的全部进程）"
    if target == "0":
        return "目标 0 是发这条命令的整个进程组"
    if target in ("$PPID", "${PPID}"):
        return "目标 $PPID 是起这条命令的进程（在会话里就是 Claude 会话本身）"
    if "$(" in target or "`" in target or "<(" in target:
        return f"目标 {target} 是命令替换：展开出几个进程、是谁起的，这一刻都不知道"
    if WHOLE_LIST_EXPANSION.search(target):
        return f"目标 {target} 是数组或位置参数的整批展开"
    bare = EXPANSION.sub("", target)
    if any(character in bare for character in "*?[") or BRACE_LIST.search(bare):
        return f"目标 {target} 里有通配或花括号展开"
    return None

def read_process_table():
    """/proc 下每个进程的父进程号、名字与命令行（只读，不发信号）：判写死的进程号打到的是谁。"""
    table = {}
    for entry in os.listdir("/proc"):
        if not entry.isdigit():
            continue
        try:
            with open(f"/proc/{entry}/stat", encoding="utf-8", errors="replace") as handle:
                status = handle.read()
            with open(f"/proc/{entry}/cmdline", "rb") as handle:
                command_line = handle.read().replace(b"\0", b" ").decode("utf-8", "replace").strip()
            table[int(entry)] = {"parent": int(status.rsplit(")", 1)[1].split()[1]),
                                 "name": status[status.index("(") + 1:status.rindex(")")], "command_line": command_line}
        except (OSError, ValueError, IndexError):
            continue
    return table

def ancestors_in(table, process_id):
    """process_id 与它在表里的全部祖先。"""
    found, current = [], process_id
    while current and current in table and current not in found:
        found.append(current)
        current = table[current]["parent"]
    return found

def protected_process_reason(process_id, table, own_ancestors):
    """写死的进程号打到的是 SSH、VSCode、Claude 会话、本地模型服务、系统服务，或别的 Claude 会话起的进程，就交回原因。不在表里的不拦（kill 自己会报错）。"""
    information = table.get(process_id)
    if information is None:
        return None
    label = f"进程 {process_id}（{information['name']}）"
    if process_id in own_ancestors:
        return f"{label}是发这条命令的会话自己的祖先（Claude 会话、VSCode 扩展宿主或更上层）"
    if process_id == 1 or information["name"] in PROTECTED_PROCESS_NAMES:
        return f"{label}是系统、SSH 或 Claude 会话的进程"
    if ".vscode-server" in information["command_line"]:
        return f"{label}是 VSCode 服务端里的进程（服务端、扩展宿主、ptyHost 或经它起的 Claude）"
    if "vllm" in information["command_line"]:
        return f"{label}是本地模型服务 vllm 的进程"
    own_sessions = {ancestor for ancestor in own_ancestors if table.get(ancestor, {}).get("name") == "claude"}
    for ancestor in ancestors_in(table, information["parent"]):
        if table[ancestor]["name"] == "claude" and ancestor not in own_sessions:
            return f"{label}是另一个 Claude 会话（进程 {ancestor}）起的"
    return None

def process_target_findings(words, targets, in_loop, what, process_table, own_ancestors):
    """kill 与 proc.py stop 共用：目标个数、每个目标的形状、写死的进程号打到谁、是不是在循环里。"""
    findings = []
    if len(targets) > 1:
        findings.append(TerminationFinding(tuple(words), f"{what} 一次给了 {len(targets)} 个目标（{' '.join(targets)}）：一次只停点名的一个", "other"))
    for target in targets:
        problem = target_problem(target)
        if problem:
            kind = "cgroup" if PROCESS_LIST_READING.search(target) or any(picker in target for picker in PROCESS_PICKERS) else "other"
            findings.append(TerminationFinding(tuple(words), f"{what}：{problem}", kind))
        elif process_table is not None and LITERAL_PROCESS_ID.match(target):
            reason = protected_process_reason(int(target), process_table, own_ancestors)
            if reason:
                findings.append(TerminationFinding(tuple(words), f"{what}：{reason}", "other"))
    if in_loop:
        findings.append(TerminationFinding(tuple(words), f"{what} 在 for / while / until 循环里逐个发：这是批量停，不是点名停一个", "loop"))
    return findings

def systemctl_findings(words, arguments):
    options, positional, index, options_done = [], [], 0, False
    while index < len(arguments):
        argument = arguments[index]
        if not options_done and argument == "--":
            options_done = True
        elif not options_done and argument.startswith("-") and argument != "-":
            options.append(argument)
            index += 1 if argument in SYSTEMCTL_OPTIONS_WITH_VALUE else 0
        else:
            positional.append(argument)
        index += 1
    if not positional:
        return []
    verb, units = positional[0], positional[1:]
    if verb in SYSTEMCTL_MACHINE_VERBS:
        return [TerminationFinding(tuple(words), f"systemctl {verb} 停的是整台机器或整个服务管理器上的东西（SSH 会话、VSCode、Claude 会话都在里面）", "other")]
    if not (verb in SYSTEMCTL_STOPPING_VERBS or (verb in SYSTEMCTL_STOPPING_WITH_NOW and "--now" in options)):
        return []
    findings = []
    for unit in units[:1] if verb == "set-property" else units:
        bare = EXPANSION.sub("", unit)
        named = unit if "." in os.path.basename(unit) else unit + ".service"
        if "$(" in unit or "`" in unit or WHOLE_LIST_EXPANSION.search(unit):
            findings.append(TerminationFinding(tuple(words), f"systemctl {verb} 的单元 {unit} 是命令替换或整批展开：一次停一批，是谁的这一刻不知道", "other"))
        elif any(character in bare for character in "*?[") or BRACE_LIST.search(bare):
            findings.append(TerminationFinding(tuple(words), f"systemctl {verb} 的单元 {unit} 带通配：一次命中一批单元，别的会话的也算", "other"))
        elif PROTECTED_UNIT.match(named):
            findings.append(TerminationFinding(tuple(words), f"systemctl {verb} {unit}：这是 SSH、登录会话、用户级 systemd、本地模型服务或它们依赖的系统服务", "other"))
    return findings

def call_findings(words, in_loop, process_table, own_ancestors):
    """一条简单命令（剥开包装之后）里的终止写法；python -c 的代码另判。交回 (findings, 这一条是不是在给进程发终止信号)。"""
    name, arguments = innermost_call(words)
    if not name:
        return [], False
    if name == "kill":
        signal_name, targets = kill_arguments(arguments)
        if targets is None or signal_name in SIGNAL_ZERO_NAMES or not targets:
            return [], False
        return process_target_findings(words, targets, in_loop, "kill", process_table, own_ancestors), True
    if name == "proc.py" and arguments[:1] == ["stop"]:
        targets, index = [], 1
        while index < len(arguments):
            if arguments[index] == "--grace":
                index += 1
            elif not arguments[index].startswith("--grace="):
                targets.append(arguments[index])
            index += 1
        return process_target_findings(words, targets, in_loop, "proc.py stop", process_table, own_ancestors), True
    if name in BY_NAME_KILLERS:
        return [TerminationFinding(tuple(words), f"{name} 按名字挑进程发信号：一次命中一批，别的会话起的同名进程也算", "other")], True
    if name in FAN_OUT_RUNNERS and any(os.path.basename(argument) in TERMINATING_NAMES for argument in arguments):
        return [TerminationFinding(tuple(words), f"{name} 把一串进程号逐个喂给 kill / proc.py：这是批量停", "loop")], True
    if name == "find" and any(argument in FIND_EXECUTING_OPTIONS and os.path.basename(following) in TERMINATING_NAMES
                              for argument, following in zip(arguments, arguments[1:])):
        return [TerminationFinding(tuple(words), "find -exec 把找到的每个进程都停掉：这是批量停", "loop")], True
    if name == "fuser" and any(argument == "--kill" or (argument.startswith("-") and not argument.startswith("--") and "k" in argument[1:])
                               for argument in arguments):
        return [TerminationFinding(tuple(words), "fuser -k 停掉用着那个文件或端口的全部进程，是谁的都算", "other")], True
    if name == "systemctl":
        return systemctl_findings(words, arguments), False
    if name == "loginctl" and any(LOGINCTL_TERMINATING_VERB.match(argument) for argument in arguments if not argument.startswith("-")):
        return [TerminationFinding(tuple(words), "loginctl terminate-* / kill-* 停掉整个登录会话或用户（SSH 会话就在里面）", "other")], False
    if name in MACHINE_STOPPING_COMMANDS and not (name == "shutdown" and "-c" in arguments):
        return [TerminationFinding(tuple(words), f"{name} 停的是整台机器", "other")], False
    if name == "tee" and any(os.path.basename(argument) in CGROUP_CONTROL_FILES for argument in arguments):
        return [TerminationFinding(tuple(words), "往 cgroup.kill / cgroup.freeze 写：一次停掉（冻住）那个 cgroup 里的全部进程", "cgroup")], True
    return [], False

def embedded_code_strings(words):
    """一条简单命令里任何位置上的 `bash -c '…'` 与 `python3 -c '…'`（前面是 shell 函数、变量里的脚本、别的包装也算）：交回 [(种类, 代码)]。"""
    found = []
    for index, word in enumerate(words):
        name, rest = os.path.basename(word), words[index + 1:]
        if name in shell_words.SHELL_NAMES:
            code = shell_words.shell_invocation(rest).code_string
            if code is not None:
                found.append(("shell", code))
        elif shell_words.PYTHON_NAME.match(name) and "-c" in rest:
            position = rest.index("-c")
            if position + 1 < len(rest) and all(item.startswith("-") for item in rest[:position]):
                found.append(("python", rest[position + 1]))
    return found

def python_heredoc_bodies(text):
    """喂给 python 的 heredoc 正文（python3 - <<EOF、python3 /dev/fd/3 3<<'PY' 这类）。"""
    return [heredoc.text for command in shell_words.commands_at_command_position(text).commands
            if command.launcher == "python" or shell_words.PYTHON_NAME.match(os.path.basename(command.words[0]))
            for heredoc in command.standard_input_heredocs]

PYTHON_SIGNAL_CALLS = {"kill", "killpg", "pidfd_send_signal"}
PYTHON_LOOP_NODES = (ast.For, ast.AsyncFor, ast.While, ast.ListComp, ast.SetComp, ast.DictComp, ast.GeneratorExp)

def python_termination_findings(code):
    """python 代码里的终止写法：os.killpg、os.kill 的目标是 0 或负数、在循环或推导式里调、同一个函数里读 cgroup.procs 或 /proc 再调。解析不了的不判。"""
    try:
        tree = ast.parse(code)
    except (SyntaxError, ValueError):
        return []
    findings = []
    def reads_process_list(scope):
        return any(isinstance(node, ast.Constant) and isinstance(node.value, str) and PROCESS_LIST_READING.search(node.value)
                   for node in ast.walk(scope))
    def visit(node, loop_depth, scope_reads):
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef, ast.Lambda)):
            loop_depth, scope_reads = 0, reads_process_list(node)
        if isinstance(node, ast.Call):
            function = node.func
            owner = function.value.id if isinstance(function, ast.Attribute) and isinstance(function.value, ast.Name) else None
            called = function.attr if isinstance(function, ast.Attribute) else function.id if isinstance(function, ast.Name) else None
            probe = len(node.args) >= 2 and isinstance(node.args[1], ast.Constant) and node.args[1].value == 0
            if called in PYTHON_SIGNAL_CALLS and owner in ("os", "signal", None) and not probe:
                first = node.args[0] if node.args else None
                negative = (isinstance(first, ast.UnaryOp) and isinstance(first.op, ast.USub)) or (
                    isinstance(first, ast.Constant) and isinstance(first.value, int) and first.value <= 0)
                if called == "killpg":
                    findings.append(TerminationFinding((), "python 里 os.killpg 发给整个进程组", "other", code, node.lineno))
                elif negative:
                    findings.append(TerminationFinding((), "python 里 os.kill 的目标是 0 或负数（整个进程组或全部进程）", "other", code, node.lineno))
                if loop_depth:
                    findings.append(TerminationFinding((), f"python 里在循环或推导式里调 {called}：这是批量停", "loop", code, node.lineno))
                if scope_reads:
                    findings.append(TerminationFinding((), f"python 里同一个函数先读 cgroup.procs 或 /proc 挑进程、再调 {called}", "cgroup", code, node.lineno))
        for child in ast.iter_child_nodes(node):
            visit(child, loop_depth + (1 if isinstance(node, PYTHON_LOOP_NODES) and child is not getattr(node, "iter", None) else 0), scope_reads)
    visit(tree, 0, reads_process_list(tree))
    return findings

def scan_shell_text(text, process_table, own_ancestors, depth):
    """termination_findings 的本体：交回 (发现, 有没有给进程发终止信号, 有没有 pgrep / pidof 挑进程)。
    包装认不出的 `bash -c '…'`（前面是 shell 函数、变量里的脚本）另按一段完整的代码递归判。"""
    findings, sends_signal, picks_processes, python_codes = [], False, False, []
    for layer in shell_layers(text):
        commands, _, _, redirections = shell_words.simple_commands(layer.tokens)
        loop_depth = 0
        for words, command_redirections in zip(commands, redirections):
            opened, closed, _ = loop_keyword_changes(words) if words else (0, 0, False)
            for operator, target in command_redirections:
                if operator in shell_words.OUTPUT_REDIRECTS and os.path.basename(target) in CGROUP_CONTROL_FILES:
                    findings.append(TerminationFinding(tuple(words), f"往 {target} 写：一次停掉（冻住）那个 cgroup 里的全部进程", "cgroup"))
            if words:
                found, sends = call_findings(words, loop_depth + opened > 0, process_table, own_ancestors)
                findings += found
                sends_signal = sends_signal or sends
                picks_processes = picks_processes or innermost_call(words)[0] in PROCESS_PICKERS
                reached = shell_code_and_timeout(words)[0]
                for kind, code in embedded_code_strings(words):
                    if kind == "python":
                        python_codes.append(code)
                    elif code != reached and depth < shell_words.MAXIMUM_NESTING:
                        inner, inner_sends, inner_picks = scan_shell_text(code, process_table, own_ancestors, depth + 1)
                        findings += inner
                        sends_signal, picks_processes = sends_signal or inner_sends, picks_processes or inner_picks
            loop_depth = max(0, loop_depth + opened - closed)
    for code in dict.fromkeys(python_codes + python_heredoc_bodies(text)):
        findings += python_termination_findings(code)
    return findings, sends_signal, picks_processes

def termination_findings(text, process_table=None, own_ancestors=(), whole_command=True):
    """整段 shell 文本（一条命令，或一整个脚本文件）里的终止写法。whole_command 为真（hook 判一条命令）时另判
    「同一条命令里先从 cgroup.procs、/proc 或 pgrep / pidof 挑进程、再发信号」；扫脚本文件时不判这一条，靠循环与命令替换两条。"""
    findings, sends_signal, picks_processes = scan_shell_text(text, process_table, own_ancestors, 0)
    if whole_command and sends_signal and (picks_processes or PROCESS_LIST_READING.search(shell_words.strip_data_heredocs(text))):
        findings.append(TerminationFinding((), "同一条命令里先从 cgroup.procs、/proc 或 pgrep / pidof 挑进程、再发信号：挑出来的是谁的，这一刻不知道", "cgroup"))
    return findings

def process_signal_refusal(command, process_table_reader=read_process_table, own_process_id=None):
    """命令里有打得到别人的终止写法就交回每一处的原因；否则交 []。写死的进程号才读进程表。前台、run_in_background 一样拒。"""
    if (os.environ.get("BASH_COMMAND_DETECTOR_DISABLE_CHECK") == "1"
            or os.environ.get("BASH_COMMAND_DETECTOR_ALLOW_PROCESS_SIGNALS") == "1"):
        return []
    table = process_table_reader() if re.search(r"(?<![\w$-])\d+\b", command) else None
    own_ancestors = ancestors_in(table, own_process_id or os.getpid()) if table is not None else []
    return list(dict.fromkeys(f"{finding.problem}（{' '.join(finding.words)[:120]}）" if finding.words else finding.problem
                              for finding in termination_findings(command, table, own_ancestors)))

# 扫脚本（门禁 73 号调 --scan-scripts）：同一份判定，按文件里的行号报；own-scope 标注与待改清单
OWN_SCOPE_ANNOTATION = re.compile(r"(?<!`)#\s*process-safety:own-scope\b[ \t]*(.*)$")   # 反引号里的是说明里举的例子，不算
OWN_SCOPE_GUARD = re.compile(r"singlefs-memory-cap-\*\.scope")
OWN_SCOPE_EXEMPTABLE_KINDS = {"loop", "cgroup"}
OWN_SCOPE_GUARD_LOOKBACK_LINES = 40
MINIMUM_ANNOTATION_REASON_CHARACTERS = 6
PENDING_LIST = os.path.join(".claude", "process-safety-pending")
SCANNED_SUFFIXES = (".sh", ".py")

def normalized_shell_line(line):
    return re.sub(r"\s+", " ", line.replace('"', "").replace("'", "").replace("\\", "")).strip()

def finding_lines(finding, text, lines, used):
    """一处发现落在文件的哪几行：python 的按代码在文件里的位置换算；shell 的按那条简单命令的词在去掉引号的行里找，
    同样的词出现几次就按次序分给几处。找不到的交 [0]。"""
    if finding.python_code:
        start = text.find(finding.python_code.rstrip("\n"))
        if start < 0:
            return [0]
        return [text.count("\n", 0, start) + finding.python_line]
    normalized = [normalized_shell_line(line) for line in lines]
    for size in (len(finding.words), 2, 1):
        joined = " ".join(finding.words[:size])
        if not joined:
            continue
        hits = [number for number, line in enumerate(normalized, 1) if joined in line]
        if hits:
            key = (joined, finding.problem)
            taken = used.setdefault(key, 0)
            used[key] = taken + 1
            return [hits[min(taken, len(hits) - 1)]]
    return [0]

def read_pending_list(root):
    """待改清单：一行一个相对仓根的路径，# 后写为什么。交回 ({路径: (行号, 理由)}, 清单本身的毛病)。"""
    path = os.path.join(root, PENDING_LIST)
    entries, problems = {}, []
    if not os.path.exists(path):
        return entries, problems
    with open(path, encoding="utf-8") as handle:
        for number, line in enumerate(handle, 1):
            stripped = line.strip()
            if not stripped or stripped.startswith("#"):
                continue
            relative, _, reason = stripped.partition("#")
            relative, reason = relative.strip(), reason.strip()
            if len(reason) < MINIMUM_ANNOTATION_REASON_CHARACTERS:
                problems.append(f"{PENDING_LIST}:{number} {relative} 没写为什么（# 后至少 {MINIMUM_ANNOTATION_REASON_CHARACTERS} 个字）")
            if not os.path.isfile(os.path.join(root, relative)):
                problems.append(f"{PENDING_LIST}:{number} {relative} 指向不存在的文件（不起作用的排除项让人以为那一份被绕开了）")
            entries[relative] = (number, reason)
    return entries, problems

def scan_script_file(path, text):
    """一个脚本文件里的发现（每条带行号）与 own-scope 标注的行号。"""
    lines = text.split("\n")
    if path.endswith(".py"):
        findings = [(finding.python_line, finding) for finding in python_termination_findings(text)]
    else:
        used = {}
        findings = [(line, finding) for finding in termination_findings(text, whole_command=False)
                    for line in finding_lines(finding, text, lines, used)]
    return findings, own_scope_annotations(path, text, lines), lines

def python_comment_annotations(code, first_line=1):
    """python 代码里写在注释（COMMENT 记号）里的 own-scope 标注：{文件行号: 理由}；字符串里写的不算。切不了记号的不认。"""
    found = {}
    try:
        for token in tokenize.generate_tokens(io.StringIO(code).readline):
            match = OWN_SCOPE_ANNOTATION.search(token.string) if token.type == tokenize.COMMENT else None
            if match:
                found[first_line + token.start[0] - 1] = match.group(1).strip()
    except (tokenize.TokenError, IndentationError, SyntaxError):
        pass
    return found

def own_scope_annotations(path, text, lines):
    """own-scope 标注的行号与理由。.py 只认注释；.sh 认 shell 那一层的行（bash -c 的代码串里也算），
    喂给别的命令的 heredoc 正文里的不算——喂给 python 的只认它的注释。"""
    if path.endswith(".py"):
        return python_comment_annotations(text)
    stripped = shell_words.strip_data_heredocs(text, keep_line_count=True).split("\n")
    data_lines = {number for number, (original, kept) in enumerate(zip(lines, stripped), 1) if original != kept and kept == ""}
    found = {number: OWN_SCOPE_ANNOTATION.search(line).group(1).strip()
             for number, line in enumerate(lines, 1) if number not in data_lines and OWN_SCOPE_ANNOTATION.search(line)}
    for body in python_heredoc_bodies(text):
        start = text.find(body.rstrip("\n"))
        if start >= 0:
            found.update(python_comment_annotations(body, text.count("\n", 0, start) + 1))
    return found

def scan_scripts(root, directories):
    """--scan-scripts 的本体：退出码 0 全绿、1 有红。"""
    pending, list_problems = read_pending_list(root)
    red, exempted, pending_hits, scanned = [], [], {}, {".sh": 0, ".py": 0}
    for directory in directories:
        if not os.path.isdir(directory):
            continue
        for file_name in sorted(os.listdir(directory)):
            path = os.path.join(directory, file_name)
            if not (os.path.isfile(path) and file_name.endswith(SCANNED_SUFFIXES)):
                continue
            relative = os.path.relpath(path, root)
            scanned[os.path.splitext(file_name)[1]] += 1
            text = open(path, encoding="utf-8", errors="replace").read()
            findings, annotations, lines = scan_script_file(path, text)
            used_annotations = set()
            for line, finding in findings:
                annotation = annotations.get(line)
                where = f"{relative}:{line or '?'}"
                if annotation is not None:
                    guarded = any(OWN_SCOPE_GUARD.search(earlier) for earlier in lines[max(0, line - 1 - OWN_SCOPE_GUARD_LOOKBACK_LINES):line - 1])
                    if finding.kind not in OWN_SCOPE_EXEMPTABLE_KINDS:
                        red.append((relative, f"{where}  {finding.problem}；own-scope 标注只放行循环与按 cgroup 挑进程两类，这一类没有例外"))
                    elif len(annotation) < MINIMUM_ANNOTATION_REASON_CHARACTERS:
                        red.append((relative, f"{where}  {finding.problem}；own-scope 标注没写怎么核的（至少 {MINIMUM_ANNOTATION_REASON_CHARACTERS} 个字）"))
                    elif not guarded:
                        red.append((relative, f"{where}  {finding.problem}；标了 own-scope，往上 {OWN_SCOPE_GUARD_LOOKBACK_LINES} 行里却没有核 singlefs-memory-cap-*.scope 的那一句"))
                    else:
                        exempted.append(where)
                    used_annotations.add(line)
                    continue
                if relative in pending:
                    pending_hits.setdefault(relative, []).append(where)
                    continue
                red.append((relative, f"{where}  {finding.problem}"))
            for line in sorted(set(annotations) - used_annotations):
                red.append((relative, f"{relative}:{line}  标了 # process-safety:own-scope，这一行却没有要放行的发信号写法（不起作用的标注让人以为那一处核过）"))
    for relative in sorted(pending):
        if relative not in pending_hits and os.path.isfile(os.path.join(root, relative)):
            list_problems.append(f"{PENDING_LIST}:{pending[relative][0]} {relative} 里一处要放行的都没有（改好了就从清单里删掉这一行）")
    for problem in list_problems:
        print(f"  ✗ {problem}")  # gate-lint:detail
    for _, message in red:
        print(f"  ✗ {message}")  # gate-lint:detail
    if red or list_problems:
        print(f"  ✗ 进程安全：{len(red)} 处发信号的写法打得到不是自己点名的那一个进程，待改清单 {len(list_problems)} 处不对（上面逐处列出）")  # gate-lint:summary
        print("     → 怎么办：只停自己起的、点名的那一个：记下它的 pid（`$!`）或任务号，单独 `kill \"$pid\"`、`kill %1` 或 "
              "`python3 .claude/singlefs-ai-sop/scripts/proc.py stop <pid>`，不在循环里、不一次给几个、不带负号或 0、不按名字挑；"
              "确实要按 cgroup 批量发信号的，先核 cgroup 路径是自己开的 singlefs-memory-cap-*.scope，再在那一行写 `# process-safety:own-scope <怎么核的>`；"
              f"别人正在改、这一轮动不了的文件登记进 {PENDING_LIST}（一行一个路径、# 后写为什么），改好了删掉那一行")
        return 1
    pending_note = "；".join(f"{relative}（{'、'.join(hits)}）" for relative, hits in sorted(pending_hits.items())) or "无"
    print(f"  ✓ 进程安全：查了 {scanned['.sh'] + scanned['.py']} 个脚本（.sh {scanned['.sh']} 个、.py {scanned['.py']} 个），"
          f"发信号的写法都只打得到点名的一个进程；own-scope 标注放行 {len(exempted)} 处（{'、'.join(exempted) or '无'}）；"
          f"没判的：{PENDING_LIST} 里的 {len(pending_hits)} 个文件 {pending_note}")
    return 0

def selftest(hook_dir):
    if shell_words is None:
        print(f"  ✗ 自检：读不到共用切词模块 {shell_words_library_path(hook_dir)}（{shell_words_error!r}）")
        print("    → 怎么办：恢复 .claude/hooks/lib_shell_words.py（切词与认命令位置只有那一份，别在 hook 里再抄一份），再跑 --selftest")
        return 1
    work = tempfile.mkdtemp(prefix="bash-command-detector-")
    detections = os.path.join(work, "detections.jsonl")
    cases = [
        ("普通命令", "cargo test --release", 0),
        ("带 timeout 的等待循环", "timeout 600 bash -c 'until grep -q done log; do sleep 10; done'", 0),
        ("while read 读文件", 'while read -r line; do echo "$line"; done < list.txt', 0),
        ("没有 timeout 的 until 等待", 'until grep -q "^exit=" log; do sleep 10; done', 1),
        ("按模式找进程归上游钩子，这里不记", "pgrep -f e154-binary", 0),
        ("写进笔记的 heredoc 正文里是一段没超时的等待循环", "cat > note.md <<'EOF'\nuntil grep -q done log; do sleep 10; done\nEOF", 0),
        ("喂给 bash 的 heredoc 里真是一段没超时的等待循环", "bash <<'EOF'\nuntil grep -q done log; do sleep 10; done\nEOF", 1),
        ("heredoc 之后的命令里是没超时的等待循环", "cat > note.md <<EOF\n说明\nEOF\nuntil grep -q done log; do sleep 10; done", 1),
        ("后台起的 nohup … & 加 disown", "nohup nice -n 19 bash gate.sh > gate.log 2>&1 & echo $!; disown", 1, True),
        ("后台起的命令末尾单独一个 &", "bash cache-keepalive.sh > keepalive.log 2>&1 &", 1, True),
        ("后台起的命令只有 2>&1、|& 与 &&", "cargo build 2>&1 | tail -3 && echo ok; make |& tee out", 0, True),
        ("后台起的普通命令", "bash cache-keepalive.sh", 0, True),
        ("后台起的并行加 wait", "bash a.sh > a.log 2>&1 & bash b.sh > b.log 2>&1 & wait", 0, True),
        ("后台起的 & 在 wait 之后", "wait; bash late.sh &", 1, True),
        ("前台命令里的 & 不算（不是 run_in_background）", "sleep 1 & wait", 0),
        ('bg-notify-r1 X01', 'nice -n 19 bash .claude/scripts/gate.sh --staged > /tmp/claude-1000/gate-triage-commit-0918/gate-run.log 2>&1 & echo "started pid $!"', 1, True),
        ('bg-notify-r1 X02', '(nice -n 19 bash .claude/scripts/gate.sh --staged > /tmp/claude-1000/gate-triage-commit-0918/gate-run.log 2>&1 &)', 1, True),
        ('bg-notify-r1 X03', 'nice -n 19 bash .claude/scripts/gate.sh --staged > /tmp/claude-1000/gate-triage-commit-0918/gate-run.log 2>&1 & echo "gate.sh started as pid $!; I will wait for the notification"', 1, True),
        ('bg-notify-r1 X04', 'nice -n 19 bash .claude/scripts/gate.sh --staged > /tmp/claude-1000/gate-triage-commit-0918/gate-run.log 2>&1 & bash research/scripts/cache-keepalive.sh & wait $!', 1, True),
        ('bg-notify-r1 X05', 'nice -n 19 bash .claude/scripts/gate.sh --staged > /tmp/claude-1000/gate-triage-commit-0918/gate-run.log 2>&1 & bash research/scripts/cache-keepalive.sh & wait -n', 1, True),
        ('bg-notify-r1 X06', '(nice -n 19 bash .claude/scripts/gate.sh --staged > /tmp/claude-1000/gate-triage-commit-0918/gate-run.log 2>&1 &); wait', 1, True),
        ('bg-notify-r1 X07', 'pid=$(nice -n 19 bash .claude/scripts/gate.sh --staged > /tmp/claude-1000/gate-triage-commit-0918/gate-run.log 2>&1 & echo $!); echo "gate pid $pid"; wait "$pid"', 1, True),
        ('bg-notify-r1 X08', 'coproc GATE { nice -n 19 bash .claude/scripts/gate.sh --staged > /tmp/claude-1000/gate-triage-commit-0918/gate-run.log 2>&1; }', 1, True),
        ('bg-notify-r1 X09', "tmux new-session -d -s gate 'nice -n 19 bash .claude/scripts/gate.sh --staged > /tmp/claude-1000/gate-triage-commit-0918/gate-run.log 2>&1'", 1, True),
        ('bg-notify-r1 X10', "screen -dmS gate bash -c 'nice -n 19 bash .claude/scripts/gate.sh --staged > /tmp/claude-1000/gate-triage-commit-0918/gate-run.log 2>&1'", 1, True),
        ('bg-notify-r1 X11', 'systemd-run --user --collect --unit gate-run nice -n 19 bash .claude/scripts/gate.sh --staged', 1, True),
        ('bg-notify-r1 X12', "nice -n 19 python3 - > /tmp/claude-1000/gate-triage-commit-0918/rerun.log 2>&1 <<'EOF' &\nimport subprocess\np = subprocess.Popen(['bash', 'research/scripts/replay.sh'])\np.wait()\nEOF", 1, True),
        ('bg-notify-r1 X13', 'nice -n 19 bash .claude/scripts/gate.sh --staged > /tmp/claude-1000/gate-triage-commit-0918/gate-run.log 2>&1 &', 0, False),
        ('bg-notify-r1 F01', 'nohup nice -n 19 bash .claude/scripts/gate.sh --staged > /tmp/claude-1000/gate-triage-commit-0918/gate-run.log 2>&1', 0, True),
        ('bg-notify-r1 F02（第三种不记；它的轮询循环没有 timeout，记的是第二种）', 'nice -n 19 bash .claude/scripts/gate.sh --staged > /tmp/claude-1000/gate-triage-commit-0918/gate-run.log 2>&1 & pid=$!; while kill -0 "$pid" 2>/dev/null; do sleep 20; done; tail -3 /tmp/claude-1000/gate-triage-commit-0918/gate-run.log', 1, True),
        ('bg-notify-r1 F03', 'nice -n 19 bash .claude/scripts/gate.sh --staged > /tmp/claude-1000/gate-triage-commit-0918/gate-run.log 2>&1 & tail --pid=$! -f /tmp/claude-1000/gate-triage-commit-0918/gate-run.log > /dev/null', 0, True),
        ('bg-notify-r1 F04', "nice -n 19 python3 - <<'EOF' > /tmp/claude-1000/gate-triage-commit-0918/tally.txt\nimport json\nrows = [json.loads(line) for line in open('/tmp/claude-1000/gate-triage-commit-0918/events.jsonl')]\nprint(sum(1 for row in rows if row['flags'] & 0x4))\nEOF", 0, True),
        ('bg-notify-r1 F05', "cat > /tmp/claude-1000/impl-probe/copy/crates/singlefs-harness/tests/probe.rs <<'EOF'\nfn digest(bytes: &[u8]) -> u64 { bytes.iter().map(|&b| b as u64).sum() }\n#[test]\nfn probe() { assert_eq!(digest(&[1, 2]), 3); }\nEOF\ncd /tmp/claude-1000/impl-probe/copy && nice -n 19 cargo test --release --test probe 2>&1 | tail -20", 0, True),
        ('bg-notify-r1 F06', "nice -n 19 cargo test --release 2>&1 | sed -E 's/^test (.*) \\.\\.\\. FAILED$/!! &/' > /tmp/claude-1000/impl-probe/test.log", 1, True),
        ('bg-notify-r1 F07', 'setsid -w nice -n 19 bash .claude/scripts/gate.sh --staged > /tmp/claude-1000/gate-triage-commit-0918/gate-run.log 2>&1', 0, True),
        ("裸 setsid 不 fork、外层照样等", "setsid bash long.sh > long.log 2>&1 < /dev/null", 0, True),
        ("setsid -f 放后台", "setsid -f bash long.sh > long.log 2>&1 < /dev/null", 1, True),
        ("setsid 跟着的 tail -f 不是 setsid 的选项", "setsid tail -f log.txt", 0, True),
    ]
    results = []
    for label, command, want, *background in cases:
        tool_input = {"command": command, "run_in_background": bool(background and background[0])}
        written = record({"tool_name": "Bash", "session_id": "s", "tool_input": tool_input}, detections)
        results.append((label, want, written))
    # 起看门狗：(说明, 命令, run_in_background, 该不该拒绝)
    watchdog_cases = [
        ("前台起、输出丢进 /dev/null 还带 &", "bash research/scripts/watch.sh a1 > /dev/null 2>&1 &", False, 1),
        ("run_in_background 起、只写 watch.sh", "bash research/scripts/watch.sh a1", True, 0),
        ("run_in_background 起却又带 &", "bash research/scripts/watch.sh a1 &", True, 1),
        ("前台 grep 把 watch.sh 当参数", "grep -n watch.sh records/x.md", False, 0),
        ("前台 agent-watch.py report 不是看门狗", "python3 research/scripts/agent-watch.py report --session-dir x", False, 0),
        ("前台起、什么都没加", "bash research/scripts/watch.sh a1,a2", False, 1),
        ("前台起 agent-watch.py watch", "python3 research/scripts/agent-watch.py watch --agents a1", False, 1),
        ("run_in_background 起 agent-watch.py watch", "python3 research/scripts/agent-watch.py watch --agents a1", True, 0),
        ("run_in_background 起、带 --ack 与 cd", "cd /home/fy5090/code/singlefs && bash research/scripts/watch.sh --ack a1:上下文过大 a1", True, 0),
        ("run_in_background 起却挂 nohup", "nohup bash research/scripts/watch.sh a1", True, 1),
        ("run_in_background 起却挂 setsid", "setsid bash research/scripts/watch.sh --processes", True, 1),
        ("run_in_background 起却跟 disown", "bash research/scripts/watch.sh a1; disown", True, 1),
        ("run_in_background 起却把输出丢进 /dev/null", "bash research/scripts/watch.sh a1 > /dev/null", True, 1),
        ("前台起、写在 bash -c 里", "bash -c 'nice -n 19 ./research/scripts/watch.sh a1'", False, 1),
        ("前台起、包在 capped.sh 里", "bash research/scripts/capped.sh 4 bash research/scripts/watch.sh a1", False, 1),
        ("run_in_background 起、包在 capped.sh 里", "bash research/scripts/capped.sh 4 bash research/scripts/watch.sh a1", True, 0),
        ("前台 watch.sh --selftest 不盯", "bash research/scripts/watch.sh --selftest", False, 0),
        ("前台 watch.sh --report 不盯", "bash research/scripts/watch.sh --report", False, 0),
        ("前台 bash -n 只查语法", "bash -n research/scripts/watch.sh", False, 0),
        ("前台读看门狗输出", "tail -5 /tmp/claude-1000/x/tasks/b1.output; cat research/scripts/watch.sh | head", False, 0),
        ("前台 echo 引号里当数据写", 'echo "bash research/scripts/watch.sh a1 &"', False, 0),
        ("前台写进笔记的 heredoc 正文", "cat > note.md <<'EOF'\nbash research/scripts/watch.sh a1 > /dev/null 2>&1 &\nEOF", False, 0),
        ("前台注释里写的", "ls  # bash research/scripts/watch.sh a1 &", False, 0),
        ("前台 diff <(git show …watch.sh) watch.sh：进程替换之后的词是 diff 的参数", "diff <(git show HEAD:research/scripts/watch.sh) research/scripts/watch.sh", False, 0),
    ]
    for label, command, background, want in watchdog_cases:
        results.append((f"看门狗:{label}", want, 1 if watchdog_rejection(command, background)[0] else 0))
    # 前台没超时的等待循环：(说明, 命令, run_in_background, 该不该拒绝)
    wait_loop_cases = [
        ("前台 until … sleep 没超时", "until grep -q x f; do sleep 10; done", False, 1),
        ("同一条套上 timeout 60", "timeout 60 bash -c 'until grep -q x f; do sleep 10; done'", False, 0),
        ("同一条走 run_in_background", "until grep -q x f; do sleep 10; done", True, 0),
        ("前台按 pid 轮询的 while kill -0", 'while kill -0 "$pid" 2>/dev/null; do sleep 20; done; tail -3 log', False, 1),
        ("前台 timeout 只在循环条件里", "until timeout 5 grep -q x f; do sleep 10; done", False, 1),
        ("前台多行写的 while true", "while true\ndo\n  grep -q x f && break\n  sleep 5\ndone", False, 1),
        ("前台 sleep 在循环条件里", "while sleep 5; do grep -q x f && break; done", False, 1),
        ("前台喂给 bash 的 heredoc 里的循环", "bash <<'EOF'\nuntil grep -q x f; do sleep 10; done\nEOF", False, 1),
        ("timeout 60 bash 的 heredoc 里的循环", "timeout 60 bash <<'EOF'\nuntil grep -q x f; do sleep 10; done\nEOF", False, 0),
        ("capped.sh 与 nice 包着的 timeout", "nice -n 19 capped.sh 4 timeout 600 bash -c 'until grep -q x f; do sleep 10; done'", False, 0),
        ("前台 bash -c 里没有 timeout", "nice -n 19 bash -c 'until grep -q x f; do sleep 10; done'", False, 1),
        ("前台 capped.sh 包着的 bash -c 里没有 timeout", "bash research/scripts/capped.sh 4 bash -c 'until grep -q x f; do sleep 10; done'", False, 1),
        ("前台 run-with-memory-cap.sh 包着的 bash -c 里没有 timeout", "bash research/scripts/run-with-memory-cap.sh 4G bash -c 'until grep -q x f; do sleep 10; done'", False, 1),
        ("前台命令替换里的循环", "state=$(until grep -q x f; do sleep 1; done; echo ok)", False, 1),
        ("前台 if 里的循环", "if true; then while ! grep -q x f; do sleep 1; done; fi", False, 1),
        ("前台 echo 引号里当数据写", 'echo "until grep -q x f; do sleep 10; done"', False, 0),
        ("前台 grep 的参数里写着循环", "grep -n 'until .*; do sleep' notes.md", False, 0),
        ("前台写进笔记的 heredoc 正文", "cat > note.md <<'EOF'\nuntil grep -q x f; do sleep 10; done\nEOF", False, 0),
        ("前台注释里写的", "ls  # until grep -q x f; do sleep 10; done", False, 0),
        ("前台 while read 没有 sleep", 'while read -r line; do echo "$line"; done < list.txt', False, 0),
        ("前台 sleep 在循环外面", 'while read -r line; do echo "$line"; done < list.txt; sleep 5', False, 0),
        ("前台 for 循环", "for attempt in 1 2 3; do grep -q x f && break; sleep 1; done", False, 0),
        ("前台单独的 sleep", "sleep 5 && tail -3 log", False, 0),
        ("前台 echo 的参数里有 while、后面才 sleep", "echo while done; sleep 1", False, 0),
        ("前台循环体里套 for、sleep 在 for 之后", "until grep -q x f; do for part in a b; do echo $part; done; sleep 1; done", False, 1),
    ]
    for label, command, background, want in wait_loop_cases:
        results.append((f"等待循环:{label}", want, 1 if wait_loop_refusal(command, background) else 0))
    # 把活放出追踪的写法：(说明, 命令, 该不该拒绝)；前台、run_in_background 一样拒，detaching_refusal 不看 run_in_background
    detaching_cases = [
        ("现场：& disown 之后又 & wait", "nice -n 19 bash long.sh > long.log 2>&1 & disown 2>/dev/null || true & wait", 1),
        ("nohup … &", "nohup bash long.sh > long.log 2>&1 &", 1),
        ("nice 包着的 nohup … &", "nice -n 19 nohup bash long.sh > long.log 2>&1 & echo $!", 1),
        ("coproc", "coproc GATE { bash gate.sh; }", 1),
        ("setsid -f", "setsid -f bash long.sh > long.log 2>&1 < /dev/null", 1),
        ("setsid --fork", "setsid --fork bash long.sh", 1),
        ("tmux new-session -d", "tmux new-session -d -s gate 'bash gate.sh'", 1),
        ("screen -dmS", "screen -dmS gate bash -c 'bash gate.sh'", 1),
        ("systemd-run 不等结束", "systemd-run --user --collect --unit gate-run bash gate.sh", 1),
        ("bash -c 里的 & disown", "bash -c 'bash long.sh & disown'", 1),
        ("命令替换里的 nohup … &", "pid=$(nohup bash long.sh > long.log 2>&1 & echo $!)", 1),
        ("capped.sh 包着的 setsid -f", "bash research/scripts/capped.sh 4 setsid -f bash long.sh", 1),
        ("喂给 bash 的 heredoc 里的 disown", "bash <<'EOF'\nbash long.sh &\ndisown\nEOF", 1),
        ("nohup 不带 &", "nohup bash long.sh > long.log 2>&1", 0),
        ("裸 setsid 不 fork", "setsid bash long.sh > long.log 2>&1 < /dev/null", 0),
        ("setsid 跟着的 tail -f", "setsid tail -f log.txt", 0),
        ("systemd-run --wait", "systemd-run --user --wait bash gate.sh", 0),
        ("tmux ls", "tmux ls", 0),
        ("并行加 wait", "bash a.sh > a.log 2>&1 & bash b.sh > b.log 2>&1 & wait", 0),
        ("echo 引号里当数据写", 'echo "nohup bash x.sh & disown"', 0),
        ("grep 的参数里写着 disown", "grep -n 'disown\\|coproc' notes.md", 0),
        ("写进笔记的 heredoc 正文", "cat > note.md <<'EOF'\nnohup bash long.sh &\ndisown\nEOF", 0),
        ("注释里写的", "ls  # nohup bash long.sh & disown", 0),
    ]
    for label, command, want in detaching_cases:
        results.append((f"放出追踪:{label}", want, 1 if detaching_refusal(command) else 0))
    # run_in_background 里后面没有 wait 的单独 &：(说明, 命令, run_in_background, 该不该拒绝)
    unwaited_ampersand_cases = [
        ("结尾单独一个 &", "bash long.sh > long.log 2>&1 &", True, 1),
        ("& 之后接 echo $!", "bash long.sh > long.log 2>&1 & echo $!", True, 1),
        ("& 在 capped.sh 前缀的那条命令上", "bash research/scripts/capped.sh 4 nice -n 19 cargo test --release > test.log 2>&1 &", True, 1),
        ("& 在 capped.sh 包着的 bash -c 里", "bash research/scripts/capped.sh 4 bash -c 'cargo test --release > test.log 2>&1 &'", True, 1),
        ("& 在 wait 之后", "wait; bash late.sh &", True, 1),
        ("圆括号子 shell 里的 &、wait 在外面", "(bash long.sh > long.log 2>&1 &); wait", True, 1),
        ("& 之后的 wait 在另一对圆括号里", "bash long.sh & (wait)", True, 1),
        ("命令替换里的 &、wait 在外面", 'pid=$(bash long.sh > long.log 2>&1 & echo $!); wait "$pid"', True, 1),
        ("bash -c 里的 &、wait 在外面", "bash -c 'bash long.sh &'; wait", True, 1),
        ("喂给 bash 的 heredoc 正文里的 &", "bash <<'EOF'\nbash long.sh > long.log 2>&1 &\nEOF", True, 1),
        ("喂给 python 的 heredoc 挂在 & 前面", "python3 - > rerun.log 2>&1 <<'EOF' &\nimport subprocess\nEOF", True, 1),
        ("& 之后只 wait -n", "bash a.sh & bash b.sh & wait -n", True, 1),
        ("& 之后按 pid 轮询不算 wait", "bash long.sh > long.log 2>&1 & tail --pid=$! -f /dev/null", True, 1),
        ("整个循环放后台", "for part in a b; do bash $part.sh; done &", True, 1),
        ("一对圆括号整组放后台", "(bash a.sh; bash b.sh) > both.log 2>&1 &", True, 1),
        ('& 之后 wait "$pid"', 'bash long.sh > long.log 2>&1 & pid=$!; wait "$pid"', True, 0),
        ("&&", "cargo build --release && cargo test --release", True, 0),
        ("2>&1 重定向", "cargo test --release > test.log 2>&1", True, 0),
        ("|& 与 &>", "make |& tee out.txt; cargo test &> test.log", True, 0),
        ("并行加不带参数的 wait", "bash a.sh > a.log 2>&1 & bash b.sh > b.log 2>&1 & wait", True, 0),
        ("循环里放后台、循环之后 wait", "for part in a b; do bash $part.sh & done; wait", True, 0),
        ("bash -c 里 & 加 wait", "bash -c 'bash a.sh & bash b.sh & wait'", True, 0),
        ("圆括号里 & 加 wait", "(bash a.sh & wait) > both.log", True, 0),
        ("if 里 & 之后 wait", "if true; then bash a.sh & wait; fi", True, 0),
        ("数组赋值的括号不打乱圆括号组", "parts=(a b); bash a.sh & wait", True, 0),
        ("几个 & 只 wait 一个 pid：不拒，照检出记", "bash a.sh & bash b.sh & wait $!", True, 0),
        ("引号里当数据写的 &", "echo 'bash long.sh &' > note.txt", True, 0),
        ("写进文件的 heredoc 正文里的 &", "cat > note.md <<'EOF'\nbash long.sh &\nEOF", True, 0),
        ("注释里的 &", "ls  # bash long.sh &", True, 0),
        ("前台：结尾单独一个 &", "bash long.sh > long.log 2>&1 &", False, 0),
        ("前台：& 之后接 echo $!", "bash long.sh > long.log 2>&1 & echo $!", False, 0),
    ]
    for label, command, background, want in unwaited_ampersand_cases:
        results.append((f"单独的 &:{label}", want, 1 if unwaited_ampersand_refusal(command, background) else 0))
    # ⑥ 终止进程：(说明, 命令, 该不该拒绝)；写死的进程号对着一张造出来的进程表判，这条命令的会话是 claude 1000（祖先 1001 → 1000 → 900 → 800 → 700 → 1）。
    # 每条命令只拿来判，一条都不执行
    fake_table = {
        1: {"parent": 0, "name": "systemd", "command_line": "/sbin/init"},
        700: {"parent": 1, "name": "sshd", "command_line": "sshd: /usr/sbin/sshd -D"},
        800: {"parent": 700, "name": "sshd-session", "command_line": "sshd-session: fy5090"},
        900: {"parent": 800, "name": "MainThread", "command_line": "/home/u/.vscode-server/cli/servers/Stable/server/node --dns-result-order=ipv4first"},
        1000: {"parent": 900, "name": "claude", "command_line": "/home/u/.vscode-server/extensions/anthropic.claude-code/native-binary/claude"},
        1001: {"parent": 1000, "name": "bash", "command_line": "/bin/bash -c source snapshot.sh"},
        1100: {"parent": 1001, "name": "cargo", "command_line": "cargo test -p singlefs-core --lib"},
        2000: {"parent": 900, "name": "claude", "command_line": "/home/u/.vscode-server/extensions/anthropic.claude-code/native-binary/claude"},
        2100: {"parent": 2000, "name": "bash", "command_line": "/bin/bash -c bash gate.sh"},
        2101: {"parent": 2100, "name": "cargo", "command_line": "cargo test --release"},
        3000: {"parent": 1, "name": "python3", "command_line": "python3 -m vllm.entrypoints.openai.api_server --model x"},
        4000: {"parent": 1, "name": "sleep", "command_line": "sleep 600"},
    }
    fake_reader = lambda: fake_table  # noqa: E731
    process_signal_cases = [
        ("kill -- 负进程号（进程组）", "kill -- -1234", 1),
        ("kill -TERM 负进程号", "kill -TERM -1234", 1),
        ("kill -s TERM 负进程号", "kill -s TERM -1234", 1),
        ("自己的任务的进程组 -\"$job\"", 'kill -INT -- -"$job"', 1),
        ("kill 0", "kill 0", 1),
        ("kill -TERM 0", "kill -TERM 0", 1),
        ("kill -9 -1（全部）", "kill -9 -1", 1),
        ("kill $PPID（Claude 会话本身）", "kill -TERM $PPID", 1),
        ("kill $(pgrep …)", "kill -TERM $(pgrep cargo)", 1),
        ("kill $(jobs -p)", "kill $(jobs -p)", 1),
        ("kill `pidof sshd`", "kill -HUP `pidof sshd`", 1),
        ("kill 数组整批展开", 'kill "${worker_pids[@]}"', 1),
        ("kill \"$@\"", 'kill "$@"', 1),
        ("kill 一次两个", 'kill "$first" "$second"', 1),
        ("kill %1 %2", "kill %1 %2", 1),
        ("kill 带通配", "kill /proc/[0-9]*", 1),
        ("for 循环里 kill", 'for pid in $(cat pids.txt); do kill "$pid"; done', 1),
        ("while read cgroup.procs 里 kill（现场那一段）",
         'while read -r process_id; do [[ "$process_id" != "$BASHPID" ]] && kill -TERM "$process_id" 2>/dev/null; done < /sys/fs/cgroup/user.slice/user-1000.slice/session-1.scope/cgroup.procs', 1),
        ("多行循环里 kill", 'for pid in "${pids[@]}"\ndo\n  kill -TERM "$pid"\ndone', 1),
        ("cgroup.procs 交给 xargs kill", "cat /sys/fs/cgroup/x/cgroup.procs | xargs kill", 1),
        ("xargs -n1 kill", "xargs -n1 kill < pids.txt", 1),
        ("pgrep 交给 xargs -r kill -9", "pgrep -u fy5090 node | xargs -r kill -9", 1),
        ("find -exec kill", "find /proc -maxdepth 1 -name '[0-9]*' -exec kill {} +", 1),
        ("pkill 按名字", "pkill cargo", 1),
        ("pkill -x claude", "pkill -x claude", 1),
        ("killall", "killall node", 1),
        ("fuser -k", "fuser -k 8200/tcp", 1),
        ("先从 cgroup.procs 挑一个再 kill", 'p=$(head -1 /sys/fs/cgroup/x/cgroup.procs); kill -TERM "$p"', 1),
        ("先 pgrep 挑一个再 kill", 'p=$(pgrep -n node); kill "$p"', 1),
        ("python -c 推导式里读 cgroup.procs 再 os.kill",
         "python3 -c 'import os; [os.kill(int(p), 15) for p in open(\"/sys/fs/cgroup/x/cgroup.procs\")]'", 1),
        ("喂给 python 的 heredoc 里 os.killpg", "python3 - <<'EOF'\nimport os, signal\nos.killpg(os.getpgid(0), signal.SIGTERM)\nEOF", 1),
        ("python -c os.kill(0, …)", "python3 -c 'import os; os.kill(0, 15)'", 1),
        ("systemctl stop ssh", "systemctl stop ssh", 1),
        ("sudo systemctl restart sshd", "sudo systemctl restart sshd", 1),
        ("systemctl kill session-1.scope", "systemctl kill session-1.scope", 1),
        ("systemctl stop user@1000.service", "systemctl stop user@1000.service", 1),
        ("systemctl set-property user.slice", "systemctl set-property user.slice MemoryMax=1G", 1),
        ("systemctl stop vllm-prod", "systemctl stop vllm-prod", 1),
        ("systemctl disable --now ssh", "systemctl disable --now ssh", 1),
        ("systemctl --user stop 带通配 singlefs-*", "systemctl --user stop 'singlefs-*'", 1),
        ("systemctl --user kill 带通配 *.scope", "systemctl --user kill '*.scope'", 1),
        ("systemctl --user stop 命令替换列出来的一批", "systemctl --user stop $(systemctl --user list-units --plain 'singlefs*' | awk '{print $1}')", 1),
        ("systemctl isolate", "systemctl isolate multi-user.target", 1),
        ("systemctl --user exit", "systemctl --user exit", 1),
        ("systemctl reboot", "systemctl reboot", 1),
        ("loginctl terminate-session", "loginctl terminate-session 1", 1),
        ("loginctl kill-user", "loginctl kill-user fy5090", 1),
        ("往 cgroup.kill 写", "echo 1 > /sys/fs/cgroup/user.slice/user-1000.slice/session-1.scope/cgroup.kill", 1),
        ("tee 进 cgroup.freeze", "echo 1 | tee /sys/fs/cgroup/x/cgroup.freeze", 1),
        ("sudo reboot", "sudo reboot", 1),
        ("shutdown -h now", "shutdown -h now", 1),
        ("写死的进程号是 sshd", "kill -TERM 700", 1),
        ("写死的进程号是 VSCode 服务端", "kill 900", 1),
        ("写死的进程号是这个会话自己的 Claude", "kill -9 1000", 1),
        ("写死的进程号是另一个 Claude 会话", "kill 2000", 1),
        ("写死的进程号是另一个 Claude 会话起的 cargo", "kill 2101", 1),
        ("写死的进程号是 vllm", "kill 3000", 1),
        ("kill -HUP 1", "kill -HUP 1", 1),
        ("proc.py stop 写死的 sshd", "python3 .claude/singlefs-ai-sop/scripts/proc.py stop 700", 1),
        ("proc.py stop 一次两个", "python3 .claude/singlefs-ai-sop/scripts/proc.py stop 1100 4000", 1),
        ("循环里 proc.py stop", 'for p in 1100 4000; do python3 .claude/singlefs-ai-sop/scripts/proc.py stop "$p"; done', 1),
        ("bash -c 里 kill 进程组", "bash -c 'kill -TERM -- -$$'", 1),
        ("nice 包着 kill -9 -1", "nice -n 19 kill -9 -1", 1),
        ("喂给 bash 的 heredoc 里 kill 0", "bash <<'EOF'\nkill 0\nEOF", 1),
        ("timeout 包着 kill 负进程号", "timeout 5 kill -TERM -4000", 1),
        ("run-with-memory-cap.sh 包着 pkill", "bash research/scripts/run-with-memory-cap.sh 1G pkill cargo", 1),
        ("shell 函数与变量里的脚本包着的 bash -c 里 kill 0", 'in_context bash "$target" 64M bash -c \'kill 0\'', 1),
        ("shell 函数包着的 python -c 里 os.killpg", "in_context python3 -c 'import os; os.killpg(1234, 15)'", 1),
        ("点名一个 kill \"$!\"", 'kill "$!"', 0),
        ("点名一个 kill -TERM \"$pid\"", 'kill -TERM "$pid"', 0),
        ("点名一个 kill -9 $worker", "kill -9 $worker", 0),
        ("点名一个任务号 kill %1", "kill %1", 0),
        ("点名一个任务号 kill %%", "kill %%", 0),
        ("点名一个任务号 kill -INT %2", "kill -INT %2", 0),
        ("kill -0 探活", 'kill -0 "$pid"', 0),
        ("kill -s 0 探活写死的进程号", "kill -s 0 700", 0),
        ("kill -l", "kill -l 15", 0),
        ("写死的进程号是这个会话起的 cargo", "kill 1100", 0),
        ("写死的进程号是没人认领的孤儿", "kill 4000", 0),
        ("写死的进程号不存在", "kill 99999999", 0),
        ("proc.py stop 一个写死的", "python3 .claude/singlefs-ai-sop/scripts/proc.py stop 1100", 0),
        ("proc.py stop 一个变量带 --grace", 'python3 .claude/singlefs-ai-sop/scripts/proc.py stop "$pid" --grace 5', 0),
        ("按 pid 轮询的 while kill -0", 'while kill -0 "$pid" 2>/dev/null; do sleep 5; done', 0),
        ("先核进程再停一个", '[[ "$(ps -p "$pid" -o args=)" == *mutate* ]] && kill "$pid"', 0),
        ("grep 参数里的 kill 字样", "grep -n 'kill -TERM -1' notes.md", 0),
        ("echo 里的 pkill", "echo 'pkill cargo'", 0),
        ("写进文件的 heredoc 正文", "cat > note.md <<'EOF'\nkill 0\nfor p in $(pgrep x); do kill $p; done\nEOF", 0),
        ("git commit 说明里写 kill 0", "git commit -m 'kill 0 的拒绝'", 0),
        ("systemctl --user stop 自己的 slice", "systemctl --user stop singlefs_memory_selftest_123_1.slice", 0),
        ("systemctl status sshd", "systemctl --user status sshd; systemctl status vllm-prod", 0),
        ("systemctl show 与 set-property 自己的 slice", "systemctl --user show -p ControlGroup --value singlefs-heavy.slice && systemctl --user set-property --runtime singlefs-heavy.slice MemoryMax=40G", 0),
        ("systemctl list-units 带通配", "systemctl --user list-units 'singlefs*'", 0),
        ("systemctl reset-failed 自己的 scope", "systemctl --user reset-failed singlefs-memory-cap-1-2.scope", 0),
        ("loginctl list-sessions", "loginctl list-sessions; loginctl show-session 1", 0),
        ("只读 cgroup.procs", "cat /sys/fs/cgroup/user.slice/user-1000.slice/session-1.scope/cgroup.procs", 0),
        ("只读 cgroup.kill", "cat /sys/fs/cgroup/x/cgroup.kill", 0),
        ("只 pgrep 不发信号", "pgrep -u fy5090 cargo; ps -o pid,args --ppid 1100", 0),
        ("python -c os.kill 探活", "python3 -c 'import os; os.kill(1100, 0)'", 0),
        ("python -c 停一个 pid 文件里的", "python3 -c 'import os,signal; os.kill(int(open(\"worker.pid\").read()), signal.SIGTERM)'", 0),
        ("timeout -s KILL 包着命令", "timeout -s KILL 60 cargo test -p singlefs-core --lib", 0),
        ("shutdown -c", "shutdown -c", 0),
    ]
    for label, command, want in process_signal_cases:
        results.append((f"终止进程:{label}", want, 1 if process_signal_refusal(command, fake_reader, 1001) else 0))
    # 扫脚本（门禁 73 号那一路）：临时目录里造脚本，走 --scan-scripts 真实入口，看红绿与报的行号
    scan_root = os.path.join(work, "scan-root")
    scan_scripts_directory = os.path.join(scan_root, "research", "scripts")
    os.makedirs(scan_scripts_directory)
    os.makedirs(os.path.join(scan_root, ".claude"))
    scan_files = {
        "group.sh": '#!/usr/bin/env bash\nset -m\nsleep 30 &\njob=$!\nkill -INT -- -"$job"\n',
        "loop.sh": '#!/usr/bin/env bash\nfor pid in "${worker_pids[@]}"; do\n  kill -TERM "$pid"\ndone\n',
        "storm-guarded.sh": ('#!/usr/bin/env bash\ngroup="/sys/fs/cgroup$(cut -d: -f3 /proc/self/cgroup)"\n'
                             'case "$group" in */singlefs-memory-cap-*.scope) ;; *) exit 0 ;; esac\n'
                             'while read -r process_id; do kill -TERM "$process_id"; done < "$group/cgroup.procs"  # process-safety:own-scope 上一行 case 核过是自己开的 scope\n'),
        "storm-unguarded.sh": ('#!/usr/bin/env bash\ngroup="/sys/fs/cgroup$(cut -d: -f3 /proc/self/cgroup)"\n'
                               'while read -r process_id; do kill -TERM "$process_id"; done < "$group/cgroup.procs"  # process-safety:own-scope 说是核过了但没核\n'),
        "wrong-kind.sh": '#!/usr/bin/env bash\n# singlefs-memory-cap-*.scope\nkill -TERM -- -"$job"  # process-safety:own-scope 进程组没有例外\n',
        "stale-annotation.sh": '#!/usr/bin/env bash\n# singlefs-memory-cap-*.scope\necho hi  # process-safety:own-scope 这一行没有发信号\n',
        "clean.sh": ('#!/usr/bin/env bash\nsleep 30 &\nworker=$!\nkill -0 "$worker" && kill -TERM "$worker"\n'
                     'while kill -0 "$worker" 2>/dev/null; do sleep 1; done\npython3 .claude/singlefs-ai-sop/scripts/proc.py stop "$worker"\n'
                     'for slice in "${slices[@]}"; do systemctl --user stop "$slice"; done\n'
                     "python3 - <<'EOF'\nimport os\nos.kill(int(open('worker.pid').read()), 15)\nEOF\n"),
        "bad.py": "import os, signal\nfor pid in pids:\n    os.kill(pid, signal.SIGKILL)\nos.killpg(0, signal.SIGTERM)\n",
        "good.py": "import os\nos.kill(pid, 0)\nprocess.kill()\n",
        "docs.sh": "#!/usr/bin/env bash\n# 那一行写 `# process-safety:own-scope <怎么核的>`\ncat > note.md <<'EOF'\nkill 0  # process-safety:own-scope 写进文件的正文里的不算\nEOF\n",
        "docs.py": 'NOTE = "kill 0  # process-safety:own-scope 写在字符串里的不算"\n',
    }
    for file_name, content in scan_files.items():
        with open(os.path.join(scan_scripts_directory, file_name), "w", encoding="utf-8") as handle:
            handle.write(content)
    def scan(root):
        return subprocess.run(["bash", os.path.join(hook_dir, "bash-command-detector.sh"), "--scan-scripts", root, os.path.join(root, "research", "scripts")],
                              capture_output=True, text=True)
    scanned = scan(scan_root)
    results.append(("扫脚本:有红就退出码 1", 1, scanned.returncode))
    for label, marker in [("自己任务的进程组报在第 5 行", "research/scripts/group.sh:5"), ("循环里 kill 报在第 3 行", "research/scripts/loop.sh:3"),
                          ("标了 own-scope 而没核 scope", "research/scripts/storm-unguarded.sh:3"), ("进程组标 own-scope 不放行", "research/scripts/wrong-kind.sh:3"),
                          ("不起作用的 own-scope 标注", "research/scripts/stale-annotation.sh:3"), ("python 循环里 os.kill", "research/scripts/bad.py:3"),
                          ("python os.killpg", "research/scripts/bad.py:4")]:
        results.append((f"扫脚本:{label}", 1, int(f"✗ {marker} " in scanned.stdout)))
    results.append(("扫脚本:核过 scope 又标了的、点名一个的、探活的、python 探活的都不报", 0,
                    int(any(f"research/scripts/{name}" in scanned.stdout for name in ("storm-guarded.sh", "clean.sh", "good.py", "docs.sh", "docs.py")))))
    results.append(("扫脚本:拒绝带出路", 1, int("→ 怎么办：只停自己起的、点名的那一个" in scanned.stdout)))
    pending_path = os.path.join(scan_root, ".claude", "process-safety-pending")
    for file_name in ("storm-unguarded.sh", "wrong-kind.sh", "stale-annotation.sh"):
        os.remove(os.path.join(scan_scripts_directory, file_name))
    with open(pending_path, "w", encoding="utf-8") as handle:
        handle.write("# 待改\nresearch/scripts/group.sh  # 自检：别人正在改\nresearch/scripts/loop.sh  # 自检：别人正在改\nresearch/scripts/bad.py  # 自检：别人正在改\n")
    pending_green = scan(scan_root)
    results.append(("扫脚本:红的都登记进待改清单就退出码 0", 0, pending_green.returncode))
    results.append(("扫脚本:成功行报查了几个、放行了哪一处、逐个列出没判的", 1,
                    int("查了 8 个脚本（.sh 5 个、.py 3 个）" in pending_green.stdout and "research/scripts/storm-guarded.sh:4" in pending_green.stdout
                        and "research/scripts/group.sh（research/scripts/group.sh:5）" in pending_green.stdout)))
    with open(pending_path, "a", encoding="utf-8") as handle:
        handle.write("research/scripts/clean.sh  # 自检：干净的也登记了\nresearch/scripts/gone.sh  # 自检：指向不存在的文件\n")
    pending_stale = scan(scan_root)
    results.append(("扫脚本:待改清单里一处都没排到的与指向不存在的判红", 1,
                    int(pending_stale.returncode == 1 and "clean.sh 里一处要放行的都没有" in pending_stale.stdout and "gone.sh 指向不存在的文件" in pending_stale.stdout)))
    # 整份覆盖 research/results/ 下未跟踪的产物：在临时 git 仓里造已跟踪与未跟踪的文件，判定与真实入口都指着它，不碰真仓
    repository = os.path.realpath(os.path.join(work, "repository"))
    os.makedirs(os.path.join(repository, "research", "results", "sub"))
    elsewhere = os.path.join(work, "elsewhere")        # 仓外同样的相对路径：cd 过去之后写的是它，不是仓里那份
    sources = os.path.join(work, "sources")            # cp 进目录时的源，文件名与仓里未跟踪的那份相同
    not_a_repository = os.path.join(work, "not-a-repository")
    for directory in (os.path.join(elsewhere, "research", "results"), sources, os.path.join(not_a_repository, "research", "results")):
        os.makedirs(directory)
    for path in (*(os.path.join(repository, relative) for relative in (
                     "research/results/untracked.out", "research/results/tracked.out", "research/results/1",
                     "research/results/sub/deep.out", "scratch.log")),
                 os.path.join(elsewhere, "research", "results", "untracked.out"), os.path.join(sources, "untracked.out"),
                 os.path.join(work, "outside.log"), os.path.join(not_a_repository, "research", "results", "untracked.out")):
        with open(path, "w", encoding="utf-8") as handle:
            handle.write("旧字节\n")
    git_ready = (subprocess.run(["git", "init", "-q", repository], capture_output=True).returncode == 0
                 and subprocess.run(["git", "-C", repository, "add", "research/results/tracked.out"], capture_output=True).returncode == 0)
    results.append(("覆盖产物:临时 git 仓建得起来（git init、git add）", 1, int(git_ready)))
    outside = os.path.join(work, "outside.log")
    # (说明, 命令, 该不该拒绝)；目标按临时仓的根解析
    results_overwrite_cases = [
        ("> 覆盖未跟踪的产物", "python3 x.py > research/results/untracked.out", 1),
        ("tee 不带 -a", "cargo run --release | tee research/results/untracked.out", 1),
        ("cp 的目标（现场原句的形态）", "cp /tmp/claude-1000/e158-s9/config-evidence-runs/yi-ring-retained.out research/results/untracked.out", 1),
        ("bash -c 里的 >", "bash -c 'python3 x.py > research/results/untracked.out'", 1),
        (">|", "python3 x.py >| research/results/untracked.out", 1),
        ("&>", "cargo test &> research/results/untracked.out", 1),
        ("2> 同样截断", "cargo test 2> research/results/untracked.out", 1),
        (">& 后面跟文件名", "cargo test >& research/results/untracked.out", 1),
        ("tee -i 不是追加", "make | tee -i research/results/untracked.out", 1),
        ("tee 写两份、其中一份是未跟踪的", "make | tee research/results/fresh.out research/results/untracked.out", 1),
        ("mv 的目标", "mv /tmp/fresh.out research/results/untracked.out", 1),
        ("install 的目标", "install -m 644 fresh.out research/results/untracked.out", 1),
        ("cp 进已有目录、源的文件名撞上", f"cp {sources}/untracked.out research/results/", 1),
        ("cp -t 目录", f"cp -t research/results {sources}/untracked.out", 1),
        ("cp 源里的通配按这一刻展开", f"cp {sources}/*.out research/results", 1),
        ("cd 进去之后写相对路径", "cd research/results && python3 x.py > untracked.out", 1),
        ("cd 进子目录之后写", "cd research/results/sub && echo > deep.out", 1),
        ("绝对路径", f"echo > {repository}/research/results/untracked.out", 1),
        ("喂给 bash 的 heredoc 正文里的 >", "bash <<'EOF'\npython3 x.py > research/results/untracked.out\nEOF", 1),
        ("命令替换里的 >", "lines=$(python3 x.py > research/results/untracked.out; echo done)", 1),
        ("cat 把 heredoc 写进未跟踪的产物", "cat > research/results/untracked.out <<'EOF'\n一行\nEOF", 1),
        ("花括号整组重定向", "{ echo a; echo b; } > research/results/untracked.out", 1),
        ("圆括号整组重定向", "(echo a; echo b) > research/results/untracked.out", 1),
        ("timeout、capped.sh 包着的 bash -c 整条重定向", "timeout 60 bash research/scripts/capped.sh 4 bash -c 'cargo run' > research/results/untracked.out", 1),
        ("前面赋过值的变量代入", 'out=research/results/untracked.out; python3 x.py > "$out"', 1),
        ("export 过的变量代入", 'export OUT=research/results/untracked.out; python3 x.py > "${OUT}"', 1),
        ("cd 到变量路径算不出来、按仓库根解析", 'cd "$scratch" && python3 x.py > research/results/untracked.out', 1),
        ("进程替换里的 tee", "make > >(tee research/results/untracked.out)", 1),
        ("整个循环重定向", "for arm in a b; do python3 x.py $arm; done > research/results/untracked.out", 1),
        ("sudo 包着的 tee", "echo x | sudo tee research/results/untracked.out", 1),
        ("git add 的是另一份，这一份照拒", "git add research/results/tracked.out && python3 x.py > research/results/untracked.out", 1),
        (">> 追加", "python3 x.py >> research/results/untracked.out", 0),
        ("&>> 追加", "cargo test &>> research/results/untracked.out", 0),
        ("写新文件名", "python3 x.py > research/results/untracked-2026-09-25.out", 0),
        ("写已跟踪的文件（git 兜得住）", "python3 x.py > research/results/tracked.out", 0),
        ("/tmp 目标（已存在、不在任何仓里）", f"python3 x.py > {outside}", 0),
        ("仓里 research/results/ 外面的未跟踪文件", "python3 x.py > scratch.log", 0),
        ("2>&1 不是写一个叫 1 的文件（research/results/1 未跟踪地存在）", "cd research/results && python3 x.py >> untracked.out 2>&1", 0),
        ("tee -a", "make | tee -a research/results/untracked.out", 0),
        ("tee --append", "make | tee --append research/results/untracked.out", 0),
        ("cp -n 不盖已有的", "cp -n fresh.out research/results/untracked.out", 0),
        ("cp --backup 旧的留成备份", "cp --backup=numbered fresh.out research/results/untracked.out", 0),
        ("install -d 建目录", "install -d research/results/untracked.out", 0),
        ("cp 的源是产物、目标在外面", "cp research/results/untracked.out /tmp/copy.out", 0),
        ("读产物、写别处", "sort < research/results/untracked.out > /tmp/sorted.out", 0),
        ("cd 到仓外之后同样的相对路径", f"cd {elsewhere} && python3 x.py > research/results/untracked.out", 0),
        ("同一条命令里先挪到带日期的旧名再写", "mv research/results/untracked.out research/results/untracked-2026-09-24-old.out && python3 x.py > research/results/untracked.out", 0),
        ("同一条命令里先 git add 再写", "git add research/results/untracked.out && python3 x.py > research/results/untracked.out", 0),
        ("引号里当数据写", "echo 'python3 x.py > research/results/untracked.out'", 0),
        ("写进笔记的 heredoc 正文", "cat > note.md <<'EOF'\npython3 x.py > research/results/untracked.out\nEOF", 0),
        ("注释里写的", "ls  # > research/results/untracked.out", 0),
        ("目标里有算不出的变量：不拦（只记检出）", "for arm in a b; do python3 x.py > research/results/untracked$arm.out; done", 0),
        ("目标里有命令替换、对不上已有文件", "python3 x.py > research/results/untracked-$(date +%F).out", 0),
    ]
    for label, command, want in results_overwrite_cases:
        results.append((f"覆盖产物:{label}", want, 1 if git_ready and results_overwrite_refusal(command, repository)[0] else 0))
    # 只记检出、不拦：(说明, 命令, 仓库根, 该不该记)
    results_noted_cases = [
        ("算不出的变量对得上一份未跟踪的产物", "for arm in a b; do python3 x.py > research/results/untracked$arm.out; done", repository, 1),
        ("变量在前、research/results/ 在后", 'python3 x.py > "$CLAUDE_PROJECT_DIR"/research/results/untracked.out', repository, 1),
        ("算不出的段对不上任何已有文件（一定是新文件名）", "for arm in a b; do python3 x.py > research/results/e-$arm.out; done", repository, 0),
        ("命令替换当日期、对不上已有文件", "python3 x.py > research/results/untracked-$(date +%F).out", repository, 0),
        ("对得上的只有已跟踪的", "for arm in a b; do python3 x.py > research/results/tracked$arm.out; done", repository, 0),
        ("git 判不了（不是 git 仓）只记不拦", "python3 x.py > research/results/untracked.out", not_a_repository, 1),
        ("拒了的不另记", "python3 x.py > research/results/untracked.out", repository, 0),
    ]
    for label, command, root, want in results_noted_cases:
        refused_here, noted_here = results_overwrite_refusal(command, root) if git_ready else ([], [])
        results.append((f"覆盖产物检出:{label}", want, int(bool(noted_here))))
    results.append(("覆盖产物检出:git 判不了的那一条不拦", 0, int(bool(results_overwrite_refusal("python3 x.py > research/results/untracked.out", not_a_repository)[0]))))
    # ⑦ 在原 inode 上改已有脚本：在临时仓里造 .sh、.py、带执行位没后缀的、指向脚本的符号链接与不是脚本的文件，scratch 根换成临时目录；只判不执行
    scratch_root = os.path.join(work, "scratch-root")
    os.makedirs(os.path.join(repository, "research", "scripts"))
    os.makedirs(os.path.join(repository, "bin"))
    os.makedirs(os.path.join(scratch_root, "probe"))
    for relative, mode in (("research/scripts/running.sh", 0o755), ("research/scripts/tool.py", 0o644), ("bin/runner", 0o755),
                           ("notes.md", 0o644), ("data.txt", 0o644)):
        with open(os.path.join(repository, relative), "w", encoding="utf-8") as handle:
            handle.write("echo 旧内容\n")
        os.chmod(os.path.join(repository, relative), mode)
    os.symlink("running.sh", os.path.join(repository, "research", "scripts", "link.sh"))
    for path in (os.path.join(scratch_root, "probe", "step.sh"), os.path.join(work, "outside.sh")):
        with open(path, "w", encoding="utf-8") as handle:
            handle.write("echo 旧内容\n")
        os.chmod(path, 0o755)
    running = "research/scripts/running.sh"
    # (说明, 命令, 该不该拒绝)；目标按临时仓的根解析
    script_in_place_cases = [
        ("> 覆盖已有的 .sh", f"echo x > {running}", 1),
        (">|", f"echo x >| {running}", 1),
        ("&>", f"make &> {running}", 1),
        ("2> 同样截断", f"make 2> {running}", 1),
        (">& 后面跟文件名", f"make >& {running}", 1),
        ("cat > 旧 <<EOF", f"cat > {running} <<'EOF'\n#!/usr/bin/env bash\necho 新内容\nEOF", 1),
        ("tee 不带 -a", f"echo x | tee {running}", 1),
        ("tee -i 不是追加", f"echo x | tee -i {running}", 1),
        ("cp 新 旧", f"cp /tmp/new.sh {running}", 1),
        ("cp -f", f"cp -f /tmp/new.sh {running}", 1),
        ("cp -a", f"cp -a /tmp/new.sh {running}", 1),
        ("cp 进已有目录、文件名撞上", f"cp {sources}/running.sh research/scripts/", 1),
        ("cp -t 目录", f"cp -t research/scripts {sources}/running.sh", 1),
        ("dd of=", f"dd if=/tmp/new.sh of={running}", 1),
        ("dd conv=notrunc", f"dd if=/tmp/new.sh of={running} conv=notrunc", 1),
        ("truncate -s 0", f"truncate -s 0 {running}", 1),
        ("truncate -cs 0（选项连写）", "truncate -cs 0 research/scripts/tool.py", 1),
        ("写已有的 .py", "echo x > research/scripts/tool.py", 1),
        ("带执行位、没有后缀的", "cp /tmp/new bin/runner", 1),
        ("经符号链接写它指向的脚本", "echo x > research/scripts/link.sh", 1),
        ("scratch 根下的脚本", f"cp /tmp/new.sh {scratch_root}/probe/step.sh", 1),
        ("cd 进去之后写相对路径", "cd research/scripts && echo x > running.sh", 1),
        ("bash -c 里的 >", f"bash -c 'echo x > {running}'", 1),
        ("喂给 bash 的 heredoc 正文里的 cp", f"bash <<'EOF'\ncp /tmp/new.sh {running}\nEOF", 1),
        ("命令替换里的 >", f"lines=$(echo x > {running}; echo done)", 1),
        ("前面赋过值的变量代入", f'target={running}; cp /tmp/new.sh "$target"', 1),
        ("nice、timeout 包着的 cp", f"timeout 60 nice -n 19 cp /tmp/new.sh {running}", 1),
        ("python3 -c 的 open(…, 'w')", f"python3 -c \"open('{running}', 'w').write('x')\"", 1),
        ("python3 -c 的 open(…, 'r+')", f"python3 -c \"open('{running}', 'r+').write('x')\"", 1),
        ("python3 -c 的 open(…, mode='wb')", "python3 -c \"open('research/scripts/tool.py', mode='wb').write(b'x')\"", 1),
        ("喂给 python 的 heredoc：只赋过一次的名字", f"python3 - <<'EOF'\npath = '{running}'\nwith open(path, 'w') as handle:\n    handle.write('x')\nEOF", 1),
        ("喂给 python 的 heredoc：Path(…) / '…' 的 write_text", "python3 - <<'EOF'\nfrom pathlib import Path\n(Path('research') / 'scripts' / 'tool.py').write_text('x')\nEOF", 1),
        ("喂给 python 的 heredoc：pathlib.Path(…).write_bytes", f"python3 <<'EOF'\nimport pathlib\npathlib.Path('{running}').write_bytes(b'x')\nEOF", 1),
        ("喂给 python 的 heredoc：Path(…).open('w')", f"python3 - <<'EOF'\nimport pathlib\nwith pathlib.Path('{running}').open('w') as handle:\n    handle.write('x')\nEOF", 1),
        ("喂给 python 的 heredoc：os.path.join 与 io.open", "python3 - <<'EOF'\nimport io, os\nio.open(os.path.join('research', 'scripts', 'tool.py'), 'w')\nEOF", 1),
        ("喂给 python 的 heredoc：shutil.copyfile", f"python3 - <<'EOF'\nimport shutil\nshutil.copyfile('/tmp/new.sh', '{running}')\nEOF", 1),
        ("喂给 python 的 heredoc：shutil.copy 进已有目录", "python3 - <<'EOF'\nimport shutil\nshutil.copy('/tmp/running.sh', 'research/scripts')\nEOF", 1),
        ("cd 之后喂给 python 的相对路径", "cd research/scripts && python3 - <<'EOF'\nopen('running.sh', 'w').write('x')\nEOF", 1),
        ("bash -c 里的 python3 -c", f"bash -c \"python3 -c \\\"open('{running}', 'w')\\\"\"", 1),
        (">> 追加", f"echo x >> {running}", 0),
        ("&>> 追加", f"make &>> {running}", 0),
        ("tee -a", f"echo x | tee -a {running}", 0),
        ("python 的 'a' 追加", f"python3 -c \"open('{running}', 'a').write('x')\"", 0),
        ("python 的 'x' 只建新文件", f"python3 -c \"open('{running}', 'x')\"", 0),
        ("python 读", f"python3 -c \"print(open('{running}').read())\"", 0),
        ("写新文件名", "echo x > research/scripts/new-step.sh", 0),
        ("写到同目录临时文件再 mv 换上", f"cat > research/scripts/.running.sh.new <<'EOF'\necho 新内容\nEOF\nchmod 755 research/scripts/.running.sh.new && mv research/scripts/.running.sh.new {running}", 0),
        ("cp 成隐藏文件、chmod、mv 换上", f"cp /tmp/new.sh research/scripts/.running.sh.installing && chmod 755 research/scripts/.running.sh.installing && mv research/scripts/.running.sh.installing {running}", 0),
        ("mv 新 旧", f"mv /tmp/new.sh {running}", 0),
        ("install 换 inode", f"install -m 755 /tmp/new.sh {running}", 0),
        ("cp --remove-destination 换 inode", f"cp --remove-destination /tmp/new.sh {running}", 0),
        ("cp -l -f 换成硬链接", f"cp -l -f /tmp/new.sh {running}", 0),
        ("cp -n 不盖已有的", f"cp -n /tmp/new.sh {running}", 0),
        ("cp -b 旧的留成备份", f"cp -b /tmp/new.sh {running}", 0),
        ("sed -i 换 inode", f"sed -i 's/旧/新/' {running}", 0),
        ("同一条命令里先 rm 再写", f"rm {running} && echo x > {running}", 0),
        ("同一条命令里先挪走再写", f"mv {running} {running}.old && echo x > {running}", 0),
        ("python 写到临时文件再 os.replace", f"python3 - <<'EOF'\nimport os\nopen('research/scripts/.running.sh.new', 'w').write('x')\nos.replace('research/scripts/.running.sh.new', '{running}')\nEOF", 0),
        ("写已有的非脚本文件（.md）", "echo x > notes.md", 0),
        ("写已有的、不带执行位的 .txt", "echo x > data.txt", 0),
        ("仓外、scratch 根外的脚本", f"echo x > {work}/outside.sh", 0),
        ("读脚本、写别处", f"cat {running} > {work}/copy.txt", 0),
        ("cp 脚本出去", f"cp {running} {work}/copy.sh", 0),
        ("引号里当数据写", f"echo 'cp /tmp/new.sh {running}'", 0),
        ("写进笔记的 heredoc 正文", f"cat > {work}/note.md <<'EOF'\necho x > {running}\nEOF", 0),
        ("注释里写的", f"ls  # cp /tmp/new.sh {running}", 0),
        ("目标里有算不出的变量（判不到）", 'cp /tmp/new.sh "$target"', 0),
        ("python 的路径从 sys.argv 来（判不到）", f"python3 -c \"import sys; open(sys.argv[1], 'w')\" {running}", 0),
        ("python 的名字赋过两次（判不到）", f"python3 - <<'EOF'\npath = 'notes.md'\npath = '{running}'\nopen(path, 'w')\nEOF", 0),
        ("python 跑脚本文件、喂给它的 heredoc 是数据", f"python3 tool.py <<'EOF'\nopen('{running}', 'w')\nEOF", 0),
    ]
    for label, command, want in script_in_place_cases:
        results.append((f"就地改脚本:{label}", want, 1 if script_in_place_refusal(command, repository, scratch_root) else 0))
    results.append(("就地改脚本:拒绝时报出写法与相对仓库根的路径", 1,
                    int(script_in_place_refusal(f"cp /tmp/new.sh {running}", repository, scratch_root) == [f"cp {running}"])))
    results.append(("就地改脚本:⑤ 不认 dd、truncate、rm 与 python 的写法（覆盖未跟踪产物照旧只看 ⑤ 那几种）", 0,
                    int(bool(results_overwrite_refusal("dd if=x of=research/results/untracked.out; truncate -s 0 research/results/untracked.out; "
                                                       "python3 -c \"open('research/results/untracked.out', 'w')\"", repository)[0])) if git_ready else 0))
    results.append(("就地改脚本:rm 掉再写未跟踪产物，⑤ 照拒（rm 没留住旧字节）", 1,
                    int(bool(results_overwrite_refusal("rm research/results/untracked.out && echo x > research/results/untracked.out", repository)[0])) if git_ready else 1))
    # 走真实入口：把 hook 与共用切词模块拷进临时仓的 .claude/hooks/，仓库根取它往上两级
    repository_hooks = os.path.join(repository, ".claude", "hooks")
    os.makedirs(repository_hooks)
    shutil.copy(os.path.join(hook_dir, "bash-command-detector.sh"), repository_hooks)
    shutil.copy(shell_words_library_path(hook_dir), repository_hooks)
    def through_entry(command, run_in_background=False):
        before_entry = sum(1 for _ in open(detections)) if os.path.exists(detections) else 0
        completed = subprocess.run(["bash", os.path.join(repository_hooks, "bash-command-detector.sh")], capture_output=True, text=True,
                                   env=dict(os.environ, AGENT_HOOK_DETECTIONS=detections),
                                   input=json.dumps({"tool_name": "Bash", "session_id": "s", "tool_input": {
                                       "command": command, "run_in_background": run_in_background}}))
        after_entry = sum(1 for _ in open(detections)) if os.path.exists(detections) else 0
        return completed, after_entry - before_entry
    scene, scene_recorded = through_entry("cp /tmp/claude-1000/e158-s9/config-evidence-runs/yi-ring-retained.out research/results/untracked.out")
    results.append(("stdin:cp 覆盖未跟踪的产物拒绝（退出码 2）", 2, scene.returncode))
    results.append(("stdin:拒绝时 stderr 写了出路（按日期另存、先 git add 或挪到带日期的旧名）", 1,
                    int("✗" in scene.stderr and "→" in scene.stderr and "research/results/untracked.out" in scene.stderr
                        and "按日期另存一个新文件名，旧的留着；确实要换掉旧产物，先 `git add` 它或者把它挪到带日期的旧名" in scene.stderr)))
    results.append(("stdin:拒了的不记检出", 0, scene_recorded))
    background_overwrite = through_entry("python3 x.py > research/results/untracked.out", True)[0]
    results.append(("stdin:run_in_background 里的 > 覆盖同样拒绝（退出码 2）", 2, background_overwrite.returncode))
    appended, appended_recorded = through_entry("python3 x.py >> research/results/untracked.out")
    results.append(("stdin:>> 追加放行（退出码 0）、不记检出", (0, 0), (appended.returncode, appended_recorded)))
    unresolved, unresolved_recorded = through_entry("for arm in a b; do python3 x.py > research/results/untracked$arm.out; done")
    results.append(("stdin:目标算不出、对得上未跟踪产物的放行（退出码 0）并记一条检出", (0, 1), (unresolved.returncode, unresolved_recorded)))
    # 走真实入口：在原 inode 上改临时仓里的已有脚本拒绝（退出码 2）、stderr 给出 mv 换上与定点改两条出路、不记检出；写到临时文件再 mv 换上的放行
    in_place, in_place_recorded = through_entry(f"cp /tmp/new.sh {running}")
    results.append(("stdin:cp 覆盖已有脚本拒绝（退出码 2）", 2, in_place.returncode))
    results.append(("stdin:拒绝时 stderr 写了出路（临时文件再 mv 换上、replace-once.py / insert-row.py 定点改）", 1,
                    int("✗" in in_place.stderr and "→" in in_place.stderr and f"cp {running}" in in_place.stderr
                        and "写到同目录临时文件再 `mv` 换上" in in_place.stderr and "research/scripts/replace-once.py" in in_place.stderr)))
    results.append(("stdin:就地改脚本拒了的不记检出", 0, in_place_recorded))
    python_in_place = through_entry(f"python3 - <<'EOF'\nopen('{running}', 'w').write('x')\nEOF", True)[0]
    results.append(("stdin:run_in_background 里喂给 python 的 open(…, 'w') 同样拒绝（退出码 2）", 2, python_in_place.returncode))
    renamed, renamed_recorded = through_entry(f"cp /tmp/new.sh research/scripts/.running.sh.installing && mv research/scripts/.running.sh.installing {running}")
    results.append(("stdin:写到临时文件再 mv 换上放行（退出码 0）、不记检出", (0, 0), (renamed.returncode, renamed_recorded)))
    # 同走 run_in_background 的那一条：只记检出
    results.append(("等待循环:同一条走 run_in_background 记一条检出", 1,
                    record({"tool_name": "Bash", "session_id": "s", "tool_input": {"command": "until grep -q x f; do sleep 10; done", "run_in_background": True}},
                           detections)))
    # 走真实入口：从标准输入喂 JSON，起看门狗的写法不对退出码 2、stderr 给出路，写法对的退出码 0
    script_for_watchdog = os.path.join(hook_dir, "bash-command-detector.sh")
    watchdog_environment = dict(os.environ, AGENT_HOOK_DETECTIONS=detections)
    foreground_watchdog = subprocess.run(["bash", script_for_watchdog], capture_output=True, text=True, env=watchdog_environment,
                                         input=json.dumps({"tool_name": "Bash", "session_id": "s", "tool_input": {"command": "bash research/scripts/watch.sh a1 > /dev/null 2>&1 &"}}))
    results.append(("stdin:前台起看门狗拒绝（退出码 2）", 2, foreground_watchdog.returncode))
    results.append(("stdin:拒绝时 stderr 写了出路（run_in_background: true 的起法）", 1,
                    int("run_in_background: true" in foreground_watchdog.stderr and "→" in foreground_watchdog.stderr)))
    background_watchdog = subprocess.run(["bash", script_for_watchdog], capture_output=True, text=True, env=watchdog_environment,
                                         input=json.dumps({"tool_name": "Bash", "session_id": "s", "tool_input": {"command": "bash research/scripts/watch.sh a1", "run_in_background": True}}))
    results.append(("stdin:run_in_background 起看门狗放行（退出码 0）", 0, background_watchdog.returncode))
    # 走真实入口：从标准输入喂 JSON，run_in_background 里没超时的等待循环退出码必须是 0（不拦），检出要落进文件
    script = os.path.join(hook_dir, "bash-command-detector.sh")
    before = sum(1 for _ in open(detections)) if os.path.exists(detections) else 0
    waiting = subprocess.run(["bash", script], input=json.dumps({"tool_name": "Bash", "session_id": "s", "tool_input": {
                                 "command": "until grep -q done log; do sleep 10; done", "run_in_background": True}}),
                             capture_output=True, text=True, env=dict(os.environ, AGENT_HOOK_DETECTIONS=detections))
    after = sum(1 for _ in open(detections)) if os.path.exists(detections) else 0
    results.append(("stdin:run_in_background 里没超时的等待循环只记不拦（退出码 0）", 0, waiting.returncode))
    results.append(("stdin:检出落进检出记录", 1, after - before))
    # 走真实入口：前台没超时的等待循环拒绝（退出码 2）、不记检出，stderr 给出两种写法；套上 timeout 的放行（退出码 0）
    foreground_wait = subprocess.run(["bash", script], input=json.dumps({"tool_name": "Bash", "session_id": "s", "tool_input": {
                                         "command": "until grep -q x f; do sleep 10; done"}}),
                                     capture_output=True, text=True, env=dict(os.environ, AGENT_HOOK_DETECTIONS=detections))
    results.append(("stdin:前台没超时的等待循环拒绝（退出码 2）", 2, foreground_wait.returncode))
    results.append(("stdin:拒绝时 stderr 写了两种写法（run_in_background 与 proc.py wait --timeout）", 1,
                    int("✗" in foreground_wait.stderr and "→" in foreground_wait.stderr and "run_in_background: true" in foreground_wait.stderr
                        and "proc.py wait <pid> --timeout <秒>" in foreground_wait.stderr)))
    bounded_wait = subprocess.run(["bash", script], input=json.dumps({"tool_name": "Bash", "session_id": "s", "tool_input": {
                                      "command": "timeout 60 bash -c 'until grep -q x f; do sleep 10; done'"}}),
                                  capture_output=True, text=True, env=dict(os.environ, AGENT_HOOK_DETECTIONS=detections))
    results.append(("stdin:套上 timeout 的前台等待循环放行（退出码 0）", 0, bounded_wait.returncode))
    results.append(("stdin:前台拒了的与套上 timeout 的都不记检出", after, sum(1 for _ in open(detections)) if os.path.exists(detections) else 0))
    # 走真实入口：run_in_background 里 & disown 拒绝（退出码 2），stderr 给出 run_in_background 的起法
    detached = subprocess.run(["bash", script], input=json.dumps({"tool_name": "Bash", "session_id": "s", "tool_input": {
                                  "command": "nice -n 19 bash long.sh > long.log 2>&1 & disown 2>/dev/null || true & wait", "run_in_background": True}}),
                              capture_output=True, text=True, env=dict(os.environ, AGENT_HOOK_DETECTIONS=detections))
    results.append(("stdin:run_in_background 里 & disown 拒绝（退出码 2）", 2, detached.returncode))
    results.append(("stdin:放出追踪的拒绝 stderr 写了 run_in_background 的起法", 1,
                    int("✗" in detached.stderr and "→" in detached.stderr and "run_in_background: true" in detached.stderr)))
    # 走真实入口：进程替换 `<(…)` 里的命令另按一条完整的命令判，`)` 之后的词是外层命令的参数；现场误拒的原句放行、不记检出，括号里是前台没超时的等待循环的拒绝（退出码 2）
    before_substitution = sum(1 for _ in open(detections)) if os.path.exists(detections) else 0
    substitution_original = subprocess.run(["bash", script], input=json.dumps({"tool_name": "Bash", "session_id": "s", "tool_input": {
                                               "command": "diff <(git show HEAD:.claude/scripts/check.sh) .claude/scripts/check.sh"}}),
                                           capture_output=True, text=True, env=dict(os.environ, AGENT_HOOK_DETECTIONS=detections))
    results.append(("stdin:进程替换的现场原句 diff <(git show …) check.sh 放行（退出码 0）", 0, substitution_original.returncode))
    results.append(("stdin:进程替换的现场原句不记检出", before_substitution, sum(1 for _ in open(detections)) if os.path.exists(detections) else 0))
    substitution_loop = subprocess.run(["bash", script], input=json.dumps({"tool_name": "Bash", "session_id": "s", "tool_input": {
                                           "command": "diff <(until grep -q x f; do sleep 10; done; cat f) x"}}),
                                       capture_output=True, text=True, env=dict(os.environ, AGENT_HOOK_DETECTIONS=detections))
    results.append(("stdin:进程替换里前台没超时的等待循环拒绝（退出码 2）", 2, substitution_loop.returncode))
    # 走真实入口：run_in_background 里后面没有 wait 的单独 & 拒绝（退出码 2），stderr 写两条出路；前台的同一条放行（退出码 0）；两条都不记检出
    before_ampersand = sum(1 for _ in open(detections)) if os.path.exists(detections) else 0
    unwaited = subprocess.run(["bash", script], input=json.dumps({"tool_name": "Bash", "session_id": "s", "tool_input": {
                                  "command": "bash long.sh > long.log 2>&1 & echo $!", "run_in_background": True}}),
                              capture_output=True, text=True, env=dict(os.environ, AGENT_HOOK_DETECTIONS=detections))
    results.append(("stdin:run_in_background 里后面没有 wait 的单独 & 拒绝（退出码 2）", 2, unwaited.returncode))
    results.append(("stdin:拒绝时 stderr 写了两条出路（run_in_background 里不加 & 与 proc.py wait <pid> --timeout <秒>）", 1,
                    int("✗" in unwaited.stderr and "→" in unwaited.stderr and "run_in_background" in unwaited.stderr
                        and "不加 `&`" in unwaited.stderr and "proc.py wait <pid> --timeout <秒>" in unwaited.stderr)))
    foreground_ampersand = subprocess.run(["bash", script], input=json.dumps({"tool_name": "Bash", "session_id": "s", "tool_input": {
                                              "command": "bash long.sh > long.log 2>&1 & echo $!"}}),
                                          capture_output=True, text=True, env=dict(os.environ, AGENT_HOOK_DETECTIONS=detections))
    results.append(("stdin:前台的同一条放行（退出码 0）", 0, foreground_ampersand.returncode))
    results.append(("stdin:拒了的与前台的单独 & 都不记检出", before_ampersand, sum(1 for _ in open(detections)) if os.path.exists(detections) else 0))
    # 走真实入口：终止进程的写法拒绝（退出码 2）、stderr 给出按 pid 停一个的出路、不记检出；点名一个的放行。命令只交给 hook 判，一条都不执行
    before_signal = sum(1 for _ in open(detections)) if os.path.exists(detections) else 0
    def signal_entry(command, run_in_background=False):
        return subprocess.run(["bash", script], input=json.dumps({"tool_name": "Bash", "session_id": "s", "tool_input": {
                                  "command": command, "run_in_background": run_in_background}}),
                              capture_output=True, text=True, env=dict(os.environ, AGENT_HOOK_DETECTIONS=detections))
    group_signal = signal_entry("kill -TERM -1234")
    results.append(("stdin:kill 负进程号拒绝（退出码 2）", 2, group_signal.returncode))
    results.append(("stdin:拒绝时 stderr 写了按 pid 停一个的出路", 1,
                    int("✗" in group_signal.stderr and "→" in group_signal.stderr and "proc.py stop <pid>" in group_signal.stderr and "`$!`" in group_signal.stderr)))
    results.append(("stdin:kill 写死的 1 拒绝（退出码 2，读真的进程表）", 2, signal_entry("kill -TERM 1").returncode))
    results.append(("stdin:kill 写死的自检进程号（hook 的祖先）拒绝（退出码 2）", 2, signal_entry(f"kill -TERM {os.getpid()}").returncode))
    results.append(("stdin:run_in_background 里循环 kill 拒绝（退出码 2）", 2, signal_entry('for p in $(cat pids); do kill "$p"; done', True).returncode))
    results.append(("stdin:kill \"$!\" 放行（退出码 0）", 0, signal_entry('kill "$!"').returncode))
    results.append(("stdin:proc.py stop 一个不存在的进程号放行（退出码 0）", 0, signal_entry("python3 .claude/singlefs-ai-sop/scripts/proc.py stop 99999999").returncode))
    results.append(("stdin:终止进程拒了的与放行的都不记检出", before_signal, sum(1 for _ in open(detections)) if os.path.exists(detections) else 0))
    after = sum(1 for _ in open(detections)) if os.path.exists(detections) else 0
    # 走真实入口：hook 旁边没有共用切词模块时照常放行（退出码 0，前台起看门狗也不拦），stderr 点名那个模块，并记一条检出
    lonely_hook_directory = os.path.join(work, "hook-without-shared-words")
    os.makedirs(lonely_hook_directory)
    shutil.copy(script, lonely_hook_directory)
    without_library = subprocess.run(["bash", os.path.join(lonely_hook_directory, "bash-command-detector.sh")], capture_output=True, text=True,
                                     env=dict(os.environ, AGENT_HOOK_DETECTIONS=detections),
                                     input=json.dumps({"tool_name": "Bash", "session_id": "s", "tool_input": {"command": "bash research/scripts/watch.sh a1 > /dev/null 2>&1 &"}}))
    recorded_after = sum(1 for _ in open(detections)) if os.path.exists(detections) else 0
    results.append(("stdin:读不到共用切词模块照常放行（退出码 0）", 0, without_library.returncode))
    results.append(("stdin:读不到共用切词模块时 stderr 点名它、记一条检出", 1,
                    int("lib_shell_words.py" in without_library.stderr and recorded_after == after + 1)))
    subprocess.run(["rm", "-rf", work])
    failures = [item for item in results if item[1] != item[2]]
    for label, want, got in failures:
        print(f"  ✗ 自检：{label} 应当是 {want}，实际 {got}")  # gate-lint:detail
    if failures:
        print("    → 看 findings_for()、record()、watchdog_rejection()、wait_loop_refusal() / unbounded_wait_loops()、detaching_refusal() / detaching_forms()、"
              "unwaited_ampersand_refusal() / unwaited_background_jobs() / jobs_with_endings()、"
              "results_overwrite_refusal() / results_overwrite_verdict() / overwrite_steps() / follow_simple_command() / destination_words() 与共用的 lib_shell_words.py；"
              "「覆盖产物:临时 git 仓建得起来」红的是本机的 git 起不来，先看 git init 能不能跑；"
              "BASH_COMMAND_DETECTOR_DISABLE_CHECK、_DISABLE_SELF_BACKGROUND、_KEEP_HEREDOC_BODIES、_ALLOW_WATCHDOG_MISUSE、"
              "_ALLOW_FOREGROUND_WAIT_LOOP、_REFUSE_EVERY_WAIT_LOOP、_ALLOW_DETACHING、_ALLOW_UNWAITED_AMPERSAND、_ALLOW_RESULTS_OVERWRITE、_ALLOW_PROCESS_SIGNALS "
              "或 _ALLOW_SCRIPT_IN_PLACE_WRITE 设着的话这里本来就该红；"
              "「终止进程:」「扫脚本:」红的看 process_signal_refusal() / termination_findings() / call_findings() / scan_scripts()；"
              "「就地改脚本:」红的看 script_in_place_refusal() / script_in_place_verdict() / python_in_place_writes() / is_existing_script()")
        return 1
    summary = ("没超时的等待循环、run_in_background 里又自己放后台记进检出记录，普通命令、重定向里的 & 、前台的 & 、写进文件的 heredoc 正文与按模式找进程（归上游钩子）不记；"
               "起看门狗不用 run_in_background、或带 &、nohup、setsid、disown、丢进 /dev/null 的拒绝（退出码 2），run_in_background 只写 watch.sh 的、把名字当参数的、"
               "不盯的模式与 agent-watch.py report 放行；前台没超时的等待循环拒绝（退出码 2、不记检出），外层套了 timeout 的、run_in_background 里的（只记检出）、"
               "引号与注释里当数据写的、不带 sleep 的与 for 循环放行；命令位置上的 disown、coproc、setsid -f、nohup … &、tmux / screen 分离、"
               "不等结束的 systemd-run 前台后台都拒，不带 & 的 nohup、裸 setsid、systemd-run --wait、并行加 wait 与当数据写的放行；"
               "run_in_background 里以单独的 & 收尾、之后同一层同一对圆括号里没有 wait 的拒绝（结尾 &、& 后接 echo $!、capped.sh 包着的、"
               "子 shell、命令替换、bash -c 与喂给 shell 的 heredoc 里的，wait -n 与按 pid 轮询不算 wait），之后用 wait / wait \"$pid\" 等的、"
               "&&、2>&1、|&、&>、当数据写的与前台的放行；在临时 git 仓里，整份覆盖 research/results/ 下已存在又没进 git 的文件拒绝（>、>|、&>、2>、>& 文件、"
               "不带 -a 的 tee、cp / mv / install 的目标与 -t、写进已有目录，bash -c、heredoc、命令替换、进程替换、整组重定向里的，cd 跟着算、cd 算不出来按仓库根、"
               "前面赋过值的变量代入），>>、&>>、tee -a、新文件名、已跟踪、仓外与 research/results/ 外、2>&1、cp -n / --backup、install -d、同一条命令里先 git add 或 mv 挪走的、"
               "当数据写的放行；目标算不出而对得上未跟踪产物的、git 判不了的只记检出；"
               "终止进程：kill 负进程号、0、$PPID、一次几个目标、命令替换与整批展开、循环里逐个 kill 或 proc.py stop、xargs / find -exec 喂给 kill、pkill、killall、fuser -k、"
               "先从 cgroup.procs 或 pgrep 挑再发、python 里 os.killpg 与循环里 os.kill、systemctl 停 SSH / 会话 / user@ / user.slice / vllm-prod 或带通配、isolate 与 reboot、"
               "loginctl terminate / kill、写 cgroup.kill、写死的进程号是 sshd、VSCode、自己的 Claude、别的 Claude 会话或它起的、vllm、pid 1 的都拒（退出码 2、不记检出），"
               "kill \"$!\"、kill %1、kill -0、kill -l、这个会话起的与没人认领的写死进程号、proc.py stop 一个、自己 slice 的 systemctl stop、只读 cgroup.procs 与当数据写的放行；"
               "扫脚本：进程组、循环 kill、没核 scope 的 own-scope 标注、给进程组标的与不起作用的标注、python 循环 os.kill 与 os.killpg 按行号报红，"
               "核过 singlefs-memory-cap-*.scope 又标了的放行，待改清单里的逐个列名、清单里干净的与不存在的判红；"
               "就地改脚本：在临时仓里，已有的 .sh、.py、带执行位的与 scratch 根下的脚本被 >、>|、&>、2>、>& 文件、cat >、不带 -a 的 tee、cp（-f、-a、写进目录、-t）、"
               "dd of=、truncate、python 的 open(…, 'w' / 'r+' / mode='wb')、Path.open('w')、write_text、write_bytes、shutil.copyfile / copy 写的拒绝"
               "（bash -c、heredoc、命令替换、cd 与变量跟着算，python 的只赋过一次的名字、Path / 、os.path.join 算得出），"
               ">>、&>>、tee -a、python 'a' / 'x' / 读、新文件名、临时文件再 mv、mv、install、cp --remove-destination / -l / -n / -b、sed -i、先 rm 或挪走再写、"
               "os.replace、非脚本文件、仓外与 scratch 根外、当数据写的与算不出的放行，⑤ 不认这几种新写法")
    print(f"  ✓ 自检通过（查了 {len(results)} 种）：{summary}")
    return 0

def main():
    hook_dir = sys.argv[1]
    if len(sys.argv) > 2 and sys.argv[2] == "--selftest":
        return selftest(hook_dir)
    if len(sys.argv) > 2 and sys.argv[2] == "--scan-scripts":
        if shell_words is None:
            print(f"  ✗ 读不到共用切词模块 {shell_words_library_path(hook_dir)}（{shell_words_error!r}），脚本没扫")
            print("     → 怎么办：恢复 .claude/hooks/lib_shell_words.py（切词与认命令位置只有那一份），再跑门禁 73 号")
            return 1
        if len(sys.argv) < 5:
            print("  ✗ --scan-scripts 要仓根与至少一个目录")
            print("     → 怎么办：写成 bash .claude/hooks/bash-command-detector.sh --scan-scripts <仓根> <目录>…（门禁 73 号这样调）")
            return 2
        return scan_scripts(sys.argv[3], sys.argv[4:])
    try:
        hook_input = json.load(sys.stdin)
    except Exception:
        return 0
    if not isinstance(hook_input, dict):
        return 0
    if shell_words is None:
        warning = f"bash-command-detector.sh 读不到共用切词模块 {shell_words_library_path(hook_dir)}（{shell_words_error!r}），这条命令没判、照常执行"
        print(f"  ! {warning}", file=sys.stderr)
        try:
            append_detection(hook_input, [warning], os.environ.get("AGENT_HOOK_DETECTIONS") or DEFAULT_DETECTIONS)
        except Exception:
            pass
        return 0
    tool_input = hook_input.get("tool_input") if isinstance(hook_input.get("tool_input"), dict) else {}
    noted_overwrites = []
    try:
        reasons, calls = watchdog_rejection(tool_input.get("command") or "", tool_input.get("run_in_background"))
    except Exception as error:
        print(f"  ! bash-command-detector.sh 没判成看门狗的起法（{error!r}），这条命令照常执行", file=sys.stderr)
        reasons, calls = [], []
    if not reasons:
        try:
            signal_problems = process_signal_refusal(tool_input.get("command") or "")
        except Exception as error:
            print(f"  ! bash-command-detector.sh 没判成终止进程的写法（{error!r}），这条命令照常执行", file=sys.stderr)
            signal_problems = []
        if signal_problems:
            print(f"  ✗ 终止进程的写法打得到不是自己点名的那一个进程（{'；'.join(signal_problems)}）："
                  "后面的命令不许停掉前面的任务，不许动 SSH、VSCode 与 Claude 会话，一次只停点名的一个", file=sys.stderr)
            print("     → 怎么办：只停自己起的进程：记下它的 pid（`$!`）或任务号，一次停一个，单独 `python3 .claude/singlefs-ai-sop/scripts/proc.py stop <pid>`"
                  "（或 `kill \"$pid\"`、`kill %1`）；不带负号、不给 0、不一次给几个、不在循环里、不按名字或 cgroup 挑。"
                  "要停的不是自己起的（别的会话的、SSH、VSCode、系统服务），别停，交主 agent 问用户。这一道只拒这种写法，不停你在跑的任何东西", file=sys.stderr)
            return 2
        try:
            forms = detaching_refusal(tool_input.get("command") or "")
        except Exception as error:
            print(f"  ! bash-command-detector.sh 没判成放出追踪的写法（{error!r}），这条命令照常执行", file=sys.stderr)
            forms = []
        if forms:
            print(f"  ✗ 把活放出了追踪（认出的写法：{'、'.join(forms)}）：进程移出 shell 的作业表或另起会话，随后的 wait 立刻返回、"
                  "完成通知当场发出，它跑完不会叫醒任何人，成了没人追踪的孤儿", file=sys.stderr)
            print("     → 怎么办：要等的活用 Bash 的 run_in_background: true 起，命令里只写那条活本身（不加 `disown`、`coproc`、`setsid -f`、"
                  "`nohup … &`、`tmux` / `screen` 的分离模式、`systemd-run`），然后结束本轮，等它的完成通知再接着做；"
                  "几条活要并行就在同一条命令里 `a & b & wait`，用不带参数的 `wait` 等齐", file=sys.stderr)
            return 2
        try:
            jobs = unwaited_ampersand_refusal(tool_input.get("command") or "", tool_input.get("run_in_background"))
        except Exception as error:
            print(f"  ! bash-command-detector.sh 没判成 run_in_background 里的单独 &（{error!r}），这条命令照常执行", file=sys.stderr)
            jobs = []
        if jobs:
            print(f"  ✗ run_in_background 里又把活放到了后台（以单独的 & 收尾、之后同一条命令里没有 wait 的作业：{'；'.join(jobs)}）："
                  "外层 shell 起完它就退出，完成通知当场发出，真跑完的那个进程不会叫醒任何人", file=sys.stderr)
            print("     → 怎么办：长活直接放 run_in_background，命令里只写那条活本身，不加 `&`"
                  "（几条活要并行就在同一条命令里 `a & b & wait`，用 `wait` 等齐）；"
                  "已经在跑、pid 已知的，另起一条 run_in_background 等它：`python3 .claude/singlefs-ai-sop/scripts/proc.py wait <pid> --timeout <秒>`。"
                  "这一道只拒这种写法，不停你在跑的任何东西", file=sys.stderr)
            return 2
        try:
            loops = wait_loop_refusal(tool_input.get("command") or "", tool_input.get("run_in_background"))
        except Exception as error:
            print(f"  ! bash-command-detector.sh 没判成前台的等待循环（{error!r}），这条命令照常执行", file=sys.stderr)
            loops = []
        if loops:
            print(f"  ✗ 前台的等待循环没有超时（认出的循环：{'；'.join(loops)}）：等的条件不成立就一直不返回，"
                  "这一次调用卡在这里期间，发给你的消息也送不到", file=sys.stderr)
            print("     → 怎么办，二选一：① 要等的活用 Bash 的 run_in_background: true 起，然后结束本轮，等它的完成通知再接着做；"
                  "② 按进程号等：`python3 .claude/singlefs-ai-sop/scripts/proc.py wait <pid> --timeout <秒>`（到点还没退出就列出还活着的、退出码 3）。"
                  "确实要在前台轮询一个文件，外面套一层 timeout：`timeout <秒> bash -c 'until …; do sleep …; done'`。"
                  "这一道只拒这种写法，不停你在跑的任何东西", file=sys.stderr)
            return 2
        try:
            overwrites, noted_overwrites = results_overwrite_refusal(tool_input.get("command") or "", repository_root_of(hook_dir))
        except Exception as error:
            print(f"  ! bash-command-detector.sh 没判成整份覆盖 research/results/ 产物的写法（{error!r}），这条命令照常执行", file=sys.stderr)
            overwrites, noted_overwrites = [], []
        if overwrites:
            print(f"  ✗ 要整份覆盖 research/results/ 下已存在、又没进 git 的产物（认出的写法与目标：{'；'.join(overwrites)}）："
                  "旧字节没有任何一份副本，盖掉就找不回来", file=sys.stderr)
            print("     → 怎么办：按日期另存一个新文件名，旧的留着；确实要换掉旧产物，先 `git add` 它或者把它挪到带日期的旧名"
                  "（`git add` 或 `mv 旧名 带日期的旧名` 可以写在同一条命令里、排在写之前）。"
                  "追加用 `>>` 或 `tee -a`，不拦。这一道只拒这种写法，不停你在跑的任何东西", file=sys.stderr)
            return 2
        try:
            in_place_writes = script_in_place_refusal(tool_input.get("command") or "", repository_root_of(hook_dir))
        except Exception as error:
            print(f"  ! bash-command-detector.sh 没判成在原 inode 上改脚本的写法（{error!r}），这条命令照常执行", file=sys.stderr)
            in_place_writes = []
        if in_place_writes:
            print(f"  ✗ 要在同一个 inode 上改一个已经存在的脚本（认出的写法与目标：{'；'.join(in_place_writes)}）："
                  "正在跑它的 bash 按文件偏移往下读，改完会从新内容的同一偏移接着读，读到的是别的东西", file=sys.stderr)
            print("     → 怎么办：写到同目录临时文件再 `mv` 换上（`mv` 换 inode，正在跑它的进程读的还是旧内容），"
                  "或者用 `research/scripts/replace-once.py` / `insert-row.py` 定点改。追加（`>>`、`tee -a`）与新建不拦；"
                  "Edit / Write 工具本来就换 inode。这一道只拒这种写法，不停你在跑的任何东西", file=sys.stderr)
            return 2
    if reasons:
        print(f"  ✗ 看门狗起法不对（{'；'.join(reasons)}；认出的调用：{'、'.join(calls)}）：这样起的看门狗叫不醒主 agent，等于没盯", file=sys.stderr)
        print("     → 怎么办：看门狗用 Bash 的 run_in_background: true 起，命令只写 `bash research/scripts/watch.sh [--ack …] <agent 号，逗号分隔>`，"
              "不加 `&`、`nohup`、`disown`，不重定向输出（只盯自己起的长进程写 `bash research/scripts/watch.sh --processes`）", file=sys.stderr)
        return 2
    try:
        record(hook_input, os.environ.get("AGENT_HOOK_DETECTIONS") or DEFAULT_DETECTIONS, noted_overwrites)
    except Exception:
        return 0
    return 0

sys.exit(main())
PY
