"""Bash 命令文本的切词、切简单命令、认命令位置、剥前缀与包装、跟 cd。

.claude/hooks/ 下判 Bash 命令的 hook（bash-command-detector.sh、project-code-guard.sh）共用这一份：
各自按文件路径用 importlib 导入，只留自己的判定。这里 def 的函数名在 .claude/hooks/ 别的文件里再 def 一份，
门禁 code-tooling 的 agent-write-scope 格判红——一份改了另一份不跟，同一条命令一个 hook 认得出、另一个认不出。

认得出的：引号与反斜杠（ANSI-C 引号 `$'…'` 里的转义照 bash 换掉、`\\'` 不收尾：ansi_c_quote_end、ansi_c_decoded）、词首的 # 注释、重定向（连同目标去掉，输出重定向的目标另交）、
命令替换 `$(…)` 与反引号（引号里外都认：整段留在所在的词里，里面的命令另按一条完整的命令递归判；`$((…))` 算术整段留在词里、不判）、
进程替换 `<(…)` 与 `>(…)`（只在引号外认，办法同命令替换：整段留在所在的词里，`)` 之后的词照旧是外层命令的参数）、
命令前的关键字与赋值（`名字+=值`、`名字[下标]=值` 同样是赋值；time 关键字后面的一个 -p 与一个 --、function 关键字连同函数名、builtin 连同它后面的 -- 一并跳过；builtin 后面不是 bash 内建的（`builtin cargo …`，bash 报错不跑）这一条没有命令词；只定义、调不到的 `function 名字 { …; }` 与 `名字() { …; }` 的函数体，调用方给了 uncalled_functions_skipped 时不判：drop_uncalled_function_bodies；
拆开拼写的调用（`"fo"o`、`fo{o,}`）解开之后就是那个名字，照调到了算，认不准的照判函数体：function_never_called）、nohup / setsid / nice / ionice / timeout / env / stdbuf / sudo / taskset / command / exec 这类前缀
（env 的选项照 getopt 切：短选项可以合写（`-iC 目录`、`-iCdir`、`-0iu NAME`），`-u` / `-C` / `-S` 的值贴着写或另起一个词，长选项认唯一前缀的缩写（`--chd 目录`），单独一个 `-` 等于 `-i`；
`-S 字符串` / `--split-string=字符串` 的字符串按 GNU env 的规则拆成词、原地换进命令里再判，拆出来的词里还可以有选项、赋值与命令词：env_split_string、skip_prefixes_expanding；
env 后面带 `=` 的词照 GNU env 都当赋值，名字不合 shell 变量名的（`env -- a-b=1 命令`）、开头就是 `=` 的（`env =1 命令`）也不是命令词；
认到头一个赋值之后不再认选项，其后头一个不带 `=` 的词就是命令词（`env =1 -u X 命令` 的 -u，env 报错不跑））、
`bash [选项] 脚本` 与 `bash -c '…'`（递归进去；`--rcfile` / `--init-file` 带一个值，`--` 与单独的 `-` 之后的那个词是脚本；-c 之后照 bash 接着认选项，选项完了的那个词是代码（`bash -c -- '…'`）；带 -s 的从标准输入读，后面的词是位置参数、不是脚本；
短选项之后的长选项（`bash -c --posix '…'`）bash 报错、带 -n 只查语法，都什么都不跑：ShellInvocation.runs_nothing）、eval 的参数（用空格连起来当一条完整的命令再切，同 bash -c）、`python3 [选项] 脚本`、`source` / `.`（后面跟的像一个路径才算）、`capped.sh N …`（N 写 `default` 的一样剥）与 `run-with-memory-cap.sh <上限> …`，
（被执行的只是脚本位上那一个词：`bash -n 脚本` 只查语法、`python3 - 词…` 的程序从标准输入读，后面的词都是参数，都不当被执行的脚本交出；
弄坏开关 LIB_SHELL_WORDS_BREAK=arguments-as-executed 把这两种里的参数照旧当被执行的脚本，哪几份自证的哪几项红见「弄坏开关」一段），
python 的选项照解释器的语义切（python_invocation）：短选项可以合写（`-Bu`），`-W`、`-X` 与 `--check-hash-based-pycs` 带一个值（贴着写或另起一个词），
值不当脚本名；`-c <代码>` 的程序是那段代码、没有被执行的脚本，交出的是 python 这一条本身（代码由调用方从参数里取）；`-m <模块>` 交出
`<模块里的点换成斜杠>.py` 当被执行的脚本（launcher 是 "python"），`-m runpy <模块>` 取后面那个模块，调用方拿它与仓内脚本的位置比
（弄坏开关 LIB_SHELL_WORDS_BREAK=python-options-by-dash 退回「以 - 开头的词都当选项跳过、第一个不以 - 开头的词当脚本」，哪几份自证的哪几项红见「弄坏开关」一段），
数组赋值 `名字=(…)`（元素是值，不当命令）、`command -v` / `-V`（查名字，后面那个词不当命令），
以及 cd / pushd 换目录、export / unset 改变量对后面命令的影响。喂给非 shell 命令的 heredoc 正文每一层都先剥掉；
heredoc 挂在没收尾的命令替换或进程替换里的命令上（`x=$(bash <<EOF`、`diff <(bash <<EOF`）时，在那一层里找挂它的命令（open_substitution_tail，
引号与括号与切词共用 command_substitution_end 那一套：双引号里收了尾的 `$(…)` 不算开着），喂给 shell 的照留。
引号外的花括号照 bash 展开成几个词（`cp X{,.bak}`、`{a,b}`、`{1..3}`；展开出来的空串照 bash 去掉，引号写出的空串留着；整数序列照 bash 补零（`{01..03}`，两端宽度不一的补到长的一边 `{01..100..3}`，brace_sequence_width）；
`${…}` 与赋值词不展开，一个词展开超过 BRACE_EXPANSION_LIMIT 个的照原样留：brace_expand；重定向的目标展开成几个词的（bash 报 ambiguous redirect、什么都不写）
与 here-string、heredoc 的结束词照原样留一个词：shell_tokens 的 flush）；
路径解开时开头的 `//` 并成一个 `/`（resolve_path：Linux 上是同一个文件，免得与同一条命令里记下的路径对不上）。
另认的：命令替换（与进程替换）里命令位置上的 `case 词 in` 之后的模式（不加左括号的 `a)`）
与词首 `#` 起的注释里的 `)` 不算替换收尾（command_substitution_end）；`coproc` 是关键字（`coproc 命令`、`coproc 名字 { …; }`
后面那条照判，路过的前缀名里记 coproc）；env 的 `-i` / `--ignore-environment` / 单独的 `-` 清掉外面带进来的与它之前写的变量（PrefixSkip.environment_cleared，
命令前写的 `BASH_ENV=x env -i bash …` 里 bash 拿不到 BASH_ENV）；从标准输入读命令的 shell 读的是盘上文件的（`bash -s < 文件`、`cat 文件 | bash`），
交出那份文件当 shell 起的脚本（shell_standard_input_files，调用方读正文判）；把参数当命令跑的函数（`run() { "$@"; }`、run_logged）的名字交 launcher_functions，
调用方把调用的参数一段段当命令判；切好的词是 ShellWord，带着还留着的花括号在不在引号里（`'echo a{,}'` 的花括号是数据，function_never_called 不把它当拆开拼写的调用；
修补报告 debt-tw-shell-words-fix2 第 95 行那一条）；CommandAtPosition 另带 piped_from、input_redirections（认 tee 拷出）与 wrappers（剥掉的 capped.sh / run-with-memory-cap.sh，
调用方核是不是本仓那一份）。
cd / pushd / env -C 的目录里写成变量的，同一段命令文本里前面赋过的（单独成一条的 `名字=值`、export、local / declare 这类）与 `$HOME`、`~` 先展开再判
（expand_known_variables；记下的值交在 CommandAtPosition.variables，调用方拿它展开别的参数）；展开不了的（没在这段文本里赋值、for / select / read / unset 改写过、
`$(…)`、`${名字:-默认}` 这类写法）照旧认不出：cd 之后的目录是 None，env -C 照外面的当前目录。
剥掉的 heredoc 正文与输出重定向挂回它所在的那条命令（CommandAtPosition 的 standard_input_heredocs、output_redirections）：
要认「同一条命令里先由 heredoc 写出文件、再执行它」的 hook 从这两格读，不自己再切一遍。重定向前的 fd 号挂在运算符上（RedirectOperator.fd，运算符照旧与 `>` 这些逐字相等；只认整个是引号外十进制数字的词，`01>` 是 1 号，`"2">`、`\\2>` 的 2 是参数）：
认这一条的标准输出写到哪的用 redirects_standard_output（`2> 文件` 不算）与 standard_output_destination（照 bash 从左到右跟 `n>&m`，`3>x >&3` 的标准输出进 x）。
case 语句的模式（`in` 与 `;;` / `;&` / `;;&` 之后到右括号为止的词）与 `[[ … ]]` 里的 `&&`、`||`、括号、`<`、`>` 不当命令位置、命令分隔与重定向（case_patterns_and_conditionals_folded）。`ln -s` 的相对源从链接所在的目录算（link_source_word，两个 hook 认拷出再执行时共用）。
剥掉包装之后命令词是一个脚本时，记下它是怎么起的（launcher：bash / sh 这类后面跟的、source / . 读进来的、python3 后面跟的）；
经 `run-with-memory-cap.sh <上限>` 剥出来的命令记 memory_capped，连同它 `bash -c` 里的命令；调用方往里读它起的脚本时，把这个标记当 memory_capped 参数传回来。
shell 启动时先读的文件当脚本跑（认不出的一律拒，偏严）：bash 起非交互 shell 读 `BASH_ENV`，交互（-i）的 bash 不读它、读 `--rcfile` / `--init-file`
指的文件（没给读 HOME 下的 .bashrc，`--norc` 不读），sh / dash / ksh 与 posix 模式的 bash（`--posix`、`-o posix`，环境里有 POSIXLY_CORRECT 或 SHELLOPTS 带 posix）读 `ENV`，zsh 读 ZDOTDIR
（没设取 HOME）下的 .zshenv、.zshrc（`-f`、`--no-rcs` 不读）；这一条命令看得到的变量（命令前、env 的参数、前面 export 的，还有 declare -x、typeset -x、local -x、先赋值后 export、
export 在先赋值在后、set -a 之后赋的；export -n、declare +x 撤掉的不算）在 shell 那一条之前多交一条 launcher 为 "startup" 的命令，词是那份文件的路径（`~` 与这段命令文本里赋过的变量先展开，
展开不了的原样交、带着 `$` 或反引号），由调用方同 `bash 脚本` 那样读正文判（startup_file_commands、startup_file_words；`bash -n`、`--version`、`--help` 什么都不读，不交）。
直接执行的脚本（命令词是脚本路径）起来时它的 #! 行指到的 shell 先读同样的文件，由调用方读到开头几个字节之后问 executed_script_startup（#! 指到 bash 或没有 #! 的文本文件读 BASH_ENV，
指到 sh 读 ENV，指到 zsh 读 zsh 的；指到 python 这类不是 shell 的不读；指到别的、而环境里设了 BASH_ENV 或 ENV 的，认不出，交解释器名由调用方照认不出办）。
实测（bash 5.2、dash）非交互的 sh 与 --posix 的 bash 其实不读 ENV、只有带 -i 才读，zsh 的 .zshrc 其实只有交互才读，这里照偏严算。
登录 shell（`-l`、`--login`，合写的短选项里有 l 也算；`exec -l`、`exec -a -名字` 起的 shell 照登录 shell 算）先读 SYSTEM_STARTUP_DIRECTORY（/etc）下的 profile，
bash 再读 HOME 下的 .bash_profile、.bash_login、.profile（bash 只读其中头一份在的，这里三份都当读，偏严），退出时读 .bash_logout 与 /etc/bash.bash_logout（`--noprofile` 不读这几份）；
sh、dash、ksh、ash 与 posix 模式的 bash 读 /etc/profile 与 HOME 下的 .profile；交互的 bash 在 .bashrc 之前先读 /etc/bash.bashrc（`--rcfile`、`--init-file` 只换掉 .bashrc、照读它，`--norc` 连它一起不读）；
zsh 先读 /etc/zsh/zshenv（-f 也读），再读 /etc/zsh/zshrc，登录的另读 /etc/zsh/zprofile、/etc/zsh/zlogin 与 ZDOTDIR 下的 .zprofile、.zlogin（本机没有 zsh，这几份没起真 shell 实测）。
实测（本机 bash 5.2、dash，strace 看 /etc 下打开的文件）：`bash -l -c` 读 /etc/profile、头一份在的 .bash_profile 再读 BASH_ENV，`exit` 时读 .bash_logout；`bash -i` 读 /etc/bash.bashrc 与 .bashrc，
`bash --rcfile 文件 -i` 读 /etc/bash.bashrc 与那份文件、不读 .bashrc，`--norc` 两份都不读；`sh -l`、`dash -l`、`busybox sh -l` 读 /etc/profile 与 .profile；`exec -l bash`、`exec -a -bash bash` 同 `bash -l`；`bash --posix -l -c` 一份都没读（这里照偏严读 /etc/profile 与 .profile）。
/etc/profile 里 `for i in /etc/profile.d/*.sh; do . $i; done` 这种循环变量起的 source，读正文的一方照脚本正文里解不出的路径办（project-code-guard.sh 不读这几份）。
别的程序里起的命令（embedded_launches）：python（-c 的代码、喂进标准输入的字面代码；照 ast 认 os.system、os.popen、subprocess、os.exec* / os.spawn*、pty.spawn、asyncio 的子进程）、
perl（-e / -E；system、exec、反引号、qx、readpipe、open 的管道）、ruby（-e；system、exec、spawn、反引号、%x、IO.popen、Open3）、node（-e / -p；child_process 的 exec / spawn 一族）、
php（-r；system、exec、shell_exec、passthru、popen、proc_open、反引号）、lua（-e；os.execute、io.popen）、tclsh / wish / expect（喂进标准输入的、expect -c；exec、spawn、system、open "|…"）、
awk 一族（程序文本、-e / --source；system()、print … | "…"、"…" | getline、|&）、sed（-e / 头一个参数；GNU 的 e 命令，单独的 e 与 s///e 执行模式空间认不出；--sandbox 什么都不起）、
make（--eval 与喂进标准输入的 Makefile 照正文判；Makefile 由调用方读正文交 program_text_launches：配方行、`名字 != 命令`、$(shell …)，变量照文件、命令行与 make 的内建默认值展开）、
git（-c 与配置里读到的别名，`!` 起头的照命令；core.pager、core.editor、core.sshCommand、core.fsmonitor、diff.external、filter、merge 驱动、credential.helper 的 `!…` 这类带命令的配置项；
这一条设的 GIT_EXTERNAL_DIFF、GIT_PAGER、PAGER、GIT_EDITOR、EDITOR、VISUAL、GIT_SSH_COMMAND、GIT_ASKPASS 这类环境变量；配置经 `git config --get-regexp` 照 git 自己的读法取，
这一份库里只有这一处起子进程）、tar（--to-command、-I / --use-compress-program、--checkpoint-action=exec、-F / --info-script、--rsh-command）、ssh（-o ProxyCommand、LocalCommand、KnownHostsCommand）、
less（+…! 命令、这一条设的 LESSOPEN / LESSCLOSE）、vim 一族（-c、--cmd、+ 的 ex 命令以 ! 起头的照命令，带 system()、:terminal、python 这类的认不出）。
字面写着的命令交 EmbeddedLaunches.texts，调用方同 bash -c 那样再判；写法认不出的（变量、拼接、__import__ / getattr / exec 动态取、ctypes、--config-env、GIT_EXEC_PATH）交 unjudged，调用方照认不出办；
程序从文件读的代码（awk -f、sed -f、make 的 -f 与默认的 GNUmakefile / makefile / Makefile）交 program_files，调用方读正文交 program_text_launches。
不读的：python / perl / ruby / tclsh 起的脚本文件（同共用约束「写」一节，脚本文件里与 python subprocess 起的写法由子 agent 自己守）、git 的钩子目录（core.hooksPath、.git/hooks）、ssh 的配置文件、
Makefile 里 include 进来的文件（交 unjudged）、vim 的 -S / -u 读的文件。弄坏开关 LIB_SHELL_WORDS_BREAK=embedded-launches-unseen 下一条都不认。
从标准输入读命令的 shell（bash / sh 这类不带 -c、不带脚本，或带 -s：shell_reads_standard_input）收到的字面文本当这个 shell 的代码再切一遍，
同 bash -c 一样挂回（它那一条命令的包装、目录、变量、memory_capped 照算；shell_standard_input）：here-string `<<<` 的文字；管道前一段是 printf（格式串与参数照 printf 现算）、
echo（-n / -e / -E 照 bash 内建认）、不带文件操作数而读 heredoc 的 cat 时它们交出的文字。喂进去的不是字面的判不了，每处一句说明交在 ShellScan.unjudged_shell_inputs：
here-string 或 printf / echo 的参数里有 `$`、反引号（变量、命令替换；单引号里写的同样算，切词时分不出，字面的那一部分照切）、不带引号结束词的 heredoc 正文里有它们、
管道前一段是别的命令（cat 文件、花括号组、子 shell……）、标准输入从文件或别的描述符来（`< 文件`、`<&`、`<>`；`< /dev/null` 不算）。调用方按各自的「判不了」处置（放行并记检出）。
弄坏开关 LIB_SHELL_WORDS_BREAK=shell-standard-input-unread 下不再切、也不交说明，哪几份自证的哪几项红见「弄坏开关」一段。
脚本文件本身不在这里读，要读的 hook 自己读，再把正文交回 commands_at_command_position。
判不到的：变量里拼出来的命令、eval 的参数里外层先展开的变量与命令替换（展开成什么判不到，照原文切）、喂给 shell 的标准输入不是字面的（交判不了的说明，见上）、管道前一段是花括号组或子 shell 时组里 printf / echo 的文字（整组当判不了）、
进程替换的输出再交给 shell 执行（`bash <(…)`、`source <(…)`，括号里的命令照判，它打印出来的不判；project-code-guard.sh 照认不出拒）、Makefile 起的命令
（xargs 与名单外启动器由 lib_project_code_runs.py 的 command_under_extra_launcher 剥）；
喂给 shell 的 heredoc 正文里引号没配对时，其后的切词会错；heredoc 挂的命令从上一行续行过来时认不出它喂给谁（当数据剥掉），没收尾的替换开在上一行的同样认不出；圆括号子 shell 里的 cd 与 export 当成对后面的命令也生效；
花括号展开不分 `[[ ]]`、case 模式这类 bash 不展开的位置（照展，判得偏严）；只定义的函数认 `function 名字 { …; }` 与 `名字() { …; }` 两种写法，函数体里套花括号的、函数体之外有变量或命令替换的照判（偏严）；`builtin` 认的是 bash 5.2 的内建表（enable -f 载进来的不认，照不跑放行）；
命令替换与进程替换里的命令按替换所在那条命令开始时的目录与 export 判（替换里的 cd 不影响外面）。
变量展开判不到的：值从上一条 Bash、source 进来的文件、eval 来的（目录照旧认不出）；单引号里的 `$名字` 照样展开（引号在切词时已去掉，分不出）；
bash -c 的那段代码按外层记下的全部变量展开（没 export 的变量里层真跑时看不到，这里照样展开，判得偏严）；圆括号子 shell 里赋的值当成对后面的命令也生效。
认得出起的是什么、不起重活与 claude 的照放（「认不出的一律拒」的另一半）：sed 脚本词里没展开的 bash 展开（`$((…))`、`$[…]` 换成整数，
`$(…)`、反引号与变量换成只认在行号地址位上的占位）在原样切不下去时逐种组合再切（sed_expanded_variants）；a / i / c 的文本认 `a\\` 换行与行末续行（sed_script_parse）；
python 起命令的 argv 头一项字面、参数不是字面的，参数换成一个值算不出的词照一段命令交出，赋过字面值没改过的名字与字面串的 .split() / shlex.split() 照算出的值（python_command_value）；
awk 里赋过字面串、没改写过的变量与命令行的 `名字=值` 当命令照值交出（awk_command_variables）；cd 的目标里 $PWD、$OLDPWD、`cd -`、`$(git rev-parse --show-toplevel)`、`$(pwd)`
照算（changed_directory、git_toplevel）；每条命令记 cd_failure_skips（前面可能失败的 cd 失败了它不跑）与 interactive（跑它的 shell 可能交互），非交互 shell source 的文件在顶层的
`$-` 守卫处截断（non_interactive_return_offset）。
认得出、判错了的照 bash 改成放（「认不出的一律拒」的另一半）：
同一行里定义的别名这一行不展开（bash 读完一整行才执行，别名定义要到下一行才生效；行号照切好的分隔符里的换行数，去掉过只定义的函数体的那一层照旧展开）；
喂给解释器的 heredoc 结束词不带引号时，正文里的 `$名字`、`${名字}` 照这段命令文本前面赋过的字面值代入再判（heredoc_body_expanded；有一处没赋过、算不出或是别的展开照旧判不了）；
python 起命令的 argv 是字面列表接 `+ 列表` 或 `*列表` 的，接上的那一段算得出照接上、算不出照一个值算不出的词交出，命令词是 shell、解释器、启动器或脚本的照旧认不出
（python_list_tail、python_tail_command_word_judged）；python 代码里只调 compile、代码不是字面的，不当起命令（字面的代码照扫，exec / eval 照判）；
sed 的 s 命令标志区照 GNU sed 认（g p e i I m M、数字、空白，w 读到行尾，`;`、换行、`}`、`#` 收尾），有别的字 GNU sed 编译就报 unknown option to `s'、整个脚本一行都不跑，不当 e 标志；
`cd 目录 && ( … )`、`cd 目录 && time ( … )` 的子 shell 照 `&&` 守着算（guarded_subshell_boundary）；只定义、不调用的 `名字() { …; }` 同 `function 名字 { …; }` 一样不判函数体。
弄坏开关（LIB_SHELL_WORDS_BREAK，几个用逗号连；只给自证判红用）。这一份没有自己的 --selftest，由导入它的几份的自证判红：
lib_project_code_runs.py、project-code-guard.sh、bash-command-detector.sh 各自的 --selftest（目录变量那两个开关另有 lib_workspace_cargo_lock.py 的 --selftest）。
逐项写开关名、弄坏哪一处、打开后哪几份自证的哪几项红（项名是那份自证的用例标签，括号里是失败行在标签之后的开头；
失败行 lib_project_code_runs.py 以「✗ lib_project_code_runs 自检：」起头，project-code-guard.sh、bash-command-detector.sh 以「✗ 自检：」起头；
project-code-guard.sh 的自证还原样转出 lib_project_code_runs.py 自证的失败行）：
  LIB_SHELL_WORDS_BREAK=arguments-as-executed  `bash -n 脚本` 的目标与 `python3 - 词…` 后面的词照旧当被执行的脚本交出
      lib_project_code_runs.py --selftest：
        python3 - 后面的门禁文件名是参数（门禁目录里跑）（「应当是 None，实际 commit-check-stage」）
        bash -n 只查门禁的语法（「应当是 None，实际 commit-check-stage」）
      project-code-guard.sh --selftest（另转出 lib_project_code_runs.py 那两项）：
        工具实现员 bash -n 查门禁的语法（「应当是 0，实际 2」）
        工具实现员在门禁目录里 python3 - 读几份门禁（2026-09-28 误拦的那一条原样，目录换成自检的临时工作区）（「应当是 0，实际 2」）（history-kept: 自证用例标签的原文，改了就对不上自证里的标签）
        实现员 bash -n 只查语法（「应当是 0，实际 2」）
        lib_project_code_runs.py 的自检（「应当是 0，实际 1」）
      bash-command-detector.sh --selftest：
        看门狗:前台 bash -n 只查语法（「应当是 0，实际 1」）
  LIB_SHELL_WORDS_BREAK=python-options-by-dash  python_invocation() 退回「以 - 开头的词都当选项跳过、第一个不以 - 开头的词当脚本」：带值选项的值当成脚本名，-m、-c 不认
      lib_project_code_runs.py --selftest：
        python3 -W ignore 起广谱变异（「应当是 broad-mutation，实际 None」）
        python3 -X dev 起广谱变异（「应当是 broad-mutation，实际 None」）
        python3 -BW default（合写、值另起一个词）起广谱变异（「应当是 broad-mutation，实际 None」）
        python3 --check-hash-based-pycs never 起广谱变异（「应当是 broad-mutation，实际 None」）
        cd 进 scripts 再 python3 -m broad-mutation（「应当是 broad-mutation，实际 None」）
        python3 -m research.scripts.broad-mutation（命名空间包）（「应当是 broad-mutation，实际 None」）
        python3 -m runpy broad-mutation（「应当是 broad-mutation，实际 None」）
        PYTHONPATH 指到 scripts 再 python3 -m broad-mutation（「应当是 broad-mutation，实际 None」）
        python3 -c runpy.run_path 起广谱变异（「应当是 broad-mutation，实际 None」）
        python3 -c exec(open(驱动).read())（「应当是 broad-mutation，实际 None」）
        python3 -c runpy.run_module 按模块名起（「应当是 broad-mutation，实际 None」）
        PYTHONPATH 指到 scripts 再 python3 -m 起判法脚本跑必红世界（「应当是 verus-stage，实际 None」）
        export PYTHONPATH 再 python3 -m 起红例重放（「应当是 verus-stage，实际 None」）
        python3 -c runpy.run_path 起判法脚本跑必红世界（「应当是 verus-stage，实际 None」）
        经 PYTHONPATH 起判法脚本的 --selftest：不在登记表的位置上，照旧算提交时才跑的检查（「应当是 commit-check-selftest，实际 None」）
        python3 -c 里改 sys.argv 成 proof、代码之后写 tool-versions（「应当是 verus-stage，实际 None」）
        python3 -c 里改 sys.argv 成 --replay、代码之后不写词（「应当是 verus-stage，实际 None」）
        python3 -c 里 __import__ 按模块名起判法脚本验证明（「应当是 verus-stage，实际 None」）
        python3 -c 里 importlib.import_module 起红例重放（「应当是 verus-stage，实际 None」）
        python3 -c 里 spec_from_file_location 载入判法脚本再调 run_check、代码之后写 toolchain（「应当是 verus-stage，实际 None」）
        python3 -c 的 print 包着 __import__ 起判法脚本（「应当是 verus-stage，实际 None」）
        python3 -c 的 print 包着 __import__、代码只有这一句（「应当是 verus-stage，实际 None」）
        python3 -c runpy.run_path 起红例钉表、代码之后写 --check（「应当是 verus-stage，实际 None」）
      project-code-guard.sh --selftest（另转出 lib_project_code_runs.py 那 23 项）：
        lib_project_code_runs.py 的自检（「应当是 0，实际 1」）
      bash-command-detector.sh 的 --selftest 不红
  LIB_SHELL_WORDS_BREAK=array-appends-unseen  record_array_writes() 什么都不记：`名字+=值`、`名字[下标]=值` 照旧不改记下的值
      lib_project_code_runs.py --selftest：cargo 那张表里「变量里的阶段路径：S 先赋目录、再 += 接上 crates 变异那一道的文件名」一格（「应当是 crates-mutation-stage，实际 None」）
      project-code-guard.sh --selftest（另转出 lib_project_code_runs.py 那一项）：
        lib_project_code_runs.py 的自检（「应当是 0，实际 1」）
      bash-command-detector.sh、detached-run.sh 的 --selftest 不红
  LIB_SHELL_WORDS_BREAK=env-chdir-ignored  skip_prefixes() 不交 env 的 -C / --chdir 换的目录：commands_at_command_position 里这一条命令照外面的当前目录判
      lib_project_code_runs.py --selftest：cargo 那张表里「env 换目录」开头的十一格里十格红（「换的目录只管这一条」那格不红）：
        env 换目录：-C 换进探针再 cargo test -p 探针自己的包名（「应当是 verification-cargo，实际 None」）
        env 换目录：-C 换进 验证包目录裸跑、-C 贴着目录写、--chdir=目录、--chdir 目录（分两个词）、bash -c 里的 env -C（「应当是 verification-cargo，实际 full-cargo」）
        env 换目录：绝对路径、当前目录在别处、经内存包装（「应当是 verification-cargo，实际 None」）
        env 换目录：换进 验证包目录裸跑放行、换进不依赖 验证包的探针裸跑放行、两个 -C 取最后一个（「应当是 None，实际 full-cargo」）
        另有「变量目录裸跑：」那一批的「export D 之后经内存包装 env -C $D/验证包目录」一格（「应当是 verification-cargo，实际 None」）
      project-code-guard.sh --selftest（另转出 lib_project_code_runs.py 那十一项）：
        实现员经内存包装 env -C 换进探针再 cargo test -p 探针自己的包名、env -C 换进 验证包目录裸跑、env --chdir=目录换进 验证包目录、env --chdir 目录（分两个词）、
        主 agent env -C 换进链接 验证包的探针 cargo run（「应当是 2，实际 0」）
        实现员经内存包装 env -C 换进不依赖 验证包的探针裸跑 cargo test：放行（「应当是 0，实际 2」）
        实现员经内存包装 env -iC 合写换进 验证包目录 cargo test：照拒、实现员经内存包装 env -S 拆串里先换目录再 cargo test：照拒（「应当是 2，实际 0」）
        实现员 export D 之后经内存包装 env -C $D/验证包目录裸跑：照拒（「应当是 2，实际 0」）
        lib_project_code_runs.py 的自检（「应当是 0，实际 1」）
      bash-command-detector.sh 的 --selftest 不红
  LIB_SHELL_WORDS_BREAK=env-option-clusters-unparsed  skip_prefixes() 里 env 的选项退回只认整词 -u / --unset / -C / --chdir：合写的 -iC、-0iu、缩写的 --chd、单独的 - 与 -S / --split-string
      不认（选项簇当不带值的开关，它的值词当命令词；-S 的字符串整个当命令词）
      lib_project_code_runs.py --selftest：cargo 那张表里「env 选项」开头六格里五格红（「-iC 换进 验证包目录裸跑放行」不红）、「env 拆串」开头七格里六格红（放行的一格不红）：
        env 选项：-iC 合写、-iC 目录贴着写、--chd 缩写（「应当是 verification-cargo，实际 None 或 full-cargo」）
        env 选项：-0iu 合写、单独一个 -（「应当是 full-cargo，实际 None」）
        env 拆串：-S、-iS、--split-string=、串里带 -C 与赋值、串后面还有参数、经内存包装与 capped.sh（「应当是 verification-cargo，实际 None」）
      project-code-guard.sh --selftest（另转出 lib_project_code_runs.py 那十一项）：
        实现员经内存包装 env -iC 合写换进 验证包目录 cargo test：照拒、env -S 拆串里是 验证包的 cargo test：照拒、env -S 拆串里先换目录再 cargo test：照拒（「应当是 2，实际 0」）
        lib_project_code_runs.py 的自检（「应当是 0，实际 1」）
      bash-command-detector.sh 的 --selftest 不红（它的自证里没有 env 带选项簇或 -S 的用例，未实跑，由主 agent 复跑核）
  LIB_SHELL_WORDS_BREAK=env-split-string-unexpanded  skip_prefixes_expanding() 不拆 env 的 -S / --split-string（-S 与它的字符串整个跳过，里面的命令不判）
      lib_project_code_runs.py --selftest：cargo 那张表里「env 拆串」开头七格里六格红（放行的「串里点名 验证包放行」不红；「应当是 verification-cargo，实际 None」）
      project-code-guard.sh --selftest（另转出 lib_project_code_runs.py 那六项）：
        实现员经内存包装 env -S 拆串里是 验证包的 cargo test：照拒、env -S 拆串里先换目录再 cargo test：照拒（「应当是 2，实际 0」）
        lib_project_code_runs.py 的自检（「应当是 0，实际 1」）
      bash-command-detector.sh 的 --selftest 不红（同上，未实跑）
  LIB_SHELL_WORDS_BREAK=directory-variables-unexpanded  expand_known_variables() 遇到 `$` 一律交 None：cd / pushd / env -C 的目录写在变量里的照旧认不出
      lib_project_code_runs.py --selftest：
        变量目录裸跑：D=… 之后 cd $D/验证包目录、经内存包装、export D 之后经内存包装 env -C $D/验证包目录、cd $HOME/… 指到 验证包目录、
        嵌套变量 D=$B/…、外层赋的变量在 bash -c 里的 cd（「应当是 verification-cargo，实际 None」）
        变量目录裸跑：cd ${D} 工作区根、经内存包装全量、declare D=… 之后 cd $D 全量（「应当是 full-cargo，实际 None」）
        变量里的阶段路径：S=…; bash "$S" 起 crates 变异那一道整道、目录放变量里拼出 crates 变异那一道、S 先赋目录、再 += 接上 crates 变异那一道的文件名（「应当是 crates-mutation-stage，实际 None」）、
        export 之后直接执行形式化那一道（「应当是 verus-stage，实际 None」）
      project-code-guard.sh --selftest（另转出 lib_project_code_runs.py 上面那些项）：
        实现员 D=… 之后 cd $D/验证包目录、经内存包装裸跑 cargo test：照拒、实现员 cd ${D} 工作区根、经 timeout、nice、capped.sh、内存包装四层全量：照拒、
        实现员 export D 之后经内存包装 env -C $D/验证包目录裸跑：照拒、实现员 cd $HOME/… 指到 验证包目录、经内存包装裸跑：照拒、
        主 agent D=… 之后 cd $D 全量：照拒（「应当是 2，实际 0」）
        cd 进写在变量里的主工作区再起：拒（转出 lib_workspace_cargo_lock.py 那一格，「应当拒，实际 放行」）
        lib_project_code_runs.py 的自检、lib_workspace_cargo_lock.py 的自检（「应当是 0，实际 1」）
      lib_workspace_cargo_lock.py --selftest：
        cd 进写在变量里的主工作区再起：拒（「应当拒，实际 放行」）
      bash-command-detector.sh --selftest：
        通删工作根:D=… 之后 cd $D 进工作根再 rm -rf *、通删工作根:cd $HOME/… 进工作根再 rm -r 第一层部分通配（「应当是 1，实际 0」；后一格只在工作根在家目录下时写成 $HOME）
        覆盖产物:cd $HOME/… 算得出来、按那个目录解析（仓外，放行）（「应当是 0，实际 1」）
      timer-only-remind-guard.sh、scripts/agents/agent-watch.py、scripts/gate/selftest-process-guard.py 的 --selftest 不红
  LIB_SHELL_WORDS_BREAK=variable-rebinding-unseen  for / select / read / unset 改写过的变量照旧用之前记下的值（rebind_variables() 与 unset 那一处不记成算不出）
      lib_project_code_runs.py --selftest：
        变量目录裸跑：for 改写过的变量照旧认不出（放行）、read 读进来的变量照旧认不出（放行）、unset 之后照旧认不出（放行）（「应当是 None，实际 verification-cargo」）
      project-code-guard.sh --selftest（另转出 lib_project_code_runs.py 那三项）：
        实现员 for 改写过的变量照旧认不出、经内存包装裸跑：放行（「应当是 0，实际 2」）
        lib_project_code_runs.py 的自检（「应当是 0，实际 1」）
      bash-command-detector.sh、lib_workspace_cargo_lock.py、timer-only-remind-guard.sh、agent-watch.py、
      selftest-process-guard.py 的 --selftest 不红
  LIB_SHELL_WORDS_BREAK=shell-standard-input-unread  commands_at_command_position() 不把 here-string 与管道喂给 shell 的文本当命令再切、也不交判不了的说明
      lib_project_code_runs.py --selftest：
        标准输入：here-string 喂给 bash 跑全量、标准输入：printf 经管道喂给 bash 跑全量、标准输入：echo 经管道喂给 sh 跑全量（「应当是 full-cargo，实际 None」）
      project-code-guard.sh --selftest（另转出 lib_project_code_runs.py 那三项）：
        实现员 here-string 喂给 bash 跑 --workspace、实现员 printf 经管道喂给 bash 跑 --workspace、实现员 echo 经管道喂给 sh 跑 --workspace、
        主 agent here-string 喂给 bash 跑 --workspace（「应当是 2，实际 0」）
        实现员 here-string 喂给 bash 的是变量：认不出照拒、照记检出：记几条检出、实现员 cat 文件经管道喂给 bash：放行并记检出：记几条检出（「应当是 1，实际 0」）
        stdin:here-string 喂给 bash 的是变量认不出照拒（退出码 2）、stderr 点名判不了、记一条检出（「应当是 (0, True, 1, 's-fed')」）
        lib_project_code_runs.py 的自检（「应当是 0，实际 1」）
      bash-command-detector.sh --selftest：
        stdin:here-string 喂给 bash 的是变量放行（退出码 0）、记一条检出（「应当是 (0, 1, True)，实际 (0, 0, False)」）
        stdin:拒了的命令里有判不了的输入（grep 的输出经管道喂给 bash），照记一条、结局被拒（「应当是 (2, 1, '被拒')，实际 (2, 0, '只记')」）
        stdin:here-string 里前台起看门狗照拒（退出码 2）（「应当是 2，实际 0」）
      run-scope-guard.sh --selftest（这个开关不在它的开关轮里，设着跑时开关轮另报「另有这几格红」）：
        here-string 喂给 bash 的 cargo test 整工作区：没带声明拒、printf 经管道喂给 bash 的门禁：没带声明拒、echo 经管道喂给 sh 的 cargo test 整工作区：没带声明拒（「应当是 2，实际 0」）
        入口：here-string 喂给 bash 的是变量放行、记一条检出（「应当是 (0, 1, True, '只记')」）
  下面七项是边角：
  LIB_SHELL_WORDS_BREAK=redirect-fd-dropped  shell_tokens() 不把 fd 号挂到重定向运算符上：redirects_standard_output() 把 `2> 文件` 也当改标准输出
      project-code-guard.sh --selftest：主 agent cat > 拷出、另把 stderr 丢进 /dev/null 再起全量脚本照拒（「应当是 2，实际 0」）、它的「：记几条检出」（「应当是 0，实际 1」）、
        「：读进去几份脚本」（「应当是 1，实际 0」），cat > 拷出、起的是 2> 写的那一份：认不出写法写出再起，照拒（「应当是 0，实际 2」）、它的「：记几条检出」（「应当是 1，实际 0」）、「：读进去几份脚本」（「应当是 0，实际 1」）
  LIB_SHELL_WORDS_BREAK=link-source-from-current-directory  link_source_word() 原样交回：ln -s 的相对源照当前目录解
      project-code-guard.sh --selftest：主 agent ln -s 的相对源从链接所在的目录算、经它起全量脚本照拒（「应当是 2，实际 0」）、它的「：记几条检出」（「应当是 0，实际 1」）、
        「：读进去几份脚本」（「应当是 1，实际 0」），ln -s 的相对源在链接所在的目录里不在：认不出写法写出再起，照拒（「应当是 0，实际 2」）、它的「：记几条检出」（「应当是 1，实际 0」）、「：读进去几份脚本」（「应当是 0，实际 1」）
  LIB_SHELL_WORDS_BREAK=env-leading-equals-as-command  skip_prefixes() 里 env 后面开头就是 `=` 的词当命令词
      project-code-guard.sh --selftest：主 agent env =1 开头就是 = 的赋值起全量照拒（「应当是 2，实际 0」）
  LIB_SHELL_WORDS_BREAK=shell-c-options-unparsed  shell_invocation() 取紧跟 -c 的那个词当代码（`bash -c -- '…'` 的代码取成 --）
      project-code-guard.sh --selftest：主 agent bash -c -- 的代码里起全量照拒（「应当是 2，实际 0」）
  LIB_SHELL_WORDS_BREAK=builtin-non-builtin-run  skip_prefixes() 把 builtin 后面不是内建的词照旧当命令词
      project-code-guard.sh --selftest：builtin 后面不是内建：bash 报错不跑，放行（「应当是 0，实际 2」）
  LIB_SHELL_WORDS_BREAK=uncalled-functions-run  drop_uncalled_function_bodies() 原样交回：只定义、调不到的函数体照命令判
      project-code-guard.sh --selftest：function 只定义不调用：函数体不跑，放行、名字() { 函数体 } 只定义不调用：函数体不跑，放行（「应当是 0，实际 2」）
  LIB_SHELL_WORDS_BREAK=shell-s-positionals-as-script  shell_invocation() 把 -s 后面的词照旧当脚本
      project-code-guard.sh --selftest：bash -s -- 后面的词是位置参数、不是脚本：放行（「应当是 0，实际 2」）、它的「：读进去几份脚本」（「应当是 0，实际 1」）
  这一批修补的十项（W1、W2 那三项与这七项）下，两个 hook 之外的 lib_project_code_runs.py、lib_workspace_cargo_lock.py、run-scope-guard.sh、handback-guard.sh、timer-only-remind-guard.sh、
  shared-index-stale-reminder.sh、scripts/agents/agent-watch.py、scripts/gate/selftest-process-guard.py、scripts/hosts/memory_peaks_lib.py
  的 --selftest 都不红（逐项实跑过；brace-sequence-unpadded 下 lib_project_code_runs.py 红的那五格除外）；这一批新加的格在已有的 braces-unexpanded、env-loose-assignments-as-command、
  shell-option-values-as-script、function-keyword-as-command 下也红，逐格写在 project-code-guard.sh、bash-command-detector.sh 文件头「同一批回归修补……在已有开关下也红」那一段
  下面九项：lib_project_code_runs.py、lib_workspace_cargo_lock.py、run-scope-guard.sh、
  handback-guard.sh、timer-only-remind-guard.sh、shared-index-stale-reminder.sh 的 --selftest 在这九项下都不红（逐项实跑过）；
  scripts/agents/agent-watch.py、scripts/gate/selftest-process-guard.py、scripts/hosts/memory_peaks_lib.py 的 --selftest 没在这九项下跑过：
  LIB_SHELL_WORDS_BREAK=time-options-as-command  skip_prefixes() 不跳 time 关键字后面的 -p / --（-p 当命令词）
      project-code-guard.sh --selftest：主 agent time -p 起全量照拒、主 agent time -- 起全量照拒、实现员 time -p 包着内存包装起 验证包照拒（「应当是 2，实际 0」）
      bash-command-detector.sh --selftest：等待循环:前台 time -p 包着的 bash -c 里没有 timeout（「应当是 1，实际 0」）
  LIB_SHELL_WORDS_BREAK=builtin-as-command  skip_prefixes() 把 builtin 当命令词（后面的 source / cd 不认）
      project-code-guard.sh --selftest：主 agent builtin source 读进全量脚本照拒、实现员先进 验证包目录、再 builtin cd 进 验证包目录经内存包装裸跑照拒（「应当是 2，实际 0」），
        主 agent builtin source 读进全量脚本照拒：读进去几份脚本（「应当是 1，实际 0」）
  LIB_SHELL_WORDS_BREAK=shell-option-values-as-script  shell_invocation() 不认 --rcfile / --init-file 的值，`--` 与单独的 `-` 当脚本
      project-code-guard.sh --selftest：主 agent bash --rcfile 起全量脚本照拒、bash -- 起全量脚本照拒、bash - 起全量脚本照拒（「应当是 2，实际 0」），三格各自的「：读进去几份脚本」（「应当是 1，实际 0」）
  LIB_SHELL_WORDS_BREAK=function-keyword-as-command  skip_prefixes() 把 function 关键字当命令词
      project-code-guard.sh --selftest：主 agent function 关键字定义的函数里起全量照拒（「应当是 2，实际 0」）
  LIB_SHELL_WORDS_BREAK=braces-unexpanded  brace_expand() 原样交回（引号外的花括号不展开）
      project-code-guard.sh --selftest：主 agent cp X{,.bak} 拷出再起全量脚本照拒（「应当是 2，实际 0」）、它的「：记几条检出」（「应当是 0，实际 1」）、「：读进去几份脚本」（「应当是 1，实际 0」）
      lib_project_code_runs.py --selftest：「命令文本：命令词的花括号照 bash 展开（{x,mutate}.sh：bash 起的是 x.sh，mutate.sh 是它的参数）」
        「命令文本：花括号里的字母序列照 bash 展开（repla{x..y}.sh：bash 起的是头一份，算门禁阶段）」（花括号留到 lib_project_code_runs.py 的 brace_expansion 照「每一份单独当命令」判）
  LIB_SHELL_WORDS_BREAK=double-slash-kept  resolve_path() 留着开头的 //（os.path.normpath 的样子）
      bash-command-detector.sh --selftest：就地改仓内文件:cd 到 // 开头写的 /tmp/ 下的目录之后 sed -i 相对路径放行（「应当是 0，实际 1」）
      project-code-guard.sh --selftest 不红（它按实际路径比，开头的 // 本来就并掉了）
  LIB_SHELL_WORDS_BREAK=env-loose-assignments-as-command  skip_prefixes() 里 env 后面名字不合变量名的 `名字=值` 当命令词
      project-code-guard.sh --selftest：主 agent env -- 带不合变量名的赋值起全量照拒（「应当是 2，实际 0」）
  LIB_SHELL_WORDS_BREAK=eval-unread  commands_at_command_position() 不切 eval 的参数（eval 当一条普通命令）
      project-code-guard.sh --selftest：主 agent eval 字面代码里起全量照拒（「应当是 2，实际 0」）
  LIB_SHELL_WORDS_BREAK=heredoc-open-substitution-unseen  heredoc_feeds_shell() 照整行找挂它的命令（没收尾的 $( / <( 里的 bash 认不出，正文当数据剥掉）
      project-code-guard.sh --selftest：主 agent $(bash <<EOF) 里起全量照拒、主 agent <(bash <<EOF) 里起全量照拒（「应当是 2，实际 0」）
      bash-command-detector.sh --selftest：等待循环:前台 $(bash <<EOF) 里的循环、等待循环:前台 <(bash <<EOF) 里的循环（「应当是 1，实际 0」）
  下面三项：
  LIB_SHELL_WORDS_BREAK=brace-empties-kept  brace_expand() 留着花括号展开出来的空串（`{cargo,} test` 切成 `cargo '' test`）
      project-code-guard.sh --selftest：主 agent {cargo,} 空的一份起全量照拒、主 agent {,} 写在命令词前面起全量照拒、
        主 agent cargo {,test} 空的一份起全量照拒、实现员 {bash,} 空的一份经内存包装起 验证包照拒（「应当是 2，实际 0」）
  LIB_SHELL_WORDS_BREAK=brace-sequence-unpadded  brace_sequence_width() 一律交 0：补零的整数序列（{01..03}）不补零（lib_project_code_runs.py 的 brace_sequence 共用它，一并不补）
      project-code-guard.sh --selftest（另转出 lib_project_code_runs.py 那五项）：主 agent cp 补零的序列拷出再起全量脚本照拒（「应当是 2，实际 0」）、
        它的「：记几条检出」（「应当是 0，实际 1」）、它的「：读进去几份脚本」（「应当是 1，实际 0」）、lib_project_code_runs.py 的自检（「应当是 0，实际 1」）
      lib_project_code_runs.py --selftest：花括号展开与 bash 一致：{01..10}、{-01..3}、{1..-03}、{0..00}、{1..05..2}（「应当是 [」；{01..100..33}、{100..00..50} 同样红）
  LIB_SHELL_WORDS_BREAK=double-quoted-substitution-unclosed  open_substitution_tail() 把双引号里的 `$(` 一律当没收尾（顶层的 bash <<EOF 当挂在替换里，正文当数据剥掉）
      project-code-guard.sh --selftest：主 agent cd "$(git rev-parse --show-toplevel)" 之后 bash <<EOF 里起全量照拒、主 agent FOO="$(pwd)" bash <<EOF 里起全量照拒
        （「应当是 2，实际 0」）
      bash-command-detector.sh --selftest：等待循环:前台 cd "$(…)" 之后 bash <<EOF 里的循环、等待循环:前台 echo "$(date)" | bash <<EOF 里的循环（「应当是 1，实际 0」）
  下面十三项：
  LIB_SHELL_WORDS_BREAK=function-calls-literal-only  function_never_called() 只数原文里的字面：拆开拼写的调用（`"fo"o`、`fo{o,}`、`fo\\o`）认不出，函数体照不判
      project-code-guard.sh --selftest：主 agent function 定义之后引号拆开拼写调用照拒、主 agent function 定义之后花括号拆开拼写调用照拒、
        主 agent function 定义之后反斜杠拆开拼写调用照拒、编译目录：function 定义之后拆开拼写调用：拒（退出码 2）（「应当是 2，实际 0」），
        lib_workspace_cargo_lock.py 的自检（「应当是 0，实际 1」；另原样转出 lib_workspace_cargo_lock.py 下面那一格）
      lib_workspace_cargo_lock.py --selftest：编译目录：function 定义之后拆开拼写调用：拒（「命令」）
  LIB_SHELL_WORDS_BREAK=redirect-fd-number-literal  redirection_fd() 照字面比 fd 号：`01>` 不算 1 号
      project-code-guard.sh --selftest：主 agent cat 01> 拷出再起全量脚本照拒（「应当是 2，实际 0」）、它的「：记几条检出」（「应当是 0，实际 1」）、「：读进去几份脚本」（「应当是 1，实际 0」）
  LIB_SHELL_WORDS_BREAK=redirect-fd-quoted-digits  shell_tokens() 把引号或反斜杠写出的数字（`"2">`、`\\2>`）也当 fd 号
      project-code-guard.sh --selftest：主 agent cat 源 "2"> 拷出（引号里的 2 是源）再起全量脚本照拒（「应当是 2，实际 0」）、它的「：记几条检出」「：读进去几份脚本」
  LIB_SHELL_WORDS_BREAK=redirect-fd-duplication-unseen  standard_output_destination() 不跟 `n>&m`，只取直接改标准输出的最后一个
      project-code-guard.sh --selftest：主 agent cat 3> 再 >&3 拷出再起全量脚本照拒、主 agent heredoc 写文件：cat <<EOF 3> 再 >&3 写出再起全量脚本照拒（「应当是 2，实际 0」），
        两格各自的「：记几条检出」「：读进去几份脚本」
  LIB_SHELL_WORDS_BREAK=shell-long-options-after-short  shell_invocation() 把短选项之后的长选项照旧当选项跳过
      project-code-guard.sh --selftest：bash -c 之后的长选项：bash 报错不跑，放行（「应当是 0，实际 2」）
  LIB_SHELL_WORDS_BREAK=shell-syntax-only-code-run  shell_invocation() 带 -n 时 -c 的代码照判
      project-code-guard.sh --selftest：bash -c -n 只查语法：放行（「应当是 0，实际 2」）
  LIB_SHELL_WORDS_BREAK=shell-option-clusters-unvalued  shell_invocation() 里合写的短选项簇不吃 -o / -O 的值词（`bash -euo pipefail -c …` 的 pipefail 不吃掉：
      pipefail 被当成 -c 的代码或脚本，整条 -c 的代码没进判）
      lib_project_code_runs.py --selftest：「选项簇：bash -euo pipefail -c 全量」「选项簇：bash -eo pipefail -c 全量」「选项簇：bash -ouc pipefail 全量」「选项簇：bash -oo pipefail errexit -c 全量（两个 o 各吃一个词）」四格红（应当是 full-cargo，实际 None）
      project-code-guard.sh --selftest（另转出 lib_project_code_runs.py 那四格）：实现员 bash -euo pipefail -c 全量照拒、实现员 bash -euo pipefail 起全量脚本照拒（「应当是 2，实际 0」），lib_project_code_runs.py 的自检（「应当是 0，实际 1」）
  LIB_SHELL_WORDS_BREAK=env-options-after-assignment  skip_prefixes() 里 env 认到赋值之后照旧认选项
      project-code-guard.sh --selftest：env =1 之后的 -u 是命令词：env 报错不跑，放行（「应当是 0，实际 2」）、lib_project_code_runs.py 的自检（「应当是 0，实际 1」；另转出下面那一格）
      lib_project_code_runs.py --selftest：env 拆串：拆出来的串里赋值之后的 -C 是命令词，env 报错不跑（「应当是 None，实际 verification-cargo」）
  LIB_SHELL_WORDS_BREAK=brace-sequence-width-padded-ends-only  brace_sequence_width() 只取补零写的那几边的长度
      project-code-guard.sh --selftest：主 agent cp 带步长、两端宽度不一的补零序列拷出再起全量脚本照拒（「应当是 2，实际 0」）、它的「：记几条检出」「：读进去几份脚本」，
        lib_project_code_runs.py 的自检（「应当是 0，实际 1」；另转出下面两格）
      lib_project_code_runs.py --selftest：花括号展开与 bash 一致：{01..100..33}、{100..00..50}（「应当是 [」）
  LIB_SHELL_WORDS_BREAK=ansi-c-quotes-unseen  shell_tokens()、open_substitution_tail()、command_substitution_end() 把 `$'…'` 照普通单引号配对
      project-code-guard.sh --selftest：主 agent $'…' 里带 \\' 之后 bash <<EOF 里起全量照拒（「应当是 2，实际 0」）
      bash-command-detector.sh --selftest：等待循环:前台 $'a\\'b' 之后 $(bash <<EOF) 里的循环（「应当是 1，实际 0」）
  LIB_SHELL_WORDS_BREAK=case-patterns-as-commands  case_patterns_and_conditionals_folded() 不改 case 的模式
      project-code-guard.sh --selftest：case 里 ;; 之后的模式词不当命令：放行、case 里 | 隔开的模式词不当命令：放行（「应当是 0，实际 2」）
      bash-command-detector.sh --selftest：stdin:子 agent case 里 ;; 之后的 reboot 模式不当命令放行（退出码 0）、stdin:子 agent case 里 ;; 之后的 claude 模式不当命令放行（退出码 0）（「应当是 0，实际 2」）
  LIB_SHELL_WORDS_BREAK=conditional-expression-split  case_patterns_and_conditionals_folded() 不改 `[[ … ]]`
      project-code-guard.sh --selftest：[[ ]] 里 && 之后的词不当命令：放行（「应当是 0，实际 2」）
  LIB_SHELL_WORDS_BREAK=redirect-target-braces-expanded  shell_tokens() 照旧展开重定向目标上的花括号（展开出来多的词成了这一条的参数；随第 3 条一起改：
    几个源的 cat 认成拷贝之后，`cat s.sh > d{,}.sh` 被当成把 s.sh 接进 d.sh，而 bash 报 ambiguous redirect、什么都不写；展开成几个词的目标现在照原样留一个词：拷出再执行认不出这个落点，写范围那一类照落点算不出拒）
      project-code-guard.sh --selftest：重定向目标花括号展开成几个词：bash 报 ambiguous redirect 不写，放行（「应当是 0，实际 2」）、它的「：记几条检出」（「应当是 1，实际 0」）、
        「：读进去几份脚本」（「应当是 0，实际 1」）
  LIB_SHELL_WORDS_BREAK=extended-assignments-as-command  skip_prefixes() 把 `名字+=值`、`名字[下标]=值` 照旧当命令词（随第 6 条一起改：ANSI-C 引号解开之后，
    定义里的 \\t 把测试目标名隔成单独的词，这类赋值词被当成没展开的命令词误拒）
      project-code-guard.sh --selftest：数组元素赋值与 += 追加是赋值、不是命令词：放行（「应当是 0，实际 2」）
  下面六项：
  LIB_SHELL_WORDS_BREAK=substitution-case-and-comment-unseen  command_substitution_end() 不认命令替换里的 case 模式与注释
      project-code-guard.sh --selftest：一百三十七：命令替换里不加括号的 case 模式不提前收尾，里面的 --all 照拒、一百三十七：命令替换里注释的 ) 不收尾，后面的 --all 照拒（「应当是 2，实际 0」）
  LIB_SHELL_WORDS_BREAK=coproc-as-command  skip_prefixes() 把 coproc 当命令词
      project-code-guard.sh --selftest：一百三十七：coproc 后面那条照判（「应当是 2，实际 0」）
      bash-command-detector.sh --selftest：一百三十七：coproc 照拒（退出码 2）不红（③ 照旧按命令词认 coproc），coproc-prefix-unseen 与它同设才红
  LIB_SHELL_WORDS_BREAK=quoted-braces-as-unknown  brace_expand() 交的词不带引号信息（引号里的花括号也当认不准）
      project-code-guard.sh --selftest：一百三十七：函数体外引号里的花括号不当认不准的调用，只定义的函数体不判：放行（「应当是 0，实际 2」）
  LIB_SHELL_WORDS_BREAK=env-ignore-environment-kept  skip_prefixes() 不认 env -i 清掉环境
      project-code-guard.sh --selftest：一百三十七：env -i 清掉 BASH_ENV 再起 bash：放行（「应当是 0，实际 2」）与它的「：读进去几份脚本」（「应当是 0，实际 1」），推的
  LIB_SHELL_WORDS_BREAK=shell-standard-input-file-unread  shell_standard_input_files() 一律交 []（`bash -s < 文件` 只记判不了）
      project-code-guard.sh --selftest：一百三十七：bash -s < 文件 读那份文件判、一百三十七：cat 文件 | bash 读那份文件判（「应当是 2，实际 0」与「：读进去几份脚本」），
        一百三十七：sh -s < 轻脚本放行：读进去几份脚本（「应当是 1，实际 0」）
  LIB_SHELL_WORDS_BREAK=function-launchers-unseen  launcher_functions() 交空集
      project-code-guard.sh --selftest：一百三十七：脚本里经自己的 run_logged 起 --all 照拒（「应当是 2，实际 0」）
  下面五项（shell 启动时先读的文件当被起的脚本判；逐格的失败行原文写在 project-code-guard.sh 与 bash-command-detector.sh 文件头各开关名下）：
  LIB_SHELL_WORDS_BREAK=startup-files-unread  startup_file_words() 一个文件都不交：BASH_ENV / ENV / --rcfile / .bashrc / zsh 的启动文件都不当被起的脚本
      project-code-guard.sh --selftest：112 格红（凡是靠启动文件判拒、判读进去几份的格：一百三十七那一批 86 格、「d137 H6」「d137 A2」读 BASH_ENV 的 9 格、「d137 已知没认：」登录与系统级 17 格）；不读它的那几格（bash 不带 --posix 的 ENV、sh 的 BASH_ENV、bash -n、env -i、unset、文件不在、-f、--no-rcs 这类）不红
      bash-command-detector.sh --selftest：「起 claude:一百三十七：」开头、靠启动文件判拒的 14 格与「起 claude:d137 已知没认：」登录、系统级拒的 5 格，共 19 格（「应当是 1，实际 0」）
      lib_project_code_runs.py --selftest 不红
  LIB_SHELL_WORDS_BREAK=declared-exports-unseen  环境里的变量只认 `export 名字=值`（declare -x、typeset -x、local -x、先赋值后 export、先 export 后赋值、export -n、declare +x、set -a 不认）
      project-code-guard.sh --selftest：21 格红；bash-command-detector.sh --selftest：4 格红（含 export -n 撤掉之后起 bash 一格，「应当是 0，实际 1」）
  LIB_SHELL_WORDS_BREAK=interactive-startup-files-unread  shell_start_options() 的 -i 不另看：交互的 bash 照旧读 BASH_ENV、不读 .bashrc 与 --rcfile / --init-file
      project-code-guard.sh --selftest：17 格红（一百三十七那一批 13 格、「d137 已知没认：系统级」读 /etc/bash.bashrc 的 4 格）；bash-command-detector.sh --selftest：4 格红（含 bash -i 不读 BASH_ENV 一格，「应当是 0，实际 1」；
        「起 claude:d137 已知没认：系统级 bash -i 读 /etc/bash.bashrc（样本目录）里起 claude」一格）；sh -i 与 ENV 的格不红（sh 本来就读 ENV）
  LIB_SHELL_WORDS_BREAK=zsh-startup-files-unread  zsh 的 .zshenv、.zshrc 不读
      project-code-guard.sh --selftest：13 格红；bash-command-detector.sh --selftest：1 格红
  LIB_SHELL_WORDS_BREAK=executed-script-startup-unread  executed_script_startup() 什么都不交：直接执行的脚本起来时它的 shell 先读的启动文件不读，#! 指到认不出的解释器的不判
      project-code-guard.sh --selftest：16 格红；bash-command-detector.sh --selftest：2 格红
  这十八项下 run-scope-guard.sh、handback-guard.sh、timer-only-remind-guard.sh、shared-index-stale-reminder.sh、
  scripts/agents/agent-watch.py、scripts/gate/selftest-process-guard.py、scripts/hosts/memory_peaks_lib.py 的 --selftest 都不红，上面没点名的
  lib_project_code_runs.py、lib_workspace_cargo_lock.py 也不红（逐项实跑过）；
  这一批新加的格在已有的 arguments-as-executed、braces-unexpanded、brace-sequence-unpadded、env-leading-equals-as-command、env-loose-assignments-as-command、
  function-keyword-as-command、uncalled-functions-run、heredoc-open-substitution-unseen、redirect-fd-dropped 下也红，逐格写在 project-code-guard.sh、bash-command-detector.sh
  文件头「三方 shell-words-r2 判决……在已有开关下也红」那一段
  下面四项：project-code-guard.sh --selftest 的「d137 …」各格红：
  LIB_SHELL_WORDS_BREAK=trap-action-unread  commands_at_command_position() 不把 trap 'action' 的字符串当 shell 代码再切（trap 的参数不判）
      project-code-guard.sh --selftest：「d137 H2：trap 'cargo test --all' EXIT 照拒」「…ERR…」「…0…」「d137 H2：trap 'bash startup-heavy.sh' DEBUG 读正文照拒」（连同它的「读进去几份脚本」）
        「d137 H2：trap -- 'cargo test --all' EXIT（-- 之后是 action）照拒」（都「应当是 2，实际 0」）
  LIB_SHELL_WORDS_BREAK=standard-input-device-script-unread  起 shell 的脚本路径是标准输入设备（/dev/stdin、/dev/fd/0、/proc/self/fd/0）时不当标准输入、照旧当一份脚本文件读
      project-code-guard.sh --selftest：「d137 H4：bash /dev/stdin < 文件 读那份文件判，照拒」（连同它的「读进去几份脚本」）「d137 H4：echo 代码 | bash /dev/stdin 照拒」
        「d137 H4：bash /dev/fd/0 <<< 代码 照拒」「d137 H4：source /dev/stdin <<< 代码 照拒」（应当是 2，实际 0）、「d137 H4：bash /dev/stdin < 轻脚本 放行：读进去几份脚本」（应当是 1，实际 0）
  LIB_SHELL_WORDS_BREAK=tilde-plus-minus-unexpanded  expand_directory_tilde() 不展开 `~+`（$PWD）与 `~-`（$OLDPWD）（当普通路径、落不到实处）
      project-code-guard.sh --selftest：「d137 H6：bash ~+/脚本 照 $PWD 解、读正文照拒」（连同「记几条检出」「读进去几份脚本」）「d137 H6：BASH_ENV=~+/脚本 起 bash，照 $PWD 解、读正文照拒」（连同「读进去几份脚本」）
        「d137 H6：bash ~-/脚本 的 OLDPWD 算不出，认不出起哪一份，照拒」（连同「记几条检出」；~- 这两格在 PROJECT_CODE_GUARD_BREAK=relative-script-undeterminable-directory-allowed 下也红）
  LIB_SHELL_WORDS_BREAK=renamed-shells-unseen  is_shell_name() 只认 bash / sh / dash / zsh / ksh，rbash、ash 不当 shell，busybox 的 sh / ash applet 不剥
      project-code-guard.sh --selftest：「d137 H8：rbash -c 代码 照 shell 判、照拒」「d137 H8：rbash 起脚本 读正文照拒」（连同「读进去几份脚本」）「d137 H8：busybox sh -c 代码 照拒」「d137 H8：busybox ash -c 代码 照拒」（应当是 2，实际 0；dash、ksh 两格不红，它们在 SHELL_NAMES）
  下面三项（逐格的失败行原文写在 project-code-guard.sh 与 bash-command-detector.sh 文件头各开关名下）：
  LIB_SHELL_WORDS_BREAK=login-profiles-unread  startup_file_words() 不认登录 shell（-l、--login、exec -l、exec -a -名字）：/etc/profile、.bash_profile、.bash_login、.profile、.bash_logout 都不交
      project-code-guard.sh --selftest：13 格红（登录那五格照拒与读进去几份、轻 profile 那格的读进去几份、系统级 bash -l 那格与它的读进去几份）；bash-command-detector.sh --selftest：4 格红（登录三格、系统级 bash -l 一格）；
        逐格的失败行原文写在两份钩子文件头这一条下
  LIB_SHELL_WORDS_BREAK=system-startup-files-unread  startup_file_words() 不交 SYSTEM_STARTUP_DIRECTORY 下的系统级启动文件（/etc/profile、/etc/bash.bashrc、/etc/zsh/ 下的）
      project-code-guard.sh --selftest：6 格红（系统级照拒的三格与它们的读进去几份）；bash-command-detector.sh --selftest：2 格红；逐格的失败行原文写在两份钩子文件头这一条下
  LIB_SHELL_WORDS_BREAK=embedded-launches-unseen  embedded_launches() 一条都不认：别的程序里起的命令不交
      project-code-guard.sh --selftest：24 格红；bash-command-detector.sh --selftest：8 格红；lib_project_code_runs.py --selftest 不红；逐格的失败行原文写在两份钩子文件头这一条下
  LIB_SHELL_WORDS_BREAK=aliases-unexpanded  shell_aliases() 交空：shopt -s expand_aliases 之后定义的别名不展开成它的值（命令词是别名时照普通命令判）
      project-code-guard.sh --selftest：「d137 H3：别名展开成 cargo test --all 照拒」「d137 H3：别名 cargo 自展开（换行、先定义后 shopt）照拒」「d137 H3：别名展开成起脚本（同一条里 alias 先提到脚本、认不出写法）照拒」（应当是 2，实际 0；两格无害别名不红）
  下面七项（认得出起的是什么、不起重活的照放）；逐格的失败行原文写在 project-code-guard.sh 文件头各开关名下，
  bash-command-detector.sh --selftest 在这七项下都不红（实测 0 格）：
  LIB_SHELL_WORDS_BREAK=sed-expansions-unparsed  sed_script_commands() 不另算 sed 脚本词里没展开的 bash 展开，原样切、切不下去照「解析不下去」拒
      project-code-guard.sh --selftest：3 格红（「d137-r2」开头的那几格）
  LIB_SHELL_WORDS_BREAK=sed-text-first-line-only  sed_script_parse() 的 a / i / c 文本只读到头一个换行（`a\\` 换行之后的文本行被当命令读）
      project-code-guard.sh --selftest：2 格红（「d137-r2」开头的那几格）
  LIB_SHELL_WORDS_BREAK=python-argument-variables-unresolved  scan_python() 起命令的参数只认整个字面的（argv 里有一项是变量、赋过字面值的变量、字面串的 .split() 都整条认不出）
      project-code-guard.sh --selftest：3 格红（「d137-r2」开头的那几格）
  LIB_SHELL_WORDS_BREAK=awk-command-variables-unresolved  scan_awk() 不认赋过字面串的变量（`cmd = "date"; cmd | getline`、命令行 cmd=sort 再 print | cmd 照写成变量拒）
      project-code-guard.sh --selftest：2 格红（「d137-r2」开头的那几格）
  LIB_SHELL_WORDS_BREAK=cd-known-targets-unexpanded  changed_directory() 不认 $PWD、$OLDPWD、`cd -`、起头的 `$(git rev-parse --show-toplevel)` 与 `$(pwd)`（cd 之后目录算不出）
      project-code-guard.sh --selftest：9 格红（「d137-r2」开头的那几格）
  LIB_SHELL_WORDS_BREAK=cd-failure-guards-unseen  CommandAtPosition.cd_failure_skips 一律是假（`cd 不在的目录 && 相对脚本`、`|| exit` 守着的也照「cd 会失败」拒；H5）
      project-code-guard.sh --selftest：5 格红（「d137-r2」开头的那几格）
  LIB_SHELL_WORDS_BREAK=non-interactive-return-unseen  non_interactive_return_offset() 交 None：非交互 shell source 的文件里 `case $- in *i*) ;; *) return;; esac` 这类守卫不截、整份照读
      project-code-guard.sh --selftest：1 格红（「d137-r2」开头的那几格）
  下面七项：逐格的失败行原文写在 bash-command-detector.sh 文件头各开关名下
  （aliases-same-line-expanded 那一条写在 project-code-guard.sh 文件头），
  lib_project_code_runs.py 的 --selftest 在这七项下都不红，project-code-guard.sh 的只在 aliases-same-line-expanded 下红一格：
  LIB_SHELL_WORDS_BREAK=aliases-same-line-expanded  commands_at_command_position() 不看别名定义在哪一行：同一行里先定义再用的照展开（E4）
      project-code-guard.sh --selftest：1 格红（「d137 闸误拒 E4：同一行先 alias 再用，这一行不展开别名、放行」）
  LIB_SHELL_WORDS_BREAK=heredoc-variables-unexpanded  interpreter_standard_input() 不代入 heredoc 正文里前面赋过字面值的变量：结束词不带引号、正文有 `$` 一律判不了（E1）
      bash-command-detector.sh --selftest：1 格红（「d137 误拒 E1」开头、正文变量前面赋过字面值的那一格）
  LIB_SHELL_WORDS_BREAK=python-list-concatenation-unresolved  python_command_value() 不认 argv 的 `+ 列表` 与 `*列表`：整条认不出（E6）
      bash-command-detector.sh --selftest：2 格红（「d137 误拒 E6」开头、命令词是 du 的那两格）
  LIB_SHELL_WORDS_BREAK=python-compile-as-execution  scan_python() 把 compile 与 exec、eval 一样当执行（E7）
      bash-command-detector.sh --selftest：1 格红（「d137 误拒 E7」开头、只调 compile 的那一格）
  LIB_SHELL_WORDS_BREAK=sed-unknown-flags-as-execution  sed_script_parse() 的 s 标志区退回扫到 `;`、换行、`}`、`w` 为止、区里有 e 就算 e 标志（E8）
      bash-command-detector.sh --selftest：1 格红（「d137 误拒 E8」开头、标志区有 GNU sed 不认的字的那一格）
  LIB_SHELL_WORDS_BREAK=cd-subshell-guards-unseen  commands_at_command_position() 不调 guarded_subshell_boundary()：括号进出一律断开 `&&` 一串（E9）
      bash-command-detector.sh --selftest：2 格红（「d137 误拒 E9」开头、照 && 守着的那两格）
  LIB_SHELL_WORDS_BREAK=posix-uncalled-functions-run  drop_uncalled_function_bodies() 只认 function 关键字：只定义不调用的 `名字() { …; }` 照判函数体（legA-r2 零星）
      project-code-guard.sh --selftest：1 格红（「名字() { 函数体 } 只定义不调用：函数体不跑，放行」）
  这一批新加的格在已有开关下也红（逐格写在 bash-command-detector.sh 文件头）：cd-failure-guards-unseen 下 E9 照 && 守着的那两格；
  python-argument-variables-unresolved 下 E6 命令词是 du 的那两格；builtin-non-builtin-run 下检出钩子 builtin coproc 那两格（「stdin:d137 误拒：builtin」开头）
"""
import os
import re
import shlex
from typing import NamedTuple

