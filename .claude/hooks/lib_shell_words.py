"""Bash 命令文本的切词、切简单命令、认命令位置、剥前缀与包装、跟 cd。

.claude/hooks/ 下判 Bash 命令的 hook（bash-command-detector.sh、heavy-test-guard.sh）共用这一份：
各自按文件路径用 importlib 导入，只留自己的判定。这里 def 的函数名在 .claude/hooks/ 别的文件里再 def 一份，
门禁 code-tooling 的 agent-write-scope 格判红——一份改了另一份不跟，同一条命令一个 hook 认得出、另一个认不出。

认得出的：引号与反斜杠（ANSI-C 引号 `$'…'` 里的转义照 bash 换掉、`\\'` 不收尾：ansi_c_quote_end、ansi_c_decoded）、词首的 # 注释、重定向（连同目标去掉，输出重定向的目标另交）、
命令替换 `$(…)` 与反引号（引号里外都认：整段留在所在的词里，里面的命令另按一条完整的命令递归判；`$((…))` 算术整段留在词里、不判）、
进程替换 `<(…)` 与 `>(…)`（只在引号外认，办法同命令替换：整段留在所在的词里，`)` 之后的词照旧是外层命令的参数）、
命令前的关键字与赋值（`名字+=值`、`名字[下标]=值` 同样是赋值；time 关键字后面的一个 -p 与一个 --、function 关键字连同函数名、builtin 连同它后面的 -- 一并跳过；builtin 后面不是 bash 内建的（`builtin cargo …`，bash 报错不跑）这一条没有命令词；只定义、调不到的 `function 名字 { …; }` 的函数体，调用方给了 uncalled_functions_skipped 时不判：drop_uncalled_function_bodies；
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
cd / pushd / env -C 的目录里写成变量的，同一段命令文本里前面赋过的（单独成一条的 `名字=值`、export、local / declare 这类）与 `$HOME`、`~` 先展开再判
（expand_known_variables；记下的值交在 CommandAtPosition.variables，调用方拿它展开别的参数）；展开不了的（没在这段文本里赋值、for / select / read / unset 改写过、
`$(…)`、`${名字:-默认}` 这类写法）照旧认不出：cd 之后的目录是 None，env -C 照外面的当前目录。
剥掉的 heredoc 正文与输出重定向挂回它所在的那条命令（CommandAtPosition 的 standard_input_heredocs、output_redirections）：
要认「同一条命令里先由 heredoc 写出文件、再执行它」的 hook 从这两格读，不自己再切一遍。重定向前的 fd 号挂在运算符上（RedirectOperator.fd，运算符照旧与 `>` 这些逐字相等；只认整个是引号外十进制数字的词，`01>` 是 1 号，`"2">`、`\\2>` 的 2 是参数）：
认这一条的标准输出写到哪的用 redirects_standard_output（`2> 文件` 不算）与 standard_output_destination（照 bash 从左到右跟 `n>&m`，`3>x >&3` 的标准输出进 x）。
case 语句的模式（`in` 与 `;;` / `;&` / `;;&` 之后到右括号为止的词）与 `[[ … ]]` 里的 `&&`、`||`、括号、`<`、`>` 不当命令位置、命令分隔与重定向（case_patterns_and_conditionals_folded）。`ln -s` 的相对源从链接所在的目录算（link_source_word，两个 hook 认拷出再执行时共用）。
剥掉包装之后命令词是一个脚本时，记下它是怎么起的（launcher：bash / sh 这类后面跟的、source / . 读进来的、python3 后面跟的）；
经 `run-with-memory-cap.sh <上限>` 剥出来的命令记 memory_capped，连同它 `bash -c` 里的命令；调用方往里读它起的脚本时，把这个标记当 memory_capped 参数传回来。
从标准输入读命令的 shell（bash / sh 这类不带 -c、不带脚本，或带 -s：shell_reads_standard_input）收到的字面文本当这个 shell 的代码再切一遍，
同 bash -c 一样挂回（它那一条命令的包装、目录、变量、memory_capped 照算；shell_standard_input）：here-string `<<<` 的文字；管道前一段是 printf（格式串与参数照 printf 现算）、
echo（-n / -e / -E 照 bash 内建认）、不带文件操作数而读 heredoc 的 cat 时它们交出的文字。喂进去的不是字面的判不了，每处一句说明交在 ShellScan.unjudged_shell_inputs：
here-string 或 printf / echo 的参数里有 `$`、反引号（变量、命令替换；单引号里写的同样算，切词时分不出，字面的那一部分照切）、不带引号结束词的 heredoc 正文里有它们、
管道前一段是别的命令（cat 文件、花括号组、子 shell……）、标准输入从文件或别的描述符来（`< 文件`、`<&`、`<>`；`< /dev/null` 不算）。调用方按各自的「判不了」处置（放行并记检出）。
弄坏开关 LIB_SHELL_WORDS_BREAK=shell-standard-input-unread 退回改前的判法（不再切、也不交说明），哪几份自证的哪几项红见「弄坏开关」一段。
脚本文件本身不在这里读，要读的 hook 自己读，再把正文交回 commands_at_command_position。
只有一处例外：detached_run_script 认 scripts/hosts/detached-run.sh（名字、符号链接、文件开头的签名行），读命令词那份文件的开头 4 KiB，两个 hook 共用这一份认法。
detached-run.sh 的参数先照它的 preflight 摘掉每一个 `--force`（连 `--` 之后的、连写在选项值位置上的），再切 start（detached_run_arguments_after_preflight）：
闸看的那条命令与单元里真跑的逐字相同（弄坏开关 LIB_SHELL_WORDS_BREAK=detached-run-force-kept 退回只跳过子命令前与选项之间的 --force，
哪几份自证的哪几项红见「弄坏开关」一段）。
判不到的：变量里拼出来的命令、eval 的参数里外层先展开的变量与命令替换（展开成什么判不到，照原文切）、喂给 shell 的标准输入不是字面的（交判不了的说明，见上）、管道前一段是花括号组或子 shell 时组里 printf / echo 的文字（整组当判不了）、
进程替换的输出再交给 shell 执行（`bash <(…)`、`source <(…)`，括号里的命令照判，它打印出来的不判）、Makefile 与 xargs 起的命令；
喂给 shell 的 heredoc 正文里引号没配对时，其后的切词会错；heredoc 挂的命令从上一行续行过来时认不出它喂给谁（当数据剥掉），没收尾的替换开在上一行的同样认不出；圆括号子 shell 里的 cd 与 export 当成对后面的命令也生效；
花括号展开不分 `[[ ]]`、case 模式这类 bash 不展开的位置（照展，判得偏严）；只定义的函数只认 `function 名字 { …; }` 一种写法，`名字() { …; }`、函数体里套花括号的、函数体之外有变量或命令替换的照判（偏严）；`builtin` 认的是 bash 5.2 的内建表（enable -f 载进来的不认，照不跑放行）；
命令替换与进程替换里的命令按替换所在那条命令开始时的目录与 export 判（替换里的 cd 不影响外面）。
变量展开判不到的：值从上一条 Bash、source 进来的文件、eval 来的（目录照旧认不出）；单引号里的 `$名字` 照样展开（引号在切词时已去掉，分不出）；
bash -c 的那段代码按外层记下的全部变量展开（没 export 的变量里层真跑时看不到，这里照样展开，判得偏严）；圆括号子 shell 里赋的值当成对后面的命令也生效。
弄坏开关（LIB_SHELL_WORDS_BREAK，几个用逗号连；只给自证判红用）。这一份没有自己的 --selftest，由导入它的几份的自证判红：
lib_heavy_tests.py、heavy-test-guard.sh、bash-command-detector.sh、scripts/hosts/detached-run.sh 各自的 --selftest（目录变量那两个开关另有 lib_workspace_cargo_lock.py 的 --selftest）。
逐项写开关名、弄坏哪一处、打开后哪几份自证的哪几项红（项名是那份自证的用例标签，括号里是失败行在标签之后的开头；
失败行 lib_heavy_tests.py 以「✗ lib_heavy_tests 自检：」起头，heavy-test-guard.sh、bash-command-detector.sh 以「✗ 自检：」起头，
detached-run.sh 以「✗ 自证格不对：」起头、标签之后不另写期望值；heavy-test-guard.sh 的自证还原样转出 lib_heavy_tests.py 自证的失败行）：
  LIB_SHELL_WORDS_BREAK=arguments-as-executed  `bash -n 脚本` 的目标与 `python3 - 词…` 后面的词照旧当被执行的脚本交出
      lib_heavy_tests.py --selftest：
        python3 - 后面的门禁文件名是参数（门禁目录里跑）（「应当是 None，实际 commit-check-stage」）
        bash -n 只查门禁的语法（「应当是 None，实际 commit-check-stage」）
      heavy-test-guard.sh --selftest（另转出 lib_heavy_tests.py 那两项）：
        工具实现员 bash -n 查门禁的语法（「应当是 0，实际 2」）
        工具实现员在门禁目录里 python3 - 读几份门禁（2026-09-28 误拦的那一条原样，目录换成自检的临时工作区）（「应当是 0，实际 2」）
        实现员 bash -n 只查语法（「应当是 0，实际 2」）
        lib_heavy_tests.py 的自检（「应当是 0，实际 1」）
      bash-command-detector.sh --selftest：
        看门狗:前台 bash -n 只查语法（「应当是 0，实际 1」）
      detached-run.sh --selftest 不红
  LIB_SHELL_WORDS_BREAK=python-options-by-dash  python_invocation() 退回「以 - 开头的词都当选项跳过、第一个不以 - 开头的词当脚本」：带值选项的值当成脚本名，-m、-c 不认
      lib_heavy_tests.py --selftest：
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
      heavy-test-guard.sh --selftest（另转出 lib_heavy_tests.py 那 23 项）：
        lib_heavy_tests.py 的自检（「应当是 0，实际 1」）
      bash-command-detector.sh、detached-run.sh 的 --selftest 不红
  LIB_SHELL_WORDS_BREAK=detached-run-force-kept  detached_run_arguments_after_preflight() 原样交回参数：-- 之后、选项值位置上的 --force 不摘
      lib_heavy_tests.py --selftest：
        detached-run.sh start 的 -- 之后先写 --force 照摘（「应当是 full-cargo，实际 None」）
      heavy-test-guard.sh --selftest（另转出 lib_heavy_tests.py 那一项）：
        主 agent 不带前缀经 detached-run.sh start、-- 之后先写 --force 再起全量照拒（「应当是 2，实际 0」）
        主 agent 不带前缀经 detached-run.sh start、--label 后面先写 --force 再起全量照拒（「应当是 2，实际 0」）
        主 agent 不带前缀经 detached-run.sh start、--label 后面先写 --force 再起全量照拒：读进去几份脚本（「应当是 0，实际 1」）
        lib_heavy_tests.py 的自检（「应当是 0，实际 1」）
      bash-command-detector.sh --selftest：
        放出追踪:detached-run.sh start 的 -- 之后先写 --force、里面 setsid -f 照拒（「应当是 1，实际 0」）
      detached-run.sh --selftest：
        Ⓟ -- 之后的 --force 照 preflight 摘：命令收到 a b，闸切出的词与登记里的命令逐字相同
  LIB_SHELL_WORDS_BREAK=array-appends-unseen  record_array_writes() 什么都不记：`名字+=值`、`名字[下标]=值` 照旧不改记下的值（改前的判法）
      lib_heavy_tests.py --selftest：cargo 那张表里「重型：崩溃点重放入口 --quick 的项数组后来追加了 --full」「重型：crates 变异那一道 --quick 的项数组元素改过」
        「重型：crates 变异那一道 --quick 的项数组从文件逐行读进来」三格（「应当是 crash-point-replay / crates-mutation-stage，实际 None」）
      heavy-test-guard.sh --selftest（另转出 lib_heavy_tests.py 那三项）：
        实现员经内存包装起 crates 变异那一道 --quick、数组后来追加了 --full：照拒（「应当是 2，实际 0」）
        lib_heavy_tests.py 的自检（「应当是 0，实际 1」）
      bash-command-detector.sh、detached-run.sh 的 --selftest 不红
  LIB_SHELL_WORDS_BREAK=env-chdir-ignored  skip_prefixes() 不交 env 的 -C / --chdir 换的目录：commands_at_command_position 里这一条命令照外面的当前目录判（改前的判法）
      lib_heavy_tests.py --selftest：cargo 那张表里「env 换目录」开头的十一格里十格红（「换的目录只管这一条」那格不红）：
        env 换目录：-C 换进探针再 cargo test -p 探针自己的包名（「应当是 checker-tier-cargo，实际 None」）
        env 换目录：-C 换进 checker 档包目录裸跑、-C 贴着目录写、--chdir=目录、--chdir 目录（分两个词）、bash -c 里的 env -C（「应当是 checker-tier-cargo，实际 full-cargo」）
        env 换目录：绝对路径、当前目录在别处、经内存包装（「应当是 checker-tier-cargo，实际 None」）
        env 换目录：换进 harness 档目录裸跑放行、换进不依赖 checker 档包的探针裸跑放行、两个 -C 取最后一个（「应当是 None，实际 full-cargo」）
        另有「变量目录裸跑：」那一批的「export D 之后经内存包装 env -C $D/checker 档包目录」一格（「应当是 checker-tier-cargo，实际 None」）
      heavy-test-guard.sh --selftest（另转出 lib_heavy_tests.py 那十一项）：
        实现员经内存包装 env -C 换进探针再 cargo test -p 探针自己的包名、env -C 换进 checker 档包目录裸跑、env --chdir=目录换进 checker 档包目录、env --chdir 目录（分两个词）、
        主 agent 不带前缀 env -C 换进链接 checker 档包的探针 cargo run（「应当是 2，实际 0」）
        实现员经内存包装 env -C 换进不依赖 checker 档包的探针裸跑 cargo test：放行（「应当是 0，实际 2」）
        实现员经内存包装 env -iC 合写换进 checker 档包目录 cargo test：照拒、实现员经内存包装 env -S 拆串里先换目录再 cargo test：照拒（「应当是 2，实际 0」）
        实现员 export D 之后经内存包装 env -C $D/checker 档包目录裸跑：照拒（「应当是 2，实际 0」）
        lib_heavy_tests.py 的自检（「应当是 0，实际 1」）
      bash-command-detector.sh、detached-run.sh 的 --selftest 不红
  LIB_SHELL_WORDS_BREAK=env-option-clusters-unparsed  skip_prefixes() 里 env 的选项退回只认整词 -u / --unset / -C / --chdir：合写的 -iC、-0iu、缩写的 --chd、单独的 - 与 -S / --split-string
      不认（选项簇当不带值的开关，它的值词当命令词；-S 的字符串整个当命令词，改前的判法）
      lib_heavy_tests.py --selftest：cargo 那张表里「env 选项」开头六格里五格红（「-iC 换进 harness 档目录裸跑放行」不红）、「env 拆串」开头七格里六格红（放行的一格不红）：
        env 选项：-iC 合写、-iC 目录贴着写、--chd 缩写（「应当是 checker-tier-cargo，实际 None 或 full-cargo」）
        env 选项：-0iu 合写、单独一个 -（「应当是 full-cargo，实际 None」）
        env 拆串：-S、-iS、--split-string=、串里带 -C 与赋值、串后面还有参数、经内存包装与 capped.sh（「应当是 checker-tier-cargo，实际 None」）
      heavy-test-guard.sh --selftest（另转出 lib_heavy_tests.py 那十一项）：
        实现员经内存包装 env -iC 合写换进 checker 档包目录 cargo test：照拒、env -S 拆串里是 checker 档包的 cargo test：照拒、env -S 拆串里先换目录再 cargo test：照拒（「应当是 2，实际 0」）
        lib_heavy_tests.py 的自检（「应当是 0，实际 1」）
      bash-command-detector.sh、detached-run.sh 的 --selftest 不红（两份自证里没有 env 带选项簇或 -S 的用例，未实跑，由主 agent 复跑核）
  LIB_SHELL_WORDS_BREAK=env-split-string-unexpanded  skip_prefixes_expanding() 不拆 env 的 -S / --split-string（-S 与它的字符串整个跳过，里面的命令不判）
      lib_heavy_tests.py --selftest：cargo 那张表里「env 拆串」开头七格里六格红（放行的「串里点名 harness 档包放行」不红；「应当是 checker-tier-cargo，实际 None」）
      heavy-test-guard.sh --selftest（另转出 lib_heavy_tests.py 那六项）：
        实现员经内存包装 env -S 拆串里是 checker 档包的 cargo test：照拒、env -S 拆串里先换目录再 cargo test：照拒（「应当是 2，实际 0」）
        lib_heavy_tests.py 的自检（「应当是 0，实际 1」）
      bash-command-detector.sh、detached-run.sh 的 --selftest 不红（同上，未实跑）
  LIB_SHELL_WORDS_BREAK=directory-variables-unexpanded  expand_known_variables() 遇到 `$` 一律交 None：cd / pushd / env -C 的目录与轻型写法的参数写在变量里的照旧认不出（改前的判法）
      lib_heavy_tests.py --selftest：
        变量目录裸跑：D=… 之后 cd $D/checker 档包目录、经内存包装、export D 之后经内存包装 env -C $D/checker 档包目录、cd $HOME/… 指到 checker 档包目录、
        嵌套变量 D=$B/…、外层赋的变量在 bash -c 里的 cd（「应当是 checker-tier-cargo，实际 None」）
        变量目录裸跑：cd ${D} 工作区根、经内存包装全量、declare D=… 之后 cd $D 全量（「应当是 full-cargo，实际 None」）
        轻型：根参数写成前面赋过值的变量、根参数写成 $HOME/…（「应当是 None，实际 verus-stage」「应当是 None，实际 crates-mutation-stage」）
      heavy-test-guard.sh --selftest（另转出 lib_heavy_tests.py 那九项）：
        实现员 D=… 之后 cd $D/checker 档包目录、经内存包装裸跑 cargo test：照拒、实现员 cd ${D} 工作区根、经 timeout、nice、capped.sh、内存包装四层全量：照拒、
        实现员 export D 之后经内存包装 env -C $D/checker 档包目录裸跑：照拒、实现员 cd $HOME/… 指到 checker 档包目录、经内存包装裸跑：照拒、
        主 agent 不带前缀 D=… 之后 cd $D 全量：照拒（「应当是 2，实际 0」）
        调查员 cd 到变量目录、经内存包装起形式化那一道 --quick 根写成同一个变量：放行（「应当是 0，实际 2」）
        cd 进写在变量里的主工作区再起：拒（转出 lib_workspace_cargo_lock.py 那一格，「应当拒，实际 放行」）
        lib_heavy_tests.py 的自检、lib_workspace_cargo_lock.py 的自检（「应当是 0，实际 1」）
      lib_workspace_cargo_lock.py --selftest：
        cd 进写在变量里的主工作区再起：拒（「应当拒，实际 放行」）
      bash-command-detector.sh --selftest：
        通删工作根:D=… 之后 cd $D 进工作根再 rm -rf *、通删工作根:cd $HOME/… 进工作根再 rm -r 第一层部分通配（「应当是 1，实际 0」；后一格只在工作根在家目录下时写成 $HOME）
        覆盖产物:cd $HOME/… 算得出来、按那个目录解析（仓外，放行）（「应当是 0，实际 1」）
      detached-run.sh、host-resources-reminder.sh、timer-only-remind-guard.sh、scripts/agents/agent-watch.py、scripts/gate/light-selftest-guard.py 的 --selftest 不红
  LIB_SHELL_WORDS_BREAK=variable-rebinding-unseen  for / select / read / unset 改写过的变量照旧用之前记下的值（rebind_variables() 与 unset 那一处不记成算不出）
      lib_heavy_tests.py --selftest：
        变量目录裸跑：for 改写过的变量照旧认不出（放行）、read 读进来的变量照旧认不出（放行）、unset 之后照旧认不出（放行）（「应当是 None，实际 checker-tier-cargo」）
      heavy-test-guard.sh --selftest（另转出 lib_heavy_tests.py 那三项）：
        实现员 for 改写过的变量照旧认不出、经内存包装裸跑：放行（「应当是 0，实际 2」）
        lib_heavy_tests.py 的自检（「应当是 0，实际 1」）
      bash-command-detector.sh、lib_workspace_cargo_lock.py、detached-run.sh、host-resources-reminder.sh、timer-only-remind-guard.sh、agent-watch.py、
      light-selftest-guard.py 的 --selftest 不红
  LIB_SHELL_WORDS_BREAK=shell-standard-input-unread  commands_at_command_position() 不把 here-string 与管道喂给 shell 的文本当命令再切、也不交判不了的说明（改前的判法）
      lib_heavy_tests.py --selftest：
        标准输入：here-string 喂给 bash 跑全量、标准输入：printf 经管道喂给 bash 跑全量、标准输入：echo 经管道喂给 sh 跑全量（「应当是 full-cargo，实际 None」）
      heavy-test-guard.sh --selftest（另转出 lib_heavy_tests.py 那三项）：
        实现员 here-string 喂给 bash 跑 --workspace、实现员 printf 经管道喂给 bash 跑 --workspace、实现员 echo 经管道喂给 sh 跑 --workspace、
        主 agent 不带前缀 here-string 喂给 bash 跑 --workspace（「应当是 2，实际 0」）
        实现员 here-string 喂给 bash 的是变量：放行并记检出：记几条检出、实现员 cat 文件经管道喂给 bash：放行并记检出：记几条检出（「应当是 1，实际 0」）
        stdin:here-string 喂给 bash 的是变量放行（退出码 0）、stderr 点名判不了、记一条检出（「应当是 (0, True, 1, 's-fed')」）
        lib_heavy_tests.py 的自检（「应当是 0，实际 1」）
      bash-command-detector.sh --selftest：
        stdin:here-string 喂给 bash 的是变量放行（退出码 0）、记一条检出（「应当是 (0, 1, True)，实际 (0, 0, False)」）
        stdin:拒了的命令里有判不了的输入（cat 文件经管道喂给 bash），照记一条、结局被拒（「应当是 (2, 1, '被拒')，实际 (2, 0, '只记')」）
        stdin:here-string 里前台起看门狗照拒（退出码 2）（「应当是 2，实际 0」）
      run-scope-guard.sh --selftest（这个开关不在它的开关轮里，设着跑时开关轮另报「另有这几格红」）：
        here-string 喂给 bash 的 cargo test 整工作区：没带声明拒、printf 经管道喂给 bash 的门禁：没带声明拒、echo 经管道喂给 sh 的 cargo test 整工作区：没带声明拒（「应当是 2，实际 0」）
        入口：here-string 喂给 bash 的是变量放行、记一条检出（「应当是 (0, 1, True, '只记')」）
      two-logs-guard.sh、detached-run.sh 的 --selftest 不红
  下面七项是同一批回归修补里的边角（三方 shell-words-r1 正推腿报告 research/prompts/shell-words-r1-sonnet-output.md 的 W3、W4、W5 三节）：
  LIB_SHELL_WORDS_BREAK=redirect-fd-dropped  shell_tokens() 不把 fd 号挂到重定向运算符上：redirects_standard_output() 把 `2> 文件` 也当改标准输出（改前的判法）
      heavy-test-guard.sh --selftest：主 agent 不带前缀 cat > 拷出、另把 stderr 丢进 /dev/null 再起全量脚本照拒（「应当是 2，实际 0」）、它的「：记几条检出」（「应当是 0，实际 1」）、
        「：读进去几份脚本」（「应当是 1，实际 0」），cat > 拷出、起的是 2> 写的那一份：放行（「应当是 0，实际 2」）、它的「：记几条检出」（「应当是 1，实际 0」）、「：读进去几份脚本」（「应当是 0，实际 1」）
      bash-command-detector.sh --selftest：stdin:子 agent 经同一条命令里 cat > 拷出、stderr 丢进 /dev/null 再起的 detached-run.sh 拒绝（退出码 2）（「应当是 2，实际 0」）、
        stdin:子 agent cat > 拷出、起的是 2> 写的那一份放行（退出码 0）（「应当是 0，实际 2」）
  LIB_SHELL_WORDS_BREAK=link-source-from-current-directory  link_source_word() 原样交回：ln -s 的相对源照当前目录解（改前的判法）
      heavy-test-guard.sh --selftest：主 agent 不带前缀 ln -s 的相对源从链接所在的目录算、经它起全量脚本照拒（「应当是 2，实际 0」）、它的「：记几条检出」（「应当是 0，实际 1」）、
        「：读进去几份脚本」（「应当是 1，实际 0」），ln -s 的相对源在链接所在的目录里不在：放行（「应当是 0，实际 2」）、它的「：记几条检出」（「应当是 1，实际 0」）、「：读进去几份脚本」（「应当是 0，实际 1」）
      bash-command-detector.sh --selftest：stdin:子 agent 经同一条命令里 cp 出来再 ln -s 相对源、经链接起的 detached-run.sh 拒绝（退出码 2）（「应当是 2，实际 0」）
  LIB_SHELL_WORDS_BREAK=env-leading-equals-as-command  skip_prefixes() 里 env 后面开头就是 `=` 的词当命令词（改前的判法）
      heavy-test-guard.sh --selftest：主 agent 不带前缀 env =1 开头就是 = 的赋值起全量照拒（「应当是 2，实际 0」）
      bash-command-detector.sh --selftest：stdin:子 agent 经env =1 开头就是 = 的赋值的 detached-run.sh 拒绝（退出码 2）（「应当是 2，实际 0」）
  LIB_SHELL_WORDS_BREAK=shell-c-options-unparsed  shell_invocation() 取紧跟 -c 的那个词当代码（`bash -c -- '…'` 的代码取成 --，改前的判法）
      heavy-test-guard.sh --selftest：主 agent 不带前缀 bash -c -- 的代码里起全量照拒（「应当是 2，实际 0」）
      bash-command-detector.sh --selftest：stdin:子 agent 经bash -c -- 的代码里起的 detached-run.sh 拒绝（退出码 2）（「应当是 2，实际 0」）
  LIB_SHELL_WORDS_BREAK=builtin-non-builtin-run  skip_prefixes() 把 builtin 后面不是内建的词照旧当命令词（改前的判法）
      heavy-test-guard.sh --selftest：builtin 后面不是内建：bash 报错不跑，放行（「应当是 0，实际 2」）
  LIB_SHELL_WORDS_BREAK=uncalled-functions-run  drop_uncalled_function_bodies() 原样交回：只定义、调不到的函数体照命令判（改前的判法）
      heavy-test-guard.sh --selftest：function 只定义不调用：函数体不跑，放行（「应当是 0，实际 2」）
      bash-command-detector.sh --selftest：stdin:子 agent function 只定义不调用放行（退出码 0）（「应当是 0，实际 2」）
  LIB_SHELL_WORDS_BREAK=shell-s-positionals-as-script  shell_invocation() 把 -s 后面的词照旧当脚本（改前的判法）
      heavy-test-guard.sh --selftest：bash -s -- 后面的词是位置参数、不是脚本：放行（「应当是 0，实际 2」）、它的「：读进去几份脚本」（「应当是 0，实际 1」）
      bash-command-detector.sh --selftest：stdin:子 agent bash -s -- 后面的词是位置参数放行（退出码 0）（「应当是 0，实际 2」）
  这一批修补的十项（W1、W2 那三项与这七项）下，两个 hook 之外的 lib_heavy_tests.py、lib_workspace_cargo_lock.py、run-scope-guard.sh、two-logs-guard.sh、handback-guard.sh、timer-only-remind-guard.sh、
  host-resources-reminder.sh、shared-index-stale-reminder.sh、scripts/hosts/detached-run.sh、scripts/agents/agent-watch.py、scripts/gate/light-selftest-guard.py、scripts/hosts/memory_peaks_lib.py
  的 --selftest 都不红（逐项实跑过；brace-sequence-unpadded 下 lib_heavy_tests.py 红的那五格除外）；这一批新加的格在已有的 braces-unexpanded、env-loose-assignments-as-command、
  shell-option-values-as-script、function-keyword-as-command 下也红，逐格写在 heavy-test-guard.sh、bash-command-detector.sh 文件头「同一批回归修补……在已有开关下也红」那一段
  下面九项是 records/2026-10-26-subagent欠账清理.md 第 3 批加的（第四十三节第 1、2 行、第四十节第 18 行）；lib_heavy_tests.py、lib_workspace_cargo_lock.py、run-scope-guard.sh、
  two-logs-guard.sh、handback-guard.sh、timer-only-remind-guard.sh、host-resources-reminder.sh、shared-index-stale-reminder.sh 的 --selftest 在这九项下都不红（逐项实跑过）；
  scripts/hosts/detached-run.sh、scripts/agents/agent-watch.py、scripts/gate/light-selftest-guard.py、scripts/hosts/memory_peaks_lib.py 的 --selftest 没在这九项下跑过：
  LIB_SHELL_WORDS_BREAK=time-options-as-command  skip_prefixes() 不跳 time 关键字后面的 -p / --（-p 当命令词，改前的判法）
      heavy-test-guard.sh --selftest：主 agent 不带前缀 time -p 起全量照拒、主 agent 不带前缀 time -- 起全量照拒、实现员 time -p 包着内存包装起 checker 档照拒（「应当是 2，实际 0」）
      bash-command-detector.sh --selftest：等待循环:前台 time -p 包着的 bash -c 里没有 timeout（「应当是 1，实际 0」）、stdin:子 agent 经time -p 包着的 detached-run.sh 拒绝（退出码 2）（「应当是 2，实际 0」）
  LIB_SHELL_WORDS_BREAK=builtin-as-command  skip_prefixes() 把 builtin 当命令词（后面的 source / cd 不认，改前的判法）
      heavy-test-guard.sh --selftest：主 agent 不带前缀 builtin source 读进全量脚本照拒、实现员先进 harness 档包目录、再 builtin cd 进 checker 档包目录经内存包装裸跑照拒（「应当是 2，实际 0」），
        主 agent 不带前缀 builtin source 读进全量脚本照拒：读进去几份脚本（「应当是 1，实际 0」）
      bash-command-detector.sh --selftest：stdin:子 agent 经builtin source 读进来的 detached-run.sh 拒绝（退出码 2）（「应当是 2，实际 0」）
  LIB_SHELL_WORDS_BREAK=shell-option-values-as-script  shell_invocation() 不认 --rcfile / --init-file 的值，`--` 与单独的 `-` 当脚本（改前的判法）
      heavy-test-guard.sh --selftest：主 agent 不带前缀 bash --rcfile 起全量脚本照拒、bash -- 起全量脚本照拒、bash - 起全量脚本照拒（「应当是 2，实际 0」），三格各自的「：读进去几份脚本」（「应当是 1，实际 0」）
      bash-command-detector.sh --selftest：stdin:子 agent 经bash --rcfile 文件之后起的 detached-run.sh 拒绝（退出码 2）、stdin:子 agent 经bash -- 之后起的 detached-run.sh 拒绝（退出码 2）（「应当是 2，实际 0」）
  LIB_SHELL_WORDS_BREAK=function-keyword-as-command  skip_prefixes() 把 function 关键字当命令词（改前的判法）
      heavy-test-guard.sh --selftest：主 agent 不带前缀 function 关键字定义的函数里起全量照拒（「应当是 2，实际 0」）
      bash-command-detector.sh --selftest：stdin:子 agent 经function 关键字定义的函数里起的 detached-run.sh 拒绝（退出码 2）（「应当是 2，实际 0」）
  LIB_SHELL_WORDS_BREAK=braces-unexpanded  brace_expand() 原样交回（引号外的花括号不展开，改前的判法）
      heavy-test-guard.sh --selftest：主 agent 不带前缀 cp X{,.bak} 拷出再起全量脚本照拒（「应当是 2，实际 0」）、它的「：记几条检出」（「应当是 0，实际 1」）、「：读进去几份脚本」（「应当是 1，实际 0」）
      bash-command-detector.sh --selftest：stdin:子 agent 经cp X{,.bak} 拷出再起的 detached-run.sh 拒绝（退出码 2）（「应当是 2，实际 0」）
  LIB_SHELL_WORDS_BREAK=double-slash-kept  resolve_path() 留着开头的 //（os.path.normpath 的样子，改前的判法）
      bash-command-detector.sh --selftest：stdin:子 agent 经cp 之后按 // 开头的路径起的 detached-run.sh 拒绝（退出码 2）（「应当是 2，实际 0」）
      heavy-test-guard.sh --selftest 不红（它按实际路径比，开头的 // 本来就并掉了）
  LIB_SHELL_WORDS_BREAK=env-loose-assignments-as-command  skip_prefixes() 里 env 后面名字不合变量名的 `名字=值` 当命令词（改前的判法）
      heavy-test-guard.sh --selftest：主 agent 不带前缀 env -- 带不合变量名的赋值起全量照拒（「应当是 2，实际 0」）
      bash-command-detector.sh --selftest：stdin:子 agent 经env -- 带不合变量名的赋值的 detached-run.sh 拒绝（退出码 2）（「应当是 2，实际 0」）
  LIB_SHELL_WORDS_BREAK=eval-unread  commands_at_command_position() 不切 eval 的参数（eval 当一条普通命令，改前的判法）
      heavy-test-guard.sh --selftest：主 agent 不带前缀 eval 字面代码里起全量照拒（「应当是 2，实际 0」）
      bash-command-detector.sh --selftest：stdin:子 agent 经eval 字面代码里起的 detached-run.sh 拒绝（退出码 2）（「应当是 2，实际 0」）
  LIB_SHELL_WORDS_BREAK=heredoc-open-substitution-unseen  heredoc_feeds_shell() 照整行找挂它的命令（没收尾的 $( / <( 里的 bash 认不出，正文当数据剥掉，改前的判法）
      heavy-test-guard.sh --selftest：主 agent 不带前缀 $(bash <<EOF) 里起全量照拒、主 agent 不带前缀 <(bash <<EOF) 里起全量照拒（「应当是 2，实际 0」）
      bash-command-detector.sh --selftest：等待循环:前台 $(bash <<EOF) 里的循环、等待循环:前台 <(bash <<EOF) 里的循环（「应当是 1，实际 0」）
  下面三项是同一批的回归修补（三方 shell-words-r1 正推腿报告 research/prompts/shell-words-r1-sonnet-output.md 的 W1、W2 两节）：
  LIB_SHELL_WORDS_BREAK=brace-empties-kept  brace_expand() 留着花括号展开出来的空串（`{cargo,} test` 切成 `cargo '' test`，改前的判法）
      heavy-test-guard.sh --selftest：主 agent 不带前缀 {cargo,} 空的一份起全量照拒、主 agent 不带前缀 {,} 写在命令词前面起全量照拒、
        主 agent 不带前缀 cargo {,test} 空的一份起全量照拒、实现员 {bash,} 空的一份经内存包装起 checker 档照拒（「应当是 2，实际 0」）
      bash-command-detector.sh --selftest：stdin:子 agent 经{bash,} 空的一份的 detached-run.sh 拒绝（退出码 2）、stdin:子 agent 经{,} 写在命令词前面的 detached-run.sh 拒绝（退出码 2）
        （「应当是 2，实际 0」）
  LIB_SHELL_WORDS_BREAK=brace-sequence-unpadded  brace_sequence_width() 一律交 0：补零的整数序列（{01..03}）不补零（这里改前的判法；lib_heavy_tests.py 的 brace_sequence 共用它，一并不补）
      heavy-test-guard.sh --selftest（另转出 lib_heavy_tests.py 那五项）：主 agent 不带前缀 cp 补零的序列拷出再起全量脚本照拒（「应当是 2，实际 0」）、
        它的「：记几条检出」（「应当是 0，实际 1」）、它的「：读进去几份脚本」（「应当是 1，实际 0」）、lib_heavy_tests.py 的自检（「应当是 0，实际 1」）
      lib_heavy_tests.py --selftest：花括号展开与 bash 一致：{01..10}、{-01..3}、{1..-03}、{0..00}、{1..05..2}（「应当是 [」；三方 shell-words-r2 判决之后加的 {01..100..33}、{100..00..50} 同样红）
      bash-command-detector.sh --selftest：stdin:子 agent 经cp 补零的序列拷出再起的 detached-run.sh 拒绝（退出码 2）（「应当是 2，实际 0」）
  LIB_SHELL_WORDS_BREAK=double-quoted-substitution-unclosed  open_substitution_tail() 把双引号里的 `$(` 一律当没收尾（改前另写一套引号状态时的样子：顶层的 bash <<EOF 当挂在替换里，正文当数据剥掉）
      heavy-test-guard.sh --selftest：主 agent 不带前缀 cd "$(git rev-parse --show-toplevel)" 之后 bash <<EOF 里起全量照拒、主 agent 不带前缀 FOO="$(pwd)" bash <<EOF 里起全量照拒
        （「应当是 2，实际 0」）
      bash-command-detector.sh --selftest：等待循环:前台 cd "$(…)" 之后 bash <<EOF 里的循环、等待循环:前台 echo "$(date)" | bash <<EOF 里的循环（「应当是 1，实际 0」）
  下面十三项是三方 shell-words-r2 判决（research/prompts/shell-words-r2-main-verification.md 第二节）第 2–10 条的修补：
  LIB_SHELL_WORDS_BREAK=function-calls-literal-only  function_never_called() 只数原文里的字面：拆开拼写的调用（`"fo"o`、`fo{o,}`、`fo\\o`）认不出，函数体照不判（改前的判法）
      heavy-test-guard.sh --selftest：主 agent 不带前缀 function 定义之后引号拆开拼写调用照拒、主 agent 不带前缀 function 定义之后花括号拆开拼写调用照拒、
        主 agent 不带前缀 function 定义之后反斜杠拆开拼写调用照拒、编译目录：function 定义之后拆开拼写调用：拒（退出码 2）（「应当是 2，实际 0」），
        lib_workspace_cargo_lock.py 的自检（「应当是 0，实际 1」；另原样转出 lib_workspace_cargo_lock.py 下面那一格）
      lib_workspace_cargo_lock.py --selftest：编译目录：function 定义之后拆开拼写调用：拒（「命令」）
      bash-command-detector.sh --selftest：stdin:子 agent 经function 定义之后引号拆开拼写调用的 detached-run.sh 拒绝（退出码 2）（「应当是 2，实际 0」）
  LIB_SHELL_WORDS_BREAK=redirect-fd-number-literal  redirection_fd() 照字面比 fd 号：`01>` 不算 1 号（改前的判法）
      heavy-test-guard.sh --selftest：主 agent 不带前缀 cat 01> 拷出再起全量脚本照拒（「应当是 2，实际 0」）、它的「：记几条检出」（「应当是 0，实际 1」）、「：读进去几份脚本」（「应当是 1，实际 0」）
      bash-command-detector.sh --selftest：stdin:子 agent 经同一条命令里 cat 01> 拷出再起的 detached-run.sh 拒绝（退出码 2）（「应当是 2，实际 0」）
  LIB_SHELL_WORDS_BREAK=redirect-fd-quoted-digits  shell_tokens() 把引号或反斜杠写出的数字（`"2">`、`\\2>`）也当 fd 号（改前的判法）
      heavy-test-guard.sh --selftest：主 agent 不带前缀 cat 源 "2"> 拷出（引号里的 2 是源）再起全量脚本照拒（「应当是 2，实际 0」）、它的「：记几条检出」「：读进去几份脚本」
      bash-command-detector.sh --selftest：stdin:子 agent 经同一条命令里 cat 源 "2"> 拷出（引号里的 2 是源）再起的 detached-run.sh 拒绝（退出码 2）（「应当是 2，实际 0」）
  LIB_SHELL_WORDS_BREAK=redirect-fd-duplication-unseen  standard_output_destination() 不跟 `n>&m`，只取直接改标准输出的最后一个（改前的判法）
      heavy-test-guard.sh --selftest：主 agent 不带前缀 cat 3> 再 >&3 拷出再起全量脚本照拒、主 agent 不带前缀 heredoc 写文件：cat <<EOF 3> 再 >&3 写出再起全量脚本照拒（「应当是 2，实际 0」），
        两格各自的「：记几条检出」「：读进去几份脚本」
      bash-command-detector.sh --selftest：stdin:子 agent 经同一条命令里 cat 3> 再 >&3 拷出再起的 detached-run.sh 拒绝（退出码 2）（「应当是 2，实际 0」）
  LIB_SHELL_WORDS_BREAK=shell-long-options-after-short  shell_invocation() 把短选项之后的长选项照旧当选项跳过（改前的判法）
      heavy-test-guard.sh --selftest：bash -c 之后的长选项：bash 报错不跑，放行（「应当是 0，实际 2」）
      bash-command-detector.sh --selftest：stdin:子 agent bash -c 之后的长选项：bash 报错不跑放行（退出码 0）（「应当是 0，实际 2」）
  LIB_SHELL_WORDS_BREAK=shell-syntax-only-code-run  shell_invocation() 带 -n 时 -c 的代码照判（改前的判法）
      heavy-test-guard.sh --selftest：bash -c -n 只查语法：放行（「应当是 0，实际 2」）
      bash-command-detector.sh --selftest：stdin:子 agent bash -c -n 只查语法放行（退出码 0）（「应当是 0，实际 2」）
  LIB_SHELL_WORDS_BREAK=env-options-after-assignment  skip_prefixes() 里 env 认到赋值之后照旧认选项（改前的判法）
      heavy-test-guard.sh --selftest：env =1 之后的 -u 是命令词：env 报错不跑，放行（「应当是 0，实际 2」）、lib_heavy_tests.py 的自检（「应当是 0，实际 1」；另转出下面那一格）
      lib_heavy_tests.py --selftest：env 拆串：拆出来的串里赋值之后的 -C 是命令词，env 报错不跑（「应当是 None，实际 checker-tier-cargo」）
      bash-command-detector.sh --selftest：stdin:子 agent env =1 之后的 -u 是命令词：env 报错不跑放行（退出码 0）（「应当是 0，实际 2」）
  LIB_SHELL_WORDS_BREAK=brace-sequence-width-padded-ends-only  brace_sequence_width() 只取补零写的那几边的长度（改前的判法）
      heavy-test-guard.sh --selftest：主 agent 不带前缀 cp 带步长、两端宽度不一的补零序列拷出再起全量脚本照拒（「应当是 2，实际 0」）、它的「：记几条检出」「：读进去几份脚本」，
        lib_heavy_tests.py 的自检（「应当是 0，实际 1」；另转出下面两格）
      lib_heavy_tests.py --selftest：花括号展开与 bash 一致：{01..100..33}、{100..00..50}（「应当是 [」）
  LIB_SHELL_WORDS_BREAK=ansi-c-quotes-unseen  shell_tokens()、open_substitution_tail()、command_substitution_end() 把 `$'…'` 照普通单引号配对（改前的判法）
      heavy-test-guard.sh --selftest：主 agent 不带前缀 $'…' 里带 \\' 之后 bash <<EOF 里起全量照拒（「应当是 2，实际 0」）
      bash-command-detector.sh --selftest：等待循环:前台 $'a\\'b' 之后 $(bash <<EOF) 里的循环（「应当是 1，实际 0」）
  LIB_SHELL_WORDS_BREAK=case-patterns-as-commands  case_patterns_and_conditionals_folded() 不改 case 的模式（改前的判法）
      heavy-test-guard.sh --selftest：case 里 ;; 之后的模式词不当命令：放行、case 里 | 隔开的模式词不当命令：放行（「应当是 0，实际 2」）
      bash-command-detector.sh --selftest：stdin:子 agent case 里 ;; 之后的 reboot 模式不当命令放行（退出码 0）、stdin:子 agent case 里 ;; 之后的 claude 模式不当命令放行（退出码 0）（「应当是 0，实际 2」）
  LIB_SHELL_WORDS_BREAK=conditional-expression-split  case_patterns_and_conditionals_folded() 不改 `[[ … ]]`（改前的判法）
      heavy-test-guard.sh --selftest：[[ ]] 里 && 之后的词不当命令：放行（「应当是 0，实际 2」）
  LIB_SHELL_WORDS_BREAK=redirect-target-braces-expanded  shell_tokens() 照旧展开重定向目标上的花括号（改前的判法：展开出来多的词成了这一条的参数；随第 3 条一起改：
    几个源的 cat 认成拷贝之后，`cat s.sh > d{,}.sh` 被当成把 s.sh 接进 d.sh，而 bash 报 ambiguous redirect、什么都不写；展开成几个词的目标现在照原样留一个词：拷出再执行认不出这个落点，写范围那一类照落点算不出拒）
      heavy-test-guard.sh --selftest：重定向目标花括号展开成几个词：bash 报 ambiguous redirect 不写，放行（「应当是 0，实际 2」）、它的「：记几条检出」（「应当是 1，实际 0」）、
        「：读进去几份脚本」（「应当是 0，实际 1」）
  LIB_SHELL_WORDS_BREAK=extended-assignments-as-command  skip_prefixes() 把 `名字+=值`、`名字[下标]=值` 照旧当命令词（改前的判法；随第 6 条一起改：ANSI-C 引号解开之后，
    定义里的 \\t 把测试目标名隔成单独的词，这类赋值词被当成没展开的命令词误拒）
      heavy-test-guard.sh --selftest：数组元素赋值与 += 追加是赋值、不是命令词：放行（「应当是 0，实际 2」）
  这十三项下 run-scope-guard.sh、two-logs-guard.sh、handback-guard.sh、timer-only-remind-guard.sh、host-resources-reminder.sh、shared-index-stale-reminder.sh、
  scripts/hosts/detached-run.sh、scripts/agents/agent-watch.py、scripts/gate/light-selftest-guard.py、scripts/hosts/memory_peaks_lib.py 的 --selftest 都不红，上面没点名的
  lib_heavy_tests.py、lib_workspace_cargo_lock.py 也不红（逐项实跑过；detached-run.sh 的「核数」几格在几项并行跑、机器忙时偶尔红，单跑不红，与开关无关）；
  这一批新加的格在已有的 arguments-as-executed、braces-unexpanded、brace-sequence-unpadded、env-leading-equals-as-command、env-loose-assignments-as-command、
  function-keyword-as-command、uncalled-functions-run、heredoc-open-substitution-unseen、redirect-fd-dropped 下也红，逐格写在 heavy-test-guard.sh、bash-command-detector.sh
  文件头「三方 shell-words-r2 判决……在已有开关下也红」那一段
"""
import os
import re
from typing import NamedTuple

