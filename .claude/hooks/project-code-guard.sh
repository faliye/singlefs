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
#   编译目录那一道（下面「再一道」）交给 lib_workspace_cargo_lock.py 的判定库同样剥它（DispatchLaunchersSeen）：里面裸起的 cargo 照拒。
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
#      有 shell 写法的包进 `bash -c`，submit_candidate）；子 agent 那一份的出路是把同一条（照主 agent 交它再判过放行才给）写进报告「要主 agent 跑的命令」一节交回
#      （subagent_task_server_policy，出路那一句是 lib_project_code_runs.SUBAGENT_HAND_BACK_WAY_OUT）。
#      直接跑的只剩：不是本仓文件的命令（只读的 git、ls、grep、cat、sed -n、wc、sha256sum、diff、du、ps 这类与草稿里自己写的、里面不跑项目代码的脚本）、语法检查（bash -n、python3 -m py_compile）、
#      lib_project_code_runs.py 的 TEXT_TOOLS_RUN_DIRECTLY（文本工具直接跑：replace-once.py、replace-batch.py、insert-row.py、work-directory-ownership.sh claim、kb-spec-check.py、
#      stale-candidates.py、kb-sections.py、checklist-specs.py、cite-check.py、claim-experiment.sh、gate-overlap.py --list，主 agent 定的 relabel-item.py、copy-originals.py，用户授权的 append-record.py，
#      加 task-server/task 的 submit / wait / status / cancel / log / clean 与 task-server/control check；子 agent 起客户端另照 ② 只许 status、log），与只给主 agent 的 MAIN_AGENT_TOOLS_RUN_DIRECTLY（proc.py、remind.sh、watch.sh、agent-watch.py、dispatch-prompt.py、
#      task-server/control 的 install / start / stop / watch：起在服务的单元里就丢了会话身份或停不了服务自己）。名单上的脚本整份放行，读进来的正文里那几行不另算（listed_scripts）。
#      经 task-server/task submit 交给服务的那一条：不判内存包装（服务替它排队、定额度、挑机器、分卡；套了包装、带了退役变量的由服务当场拒，钩子不另判），
#      主 agent 交什么都放；`--` 之后那条照 scan_uses 另判一遍，只为认出里面绕开服务的写法（③）。
#      编译目录那一道（下面「再一道」）不判交给服务的那一条（编译目录归服务）。
#   ② 子 agent（输入里有 agent_type）起 task-server/task 只许只读的 status、log：submit、wait、cancel、clean（与没写子命令的）一律执行前拒，交什么命令都一样
#      （带 --item 的、harness 测试、clippy / build、自证、样本档、弄坏开关、命名检查都算）；读进来的脚本正文里起的照判，python3 起的、仓副本里逐字节相同的客户端照认。
#      判定只在 lib_project_code_runs.subagent_task_server_refusal（run-scope-guard.sh 判子 agent 经 submit 交的验证命令也调它）；scan_uses 把它记成 DirectRun，allowed 是
#      SUBAGENT_CLIENT_LEVEL，主 agent 不判。服务分不出提交方是不是子 agent（task-server/task_server_lib.py 文件头「身份」），这一判只在钩子里
#      （records/2026-10-29-subagent拆分提案.md 第一百七十四节）。说明在 subagent_task_server_policy：要跑的命令拼成 submit 的整条、写进报告「要主 agent 跑的命令」一节交回。
#      lib_project_code_runs.TEXT_TOOLS_RUN_DIRECTLY 里客户端那一行不动（主 agent 照它直接起）。
#   ③ 绕开服务的写法，经不经 submit、谁起都拒（lib_project_code_runs.task_server_text_bypass、task_server_command_bypass；名字与路径全取自 task-server/task_server_lib.py）：
#      命令里点到库文件、签名密钥、socket（路径或文件名；直接 sqlite3 / python3 -c 开库的同样），python 代码里载入 task_server_lib，在 task-server 之外设
#      RUN_WITH_MEMORY_CAP_ADMITTED_BY_TASK_SERVER，直接起 task-server/task_server.py、attempt_runner.py，systemctl 停、禁、改 singlefs-task-server.service 与 singlefs-task-* 单元
#      （status、show 这类只读的放行；task-server/control 自己的 install / start / stop / check / watch 照 ① 的名单），直接删服务的编译目录（rm、find -delete / -exec rm、
#      python 的 shutil.rmtree 这类点到 TASK_SERVER_BUILD_DIRECTORY 或状态目录下 builds/ 的；lib_project_code_runs.build_directory_removal；出路 task-server/task clean <号>）。说明在 TASK_SERVER_BYPASS_POLICY。
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
#   PROJECT_CODE_GUARD_BREAK=disable-check  scan_decision() 一进来就放行，整道闸不判（外部程序、认不出的写法、经 task-server 那几样都不拒）：实测红的格（照失败行原文；改名与挪过来的格照新名写）
#       「实现员跑 herd7 x.litmus」「实现员经内存包装起 cargo test -p 当前目录探针自己的包名：照拒」「实现员经内存包装与 bash -c 起 cargo run --package= 点名探针：照拒」「实现员经内存包装 env -C 换进探针再 cargo test -p 探针自己的包名：照拒」「实现员 cd ${D} 工作区根、经 timeout、nice、capped.sh、内存包装四层全量：照拒」「主 agent D=… 之后 cd $D 全量：照拒」「实现员交回前自查命名：点名里夹着别的格：照拒」「实现员交回前自查命名：不带 --check 整道：照拒」「实现员交回前自查命名：naming-lint.sh：放行：读进去几份脚本」「实现员在 bash -c 里跑 --all」「实现员在仓根裸跑 cargo test」「实现员起 qemu-system」「实现员跑 herd7」「实现员直接跑 herd7 -version 再带 litmus 文件：照拒」「实现员经内存包装起循环里 ls 出来的 验证包测试二进制：命令词认不出照拒」「实现员喂给 bash 的 heredoc 里跑 --all」「实现员 here-string 喂给 bash 的是变量：认不出照拒、照记检出」「实现员 here-string 喂给 bash 的是变量：认不出照拒、照记检出：记几条检出」「实现员 cat 文件经管道喂给 bash：放行并记检出：记几条检出」「主 agent 裸 cargo run E152」「实现员 /usr/bin/time -v 包一层跑 --all」「实现员 perf stat 包一层跑 --all」「主 agent time -p 起全量照拒」「主 agent time -- 起全量照拒」「主 agent builtin source 读进全量脚本照拒」「主 agent builtin source 读进全量脚本照拒：读进去几份脚本」「主 agent bash --rcfile 起全量脚本照拒」「主 agent bash --rcfile 起全量脚本照拒：读进去几份脚本」「主 agent bash -- 起全量脚本照拒」「主 agent bash -- 起全量脚本照拒：读进去几份脚本」「主 agent bash - 起全量脚本照拒」「主 agent bash - 起全量脚本照拒：读进去几份脚本」「主 agent function 关键字定义的函数里起全量照拒」「主 agent cp X{,.bak} 拷出再起全量脚本照拒」「主 agent cp X{,.bak} 拷出再起全量脚本照拒：读进去几份脚本」「主 agent cp 之后按 // 开头的路径起全量脚本照拒」「主 agent cp 之后按 // 开头的路径起全量脚本照拒：读进去几份脚本」「主 agent env -- 带不合变量名的赋值起全量照拒」「主 agent env =1 开头就是 = 的赋值起全量照拒」「主 agent bash -c -- 的代码里起全量照拒」「主 agent function 定义之后经命令替换调用照拒」「主 agent function 定义之后引号拆开拼写调用照拒」「主 agent function 定义之后花括号拆开拼写调用照拒」「主 agent function 定义之后反斜杠拆开拼写调用照拒」「主 agent cat 01> 拷出再起全量脚本照拒」「主 agent cat 01> 拷出再起全量脚本照拒：读进去几份脚本」「主 agent cat 源 "2"> 拷出（引号里的 2 是源）再起全量脚本照拒」「主 agent cat 源 "2"> 拷出（引号里的 2 是源）再起全量脚本照拒：读进去几份脚本」「主 agent cat 3> 再 >&3 拷出再起全量脚本照拒」「主 agent cat 3> 再 >&3 拷出再起全量脚本照拒：读进去几份脚本」「别的命令带几个文件参数写到 > 文件不当 cat 接起来：认不出写法写出再起，照拒」「重定向目标花括号展开成几个词：bash 报 ambiguous redirect 不写，放行：记几条检出」「heredoc 写文件：cat <<EOF 2> 写的那一份拿不到正文：认不出写法写出再起，照拒」「主 agent heredoc 写文件：cat <<EOF 3> 再 >&3 写出再起全量脚本照拒」「主 agent heredoc 写文件：cat <<EOF 3> 再 >&3 写出再起全量脚本照拒：读进去几份脚本」「主 agent bash 长选项写在 -c 前面照拒」「主 agent env -u 写在赋值前面照拒」「主 agent cp 带步长、两端宽度不一的补零序列拷出再起全量脚本照拒」「主 agent cp 带步长、两端宽度不一的补零序列拷出再起全量脚本照拒：读进去几份脚本」「主 agent case 一项的命令里起全量照拒」「主 agent case 的模式里的命令替换起全量照拒」「主 agent eval 字面代码里起全量照拒」「主 agent 同一条命令里 cp 再 mv 改名再起全量脚本照拒」「主 agent 同一条命令里 cp 再 mv 改名再起全量脚本照拒：读进去几份脚本」「主 agent cat > 拷出、另把 stderr 丢进 /dev/null 再起全量脚本照拒」「主 agent cat > 拷出、另把 stderr 丢进 /dev/null 再起全量脚本照拒：读进去几份脚本」「cat > 拷出、起的是 2> 写的那一份：认不出写法写出再起，照拒」「主 agent ln -s 的相对源从链接所在的目录算、经它起全量脚本照拒」「主 agent ln -s 的相对源从链接所在的目录算、经它起全量脚本照拒：读进去几份脚本」「ln -s 的相对源在链接所在的目录里不在：认不出写法写出再起，照拒」「主 agent $(bash <<EOF) 里起全量照拒」「主 agent <(bash <<EOF) 里起全量照拒」「主 agent {cargo,} 空的一份起全量照拒」「主 agent {,} 写在命令词前面起全量照拒」「主 agent cargo {,test} 空的一份起全量照拒」「主 agent cp 补零的序列拷出再起全量脚本照拒」「主 agent cp 补零的序列拷出再起全量脚本照拒：读进去几份脚本」「主 agent cd "$(git rev-parse --show-toplevel)" 之后 bash <<EOF 里起全量照拒」「主 agent FOO="$(pwd)" bash <<EOF 里起全量照拒」「书记员跑 doc-decisions（直接起本仓的门禁阶段）」「通用 agent 跑 code-tooling.sh 的 agent-write-scope 格（直接起本仓的门禁阶段）」「主 agent 跑 code-tooling.sh（直接起本仓的门禁阶段）」「主 agent 跑 gate-lint.sh」「工具实现员起带 case 模式行 .claude/gate.d/lib/*.py) 的脚本：放行：读进去几份脚本」「实现员跑 gate-structure-check.py」「主 agent 跑 hooks-registered.sh」「实现员起的脚本里有 cargo test、这一层放行：读正文：读进去几份脚本」「实现员经内存包装起那份脚本：读进去几份脚本」「实现员 heredoc 写出 cargo run 再起它、这一层放行：读正文：读进去几份脚本」「实现员 bash 起含它的脚本」「实现员 bash 起含它的脚本：读进去几份脚本」「实现员 ./ 直接执行含它的脚本」「实现员 ./ 直接执行含它的脚本：读进去几份脚本」「实现员经 capped.sh 4 bash 起含它的脚本」「实现员经 capped.sh 4 bash 起含它的脚本：读进去几份脚本」「实现员 source 含它的脚本」「实现员 source 含它的脚本：读进去几份脚本」「实现员 . 读进含它的脚本、nice 与 timeout 包一层」「实现员 . 读进含它的脚本、nice 与 timeout 包一层：读进去几份脚本」「实现员 cd 之后按绝对路径起含它的脚本、输出落文件」「实现员 cd 之后按绝对路径起含它的脚本、输出落文件：读进去几份脚本」「实现员起的脚本里再调含它的脚本」「实现员起的脚本里再调含它的脚本：读进去几份脚本」「主 agent 执行含它的脚本」「主 agent 执行含它的脚本：读进去几份脚本」「实现员起调回自己的脚本（只读一遍）：读进去几份脚本」「实现员 ./ 执行超过读的上限的 shell 脚本：放行并记检出：记几条检出」「实现员起 doc-lint.sh 第 43 行那种命令替换：片段不当路径：读进去几份脚本」「实现员起 doc-lint.sh 第 70 行那种命令替换：片段不当路径：读进去几份脚本」「实现员起的脚本里命令替换、反引号、算术后面接路径：片段不当路径：记几条检出」「实现员起的脚本里命令替换、反引号、算术后面接路径：片段不当路径：读进去几份脚本」「实现员起的脚本里命令替换里跑 --all」「实现员起的脚本里命令替换里跑 --all：读进去几份脚本」「实现员起的脚本里 command -v herd7：查名字不是跑：读进去几份脚本」「实现员起的脚本里数组赋值的元素：不当命令、不当路径：读进去几份脚本」「实现员 command herd7（不带 -v）照拒」「实现员同一行里起 .sh 之后把 python 源码喂给 python3 -：正文不当命令：读进去几份脚本」「实现员喂给 bash 的正文里有 source = 与带逗号的路径：不当脚本：读进去几份脚本」「实现员写成通配的 capped.sh 包全量：照拒」「主 agent 起崩溃点重放入口：照拒」「实现员经 two-logs.sh 起全量 cargo test：照拒」「实现员起不存在的脚本：放行并记检出：记几条检出」「实现员 ./ 执行不存在的脚本：放行并记检出：记几条检出」「实现员 bash 起不是文本的文件：放行并记检出：记几条检出」「实现员起超过上限的脚本：放行并记检出：记几条检出」「实现员起的脚本嵌套 7 层：读到第 6 层、看不全，放行并记检出：记几条检出」「实现员起的脚本嵌套 7 层：读到第 6 层、看不全，放行并记检出：读进去几份脚本」「实现员 cat > 文件 <<EOF 写出含 --all 的脚本再 bash 起它」「实现员 cat > 文件 <<EOF 写出含 --all 的脚本再 bash 起它：读进去几份脚本」「实现员 cat <<EOF > 文件 写出含 --all 的脚本、chmod 之后 ./ 执行」「实现员 cat <<EOF > 文件 写出含 --all 的脚本、chmod 之后 ./ 执行：读进去几份脚本」「实现员 tee 文件 <<EOF 写出含 --all 的脚本再 source」「实现员 tee 文件 <<EOF 写出含 --all 的脚本再 source：读进去几份脚本」「实现员 <<-EOF（去制表符）写出含 --all 的脚本再 bash 起它」「实现员 <<-EOF（去制表符）写出含 --all 的脚本再 bash 起它：读进去几份脚本」「实现员 cd 进子目录写出、回来按相对路径起」「实现员 cd 进子目录写出、回来按相对路径起：读进去几份脚本」「实现员 heredoc 写出的脚本里再写出、再起含 --all 的脚本」「实现员 heredoc 写出的脚本里再写出、再起含 --all 的脚本：读进去几份脚本」「实现员 bash -c 里写出含 --all 的脚本再起它」「实现员 bash -c 里写出含 --all 的脚本再起它：读进去几份脚本」「实现员命令替换里写出含 --all 的脚本再起它」「实现员命令替换里写出含 --all 的脚本再起它：读进去几份脚本」「实现员 heredoc 写出的脚本里写出另一份，外层后面再起那一份」「实现员 heredoc 写出的脚本里写出另一份，外层后面再起那一份：读进去几份脚本」「实现员先写出含 --all 的一行、再 >> 追加一行轻的：前面那行还在」「实现员先写出含 --all 的一行、再 >> 追加一行轻的：前面那行还在：读进去几份脚本」「实现员 tee -a 往盘上含 --all 的 s.sh 追加一行轻的：盘上那几行还在」「实现员 tee -a 往盘上含 --all 的 s.sh 追加一行轻的：盘上那几行还在：读进去几份脚本」「实现员 cat -n 带选项写出：认不出，照拒」「实现员 cp 盘上含 --all 的 s.sh 出来再起：读源文件」「实现员 cp 盘上含 --all 的 s.sh 出来再起：读源文件：读进去几份脚本」「实现员 cp -f 到目录里再起：读源文件」「实现员 cp -f 到目录里再起：读源文件：读进去几份脚本」「实现员 cp 同一条命令里 heredoc 写出的脚本再起」「实现员 cp 同一条命令里 heredoc 写出的脚本再起：读进去几份脚本」「实现员 cp 的源不在：认不出，照拒」「实现员 printf 写出再起：认不出，照拒」「实现员 ./ 直接执行没扩展名、#! 指到 bash 的脚本：读进去照拒」「实现员 ./ 直接执行没扩展名、#! 指到 bash 的脚本：读进去照拒：读进去几份脚本」「实现员 ./ 直接执行 .py 名字、#! 指到 sh 的脚本：读进去照拒」「实现员 ./ 直接执行 .py 名字、#! 指到 sh 的脚本：读进去照拒：读进去几份脚本」「实现员复现输入那一条（反引号里的 walk.rs）：末尾的 doc-lint.sh 是直接起的本仓脚本，照拒」「实现员同一条命令里起含 --all 的 s.sh 两遍：只判一遍」「实现员同一条命令里起含 --all 的 s.sh 两遍：只判一遍：读进去几份脚本」「实现员起的脚本里三种起法各起一遍 s.sh：s.sh 只判一遍」「实现员起的脚本里三种起法各起一遍 s.sh：s.sh 只判一遍：读进去几份脚本」「实现员先起盘上轻的 inner-rel.sh、再 heredoc 盖成含 --all 的再起：写出的变了重判」「实现员先起盘上轻的 inner-rel.sh、再 heredoc 盖成含 --all 的再起：写出的变了重判：读进去几份脚本」「实现员先起里面起了不存在脚本的 has-missing.sh、再经 outer-missing.sh 起它：检出照复用的那一遍记两条：记几条检出」「实现员先起里面起了不存在脚本的 has-missing.sh、再经 outer-missing.sh 起它：检出照复用的那一遍记两条：读进去几份脚本」「实现员同一份 rel.sh 换个当前目录再起：按相对路径起的脚本变了，重判」「实现员同一份 rel.sh 换个当前目录再起：按相对路径起的脚本变了，重判：读进去几份脚本」「实现员先经 7 层嵌套碰到 deep6.sh、再直接起它：那一遍撞上嵌套上限，重判」「实现员先经 7 层嵌套碰到 deep6.sh、再直接起它：那一遍撞上嵌套上限，重判：记几条检出」「实现员先经 7 层嵌套碰到 deep6.sh、再直接起它：那一遍撞上嵌套上限，重判：读进去几份脚本」「实现员先起 cyc-a.sh（里面绕回自己）、再起 cyc-b.sh：cyc-b.sh 那一遍绕回外层，重判」「实现员先起 cyc-a.sh（里面绕回自己）、再起 cyc-b.sh：cyc-b.sh 那一遍绕回外层，重判：读进去几份脚本」「一百三十七：命令替换里不加括号的 case 模式不提前收尾，里面的 --all 照拒」「一百三十七：命令替换里注释的 ) 不收尾，后面的 --all 照拒」「一百三十七：coproc 后面那条照判」「一百三十七：bash <(…) 照拒」「一百三十七：eval 变量里拼的命令照拒」「一百三十七：source <(…) 照拒」「一百三十七：命令词是 $(which cargo) 照拒」「一百三十七：for 循环变量当脚本路径照拒」「一百三十七：PATH 指到放脚本的目录、命令词不带斜杠的 .sh 照读正文」「一百三十七：PATH 指到放脚本的目录、命令词不带斜杠的 .sh 照读正文：读进去几份脚本」「一百三十七：bash 起通配写出的脚本路径照 bash 展开读正文」「一百三十七：bash 起通配写出的脚本路径照 bash 展开读正文：读进去几份脚本」「一百三十七：echo 写出再起照拒」「一百三十七：python 写出再起照拒」「一百三十七：cat 源 >> 拷出再起，照写出的内容判」「一百三十七：cat 源 >> 拷出再起，照写出的内容判：读进去几份脚本」「一百三十七：cat 轻脚本 &> 拷出再起，照内容判、放行：读进去几份脚本」「一百三十七：cat 源 | tee 拷出再起，照写出的内容判」「一百三十七：cat 源 | tee 拷出再起，照写出的内容判：读进去几份脚本」「一百三十七：tee 目的 < 轻脚本 拷出再起，照内容判、放行：读进去几份脚本」「一百三十七：bash -s < 文件 读那份文件判」「一百三十七：bash -s < 文件 读那份文件判：读进去几份脚本」「一百三十七：cat 文件 | bash 读那份文件判」「一百三十七：cat 文件 | bash 读那份文件判：读进去几份脚本」「一百三十七：sh -s < 轻脚本放行：读进去几份脚本」「一百三十七：xargs bash 起读进来的脚本照拒」「一百三十七：脚本里经自己的 run_logged 起 --all 照拒」「一百三十七：脚本里经自己的 run_logged 起 --all 照拒：读进去几份脚本」「一百三十七：脚本 source 自己目录下的库，读正文时库也读」「一百三十七：脚本 source 自己目录下的库，读正文时库也读：读进去几份脚本」「一百三十七：grep 的输出经管道喂给 bash：认不出照拒」「一百三十七：grep 的输出经管道喂给 bash：认不出照拒：记几条检出」「一百三十七：别处树里的 run-with-memory-cap.sh 拷贝照读正文：读进去几份脚本」「一百三十七：脚本正文里经数组变量起拼出路径的钩子自证（件 1 那三趟的形态），照拒」「一百三十七：同一份脚本主 agent 照拒」「一百三十七：cat 源里有不在的文件、&& 接起，照拒并点名不在的源」「拒绝说明写出脚本路径与行号」「嵌套的拒绝说明写出每一层的路径与行号」「命令替换里的拒绝说明写出它所在那一行」「heredoc 写出的脚本，拒绝说明写出它的路径与正文里的行号」「heredoc 写出的脚本里再写出的，逐层写出路径与行号」「复用上一遍的结果，拒绝说明照这一处的路径与行号写」「绕回外层的那一遍不复用，从别处起时照读回来」「复用上一遍的检出，照这一处的路径与行号写」「stdin:实现员跑 --all 拒绝（退出码 2）」「stdin:拒绝时 stderr 写了出路：规矩一句、子 agent 写进报告交回」「stdin:主 agent 直接跑被拒时最前面是能照抄的经 task-server 提交的那条（照抄的那条再判放行）」「stdin:主 agent 被拒时不贴「谁能跑哪一类」「stdin:被拒的检出记录写了结局「被拒」「stdin:判的时候抛异常放行（退出码 0）、stderr 写诊断、记一条检出」「stdin:here-string 喂给 bash 的是变量认不出照拒（退出码 2）、stderr 点名判不了、记一条检出」「stdin:主 agent 直接跑整道门禁被拒时照抄的那条经 task-server 提交、四段范围声明写在 submit 前面（为什么是占位），照它再判放行」「stdin:实现员起含被拒命令的脚本拒绝（退出码 2）」「stdin:脚本里的拒绝 stderr 写出脚本路径与行号」「stdin:实现员直接跑测试目标拒绝（退出码 2）」「stdin:子 agent 不经 task-server 的拒绝 stderr 写了照抄的提交、写进报告「要主 agent 跑的命令」「stdin:脚本文件不存在时 stderr 点名它、记一条检出」「stdin:heredoc 写出的含 --all 的脚本拒绝（退出码 2）」「stdin:heredoc 写出的脚本的拒绝 stderr 写出它的路径与行号」「stdin:admission.py 判的那一刻抛异常时，点名标了 #[ignore] 的登记目标、不带 --ignored 的 cargo test 照拒（退出码 2）」「编译目录：主 agent 在主工作区裸起 cargo build：拒（退出码 2）」「编译目录：子 agent 经内存包装在主工作区起 cargo test：拒（退出码 2）」「编译目录：拒绝说明给出经 task-server 提交的出路」「编译目录：function 定义之后拆开拼写调用：拒（退出码 2）」「一百三十七：BASH_ENV 写在命令前、指到字面路径的脚本，正文起全量，照拒」「一百三十七：BASH_ENV 写相对路径，按这条命令的当前目录解，照拒」「一百三十七：env BASH_ENV=… 起 bash，照拒」「一百三十七：前面 export BASH_ENV=… 再起 bash，照拒」「一百三十七：BASH_ENV 指到的脚本正文起全量，bash 起的又是一份轻脚本，两份都读，照拒」「一百三十七：BASH_ENV 指到的脚本 source 自己目录下的库，库也读，照拒」「一百三十七：BASH_ENV 指到的脚本正文起全量，实现员照拒」「一百三十七：BASH_ENV 指到的脚本正文只有轻活，放行：读进去几份脚本」「一百三十七：BASH_ENV 写成前面赋过字面值的变量，展开之后读正文，照拒」「一百三十七：ENV 指到的脚本正文起全量，sh 起的，照拒」「一百三十七：ENV 指到的脚本正文起全量，bash --posix 起的，照拒」「一百三十七：ENV 指到的脚本正文起全量，bash -o posix 起的，照拒」「一百三十七：BASH_ENV 写成命令替换拼出的路径，认不出，实现员照拒」「一百三十七：BASH_ENV 写成命令替换拼出的路径，认不出，主 agent 照拒」「一百三十七：BASH_ENV 写成没赋过值的变量，认不出，实现员照拒」「一百三十七：env 的 BASH_ENV 写成命令替换，认不出，实现员照拒」「一百三十七：ENV 写成命令替换，sh 起的，认不出，实现员照拒」「一百三十七：直接执行 #!/bin/bash 的脚本，BASH_ENV 指的文件照读，照拒」「一百三十七：直接执行 #!/usr/bin/env bash 的脚本，BASH_ENV 指的文件照读，照拒」「一百三十七：直接执行没有 #! 的 .sh，从 bash 里起是 bash 另起自己，BASH_ENV 指的文件照读，照拒」「一百三十七：env 写的 BASH_ENV，直接执行 #!/bin/bash 的脚本，照拒」「一百三十七：直接执行 #!/bin/sh 的脚本，ENV 指的文件照读，照拒」「一百三十七：直接执行 #!/bin/zsh 的脚本，ZDOTDIR 下的 .zshenv 照读，照拒」「一百三十七：直接执行 #!/bin/bash 的脚本，BASH_ENV 指的文件只有轻活，放行：读进去几份脚本」「一百三十七：直接执行 #!/bin/sh 的脚本不读 BASH_ENV：放行：读进去几份脚本」「一百三十七：直接执行 #!/bin/bash 的脚本不读 ENV：放行：读进去几份脚本」「一百三十七：直接执行 #! 指到认不出的解释器、环境里设了 BASH_ENV，实现员照拒」「一百三十七：直接执行 #!/bin/bash 的脚本，BASH_ENV 写成命令替换拼出的路径，认不出，实现员照拒」「一百三十七：declare -x BASH_ENV=…，再起 bash，照拒」「一百三十七：typeset -x BASH_ENV=…，再起 bash，照拒」「一百三十七：先 BASH_ENV=… 赋值、后 export BASH_ENV，再起 bash，照拒」「一百三十七：先 export BASH_ENV、后赋值，再起 bash，照拒」「一百三十七：export 过之后换成另一份脚本，起 bash 读的是后一份，照拒」「一百三十七：set -a 之后赋值的 BASH_ENV 进环境，照拒」「一百三十七：local -x BASH_ENV=… 的函数里起 bash，照拒」「一百三十七：declare -x BASH_ENV 写成命令替换拼出的路径，认不出，实现员照拒」「一百三十七：先赋值后 export 的 BASH_ENV 是命令替换拼出的路径，认不出，实现员照拒」「一百三十七：bash -i 读 HOME 下的 .bashrc，里面起全量，照拒」「一百三十七：bash -ic 合写的 -i 同样读 HOME 下的 .bashrc，照拒」「一百三十七：bash --rcfile 文件 -i，文件里起全量，照拒」「一百三十七：bash --init-file 文件 -i，文件里起全量，照拒」「一百三十七：bash -i 读 HOME 下只有轻活的 .bashrc：放行：读进去几份脚本」「一百三十七：bash -i 不读 BASH_ENV，.bashrc 只有轻活：放行：读进去几份脚本」「一百三十七：bash -i 不读 ENV：放行：读进去几份脚本」「一百三十七：sh -i 读 ENV 指的文件，照拒」「一百三十七：sh -i 读 ENV 指的文件，只有轻活：放行：读进去几份脚本」「一百三十七：bash --posix -i 读 ENV 指的文件，照拒」「一百三十七：bash --rcfile 写成命令替换拼出的路径，认不出，实现员照拒」「一百三十七：bash -i 的 HOME 写成命令替换，.bashrc 路径认不出，实现员照拒」「一百三十七：zsh 读 ZDOTDIR 下的 .zshenv，里面起全量，照拒」「一百三十七：zsh 读 ZDOTDIR 下的 .zshrc，里面起全量，照拒」「一百三十七：zsh 没设 ZDOTDIR 取 HOME 下的 .zshenv，照拒」「一百三十七：前面 export 的 ZDOTDIR，zsh 照读，照拒」「一百三十七：zsh 的启动文件只有轻活：放行：读进去几份脚本」「一百三十七：zsh 的 ZDOTDIR 写成命令替换，认不出，实现员照拒」「d137 H2：trap 'herd7 x.litmus' EXIT 照拒」「d137 H2：trap 'herd7 x.litmus' ERR 照拒」「d137 H2：trap 'herd7 x.litmus' 0 照拒」「d137 H2：trap 'bash startup-heavy.sh' DEBUG 读正文照拒」「d137 H2：trap -- 'herd7 x.litmus' EXIT（-- 之后是 action）照拒」「d137 H4：bash /dev/stdin < 文件 读那份文件判，照拒」「d137 H4：echo 代码 | bash /dev/stdin 照拒」「d137 H4：bash /dev/fd/0 <<< 代码 照拒」「d137 H4：source /dev/stdin <<< 代码 照拒」「d137 H4：bash /dev/stdin < 轻脚本 放行：读进去几份脚本」「d137 H5：cd 到不在的目录、cd 失败，相对脚本认不出在哪，照拒」「d137 H5：cd 目标写成命令替换、算不出，相对脚本认不出，照拒」「d137 H5：cd 到在的目录（.）起轻脚本，放行：读进去几份脚本」「d137 H6：bash ~+/脚本 照 $PWD 解、读正文照拒」「d137 H6：BASH_ENV=~+/脚本 起 bash，照 $PWD 解、读正文照拒」「d137 H6：bash ~-/脚本 的 OLDPWD 算不出，认不出起哪一份，照拒」「d137 H8：rbash -c 代码 照 shell 判、照拒」「d137 H8：rbash 起脚本 读正文照拒」「d137 H8：busybox sh -c 代码 照拒」「d137 H8：busybox ash -c 代码 照拒」「d137 H8：dash -c 代码 照拒」「d137 H8：ksh -c 代码 照拒」「d137 H3：别名展开成 herd7 x.litmus 照拒」「d137 H3：别名 cargo 自展开（换行、先定义后 shopt）照拒」「d137 H3：别名展开成起脚本（同一条里 alias 先提到脚本、认不出写法）照拒」「d137 A2：BASH_ENV=脚本 包进起 bash 的转发函数、跟进函数体读正文照拒」「d137 A2：env BASH_ENV=脚本 起转发函数照拒」「d137 A2：function 关键字定义的函数同样跟进、照拒」「d137 A2：BASH_ENV 指到轻脚本，放行：读进去几份脚本」「d137 H4：exec < 文件; bash 读那份文件判、照拒」「d137 H4：exec 0< 文件; bash -s 照拒」「d137 H4：{ bash; } < 文件 照拒」「d137 H4：( bash ) < 文件 照拒」「d137 H4：while … done < 文件 照拒」「d137 H4：exec < 轻文件; bash 放行：读进去几份脚本」「d137 H4：{ bash; } < 轻脚本 放行：读进去几份脚本」「d137 H4：exec < 变量; bash 来源算不出，照拒」「d137 H8：cp /bin/bash ./x; ./x -c 代码 照 bash 判、照拒」「d137 H8：ln -sf /bin/bash ./x; ./x -c 代码 照拒」「d137 H8：install /bin/bash ./x; ./x 脚本 读正文照拒」「d137 H8：cp /bin/bash ./x; ./x 轻脚本 放行：读进去几份脚本」「d137 H7：多源 cat 有盘上不在的源（在的源起全量）、写出再起，照拒」「d137 H7：多源 cat 有盘上不在的源（在的源无害），照拒」「d137 H7：源是 echo 写出的（认不出写法、盘上不在），照拒」「d137 H7：cat 源 >> 追加、源盘上不在，照拒」「d137 H7：cat 源 | tee 拷出、源盘上不在，照拒」「d137 H7：多源 cat 两个源都在、都无害，放行、读写出的内容：读进去几份脚本」「d137 H7：单源 cat 源在、起全量，照拒（控制组）」「d137 已知没认：登录 bash -l 读 HOME 下的 .bash_profile，里面起全量，照拒」「d137 已知没认：登录 bash --login 同样读 .bash_profile，照拒」「d137 已知没认：登录 sh -l 读 HOME 下的 .profile，照拒」「d137 已知没认：登录 exec -l bash 照登录 shell 读 .bash_profile，照拒」「d137 已知没认：登录 exec -a -bash bash 照登录 shell 读 .bash_profile，照拒」「d137 已知没认：登录 bash -l 读的 profile 只有轻活，放行：读进去几份脚本」「d137 已知没认：系统级 bash -l 读 /etc/profile（样本目录），里面起全量，照拒」「d137 已知没认：系统级 bash -i 读 /etc/bash.bashrc（样本目录），照拒」「d137 已知没认：系统级 bash --rcfile 轻文件 -i 照读 /etc/bash.bashrc，照拒」「d137 H9：python -c 里 os.system 起全量，照拒」「d137 H9：python 读 heredoc 里 subprocess.run 起全量，照拒」「d137 H9：python -c 里 os.system 的命令是变量，认不出照拒」「d137 H9：perl -e 里 system 起全量，照拒」「d137 H9：ruby -e 里 system 起全量，照拒」「d137 H9：node -e 里 child_process 起全量，照拒」「d137 H9：tclsh 读 here-string 里 exec 起全量，照拒」「d137 H9：gawk 的 system() 起全量，照拒」「d137 H9：awk 的 print 管道到全量，照拒」「d137 H9：awk -f 程序文件里 system() 起全量，读正文照拒」「d137 H9：awk -f 程序文件不起命令，放行：读进去几份脚本」「d137 H9：sed 的 e 命令起全量，照拒」「d137 H9：sed 的 s///e 执行模式空间，认不出照拒」「d137 H9：git -c 别名 ! 起全量，照拒」「d137 H9：git -c core.pager 起全量，照拒」「d137 H9：GIT_EXTERNAL_DIFF 起全量，照拒」「d137 H9：make 读 Makefile 配方起全量，照拒」「d137 H9：make 读 Makefile 配方只有轻活，放行：读进去几份脚本」「d137 H9：make -f /dev/stdin 读 here-string 的配方起全量，照拒」「d137 H9：tar --to-command 起全量，照拒」「d137 H9：ssh -o ProxyCommand 起全量，照拒」「d137 H9：脚本正文里 perl -e 起全量，读正文照拒」「d137-r2 A：sed 算术地址后接 e 起全量，照拒」「d137-r2 A：sed 命令替换落在命令位上，照拒」「d137-r2 A：sed 地址里的命令替换起全量，照拒」「d137-r2 B：sed 文本之后另起一行 e 起全量，照拒」「d137-r2 C：python argv 起 cargo test 而参数是变量，照拒」「d137-r2 C：python argv 头一项是变量，照拒」「d137-r2 D：python 先赋变量的字面列表起全量，照拒」「d137-r2 D：python 列表赋过之后被改，认不出照拒」「d137-r2 E：awk 里 cmd 赋全量再 | getline，照拒」「d137-r2 E：awk 里 cmd 是拼接的，认不出照拒」「d137-r2 F：cd "$PWD" 之后 bash 轻脚本，放行：读进去几份脚本」「d137-r2 F：cd "$PWD" 之后 bash 重脚本，照拒」「d137-r2 F：cd 到 git 顶层之后 bash 轻脚本，放行：读进去几份脚本」「d137-r2 F：cd 到 git 顶层之后 bash 重脚本，照拒」「d137-r2 F：cd "$OLDPWD" 回来之后 bash 轻脚本，放行：读进去几份脚本」「d137-r2 F：cd "$OLDPWD" 回来之后 bash 重脚本，照拒」「d137-r2 H5：cd 盘上不在的目录 && 相对脚本，cd 失败就不跑，放行：记几条检出」「d137-r2 H5：cd 盘上不在的目录 || exit 之后的相对脚本，放行：记几条检出」「d137-r2 H5：cd 盘上不在的目录 ; 之后的相对脚本照跑，照拒」「d137-r2 H5：(cd 不在的目录 || exit) 之后的相对脚本照跑，照拒」「d137-r2 H5：cd 不在的目录 && ls || 相对脚本照跑，照拒」「d137-r2 H5：mkdir 再 cd 进去跑同一条里写出的重脚本，照拒」「d137-r2 G：登录 bash -l 读 .profile 再 source 带非交互守卫的 .bashrc，守卫之后不读，放行：读进去几份脚本」「d137-r2 G：交互的 bash -i 读带守卫的 .bashrc 不截，守卫之后的全量照拒」「d137-r2 G：守卫不在顶层（套在 if 里）不截，照拒」「一百三十七：cat 追加到不是文本的文件之后接 herd7 x.litmus，照拒」「一百三十七：同一条主 agent 起，照拒」「一百三十七#8：工具实现员经 multi-host-run.sh --peer 起门禁格，照拒」「编译目录：子 agent 经 multi-host-run.sh --peer 在主工作区起 cargo test、没给编译目录：拒（退出码 2）」「d137 闸误拒 E2：systemd-run --user --wait 包着 bash 轻脚本，放行：读进去几份脚本」「d137 闸误拒 E2：chrt -i 0 包着 bash 轻脚本，放行：读进去几份脚本」「d137 闸误拒 E2：ssh localhost 包着 bash 轻脚本，放行：读进去几份脚本」「d137 闸误拒 E2：/usr/bin/time 包着 bash 轻脚本，放行：读进去几份脚本」「d137 闸误拒 E2：strace -o /dev/null 包着 bash 轻脚本，放行：读进去几份脚本」「d137 闸误拒 E2：prlimit --as 包着 bash 轻脚本，放行：读进去几份脚本」「d137 闸误拒 E2：numactl -N0 包着 bash 轻脚本，放行：读进去几份脚本」「d137 闸误拒 E2：unshare --user 包着 bash 轻脚本，放行：读进去几份脚本」「d137 闸误拒 E2：systemd-run --user --wait 包着 bash 重脚本，读正文照拒」「d137 闸误拒 E3：t 赋过字面路径，cat > $t 读 heredoc 写出再 bash $t，放行：读进去几份脚本」「d137 闸误拒 E3：$t 没赋过，cat > $t 写出再 bash $t，照拒」「d137 闸误拒 E5：W 赋过字面目录，awk -f $W/程序文件，读正文放行：读进去几份脚本」「d137 闸误拒 E5：W 没赋过，awk -f $W/程序文件认不出，照拒」「经 task-server：照抄：实现员直接跑 harness 测试，照抄的那条经 task-server 提交」「经 task-server：照抄：经内存包装直接起的去掉包装」「经 task-server：照抄：带 && 的整条包进 bash -c」「经 task-server：子 agent 交任务的拒绝说明写明写进报告「要主 agent 跑的命令」「经 task-server：要临时服务的自证裸写被拒时，说明点名登记表、给整条包装写法」「经 task-server：子 agent 直接裸写要临时服务的自证被拒时，说明写进报告「要主 agent 跑的命令」「stdin:主 agent 经 task-server 提交要临时服务的自证裸写 --selftest 拒绝（退出码 2），stderr 给包装写法的整条」
#   PROJECT_CODE_GUARD_BREAK=ignore-written-scripts  record_files_written() 不记，同一条命令里写出的脚本不认（执行时按盘上那份判，还不在的记「不存在」）：实测红的格（照失败行原文）
#       「主 agent cp X{,.bak} 拷出再起全量脚本照拒」「主 agent cp X{,.bak} 拷出再起全量脚本照拒：记几条检出」「主 agent cp X{,.bak} 拷出再起全量脚本照拒：读进去几份脚本」「主 agent cp 之后按 // 开头的路径起全量脚本照拒：读进去几份脚本」「主 agent cat 01> 拷出再起全量脚本照拒：读进去几份脚本」「主 agent cat 源 "2"> 拷出（引号里的 2 是源）再起全量脚本照拒：读进去几份脚本」「主 agent cat 3> 再 >&3 拷出再起全量脚本照拒：读进去几份脚本」「主 agent heredoc 写文件：cat <<EOF 3> 再 >&3 写出再起全量脚本照拒：读进去几份脚本」「主 agent cp 带步长、两端宽度不一的补零序列拷出再起全量脚本照拒：读进去几份脚本」「主 agent 同一条命令里 cp 再 mv 改名再起全量脚本照拒：读进去几份脚本」「主 agent cat > 拷出、另把 stderr 丢进 /dev/null 再起全量脚本照拒：读进去几份脚本」「主 agent ln -s 的相对源从链接所在的目录算、经它起全量脚本照拒：读进去几份脚本」「主 agent cp 补零的序列拷出再起全量脚本照拒：读进去几份脚本」「实现员 heredoc 写出 cargo run 再起它、这一层放行：读正文：读进去几份脚本」「实现员 cat > 文件 <<EOF 写出含 --all 的脚本再 bash 起它：读进去几份脚本」「实现员 cat <<EOF > 文件 写出含 --all 的脚本、chmod 之后 ./ 执行：读进去几份脚本」「实现员 tee 文件 <<EOF 写出含 --all 的脚本再 source：读进去几份脚本」「实现员 <<-EOF（去制表符）写出含 --all 的脚本再 bash 起它：读进去几份脚本」「实现员 cd 进子目录写出、回来按相对路径起：读进去几份脚本」「实现员 heredoc 写出的脚本里再写出、再起含 --all 的脚本：读进去几份脚本」「实现员 bash -c 里写出含 --all 的脚本再起它：读进去几份脚本」「实现员命令替换里写出含 --all 的脚本再起它：读进去几份脚本」「实现员 heredoc 写出的脚本里写出另一份，外层后面再起那一份：记几条检出」「实现员 heredoc 写出的脚本里写出另一份，外层后面再起那一份：读进去几份脚本」「实现员先写出含 --all 的一行、再 >> 追加一行轻的：前面那行还在：读进去几份脚本」「实现员 tee -a 往盘上含 --all 的 s.sh 追加一行轻的：盘上那几行还在：读进去几份脚本」「实现员 heredoc 写出 python 程序再 ./ 执行：不读也不记」「实现员 cp 盘上含 --all 的 s.sh 出来再起：读源文件：读进去几份脚本」「实现员 cp -f 到目录里再起：读源文件：读进去几份脚本」「实现员 cp 同一条命令里 heredoc 写出的脚本再起：读进去几份脚本」「实现员先起盘上轻的 inner-rel.sh、再 heredoc 盖成含 --all 的再起：写出的变了重判：读进去几份脚本」「一百三十七：cat 源 >> 拷出再起，照写出的内容判：读进去几份脚本」「一百三十七：cat 轻脚本 &> 拷出再起，照内容判、放行」「一百三十七：cat 轻脚本 &> 拷出再起，照内容判、放行：读进去几份脚本」「一百三十七：cat 源 | tee 拷出再起，照写出的内容判：读进去几份脚本」「一百三十七：tee 目的 < 轻脚本 拷出再起，照内容判、放行」「一百三十七：tee 目的 < 轻脚本 拷出再起，照内容判、放行：读进去几份脚本」「一百三十七：cat 源里有不在的文件、&& 接起，照拒并点名不在的源」「heredoc 写出的脚本，拒绝说明写出它的路径与正文里的行号」「heredoc 写出的脚本里再写出的，逐层写出路径与行号」「stdin:heredoc 写出的轻脚本照常放行（退出码 0）」「stdin:heredoc 写出的轻脚本 stderr 不报「不存在」「stdin:heredoc 写出的脚本的拒绝 stderr 写出它的路径与行号」「d137 H8：install /bin/bash ./x; ./x 脚本 读正文照拒：读进去几份脚本」「d137 H8：cp /bin/bash ./x; ./x 轻脚本 放行」「d137 H8：cp /bin/ls ./x; ./x 不是 shell、放行」「d137 H8：cp /bin/bash ./x; ./x -c 'echo hi' 无害放行」「d137 H7：多源 cat 两个源都在、都无害，放行、读写出的内容」「d137 H7：单源 cat 源在、起全量，照拒（控制组）：读进去几份脚本」「d137-r2 H5：mkdir 再 cd 进去跑同一条里写出的重脚本，照拒：读进去几份脚本」「d137 闸误拒 E3：t 赋过字面路径，cat > $t 读 heredoc 写出再 bash $t，放行」「实现员 heredoc 写出 cargo run 再起它、这一层放行：读正文」
#   PROJECT_CODE_GUARD_BREAK=detection-outcome-dropped  检出记录不写结局（outcome：被拒 / 只记）：实测红的格（照失败行原文）
#       「stdin:被拒的检出记录写了结局「被拒」「stdin:坏 JSON 放行（退出码 0）、stderr 写诊断、记一条检出」「stdin:不是 JSON 对象放行（退出码 0）、stderr 写诊断、记一条检出」「stdin:判的时候抛异常放行（退出码 0）、stderr 写诊断、记一条检出」「stdin:放行时的检出记录写了结局「只记」
#   PROJECT_CODE_GUARD_BREAK=copy-without-run-scope  被拒时照抄的那条不带四段范围声明（SINGLEFS_RUN_SCOPE，写法在 run-scope-guard.sh 文件头）
#       「stdin:主 agent 直接跑整道门禁被拒时照抄的那条经 task-server 提交、四段范围声明写在 submit 前面（为什么是占位），照它再判放行」「经 task-server：照抄：实现员直接跑 harness 测试，照抄的那条经 task-server 提交」「经 task-server：照抄：带 && 的整条包进 bash -c」
#   PROJECT_CODE_GUARD_BREAK=failure-unrecorded  main() 里判不了的两支（hook 输入不是 JSON 对象、decide() 抛异常）只打 stderr、不记检出
#       「stdin:坏 JSON 放行（退出码 0）、stderr 写诊断、记一条检出」「stdin:不是 JSON 对象放行（退出码 0）、stderr 写诊断、记一条检出」「stdin:判的时候抛异常放行（退出码 0）、stderr 写诊断、记一条检出」
#   PROJECT_CODE_GUARD_BREAK=copy-by-cp-only  copy_pairs() 只认 cp：同一条命令里 ln / install / mv 拷出、`cat 源 > 目的` 写出的脚本不认，执行时按盘上那份判，还不在的记「不存在」：实测红的格（照失败行原文）
#       「主 agent cat 01> 拷出再起全量脚本照拒：读进去几份脚本」「主 agent cat 源 "2"> 拷出（引号里的 2 是源）再起全量脚本照拒：读进去几份脚本」「主 agent cat 3> 再 >&3 拷出再起全量脚本照拒：读进去几份脚本」「主 agent 同一条命令里 cp 再 mv 改名再起全量脚本照拒：读进去几份脚本」「主 agent cat > 拷出、另把 stderr 丢进 /dev/null 再起全量脚本照拒：读进去几份脚本」「主 agent ln -s 的相对源从链接所在的目录算、经它起全量脚本照拒：读进去几份脚本」「一百三十七：cat 源 >> 拷出再起，照写出的内容判：读进去几份脚本」「一百三十七：cat 轻脚本 &> 拷出再起，照内容判、放行」「一百三十七：cat 轻脚本 &> 拷出再起，照内容判、放行：读进去几份脚本」「一百三十七：cat 源里有不在的文件、&& 接起，照拒并点名不在的源」「d137 H8：install /bin/bash ./x; ./x 脚本 读正文照拒：读进去几份脚本」「d137 H7：多源 cat 两个源都在、都无害，放行、读写出的内容」「d137 H7：单源 cat 源在、起全量，照拒（控制组）：读进去几份脚本」
#   WORKSPACE_CARGO_LOCK_BREAK=bare-cargo-allowed  （不判编译目录）：实测红的格（照失败行原文）
#       「编译目录：主 agent 在主工作区裸起 cargo build：拒（退出码 2）」「编译目录：子 agent 经内存包装在主工作区起 cargo test：拒（退出码 2）」「编译目录：拒绝说明给出经 task-server 提交的出路」「编译目录：function 定义之后拆开拼写调用：拒（退出码 2）」「lib_workspace_cargo_lock.py 的自检」「编译目录：子 agent 经 multi-host-run.sh --peer 在主工作区起 cargo test、没给编译目录：拒（退出码 2）」
#   WORKSPACE_CARGO_LOCK_BREAK=wrapper-not-recognized  （不认 run-build-directory.sh 包着的）：实测红的格（照失败行原文）
#       「编译目录：经 run-build-directory.sh 包一层起：放行（退出码 0）」「lib_workspace_cargo_lock.py 的自检」「编译目录：子 agent 经 multi-host-run.sh --peer 与 run-build-directory.sh 起：放行（退出码 0）」
#   WORKSPACE_CARGO_LOCK_BREAK=refusal-submit-dropped  （拒绝说明不写经 task-server 提交的出路）：红的格（推的）
#       「编译目录：拒绝说明给出经 task-server 提交的出路」「lib_workspace_cargo_lock.py 的自检」
#   WORKSPACE_CARGO_LOCK_BREAK=outside-target-not-recognized  （不认仓外的编译目录）：实测红的格（照失败行原文）
#       「编译目录：带仓外 CARGO_TARGET_DIR：放行（退出码 0）」「lib_workspace_cargo_lock.py 的自检」
#   WORKSPACE_CARGO_LOCK_BREAK=non-building-refused  （不编的子命令也判）：实测红的格（照失败行原文）
#       「编译目录：cargo fmt：放行（退出码 0）」「lib_workspace_cargo_lock.py 的自检」
#   LIB_SHELL_WORDS_BREAK=arguments-as-executed  `python3 -` 后面的词、`bash -n` 的目标当被执行的脚本：实测红的格（照失败行原文）
#       「bash -c -n 只查语法：放行」「工具实现员在门禁目录里 python3 - 读几份门禁（2026-09-28 误拦的那一条原样，目录换成自检的临时工作区）」「lib_project_code_runs.py 的自检」「一百三十七：bash -n 只查语法，不读 BASH_ENV：放行」（history-kept: 自证用例标签原文，按原文匹配）
#   LIB_SHELL_WORDS_BREAK=shell-standard-input-unread  here-string 与管道喂给 shell 的文本不当命令再切、也不交判不了的说明：实测红的格（照失败行原文）
#       「实现员 here-string 喂给 bash 的是变量：认不出照拒、照记检出」「实现员 here-string 喂给 bash 的是变量：认不出照拒、照记检出：记几条检出」「一百三十七：grep 的输出经管道喂给 bash：认不出照拒」「一百三十七：grep 的输出经管道喂给 bash：认不出照拒：记几条检出」「lib_project_code_runs.py 的自检」「stdin:here-string 喂给 bash 的是变量认不出照拒（退出码 2）、stderr 点名判不了、记一条检出」「d137 H4：echo 代码 | bash /dev/stdin 照拒」「d137 H4：bash /dev/fd/0 <<< 代码 照拒」
#   LIB_SHELL_WORDS_BREAK=redirect-fd-dropped  `2> 文件` 也当改标准输出（cat 拷出取最后一个输出重定向）：实测红的格（照失败行原文）
#       「主 agent cat 3> 再 >&3 拷出再起全量脚本照拒：读进去几份脚本」「heredoc 写文件：cat <<EOF 2> 写的那一份拿不到正文：认不出写法写出再起，照拒：读进去几份脚本」「主 agent heredoc 写文件：cat <<EOF 3> 再 >&3 写出再起全量脚本照拒：读进去几份脚本」「主 agent cat > 拷出、另把 stderr 丢进 /dev/null 再起全量脚本照拒：读进去几份脚本」「cat > 拷出、起的是 2> 写的那一份：认不出写法写出再起，照拒：读进去几份脚本」
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
#   LIB_SHELL_WORDS_BREAK=posix-uncalled-functions-run  只定义不调用的函数只认 function 关键字，`名字() { …; }` 的函数体照命令判：红的格（推的，没跑）
#       「名字() { 函数体 } 只定义不调用：函数体不跑，放行」
#   LIB_SHELL_WORDS_BREAK=shell-s-positionals-as-script  bash -s 后面的词照旧当脚本：实测红的格（照失败行原文）
#       「bash -s -- 后面的词是位置参数、不是脚本：放行」「bash -s -- 后面的词是位置参数、不是脚本：放行：读进去几份脚本」
#   LIB_SHELL_WORDS_BREAK=time-options-as-command  time 关键字后面的 -p / -- 当命令词：实测红的格（照失败行原文）
#       「主 agent time -p 起全量照拒」「主 agent time -- 起全量照拒」
#   LIB_SHELL_WORDS_BREAK=builtin-as-command  builtin 当命令词（后面的 source / cd 不认）：实测红的格（照失败行原文）
#       「主 agent builtin source 读进全量脚本照拒」「主 agent builtin source 读进全量脚本照拒：读进去几份脚本」
#   LIB_SHELL_WORDS_BREAK=shell-option-values-as-script  bash 的 --rcfile / --init-file 的值、-- 与 - 当脚本：实测红的格（照失败行原文）
#       「主 agent bash --rcfile 起全量脚本照拒」「主 agent bash --rcfile 起全量脚本照拒：读进去几份脚本」「主 agent bash -- 起全量脚本照拒」「主 agent bash -- 起全量脚本照拒：读进去几份脚本」「主 agent bash - 起全量脚本照拒」「主 agent bash - 起全量脚本照拒：读进去几份脚本」「主 agent bash -c -- 的代码里起全量照拒」「一百三十七：bash --rcfile 文件 -i，文件里起全量，照拒：读进去几份脚本」「一百三十七：bash --init-file 文件 -i，文件里起全量，照拒：读进去几份脚本」「一百三十七：bash --rcfile 不带 -i 是非交互，不读：放行」「一百三十七：bash --rcfile 指到的文件不在，什么都不读：放行：记几条检出」「d137 已知没认：系统级 bash --rcfile 轻文件 -i 照读 /etc/bash.bashrc，照拒：读进去几份脚本」「lib_project_code_runs.py 的自检」
#   LIB_SHELL_WORDS_BREAK=function-keyword-as-command  function 关键字当命令词：实测红的格（照失败行原文）
#       「主 agent function 关键字定义的函数里起全量照拒」「主 agent function 定义之后引号拆开拼写调用照拒」「主 agent function 定义之后花括号拆开拼写调用照拒」「主 agent function 定义之后反斜杠拆开拼写调用照拒」「编译目录：function 定义之后拆开拼写调用：拒（退出码 2）」「lib_workspace_cargo_lock.py 的自检」
#   LIB_SHELL_WORDS_BREAK=braces-unexpanded  引号外的花括号不展开：实测红的格（照失败行原文）
#       「主 agent cp X{,.bak} 拷出再起全量脚本照拒」「主 agent cp X{,.bak} 拷出再起全量脚本照拒：记几条检出」「主 agent cp X{,.bak} 拷出再起全量脚本照拒：读进去几份脚本」「主 agent cp 带步长、两端宽度不一的补零序列拷出再起全量脚本照拒」「主 agent cp 带步长、两端宽度不一的补零序列拷出再起全量脚本照拒：记几条检出」「主 agent cp 带步长、两端宽度不一的补零序列拷出再起全量脚本照拒：读进去几份脚本」「主 agent cp 补零的序列拷出再起全量脚本照拒」「主 agent cp 补零的序列拷出再起全量脚本照拒：记几条检出」「主 agent cp 补零的序列拷出再起全量脚本照拒：读进去几份脚本」「引号写出的空串花括号展开之后留着、当命令词：放行」「lib_project_code_runs.py 的自检」
#   LIB_SHELL_WORDS_BREAK=env-loose-assignments-as-command  env 后面名字不合变量名的 `名字=值` 当命令词：实测红的格（照失败行原文）
#       「主 agent env -- 带不合变量名的赋值起全量照拒」「主 agent env =1 开头就是 = 的赋值起全量照拒」「主 agent env -u 写在赋值前面照拒」
#   LIB_SHELL_WORDS_BREAK=eval-unread  eval 的字面代码不再切：实测红的格（照失败行原文）
#       「主 agent eval 字面代码里起全量照拒」「一百三十七：eval 变量里拼的命令照拒」
#   LIB_SHELL_WORDS_BREAK=heredoc-open-substitution-unseen  heredoc 挂在没收尾的 $( / <( 里的 bash 上时当数据剥掉：实测红的格（照失败行原文）
#       「主 agent $(bash <<EOF) 里起全量照拒」「主 agent <(bash <<EOF) 里起全量照拒」
#   LIB_SHELL_WORDS_BREAK=brace-empties-kept  花括号展开出来的空串留着（`{cargo,} test` 切成 `cargo '' test`）：实测红的格（照失败行原文）
#       「主 agent {cargo,} 空的一份起全量照拒」「主 agent {,} 写在命令词前面起全量照拒」「主 agent cargo {,test} 空的一份起全量照拒」
#   LIB_SHELL_WORDS_BREAK=brace-sequence-unpadded  补零的整数序列不补零（lib_project_code_runs.py 的 brace_sequence 共用这一处）：实测红的格（照失败行原文）
#       「主 agent cp 带步长、两端宽度不一的补零序列拷出再起全量脚本照拒」「主 agent cp 带步长、两端宽度不一的补零序列拷出再起全量脚本照拒：记几条检出」「主 agent cp 带步长、两端宽度不一的补零序列拷出再起全量脚本照拒：读进去几份脚本」「主 agent cp 补零的序列拷出再起全量脚本照拒」「主 agent cp 补零的序列拷出再起全量脚本照拒：记几条检出」「主 agent cp 补零的序列拷出再起全量脚本照拒：读进去几份脚本」「lib_project_code_runs.py 的自检」
#   LIB_SHELL_WORDS_BREAK=double-quoted-substitution-unclosed  双引号里的 `$(` 当没收尾，顶层的 bash <<EOF 的正文当数据剥掉：实测红的格（照失败行原文）
#       「主 agent cd "$(git rev-parse --show-toplevel)" 之后 bash <<EOF 里起全量照拒」「主 agent FOO="$(pwd)" bash <<EOF 里起全量照拒」
#   LIB_SHELL_WORDS_BREAK=function-calls-literal-only  函数名拆开拼写的调用认不出（只数原文里的字面）：实测红的格（照失败行原文）
#       「主 agent function 定义之后引号拆开拼写调用照拒」「主 agent function 定义之后花括号拆开拼写调用照拒」「主 agent function 定义之后反斜杠拆开拼写调用照拒」「编译目录：function 定义之后拆开拼写调用：拒（退出码 2）」「lib_workspace_cargo_lock.py 的自检」
#   WORKSPACE_CARGO_LOCK_BREAK=uncalled-functions-judged  编译目录那一道照判只定义、调不到的函数体：实测红的格（照失败行原文）
#       「编译目录：只定义、调不到的 function 函数体：放行（退出码 0）」「lib_workspace_cargo_lock.py 的自检」
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
#   LIB_SHELL_WORDS_BREAK=env-options-after-assignment  ：实测红的格（照失败行原文）
#       「env =1 之后的 -u 是命令词：env 报错不跑，放行」「lib_project_code_runs.py 的自检」
#   LIB_SHELL_WORDS_BREAK=brace-sequence-width-padded-ends-only  ：实测红的格（照失败行原文）
#       「主 agent cp 带步长、两端宽度不一的补零序列拷出再起全量脚本照拒」「主 agent cp 带步长、两端宽度不一的补零序列拷出再起全量脚本照拒：记几条检出」「主 agent cp 带步长、两端宽度不一的补零序列拷出再起全量脚本照拒：读进去几份脚本」「lib_project_code_runs.py 的自检」
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
#       「d137 H9：python -c 里 os.system 起全量，照拒」「d137 H9：python 读 heredoc 里 subprocess.run 起全量，照拒」「d137 H9：python -c 里 os.system 的命令是变量，认不出照拒」「d137 H9：perl -e 里 system 起全量，照拒」「d137 H9：ruby -e 里 system 起全量，照拒」「d137 H9：node -e 里 child_process 起全量，照拒」「d137 H9：tclsh 读 here-string 里 exec 起全量，照拒」「d137 H9：gawk 的 system() 起全量，照拒」「d137 H9：awk 的 print 管道到全量，照拒」「d137 H9：awk -f 程序文件里 system() 起全量，读正文照拒」「d137 H9：awk -f 程序文件里 system() 起全量，读正文照拒：读进去几份脚本」「d137 H9：awk -f 程序文件不起命令，放行：读进去几份脚本」「d137 H9：sed 的 e 命令起全量，照拒」「d137 H9：sed 的 s///e 执行模式空间，认不出照拒」「d137 H9：git -c 别名 ! 起全量，照拒」「d137 H9：git -c core.pager 起全量，照拒」「d137 H9：GIT_EXTERNAL_DIFF 起全量，照拒」「d137 H9：make 读 Makefile 配方起全量，照拒」「d137 H9：make 读 Makefile 配方起全量，照拒：读进去几份脚本」「d137 H9：make 读 Makefile 配方只有轻活，放行：读进去几份脚本」「d137 H9：make -f /dev/stdin 读 here-string 的配方起全量，照拒」「d137 H9：tar --to-command 起全量，照拒」「d137 H9：ssh -o ProxyCommand 起全量，照拒」「d137 H9：脚本正文里 perl -e 起全量，读正文照拒」「d137-r2 A：sed 算术地址后接 e 起全量，照拒」「d137-r2 A：sed 命令替换落在命令位上，照拒」「d137-r2 B：sed 文本之后另起一行 e 起全量，照拒」「d137-r2 C：python argv 起 cargo test 而参数是变量，照拒」「d137-r2 C：python argv 头一项是变量，照拒」「d137-r2 D：python 先赋变量的字面列表起全量，照拒」「d137-r2 D：python 列表赋过之后被改，认不出照拒」「d137-r2 E：awk 里 cmd 赋全量再 | getline，照拒」「d137-r2 E：awk 里 cmd 是拼接的，认不出照拒」「d137 闸误拒 E5：W 赋过字面目录，awk -f $W/程序文件，读正文放行：读进去几份脚本」「d137 闸误拒 E5：W 没赋过，awk -f $W/程序文件认不出，照拒」
#   PROJECT_CODE_GUARD_BREAK=program-files-unread  scan_uses() 不读别的程序从文件读的代码（awk -f、sed -f、Makefile：只判命令行上字面写着的）
#       「d137 H9：awk -f 程序文件里 system() 起全量，读正文照拒」「d137 H9：awk -f 程序文件里 system() 起全量，读正文照拒：读进去几份脚本」「d137 H9：awk -f 程序文件不起命令，放行：读进去几份脚本」「d137 H9：make 读 Makefile 配方起全量，照拒」「d137 H9：make 读 Makefile 配方起全量，照拒：读进去几份脚本」「d137 H9：make 读 Makefile 配方只有轻活，放行：读进去几份脚本」「d137 闸误拒 E5：W 赋过字面目录，awk -f $W/程序文件，读正文放行：读进去几份脚本」「d137 闸误拒 E5：W 没赋过，awk -f $W/程序文件认不出，照拒」
#   PROJECT_CODE_GUARD_BREAK=embedded-unjudged-allowed  别的程序里起命令而写法认不出的，命令文本这一层只记不拒
#       「d137 H9：python -c 里 os.system 的命令是变量，认不出照拒」「d137 H9：python -c 里 os.system 的命令是变量，认不出照拒：记几条检出」「d137 H9：sed 的 s///e 执行模式空间，认不出照拒」「d137 H9：sed 的 s///e 执行模式空间，认不出照拒：记几条检出」「d137-r2 A：sed 命令替换落在命令位上，照拒」「d137-r2 D：python 列表赋过之后被改，认不出照拒」「d137-r2 E：awk 里 cmd 是拼接的，认不出照拒」「d137 闸误拒 E5：W 没赋过，awk -f $W/程序文件认不出，照拒」
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
#   LIB_SHELL_WORDS_BREAK=env-ignore-environment-kept  ：红的格（推的，没跑：命令前的 BASH_ENV 留到 env -i 后面，bash 读进 startup-heavy.sh、判出全量）
#       「一百三十七：env -i 清掉 BASH_ENV 再起 bash：放行」「一百三十七：env -i 清掉 BASH_ENV 再起 bash：放行：读进去几份脚本」
#   LIB_SHELL_WORDS_BREAK=shell-standard-input-file-unread  ：实测红的格（照失败行原文）
#       「实现员 cat 文件经管道喂给 bash：放行并记检出」「一百三十七：bash -s < 文件 读那份文件判：记几条检出」「一百三十七：bash -s < 文件 读那份文件判：读进去几份脚本」「一百三十七：cat 文件 | bash 读那份文件判：记几条检出」「一百三十七：cat 文件 | bash 读那份文件判：读进去几份脚本」「一百三十七：sh -s < 轻脚本放行」「一百三十七：sh -s < 轻脚本放行：记几条检出」「一百三十七：sh -s < 轻脚本放行：读进去几份脚本」
#   LIB_SHELL_WORDS_BREAK=function-launchers-unseen  同 launcher-functions-unjudged 那一格：实测红的格（照失败行原文）
#       「一百三十七：脚本里经自己的 run_logged 起 --all 照拒」
#   LIB_SHELL_WORDS_BREAK=startup-files-unread  ：startup_file_words() 一个文件都不交，shell 启动时先读的文件（BASH_ENV / ENV、--rcfile、.bashrc、zsh 的）都不当被起的脚本；实测红的格（照失败行原文）
#       「一百三十七：BASH_ENV 写在命令前、指到字面路径的脚本，正文起全量，照拒」「一百三十七：BASH_ENV 写在命令前、指到字面路径的脚本，正文起全量，照拒：读进去几份脚本」「一百三十七：BASH_ENV 写相对路径，按这条命令的当前目录解，照拒」「一百三十七：BASH_ENV 写相对路径，按这条命令的当前目录解，照拒：读进去几份脚本」「一百三十七：env BASH_ENV=… 起 bash，照拒」「一百三十七：env BASH_ENV=… 起 bash，照拒：读进去几份脚本」「一百三十七：前面 export BASH_ENV=… 再起 bash，照拒」「一百三十七：前面 export BASH_ENV=… 再起 bash，照拒：读进去几份脚本」「一百三十七：BASH_ENV 指到的脚本正文起全量，bash 起的又是一份轻脚本，两份都读，照拒」「一百三十七：BASH_ENV 指到的脚本正文起全量，bash 起的又是一份轻脚本，两份都读，照拒：读进去几份脚本」「一百三十七：BASH_ENV 指到的脚本 source 自己目录下的库，库也读，照拒」「一百三十七：BASH_ENV 指到的脚本 source 自己目录下的库，库也读，照拒：读进去几份脚本」「一百三十七：BASH_ENV 指到的脚本正文起全量，实现员照拒」「一百三十七：BASH_ENV 指到的脚本正文起全量，实现员照拒：读进去几份脚本」「一百三十七：BASH_ENV 指到的脚本正文只有轻活，放行：读进去几份脚本」「一百三十七：BASH_ENV 写成前面赋过字面值的变量，展开之后读正文，照拒」「一百三十七：BASH_ENV 写成前面赋过字面值的变量，展开之后读正文，照拒：读进去几份脚本」「一百三十七：ENV 指到的脚本正文起全量，sh 起的，照拒」「一百三十七：ENV 指到的脚本正文起全量，sh 起的，照拒：读进去几份脚本」「一百三十七：ENV 指到的脚本正文起全量，bash --posix 起的，照拒」「一百三十七：ENV 指到的脚本正文起全量，bash --posix 起的，照拒：读进去几份脚本」「一百三十七：ENV 指到的脚本正文起全量，bash -o posix 起的，照拒」「一百三十七：ENV 指到的脚本正文起全量，bash -o posix 起的，照拒：读进去几份脚本」「一百三十七：BASH_ENV 写成命令替换拼出的路径，认不出，实现员照拒」「一百三十七：BASH_ENV 写成命令替换拼出的路径，认不出，主 agent 照拒」「一百三十七：BASH_ENV 写成没赋过值的变量，认不出，实现员照拒」「一百三十七：env 的 BASH_ENV 写成命令替换，认不出，实现员照拒」「一百三十七：ENV 写成命令替换，sh 起的，认不出，实现员照拒」「一百三十七：直接执行 #!/bin/bash 的脚本，BASH_ENV 指的文件照读，照拒」「一百三十七：直接执行 #!/bin/bash 的脚本，BASH_ENV 指的文件照读，照拒：读进去几份脚本」「一百三十七：直接执行 #!/usr/bin/env bash 的脚本，BASH_ENV 指的文件照读，照拒」「一百三十七：直接执行 #!/usr/bin/env bash 的脚本，BASH_ENV 指的文件照读，照拒：读进去几份脚本」「一百三十七：直接执行没有 #! 的 .sh，从 bash 里起是 bash 另起自己，BASH_ENV 指的文件照读，照拒」「一百三十七：直接执行没有 #! 的 .sh，从 bash 里起是 bash 另起自己，BASH_ENV 指的文件照读，照拒：读进去几份脚本」「一百三十七：env 写的 BASH_ENV，直接执行 #!/bin/bash 的脚本，照拒」「一百三十七：env 写的 BASH_ENV，直接执行 #!/bin/bash 的脚本，照拒：读进去几份脚本」「一百三十七：直接执行 #!/bin/sh 的脚本，ENV 指的文件照读，照拒」「一百三十七：直接执行 #!/bin/sh 的脚本，ENV 指的文件照读，照拒：读进去几份脚本」「一百三十七：直接执行 #!/bin/zsh 的脚本，ZDOTDIR 下的 .zshenv 照读，照拒」「一百三十七：直接执行 #!/bin/zsh 的脚本，ZDOTDIR 下的 .zshenv 照读，照拒：读进去几份脚本」「一百三十七：直接执行 #!/bin/bash 的脚本，BASH_ENV 指的文件只有轻活，放行：读进去几份脚本」「一百三十七：直接执行 #!/bin/bash 的脚本，BASH_ENV 写成命令替换拼出的路径，认不出，实现员照拒」「一百三十七：declare -x BASH_ENV=…，再起 bash，照拒」「一百三十七：declare -x BASH_ENV=…，再起 bash，照拒：读进去几份脚本」「一百三十七：typeset -x BASH_ENV=…，再起 bash，照拒」「一百三十七：typeset -x BASH_ENV=…，再起 bash，照拒：读进去几份脚本」「一百三十七：先 BASH_ENV=… 赋值、后 export BASH_ENV，再起 bash，照拒」「一百三十七：先 BASH_ENV=… 赋值、后 export BASH_ENV，再起 bash，照拒：读进去几份脚本」「一百三十七：先 export BASH_ENV、后赋值，再起 bash，照拒」「一百三十七：先 export BASH_ENV、后赋值，再起 bash，照拒：读进去几份脚本」「一百三十七：export 过之后换成另一份脚本，起 bash 读的是后一份，照拒」「一百三十七：export 过之后换成另一份脚本，起 bash 读的是后一份，照拒：读进去几份脚本」「一百三十七：set -a 之后赋值的 BASH_ENV 进环境，照拒」「一百三十七：set -a 之后赋值的 BASH_ENV 进环境，照拒：读进去几份脚本」「一百三十七：local -x BASH_ENV=… 的函数里起 bash，照拒」「一百三十七：local -x BASH_ENV=… 的函数里起 bash，照拒：读进去几份脚本」「一百三十七：declare -x BASH_ENV 写成命令替换拼出的路径，认不出，实现员照拒」「一百三十七：先赋值后 export 的 BASH_ENV 是命令替换拼出的路径，认不出，实现员照拒」「一百三十七：bash -i 读 HOME 下的 .bashrc，里面起全量，照拒」「一百三十七：bash -i 读 HOME 下的 .bashrc，里面起全量，照拒：读进去几份脚本」「一百三十七：bash -ic 合写的 -i 同样读 HOME 下的 .bashrc，照拒」「一百三十七：bash -ic 合写的 -i 同样读 HOME 下的 .bashrc，照拒：读进去几份脚本」「一百三十七：bash --rcfile 文件 -i，文件里起全量，照拒」「一百三十七：bash --rcfile 文件 -i，文件里起全量，照拒：读进去几份脚本」「一百三十七：bash --init-file 文件 -i，文件里起全量，照拒」「一百三十七：bash --init-file 文件 -i，文件里起全量，照拒：读进去几份脚本」「一百三十七：bash -i 读 HOME 下只有轻活的 .bashrc：放行：读进去几份脚本」「一百三十七：bash -i 不读 BASH_ENV，.bashrc 只有轻活：放行：读进去几份脚本」「一百三十七：bash -i 不读 ENV：放行：读进去几份脚本」「一百三十七：sh -i 读 ENV 指的文件，照拒」「一百三十七：sh -i 读 ENV 指的文件，照拒：读进去几份脚本」「一百三十七：sh -i 读 ENV 指的文件，只有轻活：放行：读进去几份脚本」「一百三十七：bash --posix -i 读 ENV 指的文件，照拒」「一百三十七：bash --posix -i 读 ENV 指的文件，照拒：读进去几份脚本」「一百三十七：bash --rcfile 写成命令替换拼出的路径，认不出，实现员照拒」「一百三十七：bash -i 的 HOME 写成命令替换，.bashrc 路径认不出，实现员照拒」「一百三十七：zsh 读 ZDOTDIR 下的 .zshenv，里面起全量，照拒」「一百三十七：zsh 读 ZDOTDIR 下的 .zshenv，里面起全量，照拒：读进去几份脚本」「一百三十七：zsh 读 ZDOTDIR 下的 .zshrc，里面起全量，照拒」「一百三十七：zsh 读 ZDOTDIR 下的 .zshrc，里面起全量，照拒：读进去几份脚本」「一百三十七：zsh 没设 ZDOTDIR 取 HOME 下的 .zshenv，照拒」「一百三十七：zsh 没设 ZDOTDIR 取 HOME 下的 .zshenv，照拒：读进去几份脚本」「一百三十七：前面 export 的 ZDOTDIR，zsh 照读，照拒」「一百三十七：前面 export 的 ZDOTDIR，zsh 照读，照拒：读进去几份脚本」「一百三十七：zsh 的启动文件只有轻活：放行：读进去几份脚本」「一百三十七：zsh 的 ZDOTDIR 写成命令替换，认不出，实现员照拒」
#   LIB_SHELL_WORDS_BREAK=declared-exports-unseen  ：环境里的变量只认 `export 名字=值`（declare -x、typeset -x、local -x、先赋值后 export、先 export 后赋值、export -n、declare +x、set -a 都不认）；实测红的格（照失败行原文）
#       「一百三十七：declare -x BASH_ENV=…，再起 bash，照拒」「一百三十七：declare -x BASH_ENV=…，再起 bash，照拒：读进去几份脚本」「一百三十七：typeset -x BASH_ENV=…，再起 bash，照拒」「一百三十七：typeset -x BASH_ENV=…，再起 bash，照拒：读进去几份脚本」「一百三十七：先 BASH_ENV=… 赋值、后 export BASH_ENV，再起 bash，照拒」「一百三十七：先 BASH_ENV=… 赋值、后 export BASH_ENV，再起 bash，照拒：读进去几份脚本」「一百三十七：先 export BASH_ENV、后赋值，再起 bash，照拒」「一百三十七：先 export BASH_ENV、后赋值，再起 bash，照拒：读进去几份脚本」「一百三十七：export 过之后换成另一份脚本，起 bash 读的是后一份，照拒」「一百三十七：set -a 之后赋值的 BASH_ENV 进环境，照拒」「一百三十七：set -a 之后赋值的 BASH_ENV 进环境，照拒：读进去几份脚本」「一百三十七：local -x BASH_ENV=… 的函数里起 bash，照拒」「一百三十七：local -x BASH_ENV=… 的函数里起 bash，照拒：读进去几份脚本」「一百三十七：export -n 撤掉 export 之后起 bash：放行」「一百三十七：export -n 撤掉 export 之后起 bash：放行：读进去几份脚本」「一百三十七：export -n 带赋值，不进环境：放行」「一百三十七：export -n 带赋值，不进环境：放行：读进去几份脚本」「一百三十七：declare +x 撤掉 export 之后起 bash：放行」「一百三十七：declare +x 撤掉 export 之后起 bash：放行：读进去几份脚本」「一百三十七：declare -x BASH_ENV 写成命令替换拼出的路径，认不出，实现员照拒」「一百三十七：先赋值后 export 的 BASH_ENV 是命令替换拼出的路径，认不出，实现员照拒」
#   LIB_SHELL_WORDS_BREAK=interactive-startup-files-unread  ：带 -i 不另看（交互的 bash 照旧读 BASH_ENV、不读 .bashrc 与 --rcfile / --init-file）；实测红的格（照失败行原文）
#       「一百三十七：bash -i 读 HOME 下的 .bashrc，里面起全量，照拒」「一百三十七：bash -i 读 HOME 下的 .bashrc，里面起全量，照拒：读进去几份脚本」「一百三十七：bash -ic 合写的 -i 同样读 HOME 下的 .bashrc，照拒」「一百三十七：bash -ic 合写的 -i 同样读 HOME 下的 .bashrc，照拒：读进去几份脚本」「一百三十七：bash --rcfile 文件 -i，文件里起全量，照拒」「一百三十七：bash --rcfile 文件 -i，文件里起全量，照拒：读进去几份脚本」「一百三十七：bash --init-file 文件 -i，文件里起全量，照拒」「一百三十七：bash --init-file 文件 -i，文件里起全量，照拒：读进去几份脚本」「一百三十七：bash -i 读 HOME 下只有轻活的 .bashrc：放行：读进去几份脚本」「一百三十七：bash -i 不读 BASH_ENV，.bashrc 只有轻活：放行」「一百三十七：bash -i 不读 ENV：放行：读进去几份脚本」「一百三十七：bash --rcfile 写成命令替换拼出的路径，认不出，实现员照拒」「一百三十七：bash -i 的 HOME 写成命令替换，.bashrc 路径认不出，实现员照拒」
#   LIB_SHELL_WORDS_BREAK=zsh-startup-files-unread  ：zsh 的 .zshenv、.zshrc 不读；实测红的格（照失败行原文）
#       「一百三十七：直接执行 #!/bin/zsh 的脚本，ZDOTDIR 下的 .zshenv 照读，照拒」「一百三十七：直接执行 #!/bin/zsh 的脚本，ZDOTDIR 下的 .zshenv 照读，照拒：读进去几份脚本」「一百三十七：zsh 读 ZDOTDIR 下的 .zshenv，里面起全量，照拒」「一百三十七：zsh 读 ZDOTDIR 下的 .zshenv，里面起全量，照拒：读进去几份脚本」「一百三十七：zsh 读 ZDOTDIR 下的 .zshrc，里面起全量，照拒」「一百三十七：zsh 读 ZDOTDIR 下的 .zshrc，里面起全量，照拒：读进去几份脚本」「一百三十七：zsh 没设 ZDOTDIR 取 HOME 下的 .zshenv，照拒」「一百三十七：zsh 没设 ZDOTDIR 取 HOME 下的 .zshenv，照拒：读进去几份脚本」「一百三十七：前面 export 的 ZDOTDIR，zsh 照读，照拒」「一百三十七：前面 export 的 ZDOTDIR，zsh 照读，照拒：读进去几份脚本」「一百三十七：zsh 的启动文件只有轻活：放行：读进去几份脚本」「一百三十七：zsh 的 ZDOTDIR 写成命令替换，认不出，实现员照拒」
#   LIB_SHELL_WORDS_BREAK=executed-script-startup-unread  ：直接执行的脚本起来时它的 shell 先读的启动文件不读，#! 指到认不出的解释器的也不判；实测红的格（照失败行原文）
#       「一百三十七：直接执行 #!/bin/bash 的脚本，BASH_ENV 指的文件照读，照拒」「一百三十七：直接执行 #!/bin/bash 的脚本，BASH_ENV 指的文件照读，照拒：读进去几份脚本」「一百三十七：直接执行 #!/usr/bin/env bash 的脚本，BASH_ENV 指的文件照读，照拒」「一百三十七：直接执行 #!/usr/bin/env bash 的脚本，BASH_ENV 指的文件照读，照拒：读进去几份脚本」「一百三十七：直接执行没有 #! 的 .sh，从 bash 里起是 bash 另起自己，BASH_ENV 指的文件照读，照拒」「一百三十七：直接执行没有 #! 的 .sh，从 bash 里起是 bash 另起自己，BASH_ENV 指的文件照读，照拒：读进去几份脚本」「一百三十七：env 写的 BASH_ENV，直接执行 #!/bin/bash 的脚本，照拒」「一百三十七：env 写的 BASH_ENV，直接执行 #!/bin/bash 的脚本，照拒：读进去几份脚本」「一百三十七：直接执行 #!/bin/sh 的脚本，ENV 指的文件照读，照拒」「一百三十七：直接执行 #!/bin/sh 的脚本，ENV 指的文件照读，照拒：读进去几份脚本」「一百三十七：直接执行 #!/bin/zsh 的脚本，ZDOTDIR 下的 .zshenv 照读，照拒」「一百三十七：直接执行 #!/bin/zsh 的脚本，ZDOTDIR 下的 .zshenv 照读，照拒：读进去几份脚本」「一百三十七：直接执行 #!/bin/bash 的脚本，BASH_ENV 指的文件只有轻活，放行：读进去几份脚本」「一百三十七：直接执行 #! 指到认不出的解释器、环境里设了 BASH_ENV，实现员照拒」「一百三十七：直接执行 #!/bin/bash 的脚本，BASH_ENV 写成命令替换拼出的路径，认不出，实现员照拒」
#   LIB_PROJECT_CODE_RUNS_BREAK=unrecognized-launches-allowed  ：实测红的格（照失败行原文）
#       「实现员经内存包装起循环里 ls 出来的 验证包测试二进制：命令词认不出照拒」「一百三十七：bash <(…) 照拒」「一百三十七：eval 变量里拼的命令照拒」「一百三十七：source <(…) 照拒」「一百三十七：命令词是 $(which cargo) 照拒」「一百三十七：for 循环变量当脚本路径照拒」「一百三十七：xargs bash 起读进来的脚本照拒」「一百三十七：BASH_ENV 写成命令替换拼出的路径，认不出，实现员照拒」「一百三十七：BASH_ENV 写成命令替换拼出的路径，认不出，主 agent 照拒」「一百三十七：BASH_ENV 写成没赋过值的变量，认不出，实现员照拒」「一百三十七：env 的 BASH_ENV 写成命令替换，认不出，实现员照拒」「一百三十七：ENV 写成命令替换，sh 起的，认不出，实现员照拒」「一百三十七：直接执行 #!/bin/bash 的脚本，BASH_ENV 写成命令替换拼出的路径，认不出，实现员照拒」「一百三十七：declare -x BASH_ENV 写成命令替换拼出的路径，认不出，实现员照拒」「一百三十七：先赋值后 export 的 BASH_ENV 是命令替换拼出的路径，认不出，实现员照拒」「一百三十七：bash --rcfile 写成命令替换拼出的路径，认不出，实现员照拒」「一百三十七：bash -i 的 HOME 写成命令替换，.bashrc 路径认不出，实现员照拒」「一百三十七：zsh 的 ZDOTDIR 写成命令替换，认不出，实现员照拒」「lib_project_code_runs.py 的自检」「d137 闸误拒 E3：$t 没赋过，cat > $t 写出再 bash $t，照拒」
#   LIB_PROJECT_CODE_RUNS_BREAK=cargo-arguments-unexpanded  ：实测红的格（照失败行原文）
#       「一百三十七：cargo 参数写成前面赋过值的变量，展开之后照判（--lib 放行）」「lib_project_code_runs.py 的自检」
#   LIB_PROJECT_CODE_RUNS_BREAK=extra-launchers-unseen  ：实测红的格（照失败行原文）
#       「一百三十七：xargs bash 起读进来的脚本照拒」「lib_project_code_runs.py 的自检」「d137 闸误拒 E2：ssh localhost 包着 bash 轻脚本，放行：读进去几份脚本」「d137 闸误拒 E2：numactl -N0 包着 bash 轻脚本，放行：读进去几份脚本」「d137 闸误拒 E2：unshare --user 包着 bash 轻脚本，放行：读进去几份脚本」
#   LIB_PROJECT_CODE_RUNS_BREAK=sourced-libraries-unread  ：实测红的格（照失败行原文）
#       「实现员起的脚本里命令替换、反引号、算术后面接路径：片段不当路径：记几条检出」「一百三十七：脚本 source 自己目录下的库，读正文时库也读」「一百三十七：脚本 source 自己目录下的库，读正文时库也读：读进去几份脚本」「lib_project_code_runs.py 的自检」
#   LIB_PROJECT_CODE_RUNS_BREAK=script-body-launches-unjudged  ：实测红的格（照失败行原文）
#       「一百三十七：脚本正文里经数组变量起拼出路径的钩子自证（件 1 那三趟的形态），照拒」「一百三十七：同一份脚本主 agent 照拒」「lib_project_code_runs.py 的自检」
#   LIB_PROJECT_CODE_RUNS_BREAK=dispatch-launchers-unseen  （multi-host-run.sh --peer … -- 不剥）：红的格（multi-host-run.sh --peer 那几格推的、没复跑）
#       「一百三十七#8：工具实现员经 multi-host-run.sh --peer 起门禁格，照拒」「lib_project_code_runs.py 的自检」「编译目录：子 agent 经 multi-host-run.sh --peer 在主工作区起 cargo test、没给编译目录：拒（退出码 2）」
#   PROJECT_CODE_GUARD_BREAK=launcher-recorded-as-writer  （剥掉之后里面那条接着判的启动器整条记成写者）：实测红的格（照失败行原文）
#       「d137 闸误拒 E2：systemd-run --user --wait 包着 bash 轻脚本，放行」「d137 闸误拒 E2：systemd-run --user --wait 包着 bash 轻脚本，放行：读进去几份脚本」「d137 闸误拒 E2：chrt -i 0 包着 bash 轻脚本，放行」「d137 闸误拒 E2：ssh localhost 包着 bash 轻脚本，放行」「d137 闸误拒 E2：/usr/bin/time 包着 bash 轻脚本，放行」「d137 闸误拒 E2：strace -o /dev/null 包着 bash 轻脚本，放行」「d137 闸误拒 E2：prlimit --as 包着 bash 轻脚本，放行」「d137 闸误拒 E2：numactl -N0 包着 bash 轻脚本，放行」「d137 闸误拒 E2：unshare --user 包着 bash 轻脚本，放行」「d137 闸误拒 E2：systemd-run --user --wait 包着 bash 重脚本，读正文照拒：读进去几份脚本」
#   PROJECT_CODE_GUARD_BREAK=heredoc-targets-unexpanded  （heredoc 写出的目标不代入前面赋过字面值的变量）：实测红的格（照失败行原文）
#       「d137 闸误拒 E3：t 赋过字面路径，cat > $t 读 heredoc 写出再 bash $t，放行」
#   PROJECT_CODE_GUARD_BREAK=program-file-variables-unexpanded  （awk -f、sed -f、Makefile 的路径不代入前面赋过字面值的变量）：实测红的格（照失败行原文）
#       「d137 闸误拒 E5：W 赋过字面目录，awk -f $W/程序文件，读正文放行」「d137 闸误拒 E5：W 赋过字面目录，awk -f $W/程序文件，读正文放行：读进去几份脚本」
#   LIB_PROJECT_CODE_RUNS_BREAK=claude-executable-path-unrecognized  （命令词 "$CLAUDE_CODE_EXECPATH" 照变量拼出的认不出）：实测红的格（照失败行原文）
#       「lib_project_code_runs.py 的自检」
#   LIB_SHELL_WORDS_BREAK=aliases-same-line-expanded  （同一行里先定义再用的别名照展开）：实测红的格（照失败行原文）
#       「d137 闸误拒 E4：同一行先 alias 再用，这一行不展开别名、放行」
#   LIB_PROJECT_CODE_RUNS_BREAK=build-subcommands-unjudged  lib_project_code_runs.py 的 starts_a_build 不认会编的 cargo：红的格（推的，没跑）
#       「lib_project_code_runs.py 的自检」
#   LIB_PROJECT_CODE_RUNS_BREAK=build-scripts-unjudged  starts_a_build 不认自己在里面起 cargo 的那几份脚本：红的格（推的，没跑）
#       「lib_project_code_runs.py 的自检」
#   PROJECT_CODE_GUARD_BREAK=task-server-direct-allowed  task_server_decision 不判 ① 的直接跑（编译、编出来的二进制、本仓的代码、剥掉的包装、那张表认出的每一样），只照「谁能跑哪一类」判：红的格（推的）
#       judge_task_cases 那二十一格（实现员在仓根裸跑 cargo test、实现员交回前自查命名那四格、书记员跑 doc-decisions、主 agent 跑 gate-lint.sh 这一批：应当是 2，实际 0）
#       「经 task-server：照抄：实现员直接跑 harness 测试，照抄的那条经 task-server 提交」「经 task-server：照抄：经内存包装直接起的去掉包装」「经 task-server：照抄：带 && 的整条包进 bash -c」「stdin:实现员直接跑测试目标拒绝（退出码 2）」「stdin:子 agent 不经 task-server 的拒绝 stderr 写了照抄的提交、写进报告「要主 agent 跑的命令」「stdin:主 agent 直接跑被拒时最前面是能照抄的经 task-server 提交的那条（照抄的那条再判放行）」「stdin:主 agent 被拒时不贴「谁能跑哪一类」「stdin:主 agent 直接跑整道门禁被拒时照抄的那条经 task-server 提交、四段范围声明写在 submit 前面（为什么是占位），照它再判放行」
#   PROJECT_CODE_GUARD_BREAK=subagent-task-client-allowed  ② 不判：子 agent 起 task-server/task 的 submit、wait、cancel、clean 照放（子 agent 自己交；
#       「经 task-server：子 agent 交任务的拒绝说明写明写进报告「要主 agent 跑的命令」
#   PROJECT_CODE_GUARD_BREAK=task-server-bypass-allowed  ③ 不判（scan_uses 不记、命令文本这一层不看）：红的格（推的；直接起服务本体与每次尝试的那一层另由 ① 照拒，不红）
#       「经 task-server：绕开：读签名密钥照拒，说明点名绕开写法」「lib_project_code_runs.py 的自检」
#   LIB_PROJECT_CODE_RUNS_BREAK=task-server-builds-unjudged  不认直接删服务的编译目录：钩子这一侧红的格（推的）
#       「lib_project_code_runs.py 的自检」
#   PROJECT_CODE_GUARD_BREAK=task-server-copy-dropped  拒绝说明不给照抄的那一条（task_server_copy_lines 交空）：红的格（推的）
#       「经 task-server：照抄：实现员直接跑 harness 测试，照抄的那条经 task-server 提交」「经 task-server：照抄：经内存包装直接起的去掉包装」「经 task-server：照抄：带 && 的整条包进 bash -c」「stdin:子 agent 不经 task-server 的拒绝 stderr 写了照抄的提交、写进报告「要主 agent 跑的命令」「stdin:主 agent 直接跑被拒时最前面是能照抄的经 task-server 提交的那条（照抄的那条再判放行）」「stdin:主 agent 直接跑整道门禁被拒时照抄的那条经 task-server 提交、四段范围声明写在 submit 前面（为什么是占位），照它再判放行」
#   PROJECT_CODE_GUARD_BREAK=temporary-service-bare-allowed  ④ 不判（scan_uses 不记要临时服务的自证写成裸 --selftest）：红的格（推的）
#       「经 task-server：要临时服务的自证裸写被拒时，说明点名登记表、给整条包装写法」「经 task-server：子 agent 直接裸写要临时服务的自证被拒时，说明写进报告「要主 agent 跑的命令」「stdin:主 agent 经 task-server 提交要临时服务的自证裸写 --selftest 拒绝（退出码 2），stderr 给包装写法的整条」
#   PROJECT_CODE_GUARD_BREAK=temporary-service-wrapper-not-marked  剥开临时服务的包装重判里面那条时不补标记（包在包装里的要临时服务的自证被当成裸写拦下）：红的格（推的）
#       「经 task-server：主 agent 提交包着临时服务（给了 --server-binary）的要临时服务的自证：放行」
#   PROJECT_CODE_GUARD_BREAK=temporary-service-submit-inner-unread  submit 里那一条的要临时服务的自证裸写不收：红的格（推的）
#       「经 task-server：要临时服务的自证裸写被拒时，说明点名登记表、给整条包装写法」「stdin:主 agent 经 task-server 提交要临时服务的自证裸写 --selftest 拒绝（退出码 2），stderr 给包装写法的整条」
#   PROJECT_CODE_GUARD_BREAK=temporary-service-remedy-dropped  ④ 的说明只写哪一份、不贴规矩与怎么办（TEMPORARY_SERVICE_POLICY 不贴）：红的格（推的）
#       「经 task-server：要临时服务的自证裸写被拒时，说明点名登记表、给整条包装写法」「经 task-server：子 agent 直接裸写要临时服务的自证被拒时，说明写进报告「要主 agent 跑的命令」
#   LIB_PROJECT_CODE_RUNS_BREAK=temporary-service-bare-unjudged  lib_project_code_runs.py 的 temporary_service_bare_selftest 一律交 None：钩子这一侧红的格（推的）
#       「经 task-server：要临时服务的自证裸写被拒时，说明点名登记表、给整条包装写法」「经 task-server：子 agent 直接裸写要临时服务的自证被拒时，说明写进报告「要主 agent 跑的命令」「stdin:主 agent 经 task-server 提交要临时服务的自证裸写 --selftest 拒绝（退出码 2），stderr 给包装写法的整条」
#   LIB_PROJECT_CODE_RUNS_BREAK=temporary-service-marker-ignored  不看环境里的临时服务标记（套了包装、自己设了标记的也拦）：钩子这一侧红的格（推的）
#       「经 task-server：主 agent 提交包着临时服务（给了 --server-binary）的要临时服务的自证：放行」「lib_project_code_runs.py 的自检」
#   LIB_PROJECT_CODE_RUNS_BREAK=temporary-service-environment-dropped  environment_under_temporary_service 原样交回（包装里的原命令看起来没有标记）：钩子这一侧红的格（推的）
#       「经 task-server：主 agent 提交包着临时服务（给了 --server-binary）的要临时服务的自证：放行」「lib_project_code_runs.py 的自检」
#   LIB_PROJECT_CODE_RUNS_BREAK=temporary-service-own-server-as-required  自己起服务的自证也被要求套临时服务：钩子这一侧红的格（推的）
#       「经 task-server：主 agent 提交自己起服务的自证（own-server）裸写：放行」「lib_project_code_runs.py 的自检」
#   LIB_PROJECT_CODE_RUNS_BREAK=temporary-service-command-bare  说明里给的整条写法是裸写的样子：钩子这一侧红的格（推的）
#       「经 task-server：要临时服务的自证裸写被拒时，说明点名登记表、给整条包装写法」「经 task-server：子 agent 直接裸写要临时服务的自证被拒时，说明写进报告「要主 agent 跑的命令」「stdin:主 agent 经 task-server 提交要临时服务的自证裸写 --selftest 拒绝（退出码 2），stderr 给包装写法的整条」
#   LIB_PROJECT_CODE_RUNS_BREAK=temporary-service-command-binary-placeholder  说明里给的整条写法带 --server-binary 与尖括号占位：钩子这一侧红的格
#       「经 task-server：要临时服务的自证裸写被拒时，说明点名登记表、给整条包装写法」「经 task-server：子 agent 直接裸写要临时服务的自证被拒时，说明写进报告「要主 agent 跑的命令」「stdin:主 agent 经 task-server 提交要临时服务的自证裸写 --selftest 拒绝（退出码 2），stderr 给包装写法的整条」「lib_project_code_runs.py 的自检」
#   LIB_PROJECT_CODE_RUNS_BREAK=temporary-service-table-lenient  要临时服务的表写坏的行照收、不报：钩子这一侧红的格（推的，只在 lib_project_code_runs.py 的自检里）
#       「lib_project_code_runs.py 的自检」
#   LIB_PROJECT_CODE_RUNS_BREAK=project-scripts-unjudged  lib_project_code_runs.py 的 project_code_use 一律交 None：钩子这一侧红的格（推的；同一条里另有编译、包装、那张表认出的一样的照拒，不红）
#       「lib_project_code_runs.py 的自检」
#   LIB_PROJECT_CODE_RUNS_BREAK=text-tools-refused  TEXT_TOOLS_RUN_DIRECTLY 不认：钩子这一侧红的格（推的）
#       「lib_project_code_runs.py 的自检」
#   LIB_PROJECT_CODE_RUNS_BREAK=main-agent-tools-refused  MAIN_AGENT_TOOLS_RUN_DIRECTLY 不认：钩子这一侧红的格（推的）
#       「lib_project_code_runs.py 的自检」
#   LIB_PROJECT_CODE_RUNS_BREAK=task-server-submission-unparsed  task_server_submission 一律交 None（submit 那一条只当名单上的客户端放行）：钩子这一侧红的格（推的）
#       「lib_project_code_runs.py 的自检」「stdin:主 agent 直接跑整道门禁被拒时照抄的那条经 task-server 提交、四段范围声明写在 submit 前面」「经 task-server：要临时服务的自证裸写被拒时，说明点名登记表、给整条包装写法」「stdin:主 agent 经 task-server 提交要临时服务的自证裸写 --selftest 拒绝（退出码 2），stderr 给包装写法的整条」
#   LIB_PROJECT_CODE_RUNS_BREAK=task-server-paths-unjudged  不认库、密钥、socket 与载入共用库：钩子这一侧红的格（推的）
#       「lib_project_code_runs.py 的自检」
#   LIB_PROJECT_CODE_RUNS_BREAK=admitted-variable-unjudged  不认「已由 task-server 准入」开关：钩子这一侧红的格（推的）
#       「lib_project_code_runs.py 的自检」
#   LIB_PROJECT_CODE_RUNS_BREAK=task-server-scripts-unjudged  不认直接起服务本体与每次尝试的那一层：钩子这一侧红的格（推的；直接起的另由 ① 照拒）
#       「lib_project_code_runs.py 的自检」
#   LIB_PROJECT_CODE_RUNS_BREAK=task-server-units-unjudged  不认 systemctl 停禁改它的单元：钩子这一侧红的格（推的）
#       「lib_project_code_runs.py 的自检」
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
import glob, importlib.util, json, os, re, shlex, shutil, subprocess, sys, tempfile, time
from datetime import datetime, timezone
from typing import NamedTuple

def project_code_runs_library_path(hook_dir):
    return os.path.join(hook_dir, "lib_project_code_runs.py")

def load_project_code_runs(library_path):
    """判定的共用模块（它再导入同目录的 lib_shell_words.py）；读不到或导入出错就抛异常，由调用方处理。"""
    spec = importlib.util.spec_from_file_location("lib_project_code_runs", library_path)
    library = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(library)
    return library

try:
    project_code_runs = load_project_code_runs(project_code_runs_library_path(sys.argv[1]))
    shell_words, shared_library_error = project_code_runs.shell_words, None
except Exception as error:  # 文件不在、语法错、导入时抛的都算读不到
    project_code_runs, shell_words, shared_library_error = None, None, error

def shared_library_problem(hook_dir):
    return f"读不到共用模块 {project_code_runs_library_path(hook_dir)}（它再导入同目录的 lib_shell_words.py）：{shared_library_error!r}"

try:   # 检出记录的默认位置在工作根下，工作根只在同目录的 lib_work_root.py 里定义（读不到时检出记录不写）
    sys.dont_write_bytecode = True
    sys.path.insert(0, sys.argv[1]); import lib_work_root as work_roots
    DEFAULT_DETECTIONS = work_roots.default_detections_path()
except Exception:  # 文件不在、语法错、弄坏开关写错都算读不到
    work_roots, DEFAULT_DETECTIONS = None, None
try:   # 主工作区里直接起、编译目录会落进主仓的 cargo 的判法在同目录的 lib_workspace_cargo_lock.py（读不到时这一道不判，命令里有 cargo 的记一条检出）
    cargo_lock_spec = importlib.util.spec_from_file_location("lib_workspace_cargo_lock", os.path.join(sys.argv[1], "lib_workspace_cargo_lock.py"))
    cargo_lock = importlib.util.module_from_spec(cargo_lock_spec)
    cargo_lock_spec.loader.exec_module(cargo_lock)
    cargo_lock_error = None
except Exception as error:   # 文件不在、语法错都算读不到
    cargo_lock, cargo_lock_error = None, error
MAXIMUM_SCRIPT_DEPTH = 6        # 从命令文本起一层层读进去的脚本最多几层；再往里判「看不全」
MAXIMUM_SCRIPT_BYTES = 1 << 20  # 脚本超过这么大不读，判「读不到」
# 直接执行的文件先读开头这么多判是不是 shell 脚本（ELF 魔数、NUL 字节、`#!` 指到别的解释器、没有 `#!` 而扩展名不是 .sh），不是就不读也不记，之后才判大小上限：
# 编译产物常有几 MB，先判大小会把它记成「超过上限、没判」（例：scripts/experiment-apparatus/target/release/e49_chain_width 5124256 字节）
SCRIPT_HEAD_BYTES = 4096
SHELL_SCRIPT_EXTENSION = ".sh"  # 直接执行、没有 `#!` 的文件，只有这个扩展名的当 shell 脚本读

# 弄坏开关 PROJECT_CODE_GUARD_BREAK=<项>（几个用逗号连；只给自检证明点名的几格会红，项名与红的格写在文件头）
GUARD_BREAK_VARIABLE = "PROJECT_CODE_GUARD_BREAK"
KNOWN_GUARD_BREAK_ITEMS = ("disable-check", "ignore-written-scripts",
                           "detection-outcome-dropped", "copy-without-run-scope",
                           "failure-unrecorded", "copy-by-cp-only", "heredoc-write-every-redirection", "cat-concatenation-unseen",
                           "unrecognized-writes-allowed", "path-scripts-unread", "copy-redirections-unseen", "tee-copy-unseen", "launcher-functions-unjudged",
                           "glob-scripts-unread", "wrapper-copies-trusted", "cat-missing-source-unexplained", "bash-layer-unrecorded", "unjudged-shell-input-allowed",
                           "cat-append-unreadable-destination-raises", "relative-script-undeterminable-directory-allowed",
                           "bash-env-forwarding-untracked", "copied-shell-binary-untracked", "compound-stdin-untracked", "cat-missing-source-allowed",
                           "python-missing-unrecorded", "program-files-unread", "embedded-unjudged-allowed",
                           "launcher-recorded-as-writer", "heredoc-targets-unexpanded", "program-file-variables-unexpanded",
                           "task-server-direct-allowed", "subagent-task-client-allowed", "task-server-bypass-allowed", "task-server-copy-dropped",
                           "temporary-service-bare-allowed", "temporary-service-wrapper-not-marked", "temporary-service-submit-inner-unread", "temporary-service-remedy-dropped",
                           "python-filter-code-judged")
def guard_break_is_set(item):
    return item in os.environ.get(GUARD_BREAK_VARIABLE, "").split(",")

class UseAt(NamedTuple):
    test: object           # lib_project_code_runs.ProgramUse
    frames: tuple          # 读到它经过的脚本：((脚本路径, 行号或 None), …)，最后一格是它自己所在的那一行；命令文本里直接写的是 ()

class Notice(NamedTuple):
    frames: tuple          # 同 UseAt.frames：出在哪个脚本的哪一行，命令文本里直接写的是 ()
    text: str              # 读不到、看不全的是什么

class Scan(NamedTuple):
    uses: list             # [UseAt]
    notices: list          # [Notice]：读不到、看不全的脚本，这条命令照常执行，记检出
    scripts_read: list     # 读进去判过的脚本路径，判一遍记一次，复用上一遍的结果不记（放行时分得出「读了、里面没有」与「根本没读」）
    hit_depth_limit: bool  # 这一层或更里面有脚本因为嵌套超过上限没读：结果跟着从第几层起走
    lowest_cycle_frame: int | None  # 这一层或更里面绕回外层脚本而没读时，绕回的那一格在帧里的最小下标；没绕回是 None
    direct_runs: list = [] # [DirectRun]：不经 task-server 跑项目代码的每一处（文件头「经 task-server 起」①），与子 agent 不起的那几个客户端子命令（②）
    bypasses: list = []    # [Bypass]：绕开 task-server 的写法（文件头「经 task-server 起」③）
    listed_scripts: list = []  # [(真实路径, "anyone" 或 "main-agent")]：起过的名单上直接跑的脚本（读它的正文时里面的那几行不另算不经 task-server）

# DirectRun.allowed 的一种：子 agent 起提交客户端的 submit、wait、cancel、clean（文件头「经 task-server 起」②；lib_project_code_runs.subagent_task_server_refusal），只给主 agent
SUBAGENT_CLIENT_LEVEL = "main-agent-client"
# DirectRun.allowed 的另一种：要临时 prune-server 的自证写成了裸的 --selftest（文件头「经 task-server 起」④；lib_project_code_runs.temporary_service_bare_selftest），谁起都拒
TEMPORARY_SERVICE_BARE_LEVEL = "temporary-service-bare"

class DirectRun(NamedTuple):
    detail: str            # lib_project_code_runs.runs_compiled_code、starts_a_build 的说明，project_code_use 认出的那一份，剥掉的内存包装，或子 agent 不起的客户端子命令
    frames: tuple          # 同 UseAt.frames
    allowed: str = "nobody"  # "main-agent"：这一处是只给主 agent 直接跑的会话工具（lib_project_code_runs.MAIN_AGENT_TOOLS_RUN_DIRECTLY）；SUBAGENT_CLIENT_LEVEL：见上

class Bypass(NamedTuple):
    detail: str            # lib_project_code_runs.task_server_text_bypass / task_server_command_bypass 的说明
    frames: tuple          # 同 UseAt.frames

class WrittenScript(NamedTuple):
    text: str | None          # heredoc 写出的正文；cp 拷来的是 None
    copied_from: str | None   # cp 拷来的：源文件的绝对路径；heredoc 写出的是 None

class FileOnDisk(NamedTuple):
    head: bytes            # 开头 SCRIPT_HEAD_BYTES 字节（判是不是 shell 脚本用）；读不了是 b""
    data: bytes | None     # 整份；超过 MAXIMUM_SCRIPT_BYTES 或读不了是 None
    size: int
    error: str | None      # 读不了的原因（OSError 的 strerror）；读得了是 None

class JudgingState:
    """一次判定里命令文本与读进去的各层脚本共用的：写出的文件、从盘上读过的文件、判过的脚本。"""
    def __init__(self):
        self.written = {}          # 实际路径 → WrittenScript：同一条命令里到这里为止写出的文件（record_files_written）
        self.written_version = 0   # written 每变一次加一；判过的脚本只在它自那一遍起没变时复用
        self.files = {}            # 实际路径 → FileOnDisk：一次判定里同一份文件只从盘上读一遍
        self.files_read = []       # 从盘上读过的文件的实际路径，按读的次序，每份一次
        self.judged = {}           # (实际路径, 当前目录, written_version, 经没经内存包装, 参数…) → Scan，
                                   # 帧去掉了起它那一行之上的几格（复用时接上这一处的）
        self.writers = []          # 命令文本这一层里每条命令写了什么：(序号, 它点到的路径与文字, 它认得出写出内容的实际路径集, 命令名)；
                                   # 后面执行的脚本若被前面认不出写法的命令写过，照认不出拒（unrecognized_write）
        self.serial = 0            # 命令文本这一层里到这里为止判过几条命令（writers 里的序号）
        self.missing_cat_sources = []   # 几个源的 cat 里盘上不在的源：(源词, 目的词)；拒的时候拒绝说明点名它（cat 报错退 1，`&&` 后面其实不跑）

class Decision(NamedTuple):
    code: int              # 0 放行，2 拒绝
    message: str | None    # 拒绝时写给 stderr 的原因与出路
    notices: list          # 读不到、看不全的脚本（写好的句子，出在脚本里的带路径与行号）
    scripts_read: list     # 同 Scan.scripts_read
    files_read: list       # 同 JudgingState.files_read
    scan: object = None    # 这一次判的 Scan（task_server_decision 接着用；disable-check 下是 None）

# 命令位置上的词里有这些字符，就是没展开的变量、命令替换、通配、引号残片或别的语言的源码残片（逗号、等号），不是一个能读的路径
NOT_A_LITERAL_PATH = set("$`()'\"<>|&;*?[]{}=,\n")

LAUNCHER_WORDS = {"shell": "bash / sh 起的", "source": "source 读进来的",
                  "startup": "shell 启动时先读的（BASH_ENV / ENV、--rcfile、~/.bashrc、登录 shell 的 /etc/profile 与 HOME 下的 profile、/etc/bash.bashrc、zsh 的 .zshenv 与 .zshrc 指的）",
                  "program": "别的程序读进去当代码的（awk -f、sed -f、Makefile）", None: "直接执行的"}

def frames_text(frames):
    return " → ".join(f"{path}:{line}" if line else f"{path}（行号认不出）" for path, line in frames)

def command_lines(script_text):
    """脚本里每条命令（剥掉前缀与包装之后的词序列）出现在哪几行，按出现次序；
    逐个逻辑行（反斜杠续行接上）单独切词，喂给非 shell 命令的 heredoc 正文先换成同样多的空行，行号不变。"""
    lines = shell_words.strip_data_heredocs(script_text, keep_line_count=True).split("\n")
    found, number = {}, 0
    while number < len(lines):
        first_line, logical = number + 1, lines[number]
        while logical.endswith("\\") and number + 1 < len(lines):
            number += 1
            logical += "\n" + lines[number]
        number += 1
        for command in shell_words.commands_at_command_position(logical).commands:
            found.setdefault(tuple(command.words), []).append(first_line)
    return found

def literal_path(command, word):
    """命令里的一个词当文件路径（按这条命令的当前目录解开、取实际路径）；带没展开的变量、通配这类的，或目录认不出，交 None。"""
    if not word or NOT_A_LITERAL_PATH & set(word):
        return None
    resolved = shell_words.resolve_path(command.directory, os.path.expanduser(word))
    return None if resolved is None else os.path.realpath(resolved)

def heredoc_write_targets(command):
    """这一条 cat / tee 把喂给它的 heredoc 写进了哪些文件：[(目标词, 是不是追加)]；不是这种写法交 []。"""
    name, arguments = os.path.basename(command.words[0]), command.words[1:]
    if name not in ("cat", "tee") or not command.standard_input_heredocs:
        return []
    targets = []
    if name == "cat":
        if any(argument != "-" for argument in arguments):  # 带了文件参数（写出的是那些文件）或别的选项（-n 这类改了内容）
            return []
    else:
        only_files = False
        for argument in arguments:
            if not only_files and argument == "--":
                only_files = True
            elif only_files or not argument.startswith("-") or argument == "-":
                if argument != "-":
                    targets.append(argument)
        appends = any(argument in ("-a", "--append") or (argument.startswith("-") and not argument.startswith("--") and "a" in argument[1:])
                      for argument in arguments if argument != "--")
        targets = [(target, appends) for target in targets]
    if guard_break_is_set("heredoc-write-every-redirection"):   # 弄坏开关：输出重定向的目标不看 fd 号一律当写了正文
        for operator, target in command.output_redirections:
            if operator == ">&" and (target.isdigit() or target == "-"):
                continue
            targets.append((target, operator in (">>", "&>>")))
        return targets
    # cat / tee 把正文写到标准输出：只有标准输出最后落到的那个文件拿到正文（`2> 文件`、`3> 文件` 拿不到，`3>d.sh >&3` 拿得到；lib_shell_words.py 的 standard_output_destination）
    destination = shell_words.standard_output_destination(command.output_redirections)
    if destination is not None:
        targets.append((destination[0], destination[1] in (">>", "&>>")))
    return targets

# 同一条命令里先拷出再执行：cp 之外，ln（-s 的符号链接与硬链接，执行时读到的都是源的内容）、install、mv 的目标与 `cat 源 > 目的` 也按源的内容认
#   （弄坏开关 PROJECT_CODE_GUARD_BREAK=copy-by-cp-only 下只认 cp）；install 另有三个带值的选项，-d / --directory 建目录不拷文件
COPYING_COMMANDS = {"cp", "ln", "install", "mv"}
INSTALL_OPTIONS_WITH_VALUE = {"-m", "--mode", "-o", "--owner", "-g", "--group"}

def cat_destination(command):
    """`cat 源… > 目的`（不带选项、不读 heredoc）的目的词：标准输出最后落到的、由 `>` 或 `>|` 打开的那个文件（照 fd 号与 `n>&m` 跟，`2> 文件` 不算，
    `3>d.sh >&3` 算；lib_shell_words.py 的 standard_output_destination）；不是这种写法交 None。"""
    destination = shell_words.standard_output_destination(command.output_redirections)
    sources = command.words[1:]
    # `&>`、`>& 文件` 同时写标准输出与标准错误（源都在时错误输出是空的）、`>>`、`&>>` 追加；弄坏开关 copy-redirections-unseen 下只认 `>`、`>|`
    operators = (">", ">|") if guard_break_is_set("copy-redirections-unseen") else (">", ">|", "&>", ">&", ">>", "&>>")
    if (os.path.basename(command.words[0]) != "cat" or destination is None or destination[1] not in operators or not sources
            or any(source.startswith("-") for source in sources) or command.standard_input_heredocs):
        return None
    return destination[0]

def cat_copy_pairs(command):
    """`cat 源 > 目的`（只一个源，`&>`、`>& 文件` 同）：[(源词, 目的词)]；别的写法交 []（-n 这类改了内容；几个源连起来的、追加的另由 cat_concatenation 认）。"""
    destination = cat_destination(command)
    if destination is None or len(command.words) != 2 or cat_appends(command):
        return []
    return [(command.words[1], destination)]

def cat_concatenation(command):
    """`cat 源 源… > 目的`（两个以上的源，`cat s.sh "2"> d.sh` 里引号写的 2 也是一个源）：(源词表, 目的词)；不是这种写法交 None。
    目的里是各个源依次接起来的内容（不在的源 cat 报错、不写东西，照空的算；record_files_written）。
    弄坏开关 PROJECT_CODE_GUARD_BREAK=cat-concatenation-unseen 下不认（几个源的不当拷贝）。"""
    destination = cat_destination(command)
    if (destination is None or (len(command.words) < 3 and not cat_appends(command)) or guard_break_is_set("cat-concatenation-unseen")
            or guard_break_is_set("copy-by-cp-only")):
        return None
    return command.words[1:], destination


def cat_appends(command):
    """`cat 源… >> 目的` / `&>> 目的`：标准输出落到的那个文件是追加打开的。"""
    destination = shell_words.standard_output_destination(command.output_redirections)
    return destination is not None and destination[1] in (">>", "&>>")

def copy_pairs(command):
    """`cp`（与 ln / install / mv、`cat 源 > 目的`）这一条拷了哪些文件：[(源词, 目的词)]；不是这几样、或参数认不出，交 []。"""
    name = os.path.basename(command.words[0])
    cp_only = guard_break_is_set("copy-by-cp-only")
    if name == "cat" and not cp_only:
        return cat_copy_pairs(command)
    if name != "cp" and (cp_only or name not in COPYING_COMMANDS):
        return []
    if name == "install" and any(argument in ("-d", "--directory") for argument in command.words[1:]):
        return []
    operands, target_directory, no_target_directory, only_operands = [], None, False, False
    arguments, position = command.words[1:], 0
    while position < len(arguments):
        argument = arguments[position]
        if not only_operands and argument == "--":
            only_operands = True
        elif not only_operands and name == "install" and argument in INSTALL_OPTIONS_WITH_VALUE:
            position += 1
        elif not only_operands and argument in ("-t", "--target-directory", "-S", "--suffix"):
            if argument in ("-t", "--target-directory") and position + 1 < len(arguments):
                target_directory = arguments[position + 1]
            position += 1
        elif not only_operands and argument.startswith("--target-directory="):
            target_directory = argument.split("=", 1)[1]
        elif not only_operands and argument.startswith("-") and argument != "-":
            no_target_directory = no_target_directory or argument == "--no-target-directory" or (
                not argument.startswith("--") and "T" in argument[1:])
        else:
            operands.append(argument)
        position += 1
    if target_directory is not None:
        pairs = [(source, os.path.join(target_directory, os.path.basename(source))) for source in operands]
    elif len(operands) < 2:
        return []
    else:
        sources, destination = operands[:-1], operands[-1]
        destination_path = literal_path(command, destination)
        into_directory = not no_target_directory and (len(sources) > 1 or destination.endswith("/")
                                                       or (destination_path is not None and os.path.isdir(destination_path)))
        pairs = ([(source, os.path.join(destination, os.path.basename(source))) for source in sources] if into_directory
                 else [(sources[0], destination)])
    if name == "ln":   # 符号链接里的相对源从链接所在的目录算（lib_shell_words.py 的 link_source_word）
        pairs = [(shell_words.link_source_word(command.words[1:], source, destination), destination) for source, destination in pairs]
    return pairs

def file_on_disk(path, state):
    """盘上一份普通文件的内容：一次判定里同一份（按实际路径认）只读一遍，读过的记进 state.files_read。"""
    real_path = os.path.realpath(path)
    known = state.files.get(real_path)
    if known is not None:
        return known
    try:
        size = os.path.getsize(real_path)
        with open(real_path, "rb") as handle:
            data = handle.read(MAXIMUM_SCRIPT_BYTES + 1) if size <= MAXIMUM_SCRIPT_BYTES else None
            head = handle.read(SCRIPT_HEAD_BYTES) if data is None else data[:SCRIPT_HEAD_BYTES]
        if data is not None and len(data) > MAXIMUM_SCRIPT_BYTES:  # 量过大小之后才长大的
            size, data = len(data), None
        known = FileOnDisk(head, data, size, None)
    except OSError as error:
        known = FileOnDisk(b"", None, 0, error.strerror)
    state.files[real_path] = known
    state.files_read.append(real_path)
    return known

def text_on_disk(path, state):
    """追加之前盘上那份的正文：还不在交 ""，不是普通文件、超过读的上限、读不了、不是 UTF-8 文本交 None。"""
    if not os.path.lexists(path):
        return ""
    if not os.path.isfile(path):
        return None
    found = file_on_disk(path, state)
    if found.data is None:
        return None
    try:
        return found.data.decode("utf-8")
    except UnicodeDecodeError:
        return None

def written_target_word(command, word):
    """heredoc 写出的目标词：带 `$` 的照这段命令文本里前面赋过的值代入（与 note_writer 记重定向目标同一套，lib_shell_words.py 的 expand_known_variables）；
    展开不了交原词（literal_path 照认不出交 None）。例：`t=<字面路径>; cat > $t <<EOF … EOF; bash $t` 这里与记写者那里同样代入，同一个目标两处一致。
    弄坏开关 PROJECT_CODE_GUARD_BREAK=heredoc-targets-unexpanded 下原样交回。"""
    if "$" not in word or guard_break_is_set("heredoc-targets-unexpanded"):
        return word
    expanded = shell_words.expand_known_variables(word, command.variables or {})
    return word if expanded is None else expanded

def record_files_written(command, state):
    """这一条命令写出了哪些文件，记进 state.written（实际路径 → WrittenScript），后面的命令执行它们时拿这份内容判；每变一处 written_version 加一。"""
    if guard_break_is_set("ignore-written-scripts"):
        return
    written = state.written
    state.recognized_now = set()
    def remember(path, entry):
        written[path] = entry
        state.written_version += 1
        state.recognized_now.add(path)
    def forget(path):
        if written.pop(path, None) is not None:
            state.written_version += 1
    body = command.standard_input_heredocs[-1].text if command.standard_input_heredocs else None
    for target, appends in heredoc_write_targets(command):
        path = literal_path(command, written_target_word(command, target))
        if path is None:
            continue
        if not appends:
            remember(path, WrittenScript(body, None))
            continue
        entry = written.get(path)
        before = (text_on_disk(path, state) if entry is None else entry.text if entry.text is not None
                  else text_on_disk(entry.copied_from, state))
        if before is None:
            forget(path)  # 追加到读不了的文件上：不认，照旧读盘上那份
        else:
            remember(path, WrittenScript(before + body, None))
    for source, destination in copy_pairs(command):
        source_path, destination_path = literal_path(command, source), literal_path(command, destination)
        if source_path is None or destination_path is None:
            continue
        if source_path in written:
            remember(destination_path, written[source_path])
        elif os.path.isfile(source_path):
            remember(destination_path, WrittenScript(None, source_path))
        else:
            forget(destination_path)  # 源不在：认不出，执行时照旧按盘上有没有判
    concatenation = cat_concatenation(command)
    if concatenation is not None:
        sources, destination = concatenation
        destination_path = literal_path(command, destination)
        if destination_path is not None:
            pieces = []
            if cat_appends(command):   # 追加：接在这条命令里前面写出的、或盘上那份后面（读不了的不认）
                entry = written.get(destination_path)
                before = (text_on_disk(destination_path, state) if entry is None else entry.text if entry.text is not None
                          else text_on_disk(entry.copied_from, state))
                pieces = None if before is None else [before]
            missing_source = False
            for source in sources:
                if pieces is None and not guard_break_is_set("cat-append-unreadable-destination-raises"):
                    break   # 追加的目的盘上读不了（目录、超过读的上限、不是 UTF-8 文本）：目的不认，不再往 pieces 里接源（接了的话，判的那一刻抛 AttributeError、整条命令照常执行）
                source_path = literal_path(command, source)
                entry = written.get(source_path) if source_path is not None else None
                text = None if source_path is None else (
                    (entry.text if entry.text is not None else text_on_disk(entry.copied_from, state)) if entry is not None else text_on_disk(source_path, state))
                if text is None:
                    pieces = None   # 有一个源算不出（带变量、读不了、不是文本）：认不出，执行时照旧按盘上有没有判
                    break
                if entry is None and source_path is not None and not os.path.lexists(source_path):
                    # 源盘上不在、也不是同一条命令里前面认出写出的：读不到它的内容，照认不出（偏严：cat 不看源在不在那一类照拒）
                    state.missing_cat_sources.append((source, destination))
                    if not guard_break_is_set("cat-missing-source-allowed"):
                        missing_source = True
                pieces.append(text)   # 不在的源 text_on_disk 交 ""：cat 报错、不写东西
            if pieces is None or missing_source:
                forget(destination_path)
            else:
                remember(destination_path, WrittenScript("".join(pieces), None))
    tee_copy(command, state, remember, forget)


def piped_text(command, state):
    """这一条的标准输入里是什么（tee 拷出用）：`< 源`（读盘上或这条命令里前面写出的）、管道前一段是只带文件参数的 cat（几个源接起来）或 printf / echo / 读 heredoc 的 cat 的字面；
    算不出交 None。"""
    if command.input_redirections:
        operator, target = command.input_redirections[-1]
        if operator not in ("<", "<>"):
            return None
        sources = [target]
    elif command.piped_from is not None:
        words, heredocs = command.piped_from
        skipped, words = shell_words.skip_prefixes_expanding(list(words))
        if skipped.command_position is None:
            return None
        name, arguments = os.path.basename(words[skipped.command_position]), words[skipped.command_position + 1:]
        if name == "cat" and arguments and not heredocs and not any(argument.startswith("-") for argument in arguments):
            sources = arguments
        else:
            literal = shell_words.piped_literal_output(list(command.piped_from[0]), heredocs)
            return literal.code if literal.unjudged is None else None
    else:
        return None
    pieces = []
    for source in sources:
        source_path = literal_path(command, source)
        if source_path is None:
            return None
        entry = state.written.get(source_path)
        if entry is None and not os.path.lexists(source_path) and not guard_break_is_set("cat-missing-source-allowed"):
            return None   # tee 的源盘上不在、也不是同一条命令里前面认出写出的：读不到它的内容，照认不出（偏严：cat 不看源在不在那一类照拒）
        text = (entry.text if entry.text is not None else text_on_disk(entry.copied_from, state)) if entry is not None else text_on_disk(source_path, state)
        if text is None:
            return None
        pieces.append(text)
    return "".join(pieces)


def tee_copy(command, state, remember, forget):
    """`… | tee [-a] 目的…`、`tee 目的 < 源`（heredoc 喂给 tee 的由 heredoc_write_targets 认）：目的里是 tee 读到的标准输入（piped_text），-a 接在原来的后面；
    算不出的照旧不认（执行时按盘上判；前面写过的那份认不出写法，unrecognized_write 照拒）。弄坏开关 tee-copy-unseen 下不认。"""
    if guard_break_is_set("tee-copy-unseen") or os.path.basename(command.words[0]) != "tee" or command.standard_input_heredocs:
        return
    arguments = command.words[1:]
    appends = any(argument in ("-a", "--append") or (argument.startswith("-") and not argument.startswith("--") and "a" in argument[1:]) for argument in arguments)
    targets = [argument for argument in arguments if not argument.startswith("-") or argument == "-"]
    targets = [target for target in targets if target != "-"]
    text = piped_text(command, state)
    for target in targets:
        path = literal_path(command, target)
        if path is None:
            continue
        if text is None:
            forget(path)
            continue
        if appends:
            entry = state.written.get(path)
            before = (text_on_disk(path, state) if entry is None else entry.text if entry.text is not None else text_on_disk(entry.copied_from, state))
            if before is None:
                forget(path)
                continue
            remember(path, WrittenScript(before + text, None))
        else:
            remember(path, WrittenScript(text, None))

# 同一条命令里先写出、再执行的脚本，写法认不出的（echo / printf / dd / sed / head /
# install /dev/stdin / python 写出再起）：命令文本这一层里前面有一条命令点到了这份脚本（重定向的目标是它，或参数、heredoc 正文里写着它），又不是只读的命令、
# 也没被认成写出了确定内容（record_files_written 记下的那几种），执行它时照认不出拒（unrecognized_write）。弄坏开关 PROJECT_CODE_GUARD_BREAK=unrecognized-writes-allowed 下不判。
READ_ONLY_COMMANDS = frozenset({"ls", "stat", "test", "[", "[[", "file", "wc", "grep", "egrep", "fgrep", "rg", "head", "tail", "cat", "less", "more", "sha256sum",
                                "sha1sum", "md5sum", "cksum", "b2sum", "diff", "cmp", "realpath", "readlink", "basename", "dirname", "echo", "printf", "chmod", "chown",
                                "touch", "shellcheck", "du", "sort", "uniq", "cut", "true", "false", ":", "cd", "pushd", "popd", "export", "unset", "local", "declare", "typeset", "readonly",
                                "bash", "sh", "source", ".", "command", "type", "which", "hash", "mkdir", "rm", "rmdir", "nl", "od", "xxd", "hexdump", "strings", "sed",
                                "tr", "column", "jq", "find", "tree"})
# 经 PATH 找到的命令词（不带斜杠）是 shell 脚本的照读正文；这几个前缀下面的是系统包装的程序，不读
PATH_UNRESOLVED_SUFFIX = "，而这一条的 PATH 写成展开不了的值"
DIRECTORY_UNRESOLVED_SUFFIX = "，而这一条的当前目录算不出"
SYSTEM_PATH_PREFIXES = ("/usr/", "/bin/", "/sbin/", "/lib", "/opt/", "/snap/", "/etc/", "/nix/")


# python3 的代码（-c 的那个词、喂给 python3 - 的 heredoc 正文）里没有任何能写文件或起别的程序的东西，就是只读输入、往标准输出打字的过滤器：
# 代码里提到的字样（`d['task']` 里的 task）不算点到后面执行的脚本（`task-server/task`）。有这些的（open(、.write(、shutil、os.、subprocess、pathlib、exec( / eval( / compile(、
# dump(、.save( / .copy( / .move( / .link( / .touch( / .extract( 这一类）照旧整段算点到。命令词不是 python、跑的是脚本文件（python3 gen.py x.sh）的，参数照旧算点到。
# 弄坏开关 PROJECT_CODE_GUARD_BREAK=python-filter-code-judged 下不分，代码里有字样就算点到
PYTHON_COMMAND = re.compile(r"python[0-9.]*")
PYTHON_CODE_OPTION = re.compile(r"-[A-Za-z]*c")
PYTHON_WRITE_MARKER = re.compile(r"\bopen\s*\(|(?<!sys\.stdout)(?<!sys\.stderr)\.write\w*\s*\(|\bshutil\b|\bos\s*\.|\bsubprocess\b|\bpathlib\b|\bPath\s*\(|\bexec\s*\(|\beval\s*\("
                                 r"|(?<!re\.)\bcompile\s*\(|__import__|\bimportlib\b|\bctypes\b|\bdump\s*\(|\.(?:save\w*|copy\w*|move|rename|symlink|link|unlink|touch|mkdir\w*|extract\w*|truncate|chmod)\s*\("
                                 r"|urlretrieve|\bfileinput\b|\bio\s*\.|\bpopen\b|\bsystem\s*\(|\bgetattr\s*\(|\bsetattr\s*\(|\bglobals\s*\(")


def python_filter_code(command):
    """python3 -c 的代码、喂给 python3 - 的 heredoc 正文里没有任何能写文件或起别的程序的东西（PYTHON_WRITE_MARKER）的：→ (要从点到的文字里去掉的词下标集, 要不要去掉喂给它的 heredoc 正文)。
    不是这种的交 (空集, False)。"""
    words = list(command.words)
    if guard_break_is_set("python-filter-code-judged") or not words or command.launcher is not None or not PYTHON_COMMAND.fullmatch(os.path.basename(words[0])):
        return set(), False
    for index in range(1, len(words)):
        if PYTHON_CODE_OPTION.fullmatch(words[index]) and index + 1 < len(words):
            return ({index + 1} if PYTHON_WRITE_MARKER.search(words[index + 1]) is None else set()), False   # -c 之后的词是代码，再往后是代码的参数
        if not words[index].startswith("-"):
            return set(), False   # 跑的是脚本文件或 -m 的模块
    return set(), all(PYTHON_WRITE_MARKER.search(heredoc.text) is None for heredoc in command.standard_input_heredocs)   # 没有脚本也没有 -c：heredoc 是代码


def command_text_blob(command):
    """一条命令点到的文字：参数、重定向目标、喂给它的 heredoc 正文连起来（找它写没写某份脚本用）。python3 的代码没有任何写文件的东西的，代码不算（python_filter_code）。"""
    skipped, heredocs_are_filter_code = python_filter_code(command)
    pieces = [word for index, word in enumerate(command.words) if index >= 1 and index not in skipped] + [target for _, target in command.output_redirections]   # 命令词自己不算点到（再起一遍同一份脚本不是写它）
    if not heredocs_are_filter_code:
        pieces += [heredoc.text for heredoc in command.standard_input_heredocs]
    return "\n".join(pieces)


def mentions_path(blob, word, real_path):
    """blob 里写着这份脚本：照写的样子（带斜杠的）、实际路径，或文件名前后是路径边界（空白、引号、斜杠、等号、开头结尾；执行时写的不带斜杠、经 PATH 找到的不按文件名认）。"""
    if word and "/" in word and word in blob or real_path in blob:
        return True
    if word and "/" not in word and os.path.basename(real_path) == word and not word.endswith(SHELL_SCRIPT_EXTENSION):
        return False
    name = os.path.basename(real_path)
    return bool(name) and re.search(r"(?:^|[\s'\"/=(])" + re.escape(name) + r"(?=$|[\s'\");,])", blob) is not None


def note_writer(command, state, recognized, own_words=None):
    """命令文本这一层里这一条写了什么，记进 state.writers（recognized 是 record_files_written 认出写出了确定内容的实际路径集）。
    own_words 不是 None 时这一条是剥掉之后里面那条接着判的启动器（launcher_own_words）：点到的文字只取启动器自己的词与这一条的重定向、heredoc，
    里面那条剥开之后在里层自己记。"""
    state.serial += 1
    targets = []
    for _, target in command.output_redirections:
        expanded = shell_words.expand_known_variables(target, command.variables or {}) if "$" in target else target
        if expanded and not re.fullmatch(r"[0-9]+|-", expanded):
            path = literal_path(command, expanded)
            if path is not None:
                targets.append(path)
    # bash / source 起的脚本这一条是在执行它（命令词已换成脚本路径），照只读算；python 起的程序可能写文件，照写的算
    name = os.path.basename(command.words[0]) if command.words and command.launcher is None else {"shell": "bash", "source": "source", "startup": "source", "python": "python3"}.get(command.launcher, "")
    if command.launcher is None and command.words and copied_shell_binary(command, state) is not None:
        name = "bash"   # 同一条命令里拷来 / 链来的 shell 起脚本是在执行它（同 bash <脚本>）、照只读算，不是写它
    blob = command_text_blob(command if own_words is None else command._replace(words=[command.words[0], *own_words]))
    state.writers.append((state.serial, blob, frozenset(recognized), name, tuple(targets)))


def launcher_own_words(command, state):
    """这一条是剥掉之后里面那条接着判的启动器（/usr/bin/time、systemd-run、chrt、ssh 这一类，gpu-quiesce-run.sh、两种派发起法；
    scan_uses 里剥它的那几处同一套）时，交启动器自己的词（命令词之后、里面那条之前的选项与位置参数）；里面那条的词要是外层词的尾巴
    （选项带进来的 NAME=VALUE 先去掉），或像 ssh、sg 那样把尾巴几个词用空格连成 `sh -c` 的一段。不是启动器、或对不上尾巴的交 None，照整条记写者。
    例：`systemd-run --user --wait bash <轻脚本>` 不把 systemd-run 整条记成写过这份脚本的认不出写法，里面那条照常判。
    task-server/task submit（lib_project_code_runs.task_server_submission 认的那一种，python3 起的同算）同样交它自己的词（`submit`、`--cwd <目录>`、`--`）：
    客户端只把 `--` 之后那条交给服务、自己不写文件，那条在 scan_uses 里另判、在里层自己记写者。
    例：子 agent 交 `task-server/task submit -- bash <不按名字判的脚本>`（草稿里自己的辅助脚本；hook-swap 草稿树里经符号链接的崩溃点重放入口），
    不把 task 整条记成写过这份脚本的认不出写法，里面那条照常判。
    弄坏开关 PROJECT_CODE_GUARD_BREAK=launcher-recorded-as-writer 下一律交 None。"""
    if guard_break_is_set("launcher-recorded-as-writer") or not command.words:
        return None
    submitted = project_code_runs.task_server_submission(command.words, command.directory, command.launcher)
    if submitted is not None:
        submitted_words = list(submitted[0])
        cut = len(command.words) - len(submitted_words)
        return list(command.words[1:cut]) if cut >= 1 and list(command.words[cut:]) == submitted_words else None
    if command.launcher is not None:
        return None
    launched = (project_code_runs.command_under_launcher(command.words, command.directory)
                or project_code_runs.command_under_gpu_quiesce(command.words, command.directory)
                or project_code_runs.command_under_dispatch_launcher(command.words, command.directory))
    if launched is None:
        return None
    words, inner = list(command.words), list(launched[0])
    for skipped in range(len(inner)):
        if skipped and not project_code_runs.ENVIRONMENT_ASSIGNMENT_WORD.match(inner[skipped - 1]):
            break
        tail = inner[skipped:]
        cut = len(words) - len(tail)
        if cut >= 1 and words[cut:] == tail:
            return words[1:cut]
        if len(tail) == 3 and tail[:2] == ["sh", "-c"]:
            for cut in range(1, len(words)):
                if " ".join(words[cut:]) == tail[2]:
                    return words[1:cut]
    return None


def unrecognized_write(path, word, state):
    """执行的脚本 path 在命令文本这一层里前面被认不出写法的命令写过：交那条命令的命令名；没有交 None（后来一条认出了确定内容的写，盖掉前面的）。"""
    if guard_break_is_set("unrecognized-writes-allowed"):
        return None
    real_path = os.path.realpath(path)
    flagged = None
    for serial, blob, recognized, name, targets in state.writers:
        if serial == state.serial:
            continue
        if real_path in recognized:
            flagged = None
            continue
        if real_path in targets or (name not in READ_ONLY_COMMANDS and mentions_path(blob, word, real_path)):
            flagged = name or "前面一条命令"
    return flagged


FUNCTION_DEFINITION = re.compile(r"(?:^|[\s;&|(])(?:function\s+([A-Za-z_][\w-]*)|([A-Za-z_][\w-]*)\s*\(\s*\))")


def path_lookup_script(command):
    """命令词不带斜杠、直接执行的：照这一条的 PATH（命令前、env、export 设的，`$PATH` 展开成钩子自己的）在 PATH 上找，找到的不在系统目录（SYSTEM_PATH_PREFIXES）下就交它的路径；
    PATH 写成展开不了的值交 (None, 说明)；找不到、在系统目录下交 (None, None)。弄坏开关 PROJECT_CODE_GUARD_BREAK=path-scripts-unread 下一律交 (None, None)。"""
    word = command.words[0]
    if guard_break_is_set("path-scripts-unread") or word in shell_words.BASH_BUILTINS or word in shell_words.COMMAND_KEYWORDS:
        return None, None
    search = command.environment.get("PATH")
    if search is None:
        # 外面带进来的 PATH（钩子自己的）只找 .sh 结尾的（`s.sh` 这种写出来就是要经 PATH 起的脚本）；cargo、git 这类程序名不在钩子的 PATH 上找脚本读（本机用户 bin 目录下的包装照程序算）
        if not word.endswith(SHELL_SCRIPT_EXTENSION):
            return None, None
        search = os.environ.get("PATH", "")
    else:
        search = shell_words.expand_known_variables(search, dict(command.variables or {}, PATH=os.environ.get("PATH", "")))
        if search is None:
            return None, f"命令词 {word} 经 PATH 找{PATH_UNRESOLVED_SUFFIX}"
        search = ":".join(shell_words.resolve_path(command.directory, entry) or entry if entry and not entry.startswith("/") else entry
                          for entry in search.split(":"))
    found = shutil.which(word, path=search)
    if not found:
        return None, None
    found = os.path.abspath(found)
    if any(found.startswith(prefix) or os.path.realpath(found).startswith(prefix) for prefix in SYSTEM_PATH_PREFIXES):
        return None, None
    return found, None


def has_shell_script_extension(path):
    """执行的名字、或它指到的文件，扩展名是 .sh。"""
    return path.endswith(SHELL_SCRIPT_EXTENSION) or os.path.realpath(path).endswith(SHELL_SCRIPT_EXTENSION)

def script_path_of(command, written, strict=False):
    """命令位置上这一条执行的是不是一个要读进去的脚本文件：返回 (路径, None) 要读；(None, 找不到的说明) 记检出；(None, None) 不读也不记。
    written 是同一条命令里前面写出的文件（record_files_written），在里面的算找得到；直接执行的还不在，扩展名不是 .sh 的不读也不记。"""
    word = command.words[0]
    if command.launcher in ("shell", "source") and not guard_break_is_set("glob-scripts-unread") and set("*?[") & set(word) and not set("$`()'\"<>|&;{}=,\n") & set(word):
        # bash / source 起的脚本路径带通配（`bash 目录/*s.sh`）：照当前目录展开，bash 起的是排在头一个的那一份；对不上文件的 bash 报错不跑。弄坏开关 glob-scripts-unread 下不读
        pattern = shell_words.resolve_path(command.directory, os.path.expanduser(word))
        matches = sorted(glob.glob(pattern)) if pattern is not None else []
        return (matches[0], None) if matches else (None, None)
    if command.launcher == "python" or word.startswith("-") or NOT_A_LITERAL_PATH & set(word):
        return None, None
    expanded = shell_words.expand_directory_tilde(word, command.directory)
    if command.launcher is None and expanded is not None and "/" not in expanded:
        return path_lookup_script(command) if strict else (None, None)
    resolved = shell_words.resolve_path(command.directory, expanded) if expanded is not None else None
    undeterminable_directory = (strict and not guard_break_is_set("relative-script-undeterminable-directory-allowed")
                                and command.launcher in ("shell", "source", None))
    if resolved is None:
        # 当前目录算不出（cd 到的目录是算不出的变量 / 命令替换、~- 的 OLDPWD）而起的是相对路径脚本：认不出起的是哪一份（
        # 弄坏开关 PROJECT_CODE_GUARD_BREAK=relative-script-undeterminable-directory-allowed 下照旧放行）
        if undeterminable_directory:
            return None, f"{LAUNCHER_WORDS[command.launcher]}脚本 {word} 的当前目录算不出{DIRECTORY_UNRESOLVED_SUFFIX}"
        return None, None
    if (undeterminable_directory and not os.path.isabs(expanded) and command.directory is not None and not os.path.isdir(command.directory)
            and not command.cd_failure_skips):
        # cd 到的目录是字面路径、盘上不在：cd 会失败、留在原来的目录，相对路径脚本认不出在哪个目录。
        # cd 失败了这一条就不跑的（`cd 目录 && 这一条`、`cd 目录 || exit` 之后；lib_shell_words.py 的 cd_failure_skips）不在此列：照 cd 成功了的那个目录往下判
        # （同一条命令里前面写出的照读，不在的照「脚本不存在」只记；弄坏开关 LIB_SHELL_WORDS_BREAK=cd-failure-guards-unseen 下照旧拒）
        return None, (f"{LAUNCHER_WORDS[command.launcher]}脚本 {word} 要先 cd 到 {command.directory}，那个目录此刻盘上不在（同一条命令里前面 mkdir 建的也照不在算），"
                      "cd 会失败而这一条照跑（cd 后面接的是 `;`、换行或括号；写成 `cd 目录 && …` 或 `cd 目录 || exit` 才照 cd 成功了的目录判）"
                      f"{DIRECTORY_UNRESOLVED_SUFFIX}")
    if os.path.realpath(resolved) in written or os.path.lexists(resolved):
        return resolved, None
    if command.launcher == "startup":
        return None, None   # BASH_ENV / ENV 指的文件不在：shell 照样起、什么都不读（不报错、不按 PATH 找），没有可判的，不记检出
    if command.launcher is not None and "/" not in expanded:
        found = shutil.which(expanded, mode=os.F_OK)
        if found:
            return os.path.abspath(found), None
    if command.launcher is None and not has_shell_script_extension(resolved):
        return None, None
    return None, f"{LAUNCHER_WORDS[command.launcher]}脚本 {resolved} 不存在"

def shebang_interpreter(data):
    """`#!` 那一行指到的解释器名（`#!/usr/bin/env bash` 取 bash）；没有 `#!` 交 None。解析在共用模块（shell_words.shebang_command，直接执行的脚本读不读启动文件也用它）。"""
    shebang = shell_words.shebang_command(data)
    return None if shebang is None else shebang[0]

def executed_file_head(path, state):
    """直接执行的文件开头 SCRIPT_HEAD_BYTES 字节（读它的 #! 行用）：同一条命令里前面写出的拿写出的内容（cp 拷来的读源文件），其余读盘上那份；不是普通文件、读不了交 None。"""
    entry = state.written.get(os.path.realpath(path))
    if entry is not None and entry.copied_from is None:
        return (entry.text or "").encode("utf-8")[:SCRIPT_HEAD_BYTES]
    source = entry.copied_from if entry is not None else path
    if not os.path.isfile(source):
        return None
    found = file_on_disk(source, state)
    return None if found.error is not None else found.head

def executed_file_is_shell_script(executed_path, head):
    """直接执行的文件当不当 shell 脚本读，只看开头：编译产物（ELF 魔数、NUL 字节）不算；有 `#!` 的看它指不指到 shell；
    没有 `#!` 的看扩展名是不是 .sh（`.rs`、`.py`、`.md`、没扩展名的文本都不算）。"""
    if head.startswith(b"\x7fELF") or b"\0" in head:
        return False
    interpreter = shebang_interpreter(head)
    if interpreter is not None:
        return interpreter in shell_words.SHELL_NAMES
    return has_shell_script_extension(executed_path)

def decoded_script_text(data, path, launcher, executed_path):
    """已经读到手的脚本字节（盘上读的、或同一条命令里 heredoc 写出的）：返回值同 script_text。"""
    how = LAUNCHER_WORDS[launcher]
    if launcher is None and not executed_file_is_shell_script(executed_path, data[:SCRIPT_HEAD_BYTES]):
        return None, None
    if len(data) > MAXIMUM_SCRIPT_BYTES:
        return None, f"{how}脚本 {path} 有 {len(data)} 字节，超过读的上限 {MAXIMUM_SCRIPT_BYTES} 字节"
    if b"\0" in data:
        return None, f"{how}脚本 {path} 不是文本（有 NUL 字节）"
    try:
        return data.decode("utf-8"), None
    except UnicodeDecodeError:
        return None, f"{how}脚本 {path} 不是文本（不是 UTF-8）"

def script_text(path, launcher, state, executed_path):
    """盘上 path 那份当脚本读（executed_path 是执行的名字，直接执行的按它的扩展名判）：返回 (正文, None) 要读；(None, 读不到的原因) 记检出；
    (None, None) 不是普通文件（目录、设备）或直接执行的不是 shell 脚本，不读也不记。"""
    how = LAUNCHER_WORDS[launcher]
    if not os.path.isfile(path):
        return None, None
    found = file_on_disk(path, state)
    if found.error is not None:
        return None, f"{how}脚本 {path} 读不了（{found.error}）"
    if launcher is None and not executed_file_is_shell_script(executed_path, found.head):
        return None, None
    if found.data is None:
        return None, f"{how}脚本 {path} 有 {found.size} 字节，超过读的上限 {MAXIMUM_SCRIPT_BYTES} 字节"
    return decoded_script_text(found.data, path, launcher, executed_path)

def script_text_to_judge(path, launcher, state):
    """执行时这个脚本里是什么：同一条命令里前面写出的拿写出的内容（cp 拷来的读源文件），其余读盘上那份；返回值同 script_text。"""
    entry = state.written.get(os.path.realpath(path))
    if entry is None:
        return script_text(path, launcher, state, path)
    if entry.copied_from is not None:
        return script_text(entry.copied_from, launcher, state, path)
    return decoded_script_text(entry.text.encode("utf-8"), path, launcher, path)

def lower_cycle_frame(first, second):
    """两个「绕回外层的那一格」取靠外的那个；None 是没绕回。"""
    if first is None:
        return second
    return first if second is None else min(first, second)

def copied_shell_binary(command, state):
    """同一条命令里前面 cp / ln / install 拷来、链来的可执行文件（state.written 记下 copied_from），源解到已知 shell 二进制名（bash / sh / dash / zsh / ksh / rbash / ash）时交那个名；
    不是这种的（裸命令词、没拷来、源不是 shell）交 None（源算不出的那几种写法由 unrecognized_write 拦）。"""
    word = command.words[0]
    if word.startswith("-") or NOT_A_LITERAL_PATH & set(word):
        return None
    path = literal_path(command, word)
    if path is None:
        return None
    entry = state.written.get(path)
    if entry is None or entry.copied_from is None:
        return None
    name = os.path.basename(os.path.realpath(entry.copied_from))
    return name if shell_words.is_shell_name(name) else None


def program_file_text(program_file, state, variables=None):
    """别的程序从文件读的代码（awk -f、sed -f、Makefile；lib_shell_words.py 的 ProgramFile）：交 (正文, 读的路径, 说明, 说明是不是「认不出」)。
    候选路径按次序取头一份在的（同一条命令里前面写出的也算）；都不在交 (None, None, None, False)（程序自己报错、什么都不跑）；路径写成变量、命令替换，或当前目录算不出的，
    交认不出的说明（调用方在命令文本这一层照认不出拒）；读不了、太大、不是文本的同脚本，交只记的说明。
    variables 是这段命令文本里前面赋过的 shell 变量（CommandAtPosition.variables）：路径里的 `$名字`、`${名字}` 照它代入再判（d137-replay-classify 误拒 E5：
    `W=<字面目录>; awk -f $W/p.awk f.rs`）；代入不了的、带命令替换或反引号的照认不出。弄坏开关 PROJECT_CODE_GUARD_BREAK=program-file-variables-unexpanded 下不代入。"""
    for written in program_file.words:
        word = written
        if "$" in written and "`" not in written and "(" not in written and not guard_break_is_set("program-file-variables-unexpanded"):
            word = shell_words.expand_known_variables(written, variables or {}) or written
        if "$" in word or "`" in word or "(" in word:
            return None, None, f"{program_file.language} 读的文件 {written} 写成变量或命令替换，认不出读的是哪一份", True
        expanded = shell_words.expand_directory_tilde(word, program_file.directory)
        resolved = shell_words.resolve_path(program_file.directory, os.path.expanduser(expanded)) if expanded is not None else None
        if resolved is None:
            return None, None, f"{program_file.language} 读的文件 {word} 的当前目录算不出，认不出读的是哪一份", True
        if os.path.realpath(resolved) in state.written or os.path.isfile(resolved):
            body, problem = script_text_to_judge(resolved, "program", state)
            return body, resolved, problem, False
    return None, None, None, False


def scan_uses(text, directory, environment=None, frames_above=(), script=None, state=None, memory_capped=False, definitions=None, strict=None, positional=(),
               directory_guarded=True, interactive=False):
    """text 里命令位置上的每一处外部程序用法与不经 task-server 的直接起；执行的是脚本文件就读进去，用同一套切词与判定递归判。
    definitions 是外层命令文本里的变量定义（剥掉 /usr/bin/time 这一类之后里面那条接着判时带进来）；命令词没展开的，连同这一段自己的定义交给
    lib_project_code_runs.unexpanded_command_word_use 判（点到 验证包测试目标名的照直接执行它的测试二进制算）。读进去的脚本正文只用它自己的定义。
    script 是 text 所在的脚本（命令文本本身是 None），frames_above 是读到它为止经过的外层脚本与起它的那一行；
    state 是这一次判定里外各层共用的 JudgingState：脚本里写出的，外层后面的命令也看得见；同一份脚本只读一遍，判过的按键复用。
    memory_capped：text 整个跑在 run-with-memory-cap.sh 里（起它所在脚本的那一条经它包着）：只交给切词与复用的键，判不判都不看它（经没经内存包装不再判，文件头「经 task-server 起」）。
    跑项目代码的每一处（编译、编出来的二进制、本仓脚本、剥掉的内存包装）与子 agent 不起的客户端子命令记进 direct_runs，task-server/task submit 交给服务的那条
    （`--` 之后）另判一遍、只留它的检出与绕开服务的写法，绕开 task-server 的写法记进 bypasses，名单上直接跑的脚本记进 listed_scripts（文件头「经 task-server 起」）。
    strict：text 是命令文本这一层（Bash 调用的命令、它 bash -c / eval / 替换 / 剥掉启动器之后的那几段，不是读进来的脚本正文；不给时按 script is None 定）：
    认不出起什么的写法（lib_project_code_runs.unrecognized_launch）与认不出写法写出再执行的脚本（unrecognized_write）只在这一层判；脚本正文里只判命令词或脚本路径解不出、
    而定义或起它时给的参数（positional：起这份脚本的那一条的参数）里写着自证、门禁阶段、入口脚本或 cargo 的（lib_project_code_runs.script_body_unresolved_launch，文件头「认不出的写法」）。
    directory_guarded：起这段文本的那一条的 cd_failure_skips（lib_shell_words.py：前面可能失败的 cd 失败了它就不跑）；interactive：跑这段文本的 shell 可能是交互的。
    两样都交给切词，读进去的正文照起它的那一条的往下传。"""
    if state is None:
        state = JudgingState()
    if strict is None:
        strict = script is None
    launchers = frozenset() if guard_break_is_set("launcher-functions-unjudged") else shell_words.launcher_functions(text)
    shell_functions = {} if guard_break_is_set("bash-env-forwarding-untracked") else shell_words.shell_starting_functions(text)
    defined_functions = {name for pair in FUNCTION_DEFINITION.findall(text) for name in pair if name} if strict else set()
    own_definitions = project_code_runs.variable_definitions(text)
    for name, values in (definitions or {}).items():
        own_definitions.setdefault(name, []).extend(values)
    uses, notices, scripts_read, occurrences, direct_runs = [], [], [], {}, []
    bypasses, listed_scripts = [], []
    hit_depth_limit, lowest_cycle_frame = False, None
    line_index = None
    def frames_of(key, occurrence):
        nonlocal line_index
        if script is None:
            return frames_above
        if line_index is None:
            line_index = command_lines(text)
        lines = line_index.get(key, [])
        return frames_above + ((script, lines[occurrence] if occurrence < len(lines) else None),)
    # 命令文本本身（不是读进来的脚本正文：那份可能被 source 进别处、函数在外面调）里只定义、调不到的 function 的函数体不判
    scan = shell_words.commands_at_command_position(text, directory, environment, memory_capped=memory_capped, uncalled_functions_skipped=script is None,
                                                    directory_guarded=directory_guarded, interactive=interactive)
    # 喂给从标准输入读命令的 shell、这里判不了的输入（here-string 或管道前一段里有变量、命令替换，管道前一段是 cat 文件这类；lib_shell_words.py 的 shell_standard_input）：
    # 照常执行、记检出（字面的那一部分已经当命令切过、照判）
    notices += [Notice(frames_above if script is None else frames_above + ((script, None),), f"{entry}，这一处没判全")
                for entry in scan.unjudged_shell_inputs]
    # 命令文本这一层里喂给 shell 的不是字面的（管道前一段是 grep 这类别的命令、here-string 里有命令替换……）：认不出喂进去的是什么，照认不出拒（照样记检出）；
    # 弄坏开关 PROJECT_CODE_GUARD_BREAK=unjudged-shell-input-allowed 下只记不拒
    if strict and not guard_break_is_set("unjudged-shell-input-allowed"):
        uses += [UseAt(project_code_runs.program_use(project_code_runs.UNRECOGNIZED_KIND, f"{entry}，认不出喂进 shell 跑的是什么"),
                          frames_above) for entry in scan.unjudged_shell_inputs]
    def merge(inner):
        nonlocal hit_depth_limit, lowest_cycle_frame
        uses.extend(inner.uses)
        notices.extend(inner.notices)
        direct_runs.extend(inner.direct_runs)
        bypasses.extend(inner.bypasses)
        listed_scripts.extend(inner.listed_scripts)
        scripts_read.extend(inner.scripts_read)
        hit_depth_limit = hit_depth_limit or inner.hit_depth_limit
        lowest_cycle_frame = lower_cycle_frame(lowest_cycle_frame, inner.lowest_cycle_frame)
    # exec < 文件 / { … } < 文件 / ( … ) < 文件 / while … done < 文件：把 < 的文件下发给里层读标准输入的 bash / sh，照 `bash 文件` 读那份文件判
    # （弄坏开关 PROJECT_CODE_GUARD_BREAK=compound-stdin-untracked 下不认）
    if strict and not guard_break_is_set("compound-stdin-untracked"):
        feeds = shell_words.compound_redirected_stdin_files(text)
        for feed_file in feeds.files:
            merge(scan_uses(shlex.join(["bash", feed_file]), directory, environment, frames_above, None, state, memory_capped, own_definitions, strict=strict,
                             directory_guarded=directory_guarded))
        if feeds.undeterminable:
            uses.append(UseAt(project_code_runs.program_use(project_code_runs.UNRECOGNIZED_KIND,
                                 "exec < / 复合命令上的输入重定向把标准输入下发给里层 shell，来源写成变量或命令替换、算不出起的是哪一份"),
                                 frames_above))
    pending = list(scan.commands)   # 直接执行的脚本先读的启动文件在循环里接在后面（executed_script_startup）
    for command in pending:
        key = tuple(command.words)
        occurrence = occurrences.get(key, 0)
        occurrences[key] = occurrence + 1
        state.recognized_now = set()
        record_files_written(command, state)
        if strict:
            # 剥掉之后里面那条接着判的启动器（systemd-run、chrt、ssh、/usr/bin/time 这一类）只记它自己的词：里面那条起的脚本不当它写过（d137-replay-classify 误拒 E2）
            note_writer(command, state, state.recognized_now, launcher_own_words(command, state))
        # 剥掉的 capped.sh / run-with-memory-cap.sh 盘上有、又不是本仓那一份的（路径尾部相同的拷贝、经符号链接起的）：照读它的正文，这一条不算经内存包装
        # （第一百一十八节「门禁阶段与包装脚本同一口径」；弄坏开关 wrapper-copies-trusted 下照旧只按名字剥）
        if not guard_break_is_set("wrapper-copies-trusted"):
            for wrapper_word, wrapper_directory in command.wrappers:
                if project_code_runs.existing_copy_elsewhere(wrapper_word, wrapper_directory):
                    if os.path.basename(wrapper_word) == shell_words.MEMORY_CAP_WRAPPER:
                        command = command._replace(memory_capped=False)
                    merge(scan_uses(shlex.join(["bash", wrapper_word]), wrapper_directory, command.environment, frames_of(key, occurrence), None, state,
                                     False, None, strict=False, directory_guarded=command.cd_failure_skips))
        # 命令词写在变量里的先用这段命令文本里前面赋过的值展开再判（lib_project_code_runs.py 的 command_word_expanded）；
        # cargo 的参数写成前面赋过值的变量的同样展开（lib_project_code_runs.py 的 cargo_arguments_expanded）
        command = command._replace(words=project_code_runs.cargo_arguments_expanded(project_code_runs.command_word_expanded(command.words, command.variables), command.variables))
        # 文件头「经 task-server 起」③：绕开服务的写法（设准入开关、直接起服务本体与每次尝试的那一层、systemctl 停改它的单元），经不经 submit 都拒
        # （task_server_lib.py 载不进时这里什么都认不出，task_server_decision 照「判不了」记一条检出，不在每条命令上各记）
        if not guard_break_is_set("task-server-bypass-allowed"):
            bypass_details, _bypass_problem = project_code_runs.task_server_command_bypass(command.words, command.environment, command.directory, command.launcher)
            bypasses += [Bypass(detail, frames_of(key, occurrence)) for detail in bypass_details]
        # 文件头「经 task-server 起」②：子 agent 起提交客户端只许 status、log（lib_project_code_runs.subagent_task_server_refusal），别的记下由 task_server_decision 按谁在跑判
        client_refusal = project_code_runs.subagent_task_server_refusal(command.words, command.directory, command.launcher)
        if client_refusal is not None:
            direct_runs.append(DirectRun(client_refusal, frames_of(key, occurrence), SUBAGENT_CLIENT_LEVEL))
        # 文件头「经 task-server 起」①：task-server/task submit -- <命令>：交给服务的那条不经内存包装，这里只另判一遍、留下它的检出与绕开服务的写法（③）
        submitted = project_code_runs.task_server_submission(command.words, command.directory, command.launcher)
        if submitted is not None:
            submitted_words, submitted_directory = submitted
            inner = scan_uses(shlex.join(submitted_words), submitted_directory or command.directory, command.environment, frames_of(key, occurrence), None, state,
                               False, own_definitions, strict=strict, directory_guarded=command.cd_failure_skips)
            notices.extend(inner.notices)
            bypasses.extend(inner.bypasses)
            scripts_read.extend(inner.scripts_read)
            if not guard_break_is_set("temporary-service-submit-inner-unread"):
                direct_runs.extend(run for run in inner.direct_runs if run.allowed == TEMPORARY_SERVICE_BARE_LEVEL)   # 文件头「经 task-server 起」④：交给服务的那条里裸写的要临时服务的自证也拒
            continue
        # 本仓的代码（lib_project_code_runs.project_code_use）：名单上谁都能直接跑的不记，只给主 agent 的记下由 task_server_decision 按谁在跑判；
        # 剥掉的 capped.sh / run-with-memory-cap.sh 也是本仓的代码（经它们直接起照拒，交给服务的命令不带它们）
        project = project_code_runs.project_code_use(command.words, command.directory, command.launcher)
        if project is not None and project.allowed != "nobody":
            listed_scripts.append((project.real_path, project.allowed))
        if project is not None and project.allowed != "anyone":
            direct_runs.append(DirectRun(f"本仓的 {project.detail}", frames_of(key, occurrence), project.allowed))
        for wrapper_word, _wrapper_directory in command.wrappers:
            direct_runs.append(DirectRun(f"经 {os.path.basename(wrapper_word)} 直接起（交给服务的命令不带包装）", frames_of(key, occurrence)))
        # 认不出起什么的写法（命令文本这一层；lib_project_code_runs.py 的 unrecognized_launch）：照「认不出起什么的写法」那一类拒
        unrecognized = project_code_runs.unrecognized_launch(command.words, command.launcher) if strict else None
        # 脚本正文里命令词或脚本路径由变量拼出、解不出，而定义或位置参数里写着自证、门禁阶段、入口脚本或 cargo 的（例：经脚本里的变量起改过的钩子自证）
        if not strict:
            unrecognized = project_code_runs.script_body_unresolved_launch(command.words, command.launcher, own_definitions, positional,
                                                                   script if script is not None else (frames_above[-1][0] if frames_above else None))
        if unrecognized:
            uses.append(UseAt(project_code_runs.program_use(project_code_runs.UNRECOGNIZED_KIND, unrecognized),
                                 frames_of(key, occurrence)))
        # 文件头「经 task-server 起」④：要临时 prune-server 的自证写成裸的单独一个 --selftest：裸跑退 2、什么都不判，执行前拒（包在临时服务的包装里的，剥开重判时环境里带着标记，不记）
        if not guard_break_is_set("temporary-service-bare-allowed"):
            bare_selftest = project_code_runs.temporary_service_bare_selftest(command.words, command.directory, command.environment)
            if bare_selftest is not None:
                direct_runs.append(DirectRun(bare_selftest, frames_of(key, occurrence), TEMPORARY_SERVICE_BARE_LEVEL))
        # 命令词是这段文本里定义的、把参数当命令跑的函数（shell_words.launcher_functions：`run() { "$@"; }`、run_logged）：参数从每一处起的一段各当一条命令判
        if command.launcher is None and os.path.basename(command.words[0]) in launchers:
            for start in range(1, len(command.words)):
                merge(scan_uses(shlex.join(command.words[start:]), command.directory, command.environment, frames_of(key, occurrence), None, state,
                                 command.memory_capped, own_definitions, strict=strict, directory_guarded=command.cd_failure_skips))
        # 命令词是这段文本里定义的、函数体起 shell 的函数，且这一条前缀设了 BASH_ENV / ENV（函数体就地判时看不到调用处的 BASH_ENV）：
        # 拿调用处的 environment 重判函数体，补上里层 shell 读的启动文件（弄坏开关 PROJECT_CODE_GUARD_BREAK=bash-env-forwarding-untracked 下不认）
        if (command.launcher is None and os.path.basename(command.words[0]) in shell_functions
                and (shell_words.STARTUP_FILE_BASH_VARIABLE in command.environment or shell_words.STARTUP_FILE_POSIX_VARIABLE in command.environment)):
            merge(scan_uses(shell_functions[os.path.basename(command.words[0])], command.directory, command.environment,
                             frames_of(key, occurrence), None, state, command.memory_capped, own_definitions, strict=strict,
                             directory_guarded=command.cd_failure_skips))
        # 别的程序里起的命令（python / perl / awk / sed / tclsh / make / git / tar / ssh 这类；lib_shell_words.py 的 embedded_launches）：
        # 字面写着的照一段命令文本接着判（同 bash -c）；程序从文件读的代码（awk -f、sed -f、Makefile）读正文判（program_text_launches，照脚本正文那样只判解得出的）；
        # 写法认不出的，命令文本这一层照认不出拒，读进来的正文里照常执行、记检出（弄坏开关 PROJECT_CODE_GUARD_BREAK=program-files-unread 下不读程序文件，
        # embedded-unjudged-allowed 下认不出的只记不拒）
        embedded = shell_words.embedded_launches(command)
        embedded_frames = frames_of(key, occurrence)
        embedded_unjudged = list(embedded.unjudged)
        for embedded_text in embedded.texts:
            merge(scan_uses(embedded_text, command.directory, command.environment, embedded_frames, None, state, command.memory_capped, own_definitions,
                             strict=strict, directory_guarded=command.cd_failure_skips))
        for program_file in () if guard_break_is_set("program-files-unread") else embedded.program_files:
            body, location, problem, unrecognized_problem = program_file_text(program_file, state, command.variables)
            if problem and unrecognized_problem:
                embedded_unjudged.append(problem)
            elif problem:
                notices.append(Notice(embedded_frames, f"{problem}，里面起的命令没判"))
            if body is None:
                continue
            scripts_read.append(location)
            file_texts, file_unjudged = shell_words.program_text_launches(program_file.language, body, program_file.variables)
            for file_text in file_texts:
                merge(scan_uses(file_text, program_file.directory, command.environment, embedded_frames + ((location, None),), None, state, command.memory_capped,
                                 None, strict=False, directory_guarded=command.cd_failure_skips))
            notices += [Notice(embedded_frames + ((location, None),), f"{entry}，这一处没判全") for entry in file_unjudged]
        for entry in embedded_unjudged:
            if strict and not guard_break_is_set("embedded-unjudged-allowed"):
                uses.append(UseAt(project_code_runs.program_use(project_code_runs.UNRECOGNIZED_KIND, f"{entry}（{os.path.basename(command.words[0])} 里起的命令）"),
                                     embedded_frames))
            else:
                notices.append(Notice(embedded_frames, f"{entry}，这一处没判全"))
        # /usr/bin/time、flock、systemd-run 这一类包在外面照样起那条命令的：剥掉它，里面那条按一段命令文本接着判（包装、脚本、内存包装照认）；
        inner_capped = command.memory_capped
        launched = project_code_runs.command_under_launcher(command.words, command.directory)
        if launched is None:
            launched = project_code_runs.command_under_gpu_quiesce(command.words, command.directory)
        # 派发起法 multi-host-run.sh --peer … --（lib_project_code_runs.py 的 command_under_dispatch_launcher）：里面那条经它的内存包装起，
        if launched is None:
            launched = project_code_runs.command_under_dispatch_launcher(command.words, command.directory)
            if launched is not None:
                inner_capped = inner_capped or project_code_runs.dispatched_under_memory_cap(command.words, command.directory)
        if launched is not None:
            inner_words, inner_directory = launched
            inner_start = shell_words.resolve_path(command.directory, inner_directory) if inner_directory else command.directory
            # 临时 prune-server 的包装（给了 --server-binary 的才剥得开）给里面那条原命令设了临时服务的标记：重判里面那条时补上，免得把包在包装里的要临时服务的自证当成裸写（文件头「经 task-server 起」④）
            inner_environment = command.environment
            if not guard_break_is_set("temporary-service-wrapper-not-marked") and project_code_runs.command_under_temporary_service(command.words, command.directory) is not None:
                inner_environment = project_code_runs.environment_under_temporary_service(command.environment)
            inner = scan_uses(shlex.join(inner_words), inner_start, inner_environment, frames_of(key, occurrence), None, state, inner_capped,
                               own_definitions, strict=strict, directory_guarded=command.cd_failure_skips)
            merge(inner)
            continue
        # 同一条命令里 cp / ln / install 拷来、链来的、或盘上符号链接指到的可执行文件，解到已知 shell 二进制（bash / sh / dash / zsh / ksh / rbash / ash）：
        # 按那种 shell 重判它的参数（-c 的代码、起的脚本、启动文件）。弄坏开关 PROJECT_CODE_GUARD_BREAK=copied-shell-binary-untracked 下不认
        if command.launcher is None and not guard_break_is_set("copied-shell-binary-untracked"):
            shell_binary = copied_shell_binary(command, state)
            if shell_binary is not None:
                merge(scan_uses(shlex.join([shell_binary] + list(command.words[1:])), command.directory, command.environment,
                                 frames_of(key, occurrence), None, state, command.memory_capped, own_definitions, strict=strict,
                                 directory_guarded=command.cd_failure_skips))
                continue
        # 跑编译出来的代码、起编译的（文件头「经 task-server 起」①：经没经内存包装都一样，不经 task-server 的拒）
        compiled = project_code_runs.runs_compiled_code(command.words, command.directory)
        building = project_code_runs.starts_a_build(command.words, command.directory)
        if compiled or building:
            direct_runs.append(DirectRun(compiled or building, frames_of(key, occurrence)))
        # 本仓之外要经服务起的外部程序、判不出的通配（lib_project_code_runs.classify）：记一条 use，由 scan_decision 拒；已经拒了的不再读它的脚本正文
        test = project_code_runs.classify(command.words, command.directory, command.environment)
        if test:
            uses.append(UseAt(test, frames_of(key, occurrence)))
            continue
        # 按名字认的仓内入口脚本（门禁阶段、KNOWN_SCRIPT_LOCATIONS 里的）起它已经记成 ① 的直接起；里面起什么由它们自己的自证与门禁管，正文不读
        if project_code_runs.judged_by_name(command.words[0], command.directory):
            continue
        if strict and (command.launcher in ("shell", "source", "startup") or (command.launcher is None and ("/" in command.words[0] or command.words[0].endswith(SHELL_SCRIPT_EXTENSION)))):
            # 同一条命令里前面认不出写法写过的脚本（还不在盘上的也算）：执行前算不出它里面是什么，照认不出拒（unrecognized_write）
            candidate = literal_path(command, command.words[0])
            written_by = unrecognized_write(candidate, command.words[0], state) if candidate is not None else None
            if written_by:
                uses.append(UseAt(project_code_runs.program_use(project_code_runs.UNRECOGNIZED_KIND,
                                                            f"{LAUNCHER_WORDS[command.launcher if command.launcher in LAUNCHER_WORDS else None]}脚本 {candidate} 是同一条命令里前面的 {written_by} "
                                                            + ("拷出或链出的，而源此刻盘上不在、或源与目的的写法认不出（ln -s 的相对源从链接所在的目录算），执行前算不出它里面是什么"
                                                               if written_by in COPYING_COMMANDS else
                                                               "写的，写法认不出（不是 cat / tee 读 heredoc、cp / ln / install / mv、cat 源 > 目的、tee 拷出这几种），执行前算不出它里面是什么")),
                                     frames_of(key, occurrence)))
                continue
        path, missing = script_path_of(command, state.written, strict and os.path.basename(command.words[0]) not in defined_functions)
        if path is None and missing is None:
            continue
        here = frames_of(key, occurrence)
        if missing and missing.endswith((PATH_UNRESOLVED_SUFFIX, DIRECTORY_UNRESOLVED_SUFFIX)):
            uses.append(UseAt(project_code_runs.program_use(project_code_runs.UNRECOGNIZED_KIND, f"{missing}，认不出起的是哪一份"),
                                 here))
            continue
        if missing:
            notices.append(Notice(here, f"{missing}，里面的命令没判"))
            continue
        real_path = os.path.realpath(path)
        enclosing = [os.path.realpath(frame_path) for frame_path, _ in here]
        if real_path in enclosing:
            lowest_cycle_frame = lower_cycle_frame(lowest_cycle_frame, enclosing.index(real_path))
            continue
        if len(here) + 1 > MAXIMUM_SCRIPT_DEPTH:
            hit_depth_limit = True
            notices.append(Notice(here, f"{LAUNCHER_WORDS[command.launcher]}脚本 {path} 是第 {len(here) + 1} 层，"
                                        f"超过读的上限 {MAXIMUM_SCRIPT_DEPTH} 层，看不全，里面的命令没判"))
            continue
        if command.launcher is None:
            # 直接执行的脚本：它的 shell 起来时先读 BASH_ENV / ENV / .bashrc / zsh 的启动文件（lib_shell_words.py 的 executed_script_startup），
            # 读到的启动文件接在后面当被起的脚本判；#! 指到的解释器认不出、而环境里设了 BASH_ENV 或 ENV 的，照认不出拒
            startups, unknown_interpreter = shell_words.executed_script_startup(command, executed_file_head(path, state))
            pending.extend(startups)
            if unknown_interpreter is not None:
                uses.append(UseAt(project_code_runs.program_use(project_code_runs.UNRECOGNIZED_KIND,
                                                            f"脚本 {path} 的 #! 行指到 `{unknown_interpreter}`，认不出它起来时读不读 BASH_ENV / ENV 指的启动文件，而这条命令设了它们"),
                                     here))
        body, problem = script_text_to_judge(path, command.launcher, state)
        if problem:
            notices.append(Notice(here, f"{problem}，里面的命令没判"))
            continue
        if body is None:
            continue
        # 入口按自己所在的目录 source 的库换成字面路径，照读进来判（lib_project_code_runs.py 的 script_directory_expanded）
        body = project_code_runs.script_directory_expanded(body, path)
        # 跑这份正文的 shell 交不交互：source 与启动文件是起它的那个 shell，bash / sh 起的是新起的那个 shell（切词记在 command.interactive），直接执行的看 #! 行的选项里有没有 i。
        # 非交互的 shell source 的文件读到 `case $- in *i*) ;; *) return;; esac` 这类守卫就 return 了，后面的不读（lib_shell_words.py 的 non_interactive_return_offset；
        # ~/.bashrc 开头的守卫之后才 source bash-completion 这类大文件，非交互的登录 shell 读登录启动文件时照读全份，每次多两三秒、多记一条检出）
        if command.launcher in ("source", "startup", "shell"):
            body_interactive = command.interactive
        else:
            shebang = shell_words.shebang_command(executed_file_head(path, state) or b"")
            body_interactive = shebang is not None and any(option.startswith("-") and not option.startswith("--") and "i" in option[1:] for option in shebang[1])
        return_offset = shell_words.non_interactive_return_offset(body) if command.launcher in ("source", "startup") and not body_interactive else None
        if return_offset is not None:
            body = body[:return_offset]
        judged_key = (real_path, command.directory, state.written_version, command.memory_capped, tuple(command.words[1:]),
                      command.cd_failure_skips, body_interactive, return_offset)
        earlier = state.judged.get(judged_key)
        if earlier is not None:
            uses += [use._replace(frames=here + use.frames) for use in earlier.uses]
            notices += [notice._replace(frames=here + notice.frames) for notice in earlier.notices]
            direct_runs += [run._replace(frames=here + run.frames) for run in earlier.direct_runs]
            bypasses += [bypass._replace(frames=here + bypass.frames) for bypass in earlier.bypasses]
            listed_scripts += earlier.listed_scripts
            continue
        scripts_read.append(path)
        inner = scan_uses(body, command.directory, command.environment, here, path, state, command.memory_capped, strict=False,
                           positional=tuple(command.words[1:]), directory_guarded=command.cd_failure_skips, interactive=body_interactive)
        merge(inner)
        # 它自己在帧里的下标是 len(here)：绕回比它更外的、撞上嵌套上限的、判的时候写出了文件的，换一处起就可能不一样，不复用
        depends_on_where_started = inner.hit_depth_limit or (inner.lowest_cycle_frame is not None and inner.lowest_cycle_frame < len(here))
        if not depends_on_where_started and state.written_version == judged_key[2]:
            state.judged[judged_key] = Scan([use._replace(frames=use.frames[len(here):]) for use in inner.uses],
                                                 [notice._replace(frames=notice.frames[len(here):]) for notice in inner.notices],
                                                 [], False, None,
                                                 [run._replace(frames=run.frames[len(here):]) for run in inner.direct_runs],
                                                 [bypass._replace(frames=bypass.frames[len(here):]) for bypass in inner.bypasses],
                                                 list(inner.listed_scripts))
    return Scan(uses, notices, scripts_read, hit_depth_limit, lowest_cycle_frame, direct_runs, bypasses, listed_scripts)

# 一切跑项目代码的命令只认经 task-server/task 起；拒绝说明里的规矩、出路按谁在跑、被拒的是哪一类写。
TASK_SERVER_RULE = ("→ 规矩：一切跑项目代码的命令只认经 task-server/task 起，主 agent 与子 agent 同判（records/2026-10-30-编译绕过内存总上限.md 批 3b ①）："
                      "会编的 cargo（build、check、clippy、test、run、bench、doc，别名与插件照会编算；fmt、metadata、tree、clean 不编，直接跑）、直接执行编出来的二进制、"
                      "本仓里的脚本（门禁阶段与格、入口脚本、实验装置与复跑、变异、各脚本的 --selftest 与样本档与弄坏开关、喂钩子）、本仓之外的外部程序（qemu-system-*、herd7、"
                      "e152-file-system-benchmark，只问版本的 herd7 除外）；"
                      "经 run-with-memory-cap.sh、capped.sh、multi-host-run.sh 直接起的照拒（交给服务的命令不带这几样，服务自己套包装、定额度、挑机器、分卡）。"
                      "直接跑的只剩只读命令（只读的 git、ls、grep、cat、sed -n、wc、sha256sum、diff、du、ps 这类）、语法检查（bash -n、python3 -m py_compile）"
                      "与名单上的文本工具（.claude/hooks/lib_project_code_runs.py 的 TEXT_TOOLS_RUN_DIRECTLY，子 agent 起其中的 task-server/task 只许 status、log；"
                      "主 agent 另有 MAIN_AGENT_TOOLS_RUN_DIRECTLY 那几样会话工具）。")
TASK_SERVER_POLICY = (TASK_SERVER_RULE + "\n"
                      "→ 怎么办：`task-server/task submit -- <原命令>`（当场打出任务号；范围声明写在 submit 前面：`SINGLEFS_RUN_SCOPE='…' task-server/task submit -- <命令>`），"
                      "再用 run_in_background 起 `task-server/task wait <号>`，结束本轮等完成通知（wait 的退出码就是任务的，末行 TASK_SERVER_RESULT 写错误类型与日志）；"
                      "命令里有 &&、管道、cd 的写成 `task-server/task submit [--cwd <目录>] -- bash -c '<命令>'`。提交写法在 task-server/README.md「提交写法」。")
# 子 agent 不往统一任务服务交任务（文件头「经 task-server 起」②）：直接跑被拒（①）与起客户端被拒（②）的出路是同一句，写在 lib_project_code_runs.SUBAGENT_HAND_BACK_WAY_OUT
SUBAGENT_TASK_SERVER_RULE = ("→ 规矩：子 agent 一律不往统一任务服务交任务（用户 2026-11-01「现在需要发给 task-server的任务 都不能自己启动」；服务分不出提交方是不是子 agent，"
                             "拒在这道闸，records/2026-10-29-subagent拆分提案.md 第一百七十四节）：task-server/task 只许起只读的 status、log，"
                             "交什么命令都一样（带 --item 的、harness 测试、clippy / build、自证、样本档、弄坏开关、命名检查都算）。")

# 文件头「经 task-server 起」④：要临时 prune-server 的自证写成了裸的 --selftest。弄坏开关 temporary-service-remedy-dropped 下不贴这一段（说明只剩哪一份）
TEMPORARY_SERVICE_POLICY = "" if guard_break_is_set("temporary-service-remedy-dropped") else (
    "→ 规矩：.claude/hooks/selftest-temporary-service.tsv 里判定是 required 的自证，--selftest 开跑先核临时 prune-server（自证不写真服务），"
    "没有就退 2、什么都不判；要在包装 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service 底下起，包装起一份临时服务、在它底下起原命令。\n"
    "→ 怎么办：照上面「整条写成」的那一条：主 agent 经 `task-server/task submit --` 交，子 agent 把那一条写进报告「要主 agent 跑的命令」一节（子 agent 自己不交），"
    "命令里不写内存包装、host-dispatch.py 与绕开包装的起法。")

def subagent_task_server_policy(direct_run):
    """子 agent 被拒时的规矩与出路：direct_run 为真是直接跑项目代码被拒（①，先贴 TASK_SERVER_RULE），为假是起客户端被拒（②）。
    出路那一句取 lib_project_code_runs（共用模块读不到时这一道整个不判，到不了这里）。"""
    return ((TASK_SERVER_RULE + "\n") if direct_run else "") + SUBAGENT_TASK_SERVER_RULE + "\n→ 怎么办：" + project_code_runs.SUBAGENT_HAND_BACK_WAY_OUT + "。"
# 绕开 task-server 的写法（文件头「经 task-server 起」③）
TASK_SERVER_BYPASS_POLICY = ("→ 规矩：加任务、取消任务、看任务、删编译目录都走 task-server/task 的 submit / wait / status / cancel / log / clean（用户定「主agent 增加redis任务和取消redis任务都要走标准接口 不可以绕过」）："
                             "库文件、签名密钥、socket 只有服务读写；服务给的编译目录只经 `task-server/task clean <号>` 删（只许提交它的会话、有结局才删、删前删后报大小）；「已由 task-server 准入」开关只由服务设给内存包装；服务本体与每一次尝试的那一层只由 systemd 与服务起；"
                             "服务的单元只经 task-server/control 的 install / start / stop / check / watch 起停。\n"
                             "→ 怎么办：改用 task-server/task 的那几个子命令或 task-server/control；要查服务的状态用 `task-server/control check`、`task-server/task status`；"
                             "删编译目录用 `task-server/task clean <号>`（号是 wait 末行与 status 里那个任务的）。"
                             "子 agent 起 task-server/task 只许 status、log：要取消、要删编译目录的写进报告「要主 agent 跑的命令」一节交回。"
                             "命令里只是搜这几个名字的（grep 库文件名这类），把名字拆开写（`grep -n 'task-server\\.sqlite'`）。")
SCRIPT_POLICY = ("→ 这一步写在脚本里：命令位置上执行的脚本（bash / sh / ./ / source 起的，经 capped.sh / nice / timeout / env 包一层的，脚本里再起的脚本）"
                 "读进去逐条照同一张表判。把那一步从脚本里拿掉再跑，或照上一条交主 agent。")
UNRECOGNIZED_POLICY = ("→ 规矩：认不出起的是什么的写法一律拒（用户 2026-10-29 定「认不出的一律拒（偏严）」）：命令词、bash / source 起的脚本路径、shell 启动文件（BASH_ENV / ENV、--rcfile、HOME 与 ZDOTDIR 下的）路径、直接执行的脚本 #! 行指到认不出的解释器而环境里设了 BASH_ENV / ENV 的、cargo test / run / bench 的子命令与参数"
                       "由变量、命令替换、位置参数（`\"$@\"`、`$1`）、进程替换拼出而执行前展开不了的；xargs、find -exec、parallel 把读进来的词接进命令里的；"
                       "同一条命令里先用 echo / printf / dd / sed / head / python 这类写法写出、再起的脚本；读进来的脚本正文里命令词或脚本路径解不出、"
                       "而定义或起它的参数里写着自证、门禁阶段、入口脚本或 cargo 的。\n"
                       "→ 怎么改：写成展开之后的字面命令（`cargo test -p <包> --test <目标>`、`bash <脚本的字面路径>`），变量只用在同一条命令里前面赋过字面值的；"
                       "要先写出脚本的，用 `cat > 文件 <<'EOF'` 或 cp 写出、再起它；要把参数交给命令的，直接写那条命令，不经 `bash -c '…\"$@\"' _ …`、"
                       "自己定义的转发函数或 xargs；脚本里起自证、门禁阶段、入口脚本或 cargo 的，每一条写成字面路径与字面参数（不经数组、变量或 \"$@\" 拼），"
                       "改过的钩子或脚本的自证交主 agent 在主工作区或集成副本里跑。子 agent 改不成的写进交回报告交主 agent；主 agent 确实要这样跑的，经 `task-server/task submit --` 提交（服务那边不判这一条）。")
# 主 agent 被拒：规矩只贴出处一行（规矩正文在下面这几份里，不在拒绝信息里整段贴）
MAIN_AGENT_RULE_POINTER = ("→ 规矩只贴出处：.claude/rules/verification.md（一切跑项目代码的命令经 task-server/task 起、谁起、什么时候起）；"
                           "整轮门禁照 .claude/skills/gate/staged-gate.md 跑 scripts/gate/gate-staged.sh，提交时的崩溃点重放照 .claude/skills/crash-test/commit-stages.md。")
COPY_COMMAND_LIMIT = 2000   # 照抄的那条命令超过这么多字就不整条贴，只写怎么改

def run_scope_declaration(candidate, directory):
    """照抄的那条要带的四段范围声明（写法照 .claude/hooks/run-scope-guard.sh 文件头；为什么一段是占位，照抄的人要填）：
    命令文本里直接起的要声明的验证命令恰好一条、还没带声明时交声明原文；不是验证命令、几条、在驱动脚本里、已带声明的交 None。
    判法取同目录的 lib_run_scope.py（读不到、判的时候抛异常都交 None）；弄坏开关 copy-without-run-scope 下一律交 None。"""
    if guard_break_is_set("copy-without-run-scope"):
        return None
    try:
        run_scope_spec = importlib.util.spec_from_file_location("lib_run_scope", os.path.join(sys.argv[1], "lib_run_scope.py"))
        run_scope = importlib.util.module_from_spec(run_scope_spec)
        run_scope_spec.loader.exec_module(run_scope)
        run_scope.load_judging_libraries(sys.argv[1])
        runs = run_scope.scope_runs_in_text(candidate, directory)
    except Exception:
        return None
    if len(runs) != 1 or runs[0].frames or runs[0].declaration is not None:
        return None
    shape = runs[0].shape
    if any(mark in text for text in (shape.what, shape.range_hint) for mark in ("'", ";")):
        return None
    return (f"{run_scope.DECLARATION_VARIABLE}='种类={shape.kind}; 跑什么={shape.what}; 范围={shape.range_hint}; "
            "为什么=<为什么这么跑、这个范围为什么合适>'")

class DispatchLaunchersSeen:
    """交给 lib_workspace_cargo_lock.py 的判定库：command_under_launcher 另剥派发起法 multi-host-run.sh --peer … --（lib_project_code_runs.py 的 command_under_dispatch_launcher），
    里面起的 cargo 照样判编译目录；别的名字照 lib_project_code_runs 那一份。"""
    def __getattr__(self, name):
        return getattr(project_code_runs, name)

    def command_under_launcher(self, words, directory=None):
        return project_code_runs.command_under_launcher(words, directory) or project_code_runs.command_under_dispatch_launcher(words, directory)

def scan_decision(hook_input, project_root):
    """返回 Decision：code 0 放行、2 拒绝；notices 是读不到、看不全的脚本，放不放行都照记。
    判三样：认不出起什么的写法与判不出的通配（UNRECOGNIZED_KIND、pattern-unresolved）、要经 task-server 起的外部程序（external-program，lib_project_code_runs.classify）、
    主工作区里直接起且编译目录会落进主仓的 cargo；本仓的代码、子 agent 起客户端与绕开服务的写法由 task_server_decision 接着判。"""
    if guard_break_is_set("disable-check"):
        return Decision(0, None, [], [], [])
    tool_input = hook_input.get("tool_input") if isinstance(hook_input.get("tool_input"), dict) else {}
    command = tool_input.get("command") or ""
    agent_type = hook_input.get("agent_type") or None
    directory = hook_input.get("cwd") or project_root
    state = JudgingState()
    scan = scan_uses(command, directory, state=state)
    notices = [f"脚本 {frames_text(notice.frames)} 里{notice.text}" if notice.frames else notice.text for notice in scan.notices]
    external_refused, in_script, unrecognized_refused = [], False, []
    for use in scan.uses:
        where = f"脚本 {frames_text(use.frames)} 里的 " if use.frames else ""
        in_script = in_script or bool(use.frames)
        if use.test.kind in ("external-program", "project-code"):
            if not frames_mention_listed(use.frames, scan.listed_scripts, agent_type):
                external_refused.append(f"{where}{use.test.detail}")
            continue
        unrecognized_refused.append(f"{where}{use.test.detail}")
    target_refusal = None
    if "cargo" in command:   # 粗筛：命令文本里连这个词都没有的不用再切一遍
        if cargo_lock is None:
            notices.append(f"读不到 {os.path.join(sys.argv[1], 'lib_workspace_cargo_lock.py')}：{cargo_lock_error!r}，这条命令里的 cargo 没判编译目录会不会落进主仓")
        else:
            # 主工作区里直接起、编译目录会落进主仓的 cargo（文件头「再一道」编译目录那一段）：说明单独成一段，后面照它那样拒（不给「照抄」那一条）
            target_refusal, target_notices = cargo_lock.workspace_cargo_target_refusal(command, directory, shell_words, DispatchLaunchersSeen(), project_root)
            notices += target_notices
    external_refused = list(dict.fromkeys(external_refused))
    if not external_refused and not unrecognized_refused:
        if target_refusal:   # 只有编译目录这一处要拒：不给「照抄」那一条
            return Decision(2, target_refusal, notices, scan.scripts_read, state.files_read, scan)
        return Decision(0, None, notices, scan.scripts_read, state.files_read, scan)
    lines = []
    if external_refused:
        who = agent_type or "主 agent"
        lines += ([f"✗ 不经 task-server 起要经服务的外部程序被拒（{who}）：{external_refused[0]}"] + [f"  另有：{entry}" for entry in external_refused[1:]]
                  + [subagent_task_server_policy(True) if agent_type else TASK_SERVER_POLICY])
    if unrecognized_refused:
        lines += [f"✗ 认不出起什么的写法被拒：{unrecognized_refused[0]}"] + [f"  另有：{entry}" for entry in unrecognized_refused[1:]]
        lines.append(UNRECOGNIZED_POLICY if agent_type is not None else MAIN_AGENT_RULE_POINTER)
    if in_script:
        lines.append(SCRIPT_POLICY)
    message = "\n".join(lines)
    if target_refusal:
        message = target_refusal + "\n" + message
    if state.missing_cat_sources and not guard_break_is_set("cat-missing-source-unexplained"):
        # cat / tee 的源盘上不在、读不到它的内容，那个目的照认不出拒（偏严：cat 不看源在不在那一类照拒），说明点名这一种写法与改法
        named = "、".join(f"`{source}`（写进 {destination}）" for source, destination in state.missing_cat_sources)
        message += (f"\n→ 这一条里 cat / tee 的源 {named} 盘上不在、也不是同一条命令里前面认出写出的：真跑时 cat 报 No such file、退 1，用 `&&` 接的后一条其实不跑；"
                    "读不到这个源的内容，写出的那份目的照认不出、执行它照拒。要跑的话把不在的源去掉，或把引号写的 `\"2\">` 改成重定向 `2>` 本来想写的样子，再拆成两条命令。")
    return Decision(2, message, notices, scan.scripts_read, state.files_read, scan)


# 交给 task-server 的那一条照抄时：开头这几样变量写在 submit 前面（客户端照白名单带进任务）
SUBMIT_CARRIED_PREFIX_VARIABLES = ("SINGLEFS_RUN_SCOPE",)
LEADING_ASSIGNMENT = re.compile(r"\A\s*([A-Za-z_][A-Za-z0-9_]*)=(?:'[^']*'|\"(?:[^\"\\]|\\.)*\"|\S*)[ \t]+")
LEADING_MEMORY_CAP = re.compile(r"\A(?:bash\s+)?(?:\S*/)?scripts/hosts/run-with-memory-cap\.sh\s+\S+\s+")
SHELL_SYNTAX_MARKS = ("&&", "||", ";", "|", "\n", "$(", "`", "<", ">", "cd ")

def submit_candidate(command):
    """被拒的那条改写成交给 task-server 的样子：开头的范围声明挪到 submit 前面，开头一层 run-with-memory-cap.sh <上限> 去掉
    （交给服务的命令不带包装）；剩下的有 &&、管道、cd、重定向这类 shell 写法的包进 `bash -c '…'`。"""
    carried, rest = [], command.strip()
    while True:
        found = LEADING_ASSIGNMENT.match(rest)
        if not found or found.group(1) not in SUBMIT_CARRIED_PREFIX_VARIABLES:
            break
        carried.append(found.group(0).strip())
        rest = rest[found.end():]
    rest = LEADING_MEMORY_CAP.sub("", rest, count=1)
    inner = f"bash -c {shlex.quote(rest)}" if any(mark in rest for mark in SHELL_SYNTAX_MARKS) else rest
    return " ".join(carried + ["task-server/task submit --", inner])

def task_server_copy_lines(command, directory, agent_type):
    """被拒时放在最前面的照抄一行：改写成经 task-server 提交的那条（submit_candidate）；是要声明范围的验证命令、开头还没带声明的，四段声明写在 submit 前面
    （run_scope_declaration，为什么一段是占位）；照主 agent 交它再判过放行才给，太长的只写怎么改。主 agent 照它提交；子 agent 不交（文件头「经 task-server 起」②），
    照它写进报告「要主 agent 跑的命令」一节交回。弄坏开关 task-server-copy-dropped 下不给。"""
    if guard_break_is_set("task-server-copy-dropped"):
        return []
    candidate = submit_candidate(command)
    declaration = None if candidate.startswith(SUBMIT_CARRIED_PREFIX_VARIABLES[0] + "=") else run_scope_declaration(candidate, directory)
    if declaration is not None:
        candidate = f"{declaration} {candidate}"
    judged = task_server_decision({"tool_name": "Bash", "cwd": directory, "tool_input": {"command": candidate}}, directory, copy=False)
    if judged.code != 0:
        return []
    if agent_type:
        if len(candidate) > COPY_COMMAND_LIMIT:
            return [f"→ 写进报告「要主 agent 跑的命令」一节：这条命令 {len(candidate)} 字，不整条贴；写成 `task-server/task submit -- <原命令>`（范围声明挪到 submit 前面，内存包装去掉），子 agent 自己不交。"]
        return ["→ 写进报告「要主 agent 跑的命令」一节的那一条（子 agent 自己不交，由主 agent 交）：", candidate]
    if len(candidate) > COPY_COMMAND_LIMIT:
        return [f"→ 照抄：这条命令 {len(candidate)} 字，不整条贴；写成 `task-server/task submit -- <原命令>`（范围声明挪到 submit 前面，内存包装去掉），提交之后用 run_in_background 起 `task-server/task wait <号>`。"]
    return ["→ 照抄（提交之后当场打出任务号，再用 run_in_background 起 `task-server/task wait <号>`、结束本轮等完成通知）：", candidate]

def frames_mention_listed(frames, listed, agent_type):
    """这一处出在名单上直接跑的脚本的正文里（谁都能跑的那几份，或主 agent 跑的会话工具）：它整份照名单放行，正文里的几行不另算。"""
    allowed = {path for path, level in listed if level == "anyone" or (level == "main-agent" and agent_type is None)}
    return any(os.path.realpath(path) in allowed for path, _line in frames if path)

def task_server_decision(hook_input, project_root, copy=True):
    """文件头「经 task-server 起」：先判认不出的写法、外部程序与编译目录（scan_decision），再加这三样——① 不经 task-server 跑项目代码的拒，
    经 task-server/task submit 交给服务的不判内存包装；② 子 agent 起 task-server/task 的 submit、wait、cancel、clean 拒；③ 绕开服务的写法拒。
    返回 Decision。弄坏开关 task-server-direct-allowed 下 ① 的直接跑不判，subagent-task-client-allowed 下 ② 不判，
    task-server-bypass-allowed 下 ③ 不判（scan_uses 里不记、文本这一层不看）。"""
    category = scan_decision(hook_input, project_root)
    scan = category.scan
    if scan is None:   # disable-check：整道不判
        return category
    tool_input = hook_input.get("tool_input") if isinstance(hook_input.get("tool_input"), dict) else {}
    command = tool_input.get("command") or ""
    agent_type = hook_input.get("agent_type") or None
    directory = hook_input.get("cwd") or project_root
    notices = list(category.notices)
    guarded, guarded_problem = project_code_runs.task_server_guarded()
    if guarded_problem and not guard_break_is_set("task-server-bypass-allowed") and command:
        notices.append(f"{guarded_problem}：绕开 task-server 的写法这一条没判")
    bypass = [] if guard_break_is_set("task-server-bypass-allowed") else project_code_runs.task_server_text_bypass(command)[0]
    bypass += [f"脚本 {frames_text(item.frames)} 里的 {item.detail}" if item.frames else item.detail for item in scan.bypasses]
    where = lambda run: f"脚本 {frames_text(run.frames)} 里的 {run.detail}" if run.frames else run.detail
    direct = []
    if not guard_break_is_set("task-server-direct-allowed"):
        for run in scan.direct_runs:
            if ((run.allowed == "main-agent" and agent_type is None) or run.allowed in (SUBAGENT_CLIENT_LEVEL, TEMPORARY_SERVICE_BARE_LEVEL)
                    or frames_mention_listed(run.frames, scan.listed_scripts, agent_type)):
                continue
            direct.append(where(run))
    # 要经服务的外部程序（scan_decision 判成 external-program 的）也是直接跑项目之外的程序，照 ① 给照抄的提交写法
    external = any(use.test.kind in ("external-program", "project-code") and not frames_mention_listed(use.frames, scan.listed_scripts, agent_type) for use in scan.uses)
    # ② 子 agent 起提交客户端的 submit、wait、cancel、clean（scan_uses 记成 SUBAGENT_CLIENT_LEVEL 的那几处）；主 agent 不判
    client = []
    if agent_type and not guard_break_is_set("subagent-task-client-allowed"):
        client = [where(run) for run in scan.direct_runs
                  if run.allowed == SUBAGENT_CLIENT_LEVEL and not frames_mention_listed(run.frames, scan.listed_scripts, agent_type)]
    # 文件头「经 task-server 起」④：要临时 prune-server 的自证写成裸的 --selftest（scan_uses 记成 TEMPORARY_SERVICE_BARE_LEVEL，submit 里那一条的也收），谁起都拒
    bare = [] if guard_break_is_set("temporary-service-bare-allowed") else [
        where(run) for run in scan.direct_runs if run.allowed == TEMPORARY_SERVICE_BARE_LEVEL and not frames_mention_listed(run.frames, scan.listed_scripts, agent_type)]
    unique = lambda entries: list(dict.fromkeys(entries))
    bypass, direct, client, bare = unique(bypass), unique(direct), unique(client), unique(bare)
    if not (bypass or direct or client or bare):
        if external and copy and category.message:
            return category._replace(message="\n".join(task_server_copy_lines(command, directory, agent_type) + [category.message]), notices=notices)
        return category._replace(notices=notices)
    lines = []
    if bypass:
        lines += [f"✗ 绕开 task-server 的写法被拒：{bypass[0]}"] + [f"  另有：{entry}" for entry in bypass[1:]] + [TASK_SERVER_BYPASS_POLICY]
    if client:
        lines += ([f"✗ {agent_type} 起统一任务服务的客户端被拒（子 agent 只许 status、log）：{client[0]}"] + [f"  另有：{entry}" for entry in client[1:]]
                  + [subagent_task_server_policy(False)])
    if direct:
        who = agent_type or "主 agent"
        lines += ([f"✗ 不经 task-server 跑项目代码被拒（{who}）：{direct[0]}"] + [f"  另有：{entry}" for entry in direct[1:]]
                  + [subagent_task_server_policy(True) if agent_type else TASK_SERVER_POLICY])
    if (direct or external) and copy and not bypass:
        lines = task_server_copy_lines(command, directory, agent_type) + lines
    if bare:
        lines += [f"✗ 要临时 prune-server 的自证写成了裸的 --selftest 被拒：{bare[0]}"] + [f"  另有：{entry}" for entry in bare[1:]] + ([TEMPORARY_SERVICE_POLICY] if TEMPORARY_SERVICE_POLICY else [])
    if category.code == 2 and category.message:
        lines.append(category.message)   # 认不出的写法、外部程序、编译目录那一段说明照贴
    return Decision(2, "\n".join(lines), notices, category.scripts_read, category.files_read, scan)


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
    return None, (f"project-code-guard.sh 收到的 hook 输入不是 JSON 对象（{why}），这条命令没判、照常执行；"
                  f"开头 {len(head)} 字节（共 {len(raw)} 字节）：{head!r}")

def record_detection(hook_input, findings, detections_path, outcome):
    """检出记录追加一行；outcome 是这条命令的结局：「被拒」（执行前拒了、没执行）或「只记」（照常执行）。"""
    tool_input = hook_input.get("tool_input") if isinstance(hook_input.get("tool_input"), dict) else {}
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
    if not guard_break_is_set("detection-outcome-dropped"):
        entry["outcome"] = outcome
    if not detections_path:   # 读不到 lib_work_root.py、又没设 AGENT_HOOK_DETECTIONS：不知道往哪写
        return
    try:
        os.makedirs(os.path.dirname(detections_path), exist_ok=True)
        with open(detections_path, "a", encoding="utf-8") as handle:
            handle.write(json.dumps(entry, ensure_ascii=False) + "\n")
    except OSError:
        pass

def build_sample_scripts(work):
    """自检用的脚本：被拒的那一条（herd7 x.litmus）写在 s.sh 第 4 行；另有嵌套、只含轻命令、读不到、看不全与不读的几种。"""
    def write(relative, content, mode="w"):
        path = os.path.join(work, relative)
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, mode) as handle:
            handle.write(content)
    write("s.sh", "#!/usr/bin/env bash\n# 验证链：先编，再跑\nset -euo pipefail\nherd7 x.litmus\n")
    write("scripts/mutation/prove-red.sh", "#!/usr/bin/env bash\n# 样本（证红脚本的替身，正文只有一行轻命令）\necho sample\n")
    write("outer.sh", "#!/usr/bin/env bash\necho 起里一层\nbash s.sh\n")
    write("product-gpu.sh", "#!/usr/bin/env bash\n# 样本：产品 GPU 包的真卡对拍\n"
                            "cd product-gpu-sample && cargo test -p singlefs-background-gpu --features tests-on-a-real-gpu -- --test-threads 1\n")
    write("light.sh", "#!/usr/bin/env bash\nbash scripts/hosts/run-with-memory-cap.sh 4G cargo test -p singlefs-core --test core_contract\necho \"herd7 x.litmus\"\n"
                      "cat > note.md <<'EOF'\nherd7 x.litmus\nEOF\nbash scripts/hosts/run-with-memory-cap.sh 4G cargo clippy --all-targets -- -D warnings\n")
    write("loop.sh", "#!/usr/bin/env bash\n[ -n \"${AGAIN:-}\" ] || AGAIN=1 bash loop.sh\n")
    for level in range(1, 8):
        write(f"deep{level}.sh", f"bash deep{level + 1}.sh\n" if level < 7 else "herd7 x.litmus\n")
    write("blob.bin", b"\x00\x01herd7 x.litmus\n", "wb")
    write("big.sh", "# 填充\n" * (MAXIMUM_SCRIPT_BYTES // 8 + 1) + "herd7 x.litmus\n")
    write("tool", "#!/usr/bin/env python3\nimport subprocess\nsubprocess.run(['cargo', 'test', '--all'])\nherd7 x.litmus\n")
    write("prog", b"\x7fELF\x02\x01\x01\x00\x00\nherd7 x.litmus\n", "wb")
    # 超过读的上限的编译产物与 python 程序：先认出不是 shell 脚本、不读也不记，轮不到大小上限（例：e49_chain_width）
    write("bigprog", b"\x7fELF\x02\x01\x01\x00" + b"\x00" * (MAXIMUM_SCRIPT_BYTES + 64) + b"\nherd7 x.litmus\n", "wb")
    write("bigtool", "#!/usr/bin/env python3\n" + "# 填充\n" * (MAXIMUM_SCRIPT_BYTES // 8 + 1) + "herd7 x.litmus\n")
    write("elsewhere/mutate.sh", "#!/usr/bin/env bash\nherd7 x.litmus\n")
    # 路径尾部与仓内位置相同、却不是本仓那一份的拷贝：正文起外部程序，照读正文、照拒
    for copied_entry in ("scripts/experiment-apparatus/replay.sh", "scripts/crash/crash-point-replay.sh", "scripts/crash/crash-amplification.sh"):
        write(f"copy-tree/{copied_entry}", "#!/usr/bin/env bash\nherd7 x.litmus\n")
    # 经符号链接起本仓入口：入口按自己所在的路径（不解符号链接）找旁边的 .claude/scripts/preflight.sh，链接旁边放一份改过的（起 验证包 cargo）
    write("link-tree/.claude/scripts/preflight.sh", "#!/usr/bin/env bash\ncargo test --release -p singlefs-verification --lib --tests\n")
    os.makedirs(os.path.join(work, "link-tree", "scripts", "experiment-apparatus"))
    os.symlink(os.path.join(project_code_runs.HOOK_REPOSITORY, project_code_runs.KNOWN_SCRIPT_LOCATIONS["replay.sh"]),
               os.path.join(work, "link-tree", "scripts", "experiment-apparatus", "replay.sh"))
    write("scripts/gate/gate-staged.sh", "#!/usr/bin/env bash\nbash .claude/scripts/gate.sh --staged\n")
    write(".claude/singlefs-ai-sop/scripts/naming-lint.sh", "#!/usr/bin/env bash\ntrue\n")   # 交回前自查命名的那条命令起的上游 lint：脚本在，读得到
    # 查名字与数组赋值：照 .claude/scripts/fetch-deps.sh 第 44 行、scripts/gate/change-touches-crates.sh 第 61 行原样
    write("command-lookup.sh", "#!/usr/bin/env bash\n  if command -v herd7 >/dev/null 2>&1; then ok \"herd7 已在\"; return 0; fi\n")
    write("array-assignment.sh", "#!/usr/bin/env bash\n  ((${#prefixes[@]})) || prefixes=(\"crates/\")\nchains=(./s.sh ./outer.sh)\n")
    # 命令替换的两种写法，照 .claude/singlefs-ai-sop/scripts/doc-lint.sh 第 43 行、第 70 行原样：切碎之后的片段不许当成脚本路径去读
    write("doc-lint-line-43.sh", "#!/usr/bin/env bash\n  set -- \"$(cd \"$(dirname \"${BASH_SOURCE[0]}\")/..\" && pwd)\"\n")
    write("doc-lint-line-70.sh", "#!/usr/bin/env bash\n"
          "DOC_LINT_FAMILY=\"$(sed -n 's/^family=//p' \"$(dirname \"${BASH_SOURCE[0]}\")/../I18N\" 2>/dev/null || true)\"\n")
    write("substitution-fragments.sh", "#!/usr/bin/env bash\ncd $(dirname $0)/.. && echo $((total/2)) `dirname $0`/x.sh\n"
          "$(dirname \"$0\")/helper.sh --flag\n`dirname $0`/helper.sh\n"
          "out=\"$(bash scripts/hosts/run-with-memory-cap.sh 4G cargo test -p singlefs-core --test core_contract 2>&1)\"\n")
    # 不经内存包装跑一个测试目标：子 agent 起它要拒（经包装起它放行）
    write("unwrapped.sh", "#!/usr/bin/env bash\ncargo test -p singlefs-core --test core_contract\n")
    write("substitution-heavy.sh", "#!/usr/bin/env bash\n  set -- \"$(cd \"$(dirname \"${BASH_SOURCE[0]}\")/..\" && pwd)\"\n"
          "summary=\"$(herd7 x.litmus 2>&1 | tail -3)\"\n")
    # 直接执行时不当脚本读的文本：Rust 源码的文档注释里反引号括着彼此的路径与 `herd7 x.litmus`，当 shell 读就一路互相读下去（一条命令 100 秒判不完）
    checker_sources = ["walk.rs", "image.rs", "lib.rs"]
    for name in checker_sources:
        others = "` 与 `".join(f"crates/singlefs-checker/src/{other}" for other in checker_sources if other != name)
        write(f"crates/singlefs-checker/src/{name}", f"//! 池级 checker：见 `{others}`\n/// 跑法：`herd7 x.litmus`\npub fn check_pool_image() {{}}\n")
    write("notes.md", "# 验证链\nherd7 x.litmus\n")
    write("plain-text", "herd7 x.litmus\n")
    write("2", "echo from-source-two\n")   # 引号数字源测试（cat s.sh "2"> …）用的在的源："2" 是文件名、不是 fd，正常下接起来判（不是 H7 的源不在）
    write(".claude/singlefs-ai-sop/scripts/doc-lint.sh", "#!/usr/bin/env bash\necho doc-lint\n")
    # 直接起照拒的本仓脚本（project_code_use 凭路径尾部在本仓里有这份文件认，盘上不在的不记直接起）：门禁阶段、上游的 lint、门禁结构检查，样本里放一份同名的文本
    for sample_script in (".claude/gate.d/code-source-discipline.sh", ".claude/gate.d/code-tooling.sh", ".claude/gate.d/lkmm.sh",
                          ".claude/singlefs-ai-sop/scripts/gate-lint.sh", ".claude/singlefs-ai-sop/scripts/hooks-registered.sh", "scripts/gate/gate-structure-check.py"):
        write(sample_script, "#!/usr/bin/env python3\nprint('样本')\n" if sample_script.endswith(".py") else "#!/usr/bin/env bash\necho 样本\n")
    # 直接执行时当脚本读的：没扩展名而 `#!` 指到 shell 的，扩展名不对而 `#!` 指到 shell 的
    write("runner", "#!/usr/bin/env bash\nherd7 x.litmus\n")
    write("mislabeled.py", "#!/bin/sh\nherd7 x.litmus\n")
    # 一条命令里同一份脚本起几遍：twice.sh 三种起法各起一遍 s.sh；cyc-a.sh 与 cyc-b.sh 互相调；rel.sh 按相对路径起的脚本随当前目录变
    write("twice.sh", "#!/usr/bin/env bash\nbash s.sh\n./s.sh\nsource s.sh\n")
    write("cyc-a.sh", "bash cyc-b.sh\nherd7 x.litmus\n")
    write("cyc-b.sh", "bash cyc-a.sh\n")
    write("rel.sh", "bash ./inner-rel.sh\n")
    write("inner-rel.sh", "echo 轻\n")
    write("sub/inner-rel.sh", "herd7 x.litmus\n")
    write("has-missing.sh", "bash missing.sh\n")
    write("outer-missing.sh", "echo 起里一层\nbash has-missing.sh\n")
    # BASH_ENV / ENV 指的启动文件：正文只有一行重活的、只有轻活的
    write("startup-heavy.sh", "bash scripts/hosts/run-with-memory-cap.sh 16G herd7 x.litmus\n")
    write("startup-light.sh", "echo hello\n")
    # 直接执行的脚本先读的启动文件：#! 指到 bash、env bash、sh、python、认不出的解释器、zsh 的，和没有 #! 的
    for name, text in (("exec-bash.sh", "#!/bin/bash\necho hello\n"), ("exec-env-bash.sh", "#!/usr/bin/env bash\necho hello\n"), ("exec-sh.sh", "#!/bin/sh\necho hello\n"),
                       ("exec-python.py", "#!/usr/bin/env python3\nprint(1)\n"), ("exec-unknown.sh", "#!/opt/odd/interp\nhello\n"),
                       ("exec-zsh.sh", "#!/bin/zsh\necho hello\n"), ("exec-no-shebang.sh", "echo hello\n")):
        write(name, text)
    # 交互的 bash 读 HOME 下的 .bashrc，zsh 读 ZDOTDIR（没设取 HOME）下的 .zshenv 与 .zshrc
    write("home-heavy/.bashrc", "herd7 x.litmus\n")
    write("home-light/.bashrc", "echo hello\n")
    write("zdot-env/.zshenv", "bash scripts/hosts/run-with-memory-cap.sh 16G herd7 x.litmus\n")
    write("zdot-env/.zshrc", "echo hello\n")
    write("zdot-rc/.zshenv", "echo hello\n")
    write("zdot-rc/.zshrc", "herd7 x.litmus\n")
    write("zdot-light/.zshenv", "echo hello\n")
    write("zdot-light/.zshrc", "echo hello\n")

# 一条命令 100 秒判不完的写法：双引号里没转义的反引号把 `crates/singlefs-checker/src/walk.rs` 当命令替换执行（原样，cwd 换成自检的临时工作区）
SLOW_REPRODUCTION_COMMAND = 'python3 scripts/kb/replace-once.py .claude/kb/verification-build.md "池级 checker（\\`crates/singlefs-checker\\` 的 \\`walk::check_pool_image\\`，24 条）" "池级 checker（\\`crates/singlefs-checker\\` 的 \\`walk::check_pool_image\\`，26 条：第一版 23 条加 I-3.8（实例表行唯一且低于挂载根）、I-7.4（近 K 代块未被复用）、I-4.8（近 K 代根校验和自洽））" && python3 scripts/kb/replace-once.py .claude/kb/milestone/02-second-txn.md "checker 新接的只有 I-3.8（实例表行唯一且低于挂载根），I-3.1（已分配统计对得上） 与 I-2.1（校验和与内容匹配） 改按回退候选集判，预想清单里其余各条（I-7.4（近 K 代块未被复用）、I-4.8（近 K 代根校验和自洽） 那一族起）还没有 checker；" "checker 新接了 I-3.8（实例表行唯一且低于挂载根）、I-7.4（近 K 代块未被复用）、I-4.8（近 K 代根校验和自洽）（后两条按候选集里每条根各判一格：走读它引用的单元时校验和对不上或头用不了就红，坏镜像是 mkfs 树表单元——只有第 0 代根还引用——改内容重封 / 抹成零；`crates/singlefs-checker/src/walk.rs`、`image.rs` 清单 26 条），I-3.1（已分配统计对得上） 与 I-2.1（校验和与内容匹配） 改按回退候选集判；预想清单里其余各条（I-3.4（可用空间扣待删占用） 起）还没有 checker；" && bash .claude/singlefs-ai-sop/scripts/doc-lint.sh . 2>&1 | tail -1'
SLOW_REPRODUCTION_SECONDS = 1.0  # 复现输入那一条要在这么多秒内判完

class LayerRun(NamedTuple):
    returncode: int    # 同钩子入口的退出码：0 放行，2 拒绝
    stderr: str        # 拒绝说明（放行是空串）

def category_layer_run(hook_input, project_root, environment_changes, environment_removed=()):
    """自证里在进程中照一格的环境判认不出的写法、外部程序、编译目录那一层（scan_decision）：环境变量照 environment_changes 设、environment_removed 去掉，判完原样还回去。"""
    saved = dict(os.environ)
    try:
        for name in environment_removed:
            os.environ.pop(name, None)
        os.environ.update(environment_changes)
        decision = scan_decision(hook_input, project_root)
    finally:
        os.environ.clear()
        os.environ.update(saved)
    return LayerRun(decision.code, decision.message or "")

def selftest(hook_dir):
    os.environ[SELFTEST_MARK_VARIABLE] = SELFTEST_MARK_CONTENT   # 自证起的钩子子进程（环境原样继承）靠它认：弄坏开关只在自证里认
    if project_code_runs is None:
        print(f"  ✗ 自检：{shared_library_problem(hook_dir)}")
        print("    → 怎么办：恢复 .claude/hooks/lib_project_code_runs.py 与 lib_shell_words.py（判定与切词只有那两份，别在 hook 里再抄一份），再跑 --selftest")
        return 1
    work = tempfile.mkdtemp(prefix="project-code-guard-")
    results = []
    # 登记表那几格起的是本仓里那几份脚本（键按解析后的真实路径认）；喂 stage-selftest.sh 的样本拿临时样本仓的暂存区比（lib_project_code_runs 的 SAMPLE_INDEX_REPOSITORY）
    repo, real_sample_index = project_code_runs.HOOK_REPOSITORY, project_code_runs.SAMPLE_INDEX_REPOSITORY
    try:
        project_code_runs.build_sample_workspace(work)
        project_code_runs.FAMILY_OWN_REPOSITORY = work   # 样本工作区里的门禁阶段、同族脚本照「本仓那一份」认（lib_project_code_runs.py 的 family_own_copy）
        shell_words.SYSTEM_STARTUP_DIRECTORY = os.path.join(work, "system-startup-empty")   # 系统级启动文件不读本机的 /etc（换成样本目录）
        project_code_runs.SAMPLE_INDEX_REPOSITORY = os.path.join(work, project_code_runs.SAMPLE_INDEX_REFERENCE_NAME)
        build_sample_scripts(work)
        writer, crash, triage, scribe = "implementation-writer", "crash-verifier", "gate-triage", "kb-scribe"
        tier_binary = "./target/release/deps/crash_enumeration_new_pool_file_creation_stream-0123456789abcdef"
        # (说明, agent_type（None 是主 agent）, 命令, 该拒 2 / 该放 0[, 该记几条检出, 该读进去几份脚本]；后两列不写是 0、0)
        cases = [
            ("实现员跑 herd7 x.litmus", writer, "herd7 x.litmus", 2),
            # env 换目录（-C 目录、--chdir=目录、--chdir 目录）照 cd 判（lib_shell_words.py 的 LIB_SHELL_WORDS_BREAK=env-chdir-ignored 下前五格红：应当是 2，实际 0；放行那一格红：应当是 0，实际 2；
            # 第二到四格先 cd 进 验证包目录，所以不跟 env 换的目录就放过去）
            # env 的短选项合写（-iC 目录）与 -S 拆串（lib_shell_words.py 的 env_option / env_split_string；LIB_SHELL_WORDS_BREAK=env-option-clusters-unparsed 下这三格红，
            ("实现员 cd 到变量目录、经内存包装起 验证包 -p：放行", writer,
             f"D={work}; cd $D && bash scripts/hosts/run-with-memory-cap.sh 12G cargo test -p singlefs-verification --lib", 0),
            # 目录写在变量里、不带 -p 的裸 cargo test（lib_shell_words.py 的 expand_known_variables；LIB_SHELL_WORDS_BREAK=directory-variables-unexpanded 下照拒的五格红：
            ("实现员 D=… 之后 cd $D/验证包目录经内存包装裸跑：放行", writer,
             f"D={work}; cd $D/crates/singlefs-verification && bash scripts/hosts/run-with-memory-cap.sh 12G cargo test --lib", 0),
            ("实现员交回前自查命名：naming-lint.sh：放行", writer, "bash .claude/singlefs-ai-sop/scripts/naming-lint.sh .", 0, 0, 1),   # 末两列：不记检出、读进去 naming-lint.sh 那一份
            # 装置的本体与 cargo run --bin 同判（lib_project_code_runs.py 的 device_main_binary；device-main-as-test 下头一格红：应当是 0，实际 2）
            ("实现员在 bash -c 里跑 --all", writer, "bash -c 'herd7 x.litmus'", 2),
            ("实现员 cd 进 验证包裸跑、经内存包装放行（验证包随时跑）", writer, "cd crates/singlefs-verification && bash ../../scripts/hosts/run-with-memory-cap.sh 4G cargo test --release", 0),
            ("实现员 -p 验证包不挑目标、经内存包装放行", writer, "bash scripts/hosts/run-with-memory-cap.sh 4G cargo test -p singlefs-verification", 0),
            ("实现员起 qemu-system", writer, "qemu-system-x86_64 -enable-kvm -m 512", 2),
            ("实现员跑 herd7", writer, "herd7 -conf linux-kernel.cfg litmus/a.litmus", 2),
            ("实现员直接跑 herd7 -version 只取版本号：放行", writer, "herd7 -version", 0),
            ("实现员直接跑 herd7 -version 再带 litmus 文件：照拒", writer, "herd7 -version litmus/a.litmus", 2),
            # 只打工具版本的谁都能跑（tool-versions-counted 下末两格红）
            ("实现员经内存包装起循环里 ls 出来的 验证包测试二进制：命令词认不出照拒", writer,
             "b=$(ls -t ./target/release/deps/overwrite_in_one_instance-* | head -1); bash scripts/hosts/run-with-memory-cap.sh 4G $b", 2),
            ("实现员喂给 bash 的 heredoc 里跑 --all", writer, "bash <<'EOF'\nherd7 x.litmus\nEOF", 2),
            # here-string 与管道喂给 shell 的字面文本当命令判，喂进去的是变量、cat 文件这类的放行并记检出（lib_shell_words.py 的 shell_standard_input；
            ("实现员 here-string 喂给 bash 的是变量：认不出照拒、照记检出", writer, 'bash <<< "$CMD"', 2, 1, 0),
            ("实现员 cat 文件经管道喂给 bash：放行并记检出", writer, "cat steps.sh | bash", 0, 1, 0),
            ("实现员经内存包装 here-string 喂给 bash 跑 验证包的目标：放行", writer,
             "bash scripts/hosts/run-with-memory-cap.sh 4G bash <<< 'cargo test -p singlefs-verification --test overwrite_in_one_instance'", 0),
            # 包在外面照样起那条命令的（/usr/bin/time、flock、systemd-run……）剥掉之后照判；里面经内存包装的算经包装
            ("实现员 /usr/bin/time -v 包一层跑 --all", writer, "/usr/bin/time -v herd7 x.litmus", 2),
            ("实现员 perf stat 包一层跑 --all", writer, "perf stat -e cycles herd7 x.litmus", 2),
            ("实现员内存包装里 /usr/bin/time 包一层跑自己的测试目标", writer,
             "bash scripts/hosts/run-with-memory-cap.sh 4G /usr/bin/time -v cargo test -p singlefs-core --test core_contract", 0),
            ("实现员 /usr/bin/time 包在内存包装外面跑自己的测试目标", writer,
             "/usr/bin/time -v bash scripts/hosts/run-with-memory-cap.sh 4G cargo test -p singlefs-core --test core_contract", 0),
            ("主 agent time -p 起全量照拒", None, "time -p herd7 x.litmus", 2),
            ("主 agent time -- 起全量照拒", None, "time -- herd7 x.litmus", 2),
            ("主 agent builtin source 读进全量脚本照拒", None, "builtin source s.sh", 2, 0, 1),
            ("主 agent bash --rcfile 起全量脚本照拒", None, "bash --rcfile /dev/null s.sh", 2, 0, 1),
            ("主 agent bash -- 起全量脚本照拒", None, "bash -- s.sh", 2, 0, 1),
            ("主 agent bash - 起全量脚本照拒", None, "bash - s.sh", 2, 0, 1),
            ("主 agent function 关键字定义的函数里起全量照拒", None, "function f { herd7 x.litmus; }; f", 2),
            ("主 agent cp X{,.bak} 拷出再起全量脚本照拒", None, "cp s.sh{,.bak} && bash s.sh.bak", 2, 0, 1),
            ("主 agent cp 之后按 // 开头的路径起全量脚本照拒", None, f"cp s.sh s-slash.sh && bash /{work}/s-slash.sh", 2, 0, 1),
            ("主 agent env -- 带不合变量名的赋值起全量照拒", None, "env -- a-b=1 herd7 x.litmus", 2),
            ("主 agent env =1 开头就是 = 的赋值起全量照拒", None, "env =1 herd7 x.litmus", 2),
            ("主 agent bash -c -- 的代码里起全量照拒", None, "bash -c -- 'herd7 x.litmus'", 2),
            ("builtin 后面不是内建：bash 报错不跑，放行", None, "builtin herd7 x.litmus", 0),
            ("function 只定义不调用：函数体不跑，放行", None, "function f { herd7 x.litmus; }", 0),
            # 不带 function 关键字的只定义函数同样不判函数体（LIB_SHELL_WORDS_BREAK=posix-uncalled-functions-run 与 uncalled-functions-run 下这一格红）
            ("名字() { 函数体 } 只定义不调用：函数体不跑，放行", None, "f() { herd7 x.litmus; }", 0),
            ("bash -s -- 后面的词是位置参数、不是脚本：放行", None, "bash -s -- s.sh", 0, 0, 0),
            ("主 agent function 定义之后经命令替换调用照拒", None, "function f { herd7 x.litmus; }; $(echo f)", 2),
            ("主 agent function 定义之后引号拆开拼写调用照拒", None, 'function foo { herd7 x.litmus; }; "fo"o', 2),
            ("主 agent function 定义之后花括号拆开拼写调用照拒", None, "function foo { herd7 x.litmus; }; fo{o,}", 2),
            ("主 agent function 定义之后反斜杠拆开拼写调用照拒", None, "function foo { herd7 x.litmus; }; fo\\o", 2),
            #   与 PROJECT_CODE_GUARD_BREAK=cat-concatenation-unseen 下第二格红，redirect-fd-duplication-unseen 下第三格红，各连着「：记几条检出」「：读进去几份脚本」）
            ("主 agent cat 01> 拷出再起全量脚本照拒", None, "cat s.sh 01> s-fd01.sh && bash s-fd01.sh", 2, 0, 1),
            ('主 agent cat 源 "2"> 拷出（引号里的 2 是源）再起全量脚本照拒', None, 'cat s.sh "2"> s-fdq.sh ; bash s-fdq.sh', 2, 0, 1),
            ("主 agent cat 3> 再 >&3 拷出再起全量脚本照拒", None, "cat s.sh 3>s-fd3.sh >&3; bash s-fd3.sh", 2, 0, 1),
            #   （LIB_SHELL_WORDS_BREAK=redirect-target-braces-expanded 下第二格与它的「：记几条检出」「：读进去几份脚本」红；起的那一份不在，记一条检出）
            ("别的命令带几个文件参数写到 > 文件不当 cat 接起来：认不出写法写出再起，照拒", None, "wc -l s.sh light.sh > s-wc.sh; bash s-wc.sh", 2, 0, 0),
            ("重定向目标花括号展开成几个词：bash 报 ambiguous redirect 不写，放行", None, "cat s.sh > s-amb{,}.sh; bash s-amb.sh", 0, 1, 0),
            #   redirect-fd-duplication-unseen 下第二格红，各连着「：记几条检出」「：读进去几份脚本」；放行的那一格起的那一份不在，记一条检出）
            ("heredoc 写文件：cat <<EOF 2> 写的那一份拿不到正文：认不出写法写出再起，照拒", None, "cat <<'EOS' 2>s-hd2.sh\nherd7 x.litmus\nEOS\nbash s-hd2.sh", 2, 0, 0),
            ("主 agent heredoc 写文件：cat <<EOF 3> 再 >&3 写出再起全量脚本照拒", None, "cat <<'EOS' 3>s-hd3.sh >&3\nherd7 x.litmus\nEOS\nbash s-hd3.sh", 2, 0, 1),
            # bash -c 之后照 bash 只认短选项、-n 只查语法；env 认到赋值之后不再认选项（shell-long-options-after-short 下第一格红，
            #   shell-syntax-only-code-run 下第二格红，env-options-after-assignment 下第四格红；第三、五格钉住长选项写在前面、选项写在赋值前面的照判）
            ("bash -c 之后的长选项：bash 报错不跑，放行", None, "bash -c --posix 'herd7 x.litmus'", 0),
            ("bash -c -n 只查语法：放行", None, "bash -c -n 'herd7 x.litmus'", 0),
            ("主 agent bash 长选项写在 -c 前面照拒", None, "bash --posix -c 'herd7 x.litmus'", 2),
            ("env =1 之后的 -u 是命令词：env 报错不跑，放行", None, "env =1 -u X herd7 x.litmus", 0),
            ("主 agent env -u 写在赋值前面照拒", None, "env -u X =1 herd7 x.litmus", 2),
            # 带步长、两端宽度不一的补零序列照 bash 补到长的那一边（brace-sequence-width-padded-ends-only 下这一格与它的「：读进去几份脚本」红）
            ("主 agent cp 带步长、两端宽度不一的补零序列拷出再起全量脚本照拒", None, "cp s.sh s{01..100..100}.sh && bash s001.sh", 2, 0, 1),
            # ANSI-C 引号 $'…' 里的 \' 不收尾（ansi-c-quotes-unseen 下这一格红）
            ("主 agent $'…' 里带 \\' 之后 bash <<EOF 里起全量照拒", None, "echo $'a\\'b'; bash <<'EOF'\nherd7 x.litmus\nEOF", 2),
            # ANSI-C 引号解开之后，定义里的 \t 隔开了测试目标名：数组元素赋值与 += 追加照 bash 是赋值、不是命令词（LIB_SHELL_WORDS_BREAK=extended-assignments-as-command 下这一格红）
            ("数组元素赋值与 += 追加是赋值、不是命令词：放行", writer,
             "row=$'x\\tcrash_enumeration_new_pool_file_creation_stream'; seen[$row]=1; rows+=($row)", 0),
            #   第三、四格钉住一项的命令与模式里的命令替换照判）
            ("case 里 ;; 之后的模式词不当命令：放行", None, 'case "$x" in a) echo a;; herd7) echo h;; esac', 0),
            ("主 agent case 一项的命令里起全量照拒", None, 'case "$x" in a) herd7 x.litmus;; esac', 2),
            ("主 agent case 的模式里的命令替换起全量照拒", None, 'case "$x" in $(herd7 x.litmus)) :;; esac', 2),
            ("[[ ]] 里 && 之后的词不当命令：放行", writer, "[[ -f README.md && herd7 == x ]] && echo y", 0),
            ("主 agent time -p 普通命令放行", None, "time -p ls", 0),
            ("echo 花括号展开出来的词不当命令：放行", None, "echo cargo{,-x} test --all", 0),
            ("赋值词里的花括号不展开：放行", None, "a={x,y}; echo $a", 0),
            # eval 的字面代码照一条命令判（LIB_SHELL_WORDS_BREAK=eval-unread 下这一格红）
            ("主 agent eval 字面代码里起全量照拒", None, "eval 'herd7 x.litmus'", 2),
            # 同一条命令里先 cp 再 mv 拷出再起（PROJECT_CODE_GUARD_BREAK=copy-by-cp-only 下它的「：读进去几份脚本」红；第四十三节第 2 行）
            ("主 agent 同一条命令里 cp 再 mv 改名再起全量脚本照拒", None, "cp s.sh s-copy.sh && mv s-copy.sh s-moved.sh && bash s-moved.sh", 2, 0, 1),
            # cat 源 > 目的 后面另有 2> 的、ln -s 的相对源（报告 W3 一节；LIB_SHELL_WORDS_BREAK=redirect-fd-dropped 下前两格红，link-source-from-current-directory 下后两格红，
            #   各连着的「：记几条检出」「：读进去几份脚本」也红；放行的两格起的那一份不在，记一条检出）
            ("主 agent cat > 拷出、另把 stderr 丢进 /dev/null 再起全量脚本照拒", None, "cat s.sh > s-cat.sh 2>/dev/null && bash s-cat.sh", 2, 0, 1),
            ("cat > 拷出、起的是 2> 写的那一份：认不出写法写出再起，照拒", None, "cat s.sh > s-cat2.sh 2> s-err.sh && bash s-err.sh", 2, 0, 0),
            ("主 agent ln -s 的相对源从链接所在的目录算、经它起全量脚本照拒", None, "mkdir -p sub && ln -s ../s.sh sub/s-link.sh && bash sub/s-link.sh", 2, 0, 1),
            ("ln -s 的相对源在链接所在的目录里不在：认不出写法写出再起，照拒", None, "mkdir -p sub && ln -s s.sh sub/s-link2.sh && bash sub/s-link2.sh", 2, 0, 0),
            # heredoc 挂在没收尾的命令替换、进程替换里的 bash 上（第四十节第 18 行；LIB_SHELL_WORDS_BREAK=heredoc-open-substitution-unseen 下前两格红）
            ("主 agent $(bash <<EOF) 里起全量照拒", None, "x=$(bash <<'EOF'\nherd7 x.litmus\nEOF\n)", 2),
            ("主 agent <(bash <<EOF) 里起全量照拒", None, "cat <(bash <<'EOF'\nherd7 x.litmus\nEOF\n)", 2),
            ("$(cat <<EOF) 里的正文是数据：放行", None, "x=$(cat <<'EOF'\nherd7 x.litmus\nEOF\n)", 0),
            # 花括号展开出来的空串照 bash 去掉、整数序列照 bash 补零（LIB_SHELL_WORDS_BREAK=brace-empties-kept 下前四格红，
            #   brace-sequence-unpadded 下第五格与它的「：读进去几份脚本」红）
            ("主 agent {,} 写在命令词前面起全量照拒", None, "{,} herd7 x.litmus", 2),
            ("主 agent cp 补零的序列拷出再起全量脚本照拒", None, "cp s.sh s{01..01}.sh && bash s01.sh", 2, 0, 1),
            ("引号写出的空串花括号展开之后留着、当命令词：放行", None, "''{,} herd7 x.litmus", 0),
            # 同一行里先有双引号包着、已收尾的 $(…)，再接顶层的 bash <<EOF（报告 W2 一节；LIB_SHELL_WORDS_BREAK=double-quoted-substitution-unclosed 下这两格红）
            ('主 agent cd "$(git rev-parse --show-toplevel)" 之后 bash <<EOF 里起全量照拒', None,
             "cd \"$(git rev-parse --show-toplevel)\" && bash <<'EOF'\nherd7 x.litmus\nEOF", 2),
            ('主 agent FOO="$(pwd)" bash <<EOF 里起全量照拒', None, "FOO=\"$(pwd)\" bash <<'EOF'\nherd7 x.litmus\nEOF", 2),
            # 起测试的外部子命令、别名与 runner、只列不跑、没标 #[ignore] 的登记用例
            ("实现员 cargo mutants 整个 验证包（随时跑）", writer, "bash scripts/hosts/run-with-memory-cap.sh 4G cargo mutants -p singlefs-verification", 0),
            # 喂的样本与暂存区比（样本的 setup.sh 起 验证包；LIB_PROJECT_CODE_RUNS_BREAK=stage-samples-fixtures-unchecked 下这一格红）
            # 登记表的键按真实路径认（LIB_PROJECT_CODE_RUNS_BREAK=selftest-registry-by-path-tail 下第一格红）
            ("工具实现员起带 case 模式行 .claude/gate.d/lib/*.py) 的脚本：放行", "tooling-writer", "bash case-pattern.sh", 0, 0, 1),
            # 只查清单、只生成、只读、只查语法的放行（lib_shell_words.py 的 LIB_SHELL_WORDS_BREAK=arguments-as-executed 下最后两格红；
            ("实现员 gate-overlap.py --list 查清单", writer, "python3 .claude/singlefs-ai-sop/scripts/gate-overlap.py --list", 0),
            ("实现员看门禁阶段的格名表", writer, "bash .claude/gate.d/code-tooling.sh --list", 0),
            ("书记员跑阶段的 --write（生成）", scribe, "bash .claude/gate.d/doc-decisions.sh --write", 0),
            ("工具实现员在门禁目录里 python3 - 读几份门禁（2026-09-28 误拦的那一条原样，目录换成自检的临时工作区）", "tooling-writer",
             f"cd {work}/.claude/gate.d && python3 - doc-text.sh doc-process-records.sh doc-experiments.sh <<'EOF'\nimport re, sys\n"
             "for path in sys.argv[1:]:\n    text = open(path, encoding=\"utf-8\").read()\nEOF", 0),
            # 双机分片的驱动按树根里键那一行判（lib_project_code_runs.py 的 shard-driver-by-name 打开时第一格红）
            ("实现员跑自己动到的测试目标", writer,
             "bash scripts/hosts/run-with-memory-cap.sh 4G cargo test -p singlefs-verification --test overwrite_in_one_instance", 0),
            ("实现员 clippy --all-targets", writer, "bash scripts/hosts/run-with-memory-cap.sh 8G cargo clippy --all-targets -- -D warnings", 0),
            ("实现员 build --all-targets", writer, "bash scripts/hosts/run-with-memory-cap.sh 8G cargo build --offline --all-targets", 0),
            ("实现员 echo 引号里的命令", writer, 'echo "herd7 x.litmus"', 0),
            ("实现员写进笔记的 heredoc 正文", writer, "cat > note.md <<'EOF'\nherd7 x.litmus\nEOF", 0),
            ("实现员 -p harness --lib", writer, "bash scripts/hosts/run-with-memory-cap.sh 4G cargo test -p singlefs-verification --lib", 0),
            ("执行员在 research 里跑自己的 bin", "experiment-runner",
             "cd scripts/experiment-apparatus && bash ../hosts/run-with-memory-cap.sh 16G cargo test --release --bin e160-random-small-read-share", 0),
            # 跑编译出来的代码经没经内存包装（与它认的哪一类无关）：子 agent 不经它的拒，经它包着的（连同 bash -c、它起的脚本）放行，主 agent 不判
            ("实现员经内存包装再经 capped.sh", writer, "bash scripts/hosts/run-with-memory-cap.sh 4G bash scripts/hosts/capped.sh 4 cargo test -p singlefs-core --lib", 0),
            ("实现员 capped.sh 在外、内存包装在里", writer, "capped.sh 4 run-with-memory-cap.sh 4G cargo test -p singlefs-core --lib", 0),
            ("实现员 timeout 套在内存包装外面", writer, "timeout 900 bash scripts/hosts/run-with-memory-cap.sh 4G cargo test -p singlefs-core --lib", 0),
            ("实现员内存包装里 bash -c 跑", writer, "bash scripts/hosts/run-with-memory-cap.sh 4G bash -c 'cd crates/singlefs-core && cargo test --lib'", 0),
            ("实现员经内存包装直接执行测试二进制", writer, "bash scripts/hosts/run-with-memory-cap.sh 4G ./target/debug/deps/core_contract-0123456789abcdef", 0),
            ("实现员只编不跑 cargo test --no-run", writer, "bash scripts/hosts/run-with-memory-cap.sh 8G cargo test --no-run -p singlefs-core", 0),
            ("实现员经内存包装起那份脚本", writer, "bash scripts/hosts/run-with-memory-cap.sh 4G bash unwrapped.sh", 0, 0, 1),
            ("实现员起的脚本里有 cargo test、这一层放行：读正文", writer, "bash unwrapped.sh", 0, 0, 1),
            ("实现员 heredoc 写出 cargo run 再起它、这一层放行：读正文", writer,
             "cat > gen-unwrapped.sh <<'EOF'\ncargo run --release --bin e160-random-small-read-share\nEOF\nbash gen-unwrapped.sh", 0, 0, 1),
            ("起编译：主 agent 裸起 cargo fmt 与 cargo metadata：放行", None, "cargo fmt --all && cargo metadata --format-version 1 --no-deps", 0),
            ("起编译：实现员裸起 cargo fmt 与 cargo metadata：放行", writer, "cargo fmt --all && cargo metadata --format-version 1 --no-deps", 0),
            ("实现员 bash 起含它的脚本", writer, "bash s.sh", 2, 0, 1),
            ("实现员 ./ 直接执行含它的脚本", writer, "./s.sh", 2, 0, 1),
            ("实现员经 capped.sh 4 bash 起含它的脚本", writer, "capped.sh 4 bash s.sh", 2, 0, 1),
            ("实现员 source 含它的脚本", writer, "source s.sh", 2, 0, 1),
            ("实现员 . 读进含它的脚本、nice 与 timeout 包一层", writer, "nice -n 19 timeout 600 . s.sh", 2, 0, 1),
            ("实现员 cd 之后按绝对路径起含它的脚本、输出落文件", writer, f"cd {work} && bash {work}/s.sh > {work}/chain.log 2>&1", 2, 0, 1),
            ("实现员起的脚本里再调含它的脚本", writer, "bash outer.sh", 2, 0, 2),
            ("主 agent 执行含它的脚本", None, "bash s.sh", 2, 0, 1),
            ("实现员起调回自己的脚本（只读一遍）", writer, "bash loop.sh", 0, 0, 1),
            ("实现员 capped.sh 后面不带斜杠的命令词从 PATH 找，不读当前目录的同名脚本", writer, "bash scripts/hosts/capped.sh 4 s.sh", 0, 0, 0),
            ("实现员 ./ 执行 python 程序不读（射程之外）", writer, "./tool", 0, 0, 0),
            ("实现员 ./ 执行编译出来的程序不读", writer, "./prog", 0, 0, 0),
            ("实现员 ./ 执行超过读的上限的编译产物：不读也不记", writer, "./bigprog", 0, 0, 0),
            ("实现员 ./ 执行超过读的上限的 python 程序：不读也不记", writer, "./bigtool", 0, 0, 0),
            ("实现员 ./ 执行超过读的上限的 shell 脚本：放行并记检出", writer, "./big.sh", 0, 1, 0),
            ("实现员起 doc-lint.sh 第 43 行那种命令替换：片段不当路径", writer, "bash doc-lint-line-43.sh", 0, 0, 1),
            ("实现员起 doc-lint.sh 第 70 行那种命令替换：片段不当路径", writer, "bash doc-lint-line-70.sh", 0, 0, 1),
            ("实现员起的脚本里命令替换、反引号、算术后面接路径：片段不当路径", writer, "bash substitution-fragments.sh", 0, 1, 1),   # `$(dirname "$0")/helper.sh` 换成脚本所在目录之后照路径认，helper.sh 不在记一条
            ("实现员起的脚本里命令替换里跑 --all", writer, "bash substitution-heavy.sh", 2, 0, 1),
            ("实现员起的脚本里 command -v herd7：查名字不是跑", writer, "bash command-lookup.sh", 0, 0, 1),
            ("实现员起的脚本里数组赋值的元素：不当命令、不当路径", writer, "bash array-assignment.sh", 0, 0, 1),
            ("实现员直接跑 command -v herd7", writer, "command -v herd7 >/dev/null && echo 在", 0),
            ("实现员 command herd7（不带 -v）照拒", writer, "command herd7 -conf linux-kernel.cfg litmus/a.litmus", 2),
            ("实现员 bash 起一个目录：不读也不记", writer, "bash crates", 0),
            # 进程替换 `<(…)`：括号里的命令另按一条完整的命令判，`)` 之后的词是外层命令的参数（第一例是现场误拒的原句）
            ("实现员同一行里起 .sh 之后把 python 源码喂给 python3 -：正文不当命令", writer,
             "sed -n 1p light.sh > body.py && bash light.sh && python3 - <<'PY'\nsource = open(\"body.py\", encoding=\"utf-8\").read()\n"
             "body = open(\"/tmp/claude-1000/nowhere/hook_body.py\", encoding=\"utf-8\")\nusage = \"\"\"\nherd7 x.litmus\n\"\"\"\nPY", 0, 0, 1),
            # 喂给 bash 的 heredoc 里出现 python 那种写法：`source =` 不算读脚本，带逗号的词不当路径
            ("实现员喂给 bash 的正文里有 source = 与带逗号的路径：不当脚本", writer,
             "bash light.sh; bash <<'EOF'\nsource = open(\"body.py\", encoding=\"utf-8\").read()\n"
             "body = open(\"/tmp/claude-1000/nowhere/hook_body.py\", encoding=\"utf-8\")\nEOF", 0, 0, 1),
            ("实现员 bash 起通配写出来的脚本路径：不当路径、不记检出", writer, "bash ./chain-*.sh", 0, 0, 0),
            # pattern-expansions-bare 下「写成通配的 capped.sh」那一格红）
            ("实现员写成通配的 capped.sh 包全量：照拒", writer, f"cd {repo} && bash scripts/hosts/capped.s? 4 herd7 x.litmus", 2),
            ("实现员合法的通配 ls scripts/*.sh：放行", writer, f"cd {repo} && ls scripts/*.sh", 0, 0, 0),
            # 方括号里的 POSIX 字符类、等价类、整理符号交给 bash 展开（LIB_PROJECT_CODE_RUNS_BREAK=posix-brackets-by-python-glob 下前四格红，
            # posix-brackets-unresolved-literal 下第四格红）
            ("实现员合法的 POSIX 字符类 ls scripts/[[:alpha:]]*.sh：放行", writer, f"cd {repo} && ls scripts/[[:alpha:]]*.sh", 0, 0, 0),
            # two-logs.sh 包一层不让里面那条脱离闸（LIB_PROJECT_CODE_RUNS_BREAK=two-logs-opaque 下「实现员经 two-logs.sh 起全量 cargo test：照拒」红）
            ("实现员经 two-logs.sh 起全量 cargo test：照拒", writer,
             f"bash {repo}/scripts/hosts/two-logs.sh /tmp/claude-1000/somewhere/log -- herd7 x.litmus", 2),
            # 前两格与「花括号写在包装上」红，brace-command-words-bare 下「花括号写在包装上」红）
            ("实现员合法的花括号 ls scripts/{a,b}.sh：放行", writer, f"cd {repo} && ls scripts/{{a,b}}.sh", 0, 0, 0),
            ("实现员合法的参数通配 cat crates/*.toml：放行", writer, f"cd {repo} && cat crates/*.toml", 0, 0, 0),
            ("实现员起不存在的脚本：放行并记检出", writer, "bash missing.sh", 0, 1, 0),
            ("实现员 ./ 执行不存在的脚本：放行并记检出", writer, "./missing.sh", 0, 1, 0),
            ("实现员 bash 起不是文本的文件：放行并记检出", writer, "bash blob.bin", 0, 1, 0),
            ("实现员起超过上限的脚本：放行并记检出", writer, "bash big.sh", 0, 1, 0),
            ("实现员起的脚本嵌套 7 层：读到第 6 层、看不全，放行并记检出", writer, "bash deep1.sh", 0, 1, 6),
            ("实现员 cat > 文件 <<EOF 写出含 --all 的脚本再 bash 起它", writer,
             "cat > gen.sh <<'EOF'\n#!/usr/bin/env bash\nherd7 x.litmus\nEOF\nbash gen.sh", 2, 0, 1),
            ("实现员 cat <<EOF > 文件 写出含 --all 的脚本、chmod 之后 ./ 执行", writer,
             "cat <<'EOF' > gen2.sh\nherd7 x.litmus\nEOF\nchmod +x gen2.sh && ./gen2.sh", 2, 0, 1),
            ("实现员 tee 文件 <<EOF 写出含 --all 的脚本再 source", writer,
             "tee gen3.sh <<'EOF' >/dev/null\nherd7 x.litmus\nEOF\nsource gen3.sh", 2, 0, 1),
            ("实现员 <<-EOF（去制表符）写出含 --all 的脚本再 bash 起它", writer,
             "cat > gen4.sh <<-EOF\n\therd7 x.litmus\n\tEOF\nbash gen4.sh", 2, 0, 1),
            ("实现员 cd 进子目录写出、回来按相对路径起", writer,
             "mkdir -p gen-dir && cd gen-dir && cat > here.sh <<'EOF'\nherd7 x.litmus\nEOF\ncd .. && bash gen-dir/here.sh", 2, 0, 1),
            ("实现员 heredoc 写出的脚本里再写出、再起含 --all 的脚本", writer,
             "cat > gen-outer.sh <<'EOF'\ncat > gen-inner.sh <<'IN'\nherd7 x.litmus\nIN\nbash gen-inner.sh\nEOF\nbash gen-outer.sh", 2, 0, 2),
            ("实现员 bash -c 里写出含 --all 的脚本再起它", writer,
             "bash -c 'cat > gen-c.sh <<EOF\nherd7 x.litmus\nEOF\nbash gen-c.sh'", 2, 0, 1),
            ("实现员命令替换里写出含 --all 的脚本再起它", writer,
             "out=\"$(cat > gen-sub.sh <<'EOF'\nherd7 x.litmus\nEOF\nbash gen-sub.sh)\"", 2, 0, 1),
            ("实现员 heredoc 写出的脚本里写出另一份，外层后面再起那一份", writer,
             "cat > gen-maker.sh <<'EOF'\ncat > gen-made.sh <<'IN'\nherd7 x.litmus\nIN\nEOF\nbash gen-maker.sh && bash gen-made.sh", 2, 0, 2),
            ("实现员先写出含 --all 的一行、再 >> 追加一行轻的：前面那行还在", writer,
             "cat > gen8.sh <<'EOF'\nherd7 x.litmus\nEOF\ncat >> gen8.sh <<'EOF'\necho 尾\nEOF\nbash gen8.sh", 2, 0, 1),
            ("实现员 tee -a 往盘上含 --all 的 s.sh 追加一行轻的：盘上那几行还在", writer,
             "tee -a s.sh <<'EOF'\necho 尾\nEOF\nbash s.sh", 2, 0, 1),
            ("实现员 cat -n 带选项写出：认不出，照拒", writer,
             "cat -n > gen-numbered.sh <<'EOF'\nherd7 x.litmus\nEOF\nbash gen-numbered.sh", 2, 0, 0),
            ("实现员 heredoc 写出 python 程序再 ./ 执行：不读也不记", writer,
             "cat > gen.py <<'EOF'\n#!/usr/bin/env python3\nprint('herd7 x.litmus')\nEOF\n./gen.py", 0, 0, 0),
            ("实现员 cp 盘上含 --all 的 s.sh 出来再起：读源文件", writer, "cp s.sh copied.sh && bash copied.sh", 2, 0, 1),
            ("实现员 cp -f 到目录里再起：读源文件", writer, "cp -f s.sh elsewhere/ && bash elsewhere/s.sh", 2, 0, 1),
            ("实现员 cp 同一条命令里 heredoc 写出的脚本再起", writer,
             "cat > gen6.sh <<'EOF'\nherd7 x.litmus\nEOF\ncp gen6.sh gen7.sh && bash gen7.sh", 2, 0, 1),
            ("实现员 cp 的源不在：认不出，照拒", writer, "cp nowhere.sh copied-missing.sh && bash copied-missing.sh", 2, 0, 0),
            ("实现员 printf 写出再起：认不出，照拒", writer, "printf 'herd7 x.litmus\\n' > printed.sh && bash printed.sh", 2, 0, 0),
            ("实现员 ./ 直接执行 .md 文件：不当脚本读、不记", writer, "./notes.md", 0, 0, 0),
            ("实现员 ./ 直接执行没扩展名、没 #! 的文本：不当脚本读、不记", writer, "./plain-text", 0, 0, 0),
            ("实现员 ./ 直接执行还不在、没扩展名的：不记「不存在」", writer, "bash scripts/hosts/run-with-memory-cap.sh 4G ./target/release/not-built-yet", 0, 0, 0),
            ("实现员 ./ 直接执行没扩展名、#! 指到 bash 的脚本：读进去照拒", writer, "./runner", 2, 0, 1),
            ("实现员 ./ 直接执行 .py 名字、#! 指到 sh 的脚本：读进去照拒", writer, "./mislabeled.py", 2, 0, 1),
            ("实现员同一条命令里起含 --all 的 s.sh 两遍：只判一遍", writer, "bash s.sh; ./s.sh", 2, 0, 1),
            ("实现员起的脚本里三种起法各起一遍 s.sh：s.sh 只判一遍", writer, "bash twice.sh", 2, 0, 2),
            ("实现员先起盘上轻的 inner-rel.sh、再 heredoc 盖成含 --all 的再起：写出的变了重判", writer,
             "bash inner-rel.sh; cat > inner-rel.sh <<'EOF'\nherd7 x.litmus\nEOF\nbash inner-rel.sh", 2, 0, 2),
            ("实现员先起里面起了不存在脚本的 has-missing.sh、再经 outer-missing.sh 起它：检出照复用的那一遍记两条", writer,
             "bash has-missing.sh; bash outer-missing.sh", 0, 2, 2),
            ("实现员同一份 rel.sh 换个当前目录再起：按相对路径起的脚本变了，重判", writer, "bash rel.sh; cd sub && bash ../rel.sh", 2, 0, 4),
            ("实现员先经 7 层嵌套碰到 deep6.sh、再直接起它：那一遍撞上嵌套上限，重判", writer, "bash deep1.sh; bash deep6.sh", 2, 1, 8),
            ("实现员先起 cyc-a.sh（里面绕回自己）、再起 cyc-b.sh：cyc-b.sh 那一遍绕回外层，重判", writer, "bash cyc-a.sh; bash cyc-b.sh", 2, 0, 3),
        ]
        def judge_cases(case_list):
            for case in case_list:
                label, agent_type, command, want = case[:4]
                want_notices, want_read = case[4:6] if len(case) > 4 else (0, 0)
                hook_input = {"tool_name": "Bash", "cwd": work, "tool_input": {"command": command}}
                if agent_type:
                    hook_input["agent_type"] = agent_type
                decision = scan_decision(hook_input, work)
                results.append((label, want, decision.code))
                results.append((f"{label}：记几条检出", want_notices, len(decision.notices)))
                results.append((f"{label}：读进去几份脚本", want_read, len(decision.scripts_read)))
        judge_cases(cases)
        # 本仓的代码（门禁阶段与格、上游的 lint、门禁结构检查）、会编的 cargo、编出来的二进制直接起，由 task_server_decision 拒（scan_decision 只判外部程序、认不出的写法与编译目录）：
        # 这几格走完整的判法，只看退出码（读进去几份脚本、记几条检出由上面 scan_decision 的格钉住）；本仓的脚本凭路径尾部在本仓里有这份文件认，样本工作区放了同名的文本（build_sample_scripts）。
        # PROJECT_CODE_GUARD_BREAK=task-server-direct-allowed 下这几格全红（应当是 2，实际 0）
        def judge_task_cases(case_list):
            for label, agent_type, command in case_list:
                hook_input = {"tool_name": "Bash", "cwd": work, "tool_input": {"command": command}}
                if agent_type:
                    hook_input["agent_type"] = agent_type
                results.append((label, 2, task_server_decision(hook_input, work).code))
        judge_task_cases([
            ("实现员经内存包装起 cargo test -p 当前目录探针自己的包名：照拒", writer,
             "bash scripts/hosts/run-with-memory-cap.sh 8G bash -c 'cd probes/linked && cargo test -p attack-probe'"),
            ("实现员经内存包装与 bash -c 起 cargo run --package= 点名探针：照拒", writer,
             "bash scripts/hosts/run-with-memory-cap.sh 8G bash -c 'cd probes/linked && cargo run --release --package=attack-probe'"),
            ("实现员经内存包装 env -C 换进探针再 cargo test -p 探针自己的包名：照拒", writer,
             "bash scripts/hosts/run-with-memory-cap.sh 8G env -C probes/linked cargo test -p attack-probe"),
            ("实现员 cd ${D} 工作区根、经 timeout、nice、capped.sh、内存包装四层全量：照拒", writer,
             f"D={work}; cd ${{D}} && timeout 235 nice -n 19 bash scripts/hosts/capped.sh 31 bash scripts/hosts/run-with-memory-cap.sh 12G cargo test"),
            ("主 agent D=… 之后 cd $D 全量：照拒", None, f"D={work}; cd /tmp && cd $D && bash {repo}/scripts/hosts/run-with-memory-cap.sh 16G cargo test"),
            ("实现员交回前自查命名：code-source-discipline.sh 点名那四格也是直接起本仓的门禁阶段，照拒（写进报告交主 agent 跑）", writer,
             "bash .claude/gate.d/code-source-discipline.sh --check vague-names,test-file-names,ordinal-names,data-names ."),
            ("实现员交回前自查命名：经范围声明与 nice 起、等号写法点名一格，照拒", writer,
             "SINGLEFS_RUN_SCOPE='种类=items; 跑什么=命名四格; 范围=1 格; 为什么=交回前自查命名' nice -n 19 bash .claude/gate.d/code-source-discipline.sh --check=data-names ."),
            ("实现员交回前自查命名：点名里夹着别的格：照拒", writer, "bash .claude/gate.d/code-source-discipline.sh --check vague-names,format-constants ."),
            ("实现员交回前自查命名：不带 --check 整道：照拒", writer, "bash .claude/gate.d/code-source-discipline.sh ."),
            ("实现员在仓根裸跑 cargo test", writer, "cargo test"),
            ("主 agent 裸 cargo run E152", None, "cd scripts/experiment-apparatus && cargo run --release --bin e152-file-system-benchmark"),
            ("主 agent {cargo,} 空的一份起全量照拒", None, "bash scripts/hosts/run-with-memory-cap.sh 16G {cargo,} test --all"),
            ("主 agent cargo {,test} 空的一份起全量照拒", None, "bash scripts/hosts/run-with-memory-cap.sh 16G cargo {,test} --all"),
            ("书记员跑 doc-decisions（直接起本仓的门禁阶段）", scribe, "nice -n 19 bash .claude/gate.d/doc-decisions.sh"),
            ("通用 agent 跑 code-tooling.sh 的 agent-write-scope 格（直接起本仓的门禁阶段）", "general-purpose", "bash .claude/gate.d/code-tooling.sh --check agent-write-scope"),
            ("主 agent 跑 code-tooling.sh（直接起本仓的门禁阶段）", None, "bash .claude/gate.d/code-tooling.sh"),
            ("主 agent 跑 gate-lint.sh", None, "GATE_LINT_DIR=scripts bash .claude/singlefs-ai-sop/scripts/gate-lint.sh"),
            ("实现员跑 gate-structure-check.py", writer, "python3 scripts/gate/gate-structure-check.py"),
            ("主 agent 跑 hooks-registered.sh", None, "bash .claude/singlefs-ai-sop/scripts/hooks-registered.sh ."),
            ("实现员复现输入那一条（反引号里的 walk.rs）：末尾的 doc-lint.sh 是直接起的本仓脚本，照拒", writer, SLOW_REPRODUCTION_COMMAND),
            ("一百三十七#8：工具实现员经 multi-host-run.sh --peer 起门禁格，照拒", "tooling-writer",
             f"bash {repo}/scripts/hosts/multi-host-run.sh --peer --memory-cap 8G -- bash .claude/gate.d/code-tooling.sh --check agent-write-scope ."),
        ])
        # 认不出的一律拒（偏严）的各格。每格后面的注释写弄坏它的开关
        for relative, text in (("srclib/entry.sh", '#!/usr/bin/env bash\nsource "$(dirname "${BASH_SOURCE[0]}")/lib.sh"\n'),
                               ("srclib/lib.sh", "herd7 x.litmus\n"),
                               ("launcher-function.sh", '#!/usr/bin/env bash\nrun_logged() { local log=$1; shift; "$@" > "$log" 2>&1; }\nrun_logged out.log herd7 x.litmus\n'),
                               ("family-copy/scripts/hosts/run-with-memory-cap.sh", '#!/usr/bin/env bash\nshift; exec "$@"\n'),
                               ("path-dir/runs-all.sh", "#!/usr/bin/env bash\nherd7 x.litmus\n"),
                               ("foreign-dispatch/scripts/hosts/multi-host-run.sh", '#!/usr/bin/env bash\nwhile [[ $# -gt 0 && $1 != -- ]]; do shift; done; shift; exec "$@"\n')):
            os.makedirs(os.path.dirname(os.path.join(work, relative)), exist_ok=True)
            with open(os.path.join(work, relative), "w", encoding="utf-8") as handle:
                handle.write(text)
            os.chmod(os.path.join(work, relative), 0o755)
        batch137_cases = [
            # 三种静默漏拒
            ("一百三十七：命令替换里不加括号的 case 模式不提前收尾，里面的 --all 照拒", None, 'echo "$(case a in a) herd7 x.litmus;; esac)"', 2),   # LIB_SHELL_WORDS_BREAK=substitution-case-and-comment-unseen
            ("一百三十七：命令替换里注释的 ) 不收尾，后面的 --all 照拒", None, 'echo "$(echo hi # )\nherd7 x.litmus)"', 2),                        # 同上
            ("一百三十七：coproc 后面那条照判", None, "coproc herd7 x.litmus", 2),                                                            # LIB_SHELL_WORDS_BREAK=coproc-as-command
            ("一百三十七：cargo 参数写成前面赋过值的变量，展开之后照判（--lib 放行）", None, "F=--lib; bash scripts/hosts/run-with-memory-cap.sh 8G cargo test -p singlefs-core $F", 0),          # LIB_PROJECT_CODE_RUNS_BREAK=cargo-arguments-unexpanded
            ("一百三十七：bash -c \"$(cat 文件)\" 照拒", writer, 'bash -c "$(cat light.sh)"', 2),                                               # 同上
            ("一百三十七：bash <(…) 照拒", writer, "bash <(echo 'cargo test -p singlefs-core --lib')", 2),                                      # 同上
            ("一百三十七：eval 变量里拼的命令照拒", writer, "c='cargo build'; eval \"$c\"", 2),                                                  # 同上
            ("一百三十七：source <(…) 照拒", writer, "source <(echo true)", 2),                                                                 # 同上
            ("一百三十七：命令词是 $(which cargo) 照拒", writer, "$(which cargo) build", 2),                                                    # 同上
            ("一百三十七：set -- 之后 \"$@\" 照拒", writer, 'set -- cargo build; "$@"', 2),                                                      # 同上
            ("一百三十七：for 循环变量当脚本路径照拒", writer, "for f in light.sh; do bash $f; done", 2),                                         # 同上
            ("一百三十七：PATH 指到放脚本的目录、命令词不带斜杠的 .sh 照读正文", writer, f"PATH={work}/path-dir:$PATH runs-all.sh", 2, 0, 1),                  # PROJECT_CODE_GUARD_BREAK=path-scripts-unread
            ("一百三十七：bash 起通配写出的脚本路径照 bash 展开读正文", writer, "bash ./s.s?", 2, 0, 1),                                       # PROJECT_CODE_GUARD_BREAK=glob-scripts-unread
            ("一百三十七：echo 写出再起照拒", writer, "echo 'cargo build' > gen-echo.sh; bash gen-echo.sh", 2),                               # PROJECT_CODE_GUARD_BREAK=unrecognized-writes-allowed
            ("一百三十七：python 写出再起照拒", writer, "python3 -c \"open('gen-py.sh','w').write('true')\"; bash gen-py.sh", 2),                # 同上
            # 拷出写法 >>、&>、tee：认成拷贝，照拷来的内容判
            ("一百三十七：cat 源 >> 拷出再起，照写出的内容判", writer, "cat s.sh >> gen-append.sh; bash gen-append.sh", 2, 0, 1),               # PROJECT_CODE_GUARD_BREAK=copy-redirections-unseen
            ("一百三十七：cat 轻脚本 &> 拷出再起，照内容判、放行", writer, "cat light.sh &> gen-both.sh; bash gen-both.sh", 0, 0, 1),           # 同上
            ("一百三十七：cat 源 | tee 拷出再起，照写出的内容判", writer, "cat s.sh | tee gen-tee.sh >/dev/null; bash gen-tee.sh", 2, 0, 1),   # PROJECT_CODE_GUARD_BREAK=tee-copy-unseen
            ("一百三十七：tee 目的 < 轻脚本 拷出再起，照内容判、放行", writer, "tee gen-tee2.sh < light.sh >/dev/null; bash gen-tee2.sh", 0, 0, 1),  # 同上
            # 标准输入从文件来的 shell：读那份文件判
            ("一百三十七：bash -s < 文件 读那份文件判", writer, "bash -s < s.sh", 2, 0, 1),                                                      # LIB_SHELL_WORDS_BREAK=shell-standard-input-file-unread
            ("一百三十七：cat 文件 | bash 读那份文件判", writer, "cat s.sh | bash", 2, 0, 1),                                                    # 同上
            ("一百三十七：sh -s < 轻脚本放行", writer, "sh -s < light.sh", 0, 0, 1),                                                             # 同上
            # 名单外启动器
            ("一百三十七：xargs bash 起读进来的脚本照拒", writer, "echo s.sh | xargs bash", 2),                                                  # 同上
            # 派发起法剥开照判（LIB_PROJECT_CODE_RUNS_BREAK=dispatch-launchers-unseen 下红的格逐格写在文件头）
            # 函数当启动器（脚本里经自己的 run_logged 起）
            ("一百三十七：脚本里经自己的 run_logged 起 --all 照拒", writer, "bash launcher-function.sh", 2, 0, 1),                               # PROJECT_CODE_GUARD_BREAK=launcher-functions-unjudged
            # 读正文时展开入口按自己所在目录 source 的库（第一百一十八节跟进）
            ("一百三十七：脚本 source 自己目录下的库，读正文时库也读", writer, "bash srclib/entry.sh", 2, 0, 2),                                # LIB_PROJECT_CODE_RUNS_BREAK=sourced-libraries-unread
            # 别处树里的内存包装拷贝：读正文、不算经内存包装（第一百一十八节「包装脚本同一口径」）
            # 喂给 shell 的是别的命令的输出（认不出喂进去的是什么）：照拒、照记检出（PROJECT_CODE_GUARD_BREAK=unjudged-shell-input-allowed 下红）
            ("一百三十七：grep 的输出经管道喂给 bash：认不出照拒", writer, "grep -h echo light.sh | bash", 2, 1, 0),
            ("一百三十七：别处树里的 run-with-memory-cap.sh 拷贝照读正文", writer,
             f"bash {work}/family-copy/scripts/hosts/run-with-memory-cap.sh 4G cargo test -p singlefs-core --test core_contract", 0, 0, 1),     # PROJECT_CODE_GUARD_BREAK=wrapper-copies-trusted
            ("一百三十七：函数体外引号里的花括号不当认不准的调用，只定义的函数体不判：放行", None,
             "function f() { herd7 x.litmus; }\nprintf '%s\\n' 'echo a{,}' > ./t3-137.sh", 0),                                            # LIB_SHELL_WORDS_BREAK=quoted-braces-as-unknown
            # BASH_ENV / ENV 指的启动文件当被起的脚本判（件 8f）；下面每一格都在 LIB_SHELL_WORDS_BREAK=startup-files-unread 下红，不红的几格另点名
            ("一百三十七：BASH_ENV 写在命令前、指到字面路径的脚本，正文起全量，照拒", None, f"BASH_ENV={work}/startup-heavy.sh bash -c true", 2, 0, 1),
            ("一百三十七：BASH_ENV 写相对路径，按这条命令的当前目录解，照拒", None, "BASH_ENV=startup-heavy.sh bash -c true", 2, 0, 1),
            ("一百三十七：env BASH_ENV=… 起 bash，照拒", None, f"env BASH_ENV={work}/startup-heavy.sh bash -c true", 2, 0, 1),
            ("一百三十七：前面 export BASH_ENV=… 再起 bash，照拒", None, f"export BASH_ENV={work}/startup-heavy.sh; bash -c true", 2, 0, 1),
            ("一百三十七：BASH_ENV 指到的脚本正文起全量，bash 起的又是一份轻脚本，两份都读，照拒", None, f"BASH_ENV={work}/startup-heavy.sh bash startup-light.sh", 2, 0, 2),
            ("一百三十七：BASH_ENV 指到的脚本 source 自己目录下的库，库也读，照拒", None, f"BASH_ENV={work}/srclib/entry.sh bash -c true", 2, 0, 2),
            ("一百三十七：BASH_ENV 指到的脚本正文起全量，实现员照拒", writer, f"BASH_ENV={work}/startup-heavy.sh bash -c true", 2, 0, 1),
            ("一百三十七：BASH_ENV 指到的脚本正文只有轻活，放行", None, f"BASH_ENV={work}/startup-light.sh bash -c true", 0, 0, 1),
            ("一百三十七：BASH_ENV 写成前面赋过字面值的变量，展开之后读正文，照拒", None, "X=startup-heavy.sh; BASH_ENV=$X bash -c true", 2, 0, 1),
            ("一百三十七：ENV 指到的脚本正文起全量，sh 起的，照拒", None, f"ENV={work}/startup-heavy.sh sh -c true", 2, 0, 1),
            ("一百三十七：ENV 指到的脚本正文起全量，bash --posix 起的，照拒", None, f"ENV={work}/startup-heavy.sh bash --posix -c true", 2, 0, 1),
            ("一百三十七：ENV 指到的脚本正文起全量，bash -o posix 起的，照拒", None, f"ENV={work}/startup-heavy.sh bash -o posix -c true", 2, 0, 1),
            ("一百三十七：ENV 指到的脚本，bash 不带 --posix 不读：放行", None, f"ENV={work}/startup-heavy.sh bash -c true", 0),                  # 不读它的格，开关下也不红
            ("一百三十七：BASH_ENV 指到的脚本，sh 不读：放行", None, f"BASH_ENV={work}/startup-heavy.sh sh -c true", 0),                       # 同上
            ("一百三十七：bash -n 只查语法，不读 BASH_ENV：放行", None, f"BASH_ENV={work}/startup-heavy.sh bash -n -c true", 0),               # 同上
            # 下一格另在 LIB_SHELL_WORDS_BREAK=env-ignore-environment-kept 下红（推的：命令前的 BASH_ENV 留到 env -i 后面）
            ("一百三十七：env -i 清掉 BASH_ENV 再起 bash：放行", None, f"BASH_ENV={work}/startup-heavy.sh env -i bash -c true", 0),             # 同上
            ("一百三十七：unset BASH_ENV 之后起 bash：放行", None, f"export BASH_ENV={work}/startup-heavy.sh; unset BASH_ENV; bash -c true", 0), # 同上
            ("一百三十七：BASH_ENV 指到的文件不在，bash 什么都不读：放行、不记检出", None, "BASH_ENV=no-such-startup.sh bash -c true", 0),            # 同上
            ("一百三十七：BASH_ENV 写成命令替换拼出的路径，认不出，实现员照拒", writer, "BASH_ENV=$(echo startup-heavy.sh) bash -c true", 2),     # 另在 LIB_PROJECT_CODE_RUNS_BREAK=unrecognized-launches-allowed 下红
            ("一百三十七：BASH_ENV 写成命令替换拼出的路径，认不出，主 agent 照拒", None, "BASH_ENV=$(echo startup-heavy.sh) bash -c true", 2),
            ("一百三十七：BASH_ENV 写成没赋过值的变量，认不出，实现员照拒", writer, "BASH_ENV=$UNASSIGNED bash -c true", 2),
            ("一百三十七：env 的 BASH_ENV 写成命令替换，认不出，实现员照拒", writer, "env BASH_ENV=$(echo startup-heavy.sh) bash -c true", 2),
            ("一百三十七：ENV 写成命令替换，sh 起的，认不出，实现员照拒", writer, "ENV=$(echo startup-heavy.sh) sh -c true", 2),
            # shell 启动时先读的其余几种（件 8g）；括号里是弄坏它的开关（都是 LIB_SHELL_WORDS_BREAK=<项>），startup-files-unread 下读文件的格也红
            # 直接执行的脚本（executed-script-startup-unread）
            ("一百三十七：直接执行 #!/bin/bash 的脚本，BASH_ENV 指的文件照读，照拒", None, f"BASH_ENV={work}/startup-heavy.sh ./exec-bash.sh", 2, 0, 2),
            ("一百三十七：直接执行 #!/usr/bin/env bash 的脚本，BASH_ENV 指的文件照读，照拒", None, f"BASH_ENV={work}/startup-heavy.sh ./exec-env-bash.sh", 2, 0, 2),
            ("一百三十七：直接执行没有 #! 的 .sh，从 bash 里起是 bash 另起自己，BASH_ENV 指的文件照读，照拒", None, f"BASH_ENV={work}/startup-heavy.sh ./exec-no-shebang.sh", 2, 0, 2),
            ("一百三十七：env 写的 BASH_ENV，直接执行 #!/bin/bash 的脚本，照拒", None, f"env BASH_ENV={work}/startup-heavy.sh ./exec-bash.sh", 2, 0, 2),
            ("一百三十七：直接执行 #!/bin/sh 的脚本，ENV 指的文件照读，照拒", None, f"ENV={work}/startup-heavy.sh ./exec-sh.sh", 2, 0, 2),
            ("一百三十七：直接执行 #!/bin/zsh 的脚本，ZDOTDIR 下的 .zshenv 照读，照拒", None, f"ZDOTDIR={work}/zdot-env ./exec-zsh.sh", 2, 0, 3),
            ("一百三十七：直接执行 #!/bin/bash 的脚本，BASH_ENV 指的文件只有轻活，放行", None, f"BASH_ENV={work}/startup-light.sh ./exec-bash.sh", 0, 0, 2),
            ("一百三十七：直接执行 #!/bin/sh 的脚本不读 BASH_ENV：放行", None, f"BASH_ENV={work}/startup-heavy.sh ./exec-sh.sh", 0, 0, 1),               # 不读的格，开关下也不红
            ("一百三十七：直接执行 #!/bin/bash 的脚本不读 ENV：放行", None, f"ENV={work}/startup-heavy.sh ./exec-bash.sh", 0, 0, 1),                     # 同上
            ("一百三十七：直接执行 python 脚本不读 BASH_ENV：放行", None, f"BASH_ENV={work}/startup-heavy.sh ./exec-python.py", 0, 0, 0),                 # 同上
            ("一百三十七：直接执行 #! 指到认不出的解释器、环境里设了 BASH_ENV，实现员照拒", writer, f"BASH_ENV={work}/startup-light.sh ./exec-unknown.sh", 2),   # 另在 LIB_PROJECT_CODE_RUNS_BREAK=unrecognized-launches-allowed 下不红（这一句不经 unrecognized_launch）
            ("一百三十七：直接执行 #! 指到认不出的解释器、环境里没设 BASH_ENV / ENV：放行", writer, "./exec-unknown.sh", 0),                                 # 同上
            ("一百三十七：直接执行 #!/bin/bash 的脚本，BASH_ENV 写成命令替换拼出的路径，认不出，实现员照拒", writer, "BASH_ENV=$(echo startup-heavy.sh) ./exec-bash.sh", 2, 0, 1),
            # 先赋值后 export、declare -x、typeset -x、export -n、set -a（declared-exports-unseen）
            ("一百三十七：declare -x BASH_ENV=…，再起 bash，照拒", None, f"declare -x BASH_ENV={work}/startup-heavy.sh; bash -c true", 2, 0, 1),
            ("一百三十七：typeset -x BASH_ENV=…，再起 bash，照拒", None, f"typeset -x BASH_ENV={work}/startup-heavy.sh; bash -c true", 2, 0, 1),
            ("一百三十七：先 BASH_ENV=… 赋值、后 export BASH_ENV，再起 bash，照拒", None, f"BASH_ENV={work}/startup-heavy.sh; export BASH_ENV; bash -c true", 2, 0, 1),
            ("一百三十七：先 export BASH_ENV、后赋值，再起 bash，照拒", None, f"export BASH_ENV; BASH_ENV={work}/startup-heavy.sh; bash -c true", 2, 0, 1),
            ("一百三十七：export 过之后换成另一份脚本，起 bash 读的是后一份，照拒", None, f"export BASH_ENV={work}/startup-light.sh; BASH_ENV={work}/startup-heavy.sh; bash -c true", 2, 0, 1),
            ("一百三十七：set -a 之后赋值的 BASH_ENV 进环境，照拒", None, f"set -a; BASH_ENV={work}/startup-heavy.sh; bash -c true", 2, 0, 1),
            ("一百三十七：local -x BASH_ENV=… 的函数里起 bash，照拒", None, f"f() {{ local -x BASH_ENV={work}/startup-heavy.sh; bash -c true; }}; f", 2, 0, 1),
            ("一百三十七：export -n 撤掉 export 之后起 bash：放行", None, f"export BASH_ENV={work}/startup-heavy.sh; export -n BASH_ENV; bash -c true", 0, 0, 0),
            ("一百三十七：export -n 带赋值，不进环境：放行", None, f"export -n BASH_ENV={work}/startup-heavy.sh; bash -c true", 0, 0, 0),
            ("一百三十七：declare +x 撤掉 export 之后起 bash：放行", None, f"export BASH_ENV={work}/startup-heavy.sh; declare +x BASH_ENV; bash -c true", 0, 0, 0),
            ("一百三十七：只赋值不 export，bash 看不到：放行", None, f"BASH_ENV={work}/startup-heavy.sh; bash -c true", 0, 0, 0),                    # 不红的格（开关下也一样）
            ("一百三十七：set -a 又 set +a 之后赋值：放行", None, f"set -a; set +a; BASH_ENV={work}/startup-heavy.sh; bash -c true", 0, 0, 0),            # 同上
            ("一百三十七：declare -x BASH_ENV 写成命令替换拼出的路径，认不出，实现员照拒", writer, "declare -x BASH_ENV=$(echo startup-heavy.sh); bash -c true", 2),
            ("一百三十七：先赋值后 export 的 BASH_ENV 是命令替换拼出的路径，认不出，实现员照拒", writer, "BASH_ENV=$(echo startup-heavy.sh); export BASH_ENV; bash -c true", 2),
            # 交互的 bash：不读 BASH_ENV，读 --rcfile / --init-file 指的文件，没给读 HOME 下的 .bashrc；sh -i 与 posix 模式读 ENV（interactive-startup-files-unread）
            ("一百三十七：bash -i 读 HOME 下的 .bashrc，里面起全量，照拒", None, f"HOME={work}/home-heavy bash -i -c true", 2, 0, 1),
            ("一百三十七：bash -ic 合写的 -i 同样读 HOME 下的 .bashrc，照拒", None, f"HOME={work}/home-heavy bash -ic true", 2, 0, 1),
            ("一百三十七：bash --rcfile 文件 -i，文件里起全量，照拒", None, f"bash --rcfile {work}/startup-heavy.sh -i -c true", 2, 0, 1),
            ("一百三十七：bash --init-file 文件 -i，文件里起全量，照拒", None, f"bash --init-file {work}/startup-heavy.sh -i -c true", 2, 0, 1),
            ("一百三十七：bash -i 读 HOME 下只有轻活的 .bashrc：放行", None, f"HOME={work}/home-light bash -i -c true", 0, 0, 1),
            ("一百三十七：bash -i 不读 BASH_ENV，.bashrc 只有轻活：放行", None, f"BASH_ENV={work}/startup-heavy.sh HOME={work}/home-light bash -i -c true", 0, 0, 1),
            ("一百三十七：bash --norc -i 什么都不读：放行", None, f"HOME={work}/home-heavy bash --norc -i -c true", 0, 0, 0),                          # 不红的格（开关下也一样）
            ("一百三十七：bash --rcfile 不带 -i 是非交互，不读：放行", None, f"bash --rcfile {work}/startup-heavy.sh -c true", 0, 0, 0),                    # 同上
            ("一百三十七：bash -i 不读 ENV：放行", None, f"ENV={work}/startup-heavy.sh HOME={work}/home-light bash -i -c true", 0, 0, 1),                    # 同上
            ("一百三十七：sh -i 读 ENV 指的文件，照拒", None, f"ENV={work}/startup-heavy.sh sh -i -c true", 2, 0, 1),
            ("一百三十七：sh -i 读 ENV 指的文件，只有轻活：放行", None, f"ENV={work}/startup-light.sh sh -i -c true", 0, 0, 1),
            ("一百三十七：bash --posix -i 读 ENV 指的文件，照拒", None, f"ENV={work}/startup-heavy.sh bash --posix -i -c true", 2, 0, 1),
            ("一百三十七：bash --rcfile 写成命令替换拼出的路径，认不出，实现员照拒", writer, "bash --rcfile $(echo startup-heavy.sh) -i -c true", 2),
            ("一百三十七：bash -i 的 HOME 写成命令替换，.bashrc 路径认不出，实现员照拒", writer, "HOME=$(echo home-light) bash -i -c true", 2),
            ("一百三十七：bash --rcfile 指到的文件不在，什么都不读：放行", writer, f"bash --rcfile {work}/no-such-rc.sh -i -c true", 0),                              # 同上
            # zsh：读 ZDOTDIR（没设取 HOME）下的 .zshenv、.zshrc，-f、--no-rcs 不读（zsh-startup-files-unread）
            ("一百三十七：zsh 读 ZDOTDIR 下的 .zshenv，里面起全量，照拒", None, f"ZDOTDIR={work}/zdot-env zsh -c true", 2, 0, 2),
            ("一百三十七：zsh 读 ZDOTDIR 下的 .zshrc，里面起全量，照拒", None, f"ZDOTDIR={work}/zdot-rc zsh -c true", 2, 0, 2),
            ("一百三十七：zsh 没设 ZDOTDIR 取 HOME 下的 .zshenv，照拒", None, f"HOME={work}/zdot-env zsh -c true", 2, 0, 2),
            ("一百三十七：前面 export 的 ZDOTDIR，zsh 照读，照拒", None, f"export ZDOTDIR={work}/zdot-env; zsh -c true", 2, 0, 2),
            ("一百三十七：zsh 的启动文件只有轻活：放行", None, f"ZDOTDIR={work}/zdot-light zsh -c true", 0, 0, 2),
            ("一百三十七：zsh -f 不读启动文件：放行", None, f"ZDOTDIR={work}/zdot-env zsh -f -c true", 0, 0, 0),                                              # 不红的格（开关下也一样）
            ("一百三十七：zsh --no-rcs 不读启动文件：放行", None, f"ZDOTDIR={work}/zdot-env zsh --no-rcs -c true", 0, 0, 0),                                 # 同上
            ("一百三十七：bash 不读 ZDOTDIR：放行", None, f"ZDOTDIR={work}/zdot-env bash -c true", 0, 0, 0),                                                 # 同上
            ("一百三十七：zsh 的 ZDOTDIR 写成命令替换，认不出，实现员照拒", writer, "ZDOTDIR=$(echo zdot-env) zsh -c true", 2),
        ]
        judge_cases(batch137_cases)
        # trap、标准输入设备、cd 失败 / 算不出、~+ / ~-、rbash / busybox / dash / ksh。
        # 每格后面注释写弄坏它的开关；startup-heavy.sh（herd7 x.litmus）、startup-light.sh（echo hello）在 work 根
        d137_cases = [
            # H2：trap 'action' 的字符串当 shell 代码判（LIB_SHELL_WORDS_BREAK=trap-action-unread）
            ("d137 H2：trap 'herd7 x.litmus' EXIT 照拒", None, "trap 'herd7 x.litmus' EXIT; true", 2),
            ("d137 H2：trap 'herd7 x.litmus' ERR 照拒", writer, "trap 'herd7 x.litmus' ERR; true", 2),
            ("d137 H2：trap 'herd7 x.litmus' 0 照拒", writer, "trap 'herd7 x.litmus' 0; true", 2),
            ("d137 H2：trap 'bash startup-heavy.sh' DEBUG 读正文照拒", writer, "trap 'bash startup-heavy.sh' DEBUG; true", 2, 0, 1),
            ("d137 H2：trap -- 'herd7 x.litmus' EXIT（-- 之后是 action）照拒", writer, "trap -- 'herd7 x.litmus' EXIT; true", 2),
            ("d137 H2：trap 'rm -f x' EXIT 无害放行", writer, "trap 'rm -f x' EXIT; true", 0),
            ("d137 H2：trap - EXIT 重置、无 action 放行", writer, "trap - EXIT; echo hi", 0),
            # H4：标准输入设备 /dev/stdin、/dev/fd/0（LIB_SHELL_WORDS_BREAK=standard-input-device-script-unread）
            ("d137 H4：bash /dev/stdin < 文件 读那份文件判，照拒", writer, "bash /dev/stdin < startup-heavy.sh", 2, 0, 1),
            ("d137 H4：echo 代码 | bash /dev/stdin 照拒", None, "echo 'herd7 x.litmus' | bash /dev/stdin", 2),
            ("d137 H4：bash /dev/fd/0 <<< 代码 照拒", writer, "bash /dev/fd/0 <<< 'herd7 x.litmus'", 2),
            ("d137 H4：source /dev/stdin <<< 代码 照拒", writer, "source /dev/stdin <<< 'herd7 x.litmus'", 2),
            ("d137 H4：bash /dev/stdin < 轻脚本 放行", writer, "bash /dev/stdin < startup-light.sh", 0, 0, 1),
            # H5：cd 失败 / 目标算不出之后的相对路径脚本（PROJECT_CODE_GUARD_BREAK=relative-script-undeterminable-directory-allowed）
            ("d137 H5：cd 到不在的目录、cd 失败，相对脚本认不出在哪，照拒", None, "cd no-such-d137 2>/dev/null; bash startup-heavy.sh", 2),
            ("d137 H5：cd 目标写成命令替换、算不出，相对脚本认不出，照拒", writer, 'cd "$(echo no-such-d137)" 2>/dev/null; bash startup-heavy.sh', 2),
            ("d137 H5：cd 到在的目录（.）起轻脚本，放行", writer, "cd . ; bash startup-light.sh", 0, 0, 1),
            # H6：~+（$PWD）、~-（$OLDPWD 算不出）（LIB_SHELL_WORDS_BREAK=tilde-plus-minus-unexpanded）
            ("d137 H6：bash ~+/脚本 照 $PWD 解、读正文照拒", writer, "bash ~+/startup-heavy.sh", 2, 0, 1),
            ("d137 H6：BASH_ENV=~+/脚本 起 bash，照 $PWD 解、读正文照拒", None, "BASH_ENV=~+/startup-heavy.sh bash -c true", 2, 0, 1),
            ("d137 H6：bash ~-/脚本 的 OLDPWD 算不出，认不出起哪一份，照拒", writer, "bash ~-/startup-heavy.sh", 2),
            # H8：rbash / busybox applet / dash / ksh（LIB_SHELL_WORDS_BREAK=renamed-shells-unseen；dash、ksh 本来就在 SHELL_NAMES）
            ("d137 H8：rbash -c 代码 照 shell 判、照拒", None, "rbash -c 'herd7 x.litmus'", 2),
            ("d137 H8：rbash 起脚本 读正文照拒", writer, "rbash startup-heavy.sh", 2, 0, 1),
            ("d137 H8：busybox sh -c 代码 照拒", writer, "busybox sh -c 'herd7 x.litmus'", 2),
            ("d137 H8：busybox ash -c 代码 照拒", writer, "busybox ash -c 'herd7 x.litmus'", 2),
            ("d137 H8：dash -c 代码 照拒", writer, "dash -c 'herd7 x.litmus'", 2),
            ("d137 H8：ksh -c 代码 照拒", writer, "ksh -c 'herd7 x.litmus'", 2),
            ("d137 H8：busybox ls 不是 shell applet，放行", writer, "busybox ls", 0),
            ("d137 H8：rbash -c 'echo hi' 无害放行", writer, "rbash -c 'echo hi'", 0),
            # H3：shopt -s expand_aliases 之后别名展开成要拒的命令（LIB_SHELL_WORDS_BREAK=aliases-unexpanded）
            ("d137 H3：别名展开成 herd7 x.litmus 照拒", None, "shopt -s expand_aliases\nalias t='herd7 x.litmus'\nt", 2),
            ("d137 H3：别名 cargo 自展开（换行、先定义后 shopt）照拒", writer, "alias cargo='herd7 x.litmus'\nshopt -s expand_aliases\ncargo", 2),
            ("d137 H3：别名展开成起脚本（同一条里 alias 先提到脚本、认不出写法）照拒", writer, "shopt -s expand_aliases\nalias ll='bash startup-heavy.sh'\nll", 2),
            ("d137 H3：别名展开成 ls -l 无害放行", writer, "shopt -s expand_aliases\nalias ll='ls -l'\nll", 0),
            ("d137 H3：没 shopt -s expand_aliases（非交互 bash 不展开），别名不展开、放行", writer, "alias t='herd7 x.litmus'; t", 0),
        ]
        judge_cases(d137_cases)
        # BASH_ENV 包进起 shell 的转发函数、exec / 复合命令上的输入重定向、拷来 / 链来的 shell 二进制、多源 cat / >> / tee 的源盘上不在。
        # 每格后面注释写弄坏它的开关；startup-heavy.sh / s.sh（herd7 x.litmus）、startup-light.sh（echo hello）在 work 根，/bin/bash 是真的
        d137r3_cases = [
            # A2：BASH_ENV / ENV 包进起 shell 的转发函数（PROJECT_CODE_GUARD_BREAK=bash-env-forwarding-untracked）
            ("d137 A2：BASH_ENV=脚本 包进起 bash 的转发函数、跟进函数体读正文照拒", None, "f(){ bash -c true; }; BASH_ENV=startup-heavy.sh f", 2, 0, 1),
            ("d137 A2：env BASH_ENV=脚本 起转发函数照拒", writer, "g(){ bash -c true; }; env BASH_ENV=startup-heavy.sh g", 2, 0, 1),
            ("d137 A2：function 关键字定义的函数同样跟进、照拒", writer, "function f() { bash -c true; }; BASH_ENV=startup-heavy.sh f", 2, 0, 1),
            ("d137 A2：BASH_ENV 指到轻脚本，放行", writer, "f(){ bash -c true; }; BASH_ENV=startup-light.sh f", 0, 0, 1),
            ("d137 A2：函数体不起 shell（只有 echo）、BASH_ENV 不跟进，放行", writer, "f(){ echo hi; }; BASH_ENV=startup-heavy.sh f", 0, 0, 0),
            ("d137 A2：函数体起 shell 但这一条没设 BASH_ENV，放行", writer, "f(){ bash -c true; }; f", 0, 0, 0),
            # H4：exec / 复合命令上的输入重定向下发给里层读标准输入的 shell（PROJECT_CODE_GUARD_BREAK=compound-stdin-untracked）
            ("d137 H4：exec < 文件; bash 读那份文件判、照拒", writer, "exec < startup-heavy.sh; bash", 2, 0, 1),
            ("d137 H4：exec 0< 文件; bash -s 照拒", writer, "exec 0< startup-heavy.sh; bash -s", 2, 0, 1),
            ("d137 H4：{ bash; } < 文件 照拒", writer, "{ bash; } < startup-heavy.sh", 2, 0, 1),
            ("d137 H4：( bash ) < 文件 照拒", writer, "( bash ) < startup-heavy.sh", 2, 0, 1),
            ("d137 H4：while … done < 文件 照拒", writer, "while true; do bash; break; done < startup-heavy.sh", 2, 0, 1),
            ("d137 H4：exec < 轻文件; bash 放行", writer, "exec < startup-light.sh; bash", 0, 0, 1),
            ("d137 H4：{ bash; } < 轻脚本 放行", writer, "{ bash; } < startup-light.sh", 0, 0, 1),
            ("d137 H4：while read … done < 文件 里没有裸 shell，放行", writer, 'while read l; do echo "$l"; done < startup-heavy.sh', 0, 0, 0),
            ("d137 H4：exec < 变量; bash 来源算不出，照拒", writer, "exec < $SOMEVAR137; bash", 2),
            # H8：同一条命令里 cp / ln / install 拷来、链来的 shell 二进制（PROJECT_CODE_GUARD_BREAK=copied-shell-binary-untracked）
            ("d137 H8：cp /bin/bash ./x; ./x -c 代码 照 bash 判、照拒", None, "cp /bin/bash ./mysh137a && ./mysh137a -c 'herd7 x.litmus'", 2, 0, 0),
            ("d137 H8：ln -sf /bin/bash ./x; ./x -c 代码 照拒", writer, "ln -sf /bin/bash ./mysh137b; ./mysh137b -c 'herd7 x.litmus'", 2, 0, 0),
            ("d137 H8：install /bin/bash ./x; ./x 脚本 读正文照拒", writer, "install /bin/bash ./mysh137c; ./mysh137c startup-heavy.sh", 2, 0, 1),
            ("d137 H8：cp /bin/bash ./x; ./x 轻脚本 放行", writer, "cp /bin/bash ./mysh137d; ./mysh137d startup-light.sh", 0, 0, 1),
            ("d137 H8：cp /bin/ls ./x; ./x 不是 shell、放行", writer, "cp /bin/ls ./myls137; ./myls137", 0, 0, 0),
            ("d137 H8：cp /bin/bash ./x; ./x -c 'echo hi' 无害放行", writer, "cp /bin/bash ./mysh137e; ./mysh137e -c 'echo hi'", 0, 0, 0),
            # H7 连 S3：多源 cat / >> / tee 的源盘上不在，照认不出拒（PROJECT_CODE_GUARD_BREAK=cat-missing-source-allowed）
            ("d137 H7：多源 cat 有盘上不在的源（在的源起全量）、写出再起，照拒", writer, "cat s.sh nocat137a.sh > dd137a.sh && bash dd137a.sh", 2, 0, 0),
            ("d137 H7：多源 cat 有盘上不在的源（在的源无害），照拒", writer, "cat startup-light.sh nocat137b.sh > dd137b.sh && bash dd137b.sh", 2, 0, 0),
            ("d137 H7：源是 echo 写出的（认不出写法、盘上不在），照拒", writer, "echo x > m137a.txt; cat startup-light.sh m137a.txt > dd137c.sh; bash dd137c.sh", 2, 0, 0),
            ("d137 H7：cat 源 >> 追加、源盘上不在，照拒", writer, "echo x > m137b.txt; cat m137b.txt >> dd137d.sh; bash dd137d.sh", 2, 0, 0),
            ("d137 H7：cat 源 | tee 拷出、源盘上不在，照拒", writer, "echo x > m137c.txt; cat m137c.txt | tee dd137e.sh > /dev/null; bash dd137e.sh", 2, 0, 0),
            ("d137 H7：多源 cat 两个源都在、都无害，放行、读写出的内容", writer, "cat startup-light.sh startup-light.sh > dd137f.sh && bash dd137f.sh", 0, 0, 1),
            ("d137 H7：单源 cat 源在、起全量，照拒（控制组）", writer, "cat s.sh > dd137g.sh && bash dd137g.sh", 2, 0, 1),
        ]
        judge_cases(d137r3_cases)
        # 登录 shell 的 profile、系统级启动文件、别的程序里起的命令；脚本正文里起 claude 归 bash-command-detector.sh。
        # 系统级启动文件的目录（lib_shell_words.py 的 SYSTEM_STARTUP_DIRECTORY）平常换成空的 system-startup-empty（不读本机的 /etc），系统级那一组换成 system-startup-heavy
        for relative, text in (
                               ('home-login-heavy/.bash_profile', 'herd7 x.litmus\n'),
                               ('home-login-light/.bash_profile', 'echo hello\n'),
                               ('home-profile-heavy/.profile', 'herd7 x.litmus\n'),
                               ('system-startup-heavy/profile', 'herd7 x.litmus\n'),
                               ('system-startup-heavy/bash.bashrc', 'herd7 x.litmus\n'),
                               ('system-startup-empty/.keep', ''),
                               ('prog-heavy137.awk', 'BEGIN { system("herd7 x.litmus") }\n'),
                               ('prog-light137.awk', '{ print $1 }\n'),
                               ('mk-heavy137/Makefile', 'all:\n\therd7 x.litmus\n'),
                               ('mk-light137/Makefile', 'all:\n\techo hello\n'),
                               ('rcfile-light137.sh', 'echo hello\n'),
                               ('body-embedded137.sh', '#!/usr/bin/env bash\nperl -e \'system("herd7 x.litmus")\'\n'),
):
            os.makedirs(os.path.dirname(os.path.join(work, relative)), exist_ok=True)
            with open(os.path.join(work, relative), "w", encoding="utf-8") as handle:
                handle.write(text)
        # 登录 shell 读 /etc/profile 与 HOME 下的 profile（LIB_SHELL_WORDS_BREAK=login-profiles-unread 下照拒的格红，逐格见文件头开关清单）
        judge_cases([
            ('d137 已知没认：登录 bash -l 读 HOME 下的 .bash_profile，里面起全量，照拒', writer, f'HOME={work}/home-login-heavy bash -l -c true', 2, 0, 1),
            ('d137 已知没认：登录 bash --login 同样读 .bash_profile，照拒', writer, f'HOME={work}/home-login-heavy bash --login -c true', 2, 0, 1),
            ('d137 已知没认：登录 sh -l 读 HOME 下的 .profile，照拒', writer, f'HOME={work}/home-profile-heavy sh -l -c true', 2, 0, 1),
            ('d137 已知没认：登录 exec -l bash 照登录 shell 读 .bash_profile，照拒', writer, f'HOME={work}/home-login-heavy exec -l bash -c true', 2, 0, 1),
            ('d137 已知没认：登录 exec -a -bash bash 照登录 shell 读 .bash_profile，照拒', writer, f'HOME={work}/home-login-heavy exec -a -bash bash -c true', 2, 0, 1),
            ('d137 已知没认：登录 bash -l 读的 profile 只有轻活，放行', writer, f'HOME={work}/home-login-light bash -l -c true', 0, 0, 1),
            ('d137 已知没认：登录 bash --noprofile -l 不读 profile，放行', writer, f'HOME={work}/home-login-heavy bash --noprofile -l -c true', 0, 0, 0),
            ('d137 已知没认：登录 exec -a 名字不以 - 起头，不是登录 shell，放行', writer, f'HOME={work}/home-login-heavy exec -a mybash bash -c true', 0, 0, 0),
        ])
        # 系统级启动文件：/etc/profile、/etc/bash.bashrc（样本目录；LIB_SHELL_WORDS_BREAK=system-startup-files-unread 下照拒的格红）
        shell_words.SYSTEM_STARTUP_DIRECTORY = os.path.join(work, "system-startup-heavy")
        try:
            judge_cases([
                ('d137 已知没认：系统级 bash -l 读 /etc/profile（样本目录），里面起全量，照拒', writer, f'HOME={work}/home-login-light bash -l -c true', 2, 0, 2),
                ('d137 已知没认：系统级 bash -i 读 /etc/bash.bashrc（样本目录），照拒', writer, f'HOME={work}/home-light bash -i -c true', 2, 0, 2),
                ('d137 已知没认：系统级 bash --rcfile 轻文件 -i 照读 /etc/bash.bashrc，照拒', writer, f'bash --rcfile {work}/rcfile-light137.sh -i -c true', 2, 0, 2),
                ('d137 已知没认：系统级 bash --norc -i 不读 /etc/bash.bashrc，放行', writer, 'bash --norc -i -c true', 0, 0, 0),
                ('d137 已知没认：系统级 不登录不交互的 bash -c 不读 /etc 下的，放行', writer, 'bash -c true', 0, 0, 0),
            ])
        finally:
            shell_words.SYSTEM_STARTUP_DIRECTORY = os.path.join(work, "system-startup-empty")
        # 别的程序里起的命令（LIB_SHELL_WORDS_BREAK=embedded-launches-unseen 下照拒的格红；PROJECT_CODE_GUARD_BREAK=program-files-unread 下读程序文件的格红，
        # embedded-unjudged-allowed 下认不出的格红；逐格见文件头开关清单）
        judge_cases([
            ('d137 H9：python -c 里 os.system 起全量，照拒', writer, 'python3 -c \'import os; os.system("herd7 x.litmus")\'', 2, 0, 0),
            ('d137 H9：python 读 heredoc 里 subprocess.run 起全量，照拒', writer, "python3 - <<'PY'\nimport subprocess\nsubprocess.run(['herd7', 'x.litmus'])\nPY", 2, 0, 0),
            ('d137 H9：python -c 不起命令，放行', writer, "python3 -c 'print(1)'", 0, 0, 0),
            ('d137 H9：python -c 里 os.system 的命令是变量，认不出照拒', writer, "python3 -c 'import os, sys; os.system(sys.argv[1])' x", 2, 0, 0),
            ('d137 H9：perl -e 里 system 起全量，照拒', writer, 'perl -e \'system("herd7 x.litmus")\'', 2, 0, 0),
            ("d137 H9：perl -ne 'print' 不起命令，放行", writer, "perl -ne 'print' f", 0, 0, 0),
            ('d137 H9：ruby -e 里 system 起全量，照拒', writer, 'ruby -e \'system("herd7 x.litmus")\'', 2, 0, 0),
            ('d137 H9：node -e 里 child_process 起全量，照拒', writer, 'node -e \'require("child_process").execSync("herd7 x.litmus")\'', 2, 0, 0),
            ('d137 H9：tclsh 读 here-string 里 exec 起全量，照拒', writer, "tclsh <<< 'exec herd7 x.litmus'", 2, 0, 0),
            ('d137 H9：gawk 的 system() 起全量，照拒', writer, 'gawk \'BEGIN{system("herd7 x.litmus")}\'', 2, 0, 0),
            ('d137 H9：awk 的 print 管道到全量，照拒', writer, 'awk \'BEGIN { print "x" | "herd7 x.litmus" }\'', 2, 0, 0),
            ("d137 H9：awk '{print $1}' 不起命令，放行", writer, "awk '{print $1}' f", 0, 0, 0),
            ('d137 H9：awk -f 程序文件里 system() 起全量，读正文照拒', writer, f'awk -f {work}/prog-heavy137.awk f', 2, 0, 1),
            ('d137 H9：awk -f 程序文件不起命令，放行', writer, f'awk -f {work}/prog-light137.awk f', 0, 0, 1),
            ('d137 H9：sed 的 e 命令起全量，照拒', writer, "sed -n '1e herd7 x.litmus' f", 2, 0, 0),
            ('d137 H9：sed 普通替换不起命令，放行', writer, "sed -n 's/a/b/p' f", 0, 0, 0),
            ('d137 H9：sed 的 s///e 执行模式空间，认不出照拒', writer, "sed -e 's/x/y/e' f", 2, 0, 0),
            ('d137 H9：git -c 别名 ! 起全量，照拒', writer, "git -c alias.x='!herd7 x.litmus' x", 2, 0, 0),
            ('d137 H9：git -c core.pager 起全量，照拒', writer, "git -c core.pager='herd7 x.litmus' log", 2, 0, 0),
            ('d137 H9：GIT_EXTERNAL_DIFF 起全量，照拒', writer, "GIT_EXTERNAL_DIFF='herd7 x.litmus' git diff", 2, 0, 0),
            ('d137 H9：git -c 别名指到 git 子命令，放行', writer, 'git -c alias.st=status st', 0, 0, 0),
            ('d137 H9：make 读 Makefile 配方起全量，照拒', writer, f'make -C {work}/mk-heavy137', 2, 0, 1),
            ('d137 H9：make 读 Makefile 配方只有轻活，放行', writer, f'make -C {work}/mk-light137', 0, 0, 1),
            ('d137 H9：make -f /dev/stdin 读 here-string 的配方起全量，照拒', writer, "make -f /dev/stdin <<< $'all:\\n\\therd7 x.litmus'", 2, 0, 0),
            ('d137 H9：tar --to-command 起全量，照拒', writer, "tar --to-command='herd7 x.litmus' -xf a.tar", 2, 0, 0),
            ('d137 H9：ssh -o ProxyCommand 起全量，照拒', writer, "ssh -o ProxyCommand='herd7 x.litmus' host true", 2, 0, 0),
            ('d137 H9：脚本正文里 perl -e 起全量，读正文照拒', writer, f'bash {work}/body-embedded137.sh', 2, 0, 1),
        ])
        # 认得出起的是什么、不起重活与 claude 的照放，同形起重活的照拒。逐格见文件头开关清单：LIB_SHELL_WORDS_BREAK=sed-expansions-unparsed、
        # sed-text-first-line-only、python-argument-variables-unresolved、awk-command-variables-unresolved、cd-known-targets-unexpanded、
        # cd-failure-guards-unseen、non-interactive-return-unseen 下「放行」的格（cd-known-targets-unexpanded、cd-failure-guards-unseen 另有照拒格的「读进去几份脚本」）红。{work} 换成样本目录
        for relative, text in (
                               ('r2-light.sh', 'echo hello\n'),
                               ('r2-heavy.sh', 'herd7 x.litmus\n'),
                               ('gitrepo137/.git/HEAD', 'ref: refs/heads/master\n'),
                               ('gitrepo137/.git/objects/.keep', ''),
                               ('gitrepo137/.git/refs/heads/.keep', ''),
                               ('gitrepo137/inner/.keep', ''),
                               ('gitrepo137/r2-light.sh', 'echo hello\n'),
                               ('gitrepo137/r2-heavy.sh', 'herd7 x.litmus\n'),
                               ('home-guarded137/.profile', '. {work}/home-guarded137/.bashrc\n'),
                               ('home-guarded137/.bashrc', '# 非交互就 return\ncase $- in\n    *i*) ;;\n      *) return;;\nesac\nherd7 x.litmus\n'),
                               ('home-unguarded137/.profile', '. {work}/home-unguarded137/.bashrc\n'),
                               ('home-unguarded137/.bashrc', 'if false; then\n  case $- in *i*) ;; *) return;; esac\nfi\nherd7 x.litmus\n'),
):
            os.makedirs(os.path.dirname(os.path.join(work, relative)), exist_ok=True)
            with open(os.path.join(work, relative), "w", encoding="utf-8") as handle:
                handle.write(text.replace("{work}", work))
        judge_cases([(label, writer, command.replace("{work}", work), want, want_notices, want_read) for label, command, want, want_notices, want_read in [
            ('d137-r2 A：sed 地址里的算术展开照 sed 判，放行', 'n=40; sed -n "$((n-3)),$((n))p" f', 0, 0, 0),
            ('d137-r2 A：sed 地址里的 $[ ] 算术照 sed 判，放行', 'sed -n "$[n-3],$[n+3]p" f', 0, 0, 0),
            ('d137-r2 A：sed 行号地址是命令替换，照 sed 判，放行', 'sed -n "$(grep -n x f | cut -d: -f1),+45p" f', 0, 0, 0),
            ('d137-r2 A：sed 算术地址后接 e 起全量，照拒', 'sed -n "$((n))e herd7 x.litmus" f', 2, 0, 0),
            ('d137-r2 A：sed 命令替换落在命令位上，照拒', 'sed -n "1$(echo p)" f', 2, 0, 0),
            ('d137-r2 A：sed 地址里的命令替换起全量，照拒', 'sed -n "$(herd7 x.litmus),+45p" f', 2, 0, 0),
            ('d137-r2 B：sed 的 c\\ 多行文本，放行', "sed '/x/c\\\nfoo' f", 0, 0, 0),
            ('d137-r2 B：sed 的 a\\ 续行文本，放行', "sed '/x/a\\\nfoo\\\nbar' f", 0, 0, 0),
            ('d137-r2 B：sed 文本之后另起一行 e 起全量，照拒', "sed '/x/a\\\nfoo\n1e herd7 x.litmus' f", 2, 0, 0),
            ('d137-r2 C：python argv 头一项字面、参数是变量，照命令词判，放行', 'python3 -c \'import subprocess,sys; subprocess.run(["ls", sys.argv[1]])\' x', 0, 0, 0),
            ('d137-r2 C：python argv 起 cargo test 而参数是变量，照拒', 'python3 -c \'import subprocess,sys; subprocess.run(["cargo", "test", sys.argv[1]])\' x', 2, 0, 0),
            ('d137-r2 C：python argv 头一项是变量，照拒', 'python3 -c \'import subprocess,sys; subprocess.run([sys.argv[1], "test"])\' x', 2, 0, 0),
            ('d137-r2 D：python 字面列表先赋变量再起，放行', "python3 - <<'PY'\nimport subprocess\nargs = ['git', 'status']\nsubprocess.run(args)\nPY", 0, 0, 0),
            ('d137-r2 D：python 字面串 .split() 作命令，放行', 'python3 -c \'import subprocess; subprocess.run("ls -l".split())\'', 0, 0, 0),
            ('d137-r2 D：python 先赋变量的字面列表起全量，照拒', "python3 - <<'PY'\nimport subprocess\nargs = ['herd7', 'x.litmus']\nsubprocess.run(args)\nPY", 2, 0, 0),
            ('d137-r2 D：python 列表赋过之后被改，认不出照拒', "python3 - <<'PY'\nimport subprocess\nargs = ['ls']\nargs.append('x')\nsubprocess.run(args)\nPY", 2, 0, 0),
            ('d137-r2 E：awk 里 cmd 赋字面再 | getline，放行', 'awk \'BEGIN { cmd = "date"; cmd | getline d; close(cmd) }\'', 0, 0, 0),
            ('d137-r2 E：awk 命令行 cmd=sort 再 print |，放行', "awk '{ print $1 | cmd }' cmd=sort f", 0, 0, 0),
            ('d137-r2 E：awk 里 cmd 赋全量再 | getline，照拒', 'awk \'BEGIN { cmd = "herd7 x.litmus"; cmd | getline d }\'', 2, 0, 0),
            ('d137-r2 E：awk 里 cmd 是拼接的，认不出照拒', 'awk \'BEGIN { cmd = "ls " $1; cmd | getline d }\'', 2, 0, 0),
            ('d137-r2 F：cd "$PWD" 之后 bash 轻脚本，放行', 'cd "$PWD" && bash r2-light.sh', 0, 0, 1),
            ('d137-r2 F：cd "$PWD" 之后 bash 重脚本，照拒', 'cd "$PWD" && bash r2-heavy.sh', 2, 0, 1),
            ('d137-r2 F：cd 到 git 顶层之后 bash 轻脚本，放行', 'cd gitrepo137/inner && cd "$(git rev-parse --show-toplevel)" && bash r2-light.sh', 0, 0, 1),
            ('d137-r2 F：cd 到 git 顶层之后 bash 重脚本，照拒', 'cd gitrepo137/inner && cd "$(git rev-parse --show-toplevel)" && bash r2-heavy.sh', 2, 0, 1),
            ('d137-r2 F：cd "$OLDPWD" 回来之后 bash 轻脚本，放行', 'cd gitrepo137 && cd "$OLDPWD" && bash r2-light.sh', 0, 0, 1),
            ('d137-r2 F：cd "$OLDPWD" 回来之后 bash 重脚本，照拒', 'cd gitrepo137 && cd "$OLDPWD" && bash r2-heavy.sh', 2, 0, 1),
            ('d137-r2 H5：cd 盘上不在的目录 && 相对脚本，cd 失败就不跑，放行', 'cd nonexist137 && bash r2-heavy.sh', 0, 1, 0),
            ('d137-r2 H5：cd 盘上不在的目录 || exit 之后的相对脚本，放行', 'cd nonexist137 || exit 1; bash r2-heavy.sh', 0, 1, 0),
            ('d137-r2 H5：cd 盘上不在的目录 ; 之后的相对脚本照跑，照拒', 'cd nonexist137; bash r2-heavy.sh', 2, 0, 0),
            ('d137-r2 H5：(cd 不在的目录 || exit) 之后的相对脚本照跑，照拒', '(cd nonexist137 || exit); bash r2-heavy.sh', 2, 0, 0),
            ('d137-r2 H5：cd 不在的目录 && ls || 相对脚本照跑，照拒', 'cd nonexist137 && ls || bash r2-heavy.sh', 2, 0, 0),
            ('d137-r2 H5：mkdir 再 cd 进去跑同一条里写出的重脚本，照拒', "mkdir -p made137 && cat > made137/w.sh <<'EOF'\nherd7 x.litmus\nEOF\ncd made137 && bash w.sh", 2, 0, 1),
            ('d137-r2 G：登录 bash -l 读 .profile 再 source 带非交互守卫的 .bashrc，守卫之后不读，放行', 'HOME={work}/home-guarded137 bash -l -c true', 0, 0, 2),
            ('d137-r2 G：交互的 bash -i 读带守卫的 .bashrc 不截，守卫之后的全量照拒', 'HOME={work}/home-guarded137 bash -i -c true', 2, 0, 1),
            ('d137-r2 G：守卫不在顶层（套在 if 里）不截，照拒', 'HOME={work}/home-unguarded137 bash -l -c true', 2, 0, 2),
        ]])
        # 认得出起的是什么的照放，同形起重活或认不出的照拒。
        # E2 格 PROJECT_CODE_GUARD_BREAK=launcher-recorded-as-writer、E3 格 heredoc-targets-unexpanded、E5 格 program-file-variables-unexpanded、
        # E4 格 LIB_SHELL_WORDS_BREAK=aliases-same-line-expanded、口径格 LIB_PROJECT_CODE_RUNS_BREAK=claude-executable-path-unrecognized 下「放行」的格红（逐格见文件头开关清单）
        judge_cases([(label, agent_type, command.replace("{work}", work), want, want_notices, want_read)
                     for label, agent_type, command, want, want_notices, want_read in [
            ("d137 闸误拒 E2：systemd-run --user --wait 包着 bash 轻脚本，放行", writer, "systemd-run --user --wait bash startup-light.sh", 0, 0, 1),
            ("d137 闸误拒 E2：chrt -i 0 包着 bash 轻脚本，放行", writer, "chrt -i 0 bash startup-light.sh", 0, 0, 1),
            ("d137 闸误拒 E2：ssh localhost 包着 bash 轻脚本，放行", writer, "ssh localhost bash startup-light.sh", 0, 0, 1),
            ("d137 闸误拒 E2：/usr/bin/time 包着 bash 轻脚本，放行", writer, "/usr/bin/time bash startup-light.sh", 0, 0, 1),
            ("d137 闸误拒 E2：strace -o /dev/null 包着 bash 轻脚本，放行", writer, "strace -o /dev/null bash startup-light.sh", 0, 0, 1),
            ("d137 闸误拒 E2：prlimit --as 包着 bash 轻脚本，放行", writer, "prlimit --as=4000000000 bash startup-light.sh", 0, 0, 1),
            ("d137 闸误拒 E2：numactl -N0 包着 bash 轻脚本，放行", writer, "numactl -N0 bash startup-light.sh", 0, 0, 1),
            ("d137 闸误拒 E2：unshare --user 包着 bash 轻脚本，放行", writer, "unshare --user bash startup-light.sh", 0, 0, 1),
            ("d137 闸误拒 E2：systemd-run --user --wait 包着 bash 重脚本，读正文照拒", writer, "systemd-run --user --wait bash startup-heavy.sh", 2, 0, 1),
            ("d137 闸误拒 E3：t 赋过字面路径，cat > $t 读 heredoc 写出再 bash $t，放行", writer, "t={work}/e3-written137.sh; cat > $t <<'EOF'\necho hi\nEOF\nbash $t", 0, 0, 1),
            ("d137 闸误拒 E3：$t 没赋过，cat > $t 写出再 bash $t，照拒", writer, "cat > $t <<'EOF'\necho hi\nEOF\nbash $t", 2, 0, 0),
            ("d137 闸误拒 E4：同一行先 alias 再用，这一行不展开别名、放行", writer, "shopt -s expand_aliases; alias t='herd7 x.litmus'; t", 0, 0, 0),
            ("d137 闸误拒 E5：W 赋过字面目录，awk -f $W/程序文件，读正文放行", writer, "W={work}; awk -f $W/prog-light137.awk f", 0, 0, 1),
            ("d137 闸误拒 E5：W 没赋过，awk -f $W/程序文件认不出，照拒", writer, "awk -f $W/prog-light137.awk f", 2, 0, 0),
            ("d137 闸口径：主 agent 经 \"$CLAUDE_CODE_EXECPATH\" 起 claude，交检出钩子判，放行", None, '"$CLAUDE_CODE_EXECPATH" -p hi', 0, 0, 0),
        ]])
        # 读进来的脚本正文里命令词或脚本路径由变量拼出、解不出，而定义或起它的参数里写着自证、门禁阶段、入口脚本或 cargo 的，照认不出办（例：
        # 草稿里一份脚本 `R=$1; cmds=("bash $R/.claude/hooks/… --selftest" …); … ${cmds[$i]}` 起改过的钩子自证；
        # LIB_PROJECT_CODE_RUNS_BREAK=script-body-launches-unjudged 下「照拒」的三格红，应当是 2，实际 0；cargo 参数解不出那一种由 lib_project_code_runs.py 的格钉住）。直接写 `bash "$R/…/x.sh" --selftest` 按登记表照拒，不另立格；
        # 本仓里的脚本不判这一条（照旧放行那一格）
        for relative, text in (("body-array-selftest.sh", '#!/usr/bin/env bash\nR=$1\ncmds=("bash $R/.claude/hooks/project-code-guard.sh --selftest" "python3 $R/.claude/hooks/lib_project_code_runs.py --selftest")\n'
                                                          'for i in "${!cmds[@]}"; do env "$@" timeout 900 nice -n 19 ${cmds[$i]} > "out-$i" 2>&1; done\n'),
                               ("body-forward.sh", '#!/usr/bin/env bash\n"$@"\n'),
):
            with open(os.path.join(work, relative), "w", encoding="utf-8") as handle:
                handle.write(text)
        # 样本工作区平常照「本仓」认（FAMILY_OWN_REPOSITORY），这几格要它当草稿目录：判的时候摘掉，判完放回
        project_code_runs.FAMILY_OWN_REPOSITORY = None
        for label, agent_type, command, want in (
                ("一百三十七：脚本正文里经数组变量起拼出路径的钩子自证（件 1 那三趟的形态），照拒", writer, f"bash body-array-selftest.sh {work}/tree", 2),
                ("一百三十七：同一份脚本主 agent 照拒", None, f"bash body-array-selftest.sh {work}/tree", 2),
                ):
            hook_input = {"tool_name": "Bash", "cwd": work, "tool_input": {"command": command}}
            if agent_type:
                hook_input["agent_type"] = agent_type
            results.append((label, want, scan_decision(hook_input, work).code))
        project_code_runs.FAMILY_OWN_REPOSITORY = work
        # 本仓里的脚本不判这一条：还是上面「照拒」那一格的 body-array-selftest.sh 与同一条命令，这时样本工作区照「本仓」认，经变量起的自证放行，两格只差这一点。
        # 不拿仓里真实的脚本当样本：它们的正文随改动会出现字面的要拒的命令（multi-host-run.sh 自证段里有 cargo test），正文照判就拒，这一格就钉不住「经变量起的不判」了
        own_repository_scene = scan_decision({"tool_name": "Bash", "cwd": work, "agent_type": writer, "tool_input": {"command": f"bash body-array-selftest.sh {work}/tree"}}, work)
        results.append(("一百三十七：本仓里的脚本正文经变量起包装与命令，不判这一条，照旧放行", 0, own_repository_scene.code))
        # cat 的源里有不在的（修补报告 debt-tw-shell-words-fix2 第 93 行那一类）：照拒，拒绝说明点名不在的源与改法（PROJECT_CODE_GUARD_BREAK=cat-missing-source-unexplained 下红）
        missing_source = scan_decision({"tool_name": "Bash", "cwd": work, "agent_type": writer, "tool_input": {"command": 'cat s.sh "5"> d2.sh && bash d2.sh'}}, work)
        results.append(("一百三十七：cat 源里有不在的文件、&& 接起，照拒并点名不在的源", (2, True),
                        (missing_source.code, "盘上不在" in (missing_source.message or "") and "`5`" in (missing_source.message or ""))))
        # cat 追加到盘上读不了的文件（不是 UTF-8 文本的、目录）：那个目的不认，判照常走完；pieces 是 None 还接着 append 的话，判的那一刻抛 AttributeError、整条命令照常执行
        # （例：子 agent 的 `grep -c '    "X' 文件 && … cat 源 >> 超过读的上限的 mutations.tsv`；抛错那一遍引号只是同一条命令里的别的词）。
        # PROJECT_CODE_GUARD_BREAK=cat-append-unreadable-destination-raises 下抛错，这几格拿到的不是 want
        with open(os.path.join(work, "unreadable-binary.tsv"), "wb") as handle:
            handle.write(b"\xff\xfe\x00")
        with open(os.path.join(work, "append-source.txt"), "w", encoding="utf-8") as handle:
            handle.write("追加的一行\n")
        os.makedirs(os.path.join(work, "append-directory"), exist_ok=True)
        for label, agent_type, command, want in (
                ("一百三十七：cat 追加到不是文本的文件之后接 herd7 x.litmus，照拒", writer, "cat append-source.txt >> unreadable-binary.tsv && herd7 x.litmus", 2),
                ("一百三十七：同一条主 agent 起，照拒", None, "cat append-source.txt >> unreadable-binary.tsv && herd7 x.litmus", 2)):
            hook_input = {"tool_name": "Bash", "cwd": work, "tool_input": {"command": command}}
            if agent_type:
                hook_input["agent_type"] = agent_type
            try:
                code = scan_decision(hook_input, work).code
            except Exception as error:
                code = f"判的时候抛了 {error!r}"
            results.append((label, want, code))
        # 同一份文件一次判定里只从盘上读一遍；直接执行的 .rs 要读开头判是不是脚本，判完不当脚本读，它里面括着的文件不碰
        def disk_reads(command, relative, agent_type=writer):
            hook_input = {"tool_name": "Bash", "cwd": work, "tool_input": {"command": command}}
            if agent_type:
                hook_input["agent_type"] = agent_type
            return scan_decision(hook_input, work).files_read.count(os.path.realpath(os.path.join(work, relative)))
        for label, command, relative, agent_type, want in [
                ("同一条命令里起 s.sh 两遍：s.sh 从盘上读一遍", "bash s.sh; ./s.sh", "s.sh", writer, 1),
                ("起的脚本里三种起法各起一遍 s.sh：s.sh 从盘上读一遍", "bash twice.sh", "s.sh", writer, 1),
                ]:
            results.append((f"{label}（读盘次数）", want, disk_reads(command, relative, agent_type)))
        # 复现输入那一条要在 SLOW_REPRODUCTION_SECONDS 秒内判完
        started = time.monotonic()
        scan_decision({"tool_name": "Bash", "cwd": work, "agent_type": writer, "tool_input": {"command": SLOW_REPRODUCTION_COMMAND}}, work)
        elapsed = time.monotonic() - started
        results.append((f"复现输入那一条在 {SLOW_REPRODUCTION_SECONDS} 秒内判完（这一遍 {elapsed:.3f} 秒）", 1, int(elapsed < SLOW_REPRODUCTION_SECONDS)))
        # 拒绝说明写出脚本路径与行号：直接起的写到 s.sh 第 4 行，嵌套的连外层起它的那一行一起写
        for label, command, location in [("拒绝说明写出脚本路径与行号", "bash s.sh", f"脚本 {work}/s.sh:4 里的 herd7"),
                                         ("嵌套的拒绝说明写出每一层的路径与行号", "bash outer.sh", f"脚本 {work}/outer.sh:3 → {work}/s.sh:4 里的 herd7"),
                                         ("命令替换里的拒绝说明写出它所在那一行", "bash substitution-heavy.sh", f"脚本 {work}/substitution-heavy.sh:3 里的 herd7"),
                                         ("heredoc 写出的脚本，拒绝说明写出它的路径与正文里的行号",
                                          "cat > gen.sh <<'EOF'\n#!/usr/bin/env bash\nherd7 x.litmus\nEOF\nbash gen.sh", f"脚本 {work}/gen.sh:2 里的 herd7"),
                                         ("heredoc 写出的脚本里再写出的，逐层写出路径与行号",
                                          "cat > gen-outer.sh <<'EOF'\ncat > gen-inner.sh <<'IN'\nherd7 x.litmus\nIN\nbash gen-inner.sh\nEOF\nbash gen-outer.sh",
                                          f"脚本 {work}/gen-outer.sh:4 → {work}/gen-inner.sh:1 里的 herd7"),
                                         ("复用上一遍的结果，拒绝说明照这一处的路径与行号写",
                                          "bash s.sh; bash outer.sh", f"脚本 {work}/outer.sh:3 → {work}/s.sh:4 里的 herd7"),
                                         ("绕回外层的那一遍不复用，从别处起时照读回来", "bash cyc-a.sh; bash cyc-b.sh",
                                          f"脚本 {work}/cyc-b.sh:1 → {work}/cyc-a.sh:2 里的 herd7")]:
            message = scan_decision({"tool_name": "Bash", "cwd": work, "agent_type": writer, "tool_input": {"command": command}}, work).message or ""
            results.append((label, 1, int(location in message and "这一步写在脚本里" in message)))
        # 复用上一遍的检出，照这一处的路径与行号写
        reused_notices = scan_decision({"tool_name": "Bash", "cwd": work, "agent_type": writer,
                                 "tool_input": {"command": "bash has-missing.sh; bash outer-missing.sh"}}, work).notices
        results.append(("复用上一遍的检出，照这一处的路径与行号写", 1,
                        int(any(f"脚本 {work}/outer-missing.sh:2 → {work}/has-missing.sh:1 里" in notice for notice in reused_notices))))
        # 切词：source / . 后面跟的像一个路径才算读脚本（python 源码里 `source = …` 那种不算）
        for label, command, want in [("切词：source = … 不算读脚本", "source = open", [("source", None)]),
                                     ("切词：. = … 不算读脚本", ". = x", [(".", None)]),
                                     ("切词：. ./env.sh 算读脚本", ". ./env.sh", [("./env.sh", "source")])]:
            got = [(found.words[0], found.launcher) for found in shell_words.commands_at_command_position(command, work).commands]
            results.append((label, want, got))
        # 切词：进程替换的另一种写法 `>(…)` 同 `<(…)`：里面的命令另判，`)` 之后的词是外层命令的参数
        got = [(found.words[0], found.launcher) for found in shell_words.commands_at_command_position("tee >(bash run.sh) out.log", work).commands]
        results.append(("切词：tee >(bash run.sh) out.log 里 run.sh 另判、out.log 是 tee 的参数", [("run.sh", "shell"), ("tee", None)], got))
        # 切词：heredoc 正文留不留，看它挂在哪条命令上（从标准输入读命令的 bash / sh 才留），不看同一行别处有没有 shell 的名字
        for label, command, kept in [("切词：前一条起了 x.sh、heredoc 喂给 python3 -：正文剥掉", "bash light.sh && python3 - <<'PY'\nBODY\nPY", False),
                                     ("切词：heredoc 喂给 bash：正文留着", "bash <<'EOF'\nBODY\nEOF", True),
                                     ("切词：heredoc 喂给 capped.sh 4 bash -s：正文留着", "capped.sh 4 bash -s -- a <<'EOF'\nBODY\nEOF", True),
                                     ("切词：heredoc 喂给 bash -c：正文剥掉", "bash -c 'cat' <<'EOF'\nBODY\nEOF", False),
                                     ("切词：heredoc 喂给 bash 起的脚本：正文剥掉", "bash light.sh <<'EOF'\nBODY\nEOF", False)]:
            results.append((label, kept, "BODY" in shell_words.strip_data_heredocs(command)))
        # 共用判定模块自己的自检（cargo 命令行与直接执行的测试二进制两种输入）
        results.append(("lib_project_code_runs.py 的自检", 0, project_code_runs.selftest()))
        results.append(("检出记录默认放在工作根下（lib_work_root.py）",
                        os.path.join(work_roots.configured_work_root(), "agent-hook-detections.jsonl") if work_roots else "读得到 lib_work_root.py", DEFAULT_DETECTIONS))
        # 文件头「经 task-server 起」：一切跑项目代码的命令只认经 task-server/task 起（①），子 agent 起客户端只许 status、log（②），绕开服务的写法照拒（③）；
        # 弄坏开关：PROJECT_CODE_GUARD_BREAK=task-server-direct-allowed 下「：照拒」的直接跑那几格红，subagent-task-client-allowed 下子 agent 起客户端「照拒」那几格红，
        # task-server-bypass-allowed 下「绕开：」开头照拒的那几格红；LIB_PROJECT_CODE_RUNS_BREAK=project-scripts-unjudged 下「本仓脚本」那几格红、text-tools-refused 下「名单：」放行那几格红、
        # main-agent-tools-refused 下「名单：主 agent」放行那几格红、task-server-submission-unparsed 下「提交」照拒的那几格红（逐格写在文件头开关表）
        client_source = os.path.join(repo, project_code_runs.TASK_SERVER_CLIENT)
        results.append(("经 task-server：本仓有提交客户端 task-server/task", True, os.path.isfile(client_source)))
        if os.path.isfile(client_source):
            os.makedirs(os.path.join(work, "task-server"), exist_ok=True)
            shutil.copy2(client_source, os.path.join(work, project_code_runs.TASK_SERVER_CLIENT))
        for relative, text in (("own-helper-light.sh", "#!/usr/bin/env bash\n# 草稿里自己的辅助脚本：只读文本\ngrep -c x note.md\n"),
                               ("own-helper-build.sh", "#!/usr/bin/env bash\n# 草稿里自己的辅助脚本：里面直接编\ncargo build --offline -p singlefs-verification\n"),
                               ("own-helper-submits.sh", "#!/usr/bin/env bash\n# 草稿里自己的辅助脚本：里面经 task-server 提交\ntask-server/task submit -- cargo build --offline -p singlefs-verification\n"),
                               ("renamed-project-script.py", "#!/usr/bin/env python3\n# admission: always 样本：改名拷走的本仓脚本\n# run-condition: none 样本\nprint(1)\n")):
            with open(os.path.join(work, relative), "w", encoding="utf-8") as handle:
                handle.write(text)
        runner = "experiment-runner"
        # 服务的编译目录（直接删照拒，task-server/task clean 放行）：取 lib_project_code_runs 从 task_server_lib.py 读出的那一处，读不出照缺省的位置造（那几格照样判红）
        guarded_now, _guarded_problem = project_code_runs.task_server_guarded()
        service_builds = (guarded_now.build_directories[0] if guarded_now is not None and guarded_now.build_directories
                          else os.path.expanduser("~/.local/share/singlefs/task-server-builds"))
        swap = f"python3 {repo}/scripts/hosts/compile-then-swap.py {work}/draft.rs crates/singlefs-verification/src/bin/e161_sample_device.rs --scratch {work}/swap"
        # 文件头「经 task-server 起」④ 的说明：点名登记表、给整条包装写法（主 agent 与子 agent 各一格）；子 agent 的出路是写进报告「要主 agent 跑的命令」一节
        # （PROJECT_CODE_GUARD_BREAK=temporary-service-remedy-dropped 下两格红，LIB_PROJECT_CODE_RUNS_BREAK=temporary-service-command-bare 下两格红）
        bare_command = ("task-server/task submit -- python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service "
                        "-- python3 scripts/mutation/crates-mutation-rows.py --selftest")
        bare_submitted = task_server_decision({"tool_name": "Bash", "cwd": repo,
                                               "tool_input": {"command": "task-server/task submit -- python3 scripts/mutation/crates-mutation-rows.py --selftest"}}, repo)
        results.append(("经 task-server：要临时服务的自证裸写被拒时，说明点名登记表、给整条包装写法", True,
                        bare_submitted.code == 2 and all(piece in (bare_submitted.message or "") for piece in (
                            "✗ 要临时 prune-server 的自证写成了裸的 --selftest 被拒", "selftest-temporary-service.tsv", bare_command, "→ 怎么办："))))
        bare_direct = task_server_decision({"tool_name": "Bash", "cwd": repo, "agent_type": writer,
                                            "tool_input": {"command": "python3 scripts/mutation/crates-mutation-rows.py --selftest"}}, repo)
        results.append(("经 task-server：子 agent 直接裸写要临时服务的自证被拒时，说明写进报告「要主 agent 跑的命令」一节、给整条包装写法", True,
                        bare_direct.code == 2 and all(piece in (bare_direct.message or "") for piece in (
                            "✗ 要临时 prune-server 的自证写成了裸的 --selftest 被拒", bare_command, "→ 怎么办：照上面「整条写成」的那一条", "写进报告「要主 agent 跑的命令」一节"))))
        # 子 agent 交任务被拒，拒绝说明写明出路：写进报告「要主 agent 跑的命令」一节交回、不另起 run_in_background（subagent-task-client-allowed 下这一格红）
        submitted_item = task_server_decision({"tool_name": "Bash", "cwd": repo, "agent_type": writer,
                                               "tool_input": {"command": "task-server/task submit -- bash scripts/crash/crash-point-replay.sh --item fixed-script"}}, repo)
        results.append(("经 task-server：子 agent 交任务的拒绝说明写明写进报告「要主 agent 跑的命令」一节交回", True,
                        submitted_item.code == 2 and all(piece in (submitted_item.message or "") for piece in (
                            f"✗ {writer} 起统一任务服务的客户端被拒", "写进报告「要主 agent 跑的命令」一节交回", "status、log"))))
        # 绕开服务的写法（③）：读签名密钥拒，说明点名绕开写法（task-server-bypass-allowed 下这一格红）
        bypass_read = task_server_decision({"tool_name": "Bash", "cwd": repo, "tool_input": {"command": "cat ~/.config/singlefs/task-server-signing.key"}}, repo)
        results.append(("经 task-server：绕开：读签名密钥照拒，说明点名绕开写法", True,
                        bypass_read.code == 2 and "✗ 绕开 task-server 的写法被拒" in (bypass_read.message or "")))
        # 要临时服务的自证套了临时服务的包装（给了 --server-binary）提交：包装剥开重判里面那条时补上标记，放行；自己起服务的自证（own-server）裸写提交也放行
        # （temporary-service-wrapper-not-marked 与 LIB_PROJECT_CODE_RUNS_BREAK 的 temporary-service-marker-ignored、temporary-service-environment-dropped 下第一格红，
        # LIB_PROJECT_CODE_RUNS_BREAK=temporary-service-own-server-as-required 下第二格红）
        wrapped_submit = task_server_decision({"tool_name": "Bash", "cwd": repo, "tool_input": {"command": (
            "task-server/task submit -- python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service --server-binary /sample/prune-server "
            "-- python3 scripts/mutation/crates-mutation-rows.py --selftest")}}, repo)
        results.append(("经 task-server：主 agent 提交包着临时服务（给了 --server-binary）的要临时服务的自证：放行", 0, wrapped_submit.code))
        own_server_submit = task_server_decision({"tool_name": "Bash", "cwd": repo, "tool_input": {
            "command": "task-server/task submit -- python3 scripts/prune-server/prune-server-load-simulation.py --selftest"}}, repo)
        results.append(("经 task-server：主 agent 提交自己起服务的自证（own-server）裸写：放行", 0, own_server_submit.code))
        # python3 -c 在管道里读 JSON、往标准输出打字，代码里的 'task' 字样不是写了后面执行的 task-server/task；
        # 真往 task-server/task 写了再执行的照拒（PROJECT_CODE_GUARD_BREAK=python-filter-code-judged 下前两格红）
        for label, command, want in (
                ("经 task-server：管道读 JSON 的 python3 -c 代码里写着 'task'，不当写了后面执行的 task-server/task：主 agent 放行",
                 "task-server/task wait 7191 >/dev/null 2>&1; task-server/task status 7191 --json | python3 -c \"import json,sys; d=json.load(sys.stdin); print(d['task'].get('compiles'), d['attempts'][0].get('host'))\"; "
                 "task-server/task log 7191 --all 2>&1 | grep -E '✗|PRUNE_SERVER_TEMPORARY|error' | head -6",
                 0),
                ("经 task-server：循环里管道读 JSON 的 python3 -c 代码里写着 'task'，不当写了 task-server/task：主 agent 放行",
                 "for n in 6778 6780; do task-server/task status $n --json | python3 -c \"\nimport sys,json\nd=json.load(sys.stdin)\na=d['attempts'][0]\n"
                 "print('$n', {k:a.get(k) for k in ('host','build_directory','build_directory_removed','build_peak','memory_peak','state')}, 'task.env CARGO_TARGET_DIR=', d['task']['environment'].get('CARGO_TARGET_DIR'))\n\"; done; "
                 "task-server/task status 6778 --json | python3 -c \"import sys,json;d=json.load(sys.stdin);print(sorted(d['attempts'][0].keys()))\"",
                 0),
                ("经 task-server：python3 -c 往 task-server/task 写了再执行它，照拒",
                 "python3 -c \"open('task-server/task','w').write('true')\"; task-server/task status 7191",
                 2)):
            results.append((label, want, task_server_decision({"tool_name": "Bash", "cwd": repo, "tool_input": {"command": command}}, repo).code))
        # 照抄的那一条：直接跑的 harness 测试给出提交的写法，照主 agent 交它再判放行；范围声明挪到 submit 前面、内存包装去掉。
        # 子 agent 那一份的头一行是「→ 写进报告「要主 agent 跑的命令」一节的那一条」（它自己不交，②），照抄的那条照样由主 agent 交、照主 agent 判。
        # 挑目标的 cargo test 是要声明范围的验证命令，经 submit 交的照样要声明、声明写在 submit 前面（run-scope-guard.sh 文件头「经 task-server 提交」，
        # lib_run_scope.py 的 scope_runs_in_text 剥开 submit 照同一张表判）：照抄的那条带四段声明（为什么一段是占位，run_scope_declaration），
        # 照抄的那条另拿范围声明闸的判法（lib_run_scope.decide_scope）判一遍也要放行（PROJECT_CODE_GUARD_BREAK=copy-without-run-scope 下头一格与第三格红）
        def run_scope_code(hook_input):
            try:
                run_scope_spec = importlib.util.spec_from_file_location("lib_run_scope_for_copy", os.path.join(hook_dir, "lib_run_scope.py"))
                run_scope = importlib.util.module_from_spec(run_scope_spec)
                run_scope_spec.loader.exec_module(run_scope)
                run_scope.load_judging_libraries(hook_dir)
                return run_scope.decide_scope(hook_input, work).code
            except Exception as error:
                return f"范围声明闸的判法载不进或判的时候抛了 {error!r}"
        placeholder_reason = "为什么=<为什么这么跑、这个范围为什么合适>'"
        for label, agent_type, command, want_copy in (
                ("经 task-server：照抄：实现员直接跑 harness 测试，照抄的那条经 task-server 提交", writer,
                 "cargo test -p singlefs-verification --test overwrite_in_one_instance",
                 "SINGLEFS_RUN_SCOPE='种类=target; 跑什么=cargo test -p singlefs-verification 的测试目标 --test overwrite_in_one_instance; 范围=1 个测试目标; "
                 f"{placeholder_reason} task-server/task submit -- cargo test -p singlefs-verification --test overwrite_in_one_instance"),
                ("经 task-server：照抄：经内存包装直接起的去掉包装", None,
                 "bash scripts/hosts/run-with-memory-cap.sh 8G cargo build -p singlefs-verification", "task-server/task submit -- cargo build -p singlefs-verification"),
                ("经 task-server：照抄：带 && 的整条包进 bash -c", writer,
                 "cd crates/singlefs-verification && cargo test --lib",
                 f"SINGLEFS_RUN_SCOPE='种类=target; 跑什么=cargo test 的测试目标 --lib; 范围=1 个测试目标; {placeholder_reason} "
                 "task-server/task submit -- bash -c 'cd crates/singlefs-verification && cargo test --lib'")):
            hook_input = {"tool_name": "Bash", "cwd": work, "tool_input": {"command": command}}
            if agent_type:
                hook_input["agent_type"] = agent_type
            lines = (task_server_decision(hook_input, work).message or "").splitlines()
            copy_heading = "→ 写进报告「要主 agent 跑的命令」一节的那一条" if agent_type else "→ 照抄"
            got_copy = lines[1] if len(lines) > 1 and lines[0].startswith(copy_heading) else ""
            refed = {"tool_name": "Bash", "cwd": work, "tool_input": {"command": got_copy}}
            results.append((label, (want_copy, 0, 0), (got_copy, task_server_decision(refed, work).code if got_copy else None,
                                                       run_scope_code(refed) if got_copy else None)))
        # 走真实入口：从标准输入喂 JSON，拒绝退出码 2、stderr 带出路，放行退出码 0；拒绝落进检出记录
        script = os.path.join(hook_dir, "project-code-guard.sh")
        detections = os.path.join(work, "detections.jsonl")
        environment = dict(os.environ, CLAUDE_PROJECT_DIR=work, AGENT_HOOK_DETECTIONS=detections)
        def count_detections():
            return sum(1 for _ in open(detections, encoding="utf-8")) if os.path.exists(detections) else 0
        refused = subprocess.run(["bash", script], capture_output=True, text=True, env=environment,
                                 input=json.dumps({"tool_name": "Bash", "cwd": work, "agent_type": writer, "tool_input": {"command": "cargo test --all"}}))
        results.append(("stdin:实现员跑 --all 拒绝（退出码 2）", 2, refused.returncode))
        # 出路写规矩（一切跑项目代码只认经 task-server/task 起）与子 agent 的去处（写进报告「要主 agent 跑的命令」一节交回）
        results.append(("stdin:拒绝时 stderr 写了出路：规矩一句、子 agent 写进报告交回", 1,
                        int("→" in refused.stderr and "一切跑项目代码的命令只认经 task-server/task 起" in refused.stderr and "写进报告「要主 agent 跑的命令」一节" in refused.stderr)))
        # 走真实入口：主 agent 直接跑被拒，stderr 最前面是照抄的说明与经 task-server 提交的那条（照它再喂一遍放行），不贴「谁能跑哪一类」那一段；
        # 检出记录写结局「被拒」（PROJECT_CODE_GUARD_BREAK=task-server-copy-dropped 下第一格红，detection-outcome-dropped 下第三格红）
        def last_detection():
            """检出记录的最后一条；还没有记录时交 {}（那几格照样判红，不让自检抛异常）。"""
            if not os.path.exists(detections):
                return {}
            with open(detections, encoding="utf-8") as handle:
                lines = handle.read().splitlines()
            return json.loads(lines[-1]) if lines else {}
        main_command = "bash .claude/singlefs-ai-sop/scripts/doc-lint.sh ."
        main_refused = subprocess.run(["bash", script], capture_output=True, text=True, env=environment,
                                      input=json.dumps({"tool_name": "Bash", "cwd": work, "tool_input": {"command": main_command}}))
        main_lines = main_refused.stderr.splitlines()
        main_outcome = last_detection().get("outcome")
        copied = main_lines[1] if len(main_lines) > 1 and main_lines[0].startswith("→ 照抄") else ""
        copied_judged = subprocess.run(["bash", script], capture_output=True, text=True, env=environment,
                                       input=json.dumps({"tool_name": "Bash", "cwd": work, "tool_input": {"command": copied}}))
        results.append(("stdin:主 agent 直接跑被拒时最前面是能照抄的经 task-server 提交的那条（照抄的那条再判放行）", (2, "task-server/task submit -- " + main_command, 0),
                        (main_refused.returncode, copied, copied_judged.returncode)))
        results.append(("stdin:主 agent 被拒时不贴「谁能跑哪一类」那一段，规矩指到提交写法", True,
                        "task-server/README.md「提交写法」" in main_refused.stderr
                        and "→ 交回前不跑它们" not in main_refused.stderr and "→ 规矩只贴出处" not in main_refused.stderr))
        results.append(("stdin:被拒的检出记录写了结局「被拒」", "被拒", main_outcome))
        # 走真实入口：判不了的照常放行、stderr 报一句、记一条检出（结局「只记」）——hook 输入不是 JSON 对象（坏 JSON、解出来是数组；会话号写读不到，
        # 诊断带钩子名与开头的字节）、判的时候抛异常（tool_input.command 不是字符串，decide 里切词抛 TypeError）、here-string 喂给 bash 的是变量。
        # PROJECT_CODE_GUARD_BREAK=failure-unrecorded 下前三格红，disable-check 下第三格红，LIB_SHELL_WORDS_BREAK=shell-standard-input-unread 下第四格红
        for label, raw in (("stdin:坏 JSON 放行（退出码 0）、stderr 写诊断、记一条检出", b'{"tool_name": "Bash", "tool_input": '),
                           ("stdin:不是 JSON 对象放行（退出码 0）、stderr 写诊断、记一条检出", b'["Bash"]')):
            recorded = count_detections()
            unreadable = subprocess.run(["bash", script], capture_output=True, env=environment, input=raw)
            stderr_text = unreadable.stderr.decode("utf-8", "replace")
            results.append((label, (0, True, 1, "读不到", "只记"),
                            (unreadable.returncode, "project-code-guard.sh 收到的 hook 输入不是 JSON 对象" in stderr_text and repr(raw[:10])[2:-1] in stderr_text,
                             count_detections() - recorded, last_detection().get("session_id"), last_detection().get("outcome"))))
        recorded = count_detections()
        raising = subprocess.run(["bash", script], capture_output=True, text=True, env=environment,
                                 input=json.dumps({"tool_name": "Bash", "cwd": work, "session_id": "s-raise", "agent_type": writer,
                                                   "tool_input": {"command": ["cargo", "test", "--all"]}}))
        results.append(("stdin:判的时候抛异常放行（退出码 0）、stderr 写诊断、记一条检出", (0, True, 1, "s-raise", "只记"),
                        (raising.returncode, "project-code-guard.sh 没判成" in raising.stderr, count_detections() - recorded,
                         last_detection().get("session_id"), last_detection().get("outcome"))))
        recorded = count_detections()
        fed_variable = subprocess.run(["bash", script], capture_output=True, text=True, env=environment,
                                      input=json.dumps({"tool_name": "Bash", "cwd": work, "session_id": "s-fed", "agent_type": writer,
                                                        "tool_input": {"command": 'bash <<< "$CMD"'}}))
        results.append(("stdin:here-string 喂给 bash 的是变量认不出照拒（退出码 2）、stderr 点名判不了、记一条检出", (2, True, 1, "s-fed"),
                        (fed_variable.returncode, "here-string `$CMD`" in fed_variable.stderr, count_detections() - recorded,
                         last_detection().get("session_id"))))
        # 钩子 bash 这一层起不来时也记一条检出（PROJECT_CODE_GUARD_BREAK=bash-layer-unrecorded 下红）：PATH 前面放一份只对 /dev/fd/3 退 127 的假 python3
        shim = os.path.join(work, "bash-layer-shim")
        os.makedirs(shim, exist_ok=True)
        with open(os.path.join(shim, "python3"), "w", encoding="utf-8") as handle:
            handle.write(f'#!/bin/bash\nif [ "$1" = /dev/fd/3 ]; then exit 127; fi\nexec {sys.executable} "$@"\n')
        os.chmod(os.path.join(shim, "python3"), 0o755)
        recorded = count_detections()
        bash_layer = subprocess.run(["bash", script], capture_output=True, text=True, env=dict(environment, PATH=shim + os.pathsep + os.environ.get("PATH", "")),
                                    input=json.dumps({"tool_name": "Bash", "cwd": work, "session_id": "s-bash-layer", "agent_type": writer, "tool_input": {"command": "ls"}}))
        results.append(("一百三十七：钩子 bash 这一层起不来（假 python3 退 127）：放行（退出码 0）、记一条检出", (0, 1, "s-bash-layer", "只记"),
                        (bash_layer.returncode, count_detections() - recorded, last_detection().get("session_id"), last_detection().get("outcome"))))
        # PATH 上没有 python3（PROJECT_CODE_GUARD_BREAK=python-missing-unrecorded 下红）：PATH 只放记检出要用的 cat、date、mkdir、dirname
        no_python = os.path.join(work, "no-python-path")
        os.makedirs(no_python, exist_ok=True)
        for tool in ("cat", "date", "mkdir", "dirname"):
            if shutil.which(tool) and not os.path.lexists(os.path.join(no_python, tool)):
                os.symlink(shutil.which(tool), os.path.join(no_python, tool))
        recorded = count_detections()
        no_python_run = subprocess.run([shutil.which("bash"), script], capture_output=True, text=True, env=dict(environment, PATH=no_python),
                                       input=json.dumps({"tool_name": "Bash", "cwd": work, "session_id": "s-no-python", "agent_type": writer,
                                                         "tool_input": {"command": "cargo test --all"}}))
        results.append(("d137 H13：PATH 上没有 python3：放行（退出码 0）、记一条检出", (0, 1, "s-no-python", "只记"),
                        (no_python_run.returncode, count_detections() - recorded, last_detection().get("session_id"), last_detection().get("outcome"))))
        # 检出记录的位置照 lib_work_root.py 取、不退回家目录：AGENT_HOOK_DETECTIONS、SINGLEFS_WORK_ROOT、SINGLEFS_DATA_ROOT 都不设，PATH 上没有 python3，HOME 换成临时目录
        unrooted_home = os.path.join(work, "unrooted-home")
        os.makedirs(unrooted_home, exist_ok=True)
        unrooted_environment = {name: value for name, value in environment.items() if name not in ("AGENT_HOOK_DETECTIONS", "SINGLEFS_WORK_ROOT", "SINGLEFS_DATA_ROOT")}
        unrooted_run = subprocess.run([shutil.which("bash"), script], capture_output=True, text=True, env=dict(unrooted_environment, HOME=unrooted_home, PATH=no_python),
                                      input=json.dumps({"tool_name": "Bash", "cwd": work, "session_id": "s-unrooted", "agent_type": writer,
                                                        "tool_input": {"command": "cargo test --all"}}))
        results.append(("d137 H13：PATH 上没有 python3、工作根取不到：放行（退出码 0）、不往家目录下写、stderr 说检出没记成", (0, False, True),
                        (unrooted_run.returncode, os.path.exists(os.path.join(unrooted_home, ".local")), "检出没记成" in unrooted_run.stderr)))
        # 走真实入口：照抄的那条是门禁这类验证命令时带四段范围声明（.claude/hooks/run-scope-guard.sh 的写法，为什么一段是占位），照它再喂一遍放行
        # （PROJECT_CODE_GUARD_BREAK=copy-without-run-scope、task-server-copy-dropped、disable-check 下红）
        stage_command = "bash .claude/gate.d/lkmm.sh"
        stage_refused = subprocess.run(["bash", script], capture_output=True, text=True, env=environment,
                                       input=json.dumps({"tool_name": "Bash", "cwd": work, "tool_input": {"command": stage_command}}))
        stage_lines = stage_refused.stderr.splitlines()
        stage_copied = stage_lines[1] if len(stage_lines) > 1 and stage_lines[0].startswith("→ 照抄") else ""
        stage_copied_judged = subprocess.run(["bash", script], capture_output=True, text=True, env=environment,
                                             input=json.dumps({"tool_name": "Bash", "cwd": work, "tool_input": {"command": stage_copied}}))
        stage_declared = ("SINGLEFS_RUN_SCOPE='种类=whole; 跑什么=门禁 lkmm.sh 的全部格（不带 --check / --item）; 范围=整轮; "
                          "为什么=<为什么这么跑、这个范围为什么合适>' task-server/task submit -- " + stage_command)
        results.append(("stdin:主 agent 直接跑整道门禁被拒时照抄的那条经 task-server 提交、四段范围声明写在 submit 前面（为什么是占位），照它再判放行",
                        (2, stage_declared, 0), (stage_refused.returncode, stage_copied, stage_copied_judged.returncode)))
        # 文件头「经 task-server 起」④ 走真实入口：主 agent 提交要临时服务的自证裸写 --selftest，退出码 2、stderr 给包装写法的整条
        # （PROJECT_CODE_GUARD_BREAK=temporary-service-bare-allowed、temporary-service-submit-inner-unread 下红，LIB_PROJECT_CODE_RUNS_BREAK=temporary-service-command-bare 下红）
        bare_submitted_entry = subprocess.run(["bash", script], capture_output=True, text=True, env=environment,
                                              input=json.dumps({"tool_name": "Bash", "cwd": work,
                                                                "tool_input": {"command": f"task-server/task submit -- python3 {repo}/scripts/mutation/crates-mutation-rows.py --selftest"}}))
        results.append(("stdin:主 agent 经 task-server 提交要临时服务的自证裸写 --selftest 拒绝（退出码 2），stderr 给包装写法的整条", (2, 1),
                        (bare_submitted_entry.returncode,
                         int("--with-temporary-service -- python3 scripts/mutation/crates-mutation-rows.py --selftest" in bare_submitted_entry.stderr))))
        unknown_breaks = [item for item in os.environ.get(GUARD_BREAK_VARIABLE, "").split(",") if item and item not in KNOWN_GUARD_BREAK_ITEMS]
        results.append((f"{GUARD_BREAK_VARIABLE} 里的项都认得（认得的：{'、'.join(KNOWN_GUARD_BREAK_ITEMS)}）", [], unknown_breaks))
        # 走真实入口：实现员起含被拒命令的脚本，退出码 2、stderr 写出脚本路径与行号
        in_script = subprocess.run(["bash", script], capture_output=True, text=True, env=environment,
                                   input=json.dumps({"tool_name": "Bash", "cwd": work, "agent_type": writer, "tool_input": {"command": "bash s.sh"}}))
        results.append(("stdin:实现员起含被拒命令的脚本拒绝（退出码 2）", 2, in_script.returncode))
        results.append(("stdin:脚本里的拒绝 stderr 写出脚本路径与行号", 1, int(f"{work}/s.sh:4" in in_script.stderr)))
        # 走真实入口：实现员直接跑测试目标，退出码 2、stderr 写出经 task-server 提交的那条与写进报告交回的出路、不教子 agent 后台等
        # （PROJECT_CODE_GUARD_BREAK=task-server-direct-allowed 下两格红，task-server-copy-dropped 下第二格红）
        direct_run = subprocess.run(["bash", script], capture_output=True, text=True, env=environment,
                                    input=json.dumps({"tool_name": "Bash", "cwd": work, "agent_type": writer,
                                                      "tool_input": {"command": "cargo test -p singlefs-core --lib"}}))
        results.append(("stdin:实现员直接跑测试目标拒绝（退出码 2）", 2, direct_run.returncode))
        results.append(("stdin:子 agent 不经 task-server 的拒绝 stderr 写了照抄的提交、写进报告「要主 agent 跑的命令」一节交回，不教后台等", 1,
                        int("✗ 不经 task-server 跑项目代码被拒" in direct_run.stderr and "task-server/task submit -- cargo test -p singlefs-core --lib" in direct_run.stderr
                            and "写进报告「要主 agent 跑的命令」一节交回" in direct_run.stderr and "task-server/task wait <号>" not in direct_run.stderr)))
        recorded = count_detections()
        # 走真实入口：脚本文件不存在时照常放行（退出码 0），stderr 点名那个文件，并记一条检出
        missing = subprocess.run(["bash", script], capture_output=True, text=True, env=environment,
                                 input=json.dumps({"tool_name": "Bash", "cwd": work, "agent_type": writer, "tool_input": {"command": "bash missing.sh"}}))
        results.append(("stdin:脚本文件不存在照常放行（退出码 0）", 0, missing.returncode))
        results.append(("stdin:脚本文件不存在时 stderr 点名它、记一条检出", 1,
                        int(f"{work}/missing.sh" in missing.stderr and count_detections() == recorded + 1)))
        results.append(("stdin:放行时的检出记录写了结局「只记」", "只记", last_detection().get("outcome")))
        recorded = count_detections()
        # 走真实入口：同一条命令里 heredoc 写出不含要拒的命令的脚本再起它，照常放行（退出码 0），stderr 不报、不记检出
        written_light = subprocess.run(["bash", script], capture_output=True, text=True, env=environment,
                                       input=json.dumps({"tool_name": "Bash", "cwd": work, "agent_type": writer, "tool_input": {
                                           "command": "cat > gen-light.sh <<'EOF'\necho 轻\nEOF\nbash gen-light.sh"}}))
        results.append(("stdin:heredoc 写出的轻脚本照常放行（退出码 0）", 0, written_light.returncode))
        results.append(("stdin:heredoc 写出的轻脚本 stderr 不报「不存在」、不记检出", 1,
                        int("不存在" not in written_light.stderr and count_detections() == recorded)))
        written_heavy = subprocess.run(["bash", script], capture_output=True, text=True, env=environment,
                                       input=json.dumps({"tool_name": "Bash", "cwd": work, "agent_type": writer, "tool_input": {
                                           "command": "cat > gen.sh <<'EOF'\ncargo test --all\nEOF\nbash gen.sh"}}))
        results.append(("stdin:heredoc 写出的含 --all 的脚本拒绝（退出码 2）", 2, written_heavy.returncode))
        results.append(("stdin:heredoc 写出的脚本的拒绝 stderr 写出它的路径与行号", 1, int(f"{work}/gen.sh:1" in written_heavy.stderr)))
        recorded = count_detections()
        # 走真实入口：hook 旁边没有共用模块时照常放行（退出码 0），stderr 点名两份模块，并记一条检出
        lonely_hook_directory = os.path.join(work, "hook-without-shared-modules")
        os.makedirs(lonely_hook_directory)
        shutil.copy(script, lonely_hook_directory)
        without_library = subprocess.run(["bash", os.path.join(lonely_hook_directory, "project-code-guard.sh")], capture_output=True, text=True, env=environment,
                                         input=json.dumps({"tool_name": "Bash", "cwd": work, "agent_type": writer, "tool_input": {"command": "cargo test --all"}}))
        results.append(("stdin:读不到共用模块照常放行（退出码 0）", 0, without_library.returncode))
        results.append(("stdin:读不到共用模块时 stderr 点名两份模块、记一条检出", 1,
                        int("lib_project_code_runs.py" in without_library.stderr and "lib_shell_words.py" in without_library.stderr
                            and count_detections() == recorded + 1)))
        # 走真实入口：admission.py 导入得了、判的那一刻抛异常（判法里调的名字没跟上）——判不出，点名登记目标的 cargo test 照拒，不落到入口的「没判成、照常执行」
        # （lib_project_code_runs.py 的 judging-error-propagates 打开时这一格红）
        raising_repository = os.path.join(work, "repository-with-raising-admission")
        os.makedirs(os.path.join(raising_repository, ".claude", "hooks"))
        os.makedirs(os.path.join(raising_repository, "scripts"))
        os.makedirs(os.path.join(raising_repository, "scripts", "gate"), exist_ok=True)
        for name in ("project-code-guard.sh", "lib_project_code_runs.py", "lib_shell_words.py", "lib_workspace_cargo_lock.py"):
            shutil.copy(os.path.join(hook_dir, name), os.path.join(raising_repository, ".claude", "hooks", name))
        for name in ("scripts", "singlefs-ai-sop"):
            os.symlink(os.path.join(os.path.dirname(hook_dir), name), os.path.join(raising_repository, ".claude", name))
        with open(os.path.join(raising_repository, "scripts", "gate", "admission.py"), "w", encoding="utf-8") as handle:
            handle.write("def test_function_is_marked_ignored(*arguments):\n    return definitions_marked_ignored(*arguments)\n")
        raising = subprocess.run(["bash", os.path.join(raising_repository, ".claude", "hooks", "project-code-guard.sh")], capture_output=True, text=True,
                                 env=environment, input=json.dumps({"tool_name": "Bash", "cwd": work, "agent_type": writer, "tool_input": {
                                     "command": "bash scripts/hosts/run-with-memory-cap.sh 4G cargo test -p singlefs-verification --test crash_enumeration_new_pool_file_creation_stream"}}))
        results.append(("stdin:admission.py 判的那一刻抛异常时，点名标了 #[ignore] 的登记目标、不带 --ignored 的 cargo test 照拒（退出码 2）", 2, raising.returncode))
        results.append(("stdin:admission.py 判的那一刻抛异常时 stderr 不是「没判成、照常执行」", 1, int("没判成" not in raising.stderr)))
        # 走真实入口：主工作区里直接起、编译目录会落进主仓的 cargo（现场是临时 git 仓当主工作区）。WORKSPACE_CARGO_LOCK_BREAK 打开时这几格红（项与格见文件头）
        if cargo_lock is None:
            results.append(("编译目录：读得到 lib_workspace_cargo_lock.py", 0, 1))
        else:
            target_work = os.path.join(work, "cargo-target")
            os.makedirs(target_work)
            target_main = cargo_lock.build_main_workspace(target_work)
            # 编译目录（文件头「再一道」编译目录那一段）：CLAUDE_PROJECT_DIR 指到自检现场的主工作区，在它里面起；会话环境里的 CARGO_TARGET_DIR 去掉
            # 判在「谁能跑哪一类」那一层（decide）：在进程里照这一格的环境判它（经真入口直接起的 cargo 在 task_server_decision 那一层另拒，文件头「经 task-server 起」）
            def hook_in_target_workspace(command, agent_type=None):
                hook_input = {"tool_name": "Bash", "cwd": target_main, "tool_input": {"command": command}}
                if agent_type:
                    hook_input["agent_type"] = agent_type
                return category_layer_run(hook_input, target_main, {"CARGO_HOME": os.path.join(target_work, "cargo-home")},
                                          ("CARGO_TARGET_DIR", "CARGO_BUILD_TARGET_DIR"))
            bare_main = hook_in_target_workspace("bash scripts/hosts/run-with-memory-cap.sh 4G cargo build")
            results.append(("编译目录：主 agent 在主工作区裸起 cargo build：拒（退出码 2）", 2, bare_main.returncode))
            capped_writer = hook_in_target_workspace("bash scripts/hosts/run-with-memory-cap.sh 4G cargo test -p singlefs-verification --lib", writer)
            results.append(("编译目录：子 agent 经内存包装在主工作区起 cargo test：拒（退出码 2）", 2,
                            capped_writer.returncode if "编进主仓" in capped_writer.stderr else 0))
            results.append(("编译目录：拒绝说明给出经 task-server 提交的出路", 1,
                            int("编进主仓" in bare_main.stderr and "→ 怎么办" in bare_main.stderr
                                and "task-server/task submit -- " in bare_main.stderr and "task-server/task clean <号>" in bare_main.stderr)))
            results.append(("编译目录：经 run-build-directory.sh 包一层起：放行（退出码 0）", 0,
                            hook_in_target_workspace("bash scripts/hosts/run-with-memory-cap.sh 4G bash scripts/hosts/run-build-directory.sh cargo build").returncode))
            # 派发起法里面那条照判（LIB_PROJECT_CODE_RUNS_BREAK=dispatch-launchers-unseen 下第一格红）
            dispatched_bare = hook_in_target_workspace("bash scripts/hosts/multi-host-run.sh --peer --label x -- cargo test -p singlefs-verification --lib", "tooling-writer")
            results.append(("编译目录：子 agent 经 multi-host-run.sh --peer 在主工作区起 cargo test、没给编译目录：拒（退出码 2）", 2,
                            dispatched_bare.returncode if "编进主仓" in dispatched_bare.stderr else 0))
            results.append(("编译目录：子 agent 经 multi-host-run.sh --peer 与 run-build-directory.sh 起：放行（退出码 0）", 0,
                            hook_in_target_workspace("bash scripts/hosts/multi-host-run.sh --peer --label x -- bash scripts/hosts/run-build-directory.sh cargo test -p singlefs-verification --lib",
                                                     "tooling-writer").returncode))
            results.append(("编译目录：带仓外 CARGO_TARGET_DIR：放行（退出码 0）", 0,
                            hook_in_target_workspace(f"CARGO_TARGET_DIR={os.path.join(target_work, 'outside-target')} bash scripts/hosts/run-with-memory-cap.sh 4G cargo build").returncode))
            results.append(("编译目录：cargo fmt：放行（退出码 0）", 0, hook_in_target_workspace("cargo fmt --all").returncode))
            # 只定义、调不到的 function 与判项目代码那一处同一个口径（WORKSPACE_CARGO_LOCK_BREAK=uncalled-functions-judged 下第一格红，
            # LIB_SHELL_WORDS_BREAK=function-calls-literal-only 下第二格红）
            results.append(("编译目录：只定义、调不到的 function 函数体：放行（退出码 0）", 0, hook_in_target_workspace("function noop { cargo build; }").returncode))
            results.append(("编译目录：function 定义之后拆开拼写调用：拒（退出码 2）", 2, hook_in_target_workspace('function noop { bash scripts/hosts/run-with-memory-cap.sh 4G cargo build; }; "no"op').returncode))
            results.append(("lib_workspace_cargo_lock.py 的自检", 0, cargo_lock.selftest()))
    finally:
        project_code_runs.SAMPLE_INDEX_REPOSITORY = real_sample_index
        shutil.rmtree(work)
    failures = [item for item in results if item[1] != item[2]]
    for label, want, got in failures:
        print(f"  ✗ 自检：{label} 应当是 {want}，实际 {got}")  # gate-lint:detail
    if failures:
        print("    → 看 scan_uses()、scan_decision()、task_server_decision()、script_path_of()、executed_file_is_shell_script()、file_on_disk()、script_text()、command_lines() "
              "与共用的 lib_project_code_runs.py、lib_shell_words.py；"
              "PROJECT_CODE_GUARD_BREAK 设着的话（项与红的格见文件头）这里本来就该红")
        return 1
    refused_count = sum(1 for item in results if item[1] == 2)
    print(f"  ✓ 自检通过（查了 {len(results)} 种，其中该拒 {refused_count} 种）：外部程序（qemu-system-*、herd7、e152-file-system-benchmark，只问版本的 herd7 放行）与认不出起什么的写法拒绝，"
          "不经 task-server 跑项目代码（会编的 cargo、编出来的二进制、本仓的脚本）主 agent 与子 agent 一样拒、只经 task-server 提交；"
          "子 agent 起 task-server/task 的 submit、wait、cancel、clean 拒（带 --item 的、命名检查、自证、样本档、弄坏开关都算，读进来的脚本正文里的照判），status、log 放行，主 agent 交同一条放行；"
          "写进脚本再执行的（bash / ./ / capped.sh / source / 嵌套）读进去照判、拒绝写出路径与行号，脚本读不到或看不全的放行并记检出；"
          "同一条命令里 heredoc（cat / tee）写出、cp 拷出、cat >> / &> 与 tee 拷出再执行的拿写出的内容判、不记「不存在」，认不出的写法写出再起的照认不出拒；"
          "直接执行的只读 `#!` 指到 shell 的与没 `#!` 的 .sh（.rs、.md、没扩展名的文本不读不记），同一份脚本一次判定里只读一遍、只判一遍；"
          "命令词带通配的照 bash 展开、按展开出来的那几份判，带 POSIX 字符类、等价类、整理符号的交给 bash 自己展开、判不出的命令词照拒；"
          "命令词与参数里的花括号照 bash 展开再判；"
          "认不出起什么的写法（命令词、脚本路径、cargo 参数由变量、命令替换、位置参数拼出，名单外启动器读进来的词，喂给 shell 的不是字面）照拒，脚本里自己的转发函数按参数判；"
          "直接跑被拒时最前面给经 task-server 提交的那条命令（要声明范围的验证命令另带四段范围声明）；检出记录写结局（被拒 / 只记）")
    return 0

# 弄坏开关只在自证里认：命令行不是单独一个 --selftest、也不是自证起的子进程（PROJECT_CODE_GUARD_SELFTEST_MARK 对得上）而下面这些变量有一个非空
# （本钩子的，加上它导入的 lib_project_code_runs.py、lib_shell_words.py、lib_workspace_cargo_lock.py 的：它们的开关一样能放宽这道闸），读 hook 的 JSON 之前就退 2、点名变量只在自证里认，不管值是什么。
SELFTEST_MARK_VARIABLE = "PROJECT_CODE_GUARD_SELFTEST_MARK"
SELFTEST_MARK_CONTENT = "project-code-guard.sh --selftest 起的子进程"
SELFTEST_ONLY_VARIABLES = (GUARD_BREAK_VARIABLE, "LIB_PROJECT_CODE_RUNS_BREAK", "LIB_SHELL_WORDS_BREAK", "WORKSPACE_CARGO_LOCK_BREAK")

def refuse_break_outside_selftest(arguments):
    set_names = [name for name in SELFTEST_ONLY_VARIABLES if os.environ.get(name)]
    if not set_names or arguments == ["--selftest"] or os.environ.get(SELFTEST_MARK_VARIABLE) == SELFTEST_MARK_CONTENT:
        return None
    for name in set_names:
        print(f"  ✗ project-code-guard.sh：{name}={os.environ[name]} 是弄坏开关，只在自证（project-code-guard.sh --selftest）里认，平常跑设了就拒，这一条命令没判", file=sys.stderr)
    print(f"  → 怎么办：钩子的环境是 Claude Code 进程的环境，Bash 命令里 unset 改不到它：退出这个会话，在起 claude 的 shell 里 unset {'、'.join(set_names)} 再起；"
          f"要证红就跑 {set_names[0]}=<项> bash .claude/hooks/project-code-guard.sh --selftest", file=sys.stderr)
    return 2

def main():
    hook_dir = sys.argv[1]
    refused = refuse_break_outside_selftest(sys.argv[2:])
    if refused is not None:
        return refused
    if len(sys.argv) > 2 and sys.argv[2] == "--selftest":
        return selftest(hook_dir)
    raw_input = sys.stdin.buffer.read()
    detections_path = os.environ.get("AGENT_HOOK_DETECTIONS") or DEFAULT_DETECTIONS
    hook_input, unreadable = parsed_hook_input(raw_input)
    if unreadable:   # 不是 JSON 对象：照常执行，stderr 打一行可定位的诊断、记一条检出（会话号读不到，照实写读不到）
        print(f"  ! {unreadable}", file=sys.stderr)
        if not guard_break_is_set("failure-unrecorded"):
            record_detection(UNREADABLE_HOOK_INPUT, [unreadable], detections_path, "只记")
        return 0
    if project_code_runs is None:
        warning = f"project-code-guard.sh {shared_library_problem(hook_dir)}，这条命令没判、照常执行"
        print(f"  ! {warning}", file=sys.stderr)
        record_detection(hook_input, [warning], detections_path, "只记")
        return 0
    project_root = os.environ.get("CLAUDE_PROJECT_DIR") or os.path.dirname(os.path.dirname(hook_dir))
    try:
        decision = task_server_decision(hook_input, project_root)
    except Exception as error:   # 判的时候抛了：照常执行，stderr 报一句、记一条检出（与读不到共用模块那一支同样记「只记」）
        warning = f"project-code-guard.sh 没判成（{error!r}），这条命令照常执行"
        print(f"  ! {warning}", file=sys.stderr)
        if not guard_break_is_set("failure-unrecorded"):
            record_detection(hook_input, [warning], detections_path, "只记")
        return 0
    findings = [f"项目代码闸没判全：{notice}" for notice in decision.notices]
    for notice in decision.notices:
        print(f"  ! project-code-guard.sh：{notice}；{'' if decision.code else '这条命令照常执行，'}记了一条检出", file=sys.stderr)
    if decision.code:
        print(decision.message, file=sys.stderr)
        message_lines = decision.message.splitlines()
        findings.append(next((line for line in message_lines if line.startswith("✗ ")), message_lines[0]).removeprefix("✗ "))
    if findings:
        record_detection(hook_input, findings, detections_path, "被拒" if decision.code else "只记")
    return decision.code

sys.exit(main())
PY
hook_status=$?
if [[ $# -eq 0 && "$hook_status" != 0 && "$hook_status" != 2 && ",${PROJECT_CODE_GUARD_BREAK:-}," != *,bash-layer-unrecorded,* ]]; then
  hook_record_bash_layer_failure "project-code-guard.sh" "$hook_status" "$hook_input"
  exit 0
fi
exit "$hook_status"