SHELL_NAMES = {"bash", "sh", "dash", "zsh", "ksh"}
# 改名 / 别的 shell（认不出的一律拒，偏严）：rbash 是限制 bash、ash 是 busybox / BSD 的 POSIX shell，都当 shell 判
# （rbash 的启动文件、posix 判定照 bash）；busybox 的 sh / ash 这类 applet 由 skip_prefixes 剥掉 busybox 这一层。弄坏开关 LIB_SHELL_WORDS_BREAK=renamed-shells-unseen 下都不认。
RENAMED_SHELL_NAMES = {"rbash", "ash"}
BUSYBOX_SHELL_APPLETS = {"sh", "ash", "bash", "dash", "rbash"}
# shell 起的脚本路径写成标准输入设备（读的是标准输入，不是盘上一份脚本）：/dev/stdin、/dev/fd/0、/proc/self/fd/0、/proc/<pid>/fd/0
STANDARD_INPUT_DEVICE = re.compile(r"/dev/stdin|/dev/fd/0|/proc/(?:self|\d+)/fd/0")


def renamed_shells_unseen():
    return words_break_is_set("renamed-shells-unseen")


def is_shell_name(name):
    """name（已取 basename）当不当 shell 判：bash / sh / dash / zsh / ksh，连同 rbash、ash（弄坏开关 renamed-shells-unseen 下只认前五个）。"""
    return name in SHELL_NAMES or (not renamed_shells_unseen() and name in RENAMED_SHELL_NAMES)


def is_standard_input_device(word):
    """word 是不是指到标准输入的设备路径（/dev/stdin、/dev/fd/0、/proc/self/fd/0 这类）。"""
    return bool(STANDARD_INPUT_DEVICE.fullmatch(word))


def expand_directory_tilde(word, directory):
    """词开头的 `~+`（$PWD，取 directory）、`~-`（$OLDPWD，算不出）照 bash 展开，其余的 `~` / `~user` 照 os.path.expanduser；
    `~+` 的 directory 是 None 时交 None，`~-` 一律交 None。弄坏开关 LIB_SHELL_WORDS_BREAK=tilde-plus-minus-unexpanded 下 `~+` / `~-` 不展开（当普通路径，`~+/x` 落不到实处）。"""
    if not words_break_is_set("tilde-plus-minus-unexpanded"):
        if word == "~+" or word.startswith("~+/"):
            return None if directory is None else directory + word[2:]
        if word == "~-" or word.startswith("~-/"):
            return None
    return os.path.expanduser(word)
SHELL_OPERATORS = sorted(["&&", "||", ";;", "|&", "&>>", "&>", ">>", ">&", "<&", ">|", "<<<", "<<-", "<<", "<>",
                          ";", "&", "|", "(", ")", "<", ">", "\n"], key=len, reverse=True)
OPERATOR_START = set(";&|()<>\n")
COMMAND_SEPARATORS = {"&&", "||", ";;", "|&", ";", "&", "|", "(", ")", "\n"}
OUTPUT_REDIRECTS = {">", ">>", ">|", "&>", "&>>", ">&"}
REDIRECT_OPERATORS = OUTPUT_REDIRECTS | {"<", "<>", "<&", "<<", "<<-", "<<<"}   # 后面跟一个目标词的重定向运算符
COMMAND_KEYWORDS = {"if", "then", "else", "elif", "do", "while", "until", "!", "{", "}", "time"}
# 命令前面可以挂的前缀：名字 → 要带一个值的选项；timeout 另有一个时长位置参数，taskset 另有一个 CPU 掩码位置参数
PREFIX_OPTIONS_WITH_VALUE = {
    "nohup": set(), "setsid": set(), "command": set(), "exec": {"-a"},
    "nice": {"-n", "--adjustment"}, "ionice": {"-c", "-n", "-p", "-P", "-u", "--class", "--classdata"},
    "timeout": {"-s", "-k", "--signal", "--kill-after"}, "env": {"-u", "--unset", "-C", "--chdir"},
    "stdbuf": {"-i", "-o", "-e", "--input", "--output", "--error"}, "sudo": {"-u", "-g", "--user", "--group"},
    "taskset": set(),
}
PREFIX_POSITIONAL_COUNT = {"timeout": 1, "taskset": 1}
ASSIGNMENT = re.compile(r"^([A-Za-z_][A-Za-z0-9_]*)=(.*)$", re.S)
ARRAY_ASSIGNMENT_START = re.compile(r"^[A-Za-z_][A-Za-z0-9_]*\+?=$")  # `名字=(` 与 `名字+=(` 的前半，后面紧跟左括号
PYTHON_NAME = re.compile(r"^python(?:\d+(?:\.\d+)?)?$")
# python 解释器带一个值的选项：-c <代码>、-m <模块> 定执行什么；-W <警告设置>、-X <实现选项> 与 --check-hash-based-pycs <模式> 的值只是解释器的设置
PYTHON_SHORT_OPTIONS_WITH_VALUE = "cmWX"
PYTHON_LONG_OPTIONS_WITH_VALUE = {"--check-hash-based-pycs"}
# source / . 后面跟的得像一个路径才算读脚本（`source = open(…)` 这类 python 源码里的写法不算）
# bash 带一个值的长选项（`--rcfile 文件`、`--init-file 文件`）：值不是被执行的脚本
SHELL_LONG_OPTIONS_WITH_VALUE = {"--rcfile", "--init-file"}
SOURCE_TARGET = re.compile(r"^[^\s=,-][^\s=,]*$")
HEREDOC = re.compile(r"<<-?\s*(['\"]?)(\w+)\1([^\n]*)\n(.*?)\n\s*\2[ \t]*(?=\n|$)", re.S)
MAXIMUM_NESTING = 8
# 先带一个参数、再接那条命令的包装脚本：capped.sh N（线程上限；N 是正整数或 default，default 取 .claude/heavy-thread-default 的登记值）、
# run-with-memory-cap.sh <上限>（内存上限）。参数不以 - 开头就剥掉它们两个词（不看是数还是 default），往里判那条命令
WRAPPERS_TAKING_ONE_ARGUMENT = {"capped.sh", "run-with-memory-cap.sh"}
# 其中给命令定内存上限的那一个：剥掉它的命令记 memory_capped（project-code-guard.sh 判子 agent 跑编译出来的代码经没经它）
MEMORY_CAP_WRAPPER = "run-with-memory-cap.sh"
# 剥掉的 heredoc 在剥完的文字里换成这个记号（NUL 开头，不是 \w，不会被 HEREDOC 再认一次），切词之后它是 `<<` 的目标词，按编号找回正文
HEREDOC_BODY_MARKER = re.compile(r"^\x00heredoc-body-(\d+)\x00$")