SHELL_NAMES = {"bash", "sh", "dash", "zsh", "ksh"}
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
# 其中给命令定内存上限的那一个：剥掉它的命令记 memory_capped（heavy-test-guard.sh 判子 agent 跑编译出来的代码经没经它）
MEMORY_CAP_WRAPPER = "run-with-memory-cap.sh"
# scripts/hosts/detached-run.sh（脱离会话长跑的入口）：两个 hook 都要认它，按名字（路径不论）、解到的真实路径（符号链接）与文件开头的签名行
# （行首 `# detached-run-script-signature:`，改名或拷走的一份）认；读文件开头这么多字节找签名行。这份文件自己的开头里没有以它起头的行
DETACHED_RUN_SCRIPT_NAME = "detached-run.sh"
DETACHED_RUN_SCRIPT_LOCATION = "scripts/hosts/detached-run.sh"
DETACHED_RUN_SIGNATURE_LINE = re.compile(rb"^# detached-run-script-signature:", re.MULTILINE)
DETACHED_RUN_HEAD_BYTES = 4096
# 它的 start 带值的选项（`--wall-clock <within-2h|over-2h>`、`--label <名>`、`--log <文件>`、`--stop-timeout <秒>`，也可以写成 `--选项=值`）
DETACHED_RUN_OPTIONS_WITH_VALUE = frozenset({"--wall-clock", "--label", "--log", "--stop-timeout"})
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
    弄坏开关 LIB_SHELL_WORDS_BREAK=redirect-fd-number-literal 下照字面比：写法不是最简十进制的（`01`）当作哪个号都对不上（改前的判法）。"""
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
    弄坏开关 LIB_SHELL_WORDS_BREAK=redirect-fd-duplication-unseen 下不跟 `n>&m`：只取直接改标准输出（redirects_standard_output）的最后一个（改前的判法）。"""
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
                                  # "python"（python3 后面跟的程序）；直接执行的命令（含 capped.sh N 后面那条）是 None
    output_redirections: tuple = ()      # 这一条简单命令上的输出重定向：((运算符, 目标词), …)，运算符是 OUTPUT_REDIRECTS 之一；fd 号不在里面
    standard_input_heredocs: tuple = ()  # 喂给这一条的、被当数据剥掉的 heredoc：(HeredocBody, …)，按出现次序（最后一个才是它读到的标准输入）
    memory_capped: bool = False          # 经 run-with-memory-cap.sh 跑：它自己的包装里有，或外层（bash -c 的那段代码、起它所在脚本的那一条）有
    variables: dict | None = None        # 跑这条命令时这段命令文本里记下的变量（名字 → 已展开的值，None 是算不出）：调用方拿它给 expand_known_variables 展开参数


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
        if name not in SHELL_NAMES:
            return False
        return shell_reads_standard_input(arguments)
    return False


