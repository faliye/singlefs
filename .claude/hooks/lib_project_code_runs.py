"""一条命令是不是本仓之外要经 task-server 起的外部程序、认不认得出起的是什么、起的是不是本仓的代码或编译、是不是绕开服务的写法。

project-code-guard.sh（执行前拒绝）按文件路径用 importlib 导入这一份；同目录的 lib_shell_words.py（切词、剥前缀与包装）由这份自己导入，调用方从 `shell_words` 属性拿同一份。
谁能跑什么、经没经 task-server 提交，是 project-code-guard.sh 的事，不在这里。
代码分在五份：本文件是入口（文件头、部分模块的载入与读不到时的处置、名字的再导出、命令行入口），引导要的名字（弄坏开关变量名与读法、部分文件名表）在同目录的 lib_project_code_runs_bootstrap.py，常量、登记表与阶段样本的读法、cargo 与提交客户端与绕开服务的认法在 lib_project_code_runs_base.py，启动器与展开与 classify 在 lib_project_code_runs_launchers.py，自证在 lib_project_code_runs_selftest.py；本文件按 PROJECT_CODE_RUNS_PART_NAMES 的次序 import 部分模块（后一份 import 前一份、没有环），把它们的名字全部再导出，调用方照旧按这份文件的路径导入、按这份的名字取；
自证与调用方换掉的模块变量（SELFTEST_REGISTRY_PATH、SELFTEST_REGISTRY_ROOT、SAMPLE_INDEX_REPOSITORY、TEMPORARY_SERVICE_TABLE_PATH、BASH_FOR_PATTERNS 在 lib_project_code_runs_base.py，FAMILY_OWN_REPOSITORY 在 lib_project_code_runs_launchers.py）在读它们的那一份里换：调用方经这份文件里绑着的两个部分模块（lib_project_code_runs_base、lib_project_code_runs_launchers 这两个属性）换，换在这份文件自己的再导出名上读它们的函数看不到；
读不到或编不过部分文件的抛 PartFileUnreadable（ImportError 的子类，说明点名是哪一份，弄坏开关 LIB_PROJECT_CODE_RUNS_BREAK=part-unreadable-passes 见下面开关清单），导入这份模块的调用方照各自缺依赖时的处置办；
import 的名字缺了的（编得过但不完整，例如部分文件被截空）同样抛 PartFileUnreadable、点名缺的名字与几份部分文件（弄坏开关 part-incomplete-passes）。

一切跑项目代码的命令只认经 task-server/task 起，所以这一份不分档、不分类：本仓的脚本由 project_code_use 认，会编的 cargo 由 starts_a_build 认，编出来的二进制由
runs_compiled_code 认，认不出起什么的写法由 unrecognized_launch 认；这几样认不出的、本仓之外的外部程序（EXTERNAL_PROGRAM_PREFIXES、EXTERNAL_PROGRAM_NAMES：
前缀 qemu-system、herd7——只问版本的放行、e152-file-system-benchmark，名字取 basename，PATH 上直接起）由 classify 认，命中的 ProgramUse 一律拒、出路是 `task-server/task submit --`。

入口：
  classify(words, directory, environment=None) -> ProgramUse | None
      一条已经剥掉前缀与包装的命令：words[0] 是命令词（照写的样子，没取 basename），其后是参数；directory 是它的当前目录，认不出时 None；
      environment 是这条命令看得到的、命令文本里设过的变量。先照 bash 展开花括号（word_expansion_use），再展开命令词里的通配（pattern_expansion_use），
      展开出来的每一条判成外部程序或本仓的代码（project-code：脚本、会编的 cargo、编出来的二进制、剥掉的包装）就交它（说明里写展开成了什么）；
      都不是再照字面判：名字在外部程序名单里的交 ProgramUse（kind external-program），其余交 None。
      bash 起不来而命令词带 POSIX 字符类、等价类、整理符号的，判不出起的是哪一份，交 kind pattern-unresolved。
      弄坏开关 external-programs-unlisted（名单不查）、herd7-version-flag-counted（herd7 只问版本也算）、pattern-command-words-literal、expanded-project-code-unjudged、
      brace-expansion-literal、brace-sequence-literal、brace-command-words-bare、quoted-braces-expanded、pattern-expansions-bare、pattern-caret-negation-literal、
      posix-brackets-by-python-glob、posix-brackets-unresolved-literal。
  program_uses_in_command_text(text, directory, environment=None) -> list[ProgramUse]
      一段命令文本里命令位置上的每一处外部程序用法：lib_shell_words 剥掉包装，command_under_launcher 剥掉 /usr/bin/time 这一类（剥完再切一遍），逐条 classify。
  judged_by_name(word, directory) -> bool
      这个命令词是不是按名字认的仓内入口脚本（.claude/gate.d/ 下的阶段、KNOWN_SCRIPT_LOCATIONS 里的脚本在它们的仓内位置上）：起它算跑本仓的代码，
      读脚本正文的一方不再读进去。盘上有这份文件、又不是本仓那一份（路径尾部相同的拷贝、经符号链接起的、内容连同它 source / import 的库与本仓那一份不逐字节相同的、
      引的库在本仓那一边挑不到的、正文里有认不出写法又没登记的 source 行的、库往下读撞上深度上限没读完的，same_content_as_own）为假；
      盘上没有的按路径尾部为真。弄坏开关 same-family-by-path-tail（只看路径尾部）、family-copy-content-unchecked（内容相同的拷贝也不认）。
  own_script_digest(location) -> str | None
      本仓里 location 那一份的 sha256，每次调用现读；只从三张字面表里挑（KNOWN_SCRIPT_LOCATIONS 的值、门禁阶段 GATE_STAGE_FILES、派发起法 DISPATCH_LAUNCHER_SCRIPTS），
      表外的位置交 None。同族拷贝比内容（same_content_as_own）按它取本仓那一份，算键库折得出读的是哪几份、不连整个目录。弄坏开关 own-script-digest-stages-unlisted；
      自证另拿与表里那一项相等、字面指到别处的位置核读的是表里的字面路径（开关 own-script-digest-location-joined 换成拿传进来的位置拼路径读的那一种）。
  own_library_digest(library) -> str | None
      本仓里 library 那一份库的 sha256，每次调用现读；只从三张字面库表（GATE_STAGE_AND_PREFLIGHT_LIBRARY_FILES、CRASH_REPLAY_AND_HOST_LIBRARY_FILES、
      MUTATION_PYTHON_LIBRARY_FILES：三张表里每份脚本引的库，script_source_reading 认的与登记了库的，照类分）里逐张挑，
      表外的、本仓盘上没有的交 None（同族拷贝照不同认）。同族拷贝比库（same_content_as_own 的库那一半）按它取本仓那一边，算键库折得出读的是哪几份库：
      每张不超过 OWN_SCRIPT_LIBRARY_TABLE_LIMIT 份（算键库的折叠上限 FOLD_LIMIT 之内），拼成的整张 OWN_SCRIPT_LIBRARY_FILES 只给自证比表、逐行数 source 行用。
      弄坏开关 own-library-digest-unlisted、own-library-table-short（表少一项）、own-library-tables-merged（份数那一格量拼成的整张）。
  registered_selftest(word, arguments, directory) -> SelftestRow | None
      命令是不是登记表 selftest-registry.tsv 里登记过的自证：脚本、钩子、门禁目录顶层 lib- 起头的共用库只认参数正好是 ["--selftest"]，
      .claude/gate.d/lib/ 下的共用库直接执行就是跑自证，不带参数或单独一个 --selftest 都算；
      命令词要解到登记表所在仓里那一份文件（selftest_registered_file）；派发起法与 task-server/task submit 的 `--` 之后那条里的 --selftest 是里面那条的，不是外面这一条的。
      登记表的种类只有三种：selftest（脚本、钩子、门禁共用库的自证）、samples（门禁阶段的样本档，样本只读文本、里面的工具都是假的）、
      samples:<目录>（样本档，样本根里 <目录> 放着假工具，selftest-process-guard.py 不把那一层的工具当真的）。
  stage_selftest_problems(word, arguments, directory, environment=None) -> str
      stage-selftest.sh 喂的样本不盯的原因，一句；盯交 ""：命令词要解到本仓那一份 stage-selftest.sh，喂的每一道阶段的样本档都登记在表里，
      样本与 SAMPLE_INDEX_REPOSITORY 的 HEAD 加暂存区逐字节相同（带 --item 的只判点名的几道，--list-items 只列不喂）。
  command_under_launcher(words, directory=None) -> (list[str], str | None) | None
      words[0] 是 LAUNCHER_OPTIONS_WITH_VALUE 里的程序（/usr/bin/time、flock、rustup run、chrt、prlimit、systemd-run、strace、perf）时，
      剥掉它交回它起的那条命令与那条命令的当前目录（None 是不变）；调用方把交回的命令再交给 lib_shell_words 切一遍、逐条判。
      短选项合写（`-fo <文件>`、`-xw 10`、`-qu <名>`）逐个字母查那张表：头一个要值的字母之后剩下的是它的值，没剩下就取下一个词；
      长选项查要值与不要值两张表（照各自的 --help 列全），值另起一个词的（`--expand-environment no`、`--output <文件>`）跳过那个值，
      只写前缀的（getopt_long 认唯一的前缀，`--outp`）按前缀认；
      systemd-run 的 `-E NAME=VALUE` / `--setenv=NAME=VALUE`、`-p Environment=…` / `--property=Environment=…`（`EnvironmentFile=<文件>` 按内容读，
      读不了当它没设）、strace 的 `-E NAME=VALUE` / `--env=NAME=VALUE` 设的变量写在交回那条命令的最前面
      （再切一遍时当它的环境变量，认别名要用；strace 的 `-E NAME` 只清变量，不带）。
      这张表不进 lib_shell_words 的前缀表：bash-command-detector.sh 要按命令词认出 systemd-run，判它等不等结束。
  command_under_gpu_quiesce(words, directory=None) -> (list[str], None) | None
      words[0] 是 scripts/crash/gpu-quiesce-run.sh 而参数是 `-- <命令…>`（清场之后原样起那条命令；参数里的 --force 照它的 preflight 每一个都摘）时交回 `--` 之后那条命令，
      当前目录与环境照外面那一条。认它照名字与位置：名字叫 gpu-quiesce-run.sh、又落在 scripts 这一层（路径尾部，同 located_in）；
      别处的同名拷贝不剥（project-code-guard.sh 照旧读它的正文）；--selftest 与没写 `--` 的不剥。不并进 command_under_launcher：
      task-server/task_server.py 按名字认 gpu-quiesce-run.sh 判要不要卡，剥掉就认不出；project-code-guard.sh 在 command_under_launcher 之后再试它。
  command_under_two_logs(words, directory=None) -> (list[str], None) | None
      words[0] 是 scripts/hosts/two-logs.sh（日志出两份的包装：转打、写 <名>.log 与 <名>.problems.log，退出码是被包命令的）而参数是 `[--quiet] <名> -- <命令…>` 时交回
      `--` 之后那条命令，当前目录与环境照外面那一条；它不摘参数里的 --force，`--` 之后原样交回。认它照名字与位置：名字叫 two-logs.sh、又落在 scripts/hosts 这一层
      （路径尾部，同 located_in）；别处的同名拷贝不剥；--selftest、--from-log 与没写 `--` 的不剥。command_under_launcher 先试它，
      所以 project-code-guard.sh、run-scope-guard.sh 都看得穿它：包一层日志不让里面那条脱离这些闸。
  command_under_run_build_directory(words, directory=None) -> (list[str], None) | None
      words[0] 是 scripts/hosts/run-build-directory.sh（包一层起：这一趟一个自己的编译目录、跑完删）而参数是 `[--] <命令…>` 时交回那条命令，当前目录与环境照外面那一条。
      认它照名字与位置（同 command_under_two_logs）；--selftest、没给命令的不剥。command_under_launcher 在 two-logs.sh 之后试它：包一层编译目录同样不让里面那条脱离这些闸；
      lib_workspace_cargo_lock.py 的编译目录那一判另凭它认出「这条 cargo 有自己的编译目录」。
  command_under_extra_launcher(words) -> (list[str], str | None) | None
      名单外启动器（EXTRA_LAUNCHER_NAMES：doas、numactl、unshare、setpriv、nsenter、ltrace、valgrind、faketime、unbuffer、cpulimit、dbus-run-session、firejail、proot、chroot、cgexec、
      bwrap、xargs、find -exec / -execdir / -ok / -okdir、ssh、su -c、runuser、sg、script -c、watch、gdb --args、scl enable、parallel；一百三十七节 ②、research/prompts/shell-words-r1-main-verification.md 报的 25 个、
      research/prompts/shell-words-r3-sonnet-s2-output.md 第八节 A3、A4）：command_under_launcher 认不出名字时再试它。xargs、find、parallel 读进来接进命令里的词换成没展开的占位词（XARGS_INPUT_WORD、FIND_PATH_WORD、
      PARALLEL_INPUT_WORD），ssh、su -c、sg、script -c、watch 交 `sh -c <代码>`。launcher_names() 是两张表的名字（bash-command-detector.sh 的自证钉住它那一份与这里一致）。
      弄坏开关 extra-launchers-unseen 下不剥。
  command_under_dispatch_launcher(words, directory=None) -> (list[str], None) | None、dispatched_under_memory_cap(words, directory=None) -> bool
      派发起法 `bash scripts/hosts/multi-host-run.sh --peer [选项…] -- <命令…>`（它只有带 --peer 这一种派发用法，不带退 2、什么都不跑；别的选项 --memory-cap <上限>、--label <标签>）
      剥开交回 `--` 之后那条命令，当前目录与环境照外面那一条；认它照名字与位置（落在 scripts/hosts 这一层，located_in）；
      --selftest 与没写 `--` 的不剥。host-dispatch.py 是 task-server 载入的判法库（命令行只认 --selftest），不在这里剥。
      不并进 command_under_launcher（task-server/task_server.py 借 command_under_launcher 剥启动器、按名字认 GPU 用法）：project-code-guard.sh 在 command_under_launcher、command_under_gpu_quiesce 之后再试它，
      交给 lib_workspace_cargo_lock.py 判编译目录时也剥它。剥出来的那条算经内存包装（dispatched_under_memory_cap：命令词解到盘上而不是本仓那一份、内容又不同的不算，
      当前目录认不出的不算）。registered_selftest 不把剥得开的这一条当派发器自己的自证
      （`… -- <脚本> --selftest` 里的 --selftest 是里面那条的）。弄坏开关 dispatch-launchers-unseen 下不剥。
  temporary_service_wrapped(words, directory=None) -> (list[str], bool) | None、command_under_temporary_service(words, directory=None) -> (list[str], None) | None
      临时 prune-server 的包装写法 `python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service [--server-binary <二进制>] -- <原命令…>`
      （它起一份临时服务、在它底下起原命令，客户端文件头「命令行」）：认它照名字与位置（落在 scripts/prune-server 这一层，located_in），剥法照 command_under_dispatch_launcher，
      交回 `--` 之后那条原命令与给没给 --server-binary（-- 之前有认不出的参数照严的一侧取第一个 -- 之后那条）。给了 --server-binary 的包装不编、只起那份二进制，
      command_under_launcher 把它剥掉，里面那条原命令照判（project-code-guard.sh、lib_workspace_cargo_lock.py 都经它剥），registered_selftest 不把它当包装自己的自证；
      没给的包装在没有任务服务给的编译目录时退 2、不现编，有编译目录时现编 prune-server（cargo build --release），都不剥；拒绝说明（temporary_service_command）给的整条写法是不带 --server-binary 的这一种（经 task-server 提交，服务认成会编的、给编译目录），自己编了、两台都读得到的二进制才另加 --server-binary。scripts/gate/selftest-process-guard.py 照同一份认法
      按里面那条原命令判盯不盯。弄坏开关 temporary-service-wrapper-unseen 下一律交 None。
  temporary_service_bare_selftest(words, directory, environment=None) -> str | None、environment_under_temporary_service(environment) -> dict、
  temporary_service_command(key) -> str、temporary_service_table_problems(path, registry_path, root) -> list[str]
      要在临时 prune-server 底下起的自证（同目录的 selftest-temporary-service.tsv 登记，一行 `<路径> required|own-server # <理由>`；required 的自证 --selftest 开跑先核临时服务，
      环境里没有 TEMPORARY_SERVICE_VARIABLE 就退 2、什么都不判）写成裸的单独一个 --selftest 时，temporary_service_bare_selftest 交一句说明（为什么、整条包装写法，
      temporary_service_command 拼：.py 用 python3，别的用 bash），不是交 None。命令词要解到自证登记表所在仓里那一份、表里判定是 required、环境里没有标记才算；
      包装 `… --with-temporary-service --server-binary <二进制> -- <原命令>` 剥开重判里面那条时 project-code-guard.sh 用 environment_under_temporary_service 补上标记，
      命令前自己设了标记的同样放行。不并进 registered_selftest：它也被 scripts/gate/selftest-process-guard.py 与 command_under_launcher 剥开后的重判共用，
      改了它，包装里的原命令就盯不上；project-code-guard.sh 在命令位置上单独调这一个。temporary_service_table_problems 把这张表与自证登记表、脚本源码两边对一遍（表头写了两条）。
      弄坏开关 temporary-service-bare-unjudged、temporary-service-own-server-as-required、temporary-service-marker-ignored、temporary-service-environment-dropped、
      temporary-service-table-lenient、temporary-service-command-bare、temporary-service-mark-entry-only、temporary-service-family-by-directory、
      temporary-service-family-by-name、temporary-service-family-by-loading、temporary-service-family-by-text、temporary-service-family-absolute-paths（开关表逐个写红的格）。
      required 行的「源码里」是入口连同它的拆分族（source_family，认法只在 scripts/gate/lib_split_family_loaded.py 里写：同目录里名字落在命名族里、又被入口或族里别的文件载入的文件，
      载入是 import、from … import、同目录的相对 import、字面量参数的 importlib.import_module、赋给 *PART_NAMES 的写死文件名与 shell 的 source，注释与文档串里写的不算；
      名字像部分文件而入口没载入的、被载入而名字不像部分文件的共用库与被字符串点名的别的脚本、别的目录的库都不算）；
      门禁阶段入口的部分文件在 lib/ 下，不在入口的同目录里，由调用方经 source_family 的 candidates 参数给候选；
      漏登那一条只看入口原文。
  temporary_service_bare_stage_samples(words, directory, environment=None) -> str | None
      门禁样本档的同一类裸写：命令词是本仓那一份 stage-selftest.sh、用 --item 点名了阶段目录里配了样本的几道，其中有阶段源码里调了临时服务核对
      （TEMPORARY_SERVICE_SOURCE_MARK）的，环境里又没有标记，交一句说明（点名哪几道、把原命令整条套上包装的写法 wrap_in_temporary_service），不是交 None。
      哪几道要套每次从阶段源码现读，不另写名单；不带 --item 的、--list-items、阶段目录或 --item 写在变量里的交 None（别处另判）。标记与包装的认法同
      temporary_service_bare_selftest。project-code-guard.sh 在命令位置上单独调这一个。
      弄坏开关 temporary-service-stage-samples-unjudged、temporary-service-stage-samples-every-stage、temporary-service-marker-ignored、temporary-service-command-bare、
      temporary-service-command-binary-placeholder（开关表逐个写红的格）。
  unrecognized_launch(words, launcher=None) -> str | None、cargo_arguments_expanded(words, variables) -> list[str]
      认不出起什么的写法（project-code-guard.sh 在命令文本这一层判，kind unrecognized、类名「认不出起什么的写法」；认不出的一律拒）：
      命令词或 bash / source 起的脚本路径带展开不了的变量、命令替换、位置参数、进程替换，或展开出来夹着空白；source / . 的文件写成这样；cargo 起编出来的代码的子命令
      （CARGO_RUNNING_SUBCOMMANDS）或参数写成这样（`--` 之后、CARGO_HARMLESS_VALUE_OPTIONS 的值除外）。cargo 参数写成前面赋过字面值的变量的先由 cargo_arguments_expanded 展开。
      弄坏开关 unrecognized-launches-allowed、cargo-arguments-unexpanded。
  script_directory_expanded(text, script_path) -> str
      读脚本正文时把按自己所在目录取路径的写法（SCRIPT_DIRECTORY_FORMS）换成脚本所在的目录（照起它的路径，不解符号链接），source 的库照字面路径读进来判
      （弄坏开关 sourced-libraries-unread）。
  shell_source_lines(text, file_name, inherited=None) -> ([(行号, 照写的参数, 位置或 None)], ShellPlaces)、script_source_reading(path, location=None, depth_limit=None) -> SourceReading、
  script_sourced_libraries(path, location=None) -> list[str]
      shell_source_lines 列一份 shell 文件里的每一条 source / .（注释行不算）与它引的位置（相对这份文件所在目录）。认的地方：命令位置（行首、; & | ( { ! 之后、case 分支的 ) 之后、
      then / do / else / elif / if / while / until 之后，中间可以垫命令前的赋值 `QUIET=1` 与 builtin / command；shell_source_command），与引号里交给 eval 或子 shell
      （bash / sh / dash / zsh / ksh 带 -c）的那段代码起头的（SHELL_QUOTED_SOURCE）。位置：按自己所在目录的写法接字面路径、
      变量（`X="$(cd "$(dirname "$0")" && pwd)"`、`R="$(cd "$X/../.." && pwd)"`、`$(dirname "<脚本自己>")`、`L="$X/lib"`，每一处赋值都解到同一个位置才认）接字面路径、
      顶层 cd 之后的相对字面路径；引号里那几条、参数是进程替换 `<(…)` 的（照写的参数是那整段）与别的认不出的位置是 None。
      认不到的写法（不交回、不进认不出）：引号里交给 eval / 子 shell 的代码中间（不在起头、前面也不是 ; & | 这类命令位置）的 source，跨行的 eval / bash -c 串，
      引号里交给 su -c、ssh、xargs、find -exec 这类别的程序的代码里起头的 source，别的程序（python、awk、perl）里起 shell 再 source 的。
      script_source_reading 连同库再引的一起读（库看得到引它的那份设的变量；入口与读过的库只读一遍，库再引回入口、再引读过的库不再读、不往下多占一层），往下读到
      depth_limit 层（不给取 SOURCED_LIBRARY_DEPTH），落在那一层的库进 cut、不再读；交引的库、认不出又没登记的 source 行、用到的登记、读到的每一行与 cut。
      OWN_SCRIPT_UNRECOGNIZED_SOURCES 按「本仓那一份文件、照写的参数、它是那一份里第几条照写这个参数的 source 行」登记认不出的几行引的是哪一份库或不是库
      （registered_source_key）：同一份里多出一条照写同一个参数的，第几条对不上，照没登记办。script_sourced_libraries 只交库。
      同族拷贝（same_content_as_own）照它逐份比 sha256，本仓那一边照 own_library_digest 从几张库表里逐张挑，有认不出又没登记的 source 行的、cut 不空的照不同认。
      弄坏开关 script-directory-variables-skipped（变量与 cd 不认、认不出的静默跳过）、unrecognized-sources-skipped（认不出的不交回）、case-arm-position-unseen、elif-position-unseen、
      source-prefixes-unseen、quoted-shell-sources-skipped、process-substitution-sources-skipped、unrecognized-sources-registered-by-argument、library-entry-reread、
      library-depth-cut-unreported（开关表逐个写红的格）。
  runs_compiled_code(words, directory) -> str | None
      一条已经剥掉前缀与包装的命令会不会跑编译出来的代码：cargo test / t / run / r / bench（test、bench 带 --no-run 的只编不跑，不算），
      或直接执行 cargo 编出来的二进制（测试二进制，与名字里带 target 的目录底下 debug / release 里的）；会就交一句说明。
      project-code-guard.sh 拿它认「跑项目代码」的一种：不经 task-server/task 起的拒（下面「经 task-server 起」）。
  starts_a_build(words, directory) -> str | None
      一条已经剥掉前缀与包装的命令会不会起编译：会编的 cargo（CARGO_NON_BUILDING_SUBCOMMANDS 之外的子命令、别名与插件，test 带 --no-run 的也算），
      或自己在里面起会编的 cargo、里面不套内存包装的仓内脚本（SCRIPTS_STARTING_BUILDS，按文件名认；单独一个 --selftest 不算）；会就交一句说明。
      project-code-guard.sh 拿它认「跑项目代码」的一种（经没经内存包装都一样：不经 task-server/task 起的拒，下面「经 task-server 起」），主 agent 与子 agent 一样判、不看目录。
  project_code_use(words, directory, launcher=None) -> ProjectRun | None、task_server_submission(words, directory, launcher=None) -> (list[str], str | None) | None、
  task_server_text_bypass(text) -> ([str], str | None)、task_server_command_bypass(words, environment, directory, launcher=None) -> ([str], str | None)
      下面「经 task-server 起」那一段的四样判法；名单 TEXT_TOOLS_RUN_DIRECTLY、MAIN_AGENT_TOOLS_RUN_DIRECTLY 只写在这一份里。
      task_server_text_bypass 里载入共用库那一判（task_server_library_loaded）：文本里有库名（模块名 task_server_lib，文件名 task_server_lib.py 里也有它）才切；
      照 lib_shell_words 的 commands_at_command_position 切成命令位置上的一条条命令（bash -c、eval、命令替换里的照递归切）。不判的：只读命令（命令词是 grep、egrep、fgrep、rg、wc、cat、
      head、tail、ls、stat、sha256sum、md5sum、diff、cmp，带 -n 而不带 -i 的 sed，git 的 grep / show / diff / log / blame / ls-files；task_server_read_only_command）连同它读的 heredoc，
      cat / tee 写进不是 .py 的文件的 heredoc 正文。判的：写进 .py 文件的 heredoc 正文各算一段；别的每一条命令的词（python3 -c 的代码在里面）、喂给它的 heredoc 正文、
      管道前一段的词与 < / <<< 的目标连起来算一段；同一段里库名与载入写法（TASK_SERVER_LIBRARY_LOAD_WORDS：import、__import__、import_module、exec、execfile、compile、run_module、
      run_path、spec_from_file_location、SourceFileLoader、SourcelessFileLoader、load_source、load_module、exec_module）同在就算载入，拆在几行、先存进变量再用的一样算；
      python 从标准输入读程序（不带 -c、-m、脚本），标准输入是 < 一份写着库名的文件或管道前一段只读命令读的它，同样算。偏严的一面：同一条命令里只是 import 了别的模块、
      又写到库名的（python3 -c "import json; print('task_server_lib')"、echo 里写的说明）照拒，出路是把名字拆开写或改用只读命令。
      认不到的：库名本身拆开拼的（'task_server_' + 'lib'）、经变量拿到模块名的；python3 起的 .py 脚本文件正文不读（本仓多份脚本按设计载入这份库，读正文判会拒掉现役命令；
      本仓的脚本由「跑本仓的代码」那一判管）；先 cp / cat 拷成别的名字再起的；awk、perl、ruby 这类别的程序里载入 python 的；切不动的命令文本整段当一段判（偏严）。
      弄坏开关 task-server-library-loads-unjudged、task-server-library-loads-whole-text、task-server-library-loads-same-statement、task-server-library-loads-module-name-unseen、
      task-server-library-load-words-file-calls-only、task-server-library-loads-written-python-unjudged、task-server-library-stdin-program-unjudged（开关表逐个写红的格）。
  task_server_client_subcommand(words, directory, launcher=None) -> str | None、subagent_task_server_refusal(words, directory, launcher=None) -> str | None
      下面「经 task-server 起」那一段的子 agent 那一判：子 agent 起提交客户端只许 status、log；出路那一句 SUBAGENT_HAND_BACK_WAY_OUT 只写在这一份里。
  selftest_registry_problems(path, root) -> list[str]
      登记表写坏的行（判定认不出、理由太短、不在三处目录里、重复）与指到 root 下不存在的文件的行，逐条一句；空表是没毛病。

弄坏开关（只给自证判红用）：LIB_PROJECT_CODE_RUNS_BREAK 设成下面一个或几个（逗号分隔），--selftest 与 project-code-guard.sh --selftest 都必须判红；
平常跑（命令行不是单独一个 --selftest）设了就拒、退 2，不认识的值也拒。一条一行「LIB_PROJECT_CODE_RUNS_BREAK=<项>  弄坏哪一处」，下一行起写打开后红的自证项（失败行 `✗ lib_project_code_runs 自检：` 之后的句首）。
  LIB_PROJECT_CODE_RUNS_BREAK=active-sessions-unlisted  MAIN_AGENT_TOOLS_RUN_DIRECTLY 里 scripts/agents/active-sessions.py 那一项不认（别的会话工具照认）
      「名单：主 agent 查活动会话的 active-sessions.py」
  LIB_PROJECT_CODE_RUNS_BREAK=admitted-variable-unjudged  两份判法都不认「已由 task-server 准入」开关
      「绕开：命令文本里 export 准入开关」「绕开：python 代码里写 environ 的准入开关」「绕开：命令前设准入开关」
  LIB_PROJECT_CODE_RUNS_BREAK=bench-unjudged  cargo bench 不判（bench 在链接 验证包的包上、验证包上、工作区全量上都放过去）
      「跑编译出来的代码：cargo bench --no-run 只编不跑」
  LIB_PROJECT_CODE_RUNS_BREAK=brace-command-words-bare  命令词展开过的那几条直接 classify，不再切词剥包装
      「展开：花括号拼出 cargo test（{cargo,test,-p,singlefs-core}）」
  LIB_PROJECT_CODE_RUNS_BREAK=brace-expansion-literal  花括号一律照字面、不展开
      「花括号：数字序列展开（herd{1..9} 里有 herd7）」「展开：花括号拼出 cargo test（{cargo,test,-p,singlefs-core}）」
      「花括号展开与 bash 一致：」（词表里 bash 展得开的每一个词都红，一项照出这几十行）
  LIB_PROJECT_CODE_RUNS_BREAK=brace-sequence-literal  {1..3}、{a..e} 这类序列不展开
      「花括号：数字序列展开（herd{1..9} 里有 herd7）」「花括号展开与 bash 一致：{1..3}」「花括号展开与 bash 一致：{3..1}」「花括号展开与 bash 一致：{01..10}」
      「花括号展开与 bash 一致：{1..10..3}」「花括号展开与 bash 一致：{a..e}」「花括号展开与 bash 一致：{a..e..2}」「花括号展开与 bash 一致：{-2..2}」
      「花括号展开与 bash 一致：{-01..3}」「花括号展开与 bash 一致：{1..-03}」「花括号展开与 bash 一致：{0..00}」「花括号展开与 bash 一致：{1..05..2}」
      「花括号展开与 bash 一致：{01..100..33}」「花括号展开与 bash 一致：{100..00..50}」「花括号展开与 bash 一致：{+1..3}」「花括号展开与 bash 一致：{1..3..0}」
      「花括号展开与 bash 一致：x{1..3}y{a,b}」「花括号展开与 bash 一致：{1..2}{3..4}」
  LIB_PROJECT_CODE_RUNS_BREAK=build-scripts-unjudged  starts_a_build 不认 SCRIPTS_STARTING_BUILDS 里的脚本（compile-then-swap.py 一律交 None）
      「起编译：python3 起 compile-then-swap.py」「起编译：直接执行别处树里的 compile-then-swap.py」
  LIB_PROJECT_CODE_RUNS_BREAK=build-subcommands-unjudged  starts_a_build 不认会编的 cargo（一律交 None）
      「起编译：cargo build」「起编译：cargo b 简写」「起编译：cargo check」「起编译：cargo clippy」「起编译：cargo test --no-run 只编不跑也算」「起编译：cargo doc」「起编译：cargo run」「起编译：全局选项在子命令前」「起编译：--config 定的别名当会编」九格红（应当是 True，实际 False）
  LIB_PROJECT_CODE_RUNS_BREAK=cargo-arguments-unexpanded  cargo 参数不展开
      「cargo 参数展开：F=--workspace 之后 cargo test $F」
  LIB_PROJECT_CODE_RUNS_BREAK=case-arm-position-unseen  shell_source_command() 里 case 分支的 ) 之后不算命令位置
      「source 的写法：case 分支 ) 之后的 source 认出」
  LIB_PROJECT_CODE_RUNS_BREAK=checked-cells-last-only  checked_cells() 给几次 --check 只认最后一次（不取并集）
      「阶段参数 --check：--check=a 与 --check b 给两次取并集」「阶段参数 --check：--check 给三次（含 --check= 写法）取并集」
  LIB_PROJECT_CODE_RUNS_BREAK=claude-executable-path-unrecognized  unrecognized_launch() 把命令词 "$CLAUDE_CODE_EXECPATH"、"${CLAUDE_CODE_EXECPATH}" 照变量拼出的认不出：
      「认得出：命令词是 $CLAUDE_CODE_EXECPATH（检出钩子 ⑬ 判起 claude）」「认得出：命令词是 ${CLAUDE_CODE_EXECPATH}（检出钩子 ⑬ 判起 claude）」
  LIB_PROJECT_CODE_RUNS_BREAK=command-word-variables-unexpanded  command_word_expanded 原样交回（命令词写在变量里的看不见）
      「命令词变量：S=herd7 之后 $S 起 herd7」
  LIB_PROJECT_CODE_RUNS_BREAK=configuration-before-subcommand-only  cargo 子命令之后的 --config 不认
      「跑编译出来的代码：--config 写在子命令之后的别名」
  LIB_PROJECT_CODE_RUNS_BREAK=device-layer-unrecognized  实验装置那一层的登记不认
      「样本表：装置那一层登记过的单独 --selftest 认」「样本登记表里写坏的几行逐条报出来（…）」
      「要临时服务：仓里的表（…）与自证登记表、脚本源码对得上」「仓里的登记表（…）没有写坏的行、指到的文件在 … 里都在」
  LIB_PROJECT_CODE_RUNS_BREAK=dispatch-launchers-unseen  command_under_dispatch_launcher() 一律交 None（multi-host-run.sh --peer … -- 里面那条看不见）
      「启动器：multi-host-run.sh --peer 带 --memory-cap、--label 各一个值」
  LIB_PROJECT_CODE_RUNS_BREAK=elif-position-unseen  shell_source_command() 里 elif 之后不算命令位置
      「source 的写法：elif 条件里的 source 认出」
  LIB_PROJECT_CODE_RUNS_BREAK=expanded-project-code-unjudged  展开（花括号、通配）出来的本仓的代码、会编的 cargo、剥掉的包装一律不认（project_code_program_use 交 None）
      「展开：通配起本仓的脚本经 capped.sh（scripts/hosts/capped.s? 4 true）」「展开：花括号拼出 cargo test（{cargo,test,-p,singlefs-core}）」
  LIB_PROJECT_CODE_RUNS_BREAK=external-programs-unlisted  外部程序名单不查（qemu-system-*、herd7、e152-file-system-benchmark 直接起放过去）
      「应当是 external-program，实际 None」（外部程序、包在命令外面的启动器、花括号、通配、标准输入与命令词变量里，该认出外部程序的每一格都红，一项照出这些行）
  LIB_PROJECT_CODE_RUNS_BREAK=extra-launchers-unseen  名单外启动器不剥（应当是起的那条命令，实际 None）；project-code-guard.sh 的八格见它的文件头
      「启动器：」
  LIB_PROJECT_CODE_RUNS_BREAK=family-copy-content-unchecked  内容相同的拷贝也不认
      「同族：与本仓那一份逐字节相同的 mutate.sh 拷贝照按名字判」「同族：与本仓那一份逐字节相同的门禁阶段拷贝照按名字判」
      「派发起法：与本仓那一份逐字节相同的 multi-host-run.sh --peer 拷贝剥开的算经内存包装」
      「同族：连同 source 的库都与本仓那一份逐字节相同的门禁阶段拷贝照按名字判（crash-point-replay.sh 与它 lib/ 下的库）」
      「同族：先加 source 行、后建库：两边都建好、逐字节相同的拷贝照按名字判」
  LIB_PROJECT_CODE_RUNS_BREAK=gate-top-library-as-stage  门禁目录顶层以 lib- 起头的共用库当门禁阶段判：登记表里那一行当门禁阶段的样本档行（selftest 认不出、算写坏的行），命令认不出是它的自证
      「样本登记表里写坏的几行逐条报出来（…）」「仓里的登记表（…）没有写坏的行、指到的文件在 … 里都在」「样本表：门禁目录顶层 lib- 起头的共用库登记过，单独 --selftest 认」
  LIB_PROJECT_CODE_RUNS_BREAK=gpu-quiesce-opaque  command_under_gpu_quiesce 一律交 None，gpu-quiesce-run.sh -- <命令> 不剥（里面那条看不见）
      「gpu-quiesce-run.sh 清场之后起命令的包装」
  LIB_PROJECT_CODE_RUNS_BREAK=gpu-verification-server-directory-unrecognized  自证认的目录里去掉 gpu-verification-server（selftest_directories；登记表里 gpu-verification-server/control 那一行算写坏的行，它的 --selftest 不当自证）
      「样本登记表里写坏的几行逐条报出来（…）」「仓里的登记表（…）没有写坏的行、指到的文件在 … 里都在」「样本表：gpu-verification-server 那一层登记过的单独 --selftest 认」
  LIB_PROJECT_CODE_RUNS_BREAK=herd7-version-flag-counted  herd7 只问版本（-version、--version）也算外部程序
      「外部程序：herd7 -version 只问版本，放行」「外部程序：herd7 --version 只问版本，放行」
  LIB_PROJECT_CODE_RUNS_BREAK=launcher-drops-property-environment  systemd-run -p Environment= / EnvironmentFile= 设的变量不带进里面那条命令
      「启动器环境：systemd-run -p Environment= 设别名」「启动器环境：systemd-run -p EnvironmentFile= 文件里设别名」
      「启动器环境：systemd-run --property=Environment= 设两个变量、第二个是别名」「启动器环境：systemd-run --prop Environment=（长选项只写前缀）设别名」
  LIB_PROJECT_CODE_RUNS_BREAK=launcher-drops-setenv  systemd-run -E / --setenv= 设的变量不带进里面那条命令（strace -E / --env 设的同样不带）
      「启动器环境：systemd-run -E 设别名」「启动器环境：systemd-run --setenv= 设别名」「启动器环境：systemd-run -qE 合写设别名」
      「启动器环境：strace -E 设别名」「启动器环境：strace --env= 设别名」「启动器环境：strace --env 值另起一个词设别名」「启动器环境：strace -fE 合写设别名」
  LIB_PROJECT_CODE_RUNS_BREAK=launcher-long-options-partial  启动器的长选项只写前缀、值另起一个词的不认
      「strace --output 文件（值另起一个词）包一层」「strace -c --summary-col calls（长选项只写前缀）包一层」
      「systemd-run --user --scope --expand-environment no（值另起一个词）包一层」「strace --string-limit 200 包一层」
      「启动器环境：systemd-run --prop Environment=（长选项只写前缀）设别名」
  LIB_PROJECT_CODE_RUNS_BREAK=launcher-whole-word-options  启动器的短选项合写不逐字母查表，整个词当一个选项
      「strace -fo 文件（短选项合写）」「flock -xw 10 锁文件（短选项合写）」「systemd-run --user --scope -qu 名字（短选项合写）」
      「/usr/bin/time -ao 文件（短选项合写）」
  LIB_PROJECT_CODE_RUNS_BREAK=library-depth-cut-unreported  script_source_reading() 落在深度上限那一层的库不进 cut（静默截断，同族拷贝不因它读正文）
      「同族：库往下读撞上深度上限、有一份没读完的，逐字节相同的门禁阶段拷贝照不同判（读正文）」
  LIB_PROJECT_CODE_RUNS_BREAK=library-depth-limit-lowered  自证表那一格核「读完没撞上限」那一项时照 SOURCED_LIBRARY_LOWERED_DEPTH 读（本仓三张表里的那几串撞上上限）
      「同族：OWN_SCRIPT_LIBRARY_FILES 与三张表里每份脚本引的库逐个相同，表里每一份都在本仓盘上，每份脚本引的库都在深度上限之内读完」
  LIB_PROJECT_CODE_RUNS_BREAK=library-entry-reread  script_source_reading() 不先把入口记成读过：库再引回入口时入口当库再读一遍、往下多占一层（scripts/mutation/broad-mutation.py 读出自己）
      「source 的库：库再引回入口时入口不当库再读、不往下多占一层（链长到深度上限前一层的照读完）」
      「同族：OWN_SCRIPT_LIBRARY_FILES 与三张表里每份脚本引的库逐个相同，表里每一份都在本仓盘上，每份脚本引的库都在深度上限之内读完」
  LIB_PROJECT_CODE_RUNS_BREAK=libtest-list-by-presence  libtest_lists_only() 见到 --list 这个词就算只列（不跳过带值选项的值、不在 -- 处停）
      「libtest 参数：--skip --list（--list 是 --skip 的值）不是只列」「libtest 参数：--logfile --list（--list 是日志文件名）不是只列」「libtest 参数：-- 之后的 --list 是过滤词，不是只列」「libtest 参数：-- 隔开两段之后的 --list 是过滤词，不是只列」
  LIB_PROJECT_CODE_RUNS_BREAK=main-agent-tools-refused  MAIN_AGENT_TOOLS_RUN_DIRECTLY 不认
      「名单：主 agent 停进程的 proc.py」「名单：主 agent 的 task-server/control stop」「名单：主 agent 起定时提醒的 remind.sh」「名单：主 agent 直接起服务自证」
      「名单：主 agent 的 task-server/control pause」「名单：主 agent 的 task-server/control recognized-card-lease」「名单：主 agent 的 task-server/control resume」
      「名单：主 agent 的 gpu-verification-server/control start」「名单：主 agent 查活动会话的 active-sessions.py」
  LIB_PROJECT_CODE_RUNS_BREAK=mutants-help-counted  test_invocation 里 cargo mutants 只带 --help、-h、--version、-V 的照旧算跑测试
      「跑编译出来的代码：cargo mutants --help 只打帮助，不起 cargo test」「跑编译出来的代码：cargo mutants -V -h 只打版本与帮助，不起 cargo test」
  LIB_PROJECT_CODE_RUNS_BREAK=own-library-digest-unlisted  own_library_digest() 一律交 None（本仓那一边的库挑不到，引了库的拷贝照不同认、读正文）
      「同族：与本仓那一份逐字节相同的 mutate.sh 拷贝照按名字判」「同族：与本仓那一份逐字节相同的门禁阶段拷贝照按名字判」
      「派发起法：与本仓那一份逐字节相同的 multi-host-run.sh --peer 拷贝剥开的算经内存包装」
      「同族：连同 source 的库都与本仓那一份逐字节相同的门禁阶段拷贝照按名字判（crash-point-replay.sh 与它 lib/ 下的库）」
      「同族：先加 source 行、后建库：两边都建好、逐字节相同的拷贝照按名字判」
      「同族：本仓那一份照表里的字面路径读，不拿传进来的位置拼（与表里那一项相等、字面指到别处的位置照样读到本仓那一份）」
  LIB_PROJECT_CODE_RUNS_BREAK=own-library-table-phantom  表那一格比的表里多出 OWN_SCRIPT_LIBRARY_PHANTOM_BY_BREAK（.claude/gate.d/lib/library-not-on-disk.sh，本仓盘上没有）：比的表多一项、「表里每一份都在本仓盘上」那一项列出它
      「同族：OWN_SCRIPT_LIBRARY_FILES 与三张表里每份脚本引的库逐个相同，表里每一份都在本仓盘上，每份脚本引的库都在深度上限之内读完」
  LIB_PROJECT_CODE_RUNS_BREAK=own-library-table-short  OWN_SCRIPT_LIBRARY_FILES 少 OWN_SCRIPT_LIBRARY_DROPPED_BY_BREAK（.claude/scripts/preflight.sh）那一项：own_library_digest() 挑不到它、表那一格比的表里没有它
      「同族：OWN_SCRIPT_LIBRARY_FILES 与三张表里每份脚本引的库逐个相同，表里每一份都在本仓盘上，每份脚本引的库都在深度上限之内读完」
      「同族：与本仓那一份逐字节相同的 mutate.sh 拷贝照按名字判」「同族：与本仓那一份逐字节相同的门禁阶段拷贝照按名字判」
      「派发起法：与本仓那一份逐字节相同的 multi-host-run.sh --peer 拷贝剥开的算经内存包装」
      「同族：连同 source 的库都与本仓那一份逐字节相同的门禁阶段拷贝照按名字判（crash-point-replay.sh 与它 lib/ 下的库）」
      「同族：先加 source 行、后建库：两边都建好、逐字节相同的拷贝照按名字判」
  LIB_PROJECT_CODE_RUNS_BREAK=own-library-tables-merged  自证「库表：」那一格不量三张库表、量它们拼成的整张 OWN_SCRIPT_LIBRARY_FILES 一张（超过 OWN_SCRIPT_LIBRARY_TABLE_LIMIT 份，那一项列出它）
      「库表：每张库表不超过 48 份（算键库折得出 own_library_digest 逐张读的是哪几份库），量的几张拼起来就是整张 OWN_SCRIPT_LIBRARY_FILES」
  LIB_PROJECT_CODE_RUNS_BREAK=own-script-digest-location-joined  自证里拿 location_joined_digest 顶替 own_script_digest：先查三张字面表，查中拿传进来的位置拼路径读
      「同族：本仓那一份照表里的字面路径读，不拿传进来的位置拼（与表里那一项相等、字面指到别处的位置照样读到本仓那一份）」
  LIB_PROJECT_CODE_RUNS_BREAK=own-script-digest-stages-unlisted  own_script_digest() 不从 GATE_STAGE_FILES 里挑（门禁阶段本仓那一份的 sha256 一律交 None，别处逐字节相同的阶段拷贝照读正文）
      「同族：与本仓那一份逐字节相同的门禁阶段拷贝照按名字判」「同族：本仓那一份的 sha256 按位置都挑得到（KNOWN_SCRIPT_LOCATIONS 的每个值、门禁目录顶层的每一道阶段、DISPATCH_LAUNCHER_OPTIONS 的每个键）」
      「同族：连同 source 的库都与本仓那一份逐字节相同的门禁阶段拷贝照按名字判（crash-point-replay.sh 与它 lib/ 下的库）」
      「同族：门禁目录顶层列出来的阶段与 GATE_STAGE_FILES 逐个相同、每一道从表里挑出的 sha256 与盘上那一份相同」
      「同族：本仓那一份照表里的字面路径读，不拿传进来的位置拼（与表里那一项相等、字面指到别处的位置照样读到本仓那一份）」
      「同族：先加 source 行、后建库：两边都建好、逐字节相同的拷贝照按名字判」
  LIB_PROJECT_CODE_RUNS_BREAK=part-incomplete-passes  部分文件编得过但缺入口要的名字（被截空）时照原样抛 ImportError（模块层引到被截掉的名字时抛 NameError；不换成 PartFileUnreadable、不点名「部分文件」）
      「引导:截空部分文件 lib_project_code_runs_base.py 抛 PartFileUnreadable、点名那一份」「引导:截空部分文件 lib_project_code_runs_launchers.py 抛 PartFileUnreadable、点名那一份」「引导:截空部分文件 lib_project_code_runs_selftest.py 抛 PartFileUnreadable、点名那一份」
  LIB_PROJECT_CODE_RUNS_BREAK=part-probe-copied-files-annotated  自证核的语法树里模块顶层多一句带注解的再赋值（PART_PROBE_COPIED_FILES: tuple = PART_PROBE_COPIED_FILES + …）
      「引导：拷部分文件那一圈只照 PART_PROBE_COPIED_FILES 迭代、往探针目录拷的只有这一圈，它在模块里只绑定一次（不另 +=、不带注解再赋、不在函数里 global 再赋）、是字面串的元组（算键库折得出拷的是哪几份）」
  LIB_PROJECT_CODE_RUNS_BREAK=part-probe-copied-files-augmented  自证核的语法树里模块顶层多一句 PART_PROBE_COPIED_FILES += …
      「引导：拷部分文件那一圈只照 PART_PROBE_COPIED_FILES 迭代、往探针目录拷的只有这一圈，它在模块里只绑定一次（不另 +=、不带注解再赋、不在函数里 global 再赋）、是字面串的元组（算键库折得出拷的是哪几份）」
  LIB_PROJECT_CODE_RUNS_BREAK=part-probe-copied-files-global-rebound  自证核的语法树里多一个函数，global PART_PROBE_COPIED_FILES 之后再赋它
      「引导：拷部分文件那一圈只照 PART_PROBE_COPIED_FILES 迭代、往探针目录拷的只有这一圈，它在模块里只绑定一次（不另 +=、不带注解再赋、不在函数里 global 再赋）、是字面串的元组（算键库折得出拷的是哪几份）」
  LIB_PROJECT_CODE_RUNS_BREAK=part-probe-copied-files-tuple-added  自证核的语法树里 PART_PROBE_COPIED_FILES 的值换成字面元组加 PROJECT_CODE_RUNS_PART_NAMES（不是全由字面串组成的元组）
      「引导：拷部分文件那一圈只照 PART_PROBE_COPIED_FILES 迭代、往探针目录拷的只有这一圈，它在模块里只绑定一次（不另 +=、不带注解再赋、不在函数里 global 再赋）、是字面串的元组（算键库折得出拷的是哪几份）」
  LIB_PROJECT_CODE_RUNS_BREAK=part-probe-copy-beside-loop  自证核的语法树里那一圈之外多一句推导式，另往 part_probe_directory 拷一份
      「引导：拷部分文件那一圈只照 PART_PROBE_COPIED_FILES 迭代、往探针目录拷的只有这一圈，它在模块里只绑定一次（不另 +=、不带注解再赋、不在函数里 global 再赋）、是字面串的元组（算键库折得出拷的是哪几份）」
  LIB_PROJECT_CODE_RUNS_BREAK=part-probe-copy-loop-tuple-added  自证核的语法树里往 part_probe_directory 拷部分文件那一圈的迭代对象换成字面元组加 PROJECT_CODE_RUNS_PART_NAMES
      「引导：拷部分文件那一圈只照 PART_PROBE_COPIED_FILES 迭代、往探针目录拷的只有这一圈，它在模块里只绑定一次（不另 +=、不带注解再赋、不在函数里 global 再赋）、是字面串的元组（算键库折得出拷的是哪几份）」
  LIB_PROJECT_CODE_RUNS_BREAK=part-unreadable-passes  读不到或编不过部分文件时照原样抛读文件错与语法错（不换成 PartFileUnreadable、不点名「部分文件」）
      「引导:读不到部分文件 lib_project_code_runs_base.py 抛 PartFileUnreadable、点名那一份」「引导:编不过部分文件 lib_project_code_runs_base.py 抛 PartFileUnreadable、点名那一份」
      「引导:读不到部分文件 lib_project_code_runs_launchers.py 抛 PartFileUnreadable、点名那一份」「引导:编不过部分文件 lib_project_code_runs_launchers.py 抛 PartFileUnreadable、点名那一份」
      「引导:读不到部分文件 lib_project_code_runs_selftest.py 抛 PartFileUnreadable、点名那一份」「引导:编不过部分文件 lib_project_code_runs_selftest.py 抛 PartFileUnreadable、点名那一份」
  LIB_PROJECT_CODE_RUNS_BREAK=pattern-caret-negation-literal  [^…] 不换成 [!…]
      「通配：tools/herd[^x] 的 [^…] 照 bash 当取反」
  LIB_PROJECT_CODE_RUNS_BREAK=pattern-command-words-literal  命令词带通配的只照字面判、不展开
      「通配：tools/her?7 展开成 tools/herd7」「通配：tools/herd* 展开成 herd7 与 herd7-helper，其中有 herd7」
      「展开：通配起本仓的脚本经 capped.sh（scripts/hosts/capped.s? 4 true）」「通配：tools/herd[^x] 的 [^…] 照 bash 当取反」
      「通配：POSIX 字符类 tools/her[[:alpha:]]7 交给 bash 展开」「通配：展开出来的是 nice 包装，包装里的 herd7 照认」
      「bash 起不来时 POSIX 字符类起外部程序照拒」
  LIB_PROJECT_CODE_RUNS_BREAK=pattern-expansions-bare  通配展开出来的那条直接 classify，不再切词剥包装
      「通配：展开出来的是 nice 包装，包装里的 herd7 照认」「展开：通配起本仓的脚本经 capped.sh（scripts/hosts/capped.s? 4 true）」
  LIB_PROJECT_CODE_RUNS_BREAK=posix-brackets-by-python-glob  带 `[:`、`[=`、`[.` 的词照旧交给 python glob（POSIX 字符类、等价类、整理符号认不出，零命中照字面判）
      「通配：POSIX 字符类 tools/her[[:alpha:]]7 交给 bash 展开」「bash 起不来时 POSIX 字符类起外部程序照拒」
  LIB_PROJECT_CODE_RUNS_BREAK=posix-brackets-unresolved-literal  bash 展不开或判不出时照字面判（与零命中一样），不拒
      「bash 起不来时 POSIX 字符类起外部程序照拒」
  LIB_PROJECT_CODE_RUNS_BREAK=process-substitution-sources-skipped  shell_source_lines() 里参数是进程替换 <(…) 的 source 行不交回（静默跳过）
      「source 的写法：source <(…) 交回、位置交 None（进认不出）」「source 的写法：&& 之后的 . <(…) 交回、位置交 None（进认不出）」
      「同族：认不出的 source 写法（source <(…)）没登记，逐字节相同的门禁阶段拷贝照不同判（读正文）」
  LIB_PROJECT_CODE_RUNS_BREAK=project-scripts-unjudged  project_code_use 一律交 None（本仓的脚本都不当项目代码）
      「本仓代码：」「名单：」
  LIB_PROJECT_CODE_RUNS_BREAK=prune-server-directory-unrecognized  自证认的目录里去掉 prune-server（selftest_directories；登记表里 prune-server/control 那一行算写坏的行，它的 --selftest 不当自证）
      「样本登记表里写坏的几行逐条报出来（…）」「仓里的登记表（…）没有写坏的行、指到的文件在 … 里都在」「样本表：prune-server 那一层登记过的单独 --selftest 认」
  LIB_PROJECT_CODE_RUNS_BREAK=quoted-braces-expanded  引号里的花括号也照展
      「花括号：命令词位置单引号 {herd7,x.litmus} 照字面放行」「花括号：命令词位置双引号 {herd7,x.litmus} 照字面放行」
  LIB_PROJECT_CODE_RUNS_BREAK=quoted-shell-sources-skipped  shell_source_lines() 不找引号里交给 eval 或子 shell 的代码起头的 source（SHELL_QUOTED_SOURCE 不用）
      「source 的写法：eval 串里起头的 source 交回、位置交 None（进认不出）」「source 的写法：bash -c 串里起头的 source 交回、位置交 None（进认不出）」
      「同族：认不出的 source 写法（eval 串）没登记，逐字节相同的门禁阶段拷贝照不同判（读正文）」「同族：认不出的 source 写法（bash -c 串）没登记，逐字节相同的门禁阶段拷贝照不同判（读正文）」
      「同族：三张表里的脚本与它们引的库里，认不出写法的 source 行都登记在 OWN_SCRIPT_UNRECOGNIZED_SOURCES 里、登记的每一条都还在」
      「同族：三张表里的脚本与 OWN_SCRIPT_LIBRARY_FILES 里的 shell 库，命令位置上与 eval、-c 串里起头的 source / . 每一行都认出了或登记过（不经识别函数逐行数）」
  LIB_PROJECT_CODE_RUNS_BREAK=run-build-directory-opaque  command_under_run_build_directory 一律交 None，run-build-directory.sh <命令> 不剥（里面那条看不见）
      「run-build-directory.sh 这一趟自己的编译目录的包装」
  LIB_PROJECT_CODE_RUNS_BREAK=same-family-by-path-tail  同族脚本退回只按路径尾部认
      「同族：别处树里的门禁阶段拷贝不按名字判（读正文）」「同族：别处树里的 mutate.sh 拷贝不按名字判（读正文）」「路径尾部相同的别处 crash-point-replay.sh 拷贝不按名字判（读正文）」
      「同族：source 的库与本仓那一份不同的门禁阶段拷贝不按名字判（读正文）」
      「同族：.claude/gate.d/doc-text.sh 经变量 source 的 .claude/gate.d/lib/stage-cells.sh 与本仓那一份不同的拷贝不按名字判（读正文）」
      「同族：.claude/gate.d/doc-text.sh 经变量 source 的 .claude/gate.d/lib/doc-text-content-and-history-cells.sh 与本仓那一份不同的拷贝不按名字判（读正文）」
      「同族：.claude/gate.d/code-tooling.sh 经变量 source 的 .claude/gate.d/lib/code-tooling-selftest-cells.sh 与本仓那一份不同的拷贝不按名字判（读正文）」
      「同族：先加 source 行、库还没建：本仓那一份引的库盘上没有，逐字节相同的拷贝不按名字判（读正文）」
      「同族：认不出的 source 写法（source <(…)）没登记，逐字节相同的门禁阶段拷贝照不同判（读正文）」「同族：认不出的 source 写法（eval 串）没登记，逐字节相同的门禁阶段拷贝照不同判（读正文）」
      「同族：认不出的 source 写法（bash -c 串）没登记，逐字节相同的门禁阶段拷贝照不同判（读正文）」
      「同族：登记按行认：同一份里多出一条照写同一个参数的 source，不被登记过的那一条吸收，进认不出、逐字节相同的门禁阶段拷贝照不同判」
      「同族：库往下读撞上深度上限、有一份没读完的，逐字节相同的门禁阶段拷贝照不同判（读正文）」
  LIB_PROJECT_CODE_RUNS_BREAK=script-body-launches-unjudged  script_body_unresolved_launch() 一律交 None（脚本正文里经变量起的自证、门禁阶段、入口脚本、cargo 看不见）：
      「脚本正文：数组变量的定义里写着 --selftest，照认不出」「脚本正文：cargo test 的参数是解不出的变量，照认不出」
      「脚本正文："$@" 转发、起它的参数里写着 --selftest，照认不出」
  LIB_PROJECT_CODE_RUNS_BREAK=script-directory-variables-skipped  shell_source_lines() 不认变量与顶层 cd，认不出的 source 行静默跳过（只剩按自己所在目录的写法直接接字面路径的）
      「source 的库：经变量与顶层 cd 引的几种写法逐种认出，赋值不一致的、认不出的变量交 None（shell_source_lines）」
      「source 的库：库里看得到引它的入口设的变量，认不出又没登记的那一行交回、不静默跳过（script_source_reading）」
      「同族：.claude/gate.d/doc-text.sh 经变量 source 的 .claude/gate.d/lib/stage-cells.sh 与本仓那一份不同的拷贝不按名字判（读正文）」
      「同族：.claude/gate.d/doc-text.sh 经变量 source 的 .claude/gate.d/lib/doc-text-content-and-history-cells.sh 与本仓那一份不同的拷贝不按名字判（读正文）」
      「同族：.claude/gate.d/code-tooling.sh 经变量 source 的 .claude/gate.d/lib/code-tooling-selftest-cells.sh 与本仓那一份不同的拷贝不按名字判（读正文）」
      「派发起法：scripts/hosts/multi-host-run.sh 经变量 source 的 scripts/hosts/peer-host-lib.sh 与本仓那一份不同的拷贝剥开的不算经内存包装」
      「同族：OWN_SCRIPT_LIBRARY_FILES 与三张表里每份脚本引的库逐个相同，表里每一份都在本仓盘上，每份脚本引的库都在深度上限之内读完」
      「同族：三张表里的脚本与它们引的库里，认不出写法的 source 行都登记在 OWN_SCRIPT_UNRECOGNIZED_SOURCES 里、登记的每一条都还在」
      「同族：三张表里的脚本与 OWN_SCRIPT_LIBRARY_FILES 里的 shell 库，命令位置上与 eval、-c 串里起头的 source / . 每一行都认出了或登记过（不经识别函数逐行数）」
      「source 的写法：」（几种写法各一格，九格都红：认得出的五格交回的是空表，认不出的四格那一行不交回）
      「同族：认不出的 source 写法（source <(…)）没登记，逐字节相同的门禁阶段拷贝照不同判（读正文）」「同族：认不出的 source 写法（eval 串）没登记，逐字节相同的门禁阶段拷贝照不同判（读正文）」
      「同族：认不出的 source 写法（bash -c 串）没登记，逐字节相同的门禁阶段拷贝照不同判（读正文）」
      「同族：登记按行认：同一份里多出一条照写同一个参数的 source，不被登记过的那一条吸收，进认不出、逐字节相同的门禁阶段拷贝照不同判」
  LIB_PROJECT_CODE_RUNS_BREAK=selftest-extra-arguments-allowed  带了 --selftest 之外参数的也当单独一个 --selftest
      「样本表：带了 --selftest 之外参数的不认」「样本表：门禁共用库目录里的库带 --selftest 之外的参数不认」
  LIB_PROJECT_CODE_RUNS_BREAK=selftest-pattern-words-judged  命令词带通配的也当自证判
      「样本表：命令词带通配的不当登记过的自证」
  LIB_PROJECT_CODE_RUNS_BREAK=selftest-registry-by-path-tail  登记表只按路径尾部认，别的树里的同名脚本也算登记过
      「样本表：别的树里路径尾部相同的脚本不算登记」「样本表：别的树里装置那一层路径尾部相同的不认」「要临时服务：别的树里路径尾部相同的不拒」
  LIB_PROJECT_CODE_RUNS_BREAK=selftest-registry-lenient  登记表里写坏的行照收、不报
      「样本登记表里写坏的几行逐条报出来（种类、理由、目录、重复、文件不在、门禁阶段那几行的种类与假工具目录）」
      「样本表：种类写坏的一行不算登记」「样本表：理由太短的一行不算登记」
  LIB_PROJECT_CODE_RUNS_BREAK=source-prefixes-unseen  shell_source_command() 不认 source 前面垫的命令前赋值与 builtin / command
      「source 的写法：命令前垫赋值（QUIET=1 source）认出」「source 的写法：builtin source 认出」「source 的写法：command . 认出」
  LIB_PROJECT_CODE_RUNS_BREAK=sourced-libraries-unread  不换脚本所在目录的写法
      「source 的库：$(dirname "${BASH_SOURCE[0]}") 换成脚本所在目录」
  LIB_PROJECT_CODE_RUNS_BREAK=stage-directory-pwd-unexpanded  阶段目录写成 $PWD/… 时不换成当前目录
      「门禁样本：阶段目录写成 $PWD/…、--item 点名一道登记过的」
  LIB_PROJECT_CODE_RUNS_BREAK=stage-feed-script-by-name  不带 --item 时只按名字认 stage-selftest.sh，别处树里的拷贝也信
      「门禁样本：别处树里的 stage-selftest.sh 拷贝不带 --item」「门禁样本：相对着写、当前目录不在本仓的 stage-selftest.sh 不带 --item」
  LIB_PROJECT_CODE_RUNS_BREAK=stage-items-as-directory  --item / --list-items 不认，当阶段目录判
      「门禁样本：--item 点名一道登记过的」「门禁样本：--list-items 只列、不喂样本」「门禁样本：--item 点名不存在的阶段（它自己退 2、一份样本都不喂）」
      「门禁样本：阶段目录写成 $PWD/…、--item 点名一道登记过的」「门禁样本：--item 与值之间夹的 --force 照 preflight 摘掉」
      「门禁样本：--item 点名里夹登记表外的，说明点名那一道、不点没毛病的那一道」
  LIB_PROJECT_CODE_RUNS_BREAK=stage-items-script-by-name  带 --item 时只按名字认 stage-selftest.sh，别处树里的拷贝也信
      「门禁样本：别处树里的 stage-selftest.sh 拷贝带 --list-items」「门禁样本：别处树里的 stage-selftest.sh 拷贝 --item 点名一道」
      「门禁样本：别处树里的 stage-selftest.sh 拷贝不带 --item」「门禁样本：相对着写、当前目录不在本仓的 stage-selftest.sh 不带 --item」
  LIB_PROJECT_CODE_RUNS_BREAK=stage-items-unjudged  --item 点名的几道不判
      「门禁样本：--item 点名里夹一道登记表外的」「门禁样本：--item 点名的样本多一份 setup.sh」「门禁样本：--item 一个值里带空格点到登记表外的」
      「门禁样本：--item 点名写在变量里」「门禁样本：--item 点名里夹登记表外的，说明点名那一道、不点没毛病的那一道」
  LIB_PROJECT_CODE_RUNS_BREAK=stage-samples-fixtures-unchecked  喂的样本不与暂存区比
      「门禁样本：样本：喂的样本多一份 setup.sh」「门禁样本：样本：内容改过」「门禁样本：样本：执行位改过」「门禁样本：样本：少了文件」
      「门禁样本：样本：暂存之前新加的样本（样本仓自己的阶段目录）」「门禁样本：--item 点名的样本多一份 setup.sh」「门禁样本：样本与暂存区不同的说明点名那份文件」
  LIB_PROJECT_CODE_RUNS_BREAK=stage-samples-ignored  门禁样本一律不认
      「门禁样本：样本档全登记、与暂存区相同的阶段目录」「门禁样本：--item 点名一道登记过的」「门禁样本：--list-items 只列、不喂样本」「门禁样本：--item 点名不存在的阶段（它自己退 2、一份样本都不喂）」「门禁样本：阶段目录写成 $PWD/…、--item 点名一道登记过的」「门禁样本：--item 与值之间夹的 --force 照 preflight 摘掉」「门禁样本：样本与暂存区不同的说明点名那份文件」「门禁样本：--item 点名里夹登记表外的，说明点名那一道、不点没毛病的那一道」
  LIB_PROJECT_CODE_RUNS_BREAK=strace-drops-env  strace -E / --env 设的变量不带进里面那条命令
      「启动器环境：strace -E 设别名」「启动器环境：strace --env= 设别名」「启动器环境：strace --env 值另起一个词设别名」「启动器环境：strace -fE 合写设别名」
  LIB_PROJECT_CODE_RUNS_BREAK=task-server-builds-unjudged  两份判法都不认直接删服务的编译目录
      「绕开：python 代码 shutil.rmtree 服务的编译目录」「绕开：rm -rf 服务的编译目录」「绕开：rm 写 ~ 起头的编译目录」「绕开：rm 状态目录下的 builds」「绕开：find -delete 编译目录」「绕开：find -exec rm 编译目录」「绕开：rm -rf 编译目录的上级」
  LIB_PROJECT_CODE_RUNS_BREAK=task-server-directory-unrecognized  自证认的目录里去掉 task-server（selftest_directories；登记表里那一层的行算写坏的行，那一层的 --selftest 不当自证）
      「样本登记表里写坏的几行逐条报出来（…）」「仓里的登记表（…）没有写坏的行、指到的文件在 … 里都在」「样本表：task-server 那一层登记过的单独 --selftest 认」
  LIB_PROJECT_CODE_RUNS_BREAK=task-server-library-load-words-file-calls-only  task_server_library_named_with_load() 的载入写法只认按文件载入的那几种调用（TASK_SERVER_LIBRARY_FILE_LOAD_CALLS；import、exec、compile、run_module 不认）
      「绕开：python 代码载入 task_server_lib」「绕开：python 代码按文件载入 task_server_lib.py（exec(open(…))）」「绕开：python 代码先 open().read() 存进变量再 exec」
      「绕开：喂给 python 的 heredoc 里先 compile 再 exec」「绕开：喂给 python 的 heredoc 里 exec( 与 open( 拆在两行」「绕开：python 代码 import 一串里逗号后面是 task_server_lib」
      「绕开：python 代码 runpy.run_module 按模块名起 task_server_lib」「绕开：写进 .py 文件的 heredoc 正文里按文件载入照拒」
  LIB_PROJECT_CODE_RUNS_BREAK=task-server-library-loads-module-name-unseen  task_server_library_named_with_load() 的库名只认文件名 task_server_lib.py（模块名 task_server_lib 不认）
      「绕开：python 代码载入 task_server_lib」「绕开：python 代码 import 一串里逗号后面是 task_server_lib」「绕开：python 代码 runpy.run_module 按模块名起 task_server_lib」
      「绕开：python 代码先拼出路径再 spec_from_file_location」
  LIB_PROJECT_CODE_RUNS_BREAK=task-server-library-loads-same-statement  task_server_library_named_with_load() 要库名与载入写法在同一句里（按分号与换行切开再判）
      「绕开：python 代码先 open().read() 存进变量再 exec」「绕开：喂给 python 的 heredoc 里 exec( 与 open( 拆在两行」「绕开：python 代码先拼出路径再 spec_from_file_location」
  LIB_PROJECT_CODE_RUNS_BREAK=task-server-library-loads-unjudged  task_server_text_bypass 不认载入共用库（task_server_library_loaded 不调）
      「绕开：python 代码载入 task_server_lib」「绕开：python 代码按文件载入 task_server_lib.py（spec_from_file_location）」「绕开：python 代码按文件载入 task_server_lib.py（runpy.run_path）」
      「绕开：python 代码按文件载入 task_server_lib.py（exec(open(…))）」「绕开：python 代码先 open().read() 存进变量再 exec」「绕开：喂给 python 的 heredoc 里先 compile 再 exec」
      「绕开：喂给 python 的 heredoc 里 exec( 与 open( 拆在两行」「绕开：python 代码 import 一串里逗号后面是 task_server_lib」「绕开：python 代码 runpy.run_module 按模块名起 task_server_lib」
      「绕开：python 代码先拼出路径再 spec_from_file_location」「绕开：写进 .py 文件的 heredoc 正文里按文件载入照拒」
      「绕开：python 从标准输入读的程序是管道前一段 cat 出来的 task_server_lib.py」
  LIB_PROJECT_CODE_RUNS_BREAK=task-server-library-loads-whole-text  task_server_library_code_texts() 不切命令、整段文本当一段判（只读命令里写的也算，python 从标准输入读这份库那一样不认）
      「绕开：只读的 grep 搜载入调用名（带括号）不算」「绕开：只读的 grep -nF 抄自证里那一行样本不算」「绕开：只读的 sed -n 按载入调用名找行不算」
      「绕开：写进 .md 文件的 heredoc 正文里写到载入调用与文件名不算」「绕开：python 从标准输入读的程序是管道前一段 cat 出来的 task_server_lib.py」
  LIB_PROJECT_CODE_RUNS_BREAK=task-server-library-loads-written-python-unjudged  task_server_library_code_texts() 不判写进 .py 文件的 heredoc 正文
      「绕开：写进 .py 文件的 heredoc 正文里按文件载入照拒」
  LIB_PROJECT_CODE_RUNS_BREAK=task-server-library-stdin-program-unjudged  task_server_library_code_texts() 不认 python 从标准输入读这份库（< 文件、管道前一段只读命令读的）
      「绕开：python 从标准输入读的程序是管道前一段 cat 出来的 task_server_lib.py」
  LIB_PROJECT_CODE_RUNS_BREAK=task-server-paths-unjudged  task_server_text_bypass 不认库、密钥、socket
      「绕开：点到库文件名」「绕开：点到签名密钥文件名」「绕开：点到 socket」
  LIB_PROJECT_CODE_RUNS_BREAK=task-server-scripts-unjudged  task_server_command_bypass 不认直接起服务本体与每次尝试的那一层
      「绕开：直接起服务本体」
  LIB_PROJECT_CODE_RUNS_BREAK=task-server-submission-unparsed  task_server_submission 一律交 None
      「提交：submit -- 后面那条」「提交：--cwd 照这一条的目录解开」「提交：仓副本里逐字节相同的客户端」「提交：--manual 不带值，-- 后面那条照认」「提交：--full-gate 不带值，-- 后面那条照认」「提交：--whole-package-confirmed 带一个值，-- 后面那条照认」「提交：几个选项混写，--cwd 照解开」
  LIB_PROJECT_CODE_RUNS_BREAK=task-server-submit-flags-unparsed  task_server_submission 只认 --cwd（带 --manual、--full-gate、--whole-package-confirmed 的提交一律交 None，-- 后面那条不判）
      「提交：--manual 不带值，-- 后面那条照认」「提交：--full-gate 不带值，-- 后面那条照认」「提交：--whole-package-confirmed 带一个值，-- 后面那条照认」「提交：几个选项混写，--cwd 照解开」
  LIB_PROJECT_CODE_RUNS_BREAK=task-server-units-unjudged  task_server_command_bypass 不认 systemctl 停禁改它的单元
      「绕开：systemctl --user stop 服务单元」「绕开：systemctl --user kill 一次尝试的单元」「绕开：systemctl --user stop 通配」
  LIB_PROJECT_CODE_RUNS_BREAK=temporary-service-bare-unjudged  temporary_service_bare_selftest() 一律交 None（要临时服务的自证写成裸 --selftest 也不拦）：
      「要临时服务：required 行的裸 --selftest 交整条包装写法」「要临时服务：仓里 crates-mutation-rows.py 的裸 --selftest 被点名、给整条包装写法」
  LIB_PROJECT_CODE_RUNS_BREAK=temporary-service-command-bare  wrap_in_temporary_service() 给裸写的样子（拒绝说明里没有包装）：
      「要临时服务：required 行的裸 --selftest 交整条包装写法」「要临时服务：python 脚本的整条写法起头是 python3」「要临时服务：仓里 crates-mutation-rows.py 的裸 --selftest 被点名、给整条包装写法」
      「要临时服务：样本档点名要临时服务的一道、外面没套：说明点名那一道（不点不要的那一道）、给整条包装写法」「要临时服务：仓里门禁 crates-mutation-replay.sh 的样本档裸写被点名、给整条包装写法」
  LIB_PROJECT_CODE_RUNS_BREAK=temporary-service-command-binary-placeholder  temporary_service_command() 的整条写法带 --server-binary 与尖括号占位（照抄进 shell 时尖括号被当成输入重定向）：
      「要临时服务：required 行的裸 --selftest 交整条包装写法」「要临时服务：python 脚本的整条写法起头是 python3」「要临时服务：整条写法不带 --server-binary 与尖括号占位」「要临时服务：裸 --selftest 的拒绝说明里没有尖括号占位」「要临时服务：仓里 crates-mutation-rows.py 的裸 --selftest 被点名、给整条包装写法」「要临时服务：样本档点名要临时服务的一道、外面没套：说明点名那一道（不点不要的那一道）、给整条包装写法」「要临时服务：仓里门禁 crates-mutation-replay.sh 的样本档裸写被点名、给整条包装写法」
  LIB_PROJECT_CODE_RUNS_BREAK=temporary-service-environment-dropped  environment_under_temporary_service() 原样交回（包装里的原命令看起来没有标记）：
      「要临时服务：包装剥开重判时补上的标记让里面那条放行」「要临时服务：仓里 crates-mutation-rows.py 的 --selftest 在包装底下放行」「要临时服务：样本档点名的一道要临时服务、包装剥开重判补上了标记：放行」
  LIB_PROJECT_CODE_RUNS_BREAK=temporary-service-family-absolute-paths  source_family() 里命名那一半不把绝对路径换成仓内相对路径（门禁阶段入口给绝对路径时，lib/ 下 source 的部分文件认不出）：
      「要临时服务：拆分族：门禁阶段入口给绝对路径时 lib/ 下 source 的部分文件在族里」
  LIB_PROJECT_CODE_RUNS_BREAK=temporary-service-family-by-directory  source_family() 把同目录的每一份 .py 与 .sh 都算进族（不看命名与载入）：
      「要临时服务：样本表写坏的、两边对不上的几行逐条报出来（判定、理由、键重复、文件不在、没登记、源码里没有核对、漏登）」「要临时服务：拆分族里哪一份都没有记号的 required 行照旧报源码里没有核对」「要临时服务：拆分族：python 入口连同它 import 的同目录模块」「要临时服务：拆分族：shell 入口连同它 source 的同目录库」「要临时服务：拆分族：别的目录里的模块不在族里」「要临时服务：拆分族：名字像部分文件而入口没载入的同目录文件不在族里，记号只在它里面的 required 行照旧报源码里没有核对」「要临时服务：拆分族：入口 import 的共用库与字符串里点名的别的脚本不在族里，记号只在它们里面的 required 行照旧报源码里没有核对」「要临时服务：拆分族：入口注释里点名、文档串里写 import 行的同目录文件不在族里」「要临时服务：拆分族：入口按字面量参数的 importlib.import_module 载入的同目录部分文件在族里」
  LIB_PROJECT_CODE_RUNS_BREAK=temporary-service-family-by-name  source_family() 只按名字认族：命名族里的同目录文件都算，不看入口载没载入（名字像部分文件而入口没载入的也算进族；shell 入口的命名族也含同名主干的 .py）：
      「要临时服务：样本表写坏的、两边对不上的几行逐条报出来（判定、理由、键重复、文件不在、没登记、源码里没有核对、漏登）」「要临时服务：拆分族：shell 入口连同它 source 的同目录库」「要临时服务：拆分族：名字像部分文件而入口没载入的同目录文件不在族里，记号只在它里面的 required 行照旧报源码里没有核对」「要临时服务：拆分族：入口注释里点名、文档串里写 import 行的同目录文件不在族里」
  LIB_PROJECT_CODE_RUNS_BREAK=temporary-service-family-by-loading  source_family() 只按载入认族：入口或族里别的文件载入的同目录文件都算，不看名字像不像部分文件（共用库、字符串里点名的别的脚本也算进族）：
      「要临时服务：样本表写坏的、两边对不上的几行逐条报出来（判定、理由、键重复、文件不在、没登记、源码里没有核对、漏登）」「要临时服务：拆分族：入口 import 的共用库与字符串里点名的别的脚本不在族里，记号只在它们里面的 required 行照旧报源码里没有核对」
  LIB_PROJECT_CODE_RUNS_BREAK=temporary-service-family-by-text  source_family() 认族的载入按文本行扫：注释、文档串里写的文件名与 import 行都算载入，importlib.import_module 认不出：
      「要临时服务：拆分族：入口注释里点名、文档串里写 import 行的同目录文件不在族里」「要临时服务：拆分族：入口按字面量参数的 importlib.import_module 载入的同目录部分文件在族里」
  LIB_PROJECT_CODE_RUNS_BREAK=temporary-service-mark-entry-only  source_family() 只交入口自己（记号挪进拆出的同目录文件的 required 行读成「源码里没有核对」）：
      「要临时服务：样本表写坏的、两边对不上的几行逐条报出来（判定、理由、键重复、文件不在、没登记、源码里没有核对、漏登）」「要临时服务：拆分族里记号挪进同目录部分文件的 required 行不报」「要临时服务：拆分族：python 入口连同它 import 的同目录模块」「要临时服务：拆分族：shell 入口连同它 source 的同目录库」「要临时服务：拆分族：入口按字面量参数的 importlib.import_module 载入的同目录部分文件在族里」「要临时服务：拆分族：门禁阶段入口给绝对路径时 lib/ 下 source 的部分文件在族里」「要临时服务：仓里的表」
  LIB_PROJECT_CODE_RUNS_BREAK=temporary-service-marker-ignored  不看环境里的临时服务标记（套了包装、自己设了标记的也拦）：
      「要临时服务：命令前设了临时服务的标记，裸 --selftest 放行」「要临时服务：包装剥开重判时补上的标记让里面那条放行」「要临时服务：仓里 crates-mutation-rows.py 的 --selftest 在包装底下放行」「要临时服务：样本档点名的一道要临时服务、包装剥开重判补上了标记：放行」
  LIB_PROJECT_CODE_RUNS_BREAK=temporary-service-own-server-as-required  own-server 的行也当 required（自己起服务的自证被要求套临时服务）：
      「要临时服务：own-server 行的裸 --selftest 不拒」
  LIB_PROJECT_CODE_RUNS_BREAK=temporary-service-stage-samples-every-stage  temporary_service_bare_stage_samples() 点名的每一道样本档都当要临时服务（不读阶段源码）：
      「要临时服务：样本档点名要临时服务的一道、外面没套：说明点名那一道（不点不要的那一道）、给整条包装写法」「要临时服务：样本档只点名不要临时服务的一道：不拒」「要临时服务：仓里门禁 doc-text.sh 的样本档不要临时服务：不拒」
  LIB_PROJECT_CODE_RUNS_BREAK=temporary-service-stage-samples-unjudged  temporary_service_bare_stage_samples() 一律交 None（样本档裸写也不拦）：
      「要临时服务：样本档点名要临时服务的一道、外面没套：说明点名那一道（不点不要的那一道）、给整条包装写法」「要临时服务：仓里门禁 crates-mutation-replay.sh 的样本档裸写被点名、给整条包装写法」
  LIB_PROJECT_CODE_RUNS_BREAK=temporary-service-table-lenient  要临时服务的表写坏的行照收、不报（判定认不出、理由太短、键重复）：
      「要临时服务：理由太短的一行不算登记」「要临时服务：样本表写坏的、两边对不上的几行逐条报出来（判定、理由、键重复、文件不在、没登记、源码里没有核对、漏登）」
  LIB_PROJECT_CODE_RUNS_BREAK=temporary-service-wrapper-unseen  temporary_service_wrapped() 一律交 None（临时 prune-server 的包装 `prune_server_client_selftest.py --with-temporary-service … -- <原命令>` 剥不开，里面那条看不见）
      「临时 prune-server 的包装给了 --server-binary 剥开里面那条」
  LIB_PROJECT_CODE_RUNS_BREAK=text-tools-refused  TEXT_TOOLS_RUN_DIRECTLY 不认（名单上的文本工具也要经 task-server）
      「名单：replace-once.py 定点改」「名单：仓副本里逐字节相同的 replace-once.py」「名单：认领草稿目录」「名单：gate-overlap.py --list」「名单：提交客户端的 wait」「名单：task-server/control check」「名单：改标签的 relabel-item.py」「名单：备份原文件的 copy-originals.py」
      「名单：建仓副本的 minimal-copy.py」「名单：打副本闭包的 lib_repository_copy.py entries」「展开：通配起文本工具 scripts/kb/replace-onc?.py 不拒（谁都能直接跑）」
      「名单：提交客户端的 clean」「名单：task-server/control bookings」「名单：task-server/control key-stats」「名单：直接起 ledger-append-section.py 不拒」
      「名单：直接起 append-record.py 不拒」「名单：直接起 quote-rust-items.py 不拒」「名单：确认单打的 task confirm 主 agent 直接起」
  LIB_PROJECT_CODE_RUNS_BREAK=two-logs-opaque  command_under_two_logs 一律交 None，two-logs.sh <名> -- <命令> 不剥（里面那条看不见）
      「two-logs.sh 日志出两份的包装」「two-logs.sh --quiet 日志出两份的包装」
  LIB_PROJECT_CODE_RUNS_BREAK=unrecognized-launches-allowed  unrecognized_launch() 一律交 None
      「认不出：」「脚本正文：cargo test 的参数是解不出的变量，照认不出」
  LIB_PROJECT_CODE_RUNS_BREAK=unrecognized-sources-registered-by-argument  registered_source_key() 不看第几条：同一份文件、同一个参数的头一个登记就算（多出来的那一条被登记过的吸收）
      「同族：登记按行认：同一份里多出一条照写同一个参数的 source，不被登记过的那一条吸收，进认不出、逐字节相同的门禁阶段拷贝照不同判」
      「同族：三张表里的脚本与它们引的库里，认不出写法的 source 行都登记在 OWN_SCRIPT_UNRECOGNIZED_SOURCES 里、登记的每一条都还在」
  LIB_PROJECT_CODE_RUNS_BREAK=unrecognized-sources-skipped  script_source_reading() 认不出又没登记的 source 行不交回（静默跳过，同族拷贝不因它读正文）
      「source 的库：库里看得到引它的入口设的变量，认不出又没登记的那一行交回、不静默跳过（script_source_reading）」
      「同族：认不出的 source 写法（source <(…)）没登记，逐字节相同的门禁阶段拷贝照不同判（读正文）」「同族：认不出的 source 写法（eval 串）没登记，逐字节相同的门禁阶段拷贝照不同判（读正文）」
      「同族：认不出的 source 写法（bash -c 串）没登记，逐字节相同的门禁阶段拷贝照不同判（读正文）」
      「同族：登记按行认：同一份里多出一条照写同一个参数的 source，不被登记过的那一条吸收，进认不出、逐字节相同的门禁阶段拷贝照不同判」
  project-code-guard.sh --selftest 原样转出上面这几格的失败行，另加它自己的格（写在它的文件头）。
"""
import os
import sys