class RedirectOperator(str):
    """重定向运算符，带着写在它前面的 fd 号（`2>` 的 "2"）：与运算符本身逐字相等，按字面比运算符的地方照旧；没写 fd 号的运算符是普通的 str。"""
    fd = None

    def __new__(cls, operator, fd):
        instance = super().__new__(cls, operator)
        instance.fd = fd
        return instance


def redirection_fd(operator):
    """写在重定向运算符前面的 fd 号，照 bash 按十进制数认（`01>` 是 1 号）；没写交 None。
    弄坏开关 LIB_SHELL_WORDS_BREAK=redirect-fd-number-literal 下照字面比：写法不是最简十进制的（`01`）当作哪个号都对不上。"""
    fd = getattr(operator, "fd", None)
    if fd is None:
        return None
    if words_break_is_set("redirect-fd-number-literal") and fd != str(int(fd)):
        return -1
    return int(fd)


def redirects_standard_output(operator):
    """这个输出重定向改的是不是这一条的标准输出：`>`、`>>`、`>|` 前面没写 fd 号或写的是 1（照数认，`01>` 也是），`&>`、`&>>`（`2> 文件` 不算；fd 号见 RedirectOperator）。"""
    return operator in OUTPUT_REDIRECTS and operator != ">&" and redirection_fd(operator) in (None, 1)


def standard_output_destination(redirections):
    """一条简单命令跑起来时标准输出最后落到哪个文件：照 bash 从左到右处理输出重定向（((运算符, 目标词), …)，CommandAtPosition.output_redirections 那种）。
    `n>文件`、`n>>文件`、`n>|文件` 改 n 号（不写是 1）；`&>`、`&>>` 与 `>&文件`（不写 n 或写 1）同时改 1、2 号；`n>&m` 把 n 号接到 m 号此刻指的地方
    （`cat 源 3>d.sh >&3` 的标准输出进 d.sh）；`n>&-` 关掉 n 号；`n>&文件`（n 不是 1）bash 报 ambiguous redirect、整条不跑。
    交 (目标词, 打开那个文件的运算符：`>`、`>>`、`>|`、`&>`、`&>>`、`>&` 之一)；标准输出没落到文件（没改、接到别的继承来的号、关了、整条不跑）交 None。
    弄坏开关 LIB_SHELL_WORDS_BREAK=redirect-fd-duplication-unseen 下不跟 `n>&m`：只取直接改标准输出（redirects_standard_output）的最后一个。"""
    if words_break_is_set("redirect-fd-duplication-unseen"):
        direct = [(target, str(operator)) for operator, target in redirections if redirects_standard_output(operator)]
        return direct[-1] if direct else None
    table = {}   # fd 号 → ("file", 目标词, 运算符) / ("fd", 继承来的号) / ("closed",)；没写的号是继承来的
    for operator, target in redirections:
        plain, fd = str(operator), redirection_fd(operator)
        if plain in ("&>", "&>>"):
            table[1] = table[2] = ("file", target, plain)
        elif plain == ">&":
            number = 1 if fd is None else fd
            if re.fullmatch(r"[0-9]+", target):
                table[number] = table.get(int(target), ("fd", int(target)))
            elif target == "-":
                table[number] = ("closed",)
            elif number == 1:
                table[1] = table[2] = ("file", target, plain)
            else:
                return None
        elif plain in (">", ">>", ">|"):
            table[1 if fd is None else fd] = ("file", target, plain)
    entry = table.get(1, ("fd", 1))
    return (entry[1], entry[2]) if entry[0] == "file" else None


BASH_BUILTINS = frozenset(". : [ alias bg bind break builtin caller cd command compgen complete compopt continue declare dirs disown echo enable eval exec exit "
                          "export false fc fg getopts hash help history jobs kill let local logout mapfile popd printf pushd pwd read readarray readonly return "
                          "set shift shopt source suspend test times trap true type typeset ulimit umask unalias unset wait".split())   # bash 5.2 的 compgen -b


class HeredocBody(NamedTuple):
    operator: str                 # "<<" 或 "<<-"（后者的正文已经去掉每行开头的制表符）
    is_delimiter_quoted: bool     # 结束词带引号：正文照原样写出；不带引号时 shell 还要展开正文里的 $ 与命令替换，这里给的是展开之前的原文
    text: str                     # 正文，最后一行之后带一个换行（写进文件的就是这些字节）


class PrefixSkip(NamedTuple):
    command_position: int | None  # 命令词的下标；这一条只有关键字、赋值与前缀时是 None
    prefix_names: set[str]         # 路过的前缀名（nohup、setsid、nice……）
    assignments: dict[str, str]    # 路过的赋值：命令前的 `名字=值` 与 env 参数里的 `名字=值`
    chdir: str | None = None       # env 的 `-C 目录` / `-C目录` / `--chdir=目录` / `--chdir 目录` 里最后一个目录（照写的样子，没解开）；没有是 None
    split: tuple[int, int, str] | None = None   # 只在 skip_prefixes(…, expand_split=True) 下有：env 的 -S / --split-string 在词表里占的 [起, 止) 与它的字符串，由调用方拆开换进去再判
    environment_cleared: bool = False   # 路过的 env 带 -i / --ignore-environment / 单独的 -：外面带进来的变量与它之前的赋值都清掉，assignments 里只剩它之后的
    login_shell: bool = False          # 路过的 exec 带 -l、或 -a 给的名字以 - 起头：后面那个 shell 照登录 shell 起（读 /etc/profile 与 HOME 下的 profile）


class ShellInvocation(NamedTuple):
    code_string: str | None   # `bash -c '…'` 的那段代码
    script_index: int | None  # 起的是脚本时，脚本名在参数里的下标；只查语法（-n）或没给脚本时是 None
    runs_nothing: bool = False   # 照 bash 什么都不跑：带 -n（只查语法，-c 的代码、脚本、标准输入都不执行），或短选项之后又写长选项（bash 报 --: invalid option）


class PythonInvocation(NamedTuple):
    code_string: str | None   # `python3 -c '…'` 的那段代码
    module: str | None        # `python3 -m <模块>` 的模块名（`-m runpy <模块>` 取后面那个）
    script_index: int | None  # 起的是脚本文件时，脚本名在参数里的下标；-c / -m 时是程序自己的第一个参数的下标（可以等于参数个数）；
                              # 程序从标准输入读（`-`）或只有选项时是 None


class CommandAtPosition(NamedTuple):
    words: list[str]              # words[0] 是命令词（照写的样子，没取 basename），其后是参数
    directory: str | None         # 跑这条命令时的当前目录；cd 到展开不了的变量路径之后认不出，是 None（能展开的见 expand_known_variables）
    environment: dict[str, str]   # 这条命令看得到的、这段命令文本里设过的变量：前面 export 的、命令前与 env 里写的、外层传进来的
    launcher: str | None = None   # words[0] 是经什么起的脚本："shell"（bash / sh 这类后面跟的）、"source"（source / .）、
                                  # "python"（python3 后面跟的程序）、"startup"（shell 启动时先读的 BASH_ENV / ENV 指的文件，startup_file_commands）；
                                  # 直接执行的命令（含 capped.sh N 后面那条）是 None
    output_redirections: tuple = ()      # 这一条简单命令上的输出重定向：((运算符, 目标词), …)，运算符是 OUTPUT_REDIRECTS 之一；fd 号不在里面
    standard_input_heredocs: tuple = ()  # 喂给这一条的、被当数据剥掉的 heredoc：(HeredocBody, …)，按出现次序（最后一个才是它读到的标准输入）
    memory_capped: bool = False          # 经 run-with-memory-cap.sh 跑：它自己的包装里有，或外层（bash -c 的那段代码、起它所在脚本的那一条）有
    variables: dict | None = None        # 跑这条命令时这段命令文本里记下的变量（名字 → 已展开的值，None 是算不出）：调用方拿它给 expand_known_variables 展开参数
    piped_from: tuple | None = None      # 管道前一段：(词, 喂给它的 heredoc 正文)；不在管道后面是 None（认 `cat 源 | tee 目的` 这类拷出用）
    input_redirections: tuple = ()       # 这一条上改标准输入的重定向 ((运算符, 目标词), …)，STANDARD_INPUT_REDIRECTS 里的、heredoc 除外（认 `tee 目的 < 源` 用）
    wrappers: tuple = ()                 # 剥掉的 capped.sh / run-with-memory-cap.sh：((照写的命令词, 那一刻的当前目录), …)，外层（bash -c 的那段代码所在那一条）剥的也在里面；
                                         # 调用方核它是不是本仓那一份（project-code-guard.sh：路径尾部相同的别处拷贝照读正文、不算经内存包装）
    cd_failure_skips: bool = False       # 前面每一处可能失败的 cd / pushd（目标盘上不在）失败了，这一条都不会跑（`cd 目录 && 这一条`、`cd 目录 || exit` 之后同一层的）：
                                         # 目录盘上不在时调用方照 cd 成功了的那个目录判；False 是认不准，照「目录不在、cd 会失败」办
    interactive: bool = False            # 跑这一条的 shell 可能是交互的（带 -i，或不带 -c、不带脚本、从标准输入读）：它 source 的文件里 `case $- in *i*)…return` 这类守卫不截
                                         # （non_interactive_return_offset）


class ShellScan(NamedTuple):
    commands: list[CommandAtPosition]  # 命令位置上的每一条命令，包装（bash -c、capped.sh、source……）剥掉之后的那一条
    prefix_names: set[str]             # 哪一层、哪一条命令路过的前缀名都算
    has_lone_ampersand: bool           # 哪一层有单独的 &（放后台；不是 &&、>&、&>、|&）
    output_targets: list[str]          # 哪一层的输出重定向目标都算
    unjudged_shell_inputs: tuple = ()  # 从标准输入读命令的 shell 收到的、这里判不了的输入（每处一句说明，哪一层的都算；shell_standard_input）：
                                       # 调用方按自己的「判不了」处置（放行并记检出）


def heredoc_feeds_shell(head):
    """heredoc 所在那一行 `<<` 之前的文字：它挂在的那条简单命令（最后一条），剥掉前缀与 capped.sh N 之后，
    是不是从标准输入读命令的 bash / sh 这类（不带 -c、不带脚本，或带 -s）。同一行别处出现 shell 的名字（`x.sh`、前一条 bash）不算。
    `<<` 落在一个还没收尾的命令替换或进程替换里面（`x=$(bash <<EOF`、`diff <(bash <<EOF`、反引号同）时，挂的命令在那一层里找（open_substitution_tail）。
    弄坏开关 LIB_SHELL_WORDS_BREAK=heredoc-open-substitution-unseen 下照旧拿整行切（替换里的 bash 认不出，正文当数据剥掉）。"""
    if not words_break_is_set("heredoc-open-substitution-unseen"):
        head = open_substitution_tail(head)
    simple = simple_commands(shell_tokens(head))[0]
    if not simple or not simple[-1]:
        return False
    words = simple[-1]
    for _ in range(MAXIMUM_NESTING):
        skipped, words = skip_prefixes_expanding(words)
        if skipped.command_position is None:
            return False
        words = words[skipped.command_position:]
        name, arguments = os.path.basename(words[0]), words[1:]
        if name in WRAPPERS_TAKING_ONE_ARGUMENT and len(arguments) >= 2 and not arguments[0].startswith("-"):
            words = arguments[1:]
            continue
        if not is_shell_name(name):
            return False
        return shell_reads_standard_input(arguments)
    return False


def open_substitution_tail(text):
    """text 末尾还开着的最里层命令替换或进程替换（`$(`、`<(`、`>(`、反引号；收尾了的、引号里的括号不算）之后的文字；都收尾了交回 text 原样。
    引号、反斜杠、替换的收尾与 shell_tokens 用的是同一套扫描（command_substitution_end、double_quote_end、backquote_end，没配上时由它们记下最里层还开着的正文从哪起）：
    双引号里收了尾的 `$(…)` 不算开着（`cd "$(git rev-parse --show-toplevel)" && bash <<EOF` 挂的是顶层的 bash）；`<(`、`>(` 只在引号外认，词首的 `#` 起到行尾是注释。
    弄坏开关 LIB_SHELL_WORDS_BREAK=double-quoted-substitution-unclosed 下双引号里的 `$(` 一律当没收尾。"""
    position, at_word_start = 0, True
    while position < len(text):
        character = text[position]
        unclosed, end = [], None
        if character == "\\":
            position, at_word_start = position + 2, False
            continue
        if text.startswith("$'", position) and not ansi_c_quotes_unseen():
            end = ansi_c_quote_end(text, position + 2)   # ANSI-C 引号里的 `\'` 不收尾
        elif character == "'":
            end = text.find("'", position + 1)
            end = -1 if end < 0 else end + 1
        elif character == '"':
            if words_break_is_set("double-quoted-substitution-unclosed"):
                closing = text.find('"', position + 1)
                opening = text.find("$(", position + 1, len(text) if closing < 0 else closing)
                if opening >= 0:
                    return text[opening + 2:]
            end = double_quote_end(text, position + 1, unclosed)
        elif text.startswith("$(", position) or text.startswith(("<(", ">("), position):
            end = command_substitution_end(text, position + 2, unclosed)
        elif character == "`":
            end = backquote_end(text, position + 1, unclosed)
        elif character == "#" and at_word_start:
            return text
        if end is None:
            at_word_start = character in " \t\r\n;&|()<>"
            position += 1
            continue
        if end < 0:
            return text[unclosed[0]:] if unclosed else text
        position, at_word_start = end, False
    return text


def shell_reads_standard_input(arguments):
    """bash / sh 这类后面的参数：是不是从标准输入读命令（不带 -c、不带脚本，或带 -s）。heredoc 喂给 shell 与 here-string、管道喂给 shell 共用这一判。"""
    invocation = shell_invocation(arguments)
    if invocation.code_string is not None or invocation.runs_nothing:
        return False
    reads_standard_input = any(argument.startswith("-") and not argument.startswith("--") and "s" in argument[1:]
                               for argument in arguments)
    return reads_standard_input or invocation.script_index is None


def strip_data_heredocs(command, keep_line_count=False, stripped_bodies=None, shell_heredocs=None):
    """喂给非 shell 命令的 heredoc 正文剥掉（它是数据，不是在跑的命令）；喂给 bash / sh 这类的照留（判据在 heredoc_feeds_shell）。
    keep_line_count 为真时正文换成同样多的空行，剥完之后每一行的行号不变（报脚本里第几行要用）。
    stripped_bodies 是一个列表时，每剥掉一段就把它的 HeredocBody 追加进去，剥完的文字里结束词换成 HEREDOC_BODY_MARKER 那种记号、带上它在列表里的下标。
    shell_heredocs 是一个列表时，喂给 shell 的那些也剥掉，(它所在那一行 `<<` 之前的文字, 正文) 按次序追加进去，由调用方另判
    （正文里的命令算不算在挂它的那条命令的包装里，比如外面有没有 timeout）。"""
    def replace(match):
        head = command[command.rfind("\n", 0, match.start()) + 1:match.start()]
        line_breaks = "\n" * match.group(0).count("\n") if keep_line_count else "\n"
        delimiter = f"{match.group(1)}{match.group(2)}{match.group(1)}"
        if heredoc_feeds_shell(head):
            if shell_heredocs is None:
                return match.group(0)
            shell_heredocs.append((head, match.group(4)))
        elif stripped_bodies is not None:
            operator = "<<-" if match.group(0).startswith("<<-") else "<<"
            body = match.group(4)
            if operator == "<<-":
                body = "\n".join(line.lstrip("\t") for line in body.split("\n"))
            delimiter = f"\x00heredoc-body-{len(stripped_bodies)}\x00"
            stripped_bodies.append(HeredocBody(operator, bool(match.group(1)), body + "\n"))
        return f"<<{delimiter}{match.group(3)}{line_breaks}{match.group(2)}"
    return HEREDOC.sub(replace, command)


ANSI_C_ESCAPES = {"a": "\a", "b": "\b", "e": "\x1b", "E": "\x1b", "f": "\f", "n": "\n", "r": "\r", "t": "\t", "v": "\v",
                  "\\": "\\", "'": "'", '"': '"', "?": "?"}


def ansi_c_quotes_unseen():
    """弄坏开关 LIB_SHELL_WORDS_BREAK=ansi-c-quotes-unseen：`$'…'` 照普通单引号配对（里面的 `\\'` 不认；只给自检证红用）。"""
    return words_break_is_set("ansi-c-quotes-unseen")


def ansi_c_quote_end(text, start):
    """ANSI-C 引号 `$'…'` 里（start 指向开头那个单引号后面第一个字符）找收尾的单引号，交回它后面那个下标；里面反斜杠转义下一个字符（`\\'` 不收尾）；没配上交 -1。
    调用方在弄坏开关 ansi-c-quotes-unseen 下不调它（`$` 当普通字符、后面的单引号照普通单引号配对）。"""
    position = start
    while position < len(text):
        if text[position] == "\\":
            position += 2
        elif text[position] == "'":
            return position + 1
        else:
            position += 1
    return -1


def ansi_c_decoded(body):
    """`$'…'` 里的正文照 bash 换掉转义：\\a \\b \\e \\E \\f \\n \\r \\t \\v \\\\ \\' \\" \\?、\\nnn（八进制一到三位）、\\xHH、\\uHHHH、\\UHHHHHHHH、\\cX；
    认不出的照原样留下反斜杠与那个字符；换出 NUL 的，bash 在那里截断。"""
    pieces, index = [], 0
    while index < len(body):
        character = body[index]
        following = body[index + 1:index + 2]
        if character != "\\" or not following:
            pieces.append(character)
            index += 1
            continue
        octal = re.match(r"[0-7]{1,3}", body[index + 1:])
        hexadecimal = {"x": 2, "u": 4, "U": 8}.get(following)
        digits = re.match(r"[0-9A-Fa-f]{1,%d}" % hexadecimal, body[index + 2:]) if hexadecimal else None
        if following in ANSI_C_ESCAPES:
            pieces.append(ANSI_C_ESCAPES[following])
            index += 2
        elif octal:
            pieces.append(chr(int(octal.group(0), 8) & 0xFF))
            index += 1 + len(octal.group(0))
        elif digits:
            value = int(digits.group(0), 16)
            pieces.append(chr(value) if value <= 0x10FFFF else "�")
            index += 2 + len(digits.group(0))
        elif following == "c" and index + 2 < len(body):
            pieces.append(chr(ord(body[index + 2]) & 0x1F))
            index += 3
        else:
            pieces.append(character + following)
            index += 2
    return "".join(pieces).split("\x00", 1)[0]


def backquote_end(text, start, unclosed=None):
    """反引号之后（start 指向开头那个反引号后面第一个字符）找收尾的反引号，交回它后面那个下标；没配上交 -1。
    unclosed 是一个列表、而这一段没配上时，往里追加还开着的最里层替换的正文从哪起（先追加的是最里层；open_substitution_tail 用）。"""
    position = start
    while position < len(text):
        if text[position] == "\\":
            position += 2
        elif text[position] == "`":
            return position + 1
        else:
            position += 1
    if unclosed is not None:
        unclosed.append(start)
    return -1


def double_quote_end(text, start, unclosed=None):
    """双引号之后找收尾的双引号，交回它后面那个下标；里面的 `$(…)` 与反引号整段跳过（它们里面的双引号不算收尾）；没配上交 -1。unclosed 同 backquote_end。"""
    position = start
    while position < len(text):
        character = text[position]
        if character == "\\":
            position += 2
        elif character == '"':
            return position + 1
        elif text.startswith("$(", position):
            position = command_substitution_end(text, position + 2, unclosed)
        elif character == "`":
            position = backquote_end(text, position + 1, unclosed)
        else:
            position += 1
        if position < 0:
            return -1
    return -1


def substitution_case_and_comment_unseen():
    """弄坏开关 LIB_SHELL_WORDS_BREAK=substitution-case-and-comment-unseen：命令替换里不认 case 的模式与注释，模式的 `)`、注释里的 `)` 照括号配对（只给自检证红用）。"""
    return words_break_is_set("substitution-case-and-comment-unseen")


SUBSTITUTION_WORD_END = set(" \t\r\n;&|()<>")
SUBSTITUTION_COMMAND_KEYWORDS = {"do", "then", "else", "elif", "if", "while", "until", "!", "{", "time", "coproc"}


def command_substitution_end(text, start, unclosed=None):
    """`$(` 之后（start 指向它后面第一个字符）找配对的 `)`，交回它后面那个下标；引号、反斜杠、反引号与嵌套的括号都跳过；没配上交 -1。unclosed 同 backquote_end。
    照 bash 认两处不配对的右括号（`echo "$(case a in a) cargo test --all;; esac)"` 不在 `a)` 处收尾，否则 cargo 漏判）：
    词首的 `#` 起到行尾是注释（里面的 `)` 不算）；命令位置上的 `case 词 in` 之后、`;;` / `;&` / `;;&` 之后到右括号为止是模式，模式的 `(`、`)` 不算括号，
    `esac` 收尾（case 可以套 case）。弄坏开关 LIB_SHELL_WORDS_BREAK=substitution-case-and-comment-unseen 下不认这两处。"""
    depth, position = 1, start
    aware = not substitution_case_and_comment_unseen()
    cases = []                 # 套着的 case：每层 "subject"（等 in）、"pattern"（在模式里）、"body"（在那一项的命令里）
    command_start = True       # 下一个词在命令位置上
    while position < len(text):
        character = text[position]
        word_start = position == start or text[position - 1] in SUBSTITUTION_WORD_END
        if aware and word_start and character == "#":
            end = text.find("\n", position)
            position = len(text) if end < 0 else end
            continue
        if aware and word_start and (character.isalpha() or character == "_"):
            end = position
            while end < len(text) and (text[end].isalnum() or text[end] == "_"):
                end += 1
            if end == len(text) or text[end] in SUBSTITUTION_WORD_END:
                word = text[position:end]
                if word == "case" and command_start:
                    cases.append("subject")
                elif word == "in" and cases and cases[-1] == "subject":
                    cases[-1] = "pattern"
                    position, command_start = end, False
                    continue
                elif word == "esac" and cases and (command_start or cases[-1] == "pattern"):
                    cases.pop()
                    position, command_start = end, False
                    continue
                command_start = word in SUBSTITUTION_COMMAND_KEYWORDS
                position = end
                continue
        if aware and cases and cases[-1] == "body" and text.startswith(";;", position) or aware and cases and cases[-1] == "body" and text.startswith(";&", position):
            cases[-1] = "pattern"
            position += 3 if text.startswith(";;&", position) else 2
            command_start = False
            continue
        if aware and cases and cases[-1] == "pattern" and character in "()":
            if character == ")":
                cases[-1] = "body"
                command_start = True
            position += 1
            continue
        if character in ";&|(\n":
            command_start = True
        elif character not in " \t\r":
            command_start = False
        if character == "\\":
            position += 2
        elif text.startswith("$'", position) and not ansi_c_quotes_unseen():
            position = ansi_c_quote_end(text, position + 2)   # ANSI-C 引号里的 `\'` 不收尾
        elif character == "'":
            end = text.find("'", position + 1)
            position = -1 if end < 0 else end + 1
        elif character == '"':
            position = double_quote_end(text, position + 1, unclosed)
        elif character == "`":
            position = backquote_end(text, position + 1, unclosed)
        elif character == "(":
            depth += 1
            position += 1
        elif character == ")":
            depth -= 1
            position += 1
            if depth == 0:
                return position
        else:
            position += 1
        if position < 0:
            break
    if unclosed is not None:
        unclosed.append(start)
    return -1


def starts_substitution(text, position):
    """text[position:] 是不是以命令替换（`$(`、反引号）或进程替换（`<(`、`>(`）开头；引号外用，双引号里只有前两种算。
    `>>(`、`&>(`、`<<(` 这类运算符在前的不在内：切词在前一个字符上就把运算符认走了（bash 也按运算符读，后面的 `(` 是语法错）。"""
    return text.startswith(("$(", "<(", ">("), position) or text.startswith("`", position)


def substitution_span(text, position):
    """text[position:] 以 `$(`、`$((`、`<(`、`>(` 或反引号开头：交回 (这一段结束的下标, 里面要当命令判的正文或 None)。
    算术 `$((…))` 的正文不是命令，交 None；进程替换 `<(…)`、`>(…)` 的正文是命令（`<((…))` 是里面套了一层子 shell，不是算术）；没配上的，剩下的整段都算它。"""
    if text.startswith("`", position):
        end = backquote_end(text, position + 1)
        return (len(text), text[position + 1:]) if end < 0 else (end, text[position + 1:end - 1])
    end = command_substitution_end(text, position + 2)
    inner = text[position + 2:] if end < 0 else text[position + 2:end - 1]
    return (len(text) if end < 0 else end), (None if text.startswith("$((", position) else inner)


def shell_tokens(text):
    """切成 (是不是运算符, 文本)：引号去掉、词首的 # 注释去掉、重定向前的 fd 号去掉；引号没配对时剩下的当一个词。
    命令替换（`$(…)`、反引号，引号里外都算）与进程替换（`<(…)`、`>(…)`，只在引号外算，在词首、词中都算）照原样留在所在的词里，
    里面的正文作为 (None, 正文) 紧跟在那个词后面交出；`)` 之后紧跟的字符接在同一个词上，隔了空白的是下一个词。
    引号外的花括号照 bash 展开成几个词（`cp X{,.bak}` 是 `cp X X.bak`；`{a,b}`、`{1..3}`、`{a..c}`，嵌套的照展，`${…}` 不算；brace_expand），
    整词像赋值（`名字=…`）的不展开（bash 不对赋值词做花括号展开）。弄坏开关 LIB_SHELL_WORDS_BREAK=braces-unexpanded 下照旧整个留一个词。"""
    tokens, word, mask, position, pending_substitutions = [], None, "", 0, []
    def add(piece, quoted):
        nonlocal word, mask
        if quoted and not piece:
            piece, quoted = QUOTED_EMPTY, None   # 引号写出的空串：占一个位置（掩码 "e"），花括号展开之后据它判这一份是不是引号写出的空串，再去掉
        word = (word or "") + piece
        mask += {True: "q", False: "u", None: "e"}[quoted] * len(piece)
    def flush():
        nonlocal word, mask
        if word is not None:
            expansions = brace_expand(word, mask)
            target_of = tokens[-1][1] if tokens and tokens[-1][0] is True and tokens[-1][1] in REDIRECT_OPERATORS else None
            if (target_of is not None and (target_of in ("<<", "<<-", "<<<") or len(expansions) != 1)
                    and not words_break_is_set("redirect-target-braces-expanded")):
                # 重定向的目标只有一个词：here-string 与 heredoc 的结束词 bash 不做花括号展开；别的展开成几个词（或一个不剩）bash 报 ambiguous redirect、什么都不写。
                # 都照原样留一个词：拷出再执行那一类认不出这个落点（不当写出了文件），写范围那一类照「落点算不出」拒（偏严，bash 本来也不写）。
                # 弄坏开关 redirect-target-braces-expanded 下照旧展开（多出来的词成了这一条的参数）
                expansions = ["".join(character for character, kind in zip(word, mask) if kind != "e")]
            tokens.extend((False, expanded) for expanded in expansions)
            word, mask = None, ""
        tokens.extend((None, inner) for inner in pending_substitutions)
        pending_substitutions.clear()
    def take_substitution(start):
        end, inner = substitution_span(text, start)
        if inner is not None:
            pending_substitutions.append(inner)
        return end
    while position < len(text):
        character = text[position]
        if character in " \t\r":
            flush()
            position += 1
        elif character == "\\":
            if not text.startswith("\\\n", position):
                add(text[position + 1:position + 2], True)
            position += 2
        elif character == "'":
            end = text.find("'", position + 1)
            end = len(text) if end < 0 else end
            add(text[position + 1:end], True)
            position = end + 1
        elif character == '"':
            position += 1
            piece = ""
            while position < len(text) and text[position] != '"':
                if text[position] == "\\" and text[position + 1:position + 2] in ("$", "`", '"', "\\", "\n"):
                    piece += text[position + 1]
                    position += 2
                elif text.startswith("$(", position) or text[position] == "`":
                    end = take_substitution(position)
                    piece += text[position:end]
                    position = end
                else:
                    piece += text[position]
                    position += 1
            add(piece, True)
            position += 1
        elif character == "#" and word is None:
            end = text.find("\n", position)
            position = len(text) if end < 0 else end
        elif text.startswith("$'", position) and not ansi_c_quotes_unseen():
            # ANSI-C 引号：反斜杠转义照 bash 换掉（`$'a\'b'` 是 a'b、`$'\x63argo'` 是 cargo），整段算引号里的（不做花括号展开、不当 fd 号）
            end = ansi_c_quote_end(text, position + 2)
            end = len(text) + 1 if end < 0 else end
            add(ansi_c_decoded(text[position + 2:end - 1]), True)
            position = end
        elif starts_substitution(text, position):
            end = take_substitution(position)
            add(text[position:end], True)
            position = end
        elif character in OPERATOR_START:
            operator = next(candidate for candidate in SHELL_OPERATORS if text.startswith(candidate, position))
            # 紧贴在前面、整个都是引号外的十进制数字的词才是 fd 号（`2>`、`01>`）；引号或反斜杠写出的数字（`"2">`、`\2>`）是参数（弄坏开关 redirect-fd-quoted-digits 下照旧当 fd 号）
            if (operator[0] in "<>" and word is not None and re.fullmatch(r"[0-9]+", word)
                    and (set(mask) == {"u"} or words_break_is_set("redirect-fd-quoted-digits"))):
                if not words_break_is_set("redirect-fd-dropped"):
                    operator = RedirectOperator(operator, word)
                word, mask = None, ""
            flush()
            tokens.append((True, operator))
            position += len(operator)
        else:
            add(character, False)
            position += 1
    flush()
    return case_patterns_and_conditionals_folded(tokens)


class UnfoldableCase(Exception):
    """case 语句的模式切不出来（模式里夹了别的运算符、到头没有收尾的右括号）：整段照原样交回。"""


def case_patterns_and_conditionals_folded(tokens):
    """切好的记号里，bash 不当命令跑的两处照 bash 改写，免得被当成命令位置：
    ① case 语句（命令位置上的 `case 词 in`）的模式：`in` 之后、`;;` / `;&` / `;;&` 之后到右括号为止的词连同 `|`、开头可写的左括号都去掉，
       收尾的右括号换成一个 `;`（模式之后是那一项的命令，在命令位置上）；模式里的命令替换照留（bash 展开模式时照跑它），`;;&` 后面的 `&` 不当放后台；
       命令位置上的 `esac` 收尾，case 可以套 case。弄坏开关 LIB_SHELL_WORDS_BREAK=case-patterns-as-commands 下不改（`;;` 之后的模式词当命令词）。
    ② 条件表达式（命令位置上的 `[[ … ]]`）里的 `&&`、`||`、`(`、`)`、`<`、`>` 是表达式的一部分，不是命令分隔与重定向：换成普通的词（`2>` 那种 fd 号还原成一个词），
       里面的换行去掉，整段是 `[[` 这一条命令的参数。没有收尾的 `]]` 不改。弄坏开关 LIB_SHELL_WORDS_BREAK=conditional-expression-split 下不改。
    模式切不出来的（夹了别的运算符、到头没有收尾的右括号）整段照原样交回。"""
    fold_cases = not words_break_is_set("case-patterns-as-commands")
    fold_conditionals = not words_break_is_set("conditional-expression-split")
    if not (fold_cases and (False, "case") in tokens) and not (fold_conditionals and (False, "[[") in tokens):
        return tokens
    output, index, at_command, cases = [], 0, True, []   # cases：每层没收尾的 case 此刻在模式里（"pattern"）还是在一项的命令里（"body"）
    try:
        while index < len(tokens):
            kind, value = tokens[index]
            if cases and cases[-1] == "pattern":
                if (kind, value) == (True, "\n"):
                    output.append((kind, value))
                    index += 1
                    continue
                if (kind, value) == (False, "esac") and tokens[index + 1:index + 2] not in ([(True, ")")], [(True, "|")]):
                    cases.pop()
                    output.append((kind, value))
                    index, at_command = index + 1, False
                    continue
                if (kind, value) == (True, "("):
                    index += 1
                while index < len(tokens) and tokens[index] != (True, ")"):
                    if tokens[index][0] is None:
                        output.append(tokens[index])
                    elif tokens[index][0] is True and tokens[index][1] != "|":
                        raise UnfoldableCase()
                    index += 1
                if index >= len(tokens):
                    raise UnfoldableCase()
                output.append((True, ";"))
                cases[-1] = "body"
                index, at_command = index + 1, True
                continue
            if kind is False and at_command and value == "[[" and fold_conditionals:
                closing = next((position for position in range(index + 1, len(tokens)) if tokens[position] == (False, "]]")), None)
                if closing is not None:
                    output.append((kind, value))
                    for inner_kind, inner_value in tokens[index + 1:closing]:
                        if inner_kind is not True:
                            output.append((inner_kind, inner_value))
                        elif inner_value != "\n":
                            if getattr(inner_value, "fd", None) is not None:
                                output.append((False, inner_value.fd))
                            output.append((False, str(inner_value)))
                    output.append(tokens[closing])
                    index, at_command = closing + 1, False
                    continue
            if kind is False and at_command and value == "case" and fold_cases and index + 1 < len(tokens) and tokens[index + 1][0] is False:
                cursor = index + 2
                while cursor < len(tokens) and (tokens[cursor][0] is None or tokens[cursor] == (True, "\n")):
                    cursor += 1
                if cursor < len(tokens) and tokens[cursor] == (False, "in"):
                    output.extend(tokens[index:cursor + 1])
                    cases.append("pattern")
                    index, at_command = cursor + 1, False
                    continue
            if cases and (kind, value) == (True, ";;"):
                output.append((kind, value))
                index += 1
                if tokens[index:index + 1] == [(True, "&")]:
                    index += 1   # `;;&`：接着试后面的模式，不是放后台
                cases[-1], at_command = "pattern", True
                continue
            if cases and (kind, value) == (True, ";") and tokens[index + 1:index + 2] == [(True, "&")]:
                output.append((True, ";;"))   # `;&`：落到下一项的命令接着跑，下一项照样有模式
                index += 2
                cases[-1], at_command = "pattern", True
                continue
            if cases and kind is False and at_command and value == "esac":
                cases.pop()
                output.append((kind, value))
                index, at_command = index + 1, False
                continue
            output.append((kind, value))
            if kind is True:
                at_command = value in COMMAND_SEPARATORS
            elif kind is False:
                at_command = at_command and value in COMMAND_KEYWORDS
            index += 1
    except UnfoldableCase:
        return tokens
    return output


BRACE_EXPANSION_LIMIT = 64   # 一个词最多展开成这么多个；再多的照原样留一个词（bash 照展，这里只防判定卡住）
BRACE_SEQUENCE = re.compile(r"^(-?\d+|[A-Za-z])\.\.(-?\d+|[A-Za-z])(?:\.\.(-?\d+))?$")


QUOTED_EMPTY = "\x02"   # 引号写出的空串（`''`、`""`）在切词时占的一个位置（掩码 "e"），brace_expand 交回之前去掉


class ShellWord(str):
    """切好的一个词，带着它里面还留着的花括号是不是都在引号里（`'echo a{,}'` 的花括号是数据，不会再展开成别的）：与词本身逐字相等，按字面比的地方照旧；
    unquoted_braces 为真是还留着引号外的 `{` 或 `}`（`{}`、超过 BRACE_EXPANSION_LIMIT 照原样留的、`${…}`），不是 ShellWord 的普通 str 一律当为真。"""
    unquoted_braces = True

    def __new__(cls, text, unquoted_braces):
        instance = super().__new__(cls, text)
        instance.unquoted_braces = unquoted_braces
        return instance

    def __reduce__(self):   # 拷贝、深拷贝、pickle 照带着这一格（不写的话 str 子类的 __new__ 少一个参数，copy.deepcopy 抛 TypeError）
        return (ShellWord, (str(self), self.unquoted_braces))


def quoted_braces_as_unknown():
    """弄坏开关 LIB_SHELL_WORDS_BREAK=quoted-braces-as-unknown：切好的词不带引号信息，引号里的花括号也当「还留着、认不准」（只给自检证红用）。"""
    return words_break_is_set("quoted-braces-as-unknown")


def brace_expand(word, mask):
    """一个词的花括号展开：mask 与 word 等长，'u' 是引号外的字符、'q' 是引号里、转义过或替换正文里的（不参与展开），'e' 是引号写出的空串占的位置。
    交回展开出的词表（没有可展开的交 [word]），'e' 那几个位置去掉。展开出来的空串照 bash 去掉（`{cargo,} test` 是 `cargo test`、`{,}` 整个没了），
    这一份里有引号写出的空串的留着（`''{,}` 是两个空词、`{"",}` 是一个）；弄坏开关 LIB_SHELL_WORDS_BREAK=brace-empties-kept 下空串都留着。
    交回的每个词是 ShellWord：照掩码记下它还留着的花括号里有没有引号外的（函数调用认不认得准用它：function_never_called）。"""
    def without_quoted_empties(text, text_mask):
        return ShellWord("".join(character for character, kind in zip(text, text_mask) if kind != "e"),
                         quoted_braces_as_unknown() or any(character in "{}" and kind == "u" for character, kind in zip(text, text_mask)))
    if "{" not in word or words_break_is_set("braces-unexpanded") or ASSIGNMENT.match(word) or re.match(r"^[A-Za-z_][A-Za-z0-9_]*\+=", word):
        return [without_quoted_empties(word, mask)]
    results = brace_expand_once(word, mask, BRACE_EXPANSION_LIMIT)
    if not results or results == [(word, mask)]:
        return [without_quoted_empties(word, mask)]
    expanded = [(without_quoted_empties(text, text_mask), "e" in text_mask) for text, text_mask in results]
    return [text for text, quoted_empty in expanded if text or quoted_empty or words_break_is_set("brace-empties-kept")]