def open_substitution_tail(text):
    """text 末尾还开着的最里层命令替换或进程替换（`$(`、`<(`、`>(`、反引号；收尾了的、引号里的括号不算）之后的文字；都收尾了交回 text 原样。
    引号、反斜杠、替换的收尾与 shell_tokens 用的是同一套扫描（command_substitution_end、double_quote_end、backquote_end，没配上时由它们记下最里层还开着的正文从哪起）：
    双引号里收了尾的 `$(…)` 不算开着（`cd "$(git rev-parse --show-toplevel)" && bash <<EOF` 挂的是顶层的 bash）；`<(`、`>(` 只在引号外认，词首的 `#` 起到行尾是注释。
    弄坏开关 LIB_SHELL_WORDS_BREAK=double-quoted-substitution-unclosed 下双引号里的 `$(` 一律当没收尾（改前另写的那套状态的样子）。"""
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
    """弄坏开关 LIB_SHELL_WORDS_BREAK=ansi-c-quotes-unseen：`$'…'` 照普通单引号配对（里面的 `\\'` 不认，改前的判法；只给自检证红用）。"""
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


def command_substitution_end(text, start, unclosed=None):
    """`$(` 之后（start 指向它后面第一个字符）找配对的 `)`，交回它后面那个下标；引号、反斜杠、反引号与嵌套的括号都跳过；没配上交 -1。unclosed 同 backquote_end。"""
    depth, position = 1, start
    while position < len(text):
        character = text[position]
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
                # 弄坏开关 redirect-target-braces-expanded 下照旧展开（改前的判法：多出来的词成了这一条的参数）
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
       命令位置上的 `esac` 收尾，case 可以套 case。弄坏开关 LIB_SHELL_WORDS_BREAK=case-patterns-as-commands 下不改（改前的判法：`;;` 之后的模式词当命令词）。
    ② 条件表达式（命令位置上的 `[[ … ]]`）里的 `&&`、`||`、`(`、`)`、`<`、`>` 是表达式的一部分，不是命令分隔与重定向：换成普通的词（`2>` 那种 fd 号还原成一个词），
       里面的换行去掉，整段是 `[[` 这一条命令的参数。没有收尾的 `]]` 不改。弄坏开关 LIB_SHELL_WORDS_BREAK=conditional-expression-split 下不改（改前的判法）。
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