# 同目录的部分模块按这一份所在的目录载入、不留在 sys.modules 的裸名字下：同一个进程里从两个目录各载入一次这一份（调用方按文件路径导入，自证拷一份目录再载入），
# 各用各目录里的部分模块；载入期间这个目录放在 sys.path 最前、裸名字摘出来（部分模块之间的 import 取到这一个目录里的），载入之后 sys.path 与裸名字放回载入之前的样子
_LIBRARY_DIRECTORY = os.path.realpath(os.path.dirname(os.path.abspath(__file__)))
_PART_MODULE_NAMES = ("lib_project_code_runs_bootstrap", "lib_project_code_runs_base", "lib_project_code_runs_launchers", "lib_project_code_runs_selftest",)


def _begin_part_imports():
    """→ 载入之后要放回的现场：摘出来的同名模块。"""
    saved = {name: sys.modules.pop(name) for name in _PART_MODULE_NAMES if name in sys.modules}
    sys.path.insert(0, _LIBRARY_DIRECTORY)
    return saved


def _end_part_imports(saved):
    sys.path.remove(_LIBRARY_DIRECTORY)
    for name in _PART_MODULE_NAMES:
        sys.modules.pop(name, None)
    sys.modules.update(saved)


class PartFileUnreadable(ImportError):
    """这份模块的部分文件读不到、读不了、不是 UTF-8、语法错或含空字节，或编得过但不完整（入口要的名字缺了）：说明点名是哪一份、缺哪个名字。它是 ImportError，按文件路径导入这份模块的调用方照各自缺依赖（模块读不到）时的处置办。"""