def brace_expand_once(word, mask, limit):
    """展开 word 里第一处成立的花括号（引号外的 `{`，与它配对的引号外的 `}` 之间顶层有引号外的逗号、或是 `a..b` 序列；`${` 起的跳过），
    再把每个结果接着展开；交回 [(词, 掩码)]，超过 limit 个交 None。"""
    index = 0
    while index < len(word):
        if word[index] == "{" and mask[index] == "u" and not (index > 0 and word[index - 1] == "$" and mask[index - 1] == "u"):
            depth, commas, cursor = 1, [], index + 1
            while cursor < len(word) and depth:
                if mask[cursor] == "u":
                    if word[cursor] == "{":
                        depth += 1
                    elif word[cursor] == "}":
                        depth -= 1
                    elif word[cursor] == "," and depth == 1:
                        commas.append(cursor)
                cursor += 1
            if depth == 0:
                close = cursor - 1
                prefix, suffix = (word[:index], mask[:index]), (word[close + 1:], mask[close + 1:])
                alternatives = None
                if commas:
                    bounds = [index] + commas + [close]
                    alternatives = [(word[bounds[i] + 1:bounds[i + 1]], mask[bounds[i] + 1:bounds[i + 1]]) for i in range(len(bounds) - 1)]
                else:
                    alternatives = brace_sequence(word[index + 1:close], mask[index + 1:close])
                if alternatives is not None:
                    if len(alternatives) > limit:
                        return None
                    results = []
                    for alternative, alternative_mask in alternatives:
                        expanded = brace_expand_once(prefix[0] + alternative + suffix[0], prefix[1] + alternative_mask + suffix[1], limit)
                        if expanded is None or len(results) + len(expanded) > limit:
                            return None
                        results += expanded
                    return results
        elif word[index] == "{" and mask[index] == "u":
            closing = word.find("}", index)   # `${…}` 是变量，不是花括号展开：整段跳过
            index = len(word) if closing < 0 else closing
        index += 1
    return [(word, mask)]


def brace_sequence_width(first, last):
    """bash 的整数序列补零：起或止有一边写成 0 起头的两位以上（或 -0 起头的三位以上）时，每一份补零到起、止两边写法的长度里大的那个
    （`{01..03}`、`{-01..2}`；另一边没补零而写得更长的照它，`{01..100..3}` 补到三位，同 bash 5.2 的 expand_seqterm）；不补零交 0。
    这里的 brace_sequence 与 lib_project_code_runs.py 的 brace_sequence 共用这一份。弄坏开关 LIB_SHELL_WORDS_BREAK=brace-sequence-unpadded 下一律交 0；
    brace-sequence-width-padded-ends-only 下只取补零写的那几边的长度。"""
    if words_break_is_set("brace-sequence-unpadded"):
        return 0
    padded = [len(text) for text in (first, last) if (len(text) > 1 and text[0] == "0") or (len(text) > 2 and text[:2] == "-0")]
    if not padded:
        return 0
    return max(padded) if words_break_is_set("brace-sequence-width-padded-ends-only") else max(len(first), len(last))


def brace_sequence(inner, inner_mask):
    """`{1..3}`、`{a..c}`、`{1..10..2}` 的展开（全部在引号外才算；整数照 bash 补零：brace_sequence_width）：交回 [(词, 掩码)]；不是序列交 None。"""
    match = BRACE_SEQUENCE.match(inner) if "q" not in inner_mask else None
    if match is None:
        return None
    first, last, step = match.group(1), match.group(2), abs(int(match.group(3) or 1)) or 1
    if first.lstrip("-").isdigit() != last.lstrip("-").isdigit():
        return None
    numeric = first.lstrip("-").isdigit()
    low, high = (int(first), int(last)) if numeric else (ord(first), ord(last))
    if abs(high - low) // step + 1 > BRACE_EXPANSION_LIMIT:
        return [None] * (BRACE_EXPANSION_LIMIT + 1)
    values = range(low, high + (1 if high >= low else -1), step if high >= low else -step)
    width = brace_sequence_width(first, last) if numeric else 0
    texts = [(f"{value:0{width}d}" if width else str(value)) if numeric else chr(value) for value in values]
    return [(text, "u" * len(text)) for text in texts]


def simple_commands(tokens, separators=None, boundaries=None):
    """按命令边界切成一条条简单命令（只留词，重定向连同目标去掉），另交整段里输出重定向的目标，
    每条简单命令里的命令替换正文，以及每条简单命令上的重定向 ((运算符, 目标词), …)（后两样与命令一一对应；只有重定向、没有词的那一条，词是空列表）。
    separators 是一个列表时，按命令的次序追加每一条前面的那个分隔符（第一条是 None；`|` 之后换行的，下一条照旧记 `|`），调用方拿它认管道后一段。
    boundaries 是一个列表时，按命令的次序追加每一条与前一条之间的全部分隔符（元组，按出现次序；括号、换行都在里面），调用方拿它认 cd 失败之后哪几条不跑。"""
    commands, substitutions, current, current_substitutions, output_targets = [], [], [], [], []
    redirections, current_redirections = [], []
    preceding = None
    between = []   # 上一条之后到这里的分隔符
    position = 0
    while position < len(tokens):
        is_operator, text = tokens[position]
        if (is_operator is False and ARRAY_ASSIGNMENT_START.match(text) and position + 1 < len(tokens)
                and tokens[position + 1] == (True, "(")):
            # 数组赋值 `名字=(元素 …)`：元素是值，不是命令；整段收成一个赋值词
            elements, position = [], position + 2
            while position < len(tokens) and tokens[position] != (True, ")"):
                if tokens[position][0] is None:
                    current_substitutions.append(tokens[position][1])
                elif tokens[position][0] is False:
                    elements.append(tokens[position][1])
                position += 1
            current.append(f"{text}({' '.join(elements)})")
            position += 1
            continue
        if is_operator is None:
            current_substitutions.append(text)
        elif is_operator and text in COMMAND_SEPARATORS:
            if current or current_substitutions:
                commands.append(current)
                substitutions.append(current_substitutions)
                redirections.append(current_redirections)
                if separators is not None:
                    separators.append(preceding)
                if boundaries is not None:
                    boundaries.append(tuple(between))
                between = []
                preceding = text
            elif not (text == "\n" and preceding in ("|", "|&")):
                preceding = text
            between.append(text)
            current, current_substitutions, current_redirections = [], [], []
        elif is_operator:
            if position + 1 < len(tokens) and tokens[position + 1][0] is False:
                if text in OUTPUT_REDIRECTS:
                    output_targets.append(tokens[position + 1][1])
                current_redirections.append((text, tokens[position + 1][1]))
                position += 1
        else:
            current.append(text)
        position += 1
    if current or current_substitutions:
        commands.append(current)
        substitutions.append(current_substitutions)
        redirections.append(current_redirections)
        if separators is not None:
            separators.append(preceding)
        if boundaries is not None:
            boundaries.append(tuple(between))
    return commands, output_targets, substitutions, redirections


def words_break_is_set(item):
    """弄坏开关 LIB_SHELL_WORDS_BREAK 里有没有这一项（几个用逗号连；只给自检证红用）。"""
    return item in os.environ.get("LIB_SHELL_WORDS_BREAK", "").split(",")


def env_chdir_ignored():
    """弄坏开关 LIB_SHELL_WORDS_BREAK=env-chdir-ignored：env 的 -C / --chdir 换的目录不跟（按外面的当前目录判，只给自检证红用）。"""
    return "env-chdir-ignored" in os.environ.get("LIB_SHELL_WORDS_BREAK", "").split(",")


def env_directory(directory, target, variables=None):
    """env -C 的目标解开成绝对路径：相对路径按命令开始时的当前目录。目标里的变量照 expand_known_variables 展开（variables 是这段命令文本里前面赋过的）；
    展开不了（没赋过的变量、命令替换）或当前目录认不出时交 directory 原样（照旧按外面的当前目录判，
    不像 cd 那样交 None：交 None 会让这一条命令的目录类判定全部放过，比交 directory 原样还松；罩不住的：env -C 的目录写在展开不了的变量里）。"""
    expanded = os.path.expanduser(target)
    if "$" in expanded or "`" in expanded:
        expanded = expand_known_variables(expanded, variables or {})
        if expanded is None:
            return directory
    return resolve_path(directory, expanded) or directory


class EnvOption(NamedTuple):
    width: int                     # 这个选项连同它的值词占几个词
    chdir: str | None = None       # -C / --chdir 的目录
    split_string: str | None = None   # -S / --split-string 的字符串


ENV_SHORT_OPTIONS_WITH_VALUE = "uCS"                                # env 的 -u NAME、-C 目录、-S 字符串；短选项可以合写（-iC 目录、-iS 字符串、-0u NAME），值贴着写或另起一个词
ENV_LONG_OPTIONS_WITH_VALUE = ("chdir", "unset", "split-string")    # getopt_long 认唯一前缀的缩写（--chd 目录、--spl=字符串、--u NAME）；env 别的长选项没有值词


def env_option_clusters_unparsed():
    """弄坏开关 LIB_SHELL_WORDS_BREAK=env-option-clusters-unparsed：env 的选项退回只认整词 -u / --unset / -C / --chdir（合写的 -iC、缩写的 --chd、单独的 - 与 -- 当成不带值的开关或命令词；只给自检证红用）。"""
    return "env-option-clusters-unparsed" in os.environ.get("LIB_SHELL_WORDS_BREAK", "").split(",")


def env_split_string_unexpanded():
    """弄坏开关 LIB_SHELL_WORDS_BREAK=env-split-string-unexpanded：env 的 -S / --split-string 不拆（-S 当不带值的开关，后面那个词整个当命令词；只给自检证红用）。"""
    return "env-split-string-unexpanded" in os.environ.get("LIB_SHELL_WORDS_BREAK", "").split(",")


def env_option(words, position):
    """env 的第 position 个词若是选项（含合写的短选项簇）交 EnvOption，不是（赋值、命令词）交 None。
    单独一个 `-` 等于 -i，`--` 之后不再有选项（由调用方停手）。带值的：-u、-C、-S（短选项簇里头一个带值的字母之后，贴着的是值、没有就取下一个词）和
    --unset / --chdir / --split-string（`=值` 或下一个词，认唯一前缀的缩写）；别的字母与长选项（-i、-0、-v、--block-signal[=SIG]……）不带值词。"""
    argument = words[position]
    if argument in ("-", "--"):
        return EnvOption(1)
    if argument.startswith("--"):
        name, equals, value = argument[2:].partition("=")
        matched = next((long for long in ENV_LONG_OPTIONS_WITH_VALUE if name and long.startswith(name)), None)
        if matched is None:
            return EnvOption(1)
        width = 1
        if not equals:
            value, width = (words[position + 1], 2) if position + 1 < len(words) else (None, 1)
        return EnvOption(width, value if matched == "chdir" else None, value if matched == "split-string" else None)
    if not argument.startswith("-"):
        return None
    letters = argument[1:]
    for index, letter in enumerate(letters):
        if letter in ENV_SHORT_OPTIONS_WITH_VALUE:
            value, width = letters[index + 1:], 1
            if not value:
                value, width = (words[position + 1], 2) if position + 1 < len(words) else (None, 1)
            return EnvOption(width, value if letter == "C" else None, value if letter == "S" else None)
    return EnvOption(1)


def env_option_clears_environment(argument):
    """env 的这个选项词清不清掉整个环境：单独的 `-`、`--ignore-environment`（认唯一前缀的缩写 `--i`、`--ign`……）、短选项簇里在头一个带值的字母之前有 i（`-i`、`-iC 目录`、`-0i`）。
    弄坏开关 LIB_SHELL_WORDS_BREAK=env-ignore-environment-kept 下调用方不认（env -i 之前的赋值照旧算在后面那条命令上）。"""
    if argument == "-":
        return True
    if argument.startswith("--"):
        name = argument[2:].partition("=")[0]
        return bool(name) and "ignore-environment".startswith(name) and not any(long.startswith(name) for long in ENV_LONG_OPTIONS_WITH_VALUE)
    if not argument.startswith("-"):
        return False
    for letter in argument[1:]:
        if letter == "i":
            return True
        if letter in ENV_SHORT_OPTIONS_WITH_VALUE:
            return False
    return False


ENV_SPLIT_ESCAPES = {"n": "\n", "t": "\t", "f": "\f", "r": "\r", "v": "\v"}


def env_split_string(text):
    """GNU env -S 的拆词：空白（空格、制表、换行、\\v、\\f、\\r）与不在引号里的 `\\_` 隔开词；单引号里只认 `\\'` 与 `\\\\`；双引号里认 \\n \\t \\f \\r \\v \\_（空格）与 \\" \\$ \\\\；
    不在引号里的词首 `#` 起到结尾是注释；`\\c` 到此为止；`$NAME`、`${NAME}` 不展开（照原文留着）；不认的反斜杠转义（env 自己会报错不跑）宽松地留下那个字符。"""
    words, current, started, quote, index = [], [], False, None, 0
    while index < len(text):
        character = text[index]
        if quote == "'":
            if character == "'":
                quote = None
            elif character == "\\" and text[index + 1:index + 2] in ("'", "\\"):
                index += 1
                current.append(text[index])
            else:
                current.append(character)
        elif character == "\\":
            index += 1
            following = text[index:index + 1]
            if following == "c":
                break
            if following == "_" and quote is None:
                if started:
                    words.append("".join(current))
                current, started = [], False
            elif following == "_":
                current.append(" ")
            elif following in ENV_SPLIT_ESCAPES:
                current.append(ENV_SPLIT_ESCAPES[following])
                started = True
            elif following:
                current.append(following)
                started = True
            else:
                current.append("\\")
                started = True
        elif quote == '"':
            if character == '"':
                quote = None
            else:
                current.append(character)
        elif character in "'\"":
            quote, started = character, True
        elif character in " \t\n\v\f\r":
            if started:
                words.append("".join(current))
            current, started = [], False
        elif character == "#" and not started:
            break
        else:
            current.append(character)
            started = True
        index += 1
    if started:
        words.append("".join(current))
    return words


def skip_prefixes_expanding(words):
    """skip_prefixes，env 的 -S / --split-string 先按 env 的规则拆成词、原地换进命令里再判（拆出来的词里还可以有选项、赋值与命令词，MAXIMUM_NESTING 层为限）；
    交回 (PrefixSkip, 换完之后的词表)：命令词的下标指的是换完之后的词表。（弄坏开关 LIB_SHELL_WORDS_BREAK=env-split-string-unexpanded 下不拆。）"""
    for _ in range(MAXIMUM_NESTING):
        skipped = skip_prefixes(words, expand_split=True)
        if skipped.split is None:
            return skipped, words
        start, end, text = skipped.split
        words = words[:start] + env_split_string(text) + words[end:]
    return skip_prefixes(words), words