def brace_expand(word, mask):
    """一个词的花括号展开：mask 与 word 等长，'u' 是引号外的字符、'q' 是引号里、转义过或替换正文里的（不参与展开），'e' 是引号写出的空串占的位置。
    交回展开出的词表（没有可展开的交 [word]），'e' 那几个位置去掉。展开出来的空串照 bash 去掉（`{cargo,} test` 是 `cargo test`、`{,}` 整个没了），
    这一份里有引号写出的空串的留着（`''{,}` 是两个空词、`{"",}` 是一个）；弄坏开关 LIB_SHELL_WORDS_BREAK=brace-empties-kept 下空串都留着（改前的判法）。"""
    def without_quoted_empties(text, text_mask):
        return "".join(character for character, kind in zip(text, text_mask) if kind != "e")
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
    这里的 brace_sequence 与 lib_heavy_tests.py 的 brace_sequence 共用这一份。弄坏开关 LIB_SHELL_WORDS_BREAK=brace-sequence-unpadded 下一律交 0（这里改前的判法）；
    brace-sequence-width-padded-ends-only 下只取补零写的那几边的长度（三方 shell-words-r2 判决之前的判法）。"""
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


def simple_commands(tokens, separators=None):
    """按命令边界切成一条条简单命令（只留词，重定向连同目标去掉），另交整段里输出重定向的目标，
    每条简单命令里的命令替换正文，以及每条简单命令上的重定向 ((运算符, 目标词), …)（后两样与命令一一对应；只有重定向、没有词的那一条，词是空列表）。
    separators 是一个列表时，按命令的次序追加每一条前面的那个分隔符（第一条是 None；`|` 之后换行的，下一条照旧记 `|`），调用方拿它认管道后一段。"""
    commands, substitutions, current, current_substitutions, output_targets = [], [], [], [], []
    redirections, current_redirections = [], []
    preceding = None
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
                preceding = text
            elif not (text == "\n" and preceding in ("|", "|&")):
                preceding = text
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
    return commands, output_targets, substitutions, redirections


def words_break_is_set(item):
    """弄坏开关 LIB_SHELL_WORDS_BREAK 里有没有这一项（几个用逗号连；只给自检证红用）。"""
    return item in os.environ.get("LIB_SHELL_WORDS_BREAK", "").split(",")


def env_chdir_ignored():
    """弄坏开关 LIB_SHELL_WORDS_BREAK=env-chdir-ignored：env 的 -C / --chdir 换的目录不跟（改前的判法：按外面的当前目录判，只给自检证红用）。"""
    return "env-chdir-ignored" in os.environ.get("LIB_SHELL_WORDS_BREAK", "").split(",")


def env_directory(directory, target, variables=None):
    """env -C 的目标解开成绝对路径：相对路径按命令开始时的当前目录。目标里的变量照 expand_known_variables 展开（variables 是这段命令文本里前面赋过的）；
    展开不了（没赋过的变量、命令替换）或当前目录认不出时交 directory 原样（照旧按外面的当前目录判，
    不像 cd 那样交 None：交 None 会让这一条命令的目录类判定全部放过，比改前还松；罩不住的：env -C 的目录写在展开不了的变量里）。"""
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
    prefix_names, assignments, chdir = set(), {}, None
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
                return PrefixSkip(None, prefix_names, assignments, chdir)   # `builtin cargo …`：bash 报 not a shell builtin，什么都不跑
            continue
        if word == "function" and not words_break_is_set("function-keyword-as-command"):
            position += 2   # `function 名字 { …; }`、`function 名字() …`：关键字与函数名都不是命令词，函数体照命令判（同 `名字() { …; }`）
            continue
        if assignment:
            assignments[assignment.group(1)] = assignment.group(2)
            position += 1
            continue
        if (APPEND_ASSIGNMENT.match(word) or SUBSCRIPTED_ASSIGNMENT.match(word)) and not words_break_is_set("extended-assignments-as-command"):
            position += 1   # `名字+=值`、`名字+=(…)`、`名字[下标]=值`：bash 照赋值认，不是命令词（记下的值由 record_array_writes 管；弄坏开关下照旧当命令词，改前的判法）
            continue
        if name not in PREFIX_OPTIONS_WITH_VALUE:
            return PrefixSkip(position, prefix_names, assignments, chdir)
        if name == "command" and words[position + 1:position + 2] in (["-v"], ["-V"]):
            return PrefixSkip(position, prefix_names, assignments, chdir)  # command -v / -V 是查名字，不执行后面那个词
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
                    if option.split_string is not None and not env_split_string_unexpanded():
                        if expand_split:
                            return PrefixSkip(None, prefix_names, assignments, chdir, (position, position + option.width, option.split_string))
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
    return PrefixSkip(None, prefix_names, assignments, chdir)


def arguments_as_executed():
    """弄坏开关 LIB_SHELL_WORDS_BREAK=arguments-as-executed：`bash -n 脚本` 与 `python3 - 词…` 里的参数照旧当被执行的脚本（只给自检证红用）。"""
    return "arguments-as-executed" in os.environ.get("LIB_SHELL_WORDS_BREAK", "").split(",")


def shell_invocation(arguments):
    """bash / sh 这类后面的参数：是 -c 的一段代码、是起一个脚本，还是都不是（只查语法、从标准输入读）。
    -c 之后照 bash 接着认选项，选项完了（`--`、单独的 `-` 之后，或头一个不是选项的词）的那个词是代码（`bash -c -- '…'`、`bash -c -x '…'`；
    弄坏开关 LIB_SHELL_WORDS_BREAK=shell-c-options-unparsed 下取紧跟 -c 的那个词，改前的判法）；带 -s 的从标准输入读，后面的词是位置参数、不是脚本
    （`bash -s -- 词`；弄坏开关 LIB_SHELL_WORDS_BREAK=shell-s-positionals-as-script 下照旧当脚本，改前的判法）。
    bash 的长选项只认写在短选项前面的：短选项（`-c`、`-x`、`-o 值` 这类）之后再写 `--名字`（单独的 `--` 除外），bash 报 `--: invalid option`、什么都不跑
    （`bash -c --posix '…'`、`bash -x --posix 脚本`；弄坏开关 shell-long-options-after-short 下照旧当选项跳过，改前的判法）；
    带 -n（`-n`、`-cn`、`-xn`，`+n` 撤回）只查语法，-c 的代码、脚本、标准输入都不执行（弄坏开关 shell-syntax-only-code-run 下 -c 的代码照判，改前的判法）。
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
    硬链接、带 -r、绝对路径、`~` 起头、带 `$` 的照原样交回。弄坏开关 LIB_SHELL_WORDS_BREAK=link-source-from-current-directory 下一律照原样（改前的判法：源按当前目录解）。"""
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


def carries_detached_run_signature(head):
    """一份文件开头的字节（只看前 DETACHED_RUN_HEAD_BYTES 个）里有没有 detached-run.sh 的签名行（行首）。"""
    return bool(DETACHED_RUN_SIGNATURE_LINE.search(head[:DETACHED_RUN_HEAD_BYTES]))


def detached_run_script(word, directory, by_name_only=False):
    """命令词是不是 scripts/hosts/detached-run.sh：是就交一句凭什么认出来的，不是交 None。三种办法：名字叫 detached-run.sh（路径不论）；
    解到的文件的真实路径落在 scripts/hosts/detached-run.sh（符号链接）；文件开头有签名行（改名或拷走的一份）。
    后两种读盘：相对路径按 directory 解（directory 是 None 时只认绝对路径），带没展开的变量、命令替换、通配的词不解；文件不在就认不出。
    by_name_only 为真时只按名字认（两个 hook 的弄坏开关用）。"""
    expanded = os.path.expanduser(word)
    if os.path.basename(expanded) == DETACHED_RUN_SCRIPT_NAME:
        return word
    if by_name_only or not expanded or set("$`*?[") & set(expanded):
        return None
    resolved = resolve_path(directory, expanded)
    if resolved is None or not os.path.isfile(resolved):
        return None
    real = os.path.realpath(resolved)
    if real.endswith("/" + DETACHED_RUN_SCRIPT_LOCATION):
        return f"{word}（符号链接，指到 {DETACHED_RUN_SCRIPT_LOCATION}）"
    try:
        with open(real, "rb") as handle:
            head = handle.read(DETACHED_RUN_HEAD_BYTES)
    except OSError:
        return None
    if carries_detached_run_signature(head):
        return f"{word}（开头带 {DETACHED_RUN_SCRIPT_LOCATION} 的签名行：改名或拷走的一份）"
    return None


def detached_run_arguments_after_preflight(arguments):
    """detached-run.sh 脚本名之后的参数经它开头的 preflight 之后是什么：每一个整词 `--force` 都摘掉（.claude/singlefs-ai-sop/scripts/preflight.sh
    逐词比，连 `--` 之后的、连写在 `--label` 这类选项值位置上的）。弄坏开关 LIB_SHELL_WORDS_BREAK=detached-run-force-kept 下原样交回。"""
    if "detached-run-force-kept" in os.environ.get("LIB_SHELL_WORDS_BREAK", "").split(","):
        return list(arguments)
    return [argument for argument in arguments if argument != "--force"]


def detached_run_start_command(arguments):
    """detached-run.sh 脚本名之后的参数是 `[--force] start <选项> -- <命令…>` 时交回 <命令…> 的词（它把这条命令挂到用户级 systemd 下跑）；
    先照 preflight 摘掉每一个 --force（detached_run_arguments_after_preflight），交回的就是单元里真跑的那几个词。
    别的子命令（list、watch、stop、--selftest）、没有 `--`、`--` 之后是空的、带认不出的选项（它按用法错拒、什么都不跑）交 None。"""
    arguments = detached_run_arguments_after_preflight(arguments)
    position = 0
    while position < len(arguments) and arguments[position] == "--force":
        position += 1
    if arguments[position:position + 1] != ["start"]:
        return None
    position += 1
    while position < len(arguments):
        word = arguments[position]
        if word == "--":
            return arguments[position + 1:] or None
        if word in DETACHED_RUN_OPTIONS_WITH_VALUE:
            position += 2
        elif word == "--force" or word.partition("=")[0] in DETACHED_RUN_OPTIONS_WITH_VALUE:
            position += 1
        else:
            return None
    return None


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
    """弄坏开关 LIB_SHELL_WORDS_BREAK=shell-standard-input-unread：here-string 与管道喂给 shell 的文本不当命令再切、也不交判不了的说明（改前的判法，只给自检证红用）。"""
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
    """弄坏开关 LIB_SHELL_WORDS_BREAK=directory-variables-unexpanded：cd / pushd / env -C 的目录带 `$` 一律展开不了（改前的判法，只给自检证红用）。"""
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


def changed_directory(directory, arguments, variables=None):
    """cd / pushd 之后的当前目录。目标里的变量照 expand_known_variables 展开（variables 是这段命令文本里前面赋过的；不给时只认 HOME）；
    目标是 `-`、展开不了的变量或命令替换时认不出，交 None。"""
    targets = [argument for argument in arguments if not argument.startswith("-") or argument == "-"]
    if not targets:
        return os.path.expanduser("~")
    target = os.path.expanduser(targets[0])
    if target == "-":
        return None
    target = expand_known_variables(target, variables or {})
    if target is None:
        return None
    return resolve_path(directory, target)


FUNCTION_CALLING_WORDS = frozenset({"eval", "source", ".", "trap", "compgen", "declare", "typeset", "set", "alias", "enable"})   # 不点名字也调得到函数、或让子进程看得见函数的写法
IMPLICITLY_CALLED_FUNCTIONS = frozenset({"command_not_found_handle"})   # bash 自己会调的函数名：命令找不到时调它


def drop_uncalled_function_bodies(tokens, text, stripped_heredocs):
    """切好的词里只定义、不会被调到的 `function 名字 { …; }`（`function 名字() { …; }` 同）连同函数体去掉：bash 只定义不跑函数体。
    只在这几样都成立时去掉，否则照留（函数体照命令判，偏严）：函数体是一个花括号组、里面没有另一个 `{`（到头一个 `}` 为止）；名字是字面的、
    不是 bash 自己会调的（IMPLICITLY_CALLED_FUNCTIONS）；整段命令文本 text 里这个名字只出现这一次（照词的边界数，heredoc 正文、引号里的都算），
    函数体之外切好的词里也没有一个是它（拆开拼写的 `"fo"o`、`fo{o,}` 解开之后就是它；还留着成对花括号的词认不准，照调到了算：function_never_called）；
    函数体之外没有命令替换、变量、反引号、通配，没有 FUNCTION_CALLING_WORDS 里的词（eval、source、set -a 这类），剥掉的 heredoc 正文里没有 `$` 与反引号。
    弄坏开关 LIB_SHELL_WORDS_BREAK=uncalled-functions-run 下照留（改前的判法）。"""
    if words_break_is_set("uncalled-functions-run") or (False, "function") not in tokens:
        return tokens
    at_command_position, index = True, 0
    while index < len(tokens):
        kind, value = tokens[index]
        if kind is False and at_command_position and value == "function" and index + 1 < len(tokens) and tokens[index + 1][0] is False:
            name, body = tokens[index + 1][1], index + 2
            if tokens[body:body + 2] == [(True, "("), (True, ")")]:
                body += 2
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
    弄坏开关 LIB_SHELL_WORDS_BREAK=function-calls-literal-only 下只数原文里的字面（改前的判法：拆开拼写的调用认不出）。"""
    if not name or set("$`*?[\\{}") & set(name) or name in IMPLICITLY_CALLED_FUNCTIONS:
        return False
    if len(re.findall(r"(?<![A-Za-z0-9_])" + re.escape(name) + r"(?![A-Za-z0-9_])", text)) != 1:
        return False
    literal_only = words_break_is_set("function-calls-literal-only")
    for kind, value in outside:
        if kind is None or (kind is False and (set("$`*?[") & set(value) or os.path.basename(value) in FUNCTION_CALLING_WORDS)):
            return False
        if kind is False and not literal_only and (value == name or ("{" in value and "}" in value)):
            return False
    return not any("$" in heredoc.text or "`" in heredoc.text for heredoc in stripped_heredocs)


