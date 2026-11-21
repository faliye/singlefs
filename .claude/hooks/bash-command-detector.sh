#!/usr/bin/env bash
# admission: always Claude Code 每一次触发都要现判这一次调用，上一次的结论不替这一次作保
# run-condition: command python3
# PreToolUse hook（Bash）：检出可能出问题的命令，记下来交给主 agent 判断；起看门狗的错误写法、前台没超时的等待循环、把活放出追踪的写法、run_in_background 里后面没有 wait 的单独 `&`、整份覆盖 `research/results/` 下未跟踪产物的写法、打得到别人进程的终止写法、在同一个 inode 上改已有脚本的写法、`rm -r` 作用在工作根本身、它的上级或工作根下第一层通配上的写法、项目子 agent 在 Bash 里起 claude 会话的写法、暂存区非空时不带具体路径清整个索引的写法（⑮）与带路径撤掉工作区里没暂存改动的写法（⑱）、手拷仓副本的写法（⑳）、工具员在 Bash 里改两个服务已存在的源码与构建清单、往已有门禁的样本档目录写的写法（⑭）、前台命令的 `timeout` 超过 240000 毫秒的写法（㉑）、绕开 git 钩子的 `git commit` / `git push` 写法与 git 钩子没装上时的 `git commit` / `git push`（㉒；提交与推送前的大文件判法在进仓的 git 钩子里）、交给任务服务的 `cargo build` 点了全部目标而没带 `--full-gate` 的写法（㉓）在执行前拒绝，其余只记不拦，不停任何在跑的命令与脚本。
# hook-events: PreToolUse:Bash
# gate-similar: scripts/gate/large-file-git-hook.py 是 git 钩子（scripts/gate/git-hooks/ 里的各个）的判法（起 scripts/gate/large-file-check.py 的 --guard），提交与推送前的大文件判在那里；门禁 code-tooling 的 committed-file-size 格在整轮门禁里判已经在树、暂存区与没推的提交里的文件，这里的 ㉒ 不判内容，只拦绕开钩子的写法与钩子没装上，三处各管一个时点，并到一起不行
# gate-similar: project-code-guard.sh 同挂 PreToolUse[Bash]、按同一个切词模块认命令位置，但它判的是不是不经 task-server 跑项目代码；这里判等待循环、放出追踪、单独的 &、覆盖产物、终止进程、同 inode 改脚本、就地改仓内文件、rm -r 通删工作根，判据没有一条相同
# gate-similar: write-guard.sh 管 Write / Edit 的写范围与先编后换；这里 ⑧ 管有 Edit 的子 agent 绕到 Bash 里写仓内文件、⑨ 管执行员在 Bash 里写主工作区 crates/ 下的 .rs，两边判的工具不同
# gate-similar: write-guard.sh 的第五道、第七道判 Write / Edit / MultiEdit 写进 records/ 的内容；这里的 ⑲ 不看内容，拒 Bash 里往 records/ 下 .md 的追加本身（>>、&>>、tee -a、python 的 open(…, 'a')），出路 scripts/kb/append-record.py 写之前经 scripts/kb/lib_notation_check.py 读同一份判据库 lib-forbidden-notations.py；挂的工具不同，并不到一起
# gate-similar: pattern-process-guard.sh（上游）同挂 Bash、拒按模式找进程；这里不判那一类
# gate-similar: run-scope-guard.sh 同挂 PreToolUse[Bash]、同样拆 task-server/task submit 的 `--` 之后那一条（task_server_submission）并认 --full-gate，它的 lib_run_scope.py 也有一组全部目标的 cargo 选项（CARGO_EVERY_TARGET_FLAGS）；它判起验证的命令带没带范围声明、cargo test 的种类与范围，cargo build 在它那里是不要声明的一类，这里的 ㉓ 判交给服务的 cargo build 点没点全部目标，并进去它就要多管一类不起验证的命令，两组选项也不同（这里多 --workspace、--all，不收 --bins）
# gate-similar: project-code-guard.sh 的编译目录那一道调 lib_workspace_cargo_lock.py 判主工作区里直接起的 cargo 编译目录落在哪；这里的 ㉓ 调同一份 cargo_invocations 剥包装、认子命令，判的是交给服务的那一条点没点全部目标，不另写剥法，判据与放行条件都不同
# gate-similar: task-server/task_server_lib.py 的 unnamed_run_problem（服务提交时拒 ⑦，run-scope-guard.sh 也调它）判经 submit 交的测试与验证点没点名，同样对 --full-gate 放行；它只管起测试的命令、在服务与范围声明闸两处判，这里的 ㉓ 在执行前管不起测试的 cargo build，出路给不链接与点名目标的写法
# gate-similar: project-code-guard.sh 的 code_discipline_lints 与 clippy_lint_table_tail 读同一张表（check.sh 的 CODE_DISCIPLINE_LINTS）、判 clippy 带没带全；表的读法抽在同目录的 lib_code_discipline_lints.py，两边都调它，这里的 clippy_lint_tail 只为 ㉓ 的出路拼 `--` 之后的整段、不判命令，不并进它：挂的闸与判的对象不同
# gate-similar: write-guard.sh 的第五道、第七道判 Write / Edit / MultiEdit 写进的钟点、会话号、批次代号与任务号、会话短号、批号；这里的 ⑰ 判 Bash 里 heredoc、echo、printf、tee 往仓里写的字面内容，读同一份判据库 lib-forbidden-notations.py，挂的工具不同；scripts/kb/lib_notation_check.py 判定点写入脚本的新串，也读那一份
# gate-similar: write-guard.sh 的第二道（项目子 agent 的写范围）读同一张 agent-write-scope.tsv，读表与判路径落没落进范围的函数抽在 lib_write_scope.py、两边都调它；它管 Write / Edit / MultiEdit 的目标，这里的 ⑫ 管 Bash 里写进仓里的目标，挂的工具不同，并不到一起
#
# 按模式找进程（`pgrep -f`、`pkill -f`、`killall`）不在这里判：上游 SOP 的 `.claude/singlefs-ai-sop/scripts/claude-hooks/pattern-process-guard.sh`
# 在执行前拒绝（`.claude/settings.json` 里与本 hook 注册在同一条 Bash matcher 下），判据与 shell-lint 的 S2、S3 同一份、只认命令位置。
# 这里不判：「这几个词出现在哪都算」的判据会把写在 grep 参数里的词也拦了，所以拦截放上游。
#
# hook 不能终止任务和脚本，只负责检出问题，交给主 agent 去判断；子 agent 与脚本结不结束由主 agent 定。
#
# 检出两种，写进检出记录（默认工作根下的 agent-hook-detections.jsonl，同目录 lib_work_root.py 的 default_detections_path；环境变量 AGENT_HOOK_DETECTIONS 可改），每条一行 JSON：
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
# 主 agent 的看门狗（scripts/agents/agent-watch.py watch）每 15 秒读一次检出记录，读到本会话的检出就退出、叫醒主 agent。
# 两种检出都先剥掉喂给非 shell 命令的 heredoc 正文（往笔记里写一段等待循环不是在跑它）；喂给 bash / sh 的 heredoc 正文照查。其余只看顶层命令文本，
# 引号里当数据写的等待循环也会被记一条，主 agent 看了判断即可。
#
# 判不了的（放行但必记检出）：hook 输入不是 JSON 对象（stderr 打一行带钩子名与开头若干字节的诊断，检出记录的会话号写「读不到」）、
#   哪一道判的时候抛了异常、共用模块之外的库（lib_project_code_runs.py、lib_writer_lock.py、lib_write_scope.py）读不到、喂给 shell 的不是字面（见 ① 的「判不到的」）、
#   钩子自己的 python 部分文件读不到、编不过或不完整（顶层没有入口要的名字——DETECTOR_PART_REQUIRED_NAMES 一份一组，import 之前用 ast 静态核、点名缺哪个——或 import 抛错；引导先把每一份读进来、核名字、语法解析过，stderr 点名是哪一份、python 退 70，bash 那一层记检出；主函数 main 在引导核完之后由引导调用，部分文件末尾不自己退出）：
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
# ⑧ 有 Edit 工具的项目子 agent（`.claude/agents/<agent_type>.md` 的 tools 里有 Edit）在 Bash 里就地改仓内文件：命令位置上的 `sed -i`、`perl -i`（目标不在 /tmp/ 下、也不在工作根下；相对路径从 cd 之后的目录算，见下），
#   或 python 代码（含 python3 起的工作根下脚本的正文，见下）里 open(<仓内路径字面量>, 'w' / 'a' / 'x')、Path(<仓内路径字面量>).write_text( / .write_bytes(，以及 os.replace / os.rename / shutil.move 的目标（第二个参数或 dst=）
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
#   判不到的：管道中间隔了别的命令（`cat <<'EOF' | tee x | python3 -`）、代码从变量里读的。
#   命令位置上的 `perl -i` 与 sed -i 同一处判（overwrite_steps 的 every_write 交它的目标，同样跟 cd、同样放行 /tmp/ 与工作根下的目标）；开关簇按 perlrun 认（perl_in_place_files）：
#   -pi、-i.bak、-pi.bak、-pie（备份后缀为 e 的 -i）、-lpi、-0777 -i 都是就地改，-Ilib、-MList::Util 簇里的 i 不是 -i，只读的 perl -ne 不拦
#   （弄坏开关 perl-in-place-unseen 下 ⑧ 不认 perl -i；perl-switch-cluster-by-regex 下照 ^-[a-zA-Z]*i 认 -i）。
#   `python3 <脚本文件>`：命令位置上 python 起的脚本（共用模块交 launcher 为 python 的那一条，bash -c、前缀与包装里的照认），路径此刻算得出、文件读得到、落在工作根下的，
#   读进正文按上面判 python 代码的同一套正则判（python_script_bodies；相对路径从跑它时的当前目录算，cd 进草稿目录再起的、落在草稿目录里的放行）；
#   正文里写文件的目标不是字面量的（sys.argv、变量、拼出来的路径）判不出，照常执行、记一条检出（python_unliteral_write_targets）。
#   仓里的脚本（replace-once.py 这类定点写入脚本）、工作根之外的、路径算不出的、读不到的不读（弄坏开关 python-script-body-unread 下一份都不读，
#   python-script-any-location-read 下不看落在哪个根底下，python-script-unliteral-write-unnoted 下不记检出）。
#   判不到的：直接执行的 python 脚本（`./x.py`）、脚本 import 进来的别的文件、脚本里经 subprocess 起的写法。
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
#   对自己用新会话起的那一个子进程的进程组发信号（上游 .claude/rules/command-safety-by-review.md「装置起的样本进程，放进它自己的进程组」）：
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
#   python 的路径只认字符串字面量、整段代码里只赋过一次的名字、Path(…) 与 Path(…) / '…'、os.path.join 的字面量，与 `for 名字 in <元组、列表或集合字面量>:` 的循环变量：
#   名字整段只被这一个 for 绑定（绑定次数数赋值、for 与推导式的目标、with … as、函数与 lambda 的参数、import、增量赋值、海象与 del；
#   `except … as`、`def`、`class` 的名字与 match 里绑的名字不数，同名的这几种在时也照按候选判）、每个元素照前面几种都算得出的，
#   循环体里拿它（或经只赋过一次的名字引到它）当目标的，按每个元素逐个判，任一个是已有脚本就拒，
#   拒绝说明点名那一个（literal_loop_candidates、python_path_choices；⑨ ⑫ ⑭ ⑯ ⑲ 认 python 写的目标用同一段，同样逐个判；
#   弄坏开关 python-loop-variable-unexpanded 下循环变量照算不出）；模式只认字面量。
#   循环体里的条件（if、continue、break、try）不看、按每个候选判：先判 os.path.exists 只写新文件、只写某种后缀的循环，候选里有已有脚本照拒，
#   拒绝说明给这一形的出路（改已有脚本用 Edit 工具或写临时文件再 `mv`，只写新文件的把已有脚本从候选里拿掉）。
#   判不到的：变量里拼出来的命令、eval、`bash x.sh` 起的脚本文件里的写法、目标里有这一刻算不出的段（没赋过值的变量、命令替换、通配）、
#   python 里从 sys.argv 或别的运行期值来的路径；python 的循环变量里，元素有一个算不出或带星号的、名字另有绑定的（赋值、别的 for、函数参数这些）、
#   for 的目标不是单个名字的、可迭代对象不是字面量的（range、推导式、变量里的列表）、在循环体之外（含 for 的 else 段）用的、几个循环变量的取值组合多过
#   MAXIMUM_LOOP_LANDINGS 种的；循环本身写在推导式里的（`[open(f, 'w') for f in (…)]`，推导式的 for 子句不是 for 语句）与 lambda 的参数
#   （`map(lambda f: open(f, 'w'), (…))`，参数算一次绑定、也不是 for）；`os.open` 与 `os.write`、经 subprocess 起的写法。
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
#   sed -i、perl -i、rsync 的目标，python 代码里 open(…, 带 w / a / x / + 的模式)、Path.write_text / write_bytes、shutil.copy*、os.replace / rename），加上 KNOWN_WRITE_SCRIPTS 六个定点写脚本的目标文件：
#   replace-once.py 的 文件、insert-row.py 的 文件（--line、--absent-literal 这类选项的值不算目标；弄坏开关 scope-insert-row-absent-literal-as-target 下 --absent-literal 的值算）、replace-batch.py 规格 JSON 里每一项的 file（规格读不到或不是数组的，落点算不出，拒）、
#   scripts/gate/make-swap-spec.py 的 --out 那一份规格（`--out 路径`、`--out=路径`；弄坏开关 scope-swap-spec-out-unseen 下不认它；工具自己另拒 --out 落在 git 工作树里）；
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
#   没有写范围的不判：主 agent（输入里没有 agent_type）、表里没有它那几行的定义（表给每个定义都登记，包括只带 Read、Bash 的那几类，
#   门禁 code-tooling 的 agent-write-scope 格要每个定义都有行；新建而还没登记的定义走到这里，那一格会红）。
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
#   只带 Read、Bash 的定义（没有 Write、Edit，写闸第九道看不见它）写工作根下的草稿目录时，⑫ 也照写者锁判（writer_lock_refusal，lib_writer_lock.writer_verdict 与写闸第九道同一份）：
#   离落点最近的写者锁在、持锁的不是这个子 agent 的拒（锁在而判不了谁持锁的也拒）；回执还没落盘的（unclaimed）放行、不认领，并记一条检出（lib_writer_lock.py 文件头；弄坏开关 scope-unclaimed-receipt-unrecorded）；
#   没有锁的：落点所在的工作根下第一层目录有归属文件 .owner（scripts/hosts/work-directory-ownership.sh 的三行格式），写的会话不是 hook 输入的会话的拒（只有 .owner、没有写者锁的别的会话的目录，other_session_owner；
#   弄坏开关 scope-owner-only-directory-unjudged），自己会话的、没有 .owner 的、.owner 写得不对的、hook 输入里没有会话号的不判。弄坏开关 scope-writer-lock-unchecked。
#   三方腿（定义 frontmatter 写了 `leg: three-way`，lib_three_way_legs.py）往 research/prompts/ 写，写范围表放行之后再收窄到这一轮自己的报告与模型目录：轮名取派发回执（主会话记录里 status 是 async_launched、
#   agentId 对得上的那一条）里提示的行首「报告：research/prompts/<轮>-<模型>-output.md」那一行（<轮>-<模型> 就是报告名去掉 -output.md，第二次同立场腿的报告名里带 -s2），只许写 <名>-output.md、<名>-model 与 <名>-model/ 里的文件，
#   别的轮的报告与模型目录拒（leg_round_problem）；回执读不到（hook 输入里没有 agent_id 或记录、记录读不了、没找到回执、提示里没有那一行）的只照写范围表判、放行，并记一条检出（lib_writer_lock.py 文件头
#   「回执读不到」那一支同口径）。弄坏开关 scope-leg-round-unchecked。
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
# ⑭ 工具员（SERVICE_SOURCE_AGENTS：tooling-writer）在 Bash 里往主工作区 prune-server、gpu-verification-server 的 src/ 下已存在的文件、各自已存在的 Cargo.toml 与 Cargo.lock 写：
#   写闸 write-guard.sh 第十一道只看 Write / Edit / MultiEdit，这一道判 Bash 那一半；落点是同一组（SERVICE_SOURCE_DIRECTORIES、SERVICE_BUILD_MANIFESTS 对 write-guard.sh 的
#   SWAP_ONLY_SOURCE_DIRECTORIES、SWAP_ONLY_SOURCE_FILES），两边各写一份、改的时候一起改。
#   另一半同样判：往主工作区已有门禁的样本档目录写（.claude/gate.d/fixtures/<门禁>/ 里，.claude/gate.d/<门禁> 是已存在的普通文件；新建也算，落点是目录时
#   .claude/gate.d/fixtures/<门禁> 本身也算），认法与写闸第十一道那一半是同一份函数 lib_write_scope.existing_gate_of_fixture（ServiceSourceView.gate_fixture_note 调它）；
#   rm、mv 挪走、解包、打补丁（git apply、patch、tar -x）的落点是样本档根或它的上级（仓根、.claude、.claude/gate.d）、底下有已有门禁的样本目录的也拒
#   （lib_write_scope.holds_existing_gate_fixtures）。拒绝说明另写一句出路：草稿目录里 fixtures-base / fixtures-new 生成补丁，打补丁的命令交主 agent。
#   这一半判不到：往样本档文件的硬链接写（服务源码那一半按 inode 认，样本档不认）；还没有门禁阶段的样本目录照放（与新门禁一起写）。弄坏开关 gate-fixtures-unseen 下这一半不判。
#   认的写法：⑫ 认的那一批（overwrite_steps 的 every_write 与 write_scripts：>、>>、&>、tee（带不带 -a）、cp / mv / install 的目标、dd of=、truncate、sed -i、perl -i、rsync 的目标，
#   python 代码里 open(…, 带 w / a / x / + 的模式)、Path.write_text / write_bytes、shutil.copy*、os.replace / rename，replace-once.py、insert-row.py、replace-batch.py 规格里的 file），
#   cd 与前面赋过值的变量、bash -c、喂给 shell 的 heredoc、命令替换跟着算（同一份 bash_scope_verdict，service_view 参数把「落点在不在写范围里」换成「落点是不是这一组里已存在的普通文件」）。
#   另认的写法（service_writes）：
#   ·带 -b、--backup[=…]、-S 的 cp、mv、install、ln：旧文件换了名字留底，目标这个名字照样是新写进去的，当写（⑤ ⑦ 照旧当不盖掉）；
#   ·源是目录的 cp -r、rsync（落点算成目录）：目标是已有的目录时，逐份对目标目录里已有的服务源码，源里同一个相对位置有文件的拒（service_directory_copy_hits）；
#     源写成 `目录/.`、rsync 的 `目录/`、cp -T 是源里的内容直接落在目标目录下，别的写法落在 目标目录/<源的名字>/ 下；源里的通配照这一刻的文件逐个展开；cp -n、目标不存在的不拒；
#   ·ln：目标已有的服务源码被换成链接（-f、-b）拒；不带 -s 的 ln、cp -l（含 cp -al 整个 src/）的源是已有的服务源码或包着它的目录拒（建硬链接那一步拦），
#     往已经是服务源码硬链接的文件写也拒（按设备号与 inode 号认，落在仓外的草稿目录里的硬链接也认，ServiceSourceView.shares_inode）；符号链接指到服务源码不拒，经它写的落点解开之后判；
#   ·rm 与 mv 挪走：路径是已有的服务源码、或包着已有服务源码的目录（`rm -r prune-server/src`、`mv prune-server/src …`）拒，rm、mv 的路径带通配的照这一刻的文件逐个判；
#     挪走与 rm 之后同名新建是整份换上服务源码，新建的那一步（这一道与写闸第十一道）当新文件放，所以挪走与删掉这一步要拦；
#   ·解包 tar -x（-xzf、xzf、--extract、--get，-C 与 --directory 照写的顺序累计，不给是当前目录）、patch（-d 给的目录，不给是当前目录；--dry-run 与 -o 写别处的不判目录）、
#     git apply（-C 累计；--check、--stat、--numstat、--summary 与只写索引的 --cached 不判）：包里、补丁里写了什么这一刻不读，按往哪个目录里写判，
#     目录在服务源码目录里（含它自己），或包着已有的服务源码（仓根、prune-server/、gpu-verification-server/）就拒（ServiceSourceView.touches）；`patch -o 文件` 照写那个文件判。
#   ·unlink（同 rm）；find 的 -delete，与 -exec / -execdir / -ok / -okdir 起的命令对找到的路径写、删、挪走（rm、mv、unlink、gzip、sed -i {}…）：落点取 find 的起点目录（不写是当前目录），
#     起点是包着已有服务源码的目录就拒，起点之下哪些文件被匹配这一刻不展开（find_removal_steps）；`find … | xargs rm` 接进来的判不到；
#   ·rsync 的 --delete（--delete-before 这一族、--del）：目的地是包着已有服务源码的目录或其中的文件就拒；--remove-source-files（--remove-sent-files）：源同 rm；-n、--dry-run 不判；
#   ·gzip、gunzip、bzip2、bunzip2、xz、unxz、lzma、unlzma、compress、uncompress、pigz、unpigz 默认把原文件换成压缩或解压后的文件，zstd、unzstd、lz4、unlz4 带 --rm 才删原文件：
#     点名的原文件同 rm 判；带 -c、--stdout、-k、--keep、-t、-l 这类原文件不动的不判（compressor_removed_files）；
#   ·`<>` 读写重定向（`printf x 1<> 文件`）：不截断、在原 inode 上从头改写，落点是已有的服务源码就拒；
#   ·shred 的文件、sort -o / --output 的文件、gawk 与 awk 带 -i inplace（--include inplace）的文件操作数：原 inode 上整份改写，落点同写；
#   ·unzip：解进 -d 给的目录（不给是当前目录），同 tar -x 按目录判；-l、-v、-t、-p、-c、-z、-Z 不解到盘上，不判；
#   ·python 代码：os.remove、os.unlink、Path(…).unlink、shutil.rmtree，os.rename、os.replace、shutil.move 的源，落点同 rm；带 O_WRONLY 或 O_RDWR 的 os.open，落点同写；
#     shutil.copytree(源, 目标, dirs_exist_ok=True) 往已有的目标目录逐份盖，同 cp -r 源/. 目标/；路径与标志算不出的不判（python_service_writes）；
#   ·tar 的 -O、--to-stdout、--to-command 只解到标准输出，不写盘，不判；patch 带位置参数的原文件（`patch 原文件 补丁`、`patch -i 补丁 原文件`，原文件在 -d 给的目录里找）时补丁只改那一份，
#     照那一份文件判，不照目录判；
#   弄坏开关：service-source-unlink-find-unseen（unlink、find）、service-source-rsync-removal-unseen、service-source-compressor-unseen、service-source-in-place-tools-unseen（shred、sort -o、gawk）、
#   service-source-read-write-redirect-unseen、service-source-unzip-unseen、service-source-python-delete-unseen、service-source-python-in-place-unseen、service-source-stdout-extract-judged、
#   service-source-patch-operand-ignored（后两条打开后退回按目录判，多拒）。
#   落点先规整 `..`、逐段解开符号链接（lib_write_scope.landing_forms）再判：经草稿目录里指到 src/ 下文件的链接写的同样拒。
#   前台、run_in_background 一样拒。出路：在仓副本里改（共用约束「建仓副本」；删、挪走、解包、打补丁也在副本里做），把改动存成补丁写进草稿目录，打补丁的命令写进报告「要主 agent 跑的命令」一节，由主 agent 一次打上。
#   放行：新建的文件（落点不存在）；草稿目录与仓副本里的同名文件（落在主工作区之外）；两个服务的 target/ 与别的路径下的文件；读与拷出去；`--dry-run` 的定点写脚本；
#   主 agent（输入里没有 agent_type）、别的 agent 类型；认不出 agent 类型的子 agent 调用（带 agent_id 而没有 agent_type；⑫ 同样不判）。
#   判不到的（这一项不罩它们）：落点这一刻算不出的写法（⑫ 对有写范围的 agent 照算不出拒，这里不另判）、变量里拼出来的命令、eval 的变量代码、`bash x.sh` 起的脚本文件里的写法、
#   经 subprocess 起的写法；cp -r、rsync 的源带算不出的段（变量、命令替换）；tar 包里、补丁里写了什么（只按目录拦：在仓根、prune-server/、gpu-verification-server/ 或 src/ 里解包、打补丁的一概拒，
#   只新建文件的也拒；在仓副本里做）；cpio、7z、zip -m、git checkout / restore / stash / am / merge 这类别的往工作区写的命令；`find … | xargs rm`、xargs 与 parallel 接进来的删除和写；
#   ed、ex、vim -c、sponge、perl -e、ruby 这类自己打开文件写的命令；python 的 os.truncate、Path.rename / replace 的源、os.link、tarfile 与 zipfile 解包、路径或标志算不出的 os.open、
#   不带 dirs_exist_ok=True 的 copytree（目标已存在时它自己报错，不写）；
#   硬链接是别的办法建的（python 的 os.link 这类）：建链接那一步只认 ln、cp -l，别的判不到，往那个名字写那一步按 inode 认得出。
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
# ⑰ 往仓里的文件写字面内容，内容里有会话号、钟点、批次代号或任务号、会话短号、批号（主 agent 与子 agent 一样拒）：写闸 write-guard.sh 的第五道、第七道只看 Write / Edit / MultiEdit，
#   Bash 里 `cat >> records/….md <<'EOF'`、`printf … >> 文件`、`… | tee -a 文件` 这类追加不过它。
#   认的写法（literal_writes）：cat（不带文件操作数）读 heredoc、here-string 或管道前一段的字面、echo 与 printf 的参数，经 >、>>、>|、&>、&>> 写进的；
#   tee（带不带 -a）把 heredoc、here-string、管道前一段的字面写进的；bash -c 的代码、喂给 shell 的 heredoc、命令替换里的照认，cd 与前面赋过字面值的变量跟着算。
#   拆开写的号照 shell 真写出的字判（literal_output）：printf 的格式串换掉转义、每个转换代入参数（printf_rendered；格式里有认不出的转换才退回格式串与参数各占一行）；
#   echo -e 照 bash 换掉反斜杠转义，-n 不带收尾的换行（下一处追加接在同一行）；echo、printf 的参数与结束词不带引号的 heredoc、here-string 的正文里，
#   这条命令前面赋过字面值的变量先代入（没赋过的照留 $名字）。
#   判法（literal_notation_hits）：落点在本仓里的，照同一份判据库 ../gate.d/lib-forbidden-notations.py 的 session-names、clock-times、batch-codes、
#   task-and-session-numbers 四种形态判（排除前缀、放行标记、批次代号与任务号、会话短号、批号只判 .md、.tsv、.txt，
#   判决里行首的「A/B 读数：任务 <号>」一行与 records/ 下以「长跑拆分：」起头的行照写任务号、records/ 下表格里表头是「批次」的那一列照写批号、
#   围栏代码块与反引号里只放过会话短号与批号，都由库按目标的相对路径管，与写闸判 Write / Edit 的内容同一口径）；
#   会话号、钟点与批次代号判写进去的字面内容；任务号、会话短号与批号判这一处写之前、之后两份整份目标（库的 new_violation_hits，写后多出来的违例行才拒）：
#   追加的写法（>>、&>>、tee -a）写后是写前接上内容，整份写的写后就是内容；同一条命令里几处写同一个落点的，按执行的先后在内存里依次施加（literal_landing_text），
#   前一处写进的收尾围栏让后一处追加的行出了围栏、照拒；目标里已有的旧号不挡这一次写；
#   逐处列出写法、目标、第几行（任务号那一形态是写后文件的行号）、哪一种与命中的串；
#   出路取库里每种命中形态登记的整条 remedy，另指到 Edit 与 scripts/kb/insert-row.py、replace-once.py（它们写之前也判这几样）。
#   不判的：落在仓外的、落点算不出的（变量、命令替换、通配、cd 到算不出的目录之后的相对路径）、内容不是字面的（cat 文件、cp、mv、别的命令的输出）、
#   花括号整组与子 shell 整组的重定向（`{ echo …; } >> 文件`）、sed -i / perl -i 的替换串、python 等程序里写文件的、写法在脚本文件里的；
#   重定向前的 fd 号由共用切词模块去掉了，`echo … 2>> 文件` 照标准输出算（偏严）。读不到判据库照常执行、记一条检出。
#   弄坏开关：allow-literal-notations（一概不判）、literal-notations-outside-judged（仓外的目标也判）、literal-notations-new-text-only（任务号那一形态只判写进去的内容）、
#   literal-notations-before-from-disk（写前一律读现文件，看不到同一条命令里前几处写进的东西）、literal-printf-format-unexpanded（printf 交格式串与参数各占一行，不代入）、
#   literal-echo-escapes-unexpanded（echo -e 不换转义）、literal-variables-unsubstituted（内容里前面赋过值的变量不代入）；判据本身的开关是库的 LIB_FORBIDDEN_NOTATIONS_BREAK 一族。
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
# ㉑ 前台命令的 `timeout` 过长（主 agent 与子 agent 一样拒，只看前台）：run_in_background 不是 true 的 Bash 调用，`tool_input.timeout` 是数（JSON 的整数与小数，含 Infinity、NaN 两个词；或写成数字串的字符串）
#   而超过 FOREGROUND_TIMEOUT_MILLISECONDS（240000 毫秒）：前台命令卡在这一次调用里，别的消息要等它返回才送得到，最长的 600000 毫秒就是十分钟。
#   认法（foreground_timeout_refusal）：run_in_background 为 true 的、没写 timeout 的、写成布尔的、不超过的放行；字符串只认「前后可带空白、中间只有数字（可带一个小数点）」的那一种
#   （正则 `\s*\d+(?:\.\d+)?\s*`，`\d` 连别的文字写的十进制数字也算），`"3e5"`、`"300000ms"`、`"+300000"`、`"0x493e0"`、`"300000."` 这类认不出的字符串放行；
#   子 agent 的 run_in_background 归 ⑩ 拒，不在这一道。`run_in_background` 只认 JSON 的 true，字符串 `"true"` 当前台。
#   出路按调用方分（拒绝说明里写）：主 agent 把命令改成 run_in_background: true、结束本轮等完成通知；跑项目代码的经 `task-server/task submit -- <命令>` 交，
#   再用 run_in_background 起 `task-server/task wait <号>`（任务完成时退出），不设 timeout；子 agent 把命令写进报告「要主 agent 跑的命令」一节交回。
#   弄坏开关：allow-long-foreground-timeout（一概放行）。
# ㉒ 绕开 git 钩子的 `git commit` / `git push` / `git merge` / `git am` / `git config`，与 git 钩子没装上（主 agent 与子 agent 一样拒，前台、run_in_background 一样）：提交与推送前的大文件判法不在这里读命令文本判，
#   在 git 自己的钩子里（scripts/gate/git-hooks/ 里的各个钩子，判法 scripts/gate/large-file-git-hook.py；经 `git config core.hooksPath <主工作树里 scripts/gate/git-hooks 的绝对路径>` 生效，
#   装与核对 scripts/gate/git-hooks-install.py），判的是那一刻真要进提交的暂存区、真要推出去的每个引用。这一道只拦两件事（命令位置上，lib_shell_words.py 切词；`git -C <目录>`、cd、
#   bash -c、命令替换与 nice / env 这类前缀里的、xargs / find -exec / flock 这类启动器剥开之后的都认）：
#     1. 绕开钩子的写法：git commit / git push / git merge / git am 带 `--no-verify`（长选项认唯一前缀的缩写，--no-veri 同样；git merge 的 --no-verify 绕开 pre-merge-commit 与 commit-msg，
#        git am 的 --no-verify 绕开 pre-applypatch 与 applypatch-msg，打上去的提交里超限文件照落进本地；git merge 的表里 --no-veri 有歧义，不算）、git commit 与 git am 带 `-n`
#        （-nm 这类短选项合写的也认，git am 的 -p、-C 吃下一个词；git push 的 -n 是 --dry-run、git merge 的 -n 是 --no-stat，不算）；
#        命令行 `-c core.hooksPath=…`、`--config-env=core.hooksPath=…`（git merge、git am 同样认）；环境里 GIT_CONFIG_KEY_<n> 写成 core.hooksPath、GIT_CONFIG_PARAMETERS 里有它；git config 设、加、删 core.hooksPath
#        （读不算）；git config 的 --remove-section core、--rename-section core（唯一前缀的缩写与子命令写法 remove-section、rename-section 同，core 段连同 core.hooksPath 一起没了）。
#        设成安装脚本装的那个值不算：`git config core.hooksPath <值>`、`-c core.hooksPath=<值>`、GIT_CONFIG_KEY_<n>=core.hooksPath 加 GIT_CONFIG_VALUE_<n>=<值>，<值> 是绝对路径（~ 开头的先展开）、解开之后是
#        lib_git_hooks.accepted_directories 交回的钩子目录（主工作树里 scripts/gate/git-hooks 的绝对路径，installed_hooks_path 现算；两边解开符号链接再比）时放行，相对路径照拒（--git-dir、--work-tree、GIT_DIR 起的 git、链接工作树与稀疏检出的树里找不到它，安装脚本与 lib_git_hooks.hooks_path_problems 也只认绝对路径）；--config-env 的值是环境变量名、GIT_CONFIG_PARAMETERS 是 git 内部的拼法，读不出值，照拒；--add、--replace-all、--unset 照拒。HUSKY=0 这类别的工具的开关不算。别名照 `-c alias.<名>=<值>` 与配置（git config 读全局、系统、仓里的）认成它指的子命令（值照 git 的写法切开、接着往下找，
#        最多 GIT_ALIAS_DEPTH 层），`!` 开头的 shell 别名不认。出路：去掉它；窗口外提交用环境变量 SINGLEFS_COMMIT_OUTSIDE_WINDOW=1（用户私有的提交窗口闸认它，大文件照判）。
#     2. git 钩子没装上：命令所在的仓（-C、cd 跟着带）的这棵树自带进仓的钩子目录（pre-commit 在），而 core.hooksPath 没指到它、钩子文件缺或没有执行位（判法在 lib_git_hooks.py，
#        与安装脚本共用）时，git commit / git push 照拒；--help、-h、--dry-run 的 git commit 与 --help、-h 的 git push 不起钩子，不判；--git-dir / --work-tree / GIT_DIR / GIT_WORK_TREE
#        改了仓的位置的、命令所在的目录算不出或不存在的不判；别的仓（样本仓、临时仓：没有进仓钩子目录）不判。出路：经任务服务装钩子
#        （`task-server/task submit -- python3 scripts/gate/git-hooks-install.py install`，子 agent 写进报告交主 agent）。
#   不读命令文本判 git commit / git push 要带的内容有没有超过单文件上限的文件：内容全交 git 钩子；不拦只读的 git log、git status 这类。出路命令不含绕开钩子的写法，喂这一道放行。
#   弄坏开关：allow-git-hook-guard（一概放行）、bypass-flag-unseen（不认 --no-verify 与 -n）、bypass-config-unseen（不认改 core.hooksPath 的几种）、merge-bypass-unseen（不判 git merge）、am-bypass-unseen（不判 git am）、
#   config-section-unseen（不认 --remove-section core 与 --rename-section core）、hooks-path-right-value-refused（设成装对的值也拒）、hooks-path-relative-accepted（相对路径也算装对的）、hooks-not-installed-allowed（不判装没装上）、
#   bypass-every-git-command（只读的 git log、git status 这类也按 git commit 判）、alias-unresolved（不查别名）、dynamic-launcher-unseen（不剥启动器）、long-option-prefix-unseen（长选项只认全名）、
#   git-hook-messages-unguarded（写拒绝说明那一步出错不接）。
# ㉓ 交给任务服务的 cargo build 点了全部目标（主 agent 与子 agent 一样拒，前台、run_in_background 一样）：命令位置上的 task-server/task submit（lib_project_code_runs.py 的
#   task_server_submission 认的那一种：本仓的客户端或逐字节相同的拷贝，python3 起的同算；范围声明写在前面、bash -c、&& 后一段、命令替换里的都认）`--` 之后那一条里，
#   命令位置上会编的 cargo build（cargo b 同算；lib_workspace_cargo_lock.py 的 cargo_invocations 剥 env、bash -c、cd、内存包装这类，任务的当前目录取 --cwd）
#   带 --all-targets、--tests、--benches、--examples、--workspace、--all 之一（WHOLE_TARGET_BUILD_FLAGS）。为什么：调试构建把测试、示例与基准逐个链接成可执行文件，
#   验证包一趟写两百多 GiB，盘顶满时别的任务写不进、等判定库的任务被服务判卡住（whole_target_build_refusal）。
#   放行：submit 在 `--` 之前带 --full-gate 的（提交时的整轮门禁；--cwd、--whole-package-confirmed 的值不当选项认，submit_options_have_full_gate）；cargo build 点名包而不带这几个选项的；
#   别的 cargo 子命令（clippy 只查不链接，test 归 run-scope-guard.sh 与服务的「经 submit 交的要点名」）；交给服务的是脚本的（`bash scripts/gate/merge-and-verify.sh …`、门禁阶段）：
#   只判写在命令里的那一条，起的脚本正文不读，合入后验证与门禁自己的构建由它们自己管；没经 submit 直接起的 cargo（归 project-code-guard.sh）；引号里当数据写的。
#   判不了的（放行，记一条检出）：lib_project_code_runs.py 或 lib_workspace_cargo_lock.py 读不到、判的时候抛异常。
#   出路（whole_target_build_messages，拒绝说明里给三种写法）：只核编得过写 `cargo clippy -p <包> --all-targets -- -D warnings -D clippy::<名> …`（`--` 之后是 check.sh 里 CODE_DISCIPLINE_LINTS 那张表
#   每次现读的整段，clippy_lint_tail 经同目录的 lib_code_discipline_lints.py 读这一份钩子所在仓里的那一份，表里的词原样交出；project-code-guard.sh 读同一份、要 clippy 带全它，整轮门禁与合入后验证跑的 clippy 也带它）；要编、要跑的测试点名目标写
#   `cargo test -p <包> --test <目标> -- <测试全名…> --exact`（库里的换成 --lib）；构建只编库与二进制写 `cargo build -p <包>`；都经 task-server/task submit 交，子 agent 写进报告交回。
#   表读不到或没读到一条时，出路里的 clippy 写法不带那一段、说明里写明要照 check.sh 补全（project-code-guard.sh 读同一份表，读不到时也不判），并记一条判不了。
#   出路本身要过得了别的闸：自证从拒绝说明里取出三种写法（WHOLE_TARGET_HOWTO_COMMAND），占位换成样本（WHOLE_TARGET_HOWTO_SAMPLES），拼成 `task-server/task submit -- <写法>`
#   （cargo test 那一条前面加 target 种类的范围声明），主 agent 喂这一份钩子、project-code-guard.sh、run-scope-guard.sh，都要放行；hook 旁边没有那一份时那几格不判，成功句报「本次未跑」。
#   弄坏开关：allow-whole-target-build（一概放行）、whole-target-full-gate-unseen（不认 --full-gate）、whole-target-howto-unnamed-test（出路里的 cargo test 不点名测试全名）、
#   whole-target-howto-clippy-lints-dropped（出路里的 cargo clippy 不带 lint 表）；读表的共用模块 lib_code_discipline_lints.py 只认 `-D clippy::<名>` 一行一条的写法（读不成时 ㉓ 出路里的整段交空串、记一条判不了），它的弄坏开关是 LIB_CODE_DISCIPLINE_LINTS_BREAK=entry-form-unchecked 与 other-assignments-unscanned，
#   两道钩子平常跑设了它们一律退 2（refuse_break_outside_selftest 与 project-code-guard.sh 的 SELFTEST_ONLY_VARIABLES）。
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
#       「先编后换:执行员 mv 把入库装置挪走」「先编后换:执行员 bash -c 里 cp」「先编后换:执行员变量里的目录」「先编后换:执行员目标算不出、对得上已有的入库装置」「先编后换:执行员 python shutil.copy」「先编后换:执行员 python open(…, 'a')」「先编后换:执行员 python os.replace」「先编后换:执行员 python 的 for 逐个取元组，open(循环变量, 'a') 写到入库装置」
#       「先编后换:执行员 cwd 在主工作区的 crates/ 里、相对路径写 .rs 拒」「两道包装:包着的前台没超时的等待循环拒」「两道包装:--quiet 包着的前台没超时的等待循环拒」「两道包装:包着的 kill 负进程号拒」「两道包装:包着的 setsid -f 拒」「两道包装:包着的 cp 盖主工作区入库装置拒（先编后换）」
#       「Bash写范围:clerk 用 replace-once.py 写知识库（现场：越出写范围）」「Bash写范围:clerk 直接执行 replace-once.py 写知识库」「Bash写范围:clerk 先 cd 到 records 再用相对路径写知识库」「Bash写范围:kb-scribe 用 replace-once.py 写 records 拒」
#       「Bash写范围:clerk 用 insert-row.py 写知识库」「Bash写范围:clerk 用 make-swap-spec.py 的 --out 写知识库」「Bash写范围:clerk 用 replace-batch.py，规格点名知识库」「Bash写范围:clerk 脚本路径在变量里，用 replace-once.py 写知识库」「Bash写范围:clerk 脚本路径在变量里，用 insert-row.py 写知识库」
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
#       「就地改脚本:喂给 python 的 heredoc：shutil.copy 进已有目录」「就地改脚本:cd 之后喂给 python 的相对路径」「就地改脚本:喂给 python 的 heredoc：for 逐个取元组里只赋过一次的名字，open(循环变量, 'w') 盖 scratch 根下的脚本」「就地改脚本:喂给 python 的 heredoc：for 逐个取列表字面量，候选里有一个是已有脚本」「就地改脚本:bash -c 里的 python3 -c」「就地改脚本:一百三十七：rsync --inplace 盖已有脚本（rsync-inplace-unseen 下红）」
#       「就地改脚本:喂给 python 的 heredoc：with open(os.path.join(…, 循环变量), 'w') 嵌在 for 的循环体里」「就地改脚本:拒绝时报出写法与相对仓库根的路径」「就地改脚本:python 的 for 逐个取值时，拒绝点名是已有脚本的那个候选、不点名别的候选」「就地改脚本:工作根下的脚本」「就地改脚本:会话临时目录根下的脚本」「就地改脚本:会话 scratchpad 里的脚本」「就地改脚本:⑤ 现在认 dd、truncate 覆盖未跟踪产物（与 ⑦ 共用同一段 overwrite_steps，不抄第二份）」「就地改脚本:rm 掉再写未跟踪产物，⑤ 照拒（rm 没留住旧字节）」
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
#       「stdin:kill 写死的自检进程号（hook 的祖先）拒绝（退出码 2）」「stdin:run_in_background 里循环 kill 拒绝（退出码 2）」「stdin:一条命令同时踩几类时一次列全（终止进程、放出追踪、前台等待循环）」（共 492 格）
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
#       ⑧ 认 perl -i 与读 python 脚本正文那几格：
#       「就地改仓内文件:perl -pi -e 改仓内知识库」「就地改仓内文件:perl -i -pe 改入库装置」「就地改仓内文件:perl -i.bak 带备份后缀」「就地改仓内文件:perl -pi.bak -e 合写带备份后缀」
#       「就地改仓内文件:perl -pie 是备份后缀为 e 的 -i，后面的文件照拒」「就地改仓内文件:perl -lpi 合写照拒」「就地改仓内文件:perl -0777 -i -pe 照拒」「就地改仓内文件:bash -c 里的 perl -pi 照拒」
#       「就地改仓内文件:python3 起草稿目录里的脚本按字面量改入库装置照拒」「就地改仓内文件:python3 起草稿目录里的脚本按仓根起的绝对路径写知识库照拒」
#       「就地改仓内文件:python3 起草稿目录里的脚本用 os.replace 挪进 scripts/ 照拒」「就地改仓内文件:nice 前缀、带选项与参数起草稿目录里的脚本照拒」
#       「就地改仓内文件:bash -c 里起草稿目录里的脚本照拒」「就地改仓内文件:脚本的目标取自 sys.argv，记一条检出」「就地改仓内文件:脚本的目标是拼出来的变量，记一条检出」（共 15 格）
#       「Bash写范围:只读定义:experiment-designer 往实验页的索引写，拒」「Bash写范围:只读定义:mutation-triage 往 crates/mutations.tsv 追加，拒」「Bash写范围:只读定义:prior-art 用 sed -i 改 prior-art 的 kb 页，拒」
#       「Bash写范围:只读定义:sweep 用 sed -i 改被搜到的 records 文件，拒」「Bash写范围:只读定义:three-way-attack 写 crates，拒」「Bash写范围:只读定义:three-way-attack 写别的轮的背景材料，拒」
#       「Bash写范围:只读定义:写者锁:prior-art 写别的子 agent 持写者锁的草稿目录，拒」
#       「Bash写范围:只读定义:three-way-defense 写 crates，拒」「Bash写范围:只读定义:three-way-forward 写 records，拒」「Bash写范围:只读定义:three-way-materials 改正文，拒」（只读定义那九格）
#       「Bash写范围:三方腿:攻方写更早一轮的报告，拒」「Bash写范围:三方腿:攻方写更早一轮的模型目录里的文件，拒」「Bash写范围:三方腿:第二次同立场的攻方写第一次那条腿的报告，拒」「Bash写范围:三方腿:辩方写别的轮的报告，拒」「Bash写范围:三方腿:正推腿写别的轮的报告，拒」「Bash写范围:三方腿:派发回执读不到的攻方写别的轮的报告：只照写范围表放行，记一条检出」「Bash写范围:只读定义:写者锁:prior-art 写有锁没认领、派发回执读不到的草稿目录，放行并记一条检出」「Bash写范围:只读定义:写者锁:prior-art 写只有 .owner（别的会话的）、没有写者锁的草稿目录，拒」（三方腿、写者锁里没认领与只有 .owner 那八格）
#       「就地改脚本:喂给 python 的 heredoc：循环体里 target = os.path.join(…, 循环变量) 只赋过一次，open(target, 'w') 经它引到循环变量」「就地改脚本:喂给 python 的 heredoc：循环体里先判 os.path.exists 只写新文件，条件不看、候选里有已有脚本照拒」「就地改脚本:喂给 python 的 heredoc：循环体里只写 .md 后缀，条件不看、候选里有已有脚本照拒」
#       「就地改脚本:喂给 python 的 heredoc：循环变量另在 except … as 里绑定（不数进绑定次数），照按候选拒」「stdin:python 的 for 循环体里先判 os.path.exists 只写新文件照拒（退出码 2），stderr 写了这一形的出路（条件不看、Edit 工具或写临时文件再 mv）」「Bash写范围:clerk 喂给 python 的 heredoc 里 for 逐个取元组，候选里有一个落在写范围外（知识库）照拒」（python 循环按候选判那六格）
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
#       ⑧ 认 perl -i 与读 python 脚本正文那几格：
#       「就地改仓内文件:perl -pi -e 改仓内知识库」「就地改仓内文件:perl -i -pe 改入库装置」「就地改仓内文件:perl -i.bak 带备份后缀」「就地改仓内文件:perl -pi.bak -e 合写带备份后缀」
#       「就地改仓内文件:perl -pie 是备份后缀为 e 的 -i，后面的文件照拒」「就地改仓内文件:perl -lpi 合写照拒」「就地改仓内文件:perl -0777 -i -pe 照拒」「就地改仓内文件:bash -c 里的 perl -pi 照拒」
#       「就地改仓内文件:python3 起草稿目录里的脚本按字面量改入库装置照拒」「就地改仓内文件:python3 起草稿目录里的脚本按仓根起的绝对路径写知识库照拒」
#       「就地改仓内文件:python3 起草稿目录里的脚本用 os.replace 挪进 scripts/ 照拒」「就地改仓内文件:nice 前缀、带选项与参数起草稿目录里的脚本照拒」
#       「就地改仓内文件:bash -c 里起草稿目录里的脚本照拒」「就地改仓内文件:脚本的目标取自 sys.argv，记一条检出」「就地改仓内文件:脚本的目标是拼出来的变量，记一条检出」（共 15 格）
#   BASH_COMMAND_DETECTOR_BREAK=allow-compile-first-bypass  ⑨ experiment-runner 在 Bash 里写主工作区 crates/ 下的 .rs 也放行：实测红的格（照失败行原文）
#       「先编后换:执行员 cp 草稿副本盖主工作区的入库装置」「先编后换:执行员 cat > 新建入库装置」「先编后换:执行员 >> 追加」「先编后换:执行员 tee -a」「先编后换:执行员 mv 草稿副本进 bin 目录」「先编后换:执行员 cd 进 bin 目录之后 cp 到 .」「先编后换:执行员 sed -i」「先编后换:执行员 perl -pi -e」
#       「先编后换:执行员 rsync」「先编后换:执行员 rm 入库装置」「先编后换:执行员 mv 把入库装置挪走」「先编后换:执行员 bash -c 里 cp」「先编后换:执行员变量里的目录」「先编后换:执行员目标算不出、对得上已有的入库装置」「先编后换:执行员 python shutil.copy」「先编后换:执行员 python open(…, 'a')」
#       「先编后换:执行员 python os.replace」「先编后换:执行员 python 的 for 逐个取元组，open(循环变量, 'a') 写到入库装置」「先编后换:执行员 cwd 在主工作区的 crates/ 里、相对路径写 .rs 拒」「两道包装:包着的 cp 盖主工作区入库装置拒（先编后换）」「stdin:experiment-runner cp 进主工作区的入库装置拒绝（退出码 2）」「stdin:先编后换的拒绝点名那条脚本与出路」（共 22 格）
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
#       「就地改脚本:喂给 python 的 heredoc：os.path.join 与 io.open」「就地改脚本:喂给 python 的 heredoc：shutil.copyfile」「就地改脚本:喂给 python 的 heredoc：shutil.copy 进已有目录」「就地改脚本:cd 之后喂给 python 的相对路径」「就地改脚本:喂给 python 的 heredoc：for 逐个取元组里只赋过一次的名字，open(循环变量, 'w') 盖 scratch 根下的脚本」「就地改脚本:喂给 python 的 heredoc：for 逐个取列表字面量，候选里有一个是已有脚本」
#       「就地改脚本:bash -c 里的 python3 -c」「就地改脚本:一百三十七：rsync --inplace 盖已有脚本（rsync-inplace-unseen 下红）」「就地改脚本:喂给 python 的 heredoc：with open(os.path.join(…, 循环变量), 'w') 嵌在 for 的循环体里」「就地改脚本:拒绝时报出写法与相对仓库根的路径」「就地改脚本:python 的 for 逐个取值时，拒绝点名是已有脚本的那个候选、不点名别的候选」「就地改脚本:工作根下的脚本」「就地改脚本:会话临时目录根下的脚本」「就地改脚本:会话 scratchpad 里的脚本」「stdin:cp 覆盖已有脚本拒绝（退出码 2）」
#       「就地改脚本:喂给 python 的 heredoc：循环体里 target = os.path.join(…, 循环变量) 只赋过一次，open(target, 'w') 经它引到循环变量」「就地改脚本:喂给 python 的 heredoc：循环体里先判 os.path.exists 只写新文件，条件不看、候选里有已有脚本照拒」「就地改脚本:喂给 python 的 heredoc：循环体里只写 .md 后缀，条件不看、候选里有已有脚本照拒」「就地改脚本:喂给 python 的 heredoc：循环变量另在 except … as 里绑定（不数进绑定次数），照按候选拒」「stdin:python 的 for 循环体里先判 os.path.exists 只写新文件照拒（退出码 2），stderr 写了这一形的出路（条件不看、Edit 工具或写临时文件再 mv）」
#       「stdin:拒绝时 stderr 写了出路（临时文件再 mv 换上、replace-once.py / insert-row.py 定点改）」「stdin:run_in_background 里喂给 python 的 open(…, 'w') 同样拒绝（退出码 2）」（共 56 格）
#   BASH_COMMAND_DETECTOR_BREAK=python-loop-variable-unexpanded  python 的 `for 名字 in <元组、列表或集合字面量>:` 循环变量不按元素逐个代入，照算不出不判（literal_loop_candidates 交空；⑦ ⑨ ⑫ ⑭ ⑯ ⑲ 一并受影响）：打开后红的格
#       「就地改脚本:喂给 python 的 heredoc：for 逐个取元组里只赋过一次的名字，open(循环变量, 'w') 盖 scratch 根下的脚本」「就地改脚本:喂给 python 的 heredoc：for 逐个取列表字面量，候选里有一个是已有脚本」「就地改脚本:喂给 python 的 heredoc：with open(os.path.join(…, 循环变量), 'w') 嵌在 for 的循环体里」
#       「就地改脚本:喂给 python 的 heredoc：循环体里 target = os.path.join(…, 循环变量) 只赋过一次，open(target, 'w') 经它引到循环变量」「就地改脚本:喂给 python 的 heredoc：循环体里先判 os.path.exists 只写新文件，条件不看、候选里有已有脚本照拒」「就地改脚本:喂给 python 的 heredoc：循环体里只写 .md 后缀，条件不看、候选里有已有脚本照拒」「就地改脚本:喂给 python 的 heredoc：循环变量另在 except … as 里绑定（不数进绑定次数），照按候选拒」「stdin:python 的 for 循环体里先判 os.path.exists 只写新文件照拒（退出码 2），stderr 写了这一形的出路（条件不看、Edit 工具或写临时文件再 mv）」「Bash写范围:clerk 喂给 python 的 heredoc 里 for 逐个取元组，候选里有一个落在写范围外（知识库）照拒」「往现行账本追加:喂给 python 的 heredoc 里 for 逐个取元组，候选里有一个是现行账本拒」「往 records 追加:喂给 python 的 heredoc 里 for 逐个取元组，候选里有一个落在 records 下拒」
#       「就地改脚本:python 的 for 逐个取值时，拒绝点名是已有脚本的那个候选、不点名别的候选」「先编后换:执行员 python 的 for 逐个取元组，open(循环变量, 'a') 写到入库装置」「服务源码:工具员 python 的 for 逐个取元组，open(循环变量, 'w') 盖 prune-server/src/protocol.rs」（共 14 格）
#   BASH_COMMAND_DETECTOR_BREAK=keep-selftest-scratch  自证的临时目录走回 mkdtemp 不删（lib_selftest_scratch.py 的 keep）
#       「通过那条路上临时目录没删」「red 那条路上临时目录没删」「crash 那条路上临时目录没删」（共 3 格）
#   BASH_COMMAND_DETECTOR_BREAK=refuse-every-wait-loop  ② 外层 timeout 与 run_in_background 都不算，等待循环一律拒：实测红的格（照失败行原文）
#       「等待循环:同一条套上 timeout 60」「等待循环:同一条走 run_in_background」「等待循环:timeout 60 bash 的 heredoc 里的循环」「等待循环:capped.sh 与 nice 包着的 timeout」「等待循环:$(timeout 60 bash <<EOF) 里的循环」
#       「两道包装:包着的 timeout 等待循环放行」「stdin:run_in_background 里没超时的等待循环只记不拦（退出码 0）」「stdin:检出落进检出记录」「stdin:套上 timeout 的前台等待循环放行（退出码 0）」「一百三十七：一个 & 之后 kill -0 轮询等它放行（退出码 0）」（共 10 格）
#   BASH_COMMAND_DETECTOR_BREAK=stop-at-first-refusal  main() 一条命令踩了几类拒绝时只报第一类（退回逐类试的旧样子）
#       「stdin:一条命令同时踩几类时一次列全（终止进程、放出追踪、前台等待循环）」「stdin:管道喂给 python 的 heredoc 里 open(知识库, 'w') 拒绝（退出码 2，写范围拒）」
#       「stdin:前台 timeout 600000 毫秒又 git commit --no-verify，两类一起列」（共 3 格）
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
#       （不以「扫脚本:进程组:」起头的那五格出自扫脚本那一组的 group.sh：set -m 之后放后台、不是 setsid 起的，这个开关下它不再报红，待改清单里它那一行成了一处要放行的都没有的那一种；共 10 格，实测红的格（照失败行原文））
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
#       「Bash写范围:clerk 用 insert-row.py 写知识库」「Bash写范围:clerk 用 make-swap-spec.py 的 --out 写知识库」「Bash写范围:clerk 用 replace-batch.py，规格点名知识库」「Bash写范围:clerk 脚本路径在变量里，用 replace-once.py 写知识库」「Bash写范围:clerk 脚本路径在变量里，用 insert-row.py 写知识库」
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
#       「Bash写范围:kb-scribe 的 append-record.py 记录写成没赋过值的变量，落点算不出，拒」
#       「Bash写范围:只读定义:experiment-designer 往实验页的索引写，拒」「Bash写范围:只读定义:mutation-triage 往 crates/mutations.tsv 追加，拒」「Bash写范围:只读定义:prior-art 用 sed -i 改 prior-art 的 kb 页，拒」
#       「Bash写范围:只读定义:sweep 用 sed -i 改被搜到的 records 文件，拒」「Bash写范围:只读定义:three-way-attack 写 crates，拒」「Bash写范围:只读定义:three-way-attack 写别的轮的背景材料，拒」
#       「Bash写范围:只读定义:写者锁:prior-art 写别的子 agent 持写者锁的草稿目录，拒」
#       「Bash写范围:只读定义:three-way-defense 写 crates，拒」「Bash写范围:只读定义:three-way-forward 写 records，拒」「Bash写范围:只读定义:three-way-materials 改正文，拒」「Bash写范围:三方腿:攻方写更早一轮的报告，拒」「Bash写范围:三方腿:攻方写更早一轮的模型目录里的文件，拒」「Bash写范围:三方腿:第二次同立场的攻方写第一次那条腿的报告，拒」「Bash写范围:三方腿:辩方写别的轮的报告，拒」「Bash写范围:三方腿:正推腿写别的轮的报告，拒」「Bash写范围:三方腿:派发回执读不到的攻方写别的轮的报告：只照写范围表放行，记一条检出」「Bash写范围:只读定义:写者锁:prior-art 写有锁没认领、派发回执读不到的草稿目录，放行并记一条检出」「Bash写范围:只读定义:写者锁:prior-art 写只有 .owner（别的会话的）、没有写者锁的草稿目录，拒」（只读定义那九格；共 148 格）
#       「Bash写范围:clerk 喂给 python 的 heredoc 里 for 逐个取元组，候选里有一个落在写范围外（知识库）照拒」（python 循环按候选判那一格）
#   BASH_COMMAND_DETECTOR_BREAK=scope-known-scripts-unseen  ⑫ 不认 replace-once.py、insert-row.py、replace-batch.py 的目标文件参数（只认重定向、tee、cp / mv、sed -i、python 的写法）：实测红的格（照失败行原文）
#       「Bash写范围:clerk 用 replace-once.py 写知识库（现场：越出写范围）」「Bash写范围:clerk 直接执行 replace-once.py 写知识库」「Bash写范围:clerk 先 cd 到 records 再用相对路径写知识库」「Bash写范围:kb-scribe 用 replace-once.py 写 records 拒」
#       「Bash写范围:clerk 用 insert-row.py 写知识库」「Bash写范围:clerk 用 make-swap-spec.py 的 --out 写知识库」「Bash写范围:clerk 用 replace-batch.py，规格点名知识库」「Bash写范围:clerk 脚本路径在变量里，用 replace-once.py 写知识库」「Bash写范围:clerk 脚本路径在变量里，用 insert-row.py 写知识库」
#       「Bash写范围:clerk 脚本路径在变量里，用 replace-batch.py、规格点名知识库」「Bash写范围:clerk 直接执行变量里的 replace-once.py 写知识库」「Bash写范围:clerk 的目标在没赋过值的变量里：拒」「Bash写范围:没有项目定义的内置 agent 用 replace-once.py 写仓里拒」
#       「Bash写范围:python3 起没赋过值的变量里的脚本、第一个参数落在知识库，拒」「Bash写范围:python3 起没赋过值的变量里的脚本、第一个参数落在写范围里的 records 也拒」「Bash写范围:python3 起命令替换给的脚本、第一个参数是仓根起的绝对路径，拒」「Bash写范围:目标在没赋过值的变量里，拒」
#       「Bash写范围:replace-batch.py 规格读不到，拒」「Bash写范围:范围内的链接指到仓里范围外的目录，经它 replace-once.py 写，拒」「stdin:clerk 在 Bash 里用 replace-once.py 写知识库拒绝（退出码 2）」「stdin:写范围的拒绝点名写法、目标、写范围表与出路」
#       「Bash写范围:写范围里没有 records/ 的 kb-scribe 起 ledger-append-section.py 写现行账本，拒」「Bash写范围:kb-scribe 直接执行 ledger-append-section.py 写现行账本，拒」
#       「Bash写范围:kb-scribe 的 ledger-append-section.py --root 写成没赋过值的变量，落点算不出，拒」
#       「Bash写范围:写范围里没有 records/ 的 kb-scribe 起 append-record.py 往记录追加，拒」「Bash写范围:kb-scribe 直接执行 append-record.py 往记录追加，拒」
#       「Bash写范围:kb-scribe cd 进 records 之后按相对路径起 append-record.py，拒」
#       「Bash写范围:kb-scribe 的 append-record.py 先写 --root 再写记录，落点照第一个位置参数认，拒」
#       「Bash写范围:kb-scribe 的 append-record.py 记录写成没赋过值的变量，落点算不出，拒」（ledger-append-section.py、append-record.py 一并不认；共 29 格）
#       「服务源码:工具员用 replace-once.py 改已存在的 prune-server/src/protocol.rs」「服务源码:工具员用 replace-once.py 改已存在的 gpu-verification-server/Cargo.toml」
#       「服务源码:工具员 make-swap-spec.py 的 --out 写已存在的 prune-server/src/protocol.rs」（⑭ 同一份认法；再加 3 格）
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
#   BASH_COMMAND_DETECTOR_BREAK=scope-insert-row-absent-literal-as-target  ⑫ 认 insert-row.py 的目标文件时不跳过 --absent-literal 的值（--absent-literal 写在文件前面时把那串字面当目标文件，落在写范围外就误拒）：打开后红的格（照失败行原文）
#       「Bash写范围:clerk 的 insert-row.py --absent-literal 写在文件前面，文件照认、写自己写范围里的 records 放行」（共 1 格）
#   BASH_COMMAND_DETECTOR_BREAK=scope-main-agent-judged  ⑫ 把主 agent 也当成一条写范围模式都没有的子 agent 判（主 agent 写本仓的任何文件都拒）：实测红的格（照失败行原文）
#       「Bash写范围:主 agent 写知识库放行」「Bash写范围:主 agent 的目标算不出也不判」「stdin:拒了的不记检出」「stdin:>> 追加放行（退出码 0）、不记检出」「stdin:就地改脚本拒了的不记检出」「stdin:主 agent 同一条写知识库的命令放行（退出码 0）」
#       「stdin:写到临时文件再 mv 换上放行（退出码 0）、不记检出」「stdin:前台的同一条放行（退出码 0）」「Bash写范围:主 agent 起 ledger-append-section.py 放行」「stdin:Bash heredoc 往仓里追加干净内容放行（退出码 0）」「Bash写范围:主 agent 起 append-record.py 放行」「stdin:主 agent 同一条追加两个服务源码的命令放行（退出码 0）」（共 12 格，实测，任务 6176 与 20132）
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
#       「就地改脚本:喂给 python 的 heredoc：shutil.copy 进已有目录」「就地改脚本:cd 之后喂给 python 的相对路径」「就地改脚本:喂给 python 的 heredoc：for 逐个取元组里只赋过一次的名字，open(循环变量, 'w') 盖 scratch 根下的脚本」「就地改脚本:喂给 python 的 heredoc：for 逐个取列表字面量，候选里有一个是已有脚本」「就地改脚本:喂给 python 的 heredoc：with open(os.path.join(…, 循环变量), 'w') 嵌在 for 的循环体里」「就地改脚本:写进笔记的 heredoc 正文」「stdin:run_in_background 里喂给 python 的 open(…, 'w') 同样拒绝（退出码 2）」
#       「stdin:cat 写草稿脚本的 heredoc 正文里有 sed -i 知识库放行（退出码 0）」「stdin:管道喂给 python 的 heredoc 里 open(知识库, 'w') 拒绝（退出码 2，写范围拒）」「stdin:管道喂给 grep 的 heredoc 正文放行（退出码 0）」
#       「就地改脚本:喂给 python 的 heredoc：循环体里 target = os.path.join(…, 循环变量) 只赋过一次，open(target, 'w') 经它引到循环变量」「就地改脚本:喂给 python 的 heredoc：循环体里先判 os.path.exists 只写新文件，条件不看、候选里有已有脚本照拒」「就地改脚本:喂给 python 的 heredoc：循环体里只写 .md 后缀，条件不看、候选里有已有脚本照拒」「就地改脚本:喂给 python 的 heredoc：循环变量另在 except … as 里绑定（不数进绑定次数），照按候选拒」「stdin:python 的 for 循环体里先判 os.path.exists 只写新文件照拒（退出码 2），stderr 写了这一形的出路（条件不看、Edit 工具或写临时文件再 mv）」「Bash写范围:clerk 喂给 python 的 heredoc 里 for 逐个取元组，候选里有一个落在写范围外（知识库）照拒」「往现行账本追加:喂给 python 的 heredoc 里 for 逐个取元组，候选里有一个是现行账本拒」「往 records 追加:喂给 python 的 heredoc 里 for 逐个取元组，候选里有一个落在 records 下拒」
#       「往现行账本追加:喂给 python 的 heredoc 里 open(…, 'a') 拒」「往 records 追加:喂给 python 的 heredoc 里 open(…, mode='a') 拒」「就地改脚本:python 的 for 逐个取值时，拒绝点名是已有脚本的那个候选、不点名别的候选」（共 47 格）
#   BASH_COMMAND_DETECTOR_BREAK=sed-in-place-ignores-cd  ⑧ 的 sed -i 相对路径不跟 cd、也不看 hook 输入的 cwd（cd 到草稿目录之后 sed -i 相对路径照样按仓内文件拒；也不认经管道喂给 bash 的 heredoc 里的 sed -i）
#       「就地改仓内文件:cd 到工作根下的草稿目录之后 sed -i 相对路径放行」「就地改仓内文件:cd 到 /tmp/ 下的目录之后 sed -i 相对路径放行」「就地改仓内文件:cd 到 // 开头写的 /tmp/ 下的目录之后 sed -i 相对路径放行」「就地改仓内文件:cd 到前面赋过值的草稿目录之后 sed -i 相对路径放行」「就地改仓内文件:管道喂给 bash 的 heredoc 里 sed -i 仓内文件照拒」「就地改仓内文件:命令开头的当前目录在草稿目录里，sed -i 相对路径放行」「stdin:cd 到草稿目录之后 sed -i 相对路径放行（退出码 0）」「stdin:hook 输入的 cwd 在 /tmp 下时 sed -i 相对路径放行（退出码 0）」（共 8 格；「// 开头写的」那一格是推的，没跑）
#   （其中「就地改仓内文件:管道喂给 bash 的 heredoc 里 sed -i 仓内文件照拒」在 heredoc-pipe-fed-as-data 下也红）
#   BASH_COMMAND_DETECTOR_BREAK=heredoc-pipe-fed-as-data  经管道喂给 python、bash 的 heredoc 正文与 echo / printf 的文字当数据（`cat <<'EOF' | python3 -` 的正文 ⑧ ⑫ 都不判）
#       「就地改仓内文件:管道喂给 python3 - 的 heredoc 里 open(仓内路径, 'w') 照拒」「就地改仓内文件:管道喂给 python3（不带 -）的 heredoc 里 open(仓内路径, 'w') 照拒」「就地改仓内文件:管道喂给 python3 - 的 heredoc 里 Path(仓内路径).write_text 照拒」「就地改仓内文件:echo 的文字管道喂给 python3 - 照拒」「就地改仓内文件:管道喂给 bash 的 heredoc 里 sed -i 仓内文件照拒」「Bash写范围:clerk 管道喂给 python3 - 的 heredoc 里 open(知识库, 'w') 照拒」「Bash写范围:clerk 管道喂给 python3（不带 -）的 heredoc 里 open(知识库, 'w') 照拒」「Bash写范围:clerk cd 到 records 之后管道喂给 python3 - 的 heredoc 里相对路径写知识库照拒」「Bash写范围:clerk 管道喂给 bash 的 heredoc 里 sed -i 知识库照拒」「Bash写范围:clerk 管道喂给 bash -s 的 heredoc 里重定向写知识库照拒」「Bash写范围:clerk echo 的文字管道喂给 python3 - 写知识库照拒」「Bash写范围:clerk printf 的文字管道喂给 python3 - 写知识库照拒」「stdin:管道喂给 python 的 heredoc 里 open(知识库, 'w') 拒绝（退出码 2，写范围拒）」「stdin:管道喂给 python 的 heredoc 里 open(知识库, 'w') 同样被 ⑧ 拒（有 Edit 的子 agent 就地改仓内文件）」（共 14 格）
#   BASH_COMMAND_DETECTOR_BREAK=pwd-variable-unexpanded  ⑫ 里 cd 到写死的目录之后 $PWD 不按那个目录展开（`cd <目录> && … > $PWD/x` 的目标以变量开头，只记检出）
#       「Bash写范围:clerk cd 到 records 之后写 $PWD/… 放行」「Bash写范围:cd 到写死的草稿目录之后写 $PWD/… 不记」「Bash写范围:cd 到写死的范围内目录之后写 $PWD/… 不记」「Bash写范围:cd 到写死的草稿目录之后 cp 进 $PWD/子目录/ 不记」「Bash写范围:cd 到写死的草稿目录之后 T=$PWD/x 再写 $T/… 不记」「stdin:cd 之后的 $PWD/… 与 mktemp -d 赋的变量放行（退出码 0）、不记检出」（共 6 格）
#   BASH_COMMAND_DETECTOR_BREAK=mktemp-directory-unknown  ⑫ 里 `d=$(mktemp -d)` 赋的变量算不出（`> $d/x` 的目标以变量开头，只记检出）
#       「Bash写范围:clerk mktemp -d 建的变量目录里写放行」「Bash写范围:clerk mktemp -d 之后 cd 进去再写相对路径放行」「Bash写范围:W=$(mktemp -d) 之后 cp 进 $W/ 不记」「Bash写范围:W=`mktemp -d`（反引号）之后写 $W/… 不记」「Bash写范围:export W=$(mktemp -d) 之后写 $W/… 不记」「Bash写范围:mktemp -d 之后 cd 进去写相对路径不记」「Bash写范围:mktemp -d -t 带模板不记」「Bash写范围:mktemp --directory 不记」「stdin:cd 之后的 $PWD/… 与 mktemp -d 赋的变量放行（退出码 0）、不记检出」「Bash写范围:不带值的 --tmpdir：W=$(mktemp -d --tmpdir) 之后写进 $W/ 不记」「Bash写范围:套一层、不带 -p：T=$(cd $(mktemp -d) && pwd -P) 之后写进 $T/ 不记」（共 11 格）
#   （「stdin:cd 之后的 $PWD/… 与 mktemp -d 赋的变量放行（退出码 0）、不记检出」在 pwd-variable-unexpanded、mktemp-directory-unknown 与 literal-notations-outside-judged 下都红）
#   BASH_COMMAND_DETECTOR_BREAK=mktemp-scratch-template-unknown  ⑫ 里不带 -t 的 mktemp 模板一律算不出，模板目录落在自己认领的草稿目录里也一样
#       「Bash写范围:模板在自己认领的草稿目录里：T=$(mktemp -d $W/名.XXXXXX) 之后 cp 进 $T/ 不记」「Bash写范围:模板在自己认领的草稿目录里：直接写进 $(mktemp -d …)/文件 不记」「Bash写范围:cd 进自己认领的草稿目录之后模板写相对路径不记」「Bash写范围:还没有认领文件、派发回执对得上那把锁：模板在它的草稿目录里不记」「stdin:模板在自己认领的草稿目录里的 mktemp -d 放行（退出码 0）、不记检出」（共 5 格）
#   BASH_COMMAND_DETECTOR_BREAK=mktemp-template-owner-unchecked  own_scratch_directory() 不看归属文件与写者锁，工作根下的目录一律当自己认领的
#       「Bash写范围:派发回执对不上那把锁（别的子 agent）：拒」「Bash写范围:模板在别的子 agent 认领的草稿目录里，拒」「Bash写范围:模板目录的归属文件是别的会话，拒」「Bash写范围:-p 指到别的子 agent 认领的草稿目录，拒」「Bash写范围:--tmpdir= 指到别的会话的草稿目录，拒」「Bash写范围:套一层、-p 指到别的子 agent 认领的草稿目录，拒」（共 6 格）
#   BASH_COMMAND_DETECTOR_BREAK=mktemp-option-directory-unknown  ⑫ 里 `mktemp -d -p <目录>`、`--tmpdir=<目录>` 与不带值的 `--tmpdir` 一概算不出
#       「Bash写范围:-p 指到自己认领的草稿目录：T=$(mktemp -d -p "$W") 之后写进 $T/ 不记」「Bash写范围:--tmpdir= 指到自己认领的草稿目录（带模板名）：cp 进 $T/ 不记」「Bash写范围:cd 进自己认领的草稿目录之后 -p . 不记」「Bash写范围:直接写进 $(mktemp -d -p …)/文件 不记」「Bash写范围:不带值的 --tmpdir：W=$(mktemp -d --tmpdir) 之后写进 $W/ 不记」「Bash写范围:套一层 cd 加 pwd -P：T=$(cd $(mktemp -d -p "$W") && pwd -P) 之后写进 $T/ 不记」「Bash写范围:套一层、反引号写法：T=`cd $(mktemp -d -p …) && pwd` 之后写进 $T/ 不记」「Bash写范围:套一层、直接写进 $(cd $(mktemp -d -p …) && pwd -P)/文件 不记」（共 8 格）
#   BASH_COMMAND_DETECTOR_BREAK=mktemp-nested-substitution-unknown  ⑫ 里 `$(cd $(mktemp …) && pwd -P)` 这种套在别的命令替换里的 mktemp 算不出
#       「Bash写范围:套一层 cd 加 pwd -P：T=$(cd $(mktemp -d -p "$W") && pwd -P) 之后写进 $T/ 不记」「Bash写范围:套一层、不带 -p：T=$(cd $(mktemp -d) && pwd -P) 之后写进 $T/ 不记」「Bash写范围:套一层、反引号写法：T=`cd $(mktemp -d -p …) && pwd` 之后写进 $T/ 不记」「Bash写范围:套一层、直接写进 $(cd $(mktemp -d -p …) && pwd -P)/文件 不记」（共 4 格）
#   BASH_COMMAND_DETECTOR_BREAK=break-honored-outside-selftest  平常跑设了弄坏开关也照认，不拒；它是关守卫项，只在单独一项时认，与别的项同写照拒
#       「平常跑设了弄坏开关：退 2 并点名只在自证里认」「平常跑设了共用库的弄坏开关 LIB_CODE_DISCIPLINE_LINTS_BREAK：退 2 并点名只在自证里认」（共 2 格）
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
#   BASH_COMMAND_DETECTOR_BREAK=allow-long-foreground-timeout  ㉑ 前台 timeout 超过 240000 毫秒也放行
#       「前台超时:主 agent 前台 timeout 600000 毫秒拒」「前台超时:主 agent 前台 timeout 240001 毫秒拒」「前台超时:子 agent 前台 timeout 600000 毫秒拒」「前台超时:主 agent 前台 timeout 写成字符串 300000 也拒」
#       「stdin:主 agent 前台 timeout 600000 毫秒拒绝（退出码 2）、出路写 run_in_background 与 task-server/task wait」「stdin:子 agent 前台 timeout 600000 毫秒拒绝（退出码 2）、出路写进报告交回主 agent」
#       「stdin:主 agent 前台 timeout 240001 毫秒拒绝（退出码 2）」「stdin:前台 timeout 600000 毫秒又 git commit --no-verify，两类一起列」（共 8 格）
#   BASH_COMMAND_DETECTOR_BREAK=allow-git-hook-guard  ㉒ 一概放行（绕开钩子的写法、没装钩子的仓里的 git commit / git push 都不拒）
#       「提交前钩子:git commit --no-verify 拒」「提交前钩子:git commit -n（--no-verify 的短写）拒」「提交前钩子:git commit -nm 合写的 -n 也认，拒」
#       「提交前钩子:git commit --no-veri 是 --no-verify 的缩写，拒」「提交前钩子:git push --no-verify 拒」「提交前钩子:命令行 -c core.hooksPath=… 的 git commit 拒」
#       「提交前钩子:命令行 -c core.hooksPath=… 的 git push 拒」「提交前钩子:GIT_CONFIG_KEY_0=core.hooksPath 的 git commit 拒」「提交前钩子:git config 改 core.hooksPath 再 git commit 拒」
#       「提交前钩子:命令行 -c alias.ci=commit --no-verify 写的别名认成 git commit，拒」「提交前钩子:bash -c 里的 git commit --no-verify 拒」「提交前钩子:echo | xargs git commit -n 拒」
#       「提交前钩子:find -exec git commit -n 拒」「提交前钩子:钩子没装上（core.hooksPath 没设）的仓里 git commit 拒」「提交前钩子:钩子没装上的仓里 git push 拒」
#       「提交前钩子:core.hooksPath 指到 .git/hooks 的仓里 git commit 拒」「提交前钩子:钩子文件没有执行位的仓里 git commit 拒」「提交前钩子:git -C 点名没装钩子的仓、起点在别处，拒」
#       「提交前钩子:cd 进没装钩子的仓之后 git commit 拒」「提交前钩子:仓里配的别名 git ci（alias.ci=commit -q）在没装钩子的仓里认成 git commit，拒」「stdin:主 agent 的 git commit --no-verify 拒绝（退出码 2），出路写去掉它与窗口外提交的环境变量」
#       「提交前钩子:写拒绝说明那一步出错（项缺说明）折成判不了：不拒、记一条检出」
#       「stdin:子 agent 在没装钩子的仓里 git commit 拒绝（退出码 2），出路写进报告交主 agent」「stdin:主 agent 在没装钩子的仓里 git push 拒绝（退出码 2），出路给任务服务装钩子的整条命令」「stdin:前台 timeout 600000 毫秒又 git commit --no-verify，两类一起列」「提交前钩子:git merge --no-verify 拒」「提交前钩子:命令行 -c core.hooksPath=… 的 git merge 拒」「提交前钩子:git config --remove-section core 再 git commit 拒」「提交前钩子:git config --rename-section core moved 再 git commit 拒」「提交前钩子:git config --remove-sec core（--remove-section 的缩写）拒」「提交前钩子:git config remove-section core（子命令写法）拒」「提交前钩子:git config --add core.hooksPath <进仓钩子目录的绝对路径> 照拒（再加一个值）」「提交前钩子:git am --no-verify 拒」「提交前钩子:git am -n 是 --no-verify 的短写，拒」「提交前钩子:git am -3n 合写的 -n 也认，拒」「提交前钩子:git am --no-veri 是 --no-verify 的缩写，拒」「提交前钩子:命令行 -c core.hooksPath=… 的 git am 拒」「stdin:主 agent 的 git am --no-verify 拒绝（退出码 2），出路写去掉它」「提交前钩子:git config core.hooksPath 设成进仓钩子目录的相对路径，拒」「提交前钩子:命令行 -c core.hooksPath=<进仓钩子目录的相对路径> 的 git commit 拒」「提交前钩子:GIT_CONFIG_KEY_0=core.hooksPath 加 GIT_CONFIG_VALUE_0=<进仓钩子目录的相对路径> 的 git commit 拒」（共 41 格）
#   BASH_COMMAND_DETECTOR_BREAK=bypass-flag-unseen  ㉒ 认不出 --no-verify 与 git commit 的 -n
#       「提交前钩子:git commit --no-verify 拒」「提交前钩子:git commit -n（--no-verify 的短写）拒」「提交前钩子:git commit -nm 合写的 -n 也认，拒」
#       「提交前钩子:git commit --no-veri 是 --no-verify 的缩写，拒」「提交前钩子:git push --no-verify 拒」「提交前钩子:命令行 -c alias.ci=commit --no-verify 写的别名认成 git commit，拒」
#       「提交前钩子:bash -c 里的 git commit --no-verify 拒」「提交前钩子:echo | xargs git commit -n 拒」「提交前钩子:find -exec git commit -n 拒」
#       「stdin:主 agent 的 git commit --no-verify 拒绝（退出码 2），出路写去掉它与窗口外提交的环境变量」「stdin:前台 timeout 600000 毫秒又 git commit --no-verify，两类一起列」「提交前钩子:git merge --no-verify 拒」
#       「提交前钩子:git am --no-verify 拒」「提交前钩子:git am -n 是 --no-verify 的短写，拒」「提交前钩子:git am -3n 合写的 -n 也认，拒」「提交前钩子:git am --no-veri 是 --no-verify 的缩写，拒」
#       「stdin:主 agent 的 git am --no-verify 拒绝（退出码 2），出路写去掉它」（共 17 格）
#   BASH_COMMAND_DETECTOR_BREAK=bypass-config-unseen  ㉒ 认不出改 core.hooksPath 的几种（-c、--config-env、GIT_CONFIG_KEY_<n>、GIT_CONFIG_PARAMETERS、git config 写它）
#       「提交前钩子:命令行 -c core.hooksPath=… 的 git commit 拒」「提交前钩子:命令行 -c core.hooksPath=… 的 git push 拒」「提交前钩子:GIT_CONFIG_KEY_0=core.hooksPath 的 git commit 拒」
#       「提交前钩子:git config 改 core.hooksPath 再 git commit 拒」「提交前钩子:命令行 -c core.hooksPath=… 的 git merge 拒」「提交前钩子:命令行 -c core.hooksPath=… 的 git am 拒」「提交前钩子:git config --add core.hooksPath <进仓钩子目录的绝对路径> 照拒（再加一个值）」
#       「提交前钩子:git config core.hooksPath 设成进仓钩子目录的相对路径，拒」「提交前钩子:命令行 -c core.hooksPath=<进仓钩子目录的相对路径> 的 git commit 拒」「提交前钩子:GIT_CONFIG_KEY_0=core.hooksPath 加 GIT_CONFIG_VALUE_0=<进仓钩子目录的相对路径> 的 git commit 拒」（共 10 格）
#   BASH_COMMAND_DETECTOR_BREAK=merge-bypass-unseen  ㉒ 不判 git merge（git merge --no-verify、命令行 -c core.hooksPath=… 的 git merge 放过去）
#       「提交前钩子:git merge --no-verify 拒」「提交前钩子:命令行 -c core.hooksPath=… 的 git merge 拒」（共 2 格）
#   BASH_COMMAND_DETECTOR_BREAK=am-bypass-unseen  ㉒ 不判 git am（git am --no-verify、-n、命令行 -c core.hooksPath=… 的 git am 放过去）
#       「提交前钩子:git am --no-verify 拒」「提交前钩子:git am -n 是 --no-verify 的短写，拒」「提交前钩子:git am -3n 合写的 -n 也认，拒」「提交前钩子:git am --no-veri 是 --no-verify 的缩写，拒」
#       「提交前钩子:命令行 -c core.hooksPath=… 的 git am 拒」「stdin:主 agent 的 git am --no-verify 拒绝（退出码 2），出路写去掉它」（共 6 格）
#   BASH_COMMAND_DETECTOR_BREAK=config-section-unseen  ㉒ 不认 git config 的 --remove-section core 与 --rename-section core（缩写与子命令写法同）
#       「提交前钩子:git config --remove-section core 再 git commit 拒」「提交前钩子:git config --rename-section core moved 再 git commit 拒」「提交前钩子:git config --remove-sec core（--remove-section 的缩写）拒」「提交前钩子:git config remove-section core（子命令写法）拒」（共 4 格）
#   BASH_COMMAND_DETECTOR_BREAK=hooks-path-right-value-refused  ㉒ 把 core.hooksPath 设成装对的值（git config、-c、GIT_CONFIG_KEY_<n> 加 GIT_CONFIG_VALUE_<n>）也拒
#       「提交前钩子:git config core.hooksPath 设成进仓钩子目录的绝对路径，放行」「提交前钩子:命令行 -c core.hooksPath=<进仓钩子目录的绝对路径> 的 git commit 放行」「提交前钩子:GIT_CONFIG_KEY_0=core.hooksPath 加 GIT_CONFIG_VALUE_0=<进仓钩子目录的绝对路径> 的 git commit 放行」（共 3 格）
#   BASH_COMMAND_DETECTOR_BREAK=hooks-path-relative-accepted  ㉒ 相对路径的 core.hooksPath 值（git config、-c、GIT_CONFIG_KEY_<n> 加 GIT_CONFIG_VALUE_<n>）也从这棵树的顶层目录起算、算装对的而放行
#       「提交前钩子:git config core.hooksPath 设成进仓钩子目录的相对路径，拒」「提交前钩子:命令行 -c core.hooksPath=<进仓钩子目录的相对路径> 的 git commit 拒」「提交前钩子:GIT_CONFIG_KEY_0=core.hooksPath 加 GIT_CONFIG_VALUE_0=<进仓钩子目录的相对路径> 的 git commit 拒」（共 3 格）
#   BASH_COMMAND_DETECTOR_BREAK=hooks-not-installed-allowed  ㉒ 不判 git 钩子装没装上（core.hooksPath 没指到进仓的钩子目录、钩子文件缺、没有执行位也放行）
#       「提交前钩子:钩子没装上（core.hooksPath 没设）的仓里 git commit 拒」「提交前钩子:钩子没装上的仓里 git push 拒」「提交前钩子:core.hooksPath 指到 .git/hooks 的仓里 git commit 拒」
#       「提交前钩子:钩子文件没有执行位的仓里 git commit 拒」「提交前钩子:git -C 点名没装钩子的仓、起点在别处，拒」「提交前钩子:cd 进没装钩子的仓之后 git commit 拒」
#       「提交前钩子:仓里配的别名 git ci（alias.ci=commit -q）在没装钩子的仓里认成 git commit，拒」「stdin:子 agent 在没装钩子的仓里 git commit 拒绝（退出码 2），出路写进报告交主 agent」「stdin:主 agent 在没装钩子的仓里 git push 拒绝（退出码 2），出路给任务服务装钩子的整条命令」（共 9 格）
#   BASH_COMMAND_DETECTOR_BREAK=bypass-every-git-command  ㉒ 只读的 git log、git status 这类也按 git commit 判（git log -n 5 的 -n 当成 git commit -n）
#       「提交前钩子:git log -n 5 只读，放行」（共 1 格）
#   BASH_COMMAND_DETECTOR_BREAK=alias-unresolved  ㉒ 不查别名（git ci 这类别名认不成 git commit）
#       「提交前钩子:命令行 -c alias.ci=commit --no-verify 写的别名认成 git commit，拒」「提交前钩子:仓里配的别名 git ci（alias.ci=commit -q）在没装钩子的仓里认成 git commit，拒」（共 2 格）
#   BASH_COMMAND_DETECTOR_BREAK=dynamic-launcher-unseen  ㉒ 不剥启动器（xargs、find -exec 起的 git commit -n 看不见）
#       「提交前钩子:echo | xargs git commit -n 拒」「提交前钩子:find -exec git commit -n 拒」（共 2 格）
#   BASH_COMMAND_DETECTOR_BREAK=long-option-prefix-unseen  ㉒ 长选项只认全名（--no-veri 这类缩写认不出，当成认不出的选项）
#       「提交前钩子:git commit --no-veri 是 --no-verify 的缩写，拒」「提交前钩子:git am --no-veri 是 --no-verify 的缩写，拒」（共 2 格）
#   BASH_COMMAND_DETECTOR_BREAK=git-hook-messages-unguarded  ㉒ 写拒绝说明那一步出错不接，异常往外抛（别的道的拒绝一并丢）
#       「提交前钩子:写拒绝说明那一步出错（项缺说明）折成判不了：不拒、记一条检出」（共 1 格）
#   BASH_COMMAND_DETECTOR_BREAK=scope-writer-lock-unchecked  ⑫ 只带 Read、Bash 的定义写工作根下别的子 agent 持写者锁的草稿目录也放行（不照写者锁判）
#       「Bash写范围:只读定义:写者锁:prior-art 写别的子 agent 持写者锁的草稿目录，拒」「Bash写范围:只读定义:写者锁:prior-art 写有锁没认领、派发回执读不到的草稿目录，放行并记一条检出」「Bash写范围:只读定义:写者锁:prior-art 写只有 .owner（别的会话的）、没有写者锁的草稿目录，拒」（共 3 格）
#   BASH_COMMAND_DETECTOR_BREAK=scope-leg-round-unchecked  ⑫ 三方腿往 research/prompts/ 写不收窄到派发回执里「报告：」那一行点名的这一轮（只照写范围表的形态判），回执读不到也不记检出
#       「Bash写范围:三方腿:攻方写更早一轮的报告，拒」「Bash写范围:三方腿:攻方写更早一轮的模型目录里的文件，拒」「Bash写范围:三方腿:第二次同立场的攻方写第一次那条腿的报告，拒」「Bash写范围:三方腿:辩方写别的轮的报告，拒」「Bash写范围:三方腿:正推腿写别的轮的报告，拒」「Bash写范围:三方腿:派发回执读不到的攻方写别的轮的报告：只照写范围表放行，记一条检出」（共 6 格）
#   BASH_COMMAND_DETECTOR_BREAK=scope-owner-only-directory-unjudged  ⑫ 只带 Read、Bash 的定义写工作根下只有 .owner（别的会话的）、没有写者锁的草稿目录也放行
#       「Bash写范围:只读定义:写者锁:prior-art 写只有 .owner（别的会话的）、没有写者锁的草稿目录，拒」（共 1 格）
#   BASH_COMMAND_DETECTOR_BREAK=scope-unclaimed-receipt-unrecorded  ⑫ 只带 Read、Bash 的定义写有锁没认领、派发回执读不到（unclaimed）的草稿目录，放行而不记检出
#       「Bash写范围:只读定义:写者锁:prior-art 写有锁没认领、派发回执读不到的草稿目录，放行并记一条检出」（共 1 格）
#   BASH_COMMAND_DETECTOR_BREAK=allow-whole-target-build  ㉓ 交给任务服务的 cargo build 点了全部目标也放行（拒绝说明不打，出路过别的闸那几格取不出写法）
#       「全部目标构建:submit 交 cargo build --all-targets 拒」「全部目标构建:submit 交 cargo build --tests 拒」「全部目标构建:submit 交 cargo build --benches 拒」
#       「全部目标构建:submit 交 cargo build --examples 拒」「全部目标构建:submit 交 cargo build --workspace 拒」「全部目标构建:submit 交 cargo build --all 拒」
#       「全部目标构建:submit 交 cargo b --all-targets（内建别名）拒」「全部目标构建:范围声明写在前面、--cwd 点名目录的 submit 照拒」「全部目标构建:submit 之后 env 前缀里的 cargo build 照拒」
#       「全部目标构建:submit 之后 bash -c 里 cd 再 cargo build 照拒」「全部目标构建:bash -c 里的 submit 照拒」「全部目标构建:&& 连着的后一段 submit 照拒」
#       「全部目标构建:只带 --whole-package-confirmed 的 submit 照拒」「全部目标构建:--whole-package-confirmed 的值写成 --full-gate 不算带了 --full-gate，照拒」
#       「stdin:主 agent 经 submit 交 cargo build --all-targets 拒（退出码 2）、出路给三种 cargo 写法」「stdin:子 agent 经 submit 交 cargo build --tests 拒（退出码 2）、出路写进报告交回」
#       「出路过别的闸:主 agent 照抄 ㉓ 出路的 cargo clippy 写法喂 bash-command-detector.sh 放行（退出码 0）」「出路过别的闸:主 agent 照抄 ㉓ 出路的 cargo clippy 写法喂 project-code-guard.sh 放行（退出码 0）」
#       「出路过别的闸:主 agent 照抄 ㉓ 出路的 cargo clippy 写法喂 run-scope-guard.sh 放行（退出码 0）」「出路过别的闸:主 agent 照抄 ㉓ 出路的 cargo test 写法喂 bash-command-detector.sh 放行（退出码 0）」
#       「出路过别的闸:主 agent 照抄 ㉓ 出路的 cargo test 写法喂 project-code-guard.sh 放行（退出码 0）」「出路过别的闸:主 agent 照抄 ㉓ 出路的 cargo test 写法喂 run-scope-guard.sh 放行（退出码 0）」
#       「出路过别的闸:主 agent 照抄 ㉓ 出路的 cargo build 写法喂 bash-command-detector.sh 放行（退出码 0）」「出路过别的闸:主 agent 照抄 ㉓ 出路的 cargo build 写法喂 project-code-guard.sh 放行（退出码 0）」
#       「出路过别的闸:主 agent 照抄 ㉓ 出路的 cargo build 写法喂 run-scope-guard.sh 放行（退出码 0）」（共 25 格）
#   BASH_COMMAND_DETECTOR_BREAK=whole-target-full-gate-unseen  ㉓ 不认 submit 的 --full-gate（提交时的整轮门禁交的 cargo build --all-targets 也拒）
#       「全部目标构建:submit 带 --full-gate 交 cargo build --all-targets 放行」「全部目标构建:--cwd 写在 --full-gate 前面也认，放行」
#       「stdin:主 agent 带 --full-gate 交 cargo build --all-targets 放行（退出码 0）」（共 3 格）
#   BASH_COMMAND_DETECTOR_BREAK=whole-target-howto-unnamed-test  ㉓ 出路里的 cargo test 只点目标、不点名测试全名（照抄交上去被 run-scope-guard.sh 按经 submit 交的要点名拒）
#       「出路过别的闸:主 agent 照抄 ㉓ 出路的 cargo test 写法喂 run-scope-guard.sh 放行（退出码 0）」（共 1 格）
#   BASH_COMMAND_DETECTOR_BREAK=whole-target-howto-clippy-lints-dropped  ㉓ 出路里的 cargo clippy 不带 check.sh 的 lint 表、说明里也不提（照抄交上去被 project-code-guard.sh 的 clippy lint 表那一道拒）
#       「出路过别的闸:主 agent 照抄 ㉓ 出路的 cargo clippy 写法喂 project-code-guard.sh 放行（退出码 0）」「㉓ 出路:cargo clippy 写法的 -- 之后带 check.sh 现在的 CODE_DISCIPLINE_LINTS 整段（-D warnings 加每一条 -D clippy::<名>）」（共 2 格）
#   LIB_CODE_DISCIPLINE_LINTS_BREAK=entry-form-unchecked  共用模块 lib_code_discipline_lints.py 读 check.sh 的 lint 表时每一行照 shell 切词、切出来的词都收（不查 -D clippy::<名> 一行一条，开头那一行 ( 之后的元素也收）
#       「㉓ 出路的 lint 表:第一条写在开头那一行，读不成：交空串并记一条判不了」「㉓ 出路的 lint 表:一行写两条，读不成：交空串并记一条判不了」「㉓ 出路的 lint 表:-D 与名字贴着写，读不成：交空串并记一条判不了」「㉓ 出路的 lint 表:--deny 写法，读不成：交空串并记一条判不了」「㉓ 出路的 lint 表:加了一条不是 clippy 的 rustc lint，读不成：交空串并记一条判不了」「㉓ 出路的 lint 表:引号里写的 -D 与名字，读不成：交空串并记一条判不了」「㉓ 出路的 lint 表:花括号，读不成：交空串并记一条判不了」「㉓ 出路的 lint 表:数组元素，读不成：交空串并记一条判不了」「㉓ 出路的 lint 表:-F 禁止，读不成：交空串并记一条判不了」（共 9 格）
#   LIB_CODE_DISCIPLINE_LINTS_BREAK=other-assignments-unscanned  共用模块 lib_code_discipline_lints.py 不查 check.sh 里另有没有写 CODE_DISCIPLINE_LINTS 的地方（+=、再次赋值、元素赋值照旧不看），只读第一处开头
#       「㉓ 出路的 lint 表:+= 追加，读不成：交空串并记一条判不了」「㉓ 出路的 lint 表:重新赋值，读不成：交空串并记一条判不了」「㉓ 出路的 lint 表:元素赋值，读不成：交空串并记一条判不了」（共 3 格）
#   BASH_COMMAND_DETECTOR_BREAK=python-move-into-repository-unseen  ⑧ 的 python 一半不认 os.replace / os.rename / shutil.move 的目标，前缀不含 scripts/
#       「就地改仓内文件:执行员 python os.replace 把草稿挪进 scripts/ 下的入库装置」「就地改仓内文件:执行员 python os.rename 挪进知识库（源是变量）」
#       「就地改仓内文件:执行员 python shutil.move 挪进仓根起的绝对路径」「就地改仓内文件:执行员 python shutil.move 用 dst= 挪进 scripts/」
#       「就地改仓内文件:执行员 python open(scripts/ 下的文件, 'w')」（共 5 格）
#       python 脚本正文那几格：「就地改仓内文件:python3 起草稿目录里的脚本按字面量改入库装置照拒」
#       「就地改仓内文件:python3 起草稿目录里的脚本用 os.replace 挪进 scripts/ 照拒」「就地改仓内文件:nice 前缀、带选项与参数起草稿目录里的脚本照拒」
#       「就地改仓内文件:bash -c 里起草稿目录里的脚本照拒」（共 4 格）
#   BASH_COMMAND_DETECTOR_BREAK=python-relative-path-ignores-cd  ⑧ 的 python 一半相对路径不跟 cd、一律按仓内判（cd 进草稿目录里的仓副本再写 crates/…、scripts/… 也拒）：实测红的格（照失败行原文）
#       「就地改仓内文件:cd 到草稿目录里的仓副本之后 python 相对路径写 crates/ 放行」「就地改仓内文件:cd 到草稿目录里的仓副本之后 python 相对路径挪进 scripts/ 放行」「就地改仓内文件:cd 到草稿目录里的仓副本之后喂给 python 的 heredoc 相对路径写 scripts/ 放行」（共 3 格）
#       python 脚本正文那一格：「就地改仓内文件:cd 进草稿目录之后起脚本，相对路径落在草稿目录放行」（共 1 格）
#   BASH_COMMAND_DETECTOR_BREAK=perl-in-place-unseen  ⑧ 不认 perl -i（repository_in_place_edit_verdict 的写法里去掉 perl -i；⑨ ⑫ 照认）：实测红的格（照失败行原文）
#       「就地改仓内文件:perl -pi -e 改仓内知识库」「就地改仓内文件:perl -i -pe 改入库装置」「就地改仓内文件:perl -i.bak 带备份后缀」「就地改仓内文件:perl -pi.bak -e 合写带备份后缀」
#       「就地改仓内文件:perl -pie 是备份后缀为 e 的 -i，后面的文件照拒」「就地改仓内文件:perl -lpi 合写照拒」「就地改仓内文件:perl -0777 -i -pe 照拒」「就地改仓内文件:bash -c 里的 perl -pi 照拒」（共 8 格）
#   BASH_COMMAND_DETECTOR_BREAK=perl-switch-cluster-by-regex  perl_in_place_files 不按开关簇认，照 ^-[a-zA-Z]*i 认 -i（-Ilib、-MList::Util 里的 i 也当 -i）：实测红的格（照失败行原文）
#       「就地改仓内文件:perl -Ilib -ne 只读不拦（-I 簇里的 i 不是 -i）」「就地改仓内文件:perl -MList::Util=sum -ne 只读不拦（-M 簇里的 i 不是 -i）」（共 2 格）
#   BASH_COMMAND_DETECTOR_BREAK=python-script-body-unread  ⑧ 不读 python3 起的脚本正文（python_script_bodies 一份都不交）：实测红的格（照失败行原文）
#       「就地改仓内文件:python3 起草稿目录里的脚本按字面量改入库装置照拒」「就地改仓内文件:python3 起草稿目录里的脚本按仓根起的绝对路径写知识库照拒」
#       「就地改仓内文件:python3 起草稿目录里的脚本用 os.replace 挪进 scripts/ 照拒」「就地改仓内文件:nice 前缀、带选项与参数起草稿目录里的脚本照拒」
#       「就地改仓内文件:bash -c 里起草稿目录里的脚本照拒」「就地改仓内文件:脚本的目标取自 sys.argv，记一条检出」「就地改仓内文件:脚本的目标是拼出来的变量，记一条检出」（共 7 格）
#   BASH_COMMAND_DETECTOR_BREAK=python-script-any-location-read  python_script_bodies 不看脚本落在哪个根底下，仓里的脚本也读：实测红的格（照失败行原文）
#       「就地改仓内文件:仓里的脚本不读（定点写入脚本这类）」（共 1 格）
#   BASH_COMMAND_DETECTOR_BREAK=python-script-unliteral-write-unnoted  脚本正文里写文件的目标不是字面量时不记检出（repository_in_place_edit_verdict 的 noted 不收）：实测红的格（照失败行原文）
#       「就地改仓内文件:脚本的目标取自 sys.argv，记一条检出」「就地改仓内文件:脚本的目标是拼出来的变量，记一条检出」（共 2 格）
#   BASH_COMMAND_DETECTOR_BREAK=scope-script-variable-unexpanded  ⑫ 的脚本词不代入前面赋过的值，代入不出的也不交 UnresolvedScriptTarget
#       「Bash写范围:clerk 脚本路径在变量里，用 replace-once.py 写知识库」「Bash写范围:clerk 脚本路径在变量里，用 insert-row.py 写知识库」
#       「Bash写范围:clerk 脚本路径在变量里，用 replace-batch.py、规格点名知识库」「Bash写范围:clerk 直接执行变量里的 replace-once.py 写知识库」
#       「Bash写范围:python3 起没赋过值的变量里的脚本、第一个参数落在知识库，拒」「Bash写范围:python3 起没赋过值的变量里的脚本、第一个参数落在写范围里的 records 也拒」
#       「Bash写范围:python3 起命令替换给的脚本、第一个参数是仓根起的绝对路径，拒」（共 7 格）
#   BASH_COMMAND_DETECTOR_BREAK=builtin-agents-unjudged  ⑫ 没有项目定义的 agent 一概不判（内置类型、定义已删的旧实例写仓里、写仓外别处都放过）：实测红的格（照失败行原文）
#       「Bash写范围:没有项目定义的内置 agent 写仓里拒」「Bash写范围:没有项目定义的内置 agent 写仓外别处拒」「Bash写范围:没有项目定义的内置 agent 写会话临时目录根拒（scratchpad 之外）」「Bash写范围:没有项目定义的内置 agent 写会话 scratchpad 拒」「Bash写范围:没有项目定义的内置 agent 用 replace-once.py 写仓里拒」「Bash写范围:定义已删的旧实例写仓里拒」（共 6 格）
#   BASH_COMMAND_DETECTOR_BREAK=scope-read-only-definitions-unjudged  ⑫ 只认 tools 里有 Write、Edit 的定义在写范围表里的行：只带 Read、Bash 的八类定义的行读成没有，一概不判
#       「Bash写范围:只读定义:experiment-designer 往实验页的索引写，拒」「Bash写范围:只读定义:mutation-triage 往 crates/mutations.tsv 追加，拒」「Bash写范围:只读定义:prior-art 用 sed -i 改 prior-art 的 kb 页，拒」
#       「Bash写范围:只读定义:sweep 用 sed -i 改被搜到的 records 文件，拒」「Bash写范围:只读定义:three-way-attack 写 crates，拒」「Bash写范围:只读定义:three-way-attack 写别的轮的背景材料，拒」
#       「Bash写范围:只读定义:写者锁:prior-art 写别的子 agent 持写者锁的草稿目录，拒」
#       「Bash写范围:只读定义:three-way-defense 写 crates，拒」「Bash写范围:只读定义:three-way-forward 写 records，拒」「Bash写范围:只读定义:three-way-materials 改正文，拒」「Bash写范围:三方腿:攻方写更早一轮的报告，拒」「Bash写范围:三方腿:攻方写更早一轮的模型目录里的文件，拒」「Bash写范围:三方腿:第二次同立场的攻方写第一次那条腿的报告，拒」「Bash写范围:三方腿:辩方写别的轮的报告，拒」「Bash写范围:三方腿:正推腿写别的轮的报告，拒」「Bash写范围:三方腿:派发回执读不到的攻方写别的轮的报告：只照写范围表放行，记一条检出」「Bash写范围:只读定义:写者锁:prior-art 写有锁没认领、派发回执读不到的草稿目录，放行并记一条检出」「Bash写范围:只读定义:写者锁:prior-art 写只有 .owner（别的会话的）、没有写者锁的草稿目录，拒」（共 18 格）
#   BASH_COMMAND_DETECTOR_BREAK=break-switch-sets-one-way  弄坏开关四处对账的自证只查名单里的名字在另几处有没有（文件头、代码、各节里有而名单没有的不查）
#       「弄坏开关三处一致:文件头有、名单没有」「弄坏开关三处一致:代码读了、名单没有」「弄坏开关三处一致:文件头与代码有、名单没有」「弄坏开关三处一致:单引号写的读法」「弄坏开关三处一致:经常量名读」「弄坏开关三处一致:bash 层 == 读」「弄坏开关三处一致:各节写着已经删掉的开关」（共 7 格）
#   BASH_COMMAND_DETECTOR_BREAK=break-switch-indirect-reads-unseen  弄坏开关四处对账的自证里，代码里读的名字只认 detector_break_is_set("<项>") 的双引号字面（单引号、常量名、GUARD_OFF_ITEM 一族的值、bash 层 ==、!= 里的项不认）
#       「弄坏开关三处一致:这份钩子的名单、文件头开关表、代码里读的名字、各节弄坏开关一串对得上」「弄坏开关三处一致:单引号写的读法」「弄坏开关三处一致:经常量名读」「弄坏开关三处一致:bash 层 == 读」「弄坏开关三处一致:读的名字认不出」「弄坏开关三处一致:常量、bash 层读的名字也算读」（共 6 格）
#   BASH_COMMAND_DETECTOR_BREAK=break-switch-table-narrow  弄坏开关四处对账的自证里，文件头开关表照一行一个名字读（不认逗号隔开的几项、批量登记那一段的行首写法限定在那一段、不点名读不成一条的行）
#       「弄坏开关三处一致:逗号隔开的几项各算一条」「弄坏开关三处一致:同一行写第二条的点名那一行」「弄坏开关三处一致:名字套反引号的点名那一行」「弄坏开关三处一致:等号后换行的点名那一行」「弄坏开关三处一致:缩进两个空格与不缩进的行都认」「弄坏开关三处一致:说明行写着带连字符的词不当开关」（共 6 格）
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
#       「往现行账本追加:喂给 python 的 heredoc 里 for 逐个取元组，候选里有一个是现行账本拒」
#       「stdin:子 agent 往现行账本追加节标题拒、出路写经 ledger-append-section.py（退出码 2）」（共 9 格）
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
#       「Bash写范围:只读定义:写者锁:prior-art 写别的子 agent 持写者锁的草稿目录，拒」
#       ⑧ 认 perl -i 那两格：「就地改仓内文件:perl -pi 改工作根下的草稿目录不拦」「就地改仓内文件:cd 到工作根下的草稿目录之后 perl -pi 相对路径不拦」（共 2 格）
#       「Bash写范围:只读定义:写者锁:prior-art 写有锁没认领、派发回执读不到的草稿目录，放行并记一条检出」「Bash写范围:只读定义:写者锁:prior-art 写只有 .owner（别的会话的）、没有写者锁的草稿目录，拒」（⑫ 写者锁里没认领与只有 .owner 那两格）
#   SINGLEFS_WORK_ROOT_BREAK=legacy-root-accepted  写范围另认会话临时目录根：⑫ 放行没有项目定义的 agent 写会话临时目录根
#       「Bash写范围:没有项目定义的内置 agent 写会话临时目录根拒（scratchpad 之外）」「Bash写范围:没有项目定义的内置 agent 写会话 scratchpad 拒」（共 2 格）
#   BASH_COMMAND_DETECTOR_BREAK=in-place-script-session-temporary-root-unseen  ⑦ 不认会话临时目录根下的脚本（各会话的 scratchpad 在里面）
#       「就地改脚本:会话临时目录根下的脚本」「就地改脚本:会话 scratchpad 里的脚本」（共 2 格）
# 批量登记的开关（从这一行起到文件头结束，行首 `#   <名字>  说明` 的写法也是一条开关，四处对账认它；这一行之前只认 BASH_COMMAND_DETECTOR_BREAK= 起头的行）：
# 一百三十七节那一批（records/2026-10-29-一百三十七节那一轮调度记录.md 件 1）的开关与各自红的格（格名都以「一百三十七：」起头，另有两格改判的旧格；
# 这一批的格在已有的 disable-check 等开关下也红，写了「共 N 格」的已有开关在一百三十七节件 1b 照实测重数过、点名改成实测红的格）：
#   claude-launch-layers-narrow  「一百三十七：子 agent here-string 喂给 bash 的 claude 照拒（退出码 2）」「…env -S 拆串里的 claude…」「…xargs 起的 claude…」「…ssh 起的 claude…」四格
#   watchdog-tee-null-unseen  「一百三十七：看门狗输出经 tee 丢进 /dev/null 照拒（退出码 2）」
#   watchdog-null-redirect-whole-command  ① 整条命令里哪一段的 /dev/null 都算：
#       「看门狗:run_in_background 起、前一段 grep 2>/dev/null、后一段 watch.sh 放行」「看门狗:run_in_background 起、管道后段只丢 2 号放行」（共 2 格）
#   rsync-inplace-unseen  「就地改脚本:一百三十七：rsync --inplace 盖已有脚本（rsync-inplace-unseen 下红）」
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
#   LIB_SHELL_WORDS_BREAK=part-modules-cached  共用模块载入时沿用 sys.modules 里裸名字缓存的同名部分模块（lib_shell_words_core、lib_shell_words_embedded）、sys.path 留着目录：同一个进程再从别的目录载入一份时，部分模块还是先载入的那个目录里的：
#       「共用切词模块:同一个进程从两个目录各载入一次，后载入的用它自己目录里的部分模块」「共用切词模块:载入之后 sys.path 与 sys.modules 里的部分模块不变」（共 2 格）
#   LIB_SHELL_WORDS_BREAK=part-modules-per-load  共用模块的部分模块不按目录存：同一个目录里每载入一次就各载入一份自己的（类与哨兵对象各不相同）：
#       「共用切词模块:同一个目录载入两次，用的是同一份部分模块」（共 1 格）
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
#       「字面写入:heredoc 追加任务号拒」「字面写入:printf 追加括号里的会话短号拒」「字面写入:echo 经管道交 tee -a 追加别的会话的批号拒」
#       「字面写入:记录里的 A/B 读数一行拒」「字面写入:反引号里的任务号拒」「字面写入:往开着围栏的文件追加以收尾围栏起头的内容拒」
#       「字面写入:同一条命令两次追加让批号出了围栏拒」「字面写入:记录里表格别的列的批号拒」
#       「字面写入:printf 格式串与参数拆开写的任务号拒」「字面写入:前面赋过字面值的变量写进的任务号拒」
#       「字面写入:结束词不带引号的 heredoc 里前面赋过字面值的变量照代入拒」「字面写入:echo -e 转义拼出的批号拒」「字面写入:echo -n 接下一处 echo 拼出的任务号拒」
#       「stdin:Bash heredoc 往仓里追加会话名拒绝（退出码 2，点名会话号、出路指到 insert-row.py 与 replace-once.py）」（共 21 格）
#   BASH_COMMAND_DETECTOR_BREAK=literal-notations-outside-judged  ⑰ 仓外的目标也判（按文件名当相对路径）：实测红的格（照失败行原文）
#       「字面写入:仓外路径不判」「stdin:cd 之后的 $PWD/… 与 mktemp -d 赋的变量放行（退出码 0）、不记检出」（共 2 格）
#       （后一格：写范围那几格的临时仓只拷了钩子与它的五份库，没有 ../gate.d/lib-forbidden-notations.py；这个开关下仓外的目标 $PWD/a.md 也去判，
#        读不到判据库，照判不了的那一条照常放行、记一条检出）
#   BASH_COMMAND_DETECTOR_BREAK=literal-notations-new-text-only  ⑰ 任务号、会话短号与批号只判写进去的内容（写前当空串），看不到写之后整份的样子：
#       「字面写入:往开着围栏的文件追加以收尾围栏起头的内容拒」「字面写入:记录里表格行的批号放行」「字面写入:echo -n 接下一处 echo 拼出的任务号拒」（共 3 格）
#   BASH_COMMAND_DETECTOR_BREAK=literal-notations-before-from-disk  ⑰ 每一处写的写前都读现文件，看不到同一条命令里前几处写进的东西：
#       「字面写入:同一条命令两次追加让批号出了围栏拒」「字面写入:echo -n 接下一处 echo 拼出的任务号拒」（共 2 格）
#   BASH_COMMAND_DETECTOR_BREAK=literal-printf-format-unexpanded  ⑰ printf 不照格式串代入参数，交格式串与每个参数各占一行（号被格式串与参数拆开就判不到）：
#       「字面写入:printf 格式串与参数拆开写的任务号拒」（共 1 格）
#   BASH_COMMAND_DETECTOR_BREAK=literal-echo-escapes-unexpanded  ⑰ echo -e 不换反斜杠转义（转义拼出的号判不到）：
#       「字面写入:echo -e 转义拼出的批号拒」（共 1 格）
#   BASH_COMMAND_DETECTOR_BREAK=literal-variables-unsubstituted  ⑰ echo、printf 的参数与不带引号的 heredoc 正文里前面赋过字面值的变量不代入：
#       「字面写入:前面赋过字面值的变量写进的任务号拒」「字面写入:结束词不带引号的 heredoc 里前面赋过字面值的变量照代入拒」（共 2 格）
#   BASH_COMMAND_DETECTOR_BREAK=allow-record-append  ⑲ 往 records/ 下的 .md 追加一概放行：实测红的格（照失败行原文）
#       「往 records 追加:echo >> 拒」「往 records 追加:cat >> 加 heredoc 拒」「往 records 追加:&>> 拒」「往 records 追加:管道接 tee -a 拒」「往 records 追加:tee --append 拒」
#       「往 records 追加:cd 进 records 之后 >> 拒」「往 records 追加:绝对路径 >> 拒」「往 records 追加:前面赋过值的变量照代入拒」「往 records 追加:bash -c 里的 >> 拒」
#       「往 records 追加:python3 -c 里 open(…, 'a') 拒」「往 records 追加:喂给 python 的 heredoc 里 open(…, mode='a') 拒」「往 records 追加:python 的 Path.open('a') 拒」
#       「往 records 追加:子目录里的 .md 也拒」「往 records 追加:大写后缀 .MD 也拒」「往 records 追加:指进 records 的符号链接照落点拒」
#       「stdin:主 agent 往 records 追加拒、出路给 append-record.py 的整条写法（退出码 2）」「stdin:子 agent 往 records 追加拒、出路给 append-record.py 的整条写法（退出码 2）」
#       「往 records 追加:喂给 python 的 heredoc 里 for 逐个取元组，候选里有一个落在 records 下拒」
#       「出路过别的闸:主 agent 照抄 ⑲ 的出路喂 project-code-guard.sh 放行（退出码 0）」「出路过别的闸:子 agent 照抄 ⑲ 的出路喂 project-code-guard.sh 放行（退出码 0）」（共 20 格）
#   BASH_COMMAND_DETECTOR_BREAK=record-append-python-unseen  ⑲ 不认 python 的 open(…, 'a')：打开后红的格（照失败行原文）
#       「往 records 追加:喂给 python 的 heredoc 里 for 逐个取元组，候选里有一个落在 records 下拒」
#       「往 records 追加:python3 -c 里 open(…, 'a') 拒」「往 records 追加:喂给 python 的 heredoc 里 open(…, mode='a') 拒」「往 records 追加:python 的 Path.open('a') 拒」（共 4 格）
#   BASH_COMMAND_DETECTOR_BREAK=record-append-tee-unseen  ⑲ 不认 tee -a（不看 OverwriteCandidate.appends）：打开后红的格（照失败行原文）
#       「往 records 追加:管道接 tee -a 拒」「往 records 追加:tee --append 拒」（共 2 格）
#   BASH_COMMAND_DETECTOR_BREAK=record-howto-unlisted-tool  ⑲ 拒绝说明里的出路写成不在直接跑名单上的 scripts/kb/history-consolidate.py（照抄会被 project-code-guard.sh 拒）：
#       「stdin:主 agent 往 records 追加拒、出路给 append-record.py 的整条写法（退出码 2）」「stdin:子 agent 往 records 追加拒、出路给 append-record.py 的整条写法（退出码 2）」
#       「出路过别的闸:主 agent 照抄 ⑲ 的出路喂 project-code-guard.sh 放行（退出码 0）」「出路过别的闸:子 agent 照抄 ⑲ 的出路喂 project-code-guard.sh 放行（退出码 0）」（共 4 格）
#   BASH_COMMAND_DETECTOR_BREAK=howto-guard-environment-keeps-breaks  ⑲ 出路喂 project-code-guard.sh 时把自证设着的弄坏开关变量（LIB_SHELL_WORDS_BREAK 这类）原样带过去（它见到不是自己自证设的弄坏开关就拒，出路被误判过不了闸）：
#       「出路过别的闸:喂 project-code-guard.sh 的环境里不带弄坏开关变量，别的变量照带」（共 1 格）
#   BASH_COMMAND_DETECTOR_BREAK=allow-manual-repository-copy  ⑳ 手拷仓副本（rsync、cp -r / -R / -a 拷本仓的 crates/ 或仓根整个）一概放行
#       「手拷仓副本:rsync 拷 crates 进仓外目录」「手拷仓副本:cp -r 拷 crates」「手拷仓副本:rsync 拷仓根整个（./）」「手拷仓副本:rsync 拷仓根（绝对路径带结尾的 /）」
#       「手拷仓副本:cp -a 拷仓根 .」「手拷仓副本:cp -R 拷 crates/（带结尾的 /）」「手拷仓副本:cp --archive 拷 crates」「手拷仓副本:cp -t 目录写在前面、-r 写在后面」
#       「手拷仓副本:rsync 的 --exclude=值 合写」「手拷仓副本:rsync -e 带值的短选项之后的源照认」「手拷仓副本:cd 进仓根之后 rsync crates」
#       「手拷仓副本:前面赋过值的变量代入」「手拷仓副本:bash -c 里」「手拷仓副本:nice 前缀」「手拷仓副本:仓根里的通配 * 盖住整层」「手拷仓副本:crates/* 盖住 crates 整层」
#       「手拷仓副本:crates/.. 规整之后是仓根」「手拷仓副本:喂给 bash 的 heredoc 里」「手拷仓副本:rsync 一次给几个源、其中有 crates」
#       「stdin:主 agent rsync 拷 crates 拒、出路给 minimal-copy.py 的整条写法（退出码 2）」「stdin:子 agent cp -r 拷 crates 拒、出路给 minimal-copy.py 的整条写法（退出码 2）」
#       「出路过别的闸:主 agent 照抄 ⑳ 的出路喂 project-code-guard.sh 放行（退出码 0）」「出路过别的闸:子 agent 照抄 ⑳ 的出路喂 project-code-guard.sh 放行（退出码 0）」（共 23 格）
#   BASH_COMMAND_DETECTOR_BREAK=manual-copy-howto-unlisted-tool  ⑳ 拒绝说明里的出路写成不在直接跑名单上的 scripts/hosts/compile-then-swap.py（照抄会被 project-code-guard.sh 拒）
#       「出路过别的闸:主 agent 照抄 ⑳ 的出路喂 project-code-guard.sh 放行（退出码 0）」「出路过别的闸:子 agent 照抄 ⑳ 的出路喂 project-code-guard.sh 放行（退出码 0）」（共 2 格）
#   BASH_COMMAND_DETECTOR_BREAK=allow-service-source-write  ⑭ 工具员在 Bash 里改两个服务 src/ 下已存在的文件与已存在的 Cargo.toml、Cargo.lock 也放行：实测红的格（照失败行原文）
#       「服务源码:工具员 cat >> 追加已存在的 prune-server/src/protocol.rs」「服务源码:工具员用 replace-once.py 改已存在的 prune-server/src/protocol.rs」「服务源码:工具员 cp 整份盖已存在的 prune-server/src/protocol.rs」
#       「服务源码:工具员 tee 盖 gpu-verification-server/src/ 下已存在的文件」「服务源码:工具员 sed -i 改 src/ 子目录里已存在的 prune-server/src/bin/serve.rs」「服务源码:工具员 cp 进 prune-server/src/ 目录、文件名撞上」
#       「服务源码:工具员 cd 进 src 之后追加」「服务源码:工具员 mv 草稿里的文件盖 prune-server/src/protocol.rs」「服务源码:工具员 bash -c 里 cp」
#       「服务源码:工具员前面赋过值的变量里的目录」「服务源码:工具员 python open(…, 'a')」「服务源码:工具员 python shutil.copy」「服务源码:工具员 python 的 for 逐个取元组，open(循环变量, 'w') 盖 prune-server/src/protocol.rs」
#       「服务源码:工具员经符号链接 cp 写 src/ 下已存在的文件」「服务源码:工具员 cat >> 追加已存在的 prune-server/Cargo.toml」「服务源码:工具员用 replace-once.py 改已存在的 gpu-verification-server/Cargo.toml」
#       「服务源码:工具员 cp 盖已存在的 prune-server/Cargo.lock」「服务源码:工具员 sed -i 改已存在的 gpu-verification-server/Cargo.toml」「服务源码:工具员 cp -b 带备份盖已存在的 prune-server/src/protocol.rs」
#       「服务源码:工具员 install -b 带备份盖已存在的 prune-server/src/protocol.rs」「服务源码:工具员 mv --backup=numbered 盖已存在的 prune-server/src/protocol.rs」「服务源码:工具员 install 不带 -b 盖已存在的 prune-server/src/protocol.rs」
#       「服务源码:工具员 cp -r 草稿/src/. 进 prune-server/src/（逐份盖已存在的源码）」「服务源码:工具员 cp -r 草稿/src 进 prune-server/（落点是 prune-server/src）」「服务源码:工具员 rsync -a 草稿/src/ 进 prune-server/src/（源带结尾的 /）」
#       「服务源码:工具员 rsync -a 草稿/src 进 prune-server/（源不带结尾的 /，落点是 prune-server/src）」「服务源码:工具员 cp -rT 草稿/src 盖 prune-server/src」「服务源码:工具员 cp -r 草稿/src/* 进 prune-server/src/（通配的源）」
#       「服务源码:工具员 ln -sf 把已存在的 prune-server/src/protocol.rs 换成指向草稿的链接」「服务源码:工具员 ln 给 prune-server/src/bin/serve.rs 建硬链接（源是服务源码）」「服务源码:工具员 cp -l 给 prune-server/src/bin/serve.rs 建硬链接」
#       「服务源码:工具员 cp -al 给整个 prune-server/src/ 建硬链接树」「服务源码:工具员往与服务源码是同一个文件的硬链接追加」「服务源码:工具员 tar -xf 解进 prune-server」
#       「服务源码:工具员 tar xzf（不带横线）解进 prune-server/src」「服务源码:工具员 tar --extract --directory= 解进 gpu-verification-server」「服务源码:工具员 cd 进 prune-server/src 之后 tar -xf」
#       「服务源码:工具员 tar -xf 不带 -C、当前目录是仓根」「服务源码:工具员 patch -p1 在仓根打补丁」「服务源码:工具员 patch -d prune-server 打补丁」
#       「服务源码:工具员 git apply 在仓根打补丁」「服务源码:工具员 rm 已存在的 prune-server/src/protocol.rs」「服务源码:工具员 rm -f 已存在的 gpu-verification-server/Cargo.toml」
#       「服务源码:工具员 rm -r 整个 prune-server/src」「服务源码:工具员 rm 通配 prune-server/src/*.rs」「服务源码:工具员 mv 挪走已存在的 prune-server/src/protocol.rs」
#       「服务源码:工具员 mv 挪走整个 prune-server/src」「stdin:工具员 cat >> 追加两个服务 src/ 下已存在的文件拒绝（退出码 2）」「stdin:工具员改两个服务源码的拒绝点名写法、目标与仓副本、补丁的出路」
#       「stdin:工具员 rm 已存在的两个服务 src/ 下的文件拒绝（退出码 2）、拒绝点名 rm 与目标」
#       「服务源码:工具员 unlink 已存在的 prune-server/src/protocol.rs」「服务源码:工具员 find -delete 删 prune-server/src 下已存在的文件」「服务源码:工具员 find -exec rm 删 prune-server/src 下已存在的文件」
#       「服务源码:工具员 rsync -a --delete 空目录 进 prune-server/src/」「服务源码:工具员 rsync --remove-source-files 把 prune-server/src/protocol.rs 拷出去」
#       「服务源码:工具员 gzip 把 prune-server/src/protocol.rs 换成压缩文件」「服务源码:工具员 bzip2 -f 把 prune-server/src/bin/serve.rs 换成压缩文件」「服务源码:工具员 zstd --rm 压完删 prune-server/src/protocol.rs」
#       「服务源码:工具员 shred 改写 prune-server/src/protocol.rs」「服务源码:工具员 sort -o 排完写回 prune-server/src/protocol.rs」「服务源码:工具员 gawk -i inplace 就地改 prune-server/src/protocol.rs」
#       「服务源码:工具员 1<> 在原 inode 上改写 prune-server/src/protocol.rs」「服务源码:工具员 unzip -d 解进 prune-server/src」「服务源码:工具员 unzip 不带 -d、当前目录是仓根」
#       「服务源码:工具员 python os.remove 删 prune-server/src/protocol.rs」「服务源码:工具员 python os.rename 把 prune-server/src/protocol.rs 挪走」「服务源码:工具员 python shutil.rmtree 删整个 prune-server/src」
#       「服务源码:工具员 python Path.unlink 删 prune-server/src/protocol.rs」「服务源码:工具员 python os.open 带 O_WRONLY 改 prune-server/src/protocol.rs」
#       「服务源码:工具员 python shutil.copytree(dirs_exist_ok=True) 逐份盖 prune-server/src」「服务源码:工具员 patch 原文件是已存在的 prune-server/src/protocol.rs」
#       「服务源码:工具员 make-swap-spec.py 的 --out 写已存在的 prune-server/src/protocol.rs」
#       「服务源码:工具员 cp 盖已有门禁的样本」「服务源码:工具员 > 新建已有门禁样本档目录里的文件」「服务源码:工具员 tee 写已有门禁的样本」
#       「服务源码:工具员 mv 草稿里的样本进已有门禁的样本档目录」「服务源码:工具员 cp -r 草稿样本目录进已有门禁的样本档目录」
#       「服务源码:工具员 git apply 在样本档根里打补丁」「服务源码:工具员 rm 已有门禁的样本」「服务源码:工具员 rm -r 整个已有门禁的样本档目录」（共 80 格）
#   BASH_COMMAND_DETECTOR_BREAK=service-source-overwrite-only  ⑭ 只认整份覆盖的写法（>、tee、cp、mv、dd、truncate，python 的 w 与 r+），不认追加、sed -i、定点写脚本、python 的追加与 rsync
#       「服务源码:工具员 cat >> 追加已存在的 prune-server/src/protocol.rs」「服务源码:工具员用 replace-once.py 改已存在的 prune-server/src/protocol.rs」「服务源码:工具员 sed -i 改 src/ 子目录里已存在的 prune-server/src/bin/serve.rs」
#       「服务源码:工具员 cd 进 src 之后追加」「服务源码:工具员 python open(…, 'a')」「服务源码:工具员 cat >> 追加已存在的 prune-server/Cargo.toml」
#       「服务源码:工具员用 replace-once.py 改已存在的 gpu-verification-server/Cargo.toml」「服务源码:工具员 sed -i 改已存在的 gpu-verification-server/Cargo.toml」「服务源码:工具员 rsync -a 草稿/src/ 进 prune-server/src/（源带结尾的 /）」
#       「服务源码:工具员 rsync -a 草稿/src 进 prune-server/（源不带结尾的 /，落点是 prune-server/src）」「服务源码:工具员往与服务源码是同一个文件的硬链接追加」「stdin:工具员 cat >> 追加两个服务 src/ 下已存在的文件拒绝（退出码 2）」
#       「stdin:工具员改两个服务源码的拒绝点名写法、目标与仓副本、补丁的出路」「服务源码:工具员 make-swap-spec.py 的 --out 写已存在的 prune-server/src/protocol.rs」（共 14 格）
#   BASH_COMMAND_DETECTOR_BREAK=service-source-lexical-only  ⑭ 落点不解开符号链接，只按字面规整 `..`（经草稿目录里指到 src/ 下文件的链接写，看不见）
#       「服务源码:工具员经符号链接 cp 写 src/ 下已存在的文件」（共 1 格）
#   BASH_COMMAND_DETECTOR_BREAK=service-source-backup-kept  ⑭ 带 -b、--backup 的 cp、mv、install 当已有的目标不被盖掉（⑤ ⑦ 的认法）
#       「服务源码:工具员 cp -b 带备份盖已存在的 prune-server/src/protocol.rs」「服务源码:工具员 install -b 带备份盖已存在的 prune-server/src/protocol.rs」「服务源码:工具员 mv --backup=numbered 盖已存在的 prune-server/src/protocol.rs」（共 3 格）
#   BASH_COMMAND_DETECTOR_BREAK=service-source-directory-copy-unseen  ⑭ 源是目录的 cp -r、rsync 不逐份对目标目录里已有的服务源码（落点算成目录就放行）
#       「服务源码:工具员 cp -r 草稿/src/. 进 prune-server/src/（逐份盖已存在的源码）」「服务源码:工具员 cp -r 草稿/src 进 prune-server/（落点是 prune-server/src）」「服务源码:工具员 rsync -a 草稿/src/ 进 prune-server/src/（源带结尾的 /）」
#       「服务源码:工具员 rsync -a 草稿/src 进 prune-server/（源不带结尾的 /，落点是 prune-server/src）」「服务源码:工具员 cp -rT 草稿/src 盖 prune-server/src」（共 5 格）
#   BASH_COMMAND_DETECTOR_BREAK=service-source-link-unseen  ⑭ 不认 ln 与 cp -l 的硬链接：已有的服务源码换成链接、给服务源码建硬链接都放行
#       「服务源码:工具员 ln -sf 把已存在的 prune-server/src/protocol.rs 换成指向草稿的链接」「服务源码:工具员 ln 给 prune-server/src/bin/serve.rs 建硬链接（源是服务源码）」「服务源码:工具员 cp -l 给 prune-server/src/bin/serve.rs 建硬链接」
#       「服务源码:工具员 cp -al 给整个 prune-server/src/ 建硬链接树」（共 4 格）
#   BASH_COMMAND_DETECTOR_BREAK=service-source-hardlink-inode-unseen  ⑭ 往与服务源码是同一个文件的硬链接写不按设备号与 inode 号认（落点不在服务源码目录里就放行）
#       「服务源码:工具员往与服务源码是同一个文件的硬链接追加」（共 1 格）
#   BASH_COMMAND_DETECTOR_BREAK=service-source-extract-unseen  ⑭ 不认解包与打补丁（tar -x、patch、git apply 往服务源码目录或包着它的上级写）
#       「服务源码:工具员 tar -xf 解进 prune-server」「服务源码:工具员 tar xzf（不带横线）解进 prune-server/src」「服务源码:工具员 tar --extract --directory= 解进 gpu-verification-server」
#       「服务源码:工具员 cd 进 prune-server/src 之后 tar -xf」「服务源码:工具员 tar -xf 不带 -C、当前目录是仓根」「服务源码:工具员 patch -p1 在仓根打补丁」
#       「服务源码:工具员 patch -d prune-server 打补丁」「服务源码:工具员 git apply 在仓根打补丁」「服务源码:工具员 patch 原文件是已存在的 prune-server/src/protocol.rs」
#       「服务源码:工具员 git apply 在样本档根里打补丁」（共 10 格）
#   BASH_COMMAND_DETECTOR_BREAK=service-source-removal-unseen  ⑭ 不认 rm 与 mv 挪走已有的服务源码（之后同名新建就是整份换上）
#       「服务源码:工具员 rm 已存在的 prune-server/src/protocol.rs」「服务源码:工具员 rm -f 已存在的 gpu-verification-server/Cargo.toml」「服务源码:工具员 rm -r 整个 prune-server/src」
#       「服务源码:工具员 rm 通配 prune-server/src/*.rs」「服务源码:工具员 mv 挪走已存在的 prune-server/src/protocol.rs」「服务源码:工具员 mv 挪走整个 prune-server/src」
#       「stdin:工具员 rm 已存在的两个服务 src/ 下的文件拒绝（退出码 2）、拒绝点名 rm 与目标」「服务源码:工具员 find -exec rm 删 prune-server/src 下已存在的文件」
#       「服务源码:工具员 rm 已有门禁的样本」「服务源码:工具员 rm -r 整个已有门禁的样本档目录」（共 10 格；find -exec 里启动的 rm 同样走 rm 那一支）
#   BASH_COMMAND_DETECTOR_BREAK=service-source-unlink-find-unseen  ⑭ 不认 unlink 与 find 的 -delete、-exec 里对找到的路径写（删、挪走）
#       「服务源码:工具员 unlink 已存在的 prune-server/src/protocol.rs」「服务源码:工具员 find -delete 删 prune-server/src 下已存在的文件」「服务源码:工具员 find -exec rm 删 prune-server/src 下已存在的文件」（共 3 格）
#   BASH_COMMAND_DETECTOR_BREAK=service-source-rsync-removal-unseen  ⑭ 不认 rsync 的 --delete 与 --remove-source-files
#       「服务源码:工具员 rsync -a --delete 空目录 进 prune-server/src/」「服务源码:工具员 rsync --remove-source-files 把 prune-server/src/protocol.rs 拷出去」（共 2 格）
#   BASH_COMMAND_DETECTOR_BREAK=service-source-compressor-unseen  ⑭ 不认 gzip 一类把原文件换成压缩或解压后文件的命令
#       「服务源码:工具员 gzip 把 prune-server/src/protocol.rs 换成压缩文件」「服务源码:工具员 bzip2 -f 把 prune-server/src/bin/serve.rs 换成压缩文件」「服务源码:工具员 zstd --rm 压完删 prune-server/src/protocol.rs」（共 3 格）
#   BASH_COMMAND_DETECTOR_BREAK=service-source-in-place-tools-unseen  ⑭ 不认 shred、sort -o、gawk -i inplace 在原 inode 上整份改写
#       「服务源码:工具员 shred 改写 prune-server/src/protocol.rs」「服务源码:工具员 sort -o 排完写回 prune-server/src/protocol.rs」「服务源码:工具员 gawk -i inplace 就地改 prune-server/src/protocol.rs」（共 3 格）
#   BASH_COMMAND_DETECTOR_BREAK=service-source-read-write-redirect-unseen  ⑭ 不认 `<>` 读写重定向
#       「服务源码:工具员 1<> 在原 inode 上改写 prune-server/src/protocol.rs」（共 1 格）
#   BASH_COMMAND_DETECTOR_BREAK=service-source-unzip-unseen  ⑭ 不认 unzip 解进服务源码目录或包着它的上级
#       「服务源码:工具员 unzip -d 解进 prune-server/src」「服务源码:工具员 unzip 不带 -d、当前目录是仓根」（共 2 格）
#   BASH_COMMAND_DETECTOR_BREAK=service-source-python-delete-unseen  ⑭ 不认 python 的删除与挪走（os.remove、os.unlink、Path.unlink、shutil.rmtree，os.rename、os.replace、shutil.move 的源）
#       「服务源码:工具员 python os.remove 删 prune-server/src/protocol.rs」「服务源码:工具员 python os.rename 把 prune-server/src/protocol.rs 挪走」「服务源码:工具员 python shutil.rmtree 删整个 prune-server/src」
#       「服务源码:工具员 python Path.unlink 删 prune-server/src/protocol.rs」（共 4 格）
#   BASH_COMMAND_DETECTOR_BREAK=service-source-python-in-place-unseen  ⑭ 不认 python 带写标志的 os.open 与 shutil.copytree(…, dirs_exist_ok=True) 往已有目录逐份盖
#       「服务源码:工具员 python os.open 带 O_WRONLY 改 prune-server/src/protocol.rs」「服务源码:工具员 python shutil.copytree(dirs_exist_ok=True) 逐份盖 prune-server/src」（共 2 格）
#   BASH_COMMAND_DETECTOR_BREAK=service-source-stdout-extract-judged  ⑭ 解到标准输出的 tar（-O、--to-stdout、--to-command）也照解进目录判（不写盘的命令被拒）：打开后红的格（照失败行原文）
#       「服务源码:工具员 tar -xOf 解到标准输出放行」「服务源码:工具员 tar xOf（不带横线）解到标准输出放行」「服务源码:工具员 tar --to-stdout 解到标准输出放行」（共 3 格）
#   BASH_COMMAND_DETECTOR_BREAK=service-source-patch-operand-ignored  ⑭ patch 不看位置参数的原文件，照目录判（只改草稿文件的 patch 被拒）：打开后红的格（照失败行原文）
#       「服务源码:工具员 patch 原文件 补丁、原文件在草稿目录放行」「服务源码:工具员 patch -i 补丁 原文件、原文件在草稿目录放行」（共 2 格）
#   BASH_COMMAND_DETECTOR_BREAK=gate-fixtures-unseen  ⑭ 不判往已有门禁的样本档目录写（ServiceSourceView.gate_fixture_note 一律交 None）
#       「服务源码:工具员 cp 盖已有门禁的样本」「服务源码:工具员 > 新建已有门禁样本档目录里的文件」「服务源码:工具员 tee 写已有门禁的样本」
#       「服务源码:工具员 mv 草稿里的样本进已有门禁的样本档目录」「服务源码:工具员 cp -r 草稿样本目录进已有门禁的样本档目录」
#       「服务源码:工具员 git apply 在样本档根里打补丁」「服务源码:工具员 rm 已有门禁的样本」「服务源码:工具员 rm -r 整个已有门禁的样本档目录」（共 8 格）
#   BASH_COMMAND_DETECTOR_BREAK=scope-swap-spec-out-unseen  ⑫ ⑭ 不认 make-swap-spec.py 的 --out 那一份规格
#       「Bash写范围:clerk 用 make-swap-spec.py 的 --out 写知识库」「服务源码:工具员 make-swap-spec.py 的 --out 写已存在的 prune-server/src/protocol.rs」（共 2 格）
#   BASH_COMMAND_DETECTOR_BREAK=part-unreadable-passes  引导（下面读部分文件的那一段）载不全部分文件（读不到、编不过、编得过而 import 抛错或入口要的名字缺）时不按「判不了的」处置：照原样抛错、Traceback 退 1，bash 那一层兜成放行与泛泛的检出，stderr 与检出都不点名是哪一份部分文件（引导读它，只在带自证标记的子进程里认）：打开后红的格（照失败行原文）
#       「引导:读不到部分文件 bash_command_detector_scope_and_index.py 照常放行（退出码 0）、记一条点名部分文件的检出」「引导:编不过部分文件 bash_command_detector_scope_and_index.py 照常放行（退出码 0）、记一条点名部分文件的检出」「引导:截空部分文件 bash_command_detector_scope_and_index.py 照常放行（退出码 0）、记一条点名部分文件的检出」
#       「引导:读不到部分文件 bash_command_detector_hooks_and_overwrites.py 照常放行（退出码 0）、记一条点名部分文件的检出」「引导:编不过部分文件 bash_command_detector_hooks_and_overwrites.py 照常放行（退出码 0）、记一条点名部分文件的检出」「引导:截空部分文件 bash_command_detector_hooks_and_overwrites.py 照常放行（退出码 0）、记一条点名部分文件的检出」
#       「引导:读不到部分文件 bash_command_detector_literals_and_signals.py 照常放行（退出码 0）、记一条点名部分文件的检出」「引导:编不过部分文件 bash_command_detector_literals_and_signals.py 照常放行（退出码 0）、记一条点名部分文件的检出」「引导:截空部分文件 bash_command_detector_literals_and_signals.py 照常放行（退出码 0）、记一条点名部分文件的检出」
#       「引导:读不到部分文件 bash_command_detector_selftest_waits_and_scope.py 照常放行（退出码 0）、记一条点名部分文件的检出」「引导:编不过部分文件 bash_command_detector_selftest_waits_and_scope.py 照常放行（退出码 0）、记一条点名部分文件的检出」「引导:截空部分文件 bash_command_detector_selftest_waits_and_scope.py 照常放行（退出码 0）、记一条点名部分文件的检出」
#       「引导:读不到部分文件 bash_command_detector_selftest_entries_and_main.py 照常放行（退出码 0）、记一条点名部分文件的检出」「引导:编不过部分文件 bash_command_detector_selftest_entries_and_main.py 照常放行（退出码 0）、记一条点名部分文件的检出」「引导:截空部分文件 bash_command_detector_selftest_entries_and_main.py 照常放行（退出码 0）、记一条点名部分文件的检出」（共 15 格）
#   LIB_FORBIDDEN_NOTATIONS_BREAK=batch-codes  共用判据库不判批次代号这一形态（写法在 ../gate.d/lib-forbidden-notations.py 文件头）：
#       「字面写入:heredoc 追加批次代号拒」「字面写入:echo 经管道交 tee -a 追加批次代号拒」「字面写入:cd 进仓里之后按相对路径写照判」
#       「字面写入:here-string 喂给 cat 写批次代号照判」（共 4 格）
#   LIB_FORBIDDEN_NOTATIONS_BREAK=task-and-session-numbers  共用判据库不判任务号、会话短号与批号这一形态（写法在 ../gate.d/lib-forbidden-notations.py 文件头）：
#       「字面写入:heredoc 追加任务号拒」「字面写入:printf 追加括号里的会话短号拒」「字面写入:echo 经管道交 tee -a 追加别的会话的批号拒」
#       「字面写入:记录里的 A/B 读数一行拒」「字面写入:反引号里的任务号拒」「字面写入:往开着围栏的文件追加以收尾围栏起头的内容拒」
#       「字面写入:同一条命令两次追加让批号出了围栏拒」「字面写入:记录里表格别的列的批号拒」
#       「字面写入:printf 格式串与参数拆开写的任务号拒」「字面写入:前面赋过字面值的变量写进的任务号拒」
#       「字面写入:结束词不带引号的 heredoc 里前面赋过字面值的变量照代入拒」「字面写入:echo -e 转义拼出的批号拒」「字面写入:echo -n 接下一处 echo 拼出的任务号拒」（共 13 格）
#   LIB_FORBIDDEN_NOTATIONS_BREAK=after-lines-judged-alone  共用判据库判写前、写后两份整份目标时逐行单判、不认围栏与表头：
#       「字面写入:往围栏已收的文件追加以围栏起头的内容放行」「字面写入:整份写不看目标里原有的围栏，以围栏起头的内容放行」「字面写入:记录里表格行的批号放行」（共 3 格）
#   LIB_FORBIDDEN_NOTATIONS_BREAK=violations-not-netted  共用判据库不拿写前目标里已有的违例行相抵：
#       「字面写入:往有旧号的文件追加干净内容放行」（共 1 格）
#   LIB_FORBIDDEN_NOTATIONS_BREAK=table-batch-any-column  共用判据库在 records/ 下的表格行里任何一列的批号都放过：
#       「字面写入:记录里表格别的列的批号拒」（共 1 格）
#   LIB_FORBIDDEN_NOTATIONS_BREAK=fenced-blocks-judged  共用判据库在围栏代码块里也判会话短号与批号：
#       「字面写入:往围栏已收的文件追加以围栏起头的内容放行」「字面写入:整份写不看目标里原有的围栏，以围栏起头的内容放行」（共 2 格）
#   LIB_FORBIDDEN_NOTATIONS_BREAK=task-numbers-in-code-span-kept  共用判据库放过反引号里的任务号：
#       「字面写入:反引号里的任务号拒」（共 1 格）
#   LIB_FORBIDDEN_NOTATIONS_BREAK=zero-round-reading-anywhere  共用判据库的「A/B 读数：」放行不锚行首、不看文件：
#       「字面写入:记录里的 A/B 读数一行拒」（共 1 格）
# ② 末那一句之外：前台等待循环的拒绝说明另写「带计数上限、break、while read、kill -0 探活这类真会结束的循环同样照拒，套 timeout 就放行」；
# ⑦ 另认 `rsync --inplace 源 已有文件`（就地写、不换 inode）；① 另认 `| tee /dev/null`（看门狗的输出照样丢了）；
# ⑫ 重定向目标上的花括号（`> o.{a,b}`、`> o{1..2}.txt`，引号里的同样）照落点算不出拒，拒绝说明点名 bash 报 ambiguous redirect、什么都不写，改法是写成一个字面文件名；
# 钩子 bash 这一层起不来（python3 起不来、/dev/fd/3 读不到、python 在 main() 之前崩了）时照「放行但必记检出」追加一行、退 0（一百三十七节 ⑤；带参数的起法不读标准输入）。
#   PATH 上没有 python3 的，preflight 判不了运行条件、会退 78 什么都不记：钩子那种起法在 preflight 之前就照「放行但必记检出」记一条、退 0
#   （弄坏开关 BASH_COMMAND_DETECTOR_BREAK=python-missing-unrecorded 下照旧交给 preflight 退 78，只在带自证标记的子进程里认）。
# python 代码分在同目录的这几份（本文件是入口：bash 这一层与下面的引导；引导先把每一份读进来、核顶层名字、语法解析过（只读语法树、不执行），再按次序 import 它们（每一份是普通 python 模块，用到别份的名字写显式 import、不留环），不单独执行）：
# bash_command_detector_scope_and_index.py：最底层、不 import 别份：python 这一层的读法、等待循环与放出追踪与看门狗、弄坏开关名单与对账、就地改仓内文件（⑧）、先编后换（⑨）、写范围（⑫）、服务源码（⑭）、起 claude（⑬）、前台 timeout（㉑）、cargo build 点全部目标（㉓）的认法与清整个索引（⑮）、撤工作区改动（⑱）的底层函数
# bash_command_detector_hooks_and_overwrites.py：import scope_and_index：绕开 git 钩子（㉒）、后台任务输出与单独的 &（④）、覆盖产物（⑤）的写法、解包与打补丁与压缩与就地写的工具（⑭）、mktemp 与写者锁与三方腿（⑫）、清整个索引（⑮）与撤工作区改动（⑱）的拒绝、已知写脚本与服务源码目录的拷法、record 与 findings_for
# bash_command_detector_literals_and_signals.py：import 前两份：账本追加（⑯）、records 追加（⑲）、字面内容里的会话号与批号（⑰）、就地改脚本（⑦）、终止进程（⑥）、通删工作根（⑪）、手拷仓副本（⑳），与写范围（⑫）、先编后换（⑨）、就地改仓内文件（⑧）、服务源码（⑭）、cargo build（㉓）、覆盖产物（⑤）各道的 verdict 与 refusal、follow_simple_command、overwrite_steps
# bash_command_detector_selftest_waits_and_scope.py：自证函数 selftest_in_waits_and_scope：等待循环、放出追踪、看门狗、写范围、终止进程、扫脚本、就地改脚本的样本（selftest_in 把现场拆成两个函数，局部变量经 shared 交接）
# bash_command_detector_selftest_entries_and_main.py：自证函数 selftest_in_entries_and_summary：通删工作根、真实入口的样本、弄坏开关对账、汇总，selftest 与 selftest_in，账本追加、records 追加、字面内容与扫脚本的几道判定（ledger_append_refusal、record_append_refusal、literal_writes、literal_notation_hits、read_pending_list、scan_scripts），与入口 main
# 弄坏开关的名单、读开关的代码在这几份 python 里，开关表与红的格写在这个文件头；入口 bash 这一层读的那两条开关的代码在这一份里，引导读的那一条（part-unreadable-passes）在引导里。
set -uo pipefail
HOOK_PART_UNREADABLE_STATUS=70   # 引导读不到或编不过部分文件时 python 退的码：下面记检出时点名是部分文件的问题（与引导里的 PART_UNREADABLE_STATUS 同值）
# 钩子 bash 这一层起不来时也记一条检出（一百三十七节 ⑤）：python3 起不来、/dev/fd/3 读不到、python 在 main() 之前崩了，这条命令照常执行（钩子退出码不是 0、2 时 Claude Code 照常放行），
# 标准输入先在这里读进来、经 here-string 交给 python（python 起不来时 bash 还拿得到会话号）；退出码不是 0、2、又不是 --selftest 的，照「放行但必记检出」
# 往检出记录追加一行（会话号、agent 从输入里按字面取，取不到写「读不到」）、退 0；带参数的起法（--selftest、--scan-scripts）不读标准输入、照交退出码。弄坏开关 BASH_COMMAND_DETECTOR_BREAK=bash-layer-unrecorded 下不记（照旧把 python 的退出码交出去）。
hook_record_bash_layer_failure() {   # <钩子名> <python 的退出码> <hook 输入原文>
  local name="$1" status="$2" input="$3" path="${AGENT_HOOK_DETECTIONS:-}" work_root="${SINGLEFS_WORK_ROOT:-}" session="读不到" agent="读不到"
  local failure="钩子 bash 那一层起不来（python3 退出码 $status）"
  if [[ "$status" == "$HOOK_PART_UNREADABLE_STATUS" ]]; then failure="钩子的 python 部分文件读不到或编不过（python3 退出码 $status，stderr 点名是哪一份）"; fi
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
  printf '{"time": "%s", "session_id": "%s", "agent_type": "%s", "command": "", "findings": ["%s 没判成：%s，这条命令没判、照常执行"], "outcome": "只记"}\n' \
    "$(date -u +%Y-%m-%dT%H:%M:%S+00:00)" "$session" "$agent" "$name" "$failure" >> "$path" 2>/dev/null
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
import ast, os, sys
DETECTOR_PART_NAMES = ("bash_command_detector_scope_and_index.py", "bash_command_detector_hooks_and_overwrites.py", "bash_command_detector_literals_and_signals.py", "bash_command_detector_selftest_waits_and_scope.py", "bash_command_detector_selftest_entries_and_main.py",)   # 按次序 import（后一份 import 前一份、没有环）；路径按钩子自己所在的目录（argv[1]）取
DETECTOR_PART_REQUIRED_NAMES = (   # 与上一行同次序、一份部分文件一组：入口要的名字（这一份第一个、最后一个绑定的名字与别份、入口引到的名字）；import 之前用 ast 核这一份顶层的 def、class 与赋值绑定了这些名字，缺一个就是不完整（截空、截短）
    ("shell_words_library_path", "load_shell_words", "clippy_lint_tail"),
    ("GIT_COMMIT_SHORT_OPTIONS_WITH_VALUE", "untracked_among", "worktree_discard_refusal"),
    ("LEDGER_REGISTRY_PATH", "copied_source_hits", "manual_repository_copy_refusal", "whole_target_build_messages", "results_overwrite_verdict", "results_overwrite_refusal"),
    ("selftest_in_waits_and_scope",),
    ("selftest_in_entries_and_summary", "main", "selftest", "selftest_in"),
)
# 弄坏开关的变量名与读法、自证起的子进程的标记：引导在部分文件载入之前就要读它们，所以定义放在这里（用法与自证在 python 部分文件里）
DETECTOR_BREAK_VARIABLE = "BASH_COMMAND_DETECTOR_BREAK"
SELFTEST_MARK_VARIABLE = "BASH_COMMAND_DETECTOR_SELFTEST_MARK"
SELFTEST_MARK_CONTENT = "bash-command-detector-selftest"
PART_UNREADABLE_STATUS = 70   # 读不到或编不过部分文件时 python 退的码，与 bash 那一层的 HOOK_PART_UNREADABLE_STATUS 同值

def detector_break_is_set(item):
    return item in os.environ.get(DETECTOR_BREAK_VARIABLE, "").split(",")

def selftest_marked():
    mark = os.environ.get(SELFTEST_MARK_VARIABLE, "")
    try:
        return bool(mark) and open(mark, encoding="utf-8").read().strip() == SELFTEST_MARK_CONTENT
    except OSError:
        return False


def hook_part_unusable(hook_part_path, hook_part_error):
    """一份部分文件载不全（读不到、编不过、编得过而不完整）：照「判不了的（放行但必记检出）」处置（stderr 点名是哪一份、退 PART_UNREADABLE_STATUS，bash 那一层放行并记检出）。"""
    if selftest_marked() and detector_break_is_set("part-unreadable-passes"):
        raise hook_part_error   # 弄坏开关 part-unreadable-passes：照原样抛错、Traceback 退 1（只在带自证标记的子进程里认）
    print(f"  ! bash-command-detector.sh：读不到或编不过部分文件 {hook_part_path}（{type(hook_part_error).__name__}：{hook_part_error}），这条命令没判、照常执行；记一条检出", file=sys.stderr)
    sys.exit(PART_UNREADABLE_STATUS)

def hook_part_bound_names(hook_part_tree):
    """部分文件顶层（不进 if、try、函数体）绑定的名字：def、class、赋值的目标。只读语法树，不执行这一份。"""
    bound = set()
    for statement in hook_part_tree.body:
        if isinstance(statement, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)):
            bound.add(statement.name)
        elif isinstance(statement, ast.Assign):
            for target in statement.targets:
                bound.update(node.id for node in ast.walk(target) if isinstance(node, ast.Name) and isinstance(node.ctx, ast.Store))
        elif isinstance(statement, ast.AnnAssign) and isinstance(statement.target, ast.Name) and statement.value is not None:
            bound.add(statement.target.id)
    return bound