def skip_prefixes(words, expand_split=False):
    """跳过关键字、赋值与 nohup / setsid / nice / timeout / env / taskset 这类前缀，交回命令词的下标、路过的前缀名与赋值；
    env 的 `-C 目录`、`-C目录`、`--chdir=目录`、`--chdir 目录` 换的目录（最后一个，GNU env 取最后一个）交在 chdir 里，由调用方按当前目录解开
    （弄坏开关 LIB_SHELL_WORDS_BREAK=env-chdir-ignored 下不交）。env 的选项照 env_option 切（合写的 -iC、缩写、单独的 -）；-S / --split-string 的字符串：
    expand_split 为真时交在 split 里（PrefixSkip.split，不往下判，由 skip_prefixes_expanding 拆开换进词表重来），为假时只跳过它与它的值词。"""
    prefix_names, assignments, chdir, cleared, login = set(), {}, None, False, False
    position = 0
    while position < len(words):
        word = words[position]
        name = os.path.basename(word)
        assignment = ASSIGNMENT.match(word)
        if word in COMMAND_KEYWORDS:
            position += 1
            if word == "time" and not words_break_is_set("time-options-as-command"):
                # bash 的 time 关键字只认一个 -p，之后可以再跟一个 --（`time -p -p x` 的第二个 -p 是命令词）
                if words[position:position + 1] == ["-p"]:
                    position += 1
                if words[position:position + 1] == ["--"]:
                    position += 1
            continue
        if word == "builtin" and not words_break_is_set("builtin-as-command"):
            position += 1   # `builtin source 脚本`、`builtin cd 目录`：builtin 不是命令词，后面那个内建才是（`builtin -- 名字` 同）
            if words[position:position + 1] == ["--"]:
                position += 1
            following = words[position] if position < len(words) else None
            if (following is not None and following not in BASH_BUILTINS and not set("$`*?[") & set(following)
                    and not words_break_is_set("builtin-non-builtin-run")):
                return PrefixSkip(None, prefix_names, assignments, chdir, None, cleared)   # `builtin cargo …`：bash 报 not a shell builtin，什么都不跑
            continue
        if word == "coproc" and not words_break_is_set("coproc-as-command"):
            # bash 的 coproc 关键字：`coproc 命令 …` 把后面那条放进协进程照样跑，`coproc 名字 { …; }` 的名字也不是命令词（
            # 弄坏开关 coproc-as-command 下照旧把 coproc 当命令词）
            position += 1
            prefix_names.add("coproc")   # 记进路过的前缀名：bash-command-detector.sh 的 ③ 按它认把活放出追踪
            if (position + 1 < len(words) and re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", words[position]) and words[position + 1] == "{"):
                position += 1
            continue
        if name == "busybox" and not renamed_shells_unseen():
            # `busybox sh / ash / bash / dash … applet`：busybox 这一层剥掉，后面那个 applet 当命令词（它的 -c 代码、脚本照 shell 判）
            following = words[position + 1] if position + 1 < len(words) else None
            if following is not None and os.path.basename(following) in BUSYBOX_SHELL_APPLETS:
                prefix_names.add("busybox")
                position += 1
                continue
        if word == "function" and not words_break_is_set("function-keyword-as-command"):
            position += 2   # `function 名字 { …; }`、`function 名字() …`：关键字与函数名都不是命令词，函数体照命令判（同 `名字() { …; }`）
            continue
        if assignment:
            assignments[assignment.group(1)] = assignment.group(2)
            position += 1
            continue
        if (APPEND_ASSIGNMENT.match(word) or SUBSCRIPTED_ASSIGNMENT.match(word)) and not words_break_is_set("extended-assignments-as-command"):
            position += 1   # `名字+=值`、`名字+=(…)`、`名字[下标]=值`：bash 照赋值认，不是命令词（记下的值由 record_array_writes 管；弄坏开关下照旧当命令词）
            continue
        if name not in PREFIX_OPTIONS_WITH_VALUE:
            return PrefixSkip(position, prefix_names, assignments, chdir, None, cleared, login)
        if name == "command" and words[position + 1:position + 2] in (["-v"], ["-V"]):
            return PrefixSkip(position, prefix_names, assignments, chdir, None, cleared)  # command -v / -V 是查名字，不执行后面那个词
        prefix_names.add(name)
        position += 1
        positional_left = PREFIX_POSITIONAL_COUNT.get(name, 0)
        options_done, env_assigned = False, False
        while position < len(words):
            argument = words[position]
            if name == "env" and env_assigned and "=" not in argument and not words_break_is_set("env-options-after-assignment"):
                break   # GNU env 认到第一个 `名字=值` 就不再认选项：其后头一个不带 `=` 的词是命令词，`-u`、`-i`、`--` 也是（env 报 No such file、退 127）
            if name == "env" and not options_done and not env_assigned and not env_option_clusters_unparsed():
                option = env_option(words, position)
                if option is not None:
                    if env_option_clears_environment(argument) and not words_break_is_set("env-ignore-environment-kept"):
                        cleared = True          # env -i：之前的赋值（命令前写的 `BASH_ENV=x env -i …`）与外面带进来的都不给后面那条命令
                        assignments.clear()
                    if option.split_string is not None and not env_split_string_unexpanded():
                        if expand_split:
                            return PrefixSkip(None, prefix_names, assignments, chdir, (position, position + option.width, option.split_string), cleared)
                    elif option.chdir is not None and not env_chdir_ignored():
                        chdir = option.chdir
                    options_done = argument == "--"
                    position += option.width
                    continue
            if name == "env" and not env_chdir_ignored():
                if argument in ("-C", "--chdir") and position + 1 < len(words):
                    chdir = words[position + 1]
                elif argument.startswith("--chdir="):
                    chdir = argument[len("--chdir="):]
                elif argument.startswith("-C") and len(argument) > 2:
                    chdir = argument[2:]
            if name == "exec" and not words_break_is_set("login-profiles-unread") and (
                    (argument == "-a" and words[position + 1:position + 2] and words[position + 1].startswith("-"))
                    or (argument.startswith("-") and not argument.startswith("--") and "l" in argument[1:] and argument != "-a")):
                login = True   # `exec -l bash`、`exec -a -bash bash`：argv[0] 以 - 起头，bash / sh 照登录 shell 起
            if argument in PREFIX_OPTIONS_WITH_VALUE[name]:
                position += 2
            elif argument.startswith("-") and argument != "-":
                position += 1
            elif name == "env" and ASSIGNMENT.match(argument):
                environment_assignment = ASSIGNMENT.match(argument)
                assignments[environment_assignment.group(1)] = environment_assignment.group(2)
                position += 1
                env_assigned = True
            elif (name == "env" and "=" in argument[1 if words_break_is_set("env-leading-equals-as-command") else 0:]
                  and not words_break_is_set("env-loose-assignments-as-command")):
                position += 1   # GNU env 把带 `=` 的词都当 NAME=VALUE 设进环境（`env -- a-b=1 命令`、开头就是 `=` 的 `env =1 命令`），都不是命令词
                env_assigned = True
            elif positional_left:
                positional_left -= 1
                position += 1
            else:
                break
    return PrefixSkip(None, prefix_names, assignments, chdir, None, cleared)


def arguments_as_executed():
    """弄坏开关 LIB_SHELL_WORDS_BREAK=arguments-as-executed：`bash -n 脚本` 与 `python3 - 词…` 里的参数照旧当被执行的脚本（只给自检证红用）。"""
    return "arguments-as-executed" in os.environ.get("LIB_SHELL_WORDS_BREAK", "").split(",")


def shell_invocation(arguments):
    """bash / sh 这类后面的参数：是 -c 的一段代码、是起一个脚本，还是都不是（只查语法、从标准输入读）。
    -c 之后照 bash 接着认选项，选项完了（`--`、单独的 `-` 之后，或头一个不是选项的词）的那个词是代码（`bash -c -- '…'`、`bash -c -x '…'`；
    弄坏开关 LIB_SHELL_WORDS_BREAK=shell-c-options-unparsed 下取紧跟 -c 的那个词）；带 -s 的从标准输入读，后面的词是位置参数、不是脚本
    （`bash -s -- 词`；弄坏开关 LIB_SHELL_WORDS_BREAK=shell-s-positionals-as-script 下照旧当脚本）。
    bash 的长选项只认写在短选项前面的：短选项（`-c`、`-x`、`-o 值` 这类）之后再写 `--名字`（单独的 `--` 除外），bash 报 `--: invalid option`、什么都不跑
    （`bash -c --posix '…'`、`bash -x --posix 脚本`；弄坏开关 shell-long-options-after-short 下照旧当选项跳过）；
    带 -n（`-n`、`-cn`、`-xn`，`+n` 撤回）只查语法，-c 的代码、脚本、标准输入都不执行（弄坏开关 shell-syntax-only-code-run 下 -c 的代码照判）。
    这两种交 runs_nothing。"""
    option_count, only_checks_syntax, takes_code, reads_standard_input, short_option_seen = 0, False, False, False, False
    option_values_read = not words_break_is_set("shell-option-values-as-script")
    while option_count < len(arguments):
        argument = arguments[option_count]
        if argument in ("-", "--") and option_values_read:
            option_count += 1   # `bash -- 脚本`、`bash - 脚本`：选项到此为止，下一个词是脚本（后面没有词的从标准输入读）
            break
        if not argument.startswith(("-", "+")) or argument in ("-", "--"):
            break
        if argument.startswith("--") and short_option_seen and not words_break_is_set("shell-long-options-after-short"):
            return ShellInvocation(None, None, True)   # 短选项之后的长选项：bash 报 --: invalid option，什么都不跑
        option_count += 1
        short_option_seen = short_option_seen or not argument.startswith("--")
        if argument in ("-o", "+o", "-O", "+O") or (argument in SHELL_LONG_OPTIONS_WITH_VALUE and option_values_read):
            option_count += 1
            continue
        letters = "" if argument.startswith("--") else argument[1:]
        if (option_values_read and not words_break_is_set("shell-option-clusters-unvalued")
                and argument.startswith("-") and argument not in ("-o", "+o", "-O", "+O")):
            # 合写的短选项簇里每个 -o / -O 各吃后面一个词（`bash -euo pipefail -c …` 的 pipefail、`bash -oo pipefail errexit …` 两个词）：
            # 不吃掉的话 pipefail 被当成 -c 的代码或脚本，整条 -c 的代码没进判
            option_count += letters.count("o") + letters.count("O")
        if "n" in letters:
            only_checks_syntax = argument.startswith("-") and not arguments_as_executed()
        reads_standard_input = reads_standard_input or (argument.startswith("-") and "s" in letters)
        if "c" in letters:
            if words_break_is_set("shell-c-options-unparsed"):
                return ShellInvocation(arguments[option_count] if option_count < len(arguments) else "", None)
            takes_code = True
    if takes_code:
        if only_checks_syntax and not words_break_is_set("shell-syntax-only-code-run"):
            return ShellInvocation(None, None, True)
        return ShellInvocation(arguments[option_count] if option_count < len(arguments) else "", None)
    if only_checks_syntax:
        return ShellInvocation(None, None, True)
    if option_count >= len(arguments) or (reads_standard_input and not words_break_is_set("shell-s-positionals-as-script")):
        return ShellInvocation(None, None)
    return ShellInvocation(None, option_count)


def python_options_by_dash():
    """弄坏开关 LIB_SHELL_WORDS_BREAK=python-options-by-dash：python 的选项只按「以 - 开头」认，带值选项的值被当成脚本名（只给自检证红用）。"""
    return "python-options-by-dash" in os.environ.get("LIB_SHELL_WORDS_BREAK", "").split(",")


def python_invocation(arguments):
    """python 解释器后面的参数：是 -c 的一段代码、是 -m 的一个模块、是起一个脚本文件，还是都不是（从标准输入读、只有选项）。
    短选项逐个字母看（可以合写）：头一个要值的字母（PYTHON_SHORT_OPTIONS_WITH_VALUE）之后剩下的是它的值，没剩下就取下一个词；
    `--` 之后的第一个词是脚本。"""
    position = 0
    if python_options_by_dash():
        while position < len(arguments) and arguments[position].startswith("-"):
            if arguments[position] == "-" and not arguments_as_executed():
                return PythonInvocation(None, None, None)
            position += 1
        return PythonInvocation(None, None, position if position < len(arguments) else None)
    while position < len(arguments):
        argument = arguments[position]
        if argument == "-":
            if arguments_as_executed():
                position += 1
                continue
            return PythonInvocation(None, None, None)   # `python3 -`：程序从标准输入读，后面的词都是它的参数，没有被执行的脚本
        if argument == "--":
            position += 1
            break
        if not argument.startswith("-"):
            break
        if argument.startswith("--"):
            position += 2 if argument in PYTHON_LONG_OPTIONS_WITH_VALUE else 1
            continue
        letters = argument[1:]
        value_letter = next((index for index, letter in enumerate(letters) if letter in PYTHON_SHORT_OPTIONS_WITH_VALUE), None)
        if value_letter is None:
            position += 1
            continue
        attached = letters[value_letter + 1:]
        value = attached if attached else (arguments[position + 1] if position + 1 < len(arguments) else "")
        position += 1 if attached else 2
        if letters[value_letter] == "c":
            return PythonInvocation(value, None, min(position, len(arguments)))
        if letters[value_letter] == "m":
            if value == "runpy" and position < len(arguments) and not arguments[position].startswith("-"):
                value, position = arguments[position], position + 1   # `python3 -m runpy <模块>`：runpy 再按名字起后面那个模块
            return PythonInvocation(None, value, min(position, len(arguments)))
    return PythonInvocation(None, None, position if position < len(arguments) else None)


def resolve_path(directory, path):
    """相对路径按 directory 解成绝对路径；directory 认不出（None）时交 None。"""
    if os.path.isabs(path):
        return single_leading_slash(os.path.normpath(path))
    if directory is None:
        return None
    return single_leading_slash(os.path.normpath(os.path.join(directory, path)))


def single_leading_slash(path):
    """os.path.normpath 照 POSIX 留着开头恰好两个斜杠（`//tmp/x`），Linux 上它与 `/tmp/x` 是同一个文件：并成一个，免得与记下的路径对不上。"""
    if path.startswith("//") and not words_break_is_set("double-slash-kept"):
        return "/" + path.lstrip("/")
    return path


def link_source_word(arguments, source, link):
    """`ln` 的参数里 source 拷到 link（链接的路径，照写的样子）时，按当前目录解开就指到链接真读的那个文件的写法：
    符号链接（-s / --symbolic，不带 -r / --relative）里存的相对路径从链接所在的目录算（`ln -s ../s.sh sub/l.sh` 读的是 sub/../s.sh），交回 链接所在目录/源；
    硬链接、带 -r、绝对路径、`~` 起头、带 `$` 的照原样交回。弄坏开关 LIB_SHELL_WORDS_BREAK=link-source-from-current-directory 下一律照原样（源按当前目录解）。"""
    options = []
    for argument in arguments:
        if argument == "--":
            break
        if argument.startswith("-") and argument != "-":
            options.append(argument)
    symbolic = any(option == "--symbolic" or (not option.startswith("--") and "s" in option[1:]) for option in options)
    relative = any(option == "--relative" or (not option.startswith("--") and "r" in option[1:]) for option in options)
    if (not symbolic or relative or source.startswith(("/", "~")) or "$" in source
            or words_break_is_set("link-source-from-current-directory")):
        return source
    return os.path.join(os.path.dirname(link), source)


# 从标准输入读命令的 shell（shell_reads_standard_input）收到的输入：here-string `<<<` 与管道前一段是 printf / echo / cat 喂进来的字面文本当 shell 代码再切一遍；
# 认不出是什么的（带变量、命令替换，管道前一段是别的命令或读文件，标准输入从文件来）交一句判不了的说明
STANDARD_INPUT_REDIRECTS = {"<", "<<", "<<-", "<<<", "<>", "<&"}   # 改标准输入的重定向，最后一个说了算
PIPE_SEPARATORS = ("|", "|&")
BACKSLASH_ESCAPES = {"n": "\n", "t": "\t", "\\": "\\", "a": "\a", "b": "\b", "f": "\f", "r": "\r", "v": "\v", "e": "\x1b", "E": "\x1b", '"': '"', "'": "'"}
PRINTF_CONVERSION = re.compile(r"%[-+ #0]*\d*(?:\.\d*)?([diouxXfFeEgGaAcsbq%])")
ECHO_OPTION = re.compile(r"^-[neE]+$")
UNJUDGED_SNIPPET_CHARACTERS = 120


class StandardInputCode(NamedTuple):
    code: str | None       # 当 shell 代码再切的文本；没有可切的是 None
    unjudged: str | None   # 判不了的说明（喂进去的不是字面）；判得了是 None


def shell_standard_input_unread():
    """弄坏开关 LIB_SHELL_WORDS_BREAK=shell-standard-input-unread：here-string 与管道喂给 shell 的文本不当命令再切、也不交判不了的说明（只给自检证红用）。"""
    return "shell-standard-input-unread" in os.environ.get("LIB_SHELL_WORDS_BREAK", "").split(",")


def backslash_escapes(text, octal_needs_zero):
    """printf 的格式串（octal_needs_zero 为假：\\NNN）与 %b 的值、echo -e 的参数（为真：\\0NNN）里的反斜杠转义；\\c 到此为止，认不出的照原样留下。"""
    pieces, index = [], 0
    while index < len(text):
        character = text[index]
        if character != "\\" or index + 1 >= len(text):
            pieces.append(character)
            index += 1
            continue
        following = text[index + 1]
        if following == "c":
            break
        if following in BACKSLASH_ESCAPES:
            pieces.append(BACKSLASH_ESCAPES[following])
            index += 2
        elif following == "x" and re.match(r"[0-9A-Fa-f]{1,2}", text[index + 2:]):
            digits = re.match(r"[0-9A-Fa-f]{1,2}", text[index + 2:]).group(0)
            pieces.append(chr(int(digits, 16)))
            index += 2 + len(digits)
        elif following in "01234567" and (following == "0" or not octal_needs_zero):
            start = index + 2 if octal_needs_zero else index + 1
            digits = re.match(r"[0-7]{0,3}", text[start:]).group(0)
            pieces.append(chr(int(digits or "0", 8) & 0xFF))
            index = start + len(digits)
        else:
            pieces.append(character)
            index += 1
    return "".join(pieces)


def printf_output(arguments):
    """printf 的参数交给标准输出的文本：格式串里的转义照 printf 换，%… 依次取参数（%b 的参数再换一遍转义），参数没用完就把格式串再套一遍；
    `-v 名字` 写进变量、不往标准输出写，交空串。"""
    if arguments[:1] == ["--"]:
        arguments = arguments[1:]
    if not arguments or arguments[0].startswith("-v"):
        return ""
    form, values, pieces = backslash_escapes(arguments[0], False), list(arguments[1:]), []
    while True:
        used = 0
        def convert(match):
            nonlocal used
            if match.group(1) == "%":
                return "%"
            value = values[used] if used < len(values) else ""
            used += 1
            return backslash_escapes(value, True) if match.group(1) == "b" else value
        pieces.append(PRINTF_CONVERSION.sub(convert, form))
        values = values[used:]
        if not values or used == 0:
            return "".join(pieces)


def echo_output(arguments):
    """echo 的参数交给标准输出的文本：开头的 -n / -e / -E（可以合写）照 bash 的内建 echo 认，-e 时换转义，不带 -n 末尾补换行。"""
    interpret, newline, position = False, True, 0
    while position < len(arguments) and ECHO_OPTION.match(arguments[position]):
        for letter in arguments[position][1:]:
            newline = newline and letter != "n"
            interpret = (interpret or letter == "e") and letter != "E"
        position += 1
    text = " ".join(arguments[position:])
    return (backslash_escapes(text, True) if interpret else text) + ("\n" if newline else "")


def has_expansion(texts):
    """这些词（引号已去掉）里有没有 `$` 或反引号：外层 shell 先展开它们，展开之后是什么这里算不出（单引号里写的同样算，切词时分不出）。"""
    return any("$" in text or "`" in text for text in texts)


def unjudged_snippet(text):
    text = " ".join(text.split())
    return text if len(text) <= UNJUDGED_SNIPPET_CHARACTERS else text[:UNJUDGED_SNIPPET_CHARACTERS] + "…"


def piped_literal_output(words, heredocs):
    """管道前一段交给后一段的文本：printf、echo 照它们的参数现算，不带文件操作数的 cat 交它自己读到的 heredoc 正文（最后一段）。
    交 StandardInputCode：参数里有 `$` 或反引号的照算、另交判不了的说明；别的命令、cat 读文件或读更前一段的输出，只交判不了的说明。"""
    skipped, words = skip_prefixes_expanding(words)
    if skipped.command_position is None:
        return StandardInputCode(None, "管道前一段不是一条简单命令（花括号组、子 shell 这类），它交出来的输出算不出")
    name, arguments = os.path.basename(words[skipped.command_position]), words[skipped.command_position + 1:]
    shown = unjudged_snippet(" ".join(words[skipped.command_position:]))
    if name in ("printf", "echo"):
        text = printf_output(arguments) if name == "printf" else echo_output(arguments)
        unjudged = f"管道前一段 `{shown}` 的参数里有变量或命令替换，展开之后喂进去的是什么判不了" if has_expansion(arguments) else None
        return StandardInputCode(text, unjudged)
    if name == "cat" and all(argument.startswith("-") for argument in arguments if argument != "-") and heredocs:
        body = heredocs[-1]
        unjudged = None if body.is_delimiter_quoted or not has_expansion([body.text]) else \
            "管道前一段 cat 读的 heredoc 结束词没带引号、正文里有变量或命令替换，展开之后喂进去的是什么判不了"
        return StandardInputCode(body.text, unjudged)
    return StandardInputCode(None, f"管道前一段是 `{shown}`，它交出来的输出这里算不出")


def shell_standard_input_files(redirections, piped_from):
    """从标准输入读命令的 shell 读的是不是盘上的文件：改标准输入的最后一个是 `< 文件` / `<> 文件`（/dev/null 除外）交 [文件]；
    没改标准输入、管道前一段是只带文件参数的 cat（不带选项、不读 heredoc）交那几份文件（照次序接起来喂进去）；别的交 []（here-string、printf / echo 的字面照 shell_standard_input 切）。
    `bash -s < 文件`、`bash < 文件`、`sh -s < 文件`、`cat 文件 | bash` 交出文件，调用方读正文判。
    弄坏开关 LIB_SHELL_WORDS_BREAK=shell-standard-input-file-unread 下一律交 []：这几种只记判不了、放行。"""
    if words_break_is_set("shell-standard-input-file-unread"):
        return []
    stdin_redirections = [(operator, target) for operator, target in redirections if operator in STANDARD_INPUT_REDIRECTS]
    if stdin_redirections:
        operator, target = stdin_redirections[-1]
        return [target] if operator in ("<", "<>") and target != "/dev/null" else []
    if piped_from is None:
        return []
    words, heredocs = piped_from
    skipped, words = skip_prefixes_expanding(words)
    if skipped.command_position is None or heredocs:
        return []
    name, arguments = os.path.basename(words[skipped.command_position]), words[skipped.command_position + 1:]
    if name != "cat" or not arguments or any(argument.startswith("-") for argument in arguments):
        return []
    return list(arguments)


def shell_standard_input(redirections, piped_from):
    """从标准输入读命令的 shell 收到的输入：redirections 是它那一条简单命令上的重定向（((运算符, 目标词), …)，改标准输入的最后一个说了算），
    piped_from 是管道前一段（(词, 喂给它的 heredoc 正文)；不在管道后面是 None）。交 StandardInputCode：
    here-string 交它的文字（末尾补换行）；管道照 piped_literal_output；标准输入从文件或别的描述符来的（< 文件、<& 号、<> 文件）只交判不了的说明（< /dev/null 不算）；
    heredoc（<<、<<-）不归这里（喂给 shell 的正文已留在原文里当后续的行切），交两个 None。"""
    stdin_redirections = [(operator, target) for operator, target in redirections if operator in STANDARD_INPUT_REDIRECTS]
    if stdin_redirections:
        operator, target = stdin_redirections[-1]
        if operator == "<<<":
            unjudged = f"here-string `{unjudged_snippet(target)}` 里有变量或命令替换，展开之后喂进去的是什么判不了" if has_expansion([target]) else None
            return StandardInputCode(target + "\n", unjudged)
        if operator in ("<<", "<<-") or target == "/dev/null":
            return StandardInputCode(None, None)
        return StandardInputCode(None, f"标准输入从 `{operator} {unjudged_snippet(target)}` 来，读进去的是什么这里判不了")
    if piped_from is None:
        return StandardInputCode(None, None)
    return piped_literal_output(*piped_from)


AMBIENT_STDIN_OPERATORS = {"<", "<>"}   # exec / 复合命令上把标准输入从一份文件下发的重定向（fd 0 或没写 fd）


class CompoundStdinFeeds(NamedTuple):
    files: tuple           # 下发给里层 shell 的 stdin 文件词（按调用方的目录解；算得出的）
    has_bare_shell: bool   # 这一层里有没有裸的读标准输入的 shell（不带 -c、不带脚本、自己没有 < 的）
    undeterminable: bool   # 有没有算不出来源的 stdin 文件（写成变量 / 命令替换）


def compound_redirected_stdin_files(text):
    """exec < 文件、{ … } < 文件、( … ) < 文件、while / until / for … done < 文件：这几种把标准输入从一份文件下发给里层读标准输入的 bash / sh。
    交 CompoundStdinFeeds。没有裸 shell 时 files 为空（`while read … done < 文件`、`{ echo; } < 文件` 这类不下发）；下发的文件照 `bash 文件` 由调用方读正文判。"""
    tokens = shell_tokens(strip_data_heredocs(text))
    simple, _targets, _substitutions, redirections = simple_commands(tokens)
    ambient, has_bare_shell = [], False
    for words, redirs in zip(simple, redirections):
        skipped = skip_prefixes(words)
        if skipped.command_position is not None:
            rest = words[skipped.command_position + 1:]
            if is_shell_name(os.path.basename(words[skipped.command_position])) and shell_reads_standard_input(rest):
                if not any(operator in STANDARD_INPUT_REDIRECTS for operator, _ in redirs):
                    has_bare_shell = True
        exec_only = skipped.command_position is None and "exec" in skipped.prefix_names
        group_close = bool(words) and words[-1] in ("}", "done")
        if exec_only or group_close:
            ambient += [target for operator, target in redirs if operator in AMBIENT_STDIN_OPERATORS and redirection_fd(operator) in (None, 0)]
    # ( … ) < 文件：) 后面的 < 文件被 simple_commands 当没有词的尾命令丢掉，另扫一遍 token
    for index in range(len(tokens) - 2):
        if (tokens[index] == (True, ")") and tokens[index + 1][0] is True and tokens[index + 1][1] in AMBIENT_STDIN_OPERATORS
                and redirection_fd(tokens[index + 1][1]) in (None, 0) and tokens[index + 2][0] is False):
            ambient.append(tokens[index + 2][1])
    if not has_bare_shell:
        return CompoundStdinFeeds((), False, False)
    files, undeterminable = [], False
    for target in ambient:
        if target == "/dev/null":
            continue
        if has_expansion([target]):
            undeterminable = True
        else:
            files.append(target)
    return CompoundStdinFeeds(tuple(files), True, undeterminable)


# 目录词里认的变量引用：`$名字`、`${名字}`；别的写法（`${名字:-默认}`、`$1`、`$$`、`$(…)`）不认，留着的 `$` 算展开不了
VARIABLE_REFERENCE = re.compile(r"\$(?:\{([A-Za-z_][A-Za-z0-9_]*)\}|([A-Za-z_][A-Za-z0-9_]*))")
# 改写变量、改成什么这里算不出的命令：for / select 的循环变量、read 读进来的（unset 在 commands_at_command_position 里另记）
REBINDING_COMMANDS = {"for", "select", "read", "mapfile", "readarray"}
APPEND_ASSIGNMENT = re.compile(r"^([A-Za-z_][A-Za-z0-9_]*)\+=(.*)$", re.S)
ELEMENT_ASSIGNMENT = re.compile(r"^([A-Za-z_][A-Za-z0-9_]*)\[[^\]]*\]\+?=(.*)$", re.S)
SUBSCRIPTED_ASSIGNMENT = re.compile(r"^[A-Za-z_][A-Za-z0-9_]*\[.*\]\+?=", re.S)   # 下标里还可以套展开（`a[${b[$k]}]+=x`）：skip_prefixes 认它是赋值词用
# 声明命令：参数里的 `名字=值` 照赋值记
DECLARATION_COMMANDS = {"local", "declare", "typeset", "readonly"}


def directory_variables_unexpanded():
    """弄坏开关 LIB_SHELL_WORDS_BREAK=directory-variables-unexpanded：cd / pushd / env -C 的目录带 `$` 一律展开不了（只给自检证红用）。"""
    return "directory-variables-unexpanded" in os.environ.get("LIB_SHELL_WORDS_BREAK", "").split(",")


def variable_rebinding_unseen():
    """弄坏开关 LIB_SHELL_WORDS_BREAK=variable-rebinding-unseen：for / select / read / unset 改写过的变量照旧用之前记下的值（只给自检证红用）。"""
    return "variable-rebinding-unseen" in os.environ.get("LIB_SHELL_WORDS_BREAK", "").split(",")


def expand_known_variables(word, variables):
    """词里的 `$名字`、`${名字}` 换成值：先查 variables（这段命令文本里前面赋过的，值已经展开；值是 None 的是改写过、算不出的），
    查不到的只认 HOME（取钩子进程自己的环境：钩子与它判的命令是同一个用户起的）。有一处展开不了（没赋过的变量、算不出的值、别的写法、命令替换、反引号）交 None。
    弄坏开关 LIB_SHELL_WORDS_BREAK=directory-variables-unexpanded 下带 `$` 的一律交 None。"""
    if "`" in word:
        return None
    if "$" not in word:
        return word
    if directory_variables_unexpanded():
        return None
    unexpanded = []
    def replace(match):
        name = match.group(1) or match.group(2)
        value = variables[name] if name in variables else (os.environ.get("HOME") if name == "HOME" else None)
        if value is None:
            unexpanded.append(name)
            return match.group(0)
        return value
    expanded = VARIABLE_REFERENCE.sub(replace, word)
    return None if unexpanded or "$" in expanded else expanded


def assigned_value(value, variables):
    """`名字=值` 赋进去的是什么：开头的 `~` 展开，变量照 expand_known_variables 展开；展开不了交 None（记成算不出）。"""
    return expand_known_variables(os.path.expanduser(value), variables)


def rebind_variables(name, arguments, variables):
    """会改写变量的命令记进 variables：local / declare 这类声明里的 `名字=值` 照赋值记；for / select 的循环变量、read 读进来的几个名字记成 None（算不出）。
    弄坏开关 LIB_SHELL_WORDS_BREAK=variable-rebinding-unseen 下只记声明里的赋值。"""
    if name in DECLARATION_COMMANDS:
        for argument in arguments:
            assignment = ASSIGNMENT.match(argument)
            if assignment:
                variables[assignment.group(1)] = assigned_value(assignment.group(2), variables)
        return
    if variable_rebinding_unseen():
        return
    names = arguments[:1] if name in ("for", "select") else [argument for argument in arguments if not argument.startswith("-")]
    for argument in names:
        if re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", argument):
            variables[argument] = None


def array_appends_unseen():
    """弄坏开关 LIB_SHELL_WORDS_BREAK=array-appends-unseen：`名字+=值`、`名字[下标]=值` 不记（只给自检证红用）。"""
    return "array-appends-unseen" in os.environ.get("LIB_SHELL_WORDS_BREAK", "").split(",")


def record_array_writes(words, variables):
    """一条简单命令只有关键字与赋值（没有命令词）时，把里面的 `名字+=值`（数组接数组、字符串接字符串）与 `名字[下标]=值`（记成算不出）记进 variables；
    `名字=值` 由调用方照旧记。有命令词的（`名字+=值 命令`）不记。弄坏开关 array-appends-unseen 下什么都不记。"""
    if array_appends_unseen():
        return
    writes = []
    for word in words:
        if word in COMMAND_KEYWORDS or ASSIGNMENT.match(word):
            continue
        append, element = APPEND_ASSIGNMENT.match(word), ELEMENT_ASSIGNMENT.match(word)
        if append is None and element is None:
            return
        writes.append((append, element))
    for append, element in writes:
        if element is not None:
            variables[element.group(1)] = None
            continue
        name = append.group(1)
        before, added = variables.get(name), assigned_value(append.group(2), variables)
        if before is None or added is None:
            variables[name] = None
        elif before.startswith("(") and before.endswith(")") and added.startswith("(") and added.endswith(")"):
            variables[name] = "(" + " ".join(part for part in (before[1:-1].strip(), added[1:-1].strip()) if part) + ")"
        elif not before.startswith("(") and not added.startswith("("):
            variables[name] = before + added
        else:
            variables[name] = None


GIT_TOPLEVEL_SUBSTITUTION = re.compile(r"^(?:\$\(\s*git\s+rev-parse\s+--show-toplevel\s*\)|`\s*git\s+rev-parse\s+--show-toplevel\s*`)")
PWD_SUBSTITUTION = re.compile(r"^(?:\$\(\s*pwd\s*\)|`\s*pwd\s*`)")
GIT_LOCATION_VARIABLES = ("GIT_DIR", "GIT_WORK_TREE", "GIT_CEILING_DIRECTORIES", "GIT_DISCOVERY_ACROSS_FILESYSTEM", "GIT_COMMON_DIR")


def cd_known_targets_unexpanded():
    """弄坏开关 LIB_SHELL_WORDS_BREAK=cd-known-targets-unexpanded：cd 的目标里 $PWD、$OLDPWD、`cd -`、`$(git rev-parse --show-toplevel)`、`$(pwd)` 照旧算不出。"""
    return words_break_is_set("cd-known-targets-unexpanded")


def cd_failure_guards_unseen():
    """弄坏开关 LIB_SHELL_WORDS_BREAK=cd-failure-guards-unseen：cd 失败之后哪几条不跑一概不认（CommandAtPosition.cd_failure_skips 都是 False）。"""
    return words_break_is_set("cd-failure-guards-unseen")


def git_toplevel(directory):
    """在 directory 里跑 `git rev-parse --show-toplevel` 打出什么：从它（解开符号链接）往上找头一个 .git（带 HEAD 的目录，或以 `gitdir: ` 起头的文件），交那一层目录；
    一直找到 / 都没有交 ""（git 报错、标准输出空）；directory 是 None、盘上不在、找到的 .git 认不准（目录里没有 HEAD、文件不是 gitdir: 起头、那一层不归这个用户，git 会拒）交 None。"""
    if directory is None or not os.path.isdir(directory):
        return None
    current = os.path.realpath(directory)
    while True:
        marker = os.path.join(current, ".git")
        if os.path.lexists(marker):
            try:
                if os.stat(current).st_uid != os.getuid():
                    return None
                if os.path.isdir(marker):
                    return current if os.path.isfile(os.path.join(marker, "HEAD")) else None
                with open(marker, encoding="utf-8", errors="replace") as handle:
                    return current if handle.read(8) == "gitdir: " else None
            except OSError:
                return None
        parent = os.path.dirname(current)
        if parent == current:
            return ""
        current = parent


def changed_directory(directory, arguments, variables=None, previous_directory=None):
    """cd / pushd 之后的当前目录。目标里的变量照 expand_known_variables 展开（variables 是这段命令文本里前面赋过的；不给时只认 HOME）；
    $PWD 是 directory、$OLDPWD 与 `-` 是 previous_directory（上一处 cd 之前的目录，None 是算不出），起头的 `$(git rev-parse --show-toplevel)` 照 git_toplevel 算、`$(pwd)` 是 directory
    （弄坏开关 LIB_SHELL_WORDS_BREAK=cd-known-targets-unexpanded 下这几样照旧算不出）；cd 到空串（git 不在仓里时）bash 5.2 不换目录。
    展开不了的变量或别的命令替换认不出，交 None。"""
    targets = [argument for argument in arguments if not argument.startswith("-") or argument == "-"]
    if not targets:
        return os.path.expanduser("~")
    target = os.path.expanduser(targets[0])
    known = dict(variables or {})
    if not cd_known_targets_unexpanded():
        if target == "-":
            return previous_directory
        if directory is not None:
            known.setdefault("PWD", directory)
        if previous_directory is not None:
            known.setdefault("OLDPWD", previous_directory)
        leading = GIT_TOPLEVEL_SUBSTITUTION.match(target) or PWD_SUBSTITUTION.match(target)
        if leading is not None:
            if leading.re is GIT_TOPLEVEL_SUBSTITUTION:
                replacement = None if any(name in known for name in GIT_LOCATION_VARIABLES) else git_toplevel(directory)
            else:
                replacement = directory
            if replacement is None:
                return None
            target = replacement + target[leading.end():]
            if target == "":
                return directory
    if target == "-":
        return None
    target = expand_known_variables(target, known)
    if target is None:
        return None
    return resolve_path(directory, target)


SUBSHELL_PREFIX_KEYWORDS = frozenset({"time", "-p", "--", "!"})   # 子 shell 前面只隔着这几个关键字（`time ( … )`、`! ( … )`）的，照紧跟在 `&&` 后面算


def guarded_subshell_boundary(boundary, previous_words, guards, cd_and_chain, cd_failure_poisoned, cd_seen):
    """commands_at_command_position 里一条简单命令前面的分隔符 boundary 先过这一道：`cd 目录 && ( … )` 的子 shell 在 cd 失败时整个不跑，组里的命令照 `&&` 守着算
    （`cd 不存在 && …` 因 cd 失败不跑，不当拒）。
    guards 是 {"depth": 括号层数, "groups": [(开组之前的层数, 组外的 cd_failure_poisoned, 组外的 cd_seen), …]}，跨命令带着走。
    先收掉 boundary 开头合上这样一组的右括号：组外接着算在 `&&` 一串里（cd_and_chain 回到真），组里 cd 过的照旧不守后面的（组里的 cd 这里当对后面也生效）；
    再认这一处是不是开了这样一组：`&&` 一串还在、右括号都收掉了，剩下的是 `&&` 后面紧跟左括号，或前一条只有 time / ! 这类关键字（SUBSHELL_PREFIX_KEYWORDS）、这里紧跟左括号。
    开了组的交 (()，组里从头记 `&&` 一串与 cd)；收了组的交右括号后面那几个分隔符（换行去掉），由调用方照原来那两条判据接着判；两样都没有的原样交回 boundary。
    交 (留给原来那两条判据的分隔符, cd_and_chain, cd_failure_poisoned, cd_seen)。"""
    separators = [separator for separator in boundary if separator != "\n"]
    position, kept_closings, changed = 0, [], False
    while position < len(separators) and separators[position] == ")":
        guards["depth"] -= 1
        if guards["groups"] and guards["depth"] == guards["groups"][-1][0]:
            _, outer_poisoned, outer_cd_seen = guards["groups"].pop()
            cd_failure_poisoned = outer_poisoned or cd_failure_poisoned or cd_seen
            cd_seen = outer_cd_seen or cd_seen
            cd_and_chain, changed = True, True
        else:
            kept_closings.append(")")
        position += 1
    rest = separators[position:]
    after_keywords = bool(previous_words) and set(previous_words) <= SUBSHELL_PREFIX_KEYWORDS and previous_words[0] in ("time", "!")
    opens = cd_and_chain and not kept_closings and (
        (len(rest) > 1 and rest[0] == "&&" and set(rest[1:]) == {"("}) or (bool(rest) and set(rest) == {"("} and after_keywords))
    if opens:
        guards["groups"].append((guards["depth"], cd_failure_poisoned, cd_seen))
        guards["depth"] += rest.count("(")
        return (), False, cd_failure_poisoned, False
    guards["depth"] += rest.count("(") - rest.count(")")
    return (tuple(kept_closings + rest) if changed else boundary), cd_and_chain, cd_failure_poisoned, cd_seen


def cd_failure_exits(simple, boundaries, position):
    """simple[position] 是不是一条单独的 `exit [n]`、跑完之后这一层就结束（后面跟的是 `;`、换行或这段文本的末尾，不在管道、后台、括号里）：
    `cd 目录 || exit` 的 cd 失败了，后面同一层的都不跑（`return` 在顶层不退出、照跑，不算）。"""
    if position >= len(simple) or not simple[position] or simple[position][0] != "exit" or len(simple[position]) > 2:
        return False
    after = boundaries[position + 1] if position + 1 < len(boundaries) else ()
    return all(separator in (";", "\n") for separator in after)


FUNCTION_CALLING_WORDS = frozenset({"eval", "source", ".", "trap", "compgen", "declare", "typeset", "set", "alias", "enable"})   # 不点名字也调得到函数、或让子进程看得见函数的写法
IMPLICITLY_CALLED_FUNCTIONS = frozenset({"command_not_found_handle"})   # bash 自己会调的函数名：命令找不到时调它


def drop_uncalled_function_bodies(tokens, text, stripped_heredocs):
    """切好的词里只定义、不会被调到的 `function 名字 { …; }`（`function 名字() { …; }` 同）连同函数体去掉：bash 只定义不跑函数体。
    只在这几样都成立时去掉，否则照留（函数体照命令判，偏严）：函数体是一个花括号组、里面没有另一个 `{`（到头一个 `}` 为止）；名字是字面的、
    不是 bash 自己会调的（IMPLICITLY_CALLED_FUNCTIONS）；整段命令文本 text 里这个名字只出现这一次（照词的边界数，heredoc 正文、引号里的都算），
    函数体之外切好的词里也没有一个是它（拆开拼写的 `"fo"o`、`fo{o,}` 解开之后就是它；还留着成对花括号的词认不准，照调到了算：function_never_called）；
    函数体之外没有命令替换、变量、反引号、通配，没有 FUNCTION_CALLING_WORDS 里的词（eval、source、set -a 这类），剥掉的 heredoc 正文里没有 `$` 与反引号。
    不带 function 关键字的 `名字() { …; }`（命令位置上的名字紧跟一对空的圆括号）照同一套判据（复跑分类报告 d137-replay-classify 二节末 legA-r2 零星；
    弄坏开关 LIB_SHELL_WORDS_BREAK=posix-uncalled-functions-run 下只认 function 关键字）。
    弄坏开关 LIB_SHELL_WORDS_BREAK=uncalled-functions-run 下照留。"""
    posix_definitions = not words_break_is_set("posix-uncalled-functions-run")
    if words_break_is_set("uncalled-functions-run") or ((False, "function") not in tokens and not (posix_definitions and (True, "(") in tokens)):
        return tokens
    at_command_position, index = True, 0
    while index < len(tokens):
        kind, value = tokens[index]
        definition = None   # (函数名, 函数体从哪个下标起)
        if kind is False and at_command_position and value == "function" and index + 1 < len(tokens) and tokens[index + 1][0] is False:
            definition = (tokens[index + 1][1], index + 2)
            if tokens[definition[1]:definition[1] + 2] == [(True, "("), (True, ")")]:
                definition = (definition[0], definition[1] + 2)
        elif (posix_definitions and kind is False and at_command_position and value not in COMMAND_KEYWORDS and "=" not in value
              and tokens[index + 1:index + 3] == [(True, "("), (True, ")")]):
            definition = (value, index + 3)
        if definition is not None:
            name, body = definition
            while tokens[body:body + 1] == [(True, "\n")]:
                body += 1
            closing = next((position for position in range(body + 1, len(tokens)) if tokens[position] in ((False, "{"), (False, "}"))), None)
            if (tokens[body:body + 1] == [(False, "{")] and closing is not None and tokens[closing] == (False, "}")
                    and function_never_called(name, tokens[:index] + tokens[closing + 1:], text, stripped_heredocs)):
                tokens = tokens[:index] + tokens[closing + 1:]
                continue
        if kind is True:
            at_command_position = value in COMMAND_SEPARATORS
        elif kind is False:
            at_command_position = at_command_position and value in COMMAND_KEYWORDS
        index += 1
    return tokens


def function_never_called(name, outside, text, stripped_heredocs):
    """drop_uncalled_function_bodies 的判据：函数体之外的词 outside 与整段命令文本 text 里，这个名字确定调不到。
    名字在原文里照字面只出现定义那一次；函数体之外切好的词（引号、反斜杠、反斜杠续行、花括号展开都已照 bash 解开）里有一个正是这个名字的算调到了
    （`"fo"o`、`fo{o,}`、`fo\\o` 这类拆开拼写的调用）；词里还留着成对的花括号（超过 BRACE_EXPANSION_LIMIT 照原样留的、`{}`）认不准展开成什么，照调到了算（偏严）。
    弄坏开关 LIB_SHELL_WORDS_BREAK=function-calls-literal-only 下只数原文里的字面（拆开拼写的调用认不出）。"""
    if not name or set("$`*?[\\{}") & set(name) or name in IMPLICITLY_CALLED_FUNCTIONS:
        return False
    if len(re.findall(r"(?<![A-Za-z0-9_])" + re.escape(name) + r"(?![A-Za-z0-9_])", text)) != 1:
        return False
    literal_only = words_break_is_set("function-calls-literal-only")
    for kind, value in outside:
        if kind is None or (kind is False and (set("$`*?[") & set(value) or os.path.basename(value) in FUNCTION_CALLING_WORDS)):
            return False
        # 还留着的花括号都在引号里（ShellWord.unquoted_braces 为假，`'echo a{,}'`）的是数据，不会再展开成这个名字（修补报告 debt-tw-shell-words-fix2 第 95 行那一条；
        # 弄坏开关 quoted-braces-as-unknown 下引号里的也当认不准）
        if kind is False and not literal_only and (value == name or ("{" in value and "}" in value and getattr(value, "unquoted_braces", True))):
            return False
    return not any("$" in heredoc.text or "`" in heredoc.text for heredoc in stripped_heredocs)


POSITIONAL_COMMAND_WORD = re.compile(r"^\$(?:[@*0-9]|\{[@*0-9]+\})")   # 命令词是位置参数：`"$@"`、`$1`、`${@}`（引号已去掉）


def launcher_functions(text):
    """text 里定义的、把调用时的参数当命令跑的函数名（`run() { "$@"; }`、`w() { nice "$@"; }`、`run_logged() { local l=$1; shift; "$@" > "$l"; }`）：
    函数体（`名字() { …; }` 或 `function 名字 [()] { …; }`，到配对的 `}` 为止）里有一条命令的命令词是位置参数（POSITIONAL_COMMAND_WORD，前缀、包装照剥）。
    调用方遇到命令词是这里的名字的，把调用的参数一段段当命令再判（例：仓副本里经脚本自己的 run_logged 起 验证包）。
    认不出函数体的（子 shell 当函数体、花括号没配上）不算。弄坏开关 LIB_SHELL_WORDS_BREAK=function-launchers-unseen 下交空集（调用照普通命令判）。"""
    if words_break_is_set("function-launchers-unseen"):
        return frozenset()
    names = set()
    for name, body in function_bodies(text).items():
        if any(POSITIONAL_COMMAND_WORD.match(command.words[0]) for command in commands_at_command_position(body).commands):
            names.add(name)
    return frozenset(names)


def function_bodies(text):
    """text 里定义的每个函数名 → 函数体文本（`名字() { … }` 或 `function 名字 [()] { … }`，到配对的 `}` 为止；认不出函数体的——子 shell 当函数体、花括号没配上——不收）。
    把函数体里切好的词重新用空格连起来、引号去掉，照一段命令文本再切（launcher_functions、shell_starting_functions 共用这一份）。"""
    if "(" not in text and "function" not in text:
        return {}
    tokens = shell_tokens(strip_data_heredocs(text))
    found, index, at_command_position = {}, 0, True
    while index < len(tokens):
        kind, value = tokens[index]
        name, body = None, None
        if kind is False and at_command_position:
            if value == "function" and index + 1 < len(tokens) and tokens[index + 1][0] is False:
                name, body = tokens[index + 1][1], index + 2
                if tokens[body:body + 2] == [(True, "("), (True, ")")]:
                    body += 2
            elif tokens[index + 1:index + 3] == [(True, "("), (True, ")")]:
                name, body = value, index + 3
        if name is not None:
            while tokens[body:body + 1] == [(True, "\n")]:
                body += 1
            if tokens[body:body + 1] == [(False, "{")]:
                depth, cursor = 0, body
                while cursor < len(tokens):
                    if tokens[cursor] == (False, "{"):
                        depth += 1
                    elif tokens[cursor] == (False, "}"):
                        depth -= 1
                        if depth == 0:
                            break
                    cursor += 1
                pieces = []
                for piece_kind, piece in tokens[body + 1:cursor]:
                    if piece_kind is True:
                        pieces.append(piece if piece == "\n" else f" {piece} ")
                    elif piece_kind is False:
                        pieces.append(" " + shlex.quote(piece))
                found[name] = "".join(pieces)
        if kind is True:
            at_command_position = value in COMMAND_SEPARATORS
        elif kind is False:
            at_command_position = at_command_position and value in COMMAND_KEYWORDS
        index += 1
    return found


def function_body_starts_shell(body):
    """函数体文本里有没有一条命令起一个会读启动文件的 shell：命令词剥前缀后 basename 是 is_shell_name，且不是只查语法 / 只打印版本（shell_invocation.runs_nothing 为假）。"""
    for words in simple_commands(shell_tokens(strip_data_heredocs(body)))[0]:
        skipped = skip_prefixes(words)
        if skipped.command_position is None:
            continue
        name = os.path.basename(words[skipped.command_position])
        if is_shell_name(name) and not shell_invocation(words[skipped.command_position + 1:]).runs_nothing:
            return True
    return False


def shell_starting_functions(text):
    """text 里函数体会起一个读启动文件的 shell 的函数名 → 函数体文本（`f(){ bash -c true; }; BASH_ENV=脚本 f` 的函数体就地按词法环境判，看不到调用处前缀设的 BASH_ENV / ENV；
    调用方拿调用处的环境重判这份函数体，补上里层 shell 的启动文件）。"""
    return {name: body for name, body in function_bodies(text).items() if function_body_starts_shell(body)}
# shell 启动时先读的文件、当脚本跑（认不出的一律拒，偏严）：
# bash 非交互读 BASH_ENV（posix 模式读 ENV）；交互（-i）不读 BASH_ENV，读 --rcfile / --init-file 指的文件，没给读 ~/.bashrc（--norc 不读）；
# sh、dash、ksh 读 ENV；zsh 读 ZDOTDIR（没设取 HOME）下的 .zshenv、.zshrc（-f、--no-rcs 不读）。启动文件的内容照 `bash 脚本` 那样由调用方读正文判。
STARTUP_FILE_BASH_VARIABLE = "BASH_ENV"
STARTUP_FILE_POSIX_VARIABLE = "ENV"
POSIX_STARTUP_SHELL_NAMES = {"sh", "dash", "ksh", "ash"}
SHELL_INFORMATION_OPTIONS = {"--version", "--help"}   # 打印完就退出，不读启动文件
BASH_INTERACTIVE_STARTUP_FILE = ".bashrc"
ZSH_STARTUP_FILES = (".zshenv", ".zshrc")
# 登录 shell 与系统级的启动文件；SYSTEM_STARTUP_DIRECTORY 是 /etc，自证换成样本目录（不读本机的 /etc）
SYSTEM_STARTUP_DIRECTORY = "/etc"
BASH_LOGIN_STARTUP_FILES = (".bash_profile", ".bash_login", ".profile")   # bash 只读头一份在的；这里三份都当读（偏严）
BASH_LOGOUT_FILE = ".bash_logout"
POSIX_LOGIN_STARTUP_FILE = ".profile"
SYSTEM_PROFILE = "profile"
SYSTEM_BASH_INTERACTIVE_FILE = "bash.bashrc"
SYSTEM_BASH_LOGOUT_FILE = "bash.bash_logout"
ZSH_SYSTEM_ALWAYS_FILE = "zsh/zshenv"
ZSH_SYSTEM_STARTUP_FILES = ("zsh/zshrc",)
ZSH_SYSTEM_LOGIN_FILES = ("zsh/zprofile", "zsh/zlogin")
ZSH_LOGIN_STARTUP_FILES = (".zprofile", ".zlogin")
# 直接执行的脚本，#! 指到这些的不是 shell，不读 BASH_ENV / ENV 之类的启动文件；指到别的（不是 shell 也不在这张表里）的认不出（executed_script_startup）
NON_SHELL_INTERPRETER = re.compile(r"python[0-9.]*|pypy[0-9]*|perl[0-9.]*|ruby[0-9.]*|node|nodejs|deno|bun|php[0-9.]*|lua[0-9.]*|luajit|g?awk|mawk|nawk|sed"
                                   r"|tclsh[0-9.]*|wish|expect|Rscript|cargo|rustc|go|julia|runghc|runhaskell|ocaml|guile|racket|java|groovy|osascript|cat|true|false")


def startup_files_unread():
    """弄坏开关 LIB_SHELL_WORDS_BREAK=startup-files-unread：shell 启动时先读的文件一个都不当被起的脚本交出（只给自检证红用）。"""
    return words_break_is_set("startup-files-unread")


def bash_in_posix_mode(arguments, environment):
    """bash 按 posix 模式启动：脚本名或代码之前的选项里有 `--posix`、`-o posix`（`-oposix` 同），或环境里设了 POSIXLY_CORRECT、SHELLOPTS 带 posix。"""
    if "POSIXLY_CORRECT" in environment or "posix" in environment.get("SHELLOPTS", "").split(":"):
        return True
    position = 0
    while position < len(arguments):
        argument = arguments[position]
        if argument == "--" or not argument.startswith(("-", "+")):
            return False
        if argument in ("--posix", "-oposix") or (argument == "-o" and arguments[position + 1:position + 2] == ["posix"]):
            return True
        position += 2 if argument in ("-o", "+o", "-O", "+O") or argument in SHELL_LONG_OPTIONS_WITH_VALUE else 1
    return False


class ShellStartOptions(NamedTuple):
    interactive: bool       # 带 -i（合写的短选项里有 i 也算）
    no_rc: bool             # bash 的 --norc，zsh 的 -f、--no-rcs
    rcfile: str | None      # bash 的 --rcfile / --init-file 给的路径（最后一个），没给是 None
    login: bool = False     # 带 -l / --login（合写的短选项里有 l 也算）：登录 shell
    no_profile: bool = False   # bash 的 --noprofile：登录 shell 也不读 /etc/profile 与 HOME 下的 profile


def shell_start_options(name, arguments):
    """bash / zsh 命令行上脚本名或代码之前的选项里，与启动文件有关的几样（走法同 shell_invocation：选项到第一个不是选项的词或 `--` 为止，-o / -O 带一个值）。"""
    interactive = no_rc = login = no_profile = False
    rcfile = None
    position = 0
    while position < len(arguments):
        argument = arguments[position]
        if argument in ("-", "--") or not argument.startswith(("-", "+")):
            break
        position += 1
        if argument in ("-o", "+o", "-O", "+O"):
            position += 1
        elif argument in SHELL_LONG_OPTIONS_WITH_VALUE:
            rcfile = arguments[position] if position < len(arguments) else rcfile
            position += 1
        elif argument.startswith("--"):
            no_rc = no_rc or argument in ("--norc", "--no-rcs")
            login = login or argument == "--login"
            no_profile = no_profile or argument == "--noprofile"
        elif argument.startswith("-"):
            interactive = interactive or "i" in argument[1:]
            login = login or "l" in argument[1:]
            no_rc = no_rc or (name == "zsh" and "f" in argument[1:])
    return ShellStartOptions(interactive, no_rc, rcfile, login, no_profile)


def startup_file_words(name, arguments, environment, login_by_name=False):
    """这个 shell 启动时会先读哪几个文件，每个一个路径词（照写的样子，`~` 与变量由 startup_file_commands 展开）。
    bash：非交互读 BASH_ENV（posix 模式读 ENV）；交互读 --rcfile / --init-file 给的、没给读 HOME 下的 .bashrc（--norc 不读；posix 模式读 ENV，不读 .bashrc）。
    sh、dash、ksh：读 ENV（实测非交互的 sh 其实不读，只有 -i 才读，这里照偏严算）。zsh：ZDOTDIR（没设取 HOME）下的 .zshenv、.zshrc（-f、--no-rcs 不读；
    .zshrc 其实只有交互才读，照偏严都读）。别的 shell、只打印版本或帮助的不读。
    弄坏开关 LIB_SHELL_WORDS_BREAK=startup-files-unread 下一个都不交；interactive-startup-files-unread 下 -i 不另看（交互的 bash 照旧读 BASH_ENV、不读 .bashrc 与 --rcfile）；
    zsh-startup-files-unread 下 zsh 不读。
    登录 shell（-l、--login，或 login_by_name：`exec -l`、`exec -a -名字` 起的）另读 /etc/profile 与 HOME 下的 profile 一类，交互的 bash 另读 /etc/bash.bashrc，zsh 另读 /etc/zsh/ 下的
    （文件头「登录 shell」一段）。弄坏开关 login-profiles-unread 下登录的那几份不交，system-startup-files-unread 下 SYSTEM_STARTUP_DIRECTORY 下的不交。"""
    if startup_files_unread() or (arguments and arguments[0] in SHELL_INFORMATION_OPTIONS):
        return []
    options = shell_start_options(name, arguments)
    home = environment.get("HOME") or os.path.expanduser("~")
    login = (options.login or login_by_name) and not words_break_is_set("login-profiles-unread")
    system = [] if words_break_is_set("system-startup-files-unread") else [SYSTEM_STARTUP_DIRECTORY]
    if name == "zsh":
        zsh_directory = environment.get("ZDOTDIR") or home
        found = [f"{directory}/{ZSH_SYSTEM_ALWAYS_FILE}" for directory in system]   # /etc/zsh/zshenv 连 -f 也读
        if words_break_is_set("zsh-startup-files-unread") or options.no_rc:
            return found
        found += [f"{zsh_directory}/{ZSH_STARTUP_FILES[0]}"]
        if login:
            found += [f"{directory}/{ZSH_SYSTEM_LOGIN_FILES[0]}" for directory in system] + [f"{zsh_directory}/{ZSH_LOGIN_STARTUP_FILES[0]}"]
        found += [f"{directory}/{startup_name}" for directory in system for startup_name in ZSH_SYSTEM_STARTUP_FILES] + [f"{zsh_directory}/{ZSH_STARTUP_FILES[1]}"]
        if login:
            found += [f"{directory}/{ZSH_SYSTEM_LOGIN_FILES[1]}" for directory in system] + [f"{zsh_directory}/{ZSH_LOGIN_STARTUP_FILES[1]}"]
        return found
    posix = bash_in_posix_mode(arguments, environment) if name == "bash" else name in POSIX_STARTUP_SHELL_NAMES
    found = []
    if login and not options.no_profile:
        found += [f"{directory}/{SYSTEM_PROFILE}" for directory in system]
        found += [f"{home}/{POSIX_LOGIN_STARTUP_FILE}"] if posix else [f"{home}/{startup_name}" for startup_name in BASH_LOGIN_STARTUP_FILES]
    if name == "bash" and not posix:
        if not options.interactive or words_break_is_set("interactive-startup-files-unread"):
            value = environment.get(STARTUP_FILE_BASH_VARIABLE)
            found += [value] if value else []
        elif not options.no_rc:
            found += [f"{directory}/{SYSTEM_BASH_INTERACTIVE_FILE}" for directory in system] + [options.rcfile or f"{home}/{BASH_INTERACTIVE_STARTUP_FILE}"]
        if login and not options.no_profile:
            found += [f"{home}/{BASH_LOGOUT_FILE}"] + [f"{directory}/{SYSTEM_BASH_LOGOUT_FILE}" for directory in system]
        return found
    value = environment.get(STARTUP_FILE_POSIX_VARIABLE) if posix else None
    return found + ([value] if value else [])


def startup_file_commands(name, arguments, environment, variables, directory, memory_capped, wrappers, login_by_name=False, interactive=False):
    """shell（name）启动时先读的文件（startup_file_words），每个当一条 launcher 为 "startup" 的命令交出（词是文件路径，按这个 shell 的当前目录解；调用方同 `bash 脚本` 那样读正文判）。
    路径里的 `~` 与这段命令文本里记下的变量先展开；展开不了的（命令替换、没赋过的变量）原样交，路径带着 `$` 或反引号，调用方照「认不出」办。
    interactive 或 arguments 里带 -i 的，交出的命令记成交互的（CommandAtPosition.interactive）。"""
    found = []
    interactive = interactive or shell_start_options(name, arguments).interactive
    for value in startup_file_words(name, arguments, environment, login_by_name):
        expanded = expand_known_variables(expand_directory_tilde(value, directory) or value, variables) if expand_directory_tilde(value, directory) is not None else None
        found.append(CommandAtPosition([value if expanded is None else expanded], directory, environment, "startup",
                                       memory_capped=memory_capped, variables=dict(variables), wrappers=wrappers, interactive=interactive))
    return found


NON_INTERACTIVE_RETURN_GUARDS = (
    re.compile(r'case[ \t]+"?\$-"?[ \t]+in\s+\*i\*\)\s*;;\s*\*\)\s*return(?:[ \t]+[0-9]+)?\s*;;\s*esac\b'),
    re.compile(r'\[\[[ \t]+"?\$-"?[ \t]+!=[ \t]+\*i\*[ \t]+\]\][ \t]*&&[ \t]*return\b'),
    re.compile(r'\[\[[ \t]+"?\$-"?[ \t]+==?[ \t]+\*i\*[ \t]+\]\][ \t]*\|\|[ \t]*return\b'),
)
BLOCK_OPENING_WORDS = frozenset({"if", "then", "else", "elif", "fi", "case", "esac", "for", "while", "until", "select", "do", "done", "{", "}", "function", "coproc", "!"})


def non_interactive_return_unseen():
    """弄坏开关 LIB_SHELL_WORDS_BREAK=non-interactive-return-unseen：source / 启动文件里非交互就 return 的守卫不认，整份照读。"""
    return words_break_is_set("non-interactive-return-unseen")


def non_interactive_return_offset(text):
    """非交互的 shell source 这份文本（启动文件、`.` / source 读进来的）时，读到哪里就 return 了：交那一处守卫在 text 里的起点，后面的不会跑；认不出交 None。
    认的守卫：行首的 `case $- in *i*) ;; *) return;; esac`（Debian 的 ~/.bashrc）、`[[ $- != *i* ]] && return`、`[[ $- == *i* ]] || return`。守卫之前的每一行只能是空行、注释，
    或不开不收复合命令、不续行、不带 heredoc 与括号的简单命令（这样守卫在顶层，return 结束的是这一份 source）。弄坏开关 LIB_SHELL_WORDS_BREAK=non-interactive-return-unseen 下交 None。"""
    if non_interactive_return_unseen():
        return None
    offset = 0
    for line in text.split("\n"):
        stripped = line.strip()
        if any(guard.match(text, offset + len(line) - len(line.lstrip())) for guard in NON_INTERACTIVE_RETURN_GUARDS) and line[:1] not in (" ", "\t"):
            return offset
        if stripped and not stripped.startswith("#"):
            if stripped.endswith(("\\", "|", "&&", "||")) or "<<" in stripped or "(" in stripped or ")" in stripped or "`" in stripped:
                return None
            tokens = shell_tokens(stripped)
            if any(is_operator is False and value in BLOCK_OPENING_WORDS for is_operator, value in tokens) or any(is_operator is None for is_operator, _ in tokens):
                return None
        offset += len(line) + 1
    return None


def shebang_command(head):
    """`#!` 那一行指到的解释器名与它的参数（`#!/usr/bin/env bash` 取 bash，env 自己的选项与赋值词跳过，`-u` / `-C` 带一个值）；没有 `#!` 交 None，`#!` 后面空着交 ("", [])。"""
    if not head.startswith(b"#!"):
        return None
    end = head.find(b"\n")
    words = head[2:end if end >= 0 else len(head)].decode("utf-8", "replace").split()
    if not words:
        return "", []
    interpreter, rest = os.path.basename(words[0]), words[1:]
    if interpreter == "env":
        index = 0
        while index < len(rest) and (rest[index].startswith("-") or "=" in rest[index]):
            index += 2 if rest[index] in ("-u", "-C", "--unset", "--chdir") else 1
        interpreter, rest = (os.path.basename(rest[index]), rest[index + 1:]) if index < len(rest) else ("", [])
    return interpreter, rest


def executed_script_startup(command, head):
    """直接执行的脚本（command 的 launcher 是 None，命令词是脚本路径；head 是它开头的字节，读不到是 None）起来时，它的 shell 先读的启动文件：
    交 (启动文件命令的列表, 认不出的解释器名)。#! 指到 shell 的，按那个 shell 与 #! 行上的参数算（startup_file_commands）；没有 `#!` 的文本文件，从 bash 里执行是 bash 另起一份自己读它（实测读 BASH_ENV），
    当 bash 算；#! 指到 NON_SHELL_INTERPRETER 的不读；指到别的、而这条命令的环境里设了 BASH_ENV 或 ENV 的，认不出它读不读，交解释器名（空的 `#!` 交 ""），调用方照认不出办。
    编译产物（ELF、有 NUL 字节）不算脚本。弄坏开关 LIB_SHELL_WORDS_BREAK=executed-script-startup-unread 下什么都不交。"""
    if head is None or head.startswith(b"\x7fELF") or b"\0" in head or words_break_is_set("executed-script-startup-unread"):
        return [], None
    shebang = shebang_command(head)
    interpreter, arguments = ("bash", []) if shebang is None else shebang
    if is_shell_name(interpreter):
        startup_shell = "bash" if interpreter == "rbash" else interpreter
        return startup_file_commands(startup_shell, arguments + [command.words[0]], command.environment, command.variables or {}, command.directory,
                                     command.memory_capped, command.wrappers), None
    if NON_SHELL_INTERPRETER.fullmatch(interpreter) or not (command.environment.get(STARTUP_FILE_BASH_VARIABLE) or command.environment.get(STARTUP_FILE_POSIX_VARIABLE)):
        return [], None
    return [], interpreter


def export_forms_unseen():
    """弄坏开关 LIB_SHELL_WORDS_BREAK=declared-exports-unseen：环境里的变量只认 `export 名字=值`（declare -x、typeset -x、先赋值后 export、export -n、set -a 都不认，只给自检证红用）。"""
    return words_break_is_set("declared-exports-unseen")


IDENTIFIER = re.compile(r"[A-Za-z_][A-Za-z0-9_]*")
EXPORTING_COMMANDS = {"export", "declare", "typeset", "local", "readonly"}


def note_assignment(name, value, exported, plain_values, export_marked, allexport):
    """`名字=值` 赋值（单独成一条的、declare 不带 -x 的）：原文记进 plain_values；这个名字带着 export 属性（export 过、`export 名字` 先于赋值、set -a 之后）的，
    环境里的值跟着改（`export BASH_ENV=a; BASH_ENV=脚本; bash` 起的 bash 读的是脚本）。"""
    plain_values[name] = value
    if name in exported or name in export_marked or allexport:
        exported[name] = value


def apply_export_command(command_name, arguments, exported, plain_values, export_marked, variables):
    """export / declare / typeset / local / readonly 对环境的作用（弄坏开关 declared-exports-unseen 下调用方不走这里）：
    export 与带 -x 的 declare / typeset / local 让名字带上 export 属性（`名字=值` 同时赋值；单写名字的把已赋的值带进环境，还没值的记进 export_marked，以后赋了就进环境），
    export -n 与带 +x 的撤掉（值留着、不进环境）；别的只是赋值（note_assignment）。-f（函数）与 -p（打印）带的单写名字不管。"""
    minus, plus, names = "", "", []
    options_done = False
    for argument in arguments:
        if not options_done and argument == "--":
            options_done = True
        elif not options_done and len(argument) > 1 and argument[0] in "-+" and not ASSIGNMENT.match(argument):
            if argument[0] == "-":
                minus += argument[1:]
            else:
                plus += argument[1:]
        else:
            names.append(argument)
    exporting = "n" not in minus if command_name == "export" else "x" in minus
    unexporting = "n" in minus if command_name == "export" else "x" in plus
    for argument in names:
        assignment = ASSIGNMENT.match(argument)
        if assignment is None and (not IDENTIFIER.fullmatch(argument) or "f" in minus or "p" in minus):
            continue
        variable = assignment.group(1) if assignment else argument
        if assignment:
            plain_values[variable] = assignment.group(2)
            if command_name == "export":
                variables[variable] = assigned_value(assignment.group(2), variables)
        if unexporting:
            exported.pop(variable, None)
            export_marked.discard(variable)
        elif exporting:
            if variable in plain_values:
                exported[variable] = plain_values[variable]
            elif variable not in exported:
                export_marked.add(variable)
        elif assignment:
            note_assignment(variable, assignment.group(2), exported, plain_values, export_marked, False)


def allexport_after_set(arguments, allexport):
    """`set -a`、`set -o allexport` 之后赋值的变量都进环境，`set +a`、`set +o allexport` 撤回；别的选项不动它。"""
    position = 0
    while position < len(arguments):
        argument = arguments[position]
        if argument == "--":
            break
        if argument in ("-o", "+o"):
            if arguments[position + 1:position + 2] == ["allexport"]:
                allexport = argument == "-o"
            position += 2
            continue
        if len(argument) > 1 and argument[0] in "-+" and not argument.startswith("--") and "a" in argument[1:]:
            allexport = argument[0] == "-"
        position += 1
    return allexport


def aliases_unexpanded():
    """弄坏开关 LIB_SHELL_WORDS_BREAK=aliases-unexpanded：别名不展开（只给自检证红用）。"""
    return words_break_is_set("aliases-unexpanded")


def shell_aliases(text, definition_lines=None):
    """text 里 `alias 名字='值'` 定义的别名（名字 -> 值，值照字面当命令再切）；只在文本里有 `shopt -s expand_aliases` 开着时才交
    （非交互 bash 默认不展开别名）。弄坏开关 LIB_SHELL_WORDS_BREAK=aliases-unexpanded 下交空。
    definition_lines 是一个 dict 时，按名字记下定义它的那几行（从 0 数，照切好的分隔符里的换行数；commands_at_command_position 拿它认同一行里定义的不展开）。"""
    if aliases_unexpanded() or "alias" not in text:
        return {}
    boundaries = []
    simple = simple_commands(shell_tokens(strip_data_heredocs(text)), boundaries=boundaries)[0]
    enabled, found, line = False, {}, 0
    for words, boundary in zip(simple, boundaries):
        line += boundary.count("\n")
        skipped = skip_prefixes(words)
        if skipped.command_position is None:
            continue
        rest = words[skipped.command_position:]
        head = os.path.basename(rest[0])
        if head == "shopt" and "-s" in rest[1:] and "expand_aliases" in rest[1:]:
            enabled = True
        elif head == "alias":
            for argument in rest[1:]:
                assignment = ASSIGNMENT.match(argument)
                if assignment:
                    found[assignment.group(1)] = assignment.group(2)
                    if definition_lines is not None:
                        definition_lines.setdefault(assignment.group(1), set()).add(line)
    return found if enabled else {}


def commands_at_command_position(text, directory=None, environment=None, depth=0, stripped_heredocs=None, memory_capped=False, variables=None,
                                 uncalled_functions_skipped=False, wrappers=(), directory_guarded=True, interactive=False):
    """整段命令文本里命令位置上的每一条命令（前缀与包装剥掉、cd 与 export 跟着走），连同各层的放后台与输出重定向写法。
    stripped_heredocs 是递归时往里传的、各层剥掉的 heredoc 正文（记号里的下标指向它），调用方不用给。
    memory_capped：这段文本整个跑在 run-with-memory-cap.sh 里（它是经它起的脚本的正文）；每条命令的 memory_capped 从它起算。
    喂给 shell 的 heredoc 正文当同一段文本里的后续命令切，挂 heredoc 的那条命令上的包装不算在正文的命令上。
    variables：cd / pushd / env -C 的目录里展开变量用的值（名字 → 已展开的值，None 是算不出），递归时往里传，调用方不用给；不给时从 environment 起算。
    单独成一条的 `名字=值`、export、local / declare 这类记进去，for / select / read / mapfile / readarray / unset 改写过的记成算不出（rebind_variables）；
    数组 `名字=(元素 …)` 记成 `(元素 …)` 这一串（元素切过词、引号已去掉、用空格连），单独成一条的 `名字+=(…)` / `名字+=值` 接在记下的值后面、`名字[下标]=值` 与 `unset 名字[下标]`
    记成算不出（record_array_writes）；接不上的（之前没记过、算不出、一边是数组一边不是）记成算不出。
    命令前的 `名字=值 命令` 不记（只给那一条命令，它的参数也不按它展开）。
    uncalled_functions_skipped：text 是一整条交给 shell 跑完就退出的命令（Bash 调用的命令文本本身，不是会被 source 进别处的脚本正文）时由调用方给真，
    只定义、调不到的 `function 名字 { …; }` 的函数体不判（drop_uncalled_function_bodies；只管这一层，bash -c、eval、替换里的照判）。
    directory_guarded：起这段文本的那一条在外层的 cd_failure_skips（递归时往里传；最外层是真）；每条命令的 cd_failure_skips 是它与这一层里前面各处可能失败的 cd 都守着它
    （`&&` 一串、`|| exit` 之后）的合取：`;`、换行、`||`、`&` 断开 `&&` 一串，括号进出一律断开（弄坏开关 cd-failure-guards-unseen 下都是假），
    只有 `&&` 后面紧跟的子 shell（`cd 目录 && ( … )`、`cd 目录 && time ( … )`）照守着算（guarded_subshell_boundary；弄坏开关 cd-subshell-guards-unseen 下照旧断开）。
    interactive：跑这段文本的 shell 可能是交互的（递归进 bash -i 的代码、脚本照那个 shell 改），记进每条命令的 interactive。"""
    if stripped_heredocs is None:
        stripped_heredocs = []
    tokens = shell_tokens(strip_data_heredocs(text, stripped_bodies=stripped_heredocs))
    undropped_tokens = tokens
    if uncalled_functions_skipped and depth == 0:
        tokens = drop_uncalled_function_bodies(tokens, text, stripped_heredocs)
    alias_lines = {}
    aliases = shell_aliases(text, alias_lines)   # shopt -s expand_aliases 开着时，命令词是别名的展开
    # 同一行里定义的别名这一行不展开（bash 读完一整行才执行；复跑分类报告 d137-replay-classify 二节 E4）：行号照分隔符里的换行数，与 shell_aliases 记的对得上才认；
    # 去掉过只定义的函数体的（行号对不上）照旧展开。弄坏开关 LIB_SHELL_WORDS_BREAK=aliases-same-line-expanded 下不看行号
    if tokens is not undropped_tokens or words_break_is_set("aliases-same-line-expanded"):
        alias_lines = None
    command_line = 0   # 这一条简单命令在 text 里第几行（从 0 数）
    separators, boundaries = [], []
    simple, output_targets, substitutions, redirections = simple_commands(tokens, separators, boundaries)
    cd_failure_poisoned, cd_and_chain, cd_seen = False, False, False   # 前面有一处可能失败的 cd 不守着后面了；在一串 `&&` 里；这一层 cd 过
    subshell_guards = {"depth": 0, "groups": []}   # 括号层数与开着的 `cd 目录 && (` 那几组（guarded_subshell_boundary）
    previous_directory = None   # 上一处 cd 之前的目录（$OLDPWD、`cd -`）；这段文本里还没 cd 过是 None（算不出）
    commands, prefix_names, unjudged = [], set(), []
    previous_command = None   # 上一条简单命令：(词, 喂给它的 heredoc 正文)，管道后一段的 shell 从它取标准输入
    has_lone_ampersand = (True, "&") in tokens
    exported = dict(environment or {})
    plain_values, export_marked, allexport = {}, set(), False   # 赋过的变量原文（export 单写名字时带进环境）、带 export 属性而还没值的名字、set -a 开着没有
    if variables is None:
        variables = {}
        for name, value in exported.items():
            variables[name] = assigned_value(value, variables)
    else:
        variables = dict(variables)
    for index, (words, command_substitutions, command_redirections) in enumerate(zip(simple, substitutions, redirections)):
        boundary = boundaries[index] if index < len(boundaries) else ()
        command_line += boundary.count("\n")
        if not words_break_is_set("cd-subshell-guards-unseen"):
            boundary, cd_and_chain, cd_failure_poisoned, cd_seen = guarded_subshell_boundary(
                boundary, simple[index - 1] if index else [], subshell_guards, cd_and_chain, cd_failure_poisoned, cd_seen)
        if cd_seen and ("(" in boundary or ")" in boundary):
            cd_failure_poisoned = True
        if cd_and_chain and not (boundary and boundary[0] in ("&&", "|", "|&") and all(separator == "\n" for separator in boundary[1:])):
            cd_and_chain, cd_failure_poisoned = False, True
        command_guarded = directory_guarded and not cd_failure_poisoned and not cd_failure_guards_unseen()
        fed_heredocs = heredocs_fed(command_redirections, stripped_heredocs)
        piped_from = previous_command if index < len(separators) and separators[index] in PIPE_SEPARATORS else None
        previous_command = (words, fed_heredocs)
        for substitution in command_substitutions:
            if depth < MAXIMUM_NESTING:
                inner = commands_at_command_position(substitution, directory, exported, depth + 1, stripped_heredocs, memory_capped, variables, wrappers=wrappers,
                                                     directory_guarded=command_guarded, interactive=interactive)
                commands += inner.commands
                prefix_names |= inner.prefix_names
                has_lone_ampersand = has_lone_ampersand or inner.has_lone_ampersand
                output_targets += inner.output_targets
                unjudged += inner.unjudged_shell_inputs
        if not words:
            continue
        record_array_writes(words, variables)
        standalone = skip_prefixes(words)
        if standalone.command_position is None and not standalone.prefix_names:
            for word in words:
                assignment = ASSIGNMENT.match(word)
                if assignment:
                    variables[assignment.group(1)] = assigned_value(assignment.group(2), variables)
                    if not export_forms_unseen():
                        note_assignment(assignment.group(1), assignment.group(2), exported, plain_values, export_marked, allexport)
        attached = {
            "variables": dict(variables),
            "output_redirections": tuple((operator, target) for operator, target in command_redirections if operator in OUTPUT_REDIRECTS),
            "standard_input_heredocs": fed_heredocs,
            "piped_from": piped_from,
            "input_redirections": tuple((operator, target) for operator, target in command_redirections
                                        if operator in STANDARD_INPUT_REDIRECTS and operator not in ("<<", "<<-")),
            "cd_failure_skips": command_guarded,
            "interactive": interactive,
        }
        command_environment = dict(exported)
        command_directory = directory   # 这一条命令的当前目录：env -C 换的只管这一条，不像 cd 那样往后带
        remaining = words
        launcher = None
        command_memory_capped = memory_capped
        command_wrappers = tuple(wrappers)
        command_login = False   # 前缀里的 exec -l / exec -a -名字：后面那个 shell 是登录 shell
        for _ in range(MAXIMUM_NESTING):
            skipped, remaining = skip_prefixes_expanding(remaining)
            prefix_names |= skipped.prefix_names
            command_login = command_login or skipped.login_shell
            if skipped.environment_cleared:
                command_environment.clear()   # env -i 清掉外面带进来的与它之前写的（PrefixSkip.environment_cleared）
            command_environment.update(skipped.assignments)
            if skipped.chdir is not None:
                command_directory = env_directory(command_directory, skipped.chdir, variables)
            if skipped.command_position is None:
                break
            remaining = remaining[skipped.command_position:]
            name, arguments = os.path.basename(remaining[0]), remaining[1:]
            if remaining[0] in aliases and depth < MAXIMUM_NESTING and (alias_lines is None or alias_lines.get(remaining[0]) != {command_line}):
                # 命令词是 shopt -s expand_aliases 之后定义的别名：展开成别名的值再切；
                # 别名的值当文本重切，它自己的 text 里没有别名定义，不会再展开同一个名字、不会无限递归
                expanded_text = aliases[remaining[0]] + ("".join(" " + shlex.quote(argument) for argument in arguments))
                inner = commands_at_command_position(expanded_text, command_directory, command_environment, depth + 1, stripped_heredocs,
                                                     command_memory_capped, variables, wrappers=command_wrappers,
                                                     directory_guarded=command_guarded, interactive=attached["interactive"])
                commands += inner.commands
                prefix_names |= inner.prefix_names
                has_lone_ampersand = has_lone_ampersand or inner.has_lone_ampersand
                output_targets += inner.output_targets
                unjudged += inner.unjudged_shell_inputs
                break
            if name == "export":
                if export_forms_unseen():
                    for argument in arguments:
                        assignment = ASSIGNMENT.match(argument)
                        if assignment:
                            exported[assignment.group(1)] = assignment.group(2)
                            variables[assignment.group(1)] = assigned_value(assignment.group(2), variables)
                else:
                    apply_export_command(name, arguments, exported, plain_values, export_marked, variables)
                break
            if name == "unset":
                for argument in arguments:
                    if not argument.startswith("-"):
                        exported.pop(argument, None)
                        plain_values.pop(argument, None)
                        export_marked.discard(argument)
                        if not variable_rebinding_unseen():
                            variables[argument.split("[", 1)[0]] = None
                break
            if name in ("cd", "pushd"):
                before_cd = directory
                directory = changed_directory(directory, arguments, variables, previous_directory)
                previous_directory, cd_seen = before_cd, True
                if directory is not None and not os.path.isdir(directory):
                    # 目标盘上不在：这一处 cd 可能失败（同一条命令里前面 mkdir 的也算在内）。后面跟 `&&` 的，那一串里的照 cd 成功了判；`|| exit` 守着的，后面同一层的照 cd 成功了判；
                    # 别的写法 cd 失败之后照样往下跑，后面的都认不准
                    following = boundaries[index + 1] if index + 1 < len(boundaries) else ()
                    continued = bool(following) and all(separator == "\n" for separator in following[1:])
                    opens_subshell = (bool(following) and following[0] == "&&" and not words_break_is_set("cd-subshell-guards-unseen")
                                      and {separator for separator in following[1:] if separator != "\n"} == {"("})
                    if (continued or opens_subshell) and following[0] == "&&":
                        cd_and_chain = True
                    elif not (continued and following[0] == "||" and cd_failure_exits(simple, boundaries, index + 1)):
                        cd_failure_poisoned = True
                break
            if name == "eval" and arguments and not words_break_is_set("eval-unread"):
                # eval 把参数（引号已去掉）用空格连起来当代码再跑一遍：连起来的文字照一条完整的命令再切（同 bash -c）。
                # 参数里外层先展开的变量与命令替换照原文切（展开成什么判不到，变量里拼出来的命令照旧认不出）
                if depth < MAXIMUM_NESTING:
                    inner = commands_at_command_position(" ".join(arguments), command_directory, command_environment, depth + 1, stripped_heredocs,
                                                         command_memory_capped, variables, wrappers=command_wrappers,
                                                     directory_guarded=command_guarded, interactive=attached["interactive"])
                    commands += inner.commands
                    prefix_names |= inner.prefix_names
                    has_lone_ampersand = has_lone_ampersand or inner.has_lone_ampersand
                    output_targets += inner.output_targets
                    unjudged += inner.unjudged_shell_inputs
                break
            if name == "trap" and not words_break_is_set("trap-action-unread"):
                # trap 'action' SIG…：第一个操作数（跳过 -l / -p / --）是一段 shell 代码，trap 触发时执行；照一条完整命令再切（同 bash -c）。
                # action 是 `-`（重置）或没给 action 的不判
                option_index = 0
                while option_index < len(arguments) and arguments[option_index] in ("-l", "-p", "--"):
                    terminator = arguments[option_index] == "--"
                    option_index += 1
                    if terminator:
                        break
                action = arguments[option_index] if option_index < len(arguments) else None
                if action is not None and action != "-" and depth < MAXIMUM_NESTING:
                    inner = commands_at_command_position(action, command_directory, command_environment, depth + 1, stripped_heredocs,
                                                         command_memory_capped, variables, wrappers=command_wrappers,
                                                     directory_guarded=command_guarded, interactive=attached["interactive"])
                    commands += inner.commands
                    prefix_names |= inner.prefix_names
                    has_lone_ampersand = has_lone_ampersand or inner.has_lone_ampersand
                    output_targets += inner.output_targets
                    unjudged += inner.unjudged_shell_inputs
                break
            if name == "set" and not export_forms_unseen():
                allexport = allexport_after_set(arguments, allexport)
            if name in EXPORTING_COMMANDS and not export_forms_unseen():
                apply_export_command(name, arguments, exported, plain_values, export_marked, variables)
            if name in REBINDING_COMMANDS or name in DECLARATION_COMMANDS:
                rebind_variables(name, arguments, variables)
            if is_shell_name(name):
                shell_name = "bash" if name == "rbash" else name   # 限制 bash 的启动文件、posix 判定照 bash
                invocation = shell_invocation(arguments)
                # 这个 shell 交不交互：带 -i 的、不带 -c 也不带脚本（从标准输入读，可能是终端）的照交互算；它的启动文件、代码、脚本照这个记
                attached["interactive"] = shell_start_options(shell_name, arguments).interactive or (invocation.code_string is None and invocation.script_index is None)
                if not invocation.runs_nothing:
                    # 启动时先读 BASH_ENV / ENV 指的文件：在这个 shell 自己的脚本、代码之前执行
                    commands += startup_file_commands(shell_name, arguments, command_environment, variables, command_directory, command_memory_capped, command_wrappers,
                                                      command_login, attached["interactive"])
                if invocation.code_string is not None:
                    if depth < MAXIMUM_NESTING:
                        inner = commands_at_command_position(invocation.code_string, command_directory, command_environment, depth + 1, stripped_heredocs,
                                                             command_memory_capped, variables, wrappers=command_wrappers,
                                                     directory_guarded=command_guarded, interactive=attached["interactive"])
                        commands += inner.commands
                        prefix_names |= inner.prefix_names
                        has_lone_ampersand = has_lone_ampersand or inner.has_lone_ampersand
                        output_targets += inner.output_targets
                        unjudged += inner.unjudged_shell_inputs
                    break
                # bash / sh 起的脚本路径是标准输入设备（/dev/stdin、/dev/fd/0、/proc/self/fd/0）：这个 shell 读的是标准输入，同不带脚本、带 -s 那样从重定向 / 管道取
                # （弄坏开关 LIB_SHELL_WORDS_BREAK=standard-input-device-script-unread 下照旧把设备路径当一份脚本文件读）
                script_word = arguments[invocation.script_index] if invocation.script_index is not None else None
                reads_from_device = (script_word is not None and is_standard_input_device(script_word) and not words_break_is_set("standard-input-device-script-unread"))
                reads_standard = shell_reads_standard_input(arguments) or reads_from_device
                fed_files = shell_standard_input_files(command_redirections, piped_from) if reads_standard else []
                for fed_file in fed_files:
                    # 标准输入是一份文件（`bash -s < 文件`、`bash < 文件`、`cat 文件 | bash`、`bash /dev/stdin < 文件`）：等于把那份文件交给这个 shell 跑，交出这份文件当 shell 起的脚本，由调用方读正文判
                    commands.append(CommandAtPosition([fed_file], command_directory, command_environment, "shell", **attached, memory_capped=command_memory_capped, wrappers=command_wrappers))
                if reads_standard and not shell_standard_input_unread() and not fed_files:
                    # here-string 与管道喂进来的字面文本当这个 shell 的代码再切一遍（同 bash -c：包装、目录、变量、内存包装照这一条算）；判不了的记一句
                    fed = shell_standard_input(command_redirections, piped_from)
                    if fed.unjudged:
                        unjudged.append(f"喂给 {name} 的标准输入判不了：{fed.unjudged}")
                    if fed.code is not None and depth < MAXIMUM_NESTING:
                        inner = commands_at_command_position(fed.code, command_directory, command_environment, depth + 1, stripped_heredocs,
                                                             command_memory_capped, variables, wrappers=command_wrappers,
                                                     directory_guarded=command_guarded, interactive=attached["interactive"])
                        commands += inner.commands
                        prefix_names |= inner.prefix_names
                        has_lone_ampersand = has_lone_ampersand or inner.has_lone_ampersand
                        output_targets += inner.output_targets
                        unjudged += inner.unjudged_shell_inputs
                if invocation.script_index is None or reads_from_device:
                    commands.append(CommandAtPosition(remaining, command_directory, command_environment, **attached, memory_capped=command_memory_capped, wrappers=command_wrappers))
                    break
                remaining = arguments[invocation.script_index:]
                launcher = "shell"
                continue
            if PYTHON_NAME.match(name):
                invocation = python_invocation(arguments)
                if invocation.module:
                    # `python3 -m <模块>`：被执行的是 sys.path 上的 <模块>.py（当前目录排第一），交出这个名字，调用方按位置认
                    remaining = [invocation.module.replace(".", "/") + ".py"] + arguments[invocation.script_index:]
                    launcher = "python"
                    continue
                if invocation.code_string is not None or invocation.script_index is None:
                    # `python3 -c '…'`、`python3 -`、只有选项：没有被执行的脚本文件，交出 python 这一条本身
                    commands.append(CommandAtPosition(remaining, command_directory, command_environment, **attached, memory_capped=command_memory_capped, wrappers=command_wrappers))
                    break
                remaining = arguments[invocation.script_index:]
                launcher = "python"
                continue
            if name in ("source", "."):
                if arguments and is_standard_input_device(arguments[0]) and not words_break_is_set("standard-input-device-script-unread"):
                    # source /dev/stdin：读的是标准输入（here-string、管道喂进来的文本），当这个文本再切
                    fed = shell_standard_input(command_redirections, piped_from)
                    if fed.unjudged:
                        unjudged.append(f"喂给 source 的标准输入判不了：{fed.unjudged}")
                    if fed.code is not None and depth < MAXIMUM_NESTING:
                        inner = commands_at_command_position(fed.code, command_directory, command_environment, depth + 1, stripped_heredocs,
                                                             command_memory_capped, variables, wrappers=command_wrappers,
                                                     directory_guarded=command_guarded, interactive=attached["interactive"])
                        commands += inner.commands
                        prefix_names |= inner.prefix_names
                        has_lone_ampersand = has_lone_ampersand or inner.has_lone_ampersand
                        output_targets += inner.output_targets
                        unjudged += inner.unjudged_shell_inputs
                    break
                if not arguments or not SOURCE_TARGET.match(arguments[0]):
                    commands.append(CommandAtPosition(remaining, command_directory, command_environment, **attached, memory_capped=command_memory_capped, wrappers=command_wrappers))
                    break
                remaining = arguments
                launcher = "source"
                continue
            if name in WRAPPERS_TAKING_ONE_ARGUMENT:
                if not arguments or arguments[0].startswith("-") or len(arguments) < 2:
                    commands.append(CommandAtPosition(remaining, command_directory, command_environment, **attached, memory_capped=command_memory_capped, wrappers=command_wrappers))
                    break
                command_wrappers += ((remaining[0], command_directory),)
                remaining = arguments[1:]
                launcher = None
                command_memory_capped = command_memory_capped or name == MEMORY_CAP_WRAPPER
                continue
            commands.append(CommandAtPosition(remaining, command_directory, command_environment, launcher, **attached, memory_capped=command_memory_capped, wrappers=command_wrappers))
            break
    return ShellScan(commands, prefix_names, has_lone_ampersand, output_targets, tuple(unjudged))


# 别的程序里起 shell 与命令（认不出的一律拒，偏严）：
# python / perl / ruby / node / php / lua 的 -c、-e 这类代码串与喂进标准输入的字面代码，awk 的程序文本，sed 的脚本，tclsh / expect 的代码，make 的 Makefile 与 --eval，
# git 的别名与带命令的配置项，tar、ssh、less、vim 带命令的选项。字面写着的命令照一段 shell 命令交出（调用方同 bash -c 那样再判）；起了命令而写法认不出的（变量、拼接、
# 动态求值）交一句说明，调用方照认不出办；程序从文件读的代码（awk -f、sed -f、Makefile）交文件，调用方读正文交 program_text_launches 判。
# python / perl / ruby / tclsh 起的脚本文件不读（.claude/agent-common.md「写」一节：python subprocess 起的、脚本文件里的写法由子 agent 自己守）。
class ProgramFile(NamedTuple):
    language: str          # "awk"、"sed"、"make"：program_text_launches 照哪种语言读
    words: tuple           # 候选路径词（照写的样子，没解开）：make 没给 -f 时按 GNUmakefile、makefile、Makefile 的次序取头一份在的，别的只有一个
    directory: str | None  # 解相对路径用的当前目录（make -C 换过的）
    variables: tuple = ()  # make / awk 命令行上的 `名字=值`：((名字, 值), …)，读 Makefile 时先于文件里的定义；awk 的值写成 shell 变量、命令替换的是 None


class EmbeddedLaunches(NamedTuple):
    texts: tuple           # 程序里字面写着、照一段 shell 命令判的文本
    unjudged: tuple        # 起了命令、写法认不出的说明（每处一句）
    program_files: tuple   # (ProgramFile, …)


EMBEDDED_PYTHON_SHELL_CALLS = {("os", "system"), ("os", "popen"), ("posix", "system"), ("subprocess", "getoutput"), ("subprocess", "getstatusoutput"),
                               ("commands", "getoutput"), ("commands", "getstatusoutput"), ("asyncio", "create_subprocess_shell")}
EMBEDDED_PYTHON_SUBPROCESS_CALLS = {"run", "call", "check_call", "check_output", "Popen"}
EMBEDDED_PYTHON_EXEC_LIST_CALLS = {"execl", "execle", "execlp", "execlpe"}        # (程序, argv0, 参数…)
EMBEDDED_PYTHON_EXEC_VECTOR_CALLS = {"execv", "execve", "execvp", "execvpe"}     # (程序, [argv0, 参数…])
EMBEDDED_PYTHON_SPAWN_LIST_CALLS = {"spawnl", "spawnle", "spawnlp", "spawnlpe"}  # (模式, 程序, argv0, 参数…)
EMBEDDED_PYTHON_SPAWN_VECTOR_CALLS = {"spawnv", "spawnve", "spawnvp", "spawnvpe"}
EMBEDDED_PYTHON_LAUNCH_MODULES = {"os", "posix", "subprocess", "commands", "asyncio", "pty"}
EMBEDDED_PYTHON_OPAQUE_MODULES = {"ctypes", "cffi"}   # 能直接调 libc 的 system / exec：认不出它起什么
EMBEDDED_PYTHON_DISTINCT_LAUNCH_NAMES = ({"system", "popen", "Popen", "check_output", "check_call", "getoutput", "getstatusoutput", "posix_spawn", "posix_spawnp",
                                          "create_subprocess_shell", "create_subprocess_exec"}
                                         | EMBEDDED_PYTHON_EXEC_LIST_CALLS | EMBEDDED_PYTHON_EXEC_VECTOR_CALLS | EMBEDDED_PYTHON_SPAWN_LIST_CALLS
                                         | EMBEDDED_PYTHON_SPAWN_VECTOR_CALLS)
EMBEDDED_PYTHON_SYNTAX_HINTS = ("system", "popen", "Popen", "subprocess", "exec", "spawn", "pty", "ctypes", "__import__", "import_module")
PERL_LAUNCH_WORD = re.compile(r"(?<![\w$@%&>:.\-])(system|exec|readpipe|qx|open)\b")
RUBY_LAUNCH_WORD = re.compile(r"(?<![\w$@:.])(?:system|exec|spawn|IO\.popen|Open3\.\w+|Process\.spawn|Kernel\.(?:system|exec|spawn)|open)\b|%x")
NODE_LAUNCH_WORD = re.compile(r"(?<![\w$])(execSync|execFileSync|execFile|spawnSync|spawn|exec|fork)\s*\(")
PHP_LAUNCH_WORD = re.compile(r"(?<![\w$>:])(system|exec|shell_exec|passthru|popen|proc_open|pcntl_exec)\s*\(")
LUA_LAUNCH_WORD = re.compile(r"\b(os\.execute|io\.popen)\b")
TCL_LAUNCH_COMMANDS = {"exec", "spawn", "exp_spawn", "system", "exp_system"}
TCL_OPAQUE_COMMANDS = {"eval", "uplevel", "subst", "interp", "source"}
AWK_NAMES = {"awk", "gawk", "mawk", "nawk"}
PERL_NAME = re.compile(r"^perl[0-9.]*$")
RUBY_NAME = re.compile(r"^ruby[0-9.]*$")
PHP_NAME = re.compile(r"^php[0-9.]*$")
LUA_NAME = re.compile(r"^(?:lua[0-9.]*|luajit)$")
TCL_NAMES = {"tclsh", "wish", "expect"}
TCL_NAME = re.compile(r"^(?:tclsh|wish)[0-9.]*$|^expect$")
NODE_NAMES = {"node", "nodejs"}
VIM_NAMES = {"vim", "vi", "nvim", "ex", "view", "vimdiff", "gvim", "rvim"}
MAKE_NAMES = {"make", "gmake"}
MAKE_DEFAULT_FILES = ("GNUmakefile", "makefile", "Makefile")
GIT_COMMAND_CONFIG_KEY = re.compile(r"^(?:core\.(?:pager|editor|sshcommand|fsmonitor|askpass|alternaterefscommand)|sequence\.editor|diff\.external|diff\..+\.(?:command|textconv)"
                                    r"|merge\..+\.driver|filter\..+\.(?:clean|smudge|process)|gpg\.program|gpg\..+\.program|pager\..+|interactive\.difffilter"
                                    r"|uploadpack\.packobjectshook)$", re.I)
GIT_CONFIG_QUERY_PATTERN = (r"^(alias\..*|core\.(pager|editor|sshcommand|fsmonitor|askpass|alternaterefscommand)|sequence\.editor|diff\.external|diff\..*\.(command|textconv)"
                            r"|merge\..*\.driver|filter\..*\.(clean|smudge|process)|credential\.helper|credential\..*\.helper|gpg\.program|gpg\..*\.program|pager\..*"
                            r"|interactive\.difffilter|uploadpack\.packobjectshook)$")
GIT_BOOLEAN_VALUES = {"true", "false", "yes", "no", "on", "off", "1", "0", ""}
GIT_COMMAND_ENVIRONMENT = ("GIT_EXTERNAL_DIFF", "GIT_PAGER", "PAGER", "GIT_EDITOR", "EDITOR", "VISUAL", "GIT_SEQUENCE_EDITOR", "GIT_SSH_COMMAND", "GIT_SSH",
                           "GIT_ASKPASS", "SSH_ASKPASS", "GIT_PROXY_COMMAND")
GIT_OPAQUE_ENVIRONMENT = ("GIT_EXEC_PATH", "GIT_CONFIG_PARAMETERS")   # 换掉 git 的子命令程序、或带着 git 自己引号写法的配置：认不出起什么
GIT_OPTIONS_WITH_VALUE = {"-c", "-C", "--git-dir", "--work-tree", "--namespace", "--super-prefix", "--config-env"}
GIT_CONFIG_QUERY_SECONDS = 5
GIT_CONFIG_QUERY_CACHE = {}
TAR_COMMAND_OPTIONS = ("--to-command", "--use-compress-program", "--info-script", "--new-volume-script", "--rsh-command", "--rmt-command")
SSH_COMMAND_OPTIONS = ("proxycommand", "localcommand", "knownhostscommand")
LESS_OPEN_ENVIRONMENT = ("LESSOPEN", "LESSCLOSE")
MAKE_DEFAULT_VARIABLES = {"MAKE": "make", "CC": "cc", "CXX": "g++", "CPP": "cc -E", "AR": "ar", "AS": "as", "LD": "ld", "RM": "rm -f", "SHELL": "/bin/sh",
                          "LEX": "lex", "YACC": "yacc", "FC": "f77", "M2C": "m2c", "PC": "pc", "CO": "co", "GET": "get", "TEX": "tex", "MAKEINFO": "makeinfo"}
MAKE_RECIPE_PREFIX = re.compile(r"^[@+\-\s]*")
MAKE_ASSIGNMENT = re.compile(r"^(?:(?:export|override|private)\s+)*([A-Za-z_][A-Za-z0-9_.\-]*)\s*(:::=|::=|:=|\?=|\+=|!=|=)\s*(.*)$")
MAKE_VARIABLE_PLACEHOLDER_PREFIX = "MAKE_UNRESOLVED_"   # 展开不了的 make 变量换成没展开的 shell 变量：落在命令词上的照认不出办
EMBEDDED_NESTING = 4


def embedded_launches_unseen():
    """弄坏开关 LIB_SHELL_WORDS_BREAK=embedded-launches-unseen：别的程序里起的命令一条都不认。"""
    return words_break_is_set("embedded-launches-unseen")


def heredoc_variables_unexpanded():
    """弄坏开关 LIB_SHELL_WORDS_BREAK=heredoc-variables-unexpanded：heredoc 正文里前面赋过字面值的变量不代入。"""
    return words_break_is_set("heredoc-variables-unexpanded")


def heredoc_body_expanded(text, variables):
    """结束词不带引号的 heredoc 正文照 bash 展开之后的文字：`$名字`、`${名字}` 换成这段命令文本里前面赋过的值（variables，名字 → 已展开的值、None 是算不出；
    查不到的只认 HOME，同 expand_known_variables），`\\$`、`\\``、`\\\\` 换成后面那个字，`\\` 接换行去掉（续行），别的反斜杠照留。
    有一处展开不了交 None：没赋过或算不出的变量、`$(…)`、反引号、`$((…))`、`${名字:-…}` 这类别的写法、`$1`、`$@` 这类特殊参数，`$` 后面不接名字的也算
    （复跑分类报告 d137-replay-classify 二节 E1；用户口径「认不出的一律拒」：认得出的照代入，有一处认不出整段照旧判不了）。"""
    output, index = [], 0
    while index < len(text):
        character = text[index]
        if character == "\\" and index + 1 < len(text):
            following = text[index + 1]
            if following != "\n":
                output.append(following if following in "$`\\" else character + following)
            index += 2
            continue
        if character == "`":
            return None
        if character == "$":
            reference = VARIABLE_REFERENCE.match(text, index)
            if reference is None:
                return None
            name = reference.group(1) or reference.group(2)
            value = variables[name] if name in variables else (os.environ.get("HOME") if name == "HOME" else None)
            if value is None:
                return None
            output.append(value)
            index = reference.end()
            continue
        output.append(character)
        index += 1
    return "".join(output)


def interpreter_standard_input(command):
    """解释器从标准输入读代码时读到的：交 (代码文本或 None, 判不了的说明或 None, 是不是盘上的文件)。heredoc 取最后一个的正文：结束词不带引号、正文里有 `$` 或反引号的
    照 heredoc_body_expanded 代入前面赋过字面值的变量再交（弄坏开关 LIB_SHELL_WORDS_BREAK=heredoc-variables-unexpanded 下不代入），代入不了的交原文、另交说明；
    `< 文件`、`cat 文件 |` 是盘上的文件（这里不读）；here-string 与管道前一段的 printf / echo 照 shell_standard_input。"""
    if command.standard_input_heredocs:
        body = command.standard_input_heredocs[-1]
        if body.is_delimiter_quoted or not has_expansion([body.text]):
            return body.text, None, False
        expanded = None if heredoc_variables_unexpanded() else heredoc_body_expanded(body.text, command.variables or {})
        if expanded is not None:
            return expanded, None, False
        return body.text, "喂进去的 heredoc 结束词没带引号、正文里有变量或命令替换，展开之后是什么判不了", False
    if shell_standard_input_files(command.input_redirections, command.piped_from):
        return None, None, True
    fed = shell_standard_input(command.input_redirections, command.piped_from)
    return fed.code, fed.unjudged, False


def literal_quoted_string(text, start, interpolating_marks):
    """text[start] 是 ' 或 " 或反引号：交 (字符串的值或 None（带插值、认不出）, 收尾之后的下标)；没收尾交 (None, len(text))。单引号里只认 \\\\ 与 \\'；双引号与反引号里的反斜杠转义照收，
    值里有 interpolating_marks 里的哪一种（perl 的 $ 与 @、ruby 的 #{、js 的 ${）就交 None。"""
    quote = text[start]
    index, value, interpolated = start + 1, [], False
    while index < len(text):
        character = text[index]
        if character == "\\" and index + 1 < len(text):
            following = text[index + 1]
            if quote == "'" and following not in "\\'":
                value.append(character)
            else:
                value.append({"n": "\n", "t": "\t"}.get(following, following) if quote != "'" else following)
            index += 2
            continue
        if character == quote:
            return (None if interpolated else "".join(value)), index + 1
        if quote != "'" and any(text.startswith(mark, index) for mark in interpolating_marks):
            interpolated = True
        value.append(character)
        index += 1
    return None, len(text)


def literal_argument_list(text, start, interpolating_marks, extra_quotes=""):
    """从 text[start] 起读一串用逗号隔开的字面字符串参数（外面可以套一层圆括号，参数也可以写成一个 [ … ] 数组）：交 (值的列表或 None（有不是字面的）, 读到的终点)。
    读到 `)`、`;`、换行、`}` 或字符串之外的别的写法就停；一个都没读到交 None。"""
    index = start
    while index < len(text) and text[index] in " \t":
        index += 1
    parenthesized = index < len(text) and text[index] == "("
    if parenthesized:
        index += 1
    values, bracketed = [], False
    while index < len(text):
        while index < len(text) and text[index] in " \t\n":
            index += 1
        if index >= len(text):
            break
        character = text[index]
        if character == "[" and not bracketed:
            bracketed = True   # 数组参数（`spawn("cargo", ["test", "--all"])`、`IO.popen(["cargo", …])`）的元素接在前面的参数后面
            index += 1
            continue
        if character in "'\"" + extra_quotes:
            value, index = literal_quoted_string(text, index, interpolating_marks)
            if value is None:
                return None, index
            values.append(value)
            while index < len(text) and text[index] in " \t":
                index += 1
            if index < len(text) and text[index] == ",":
                index += 1
                continue
            if index < len(text) and text[index] == "]" and bracketed:
                bracketed = False
                index += 1
                continue
            break
        if character in ");}\n" or (character == "]" and bracketed):
            break
        return None, index
    return (values or None), index


def launched_command_text(values):
    """一串字面参数：只有一个的是交给 shell 的命令串；几个的是逐词交的 argv，照 shell 引号连成一条命令。"""
    return values[0] if len(values) == 1 else shlex.join(values)


def string_spans(text, quotes, comment_start=None):
    """text 里引号括起来的区间 [(起, 止), …]（转义照认），comment_start 起到行尾的注释也算；扫语言代码时跳过这些区间里的字。"""
    spans, index = [], 0
    while index < len(text):
        character = text[index]
        if comment_start and character == comment_start and (index == 0 or text[index - 1] in " \t\n;{}("):
            end = text.find("\n", index)
            end = len(text) if end < 0 else end
            spans.append((index, end))
            index = end
            continue
        if character in quotes:
            _, end = literal_quoted_string(text, index, ())
            spans.append((index, end))
            index = end
            continue
        index += 1
    return spans


def inside_spans(position, spans):
    return any(start <= position < end for start, end in spans)


def scan_quoted_language(language, text, pattern, interpolating_marks, quotes, comment_start, texts, unjudged):
    """perl / ruby / php / node / lua 这类：代码里起命令的写法（pattern 认，引号里、注释里的不算）后面跟的字面参数照命令交出，不是字面的交说明。"""
    spans = string_spans(text, quotes, comment_start)
    for match in pattern.finditer(text):
        if inside_spans(match.start(), spans):
            continue
        word = match.group(0).rstrip("( \t")
        call_start = match.end() - 1 if match.group(0).endswith("(") else match.end()
        rest = match.end()
        while rest < len(text) and text[rest] in " \t":
            rest += 1
        following = text[rest] if rest < len(text) else ""
        if word in ("qx", "%x") or word.endswith("%x"):
            closing = {"{": "}", "(": ")", "[": "]", "<": ">"}.get(following, following)
            end = text.find(closing, rest + 1) if following else -1
            body = text[rest + 1:end] if end > rest else None
            if body is None or (following != "'" and any(mark in body for mark in interpolating_marks)):
                unjudged.append(f"{language} 代码里的 {word}{following}…{closing} 认不出起的是什么")
            else:
                texts.append(body)
            continue
        if not match.group(0).endswith("(") and following and following not in "(\"'`[" and not (language == "perl" and following in "$@{"):
            continue   # `print if /exec/`、`system =>` 这类不是调用
        if word == "open":
            values, _ = literal_argument_list(text, call_start, interpolating_marks)
            if values is None:
                arguments_text = text[call_start:call_start + 200]
                if "|" in arguments_text.split(";")[0]:
                    unjudged.append(f"{language} 代码里 open 的管道命令认不出")
                continue
            pipe_values = [value for value in values if value.startswith("|") or value.endswith("|")]
            if pipe_values:
                texts.append(pipe_values[0].strip("|").strip())
            elif any(value in ("-|", "|-") for value in values):
                rest_values = values[[value in ("-|", "|-") for value in values].index(True) + 1:]
                if rest_values:
                    texts.append(launched_command_text(rest_values))
                else:
                    unjudged.append(f"{language} 代码里 open 的管道命令认不出")
            continue
        values, _ = literal_argument_list(text, call_start, interpolating_marks, "`" if language == "node" else "")
        if values is None:
            unjudged.append(f"{language} 代码里的 {word}(…) 起的命令不是字面的，认不出起的是什么")
        else:
            texts.append(launched_command_text(values))
    if language in ("perl", "ruby", "php"):
        for start, _ in spans:
            if text[start] != "`":
                continue
            value, _ = literal_quoted_string(text, start, interpolating_marks)
            if value is None:
                unjudged.append(f"{language} 代码里反引号起的命令不是字面的，认不出起的是什么")
            else:
                texts.append(value)


def python_literal(node):
    """ast 节点是字面的字符串、或字面字符串组成的列表 / 元组时交它的值，别的交 None。"""
    import ast
    try:
        value = ast.literal_eval(node)
    except (ValueError, TypeError, SyntaxError, MemoryError, RecursionError):
        return None
    if isinstance(value, str):
        return value
    if isinstance(value, (list, tuple)) and value and all(isinstance(item, str) for item in value):
        return list(value)
    return None


class PythonUnresolvedElement:
    """argv 列表里不是字面的一项（python_command_value 交的列表里的占位）：照一段 shell 命令判时换成 PYTHON_UNRESOLVED_ARGUMENT。"""


PYTHON_UNRESOLVED_ELEMENT = PythonUnresolvedElement()


class PythonUnresolvedTail:
    """argv 列表里 `+ 名字`、`*名字` 接上的、算不出的一段（几项都可能；python_list_tail）：照一段 shell 命令判时同样换成 PYTHON_UNRESOLVED_ARGUMENT，
    命令词要过 python_tail_command_word_judged 才交出。"""


PYTHON_UNRESOLVED_TAIL = PythonUnresolvedTail()
# argv 后面接了算不出的一段时，命令词是这些的照认不出（参数里能带要跑的命令）：启动器（lib_project_code_runs.py 的 LAUNCHER_OPTIONS_WITH_VALUE 与 EXTRA_LAUNCHER_NAMES，
# bash-command-detector.sh 的自证核两边一致）与参数里带命令的程序；shell、解释器、前缀与包装另按各自的名单认（python_tail_command_word_judged）
ARGUMENT_COMMAND_LAUNCHER_NAMES = frozenset({"time", "flock", "chrt", "prlimit", "systemd-run", "strace", "perf", "rustup",
                                             "bwrap", "cgexec", "chroot", "cpulimit", "dbus-run-session", "doas", "faketime", "find", "firejail", "gdb", "ltrace",
                                             "nsenter", "numactl", "parallel", "proot", "runuser", "scl", "script", "setpriv", "sg", "ssh", "su", "unbuffer",
                                             "unshare", "valgrind", "watch", "xargs"})
ARGUMENT_COMMAND_PROGRAM_NAMES = frozenset({"sed", "git", "tar", "less", "busybox", "eval", "source", ".", "builtin"})
SCRIPT_FILE_SUFFIXES = (".sh", ".bash", ".py", ".pl", ".rb")
PYTHON_UNRESOLVED_ARGUMENT = '"$(unresolved-python-argument)"'   # argv 里不是字面的一项换成的词：恰好一个词、值算不出（落在命令词上、cargo 的参数上照认不出办）
PYTHON_READ_ONLY_BUILTINS = frozenset({"print", "len", "str", "repr", "list", "tuple", "sorted"})   # 把列表交给它们不会改列表
PYTHON_LIST_MUTATING_METHODS = frozenset({"append", "extend", "insert", "pop", "remove", "clear", "sort", "reverse", "__setitem__", "__delitem__", "__iadd__"})


def python_argument_variables_unresolved():
    """弄坏开关 LIB_SHELL_WORDS_BREAK=python-argument-variables-unresolved：python 里起命令的参数只认整个字面的（argv 里有一项是变量就整条认不出、赋过字面值的变量与
    字面串的 .split() 不认）。"""
    return words_break_is_set("python-argument-variables-unresolved")


def python_constant_bindings(tree, launch_call):
    """python 代码里赋过字面值、之后没变过的名字：交 {名字: 赋给它的值的 ast 节点}。整段代码里只绑定过一次（赋值、for / with / except / import / 函数参数 / 函数与类名 /
    global / match 的捕获都算绑定，作用域不分）、那一次是 `名字 = 值`（或带注解、海象）的才收；值是可变的（列表、.split() 的结果）的另要求没被改过：
    `名字[…] = …`、`del 名字[…]`、`名字.append(…)` 这类，或当参数交给 launch_call 认的起命令调用与 print / len / str.join 这类之外的函数，都算改过。"""
    import ast
    bindings = {}
    def bind(name, times=1):
        bindings[name] = bindings.get(name, 0) + times
    for node in ast.walk(tree):
        if isinstance(node, ast.Name) and isinstance(node.ctx, (ast.Store, ast.Del)):
            bind(node.id)
        elif isinstance(node, ast.arg):
            bind(node.arg)
        elif isinstance(node, (ast.Import, ast.ImportFrom)):
            for alias in node.names:
                bind((alias.asname or alias.name).split(".")[0])
        elif isinstance(node, (ast.Global, ast.Nonlocal)):
            for name in node.names:
                bind(name, 2)
        elif isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)):
            bind(node.name)
        elif isinstance(node, ast.ExceptHandler) and node.name:
            bind(node.name)
        elif isinstance(node, (ast.MatchAs, ast.MatchStar)) and node.name:
            bind(node.name)
        elif isinstance(node, ast.MatchMapping) and node.rest:
            bind(node.rest)
    values = {}
    for node in ast.walk(tree):
        if isinstance(node, ast.Assign) and len(node.targets) == 1 and isinstance(node.targets[0], ast.Name):
            values[node.targets[0].id] = node.value
        elif isinstance(node, (ast.AnnAssign, ast.NamedExpr)) and isinstance(node.target, ast.Name) and node.value is not None:
            values[node.target.id] = node.value
    constants = {name: value for name, value in values.items() if bindings.get(name) == 1}
    for node in ast.walk(tree):
        if isinstance(node, (ast.Subscript, ast.Attribute)) and isinstance(node.value, ast.Name):
            if isinstance(node.ctx, (ast.Store, ast.Del)) or (isinstance(node, ast.Attribute) and node.attr in PYTHON_LIST_MUTATING_METHODS):
                constants.pop(node.value.id, None)
        elif isinstance(node, ast.Call) and not launch_call(node):
            if ((isinstance(node.func, ast.Name) and node.func.id in PYTHON_READ_ONLY_BUILTINS and node.func.id not in bindings)
                    or (isinstance(node.func, ast.Attribute) and node.func.attr == "join")):
                continue
            for argument in list(node.args) + [keyword.value for keyword in node.keywords]:
                passed = argument.value if isinstance(argument, ast.Starred) else argument
                if isinstance(passed, ast.Name) and passed.id in constants and not isinstance(constants[passed.id], (ast.Constant, ast.Tuple)):
                    constants.pop(passed.id)
    return constants