def commands_at_command_position(text, directory=None, environment=None, depth=0, stripped_heredocs=None, memory_capped=False, variables=None,
                                 uncalled_functions_skipped=False):
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
    只定义、调不到的 `function 名字 { …; }` 的函数体不判（drop_uncalled_function_bodies；只管这一层，bash -c、eval、替换里的照判）。"""
    if stripped_heredocs is None:
        stripped_heredocs = []
    tokens = shell_tokens(strip_data_heredocs(text, stripped_bodies=stripped_heredocs))
    if uncalled_functions_skipped and depth == 0:
        tokens = drop_uncalled_function_bodies(tokens, text, stripped_heredocs)
    separators = []
    simple, output_targets, substitutions, redirections = simple_commands(tokens, separators)
    commands, prefix_names, unjudged = [], set(), []
    previous_command = None   # 上一条简单命令：(词, 喂给它的 heredoc 正文)，管道后一段的 shell 从它取标准输入
    has_lone_ampersand = (True, "&") in tokens
    exported = dict(environment or {})
    if variables is None:
        variables = {}
        for name, value in exported.items():
            variables[name] = assigned_value(value, variables)
    else:
        variables = dict(variables)
    for index, (words, command_substitutions, command_redirections) in enumerate(zip(simple, substitutions, redirections)):
        fed_heredocs = heredocs_fed(command_redirections, stripped_heredocs)
        piped_from = previous_command if index < len(separators) and separators[index] in PIPE_SEPARATORS else None
        previous_command = (words, fed_heredocs)
        for substitution in command_substitutions:
            if depth < MAXIMUM_NESTING:
                inner = commands_at_command_position(substitution, directory, exported, depth + 1, stripped_heredocs, memory_capped, variables)
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
        attached = {
            "variables": dict(variables),
            "output_redirections": tuple((operator, target) for operator, target in command_redirections if operator in OUTPUT_REDIRECTS),
            "standard_input_heredocs": fed_heredocs,
        }
        command_environment = dict(exported)
        command_directory = directory   # 这一条命令的当前目录：env -C 换的只管这一条，不像 cd 那样往后带
        remaining = words
        launcher = None
        command_memory_capped = memory_capped
        for _ in range(MAXIMUM_NESTING):
            skipped, remaining = skip_prefixes_expanding(remaining)
            prefix_names |= skipped.prefix_names
            command_environment.update(skipped.assignments)
            if skipped.chdir is not None:
                command_directory = env_directory(command_directory, skipped.chdir, variables)
            if skipped.command_position is None:
                break
            remaining = remaining[skipped.command_position:]
            name, arguments = os.path.basename(remaining[0]), remaining[1:]
            if name == "export":
                for argument in arguments:
                    assignment = ASSIGNMENT.match(argument)
                    if assignment:
                        exported[assignment.group(1)] = assignment.group(2)
                        variables[assignment.group(1)] = assigned_value(assignment.group(2), variables)
                break
            if name == "unset":
                for argument in arguments:
                    if not argument.startswith("-"):
                        exported.pop(argument, None)
                        if not variable_rebinding_unseen():
                            variables[argument.split("[", 1)[0]] = None
                break
            if name in ("cd", "pushd"):
                directory = changed_directory(directory, arguments, variables)
                break
            if name == "eval" and arguments and not words_break_is_set("eval-unread"):
                # eval 把参数（引号已去掉）用空格连起来当代码再跑一遍：连起来的文字照一条完整的命令再切（同 bash -c）。
                # 参数里外层先展开的变量与命令替换照原文切（展开成什么判不到，变量里拼出来的命令照旧认不出）
                if depth < MAXIMUM_NESTING:
                    inner = commands_at_command_position(" ".join(arguments), command_directory, command_environment, depth + 1, stripped_heredocs,
                                                         command_memory_capped, variables)
                    commands += inner.commands
                    prefix_names |= inner.prefix_names
                    has_lone_ampersand = has_lone_ampersand or inner.has_lone_ampersand
                    output_targets += inner.output_targets
                    unjudged += inner.unjudged_shell_inputs
                break
            if name in REBINDING_COMMANDS or name in DECLARATION_COMMANDS:
                rebind_variables(name, arguments, variables)
            if name in SHELL_NAMES:
                invocation = shell_invocation(arguments)
                if invocation.code_string is not None:
                    if depth < MAXIMUM_NESTING:
                        inner = commands_at_command_position(invocation.code_string, command_directory, command_environment, depth + 1, stripped_heredocs,
                                                             command_memory_capped, variables)
                        commands += inner.commands
                        prefix_names |= inner.prefix_names
                        has_lone_ampersand = has_lone_ampersand or inner.has_lone_ampersand
                        output_targets += inner.output_targets
                        unjudged += inner.unjudged_shell_inputs
                    break
                if shell_reads_standard_input(arguments) and not shell_standard_input_unread():
                    # here-string 与管道喂进来的字面文本当这个 shell 的代码再切一遍（同 bash -c：包装、目录、变量、内存包装照这一条算）；判不了的记一句
                    fed = shell_standard_input(command_redirections, piped_from)
                    if fed.unjudged:
                        unjudged.append(f"喂给 {name} 的标准输入判不了：{fed.unjudged}")
                    if fed.code is not None and depth < MAXIMUM_NESTING:
                        inner = commands_at_command_position(fed.code, command_directory, command_environment, depth + 1, stripped_heredocs,
                                                             command_memory_capped, variables)
                        commands += inner.commands
                        prefix_names |= inner.prefix_names
                        has_lone_ampersand = has_lone_ampersand or inner.has_lone_ampersand
                        output_targets += inner.output_targets
                        unjudged += inner.unjudged_shell_inputs
                if invocation.script_index is None:
                    commands.append(CommandAtPosition(remaining, command_directory, command_environment, **attached, memory_capped=command_memory_capped))
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
                    commands.append(CommandAtPosition(remaining, command_directory, command_environment, **attached, memory_capped=command_memory_capped))
                    break
                remaining = arguments[invocation.script_index:]
                launcher = "python"
                continue
            if name in ("source", "."):
                if not arguments or not SOURCE_TARGET.match(arguments[0]):
                    commands.append(CommandAtPosition(remaining, command_directory, command_environment, **attached, memory_capped=command_memory_capped))
                    break
                remaining = arguments
                launcher = "source"
                continue
            if name in WRAPPERS_TAKING_ONE_ARGUMENT:
                if not arguments or arguments[0].startswith("-") or len(arguments) < 2:
                    commands.append(CommandAtPosition(remaining, command_directory, command_environment, **attached, memory_capped=command_memory_capped))
                    break
                remaining = arguments[1:]
                launcher = None
                command_memory_capped = command_memory_capped or name == MEMORY_CAP_WRAPPER
                continue
            commands.append(CommandAtPosition(remaining, command_directory, command_environment, launcher, **attached, memory_capped=command_memory_capped))
            break
    return ShellScan(commands, prefix_names, has_lone_ampersand, output_targets, tuple(unjudged))


def heredocs_fed(redirections, stripped_heredocs):
    """一条简单命令上 `<<` / `<<-` 喂给它的、被当数据剥掉的 heredoc 正文（HeredocBody，按出现次序）：重定向目标是 HEREDOC_BODY_MARKER 记号的，按编号从 stripped_heredocs 取。"""
    found = [HEREDOC_BODY_MARKER.match(target) for operator, target in redirections if operator in ("<<", "<<-")]
    return tuple(stripped_heredocs[int(marker.group(1))] for marker in found if marker and int(marker.group(1)) < len(stripped_heredocs))
