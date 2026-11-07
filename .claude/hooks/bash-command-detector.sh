#!/usr/bin/env bash
# admission: always Claude Code 每一次触发都要现判这一次调用，上一次的结论不替这一次作保
# run-condition: command python3
# PreToolUse hook（Bash）：检出可能出问题的命令，记下来交给主 agent 判断；起看门狗的错误写法、前台没超时的等待循环、把活放出追踪的写法、run_in_background 里后面没有 wait 的单独 `&`、整份覆盖 `research/results/` 下未跟踪产物的写法、打得到别人进程的终止写法、在同一个 inode 上改已有脚本的写法、`rm -r` 作用在工作根本身、它的上级或工作根下第一层通配上的写法、项目子 agent 在 Bash 里起 claude 会话的写法、暂存区非空时不带具体路径清整个索引的写法（⑮）与带路径撤掉工作区里没暂存改动的写法（⑱）、手拷仓副本的写法（⑳）在执行前拒绝，其余只记不拦，不停任何在跑的命令与脚本。
# hook-events: PreToolUse:Bash
# gate-similar: project-code-guard.sh 同挂 PreToolUse[Bash]、按同一个切词模块认命令位置，但它判的是不是不经 task-server 跑项目代码；这里判等待循环、放出追踪、单独的 &、覆盖产物、终止进程、同 inode 改脚本、就地改仓内文件、rm -r 通删工作根，判据没有一条相同
# gate-similar: write-guard.sh 管 Write / Edit 的写范围与先编后换；这里 ⑧ 管有 Edit 的子 agent 绕到 Bash 里写仓内文件、⑨ 管执行员在 Bash 里写主工作区 crates/ 下的 .rs，两边判的工具不同
# gate-similar: write-guard.sh 的第五道、第七道判 Write / Edit / MultiEdit 写进 records/ 的内容；这里的 ⑲ 不看内容，拒 Bash 里往 records/ 下 .md 的追加本身（>>、&>>、tee -a、python 的 open(…, 'a')），出路 scripts/kb/append-record.py 写之前经 scripts/kb/lib_notation_check.py 读同一份判据库 lib-forbidden-notations.py；挂的工具不同，并不到一起
# gate-similar: pattern-process-guard.sh（上游）同挂 Bash、拒按模式找进程；这里不判那一类
# gate-similar: write-guard.sh 的第五道、第七道判 Write / Edit / MultiEdit 写进的钟点、会话号与批次代号；这里的 ⑰ 判 Bash 里 heredoc、echo、printf、tee 往仓里写的字面内容，读同一份判据库 lib-forbidden-notations.py，挂的工具不同；scripts/kb/lib_notation_check.py 判定点写入脚本的新串，也读那一份
# gate-similar: write-guard.sh 的第二道（项目子 agent 的写范围）读同一张 agent-write-scope.tsv，读表与判路径落没落进范围的函数抽在 lib_write_scope.py、两边都调它；它管 Write / Edit / MultiEdit 的目标，这里的 ⑫ 管 Bash 里写进仓里的目标，挂的工具不同，并不到一起
#
# 按模式找进程（`pgrep -f`、`pkill -f`、`killall`）不在这里判：上游 SOP 的 `.claude/singlefs-ai-sop/scripts/claude-hooks/pattern-process-guard.sh`
# 在执行前拒绝（`.claude/settings.json` 里与本 hook 注册在同一条 Bash matcher 下），判据与 shell-lint 的 S2、S3 同一份、只认命令位置。
# 这里不判：「这几个词出现在哪都算」的判据会把写在 grep 参数里的词也拦了，所以拦截放上游。
#
# hook 不能终止任务和脚本，只负责检出问题，交给主 agent 去判断；子 agent 与脚本结不结束由主 agent 定。
#
# 检出三种，写进检出记录（默认工作根下的 agent-hook-detections.jsonl，同目录 lib_work_root.py 的 default_detections_path；环境变量 AGENT_HOOK_DETECTIONS 可改），每条一行 JSON：
#   ① 没有 `timeout` 的 `until` / `while` 等待循环（里面有 `sleep`）：等的条件不成立时会一直等（前台的在执行前拒绝，见「拒绝七种」②，拒了不再记）；
#   ② `run_in_background` 起的命令里又自己放后台：外层 shell 当场退出（下面列的形态里，命令位置上的 disown、coproc、setsid -f、nohup … &、
#      tmux / screen 的分离模式、systemd-run 在执行前拒绝，见「拒绝七种」③；单独的 `&` 而之后同一层没有 `wait` 的在执行前拒绝，见「拒绝七种」④；
#      拒了不再记。这里剩下的只记：`start-stop-daemon -b`，与不带 --wait 的 setsid），
#      harness 的完成通知当场发出，真正在跑的东西跑完不再叫醒谁。认的形态：`disown`、`coproc`、`setsid -f/--fork`、同一行带单独 `&` 的 `nohup`、
#      `tmux new(-session) … -d`、`screen -dm…`、不带 --wait/--pipe/--pty/--scope 的 `systemd-run`、`start-stop-daemon -b`（这几种按正则 SELF_DETACH 找），
#      以及没等到的单独 `&`（不是 `&&`、`>&`、`&>`、`|&`、`<&`）：与 ④ 同一套数法（unwaited_background_jobs 按层、按圆括号逐个数 wait_coverage），
#      不带参数的 `wait`、调用里有它的函数等齐之前的全部，`wait <进程号>…` 写了几个号等几个（每个 `&` 记了 `$!`、之后逐个 `wait "$pN"` 的算等齐），
#      `wait -n`、`tail --pid`、`while|until … kill -0` 轮询各等一个；④ 拒了的不再记，④ 不拒的这里也不记（主 agent 的
#      `( … ) & p1=$!; ( … ) & p2=$!; wait $p1; wait $p2` 不记；弄坏开关 background-ampersand-by-regex 下按正则数，会误记）。
#      命令位置上的 setsid 前缀不带 -w / --wait（带 -f / --fork 的归 ③ 拒）：它起的那一刻若是进程组组长（包装 exec 过去、bash -c 只有这一条）就 fork 一份、
#      自己马上退出，执行前看不出会不会是组长，照记一条（bare_setsid_forms；每一层、前缀与 capped.sh / run-with-memory-cap.sh 这类包装剥开；
#      弄坏开关 bare-setsid-unrecorded 下不记）。
#      喂给非 shell 解释器的 heredoc 正文先剥掉再找（Python 的按位与、Rust 的引用不算）；引号里当数据用的 `&`（sed 替换串、URL 参数）不算，`bash -c '… &'` 的代码是一层、照数。
#      判不到的：前台起的 `… &`（run_in_background 为假，hook 那一刻看不出之后会不会结束本轮去等；由看门狗那一侧的
#      「结束本轮、手里没有在跑的后台任务、也没交回」告警兜）；SELF_DETACH 那几种按正则找，引号里当数据写的也会误记。
#   ③ 项目子 agent（输入里有 agent_type）前台 Bash 调用的 `tool_input.timeout` 超过 SUBAGENT_FOREGROUND_TIMEOUT_MILLISECONDS（240000 毫秒）：
#      共用约束 .claude/agent-common.md「长活交回主 agent」写「前台命令的 `timeout` 不超过 240000 毫秒」，那一句不在「执行前拒绝的写法」里，这里只记、不拒
#      （subagent_timeout_findings）。主 agent、run_in_background 的（⑩ 已拒）、没写 timeout 的、写得不是数的不记；这条命令被别的道拒了的不记。
#      弄坏开关 subagent-timeout-unnoted 下不记。
# 主 agent 的看门狗（scripts/agents/agent-watch.py watch）每 15 秒读一次检出记录，读到本会话的检出就退出、叫醒主 agent。
# 两种检出都先剥掉喂给非 shell 命令的 heredoc 正文（往笔记里写一段等待循环不是在跑它）；喂给 bash / sh 的 heredoc 正文照查。其余只看顶层命令文本，
# 引号里当数据写的等待循环也会被记一条，主 agent 看了判断即可。
#
# 判不了的（放行但必记检出）：hook 输入不是 JSON 对象（stderr 打一行带钩子名与开头若干字节的诊断，检出记录的会话号写「读不到」）、
#   哪一道判的时候抛了异常、共用模块之外的库（lib_project_code_runs.py、lib_writer_lock.py、lib_write_scope.py）读不到、喂给 shell 的不是字面（见 ① 的「判不到的」）：
#   照常执行，各记进这一条命令的检出（异常与库读不到另打一行 stderr）；这条命令被别的道拒了的，判不了的几样照记一条，结局写「被拒」。
# ⑩ 子 agent（输入里有 agent_type）用 run_in_background 起命令：一律拒，task-server/task wait 也拒（子 agent 一律不往统一任务服务交任务，也就没有要后台等的）。出路：这一条不起，跑项目代码的命令拼成 submit 的整条、
#   写进报告「要主 agent 跑的命令」一节，结束本轮交回主 agent。排在最前面判，主 agent 不判（subagent_background_refusal）。弄坏开关 allow-subagent-background（一概放行）。
# ⑬ 项目子 agent（输入里有 agent_type）在 Bash 里起 claude 会话：命令位置上的命令词是 `claude`、以 `/claude` 结尾的路径（`~/.local/bin/claude`），
#   或 `"$CLAUDE_CODE_EXECPATH"` / `"${CLAUDE_CODE_EXECPATH}"`（会话自己那份可执行文件）。共用约束 .claude/agent-common.md「派发」一节写明子 agent 不从 Bash 里起
#   claude 会话，这一道把那句话做成执行前拒（claude_launch_refusal）；前台、run_in_background 一样拒，主 agent 不判。认法与别的道同一套：按 shell 切词只认命令位置，
#   nice / env / exec / command 这类前缀、capped.sh N、bash -c、喂给 shell 的 heredoc、命令替换里的都认；`command -v claude`、`which claude`、
#   grep 参数里的 claude、引号里当数据写的、写进文件的 heredoc 正文不拒。拒绝说明的出路：要另起一个 agent 的活写进报告交回主 agent，由主 agent 派。
#   另照共用模块切出的每一条命令认（launched_command_words）：here-string 与管道喂给 shell 的字面、env -S 拆出来的、名单外启动器（xargs、ssh、find -exec 这类，
#   lib_project_code_runs.py 的 command_under_launcher 剥开）里面的 claude（弄坏开关 claude-launch-layers-narrow 下不认）。
#   shell 启动时先读的文件（BASH_ENV / ENV、--rcfile、HOME 下的 .bashrc、zsh 的 .zshenv 与 .zshrc；共用模块 launcher 为 "startup" 的命令，另有直接执行的脚本 #! 行指到的 shell 先读的，
#   startup_files_of）是例外：路径是字面的（前面赋过字面值的变量先展开）、读得到的，读进来按同一套认里面起的 claude
#   （startup_file_claude_launches；文件里再有这样的写法接着读，同一份只读一遍，MAXIMUM_NESTING 层为限）；路径解不出的子 agent 由
#   project-code-guard.sh 照认不出的写法拒；弄坏开关 startup-file-claude-unread 下不读这类文件。登录 shell 读的 /etc/profile 与 HOME 下的 profile、交互 bash 读的 /etc/bash.bashrc
#   同样是共用模块交的启动文件，照这样读。
#   bash / sh / source 起的脚本与直接执行的 shell 脚本（路径带斜杠，#! 指到 shell 或没有 #! 而扩展名是 .sh）：路径是字面的、读得到的，读正文照同一套认里面起的 claude，
#   同一条命令里前面 cat / tee 读 heredoc 写出的拿写出的内容（file_claude_launches；与 project-code-guard.sh 读脚本正文同一个口子，cp 拷来的不认；弄坏开关 script-body-claude-unread 下不读）。
#   别的程序里字面写着起的 claude（python -c、perl -e、awk system()、sed e、tclsh exec、git 别名 这类；共用模块 embedded_launches）照命令认，awk -f、sed -f、Makefile 读正文认
#   （弄坏开关 embedded-launch-claude-unread 下不认）；写法认不出的子 agent 由 project-code-guard.sh 照认不出拒。
#   判不到的：变量里拼出来的命令（`c=claude; $c -p …`：命令词认不出起什么，子 agent 由 project-code-guard.sh 照认不出的写法拒）、eval 的变量代码、
#   claude 改了名的拷贝或链接、python 等脚本文件里经 subprocess 这类调用起的（字面写在命令里的照 embedded_launches 认）。
#   弄坏开关 allow-subagent-claude-launch 下放行。
#
# 拒绝七种（退出 2，stderr 写原因与出路；拒了不记检出；一条命令同时踩几类的，几类都判完一次列全，每类带自己的出路）。它们拒的是一种危险写法、在执行之前，不停在跑的任务，
# 是 .claude/singlefs-ai-sop/rules/command-safety.md「检出和处置分开」那一条写明的例外（「拒绝某一种危险写法的钩子不在此列」）。
# ① 命令位置上起看门狗——`scripts/agents/watch.sh`（带 `--selftest`、`--report`、`--dry-run` 的不盯，不算）
#   或 `scripts/agents/agent-watch.py watch`——而 run_in_background 不是 true，或整条命令里有单独的 `&`、`nohup`、`disown`、`setsid`，
#   或看门狗所在那一段把输出重定向到 /dev/null：这样起的看门狗叫不醒主 agent，等于没盯。
#   /dev/null 只算看门狗所在那一段的（null_redirects_reaching_watchdog）：挂在别的简单命令上、又不在管道后段改标准输出的（`grep … 2>/dev/null; bash scripts/agents/watch.sh …`）
#   不算；挂在看门狗自己、包着它的那一层（`bash -c '…' > /dev/null`、花括号与圆括号整组、`exec >/dev/null`）、管道后段改标准输出的照算。
#   `main-agent.md`「调用里不再加 `&`、`nohup`、`disown`」靠自觉守不住。
#   只认命令位置：按 shell 的规矩切词（引号与 # 注释去掉；case 的模式词、`[[ … ]]` 里的词不在命令位置，共用模块的 case_patterns_and_conditionals_folded），`grep -n watch.sh`、`cat` / `tail` 读看门狗输出这类把名字当参数的不拦；
#   `bash -c '…'`、`capped.sh N …`、`source`、`python3 脚本` 与 nice / timeout / env / taskset 这类前缀里面的照判，`bash -n`（只查语法）不算，
#   喂给非 shell 命令的 heredoc 正文每一层都先剥掉。
#   命令替换 `$(…)` 与反引号在双引号里外都判（里面的命令另按一条完整的命令判，lib_shell_words.py）。
#   here-string（`bash <<< '…'`）与管道前一段是 printf、echo、读 heredoc 的 cat（`printf '…' | bash`）喂给从标准输入读命令的 shell 的字面文本照命令判（lib_shell_words.py 的 shell_standard_input）。
#   判不到的：变量里拼出来的命令、eval、喂给 shell 的不是字面（here-string 或 printf / echo 的参数里有变量、命令替换，管道前一段是 cat 文件或别的命令，标准输入从文件来：
#   记一条检出，见下面「判不了的」）；喂给 shell 的 heredoc 正文里引号没配对时，其后的切词会错。
# ② 前台（run_in_background 不是 true）的命令里有没超时的等待循环：命令位置上的 `until` / `while` 循环，条件或循环体里有一条 `sleep`
#   （剥掉 nice / timeout 这类前缀之后命令词是 sleep），外面又没有 `timeout`。前台的
#   `until grep -q '^rc=' <日志>; do sleep 10; done` 卡在这一次调用里，主 agent 发去的纠正消息要等它下一次调用工具才送得到，
#   只能按 pid 从外面停掉那个循环：只记检出叫得醒主 agent，叫不醒卡住的那一个。
#   主 agent 与子 agent 都拒；run_in_background 里的等待循环照旧只记检出（①），只有条件或循环体里点名了本会话后台任务输出文件
#   （`tasks/<id>.output`）的例外：等自己起的后台任务要结束本轮等完成通知，这种轮询前台、run_in_background 一样拒，外面套了 timeout 也拒。
# ⑧ 有 Edit 工具的项目子 agent（`.claude/agents/<agent_type>.md` 的 tools 里有 Edit）在 Bash 里就地改仓内文件：命令位置上的 `sed -i`（目标不在 /tmp/ 下、也不在工作根下；相对路径从 cd 之后的目录算，见下），
#   或 python 代码里 open(<仓内路径字面量>, 'w' / 'a' / 'x')、Path(<仓内路径字面量>).write_text( / .write_bytes(，以及 os.replace / os.rename / shutil.move 的目标（第二个参数或 dst=）
#   是仓内路径字面量的（先在草稿目录写临时文件、再挪进仓里，同样是绕开 Edit；弄坏开关 python-move-into-repository-unseen 下不认这三种）。
#   仓内路径字面量：crates/、scripts/、.claude/、research/、records/、litmus/、briefs/ 起头的（可带 ./），或仓根起的绝对路径。写范围闸只看 Write / Edit，这样写它看不见；
#   草稿目录照写，定义点名的脚本（replace-once.py 这类）不在这一条里。路径放在变量里的认不出。
#   python 代码只指真要被 python 跑的：`python3 -c` 的那一段，和喂给 python 的 heredoc 正文（bash -c、命令替换、喂给 shell 的 heredoc 里的那几层也找）；
#   喂给 cat、tee 这类别的命令的 heredoc 正文是数据，引号里当数据写的文字也不判（往草稿目录写一份脚本或笔记，正文里出现 open(…, 'w') 不是在写；
#   弄坏开关 heredoc-body-judged-as-commands 下这一半的正则在整条命令文本上跑，写草稿脚本会误拒）。
#   经管道喂给 python 的 heredoc（`cat <<'EOF' | python3 -`、`| python3`，管道前一段是 cat，或 echo / printf 的文字）也是 python 代码，与 `python3 - <<'EOF'` 同一处判（overwrite_steps 的 PythonCode）；
#   管道后一段是 bash / sh 的，正文照命令逐行判；管道后一段是 grep、tee 这类别的命令的，正文是数据（弄坏开关 heredoc-pipe-fed-as-data 退回正文算数据）。
#   sed -i 一半取 overwrite_steps 的结果（与 ⑫ 同一处的 cd 跟踪，不另抄一份）：相对路径从 cd 之后的目录起算（起点是 hook 输入的 cwd，没有就是仓库根），
#   算出来落在 /tmp/ 或工作根下、不在本仓里的放行；cd 到仓内目录、cd 的目录算不出、目标带变量或通配的照旧按仓内判（弄坏开关 sed-in-place-ignores-cd 退回相对路径不跟 cd）。
#   python 一半的相对路径（open / Path / 三种挪法的目标写成 crates/…、scripts/… 这类）同样从 cd 之后的目录起算（overwrite_steps 的 PythonCode 记的当前目录，起点同上）：
#   算出来落在本仓之外的放行（cd 进草稿目录里的仓副本再写相对路径）；cd 到仓内、cd 的目录算不出、同一段代码在几处跑而有一处落在仓里、只在 shell_layers 里找到的代码，
#   照旧按仓内判；仓根起的绝对路径一律按仓内判（弄坏开关 python-relative-path-ignores-cd 退回相对路径不跟 cd、一律按仓内判）。
#   判不到的：管道中间隔了别的命令（`cat <<'EOF' | tee x | python3 -`）、代码从变量或文件里读的。
# ⑨ 先编后换：experiment-runner（名单是 COMPILE_FIRST_AGENTS）在 Bash 里写主工作区 crates/ 下的 .rs：出路是在草稿目录的副本里改、
#   经 scripts/hosts/compile-then-swap.py 编过再整份换进来。write-guard.sh「四、先编后换」判 Write / Edit 那一半，这里判 Bash 那一半；
#   agent 名单与路径判据两边各写一份、改的时候一起改。排在 ⑧ 之前判，执行员拿到的出路是这条脚本。
#   为什么：执行员直接在主工作区改入库装置，改到一半编不过，别的会话带 --all-targets 的编译一起卡住（records/2026-09-16-subagent拆分提案.md 第四十节那张表第 51 行）。
#   认的写法：⑤ ⑦ 那一段 overwrite_steps 认的（>、>|、&>、>& 文件、不带 -a 的 tee、cp / mv / install 的目标、dd of=、truncate），every_write 时另认
#   >>、&>>、tee -a、sed -i、perl -i、rsync 的目标，mv 挪走与 rm 删掉的源（连同 crates/ 里的目录），喂给 python 的代码里 open(…, 带 w / a / x / + 的模式)、
#   Path.write_text / write_bytes、shutil.copyfile / copy / copy2 / move 与 os.replace / os.rename 的目标。cd 与前面赋过值的变量跟着算，每一层都判；
#   相对路径从 cd 之后的目录起算，起点是 hook 输入的 cwd（没有或不是目录时是仓库根；同 ⑧ ⑫），cd 算不出之后的照仓库根解
#   （compile_first_verdict 的 start_directory；弄坏开关 compile-first-ignores-cwd 退回起点一律是仓库根）；
#   目标里有这一刻算不出的段的，按通配对主工作区已有的文件，对得上 crates/ 下的 .rs 就拒。前台、run_in_background 一样拒。
#   放行：主 agent、implementation-writer 与别的 agent；草稿目录里的写；把主工作区那份拷进草稿目录；crates/ 下不是 .rs 的（crates/mutations.tsv）；
#   `python3 scripts/hosts/compile-then-swap.py …`（脚本文件里的写这里本来就判不到）；引号里当数据写的、写进文件的 heredoc 正文、注释里的。
#   判不到的：变量里拼出来的命令、eval、`bash x.sh` 起的脚本文件里的写法、patch 与 git apply / checkout / restore、目标算不出又是新文件名的、
#   python 里从运行期值来的路径与 os.open / os.write、经 subprocess 起的写法。
#   「外面有 timeout」按包装的层次认：`timeout N bash -c '…'`、`capped.sh N timeout N bash -c '…'`、`timeout N bash <<EOF` 喂进去的正文里的循环算有；
#   只在循环条件或循环体里的 timeout（`until timeout 5 grep …; do sleep 10; done`）不算，循环照样能一直转下去。
#   只认命令位置（按 shell 的规矩切词），引号里当数据写的、写进文件的 heredoc 正文、注释里的循环不拒；
#   `bash -c '…'`、`capped.sh N …`、喂给 shell 的 heredoc 正文、命令替换里面的递归判。多行写的循环（`do` 另起一行）同样认。
#   不带 sleep 的循环（`while read -r 行; do …; done < 文件`）、`for` 循环不拒；带计数上限的 while 循环机器分不出上限，照拒，外面套一层 timeout 就放行。
#   判不到的：变量里拼出来的命令、eval、`bash x.sh` 起的脚本文件里的循环。
# ③ 命令位置上把活放出追踪的写法：`disown`、`coproc`、`setsid -f` / `--fork`、`nohup` 而同一层有单独的 `&`、`tmux new(-session) … -d`、
#   `screen -dm…` / `-d -m`、不带 --wait / --pipe / --pty / --scope 的 `systemd-run`——前台、run_in_background 一样拒，主 agent 与子 agent 都拒。
#   进程移出 shell 的作业表或另起会话之后，随后的 `wait` 立刻返回、harness 的完成通知当场发出，进程成了没人追踪的孤儿接着跑
#   （例：`… & disown 2>/dev/null || true & wait`）；`.claude/agent-common.md`「长活交回主 agent」那一条明令不用这几种，靠自觉守不住。
#   每一层（`bash -c '…'`、喂给 shell 的 heredoc 正文、命令替换）都判，nice / timeout / env 这类前缀、`capped.sh N`、`bash 包装脚本` 剥开往里找；
#   引号里当数据写的、写进文件的 heredoc 正文、注释里的不拒；`nohup` 不带 `&`、不带 -f 的 `setsid`、`systemd-run --wait` 这类会等到结束的不拒
#   （不带 --wait 的 setsid 在 run_in_background 里照检出 ② 记一条：它是进程组组长时会 fork）；
#   单独的 `&` 不在这一条：run_in_background 里之后没有 `wait` 的归 ④，其余照检出 ② 只记。
#   判不到的：变量里拼出来的命令、eval、`bash x.sh` 起的脚本文件里的写法、`start-stop-daemon -b`（只在 ② 里记）。
# ④ run_in_background 为 true 的命令里，有一个作业以单独的 `&` 收尾（不是 `&&`、`>&`、`&>`、`|&`），之后同一层、同一对圆括号里再没有 `wait`
#   （等齐它的 `wait`，数法见下面「按次序数等了几个」那一段）：外层 shell 起完它就退出，完成通知当场发出，
#   真跑完的那个进程叫不醒任何人（例：run_in_background 里写 `… > 日志 2>&1 &`）。主 agent 与子 agent 都拒；前台的 `… &` 不拒。
#   按共用模块切出的记号逐层判：`bash -c '…'`（capped.sh N、nice 这类包着的剥开）、喂给 shell 的 heredoc 正文、命令替换各是一层，
#   里面的 `&` 要在同一层里等；圆括号是子 shell，`(… &); wait`、`… & (wait)` 里的 `wait` 等不到那个作业，照拒；
#   `{ …; }`、循环与 if 的身子与外面同一层。
#   按次序数等了几个（wait_coverage）：不带参数的 wait、调用里有不带参数 wait 的函数（`w() { wait; }; … & w`）等齐之前的全部；
#   `wait <进程号>…` 等它写了几个号就几个，两个 `&` 只等一个的，剩下那个照拒；`wait -n`、`tail --pid`、`while|until … kill -0` 轮询各等一个（检出 ② 末那一句）。
#   弄坏开关 partial-wait-allowed 下带进程号的 wait 照等齐算，wait-forms-unseen 下 wait -n、tail --pid、kill -0 轮询、函数里的 wait 都不算。
#   引号里当数据写的、写进文件的 heredoc 正文、注释里的不拒。
#   判不到的：变量里拼出来的命令、eval、`bash x.sh` 起的脚本文件里的写法。
# ⑤ 命令位置上会整份覆盖 `research/results/` 下一个已存在、又没进 git（`git ls-files --error-unmatch` 失败）的文件：重定向 `>`、`>|`、`&>`、`>& 文件`
#   （前面带 fd 号的 `2>` 一样截断，一样算）、不带 `-a` / `--append` 的 `tee` 写的每个文件、`cp` / `mv` / `install` 的目标（`-t 目录`、或目标是已有目录、
#   以 `/` 结尾的，写的是 目录/源的文件名，源里的通配按这一刻的文件展开）、`dd of=`（带不带 `conv=notrunc` 都算）与 `truncate` 的文件（
#   与 ⑦ 认的是同一段 `overwrite_steps` / `follow_simple_command`，不重复判定）。整份覆盖未跟踪的产物，旧字节找不回；
#   write-guard.sh 只拦 Write / Edit 工具，拦不到 shell。
#   进没进 git 的判法：整条命令里确定的目标路径收齐，一次 `git ls-files` 查完（tracked_verdicts）；算不出的样式（通配、变量）另按对得上的文件一次查。
#   前台、run_in_background 一样拒，主 agent 与子 agent 都拒。
#   放行：`>>`、`&>>`、`tee -a` 追加；目标不存在（新文件名）；目标已进 git（`git add` 过就算，git 兜得住）；目标不在仓库根的 `research/results/` 下；
#   `2>&1`、`>&2`、`>&-` 这类 fd 复制与关闭；`cp` / `mv` 带 `-n`、`--no-clobber`、`--update=none`、`-b`、`--backup` 的（已有的目标不被整份盖掉）；`install -d`；
#   同一条命令里排在写之前 `git add` 过、或被 `mv` 挪走的那个文件（拒绝信息给的两条出路写进同一条命令也认）。
#   目标按命令里的 cd / pushd 跟着算；cd 算不出来（`cd "$变量"` 而变量没在这段命令里赋值、`cd -`、`popd`；`$HOME` 照钩子自己的环境展开，共用模块的 changed_directory）或没有 cd 时按仓库根（这个 hook 所在的仓，`.claude/hooks/` 往上两级）解析。
#   这条命令里前面独立赋过值的变量（`out=…`、`export out=…`）在目标里的 `$out`、`${out}` 先代入。
#   目标这一刻算不出来（没赋过值的变量、命令替换、通配、花括号展开）的不拦：算不出的段换成 `*` 去对 research/results/ 下已有的文件，
#   对得上没进 git 的就记一条检出（对不上的一定是新文件名，不记）；git 自己出错、判不了进没进 git 的也只记检出。
#   每一层（`bash -c '…'`、喂给 shell 的 heredoc 正文、命令替换与进程替换）都判，nice / timeout / env / sudo 这类前缀、`capped.sh N` 与 `bash 包装脚本` 剥开往里找；
#   引号里当数据写的、写进文件的 heredoc 正文、注释里的不拒。
#   判不到的：变量里拼出来的命令、eval、`bash x.sh` 起的脚本文件里的写法、xargs 与 `find -exec` 起的 cp、`cp -r` 整目录拷进已有目录时里面逐个文件、
#   `sed -i`、`sort -o`、`rsync`、`ln -f` 与 python 里 `open(…, 'w')` 这类别的写法；圆括号子 shell 里的 cd 当成对后面的命令也生效；
#   不看 hook 输入里的 cwd（没有 cd 的相对路径一律按仓库根解析）。
# ⑥ 终止进程只许点名一个自己起的进程号或任务号，一次一个：后面的脚本不能终止前面的脚本，不能动 ssh，终止按任务号、禁止终止所有。
#   为什么：读自己 cgroup 的 cgroup.procs、给里面每个进程发 TERM，而进程就在 SSH 会话的 session-1.scope 里时，
#   VSCode 服务端、扩展宿主、ptyHost 与 Claude 会话会一起断掉。
#   前台、run_in_background 一样拒，主 agent 与子 agent 都拒。拒的写法：
#   kill 的目标带负号（进程组；-1 是能发到的全部）、是 0（发命令的整个进程组）或 $PPID（会话本身）；一次给了不止一个目标；目标是命令替换
#   （`$(pgrep …)`、`$(jobs -p)`）、数组或位置参数的整批展开、通配；在 for / while / until 循环里逐个发（`kill -0` 探活、`kill -l` 不算发）；
#   `proc.py stop` 同样一次一个、不在循环里；xargs / parallel / find -exec 把一串进程号喂给 kill 或 proc.py；pkill、killall、skill 按名字挑；fuser -k；
#   同一条命令里先从 cgroup.procs、/proc 或 pgrep / pidof 挑进程、再发信号；往 cgroup.kill、cgroup.freeze 写；
#   systemctl 停、杀、重启、改属性（stop、kill、restart 一族、freeze、set-property、disable / mask --now）ssh、sshd、session-*.scope、user@*.service、
#   user.slice、user-*.slice、私有配置里登记的本地模型服务单元、systemd-logind、dbus，或单元带通配、是命令替换；systemctl isolate、poweroff、reboot 一类与 --user exit；
#   loginctl terminate-* / kill-*；reboot、poweroff、halt、shutdown（shutdown -c 不算）；
#   目标是写死的进程号时现读进程表（整条命令里第一次判到 kill 或 proc.py stop 的写死的进程号才读 /proc、读一次；不带这种目标的命令，带数字的 git log -1、sleep 2 也不读）：是这条命令所在会话自己的祖先（Claude 会话、VSCode 扩展宿主往上）、pid 1、名叫 sshd / systemd / dbus / logind / claude 的、
#   命令行里带 .vscode-server 或私有配置里登记的本地模型服务标记的，或它的祖先里有另一个 Claude 会话（别的会话起的）——都拒；读不到那个进程的不拦（kill 自己会报错）。
#   本地模型服务的单元名与进程标记是本机的私有信息，不写进仓：从仓外的私有配置读，文件是 BASH_COMMAND_DETECTOR_PRIVATE_CONFIG，
#   不设就是 ${XDG_CONFIG_HOME:-~/.config}/singlefs/bash-command-detector.env；一行一个 KEY=值，值按空白或逗号切成几个：
#   LOCAL_MODEL_SERVICE_UNITS（systemctl 的单元名，不写后缀的按 .service 补上再比）、LOCAL_MODEL_SERVICE_PROCESS_MARKERS（进程命令行里出现就算的片段）。
#   文件不在、读不了、没有那个键，那一项就不判、不报错（其余照判）。
#   python3 -c 与喂给 python 的 heredoc 里的代码同样判：os.killpg、os.kill 的目标是 0 或负数、在循环或推导式里调 os.kill、同一个函数里读 cgroup.procs 或 /proc 再 os.kill。
#   放行：`kill "$!"`、`kill -TERM "$pid"`、`kill %1`、`kill %%` 这样点名一个的；`proc.py stop <一个 pid>`；
#   对自己用新会话起的那一个子进程的进程组发信号（上游 .claude/singlefs-ai-sop/rules/command-safety.md「装置起的样本进程，放进它自己的进程组」）：
#   shell 里同一层先 `setsid [--wait] <命令> &`（不带 -f）、紧跟着 `名字=$!`，之后 `kill -<信号> -- -"$名字"`；python 里子进程是带 start_new_session=True
#   （或 process_group=0）的 Popen 这一类，组号写 <子进程>.pid 或 os.getpgid(<子进程>.pid)，发 os.killpg(组号, …) 或 os.kill(-组号, …)。
#   认法与照旧拒的写法（参数、属性、改绑过的名字、0、-1、负的写死号、os.getpgid(0)、别的变量）写在代码里 SESSION_CHILD_CALLS 与 OWN_GROUP_TARGET 两段注释；
#   systemctl 对任何单元的 status / show / list-units / reset-failed，对自己起的单元（`systemctl --user stop singlefs_memory_selftest_…slice`）的停；
#   只读 cgroup.procs、只跑 pgrep 不发信号的；引号里当数据写的、写进文件的 heredoc 正文、注释里的。
#   每一层（`bash -c '…'`、喂给 shell 的 heredoc 正文、命令替换）都判，nice / timeout / env / sudo / builtin 这类前缀、`capped.sh N`、`run-with-memory-cap.sh <上限>`、
#   `bash 包装脚本` 与 `python3 脚本` 剥开往里找。
#   同一份判定由门禁 code-tooling 的 research-gate-lint 格拿去扫 scripts/、.claude/hooks/、.claude/scripts/ 与 .claude/gate.d/ 下的脚本（本文件 --scan-scripts，按行号报；
#   不判「同一条命令里先挑再发」那一条，脚本里靠循环与命令替换两条）。脚本里确实要按 cgroup 批量发信号的（run-with-memory-cap.sh 的 TERM 风暴自测），
#   先核 cgroup 路径是自己开的 singlefs-memory-cap-*.scope，那一行写 `# process-safety:own-scope <怎么核的>`，往上 40 行里要有核 `singlefs-memory-cap-*.scope` 的那一句；
#   这个标注只放行循环与按 cgroup 挑进程两类，标了而那一行没有要放行的判红。别人正在改、这一轮动不了的脚本登记进 .claude/process-safety-pending
#   （一行一个相对仓根的路径、# 后写为什么；指向不存在的、一处都没排到的判红），成功行逐个列名。
#   判不到的：变量里拼出来的命令、eval、`bash x.sh` 起的脚本文件里的写法（脚本归门禁 code-tooling 的 research-gate-lint 格）；`kill $pids` 这种不带引号、里面其实放了几个进程号的变量；
#   先把 cgroup.procs 读进文件、下一条命令再读文件 kill；循环里 `bash -c "kill $p"`（里面那一层不知道自己在循环里）；python 里经 subprocess 起的 kill、
#   Popen 的 terminate / kill；进程号写死而那一刻进程表读不到的。
# ⑦ 在同一个 inode 上改一个已经存在的脚本（`.sh`、`.py`，或带执行位的文件；在仓里、工作根或 Claude Code 的会话临时目录根 /tmp/claude-<uid>（各会话的 scratchpad 在里面）下，两个根只在同目录 lib_work_root.py 里定义）：正在跑它的 bash 按文件偏移往下读，
#   改完会从新内容的同一偏移接着读，读到的是别的东西；手敲的就地写同样要拦。
#   前台、run_in_background 一样拒，主 agent 与子 agent 都拒。拒的写法（每一种都实测过：已经打开着文件的读者在改写之后接着读到新内容）：
#   重定向 `>`、`>|`、`&>`、`>& 文件`（带 fd 号的 `2>` 一样）、`cat > 旧`；不带 `-a` / `--append` 的 `tee`；`cp` 的目标（`-f`、`-u`、`-a` 一样，写进已有目录的按 目录/源的文件名算）；
#   `dd of=`（带不带 `conv=notrunc` 都算）；`truncate` 的文件（这两种 ⑤ 判整份覆盖 `research/results/` 时也认，同一段 `overwrite_steps` 生成，
#   不重复判定）；喂给 python 的代码（`python3 -c` 与喂给它的 heredoc）里 `open(旧, 带 w 或 r+ 的模式)`（io / codecs 的 open 一样）、
#   `Path(旧).open(同上)`、`Path(旧).write_text` / `write_bytes`、`shutil.copyfile` / `copy` / `copy2` 的目标。
#   放行：`>>`、`&>>`、`tee -a`、python 的 `'a'` 追加（不动已经读过的偏移之前的字节）；目标不存在（新建）；目标不是脚本；目标在仓外又不在工作根与会话临时目录根下；
#   换 inode 的写法：`mv`、`install`、`cp --remove-destination`、`cp -l` / `-s`、`cp -n` / `-b` / `--update=none`、`os.replace`；
#   同一条命令里排在写之前 `mv` 挪走或 `rm` 掉的那个文件（之后写的是一个新 inode）。
#   目标跟着 cd / pushd 与这条命令里前面赋过值的变量算，与 ⑤ 同一套（overwrite_steps）；python 的相对路径按跑它那一刻的当前目录算。
#   python 的路径只认字符串字面量、整段代码里只赋过一次的名字、Path(…) 与 Path(…) / '…'、os.path.join 的字面量；模式只认字面量。
#   判不到的：变量里拼出来的命令、eval、`bash x.sh` 起的脚本文件里的写法、目标里有这一刻算不出的段（没赋过值的变量、命令替换、通配）、
#   python 里从 sys.argv 或别的运行期值来的路径、`os.open` 与 `os.write`、经 subprocess 起的写法；`rsync --inplace` 这类别的就地写。
#   Edit / Write 工具本来就换 inode，不经这个 hook。
# ⑪ 命令位置上的 `rm -r`（`-r`、`-R`、`--recursive`，与别的短选项合写的、写在操作数后面的都算）作用在工作根本身、它的上级，或工作根下第一层带通配的路径上
#   （`rm -rf ~/.local/share/singlefs/work/*`、`…/work/owed-*`、`…/work/*/target`、`rm -rf /tmp/claude-<uid>/*`、`rm -rf /tmp/*`、`cd <工作根> && rm -rf *`）。
#   工作根是同目录 lib_work_root.py 的配置值（不看它的弄坏开关）与会话临时目录根 /tmp/claude-<uid>（各会话的临时目录在里面），读不到它时按它的两条默认算。
#   为什么：按目录名通删工作根下的目录，会把别的会话在用的冻结副本、交回报告与只存在仓外的跑前登记一起删掉（.claude/kb/checks-owed.md C792（清盘按目录名通删没有归属标记））。
#   出路是 scripts/hosts/work-directory-ownership.sh：claim 写归属文件，sweep 只删归属是本会话、没有进程开着的，删前逐项连大小写日志。
#   前台、run_in_background 一样拒，主 agent 与子 agent 都拒。放行：工作根下一个具体目录（`rm -rf <工作根>/<目录>`）与它里面的通配（`<工作根>/<目录>/copy-*`）、
#   同名前缀的兄弟目录、别处的 rm -r、不带 -r 的 rm、当数据写的（引号里、写进文件的 heredoc 正文、注释）。
#   路径里 ~、$HOME、$SINGLEFS_WORK_ROOT 与这条命令里 export 过、命令前写过的变量代入，相对路径跟着 cd 算（cd 的目录里同一段命令前面赋过的变量与 $HOME 先展开，共用模块的 expand_known_variables；没有 cd 按 hook 输入的 cwd，再没有按仓库根）；
#   通配照 bash 的默认认（不开 dotglob：`~/*` 不碰家目录下以点开头的条目）。每一层（bash -c、喂给 shell 的 heredoc、命令替换）都判，前缀与 capped.sh 剥开。
#   判不到的：变量里拼出来的命令、eval、`bash x.sh` 起的脚本文件里的写法、没赋过值的变量（`rm -rf "$d"/*`）、单独一行赋值的变量（`w=…; rm -rf "$w"/*`）、
#   find -delete 与 find -exec rm、xargs rm、python 的 shutil.rmtree、会话临时目录根下 <项目>/ 那一层的通配（`/tmp/claude-<uid>/<项目>/*`）。
# ⑫ 有写范围的项目子 agent 在 Bash 里写自己写范围之外的仓内文件。写范围闸 write-guard.sh 只看 Write / Edit / MultiEdit（它文件头「拦不住 Bash 里的写」），
#   一个 clerk 在 Bash 里跑 scripts/kb/replace-once.py 写进了定义不让它写的知识库目录而没人拦（records/2026-09-16-subagent拆分提案.md 第八十一节 ④、第 64 行）。
#   这一道照同目录 agent-write-scope.tsv 同一张表判：读表、展开工作根、判路径落没落进范围的函数在 lib_write_scope.py，write-guard.sh 的第二道调的是同一份；
#   该 agent 的行里没有一条模式落到这个文件，就拒。认的写法：⑨ 认的那一批（overwrite_steps 的 every_write：>、>>、&>、tee（带不带 -a）、cp / mv / install 的目标、dd of=、truncate、
#   sed -i、perl -i、rsync 的目标，python 代码里 open(…, 带 w / a / x / + 的模式)、Path.write_text / write_bytes、shutil.copy*、os.replace / rename），加上 KNOWN_WRITE_SCRIPTS 五个定点写脚本的目标文件：
#   replace-once.py 的 文件、insert-row.py 的 文件（--line 这类选项的值不算目标）、replace-batch.py 规格 JSON 里每一项的 file（规格读不到或不是数组的，落点算不出，拒）；
#   ledger-append-section.py 不带目标文件参数，写的是仓根下登记表 .claude/gate.d/split-proposal-ledgers.tsv 里 current 那一份账本（LedgerAppendTarget）：仓根照它自己的认法取
#   （--root / --root= 给的；没给的是 python3 起的脚本路径解开符号链接之后往上三层；直接执行只剩脚本名，按本仓算），--root 或脚本路径算不出的照落点算不出拒，
#   登记表读不到、current 不是恰好一行的不判（脚本自己退 2、什么都不写）；写范围里没有 records/ 的子 agent 起它就拒（弄坏开关 scope-ledger-append-unseen 下不认它）；
#   append-record.py 写的是 <记录>：第一个不是选项的参数（--root 的值不算：RECORD_APPEND_OPTIONS_WITH_VALUE），相对路径从那一刻的当前目录起算（脚本自己也这样认）；
#   写范围里没有 records/ 的子 agent 起它往记录目录写就拒（弄坏开关 scope-record-append-unseen 下不认它，scope-record-append-root-as-target 下把 --root 的值当记录）；
#   带 --dry-run、--selftest、-h 的不写，不算。脚本词先按同一条命令里前面赋过的值代入再比 KNOWN_WRITE_SCRIPTS（`S=scripts/kb/replace-once.py; python3 $S <文件> …` 照认；
#   弄坏开关 scope-script-variable-unexpanded 下不代入）；python3 起的脚本词代入之后还有算不出的段（没赋过值或来源不明的变量、命令替换、for 循环变量），
#   而第一个不是选项的参数算得出、落在本仓里的（`python3 $X .claude/kb/a.md …`），认不出起的是不是定点写脚本，照「落点算不出」拒（UnresolvedScriptTarget）；
#   第一个参数在仓外、算不出或带 --dry-run 这类不写的参数的不判。python3 起的和直接执行的都认，bash -c、heredoc、命令替换、nice / capped.sh 这类前缀、scripts/hosts/two-logs.sh 包着的都剥开，cd 与前面赋过值的变量跟着算，相对路径从 hook 输入的 cwd 起算。
#   每一处写先算出最终落点（bash_scope_verdict）：落点在本仓里而不在范围里的拒；落点算得出而在仓外的（/tmp、/dev/null、工作根下的草稿目录）不判，工作根的写范围由表里 $SINGLEFS_WORK_ROOT/** 那一行放行。
#   mv 挪走的源与 rm 删掉的不判（只判写进去的）。落点与写闸 write-guard.sh 同一份判法（lib_write_scope.py 的 landings_out_of_scope）：先规整 `..`，再逐段解开符号链接（`<仓外的链接>/../x` 回到链接指向那一层的上级），每一种落点都要在范围里，字面路径在范围里而解开之后落在仓里范围外的照拒，字面路径在范围外而解开之后落回范围里的放行，解开之后落在仓外的不判；解不开（读不了链接）的照算不出拒（弄坏开关 scope-landing-lexical-only）。
#   没有写范围的不判：主 agent（输入里没有 agent_type）、表里没有它那几行的定义（表只登记有 Write / Edit 的定义，这样的 agent 在 Bash 里写报告，
#   范围写在各自定义的「写范围」里，机器没有表可比）。
#   没有项目定义的 agent（内置类型、插件带来的、定义已删的旧实例；输入里有 agent_type 而 `.claude/agents/<名>.md` 不在）只许写工作根（BUILTIN_AGENT_SCOPE，展开同表里的
#   $SINGLEFS_WORK_ROOT/** 那一行），与 write-guard.sh 第二道同一口径（只许写草稿目录）：
#   落在仓里的照上面判，落在仓外而不在工作根下的也拒（会话临时目录根 /tmp/claude-<uid> 下的同样拒；/dev/、/proc/ 下的设备与文件描述符不判：`2>/dev/null`、`> /dev/stderr` 照写）；落点算不出的同样拒。
#   弄坏开关 builtin-agents-unjudged 下没有项目定义的一概不判。
#   落点这一刻算不出的一律在执行前拒（静态前缀落在仓外、已在写范围里也拒：变量的值可以带 `/..` 爬回仓里，`$S/out-$f` 的 f 可以是 `x/../../<仓>/y`）：
#   目标里有命令替换（`$(echo $f | tr / _)`）、没赋过值或来源不明的变量、`while read` 读进来的变量、来自变量的 `for` 列表（`for f in $FILES`）、通配、
#   共用切词展开之后还留着的 `{…,…}` / `{a..b}`（一个词展开超过 BRACE_EXPANSION_LIMIT 份照原样留的、引号里写的、重定向目标上展开成几个词的：
#   `> 目录/{a,b}.txt` bash 报 ambiguous redirect、什么都不写，共用切词照原样留一个词，这里照算不出拒，偏严），
#   cd 到算不出的目录（或循环变量拼的目录）之后的相对路径，读不到规格的 replace-batch.py。拒绝说明给三条出路：备份原文件用 scripts/agents/copy-originals.py；
#   按文件写多份输出的，循环写成 `for 名字 in 写死的词` 且目标只用这些变量与写死的段，或逐条写字面路径；别的算不出的写法改成字面路径。弄坏开关 scope-undeterminable-only-noted 退回只记检出不拒。
#   算得出的照算：命令参数上引号外的花括号由共用切词照 bash 展开成几个落点、逐个判（`tee records/{a,b}.md` 判 records/a.md 与 records/b.md 两处，范围外的那一处照拒）；同一条命令里前面赋的写死值；`for 名字 in 写死的词…`（select 同）的循环变量逐个取值，每个取值各算一次落点（bind_loop_variables、loop_value_choices；
#   取值里的 `..` 与 / 照算，爬回仓里的照判；组合多过 MAXIMUM_LOOP_LANDINGS 种按算不出拒；弄坏开关 scope-loop-values-unknown 把写死的词也当算不出，
#   scope-loop-landings-trusted 下每个取值只当一个普通名字）；read / mapfile 读进来的与 `for f in $列表` 的循环变量作废，前面赋过的同名值不再用；
#   没在命令里赋过值的 `$HOME`、`${HOME}` 照 hook 自己的环境展开（home_expanded，弄坏开关 scope-home-unexpanded）。
#   heredoc 正文只在喂给 shell（bash、sh）时照命令逐行判，喂给 python 的照 python 写文件的判法判（那是真代码），喂给 cat、tee 这类别的命令的是数据、不判，
#   但 `cat > 目标 <<EOF`、`tee 目标 <<EOF` 的目标本身照判（弄坏开关 heredoc-body-judged-as-commands）；经管道喂给 python 的（`cat <<'EOF' | python3 -`，或 echo / printf 的文字）
#   照 python 写文件的判法判，喂给 bash / sh 的照命令逐行判（弄坏开关 heredoc-pipe-fed-as-data）。前台、run_in_background 一样拒。
#   `cd <写死的目录>` 之后 `$PWD` 按那个目录展开（remember_working_directory，弄坏开关 pwd-variable-unexpanded；cd 算不出时作废，没有 cd 的 `$PWD` 算不出、拒）；
#   `d=$(mktemp -d)`（或反引号写法、`-t` 加不含 / 的模板、不带值的 `--tmpdir`）赋的变量当作落在 ${TMPDIR:-/tmp} 下（仓外，mktemp_directory_value，弄坏开关 mktemp-directory-unknown；
#   不带 `-d` 的不认）。不带 `-t` 的模板（`$(mktemp -d <目录>/名.XXXXXX)`，或 cd 之后相对当前目录写的），与 `-p <目录>`、`--tmpdir=<目录>` 指的目录（可再带一个不含 / 的模板名），
#   所在的目录落在这个子 agent 认领的草稿目录里时（拆分提案第九十节 ⑰、㉕），建的目录按「草稿目录内」算：赋给变量的与直接写在目标里的都认（mktemp_created_directory 先看
#   mktemp_option_directory 再看 mktemp_template_directory，own_scratch_directory 认归属；弄坏开关 mktemp-option-directory-unknown 退回不认 `-p`、`--tmpdir=` 与不带值的 `--tmpdir`）。
#   套在 `$(cd $(mktemp …) && pwd -P)`（`pwd`、`pwd -L`、`;` 代 `&&`、反引号写法同，mktemp_inside_pwd）里的，按里面那条 mktemp 算（弄坏开关 mktemp-nested-substitution-unknown 退回算不出）。
#   `-p` 指到 /tmp、仓里、别的子 agent 或别的会话的草稿目录，目录里带变量与通配的，照旧算不出、拒。
#   「认领的草稿目录」：工作根下第一层那个目录的归属文件 .owner 写的会话是 hook 输入的会话（scripts/hosts/work-directory-ownership.sh），而且离模板目录最近的
#   写者锁归这个子 agent（认领文件写的是它的 agent_id；还没有认领文件的，主会话记录里它的派发回执对得上那把锁；读法在 lib_writer_lock.py，这里只读、不建认领文件）。
#   模板目录不在草稿目录里、在别的子 agent 的草稿目录里、模板里还有算不出的变量、hook 输入里没有 agent_id 的，算不出、拒。
#   误拒的风险：有写范围的子 agent 在 Bash 里往本仓写、又不在它的行里的，都会被拒；仓内位置不在它的行里（比如 scripts/、.claude/hooks/ 对 clerk）写进去本来就是越界，出路是交回主 agent。
#   落点算不出的写法（往草稿目录写也一样）被拒，出路是上面三条。
#   eval 的代码与启动器里面那条（一百三十七节件 1b 第 4 条，定义批 d137-defs 的探针量出 `eval 'sed -i … <仓里文件>'`、`find … -exec sed -i … {} ';'`、
#   `echo <仓里文件> | xargs sed -i …` 两道钩子都放）：eval 的参数照 eval 的样子用空格接成一段代码、照命令接着判（字面的落点照算；里面有没赋过值的变量的照算不出拒）；
#   命令词是启动器（xargs、find -exec / -execdir / -ok、parallel、flock、/usr/bin/time 这类，名单与 lib_project_code_runs.py 的 launcher_names() 同一份）的，
#   剥开（command_under_launcher）之后里面那条照判；xargs、find、parallel 读进来接进命令的词（`{}`、xargs 补在后面的参数）落点算不出，改文件的命令
#   （sed -i、tee、cp、mv、> 这类）目标是它的照算不出拒（认不出的一律拒），拒绝说明写改法：逐个写字面路径（scope_launcher_inner_text；
#   弄坏开关 scope-launcher-writes-unseen 下不认这几种）。启动器里面写的目标是字面路径的照落点判（`find … -exec cp {} <仓里文件> ';'` 照范围判）。
#   判不到的：变量里拼出来的命令（`c='sed -i …'; $c`：命令词认不出起什么，子 agent 由 project-code-guard.sh 照认不出的写法拒）、`eval "$c"` 这种代码在变量里的、
#   `bash x.sh` 起的脚本文件里的写法、patch 与 git apply、经 subprocess 起的写法、规格在同一条命令里才写出来的 replace-batch.py。
#
# ⑮ 另认：`git stash` / `git stash push|save`（不带 -k / --keep-index、不带路径或路径是整棵树）、`git rm --cached` 的路径是整棵树、
#   `git checkout [<提交>] -- <整棵树>` 同样清整个索引（git_other_form_clears_index；弄坏开关 whole-index-other-forms-unseen 下不认）；git 的子命令写成变量、命令替换的，
#   前面赋过字面值的照展开再判，展开不了的认不出是不是清索引，照清索引拒（git-subcommand-unexpanded-allowed 下不认）。
# ⑮ 暂存区非空（`git diff --cached --quiet` 退 1）时，不带具体路径清整个索引的写法：`git reset`（不带路径，--soft 之外的 --mixed、--hard、--keep、--merge 与 -q 都算，
#   带不带 HEAD 或别的提交都算）、`git restore --staged`（-S，带不带 --worktree）的路径只有 `.`、`:/`、`:/:`、`:(top)`、`*` 这类整棵树的写法；主 agent 与子 agent 一样拒。
#   暂存区里可能有别的会话暂存的改动，整份清掉就是替它们撤了暂存。
#   带具体路径的（`git reset HEAD -- a.md`、`git restore --staged -- a.md`、`git reset a.md`）、暂存区为空的、`git reset --soft` 放行；`git -C <目录>`、`--git-dir` / `--work-tree`
#   与 cd 跟着算，暂存区按那个目录现查；目录算不出（cd 到没赋过值的变量）的照拒（判不了空不空）。拒绝说明的出路：只清自己的路径 `git restore --staged -- <路径…>`。
#   判不到的：git 别名、`git read-tree`、命令词 git 本身写成变量（`g=git; $g reset`；子 agent 由 project-code-guard.sh 照认不出的写法拒，主 agent 放行）、
#   `bash x.sh` 起的脚本文件里的写法。弄坏开关：allow-whole-index-reset（一概放行）、
#   whole-index-staged-unchecked（不看暂存区空不空）、whole-index-pathspec-ignored（带路径的也当清整个索引）。
#
# ⑰ 往仓里的文件写字面内容，内容里有会话号、钟点或批次代号（主 agent 与子 agent 一样拒）：写闸 write-guard.sh 的第五道、第七道只看 Write / Edit / MultiEdit，
#   Bash 里 `cat >> records/….md <<'EOF'`、`printf … >> 文件`、`… | tee -a 文件` 这类追加不过它。
#   认的写法（literal_writes）：cat（不带文件操作数）读 heredoc、here-string 或管道前一段的字面、echo 与 printf 的参数，经 >、>>、>|、&>、&>> 写进的；
#   tee（带不带 -a）把 heredoc、here-string、管道前一段的字面写进的；bash -c 的代码、喂给 shell 的 heredoc、命令替换里的照认，cd 与前面赋过字面值的变量跟着算。
#   判法（literal_notation_hits）：落点在本仓里的，照同一份判据库 ../gate.d/lib-forbidden-notations.py 的 session-names、clock-times、batch-codes 三种形态判
#   （排除前缀、放行标记、批次代号只判 .md、.tsv、.txt 都由库管，与写闸判 Write / Edit 的内容同一口径），逐处列出写法、目标、第几行、哪一种与命中的串；
#   出路取库里每种命中形态登记的 remedy 第一条，另指到 Edit 与 scripts/kb/insert-row.py、replace-once.py（它们写之前也判这几样）。
#   不判的：落在仓外的、落点算不出的（变量、命令替换、通配、cd 到算不出的目录之后的相对路径）、内容不是字面的（cat 文件、cp、mv、别的命令的输出）、
#   花括号整组与子 shell 整组的重定向（`{ echo …; } >> 文件`）、sed -i / perl -i 的替换串、python 等程序里写文件的、写法在脚本文件里的；
#   重定向前的 fd 号由共用切词模块去掉了，`echo … 2>> 文件` 照标准输出算（偏严）。读不到判据库照常执行、记一条检出。
#   弄坏开关：allow-literal-notations（一概不判）、literal-notations-outside-judged（仓外的目标也判）；判据本身的开关是库的 LIB_FORBIDDEN_NOTATIONS_BREAK 一族。
#
# ⑯ 往现行账本直接追加节标题：命令里有「## <汉字节号>、」（节标题的写法同门禁 ledger-section-numbers 格），又有一处写会落在拆分提案账本登记表
#   （.claude/gate.d/split-proposal-ledgers.tsv，门禁 doc-process-records 的三格读同一张）里 current 那一行点名的账本上，就拒；主 agent 与子 agent 一样拒，前台、run_in_background 一样。
#   认的写法（ledger_append_refusal，沿用 ⑤ ⑫ 的 overwrite_steps：cd、前面赋过值的变量、bash -c、喂给 shell 的 heredoc 照跟）：重定向 `>`、`>>`、`>|`、`&>`、`&>>`，
#   tee（带不带 -a），喂给 python 的代码里 open(…, 'a' / 'w')、Path.write_text 这类字面路径的写。目标这一刻算不出的（命令替换、没赋过值的变量、通配）不判，也不记。
#   放行：写别的文件、封存的账本、写进账本的一行不带节标题、只读账本、cp / mv / sed -i / perl -i 这类（改节号、改正文走 replace-once.py 或 sed -i）、
#   经 scripts/kb/ledger-append-section.py 追加。登记表读不到、current 不是恰好一行，而命令里又有节标题的：记一条检出（照常执行）。
#   为什么：几个会话同时往账本末尾追加，各自读文件、各自取号，节号会撞，到提交时门禁才判出重号（records/2026-10-29-subagent拆分提案.md 第一百五十六节）。
#   出路：scripts/kb/ledger-append-section.py（现取最大节号加一，一次写入索引表末行与末尾新节，写前复核 sha256，撞了拒）。Edit / Write 里的同类写归 write-guard.sh 的第十道。
#   弄坏开关：allow-ledger-append（一概放行）。
#
# ⑱ 带路径撤掉工作区里没暂存的改动（records/2026-10-29-subagent拆分提案.md 第一百三十九节 ②）：命令位置上的 `git checkout [<提交>] [--] <路径…>`、
#   `git restore <路径…>`（不带 --staged / -S，或带 --worktree / -W）、`git stash [push] [--] <路径…>`、`git clean <路径…>`（不带 -n / --dry-run），
#   点名的路径在工作区里有没暂存的改动就拒：`git diff --name-only -z -- <路径…>` 现查（工作区比暂存区）；git clean 查的是会被删的没跟踪文件，
#   `git status --porcelain=v1 -z --untracked-files=all -- <路径…>` 的 `??` 行，带 -x / -X 的另加 `--ignored` 认 `!!` 行。两条都设 GIT_OPTIONAL_LOCKS=0，不碰索引。
#   主 agent 与子 agent 一样拒，前台、run_in_background 一样。cd、`git -C`、`--git-dir` / `--work-tree` 跟着算，前面赋过值的变量照展开（worktree_discard_refusal）；
#   git checkout 不带 -- 时第一个位置参数是提交还是路径照 git 的认法猜（同 ⑮ 的 git reset：像提交的写法，或盘上没有这个路径而 git rev-parse 认得）。
#   判不了的照拒：路径写成展开不了的变量或命令替换、路径从 --pathspec-from-file 读、目录算不出（cd 到没赋过值的变量、git -C 带变量）、git 起不来。
#   放行：点名的路径都没有没暂存的改动（只暂存了、工作区与暂存区相同的也放）、不在仓里的路径、只动暂存区的 `git restore --staged`、git clean -n、
#   切分支（git checkout <分支>、-b / -B / --orphan / --detach）、没点名路径的（git stash、git clean -fd：不归这一道，暂存区非空时归 ⑮）、
#   git 报错的（不是 git 仓、路径在仓外：命令自己也跑不成）。
#   为什么：主工作区几个会话共用，checkout / restore 一份点名的文件会整份冲掉别的会话没暂存的改动，无声改回 HEAD 之后查不到是哪条命令。
#   出路：先 cp 一份留底（scripts/agents/copy-originals.py copy）；只撤自己那几块，把自己的块存成补丁再 `git apply -R`，别人的块照留。
#   判不到的：git 别名、命令词 git 写成变量、`bash x.sh` 起的脚本文件里的写法、不点名路径的（`git reset --hard`、`git checkout -f`、`git switch --discard-changes`）、
#   `git checkout <提交> -- <路径>` 与 `git restore --source <提交> --staged --worktree` 连暂存区一起盖掉的已暂存改动（这一道只看没暂存的）。
#   接受的误拒：git clean 不带 -d 时没跟踪目录里的文件不会被删，这里照算；git clean -X 只删被忽略的，这里没跟踪的也算；目录算不出时不带 -- 的 git checkout <分支> 照拒。
#   弄坏开关：allow-path-discard（一概放行）、path-discard-changes-unchecked（不查改动，点名的路径都当有改动）、path-discard-staged-counted（git restore --staged 也当动工作区）。
#
# ⑲ 往 records/ 下的 .md 追加（主 agent 与子 agent 一样拒，前台、run_in_background 一样）：落点在本仓 records/ 下（子目录里的也算）、后缀是 .md（不分大小写）的追加，
#   不看追加的是什么内容就拒。为什么：这样追加的文本不过写闸 write-guard.sh 第五道、第七道的钟点、会话号与批次代号判定，python 写进去的连 ⑰ 也看不见；
#   一份调度记录经 python 以追加方式写进一处带时区的钟点与 110 处字母批次代号，到门禁 doc-text 的 clock-times 格才判出来（records/2026-10-29-subagent拆分提案.md 第一百八十二节）。
#   认的写法（record_append_refusal，沿用 ⑯ 的 overwrite_steps(every_write=True)：cd、前面赋过值的变量、bash -c、喂给 shell 的 heredoc、命令替换照跟）：
#   重定向 `>>`、`&>>`，`tee -a` / `tee --append`（OverwriteCandidate.appends），python 代码（`python3 -c`、喂给 python 的 heredoc 与经管道喂的正文）里
#   open(…, 带 a 的模式)、Path(…).open(带 a 的模式)，路径是字面量或只赋过一次的名字（python_in_place_writes 认得出的那几种）。落点规整 .. 并解开符号链接再比。
#   放行：经 scripts/kb/append-record.py 追加（它写之前照同一份判据库判要追加的文本；写范围里没有 records/ 的子 agent 起它由 ⑫ 拒）、整份写（`>`、不带 -a 的 tee、python 'w'：不是追加，字面内容归 ⑰）、
#   records/ 下别的后缀、records/ 之外与仓外的 .md、只读。
#   判不了的（不判，也不记）：目标这一刻算不出的（命令替换、没赋过值的变量、通配、cd 到算不出的目录之后的相对路径）、python 里路径或模式算不出的、
#   os.open 带 O_APPEND 与 os.write、写法在脚本文件里的（`bash x.sh`、`python3 x.py`）、sed -i / perl -i 在末尾加行、cp / mv / install 换掉整份、git apply、patch。
#   出路（拒绝说明里给整条写法，记录填认出的第一份）：追加的文本先写进草稿目录里的文件，`python3 scripts/kb/append-record.py <记录> <文本文件>`（标准输入写 `-`）；
#   它不收一、二级标题：新开一节用 Edit 接在文件末尾（写闸照判同几样），拆分提案现行账本加节用 ledger-append-section.py。
#   Write / Edit 写 records/ 不归这一道：写闸第五道、第七道已判写进去的内容。
#   弄坏开关：allow-record-append（一概放行）、record-append-python-unseen（不认 python 的 open(…, 'a')）、record-append-tee-unseen（不认 tee -a）、record-howto-unlisted-tool（出路写成不在直接跑名单上的脚本）。
#   出路本身要过得了别的闸：自证把拒绝说明里的整条写法照抄（文本文件换成自证草稿里的一份）喂给同挂 PreToolUse[Bash] 的 project-code-guard.sh，主 agent 与子 agent 都要放行；
#   hook 旁边没有 project-code-guard.sh 时这两格不判，成功句报「本次未跑」。
# ⑳ 手拷仓副本（主 agent 与子 agent 一样拒，前台、run_in_background 一样）：命令位置上的 `rsync`、`cp -r` / `-R` / `-a`（`--recursive`、`--archive`，与别的短选项合写的、
#   写在操作数后面的都算）的源是本仓的 crates/ 或仓根整个。为什么：手列的拷贝范围跟不上编译与测试要读的仓根条目（path 依赖指到的 prune-server/、测试 include_str! 读的 litmus/ 这类），
#   拷出来的副本缺东西，到 cargo 编不过、测试读不到才知道（证红任务会退 2「拷贝范围跟不全 path 依赖：…编译期读的 litmus/… 在仓里没有」）。
#   认的写法（manual_repository_copy_refusal）：源操作数（rsync 是除最后一个目的之外的，cp 是除目标之外的、给了 -t 的是全部）照 ~、前面赋过值的变量与 $HOME 展开，
#   相对路径从 cd 之后的目录起算（起点是 hook 输入的 cwd，没有就是仓库根），规整 .. 与结尾的 / 并解开符号链接之后等于本仓根或本仓的 crates/；
#   通配照 bash 的默认展开（不开 dotglob），展开到它们的、通配写在它们下一层整层的（仓根里的 `*`、`crates/*`）同样认；bash -c、喂给 shell 的 heredoc、命令替换、
#   nice / timeout / env 这类前缀里的照判（共用模块 lib_shell_words.py 的 commands_at_command_position）。
#   放行：出路本身 `python3 scripts/hosts/minimal-copy.py <仓根> <副本> --with-test-inputs`；拷草稿目录里别的东西（别的副本的 crates/、笔记）；拷 crates/ 下单个包的目录或单个文件；
#   rsync 带 -n / --dry-run / --list-only 的、只给一个操作数的（只列不拷）；cp 不带递归的（目录拷不过去）；远端写法（主机:路径）；引号里当数据写的、写进文件的 heredoc 正文、grep 的参数。
#   判不了的（不判，也不记）：源这一刻算不出的（没赋过值的变量、命令替换、cd 到算不出的目录之后的相对路径）、tar / git archive / cpio / install 这类别的拷法、
#   写法在脚本文件里的（`bash x.sh`、`python3 x.py`）、只拷 crates/ 里几个包或仓根下几样别的条目拼成的副本（每一条都不是 crates/ 或仓根本身）。
#   出路（拒绝说明里给整条写法，仓根填这一份钩子所在的仓根，副本写 <草稿目录>/copy）：`python3 scripts/hosts/minimal-copy.py <仓根> <草稿目录>/copy --with-test-inputs`，
#   要门禁表的加 --with-gate-tables，要在副本里起仓里脚本的逐条加 --with scripts=<为什么>；吃副本的脚本（prove-red.sh --copy）开跑前另按同一份闭包核副本缺不缺。
#   弄坏开关：allow-manual-repository-copy（一概放行）、manual-copy-howto-unlisted-tool（出路写成不在直接跑名单上的脚本）。
#   出路本身要过得了别的闸：自证把拒绝说明里的整条写法照抄（副本换成自证草稿里的一个路径）喂给同挂 PreToolUse[Bash] 的 project-code-guard.sh，主 agent 与子 agent 都要放行；
#   hook 旁边没有 project-code-guard.sh 时这两格不判，成功句报「本次未跑」。
#
# 切词、切简单命令、认命令位置、剥前缀与包装、跟 cd：同目录的 lib_shell_words.py，与 project-code-guard.sh 共用一份，按文件路径导入，
# 这里只留检出与拒绝的判定；那几个函数在 .claude/hooks/ 别的文件里再定义一份，门禁 code-tooling 的 agent-write-scope 格判红。
# 读不到它时这条命令照常执行，stderr 报一句并记一条检出。
#
#   bash-command-detector.sh             # 从 stdin 读 hook 的 JSON；拒绝七种之一退出 2（拒绝），其余退出 0
#   bash-command-detector.sh --scan-scripts <仓根> <目录>…  # ⑥ 的判定扫目录里的 .sh 与 .py（门禁 code-tooling 的 research-gate-lint 格调）；有红退出 1
#   bash-command-detector.sh --selftest  # 走一遍检出、不检出、拒绝与放行；
# 弄坏开关 BASH_COMMAND_DETECTOR_BREAK=<项>（几个用逗号连；项名写错的，自检「BASH_COMMAND_DETECTOR_BREAK 里的项都认得」一格红），设了哪一项自检都必须判红。
# 弄坏开关只在自证里认（门禁 code-tooling 的 break-switch-outside-selftest 格）：平常跑（不是 --selftest，也不是自证起的子进程）设了它就退 2、stderr 点名变量只在自证里认；
# 自证起的子进程靠 BASH_COMMAND_DETECTOR_SELFTEST_MARK 指到自证建的标记文件（内容对得上）才认，别处设了标记变量不顶用（refuse_break_outside_selftest）。
# 逐项写开关名、弄坏哪一处、打开后自检哪几格红。格名照自检失败行「✗ 自检：」之后、「应当是」之前那一段原样写；
# keep-selftest-scratch 那三格由 lib_selftest_scratch.py 报，失败行在格名后面接「：」与留下的名字；
# 共用模块的 LIB_SHELL_WORDS_BREAK 那几条同样照自检输出逐字写格名（stdin: 前缀与「（退出码 N）」后缀都在格名里）：
#   BASH_COMMAND_DETECTOR_BREAK=disable-check  findings_for() 不检出；③ 放出追踪、② 前台等待循环、④ 单独的 &、⑤ 覆盖产物（连同它的检出）、⑥ 终止进程、⑦ 同 inode 改脚本、⑧ 就地改仓内
#       文件、⑨ 先编后换、⑪ 通删工作根、⑫ Bash 里写写范围之外的文件都放行（① 起看门狗的错误写法照拒）：实测红的格（照失败行原文）
#       「没有 timeout 的 until 等待」「喂给 bash 的 heredoc 里真是一段没超时的等待循环」「heredoc 之后的命令里是没超时的等待循环」「后台起的 nohup … & 加 disown」「后台起的命令末尾单独一个 &」「后台起的 & 在 wait 之后」「bg-notify-r1 X01」「bg-notify-r1 X02」
#       「bg-notify-r1 X03」「bg-notify-r1 X04」「bg-notify-r1 X05」「bg-notify-r1 X06」「bg-notify-r1 X07」「bg-notify-r1 X08」「bg-notify-r1 X09」「bg-notify-r1 X10」「bg-notify-r1 X11」「bg-notify-r1 X12」
#       「bg-notify-r1 F02（第三种不记；它的轮询循环没有 timeout，记的是第二种）」「一百三十七：裸 setsid 不带 --wait，照记一条」「一百三十七：内存包装里的 setsid 不带 --wait，照记一条」「setsid -f 放后台」「等待循环:前台 until … sleep 没超时」
#       「等待循环:run_in_background 里轮询后台任务的输出文件」「等待循环:run_in_background 里轮询后台任务输出、外面套了 timeout 也拒」「等待循环:前台按 pid 轮询的 while kill -0」「等待循环:前台 timeout 只在循环条件里」「等待循环:前台多行写的 while true」
#       「等待循环:前台 sleep 在循环条件里」「等待循环:前台喂给 bash 的 heredoc 里的循环」「等待循环:前台 bash -c 里没有 timeout」「等待循环:前台 capped.sh 包着的 bash -c 里没有 timeout」「等待循环:前台 run-with-memory-cap.sh 包着的 bash -c 里没有 timeout」
#       「等待循环:前台命令替换里的循环」「等待循环:前台 if 里的循环」「等待循环:前台循环体里套 for、sleep 在 for 之后」「等待循环:前台 time -p 包着的 bash -c 里没有 timeout」「等待循环:前台 $(bash <<EOF) 里的循环」「等待循环:前台 <(bash <<EOF) 里的循环」
#       「等待循环:前台 cd "$(…)" 之后 bash <<EOF 里的循环」「等待循环:前台 echo "$(date)" | bash <<EOF 里的循环」「等待循环:前台 $'a\'b' 之后 $(bash <<EOF) 里的循环」「起 claude:子 agent 命令位置上的 claude -p」
#       「起 claude:子 agent 经 "$CLAUDE_CODE_EXECPATH" 起」「起 claude:子 agent 经 ${CLAUDE_CODE_EXECPATH} 起」「起 claude:子 agent 起以 /claude 结尾的可执行文件」「起 claude:子 agent nice 包着起 claude」
#       「起 claude:子 agent bash -c 里起 claude」「起 claude:子 agent 命令替换里起 claude」「起 claude:子 agent 喂给 bash 的 heredoc 里起 claude」「就地改仓内文件:执行员 sed -i 改仓内源文件」「就地改仓内文件:执行员 python open(仓内路径, 'w')」
#       「就地改仓内文件:执行员 python Path(仓内绝对路径).write_text」「就地改仓内文件:执行员 sed -i 改工作根同名前缀的兄弟目录」「就地改仓内文件:执行员 python os.replace 把草稿挪进 scripts/ 下的入库装置」「就地改仓内文件:执行员 python os.rename 挪进知识库（源是变量）」
#       「就地改仓内文件:执行员 python shutil.move 挪进仓根起的绝对路径」「就地改仓内文件:执行员 python shutil.move 用 dst= 挪进 scripts/」「就地改仓内文件:执行员 python open(scripts/ 下的文件, 'w')」「就地改仓内文件:cd 到仓根之后 python 相对路径写 crates/ 照拒」
#       「就地改仓内文件:cd 到算不出的目录之后 python 相对路径写 crates/ 照旧按仓内判」「就地改仓内文件:cd 到草稿目录里的仓副本之后 python 写仓根起的绝对路径照拒」「就地改仓内文件:喂给 python 的 heredoc 里 open(仓内路径, 'w') 照拒」
#       「就地改仓内文件:喂给 bash 的 heredoc 里的 sed -i 照拒」「就地改仓内文件:bash -c 里的 python3 -c 写仓内文件照拒」「就地改仓内文件:cd 到仓内目录之后 sed -i 相对路径照拒」「就地改仓内文件:不 cd 的仓内相对路径 sed -i 照拒」
#       「就地改仓内文件:cd 到算不出的目录之后 sed -i 相对路径照旧按仓内判」「就地改仓内文件:先 cd 到草稿目录、再 cd 回仓内之后 sed -i 相对路径照拒」「就地改仓内文件:cd 到草稿目录之后 sed -i 的相对路径用 .. 跳出工作根照拒」「就地改仓内文件:cd 到草稿目录之后 sed -i 的目标带通配，照旧按仓内判」
#       「就地改仓内文件:管道喂给 python3 - 的 heredoc 里 open(仓内路径, 'w') 照拒」「就地改仓内文件:管道喂给 python3（不带 -）的 heredoc 里 open(仓内路径, 'w') 照拒」「就地改仓内文件:管道喂给 python3 - 的 heredoc 里 Path(仓内路径).write_text 照拒」
#       「就地改仓内文件:echo 的文字管道喂给 python3 - 照拒」「就地改仓内文件:管道喂给 bash 的 heredoc 里 sed -i 仓内文件照拒」「就地改仓内文件:命令开头的当前目录在仓内，sed -i 相对路径照拒」「先编后换:执行员 cp 草稿副本盖主工作区的入库装置」「先编后换:执行员 cat > 新建入库装置」
#       「先编后换:执行员 >> 追加」「先编后换:执行员 tee -a」「先编后换:执行员 mv 草稿副本进 bin 目录」「先编后换:执行员 cd 进 bin 目录之后 cp 到 .」「先编后换:执行员 sed -i」「先编后换:执行员 perl -pi -e」「先编后换:执行员 rsync」「先编后换:执行员 rm 入库装置」
#       「先编后换:执行员 mv 把入库装置挪走」「先编后换:执行员 bash -c 里 cp」「先编后换:执行员变量里的目录」「先编后换:执行员目标算不出、对得上已有的入库装置」「先编后换:执行员 python shutil.copy」「先编后换:执行员 python open(…, 'a')」「先编后换:执行员 python os.replace」
#       「先编后换:执行员 cwd 在主工作区的 crates/ 里、相对路径写 .rs 拒」「两道包装:包着的前台没超时的等待循环拒」「两道包装:--quiet 包着的前台没超时的等待循环拒」「两道包装:包着的 kill 负进程号拒」「两道包装:包着的 setsid -f 拒」「两道包装:包着的 cp 盖主工作区入库装置拒（先编后换）」
#       「Bash写范围:clerk 用 replace-once.py 写知识库（现场：越出写范围）」「Bash写范围:clerk 直接执行 replace-once.py 写知识库」「Bash写范围:clerk 先 cd 到 records 再用相对路径写知识库」「Bash写范围:kb-scribe 用 replace-once.py 写 records 拒」
#       「Bash写范围:clerk 用 insert-row.py 写知识库」「Bash写范围:clerk 用 replace-batch.py，规格点名知识库」「Bash写范围:clerk 脚本路径在变量里，用 replace-once.py 写知识库」「Bash写范围:clerk 脚本路径在变量里，用 insert-row.py 写知识库」
#       「Bash写范围:clerk 脚本路径在变量里，用 replace-batch.py、规格点名知识库」「Bash写范围:clerk 直接执行变量里的 replace-once.py 写知识库」「Bash写范围:clerk sed -i 改 crates」「Bash写范围:clerk 重定向 > 写 crates」
#       「Bash写范围:clerk 重定向 >> 写知识库」「Bash写范围:clerk tee 写 crates」「Bash写范围:clerk tee -a 写知识库」「Bash写范围:clerk cp 到知识库」「Bash写范围:clerk cp 进知识库目录」「Bash写范围:clerk mv 到 crates」
#       「Bash写范围:clerk heredoc 写知识库」「Bash写范围:clerk 先 cd 进知识库目录再重定向」「Bash写范围:clerk bash -c 里的写」「Bash写范围:clerk 变量里的目录（前面赋过值）」「Bash写范围:clerk python open(…, 'w') 写知识库」
#       「Bash写范围:clerk 喂给 bash 的 heredoc 里同样一句 sed -i 知识库照拒」「Bash写范围:clerk 喂给 bash 的 heredoc 里的重定向写知识库照拒」「Bash写范围:clerk 喂给 python 的 heredoc 里 open(知识库, 'w') 照拒」
#       「Bash写范围:clerk cat > 知识库 <<EOF 的目标本身照拒（正文是数据，目标不是）」「Bash写范围:clerk tee 知识库 <<EOF 的目标本身照拒」「Bash写范围:clerk 经 two-logs.sh 包着写知识库」「Bash写范围:clerk 的目标在没赋过值的变量里：拒」「Bash写范围:clerk 重定向目标带通配：拒」
#       「Bash写范围:clerk cd 到没赋过值的变量目录之后写相对路径：拒」「Bash写范围:clerk cd 到没赋过值的变量目录之后写绝对路径的知识库：照拒」「Bash写范围:clerk 管道喂给 python3 - 的 heredoc 里 open(知识库, 'w') 照拒」
#       「Bash写范围:clerk 管道喂给 python3（不带 -）的 heredoc 里 open(知识库, 'w') 照拒」「Bash写范围:clerk cd 到 records 之后管道喂给 python3 - 的 heredoc 里相对路径写知识库照拒」「Bash写范围:clerk 管道喂给 bash 的 heredoc 里 sed -i 知识库照拒」
#       「Bash写范围:clerk 管道喂给 bash -s 的 heredoc 里重定向写知识库照拒」「Bash写范围:clerk echo 的文字管道喂给 python3 - 写知识库照拒」「Bash写范围:clerk printf 的文字管道喂给 python3 - 写知识库照拒」
#       「Bash写范围:clerk 自己的 heredoc 盖过管道：echo | python3 - <<EOF 里写知识库照拒」「Bash写范围:clerk cd 到知识库目录之后写 $PWD/… 照拒」「Bash写范围:clerk cd 到知识库目录之后 cp 进 $PWD/… 照拒」「Bash写范围:没有项目定义的内置 agent 写仓里拒」
#       「Bash写范围:没有项目定义的内置 agent 写仓外别处拒」「Bash写范围:没有项目定义的内置 agent 写会话临时目录根拒（scratchpad 之外）」「Bash写范围:没有项目定义的内置 agent 写会话 scratchpad 拒」「Bash写范围:没有项目定义的内置 agent 用 replace-once.py 写仓里拒」「Bash写范围:定义已删的旧实例写仓里拒」「Bash写范围:一百三十七：clerk eval 字面代码里 sed -i 知识库，拒」
#       「Bash写范围:一百三十七：clerk find -exec sed -i 读进来的路径，落点算不出，拒」「Bash写范围:一百三十七：clerk echo 喂给 xargs sed -i，落点算不出，拒」「Bash写范围:一百三十七：clerk xargs sed -i 写范围里的 records 也拒（落点算不出）」
#       「Bash写范围:一百三十七：clerk find -exec cp 到写死的知识库路径，拒」「Bash写范围:一百三十七：钩子进程在工作根下，clerk sed -i 没赋过值的变量照拒」「Bash写范围:一百三十七：钩子进程在工作根下，clerk xargs sed -i 照拒」
#       「Bash写范围:python3 起没赋过值的变量里的脚本、第一个参数落在知识库，拒」「Bash写范围:python3 起没赋过值的变量里的脚本、第一个参数落在写范围里的 records 也拒」「Bash写范围:python3 起命令替换给的脚本、第一个参数是仓根起的绝对路径，拒」「Bash写范围:目标在没赋过值的变量里，拒」
#       「Bash写范围:重定向目标带通配，拒」「Bash写范围:replace-batch.py 规格读不到，拒」「Bash写范围:cd 到没赋过值的变量目录之后写相对路径，拒」「Bash写范围:cd 算不出之后 python 写相对路径，拒」
#       「Bash写范围:检出原文：mkdir -p <草稿目录>/originals && for f in …; do cp -p "$f" "<草稿目录>/originals/$(echo $f | tr / _)"; done，拒」
#       「Bash写范围:检出原文：S=<草稿目录> && mkdir -p $S/originals && for f in …; do cp -- "$f" "$S/originals/$(echo $f | tr / _)"; done，拒」「Bash写范围:for 的列表里有爬回仓里的 ..，落在写范围外，拒」
#       「Bash写范围:for 的列表里有 .. 段、落点回到写范围之外，拒」「Bash写范围:for 的列表里有写范围外的仓内路径，cp 写到它，拒」「Bash写范围:f 来源不明的 > <草稿目录>/out-$f，拒」「Bash写范围:变量来源不明、静态前缀在工作根下（仓外），拒」「Bash写范围:变量来源不明、静态前缀在 /tmp 下（仓外），拒」
#       「Bash写范围:变量来源不明、静态前缀已在写范围里，拒」「Bash写范围:命令替换在最后一段、静态前缀已在写范围里，拒」「Bash写范围:cp 进范围内目录、源名算不出，拒」「Bash写范围:cd 到写死的范围内目录之后写带来源不明变量的相对路径，拒」「Bash写范围:cd 到写死的范围内目录之后写变量开头的相对路径，拒」
#       「Bash写范围:for 循环的列表里有算不出的词，拒」「Bash写范围:通配目标在范围内目录下，拒」「Bash写范围:静态前缀在仓内、写范围之外，拒」「Bash写范围:目标以变量开头，拒」「Bash写范围:变量单独占一段，拒」「Bash写范围:变量之后有字面的 .. 段，拒」「Bash写范围:仓外目录下变量单独占一段，拒」
#       「Bash写范围:cd 到写死的范围外目录之后写带变量的相对路径，拒」「Bash写范围:cd 到算不出的目录之后写带变量的相对路径，拒」「Bash写范围:cd 进循环变量拼的目录之后写相对路径，拒」「Bash写范围:目标明确、写范围之外的拒」「Bash写范围:仓外的符号链接指到仓内知识库，<链接>/../x 回到仓里，拒」
#       「Bash写范围:范围内的链接指到仓里范围外的目录，经它 > 写，拒」「Bash写范围:范围内的链接指到仓里范围外的目录，经它 cp 写，拒」「Bash写范围:范围内的链接指到仓里范围外的目录，经它 replace-once.py 写，拒」「Bash写范围:范围内的链接指到仓里范围外的目录，cp 进链接（目标是目录），拒」
#       「Bash写范围:没有 cd 的 $PWD，拒」「Bash写范围:cd 到算不出的目录之后 $PWD，拒」「Bash写范围:先 cd 到写死的目录、再 cd 到算不出的目录，$PWD 作废，拒」「Bash写范围:cd 到写死的草稿目录之后 $PWD/子目录/$p，$p 来源不明，拒」「Bash写范围:mktemp -d -p 指的目录算不出，拒」
#       「Bash写范围:mktemp 不带 -d 建的是文件，拒」「Bash写范围:mktemp -d 的模板在当前目录里，拒」「Bash写范围:while read 的循环变量，拒」「Bash写范围:for 的列表来自变量，拒」「Bash写范围:前面赋过写死值的变量又被 for $列表 改写，拒」「Bash写范围:派发回执对不上那把锁（别的子 agent）：拒」
#       「Bash写范围:模板在别的子 agent 认领的草稿目录里，拒」「Bash写范围:模板目录的归属文件是别的会话，拒」「Bash写范围:模板在 /tmp 下（不是草稿目录），拒」「Bash写范围:hook 输入里没有 agent_id，模板在草稿目录里也拒」「Bash写范围:-p 指到别的子 agent 认领的草稿目录，拒」
#       「Bash写范围:--tmpdir= 指到别的会话的草稿目录，拒」「Bash写范围:-p 指到 /tmp（不是草稿目录），拒」「Bash写范围:-p 指到仓里的目录，拒」「Bash写范围:hook 输入里没有 agent_id，-p 指到草稿目录也拒」「Bash写范围:套一层、-p 指到别的子 agent 认领的草稿目录，拒」
#       「Bash写范围:套一层、cd 之后接的不是 pwd，拒」「放出追踪:现场：& disown 之后又 & wait」「放出追踪:nohup … &」「放出追踪:nice 包着的 nohup … &」「放出追踪:coproc」「放出追踪:setsid -f」「放出追踪:setsid --fork」「放出追踪:tmux new-session -d」
#       「放出追踪:screen -dmS」「放出追踪:systemd-run 不等结束」「放出追踪:bash -c 里的 & disown」「放出追踪:命令替换里的 nohup … &」「放出追踪:capped.sh 包着的 setsid -f」「放出追踪:喂给 bash 的 heredoc 里的 disown」
#       「单独的 &:结尾单独一个 &」「单独的 &:& 之后接 echo $!」「单独的 &:& 在 capped.sh 前缀的那条命令上」「单独的 &:& 在 capped.sh 包着的 bash -c 里」
#       「单独的 &:& 在 wait 之后」「单独的 &:圆括号子 shell 里的 &、wait 在外面」「单独的 &:& 之后的 wait 在另一对圆括号里」「单独的 &:命令替换里的 &、wait 在外面」「单独的 &:bash -c 里的 &、wait 在外面」「单独的 &:喂给 bash 的 heredoc 正文里的 &」
#       「单独的 &:喂给 python 的 heredoc 挂在 & 前面」「单独的 &:& 之后只 wait -n」「单独的 &:整个循环放后台」「单独的 &:一对圆括号整组放后台」「单独的 &:几个 & 只 wait 一个 pid：剩下的照拒」「终止进程:kill -- 负进程号（进程组）」「终止进程:kill -TERM 负进程号」
#       「终止进程:kill -s TERM 负进程号」「终止进程:自己的任务的进程组 -"$job"」「终止进程:kill 0」「终止进程:kill -TERM 0」「终止进程:kill -9 -1（全部）」「终止进程:kill $PPID（Claude 会话本身）」「终止进程:kill $(pgrep …)」「终止进程:kill $(jobs -p)」
#       「终止进程:kill `pidof sshd`」「终止进程:kill 数组整批展开」「终止进程:kill "$@"」「终止进程:kill 一次两个」「终止进程:kill %1 %2」「终止进程:kill 带通配」「终止进程:for 循环里 kill」「终止进程:while read cgroup.procs 里 kill（现场那一段）」
#       「终止进程:多行循环里 kill」「终止进程:cgroup.procs 交给 xargs kill」「终止进程:xargs -n1 kill」「终止进程:pgrep 交给 xargs -r kill -9」「终止进程:find -exec kill」「终止进程:pkill 按名字」「终止进程:pkill -x claude」「终止进程:killall」
#       「终止进程:fuser -k」「终止进程:先从 cgroup.procs 挑一个再 kill」「终止进程:先 pgrep 挑一个再 kill」「终止进程:python -c 推导式里读 cgroup.procs 再 os.kill」「终止进程:喂给 python 的 heredoc 里 os.killpg」
#       「终止进程:python -c os.kill(0, …)」「终止进程:systemctl stop ssh」「终止进程:sudo systemctl restart sshd」「终止进程:systemctl kill session-1.scope」「终止进程:systemctl stop user@1000.service」
#       「终止进程:systemctl set-property user.slice」「终止进程:systemctl stop 私有配置登记的本地模型服务（不写后缀）」「终止进程:systemctl restart 私有配置登记的第二个单元」「终止进程:systemctl disable --now ssh」
#       「终止进程:systemctl --user stop 带通配 singlefs-*」「终止进程:systemctl --user kill 带通配 *.scope」「终止进程:systemctl --user stop 命令替换列出来的一批」「终止进程:systemctl isolate」「终止进程:systemctl --user exit」
#       「终止进程:systemctl reboot」「终止进程:loginctl terminate-session」「终止进程:loginctl kill-user」「终止进程:往 cgroup.kill 写」「终止进程:tee 进 cgroup.freeze」「终止进程:sudo reboot」「终止进程:shutdown -h now」
#       「终止进程:写死的进程号是 sshd」「终止进程:写死的进程号是 VSCode 服务端」「终止进程:写死的进程号是这个会话自己的 Claude」「终止进程:写死的进程号是另一个 Claude 会话」「终止进程:写死的进程号是另一个 Claude 会话起的 cargo」「终止进程:写死的进程号是私有配置登记的本地模型服务」「终止进程:kill -HUP 1」
#       「终止进程:proc.py stop 写死的 sshd」「终止进程:proc.py stop 一次两个」「终止进程:循环里 proc.py stop」「终止进程:bash -c 里 kill 进程组」「终止进程:nice 包着 kill -9 -1」「终止进程:喂给 bash 的 heredoc 里 kill 0」
#       「终止进程:timeout 包着 kill 负进程号」「终止进程:run-with-memory-cap.sh 包着 pkill」「终止进程:shell 函数与变量里的脚本包着的 bash -c 里 kill 0」「终止进程:shell 函数包着的 python -c 里 os.killpg」「终止进程:kill 写死的进程号读一次进程表」
#       「终止进程:proc.py stop 写死的进程号读一次进程表」「终止进程:一条命令里几处写死的进程号共读一次进程表」「终止进程:私有配置文件不在:systemctl stop ssh 照拒」「终止进程:私有配置里没有那两个键:systemctl stop ssh 照拒」「覆盖产物:> 覆盖未跟踪的产物」「覆盖产物:tee 不带 -a」
#       「覆盖产物:cp 的目标（现场原句的形态）」「覆盖产物:bash -c 里的 >」「覆盖产物:>|」「覆盖产物:&>」「覆盖产物:2> 同样截断」「覆盖产物:>& 后面跟文件名」「覆盖产物:tee -i 不是追加」「覆盖产物:tee 写两份、其中一份是未跟踪的」「覆盖产物:mv 的目标」「覆盖产物:install 的目标」
#       「覆盖产物:cp 进已有目录、源的文件名撞上」「覆盖产物:cp -t 目录」「覆盖产物:cp 源里的通配按这一刻展开」「覆盖产物:cd 进去之后写相对路径」「覆盖产物:cd 进子目录之后写」「覆盖产物:绝对路径」「覆盖产物:喂给 bash 的 heredoc 正文里的 >」「覆盖产物:命令替换里的 >」「覆盖产物:cat 把 heredoc 写进未跟踪的产物」
#       「覆盖产物:花括号整组重定向」「覆盖产物:圆括号整组重定向」「覆盖产物:timeout、capped.sh 包着的 bash -c 整条重定向」「覆盖产物:前面赋过值的变量代入」「覆盖产物:export 过的变量代入」「覆盖产物:cd 到变量路径算不出来、按仓库根解析」「覆盖产物:进程替换里的 tee」「覆盖产物:整个循环重定向」
#       「覆盖产物:sudo 包着的 tee」「覆盖产物:git add 的是另一份，这一份照拒」「覆盖产物:dd of= 覆盖未跟踪的产物」「覆盖产物:dd conv=notrunc 覆盖未跟踪的产物」「覆盖产物:truncate -s 0 覆盖未跟踪的产物」「覆盖产物:truncate 选项连写覆盖未跟踪产物」「覆盖产物:一条命令里两份未跟踪的目标各自点名」
#       「覆盖产物:一条命令里两份未跟踪的目标同时判，git ls-files 只起一次」「覆盖产物:已跟踪的、未跟踪的、不存在的目标混在一起同时判，只点名未跟踪的」「覆盖产物:已跟踪的、未跟踪的、不存在的目标混在一起，git ls-files 只起一次」「覆盖产物检出:算不出的变量对得上一份未跟踪的产物」
#       「覆盖产物检出:变量在前、research/results/ 在后」「覆盖产物检出:git 判不了（不是 git 仓）只记不拦」「就地改脚本:> 覆盖已有的 .sh」「就地改脚本:>|」「就地改脚本:&>」「就地改脚本:2> 同样截断」「就地改脚本:>& 后面跟文件名」「就地改脚本:cat > 旧 <<EOF」「就地改脚本:tee 不带 -a」
#       「就地改脚本:tee -i 不是追加」「就地改脚本:cp 新 旧」「就地改脚本:cp -f」「就地改脚本:cp -a」「就地改脚本:cp 进已有目录、文件名撞上」「就地改脚本:cp -t 目录」「就地改脚本:dd of=」「就地改脚本:dd conv=notrunc」「就地改脚本:truncate -s 0」
#       「就地改脚本:truncate -cs 0（选项连写）」「就地改脚本:写已有的 .py」「就地改脚本:带执行位、没有后缀的」「就地改脚本:经符号链接写它指向的脚本」「就地改脚本:scratch 根下的脚本」「就地改脚本:cd 进去之后写相对路径」「就地改脚本:bash -c 里的 >」「就地改脚本:喂给 bash 的 heredoc 正文里的 cp」
#       「就地改脚本:命令替换里的 >」「就地改脚本:前面赋过值的变量代入」「就地改脚本:nice、timeout 包着的 cp」「就地改脚本:python3 -c 的 open(…, 'w')」「就地改脚本:python3 -c 的 open(…, 'r+')」「就地改脚本:python3 -c 的 open(…, mode='wb')」
#       「就地改脚本:喂给 python 的 heredoc：只赋过一次的名字」「就地改脚本:喂给 python 的 heredoc：Path(…) / '…' 的 write_text」「就地改脚本:喂给 python 的 heredoc：pathlib.Path(…).write_bytes」
#       「就地改脚本:喂给 python 的 heredoc：Path(…).open('w')」「就地改脚本:喂给 python 的 heredoc：os.path.join 与 io.open」「就地改脚本:喂给 python 的 heredoc：shutil.copyfile」
#       「就地改脚本:喂给 python 的 heredoc：shutil.copy 进已有目录」「就地改脚本:cd 之后喂给 python 的相对路径」「就地改脚本:bash -c 里的 python3 -c」「就地改脚本:一百三十七：rsync --inplace 盖已有脚本（正推腿乙 Z8；rsync-inplace-unseen 下红）」（history-kept: 自证用例标签原文，按原文匹配）
#       「就地改脚本:拒绝时报出写法与相对仓库根的路径」「就地改脚本:工作根下的脚本」「就地改脚本:会话临时目录根下的脚本」「就地改脚本:会话 scratchpad 里的脚本」「就地改脚本:⑤ 现在认 dd、truncate 覆盖未跟踪产物（与 ⑦ 共用同一段 overwrite_steps，不抄第二份）」「就地改脚本:rm 掉再写未跟踪产物，⑤ 照拒（rm 没留住旧字节）」
#       「stdin:cp 覆盖未跟踪的产物拒绝（退出码 2）」「stdin:拒绝时 stderr 写了出路（按日期另存、先 git add 或挪到带日期的旧名）」「stdin:run_in_background 里的 > 覆盖同样拒绝（退出码 2）」「stdin:目标算不出、对得上未跟踪产物的放行（退出码 0）并记一条检出」
#       「stdin:cp 覆盖已有脚本拒绝（退出码 2）」「stdin:拒绝时 stderr 写了出路（临时文件再 mv 换上、replace-once.py / insert-row.py 定点改）」「stdin:run_in_background 里喂给 python 的 open(…, 'w') 同样拒绝（退出码 2）」「通删工作根:工作根下通配（~ 写法）」
#       「通删工作根:工作根下通配（$HOME 写法、引号里外）」「通删工作根:工作根本身」「通删工作根:工作根本身带结尾的 /」「通删工作根:会话临时目录根下通配」「通删工作根:rm -fr」「通删工作根:rm -r -f 分开写」「通删工作根:rm --recursive --force」「通删工作根:rm -r 不带 -f」「通删工作根:工作根下第一层部分通配」
#       「通删工作根:工作根下第一层通配再往下」「通删工作根:cd 进工作根之后 rm -rf *」「通删工作根:D=… 之后 cd $D 进工作根再 rm -rf *」「通删工作根:cd $HOME/… 进工作根再 rm -r 第一层部分通配」「通删工作根:bash -c 里」「通删工作根:nice、sudo 前缀」「通删工作根:工作根的上级」
#       「通删工作根:$SINGLEFS_WORK_ROOT 写法」「通删工作根:/tmp/* 盖住会话临时目录根」「通删工作根:前面 export 过的变量代入」「通删工作根:选项写在操作数后面」「通删工作根:喂给 bash 的 heredoc 里」「stdin:rm -rf 工作根下通配拒绝（退出码 2）」
#       「stdin:通删工作根的拒绝写了出路（work-directory-ownership.sh 的 sweep 与 claim）」「stdin:子 agent 用 run_in_background 起命令拒绝（退出码 2）」「stdin:子 agent 的 run_in_background 拒绝时 stderr 写了不起、写进报告「要主 agent 跑的命令」一节、结束本轮交回的出路，不教后台等」「stdin:子 agent 后台起 task-server/task wait 7 照拒（退出码 2）」「stdin:子 agent 后台起 cd <仓根> && task-server/task wait 7 照拒（退出码 2）」「stdin:子 agent 后台起 task-server/task wait 7 && echo x 照拒（退出码 2）」「stdin:子 agent 后台起 sleep 5 照拒（退出码 2）」「stdin:子 agent 后台起 cargo build 照拒（退出码 2）」「stdin:子 agent 后台起 task-server/task wait 后面不是号照拒（退出码 2）」「stdin:子 agent 后台起 task-server/task status 照拒（退出码 2）」
#       「stdin:experiment-runner cp 进主工作区的入库装置拒绝（退出码 2）」「stdin:先编后换的拒绝点名那条脚本与出路」
#       「stdin:子 agent 起 claude 拒绝（退出码 2）」「stdin:子 agent 经 "$CLAUDE_CODE_EXECPATH" 起 claude 拒绝（退出码 2）」
#       「stdin:子 agent 起以 /claude 结尾的可执行文件拒绝（退出码 2）」「stdin:子 agent 起 claude 被拒时 stderr 点名「派发」一节、写交回主 agent 的出路」
#       「stdin:子 agent case 一项的命令里的 reboot 照拒（退出码 2）」「stdin:clerk 在 Bash 里用 replace-once.py 写知识库拒绝（退出码 2）」「stdin:写范围的拒绝点名写法、目标、写范围表与出路」「stdin:范围内的链接指到仓里范围外，经它写拒绝（退出码 2）、拒绝说明写出解开符号链接之后的落点」
#       「stdin:目标里有命令替换的拒绝（退出码 2）、不记检出」「stdin:算不出落点的拒绝说明点名算不出、copy-originals.py 与写死的词的 for 循环两条出路」「stdin:喂给 bash 的 heredoc 里同一句 sed -i 知识库拒绝（退出码 2）」
#       「stdin:管道喂给 python 的 heredoc 里 open(知识库, 'w') 拒绝（退出码 2，写范围拒）」「stdin:管道喂给 python 的 heredoc 里 open(知识库, 'w') 同样被 ⑧ 拒（有 Edit 的子 agent 就地改仓内文件）」
#       「stdin:cd 到仓内 records 之后 sed -i 相对路径被 ⑧ 拒绝（退出码 2）」「stdin:hook 输入没有 cwd 时同一句 sed -i 相对路径拒绝（退出码 2）」「等待循环:同一条走 run_in_background 记一条检出」「stdin:检出落进检出记录」「stdin:前台没超时的等待循环拒绝（退出码 2）」
#       「stdin:拒绝时 stderr 写了两种写法（run_in_background 与 proc.py wait --timeout）」「stdin:拒了的命令里有判不了的输入（grep 的输出经管道喂给 bash），照记一条、结局被拒」「一百三十七：子 agent here-string 喂给 bash 的 claude 照拒（退出码 2）」
#       「一百三十七：子 agent env -S 拆串里的 claude 照拒（退出码 2）」「一百三十七：子 agent xargs 起的 claude 照拒（退出码 2）」「一百三十七：子 agent ssh 起的 claude 照拒（退出码 2）」「一百三十七：两个 & 只等其中一个照拒（退出码 2）」「一百三十七：coproc 照拒（退出码 2）」
#       「一百三十七：重定向目标花括号照拒，拒绝说明点名 ambiguous redirect 与改法」「一百三十七：带计数上限的前台等待循环照拒，拒绝说明点名真会结束的几种也要套 timeout」「stdin:run_in_background 里 & disown 拒绝（退出码 2）」
#       「stdin:放出追踪的拒绝 stderr 写了 run_in_background 的起法」「stdin:进程替换里前台没超时的等待循环拒绝（退出码 2）」「stdin:run_in_background 里后面没有 wait 的单独 & 拒绝（退出码 2）」
#       「stdin:拒绝时 stderr 写了两条出路（run_in_background 里不加 & 与 proc.py wait <pid> --timeout <秒>）」「stdin:kill 负进程号拒绝（退出码 2）」「stdin:拒绝时 stderr 写了按 pid 停一个的出路」「stdin:kill 写死的 1 拒绝（退出码 2，读真的进程表）」
#       「stdin:kill 写死的自检进程号（hook 的祖先）拒绝（退出码 2）」「stdin:run_in_background 里循环 kill 拒绝（退出码 2）」「stdin:一条命令同时踩几类时一次列全（终止进程、放出追踪、前台等待循环）」（共 486 格）
#       「起 claude:一百三十七：子 agent BASH_ENV 指到的脚本里起 claude」「起 claude:一百三十七：子 agent env BASH_ENV=… 起 bash，脚本里起 claude」
#       「起 claude:一百三十七：子 agent 前面 export BASH_ENV 再起 bash，脚本里起 claude」「起 claude:一百三十七：子 agent ENV 指到的脚本里起 claude，sh 起的」
#       「起 claude:一百三十七：子 agent BASH_ENV 写相对路径，按命令开头的当前目录解」「起 claude:一百三十七：子 agent 直接执行 #!/bin/bash 的脚本，BASH_ENV 指到的脚本里起 claude」
#       「起 claude:一百三十七：子 agent 直接执行 #!/bin/sh 的脚本，ENV 指到的脚本里起 claude」「起 claude:一百三十七：子 agent declare -x BASH_ENV 之后起 bash，脚本里起 claude」
#       「起 claude:一百三十七：子 agent typeset -x BASH_ENV 之后起 bash，脚本里起 claude」「起 claude:一百三十七：子 agent 先赋值后 export BASH_ENV，再起 bash，脚本里起 claude」
#       「起 claude:一百三十七：子 agent bash -i 读 HOME 下的 .bashrc 里起 claude」「起 claude:一百三十七：子 agent bash --rcfile 文件 -i，文件里起 claude」
#       「起 claude:一百三十七：子 agent sh -i 读 ENV 指到的脚本里起 claude」「起 claude:一百三十七：子 agent zsh 读 ZDOTDIR 下的 .zshenv 里起 claude」
#       「起 claude:d137 已知没认：登录 bash -l 读 HOME 下的 .bash_profile 里起 claude」「起 claude:d137 已知没认：登录 exec -l bash 照登录 shell 读 .bash_profile 里起 claude」
#       「起 claude:d137 已知没认：登录 sh -l 读 HOME 下的 .profile 里起 claude」「起 claude:d137 已知没认：系统级 bash -l 读 /etc/profile（样本目录）里起 claude」
#       「起 claude:d137 已知没认：系统级 bash -i 读 /etc/bash.bashrc（样本目录）里起 claude」「起 claude:d137 H9：python -c 里 os.system 起 claude」「起 claude:d137 H9：perl -e 里 system 起 claude」
#       「起 claude:d137 H9：gawk 的 system() 起 claude」「起 claude:d137 H9：sed 的 e 命令起 claude」「起 claude:d137 H9：git -c 别名 ! 起 claude」
#       「起 claude:d137 H9：tclsh 读 here-string 里 exec 起 claude」「起 claude:d137 H9：make 读 Makefile 配方起 claude」「起 claude:d137 H9：awk -f 程序文件里 system() 起 claude」
#       「起 claude:d137 已知没认：bash x.sh 起的脚本正文里起 claude」「起 claude:d137 已知没认：source 读进来的脚本正文里起 claude」「起 claude:d137 已知没认：直接执行的 #!/usr/bin/env bash 脚本正文里起 claude」
#       「起 claude:d137 已知没认：同一条命令里 heredoc 写出再 bash 起，正文里起 claude」
#       「stdin:先编后换的出路经 task-server 提交、不写 capped.sh 直接起」
#       「Bash写范围:写范围里没有 records/ 的 kb-scribe 起 ledger-append-section.py 写现行账本，拒」「Bash写范围:kb-scribe 直接执行 ledger-append-section.py 写现行账本，拒」
#       「Bash写范围:kb-scribe 的 ledger-append-section.py --root 写成没赋过值的变量，落点算不出，拒」
#       「Bash写范围:写范围里没有 records/ 的 kb-scribe 起 append-record.py 往记录追加，拒」「Bash写范围:kb-scribe 直接执行 append-record.py 往记录追加，拒」
#       「Bash写范围:kb-scribe cd 进 records 之后按相对路径起 append-record.py，拒」
#       「Bash写范围:kb-scribe 的 append-record.py 先写 --root 再写记录，落点照第一个位置参数认，拒」
#       「Bash写范围:kb-scribe 的 append-record.py 记录写成没赋过值的变量，落点算不出，拒」
#       「终止进程:不是 setsid 起的后台任务的进程组 -"$leader"」「终止进程:setsid 起的后台任务的进程号又被改写过 -"$leader"」
#       「终止进程:python -c 对没开新会话的子进程的进程组 os.killpg」「终止进程:python -c 子进程的名字又被改绑过的 os.killpg」
#   BASH_COMMAND_DETECTOR_BREAK=disable-self-background  findings_for() 不检出 run_in_background 里又自己放后台的那一种：实测红的格（照失败行原文）
#       「后台起的 nohup … & 加 disown」「后台起的命令末尾单独一个 &」「后台起的 & 在 wait 之后」「bg-notify-r1 X01」「bg-notify-r1 X02」「bg-notify-r1 X03」「bg-notify-r1 X04」「bg-notify-r1 X05」「bg-notify-r1 X06」
#       「bg-notify-r1 X07」「bg-notify-r1 X08」「bg-notify-r1 X09」「bg-notify-r1 X10」「bg-notify-r1 X11」「bg-notify-r1 X12」「setsid -f 放后台」（共 16 格）
#   BASH_COMMAND_DETECTOR_BREAK=keep-heredoc-bodies  findings_for() 不剥 heredoc 正文，写进文件的正文里的等待循环也检出
#       「写进笔记的 heredoc 正文里是一段没超时的等待循环」（共 1 格）
#   BASH_COMMAND_DETECTOR_BREAK=allow-watchdog-misuse  ① 起看门狗的错误写法也放行：实测红的格（照失败行原文）
#       「看门狗:前台起、输出丢进 /dev/null 还带 &」「看门狗:run_in_background 起却又带 &」「看门狗:前台起、什么都没加」「看门狗:前台起 agent-watch.py watch」「看门狗:run_in_background 起却挂 nohup」「看门狗:run_in_background 起却挂 setsid」
#       「看门狗:run_in_background 起却跟 disown」「看门狗:run_in_background 起却把输出丢进 /dev/null」「看门狗:前台起、写在 bash -c 里」「看门狗:前台起、包在 capped.sh 里」「stdin:前台起看门狗拒绝（退出码 2）」
#       「stdin:拒绝时 stderr 写了出路（run_in_background: true 的起法）」「stdin:here-string 里前台起看门狗照拒（退出码 2）」「一百三十七：看门狗输出经 tee 丢进 /dev/null 照拒（退出码 2）」
#       「看门狗:run_in_background 起、前一段 grep 2>/dev/null、watch.sh 自己也丢进 /dev/null 照拒」「看门狗:run_in_background 起、bash -c 整条丢进 /dev/null 照拒」
#       「看门狗:run_in_background 起、花括号整组丢进 /dev/null 照拒」「看门狗:run_in_background 起、管道后段把标准输出丢进 /dev/null 照拒」（共 18 格）
#   BASH_COMMAND_DETECTOR_BREAK=allow-subagent-background  ⑩ 子 agent 的 run_in_background 也放行
#       「stdin:子 agent 用 run_in_background 起命令拒绝（退出码 2）」「stdin:子 agent 的 run_in_background 拒绝时 stderr 写了不起、写进报告「要主 agent 跑的命令」一节、结束本轮交回的出路，不教后台等」
#       「stdin:子 agent 后台起 task-server/task wait 7 照拒（退出码 2）」「stdin:子 agent 后台起 cd <仓根> && task-server/task wait 7 照拒（退出码 2）」
#       「stdin:子 agent 后台起 task-server/task wait 7 && echo x 照拒（退出码 2）」「stdin:子 agent 后台起 sleep 5 照拒（退出码 2）」「stdin:子 agent 后台起 cargo build 照拒（退出码 2）」「stdin:子 agent 后台起 task-server/task wait 后面不是号照拒（退出码 2）」「stdin:子 agent 后台起 task-server/task status 照拒（退出码 2）」（共 9 格）
#   BASH_COMMAND_DETECTOR_BREAK=allow-foreground-wait-loop  ② 等待循环一律放行（前台没超时的、run_in_background 里轮询 tasks/*.output 的都放）：实测红的格（照失败行原文）
#       「等待循环:前台 until … sleep 没超时」「等待循环:run_in_background 里轮询后台任务的输出文件」「等待循环:run_in_background 里轮询后台任务输出、外面套了 timeout 也拒」「等待循环:前台按 pid 轮询的 while kill -0」「等待循环:前台 timeout 只在循环条件里」
#       「等待循环:前台多行写的 while true」「等待循环:前台 sleep 在循环条件里」「等待循环:前台喂给 bash 的 heredoc 里的循环」「等待循环:前台 bash -c 里没有 timeout」「等待循环:前台 capped.sh 包着的 bash -c 里没有 timeout」
#       「等待循环:前台 run-with-memory-cap.sh 包着的 bash -c 里没有 timeout」「等待循环:前台命令替换里的循环」「等待循环:前台 if 里的循环」「等待循环:前台循环体里套 for、sleep 在 for 之后」「等待循环:前台 time -p 包着的 bash -c 里没有 timeout」
#       「等待循环:前台 $(bash <<EOF) 里的循环」「等待循环:前台 <(bash <<EOF) 里的循环」「等待循环:前台 cd "$(…)" 之后 bash <<EOF 里的循环」「等待循环:前台 echo "$(date)" | bash <<EOF 里的循环」「等待循环:前台 $'a\'b' 之后 $(bash <<EOF) 里的循环」
#       「两道包装:包着的前台没超时的等待循环拒」「两道包装:--quiet 包着的前台没超时的等待循环拒」「stdin:前台没超时的等待循环拒绝（退出码 2）」「stdin:拒绝时 stderr 写了两种写法（run_in_background 与 proc.py wait --timeout）」「stdin:前台拒了的与套上 timeout 的都不记检出」
#       「stdin:拒了的命令里有判不了的输入（grep 的输出经管道喂给 bash），照记一条、结局被拒」「一百三十七：带计数上限的前台等待循环照拒，拒绝说明点名真会结束的几种也要套 timeout」「stdin:进程替换里前台没超时的等待循环拒绝（退出码 2）」「stdin:一条命令同时踩几类时一次列全（终止进程、放出追踪、前台等待循环）」（共 29 格）
#   BASH_COMMAND_DETECTOR_BREAK=allow-detaching  ③ 把活放出追踪的写法也放行：实测红的格（照失败行原文）
#       「两道包装:包着的 setsid -f 拒」「放出追踪:现场：& disown 之后又 & wait」「放出追踪:nohup … &」「放出追踪:nice 包着的 nohup … &」「放出追踪:coproc」「放出追踪:setsid -f」「放出追踪:setsid --fork」「放出追踪:tmux new-session -d」
#       「放出追踪:screen -dmS」「放出追踪:systemd-run 不等结束」「放出追踪:bash -c 里的 & disown」「放出追踪:命令替换里的 nohup … &」「放出追踪:capped.sh 包着的 setsid -f」「放出追踪:喂给 bash 的 heredoc 里的 disown」
#       「一百三十七：coproc 照拒（退出码 2）」「stdin:run_in_background 里 & disown 拒绝（退出码 2）」「stdin:放出追踪的拒绝 stderr 写了 run_in_background 的起法」
#       「stdin:一条命令同时踩几类时一次列全（终止进程、放出追踪、前台等待循环）」（共 18 格）
#   BASH_COMMAND_DETECTOR_BREAK=allow-unwaited-ampersand  ④ run_in_background 里后面没有 wait 的单独 & 也放行：实测红的格（照失败行原文）
#       「单独的 &:结尾单独一个 &」「单独的 &:& 之后接 echo $!」「单独的 &:& 在 capped.sh 前缀的那条命令上」「单独的 &:& 在 capped.sh 包着的 bash -c 里」「单独的 &:& 在 wait 之后」「单独的 &:圆括号子 shell 里的 &、wait 在外面」「单独的 &:& 之后的 wait 在另一对圆括号里」
#       「单独的 &:命令替换里的 &、wait 在外面」「单独的 &:bash -c 里的 &、wait 在外面」「单独的 &:喂给 bash 的 heredoc 正文里的 &」「单独的 &:喂给 python 的 heredoc 挂在 & 前面」「单独的 &:& 之后只 wait -n」「单独的 &:整个循环放后台」「单独的 &:一对圆括号整组放后台」
#       「单独的 &:几个 & 只 wait 一个 pid：剩下的照拒」「一百三十七：两个 & 只等其中一个照拒（退出码 2）」「stdin:run_in_background 里后面没有 wait 的单独 & 拒绝（退出码 2）」
#       「stdin:拒绝时 stderr 写了两条出路（run_in_background 里不加 & 与 proc.py wait <pid> --timeout <秒>）」「stdin:拒了的与前台的单独 & 都不记检出」（共 19 格）
#   BASH_COMMAND_DETECTOR_BREAK=allow-background-task-output-wait  ② run_in_background 里轮询 tasks/*.output 的等待循环也放行
#       「等待循环:run_in_background 里轮询后台任务的输出文件」「等待循环:run_in_background 里轮询后台任务输出、外面套了 timeout 也拒」（共 2 格）
#   BASH_COMMAND_DETECTOR_BREAK=allow-repository-in-place-edit  ⑧ 有 Edit 的子 agent 在 Bash 里就地改仓内文件也放行：实测红的格（照失败行原文）
#       「就地改仓内文件:执行员 sed -i 改仓内源文件」「就地改仓内文件:执行员 python open(仓内路径, 'w')」「就地改仓内文件:执行员 python Path(仓内绝对路径).write_text」「就地改仓内文件:执行员 sed -i 改工作根同名前缀的兄弟目录」
#       「就地改仓内文件:执行员 python os.replace 把草稿挪进 scripts/ 下的入库装置」「就地改仓内文件:执行员 python os.rename 挪进知识库（源是变量）」「就地改仓内文件:执行员 python shutil.move 挪进仓根起的绝对路径」
#       「就地改仓内文件:执行员 python shutil.move 用 dst= 挪进 scripts/」「就地改仓内文件:执行员 python open(scripts/ 下的文件, 'w')」「就地改仓内文件:cd 到仓根之后 python 相对路径写 crates/ 照拒」
#       「就地改仓内文件:cd 到算不出的目录之后 python 相对路径写 crates/ 照旧按仓内判」「就地改仓内文件:cd 到草稿目录里的仓副本之后 python 写仓根起的绝对路径照拒」「就地改仓内文件:喂给 python 的 heredoc 里 open(仓内路径, 'w') 照拒」
#       「就地改仓内文件:喂给 bash 的 heredoc 里的 sed -i 照拒」「就地改仓内文件:bash -c 里的 python3 -c 写仓内文件照拒」「就地改仓内文件:cd 到仓内目录之后 sed -i 相对路径照拒」「就地改仓内文件:不 cd 的仓内相对路径 sed -i 照拒」
#       「就地改仓内文件:cd 到算不出的目录之后 sed -i 相对路径照旧按仓内判」「就地改仓内文件:先 cd 到草稿目录、再 cd 回仓内之后 sed -i 相对路径照拒」「就地改仓内文件:cd 到草稿目录之后 sed -i 的相对路径用 .. 跳出工作根照拒」「就地改仓内文件:cd 到草稿目录之后 sed -i 的目标带通配，照旧按仓内判」
#       「就地改仓内文件:管道喂给 python3 - 的 heredoc 里 open(仓内路径, 'w') 照拒」「就地改仓内文件:管道喂给 python3（不带 -）的 heredoc 里 open(仓内路径, 'w') 照拒」「就地改仓内文件:管道喂给 python3 - 的 heredoc 里 Path(仓内路径).write_text 照拒」
#       「就地改仓内文件:echo 的文字管道喂给 python3 - 照拒」「就地改仓内文件:管道喂给 bash 的 heredoc 里 sed -i 仓内文件照拒」「就地改仓内文件:命令开头的当前目录在仓内，sed -i 相对路径照拒」
#       「stdin:管道喂给 python 的 heredoc 里 open(知识库, 'w') 同样被 ⑧ 拒（有 Edit 的子 agent 就地改仓内文件）」「stdin:cd 到仓内 records 之后 sed -i 相对路径被 ⑧ 拒绝（退出码 2）」（共 29 格）
#   BASH_COMMAND_DETECTOR_BREAK=allow-compile-first-bypass  ⑨ experiment-runner 在 Bash 里写主工作区 crates/ 下的 .rs 也放行：实测红的格（照失败行原文）
#       「先编后换:执行员 cp 草稿副本盖主工作区的入库装置」「先编后换:执行员 cat > 新建入库装置」「先编后换:执行员 >> 追加」「先编后换:执行员 tee -a」「先编后换:执行员 mv 草稿副本进 bin 目录」「先编后换:执行员 cd 进 bin 目录之后 cp 到 .」「先编后换:执行员 sed -i」「先编后换:执行员 perl -pi -e」
#       「先编后换:执行员 rsync」「先编后换:执行员 rm 入库装置」「先编后换:执行员 mv 把入库装置挪走」「先编后换:执行员 bash -c 里 cp」「先编后换:执行员变量里的目录」「先编后换:执行员目标算不出、对得上已有的入库装置」「先编后换:执行员 python shutil.copy」「先编后换:执行员 python open(…, 'a')」
#       「先编后换:执行员 python os.replace」「先编后换:执行员 cwd 在主工作区的 crates/ 里、相对路径写 .rs 拒」「两道包装:包着的 cp 盖主工作区入库装置拒（先编后换）」「stdin:experiment-runner cp 进主工作区的入库装置拒绝（退出码 2）」「stdin:先编后换的拒绝点名那条脚本与出路」（共 21 格）
#       「stdin:先编后换的出路经 task-server 提交、不写 capped.sh 直接起」
#   BASH_COMMAND_DETECTOR_BREAK=allow-results-overwrite  ⑤ 整份覆盖 research/results/ 下未跟踪产物也放行、也不记检出：实测红的格（照失败行原文）
#       「覆盖产物:> 覆盖未跟踪的产物」「覆盖产物:tee 不带 -a」「覆盖产物:cp 的目标（现场原句的形态）」「覆盖产物:bash -c 里的 >」「覆盖产物:>|」「覆盖产物:&>」「覆盖产物:2> 同样截断」「覆盖产物:>& 后面跟文件名」「覆盖产物:tee -i 不是追加」「覆盖产物:tee 写两份、其中一份是未跟踪的」「覆盖产物:mv 的目标」
#       「覆盖产物:install 的目标」「覆盖产物:cp 进已有目录、源的文件名撞上」「覆盖产物:cp -t 目录」「覆盖产物:cp 源里的通配按这一刻展开」「覆盖产物:cd 进去之后写相对路径」「覆盖产物:cd 进子目录之后写」「覆盖产物:绝对路径」「覆盖产物:喂给 bash 的 heredoc 正文里的 >」「覆盖产物:命令替换里的 >」
#       「覆盖产物:cat 把 heredoc 写进未跟踪的产物」「覆盖产物:花括号整组重定向」「覆盖产物:圆括号整组重定向」「覆盖产物:timeout、capped.sh 包着的 bash -c 整条重定向」「覆盖产物:前面赋过值的变量代入」「覆盖产物:export 过的变量代入」「覆盖产物:cd 到变量路径算不出来、按仓库根解析」「覆盖产物:进程替换里的 tee」
#       「覆盖产物:整个循环重定向」「覆盖产物:sudo 包着的 tee」「覆盖产物:git add 的是另一份，这一份照拒」「覆盖产物:dd of= 覆盖未跟踪的产物」「覆盖产物:dd conv=notrunc 覆盖未跟踪的产物」「覆盖产物:truncate -s 0 覆盖未跟踪的产物」「覆盖产物:truncate 选项连写覆盖未跟踪产物」
#       「覆盖产物:一条命令里两份未跟踪的目标各自点名」「覆盖产物:一条命令里两份未跟踪的目标同时判，git ls-files 只起一次」「覆盖产物:已跟踪的、未跟踪的、不存在的目标混在一起同时判，只点名未跟踪的」「覆盖产物:已跟踪的、未跟踪的、不存在的目标混在一起，git ls-files 只起一次」「覆盖产物检出:算不出的变量对得上一份未跟踪的产物」
#       「覆盖产物检出:变量在前、research/results/ 在后」「覆盖产物检出:git 判不了（不是 git 仓）只记不拦」「就地改脚本:⑤ 现在认 dd、truncate 覆盖未跟踪产物（与 ⑦ 共用同一段 overwrite_steps，不抄第二份）」「就地改脚本:rm 掉再写未跟踪产物，⑤ 照拒（rm 没留住旧字节）」
#       「stdin:cp 覆盖未跟踪的产物拒绝（退出码 2）」「stdin:拒绝时 stderr 写了出路（按日期另存、先 git add 或挪到带日期的旧名）」「stdin:run_in_background 里的 > 覆盖同样拒绝（退出码 2）」「stdin:目标算不出、对得上未跟踪产物的放行（退出码 0）并记一条检出」（共 48 格）
#   BASH_COMMAND_DETECTOR_BREAK=allow-process-signals  ⑥ 终止进程的写法也放行：实测红的格（照失败行原文）
#       「两道包装:包着的 kill 负进程号拒」「终止进程:kill -- 负进程号（进程组）」「终止进程:kill -TERM 负进程号」「终止进程:kill -s TERM 负进程号」「终止进程:自己的任务的进程组 -"$job"」「终止进程:kill 0」「终止进程:kill -TERM 0」「终止进程:kill -9 -1（全部）」
#       「终止进程:kill $PPID（Claude 会话本身）」「终止进程:kill $(pgrep …)」「终止进程:kill $(jobs -p)」「终止进程:kill `pidof sshd`」「终止进程:kill 数组整批展开」「终止进程:kill "$@"」「终止进程:kill 一次两个」「终止进程:kill %1 %2」「终止进程:kill 带通配」
#       「终止进程:for 循环里 kill」「终止进程:while read cgroup.procs 里 kill（现场那一段）」「终止进程:多行循环里 kill」「终止进程:cgroup.procs 交给 xargs kill」「终止进程:xargs -n1 kill」「终止进程:pgrep 交给 xargs -r kill -9」
#       「终止进程:find -exec kill」「终止进程:pkill 按名字」「终止进程:pkill -x claude」「终止进程:killall」「终止进程:fuser -k」「终止进程:先从 cgroup.procs 挑一个再 kill」「终止进程:先 pgrep 挑一个再 kill」
#       「终止进程:python -c 推导式里读 cgroup.procs 再 os.kill」「终止进程:喂给 python 的 heredoc 里 os.killpg」「终止进程:python -c os.kill(0, …)」「终止进程:systemctl stop ssh」「终止进程:sudo systemctl restart sshd」
#       「终止进程:systemctl kill session-1.scope」「终止进程:systemctl stop user@1000.service」「终止进程:systemctl set-property user.slice」「终止进程:systemctl stop 私有配置登记的本地模型服务（不写后缀）」
#       「终止进程:systemctl restart 私有配置登记的第二个单元」「终止进程:systemctl disable --now ssh」「终止进程:systemctl --user stop 带通配 singlefs-*」「终止进程:systemctl --user kill 带通配 *.scope」
#       「终止进程:systemctl --user stop 命令替换列出来的一批」「终止进程:systemctl isolate」「终止进程:systemctl --user exit」「终止进程:systemctl reboot」「终止进程:loginctl terminate-session」「终止进程:loginctl kill-user」
#       「终止进程:往 cgroup.kill 写」「终止进程:tee 进 cgroup.freeze」「终止进程:sudo reboot」「终止进程:shutdown -h now」「终止进程:写死的进程号是 sshd」「终止进程:写死的进程号是 VSCode 服务端」「终止进程:写死的进程号是这个会话自己的 Claude」
#       「终止进程:写死的进程号是另一个 Claude 会话」「终止进程:写死的进程号是另一个 Claude 会话起的 cargo」「终止进程:写死的进程号是私有配置登记的本地模型服务」「终止进程:kill -HUP 1」「终止进程:proc.py stop 写死的 sshd」「终止进程:proc.py stop 一次两个」
#       「终止进程:循环里 proc.py stop」「终止进程:bash -c 里 kill 进程组」「终止进程:nice 包着 kill -9 -1」「终止进程:喂给 bash 的 heredoc 里 kill 0」「终止进程:timeout 包着 kill 负进程号」「终止进程:run-with-memory-cap.sh 包着 pkill」
#       「终止进程:shell 函数与变量里的脚本包着的 bash -c 里 kill 0」「终止进程:shell 函数包着的 python -c 里 os.killpg」「终止进程:kill 写死的进程号读一次进程表」「终止进程:proc.py stop 写死的进程号读一次进程表」「终止进程:一条命令里几处写死的进程号共读一次进程表」
#       「终止进程:私有配置文件不在:systemctl stop ssh 照拒」「终止进程:私有配置里没有那两个键:systemctl stop ssh 照拒」「stdin:子 agent case 一项的命令里的 reboot 照拒（退出码 2）」「stdin:kill 负进程号拒绝（退出码 2）」「stdin:拒绝时 stderr 写了按 pid 停一个的出路」
#       「stdin:kill 写死的 1 拒绝（退出码 2，读真的进程表）」「stdin:kill 写死的自检进程号（hook 的祖先）拒绝（退出码 2）」「stdin:run_in_background 里循环 kill 拒绝（退出码 2）」「stdin:一条命令同时踩几类时一次列全（终止进程、放出追踪、前台等待循环）」（共 82 格）
#       「终止进程:不是 setsid 起的后台任务的进程组 -"$leader"」「终止进程:setsid 起的后台任务的进程号又被改写过 -"$leader"」
#       「终止进程:python -c 对没开新会话的子进程的进程组 os.killpg」「终止进程:python -c 子进程的名字又被改绑过的 os.killpg」
#   BASH_COMMAND_DETECTOR_BREAK=ignore-private-config  ⑥ 不读私有配置，本地模型服务的单元与进程也放行
#       「终止进程:systemctl stop 私有配置登记的本地模型服务（不写后缀）」「终止进程:systemctl restart 私有配置登记的第二个单元」「终止进程:写死的进程号是私有配置登记的本地模型服务」（共 3 格）
#   BASH_COMMAND_DETECTOR_BREAK=allow-script-in-place-write  ⑦ 在同一个 inode 上改已有脚本也放行：实测红的格（照失败行原文）
#       「就地改脚本:> 覆盖已有的 .sh」「就地改脚本:>|」「就地改脚本:&>」「就地改脚本:2> 同样截断」「就地改脚本:>& 后面跟文件名」「就地改脚本:cat > 旧 <<EOF」「就地改脚本:tee 不带 -a」「就地改脚本:tee -i 不是追加」「就地改脚本:cp 新 旧」「就地改脚本:cp -f」「就地改脚本:cp -a」
#       「就地改脚本:cp 进已有目录、文件名撞上」「就地改脚本:cp -t 目录」「就地改脚本:dd of=」「就地改脚本:dd conv=notrunc」「就地改脚本:truncate -s 0」「就地改脚本:truncate -cs 0（选项连写）」「就地改脚本:写已有的 .py」「就地改脚本:带执行位、没有后缀的」「就地改脚本:经符号链接写它指向的脚本」
#       「就地改脚本:scratch 根下的脚本」「就地改脚本:cd 进去之后写相对路径」「就地改脚本:bash -c 里的 >」「就地改脚本:喂给 bash 的 heredoc 正文里的 cp」「就地改脚本:命令替换里的 >」「就地改脚本:前面赋过值的变量代入」「就地改脚本:nice、timeout 包着的 cp」
#       「就地改脚本:python3 -c 的 open(…, 'w')」「就地改脚本:python3 -c 的 open(…, 'r+')」「就地改脚本:python3 -c 的 open(…, mode='wb')」「就地改脚本:喂给 python 的 heredoc：只赋过一次的名字」
#       「就地改脚本:喂给 python 的 heredoc：Path(…) / '…' 的 write_text」「就地改脚本:喂给 python 的 heredoc：pathlib.Path(…).write_bytes」「就地改脚本:喂给 python 的 heredoc：Path(…).open('w')」
#       「就地改脚本:喂给 python 的 heredoc：os.path.join 与 io.open」「就地改脚本:喂给 python 的 heredoc：shutil.copyfile」「就地改脚本:喂给 python 的 heredoc：shutil.copy 进已有目录」「就地改脚本:cd 之后喂给 python 的相对路径」
#       「就地改脚本:bash -c 里的 python3 -c」「就地改脚本:一百三十七：rsync --inplace 盖已有脚本（正推腿乙 Z8；rsync-inplace-unseen 下红）」「就地改脚本:拒绝时报出写法与相对仓库根的路径」「就地改脚本:工作根下的脚本」「就地改脚本:会话临时目录根下的脚本」「就地改脚本:会话 scratchpad 里的脚本」「stdin:cp 覆盖已有脚本拒绝（退出码 2）」（history-kept: 自证用例标签原文，按原文匹配）
#       「stdin:拒绝时 stderr 写了出路（临时文件再 mv 换上、replace-once.py / insert-row.py 定点改）」「stdin:run_in_background 里喂给 python 的 open(…, 'w') 同样拒绝（退出码 2）」（共 47 格）
#   BASH_COMMAND_DETECTOR_BREAK=keep-selftest-scratch  自证的临时目录走回 mkdtemp 不删（lib_selftest_scratch.py 的 keep）
#       「通过那条路上临时目录没删」「red 那条路上临时目录没删」「crash 那条路上临时目录没删」（共 3 格）
#   BASH_COMMAND_DETECTOR_BREAK=refuse-every-wait-loop  ② 外层 timeout 与 run_in_background 都不算，等待循环一律拒：实测红的格（照失败行原文）
#       「等待循环:同一条套上 timeout 60」「等待循环:同一条走 run_in_background」「等待循环:timeout 60 bash 的 heredoc 里的循环」「等待循环:capped.sh 与 nice 包着的 timeout」「等待循环:$(timeout 60 bash <<EOF) 里的循环」
#       「两道包装:包着的 timeout 等待循环放行」「stdin:run_in_background 里没超时的等待循环只记不拦（退出码 0）」「stdin:检出落进检出记录」「stdin:套上 timeout 的前台等待循环放行（退出码 0）」「一百三十七：一个 & 之后 kill -0 轮询等它放行（退出码 0）」（共 10 格）
#   BASH_COMMAND_DETECTOR_BREAK=stop-at-first-refusal  main() 一条命令踩了几类拒绝时只报第一类（退回逐类试的旧样子）
#       「stdin:一条命令同时踩几类时一次列全（终止进程、放出追踪、前台等待循环）」「stdin:管道喂给 python 的 heredoc 里 open(知识库, 'w') 拒绝（退出码 2，写范围拒）」（共 2 格）
#   BASH_COMMAND_DETECTOR_BREAK=detection-outcome-dropped  检出记录不写结局（outcome：只记）：实测红的格（照失败行原文）
#       「stdin:拒了的命令里有判不了的输入（grep 的输出经管道喂给 bash），照记一条、结局被拒」「stdin:只记的检出记录写了结局「只记」」（共 2 格）
#   BASH_COMMAND_DETECTOR_BREAK=failure-unrecorded  判不了的几样（hook 输入不是 JSON 对象、哪一道抛异常、库读不到、喂给 shell 的不是字面）只打 stderr、不记检出
#       「stdin:坏 JSON 放行（退出码 0）、stderr 写诊断、记一条检出」「stdin:不是 JSON 对象放行（退出码 0）、stderr 写诊断、记一条检出」
#       「stdin:判的时候抛异常放行（退出码 0）、stderr 写诊断、记一条检出」「stdin:here-string 喂给 bash 的是变量放行（退出码 0）、记一条检出」
#       「stdin:拒了的命令里有判不了的输入（grep 的输出经管道喂给 bash），照记一条、结局被拒」（共 5 格）
#   BASH_COMMAND_DETECTOR_BREAK=allow-work-root-removal  ⑪ rm -r 作用在工作根本身、它的上级或工作根下第一层通配上也放行
#       「通删工作根:工作根下通配（~ 写法）」「通删工作根:工作根下通配（$HOME 写法、引号里外）」「通删工作根:工作根本身」「通删工作根:工作根本身带结尾的 /」「通删工作根:会话临时目录根下通配」「通删工作根:rm -fr」「通删工作根:rm -r -f 分开写」「通删工作根:rm --recursive --force」「通删工作根:rm -r 不带 -f」「通删工作根:工作根下第一层部分通配」「通删工作根:工作根下第一层通配再往下」「通删工作根:cd 进工作根之后 rm -rf *」「通删工作根:bash -c 里」「通删工作根:nice、sudo 前缀」「通删工作根:工作根的上级」「通删工作根:$SINGLEFS_WORK_ROOT 写法」「通删工作根:/tmp/* 盖住会话临时目录根」「通删工作根:前面 export 过的变量代入」「通删工作根:选项写在操作数后面」「通删工作根:喂给 bash 的 heredoc 里」「通删工作根:D=… 之后 cd $D 进工作根再 rm -rf *」「通删工作根:cd $HOME/… 进工作根再 rm -r 第一层部分通配」「stdin:rm -rf 工作根下通配拒绝（退出码 2）」「stdin:通删工作根的拒绝写了出路（work-directory-ownership.sh 的 sweep 与 claim）」（共 24 格）
#   （这两格在别的开关下也红：disable-check 下两格都红；allow-detaching、allow-foreground-wait-loop、allow-process-signals 下「一次列全」那一格红）
#   BASH_COMMAND_DETECTOR_BREAK=always-read-process-table  ⑥ 命令里有数字就先整扫 /proc 读进程表（正常只在判到 kill / proc.py stop 的写死的进程号时才读）
#       「终止进程:没有终止词的带数字命令不读进程表（git log -1、sleep 2、--threads 8）」（共 1 格）
#   BASH_COMMAND_DETECTOR_BREAK=own-session-group-unrecognized  ⑥ 不认自己用新会话起的子进程的进程组（OwnSessionGroups.is_own_group 一律交假、process_target_findings 不认 `-$名字`）
#       「终止进程:setsid 起的后台任务的进程组 -"$leader"」「终止进程:python -c 对 start_new_session=True 起的子进程的进程组 os.killpg 与 os.kill(-组号)」
#       「扫脚本:进程组:自己用 setsid 或 start_new_session=True 起的子进程的组不报」（共 3 格，照判法推的、还没复跑）
#   BASH_COMMAND_DETECTOR_BREAK=own-session-group-session-unchecked  ⑥ 认子进程时不看起法：Popen 这一类不看 start_new_session / process_group，shell 里放后台的不看是不是 setsid、带没带 -f
#       「终止进程:不是 setsid 起的后台任务的进程组 -"$leader"」「终止进程:python -c 对没开新会话的子进程的进程组 os.killpg」
#       「扫脚本:进程组:没用 setsid 起的后台任务的组」「扫脚本:进程组:setsid --fork 起的」「扫脚本:进程组:python 没开新会话的子进程」
#       「扫脚本:自己任务的进程组报在第 5 行」「扫脚本:红的都登记进待改清单就退出码 0」「扫脚本:成功行报查了几个、放行了哪一处、逐个列出没判的」
#       「扫脚本:待改清单里的行所在的目录这一趟没扫：退出码 0、报本次未判」「扫脚本:待改清单里的行所在的目录这一趟也扫了：照常列进没判的、不报本次未判」
#       （不以「扫脚本:进程组:」起头的那五格出自扫脚本那一组的 group.sh：set -m 之后放后台、不是 setsid 起的，这个开关下它不再报红，待改清单里它那一行成了「一处要放行的都没有」；共 10 格，照判法推的、还没复跑）
#   BASH_COMMAND_DETECTOR_BREAK=own-session-group-first-binding-only  ⑥ 认子进程与 `名字=$!` 时只看第一处绑定，后面的改绑看不见
#       「终止进程:setsid 起的后台任务的进程号又被改写过 -"$leader"」「终止进程:python -c 子进程的名字又被改绑过的 os.killpg」
#       「扫脚本:进程组:setsid 起的进程号又被改写过」「扫脚本:进程组:python 子进程的名字又被改绑过」（共 4 格，照判法推的、还没复跑）
#   BASH_COMMAND_DETECTOR_BREAK=per-path-git-query  ⑤ 确定的目标一份起一次 git ls-files（正常收齐一次查完）
#       「覆盖产物:一条命令里两份未跟踪的目标同时判，git ls-files 只起一次」「覆盖产物:已跟踪的、未跟踪的、不存在的目标混在一起，git ls-files 只起一次」（共 2 格）
#   BASH_COMMAND_DETECTOR_BREAK=opaque-two-logs  wrapped_commands() 不剥 scripts/hosts/two-logs.sh：它包着的命令照旧看不见
#       「两道包装:包着的前台没超时的等待循环拒」「两道包装:--quiet 包着的前台没超时的等待循环拒」「两道包装:包着的 kill 负进程号拒」「两道包装:包着的 setsid -f 拒」「两道包装:包着的 cp 盖主工作区入库装置拒（先编后换）」「Bash写范围:clerk 经 two-logs.sh 包着写知识库」（共 6 格）
#   （同样的格在 LIB_PROJECT_CODE_RUNS_BREAK=two-logs-opaque 下也红：lib_project_code_runs.py 的 command_under_two_logs 一律交 None，剥法只有那一份）
#   BASH_COMMAND_DETECTOR_BREAK=allow-bash-scope-write  ⑫ 有写范围的项目子 agent 在 Bash 里写写范围之外的仓内文件也放行：实测红的格（照失败行原文）
#       「Bash写范围:clerk 用 replace-once.py 写知识库（现场：越出写范围）」「Bash写范围:clerk 直接执行 replace-once.py 写知识库」「Bash写范围:clerk 先 cd 到 records 再用相对路径写知识库」「Bash写范围:kb-scribe 用 replace-once.py 写 records 拒」
#       「Bash写范围:clerk 用 insert-row.py 写知识库」「Bash写范围:clerk 用 replace-batch.py，规格点名知识库」「Bash写范围:clerk 脚本路径在变量里，用 replace-once.py 写知识库」「Bash写范围:clerk 脚本路径在变量里，用 insert-row.py 写知识库」
#       「Bash写范围:clerk 脚本路径在变量里，用 replace-batch.py、规格点名知识库」「Bash写范围:clerk 直接执行变量里的 replace-once.py 写知识库」「Bash写范围:clerk sed -i 改 crates」「Bash写范围:clerk 重定向 > 写 crates」
#       「Bash写范围:clerk 重定向 >> 写知识库」「Bash写范围:clerk tee 写 crates」「Bash写范围:clerk tee -a 写知识库」「Bash写范围:clerk cp 到知识库」「Bash写范围:clerk cp 进知识库目录」「Bash写范围:clerk mv 到 crates」
#       「Bash写范围:clerk heredoc 写知识库」「Bash写范围:clerk 先 cd 进知识库目录再重定向」「Bash写范围:clerk bash -c 里的写」「Bash写范围:clerk 变量里的目录（前面赋过值）」「Bash写范围:clerk python open(…, 'w') 写知识库」
#       「Bash写范围:clerk 喂给 bash 的 heredoc 里同样一句 sed -i 知识库照拒」「Bash写范围:clerk 喂给 bash 的 heredoc 里的重定向写知识库照拒」「Bash写范围:clerk 喂给 python 的 heredoc 里 open(知识库, 'w') 照拒」
#       「Bash写范围:clerk cat > 知识库 <<EOF 的目标本身照拒（正文是数据，目标不是）」「Bash写范围:clerk tee 知识库 <<EOF 的目标本身照拒」「Bash写范围:clerk 经 two-logs.sh 包着写知识库」「Bash写范围:clerk 的目标在没赋过值的变量里：拒」「Bash写范围:clerk 重定向目标带通配：拒」
#       「Bash写范围:clerk cd 到没赋过值的变量目录之后写相对路径：拒」「Bash写范围:clerk cd 到没赋过值的变量目录之后写绝对路径的知识库：照拒」「Bash写范围:clerk 管道喂给 python3 - 的 heredoc 里 open(知识库, 'w') 照拒」
#       「Bash写范围:clerk 管道喂给 python3（不带 -）的 heredoc 里 open(知识库, 'w') 照拒」「Bash写范围:clerk cd 到 records 之后管道喂给 python3 - 的 heredoc 里相对路径写知识库照拒」「Bash写范围:clerk 管道喂给 bash 的 heredoc 里 sed -i 知识库照拒」
#       「Bash写范围:clerk 管道喂给 bash -s 的 heredoc 里重定向写知识库照拒」「Bash写范围:clerk echo 的文字管道喂给 python3 - 写知识库照拒」「Bash写范围:clerk printf 的文字管道喂给 python3 - 写知识库照拒」
#       「Bash写范围:clerk 自己的 heredoc 盖过管道：echo | python3 - <<EOF 里写知识库照拒」「Bash写范围:clerk cd 到知识库目录之后写 $PWD/… 照拒」「Bash写范围:clerk cd 到知识库目录之后 cp 进 $PWD/… 照拒」「Bash写范围:没有项目定义的内置 agent 写仓里拒」
#       「Bash写范围:没有项目定义的内置 agent 写仓外别处拒」「Bash写范围:没有项目定义的内置 agent 写会话临时目录根拒（scratchpad 之外）」「Bash写范围:没有项目定义的内置 agent 写会话 scratchpad 拒」「Bash写范围:没有项目定义的内置 agent 用 replace-once.py 写仓里拒」「Bash写范围:定义已删的旧实例写仓里拒」「Bash写范围:一百三十七：clerk eval 字面代码里 sed -i 知识库，拒」
#       「Bash写范围:一百三十七：clerk find -exec sed -i 读进来的路径，落点算不出，拒」「Bash写范围:一百三十七：clerk echo 喂给 xargs sed -i，落点算不出，拒」「Bash写范围:一百三十七：clerk xargs sed -i 写范围里的 records 也拒（落点算不出）」
#       「Bash写范围:一百三十七：clerk find -exec cp 到写死的知识库路径，拒」「Bash写范围:一百三十七：钩子进程在工作根下，clerk sed -i 没赋过值的变量照拒」「Bash写范围:一百三十七：钩子进程在工作根下，clerk xargs sed -i 照拒」
#       「Bash写范围:python3 起没赋过值的变量里的脚本、第一个参数落在知识库，拒」「Bash写范围:python3 起没赋过值的变量里的脚本、第一个参数落在写范围里的 records 也拒」「Bash写范围:python3 起命令替换给的脚本、第一个参数是仓根起的绝对路径，拒」「Bash写范围:目标在没赋过值的变量里，拒」
#       「Bash写范围:重定向目标带通配，拒」「Bash写范围:replace-batch.py 规格读不到，拒」「Bash写范围:cd 到没赋过值的变量目录之后写相对路径，拒」「Bash写范围:cd 算不出之后 python 写相对路径，拒」
#       「Bash写范围:检出原文：mkdir -p <草稿目录>/originals && for f in …; do cp -p "$f" "<草稿目录>/originals/$(echo $f | tr / _)"; done，拒」
#       「Bash写范围:检出原文：S=<草稿目录> && mkdir -p $S/originals && for f in …; do cp -- "$f" "$S/originals/$(echo $f | tr / _)"; done，拒」「Bash写范围:for 的列表里有爬回仓里的 ..，落在写范围外，拒」
#       「Bash写范围:for 的列表里有 .. 段、落点回到写范围之外，拒」「Bash写范围:for 的列表里有写范围外的仓内路径，cp 写到它，拒」「Bash写范围:f 来源不明的 > <草稿目录>/out-$f，拒」「Bash写范围:变量来源不明、静态前缀在工作根下（仓外），拒」「Bash写范围:变量来源不明、静态前缀在 /tmp 下（仓外），拒」
#       「Bash写范围:变量来源不明、静态前缀已在写范围里，拒」「Bash写范围:命令替换在最后一段、静态前缀已在写范围里，拒」「Bash写范围:cp 进范围内目录、源名算不出，拒」「Bash写范围:cd 到写死的范围内目录之后写带来源不明变量的相对路径，拒」「Bash写范围:cd 到写死的范围内目录之后写变量开头的相对路径，拒」
#       「Bash写范围:for 循环的列表里有算不出的词，拒」「Bash写范围:通配目标在范围内目录下，拒」「Bash写范围:静态前缀在仓内、写范围之外，拒」「Bash写范围:目标以变量开头，拒」「Bash写范围:变量单独占一段，拒」「Bash写范围:变量之后有字面的 .. 段，拒」「Bash写范围:仓外目录下变量单独占一段，拒」
#       「Bash写范围:cd 到写死的范围外目录之后写带变量的相对路径，拒」「Bash写范围:cd 到算不出的目录之后写带变量的相对路径，拒」「Bash写范围:cd 进循环变量拼的目录之后写相对路径，拒」「Bash写范围:目标明确、写范围之外的拒」「Bash写范围:仓外的符号链接指到仓内知识库，<链接>/../x 回到仓里，拒」
#       「Bash写范围:范围内的链接指到仓里范围外的目录，经它 > 写，拒」「Bash写范围:范围内的链接指到仓里范围外的目录，经它 cp 写，拒」「Bash写范围:范围内的链接指到仓里范围外的目录，经它 replace-once.py 写，拒」「Bash写范围:范围内的链接指到仓里范围外的目录，cp 进链接（目标是目录），拒」
#       「Bash写范围:没有 cd 的 $PWD，拒」「Bash写范围:cd 到算不出的目录之后 $PWD，拒」「Bash写范围:先 cd 到写死的目录、再 cd 到算不出的目录，$PWD 作废，拒」「Bash写范围:cd 到写死的草稿目录之后 $PWD/子目录/$p，$p 来源不明，拒」「Bash写范围:mktemp -d -p 指的目录算不出，拒」
#       「Bash写范围:mktemp 不带 -d 建的是文件，拒」「Bash写范围:mktemp -d 的模板在当前目录里，拒」「Bash写范围:while read 的循环变量，拒」「Bash写范围:for 的列表来自变量，拒」「Bash写范围:前面赋过写死值的变量又被 for $列表 改写，拒」「Bash写范围:派发回执对不上那把锁（别的子 agent）：拒」
#       「Bash写范围:模板在别的子 agent 认领的草稿目录里，拒」「Bash写范围:模板目录的归属文件是别的会话，拒」「Bash写范围:模板在 /tmp 下（不是草稿目录），拒」「Bash写范围:hook 输入里没有 agent_id，模板在草稿目录里也拒」「Bash写范围:-p 指到别的子 agent 认领的草稿目录，拒」
#       「Bash写范围:--tmpdir= 指到别的会话的草稿目录，拒」「Bash写范围:-p 指到 /tmp（不是草稿目录），拒」「Bash写范围:-p 指到仓里的目录，拒」「Bash写范围:hook 输入里没有 agent_id，-p 指到草稿目录也拒」「Bash写范围:套一层、-p 指到别的子 agent 认领的草稿目录，拒」
#       「Bash写范围:套一层、cd 之后接的不是 pwd，拒」「stdin:clerk 在 Bash 里用 replace-once.py 写知识库拒绝（退出码 2）」「stdin:写范围的拒绝点名写法、目标、写范围表与出路」「stdin:范围内的链接指到仓里范围外，经它写拒绝（退出码 2）、拒绝说明写出解开符号链接之后的落点」
#       「stdin:目标里有命令替换的拒绝（退出码 2）、不记检出」「stdin:算不出落点的拒绝说明点名算不出、copy-originals.py 与写死的词的 for 循环两条出路」「stdin:管道喂给 python 的 heredoc 里 open(知识库, 'w') 拒绝（退出码 2，写范围拒）」
#       「一百三十七：重定向目标花括号照拒，拒绝说明点名 ambiguous redirect 与改法」
#       「Bash写范围:写范围里没有 records/ 的 kb-scribe 起 ledger-append-section.py 写现行账本，拒」「Bash写范围:kb-scribe 直接执行 ledger-append-section.py 写现行账本，拒」
#       「Bash写范围:kb-scribe 的 ledger-append-section.py --root 写成没赋过值的变量，落点算不出，拒」
#       「Bash写范围:写范围里没有 records/ 的 kb-scribe 起 append-record.py 往记录追加，拒」「Bash写范围:kb-scribe 直接执行 append-record.py 往记录追加，拒」
#       「Bash写范围:kb-scribe cd 进 records 之后按相对路径起 append-record.py，拒」
#       「Bash写范围:kb-scribe 的 append-record.py 先写 --root 再写记录，落点照第一个位置参数认，拒」
#       「Bash写范围:kb-scribe 的 append-record.py 记录写成没赋过值的变量，落点算不出，拒」（共 130 格）
#   BASH_COMMAND_DETECTOR_BREAK=scope-known-scripts-unseen  ⑫ 不认 replace-once.py、insert-row.py、replace-batch.py 的目标文件参数（只认重定向、tee、cp / mv、sed -i、python 的写法）：实测红的格（照失败行原文）
#       「Bash写范围:clerk 用 replace-once.py 写知识库（现场：越出写范围）」「Bash写范围:clerk 直接执行 replace-once.py 写知识库」「Bash写范围:clerk 先 cd 到 records 再用相对路径写知识库」「Bash写范围:kb-scribe 用 replace-once.py 写 records 拒」
#       「Bash写范围:clerk 用 insert-row.py 写知识库」「Bash写范围:clerk 用 replace-batch.py，规格点名知识库」「Bash写范围:clerk 脚本路径在变量里，用 replace-once.py 写知识库」「Bash写范围:clerk 脚本路径在变量里，用 insert-row.py 写知识库」
#       「Bash写范围:clerk 脚本路径在变量里，用 replace-batch.py、规格点名知识库」「Bash写范围:clerk 直接执行变量里的 replace-once.py 写知识库」「Bash写范围:clerk 的目标在没赋过值的变量里：拒」「Bash写范围:没有项目定义的内置 agent 用 replace-once.py 写仓里拒」
#       「Bash写范围:python3 起没赋过值的变量里的脚本、第一个参数落在知识库，拒」「Bash写范围:python3 起没赋过值的变量里的脚本、第一个参数落在写范围里的 records 也拒」「Bash写范围:python3 起命令替换给的脚本、第一个参数是仓根起的绝对路径，拒」「Bash写范围:目标在没赋过值的变量里，拒」
#       「Bash写范围:replace-batch.py 规格读不到，拒」「Bash写范围:范围内的链接指到仓里范围外的目录，经它 replace-once.py 写，拒」「stdin:clerk 在 Bash 里用 replace-once.py 写知识库拒绝（退出码 2）」「stdin:写范围的拒绝点名写法、目标、写范围表与出路」
#       「Bash写范围:写范围里没有 records/ 的 kb-scribe 起 ledger-append-section.py 写现行账本，拒」「Bash写范围:kb-scribe 直接执行 ledger-append-section.py 写现行账本，拒」
#       「Bash写范围:kb-scribe 的 ledger-append-section.py --root 写成没赋过值的变量，落点算不出，拒」
#       「Bash写范围:写范围里没有 records/ 的 kb-scribe 起 append-record.py 往记录追加，拒」「Bash写范围:kb-scribe 直接执行 append-record.py 往记录追加，拒」
#       「Bash写范围:kb-scribe cd 进 records 之后按相对路径起 append-record.py，拒」
#       「Bash写范围:kb-scribe 的 append-record.py 先写 --root 再写记录，落点照第一个位置参数认，拒」
#       「Bash写范围:kb-scribe 的 append-record.py 记录写成没赋过值的变量，落点算不出，拒」（ledger-append-section.py、append-record.py 一并不认；共 28 格）
#   BASH_COMMAND_DETECTOR_BREAK=scope-ledger-append-unseen  ⑫ 不认 ledger-append-section.py 要写的现行账本（它不带目标文件参数，⑫ 看不出它写哪一份）
#       「Bash写范围:写范围里没有 records/ 的 kb-scribe 起 ledger-append-section.py 写现行账本，拒」「Bash写范围:kb-scribe 直接执行 ledger-append-section.py 写现行账本，拒」
#       「Bash写范围:kb-scribe 的 ledger-append-section.py --root 写成没赋过值的变量，落点算不出，拒」（共 3 格）
#   BASH_COMMAND_DETECTOR_BREAK=scope-record-append-unseen  ⑫ 不认 append-record.py 要写的记录（它不在 KNOWN_WRITE_SCRIPTS 里，写范围里没有 records/ 的子 agent 起它往 records/ 写看不见）：实测红的格（照失败行原文）
#       「Bash写范围:写范围里没有 records/ 的 kb-scribe 起 append-record.py 往记录追加，拒」「Bash写范围:kb-scribe 直接执行 append-record.py 往记录追加，拒」
#       「Bash写范围:kb-scribe cd 进 records 之后按相对路径起 append-record.py，拒」
#       「Bash写范围:kb-scribe 的 append-record.py 先写 --root 再写记录，落点照第一个位置参数认，拒」
#       「Bash写范围:kb-scribe 的 append-record.py 记录写成没赋过值的变量，落点算不出，拒」（共 5 格）
#   BASH_COMMAND_DETECTOR_BREAK=scope-record-append-root-as-target  ⑫ 认 append-record.py 的记录时不跳过 --root 的值（--root 写在记录前面时把仓根当记录，落在仓外不判）：打开后红的格（照失败行原文）
#       「Bash写范围:kb-scribe 的 append-record.py 先写 --root 再写记录，落点照第一个位置参数认，拒」（共 1 格）
#   BASH_COMMAND_DETECTOR_BREAK=scope-main-agent-judged  ⑫ 把主 agent 也当成一条写范围模式都没有的子 agent 判（主 agent 写本仓的任何文件都拒）：实测红的格（照失败行原文）
#       「Bash写范围:主 agent 写知识库放行」「Bash写范围:主 agent 的目标算不出也不判」「stdin:拒了的不记检出」「stdin:>> 追加放行（退出码 0）、不记检出」「stdin:就地改脚本拒了的不记检出」「stdin:主 agent 同一条写知识库的命令放行（退出码 0）」
#       「stdin:写到临时文件再 mv 换上放行（退出码 0）、不记检出」「stdin:前台的同一条放行（退出码 0）」「Bash写范围:主 agent 起 ledger-append-section.py 放行」「stdin:Bash heredoc 往仓里追加干净内容放行（退出码 0）」「Bash写范围:主 agent 起 append-record.py 放行」（共 11 格，实测，任务 6176）
#   BASH_COMMAND_DETECTOR_BREAK=scope-undeterminable-only-noted  ⑫ 落点这一刻算不出的（命令替换、没赋过值或来源不明的变量、while read 与 for $列表 的循环变量、通配、cd 算不出之后的相对路径、读不到规格的 replace-batch.py）只记检出、不拒：实测红的格（照失败行原
#       文）
#       「Bash写范围:clerk 的目标在没赋过值的变量里：拒」「Bash写范围:clerk 重定向目标带通配：拒」「Bash写范围:clerk cd 到没赋过值的变量目录之后写相对路径：拒」「Bash写范围:一百三十七：clerk find -exec sed -i 读进来的路径，落点算不出，拒」
#       「Bash写范围:一百三十七：clerk echo 喂给 xargs sed -i，落点算不出，拒」「Bash写范围:一百三十七：clerk xargs sed -i 写范围里的 records 也拒（落点算不出）」「Bash写范围:一百三十七：钩子进程在工作根下，clerk sed -i 没赋过值的变量照拒」
#       「Bash写范围:一百三十七：钩子进程在工作根下，clerk xargs sed -i 照拒」「Bash写范围:python3 起没赋过值的变量里的脚本、第一个参数落在知识库，拒」「Bash写范围:python3 起没赋过值的变量里的脚本、第一个参数落在写范围里的 records 也拒」
#       「Bash写范围:python3 起命令替换给的脚本、第一个参数是仓根起的绝对路径，拒」「Bash写范围:目标在没赋过值的变量里，拒」「Bash写范围:重定向目标带通配，拒」「Bash写范围:replace-batch.py 规格读不到，拒」「Bash写范围:cd 到没赋过值的变量目录之后写相对路径，拒」
#       「Bash写范围:cd 算不出之后 python 写相对路径，拒」「Bash写范围:检出原文：mkdir -p <草稿目录>/originals && for f in …; do cp -p "$f" "<草稿目录>/originals/$(echo $f | tr / _)"; done，拒」
#       「Bash写范围:检出原文：S=<草稿目录> && mkdir -p $S/originals && for f in …; do cp -- "$f" "$S/originals/$(echo $f | tr / _)"; done，拒」「Bash写范围:f 来源不明的 > <草稿目录>/out-$f，拒」
#       「Bash写范围:变量来源不明、静态前缀在工作根下（仓外），拒」「Bash写范围:变量来源不明、静态前缀在 /tmp 下（仓外），拒」「Bash写范围:变量来源不明、静态前缀已在写范围里，拒」「Bash写范围:命令替换在最后一段、静态前缀已在写范围里，拒」「Bash写范围:cp 进范围内目录、源名算不出，拒」
#       「Bash写范围:cd 到写死的范围内目录之后写带来源不明变量的相对路径，拒」「Bash写范围:cd 到写死的范围内目录之后写变量开头的相对路径，拒」「Bash写范围:for 循环的列表里有算不出的词，拒」「Bash写范围:通配目标在范围内目录下，拒」「Bash写范围:静态前缀在仓内、写范围之外，拒」「Bash写范围:目标以变量开头，拒」
#       「Bash写范围:变量单独占一段，拒」「Bash写范围:变量之后有字面的 .. 段，拒」「Bash写范围:仓外目录下变量单独占一段，拒」「Bash写范围:cd 到写死的范围外目录之后写带变量的相对路径，拒」「Bash写范围:cd 到算不出的目录之后写带变量的相对路径，拒」「Bash写范围:cd 进循环变量拼的目录之后写相对路径，拒」
#       「Bash写范围:没有 cd 的 $PWD，拒」「Bash写范围:cd 到算不出的目录之后 $PWD，拒」「Bash写范围:先 cd 到写死的目录、再 cd 到算不出的目录，$PWD 作废，拒」「Bash写范围:cd 到写死的草稿目录之后 $PWD/子目录/$p，$p 来源不明，拒」「Bash写范围:mktemp -d -p 指的目录算不出，拒」
#       「Bash写范围:mktemp 不带 -d 建的是文件，拒」「Bash写范围:mktemp -d 的模板在当前目录里，拒」「Bash写范围:while read 的循环变量，拒」「Bash写范围:for 的列表来自变量，拒」「Bash写范围:前面赋过写死值的变量又被 for $列表 改写，拒」「Bash写范围:派发回执对不上那把锁（别的子 agent）：拒」
#       「Bash写范围:模板在别的子 agent 认领的草稿目录里，拒」「Bash写范围:模板目录的归属文件是别的会话，拒」「Bash写范围:模板在 /tmp 下（不是草稿目录），拒」「Bash写范围:hook 输入里没有 agent_id，模板在草稿目录里也拒」「Bash写范围:-p 指到别的子 agent 认领的草稿目录，拒」
#       「Bash写范围:--tmpdir= 指到别的会话的草稿目录，拒」「Bash写范围:-p 指到 /tmp（不是草稿目录），拒」「Bash写范围:-p 指到仓里的目录，拒」「Bash写范围:hook 输入里没有 agent_id，-p 指到草稿目录也拒」「Bash写范围:套一层、-p 指到别的子 agent 认领的草稿目录，拒」
#       「Bash写范围:套一层、cd 之后接的不是 pwd，拒」「stdin:目标里有命令替换的拒绝（退出码 2）、不记检出」「stdin:算不出落点的拒绝说明点名算不出、copy-originals.py 与写死的词的 for 循环两条出路」「一百三十七：重定向目标花括号照拒，拒绝说明点名 ambiguous redirect 与改法」
#       「Bash写范围:kb-scribe 的 ledger-append-section.py --root 写成没赋过值的变量，落点算不出，拒」「Bash写范围:kb-scribe 的 append-record.py 记录写成没赋过值的变量，落点算不出，拒」（共 63 格）
#   BASH_COMMAND_DETECTOR_BREAK=scope-loop-values-unknown  ⑫ bind_loop_variables() 把 `for 名字 in 写死的词` 的循环变量也当算不出（不逐个取值）
#       「Bash写范围:for b in 写死的词; do … > <草稿目录>/breaks/$b.out，放行」「Bash写范围:cd 到写死的范围内目录之后 for 循环（列表是写死的名字）里写 $d/…，放行」「Bash写范围:for f in 写死的仓内路径; do cp $f <草稿目录>/; done，放行」「Bash写范围:S=<草稿目录> 之后 for f in 写死的词 写 $S/$f.out，放行」「stdin:算不出落点的拒绝说明点名算不出、copy-originals.py 与写死的词的 for 循环两条出路」「stdin:写死的词的循环变量拼的目标放行（退出码 0）、不记检出」（共 6 格）
#   BASH_COMMAND_DETECTOR_BREAK=scope-loop-landings-trusted  ⑫ loop_value_choices() 每个循环取值只当一个普通名字（取值里的 `..` 与 / 看不见，爬回仓里的放过去）
#       「Bash写范围:for 的列表里有爬回仓里的 ..，落在写范围外，拒」「Bash写范围:for 的列表里有 .. 段、落点回到写范围之外，拒」「stdin:算不出落点的拒绝说明点名算不出、copy-originals.py 与写死的词的 for 循环两条出路」（共 3 格）
#   BASH_COMMAND_DETECTOR_BREAK=scope-home-unexpanded  ⑫ home_expanded() 不展开没赋过值的 $HOME（当算不出）：实测红的格（照失败行原文）
#       「Bash写范围:python3 起 $HOME 下的脚本不当算不出，放行」「Bash写范围:$HOME 照 hook 的环境展开，落在仓外放行」（共 2 格）
#   BASH_COMMAND_DETECTOR_BREAK=scope-landing-lexical-only  ⑫ 落点不解开符号链接，只按字面规整 `..`（经 lib_write_scope.landings_out_of_scope 的 lexical_only 交进去；`<仓外的链接>/../x` 回到仓里看不见，范围内的链接指到仓里范围外、经它写进去也看不见）
#       「Bash写范围:仓外的符号链接指到仓内知识库，<链接>/../x 回到仓里，拒」「Bash写范围:范围内的链接指到仓里范围外的目录，经它 > 写，拒」「Bash写范围:范围内的链接指到仓里范围外的目录，经它 cp 写，拒」「Bash写范围:范围内的链接指到仓里范围外的目录，经它 replace-once.py 写，拒」「Bash写范围:范围内的链接指到仓里范围外的目录，cp 进链接（目标是目录），拒」「Bash写范围:范围外的链接指回范围内，经它写，放行」「Bash写范围:范围外的链接指到仓外，经它写的落点在仓外，不判」「stdin:范围内的链接指到仓里范围外，经它写拒绝（退出码 2）、拒绝说明写出解开符号链接之后的落点」（共 8 格）
#   BASH_COMMAND_DETECTOR_BREAK=heredoc-body-judged-as-commands  heredoc 正文不分喂给谁，一律当命令行与 python 代码判（⑧ 的 python 判法在整条命令文本上跑，写进文件的正文、引号里的文字也算；overwrite_steps 
#       不剥喂给别的命令的 heredoc 正文，⑤ ⑦ ⑨ ⑫ 一并受影响）：实测红的格（照失败行原文）
#       「就地改仓内文件:cd 到草稿目录里的仓副本之后 python 相对路径写 crates/ 放行」「就地改仓内文件:cd 到草稿目录里的仓副本之后 python 相对路径挪进 scripts/ 放行」「就地改仓内文件:cd 到草稿目录里的仓副本之后喂给 python 的 heredoc 相对路径写 scripts/ 放行」
#       「就地改仓内文件:往草稿目录 cat 写脚本，heredoc 正文里的 python open(仓内路径, 'w') 是数据」「就地改仓内文件:往草稿目录 cat 写笔记，heredoc 正文里的 python Path(仓内路径).write_text 是数据」「就地改仓内文件:引号里 echo 的 open(仓内路径, 'w') 文字是数据」
#       「就地改仓内文件:cat 写草稿脚本的 heredoc 正文里的 sed -i 是数据」「就地改仓内文件:管道喂给 grep 的 heredoc 正文是数据」「就地改仓内文件:echo 的文字管道喂给 grep 是数据」「就地改仓内文件:管道喂给 python3 脚本文件的 heredoc 正文是数据（程序从文件读）」
#       「Bash写范围:clerk 往草稿脚本 cat 写的 heredoc 正文里有 sed -i 知识库：正文是数据，放行」「Bash写范围:clerk 往草稿脚本 cat 写的 heredoc 正文里有重定向和 cp 到知识库：放行」「Bash写范围:clerk 往草稿脚本 tee 写的 heredoc 正文里有 sed -i 知识库：放行」
#       「Bash写范围:clerk 喂给 python 的 heredoc 里 open(知识库, 'w') 照拒」「Bash写范围:clerk 管道喂给 python3 - 的 heredoc 里 open(知识库, 'w') 照拒」「Bash写范围:clerk 管道喂给 python3（不带 -）的 heredoc 里 open(知识库, 'w') 照拒」
#       「Bash写范围:clerk cd 到 records 之后管道喂给 python3 - 的 heredoc 里相对路径写知识库照拒」「Bash写范围:clerk 自己的 heredoc 盖过管道：echo | python3 - <<EOF 里写知识库照拒」「Bash写范围:clerk 管道喂给 grep 的 heredoc 正文是数据，放行」
#       「覆盖产物:写进笔记的 heredoc 正文」「就地改脚本:喂给 python 的 heredoc：只赋过一次的名字」「就地改脚本:喂给 python 的 heredoc：Path(…) / '…' 的 write_text」「就地改脚本:喂给 python 的 heredoc：pathlib.Path(…).write_bytes」
#       「就地改脚本:喂给 python 的 heredoc：Path(…).open('w')」「就地改脚本:喂给 python 的 heredoc：os.path.join 与 io.open」「就地改脚本:喂给 python 的 heredoc：shutil.copyfile」
#       「就地改脚本:喂给 python 的 heredoc：shutil.copy 进已有目录」「就地改脚本:cd 之后喂给 python 的相对路径」「就地改脚本:写进笔记的 heredoc 正文」「stdin:run_in_background 里喂给 python 的 open(…, 'w') 同样拒绝（退出码 2）」
#       「stdin:cat 写草稿脚本的 heredoc 正文里有 sed -i 知识库放行（退出码 0）」「stdin:管道喂给 python 的 heredoc 里 open(知识库, 'w') 拒绝（退出码 2，写范围拒）」「stdin:管道喂给 grep 的 heredoc 正文放行（退出码 0）」
#       「往现行账本追加:喂给 python 的 heredoc 里 open(…, 'a') 拒」「往 records 追加:喂给 python 的 heredoc 里 open(…, mode='a') 拒」（共 35 格）
#   BASH_COMMAND_DETECTOR_BREAK=sed-in-place-ignores-cd  ⑧ 的 sed -i 相对路径不跟 cd、也不看 hook 输入的 cwd（cd 到草稿目录之后 sed -i 相对路径照样按仓内文件拒；也不认经管道喂给 bash 的 heredoc 里的 sed -i）
#       「就地改仓内文件:cd 到工作根下的草稿目录之后 sed -i 相对路径放行」「就地改仓内文件:cd 到 /tmp/ 下的目录之后 sed -i 相对路径放行」「就地改仓内文件:cd 到 // 开头写的 /tmp/ 下的目录之后 sed -i 相对路径放行」「就地改仓内文件:cd 到前面赋过值的草稿目录之后 sed -i 相对路径放行」「就地改仓内文件:管道喂给 bash 的 heredoc 里 sed -i 仓内文件照拒」「就地改仓内文件:命令开头的当前目录在草稿目录里，sed -i 相对路径放行」「stdin:cd 到草稿目录之后 sed -i 相对路径放行（退出码 0）」「stdin:hook 输入的 cwd 在 /tmp 下时 sed -i 相对路径放行（退出码 0）」（共 8 格；「// 开头写的」那一格是推的，没跑）
#   （其中「就地改仓内文件:管道喂给 bash 的 heredoc 里 sed -i 仓内文件照拒」在 heredoc-pipe-fed-as-data 下也红）
#   BASH_COMMAND_DETECTOR_BREAK=heredoc-pipe-fed-as-data  经管道喂给 python、bash 的 heredoc 正文与 echo / printf 的文字当数据（`cat <<'EOF' | python3 -` 的正文 ⑧ ⑫ 都不判）
#       「就地改仓内文件:管道喂给 python3 - 的 heredoc 里 open(仓内路径, 'w') 照拒」「就地改仓内文件:管道喂给 python3（不带 -）的 heredoc 里 open(仓内路径, 'w') 照拒」「就地改仓内文件:管道喂给 python3 - 的 heredoc 里 Path(仓内路径).write_text 照拒」「就地改仓内文件:echo 的文字管道喂给 python3 - 照拒」「就地改仓内文件:管道喂给 bash 的 heredoc 里 sed -i 仓内文件照拒」「Bash写范围:clerk 管道喂给 python3 - 的 heredoc 里 open(知识库, 'w') 照拒」「Bash写范围:clerk 管道喂给 python3（不带 -）的 heredoc 里 open(知识库, 'w') 照拒」「Bash写范围:clerk cd 到 records 之后管道喂给 python3 - 的 heredoc 里相对路径写知识库照拒」「Bash写范围:clerk 管道喂给 bash 的 heredoc 里 sed -i 知识库照拒」「Bash写范围:clerk 管道喂给 bash -s 的 heredoc 里重定向写知识库照拒」「Bash写范围:clerk echo 的文字管道喂给 python3 - 写知识库照拒」「Bash写范围:clerk printf 的文字管道喂给 python3 - 写知识库照拒」「stdin:管道喂给 python 的 heredoc 里 open(知识库, 'w') 拒绝（退出码 2，写范围拒）」「stdin:管道喂给 python 的 heredoc 里 open(知识库, 'w') 同样被 ⑧ 拒（有 Edit 的子 agent 就地改仓内文件）」（共 14 格）
#   BASH_COMMAND_DETECTOR_BREAK=pwd-variable-unexpanded  ⑫ 里 cd 到写死的目录之后 $PWD 不按那个目录展开（`cd <目录> && … > $PWD/x` 的目标以变量开头，只记检出）
#       「Bash写范围:clerk cd 到 records 之后写 $PWD/… 放行」「Bash写范围:cd 到写死的草稿目录之后写 $PWD/… 不记」「Bash写范围:cd 到写死的范围内目录之后写 $PWD/… 不记」「Bash写范围:cd 到写死的草稿目录之后 cp 进 $PWD/子目录/ 不记」「Bash写范围:cd 到写死的草稿目录之后 T=$PWD/x 再写 $T/… 不记」「stdin:cd 之后的 $PWD/… 与 mktemp -d 赋的变量放行（退出码 0）、不记检出」（共 6 格）
#   BASH_COMMAND_DETECTOR_BREAK=mktemp-directory-unknown  ⑫ 里 `d=$(mktemp -d)` 赋的变量算不出（`> $d/x` 的目标以变量开头，只记检出）
#       「Bash写范围:clerk mktemp -d 建的变量目录里写放行」「Bash写范围:clerk mktemp -d 之后 cd 进去再写相对路径放行」「Bash写范围:W=$(mktemp -d) 之后 cp 进 $W/ 不记」「Bash写范围:W=`mktemp -d`（反引号）之后写 $W/… 不记」「Bash写范围:export W=$(mktemp -d) 之后写 $W/… 不记」「Bash写范围:mktemp -d 之后 cd 进去写相对路径不记」「Bash写范围:mktemp -d -t 带模板不记」「Bash写范围:mktemp --directory 不记」「stdin:cd 之后的 $PWD/… 与 mktemp -d 赋的变量放行（退出码 0）、不记检出」「Bash写范围:不带值的 --tmpdir：W=$(mktemp -d --tmpdir) 之后写进 $W/ 不记」「Bash写范围:套一层、不带 -p：T=$(cd $(mktemp -d) && pwd -P) 之后写进 $T/ 不记」（共 11 格）
#   （「stdin:cd 之后的 $PWD/… 与 mktemp -d 赋的变量放行（退出码 0）、不记检出」在 pwd-variable-unexpanded 与 mktemp-directory-unknown 下都红）
#   BASH_COMMAND_DETECTOR_BREAK=mktemp-scratch-template-unknown  ⑫ 里不带 -t 的 mktemp 模板一律算不出，模板目录落在自己认领的草稿目录里也一样
#       「Bash写范围:模板在自己认领的草稿目录里：T=$(mktemp -d $W/名.XXXXXX) 之后 cp 进 $T/ 不记」「Bash写范围:模板在自己认领的草稿目录里：直接写进 $(mktemp -d …)/文件 不记」「Bash写范围:cd 进自己认领的草稿目录之后模板写相对路径不记」「Bash写范围:还没有认领文件、派发回执对得上那把锁：模板在它的草稿目录里不记」「stdin:模板在自己认领的草稿目录里的 mktemp -d 放行（退出码 0）、不记检出」（共 5 格）
#   BASH_COMMAND_DETECTOR_BREAK=mktemp-template-owner-unchecked  own_scratch_directory() 不看归属文件与写者锁，工作根下的目录一律当自己认领的
#       「Bash写范围:派发回执对不上那把锁（别的子 agent）：拒」「Bash写范围:模板在别的子 agent 认领的草稿目录里，拒」「Bash写范围:模板目录的归属文件是别的会话，拒」「Bash写范围:-p 指到别的子 agent 认领的草稿目录，拒」「Bash写范围:--tmpdir= 指到别的会话的草稿目录，拒」「Bash写范围:套一层、-p 指到别的子 agent 认领的草稿目录，拒」（共 6 格）
#   BASH_COMMAND_DETECTOR_BREAK=mktemp-option-directory-unknown  ⑫ 里 `mktemp -d -p <目录>`、`--tmpdir=<目录>` 与不带值的 `--tmpdir` 一概算不出
#       「Bash写范围:-p 指到自己认领的草稿目录：T=$(mktemp -d -p "$W") 之后写进 $T/ 不记」「Bash写范围:--tmpdir= 指到自己认领的草稿目录（带模板名）：cp 进 $T/ 不记」「Bash写范围:cd 进自己认领的草稿目录之后 -p . 不记」「Bash写范围:直接写进 $(mktemp -d -p …)/文件 不记」「Bash写范围:不带值的 --tmpdir：W=$(mktemp -d --tmpdir) 之后写进 $W/ 不记」「Bash写范围:套一层 cd 加 pwd -P：T=$(cd $(mktemp -d -p "$W") && pwd -P) 之后写进 $T/ 不记」「Bash写范围:套一层、反引号写法：T=`cd $(mktemp -d -p …) && pwd` 之后写进 $T/ 不记」「Bash写范围:套一层、直接写进 $(cd $(mktemp -d -p …) && pwd -P)/文件 不记」（共 8 格）
#   BASH_COMMAND_DETECTOR_BREAK=mktemp-nested-substitution-unknown  ⑫ 里 `$(cd $(mktemp …) && pwd -P)` 这种套在别的命令替换里的 mktemp 算不出
#       「Bash写范围:套一层 cd 加 pwd -P：T=$(cd $(mktemp -d -p "$W") && pwd -P) 之后写进 $T/ 不记」「Bash写范围:套一层、不带 -p：T=$(cd $(mktemp -d) && pwd -P) 之后写进 $T/ 不记」「Bash写范围:套一层、反引号写法：T=`cd $(mktemp -d -p …) && pwd` 之后写进 $T/ 不记」「Bash写范围:套一层、直接写进 $(cd $(mktemp -d -p …) && pwd -P)/文件 不记」（共 4 格）
#   BASH_COMMAND_DETECTOR_BREAK=break-honored-outside-selftest  平常跑设了弄坏开关也照认，不拒；它是关守卫项，只在单独一项时认，与别的项同写照拒
#       「平常跑设了弄坏开关：退 2 并点名只在自证里认」（共 1 格）
#   BASH_COMMAND_DETECTOR_BREAK=pending-outside-scanned-as-stale  --scan-scripts 把待改清单里落在这一趟没扫的目录下的行也判成「一处要放行的都没有」：实测红的格（照失败行原文）
#       「扫脚本:待改清单里的行所在的目录这一趟没扫：退出码 0、报本次未判」「扫脚本:待改清单里的行所在的目录这一趟没扫：往 GATE_NOT_RUN_FILE 报一行」（共 2 格）
#   BASH_COMMAND_DETECTOR_BREAK=allow-subagent-claude-launch  ⑬ 子 agent 在 Bash 里起 claude 会话也放行：实测红的格（照失败行原文）
#       「起 claude:子 agent 命令位置上的 claude -p」「起 claude:子 agent 经 "$CLAUDE_CODE_EXECPATH" 起」「起 claude:子 agent 经 ${CLAUDE_CODE_EXECPATH} 起」「起 claude:子 agent 起以 /claude 结尾的可执行文件」
#       「起 claude:子 agent nice 包着起 claude」「起 claude:子 agent bash -c 里起 claude」「起 claude:子 agent 命令替换里起 claude」「起 claude:子 agent 喂给 bash 的 heredoc 里起 claude」
#       「stdin:子 agent 起 claude 拒绝（退出码 2）」「stdin:子 agent 经 "$CLAUDE_CODE_EXECPATH" 起 claude 拒绝（退出码 2）」「stdin:子 agent 起以 /claude 结尾的可执行文件拒绝（退出码 2）」
#       「stdin:子 agent 起 claude 被拒时 stderr 点名「派发」一节、写交回主 agent 的出路」「一百三十七：子 agent here-string 喂给 bash 的 claude 照拒（退出码 2）」「一百三十七：子 agent env -S 拆串里的 claude 照拒（退出码 2）」
#       「一百三十七：子 agent xargs 起的 claude 照拒（退出码 2）」「一百三十七：子 agent ssh 起的 claude 照拒（退出码 2）」
#       「起 claude:一百三十七：子 agent BASH_ENV 指到的脚本里起 claude」「起 claude:一百三十七：子 agent env BASH_ENV=… 起 bash，脚本里起 claude」「起 claude:一百三十七：子 agent 前面 export BASH_ENV 再起 bash，脚本里起 claude」
#       「起 claude:一百三十七：子 agent ENV 指到的脚本里起 claude，sh 起的」「起 claude:一百三十七：子 agent BASH_ENV 写相对路径，按命令开头的当前目录解」「起 claude:一百三十七：子 agent 直接执行 #!/bin/bash 的脚本，BASH_ENV 指到的脚本里起 claude」「起 claude:一百三十七：子 agent 直接执行 #!/bin/sh 的脚本，ENV 指到的脚本里起 claude」
#       「起 claude:一百三十七：子 agent declare -x BASH_ENV 之后起 bash，脚本里起 claude」「起 claude:一百三十七：子 agent typeset -x BASH_ENV 之后起 bash，脚本里起 claude」「起 claude:一百三十七：子 agent 先赋值后 export BASH_ENV，再起 bash，脚本里起 claude」
#       「起 claude:一百三十七：子 agent bash -i 读 HOME 下的 .bashrc 里起 claude」「起 claude:一百三十七：子 agent bash --rcfile 文件 -i，文件里起 claude」「起 claude:一百三十七：子 agent sh -i 读 ENV 指到的脚本里起 claude」
#       「起 claude:一百三十七：子 agent zsh 读 ZDOTDIR 下的 .zshenv 里起 claude」（共 30 格）
#       「起 claude:d137 已知没认：登录 bash -l 读 HOME 下的 .bash_profile 里起 claude」「起 claude:d137 已知没认：登录 exec -l bash 照登录 shell 读 .bash_profile 里起 claude」
#       「起 claude:d137 已知没认：登录 sh -l 读 HOME 下的 .profile 里起 claude」「起 claude:d137 已知没认：系统级 bash -l 读 /etc/profile（样本目录）里起 claude」
#       「起 claude:d137 已知没认：系统级 bash -i 读 /etc/bash.bashrc（样本目录）里起 claude」「起 claude:d137 H9：python -c 里 os.system 起 claude」「起 claude:d137 H9：perl -e 里 system 起 claude」
#       「起 claude:d137 H9：gawk 的 system() 起 claude」「起 claude:d137 H9：sed 的 e 命令起 claude」「起 claude:d137 H9：git -c 别名 ! 起 claude」
#       「起 claude:d137 H9：tclsh 读 here-string 里 exec 起 claude」「起 claude:d137 H9：make 读 Makefile 配方起 claude」「起 claude:d137 H9：awk -f 程序文件里 system() 起 claude」
#       「起 claude:d137 已知没认：bash x.sh 起的脚本正文里起 claude」「起 claude:d137 已知没认：source 读进来的脚本正文里起 claude」「起 claude:d137 已知没认：直接执行的 #!/usr/bin/env bash 脚本正文里起 claude」
#       「起 claude:d137 已知没认：同一条命令里 heredoc 写出再 bash 起，正文里起 claude」
#   BASH_COMMAND_DETECTOR_BREAK=subagent-timeout-unnoted  检出 ③ 子 agent 前台 timeout 超过 240000 毫秒不记
#       「超时检出:子 agent 前台 timeout 600000 毫秒记」「超时检出:子 agent 前台 timeout 写成字符串 300000 也记」
#       「stdin:子 agent 前台 timeout 600000 毫秒放行（退出码 0）、记一条检出」（共 3 格）
#   BASH_COMMAND_DETECTOR_BREAK=python-move-into-repository-unseen  ⑧ 的 python 一半不认 os.replace / os.rename / shutil.move 的目标，前缀不含 scripts/
#       「就地改仓内文件:执行员 python os.replace 把草稿挪进 scripts/ 下的入库装置」「就地改仓内文件:执行员 python os.rename 挪进知识库（源是变量）」
#       「就地改仓内文件:执行员 python shutil.move 挪进仓根起的绝对路径」「就地改仓内文件:执行员 python shutil.move 用 dst= 挪进 scripts/」
#       「就地改仓内文件:执行员 python open(scripts/ 下的文件, 'w')」（共 5 格）
#   BASH_COMMAND_DETECTOR_BREAK=python-relative-path-ignores-cd  ⑧ 的 python 一半相对路径不跟 cd、一律按仓内判（cd 进草稿目录里的仓副本再写 crates/…、scripts/… 也拒）：实测红的格（照失败行原文）
#       「就地改仓内文件:cd 到草稿目录里的仓副本之后 python 相对路径写 crates/ 放行」「就地改仓内文件:cd 到草稿目录里的仓副本之后 python 相对路径挪进 scripts/ 放行」「就地改仓内文件:cd 到草稿目录里的仓副本之后喂给 python 的 heredoc 相对路径写 scripts/ 放行」（共 3 格）
#   BASH_COMMAND_DETECTOR_BREAK=scope-script-variable-unexpanded  ⑫ 的脚本词不代入前面赋过的值，代入不出的也不交 UnresolvedScriptTarget
#       「Bash写范围:clerk 脚本路径在变量里，用 replace-once.py 写知识库」「Bash写范围:clerk 脚本路径在变量里，用 insert-row.py 写知识库」
#       「Bash写范围:clerk 脚本路径在变量里，用 replace-batch.py、规格点名知识库」「Bash写范围:clerk 直接执行变量里的 replace-once.py 写知识库」
#       「Bash写范围:python3 起没赋过值的变量里的脚本、第一个参数落在知识库，拒」「Bash写范围:python3 起没赋过值的变量里的脚本、第一个参数落在写范围里的 records 也拒」
#       「Bash写范围:python3 起命令替换给的脚本、第一个参数是仓根起的绝对路径，拒」（共 7 格）
#   BASH_COMMAND_DETECTOR_BREAK=builtin-agents-unjudged  ⑫ 没有项目定义的 agent 一概不判（内置类型、定义已删的旧实例写仓里、写仓外别处都放过）：实测红的格（照失败行原文）
#       「Bash写范围:没有项目定义的内置 agent 写仓里拒」「Bash写范围:没有项目定义的内置 agent 写仓外别处拒」「Bash写范围:没有项目定义的内置 agent 写会话临时目录根拒（scratchpad 之外）」「Bash写范围:没有项目定义的内置 agent 写会话 scratchpad 拒」「Bash写范围:没有项目定义的内置 agent 用 replace-once.py 写仓里拒」「Bash写范围:定义已删的旧实例写仓里拒」（共 6 格）
#   BASH_COMMAND_DETECTOR_BREAK=allow-whole-index-reset  ⑮ 暂存区非空时不带具体路径清整个索引的写法也放行：实测红的格（照失败行原文）
#       「一百三十七：暂存区非空时 git stash 照拒（退出码 2）」「一百三十七：暂存区非空时 git rm -r --cached . 照拒（退出码 2）」「一百三十七：暂存区非空时 git checkout HEAD -- . 照拒（退出码 2）」「一百三十七：暂存区非空时子命令写成没赋过值的变量照拒（退出码 2）」「清整个索引:git reset 拒」
#       「清整个索引:git reset HEAD 拒」「清整个索引:git reset --mixed 拒」「清整个索引:git reset -q --hard 拒」「清整个索引:git restore --staged . 拒」「清整个索引:git restore --staged :/ 拒」「清整个索引:git -C 点名暂存了的仓、起点在别处，拒」
#       「清整个索引:cd 进暂存了的仓之后 git reset HEAD 拒」「清整个索引:cd 到没赋过值的变量之后 git reset，暂存区判不了，拒」「stdin:主 agent 在暂存区非空时 git reset 拒、出路写只清自己的路径（退出码 2）」
#       「stdin:子 agent 在暂存区非空时 git reset 拒、出路写只清自己的路径（退出码 2）」（共 15 格）
#   BASH_COMMAND_DETECTOR_BREAK=whole-index-staged-unchecked  ⑮ 不看暂存区空不空（空的也拒）
#       「清整个索引:暂存区为空 git reset 放行」「清整个索引:暂存区为空 git restore --staged . 放行」（共 2 格）
#   BASH_COMMAND_DETECTOR_BREAK=whole-index-pathspec-ignored  ⑮ 带具体路径的也当清整个索引
#       「清整个索引:git reset HEAD -- a.txt 带路径放行」「清整个索引:git reset a.txt 带路径放行」「清整个索引:git restore --staged -- a.txt b.txt 带路径放行」（共 3 格）
#   BASH_COMMAND_DETECTOR_BREAK=allow-ledger-append  ⑯ 往现行账本直接追加节标题的写法也放行：实测红的格（照失败行原文）
#       「往现行账本追加:cat >> 加 heredoc 拒」「往现行账本追加:echo >> 拒」「往现行账本追加:管道接 tee -a 拒」「往现行账本追加:cd 进 records 之后 >> 拒」「往现行账本追加:绝对路径 >> 拒」
#       「往现行账本追加:喂给 python 的 heredoc 里 open(…, 'a') 拒」「stdin:主 agent 往现行账本追加节标题拒、出路写经 ledger-append-section.py（退出码 2）」
#       「stdin:子 agent 往现行账本追加节标题拒、出路写经 ledger-append-section.py（退出码 2）」（共 8 格）
#   BASH_COMMAND_DETECTOR_BREAK=allow-path-discard  ⑱ 带路径撤掉工作区里没暂存改动的写法也放行：红的格（照失败行原文）
#       「撤工作区改动:git checkout -- 有改动的路径拒」「撤工作区改动:git checkout HEAD -- 有改动的路径拒」「撤工作区改动:git checkout 不带 -- 的有改动路径拒」
#       「撤工作区改动:git restore 有改动的路径拒」「撤工作区改动:git restore --staged --worktree 有改动的路径拒」「撤工作区改动:git stash push -- 有改动的路径拒」
#       「撤工作区改动:git stash push -m 带说明、有改动的路径拒」「撤工作区改动:git clean -f 点名没跟踪的文件拒」「撤工作区改动:有改动的路径与没改动的路径同写拒」
#       「撤工作区改动:git checkout -- . 整棵树里有改动拒」「撤工作区改动:前面赋过值的变量照展开，有改动拒」「撤工作区改动:cd 进仓之后 git checkout -- 有改动的路径拒」
#       「撤工作区改动:git -C 点名仓、起点在别处，拒」「撤工作区改动:路径写成没赋过值的变量，判不了，拒」「撤工作区改动:cd 到没赋过值的变量之后，判不了，拒」
#       「stdin:主 agent 带路径撤掉没暂存的改动拒、出路写先留底与 git apply -R（退出码 2）」「stdin:子 agent 带路径撤掉没暂存的改动拒、出路写先留底与 git apply -R（退出码 2）」（共 17 格）
#   BASH_COMMAND_DETECTOR_BREAK=path-discard-changes-unchecked  ⑱ 不查点名的路径有没有改动，点名的都当有改动
#       「撤工作区改动:git checkout -- 没改动的路径放行」「撤工作区改动:git restore 没改动的路径放行」「撤工作区改动:只暂存了、工作区与暂存区相同的路径放行」
#       「撤工作区改动:git stash push -- 没改动的路径放行」「撤工作区改动:git clean -f 点名的路径下没有没跟踪的文件放行」「撤工作区改动:不在仓里的路径放行」（共 6 格）
#   BASH_COMMAND_DETECTOR_BREAK=path-discard-staged-counted  ⑱ git restore --staged 也当动工作区
#       「撤工作区改动:git restore --staged 有改动的路径放行（只动暂存区）」「撤工作区改动:git restore -S 有改动的路径放行（只动暂存区）」（共 2 格）
#   BASH_COMMAND_DETECTOR_BREAK=break-honored-outside-selftest-with-others  带标记的自证子进程里，关守卫项 break-honored-outside-selftest 与别的项同写也照认
#       （写在哪一项都认；只在带标记的子进程里认，平常跑同写照拒）
#       「关守卫项与别的项同写：带标记的子进程里也退 2 并点名它」（共 1 格）
#   LIB_SHELL_WORDS_BREAK=shell-standard-input-unread  共用模块 lib_shell_words.py 不把 here-string 与管道喂给 shell 的文本当命令再切、也不交判不了的说明：实测红的格（照失败行原文）
#       「stdin:here-string 喂给 bash 的是变量放行（退出码 0）、记一条检出」「stdin:拒了的命令里有判不了的输入（grep 的输出经管道喂给 bash），照记一条、结局被拒」「stdin:here-string 里前台起看门狗照拒（退出码 2）」
#       「一百三十七：子 agent here-string 喂给 bash 的 claude 照拒（退出码 2）」（共 4 格）
#   LIB_SHELL_WORDS_BREAK=startup-files-unread  共用模块 lib_shell_words.py startup_file_words() 一个文件都不交，shell 启动时先读的文件（BASH_ENV / ENV、--rcfile、.bashrc、zsh 的）都不当被起的脚本：实测红的格（照失败行原文）
#       「起 claude:一百三十七：子 agent BASH_ENV 指到的脚本里起 claude」「起 claude:一百三十七：子 agent env BASH_ENV=… 起 bash，脚本里起 claude」「起 claude:一百三十七：子 agent 前面 export BASH_ENV 再起 bash，脚本里起 claude」
#       「起 claude:一百三十七：子 agent ENV 指到的脚本里起 claude，sh 起的」「起 claude:一百三十七：子 agent BASH_ENV 写相对路径，按命令开头的当前目录解」「起 claude:一百三十七：子 agent 直接执行 #!/bin/bash 的脚本，BASH_ENV 指到的脚本里起 claude」
#       「起 claude:一百三十七：子 agent 直接执行 #!/bin/sh 的脚本，ENV 指到的脚本里起 claude」「起 claude:一百三十七：子 agent declare -x BASH_ENV 之后起 bash，脚本里起 claude」「起 claude:一百三十七：子 agent typeset -x BASH_ENV 之后起 bash，脚本里起 claude」
#       「起 claude:一百三十七：子 agent 先赋值后 export BASH_ENV，再起 bash，脚本里起 claude」「起 claude:一百三十七：子 agent bash -i 读 HOME 下的 .bashrc 里起 claude」「起 claude:一百三十七：子 agent bash --rcfile 文件 -i，文件里起 claude」
#       「起 claude:一百三十七：子 agent sh -i 读 ENV 指到的脚本里起 claude」「起 claude:一百三十七：子 agent zsh 读 ZDOTDIR 下的 .zshenv 里起 claude」（共 14 格）
#       「起 claude:d137 已知没认：登录 bash -l 读 HOME 下的 .bash_profile 里起 claude」「起 claude:d137 已知没认：登录 exec -l bash 照登录 shell 读 .bash_profile 里起 claude」
#       「起 claude:d137 已知没认：登录 sh -l 读 HOME 下的 .profile 里起 claude」「起 claude:d137 已知没认：系统级 bash -l 读 /etc/profile（样本目录）里起 claude」
#       「起 claude:d137 已知没认：系统级 bash -i 读 /etc/bash.bashrc（样本目录）里起 claude」
#   LIB_SHELL_WORDS_BREAK=declared-exports-unseen  共用模块 lib_shell_words.py 环境里的变量只认 `export 名字=值`（declare -x、typeset -x、local -x、先赋值后 export、先 export 后赋值、export -n、declare +x、set -a 都不认）：实测红的格（照失败行原文）
#       「起 claude:一百三十七：子 agent declare -x BASH_ENV 之后起 bash，脚本里起 claude」「起 claude:一百三十七：子 agent typeset -x BASH_ENV 之后起 bash，脚本里起 claude」「起 claude:一百三十七：子 agent 先赋值后 export BASH_ENV，再起 bash，脚本里起 claude」
#       「起 claude:一百三十七：子 agent export -n 撤掉 BASH_ENV 之后起 bash」（共 4 格）
#   LIB_SHELL_WORDS_BREAK=interactive-startup-files-unread  共用模块 lib_shell_words.py 带 -i 不另看（交互的 bash 照旧读 BASH_ENV、不读 .bashrc 与 --rcfile / --init-file）：实测红的格（照失败行原文）
#       「起 claude:一百三十七：子 agent bash -i 读 HOME 下的 .bashrc 里起 claude」「起 claude:一百三十七：子 agent bash --rcfile 文件 -i，文件里起 claude」「起 claude:一百三十七：子 agent bash -i 不读 BASH_ENV，.bashrc 里没有 claude」（共 3 格）
#       「起 claude:d137 已知没认：系统级 bash -i 读 /etc/bash.bashrc（样本目录）里起 claude」
#   LIB_SHELL_WORDS_BREAK=zsh-startup-files-unread  共用模块 lib_shell_words.py zsh 的 .zshenv、.zshrc 不读：实测红的格（照失败行原文）
#       「起 claude:一百三十七：子 agent zsh 读 ZDOTDIR 下的 .zshenv 里起 claude」（共 1 格）
#   LIB_SHELL_WORDS_BREAK=executed-script-startup-unread  共用模块 lib_shell_words.py 直接执行的脚本起来时它的 shell 先读的启动文件不读，#! 指到认不出的解释器的也不判：实测红的格（照失败行原文）
#       「起 claude:一百三十七：子 agent 直接执行 #!/bin/bash 的脚本，BASH_ENV 指到的脚本里起 claude」「起 claude:一百三十七：子 agent 直接执行 #!/bin/sh 的脚本，ENV 指到的脚本里起 claude」（共 2 格）
#   共用模块的边角开关：
#   LIB_SHELL_WORDS_BREAK=redirect-fd-dropped  共用模块把 `2> 文件` 也当改标准输出：实测红的格（照失败行原文）
#       「看门狗:run_in_background 起、管道后段只丢 2 号放行」（共 1 格）
#   LIB_SHELL_WORDS_BREAK=shell-c-options-unparsed  共用模块取紧跟 -c 的词当代码
#       「起 claude:一百三十七：子 agent bash -n 只查语法，不读 BASH_ENV」（共 1 格）
#   LIB_SHELL_WORDS_BREAK=time-options-as-command  共用模块不认 time 关键字的 -p / --
#       「等待循环:前台 time -p 包着的 bash -c 里没有 timeout」（共 1 格）
#   LIB_SHELL_WORDS_BREAK=double-slash-kept  共用模块的 resolve_path 留着开头的 //
#       「就地改仓内文件:cd 到 // 开头写的 /tmp/ 下的目录之后 sed -i 相对路径放行」（共 1 格；推的，没跑）
#   LIB_SHELL_WORDS_BREAK=heredoc-open-substitution-unseen  共用模块对挂在没收尾的 $( / <( 里的 bash 上的 heredoc 照数据剥掉：实测红的格（照失败行原文）
#       「等待循环:前台 $(bash <<EOF) 里的循环」「等待循环:前台 <(bash <<EOF) 里的循环」「等待循环:前台 $'a\'b' 之后 $(bash <<EOF) 里的循环」（共 3 格）
#   共用模块的双引号里 `$(` 开关：
#   LIB_SHELL_WORDS_BREAK=double-quoted-substitution-unclosed  共用模块把双引号里的 `$(` 当没收尾
#       「等待循环:前台 cd "$(…)" 之后 bash <<EOF 里的循环」「等待循环:前台 echo "$(date)" | bash <<EOF 里的循环」（共 2 格）
#   共用模块的其余开关：
#   LIB_SHELL_WORDS_BREAK=shell-syntax-only-code-run  共用模块带 -n 时照判 -c 的代码
#       「起 claude:一百三十七：子 agent bash -n 只查语法，不读 BASH_ENV」（共 1 格）
#   LIB_SHELL_WORDS_BREAK=ansi-c-quotes-unseen  共用模块把 `$'…'` 照普通单引号配对
#       「等待循环:前台 $'a\'b' 之后 $(bash <<EOF) 里的循环」（共 1 格）
#   LIB_SHELL_WORDS_BREAK=case-patterns-as-commands  共用模块不改 case 的模式
#       「stdin:子 agent case 里 ;; 之后的 reboot 模式不当命令放行（退出码 0）」
#       「stdin:子 agent case 里 ;; 之后的 claude 模式不当命令放行（退出码 0）」（共 2 格）
#   （conditional-expression-split、brace-sequence-width-padded-ends-only、extended-assignments-as-command、link-source-from-current-directory、env-leading-equals-as-command、uncalled-functions-run、shell-s-positionals-as-script、builtin-as-command、shell-option-values-as-script、function-keyword-as-command、braces-unexpanded、env-loose-assignments-as-command、eval-unread、brace-empties-kept、brace-sequence-unpadded、function-calls-literal-only、redirect-fd-number-literal、redirect-fd-quoted-digits、redirect-fd-duplication-unseen、shell-long-options-after-short、env-options-after-assignment、posix-uncalled-functions-run 下这里不红：这一份自证里没有这几类的格，由项目代码闸的格判红）
#   BASH_COMMAND_DETECTOR_BREAK=compile-first-capped-hint  ⑨ 的出路写成 capped.sh <线程上限> 直接起 compile-then-swap.py：
#       「stdin:先编后换的出路经 task-server 提交、不写 capped.sh 直接起」（共 1 格）
#   BASH_COMMAND_DETECTOR_BREAK=compile-first-ignores-cwd  ⑨ 的相对路径一律从仓库根起算（compile_first_verdict 不看 hook 输入的 cwd）：
#       「先编后换:执行员 cwd 在仓副本里、相对路径写 crates/ 下的 .rs 放行」「先编后换:执行员 cwd 在主工作区的 crates/ 里、相对路径写 .rs 拒」（共 2 格）
# 工作根的两个弄坏开关在同目录 lib_work_root.py（不经 BASH_COMMAND_DETECTOR_BREAK），另一个 ⑦ 的在下面：
#   SINGLEFS_WORK_ROOT_BREAK=new-root-unknown  工作根退回会话临时目录根：⑧ 不放行工作根下的 sed -i，⑦ 不认工作根下的脚本：实测红的格（照失败行原文）
#       「就地改仓内文件:执行员 sed -i 改工作根下的草稿目录」「就地改仓内文件:执行员 sed -i 改工作根下的草稿目录（~ 写法）」「就地改仓内文件:cd 到工作根下的草稿目录之后 sed -i 相对路径放行」「就地改仓内文件:cd 到前面赋过值的草稿目录之后 sed -i 相对路径放行」
#       「就地改仓内文件:命令开头的当前目录在草稿目录里，sed -i 相对路径放行」「Bash写范围:没有项目定义的内置 agent 写工作根放行」「Bash写范围:模板在自己认领的草稿目录里：T=$(mktemp -d $W/名.XXXXXX) 之后 cp 进 $T/ 不记」
#       「Bash写范围:模板在自己认领的草稿目录里：直接写进 $(mktemp -d …)/文件 不记」「Bash写范围:cd 进自己认领的草稿目录之后模板写相对路径不记」「Bash写范围:还没有认领文件、派发回执对得上那把锁：模板在它的草稿目录里不记」
#       「Bash写范围:-p 指到自己认领的草稿目录：T=$(mktemp -d -p "$W") 之后写进 $T/ 不记」「Bash写范围:--tmpdir= 指到自己认领的草稿目录（带模板名）：cp 进 $T/ 不记」「Bash写范围:cd 进自己认领的草稿目录之后 -p . 不记」「Bash写范围:直接写进 $(mktemp -d -p …)/文件 不记」
#       「Bash写范围:套一层 cd 加 pwd -P：T=$(cd $(mktemp -d -p "$W") && pwd -P) 之后写进 $T/ 不记」「Bash写范围:套一层、反引号写法：T=`cd $(mktemp -d -p …) && pwd` 之后写进 $T/ 不记」
#       「Bash写范围:套一层、直接写进 $(cd $(mktemp -d -p …) && pwd -P)/文件 不记」「stdin:模板在自己认领的草稿目录里的 mktemp -d 放行（退出码 0）、不记检出」「就地改脚本:工作根下的脚本」「stdin:cd 到草稿目录之后 sed -i 相对路径放行（退出码 0）」「Bash写范围:没有项目定义的内置 agent 写会话临时目录根拒（scratchpad 之外）」「Bash写范围:没有项目定义的内置 agent 写会话 scratchpad 拒」（共 22 格；后两格是工作根退回会话临时目录根之后它成了工作根）
#   SINGLEFS_WORK_ROOT_BREAK=legacy-root-accepted  写范围另认会话临时目录根：⑫ 放行没有项目定义的 agent 写会话临时目录根
#       「Bash写范围:没有项目定义的内置 agent 写会话临时目录根拒（scratchpad 之外）」「Bash写范围:没有项目定义的内置 agent 写会话 scratchpad 拒」（共 2 格）
#   BASH_COMMAND_DETECTOR_BREAK=in-place-script-session-temporary-root-unseen  ⑦ 不认会话临时目录根下的脚本（各会话的 scratchpad 在里面）
#       「就地改脚本:会话临时目录根下的脚本」「就地改脚本:会话 scratchpad 里的脚本」（共 2 格）
# 一百三十七节那一批（records/2026-10-29-一百三十七节那一轮调度记录.md 件 1）的开关与各自红的格（格名都以「一百三十七：」起头，另有两格改判的旧格；
# 这一批的格在已有的 disable-check 等开关下也红，写了「共 N 格」的已有开关在一百三十七节件 1b 照实测重数过、点名改成实测红的格）：
#   claude-launch-layers-narrow  「一百三十七：子 agent here-string 喂给 bash 的 claude 照拒（退出码 2）」「…env -S 拆串里的 claude…」「…xargs 起的 claude…」「…ssh 起的 claude…」四格
#   watchdog-tee-null-unseen  「一百三十七：看门狗输出经 tee 丢进 /dev/null 照拒（退出码 2）」
#   watchdog-null-redirect-whole-command  ① 整条命令里哪一段的 /dev/null 都算：
#       「看门狗:run_in_background 起、前一段 grep 2>/dev/null、后一段 watch.sh 放行」「看门狗:run_in_background 起、管道后段只丢 2 号放行」（共 2 格）
#   rsync-inplace-unseen  「就地改脚本:一百三十七：rsync --inplace 盖已有脚本（正推腿乙 Z8；rsync-inplace-unseen 下红）」（history-kept: 自证用例标签原文，按原文匹配）
#   partial-wait-allowed  「一百三十七：两个 & 只等其中一个照拒（退出码 2）」「单独的 &:几个 & 只 wait 一个 pid：剩下的照拒」
#   wait-forms-unseen  「一百三十七：一个 & 之后 wait -n 放行（退出码 0）」「…tail --pid 等它放行…」「…kill -0 轮询等它放行…」「一百三十七：wait 写在函数里、函数被调用放行（退出码 0）」
#       「单独的 &:& 之后按 pid 轮询（tail --pid）算等到了（与检出 ② 同一套数法）」
#   whole-index-other-forms-unseen  「一百三十七：暂存区非空时 git stash 照拒（退出码 2）」「…git rm -r --cached . 照拒…」「…git checkout HEAD -- . 照拒…」三格
#   git-subcommand-unexpanded-allowed  「一百三十七：暂存区非空时子命令写成没赋过值的变量照拒（退出码 2）」
#   brace-target-reason-generic  「一百三十七：重定向目标花括号照拒，拒绝说明点名 ambiguous redirect 与改法」
#   wait-loop-rewrite-unexplained  「一百三十七：带计数上限的前台等待循环照拒，拒绝说明点名真会结束的几种也要套 timeout」
#   coproc-prefix-unseen  「一百三十七：coproc 照拒（退出码 2）」、「放出追踪:coproc」（共用模块把 coproc 当关键字跳过之后，只靠前缀名认它）
#   startup-file-claude-unread  ⑬ 不读 shell 启动时先读的文件：实测红的格（照失败行原文）
#       「起 claude:一百三十七：子 agent BASH_ENV 指到的脚本里起 claude」「起 claude:一百三十七：子 agent env BASH_ENV=… 起 bash，脚本里起 claude」「起 claude:一百三十七：子 agent 前面 export BASH_ENV 再起 bash，脚本里起 claude」
#       「起 claude:一百三十七：子 agent ENV 指到的脚本里起 claude，sh 起的」「起 claude:一百三十七：子 agent BASH_ENV 写相对路径，按命令开头的当前目录解」「起 claude:一百三十七：子 agent 直接执行 #!/bin/bash 的脚本，BASH_ENV 指到的脚本里起 claude」
#       「起 claude:一百三十七：子 agent 直接执行 #!/bin/sh 的脚本，ENV 指到的脚本里起 claude」「起 claude:一百三十七：子 agent declare -x BASH_ENV 之后起 bash，脚本里起 claude」「起 claude:一百三十七：子 agent typeset -x BASH_ENV 之后起 bash，脚本里起 claude」
#       「起 claude:一百三十七：子 agent 先赋值后 export BASH_ENV，再起 bash，脚本里起 claude」「起 claude:一百三十七：子 agent bash -i 读 HOME 下的 .bashrc 里起 claude」「起 claude:一百三十七：子 agent bash --rcfile 文件 -i，文件里起 claude」
#       「起 claude:一百三十七：子 agent sh -i 读 ENV 指到的脚本里起 claude」「起 claude:一百三十七：子 agent zsh 读 ZDOTDIR 下的 .zshenv 里起 claude」（共 14 格）
#   bash-layer-unrecorded  「一百三十七：钩子 bash 这一层起不来（假 python3 退 127）：放行（退出码 0）、记一条检出」（只在带自证标记的子进程里认）
#   python-missing-unrecorded  PATH 上没有 python3 时不在 preflight 之前记检出：「d137 H13：PATH 上没有 python3：放行（退出码 0）、记一条检出」「d137 H13：PATH 上没有 python3、工作根取不到：放行（退出码 0）、不往家目录下写、stderr 说检出没记成」（共 2 格，只在带自证标记的子进程里认）
#   ⑬ 这一道读脚本正文与别的程序里起的 claude 的开关与各自红的格（「起 claude:d137 …」）：
#   script-body-claude-unread  ⑬ 不读 bash / sh / source 起的与直接执行的脚本正文：「起 claude:d137 已知没认：bash x.sh 起的脚本正文里起 claude」「起 claude:d137 已知没认：source 读进来的脚本正文里起 claude」「起 claude:d137 已知没认：直接执行的 #!/usr/bin/env bash 脚本正文里起 claude」「起 claude:d137 已知没认：同一条命令里 heredoc 写出再 bash 起，正文里起 claude」（共 4 格）
#   embedded-launch-claude-unread  ⑬ 不认别的程序里起的 claude、不读它们读的程序文件：「起 claude:d137 H9：python -c 里 os.system 起 claude」「起 claude:d137 H9：perl -e 里 system 起 claude」「起 claude:d137 H9：gawk 的 system() 起 claude」「起 claude:d137 H9：sed 的 e 命令起 claude」「起 claude:d137 H9：git -c 别名 ! 起 claude」「起 claude:d137 H9：tclsh 读 here-string 里 exec 起 claude」「起 claude:d137 H9：make 读 Makefile 配方起 claude」「起 claude:d137 H9：awk -f 程序文件里 system() 起 claude」（共 8 格）
#   LIB_SHELL_WORDS_BREAK=login-profiles-unread  共用模块不认登录 shell 读的 profile：
#       「起 claude:d137 已知没认：登录 bash -l 读 HOME 下的 .bash_profile 里起 claude」「起 claude:d137 已知没认：登录 exec -l bash 照登录 shell 读 .bash_profile 里起 claude」「起 claude:d137 已知没认：登录 sh -l 读 HOME 下的 .profile 里起 claude」「起 claude:d137 已知没认：系统级 bash -l 读 /etc/profile（样本目录）里起 claude」（共 4 格）
#   LIB_SHELL_WORDS_BREAK=system-startup-files-unread  共用模块不交系统级启动文件：
#       「起 claude:d137 已知没认：系统级 bash -l 读 /etc/profile（样本目录）里起 claude」「起 claude:d137 已知没认：系统级 bash -i 读 /etc/bash.bashrc（样本目录）里起 claude」（共 2 格）
#   LIB_SHELL_WORDS_BREAK=embedded-launches-unseen  共用模块 embedded_launches() 一条都不认：
#       「起 claude:d137 H9：python -c 里 os.system 起 claude」「起 claude:d137 H9：perl -e 里 system 起 claude」「起 claude:d137 H9：gawk 的 system() 起 claude」「起 claude:d137 H9：sed 的 e 命令起 claude」「起 claude:d137 H9：git -c 别名 ! 起 claude」「起 claude:d137 H9：tclsh 读 here-string 里 exec 起 claude」「起 claude:d137 H9：make 读 Makefile 配方起 claude」「起 claude:d137 H9：awk -f 程序文件里 system() 起 claude」（共 8 格）
#   这一批的格在已有开关下也红：startup-file-claude-unread：「起 claude:d137 已知没认：登录 bash -l 读 HOME 下的 .bash_profile 里起 claude」「起 claude:d137 已知没认：登录 exec -l bash 照登录 shell 读 .bash_profile 里起 claude」「起 claude:d137 已知没认：登录 sh -l 读 HOME 下的 .profile 里起 claude」「起 claude:d137 已知没认：系统级 bash -l 读 /etc/profile（样本目录）里起 claude」「起 claude:d137 已知没认：系统级 bash -i 读 /etc/bash.bashrc（样本目录）里起 claude」（5 格）；
#     claude-launch-layers-narrow：「起 claude:d137 H9：python -c 里 os.system 起 claude」「起 claude:d137 H9：perl -e 里 system 起 claude」「起 claude:d137 H9：gawk 的 system() 起 claude」「起 claude:d137 H9：sed 的 e 命令起 claude」「起 claude:d137 H9：git -c 别名 ! 起 claude」「起 claude:d137 H9：tclsh 读 here-string 里 exec 起 claude」（6 格）；
#     LIB_SHELL_WORDS_BREAK=startup-files-unread：「起 claude:d137 已知没认：登录 bash -l 读 HOME 下的 .bash_profile 里起 claude」「起 claude:d137 已知没认：登录 exec -l bash 照登录 shell 读 .bash_profile 里起 claude」「起 claude:d137 已知没认：登录 sh -l 读 HOME 下的 .profile 里起 claude」「起 claude:d137 已知没认：系统级 bash -l 读 /etc/profile（样本目录）里起 claude」「起 claude:d137 已知没认：系统级 bash -i 读 /etc/bash.bashrc（样本目录）里起 claude」（5 格）；interactive-startup-files-unread：「起 claude:d137 已知没认：系统级 bash -i 读 /etc/bash.bashrc（样本目录）里起 claude」（1 格）
# 误拒修补的开关与各自红的格（自证里「d137 误拒」开头的格）：
#   BASH_COMMAND_DETECTOR_BREAK=builtin-coproc-detached  ③ 的 coproc 照命令词前面那段的词认、不看共用模块记没记它（`builtin coproc …` 照拒）：
#       「stdin:d137 误拒：builtin coproc 里 coproc 不是内建、bash 报错不跑，放行（退出码 0）」「stdin:d137 误拒：builtin -- coproc 里 coproc 不是内建、bash 报错不跑，放行（退出码 0）」（共 2 格）
#   LIB_SHELL_WORDS_BREAK=heredoc-variables-unexpanded  共用模块不代入 heredoc 正文里前面赋过字面值的变量：
#       「d137 误拒 E1：python3 读 heredoc，正文变量前面赋过字面值，照代入、不交判不了」（共 1 格）
#   LIB_SHELL_WORDS_BREAK=python-list-concatenation-unresolved  共用模块不认 python argv 的 `+ 列表` 与 `*列表`：
#       「d137 误拒 E6：subprocess.run([du, -sch] + ds)，ds 是赋过的字面列表，照算出的命令交出」「d137 误拒 E6：subprocess.run([du, -sch, *ds])，ds 算不出，照命令词 du 交出」（共 2 格）
#   LIB_SHELL_WORDS_BREAK=python-compile-as-execution  共用模块把 python 的 compile 当执行：
#       「d137 误拒 E7：python 只调 compile 不执行，不交判不了」（共 1 格）
#   LIB_SHELL_WORDS_BREAK=sed-unknown-flags-as-execution  共用模块把 sed 的 s 标志区里的字母 e 一律当 e 标志：
#       「d137 误拒 E8：sed 的 s 命令标志区有 GNU sed 不认的字（分隔符被转义的反斜杠提前收尾），编译就报错、什么都不起」（共 1 格）
#   LIB_SHELL_WORDS_BREAK=cd-subshell-guards-unseen  共用模块里括号进出一律断开 cd 的 `&&` 守护：
#       「d137 误拒 E9：cd 不在的目录 && ( bash x.sh )，子 shell 照 && 守着」「d137 误拒 E9：cd 不在的目录 && time ( bash x.sh )，照 && 守着」（共 2 格）
#   这一批的格在已有开关下也红：cd-failure-guards-unseen 下 E9 照 && 守着的两格，python-argument-variables-unresolved 下 E6 命令词是 du 的两格，
#     builtin-non-builtin-run 下 builtin coproc 两格（这三项的开关这份文件头没列，自证在它们下的别的格不红）
# 一百三十七节件 1b（d137-hooks-2）的开关与各自红的格：
#   background-ampersand-by-regex  检出 ② 的单独 & 按正则数：「bg-notify-r1 F06：sed 替换串引号里的 & 不记」
#       「一百三十七：两个子 shell 各记 $!、逐个 wait "$pN" 不记」（共 2 格）
#   bare-setsid-unrecorded  检出 ② 不记不带 --wait 的 setsid：「一百三十七：裸 setsid 不带 --wait，照记一条」「一百三十七：内存包装里的 setsid 不带 --wait，照记一条」（共 2 格）
#   scope-launcher-writes-unseen  ⑫ 不认 eval 的代码与启动器里面那条：「Bash写范围:一百三十七：clerk eval 字面代码里 sed -i 知识库，拒」
#       「Bash写范围:一百三十七：clerk find -exec sed -i 读进来的路径，落点算不出，拒」「Bash写范围:一百三十七：clerk echo 喂给 xargs sed -i，落点算不出，拒」
#       「Bash写范围:一百三十七：clerk xargs sed -i 写范围里的 records 也拒（落点算不出）」「Bash写范围:一百三十七：clerk find -exec cp 到写死的知识库路径，拒」
#       「Bash写范围:一百三十七：钩子进程在工作根下，clerk xargs sed -i 照拒」（共 6 格）
#   sed-unknown-target-resolved-from-cwd  sed -i 展开不了的目标照钩子进程的当前目录补成绝对路径：
#       「Bash写范围:一百三十七：钩子进程在工作根下，clerk sed -i 没赋过值的变量照拒」「Bash写范围:一百三十七：钩子进程在工作根下，clerk xargs sed -i 照拒」（共 2 格）
#   BASH_COMMAND_DETECTOR_BREAK=allow-literal-notations  ⑰ 往仓里写的字面内容一律不判：
#       「字面写入:heredoc 追加会话名拒」「字面写入:heredoc 追加批次代号拒」「字面写入:printf 追加钟点拒」「字面写入:echo 经管道交 tee -a 追加批次代号拒」
#       「字面写入:cd 进仓里之后按相对路径写照判」「字面写入:bash -c 里的 echo 追加会话名照判」「字面写入:here-string 喂给 cat 写批次代号照判」
#       「stdin:Bash heredoc 往仓里追加会话名拒绝（退出码 2，点名会话号、出路指到 insert-row.py 与 replace-once.py）」（共 8 格）
#   BASH_COMMAND_DETECTOR_BREAK=literal-notations-outside-judged  ⑰ 仓外的目标也判（按文件名当相对路径）：
#       「字面写入:仓外路径不判」（共 1 格）
#   BASH_COMMAND_DETECTOR_BREAK=allow-record-append  ⑲ 往 records/ 下的 .md 追加一概放行：实测红的格（照失败行原文）
#       「往 records 追加:echo >> 拒」「往 records 追加:cat >> 加 heredoc 拒」「往 records 追加:&>> 拒」「往 records 追加:管道接 tee -a 拒」「往 records 追加:tee --append 拒」
#       「往 records 追加:cd 进 records 之后 >> 拒」「往 records 追加:绝对路径 >> 拒」「往 records 追加:前面赋过值的变量照代入拒」「往 records 追加:bash -c 里的 >> 拒」
#       「往 records 追加:python3 -c 里 open(…, 'a') 拒」「往 records 追加:喂给 python 的 heredoc 里 open(…, mode='a') 拒」「往 records 追加:python 的 Path.open('a') 拒」
#       「往 records 追加:子目录里的 .md 也拒」「往 records 追加:大写后缀 .MD 也拒」「往 records 追加:指进 records 的符号链接照落点拒」
#       「stdin:主 agent 往 records 追加拒、出路给 append-record.py 的整条写法（退出码 2）」「stdin:子 agent 往 records 追加拒、出路给 append-record.py 的整条写法（退出码 2）」
#       「出路过别的闸:主 agent 照抄 ⑲ 的出路喂 project-code-guard.sh 放行（退出码 0）」「出路过别的闸:子 agent 照抄 ⑲ 的出路喂 project-code-guard.sh 放行（退出码 0）」（共 19 格）
#   BASH_COMMAND_DETECTOR_BREAK=record-append-python-unseen  ⑲ 不认 python 的 open(…, 'a')：打开后红的格（照失败行原文）
#       「往 records 追加:python3 -c 里 open(…, 'a') 拒」「往 records 追加:喂给 python 的 heredoc 里 open(…, mode='a') 拒」「往 records 追加:python 的 Path.open('a') 拒」（共 3 格）
#   BASH_COMMAND_DETECTOR_BREAK=record-append-tee-unseen  ⑲ 不认 tee -a（不看 OverwriteCandidate.appends）：打开后红的格（照失败行原文）
#       「往 records 追加:管道接 tee -a 拒」「往 records 追加:tee --append 拒」（共 2 格）
#   BASH_COMMAND_DETECTOR_BREAK=record-howto-unlisted-tool  ⑲ 拒绝说明里的出路写成不在直接跑名单上的 scripts/kb/history-consolidate.py（照抄会被 project-code-guard.sh 拒）：
#       「stdin:主 agent 往 records 追加拒、出路给 append-record.py 的整条写法（退出码 2）」「stdin:子 agent 往 records 追加拒、出路给 append-record.py 的整条写法（退出码 2）」
#       「出路过别的闸:主 agent 照抄 ⑲ 的出路喂 project-code-guard.sh 放行（退出码 0）」「出路过别的闸:子 agent 照抄 ⑲ 的出路喂 project-code-guard.sh 放行（退出码 0）」（共 4 格）
#   BASH_COMMAND_DETECTOR_BREAK=howto-guard-environment-keeps-breaks  ⑲ 出路喂 project-code-guard.sh 时把自证设着的弄坏开关变量（LIB_SHELL_WORDS_BREAK 这类）原样带过去（它见到不是自己自证设的弄坏开关就拒，出路被误判过不了闸）：
#       「出路过别的闸:喂 project-code-guard.sh 的环境里不带弄坏开关变量，别的变量照带」（共 1 格）
#   BASH_COMMAND_DETECTOR_BREAK=allow-manual-repository-copy  ⑳ 手拷仓副本（rsync、cp -r / -R / -a 拷本仓的 crates/ 或仓根整个）一概放行：照判法推的格（还没复跑）
#       「手拷仓副本:rsync 拷 crates 进仓外目录」「手拷仓副本:cp -r 拷 crates」「手拷仓副本:rsync 拷仓根整个（./）」「手拷仓副本:rsync 拷仓根（绝对路径带结尾的 /）」
#       「手拷仓副本:cp -a 拷仓根 .」「手拷仓副本:cp -R 拷 crates/（带结尾的 /）」「手拷仓副本:cp --archive 拷 crates」「手拷仓副本:cp -t 目录写在前面、-r 写在后面」
#       「手拷仓副本:rsync 的 --exclude=值 合写」「手拷仓副本:rsync -e 带值的短选项之后的源照认」「手拷仓副本:cd 进仓根之后 rsync crates」
#       「手拷仓副本:前面赋过值的变量代入」「手拷仓副本:bash -c 里」「手拷仓副本:nice 前缀」「手拷仓副本:仓根里的通配 * 盖住整层」「手拷仓副本:crates/* 盖住 crates 整层」
#       「手拷仓副本:crates/.. 规整之后是仓根」「手拷仓副本:喂给 bash 的 heredoc 里」「手拷仓副本:rsync 一次给几个源、其中有 crates」
#       「stdin:主 agent rsync 拷 crates 拒、出路给 minimal-copy.py 的整条写法（退出码 2）」「stdin:子 agent cp -r 拷 crates 拒、出路给 minimal-copy.py 的整条写法（退出码 2）」
#       「出路过别的闸:主 agent 照抄 ⑳ 的出路喂 project-code-guard.sh 放行（退出码 0）」「出路过别的闸:子 agent 照抄 ⑳ 的出路喂 project-code-guard.sh 放行（退出码 0）」（共 23 格）
#   BASH_COMMAND_DETECTOR_BREAK=manual-copy-howto-unlisted-tool  ⑳ 拒绝说明里的出路写成不在直接跑名单上的 scripts/hosts/compile-then-swap.py（照抄会被 project-code-guard.sh 拒）：照判法推的格（还没复跑）
#       「出路过别的闸:主 agent 照抄 ⑳ 的出路喂 project-code-guard.sh 放行（退出码 0）」「出路过别的闸:子 agent 照抄 ⑳ 的出路喂 project-code-guard.sh 放行（退出码 0）」（共 2 格）
#   LIB_FORBIDDEN_NOTATIONS_BREAK=batch-codes  共用判据库不判批次代号这一形态（写法在 ../gate.d/lib-forbidden-notations.py 文件头）：
#       「字面写入:heredoc 追加批次代号拒」「字面写入:echo 经管道交 tee -a 追加批次代号拒」「字面写入:cd 进仓里之后按相对路径写照判」
#       「字面写入:here-string 喂给 cat 写批次代号照判」（共 4 格）
# ② 末那一句之外：前台等待循环的拒绝说明另写「带计数上限、break、while read、kill -0 探活这类真会结束的循环同样照拒，套 timeout 就放行」；
# ⑦ 另认 `rsync --inplace 源 已有文件`（就地写、不换 inode）；① 另认 `| tee /dev/null`（看门狗的输出照样丢了）；
# ⑫ 重定向目标上的花括号（`> o.{a,b}`、`> o{1..2}.txt`，引号里的同样）照落点算不出拒，拒绝说明点名 bash 报 ambiguous redirect、什么都不写，改法是写成一个字面文件名；
# 钩子 bash 这一层起不来（python3 起不来、/dev/fd/3 读不到、python 在 main() 之前崩了）时照「放行但必记检出」追加一行、退 0（一百三十七节 ⑤；带参数的起法不读标准输入）。
#   PATH 上没有 python3 的，preflight 判不了运行条件、会退 78 什么都不记：钩子那种起法在 preflight 之前就照「放行但必记检出」记一条、退 0
#   （弄坏开关 BASH_COMMAND_DETECTOR_BREAK=python-missing-unrecorded 下照旧交给 preflight 退 78，只在带自证标记的子进程里认）。
set -uo pipefail
# 钩子 bash 这一层起不来时也记一条检出（一百三十七节 ⑤）：python3 起不来、/dev/fd/3 读不到、python 在 main() 之前崩了，这条命令照常执行（钩子退出码不是 0、2 时 Claude Code 照常放行），
# 标准输入先在这里读进来、经 here-string 交给 python（python 起不来时 bash 还拿得到会话号）；退出码不是 0、2、又不是 --selftest 的，照「放行但必记检出」
# 往检出记录追加一行（会话号、agent 从输入里按字面取，取不到写「读不到」）、退 0；带参数的起法（--selftest、--scan-scripts）不读标准输入、照交退出码。弄坏开关 BASH_COMMAND_DETECTOR_BREAK=bash-layer-unrecorded 下不记（照旧把 python 的退出码交出去）。
hook_record_bash_layer_failure() {   # <钩子名> <python 的退出码> <hook 输入原文>
  local name="$1" status="$2" input="$3" path="${AGENT_HOOK_DETECTIONS:-}" work_root="${SINGLEFS_WORK_ROOT:-}" session="读不到" agent="读不到"
  # 检出记录的位置照 lib_work_root.py 取，不补缺省、不退回家目录（第二台的家目录是另一个）：AGENT_HOOK_DETECTIONS，没设是工作根下的 agent-hook-detections.jsonl；
  # 工作根是 SINGLEFS_WORK_ROOT，没设由同目录的 lib_work_root.py 交（python 起得来时）；都取不到就不记，stderr 说检出没记成与出路
  if [[ -z "$path" && -z "$work_root" ]]; then work_root="$(python3 "$(dirname "${BASH_SOURCE[0]}")/lib_work_root.py" 2>/dev/null)" || work_root=""; fi
  if [[ -z "$path" && -n "$work_root" ]]; then path="$work_root/agent-hook-detections.jsonl"; fi
  if [[ "$input" =~ \"session_id\"[[:space:]]*:[[:space:]]*\"([^\"\\]*)\" ]]; then session="${BASH_REMATCH[1]}"; fi
  if [[ "$input" =~ \"agent_type\"[[:space:]]*:[[:space:]]*\"([^\"\\]*)\" ]]; then agent="${BASH_REMATCH[1]}"; elif [[ -n "$input" ]]; then agent="主 agent"; fi
  if [[ -z "$path" ]]; then
    echo "  ! $name：python 那一层没起来或没判完（退出码 $status），这条命令没判、照常执行；检出没记成：取不到工作根（SINGLEFS_WORK_ROOT 没设，.claude/hooks/lib_work_root.py 也没交出来）" >&2
    echo "  → 怎么办：在多机配置里写 SHARED_DATA_ROOT（.claude/hooks/lib_work_root.py 文件头「数据根」），或设 SINGLEFS_WORK_ROOT、AGENT_HOOK_DETECTIONS；再修好 python3" >&2
    return 0
  fi
  echo "  ! $name：python 那一层没起来或没判完（退出码 $status），这条命令没判、照常执行；记了一条检出" >&2
  mkdir -p "$(dirname "$path")" 2>/dev/null
  printf '{"time": "%s", "session_id": "%s", "agent_type": "%s", "command": "", "findings": ["%s 没判成：钩子 bash 那一层起不来（python3 退出码 %s），这条命令没判、照常执行"], "outcome": "只记"}\n' \
    "$(date -u +%Y-%m-%dT%H:%M:%S+00:00)" "$session" "$agent" "$name" "$status" >> "$path" 2>/dev/null
}
hook_input=""
if [[ $# -eq 0 ]]; then hook_input="$(cat)"; fi   # 只有钩子那种起法（不带参数，从标准输入读 JSON）读；--selftest、--scan-scripts 这类不读
# PATH 上没有 python3：下面的 preflight 判不了运行条件，会退 78、什么都不记；钩子那种起法在它之前照「放行但必记检出」记一条、退 0（三方 d137-gov-r1 判决 H13）
if [[ $# -eq 0 ]] && ! command -v python3 >/dev/null 2>&1 && [[ "$(cat "${BASH_COMMAND_DETECTOR_SELFTEST_MARK:-/nonexistent}" 2>/dev/null)" != bash-command-detector-selftest || ",${BASH_COMMAND_DETECTOR_BREAK:-}," != *,python-missing-unrecorded,* ]]; then
  hook_record_bash_layer_failure "bash-command-detector.sh" "127，PATH 上没有 python3" "$hook_input"
  exit 0
fi
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
HOOK_DIR="$(cd "$(dirname "$0")" && pwd)"
# python 程序从文件描述符 3 读，标准输入留给 hook 的 JSON（与 agent-write-scope.sh 同一个坑）。
python3 /dev/fd/3 "$HOOK_DIR" "$@" <<<"$hook_input" 3<<'PY'
import ast, glob, importlib.util, io, json, os, re, shlex, shutil, subprocess, sys, tempfile, tokenize
from datetime import datetime, timezone
from typing import NamedTuple

def shell_words_library_path(hook_dir):
    return os.path.join(hook_dir, "lib_shell_words.py")

def load_shell_words(library_path):
    """切词与认命令位置的共用模块，与 project-code-guard.sh 同一份；读不到或导入出错就抛异常，由调用方处理。"""
    spec = importlib.util.spec_from_file_location("lib_shell_words", library_path)
    library = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(library)
    return library

try:
    shell_words, shell_words_error = load_shell_words(shell_words_library_path(sys.argv[1])), None
except Exception as error:  # 文件不在、语法错、导入时抛的都算读不到
    shell_words, shell_words_error = None, error
try:   # 工作根与会话临时目录根的唯一定义：同目录的 lib_work_root.py（读不到时 ⑦ 只认仓里、⑧ 只放行 /tmp/ 下，检出记录不写）
    sys.dont_write_bytecode = True
    sys.path.insert(0, sys.argv[1]); import lib_work_root as work_roots
    # ⑦ 认的根：草稿根（工作根）加会话临时目录根（各会话的 scratchpad 在里面，那里的脚本同样有 bash 在读）
    DEFAULT_DETECTIONS = work_roots.default_detections_path()
    SCRIPT_SCRATCH_ROOTS = tuple(dict.fromkeys((*work_roots.scratch_roots(), work_roots.claude_session_temporary_root())))
except Exception as error:  # 文件不在、语法错、弄坏开关写错都算读不到
    work_roots, DEFAULT_DETECTIONS, SCRIPT_SCRATCH_ROOTS = None, None, ()

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
    """检出 ②：放出追踪的那几种写法（SELF_DETACH，③ 拒了的不再记）与没等到的单独 &。单独的 & 与 ④ 同一套数法（unwaited_background_jobs 按层、按圆括号逐个数
    wait_coverage）：每个 & 记了 $!、之后逐个 `wait "$pN"` 的算等齐（主 agent 的 `( … ) & p1=$!; ( … ) & p2=$!; wait $p1; wait $p2` 不记）；
    引号里当数据写的 & 不算。弄坏开关 background-ampersand-by-regex 下按正则数（引号里的 & 照记，几个 & 而 `wait "$pid"` 逐个等的照记）。"""
    stripped = shell_words.strip_data_heredocs(command)
    if SELF_DETACH.search(stripped):
        return True
    if not detector_break_is_set("background-ampersand-by-regex"):
        return bool(unwaited_background_jobs(command))
    command = stripped
    if SUBSHELL_AMPERSAND.search(command):
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

def findings_for(command, run_in_background=False):
    if detector_break_is_set("disable-check"):
        return []
    findings = []
    executed = command if detector_break_is_set("keep-heredoc-bodies") else shell_words.strip_data_heredocs(command)
    if WAIT_LOOP.search(executed) and not re.search(r"\btimeout\b", executed):
        findings.append("没有 timeout 的等待循环")
    if run_in_background and puts_itself_in_background(command) and not detector_break_is_set("disable-self-background"):
        findings.append("run_in_background 里又自己放后台（nohup / setsid / disown / 后面没有 wait 的 &）：完成通知当场发出，跑完的那个不会叫醒你")
    if run_in_background and not detector_break_is_set("bare-setsid-unrecorded"):
        findings += [f"run_in_background 里 {form} 不带 --wait：它起的那一刻若是进程组组长（包装 exec 过去、bash -c 只有这一条），setsid 就 fork 一份、自己马上退出，"
                     "完成通知当场发出；写成 setsid --wait（-w）" for form in bare_setsid_forms(command)]
    return findings

def bare_setsid_forms(command):
    """命令位置上（每一层，前缀、capped.sh / run-with-memory-cap.sh 这类包装剥开）带 setsid 前缀、又没带 -w / --wait 的那几条（带 -f / --fork 的归 ③ 拒，这里不重复）。
    setsid 只在它自己是进程组组长时 fork、父进程马上退出；会不会是组长，执行前看不出（一百三十七节件 1b 第 5 条）。"""
    found = []
    for layer in shell_layers(command):
        for words in layer.commands:
            for prefix_words, _, name, arguments in wrapped_commands(words):
                for index, word in enumerate(prefix_words):
                    if os.path.basename(word) != "setsid":
                        continue
                    options = []
                    for option in prefix_words[index + 1:]:
                        if not option.startswith("-") or option == "-":
                            break
                        options.append(option)
                    short_letters = "".join(option[1:] for option in options if not option.startswith("--"))
                    if "--wait" in options or "w" in short_letters or "--fork" in options or "f" in short_letters:
                        continue
                    found.append(" ".join([word, *options, name or ""]).strip()[:120])
    return list(dict.fromkeys(found))

def record(hook_input, detections_path, extra_findings=()):
    """检出就追加一行；返回写了几行。永远不拦命令。extra_findings 是入口另外判出、只记不拦的（⑤ 目标算不出、git 判不了的，判不了的几样）。"""
    tool_input = hook_input.get("tool_input") or {}
    try:
        findings = findings_for(tool_input.get("command") or "", bool(tool_input.get("run_in_background")))
    except Exception as error:   # 判的时候抛了：照常执行，stderr 报一句、记进这一行检出
        message = not_judged(f"bash-command-detector.sh 没判成没超时的等待循环与后台里又放后台（{error!r}），这条命令照常执行")
        findings = [] if detector_break_is_set("failure-unrecorded") else [message]
    findings += list(extra_findings)
    if not findings:
        return 0
    append_detection(hook_input, findings, detections_path)
    return 1

# 一次判定里判不了的几样（某一道抛了异常、某个库读不到）：not_judged 打 stderr 并收进这里，main 把它们记进检出（拒了的那一条也记）
JUDGING_NOTICES = []

def not_judged(message):
    """判不了的一样：stderr 打一行，收进 JUDGING_NOTICES 由 main 记检出；交回这一句。"""
    print(f"  ! {message}", file=sys.stderr)
    JUDGING_NOTICES.append(message)
    return message

HOOK_INPUT_HEAD_BYTES = 200   # hook 输入不是 JSON 对象时，诊断里贴它开头这么多字节
# hook 输入读不出来时记检出用的那一份：会话号、agent 都读不到，照实写读不到
UNREADABLE_HOOK_INPUT = {"session_id": "读不到", "agent_id": "读不到", "agent_type": "读不到", "transcript_path": "读不到", "tool_input": {"command": ""}}

def parsed_hook_input(raw):
    """标准输入读到的字节 → (hook 输入, None)；不是 JSON 对象（解析出错、解出来不是对象）→ (None, 一行诊断：钩子名、为什么、开头若干字节)。"""
    try:
        hook_input = json.loads(raw)
    except Exception as error:   # 不是合法 JSON、不是 UTF-8
        why = f"解析出错：{error!r}"
    else:
        if isinstance(hook_input, dict):
            return hook_input, None
        why = f"解出来是 {type(hook_input).__name__}，不是对象"
    head = raw[:HOOK_INPUT_HEAD_BYTES]
    return None, (f"bash-command-detector.sh 收到的 hook 输入不是 JSON 对象（{why}），这条命令没判、照常执行；"
                  f"开头 {len(head)} 字节（共 {len(raw)} 字节）：{head!r}")

def append_detection(hook_input, findings, detections_path, outcome="只记"):
    """检出记录追加一行；outcome 是这条命令的结局：只记不拦的写「只记」，判不了的几样跟着一条被拒的命令记时写「被拒」。"""
    tool_input = hook_input.get("tool_input") or {}
    command = tool_input.get("command") or ""
    entry = {
        "time": datetime.now(timezone.utc).isoformat(),
        "session_id": hook_input.get("session_id"),
        "agent_id": hook_input.get("agent_id"),
        "agent_type": hook_input.get("agent_type") or "主 agent",
        "transcript_path": hook_input.get("transcript_path"),
        "command": (command if isinstance(command, str) else repr(command))[:500],
        "findings": findings,
    }
    if not detector_break_is_set("detection-outcome-dropped"):
        entry["outcome"] = outcome   # 照常执行的写「只记」；拒了的命令只在带着判不了的几样时才记，写「被拒」。看门狗按这一栏说下一步
    if not detections_path:   # 读不到 lib_work_root.py、又没设 AGENT_HOOK_DETECTIONS：不知道往哪写
        return
    os.makedirs(os.path.dirname(detections_path), exist_ok=True)
    with open(detections_path, "a", encoding="utf-8") as handle:
        handle.write(json.dumps(entry, ensure_ascii=False) + "\n")

# 起看门狗的写法：只认命令位置，所以先按 shell 的规矩切词（共用模块），不拿正则在整条文本上找名字。
WATCH_SH_MODES_THAT_DO_NOT_WATCH = {"--selftest", "--report", "--dry-run"}

def is_watchdog_call(command):
    name, arguments = os.path.basename(command.words[0]), command.words[1:]
    if name == "watch.sh":
        return not WATCH_SH_MODES_THAT_DO_NOT_WATCH.intersection(arguments)
    return name == "agent-watch.py" and "watch" in arguments and arguments[:1] != ["--selftest"]

def null_redirects_reaching_watchdog(scan):
    """整条命令里输出重定向到 /dev/null 的处数，减去挂在别的简单命令上、碰不到看门狗输出的那几处：那一条不是看门狗、不在管道后段，或在管道后段而只改 2 号这类非标准输出。
    共用模块的 output_targets 收全部层的重定向；每条命令的 output_redirections 只收它自己那一条简单命令上的（同一条简单命令交出的几条共用一个元组，按元组认一次）。
    包着看门狗的那一层（bash -c 整条、花括号与圆括号整组、exec）上的重定向不挂在哪一条交出来的命令上，减不掉，照算。"""
    groups = {}
    for command in scan.commands:
        redirections, carries_watchdog, piped = groups.get(id(command.output_redirections), (command.output_redirections, False, False))
        groups[id(command.output_redirections)] = (redirections, carries_watchdog or is_watchdog_call(command), piped or command.piped_from is not None)
    elsewhere = sum(1 for redirections, carries_watchdog, piped in groups.values() if not carries_watchdog
                    for operator, target in redirections if target == "/dev/null" and not (piped and shell_words.redirects_standard_output(operator)))
    return scan.output_targets.count("/dev/null") - elsewhere

def watchdog_calls_and_detaching(text):
    """返回 (命令位置上起看门狗的调用, 整条命令里放后台或丢输出的写法)；bash -c、capped.sh 这类包装里的由共用模块递归进去一起交回。"""
    scan = shell_words.commands_at_command_position(text)
    calls, detaching = [], []
    if scan.has_lone_ampersand:
        detaching.append("命令里有单独的 &")
    if ("/dev/null" in scan.output_targets if detector_break_is_set("watchdog-null-redirect-whole-command")
            else null_redirects_reaching_watchdog(scan) > 0):
        detaching.append("输出重定向到了 /dev/null")
    detaching += [f"命令里有 {prefix}" for prefix in ("nohup", "setsid") if prefix in scan.prefix_names]
    for command in scan.commands:
        name, arguments = os.path.basename(command.words[0]), command.words[1:]
        if name == "disown":
            detaching.append("命令里有 disown")
        if (name == "tee" and "/dev/null" in [argument for argument in arguments if not argument.startswith("-")]
                and not detector_break_is_set("watchdog-tee-null-unseen")):
            detaching.append("输出经 tee 丢进了 /dev/null")   # `watch.sh | tee /dev/null`
        if is_watchdog_call(command):
            calls.append(" ".join([name, *arguments]))
    return calls, detaching

def watchdog_rejection(command, run_in_background):
    """起看门狗的写法不对就返回 (原因, 认出的调用)；写法对、或根本没起看门狗，原因为空。"""
    if detector_break_is_set("allow-watchdog-misuse"):
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

try:   # 写范围表的读法与判法：同目录的 lib_write_scope.py，与 write-guard.sh 同一份（读不到时 ⑫ 不判，stderr 报一句）
    sys.path.insert(0, sys.argv[1]); import lib_write_scope as write_scope
    write_scope_error = None
except Exception as error:   # 文件不在、语法错都算读不到
    write_scope, write_scope_error = None, error

# scripts/hosts/two-logs.sh [--quiet] <名> -- <命令…>（日志出两份的包装）：wrapped_commands 把它剥掉，`--` 之后那条命令照原判法判（等待循环、放出追踪、终止进程、覆盖产物、先编后换、⑫ 写范围……）。
#   认法与剥法只有 lib_project_code_runs.py 的 command_under_two_logs 一份（项目代码闸与范围声明闸剥它用的也是这一个），按文件路径导入，第一次用到才装（它有几千行，没有 two-logs.sh 的命令不必付这个钱）；
#   装不上时 two-logs.sh 当不透明的命令（里面那条看不见），stderr 报一句
TWO_LOGS_SCRIPT_NAME = "two-logs.sh"
PROJECT_CODE_RUNS_LIBRARY = []   # 空表是还没装；装过的放一个模块，装不上放 None

def project_code_runs_library():
    if not PROJECT_CODE_RUNS_LIBRARY:
        try:
            specification = importlib.util.spec_from_file_location("lib_project_code_runs", os.path.join(sys.argv[1], "lib_project_code_runs.py"))
            module = importlib.util.module_from_spec(specification)
            specification.loader.exec_module(module)
            PROJECT_CODE_RUNS_LIBRARY.append(module)
        except Exception as error:   # 文件不在、语法错、它导入 lib_shell_words 失败
            not_judged(f"bash-command-detector.sh 读不到 lib_project_code_runs.py（{error!r}），scripts/hosts/two-logs.sh 包着的命令这一次没剥开判")
            PROJECT_CODE_RUNS_LIBRARY.append(None)
    return PROJECT_CODE_RUNS_LIBRARY[0]

def two_logs_inner_words(words):
    """words[0] 是 scripts/hosts/two-logs.sh 而参数是 `[--quiet] <名> -- <命令…>` 时交回 <命令…> 的词，不是交 None。"""
    library = project_code_runs_library()
    inner = library.command_under_two_logs(words) if library is not None else None
    return inner[0] if inner else None

# 弄坏开关 BASH_COMMAND_DETECTOR_BREAK=<项>（几个用逗号连；只给自检证明点名的几格会红，项名与红的格写在文件头）
DETECTOR_BREAK_VARIABLE = "BASH_COMMAND_DETECTOR_BREAK"
KNOWN_DETECTOR_BREAK_ITEMS = ("disable-check", "disable-self-background", "keep-heredoc-bodies", "allow-watchdog-misuse",
                              "allow-foreground-wait-loop", "allow-subagent-background", "allow-detaching", "allow-unwaited-ampersand",
                              "allow-background-task-output-wait", "allow-repository-in-place-edit", "allow-compile-first-bypass", "allow-results-overwrite",
                              "allow-process-signals", "ignore-private-config", "allow-script-in-place-write", "keep-selftest-scratch",
                              "refuse-every-wait-loop", "stop-at-first-refusal", "detection-outcome-dropped",
                              "allow-work-root-removal", "always-read-process-table", "per-path-git-query",
                              "opaque-two-logs", "allow-bash-scope-write", "scope-known-scripts-unseen", "scope-main-agent-judged", "scope-undeterminable-only-noted",
                              "scope-loop-values-unknown", "scope-loop-landings-trusted", "scope-home-unexpanded", "scope-landing-lexical-only",
                              "heredoc-body-judged-as-commands", "sed-in-place-ignores-cd", "heredoc-pipe-fed-as-data",
                              "pwd-variable-unexpanded", "mktemp-directory-unknown", "mktemp-scratch-template-unknown", "mktemp-template-owner-unchecked",
                              "mktemp-option-directory-unknown", "mktemp-nested-substitution-unknown", "break-honored-outside-selftest", "pending-outside-scanned-as-stale",
                              "failure-unrecorded", "allow-subagent-claude-launch", "subagent-timeout-unnoted", "python-move-into-repository-unseen",
                              "scope-script-variable-unexpanded", "builtin-agents-unjudged",
                              "allow-whole-index-reset", "whole-index-staged-unchecked", "whole-index-pathspec-ignored", "break-honored-outside-selftest-with-others", "allow-ledger-append",
                              "allow-path-discard", "path-discard-changes-unchecked", "path-discard-staged-counted", "scope-ledger-append-unseen", "scope-record-append-unseen", "scope-record-append-root-as-target",
                              "python-relative-path-ignores-cd", "compile-first-ignores-cwd",
                              "claude-launch-layers-narrow", "watchdog-tee-null-unseen", "rsync-inplace-unseen", "partial-wait-allowed", "wait-forms-unseen",
                              "whole-index-other-forms-unseen", "git-subcommand-unexpanded-allowed", "brace-target-reason-generic", "wait-loop-rewrite-unexplained",
                              "coproc-prefix-unseen", "bash-layer-unrecorded", "startup-file-claude-unread", "python-missing-unrecorded",
                              "script-body-claude-unread", "embedded-launch-claude-unread",
                              "background-ampersand-by-regex", "bare-setsid-unrecorded", "scope-launcher-writes-unseen",
                              "sed-unknown-target-resolved-from-cwd", "watchdog-null-redirect-whole-command",
                              "in-place-script-session-temporary-root-unseen", "builtin-coproc-detached", "compile-first-capped-hint",
                              "allow-literal-notations", "literal-notations-outside-judged",
                              "allow-record-append", "record-append-python-unseen", "record-append-tee-unseen",
                              "record-howto-unlisted-tool", "howto-guard-environment-keeps-breaks",
                              "own-session-group-unrecognized", "own-session-group-session-unchecked", "own-session-group-first-binding-only",
                              "allow-manual-repository-copy", "manual-copy-howto-unlisted-tool")

def detector_break_is_set(item):
    return item in os.environ.get(DETECTOR_BREAK_VARIABLE, "").split(",")

# 弄坏开关只在自证里认：自证起的子进程靠这个变量指到自证建的标记文件（内容是 SELFTEST_MARK_CONTENT）才认，门禁 code-tooling 的 break-switch-outside-selftest 格逐份探这一条
SELFTEST_MARK_VARIABLE = "BASH_COMMAND_DETECTOR_SELFTEST_MARK"
SELFTEST_MARK_CONTENT = "bash-command-detector-selftest"

# 关守卫项（records/2026-10-23-subagent拆分提案.md 第一百二十二节 ②）：只在它是唯一一项时认；与别的项同写照拒并点名它（带标记的自证子进程里也拒），
#   免得「关守卫项,别的项」平常跑借道把别的项也认了。GUARD_OFF_WITH_OTHERS_ITEM 是它的弄坏开关：带标记的自证子进程里同写着它时，同写的关守卫项照认
GUARD_OFF_ITEM = "break-honored-outside-selftest"
GUARD_OFF_WITH_OTHERS_ITEM = "break-honored-outside-selftest-with-others"

def refuse_break_outside_selftest():
    """弄坏开关只在自证里认：平常跑（不是 --selftest，也不是自证起的子进程）设了 BASH_COMMAND_DETECTOR_BREAK 就退 2，stderr 点名变量只在自证里认；没设、--selftest、
    标记文件在而内容对得上的放行（返回 0）。关守卫项 break-honored-outside-selftest 单独一项时不拒；与别的项同写一律拒、点名它
    （带标记的子进程里同写着 break-honored-outside-selftest-with-others 的除外）。"""
    value = os.environ.get(DETECTOR_BREAK_VARIABLE, "")
    if not value or (len(sys.argv) > 2 and sys.argv[2] == "--selftest"):
        return 0
    items = [item for item in value.split(",") if item]
    mark = os.environ.get(SELFTEST_MARK_VARIABLE, "")
    try:
        marked = bool(mark) and open(mark, encoding="utf-8").read().strip() == SELFTEST_MARK_CONTENT
    except OSError:
        marked = False
    others = [item for item in items if item != GUARD_OFF_ITEM]
    if GUARD_OFF_ITEM in items and others and not (marked and GUARD_OFF_WITH_OTHERS_ITEM in others):
        print(f"  ✗ {DETECTOR_BREAK_VARIABLE}={value}：关守卫项 {GUARD_OFF_ITEM} 只在它是唯一一项时认，与别的项（{'、'.join(others)}）同写照拒；这一次没判", file=sys.stderr)
        print(f"     → 怎么办：unset {DETECTOR_BREAK_VARIABLE} 再来；要看某一项红在哪一格，跑 {DETECTOR_BREAK_VARIABLE}=<项> bash .claude/hooks/bash-command-detector.sh --selftest，一次只设那一项", file=sys.stderr)
        return 2
    if marked or items == [GUARD_OFF_ITEM]:
        return 0
    print(f"  ✗ {DETECTOR_BREAK_VARIABLE}={value} 是弄坏开关，只在自证（bash-command-detector.sh --selftest）里认；平常跑设了就拒，这一次没判", file=sys.stderr)
    print(f"     → 怎么办：unset {DETECTOR_BREAK_VARIABLE} 再来；要看开关红在哪一格，跑 bash .claude/hooks/bash-command-detector.sh --selftest（开关写在文件头）", file=sys.stderr)
    return 2
# 剥掉之后里面那条接着判的启动器：lib_project_code_runs.py 的 LAUNCHER_OPTIONS_WITH_VALUE 的键（/usr/bin/time、flock、systemd-run……，剥法也在那一份：command_under_launcher）。
#   命令词是这几个才去装那份几千行的库；两边的名字集合不一致时自检「启动器名单与 lib_project_code_runs 的一致」那一格红
LAUNCHER_NAMES = frozenset({"time", "flock", "chrt", "prlimit", "systemd-run", "strace", "perf", "rustup",
                            # 名单外启动器（lib_project_code_runs.py 的 EXTRA_LAUNCHER_NAMES；一百三十七节 ②）
                            "doas", "numactl", "unshare", "setpriv", "nsenter", "ltrace", "valgrind", "faketime", "unbuffer", "cpulimit",
                            "dbus-run-session", "firejail", "proot", "chroot", "cgexec", "xargs", "find", "ssh", "su", "runuser", "sg", "script",
                            "watch", "gdb", "scl", "parallel", "bwrap"})
def wrapped_commands(words):
    """一条简单命令一层层剥开前缀、capped.sh N、`bash 包装脚本`（bash 起的是脚本时接着往里剥）与 `two-logs.sh … --`：
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
        if name == TWO_LOGS_SCRIPT_NAME and not detector_break_is_set("opaque-two-logs"):
            inner = two_logs_inner_words(remaining)
            if inner:
                remaining = inner
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
        head_commands = shell_words.simple_commands(shell_words.shell_tokens(shell_words.open_substitution_tail(head)))[0]   # 挂在没收尾的 $( / <( 里的，在那一层里找挂它的命令
        feeds_under_timeout = bool(head_commands) and shell_code_and_timeout(head_commands[-1])[1] and honor_timeout
        layers += shell_layers(body, under_timeout or feeds_under_timeout, honor_timeout, depth + 1)
    for words, command_substitutions in zip(commands, substitutions):
        for substitution in command_substitutions:
            layers += shell_layers(substitution, under_timeout, honor_timeout, depth + 1)
        code, has_timeout = shell_code_and_timeout(words) if words else (None, False)
        if code is not None:
            layers += shell_layers(code, under_timeout or (has_timeout and honor_timeout), honor_timeout, depth + 1)
    return layers

def wait_loops_in(commands, loop_filter=None):
    """一层里的 until / while 循环，条件或循环体里有 sleep 的（给了 loop_filter 的还要它对整个循环的简单命令判真），交回每个循环开头那条简单命令。"""
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
        if any(is_sleep_command(words) for words in commands[index:end] if words) and (loop_filter is None or loop_filter(commands[index:end])):
            found.append(" ".join(commands[index])[:120])
            index = end
        else:
            index += 1
    return found

def unbounded_wait_loops(text, honor_timeout=True):
    """命令位置上的 until / while 循环里（条件或循环体）有 sleep、外面又没有 timeout 的，交回每个循环开头那条简单命令。"""
    return [loop for layer in shell_layers(text, honor_timeout=honor_timeout) if not layer.under_timeout for loop in wait_loops_in(layer.commands)]

# 把活放出追踪的写法（agent-common.md「长活交回主 agent」那一条列的）：命令词是这几个的
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

def detaching_prefixes(prefix_words, has_lone_ampersand, prefix_names):
    """命令词前面那段前缀里的 `setsid -f`、（这一层有单独的 & 时的）`nohup` 与共用模块当关键字跳过的 `coproc`（prefix_names 是共用模块记下的路过的前缀名）。"""
    found = []
    for index, word in enumerate(prefix_words):
        name = os.path.basename(word)
        if name == "coproc" and not detector_break_is_set("coproc-prefix-unseen"):
            # 共用模块把 coproc 当关键字跳过时把它记进路过的前缀名，命令词前面那段里认它；`builtin coproc …`、`builtin -- coproc …`
            # 里的 coproc 不是内建，bash 报 not a shell builtin、什么都不跑，共用模块不记它，这里也不认（复跑分类报告 d137-replay-classify 6.5 节；
            # 弄坏开关 builtin-coproc-detached 下照词认）
            if "coproc" in prefix_names or detector_break_is_set("builtin-coproc-detached"):
                found.append("coproc")
            continue
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
            for prefix_words, prefix_names, name, arguments in wrapped_commands(words):
                found += detaching_prefixes(prefix_words, layer.has_lone_ampersand, prefix_names)
                form = detaching_command(name, arguments) if name else None
                if form:
                    found.append(form)
    return list(dict.fromkeys(found))

def detaching_refusal(command):
    """命令位置上有把活放出追踪的写法就交回认出的写法；否则交 []。前台、run_in_background 一样拒。"""
    if (detector_break_is_set("disable-check")
            or detector_break_is_set("allow-detaching")):
        return []
    return detaching_forms(command)

# ⑧ 有 Edit 工具的项目子 agent 在 Bash 里就地改仓内文件：命令位置上的 `sed -i`（目标不在 /tmp/ 下、也不在工作根下），或 python 代码里 open(<仓内路径字面量>, 'w' / 'a' / 'x')、
#   Path(<仓内路径字面量>).write_text( / .write_bytes(，os.replace / os.rename / shutil.move 的目标（第二个参数或 dst=）是仓内路径字面量的。
#   仓内路径字面量：crates/、scripts/、.claude/、research/、records/、litmus/、briefs/ 起头的，或仓根起的绝对路径。
#   认不出：路径放在变量里的、经脚本文件间接写的（replace-once.py、insert-row.py 这类定义点名的脚本本来就放行）。
#   弄坏开关 python-move-into-repository-unseen 下前缀不含 scripts/，三种挪法不认（PYTHON_REPOSITORY_WRITE_BEFORE_MOVES）。
REPOSITORY_PATH_PREFIX = r"(?:\./)?(?:crates|scripts|\.claude|research|records|litmus|briefs)/"
REPOSITORY_PATH_PREFIX_BEFORE_MOVES = r"(?:\./)?(?:crates|\.claude|research|records|litmus|briefs)/"
# 挪法的第一个参数：一个字符串字面量，或不带逗号、括号与引号的一个表达式（名字、属性）
PYTHON_MOVE_SOURCE = r"""(?:f?'[^']*'|f?"[^"]*"|[^,()'"]+)"""

def python_repository_write_pattern(prefix, with_moves):
    """⑧ python 一半的正则（{root} 处由调用方换成仓根）：open(…, 'w' / 'a' / 'x')、Path(…).write_text / write_bytes，with_moves 时另认 os.replace / os.rename / shutil.move 的目标。"""
    alternatives = [r"""open\(\s*f?['"](?P<open>(?:{root})?""" + prefix + r"""[^'"]*)['"]\s*,\s*['"][wax]b?\+?['"]""",
                    r"""Path\(\s*f?['"](?P<path>(?:{root})?""" + prefix + r"""[^'"]*)['"]\s*\)\s*\.write_(?:text|bytes)\("""]
    if with_moves:
        alternatives.append(r"""(?:\bos\.(?:replace|rename)|\bshutil\.move)\(\s*""" + PYTHON_MOVE_SOURCE + r"""\s*,\s*(?:dst\s*=\s*)?f?['"](?P<move>(?:{root})?"""
                            + prefix + r"""[^'"]*)['"]""")
    return re.compile("|".join(alternatives))

PYTHON_REPOSITORY_WRITE = python_repository_write_pattern(REPOSITORY_PATH_PREFIX, True)
PYTHON_REPOSITORY_WRITE_BEFORE_MOVES = python_repository_write_pattern(REPOSITORY_PATH_PREFIX_BEFORE_MOVES, False)

def agent_has_edit_tool(repository_root, agent_type):
    try:
        head = open(os.path.join(repository_root, ".claude", "agents", f"{agent_type}.md"), encoding="utf-8").read().split("\n---", 1)[0]
    except OSError:
        return False
    tools_line = next((line for line in head.splitlines() if line.startswith("tools:")), "")
    return "Edit" in {tool.strip() for tool in tools_line[len("tools:"):].split(",")}

def sed_in_place_targets(arguments):
    """sed 的参数里带 -i / --in-place 时交回它要改的文件（不在 /tmp/ 下、也不在工作根下的；工作根认绝对路径与 ~/…、$HOME/… 写法）；不带交 []。"""
    if not any(argument == "-i" or re.match(r"^-[a-zA-Z]*i", argument) or argument.startswith("--in-place") for argument in arguments):
        return []
    has_script_option = any(argument in ("-e", "-f") or argument.startswith(("--expression", "--file")) for argument in arguments)
    operands = [argument for argument in arguments if not argument.startswith("-")]
    files = operands if has_script_option else operands[1:]
    return [path for path in files if not path.startswith("/tmp/")
            and not (work_roots and path.startswith(("/", "~", "$")) and written_path_is_absolute(path) and work_roots.is_under_scratch_root(path))]

def written_path_is_absolute(path):
    """sed -i 的目标照文字展开（$SINGLEFS_WORK_ROOT、$HOME、~）之后是绝对路径：展开不了的（`$F`、xargs 的占位词）不拿钩子进程自己的当前目录去补成绝对路径
    （否则钩子进程的当前目录在工作根下时，`sed -i … $F` 被当成工作根下的草稿、从目标里丢掉）。弄坏开关 sed-unknown-target-resolved-from-cwd 下一律补成绝对路径。"""
    if detector_break_is_set("sed-unknown-target-resolved-from-cwd"):
        return True
    return os.path.isabs(work_roots.expand_written_path(path))

def embedded_python_codes(command, repository_root):
    """命令里真要被 python 跑的代码：`python3 -c` 的那一段，和喂给 python 的 heredoc 正文（含 bash -c、命令替换、喂给 shell 的 heredoc 里的那几层）。
    喂给 cat、tee 这类别的命令的 heredoc 正文是数据，引号里当数据写的文字也不是代码，都不算（往草稿目录写一份脚本、笔记，正文里出现 open(…, 'w') 不是在写）。"""
    codes = [step.code for step in overwrite_steps(command, repository_root, every_write=False) if isinstance(step, PythonCode)]
    for layer in shell_layers(command):
        for words in layer.commands:
            codes += [code for kind, code in embedded_code_strings(words) if kind == "python"]
    return list(dict.fromkeys(codes))

def sed_target_resolved_in_scratch(step, repository_root):
    """sed -i 的目标是相对路径、前面 cd 到了算得出的目录时，从那个目录起算：落在 /tmp/ 或工作根下、而不在本仓里的，不是仓内文件。
    cd 的目录算不出（step.directory 是 None）、目标本身带变量或通配、本来就是绝对路径的，这里不管：照旧按仓内判（绝对路径的草稿目录在 sed_in_place_targets 里已经放行）。"""
    if step.directory is None or re.search(r"[$`*?\[{]", step.target) or os.path.isabs(os.path.expanduser(step.target)):
        return False
    absolute = shell_words.resolve_path(step.directory, step.target)
    if any(candidate == root or candidate.startswith(root + os.sep)
           for candidate in (absolute, os.path.realpath(absolute)) for root in (os.path.normpath(repository_root), os.path.realpath(repository_root))):
        return False
    return absolute.startswith("/tmp/") or bool(work_roots and work_roots.is_under_scratch_root(absolute))

def python_code_directories(command, start_directory):
    """⑧ python 一半：每段喂给 python 的代码 → 跑它那一刻的当前目录（几处都跑的列全；cd 的目录算不出的那一处是 None）。取 overwrite_steps 的 PythonCode，与 sed -i 一半同一处的 cd 跟踪。"""
    directories = {}
    for step in overwrite_steps(command, start_directory, every_write=False):
        if isinstance(step, PythonCode):
            directories.setdefault(step.code, []).append(step.directory)
    return directories

def python_target_outside_repository(target, directories, repository_root):
    """⑧ python 一半的目标是相对路径、每一处跑它的当前目录都算得出、按那里解开都落在本仓之外时为真（cd 进草稿目录里的仓副本再写 crates/…、scripts/…）。
    仓根起的绝对路径、当前目录算不出或不知道（directories 是 None 或空）、有一处落在仓里的都为假，照旧按仓内判。"""
    if os.path.isabs(target) or not directories or None in directories:
        return False
    roots = (os.path.normpath(repository_root), os.path.realpath(repository_root))
    for directory in directories:
        absolute = shell_words.resolve_path(directory, target)
        if absolute is None or any(candidate == root or candidate.startswith(root + os.sep)
                                   for candidate in (absolute, os.path.realpath(absolute)) for root in roots):
            return False
    return True

def repository_in_place_edit_refusal(command, agent_type, repository_root, start_directory=None):
    """start_directory 是命令开头的当前目录（hook 输入里的 cwd，没有就是仓库根）；sed -i 的相对路径从 cd 之后的目录起算（跟 ⑫ 同一处的 cd 跟踪，overwrite_steps）。"""
    if (not agent_type or detector_break_is_set("disable-check")
            or detector_break_is_set("allow-repository-in-place-edit")
            or not agent_has_edit_tool(repository_root, agent_type)):
        return []
    found = []
    if detector_break_is_set("sed-in-place-ignores-cd"):   # 弄坏开关：相对路径不跟 cd
        for layer in shell_layers(command):
            for words in layer.commands:
                for _, _, name, arguments in wrapped_commands(words):
                    if name == "sed":
                        found += [f"sed -i {path}" for path in sed_in_place_targets(arguments)]
    else:
        found += [f"sed -i {step.target}" for step in overwrite_steps(command, start_directory or repository_root, every_write=True)
                  if isinstance(step, OverwriteCandidate) and step.form == "sed -i" and not sed_target_resolved_in_scratch(step, repository_root)]
    template = PYTHON_REPOSITORY_WRITE_BEFORE_MOVES if detector_break_is_set("python-move-into-repository-unseen") else PYTHON_REPOSITORY_WRITE
    pattern = re.compile(template.pattern.replace("{root}", re.escape(repository_root.rstrip("/") + "/")))
    # 相对路径从跑那段代码时的当前目录起算（弄坏开关 python-relative-path-ignores-cd 下不跟 cd，一律按仓内判）
    code_directories = {} if detector_break_is_set("python-relative-path-ignores-cd") else python_code_directories(command, start_directory or repository_root)
    for code in (embedded_python_codes(command, repository_root) if not detector_break_is_set("heredoc-body-judged-as-commands") else [command]):
        for match in pattern.finditer(code):
            moved = match.groupdict().get("move")
            target = moved or match.group('open') or match.group('path')
            if python_target_outside_repository(target, code_directories.get(code), repository_root):
                continue
            found.append(f"python 挪进 {moved}" if moved else f"python 写 {target}")
    return list(dict.fromkeys(found))

# ⑨ 先编后换：experiment-runner 在 Bash 里写主工作区 crates/ 下的 .rs。写法沿用 ⑤ ⑦ 的 overwrite_steps（every_write=True），
#   mv 挪走、rm 删掉的源也算改。agent 名单与路径判据与 write-guard.sh「四、先编后换」各写一份、改的时候一起改。
COMPILE_FIRST_AGENTS = ("experiment-runner",)
COMPILE_THEN_SWAP_SCRIPT = "scripts/hosts/compile-then-swap.py"

def is_main_workspace_crate_source(path, repository_root):
    return path.endswith(".rs") and is_inside(path, os.path.join(repository_root, "crates"))

def compile_first_verdict(command, repository_root, start_directory=None):
    """整条命令里写主工作区 crates/ 下 .rs 的每一步，交回 `写法 相对仓库根的路径`。
    相对路径从 cd 之后的目录起算，起点是 start_directory（hook 输入的 cwd，同 ⑧ ⑫；不给是仓库根）；cd 算不出之后的照仓库根解。
    弄坏开关 BASH_COMMAND_DETECTOR_BREAK=compile-first-ignores-cwd 下起点一律是仓库根。"""
    crates_directory = os.path.join(repository_root, "crates")
    refused = []
    if start_directory is None or detector_break_is_set("compile-first-ignores-cwd"):
        start_directory = repository_root
    for step in overwrite_steps(command, start_directory, every_write=True):
        base = step.directory if step.directory is not None else repository_root
        targets = []
        if isinstance(step, PreservingStep):
            if step.kind in ("mv", "rm"):
                for path in preserved_paths(step, base):
                    if is_main_workspace_crate_source(path, repository_root) or (
                            os.path.isdir(path) and (os.path.normpath(path) == crates_directory or is_inside(path, crates_directory))):
                        refused.append(f"{step.kind} {os.path.relpath(path, repository_root)}")
            continue
        if isinstance(step, PythonCode):
            for write in python_in_place_writes(step.code, every_write=True):
                path = shell_words.resolve_path(base, write.path)
                if write.source is not None and os.path.isdir(path):
                    path = os.path.join(path, os.path.basename(write.source))
                targets.append((write.form, path))
        else:
            for word in destination_words(step, base):
                pattern, has_unknown_piece, has_glob = target_pattern(os.path.expanduser(word))
                path = shell_words.resolve_path(base, pattern)
                if has_unknown_piece or has_glob:
                    # `"$变量"/crates/…` 这类变量在前的，另从 crates/ 那一段起按仓库根接（与 ⑤ 认 research/results/ 同一个办法）
                    anchor = pattern.find("crates/")
                    patterns = [path] + ([os.path.join(repository_root, pattern[anchor:])] if anchor > 0 and "*" in pattern[:anchor] else [])
                    targets += [(step.form, match) for candidate in patterns for match in sorted(glob.glob(candidate))]
                else:
                    targets.append((step.form, path))
        for form, path in targets:
            if is_main_workspace_crate_source(path, repository_root):
                refused.append(f"{form} {os.path.relpath(path, repository_root)}")
    return list(dict.fromkeys(refused))

def compile_first_refusal(command, agent_type, repository_root, start_directory=None):
    """交回要拒的写法与目标；只判 COMPILE_FIRST_AGENTS 里的 agent，前台、run_in_background 一样判。start_directory 是 hook 输入的 cwd（compile_first_verdict）。"""
    if (agent_type not in COMPILE_FIRST_AGENTS or detector_break_is_set("disable-check")
            or detector_break_is_set("allow-compile-first-bypass")):
        return []
    return compile_first_verdict(command, repository_root, start_directory)

# ⑫ 有写范围的项目子 agent 在 Bash 里写自己写范围之外的仓内文件。写范围闸 write-guard.sh 只看 Write / Edit（它文件头「拦不住 Bash 里的写」），
#   子 agent 在 Bash 里跑 scripts/kb/replace-once.py 就写进了定义不让它写的知识库目录（records/2026-09-16-subagent拆分提案.md 第八十一节 ④）。这一道照
#   同目录 agent-write-scope.tsv 那张表判（读法与判法在 lib_write_scope.py，write-guard.sh 用的是同一份）：该 agent 的行里没有一条模式落到这个文件，就拒。
#   认的写法：⑨ 认的那一批（overwrite_steps 的 every_write：>、>>、&>、tee（带不带 -a）、cp / mv / install 的目标、dd of=、truncate、sed -i、perl -i、rsync 的目标，
#   python 代码里 open(…, 带 w / a / x / + 的模式)、Path.write_text / write_bytes、shutil.copy*、os.replace / rename），加上 KNOWN_WRITE_SCRIPTS 五个定点写脚本的目标文件：
#   replace-once.py 的 文件、insert-row.py 的 文件（带 --dry-run 的不写，不算）、replace-batch.py 规格 JSON 里每一项的 file（规格读不到或不是数组的拒；带 --dry-run 的不算）、
#   ledger-append-section.py 写的登记表 current 那一份账本（LedgerAppendTarget，仓根的认法见文件头 ⑫）、append-record.py 的 <记录>（第一个不是选项的参数，--root 的值不算）。
#   落点算得出而在仓外的（/tmp、/dev/null、工作根下的草稿目录）不判。没有写范围的不判：主 agent（输入里没有 agent_type）、没有项目定义的内置 agent、
#   表里没有它那几行的定义（表只登记有 Write / Edit 的定义；这样的 agent 在 Bash 里写报告文件，范围写在它们定义的「写范围」里，机器没有表可比）。
#   落点这一刻算不出的（没赋过值或来源不明的变量、命令替换、通配、cd 算不出之后的相对路径）一律拒，静态前缀在仓外也拒；拒绝说明指到 scripts/agents/copy-originals.py 与写死的词的 for 循环。
#   算得出的照算（前面赋的写死值、for 写死的词逐个取值、cd 之后的 $PWD、mktemp -d、$HOME；落点规整 .. 并逐段解开符号链接），判法见文件头 ⑫。
#   heredoc 正文：喂给 shell 的照命令判，喂给 python 的照 python 写文件判，喂给 cat、tee 这类的是数据不判（它们的目标照判）。mv 挪走的源与 rm 删掉的不判（只判写进去的）。cd 跟着算，相对路径从 hook 输入的 cwd 起算。
#   前台、run_in_background 一样拒。判不到的：变量里拼出来的命令、eval、`bash x.sh` 起的脚本文件里的写法、patch 与 git apply、经 subprocess 起的写法、xargs / find -exec。
KNOWN_WRITE_SCRIPTS = ("replace-once.py", "replace-batch.py", "insert-row.py", "ledger-append-section.py", "append-record.py")   # scripts/kb/ 下定点写文件的五个脚本
LEDGER_APPEND_SCRIPT = "ledger-append-section.py"   # 不带目标文件参数：写的是仓根下登记表 current 那一份账本（LedgerAppendTarget）
WRITE_SCOPE_TABLE_NAME = "agent-write-scope.tsv"                                  # 与 write-guard.sh 同一张表，在这个 hook 的同目录
INSERT_ROW_OPTIONS_WITH_VALUE = {"--after", "--before", "--after-literal", "--before-literal", "--line", "--absent"}
NON_WRITING_FLAGS = {"--selftest", "--dry-run", "-h", "--help"}
RECORD_APPEND_SCRIPT = "append-record.py"         # 写的是 <记录>：第一个不是选项的参数（往 records/ 下的 .md 追加，⑲ 的出路）
RECORD_APPEND_OPTIONS_WITH_VALUE = {"--root"}     # append-record.py 带值的选项：它的值是仓根，不是记录

class UnknownScriptTarget(NamedTuple):
    """定点写脚本要改哪个文件这一刻读不出来（⑫ 只记检出）。"""
    form: str
    reason: str

def first_operand(arguments, options_with_value=()):
    """参数里第一个不是选项的词（options_with_value 里的选项连它的值一起跳过）；没有交 None。"""
    position = 0
    while position < len(arguments):
        argument = arguments[position]
        if argument in options_with_value:
            position += 2
        elif argument.startswith("-") and argument != "-":
            position += 1
        else:
            return argument
    return None

class UnresolvedScriptTarget(NamedTuple):
    """python3 起的脚本词代入前面赋过的值之后还有算不出的段（认不出起的是不是定点写脚本），第一个不是选项的参数是 target：
    bash_scope_verdict 算它的落点，落在本仓里的照「落点算不出」拒。"""
    form: str
    target: str
    directory: str | None

def unresolved_script_word(word):
    """脚本词代入之后还有算不出的段：没赋过值或来源不明的变量、命令替换、for 循环变量（代入时包着 LOOP_VALUE_MARK）。"""
    return "$" in word or "`" in word or LOOP_VALUE_MARK in word

class LedgerAppendTarget(NamedTuple):
    """ledger-append-section.py 要写的账本（⑫）：它不带目标文件参数，写的是仓根下登记表 current 那一份；仓根照它自己的认法取（ledger_append_root）。"""
    form: str
    root: str | None        # --root 给的（前面赋过的变量已代入）；没给是 None
    script: str | None      # 没给 --root 时 python3 起的脚本路径（变量已代入）；直接执行只剩脚本名，是 None
    directory: str | None   # 那一刻的当前目录；cd 算不出来是 None

def last_option_value(arguments, option):
    """参数里 option 的值：`--root X` 与 `--root=X` 两种写法都认，写了几次取最后一次（同 argparse）；没写交 None。"""
    value = None
    for position, argument in enumerate(arguments):
        if argument == option and position + 1 < len(arguments):
            value = arguments[position + 1]
        elif argument.startswith(option + "="):
            value = argument[len(option) + 1:]
    return value

def ledger_append_root(step, repository_root):
    """LedgerAppendTarget 要写的那个仓的根 → (根, None)；算不出 → (None, 原因)。--root 给的照它（取 realpath，同脚本）；没给的是脚本路径解开符号链接之后往上三层
    （脚本自己按 realpath(__file__) 的 scripts/kb/ 上两层认仓根）；直接执行只剩脚本名的按 repository_root（判的这个仓）。"""
    word = step.root if step.root is not None else step.script
    if word is None:
        return repository_root, None
    shown = f"--root {word}" if step.root is not None else f"脚本路径 {word}"
    pattern, has_unknown_piece, has_glob = target_pattern(os.path.expanduser(home_expanded(word)))
    if has_unknown_piece or has_glob:
        return None, f"{shown} 里有这一刻算不出的段（没赋过值或来源不明的变量、命令替换、通配）"
    if step.directory is None and not os.path.isabs(pattern):
        return None, f"前面有 cd 到这一刻算不出的目录，{shown} 落在哪算不出"
    path = os.path.realpath(pattern if os.path.isabs(pattern) else os.path.join(step.directory, pattern))
    return (path if step.root is not None else os.path.dirname(os.path.dirname(os.path.dirname(path)))), None

def known_write_script_steps(name, arguments, directory, variables):
    """命令是 KNOWN_WRITE_SCRIPTS 里的定点写脚本（python3 起它的，或直接执行它的）时，它要写的文件各交一个 OverwriteCandidate；读不出要写哪个文件的交 UnknownScriptTarget。
    name 是命令词的名字（basename；直接执行 `$S …` 时就是 `$S`，代入之后再取名字）。脚本词先按 variables（这条命令里前面赋过的值）代入再取名字比；
    python3 起的脚本词代入之后还有算不出的段、第一个不是选项的参数在、又不带不写的参数的，交 UnresolvedScriptTarget（落点由 bash_scope_verdict 判）。
    弄坏开关 scope-known-scripts-unseen 下一概交 []；scope-script-variable-unexpanded 下脚本词不代入、也不交 UnresolvedScriptTarget。"""
    if detector_break_is_set("scope-known-scripts-unseen"):
        return []
    substitute = not detector_break_is_set("scope-script-variable-unexpanded")
    launched_by_python = bool(shell_words.PYTHON_NAME.match(name))
    if launched_by_python:
        invocation = shell_words.python_invocation(arguments)
        if invocation.code_string is not None or invocation.module is not None or invocation.script_index is None or invocation.script_index >= len(arguments):
            return []
        script_word, rest = arguments[invocation.script_index], arguments[invocation.script_index + 1:]
    else:
        script_word, rest = name, arguments
    expanded_word = substitute_assigned_variables(script_word, variables) if substitute else script_word
    script = os.path.basename(expanded_word)
    if script not in KNOWN_WRITE_SCRIPTS:
        if substitute and launched_by_python and unresolved_script_word(home_expanded(expanded_word)):
            rest = [substitute_assigned_variables(argument, variables) for argument in rest]
            target = first_operand(rest)
            if target and not any(argument in NON_WRITING_FLAGS for argument in rest):
                return [UnresolvedScriptTarget(f"python3 {script_word}", target, directory)]
        return []
    rest = [substitute_assigned_variables(argument, variables) for argument in rest]
    if any(argument in NON_WRITING_FLAGS for argument in rest):
        return []
    if script == LEDGER_APPEND_SCRIPT:
        # 不带目标文件参数：写的是仓根下登记表 current 那一份账本；弄坏开关 scope-ledger-append-unseen 下不认它
        if detector_break_is_set("scope-ledger-append-unseen"):
            return []
        root = last_option_value(rest, "--root")
        return [LedgerAppendTarget(script, root, expanded_word if launched_by_python and root is None else None, directory)]
    if script == RECORD_APPEND_SCRIPT:
        # 写的是 <记录>（第一个不是选项的参数，--root 的值不算）；弄坏开关 scope-record-append-unseen 下不认它，scope-record-append-root-as-target 下把 --root 的值当记录
        if detector_break_is_set("scope-record-append-unseen"):
            return []
        options_with_value = () if detector_break_is_set("scope-record-append-root-as-target") else RECORD_APPEND_OPTIONS_WITH_VALUE
        record = first_operand(rest, options_with_value)
        return [OverwriteCandidate(script, record, directory)] if record else []
    if script == "replace-batch.py":
        specification = first_operand(rest)
        path = shell_words.resolve_path(directory, os.path.expanduser(specification)) if specification else None
        try:
            with open(path, encoding="utf-8") as handle:
                files = [item["file"] for item in json.load(handle) if isinstance(item, dict) and isinstance(item.get("file"), str)]
        except (OSError, TypeError, ValueError, KeyError):
            files = []
        if not files:
            return [UnknownScriptTarget(script, f"{script} 的规格 {specification or '（没给）'} 这一刻读不出要改哪些文件（不存在、不是 JSON 数组或没有 file）")]
        return [OverwriteCandidate(script, file, directory) for file in files]
    target = first_operand(rest, INSERT_ROW_OPTIONS_WITH_VALUE if script == "insert-row.py" else ())
    return [OverwriteCandidate(script, target, directory)] if target else []

# 没有项目定义的 agent（内置类型、插件带来的、定义已删的旧实例）的写范围：只有工作根（文件头 ⑫；write-guard.sh 第二道用同一句模式）
BUILTIN_AGENT_SCOPE = ("$SINGLEFS_WORK_ROOT/**",)
DEVICE_PREFIXES = ("/dev/", "/proc/")   # 没有项目定义的 agent 写仓外时不判的：设备与文件描述符（2>/dev/null、> /dev/stderr）

def agent_has_definition(repository_root, agent_type):
    return os.path.isfile(os.path.join(repository_root, ".claude", "agents", f"{agent_type}.md"))

def agent_scope_patterns(repository_root, agent_type, table_path):
    """项目定义的 agent 在写范围表里的模式（表里没有它的行交 []）；没有项目定义的交 BUILTIN_AGENT_SCOPE（弄坏开关 builtin-agents-unjudged 下交 []）。
    读表读不了抛 OSError。"""
    if not agent_has_definition(repository_root, agent_type):
        return [] if detector_break_is_set("builtin-agents-unjudged") else list(BUILTIN_AGENT_SCOPE)
    return write_scope.load_scopes(table_path).get(agent_type, [])

LOOP_VALUE_MARK = "\x01"                          # ⑫ 里「for 名字 in 写死的词…」的循环变量代入时写成 \x01词\x02词…\x01，判的时候逐个取值；路径里不会有这两个字符
LOOP_VALUE_SEPARATOR = "\x02"
LOOP_VALUE_PIECE = re.compile("\x01([^\x01]*)\x01")
LITERAL_LOOP_WORD = re.compile(r"[^$`*?\[\]{}~\x01\x02]+")   # 循环列表里写死的词：没有变量、命令替换、通配、花括号与 ~
LOOP_BINDING_COMMANDS = {"for", "select"}
READING_COMMANDS = {"read", "mapfile", "readarray"}  # 读进来的值在命令文本里算不出
MAXIMUM_LOOP_LANDINGS = 4096                      # 一个目标里几个循环变量的取值组合多过这么多，照算不出判
VARIABLE_NAME = re.compile(r"[A-Za-z_][A-Za-z0-9_]*")
HOME_REFERENCE = re.compile(r"\$(?:HOME\b|\{HOME\})")

class ScopeVerdict(NamedTuple):
    """⑫ 判一条命令的结局：写到写范围之外的仓内文件（要拒）、落点算不出的（要拒；弄坏开关 scope-undeterminable-only-noted 下改记进 noted）、只记检出的、该 agent 的写范围模式。"""
    out_of_scope: list
    undeterminable: list
    noted: list
    patterns: list
    builtin: bool = False   # 没有项目定义的 agent：只许写工作根，仓外别处的落点也拒

def bind_loop_variables(name, arguments, variables):
    """⑫ 跟 for / select 与 read / mapfile 这类改写变量的命令（arguments 是还没代入变量的原词）：`for 名字 in 词…` 列出的词都写死时，名字记成这几个取值
    （LOOP_VALUE_MARK 包着，之后的目标逐个取值算落点）；列表算不出的（来自变量、命令替换、通配、花括号、没写 in）与 read 这类读进来的名字从 variables 里去掉，
    之后照算不出判（前面赋过的值也作废）。弄坏开关 scope-loop-values-unknown 下写死的词也当算不出。"""
    if name in LOOP_BINDING_COMMANDS:
        if not arguments or not VARIABLE_NAME.fullmatch(arguments[0]):
            return
        listed = arguments[2:] if arguments[1:2] == ["in"] else []
        if listed and all(LITERAL_LOOP_WORD.fullmatch(word) for word in listed) and not detector_break_is_set("scope-loop-values-unknown"):
            variables[arguments[0]] = LOOP_VALUE_MARK + LOOP_VALUE_SEPARATOR.join(listed) + LOOP_VALUE_MARK
        else:
            variables.pop(arguments[0], None)
        return
    for argument in arguments:
        if VARIABLE_NAME.fullmatch(argument):
            variables.pop(argument, None)

def loop_value_choices(texts):
    """目标与源里代入的循环取值逐个组合：交回 [{包着的那一段: 取值}]；没有循环取值交 [{}]，组合多过 MAXIMUM_LOOP_LANDINGS 交 None（照算不出判）。
    弄坏开关 scope-loop-landings-trusted 下每一段只取一个写死的普通名字（取值里的 `..` 与 / 看不见）。"""
    pieces = list(dict.fromkeys(match.group(0) for text in texts for match in LOOP_VALUE_PIECE.finditer(text)))
    if detector_break_is_set("scope-loop-landings-trusted"):
        return [{piece: "plain-value" for piece in pieces}]
    choices = [{}]
    for piece in pieces:
        values = piece[1:-1].split(LOOP_VALUE_SEPARATOR)
        if len(choices) * len(values) > MAXIMUM_LOOP_LANDINGS:
            return None
        choices = [{**choice, piece: value} for choice in choices for value in values]
    return choices

def with_loop_values(text, choice):
    return LOOP_VALUE_PIECE.sub(lambda match: choice.get(match.group(0), match.group(0)), text)

def readable_loop_values(text):
    """拒绝说明里把循环取值写回 `{词,词}` 的样子。"""
    return LOOP_VALUE_PIECE.sub(lambda match: "{" + ",".join(match.group(1).split(LOOP_VALUE_SEPARATOR)) + "}", text)

def home_expanded(word):
    """目标词里没在这条命令里赋过值的 $HOME、${HOME}（赋过的已经代入）换成 hook 自己环境里的值：钩子与它判的命令是同一个用户起的。弄坏开关 scope-home-unexpanded 下不换。"""
    home = os.environ.get("HOME")
    if not home or detector_break_is_set("scope-home-unexpanded"):
        return word
    return HOME_REFERENCE.sub(lambda _: home, word)

def bash_scope_verdict(command, repository_root, start_directory, patterns, outside_repository_judged=False):
    """整条命令里每一处写，算出最终落点：落点在本仓里、又不在 patterns（写范围表里的模式）里的交进 out_of_scope（`写法 相对仓库根的路径`）；
    outside_repository_judged（没有项目定义的 agent）时落在仓外、不在 patterns 里、也不在 /dev/、/proc/ 下的同样交进 out_of_scope（`写法 绝对路径`）；
    落点这一刻算不出的（目标里有没赋过值或来源不明的变量、命令替换、通配，cd 到算不出的目录之后的相对路径，读不到规格的 replace-batch.py）交进 undeterminable。
    能算的照算：同一条命令里前面赋的写死值、`for 名字 in 写死的词` 逐个取值、cd 之后的 $PWD、mktemp -d 赋的变量、$HOME；落点先规整 `..`、解开符号链接再判。"""
    expanded = write_scope.expanded_scope_patterns(patterns, work_roots)
    refused, undeterminable = [], []
    roots = list(dict.fromkeys((os.path.normpath(repository_root), os.path.realpath(repository_root))))
    lexical_only = detector_break_is_set("scope-landing-lexical-only")   # 弄坏开关：落点只按字面规整
    def inside_repository(path):
        return next((os.path.relpath(path, root) for root in roots if path.startswith(root + os.sep)), None)
    def judge(form, raw_path):
        """与写闸同一份判法（lib_write_scope.landings_out_of_scope）：每一种落点（照 shell 逐段解开符号链接的、规整 `..` 之后再解开的）都要在范围里；
        不在范围里的落点落在本仓里的拒，落在仓外的不判。字面路径在范围里、解开之后落在仓里范围外的照拒。解不开（读不了链接）的照算不出拒。"""
        try:
            outside = write_scope.landings_out_of_scope(expanded, raw_path, repository_root, lexical_only=lexical_only)
        except OSError as error:
            undeterminable.append(f"{form} {raw_path}：解开符号链接时出错（{error!r}），落点算不出")
            return
        for landing in outside:
            shown = inside_repository(landing)
            if shown is None:
                if outside_repository_judged and not any(path.startswith(DEVICE_PREFIXES) for path in (landing, os.path.normpath(raw_path))):
                    refused.append(f"{form} {landing}")   # 没有项目定义的 agent：仓外别处同样拒
                continue   # 落在仓外：有写范围的项目 agent 不判
            literal = inside_repository(os.path.normpath(raw_path)) or raw_path
            refused.append(f"{form} {shown}" if literal == shown else f"{form} {literal}（解开符号链接之后落在 {shown}）")
    for step in overwrite_steps(command, start_directory, every_write=True, write_scripts=True):
        if isinstance(step, UnknownScriptTarget):
            undeterminable.append(step.reason)
            continue
        if isinstance(step, UnresolvedScriptTarget):
            # 脚本词算不出：第一个参数算得出、落在本仓里的（在不在写范围里都一样）照算不出拒；参数本身算不出、cd 算不出之后的相对路径、落在仓外的不判
            word = home_expanded(step.target)
            pattern, has_unknown_piece, has_glob = target_pattern(os.path.expanduser(word))
            if has_unknown_piece or has_glob or (step.directory is None and not os.path.isabs(pattern)):
                continue
            path = os.path.normpath(pattern if os.path.isabs(pattern) else os.path.join(step.directory, pattern))
            if inside_repository(path) is not None or inside_repository(os.path.realpath(path)) is not None:
                undeterminable.append(f"{step.form} {step.target}：脚本词里有这一刻算不出的段，认不出起的是不是 replace-once.py / insert-row.py / replace-batch.py，"
                                      "第一个参数又落在本仓里")
            continue
        if isinstance(step, LedgerAppendTarget):
            # ledger-append-section.py：落点是那个仓登记表 current 那一份账本；仓根算不出的拒，登记表读不到的不判（脚本自己退 2、什么都不写）
            ledger_root, problem = ledger_append_root(step, repository_root)
            if problem:
                undeterminable.append(f"{step.form} {problem}，要写哪一份账本算不出")
                continue
            ledger, _ = current_ledger_relative(ledger_root)
            if ledger is not None:
                judge(f"{step.form}（登记表 current 那一份账本）", os.path.join(ledger_root, ledger))
            continue
        if isinstance(step, PreservingStep):
            continue   # git add、mv 挪走的源、rm：不是写进去的
        base = step.directory if step.directory is not None else repository_root
        if isinstance(step, PythonCode):
            for write in python_in_place_writes(step.code, every_write=True):
                if step.directory is None and not os.path.isabs(os.path.expanduser(write.path)):
                    undeterminable.append(f"{write.form} {write.path}：前面有 cd 到这一刻算不出的目录，相对路径落在哪算不出")
                    continue
                path = write.path if os.path.isabs(write.path) else os.path.join(base, write.path)
                if write.source is not None and os.path.isdir(path):
                    path = os.path.join(path, os.path.basename(write.source))
                judge(write.form, path)
            continue
        choices = loop_value_choices((step.target, *step.sources))
        if choices is None:
            undeterminable.append(f"{step.form} {readable_loop_values(step.target)}：循环变量的取值组合多过 {MAXIMUM_LOOP_LANDINGS} 种")
            continue
        for choice in choices:
            candidate = step._replace(target=home_expanded(with_loop_values(step.target, choice)),
                                      sources=tuple(home_expanded(with_loop_values(source, choice)) for source in step.sources))
            for word in destination_words(candidate, base):
                word = mktemp_substitutions_expanded(word, step.directory)   # 直接写在目标里、模板在自己草稿目录里的 $(mktemp -d …)
                pattern, has_unknown_piece, has_glob = target_pattern(os.path.expanduser(word))
                if (has_unknown_piece and step.form in OVERWRITING_REDIRECTS | APPENDING_REDIRECTS and BRACE_PIECE.search(word)
                        and not detector_break_is_set("brace-target-reason-generic")):
                    # 重定向目标上的花括号照拒，说明点名 bash 报 ambiguous redirect、什么都不写
                    undeterminable.append(f"{step.form} {word}：重定向目标上有花括号。引号外的 bash 展开成几个词、报 ambiguous redirect、一个字节都不写；"
                                          "引号里的（`> 'o.{a,b}'`）bash 当一个字面文件名，这道闸切词已分得出引号里外，对重定向目标上的花括号一律照算不出拒（偏严，用户 2026-10-29 定）。"
                                          "写成一个不带花括号的字面文件名；要写几份就逐份写 `> 文件1`、`> 文件2`，或用 tee 带几个文件参数")
                    continue
                if has_unknown_piece and any(placeholder in word for placeholder in launcher_input_words()):
                    undeterminable.append(f"{step.form} {word}：落点是 xargs、find -exec 或 parallel 读进来接进命令的词，这一刻算不出（认不出的一律拒）。"
                                          "改成逐个写字面路径（`sed -i … <文件1> <文件2>`）；要先找文件的，先 find / grep -l 列出来看，再逐个写字面路径")
                    continue
                if has_unknown_piece or has_glob:
                    undeterminable.append(f"{step.form} {word}：目标里有这一刻算不出的段（没赋过值或来源不明的变量、命令替换、通配）")
                    continue
                if step.directory is None and not os.path.isabs(pattern):
                    undeterminable.append(f"{step.form} {word}：前面有 cd 到这一刻算不出的目录，相对路径落在哪算不出")
                    continue
                judge(step.form, pattern if os.path.isabs(pattern) else os.path.join(base, pattern))
    return list(dict.fromkeys(refused)), list(dict.fromkeys(undeterminable))

def bash_scope_refusal(command, agent_type, repository_root, start_directory, table_path, identity=None):
    """交回 ScopeVerdict；前台、run_in_background 一样判。只判有写范围的项目子 agent（文件头 ⑫ 的「没有写范围的不判」）。
    identity：hook 输入里的 agent_id、session_id、transcript_path，认 mktemp 模板在不在这个子 agent 认领的草稿目录里用（不给就都不在）。
    弄坏开关 allow-bash-scope-write 一概放行；scope-main-agent-judged 把主 agent 也当成一条模式都没有的子 agent 判；
    scope-undeterminable-only-noted 下落点算不出的只记检出、不拒。"""
    if detector_break_is_set("disable-check") or detector_break_is_set("allow-bash-scope-write"):
        return ScopeVerdict([], [], [], [])
    if agent_type:
        patterns = agent_scope_patterns(repository_root, agent_type, table_path)
    else:
        patterns = [] if detector_break_is_set("scope-main-agent-judged") else None
    if patterns is None or (not patterns and agent_type):
        return ScopeVerdict([], [], [], [])
    if write_scope is None:
        raise RuntimeError(f"读不到写范围的判法 lib_write_scope.py（{write_scope_error!r}）")
    builtin = bool(agent_type) and not agent_has_definition(repository_root, agent_type)   # 没有项目定义的 agent：只许写工作根，仓外别处也拒
    SCRATCH_OWNER_IDENTITY.clear()
    SCRATCH_OWNER_IDENTITY.update({key: value for key, value in (identity or {}).items() if isinstance(value, str) and value})
    try:
        refused, undeterminable = bash_scope_verdict(command, repository_root, start_directory, patterns, outside_repository_judged=builtin)
    finally:
        SCRATCH_OWNER_IDENTITY.clear()
    if detector_break_is_set("scope-undeterminable-only-noted"):
        return ScopeVerdict(refused, [], [f"{item}，写范围没判，没拦" for item in undeterminable], patterns, builtin)
    return ScopeVerdict(refused, undeterminable, [], patterns, builtin)

# ⑬ 项目子 agent 在 Bash 里起 claude 会话（共用约束「派发」一节：不从 Bash 里起 claude 会话）：命令位置上的命令词名叫 claude（光名字或以 /claude 结尾的路径），
#   或是会话自己那份可执行文件的变量 $CLAUDE_CODE_EXECPATH。只判输入里有 agent_type 的，前台、run_in_background 一样拒。
CLAUDE_EXECUTABLE_NAME = "claude"
CLAUDE_EXECUTABLE_PATH_WORDS = {"$CLAUDE_CODE_EXECPATH", "${CLAUDE_CODE_EXECPATH}"}

STARTUP_FILE_READ_LIMIT = 1 << 20   # 读 BASH_ENV / ENV 指的文件最多读这么多字节，同 project-code-guard.sh 读脚本的上限
STARTUP_FILE_HEAD_BYTES = 4096      # 直接执行的脚本读开头这么多字节认 #! 行，同 project-code-guard.sh 的 SCRIPT_HEAD_BYTES

def claude_launches(command, directory=None):
    """命令里起 claude 的简单命令（claude_launches_in_text 认的），加上读得到正文的文件里起的（file_claude_launches：shell 启动时先读的文件、bash / sh / source 起的与直接执行的 shell 脚本、
    别的程序读进去当代码的 awk -f / sed -f / Makefile）；directory 是命令开头的当前目录，解相对路径用。"""
    found = claude_launches_in_text(command)
    found += file_claude_launches(command, directory)
    return list(dict.fromkeys(found))

def startup_files_of(command):
    """这一条命令起来时它的 shell 先读的文件，每个一条共用模块交的 launcher 为 "startup" 的命令：命令本身就是（BASH_ENV / ENV / --rcfile / .bashrc / zsh 的启动文件，
    shell_words.startup_file_commands 交的）；或是直接执行的脚本（路径带斜杠、字面的、读得到的）的 #! 行指到的 shell 先读的（shell_words.executed_script_startup）。"""
    if command.launcher == "startup":
        return [command]
    word = command.words[0]
    if command.launcher is not None or "/" not in word or "$" in word or "`" in word:
        return []
    resolved = shell_words.resolve_path(command.directory, os.path.expanduser(word))
    head = None
    if resolved and os.path.isfile(resolved):
        try:
            with open(resolved, "rb") as handle:
                head = handle.read(STARTUP_FILE_HEAD_BYTES)
        except OSError:
            pass
    return shell_words.executed_script_startup(command, head)[0]

def heredoc_written_files(commands):
    """同一段命令文本里 cat / tee 读 heredoc 写出的文件：真实路径 → 写出的正文（`cat > x.sh <<'EOF' … EOF` 之后紧接着 `bash x.sh`，判的那一刻文件还不在，拿写出的内容判；
    同 project-code-guard.sh 认 heredoc 写出的那一种，cp 拷来的、别的写法写出的不认）。"""
    written = {}
    for command in commands:
        if not command.standard_input_heredocs or command.launcher is not None:
            continue
        name = os.path.basename(command.words[0])
        if name == "cat":
            targets = [target for operator, target in command.output_redirections if operator in (">", ">|", ">>", "&>")][-1:]
        elif name == "tee":
            targets = [word for word in command.words[1:] if not word.startswith("-")]
        else:
            continue
        for target in targets:
            resolved = shell_words.resolve_path(command.directory, os.path.expanduser(target)) if "$" not in target and "`" not in target else None
            if resolved:
                written[os.path.realpath(resolved)] = command.standard_input_heredocs[-1].text
    return written

def readable_body(word, directory, written, executed):
    """字面的路径词（`~`、`~+` 照展开，按 directory 解）读成正文：交 (真实路径, 正文)；同一段命令文本里前面 heredoc 写出的拿写出的内容。
    executed 为真（直接执行的）只读 #! 指到 shell、或没有 #! 而扩展名是 .sh 的；带变量、命令替换的，不在的，超过 STARTUP_FILE_READ_LIMIT、有 NUL 字节、不是 UTF-8 的交 None。"""
    if "$" in word or "`" in word:
        return None
    expanded = shell_words.expand_directory_tilde(word, directory)
    resolved = shell_words.resolve_path(directory, os.path.expanduser(expanded)) if expanded is not None else None
    if resolved is None:
        return None
    real = os.path.realpath(resolved)
    if real in written:
        return real, written[real]
    if not os.path.isfile(real):
        return None
    try:
        with open(real, "rb") as handle:
            data = handle.read(STARTUP_FILE_READ_LIMIT + 1)
    except OSError:
        return None
    if len(data) > STARTUP_FILE_READ_LIMIT or b"\0" in data:
        return None
    if executed:
        shebang = shell_words.shebang_command(data[:STARTUP_FILE_HEAD_BYTES])
        if (shebang is None and not resolved.endswith(".sh")) or (shebang is not None and not shell_words.is_shell_name(shebang[0])):
            return None
    try:
        return real, data.decode("utf-8")
    except UnicodeDecodeError:
        return None

def file_claude_launches(text, directory=None, environment=None, depth=0, seen=None):
    """text 里读得到正文的文件，读进来认里面起的 claude，文件里再有这样的写法（环境里的变量照样带进去）接着读，同一份只读一遍，MAXIMUM_NESTING 层为限；路径解不出的、读不到的不判
    （子 agent 由 project-code-guard.sh 照认不出的写法拒）。读哪几种：shell 启动时先读的文件（startup_files_of；弄坏开关 startup-file-claude-unread 下不读）；
    bash / sh / source 起的脚本与直接执行的 shell 脚本（路径带斜杠的；同一段命令文本里前面 cat / tee 读 heredoc 写出的拿写出的内容；
    `bash x.sh` 起的脚本正文里起 claude，与 project-code-guard.sh 读脚本正文同一个口子；弄坏开关 script-body-claude-unread 下不读）；
    别的程序读进去当代码的文件（共用模块 embedded_launches 交的 awk -f、sed -f、Makefile，正文交 program_text_launches 切出命令；弄坏开关 embedded-launch-claude-unread 下不读）。"""
    seen = set() if seen is None else seen
    found = []
    if depth >= shell_words.MAXIMUM_NESTING:
        return found
    commands = shell_words.commands_at_command_position(text, directory, environment).commands
    written = heredoc_written_files(commands)
    library = project_code_runs_library()
    for command in commands:
        targets = []   # (路径词, 当前目录, 环境, 是不是直接执行, 程序文件或 None)
        if not detector_break_is_set("startup-file-claude-unread"):
            targets += [(startup.words[0], startup.directory, startup.environment, False, None) for startup in startup_files_of(command)]
        if not detector_break_is_set("script-body-claude-unread") and command.words and (
                command.launcher in ("shell", "source") or (command.launcher is None and "/" in command.words[0])):
            targets.append((command.words[0], command.directory, command.environment, command.launcher is None, None))
        if not detector_break_is_set("embedded-launch-claude-unread"):
            targets += [(word, program_file.directory, command.environment, False, program_file)
                        for program_file in shell_words.embedded_launches(command).program_files for word in program_file.words]
        for word, target_directory, target_environment, executed, program_file in targets:
            read = readable_body(word, target_directory, written, executed)
            if read is None or read[0] in seen:
                continue
            real, body = read
            seen.add(real)
            bodies = shell_words.program_text_launches(program_file.language, body, program_file.variables)[0] if program_file is not None else [
                library.script_directory_expanded(body, real) if library is not None else body]
            for program_text in bodies:
                found += claude_launches_in_text(program_text)
                found += file_claude_launches(program_text, target_directory, target_environment, depth + 1, seen)
    return found

def claude_launches_in_text(command):
    """整条命令每一层（顶层、bash -c、喂给 shell 的 heredoc、命令替换）命令位置上起 claude 的简单命令，前缀与包装剥开之后认；交回认出的那几条（截短）。
    另照共用模块切出的每一条命令认（here-string 与管道喂给 shell 的字面、env -S 拆出来的、名单外启动器 xargs / ssh / find -exec 这类剥开之后的，
    例：`bash <<< 'claude -p hi'`、`env -S 'claude -p hi'`、`xargs claude`、`ssh host claude`）；
    弄坏开关 claude-launch-layers-narrow 下只认前一种。"""
    found = []
    for layer in shell_layers(command):
        for words in layer.commands:
            for _, _, name, arguments in wrapped_commands(words):
                if name == CLAUDE_EXECUTABLE_NAME or name in CLAUDE_EXECUTABLE_PATH_WORDS:
                    found.append(" ".join([name, *arguments])[:120])
    if not detector_break_is_set("claude-launch-layers-narrow"):
        for words in launched_command_words(command):
            name = os.path.basename(words[0])
            if name == CLAUDE_EXECUTABLE_NAME or words[0] in CLAUDE_EXECUTABLE_PATH_WORDS:
                found.append(" ".join([name, *words[1:]])[:120])
    return list(dict.fromkeys(found))


def launched_command_words(text, depth=0):
    """共用模块切出的每一条命令的词（here-string、管道喂给 shell 的字面、env -S、bash -c 都已切开）；命令词是启动器（LAUNCHER_NAMES）的，
    照 lib_project_code_runs.py 的 command_under_launcher 剥掉、里面那条再切一遍接着交（MAXIMUM_NESTING 层为限）。lib_project_code_runs.py 读不到时只交切出来的那一层。"""
    library = project_code_runs_library()
    for command in shell_words.commands_at_command_position(text).commands:
        yield command.words
        # 别的程序（python -c、perl -e、awk、sed e、tclsh、git 别名、make --eval 这类）里字面写着起的命令照一段命令文本接着切（共用模块 embedded_launches；三方 d137-gov-r1 判决 H9；
        # 弄坏开关 embedded-launch-claude-unread 下不认）
        if depth < shell_words.MAXIMUM_NESTING and not detector_break_is_set("embedded-launch-claude-unread"):
            for embedded_text in shell_words.embedded_launches(command).texts:
                yield from launched_command_words(embedded_text, depth + 1)
        if library is not None and depth < shell_words.MAXIMUM_NESTING and os.path.basename(command.words[0]) in LAUNCHER_NAMES:
            inner = library.command_under_launcher(command.words, command.directory)
            if inner is not None:
                yield from launched_command_words(shlex.join(inner[0]), depth + 1)

def claude_launch_refusal(command, agent_type, directory=None):
    """交回要拒的起法；主 agent（没有 agent_type）不判。directory 是命令开头的当前目录（解 BASH_ENV / ENV 指的相对路径用）。弄坏开关 allow-subagent-claude-launch 与 disable-check 下放行。"""
    if not agent_type or detector_break_is_set("disable-check") or detector_break_is_set("allow-subagent-claude-launch"):
        return []
    return claude_launches(command, directory)

# ⑮ 暂存区非空时不带具体路径清整个索引（文件头 ⑮）：git reset 不带路径（--soft 除外），git restore --staged 的路径只有整棵树的写法
GIT_GLOBAL_OPTIONS_WITH_VALUE = {"-C", "-c", "--git-dir", "--work-tree", "--namespace", "--super-prefix", "--config-env", "--exec-path"}
GIT_LOCATION_OPTIONS = {"-C", "--git-dir", "--work-tree"}   # 现查暂存区时照带的（-c 这类配置不带进钩子自己起的 git）
GIT_RESET_INDEX_MODES = {"--mixed", "--hard", "--keep", "--merge"}
GIT_RESET_OPTIONS_WITHOUT_VALUE = {"-q", "--quiet", "--no-quiet", "-N", "--intent-to-add", "--no-refresh", "--refresh", "--recurse-submodules", "--no-recurse-submodules"}
GIT_RESTORE_OPTIONS_WITH_VALUE = {"-s", "--source", "--conflict", "--pathspec-from-file"}
WHOLE_TREE_PATHSPECS = {".", "./", ":/", ":/:", ":(top)", ":(top).", "*", ":/*", ":(glob)**"}
GIT_REVISION_FORMS = re.compile(r"^(?:HEAD|@|ORIG_HEAD|FETCH_HEAD|MERGE_HEAD|[0-9a-f]{7,40})(?:[~^][0-9]*|@\{[^}]*\})*$|^refs/")

def git_global_split(arguments):
    """git 之后的参数切成 (全局选项, 子命令, 子命令的参数)；全局选项里认不出的照放进去。没有子命令交 (…, None, [])。"""
    index = 0
    while index < len(arguments):
        word = arguments[index]
        if word in GIT_GLOBAL_OPTIONS_WITH_VALUE:
            index += 2
        elif word.startswith("-"):
            index += 1
        else:
            return arguments[:index], word, arguments[index + 1:]
    return arguments, None, []

def git_location(global_options, directory):
    """全局选项里管「在哪个仓」的那几个：交回 (-C 一层层换到的目录（directory 是 None 或 -C 带变量时交 None）, --git-dir / --work-tree 原样的几个词)；-c 这类配置不交。"""
    kept, index = [], 0
    while index < len(global_options):
        word = global_options[index]
        if word == "-C" and index + 1 < len(global_options):
            value = global_options[index + 1]
            directory = None if directory is None or set("$`*?") & set(value) else os.path.normpath(os.path.join(directory, os.path.expanduser(value)))
            index += 2
            continue
        if word in GIT_LOCATION_OPTIONS and index + 1 < len(global_options):
            kept += global_options[index:index + 2]
            index += 2
            continue
        if word.split("=", 1)[0] in GIT_LOCATION_OPTIONS and "=" in word:
            kept.append(word)
        index += 1
    return directory, kept

def whole_index_pathspec(pathspec):
    """路径都是整棵树的写法（或根本没给）就算清整个索引；弄坏开关 whole-index-pathspec-ignored 下带路径的也算。"""
    if detector_break_is_set("whole-index-pathspec-ignored"):
        return True
    return all(path in WHOLE_TREE_PATHSPECS for path in pathspec)

def git_reset_clears_index(arguments, directory, location=()):
    """git reset 的参数（子命令之后）：清整个索引交 True。--soft 不碰索引；-p 交互不判；--pathspec-from-file 当带了路径。
    没有 -- 时，第一个位置参数是提交（HEAD、HEAD~1、十六进制、refs/…；或不是现成路径而 git rev-parse 认得）还是路径照 git 的认法猜。"""
    options, positional, pathspec, after_separator = [], [], [], False
    for word in arguments:
        if after_separator:
            pathspec.append(word)
        elif word == "--":
            after_separator = True
        elif word.startswith("-"):
            options.append(word)
        else:
            positional.append(word)
    if "--soft" in options or any(option in ("-p", "--patch") or option.startswith("--pathspec-from-file") for option in options):
        return False
    if positional:
        leading_word = positional[0]
        looks_revision = bool(GIT_REVISION_FORMS.match(leading_word)) or (
            directory is not None and not os.path.exists(os.path.join(directory, leading_word)) and
            subprocess.run(["git", "-C", directory, *location, "rev-parse", "--verify", "--quiet", f"{leading_word}^{{commit}}"], capture_output=True, timeout=30).returncode == 0)
        pathspec = (positional[1:] if looks_revision else positional) + pathspec
    return not pathspec or whole_index_pathspec(pathspec)

def git_restore_clears_index(arguments):
    """git restore 的参数：带 --staged / -S（短选项合写的也算）、路径只有整棵树的写法交 True；没给路径（git 自己报错）、--pathspec-from-file 交 False。"""
    staged, pathspec, after_separator, skip = False, [], False, False
    for word in arguments:
        if skip:
            skip = False
            continue
        if after_separator:
            pathspec.append(word)
        elif word == "--":
            after_separator = True
        elif word in GIT_RESTORE_OPTIONS_WITH_VALUE:
            skip = True
        elif word == "--staged" or (word.startswith("-") and not word.startswith("--") and "S" in word[1:] and word[1:].isalpha()):
            staged = True
        elif word.startswith("--pathspec-from-file"):
            return False
        elif not word.startswith("-"):
            pathspec.append(word)
    return staged and bool(pathspec) and whole_index_pathspec(pathspec)

def git_other_form_clears_index(subcommand, arguments):
    """git stash / git rm --cached / git checkout 清不清整个索引：
    `git stash`、`git stash push|save`（不带 -k / --keep-index、不带路径或路径是整棵树）把暂存的改动收走；`git rm --cached` 的路径是整棵树（`-r --cached .`）把索引里的文件全删；
    `git checkout [<提交>] -- <整棵树>` 拿提交或索引盖掉整棵树的暂存改动。别的子命令与带具体路径的交 False。"""
    if subcommand == "stash":
        action = arguments[0] if arguments and not arguments[0].startswith("-") else "push"
        rest = arguments[1:] if arguments and not arguments[0].startswith("-") else arguments
        if action not in ("push", "save"):
            return False
        if any(option in ("-k", "--keep-index") for option in rest):
            return False
        pathspec = [word for word in (rest[rest.index("--") + 1:] if "--" in rest else rest) if not word.startswith("-")] if action == "push" else []
        return not pathspec or whole_index_pathspec(pathspec)
    if subcommand == "rm":
        if "--cached" not in arguments:
            return False
        pathspec = [word for word in (arguments[arguments.index("--") + 1:] if "--" in arguments else arguments) if not word.startswith("-")]
        return bool(pathspec) and whole_index_pathspec(pathspec)
    if subcommand == "checkout":
        if "--" in arguments:
            pathspec = arguments[arguments.index("--") + 1:]
        else:
            positional = [word for word in arguments if not word.startswith("-")]
            pathspec = positional[1:] if positional and GIT_REVISION_FORMS.match(positional[0]) else []
        return bool(pathspec) and whole_index_pathspec(pathspec)
    return False

def staged_index_state(location, directory):
    """暂存区里有没有东西：'有'、'空' 或 '判不了'（目录算不出、不是 git 仓、git 起不来）。弄坏开关 whole-index-staged-unchecked 下一律交 '有'。"""
    if detector_break_is_set("whole-index-staged-unchecked"):
        return "有"
    if directory is None:
        return "判不了"
    try:
        completed = subprocess.run(["git", "-C", directory, *location, "diff", "--cached", "--quiet"], capture_output=True, timeout=30)
    except (OSError, subprocess.SubprocessError):
        return "判不了"
    return {0: "空", 1: "有"}.get(completed.returncode, "判不了")

def whole_index_reset_refusal(command, start_directory):
    """⑮：命令位置上（lib_shell_words 切，cd 与 git -C 跟着算）清整个索引的 git 命令，暂存区非空或判不了时交回认出的写法；主 agent 与子 agent 一样判。
    弄坏开关 allow-whole-index-reset 下一概放行。"""
    if detector_break_is_set("allow-whole-index-reset"):
        return []
    found = []
    for command_at in shell_words.commands_at_command_position(command, start_directory).commands:
        words = command_at.words
        if not words or os.path.basename(words[0]) != "git":
            continue
        global_options, subcommand, arguments = git_global_split(words[1:])
        unresolved = False
        if subcommand and ("$" in subcommand or "`" in subcommand) and not detector_break_is_set("git-subcommand-unexpanded-allowed"):
            # 子命令写成变量、命令替换（`x=reset; git $x`）：前面赋过字面值的照展开再判，展开不了的认不出是不是清索引，照清索引的那一类拒
            expanded = shell_words.expand_known_variables(subcommand, command_at.variables or {})
            if expanded is None:
                unresolved = True
            else:
                subcommand = expanded
        other_forms = not detector_break_is_set("whole-index-other-forms-unseen")
        if not unresolved and subcommand not in ("reset", "restore") and not (other_forms and subcommand in ("stash", "rm", "checkout")):
            continue
        directory, location = git_location(global_options, command_at.directory)
        if unresolved:
            clears = True
        elif subcommand == "reset":
            clears = git_reset_clears_index(arguments, directory, location)
        elif subcommand == "restore":
            clears = git_restore_clears_index(arguments)
        else:
            clears = git_other_form_clears_index(subcommand, arguments)
        if not clears:
            continue
        state = staged_index_state(location, directory)
        if state == "空":
            continue
        shown = " ".join(words)[:120]
        found.append(shown if state == "有" else f"{shown}（目录算不出，暂存区空不空判不了）")
    return list(dict.fromkeys(found))

# ⑱ 带路径撤掉工作区里没暂存的改动（文件头 ⑱）：git checkout [<提交>] [--] <路径…>、git restore <路径…>（不带 --staged，或带 --worktree）、
#   git stash [push] [--] <路径…>、git clean <路径…>；点名的路径里有没暂存的改动（git clean 是会被删的没跟踪文件）就拒
GIT_CHECKOUT_BRANCH_OPTIONS = {"-b", "-B", "--orphan", "--detach"}
GIT_CHECKOUT_OPTIONS_WITH_VALUE = {"--conflict", "--pathspec-from-file"}
GIT_STASH_OPTIONS_WITH_VALUE = {"-m", "--message", "--pathspec-from-file"}
GIT_CLEAN_OPTIONS_WITH_VALUE = {"-e", "--exclude"}
PATHSPEC_FROM_FILE_OPTION = "--pathspec-from-file"

class WorktreeDiscard(NamedTuple):
    paths: list           # 命令点名的路径（照写的样子，变量还没展开）
    untracked: bool       # True：git clean，删的是没跟踪的文件；False：撤的是工作区比暂存区多出来的改动
    ignored: bool         # git clean 带 -x / -X：被忽略的文件也删
    unresolved: str       # 判不了的原因（路径从 --pathspec-from-file 读）；判得了是空串

def short_option_cluster_has(word, letter):
    """短选项合写（-fdx 这类）里有没有 letter；长选项与不是选项的词交 False。"""
    return word.startswith("-") and not word.startswith("--") and len(word) > 1 and word[1:].isalpha() and letter in word[1:]

def git_arguments_split(arguments, options_with_value):
    """子命令的参数切成 (-- 之前的位置参数, -- 之后的词, 选项, 有没有 --)；options_with_value 里分开写的选项连同它的值一起跳过（值不进位置参数）。"""
    positional, after, options, skip, separated = [], [], [], False, False
    for word in arguments:
        if skip:
            skip = False
        elif separated:
            after.append(word)
        elif word == "--":
            separated = True
        elif word in options_with_value:
            options.append(word)
            skip = True
        elif word.startswith("-") and word != "-":
            options.append(word)
        else:
            positional.append(word)
    return positional, after, options, separated

def git_names_commit(word, directory, location):
    """不带 -- 的 git checkout 第一个位置参数是不是提交：像提交的写法（HEAD、十六进制、refs/…），或盘上没有这个路径而 git rev-parse 认得（同 git_reset_clears_index）。"""
    if GIT_REVISION_FORMS.match(word):
        return True
    if directory is None or os.path.exists(os.path.join(directory, word)):
        return False
    return subprocess.run(["git", "-C", directory, *location, "rev-parse", "--verify", "--quiet", f"{word}^{{commit}}"],
                          capture_output=True, timeout=30).returncode == 0

def worktree_discard_paths(subcommand, arguments, directory, location):
    """⑱ 认写法：带路径撤工作区改动的 git 命令交 WorktreeDiscard；没点名路径的、只动暂存区的 restore --staged、切分支、git clean -n、别的子命令交 None。
    弄坏开关 path-discard-staged-counted 下 restore --staged（不带 --worktree）也当动工作区。"""
    untracked = ignored = False
    if subcommand == "checkout":
        positional, after, options, separated = git_arguments_split(arguments, GIT_CHECKOUT_OPTIONS_WITH_VALUE)
        if any(option.split("=", 1)[0] in GIT_CHECKOUT_BRANCH_OPTIONS for option in options):
            return None
        if separated:
            paths = after
        elif positional and git_names_commit(positional[0], directory, location):
            paths = positional[1:]
        else:
            paths = positional
    elif subcommand == "restore":
        positional, after, options, _ = git_arguments_split(arguments, GIT_RESTORE_OPTIONS_WITH_VALUE)
        staged = any(option == "--staged" or short_option_cluster_has(option, "S") for option in options)
        worktree = any(option == "--worktree" or short_option_cluster_has(option, "W") for option in options)
        if staged and not worktree and not detector_break_is_set("path-discard-staged-counted"):
            return None
        paths = positional + after
    elif subcommand == "stash":
        explicit = bool(arguments) and not arguments[0].startswith("-")
        if explicit and arguments[0] != "push":
            return None
        positional, after, options, _ = git_arguments_split(arguments[1:] if explicit else arguments, GIT_STASH_OPTIONS_WITH_VALUE)
        paths = positional + after
    elif subcommand == "clean":
        positional, after, options, _ = git_arguments_split(arguments, GIT_CLEAN_OPTIONS_WITH_VALUE)
        if any(option == "--dry-run" or short_option_cluster_has(option, "n") for option in options):
            return None
        paths = positional + after
        untracked = True
        ignored = any(short_option_cluster_has(option, "x") or short_option_cluster_has(option, "X") for option in options)
    else:
        return None
    if any(option.split("=", 1)[0] == PATHSPEC_FROM_FILE_OPTION for option in options):
        return WorktreeDiscard([], untracked, ignored, f"路径从 {PATHSPEC_FROM_FILE_OPTION} 读")
    if not paths:
        return None
    return WorktreeDiscard(paths, untracked, ignored, "")

def unstaged_hits(discard, paths, location, directory):
    """点名的路径里有没暂存改动的（git clean 是会被删的没跟踪文件）：交回它们的路径（git 打的、相对仓根）；git 报错（不是 git 仓、路径在仓外：命令自己也跑不成）交 []；
    git 起不来交 None（判不了）。弄坏开关 path-discard-changes-unchecked 下不查，点名的路径都算有改动。"""
    if detector_break_is_set("path-discard-changes-unchecked"):
        return list(paths)
    if discard.untracked:
        query = ["status", "--porcelain=v1", "-z", "--untracked-files=all"] + (["--ignored"] if discard.ignored else []) + ["--", *paths]
    else:
        query = ["diff", "--name-only", "-z", "--", *paths]
    try:
        completed = subprocess.run(["git", "-C", directory, *location, *query], capture_output=True, timeout=30,
                                   env=dict(os.environ, GIT_OPTIONAL_LOCKS="0", LC_ALL="C"))
    except (OSError, subprocess.SubprocessError):
        return None
    if completed.returncode != 0:
        return []
    entries = [entry for entry in completed.stdout.decode("utf-8", "replace").split("\0") if entry]
    if discard.untracked:
        return [entry[3:] for entry in entries if entry[:3] in ("?? ", "!! ")]
    return entries

def worktree_discard_refusal(command, start_directory):
    """⑱：命令位置上（lib_shell_words 切，cd 与 git -C 跟着算）带路径撤工作区改动的 git 命令，点名的路径里有没暂存的改动、或判不了时交回认出的写法；
    主 agent 与子 agent 一样判。弄坏开关 allow-path-discard 下一概放行。"""
    if detector_break_is_set("allow-path-discard"):
        return []
    found = []
    for command_at in shell_words.commands_at_command_position(command, start_directory).commands:
        words = command_at.words
        if not words or os.path.basename(words[0]) != "git":
            continue
        global_options, subcommand, arguments = git_global_split(words[1:])
        if subcommand not in ("checkout", "restore", "stash", "clean"):
            continue
        directory, location = git_location(global_options, command_at.directory)
        discard = worktree_discard_paths(subcommand, arguments, directory, location)
        if discard is None:
            continue
        shown = " ".join(words)[:120]
        paths = [shell_words.expand_known_variables(path, command_at.variables or {}) for path in discard.paths]
        problem = (discard.unresolved or ("路径写成展开不了的变量或命令替换" if None in paths else "")
                   or ("目录算不出" if directory is None else ""))
        if problem:
            found.append(f"{shown}（{problem}，判不了点名的路径有没有没暂存的改动）")
            continue
        hits = unstaged_hits(discard, [os.path.expanduser(path) for path in paths], location, directory)
        if hits is None:
            found.append(f"{shown}（git 起不来，判不了点名的路径有没有没暂存的改动）")
        elif hits:
            kind = "没跟踪的文件" if discard.untracked else "没暂存的改动"
            found.append(f"{shown}（有{kind}：{'、'.join(hits[:5])}{'…' if len(hits) > 5 else ''}）")
    return list(dict.fromkeys(found))

# 检出 ③：项目子 agent 前台 Bash 调用的 tool_input.timeout 超过共用约束「长活交回主 agent」写的上限，只记、不拒
SUBAGENT_FOREGROUND_TIMEOUT_MILLISECONDS = 240000

def subagent_timeout_findings(hook_input):
    """子 agent（输入里有 agent_type）前台（run_in_background 不是 true）的 tool_input.timeout 超过 SUBAGENT_FOREGROUND_TIMEOUT_MILLISECONDS：交回一条检出；
    主 agent、run_in_background、没写、写的不是数（布尔、认不出的字符串）的交 []。弄坏开关 subagent-timeout-unnoted 下一概交 []。"""
    tool_input = hook_input.get("tool_input") if isinstance(hook_input.get("tool_input"), dict) else {}
    if not hook_input.get("agent_type") or tool_input.get("run_in_background") is True or detector_break_is_set("subagent-timeout-unnoted"):
        return []
    value = tool_input.get("timeout")
    if isinstance(value, str) and re.fullmatch(r"\s*\d+(?:\.\d+)?\s*", value):
        value = float(value)
    if isinstance(value, bool) or not isinstance(value, (int, float)) or value <= SUBAGENT_FOREGROUND_TIMEOUT_MILLISECONDS:
        return []
    shown = int(value) if float(value).is_integer() else value
    return [f"子 agent 前台命令的 timeout 是 {shown} 毫秒，超过共用约束 .claude/agent-common.md「长活交回主 agent」写的 {SUBAGENT_FOREGROUND_TIMEOUT_MILLISECONDS} 毫秒："
            "前台跑不完的活不起，命令原样写进报告「要主 agent 跑的命令」一节交回主 agent"]

# run_in_background 里等本会话后台任务输出文件（<会话目录>/tasks/<id>.output）的循环：等自己起的后台任务要结束本轮等完成通知，不写轮询
TASK_OUTPUT_PATH = re.compile(r"(?:^|/)tasks/[^/\s]+\.output\b")

def waits_on_task_output(loop_commands):
    return any(TASK_OUTPUT_PATH.search(word) for words in loop_commands for word in words if words)

def background_task_output_wait_loops(text):
    """命令位置上的等待循环、条件或循环体里点名了 tasks/<id>.output 的，外面有没有 timeout 都算。"""
    return [loop for layer in shell_layers(text, honor_timeout=False) for loop in wait_loops_in(layer.commands, waits_on_task_output)]

def subagent_background_refusal(agent_type, run_in_background):
    """⑩ 子 agent（输入里有 agent_type）用 run_in_background 起命令就交 True（task-server/task wait 也算）；主 agent 与前台命令交 False。"""
    if detector_break_is_set("disable-check") or detector_break_is_set("allow-subagent-background"):
        return False
    return bool(agent_type) and run_in_background is True

def wait_loop_refusal(command, run_in_background):
    """前台（run_in_background 不是 true）的命令里有没超时的等待循环，就交回认出的循环（每个循环开头那条简单命令）；否则交 []。"""
    if (detector_break_is_set("disable-check")
            or detector_break_is_set("allow-foreground-wait-loop")):
        return []
    refuse_every = detector_break_is_set("refuse-every-wait-loop")
    if run_in_background is True and not refuse_every:
        if detector_break_is_set("allow-background-task-output-wait"):
            return []
        return background_task_output_wait_loops(command)
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

def wait_coverage(words, wait_functions):
    """一条命令等了几个之前放后台的作业：不带参数的 wait、调用里写着不带参数 wait 的函数（Z14）交 None（全等）；`wait <进程号>…` 交进程号个数；
    `wait -n`（只带选项）、`tail --pid`、`while|until … kill -0` 轮询（文件头 ② 末那一句，Z12、Z13）各交 1；别的交 0。
    弄坏开关 wait-forms-unseen 下 wait -n、tail --pid、kill -0 轮询、函数里的 wait 都不算；partial-wait-allowed 下带进程号的 wait 照全等算。"""
    skipped = shell_words.skip_prefixes(words)
    keywords = words[:skipped.command_position] if skipped.command_position is not None else words
    if skipped.command_position is None:
        return 0
    name, arguments = words[skipped.command_position], words[skipped.command_position + 1:]
    forms_seen = not detector_break_is_set("wait-forms-unseen")
    if name == "wait":
        pids = [argument for argument in arguments if not argument.startswith("-")]
        if not arguments:
            return None
        if pids:
            return None if detector_break_is_set("partial-wait-allowed") else len(pids)
        return 1 if forms_seen else 0
    if forms_seen and name in wait_functions:
        return None
    if forms_seen and name == "tail" and any(argument.startswith("--pid") for argument in arguments):
        return 1
    if forms_seen and any(keyword in ("while", "until") for keyword in keywords) and name == "kill" and arguments[:1] == ["-0"]:
        return 1
    return 0


def wait_function_names(tokens):
    """一层记号里定义的、函数体里有一条不带参数的 wait 的函数名（`w() { wait; }`、`function w { wait; }`）。"""
    names = set()
    for index, (kind, value) in enumerate(tokens):
        name, body = None, None
        if kind is False and value == "function" and index + 1 < len(tokens) and tokens[index + 1][0] is False:
            name, body = tokens[index + 1][1], index + 2
            if tokens[body:body + 2] == [(True, "("), (True, ")")]:
                body += 2
        elif kind is False and tokens[index + 1:index + 3] == [(True, "("), (True, ")")]:
            name, body = value, index + 3
        if name is None or tokens[body:body + 1] != [(False, "{")]:
            continue
        depth, cursor = 0, body
        while cursor < len(tokens):
            if tokens[cursor] == (False, "{"):
                depth += 1
            elif tokens[cursor] == (False, "}"):
                depth -= 1
                if depth == 0:
                    break
            cursor += 1
        for body_words in shell_words.simple_commands(tokens[body + 1:cursor])[0]:
            if body_words and body_words[-1:] != ["}"] and wait_coverage([word for word in body_words if word != "{"], set()) is None:
                names.add(name)
    return names


def unwaited_background_jobs(text):
    """整条命令每一层（顶层、bash -c 的代码、喂给 shell 的 heredoc 正文、命令替换）里以单独的 & 收尾、之后同一层同一对圆括号里没等到的作业，交回每个作业的说法。
    逐层按次序数：不带参数的 wait（或调用里有它的函数）等齐之前的全部；`wait <进程号>` 等它写了几个号就几个（两个 & 只等一个的，剩下那个照没等到拒）；
    `wait -n`、`tail --pid`、`kill -0` 轮询各等一个（Z12、Z13），先等先起的那个。"""
    found = []
    for layer in shell_layers(text):
        jobs = jobs_with_endings(layer.tokens)
        wait_functions = set() if detector_break_is_set("wait-forms-unseen") else wait_function_names(layer.tokens)
        pending = {}
        for words, ending, group in jobs:
            if ending == "&":
                pending.setdefault(group, []).append(words)
                continue
            covered = wait_coverage(words, wait_functions)
            if covered is None:
                pending[group] = []
            elif covered:
                pending[group] = pending.get(group, [])[covered:]
        for waiting in pending.values():
            found += [background_job_label(words) for words in waiting]
    return list(dict.fromkeys(found))

def unwaited_ampersand_refusal(command, run_in_background):
    """run_in_background 为 true 的命令里有以单独的 & 收尾、之后同一层没有 wait 的作业，就交回认出的作业；否则交 []。前台不拒。"""
    if (detector_break_is_set("disable-check")
            or detector_break_is_set("allow-unwaited-ampersand")):
        return []
    if run_in_background is not True:
        return []
    return unwaited_background_jobs(command)

# ⑤ 整份覆盖 research/results/ 下未跟踪的产物：按命令位置认会整份覆盖文件的写法，跟着 cd 与这条命令里赋过的变量算目标
OVERWRITING_REDIRECTS = {">", ">|", "&>", ">&"}   # >> 与 &>> 是追加；>& 后面跟 fd 号或 - 时是复制、关闭 fd，不写文件
APPENDING_REDIRECTS = {">>", "&>>"}                # 只有 ⑨（every_write）认：追加也改了那一份
FILE_DESCRIPTOR_WORD = re.compile(r"^(?:\d+-?|-)$")
ASSIGNED_VARIABLE = re.compile(r"\$(?:\{([A-Za-z_][A-Za-z0-9_]*)\}|([A-Za-z_][A-Za-z0-9_]*))")
# 目标词里这一刻算不出的段：变量（含 ${…} 的各种展开与特殊参数）、花括号展开；命令替换另由共用模块的 substitution_span 认
BRACE_PIECE = re.compile(r"\{[^{}/]*(?:,|\.\.)[^{}/]*\}")   # 目标词里的花括号展开（`{a,b}`、`{1..2}`）：重定向目标上的单列一种拒绝说明
UNKNOWN_PIECE = re.compile(r"\$\{[^}]*\}|\$[A-Za-z_][A-Za-z0-9_]*|\$[0-9@*#?$!-]|\{[^{}/]*(?:,|\.\.)[^{}/]*\}")
GLOB_CHARACTERS = set("*?[")
RESULTS_DIRECTORY_WORDS = ("research", "results")
# cp / mv / install：命令名 → 要带一个值的短选项；带值的长选项（不写 = 时值是下一个词）
COPY_SHORT_OPTIONS_WITH_VALUE = {"cp": "tS", "mv": "tS", "install": "tSgmo"}
COPY_LONG_OPTIONS_WITH_VALUE = {"--target-directory", "--suffix", "--no-preserve", "--sparse", "--group", "--mode", "--owner", "--strip-program"}
GIT_GLOBAL_OPTIONS_WITH_VALUE = {"-C", "-c", "--git-dir", "--work-tree", "--namespace", "--config-env"}
GIT_ADD_OPTIONS_THAT_DO_NOT_KEEP_BYTES = {"-n", "--dry-run", "-u", "--update", "-N", "--intent-to-add", "-p", "--patch", "-i", "--interactive", "-e", "--edit", "--refresh"}
RESULTS_PRESERVING_KINDS = {"git add", "mv"}   # rm 掉的旧字节没留住，⑤ 不认

class OverwriteCandidate(NamedTuple):
    form: str                            # 认出的写法：>、>|、&>、>&、tee、cp、mv、install、dd、truncate；⑤ 与 ⑦ 共用同一份（PythonCode 另判，只给 ⑦ 用）
    target: str                          # 目标词（这条命令里前面赋过值的变量已代入）
    directory: str | None                # 那一刻的当前目录；cd 算不出来是 None，按仓库根解析
    sources: tuple = ()                  # cp / mv / install 的源
    into_directory: bool | None = False  # True：-t 给的目录；None：看目标是不是已有目录、以 / 结尾；False：目标就是写的那个文件
    replaces_inode: bool = False         # 先删掉目标或换成链接再写（cp --remove-destination、-l、-s）：⑦ 不拦，⑤ 照样算覆盖
    appends: bool = False                # tee -a / --append（只在 every_write 下交，form 照写 tee）：⑲ 认它是追加

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

def in_place_editor_files(name, arguments):
    """sed -i / perl -i 改的文件（⑨ 用）；不带 -i 的交 []。sed 沿用 ⑧ 的 sed_in_place_targets（/tmp/ 下的不交）。"""
    if name == "sed":
        return sed_in_place_targets(arguments)
    if not any(re.match(r"^-[a-zA-Z]*i", argument) for argument in arguments):
        return []
    files, skip_next, script_given = [], False, False
    for argument in arguments:
        if skip_next:
            skip_next = False
        elif re.match(r"^-[a-zA-Z]*[eE]$", argument):
            skip_next, script_given = True, True
        elif not argument.startswith("-"):
            files.append(argument)
    return files if script_given else files[1:]

MKTEMP_SUBSTITUTION = re.compile(r"^(?:\$\(\s*mktemp\s+([^()]*)\)|`\s*mktemp\s+([^`]*)`)$")

MKTEMP_CALL = r"(?:\$\(\s*mktemp\s+[^()]*\)|`\s*mktemp\s+[^`]*`)"
# `$(cd $(mktemp …) && pwd -P)`（`pwd`、`pwd -L`、`;` 代 `&&`、反引号写法）：整段、第 1 或 2 组是里面那条 mktemp 命令替换
MKTEMP_INSIDE_PWD = re.compile(r"^(?:\$\(\s*cd\s+\"?(" + MKTEMP_CALL + r")\"?\s*(?:&&|;)\s*pwd(?:\s+-[LP])?\s*\)"
                               r"|`\s*cd\s+\"?(" + MKTEMP_CALL + r")\"?\s*(?:&&|;)\s*pwd(?:\s+-[LP])?\s*`)$")
MKTEMP_IN_WORD = re.compile(r"\$\(\s*cd\s+\"?" + MKTEMP_CALL + r"\"?\s*(?:&&|;)\s*pwd(?:\s+-[LP])?\s*\)|" + MKTEMP_CALL)
SCRATCH_OWNER_IDENTITY = {}   # ⑫ 判一条命令时由 bash_scope_refusal 设成 hook 输入里的 agent_id、session_id、transcript_path；别处调 overwrite_steps 时空着
WRITER_LOCKS = []             # 同目录的 lib_writer_lock.py，第一次用到才导入；导入不了是 [None]

def writer_locks_library():
    if not WRITER_LOCKS:
        try:
            sys.path.insert(0, sys.argv[1]); import lib_writer_lock
            WRITER_LOCKS.append(lib_writer_lock)
        except Exception as error:   # 文件不在、语法错：认不出自己的草稿目录，带目录的 mktemp 模板照旧算不出
            not_judged(f"bash-command-detector.sh 读不到 lib_writer_lock.py（{error!r}），mktemp 模板在不在自己的草稿目录里这一次没判")
            WRITER_LOCKS.append(None)
    return WRITER_LOCKS[0]

def own_scratch_directory(directory):
    """directory 落在这个子 agent（SCRATCH_OWNER_IDENTITY）认领的草稿目录里：工作根下第一层那个目录的归属文件 .owner 写的会话是 hook 输入的会话，
    而且离 directory 最近的写者锁归它（认领文件写的是它的 agent_id；还没有认领文件时，主会话记录里它的派发回执与锁对得上）。只读，不建认领文件。
    弄坏开关 mktemp-template-owner-unchecked 下工作根下的目录一律算。"""
    agent_id, session_id = SCRATCH_OWNER_IDENTITY.get("agent_id"), SCRATCH_OWNER_IDENTITY.get("session_id")
    if not agent_id or work_roots is None:
        return False
    directory = os.path.normpath(directory)
    for root in work_roots.scratch_roots():
        root = root.rstrip("/")
        if not directory.startswith(root + "/"):
            continue
        if detector_break_is_set("mktemp-template-owner-unchecked"):
            return True
        first_level = os.path.join(root, directory[len(root) + 1:].split("/")[0])
        try:
            owner_lines = open(os.path.join(first_level, ".owner"), encoding="utf-8").read().splitlines()
        except OSError:
            return False
        if not session_id or f"session: {session_id}" not in [line.strip() for line in owner_lines]:
            return False
        locks = writer_locks_library()
        lock_directory = locks.find_lock_directory(os.path.join(directory, "probe"), [root]) if locks is not None else None
        lock = locks.read_lock(lock_directory) if lock_directory is not None else None
        if lock is None:
            return False
        claim = locks.read_claim(lock_directory, lock)
        if claim is not None:
            return claim["agent_id"] == agent_id
        receipt = locks.launch_receipt(agent_id, SCRATCH_OWNER_IDENTITY.get("transcript_path"))
        return (receipt is not None and receipt[1] == lock.get("prompt_sha256")
                and receipt[0] >= lock["dispatched_at"] - locks.LAUNCH_SLACK_SECONDS)
    return False

def mktemp_template_directory(value, directory=None):
    """`$(mktemp -d <模板>)`（或反引号写法）不带 -t、模板所在的目录（带目录的写法，或相对 directory 的）落在这个子 agent 认领的草稿目录里时，
    交回它建的目录（那个目录下一个名字）；别的写法、模板里还有 $ 与通配、目录不在自己的草稿目录里交 None。弄坏开关 mktemp-scratch-template-unknown 下一概交 None。"""
    if detector_break_is_set("mktemp-scratch-template-unknown"):
        return None
    match = MKTEMP_SUBSTITUTION.match(value.strip())
    if not match:
        return None
    arguments = (match.group(1) if match.group(1) is not None else match.group(2)).split()
    options = [argument for argument in arguments if argument.startswith("-")]
    operands = [argument for argument in arguments if not argument.startswith("-")]
    if len(operands) != 1 or not all(option == "--directory" or re.fullmatch(r"-[dq]+", option) for option in options):
        return None
    if not any(option == "--directory" or "d" in option for option in options) or re.search(r"[$`*?\[{]", operands[0]):
        return None
    template = shell_words.resolve_path(directory, os.path.expanduser(operands[0]))
    if template is None:
        return None
    parent = os.path.dirname(template)
    return os.path.join(parent, "mktemp-directory") if own_scratch_directory(parent) else None

def mktemp_inside_pwd(value):
    """`$(cd $(mktemp …) && pwd -P)`（`pwd`、`pwd -L`、`;` 代 `&&`、反引号写法同）：交回里面那条 mktemp 命令替换；不是这个形状交 None。
    弄坏开关 mktemp-nested-substitution-unknown 下一概交 None。"""
    if detector_break_is_set("mktemp-nested-substitution-unknown"):
        return None
    match = MKTEMP_INSIDE_PWD.match(value.strip())
    return (match.group(1) or match.group(2)) if match else None

def unquoted(word):
    """整个词被一对同样的引号包着的，去掉那一对。"""
    return word[1:-1] if len(word) >= 2 and word[0] == word[-1] and word[0] in "'\"" else word

def mktemp_option_directory(value, directory=None):
    """`$(mktemp -d -p <目录>)`、`--tmpdir=<目录>`（可再带一个不含 / 的模板名；`-p` 可与 -d、-q、-t 合写）：<目录>（相对 directory 的也算）落在这个子 agent 认领的草稿目录里时，
    交回它建的目录（那个目录下一个名字）；<目录> 带变量与通配、在 /tmp、仓里、别的子 agent 或别的会话的草稿目录里、别的选项、模板名带 / 都交 None。
    弄坏开关 mktemp-option-directory-unknown 下一概交 None。"""
    if detector_break_is_set("mktemp-option-directory-unknown"):
        return None
    match = MKTEMP_SUBSTITUTION.match(value.strip())
    if not match:
        return None
    arguments = (match.group(1) if match.group(1) is not None else match.group(2)).split()
    option_directory, rest, index = None, [], 0
    while index < len(arguments):
        argument = arguments[index]
        if re.fullmatch(r"-[dqt]*p", argument):
            if index + 1 >= len(arguments):
                return None
            option_directory = arguments[index + 1]
            if len(argument) > 2:
                rest.append(argument[:-1])
            index += 2
        elif argument.startswith("--tmpdir="):
            option_directory = argument[len("--tmpdir="):]
            index += 1
        else:
            rest.append(argument)
            index += 1
    if option_directory is None:
        return None
    options = [argument for argument in rest if argument.startswith("-")]
    operands = [unquoted(argument) for argument in rest if not argument.startswith("-")]
    if len(operands) > 1 or not all(option == "--directory" or re.fullmatch(r"-[dqt]+", option) for option in options):
        return None
    if not any(option == "--directory" or (not option.startswith("--") and "d" in option) for option in options):
        return None
    if operands and ("/" in operands[0] or re.search(r"[$`*?\[{]", operands[0])):
        return None
    option_directory = unquoted(option_directory)
    if not option_directory or re.search(r"[$`*?\[{]", option_directory):
        return None
    parent = shell_words.resolve_path(directory, os.path.expanduser(option_directory))
    if parent is None:
        return None
    return os.path.join(parent, "mktemp-directory") if own_scratch_directory(parent) else None

def mktemp_created_directory(value, directory=None):
    """一条 mktemp 命令替换（可套在 `$(cd $(mktemp …) && pwd -P)` 里，mktemp_inside_pwd）建出来的目录，落在这个子 agent 认领的草稿目录里时交回它的位置：
    `-p <目录>`、`--tmpdir=<目录>` 指的（mktemp_option_directory），或不带 -t 的模板所在的（mktemp_template_directory）；别的交 None。"""
    inner = mktemp_inside_pwd(value)
    if inner is not None:
        value = inner
    return mktemp_option_directory(value, directory) or mktemp_template_directory(value, directory)

def mktemp_substitutions_expanded(word, directory):
    """目标词里直接写的 `$(mktemp -d <模板>)`、`$(mktemp -d -p <目录>)`（可套在 `$(cd … && pwd -P)` 里），建的目录在自己认领的草稿目录里的，换成它建的目录（mktemp_created_directory）；别的照留。"""
    return MKTEMP_IN_WORD.sub(lambda match: mktemp_created_directory(match.group(0), directory) or match.group(0), word)

def mktemp_directory_value(value, directory=None):
    """赋的值是 `$(mktemp -d)`（或反引号写法，可带 -t 与不含 / 的模板）时，交回它建的目录落在哪：${TMPDIR:-/tmp} 下一个名字，仓外；
    不带 -t 的模板在这个子 agent 认领的草稿目录里时，交回草稿目录里的一个名字（mktemp_template_directory，directory 是相对模板的当前目录）；
    `-p <目录>`、`--tmpdir=<目录>` 指到自己认领的草稿目录里的同样（mktemp_option_directory），不带值的 `--tmpdir` 同 -t；套在 `$(cd $(mktemp …) && pwd -P)` 里的按里面那条算（mktemp_inside_pwd）；
    别的写法（不带 -d 的、-p 与 --tmpdir 指的目录不在自己的草稿目录里的、模板在别处的）交 None，变量照旧算不出。弄坏开关 mktemp-directory-unknown 下 ${TMPDIR:-/tmp} 那一种交 None。"""
    in_scratch = mktemp_created_directory(value, directory)
    if in_scratch is not None:
        return in_scratch
    if detector_break_is_set("mktemp-directory-unknown"):
        return None
    match = MKTEMP_SUBSTITUTION.match((mktemp_inside_pwd(value) or value).strip())
    if not match:
        return None
    arguments = (match.group(1) if match.group(1) is not None else match.group(2)).split()
    options = [argument for argument in arguments if argument.startswith("-")]
    operands = [argument for argument in arguments if not argument.startswith("-")]
    bare_tmpdir_known = not detector_break_is_set("mktemp-option-directory-unknown")
    if not all(option == "--directory" or (option == "--tmpdir" and bare_tmpdir_known) or re.fullmatch(r"-[dtq]+", option) for option in options):
        return None
    makes_directory = any(option == "--directory" or (not option.startswith("--") and "d" in option) for option in options)
    uses_template_in_temporary = any((not option.startswith("--") and "t" in option) or option == "--tmpdir" for option in options)
    if not makes_directory or len(operands) > 1 or (operands and (not uses_template_in_temporary or "/" in operands[0])):
        return None
    temporary = os.environ.get("TMPDIR") or "/tmp"
    return os.path.join(temporary if os.path.isabs(temporary) else "/tmp", "mktemp-directory")

def remember_working_directory(variables, directory):
    """cd 之后 $PWD 就是那个目录（算得出时记进 variables，后面的目标代入）；算不出时把之前记的作废。弄坏开关 pwd-variable-unexpanded 下不记。"""
    if detector_break_is_set("pwd-variable-unexpanded"):
        return
    if directory is None:
        variables.pop("PWD", None)
    else:
        variables["PWD"] = directory

def follow_simple_command(words, directory, variables, steps, depth, heredoc_bodies=(), every_write=False, write_scripts=False):
    """一条简单命令：会整份覆盖文件的写法（tee、cp、mv、install、dd、truncate；⑦ 另认喂给 python 的代码）与让旧字节留住的一步
    （git add、mv 挪走源；⑦ 另认 rm）追加进 steps；跟着 cd / pushd 换目录，跟着独立的赋值与 export 记变量（改 variables）；
    bash -c 的那段代码按这一刻的目录与变量递归进去。heredoc_bodies 是喂给这一条的、被当数据剥掉的 heredoc 正文。交回这条之后的当前目录。"""
    skipped = shell_words.skip_prefixes(words)
    if skipped.command_position is None:
        for name, value in skipped.assignments.items():
            variables[name] = mktemp_directory_value(substitute_assigned_variables(value, variables), directory) or substitute_assigned_variables(value, variables)
        return directory
    name, arguments = None, []
    for _, _, name, arguments in wrapped_commands(words):
        pass
    if name is None:
        return directory
    if write_scripts:
        steps += known_write_script_steps(name, arguments, directory, variables)
        inner_text = scope_launcher_inner_text(name, arguments, directory)
        if inner_text is not None:
            steps += overwrite_steps(inner_text, directory, variables, depth + 1, every_write, write_scripts)
            return directory
    if shell_words.PYTHON_NAME.match(name):
        steps += [PythonCode(code, directory) for code in python_code_arguments(arguments, heredoc_bodies)]
        return directory
    if write_scripts and (name in LOOP_BINDING_COMMANDS or name in READING_COMMANDS):
        bind_loop_variables(name, arguments, variables)   # ⑫：循环变量逐个取值，read 读进来的作废
        return directory
    arguments = [substitute_assigned_variables(argument, variables) for argument in arguments]
    if name == "export":
        for argument in arguments:
            assignment = shell_words.ASSIGNMENT.match(argument)
            if assignment:
                variables[assignment.group(1)] = mktemp_directory_value(assignment.group(2), directory) or assignment.group(2)
    elif name in ("cd", "pushd"):
        directory = shell_words.changed_directory(directory, arguments)
        if directory is not None and LOOP_VALUE_MARK in directory:
            directory = None   # ⑫ cd 进循环变量拼的目录：不逐个取值跟进去，之后的相对路径照算不出判
        remember_working_directory(variables, directory)
        return directory
    elif name == "popd":
        remember_working_directory(variables, None)
        return None
    elif name == "tee":
        files = tee_files(arguments)
        appending = every_write and not files   # tee -a / --append：tee_files 对它交空表
        if appending:
            files = [argument for argument in arguments if argument != "-" and not argument.startswith("-")]
        steps += [OverwriteCandidate("tee", file, directory, appends=appending) for file in files]
    elif every_write and name in ("sed", "perl"):
        steps += [OverwriteCandidate(f"{name} -i", file, directory) for file in in_place_editor_files(name, arguments)]
    elif (name == "rsync" and any(argument == "--inplace" or argument.startswith("--inplace=") for argument in arguments)
          and not detector_break_is_set("rsync-inplace-unseen")):
        # rsync --inplace 往已有的文件里就地写、不换 inode，⑤ ⑦ 照整份覆盖认；弄坏开关 rsync-inplace-unseen 下只有 ⑨ ⑫ 认它
        operands = [argument for argument in arguments if not argument.startswith("-")]
        if len(operands) >= 2:
            steps.append(OverwriteCandidate("rsync --inplace", operands[-1], directory, tuple(operands[:-1]), None))
    elif every_write and name == "rsync":
        operands = [argument for argument in arguments if not argument.startswith("-")]
        if len(operands) >= 2:
            steps.append(OverwriteCandidate("rsync", operands[-1], directory, tuple(operands[:-1]), None))
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
            steps += overwrite_steps(code, directory, variables, depth + 1, every_write, write_scripts)
        elif heredoc_bodies and shell_words.heredoc_feeds_shell(shlex.join([name, *arguments])):
            # 管道喂进来的：`cat <<'EOF' | bash` 的正文被当数据剥掉了，bash 从标准输入读它，照命令逐行判
            steps += overwrite_steps(heredoc_bodies[-1].text, directory, variables, depth + 1, every_write, write_scripts)
    return directory

def scope_launcher_inner_text(name, arguments, directory):
    """⑫ 用：eval 的代码（参数照 eval 的样子用空格接起来）与启动器（xargs、find -exec、parallel、flock、/usr/bin/time 这类，lib_project_code_runs.py 的 command_under_launcher）
    里面那条命令，交回一段命令文本，照命令接着判写到哪；xargs、find、parallel 读进来接进命令的词是没展开的占位词（LAUNCHER_INPUT_WORDS），落点照算不出拒。
    不是这几种的交 None。弄坏开关 scope-launcher-writes-unseen 下一律交 None（这几种起的写看不见）。"""
    if detector_break_is_set("scope-launcher-writes-unseen"):
        return None
    if name == "eval":
        return " ".join(arguments) if arguments else None
    if name not in LAUNCHER_NAMES:   # 与 lib_project_code_runs.launcher_names() 同一份名单（自证「启动器名单与 lib_project_code_runs 的一致」）；不是启动器的不装那个库
        return None
    library = project_code_runs_library()
    if library is None:
        return None
    inner = library.command_under_launcher([name, *arguments], directory)
    if inner is None:
        return None
    inner_words, inner_directory = inner
    prefix = f"cd {shlex.quote(inner_directory)} && " if inner_directory else ""
    return prefix + " ".join(word if word in launcher_input_words() else shlex.quote(word) for word in inner_words)

def launcher_input_words():
    """xargs、find、parallel 读进来接进命令的词在 lib_project_code_runs.py 里写成的占位词（读不到它时是空的）。"""
    library = project_code_runs_library()
    if library is None:
        return ()
    return tuple(getattr(library, name) for name in ("XARGS_INPUT_WORD", "FIND_PATH_WORD", "PARALLEL_INPUT_WORD") if hasattr(library, name))

def command_pieces(tokens):
    """一层的记号按命令分隔符切成一段段：一段至多一条简单命令，连同它的重定向与替换正文；只有重定向、没有词的也成一段
    （`(…) > 文件`、`{ …; } > 文件` 右括号之后那段，共用模块的 simple_commands 不交它）。数组赋值 `名字=(…)` 的括号留在段里。
    交回 (段, 管道标记)，两个列表一一对应：标记为真的段，标准输入是前一段经 `|` 或 `|&` 送来的（`cat <<'EOF' | python3 -` 里 python3 那一段）。"""
    pieces, piped, after_pipe, piece, position = [], [], False, [], 0
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
                piped.append(after_pipe)
            after_pipe = text in ("|", "|&") or (after_pipe and not piece and text == "\n")   # 管道符之后换行，下一段还是管道后一段
            piece = []
        else:
            piece.append(tokens[position])
        position += 1
    if piece:
        pieces.append(piece)
        piped.append(after_pipe)
    return pieces, piped

PRINTF_FORMAT_SLOT = re.compile(r"%[system_configuration]")

def piped_output_bodies(words, stdin_bodies):
    """一条命令交给管道的文本（包成 HeredocBody，管道后一段当它的标准输入）：cat（没有文件操作数）原样交出它读到的标准输入，echo、printf 交出自己的参数；
    别的命令交 ()——它们的输出算不出。stdin_bodies 是这条命令自己的标准输入（heredoc 或更前一段经管道送来的）。"""
    name, arguments = None, []
    for _, _, name, arguments in wrapped_commands(words):
        pass
    if name == "cat":
        operands = [argument for argument in arguments if not argument.startswith("-") or argument == "-"]
        return tuple(stdin_bodies[-1:]) if all(operand == "-" for operand in operands) else ()
    if name == "echo":
        start = next((index for index, argument in enumerate(arguments) if not re.fullmatch(r"-[neE]+", argument)), len(arguments))
        return (shell_words.HeredocBody("<<", True, " ".join(arguments[start:]).replace("\\n", "\n").replace("\\t", "\t") + "\n"),)
    if name == "printf" and arguments:
        values = iter(arguments[1:])
        text = PRINTF_FORMAT_SLOT.sub(lambda _: next(values, ""), arguments[0]).replace("\\n", "\n").replace("\\t", "\t")
        return (shell_words.HeredocBody("<<", True, text),)
    return ()

def overwrite_steps(text, directory, variables=None, depth=0, every_write=False, write_scripts=False):
    """整条命令每一层（顶层、bash -c 的代码、喂给 shell 的 heredoc 正文、命令替换与进程替换）命令位置上，会整份覆盖文件的写法
    （OverwriteCandidate：重定向、tee、cp、mv、install、dd、truncate）与让旧字节留住的一步（PreservingStep），按执行的先后交回。
    喂给 shell 的 heredoc 正文留在原位按行切（共用模块的 strip_data_heredocs 只剥喂给别的命令的）；directory 是开头的当前目录。
    every_write（⑨ 与 ⑫ 用）另交追加与别的改法：>>、&>>、tee -a、sed -i、perl -i、rsync 的目标。
    write_scripts（⑫ 用）另交 KNOWN_WRITE_SCRIPTS 那几个定点写脚本要改的文件（OverwriteCandidate），目标读不出的交 UnknownScriptTarget。"""
    if depth > shell_words.MAXIMUM_NESTING:
        return []
    variables = dict(variables or {})
    steps = []
    data_heredocs = []
    heredoc_text = text if detector_break_is_set("heredoc-body-judged-as-commands") else shell_words.strip_data_heredocs(text, stripped_bodies=data_heredocs)
    pieces, piped = command_pieces(shell_words.shell_tokens(heredoc_text))
    carried = ()   # 前一段经管道交出的文本
    for piece, after_pipe in zip(pieces, piped):
        for is_operator, substitution in piece:
            if is_operator is None:
                steps += overwrite_steps(substitution, directory, variables, depth + 1, every_write, write_scripts)
        for (is_operator, operator), (next_is_operator, target) in zip(piece, piece[1:]):
            if (is_operator and (operator in OVERWRITING_REDIRECTS or (every_write and operator in APPENDING_REDIRECTS)) and next_is_operator is False
                    and not (operator == ">&" and FILE_DESCRIPTOR_WORD.match(target))):
                steps.append(OverwriteCandidate(operator, substitute_assigned_variables(target, variables), directory))
        commands, _, _, redirections = shell_words.simple_commands(piece)
        if commands and commands[0]:
            fed_heredocs = [data_heredocs[int(marker.group(1))] for operator, target in redirections[0] if operator in ("<<", "<<-")
                            for marker in [shell_words.HEREDOC_BODY_MARKER.match(target)] if marker and int(marker.group(1)) < len(data_heredocs)]
            stdin_bodies = fed_heredocs or (list(carried) if after_pipe and not detector_break_is_set("heredoc-pipe-fed-as-data") else [])
            directory = follow_simple_command(commands[0], directory, variables, steps, depth, stdin_bodies, every_write, write_scripts)
            carried = piped_output_bodies(commands[0], stdin_bodies)
        else:
            carried = ()
    return steps

def target_pattern(word, unknown_mark="*"):
    """目标词里这一刻算不出的段（没赋过值的变量、命令替换、花括号展开）换成 unknown_mark（默认 *）：交回 (样式, 有没有算不出的段, 有没有通配)。
    unknown_mark 给一个不会出现在路径里的字符时，样式里还分得出哪些是算不出的段、哪些是原本的通配。"""
    pieces, position, has_unknown_piece, has_glob = [], 0, False, False
    while position < len(word):
        if word.startswith("$(", position) or word[position] == "`":
            position = shell_words.substitution_span(word, position)[0]
            pieces.append(unknown_mark)
            has_unknown_piece = True
            continue
        unknown = UNKNOWN_PIECE.match(word, position)
        if unknown:
            pieces.append(unknown_mark)
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

GIT_LS_FILES_RUNS = [0]   # 起了几次 git ls-files，自证数它

def run_git_ls_files(repository_root, arguments):
    GIT_LS_FILES_RUNS[0] += 1
    return subprocess.run(["git", "-C", repository_root, "ls-files", *arguments], capture_output=True, text=True, timeout=20)

def git_tracks(repository_root, path):
    """`git ls-files --error-unmatch`：进过 git 交 True，没进交 False，git 自己出错（不是仓、路径在仓外、超时）交 None。
    一份一份查，只给弄坏开关 per-path-git-query 用；正常走 tracked_verdicts 一次查完。"""
    try:
        completed = run_git_ls_files(repository_root, ["--error-unmatch", "--", path])
    except (OSError, subprocess.TimeoutExpired):
        return None
    return {0: True, 1: False}.get(completed.returncode)

def tracked_names(repository_root, paths):
    """一次 git ls-files 查这几份文件，交回进了 git 的那些（规范化的绝对路径集合）；git 自己出错交 None。"""
    try:
        completed = run_git_ls_files(repository_root, ["-z", "--full-name", "--", *paths])
    except (OSError, subprocess.TimeoutExpired):
        return None
    if completed.returncode != 0:
        return None
    top = os.path.realpath(repository_root)
    return {os.path.normpath(os.path.join(top, name)) for name in completed.stdout.split("\0") if name}

def tracked_verdicts(repository_root, paths):
    """几份已有的文件各自进没进 git，一次 git ls-files 查完 → {路径: True（进了）/ False（没进）/ None（git 自己出错）}。
    符号链接按链接本身判（只解析它所在的目录），与 git ls-files --error-unmatch 同义。"""
    paths = list(dict.fromkeys(paths))
    if not paths:
        return {}
    if detector_break_is_set("per-path-git-query"):   # 弄坏开关：一份一份查
        return {path: git_tracks(repository_root, path) for path in paths}
    tracked = tracked_names(repository_root, paths)
    if tracked is None:
        return dict.fromkeys(paths)
    return {path: os.path.normpath(os.path.join(os.path.realpath(os.path.dirname(path)), os.path.basename(path))) in tracked for path in paths}

def untracked_among(repository_root, paths):
    """几份已有的文件里没进 git 的，一次 git ls-files 查完；git 自己出错交 None。"""
    if not paths:
        return []
    tracked = tracked_names(repository_root, paths)
    if tracked is None:
        return None
    return [path for path in paths if os.path.normpath(os.path.realpath(path)) not in tracked]

def results_overwrite_verdict(command, repository_root):
    """整条命令里会整份覆盖 research/results/ 下文件的每一步：目标确定、已存在、没进 git、前面也没被 git add 或 mv 挪走的，交进要拒的
    （`写法 相对仓库根的路径`）；目标这一刻算不出、样式对得上已有的未跟踪文件的，与 git 判不了的，交进要记检出的。"""
    results_directory = os.path.join(repository_root, *RESULTS_DIRECTORY_WORDS)
    preserved, refused, noted = [], [], []
    candidates = []   # 按出现的次序：("exact", 写法, 目标) 或 ("pattern", 写法, 原词, 前面没被 git add / mv 留住的对得上的文件)；留住与否在步骤出现那一刻判
    for step in overwrite_steps(command, repository_root):
        base = step.directory if step.directory is not None else repository_root
        if isinstance(step, PreservingStep):
            if step.kind in RESULTS_PRESERVING_KINDS:
                preserved += preserved_paths(step, base)
            continue
        if isinstance(step, PythonCode):
            continue
        for word in destination_words(step, base):
            exact, matches = resolve_destination(word, base, repository_root)
            if exact is not None:
                if is_inside(exact, results_directory) and os.path.isfile(exact) and not is_preserved(exact, preserved):
                    candidates.append(("exact", step.form, exact))
                continue
            candidates.append(("pattern", step.form, word, [match for match in matches if not is_preserved(match, preserved)]))
    verdicts = tracked_verdicts(repository_root, [item[2] for item in candidates if item[0] == "exact"])   # 确定的目标一次查完
    for item in candidates:
        if item[0] == "exact":
            _, form, exact = item
            tracked = verdicts[exact]
            relative = os.path.relpath(exact, repository_root)
            if tracked is False:
                refused.append(f"{form} {relative}")
            elif tracked is None:
                noted.append(f"判不了 {relative} 进没进 git（git ls-files 出错），{form} 覆盖它这一步没拦")
        else:
            _, form, word, remaining = item
            untracked = untracked_among(repository_root, remaining)
            if untracked is None:
                noted.append(f"判不了 {form} {word} 对得上的文件进没进 git（git ls-files 出错），没拦")
            elif untracked:
                shown = "、".join(os.path.relpath(path, repository_root) for path in untracked[:3])
                noted.append(f"{form} {word} 的目标这一刻算不出来（变量、命令替换或通配），没拦；"
                             f"它对得上 research/results/ 下 {len(untracked)} 份已有的未跟踪文件：{shown}{' 等' if len(untracked) > 3 else ''}")
    return list(dict.fromkeys(refused)), list(dict.fromkeys(noted))

def results_overwrite_refusal(command, repository_root):
    """交回 (要拒的写法与目标, 要记的检出)；前台、run_in_background 一样判，主 agent 与子 agent 都判。"""
    if (detector_break_is_set("disable-check")
            or detector_break_is_set("allow-results-overwrite")):
        return [], []
    return results_overwrite_verdict(command, repository_root)

# ⑯ 往现行账本直接追加节标题（文件头 ⑯）：登记表读法与 write-guard.sh 第十道各写一份，节标题的写法同门禁 ledger-section-numbers 格
LEDGER_REGISTRY_PATH = os.path.join(".claude", "gate.d", "split-proposal-ledgers.tsv")
LEDGER_HEADING_TEXT = re.compile(r"##[ \t]*[零〇一二两三四五六七八九十百千]+、")
LEDGER_WRITING_FORMS = OVERWRITING_REDIRECTS | APPENDING_REDIRECTS | {"tee"}

def current_ledger_relative(repository_root):
    """登记表里 current 那一行点名的账本 → (相对仓根的路径, None)；表读不到或 current 不是恰好一行 → (None, 原因)。"""
    registry = os.path.join(repository_root, LEDGER_REGISTRY_PATH)
    try:
        with open(registry, encoding="utf-8") as handle:
            rows = [line.rstrip("\n").split("\t") for line in handle if line.strip() and not line.startswith("#")]
    except OSError as error:
        return None, f"读不到 {registry}（{type(error).__name__}: {error}）"
    current = [parts[1] for parts in rows if len(parts) == 4 and parts[0] == "current"]
    if len(current) != 1:
        return None, f"{registry} 里 current 的行有 {len(current)} 行，要恰好一行"
    return current[0], None

def lands_on_ledger(path, base, ledger):
    """路径（相对的从 base 起算，~ 展开）解开符号链接之后是不是那份账本。"""
    expanded = os.path.expanduser(path)
    return os.path.realpath(expanded if os.path.isabs(expanded) else os.path.join(base, expanded)) == ledger

def ledger_append_refusal(command, repository_root, start_directory):
    """⑯：交回 (要拒的写法与目标, 要记的检出)。命令里没有节标题的不读登记表；弄坏开关 allow-ledger-append 下一概放行。"""
    if detector_break_is_set("allow-ledger-append") or not LEDGER_HEADING_TEXT.search(command):
        return [], []
    relative, problem = current_ledger_relative(repository_root)
    if relative is None:
        return [], [f"判不了往现行账本追加节标题：{problem}，没拦"]
    ledger = os.path.realpath(os.path.join(repository_root, relative))
    refused = []
    for step in overwrite_steps(command, start_directory, every_write=True):
        base = step.directory if getattr(step, "directory", None) is not None else repository_root
        if isinstance(step, PythonCode):
            if LEDGER_HEADING_TEXT.search(step.code):
                refused += [f"{write.form} {relative}" for write in python_in_place_writes(step.code, every_write=True) if lands_on_ledger(write.path, base, ledger)]
        elif isinstance(step, OverwriteCandidate) and step.form in LEDGER_WRITING_FORMS:
            pattern, has_unknown_piece, has_glob = target_pattern(os.path.expanduser(step.target))
            if not has_unknown_piece and not has_glob and lands_on_ledger(pattern, base, ledger):
                refused.append(f"{step.form} {relative}")
    return list(dict.fromkeys(refused)), []

# ⑲ 往 records/ 下的 .md 追加（文件头 ⑲）：追加只经 scripts/kb/append-record.py，它写之前照同一份判据库判要追加的文本；这里不看内容，认出追加就拒
RECORDS_DIRECTORY_NAME = "records"
RECORD_APPEND_TOOL = "scripts/kb/append-record.py"
RECORD_APPEND_UNLISTED_TOOL = "scripts/kb/history-consolidate.py"   # 弄坏开关 record-howto-unlisted-tool 下出路写成它：不在 lib_project_code_runs.py 的直接跑名单上，照抄会被 project-code-guard.sh 拒
RECORD_HOWTO_TEXT_PLACEHOLDER = "<草稿目录里写好的文本文件>"   # 出路里文本文件那一处的占位，自证照抄出路喂 project-code-guard.sh 时换成真文件
# ⑲ 出路喂给同一触发点上别的闸（project-code-guard.sh）时用的环境：本钩子这一趟自证设着的弄坏开关（名字以 _BREAK 收尾的变量，含 LIB_SHELL_WORDS_BREAK 这类共用库的）不带过去。
# 别的钩子见到不是它自己自证设的弄坏开关就拒（退 2），出路过不了闸是被这个开关挡的，不是出路写错。弄坏开关 howto-guard-environment-keeps-breaks 下原样带过去
def howto_guard_environment(base, detections_path):
    environment = dict(base, AGENT_HOOK_DETECTIONS=detections_path)
    if detector_break_is_set("howto-guard-environment-keeps-breaks"):
        return environment
    return {name: value for name, value in environment.items() if not name.endswith("_BREAK")}

RECORD_HOWTO_COMMAND = re.compile(r"→ 怎么办：追加的文本先写进草稿目录里的文件，再 `([^`]*)`")   # 自证从拒绝说明里取出路的整条写法

def record_howto_tool():
    """⑲ 出路里写的追加工具；弄坏开关 record-howto-unlisted-tool 下换成不在直接跑名单上的脚本。"""
    return RECORD_APPEND_UNLISTED_TOOL if detector_break_is_set("record-howto-unlisted-tool") else RECORD_APPEND_TOOL
PYTHON_OPENING_MODE = re.compile(r"^python (?:open\(…, |Path\.open\()'([^']*)'\)$")   # python_in_place_writes 报的 open 写法，引号里是模式

def record_append_landing(path, directory, repository_root):
    """追加的落点（python 里的字面路径，或 shell 目标词算出的样式）落在本仓 records/ 下一份 .md（后缀不分大小写，子目录里的也算）上就交它相对仓根的路径，否则交 None。
    相对路径从 directory 起算，directory 是 None（cd 算不出）的交 None；落点规整 .. 并解开符号链接，与同样解开链接的 <仓根>/records 比
    （records 同名前缀的兄弟目录不算；仓根下的 records 是链接的，比的是它指到的目录，hook-swap 的草稿树就是这样）。"""
    if directory is None and not os.path.isabs(path):
        return None
    records = os.path.realpath(os.path.join(repository_root, RECORDS_DIRECTORY_NAME))
    landing = os.path.realpath(path if os.path.isabs(path) else os.path.join(directory, path))
    if not landing.lower().endswith(".md") or not landing.startswith(records + os.sep):
        return None
    return os.path.join(RECORDS_DIRECTORY_NAME, os.path.relpath(landing, records))

def record_append_refusal(command, repository_root, start_directory):
    """⑲：交回要拒的 [(写法, 相对仓根的目标)]，去重、按出现次序。目标这一刻算不出的不判（文件头 ⑲「判不了的」）。
    弄坏开关 allow-record-append 下一概放行；record-append-python-unseen 下不认 python 的 open(…, 'a')；record-append-tee-unseen 下不认 tee -a。"""
    if detector_break_is_set("allow-record-append"):
        return []
    refused = []
    for step in overwrite_steps(command, start_directory, every_write=True):
        if isinstance(step, PythonCode):
            if detector_break_is_set("record-append-python-unseen"):
                continue
            for write in python_in_place_writes(step.code, every_write=True):
                mode = PYTHON_OPENING_MODE.match(write.form)
                landing = record_append_landing(write.path, step.directory, repository_root) if mode and "a" in mode.group(1) else None
                if landing:
                    refused.append((write.form, landing))
        elif isinstance(step, OverwriteCandidate):
            if step.form in APPENDING_REDIRECTS:
                form = step.form
            elif step.form == "tee" and step.appends and not detector_break_is_set("record-append-tee-unseen"):
                form = "tee -a"
            else:
                continue
            pattern, has_unknown_piece, has_glob = target_pattern(os.path.expanduser(step.target))
            landing = None if has_unknown_piece or has_glob else record_append_landing(pattern, step.directory, repository_root)
            if landing:
                refused.append((form, landing))
    return list(dict.fromkeys(refused))

def repository_root_of(hook_dir):
    """这个 hook 所在的仓：`.claude/hooks/` 往上两级。"""
    return os.path.realpath(os.path.join(hook_dir, os.pardir, os.pardir))

# ⑰ 往仓里的文件写字面内容：会话号、钟点、批次代号照写闸 write-guard.sh 第五道、第七道同一份判据库（../gate.d/lib-forbidden-notations.py）判，命中就拒（文件头 ⑰）
LITERAL_NOTATION_FORMS = (("session-names", "会话号"), ("clock-times", "钟点"), ("batch-codes", "批次代号"))
LITERAL_NOTATION_LISTED = 20          # 拒绝说明里逐处列出的上限，多出的只报处数
NOTATIONS_LIBRARY = {}                # 库路径 → 装好的模块：一个进程里只装一次

class LiteralWrite(NamedTuple):
    """一条命令往一个目标里写的字面内容：写法（「cat >>」「echo >」「tee -a」这类）、目标词（前面赋过的变量已代入）、那一刻的当前目录（算不出是 None）与内容。"""
    form: str
    target: str
    directory: str | None
    text: str

def notations_library_path(hook_dir):
    return os.path.join(os.path.dirname(hook_dir), "gate.d", "lib-forbidden-notations.py")

def load_notations_library(library_path):
    """写闸、门禁 doc-text 与这一道共用的判据库：装过的直接交回；读不到、语法错抛异常（由 main() 记判不了）。不留 __pycache__。"""
    library = NOTATIONS_LIBRARY.get(library_path)
    if library is not None:
        return library
    specification = importlib.util.spec_from_file_location("lib_forbidden_notations", library_path)
    library = importlib.util.module_from_spec(specification)
    before = sys.dont_write_bytecode
    sys.dont_write_bytecode = True
    try:
        specification.loader.exec_module(library)
    finally:
        sys.dont_write_bytecode = before
    NOTATIONS_LIBRARY[library_path] = library
    return library

def literal_output(words, stdin_bodies):
    """一条命令写到标准输出的字面内容：cat（没有文件操作数，或只有 -）交它读到的 heredoc、here-string 或管道前一段的字面；echo 交参数（空格接起来）；
    printf 交格式串与每个参数各占一行（格式重复几遍算不准，把参数逐个列进来，哪个参数里有都判得到）；别的命令交 None（输出算不出，不判）。"""
    name, arguments = None, []
    for _, _, name, arguments in wrapped_commands(words):
        pass
    if name == "cat":
        operands = [argument for argument in arguments if not argument.startswith("-") or argument == "-"]
        return stdin_bodies[-1].text if stdin_bodies and all(operand == "-" for operand in operands) else None
    if name == "echo":
        start = next((index for index, argument in enumerate(arguments) if not re.fullmatch(r"-[neE]+", argument)), len(arguments))
        return " ".join(arguments[start:]) + "\n"
    if name == "printf" and arguments:
        return "\n".join(arguments) + "\n"
    return None

def literal_writes(text, directory, variables=None, depth=0):
    """整条命令每一层（顶层、bash -c 的代码、喂给 shell 的 heredoc 正文、命令替换）命令位置上往文件里写字面内容的几处（LiteralWrite，按执行的先后）：
    cat / echo / printf 的输出经 >、>>、>|、&>、&>> 写进的，与 tee（带不带 -a）把 heredoc、here-string、管道前一段的字面写进的。cd 与赋值跟着算（follow_simple_command）。"""
    if depth > shell_words.MAXIMUM_NESTING:
        return []
    variables = dict(variables or {})
    writes, data_heredocs, carried = [], [], []
    pieces, piped = command_pieces(shell_words.shell_tokens(shell_words.strip_data_heredocs(text, stripped_bodies=data_heredocs)))
    for piece, after_pipe in zip(pieces, piped):
        for is_operator, substitution in piece:
            if is_operator is None:
                writes += literal_writes(substitution, directory, variables, depth + 1)
        commands, _, _, redirections = shell_words.simple_commands(piece)
        words = commands[0] if commands and commands[0] else []
        if not words:
            carried = []
            continue
        fed = [data_heredocs[int(marker.group(1))] for operator, target in redirections[0] if operator in ("<<", "<<-")
               for marker in [shell_words.HEREDOC_BODY_MARKER.match(target)] if marker and int(marker.group(1)) < len(data_heredocs)]
        fed += [shell_words.HeredocBody("<<<", True, target + "\n") for operator, target in redirections[0] if operator == "<<<"]
        stdin_bodies = fed or (carried if after_pipe else [])
        output = literal_output(words, stdin_bodies)
        name, arguments = None, []
        for _, _, name, arguments in wrapped_commands(words):
            pass
        if output is not None:
            for (is_operator, operator), (next_is_operator, target) in zip(piece, piece[1:]):
                if (is_operator and operator in OVERWRITING_REDIRECTS | APPENDING_REDIRECTS and next_is_operator is False
                        and not (operator == ">&" and FILE_DESCRIPTOR_WORD.match(target))):
                    writes.append(LiteralWrite(f"{name} {operator}", substitute_assigned_variables(target, variables), directory, output))
        if name == "tee" and stdin_bodies:
            form = "tee" if tee_files(arguments) else "tee -a"   # tee_files 对 -a / --append 交空表
            writes += [LiteralWrite(form, substitute_assigned_variables(argument, variables), directory, stdin_bodies[-1].text)
                       for argument in arguments if argument != "-" and not argument.startswith("-")]
        if name in shell_words.SHELL_NAMES:
            code = shell_words.shell_invocation(arguments).code_string
            if code is not None:
                writes += literal_writes(code, directory, variables, depth + 1)
            elif stdin_bodies and shell_words.heredoc_feeds_shell(shlex.join([name, *arguments])):
                writes += literal_writes(stdin_bodies[-1].text, directory, variables, depth + 1)
        directory = follow_simple_command(words, directory, variables, [], depth)
        carried = [shell_words.HeredocBody("|", True, output)] if output is not None else []
    return writes

def literal_write_landing(write):
    """这一处写的落点（绝对路径，解开符号链接）；目标词里有算不出的段、通配、循环变量，或当前目录算不出的，交 None（不判）。"""
    pattern, has_unknown_piece, has_glob = target_pattern(os.path.expanduser(write.target))
    if has_unknown_piece or has_glob or LOOP_VALUE_MARK in pattern:
        return None
    path = shell_words.resolve_path(write.directory, pattern)
    return os.path.realpath(path) if path else None

def literal_notation_hits(command, repository_root, start_directory, library_path):
    """⑰：这条命令往仓里的文件写的字面内容里，判据库三种形态（会话号、钟点、批次代号）的每一处：[(写法, 相对仓根的目标, 形态名, 行号, 命中的串)]。
    落在仓外、落点算不出、在那一形态的排除前缀下的不判；后缀与放行标记由库判（批次代号只判 .md、.tsv、.txt）。读不到库抛异常。
    弄坏开关 allow-literal-notations 下交空表；literal-notations-outside-judged 下仓外的目标也判（按文件名当相对路径）。"""
    if detector_break_is_set("allow-literal-notations"):
        return []
    root, hits = os.path.realpath(repository_root), []
    for write in literal_writes(command, start_directory):
        landing = literal_write_landing(write)
        if landing is None:
            continue
        inside = os.path.commonpath([landing, root]) == root
        if not inside and not detector_break_is_set("literal-notations-outside-judged"):
            continue
        relative = os.path.relpath(landing, root) if inside else os.path.basename(landing)
        library = load_notations_library(library_path)
        for form_key, form_name in LITERAL_NOTATION_FORMS:
            if library.is_excluded(form_key, relative):
                continue
            hits += [(write.form, relative, form_name, number, matched) for number, _, _, matched in library.all_hits(form_key, write.text, relative)]
    return hits

def literal_notation_remedies(hits, library_path):
    """拒绝说明的出路：命中的每种形态各一行，取库里那一形态登记的 remedy 第一条。"""
    library = load_notations_library(library_path)
    named = {form_name for _, _, form_name, _, _ in hits}
    return [f"{form_name}：{library.FORMS_BY_KEY[form_key]['remedy'][0]}" for form_key, form_name in LITERAL_NOTATION_FORMS if form_name in named]

# ⑦ 在同一个 inode 上改一个已经存在的脚本：shell 那一半沿用 ⑤ 的 overwrite_steps（同一份写法、cd 与变量），另判喂给 python 的代码
SCRIPT_SUFFIXES = (".sh", ".py")
INODE_REPLACING_FORMS = {"mv", "install"}               # 实测换 inode：已经打开着文件的读者接着读到旧内容
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

def python_in_place_writes(code, every_write=False):
    """python 代码里在原 inode 上写文件的调用：open(路径, 带 w 或 r+ 的模式)、Path(路径).open(同上)、Path(路径).write_text / write_bytes、
    shutil.copyfile / copy / copy2 的目标。路径与模式算不出的不判，解析不了的代码不判。
    every_write（⑨ 用）另交带 a、x、+ 的模式与换 inode 的写：shutil.move、os.replace、os.rename 的目标。"""
    try:
        tree = ast.parse(code)
    except (SyntaxError, ValueError):
        return []
    assignments = single_assignments(tree)
    writes = []
    mode_writes = (lambda mode: mode is not None and any(letter in mode for letter in "wax+")) if every_write else mode_writes_in_place
    for node in ast.walk(tree):
        if not isinstance(node, ast.Call):
            continue
        name, owner = called_name_and_owner(node.func)
        receiver = node.func.value if isinstance(node.func, ast.Attribute) else None
        if name == "open" and (receiver is None or (isinstance(receiver, ast.Name) and receiver.id in PYTHON_OPEN_MODULES)):
            mode = opening_mode(call_argument(node, 1, "mode"))
            path = python_literal_path(call_argument(node, 0, "file"), assignments)
            if path is not None and mode_writes(mode):
                writes.append(PythonWrite(f"python open(…, '{mode}')", path, None))
        elif name == "open":
            mode = opening_mode(call_argument(node, 0, "mode"))
            path = python_literal_path(receiver, assignments)
            if path is not None and mode_writes(mode):
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
        elif every_write and ((name == "move" and owner == "shutil") or (name in ("replace", "rename") and owner == "os")):
            path = python_literal_path(call_argument(node, 1, "dst"), assignments)
            source = python_literal_path(call_argument(node, 0, "src"), assignments)
            if path is not None:
                writes.append(PythonWrite(f"python {owner}.{name}", path, source))
    return writes

def is_existing_script(path, repository_root, scratch_root):
    """path 是仓里或 scratch_root（一个根，或几个根的元组）下一个已经存在的脚本：.sh、.py，或带执行位的普通文件（符号链接按它指向的文件算）。
    弄坏开关 in-place-script-session-temporary-root-unseen 下会话临时目录根不算。"""
    roots = (scratch_root,) if isinstance(scratch_root, str) else tuple(scratch_root)
    if work_roots and detector_break_is_set("in-place-script-session-temporary-root-unseen"):
        roots = tuple(root for root in roots if root != work_roots.claude_session_temporary_root())
    if not (is_inside(path, repository_root) or any(is_inside(path, root) for root in roots)):
        return False
    real = os.path.realpath(path)
    if not os.path.isfile(real):
        return False
    return path.endswith(SCRIPT_SUFFIXES) or real.endswith(SCRIPT_SUFFIXES) or bool(os.stat(real).st_mode & 0o111)

def script_in_place_verdict(command, repository_root, scratch_root=SCRIPT_SCRATCH_ROOTS):
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

def script_in_place_refusal(command, repository_root, scratch_root=SCRIPT_SCRATCH_ROOTS):
    """交回要拒的写法与目标；前台、run_in_background 一样判，主 agent 与子 agent 都判。"""
    if (detector_break_is_set("disable-check")
            or detector_break_is_set("allow-script-in-place-write")):
        return []
    return script_in_place_verdict(command, repository_root, scratch_root)

# ⑥ 终止进程只许点名一个自己起的进程号或任务号；同一份判定也交给门禁 code-tooling 的 research-gate-lint 格扫脚本（--scan-scripts），hook 与门禁不各写一份
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
# SSH 会话、登录会话、用户级 systemd 与它们依赖的系统服务；本地模型服务的单元另从私有配置读（private_words）；systemctl 不写后缀的单元按 .service 补上再比
PROTECTED_UNIT = re.compile(r"^(?:sshd?(?:@[^/]*)?(?:\.service|\.socket)|session-[^/]*\.scope|user@[^/]*\.service|user(?:-[^/]*)?\.slice"
                            r"|-\.slice|init\.scope|systemd-logind\.service|dbus(?:-broker)?\.(?:service|socket))$")

def private_config_path():
    configured = os.environ.get("BASH_COMMAND_DETECTOR_PRIVATE_CONFIG")
    return configured or os.path.join(os.environ.get("XDG_CONFIG_HOME") or os.path.join(os.path.expanduser("~"), ".config"),
                                      "singlefs", "bash-command-detector.env")

def private_words(key):
    """私有配置里 key 的值按空白与逗号切开的词；文件不在、读不了、没有这个键、弄坏开关设着，都交回空表（那一项不判、不报错）。"""
    if detector_break_is_set("ignore-private-config"):
        return []
    try:
        with open(private_config_path(), encoding="utf-8") as handle:
            lines = handle.read().splitlines()
    except (OSError, UnicodeDecodeError):
        return []
    words = []
    for line in lines:
        name, separator, value = line.strip().partition("=")
        if line.strip().startswith("#") or not separator or name.strip() != key:
            continue
        words += [word for word in re.split(r"[\s,]+", value.strip().strip("'\"")) if word]
    return words

def unit_with_suffix(unit):
    return unit if "." in os.path.basename(unit) else unit + ".service"
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

class ProcessTableOnDemand:
    """进程表与发这条命令的会话自己的祖先：第一次有人要才读（判 kill / proc.py stop 的写死的进程号打到谁时才要），读过的不再读。
    process_target_findings 认它：有它就按需取，是字典就照旧用字典与 own_ancestors。"""
    def __init__(self, reader, own_process_id=None):
        self.reader, self.own_process_id, self.loaded = reader, own_process_id, None

    def load(self):
        """→ (进程表, 这个会话自己的祖先)。"""
        if self.loaded is None:
            table = self.reader()
            self.loaded = (table, ancestors_in(table, self.own_process_id or os.getpid()))
        return self.loaded

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
    if any(marker in information["command_line"] for marker in private_words("LOCAL_MODEL_SERVICE_PROCESS_MARKERS")):
        return f"{label}是本地模型服务的进程（命令行里有私有配置登记的标记）"
    own_sessions = {ancestor for ancestor in own_ancestors if table.get(ancestor, {}).get("name") == "claude"}
    for ancestor in ancestors_in(table, information["parent"]):
        if table[ancestor]["name"] == "claude" and ancestor not in own_sessions:
            return f"{label}是另一个 Claude 会话（进程 {ancestor}）起的"
    return None

def process_target_findings(words, targets, in_loop, what, process_table, own_ancestors, own_group_variables=frozenset()):
    """kill 与 proc.py stop 共用：目标个数、每个目标的形状、写死的进程号打到谁、是不是在循环里。
    own_group_variables 是这一层里记着「自己用 setsid 起的后台任务的进程号」的变量名（scan_shell_text 记）：`-$名字` 打的是那一组，不算带负号。"""
    findings = []
    if len(targets) > 1:
        findings.append(TerminationFinding(tuple(words), f"{what} 一次给了 {len(targets)} 个目标（{' '.join(targets)}）：一次只停点名的一个", "other"))
    for target in targets:
        own_group = OWN_GROUP_TARGET.match(target)
        if own_group and own_group.group(1) in own_group_variables and not detector_break_is_set("own-session-group-unrecognized"):
            continue
        problem = target_problem(target)
        if problem:
            kind = "cgroup" if PROCESS_LIST_READING.search(target) or any(picker in target for picker in PROCESS_PICKERS) else "other"
            findings.append(TerminationFinding(tuple(words), f"{what}：{problem}", kind))
        elif process_table is not None and LITERAL_PROCESS_ID.match(target):
            table, ancestors = process_table.load() if isinstance(process_table, ProcessTableOnDemand) else (process_table, own_ancestors)
            reason = protected_process_reason(int(target), table, ancestors)
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
    private_units = {unit_with_suffix(unit) for unit in private_words("LOCAL_MODEL_SERVICE_UNITS")}
    for unit in units[:1] if verb == "set-property" else units:
        bare = EXPANSION.sub("", unit)
        named = unit_with_suffix(unit)
        if "$(" in unit or "`" in unit or WHOLE_LIST_EXPANSION.search(unit):
            findings.append(TerminationFinding(tuple(words), f"systemctl {verb} 的单元 {unit} 是命令替换或整批展开：一次停一批，是谁的这一刻不知道", "other"))
        elif any(character in bare for character in "*?[") or BRACE_LIST.search(bare):
            findings.append(TerminationFinding(tuple(words), f"systemctl {verb} 的单元 {unit} 带通配：一次命中一批单元，别的会话的也算", "other"))
        elif PROTECTED_UNIT.match(named) or named in private_units:
            findings.append(TerminationFinding(tuple(words), f"systemctl {verb} {unit}：这是 SSH、登录会话、用户级 systemd、本地模型服务或它们依赖的系统服务", "other"))
    return findings

def call_findings(words, in_loop, process_table, own_ancestors, own_group_variables=frozenset()):
    """一条简单命令（剥开包装之后）里的终止写法；python -c 的代码另判。交回 (findings, 这一条是不是在给进程发终止信号)。
    own_group_variables 只给 kill 用（process_target_findings）：proc.py stop 一次停一个进程，不认进程组。"""
    name, arguments = innermost_call(words)
    if not name:
        return [], False
    if name == "kill":
        signal_name, targets = kill_arguments(arguments)
        if targets is None or signal_name in SIGNAL_ZERO_NAMES or not targets:
            return [], False
        return process_target_findings(words, targets, in_loop, "kill", process_table, own_ancestors, own_group_variables), True
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

# 放行一种进程组：自己用新会话起的那个子进程的组（上游 .claude/singlefs-ai-sop/rules/command-safety.md「装置起的样本进程，放进它自己的进程组」：
#   每条样本 start_new_session=True 起、超时与结束时对整组发信号）。认得的写法：
#   - 子进程：一个名字，它在自己所在的作用域里每一处绑定都是带 start_new_session=True（或 process_group=0）的 subprocess.Popen、
#     asyncio.create_subprocess_exec / create_subprocess_shell（带 await 的也算），或 `with <那样的调用> as 名字`，
#     或调同一份代码里的一个函数、那个函数（同样是每一处绑定都是这一个 def）自己这一层的每条 return 都交回那样起的子进程；
#   - 进程组号：<子进程>.pid、os.getpgid(<子进程>.pid)，或一个每一处绑定都是这两样之一的名字；
#   - os.killpg(<进程组号>, …) 与 os.kill(-<进程组号>, …) 放行，在循环里调也放行（发的是那一个子进程的组）。
#   照旧拒：参数、循环变量、属性（self.process.pid）、改绑过的名字、0、负的写死号、os.getpgid(0)、从别处读来的进程号；
#   同一个函数里读 cgroup.procs 或 /proc 的那一条另判，目标是自己起的也照报。
SESSION_CHILD_CALLS = {"Popen", "create_subprocess_exec", "create_subprocess_shell"}
PYTHON_SCOPE_NODES = (ast.Module, ast.FunctionDef, ast.AsyncFunctionDef, ast.Lambda, ast.ClassDef,
                      ast.ListComp, ast.SetComp, ast.DictComp, ast.GeneratorExp)
OWN_SESSION_RESOLUTION_DEPTH = 6   # 名字套名字、函数套函数往下追的层数上限；绕回自己的写法追到这里就当认不出

def python_scope_bindings(scope):
    """一个作用域（模块、函数、lambda、类体、推导式）自己这一层的名字绑定，按源码次序：{名字: [(种类, 节点)]}；
    种类 value（赋进去的值节点）、def（函数定义节点）、other（参数、循环变量、import、except、global 这类说不出值的）。
    不进嵌套的作用域；推导式里的海象赋值绑在外层，记成 other。"""
    bindings = {}
    def bind(name, kind, node):
        bindings.setdefault(name, []).append((kind, node))
    def bind_target(target, value):
        if isinstance(target, ast.Name):
            bind(target.id, "value" if value is not None else "other", value)
        elif isinstance(target, (ast.Tuple, ast.List)):
            for element in target.elts:
                bind_target(element, None)
        elif isinstance(target, ast.Starred):
            bind_target(target.value, None)
    def walk(node):
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
            bind(node.name, "def", node)
            return
        if isinstance(node, ast.ClassDef):
            bind(node.name, "other", None)
            return
        if isinstance(node, ast.Lambda):
            return
        if isinstance(node, (ast.ListComp, ast.SetComp, ast.DictComp, ast.GeneratorExp)):
            for inner in ast.walk(node):
                if isinstance(inner, ast.NamedExpr) and isinstance(inner.target, ast.Name):
                    bind(inner.target.id, "other", None)
            return
        if isinstance(node, ast.Assign):
            for target in node.targets:
                bind_target(target, node.value)
        elif isinstance(node, ast.AnnAssign) and node.value is not None:
            bind_target(node.target, node.value)
        elif isinstance(node, ast.AugAssign):
            bind_target(node.target, None)
        elif isinstance(node, ast.NamedExpr) and isinstance(node.target, ast.Name):
            bind(node.target.id, "value", node.value)
        elif isinstance(node, (ast.For, ast.AsyncFor)):
            bind_target(node.target, None)
        elif isinstance(node, (ast.With, ast.AsyncWith)):
            for item in node.items:
                if isinstance(item.optional_vars, ast.Name):
                    bind(item.optional_vars.id, "value", item.context_expr)
                elif item.optional_vars is not None:
                    bind_target(item.optional_vars, None)
        elif isinstance(node, ast.ExceptHandler) and node.name:
            bind(node.name, "other", None)
        elif isinstance(node, (ast.Import, ast.ImportFrom)):
            for alias in node.names:
                bind((alias.asname or alias.name).split(".")[0], "other", None)
        elif isinstance(node, (ast.Global, ast.Nonlocal)):
            for name in node.names:
                bind(name, "other", None)
        elif isinstance(node, ast.Delete):
            for target in node.targets:
                bind_target(target, None)
        elif type(node).__name__ in ("MatchAs", "MatchStar") and getattr(node, "name", None):
            bind(node.name, "other", None)
        elif type(node).__name__ == "MatchMapping" and getattr(node, "rest", None):
            bind(node.rest, "other", None)
        for child in ast.iter_child_nodes(node):
            walk(child)
    if isinstance(scope, (ast.FunctionDef, ast.AsyncFunctionDef, ast.Lambda)):
        arguments = scope.args
        for argument in [*arguments.posonlyargs, *arguments.args, *arguments.kwonlyargs, arguments.vararg, arguments.kwarg]:
            if argument is not None:
                bind(argument.arg, "other", None)
        for statement in (scope.body if isinstance(scope.body, list) else [scope.body]):
            walk(statement)
    elif isinstance(scope, (ast.ListComp, ast.SetComp, ast.DictComp, ast.GeneratorExp)):
        for generator in scope.generators:
            bind_target(generator.target, None)
    else:
        for statement in scope.body:
            walk(statement)
    return bindings

def function_returns(definition):
    """函数自己这一层的 return 交回的表达式（裸 return 交 None）；lambda 交它的式子；生成器（有 yield）交 []。"""
    if isinstance(definition, ast.Lambda):
        return [definition.body]
    values, generator = [], False
    def walk(node):
        nonlocal generator
        for child in ast.iter_child_nodes(node):
            if isinstance(child, PYTHON_SCOPE_NODES):
                continue
            if isinstance(child, ast.Return):
                values.append(child.value)
            elif isinstance(child, (ast.Yield, ast.YieldFrom)):
                generator = True
            walk(child)
    walk(definition)
    return [] if generator else values

class OwnSessionGroups:
    """一段 python 代码里认「自己用新会话起的子进程的进程组」（上面 SESSION_CHILD_CALLS 那段注释）。scope_chain 是从模块到当前作用域的节点列表。
    弄坏开关：own-session-group-unrecognized 一样都不认；own-session-group-session-unchecked 不看 start_new_session 与 process_group，
    凡 Popen 这一类起的都认；own-session-group-first-binding-only 名字只看第一处绑定，后面改绑的看不见。"""
    def __init__(self):
        self.bindings_by_scope = {}

    def bindings(self, scope):
        if id(scope) not in self.bindings_by_scope:
            self.bindings_by_scope[id(scope)] = python_scope_bindings(scope)
        return self.bindings_by_scope[id(scope)]

    def resolve(self, name, scope_chain):
        """名字绑在哪一层：交 (到那一层为止的作用域链, 它在那一层的全部绑定)；哪一层都没绑（内置、别处来的）交 None。类体里绑的只在类体自己里看得见。"""
        innermost = len(scope_chain) - 1
        for index in range(innermost, -1, -1):
            scope = scope_chain[index]
            if isinstance(scope, ast.ClassDef) and index != innermost:
                continue
            entries = self.bindings(scope).get(name)
            if entries:
                if detector_break_is_set("own-session-group-first-binding-only"):
                    entries = entries[:1]
                return scope_chain[:index + 1], entries
        return None

    @staticmethod
    def starts_own_session(call):
        function = call.func
        name = function.attr if isinstance(function, ast.Attribute) else function.id if isinstance(function, ast.Name) else None
        if name not in SESSION_CHILD_CALLS:
            return False
        if detector_break_is_set("own-session-group-session-unchecked"):
            return True
        return any(isinstance(keyword.value, ast.Constant) and (
                       (keyword.arg == "start_new_session" and keyword.value.value is True)
                       or (keyword.arg == "process_group" and type(keyword.value.value) is int and keyword.value.value == 0))
                   for keyword in call.keywords)

    def is_own_child(self, node, scope_chain, depth=0):
        """node 这个表达式的值是自己用新会话起的子进程。"""
        if depth > OWN_SESSION_RESOLUTION_DEPTH:
            return False
        if isinstance(node, ast.Await):
            node = node.value
        if isinstance(node, ast.Name):
            found = self.resolve(node.id, scope_chain)
            return found is not None and all(kind == "value" and self.is_own_child(value, found[0], depth + 1) for kind, value in found[1])
        if not isinstance(node, ast.Call):
            return False
        if self.starts_own_session(node):
            return True
        if not isinstance(node.func, ast.Name):
            return False
        found = self.resolve(node.func.id, scope_chain)
        if found is None or not all(kind == "def" for kind, _ in found[1]):
            return False
        for _, definition in found[1]:
            returns = function_returns(definition)
            if not returns or not all(value is not None and self.is_own_child(value, found[0] + [definition], depth + 1) for value in returns):
                return False
        return True

    def is_own_group(self, node, scope_chain, through_names=True):
        """node 是自己用新会话起的子进程的进程组号：<子进程>.pid、os.getpgid(<子进程>.pid)，或每一处绑定都是这两样之一的名字。"""
        if detector_break_is_set("own-session-group-unrecognized"):
            return False
        if isinstance(node, ast.Attribute) and node.attr == "pid":
            return self.is_own_child(node.value, scope_chain)
        if isinstance(node, ast.Call) and len(node.args) == 1 and not node.keywords:
            function = node.func
            asks_group = ((isinstance(function, ast.Attribute) and function.attr == "getpgid" and isinstance(function.value, ast.Name) and function.value.id == "os")
                          or (isinstance(function, ast.Name) and function.id == "getpgid"))
            argument = node.args[0]
            return asks_group and isinstance(argument, ast.Attribute) and argument.attr == "pid" and self.is_own_child(argument.value, scope_chain)
        if isinstance(node, ast.Name) and through_names:
            found = self.resolve(node.id, scope_chain)
            return found is not None and all(kind == "value" and self.is_own_group(value, found[0], through_names=False) for kind, value in found[1])
        return False

def python_termination_findings(code):
    """python 代码里的终止写法：os.killpg、os.kill 的目标是 0 或负数、在循环或推导式里调、同一个函数里读 cgroup.procs 或 /proc 再调。解析不了的不判。
    发给自己用新会话起的那个子进程的组的放行（OwnSessionGroups）。"""
    try:
        tree = ast.parse(code)
    except (SyntaxError, ValueError):
        return []
    findings = []
    groups = OwnSessionGroups()
    def reads_process_list(scope):
        return any(isinstance(node, ast.Constant) and isinstance(node.value, str) and PROCESS_LIST_READING.search(node.value)
                   for node in ast.walk(scope))
    def visit(node, loop_depth, scope_reads, scope_chain):
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef, ast.Lambda)):
            loop_depth, scope_reads = 0, reads_process_list(node)
        if isinstance(node, PYTHON_SCOPE_NODES) and node is not scope_chain[-1]:
            scope_chain = scope_chain + [node]
        if isinstance(node, ast.Call):
            function = node.func
            owner = function.value.id if isinstance(function, ast.Attribute) and isinstance(function.value, ast.Name) else None
            called = function.attr if isinstance(function, ast.Attribute) else function.id if isinstance(function, ast.Name) else None
            probe = len(node.args) >= 2 and isinstance(node.args[1], ast.Constant) and node.args[1].value == 0
            if called in PYTHON_SIGNAL_CALLS and owner in ("os", "signal", None) and not probe:
                first = node.args[0] if node.args else None
                negative = (isinstance(first, ast.UnaryOp) and isinstance(first.op, ast.USub)) or (
                    isinstance(first, ast.Constant) and isinstance(first.value, int) and first.value <= 0)
                own_group = first is not None and (
                    (called == "killpg" and groups.is_own_group(first, scope_chain))
                    or (called == "kill" and isinstance(first, ast.UnaryOp) and isinstance(first.op, ast.USub)
                        and groups.is_own_group(first.operand, scope_chain)))
                if called == "killpg":
                    if not own_group:
                        findings.append(TerminationFinding((), "python 里 os.killpg 发给整个进程组", "other", code, node.lineno))
                elif negative and not own_group:
                    findings.append(TerminationFinding((), "python 里 os.kill 的目标是 0 或负数（整个进程组或全部进程）", "other", code, node.lineno))
                if loop_depth and not own_group:
                    findings.append(TerminationFinding((), f"python 里在循环或推导式里调 {called}：这是批量停", "loop", code, node.lineno))
                if scope_reads:
                    findings.append(TerminationFinding((), f"python 里同一个函数先读 cgroup.procs 或 /proc 挑进程、再调 {called}", "cgroup", code, node.lineno))
        for child in ast.iter_child_nodes(node):
            visit(child, loop_depth + (1 if isinstance(node, PYTHON_LOOP_NODES) and child is not getattr(node, "iter", None) else 0), scope_reads, scope_chain)
    visit(tree, 0, reads_process_list(tree), [tree])
    return findings

# shell 这一侧放行的进程组：同一层里先用命令位置上的 setsid（不带 -f / --fork）放后台起一条（`setsid --wait <命令> &`），紧跟着 `名字=$!`
#   （前面可以挂 local、declare 这类）记下它的进程号，之后 `kill -<信号> -- -"$名字"`（`-$名字`、`-${名字}` 同）打的就是这个新会话的组。
#   按文本次序认：先 kill 后赋值的、赋值之前最近一次放后台的不是 setsid 起的、之后又被别的赋值、read、for 的循环变量、unset 改过的，照旧按带负号拒；
#   kill 0、kill -1、负的写死号、别的变量照旧拒。弄坏开关同 OwnSessionGroups 的三条：unrecognized 不认、session-unchecked 不看 setsid、
#   first-binding-only 后面的改绑看不见。
OWN_GROUP_TARGET = re.compile(r"^-\$\{?([A-Za-z_][A-Za-z0-9_]*)\}?$")
SHELL_VARIABLE_BINDING = re.compile(r"^([A-Za-z_][A-Za-z0-9_]*)(?:\[[^\]]*\])?\+?=(.*)$", re.S)
DECLARATION_BUILTINS = {"local", "declare", "typeset", "readonly", "export"}
VARIABLE_READING_BUILTINS = {"read", "unset", "mapfile", "readarray"}
LAST_BACKGROUND_JOB = ("$!", "${!}")

def setsid_started_without_fork(words):
    """这条简单命令是经命令位置上的 setsid 起的，setsid 不带 -f / --fork（带了的那一份 fork 出去，$! 就不是新会话的组长）。"""
    skipped = shell_words.skip_prefixes(words)
    if "setsid" not in skipped.prefix_names:
        return False
    end = skipped.command_position if skipped.command_position is not None else len(words)
    for position, word in enumerate(words[:end]):
        if os.path.basename(word) != "setsid":
            continue
        for option in words[position + 1:end]:
            if not option.startswith("-") or option == "--":
                break
            if option == "--fork" or (not option.startswith("--") and "f" in option[1:]):
                return False
        return True
    return False

def update_own_group_variables(words, own_group_variables, last_background_own_session):
    """一条不放后台的简单命令改了哪些变量：`名字=$!` 在最近一次放后台的是 setsid 起的时记进 own_group_variables；
    别的赋值、read、for 的循环变量、unset 把那个名字拿掉。挂在命令前面的赋值只给那一条命令，不改 shell 里的变量。"""
    position = 0
    while position < len(words) and words[position] in shell_words.COMMAND_KEYWORDS:
        position += 1
    rest = words[position:]
    if not rest:
        return
    rebound = set()
    if rest[0] == "for" and len(rest) > 1:
        rebound.add(rest[1])
    elif rest[0] in VARIABLE_READING_BUILTINS:
        rebound.update(word for word in rest[1:] if not word.startswith("-"))
    else:
        candidates = rest[1:] if rest[0] in DECLARATION_BUILTINS else rest
        if rest[0] not in DECLARATION_BUILTINS and not all(SHELL_VARIABLE_BINDING.match(word) for word in candidates):
            return
        for word in candidates:
            binding = SHELL_VARIABLE_BINDING.match(word)
            if binding is None and word.startswith("-"):
                continue
            name = binding.group(1) if binding else word
            if last_background_own_session and word in tuple(f"{name}={value}" for value in LAST_BACKGROUND_JOB):
                own_group_variables.add(name)
            else:
                rebound.add(name)
    if not detector_break_is_set("own-session-group-first-binding-only"):
        own_group_variables.difference_update(rebound)

def scan_shell_text(text, process_table, own_ancestors, depth):
    """termination_findings 的本体：交回 (发现, 有没有给进程发终止信号, 有没有 pgrep / pidof 挑进程)。
    包装认不出的 `bash -c '…'`（前面是 shell 函数、变量里的脚本）另按一段完整的代码递归判。"""
    findings, sends_signal, picks_processes, python_codes = [], False, False, []
    for layer in shell_layers(text):
        boundaries = []
        commands, _, _, redirections = shell_words.simple_commands(layer.tokens, boundaries=boundaries)
        loop_depth = 0
        own_group_variables, last_background_own_session = set(), False   # 这一层里记着自己用 setsid 起的后台任务进程号的变量（OWN_GROUP_TARGET 那段注释）
        for index, (words, command_redirections) in enumerate(zip(commands, redirections)):
            opened, closed, _ = loop_keyword_changes(words) if words else (0, 0, False)
            for operator, target in command_redirections:
                if operator in shell_words.OUTPUT_REDIRECTS and os.path.basename(target) in CGROUP_CONTROL_FILES:
                    findings.append(TerminationFinding(tuple(words), f"往 {target} 写：一次停掉（冻住）那个 cgroup 里的全部进程", "cgroup"))
            if words:
                if index + 1 < len(boundaries) and boundaries[index + 1][:1] == ("&",):
                    last_background_own_session = (setsid_started_without_fork(words)
                                                   or detector_break_is_set("own-session-group-session-unchecked"))
                else:
                    update_own_group_variables(words, own_group_variables, last_background_own_session)
                found, sends = call_findings(words, loop_depth + opened > 0, process_table, own_ancestors, frozenset(own_group_variables))
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
    """命令里有打得到别人的终止写法就交回每一处的原因；否则交 []。进程表只在判到 kill / proc.py stop 的写死的进程号时才读（ProcessTableOnDemand，读一次）。
    前台、run_in_background 一样拒。弄坏开关 always-read-process-table：命令里有数字就先整扫 /proc。"""
    if (detector_break_is_set("disable-check")
            or detector_break_is_set("allow-process-signals")):
        return []
    process_table = ProcessTableOnDemand(process_table_reader, own_process_id)
    if detector_break_is_set("always-read-process-table") and re.search(r"(?<![\w$-])\d+\b", command):
        process_table.load()
    return list(dict.fromkeys(f"{finding.problem}（{' '.join(finding.words)[:120]}）" if finding.words else finding.problem
                              for finding in termination_findings(command, process_table, ())))

# 扫脚本（门禁 code-tooling 的 research-gate-lint 格调 --scan-scripts）：同一份判定，按文件里的行号报；own-scope 标注与待改清单
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
    scanned_folders = {os.path.realpath(directory) for directory in directories if os.path.isdir(directory)}   # 这一趟真扫的目录（只扫这一层，不进子目录）
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
    not_judged_here = []   # 待改清单里所在的目录这一趟没扫的行：没扫就不知道它还有没有要放行的，判「本次未判」，不判成「一处要放行的都没有」
    for relative in sorted(pending):
        if relative not in pending_hits and os.path.isfile(os.path.join(root, relative)):
            if (os.path.realpath(os.path.dirname(os.path.join(root, relative))) not in scanned_folders
                    and not detector_break_is_set("pending-outside-scanned-as-stale")):
                not_judged_here.append(relative)
            else:
                list_problems.append(f"{PENDING_LIST}:{pending[relative][0]} {relative} 里一处要放行的都没有（改好了就从清单里删掉这一行）")
    not_run_file = os.environ.get("GATE_NOT_RUN_FILE")
    for relative in not_judged_here:
        print(f"  ! 本次未判（这不是通过）：{PENDING_LIST}:{pending[relative][0]} {relative} 所在的目录 {os.path.dirname(relative) or '.'} 这一趟没扫，"
              "它还有没有要放行的没判；要判就把那个目录也交给 --scan-scripts")
        if not_run_file and os.path.exists(not_run_file):
            with open(not_run_file, "a", encoding="utf-8") as handle:
                handle.write(f"进程安全：{PENDING_LIST} 里 {relative} 所在的目录这一趟没扫，本次未判（要判就把 {os.path.dirname(relative) or '.'} 也交给 bash-command-detector.sh --scan-scripts）\n")
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
          f"没判的：{PENDING_LIST} 里的 {len(pending_hits)} 个文件 {pending_note}"
          + (f"；本次未判：{PENDING_LIST} 里 {len(not_judged_here)} 个文件所在的目录这一趟没扫（{'、'.join(not_judged_here)}）" if not_judged_here else ""))
    return 0

# ⑪ rm -r 作用在工作根本身、它的上级或工作根下第一层带通配：工作根下的目录分属不同会话，按名字通删认不出归属。
#   出路是 scripts/hosts/work-directory-ownership.sh sweep（只删归属是本会话、没有进程开着的，删前逐项连大小写日志）。
import fnmatch
WORK_ROOT_SWEEP_SCRIPT = "scripts/hosts/work-directory-ownership.sh"
REMOVAL_GLOB_CHARACTERS = set("*?[")
REMOVAL_VARIABLE = re.compile(r"\$(?:\{([A-Za-z_][A-Za-z0-9_]*)\}|([A-Za-z_][A-Za-z0-9_]*))")

def work_root_removal_roots():
    """⑪ 认的根：配置的工作根（不看弄坏开关）与会话临时目录根 /tmp/claude-<uid>（里面是各会话的临时目录），连同解开符号链接的写法。
    同目录的 lib_work_root.py 读不到时只认 SINGLEFS_WORK_ROOT（不补缺省、不退回家目录：没设就只认会话临时目录根）。"""
    if work_roots:
        roots = [work_roots.configured_work_root(), work_roots.claude_session_temporary_root()]
    else:
        configured = os.environ.get("SINGLEFS_WORK_ROOT")
        roots = ([os.path.normpath(os.path.abspath(configured))] if configured else []) + [f"/tmp/claude-{os.getuid()}"]
        roots = [os.path.normpath(os.path.abspath(os.path.expanduser(os.environ.get("SINGLEFS_WORK_ROOT") or os.path.join(data_root, "work")))),
                 f"/tmp/claude-{os.getuid()}"]
    return list(dict.fromkeys(roots + [os.path.realpath(root) for root in roots]))

def recursive_removal_operands(arguments):
    """rm 的参数里带 -r / -R / --recursive（与别的短选项合写的也算，写在操作数后面的也算）时交回操作数；不带交 []。"""
    recursive, operands, options_ended = False, [], False
    for argument in arguments:
        if not options_ended and argument == "--":
            options_ended = True
        elif not options_ended and argument.startswith("--"):
            recursive = recursive or argument == "--recursive"
        elif not options_ended and argument.startswith("-") and argument != "-":
            recursive = recursive or "r" in argument[1:] or "R" in argument[1:]
        else:
            operands.append(argument)
    return operands if recursive else []

def removal_target_path(word, directory, environment, configured_root):
    """rm 的一个操作数换成绝对路径（通配原样留着）：~、$HOME、$SINGLEFS_WORK_ROOT 与这条命令里设过的变量代入，相对路径按当时的目录；算不出交 None。"""
    if word == "~" or word.startswith("~/"):
        word = os.path.expanduser("~") + word[1:]
    unknown = []
    def substitute(match):
        name = match.group(1) or match.group(2)
        if name in environment:
            return environment[name]
        if name == "HOME":
            return os.path.expanduser("~")
        if name == "SINGLEFS_WORK_ROOT":
            return os.environ.get(name) or configured_root
        unknown.append(name)
        return ""
    word = REMOVAL_VARIABLE.sub(substitute, word)
    if unknown or "$" in word or "`" in word:
        return None
    if not os.path.isabs(word):
        if directory is None:
            return None
        word = os.path.join(directory, word)
    return os.path.normpath(word)

def removal_component_matches(name, pattern):
    """一段路径名对一段可能带通配的写法：照 bash 的默认（不开 dotglob），通配起头的不认点开头的名字。"""
    if not REMOVAL_GLOB_CHARACTERS & set(pattern):
        return name == pattern
    if name.startswith(".") and not pattern.startswith("."):
        return False
    return fnmatch.fnmatchcase(name, pattern)

def removal_hits_work_root(path, root):
    """rm -r 的一个目标（绝对路径，可能带通配）打到工作根本身、它的上级或工作根下第一层的通配，交回是哪一种；不是交 None。"""
    parts = [part for part in path.split("/") if part]
    root_parts = [part for part in root.split("/") if part]
    if not all(removal_component_matches(name, pattern) for name, pattern in zip(root_parts, parts)):
        return None
    covered_by_glob = any(REMOVAL_GLOB_CHARACTERS & set(pattern) for pattern in parts[:len(root_parts)])
    if len(parts) < len(root_parts):
        return "通配盖住了工作根的上级" if covered_by_glob else "工作根的上级"
    if len(parts) == len(root_parts):
        return "通配盖住了工作根本身" if covered_by_glob else "工作根本身"
    if REMOVAL_GLOB_CHARACTERS & set(parts[len(root_parts)]):
        return "工作根下第一层带通配"
    return None

def work_root_removal_refusal(command, directory):
    """命令位置上的 rm -r（每一层：bash -c、喂给 shell 的 heredoc、命令替换；前缀与 capped.sh 剥开；cd 与 export 跟着算）
    作用在工作根本身、它的上级或工作根下第一层带通配的路径上，交回认出的写法；否则交 []。前台、run_in_background、主 agent 与子 agent 一样拒。"""
    if (detector_break_is_set("disable-check")
            or detector_break_is_set("allow-work-root-removal")):
        return []
    roots = work_root_removal_roots()
    found = []
    for at in shell_words.commands_at_command_position(command, directory).commands:
        if not at.words or os.path.basename(at.words[0]) != "rm":
            continue
        for operand in recursive_removal_operands(at.words[1:]):
            path = removal_target_path(operand, at.directory, at.environment, roots[0])
            if path is None:
                continue
            hit = next(((root, reason) for root in roots for reason in [removal_hits_work_root(path, root)] if reason), None)
            if hit:
                found.append(f"rm -r {operand}（{hit[1]}：{hit[0]}）")
    return list(dict.fromkeys(found))

# ⑳ 手拷仓副本（文件头 ⑳）：命令位置上的 rsync、cp -r / -R / -a 的源是本仓的 crates/ 或仓根整个。仓副本只经 scripts/hosts/minimal-copy.py 建，
#   拷哪些由 scripts/hosts/lib_repository_copy.py 的闭包现算；手列的范围跟不上编译与测试要读的仓根条目。
MINIMAL_COPY_TOOL = "scripts/hosts/minimal-copy.py"
MANUAL_COPY_UNLISTED_TOOL = "scripts/hosts/compile-then-swap.py"   # 弄坏开关 manual-copy-howto-unlisted-tool 下出路写成它：不在 lib_project_code_runs.py 的直接跑名单上，照抄会被 project-code-guard.sh 拒
MANUAL_COPY_HOWTO_COPY_PLACEHOLDER = "<草稿目录>/copy"   # 出路里副本那一处的占位，自证照抄出路喂 project-code-guard.sh 时换成自证草稿里的一个路径
MANUAL_COPY_HOWTO_COMMAND = re.compile(r"→ 怎么办：仓副本只用 `([^`]*)` 建")   # 自证从拒绝说明里取出路的整条写法
# rsync 带值的选项（值另起一个词时跳过它；写成 --选项=值 的不用跳）：短的是 -e、-f、-T、-B、-M、-@
RSYNC_SHORT_OPTIONS_WITH_VALUE = "efTBM@"
RSYNC_LONG_OPTIONS_WITH_VALUE = frozenset({
    "--exclude", "--include", "--exclude-from", "--include-from", "--filter", "--files-from", "--rsh", "--rsync-path", "--temp-dir",
    "--compare-dest", "--copy-dest", "--link-dest", "--backup-dir", "--suffix", "--chmod", "--chown", "--usermap", "--groupmap", "--timeout",
    "--contimeout", "--port", "--sockopts", "--out-format", "--log-file", "--log-file-format", "--password-file", "--bwlimit", "--partial-dir",
    "--max-size", "--min-size", "--max-delete", "--block-size", "--modify-window", "--checksum-choice", "--compress-choice", "--compress-level",
    "--skip-compress", "--iconv", "--info", "--debug", "--stop-after", "--stop-at", "--write-batch", "--only-write-batch", "--read-batch",
    "--protocol", "--outbuf", "--remote-option", "--address", "--copy-as", "--max-alloc"})
RSYNC_LISTING_OPTIONS = frozenset({"--dry-run", "--list-only"})   # 只演一遍或只列，不拷
RSYNC_REMOTE_OPERAND = re.compile(r"^[^/]*:")   # 主机:路径、rsync://主机/…（本地路径里带冒号的写成 ./ 起头）

def manual_copy_howto(repository_root):
    """⑳ 拒绝说明里的出路整条：minimal-copy.py 照这一份钩子所在的仓根建副本、带测试输入；弄坏开关 manual-copy-howto-unlisted-tool 下换成不在直接跑名单上的脚本。"""
    tool = MANUAL_COPY_UNLISTED_TOOL if detector_break_is_set("manual-copy-howto-unlisted-tool") else MINIMAL_COPY_TOOL
    return f"python3 {tool} {repository_root} {MANUAL_COPY_HOWTO_COPY_PLACEHOLDER} --with-test-inputs"

def rsync_copied_sources(arguments):
    """rsync 的参数切开交回源（操作数里除最后一个目的之外的）；带 -n / --dry-run / --list-only 的、只给一个操作数的（只列不拷）交 []。"""
    operands, position, listing_only = [], 0, False
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
            option, has_value, _value = argument.partition("=")
            if option in RSYNC_LISTING_OPTIONS:
                listing_only = True
            elif option in RSYNC_LONG_OPTIONS_WITH_VALUE and not has_value:
                position += 1
            continue
        letters = argument[1:]
        for index, letter in enumerate(letters):
            if letter in RSYNC_SHORT_OPTIONS_WITH_VALUE:
                if index + 1 == len(letters):
                    position += 1
                break
            if letter == "n":
                listing_only = True
    return [] if listing_only or len(operands) < 2 else operands[:-1]

def copy_arguments_recursive(arguments):
    """cp 的参数里带 -r / -R / -a（与别的短选项合写的也算，写在操作数后面的也算）、--recursive、--archive；`--` 之后的不算。"""
    for argument in arguments:
        if argument == "--":
            return False
        if argument in ("--recursive", "--archive"):
            return True
        if argument.startswith("-") and not argument.startswith("--") and argument != "-":
            for letter in argument[1:]:
                if letter in COPY_SHORT_OPTIONS_WITH_VALUE["cp"]:
                    break
                if letter in "rRa":
                    return True
    return False

def copied_sources(name, arguments):
    """rsync 与递归的 cp 的源操作数；别的命令、不拷的写法（rsync -n、只给一个操作数、cp 不带递归）交 []。"""
    if name == "rsync":
        return rsync_copied_sources(arguments)
    if name == "cp" and copy_arguments_recursive(arguments):
        parsed = parse_copy_arguments("cp", arguments)
        return list(parsed.operands) if parsed.target_directory else list(parsed.operands[:-1])
    return []

def copied_source_hits(word, directory, variables, protected):
    """一个源操作数打到 protected（{真实路径: 说法}）里的哪一处：交说法的列表；远端写法、这一刻算不出的、没打到的交 []。
    ~、前面赋过值的变量与 $HOME 展开，相对路径按当时的目录，规整 .. 与结尾的 /；带通配的照 bash 的默认展开，通配写在 protected 那一处下一层整层的也算。"""
    if RSYNC_REMOTE_OPERAND.match(word):
        return []
    if word == "~" or word.startswith("~/"):
        word = os.path.expanduser("~") + word[1:]
    expanded = shell_words.expand_known_variables(word, variables or {})
    if expanded is None:
        return []
    if not os.path.isabs(expanded):
        if directory is None:
            return []
        expanded = os.path.join(directory, expanded)
    expanded = os.path.normpath(expanded)
    if not REMOVAL_GLOB_CHARACTERS & set(expanded):
        what = protected.get(os.path.realpath(expanded))
        return [what] if what else []
    hits = []
    parent, last = os.path.split(expanded)
    covered = protected.get(os.path.realpath(parent)) if REMOVAL_GLOB_CHARACTERS & set(last) and not REMOVAL_GLOB_CHARACTERS & set(parent) else None
    if covered:
        hits.append(f"通配盖住{covered}的整层")
    for match in sorted(glob.glob(expanded)):
        what = protected.get(os.path.realpath(match))
        if what:
            hits.append(f"通配展开到{what}")
    return hits

def manual_repository_copy_refusal(command, repository_root, start_directory):
    """命令位置上的 rsync、cp -r / -R / -a（每一层：bash -c、喂给 shell 的 heredoc、命令替换；前缀剥开；cd 与前面赋过值的变量跟着算）的源是本仓的 crates/
    或仓根整个（文件头 ⑳），交回认出的写法；否则交 []。前台、run_in_background、主 agent 与子 agent 一样拒。弄坏开关 allow-manual-repository-copy 下一概放行。"""
    if detector_break_is_set("allow-manual-repository-copy"):
        return []
    root = os.path.realpath(repository_root)
    protected = {root: "仓根整个", os.path.realpath(os.path.join(root, "crates")): "本仓的 crates/"}
    found = []
    for at in shell_words.commands_at_command_position(command, start_directory).commands:
        if not at.words:
            continue
        name = os.path.basename(at.words[0])
        for word in copied_sources(name, at.words[1:]):
            for hit in copied_source_hits(word, at.directory, at.variables, protected):
                found.append(f"{name} {word}（{hit}）")
    return list(dict.fromkeys(found))

def selftest(hook_dir):
    if shell_words is None:
        print(f"  ✗ 自检：读不到共用切词模块 {shell_words_library_path(hook_dir)}（{shell_words_error!r}）")
        print("    → 怎么办：恢复 .claude/hooks/lib_shell_words.py（切词与认命令位置只有那一份，别在 hook 里再抄一份），再跑 --selftest")
        return 1
    # 临时目录由共用库建（通过、判红、抛异常都删），那三格也由它判；主体在 selftest_in
    scratch_library_path = os.path.join(hook_dir, "lib_selftest_scratch.py")
    try:
        scratch_specification = importlib.util.spec_from_file_location("lib_selftest_scratch", scratch_library_path)
        scratch = importlib.util.module_from_spec(scratch_specification)
        scratch_specification.loader.exec_module(scratch)
    except Exception as error:   # 文件不在、语法错、导入时抛的都算读不到
        print(f"  ✗ 自检：bash-command-detector 读不到共用的临时目录库 {scratch_library_path}（{error!r}）")
        print("    → 怎么办：恢复 .claude/hooks/lib_selftest_scratch.py（自证临时目录的建法与三格探查只有那一份），再跑 --selftest")
        return 1
    exit_code, problems = scratch.run_with_scratch(
        lambda work: selftest_in(hook_dir, work), "bash-command-detector-",
        detector_break_is_set("keep-selftest-scratch"), "BASH_COMMAND_DETECTOR_SELFTEST_ROLE",
        ["bash", os.path.join(hook_dir, "bash-command-detector.sh"), "--selftest"])
    return scratch.merged_exit_code(exit_code, problems, ".claude/hooks/bash-command-detector.sh 的 selftest()",
                                    "BASH_COMMAND_DETECTOR_BREAK=keep-selftest-scratch")

def selftest_in(hook_dir, work):
    detections = os.path.join(work, "detections.jsonl")
    # 弄坏开关只在自证里认：标记文件写进环境，自证起的子进程（入口、--scan-scripts）带着它才认调用方设的开关值
    with open(os.path.join(work, "selftest-mark"), "w", encoding="utf-8") as mark_handle:
        mark_handle.write(SELFTEST_MARK_CONTENT + "\n")
    os.environ[SELFTEST_MARK_VARIABLE] = os.path.join(work, "selftest-mark")
    shell_words.SYSTEM_STARTUP_DIRECTORY = os.path.join(work, "system-startup-empty")   # 系统级启动文件不读本机的 /etc（三方 d137-gov-r1 第三段那一组另换样本目录）
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
        ('bg-notify-r1 X04', 'nice -n 19 bash .claude/scripts/gate.sh --staged > /tmp/claude-1000/gate-triage-commit-0918/gate-run.log 2>&1 & bash scripts/hosts/cache-keepalive.sh & wait $!', 1, True),
        ('bg-notify-r1 X05', 'nice -n 19 bash .claude/scripts/gate.sh --staged > /tmp/claude-1000/gate-triage-commit-0918/gate-run.log 2>&1 & bash scripts/hosts/cache-keepalive.sh & wait -n', 1, True),
        ('bg-notify-r1 X06', '(nice -n 19 bash .claude/scripts/gate.sh --staged > /tmp/claude-1000/gate-triage-commit-0918/gate-run.log 2>&1 &); wait', 1, True),
        ('bg-notify-r1 X07', 'pid=$(nice -n 19 bash .claude/scripts/gate.sh --staged > /tmp/claude-1000/gate-triage-commit-0918/gate-run.log 2>&1 & echo $!); echo "gate pid $pid"; wait "$pid"', 1, True),
        ('bg-notify-r1 X08', 'coproc GATE { nice -n 19 bash .claude/scripts/gate.sh --staged > /tmp/claude-1000/gate-triage-commit-0918/gate-run.log 2>&1; }', 1, True),
        ('bg-notify-r1 X09', "tmux new-session -d -s gate 'nice -n 19 bash .claude/scripts/gate.sh --staged > /tmp/claude-1000/gate-triage-commit-0918/gate-run.log 2>&1'", 1, True),
        ('bg-notify-r1 X10', "screen -dmS gate bash -c 'nice -n 19 bash .claude/scripts/gate.sh --staged > /tmp/claude-1000/gate-triage-commit-0918/gate-run.log 2>&1'", 1, True),
        ('bg-notify-r1 X11', 'systemd-run --user --collect --unit gate-run nice -n 19 bash .claude/scripts/gate.sh --staged', 1, True),
        ('bg-notify-r1 X12', "nice -n 19 python3 - > /tmp/claude-1000/gate-triage-commit-0918/rerun.log 2>&1 <<'EOF' &\nimport subprocess\np = subprocess.Popen(['bash', 'scripts/experiment-apparatus/replay.sh'])\np.wait()\nEOF", 1, True),
        ('bg-notify-r1 X13', 'nice -n 19 bash .claude/scripts/gate.sh --staged > /tmp/claude-1000/gate-triage-commit-0918/gate-run.log 2>&1 &', 0, False),
        ('bg-notify-r1 F01', 'nohup nice -n 19 bash .claude/scripts/gate.sh --staged > /tmp/claude-1000/gate-triage-commit-0918/gate-run.log 2>&1', 0, True),
        ('bg-notify-r1 F02（第三种不记；它的轮询循环没有 timeout，记的是第二种）', 'nice -n 19 bash .claude/scripts/gate.sh --staged > /tmp/claude-1000/gate-triage-commit-0918/gate-run.log 2>&1 & pid=$!; while kill -0 "$pid" 2>/dev/null; do sleep 20; done; tail -3 /tmp/claude-1000/gate-triage-commit-0918/gate-run.log', 1, True),
        ('bg-notify-r1 F03', 'nice -n 19 bash .claude/scripts/gate.sh --staged > /tmp/claude-1000/gate-triage-commit-0918/gate-run.log 2>&1 & tail --pid=$! -f /tmp/claude-1000/gate-triage-commit-0918/gate-run.log > /dev/null', 0, True),
        ('bg-notify-r1 F04', "nice -n 19 python3 - <<'EOF' > /tmp/claude-1000/gate-triage-commit-0918/tally.txt\nimport json\nrows = [json.loads(line) for line in open('/tmp/claude-1000/gate-triage-commit-0918/events.jsonl')]\nprint(sum(1 for row in rows if row['flags'] & 0x4))\nEOF", 0, True),
        ('bg-notify-r1 F05', "cat > /tmp/claude-1000/impl-probe/copy/crates/singlefs-verification/tests/probe.rs <<'EOF'\nfn digest(bytes: &[u8]) -> u64 { bytes.iter().map(|&b| b as u64).sum() }\n#[test]\nfn probe() { assert_eq!(digest(&[1, 2]), 3); }\nEOF\ncd /tmp/claude-1000/impl-probe/copy && nice -n 19 cargo test --release --test probe 2>&1 | tail -20", 0, True),
        # 引号里当数据写的 & 不算（一百三十七节件 1b 改成与 ④ 同一套数法之后；弄坏开关 background-ampersand-by-regex 下照旧误记、这一格红）
        ('bg-notify-r1 F06：sed 替换串引号里的 & 不记', "nice -n 19 cargo test --release 2>&1 | sed -E 's/^test (.*) \\.\\.\\. FAILED$/!! &/' > /tmp/claude-1000/impl-probe/test.log", 0, True),
        ('bg-notify-r1 F07', 'setsid -w nice -n 19 bash .claude/scripts/gate.sh --staged > /tmp/claude-1000/gate-triage-commit-0918/gate-run.log 2>&1', 0, True),
        # 不带 --wait 的 setsid 是进程组组长时会 fork、自己马上退出（一百三十七节件 1b 第 5 条；弄坏开关 bare-setsid-unrecorded 下前两格红）
        ("一百三十七：裸 setsid 不带 --wait，照记一条", "setsid bash long.sh > long.log 2>&1 < /dev/null", 1, True),
        ("一百三十七：内存包装里的 setsid 不带 --wait，照记一条", "bash scripts/hosts/run-with-memory-cap.sh 8G setsid bash x.sh", 1, True),
        ("一百三十七：内存包装里的 setsid --wait 不记", "bash scripts/hosts/run-with-memory-cap.sh 8G setsid --wait bash x.sh", 0, True),
        ("setsid -f 放后台", "setsid -f bash long.sh > long.log 2>&1 < /dev/null", 1, True),
        ("setsid 跟着的 tail -f 不是 setsid 的选项", "setsid -w tail -f log.txt", 0, True),
        # 每个 & 记了 $!、之后逐个 wait "$pN"：与 ④ 同一套数法，不记（一百三十七节件 1b 第 5 条主 agent 的原样；弄坏开关 background-ampersand-by-regex 下红）
        ("一百三十七：两个子 shell 各记 $!、逐个 wait \"$pN\" 不记",
         '( bash a.sh > o1 2>&1; echo "rc=$?" >> o1 ) & p1=$!; ( bash b.sh > o2 2>&1 ) & p2=$!; wait $p1; wait $p2', 0, True),
    ]
    results = []
    for label, command, want, *background in cases:
        tool_input = {"command": command, "run_in_background": bool(background and background[0])}
        written = record({"tool_name": "Bash", "session_id": "s", "tool_input": tool_input}, detections)
        results.append((label, want, written))
    # 起看门狗：(说明, 命令, run_in_background, 该不该拒绝)
    watchdog_cases = [
        ("前台起、输出丢进 /dev/null 还带 &", "bash scripts/agents/watch.sh a1 > /dev/null 2>&1 &", False, 1),
        ("run_in_background 起、只写 watch.sh", "bash scripts/agents/watch.sh a1", True, 0),
        ("run_in_background 起却又带 &", "bash scripts/agents/watch.sh a1 &", True, 1),
        ("前台 grep 把 watch.sh 当参数", "grep -n watch.sh records/x.md", False, 0),
        ("前台 agent-watch.py report 不是看门狗", "python3 scripts/agents/agent-watch.py report --session-dir x", False, 0),
        ("前台起、什么都没加", "bash scripts/agents/watch.sh a1,a2", False, 1),
        ("前台起 agent-watch.py watch", "python3 scripts/agents/agent-watch.py watch --agents a1", False, 1),
        ("run_in_background 起 agent-watch.py watch", "python3 scripts/agents/agent-watch.py watch --agents a1", True, 0),
        ("run_in_background 起、带 --ack 与 cd", "cd ~/code/singlefs && bash scripts/agents/watch.sh --ack a1:上下文过大 a1", True, 0),
        ("run_in_background 起却挂 nohup", "nohup bash scripts/agents/watch.sh a1", True, 1),
        ("run_in_background 起却挂 setsid", "setsid bash scripts/agents/watch.sh --processes", True, 1),
        ("run_in_background 起却跟 disown", "bash scripts/agents/watch.sh a1; disown", True, 1),
        ("run_in_background 起却把输出丢进 /dev/null", "bash scripts/agents/watch.sh a1 > /dev/null", True, 1),
        ("前台起、写在 bash -c 里", "bash -c 'nice -n 19 ./scripts/agents/watch.sh a1'", False, 1),
        ("前台起、包在 capped.sh 里", "bash scripts/hosts/capped.sh 4 bash scripts/agents/watch.sh a1", False, 1),
        ("run_in_background 起、包在 capped.sh 里", "bash scripts/hosts/capped.sh 4 bash scripts/agents/watch.sh a1", True, 0),
        ("前台 watch.sh --selftest 不盯", "bash scripts/agents/watch.sh --selftest", False, 0),
        ("前台 watch.sh --report 不盯", "bash scripts/agents/watch.sh --report", False, 0),
        ("前台 bash -n 只查语法", "bash -n scripts/agents/watch.sh", False, 0),
        ("前台读看门狗输出", "tail -5 /tmp/claude-1000/x/tasks/b1.output; cat scripts/agents/watch.sh | head", False, 0),
        ("前台 echo 引号里当数据写", 'echo "bash scripts/agents/watch.sh a1 &"', False, 0),
        ("前台写进笔记的 heredoc 正文", "cat > note.md <<'EOF'\nbash scripts/agents/watch.sh a1 > /dev/null 2>&1 &\nEOF", False, 0),
        ("前台注释里写的", "ls  # bash scripts/agents/watch.sh a1 &", False, 0),
        ("前台 diff <(git show …watch.sh) watch.sh：进程替换之后的词是 diff 的参数", "diff <(git show HEAD:scripts/agents/watch.sh) scripts/agents/watch.sh", False, 0),
        # 一百三十七节件 8b：/dev/null 只算看门狗所在那一段的（watchdog-null-redirect-whole-command 下该放行的两格红，照拒的四格 allow-watchdog-misuse 下红）
        ("run_in_background 起、前一段 grep 2>/dev/null、后一段 watch.sh 放行", "grep -n x records/a.md 2>/dev/null; bash scripts/agents/watch.sh a1", True, 0),
        ("run_in_background 起、前一段 grep 2>/dev/null、watch.sh 自己也丢进 /dev/null 照拒", "grep -n x records/a.md 2>/dev/null; bash scripts/agents/watch.sh a1 > /dev/null", True, 1),
        ("run_in_background 起、bash -c 整条丢进 /dev/null 照拒", "bash -c 'bash scripts/agents/watch.sh a1' > /dev/null", True, 1),
        ("run_in_background 起、花括号整组丢进 /dev/null 照拒", "{ bash scripts/agents/watch.sh a1; } > /dev/null", True, 1),
        ("run_in_background 起、管道后段把标准输出丢进 /dev/null 照拒", "bash scripts/agents/watch.sh a1 | grep x > /dev/null", True, 1),
        ("run_in_background 起、管道后段只丢 2 号放行", "bash scripts/agents/watch.sh a1 | grep x 2>/dev/null", True, 0),
    ]
    for label, command, background, want in watchdog_cases:
        results.append((f"看门狗:{label}", want, 1 if watchdog_rejection(command, background)[0] else 0))
    # 前台没超时的等待循环：(说明, 命令, run_in_background, 该不该拒绝)
    wait_loop_cases = [
        ("前台 until … sleep 没超时", "until grep -q x f; do sleep 10; done", False, 1),
        ("同一条套上 timeout 60", "timeout 60 bash -c 'until grep -q x f; do sleep 10; done'", False, 0),
        ("同一条走 run_in_background", "until grep -q x f; do sleep 10; done", True, 0),
        ("run_in_background 里轮询后台任务的输出文件", "until [ -s /tmp/claude-1000/-home-x/abc/tasks/b1x2.output ]; do sleep 20; done; tail -5 /tmp/claude-1000/-home-x/abc/tasks/b1x2.output", True, 1),
        ("run_in_background 里轮询后台任务输出、外面套了 timeout 也拒", "timeout 600 bash -c 'until grep -q done tasks/b9.output; do sleep 5; done'", True, 1),
        ("run_in_background 里读一次后台任务输出、不轮询", "tail -5 /tmp/x/tasks/b1.output", True, 0),
        ("前台按 pid 轮询的 while kill -0", 'while kill -0 "$pid" 2>/dev/null; do sleep 20; done; tail -3 log', False, 1),
        ("前台 timeout 只在循环条件里", "until timeout 5 grep -q x f; do sleep 10; done", False, 1),
        ("前台多行写的 while true", "while true\ndo\n  grep -q x f && break\n  sleep 5\ndone", False, 1),
        ("前台 sleep 在循环条件里", "while sleep 5; do grep -q x f && break; done", False, 1),
        ("前台喂给 bash 的 heredoc 里的循环", "bash <<'EOF'\nuntil grep -q x f; do sleep 10; done\nEOF", False, 1),
        ("timeout 60 bash 的 heredoc 里的循环", "timeout 60 bash <<'EOF'\nuntil grep -q x f; do sleep 10; done\nEOF", False, 0),
        ("capped.sh 与 nice 包着的 timeout", "nice -n 19 capped.sh 4 timeout 600 bash -c 'until grep -q x f; do sleep 10; done'", False, 0),
        ("前台 bash -c 里没有 timeout", "nice -n 19 bash -c 'until grep -q x f; do sleep 10; done'", False, 1),
        ("前台 capped.sh 包着的 bash -c 里没有 timeout", "bash scripts/hosts/capped.sh 4 bash -c 'until grep -q x f; do sleep 10; done'", False, 1),
        ("前台 run-with-memory-cap.sh 包着的 bash -c 里没有 timeout", "bash scripts/hosts/run-with-memory-cap.sh 4G bash -c 'until grep -q x f; do sleep 10; done'", False, 1),
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
        # time 关键字带 -p（LIB_SHELL_WORDS_BREAK=time-options-as-command 下这一格红）
        ("前台 time -p 包着的 bash -c 里没有 timeout", "time -p bash -c 'until grep -q x f; do sleep 10; done'", False, 1),
        # heredoc 挂在没收尾的命令替换、进程替换里的 bash 上（LIB_SHELL_WORDS_BREAK=heredoc-open-substitution-unseen 下前两格红）
        ("前台 $(bash <<EOF) 里的循环", "x=$(bash <<'EOF'\nuntil grep -q x f; do sleep 10; done\nEOF\n)", False, 1),
        ("前台 <(bash <<EOF) 里的循环", "diff <(bash <<'EOF'\nuntil grep -q x f; do sleep 10; done\nEOF\n) y", False, 1),
        ("$(timeout 60 bash <<EOF) 里的循环", "x=$(timeout 60 bash <<'EOF'\nuntil grep -q x f; do sleep 10; done\nEOF\n)", False, 0),
        ("$(cat <<EOF) 喂的是数据", "x=$(cat <<'EOF'\nuntil grep -q x f; do sleep 10; done\nEOF\n)", False, 0),
        # 同一行里先有双引号包着、已收尾的 $(…)，再接顶层的 bash <<EOF（LIB_SHELL_WORDS_BREAK=double-quoted-substitution-unclosed 下这两格红）
        ('前台 cd "$(…)" 之后 bash <<EOF 里的循环', "cd \"$(git rev-parse --show-toplevel)\" && bash <<'EOF'\nuntil grep -q x f; do sleep 10; done\nEOF", False, 1),
        ('前台 echo "$(date)" | bash <<EOF 里的循环', "echo \"$(date)\" | bash <<'EOF'\nuntil grep -q x f; do sleep 10; done\nEOF", False, 1),
        # ANSI-C 引号 $'…' 里的 \' 不收尾，之后没收尾的 $( 里的 bash <<EOF 照认（LIB_SHELL_WORDS_BREAK=ansi-c-quotes-unseen 下这一格红）
        ("前台 $'a\\'b' 之后 $(bash <<EOF) 里的循环", "echo $'a\\'b'; x=$(bash <<'EOF'\nuntil grep -q x f; do sleep 10; done\nEOF\n)", False, 1),
    ]
    for label, command, background, want in wait_loop_cases:
        results.append((f"等待循环:{label}", want, 1 if wait_loop_refusal(command, background) else 0))
    # ⑬ 项目子 agent 在 Bash 里起 claude 会话：(说明, agent 类型, 命令, 该不该拒)；弄坏开关 allow-subagent-claude-launch 与 disable-check 下「拒」的八格红
    claude_cases = [
        ("子 agent 命令位置上的 claude -p", "implementation-writer", "claude -p '接着做'", 1),
        ("子 agent 经 \"$CLAUDE_CODE_EXECPATH\" 起", "clerk", '"$CLAUDE_CODE_EXECPATH" -p x', 1),
        ("子 agent 经 ${CLAUDE_CODE_EXECPATH} 起", "clerk", "${CLAUDE_CODE_EXECPATH} --print x", 1),
        ("子 agent 起以 /claude 结尾的可执行文件", "tooling-writer", "~/.local/bin/claude -p x", 1),
        ("子 agent nice 包着起 claude", "implementation-writer", "nice -n 19 claude -p x", 1),
        ("子 agent bash -c 里起 claude", "implementation-writer", "bash -c 'claude -p x'", 1),
        ("子 agent 命令替换里起 claude", "implementation-writer", "out=$(claude -p x)", 1),
        ("子 agent 喂给 bash 的 heredoc 里起 claude", "implementation-writer", "bash <<'EOF'\nclaude -p x\nEOF", 1),
        ("子 agent command -v claude 只查名字", "implementation-writer", "command -v claude", 0),
        ("子 agent grep 参数里的 claude", "implementation-writer", "grep -n claude records/x.md", 0),
        ("子 agent echo 引号里写的 claude -p", "implementation-writer", "echo 'claude -p x'", 0),
        ("子 agent 写进文件的 heredoc 正文里的 claude", "implementation-writer", "cat > /tmp/x.sh <<'EOF'\nclaude -p x\nEOF", 0),
        ("子 agent 起名字里带 claude 的别的程序", "implementation-writer", "claude-code-helper --version", 0),
        ("主 agent 起 claude 不判", None, "claude -p x", 0),
    ]
    for label, agent, command, want in claude_cases:
        results.append((f"起 claude:{label}", want, 1 if claude_launch_refusal(command, agent) else 0))
    # ⑬ shell 启动时先读的文件里起 claude（一百三十七节件 8f、8g）：弄坏开关 startup-file-claude-unread 与 LIB_SHELL_WORDS_BREAK=startup-files-unread 下「拒」的格红，
    # 各格另在 LIB_SHELL_WORDS_BREAK 的 declared-exports-unseen、interactive-startup-files-unread、zsh-startup-files-unread、executed-script-startup-unread 下红（逐格见文件头开关清单）
    startup_directory = tempfile.mkdtemp()
    try:
        for startup_name, startup_text in (("startup-claude.sh", "claude -p x\n"), ("startup-light.sh", "echo hi\n"),
                                           # 直接执行的脚本先读的启动文件（件 8g）：#! 指到 bash、sh、python 的脚本；交互的 bash 读 HOME 下的 .bashrc，zsh 读 ZDOTDIR 下的 .zshenv
                                           ("exec-bash.sh", "#!/bin/bash\necho hi\n"), ("exec-sh.sh", "#!/bin/sh\necho hi\n"), ("exec-python.py", "#!/usr/bin/env python3\nprint(1)\n"),
                                           ("home-claude/.bashrc", "claude -p x\n"), ("home-light/.bashrc", "echo hi\n"), ("zdot-claude/.zshenv", "claude -p x\n")):
            os.makedirs(os.path.dirname(os.path.join(startup_directory, startup_name)), exist_ok=True)
            with open(os.path.join(startup_directory, startup_name), "w", encoding="utf-8") as handle:
                handle.write(startup_text)
        startup_claude = os.path.join(startup_directory, "startup-claude.sh")
        for label, agent, command, want in (
                ("子 agent BASH_ENV 指到的脚本里起 claude", "implementation-writer", f"BASH_ENV={startup_claude} bash -c true", 1),
                ("子 agent env BASH_ENV=… 起 bash，脚本里起 claude", "implementation-writer", f"env BASH_ENV={startup_claude} bash -c true", 1),
                ("子 agent 前面 export BASH_ENV 再起 bash，脚本里起 claude", "implementation-writer", f"export BASH_ENV={startup_claude}; bash -c true", 1),
                ("子 agent ENV 指到的脚本里起 claude，sh 起的", "implementation-writer", f"ENV={startup_claude} sh -c true", 1),
                ("子 agent BASH_ENV 写相对路径，按命令开头的当前目录解", "implementation-writer", "BASH_ENV=startup-claude.sh bash -c true", 1),
                ("子 agent 直接执行 #!/bin/bash 的脚本，BASH_ENV 指到的脚本里起 claude", "implementation-writer", f"BASH_ENV={startup_claude} ./exec-bash.sh", 1),
                ("子 agent 直接执行 #!/bin/sh 的脚本，ENV 指到的脚本里起 claude", "implementation-writer", f"ENV={startup_claude} ./exec-sh.sh", 1),
                ("子 agent declare -x BASH_ENV 之后起 bash，脚本里起 claude", "implementation-writer", f"declare -x BASH_ENV={startup_claude}; bash -c true", 1),
                ("子 agent typeset -x BASH_ENV 之后起 bash，脚本里起 claude", "implementation-writer", f"typeset -x BASH_ENV={startup_claude}; bash -c true", 1),
                ("子 agent 先赋值后 export BASH_ENV，再起 bash，脚本里起 claude", "implementation-writer", f"BASH_ENV={startup_claude}; export BASH_ENV; bash -c true", 1),
                ("子 agent bash -i 读 HOME 下的 .bashrc 里起 claude", "implementation-writer", f"HOME={startup_directory}/home-claude bash -i -c true", 1),
                ("子 agent bash --rcfile 文件 -i，文件里起 claude", "implementation-writer", f"bash --rcfile {startup_claude} -i -c true", 1),
                ("子 agent sh -i 读 ENV 指到的脚本里起 claude", "implementation-writer", f"ENV={startup_claude} sh -i -c true", 1),
                ("子 agent zsh 读 ZDOTDIR 下的 .zshenv 里起 claude", "implementation-writer", f"ZDOTDIR={startup_directory}/zdot-claude zsh -c true", 1),
                ("子 agent export -n 撤掉 BASH_ENV 之后起 bash", "implementation-writer", f"export BASH_ENV={startup_claude}; export -n BASH_ENV; bash -c true", 0),
                ("子 agent bash -i 不读 BASH_ENV，.bashrc 里没有 claude", "implementation-writer", f"BASH_ENV={startup_claude} HOME={startup_directory}/home-light bash -i -c true", 0),
                ("子 agent zsh -f 不读启动文件", "implementation-writer", f"ZDOTDIR={startup_directory}/zdot-claude zsh -f -c true", 0),
                ("子 agent 直接执行 #!/bin/sh 的脚本不读 BASH_ENV", "implementation-writer", f"BASH_ENV={startup_claude} ./exec-sh.sh", 0),
                ("子 agent 直接执行 python 脚本不读 BASH_ENV", "implementation-writer", f"BASH_ENV={startup_claude} ./exec-python.py", 0),
                ("子 agent BASH_ENV 指到的脚本里没有 claude", "implementation-writer", f"BASH_ENV={startup_directory}/startup-light.sh bash -c true", 0),
                ("子 agent ENV 指到的脚本，bash 不带 --posix 不读", "implementation-writer", f"ENV={startup_claude} bash -c true", 0),
                ("子 agent bash -n 只查语法，不读 BASH_ENV", "implementation-writer", f"BASH_ENV={startup_claude} bash -n -c true", 0),
                ("子 agent BASH_ENV 指到的文件不在", "implementation-writer", f"BASH_ENV={startup_directory}/no-such.sh bash -c true", 0),
                ("子 agent BASH_ENV 是命令替换拼的，这一道不判（由 project-code-guard.sh 照认不出拒）", "implementation-writer", f"BASH_ENV=$(echo {startup_claude}) bash -c true", 0),
                ("主 agent BASH_ENV 指到的脚本里起 claude 不判", None, f"BASH_ENV={startup_claude} bash -c true", 0)):
            results.append((f"起 claude:一百三十七：{label}", want, 1 if claude_launch_refusal(command, agent, startup_directory) else 0))
    finally:
        shutil.rmtree(startup_directory, ignore_errors=True)
    # ⑬ 登录 shell 的 profile（LIB_SHELL_WORDS_BREAK=login-profiles-unread）、系统级启动文件（system-startup-files-unread；
    # SYSTEM_STARTUP_DIRECTORY 换成样本目录）、别的程序里起的 claude（LIB_SHELL_WORDS_BREAK=embedded-launches-unseen 与 embedded-launch-claude-unread）、
    # bash x.sh 起的脚本正文里起 claude（script-body-claude-unread）；「拒」的格在各自的开关下红（逐格见文件头开关清单）
    d137_directory = tempfile.mkdtemp()
    try:
        for d137_name, d137_text in (
                                   ('home-claude-login/.bash_profile', 'claude -p x\n'),
                                   ('home-claude-profile/.profile', 'claude -p x\n'),
                                   ('home-light-login/.bash_profile', 'echo hi\n'),
                                   ('home-light/.bashrc', 'echo hi\n'),
                                   ('system-claude/profile', 'claude -p x\n'),
                                   ('system-claude/bash.bashrc', 'claude -p x\n'),
                                   ('claude-body.sh', '#!/usr/bin/env bash\nclaude -p x\n'),
                                   ('claude-free.sh', '#!/usr/bin/env bash\necho hi\ncommand -v claude\n'),
                                   ('mk-claude/Makefile', 'all:\n\tclaude -p x\n'),
                                   ('prog-claude.awk', 'BEGIN { system("claude -p x") }\n'),
):
            os.makedirs(os.path.dirname(os.path.join(d137_directory, d137_name)), exist_ok=True)
            with open(os.path.join(d137_directory, d137_name), "w", encoding="utf-8") as handle:
                handle.write(d137_text)
        os.chmod(os.path.join(d137_directory, "claude-body.sh"), 0o755)
        for label, agent, command, want, system in (
                ('d137 已知没认：登录 bash -l 读 HOME 下的 .bash_profile 里起 claude', 'implementation-writer', f'HOME={d137_directory}/home-claude-login bash -l -c true', 1, 'system-empty'),
                ('d137 已知没认：登录 exec -l bash 照登录 shell 读 .bash_profile 里起 claude', 'implementation-writer', f'HOME={d137_directory}/home-claude-login exec -l bash -c true', 1, 'system-empty'),
                ('d137 已知没认：登录 sh -l 读 HOME 下的 .profile 里起 claude', 'implementation-writer', f'HOME={d137_directory}/home-claude-profile sh -l -c true', 1, 'system-empty'),
                ('d137 已知没认：登录 bash -l 读的 profile 里没有 claude', 'implementation-writer', f'HOME={d137_directory}/home-light-login bash -l -c true', 0, 'system-empty'),
                ('d137 已知没认：系统级 bash -l 读 /etc/profile（样本目录）里起 claude', 'implementation-writer', f'HOME={d137_directory}/home-light-login bash -l -c true', 1, 'system-claude'),
                ('d137 已知没认：系统级 bash -i 读 /etc/bash.bashrc（样本目录）里起 claude', 'implementation-writer', f'HOME={d137_directory}/home-light bash -i -c true', 1, 'system-claude'),
                ('d137 已知没认：系统级 不登录不交互的 bash -c 不读 /etc 下的', 'implementation-writer', 'bash -c true', 0, 'system-claude'),
                ('d137 H9：python -c 里 os.system 起 claude', 'implementation-writer', 'python3 -c \'import os; os.system("claude -p x")\'', 1, 'system-empty'),
                ('d137 H9：perl -e 里 system 起 claude', 'implementation-writer', 'perl -e \'system("claude -p x")\'', 1, 'system-empty'),
                ('d137 H9：gawk 的 system() 起 claude', 'implementation-writer', 'gawk \'BEGIN{system("claude -p x")}\'', 1, 'system-empty'),
                ('d137 H9：sed 的 e 命令起 claude', 'implementation-writer', "sed -n '1e claude -p x' f", 1, 'system-empty'),
                ('d137 H9：git -c 别名 ! 起 claude', 'implementation-writer', "git -c alias.x='!claude -p x' x", 1, 'system-empty'),
                ('d137 H9：tclsh 读 here-string 里 exec 起 claude', 'implementation-writer', "tclsh <<< 'exec claude -p x'", 1, 'system-empty'),
                ('d137 H9：make 读 Makefile 配方起 claude', 'implementation-writer', f'make -C {d137_directory}/mk-claude', 1, 'system-empty'),
                ('d137 H9：awk -f 程序文件里 system() 起 claude', 'implementation-writer', f'awk -f {d137_directory}/prog-claude.awk f', 1, 'system-empty'),
                ('d137 H9：python -c 不起命令', 'implementation-writer', "python3 -c 'print(1)'", 0, 'system-empty'),
                ("d137 H9：awk '{print $1}' 不起命令", 'implementation-writer', "awk '{print $1}' f", 0, 'system-empty'),
                ('d137 已知没认：bash x.sh 起的脚本正文里起 claude', 'implementation-writer', f'bash {d137_directory}/claude-body.sh', 1, 'system-empty'),
                ('d137 已知没认：source 读进来的脚本正文里起 claude', 'implementation-writer', f'source {d137_directory}/claude-body.sh', 1, 'system-empty'),
                ('d137 已知没认：直接执行的 #!/usr/bin/env bash 脚本正文里起 claude', 'implementation-writer', f'{d137_directory}/claude-body.sh', 1, 'system-empty'),
                ('d137 已知没认：同一条命令里 heredoc 写出再 bash 起，正文里起 claude', 'implementation-writer', f"cat > {d137_directory}/hd137.sh <<'EOF'\nclaude -p x\nEOF\nbash {d137_directory}/hd137.sh", 1, 'system-empty'),
                ('d137 已知没认：脚本正文里只有 command -v claude，不拒', 'implementation-writer', f'bash {d137_directory}/claude-free.sh', 0, 'system-empty'),
                ('d137 已知没认：主 agent bash x.sh 不判', None, f'bash {d137_directory}/claude-body.sh', 0, 'system-empty'),
):
            shell_words.SYSTEM_STARTUP_DIRECTORY = os.path.join(d137_directory, system)
            results.append((f"起 claude:{label}", want, 1 if claude_launch_refusal(command, agent, d137_directory) else 0))
    finally:
        shell_words.SYSTEM_STARTUP_DIRECTORY = os.path.join(work, "system-startup-empty")
        shutil.rmtree(d137_directory, ignore_errors=True)
    # 检出 ③：子 agent 前台 Bash 调用的 timeout 超过 240000 毫秒记一条（弄坏开关 subagent-timeout-unnoted 下头两格红）
    for label, hook_payload, want in (
            ("子 agent 前台 timeout 600000 毫秒记", {"agent_type": "implementation-writer", "tool_input": {"command": "ls", "timeout": 600000}}, 1),
            ("子 agent 前台 timeout 写成字符串 300000 也记", {"agent_type": "clerk", "tool_input": {"command": "ls", "timeout": "300000"}}, 1),
            ("子 agent 前台 timeout 正好 240000 毫秒不记", {"agent_type": "implementation-writer", "tool_input": {"command": "ls", "timeout": 240000}}, 0),
            ("子 agent 前台没写 timeout 不记", {"agent_type": "implementation-writer", "tool_input": {"command": "ls"}}, 0),
            ("主 agent 前台 timeout 600000 毫秒不记", {"tool_input": {"command": "ls", "timeout": 600000}}, 0),
            ("子 agent run_in_background 的 timeout 不在这一条（⑩ 已拒）",
             {"agent_type": "clerk", "tool_input": {"command": "ls", "timeout": 600000, "run_in_background": True}}, 0),
            ("子 agent timeout 写成 true 不记", {"agent_type": "clerk", "tool_input": {"command": "ls", "timeout": True}}, 0)):
        results.append((f"超时检出:{label}", want, len(subagent_timeout_findings(hook_payload))))
    # ⑧ 有 Edit 工具的项目子 agent 在 Bash 里就地改仓内文件：(说明, agent 类型, 命令, 该不该拒)
    edit_repository = tempfile.mkdtemp()
    # 工作根下的样本路径不看弄坏开关（lib_work_root.py 的 configured_work_root）；~ 写法在工作根落在家目录下时才有
    new_root = work_roots.configured_work_root() if work_roots else "/nonexistent-work-root"
    home = os.path.expanduser("~")
    home_new_root = "~" + new_root[len(home):] if new_root.startswith(home + "/") else new_root
    try:
        os.makedirs(os.path.join(edit_repository, ".claude", "agents"))
        open(os.path.join(edit_repository, ".claude", "agents", "experiment-runner.md"), "w").write("---\ntools: Read, Edit, Write, Bash\n---\n")
        open(os.path.join(edit_repository, ".claude", "agents", "sweep.md"), "w").write("---\ntools: Read, Bash\n---\n")
        repository_edit_cases = [
            ("执行员 sed -i 改仓内源文件", "experiment-runner", "sed -i 's/a/b/' scripts/experiment-apparatus/e1.rs", 1),
            ("执行员 python open(仓内路径, 'w')", "experiment-runner", "python3 -c \"open('crates/x/src/lib.rs', 'w').write('x')\"", 1),
            ("执行员 python Path(仓内绝对路径).write_text", "experiment-runner",
             f"python3 - <<'EOF'\nfrom pathlib import Path\nPath('{edit_repository}/.claude/kb/x.md').write_text('y')\nEOF", 1),
            ("执行员 sed -i 改草稿目录", "experiment-runner", "sed -i 's/a/b/' /tmp/claude-1000/e/notes.md", 0),
            ("执行员 python 写草稿目录", "experiment-runner", "python3 -c \"open('/tmp/claude-1000/e/x.txt', 'w').write('x')\"", 0),
            ("执行员 sed -i 改工作根下的草稿目录", "experiment-runner", f"sed -i 's/a/b/' {new_root}/e/notes.md", 0),
            ("执行员 sed -i 改工作根下的草稿目录（~ 写法）", "experiment-runner", f"sed -i 's/a/b/' {home_new_root}/e/notes.md", 0),
            ("执行员 sed -i 改工作根同名前缀的兄弟目录", "experiment-runner", f"sed -i 's/a/b/' {new_root}-other/notes.md", 1),
            ("执行员 python 只读仓内文件", "experiment-runner", "python3 -c \"print(open('crates/x/src/lib.rs').read())\"", 0),
            # os.replace / os.rename / shutil.move 挪进仓里也是就地改，scripts/ 起头的同算仓内（弄坏开关 python-move-into-repository-unseen 下前五格红）
            ("执行员 python os.replace 把草稿挪进 scripts/ 下的入库装置", "experiment-runner",
             f"python3 -c \"import os; os.replace('{new_root}/e/e1.py', 'scripts/experiment-apparatus/e1.py')\"", 1),
            ("执行员 python os.rename 挪进知识库（源是变量）", "experiment-runner", "python3 -c \"import os; os.rename(draft, '.claude/kb/a.md')\"", 1),
            ("执行员 python shutil.move 挪进仓根起的绝对路径", "experiment-runner",
             f"python3 - <<'EOF'\nimport shutil\nshutil.move('{new_root}/e/x.rs', '{edit_repository}/crates/x/src/lib.rs')\nEOF", 1),
            ("执行员 python shutil.move 用 dst= 挪进 scripts/", "experiment-runner", "python3 -c \"import shutil; shutil.move(source, dst='scripts/kb/x.py')\"", 1),
            ("执行员 python open(scripts/ 下的文件, 'w')", "experiment-runner", "python3 -c \"open('scripts/kb/x.py', 'w').write('x')\"", 1),
            ("执行员 python os.replace 的目标在草稿目录放行", "experiment-runner",
             f"python3 -c \"import os; os.replace('{new_root}/e/x.tmp', '{new_root}/e/x.py')\"", 0),
            ("执行员 python os.replace 把仓内文件挪进草稿目录放行", "experiment-runner",
             f"python3 -c \"import os; os.replace('scripts/kb/x.py', '{new_root}/e/x.py')\"", 0),
            # python 一半的相对路径从 cd 之后的目录算：cd 进草稿目录里的仓副本再写 crates/…、scripts/… 放行（弄坏开关 python-relative-path-ignores-cd 下前三格红）
            ("cd 到草稿目录里的仓副本之后 python 相对路径写 crates/ 放行", "experiment-runner",
             f"cd {new_root}/e/copy && python3 -c \"open('crates/x/src/lib.rs', 'w').write('x')\"", 0),
            ("cd 到草稿目录里的仓副本之后 python 相对路径挪进 scripts/ 放行", "experiment-runner",
             f"cd {new_root}/e/copy && python3 -c \"import os; os.replace('{new_root}/e/x.tmp', 'scripts/kb/x.py')\"", 0),
            ("cd 到草稿目录里的仓副本之后喂给 python 的 heredoc 相对路径写 scripts/ 放行", "experiment-runner",
             f"cd {new_root}/e/copy && python3 - <<'EOF'\nopen('scripts/kb/x.py', 'w').write('x')\nEOF", 0),
            ("cd 到仓根之后 python 相对路径写 crates/ 照拒", "experiment-runner",
             f"cd {edit_repository} && python3 -c \"open('crates/x/src/lib.rs', 'w').write('x')\"", 1),
            ("cd 到算不出的目录之后 python 相对路径写 crates/ 照旧按仓内判", "experiment-runner",
             "cd \"$WHERE\" && python3 -c \"open('crates/x/src/lib.rs', 'w').write('x')\"", 1),
            ("cd 到草稿目录里的仓副本之后 python 写仓根起的绝对路径照拒", "experiment-runner",
             f"cd {new_root}/e/copy && python3 -c \"open('{edit_repository}/crates/x/src/lib.rs', 'w').write('x')\"", 1),
            # heredoc 正文只在喂给 shell 或 python 时才是代码（弄坏开关 heredoc-body-judged-as-commands 下，前三格是 ⑧ 的 python 判法在整条文本上跑而红）
            ("往草稿目录 cat 写脚本，heredoc 正文里的 python open(仓内路径, 'w') 是数据", "experiment-runner",
             f"cat > {new_root}/e/x.py <<'EOF'\nopen('.claude/kb/a.md', 'w').write('x')\nEOF", 0),
            ("往草稿目录 cat 写笔记，heredoc 正文里的 python Path(仓内路径).write_text 是数据", "experiment-runner",
             f"cat > {new_root}/e/n.md <<'EOF'\nPath('records/x.md').write_text('y')\nEOF", 0),
            ("引号里 echo 的 open(仓内路径, 'w') 文字是数据", "experiment-runner", "echo \"open('.claude/kb/a.md', 'w')\"", 0),
            ("喂给 python 的 heredoc 里 open(仓内路径, 'w') 照拒", "experiment-runner", "python3 - <<'EOF'\nopen('.claude/kb/a.md', 'w').write('x')\nEOF", 1),
            ("喂给 bash 的 heredoc 里的 sed -i 照拒", "experiment-runner", "bash <<'EOF'\nsed -i 's/a/b/' .claude/kb/a.md\nEOF", 1),
            ("cat 写草稿脚本的 heredoc 正文里的 sed -i 是数据", "experiment-runner", f"cat > {new_root}/e/x.sh <<'EOF'\nsed -i 's/a/b/' .claude/kb/a.md\nEOF", 0),
            ("bash -c 里的 python3 -c 写仓内文件照拒", "experiment-runner", "bash -c \"python3 -c \\\"open('.claude/kb/a.md', 'w')\\\"\"", 1),
            ("没有 Edit 的回扫员不判", "sweep", "sed -i 's/a/b/' research/x.md", 0),
            ("主 agent 不判", None, "sed -i 's/a/b/' research/x.md", 0),
            # sed -i 的相对路径从 cd 之后的目录算（弄坏开关 sed-in-place-ignores-cd 下，前四格红；共用模块的 LIB_SHELL_WORDS_BREAK=double-slash-kept 下 // 开头写的那一格红）
            ("cd 到工作根下的草稿目录之后 sed -i 相对路径放行", "experiment-runner", f"cd {new_root}/e && sed -i 's/a/b/' x.py", 0),
            ("cd 到 /tmp/ 下的目录之后 sed -i 相对路径放行", "experiment-runner", "cd /tmp/claude-1000/e && sed -i 's/a/b/' x.py", 0),
            ("cd 到 // 开头写的 /tmp/ 下的目录之后 sed -i 相对路径放行", "experiment-runner", "cd //tmp/claude-1000/e && sed -i 's/a/b/' x.py", 0),
            ("cd 到前面赋过值的草稿目录之后 sed -i 相对路径放行", "experiment-runner", f"W={new_root}/e; cd $W && sed -i 's/a/b/' x.py", 0),
            ("cd 到仓内目录之后 sed -i 相对路径照拒", "experiment-runner", f"cd {edit_repository}/records && sed -i 's/a/b/' x.md", 1),
            ("不 cd 的仓内相对路径 sed -i 照拒", "experiment-runner", "sed -i 's/a/b/' records/x.md", 1),
            ("cd 到算不出的目录之后 sed -i 相对路径照旧按仓内判", "experiment-runner", "cd \"$WHERE\" && sed -i 's/a/b/' x.md", 1),
            ("先 cd 到草稿目录、再 cd 回仓内之后 sed -i 相对路径照拒", "experiment-runner",
             f"cd {new_root}/e && cd {edit_repository}/records && sed -i 's/a/b/' x.md", 1),
            ("cd 到草稿目录之后 sed -i 的相对路径用 .. 跳出工作根照拒", "experiment-runner", f"cd {new_root}/e && sed -i 's/a/b/' ../../../x.md", 1),
            ("cd 到草稿目录之后 sed -i 的目标带通配，照旧按仓内判", "experiment-runner", f"cd {new_root}/e && sed -i 's/a/b/' x*.py", 1),
            # 经管道喂给 python、bash 的 heredoc 正文是代码，喂给 grep 的是数据（弄坏开关 heredoc-pipe-fed-as-data 下前五格红）
            ("管道喂给 python3 - 的 heredoc 里 open(仓内路径, 'w') 照拒", "experiment-runner",
             "cat <<'EOF' | python3 -\nopen('.claude/kb/a.md', 'w').write('x')\nEOF", 1),
            ("管道喂给 python3（不带 -）的 heredoc 里 open(仓内路径, 'w') 照拒", "experiment-runner",
             "cat <<'EOF' | python3\nopen('.claude/kb/a.md', 'w').write('x')\nEOF", 1),
            ("管道喂给 python3 - 的 heredoc 里 Path(仓内路径).write_text 照拒", "experiment-runner",
             "cat <<'EOF' | python3 -\nfrom pathlib import Path\nPath('records/x.md').write_text('y')\nEOF", 1),
            ("echo 的文字管道喂给 python3 - 照拒", "experiment-runner", "echo \"open('.claude/kb/a.md', 'w')\" | python3 -", 1),
            ("管道喂给 bash 的 heredoc 里 sed -i 仓内文件照拒", "experiment-runner", "cat <<'EOF' | bash\nsed -i 's/a/b/' .claude/kb/a.md\nEOF", 1),
            ("管道喂给 grep 的 heredoc 正文是数据", "experiment-runner",
             "cat <<'EOF' | grep x\nopen('.claude/kb/a.md', 'w').write('x')\nsed -i 's/a/b/' .claude/kb/a.md\nEOF", 0),
            ("echo 的文字管道喂给 grep 是数据", "experiment-runner", "echo \"open('.claude/kb/a.md', 'w')\" | grep x", 0),
            ("管道喂给 python3 脚本文件的 heredoc 正文是数据（程序从文件读）", "experiment-runner",
             "cat <<'EOF' | python3 x.py\nopen('.claude/kb/a.md', 'w').write('x')\nEOF", 0),
        ]
        for label, agent, command, want in repository_edit_cases:
            results.append((f"就地改仓内文件:{label}", want, 1 if repository_in_place_edit_refusal(command, agent, edit_repository) else 0))
        # 命令开头的当前目录（hook 输入里的 cwd）：sed -i 的相对路径从那里算（弄坏开关 sed-in-place-ignores-cd 下第一格红）
        for label, command, start_directory, want in (
                ("命令开头的当前目录在草稿目录里，sed -i 相对路径放行", "sed -i 's/a/b/' x.py", f"{new_root}/e", 0),
                ("命令开头的当前目录在仓内，sed -i 相对路径照拒", "sed -i 's/a/b/' x.py", f"{edit_repository}/records", 1)):
            results.append((f"就地改仓内文件:{label}", want,
                            1 if repository_in_place_edit_refusal(command, "experiment-runner", edit_repository, start_directory) else 0))
        # ⑨ 先编后换：experiment-runner 在 Bash 里写主工作区 crates/ 下的 .rs：(说明, agent 类型, 命令, 该不该拒)
        harness_bin = os.path.join(edit_repository, "crates", "singlefs-verification", "src", "bin")
        os.makedirs(harness_bin)
        open(os.path.join(harness_bin, "e161_x.rs"), "w").write("fn main() {}\n")
        open(os.path.join(edit_repository, "crates", "mutations.tsv"), "w").write("# 表头\n")
        bin_path = "crates/singlefs-verification/src/bin/e161_x.rs"
        draft_path = f"{new_root}/runner-x/e161_x.rs"
        compile_first_cases = [
            ("执行员 cp 草稿副本盖主工作区的入库装置", "experiment-runner", f"cp {draft_path} {bin_path}", 1),
            ("执行员 cat > 新建入库装置", "experiment-runner", "cat > crates/singlefs-verification/src/bin/e999_new.rs <<'EOF'\nfn main() {}\nEOF", 1),
            ("执行员 >> 追加", "experiment-runner", f"echo '// x' >> {bin_path}", 1),
            ("执行员 tee -a", "experiment-runner", f"echo x | tee -a {bin_path}", 1),
            ("执行员 mv 草稿副本进 bin 目录", "experiment-runner", f"mv {draft_path} crates/singlefs-verification/src/bin/", 1),
            ("执行员 cd 进 bin 目录之后 cp 到 .", "experiment-runner", f"cd crates/singlefs-verification/src/bin && cp {draft_path} .", 1),
            ("执行员 sed -i", "experiment-runner", f"sed -i 's/a/b/' {bin_path}", 1),
            ("执行员 perl -pi -e", "experiment-runner", f"perl -pi -e 's/a/b/' {bin_path}", 1),
            ("执行员 rsync", "experiment-runner", f"rsync -a {draft_path} {bin_path}", 1),
            ("执行员 rm 入库装置", "experiment-runner", f"rm {bin_path}", 1),
            ("执行员 mv 把入库装置挪走", "experiment-runner", f"mv {bin_path} {new_root}/runner-x/", 1),
            ("执行员 bash -c 里 cp", "experiment-runner", f"bash -c 'cp {draft_path} {bin_path}'", 1),
            ("执行员变量里的目录", "experiment-runner", f"d=crates/singlefs-verification/src/bin; cp {draft_path} \"$d/e161_x.rs\"", 1),
            ("执行员目标算不出、对得上已有的入库装置", "experiment-runner", f"cp {draft_path} \"$DIR\"/crates/singlefs-verification/src/bin/e161_x.rs", 1),
            ("执行员 python shutil.copy", "experiment-runner",
             f"python3 -c \"import shutil; shutil.copy('{draft_path}', '{bin_path}')\"", 1),
            ("执行员 python open(…, 'a')", "experiment-runner", f"python3 -c \"open('{bin_path}', 'a').write('x')\"", 1),
            ("执行员 python os.replace", "experiment-runner", f"python3 -c \"import os; os.replace('{draft_path}', '{bin_path}')\"", 1),
            ("执行员经先编后换脚本换进来放行", "experiment-runner",
             f"bash scripts/hosts/capped.sh 16 python3 {COMPILE_THEN_SWAP_SCRIPT} {draft_path} {bin_path} --scratch {new_root}/runner-x", 0),
            ("执行员把主工作区那份拷进草稿目录放行", "experiment-runner", f"cp {bin_path} {new_root}/runner-x/", 0),
            ("执行员在草稿目录里改放行", "experiment-runner", f"sed -i 's/a/b/' {draft_path}", 0),
            ("执行员追加 crates/mutations.tsv 放行", "experiment-runner", "printf 'x\\n' >> crates/mutations.tsv", 0),
            ("执行员 grep 读入库装置放行", "experiment-runner", f"grep -n main {bin_path}", 0),
            ("执行员 python 只读入库装置放行", "experiment-runner", f"python3 -c \"print(open('{bin_path}').read())\"", 0),
            ("执行员引号里当数据写的放行", "experiment-runner", f"echo 'cp {draft_path} {bin_path}'", 0),
            ("实现员同样的 cp 放行", "implementation-writer", f"cp {draft_path} {bin_path}", 0),
            ("主 agent 同样的 cp 放行", None, f"cp {draft_path} {bin_path}", 0),
        ]
        for label, agent, command, want in compile_first_cases:
            results.append((f"先编后换:{label}", want, 1 if compile_first_refusal(command, agent, edit_repository) else 0))
        # 相对路径从 hook 输入的 cwd 起算，同 ⑧ ⑫（BASH_COMMAND_DETECTOR_BREAK=compile-first-ignores-cwd 下第一格红）：
        # cwd 是草稿目录里的仓副本时 crates/… 写的是副本；cwd 在主工作区的子目录里时相对路径从那里算
        copy_root = edit_repository + "-copy"
        results.append(("先编后换:执行员 cwd 在仓副本里、相对路径写 crates/ 下的 .rs 放行", 0,
                        1 if compile_first_refusal(f"cp {draft_path} {bin_path}", "experiment-runner", edit_repository, copy_root) else 0))
        results.append(("先编后换:执行员 cwd 在主工作区的 crates/ 里、相对路径写 .rs 拒", 1,
                        1 if compile_first_refusal(f"cp {draft_path} singlefs-verification/src/bin/e161_x.rs", "experiment-runner", edit_repository,
                                                   os.path.join(edit_repository, "crates")) else 0))
        # scripts/hosts/two-logs.sh <名> -- <命令> 包着的命令照原判法判（wrapped_commands 剥它，认法在 lib_project_code_runs.py 的 command_under_two_logs）：(说明, 判法, 命令, 该不该拒)
        two_logs_cases = [
            ("包着的前台没超时的等待循环拒", "等待循环", "bash scripts/hosts/two-logs.sh /tmp/n -- bash -c 'until grep -q x f; do sleep 10; done'", 1),
            ("--quiet 包着的前台没超时的等待循环拒", "等待循环", "bash scripts/hosts/two-logs.sh --quiet /tmp/n -- bash -c 'until grep -q x f; do sleep 10; done'", 1),
            ("包着的 kill 负进程号拒", "终止进程", "bash scripts/hosts/two-logs.sh /tmp/n -- kill -TERM -1234", 1),
            ("包着的 setsid -f 拒", "放出追踪", "bash scripts/hosts/two-logs.sh /tmp/n -- setsid -f bash long.sh", 1),
            ("包着的 cp 盖主工作区入库装置拒（先编后换）", "先编后换", f"bash scripts/hosts/two-logs.sh /tmp/n -- cp {draft_path} {bin_path}", 1),
            ("包着的普通命令放行（等待循环）", "等待循环", "bash scripts/hosts/two-logs.sh /tmp/n -- bash gate.sh", 0),
            ("包着的 timeout 等待循环放行", "等待循环", "bash scripts/hosts/two-logs.sh /tmp/n -- timeout 60 bash -c 'until grep -q x f; do sleep 10; done'", 0),
            ("--from-log 不跑命令放行（终止进程）", "终止进程", "bash scripts/hosts/two-logs.sh --from-log /tmp/n", 0),
            ("别处的同名拷贝不剥（终止进程）", "终止进程", "bash /tmp/elsewhere/two-logs.sh /tmp/n -- kill -TERM -1234", 0),
        ]
        two_logs_judges = {
            "等待循环": lambda command: wait_loop_refusal(command, False),
            "终止进程": process_signal_refusal,
            "放出追踪": detaching_refusal,
            "先编后换": lambda command: compile_first_refusal(command, "experiment-runner", edit_repository),
        }
        for label, judged, command, want in two_logs_cases:
            results.append((f"两道包装:{label}", want, 1 if two_logs_judges[judged](command) else 0))
    finally:
        shutil.rmtree(edit_repository, ignore_errors=True)
    # ⑫ 有写范围的项目子 agent 在 Bash 里写写范围之外的仓内文件：临时仓里放 clerk、kb-scribe、reporter（表里没有行）的定义和一张写范围表，
    #   同目录放一份拷贝出来的 hook 与它读的库，真实入口那几格（stdin:）也在这个仓里起；(说明, agent 类型, 命令, 该不该拒)
    scope_repository = os.path.realpath(os.path.join(work, "scope-repository"))
    scope_hooks = os.path.join(scope_repository, ".claude", "hooks")
    os.makedirs(scope_hooks)
    os.makedirs(os.path.join(scope_repository, ".claude", "agents"))
    for library_name in ("bash-command-detector.sh", "lib_shell_words.py", "lib_project_code_runs.py", "lib_write_scope.py", "lib_work_root.py", "lib_writer_lock.py"):
        shutil.copy(os.path.join(hook_dir, library_name), scope_hooks)
    for agent_name in ("clerk", "kb-scribe", "reporter"):
        open(os.path.join(scope_repository, ".claude", "agents", f"{agent_name}.md"), "w").write("---\ntools: Read, Edit, Write, Bash\n---\n")
    scope_table = os.path.join(scope_hooks, WRITE_SCOPE_TABLE_NAME)
    open(scope_table, "w").write("clerk\trecords/**\t样本\nclerk\tresearch/prompts/**\t样本\nclerk\t$SINGLEFS_WORK_ROOT/**\t样本\n"
                                 "kb-scribe\t.claude/kb/**\t样本\nkb-scribe\t$SINGLEFS_WORK_ROOT/**\t样本\n")
    open(os.path.join(scope_repository, "spec-out.json"), "w").write(json.dumps([{"file": "records/x.md", "old": "a", "new": "b"}, {"file": ".claude/kb/x.md", "old": "a", "new": "b"}]))
    open(os.path.join(scope_repository, "spec-in.json"), "w").write(json.dumps([{"file": "records/x.md", "old": "a", "new": "b"}, {"file": "research/prompts/y.md", "old": "a", "new": "b"}]))
    scope_cases = [
        ("clerk 用 replace-once.py 写知识库（现场：越出写范围）", "clerk", "python3 scripts/kb/replace-once.py .claude/kb/checks-owed.md a b", 1),
        ("clerk 直接执行 replace-once.py 写知识库", "clerk", "scripts/kb/replace-once.py .claude/kb/checks-owed.md a b", 1),
        ("clerk 用 replace-once.py 写自己写范围里的 records", "clerk", "python3 scripts/kb/replace-once.py records/x.md a b", 0),
        ("clerk 用 replace-once.py 写工作根下自己的草稿目录", "clerk", f"python3 scripts/kb/replace-once.py {new_root}/clerk-x/notes.md a b", 0),
        ("clerk 先 cd 到 records 再用相对路径写知识库", "clerk", "cd records && python3 ../scripts/kb/replace-once.py ../.claude/kb/x.md a b", 1),
        ("kb-scribe 用 replace-once.py 写知识库放行", "kb-scribe", "python3 scripts/kb/replace-once.py .claude/kb/x.md a b", 0),
        ("kb-scribe 用 replace-once.py 写 records 拒", "kb-scribe", "python3 scripts/kb/replace-once.py records/x.md a b", 1),
        ("clerk 用 insert-row.py 写知识库", "clerk", "python3 scripts/kb/insert-row.py .claude/kb/checks-owed.md --after '^# a' --line 'x'", 1),
        ("clerk 的 insert-row.py --line 值里的路径不当目标", "clerk", "python3 scripts/kb/insert-row.py records/x.md --after '^# a' --line '.claude/kb/y.md'", 0),
        ("clerk 的 insert-row.py --dry-run 不写", "clerk", "python3 scripts/kb/insert-row.py .claude/kb/checks-owed.md --after a --line x --dry-run", 0),
        ("clerk 的 replace-once.py --selftest 不写", "clerk", "python3 scripts/kb/replace-once.py --selftest", 0),
        ("clerk 用 replace-batch.py，规格点名知识库", "clerk", "python3 scripts/kb/replace-batch.py spec-out.json", 1),
        ("clerk 用 replace-batch.py，规格点名的都在写范围里", "clerk", "python3 scripts/kb/replace-batch.py spec-in.json", 0),
        ("clerk 的 replace-batch.py --dry-run 不写", "clerk", "python3 scripts/kb/replace-batch.py --dry-run spec-out.json", 0),
        # 脚本路径放在前面赋过值的变量里：代入之后照认（弄坏开关 scope-script-variable-unexpanded 下前四格红）
        ("clerk 脚本路径在变量里，用 replace-once.py 写知识库", "clerk", "S=scripts/kb/replace-once.py; python3 $S .claude/kb/x.md a b", 1),
        ("clerk 脚本路径在变量里，用 insert-row.py 写知识库", "clerk", "W=scripts/kb/insert-row.py; python3 \"$W\" .claude/kb/checks-owed.md --after a --line x", 1),
        ("clerk 脚本路径在变量里，用 replace-batch.py、规格点名知识库", "clerk", "B=scripts/kb/replace-batch.py; python3 ${B} spec-out.json", 1),
        ("clerk 直接执行变量里的 replace-once.py 写知识库", "clerk", "S=scripts/kb/replace-once.py; $S .claude/kb/x.md a b", 1),
        ("clerk 脚本路径在变量里，写自己写范围里的 records 放行", "clerk", "S=scripts/kb/replace-once.py; python3 $S records/x.md a b", 0),
        ("clerk 脚本路径在变量里、带 --dry-run 不写", "clerk", "S=scripts/kb/replace-once.py; python3 $S .claude/kb/x.md a b --dry-run", 0),
        ("clerk sed -i 改 crates", "clerk", "sed -i 's/a/b/' crates/x/src/lib.rs", 1),
        ("clerk sed -i 改 records 放行", "clerk", "sed -i 's/a/b/' records/x.md", 0),
        ("clerk 重定向 > 写 crates", "clerk", "echo x > crates/x/src/lib.rs", 1),
        ("clerk 重定向 >> 写知识库", "clerk", "echo x >> .claude/kb/a.md", 1),
        ("clerk tee 写 crates", "clerk", "echo x | tee crates/x/src/lib.rs", 1),
        ("clerk tee -a 写知识库", "clerk", "echo x | tee -a .claude/kb/a.md", 1),
        ("clerk cp 到知识库", "clerk", "cp records/a.md .claude/kb/a.md", 1),
        ("clerk cp 进知识库目录", "clerk", "cp records/a.md .claude/kb/", 1),
        ("clerk mv 到 crates", "clerk", "mv records/a.md crates/a.md", 1),
        ("clerk cp 到 records 放行", "clerk", "cp /tmp/a.md records/a.md", 0),
        ("clerk heredoc 写 records 放行", "clerk", "cat > records/ok.md <<'EOF'\nx\nEOF", 0),
        ("clerk heredoc 写知识库", "clerk", "cat > .claude/kb/ok.md <<'EOF'\nx\nEOF", 1),
        ("clerk 先 cd 进知识库目录再重定向", "clerk", "cd .claude/kb && echo x > a.md", 1),
        ("clerk bash -c 里的写", "clerk", "bash -c 'echo x > .claude/kb/a.md'", 1),
        ("clerk 变量里的目录（前面赋过值）", "clerk", "d=.claude/kb; echo x > $d/a.md", 1),
        ("clerk python open(…, 'w') 写知识库", "clerk", "python3 -c \"open('.claude/kb/x.md', 'w').write('x')\"", 1),
        ("clerk 写 /dev/null、2>&1 与 /tmp 放行", "clerk", "ls > /dev/null 2>&1; echo x > /tmp/y.out", 0),
        ("clerk 写工作根下的草稿目录放行", "clerk", f"echo x > {new_root}/clerk-x/a.md", 0),
        # heredoc 正文只在喂给 shell（bash、sh）或 python 时才是代码；喂给 cat、tee 的是数据，但它们的目标照判（弄坏开关 heredoc-body-judged-as-commands 下红）
        ("clerk 往草稿脚本 cat 写的 heredoc 正文里有 sed -i 知识库：正文是数据，放行", "clerk",
         f"cat > {new_root}/clerk-x/x.sh <<'EOF'\nsed -i 's/a/b/' .claude/kb/a.md\nEOF", 0),
        ("clerk 往草稿脚本 cat 写的 heredoc 正文里有重定向和 cp 到知识库：放行", "clerk",
         f"cat > {new_root}/clerk-x/x.sh <<'EOF'\necho x > .claude/kb/a.md\ncp a.md .claude/kb/b.md\nEOF", 0),
        ("clerk 往草稿脚本 tee 写的 heredoc 正文里有 sed -i 知识库：放行", "clerk",
         f"tee {new_root}/clerk-x/x.sh <<'EOF'\nsed -i 's/a/b/' .claude/kb/a.md\nEOF", 0),
        ("clerk 喂给 bash 的 heredoc 里同样一句 sed -i 知识库照拒", "clerk", "bash <<'EOF'\nsed -i 's/a/b/' .claude/kb/a.md\nEOF", 1),
        ("clerk 喂给 bash 的 heredoc 里的重定向写知识库照拒", "clerk", "bash <<'EOF'\necho x > .claude/kb/a.md\nEOF", 1),
        ("clerk 喂给 python 的 heredoc 里 open(知识库, 'w') 照拒", "clerk", "python3 - <<'EOF'\nopen('.claude/kb/a.md', 'w').write('x')\nEOF", 1),
        ("clerk 喂给 python 的 heredoc 里写自己写范围里的 records 放行", "clerk", "python3 - <<'EOF'\nopen('records/a.md', 'w').write('x')\nEOF", 0),
        ("clerk cat > 知识库 <<EOF 的目标本身照拒（正文是数据，目标不是）", "clerk", "cat > .claude/kb/a.md <<'EOF'\nx\nEOF", 1),
        ("clerk tee 知识库 <<EOF 的目标本身照拒", "clerk", "tee .claude/kb/a.md <<'EOF'\nx\nEOF", 1),
        ("clerk 只读知识库放行", "clerk", "cat .claude/kb/a.md | grep x", 0),
        ("clerk 引号里当数据写的放行", "clerk", "echo 'echo x > .claude/kb/a.md'", 0),
        ("clerk 经 two-logs.sh 包着写知识库", "clerk", "bash scripts/hosts/two-logs.sh /tmp/n -- tee .claude/kb/a.md", 1),
        ("clerk 的目标在没赋过值的变量里：拒", "clerk", "python3 scripts/kb/replace-once.py \"$TARGET\" a b", 1),
        ("clerk 重定向目标带通配：拒", "clerk", "echo x > .claude/kb/*.md", 1),
        ("clerk cd 到没赋过值的变量目录之后写相对路径：拒", "clerk", "cd \"$WHERE\" && echo x > a.md", 1),
        ("clerk cd 到没赋过值的变量目录之后写绝对路径的知识库：照拒", "clerk", f"cd \"$WHERE\" && echo x > {scope_repository}/.claude/kb/a.md", 1),
        # 经管道喂给 python、bash 的 heredoc 正文是代码，喂给 grep、python 脚本文件的是数据（弄坏开关 heredoc-pipe-fed-as-data 下带「照拒」的红）
        ("clerk 管道喂给 python3 - 的 heredoc 里 open(知识库, 'w') 照拒", "clerk", "cat <<'EOF' | python3 -\nopen('.claude/kb/a.md', 'w').write('x')\nEOF", 1),
        ("clerk 管道喂给 python3（不带 -）的 heredoc 里 open(知识库, 'w') 照拒", "clerk", "cat <<'EOF' | python3\nopen('.claude/kb/a.md', 'w').write('x')\nEOF", 1),
        ("clerk 管道喂给 python3 - 的 heredoc 里写自己写范围里的 records 放行", "clerk", "cat <<'EOF' | python3 -\nopen('records/a.md', 'w').write('x')\nEOF", 0),
        ("clerk cd 到 records 之后管道喂给 python3 - 的 heredoc 里相对路径写知识库照拒", "clerk",
         "cd records && cat <<'EOF' | python3 -\nopen('../.claude/kb/a.md', 'w').write('x')\nEOF", 1),
        ("clerk 管道喂给 bash 的 heredoc 里 sed -i 知识库照拒", "clerk", "cat <<'EOF' | bash\nsed -i 's/a/b/' .claude/kb/a.md\nEOF", 1),
        ("clerk 管道喂给 bash -s 的 heredoc 里重定向写知识库照拒", "clerk", "cat <<'EOF' | bash -s\necho x > .claude/kb/a.md\nEOF", 1),
        ("clerk echo 的文字管道喂给 python3 - 写知识库照拒", "clerk", "echo \"open('.claude/kb/a.md', 'w').write('x')\" | python3 -", 1),
        ("clerk printf 的文字管道喂给 python3 - 写知识库照拒", "clerk", "printf '%s\\n' \"open('.claude/kb/a.md', 'w')\" | python3 -", 1),
        ("clerk 自己的 heredoc 盖过管道：echo | python3 - <<EOF 里写知识库照拒", "clerk", "echo x | python3 - <<'EOF'\nopen('.claude/kb/a.md', 'w').write('x')\nEOF", 1),
        ("clerk 管道喂给 grep 的 heredoc 正文是数据，放行", "clerk",
         "cat <<'EOF' | grep x\necho x > .claude/kb/a.md\nopen('.claude/kb/a.md', 'w')\nsed -i 's/a/b/' .claude/kb/a.md\nEOF", 0),
        ("clerk echo 的文字管道喂给 grep 是数据，放行", "clerk", "echo \"open('.claude/kb/a.md', 'w')\" | grep x", 0),
        ("clerk 管道喂给 python3 脚本文件的 heredoc 正文是数据，放行", "clerk", "cat <<'EOF' | python3 x.py\nopen('.claude/kb/a.md', 'w').write('x')\nEOF", 0),
        # cd 到写死的目录之后 $PWD 按那个目录展开（弄坏开关 pwd-variable-unexpanded 下红）；mktemp -d 赋的变量落在 ${TMPDIR:-/tmp} 下（仓外）
        ("clerk cd 到知识库目录之后写 $PWD/… 照拒", "clerk", "cd .claude/kb && echo x > $PWD/a.md", 1),
        ("clerk cd 到知识库目录之后 cp 进 $PWD/… 照拒", "clerk", "cd .claude/kb && cp /tmp/a.md $PWD/b.md", 1),
        ("clerk cd 到 records 之后写 $PWD/… 放行", "clerk", "cd records && echo x > $PWD/a.md", 0),
        ("clerk mktemp -d 建的变量目录里写放行", "clerk", "W=$(mktemp -d) && cp -a records/a.md $W/", 0),
        ("clerk mktemp -d 之后 cd 进去再写相对路径放行", "clerk", "W=$(mktemp -d) && cd \"$W\" && echo x > a.md", 0),
        ("主 agent 写知识库放行", None, "python3 scripts/kb/replace-once.py .claude/kb/x.md a b", 0),
        # 没有项目定义的 agent 只许写工作根（弄坏开关 builtin-agents-unjudged 下「拒」的六格红；会话临时目录根下那两格另在 SINGLEFS_WORK_ROOT_BREAK=legacy-root-accepted 下红）
        ("没有项目定义的内置 agent 写仓里拒", "Explore", "echo x > crates/x.rs", 1),
        ("没有项目定义的内置 agent 写仓外别处拒", "general-purpose", "echo x > /var/tmp/singlefs-builtin-elsewhere/a.md", 1),
        ("没有项目定义的内置 agent 写会话临时目录根拒（scratchpad 之外）", "general-purpose", f"echo x > /tmp/claude-{os.getuid()}/singlefs-builtin-x/a.md", 1),
        ("没有项目定义的内置 agent 写会话 scratchpad 拒", "Explore", f"echo x > /tmp/claude-{os.getuid()}/-home-sample/selftest-session/scratchpad/a.md", 1),
        ("没有项目定义的内置 agent 用 replace-once.py 写仓里拒", "general-purpose", "python3 scripts/kb/replace-once.py records/x.md a b", 1),
        ("定义已删的旧实例写仓里拒", "gate-triage", "cp /tmp/a.md records/a.md", 1),
        ("没有项目定义的内置 agent 写工作根放行", "Explore", f"echo x > {new_root}/explore-x/a.md", 0),
        ("没有项目定义的内置 agent 写 /dev/null、2>&1 与只读放行", "Explore", "grep -rn x . > /dev/null 2>&1; cat records/a.md | head -3", 0),
        ("表里没有行的项目定义不判", "reporter", "echo x > crates/x.rs", 0),
        # eval 的代码与启动器里面那条照判，xargs、find -exec 读进来的落点算不出照拒（一百三十七节件 1b 第 4 条；弄坏开关 scope-launcher-writes-unseen 下「拒」的五格与下面工作根那一段的 xargs 一格红）
        ("一百三十七：clerk eval 字面代码里 sed -i 知识库，拒", "clerk", "eval 'sed -i s/a/b/ .claude/kb/a.md'", 1),
        ("一百三十七：clerk find -exec sed -i 读进来的路径，落点算不出，拒", "clerk", "find .claude/kb -name a.md -exec sed -i s/a/b/ {} ';'", 1),
        ("一百三十七：clerk echo 喂给 xargs sed -i，落点算不出，拒", "clerk", "echo .claude/kb/a.md | xargs sed -i s/a/b/", 1),
        ("一百三十七：clerk xargs sed -i 写范围里的 records 也拒（落点算不出）", "clerk", "echo records/a.md | xargs sed -i s/a/b/", 1),
        ("一百三十七：clerk find -exec cp 到写死的知识库路径，拒", "clerk", "find records -name a.md -exec cp {} .claude/kb/b.md ';'", 1),
        ("一百三十七：clerk eval 字面代码写 records 放行", "clerk", "eval 'echo x > records/a.md'", 0),
        ("一百三十七：clerk xargs 起只读的 grep 放行", "clerk", "echo records/a.md | xargs grep -l x", 0),
        ("一百三十七：clerk find -exec cp 到写死的 records 路径放行", "clerk", "find records -name a.md -exec cp {} records/b.md ';'", 0),
    ]
    for label, agent, command, want in scope_cases:
        case_verdict = bash_scope_refusal(command, agent, scope_repository, scope_repository, scope_table)
        results.append((f"Bash写范围:{label}", want, 1 if case_verdict.out_of_scope or case_verdict.undeterminable else 0))
    # ⑫ 认 ledger-append-section.py：它写的是仓根下登记表 current 那一份账本。另起一个假仓（不往 scope_repository 里建 records/：上面几格 cd records 的判法看目录在不在），
    #   放 clerk、kb-scribe 的定义、登记表与账本，写范围表用上面那一张；仓外再放一份带登记表的副本给 --root 指。(说明, agent 类型, 命令, 该不该拒)
    #   弄坏开关 scope-ledger-append-unseen 下「拒」的三格红
    ledger_scope_repository = os.path.realpath(os.path.join(work, "ledger-scope-repository"))
    ledger_scope_copy = os.path.realpath(os.path.join(work, "ledger-scope-copy"))
    for ledger_scope_root in (ledger_scope_repository, ledger_scope_copy):
        os.makedirs(os.path.join(ledger_scope_root, ".claude", "agents"))
        os.makedirs(os.path.join(ledger_scope_root, ".claude", "gate.d"))
        os.makedirs(os.path.join(ledger_scope_root, "records"))
        with open(os.path.join(ledger_scope_root, LEDGER_REGISTRY_PATH), "w", encoding="utf-8") as handle:
            handle.write("current\trecords/ledger.md\t-\t样本里的现行账本，自证用\n")
        with open(os.path.join(ledger_scope_root, "records", "ledger.md"), "w", encoding="utf-8") as handle:
            handle.write("# 账本\n")
    for agent_name in ("clerk", "kb-scribe"):
        with open(os.path.join(ledger_scope_repository, ".claude", "agents", f"{agent_name}.md"), "w", encoding="utf-8") as handle:
            handle.write("---\ntools: Read, Edit, Write, Bash\n---\n")
    ledger_append_command = "python3 scripts/kb/ledger-append-section.py --title x --body-file /tmp/body.md"
    ledger_scope_cases = [
        ("写范围里没有 records/ 的 kb-scribe 起 ledger-append-section.py 写现行账本，拒", "kb-scribe", ledger_append_command, 1),
        ("kb-scribe 直接执行 ledger-append-section.py 写现行账本，拒", "kb-scribe", "scripts/kb/ledger-append-section.py --title x --body-file /tmp/body.md", 1),
        ("kb-scribe 的 ledger-append-section.py --root 写成没赋过值的变量，落点算不出，拒", "kb-scribe",
         "python3 scripts/kb/ledger-append-section.py --root \"$LEDGER_ROOT_UNSET\" --title x --body-file /tmp/body.md", 1),
        ("写范围有 records/ 的 clerk 起 ledger-append-section.py 放行", "clerk", ledger_append_command, 0),
        ("主 agent 起 ledger-append-section.py 放行", None, ledger_append_command, 0),
        ("kb-scribe 的 ledger-append-section.py --dry-run 不写，放行", "kb-scribe", ledger_append_command + " --dry-run", 0),
        ("kb-scribe 的 ledger-append-section.py --root= 点名仓外的副本，放行", "kb-scribe",
         f"python3 scripts/kb/ledger-append-section.py --root={ledger_scope_copy} --title x --body-file /tmp/body.md", 0),
    ]
    for label, agent, command, want in ledger_scope_cases:
        case_verdict = bash_scope_refusal(command, agent, ledger_scope_repository, ledger_scope_repository, scope_table)
        results.append((f"Bash写范围:{label}", want, 1 if case_verdict.out_of_scope or case_verdict.undeterminable else 0))
    # ⑫ 认 append-record.py：它写的是第一个不是选项的参数 <记录>（--root 的值不算）。用上面那个假仓（有 records/ledger.md、clerk 与 kb-scribe 的定义）与同一张写范围表；
    #   弄坏开关 scope-record-append-unseen 下「拒」的五格红，scope-record-append-root-as-target 下 --root 写在记录前面那一格红。(说明, agent 类型, 命令, 该不该拒)
    record_append_command = "python3 scripts/kb/append-record.py records/ledger.md /tmp/text.md"
    record_append_scope_cases = [
        ("写范围里没有 records/ 的 kb-scribe 起 append-record.py 往记录追加，拒", "kb-scribe", record_append_command, 1),
        ("kb-scribe 直接执行 append-record.py 往记录追加，拒", "kb-scribe", "scripts/kb/append-record.py records/ledger.md /tmp/text.md", 1),
        ("kb-scribe cd 进 records 之后按相对路径起 append-record.py，拒", "kb-scribe", "cd records && python3 ../scripts/kb/append-record.py ledger.md -", 1),
        ("kb-scribe 的 append-record.py 先写 --root 再写记录，落点照第一个位置参数认，拒", "kb-scribe",
         f"python3 scripts/kb/append-record.py --root {ledger_scope_copy} records/ledger.md /tmp/text.md", 1),
        ("kb-scribe 的 append-record.py 记录写成没赋过值的变量，落点算不出，拒", "kb-scribe",
         "python3 scripts/kb/append-record.py \"$RECORD_UNSET\" /tmp/text.md", 1),
        ("写范围有 records/ 的 clerk 起 append-record.py 放行", "clerk", record_append_command, 0),
        ("主 agent 起 append-record.py 放行", None, record_append_command, 0),
        ("kb-scribe 的 append-record.py --dry-run 不写，放行", "kb-scribe", record_append_command + " --dry-run", 0),
        ("kb-scribe 的 append-record.py --selftest 不写，放行", "kb-scribe", "python3 scripts/kb/append-record.py --selftest", 0),
        ("kb-scribe 的 append-record.py 记录在仓外的副本里，不判", "kb-scribe",
         f"python3 scripts/kb/append-record.py {ledger_scope_copy}/records/ledger.md /tmp/text.md", 0),
    ]
    for label, agent, command, want in record_append_scope_cases:
        case_verdict = bash_scope_refusal(command, agent, ledger_scope_repository, ledger_scope_repository, scope_table)
        results.append((f"Bash写范围:{label}", want, 1 if case_verdict.out_of_scope or case_verdict.undeterminable else 0))
    # 钩子进程自己的当前目录在工作根下时，sed -i 展开不了的目标照算不出拒（一百三十七节件 1b；弄坏开关 sed-unknown-target-resolved-from-cwd 下这两格红）
    if work_roots is not None and os.path.isdir(work_roots.work_root()):
        process_directory = os.getcwd()
        os.chdir(work_roots.work_root())
        try:
            for label, command in (("一百三十七：钩子进程在工作根下，clerk sed -i 没赋过值的变量照拒", "sed -i s/a/b/ $F"),
                                   ("一百三十七：钩子进程在工作根下，clerk xargs sed -i 照拒", "echo records/a.md | xargs sed -i s/a/b/")):
                case_verdict = bash_scope_refusal(command, "clerk", scope_repository, scope_repository, scope_table)
                results.append((f"Bash写范围:{label}", 1, 1 if case_verdict.out_of_scope or case_verdict.undeterminable else 0))
        finally:
            os.chdir(process_directory)
    # 目标里有变量、命令替换、通配的：落点算得出的照判，算不出的拒（弄坏开关 scope-undeterminable-only-noted 下「拒」那几格变成「记」、红）；(说明, agent 类型, 命令, 结局：拒 / 放行 / 记)
    def scope_outcome(command, agent="clerk", identity=None):
        verdict = bash_scope_refusal(command, agent, scope_repository, scope_repository, scope_table, identity)
        return "拒" if verdict.out_of_scope or verdict.undeterminable else "记" if verdict.noted else "放行"
    landing_directory = os.path.join(work, "landing")          # 仓外的目录里放一个指到仓内知识库的符号链接：`<链接>/../x` 照 shell 逐段解开回到仓里
    os.makedirs(os.path.join(scope_repository, ".claude", "kb", "sub"), exist_ok=True)
    os.makedirs(landing_directory, exist_ok=True)
    os.symlink(os.path.join(scope_repository, ".claude", "kb", "sub"), os.path.join(landing_directory, "kb-link"))
    climb_back = os.path.relpath(os.path.join(scope_repository, ".claude", "kb", "a.md"), os.path.join(new_root, "clerk-x"))   # 从草稿目录爬回仓里知识库的 ../…
    # 仓里的链接：范围内的指到仓里范围外（records/kb-link → .claude/kb）、范围内的指回范围内、范围外的指回范围内、范围外的指到仓外
    os.makedirs(os.path.join(scope_repository, "records", "sub"), exist_ok=True)
    os.symlink(os.path.join(scope_repository, ".claude", "kb"), os.path.join(scope_repository, "records", "kb-link"))
    os.symlink(os.path.join(scope_repository, "records", "sub"), os.path.join(scope_repository, "records", "back-link"))
    os.symlink(os.path.join(scope_repository, "records"), os.path.join(scope_repository, ".claude", "kb", "records-link"))
    os.symlink(landing_directory, os.path.join(scope_repository, ".claude", "kb", "outside-link"))
    scope_dynamic_cases = [
        # python3 起的脚本词代入之后还算不出、第一个参数落在本仓里的，照算不出拒（弄坏开关 scope-script-variable-unexpanded 下前三格变成「放行」、
        # scope-undeterminable-only-noted 下变成「记」，都红）；第一个参数在仓外、带 --dry-run、$HOME 下的脚本不判
        ("python3 起没赋过值的变量里的脚本、第一个参数落在知识库，拒", "clerk", "python3 \"$WRITER\" .claude/kb/x.md a b", "拒"),
        ("python3 起没赋过值的变量里的脚本、第一个参数落在写范围里的 records 也拒", "clerk", "python3 $WRITER records/x.md a b", "拒"),
        ("python3 起命令替换给的脚本、第一个参数是仓根起的绝对路径，拒", "clerk", f"python3 $(ls scripts/kb/*.py | head -1) {scope_repository}/.claude/kb/x.md a b", "拒"),
        ("python3 起没赋过值的变量里的脚本、第一个参数在仓外，放行", "clerk", "python3 \"$WRITER\" /tmp/x.md a b", "放行"),
        ("python3 起没赋过值的变量里的脚本、带 --dry-run，放行", "clerk", "python3 \"$WRITER\" .claude/kb/x.md a b --dry-run", "放行"),
        ("python3 起 $HOME 下的脚本不当算不出，放行", "clerk", "python3 $HOME/tools/report.py records/x.md", "放行"),
        ("目标在没赋过值的变量里，拒", "clerk", "python3 scripts/kb/replace-once.py \"$TARGET\" a b", "拒"),
        ("重定向目标带通配，拒", "clerk", "echo x > .claude/kb/*.md", "拒"),
        ("replace-batch.py 规格读不到，拒", "clerk", "python3 scripts/kb/replace-batch.py no-such-spec.json", "拒"),
        ("cd 到没赋过值的变量目录之后写相对路径，拒", "clerk", "cd \"$WHERE\" && echo x > a.md", "拒"),
        ("cd 算不出之后 python 写相对路径，拒", "clerk", "cd \"$WHERE\" && python3 -c \"open('a.md', 'w').write('x')\"", "拒"),
        # 例：for 列表写死，但目标里有命令替换 $(echo $f | tr / _)
        ("检出原文：mkdir -p <草稿目录>/originals && for f in …; do cp -p \"$f\" \"<草稿目录>/originals/$(echo $f | tr / _)\"; done，拒", "clerk",
         f"mkdir -p {new_root}/m4-per-run-build-dirs/originals && for f in scripts/gate/merge-and-verify.sh scripts/mutation/crates-mutation-rows.py "
         f"scripts/crash/crash-point-replay.sh; do cp -p \"$f\" \"{new_root}/m4-per-run-build-dirs/originals/$(echo $f | tr / _)\"; done", "拒"),
        ("检出原文：S=<草稿目录> && mkdir -p $S/originals && for f in …; do cp -- \"$f\" \"$S/originals/$(echo $f | tr / _)\"; done，拒", "clerk",
         f"S={new_root}/m4-completion-cleanup-hooks && mkdir -p $S/originals && for f in .claude/settings.json .claude/hooks/selftest-registry.tsv "
         f".claude/agent-common.md; do cp -- \"$f\" \"$S/originals/$(echo $f | tr / _)\"; done", "拒"),
        # 写死的词的循环变量逐个取值（弄坏开关 scope-loop-values-unknown 下这几格红）
        ("for b in 写死的词; do … > <草稿目录>/breaks/$b.out，放行", "clerk", f"for b in one two three; do echo x > {new_root}/clerk-x/breaks/$b.out; done", "放行"),
        ("cd 到写死的范围内目录之后 for 循环（列表是写死的名字）里写 $d/…，放行", "clerk", "cd records && for d in a b; do echo x > $d/.gate-cells; done", "放行"),
        ("for f in 写死的仓内路径; do cp $f <草稿目录>/; done，放行", "clerk", f"for f in records/a.md .claude/kb/b.md; do cp \"$f\" {new_root}/clerk-x/; done", "放行"),
        ("S=<草稿目录> 之后 for f in 写死的词 写 $S/$f.out，放行", "clerk", f"S={new_root}/clerk-x && for f in a b; do echo x > \"$S/$f.out\"; done", "放行"),
        # 取值里的 .. 与 / 照逐个落点判（弄坏开关 scope-loop-landings-trusted 下这几格红）
        ("for 的列表里有爬回仓里的 ..，落在写范围外，拒", "clerk", f"for f in ok {climb_back}; do echo x > {new_root}/clerk-x/$f; done", "拒"),
        ("for 的列表里有 .. 段、落点回到写范围之外，拒", "clerk", "cd records && for d in a ../b; do echo x > $d/a.md; done", "拒"),
        ("for 的列表里有写范围外的仓内路径，cp 写到它，拒", "clerk", "for f in records/a.md .claude/kb/b.md; do cp /tmp/x \"$f\"; done", "拒"),
        # 变量来源不明的：静态前缀在哪都拒（静态前缀在仓外或范围里也拒：变量的值可以带 /.. 爬回仓里）
        ("f 来源不明的 > <草稿目录>/out-$f，拒", "clerk", f"echo x > {new_root}/clerk-x/out-$f", "拒"),
        ("变量来源不明、静态前缀在工作根下（仓外），拒", "clerk", f"echo x > {new_root}/clerk-x/breaks/$b.out", "拒"),
        ("变量来源不明、静态前缀在 /tmp 下（仓外），拒", "clerk", "echo x > /tmp/claude-1000/s/scratchpad/$f.out", "拒"),
        ("变量来源不明、静态前缀已在写范围里，拒", "clerk", "echo x > records/n-$f.md", "拒"),
        ("命令替换在最后一段、静态前缀已在写范围里，拒", "clerk", "echo x > records/n-$(date +%F).md", "拒"),
        ("cp 进范围内目录、源名算不出，拒", "clerk", "cp $SRC records/", "拒"),
        ("cd 到写死的范围内目录之后写带来源不明变量的相对路径，拒", "clerk", "cd records && echo x > gz-$k/.gate-cells", "拒"),
        ("cd 到写死的范围内目录之后写变量开头的相对路径，拒", "clerk", "cd records && echo x > $k/.gate-cells", "拒"),
        ("for 循环的列表里有算不出的词，拒", "clerk", "cd records && for d in $(ls); do echo x > $d/a.md; done", "拒"),
        ("通配目标在范围内目录下，拒", "clerk", "echo x > records/*.md", "拒"),
        ("静态前缀在仓内、写范围之外，拒", "clerk", "echo x > .claude/kb/$x.md", "拒"),
        ("目标以变量开头，拒", "clerk", "echo x > $REPO/.claude/kb/$x.md", "拒"),
        ("变量单独占一段，拒", "clerk", "echo x > records/$x", "拒"),
        ("变量之后有字面的 .. 段，拒", "clerk", "echo x > records/n-$x/../../.claude/kb/a.md", "拒"),
        ("仓外目录下变量单独占一段，拒", "clerk", f"echo x > {new_root}/clerk-x/$f", "拒"),
        ("cd 到写死的范围外目录之后写带变量的相对路径，拒", "clerk", "cd .claude/kb && echo x > gz-$k/a.md", "拒"),
        ("cd 到算不出的目录之后写带变量的相对路径，拒", "clerk", "cd \"$WHERE\" && echo x > gz-$k/a.md", "拒"),
        ("cd 进循环变量拼的目录之后写相对路径，拒", "clerk", f"cd {new_root}/clerk-x && for d in a b; do cd $d; echo x > y; done", "拒"),
        ("目标明确的放行", "clerk", "echo x > records/a.md", "放行"),
        ("目标明确、写范围之外的拒", "clerk", "echo x > crates/a.rs", "拒"),
        # 落点照 shell 逐段解开符号链接（弄坏开关 scope-landing-lexical-only 下这一格红）
        ("仓外的符号链接指到仓内知识库，<链接>/../x 回到仓里，拒", "clerk", f"echo x > {landing_directory}/kb-link/../x.md", "拒"),
        # 仓里的链接按解开之后的落点判，与写闸同一份判法（弄坏开关 scope-landing-lexical-only 下除「范围内的链接指回范围内」之外这几格红）
        ("范围内的链接指到仓里范围外的目录，经它 > 写，拒", "clerk", "echo x > records/kb-link/a.md", "拒"),
        ("范围内的链接指到仓里范围外的目录，经它 cp 写，拒", "clerk", "cp /tmp/a.md records/kb-link/a.md", "拒"),
        ("范围内的链接指到仓里范围外的目录，经它 replace-once.py 写，拒", "clerk", "python3 scripts/kb/replace-once.py records/kb-link/a.md a b", "拒"),
        ("范围内的链接指到仓里范围外的目录，cp 进链接（目标是目录），拒", "clerk", "cp /tmp/a.md records/kb-link/", "拒"),
        ("范围内的链接指回范围内，经它写，放行", "clerk", "echo x > records/back-link/a.md", "放行"),
        ("范围外的链接指回范围内，经它写，放行", "clerk", "echo x > .claude/kb/records-link/a.md", "放行"),
        ("范围外的链接指到仓外，经它写的落点在仓外，不判", "clerk", "echo x > .claude/kb/outside-link/a.md", "放行"),
        ("$HOME 照 hook 的环境展开，落在仓外放行", "clerk", "echo x > $HOME/bash-command-detector-selftest-probe.out", "放行"),
        # $PWD：前面有算得出的 cd 时按那个目录展开（弄坏开关 pwd-variable-unexpanded 下前四格红）；没有 cd、cd 算不出的拒
        ("cd 到写死的草稿目录之后写 $PWD/… 不记", "clerk", f"cd {new_root}/clerk-x && echo x > $PWD/a.md", "放行"),
        ("cd 到写死的范围内目录之后写 $PWD/… 不记", "clerk", "cd records && echo x > $PWD/a.md", "放行"),
        ("cd 到写死的草稿目录之后 cp 进 $PWD/子目录/ 不记", "clerk", f"cd {new_root}/clerk-x && cp a.md $PWD/sub/", "放行"),
        ("cd 到写死的草稿目录之后 T=$PWD/x 再写 $T/… 不记", "clerk", f"cd {new_root}/clerk-x && T=$PWD/x && echo x > $T/a.md", "放行"),
        ("没有 cd 的 $PWD，拒", "clerk", "echo x > $PWD/a.md", "拒"),
        ("cd 到算不出的目录之后 $PWD，拒", "clerk", "cd \"$WHERE\" && echo x > $PWD/a.md", "拒"),
        ("先 cd 到写死的目录、再 cd 到算不出的目录，$PWD 作废，拒", "clerk", "cd records && cd \"$WHERE\" && echo x > $PWD/a.md", "拒"),
        ("cd 到写死的草稿目录之后 $PWD/子目录/$p，$p 来源不明，拒", "clerk", f"cd {new_root}/clerk-x && cp a.md $PWD/sub/$p", "拒"),
        # mktemp -d 赋的变量当作落在 ${TMPDIR:-/tmp} 下（弄坏开关 mktemp-directory-unknown 下前六格红）；别的写法的 mktemp 拒
        ("W=$(mktemp -d) 之后 cp 进 $W/ 不记", "clerk", "W=$(mktemp -d) && cp -a records/a.md $W/", "放行"),
        ("W=`mktemp -d`（反引号）之后写 $W/… 不记", "clerk", "W=`mktemp -d`; echo x > $W/a.md", "放行"),
        ("export W=$(mktemp -d) 之后写 $W/… 不记", "clerk", "export W=$(mktemp -d); echo x > $W/a.md", "放行"),
        ("mktemp -d 之后 cd 进去写相对路径不记", "clerk", "W=$(mktemp -d) && cd \"$W\" && echo x > a.md", "放行"),
        ("mktemp -d -t 带模板不记", "clerk", "W=$(mktemp -d -t gate.XXXXXX) && echo x > $W/a.md", "放行"),
        ("mktemp --directory 不记", "clerk", "W=$(mktemp --directory) && echo x > $W/a.md", "放行"),
        ("mktemp -d -p 指的目录算不出，拒", "clerk", "W=$(mktemp -d -p \"$BASE\") && echo x > $W/a.md", "拒"),
        ("mktemp 不带 -d 建的是文件，拒", "clerk", "W=$(mktemp) && echo x > $W/a.md", "拒"),
        ("mktemp -d 的模板在当前目录里，拒", "clerk", "W=$(mktemp -d work.XXXXXX) && echo x > $W/a.md", "拒"),
        # 循环变量读的是什么算不出（while read 读文件，for 的列表来自变量）：拒，前面赋过的同名变量也作废
        ("while read 的循环变量，拒", "clerk", f"while read f; do cp a.md {new_root}/clerk-x/tree/$f; done < files.txt", "拒"),
        ("for 的列表来自变量，拒", "clerk", f"for f in $FILES; do cp a.md {new_root}/clerk-x/tree/$f; done", "拒"),
        ("前面赋过写死值的变量又被 for $列表 改写，拒", "clerk", f"f=ok; for f in $FILES; do cp a.md {new_root}/clerk-x/tree/$f; done", "拒"),
        ("主 agent 的目标算不出也不判", None, "echo x > $F", "放行"),
    ]
    for label, agent, command, want in scope_dynamic_cases:
        results.append((f"Bash写范围:{label}", want, scope_outcome(command, agent)))
    # mktemp -d 的模板在这个子 agent 认领的草稿目录里（拆分提案第九十节 ⑰）：工作根下建两个第一层目录，一个的归属文件写会话 s-own（own/ 的锁认领给 agent-own，
    #   other/ 的锁认领给 agent-other，receipt/ 的锁没有认领文件、靠派发回执认），另一个的归属文件写别的会话（锁认领给 agent-own）；
    #   弄坏开关 mktemp-scratch-template-unknown 下「不记」的四格与 stdin 那一格红，mktemp-template-owner-unchecked 下「别的子 agent」「别的会话」「派发回执对不上」三格红
    locks = writer_locks_library()
    owned_scratch, foreign_scratch = None, None
    try:
        os.makedirs(new_root, exist_ok=True)
        owned_scratch = tempfile.mkdtemp(prefix="bash-command-detector-selftest-", dir=new_root)
        foreign_scratch = tempfile.mkdtemp(prefix="bash-command-detector-selftest-", dir=new_root)
        own_directory, other_directory = os.path.join(owned_scratch, "own"), os.path.join(owned_scratch, "other")
        for first_level, session in ((owned_scratch, "s-own"), (foreign_scratch, "s-foreign")):
            open(os.path.join(first_level, ".owner"), "w", encoding="utf-8").write(f"session: {session}\ncreated: 2026-10-22\npurpose: 自证\n")
        for directory, holder in ((own_directory, "agent-own"), (other_directory, "agent-other"), (foreign_scratch, "agent-own")):
            lock = locks.new_lock("s-own", f"自证 {holder}", "clerk", os.path.join(work, "no-transcript.jsonl"))
            locks.create_lock(directory, lock)
            locks.write_exclusively(locks.claim_path(directory, lock), {"agent_id": holder, "claimed_at": 0})
        identity = {"agent_id": "agent-own", "session_id": "s-own", "transcript_path": os.path.join(work, "no-transcript.jsonl")}
        # 还没有认领文件的那把锁：主会话记录里 agent-receipt 的派发回执（提示与锁对得上、时刻不早于锁）认它
        receipt_directory, receipt_transcript = os.path.join(owned_scratch, "receipt"), os.path.join(work, "receipt-session.jsonl")
        receipt_lock = locks.new_lock("s-own", "自证 receipt", "clerk", receipt_transcript)
        locks.create_lock(receipt_directory, receipt_lock)
        with open(receipt_transcript, "w", encoding="utf-8") as handle:
            handle.write(json.dumps({"timestamp": datetime.fromtimestamp(receipt_lock["dispatched_at"] + 1, timezone.utc).isoformat(),
                                     "toolUseResult": {"status": "async_launched", "agentId": "agent-receipt", "prompt": "自证 receipt"}}, ensure_ascii=False) + "\n")
        receipt_identity = {"agent_id": "agent-receipt", "session_id": "s-own", "transcript_path": receipt_transcript}
        scratch_mktemp_cases = [
            ("模板在自己认领的草稿目录里：T=$(mktemp -d $W/名.XXXXXX) 之后 cp 进 $T/ 不记", identity,
             f"W={own_directory}; T=$(mktemp -d $W/rs.XXXXXX); cp -a records/a.md $T/", "放行"),
            ("模板在自己认领的草稿目录里：直接写进 $(mktemp -d …)/文件 不记", identity, f"echo x > $(mktemp -d {own_directory}/real.XXXXXX)/cell.py", "放行"),
            ("cd 进自己认领的草稿目录之后模板写相对路径不记", identity, f"cd {own_directory} && T=$(mktemp -d runs.XXXXXX) && echo x > $T/a.md", "放行"),
            ("还没有认领文件、派发回执对得上那把锁：模板在它的草稿目录里不记", receipt_identity, f"T=$(mktemp -d {receipt_directory}/rs.XXXXXX); echo x > $T/a.md", "放行"),
            ("派发回执对不上那把锁（别的子 agent）：拒", {**receipt_identity, "agent_id": "agent-stranger"}, f"T=$(mktemp -d {receipt_directory}/rs.XXXXXX); echo x > $T/a.md", "拒"),
            ("模板在别的子 agent 认领的草稿目录里，拒", identity, f"T=$(mktemp -d {other_directory}/rs.XXXXXX); cp -a records/a.md $T/", "拒"),
            ("模板目录的归属文件是别的会话，拒", identity, f"T=$(mktemp -d {foreign_scratch}/rs.XXXXXX); echo x > $T/a.md", "拒"),
            ("模板在 /tmp 下（不是草稿目录），拒", identity, "T=$(mktemp -d /tmp/probe.XXXXXX); echo x > $T/a.md", "拒"),
            ("hook 输入里没有 agent_id，模板在草稿目录里也拒", None, f"W={own_directory}; T=$(mktemp -d $W/rs.XXXXXX); cp -a records/a.md $T/", "拒"),
            # 拆分提案第九十节 ㉕：-p、--tmpdir= 与套在 $(cd … && pwd -P) 里的形状
            ("-p 指到自己认领的草稿目录：T=$(mktemp -d -p \"$W\") 之后写进 $T/ 不记", identity, f"W={own_directory}; T=$(mktemp -d -p \"$W\"); echo x > $T/a.md", "放行"),
            ("--tmpdir= 指到自己认领的草稿目录（带模板名）：cp 进 $T/ 不记", identity,
             f"T=$(mktemp -d --tmpdir={own_directory} rs.XXXXXX); cp -a records/a.md $T/", "放行"),
            ("cd 进自己认领的草稿目录之后 -p . 不记", identity, f"cd {own_directory} && T=$(mktemp -d -p .) && echo x > $T/a.md", "放行"),
            ("直接写进 $(mktemp -d -p …)/文件 不记", identity, f"echo x > $(mktemp -d -p {own_directory})/cell.py", "放行"),
            ("不带值的 --tmpdir：W=$(mktemp -d --tmpdir) 之后写进 $W/ 不记", identity, "W=$(mktemp -d --tmpdir) && echo x > $W/a.md", "放行"),
            ("-p 指到别的子 agent 认领的草稿目录，拒", identity, f"T=$(mktemp -d -p {other_directory}); cp -a records/a.md $T/", "拒"),
            ("--tmpdir= 指到别的会话的草稿目录，拒", identity, f"T=$(mktemp -d --tmpdir={foreign_scratch}); echo x > $T/a.md", "拒"),
            ("-p 指到 /tmp（不是草稿目录），拒", identity, "T=$(mktemp -d -p /tmp); echo x > $T/a.md", "拒"),
            ("-p 指到仓里的目录，拒", identity, "T=$(mktemp -d -p records); echo x > $T/a.md", "拒"),
            ("hook 输入里没有 agent_id，-p 指到草稿目录也拒", None, f"T=$(mktemp -d -p {own_directory}); echo x > $T/a.md", "拒"),
            ("套一层 cd 加 pwd -P：T=$(cd $(mktemp -d -p \"$W\") && pwd -P) 之后写进 $T/ 不记", identity,
             f"W={own_directory}; T=$(cd $(mktemp -d -p \"$W\") && pwd -P); echo x > $T/a.md", "放行"),
            ("套一层、不带 -p：T=$(cd $(mktemp -d) && pwd -P) 之后写进 $T/ 不记", identity, "T=$(cd $(mktemp -d) && pwd -P); echo x > $T/a.md", "放行"),
            ("套一层、反引号写法：T=`cd $(mktemp -d -p …) && pwd` 之后写进 $T/ 不记", identity,
             f"T=`cd $(mktemp -d -p {own_directory}) && pwd`; echo x > $T/a.md", "放行"),
            ("套一层、直接写进 $(cd $(mktemp -d -p …) && pwd -P)/文件 不记", identity,
             f"echo x > $(cd $(mktemp -d -p {own_directory}) && pwd -P)/cell.py", "放行"),
            ("套一层、-p 指到别的子 agent 认领的草稿目录，拒", identity,
             f"T=$(cd $(mktemp -d -p {other_directory}) && pwd -P); echo x > $T/a.md", "拒"),
            ("套一层、cd 之后接的不是 pwd，拒", identity, "T=$(cd $(mktemp -d) && echo /tmp); echo x > $T/a.md", "拒"),
        ]
        for label, case_identity, command, want in scratch_mktemp_cases:
            results.append((f"Bash写范围:{label}", want, scope_outcome(command, "clerk", case_identity)))
        before_scratch_noted = sum(1 for _ in open(detections)) if os.path.exists(detections) else 0
        scratch_payload = {"tool_name": "Bash", "session_id": "s-own", "agent_id": "agent-own", "agent_type": "clerk", "transcript_path": identity["transcript_path"],
                           "tool_input": {"command": f"T=$(mktemp -d {own_directory}/rs.XXXXXX); cp -a records/a.md $T/; echo x > $(mktemp -d {own_directory}/real.XXXXXX)/cell.py",
                                          "run_in_background": False}}
        scratch_entry = subprocess.run(["bash", os.path.join(scope_hooks, "bash-command-detector.sh")], capture_output=True, text=True,
                                       env=dict(os.environ, AGENT_HOOK_DETECTIONS=detections), input=json.dumps(scratch_payload))
        results.append(("stdin:模板在自己认领的草稿目录里的 mktemp -d 放行（退出码 0）、不记检出", (0, 0),
                        (scratch_entry.returncode, (sum(1 for _ in open(detections)) if os.path.exists(detections) else 0) - before_scratch_noted)))
    finally:
        for directory in (owned_scratch, foreign_scratch):
            if directory:
                shutil.rmtree(directory, ignore_errors=True)
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
        ("capped.sh 包着的 setsid -f", "bash scripts/hosts/capped.sh 4 setsid -f bash long.sh", 1),
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
        ("& 在 capped.sh 前缀的那条命令上", "bash scripts/hosts/capped.sh 4 nice -n 19 cargo test --release > test.log 2>&1 &", True, 1),
        ("& 在 capped.sh 包着的 bash -c 里", "bash scripts/hosts/capped.sh 4 bash -c 'cargo test --release > test.log 2>&1 &'", True, 1),
        ("& 在 wait 之后", "wait; bash late.sh &", True, 1),
        ("圆括号子 shell 里的 &、wait 在外面", "(bash long.sh > long.log 2>&1 &); wait", True, 1),
        ("& 之后的 wait 在另一对圆括号里", "bash long.sh & (wait)", True, 1),
        ("命令替换里的 &、wait 在外面", 'pid=$(bash long.sh > long.log 2>&1 & echo $!); wait "$pid"', True, 1),
        ("bash -c 里的 &、wait 在外面", "bash -c 'bash long.sh &'; wait", True, 1),
        ("喂给 bash 的 heredoc 正文里的 &", "bash <<'EOF'\nbash long.sh > long.log 2>&1 &\nEOF", True, 1),
        ("喂给 python 的 heredoc 挂在 & 前面", "python3 - > rerun.log 2>&1 <<'EOF' &\nimport subprocess\nEOF", True, 1),
        ("& 之后只 wait -n", "bash a.sh & bash b.sh & wait -n", True, 1),
        ("& 之后按 pid 轮询（tail --pid）算等到了（与检出 ② 同一套数法）", "bash long.sh > long.log 2>&1 & tail --pid=$! -f /dev/null", True, 0),
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
        ("几个 & 只 wait 一个 pid：剩下的照拒", "bash a.sh & bash b.sh & wait $!", True, 1),
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
        3000: {"parent": 1, "name": "python3", "command_line": "python3 -m selftest_model_server.entrypoints.api_server --model x"},
        4000: {"parent": 1, "name": "sleep", "command_line": "sleep 600"},
    }
    fake_reader = lambda: fake_table  # noqa: E731
    # 本地模型服务的单元与进程标记走自证自己的私有配置（不读本机那一份）；跑完这一段换回原来的环境
    private_config = os.path.join(work, "private.env")
    with open(private_config, "w", encoding="utf-8") as handle:
        handle.write("# 自证用的私有配置\nOTHER_KEY=selftest-other\nLOCAL_MODEL_SERVICE_UNITS=selftest-model-server, other-model.service\n"
                     "LOCAL_MODEL_SERVICE_PROCESS_MARKERS='selftest_model_server'\n")
    private_config_before = os.environ.get("BASH_COMMAND_DETECTOR_PRIVATE_CONFIG")
    os.environ["BASH_COMMAND_DETECTOR_PRIVATE_CONFIG"] = private_config
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
        ("systemctl stop 私有配置登记的本地模型服务（不写后缀）", "systemctl stop selftest-model-server", 1),
        ("systemctl restart 私有配置登记的第二个单元", "systemctl restart other-model.service", 1),
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
        ("写死的进程号是私有配置登记的本地模型服务", "kill 3000", 1),
        ("kill -HUP 1", "kill -HUP 1", 1),
        ("proc.py stop 写死的 sshd", "python3 .claude/singlefs-ai-sop/scripts/proc.py stop 700", 1),
        ("proc.py stop 一次两个", "python3 .claude/singlefs-ai-sop/scripts/proc.py stop 1100 4000", 1),
        ("循环里 proc.py stop", 'for p in 1100 4000; do python3 .claude/singlefs-ai-sop/scripts/proc.py stop "$p"; done', 1),
        ("bash -c 里 kill 进程组", "bash -c 'kill -TERM -- -$$'", 1),
        ("nice 包着 kill -9 -1", "nice -n 19 kill -9 -1", 1),
        ("喂给 bash 的 heredoc 里 kill 0", "bash <<'EOF'\nkill 0\nEOF", 1),
        ("timeout 包着 kill 负进程号", "timeout 5 kill -TERM -4000", 1),
        ("run-with-memory-cap.sh 包着 pkill", "bash scripts/hosts/run-with-memory-cap.sh 1G pkill cargo", 1),
        ("shell 函数与变量里的脚本包着的 bash -c 里 kill 0", 'in_context bash "$target" 64M bash -c \'kill 0\'', 1),
        ("shell 函数包着的 python -c 里 os.killpg", "in_context python3 -c 'import os; os.killpg(1234, 15)'", 1),
        # 自己用新会话起的子进程的进程组放行，这四格照拒（OWN_GROUP_TARGET 与 SESSION_CHILD_CALLS 两段注释）：
        # own-session-group-session-unchecked 下「不是 setsid 起的」「没开新会话的」两格红，own-session-group-first-binding-only 下「改写过」「改绑过」两格红
        ("不是 setsid 起的后台任务的进程组 -\"$leader\"", 'sleep 30 & leader=$!; kill -TERM -- -"$leader"', 1),
        ("setsid 起的后台任务的进程号又被改写过 -\"$leader\"", 'setsid --wait sleep 30 & leader=$!; leader="$other_group"; kill -TERM -- -"$leader"', 1),
        ("python -c 对没开新会话的子进程的进程组 os.killpg",
         "python3 -c 'import os, subprocess; child = subprocess.Popen([\"sleep\", \"30\"]); os.killpg(child.pid, 15)'", 1),
        ("python -c 子进程的名字又被改绑过的 os.killpg",
         "python3 -c 'import os, subprocess; child = subprocess.Popen([\"sleep\", \"30\"], start_new_session=True); child = other; os.killpg(child.pid, 15)'", 1),
        # 放行的两格在 own-session-group-unrecognized 下红
        ("setsid 起的后台任务的进程组 -\"$leader\"", 'setsid --wait sleep 30 & leader=$!; kill -TERM -- -"$leader"', 0),
        ("python -c 对 start_new_session=True 起的子进程的进程组 os.killpg 与 os.kill(-组号)",
         "python3 -c 'import os, subprocess; child = subprocess.Popen([\"sleep\", \"30\"], start_new_session=True); os.killpg(os.getpgid(child.pid), 15); os.kill(-child.pid, 9)'", 0),
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
        ("systemctl status sshd", "systemctl --user status sshd; systemctl status selftest-model-server", 0),
        ("systemctl stop 私有配置里只在别的键下出现的单元", "systemctl stop selftest-other", 0),
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
    # 进程表按需读：只有判到 kill / proc.py stop 的写死的进程号才读，读一次；BASH_COMMAND_DETECTOR_BREAK=always-read-process-table（命令里有数字就先读）下
    # 「没有终止词的带数字命令不读进程表」一格红
    table_reads = []
    def counting_reader():
        table_reads.append(1)
        return fake_table
    def reads_for(*commands):
        del table_reads[:]
        for one_command in commands:
            process_signal_refusal(one_command, counting_reader, 1001)
        return len(table_reads)
    results.append(("终止进程:没有终止词的带数字命令不读进程表（git log -1、sleep 2、--threads 8）", 0,
                    reads_for("git log -1", "sleep 2", "cargo test -p singlefs-core --threads 8")))
    results.append(("终止进程:kill 写死的进程号读一次进程表", 1, reads_for("kill 4000")))
    results.append(("终止进程:proc.py stop 写死的进程号读一次进程表", 1, reads_for("python3 .claude/singlefs-ai-sop/scripts/proc.py stop 4000")))
    results.append(("终止进程:一条命令里几处写死的进程号共读一次进程表", 1, reads_for("kill 4000; kill 1100")))
    results.append(("终止进程:kill 的目标不是写死的进程号不读进程表", 0, reads_for('kill "$pid"')))
    # 私有配置不在、没有那两个键：本地模型服务那一项不判、不报错，别的照判
    with open(os.path.join(work, "private-keyless.env"), "w", encoding="utf-8") as handle:
        handle.write("# 自证用的私有配置，没有那两个键\nOTHER_KEY=selftest-model-server selftest_model_server\n")
    for private_case, private_path in (("私有配置文件不在", os.path.join(work, "private-absent.env")),
                                       ("私有配置里没有那两个键", os.path.join(work, "private-keyless.env"))):
        os.environ["BASH_COMMAND_DETECTOR_PRIVATE_CONFIG"] = private_path
        for label, command, want in (("systemctl stop 没登记的单元放行", "systemctl stop selftest-model-server", 0),
                                     ("写死的进程号没登记标记放行", "kill 3000", 0),
                                     ("systemctl stop ssh 照拒", "systemctl stop ssh", 1)):
            results.append((f"终止进程:{private_case}:{label}", want, 1 if process_signal_refusal(command, fake_reader, 1001) else 0))
    if private_config_before is None:
        os.environ.pop("BASH_COMMAND_DETECTOR_PRIVATE_CONFIG", None)
    else:
        os.environ["BASH_COMMAND_DETECTOR_PRIVATE_CONFIG"] = private_config_before
    # 扫脚本（门禁 code-tooling 的 research-gate-lint 格那一路）：临时目录里造脚本，走 --scan-scripts 真实入口，看红绿与报的行号
    scan_root = os.path.join(work, "scan-root")
    scan_scripts_directory = os.path.join(scan_root, "scripts")
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
        return subprocess.run(["bash", os.path.join(hook_dir, "bash-command-detector.sh"), "--scan-scripts", root, os.path.join(root, "scripts")],
                              capture_output=True, text=True)
    scanned = scan(scan_root)
    results.append(("扫脚本:有红就退出码 1", 1, scanned.returncode))
    for label, marker in [("自己任务的进程组报在第 5 行", "scripts/group.sh:5"), ("循环里 kill 报在第 3 行", "scripts/loop.sh:3"),
                          ("标了 own-scope 而没核 scope", "scripts/storm-unguarded.sh:3"), ("进程组标 own-scope 不放行", "scripts/wrong-kind.sh:3"),
                          ("不起作用的 own-scope 标注", "scripts/stale-annotation.sh:3"), ("python 循环里 os.kill", "scripts/bad.py:3"),
                          ("python os.killpg", "scripts/bad.py:4")]:
        results.append((f"扫脚本:{label}", 1, int(f"✗ {marker} " in scanned.stdout)))
    results.append(("扫脚本:核过 scope 又标了的、点名一个的、探活的、python 探活的都不报", 0,
                    int(any(f"scripts/{name}" in scanned.stdout for name in ("storm-guarded.sh", "clean.sh", "good.py", "docs.sh", "docs.py")))))
    results.append(("扫脚本:拒绝带出路", 1, int("→ 怎么办：只停自己起的、点名的那一个" in scanned.stdout)))
    # 自己用新会话起的子进程的进程组（OWN_GROUP_TARGET 与 SESSION_CHILD_CALLS 两段注释）：另一个扫描根，上面那一组格数的脚本个数不动。
    # own-session-group-unrecognized 下「不报」那一格红；own-session-group-session-unchecked 下「没用 setsid」「setsid --fork」「没开新会话」三格红；
    # own-session-group-first-binding-only 下两格「改写过」「改绑过」红
    group_scan_root = os.path.join(work, "scan-group-root")
    os.makedirs(os.path.join(group_scan_root, "scripts"))
    os.makedirs(os.path.join(group_scan_root, ".claude"))
    group_scan_files = {
        "own-group.sh": ('#!/usr/bin/env bash\nsetsid --wait sleep 30 &\nleader=$!\nkill -TERM -- -"$leader"\nsleep 1\nkill -KILL -- -"${leader}"\n'),
        "own-group.py": ("import asyncio\nimport os\nimport signal\nimport subprocess\n\n\n"
                         "def start_worker(command):\n    return subprocess.Popen(command, start_new_session=True)\n\n\n"
                         "def stop_direct(command):\n    process = subprocess.Popen(command, start_new_session=True)\n"
                         "    os.killpg(process.pid, signal.SIGTERM)\n    os.killpg(os.getpgid(process.pid), signal.SIGKILL)\n\n\n"
                         "def stop_through_helper(command):\n    worker = start_worker(command)\n    group = os.getpgid(worker.pid)\n"
                         "    for attempt in range(3):\n        os.killpg(group, signal.SIGTERM)\n    os.kill(-worker.pid, signal.SIGKILL)\n\n\n"
                         "def stop_in_context(command):\n    with subprocess.Popen(command, process_group=0) as runner:\n"
                         "        def stop_later():\n            os.killpg(runner.pid, signal.SIGTERM)\n        stop_later()\n\n\n"
                         "async def stop_asynchronous(command):\n    child = await asyncio.create_subprocess_exec(*command, start_new_session=True)\n"
                         "    os.killpg(child.pid, signal.SIGTERM)\n"),
        "other-group.sh": ('#!/usr/bin/env bash\nsleep 30 &\nleader=$!\nkill -TERM -- -"$leader"\n'
                           'setsid --wait sleep 30 &\nleader=$!\nleader="$other_group"\nkill -INT -- -"$leader"\n'
                           'setsid --fork sleep 30 &\nforked=$!\nkill -HUP -- -"$forked"\nkill -TERM -- -1234\n'),
        "other-group.py": ("import os\nimport signal\nimport subprocess\n\n\n"
                           "def stop_given(process):\n    os.killpg(process.pid, signal.SIGTERM)\n\n\n"
                           "def stop_plain(command):\n    process = subprocess.Popen(command)\n    os.killpg(process.pid, signal.SIGTERM)\n\n\n"
                           "def stop_rebound(command, other):\n    process = subprocess.Popen(command, start_new_session=True)\n    process = other\n"
                           "    os.killpg(process.pid, signal.SIGTERM)\n\n\n"
                           "def stop_own_group():\n    os.killpg(os.getpgid(0), signal.SIGTERM)\n\n\n"
                           "def stop_picked():\n    group = int(open(\"/sys/fs/cgroup/x/cgroup.procs\").readline())\n    os.killpg(group, signal.SIGTERM)\n\n\n"
                           "class Runner:\n    def stop(self):\n        os.killpg(self.process.pid, signal.SIGTERM)\n\n\n"
                           "os.kill(-1, signal.SIGTERM)\n"),
    }
    for file_name, content in group_scan_files.items():
        with open(os.path.join(group_scan_root, "scripts", file_name), "w", encoding="utf-8") as handle:
            handle.write(content)
    group_scanned = scan(group_scan_root)
    results.append(("扫脚本:进程组:有红就退出码 1", 1, group_scanned.returncode))
    results.append(("扫脚本:进程组:自己用 setsid 或 start_new_session=True 起的子进程的组不报", 0,
                    int(any(f"scripts/{name}" in group_scanned.stdout for name in ("own-group.sh", "own-group.py")))))
    for label, marker in [("没用 setsid 起的后台任务的组", "scripts/other-group.sh:4"), ("setsid 起的进程号又被改写过", "scripts/other-group.sh:8"),
                          ("setsid --fork 起的", "scripts/other-group.sh:11"), ("负的写死号", "scripts/other-group.sh:12"),
                          ("python 参数传进来的子进程", "scripts/other-group.py:7"), ("python 没开新会话的子进程", "scripts/other-group.py:12"),
                          ("python 子进程的名字又被改绑过", "scripts/other-group.py:18"), ("python os.getpgid(0)", "scripts/other-group.py:22"),
                          ("python 从 cgroup.procs 读来的组", "scripts/other-group.py:27"), ("python 属性上的子进程 self.process", "scripts/other-group.py:32"),
                          ("python os.kill(-1, …)", "scripts/other-group.py:35")]:
        results.append((f"扫脚本:进程组:{label}", 1, int(f"✗ {marker} " in group_scanned.stdout)))
    # 弄坏开关只在自证里认：平常跑（不带标记）设了就退 2 并点名变量；自证起的子进程（带标记）照认。break-honored-outside-selftest 设着时探针只写它一项，第一格红
    unmarked_environment = {key: value for key, value in os.environ.items() if key != SELFTEST_MARK_VARIABLE}
    probe_items = GUARD_OFF_ITEM if detector_break_is_set(GUARD_OFF_ITEM) else "disable-check"   # 关守卫项只在单独一项时认：开关设着时探针只写它一项
    plain_run = subprocess.run(["bash", os.path.join(hook_dir, "bash-command-detector.sh"), "--force", "--break-guard-probe-unknown-option"], input="", capture_output=True, text=True,
                               env=dict(unmarked_environment, **{DETECTOR_BREAK_VARIABLE: probe_items, "AGENT_HOOK_DETECTIONS": os.path.join(work, "probe-detections.jsonl")}))
    results.append(("平常跑设了弄坏开关：退 2 并点名只在自证里认", (2, True), (plain_run.returncode, DETECTOR_BREAK_VARIABLE in plain_run.stderr and "自证" in plain_run.stderr)))
    marked_run = subprocess.run(["bash", os.path.join(hook_dir, "bash-command-detector.sh")], capture_output=True, text=True,
                                input=json.dumps({"tool_name": "Bash", "tool_input": {"command": 'until grep -q "^exit=" log; do sleep 10; done', "run_in_background": False}}),
                                env=dict(os.environ, **{DETECTOR_BREAK_VARIABLE: "disable-check", "AGENT_HOOK_DETECTIONS": os.path.join(work, "probe-detections.jsonl")}))
    results.append(("自证起的子进程（带标记）照认弄坏开关", 0, marked_run.returncode))
    # 关守卫项与别的项同写：平常跑与带标记的子进程都退 2、点名它只在单独一项时认；开关 break-honored-outside-selftest-with-others 设着时带标记那一格的开关里也带上它（同写照认），那一格红
    combined_items = f"{GUARD_OFF_ITEM},disable-check"
    combined_plain = subprocess.run(["bash", os.path.join(hook_dir, "bash-command-detector.sh"), "--force", "--break-guard-probe-unknown-option"], input="", capture_output=True, text=True,
                                    env=dict(unmarked_environment, **{DETECTOR_BREAK_VARIABLE: combined_items, "AGENT_HOOK_DETECTIONS": os.path.join(work, "probe-detections.jsonl")}))
    results.append(("关守卫项与别的项同写：平常跑退 2 并点名它只在单独一项时认", (2, True),
                    (combined_plain.returncode, GUARD_OFF_ITEM in combined_plain.stderr and "唯一一项" in combined_plain.stderr)))
    lenient_items = combined_items + (f",{GUARD_OFF_WITH_OTHERS_ITEM}" if detector_break_is_set(GUARD_OFF_WITH_OTHERS_ITEM) else "")
    combined_marked = subprocess.run(["bash", os.path.join(hook_dir, "bash-command-detector.sh"), "--force", "--break-guard-probe-unknown-option"], input="", capture_output=True, text=True,
                                     env=dict(os.environ, **{DETECTOR_BREAK_VARIABLE: lenient_items, "AGENT_HOOK_DETECTIONS": os.path.join(work, "probe-detections.jsonl")}))
    results.append(("关守卫项与别的项同写：带标记的子进程里也退 2 并点名它", (2, True),
                    (combined_marked.returncode, GUARD_OFF_ITEM in combined_marked.stderr and "唯一一项" in combined_marked.stderr)))
    pending_path = os.path.join(scan_root, ".claude", "process-safety-pending")
    for file_name in ("storm-unguarded.sh", "wrong-kind.sh", "stale-annotation.sh"):
        os.remove(os.path.join(scan_scripts_directory, file_name))
    with open(pending_path, "w", encoding="utf-8") as handle:
        handle.write("# 待改\nscripts/group.sh  # 自检：别人正在改\nscripts/loop.sh  # 自检：别人正在改\nscripts/bad.py  # 自检：别人正在改\n")
    pending_green = scan(scan_root)
    results.append(("扫脚本:红的都登记进待改清单就退出码 0", 0, pending_green.returncode))
    results.append(("扫脚本:成功行报查了几个、放行了哪一处、逐个列出没判的", 1,
                    int("查了 8 个脚本（.sh 5 个、.py 3 个）" in pending_green.stdout and "scripts/storm-guarded.sh:4" in pending_green.stdout
                        and "scripts/group.sh（scripts/group.sh:5）" in pending_green.stdout)))
    with open(pending_path, "a", encoding="utf-8") as handle:
        handle.write("scripts/clean.sh  # 自检：干净的也登记了\nscripts/gone.sh  # 自检：指向不存在的文件\n")
    pending_stale = scan(scan_root)
    results.append(("扫脚本:待改清单里一处都没排到的与指向不存在的判红", 1,
                    int(pending_stale.returncode == 1 and "clean.sh 里一处要放行的都没有" in pending_stale.stdout and "gone.sh 指向不存在的文件" in pending_stale.stdout)))
    # 待改清单里的行所在的目录这一趟没扫（拆分提案第九十节 ㉝ d）：只扫 scripts/ 这一层，清单另有一行落在 scripts/deeper/ 下；没扫就不知道它还有没有要放行的，判本次未判
    os.makedirs(os.path.join(scan_scripts_directory, "deeper"))
    with open(os.path.join(scan_scripts_directory, "deeper", "killer.sh"), "w", encoding="utf-8") as handle:
        handle.write('#!/usr/bin/env bash\nfor pid in "${worker_pids[@]}"; do\n  kill -TERM "$pid"\ndone\n')
    with open(pending_path, "w", encoding="utf-8") as handle:
        handle.write("# 待改\nscripts/group.sh  # 自检：别人正在改\nscripts/loop.sh  # 自检：别人正在改\nscripts/bad.py  # 自检：别人正在改\n"
                     "scripts/deeper/killer.sh  # 自检：别人正在改，这一趟没扫它所在的目录\n")
    not_run_marker = os.path.join(work, "not-run-lines")
    open(not_run_marker, "w").close()
    def scan_directories(directories, extra_environment=None):
        return subprocess.run(["bash", os.path.join(hook_dir, "bash-command-detector.sh"), "--scan-scripts", scan_root, *directories],
                              capture_output=True, text=True, env=dict(os.environ, **(extra_environment or {})))
    outside_scanned = scan_directories([scan_scripts_directory], {"GATE_NOT_RUN_FILE": not_run_marker})
    results.append(("扫脚本:待改清单里的行所在的目录这一趟没扫：退出码 0、报本次未判", (0, True, False),
                    (outside_scanned.returncode,
                     "本次未判" in outside_scanned.stdout and "scripts/deeper/killer.sh" in outside_scanned.stdout,
                     "scripts/deeper/killer.sh 里一处要放行的都没有" in outside_scanned.stdout)))
    results.append(("扫脚本:待改清单里的行所在的目录这一趟没扫：往 GATE_NOT_RUN_FILE 报一行", 1,
                    sum(1 for line in open(not_run_marker, encoding="utf-8") if "scripts/deeper/killer.sh" in line and "本次未判" in line)))
    both_scanned = scan_directories([scan_scripts_directory, os.path.join(scan_scripts_directory, "deeper")])
    results.append(("扫脚本:待改清单里的行所在的目录这一趟也扫了：照常列进没判的、不报本次未判", (0, True, False),
                    (both_scanned.returncode, "scripts/deeper/killer.sh（scripts/deeper/killer.sh:3）" in both_scanned.stdout, "本次未判" in both_scanned.stdout)))
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
        ("timeout、capped.sh 包着的 bash -c 整条重定向", "timeout 60 bash scripts/hosts/capped.sh 4 bash -c 'cargo run' > research/results/untracked.out", 1),
        ("前面赋过值的变量代入", 'out=research/results/untracked.out; python3 x.py > "$out"', 1),
        ("export 过的变量代入", 'export OUT=research/results/untracked.out; python3 x.py > "${OUT}"', 1),
        ("cd 到变量路径算不出来、按仓库根解析", 'cd "$scratch" && python3 x.py > research/results/untracked.out', 1),
        ("cd $HOME/… 算得出来、按那个目录解析（仓外，放行）", 'cd "$HOME/no-such-directory-for-detector-selftest" && python3 x.py > research/results/untracked.out', 0),
        ("进程替换里的 tee", "make > >(tee research/results/untracked.out)", 1),
        ("整个循环重定向", "for arm in a b; do python3 x.py $arm; done > research/results/untracked.out", 1),
        ("sudo 包着的 tee", "echo x | sudo tee research/results/untracked.out", 1),
        ("git add 的是另一份，这一份照拒", "git add research/results/tracked.out && python3 x.py > research/results/untracked.out", 1),
        ("dd of= 覆盖未跟踪的产物", "dd if=/tmp/new.out of=research/results/untracked.out", 1),
        ("dd conv=notrunc 覆盖未跟踪的产物", "dd if=/tmp/new.out of=research/results/untracked.out conv=notrunc", 1),
        ("truncate -s 0 覆盖未跟踪的产物", "truncate -s 0 research/results/untracked.out", 1),
        ("truncate 选项连写覆盖未跟踪产物", "truncate -cs 0 research/results/sub/deep.out", 1),
        ("dd of= 写已跟踪的文件（git 兜得住）", "dd if=/tmp/new.out of=research/results/tracked.out", 0),
        ("truncate 写已跟踪的文件（git 兜得住）", "truncate -s 0 research/results/tracked.out", 0),
        ("dd of= 写新文件名", "dd if=/tmp/new.out of=research/results/untracked-2026-09-25.out", 0),
        ("dd of= 目标在 research/results/ 外", "dd if=/tmp/new.out of=scratch.log", 0),
        ("同一条命令里先 git add 再 dd 覆盖", "git add research/results/untracked.out && dd if=/tmp/new.out of=research/results/untracked.out", 0),
        ("引号里当数据写的 dd", "echo 'dd if=/tmp/new.out of=research/results/untracked.out'", 0),
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
    # 一条命令里几个确定的目标同时判：各自点名，且 git ls-files 只起一次（BASH_COMMAND_DETECTOR_BREAK=per-path-git-query 一份一份查，后两格的次数红）
    def refused_and_runs(command):
        GIT_LS_FILES_RUNS[0] = 0
        refused_here = results_overwrite_refusal(command, repository)[0] if git_ready else []
        return refused_here, GIT_LS_FILES_RUNS[0]
    two_refused, two_runs = refused_and_runs("make | tee research/results/untracked.out research/results/sub/deep.out")
    mixed_refused, mixed_runs = refused_and_runs("make | tee research/results/tracked.out research/results/untracked.out research/results/sub/deep.out research/results/fresh.out")
    results.append(("覆盖产物:一条命令里两份未跟踪的目标各自点名", ["tee research/results/untracked.out", "tee research/results/sub/deep.out"], two_refused))
    results.append(("覆盖产物:一条命令里两份未跟踪的目标同时判，git ls-files 只起一次", 1, two_runs))
    results.append(("覆盖产物:已跟踪的、未跟踪的、不存在的目标混在一起同时判，只点名未跟踪的", ["tee research/results/untracked.out", "tee research/results/sub/deep.out"], mixed_refused))
    results.append(("覆盖产物:已跟踪的、未跟踪的、不存在的目标混在一起，git ls-files 只起一次", 1, mixed_runs))
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
    os.makedirs(os.path.join(repository, "scripts"))
    os.makedirs(os.path.join(repository, "bin"))
    os.makedirs(os.path.join(scratch_root, "probe"))
    for relative, mode in (("scripts/running.sh", 0o755), ("scripts/tool.py", 0o644), ("bin/runner", 0o755),
                           ("notes.md", 0o644), ("data.txt", 0o644)):
        with open(os.path.join(repository, relative), "w", encoding="utf-8") as handle:
            handle.write("echo 旧内容\n")
        os.chmod(os.path.join(repository, relative), mode)
    os.symlink("running.sh", os.path.join(repository, "scripts", "link.sh"))
    for path in (os.path.join(scratch_root, "probe", "step.sh"), os.path.join(work, "outside.sh")):
        with open(path, "w", encoding="utf-8") as handle:
            handle.write("echo 旧内容\n")
        os.chmod(path, 0o755)
    running = "scripts/running.sh"
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
        ("cp 进已有目录、文件名撞上", f"cp {sources}/running.sh scripts/", 1),
        ("cp -t 目录", f"cp -t scripts {sources}/running.sh", 1),
        ("dd of=", f"dd if=/tmp/new.sh of={running}", 1),
        ("dd conv=notrunc", f"dd if=/tmp/new.sh of={running} conv=notrunc", 1),
        ("truncate -s 0", f"truncate -s 0 {running}", 1),
        ("truncate -cs 0（选项连写）", "truncate -cs 0 scripts/tool.py", 1),
        ("写已有的 .py", "echo x > scripts/tool.py", 1),
        ("带执行位、没有后缀的", "cp /tmp/new bin/runner", 1),
        ("经符号链接写它指向的脚本", "echo x > scripts/link.sh", 1),
        ("scratch 根下的脚本", f"cp /tmp/new.sh {scratch_root}/probe/step.sh", 1),
        ("cd 进去之后写相对路径", "cd scripts && echo x > running.sh", 1),
        ("bash -c 里的 >", f"bash -c 'echo x > {running}'", 1),
        ("喂给 bash 的 heredoc 正文里的 cp", f"bash <<'EOF'\ncp /tmp/new.sh {running}\nEOF", 1),
        ("命令替换里的 >", f"lines=$(echo x > {running}; echo done)", 1),
        ("前面赋过值的变量代入", f'target={running}; cp /tmp/new.sh "$target"', 1),
        ("nice、timeout 包着的 cp", f"timeout 60 nice -n 19 cp /tmp/new.sh {running}", 1),
        ("python3 -c 的 open(…, 'w')", f"python3 -c \"open('{running}', 'w').write('x')\"", 1),
        ("python3 -c 的 open(…, 'r+')", f"python3 -c \"open('{running}', 'r+').write('x')\"", 1),
        ("python3 -c 的 open(…, mode='wb')", "python3 -c \"open('scripts/tool.py', mode='wb').write(b'x')\"", 1),
        ("喂给 python 的 heredoc：只赋过一次的名字", f"python3 - <<'EOF'\npath = '{running}'\nwith open(path, 'w') as handle:\n    handle.write('x')\nEOF", 1),
        ("喂给 python 的 heredoc：Path(…) / '…' 的 write_text", "python3 - <<'EOF'\nfrom pathlib import Path\n(Path('scripts') / 'tool.py').write_text('x')\nEOF", 1),
        ("喂给 python 的 heredoc：pathlib.Path(…).write_bytes", f"python3 <<'EOF'\nimport pathlib\npathlib.Path('{running}').write_bytes(b'x')\nEOF", 1),
        ("喂给 python 的 heredoc：Path(…).open('w')", f"python3 - <<'EOF'\nimport pathlib\nwith pathlib.Path('{running}').open('w') as handle:\n    handle.write('x')\nEOF", 1),
        ("喂给 python 的 heredoc：os.path.join 与 io.open", "python3 - <<'EOF'\nimport io, os\nio.open(os.path.join('scripts', 'tool.py'), 'w')\nEOF", 1),
        ("喂给 python 的 heredoc：shutil.copyfile", f"python3 - <<'EOF'\nimport shutil\nshutil.copyfile('/tmp/new.sh', '{running}')\nEOF", 1),
        ("喂给 python 的 heredoc：shutil.copy 进已有目录", "python3 - <<'EOF'\nimport shutil\nshutil.copy('/tmp/running.sh', 'scripts')\nEOF", 1),
        ("cd 之后喂给 python 的相对路径", "cd scripts && python3 - <<'EOF'\nopen('running.sh', 'w').write('x')\nEOF", 1),
        ("bash -c 里的 python3 -c", f"bash -c \"python3 -c \\\"open('{running}', 'w')\\\"\"", 1),
        (">> 追加", f"echo x >> {running}", 0),
        ("&>> 追加", f"make &>> {running}", 0),
        ("tee -a", f"echo x | tee -a {running}", 0),
        ("python 的 'a' 追加", f"python3 -c \"open('{running}', 'a').write('x')\"", 0),
        ("python 的 'x' 只建新文件", f"python3 -c \"open('{running}', 'x')\"", 0),
        ("python 读", f"python3 -c \"print(open('{running}').read())\"", 0),
        ("写新文件名", "echo x > scripts/new-step.sh", 0),
        ("写到同目录临时文件再 mv 换上", f"cat > scripts/.running.sh.new <<'EOF'\necho 新内容\nEOF\nchmod 755 scripts/.running.sh.new && mv scripts/.running.sh.new {running}", 0),
        ("cp 成隐藏文件、chmod、mv 换上", f"cp /tmp/new.sh scripts/.running.sh.installing && chmod 755 scripts/.running.sh.installing && mv scripts/.running.sh.installing {running}", 0),
        ("mv 新 旧", f"mv /tmp/new.sh {running}", 0),
        ("install 换 inode", f"install -m 755 /tmp/new.sh {running}", 0),
        ("cp --remove-destination 换 inode", f"cp --remove-destination /tmp/new.sh {running}", 0),
        ("cp -l -f 换成硬链接", f"cp -l -f /tmp/new.sh {running}", 0),
        ("cp -n 不盖已有的", f"cp -n /tmp/new.sh {running}", 0),
        ("cp -b 旧的留成备份", f"cp -b /tmp/new.sh {running}", 0),
        ("sed -i 换 inode", f"sed -i 's/旧/新/' {running}", 0),
        ("同一条命令里先 rm 再写", f"rm {running} && echo x > {running}", 0),
        ("同一条命令里先挪走再写", f"mv {running} {running}.old && echo x > {running}", 0),
        ("python 写到临时文件再 os.replace", f"python3 - <<'EOF'\nimport os\nopen('scripts/.running.sh.new', 'w').write('x')\nos.replace('scripts/.running.sh.new', '{running}')\nEOF", 0),
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
        ("一百三十七：rsync --inplace 盖已有脚本（正推腿乙 Z8；rsync-inplace-unseen 下红）", f"rsync --inplace /tmp/new.sh {running}", 1),
        ("一百三十七：rsync 不带 --inplace 换 inode", f"rsync /tmp/new.sh {running}", 0),
    ]
    for label, command, want in script_in_place_cases:
        results.append((f"就地改脚本:{label}", want, 1 if script_in_place_refusal(command, repository, scratch_root) else 0))
    results.append(("就地改脚本:拒绝时报出写法与相对仓库根的路径", 1,
                    int(script_in_place_refusal(f"cp /tmp/new.sh {running}", repository, scratch_root) == [f"cp {running}"])))
    # 不给 scratch_root 时认的是工作根与会话临时目录根（SCRIPT_SCRATCH_ROOTS）：几处各放一份已有的脚本，自己建、自己删；
    # 会话 scratchpad 那一份照 Claude Code 的布局 <会话临时目录根>/<项目目录名>/<会话号>/scratchpad/ 放（项目目录名那一层是自己建的临时目录）
    session_temporary_root = work_roots.claude_session_temporary_root() if work_roots else None
    for label, root, inner in (("工作根下的脚本", work_roots.configured_work_root() if work_roots else None, ""),
                               ("会话临时目录根下的脚本", session_temporary_root, ""),
                               ("会话 scratchpad 里的脚本", session_temporary_root, os.path.join("selftest-session", "scratchpad"))):
        refused = 0
        if root is not None:
            os.makedirs(root, exist_ok=True)
            probe_directory = tempfile.mkdtemp(prefix="-home-bash-command-detector-selftest-", dir=root)
            try:
                os.makedirs(os.path.join(probe_directory, inner), exist_ok=True)
                probe = os.path.join(probe_directory, inner, "step.sh")
                with open(probe, "w", encoding="utf-8") as handle:
                    handle.write("echo 旧内容\n")
                refused = int(bool(script_in_place_refusal(f"cp /tmp/new.sh {probe}", repository)))
            finally:
                shutil.rmtree(probe_directory, ignore_errors=True)
        results.append((f"就地改脚本:{label}", 1, refused))
    results.append(("就地改脚本:⑤ 现在认 dd、truncate 覆盖未跟踪产物（与 ⑦ 共用同一段 overwrite_steps，不抄第二份）", 1,
                    int(bool(results_overwrite_refusal("dd if=x of=research/results/untracked.out; truncate -s 0 research/results/untracked.out",
                                                       repository)[0])) if git_ready else 1))
    results.append(("就地改脚本:⑤ 仍不认 python 的写法（覆盖未跟踪产物；rm 另见下一条，那条测的是 PreservingStep 不是 form）", 0,
                    int(bool(results_overwrite_refusal("python3 -c \"open('research/results/untracked.out', 'w')\"", repository)[0])) if git_ready else 0))
    results.append(("就地改脚本:rm 掉再写未跟踪产物，⑤ 照拒（rm 没留住旧字节）", 1,
                    int(bool(results_overwrite_refusal("rm research/results/untracked.out && echo x > research/results/untracked.out", repository)[0])) if git_ready else 1))
    # 走真实入口：把 hook 与共用切词模块拷进临时仓的 .claude/hooks/，仓库根取它往上两级
    repository_hooks = os.path.join(repository, ".claude", "hooks")
    os.makedirs(repository_hooks)
    shutil.copy(os.path.join(hook_dir, "bash-command-detector.sh"), repository_hooks)
    shutil.copy(shell_words_library_path(hook_dir), repository_hooks)
    shutil.copy(os.path.join(hook_dir, "lib_project_code_runs.py"), repository_hooks)   # 剥 /usr/bin/time、flock 这类启动器用它（command_under_launcher）
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
                        and "写到同目录临时文件再 `mv` 换上" in in_place.stderr and "scripts/kb/replace-once.py" in in_place.stderr)))
    results.append(("stdin:就地改脚本拒了的不记检出", 0, in_place_recorded))
    python_in_place = through_entry(f"python3 - <<'EOF'\nopen('{running}', 'w').write('x')\nEOF", True)[0]
    results.append(("stdin:run_in_background 里喂给 python 的 open(…, 'w') 同样拒绝（退出码 2）", 2, python_in_place.returncode))
    # ⑪ rm -r 通删工作根：(说明, 命令, 该不该拒)；样本路径照配置的工作根造（不看弄坏开关），会话临时目录根是 /tmp/claude-<uid>
    removal_root = work_root_removal_roots()[0]
    removal_home = os.path.expanduser("~")
    removal_tilde = "~" + removal_root[len(removal_home):] if removal_root.startswith(removal_home + "/") else removal_root
    removal_dollar = "$HOME" + removal_root[len(removal_home):] if removal_root.startswith(removal_home + "/") else removal_root
    removal_session_temporary_root = f"/tmp/claude-{os.getuid()}"
    removal_cases = [
        ("工作根下通配（~ 写法）", f"rm -rf {removal_tilde}/*", 1),
        ("工作根下通配（$HOME 写法、引号里外）", f'rm -rf "{removal_dollar}/"*', 1),
        ("工作根本身", f"rm -rf {removal_root}", 1),
        ("工作根本身带结尾的 /", f"rm -rf {removal_tilde}/", 1),
        ("会话临时目录根下通配", f"rm -rf {removal_session_temporary_root}/*", 1),
        ("rm -fr", f"rm -fr {removal_root}/*", 1),
        ("rm -r -f 分开写", f"rm -r -f {removal_root}/*", 1),
        ("rm --recursive --force", f"rm --recursive --force {removal_root}/*", 1),
        ("rm -r 不带 -f", f"rm -r {removal_root}/*", 1),
        ("工作根下第一层部分通配", f"rm -rf {removal_root}/owed-*", 1),
        ("工作根下第一层通配再往下", f"rm -rf {removal_root}/*/target", 1),
        ("cd 进工作根之后 rm -rf *", f"cd {removal_tilde} && rm -rf *", 1),
        # cd 的目录写在变量里：同一段命令里前面赋过的变量与 $HOME 展开之后再判（共用模块 lib_shell_words.py 的 expand_known_variables）
        ("D=… 之后 cd $D 进工作根再 rm -rf *", f"D={removal_root}; cd $D && rm -rf *", 1),
        ("cd $HOME/… 进工作根再 rm -r 第一层部分通配", f"cd {removal_dollar} && rm -r owed-*", 1),
        ("cd 进写在变量里的工作根下具体目录再 rm -rf * 放行", f"D={removal_root}/owed-x; cd $D && rm -rf *", 0),
        ("bash -c 里", f"bash -c 'rm -rf {removal_root}/*'", 1),
        ("nice、sudo 前缀", f"nice -n 19 sudo rm -rf {removal_root}/*", 1),
        ("工作根的上级", f"rm -rf {os.path.dirname(removal_root)}", 1),
        ("$SINGLEFS_WORK_ROOT 写法", 'rm -rf "$SINGLEFS_WORK_ROOT"/*', 1),
        ("/tmp/* 盖住会话临时目录根", "rm -rf /tmp/*", 1),
        # 探针串拆成两段相邻字面量拼起来：探针要的就是「变量路径不带空值守卫」这个写法，整串写在源码里会被 shell 纪律检查当成本脚本自己的 rm -rf
        ("前面 export 过的变量代入", f'export W={removal_root}; rm -rf "$' 'W"/*', 1),
        ("选项写在操作数后面", f"rm {removal_root}/* -rf", 1),
        ("喂给 bash 的 heredoc 里", f"bash <<'EOF'\nrm -rf {removal_root}/*\nEOF", 1),
        ("工作根下一个具体目录放行", f"rm -rf {removal_tilde}/owed-c792-work-ownership", 0),
        ("具体目录里面的通配放行", f"rm -rf {removal_root}/owed-x/copy-*", 0),
        ("别处的 rm -rf 通配放行", "rm -rf /tmp/singlefs-mutation-x/*", 0),
        ("仓里的相对路径放行", "rm -rf target/debug/incremental", 0),
        ("不带 -r 的 rm 放行", f"rm -f {removal_root}/*.tmp", 0),
        ("echo 引号里当数据写的放行", f"echo 'rm -rf {removal_root}/*'", 0),
        ("grep 参数里的放行", "grep -n 'rm -rf' .claude/hooks/bash-command-detector.sh", 0),
        ("算不出的变量放行", 'rm -rf "$d"/*', 0),
        ("同名前缀的兄弟目录放行", f"rm -rf {removal_root}-other/*", 0),
        ("会话临时目录根下一个具体目录放行", f"rm -rf {removal_session_temporary_root}/owed-x", 0),
        ("写进文件的 heredoc 正文放行", f"cat > note.md <<'EOF'\nrm -rf {removal_root}/*\nEOF", 0),
        ("家目录通配不碰点目录放行", "rm -rf ~/*", 0),
    ]
    for label, command, want in removal_cases:
        results.append((f"通删工作根:{label}", want, 1 if work_root_removal_refusal(command, work) else 0))
    removal_scene = through_entry(f"rm -rf {removal_root}/*")[0]
    results.append(("stdin:rm -rf 工作根下通配拒绝（退出码 2）", 2, removal_scene.returncode))
    results.append(("stdin:通删工作根的拒绝写了出路（work-directory-ownership.sh 的 sweep 与 claim）", 1,
                    int("✗" in removal_scene.stderr and "→" in removal_scene.stderr
                        and f"bash {WORK_ROOT_SWEEP_SCRIPT} sweep --dry-run" in removal_scene.stderr
                        and f"bash {WORK_ROOT_SWEEP_SCRIPT} claim <目录> --purpose <一句>" in removal_scene.stderr)))
    results.append(("stdin:rm -rf 工作根下一个具体目录放行（退出码 0）", 0, through_entry(f"rm -rf {removal_root}/owed-x")[0].returncode))
    # 走真实入口：experiment-runner 在 Bash 里 cp 进主工作区 crates/ 下的 .rs 拒绝（退出码 2）、stderr 点名先编后换那条脚本；implementation-writer 放行
    def entry_as(agent_type, command):
        return subprocess.run(["bash", os.path.join(repository_hooks, "bash-command-detector.sh")], capture_output=True, text=True,
                              env=dict(os.environ, AGENT_HOOK_DETECTIONS=detections),
                              input=json.dumps({"tool_name": "Bash", "session_id": "s", "agent_type": agent_type,
                                                "tool_input": {"command": command, "run_in_background": False}}))
    def entry_in_background_as(agent_type, command, working_directory=None):
        hook_payload = {"tool_name": "Bash", "session_id": "s", "tool_input": {"command": command, "run_in_background": True}}
        if agent_type:
            hook_payload["agent_type"] = agent_type
        if working_directory:   # 真实输入带 cwd：客户端 task-server/task 写的是相对路径，按 cwd 解；不带 cwd 就解到临时仓，那里没有客户端
            hook_payload["cwd"] = working_directory
        return subprocess.run(["bash", os.path.join(repository_hooks, "bash-command-detector.sh")], capture_output=True, text=True,
                              env=dict(os.environ, AGENT_HOOK_DETECTIONS=detections), input=json.dumps(hook_payload))
    subagent_background = entry_in_background_as("implementation-writer", "cargo build -p singlefs-core")
    results.append(("stdin:子 agent 用 run_in_background 起命令拒绝（退出码 2）", 2, subagent_background.returncode))
    results.append(("stdin:子 agent 的 run_in_background 拒绝时 stderr 写了不起、写进报告「要主 agent 跑的命令」一节、结束本轮交回的出路，不教后台等", 1,
                    int("✗" in subagent_background.stderr and "→" in subagent_background.stderr
                        and "写进报告「要主 agent 跑的命令」一节交回" in subagent_background.stderr and "结束本轮交回主 agent" in subagent_background.stderr
                        and "task-server/task wait <号>" not in subagent_background.stderr)))
    # ⑩ 子 agent 的 run_in_background 一律拒，task-server/task wait 也拒（allow-subagent-background、disable-check 下这几格红）；
    # cwd 给本仓根（临时仓里没有 task-server/task），客户端在盘上也照拒
    task_repository = repository_root_of(hook_dir)
    for label, command, want in (
            ("stdin:子 agent 后台起 task-server/task wait 7 照拒（退出码 2）", "task-server/task wait 7", 2),
            ("stdin:子 agent 后台起 cd <仓根> && task-server/task wait 7 照拒（退出码 2）", f"cd {task_repository} && task-server/task wait 7", 2),
            ("stdin:子 agent 后台起 task-server/task wait 7 && echo x 照拒（退出码 2）", "task-server/task wait 7 && echo x", 2),
            ("stdin:子 agent 后台起 sleep 5 照拒（退出码 2）", "sleep 5", 2),
            ("stdin:子 agent 后台起 cargo build 照拒（退出码 2）", "cargo build", 2),
            ("stdin:子 agent 后台起 task-server/task wait 后面不是号照拒（退出码 2）", "task-server/task wait x", 2),
            ("stdin:子 agent 后台起 task-server/task status 照拒（退出码 2）", "task-server/task status", 2)):
        results.append((label, want, entry_in_background_as("implementation-writer", command, task_repository).returncode))
    results.append(("stdin:主 agent 用 run_in_background 起同一条命令放行（退出码 0）", 0, entry_in_background_as(None, "cargo build -p singlefs-core").returncode))
    results.append(("stdin:子 agent 前台起同一条命令放行（退出码 0）", 0, entry_as("implementation-writer", "cargo build -p singlefs-core").returncode))
    runner_copy = entry_as("experiment-runner", "cp ~/.local/share/singlefs/work/runner-x/e161_x.rs crates/singlefs-verification/src/bin/e161_x.rs")
    results.append(("stdin:experiment-runner cp 进主工作区的入库装置拒绝（退出码 2）", 2, runner_copy.returncode))
    results.append(("stdin:先编后换的拒绝点名那条脚本与出路", 1,
                    int("✗" in runner_copy.stderr and "→" in runner_copy.stderr and COMPILE_THEN_SWAP_SCRIPT in runner_copy.stderr)))
    # 出路经 task-server 提交、不再教 capped.sh 直接起（BASH_COMMAND_DETECTOR_BREAK=compile-first-capped-hint、disable-check、allow-compile-first-bypass 下这一格红）
    results.append(("stdin:先编后换的出路经 task-server 提交、不写 capped.sh 直接起", 1,
                    int(f"task-server/task submit -- python3 {COMPILE_THEN_SWAP_SCRIPT}" in runner_copy.stderr and "capped.sh" not in runner_copy.stderr)))
    writer_copy = entry_as("implementation-writer", "cp ~/.local/share/singlefs/work/runner-x/e161_x.rs crates/singlefs-verification/src/bin/e161_x.rs")
    results.append(("stdin:implementation-writer 同样的 cp 放行（退出码 0）", 0, writer_copy.returncode))
    # 走真实入口：子 agent 在 Bash 里起 claude 会话拒绝（弄坏开关 allow-subagent-claude-launch 与 disable-check 下前四格红）；主 agent 起同一条放行
    claude_refused = entry_as("implementation-writer", "claude -p '接着做'")
    results.append(("stdin:子 agent 起 claude 拒绝（退出码 2）", 2, claude_refused.returncode))
    results.append(("stdin:子 agent 经 \"$CLAUDE_CODE_EXECPATH\" 起 claude 拒绝（退出码 2）", 2, entry_as("clerk", '"$CLAUDE_CODE_EXECPATH" -p x').returncode))
    results.append(("stdin:子 agent 起以 /claude 结尾的可执行文件拒绝（退出码 2）", 2, entry_as("tooling-writer", "/home/u/.local/bin/claude --print x").returncode))
    results.append(("stdin:子 agent 起 claude 被拒时 stderr 点名「派发」一节、写交回主 agent 的出路", 1,
                    int("✗" in claude_refused.stderr and "→" in claude_refused.stderr and "「派发」" in claude_refused.stderr and "由主 agent 派" in claude_refused.stderr)))
    results.append(("stdin:主 agent 起同一条 claude 放行（退出码 0）", 0, entry_as(None, "claude -p '接着做'").returncode))
    launcher_library = project_code_runs_library()
    launcher_names = sorted(launcher_library.launcher_names()) if launcher_library is not None else None
    results.append(("启动器名单与 lib_project_code_runs 的一致", sorted(LAUNCHER_NAMES), launcher_names))
    # 照 bash 不跑的放行：case 里 ;; 之后的模式词（case-patterns-as-commands 下这两格红）
    for label, agent_type, command in (("case 里 ;; 之后的 reboot 模式不当命令", "implementation-writer", 'case "$1" in a) :;; reboot) :;; esac'),
                                       ("case 里 ;; 之后的 claude 模式不当命令", "implementation-writer", 'case "$1" in\n  start) echo s;;\n  claude) echo c;;\nesac')):
        results.append((f"stdin:子 agent {label}放行（退出码 0）", 0, entry_as(agent_type, command).returncode))
    results.append(("stdin:子 agent case 一项的命令里的 reboot 照拒（退出码 2）", 2, entry_as("implementation-writer", 'case "$1" in a) reboot;; esac').returncode))
    # 钩子认得出、判错了的几处，每处一对放与拒，
    # 拒的那一侧照报告里的同形写法；放的那一格后面的注释写弄坏它的开关（共用模块 LIB_SHELL_WORDS_BREAK=<项>，检出钩子 BASH_COMMAND_DETECTOR_BREAK=<项>）
    def d137_command(text, name):
        return next(command for command in shell_words.commands_at_command_position(text, work).commands if command.words and os.path.basename(command.words[0]) == name)
    def d137_python_scan(code):
        found_texts, found_unjudged = [], []
        shell_words.scan_python(code, found_texts, found_unjudged)
        return found_texts, found_unjudged
    def d137_sed_scan(script):
        found_texts, found_unjudged = [], []
        shell_words.sed_script_commands(script, found_texts, found_unjudged, shell_word=True)
        return found_texts, found_unjudged
    heredoc_code, heredoc_problem, _ = shell_words.interpreter_standard_input(d137_command('D=/tmp/d137-e1; python3 - <<EOF\nprint("$D")\nEOF', "python3"))
    results.append(('d137 误拒 E1：python3 读 heredoc，正文变量前面赋过字面值，照代入、不交判不了', (None, True), (heredoc_problem, 'print("/tmp/d137-e1")' in heredoc_code)))   # heredoc-variables-unexpanded
    for label, text in (("正文里有一个变量没赋过", 'python3 - <<EOF\nprint("$D137_NEVER_ASSIGNED")\nEOF'),
                        ("正文里有命令替换", 'D=/tmp/d137-e1; python3 - <<EOF\nprint("$D $(date)")\nEOF')):
        results.append((f"d137 误拒 E1：python3 读 heredoc，{label}，照交判不了", True, shell_words.interpreter_standard_input(d137_command(text, "python3"))[1] is not None))
    results.append(('d137 误拒 E6：subprocess.run([du, -sch] + ds)，ds 是赋过的字面列表，照算出的命令交出', (["du -sch f.rs"], []),
                    d137_python_scan('import subprocess; ds=["f.rs"]; subprocess.run(["du","-sch"]+ds)')))   # python-list-concatenation-unresolved
    results.append(('d137 误拒 E6：subprocess.run([du, -sch, *ds])，ds 算不出，照命令词 du 交出', ([f"du -sch {shell_words.PYTHON_UNRESOLVED_ARGUMENT}"], []),
                    d137_python_scan('import subprocess, sys\nds = sys.argv[1:]\nsubprocess.run(["du", "-sch", *ds])')))   # 同上
    for label, code in (("subprocess.run([git, log] + ds)", 'import subprocess, sys\nds = sys.argv[1:]\nsubprocess.run(["git", "log"] + ds)'),
                        ("subprocess.run([git, log, *ds])", 'import subprocess, sys\nds = sys.argv[1:]\nsubprocess.run(["git", "log", *ds])')):
        git_texts, git_unjudged = d137_python_scan(code)
        results.append((f"d137 误拒 E6：{label}，ds 算不出，命令词 git 照认不出", ([], True), (git_texts, bool(git_unjudged))))
    launcher_library = project_code_runs_library()
    known_launchers = ((set(launcher_library.LAUNCHER_OPTIONS_WITH_VALUE) | set(launcher_library.EXTRA_LAUNCHER_NAMES)) if launcher_library is not None
                       else {"读不到 lib_project_code_runs.py"})
    results.append(("d137 误拒 E6：lib_project_code_runs.py 认的启动器都在共用模块的 ARGUMENT_COMMAND_LAUNCHER_NAMES 里", set(),
                    known_launchers - shell_words.ARGUMENT_COMMAND_LAUNCHER_NAMES))
    results.append(('d137 误拒 E7：python 只调 compile 不执行，不交判不了', [], d137_python_scan("import sys; compile(open(sys.argv[1]).read(), sys.argv[1], 'exec')")[1]))   # python-compile-as-execution
    results.append(("d137 误拒 E7：python 调 exec 跑读进来的代码，照交判不了", True, bool(d137_python_scan("exec(open('gen.py').read())")[1])))
    results.append(("d137 误拒 E7：compile 的代码是字面的，照扫字面代码", (["cargo test --workspace"], []),
                    d137_python_scan("""compile("import subprocess; subprocess.run(['cargo', 'test', '--workspace'])", 'x', 'exec')""")))
    results.append(('d137 误拒 E8：sed 的 s 命令标志区有 GNU sed 不认的字（分隔符被转义的反斜杠提前收尾），编译就报错、什么都不起', ([], []), d137_sed_scan("s|ab\\\\| none zz|ab\\\\| none zz|")))   # sed-unknown-flags-as-execution
    for label, script in (("s|ab|cd|e 真带 e 标志", "s|ab|cd|e"), ("s/a/b/e#c 的 e 标志在 # 前面", "s/a/b/e#c")):
        results.append((f"d137 误拒 E8：sed 的 {label}，照交判不了", True, bool(d137_sed_scan(script)[1])))
    missing_directory = os.path.join(work, "d137-missing-directory")
    for label, text, name, want in (('d137 误拒 E9：cd 不在的目录 && ( bash x.sh )，子 shell 照 && 守着', f"cd {missing_directory} && (bash x.sh)", "x.sh", [True]),         # cd-subshell-guards-unseen
                                    ('d137 误拒 E9：cd 不在的目录 && time ( bash x.sh )，照 && 守着', f"cd {missing_directory} && time (bash x.sh)", "x.sh", [True]),    # 同上
                                    ("d137 误拒 E9：cd 不在的目录 && 子 shell 之后用分号接的 bash y.sh 照跑，不守", f"cd {missing_directory} && (bash x.sh); bash y.sh", "y.sh", [False]),
                                    ("d137 误拒 E9：cd 不在的目录; ( bash x.sh ) 照跑，不守", f"cd {missing_directory}; (bash x.sh)", "x.sh", [False])):
        results.append((label, want, [command.cd_failure_skips for command in shell_words.commands_at_command_position(text, work).commands if command.words[:1] == [name]]))
    results.append(("d137 误拒：名字() { 函数体 } 定义之后调用，函数体照判", True,
                    any(command.words[:1] == ["scripts/sample-called.sh"] for command in shell_words.commands_at_command_position(
                        "f() { bash scripts/sample-called.sh start --label m -- sleep 1; }; f", work, uncalled_functions_skipped=True).commands)))
    for label, command in zip(('stdin:d137 误拒：builtin coproc 里 coproc 不是内建、bash 报错不跑，放行（退出码 0）', 'stdin:d137 误拒：builtin -- coproc 里 coproc 不是内建、bash 报错不跑，放行（退出码 0）'), ("builtin coproc sleep 1", "builtin -- coproc sleep 1")):   # builtin-coproc-detached（拒的那一侧是后面「一百三十七：coproc 照拒」那一格）
        results.append((label, 0, entry_as(None, command).returncode))
    def scope_entry(agent_type, command, cwd=None):
        payload = {"tool_name": "Bash", "session_id": "s", "tool_input": {"command": command, "run_in_background": False}}
        if agent_type:
            payload["agent_type"] = agent_type
        if cwd:
            payload["cwd"] = cwd
        return subprocess.run(["bash", os.path.join(scope_hooks, "bash-command-detector.sh")], capture_output=True, text=True,
                              env=dict(os.environ, AGENT_HOOK_DETECTIONS=detections), input=json.dumps(payload))
    scope_refused = scope_entry("clerk", "python3 scripts/kb/replace-once.py .claude/kb/checks-owed.md a b")
    results.append(("stdin:clerk 在 Bash 里用 replace-once.py 写知识库拒绝（退出码 2）", 2, scope_refused.returncode))
    results.append(("stdin:写范围的拒绝点名写法、目标、写范围表与出路", 1,
                    int(all(fragment in scope_refused.stderr for fragment in ("✗ clerk", "replace-once.py .claude/kb/checks-owed.md", "agent-write-scope.tsv",
                                                                               "→ 怎么办：范围外的改动不自己做", "records/**")))))
    results.append(("stdin:clerk 在 Bash 里写自己的草稿目录放行（退出码 0）", 0,
                    scope_entry("clerk", f"python3 scripts/kb/replace-once.py {new_root}/clerk-x/notes.md a b").returncode))
    results.append(("stdin:主 agent 同一条写知识库的命令放行（退出码 0）", 0, scope_entry(None, "python3 scripts/kb/replace-once.py .claude/kb/checks-owed.md a b").returncode))
    scope_linked = scope_entry("clerk", "echo x > records/kb-link/a.md")
    results.append(("stdin:范围内的链接指到仓里范围外，经它写拒绝（退出码 2）、拒绝说明写出解开符号链接之后的落点", (2, True),
                    (scope_linked.returncode, "records/kb-link/a.md（解开符号链接之后落在 .claude/kb/a.md）" in scope_linked.stderr)))
    before_scope_noted = sum(1 for _ in open(detections)) if os.path.exists(detections) else 0
    scope_variable = scope_entry("clerk", f'mkdir -p {new_root}/clerk-x/originals && for f in records/a.md .claude/kb/b.md; do cp -p "$f" "{new_root}/clerk-x/originals/$(echo $f | tr / _)"; done')
    scope_lines = open(detections, encoding="utf-8").read().splitlines()[before_scope_noted:] if os.path.exists(detections) else []
    results.append(("stdin:目标里有命令替换的拒绝（退出码 2）、不记检出", (2, 0), (scope_variable.returncode, len(scope_lines))))
    results.append(("stdin:算不出落点的拒绝说明点名算不出、copy-originals.py 与写死的词的 for 循环两条出路", 1,
                    int(all(fragment in scope_variable.stderr for fragment in ("✗ clerk", "算不出最终落点", "$(echo records/a.md | tr / _)",
                                                                                "python3 scripts/agents/copy-originals.py copy <草稿目录> <仓内文件>…",
                                                                                "for 名字 in 写死的词…", "逐条写字面路径")))))
    before_scope_settled = sum(1 for _ in open(detections)) if os.path.exists(detections) else 0
    scope_settled = scope_entry("clerk", f"for b in one two; do echo x > {new_root}/clerk-x/breaks/$b.out; done; cd records && for k in a b; do echo x > gz-$k/.gate-cells; done")
    results.append(("stdin:写死的词的循环变量拼的目标放行（退出码 0）、不记检出", (0, 0),
                    (scope_settled.returncode, (sum(1 for _ in open(detections)) if os.path.exists(detections) else 0) - before_scope_settled)))
    scope_cat_body = scope_entry("clerk", f"cat > {new_root}/clerk-x/x.sh <<'EOF'\nsed -i 's/a/b/' .claude/kb/a.md\nEOF")
    results.append(("stdin:cat 写草稿脚本的 heredoc 正文里有 sed -i 知识库放行（退出码 0）", 0, scope_cat_body.returncode))
    scope_bash_body = scope_entry("clerk", "bash <<'EOF'\nsed -i 's/a/b/' .claude/kb/a.md\nEOF")
    results.append(("stdin:喂给 bash 的 heredoc 里同一句 sed -i 知识库拒绝（退出码 2）", 2, scope_bash_body.returncode))
    scope_pipe_python = scope_entry("clerk", "cat <<'EOF' | python3 -\nopen('.claude/kb/a.md', 'w').write('x')\nEOF")
    results.append(("stdin:管道喂给 python 的 heredoc 里 open(知识库, 'w') 拒绝（退出码 2，写范围拒）", (2, True),
                    (scope_pipe_python.returncode, "在 Bash 里写了它写范围之外的仓内文件" in scope_pipe_python.stderr)))
    results.append(("stdin:管道喂给 python 的 heredoc 里 open(知识库, 'w') 同样被 ⑧ 拒（有 Edit 的子 agent 就地改仓内文件）", 1,
                    int("就地改仓内文件" in scope_pipe_python.stderr)))
    results.append(("stdin:管道喂给 grep 的 heredoc 正文放行（退出码 0）", 0,
                    scope_entry("clerk", "cat <<'EOF' | grep x\nopen('.claude/kb/a.md', 'w')\nsed -i 's/a/b/' .claude/kb/a.md\nEOF").returncode))
    results.append(("stdin:cd 到草稿目录之后 sed -i 相对路径放行（退出码 0）", 0,
                    scope_entry("clerk", f"cd {new_root}/clerk-x && sed -i 's/a/b/' x.py").returncode))
    scope_sed_in_records = scope_entry("clerk", "cd records && sed -i 's/a/b/' x.md")
    results.append(("stdin:cd 到仓内 records 之后 sed -i 相对路径被 ⑧ 拒绝（退出码 2）", (2, True),
                    (scope_sed_in_records.returncode, "就地改仓内文件" in scope_sed_in_records.stderr)))
    results.append(("stdin:hook 输入的 cwd 在 /tmp 下时 sed -i 相对路径放行（退出码 0）", 0,
                    scope_entry("clerk", "sed -i 's/a/b/' x.py", "/tmp").returncode))
    results.append(("stdin:hook 输入没有 cwd 时同一句 sed -i 相对路径拒绝（退出码 2）", 2, scope_entry("clerk", "sed -i 's/a/b/' x.py").returncode))
    before_pwd_noted = sum(1 for _ in open(detections)) if os.path.exists(detections) else 0
    scope_pwd = scope_entry("clerk", f"cd {new_root}/clerk-x && echo x > $PWD/a.md; W=$(mktemp -d) && cp -a records/a.md $W/")
    results.append(("stdin:cd 之后的 $PWD/… 与 mktemp -d 赋的变量放行（退出码 0）、不记检出", (0, 0),
                    (scope_pwd.returncode, (sum(1 for _ in open(detections)) if os.path.exists(detections) else 0) - before_pwd_noted)))
    results.append(("stdin:拒了的写范围不记检出", 0, sum(1 for line in (open(detections, encoding="utf-8").read().splitlines() if os.path.exists(detections) else [])
                                                      if "replace-once.py .claude/kb/checks-owed.md" in line)))
    unknown_break_items = [item for item in os.environ.get(DETECTOR_BREAK_VARIABLE, "").split(",") if item and item not in KNOWN_DETECTOR_BREAK_ITEMS]
    results.append((f"{DETECTOR_BREAK_VARIABLE} 里的项都认得（认得的：{'、'.join(KNOWN_DETECTOR_BREAK_ITEMS)}）", [], unknown_break_items))
    renamed, renamed_recorded = through_entry(f"cp /tmp/new.sh scripts/.running.sh.installing && mv scripts/.running.sh.installing {running}")
    results.append(("stdin:写到临时文件再 mv 换上放行（退出码 0）、不记检出", (0, 0), (renamed.returncode, renamed_recorded)))
    # 同走 run_in_background 的那一条：只记检出
    results.append(("等待循环:同一条走 run_in_background 记一条检出", 1,
                    record({"tool_name": "Bash", "session_id": "s", "tool_input": {"command": "until grep -q x f; do sleep 10; done", "run_in_background": True}},
                           detections)))
    # 走真实入口：从标准输入喂 JSON，起看门狗的写法不对退出码 2、stderr 给出路，写法对的退出码 0
    script_for_watchdog = os.path.join(hook_dir, "bash-command-detector.sh")
    watchdog_environment = dict(os.environ, AGENT_HOOK_DETECTIONS=detections)
    foreground_watchdog = subprocess.run(["bash", script_for_watchdog], capture_output=True, text=True, env=watchdog_environment,
                                         input=json.dumps({"tool_name": "Bash", "session_id": "s", "tool_input": {"command": "bash scripts/agents/watch.sh a1 > /dev/null 2>&1 &"}}))
    results.append(("stdin:前台起看门狗拒绝（退出码 2）", 2, foreground_watchdog.returncode))
    results.append(("stdin:拒绝时 stderr 写了出路（run_in_background: true 的起法）", 1,
                    int("run_in_background: true" in foreground_watchdog.stderr and "→" in foreground_watchdog.stderr)))
    background_watchdog = subprocess.run(["bash", script_for_watchdog], capture_output=True, text=True, env=watchdog_environment,
                                         input=json.dumps({"tool_name": "Bash", "session_id": "s", "tool_input": {"command": "bash scripts/agents/watch.sh a1", "run_in_background": True}}))
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
    # 走真实入口：判不了的照常放行、stderr 报一句、记一条检出——hook 输入不是 JSON 对象（坏 JSON、解出来是数组；会话号写读不到，诊断带钩子名与开头的字节）、
    # 各道判的时候抛异常（tool_input.command 不是字符串）、here-string 喂给 bash 的是变量；拒了的命令里带着判不了的输入，照记一条（结局「被拒」）。
    # BASH_COMMAND_DETECTOR_BREAK=failure-unrecorded 下这五格红；LIB_SHELL_WORDS_BREAK=shell-standard-input-unread 下后两格与「here-string 里前台起看门狗」一格红
    def detection_lines():
        return sum(1 for _ in open(detections, encoding="utf-8")) if os.path.exists(detections) else 0
    def last_detection_entry():
        if not os.path.exists(detections):
            return {}
        with open(detections, encoding="utf-8") as handle:
            lines = handle.read().splitlines()
        return json.loads(lines[-1]) if lines else {}
    for label, raw in (("stdin:坏 JSON 放行（退出码 0）、stderr 写诊断、记一条检出", b'{"tool_name": "Bash", "tool_input": '),
                       ("stdin:不是 JSON 对象放行（退出码 0）、stderr 写诊断、记一条检出", b'["Bash"]')):
        before = detection_lines()
        unreadable = subprocess.run(["bash", script], input=raw, capture_output=True, env=dict(os.environ, AGENT_HOOK_DETECTIONS=detections))
        stderr_text = unreadable.stderr.decode("utf-8", "replace")
        results.append((label, (0, True, 1, "读不到"),
                        (unreadable.returncode, "bash-command-detector.sh 收到的 hook 输入不是 JSON 对象" in stderr_text and repr(raw[:10])[2:-1] in stderr_text,
                         detection_lines() - before, last_detection_entry().get("session_id"))))
    before = detection_lines()
    raising = subprocess.run(["bash", script], input=json.dumps({"tool_name": "Bash", "session_id": "s-raise", "tool_input": {"command": ["sleep", "1"]}}),
                             capture_output=True, text=True, env=dict(os.environ, AGENT_HOOK_DETECTIONS=detections))
    results.append(("stdin:判的时候抛异常放行（退出码 0）、stderr 写诊断、记一条检出", (0, True, 1, "s-raise"),
                    (raising.returncode, "bash-command-detector.sh 没判成" in raising.stderr, detection_lines() - before, last_detection_entry().get("session_id"))))
    before = detection_lines()
    fed_variable = subprocess.run(["bash", script], input=json.dumps({"tool_name": "Bash", "session_id": "s-fed", "tool_input": {"command": 'bash <<< "$CMD"'}}),
                                  capture_output=True, text=True, env=dict(os.environ, AGENT_HOOK_DETECTIONS=detections))
    results.append(("stdin:here-string 喂给 bash 的是变量放行（退出码 0）、记一条检出", (0, 1, True), (fed_variable.returncode, detection_lines() - before,
                    any("here-string `$CMD`" in finding for finding in last_detection_entry().get("findings", [])))))
    before = detection_lines()
    refused_fed = subprocess.run(["bash", script], input=json.dumps({"tool_name": "Bash", "session_id": "s-fed-refused", "tool_input": {
                                     "command": 'until grep -q x f; do sleep 10; done; grep -h step steps.sh | bash'}}),
                                 capture_output=True, text=True, env=dict(os.environ, AGENT_HOOK_DETECTIONS=detections))
    results.append(("stdin:拒了的命令里有判不了的输入（grep 的输出经管道喂给 bash），照记一条、结局被拒", (2, 1, "被拒"),
                    (refused_fed.returncode, detection_lines() - before, last_detection_entry().get("outcome"))))
    fed_watchdog = subprocess.run(["bash", script], input=json.dumps({"tool_name": "Bash", "session_id": "s", "tool_input": {
                                      "command": "bash <<< 'bash scripts/agents/watch.sh a1 > /dev/null 2>&1 &'"}}),
                                  capture_output=True, text=True, env=dict(os.environ, AGENT_HOOK_DETECTIONS=detections))
    results.append(("stdin:here-string 里前台起看门狗照拒（退出码 2）", 2, fed_watchdog.returncode))

    # 一百三十七节那一批（records/2026-10-29-一百三十七节那一轮调度记录.md 件 1）：每格后面的注释写弄坏它的开关
    def batch137_entry(command, agent_type=None, run_in_background=False, environment=None):
        payload = {"tool_name": "Bash", "session_id": "s-137", "tool_input": {"command": command, "run_in_background": run_in_background}}
        if agent_type:
            payload["agent_type"] = agent_type
        return subprocess.run(["bash", script], input=json.dumps(payload), capture_output=True, text=True,
                              env=environment or dict(os.environ, AGENT_HOOK_DETECTIONS=detections))
    for label, command, agent_type, background, want in (
            ("一百三十七：子 agent here-string 喂给 bash 的 claude 照拒（退出码 2）", "bash <<< 'claude -p hi'", "clerk", False, 2),          # claude-launch-layers-narrow
            ("一百三十七：子 agent env -S 拆串里的 claude 照拒（退出码 2）", "env -S 'claude -p hi'", "clerk", False, 2),                      # 同上
            ("一百三十七：子 agent xargs 起的 claude 照拒（退出码 2）", "echo hi | xargs claude -p", "clerk", False, 2),                        # 同上
            ("一百三十七：子 agent ssh 起的 claude 照拒（退出码 2）", "ssh localhost 'claude -p hi'", "clerk", False, 2),                       # 同上
            ("一百三十七：看门狗输出经 tee 丢进 /dev/null 照拒（退出码 2）", "bash scripts/agents/watch.sh a1 | tee /dev/null", None, True, 2),  # watchdog-tee-null-unseen
            ("一百三十七：两个 & 只等其中一个照拒（退出码 2）", "sleep 1 & p1=$!; sleep 2 & p2=$!; wait $p1", None, True, 2),                  # partial-wait-allowed
            ("一百三十七：两个 & 各等一个放行（退出码 0）", "sleep 1 & p1=$!; sleep 2 & p2=$!; wait $p1; wait $p2", None, True, 0),
            ("一百三十七：一个 & 之后 wait -n 放行（退出码 0）", "sleep 1 & wait -n", None, True, 0),                                         # wait-forms-unseen
            ("一百三十七：一个 & 之后 tail --pid 等它放行（退出码 0）", "sleep 2 & tail --pid=$! -f /dev/null", None, True, 0),                 # 同上
            ("一百三十七：一个 & 之后 kill -0 轮询等它放行（退出码 0）", "sleep 1 & p=$!; while kill -0 $p 2>/dev/null; do sleep 0.2; done", None, True, 0),  # 同上
            ("一百三十七：wait 写在函数里、函数被调用放行（退出码 0）", "w() { wait; }; sleep 1 & w", None, True, 0),                        # 同上
            ("一百三十七：coproc 照拒（退出码 2）", "coproc sleep 1", None, False, 2)):                                                       # coproc-prefix-unseen（连同 LIB_SHELL_WORDS_BREAK=coproc-as-command 时不红）
        results.append((label, want, batch137_entry(command, agent_type, background).returncode))
    # 暂存区非空时 git stash、git rm -r --cached .、git checkout HEAD -- . 与子命令写成变量的 git（⑮；whole-index-other-forms-unseen 下前三格红，
    # git-subcommand-unexpanded-allowed 下第四格红）：在一个暂存了改动的临时仓里判
    index_repository = os.path.join(work, "index-repository-137")
    os.makedirs(index_repository)
    subprocess.run(["git", "-C", index_repository, "init", "-q"], check=True)
    with open(os.path.join(index_repository, "a.txt"), "w", encoding="utf-8") as handle:
        handle.write("a\n")
    subprocess.run(["git", "-C", index_repository, "add", "a.txt"], check=True)
    for label, command in (("一百三十七：暂存区非空时 git stash 照拒（退出码 2）", "git stash"),
                           ("一百三十七：暂存区非空时 git rm -r --cached . 照拒（退出码 2）", "git rm -r --cached -q ."),
                           ("一百三十七：暂存区非空时 git checkout HEAD -- . 照拒（退出码 2）", "git checkout HEAD -- ."),
                           ("一百三十七：暂存区非空时子命令写成没赋过值的变量照拒（退出码 2）", "git $SUBCOMMAND_FROM_ELSEWHERE")):
        results.append((label, 2, batch137_entry(f"cd {index_repository} && {command}").returncode))
    results.append(("一百三十七：暂存区非空时 git stash push -- 具体路径放行（退出码 0）", 0, batch137_entry(f"cd {index_repository} && git stash list").returncode))
    # 拒绝说明：重定向目标上的花括号点名 ambiguous redirect（brace-target-reason-generic 下红）；等待循环点名真会结束的几种也要套 timeout（wait-loop-rewrite-unexplained 下红）
    brace_target = batch137_entry(f"echo hi > {work}/o.{{a,b}}", "implementation-writer")
    results.append(("一百三十七：重定向目标花括号照拒，拒绝说明点名 ambiguous redirect 与改法", (2, True),
                    (brace_target.returncode, "ambiguous redirect" in brace_target.stderr and "逐份写" in brace_target.stderr)))
    counted_loop = batch137_entry("i=0; while [ $i -lt 3 ]; do sleep 0.1; i=$((i+1)); done")
    results.append(("一百三十七：带计数上限的前台等待循环照拒，拒绝说明点名真会结束的几种也要套 timeout", (2, True),
                    (counted_loop.returncode, "真会结束的循环" in counted_loop.stderr and "tail --pid" in counted_loop.stderr)))
    # 钩子 bash 这一层起不来时也记一条检出（bash-layer-unrecorded 下红）：PATH 前面放一份只对 /dev/fd/3 退 127 的假 python3
    shim = os.path.join(work, "bash-layer-shim")
    os.makedirs(shim, exist_ok=True)
    with open(os.path.join(shim, "python3"), "w", encoding="utf-8") as handle:
        handle.write(f'#!/bin/bash\nif [ "$1" = /dev/fd/3 ]; then exit 127; fi\nexec {sys.executable} "$@"\n')
    os.chmod(os.path.join(shim, "python3"), 0o755)
    before = detection_lines()
    bash_layer = batch137_entry("ls", "clerk", environment=dict(os.environ, AGENT_HOOK_DETECTIONS=detections, PATH=shim + os.pathsep + os.environ.get("PATH", "")))
    results.append(("一百三十七：钩子 bash 这一层起不来（假 python3 退 127）：放行（退出码 0）、记一条检出", (0, 1, "s-137", "只记"),
                    (bash_layer.returncode, detection_lines() - before, last_detection_entry().get("session_id"), last_detection_entry().get("outcome"))))
    # PATH 上没有 python3（python-missing-unrecorded 下红）：PATH 只放记检出要用的 cat、date、mkdir、dirname
    no_python = os.path.join(work, "no-python-path")
    os.makedirs(no_python, exist_ok=True)
    for tool in ("cat", "date", "mkdir", "dirname"):
        if shutil.which(tool) and not os.path.lexists(os.path.join(no_python, tool)):
            os.symlink(shutil.which(tool), os.path.join(no_python, tool))
    before = detection_lines()
    no_python_payload = {"tool_name": "Bash", "session_id": "s-137", "agent_type": "clerk", "tool_input": {"command": "claude -p hi", "run_in_background": False}}
    no_python_run = subprocess.run([shutil.which("bash"), script], input=json.dumps(no_python_payload), capture_output=True, text=True,
                                   env=dict(os.environ, AGENT_HOOK_DETECTIONS=detections, PATH=no_python))
    results.append(("d137 H13：PATH 上没有 python3：放行（退出码 0）、记一条检出", (0, 1, "s-137", "只记"),
                    (no_python_run.returncode, detection_lines() - before, last_detection_entry().get("session_id"), last_detection_entry().get("outcome"))))
    # 检出记录的位置照 lib_work_root.py 取、不退回家目录：AGENT_HOOK_DETECTIONS、SINGLEFS_WORK_ROOT、SINGLEFS_DATA_ROOT 都不设，PATH 上没有 python3，HOME 换成临时目录
    unrooted_home = os.path.join(work, "unrooted-home")
    os.makedirs(unrooted_home, exist_ok=True)
    unrooted_environment = {name: value for name, value in os.environ.items() if name not in ("AGENT_HOOK_DETECTIONS", "SINGLEFS_WORK_ROOT", "SINGLEFS_DATA_ROOT")}
    unrooted_run = subprocess.run([shutil.which("bash"), script], input=json.dumps(no_python_payload), capture_output=True, text=True,
                                  env=dict(unrooted_environment, HOME=unrooted_home, PATH=no_python))
    results.append(("d137 H13：PATH 上没有 python3、工作根取不到：放行（退出码 0）、不往家目录下写、stderr 说检出没记成", (0, False, True),
                    (unrooted_run.returncode, os.path.exists(os.path.join(unrooted_home, ".local")), "检出没记成" in unrooted_run.stderr)))
    # 走真实入口：子 agent 前台 timeout 超过 240000 毫秒放行、记一条检出（弄坏开关 subagent-timeout-unnoted 下第一格红）；不超过的、主 agent 的不记
    def timeout_entry(agent_type, timeout):
        timeout_payload = {"tool_name": "Bash", "session_id": "s-timeout", "tool_input": {"command": "ls", "run_in_background": False, "timeout": timeout}}
        if agent_type:
            timeout_payload["agent_type"] = agent_type
        before_timeout = detection_lines()
        completed = subprocess.run(["bash", script], input=json.dumps(timeout_payload), capture_output=True, text=True,
                                   env=dict(os.environ, AGENT_HOOK_DETECTIONS=detections))
        return completed.returncode, detection_lines() - before_timeout
    timeout_over = timeout_entry("implementation-writer", 600000)
    results.append(("stdin:子 agent 前台 timeout 600000 毫秒放行（退出码 0）、记一条检出", (0, 1, True),
                    (timeout_over[0], timeout_over[1], any("timeout 是 600000 毫秒" in finding for finding in last_detection_entry().get("findings", [])))))
    results.append(("stdin:子 agent 前台 timeout 120000 毫秒放行、不记", (0, 0), timeout_entry("implementation-writer", 120000)))
    results.append(("stdin:主 agent 前台 timeout 600000 毫秒放行、不记", (0, 0), timeout_entry(None, 600000)))
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
    # 走真实入口：一条命令同时踩终止进程、放出追踪、前台等待循环三类，一次列全（BASH_COMMAND_DETECTOR_BREAK=stop-at-first-refusal 下第一格红）；
    # 只记的检出记录写结局「只记」（detection-outcome-dropped 下第二格红）
    three_kinds = signal_entry("kill -TERM -1234; nohup sleep 100 & disown; until false; do sleep 1; done")
    results.append(("stdin:一条命令同时踩几类时一次列全（终止进程、放出追踪、前台等待循环）", (2, True),
                    (three_kinds.returncode, all(fragment in three_kinds.stderr for fragment in
                                                 ("踩了 3 类", "✗ 终止进程的写法", "✗ 把活放出了追踪", "✗ 前台的等待循环没有超时")))))
    signal_entry("until false; do sleep 1; done", True)
    recorded_lines = open(detections, encoding="utf-8").read().splitlines() if os.path.exists(detections) else []
    last_outcome = json.loads(recorded_lines[-1]).get("outcome") if recorded_lines else None   # 没记下检出时这一格照样红，不抛异常
    # ⑮ 暂存区非空时不带具体路径清整个索引：临时仓两份（一份暂存了两个文件、一份暂存区空），进程里判与走入口各几格；(说明, 起点仓, 命令, 该不该拒)
    staged_repository, clean_repository = os.path.join(work, "index-staged"), os.path.join(work, "index-clean")
    for index_repository in (staged_repository, clean_repository):
        subprocess.run(["git", "init", "-q", index_repository], check=True)
        subprocess.run(["git", "-C", index_repository, "-c", "user.email=t@t", "-c", "user.name=t", "-c", "commit.gpgsign=false",
                        "commit", "-q", "--allow-empty", "-m", "base"], check=True)
        for name in ("a.txt", "b.txt"):
            with open(os.path.join(index_repository, name), "w", encoding="utf-8") as handle:
                handle.write(name + "\n")
    subprocess.run(["git", "-C", staged_repository, "add", "a.txt", "b.txt"], check=True)
    index_cases = [
        ("清整个索引:git reset 拒", staged_repository, "git reset", 1),
        ("清整个索引:git reset HEAD 拒", staged_repository, "git reset HEAD", 1),
        ("清整个索引:git reset --mixed 拒", staged_repository, "git reset --mixed", 1),
        ("清整个索引:git reset -q --hard 拒", staged_repository, "git reset -q --hard", 1),
        ("清整个索引:git restore --staged . 拒", staged_repository, "git restore --staged .", 1),
        ("清整个索引:git restore --staged :/ 拒", staged_repository, "git restore --staged :/", 1),
        ("清整个索引:git -C 点名暂存了的仓、起点在别处，拒", clean_repository, f"git -C {staged_repository} reset", 1),
        ("清整个索引:cd 进暂存了的仓之后 git reset HEAD 拒", clean_repository, f"cd {staged_repository} && git reset HEAD", 1),
        ("清整个索引:cd 到没赋过值的变量之后 git reset，暂存区判不了，拒", clean_repository, 'cd "$WHERE" && git reset', 1),
        ("清整个索引:git reset HEAD -- a.txt 带路径放行", staged_repository, "git reset HEAD -- a.txt", 0),
        ("清整个索引:git reset a.txt 带路径放行", staged_repository, "git reset a.txt", 0),
        ("清整个索引:git restore --staged -- a.txt b.txt 带路径放行", staged_repository, "git restore --staged -- a.txt b.txt", 0),
        ("清整个索引:git reset --soft HEAD 不碰索引，放行", staged_repository, "git reset --soft HEAD", 0),
        ("清整个索引:暂存区为空 git reset 放行", clean_repository, "git reset", 0),
        ("清整个索引:暂存区为空 git restore --staged . 放行", clean_repository, "git restore --staged .", 0),
        ("清整个索引:grep 参数里的 git reset 不判", staged_repository, "grep -rn 'git reset' a.txt", 0),
    ]
    for label, start, command, want in index_cases:
        results.append((label, want, 1 if whole_index_reset_refusal(command, start) else 0))
    for agent_type in (None, "clerk"):
        index_payload = {"tool_name": "Bash", "session_id": "s", "cwd": staged_repository, "tool_input": {"command": "git reset", "run_in_background": False}}
        if agent_type:
            index_payload["agent_type"] = agent_type
        index_entry = subprocess.run(["bash", os.path.join(hook_dir, "bash-command-detector.sh")], capture_output=True, text=True,
                                     env=dict(os.environ, AGENT_HOOK_DETECTIONS=os.path.join(work, "index-detections.jsonl")), input=json.dumps(index_payload))
        results.append((f"stdin:{'子 agent' if agent_type else '主 agent'} 在暂存区非空时 git reset 拒、出路写只清自己的路径（退出码 2）", (2, True),
                        (index_entry.returncode, "git restore --staged -- <路径…>" in index_entry.stderr)))
    # ⑰ 往仓里的文件写字面内容：临时目录里造一个假仓（判据库用 hook 所在仓的那一份），进程里判几格，走入口两格；
    # 样本串里的会话名与钟点拆开拼，这份源码里不出现字面写法（门禁 doc-text 也扫它）；(说明, 命令, 该命中的形态)
    literal_session = "".join(("singlefs-", "e1"))
    literal_clock = "".join(("用户 23", ":", "10 定"))
    literal_repository, literal_outside = os.path.join(work, "literal-repo"), os.path.join(work, "literal-outside")
    for literal_directory in (os.path.join(literal_repository, "records"), os.path.join(literal_repository, "scripts"), literal_outside):
        os.makedirs(literal_directory, exist_ok=True)
    literal_cases = [
        ("字面写入:heredoc 追加会话名拒", f"cat >> records/x.md <<'EOF'\n已发消息，{literal_session} 回话\nEOF", ("会话号",)),
        ("字面写入:heredoc 追加批次代号拒", "cat >> records/x.md <<'EOF'\n- 24c-1 交回\nEOF", ("批次代号",)),
        ("字面写入:printf 追加钟点拒", f"printf '%s\\n' '{literal_clock}' >> records/x.md", ("钟点",)),
        ("字面写入:echo 经管道交 tee -a 追加批次代号拒", "echo '22a 的接口改了' | tee -a records/x.md", ("批次代号",)),
        ("字面写入:cd 进仓里之后按相对路径写照判", "cd records && cat > notes.md <<'EOF'\n批 10 甲交回\nEOF", ("批次代号",)),
        ("字面写入:bash -c 里的 echo 追加会话名照判", f"bash -c \"echo '{literal_session} 回话' >> records/x.md\"", ("会话号",)),
        ("字面写入:here-string 喂给 cat 写批次代号照判", "cat <<< '24c-1 交回' >> records/x.md", ("批次代号",)),
        ("字面写入:干净内容放行", "cat >> records/x.md <<'EOF'\n配置判法那一份补丁交回\nEOF", ()),
        ("字面写入:两位会话短号放行", "echo '已告诉 87 与 7a' >> records/x.md", ()),
        ("字面写入:仓外路径不判", f"cat >> {literal_outside}/notes.md <<'EOF'\n24c-1 交回，{literal_session}\nEOF", ()),
        ("字面写入:排除前缀下的样本不判", f"echo '{literal_session}' > .claude/gate.d/fixtures/x/expect", ()),
        ("字面写入:写进 .py 的批次代号不判（批次代号只判 .md、.tsv、.txt）", "echo '# 24c-1 交回' >> scripts/x.py", ()),
        ("字面写入:目标算不出的不判", "cat >> \"$TARGET\" <<'EOF'\n24c-1 交回\nEOF", ()),
        ("字面写入:cat 拷文件不是字面内容，不判", "cat a.md >> records/x.md", ()),
    ]
    for label, command, want in literal_cases:
        literal_hits = literal_notation_hits(command, literal_repository, literal_repository, notations_library_path(hook_dir))
        results.append((label, want, tuple(sorted({form_name for _, _, form_name, _, _ in literal_hits}))))
    for label, command, want in (
            ("stdin:Bash heredoc 往仓里追加会话名拒绝（退出码 2，点名会话号、出路指到 insert-row.py 与 replace-once.py）",
             f"cat >> research/bash-command-detector-selftest-literal.md <<'EOF'\n已发消息，{literal_session} 回话\nEOF", (2, True)),
            ("stdin:Bash heredoc 往仓里追加干净内容放行（退出码 0）",
             "cat >> research/bash-command-detector-selftest-literal.md <<'EOF'\n配置判法那一份补丁交回\nEOF", (0, False))):
        literal_entry = subprocess.run(["bash", os.path.join(hook_dir, "bash-command-detector.sh")], capture_output=True, text=True,
                                       env=dict(os.environ, AGENT_HOOK_DETECTIONS=os.path.join(work, "literal-detections.jsonl")),
                                       input=json.dumps({"tool_name": "Bash", "session_id": "s", "cwd": repository_root_of(hook_dir),
                                                         "tool_input": {"command": command, "run_in_background": False}}))
        results.append((label, want, (literal_entry.returncode, all(fragment in literal_entry.stderr
                                                                   for fragment in (literal_session, "会话号", "insert-row.py", "replace-once.py")))))
    # ⑱ 带路径撤掉工作区里没暂存的改动：临时仓一份（a.txt 有没暂存的改动、b.txt 干净、c.txt 改了而且整份暂存、u.txt 没跟踪），进程里判与走入口各几格；
    # (说明, 起点, 命令, 该不该拒)；起点 work 不是 git 仓
    discard_repository = os.path.join(work, "discard-repository")
    subprocess.run(["git", "init", "-q", discard_repository], check=True)
    for name in ("a.txt", "b.txt", "c.txt"):
        with open(os.path.join(discard_repository, name), "w", encoding="utf-8") as handle:
            handle.write(name + "\n")
    subprocess.run(["git", "-C", discard_repository, "add", "a.txt", "b.txt", "c.txt"], check=True)
    subprocess.run(["git", "-C", discard_repository, "-c", "user.email=t@t", "-c", "user.name=t", "-c", "commit.gpgsign=false",
                    "commit", "-q", "-m", "base"], check=True)
    for name, text in (("a.txt", "a 没暂存的改动\n"), ("c.txt", "c 暂存了的改动\n"), ("u.txt", "没跟踪\n")):
        with open(os.path.join(discard_repository, name), "w", encoding="utf-8") as handle:
            handle.write(text)
    subprocess.run(["git", "-C", discard_repository, "add", "c.txt"], check=True)
    discard_cases = [
        ("撤工作区改动:git checkout -- 有改动的路径拒", discard_repository, "git checkout -- a.txt", 1),
        ("撤工作区改动:git checkout HEAD -- 有改动的路径拒", discard_repository, "git checkout HEAD -- a.txt", 1),
        ("撤工作区改动:git checkout 不带 -- 的有改动路径拒", discard_repository, "git checkout a.txt", 1),
        ("撤工作区改动:git restore 有改动的路径拒", discard_repository, "git restore a.txt", 1),
        ("撤工作区改动:git restore --staged --worktree 有改动的路径拒", discard_repository, "git restore --staged --worktree -- a.txt", 1),
        ("撤工作区改动:git stash push -- 有改动的路径拒", discard_repository, "git stash push -- a.txt", 1),
        ("撤工作区改动:git stash push -m 带说明、有改动的路径拒", discard_repository, "git stash push -m keep -- a.txt", 1),
        ("撤工作区改动:git clean -f 点名没跟踪的文件拒", discard_repository, "git clean -f u.txt", 1),
        ("撤工作区改动:有改动的路径与没改动的路径同写拒", discard_repository, "git checkout -- b.txt a.txt", 1),
        ("撤工作区改动:git checkout -- . 整棵树里有改动拒", discard_repository, "git checkout -- .", 1),
        ("撤工作区改动:前面赋过值的变量照展开，有改动拒", discard_repository, 'f=a.txt; git checkout -- "$f"', 1),
        ("撤工作区改动:cd 进仓之后 git checkout -- 有改动的路径拒", work, f"cd {discard_repository} && git checkout -- a.txt", 1),
        ("撤工作区改动:git -C 点名仓、起点在别处，拒", work, f"git -C {discard_repository} restore a.txt", 1),
        ("撤工作区改动:路径写成没赋过值的变量，判不了，拒", discard_repository, 'git checkout -- "$SOMEWHERE_UNSET"', 1),
        ("撤工作区改动:cd 到没赋过值的变量之后，判不了，拒", work, 'cd "$WHERE_UNSET" && git checkout -- a.txt', 1),
        ("撤工作区改动:git checkout -- 没改动的路径放行", discard_repository, "git checkout -- b.txt", 0),
        ("撤工作区改动:git restore 没改动的路径放行", discard_repository, "git restore b.txt", 0),
        ("撤工作区改动:只暂存了、工作区与暂存区相同的路径放行", discard_repository, "git restore c.txt", 0),
        ("撤工作区改动:git stash push -- 没改动的路径放行", discard_repository, "git stash push -- b.txt", 0),
        ("撤工作区改动:git clean -f 点名的路径下没有没跟踪的文件放行", discard_repository, "git clean -f b.txt", 0),
        ("撤工作区改动:不在仓里的路径放行", discard_repository, "git checkout -- not-here.txt", 0),
        ("撤工作区改动:git restore --staged 有改动的路径放行（只动暂存区）", discard_repository, "git restore --staged a.txt", 0),
        ("撤工作区改动:git restore -S 有改动的路径放行（只动暂存区）", discard_repository, "git restore -S -- a.txt", 0),
        ("撤工作区改动:git clean -n 只列不删放行", discard_repository, "git clean -n u.txt", 0),
        ("撤工作区改动:git checkout -b 切新分支放行", discard_repository, "git checkout -b side", 0),
        ("撤工作区改动:grep 参数里的 git checkout 不判", discard_repository, "grep -n 'git checkout -- a.txt' a.txt", 0),
    ]
    for label, start, command, want in discard_cases:
        results.append((label, want, 1 if worktree_discard_refusal(command, start) else 0))
    for agent_type in (None, "clerk"):
        discard_payload = {"tool_name": "Bash", "session_id": "s", "cwd": discard_repository,
                           "tool_input": {"command": "git checkout -- a.txt", "run_in_background": False}}
        if agent_type:
            discard_payload["agent_type"] = agent_type
        discard_entry = subprocess.run(["bash", os.path.join(hook_dir, "bash-command-detector.sh")], capture_output=True, text=True,
                                       env=dict(os.environ, AGENT_HOOK_DETECTIONS=os.path.join(work, "discard-detections.jsonl")), input=json.dumps(discard_payload))
        results.append((f"stdin:{'子 agent' if agent_type else '主 agent'} 带路径撤掉没暂存的改动拒、出路写先留底与 git apply -R（退出码 2）", (2, True),
                        (discard_entry.returncode, "git apply -R" in discard_entry.stderr)))
    # ⑯ 往现行账本直接追加节标题：假仓里放登记表与两份账本（进程里判），再走真实入口一趟（真仓的登记表，只判不写）；(说明, 命令, 该不该拒)
    ledger_repository = os.path.join(work, "ledger-repo")
    os.makedirs(os.path.join(ledger_repository, ".claude", "gate.d"))
    os.makedirs(os.path.join(ledger_repository, "records"))
    with open(os.path.join(ledger_repository, LEDGER_REGISTRY_PATH), "w", encoding="utf-8") as handle:
        handle.write("current\trecords/ledger.md\t-\t样本里的现行账本，自证用\nsealed\trecords/old.md\t" + "0" * 64 + "\t样本里的封存账本，自证用\n")
    for ledger_name in ("ledger.md", "old.md"):
        with open(os.path.join(ledger_repository, "records", ledger_name), "w", encoding="utf-8") as handle:
            handle.write("# 账本\n\n## 一、甲\n")
    new_heading = "## 一百七十九、新节"
    ledger_cases = [
        ("往现行账本追加:cat >> 加 heredoc 拒", f"cat >> records/ledger.md <<'EOF'\n{new_heading}\n\n正文\nEOF", 1),
        ("往现行账本追加:echo >> 拒", f"echo '{new_heading}' >> records/ledger.md", 1),
        ("往现行账本追加:管道接 tee -a 拒", f"printf '\\n{new_heading}\\n' | tee -a records/ledger.md", 1),
        ("往现行账本追加:cd 进 records 之后 >> 拒", f"cd records && echo '{new_heading}' >> ledger.md", 1),
        ("往现行账本追加:绝对路径 >> 拒", f"echo '{new_heading}' >> {ledger_repository}/records/ledger.md", 1),
        ("往现行账本追加:喂给 python 的 heredoc 里 open(…, 'a') 拒", f"python3 - <<'EOF'\nopen('records/ledger.md', 'a').write('{new_heading}')\nEOF", 1),
        ("往现行账本追加:写别的文件放行", f"echo '{new_heading}' >> records/notes.md", 0),
        ("往现行账本追加:封存的账本放行", f"echo '{new_heading}' >> records/old.md", 0),
        ("往现行账本追加:往账本写不带节标题的一行放行", "echo '去向：做完' >> records/ledger.md", 0),
        ("往现行账本追加:只读账本里的节标题放行", f"grep -n '{new_heading}' records/ledger.md", 0),
        ("往现行账本追加:经 ledger-append-section.py 放行", f"python3 scripts/kb/ledger-append-section.py --title '{new_heading}' --body-file draft.txt", 0),
        ("往现行账本追加:sed -i 改节号放行", f"sed -i 's/^## 一、/{new_heading}/' records/ledger.md", 0),
    ]
    for label, command, want in ledger_cases:
        results.append((label, want, 1 if ledger_append_refusal(command, ledger_repository, ledger_repository)[0] else 0))
    real_ledger, _ = current_ledger_relative(repository_root_of(hook_dir))
    for agent_type in (None, "clerk"):
        ledger_payload = {"tool_name": "Bash", "session_id": "s", "cwd": repository_root_of(hook_dir),
                          "tool_input": {"command": f"echo '{new_heading}' >> {real_ledger}", "run_in_background": False}}
        if agent_type:
            ledger_payload["agent_type"] = agent_type
        ledger_entry = subprocess.run(["bash", os.path.join(hook_dir, "bash-command-detector.sh")], capture_output=True, text=True,
                                      env=dict(os.environ, AGENT_HOOK_DETECTIONS=os.path.join(work, "ledger-detections.jsonl")), input=json.dumps(ledger_payload))
        results.append((f"stdin:{'子 agent' if agent_type else '主 agent'} 往现行账本追加节标题拒、出路写经 ledger-append-section.py（退出码 2）", (2, True),
                        (ledger_entry.returncode, "scripts/kb/ledger-append-section.py" in ledger_entry.stderr)))
    # ⑲ 往 records/ 下的 .md 追加：假仓里判几格（进程里），再走真实入口两格（真仓里一份不在的记录名，只判不写）；(说明, 命令, 该不该拒)
    record_repository = os.path.join(work, "record-repo")
    for record_directory in ("records", "scripts", "research"):
        os.makedirs(os.path.join(record_repository, record_directory))
    with open(os.path.join(record_repository, "records", "run.md"), "w", encoding="utf-8") as handle:
        handle.write("# 记录\n")
    os.symlink(os.path.join("..", "records", "run.md"), os.path.join(record_repository, "scripts", "link-in.md"))
    os.symlink(os.path.join(work, "record-outside.md"), os.path.join(record_repository, "records", "link-out.md"))
    record_cases = [
        ("往 records 追加:echo >> 拒", "echo '- 乙' >> records/run.md", 1),
        ("往 records 追加:cat >> 加 heredoc 拒", "cat >> records/run.md <<'EOF'\n- 乙\nEOF", 1),
        ("往 records 追加:&>> 拒", "make &>> records/run.md", 1),
        ("往 records 追加:管道接 tee -a 拒", "printf '%s\\n' '- 乙' | tee -a records/run.md", 1),
        ("往 records 追加:tee --append 拒", "echo x | tee --append records/run.md", 1),
        ("往 records 追加:cd 进 records 之后 >> 拒", "cd records && echo x >> run.md", 1),
        ("往 records 追加:绝对路径 >> 拒", f"echo x >> {record_repository}/records/run.md", 1),
        ("往 records 追加:前面赋过值的变量照代入拒", "R=records/run.md; echo x >> $R", 1),
        ("往 records 追加:bash -c 里的 >> 拒", "bash -c \"echo x >> records/run.md\"", 1),
        ("往 records 追加:python3 -c 里 open(…, 'a') 拒", "python3 -c \"open('records/run.md', 'a').write('x')\"", 1),
        ("往 records 追加:喂给 python 的 heredoc 里 open(…, mode='a') 拒", "python3 - <<'EOF'\nopen('records/run.md', mode='a').write('x')\nEOF", 1),
        ("往 records 追加:python 的 Path.open('a') 拒", "python3 -c \"from pathlib import Path; Path('records/run.md').open('a').write('x')\"", 1),
        ("往 records 追加:子目录里的 .md 也拒", "echo x >> records/sub/notes.md", 1),
        ("往 records 追加:大写后缀 .MD 也拒", "echo x >> records/RUN.MD", 1),
        ("往 records 追加:指进 records 的符号链接照落点拒", "echo x >> scripts/link-in.md", 1),
        ("往 records 追加:经 append-record.py 放行", "python3 scripts/kb/append-record.py records/run.md draft.txt", 0),
        ("往 records 追加:标准输入喂给 append-record.py 放行", "printf '%s\\n' '- 乙' | python3 scripts/kb/append-record.py records/run.md -", 0),
        ("往 records 追加:records 下别的后缀不判", "echo x >> records/table.tsv", 0),
        ("往 records 追加:records 之外的 .md 不判", "echo x >> research/notes.md", 0),
        ("往 records 追加:records 同名前缀的兄弟目录不判", "echo x >> records-old/run.md", 0),
        ("往 records 追加:records 里指到仓外的符号链接照落点不判", "echo x >> records/link-out.md", 0),
        ("往 records 追加:仓外同名路径不判", f"echo x >> {work}/elsewhere/records/run.md", 0),
        ("往 records 追加:> 整份写不是追加，不判", "echo x > records/run.md", 0),
        ("往 records 追加:python open(…, 'w') 不是追加，不判", "python3 -c \"open('records/run.md', 'w').write('x')\"", 0),
        ("往 records 追加:只读不判", "grep -n x records/run.md", 0),
        ("往 records 追加:目标算不出的不判", "echo x >> \"$UNSET_RECORD\"", 0),
    ]
    for label, command, want in record_cases:
        results.append((label, want, 1 if record_append_refusal(command, record_repository, record_repository) else 0))
    howto_not_run = []   # 出路过别的闸那两格没判的（hook 旁边没有 project-code-guard.sh），成功句里报「本次未跑」
    for agent_type in (None, "clerk"):
        record_payload = {"tool_name": "Bash", "session_id": "s", "cwd": repository_root_of(hook_dir),
                          "tool_input": {"command": "echo '- 自证' >> records/bash-command-detector-selftest-record.md", "run_in_background": False}}
        if agent_type:
            record_payload["agent_type"] = agent_type
        record_entry = subprocess.run(["bash", os.path.join(hook_dir, "bash-command-detector.sh")], capture_output=True, text=True,
                                      env=dict(os.environ, AGENT_HOOK_DETECTIONS=os.path.join(work, "record-detections.jsonl")), input=json.dumps(record_payload))
        results.append((f"stdin:{'子 agent' if agent_type else '主 agent'} 往 records 追加拒、出路给 append-record.py 的整条写法（退出码 2）", (2, True),
                        (record_entry.returncode, "python3 scripts/kb/append-record.py records/bash-command-detector-selftest-record.md" in record_entry.stderr)))
        # 出路本身要过得了别的闸：拒绝说明里那条整条写法照抄（文本文件换成草稿里的一份真文件），喂给同挂 PreToolUse[Bash] 的 project-code-guard.sh，要放行
        howto_label = f"出路过别的闸:{'子 agent' if agent_type else '主 agent'} 照抄 ⑲ 的出路喂 project-code-guard.sh 放行（退出码 0）"
        guard_script = os.path.join(hook_dir, "project-code-guard.sh")
        if not os.path.isfile(guard_script):
            howto_not_run.append(f"{howto_label}（hook 旁边没有 project-code-guard.sh）")
        else:
            howto = RECORD_HOWTO_COMMAND.search(record_entry.stderr)
            howto_exit = None   # 拒绝说明里取不出出路也判红
            if howto:
                howto_text = os.path.join(work, "record-howto-text.txt")
                with open(howto_text, "w", encoding="utf-8") as handle:
                    handle.write("- 自证\n")
                guard_payload = {"tool_name": "Bash", "session_id": "s", "cwd": repository_root_of(hook_dir),
                                 "tool_input": {"command": howto.group(1).replace(RECORD_HOWTO_TEXT_PLACEHOLDER, howto_text), "run_in_background": False}}
                if agent_type:
                    guard_payload["agent_type"] = agent_type
                howto_exit = subprocess.run(["bash", guard_script], capture_output=True, text=True, input=json.dumps(guard_payload),
                                            env=howto_guard_environment(os.environ, os.path.join(work, "record-howto-detections.jsonl"))).returncode
            results.append((howto_label, 0, howto_exit))
    # ⑳ 手拷仓副本：(说明, 命令, 该不该拒[, 起点目录])；假仓在草稿里（crates/demo、Cargo.toml），草稿目录另放一份副本的 crates/ 与笔记；起点不写是假仓根
    copy_repository = os.path.join(work, "manual-copy-repository")
    copy_scratch = os.path.join(work, "manual-copy-scratch")
    for base_directory, relative in ((copy_repository, "crates/demo/src/lib.rs"), (copy_repository, "Cargo.toml"),
                                     (copy_scratch, "notes/a.md"), (copy_scratch, "copy/crates/demo/src/lib.rs")):
        os.makedirs(os.path.dirname(os.path.join(base_directory, relative)), exist_ok=True)
        with open(os.path.join(base_directory, relative), "w", encoding="utf-8") as handle:
            handle.write("\n")
    manual_copy_cases = [
        ("rsync 拷 crates 进仓外目录", f"rsync -a --exclude target crates {copy_scratch}/copy/", 1),
        ("cp -r 拷 crates", f"cp -r crates {copy_scratch}/copy/", 1),
        ("rsync 拷仓根整个（./）", f"rsync -a --exclude target --exclude .git ./ {copy_scratch}/copy/", 1),
        ("rsync 拷仓根（绝对路径带结尾的 /）", f"rsync -a --exclude target --exclude .git {copy_repository}/ {copy_scratch}/copy/", 1),
        ("cp -a 拷仓根 .", f"cp -a . {copy_scratch}/copy", 1),
        ("cp -R 拷 crates/（带结尾的 /）", f"cp -R crates/ {copy_scratch}/copy/crates", 1),
        ("cp --archive 拷 crates", f"cp --archive crates {copy_scratch}/copy/", 1),
        ("cp -t 目录写在前面、-r 写在后面", f"cp -t {copy_scratch}/copy -r crates", 1),
        ("rsync 的 --exclude=值 合写", f"rsync -av --exclude=target crates {copy_scratch}/copy/", 1),
        ("rsync -e 带值的短选项之后的源照认", f"rsync -a -e ssh crates {copy_scratch}/copy/", 1),
        ("cd 进仓根之后 rsync crates", f"cd {copy_repository} && rsync -a crates {copy_scratch}/copy/", 1, copy_scratch),
        ("前面赋过值的变量代入", f"R={copy_repository}; rsync -a $R/crates {copy_scratch}/copy/", 1, copy_scratch),
        ("bash -c 里", f"bash -c 'cp -r crates {copy_scratch}/copy/'", 1),
        ("nice 前缀", f"nice -n 19 rsync -a crates {copy_scratch}/copy/", 1),
        ("仓根里的通配 * 盖住整层", f"cp -r * {copy_scratch}/copy/", 1),
        ("crates/* 盖住 crates 整层", f"cp -r crates/* {copy_scratch}/copy/crates/", 1),
        ("crates/.. 规整之后是仓根", f"rsync -a crates/.. {copy_scratch}/copy/", 1),
        ("喂给 bash 的 heredoc 里", f"bash <<'EOF'\nrsync -a crates {copy_scratch}/copy/\nEOF", 1),
        ("rsync 一次给几个源、其中有 crates", f"rsync -a Cargo.toml crates {copy_scratch}/copy/", 1),
        ("minimal-copy.py 本身放行", f"python3 scripts/hosts/minimal-copy.py {copy_repository} {copy_scratch}/copy --with-test-inputs", 0),
        ("拷草稿目录里不含本仓 crates 的东西放行", f"rsync -a {copy_scratch}/notes/ {copy_scratch}/notes-backup/", 0),
        ("拷草稿目录里别的副本的 crates 放行", f"cp -r {copy_scratch}/copy/crates {copy_scratch}/copy-two/", 0),
        ("cp 不带递归放行（目录拷不过去）", f"cp crates {copy_scratch}/x", 0),
        ("rsync -n 只演一遍放行", f"rsync -an crates {copy_scratch}/copy/", 0),
        ("rsync --dry-run 放行", f"rsync --dry-run -a crates {copy_scratch}/copy/", 0),
        ("rsync 只给一个操作数是只列，放行", "rsync -a crates", 0),
        ("拷 crates 下单个包的目录放行", f"cp -r crates/demo {copy_scratch}/copy/crates/", 0),
        ("拷 crates 下单个文件放行", f"cp crates/demo/src/lib.rs {copy_scratch}/lib.rs", 0),
        ("echo 引号里当数据写的放行", f"echo 'rsync -a crates {copy_scratch}/copy/'", 0),
        ("grep 参数里的放行", "grep -n 'cp -r crates' notes.md", 0),
        ("写进文件的 heredoc 正文放行", f"cat > {copy_scratch}/note.md <<'EOF'\nrsync -a crates /x/\nEOF", 0),
        ("源是没赋过值的变量放行（算不出）", 'rsync -a "$MANUAL_COPY_SOURCE_UNSET" /x/', 0),
        ("远端源（主机:路径）放行", "rsync -a host:crates /x/", 0),
        ("同名前缀的兄弟目录放行", f"cp -r crates-old {copy_scratch}/x", 0),
        ("-- 之后的 -r 是文件名、不算递归，放行", "cp -- -r crates", 0),
    ]
    for label, command, want, *start in manual_copy_cases:
        results.append((f"手拷仓副本:{label}", want, 1 if manual_repository_copy_refusal(command, copy_repository, start[0] if start else copy_repository) else 0))
    # 走真实入口（这一份钩子、它所在的仓根）：主 agent 与子 agent 手拷本仓的 crates 各拒一格、拒绝说明给 minimal-copy.py 的整条写法；
    # 出路本身要过得了别的闸：那条整条写法照抄（副本换成草稿里的一个路径）喂给同挂 PreToolUse[Bash] 的 project-code-guard.sh，要放行；起 minimal-copy.py 本身放行
    manual_copy_root = repository_root_of(hook_dir)
    for agent_type, command, label in ((None, f"rsync -a --exclude target crates {copy_scratch}/copy/", "主 agent rsync 拷 crates"),
                                       ("implementation-writer", f"cp -r crates {copy_scratch}/copy/", "子 agent cp -r 拷 crates")):
        manual_payload = {"tool_name": "Bash", "session_id": "s", "cwd": manual_copy_root, "tool_input": {"command": command, "run_in_background": False}}
        if agent_type:
            manual_payload["agent_type"] = agent_type
        manual_entry = subprocess.run(["bash", os.path.join(hook_dir, "bash-command-detector.sh")], capture_output=True, text=True,
                                      env=dict(os.environ, AGENT_HOOK_DETECTIONS=os.path.join(work, "manual-copy-detections.jsonl")), input=json.dumps(manual_payload))
        manual_howto = MANUAL_COPY_HOWTO_COMMAND.search(manual_entry.stderr)
        results.append((f"stdin:{label} 拒、出路给 minimal-copy.py 的整条写法（退出码 2）", (2, True), (manual_entry.returncode, manual_howto is not None)))
        manual_howto_label = f"出路过别的闸:{'子 agent' if agent_type else '主 agent'} 照抄 ⑳ 的出路喂 project-code-guard.sh 放行（退出码 0）"
        manual_guard_script = os.path.join(hook_dir, "project-code-guard.sh")
        if not os.path.isfile(manual_guard_script):
            howto_not_run.append(f"{manual_howto_label}（hook 旁边没有 project-code-guard.sh）")
            continue
        manual_howto_exit = None   # 拒绝说明里取不出出路也判红
        if manual_howto:
            manual_guard_payload = {"tool_name": "Bash", "session_id": "s", "cwd": manual_copy_root,
                                    "tool_input": {"command": manual_howto.group(1).replace(MANUAL_COPY_HOWTO_COPY_PLACEHOLDER, os.path.join(copy_scratch, "howto-copy")),
                                                   "run_in_background": False}}
            if agent_type:
                manual_guard_payload["agent_type"] = agent_type
            manual_howto_exit = subprocess.run(["bash", manual_guard_script], capture_output=True, text=True, input=json.dumps(manual_guard_payload),
                                               env=howto_guard_environment(os.environ, os.path.join(work, "manual-copy-howto-detections.jsonl"))).returncode
        results.append((manual_howto_label, 0, manual_howto_exit))
    manual_green = subprocess.run(["bash", os.path.join(hook_dir, "bash-command-detector.sh")], capture_output=True, text=True,
                                  env=dict(os.environ, AGENT_HOOK_DETECTIONS=os.path.join(work, "manual-copy-detections.jsonl")),
                                  input=json.dumps({"tool_name": "Bash", "session_id": "s", "cwd": manual_copy_root, "tool_input": {
                                      "command": f"python3 scripts/hosts/minimal-copy.py {manual_copy_root} {copy_scratch}/copy --with-test-inputs",
                                      "run_in_background": False}}))
    results.append(("stdin:主 agent 起 minimal-copy.py 建副本放行（退出码 0）", 0, manual_green.returncode))
    sample_environment = howto_guard_environment({"LIB_SHELL_WORDS_BREAK": "x", "PROJECT_CODE_GUARD_BREAK": "y", "PATH": "/sample"}, "/sample/detections.jsonl")
    results.append(("出路过别的闸:喂 project-code-guard.sh 的环境里不带弄坏开关变量，别的变量照带", (False, False, True, True),
                    ("LIB_SHELL_WORDS_BREAK" in sample_environment, "PROJECT_CODE_GUARD_BREAK" in sample_environment, sample_environment.get("PATH") == "/sample",
                     sample_environment.get("AGENT_HOOK_DETECTIONS") == "/sample/detections.jsonl")))
    results.append(("stdin:只记的检出记录写了结局「只记」", "只记", last_outcome))
    after = sum(1 for _ in open(detections)) if os.path.exists(detections) else 0
    # 走真实入口：hook 旁边没有共用切词模块时照常放行（退出码 0，前台起看门狗也不拦），stderr 点名那个模块，并记一条检出
    lonely_hook_directory = os.path.join(work, "hook-without-shared-words")
    os.makedirs(lonely_hook_directory)
    shutil.copy(script, lonely_hook_directory)
    without_library = subprocess.run(["bash", os.path.join(lonely_hook_directory, "bash-command-detector.sh")], capture_output=True, text=True,
                                     env=dict(os.environ, AGENT_HOOK_DETECTIONS=detections),
                                     input=json.dumps({"tool_name": "Bash", "session_id": "s", "tool_input": {"command": "bash scripts/agents/watch.sh a1 > /dev/null 2>&1 &"}}))
    recorded_after = sum(1 for _ in open(detections)) if os.path.exists(detections) else 0
    results.append(("stdin:读不到共用切词模块照常放行（退出码 0）", 0, without_library.returncode))
    results.append(("stdin:读不到共用切词模块时 stderr 点名它、记一条检出", 1,
                    int("lib_shell_words.py" in without_library.stderr and recorded_after == after + 1)))
    failures = [item for item in results if item[1] != item[2]]
    for label, want, got in failures:
        print(f"  ✗ 自检：{label} 应当是 {want}，实际 {got}")  # gate-lint:detail
    if failures:
        print("    → 看 findings_for()、record()、watchdog_rejection()、wait_loop_refusal() / unbounded_wait_loops()、detaching_refusal() / detaching_forms()、"
              "unwaited_ampersand_refusal() / unwaited_background_jobs() / jobs_with_endings()、"
              "results_overwrite_refusal() / results_overwrite_verdict() / tracked_verdicts() / overwrite_steps() / follow_simple_command() / destination_words() 与共用的 lib_shell_words.py；"
              "「覆盖产物:临时 git 仓建得起来」红的是本机的 git 起不来，先看 git init 能不能跑；"
              "BASH_COMMAND_DETECTOR_BREAK 设着的话（项与红的格见文件头）这里本来就该红；"
              "「终止进程:」「扫脚本:」红的看 process_signal_refusal() / ProcessTableOnDemand / termination_findings() / call_findings() / scan_scripts()；"
              "「就地改脚本:」红的看 script_in_place_refusal() / script_in_place_verdict() / python_in_place_writes() / is_existing_script()；"
              "「先编后换:」红的看 compile_first_refusal() / compile_first_verdict() / overwrite_steps(every_write=True)，BASH_COMMAND_DETECTOR_BREAK=allow-compile-first-bypass 设着的话本来就该红；"
              "「通删工作根:」红的看 work_root_removal_refusal() / recursive_removal_operands() / removal_target_path() / removal_hits_work_root()，BASH_COMMAND_DETECTOR_BREAK=allow-work-root-removal 设着的话本来就该红；"
              "「两道包装:」红的看 wrapped_commands() / two_logs_inner_words() / project_code_runs_library() 与 lib_project_code_runs.py 的 command_under_two_logs()，BASH_COMMAND_DETECTOR_BREAK=opaque-two-logs 设着的话本来就该红；"
              "「Bash写范围:」红的看 bash_scope_refusal() / bash_scope_verdict() / bind_loop_variables() / loop_value_choices() / home_expanded() / known_write_script_steps() / agent_scope_patterns() 与 lib_write_scope.py，"
              "mktemp_directory_value() / mktemp_created_directory() / mktemp_option_directory() / mktemp_inside_pwd() / mktemp_template_directory() / own_scratch_directory() 与 lib_writer_lock.py / remember_working_directory() / piped_output_bodies() / command_pieces()，"
              "BASH_COMMAND_DETECTOR_BREAK 的 allow-bash-scope-write、scope-known-scripts-unseen、scope-main-agent-judged、scope-undeterminable-only-noted、scope-loop-values-unknown、scope-loop-landings-trusted、scope-home-unexpanded、scope-landing-lexical-only、heredoc-body-judged-as-commands、"
              "heredoc-pipe-fed-as-data、pwd-variable-unexpanded、mktemp-directory-unknown、mktemp-scratch-template-unknown、mktemp-template-owner-unchecked、mktemp-option-directory-unknown、mktemp-nested-substitution-unknown 设着的话本来就该红；"
              "「平常跑设了弄坏开关」红的看 refuse_break_outside_selftest()，「扫脚本:待改清单里的行所在的目录这一趟没扫」红的看 scan_scripts()，break-honored-outside-selftest、pending-outside-scanned-as-stale 设着的话本来就该红；"
              "「就地改仓内文件:」红的看 repository_in_place_edit_refusal() / sed_target_resolved_in_scratch() / embedded_python_codes() / python_repository_write_pattern()，"
              "sed-in-place-ignores-cd、python-move-into-repository-unseen 设着的话本来就该红；"
              "「起 claude:」与 stdin 里起 claude 那几格红的看 claude_launch_refusal() / claude_launches()，allow-subagent-claude-launch 设着的话本来就该红；"
              "「超时检出:」与 stdin 里 timeout 那几格红的看 subagent_timeout_findings() 与 main() 末尾的 record()，subagent-timeout-unnoted 设着的话本来就该红；"
              "「Bash写范围:」里脚本路径在变量里的那几格红的看 known_write_script_steps() / unresolved_script_word() 与 bash_scope_verdict() 的 UnresolvedScriptTarget 一支，"
              "scope-script-variable-unexpanded 设着的话本来就该红；"
              "「Bash写范围:」里没有项目定义的 agent 那几格红的看 agent_scope_patterns() / bash_scope_refusal() 的 builtin 一支，builtin-agents-unjudged 设着的话本来就该红；"
              "「后台:」与 stdin 里子 agent run_in_background 那几格红的看 subagent_background_refusal()，"
              "allow-subagent-background 设着的话本来就该红；「清整个索引:」与 stdin 里 git reset 那两格红的看 whole_index_reset_refusal() / git_reset_clears_index() / "
              "git_restore_clears_index() / staged_index_state()，allow-whole-index-reset、whole-index-staged-unchecked、whole-index-pathspec-ignored 设着的话本来就该红；"
              "「关守卫项与别的项同写」红的看 refuse_break_outside_selftest()，break-honored-outside-selftest-with-others 设着的话本来就该红；"
              "「往现行账本追加:」与 stdin 里往现行账本追加节标题那两格红的看 ledger_append_refusal() / current_ledger_relative() / lands_on_ledger() / overwrite_steps(every_write=True)，"
              "allow-ledger-append 设着的话本来就该红；"
              "「字面写入:」与 stdin 里 Bash heredoc 往仓里追加那两格红的看 literal_notation_hits() / literal_writes() / literal_output() / literal_write_landing() "
              "与 ../gate.d/lib-forbidden-notations.py，allow-literal-notations、literal-notations-outside-judged 与库的 LIB_FORBIDDEN_NOTATIONS_BREAK 设着的话本来就该红；"
              "「撤工作区改动:」与 stdin 里撤工作区改动那两格红的看 worktree_discard_refusal() / worktree_discard_paths() / unstaged_hits()，"
              "allow-path-discard、path-discard-changes-unchecked、path-discard-staged-counted 设着的话本来就该红；"
              "「Bash写范围:」里 ledger-append-section.py 那几格红的看 known_write_script_steps() / last_option_value() / ledger_append_root() 与 bash_scope_verdict() 的 LedgerAppendTarget 一支，"
              "scope-ledger-append-unseen 设着的话本来就该红；"
              "「Bash写范围:」里 append-record.py 那几格红的看 known_write_script_steps() 的 RECORD_APPEND_SCRIPT 一支与 first_operand()，"
              "scope-record-append-unseen、scope-record-append-root-as-target 设着的话本来就该红；"
              "「往 records 追加:」与 stdin 里往 records 追加那两格红的看 record_append_refusal() / record_append_landing() / overwrite_steps(every_write=True) 的 tee 一支与 python_in_place_writes()，"
              "allow-record-append、record-append-python-unseen、record-append-tee-unseen 设着的话本来就该红；"
              "「手拷仓副本:」与 stdin 里手拷 crates、⑳ 出路过别的闸那几格红的看 manual_repository_copy_refusal() / copied_sources() / rsync_copied_sources() / copy_arguments_recursive() / copied_source_hits() / manual_copy_howto()，allow-manual-repository-copy、manual-copy-howto-unlisted-tool 设着的话本来就该红")
        return 1
    summary = ("没超时的等待循环、run_in_background 里又自己放后台记进检出记录，普通命令、重定向里的 & 、前台的 & 、写进文件的 heredoc 正文与按模式找进程（归上游钩子）不记；"
               "起看门狗不用 run_in_background、或带 &、nohup、setsid、disown、丢进 /dev/null 的拒绝（退出码 2），run_in_background 只写 watch.sh 的、把名字当参数的、"
               "不盯的模式与 agent-watch.py report 放行；前台没超时的等待循环拒绝（退出码 2、不记检出），外层套了 timeout 的、run_in_background 里的（只记检出）、"
               "引号与注释里当数据写的、不带 sleep 的与 for 循环放行；命令位置上的 disown、coproc、setsid -f、nohup … &、tmux / screen 分离、"
               "不等结束的 systemd-run 前台后台都拒，不带 & 的 nohup、裸 setsid、systemd-run --wait、并行加 wait 与当数据写的放行；"
               "run_in_background 里以单独的 & 收尾、之后同一层同一对圆括号里没有 wait 的拒绝（结尾 &、& 后接 echo $!、capped.sh 包着的、"
               "子 shell、命令替换、bash -c 与喂给 shell 的 heredoc 里的，wait -n 与按 pid 轮询不算 wait），之后用 wait / wait \"$pid\" 等的、"
               "&&、2>&1、|&、&>、当数据写的与前台的放行；在临时 git 仓里，整份覆盖 research/results/ 下已存在又没进 git 的文件拒绝（>、>|、&>、2>、>& 文件、"
               "不带 -a 的 tee、cp / mv / install 的目标与 -t、写进已有目录、dd of=（带不带 conv=notrunc 都算）、truncate 的文件（与 ⑦ 共用同一段 overwrite_steps），"
               "bash -c、heredoc、命令替换、进程替换、整组重定向里的，cd 跟着算、cd 算不出来按仓库根、"
               "前面赋过值的变量代入），>>、&>>、tee -a、新文件名、已跟踪、仓外与 research/results/ 外、2>&1、cp -n / --backup、install -d、同一条命令里先 git add 或 mv 挪走的、"
               "当数据写的放行；目标算不出而对得上未跟踪产物的、git 判不了的只记检出；"
               "终止进程：kill 负进程号、0、$PPID、一次几个目标、命令替换与整批展开、循环里逐个 kill 或 proc.py stop、xargs / find -exec 喂给 kill、pkill、killall、fuser -k、"
               "先从 cgroup.procs 或 pgrep 挑再发、python 里 os.killpg 与循环里 os.kill、systemctl 停 SSH / 会话 / user@ / user.slice / 私有配置登记的本地模型服务单元或带通配、isolate 与 reboot、"
               "loginctl terminate / kill、写 cgroup.kill、写死的进程号是 sshd、VSCode、自己的 Claude、别的 Claude 会话或它起的、私有配置登记的本地模型服务、pid 1 的都拒（退出码 2、不记检出），"
               "私有配置不在或没有那两个键时本地模型服务那一项放行、别的照拒，"
               "kill \"$!\"、kill %1、kill -0、kill -l、这个会话起的与没人认领的写死进程号、proc.py stop 一个、自己 slice 的 systemctl stop、只读 cgroup.procs 与当数据写的放行；"
               "自己用 setsid 或 start_new_session=True 起的子进程的进程组放行，不是这样起的、改绑过的照拒；"
               "扫脚本：进程组、循环 kill、没核 scope 的 own-scope 标注、给进程组标的与不起作用的标注、python 循环 os.kill 与 os.killpg 按行号报红，"
               "自己用新会话起的子进程的组不报，参数、属性、改绑过的、没开新会话的、setsid --fork 的、os.getpgid(0)、cgroup.procs 读来的组按行号报红，"
               "核过 singlefs-memory-cap-*.scope 又标了的放行，待改清单里的逐个列名、清单里干净的与不存在的判红；"
               "就地改脚本：在临时仓里，已有的 .sh、.py、带执行位的与 scratch 根下的脚本被 >、>|、&>、2>、>& 文件、cat >、不带 -a 的 tee、cp（-f、-a、写进目录、-t）、"
               "dd of=、truncate、python 的 open(…, 'w' / 'r+' / mode='wb')、Path.open('w')、write_text、write_bytes、shutil.copyfile / copy 写的拒绝"
               "（bash -c、heredoc、命令替换、cd 与变量跟着算，python 的只赋过一次的名字、Path / 、os.path.join 算得出），"
               ">>、&>>、tee -a、python 'a' / 'x' / 读、新文件名、临时文件再 mv、mv、install、cp --remove-destination / -l / -n / -b、sed -i、先 rm 或挪走再写、"
               "os.replace、非脚本文件、仓外与 scratch 根外、当数据写的与算不出的放行；dd、truncate 现在 ⑤ 也认（同一段 overwrite_steps），⑤ 仍不认 python 的写法；"
               "先编后换：experiment-runner 在 Bash 里写主工作区 crates/ 下的 .rs（cp、cat >、>>、tee -a、mv 进目录、cd 之后 cp 到 .、sed -i、perl -pi、rsync、rm、mv 挪走、"
               "bash -c、变量里的目录、算不出而对得上已有装置的、python shutil.copy / open 'a' / os.replace）拒绝并点名 compile-then-swap.py，"
               "经那条脚本换进来、拷进草稿目录、改草稿目录、追加 crates/mutations.tsv、只读、当数据写的与 implementation-writer、主 agent 放行；"
               "通删工作根：rm -r 作用在工作根本身、它的上级或工作根下第一层带通配上的（~、$HOME、$SINGLEFS_WORK_ROOT、export 的变量、cd 之后的 *、bash -c、喂给 bash 的 heredoc、前缀、-fr / -r -f / --recursive / 选项写在后面、会话临时目录根 /tmp/claude-<uid>/*、/tmp/*）拒绝并点名 work-directory-ownership.sh 的 sweep 与 claim，工作根下具体目录与它里面的通配、同名前缀的兄弟目录、别处、不带 -r、当数据写的与算不出的变量放行；"
               "两道包装：scripts/hosts/two-logs.sh <名> -- 包着的前台没超时的等待循环、kill 负进程号、setsid -f、cp 盖入库装置照拒，--from-log、别处的同名拷贝与包着的普通命令放行；"
               "Bash 写范围（⑫）：有写范围的项目子 agent（clerk、kb-scribe）用 replace-once.py / insert-row.py / replace-batch.py（规格里的 file）、sed -i、>、>>、tee、cp / mv、python 的 open(…, 'w') "
               "（bash -c、heredoc、cd、变量、two-logs.sh 包着的都跟着算）写进本仓、又落在 agent-write-scope.tsv 里它那几行之外的拒绝（退出码 2，拒绝说明点名写范围表与交回主 agent 的出路），"
               "范围里的、工作根下的草稿目录、仓外的、--dry-run 与 --selftest、只读、当数据写的、主 agent、没有项目定义的 agent 与表里没有行的定义放行；落点算不出的拒（命令替换、没赋过值或来源不明的变量、while read 与 for $列表 的循环变量、通配、cd 算不出之后的相对路径、读不到规格的 replace-batch.py，静态前缀在仓外也拒；两条检出原文 $(echo $f | tr / _) 拒，拒绝说明点名 copy-originals.py），算得出的照算：for 写死的词逐个取值（取值里爬回仓里的 .. 照判）、前面赋的写死值、cd 之后的 $PWD、mktemp -d、$HOME，落点与写闸同一份判法逐段解开符号链接（范围内的链接指到仓里范围外的照拒，范围外的链接指回范围内的放行，解开之后落在仓外的不判）；"
               "heredoc 正文只在喂给 bash 或 python 时判（cat、tee 写草稿脚本的正文里的 sed -i、重定向、cp、open 是数据，它们的目标照判；⑧ 的 python 判法同样只看 python -c 与喂给 python 的正文；"
               "经管道喂给 python 或 bash 的也是代码：`cat <<'EOF' | python3 -`、`| bash`，echo / printf 的文字同理，喂给 grep 的是数据）；"
               "⑧ 的 sed -i 相对路径从 cd 之后的目录算（cd 到草稿目录放行，cd 到仓内或算不出的照拒）；⑫ 里 cd 到写死的目录之后 $PWD 按那个目录展开，mktemp -d 赋的变量落在 ${TMPDIR:-/tmp} 下（仓外），模板在自己认领的草稿目录里的按草稿目录内算（别的子 agent 的、别的会话的、/tmp 下的、没有 agent_id 的照记）；"
               "进程表只在判到 kill / proc.py stop 的写死的进程号时读、一次（没有终止词的带数字命令不读），一条命令里几个确定的产物目标一次 git ls-files 查完；"
               "⑧ 另认 python 的 os.replace / os.rename / shutil.move 挪进仓里（scripts/ 起头的同算仓内），目标在草稿目录的放行；"
               "⑫ 的脚本词先代入前面赋过的值（脚本路径放在变量里的 replace-once.py / insert-row.py / replace-batch.py 照认），python3 起的脚本词算不出而第一个参数落在仓里的照算不出拒；"
               "⑬ 子 agent 命令位置上起 claude、\"$CLAUDE_CODE_EXECPATH\"、以 /claude 结尾的可执行文件拒绝（bash -c、heredoc、命令替换、前缀里的都认），"
               "command -v claude、当参数与当数据写的、主 agent 放行；子 agent 前台 timeout 超过 240000 毫秒放行并记一条检出，不超过的、主 agent 的、run_in_background 的不记；"
               "⑫ 没有项目定义的 agent（内置类型、定义已删的旧实例）只许写工作根：写仓里、仓外别处拒，写工作根、/dev/null 与只读放行；"
               "⑧ 的 python 相对路径从 cd 之后算：cd 进草稿目录里的仓副本再写放行，仓根下与目录算不出的照拒；"
               "⑮ 暂存区非空时 git reset（不带路径，HEAD、--mixed、--hard）、git restore --staged . / :/、git -C 与 cd 进去的、目录算不出的拒（主 agent 与子 agent 一样），"
               "带具体路径的、--soft、暂存区为空的、当参数写的放行；关守卫项与别的项同写平常跑与带标记的子进程都拒并点名它；"
               "⑯ 命令里有「## <节号>、」又往拆分提案现行账本写（>、>>、tee -a、cd 之后的相对路径、绝对路径、喂给 python 的 heredoc 里 open(…, 'a')）拒（主 agent 与子 agent 一样），"
               "写别的文件、封存的账本、不带节标题的一行、只读、经 ledger-append-section.py、sed -i 改节号放行；"
               "⑰ heredoc、printf、echo 经管道交 tee -a、cd 之后的相对路径、bash -c、here-string 往假仓里写会话名、钟点、批次代号拒，"
               "干净内容、两位会话短号、仓外、排除前缀下、.py 里的批次代号、目标算不出的、cat 拷文件放行，走入口拒一格放一格；"
               "⑱ 带路径的 git checkout [<提交>] [--]、git restore（不带 --staged 或带 --worktree）、git stash push、git clean 点名的路径里有没暂存的改动（clean 是没跟踪的文件）拒（主 agent 与子 agent 一样），cd、git -C、前面赋过值的变量跟着算，路径或目录算不出的拒；没改动的、只暂存了的、不在仓里的路径、--staged、git clean -n、切分支、当参数写的放行；"
               "⑫ 另认 ledger-append-section.py 写的登记表 current 那一份账本：写范围里没有 records/ 的子 agent 起它（python3 起的、直接执行的）拒，--root 算不出的拒，"
               "写范围有 records/ 的、主 agent、--dry-run、--root= 指到仓外副本的放行；"
               "⑫ 另认 append-record.py 的记录：写范围里没有 records/ 的子 agent 起它（python3 起的、直接执行的、cd 之后相对路径的、--root 写在前面的）拒，记录算不出的拒，"
               "写范围有 records/ 的、主 agent、--dry-run、--selftest、记录在仓外的放行；"
               "⑲ 往假仓 records/ 下的 .md 追加（>>、&>>、tee -a、tee --append、cd 之后的相对路径、绝对路径、前面赋过值的变量、bash -c、python 的 open(…, 'a') 与 Path.open('a')、"
               "子目录、大写后缀、指进 records 的符号链接）拒，经 append-record.py、records 下别的后缀、records 外与仓外、同名前缀的兄弟目录、指到仓外的链接、"
               "> 整份写、python 'w'、只读、目标算不出的放行，走入口主 agent 与子 agent 各拒一格并给出整条写法，"
               "那条写法照抄喂给 project-code-guard.sh 主 agent 与子 agent 都放行（hook 旁边没有它时报本次未跑）；"
               "⑳ rsync、cp -r / -R / -a / --archive 拷假仓的 crates/ 或仓根整个（./、.、绝对路径带结尾的 /、crates/、crates/..、仓根里的 *、crates/*、-t 写在前面、--exclude 带值、-e 带值、cd 之后、前面赋过值的变量、bash -c、nice 前缀、喂给 bash 的 heredoc、几个源里有 crates）拒，minimal-copy.py 本身、拷草稿目录里别的东西、cp 不带递归、rsync -n / --dry-run / 只给一个操作数、拷 crates/ 下单个包或单个文件、当数据写的、算不出的变量、远端源、同名前缀的兄弟目录、-- 之后的 -r 放行，走入口主 agent 与子 agent 各拒一格并给出整条写法、那条写法照抄喂给 project-code-guard.sh 两边都放行，起 minimal-copy.py 本身走入口放行")
    not_run_note = f"；本次未跑 {len(howto_not_run)} 格：{'、'.join(howto_not_run)}" if howto_not_run else ""
    print(f"  ✓ 自检通过（查了 {len(results)} 种）：{summary}{not_run_note}")
    return 0

def main():
    hook_dir = sys.argv[1]
    refused = refuse_break_outside_selftest()
    if refused:
        return refused
    if len(sys.argv) > 2 and sys.argv[2] == "--selftest":
        return selftest(hook_dir)
    if len(sys.argv) > 2 and sys.argv[2] == "--scan-scripts":
        if shell_words is None:
            print(f"  ✗ 读不到共用切词模块 {shell_words_library_path(hook_dir)}（{shell_words_error!r}），脚本没扫")
            print("     → 怎么办：恢复 .claude/hooks/lib_shell_words.py（切词与认命令位置只有那一份），再跑门禁 code-tooling 的 research-gate-lint 格")
            return 1
        if len(sys.argv) < 5:
            print("  ✗ --scan-scripts 要仓根与至少一个目录")
            print("     → 怎么办：写成 bash .claude/hooks/bash-command-detector.sh --scan-scripts <仓根> <目录>…（门禁 code-tooling 的 research-gate-lint 格这样调）")
            return 2
        return scan_scripts(sys.argv[3], sys.argv[4:])
    hook_input, unreadable = parsed_hook_input(sys.stdin.buffer.read())
    if unreadable:   # 不是 JSON 对象：照常执行，stderr 打一行可定位的诊断、记一条检出（会话号读不到，照实写读不到）
        print(f"  ! {unreadable}", file=sys.stderr)
        if not detector_break_is_set("failure-unrecorded"):
            try:
                append_detection(UNREADABLE_HOOK_INPUT, [unreadable], os.environ.get("AGENT_HOOK_DETECTIONS") or DEFAULT_DETECTIONS)
            except Exception as error:
                print(f"  ! bash-command-detector.sh 没把这一条写进检出记录（{error!r}）", file=sys.stderr)
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
    # 拒绝的那几类都判完再报：每类两行（✗ 原因、→ 怎么办），按下面的次序收进来，一类都没有才记检出、放行
    refusal_lines = []
    if subagent_background_refusal(hook_input.get("agent_type"), tool_input.get("run_in_background")):
        refusal_lines.append(f"  ✗ {hook_input.get('agent_type')} 用 run_in_background 起命令：子 agent 一律不起后台任务，task wait 也不起"
              "（用户 2026-11-01 定子 agent 一律不往统一任务服务交任务，records/2026-10-29-subagent拆分提案.md 第一百七十四节）")
        refusal_lines.append("     → 怎么办：这一条不起。跑项目代码的命令照共用约束 .claude/agent-common.md「不做」一节「子 agent 不交任务」那一条拼成 submit 的整条"
              "（范围声明写在 submit 前面），写进报告「要主 agent 跑的命令」一节交回，标它在 .claude/hooks/selftest-registry.tsv 登记的档；"
              "停在能交接的点，结束本轮交回主 agent，由主 agent 交服务、等结局、新派同类 agent 接着做。只读的短命令照常前台跑")
    noted_overwrites = []
    try:
        reasons, calls = watchdog_rejection(tool_input.get("command") or "", tool_input.get("run_in_background"))
    except Exception as error:
        not_judged(f"bash-command-detector.sh 没判成看门狗的起法（{error!r}），这条命令照常执行")
        reasons, calls = [], []
    try:
        signal_problems = process_signal_refusal(tool_input.get("command") or "")
    except Exception as error:
        not_judged(f"bash-command-detector.sh 没判成终止进程的写法（{error!r}），这条命令照常执行")
        signal_problems = []
    if signal_problems:
        refusal_lines.append(f"  ✗ 终止进程的写法打得到不是自己点名的那一个进程（{'；'.join(signal_problems)}）："
              "后面的命令不许停掉前面的任务，不许动 SSH、VSCode 与 Claude 会话，一次只停点名的一个")
        refusal_lines.append("     → 怎么办：只停自己起的进程：记下它的 pid（`$!`）或任务号，一次停一个，单独 `python3 .claude/singlefs-ai-sop/scripts/proc.py stop <pid>`"
              "（或 `kill \"$pid\"`、`kill %1`）；不带负号、不给 0、不一次给几个、不在循环里、不按名字或 cgroup 挑。"
              "要停的不是自己起的（别的会话的、SSH、VSCode、系统服务），别停，交主 agent 问用户。这一道只拒这种写法，不停你在跑的任何东西")
    try:
        forms = detaching_refusal(tool_input.get("command") or "")
    except Exception as error:
        not_judged(f"bash-command-detector.sh 没判成放出追踪的写法（{error!r}），这条命令照常执行")
        forms = []
    if forms:
        refusal_lines.append(f"  ✗ 把活放出了追踪（认出的写法：{'、'.join(forms)}）：进程移出 shell 的作业表或另起会话，随后的 wait 立刻返回、"
              "完成通知当场发出，它跑完不会叫醒任何人，成了没人追踪的孤儿")
        refusal_lines.append("     → 怎么办：要等的活用 Bash 的 run_in_background: true 起，命令里只写那条活本身（不加 `disown`、`coproc`、`setsid -f`、"
              "`nohup … &`、`tmux` / `screen` 的分离模式、`systemd-run`），然后结束本轮，等它的完成通知再接着做；"
              "几条活要并行就在同一条命令里每件把退出码写进自己的文件（`{ <命令>; echo \"$?\" > <草稿目录>/b<批号>-<件号>.rc; } &`，"
              "每批开跑前先删掉这一批的 .rc），最后单独一个不带参数的 `wait` 等齐，数一遍 .rc 与派出去的件数对不上整批作废，"
              "对得上再按派活的次序逐个读（共用约束 .claude/agent-common.md「执行前拒绝的写法」那一条的 ④）；"
              "跑得过后台时限的长活由主 agent 经 `task-server/task submit -- <命令…>` 交给服务（起成脱离会话的单元，会话被停也不停），"
              "会话里只用 run_in_background 起 `task-server/task wait <号>` 等它；子 agent 不交，写进报告「要主 agent 跑的命令」一节交回")
    given_directory = hook_input.get("cwd")
    try:
        claude_calls = claude_launch_refusal(tool_input.get("command") or "", hook_input.get("agent_type"),
                                             given_directory if isinstance(given_directory, str) and os.path.isdir(given_directory) else repository_root_of(hook_dir))
    except Exception as error:
        not_judged(f"bash-command-detector.sh 没判成子 agent 在 Bash 里起 claude 会话的写法（{error!r}），这条命令照常执行")
        claude_calls = []
    if claude_calls:
        refusal_lines.append(f"  ✗ {hook_input.get('agent_type')} 在 Bash 里起 claude 会话（认出的调用：{'；'.join(claude_calls)}）："
              "子 agent 不派 subagent，也不从 Bash 里起 claude 会话（共用约束 .claude/agent-common.md「派发」一节），这样起的会话不受派发闸与写范围表管")
        refusal_lines.append("     → 怎么办：不起这条命令；要另一个 agent 做的活（连同要它读什么、产出写到哪）写进交回报告，由主 agent 派。"
              "只查 claude 在不在的 `command -v claude` 不拦。这一道只拒这种写法，不停你在跑的任何东西")
    try:
        jobs = unwaited_ampersand_refusal(tool_input.get("command") or "", tool_input.get("run_in_background"))
    except Exception as error:
        not_judged(f"bash-command-detector.sh 没判成 run_in_background 里的单独 &（{error!r}），这条命令照常执行")
        jobs = []
    if jobs:
        refusal_lines.append(f"  ✗ run_in_background 里又把活放到了后台（以单独的 & 收尾、之后同一条命令里没有 wait 的作业：{'；'.join(jobs)}）："
              "外层 shell 起完它就退出，完成通知当场发出，真跑完的那个进程不会叫醒任何人")
        refusal_lines.append("     → 怎么办：长活直接放 run_in_background，命令里只写那条活本身，不加 `&`"
              "；几条活要并行，就每件把退出码写进自己的文件（`{ <命令>; echo \"$?\" > <草稿目录>/b<批号>-<件号>.rc; } &`，"
              "每批开跑前先删掉这一批的 .rc），最后单独一个不带参数的 `wait` 等齐，数一遍 .rc 与派出去的件数对不上整批作废，"
              "对得上再按派活的次序逐个读；不用 `wait \"$pid\"` 收，也不写 `a & b & wait`：它吞掉每件的退出码"
              "（共用约束 .claude/agent-common.md「执行前拒绝的写法」那一条的 ④）；"
              "已经在跑、pid 已知的，另起一条 run_in_background 等它：`python3 .claude/singlefs-ai-sop/scripts/proc.py wait <pid> --timeout <秒>`。"
              "这一道只拒这种写法，不停你在跑的任何东西")
    try:
        loops = wait_loop_refusal(tool_input.get("command") or "", tool_input.get("run_in_background"))
    except Exception as error:
        not_judged(f"bash-command-detector.sh 没判成前台的等待循环（{error!r}），这条命令照常执行")
        loops = []
    if loops and tool_input.get("run_in_background") is True:
        refusal_lines.append(f"  ✗ run_in_background 里轮询后台任务的输出文件（认出的循环：{'；'.join(loops)}）：等自己起的后台任务不写轮询")
        refusal_lines.append("     → 怎么办：结束本轮，等那个后台任务的完成通知再接着做（共用约束 .claude/agent-common.md「长活交回主 agent」）；"
              "等的是别人起的进程，按进程号等：`python3 .claude/singlefs-ai-sop/scripts/proc.py wait <pid> --timeout <秒>`。"
              "这一道只拒这种写法，不停你在跑的任何东西")
    elif loops:
        refusal_lines.append(f"  ✗ 前台的等待循环没有超时（认出的循环：{'；'.join(loops)}）：等的条件不成立就一直不返回，"
              "这一次调用卡在这里期间，发给你的消息也送不到")
        refusal_lines.append("     → 怎么办，二选一：① 要等的活用 Bash 的 run_in_background: true 起，然后结束本轮，等它的完成通知再接着做；"
              "② 按进程号等：`python3 .claude/singlefs-ai-sop/scripts/proc.py wait <pid> --timeout <秒>`（到点还没退出就列出还活着的、退出码 3）。"
              "确实要在前台轮询一个文件，外面套一层 timeout：`timeout <秒> bash -c 'until …; do sleep …; done'`。"
              + ("" if detector_break_is_set("wait-loop-rewrite-unexplained") else
                 "带计数上限、break、`while read`、`kill -0` 探活这类真会结束的循环，这道闸照字面分不出会不会停（正推腿乙报告第五节 Z10），同样照拒：外面套 timeout 就放行，"
                 "等进程退出的改成 `timeout <秒> tail --pid=<pid> -f /dev/null` 或上面的 proc.py wait。")
              + "这一道只拒这种写法，不停你在跑的任何东西")
    try:
        overwrites, noted_overwrites = results_overwrite_refusal(tool_input.get("command") or "", repository_root_of(hook_dir))
    except Exception as error:
        not_judged(f"bash-command-detector.sh 没判成整份覆盖 research/results/ 产物的写法（{error!r}），这条命令照常执行")
        overwrites, noted_overwrites = [], []
    if overwrites:
        refusal_lines.append(f"  ✗ 要整份覆盖 research/results/ 下已存在、又没进 git 的产物（认出的写法与目标：{'；'.join(overwrites)}）："
              "旧字节没有任何一份副本，盖掉就找不回来")
        refusal_lines.append("     → 怎么办：按日期另存一个新文件名，旧的留着；确实要换掉旧产物，先 `git add` 它或者把它挪到带日期的旧名"
              "（`git add` 或 `mv 旧名 带日期的旧名` 可以写在同一条命令里、排在写之前）。"
              "追加用 `>>` 或 `tee -a`，不拦。这一道只拒这种写法，不停你在跑的任何东西")
    try:
        in_place_writes = script_in_place_refusal(tool_input.get("command") or "", repository_root_of(hook_dir))
    except Exception as error:
        not_judged(f"bash-command-detector.sh 没判成在原 inode 上改脚本的写法（{error!r}），这条命令照常执行")
        in_place_writes = []
    if in_place_writes:
        refusal_lines.append(f"  ✗ 要在同一个 inode 上改一个已经存在的脚本（认出的写法与目标：{'；'.join(in_place_writes)}）："
              "正在跑它的 bash 按文件偏移往下读，改完会从新内容的同一偏移接着读，读到的是别的东西")
        refusal_lines.append("     → 怎么办：写到同目录临时文件再 `mv` 换上（`mv` 换 inode，正在跑它的进程读的还是旧内容），"
              "或者用 `scripts/kb/replace-once.py` / `insert-row.py` 定点改。追加（`>>`、`tee -a`）与新建不拦；"
              "Edit / Write 工具本来就换 inode。这一道只拒这种写法，不停你在跑的任何东西")
    try:
        crate_directory = hook_input.get("cwd")
        crate_writes = compile_first_refusal(tool_input.get("command") or "", hook_input.get("agent_type"), repository_root_of(hook_dir),
                                             crate_directory if isinstance(crate_directory, str) and os.path.isdir(crate_directory) else None)
    except Exception as error:
        not_judged(f"bash-command-detector.sh 没判成绕过先编后换的写法（{error!r}），这条命令照常执行")
        crate_writes = []
    if crate_writes:
        refusal_lines.append(f"  ✗ {hook_input.get('agent_type')} 在 Bash 里写主工作区 crates/ 下的 .rs（认出的写法与目标：{'；'.join(crate_writes)}）："
              "入库装置先在草稿目录的副本里改，编过再整份换进主工作区，主工作区里任何时候都只放编得过的版本")
        swap_launch = (f"跑 `bash scripts/hosts/capped.sh <线程上限> python3 {COMPILE_THEN_SWAP_SCRIPT} " if detector_break_is_set("compile-first-capped-hint")
                       else f"由主 agent 交 `task-server/task submit -- python3 {COMPILE_THEN_SWAP_SCRIPT} ")
        refusal_lines.append(f"     → 怎么办：在草稿目录那一份上改完，{swap_launch}"
              "<草稿副本> crates/<crate>/src/bin/<名>.rs --scratch <草稿目录>`，它在仓副本里编过才整份换进来，编不过一个字节不写；子 agent 不往统一任务服务交任务，"
              "这一条写进报告「要主 agent 跑的命令」一节交回、由主 agent 交；"
              "把主工作区那份拷进草稿目录（`cp crates/… <草稿目录>/`）不拦。目标不是 crates/<crate>/src/bin/ 下的装置的，不归你改，"
              "写进报告交回主 agent。这一道只拒这种写法，不停你在跑的任何东西")
    try:
        edit_directory = hook_input.get("cwd")
        repository_edits = repository_in_place_edit_refusal(tool_input.get("command") or "", hook_input.get("agent_type"), repository_root_of(hook_dir),
                                                            edit_directory if isinstance(edit_directory, str) and os.path.isdir(edit_directory) else None)
    except Exception as error:
        not_judged(f"bash-command-detector.sh 没判成 Bash 里就地改仓内文件的写法（{error!r}），这条命令照常执行")
        repository_edits = []
    if repository_edits:
        refusal_lines.append(f"  ✗ {hook_input.get('agent_type')} 有 Edit 工具，却在 Bash 里就地改仓内文件（认出的写法与目标：{'；'.join(repository_edits)}）："
              "写范围闸只看 Write / Edit，这样写它看不见")
        refusal_lines.append("     → 怎么办：仓内文件用 Edit 改（旧串在文件里恰好命中一次），或用定义点名的脚本（replace-once.py、insert-row.py 这类）；"
              "草稿目录（工作根 ~/.local/share/singlefs/work/ 下）里随便写。这一道只拒这种写法，不停你在跑的任何东西")
    try:
        scope_directory = hook_input.get("cwd")
        scope_verdict = bash_scope_refusal(
            tool_input.get("command") or "", hook_input.get("agent_type"), repository_root_of(hook_dir),
            scope_directory if isinstance(scope_directory, str) and os.path.isdir(scope_directory) else repository_root_of(hook_dir),
            os.path.join(hook_dir, WRITE_SCOPE_TABLE_NAME),
            {key: hook_input.get(key) for key in ("agent_id", "session_id", "transcript_path")})
    except Exception as error:
        not_judged(f"bash-command-detector.sh 没判成 Bash 里写写范围之外的仓内文件（{error!r}），这条命令照常执行")
        scope_verdict = ScopeVerdict([], [], [], [])
    if scope_verdict.out_of_scope and scope_verdict.builtin:
        refusal_lines.append(f"  ✗ {hook_input.get('agent_type')} 没有项目定义（.claude/agents/{hook_input.get('agent_type')}.md 不在：内置类型、插件带来的或定义已删的旧实例），"
              f"只许写工作根，在 Bash 里写到了工作根之外（认出的写法与目标：{'；'.join(scope_verdict.out_of_scope)}）：与写范围闸 write-guard.sh 第二道同一口径")
        refusal_lines.append(f"     → 怎么办：草稿、报告写进派发提示给的草稿目录（工作根 {work_roots.work_root() if work_roots else '~/.local/share/singlefs/work'}/ 下；会话临时目录根 /tmp/claude-<uid>/ 不算）；"
              "仓里与别处的改动不自己做，写进报告交回主 agent，由主 agent 改或派有写范围的项目 agent（一次性的活派 clerk）。"
              "2>/dev/null 这类设备不拦。这一道只拒这种写法，不停你在跑的任何东西")
    elif scope_verdict.out_of_scope:
        refusal_lines.append(f"  ✗ {hook_input.get('agent_type')} 在 Bash 里写了它写范围之外的仓内文件（认出的写法与目标：{'；'.join(scope_verdict.out_of_scope)}）："
              "写范围闸 write-guard.sh 只看 Write / Edit，Bash 里的写由这一道照同一张表（.claude/hooks/agent-write-scope.tsv）判")
        refusal_lines.append(f"     → 怎么办：范围外的改动不自己做，写进报告交回主 agent，由主 agent 改或派该写的 agent；它的写范围：{'、'.join(scope_verdict.patterns)}"
              "（$SINGLEFS_WORK_ROOT 是工作根 ~/.local/share/singlefs/work；会话临时目录根 /tmp/claude-<uid> 不在里面）；草稿目录里随便写。"
              "这一道只拒这种写法，不停你在跑的任何东西")
    if scope_verdict.undeterminable:
        refusal_lines.append(f"  ✗ {hook_input.get('agent_type')} 在 Bash 里写的目标这一刻算不出最终落点（{'；'.join(scope_verdict.undeterminable)}）："
              "落点算不出就判不了在不在写范围里（变量的值可以带 `/..` 爬回仓里），这一道照 .claude/hooks/agent-write-scope.tsv 判之前先要算得出落点")
        refusal_lines.append("     → 怎么办：改仓内文件之前留底原文件，用 `python3 scripts/agents/copy-originals.py copy <草稿目录> <仓内文件>…`"
              "（拷到 <草稿目录>/originals/<相对仓根的路径>、记 sha256 清单；之后 `python3 scripts/agents/copy-originals.py check <草稿目录>` 列出改过的），"
              "不手拼 `$(echo $f | tr / _)` 这类目标；按文件写多份输出的，循环写成 `for 名字 in 写死的词…; do … > <草稿目录>/<写死的段>/$名字.out; done`，"
              "目标只用这些循环变量、同一条命令里前面赋过写死值的变量与写死的段，或逐条写字面路径；别的算不出的写法（命令替换、没赋过值或来源不明的变量、"
              "`while read` 读进来的、`for f in $列表`、通配、cd 到算不出的目录之后的相对路径、读不到规格的 replace-batch.py）改成字面路径。"
              "算得出的照算：cd 之后的 $PWD、mktemp -d 赋的变量、$HOME。这一道只拒这种写法，不停你在跑的任何东西")
    given_cwd = hook_input.get("cwd")
    command_directory = given_cwd if isinstance(given_cwd, str) and os.path.isdir(given_cwd) else repository_root_of(hook_dir)
    try:
        notation_hits = literal_notation_hits(tool_input.get("command") or "", repository_root_of(hook_dir), command_directory,
                                              notations_library_path(hook_dir))
        notation_remedies = literal_notation_remedies(notation_hits, notations_library_path(hook_dir)) if notation_hits else []
    except Exception as error:
        not_judged(f"bash-command-detector.sh 没判成往仓里写的字面内容里的会话号、钟点与批次代号（{error!r}），这条命令照常执行")
        notation_hits, notation_remedies = [], []
    if notation_hits:
        shown = [f"{form} {relative} 第 {number} 行{form_name}「{matched}」" for form, relative, form_name, number, matched in notation_hits[:LITERAL_NOTATION_LISTED]]
        if len(notation_hits) > LITERAL_NOTATION_LISTED:
            shown.append(f"另有 {len(notation_hits) - LITERAL_NOTATION_LISTED} 处没列，改完上面这些再起会列出来")
        refusal_lines.append(f"  ✗ 往仓里的文件写的字面内容里有会话号、钟点或批次代号（认出的写法与目标：{'；'.join(shown)}）："
              "写闸 write-guard.sh 只看 Write / Edit，Bash 里 heredoc、echo、printf、tee 往仓里写的字面内容由这一道照同一份判据库（.claude/gate.d/lib-forbidden-notations.py）判")
        refusal_lines.append("     → 怎么办：" + "；".join(notation_remedies)
              + "。改完用 Edit 写，或用 `python3 scripts/kb/insert-row.py` / `python3 scripts/kb/replace-once.py` 定点写（它们写之前也判会话号、钟点与批次代号）；"
              "命中的其实不是这几样的，改那份判据库的上下文规则或排除前缀。这一道只拒这种写法，不停你在跑的任何东西")
    try:
        index_resets = whole_index_reset_refusal(tool_input.get("command") or "", command_directory)
    except Exception as error:
        not_judged(f"bash-command-detector.sh 没判成清整个索引的 git 写法（{error!r}），这条命令照常执行")
        index_resets = []
    if index_resets:
        refusal_lines.append(f"  ✗ 暂存区里有东西，这条命令不带具体路径清整个索引（认出的写法：{'；'.join(index_resets)}）："
              "暂存区里可能有别的会话暂存的改动，整份清掉就是替它们撤了暂存")
        refusal_lines.append("     → 怎么办：只清自己的路径：`git restore --staged -- <路径…>`；先 `git diff --cached --name-only` 看暂存区里有哪些、哪些是自己的。"
              "这一道只拒这种写法，不停你在跑的任何东西")
    try:
        discards = worktree_discard_refusal(tool_input.get("command") or "", command_directory)
    except Exception as error:
        not_judged(f"bash-command-detector.sh 没判成带路径撤掉工作区改动的 git 写法（{error!r}），这条命令照常执行")
        discards = []
    if discards:
        refusal_lines.append(f"  ✗ 这条命令要撤掉工作区里还没暂存的改动（认出的写法：{'；'.join(discards)}）："
              "git checkout / git restore 拿暂存区或提交盖掉点名的路径，git stash push 把它们收走，git clean 删掉没跟踪的文件；"
              "主工作区几个会话共用，这几处可能是别的会话还没提交的改动，撤了就找不回来")
        refusal_lines.append("     → 怎么办：先 cp 一份留底（`python3 scripts/agents/copy-originals.py copy <草稿目录> <路径…>`）；"
              "只撤自己那几块：把自己的改动存成补丁（`git diff -- <路径> > <草稿目录>/own.diff`，删掉不是自己的块），再 `git apply -R <草稿目录>/own.diff`，别人的块照留；"
              "只动暂存区的 `git restore --staged -- <路径…>` 不拦。这一道只拒这种写法，不停你在跑的任何东西")
    try:
        ledger_writes, noted_ledger = ledger_append_refusal(tool_input.get("command") or "", repository_root_of(hook_dir), command_directory)
    except Exception as error:
        not_judged(f"bash-command-detector.sh 没判成往现行账本直接追加节标题的写法（{error!r}），这条命令照常执行")
        ledger_writes, noted_ledger = [], []
    if ledger_writes:
        refusal_lines.append(f"  ✗ 要往现行账本直接追加「## <节号>、」标题（认出的写法与目标：{'；'.join(ledger_writes)}）："
              "几个会话同时追加，各自读文件、各自取号，节号会撞，到提交时门禁 doc-process-records 的 ledger-section-numbers 格才判出重号")
        refusal_lines.append("     → 怎么办：新节经 `python3 scripts/kb/ledger-append-section.py --title '<标题（含日期）>' --body-file <草稿目录里写好的正文文件>`：现取最大节号加一，"
              "一次写入索引表末行与末尾新节，写前复核 sha256，撞了拒（重跑取下一个号）；索引行的判定、历史行、当日还开着的几行用 --judgement、--history-rows、--open-rows 给。"
              "改已有节的节号或正文用 `scripts/kb/replace-once.py` 定点改。这一道只拒这种写法，不停你在跑的任何东西")
    try:
        record_appends = record_append_refusal(tool_input.get("command") or "", repository_root_of(hook_dir), command_directory)
    except Exception as error:
        not_judged(f"bash-command-detector.sh 没判成往 records/ 下的 .md 追加的写法（{error!r}），这条命令照常执行")
        record_appends = []
    if record_appends:
        shown_appends = "；".join(f"{form} {relative}" for form, relative in record_appends)
        refusal_lines.append(f"  ✗ 要在 Bash 里往 records/ 下的 .md 追加（认出的写法与目标：{shown_appends}）："
              "这样追加的文本不过写闸 write-guard.sh 的钟点、会话号与批次代号判定，要等提交时门禁 doc-text 才判出来")
        refusal_lines.append(f"     → 怎么办：追加的文本先写进草稿目录里的文件，再 `python3 {record_howto_tool()} {record_appends[0][1]} {RECORD_HOWTO_TEXT_PLACEHOLDER}`"
              "（文本从标准输入读写 `-`）：写之前照门禁 doc-text 的 clock-times、session-names 与批次代号同一份判据判这段文本，命中就拒、点名第几行；"
              "读时记 sha256、换上之前复核、改名换上。它不收一、二级标题：新开一节用 Edit 接在文件末尾（写闸照判同几样），拆分提案现行账本加节用 ledger-append-section.py。"
              "这一道只拒这种写法，不停你在跑的任何东西")
    try:
        removal_directory = hook_input.get("cwd")
        removals = work_root_removal_refusal(tool_input.get("command") or "",
                                             removal_directory if isinstance(removal_directory, str) and os.path.isdir(removal_directory)
                                             else repository_root_of(hook_dir))
    except Exception as error:
        not_judged(f"bash-command-detector.sh 没判成 rm -r 通删工作根的写法（{error!r}），这条命令照常执行")
        removals = []
    if removals:
        refusal_lines.append(f"  ✗ rm -r 要删工作根本身、它的上级或工作根下第一层的通配（认出的写法：{'；'.join(removals)}）："
              "工作根下的目录分属不同会话，按名字通删会连别的会话正在用的副本、报告与只存在仓外的跑前登记一起删掉，删了救不回来")
        refusal_lines.append(f"     → 怎么办：清本会话自己的目录，先 `bash {WORK_ROOT_SWEEP_SCRIPT} sweep --dry-run` 看会删哪些，再 `bash {WORK_ROOT_SWEEP_SCRIPT} sweep`"
              "（只删归属是本会话、没有进程开着的，删前逐项连大小写进 <工作根>/sweep-<日期>.log；别的会话的、没有归属文件的列出来交用户定）；"
              f"在工作根下新建目录时 `bash {WORK_ROOT_SWEEP_SCRIPT} claim <目录> --purpose <一句>` 写归属文件。"
              "删一个具体目录直接写它的路径（`rm -rf <工作根>/<目录>`，不带通配）。这一道只拒这种写法，不停你在跑的任何东西")
    try:
        copy_directory = hook_input.get("cwd")
        manual_copies = manual_repository_copy_refusal(tool_input.get("command") or "", repository_root_of(hook_dir),
                                                       copy_directory if isinstance(copy_directory, str) and os.path.isdir(copy_directory)
                                                       else repository_root_of(hook_dir))
    except Exception as error:
        not_judged(f"bash-command-detector.sh 没判成手拷仓副本的写法（{error!r}），这条命令照常执行")
        manual_copies = []
    if manual_copies:
        refusal_lines.append(f"  ✗ 要手拷仓副本（认出的写法：{'；'.join(manual_copies)}）：手列的拷贝范围跟不上编译与测试要读的仓根条目"
              "（path 依赖指到的目录、include_str! 与测试读的仓根文件），拷出来的副本缺东西，到 cargo 编不过、测试读不到才知道")
        refusal_lines.append(f"     → 怎么办：仓副本只用 `{manual_copy_howto(repository_root_of(hook_dir))}` 建（拷哪些由 scripts/hosts/lib_repository_copy.py 的闭包现算："
              "path 依赖、编译期与运行期读的仓根文件、测试输入；直接跑、不经服务，副本写进自己的草稿目录），要门禁表的加 `--with-gate-tables`，"
              "要在副本里起仓里脚本的逐条加 `--with scripts=<为什么>`；它拒了照它的出路办。拷草稿目录里自己的东西、拷 crates/ 下单个包或单个文件不拦。"
              "这一道只拒这种写法，不停你在跑的任何东西")
    if reasons:
        refusal_lines.append(f"  ✗ 看门狗起法不对（{'；'.join(reasons)}；认出的调用：{'、'.join(calls)}）：这样起的看门狗叫不醒主 agent，等于没盯")
        refusal_lines.append("     → 怎么办：看门狗用 Bash 的 run_in_background: true 起，命令只写 `bash scripts/agents/watch.sh [--ack …] <agent 号，逗号分隔>`，"
              "不加 `&`、`nohup`、`disown`，不重定向输出（只盯自己起的长进程写 `bash scripts/agents/watch.sh --processes`）")
    # 喂给从标准输入读命令的 shell、判不了的输入（here-string 或 printf / echo 的参数里有变量、命令替换，管道前一段是 cat 文件这类；lib_shell_words.py 的 shell_standard_input）：
    # 照常执行、记一条检出（字面的那一部分已经当命令切过，上面各道照判）
    try:
        unjudged_inputs = list(shell_words.commands_at_command_position(tool_input.get("command") or "").unjudged_shell_inputs)
    except Exception as error:
        not_judged(f"bash-command-detector.sh 没判成喂给 shell 的标准输入（{error!r}），这条命令照常执行")
        unjudged_inputs = []
    # 判不了的几样（各道抛异常、库读不到、喂给 shell 的不是字面）都记检出；弄坏开关 failure-unrecorded 下不记（只打 stderr）
    unjudged_notes = [] if detector_break_is_set("failure-unrecorded") else list(JUDGING_NOTICES) + unjudged_inputs
    detections_path = os.environ.get("AGENT_HOOK_DETECTIONS") or DEFAULT_DETECTIONS
    if refusal_lines:
        blocks = [refusal_lines[index:index + 2] for index in range(0, len(refusal_lines), 2)]
        if detector_break_is_set("stop-at-first-refusal"):
            blocks = blocks[:1]   # 弄坏开关：退回只报第一类的旧样子（只给自检证明「一次列全」那一格会红）
        if len(blocks) > 1:
            print(f"  ✗ 这条命令踩了 {len(blocks)} 类执行前拒绝的写法，下面一次列全；逐类照各自的「→ 怎么办」改完再起", file=sys.stderr)
        for block in blocks:
            print("\n".join(block), file=sys.stderr)
        if unjudged_notes:   # 拒了的命令不记别的检出，判不了的几样照记（结局「被拒」）
            try:
                append_detection(hook_input, unjudged_notes, detections_path, "被拒")
            except Exception as error:
                print(f"  ! bash-command-detector.sh 没把判不了的几样写进检出记录（{error!r}）", file=sys.stderr)
        return 2
    try:
        timeout_notes = subagent_timeout_findings(hook_input)
    except Exception as error:
        timeout_notes = [] if detector_break_is_set("failure-unrecorded") else [
            not_judged(f"bash-command-detector.sh 没判成子 agent 前台命令的 timeout（{error!r}），这条命令照常执行")]
    try:
        record(hook_input, detections_path, list(noted_overwrites) + list(scope_verdict.noted) + list(noted_ledger) + unjudged_notes + timeout_notes)
    except Exception as error:
        print(f"  ! bash-command-detector.sh 没把检出写进检出记录（{error!r}），这条命令照常执行", file=sys.stderr)
    return 0

sys.exit(main())
PY
hook_status=$?
if [[ $# -eq 0 && "$hook_status" != 0 && "$hook_status" != 2 && ( "$(cat "${BASH_COMMAND_DETECTOR_SELFTEST_MARK:-/nonexistent}" 2>/dev/null)" != bash-command-detector-selftest || ",${BASH_COMMAND_DETECTOR_BREAK:-}," != *,bash-layer-unrecorded,* ) ]]; then
  hook_record_bash_layer_failure "bash-command-detector.sh" "$hook_status" "$hook_input"
  exit 0
fi
exit "$hook_status"