def python_list_concatenation_unresolved():
    """弄坏开关 LIB_SHELL_WORDS_BREAK=python-list-concatenation-unresolved：argv 的 `+ 列表` 与 `*列表` 整条认不出。"""
    return words_break_is_set("python-list-concatenation-unresolved")


def python_list_tail(node, constants, modules, depth):
    """argv 列表里 `*名字` 与 `字面列表 + 名字` 接上的那一段：算得出是列表的照接上，算不出的换成一个 PYTHON_UNRESOLVED_TAIL（几项都可能）。"""
    value = python_command_value(node, constants, modules, depth)
    return list(value) if isinstance(value, list) else [PYTHON_UNRESOLVED_TAIL]


def python_tail_command_word_judged(word):
    """argv 后面接了算不出的一段（PYTHON_UNRESOLVED_TAIL）时，命令词能不能单独照命令判：字面的、不是 shell、解释器（python / perl / ruby / node / php / lua / tcl /
    awk / make / vim 与 ARGUMENT_COMMAND_PROGRAM_NAMES）、前缀与包装（PREFIX_OPTIONS_WITH_VALUE、WRAPPERS_TAKING_ONE_ARGUMENT）、启动器
    （ARGUMENT_COMMAND_LAUNCHER_NAMES），也不是脚本（SCRIPT_FILE_SUFFIXES 结尾）的才算；别的照认不出办（复跑分类报告 d137-replay-classify 二节 E6）。"""
    if not isinstance(word, str) or not word or set("$`") & set(word):
        return False
    name = os.path.basename(word)
    return not (is_shell_name(name) or PYTHON_NAME.match(name) or PERL_NAME.match(name) or RUBY_NAME.match(name) or PHP_NAME.match(name)
                or LUA_NAME.match(name) or TCL_NAME.match(name) or name in NODE_NAMES or name in AWK_NAMES or name in MAKE_NAMES or name in VIM_NAMES
                or name in ARGUMENT_COMMAND_PROGRAM_NAMES or name in PREFIX_OPTIONS_WITH_VALUE or name in WRAPPERS_TAKING_ONE_ARGUMENT
                or name in ARGUMENT_COMMAND_LAUNCHER_NAMES or name.endswith(SCRIPT_FILE_SUFFIXES))


