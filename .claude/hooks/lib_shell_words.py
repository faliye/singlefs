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
弄坏开关 LIB_SHELL_WORDS_BREAK=arguments-as-executed 把这两种里的参数照旧当被执行的脚本，红格登在哪见「弄坏开关」一段），
python 的选项照解释器的语义切（python_invocation）：短选项可以合写（`-Bu`），`-W`、`-X` 与 `--check-hash-based-pycs` 带一个值（贴着写或另起一个词），
值不当脚本名；`-c <代码>` 的程序是那段代码、没有被执行的脚本，交出的是 python 这一条本身（代码由调用方从参数里取）；`-m <模块>` 交出
`<模块里的点换成斜杠>.py` 当被执行的脚本（launcher 是 "python"），`-m runpy <模块>` 取后面那个模块，调用方拿它与仓内脚本的位置比
（弄坏开关 LIB_SHELL_WORDS_BREAK=python-options-by-dash 退回「以 - 开头的词都当选项跳过、第一个不以 - 开头的词当脚本」，红格登在哪见「弄坏开关」一段），
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
字面写着的命令交 EmbeddedLaunches.texts，调用方同 bash -c 那样再判；写法认不出的（变量、拼接、__import__ / getattr / exec 动态取、ctypes、--config-env、GIT_EXEC_PATH，
make -C 的目录与 git -c 的配置名、别名值写成这段命令文本里代入不了的变量或命令替换）交 unjudged，调用方照认不出办（make -C、git -c 前面赋过字面值的照代入）；
程序从文件读的代码（awk -f、sed -f、make 的 -f 与默认的 GNUmakefile / makefile / Makefile）交 program_files，调用方读正文交 program_text_launches。
不读的：python / perl / ruby / tclsh 起的脚本文件（同共用约束「写」一节，脚本文件里与 python subprocess 起的写法由子 agent 自己守）、git 的钩子目录（core.hooksPath、.git/hooks）、ssh 的配置文件、
Makefile 里 include 进来的文件（交 unjudged）、vim 的 -S / -u 读的文件。弄坏开关 LIB_SHELL_WORDS_BREAK=embedded-launches-unseen 下一条都不认。
从标准输入读命令的 shell（bash / sh 这类不带 -c、不带脚本，或带 -s：shell_reads_standard_input）收到的字面文本当这个 shell 的代码再切一遍，
同 bash -c 一样挂回（它那一条命令的包装、目录、变量、memory_capped 照算；shell_standard_input）：here-string `<<<` 的文字；管道前一段是 printf（格式串与参数照 printf 现算）、
echo（-n / -e / -E 照 bash 内建认）、不带文件操作数而读 heredoc 的 cat 时它们交出的文字。喂进去的不是字面的判不了，每处一句说明交在 ShellScan.unjudged_shell_inputs：
here-string 或 printf / echo 的参数里有 `$`、反引号（变量、命令替换；单引号里写的同样算，切词时分不出，字面的那一部分照切）、不带引号结束词的 heredoc 正文里有它们、
管道前一段是别的命令（cat 文件、花括号组、子 shell……）、标准输入从文件或别的描述符来（`< 文件`、`<&`、`<>`；`< /dev/null` 不算）。调用方按各自的「判不了」处置（放行并记检出）。
弄坏开关 LIB_SHELL_WORDS_BREAK=shell-standard-input-unread 下不再切、也不交说明，红格登在哪见「弄坏开关」一段。
脚本文件本身不在这里读，要读的 hook 自己读，再把正文交回 commands_at_command_position。
判不到的：变量里拼出来的命令、eval 的参数里外层先展开的变量与命令替换（展开成什么判不到，照原文切）、喂给 shell 的标准输入不是字面的（交判不了的说明，见上）、管道前一段是花括号组或子 shell 时组里 printf / echo 的文字（整组当判不了）、
进程替换的输出再交给 shell 执行（`bash <(…)`、`source <(…)`，括号里的命令照判，它打印出来的不判；project-code-guard.sh 照认不出拒）、Makefile 起的命令
（xargs 与名单外启动器由 lib_project_code_runs.py 的 command_under_extra_launcher 剥）；
喂给 shell 的 heredoc 正文里引号没配对时，其后的切词会错；heredoc 挂的命令从上一行续行过来时认不出它喂给谁（当数据剥掉），没收尾的替换开在上一行的同样认不出；圆括号子 shell 里的 cd 与 export 当成对后面的命令也生效（变量记值另分作用域，见下）；
花括号展开不分 `[[ ]]`、case 模式这类 bash 不展开的位置（照展，判得偏严）；只定义的函数认 `function 名字 { …; }` 与 `名字() { …; }` 两种写法，函数体里套花括号的、函数体之外有变量或命令替换的照判（偏严）；`builtin` 认的是 bash 5.2 的内建表（enable -f 载进来的不认，照不跑放行）；
命令替换与进程替换里的命令按替换所在那条命令开始时的目录与 export 判（替换里的 cd 不影响外面）。
变量展开判不到的：值从上一条 Bash、source 进来的文件、eval 来的（目录照旧认不出）；单引号里的 `$名字` 照样展开（引号在切词时已去掉，分不出）；
bash -c 的那段代码按外层记下的全部变量展开（没 export 的变量里层真跑时看不到，这里照样展开，判得偏严）；
变量记值分作用域（VariableScopes，认不准的记成算不出、偏严）：圆括号子 shell、管道段、放后台的里面改过的值出来之后算不出；`&&` / `||` 之后、if / case / 循环 / 函数体里
把记过的值改成别的，离开那一串、那一组之后算不出（一串 `&&` 里接着用的照新值）；循环体里后一遍用到前一遍赋的值不认；export 进环境的与 cd 的目录照旧不分作用域。
别名（认不准开没开展开的照开着算，偏严）：shopt -s expand_aliases、set -o posix 之后，起来时就开着的 shell（bash -O expand_aliases、-i、posix 模式、读启动文件的，sh / dash / ksh / ash / zsh）
里定义的别名照展开（shell_starts_with_aliases）；eval、trap、命令替换、source /dev/stdin 的代码带着外面先定义的别名，eval 里定义的别名对后面的行同样生效；
别名的值的头一个词是别的别名的接着展开；同一行里定义的不展开只照 bash 认。
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
弄坏开关（LIB_SHELL_WORDS_BREAK，几个用逗号连；只给自证判红用）。这一份没有自己的 --selftest，由导入它的几份的自证判红。
每条开关在这里只留「名字、弄坏哪一处」；打开后哪几份自证的哪几格红只登记一处：project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
（门禁 code-tooling 的 break-switch-replay 格逐条设上开关、跑那一份的 --selftest，点名的格与实际红的格逐格比），这里不抄第二份。
每条开关下面那一行只是指路（break-switch-headers 格要求每条开关有点名的项）；写着「尚未登记」的，两份钩子的自证里读不出会红的格。
  LIB_SHELL_WORDS_BREAK=arguments-as-executed  `bash -n 脚本` 的目标与 `python3 - 词…` 后面的词照旧当被执行的脚本交出
      红格见 project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=python-options-by-dash  python_invocation() 退回「以 - 开头的词都当选项跳过、第一个不以 - 开头的词当脚本」：带值选项的值当成脚本名，-m、-c 不认
      红格见 project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=array-appends-unseen  record_array_writes() 什么都不记：`名字+=值`、`名字[下标]=值` 照旧不改记下的值
      尚未登记：project-code-guard.sh、bash-command-detector.sh 的自证里读不出会红的格
  LIB_SHELL_WORDS_BREAK=env-chdir-ignored  skip_prefixes() 不交 env 的 -C / --chdir 换的目录：commands_at_command_position 里这一条命令照外面的当前目录判
      尚未登记：project-code-guard.sh、bash-command-detector.sh 的自证里读不出会红的格
  LIB_SHELL_WORDS_BREAK=env-option-clusters-unparsed  skip_prefixes() 里 env 的选项退回只认整词 -u / --unset / -C / --chdir：合写的 -iC、-0iu、缩写的 --chd、单独的 - 与 -S / --split-string 不认（选项簇当不带值的开关，它的值词当命令词；-S 的字符串整个当命令词）
      尚未登记：project-code-guard.sh、bash-command-detector.sh 的自证里读不出会红的格
  LIB_SHELL_WORDS_BREAK=env-split-string-unexpanded  skip_prefixes_expanding() 不拆 env 的 -S / --split-string（-S 与它的字符串整个跳过，里面的命令不判）
      尚未登记：project-code-guard.sh、bash-command-detector.sh 的自证里读不出会红的格
  LIB_SHELL_WORDS_BREAK=directory-variables-unexpanded  expand_known_variables() 遇到 `$` 一律交 None：cd / pushd / env -C 的目录写在变量里的照旧认不出
      红格见 project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=variable-rebinding-unseen  for / select / read / unset 改写过的变量照旧用之前记下的值（rebind_variables() 与 unset 那一处不记成算不出）
      尚未登记：project-code-guard.sh、bash-command-detector.sh 的自证里读不出会红的格
  LIB_SHELL_WORDS_BREAK=shell-standard-input-unread  commands_at_command_position() 不把 here-string 与管道喂给 shell 的文本当命令再切、也不交判不了的说明
      红格见 project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=redirect-fd-dropped  shell_tokens() 不把 fd 号挂到重定向运算符上：redirects_standard_output() 把 `2> 文件` 也当改标准输出
      红格见 project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=link-source-from-current-directory  link_source_word() 原样交回：ln -s 的相对源照当前目录解
      红格见 project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=env-leading-equals-as-command  skip_prefixes() 里 env 后面开头就是 `=` 的词当命令词
      红格见 project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=shell-c-options-unparsed  shell_invocation() 取紧跟 -c 的那个词当代码（`bash -c -- '…'` 的代码取成 --）
      红格见 project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=builtin-non-builtin-run  skip_prefixes() 把 builtin 后面不是内建的词照旧当命令词
      红格见 project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=uncalled-functions-run  drop_uncalled_function_bodies() 原样交回：只定义、调不到的函数体照命令判
      红格见 project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=shell-s-positionals-as-script  shell_invocation() 把 -s 后面的词照旧当脚本
      红格见 project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=time-options-as-command  skip_prefixes() 不跳 time 关键字后面的 -p / --（-p 当命令词）
      红格见 project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=builtin-as-command  skip_prefixes() 把 builtin 当命令词（后面的 source / cd 不认）
      红格见 project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=shell-option-values-as-script  shell_invocation() 不认 --rcfile / --init-file 的值，`--` 与单独的 `-` 当脚本
      红格见 project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=function-keyword-as-command  skip_prefixes() 把 function 关键字当命令词
      红格见 project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=braces-unexpanded  brace_expand() 原样交回（引号外的花括号不展开）
      红格见 project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=double-slash-kept  resolve_path() 留着开头的 //（os.path.normpath 的样子）
      红格见 project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=env-loose-assignments-as-command  skip_prefixes() 里 env 后面名字不合变量名的 `名字=值` 当命令词
      红格见 project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=eval-unread  commands_at_command_position() 不切 eval 的参数（eval 当一条普通命令）
      红格见 project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=heredoc-open-substitution-unseen  heredoc_feeds_shell() 照整行找挂它的命令（没收尾的 $( / <( 里的 bash 认不出，正文当数据剥掉）
      红格见 project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=brace-empties-kept  brace_expand() 留着花括号展开出来的空串（`{cargo,} test` 切成 `cargo '' test`）
      红格见 project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=brace-sequence-unpadded  brace_sequence_width() 一律交 0：补零的整数序列（{01..03}）不补零（lib_project_code_runs.py 的花括号展开调同一个 brace_sequence，一并不补）
      红格见 project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=double-quoted-substitution-unclosed  open_substitution_tail() 把双引号里的 `$(` 一律当没收尾（顶层的 bash <<EOF 当挂在替换里，正文当数据剥掉）
      红格见 project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=function-calls-literal-only  function_never_called() 只数原文里的字面：拆开拼写的调用（`"fo"o`、`fo{o,}`、`fo\\o`）认不出，函数体照不判
      红格见 project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=redirect-fd-number-literal  redirection_fd() 照字面比 fd 号：`01>` 不算 1 号
      红格见 project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=redirect-fd-quoted-digits  shell_tokens() 把引号或反斜杠写出的数字（`"2">`、`\\2>`）也当 fd 号
      红格见 project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=redirect-fd-duplication-unseen  standard_output_destination() 不跟 `n>&m`，只取直接改标准输出的最后一个
      红格见 project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=shell-long-options-after-short  shell_invocation() 把短选项之后的长选项照旧当选项跳过
      红格见 project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=shell-syntax-only-code-run  shell_invocation() 带 -n 时 -c 的代码照判
      红格见 project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=shell-option-clusters-unvalued  shell_invocation() 里合写的短选项簇不吃 -o / -O 的值词（`bash -euo pipefail -c …` 的 pipefail 不吃掉，被当成 -c 的代码或脚本，整条 -c 的代码没进判）
      尚未登记：project-code-guard.sh、bash-command-detector.sh 的自证里读不出会红的格
  LIB_SHELL_WORDS_BREAK=env-options-after-assignment  skip_prefixes() 里 env 认到赋值之后照旧认选项
      红格见 project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=brace-sequence-width-padded-ends-only  brace_sequence_width() 只取补零写的那几边的长度
      红格见 project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=ansi-c-quotes-unseen  shell_tokens()、open_substitution_tail()、command_substitution_end() 把 `$'…'` 照普通单引号配对
      红格见 project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=case-patterns-as-commands  case_patterns_and_conditionals_folded() 不改 case 的模式
      红格见 project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=conditional-expression-split  case_patterns_and_conditionals_folded() 不改 `[[ … ]]`
      红格见 project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=redirect-target-braces-expanded  shell_tokens() 照旧展开重定向目标上的花括号（展开出来多的词成了这一条的参数）
      红格见 project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=extended-assignments-as-command  skip_prefixes() 把 `名字+=值`、`名字[下标]=值` 照旧当命令词
      红格见 project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=substitution-case-and-comment-unseen  command_substitution_end() 不认命令替换里的 case 模式与注释
      红格见 project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=coproc-as-command  skip_prefixes() 把 coproc 当命令词
      红格见 project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=quoted-braces-as-unknown  brace_expand() 交的词不带引号信息（引号里的花括号也当认不准）
      红格见 project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=env-ignore-environment-kept  skip_prefixes() 不认 env -i 清掉环境
      红格见 project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=shell-standard-input-file-unread  shell_standard_input_files() 一律交 []（`bash -s < 文件` 只记判不了）
      红格见 project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=function-launchers-unseen  launcher_functions() 交空集
      红格见 project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=startup-files-unread  startup_file_words() 一个文件都不交：BASH_ENV / ENV / --rcfile / .bashrc / zsh 的启动文件都不当被起的脚本
      红格见 project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=declared-exports-unseen  环境里的变量只认 `export 名字=值`（declare -x、typeset -x、local -x、先赋值后 export、先 export 后赋值、export -n、declare +x、set -a 不认）
      红格见 project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=interactive-startup-files-unread  shell_start_options() 的 -i 不另看：交互的 bash 照旧读 BASH_ENV、不读 .bashrc 与 --rcfile / --init-file
      红格见 project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=zsh-startup-files-unread  zsh 的 .zshenv、.zshrc 不读
      红格见 project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=executed-script-startup-unread  executed_script_startup() 什么都不交：直接执行的脚本起来时它的 shell 先读的启动文件不读，#! 指到认不出的解释器的不判
      红格见 project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=trap-action-unread  commands_at_command_position() 不把 trap 'action' 的字符串当 shell 代码再切（trap 的参数不判）
      红格见 project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=standard-input-device-script-unread  起 shell 的脚本路径是标准输入设备（/dev/stdin、/dev/fd/0、/proc/self/fd/0）时不当标准输入、照旧当一份脚本文件读
      红格见 project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=tilde-plus-minus-unexpanded  expand_directory_tilde() 不展开 `~+`（$PWD）与 `~-`（$OLDPWD）（当普通路径、落不到实处）
      红格见 project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=renamed-shells-unseen  is_shell_name() 只认 bash / sh / dash / zsh / ksh，rbash、ash 不当 shell，busybox 的 sh / ash applet 不剥
      红格见 project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=login-profiles-unread  startup_file_words() 不认登录 shell（-l、--login、exec -l、exec -a -名字）：/etc/profile、.bash_profile、.bash_login、.profile、.bash_logout 都不交
      红格见 project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=system-startup-files-unread  startup_file_words() 不交 SYSTEM_STARTUP_DIRECTORY 下的系统级启动文件（/etc/profile、/etc/bash.bashrc、/etc/zsh/ 下的）
      红格见 project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=embedded-launches-unseen  embedded_launches() 一条都不认：别的程序里起的命令不交
      红格见 project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=aliases-unexpanded  shell_aliases() 交空：shopt -s expand_aliases 之后定义的别名不展开成它的值（命令词是别名时照普通命令判）
      红格见 project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=sed-expansions-unparsed  sed_script_commands() 不另算 sed 脚本词里没展开的 bash 展开，原样切、切不下去照「解析不下去」拒
      红格见 project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=sed-text-first-line-only  sed_script_parse() 的 a / i / c 文本只读到头一个换行（`a\\` 换行之后的文本行被当命令读）
      红格见 project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=python-argument-variables-unresolved  scan_python() 起命令的参数只认整个字面的（argv 里有一项是变量、赋过字面值的变量、字面串的 .split() 都整条认不出）
      红格见 project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=awk-command-variables-unresolved  scan_awk() 不认赋过字面串的变量（`cmd = "date"; cmd | getline`、命令行 cmd=sort 再 print | cmd 照写成变量拒）
      红格见 project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=cd-known-targets-unexpanded  changed_directory() 不认 $PWD、$OLDPWD、`cd -`、起头的 `$(git rev-parse --show-toplevel)` 与 `$(pwd)`（cd 之后目录算不出）
      红格见 project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=cd-failure-guards-unseen  CommandAtPosition.cd_failure_skips 一律是假（`cd 不在的目录 && 相对脚本`、`|| exit` 守着的也照「cd 会失败」拒）
      红格见 project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=non-interactive-return-unseen  non_interactive_return_offset() 交 None：非交互 shell source 的文件里 `case $- in *i*) ;; *) return;; esac` 这类守卫不截、整份照读
      红格见 project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=aliases-same-line-expanded  commands_at_command_position() 不看别名定义在哪一行：同一行里先定义再用的照展开
      红格见 project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=heredoc-variables-unexpanded  interpreter_standard_input() 不代入 heredoc 正文里前面赋过字面值的变量：结束词不带引号、正文有 `$` 一律判不了
      红格见 project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=python-list-concatenation-unresolved  python_command_value() 不认 argv 的 `+ 列表` 与 `*列表`：整条认不出
      红格见 project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=python-compile-as-execution  scan_python() 把 compile 与 exec、eval 一样当执行
      红格见 project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=sed-unknown-flags-as-execution  sed_script_parse() 的 s 标志区退回扫到 `;`、换行、`}`、`w` 为止、区里有 e 就算 e 标志
      红格见 project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=cd-subshell-guards-unseen  commands_at_command_position() 不调 guarded_subshell_boundary()：括号进出一律断开 `&&` 一串
      红格见 project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=posix-uncalled-functions-run  drop_uncalled_function_bodies() 只认 function 关键字：只定义不调用的 `名字() { …; }` 照判函数体
      红格见 project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=alias-start-options-unseen  shell_starts_with_aliases() 一律交假、set -o posix 不算开：bash -O expand_aliases、sh / dash 这类起来时就开着别名展开的 -c 代码里的别名不展开
      红格见 project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=alias-nested-texts-unseen  eval、trap、命令替换、source /dev/stdin 的代码不带外面的别名，eval 里定义的别名不收
      红格见 project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=compound-stdin-closers-unseen  compound_redirected_stdin_files() 只认 exec、`}`、`done`、`)` 上的 `< 文件`：fi / esac 收尾、调函数时的 `< 文件`、`<&` / here-string / heredoc 下发、管道接进复合命令、bash -c 与 eval 代码里的都不认
      红格见 project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=embedded-option-variables-unexpanded  embedded_launches() 的 make -C 目录与 scan_git() 的 -c 配置写成变量的不代入、不交说明
      红格见 project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=variable-scopes-unseen  commands_at_command_position() 不建 VariableScopes：子 shell、管道段、放后台、`||` 之后、if 里赋的值照旧当对后面都生效
      红格见 project-code-guard.sh、bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=perl-bare-options-take-next-word  embedded_launches() 把 perl 里值只能贴着写的选项（-i、-l、-0、-x、-C、-d、-D）当成带值选项：单独写的 `perl -i -e '…'` 的 -e 连同代码被当成值跳过，里面起的命令不判
      红格见 project-code-guard.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=part-modules-cached  载入这一份时沿用 sys.modules 里裸名字缓存的同名部分模块（lib_shell_words_core、lib_shell_words_embedded）、sys.path 里没有这个目录才加、不放回：同一个进程再从别的目录载入一份时，部分模块还是先载入的那个目录里的
      红格见 bash-command-detector.sh 文件头同名开关下
  LIB_SHELL_WORDS_BREAK=part-modules-per-load  部分模块不按目录存：同一个目录里每载入一次这一份库就各载入一份自己的部分模块（类与哨兵对象各不相同，本进程里按文件路径载入它的别的库认不出这一份切出来的词）
      红格见 bash-command-detector.sh 文件头同名开关下
代码分在三份：lib_shell_words_core.py（切词、引号与替换、花括号、简单命令、前缀与包装、标准输入与 heredoc、变量与 cd、函数体）、
lib_shell_words_embedded.py（别的程序里起的命令 embedded_launches）与这一份（启动文件、别名、变量作用域、commands_at_command_position）；
这一份把另两份的名字全部再导出，调用方照旧只按这一份的路径导入、按这一份的名字取。SYSTEM_STARTUP_DIRECTORY 这类自证会换掉的模块变量留在读它的函数所在的这一份里。
另两份按这一份所在的目录载入、按目录各存一份：同一个进程里从两个目录各载入一次，后载入的用的是它自己目录里的部分模块；同一个目录里载入几次，用的是同一份部分模块；
sys.path 与裸名字的 sys.modules 载入前后不变（_begin_part_imports、_end_part_imports）。
"""
import os
import re
import shlex
import sys
from typing import NamedTuple

# 同目录的两份部分模块按这一份自己所在的目录载入：不按工作目录找（调用方按文件路径导入这一份，目录不一定在 sys.path 里），
# 也不沿用 sys.modules 里按裸名字缓存的那一份（同一个进程里从别的目录载入过这一份库时，那一份在别的目录里）。
# 部分模块按目录各存一份：同一个目录里载入几次这一份库（本进程里别的库也按文件路径载入它），用的是同一份部分模块（类与哨兵对象是同一个，isinstance 与 is 照旧对得上），
# 不同目录各用各的；存在 sys.modules 的「<部分模块名>@<目录>」键下。下面两条 from 导入之前，把这个目录放在 sys.path 最前、把 sys.modules 里裸名字的摘出来
# （这个目录已经载入过的，把那一份放到裸名字下，from 导入直接取它、不再读文件），导入之后核两份的文件确在这个目录里、存到目录键下，
# 把 sys.path 与裸名字放回导入之前的样子（这一份自己用的名字已经绑在本模块里）。
_LIBRARY_DIRECTORY = os.path.realpath(os.path.dirname(os.path.abspath(__file__)))   # 目录的写法不同（符号链接）也是同一个目录、同一份部分模块
_PART_MODULE_NAMES = ("lib_shell_words_core", "lib_shell_words_embedded")


def _part_module_key(name):
    return f"{name}@{_LIBRARY_DIRECTORY}"


def _begin_part_imports():
    """→ 导入之后要放回的现场 (摘出来的裸名字模块, 这个目录的部分模块是不是已经载入过, 导入后要不要按目录存)，或 None：
    弄坏开关 LIB_SHELL_WORDS_BREAK=part-modules-cached 下沿用 sys.modules 里裸名字缓存的同名部分模块、sys.path 里没有这个目录才加、不放回；
    =part-modules-per-load 下不按目录存，每载入一次这一份库就各载入一份自己的部分模块。"""
    breaks = os.environ.get("LIB_SHELL_WORDS_BREAK", "").split(",")
    if "part-modules-cached" in breaks:
        if _LIBRARY_DIRECTORY not in sys.path:
            sys.path.insert(0, _LIBRARY_DIRECTORY)
        return None
    per_load = "part-modules-per-load" in breaks
    saved = {name: sys.modules.pop(name) for name in _PART_MODULE_NAMES if name in sys.modules}
    stored = {name: sys.modules.get(_part_module_key(name)) for name in _PART_MODULE_NAMES}
    if not per_load and all(stored.values()):
        sys.modules.update(stored)
        return saved, True, True
    sys.path.insert(0, _LIBRARY_DIRECTORY)
    return saved, False, not per_load


def _end_part_imports(state):
    """把 sys.path 与裸名字放回导入之前的样子，这个目录的部分模块按目录存起来；两份部分模块有不在这个目录里的就报错（也不存），不带着别的目录的那一份往下走。"""
    if state is None:
        return
    saved, already_loaded, store = state
    if not already_loaded:
        sys.path.remove(_LIBRARY_DIRECTORY)
    modules = {name: sys.modules.pop(name) for name in _PART_MODULE_NAMES}
    sys.modules.update(saved)
    misplaced = [f"{name} 载入的是 {os.path.abspath(module.__file__)}" for name, module in modules.items()
                 if os.path.realpath(os.path.dirname(os.path.abspath(module.__file__))) != _LIBRARY_DIRECTORY]
    if misplaced:
        raise ImportError(f"{'；'.join(misplaced)}，不在这一份所在的目录 {_LIBRARY_DIRECTORY} 里")
    if store:
        for name, module in modules.items():
            sys.modules[_part_module_key(name)] = module


_part_imports_state = _begin_part_imports()
from lib_shell_words_core import (
    SHELL_NAMES,
    RENAMED_SHELL_NAMES,
    BUSYBOX_SHELL_APPLETS,
    STANDARD_INPUT_DEVICE,
    renamed_shells_unseen,
    is_shell_name,
    is_standard_input_device,
    expand_directory_tilde,
    SHELL_OPERATORS,
    OPERATOR_START,
    COMMAND_SEPARATORS,
    OUTPUT_REDIRECTS,
    REDIRECT_OPERATORS,
    COMMAND_KEYWORDS,
    PREFIX_OPTIONS_WITH_VALUE,
    PREFIX_POSITIONAL_COUNT,
    ASSIGNMENT,
    ARRAY_ASSIGNMENT_START,
    PYTHON_NAME,
    PYTHON_SHORT_OPTIONS_WITH_VALUE,
    PYTHON_LONG_OPTIONS_WITH_VALUE,
    SHELL_LONG_OPTIONS_WITH_VALUE,
    SOURCE_TARGET,
    HEREDOC,
    MAXIMUM_NESTING,
    WRAPPERS_TAKING_ONE_ARGUMENT,
    MEMORY_CAP_WRAPPER,
    HEREDOC_BODY_MARKER,
    RedirectOperator,
    redirection_fd,
    redirects_standard_output,
    standard_output_destination,
    BASH_BUILTINS,
    HeredocBody,
    PrefixSkip,
    ShellInvocation,
    PythonInvocation,
    CommandAtPosition,
    ShellScan,
    heredoc_feeds_shell,
    open_substitution_tail,
    shell_reads_standard_input,
    strip_data_heredocs,
    ANSI_C_ESCAPES,
    ansi_c_quotes_unseen,
    ansi_c_quote_end,
    ansi_c_decoded,
    backquote_end,
    double_quote_end,
    substitution_case_and_comment_unseen,
    SUBSTITUTION_WORD_END,
    SUBSTITUTION_COMMAND_KEYWORDS,
    command_substitution_end,
    starts_substitution,
    substitution_span,
    shell_tokens,
    UnfoldableCase,
    case_patterns_and_conditionals_folded,
    BRACE_EXPANSION_LIMIT,
    BRACE_SEQUENCE,
    QUOTED_EMPTY,
    ShellWord,
    quoted_braces_as_unknown,
    brace_expand,
    brace_expand_once,
    brace_sequence_width,
    brace_sequence,
    simple_commands,
    words_break_is_set,
    env_chdir_ignored,
    env_directory,
    EnvOption,
    ENV_SHORT_OPTIONS_WITH_VALUE,
    ENV_LONG_OPTIONS_WITH_VALUE,
    env_option_clusters_unparsed,
    env_split_string_unexpanded,
    env_option,
    env_option_clears_environment,
    ENV_SPLIT_ESCAPES,
    env_split_string,
    skip_prefixes_expanding,
    skip_prefixes,
    arguments_as_executed,
    shell_invocation,
    python_options_by_dash,
    python_invocation,
    resolve_path,
    single_leading_slash,
    link_source_word,
    STANDARD_INPUT_REDIRECTS,
    PIPE_SEPARATORS,
    BACKSLASH_ESCAPES,
    PRINTF_CONVERSION,
    ECHO_OPTION,
    UNJUDGED_SNIPPET_CHARACTERS,
    StandardInputCode,
    shell_standard_input_unread,
    backslash_escapes,
    printf_output,
    echo_output,
    has_expansion,
    unjudged_snippet,
    piped_literal_output,
    shell_standard_input_files,
    shell_standard_input,
    AMBIENT_STDIN_OPERATORS,
    CompoundStdinFeeds,
    COMPOUND_CLOSING_WORDS,
    COMPOUND_PIPED_OPENINGS,
    compound_stdin_closers_unseen,
    compound_redirected_stdin_files,
    VARIABLE_REFERENCE,
    REBINDING_COMMANDS,
    APPEND_ASSIGNMENT,
    ELEMENT_ASSIGNMENT,
    SUBSCRIPTED_ASSIGNMENT,
    DECLARATION_COMMANDS,
    directory_variables_unexpanded,
    variable_rebinding_unseen,
    expand_known_variables,
    assigned_value,
    rebind_variables,
    array_appends_unseen,
    record_array_writes,
    GIT_TOPLEVEL_SUBSTITUTION,
    PWD_SUBSTITUTION,
    GIT_LOCATION_VARIABLES,
    cd_known_targets_unexpanded,
    cd_failure_guards_unseen,
    git_toplevel,
    changed_directory,
    SUBSHELL_PREFIX_KEYWORDS,
    guarded_subshell_boundary,
    cd_failure_exits,
    FUNCTION_CALLING_WORDS,
    IMPLICITLY_CALLED_FUNCTIONS,
    drop_uncalled_function_bodies,
    function_never_called,
    POSITIONAL_COMMAND_WORD,
    function_bodies,
    function_body_starts_shell,
    shell_starting_functions,
    heredocs_fed,
)
from lib_shell_words_embedded import (
    ProgramFile,
    EmbeddedLaunches,
    EMBEDDED_PYTHON_SHELL_CALLS,
    EMBEDDED_PYTHON_SUBPROCESS_CALLS,
    EMBEDDED_PYTHON_EXEC_LIST_CALLS,
    EMBEDDED_PYTHON_EXEC_VECTOR_CALLS,
    EMBEDDED_PYTHON_SPAWN_LIST_CALLS,
    EMBEDDED_PYTHON_SPAWN_VECTOR_CALLS,
    EMBEDDED_PYTHON_LAUNCH_MODULES,
    EMBEDDED_PYTHON_OPAQUE_MODULES,
    EMBEDDED_PYTHON_DISTINCT_LAUNCH_NAMES,
    EMBEDDED_PYTHON_SYNTAX_HINTS,
    PERL_LAUNCH_WORD,
    RUBY_LAUNCH_WORD,
    NODE_LAUNCH_WORD,
    PHP_LAUNCH_WORD,
    LUA_LAUNCH_WORD,
    TCL_LAUNCH_COMMANDS,
    TCL_OPAQUE_COMMANDS,
    AWK_NAMES,
    PERL_NAME,
    PERL_OPTIONS_WITH_VALUE_WORD,
    PERL_OPTIONS_WITH_ATTACHED_VALUE,
    RUBY_NAME,
    PHP_NAME,
    LUA_NAME,
    TCL_NAMES,
    TCL_NAME,
    NODE_NAMES,
    VIM_NAMES,
    MAKE_NAMES,
    MAKE_DEFAULT_FILES,
    GIT_COMMAND_CONFIG_KEY,
    GIT_CONFIG_QUERY_PATTERN,
    GIT_BOOLEAN_VALUES,
    GIT_COMMAND_ENVIRONMENT,
    GIT_OPAQUE_ENVIRONMENT,
    GIT_OPTIONS_WITH_VALUE,
    GIT_CONFIG_QUERY_SECONDS,
    GIT_CONFIG_QUERY_CACHE,
    TAR_COMMAND_OPTIONS,
    SSH_COMMAND_OPTIONS,
    LESS_OPEN_ENVIRONMENT,
    MAKE_DEFAULT_VARIABLES,
    MAKE_RECIPE_PREFIX,
    MAKE_ASSIGNMENT,
    MAKE_VARIABLE_PLACEHOLDER_PREFIX,
    EMBEDDED_NESTING,
    embedded_launches_unseen,
    perl_attached_options_take_next_word,
    heredoc_variables_unexpanded,
    heredoc_body_expanded,
    interpreter_standard_input,
    literal_quoted_string,
    literal_argument_list,
    launched_command_text,
    string_spans,
    inside_spans,
    scan_quoted_language,
    python_literal,
    PythonUnresolvedElement,
    PYTHON_UNRESOLVED_ELEMENT,
    PythonUnresolvedTail,
    PYTHON_UNRESOLVED_TAIL,
    ARGUMENT_COMMAND_LAUNCHER_NAMES,
    ARGUMENT_COMMAND_PROGRAM_NAMES,
    SCRIPT_FILE_SUFFIXES,
    PYTHON_UNRESOLVED_ARGUMENT,
    PYTHON_READ_ONLY_BUILTINS,
    PYTHON_LIST_MUTATING_METHODS,
    python_argument_variables_unresolved,
    python_constant_bindings,
    python_list_concatenation_unresolved,
    python_list_tail,
    python_tail_command_word_judged,
    python_command_value,
    python_argv_text,
    scan_python,
    tcl_words,
    scan_tcl,
    awk_tokens,
    AWK_ASSIGNMENT_ENDS,
    AWK_CHANGING_OPERATORS,
    AWK_TARGET_BUILTINS,
    awk_command_variables_unresolved,
    awk_command_variables,
    scan_awk,
    SED_SUBSTITUTION_ADDRESS,
    SED_ARITHMETIC_VALUE,
    SED_EXPANSION_LIMIT,
    GNU_SED_COMMAND_CHARACTERS,
    SED_SUBSTITUTE_FLAG_CHARACTERS,
    SED_VARIABLE,
    sed_expansions_unparsed,
    sed_text_first_line_only,
    SedParse,
    sed_script_expansions,
    sed_expanded_variants,
    sed_script_commands,
    sed_script_parse,
    make_expanded,
    scan_makefile,
    program_text_launches,
    git_config_commands,
    embedded_option_variables_unexpanded,
    scan_git,
    scan_options_for_commands,
    interpreter_code_options,
    embedded_launches,
)
_end_part_imports(_part_imports_state)
del _part_imports_state


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


def alias_start_options_unseen():
    """弄坏开关 LIB_SHELL_WORDS_BREAK=alias-start-options-unseen：起 shell 时就开着别名展开的（-O expand_aliases、-i、posix 模式、sh / dash / zsh 这类、读启动文件的）
    照不开算，set -o posix 也不算开（只给自检证红用）。"""
    return words_break_is_set("alias-start-options-unseen")


def alias_nested_texts_unseen():
    """弄坏开关 LIB_SHELL_WORDS_BREAK=alias-nested-texts-unseen：eval、trap、命令替换与 source /dev/stdin 的代码不带外面定义的别名，eval 里定义的别名不收（只给自检证红用）。"""
    return words_break_is_set("alias-nested-texts-unseen")


def shell_alias_definitions(text, definition_lines, depth=0, eval_line=None):
    """text 里的别名定义与开没开展开：交 (名字 -> 值, 开没开)。`shopt -s expand_aliases` 与 `set -o posix`（posix 模式的 bash 照展开别名）算开；
    eval 的参数照一段代码接着找（eval 跑的时候才切，那里面定义的别名对后面的行同样生效），定义的行记成 eval 所在的那一行。"""
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
        elif (head == "set" and not alias_start_options_unseen()
              and ("-oposix" in rest[1:] or any(rest[position:position + 2] == ["-o", "posix"] for position in range(1, len(rest))))):
            enabled = True
        elif head == "alias":
            for argument in rest[1:]:
                assignment = ASSIGNMENT.match(argument)
                if assignment:
                    found[assignment.group(1)] = assignment.group(2)
                    if definition_lines is not None:
                        definition_lines.setdefault(assignment.group(1), set()).add(line if eval_line is None else eval_line)
        elif head == "eval" and len(rest) > 1 and depth < MAXIMUM_NESTING and not alias_nested_texts_unseen():
            inner_found, inner_enabled = shell_alias_definitions(" ".join(rest[1:]), definition_lines, depth + 1, line if eval_line is None else eval_line)
            found.update(inner_found)
            enabled = enabled or inner_enabled
    return found, enabled


def shell_aliases(text, definition_lines=None, enabled_from_start=False):
    """text 里 `alias 名字='值'` 定义的别名（名字 -> 值，值照字面当命令再切）；只在文本里有 `shopt -s expand_aliases` 或 `set -o posix` 开着、
    或跑这段文本的 shell 起来时就开着（enabled_from_start，shell_starts_with_aliases）时才交（非交互、不在 posix 模式的 bash 默认不展开别名）。
    eval 的参数里定义的同样收（shell_alias_definitions）。弄坏开关 LIB_SHELL_WORDS_BREAK=aliases-unexpanded 下交空。
    definition_lines 是一个 dict 时，按名字记下定义它的那几行（从 0 数，照切好的分隔符里的换行数；commands_at_command_position 拿它认同一行里定义的不展开）。"""
    if aliases_unexpanded() or "alias" not in text:
        return {}
    found, enabled = shell_alias_definitions(text, definition_lines)
    return found if enabled or enabled_from_start else {}


def shell_starts_with_aliases(name, arguments, environment):
    """这个 shell 起来时就开着别名展开（认不准开没开的照开着算，偏严）：sh、dash、ksh、ash、zsh（非交互也展开别名）；bash 带 `-O expand_aliases`、
    带 -i、posix 模式（bash_in_posix_mode），或起来时要先读启动文件（startup_file_words：BASH_ENV、--rcfile 这类，文件里可能 shopt -s expand_aliases）。
    弄坏开关 LIB_SHELL_WORDS_BREAK=alias-start-options-unseen 下一律交假。"""
    if alias_start_options_unseen():
        return False
    if name != "bash":
        return name in POSIX_STARTUP_SHELL_NAMES or name == "zsh"
    if shell_start_options(name, arguments).interactive or bash_in_posix_mode(arguments, environment):
        return True
    position = 0
    while position < len(arguments):
        argument = arguments[position]
        if argument in ("-", "--") or not argument.startswith(("-", "+")):
            break
        if argument == "-O" and arguments[position + 1:position + 2] == ["expand_aliases"]:
            return True
        position += 2 if argument in ("-o", "+o", "-O", "+O") or argument in SHELL_LONG_OPTIONS_WITH_VALUE else 1
    return bool(startup_file_words(name, arguments, environment))


def variable_scopes_unseen():
    """弄坏开关 LIB_SHELL_WORDS_BREAK=variable-scopes-unseen：变量不分作用域，子 shell、管道段、放后台的、`&&` / `||` 之后、if / case / 循环 / 函数体里赋的值
    照旧当对后面的命令都生效（只给自检证红用）。"""
    return words_break_is_set("variable-scopes-unseen")


SCOPE_OPENING_WORDS = frozenset({"{", "if", "while", "until", "for", "select", "case"})
SCOPE_CONDITIONAL_OPENINGS = frozenset({"if", "while", "until", "for", "select", "case"})   # 里面的命令跑不跑、跑几遍由条件定
SCOPE_CLOSING_WORDS = frozenset({"}", "fi", "done", "esac"})
SCOPE_PASSING_WORDS = frozenset({"then", "do", "else", "elif", "!", "time"})


def forget_changed_variables(before, variables, known_only):
    """variables 里与 before 不同的名字记成算不出（None）。known_only 为真时只管 before 里记过的名字
    （条件赋值：之前没记过的，没跑的那一路本来就展开不了，留着新值）。"""
    for name, value in list(variables.items()):
        if name in before:
            if before[name] != value:
                variables[name] = None
        elif not known_only:
            variables[name] = None


class VariableScopes:
    """commands_at_command_position 记变量时的作用域（认不准的记成算不出，偏严；不照 bash 一条条还原外面的值）。每条简单命令记它的赋值之前调 before_command：
    ① 圆括号子 shell：`(` 处记下此刻的变量，`)` 处里面改过的名字记成算不出（子 shell 里赋的不带出来）；
    ② 管道段与放后台的：前面紧挨 `|`、`|&`，或后面紧跟 `|`、`|&`、单独的 `&` 的这一条，它改过的名字在下一条之前记成算不出（各段在子 shell 里跑）；
       花括号组与 if / case / 循环整个接在管道里或放后台的，在收尾词处照同一条记；
    ③ `&&` / `||` 之后的条件赋值：一串 `&&` 里接着用的照新值（前面不成这一条根本不跑）；这一串断开（`;`、换行、`&`、括号）或遇到 `||` 时，
       之前记过值、又被改成别的的名字记成算不出；
    ④ if / case / 循环 / 函数体（与紧跟在 `&&` / `||` 后面起头的组）：收尾词处，之前记过值、里面改成别的的名字记成算不出。
    循环体里后一遍用到前一遍赋的值（回边）不认，照第一遍的值判。弄坏开关 LIB_SHELL_WORDS_BREAK=variable-scopes-unseen 下不建它（调用方照旧不分作用域）。"""

    def __init__(self):
        self.parentheses = []        # 每个没收尾的 `(` 处的变量
        self.groups = []             # 每个没收尾的组：(起头词，函数体记成 "function", 起头处的变量, 接在管道后面, 紧跟在 `&&` / `||` 后面)
        self.and_or_before = None    # 这一串 `&&` / `||` 里头一处条件赋值之前的变量；不在条件里是 None
        self.or_seen = False         # 这一串里从 and_or_before 记下起遇到过 `||`
        self.segment_before = None   # 上一条是管道段或放后台时，它之前的变量

    def flush_and_or(self, variables):
        if self.and_or_before is not None:
            forget_changed_variables(self.and_or_before, variables, True)
        self.and_or_before, self.or_seen = None, False

    def before_command(self, boundary, words, following, variables):
        """boundary：这一条与前一条之间的分隔符（simple_commands 的 boundaries，没经 guarded_subshell_boundary 改过）；following：这一条与后一条之间的。"""
        if self.segment_before is not None:
            forget_changed_variables(self.segment_before, variables, False)
            self.segment_before = None
        operator = boundary[0] if boundary and boundary[0] in ("&&", "||") else None
        if operator is None:
            self.flush_and_or(variables)
        elif operator == "||" or self.or_seen:
            self.flush_and_or(variables)
            self.and_or_before, self.or_seen = dict(variables), operator == "||"
        elif self.and_or_before is None:
            self.and_or_before = dict(variables)
        for separator in boundary:
            if separator == "(":
                self.parentheses.append(dict(variables))
            elif separator == ")" and self.parentheses:
                forget_changed_variables(self.parentheses.pop(), variables, False)
        piped_in = bool(boundary) and boundary[0] in PIPE_SEPARATORS
        piped_out = bool(following) and (following[0] in PIPE_SEPARATORS or following[0] == "&")
        if words and words[0] in SCOPE_CLOSING_WORDS and self.groups:
            opening, before, group_piped_in, conditional_entry = self.groups.pop()
            if group_piped_in or piped_out:
                forget_changed_variables(before, variables, False)
            elif opening in SCOPE_CONDITIONAL_OPENINGS or opening == "function" or conditional_entry:
                forget_changed_variables(before, variables, True)
            return
        function_body = [separator for separator in boundary if separator != "\n"][-2:] == ["(", ")"]   # `名字() {` 的花括号
        position, opened = 0, False
        while position < len(words):
            word = words[position]
            if word == "function":
                position, function_body = position + 2, True   # `function 名字 {`：关键字与函数名之后的花括号是函数体
                continue
            if word in SCOPE_OPENING_WORDS:
                self.groups.append(("function" if word == "{" and function_body else word, dict(variables), piped_in and not opened, operator is not None))
                position, opened, function_body = position + 1, True, False
                continue
            if word not in SCOPE_PASSING_WORDS:
                break
            position += 1
        if (piped_in or piped_out) and not opened:
            self.segment_before = dict(variables)


def commands_at_command_position(text, directory=None, environment=None, depth=0, stripped_heredocs=None, memory_capped=False, variables=None,
                                 uncalled_functions_skipped=False, wrappers=(), directory_guarded=True, interactive=False,
                                 inherited_aliases=None, aliases_enabled=False, alias_same_line=True):
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
    interactive：跑这段文本的 shell 可能是交互的（递归进 bash -i 的代码、脚本照那个 shell 改），记进每条命令的 interactive。
    别名（认不准开没开展开的照开着算，偏严）：aliases_enabled 是跑这段文本的 shell 起来时就开着别名展开（shell_starts_with_aliases；递归进 bash -c、喂给 shell 的标准输入时由这里给）；
    inherited_aliases 是外面那一层此刻生效的别名（eval、trap、命令替换、source /dev/stdin 的代码跑的时候才切，外面先定义的照展开，不看同一行）；
    alias_same_line 为假时同一行里定义的照展开（同一行不展开只照 bash 认，sh / dash / zsh 这类认不准照展开）。
    变量的作用域（VariableScopes，认不准的记成算不出，偏严）：圆括号子 shell、管道段、放后台的、`&&` / `||` 之后、if / case / 循环 / 函数体里赋的值，离开那一段之后照 VariableScopes 的规则记。"""
    if stripped_heredocs is None:
        stripped_heredocs = []
    tokens = shell_tokens(strip_data_heredocs(text, stripped_bodies=stripped_heredocs))
    undropped_tokens = tokens
    if uncalled_functions_skipped and depth == 0:
        tokens = drop_uncalled_function_bodies(tokens, text, stripped_heredocs)
    alias_lines = {}
    aliases = shell_aliases(text, alias_lines, aliases_enabled)   # shopt -s expand_aliases 开着时，命令词是别名的展开
    if inherited_aliases and not alias_nested_texts_unseen():
        aliases = {**inherited_aliases, **aliases}   # 外面那一层先定义的：这段代码跑的时候才切，照展开
    # eval、trap 的代码在同一个 shell 里跑：这一层开着展开（有生效的别名，或起来时就开着），它们的代码也照开着切
    nested_aliases_enabled = (bool(aliases) or aliases_enabled) and not alias_nested_texts_unseen()
    # 同一行里定义的别名这一行不展开（bash 读完一整行才执行；复跑分类报告 d137-replay-classify 二节 E4）：行号照分隔符里的换行数，与 shell_aliases 记的对得上才认；
    # 去掉过只定义的函数体的（行号对不上）照旧展开。弄坏开关 LIB_SHELL_WORDS_BREAK=aliases-same-line-expanded 下不看行号
    if tokens is not undropped_tokens or words_break_is_set("aliases-same-line-expanded") or not alias_same_line:
        alias_lines = None
    scopes = None if variable_scopes_unseen() else VariableScopes()
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
        if scopes is not None:
            scopes.before_command(boundary, words, boundaries[index + 1] if index + 1 < len(boundaries) else (), variables)
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
                                                     directory_guarded=command_guarded, interactive=interactive, inherited_aliases=aliases)
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
                # 命令词是开着展开时定义的别名：展开成别名的值再切；
                # 值的头一个词是别的别名的照 bash 接着展开（带进去的别名去掉正在展开的这一个：同一个名字不再展开第二遍、不会无限递归）
                expanded_text = aliases[remaining[0]] + ("".join(" " + shlex.quote(argument) for argument in arguments))
                inner = commands_at_command_position(expanded_text, command_directory, command_environment, depth + 1, stripped_heredocs,
                                                     command_memory_capped, variables, wrappers=command_wrappers,
                                                     directory_guarded=command_guarded, interactive=attached["interactive"],
                                                     inherited_aliases={alias: value for alias, value in aliases.items() if alias != remaining[0]})
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
                # 参数里外层先展开的变量与命令替换照原文切（展开成什么判不到，变量里拼出来的命令照旧认不出）；
                # eval 跑的时候才切它的代码：外面先定义的别名照展开（同一行定义的也算），开没开照外面这一层
                if depth < MAXIMUM_NESTING:
                    inner = commands_at_command_position(" ".join(arguments), command_directory, command_environment, depth + 1, stripped_heredocs,
                                                         command_memory_capped, variables, wrappers=command_wrappers,
                                                     directory_guarded=command_guarded, interactive=attached["interactive"],
                                                     inherited_aliases=aliases, aliases_enabled=nested_aliases_enabled)
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
                                                     directory_guarded=command_guarded, interactive=attached["interactive"],
                                                     inherited_aliases=aliases, aliases_enabled=nested_aliases_enabled)
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
                # 新起的 shell 不带外面的别名；它起来时开没开别名展开照 shell_starts_with_aliases（认不准的照开着算），同一行不展开只照 bash 认
                started_with_aliases = shell_starts_with_aliases(shell_name, arguments, command_environment)
                same_line_rule = shell_name == "bash" or alias_start_options_unseen()
                if invocation.code_string is not None:
                    if depth < MAXIMUM_NESTING:
                        inner = commands_at_command_position(invocation.code_string, command_directory, command_environment, depth + 1, stripped_heredocs,
                                                             command_memory_capped, variables, wrappers=command_wrappers,
                                                     directory_guarded=command_guarded, interactive=attached["interactive"],
                                                     aliases_enabled=started_with_aliases, alias_same_line=same_line_rule)
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
                                                     directory_guarded=command_guarded, interactive=attached["interactive"],
                                                     aliases_enabled=started_with_aliases, alias_same_line=same_line_rule)
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
                                                     directory_guarded=command_guarded, interactive=attached["interactive"],
                                                     inherited_aliases=aliases, aliases_enabled=nested_aliases_enabled)
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


if __name__ == "__main__":
    # 这份库没有命令行入口（文件头「弄坏开关」：由导入它的几份的自证判红）；直接执行时设着 LIB_SHELL_WORDS_BREAK 一律拒，不管值是什么：弄坏开关只在导入它的几份的自证里认。
    import sys
    if os.environ.get("LIB_SHELL_WORDS_BREAK"):
        print(f"  ✗ LIB_SHELL_WORDS_BREAK={os.environ['LIB_SHELL_WORDS_BREAK']} 是弄坏开关，只在导入这份库的几份的自证（bash-command-detector.sh、project-code-guard.sh 等的 --selftest）里认，直接执行设了就拒，这一趟什么都没做", file=sys.stderr)
        print("  → 怎么办：去掉 LIB_SHELL_WORDS_BREAK 再跑；要证红就跑 LIB_SHELL_WORDS_BREAK=<项> bash .claude/hooks/project-code-guard.sh --selftest（项与红的格写在文件头「弄坏开关」一段）", file=sys.stderr)
        sys.exit(2)