def report_part_unusable(part_file_name, part_error):
    """一份部分文件载不全：抛 PartFileUnreadable、说明点名是哪一份，不让导入时抛读文件错与语法错；
    读不到、编不过（OSError、SyntaxError、ValueError、模块读不到）与编得过而不完整（入口要的名字缺了：ImportError、NameError）分开处置，弄坏开关各管一类。
    PROJECT_CODE_RUNS_PART_NAMES 与 break_is_set 是下面先 import 的引导要的名字，调用的时候已经在了。"""
    incomplete = isinstance(part_error, (ImportError, NameError)) and not isinstance(part_error, ModuleNotFoundError)
    if break_is_set("part-incomplete-passes" if incomplete else "part-unreadable-passes"):
        raise part_error   # 弄坏开关 part-unreadable-passes、part-incomplete-passes：照原样抛原来的错
    if incomplete:
        raise PartFileUnreadable(f"lib_project_code_runs.py 的部分文件编得过但不完整，入口要的名字缺了，没有哪一份部分文件定义它（{type(part_error).__name__}：{part_error}）："
                                 + "；".join(f"部分文件 {os.path.join(_LIBRARY_DIRECTORY, name)}" for name in PROJECT_CODE_RUNS_PART_NAMES)) from part_error
    raise PartFileUnreadable(f"lib_project_code_runs.py 读不到或编不过部分文件 {os.path.join(_LIBRARY_DIRECTORY, part_file_name)}（{type(part_error).__name__}：{part_error}）") from part_error