def python_command_value(node, constants, modules, depth=0):
    """起命令的参数的值：字面的 str 交 str；列表 / 元组交 list，里面不是字面 str 的项换成 PYTHON_UNRESOLVED_ELEMENT；
    赋过字面值的名字（python_constant_bindings）、字面串的 `.split()` 与 `shlex.split(字面串)` 照算出的值；
    头一项不是展开的列表里的 `*名字` 与 `列表 + 名字` 照 python_list_tail 接上（复跑分类报告 d137-replay-classify 二节 E6；弄坏开关 python-list-concatenation-unresolved
    下整条认不出）；别的交 None（认不出）。"""
    import ast
    if node is None or depth > EMBEDDED_NESTING:
        return None
    if isinstance(node, ast.Constant) and isinstance(node.value, str):
        return node.value
    if isinstance(node, ast.Name) and node.id in constants:
        return python_command_value(constants[node.id], constants, modules, depth + 1)
    if isinstance(node, (ast.List, ast.Tuple)):
        if not node.elts or isinstance(node.elts[0], ast.Starred) or (
                python_list_concatenation_unresolved() and any(isinstance(element, ast.Starred) for element in node.elts)):
            return None
        values = []
        for element in node.elts:
            if isinstance(element, ast.Starred):
                values += python_list_tail(element.value, constants, modules, depth + 1)
                continue
            value = python_command_value(element, constants, modules, depth + 1)
            values.append(value if isinstance(value, str) else PYTHON_UNRESOLVED_ELEMENT)
        return values
    if isinstance(node, ast.BinOp) and isinstance(node.op, ast.Add) and not python_list_concatenation_unresolved():
        head = python_command_value(node.left, constants, modules, depth + 1)
        if not isinstance(head, list) or not head or not isinstance(head[0], str):
            return None
        return head + python_list_tail(node.right, constants, modules, depth + 1)
    if isinstance(node, ast.Call) and isinstance(node.func, ast.Attribute) and node.func.attr == "split" and not node.keywords and len(node.args) <= 1:
        if isinstance(node.func.value, ast.Name) and modules.get(node.func.value.id) == "shlex":
            text = python_command_value(node.args[0], constants, modules, depth + 1) if node.args else None
            if not isinstance(text, str):
                return None
            try:
                return shlex.split(text)
            except ValueError:
                return None
        text = python_command_value(node.func.value, constants, modules, depth + 1)
        separator = python_command_value(node.args[0], constants, modules, depth + 1) if node.args else None
        if isinstance(text, str) and (not node.args or (isinstance(separator, str) and separator)):
            return text.split(separator) if node.args else text.split()
    return None


def python_argv_text(argv):
    """argv（python_command_value 的列表）照一段 shell 命令写出：字面的项照 shlex 加引号，不是字面的项换成 PYTHON_UNRESOLVED_ARGUMENT。"""
    return " ".join(PYTHON_UNRESOLVED_ARGUMENT if element is PYTHON_UNRESOLVED_ELEMENT or element is PYTHON_UNRESOLVED_TAIL else shlex.quote(element)
                    for element in argv)


def scan_python(text, texts, unjudged, depth=0):
    """python 代码（-c 的代码串、喂进标准输入的）里起命令的调用：os.system / os.popen / subprocess（shell=True 的照命令串）/ os.exec* / os.spawn* / pty.spawn 这类，
    命令串、argv 照 python_command_value 取值（字面的、赋过字面值没变过的名字、字面串的 .split()）：命令串要整个算得出，argv 里算不出的参数照一个值算不出的词交出
    （落在命令词上调用方照认不出办）；整个认不出的、经 __import__ / importlib / getattr / exec / eval 动态取的、用了 ctypes 的交说明。语法错的代码 python 自己不跑，带起命令字样的照认不出办。"""
    import ast
    try:
        tree = ast.parse(text)
    except (SyntaxError, ValueError, MemoryError, RecursionError):
        if any(hint in text for hint in EMBEDDED_PYTHON_SYNTAX_HINTS):
            unjudged.append("python 代码解析不了、里面带起命令的字样，认不出起的是什么")
        return
    modules, functions = {}, {}
    for node in ast.walk(tree):
        if isinstance(node, ast.Import):
            for alias in node.names:
                top = alias.name.split(".")[0]
                modules[alias.asname or top] = alias.name if alias.asname else top
                if top in EMBEDDED_PYTHON_OPAQUE_MODULES:
                    unjudged.append(f"python 代码用了 {top}，能直接调 libc 起命令，认不出起的是什么")
        elif isinstance(node, ast.ImportFrom) and node.module:
            top = node.module.split(".")[0]
            if top in EMBEDDED_PYTHON_OPAQUE_MODULES:
                unjudged.append(f"python 代码用了 {top}，能直接调 libc 起命令，认不出起的是什么")
            for alias in node.names:
                if alias.name == "*":
                    for name in EMBEDDED_PYTHON_DISTINCT_LAUNCH_NAMES | EMBEDDED_PYTHON_SUBPROCESS_CALLS | {"spawn"}:
                        functions.setdefault(name, (top, name))
                else:
                    functions[alias.asname or alias.name] = (top, alias.name)
    def module_of(value):
        if isinstance(value, ast.Name):
            return modules.get(value.id)
        if (isinstance(value, ast.Call) and value.args and isinstance(value.args[0], ast.Constant) and isinstance(value.args[0].value, str)
                and ((isinstance(value.func, ast.Name) and value.func.id == "__import__")
                     or (isinstance(value.func, ast.Attribute) and value.func.attr == "import_module"))):
            return value.args[0].value.split(".")[0]
        return None
    def launch_call(call):
        if isinstance(call.func, ast.Attribute):
            return module_of(call.func.value) in EMBEDDED_PYTHON_LAUNCH_MODULES
        return isinstance(call.func, ast.Name) and call.func.id in functions and functions[call.func.id][0] in EMBEDDED_PYTHON_LAUNCH_MODULES
    literal_only = python_argument_variables_unresolved()
    constants = {} if literal_only else python_constant_bindings(tree, launch_call)
    def value_of(node):
        return python_literal(node) if literal_only else python_command_value(node, constants, modules)
    def argv_elements(values):
        if literal_only:
            return values if values and all(isinstance(value, str) for value in values) else None
        return [value if isinstance(value, str) else PYTHON_UNRESOLVED_ELEMENT for value in values] or None
    for node in ast.walk(tree):
        if not isinstance(node, ast.Call):
            continue
        target = None
        if isinstance(node.func, ast.Attribute):
            owner = module_of(node.func.value)
            if owner is not None:
                target = (owner, node.func.attr)
            elif node.func.attr in EMBEDDED_PYTHON_DISTINCT_LAUNCH_NAMES:
                unjudged.append(f"python 代码里的 ….{node.func.attr}(…) 认不出是哪个模块的、起的是什么")
                continue
        elif isinstance(node.func, ast.Name):
            name = node.func.id
            if name in functions:
                target = functions[name]
            elif name in ("exec", "eval", "compile"):
                code = python_literal(node.args[0]) if node.args else None
                if isinstance(code, str) and depth < EMBEDDED_NESTING:
                    scan_python(code, texts, unjudged, depth + 1)
                elif node.args and (name != "compile" or isinstance(code, str) or words_break_is_set("python-compile-as-execution")):
                    # 只调 compile、代码不是字面的，不当起命令（复跑分类报告 d137-replay-classify 二节 E7：compile 只编不跑，编出来的交给 exec / eval 时由那一处照判）；
                    # 字面的代码照旧扫。弄坏开关 LIB_SHELL_WORDS_BREAK=python-compile-as-execution 下 compile 照执行算
                    unjudged.append(f"python 代码里 {name}(…) 的代码不是字面的，认不出起的是什么")
                continue
            elif name in ("__import__", "getattr") and node.args and not (name == "__import__" and python_literal(node.args[0]) is not None):
                if name == "__import__" or module_of(node.args[0]) in EMBEDDED_PYTHON_LAUNCH_MODULES:
                    unjudged.append(f"python 代码里经 {name}(…) 动态取模块或函数，认不出起的是什么")
                continue
        if target is None:
            continue
        module, function = target
        if module == "importlib" and function == "import_module" and node.args and python_literal(node.args[0]) is None:
            unjudged.append("python 代码里经 importlib.import_module 动态取模块，认不出起的是什么")
            continue
        if module not in EMBEDDED_PYTHON_LAUNCH_MODULES:
            continue
        positional = node.args
        if any(isinstance(argument, ast.Starred) for argument in positional):
            unjudged.append(f"python 代码里 {module}.{function}(*…) 的参数是展开的，认不出起的是什么")
            continue
        keywords = {keyword.arg: keyword.value for keyword in node.keywords}
        if (module, function) in EMBEDDED_PYTHON_SHELL_CALLS:
            command = value_of(positional[0]) if positional else value_of(keywords.get("cmd")) if "cmd" in keywords else None
            if isinstance(command, str):
                texts.append(command)
            else:
                unjudged.append(f"python 代码里 {module}.{function}(…) 的命令不是字面的，认不出起的是什么")
            continue
        argv = None
        if module == "subprocess" and function in EMBEDDED_PYTHON_SUBPROCESS_CALLS:
            shell = keywords.get("shell")
            arguments = value_of(positional[0]) if positional else value_of(keywords["args"]) if "args" in keywords else None
            shell_value = None if shell is None else python_literal(shell) if not isinstance(shell, ast.Constant) else shell.value
            if shell is not None and not isinstance(shell_value, bool):
                unjudged.append(f"python 代码里 subprocess.{function}(…) 的 shell= 不是字面的，认不出起的是什么")
                continue
            if arguments is None:
                unjudged.append(f"python 代码里 subprocess.{function}(…) 的命令不是字面的，认不出起的是什么")
                continue
            if shell_value:
                # shell=True：命令串（列表的头一项）整个交给 sh -c，里面有一处算不出就认不出
                command_text = arguments if isinstance(arguments, str) else arguments[0]
                if isinstance(command_text, str):
                    texts.append(command_text)
                else:
                    unjudged.append(f"python 代码里 subprocess.{function}(…, shell=True) 的命令串不是字面的，认不出起的是什么")
                continue
            argv = [arguments] if isinstance(arguments, str) else arguments
        elif module in ("os", "posix") and function in EMBEDDED_PYTHON_EXEC_LIST_CALLS | EMBEDDED_PYTHON_SPAWN_LIST_CALLS:
            offset = 1 if function.startswith("spawn") else 0
            values = [value_of(argument) for argument in positional[offset:]]
            if function.endswith("e"):
                values = values[:-1]
            values = argv_elements(values)
            if values:
                argv = [values[0]] + values[2:]
        elif module in ("os", "posix") and function in EMBEDDED_PYTHON_EXEC_VECTOR_CALLS | EMBEDDED_PYTHON_SPAWN_VECTOR_CALLS | {"posix_spawn", "posix_spawnp"}:
            offset = 1 if function.startswith("spawn") else 0
            program = value_of(positional[offset]) if len(positional) > offset else None
            vector = value_of(positional[offset + 1]) if len(positional) > offset + 1 else None
            if isinstance(vector, list) and len(positional) > offset and (isinstance(program, str) or not literal_only):
                argv = [program if isinstance(program, str) else PYTHON_UNRESOLVED_ELEMENT] + vector[1:]
        elif module == "pty" and function == "spawn":
            value = value_of(positional[0]) if positional else None
            argv = [value] if isinstance(value, str) else value
        elif module == "asyncio" and function == "create_subprocess_exec":
            argv = argv_elements([value_of(argument) for argument in positional])
        else:
            continue
        if argv and any(element is PYTHON_UNRESOLVED_TAIL for element in argv) and not python_tail_command_word_judged(argv[0]):
            unjudged.append(f"python 代码里 {module}.{function}(…) 的命令词是 shell、解释器、启动器或脚本（或不是字面的），后面接上的参数算不出，认不出起的是什么")
        elif argv:
            texts.append(python_argv_text(argv))
        else:
            unjudged.append(f"python 代码里 {module}.{function}(…) 起的命令不是字面的，认不出起的是什么")


def tcl_words(command_text):
    """一条 tcl 命令切成词：{…} 照字面（可以套）；"…" 与光词里有 $ 或 [ … ] 的不是字面（那个词交 None）；交 (词表, [命令替换 [ … ] 里的代码…])。"""
    words, substitutions, index = [], [], 0
    while index < len(command_text):
        while index < len(command_text) and command_text[index] in " \t":
            index += 1
        if index >= len(command_text):
            break
        if command_text[index] == "{":
            level, end = 0, index
            while end < len(command_text):
                if command_text[end] == "\\":
                    end += 2
                    continue
                level += {"{": 1, "}": -1}.get(command_text[end], 0)
                if level == 0:
                    break
                end += 1
            words.append(command_text[index + 1:end])
            index = end + 1
            continue
        quoted = command_text[index] == '"'
        end = index + 1 if quoted else index
        value, literal = [], True
        while end < len(command_text):
            current = command_text[end]
            if not quoted and current in " \t":
                break
            if quoted and current == '"':
                end += 1
                break
            if current == "\\" and end + 1 < len(command_text):
                value.append(command_text[end + 1])
                end += 2
                continue
            if current == "$":
                literal = False
            if current == "[":
                level, inner = 0, end
                while inner < len(command_text):
                    level += {"[": 1, "]": -1}.get(command_text[inner], 0)
                    if level == 0:
                        break
                    inner += 1
                substitutions.append(command_text[end + 1:inner])
                literal = False
                end = inner + 1
                continue
            value.append(current)
            end += 1
        words.append("".join(value) if literal else None)
        index = end
    return words, substitutions


def scan_tcl(text, texts, unjudged, depth=0):
    """tcl（tclsh、wish、expect）代码：exec / spawn / system 这几条命令的词都是字面的照 argv 交出（exec 的 -ignorestderr、-keepnewline、-- 跳过，`|` `<` `>` `&` 起的重定向与后台之后不要），
    open 的 `|命令` 照命令交；词里有变量或命令替换的、eval / uplevel / subst / interp / source 动态起代码的交说明。命令替换 [ … ] 里的代码接着判。"""
    for line in re.split(r"[\n;]", text):
        stripped = line.strip()
        if not stripped or stripped.startswith("#"):
            continue
        words, substitutions = tcl_words(stripped)
        if depth < EMBEDDED_NESTING:
            for substitution in substitutions:
                scan_tcl(substitution, texts, unjudged, depth + 1)
        if not words or words[0] is None:
            continue
        name = words[0]
        if name in TCL_OPAQUE_COMMANDS:
            unjudged.append(f"tcl 代码里的 {name} 动态起代码，认不出起的是什么")
            continue
        if name == "open" and len(words) > 1:
            if words[1] is None:
                unjudged.append("tcl 代码里 open 的目标不是字面的，认不出是不是起命令")
            elif words[1].startswith("|"):
                texts.append(words[1][1:].strip())
            continue
        if name not in TCL_LAUNCH_COMMANDS:
            continue
        arguments = words[1:]
        while arguments and arguments[0] in ("-ignorestderr", "-keepnewline", "-noecho", "-nottycopy", "-nottyinit", "-console", "-pty", "-open", "-leaveopen"):
            arguments = arguments[1:]
        if arguments and arguments[0] == "--":
            arguments = arguments[1:]
        kept = []
        for argument in arguments:
            if argument is not None and (argument in ("|", "&") or argument.startswith(("<", ">", "2>", "|"))):
                break
            kept.append(argument)
        if not kept or any(argument is None for argument in kept):
            unjudged.append(f"tcl 代码里的 {name} 起的命令不是字面的，认不出起的是什么")
        elif name in ("system", "exp_system"):
            texts.append(" ".join(kept))
        else:
            texts.append(shlex.join(kept))


def awk_tokens(text):
    """awk 程序切成 (种类, 值)：string（双引号串的值）、regex、name、number、punct；# 注释与换行之外的空白去掉。斜杠在值或右括号之后是除号，别处是正则。"""
    tokens, index = [], 0
    while index < len(text):
        character = text[index]
        if character in " \t\r":
            index += 1
            continue
        if character == "\\" and text[index + 1:index + 2] == "\n":
            index += 2
            continue
        if character == "#":
            end = text.find("\n", index)
            index = len(text) if end < 0 else end
            continue
        if character == '"':
            value, end = literal_quoted_string(text, index, ())
            tokens.append(("string", value if value is not None else ""))
            index = end
            continue
        if character == "/" and (not tokens or tokens[-1][0] == "punct" and tokens[-1][1] not in (")", "]")
                                 or tokens[-1][0] == "name" and tokens[-1][1] in ("print", "printf", "return", "in", "getline")):
            end = index + 1
            while end < len(text) and text[end] != "/" and text[end] != "\n":
                end += 2 if text[end] == "\\" else 1
            tokens.append(("regex", text[index + 1:end]))
            index = end + 1
            continue
        if character.isalpha() or character == "_":
            end = index
            while end < len(text) and (text[end].isalnum() or text[end] == "_"):
                end += 1
            tokens.append(("name", text[index:end]))
            index = end
            continue
        if character.isdigit():
            end = index
            while end < len(text) and (text[end].isalnum() or text[end] == "."):
                end += 1
            tokens.append(("number", text[index:end]))
            index = end
            continue
        for operator in ("|&", "||", "&&", "==", "!=", "<=", ">=", ">>", "++", "--", "+=", "-=", "*=", "/=", "%=", "^=", "**"):
            if text.startswith(operator, index):
                tokens.append(("punct", operator))
                index += len(operator)
                break
        else:
            tokens.append(("punct", character))
            index += 1
    return tokens


AWK_ASSIGNMENT_ENDS = (";", "}", "\n")
AWK_CHANGING_OPERATORS = ("+=", "-=", "*=", "/=", "%=", "^=", "++", "--", "[")
AWK_TARGET_BUILTINS = frozenset({"sub", "gsub", "split", "patsplit", "match"})   # 参数里的变量会被它们改写（替换的目标、分出来的数组）


def awk_command_variables_unresolved():
    """弄坏开关 LIB_SHELL_WORDS_BREAK=awk-command-variables-unresolved：awk 里当命令用的变量一律认不出（`cmd = "date"; cmd | getline`、命令行 `cmd=sort` 再
    `print | cmd` 都照写成变量拒）。"""
    return words_break_is_set("awk-command-variables-unresolved")


def awk_command_variables(tokens, assignments=()):
    """awk 程序里变量能取到的字面命令串：交 {名字: [值, …]}。程序里每一处赋值都是 `名字 = "字面串"`（后面接 ; } 换行或程序末尾）、命令行给的（-v 与操作数里的 `名字=值`，
    assignments 里 (名字, 值)，值是 None 的是写成了 shell 变量或命令替换）都是字面的，而别处没有改写它的写法（拼接赋值、+= 这类、++ / --、getline 名字、数组、for … in、
    sub / gsub / split / patsplit / match 的参数、函数参数）才收；收不了的名字不在里面。"""
    literal, broken = {}, set()
    for name, value in assignments:
        if value is None:
            broken.add(name)
        else:
            literal.setdefault(name, []).append(value)
    target_depths, depth = [], 0
    for position, (kind, value) in enumerate(tokens):
        before = tokens[position - 1] if position > 0 else None
        after = tokens[position + 1] if position + 1 < len(tokens) else None
        if (kind, value) == ("punct", "("):
            depth += 1
            if before is not None and before[0] == "name" and before[1] in AWK_TARGET_BUILTINS:
                target_depths.append(depth)
            continue
        if (kind, value) == ("punct", ")"):
            if target_depths and target_depths[-1] == depth:
                target_depths.pop()
            depth -= 1
            continue
        if kind != "name":
            continue
        if target_depths:
            broken.add(value)
        if before is not None and before[0] == "name" and before[1] in ("function", "func") and after == ("punct", "("):
            end = position + 2
            while end < len(tokens) and tokens[end] != ("punct", ")"):
                if tokens[end][0] == "name":
                    broken.add(tokens[end][1])
                end += 1
        if before == ("punct", "$"):
            continue   # $名字 是字段，不是这个变量本身
        if after == ("punct", "="):
            assigned = tokens[position + 2] if position + 2 < len(tokens) else None
            ending = tokens[position + 3] if position + 3 < len(tokens) else None
            if assigned is not None and assigned[0] == "string" and (ending is None or (ending[0] == "punct" and ending[1] in AWK_ASSIGNMENT_ENDS)):
                literal.setdefault(value, []).append(assigned[1])
            else:
                broken.add(value)
        elif ((after is not None and after[0] == "punct" and after[1] in AWK_CHANGING_OPERATORS) or after == ("name", "in")
              or before in (("punct", "++"), ("punct", "--"), ("name", "getline"))):
            broken.add(value)
    return {name: values for name, values in literal.items() if name not in broken}


def scan_awk(text, texts, unjudged, assignments=()):
    """awk 程序：system(…)、print … | …、… | getline、|& 协进程里的命令是字面字符串、或是赋过字面串而没改写过的变量（awk_command_variables；assignments 是命令行给的
    `名字=值`）的照命令交出（变量取到的每个值都交）；写成别的变量、拼接的交说明。"""
    tokens = awk_tokens(text)
    variables = {} if awk_command_variables_unresolved() else awk_command_variables(tokens, assignments)
    def command_values(token):
        if token is not None and token[0] == "string":
            return [token[1]]
        if token is not None and token[0] == "name" and token[1] in variables:
            return variables[token[1]]
        return None
    for position, (kind, value) in enumerate(tokens):
        following = tokens[position + 1:position + 4]
        if kind == "name" and value == "system" and following[:1] == [("punct", "(")]:
            values = command_values(following[1]) if len(following) >= 3 and following[2] == ("punct", ")") else None
            if values is not None:
                texts.extend(values)
            else:
                unjudged.append("awk 程序里 system(…) 的命令不是字面字符串，认不出起的是什么")
        elif kind == "punct" and value in ("|", "|&"):
            if following[:1] == [("name", "getline")]:
                previous = tokens[position - 1] if position > 0 else None
                before = tokens[position - 2] if position > 1 else None
                values = command_values(previous)
                if values is not None and (before is None or before[0] == "punct" and before[1] not in (")", "$")):
                    texts.extend(values)
                else:
                    unjudged.append("awk 程序里 … | getline 的命令不是字面字符串，认不出起的是什么")
            elif (following[:1] and command_values(following[0]) is not None
                  and (len(following) < 2 or following[1][0] == "punct" and following[1][1] in (";", "}", ")", "\n"))):
                texts.extend(command_values(following[0]))
            else:
                unjudged.append("awk 程序里管道 | 到的命令不是字面字符串，认不出起的是什么")


