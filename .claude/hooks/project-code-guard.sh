#!/usr/bin/env bash
# admission: always Claude Code 每一次触发都要现判这一次调用，上一次的结论不替这一次作保
# run-condition: command python3
# PreToolUse hook（Bash）：一切跑项目代码的命令只认经 task-server/task 起、绕开服务的写法照拒（下面「经 task-server 起」）；子 agent 起 task-server/task 只许只读的 status、log，submit、wait、cancel、clean 一律拒（服务认不出提交的是不是子 agent）；不合的当场拒绝（退出 2，stderr 写原因与出路），同时记进检出记录交主 agent 看。
# hook-events: PreToolUse
# gate-similar: bash-command-detector.sh 同挂 PreToolUse[Bash]、判同一条命令，切词已抽成 lib_shell_words.py 两边共用；它判命令的形状（没超时的等待循环、后台里又放后台、起看门狗的写法），执行前拒绝几种写法（列在 bash-command-detector.sh 文件头）、其余只记不拦；这里判命令是不是不经 task-server 跑项目代码，按 agent_type 判、不合就拒，还把执行的脚本读进去逐层判，判定与看门狗共用 lib_project_code_runs.py
# gate-similar: pattern-process-guard.sh 同挂 PreToolUse[Bash]、也在执行前拒绝；它是上游 SOP 的钩子，判按模式找进程（pgrep / pkill 带 -f、killall），对谁都拒、没有放行条件，判据与 shell-lint 的 S2、S3 同一份；本仓的项目代码（崩溃点重放、QEMU、herd7 一类装置）是本工程自己的，不归上游管，也不许在 .claude/singlefs-ai-sop/ 里就地改
# gate-similar: runner-dispatch-guard.sh 管同一条规矩（子 agent 不跑项目代码），但挂 PreToolUse[Agent|Task]、在派发那一刻判派发提示里的中文句子（只在「跑 / 复跑 / 执行 / 运行 / bash / 起」的宾语是本仓的代码时才拒，否定、转述、引号里的都不判）；这里挂 PreToolUse[Bash]、在执行那一刻判真要跑的命令与它执行的脚本，一边判自然语言、一边判 shell 命令，判法没有能共用的
# gate-similar: write-guard.sh 同样按 agent_type 分谁能做什么，但挂 PreToolUse[Write|Edit]，判的是写哪个文件、写进什么字（整份覆盖未跟踪文件、写范围表、撇号类字符）；这里判 Bash 命令，放行表是各 agent 能直接跑哪几类命令，与写范围表没有共用的行
# gate-similar: continuation-guard.sh 挂 PreToolUse[SendMessage]，判的是收件的子 agent 在会话记录里交回过没有、被中断过没有；这里挂 PreToolUse[Bash]，判命令本身，两边的对象与输入没有交集
# gate-overlap:copy-kept bash-command-detector.sh 两边 main() 开头那十行（认 --selftest、从 stdin 读 hook 的 JSON、不是 JSON 对象就打一行诊断、记一条检出、放行）是入口，读不到共用模块时照样要跑：那时还要拿这份 JSON 带着 session_id 记一条检出，看门狗按 session_id 认本会话；抽进 lib_shell_words.py 或另一个同目录模块，模块读不到的那一刻入口跟着一起没了（两个 hook 自检里「hook 旁边没有共用模块」那一例走的就是这条路）
#
# 为什么：一切跑项目代码的命令只认经 task-server/task 起，
# 子 agent 不交任务。定义里写一句「不跑」守不住：
# 同一天几个实现员在各自的副本里直接跑全量 cargo test，满载下每遍四五个小时（records/2026-09-16-subagent拆分提案.md 第四十节那张表第 4、10、12、14 行）。
# 同日一个实现员把验证链写成 /tmp 下的脚本再用后台 Bash 起它，链里有一步是直接跑项目代码，这道闸只看到「执行一个脚本」、放行了
# （同一张表第 15 行）：所以命令位置上执行的脚本文件要读进去判。
# 子 agent 自测时常在同一条命令里先用 heredoc 写出脚本、紧接着执行它，判的那一刻文件还不在，按「不存在」记检出，一小时里叫醒主 agent 六七次
# （同一张表第 18 行）：所以同一条命令里前面写出的脚本拿写出的内容判，不记「不存在」。
# 双引号里没转义的反引号把 `crates/…/walk.rs` 当命令替换执行，这个 `.rs` 被当 shell 脚本读进去、里面像路径的词再当脚本读，一路递归到第 6 层，
# 一条命令 100 秒没判完、拖慢每个 agent 的每条命令（同一张表第 18 行）：所以直接执行的只读 shell 脚本，同一份脚本一次判定里只读一遍、只判一遍。
#
# scripts/crash/gpu-quiesce-run.sh -- <命令> 与 /usr/bin/time 那一类一样剥掉（lib_project_code_runs.py 的 command_under_gpu_quiesce），里面那条照这张表判；清场归 task-server，命令里不套它。
# lib_shell_words 前缀表之外、包在命令外面照样起那条命令的 `/usr/bin/time`、`flock`、`rustup run`、`chrt`、`prlimit`、`systemd-run`、`strace`、
#   `perf stat|record|trace`（lib_project_code_runs.py 的 LAUNCHER_OPTIONS_WITH_VALUE）与名单外启动器（下面「认不出的写法」那一段）剥掉之后，里面那条当一段命令文本接着判：环境变量、内存包装、脚本照认。
# 写进脚本文件再执行的，读脚本正文、逐条照同一张表判：命令位置上是 `bash|sh 文件`（不带 -c）、`./x.sh` 或 `路径/x.sh`、`source 文件` / `. 文件`，
#   经 capped.sh N、nice、timeout、env、命令前的变量赋值包一层的同样认；脚本里再起脚本的递归读，读到第 MAXIMUM_SCRIPT_DEPTH（6）层为止，再往里判「看不全」。
#   外层命令带的变量赋值与 export 往脚本里传（BASH_ENV、PATH 这类照认）；谁能跑什么照上面那张表，不因为写在脚本里而变。
#   拒绝说明写出脚本路径与行号（`外层脚本:行 → 里一层的脚本:行`）。
#   按名字认的仓内入口脚本（.claude/gate.d/ 下的阶段，gate.sh、gate-staged.sh、mutate.sh、capped.sh、run-with-memory-cap.sh、崩溃放量与崩溃点重放的入口、build-patched-qemu.sh、
#   上游的 stage-selftest.sh 这类；KNOWN_SCRIPT_LOCATIONS）：起它已经记成 ① 的直接起，里面起什么由它们自己的自证与门禁管，正文不读（lib_project_code_runs.py 的 judged_by_name）。
#   盘上有这份文件、又不是本仓那一份（路径里有符号链接、落在别的树里）而内容（连同它按自己所在目录 source / import 的库）与本仓那一份不逐字节相同的照读正文；
#   盘上没有的照旧按名字认（existing_copy_elsewhere、same_content_as_own；弄坏开关 LIB_PROJECT_CODE_RUNS_BREAK=same-family-by-path-tail、family-copy-content-unchecked）。
#   剥掉的 capped.sh / run-with-memory-cap.sh 同一口径：别处那份内容不同的照读正文、这一条不算经内存包装（PROJECT_CODE_GUARD_BREAK=wrapper-copies-trusted 下照旧只按名字剥）。
#   读脚本正文时，入口按自己所在目录 source 的库（`source "$(dirname "${BASH_SOURCE[0]}")/…"`、`"${BASH_SOURCE[0]%/*}/…"`、`$(cd "$(dirname "$0")" && pwd)`）换成字面路径照读
#   （经链接起的 crash-point-replay.sh --list-items、crash-amplification.sh --list-items 读正文时库也读；lib_project_code_runs.py 的 script_directory_expanded，
#   LIB_PROJECT_CODE_RUNS_BREAK=sourced-libraries-unread）；工作根底下仓副本里的自证另比被起脚本 source / import 的库（selftest_work_copy_problem，
#   LIB_PROJECT_CODE_RUNS_BREAK=selftest-work-copy-libraries-unchecked）。
#   直接执行的（`./x`、`路径/x`，经包装的同样）只当两种文件是脚本：`#!` 指到 shell（SHELL_NAMES：bash / sh / dash / zsh / ksh，`#!/usr/bin/env` 转一层也算），
#   或没有 `#!`、扩展名是 `.sh`（执行的名字或它指到的文件）；开头是编译产物（ELF 魔数、NUL 字节）的也不算。别的一律不读、不记：
#   `.rs`、`.py`、`.md`、`.tsv` 这类、没扩展名又没 `#!` 的文本、`#!` 指到 python 这类别的解释器的。判它只看开头 SCRIPT_HEAD_BYTES（4 KiB），判在大小上限之前；
#   还不在的，只有扩展名是 `.sh` 的记「不存在」。`bash x`、`sh x`、`source x`、`. x` 显式交给 shell 的不看扩展名与 `#!`，照读。
#   读不到的——文件不存在、不是文本、超过 MAXIMUM_SCRIPT_BYTES（1 MiB）、嵌套超过上限看不全——这条命令照常执行，stderr 报一句并记一条检出；
#   不是普通文件的（目录、设备）不读、不记。脚本调回它自己（或绕一圈回来）不再读第二遍。
#   一次判定里（JudgingState）同一份文件按实际路径只从盘上读一遍；同一份脚本判过一遍，再遇到时直接用那一遍的结果，拒绝说明与检出照这一处的路径与行号写。
#   复用要这几样都相同：实际路径、执行时的当前目录、同一条命令里写出的文件自那一遍起没变过（written_version）、起它的那一条经没经内存包装；
#   那一遍撞上了嵌套上限、绕回了它外层的脚本、或自己写出了文件的，结果跟着从哪里起、什么时候起走，不复用，再遇到时重判（正文仍不重读）。
# 同一条命令里前面写出、后面执行的脚本（WrittenScript），执行时拿写出的内容判，先于盘上那一份（执行时盘上已经是写出的内容），不记「不存在」：
#   heredoc 喂给 cat 再输出重定向到那个文件（`cat > x.sh <<'EOF'`、`cat <<'EOF' > x.sh`；cat 带了文件参数或别的选项的不算），
#   heredoc 喂给 tee（参数里的文件，连同它的输出重定向），拿 heredoc 正文当脚本内容；输出重定向只认标准输出最后落到的那个文件
#   （照 fd 号与 `n>&m` 跟：`2> 文件`、`3> 文件` 拿不到正文，`3>x.sh >&3` 拿得到；lib_shell_words.py 的 standard_output_destination；
#   弄坏开关 PROJECT_CODE_GUARD_BREAK=heredoc-write-every-redirection 退回不看 fd 号）；
#   `cp 源 目的`（也认 `cp 源… 目录`、`-t 目录`；ln（-s 的符号链接与硬链接）、install（-d 建目录的不算）、mv 同样认，不带选项的 `cat 源 > 目的` 也认，
#   目的同样照标准输出最后落到的、由 `>` 或 `>|` 打开的那个文件认；几个源的 `cat 源 源 > 目的`（`cat s.sh "2"> d.sh` 引号里的 2 也是源）按各个源依次接起来的内容认，
#   不在的源照空的算（cat_concatenation；弄坏开关 PROJECT_CODE_GUARD_BREAK=cat-concatenation-unseen 退回几个源的不认）），
#   源在盘上就读源文件，源是同一条命令里更前面写出的就接着用那份内容；
#   追加（`>>`、`tee -a`）接在这条命令里前面写出的内容、或盘上那份（读得了的文本）后面；盘上那份读不了的（目录、超过读的上限、不是 UTF-8）不认那个目的，照旧读盘上，
#   这一条命令里后面的词照常判（PROJECT_CODE_GUARD_BREAK=cat-append-unreadable-destination-raises 下 cat 追加到这样的文件上，判的时候抛 AttributeError、整条命令照常执行）。
#   脚本里再写出、再执行的同样认，写出的文件对外层后面的命令也算数。读法与判定同盘上的脚本（开头的 `#!`、大小上限、递归读）。
#   `cat 源 >> 目的`、`&>`、`>& 目的`、`&>>` 写出的 cat 拷贝与 tee 拷出（`cat 源 | tee [-a] 目的`、`tee 目的 < 源`、管道前一段是 printf / echo / 读 heredoc 的 cat 的）
#   同样按拷来的内容认（弄坏开关 PROJECT_CODE_GUARD_BREAK=copy-redirections-unseen、tee-copy-unseen）。
#   认不出的写法（printf、echo、dd、sed、head、install /dev/stdin、python 写文件、带选项的 cat、变量拼出的目标）写出再执行的，在命令文本这一层照「认不出起什么的写法」拒
#   （下面「认不出的写法」；脚本正文里的照旧：还不在的按「不存在」记检出，已经在的读盘上那份）。
#   几个源的 cat（`cat a b > d`）、`cat 源 >> d`、`cat 源 | tee d` 里有盘上不在、又不是同一条命令里前面认出写出的源：读不到它的内容，那个目的照认不出、执行它时照拒
#   （偏严：cat 不看源在不在那一类照拒）；拒绝说明点名不在的源与改法
#   （PROJECT_CODE_GUARD_BREAK=cat-missing-source-unexplained 下不点名；cat-missing-source-allowed 下照把在的源接起来当写出的内容判）。
#   正文不带引号的 heredoc（`<<EOF`）写进文件之前 shell 还要展开 `$` 与命令替换，这里判的是展开之前的原文。
# 命令位置上的词里带没展开的 `$`、反引号、括号、引号残片、重定向、通配字符、逗号或等号（NOT_A_LITERAL_PATH）的，不当脚本路径去读；
#   命令替换 `$(…)` 与反引号整段留在所在的词里，里面的命令按一条完整的命令另判（lib_shell_words.py）。
# 喂给从标准输入读命令的 shell（bash / sh 这类不带 -c、不带脚本，或带 -s）的字面文本当命令判：here-string（`bash <<< '…'`）与管道前一段是 printf、echo、
#   读 heredoc 的 cat 的（`printf '…' | bash`），同 bash -c 一样挂它的那条命令上的包装（内存包装、目录、变量）算在里面（lib_shell_words.py 的 shell_standard_input）；
#   标准输入是盘上的文件（`bash -s < 文件`、`bash < 文件`、`sh -s < 文件`、`cat 文件 | bash`）照 `bash 文件` 读那份文件判（lib_shell_words.py 的 shell_standard_input_files）。
#   喂进去的不是字面（here-string 或 printf / echo 的参数里有变量、命令替换，管道前一段是 grep 这类别的命令）认不出：命令文本这一层照「认不出起什么的写法」拒、照记一条检出
#   （PROJECT_CODE_GUARD_BREAK=unjudged-shell-input-allowed 下只记不拒）；读进来的脚本正文里的照常执行、记检出。
# 判不了的几样都照常执行，stderr 报一句、记一条检出（结局「只记」）：读不到共用模块、decide() 抛异常、hook 输入不是 JSON 对象（诊断带开头的字节，会话号读不到照实写读不到）、
#   读不到或看不全的脚本、脚本正文里喂给 shell 的不是字面的。
#   钩子 bash 这一层起不来（python3 起不来、/dev/fd/3 读不到、python 在 main() 之前崩了，退出码不是 0、2）时，bash 照「放行但必记检出」往检出记录追加一行、退 0
#   （会话号、agent 从输入里按字面取；弄坏开关 PROJECT_CODE_GUARD_BREAK=bash-layer-unrecorded 下不记，照旧把 python 的退出码交出去）。
#   PATH 上没有 python3 的，preflight 判不了运行条件、会退 78 什么都不记：钩子那种起法（不带参数）在 preflight 之前就照「放行但必记检出」记一条、退 0
#   （弄坏开关 PROJECT_CODE_GUARD_BREAK=python-missing-unrecorded 下照旧交给 preflight 退 78）。
# 命令词带通配（`*`、`?`、`[`）的，判定照 bash 展开、按展开出来的那几份的名字判（外部程序名单里的程序，写成通配的包装照剥；
#   lib_project_code_runs.py 文件头「命令词带通配」），展开出来的脚本照样不读正文。
#   带方括号里的 POSIX 字符类、等价类、整理符号的（`[[:alpha:]]`、`[[=h=]]`、`[[.h.]]`）交给 bash 自己展开（python glob 不认），判不出展开成哪一份的命令词照拒。
# 命令词与参数里的花括号（`{a,b}`、`{1..3}`）照 bash 展开之后再判，展开出来有一条判成哪一类（外部程序、本仓的代码）就照它拒
#   （`{cargo,test}`、门禁 `crash-point-replay.{sh,}`；lib_project_code_runs.py 文件头 classify 一段），花括号展开出来的脚本同样不读正文。
# 命令词写在变量里、同一段命令文本里前面赋过字面值的（`S=.claude/gate.d/<那一道>; bash "$S"`、`G=…; bash "$G/<那一道>"`、export 的同样认）：先展开再判，
#   展开出来的脚本照读（lib_project_code_runs.py 的 command_word_expanded）。
# 认不出的写法（认不出的一律拒，偏严）：
#   命令文本这一层（Bash 调用的命令、它 bash -c / eval / 命令替换 / here-string 与管道喂给 shell 的字面、剥掉启动器之后的那几段；不是读进来的脚本正文）里下面几种，
#   照「认不出起什么的写法」这一类（kind unrecognized）拒：主 agent 与子 agent 一样拒，拒绝说明写改法：
#   ① 命令词、bash / sh / source 起的脚本路径、python 起的脚本路径（python 起的脚本正文不读，路径照样要认得出；lib_project_code_runs.py 的 LAUNCHED_FILE_NAMES）
#      由变量、命令替换、位置参数（`"$@"`、`$1`、`$0`）、进程替换拼出而执行前展开不了的，或展开出来夹着空白的
#      （`c='cargo test'; $c`、`$(which cargo) test`、`set -- …; "$@"`、`bash <(…)`、`source <(…)`、`bash -c "$(cat 文件)"`、`eval "$c"`、`for f in …; do bash $f; done`）；
#      前面赋过字面值的照展开再判（下面「命令词写在变量里」）；命令词是 `"$CLAUDE_CODE_EXECPATH"`、`"${CLAUDE_CODE_EXECPATH}"` 的（会话自己那份可执行文件）不算认不出：
#      检出钩子 ⑬ 认它是起 claude 会话，交那一道判（复跑分类报告 d137-replay-classify「四」两道口径；弄坏开关 LIB_PROJECT_CODE_RUNS_BREAK=claude-executable-path-unrecognized）；
#   ② cargo test / run / bench（连同 nextest、miri、llvm-cov、hack、mutants）的子命令或参数写成展开不了的变量、命令替换、位置参数的（`cargo $(echo test --workspace)`、
#      `bash -c 'cargo "$@"' _ test --workspace`；`--` 之后给测试二进制的、--target-dir / -j / --color / --message-format 的值除外）；前面赋过字面值的先展开
#      （`F=--workspace; cargo test $F` 照全量拒；lib_project_code_runs.py 的 cargo_arguments_expanded、unrecognized_launch）；
#   ③ 同一条命令里前面一条认不出写法的命令写过（重定向的目标是它，或参数、heredoc 正文里写着它，又不是只读的命令）、后面执行的脚本（
#      unrecognized_write，弄坏开关 PROJECT_CODE_GUARD_BREAK=unrecognized-writes-allowed）；
#      python3 -c 的代码、喂给 python3 - 的 heredoc 正文里没有任何能写文件的东西的（PYTHON_WRITE_MARKER），代码里的字样不算点到后面执行的脚本：管道里读 JSON 打字的
#      `python3 -c "…d['task']…"` 不是写了 `task-server/task`（python_filter_code，弄坏开关 PROJECT_CODE_GUARD_BREAK=python-filter-code-judged）；
#      剥掉之后里面那条接着判的启动器（systemd-run、chrt、ssh、/usr/bin/time、strace、prlimit、numactl、unshare 这一类）只算它自己的词点到的，里面那条起的脚本不算它写过
#      （d137-replay-classify 误拒 E2；launcher_own_words，弄坏开关 PROJECT_CODE_GUARD_BREAK=launcher-recorded-as-writer）；task-server/task submit 的客户端同样只算
#      `submit`、`--cwd <目录>`、`--` 这几个词点到的，`--` 之后那条起的脚本不算它写过（收尾批 #3 误拒：子 agent 交 `-- bash <不按名字判的脚本>` 被认成 task 写过那份脚本；同一个开关）；heredoc 写出的目标与记写者时的重定向目标同一套，
#      前面赋过字面值的变量照代入（误拒 E3；written_target_word，弄坏开关 heredoc-targets-unexpanded）；
#   ④ 命令词不带斜杠的：这一条设了 PATH（命令前、env、export）的照它找，没设的只找 .sh 结尾的名字（钩子自己的 PATH），找到的不在系统目录下照读正文（
#      path_lookup_script，PROJECT_CODE_GUARD_BREAK=path-scripts-unread）；PATH 写成展开不了的值照认不出拒；
#   ⑤ bash / source 起的脚本路径带通配的照当前目录展开、读排在头一个的那一份（PROJECT_CODE_GUARD_BREAK=glob-scripts-unread）；
#   ⑥ 喂给 shell 的标准输入不是字面的（见上）；
#   ⑦ shell 启动时先读的文件路径由变量、命令替换拼出而解不出的（`BASH_ENV=$(…) bash -c true`、`bash --rcfile $(…) -i`、`ZDOTDIR=$(…) zsh`、`HOME=$(…) bash -i`；写在命令前、env 的参数里、
#      前面 export / declare -x 的都算；字面路径、前面赋过字面值的变量读得到的，照 `bash 脚本` 读正文判，下面「shell 启动时先读的文件」一段）；
#   ⑧ 直接执行的脚本 #! 行指到认不出的解释器（不是 shell、也不在 lib_shell_words.py 的 NON_SHELL_INTERPRETER 里），而这条命令的环境里设了 BASH_ENV 或 ENV 的（认不出它读不读）。
#   ⑨ 复合命令与函数把标准输入转交给里层读标准输入的 shell 而认不出转交的是什么：`<&3` 这类别的描述符、here-string、heredoc 挂在 exec、收尾词（`}`、done、fi、esac、`)`）
#      或这段文本里定义的函数调用上，管道接进 `( … )`、if / case / 循环（`… | ( bash )`）；`< 文件` 挂在 fi / esac 收尾词与函数调用上的照 `bash 文件` 读那份文件判
#      （lib_shell_words.py 的 compound_redirected_stdin_files；弄坏开关 LIB_SHELL_WORDS_BREAK=compound-stdin-closers-unseen）；
#   ⑩ make -C 的目录、git -c 的配置名或别名值写成这段文本里代入不了的变量、命令替换的（前面赋过字面值的照代入；`make -C "$x"`、`git -c "$x" 子命令`；
#      lib_shell_words.py 的 embedded_launches、scan_git，弄坏开关 LIB_SHELL_WORDS_BREAK=embedded-option-variables-unexpanded）；
#   ⑪ 命令词写成变量而这个变量的值认不准的：在圆括号子 shell、管道段、放后台的命令里改过，或在 `||` 之后、if / case / 循环 / 函数体里把记过的值改成别的
#      （`C=cargo; (C=ls); $C test`；一串 `&&` 里接着用的照新值；lib_shell_words.py 的 VariableScopes，弄坏开关 LIB_SHELL_WORDS_BREAK=variable-scopes-unseen），照 ① 拒。
#   别名照展开再判（认不准开没开展开的照开着算）：`bash -O expand_aliases -c`、`sh -c`、带 -i、posix 模式、读启动文件的 shell 里定义的别名（弄坏开关 LIB_SHELL_WORDS_BREAK=alias-start-options-unseen），
#   eval、trap、命令替换里用外面先定义的别名、eval 里定义的别名（alias-nested-texts-unseen），展开出来的命令照这张表判。
#   弄坏开关 LIB_PROJECT_CODE_RUNS_BREAK=unrecognized-launches-allowed 下 ①② 不判。
#   读进来的脚本正文里 ①② 只判一种（例：草稿里一份脚本 `R=$1; cmds=("bash $R/.claude/hooks/… --selftest" …)`、
#   `… ${cmds[$i]}` 起改过的钩子自证，读正文却认不出就会放行）：命令词、bash / sh / source / python 起的脚本路径由变量、命令替换、位置参数拼出、执行前解不出，
#   而它连同它点到的变量在这份正文里的定义（`名=值`、`名=(…)`、`for 名 in 词…`，往下点到的接着收）、点到位置参数时连同起这份脚本的那一条的参数，里面写着
#   自证（`--selftest`）、门禁阶段（`.claude/gate.d/<名>.sh`）、入口脚本（按名字判的那几份、Verus 两份判法、广谱变异驱动、prove-red.sh）或 cargo 的，照「认不出起什么的写法」拒；
#   字面命令词 cargo 而起编出来代码的子命令或参数解不出的同样（lib_project_code_runs.py 的 script_body_unresolved_launch；弄坏开关 LIB_PROJECT_CODE_RUNS_BREAK=script-body-launches-unjudged）。
#   边界（普通脚本照常起）：定义里与起它的参数里都点不到这几样的 `"$@"`、`$cmd`（转发参数、拼普通命令）照旧不判；脚本起它自己（`bash "$0" --selftest`）不判；
#   正文所在的脚本落在本仓里（整条路径逐级不是符号链接），或与本仓同一相对位置那一份逐字节相同的拷贝，这一条不判（仓里 multi-host-run.sh、
#   merge-and-verify.sh 这类本来就经变量起包装与命令；script_in_own_repository）。直接写 `bash "$R/…/x.sh" --selftest` 的按登记表照拒（解到的不是登记的那一份）。
#   脚本正文按 SCRIPT_POLICY 那一段读，下面「脚本里自己的转发函数」另认。
# shell 启动时先读的文件（认不出的一律拒，偏严）：bash 起非交互 shell 先读 `BASH_ENV` 指的文件并执行，
#   所以 `BASH_ENV=<脚本> bash -c true` 等于起了那份脚本；交互的 bash（带 -i）不读 `BASH_ENV`，读 `--rcfile` / `--init-file` 指的文件（没给读 HOME 下的 .bashrc，`--norc` 不读）；
#   sh / dash / ksh 与 posix 模式的 bash（`--posix`、`-o posix`）读 `ENV`（实测 bash 5.2、dash：非交互的 sh 与 --posix 其实不读 ENV，只有带 -i 才读，这里照偏严算）；
#   zsh 读 ZDOTDIR（没设取 HOME）下的 .zshenv、.zshrc（`-f`、`--no-rcs` 不读；.zshrc 其实只有交互才读，照偏严都读；本机没有 zsh，这一项没起真 shell 实测，照文档）。
#   变量写在命令前、`env` 的参数里、前面 `export` 的，`declare -x` / `typeset -x` / `local -x`，先赋值后 `export 名字`、先 `export 名字` 后赋值，`set -a` 之后赋的都认
#   （lib_shell_words.py 的 startup_file_commands，交一条 launcher 为 "startup" 的命令；`export -n`、`declare +x` 撤掉的不算）：
#   路径是字面的（`~` 与前面赋过字面值的变量先展开）、读得到的，读正文判，同 `bash 脚本`（正文里 source 的库、再起的脚本照读）；路径不在的什么都不读、不记检出（shell 也不报错）；
#   路径解不出（命令替换、没赋过的变量）的，命令文本这一层照「认不出起什么的写法」拒（lib_project_code_runs.py 的 unrecognized_launch）；`bash -n`、`--version`、`--help`、
#   `env -i` 清掉变量之后、`unset` 之后的不判。读进来的启动文件里再起 bash，变量照样带进去（真 bash 会无限递归），绕回读过的那份按绕回算、不再读。
#   直接执行的脚本（命令词是脚本路径，`BASH_ENV=<文件> ./other.sh`）：读到它之后看开头的 #! 行（executed_file_head、lib_shell_words.py 的 executed_script_startup），#! 指到 bash（含 env bash）
#   或没有 #! 的文本文件（从 bash 里起是 bash 另起自己，实测读 BASH_ENV）读 BASH_ENV，指到 sh 读 ENV，指到 zsh 读 zsh 的；指到 python 这类不是 shell 的不读；
#   指到别的而环境里设了 BASH_ENV 或 ENV 的，认不出它读不读，照认不出拒；启动文件接在这条命令之后当被起的脚本判（pending）。
#   弄坏开关 LIB_SHELL_WORDS_BREAK=startup-files-unread 下一个启动文件都不认；declared-exports-unseen 下环境只认 `export 名字=值`；
#   interactive-startup-files-unread 下 -i 不另看；zsh-startup-files-unread 下 zsh 不读；executed-script-startup-unread 下直接执行的脚本不读（逐格见下面开关清单）。
#   登录 shell（`-l`、`--login`、`exec -l`、`exec -a -名字`）另读 /etc/profile 与 HOME 下的 .bash_profile、.bash_login、.profile（sh 一类与 posix 模式只读 .profile）与 .bash_logout，
#   交互的 bash 另读 /etc/bash.bashrc（`--norc` 不读，`--rcfile` 只换掉 .bashrc），zsh 另读 /etc/zsh/ 下的（lib_shell_words.py 文件头「登录 shell」一段，
#   读哪几份照本机 bash 5.2、dash 实测）；/etc/profile 里循环变量起的 source（`. $i`）照脚本正文里解不出的路径办、不读。弄坏开关 LIB_SHELL_WORDS_BREAK=login-profiles-unread、
#   system-startup-files-unread 下不读这两类（逐格见下面开关清单）。自证里系统级的目录换成样本目录，不读本机的 /etc。
#   非交互的 shell source 的文件（启动文件、`.` / source 读进来的）读到顶层的 `case $- in *i*) ;; *) return;; esac`、`[[ $- != *i* ]] && return` 这类守卫就 return 了，守卫之后的不读
#   （lib_shell_words.py 的 non_interactive_return_offset；跑它的 shell 交不交互照切词记的 CommandAtPosition.interactive：带 -i、不带 -c 也不带脚本的照交互算，直接执行的看 #! 行；
#   例：非交互的 `bash -lc` 读登录 shell 启动文件再 source ~/.bashrc，守卫之后的 bash-completion 这类大文件照读全份，每次多两三秒、多记一条检出）。
#   弄坏开关 LIB_SHELL_WORDS_BREAK=non-interactive-return-unseen 下整份照读。
# cd 之后的相对路径脚本：cd 的目标里 $PWD、$OLDPWD、`cd -`、起头的 `$(git rev-parse --show-toplevel)` 与 `$(pwd)` 照算出的目录
#   （lib_shell_words.py 的 changed_directory）；cd 到盘上不在的目录，后面跟 `&&` 的那一串与 `cd 目录 || exit` 之后同一层的（cd 失败就不跑；CommandAtPosition.cd_failure_skips）
#   照 cd 成功了的那个目录判（同一条命令里前面写出的照读，不在的照「脚本不存在」只记），`;`、`||`、换行、括号之后照跑的照「目录不在、cd 会失败」拒；
#   弄坏开关 LIB_SHELL_WORDS_BREAK=cd-known-targets-unexpanded、cd-failure-guards-unseen（逐格见下面开关清单）。
# 别的程序里起的命令（lib_shell_words.py 的 embedded_launches，认哪几种程序、哪几种写法写在它的文件头）：
#   python / perl / ruby / node / php / lua 的代码串与喂进标准输入的字面代码、awk 的程序、sed 的 e 命令、tclsh / expect、make --eval 与喂进标准输入的 Makefile、git 的别名与带命令的配置、
#   tar / ssh / less / vim 带命令的选项里，字面写着的命令照一段命令文本接着判（同 bash -c）；awk -f、sed -f、Makefile 读正文交 program_text_launches 切出命令，照脚本正文那样判
#   （program_file_text；文件都不在的不判，程序自己报错；路径写成前面赋过字面值的变量的照代入，d137-replay-classify 误拒 E5，弄坏开关 PROJECT_CODE_GUARD_BREAK=program-file-variables-unexpanded）；起了命令而写法认不出的（变量、拼接、动态求值），命令文本这一层照「认不出起什么的写法」拒，读进来的正文与程序文件里的照常执行、记检出。
#   python / perl / ruby / tclsh 起的脚本文件不读（共用约束「写」一节由子 agent 自己守）。
#   弄坏开关 LIB_SHELL_WORDS_BREAK=embedded-launches-unseen、PROJECT_CODE_GUARD_BREAK=program-files-unread、embedded-unjudged-allowed（逐格见下面开关清单）。
# 脚本里自己的转发函数（`run() { "$@"; }`、`run_logged() { local log=$1; shift; "$@" > "$log"; }`，函数体里有一条命令的命令词是位置参数；lib_shell_words.py 的 launcher_functions）：
#   命令文本与读进来的脚本正文里都认，调用的参数从每一处起的一段各当一条命令判（例：仓副本里经 crash-point-replay.sh 自己的 run_logged 起 验证包）；
#   弄坏开关 PROJECT_CODE_GUARD_BREAK=launcher-functions-unjudged 下不认。
# 名单外启动器（xargs、find -exec / -execdir / -ok、ssh、doas、script -c、unshare、setpriv、numactl、nsenter、su -c、runuser、sg、watch、parallel、ltrace、valgrind、gdb --args、
#   faketime、unbuffer、cpulimit、dbus-run-session、firejail、bwrap、proot、scl enable、chroot、cgexec；lib_project_code_runs.py 的 EXTRA_LAUNCHER_NAMES、command_under_extra_launcher）
#   照 /usr/bin/time 那一类剥掉、里面那条照判；xargs、find、parallel 读进来接进命令里的词换成没展开的占位词，照上面 ①② 认不出拒（`xargs bash`、`find -exec bash {} \;`）。
# 派发起法 `bash scripts/hosts/multi-host-run.sh --peer [选项…] -- <命令…>`（它只有这一种派发用法，不带 --peer 退 2；lib_project_code_runs.py 的 command_under_dispatch_launcher）
#   照 /usr/bin/time 那一类剥掉、`--` 后面那条照判，派发器本身是本仓的脚本，直接起照下面「经 task-server 起」① 拒；
#   `… -- <脚本> --selftest` 照里面那条的登记判，不当派发器自己的自证带了别的参数；multi-host-run.sh --selftest 照它自己的登记判。
#   host-dispatch.py 是 task-server 载入的判法库（命令行只认 --selftest），不在这里剥：带了别的参数照本仓的脚本拒。
#   编译目录那一道（下面「再一道」）交给 lib_workspace_cargo_lock.py 的判定库同样剥它（dispatch_launchers_seen）：里面裸起的 cargo 照拒。
#   弄坏开关 LIB_PROJECT_CODE_RUNS_BREAK=dispatch-launchers-unseen 下不剥（逐格见下面开关清单）。
# 看不见的（照常放行、不记检出）：读进来的脚本正文里变量拼出来的命令与脚本路径，定义与起它的参数里点不到 验证包测试目标名、自证、门禁阶段、入口脚本、cargo 的
#   （值从文件、命令输出、环境里来的：`$(cat 命令表)`、`read` 读进来的），与本仓里的脚本正文里经变量起的；eval 的参数；
#   喂给 python 这类非 shell 解释器的程序（`python3 x.py` 里用 subprocess 起的命令；heredoc 正文先剥掉）；make（Makefile 里起的命令）；
#   cd 到展开不了的变量路径之后（变量不在这段命令文本里赋值，或被 for / read / unset 改写过；前面赋过的与 $HOME 照展开）不带 -p 的 cargo test（认不出在不在工作区根、在哪个包里；
#   -p 点名 验证包的照判）、--manifest-path 写在变量里指到工作区根或探针包的与相对路径的脚本；
#   钩子自己的 PATH 上找到的、名字不带 .sh 的脚本（本机用户 bin 目录下的 cargo 包装这类照程序算）；source 进来的文件里 export 的变量对外层后面命令的影响；
#   .cargo/config.toml 与 `--config <文件>` 里定的别名与 runner；拷走改名、又不点名登记的用例函数的测试二进制（看门狗在进程这一层同样认不出）；
#   ssh 起的命令在别的机器上跑，这里照本机那一条判（偏严）。
# 经 task-server 起（加任务、取消任务都走标准接口，不绕过；子 agent 也过 task-server）：
#   上面那张「谁能跑哪一类」的表之外另判三样，钩子入口照 task_server_decision 判（decide 是那张表，它先判）。
#   ① 一切跑项目代码的命令只认经 task-server/task 起，主 agent 与子 agent 同判：不经 `task-server/task submit -- …` 起的这几样执行前拒——会编的 cargo（build、check、clippy、test、run、bench、doc，
#      别名与插件照会编算；fmt、metadata、tree、clean 不编，直接跑：lib_project_code_runs.starts_a_build）、跑编译出来的代码（cargo test / run / bench、直接执行编出来的二进制：
#      runs_compiled_code）、本仓的代码（门禁阶段与格、实验装置与复跑、变异、各脚本的 --selftest 与样本档与弄坏开关、喂钩子、单测与本仓里别的脚本，仓副本与改名拷走的照认：
#      lib_project_code_runs.project_code_use）、经 run-with-memory-cap.sh / capped.sh 直接起的（剥掉的包装也算本仓的代码），与 lib_project_code_runs.classify 认出的每一样（要经服务的外部程序 qemu-system-*、herd7——只问版本的除外、e152-file-system-benchmark，判不出的通配，认不出起什么的写法）。
#      读进来的脚本正文里的照判（脚本路径与行号写进拒绝说明）。主 agent 那一份拒绝说明的出路是 `task-server/task submit -- <原命令>`，
#      再用 run_in_background 起 `task-server/task wait <号>`（TASK_SERVER_POLICY；照抄的那一条照同一张表再判过放行才给：范围声明挪到 submit 前面、开头一层内存包装去掉、
#      有 shell 写法的包进 `bash -c`、提交时才许不指定范围地跑的整道门禁与入口在 submit 后面带 --full-gate，submit_candidate 与 submit_loosening_flag）；子 agent 那一份的出路是把同一条（照主 agent 交它再判过放行才给）写进报告「要主 agent 跑的命令」一节交回
#      （subagent_task_server_policy，出路那一句是 lib_project_code_runs.SUBAGENT_HAND_BACK_WAY_OUT）。
#      直接跑的只剩：不是本仓文件的命令（只读的 git、ls、grep、cat、sed -n、wc、sha256sum、diff、du、ps 这类与草稿里自己写的、里面不跑项目代码的脚本）、语法检查（bash -n、python3 -m py_compile）、
#      lib_project_code_runs.py 的 TEXT_TOOLS_RUN_DIRECTLY（谁都能直接跑的文本工具，加 task-server/task 的子命令与 task-server/control 的只读子命令；子 agent 起客户端另照 ② 只许 status、log），
#      与只给主 agent 的 MAIN_AGENT_TOOLS_RUN_DIRECTLY（会话工具与几个服务的装、起停：起在服务的单元里就丢了会话身份或停不了服务自己）；两张名单与各自放行的参数形态只写在 lib_project_code_runs.py 里，这里不抄。
#      名单上的脚本整份放行，读进来的正文里那几行不另算（listed_scripts）。
#      经 task-server/task submit 交给服务的那一条：不判内存包装（服务替它排队、定额度、挑机器、分卡；套了包装、带了退役变量的由服务当场拒，钩子不另判），
#      主 agent 交什么都放；`--` 之后那条照 scan_uses 另判一遍，只为认出里面绕开服务的写法（③）。
#      编译目录那一道（下面「再一道」）不判交给服务的那一条（编译目录归服务）。
#   ② 子 agent（输入里有 agent_type）起 task-server/task 只许只读的 status、log：submit、wait、cancel、clean（与没写子命令的）一律执行前拒，交什么命令都一样
#      （带 --item 的、harness 测试、clippy / build、自证、样本档、弄坏开关、命名检查都算）；读进来的脚本正文里起的照判，python3 起的、仓副本里逐字节相同的客户端照认。
#      判定只在 lib_project_code_runs.subagent_task_server_refusal（run-scope-guard.sh 判子 agent 经 submit 交的验证命令也调它）；scan_uses 把它记成 DirectRun，allowed 是
#      SUBAGENT_CLIENT_LEVEL，主 agent 不判。服务分不出提交方是不是子 agent（task-server/task_server_lib.py 文件头「身份」），这一判只在钩子里
#      （records/2026-11-17-subagent账本自上而下清欠.md 第一百七十四节）。说明在 subagent_task_server_policy：要跑的命令拼成 submit 的整条、写进报告「要主 agent 跑的命令」一节交回。
#      lib_project_code_runs.TEXT_TOOLS_RUN_DIRECTLY 里客户端那一行不动（主 agent 照它直接起）。
#   ③ 绕开服务的写法，经不经 submit、谁起都拒（lib_project_code_runs.task_server_text_bypass、task_server_command_bypass；名字与路径全取自 task-server/task_server_lib.py）：
#      命令里点到库文件、签名密钥、socket（路径或文件名；直接 sqlite3 / python3 -c 开库的同样），python 代码里载入 task_server_lib，在 task-server 之外设
#      RUN_WITH_MEMORY_CAP_ADMITTED_BY_TASK_SERVER，直接起 task-server/task_server.py、attempt_runner.py，systemctl 停、禁、改 singlefs-task-server.service 与 singlefs-task-* 单元
#      （status、show 这类只读的放行；task-server/control 自己的 install / start / stop / check / watch 照 ① 的名单），直接删服务的编译目录（rm、find -delete / -exec rm、
#      python 的 shutil.rmtree 这类点到 TASK_SERVER_BUILD_DIRECTORY 或状态目录下 builds/ 的；lib_project_code_runs.build_directory_removal；出路：不用自己删，删掉它的树，服务扫到树不在了就删它那一整组；
#      树是任务 --cwd 解开符号链接后往上第一层有 .git 的那一层（哪一层都没有就是解开后的 --cwd 本身）：在自己建的仓副本里交的删那份副本，解开符号链接后往上哪一层都没有 .git 的删解开后的那个 --cwd 目录）。
#      说明在 TASK_SERVER_BYPASS_POLICY。
#      task_server_lib.py 载不进时这一样判不了，照常执行、记一条检出（结局只记）。
#   罩不住的写法在 lib_project_code_runs.py 文件头「经 task-server 起」那一段。
#   弄坏开关（只给自检证明会红，红的格写在下面开关表）：PROJECT_CODE_GUARD_BREAK=task-server-direct-allowed（① 的直接跑不判）、subagent-task-client-allowed
#   （② 不判：子 agent 起 submit、wait、cancel、clean 照放）、task-server-bypass-allowed（③ 不判）、task-server-copy-dropped（拒绝说明不给照抄的那一条）。
#   ④ 要临时 prune-server 的自证写成裸的单独一个 --selftest（.claude/hooks/selftest-temporary-service.tsv 里判定是 required 的脚本：--selftest 开跑先核临时服务，
#      环境里没有标记就退 2、什么都不判），经不经 submit、谁起都拒，说明给 `task-server/task submit -- python3 scripts/prune-server/prune_server_client_selftest.py
#      --with-temporary-service -- <解释器> <脚本> --selftest` 的整条（不带 --server-binary；lib_project_code_runs.temporary_service_bare_selftest）。
#      包装（给了 --server-binary 的）剥开重判里面那条时用 lib_project_code_runs.environment_under_temporary_service 补上标记，所以包在包装里的放行；scan_uses 把裸写的记成 DirectRun，
#      allowed 是 TEMPORARY_SERVICE_BARE_LEVEL，task_server_decision 单列一块说明（TEMPORARY_SERVICE_POLICY），不当「不经 task-server 跑项目代码」；submit 里那一条只收这一种
#      （交给服务的那条别的用法照旧不收）。判法不在 decide() 那一层（decide 判「谁能跑哪一类」，这一条与哪一类无关），那张表的格不受影响。
#      弄坏开关（只给自检证明会红，红的格写在下面开关表）：PROJECT_CODE_GUARD_BREAK=temporary-service-bare-allowed（④ 不判）、temporary-service-wrapper-not-marked
#      （剥开包装重判时不补标记）、temporary-service-submit-inner-unread（submit 里那一条不收）、temporary-service-remedy-dropped（说明不贴规矩与怎么办）。
#   ⑤ cargo clippy 的 `--` 之后没带 `-D warnings` 与 `.claude/singlefs-ai-sop/scripts/check.sh` 里 CODE_DISCIPLINE_LINTS 那张表的每一条 `-D clippy::<名>`（表每次开跑现读，
#      这一份钩子所在的仓里的那一份；整轮门禁与合入后验证 scripts/gate/merge-and-verify.sh 跑的 clippy 都带这张表，写手交的命令少了一条，合入后才红）：命令文本这一层
#      （Bash 调用的命令、submit 里 `--` 之后那条、bash -c、替换里的，不是读进来的脚本正文）里的 clippy 命令照拒，谁起都拒；只问帮助与版本（--help、-h、--version、-V）、
#      带 --fix 或 --explain 的、`--` 之后写成变量或命令替换的（认不出）不判；表的文件读不到、没有 CODE_DISCIPLINE_LINTS=( 那一段时不判。
#      表有那一段而写法读不成（只认 `-D clippy::<名>` 一行一条，`+=`、重新赋值、行末 `\`、花括号、数组元素、`-F` 这类一律读不成，全文在 lib_code_discipline_lints.py 文件头）时，
#      clippy 命令一概拒：说明点名读不成的那一行，出路写 check.sh 是上游副本、本仓不改、交上游改成读得成的写法（子 agent 写进报告交主 agent）。
#      表读得成时另拒两种削弱：`--` 之后在表里某条（或 warnings）最后一次 deny 之后又用 -A、-W、--allow、--warn 放行或降级它（点名一整组 clippy::pedantic 这类、warnings 的同样算），
#      与带 --cap-lints；先放行、后 deny 的写法 deny 照起作用，不拒。说明点名缺哪几条、被放行的是哪个词，给 `-- -D warnings …` 整段现读的写法
#      （lib_shell_words 里没有这一判，实现在本文件的 clippy_lint_table_missing、code_discipline_lints；表的读法在同目录的 lib_code_discipline_lints.py，检出钩子的 ㉓ 出路读同一份；
#      scan_uses 把它记成 CLIPPY_LINT_TABLE_LEVEL，submit 里那一条的也收）。
#      弄坏开关（只给自检证明会红，红的格写在下面开关表）：PROJECT_CODE_GUARD_BREAK=clippy-lint-table-unchecked（⑤ 不判）、clippy-lint-table-remedy-dropped（说明不给出路）、
#      clippy-lint-weakening-unchecked（不查 deny 之后的放行与 --cap-lints）、clippy-lint-unreadable-table-passed（表读不成时放行）。表读法的弄坏开关 LIB_CODE_DISCIPLINE_LINTS_BREAK 与另三族一样平常设了退 2。
#   ⑥ 门禁样本档（`stage-selftest.sh <阶段目录> --item <阶段>`）点名的阶段要临时 prune-server 而外面没套包装：样本档进门先核临时服务，环境里没有标记就退 2、一份样本都不判，
#      经不经 submit、谁起都拒，说明给 `task-server/task submit -- python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash <原命令>` 的整条（不带 --server-binary）。
#      哪几道要套每次从阶段源码现读（阶段源码里调了临时服务核对的；lib_project_code_runs.temporary_service_bare_stage_samples，认法同 ④，不另写名单），
#      只对点名了 --item 的判（不带 --item 的整目录样本档、--list-items 不判）。包装（给了 --server-binary 的）剥开重判里面那条时补标记，所以包在包装里的放行；
#      scan_uses 把裸写的记成 DirectRun，allowed 是 TEMPORARY_SERVICE_STAGE_SAMPLES_LEVEL，task_server_decision 单列一块说明（TEMPORARY_SERVICE_STAGE_SAMPLES_POLICY），submit 里那一条的也收。
#      弄坏开关（只给自检证明会红，红的格写在下面开关表）：PROJECT_CODE_GUARD_BREAK=temporary-service-stage-samples-allowed（⑥ 不判）；
#      库一侧的开关：LIB_PROJECT_CODE_RUNS_BREAK=temporary-service-stage-samples-unjudged（判法一律交 None）、temporary-service-stage-samples-every-stage（点名的每一道都当要套）。
# 再一道，与谁在跑无关（主 agent 与子 agent 一样判）：主工作区里直接起的、编译目录会落进主仓的 cargo（build、test、check、clippy、run、bench、doc 这类会编的；
#   fmt、metadata、tree、clean 这类不编的放行）执行前拒绝（所有脚本自行编译、完成后自行删除、不预留生成物：主工作区的 target/ 删了又长）。
#   目录取 cd、env -C、-C、--manifest-path 之后的（包装剥掉之后：run-with-memory-cap.sh、capped.sh、nice、timeout、env、bash -c、/usr/bin/time 这一类），
#   只判落在主仓（project_root）那棵主工作树里的，仓副本、git worktree 加出来的树不判；--target-dir、CARGO_TARGET_DIR、CARGO_BUILD_TARGET_DIR、--config build.target-dir
#   或配置文件的 build.target-dir 指到仓外的放行，经 scripts/hosts/run-build-directory.sh 包一层起的放行（它给这一次一个自己的编译目录、跑完删）；
#   拒绝说明给出在第一个 cargo 前插上包一层的那一条。入口脚本自己起的 cargo 不经这一道（不读脚本正文）。命令文本本身里只定义、调不到的 `function 名字 { …; }` 的函数体
#   与项目代码那一处同一个口径不判（拆开拼写调到的、认不准的照判：lib_shell_words.py 的 function_never_called）。判定在 lib_workspace_cargo_lock.py 的
#   workspace_cargo_target_refusal（用 cargo_invocations 取每一条 cargo，文件头「编译目录」一段）；弄坏开关 WORKSPACE_CARGO_LOCK_BREAK=bare-cargo-allowed 时不判这一道，自检必须判红。
# 判定（一条命令是不是外部程序、认不认得出起什么、起的是不是本仓的代码）在同目录的 lib_project_code_runs.py；
# 切词、切简单命令、认命令位置、剥前缀与包装、跟 cd 与 export 在同目录的 lib_shell_words.py（与 bash-command-detector.sh 共用），由 lib_project_code_runs.py 导入。
# 两份都按文件路径导入，这里只留谁能跑什么、读脚本与拒绝；lib_shell_words.py 的函数在 .claude/hooks/ 别的文件里再定义一份，门禁 code-tooling 的 agent-write-scope 格判红。
# 读不到它们时这条命令照常执行，stderr 报一句并记一条检出。
#
#   project-code-guard.sh             # 从 stdin 读 hook 的 JSON；拒绝退出 2，放行退出 0
#   project-code-guard.sh --selftest  # 在临时工作区里走一遍拒绝与放行（连同 lib_project_code_runs.py 的自检）；
# 弄坏开关 PROJECT_CODE_GUARD_BREAK=<项>（几个用逗号连；项名写错的，自检「PROJECT_CODE_GUARD_BREAK 里的项都认得」一格红），设了哪一项自检都必须判红。
# 逐项写开关名、弄坏哪一处、打开后自检哪几格红（格名是 selftest 里 cases 与 stdin 那几格的说明，与自检失败行「✗ 自检：」之后、「应当是」之前那一段相同；
# cases 里写了该记几条检出、该读进去几份脚本的，各另成一格，格名是说明后面接「：记几条检出」「：读进去几份脚本」）：
# 这份自证转出的失败行有三种句首：「✗ 自检：」（本文件 cases 与 stdin 那几格，点名写格名）、「✗ lib_project_code_runs 自检：」（lib_project_code_runs.py 的自证转出的，点名写成「lib_project_code_runs 自检：」加格名）、
#   「✗ 自检：」之后接「：命令 '…'」的（lib_workspace_cargo_lock.py 的自证转出的，点名写到「：命令」为止）；点名的各段按次序出现在同一行里就算照出，所以一项可以照出多行，
#   几百行才写得下的（disable-check）只写它们共有的句首「✗ 自检：」。
#   PROJECT_CODE_GUARD_BREAK=disable-check  scan_decision() 一进来就放行，整道闸不判（外部程序、认不出的写法、经 task-server 那几样都不拒）：红的格是 cases、stdin、编译目录、经 task-server 那几组里该拒（应当是 2，实际 0）的几百格与它们带的「：记几条检出」「：读进去几份脚本」两种衍生格，逐格写不下，用一项「✗ 自检：」照出每一行（实测）
#       「✗ 自检：」
#   PROJECT_CODE_GUARD_BREAK=ignore-written-scripts  record_files_written() 不记，同一条命令里写出的脚本不认（执行时按盘上那份判，还不在的记「不存在」）：实测红的格（照失败行原文）
#       「主 agent cp X{,.bak} 拷出再起全量脚本照拒」「主 agent cp X{,.bak} 拷出再起全量脚本照拒：记几条检出」「主 agent cp X{,.bak} 拷出再起全量脚本照拒：读进去几份脚本」「主 agent cp 之后按 // 开头的路径起全量脚本照拒：读进去几份脚本」「主 agent cat 01> 拷出再起全量脚本照拒：读进去几份脚本」「主 agent cat 源 "2"> 拷出（引号里的 2 是源）再起全量脚本照拒：读进去几份脚本」「主 agent cat 3> 再 >&3 拷出再起全量脚本照拒：读进去几份脚本」「主 agent heredoc 写文件：cat <<EOF 3> 再 >&3 写出再起全量脚本照拒：读进去几份脚本」「主 agent cp 带步长、两端宽度不一的补零序列拷出再起全量脚本照拒：读进去几份脚本」「主 agent 同一条命令里 cp 再 mv 改名再起全量脚本照拒：读进去几份脚本」「主 agent cat > 拷出、另把 stderr 丢进 /dev/null 再起全量脚本照拒：读进去几份脚本」「主 agent ln -s 的相对源从链接所在的目录算、经它起全量脚本照拒：读进去几份脚本」「主 agent cp 补零的序列拷出再起全量脚本照拒：读进去几份脚本」「实现员 heredoc 写出 cargo run 再起它、这一层放行：读正文：读进去几份脚本」「实现员 cat > 文件 <<EOF 写出含 --all 的脚本再 bash 起它：读进去几份脚本」「实现员 cat <<EOF > 文件 写出含 --all 的脚本、chmod 之后 ./ 执行：读进去几份脚本」「实现员 tee 文件 <<EOF 写出含 --all 的脚本再 source：读进去几份脚本」「实现员 <<-EOF（去制表符）写出含 --all 的脚本再 bash 起它：读进去几份脚本」「实现员 cd 进子目录写出、回来按相对路径起：读进去几份脚本」「实现员 heredoc 写出的脚本里再写出、再起含 --all 的脚本：读进去几份脚本」「实现员 bash -c 里写出含 --all 的脚本再起它：读进去几份脚本」「实现员命令替换里写出含 --all 的脚本再起它：读进去几份脚本」「实现员 heredoc 写出的脚本里写出另一份，外层后面再起那一份：记几条检出」「实现员 heredoc 写出的脚本里写出另一份，外层后面再起那一份：读进去几份脚本」「实现员先写出含 --all 的一行、再 >> 追加一行轻的：前面那行还在：读进去几份脚本」「实现员 tee -a 往盘上含 --all 的 s.sh 追加一行轻的：盘上那几行还在：读进去几份脚本」「实现员 heredoc 写出 python 程序再 ./ 执行：不读也不记」「实现员 cp 盘上含 --all 的 s.sh 出来再起：读源文件：读进去几份脚本」「实现员 cp -f 到目录里再起：读源文件：读进去几份脚本」「实现员 cp 同一条命令里 heredoc 写出的脚本再起：读进去几份脚本」「实现员先起盘上轻的 inner-rel.sh、再 heredoc 盖成含 --all 的再起：写出的变了重判：读进去几份脚本」
#       「一百三十七：cat 源 >> 拷出再起，照写出的内容判：读进去几份脚本」「一百三十七：cat 轻脚本 &> 拷出再起，照内容判、放行」「一百三十七：cat 轻脚本 &> 拷出再起，照内容判、放行：读进去几份脚本」「一百三十七：cat 源 | tee 拷出再起，照写出的内容判：读进去几份脚本」「一百三十七：tee 目的 < 轻脚本 拷出再起，照内容判、放行」「一百三十七：tee 目的 < 轻脚本 拷出再起，照内容判、放行：读进去几份脚本」「一百三十七：cat 源里有不在的文件、&& 接起，照拒并点名不在的源」「heredoc 写出的脚本，拒绝说明写出它的路径与正文里的行号」「heredoc 写出的脚本里再写出的，逐层写出路径与行号」「stdin:heredoc 写出的轻脚本照常放行（退出码 0）」「实现员 heredoc 写出 cargo run 再起它、这一层放行：读正文」「d137 H8：install /bin/bash ./x; ./x 脚本 读正文照拒：读进去几份脚本」「d137 H8：cp /bin/bash ./x; ./x 轻脚本 放行」「d137 H8：cp /bin/bash ./x; ./x 轻脚本 放行：读进去几份脚本」「d137 H8：cp /bin/ls ./x; ./x 不是 shell、放行」「d137 H8：cp /bin/bash ./x; ./x -c 'echo hi' 无害放行」「d137 H7：多源 cat 两个源都在、都无害，放行、读写出的内容」「d137 H7：多源 cat 两个源都在、都无害，放行、读写出的内容：读进去几份脚本」「d137 H7：单源 cat 源在、起全量，照拒（控制组）：读进去几份脚本」「d137-r2 H5：mkdir 再 cd 进去跑同一条里写出的重脚本，照拒：读进去几份脚本」「d137 闸误拒 E3：t 赋过字面路径，cat > $t 读 heredoc 写出再 bash $t，放行」「d137 闸误拒 E3：t 赋过字面路径，cat > $t 读 heredoc 写出再 bash $t，放行：读进去几份脚本」「stdin:heredoc 写出的轻脚本 stderr 不报「不存在」、不记检出」「stdin:heredoc 写出的脚本的拒绝 stderr 写出它的路径与行号」
#   PROJECT_CODE_GUARD_BREAK=detection-outcome-dropped  检出记录不写结局（outcome：被拒 / 只记）：实测红的格（照失败行原文）
#       「stdin:被拒的检出记录写了结局「被拒」」「stdin:坏 JSON 放行（退出码 0）、stderr 写诊断、记一条检出」「stdin:不是 JSON 对象放行（退出码 0）、stderr 写诊断、记一条检出」「stdin:判的时候抛异常放行（退出码 0）、stderr 写诊断、记一条检出」「stdin:放行时的检出记录写了结局「只记」」
#   PROJECT_CODE_GUARD_BREAK=copy-without-run-scope  被拒时照抄的那条不带四段范围声明（SINGLEFS_RUN_SCOPE，写法在 run-scope-guard.sh 文件头）
#       「stdin:主 agent 直接跑整道门禁被拒时照抄的那条经 task-server 提交、四段范围声明写在 submit 前面（为什么是占位），照它再判放行」「经 task-server：照抄：实现员直接跑 harness 测试，照抄的那条经 task-server 提交」「经 task-server：照抄：带 && 的整条包进 bash -c」
#   PROJECT_CODE_GUARD_BREAK=copy-without-full-gate-flag  被拒时照抄的那条（提交时才许不指定范围地跑的整道门禁或入口）不带 --full-gate（submit_loosening_flag 一律交空串；经 submit 交上去，范围声明闸当场拒）：实测红的格（照失败行原文）
#       「stdin:主 agent 直接跑整道门禁被拒时照抄的那条经 task-server 提交、四段范围声明写在 submit 前面（为什么是占位），照它再判放行」「经 task-server：照抄：打出的每条照抄命令照 unnamed_run_problem 判都点了名」
#   PROJECT_CODE_GUARD_BREAK=copy-test-names-dropped  submit_candidate 改写照抄的那一条时丢掉 ` -- ` 之后的测试全名与 --exact（照抄的 cargo test 只点到测试目标，服务提交时当没点名拒）：实测红的格（照失败行原文）
#       「经 task-server：照抄：实现员直接跑 harness 测试，照抄的那条经 task-server 提交」「经 task-server：照抄：带 && 的整条包进 bash -c」「经 task-server：照抄：打出的每条照抄命令照 unnamed_run_problem 判都点了名」
#   PROJECT_CODE_GUARD_BREAK=failure-unrecorded  main() 里判不了的两支（hook 输入不是 JSON 对象、decide() 抛异常）只打 stderr、不记检出
#       「stdin:坏 JSON 放行（退出码 0）、stderr 写诊断、记一条检出」「stdin:不是 JSON 对象放行（退出码 0）、stderr 写诊断、记一条检出」「stdin:判的时候抛异常放行（退出码 0）、stderr 写诊断、记一条检出」
#   PROJECT_CODE_GUARD_BREAK=copy-by-cp-only  copy_pairs() 只认 cp：同一条命令里 ln / install / mv 拷出、`cat 源 > 目的` 写出的脚本不认，执行时按盘上那份判，还不在的记「不存在」：实测红的格（照失败行原文）
#       「主 agent cat 01> 拷出再起全量脚本照拒：读进去几份脚本」「主 agent cat 源 "2"> 拷出（引号里的 2 是源）再起全量脚本照拒：读进去几份脚本」「主 agent cat 3> 再 >&3 拷出再起全量脚本照拒：读进去几份脚本」「主 agent 同一条命令里 cp 再 mv 改名再起全量脚本照拒：读进去几份脚本」「主 agent cat > 拷出、另把 stderr 丢进 /dev/null 再起全量脚本照拒：读进去几份脚本」「主 agent ln -s 的相对源从链接所在的目录算、经它起全量脚本照拒：读进去几份脚本」「一百三十七：cat 源 >> 拷出再起，照写出的内容判：读进去几份脚本」「一百三十七：cat 轻脚本 &> 拷出再起，照内容判、放行」「一百三十七：cat 轻脚本 &> 拷出再起，照内容判、放行：读进去几份脚本」「一百三十七：cat 源里有不在的文件、&& 接起，照拒并点名不在的源」「d137 H8：install /bin/bash ./x; ./x 脚本 读正文照拒：读进去几份脚本」「d137 H7：多源 cat 两个源都在、都无害，放行、读写出的内容」「d137 H7：单源 cat 源在、起全量，照拒（控制组）：读进去几份脚本」
#   WORKSPACE_CARGO_LOCK_BREAK=bare-cargo-allowed  （不判编译目录）：实测红的格（照失败行原文）
#       「编译目录：主 agent 在主工作区裸起 cargo build：拒（退出码 2）」「编译目录：子 agent 经内存包装在主工作区起 cargo test：拒（退出码 2）」「编译目录：拒绝说明给出经 task-server 提交的出路」「编译目录：function 定义之后拆开拼写调用：拒（退出码 2）」「lib_workspace_cargo_lock.py 的自检」「编译目录：子 agent 经 multi-host-run.sh --peer 在主工作区起 cargo test、没给编译目录：拒（退出码 2）」
#       「编译目录：裸 cargo test：拒：命令」「编译目录：cd 进 crates/ 之后的 cargo build：拒：命令」「cd 进写在变量里的主工作区再起：拒：命令」「编译目录：cd 进 crates/ 的子目录（相对路径）之后的 cargo check：拒：命令」「编译目录：经内存包装起：拒：命令」「编译目录：capped.sh、nice、timeout、env 包着：拒：命令」「编译目录：bash -c 里起：拒：命令」「编译目录：/usr/bin/time 包着 bash -c 起：拒：命令」「编译目录：CARGO_TARGET_DIR 指到仓里：拒：命令」「编译目录：--target-dir 指到仓里的相对路径：拒：命令」「编译目录：仓里的配置文件给的 build.target-dir 在仓里：拒：命令」「编译目录：包装脚本外面又给了仓里的 CARGO_TARGET_DIR：拒：命令」「编译目录：CARGO_TARGET_DIR 写在算不出的变量里：放行并记一条：命令」「编译目录：function 定义之后拆开拼写调用：拒：命令」
#   WORKSPACE_CARGO_LOCK_BREAK=wrapper-not-recognized  （不认 run-build-directory.sh 包着的）：实测红的格（照失败行原文）
#       「编译目录：经 run-build-directory.sh 包一层起：放行（退出码 0）」「lib_workspace_cargo_lock.py 的自检」「编译目录：子 agent 经 multi-host-run.sh --peer 与 run-build-directory.sh 起：放行（退出码 0）」
#       「编译目录：经包装脚本起：放行：命令」「编译目录：内存包装里套包装脚本起：放行：命令」「编译目录：包装脚本里 bash -c 再 cd 进 crates/ 起：放行：命令」
#   WORKSPACE_CARGO_LOCK_BREAK=refusal-submit-dropped  （拒绝说明不写经 task-server 提交的出路与编译目录归树的说明）：红的格（实测）
#       「编译目录：拒绝说明给出经 task-server 提交的出路」「lib_workspace_cargo_lock.py 的自检」
#   WORKSPACE_CARGO_LOCK_BREAK=outside-target-not-recognized  （不认仓外的编译目录）：实测红的格（照失败行原文）
#       「编译目录：带仓外 CARGO_TARGET_DIR：放行（退出码 0）」「lib_workspace_cargo_lock.py 的自检」
#       「编译目录：包装脚本外面又给了仓里的 CARGO_TARGET_DIR：拒：命令」「编译目录：带仓外 CARGO_TARGET_DIR：放行：命令」「编译目录：先赋变量再拼仓外 CARGO_TARGET_DIR：放行：命令」「编译目录：export 了仓外 CARGO_TARGET_DIR 再起：放行：命令」「编译目录：经内存包装、env 给仓外 CARGO_TARGET_DIR：放行：命令」「编译目录：会话环境里有仓外 CARGO_TARGET_DIR：放行：命令」「编译目录：--target-dir 指到仓外：放行：命令」「编译目录：--config build.target-dir 指到仓外：放行：命令」「编译目录：CARGO_TARGET_DIR 写在算不出的变量里：放行并记一条：命令」
#   WORKSPACE_CARGO_LOCK_BREAK=non-building-refused  （不编的子命令也判）：实测红的格（照失败行原文）
#       「编译目录：cargo fmt：放行（退出码 0）」「lib_workspace_cargo_lock.py 的自检」
#       「编译目录：cargo fmt：放行：命令」「编译目录：cargo metadata、cargo tree、cargo clean：放行：命令」
#   LIB_SHELL_WORDS_BREAK=arguments-as-executed  `python3 -` 后面的词、`bash -n` 的目标当被执行的脚本：实测红的格（照失败行原文）
#       「bash -c -n 只查语法：放行」「工具实现员在门禁目录里 python3 - 读几份门禁：走完整的判法，放行」「一百三十七：bash -n 只查语法，不读 BASH_ENV：放行」
#   LIB_SHELL_WORDS_BREAK=shell-standard-input-unread  here-string 与管道喂给 shell 的文本不当命令再切、也不交判不了的说明：实测红的格（照失败行原文）
#       「实现员 here-string 喂给 bash 的是变量：认不出照拒、照记检出」「实现员 here-string 喂给 bash 的是变量：认不出照拒、照记检出：记几条检出」「一百三十七：grep 的输出经管道喂给 bash：认不出照拒」「一百三十七：grep 的输出经管道喂给 bash：认不出照拒：记几条检出」「lib_project_code_runs.py 的自检」「stdin:here-string 喂给 bash 的是变量认不出照拒（退出码 2）、stderr 点名判不了、记一条检出」「d137 H4：echo 代码 | bash /dev/stdin 照拒」「d137 H4：bash /dev/fd/0 <<< 代码 照拒」
#       「lib_project_code_runs 自检：标准输入：here-string 喂给 bash 跑 herd7」「lib_project_code_runs 自检：标准输入：printf 经管道喂给 bash 跑 herd7」「lib_project_code_runs 自检：标准输入：echo 经管道喂给 sh 跑 herd7」
#   LIB_SHELL_WORDS_BREAK=redirect-fd-dropped  `2> 文件` 也当改标准输出（cat 拷出取最后一个输出重定向）：实测红的格（照失败行原文）
#       「主 agent cat 3> 再 >&3 拷出再起全量脚本照拒：读进去几份脚本」「heredoc 写文件：cat <<EOF 2> 写的那一份拿不到正文：认不出写法写出再起，照拒：读进去几份脚本」「主 agent heredoc 写文件：cat <<EOF 3> 再 >&3 写出再起全量脚本照拒：读进去几份脚本」「主 agent cat > 拷出、另把 stderr 丢进 /dev/null 再起全量脚本照拒：读进去几份脚本」「cat > 拷出、起的是 2> 写的那一份：认不出写法写出再起，照拒：读进去几份脚本」
#       「一百六十六①：{ bash; } <&3 从别的描述符转交，认不出照拒：读进去几份脚本」
#   LIB_SHELL_WORDS_BREAK=link-source-from-current-directory  ln -s 的相对源照当前目录解：实测红的格（照失败行原文）
#       「主 agent ln -s 的相对源从链接所在的目录算、经它起全量脚本照拒：读进去几份脚本」「ln -s 的相对源在链接所在的目录里不在：认不出写法写出再起，照拒：读进去几份脚本」
#   LIB_SHELL_WORDS_BREAK=env-leading-equals-as-command  env 后面开头就是 = 的词当命令词：实测红的格（照失败行原文）
#       「主 agent env =1 开头就是 = 的赋值起全量照拒」「主 agent env -u 写在赋值前面照拒」
#   LIB_SHELL_WORDS_BREAK=shell-c-options-unparsed  bash -c 的代码取紧跟 -c 的词：实测红的格（照失败行原文）
#       「主 agent bash -c -- 的代码里起全量照拒」「一百三十七：bash -n 只查语法，不读 BASH_ENV：放行」
#   LIB_SHELL_WORDS_BREAK=builtin-non-builtin-run  builtin 后面不是内建的词照旧当命令词：实测红的格（照失败行原文）
#       「builtin 后面不是内建：bash 报错不跑，放行」
#   LIB_SHELL_WORDS_BREAK=uncalled-functions-run  只定义、调不到的函数体照命令判：实测红的格（照失败行原文）
#       「function 只定义不调用：函数体不跑，放行」「名字() { 函数体 } 只定义不调用：函数体不跑，放行」「一百三十七：函数体外引号里的花括号不当认不准的调用，只定义的函数体不判：放行」「编译目录：只定义、调不到的 function 函数体：放行（退出码 0）」「lib_workspace_cargo_lock.py 的自检」「名字() { 函数体 }」
#       「编译目录：只定义、调不到的 function 函数体：放行：命令」
#   LIB_SHELL_WORDS_BREAK=posix-uncalled-functions-run  只定义不调用的函数只认 function 关键字，`名字() { …; }` 的函数体照命令判：红的格（推的，没跑）
#       「名字() { 函数体 } 只定义不调用：函数体不跑，放行」
#   LIB_SHELL_WORDS_BREAK=shell-s-positionals-as-script  bash -s 后面的词照旧当脚本：实测红的格（照失败行原文）
#       「bash -s -- 后面的词是位置参数、不是脚本：放行」「bash -s -- 后面的词是位置参数、不是脚本：放行：读进去几份脚本」
#   LIB_SHELL_WORDS_BREAK=time-options-as-command  time 关键字后面的 -p / -- 当命令词：实测红的格（照失败行原文）
#       「主 agent time -p 起全量照拒」「主 agent time -- 起全量照拒」
#   LIB_SHELL_WORDS_BREAK=builtin-as-command  builtin 当命令词（后面的 source / cd 不认）：实测红的格（照失败行原文）
#       「主 agent builtin source 读进全量脚本照拒」「主 agent builtin source 读进全量脚本照拒：读进去几份脚本」
#   LIB_SHELL_WORDS_BREAK=shell-option-values-as-script  bash 的 --rcfile / --init-file 的值、-- 与 - 当脚本：实测红的格（照失败行原文）
#       「主 agent bash --rcfile 起全量脚本照拒」「主 agent bash --rcfile 起全量脚本照拒：读进去几份脚本」「主 agent bash -- 起全量脚本照拒」「主 agent bash -- 起全量脚本照拒：读进去几份脚本」「主 agent bash - 起全量脚本照拒」「主 agent bash - 起全量脚本照拒：读进去几份脚本」「主 agent bash -c -- 的代码里起全量照拒」「一百三十七：bash --rcfile 文件 -i，文件里起全量，照拒：读进去几份脚本」「一百三十七：bash --init-file 文件 -i，文件里起全量，照拒：读进去几份脚本」「一百三十七：bash --rcfile 不带 -i 是非交互，不读：放行」「一百三十七：bash --rcfile 指到的文件不在，什么都不读：放行：记几条检出」「d137 已知没认：系统级 bash --rcfile 轻文件 -i 照读 /etc/bash.bashrc，照拒：读进去几份脚本」
#   LIB_SHELL_WORDS_BREAK=function-keyword-as-command  function 关键字当命令词：实测红的格（照失败行原文）
#       「主 agent function 关键字定义的函数里起全量照拒」「主 agent function 定义之后引号拆开拼写调用照拒」「主 agent function 定义之后花括号拆开拼写调用照拒」「主 agent function 定义之后反斜杠拆开拼写调用照拒」「编译目录：function 定义之后拆开拼写调用：拒（退出码 2）」「lib_workspace_cargo_lock.py 的自检」
#       「编译目录：function 定义之后拆开拼写调用：拒：命令」
#   LIB_SHELL_WORDS_BREAK=braces-unexpanded  引号外的花括号不展开：实测红的格（照失败行原文）
#       「主 agent cp X{,.bak} 拷出再起全量脚本照拒」「主 agent cp X{,.bak} 拷出再起全量脚本照拒：记几条检出」「主 agent cp X{,.bak} 拷出再起全量脚本照拒：读进去几份脚本」「主 agent cp 带步长、两端宽度不一的补零序列拷出再起全量脚本照拒」「主 agent cp 带步长、两端宽度不一的补零序列拷出再起全量脚本照拒：记几条检出」「主 agent cp 带步长、两端宽度不一的补零序列拷出再起全量脚本照拒：读进去几份脚本」「主 agent cp 补零的序列拷出再起全量脚本照拒」「主 agent cp 补零的序列拷出再起全量脚本照拒：记几条检出」「主 agent cp 补零的序列拷出再起全量脚本照拒：读进去几份脚本」「引号写出的空串花括号展开之后留着、当命令词：放行」
#   LIB_SHELL_WORDS_BREAK=env-loose-assignments-as-command  env 后面名字不合变量名的 `名字=值` 当命令词：实测红的格（照失败行原文）
#       「主 agent env -- 带不合变量名的赋值起全量照拒」「主 agent env =1 开头就是 = 的赋值起全量照拒」「主 agent env -u 写在赋值前面照拒」
#   LIB_SHELL_WORDS_BREAK=eval-unread  eval 的字面代码不再切：实测红的格（照失败行原文）
#       「主 agent eval 字面代码里起全量照拒」「一百三十七：eval 变量里拼的命令照拒」
#       一百六十六①（推的，没跑）：「一百六十六①：shopt 开着时 eval 里用前面定义的别名，照拒」「一百六十六①：同一行 shopt、alias 之后 eval 里用别名（eval 跑的时候才切），照拒」
#   LIB_SHELL_WORDS_BREAK=heredoc-open-substitution-unseen  heredoc 挂在没收尾的 $( / <( 里的 bash 上时当数据剥掉：实测红的格（照失败行原文）
#       「主 agent $(bash <<EOF) 里起全量照拒」「主 agent <(bash <<EOF) 里起全量照拒」
#   LIB_SHELL_WORDS_BREAK=brace-empties-kept  花括号展开出来的空串留着（`{cargo,} test` 切成 `cargo '' test`）：实测红的格（照失败行原文）
#       「主 agent {,} 写在命令词前面起全量照拒」
#   LIB_SHELL_WORDS_BREAK=brace-sequence-unpadded  补零的整数序列不补零（lib_project_code_runs.py 的花括号展开调同一个 brace_sequence，一并不补）：实测红的格（照失败行原文）
#       「主 agent cp 带步长、两端宽度不一的补零序列拷出再起全量脚本照拒」「主 agent cp 带步长、两端宽度不一的补零序列拷出再起全量脚本照拒：记几条检出」「主 agent cp 带步长、两端宽度不一的补零序列拷出再起全量脚本照拒：读进去几份脚本」「主 agent cp 补零的序列拷出再起全量脚本照拒」「主 agent cp 补零的序列拷出再起全量脚本照拒：记几条检出」「主 agent cp 补零的序列拷出再起全量脚本照拒：读进去几份脚本」「lib_project_code_runs.py 的自检」
#       「lib_project_code_runs 自检：花括号展开与 bash 一致：{01..10}」「lib_project_code_runs 自检：花括号展开与 bash 一致：{-01..3}」「lib_project_code_runs 自检：花括号展开与 bash 一致：{1..-03}」「lib_project_code_runs 自检：花括号展开与 bash 一致：{0..00}」「lib_project_code_runs 自检：花括号展开与 bash 一致：{1..05..2}」「lib_project_code_runs 自检：花括号展开与 bash 一致：{01..100..33}」「lib_project_code_runs 自检：花括号展开与 bash 一致：{100..00..50}」
#   LIB_SHELL_WORDS_BREAK=double-quoted-substitution-unclosed  双引号里的 `$(` 当没收尾，顶层的 bash <<EOF 的正文当数据剥掉：实测红的格（照失败行原文）
#       「主 agent cd "$(git rev-parse --show-toplevel)" 之后 bash <<EOF 里起全量照拒」「主 agent FOO="$(pwd)" bash <<EOF 里起全量照拒」
#   LIB_SHELL_WORDS_BREAK=function-calls-literal-only  函数名拆开拼写的调用认不出（只数原文里的字面）：实测红的格（照失败行原文）
#       「主 agent function 定义之后引号拆开拼写调用照拒」「主 agent function 定义之后花括号拆开拼写调用照拒」「主 agent function 定义之后反斜杠拆开拼写调用照拒」「编译目录：function 定义之后拆开拼写调用：拒（退出码 2）」「lib_workspace_cargo_lock.py 的自检」
#       「编译目录：function 定义之后拆开拼写调用：拒：命令」
#   WORKSPACE_CARGO_LOCK_BREAK=uncalled-functions-judged  编译目录那一道照判只定义、调不到的函数体：实测红的格（照失败行原文）
#       「编译目录：只定义、调不到的 function 函数体：放行（退出码 0）」「lib_workspace_cargo_lock.py 的自检」
#       「编译目录：只定义、调不到的 function 函数体：放行：命令」
#   LIB_SHELL_WORDS_BREAK=redirect-fd-number-literal  `01>` 不算 1 号：实测红的格（照失败行原文）
#       「主 agent cat 01> 拷出再起全量脚本照拒：读进去几份脚本」
#   LIB_SHELL_WORDS_BREAK=redirect-fd-quoted-digits  引号写出的数字也当 fd 号；PROJECT_CODE_GUARD_BREAK=cat-concatenation-unseen  cat_concatenation() 不认几个源的 cat：
#       「主 agent cat 源 "2"> 拷出（引号里的 2 是源）再起全量脚本照拒：读进去几份脚本」「一百三十七：cat 源里有不在的文件、&& 接起，照拒并点名不在的源」
#   LIB_SHELL_WORDS_BREAK=redirect-fd-duplication-unseen  不跟 `n>&m`：实测红的格（照失败行原文）
#       「主 agent cat 3> 再 >&3 拷出再起全量脚本照拒：读进去几份脚本」「主 agent heredoc 写文件：cat <<EOF 3> 再 >&3 写出再起全量脚本照拒：读进去几份脚本」
#   LIB_SHELL_WORDS_BREAK=redirect-target-braces-expanded  重定向目标上的花括号照旧展开：实测红的格（照失败行原文）
#       「重定向目标花括号展开成几个词：bash 报 ambiguous redirect 不写，放行」「重定向目标花括号展开成几个词：bash 报 ambiguous redirect 不写，放行：记几条检出」
#   PROJECT_CODE_GUARD_BREAK=heredoc-write-every-redirection  heredoc_write_targets() 的输出重定向不看 fd 号、一律当写了正文：实测红的格（照失败行原文）
#       「heredoc 写文件：cat <<EOF 2> 写的那一份拿不到正文：认不出写法写出再起，照拒：读进去几份脚本」
#   LIB_SHELL_WORDS_BREAK=shell-long-options-after-short  ：实测红的格（照失败行原文）
#       「bash -c 之后的长选项：bash 报错不跑，放行」
#   LIB_SHELL_WORDS_BREAK=shell-syntax-only-code-run  ：实测红的格（照失败行原文）
#       「bash -c -n 只查语法：放行」
#       「一百三十七：bash -n 只查语法，不读 BASH_ENV：放行」「一百三十七：bash -n 只查语法，不读 BASH_ENV：放行：读进去几份脚本」
#   LIB_SHELL_WORDS_BREAK=env-options-after-assignment  ：实测红的格（照失败行原文）
#       「env =1 之后的 -u 是命令词：env 报错不跑，放行」
#   LIB_SHELL_WORDS_BREAK=brace-sequence-width-padded-ends-only  ：实测红的格（照失败行原文）
#       「主 agent cp 带步长、两端宽度不一的补零序列拷出再起全量脚本照拒」「主 agent cp 带步长、两端宽度不一的补零序列拷出再起全量脚本照拒：记几条检出」「主 agent cp 带步长、两端宽度不一的补零序列拷出再起全量脚本照拒：读进去几份脚本」「lib_project_code_runs.py 的自检」
#       「lib_project_code_runs 自检：花括号展开与 bash 一致：{01..100..33}」「lib_project_code_runs 自检：花括号展开与 bash 一致：{100..00..50}」
#   LIB_SHELL_WORDS_BREAK=ansi-c-quotes-unseen  ：实测红的格（照失败行原文）
#       「主 agent $'…' 里带 \' 之后 bash <<EOF 里起全量照拒」（推的）
#   LIB_SHELL_WORDS_BREAK=case-patterns-as-commands  ：实测红的格（照失败行原文）
#       「case 里 ;; 之后的模式词不当命令：放行」
#   LIB_SHELL_WORDS_BREAK=conditional-expression-split  ：实测红的格（照失败行原文）
#       「[[ ]] 里 && 之后的词不当命令：放行」
#   LIB_SHELL_WORDS_BREAK=extended-assignments-as-command  ：实测红的格（照失败行原文）
#       「数组元素赋值与 += 追加是赋值、不是命令词：放行」
#   PROJECT_CODE_GUARD_BREAK=unrecognized-writes-allowed  ：实测红的格（照失败行原文）
#       「别的命令带几个文件参数写到 > 文件不当 cat 接起来：认不出写法写出再起，照拒」「别的命令带几个文件参数写到 > 文件不当 cat 接起来：认不出写法写出再起，照拒：记几条检出」「heredoc 写文件：cat <<EOF 2> 写的那一份拿不到正文：认不出写法写出再起，照拒」「heredoc 写文件：cat <<EOF 2> 写的那一份拿不到正文：认不出写法写出再起，照拒：记几条检出」「cat > 拷出、起的是 2> 写的那一份：认不出写法写出再起，照拒」「cat > 拷出、起的是 2> 写的那一份：认不出写法写出再起，照拒：记几条检出」「ln -s 的相对源在链接所在的目录里不在：认不出写法写出再起，照拒」「ln -s 的相对源在链接所在的目录里不在：认不出写法写出再起，照拒：记几条检出」「实现员 cat -n 带选项写出：认不出，照拒」「实现员 cat -n 带选项写出：认不出，照拒：记几条检出」「实现员 cp 的源不在：认不出，照拒」「实现员 cp 的源不在：认不出，照拒：记几条检出」「实现员 printf 写出再起：认不出，照拒」「实现员 printf 写出再起：认不出，照拒：记几条检出」「一百三十七：echo 写出再起照拒」「一百三十七：echo 写出再起照拒：记几条检出」「一百三十七：python 写出再起照拒」「一百三十七：python 写出再起照拒：记几条检出」
#       「d137 H3：别名展开成起脚本（同一条里 alias 先提到脚本、认不出写法）照拒：读进去几份脚本」「d137 H7：多源 cat 有盘上不在的源（在的源起全量）、写出再起，照拒」「d137 H7：多源 cat 有盘上不在的源（在的源起全量）、写出再起，照拒：记几条检出」「d137 H7：多源 cat 有盘上不在的源（在的源无害），照拒」「d137 H7：多源 cat 有盘上不在的源（在的源无害），照拒：记几条检出」「d137 H7：源是 echo 写出的（认不出写法、盘上不在），照拒」「d137 H7：源是 echo 写出的（认不出写法、盘上不在），照拒：记几条检出」「d137 H7：cat 源 >> 追加、源盘上不在，照拒」「d137 H7：cat 源 >> 追加、源盘上不在，照拒：记几条检出」「d137 H7：cat 源 | tee 拷出、源盘上不在，照拒」「d137 H7：cat 源 | tee 拷出、源盘上不在，照拒：记几条检出」「一百三十七：cat 源里有不在的文件、&& 接起，照拒并点名不在的源」「经 task-server：python3 -c 往 task-server/task 写了再执行它，照拒」
#   PROJECT_CODE_GUARD_BREAK=python-filter-code-judged  ：python3 -c 的代码、喂给 python3 - 的 heredoc 正文里没有任何能写文件的东西，也照旧算点到后面执行的脚本（python_filter_code 交空）：红的格（推的）
#       「经 task-server：管道读 JSON 的 python3 -c 代码里写着 'task'，不当写了后面执行的 task-server/task：主 agent 放行」「经 task-server：循环里管道读 JSON 的 python3 -c 代码里写着 'task'，不当写了 task-server/task：主 agent 放行」
#   PROJECT_CODE_GUARD_BREAK=path-scripts-unread  ：实测红的格（照失败行原文）
#       「一百三十七：PATH 指到放脚本的目录、命令词不带斜杠的 .sh 照读正文」「一百三十七：PATH 指到放脚本的目录、命令词不带斜杠的 .sh 照读正文：读进去几份脚本」
#   PROJECT_CODE_GUARD_BREAK=glob-scripts-unread  ：实测红的格（照失败行原文）
#       「一百三十七：bash 起通配写出的脚本路径照 bash 展开读正文」「一百三十七：bash 起通配写出的脚本路径照 bash 展开读正文：读进去几份脚本」
#   PROJECT_CODE_GUARD_BREAK=copy-redirections-unseen  ：实测红的格（照失败行原文）
#       「一百三十七：cat 源 >> 拷出再起，照写出的内容判：读进去几份脚本」「一百三十七：cat 轻脚本 &> 拷出再起，照内容判、放行」「一百三十七：cat 轻脚本 &> 拷出再起，照内容判、放行：读进去几份脚本」
#   PROJECT_CODE_GUARD_BREAK=tee-copy-unseen  ：实测红的格（照失败行原文）
#       「一百三十七：cat 源 | tee 拷出再起，照写出的内容判：读进去几份脚本」「一百三十七：tee 目的 < 轻脚本 拷出再起，照内容判、放行」「一百三十七：tee 目的 < 轻脚本 拷出再起，照内容判、放行：读进去几份脚本」
#   PROJECT_CODE_GUARD_BREAK=launcher-functions-unjudged  ：实测红的格（照失败行原文）
#       「一百三十七：脚本里经自己的 run_logged 起 --all 照拒」
#   PROJECT_CODE_GUARD_BREAK=wrapper-copies-trusted  ：实测红的格（照失败行原文）
#       「一百三十七：别处树里的 run-with-memory-cap.sh 拷贝照读正文：读进去几份脚本」
#   PROJECT_CODE_GUARD_BREAK=cat-missing-source-unexplained  ：实测红的格（照失败行原文）
#       「一百三十七：cat 源里有不在的文件、&& 接起，照拒并点名不在的源」
#   PROJECT_CODE_GUARD_BREAK=cat-append-unreadable-destination-raises  ：cat 追加到盘上读不了的文件时，不认那个目的这一支不生效（pieces 是 None 还接着 append，判的时候抛 AttributeError）；实测红的格（照失败行原文，实际一栏是抛的错）
#       「一百三十七：cat 追加到不是文本的文件之后接 herd7 x.litmus，照拒」「一百三十七：同一条主 agent 起，照拒」
#   PROJECT_CODE_GUARD_BREAK=relative-script-undeterminable-directory-allowed  script_path_of() 不拒当前目录算不出（cd 失败、cd 到的目录是算不出的变量 / 命令替换、~- 的 OLDPWD）而起的相对路径脚本（算不出就当脚本不在、放行）
#       「d137 H5：cd 到不在的目录、cd 失败，相对脚本认不出在哪，照拒」「记几条检出」「d137 H5：cd 目标写成命令替换、算不出，相对脚本认不出，照拒」「d137 H6：bash ~-/脚本 的 OLDPWD 算不出，认不出起哪一份，照拒」「应当是 2，实际 0」
#   PROJECT_CODE_GUARD_BREAK=bash-env-forwarding-untracked  scan_uses() 不拿调用处的 environment 重判「体内起 shell 的转发函数」的函数体（函数体就地按词法环境判、看不到调用处前缀设的 BASH_ENV / ENV）
#       「d137 A2：BASH_ENV=脚本 包进起 bash 的转发函数、跟进函数体读正文照拒」「读进去几份脚本」「d137 A2：env BASH_ENV=脚本 起转发函数照拒」「读进去几份脚本」「d137 A2：function 关键字定义的函数同样跟进、照拒」「读进去几份脚本」「d137 A2：BASH_ENV 指到轻脚本，放行：读进去几份脚本」
#   PROJECT_CODE_GUARD_BREAK=compound-stdin-untracked  scan_uses() 不认 exec < 文件 / { … } < 文件 / ( … ) < 文件 / while … done < 文件 下发给里层 shell 的标准输入文件（里层读标准输入的 shell 看不到复合命令 / exec 上的输入重定向）
#       「d137 H4：exec < 文件; bash 读那份文件判、照拒」「d137 H4：exec 0< 文件; bash -s 照拒」「d137 H4：{ bash; } < 文件 照拒」「d137 H4：( bash ) < 文件 照拒」「d137 H4：while … done < 文件 照拒」「读进去几份脚本」「d137 H4：exec < 变量; bash 来源算不出，照拒」「d137 H4：exec < 轻文件; bash 放行：读进去几份脚本」「d137 H4：{ bash; } < 轻脚本 放行：读进去几份脚本」
#       一百六十六①（推的，没跑）：「一百六十六①：if … fi < 文件 里的裸 bash 读那份文件判，照拒」「一百六十六①：if … fi < 文件 里的裸 bash 读那份文件判，照拒：读进去几份脚本」「一百六十六①：case … esac < 文件 里的裸 bash 读那份文件判，照拒」「一百六十六①：case … esac < 文件 里的裸 bash 读那份文件判，照拒：读进去几份脚本」「一百六十六①：函数体里的裸 bash、调用时 < 文件，读那份文件判，照拒」「一百六十六①：函数体里的裸 bash、调用时 < 文件，读那份文件判，照拒：读进去几份脚本」「一百六十六①：{ bash; } <&3 从别的描述符转交，认不出照拒」「一百六十六①：printf 代码 | ( bash ) 管道接进子 shell 里的裸 bash，认不出照拒」「一百六十六①：{ bash; } <<< 代码 here-string 转交，认不出照拒」「一百六十六①：if … fi < 轻脚本 里的裸 bash，读正文放行：读进去几份脚本」
#   PROJECT_CODE_GUARD_BREAK=copied-shell-binary-untracked  scan_uses() 不认同一条命令里 cp / ln / install 拷来、链来的 shell 二进制（copied_shell_binary 交 None：./x 命令词不在 SHELL_NAMES、它的参数没当 shell 判）
#       「d137 H8：cp /bin/bash ./x; ./x -c 代码 照 bash 判、照拒」「d137 H8：ln -sf /bin/bash ./x; ./x -c 代码 照拒」「d137 H8：install /bin/bash ./x; ./x 脚本 读正文照拒」「读进去几份脚本」「d137 H8：cp /bin/bash ./x; ./x 轻脚本 放行：读进去几份脚本」
#   PROJECT_CODE_GUARD_BREAK=cat-missing-source-allowed  record_files_written() 与 piped_text() 下多源 cat / >> / tee 的源盘上不在、读不到内容时，照把在的源接起来当写出的内容判，不照认不出拒
#       「d137 H7：多源 cat 有盘上不在的源（在的源无害），照拒」「d137 H7：源是 echo 写出的（认不出写法、盘上不在），照拒」「d137 H7：cat 源 >> 追加、源盘上不在，照拒」「d137 H7：cat 源 | tee 拷出、源盘上不在，照拒」「读进去几份脚本」「d137 H7：多源 cat 有盘上不在的源（在的源起全量）、写出再起，照拒：读进去几份脚本」
#   PROJECT_CODE_GUARD_BREAK=unjudged-shell-input-allowed  ：实测红的格（照失败行原文）
#       「一百三十七：grep 的输出经管道喂给 bash：认不出照拒」
#   PROJECT_CODE_GUARD_BREAK=bash-layer-unrecorded  ：实测红的格（照失败行原文）
#       「一百三十七：钩子 bash 这一层起不来（假 python3 退 127）：放行（退出码 0）、记一条检出」
#   PROJECT_CODE_GUARD_BREAK=python-missing-unrecorded  PATH 上没有 python3 时不在 preflight 之前记检出，照旧由 preflight 退 78、什么都不记
#       「d137 H13：PATH 上没有 python3：放行（退出码 0）、记一条检出」「d137 H13：PATH 上没有 python3、工作根取不到：放行（退出码 0）、不往家目录下写、stderr 说检出没记成」
#   LIB_SHELL_WORDS_BREAK=login-profiles-unread  共用模块 startup_file_words() 不认登录 shell（-l、--login、exec -l、exec -a -名字）：/etc/profile、HOME 下的 profile 一类都不读
#       「d137 已知没认：登录 bash -l 读 HOME 下的 .bash_profile，里面起全量，照拒」「d137 已知没认：登录 bash -l 读 HOME 下的 .bash_profile，里面起全量，照拒：读进去几份脚本」「d137 已知没认：登录 bash --login 同样读 .bash_profile，照拒」「d137 已知没认：登录 bash --login 同样读 .bash_profile，照拒：读进去几份脚本」「d137 已知没认：登录 sh -l 读 HOME 下的 .profile，照拒」「d137 已知没认：登录 sh -l 读 HOME 下的 .profile，照拒：读进去几份脚本」「d137 已知没认：登录 exec -l bash 照登录 shell 读 .bash_profile，照拒」「d137 已知没认：登录 exec -l bash 照登录 shell 读 .bash_profile，照拒：读进去几份脚本」「d137 已知没认：登录 exec -a -bash bash 照登录 shell 读 .bash_profile，照拒」「d137 已知没认：登录 exec -a -bash bash 照登录 shell 读 .bash_profile，照拒：读进去几份脚本」「d137 已知没认：登录 bash -l 读的 profile 只有轻活，放行：读进去几份脚本」「d137 已知没认：系统级 bash -l 读 /etc/profile（样本目录），里面起全量，照拒」「d137 已知没认：系统级 bash -l 读 /etc/profile（样本目录），里面起全量，照拒：读进去几份脚本」「d137-r2 G：登录 bash -l 读 .profile 再 source 带非交互守卫的 .bashrc，守卫之后不读，放行：读进去几份脚本」「d137-r2 G：守卫不在顶层（套在 if 里）不截，照拒」
#   LIB_SHELL_WORDS_BREAK=system-startup-files-unread  共用模块不交 SYSTEM_STARTUP_DIRECTORY 下的系统级启动文件（/etc/profile、/etc/bash.bashrc）
#       「d137 已知没认：系统级 bash -l 读 /etc/profile（样本目录），里面起全量，照拒」「d137 已知没认：系统级 bash -l 读 /etc/profile（样本目录），里面起全量，照拒：读进去几份脚本」「d137 已知没认：系统级 bash -i 读 /etc/bash.bashrc（样本目录），照拒」「d137 已知没认：系统级 bash -i 读 /etc/bash.bashrc（样本目录），照拒：读进去几份脚本」「d137 已知没认：系统级 bash --rcfile 轻文件 -i 照读 /etc/bash.bashrc，照拒」「d137 已知没认：系统级 bash --rcfile 轻文件 -i 照读 /etc/bash.bashrc，照拒：读进去几份脚本」
#   LIB_SHELL_WORDS_BREAK=embedded-launches-unseen  共用模块 embedded_launches() 一条都不认：别的程序里起的命令不判
#       「d137 H9：python -c 里 os.system 起全量，照拒」「d137 H9：python 读 heredoc 里 subprocess.run 起全量，照拒」「d137 H9：python -c 里 os.system 的命令是变量，认不出照拒」「d137 H9：perl -e 里 system 起全量，照拒」「perl 单独写的 -i 不吃下一个词：perl -i -e 里 system 起全量，照拒」「perl 单独写的 -l 不吃下一个词：perl -l -e 里 system 起全量，照拒」「perl 单独写的 -0 不吃下一个词：perl -0 -e 里 system 起全量，照拒」「d137 H9：ruby -e 里 system 起全量，照拒」「d137 H9：node -e 里 child_process 起全量，照拒」「d137 H9：tclsh 读 here-string 里 exec 起全量，照拒」「d137 H9：gawk 的 system() 起全量，照拒」「d137 H9：awk 的 print 管道到全量，照拒」「d137 H9：awk -f 程序文件里 system() 起全量，读正文照拒」「d137 H9：awk -f 程序文件里 system() 起全量，读正文照拒：读进去几份脚本」「d137 H9：awk -f 程序文件不起命令，放行：读进去几份脚本」「d137 H9：sed 的 e 命令起全量，照拒」「d137 H9：sed 的 s///e 执行模式空间，认不出照拒」「d137 H9：git -c 别名 ! 起全量，照拒」「d137 H9：git -c core.pager 起全量，照拒」「d137 H9：GIT_EXTERNAL_DIFF 起全量，照拒」「d137 H9：make 读 Makefile 配方起全量，照拒」「d137 H9：make 读 Makefile 配方起全量，照拒：读进去几份脚本」「d137 H9：make 读 Makefile 配方只有轻活，放行：读进去几份脚本」「d137 H9：make -f /dev/stdin 读 here-string 的配方起全量，照拒」「d137 H9：tar --to-command 起全量，照拒」「d137 H9：ssh -o ProxyCommand 起全量，照拒」「d137 H9：脚本正文里 perl -e 起全量，读正文照拒」「d137-r2 A：sed 算术地址后接 e 起全量，照拒」「d137-r2 A：sed 命令替换落在命令位上，照拒」「d137-r2 B：sed 文本之后另起一行 e 起全量，照拒」「d137-r2 C：python argv 起 cargo test 而参数是变量，照拒」「d137-r2 C：python argv 头一项是变量，照拒」「d137-r2 D：python 先赋变量的字面列表起全量，照拒」「d137-r2 D：python 列表赋过之后被改，认不出照拒」「d137-r2 E：awk 里 cmd 赋全量再 | getline，照拒」「d137-r2 E：awk 里 cmd 是拼接的，认不出照拒」「d137 闸误拒 E5：W 赋过字面目录，awk -f $W/程序文件，读正文放行：读进去几份脚本」「d137 闸误拒 E5：W 没赋过，awk -f $W/程序文件认不出，照拒」
#       一百六十六①（推的，没跑）：「一百六十六①：make -C 的目录写成没赋过的变量，认不出照拒」「一百六十六①：make -C 的目录写成前面赋过字面值的变量，代入之后读 Makefile 照拒」「一百六十六①：make -C 的目录写成前面赋过字面值的变量，代入之后读 Makefile 照拒：读进去几份脚本」「一百六十六①：make -C 的目录写成前面赋过字面值的变量、Makefile 只有轻活，放行：读进去几份脚本」「一百六十六①：git -c 的配置整个写成没赋过的变量，认不出照拒」「一百六十六①：git -c 别名的值写成没赋过的变量，认不出照拒」「一百六十六①：git -c 的配置写成前面赋过字面值的变量（! 别名起全量），代入照拒」
#   PROJECT_CODE_GUARD_BREAK=program-files-unread  scan_uses() 不读别的程序从文件读的代码（awk -f、sed -f、Makefile：只判命令行上字面写着的）
#       「d137 H9：awk -f 程序文件里 system() 起全量，读正文照拒」「d137 H9：awk -f 程序文件里 system() 起全量，读正文照拒：读进去几份脚本」「d137 H9：awk -f 程序文件不起命令，放行：读进去几份脚本」「d137 H9：make 读 Makefile 配方起全量，照拒」「d137 H9：make 读 Makefile 配方起全量，照拒：读进去几份脚本」「d137 H9：make 读 Makefile 配方只有轻活，放行：读进去几份脚本」「d137 闸误拒 E5：W 赋过字面目录，awk -f $W/程序文件，读正文放行：读进去几份脚本」「d137 闸误拒 E5：W 没赋过，awk -f $W/程序文件认不出，照拒」
#       一百六十六①（推的，没跑）：「一百六十六①：make -C 的目录写成前面赋过字面值的变量，代入之后读 Makefile 照拒」「一百六十六①：make -C 的目录写成前面赋过字面值的变量，代入之后读 Makefile 照拒：读进去几份脚本」「一百六十六①：make -C 的目录写成前面赋过字面值的变量、Makefile 只有轻活，放行：读进去几份脚本」
#   PROJECT_CODE_GUARD_BREAK=embedded-unjudged-allowed  别的程序里起命令而写法认不出的，命令文本这一层只记不拒
#       「d137 H9：python -c 里 os.system 的命令是变量，认不出照拒」「d137 H9：python -c 里 os.system 的命令是变量，认不出照拒：记几条检出」「d137 H9：sed 的 s///e 执行模式空间，认不出照拒」「d137 H9：sed 的 s///e 执行模式空间，认不出照拒：记几条检出」「d137-r2 A：sed 命令替换落在命令位上，照拒」「d137-r2 D：python 列表赋过之后被改，认不出照拒」「d137-r2 E：awk 里 cmd 是拼接的，认不出照拒」「d137 闸误拒 E5：W 没赋过，awk -f $W/程序文件认不出，照拒」
#       一百六十六①（推的，没跑）：「一百六十六①：make -C 的目录写成没赋过的变量，认不出照拒」「一百六十六①：make -C 的目录写成没赋过的变量，认不出照拒：记几条检出」「一百六十六①：git -c 的配置整个写成没赋过的变量，认不出照拒」「一百六十六①：git -c 的配置整个写成没赋过的变量，认不出照拒：记几条检出」「一百六十六①：git -c 别名的值写成没赋过的变量，认不出照拒」「一百六十六①：git -c 别名的值写成没赋过的变量，认不出照拒：记几条检出」
#   LIB_SHELL_WORDS_BREAK=sed-expansions-unparsed  sed_script_commands() 不另算 sed 脚本词里没展开的 bash 展开，原样切、切不下去照「解析不下去」拒：实测红的格（照失败行原文）
#       「d137-r2 A：sed 地址里的算术展开照 sed 判，放行」「d137-r2 A：sed 地址里的 $[ ] 算术照 sed 判，放行」「d137-r2 A：sed 行号地址是命令替换，照 sed 判，放行」
#   LIB_SHELL_WORDS_BREAK=sed-text-first-line-only  sed_script_parse() 的 a / i / c 文本只读到头一个换行（`a\` 换行之后的文本行被当命令读）：实测红的格（照失败行原文）
#       「d137-r2 B：sed 的 c\ 多行文本，放行」「d137-r2 B：sed 的 a\ 续行文本，放行」（推的）
#   LIB_SHELL_WORDS_BREAK=python-argument-variables-unresolved  scan_python() 起命令的参数只认整个字面的（argv 里有一项是变量、赋过字面值的变量、字面串的 .split() 都整条认不出）：实测红的格（照失败行原文）
#       「d137-r2 C：python argv 头一项字面、参数是变量，照命令词判，放行」「d137-r2 D：python 字面列表先赋变量再起，放行」「d137-r2 D：python 字面串 .split() 作命令，放行」
#   LIB_SHELL_WORDS_BREAK=awk-command-variables-unresolved  scan_awk() 不认赋过字面串的变量（`cmd = "date"; cmd | getline`、命令行 cmd=sort 再 print | cmd 照写成变量拒）：实测红的格（照失败行原文）
#       「d137-r2 E：awk 里 cmd 赋字面再 | getline，放行」「d137-r2 E：awk 命令行 cmd=sort 再 print |，放行」
#   LIB_SHELL_WORDS_BREAK=cd-known-targets-unexpanded  changed_directory() 不认 $PWD、$OLDPWD、`cd -`、起头的 `$(git rev-parse --show-toplevel)` 与 `$(pwd)`（cd 之后目录算不出）：实测红的格（照失败行原文）
#       「d137-r2 F：cd "$PWD" 之后 bash 轻脚本，放行」「d137-r2 F：cd "$PWD" 之后 bash 轻脚本，放行：读进去几份脚本」「d137-r2 F：cd "$PWD" 之后 bash 重脚本，照拒：读进去几份脚本」「d137-r2 F：cd 到 git 顶层之后 bash 轻脚本，放行」「d137-r2 F：cd 到 git 顶层之后 bash 轻脚本，放行：读进去几份脚本」「d137-r2 F：cd 到 git 顶层之后 bash 重脚本，照拒：读进去几份脚本」「d137-r2 F：cd "$OLDPWD" 回来之后 bash 轻脚本，放行」「d137-r2 F：cd "$OLDPWD" 回来之后 bash 轻脚本，放行：读进去几份脚本」「d137-r2 F：cd "$OLDPWD" 回来之后 bash 重脚本，照拒：读进去几份脚本」
#   LIB_SHELL_WORDS_BREAK=cd-failure-guards-unseen  CommandAtPosition.cd_failure_skips 一律是假（`cd 不在的目录 && 相对脚本`、`|| exit` 守着的也照「cd 会失败」拒）：实测红的格（照失败行原文）
#       「d137-r2 H5：cd 盘上不在的目录 && 相对脚本，cd 失败就不跑，放行」「d137-r2 H5：cd 盘上不在的目录 && 相对脚本，cd 失败就不跑，放行：记几条检出」「d137-r2 H5：cd 盘上不在的目录 || exit 之后的相对脚本，放行」「d137-r2 H5：cd 盘上不在的目录 || exit 之后的相对脚本，放行：记几条检出」「d137-r2 H5：mkdir 再 cd 进去跑同一条里写出的重脚本，照拒：读进去几份脚本」
#   LIB_SHELL_WORDS_BREAK=non-interactive-return-unseen  non_interactive_return_offset() 交 None：非交互 shell source 的文件里 `case $- in *i*) ;; *) return;; esac` 这类守卫不截、整份照读：实测红的格（照失败行原文）
#       「d137-r2 G：登录 bash -l 读 .profile 再 source 带非交互守卫的 .bashrc，守卫之后不读，放行」
#   LIB_SHELL_WORDS_BREAK=substitution-case-and-comment-unseen  ：实测红的格（照失败行原文）
#       「一百三十七：命令替换里不加括号的 case 模式不提前收尾，里面的 --all 照拒」「一百三十七：命令替换里注释的 ) 不收尾，后面的 --all 照拒」
#   LIB_SHELL_WORDS_BREAK=coproc-as-command  ：实测红的格（照失败行原文）
#       「一百三十七：coproc 后面那条照判」
#   LIB_SHELL_WORDS_BREAK=quoted-braces-as-unknown  ：实测红的格（照失败行原文）
#       「一百三十七：函数体外引号里的花括号不当认不准的调用，只定义的函数体不判：放行」
#       「lib_project_code_runs 自检：花括号：命令词位置单引号 {herd7,x.litmus} 照字面放行」「lib_project_code_runs 自检：花括号：命令词位置双引号 {herd7,x.litmus} 照字面放行」「lib_project_code_runs.py 的自检」
#   LIB_SHELL_WORDS_BREAK=env-ignore-environment-kept  ：红的格（推的，没跑：命令前的 BASH_ENV 留到 env -i 后面，bash 读进 startup-heavy.sh、判出全量）
#       「一百三十七：env -i 清掉 BASH_ENV 再起 bash：放行」「一百三十七：env -i 清掉 BASH_ENV 再起 bash：放行：读进去几份脚本」
#   LIB_SHELL_WORDS_BREAK=shell-standard-input-file-unread  ：实测红的格（照失败行原文）
#       「实现员 cat 文件经管道喂给 bash：放行并记检出」「一百三十七：bash -s < 文件 读那份文件判：记几条检出」「一百三十七：bash -s < 文件 读那份文件判：读进去几份脚本」「一百三十七：cat 文件 | bash 读那份文件判：记几条检出」「一百三十七：cat 文件 | bash 读那份文件判：读进去几份脚本」「一百三十七：sh -s < 轻脚本放行」「一百三十七：sh -s < 轻脚本放行：记几条检出」「一百三十七：sh -s < 轻脚本放行：读进去几份脚本」
#       「d137 H4：bash /dev/stdin < 文件 读那份文件判，照拒：记几条检出」「d137 H4：bash /dev/stdin < 文件 读那份文件判，照拒：读进去几份脚本」「d137 H4：bash /dev/stdin < 轻脚本 放行」「d137 H4：bash /dev/stdin < 轻脚本 放行：记几条检出」「d137 H4：bash /dev/stdin < 轻脚本 放行：读进去几份脚本」
#   LIB_SHELL_WORDS_BREAK=function-launchers-unseen  同 launcher-functions-unjudged 那一格：实测红的格（照失败行原文）
#       「一百三十七：脚本里经自己的 run_logged 起 --all 照拒」
#   LIB_SHELL_WORDS_BREAK=startup-files-unread  ：startup_file_words() 一个文件都不交，shell 启动时先读的文件（BASH_ENV / ENV、--rcfile、.bashrc、zsh 的）都不当被起的脚本；实测红的格（照失败行原文）
#       「一百三十七：BASH_ENV 写在命令前、指到字面路径的脚本，正文起全量，照拒」「一百三十七：BASH_ENV 写在命令前、指到字面路径的脚本，正文起全量，照拒：读进去几份脚本」「一百三十七：BASH_ENV 写相对路径，按这条命令的当前目录解，照拒」「一百三十七：BASH_ENV 写相对路径，按这条命令的当前目录解，照拒：读进去几份脚本」「一百三十七：env BASH_ENV=… 起 bash，照拒」「一百三十七：env BASH_ENV=… 起 bash，照拒：读进去几份脚本」「一百三十七：前面 export BASH_ENV=… 再起 bash，照拒」「一百三十七：前面 export BASH_ENV=… 再起 bash，照拒：读进去几份脚本」「一百三十七：BASH_ENV 指到的脚本正文起全量，bash 起的又是一份轻脚本，两份都读，照拒」「一百三十七：BASH_ENV 指到的脚本正文起全量，bash 起的又是一份轻脚本，两份都读，照拒：读进去几份脚本」「一百三十七：BASH_ENV 指到的脚本 source 自己目录下的库，库也读，照拒」「一百三十七：BASH_ENV 指到的脚本 source 自己目录下的库，库也读，照拒：读进去几份脚本」「一百三十七：BASH_ENV 指到的脚本正文起全量，实现员照拒」「一百三十七：BASH_ENV 指到的脚本正文起全量，实现员照拒：读进去几份脚本」「一百三十七：BASH_ENV 指到的脚本正文只有轻活，放行：读进去几份脚本」「一百三十七：BASH_ENV 写成前面赋过字面值的变量，展开之后读正文，照拒」「一百三十七：BASH_ENV 写成前面赋过字面值的变量，展开之后读正文，照拒：读进去几份脚本」「一百三十七：ENV 指到的脚本正文起全量，sh 起的，照拒」「一百三十七：ENV 指到的脚本正文起全量，sh 起的，照拒：读进去几份脚本」「一百三十七：ENV 指到的脚本正文起全量，bash --posix 起的，照拒」「一百三十七：ENV 指到的脚本正文起全量，bash --posix 起的，照拒：读进去几份脚本」「一百三十七：ENV 指到的脚本正文起全量，bash -o posix 起的，照拒」「一百三十七：ENV 指到的脚本正文起全量，bash -o posix 起的，照拒：读进去几份脚本」「一百三十七：BASH_ENV 写成命令替换拼出的路径，认不出，实现员照拒」「一百三十七：BASH_ENV 写成命令替换拼出的路径，认不出，主 agent 照拒」「一百三十七：BASH_ENV 写成没赋过值的变量，认不出，实现员照拒」「一百三十七：env 的 BASH_ENV 写成命令替换，认不出，实现员照拒」「一百三十七：ENV 写成命令替换，sh 起的，认不出，实现员照拒」「一百三十七：直接执行 #!/bin/bash 的脚本，BASH_ENV 指的文件照读，照拒」「一百三十七：直接执行 #!/bin/bash 的脚本，BASH_ENV 指的文件照读，照拒：读进去几份脚本」「一百三十七：直接执行 #!/usr/bin/env bash 的脚本，BASH_ENV 指的文件照读，照拒」「一百三十七：直接执行 #!/usr/bin/env bash 的脚本，BASH_ENV 指的文件照读，照拒：读进去几份脚本」「一百三十七：直接执行没有 #! 的 .sh，从 bash 里起是 bash 另起自己，BASH_ENV 指的文件照读，照拒」「一百三十七：直接执行没有 #! 的 .sh，从 bash 里起是 bash 另起自己，BASH_ENV 指的文件照读，照拒：读进去几份脚本」「一百三十七：env 写的 BASH_ENV，直接执行 #!/bin/bash 的脚本，照拒」「一百三十七：env 写的 BASH_ENV，直接执行 #!/bin/bash 的脚本，照拒：读进去几份脚本」「一百三十七：直接执行 #!/bin/sh 的脚本，ENV 指的文件照读，照拒」「一百三十七：直接执行 #!/bin/sh 的脚本，ENV 指的文件照读，照拒：读进去几份脚本」「一百三十七：直接执行 #!/bin/zsh 的脚本，ZDOTDIR 下的 .zshenv 照读，照拒」「一百三十七：直接执行 #!/bin/zsh 的脚本，ZDOTDIR 下的 .zshenv 照读，照拒：读进去几份脚本」「一百三十七：直接执行 #!/bin/bash 的脚本，BASH_ENV 指的文件只有轻活，放行：读进去几份脚本」「一百三十七：直接执行 #!/bin/bash 的脚本，BASH_ENV 写成命令替换拼出的路径，认不出，实现员照拒」「一百三十七：declare -x BASH_ENV=…，再起 bash，照拒」「一百三十七：declare -x BASH_ENV=…，再起 bash，照拒：读进去几份脚本」「一百三十七：typeset -x BASH_ENV=…，再起 bash，照拒」「一百三十七：typeset -x BASH_ENV=…，再起 bash，照拒：读进去几份脚本」「一百三十七：先 BASH_ENV=… 赋值、后 export BASH_ENV，再起 bash，照拒」「一百三十七：先 BASH_ENV=… 赋值、后 export BASH_ENV，再起 bash，照拒：读进去几份脚本」「一百三十七：先 export BASH_ENV、后赋值，再起 bash，照拒」「一百三十七：先 export BASH_ENV、后赋值，再起 bash，照拒：读进去几份脚本」「一百三十七：export 过之后换成另一份脚本，起 bash 读的是后一份，照拒」「一百三十七：export 过之后换成另一份脚本，起 bash 读的是后一份，照拒：读进去几份脚本」「一百三十七：set -a 之后赋值的 BASH_ENV 进环境，照拒」「一百三十七：set -a 之后赋值的 BASH_ENV 进环境，照拒：读进去几份脚本」「一百三十七：local -x BASH_ENV=… 的函数里起 bash，照拒」「一百三十七：local -x BASH_ENV=… 的函数里起 bash，照拒：读进去几份脚本」「一百三十七：declare -x BASH_ENV 写成命令替换拼出的路径，认不出，实现员照拒」「一百三十七：先赋值后 export 的 BASH_ENV 是命令替换拼出的路径，认不出，实现员照拒」「一百三十七：bash -i 读 HOME 下的 .bashrc，里面起全量，照拒」「一百三十七：bash -i 读 HOME 下的 .bashrc，里面起全量，照拒：读进去几份脚本」「一百三十七：bash -ic 合写的 -i 同样读 HOME 下的 .bashrc，照拒」「一百三十七：bash -ic 合写的 -i 同样读 HOME 下的 .bashrc，照拒：读进去几份脚本」「一百三十七：bash --rcfile 文件 -i，文件里起全量，照拒」「一百三十七：bash --rcfile 文件 -i，文件里起全量，照拒：读进去几份脚本」「一百三十七：bash --init-file 文件 -i，文件里起全量，照拒」「一百三十七：bash --init-file 文件 -i，文件里起全量，照拒：读进去几份脚本」「一百三十七：bash -i 读 HOME 下只有轻活的 .bashrc：放行：读进去几份脚本」「一百三十七：bash -i 不读 BASH_ENV，.bashrc 只有轻活：放行：读进去几份脚本」「一百三十七：bash -i 不读 ENV：放行：读进去几份脚本」「一百三十七：sh -i 读 ENV 指的文件，照拒」「一百三十七：sh -i 读 ENV 指的文件，照拒：读进去几份脚本」「一百三十七：sh -i 读 ENV 指的文件，只有轻活：放行：读进去几份脚本」「一百三十七：bash --posix -i 读 ENV 指的文件，照拒」「一百三十七：bash --posix -i 读 ENV 指的文件，照拒：读进去几份脚本」「一百三十七：bash --rcfile 写成命令替换拼出的路径，认不出，实现员照拒」「一百三十七：bash -i 的 HOME 写成命令替换，.bashrc 路径认不出，实现员照拒」「一百三十七：zsh 读 ZDOTDIR 下的 .zshenv，里面起全量，照拒」「一百三十七：zsh 读 ZDOTDIR 下的 .zshenv，里面起全量，照拒：读进去几份脚本」「一百三十七：zsh 读 ZDOTDIR 下的 .zshrc，里面起全量，照拒」「一百三十七：zsh 读 ZDOTDIR 下的 .zshrc，里面起全量，照拒：读进去几份脚本」「一百三十七：zsh 没设 ZDOTDIR 取 HOME 下的 .zshenv，照拒」「一百三十七：zsh 没设 ZDOTDIR 取 HOME 下的 .zshenv，照拒：读进去几份脚本」「一百三十七：前面 export 的 ZDOTDIR，zsh 照读，照拒」「一百三十七：前面 export 的 ZDOTDIR，zsh 照读，照拒：读进去几份脚本」「一百三十七：zsh 的启动文件只有轻活：放行：读进去几份脚本」「一百三十七：zsh 的 ZDOTDIR 写成命令替换，认不出，实现员照拒」
#       「d137 H6：BASH_ENV=~+/脚本 起 bash，照 $PWD 解、读正文照拒」「d137 H6：BASH_ENV=~+/脚本 起 bash，照 $PWD 解、读正文照拒：读进去几份脚本」「d137 A2：BASH_ENV=脚本 包进起 bash 的转发函数、跟进函数体读正文照拒」「d137 A2：BASH_ENV=脚本 包进起 bash 的转发函数、跟进函数体读正文照拒：读进去几份脚本」「d137 A2：env BASH_ENV=脚本 起转发函数照拒」「d137 A2：env BASH_ENV=脚本 起转发函数照拒：读进去几份脚本」「d137 A2：function 关键字定义的函数同样跟进、照拒」「d137 A2：function 关键字定义的函数同样跟进、照拒：读进去几份脚本」「d137 A2：BASH_ENV 指到轻脚本，放行：读进去几份脚本」「d137 已知没认：登录 bash -l 读 HOME 下的 .bash_profile，里面起全量，照拒」「d137 已知没认：登录 bash -l 读 HOME 下的 .bash_profile，里面起全量，照拒：读进去几份脚本」「d137 已知没认：登录 bash --login 同样读 .bash_profile，照拒」「d137 已知没认：登录 bash --login 同样读 .bash_profile，照拒：读进去几份脚本」「d137 已知没认：登录 sh -l 读 HOME 下的 .profile，照拒」「d137 已知没认：登录 sh -l 读 HOME 下的 .profile，照拒：读进去几份脚本」「d137 已知没认：登录 exec -l bash 照登录 shell 读 .bash_profile，照拒」「d137 已知没认：登录 exec -l bash 照登录 shell 读 .bash_profile，照拒：读进去几份脚本」「d137 已知没认：登录 exec -a -bash bash 照登录 shell 读 .bash_profile，照拒」「d137 已知没认：登录 exec -a -bash bash 照登录 shell 读 .bash_profile，照拒：读进去几份脚本」「d137 已知没认：登录 bash -l 读的 profile 只有轻活，放行：读进去几份脚本」「d137 已知没认：系统级 bash -l 读 /etc/profile（样本目录），里面起全量，照拒」「d137 已知没认：系统级 bash -l 读 /etc/profile（样本目录），里面起全量，照拒：读进去几份脚本」「d137 已知没认：系统级 bash -i 读 /etc/bash.bashrc（样本目录），照拒」「d137 已知没认：系统级 bash -i 读 /etc/bash.bashrc（样本目录），照拒：读进去几份脚本」「d137 已知没认：系统级 bash --rcfile 轻文件 -i 照读 /etc/bash.bashrc，照拒」「d137 已知没认：系统级 bash --rcfile 轻文件 -i 照读 /etc/bash.bashrc，照拒：读进去几份脚本」
#       「d137-r2 G：登录 bash -l 读 .profile 再 source 带非交互守卫的 .bashrc，守卫之后不读，放行：读进去几份脚本」「d137-r2 G：交互的 bash -i 读带守卫的 .bashrc 不截，守卫之后的全量照拒」「d137-r2 G：交互的 bash -i 读带守卫的 .bashrc 不截，守卫之后的全量照拒：读进去几份脚本」「d137-r2 G：守卫不在顶层（套在 if 里）不截，照拒」「d137-r2 G：守卫不在顶层（套在 if 里）不截，照拒：读进去几份脚本」
#   LIB_SHELL_WORDS_BREAK=declared-exports-unseen  ：环境里的变量只认 `export 名字=值`（declare -x、typeset -x、local -x、先赋值后 export、先 export 后赋值、export -n、declare +x、set -a 都不认）；实测红的格（照失败行原文）
#       「一百三十七：declare -x BASH_ENV=…，再起 bash，照拒」「一百三十七：declare -x BASH_ENV=…，再起 bash，照拒：读进去几份脚本」「一百三十七：typeset -x BASH_ENV=…，再起 bash，照拒」「一百三十七：typeset -x BASH_ENV=…，再起 bash，照拒：读进去几份脚本」「一百三十七：先 BASH_ENV=… 赋值、后 export BASH_ENV，再起 bash，照拒」「一百三十七：先 BASH_ENV=… 赋值、后 export BASH_ENV，再起 bash，照拒：读进去几份脚本」「一百三十七：先 export BASH_ENV、后赋值，再起 bash，照拒」「一百三十七：先 export BASH_ENV、后赋值，再起 bash，照拒：读进去几份脚本」「一百三十七：export 过之后换成另一份脚本，起 bash 读的是后一份，照拒」「一百三十七：set -a 之后赋值的 BASH_ENV 进环境，照拒」「一百三十七：set -a 之后赋值的 BASH_ENV 进环境，照拒：读进去几份脚本」「一百三十七：local -x BASH_ENV=… 的函数里起 bash，照拒」「一百三十七：local -x BASH_ENV=… 的函数里起 bash，照拒：读进去几份脚本」「一百三十七：export -n 撤掉 export 之后起 bash：放行」「一百三十七：export -n 撤掉 export 之后起 bash：放行：读进去几份脚本」「一百三十七：export -n 带赋值，不进环境：放行」「一百三十七：export -n 带赋值，不进环境：放行：读进去几份脚本」「一百三十七：declare +x 撤掉 export 之后起 bash：放行」「一百三十七：declare +x 撤掉 export 之后起 bash：放行：读进去几份脚本」「一百三十七：declare -x BASH_ENV 写成命令替换拼出的路径，认不出，实现员照拒」「一百三十七：先赋值后 export 的 BASH_ENV 是命令替换拼出的路径，认不出，实现员照拒」
#   LIB_SHELL_WORDS_BREAK=interactive-startup-files-unread  ：带 -i 不另看（交互的 bash 照旧读 BASH_ENV、不读 .bashrc 与 --rcfile / --init-file）；实测红的格（照失败行原文）
#       「一百三十七：bash -i 读 HOME 下的 .bashrc，里面起全量，照拒」「一百三十七：bash -i 读 HOME 下的 .bashrc，里面起全量，照拒：读进去几份脚本」「一百三十七：bash -ic 合写的 -i 同样读 HOME 下的 .bashrc，照拒」「一百三十七：bash -ic 合写的 -i 同样读 HOME 下的 .bashrc，照拒：读进去几份脚本」「一百三十七：bash --rcfile 文件 -i，文件里起全量，照拒」「一百三十七：bash --rcfile 文件 -i，文件里起全量，照拒：读进去几份脚本」「一百三十七：bash --init-file 文件 -i，文件里起全量，照拒」「一百三十七：bash --init-file 文件 -i，文件里起全量，照拒：读进去几份脚本」「一百三十七：bash -i 读 HOME 下只有轻活的 .bashrc：放行：读进去几份脚本」「一百三十七：bash -i 不读 BASH_ENV，.bashrc 只有轻活：放行」「一百三十七：bash -i 不读 ENV：放行：读进去几份脚本」「一百三十七：bash --rcfile 写成命令替换拼出的路径，认不出，实现员照拒」「一百三十七：bash -i 的 HOME 写成命令替换，.bashrc 路径认不出，实现员照拒」
#       「d137 已知没认：系统级 bash -i 读 /etc/bash.bashrc（样本目录），照拒」「d137 已知没认：系统级 bash -i 读 /etc/bash.bashrc（样本目录），照拒：读进去几份脚本」「d137 已知没认：系统级 bash --rcfile 轻文件 -i 照读 /etc/bash.bashrc，照拒」「d137 已知没认：系统级 bash --rcfile 轻文件 -i 照读 /etc/bash.bashrc，照拒：读进去几份脚本」「d137-r2 G：交互的 bash -i 读带守卫的 .bashrc 不截，守卫之后的全量照拒」「d137-r2 G：交互的 bash -i 读带守卫的 .bashrc 不截，守卫之后的全量照拒：读进去几份脚本」
#   LIB_SHELL_WORDS_BREAK=zsh-startup-files-unread  ：zsh 的 .zshenv、.zshrc 不读；实测红的格（照失败行原文）
#       「一百三十七：直接执行 #!/bin/zsh 的脚本，ZDOTDIR 下的 .zshenv 照读，照拒」「一百三十七：直接执行 #!/bin/zsh 的脚本，ZDOTDIR 下的 .zshenv 照读，照拒：读进去几份脚本」「一百三十七：zsh 读 ZDOTDIR 下的 .zshenv，里面起全量，照拒」「一百三十七：zsh 读 ZDOTDIR 下的 .zshenv，里面起全量，照拒：读进去几份脚本」「一百三十七：zsh 读 ZDOTDIR 下的 .zshrc，里面起全量，照拒」「一百三十七：zsh 读 ZDOTDIR 下的 .zshrc，里面起全量，照拒：读进去几份脚本」「一百三十七：zsh 没设 ZDOTDIR 取 HOME 下的 .zshenv，照拒」「一百三十七：zsh 没设 ZDOTDIR 取 HOME 下的 .zshenv，照拒：读进去几份脚本」「一百三十七：前面 export 的 ZDOTDIR，zsh 照读，照拒」「一百三十七：前面 export 的 ZDOTDIR，zsh 照读，照拒：读进去几份脚本」「一百三十七：zsh 的启动文件只有轻活：放行：读进去几份脚本」「一百三十七：zsh 的 ZDOTDIR 写成命令替换，认不出，实现员照拒」
#   LIB_SHELL_WORDS_BREAK=executed-script-startup-unread  ：直接执行的脚本起来时它的 shell 先读的启动文件不读，#! 指到认不出的解释器的也不判；实测红的格（照失败行原文）
#       「一百三十七：直接执行 #!/bin/bash 的脚本，BASH_ENV 指的文件照读，照拒」「一百三十七：直接执行 #!/bin/bash 的脚本，BASH_ENV 指的文件照读，照拒：读进去几份脚本」「一百三十七：直接执行 #!/usr/bin/env bash 的脚本，BASH_ENV 指的文件照读，照拒」「一百三十七：直接执行 #!/usr/bin/env bash 的脚本，BASH_ENV 指的文件照读，照拒：读进去几份脚本」「一百三十七：直接执行没有 #! 的 .sh，从 bash 里起是 bash 另起自己，BASH_ENV 指的文件照读，照拒」「一百三十七：直接执行没有 #! 的 .sh，从 bash 里起是 bash 另起自己，BASH_ENV 指的文件照读，照拒：读进去几份脚本」「一百三十七：env 写的 BASH_ENV，直接执行 #!/bin/bash 的脚本，照拒」「一百三十七：env 写的 BASH_ENV，直接执行 #!/bin/bash 的脚本，照拒：读进去几份脚本」「一百三十七：直接执行 #!/bin/sh 的脚本，ENV 指的文件照读，照拒」「一百三十七：直接执行 #!/bin/sh 的脚本，ENV 指的文件照读，照拒：读进去几份脚本」「一百三十七：直接执行 #!/bin/zsh 的脚本，ZDOTDIR 下的 .zshenv 照读，照拒」「一百三十七：直接执行 #!/bin/zsh 的脚本，ZDOTDIR 下的 .zshenv 照读，照拒：读进去几份脚本」「一百三十七：直接执行 #!/bin/bash 的脚本，BASH_ENV 指的文件只有轻活，放行：读进去几份脚本」「一百三十七：直接执行 #! 指到认不出的解释器、环境里设了 BASH_ENV，实现员照拒」「一百三十七：直接执行 #!/bin/bash 的脚本，BASH_ENV 写成命令替换拼出的路径，认不出，实现员照拒」
#   LIB_PROJECT_CODE_RUNS_BREAK=unrecognized-launches-allowed  ：实测红的格（照失败行原文）
#       「实现员经内存包装起循环里 ls 出来的 验证包测试二进制：命令词认不出照拒」「一百三十七：bash <(…) 照拒」「一百三十七：eval 变量里拼的命令照拒」「一百三十七：source <(…) 照拒」「一百三十七：命令词是 $(which cargo) 照拒」「一百三十七：for 循环变量当脚本路径照拒」「一百三十七：xargs bash 起读进来的脚本照拒」「一百三十七：BASH_ENV 写成命令替换拼出的路径，认不出，实现员照拒」「一百三十七：BASH_ENV 写成命令替换拼出的路径，认不出，主 agent 照拒」「一百三十七：BASH_ENV 写成没赋过值的变量，认不出，实现员照拒」「一百三十七：env 的 BASH_ENV 写成命令替换，认不出，实现员照拒」「一百三十七：ENV 写成命令替换，sh 起的，认不出，实现员照拒」「一百三十七：直接执行 #!/bin/bash 的脚本，BASH_ENV 写成命令替换拼出的路径，认不出，实现员照拒」「一百三十七：declare -x BASH_ENV 写成命令替换拼出的路径，认不出，实现员照拒」「一百三十七：先赋值后 export 的 BASH_ENV 是命令替换拼出的路径，认不出，实现员照拒」「一百三十七：bash --rcfile 写成命令替换拼出的路径，认不出，实现员照拒」「一百三十七：bash -i 的 HOME 写成命令替换，.bashrc 路径认不出，实现员照拒」「一百三十七：zsh 的 ZDOTDIR 写成命令替换，认不出，实现员照拒」「lib_project_code_runs.py 的自检」「d137 闸误拒 E3：$t 没赋过，cat > $t 写出再 bash $t，照拒」
#       一百六十六①：「一百六十六①：子 shell 里赋的值不带出来，外面的命令词认不出照拒」「一百六十六①：管道段里赋的值不带出来，照拒」「一百六十六①：放后台的赋值不带出来，照拒」「一百六十六①：|| 之后的条件赋值不一定生效，照拒」「一百六十六①：if 里赋的值不一定生效，照拒」
#       「lib_project_code_runs 自检：认不出：命令词是变量」「lib_project_code_runs 自检：认不出：命令词是命令替换」「lib_project_code_runs 自检：认不出：命令词展开出来夹着空白」「lib_project_code_runs 自检：认不出：bash 起的脚本是进程替换」「lib_project_code_runs 自检：认不出：source 进程替换」「lib_project_code_runs 自检：认不出：cargo 子命令是命令替换」「lib_project_code_runs 自检：认不出：cargo test 的参数是变量」「lib_project_code_runs 自检：认不出：cargo test 的参数是位置参数」「lib_project_code_runs 自检：脚本正文：cargo test 的参数是解不出的变量，照认不出」「一百三十七：bash -c "$(cat 文件)" 照拒」「一百三十七：set -- 之后 "$@" 照拒」「d137-r2 C：python argv 起 cargo test 而参数是变量，照拒」「d137-r2 C：python argv 头一项是变量，照拒」
#   LIB_PROJECT_CODE_RUNS_BREAK=cargo-arguments-unexpanded  ：实测红的格（照失败行原文）
#       「一百三十七：cargo 参数写成前面赋过值的变量，展开之后照判（--lib 放行）」「lib_project_code_runs.py 的自检」
#       「lib_project_code_runs 自检：cargo 参数展开：F=--workspace 之后 cargo test $F」
#   LIB_PROJECT_CODE_RUNS_BREAK=extra-launchers-unseen  ：实测红的格（照失败行原文）
#       「一百三十七：xargs bash 起读进来的脚本照拒」「lib_project_code_runs.py 的自检」「d137 闸误拒 E2：ssh localhost 包着 bash 轻脚本，放行：读进去几份脚本」「d137 闸误拒 E2：numactl -N0 包着 bash 轻脚本，放行：读进去几份脚本」「d137 闸误拒 E2：unshare --user 包着 bash 轻脚本，放行：读进去几份脚本」
#       「lib_project_code_runs 自检：启动器：xargs 接在命令后面的读进来的词」「lib_project_code_runs 自检：启动器：xargs -I{} 换掉替换串」「lib_project_code_runs 自检：启动器：xargs 不给命令是 echo」「lib_project_code_runs 自检：启动器：find -exec 的命令、{} 换成找到的路径」「lib_project_code_runs 自检：启动器：find 两个 -exec 连成 sh -c」「lib_project_code_runs 自检：启动器：ssh 把 host 之后的词交给远端 shell」「lib_project_code_runs 自检：启动器：doas -u 用户」「lib_project_code_runs 自检：启动器：numactl -N0 贴着写」「lib_project_code_runs 自检：启动器：numactl --membind 值另起一个词」「lib_project_code_runs 自检：启动器：script -qec 的代码」「lib_project_code_runs 自检：启动器：unshare --user」「lib_project_code_runs 自检：启动器：setpriv --reuid=」「lib_project_code_runs 自检：启动器：nsenter -t 进程号」「lib_project_code_runs 自检：启动器：su -c 的代码」「lib_project_code_runs 自检：启动器：runuser -u 用户 --」「lib_project_code_runs 自检：启动器：sg 组 -c」「lib_project_code_runs 自检：启动器：watch -n 1」「lib_project_code_runs 自检：启动器：gdb --args」「lib_project_code_runs 自检：启动器：scl enable 集合 --」「lib_project_code_runs 自检：启动器：parallel ::: 之前是命令」「lib_project_code_runs 自检：启动器：bwrap --bind 带两个值」「lib_project_code_runs 自检：启动器：ltrace -o 文件」「lib_project_code_runs 自检：启动器：valgrind --tool=」「lib_project_code_runs 自检：启动器：faketime 时间戳」「lib_project_code_runs 自检：启动器：unbuffer -p」「lib_project_code_runs 自检：启动器：cpulimit -l 50 --」「lib_project_code_runs 自检：启动器：dbus-run-session --」「lib_project_code_runs 自检：启动器：firejail --quiet」「lib_project_code_runs 自检：启动器：proot -r 根」「lib_project_code_runs 自检：启动器：chroot 新根」
#       「lib_project_code_runs 自检：启动器：cgexec -g」
#   LIB_PROJECT_CODE_RUNS_BREAK=sourced-libraries-unread  ：实测红的格（照失败行原文）
#       「实现员起的脚本里命令替换、反引号、算术后面接路径：片段不当路径：记几条检出」「一百三十七：脚本 source 自己目录下的库，读正文时库也读」「一百三十七：脚本 source 自己目录下的库，读正文时库也读：读进去几份脚本」「lib_project_code_runs.py 的自检」
#       「lib_project_code_runs 自检：source 的库：$(dirname "${BASH_SOURCE[0]}") 换成脚本所在目录」「一百三十七：BASH_ENV 指到的脚本 source 自己目录下的库，库也读，照拒」「一百三十七：BASH_ENV 指到的脚本 source 自己目录下的库，库也读，照拒：读进去几份脚本」
#   LIB_PROJECT_CODE_RUNS_BREAK=script-body-launches-unjudged  ：实测红的格（照失败行原文）
#       「一百三十七：脚本正文里经数组变量起拼出路径的钩子自证（件 1 那三趟的形态），照拒」「一百三十七：同一份脚本主 agent 照拒」「lib_project_code_runs.py 的自检」
#       「lib_project_code_runs 自检：脚本正文：数组变量的定义里写着 --selftest，照认不出」「lib_project_code_runs 自检：脚本正文："$@" 转发、起它的参数里写着 --selftest，照认不出」「lib_project_code_runs 自检：脚本正文：cargo test 的参数是解不出的变量，照认不出」
#   LIB_PROJECT_CODE_RUNS_BREAK=dispatch-launchers-unseen  （multi-host-run.sh --peer … -- 不剥）：实测红的格（照失败行原文）
#       「lib_project_code_runs.py 的自检」「编译目录：子 agent 经 multi-host-run.sh --peer 在主工作区起 cargo test、没给编译目录：拒（退出码 2）」
#       「lib_project_code_runs 自检：启动器：multi-host-run.sh --peer 带 --memory-cap、--label 各一个值」
#   PROJECT_CODE_GUARD_BREAK=launcher-recorded-as-writer  （剥掉之后里面那条接着判的启动器整条记成写者）：实测红的格（照失败行原文）
#       「d137 闸误拒 E2：systemd-run --user --wait 包着 bash 轻脚本，放行」「d137 闸误拒 E2：systemd-run --user --wait 包着 bash 轻脚本，放行：读进去几份脚本」「d137 闸误拒 E2：chrt -i 0 包着 bash 轻脚本，放行」「d137 闸误拒 E2：ssh localhost 包着 bash 轻脚本，放行」「d137 闸误拒 E2：/usr/bin/time 包着 bash 轻脚本，放行」「d137 闸误拒 E2：strace -o /dev/null 包着 bash 轻脚本，放行」「d137 闸误拒 E2：prlimit --as 包着 bash 轻脚本，放行」「d137 闸误拒 E2：numactl -N0 包着 bash 轻脚本，放行」「d137 闸误拒 E2：unshare --user 包着 bash 轻脚本，放行」「d137 闸误拒 E2：systemd-run --user --wait 包着 bash 重脚本，读正文照拒：读进去几份脚本」
#   PROJECT_CODE_GUARD_BREAK=heredoc-targets-unexpanded  （heredoc 写出的目标不代入前面赋过字面值的变量）：实测红的格（照失败行原文）
#       「d137 闸误拒 E3：t 赋过字面路径，cat > $t 读 heredoc 写出再 bash $t，放行」
#   PROJECT_CODE_GUARD_BREAK=program-file-variables-unexpanded  （awk -f、sed -f、Makefile 的路径不代入前面赋过字面值的变量）：实测红的格（照失败行原文）
#       「d137 闸误拒 E5：W 赋过字面目录，awk -f $W/程序文件，读正文放行」「d137 闸误拒 E5：W 赋过字面目录，awk -f $W/程序文件，读正文放行：读进去几份脚本」
#   LIB_PROJECT_CODE_RUNS_BREAK=claude-executable-path-unrecognized  （命令词 "$CLAUDE_CODE_EXECPATH" 照变量拼出的认不出）：实测红的格（照失败行原文）
#       「lib_project_code_runs.py 的自检」
#       「lib_project_code_runs 自检：认得出：命令词是 $CLAUDE_CODE_EXECPATH（检出钩子 ⑬ 判起 claude）」「lib_project_code_runs 自检：认得出：命令词是 ${CLAUDE_CODE_EXECPATH}（检出钩子 ⑬ 判起 claude）」「d137 闸口径：主 agent 经 "$CLAUDE_CODE_EXECPATH" 起 claude，交检出钩子判，放行」
#   LIB_SHELL_WORDS_BREAK=aliases-same-line-expanded  （同一行里先定义再用的别名照展开）：实测红的格（照失败行原文）
#       「d137 闸误拒 E4：同一行先 alias 再用，这一行不展开别名、放行」
#   LIB_SHELL_WORDS_BREAK=alias-start-options-unseen  （bash -O expand_aliases、sh / dash 这类起来时就开着别名展开的不认，set -o posix 不算开）：红的格（推的，没跑）
#       「一百六十六①：bash -O expand_aliases -c 里换行之后用别名展开成全量，照拒」「一百六十六①：sh -c 里换行之后用别名展开成全量，照拒」
#   LIB_SHELL_WORDS_BREAK=alias-nested-texts-unseen  （eval、trap、命令替换、source /dev/stdin 的代码不带外面的别名，eval 里定义的别名不收）：红的格（推的，没跑）
#       「一百六十六①：shopt 开着时 eval 里用前面定义的别名，照拒」「一百六十六①：同一行 shopt、alias 之后 eval 里用别名（eval 跑的时候才切），照拒」「一百六十六①：eval 里定义的别名，下一行照展开，照拒」
#   LIB_SHELL_WORDS_BREAK=compound-stdin-closers-unseen  （fi / esac 收尾、函数调用上的 < 文件，<& / here-string / heredoc 转交，管道接进复合命令，bash -c 与 eval 代码里的都不认）：红的格（推的，没跑）
#       「一百六十六①：if … fi < 文件 里的裸 bash 读那份文件判，照拒」「一百六十六①：if … fi < 文件 里的裸 bash 读那份文件判，照拒：读进去几份脚本」「一百六十六①：case … esac < 文件 里的裸 bash 读那份文件判，照拒」「一百六十六①：case … esac < 文件 里的裸 bash 读那份文件判，照拒：读进去几份脚本」「一百六十六①：函数体里的裸 bash、调用时 < 文件，读那份文件判，照拒」「一百六十六①：函数体里的裸 bash、调用时 < 文件，读那份文件判，照拒：读进去几份脚本」「一百六十六①：{ bash; } <&3 从别的描述符转交，认不出照拒」「一百六十六①：printf 代码 | ( bash ) 管道接进子 shell 里的裸 bash，认不出照拒」「一百六十六①：{ bash; } <<< 代码 here-string 转交，认不出照拒」「一百六十六①：if … fi < 轻脚本 里的裸 bash，读正文放行：读进去几份脚本」
#   LIB_SHELL_WORDS_BREAK=embedded-option-variables-unexpanded  （make -C 的目录、git -c 的配置写成变量的不代入、不交说明）：红的格（推的，没跑）
#       「一百六十六①：make -C 的目录写成没赋过的变量，认不出照拒」「一百六十六①：make -C 的目录写成前面赋过字面值的变量，代入之后读 Makefile 照拒」「一百六十六①：make -C 的目录写成前面赋过字面值的变量，代入之后读 Makefile 照拒：读进去几份脚本」「一百六十六①：make -C 的目录写成前面赋过字面值的变量、Makefile 只有轻活，放行：读进去几份脚本」「一百六十六①：git -c 的配置整个写成没赋过的变量，认不出照拒」「一百六十六①：git -c 别名的值写成没赋过的变量，认不出照拒」「一百六十六①：git -c 的配置写成前面赋过字面值的变量（! 别名起全量），代入照拒」
#   LIB_SHELL_WORDS_BREAK=variable-scopes-unseen  （变量记值不分作用域）：红的格（推的，没跑）
#       「一百六十六①：子 shell 里赋的值不带出来，外面的命令词认不出照拒」「一百六十六①：管道段里赋的值不带出来，照拒」「一百六十六①：放后台的赋值不带出来，照拒」「一百六十六①：|| 之后的条件赋值不一定生效，照拒」「一百六十六①：if 里赋的值不一定生效，照拒」
#   LIB_SHELL_WORDS_BREAK=aliases-unexpanded  shell_aliases() 交空：shopt -s expand_aliases 之后定义的别名不展开成它的值（命令词是别名时照普通命令判）：红的格（推的，没跑）
#       「d137 H3：别名展开成 herd7 x.litmus 照拒」「d137 H3：别名 cargo 自展开（换行、先定义后 shopt）照拒」「d137 H3：别名展开成起脚本（同一条里 alias 先提到脚本、认不出写法）照拒」「一百六十六①：bash -O expand_aliases -c 里换行之后用别名展开成全量，照拒」「一百六十六①：sh -c 里换行之后用别名展开成全量，照拒」「一百六十六①：shopt 开着时 eval 里用前面定义的别名，照拒」「一百六十六①：同一行 shopt、alias 之后 eval 里用别名（eval 跑的时候才切），照拒」「一百六十六①：eval 里定义的别名，下一行照展开，照拒」
#   LIB_SHELL_WORDS_BREAK=trap-action-unread  commands_at_command_position() 不把 trap 'action' 的字符串当 shell 代码再切（trap 的参数不判）：红的格（推的，没跑）
#       「d137 H2：trap 'herd7 x.litmus' EXIT 照拒」「d137 H2：trap 'herd7 x.litmus' ERR 照拒」「d137 H2：trap 'herd7 x.litmus' 0 照拒」「d137 H2：trap 'bash startup-heavy.sh' DEBUG 读正文照拒」「d137 H2：trap 'bash startup-heavy.sh' DEBUG 读正文照拒：读进去几份脚本」「d137 H2：trap -- 'herd7 x.litmus' EXIT（-- 之后是 action）照拒」
#   LIB_SHELL_WORDS_BREAK=standard-input-device-script-unread  起 shell 的脚本路径是标准输入设备（/dev/stdin、/dev/fd/0、/proc/self/fd/0）时不当标准输入、照旧当一份脚本文件读：红的格（推的，没跑）
#       「d137 H4：bash /dev/stdin < 文件 读那份文件判，照拒」「d137 H4：bash /dev/stdin < 文件 读那份文件判，照拒：读进去几份脚本」「d137 H4：echo 代码 | bash /dev/stdin 照拒」「d137 H4：bash /dev/fd/0 <<< 代码 照拒」「d137 H4：source /dev/stdin <<< 代码 照拒」「d137 H4：bash /dev/stdin < 轻脚本 放行：读进去几份脚本」
#   LIB_SHELL_WORDS_BREAK=tilde-plus-minus-unexpanded  expand_directory_tilde() 不展开 `~+`（$PWD）与 `~-`（$OLDPWD）（当普通路径、落不到实处）：红的格（推的，没跑）
#       「d137 H6：bash ~+/脚本 照 $PWD 解、读正文照拒」「d137 H6：bash ~+/脚本 照 $PWD 解、读正文照拒：记几条检出」「d137 H6：bash ~+/脚本 照 $PWD 解、读正文照拒：读进去几份脚本」「d137 H6：BASH_ENV=~+/脚本 起 bash，照 $PWD 解、读正文照拒」「d137 H6：BASH_ENV=~+/脚本 起 bash，照 $PWD 解、读正文照拒：读进去几份脚本」「d137 H6：bash ~-/脚本 的 OLDPWD 算不出，认不出起哪一份，照拒」「d137 H6：bash ~-/脚本 的 OLDPWD 算不出，认不出起哪一份，照拒：记几条检出」
#   LIB_SHELL_WORDS_BREAK=renamed-shells-unseen  is_shell_name() 只认 bash / sh / dash / zsh / ksh，rbash、ash 不当 shell，busybox 的 sh / ash applet 不剥：红的格（推的，没跑）
#       「d137 H8：rbash -c 代码 照 shell 判、照拒」「d137 H8：rbash 起脚本 读正文照拒」「d137 H8：rbash 起脚本 读正文照拒：读进去几份脚本」「d137 H8：busybox sh -c 代码 照拒」「d137 H8：busybox ash -c 代码 照拒」
#   LIB_SHELL_WORDS_BREAK=python-options-by-dash  python_invocation() 退回「以 - 开头的词都当选项跳过、第一个不以 - 开头的词当脚本」：-c 的代码当脚本名、没有被扫的代码：红的格（推的，没跑）
#       「一百三十七：python 写出再起照拒」「一百三十七：python 写出再起照拒：记几条检出」「d137-r2 C：python argv 头一项字面、参数是变量，照命令词判，放行」「d137-r2 D：python 字面串 .split() 作命令，放行」「经 task-server：管道读 JSON 的 python3 -c 代码里写着 'task'，不当写了后面执行的 task-server/task：主 agent 放行」「经 task-server：循环里管道读 JSON 的 python3 -c 代码里写着 'task'，不当写了 task-server/task：主 agent 放行」「经 task-server：python3 -c 往 task-server/task 写了再执行它，照拒」
#   LIB_SHELL_WORDS_BREAK=directory-variables-unexpanded  expand_known_variables() 遇到 `$` 一律交 None：cd / pushd / env -C 的目录与别处写在变量里的词（命令词、cargo 参数、BASH_ENV、重定向目标、make -C、git -c、awk -f 的路径）照旧认不出：红的格（推的，没跑）
#       「d137-r2 F：cd "$PWD" 之后 bash 轻脚本，放行」「d137-r2 F：cd "$PWD" 之后 bash 轻脚本，放行：读进去几份脚本」「d137-r2 F：cd "$PWD" 之后 bash 重脚本，照拒：读进去几份脚本」「d137-r2 F：cd "$OLDPWD" 回来之后 bash 轻脚本，放行」「d137-r2 F：cd "$OLDPWD" 回来之后 bash 轻脚本，放行：读进去几份脚本」「d137-r2 F：cd "$OLDPWD" 回来之后 bash 重脚本，照拒：读进去几份脚本」「一百三十七：BASH_ENV 写成前面赋过字面值的变量，展开之后读正文，照拒：读进去几份脚本」「一百三十七：cargo 参数写成前面赋过值的变量，展开之后照判（--lib 放行）」「d137 闸误拒 E3：t 赋过字面路径，cat > $t 读 heredoc 写出再 bash $t，放行」「d137 闸误拒 E3：t 赋过字面路径，cat > $t 读 heredoc 写出再 bash $t，放行：读进去几份脚本」「d137 闸误拒 E5：W 赋过字面目录，awk -f $W/程序文件，读正文放行」「d137 闸误拒 E5：W 赋过字面目录，awk -f $W/程序文件，读正文放行：读进去几份脚本」「一百六十六①：make -C 的目录写成前面赋过字面值的变量，代入之后读 Makefile 照拒：读进去几份脚本」「一百六十六①：make -C 的目录写成前面赋过字面值的变量、Makefile 只有轻活，放行」「一百六十六①：make -C 的目录写成前面赋过字面值的变量、Makefile 只有轻活，放行：读进去几份脚本」「一百六十六①：git -c 的配置写成前面赋过字面值的变量（user.name），放行」「一百六十六①：子 shell 里赋值、在子 shell 里用，照里面的值判，放行」「一百六十六①：一串 && 里赋值、同一串里接着用，放行」「一百六十六①：子 shell 里只读不改，外面的值照用，放行」「lib_project_code_runs.py 的自检」「lib_project_code_runs 自检：命令词变量：S=herd7 之后 $S 起 herd7」「lib_project_code_runs 自检：cargo 参数展开：F=--workspace 之后 cargo test $F」「cd 进写在变量里的主工作区再起：拒：命令」「lib_workspace_cargo_lock.py 的自检」「编译目录：先赋变量再拼仓外 CARGO_TARGET_DIR：放行：命令」「一百三十七：PATH 指到放脚本的目录、命令词不带斜杠的 .sh 照读正文：读进去几份脚本」
#   LIB_SHELL_WORDS_BREAK=perl-bare-options-take-next-word  embedded_launches() 把 perl 里值只能贴着写的选项（-i、-l、-0 这类）单独写时也当带值选项、吃掉下一个词：红的格（推的，没跑）
#       「perl 单独写的 -i 不吃下一个词：perl -i -e 里 system 起全量，照拒」「perl 单独写的 -l 不吃下一个词：perl -l -e 里 system 起全量，照拒」「perl 单独写的 -0 不吃下一个词：perl -0 -e 里 system 起全量，照拒」
#   LIB_PROJECT_CODE_RUNS_BREAK=active-sessions-unlisted  lib_project_code_runs.py 的 MAIN_AGENT_TOOLS_RUN_DIRECTLY 里 scripts/agents/active-sessions.py 那一项不认：钩子这一侧红的格（推的，没跑）
#       「lib_project_code_runs.py 的自检」
#       「lib_project_code_runs 自检：名单：主 agent 查活动会话的 active-sessions.py」「经 task-server：主 agent 直接起查活动会话的 active-sessions.py：放行」
#   LIB_PROJECT_CODE_RUNS_BREAK=build-subcommands-unjudged  lib_project_code_runs.py 的 starts_a_build 不认会编的 cargo：实测红的格（照失败行原文）
#       「lib_project_code_runs.py 的自检」
#       「lib_project_code_runs 自检：起编译：cargo build」「lib_project_code_runs 自检：起编译：cargo b 简写」「lib_project_code_runs 自检：起编译：cargo check」「lib_project_code_runs 自检：起编译：cargo clippy」「lib_project_code_runs 自检：起编译：cargo test --no-run 只编不跑也算」「lib_project_code_runs 自检：起编译：cargo doc」「lib_project_code_runs 自检：起编译：cargo run」「lib_project_code_runs 自检：起编译：全局选项在子命令前」「lib_project_code_runs 自检：起编译：--config 定的别名当会编」
#   LIB_PROJECT_CODE_RUNS_BREAK=build-scripts-unjudged  starts_a_build 不认自己在里面起 cargo 的那几份脚本：实测红的格（照失败行原文）
#       「lib_project_code_runs.py 的自检」
#       「lib_project_code_runs 自检：起编译：python3 起 compile-then-swap.py」「lib_project_code_runs 自检：起编译：直接执行别处树里的 compile-then-swap.py」
#   PROJECT_CODE_GUARD_BREAK=task-server-direct-allowed  task_server_decision 不判 ① 的直接跑（编译、编出来的二进制、本仓的代码、剥掉的包装、那张表认出的每一样），只照「谁能跑哪一类」判：实测红的格（照失败行原文）
#       「经 task-server：照抄：实现员直接跑 harness 测试，照抄的那条经 task-server 提交」「经 task-server：照抄：经内存包装直接起的去掉包装」「经 task-server：照抄：带 && 的整条包进 bash -c」「stdin:实现员直接跑测试目标拒绝（退出码 2）」「实现员经内存包装起 cargo test -p 当前目录探针自己的包名：照拒」「实现员经内存包装与 bash -c 起 cargo run --package= 点名探针：照拒」「实现员经内存包装 env -C 换进探针再 cargo test -p 探针自己的包名：照拒」「实现员 cd ${D} 工作区根、经 timeout、nice、capped.sh、内存包装四层全量：照拒」「主 agent D=… 之后 cd $D 全量：照拒」「实现员交回前自查命名：code-source-discipline.sh 点名那四格也是直接起本仓的门禁阶段，照拒（写进报告交主 agent 跑）」「实现员交回前自查命名：经范围声明与 nice 起、等号写法点名一格，照拒」「实现员交回前自查命名：点名里夹着别的格：照拒」「实现员交回前自查命名：不带 --check 整道：照拒」「实现员在仓根裸跑 cargo test」「主 agent 裸 cargo run E152」「主 agent {cargo,} 空的一份起全量照拒」「主 agent cargo {,test} 空的一份起全量照拒」「书记员跑 doc-decisions（直接起本仓的门禁阶段）」「通用 agent 跑 code-tooling.sh 的 agent-write-scope 格（直接起本仓的门禁阶段）」「主 agent 跑 code-tooling.sh（直接起本仓的门禁阶段）」「主 agent 跑 gate-lint.sh」「实现员跑 gate-structure-check.py」「主 agent 跑 hooks-registered.sh」「经 task-server：子 agent 直接起查活动会话的 active-sessions.py：照拒」「实现员复现输入那一条（反引号里的 walk.rs）：末尾的 doc-lint.sh 是直接起的本仓脚本，照拒」「一百三十七#8：工具实现员经 multi-host-run.sh --peer 起门禁格，照拒」「stdin:实现员跑 --all 拒绝（退出码 2）」「stdin:拒绝时 stderr 写了出路：规矩一句、子 agent 写进报告交回」「stdin:主 agent 直接跑被拒时最前面是能照抄的经 task-server 提交的那条（照抄的那条再判放行）」「stdin:主 agent 被拒时不贴「谁能跑哪一类」那一段，规矩指到提交写法」「stdin:被拒的检出记录写了结局「被拒」」「stdin:主 agent 直接跑整道门禁被拒时照抄的那条经 task-server 提交、四段范围声明写在 submit 前面（为什么是占位），照它再判放行」「stdin:子 agent 不经 task-server 的拒绝 stderr 写了照抄的提交、写进报告「要主 agent 跑的命令」一节交回，不教后台等」「stdin:heredoc 写出的含 --all 的脚本拒绝（退出码 2）」
#       「stdin:heredoc 写出的脚本的拒绝 stderr 写出它的路径与行号」「stdin:admission.py 判的那一刻抛异常时，点名标了 #[ignore] 的登记目标、不带 --ignored 的 cargo test 照拒（退出码 2）」
#       「经 task-server：照抄：打出的每条照抄命令照 unnamed_run_problem 判都点了名」（一条照抄都没打出，判到 0 条）
#   PROJECT_CODE_GUARD_BREAK=subagent-task-client-allowed  ② 不判：子 agent 起 task-server/task 的 submit、wait、cancel、clean 照放（子 agent 自己交；
#       「经 task-server：子 agent 交任务的拒绝说明写明写进报告「要主 agent 跑的命令」一节交回」
#   PROJECT_CODE_GUARD_BREAK=task-server-bypass-allowed  ③ 不判（scan_uses 不记、命令文本这一层不看）：实测红的格（照失败行原文；直接起服务本体与每次尝试的那一层另由 ① 照拒，不红）
#       「经 task-server：绕开：读签名密钥照拒，说明点名绕开写法」
#   LIB_PROJECT_CODE_RUNS_BREAK=task-server-builds-unjudged  不认直接删服务的编译目录：钩子这一侧实测红的格（照失败行原文）
#       「lib_project_code_runs.py 的自检」
#       「lib_project_code_runs 自检：绕开：python 代码 shutil.rmtree 服务的编译目录」「lib_project_code_runs 自检：绕开：rm -rf 服务的编译目录」「lib_project_code_runs 自检：绕开：rm 写 ~ 起头的编译目录」「lib_project_code_runs 自检：绕开：rm 状态目录下的 builds」「lib_project_code_runs 自检：绕开：find -delete 编译目录」「lib_project_code_runs 自检：绕开：find -exec rm 编译目录」「lib_project_code_runs 自检：绕开：rm -rf 编译目录的上级」
#   PROJECT_CODE_GUARD_BREAK=task-server-copy-dropped  拒绝说明不给照抄的那一条（task_server_copy_lines 交空）：实测红的格（照失败行原文）
#       「经 task-server：照抄：实现员直接跑 harness 测试，照抄的那条经 task-server 提交」「经 task-server：照抄：经内存包装直接起的去掉包装」「经 task-server：照抄：带 && 的整条包进 bash -c」「stdin:主 agent 直接跑被拒时最前面是能照抄的经 task-server 提交的那条（照抄的那条再判放行）」「stdin:主 agent 直接跑整道门禁被拒时照抄的那条经 task-server 提交、四段范围声明写在 submit 前面（为什么是占位），照它再判放行」「stdin:子 agent 不经 task-server 的拒绝 stderr 写了照抄的提交、写进报告「要主 agent 跑的命令」一节交回，不教后台等」
#       「经 task-server：照抄：打出的每条照抄命令照 unnamed_run_problem 判都点了名」（一条照抄都没打出，判到 0 条）
#   PROJECT_CODE_GUARD_BREAK=temporary-service-bare-allowed  ④ 不判（scan_uses 不记要临时服务的自证写成裸 --selftest）：实测红的格（照失败行原文）
#       「经 task-server：提交带 --manual、--full-gate、--whole-package-confirmed 时 -- 后面那条照过闸」「经 task-server：要临时服务的自证裸写被拒时，说明点名登记表、给整条包装写法」「经 task-server：子 agent 直接裸写要临时服务的自证被拒时，说明写进报告「要主 agent 跑的命令」一节、给整条包装写法」「stdin:主 agent 经 task-server 提交要临时服务的自证裸写 --selftest 拒绝（退出码 2），stderr 给包装写法的整条」
#   PROJECT_CODE_GUARD_BREAK=temporary-service-wrapper-not-marked  剥开临时服务的包装重判里面那条时不补标记（包在包装里的要临时服务的自证被当成裸写拦下）：红的格（推的）
#       「经 task-server：主 agent 提交包着临时服务（给了 --server-binary）的要临时服务的自证：放行」「经 task-server：主 agent 提交包着临时服务（给了 --server-binary）的样本档：放行」
#   PROJECT_CODE_GUARD_BREAK=temporary-service-submit-inner-unread  submit 里那一条的要临时服务的自证裸写、没套包装的样本档、没带全 lint 表的 clippy 都不收：红的格（推的）
#       「经 task-server：提交带 --manual、--full-gate、--whole-package-confirmed 时 -- 后面那条照过闸」「经 task-server：要临时服务的自证裸写被拒时，说明点名登记表、给整条包装写法」「stdin:主 agent 经 task-server 提交要临时服务的自证裸写 --selftest 拒绝（退出码 2），stderr 给包装写法的整条」「经 task-server：提交没带全 check.sh lint 表的 clippy 被拒，说明点名缺的 lint、给整段现读的写法」「经 task-server：check.sh 的 lint 表读不成（另有 += 追加）时 clippy 被拒，说明点名读不成的行、出路交上游改」「经 task-server：clippy 带全表、deny 之后又 -A 表里的 lint 被拒，说明点名被放行的那一条」「经 task-server：clippy 带全表、deny 之后又 -A clippy::pedantic（一整组）被拒」「经 task-server：clippy 带全表又带 --cap-lints allow 被拒，说明点名 --cap-lints」「经 task-server：样本档点名要临时服务的一道、外面没套被拒时，说明点名那一道、给整条包装写法」「stdin:主 agent 经 task-server 提交样本档点名要临时服务的一道、外面没套 拒绝（退出码 2），stderr 给包装写法的整条」
#   PROJECT_CODE_GUARD_BREAK=temporary-service-remedy-dropped  ④ 的说明只写哪一份、不贴规矩与怎么办（TEMPORARY_SERVICE_POLICY 不贴）：实测红的格（照失败行原文）
#       「经 task-server：要临时服务的自证裸写被拒时，说明点名登记表、给整条包装写法」「经 task-server：子 agent 直接裸写要临时服务的自证被拒时，说明写进报告「要主 agent 跑的命令」一节、给整条包装写法」
#   PROJECT_CODE_GUARD_BREAK=clippy-lint-table-unchecked  ⑤ 不判（scan_uses 不记没带全 lint 表、放行了表里的 lint、表读不成的 clippy）：红的格（照失败行原文）
#       「经 task-server：提交没带全 check.sh lint 表的 clippy 被拒，说明点名缺的 lint、给整段现读的写法」「经 task-server：check.sh 的 lint 表读不成（另有 += 追加）时 clippy 被拒，说明点名读不成的行、出路交上游改」「经 task-server：clippy 带全表、deny 之后又 -A 表里的 lint 被拒，说明点名被放行的那一条」「经 task-server：clippy 带全表、deny 之后又 -A clippy::pedantic（一整组）被拒」「经 task-server：clippy 带全表又带 --cap-lints allow 被拒，说明点名 --cap-lints」
#   PROJECT_CODE_GUARD_BREAK=clippy-lint-table-remedy-dropped  ⑤ 的说明只点名缺哪几条、不给出路（带整段现读的写法的、表读不成时交上游改的「出路」那一行不贴）：红的格（照失败行原文）
#       「经 task-server：提交没带全 check.sh lint 表的 clippy 被拒，说明点名缺的 lint、给整段现读的写法」「经 task-server：check.sh 的 lint 表读不成（另有 += 追加）时 clippy 被拒，说明点名读不成的行、出路交上游改」
#   PROJECT_CODE_GUARD_BREAK=clippy-lint-weakening-unchecked  ⑤ 不查带全表之后又 -A、-W、--allow、--warn 放行表里的 lint 与 --cap-lints：红的格（照失败行原文）
#       「经 task-server：clippy 带全表、deny 之后又 -A 表里的 lint 被拒，说明点名被放行的那一条」「经 task-server：clippy 带全表、deny 之后又 -A clippy::pedantic（一整组）被拒」「经 task-server：clippy 带全表又带 --cap-lints allow 被拒，说明点名 --cap-lints」
#   PROJECT_CODE_GUARD_BREAK=clippy-lint-unreadable-table-passed  ⑤ 在 check.sh 的表有这一段而写法读不成时放行（clippy_lint_table_problem 一概交空）：红的格（照失败行原文）
#       「经 task-server：check.sh 的 lint 表读不成（另有 += 追加）时 clippy 被拒，说明点名读不成的行、出路交上游改」
#   PROJECT_CODE_GUARD_BREAK=temporary-service-stage-samples-allowed  ⑥ 不判（scan_uses 不记点名了要临时服务的阶段而外面没套包装的样本档）：红的格（照失败行原文）
#       「经 task-server：样本档点名要临时服务的一道、外面没套被拒时，说明点名那一道、给整条包装写法」「stdin:主 agent 经 task-server 提交样本档点名要临时服务的一道、外面没套 拒绝（退出码 2），stderr 给包装写法的整条」
#   LIB_PROJECT_CODE_RUNS_BREAK=temporary-service-bare-unjudged  lib_project_code_runs.py 的 temporary_service_bare_selftest 一律交 None：钩子这一侧实测红的格（照失败行原文）
#       「经 task-server：提交带 --manual、--full-gate、--whole-package-confirmed 时 -- 后面那条照过闸」「经 task-server：要临时服务的自证裸写被拒时，说明点名登记表、给整条包装写法」「lib_project_code_runs 自检：要临时服务：required 行的裸 --selftest 交整条包装写法」「lib_project_code_runs 自检：要临时服务：仓里 crates-mutation-rows.py 的裸 --selftest 被点名、给整条包装写法」「lib_project_code_runs.py 的自检」「经 task-server：子 agent 直接裸写要临时服务的自证被拒时，说明写进报告「要主 agent 跑的命令」一节、给整条包装写法」「stdin:主 agent 经 task-server 提交要临时服务的自证裸写 --selftest 拒绝（退出码 2），stderr 给包装写法的整条」
#   LIB_PROJECT_CODE_RUNS_BREAK=temporary-service-marker-ignored  不看环境里的临时服务标记（套了包装、自己设了标记的也拦）：钩子这一侧实测红的格（照失败行原文）
#       「经 task-server：主 agent 提交包着临时服务（给了 --server-binary）的要临时服务的自证：放行」「lib_project_code_runs.py 的自检」「经 task-server：主 agent 提交包着临时服务（给了 --server-binary）的样本档：放行」
#       「lib_project_code_runs 自检：要临时服务：命令前设了临时服务的标记，裸 --selftest 放行」「lib_project_code_runs 自检：要临时服务：包装剥开重判时补上的标记让里面那条放行」「lib_project_code_runs 自检：要临时服务：仓里 crates-mutation-rows.py 的 --selftest 在包装底下放行」「lib_project_code_runs 自检：要临时服务：样本档点名的一道要临时服务、包装剥开重判补上了标记：放行」
#   LIB_PROJECT_CODE_RUNS_BREAK=temporary-service-environment-dropped  environment_under_temporary_service 原样交回（包装里的原命令看起来没有标记）：钩子这一侧实测红的格（照失败行原文）
#       「经 task-server：主 agent 提交包着临时服务（给了 --server-binary）的要临时服务的自证：放行」「lib_project_code_runs.py 的自检」「经 task-server：主 agent 提交包着临时服务（给了 --server-binary）的样本档：放行」
#       「lib_project_code_runs 自检：要临时服务：包装剥开重判时补上的标记让里面那条放行」「lib_project_code_runs 自检：要临时服务：仓里 crates-mutation-rows.py 的 --selftest 在包装底下放行」「lib_project_code_runs 自检：要临时服务：样本档点名的一道要临时服务、包装剥开重判补上了标记：放行」
#   LIB_PROJECT_CODE_RUNS_BREAK=temporary-service-own-server-as-required  自己起服务的自证也被要求套临时服务：钩子这一侧实测红的格（照失败行原文）
#       「经 task-server：主 agent 提交自己起服务的自证（own-server）裸写：放行」「lib_project_code_runs.py 的自检」
#       「lib_project_code_runs 自检：要临时服务：own-server 行的裸 --selftest 不拒」
#   LIB_PROJECT_CODE_RUNS_BREAK=temporary-service-stage-samples-unjudged  lib_project_code_runs.py 的 temporary_service_bare_stage_samples 一律交 None：钩子这一侧红的格（照失败行原文）
#       「lib_project_code_runs.py 的自检」「lib_project_code_runs 自检：要临时服务：样本档点名要临时服务的一道、外面没套：说明点名那一道（不点不要的那一道）、给整条包装写法」「lib_project_code_runs 自检：要临时服务：仓里门禁 crates-mutation-replay.sh 的样本档裸写被点名、给整条包装写法」「经 task-server：样本档点名要临时服务的一道、外面没套被拒时，说明点名那一道、给整条包装写法」「stdin:主 agent 经 task-server 提交样本档点名要临时服务的一道、外面没套 拒绝（退出码 2），stderr 给包装写法的整条」
#   LIB_PROJECT_CODE_RUNS_BREAK=temporary-service-stage-samples-every-stage  点名的每一道样本档都当要临时服务（不读阶段源码）：钩子这一侧红的格（照失败行原文；只在 lib_project_code_runs.py 的自检里）
#       「lib_project_code_runs.py 的自检」「lib_project_code_runs 自检：要临时服务：样本档点名要临时服务的一道、外面没套：说明点名那一道（不点不要的那一道）、给整条包装写法」「lib_project_code_runs 自检：要临时服务：样本档只点名不要临时服务的一道：不拒」「lib_project_code_runs 自检：要临时服务：仓里门禁 doc-text.sh 的样本档不要临时服务：不拒」
#   LIB_PROJECT_CODE_RUNS_BREAK=temporary-service-command-bare  说明里给的整条写法是裸写的样子：钩子这一侧实测红的格（照失败行原文）
#       「经 task-server：要临时服务的自证裸写被拒时，说明点名登记表、给整条包装写法」「lib_project_code_runs 自检：要临时服务：required 行的裸 --selftest 交整条包装写法」「lib_project_code_runs 自检：要临时服务：python 脚本的整条写法起头是 python3」「lib_project_code_runs 自检：要临时服务：仓里 crates-mutation-rows.py 的裸 --selftest 被点名、给整条包装写法」「lib_project_code_runs.py 的自检」「经 task-server：子 agent 直接裸写要临时服务的自证被拒时，说明写进报告「要主 agent 跑的命令」一节、给整条包装写法」「stdin:主 agent 经 task-server 提交要临时服务的自证裸写 --selftest 拒绝（退出码 2），stderr 给包装写法的整条」「lib_project_code_runs 自检：要临时服务：样本档点名要临时服务的一道、外面没套：说明点名那一道（不点不要的那一道）、给整条包装写法」「lib_project_code_runs 自检：要临时服务：仓里门禁 crates-mutation-replay.sh 的样本档裸写被点名、给整条包装写法」「经 task-server：样本档点名要临时服务的一道、外面没套被拒时，说明点名那一道、给整条包装写法」「stdin:主 agent 经 task-server 提交样本档点名要临时服务的一道、外面没套 拒绝（退出码 2），stderr 给包装写法的整条」
#   LIB_PROJECT_CODE_RUNS_BREAK=temporary-service-command-binary-placeholder  说明里给的整条写法带 --server-binary 与尖括号占位：钩子这一侧红的格
#       「经 task-server：要临时服务的自证裸写被拒时，说明点名登记表、给整条包装写法」「lib_project_code_runs 自检：要临时服务：required 行的裸 --selftest 交整条包装写法」「lib_project_code_runs 自检：要临时服务：python 脚本的整条写法起头是 python3」「lib_project_code_runs 自检：要临时服务：整条写法不带 --server-binary 与尖括号占位」「lib_project_code_runs 自检：要临时服务：裸 --selftest 的拒绝说明里没有尖括号占位」「lib_project_code_runs 自检：要临时服务：仓里 crates-mutation-rows.py 的裸 --selftest 被点名、给整条包装写法」「lib_project_code_runs.py 的自检」「经 task-server：子 agent 直接裸写要临时服务的自证被拒时，说明写进报告「要主 agent 跑的命令」一节、给整条包装写法」「stdin:主 agent 经 task-server 提交要临时服务的自证裸写 --selftest 拒绝（退出码 2），stderr 给包装写法的整条」「lib_project_code_runs 自检：要临时服务：样本档点名要临时服务的一道、外面没套：说明点名那一道（不点不要的那一道）、给整条包装写法」「lib_project_code_runs 自检：要临时服务：仓里门禁 crates-mutation-replay.sh 的样本档裸写被点名、给整条包装写法」「经 task-server：样本档点名要临时服务的一道、外面没套被拒时，说明点名那一道、给整条包装写法」「stdin:主 agent 经 task-server 提交样本档点名要临时服务的一道、外面没套 拒绝（退出码 2），stderr 给包装写法的整条」
#   LIB_PROJECT_CODE_RUNS_BREAK=temporary-service-mark-entry-only  拆分族只算入口自己（临时服务核对挪进拆出的同目录文件的 required 行读成「源码里没有核对」）：钩子这一侧实测红的格（照失败行原文；只在 lib_project_code_runs.py 的自检里）
#       「lib_project_code_runs.py 的自检」
#       「lib_project_code_runs 自检：要临时服务：样本表写坏的、两边对不上的几行逐条报出来（判定、理由、键重复、文件不在、没登记、源码里没有核对、漏登）」「lib_project_code_runs 自检：要临时服务：拆分族里记号挪进同目录部分文件的 required 行不报」「lib_project_code_runs 自检：要临时服务：拆分族：python 入口连同它 import 的同目录模块」「lib_project_code_runs 自检：要临时服务：拆分族：shell 入口连同它 source 的同目录库」「lib_project_code_runs 自检：要临时服务：仓里的表」「lib_project_code_runs 自检：要临时服务：拆分族：入口按字面量参数的 importlib.import_module 载入的同目录部分文件在族里」「lib_project_code_runs 自检：要临时服务：拆分族：门禁阶段入口给绝对路径时 lib/ 下 source 的部分文件在族里」
#   LIB_PROJECT_CODE_RUNS_BREAK=temporary-service-family-by-directory  拆分族把同目录的每一份 .py 与 .sh 都算进去（不看命名与载入）：钩子这一侧红的格（实测；只在 lib_project_code_runs.py 的自检里）
#       「lib_project_code_runs.py 的自检」
#       「lib_project_code_runs 自检：要临时服务：样本表写坏的、两边对不上的几行逐条报出来（判定、理由、键重复、文件不在、没登记、源码里没有核对、漏登）」「lib_project_code_runs 自检：要临时服务：拆分族里哪一份都没有记号的 required 行照旧报源码里没有核对」「lib_project_code_runs 自检：要临时服务：拆分族：python 入口连同它 import 的同目录模块」「lib_project_code_runs 自检：要临时服务：拆分族：shell 入口连同它 source 的同目录库」「lib_project_code_runs 自检：要临时服务：拆分族：别的目录里的模块不在族里」「lib_project_code_runs 自检：要临时服务：拆分族：名字像部分文件而入口没载入的同目录文件不在族里，记号只在它里面的 required 行照旧报源码里没有核对」「lib_project_code_runs 自检：要临时服务：拆分族：入口 import 的共用库与字符串里点名的别的脚本不在族里，记号只在它们里面的 required 行照旧报源码里没有核对」「lib_project_code_runs 自检：要临时服务：拆分族：入口注释里点名、文档串里写 import 行的同目录文件不在族里」「lib_project_code_runs 自检：要临时服务：拆分族：入口按字面量参数的 importlib.import_module 载入的同目录部分文件在族里」
#   LIB_PROJECT_CODE_RUNS_BREAK=temporary-service-family-by-name  拆分族只按名字认（命名族里的同目录文件都算，不看入口载没载入）：钩子这一侧红的格（实测；只在 lib_project_code_runs.py 的自检里）
#       「lib_project_code_runs.py 的自检」
#       「lib_project_code_runs 自检：要临时服务：样本表写坏的、两边对不上的几行逐条报出来（判定、理由、键重复、文件不在、没登记、源码里没有核对、漏登）」「lib_project_code_runs 自检：要临时服务：拆分族：shell 入口连同它 source 的同目录库」「lib_project_code_runs 自检：要临时服务：拆分族：名字像部分文件而入口没载入的同目录文件不在族里，记号只在它里面的 required 行照旧报源码里没有核对」「lib_project_code_runs 自检：要临时服务：拆分族：入口注释里点名、文档串里写 import 行的同目录文件不在族里」
#   LIB_PROJECT_CODE_RUNS_BREAK=temporary-service-family-by-loading  拆分族只按载入认（入口或族里别的文件载入的同目录文件都算，不看名字像不像部分文件）：钩子这一侧红的格（实测；只在 lib_project_code_runs.py 的自检里）
#       「lib_project_code_runs.py 的自检」
#       「lib_project_code_runs 自检：要临时服务：样本表写坏的、两边对不上的几行逐条报出来（判定、理由、键重复、文件不在、没登记、源码里没有核对、漏登）」「lib_project_code_runs 自检：要临时服务：拆分族：入口 import 的共用库与字符串里点名的别的脚本不在族里，记号只在它们里面的 required 行照旧报源码里没有核对」
#   LIB_PROJECT_CODE_RUNS_BREAK=temporary-service-table-lenient  要临时服务的表写坏的行照收、不报：钩子这一侧实测红的格（照失败行原文；只在 lib_project_code_runs.py 的自检里）
#       「lib_project_code_runs.py 的自检」
#       「lib_project_code_runs 自检：要临时服务：理由太短的一行不算登记」「lib_project_code_runs 自检：要临时服务：样本表写坏的、两边对不上的几行逐条报出来（判定、理由、键重复、文件不在、没登记、源码里没有核对、漏登）」
#   LIB_PROJECT_CODE_RUNS_BREAK=project-scripts-unjudged  lib_project_code_runs.py 的 project_code_use 一律交 None：钩子这一侧实测红的格（照失败行原文；同一条里另有编译、包装、那张表认出的一样的照拒，不红）
#       「lib_project_code_runs.py 的自检」
#       「lib_project_code_runs 自检：本仓代码：python3 起本仓的脚本」「lib_project_code_runs 自检：本仓代码：bash 起本仓的钩子（喂钩子）」「lib_project_code_runs 自检：本仓代码：仓副本里改过正文的文本工具不算名单上的」「lib_project_code_runs 自检：本仓代码：改名拷走、文件头带准入声明」「lib_project_code_runs 自检：本仓代码：python3 -m pytest 起单测」「lib_project_code_runs 自检：本仓代码：文本工具带 --selftest 不在名单的参数里」「lib_project_code_runs 自检：本仓代码：work-directory-ownership.sh 的 sweep 不在名单的参数里」「lib_project_code_runs 自检：本仓代码：gate-overlap.py 不带 --list」「lib_project_code_runs 自检：本仓代码：minimal-copy.py 带 --selftest 不在名单的参数里」「lib_project_code_runs 自检：本仓代码：active-sessions.py 带 --selftest 不在名单的参数里」「lib_project_code_runs 自检：本仓代码：lib_repository_copy.py 的 link-into 不在名单的参数里」「lib_project_code_runs 自检：名单：replace-once.py 定点改」「lib_project_code_runs 自检：名单：仓副本里逐字节相同的 replace-once.py」「lib_project_code_runs 自检：名单：认领草稿目录」「lib_project_code_runs 自检：名单：gate-overlap.py --list」「lib_project_code_runs 自检：名单：提交客户端的 wait」「lib_project_code_runs 自检：名单：提交客户端的 clean」「lib_project_code_runs 自检：名单：task-server/control check」「lib_project_code_runs 自检：名单：task-server/control bookings」「lib_project_code_runs 自检：名单：task-server/control key-stats」「lib_project_code_runs 自检：名单：改标签的 relabel-item.py」「lib_project_code_runs 自检：名单：备份原文件的 copy-originals.py」「lib_project_code_runs 自检：名单：直接起 ledger-append-section.py 不拒」「lib_project_code_runs 自检：名单：直接起 append-record.py 不拒」「lib_project_code_runs 自检：名单：直接起 quote-rust-items.py 不拒」「lib_project_code_runs 自检：名单：建仓副本的 minimal-copy.py」「lib_project_code_runs 自检：名单：打副本闭包的 lib_repository_copy.py entries」
#       「lib_project_code_runs 自检：名单：主 agent 停进程的 proc.py」「lib_project_code_runs 自检：名单：主 agent 的 task-server/control stop」「lib_project_code_runs 自检：名单：主 agent 的 task-server/control pause」「lib_project_code_runs 自检：名单：主 agent 的 task-server/control recognized-card-lease」「lib_project_code_runs 自检：名单：主 agent 的 task-server/control resume」「lib_project_code_runs 自检：名单：主 agent 的 gpu-verification-server/control start」「lib_project_code_runs 自检：名单：主 agent 直接起服务自证」「lib_project_code_runs 自检：名单：服务自证带别的参数不认」「lib_project_code_runs 自检：名单：主 agent 起定时提醒的 remind.sh」「lib_project_code_runs 自检：名单：主 agent 查活动会话的 active-sessions.py」「lib_project_code_runs 自检：名单：确认单打的 task confirm 主 agent 直接起」「实现员交回前自查命名：code-source-discipline.sh 点名那四格也是直接起本仓的门禁阶段，照拒（写进报告交主 agent 跑）」「实现员交回前自查命名：经范围声明与 nice 起、等号写法点名一格，照拒」「实现员交回前自查命名：点名里夹着别的格：照拒」「实现员交回前自查命名：不带 --check 整道：照拒」「书记员跑 doc-decisions（直接起本仓的门禁阶段）」「通用 agent 跑 code-tooling.sh 的 agent-write-scope 格（直接起本仓的门禁阶段）」「主 agent 跑 code-tooling.sh（直接起本仓的门禁阶段）」「主 agent 跑 gate-lint.sh」「实现员跑 gate-structure-check.py」「主 agent 跑 hooks-registered.sh」「经 task-server：子 agent 直接起查活动会话的 active-sessions.py：照拒」「实现员复现输入那一条（反引号里的 walk.rs）：末尾的 doc-lint.sh 是直接起的本仓脚本，照拒」「一百三十七#8：工具实现员经 multi-host-run.sh --peer 起门禁格，照拒」「stdin:主 agent 直接跑被拒时最前面是能照抄的经 task-server 提交的那条（照抄的那条再判放行）」「stdin:主 agent 被拒时不贴「谁能跑哪一类」那一段，规矩指到提交写法」「stdin:主 agent 直接跑整道门禁被拒时照抄的那条经 task-server 提交、四段范围声明写在 submit 前面（为什么是占位），照它再判放行」
#   LIB_PROJECT_CODE_RUNS_BREAK=text-tools-refused  TEXT_TOOLS_RUN_DIRECTLY 不认：钩子这一侧实测红的格（照失败行原文）
#       「lib_project_code_runs.py 的自检」
#       「lib_project_code_runs 自检：展开：通配起文本工具 scripts/kb/replace-onc?.py 不拒（谁都能直接跑）」「lib_project_code_runs 自检：名单：replace-once.py 定点改」「lib_project_code_runs 自检：名单：仓副本里逐字节相同的 replace-once.py」「lib_project_code_runs 自检：名单：认领草稿目录」「lib_project_code_runs 自检：名单：gate-overlap.py --list」「lib_project_code_runs 自检：名单：提交客户端的 wait」「lib_project_code_runs 自检：名单：提交客户端的 clean」「lib_project_code_runs 自检：名单：task-server/control check」「lib_project_code_runs 自检：名单：task-server/control bookings」「lib_project_code_runs 自检：名单：task-server/control key-stats」「lib_project_code_runs 自检：名单：改标签的 relabel-item.py」「lib_project_code_runs 自检：名单：备份原文件的 copy-originals.py」「lib_project_code_runs 自检：名单：直接起 ledger-append-section.py 不拒」「lib_project_code_runs 自检：名单：直接起 append-record.py 不拒」「lib_project_code_runs 自检：名单：直接起 quote-rust-items.py 不拒」「lib_project_code_runs 自检：名单：建仓副本的 minimal-copy.py」「lib_project_code_runs 自检：名单：打副本闭包的 lib_repository_copy.py entries」「lib_project_code_runs 自检：名单：确认单打的 task confirm 主 agent 直接起」「经 task-server：管道读 JSON 的 python3 -c 代码里写着 'task'，不当写了后面执行的 task-server/task：主 agent 放行」「经 task-server：循环里管道读 JSON 的 python3 -c 代码里写着 'task'，不当写了 task-server/task：主 agent 放行」
#   LIB_PROJECT_CODE_RUNS_BREAK=main-agent-tools-refused  MAIN_AGENT_TOOLS_RUN_DIRECTLY 不认：钩子这一侧实测红的格（照失败行原文）
#       「lib_project_code_runs.py 的自检」
#       「lib_project_code_runs 自检：名单：主 agent 停进程的 proc.py」「lib_project_code_runs 自检：名单：主 agent 的 task-server/control stop」「lib_project_code_runs 自检：名单：主 agent 的 task-server/control pause」「lib_project_code_runs 自检：名单：主 agent 的 task-server/control recognized-card-lease」「lib_project_code_runs 自检：名单：主 agent 的 task-server/control resume」「lib_project_code_runs 自检：名单：主 agent 的 gpu-verification-server/control start」「lib_project_code_runs 自检：名单：主 agent 直接起服务自证」「lib_project_code_runs 自检：名单：主 agent 起定时提醒的 remind.sh」「lib_project_code_runs 自检：名单：主 agent 查活动会话的 active-sessions.py」「经 task-server：主 agent 直接起查活动会话的 active-sessions.py：放行」
#   LIB_PROJECT_CODE_RUNS_BREAK=task-server-submission-unparsed  task_server_submission 一律交 None（submit 那一条只当名单上的客户端放行）：钩子这一侧实测红的格（照失败行原文）
#       「lib_project_code_runs.py 的自检」「经 task-server：提交带 --manual、--full-gate、--whole-package-confirmed 时 -- 后面那条照过闸」「stdin:主 agent 直接跑整道门禁被拒时照抄的那条经 task-server 提交、四段范围声明写在 submit 前面」「经 task-server：照抄：打出的每条照抄命令照 unnamed_run_problem 判都点了名」「经 task-server：要临时服务的自证裸写被拒时，说明点名登记表、给整条包装写法」「stdin:主 agent 经 task-server 提交要临时服务的自证裸写 --selftest 拒绝（退出码 2），stderr 给包装写法的整条」「经 task-server：提交没带全 check.sh lint 表的 clippy 被拒，说明点名缺的 lint、给整段现读的写法」「经 task-server：check.sh 的 lint 表读不成（另有 += 追加）时 clippy 被拒，说明点名读不成的行、出路交上游改」「经 task-server：clippy 带全表、deny 之后又 -A 表里的 lint 被拒，说明点名被放行的那一条」「经 task-server：clippy 带全表、deny 之后又 -A clippy::pedantic（一整组）被拒」「经 task-server：clippy 带全表又带 --cap-lints allow 被拒，说明点名 --cap-lints」「经 task-server：样本档点名要临时服务的一道、外面没套被拒时，说明点名那一道、给整条包装写法」「stdin:主 agent 经 task-server 提交样本档点名要临时服务的一道、外面没套 拒绝（退出码 2），stderr 给包装写法的整条」
#       「lib_project_code_runs 自检：提交：submit -- 后面那条」「lib_project_code_runs 自检：提交：--cwd 照这一条的目录解开」「lib_project_code_runs 自检：提交：仓副本里逐字节相同的客户端」「lib_project_code_runs 自检：提交：--manual 不带值，-- 后面那条照认」「lib_project_code_runs 自检：提交：--full-gate 不带值，-- 后面那条照认」「lib_project_code_runs 自检：提交：--whole-package-confirmed 带一个值，-- 后面那条照认」「lib_project_code_runs 自检：提交：几个选项混写，--cwd 照解开」「经 task-server：照抄：实现员直接跑 harness 测试，照抄的那条经 task-server 提交」「经 task-server：照抄：带 && 的整条包进 bash -c」
#   LIB_PROJECT_CODE_RUNS_BREAK=task-server-submit-flags-unparsed  task_server_submission 只认 --cwd（带 --manual、--full-gate、--whole-package-confirmed 的提交只当名单上的客户端放行）：钩子这一侧实测红的格（照失败行原文）
#       「lib_project_code_runs.py 的自检」「经 task-server：提交带 --manual、--full-gate、--whole-package-confirmed 时 -- 后面那条照过闸」「stdin:主 agent 直接跑整道门禁被拒时照抄的那条经 task-server 提交、四段范围声明写在 submit 前面（为什么是占位），照它再判放行」
#       「lib_project_code_runs 自检：提交：--manual 不带值，-- 后面那条照认」「lib_project_code_runs 自检：提交：--full-gate 不带值，-- 后面那条照认」「lib_project_code_runs 自检：提交：--whole-package-confirmed 带一个值，-- 后面那条照认」「lib_project_code_runs 自检：提交：几个选项混写，--cwd 照解开」
#   LIB_PROJECT_CODE_RUNS_BREAK=task-server-paths-unjudged  不认库、密钥、socket：钩子这一侧红的格（照失败行原文；推的，没跑）
#       「lib_project_code_runs.py 的自检」
#       「lib_project_code_runs 自检：绕开：点到库文件名」「lib_project_code_runs 自检：绕开：点到签名密钥文件名」「lib_project_code_runs 自检：绕开：点到 socket」「经 task-server：绕开：读签名密钥照拒，说明点名绕开写法」
#   LIB_PROJECT_CODE_RUNS_BREAK=admitted-variable-unjudged  不认「已由 task-server 准入」开关：钩子这一侧实测红的格（照失败行原文）
#       「lib_project_code_runs.py 的自检」
#       「lib_project_code_runs 自检：绕开：命令文本里 export 准入开关」「lib_project_code_runs 自检：绕开：python 代码里写 environ 的准入开关」「lib_project_code_runs 自检：绕开：命令前设准入开关」
#   LIB_PROJECT_CODE_RUNS_BREAK=task-server-scripts-unjudged  不认直接起服务本体与每次尝试的那一层：钩子这一侧实测红的格（照失败行原文；直接起的另由 ① 照拒）
#       「lib_project_code_runs.py 的自检」
#       「lib_project_code_runs 自检：绕开：直接起服务本体」
#   LIB_PROJECT_CODE_RUNS_BREAK=task-server-units-unjudged  不认 systemctl 停禁改它的单元：钩子这一侧实测红的格（照失败行原文）
#       「lib_project_code_runs.py 的自检」
#       「lib_project_code_runs 自检：绕开：systemctl --user stop 服务单元」「lib_project_code_runs 自检：绕开：systemctl --user kill 一次尝试的单元」「lib_project_code_runs 自检：绕开：systemctl --user stop 通配」
# 引导（下面读 python 部分文件的那一段）的弄坏开关：
#   PROJECT_CODE_GUARD_BREAK=part-unreadable-passes  引导载不全部分文件（读不到、编不过、编得过而 import 抛错或入口要的名字缺）时不按拒判：照原样抛错、进程退 1（bash 那一层照「放行但必记检出」放行）；只在自证起的子进程里认（引导读它，不经 python 部分文件）
#       「引导:读不到部分文件 project_code_guard_decisions.py 按拒判（退 2、点名那一份）」「引导:编不过部分文件 project_code_guard_decisions.py 按拒判（退 2、点名那一份）」「引导:截空部分文件 project_code_guard_decisions.py 按拒判（退 2、点名那一份）」
#       「引导:读不到部分文件 project_code_guard_selftest.py 按拒判（退 2、点名那一份）」「引导:编不过部分文件 project_code_guard_selftest.py 按拒判（退 2、点名那一份）」「引导:截空部分文件 project_code_guard_selftest.py 按拒判（退 2、点名那一份）」
# 共用模块 lib_project_code_runs.py 的部分文件读不到或编不过：库抛 PartFileUnreadable（ImportError 的子类，说明点名那一份），入口照读不到共用模块处置（文件头「判不了的几样都照常执行」那一段）：照常执行、stderr 点名、记一条检出。
#   LIB_PROJECT_CODE_RUNS_BREAK=part-unreadable-passes  库读不到或编不过部分文件时照原样抛读文件错与语法错（不换成 PartFileUnreadable、不点名「部分文件」）：钩子这一侧实测红的格（照失败行原文）
#       「lib_project_code_runs.py 的自检」
#       「lib_project_code_runs 自检：引导:读不到部分文件 lib_project_code_runs_base.py 抛 PartFileUnreadable、点名那一份」「lib_project_code_runs 自检：引导:编不过部分文件 lib_project_code_runs_base.py 抛 PartFileUnreadable、点名那一份」
#       「lib_project_code_runs 自检：引导:读不到部分文件 lib_project_code_runs_launchers.py 抛 PartFileUnreadable、点名那一份」「lib_project_code_runs 自检：引导:编不过部分文件 lib_project_code_runs_launchers.py 抛 PartFileUnreadable、点名那一份」
#       「lib_project_code_runs 自检：引导:读不到部分文件 lib_project_code_runs_selftest.py 抛 PartFileUnreadable、点名那一份」「lib_project_code_runs 自检：引导:编不过部分文件 lib_project_code_runs_selftest.py 抛 PartFileUnreadable、点名那一份」
#       「stdin:共用模块的部分文件读不到照常放行（退出码 0）、stderr 点名那一份、记一条检出」
# python 代码分在同目录的这几份（本文件是入口：bash 这一层与下面的引导；引导先把每一份读进来、核顶层名字、语法解析过（只读语法树、不执行），再按次序 import 它们（每一份是普通 python 模块，用到别份的名字写显式 import、不留环），不单独执行）：
# project_code_guard_decisions.py：共用库的读入、弄坏开关名单、clippy lint 表、直接跑项目代码的认法（扫命令文本与读进来的脚本、cp 与 tee 与 heredoc 写出的文件）、绕开服务、临时服务、范围声明与提交标识、拒绝说明、检出记录
# project_code_guard_selftest.py：自证的样本脚本、selftest（拒绝与放行的样本、走真实入口的几格、开关轮）、弄坏开关只在自证里认的拒绝与 main
# 载不全任何一份（读不到、编不过、编得过而不完整：顶层没有入口要的名字——PROJECT_CODE_GUARD_PART_REQUIRED_NAMES 一份一组，import 之前用 ast 静态核、点名缺哪个——或 import 抛错）：照缺依赖的处置按拒判，退 2、stderr 点名是哪一份，不抛错退 1（退 1 时 bash 那一层当「没判成」放行）；
# 这一份恢复之前拒每一条 Bash 命令，恢复要用 Edit、Write；主函数 main 在引导核完之后由引导调用，部分文件末尾不自己退出。
# 弄坏开关的名单、读开关的代码在这几份 python 里（引导里那一条在引导里），开关表与红的格写在这个文件头。
set -uo pipefail
# 钩子 bash 这一层起不来时也记一条检出：python3 起不来、/dev/fd/3 读不到、python 在 main() 之前崩了，这条命令照常执行（钩子退出码不是 0、2 时 Claude Code 照常放行），
# 标准输入先在这里读进来、经 here-string 交给 python（python 起不来时 bash 还拿得到会话号）；退出码不是 0、2、又不是 --selftest 的，照「放行但必记检出」
# 往检出记录追加一行（会话号、agent 从输入里按字面取，取不到写「读不到」）、退 0。弄坏开关 PROJECT_CODE_GUARD_BREAK=bash-layer-unrecorded 下不记（照旧把 python 的退出码交出去）。
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
if [[ $# -eq 0 ]]; then hook_input="$(cat)"; fi   # 只有钩子那种起法（不带参数，从标准输入读 JSON）读；--selftest 不读
# PATH 上没有 python3：下面的 preflight 判不了运行条件，会退 78、什么都不记；钩子那种起法在它之前照「放行但必记检出」记一条、退 0（
# 弄坏开关 PROJECT_CODE_GUARD_BREAK=python-missing-unrecorded 下照旧交给 preflight）
if [[ $# -eq 0 ]] && ! command -v python3 >/dev/null 2>&1 && [[ ",${PROJECT_CODE_GUARD_BREAK:-}," != *,python-missing-unrecorded,* ]]; then
  hook_record_bash_layer_failure "project-code-guard.sh" "127，PATH 上没有 python3" "$hook_input"
  exit 0
fi
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
HOOK_DIR="$(cd "$(dirname "$0")" && pwd)"
# python 程序从文件描述符 3 读，标准输入留给 hook 的 JSON（与 write-guard.sh 同一个坑）。
python3 /dev/fd/3 "$HOOK_DIR" "$@" <<<"$hook_input" 3<<'PY'
import ast, os, sys
PROJECT_CODE_GUARD_PART_NAMES = ("project_code_guard_decisions.py", "project_code_guard_selftest.py",)   # 按次序 import（后一份 import 前一份、没有环）；路径按钩子自己所在的目录（argv[1]）取
PROJECT_CODE_GUARD_PART_REQUIRED_NAMES = (   # 与上一行同次序、一份部分文件一组：入口要的名字（这一份第一个、最后一个绑定的名字与别份、入口引到的名字）；import 之前用 ast 核这一份顶层的 def、class 与赋值绑定了这些名字，缺一个就是不完整（截空、截短）
    ("project_code_runs_library_path", "load_project_code_runs", "parsed_hook_input", "record_detection"),
    ("build_sample_scripts", "selftest", "refuse_break_outside_selftest", "main"),
)
# 弄坏开关只在自证里认：自证起的子进程靠这两个值认（自证与读开关的代码在 project_code_guard_selftest.py）；引导在部分文件载入之前就读 PROJECT_CODE_GUARD_BREAK，所以定义放在这里
SELFTEST_MARK_VARIABLE = "PROJECT_CODE_GUARD_SELFTEST_MARK"
SELFTEST_MARK_CONTENT = "project-code-guard.sh --selftest 起的子进程"

def hook_part_unusable(hook_part_path, hook_part_error):
    """一份部分文件载不全（读不到、编不过、编得过而不完整）：按缺依赖的处置拒（退 2、点名是哪一份）。"""
    if os.environ.get(SELFTEST_MARK_VARIABLE) == SELFTEST_MARK_CONTENT and "part-unreadable-passes" in os.environ.get("PROJECT_CODE_GUARD_BREAK", "").split(","):
        raise hook_part_error   # 弄坏开关 part-unreadable-passes：照原样抛错、进程退 1（bash 那一层照「放行但必记检出」放行；只在自证起的子进程里认）
    print(f"  ✗ project-code-guard.sh：读不到或编不过部分文件 {hook_part_path}（{type(hook_part_error).__name__}：{hook_part_error}），这条命令没法判，按拒判", file=sys.stderr)
    print("     → 怎么办：用 Edit、Write 把这一份恢复成编得过、完整的版本（项目代码闸挂在 Bash 上，这一份恢复之前拒每一条 Bash 命令，Bash 里恢复不了；写闸挂在 Write、Edit 上，不受它影响）", file=sys.stderr)
    sys.exit(2)

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
for hook_part_name, hook_part_required in zip(PROJECT_CODE_GUARD_PART_NAMES, PROJECT_CODE_GUARD_PART_REQUIRED_NAMES, strict=True):   # 每一份先读进来、核顶层绑定了入口要的名字、语法解析过（只读语法树、不执行），全核完全过了才 import：读不到、语法错或缺名字的，按缺依赖的处置（点名是哪一份、缺哪个名字），不让 python 在 main 之前抛错退 1 而被照常放行
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
    import project_code_guard_decisions
except Exception as hook_part_error:
    hook_part_unusable(os.path.join(sys.argv[1], "project_code_guard_decisions.py"), hook_part_error)
try:
    from project_code_guard_selftest import main
except Exception as hook_part_error:
    hook_part_unusable(os.path.join(sys.argv[1], "project_code_guard_selftest.py"), hook_part_error)
sys.exit(main())   # 主函数由引导在核完之后调，不靠部分文件末行自己退出
PY
hook_status=$?
if [[ $# -eq 0 && "$hook_status" != 0 && "$hook_status" != 2 && ",${PROJECT_CODE_GUARD_BREAK:-}," != *,bash-layer-unrecorded,* ]]; then
  hook_record_bash_layer_failure "project-code-guard.sh" "$hook_status" "$hook_input"
  exit 0
fi
exit "$hook_status"