sys.dont_write_bytecode = True   # 部分文件按同目录的模块 import：不往 __pycache__ 里写（自证拷一份钩子目录再写坏、截空其中一份，缓存的字节码不许盖住）
sys.path.insert(0, sys.argv[1])   # 部分文件之间的 import 与下面的 import 都从钩子自己所在的目录找
for hook_part_name, hook_part_required in zip(DETECTOR_PART_NAMES, DETECTOR_PART_REQUIRED_NAMES, strict=True):   # 每一份先读进来、核顶层绑定了入口要的名字、语法解析过（只读语法树、不执行），全核完全过了才 import：读不到、语法错或缺名字的，按缺依赖的处置（点名是哪一份、缺哪个名字），不让 python 在 main 之前抛错退 1 而被照常放行
    hook_part_path = os.path.join(sys.argv[1], hook_part_name)
    try:
        with open(hook_part_path, encoding="utf-8") as hook_part_handle:
            hook_part_tree = ast.parse(hook_part_handle.read(), hook_part_path)
        hook_part_bound = hook_part_bound_names(hook_part_tree)
        hook_part_missing = [name for name in hook_part_required if name not in hook_part_bound]
        if hook_part_missing:
            raise LookupError(f"顶层没有绑定 {'、'.join(hook_part_missing)}，这一份不完整")
    except (OSError, SyntaxError, ValueError, LookupError) as hook_part_error:   # 文件不在、读不了、不是 UTF-8、语法错、含空字节、缺名字
        hook_part_unusable(hook_part_path, hook_part_error)