SED_SUBSTITUTION_ADDRESS = "\x01"   # sed 脚本词里的命令替换（`$(…)`、反引号）与变量（`${名字}`、`$名字`）展开之后换成的占位：只认在行号地址位上（sed_script_parse）
SED_ARITHMETIC_VALUE = "1"          # 算术展开（`$((…))`、`$[…]`）换成的值：结果只能是整数，哪个整数都不改 sed 脚本的切法（0 与负数让 sed 报错、什么都不跑）
SED_EXPANSION_LIMIT = 6             # 一份脚本里没展开的展开超过这么多处不逐个组合（2 的这么多次方趟），照「解析不下去」办
GNU_SED_COMMAND_CHARACTERS = frozenset("{}=:#!abcdDeFgGhHilLnNpPqQrRstTvwWxyz")   # GNU sed 认得的命令字：解析卡在这些字上是这里没读懂，不是 sed 一定报错
# GNU sed 的 s 命令标志区认得的字（compile.c 的 mark_subst_opts；空白实测 GNU sed 4.9 照认）：别的字编译就报 unknown option to `s'，整个脚本一行都不跑
SED_SUBSTITUTE_FLAG_CHARACTERS = frozenset("gpeiImM0123456789 \t\r")
SED_VARIABLE = re.compile(r"\$(?:\{[A-Za-z_][A-Za-z0-9_]*\}|[A-Za-z_][A-Za-z0-9_]*|[0-9#?$!*@-])")


def sed_expansions_unparsed():
    """弄坏开关 LIB_SHELL_WORDS_BREAK=sed-expansions-unparsed：sed 脚本词里没展开的 bash 展开不另算，原样切、切不下去照「解析不下去」。"""
    return words_break_is_set("sed-expansions-unparsed")


def sed_text_first_line_only():
    """弄坏开关 LIB_SHELL_WORDS_BREAK=sed-text-first-line-only：a / i / c 的文本只读到头一个换行（`a\\` 换行之后的文本行被当命令读）。"""
    return words_break_is_set("sed-text-first-line-only")


class SedParse(NamedTuple):
    texts: list              # e 命令后面的字面命令
    unjudged: list           # 起命令而认不出的说明（单独的 e、s///e、解析不下去）
    failed_at: str | None    # 解析不下去时卡在的那个字（脚本读完了还缺东西是 ""）；切完了是 None
    misplaced: bool          # 占位 SED_SUBSTITUTION_ADDRESS 落在了行号地址位之外（正则地址、文本、s 的正文这类）


def sed_script_expansions(script):
    """sed 脚本词里没展开的 bash 展开（切词时引号已去掉、分不出原来在单引号里还是双引号里）：交 [(起, 止, 种类)]，种类 "arithmetic"（`$((…))`、`$[…]`）、
    "substitution"（`$(…)`、反引号）、"variable"（`${名字}`、`$名字`、`$1` 这类）；有认不出的（没配上的、算术里套命令替换、`${名字:-…}` 这类写法）交 None。"""
    found, index = [], 0
    while index < len(script):
        if script.startswith("$((", index):
            end = command_substitution_end(script, index + 2)
            if end < 0 or "$(" in script[index + 3:end] or "`" in script[index + 3:end]:
                return None
            found.append((index, end, "arithmetic"))
            index = end
        elif script.startswith("$[", index):
            depth, end = 0, index + 1
            while end < len(script):
                depth += 1 if script[end] == "[" else -1 if script[end] == "]" else 0
                end += 1
                if depth == 0:
                    break
            if depth != 0 or "$(" in script[index + 2:end] or "`" in script[index + 2:end]:
                return None
            found.append((index, end, "arithmetic"))
            index = end
        elif script.startswith("$(", index) or script[index] == "`":
            end = command_substitution_end(script, index + 2) if script[index] == "$" else backquote_end(script, index + 1)
            if end < 0:
                return None
            found.append((index, end, "substitution"))
            index = end
        elif script.startswith("${", index):
            variable = SED_VARIABLE.match(script, index)
            if variable is None:
                return None
            found.append((index, variable.end(), "variable"))
            index = variable.end()
        elif script[index] == "$" and SED_VARIABLE.match(script, index):
            end = SED_VARIABLE.match(script, index).end()
            found.append((index, end, "variable"))
            index = end
        else:
            index += 1
    return found


def sed_expanded_variants(script):
    """sed 脚本词里有没展开的 bash 展开、原样切不下去时（sed_script_commands）：每一处按「照字面留着」「照 bash 展开」两种各切一遍（单引号里的是字面、双引号里的展开，切词时分不出），
    算术展开换成 SED_ARITHMETIC_VALUE，命令替换与变量换成占位 SED_SUBSTITUTION_ADDRESS（展开的值当行号读）。
    每一种组合要么切得完、占位都在行号地址位上，要么卡在 GNU sed 没有的命令字上（sed 编译就报错、什么都不跑）；都是的交 (字面命令列表, 认不出的说明列表)，有一种不是的交 None。"""
    import itertools
    pieces = sed_script_expansions(script)
    if not pieces or len(pieces) > SED_EXPANSION_LIMIT:
        return None
    texts, unjudged = [], []
    for choice in itertools.product((False, True), repeat=len(pieces)):
        rebuilt, last = [], 0
        for (start, end, kind), expanded in zip(pieces, choice):
            rebuilt.append(script[last:start])
            rebuilt.append((SED_ARITHMETIC_VALUE if kind == "arithmetic" else SED_SUBSTITUTION_ADDRESS) if expanded else script[start:end])
            last = end
        rebuilt.append(script[last:])
        parsed = sed_script_parse("".join(rebuilt))
        if parsed.failed_at is not None:
            if parsed.failed_at == "" or parsed.failed_at in GNU_SED_COMMAND_CHARACTERS or parsed.failed_at == SED_SUBSTITUTION_ADDRESS:
                return None
            continue
        if parsed.misplaced:
            return None
        texts += parsed.texts
        unjudged += parsed.unjudged
    return texts, unjudged


def sed_script_commands(script, texts, unjudged, shell_word=False):
    """GNU sed 脚本里起的命令（sed_script_parse）交进 texts / unjudged。shell_word：脚本是命令行上的一个词（-e 的值、头一个参数），里面可能留着没展开的 bash 展开；
    原样切不下去时照 sed_expanded_variants 逐种展开再切（弄坏开关 LIB_SHELL_WORDS_BREAK=sed-expansions-unparsed 下不另算）。sed -f 读的文件里没有 bash 展开，不给。"""
    parsed = sed_script_parse(script)
    if parsed.failed_at is not None and shell_word and not sed_expansions_unparsed():
        variants = sed_expanded_variants(script)
        if variants is not None:
            texts.extend(variants[0])
            unjudged.extend(variants[1])
            return
    texts.extend(parsed.texts)
    unjudged.extend(parsed.unjudged)


def sed_script_parse(script):
    """GNU sed 脚本：e 命令后面到脚本末尾的文字照命令交出（实测 GNU sed 4.9 连换行一起交给 sh）；单独的 e、s///e 执行模式空间里的文字（内容是数据，认不出）交说明；
    a / i / c 读文本（`a\\` 换行之后的文本行、行末带 `\\` 的续行都算文本，到不续行的换行为止），r / R / w / W / : / b / t / T / # 读到行尾，s 与 y 照分隔符读。
    认不出的写法（解析不下去）交说明，SedParse.failed_at 记卡在哪个字。"""
    texts, unjudged = [], []
    index, placeholders_in_addresses = 0, 0
    def finished(failed_at=None):
        return SedParse(texts, unjudged, failed_at, placeholders_in_addresses != script.count(SED_SUBSTITUTION_ADDRESS))
    def skip_spaces(position):
        while position < len(script) and script[position] in " \t":
            position += 1
        return position
    def read_address(position):
        nonlocal placeholders_in_addresses
        if position < len(script) and script[position] == SED_SUBSTITUTION_ADDRESS:
            # 占位单独占一个行号地址（前后不接数字，后面可以接 ~步长）：展开的值整个当这一个行号读
            placeholders_in_addresses += 1
            position += 1
            if position < len(script) and script[position] == "~":
                position += 1
                while position < len(script) and script[position].isdigit():
                    position += 1
        elif position < len(script) and script[position].isdigit():
            while position < len(script) and (script[position].isdigit() or script[position] == "~"):
                position += 1
        elif position < len(script) and script[position] == "$":
            position += 1
        elif position < len(script) and script[position] in "/\\":
            delimiter = script[position] if script[position] == "/" else script[position + 1:position + 2]
            position += 1 if script[position] == "/" else 2
            while position < len(script) and script[position] != delimiter:
                position += 2 if script[position] == "\\" else 1
            position += 1
            while position < len(script) and script[position] in "IM":
                position += 1
        return position
    while index < len(script):
        index = skip_spaces(index)
        if index >= len(script):
            break
        if script[index] in ";\n}":
            index += 1
            continue
        index = read_address(index)
        index = skip_spaces(index)
        if index < len(script) and script[index] == ",":
            index = skip_spaces(index + 1)
            if index < len(script) and script[index] in "+~":
                index += 1
            index = read_address(index)
        index = skip_spaces(index)
        while index < len(script) and script[index] == "!":
            index = skip_spaces(index + 1)
        if index >= len(script):
            break
        command = script[index]
        index += 1
        if command == "e":
            rest = script[index:].strip()
            if rest:
                texts.append(rest)
            else:
                unjudged.append("sed 脚本里单独的 e 把模式空间里的文字当命令执行，认不出起的是什么")
            return finished()
        if command in ":btTv":
            while index < len(script) and script[index] not in ";\n}":
                index += 1
            continue
        if command in "aic" and not sed_text_first_line_only():
            # 文本从命令字之后起（`a\\` 紧跟换行的，文本在下一行）；行末的 `\\` 把下一行接进文本，到不续行的换行为止
            index = skip_spaces(index)
            if script.startswith("\\\n", index):
                index += 2
            while index < len(script) and script[index] != "\n":
                index += 2 if script[index] == "\\" else 1
            index += 1
            continue
        if command in "aicrRwW#":
            end = script.find("\n", index)
            index = len(script) if end < 0 else end + 1
            continue
        if command in "sy":
            if index >= len(script):
                unjudged.append("sed 脚本解析不下去，认不出里面起不起命令")
                return finished("")
            delimiter = script[index]
            index += 1
            for _ in range(2):
                while index < len(script) and script[index] != delimiter:
                    index += 2 if script[index] == "\\" else 1
                index += 1
            if command == "s":
                flags_start = index
                unknown_flags_as_execution = words_break_is_set("sed-unknown-flags-as-execution")
                while index < len(script) and script[index] not in (";\n}w" if unknown_flags_as_execution else ";\n}w#"):
                    index += 1
                if not unknown_flags_as_execution and set(script[flags_start:index]) - SED_SUBSTITUTE_FLAG_CHARACTERS:
                    # 标志区有 GNU sed 不认的字（复跑分类报告 d137-replay-classify 二节 E8：分隔符被转义的反斜杠提前收尾，后面的字落进标志区）：编译就报错、一行都不跑
                    # （实测 GNU sed 4.9 报 unknown option to `s'、不读输入）；弄坏开关 LIB_SHELL_WORDS_BREAK=sed-unknown-flags-as-execution 下区里有 e 就算 e 标志
                    texts.clear()
                    unjudged.clear()
                    return finished()
                if "e" in script[flags_start:index]:
                    unjudged.append("sed 脚本里 s///e 把替换之后的模式空间当命令执行，认不出起的是什么")
                if index < len(script) and script[index] == "w":
                    end = script.find("\n", index)
                    index = len(script) if end < 0 else end + 1
            continue
        if command in "{=dDgGhHxnNpPlLqQzF":
            while index < len(script) and script[index].isdigit():
                index += 1
            continue
        unjudged.append("sed 脚本解析不下去，认不出里面起不起命令")
        return finished(command)
    return finished()


def make_expanded(text, variables, depth, shell_texts, unjudged):
    """make 文本里的变量引用照这份 Makefile 与命令行上的定义展开：$$ 是 $；$(shell …) 里的命令交进 shell_texts、结果换成占位；别的函数与展开不了的变量换成没展开的 shell 变量
    （落在命令词上调用方照认不出办）；自动变量 $@ $< 这类换成占位。"""
    output, index = [], 0
    while index < len(text):
        character = text[index]
        if character != "$" or index + 1 >= len(text):
            output.append(character)
            index += 1
            continue
        following = text[index + 1]
        if following == "$":
            output.append("$$")
            index += 2
            continue
        if following in "({":
            closing = ")" if following == "(" else "}"
            level, end = 0, index + 1
            while end < len(text):
                level += 1 if text[end] == following else -1 if text[end] == closing else 0
                if level == 0:
                    break
                end += 1
            inner = text[index + 2:end]
            index = end + 1
            name_match = re.match(r"^([A-Za-z_][A-Za-z0-9_.\-]*)$", inner)
            if inner.startswith("shell ") or inner.startswith("shell\t"):
                shell_texts.append(make_expanded(inner[6:], variables, depth + 1, shell_texts, unjudged) if depth < EMBEDDED_NESTING else inner[6:])
                output.append("$" + MAKE_VARIABLE_PLACEHOLDER_PREFIX + "SHELL_RESULT")
            elif name_match and name_match.group(1) in variables and depth < EMBEDDED_NESTING:
                output.append(make_expanded(variables[name_match.group(1)], variables, depth + 1, shell_texts, unjudged))
            elif name_match:
                output.append("$" + MAKE_VARIABLE_PLACEHOLDER_PREFIX + re.sub(r"[^A-Za-z0-9_]", "_", name_match.group(1)))
            else:
                if depth < EMBEDDED_NESTING:
                    make_expanded(inner, variables, depth + 1, shell_texts, unjudged)   # 函数参数里的 $(shell …) 照收
                output.append("$" + MAKE_VARIABLE_PLACEHOLDER_PREFIX + "FUNCTION")
            continue
        output.append("$" + MAKE_VARIABLE_PLACEHOLDER_PREFIX + ("AUTOMATIC" if following in "@<^?*+|%" else re.sub(r"[^A-Za-z0-9_]", "_", following)))
        index += 2
    return "".join(output).replace("$$", "$")


def scan_makefile(text, texts, unjudged, command_line_variables=()):
    """Makefile 正文：配方行（制表符起头的行，与 `目标: 依赖 ; 配方`）与 $(shell …) 照 shell 命令交出，变量照文件与命令行的定义展开（make_expanded）；
    include 别的文件、SHELL 换成不是 sh / bash 的、.RECIPEPREFIX 换了配方前缀的交说明。"""
    joined = re.sub(r"\\\n[ \t]*", " ", text)
    variables = dict(MAKE_DEFAULT_VARIABLES)
    lines = joined.split("\n")
    define_name, define_lines = None, []
    for line in lines:
        if define_name is not None:
            if line.strip() == "endef":
                variables[define_name] = "\n".join(define_lines)
                define_name = None
            else:
                define_lines.append(line)
            continue
        define = re.match(r"^(?:(?:export|override)\s+)*define\s+([A-Za-z_][A-Za-z0-9_.\-]*)", line)
        if define:
            define_name, define_lines = define.group(1), []
            continue
        if line.startswith("\t"):
            continue
        assignment = MAKE_ASSIGNMENT.match(line)
        if assignment:
            name, operator, value = assignment.group(1), assignment.group(2), assignment.group(3).strip()
            if operator == "+=" and name in variables:
                variables[name] = variables[name] + " " + value
            elif operator == "?=" and name in variables and name not in MAKE_DEFAULT_VARIABLES:
                pass
            elif operator == "!=":
                variables[name] = "$(" + MAKE_VARIABLE_PLACEHOLDER_PREFIX + "SHELL_ASSIGNMENT)"
            else:
                variables[name] = value
    variables.update(dict(command_line_variables))
    shell_value = variables.get("SHELL", "/bin/sh").strip()
    if os.path.basename(shell_value.split()[0] if shell_value.split() else "sh") not in SHELL_NAMES | RENAMED_SHELL_NAMES:
        unjudged.append(f"Makefile 把 SHELL 换成了 {shell_value}，认不出配方交给谁跑")
    shell_texts = []
    in_define = False
    for line in lines:
        stripped = line.strip()
        if re.match(r"^(?:(?:export|override)\s+)*define\s", line):
            in_define = True
            continue
        if in_define:
            in_define = stripped != "endef"
            continue
        if re.match(r"^-?(?:include|sinclude)\s", stripped):
            unjudged.append(f"Makefile 里 `{stripped[:80]}` 读进别的文件，那里的配方没读")
            continue
        if stripped.startswith(".RECIPEPREFIX"):
            unjudged.append("Makefile 换了配方前缀（.RECIPEPREFIX），认不出哪几行是配方")
            continue
        recipe = None
        shell_assignment = MAKE_ASSIGNMENT.match(line) if not line.startswith("\t") else None
        if shell_assignment and shell_assignment.group(2) == "!=":
            texts.append(make_expanded(shell_assignment.group(3), variables, 0, shell_texts, unjudged))   # `名字 != 命令`：读 Makefile 时就跑这条命令
            continue
        if line.startswith("\t"):
            recipe = line[1:]
        elif ";" in line and ":" in line.split(";", 1)[0] and not MAKE_ASSIGNMENT.match(line):
            recipe = line.split(";", 1)[1]
        else:
            make_expanded(line, variables, 0, shell_texts, unjudged)   # 变量定义与规则行里的 $(shell …) 读 Makefile 时就跑
            continue
        recipe = MAKE_RECIPE_PREFIX.sub("", recipe)
        if recipe.strip():
            texts.append(make_expanded(recipe, variables, 0, shell_texts, unjudged))
    texts.extend(shell_texts)


def program_text_launches(language, text, variables=()):
    """程序从文件读的代码（awk -f、sed -f、Makefile）里起的命令：交 (字面命令的列表, 认不出的说明列表)。variables 是 make 命令行上的定义。"""
    texts, unjudged = [], []
    if language == "awk":
        scan_awk(text, texts, unjudged, variables)
    elif language == "sed":
        sed_script_commands(text, texts, unjudged)
    elif language == "make":
        scan_makefile(text, texts, unjudged, variables)
    return texts, unjudged


def git_config_commands(arguments, directory, environment):
    """git 这一条要读的配置里带命令的几项与别名：同一个进程里按 (目录, 配置相关的环境变量) 缓存，交 [(键, 值), …]。经 `git config --get-regexp` 照 git 自己的读法取
    （全局、系统、仓里的配置与 include 一并读）；git 起不来、超时、不在仓里交读到的那些（可以为空）。这一份库里只有这一处起子进程。"""
    import subprocess
    directory_arguments = []
    for option, value in arguments:
        if option in ("-C", "--git-dir", "--work-tree"):
            directory_arguments += [option, value]
    keyed_environment = tuple(sorted((name, value) for name, value in environment.items() if name.startswith("GIT_CONFIG") or name in ("HOME", "XDG_CONFIG_HOME", "GIT_DIR")))
    key = (directory, tuple(directory_arguments), keyed_environment)
    if key not in GIT_CONFIG_QUERY_CACHE:
        found = []
        try:
            result = subprocess.run(["git", *directory_arguments, "config", "--null", "--get-regexp", GIT_CONFIG_QUERY_PATTERN], cwd=directory if directory and os.path.isdir(directory) else None,
                                    env=dict(os.environ, **dict(keyed_environment)), capture_output=True, text=True, timeout=GIT_CONFIG_QUERY_SECONDS)
            for entry in result.stdout.split("\0"):
                if "\n" in entry:
                    name, value = entry.split("\n", 1)
                    found.append((name.lower(), value))
        except (OSError, subprocess.SubprocessError, UnicodeDecodeError):
            pass
        GIT_CONFIG_QUERY_CACHE[key] = found
    return GIT_CONFIG_QUERY_CACHE[key]


def scan_git(arguments, directory, environment, texts, unjudged):
    """git：-c 给的与配置里读到的别名（`!` 起头的照命令交出，别的接着照别名链找，最多 EMBEDDED_NESTING 层）、带命令的配置项（core.pager、core.fsmonitor、diff.external……）
    与这一条设的带命令的环境变量（GIT_EXTERNAL_DIFF、GIT_PAGER、GIT_SSH_COMMAND……）照命令交出；GIT_EXEC_PATH、GIT_CONFIG_PARAMETERS 与 --config-env 交说明。"""
    options, position, given = [], 0, {}
    while position < len(arguments):
        argument = arguments[position]
        if not argument.startswith("-") or argument == "--":
            break
        option, has_value, value = argument.partition("=") if argument.startswith("--") else (argument, False, "")
        if option in GIT_OPTIONS_WITH_VALUE and not has_value:
            value = arguments[position + 1] if position + 1 < len(arguments) else ""
            position += 1
        position += 1
        options.append((option, value))
        if option == "-c":
            name, _, config_value = value.partition("=")
            given[name.lower()] = config_value
        elif option == "--config-env":
            unjudged.append(f"git --config-env {value} 从环境变量取配置值，认不出它起不起命令")
        elif option == "--exec-path" and has_value:
            unjudged.append(f"git --exec-path={value} 换掉了 git 的子命令程序，认不出起的是什么")
    for name in GIT_OPAQUE_ENVIRONMENT:
        if name in environment:
            unjudged.append(f"git 这一条设了 {name}，认不出 git 起的是什么")
    count = environment.get("GIT_CONFIG_COUNT")
    if count is not None:
        if not count.isdigit():
            unjudged.append("git 这一条的 GIT_CONFIG_COUNT 不是字面的数，认不出它给的配置")
        else:
            for number in range(int(count)):
                name, value = environment.get(f"GIT_CONFIG_KEY_{number}"), environment.get(f"GIT_CONFIG_VALUE_{number}")
                if name is None or value is None:
                    unjudged.append(f"git 这一条的 GIT_CONFIG_KEY_{number} / GIT_CONFIG_VALUE_{number} 没给全，认不出它给的配置")
                else:
                    given[name.lower()] = value
    for name in GIT_COMMAND_ENVIRONMENT:
        value = environment.get(name)
        if value:
            texts.append(value)
    configured = dict(git_config_commands(options, directory, environment))
    configured.update(given)
    for name, value in configured.items():
        if name.startswith("alias."):
            continue
        if name in ("credential.helper",) or (name.startswith("credential.") and name.endswith(".helper")):
            if value.startswith("!"):
                texts.append(value[1:])
            continue
        if GIT_COMMAND_CONFIG_KEY.match(name) and not ((name == "core.pager" or name.startswith("pager.") or name == "core.fsmonitor") and value.lower() in GIT_BOOLEAN_VALUES):
            texts.append(value)
    subcommand_words = arguments[position:]
    for _ in range(EMBEDDED_NESTING):
        if not subcommand_words:
            break
        alias = configured.get("alias." + subcommand_words[0].lower())
        if alias is None:
            break
        if alias.startswith("!"):
            texts.append(alias[1:] + "".join(" " + shlex.quote(word) for word in subcommand_words[1:]))
            break
        try:
            subcommand_words = shlex.split(alias) + subcommand_words[1:]
        except ValueError:
            unjudged.append(f"git 别名 {subcommand_words[0]} 的值切不开，认不出起的是什么")
            break


def scan_options_for_commands(name, arguments, environment, texts, unjudged):
    """tar、ssh、less、vim 这几样带命令的选项与环境变量：值是字面的照命令交出。"""
    if name in ("tar", "bsdtar", "gtar"):
        position = 0
        while position < len(arguments):
            argument = arguments[position]
            option, has_value, value = argument.partition("=")
            matched = next((known for known in TAR_COMMAND_OPTIONS if known.startswith(option) and option.startswith("--") and len(option) > 3), None)
            if matched:
                if not has_value:
                    value = arguments[position + 1] if position + 1 < len(arguments) else ""
                    position += 1
                texts.append(value)
            elif argument.startswith("--checkpoint-action=exec="):
                texts.append(argument[len("--checkpoint-action=exec="):])
            elif argument in ("-I", "-F"):
                texts.append(arguments[position + 1] if position + 1 < len(arguments) else "")
                position += 1
            elif argument.startswith("-") and not argument.startswith("--") and ("I" in argument[1:] or "F" in argument[1:]):
                unjudged.append(f"tar 的短选项 {argument} 里带 -I / -F（起命令的选项），认不出起的是什么")
            position += 1
    elif name == "ssh":
        position = 0
        while position < len(arguments):
            argument = arguments[position]
            value = None
            if argument == "-o" and position + 1 < len(arguments):
                value = arguments[position + 1]
                position += 1
            elif argument.startswith("-o") and len(argument) > 2:
                value = argument[2:]
            if value is not None:
                key, _, setting = value.replace("=", " ", 1).partition(" ")
                if key.lower() in SSH_COMMAND_OPTIONS and setting.strip().lower() != "none":
                    texts.append(setting.strip())
            elif not argument.startswith("-"):
                break
            position += 1
    elif name == "less":
        for argument in arguments:
            if argument.startswith("+") and "!" in argument:
                texts.append(argument.split("!", 1)[1])
        for variable in LESS_OPEN_ENVIRONMENT:
            value = environment.get(variable)
            if value:
                texts.append(value.lstrip("|-").replace("%s", "file"))
    elif name in VIM_NAMES:
        position = 0
        while position < len(arguments):
            argument = arguments[position]
            ex_command = None
            if argument in ("-c", "--cmd") and position + 1 < len(arguments):
                ex_command = arguments[position + 1]
                position += 1
            elif argument.startswith("+") and len(argument) > 1:
                ex_command = argument[1:]
            if ex_command is not None:
                stripped = ex_command.lstrip(": ")
                if stripped.startswith("!"):
                    texts.append(stripped[1:])
                elif re.search(r"!|system\s*\(|\bterm|\bsh(?:ell)?\b|\bpy|\blua|\bperl|\bruby", stripped):
                    unjudged.append(f"vim 的 ex 命令 `{stripped[:80]}` 能起命令，认不出起的是什么")
            position += 1


def interpreter_code_options(arguments, code_options, value_options, flag_takes_code=""):
    """解释器的命令行：code_options 里的选项（`-e` 这类，值贴着写或另起一个词，短选项可以合写在一簇的末尾）给的代码串都收；value_options 带一个值、不是代码。
    交 (代码串列表, 第一个不是选项的词的下标或 None)。"""
    codes, position = [], 0
    while position < len(arguments):
        argument = arguments[position]
        if argument == "--":
            return codes, position + 1 if position + 1 < len(arguments) else None
        if not argument.startswith("-") or argument == "-":
            return codes, position
        matched = False
        for option in code_options:
            if argument == option:
                codes.append(arguments[position + 1] if position + 1 < len(arguments) else "")
                position += 2
                matched = True
                break
            if option.startswith("--") and argument.startswith(option + "="):
                codes.append(argument[len(option) + 1:])
                position += 1
                matched = True
                break
        if matched:
            continue
        if argument in value_options:
            position += 2
            continue
        if not argument.startswith("--") and flag_takes_code:
            letters = argument[1:]
            for offset, letter in enumerate(letters):
                if letter in flag_takes_code:
                    rest = letters[offset + 1:]
                    if rest:
                        codes.append(rest)
                        position += 1
                    else:
                        codes.append(arguments[position + 1] if position + 1 < len(arguments) else "")
                        position += 2
                    break
            else:
                position += 1
            continue
        position += 1
    return codes, None


def embedded_launches(command):
    """命令位置上的一条命令（CommandAtPosition）是别的程序时，它里面起的命令（文件头「别的程序里起 shell」一段；EmbeddedLaunches）。
    弄坏开关 LIB_SHELL_WORDS_BREAK=embedded-launches-unseen 下一条都不认。"""
    if embedded_launches_unseen() or not command.words:
        return EmbeddedLaunches((), (), ())
    name, arguments = os.path.basename(command.words[0]), list(command.words[1:])
    environment = command.environment or {}
    texts, unjudged, files = [], [], []
    awk_assignments = []   # awk 命令行上的 `名字=值`（-v 与操作数）：(名字, 值或 None)，值写成 shell 变量、命令替换的是 None
    def scan_code(language, code_texts, reads_standard_input):
        if reads_standard_input:
            code, problem, from_file = interpreter_standard_input(command)
            if code is not None:
                code_texts = list(code_texts) + [code]
            if problem:
                unjudged.append(f"{name} 从标准输入读代码：{problem}")
        for code in code_texts:
            if language == "python":
                scan_python(code, texts, unjudged)
            elif language == "tcl":
                scan_tcl(code, texts, unjudged)
            elif language == "awk":
                scan_awk(code, texts, unjudged, awk_assignments)
            elif language == "perl":
                scan_quoted_language("perl", code, PERL_LAUNCH_WORD, ("$", "@"), "'\"`", "#", texts, unjudged)
            elif language == "ruby":
                scan_quoted_language("ruby", code, RUBY_LAUNCH_WORD, ("#{",), "'\"`", "#", texts, unjudged)
            elif language == "php":
                scan_quoted_language("php", code, PHP_LAUNCH_WORD, ("$",), "'\"`", "#", texts, unjudged)
            elif language == "lua":
                scan_quoted_language("lua", code, LUA_LAUNCH_WORD, (), "'\"", None, texts, unjudged)
            elif language == "node":
                if "child_process" in code:
                    before = len(texts) + len(unjudged)
                    scan_quoted_language("node", code, NODE_LAUNCH_WORD, ("${",), "'\"`", None, texts, unjudged)
                    if len(texts) + len(unjudged) == before:
                        unjudged.append("node 代码用了 child_process，认不出起的是什么")
    if PYTHON_NAME.match(name):
        invocation = python_invocation(arguments)
        if invocation.code_string is not None:
            scan_code("python", [invocation.code_string], False)
        elif invocation.module is None and invocation.script_index is None:
            scan_code("python", [], True)
    elif PERL_NAME.match(name):
        codes, script_index = interpreter_code_options(arguments, ("-e", "-E"), ("-I", "-M", "-m", "-x", "-0", "-l", "-i", "-C", "-d", "-D"), flag_takes_code="eE")
        scan_code("perl", codes, not codes and script_index is None)
    elif RUBY_NAME.match(name):
        codes, script_index = interpreter_code_options(arguments, ("-e",), ("-I", "-r", "-C", "-E", "-F", "-K", "-T", "-W", "-x", "--encoding"), flag_takes_code="e")
        scan_code("ruby", codes, not codes and script_index is None)
    elif name in NODE_NAMES:
        codes, script_index = interpreter_code_options(arguments, ("-e", "--eval", "-p", "--print"), ("-r", "--require", "--import", "--loader", "-C", "--conditions"))
        scan_code("node", codes, not codes and script_index is None)
    elif PHP_NAME.match(name):
        codes, script_index = interpreter_code_options(arguments, ("-r", "-B", "-R", "-E"), ("-c", "-d", "-z", "-t", "-F"))
        scan_code("php", codes, False)
    elif LUA_NAME.match(name):
        codes, script_index = interpreter_code_options(arguments, ("-e",), ("-l",))
        scan_code("lua", codes, not codes and script_index is None)
    elif TCL_NAME.match(name):
        codes, script_index = interpreter_code_options(arguments, ("-c",), ("-f", "-encoding", "-D"))
        scan_code("tcl", codes, not codes and script_index is None)
    elif name in AWK_NAMES or (name == "busybox" and arguments[:1] == ["awk"]):
        if name == "busybox":
            arguments = arguments[1:]
        programs, program_files, position = [], [], 0
        while position < len(arguments):
            argument = arguments[position]
            if argument == "--":
                position += 1
                break
            if argument in ("-f", "--file") or argument.startswith("--file="):
                program_files.append(argument.split("=", 1)[1] if "=" in argument else (arguments[position + 1] if position + 1 < len(arguments) else ""))
                position += 1 if "=" in argument else 2
                continue
            if argument in ("-e", "--source") or argument.startswith("--source="):
                programs.append(argument.split("=", 1)[1] if "=" in argument else (arguments[position + 1] if position + 1 < len(arguments) else ""))
                position += 1 if "=" in argument else 2
                continue
            if argument in ("-v", "--assign") or argument.startswith("--assign=") or (argument.startswith("-v") and len(argument) > 2):
                given = argument.split("=", 1)[1] if argument.startswith("--assign=") else argument[2:] if argument.startswith("-v") and len(argument) > 2 else (
                    arguments[position + 1] if position + 1 < len(arguments) else "")
                assignment = ASSIGNMENT.match(given)
                if assignment:
                    awk_assignments.append((assignment.group(1), None if "$" in assignment.group(2) or "`" in assignment.group(2) else assignment.group(2)))
                position += 2 if argument in ("-v", "--assign") else 1
                continue
            if argument in ("-F", "--field-separator", "-i", "--include", "-l", "--load", "-W"):
                position += 2
                continue
            if argument.startswith("-f") and len(argument) > 2:
                program_files.append(argument[2:])
                position += 1
                continue
            if argument.startswith("-") and argument != "-":
                position += 1
                continue
            break
        if not programs and not program_files and position < len(arguments):
            programs.append(arguments[position])
            position += 1
        for operand in arguments[position:]:
            assignment = ASSIGNMENT.match(operand)
            if assignment:
                awk_assignments.append((assignment.group(1), None if "$" in assignment.group(2) or "`" in assignment.group(2) else assignment.group(2)))
        scan_code("awk", programs, False)
        files += [ProgramFile("awk", (program_file,), command.directory, tuple(awk_assignments)) for program_file in program_files]
    elif name in ("sed", "gsed") or (name == "busybox" and arguments[:1] == ["sed"]):
        if name == "busybox":
            arguments = arguments[1:]
        scripts, script_files, position, sandbox = [], [], 0, False
        while position < len(arguments):
            argument = arguments[position]
            if argument == "--":
                position += 1
                break
            if argument == "--sandbox":
                sandbox = True
            elif argument.startswith("--expression") or argument.startswith("--file"):
                value = argument.split("=", 1)[1] if "=" in argument else (arguments[position + 1] if position + 1 < len(arguments) else "")
                (scripts if argument.startswith("--expression") else script_files).append(value)
                position += 1 if "=" in argument else 2
                continue
            elif argument.startswith("--"):
                if argument in ("--line-length",):
                    position += 1
            elif argument.startswith("-") and argument != "-":
                letters = argument[1:]
                consumed = False
                for offset, letter in enumerate(letters):
                    if letter in "efl":
                        value = letters[offset + 1:] or (arguments[position + 1] if position + 1 < len(arguments) else "")
                        if letter == "e":
                            scripts.append(value)
                        elif letter == "f":
                            script_files.append(value)
                        position += 1 if letters[offset + 1:] else 2
                        consumed = True
                        break
                    if letter == "i":
                        break
                if consumed:
                    continue
            else:
                break
            position += 1
        if not scripts and not script_files and position < len(arguments):
            scripts.append(arguments[position])
        if not sandbox:
            for script in scripts:
                sed_script_commands(script, texts, unjudged, shell_word=True)
            files += [ProgramFile("sed", (script_file,), command.directory) for script_file in script_files]
    elif name in MAKE_NAMES:
        makefiles, evaluations, directory, variables, position, runs_nothing = [], [], command.directory, [], 0, False
        while position < len(arguments):
            argument = arguments[position]
            if argument in ("-f", "--file", "--makefile", "-C", "--directory", "-E", "--eval", "-I", "--include-dir", "-o", "--old-file", "--assume-old", "-W", "--what-if", "--new-file", "--assume-new"):
                value = arguments[position + 1] if position + 1 < len(arguments) else ""
                position += 2
            elif argument.startswith("--") and "=" in argument:
                argument, value = argument.split("=", 1)
                position += 1
            elif argument[:2] in ("-f", "-C", "-E", "-I", "-o", "-W") and len(argument) > 2:
                argument, value = argument[:2], argument[2:]
                position += 1
            else:
                value = None
                position += 1
            if argument in ("-f", "--file", "--makefile"):
                makefiles.append(value)
            elif argument in ("-C", "--directory"):
                directory = resolve_path(directory, os.path.expanduser(value)) if directory is not None or os.path.isabs(value) else None
            elif argument in ("-E", "--eval"):
                evaluations.append(value)
            elif argument in ("-v", "--version", "-h", "--help"):
                runs_nothing = True
            elif value is None and ASSIGNMENT.match(argument):
                assignment = ASSIGNMENT.match(argument)
                variables.append((assignment.group(1), assignment.group(2)))
        if not runs_nothing:
            for evaluation in evaluations:
                scan_makefile(evaluation, texts, unjudged, variables)
            standard_input_makefiles = [makefile for makefile in makefiles if makefile in ("-", "/dev/stdin")]
            if standard_input_makefiles:
                code, problem, from_file = interpreter_standard_input(command)
                if code is not None:
                    scan_makefile(code, texts, unjudged, variables)
                if problem:
                    unjudged.append(f"make 从标准输入读 Makefile：{problem}")
            other_makefiles = [makefile for makefile in makefiles if makefile not in ("-", "/dev/stdin")]
            if other_makefiles:
                files += [ProgramFile("make", (makefile,), directory, tuple(variables)) for makefile in other_makefiles]
            elif not makefiles:
                files.append(ProgramFile("make", MAKE_DEFAULT_FILES, directory, tuple(variables)))
    elif name == "git":
        scan_git(arguments, command.directory, environment, texts, unjudged)
    else:
        scan_options_for_commands(name, arguments, environment, texts, unjudged)
    return EmbeddedLaunches(tuple(text for text in texts if text.strip()), tuple(dict.fromkeys(unjudged)), tuple(files))


def heredocs_fed(redirections, stripped_heredocs):
    """一条简单命令上 `<<` / `<<-` 喂给它的、被当数据剥掉的 heredoc 正文（HeredocBody，按出现次序）：重定向目标是 HEREDOC_BODY_MARKER 记号的，按编号从 stripped_heredocs 取。"""
    found = [HEREDOC_BODY_MARKER.match(target) for operator, target in redirections if operator in ("<<", "<<-")]
    return tuple(stripped_heredocs[int(marker.group(1))] for marker in found if marker and int(marker.group(1)) < len(stripped_heredocs))


if __name__ == "__main__":
    # 这份库没有命令行入口（文件头「弄坏开关」：由导入它的几份的自证判红）；直接执行时设着 LIB_SHELL_WORDS_BREAK 一律拒，不管值是什么：弄坏开关只在导入它的几份的自证里认。
    import sys
    if os.environ.get("LIB_SHELL_WORDS_BREAK"):
        print(f"  ✗ LIB_SHELL_WORDS_BREAK={os.environ['LIB_SHELL_WORDS_BREAK']} 是弄坏开关，只在导入这份库的几份的自证（bash-command-detector.sh、project-code-guard.sh 等的 --selftest）里认，直接执行设了就拒，这一趟什么都没做", file=sys.stderr)
        print("  → 怎么办：去掉 LIB_SHELL_WORDS_BREAK 再跑；要证红就跑 LIB_SHELL_WORDS_BREAK=<项> bash .claude/hooks/project-code-guard.sh --selftest（项与红的格写在文件头「弄坏开关」一段）", file=sys.stderr)
        sys.exit(2)