_saved_modules = _begin_part_imports()
# 引导要的名字这一份读不到时没法点名是哪一份，直接抛原来的错（这一份没载进来：调用方整份丢掉这次导入）
from lib_project_code_runs_bootstrap import (
    BREAK_VARIABLE, PROJECT_CODE_RUNS_BOOTSTRAP_FILE_NAME, PROJECT_CODE_RUNS_PART_NAMES, break_is_set,
)
try:
    try:
        from lib_project_code_runs_base import (
            ACTIVE_SESSIONS_SCRIPT, ALIAS_CONFIGURATION, ALIAS_ENVIRONMENT_PREFIX, BASH_BRACKET_PATTERN, BASH_PATTERN_TIMEOUT, BRACE_SCAN_LIMIT,
            BRACE_TOKENS, BROAD_MUTATION_SCRIPT_NAME, BUILT_BINARY_PATH, CARET_NEGATION, CARGO_GLOBAL_OPTIONS_WITH_VALUE, CRASH_REPLAY_AND_HOST_LIBRARY_FILES,
            CARGO_NON_BUILDING_SUBCOMMANDS, CARGO_TEST_OPTIONS_WITH_VALUE, COMPILED_CODE_SUBCOMMANDS, CargoCommand, DEVICE_SELFTEST_PARENT,
            DISPATCH_LAUNCHER_DIRECTORY, DISPATCH_LAUNCHER_OPTIONS, DISPATCH_LAUNCHER_SCRIPTS, E165_PIN_SCRIPT, EXTERNAL_PROGRAM_NAMES, EXTERNAL_PROGRAM_PREFIXES,
            EXTERNAL_PROGRAM_VERSION_ONLY_ARGUMENTS, FIND_EXECUTION_ACTIONS, FIND_LEADING_OPTIONS, GATE_LIBRARY_DIRECTORY, GATE_STAGE_AND_PREFLIGHT_LIBRARY_FILES, GATE_STAGE_DIRECTORY,
            GATE_STAGE_FILES, GATE_STAGE_PATH, GATE_TOP_LIBRARY_PREFIX, GLOB_CHARACTERS, GPU_QUIESCE_DIRECTORY, GPU_QUIESCE_SCRIPT, GPU_VERIFICATION_SERVER_DIRECTORY,
            HACK_INNER_SUBCOMMANDS, HOOK_REPOSITORY, KIND_CATEGORY, KNOWN_SCRIPT_LOCATIONS, LIBTEST_OPTIONS_WITH_VALUE, LIST_ONLY_TEST_ARGUMENT,
            LLVM_COV_NON_RUNNING_SUBCOMMANDS, MAIN_AGENT_TOOLS_RUN_DIRECTLY, MULTI_HOST_RUN_OTHER_MODES, MULTI_HOST_RUN_SCRIPT,
            MUTANTS_QUERY_ARGUMENTS, MUTATION_PYTHON_LIBRARY_FILES, MUTATION_REGRESSION_TESTS_DIRECTORY, OWN_SCRIPT_LIBRARY_DROPPED_BY_BREAK, OWN_SCRIPT_LIBRARY_FILES,
            OWN_SCRIPT_LIBRARY_PHANTOM_BY_BREAK, OWN_SCRIPT_LIBRARY_TABLE_LIMIT, OWN_SCRIPT_UNRECOGNIZED_SOURCES,
            PATTERN_EXPANSION_LIMIT, PROJECT_HEAD_BYTES, PROJECT_SCRIPT_HEADER,
            PROVE_RED_SCRIPT, PRUNE_SERVER_DIRECTORY, PYTHON_REMOVAL_CALL, ProgramUse, ProjectRun, REMOVAL_COMMAND_NAMES,
            RUN_BUILD_DIRECTORY_LOCATION, RUN_BUILD_DIRECTORY_SCRIPT, SCRIPTS_STARTING_BUILDS, SCRIPT_CATEGORY_DIRECTORIES, SELFTEST_DIRECTORIES,
            SELFTEST_KIND, SELFTEST_PATTERN_CHARACTERS, SELFTEST_REASON_MINIMUM, SELFTEST_REGISTRY_NAME, SESSION_CHECKS_DIRECTORY, STAGE_SAMPLES,
            STAGE_SAMPLES_WITH_TOOLS, STAGE_SELFTEST_ITEM_OPTION, STAGE_SELFTEST_LIST_OPTION, STAGE_SELFTEST_SCRIPT, SUBAGENT_HAND_BACK_WAY_OUT,
            SUBAGENT_TASK_SERVER_SUBCOMMANDS, SYSTEMCTL_MANAGER_VERBS, SYSTEMCTL_OPTIONS_WITH_VALUE, SYSTEMCTL_READ_ONLY_VERBS, SelftestRow,
            StageSelftestArguments, TASK_SERVER_CLIENT, TASK_SERVER_CLIENT_SUBCOMMANDS, TASK_SERVER_DIRECTORY, TASK_SERVER_HEREDOC_WRITERS, TASK_SERVER_LIBRARY_FILE_LOAD_CALLS,
            TASK_SERVER_LIBRARY_FILE_NAME, TASK_SERVER_LIBRARY_LOAD_WORDS, TASK_SERVER_LIBRARY_NAME, TASK_SERVER_READ_ONLY_COMMANDS, TASK_SERVER_READ_ONLY_GIT_SUBCOMMANDS,
            TASK_SERVER_LIBRARY_LOADED, TASK_SERVER_LIBRARY_PATH, TASK_SERVER_STATE_BUILDS_NAME, TASK_SERVER_SUBMIT_BARE_OPTIONS,
            TASK_SERVER_SUBMIT_DIRECTORY_OPTION, TASK_SERVER_SUBMIT_VALUE_OPTION, TEMPORARY_SERVICE_BINARY_OPTION, TEMPORARY_SERVICE_OWN_SERVER,
            TEMPORARY_SERVICE_REQUIRED, TEMPORARY_SERVICE_SOURCE_MARK, TEMPORARY_SERVICE_TABLE_NAME, TEMPORARY_SERVICE_VARIABLE,
            TEMPORARY_SERVICE_WRAPPER_DIRECTORY, TEMPORARY_SERVICE_WRAPPER_MODE, TEMPORARY_SERVICE_WRAPPER_SCRIPT, TEST_BINARY_PATH,
            TEST_RUNNER_MODULE_FILES, TEXT_TOOLS_RUN_DIRECTLY, TWO_LOGS_DIRECTORY, TWO_LOGS_SCRIPT, TaskServerGuarded, UNEXPANDED_DEFINITION_DEPTH,
            UNEXPANDED_VARIABLE_NAME, UNRESOLVED_WORD_CHARACTERS, VERUS_PROOF_CHECK_SCRIPT, WORD_EXPANSION_DEPTH_LIMIT, WORD_EXPANSION_LIMIT,
            WORKING_DIRECTORY_VARIABLE, alias_of_subcommand, argument_form_allows, build_directory_removal, cargo_subcommand, checked_cells,
            command_word_expanded, configuration_values_after_subcommand, device_selftest_layer, directly_run_level, expanded_target, file_sha256,
            gate_stage_name, gate_top_library_key, git_blob_name, has_project_header, launched_file, libtest_lists_only, load_sibling_module,
            located_in, own_library_digest, own_script_digest, path_mentioned, program_use, project_code_use, project_relative_path, project_top_directories, read_selftest_registry,
            runs_compiled_code, same_as_own_copy, sample_directory_changes, sample_index_entries, selftest_directories, selftest_registered_file,
            selftest_registry_at, selftest_registry_key, selftest_registry_problems, shell_words, stage_feed_problem, stage_samples_kind_is_valid,
            stage_samples_row, stage_selftest_arguments, stage_selftest_directory, stage_selftest_fed_stages, stage_selftest_is_own_copy,
            stage_selftest_item_problems, stage_selftest_problems, stage_selftest_written_directory, starts_a_build, subagent_task_server_refusal,
            systemctl_unit_touch, task_server_client_subcommand, task_server_command_bypass, task_server_guarded, task_server_library,
            task_server_library_code_texts, task_server_library_loaded, task_server_library_named_with_load, task_server_read_only_command,
            task_server_submission, task_server_text_bypass, test_invocation,
        )
        import lib_project_code_runs_base
    except (ImportError, NameError, OSError, SyntaxError, ValueError) as part_error:
        report_part_unusable("lib_project_code_runs_base.py", part_error)
    try:
        from lib_project_code_runs_launchers import (
            BWRAP_OPTION_ARITY, BraceScanTooLong, CARGO_GLOBAL_FLAGS_WITH_VALUE, CARGO_HARMLESS_VALUE_OPTIONS, CARGO_RUNNING_SUBCOMMANDS,
            CLAUDE_EXECUTABLE_PATH_WORDS, ENVIRONMENT_ASSIGNMENT_WORD, EXTRA_LAUNCHER_NAMES, EXTRA_LAUNCHER_OPTIONS, EXTRA_LAUNCHER_SPECIAL,
            FIND_PATH_WORD, LAUNCHED_FILE_NAMES, LAUNCHER_ENVIRONMENT_OPTIONS, LAUNCHER_LONG_OPTIONS_COMPLETED, LAUNCHER_LONG_OPTIONS_WITHOUT_VALUE,
            LAUNCHER_OPTIONS_WITH_VALUE, LAUNCHER_POSITIONAL_COUNT, MAXIMUM_LIBRARY_SCAN_BYTES, PARALLEL_INPUT_WORD,
            PARALLEL_LONG_OPTIONS_WITH_VALUE, PARALLEL_SHORT_OPTIONS_WITH_VALUE, PERF_GROUPS_WITH_RECORD, PERF_SUBCOMMANDS_RUNNING_A_COMMAND,
            POSITIONAL_REFERENCE, PYTHON_SIBLING_REFERENCE, SCRIPT_BODY_ENTRY_NAMES, SCRIPT_BODY_LAUNCH_SIGNATURES, SCRIPT_DIRECTORY_FORMS,
            SCRIPT_LONG_OPTIONS_WITH_VALUE, SCRIPT_SELF_REFERENCES, SCRIPT_SHORT_OPTIONS_WITH_VALUE, SHELL_ASSIGNMENT, SHELL_CHANGED_DIRECTORY,
            SHELL_COMMAND_POSITION, SHELL_DIRECTORY_OF, SHELL_LOOP_VARIABLE, SHELL_OWN_NAME, SHELL_PATH_HEAD, SHELL_PATH_REST, SHELL_PLAIN_RELATIVE,
            SHELL_QUOTED_SOURCE, SHELL_RELATIVE_ANCHOR, SHELL_SOURCE_CASE_ARM_CHARACTER, SHELL_SOURCE_COMMAND, SHELL_SOURCE_ELIF_KEYWORD, SHELL_SOURCE_POSITION_CHARACTERS,
            SHELL_SOURCE_POSITION_KEYWORDS, SHELL_SOURCE_PREFIXES, SHELL_TOP_LEVEL_CD, ShellPlaces, SourceReading,
            SOURCED_LIBRARY_DEPTH, SOURCED_LIBRARY_LOWERED_DEPTH, SPLIT_FAMILY_LIBRARY_PATH, SSH_SHORT_OPTIONS_WITH_VALUE, SU_LONG_OPTIONS_WITH_VALUE,
            SU_SHORT_OPTIONS_WITH_VALUE, UNREADABLE_ENVIRONMENT_FILE_RUNNER, UNRECOGNIZED_KIND, UNRESOLVED_LAUNCH_MARKS,
            WATCH_LONG_OPTIONS_WITH_VALUE, WATCH_SHORT_OPTIONS_WITH_VALUE, XARGS_INPUT_WORD, XARGS_LONG_OPTIONS_WITH_VALUE,
            XARGS_SHORT_OPTIONS_WITH_VALUE, assignments_of_environment_property, bash_pattern_expansions, brace_expansion,
            brace_expansion_with_empties, brace_gobble, cargo_arguments_expanded, classify, command_under_dispatch_launcher,
            command_under_extra_launcher, command_under_gpu_quiesce, command_under_launcher, command_under_run_build_directory,
            command_under_temporary_service, command_under_two_logs, command_word_expansions, dispatched_command, dispatched_under_memory_cap,
            environment_under_temporary_service, existing_copy_elsewhere, expanded_words, external_program_use, family_has_temporary_service_mark,
            family_own_copy, judged_by_name, known_script_at_location, launcher_names, launcher_option, launcher_option_value,
            path_without_symbolic_links, pattern_expansion_use, program_uses_in_command_text, project_code_program_use, read_temporary_service_table,
            registered_selftest, registered_source_key, resolved_long_option, same_content_as_own, script_body_unresolved_launch, script_directory_expanded,
            script_in_own_repository, script_source_reading, script_sourced_libraries, script_sources_into, shell_assigned_location, shell_location,
            shell_own_directory, shell_places_inside, shell_source_command, shell_source_lines, shell_substitution_end, shell_word_at, shell_word_unquoted,
            skipped_launcher_options, source_family, source_has_temporary_service_mark,
            split_family_library, temporary_service_bare_selftest, temporary_service_bare_stage_samples, temporary_service_command,
            temporary_service_table_at, temporary_service_table_problems, temporary_service_wrapped, unrecognized_launch, variable_definitions,
            word_expansion_use, word_unresolved, wrap_in_temporary_service,
        )
        import lib_project_code_runs_launchers
    except (ImportError, NameError, OSError, SyntaxError, ValueError) as part_error:
        report_part_unusable("lib_project_code_runs_launchers.py", part_error)
    try:
        from lib_project_code_runs_selftest import (
            EqualLocation, FAMILY_LIBRARY_PROBE_FILES, INDEPENDENT_SOURCE_LINE, OWN_DIGEST_PROBE_LIBRARIES, OWN_DIGEST_PROBE_SCRIPTS, OWN_SCRIPT_LIBRARY_NAMED_TABLES,
            PART_PROBE_COPIED_FILES,
            REGISTERED_ARGUMENT_REPEATED_MARK, REGISTERED_ARGUMENT_REPEATED_SOURCE, REGISTERED_SOURCE_LIBRARY,
            SAMPLE_DEVICE_LAYER, SAMPLE_INDEX_REFERENCE_NAME, SAMPLE_SELFTEST_REGISTRY, SAMPLE_SELFTEST_REGISTRY_PROBLEM_KEYS, SHELL_SOURCE_DIRECTORY_ASSIGNMENT,
            SHELL_SOURCE_FORM_CASES, SHELL_SOURCE_FORMS_EXPECTED, SHELL_SOURCE_FORMS_SAMPLE, STALE_VARIABLE_LIBRARY_PROBES, TWO_STEP_LIBRARY, TWO_STEP_SOURCE_LINE,
            TWO_STEP_STAGE, UNRECOGNIZED_SOURCE_APPENDED_FORMS, appended_stage_trees, argument_reading_cells,
            build_sample_workspace, copy_with_own_libraries, location_joined_digest, own_directory_source, own_repository_swapped, selftest, selftest_registry_cells,
            selftest_registry_swapped, source_reading_cells, temporary_service_cells,
        )
    except (ImportError, NameError, OSError, SyntaxError, ValueError) as part_error:
        report_part_unusable("lib_project_code_runs_selftest.py", part_error)
finally:
    _end_part_imports(_saved_modules)


if __name__ == "__main__":
    if sys.argv[1:] == ["--selftest"]:
        sys.exit(selftest())
    if os.environ.get(BREAK_VARIABLE):   # 弄坏开关只在自证里认：命令行不是单独一个 --selftest 而变量非空，一切之前就拒，不管值是什么
        print(f"  ✗ {BREAK_VARIABLE}={os.environ[BREAK_VARIABLE]} 是弄坏开关，只在自证（lib_project_code_runs.py --selftest）里认，平常跑设了就拒，这一趟什么都没做", file=sys.stderr)
        print(f"  → 怎么办：去掉 {BREAK_VARIABLE} 再跑；要证红就跑 {BREAK_VARIABLE}=<项> python3 .claude/hooks/lib_project_code_runs.py --selftest", file=sys.stderr)
        sys.exit(2)
    print("用法：python3 .claude/hooks/lib_project_code_runs.py --selftest（判定入口按文件路径导入这份模块再调，见文件头）", file=sys.stderr)
    sys.exit(2)