try:   # 每一份单独 import（后一份 import 前一份、没有环）：名字齐了、运行到一半抛错（引到被截掉的名字、模块层语句出错）点名出错的那一份
    import bash_command_detector_scope_and_index
except Exception as hook_part_error:
    hook_part_unusable(os.path.join(sys.argv[1], "bash_command_detector_scope_and_index.py"), hook_part_error)
try:
    import bash_command_detector_hooks_and_overwrites
except Exception as hook_part_error:
    hook_part_unusable(os.path.join(sys.argv[1], "bash_command_detector_hooks_and_overwrites.py"), hook_part_error)
try:
    import bash_command_detector_literals_and_signals
except Exception as hook_part_error:
    hook_part_unusable(os.path.join(sys.argv[1], "bash_command_detector_literals_and_signals.py"), hook_part_error)
try:
    import bash_command_detector_selftest_waits_and_scope
except Exception as hook_part_error:
    hook_part_unusable(os.path.join(sys.argv[1], "bash_command_detector_selftest_waits_and_scope.py"), hook_part_error)
try:
    from bash_command_detector_selftest_entries_and_main import main
except Exception as hook_part_error:
    hook_part_unusable(os.path.join(sys.argv[1], "bash_command_detector_selftest_entries_and_main.py"), hook_part_error)
sys.exit(main())   # 主函数由引导在核完之后调，不靠部分文件末行自己退出
PY
hook_status=$?
if [[ $# -eq 0 && "$hook_status" != 0 && "$hook_status" != 2 && ( "$(cat "${BASH_COMMAND_DETECTOR_SELFTEST_MARK:-/nonexistent}" 2>/dev/null)" != bash-command-detector-selftest || ",${BASH_COMMAND_DETECTOR_BREAK:-}," != *,bash-layer-unrecorded,* ) ]]; then
  hook_record_bash_layer_failure "bash-command-detector.sh" "$hook_status" "$hook_input"
  exit 0
fi
exit "$hook_status"
