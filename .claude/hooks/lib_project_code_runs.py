"""一条命令是不是本仓之外要经 task-server 起的外部程序、认不认得出起的是什么、起的是不是本仓的代码或编译、是不是绕开服务的写法。

project-code-guard.sh（执行前拒绝）按文件路径用 importlib 导入这一份；同目录的 lib_shell_words.py（切词、剥前缀与包装）由这份自己导入，调用方从 `shell_words` 属性拿同一份。
谁能跑什么、经没经 task-server 提交，是 project-code-guard.sh 的事，不在这里。

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
      读脚本正文的一方不再读进去。盘上有这份文件、又不是本仓那一份（路径尾部相同的拷贝、经符号链接起的、内容连同它 source / import 的库与本仓那一份不逐字节相同的）为假；
      盘上没有的按路径尾部为真。弄坏开关 same-family-by-path-tail（只看路径尾部）、family-copy-content-unchecked（内容相同的拷贝也不认）。
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
      temporary-service-table-lenient、temporary-service-command-bare（开关表逐个写红的格）。
  unrecognized_launch(words, launcher=None) -> str | None、cargo_arguments_expanded(words, variables) -> list[str]
      认不出起什么的写法（project-code-guard.sh 在命令文本这一层判，kind unrecognized、类名「认不出起什么的写法」；认不出的一律拒）：
      命令词或 bash / source 起的脚本路径带展开不了的变量、命令替换、位置参数、进程替换，或展开出来夹着空白；source / . 的文件写成这样；cargo 起编出来的代码的子命令
      （CARGO_RUNNING_SUBCOMMANDS）或参数写成这样（`--` 之后、CARGO_HARMLESS_VALUE_OPTIONS 的值除外）。cargo 参数写成前面赋过字面值的变量的先由 cargo_arguments_expanded 展开。
      弄坏开关 unrecognized-launches-allowed、cargo-arguments-unexpanded。
  script_directory_expanded(text, script_path) -> str、script_sourced_libraries(path) -> list[str]
      读脚本正文时把按自己所在目录取路径的写法（SCRIPT_DIRECTORY_FORMS）换成脚本所在的目录（照起它的路径，不解符号链接），source 的库照字面路径读进来判；
      script_sourced_libraries 列一份脚本按自己所在目录 source（shell）或 import / 点名（python）的库，同族拷贝
      （same_content_as_own）照它逐份比 sha256（第一百一十八节；弄坏开关 sourced-libraries-unread、selftest-work-copy-libraries-unchecked）。
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
  task_server_client_subcommand(words, directory, launcher=None) -> str | None、subagent_task_server_refusal(words, directory, launcher=None) -> str | None
      下面「经 task-server 起」那一段的子 agent 那一判：子 agent 起提交客户端只许 status、log；出路那一句 SUBAGENT_HAND_BACK_WAY_OUT 只写在这一份里。
  selftest_registry_problems(path, root) -> list[str]
      登记表写坏的行（判定认不出、理由太短、不在三处目录里、重复）与指到 root 下不存在的文件的行，逐条一句；空表是没毛病。

弄坏开关（只给自证判红用）：LIB_PROJECT_CODE_RUNS_BREAK 设成下面一个或几个（逗号分隔），--selftest 与 project-code-guard.sh --selftest 都必须判红；
平常跑（命令行不是单独一个 --selftest）设了就拒、退 2，不认识的值也拒。一条一行「LIB_PROJECT_CODE_RUNS_BREAK=<项>  弄坏哪一处」，下一行起写打开后红的自证项（失败行 `✗ lib_project_code_runs 自检：` 之后的句首）。
  LIB_PROJECT_CODE_RUNS_BREAK=admitted-variable-unjudged  两份判法都不认「已由 task-server 准入」开关
      「绕开：命令文本里 export 准入开关」「绕开：python 代码里写 environ 的准入开关」「绕开：命令前设准入开关」
  LIB_PROJECT_CODE_RUNS_BREAK=bench-unjudged  cargo bench 不判（bench 在链接 验证包的包上、验证包上、工作区全量上都放过去）
      「跑编译出来的代码：cargo bench --no-run 只编不跑」
  LIB_PROJECT_CODE_RUNS_BREAK=brace-command-words-bare  命令词展开过的那几条直接 classify，不再切词剥包装
      「花括号：花括号写在包装上（{nice,-n,5} herd7 x）」
  LIB_PROJECT_CODE_RUNS_BREAK=brace-expansion-literal  花括号一律照字面、不展开
      「花括号：命令词里的花括号展开（herd{7,x} 起的是 herd7）」「花括号展开与 bash 一致：{a,b}」
  LIB_PROJECT_CODE_RUNS_BREAK=brace-sequence-literal  {1..3}、{a..e} 这类序列不展开
      「花括号：数字序列展开（herd{1..9} 里有 herd7）」「花括号展开与 bash 一致：{1..3}」
  LIB_PROJECT_CODE_RUNS_BREAK=build-scripts-unjudged  starts_a_build 不认 SCRIPTS_STARTING_BUILDS 里的脚本（compile-then-swap.py 一律交 None）
      「起编译：python3 起 compile-then-swap.py」「起编译：直接执行别处树里的 compile-then-swap.py」
  LIB_PROJECT_CODE_RUNS_BREAK=build-subcommands-unjudged  starts_a_build 不认会编的 cargo（一律交 None）
      「起编译：cargo build」「起编译：cargo b 简写」「起编译：cargo check」「起编译：cargo clippy」「起编译：cargo test --no-run 只编不跑也算」「起编译：cargo doc」「起编译：cargo run」「起编译：全局选项在子命令前」「起编译：--config 定的别名当会编」九格红（应当是 True，实际 False）
  LIB_PROJECT_CODE_RUNS_BREAK=cargo-arguments-unexpanded  cargo 参数不展开
      「cargo 参数展开：F=--workspace 之后 cargo test $F」
  LIB_PROJECT_CODE_RUNS_BREAK=checked-cells-last-only  checked_cells() 给几次 --check 只认最后一次（不取并集）
      「阶段参数 --check：--check=a 与 --check b 给两次取并集」「阶段参数 --check：--check 给三次（含 --check= 写法）取并集」
  LIB_PROJECT_CODE_RUNS_BREAK=claude-executable-path-unrecognized  unrecognized_launch() 把命令词 "$CLAUDE_CODE_EXECPATH"、"${CLAUDE_CODE_EXECPATH}" 照变量拼出的认不出：
      「认得出：命令词是 $CLAUDE_CODE_EXECPATH（检出钩子 ⑬ 判起 claude）」「认得出：命令词是 ${CLAUDE_CODE_EXECPATH}（检出钩子 ⑬ 判起 claude）」
  LIB_PROJECT_CODE_RUNS_BREAK=command-word-variables-unexpanded  command_word_expanded 原样交回（命令词写在变量里的看不见）
      「命令词变量：S=herd7 之后 $S 起 herd7」
  LIB_PROJECT_CODE_RUNS_BREAK=configuration-before-subcommand-only  cargo 子命令之后的 --config 不认
      「跑编译出来的代码：--config 写在子命令之后的别名」
  LIB_PROJECT_CODE_RUNS_BREAK=device-layer-unrecognized  实验装置那一层的登记不认
      「样本表：装置那一层登记过的单独 --selftest 认」
  LIB_PROJECT_CODE_RUNS_BREAK=dispatch-launchers-unseen  command_under_dispatch_launcher() 一律交 None（multi-host-run.sh --peer … -- 里面那条看不见，registered_selftest 照旧把
      「启动器：multi-host-run.sh --peer 带 --memory-cap、--label 各一个值」「派发起法：multi-host-run.sh --peer -- <脚本> --selftest 不当派发器自己的自证」
  LIB_PROJECT_CODE_RUNS_BREAK=expanded-project-code-unjudged  展开（花括号、通配）出来的本仓的代码、会编的 cargo、剥掉的包装一律不认（project_code_program_use 交 None）
      「展开：通配起本仓的脚本经 capped.sh（scripts/hosts/capped.s? 4 true）」「展开：花括号拼出 cargo test（{cargo,test,-p,singlefs-core}）」
  LIB_PROJECT_CODE_RUNS_BREAK=external-programs-unlisted  外部程序名单不查（qemu-system-*、herd7、e152-file-system-benchmark 直接起放过去）
      「外部程序：qemu-system-x86_64」「外部程序：herd7 跑 litmus」「外部程序：e152-file-system-benchmark 光名字从 PATH 起」
  LIB_PROJECT_CODE_RUNS_BREAK=extra-launchers-unseen  名单外启动器不剥（应当是起的那条命令，实际 None）；project-code-guard.sh 的八格见它的文件头
      「启动器：」
  LIB_PROJECT_CODE_RUNS_BREAK=family-copy-content-unchecked  内容相同的拷贝也不认
      「同族：与本仓那一份逐字节相同的 mutate.sh 拷贝照按名字判」
  LIB_PROJECT_CODE_RUNS_BREAK=gate-top-library-as-stage  门禁目录顶层以 lib- 起头的共用库当门禁阶段判：登记表里那一行当门禁阶段的样本档行（selftest 认不出、算写坏的行），命令认不出是它的自证
      「样本登记表里写坏的几行逐条报出来（…）」「仓里的登记表（…）没有写坏的行、指到的文件在 … 里都在」「样本表：门禁目录顶层 lib- 起头的共用库登记过，单独 --selftest 认」
  LIB_PROJECT_CODE_RUNS_BREAK=gpu-quiesce-opaque  command_under_gpu_quiesce 一律交 None，gpu-quiesce-run.sh -- <命令> 不剥（里面那条看不见）
      「gpu-quiesce-run.sh 清场之后起命令的包装」
  LIB_PROJECT_CODE_RUNS_BREAK=herd7-version-flag-counted  herd7 只问版本（-version、--version）也算外部程序
      「外部程序：herd7 -version 只问版本，放行」「外部程序：herd7 --version 只问版本，放行」
  LIB_PROJECT_CODE_RUNS_BREAK=launcher-drops-property-environment  systemd-run -p Environment= / EnvironmentFile= 设的变量不带进里面那条命令
      「启动器环境：systemd-run -p Environment= 设别名」「启动器环境：systemd-run -p EnvironmentFile= 文件里设别名」
  LIB_PROJECT_CODE_RUNS_BREAK=launcher-drops-setenv  systemd-run -E / --setenv= 设的变量不带进里面那条命令
      「启动器环境：systemd-run -E 设别名」「启动器环境：systemd-run --setenv= 设别名」
  LIB_PROJECT_CODE_RUNS_BREAK=launcher-long-options-partial  启动器的长选项只写前缀、值另起一个词的不认
      「strace --output 文件（值另起一个词）包一层」「strace -c --summary-col calls（长选项只写前缀）包一层」
  LIB_PROJECT_CODE_RUNS_BREAK=launcher-whole-word-options  启动器的短选项合写不逐字母查表，整个词当一个选项
      「strace -fo 文件（短选项合写）」「flock -xw 10 锁文件（短选项合写）」
  LIB_PROJECT_CODE_RUNS_BREAK=libtest-list-by-presence  libtest_lists_only() 见到 --list 这个词就算只列（不跳过带值选项的值、不在 -- 处停）
      「libtest 参数：--skip --list（--list 是 --skip 的值）不是只列」「libtest 参数：--logfile --list（--list 是日志文件名）不是只列」「libtest 参数：-- 之后的 --list 是过滤词，不是只列」「libtest 参数：-- 隔开两段之后的 --list 是过滤词，不是只列」
  LIB_PROJECT_CODE_RUNS_BREAK=main-agent-tools-refused  MAIN_AGENT_TOOLS_RUN_DIRECTLY 不认
      「名单：主 agent 停进程的 proc.py」「名单：主 agent 的 task-server/control stop」「名单：主 agent 起定时提醒的 remind.sh」「名单：主 agent 直接起服务自证」
  LIB_PROJECT_CODE_RUNS_BREAK=mutants-help-counted  test_invocation 里 cargo mutants 只带 --help、-h、--version、-V 的照旧算跑测试
      「跑编译出来的代码：cargo mutants --help 只打帮助，不起 cargo test」「跑编译出来的代码：cargo mutants -V -h 只打版本与帮助，不起 cargo test」
  LIB_PROJECT_CODE_RUNS_BREAK=pattern-caret-negation-literal  [^…] 不换成 [!…]
      「通配：tools/herd[^x] 的 [^…] 照 bash 当取反」
  LIB_PROJECT_CODE_RUNS_BREAK=pattern-command-words-literal  命令词带通配的只照字面判、不展开
      「通配：tools/her?7 展开成 tools/herd7」「通配：tools/herd* 展开成 herd7 与 herd7-helper，其中有 herd7」
  LIB_PROJECT_CODE_RUNS_BREAK=pattern-expansions-bare  通配展开出来的那条直接 classify，不再切词剥包装
      「通配：展开出来的是 nice 包装，包装里的 herd7 照认」
  LIB_PROJECT_CODE_RUNS_BREAK=posix-brackets-by-python-glob  带 `[:`、`[=`、`[.` 的词照旧交给 python glob（POSIX 字符类、等价类、整理符号认不出，零命中照字面判）
      「通配：POSIX 字符类 tools/her[[:alpha:]]7 交给 bash 展开」「bash 起不来时 POSIX 字符类起外部程序照拒」
  LIB_PROJECT_CODE_RUNS_BREAK=posix-brackets-unresolved-literal  bash 展不开或判不出时照字面判（与零命中一样），不拒
      「bash 起不来时 POSIX 字符类起外部程序照拒」
  LIB_PROJECT_CODE_RUNS_BREAK=project-scripts-unjudged  project_code_use 一律交 None（本仓的脚本都不当项目代码）
      「本仓代码：」「名单：」
  LIB_PROJECT_CODE_RUNS_BREAK=prune-server-directory-unrecognized  自证认的目录里去掉 prune-server（selftest_directories；登记表里 prune-server/control 那一行算写坏的行，它的 --selftest 不当自证）
      「样本登记表里写坏的几行逐条报出来（…）」「仓里的登记表（…）没有写坏的行、指到的文件在 … 里都在」
  LIB_PROJECT_CODE_RUNS_BREAK=quoted-braces-expanded  引号里的花括号也照展
      「花括号：命令词位置单引号 {herd7,x.litmus} 照字面放行」「花括号：命令词位置双引号 {herd7,x.litmus} 照字面放行」
  LIB_PROJECT_CODE_RUNS_BREAK=run-build-directory-opaque  command_under_run_build_directory 一律交 None，run-build-directory.sh <命令> 不剥（里面那条看不见）
      「run-build-directory.sh 这一趟自己的编译目录的包装」
  LIB_PROJECT_CODE_RUNS_BREAK=same-family-by-path-tail  同族脚本退回只按路径尾部认
      「同族：别处树里的门禁阶段拷贝不按名字判（读正文）」「同族：别处树里的 mutate.sh 拷贝不按名字判（读正文）」
  LIB_PROJECT_CODE_RUNS_BREAK=script-body-launches-unjudged  script_body_unresolved_launch() 一律交 None（脚本正文里经变量起的自证、门禁阶段、入口脚本、cargo 看不见）：
      「脚本正文：数组变量的定义里写着 --selftest，照认不出」「脚本正文：cargo test 的参数是解不出的变量，照认不出」
  LIB_PROJECT_CODE_RUNS_BREAK=selftest-extra-arguments-allowed  带了 --selftest 之外参数的也当单独一个 --selftest
      「样本表：带了 --selftest 之外参数的不认」
  LIB_PROJECT_CODE_RUNS_BREAK=selftest-pattern-words-judged  命令词带通配的也当自证判
      「样本表：命令词带通配的不当登记过的自证」
  LIB_PROJECT_CODE_RUNS_BREAK=selftest-registry-by-path-tail  登记表只按路径尾部认，别的树里的同名脚本也算登记过
      「样本表：别的树里路径尾部相同的脚本不算登记」
  LIB_PROJECT_CODE_RUNS_BREAK=selftest-registry-lenient  登记表里写坏的行照收、不报
      「样本登记表里写坏的几行逐条报出来（种类、理由、目录、重复、文件不在、门禁阶段那几行的种类与假工具目录）」
  LIB_PROJECT_CODE_RUNS_BREAK=sourced-libraries-unread  不换脚本所在目录的写法
      「source 的库：$(dirname "${BASH_SOURCE[0]}") 换成脚本所在目录」
  LIB_PROJECT_CODE_RUNS_BREAK=stage-directory-pwd-unexpanded  阶段目录写成 $PWD/… 时不换成当前目录
      「门禁样本：阶段目录写成 $PWD/…、--item 点名一道登记过的」
  LIB_PROJECT_CODE_RUNS_BREAK=stage-feed-script-by-name  不带 --item 时只按名字认 stage-selftest.sh，别处树里的拷贝也信
      「门禁样本：别处树里的 stage-selftest.sh 拷贝不带 --item」
  LIB_PROJECT_CODE_RUNS_BREAK=stage-items-as-directory  --item / --list-items 不认，当阶段目录判
      「门禁样本：--item 点名一道登记过的」「门禁样本：--list-items 只列、不喂样本」
  LIB_PROJECT_CODE_RUNS_BREAK=stage-items-script-by-name  带 --item 时只按名字认 stage-selftest.sh，别处树里的拷贝也信
      「门禁样本：别处树里的 stage-selftest.sh 拷贝带 --list-items」
  LIB_PROJECT_CODE_RUNS_BREAK=stage-items-unjudged  --item 点名的几道不判
      「门禁样本：--item 点名里夹一道登记表外的」「门禁样本：--item 点名的样本多一份 setup.sh」
  LIB_PROJECT_CODE_RUNS_BREAK=stage-samples-fixtures-unchecked  喂的样本不与暂存区比
      「门禁样本：样本多一份 setup.sh」「门禁样本：内容改过」「门禁样本：执行位改过」「门禁样本：少了文件」
  LIB_PROJECT_CODE_RUNS_BREAK=stage-samples-ignored  门禁样本一律不认
      「门禁样本：样本档全登记、与暂存区相同的阶段目录」「门禁样本：--item 点名一道登记过的」「门禁样本：--list-items 只列、不喂样本」「门禁样本：--item 点名不存在的阶段（它自己退 2、一份样本都不喂）」「门禁样本：阶段目录写成 $PWD/…、--item 点名一道登记过的」「门禁样本：--item 与值之间夹的 --force 照 preflight 摘掉」「门禁样本：样本与暂存区不同的说明点名那份文件」「门禁样本：--item 点名里夹登记表外的，说明点名那一道、不点没毛病的那一道」
  LIB_PROJECT_CODE_RUNS_BREAK=strace-drops-env  strace -E / --env 设的变量不带进里面那条命令
      「启动器环境：strace -E 设别名」「启动器环境：strace --env= 设别名」
  LIB_PROJECT_CODE_RUNS_BREAK=task-server-builds-unjudged  两份判法都不认直接删服务的编译目录
      「绕开：python 代码 shutil.rmtree 服务的编译目录」「绕开：rm -rf 服务的编译目录」「绕开：rm 写 ~ 起头的编译目录」「绕开：rm 状态目录下的 builds」「绕开：find -delete 编译目录」「绕开：find -exec rm 编译目录」「绕开：rm -rf 编译目录的上级」
  LIB_PROJECT_CODE_RUNS_BREAK=task-server-directory-unrecognized  自证认的目录里去掉 task-server（selftest_directories；登记表里那一层的行算写坏的行，那一层的 --selftest 不当自证）
      「样本登记表里写坏的几行逐条报出来（…）」「仓里的登记表（…）没有写坏的行、指到的文件在 … 里都在」
  LIB_PROJECT_CODE_RUNS_BREAK=task-server-paths-unjudged  task_server_text_bypass 不认库、密钥、socket 与载入共用库
      「绕开：点到库文件名」「绕开：点到签名密钥文件名」「绕开：点到 socket」「绕开：python 代码载入 task_server_lib」
  LIB_PROJECT_CODE_RUNS_BREAK=task-server-scripts-unjudged  task_server_command_bypass 不认直接起服务本体与每次尝试的那一层
      「绕开：直接起服务本体」
  LIB_PROJECT_CODE_RUNS_BREAK=task-server-submission-unparsed  task_server_submission 一律交 None
      「提交：submit -- 后面那条」「提交：--cwd 照这一条的目录解开」「提交：仓副本里逐字节相同的客户端」「样本表：task-server/task submit -- 里那条带 --selftest，不当客户端自己的自证」
  LIB_PROJECT_CODE_RUNS_BREAK=task-server-submit-as-selftest  registered_selftest 不认 `task-server/task submit -- <命令…>`：里面那条带的 --selftest 当客户端自己的自证判
      「样本表：task-server/task submit -- 里那条带 --selftest，不当客户端自己的自证」
  LIB_PROJECT_CODE_RUNS_BREAK=task-server-units-unjudged  task_server_command_bypass 不认 systemctl 停禁改它的单元
      「绕开：systemctl --user stop 服务单元」「绕开：systemctl --user kill 一次尝试的单元」「绕开：systemctl --user stop 通配」
  LIB_PROJECT_CODE_RUNS_BREAK=temporary-service-bare-unjudged  temporary_service_bare_selftest() 一律交 None（要临时服务的自证写成裸 --selftest 也不拦）：
      「要临时服务：required 行的裸 --selftest 交整条包装写法」「要临时服务：仓里 crates-mutation-rows.py 的裸 --selftest 被点名、给整条包装写法」
  LIB_PROJECT_CODE_RUNS_BREAK=temporary-service-command-bare  temporary_service_command() 给裸写的样子（拒绝说明里没有包装）：
      「要临时服务：required 行的裸 --selftest 交整条包装写法」「要临时服务：python 脚本的整条写法起头是 python3」「要临时服务：仓里 crates-mutation-rows.py 的裸 --selftest 被点名、给整条包装写法」
  LIB_PROJECT_CODE_RUNS_BREAK=temporary-service-command-binary-placeholder  temporary_service_command() 的整条写法带 --server-binary 与尖括号占位（照抄进 shell 时尖括号被当成输入重定向）：
      「要临时服务：required 行的裸 --selftest 交整条包装写法」「要临时服务：python 脚本的整条写法起头是 python3」「要临时服务：整条写法不带 --server-binary 与尖括号占位」「要临时服务：裸 --selftest 的拒绝说明里没有尖括号占位」「要临时服务：仓里 crates-mutation-rows.py 的裸 --selftest 被点名、给整条包装写法」
  LIB_PROJECT_CODE_RUNS_BREAK=temporary-service-environment-dropped  environment_under_temporary_service() 原样交回（包装里的原命令看起来没有标记）：
      「要临时服务：包装剥开重判时补上的标记让里面那条放行」「要临时服务：仓里 crates-mutation-rows.py 的 --selftest 在包装底下放行」
  LIB_PROJECT_CODE_RUNS_BREAK=temporary-service-marker-ignored  不看环境里的临时服务标记（套了包装、自己设了标记的也拦）：
      「要临时服务：命令前设了临时服务的标记，裸 --selftest 放行」「要临时服务：包装剥开重判时补上的标记让里面那条放行」「要临时服务：仓里 crates-mutation-rows.py 的 --selftest 在包装底下放行」
  LIB_PROJECT_CODE_RUNS_BREAK=temporary-service-own-server-as-required  own-server 的行也当 required（自己起服务的自证被要求套临时服务）：
      「要临时服务：own-server 行的裸 --selftest 不拒」
  LIB_PROJECT_CODE_RUNS_BREAK=temporary-service-table-lenient  要临时服务的表写坏的行照收、不报（判定认不出、理由太短、键重复）：
      「要临时服务：理由太短的一行不算登记」「要临时服务：样本表写坏的、两边对不上的几行逐条报出来（判定、理由、键重复、文件不在、没登记、源码里没有核对、漏登）」
  LIB_PROJECT_CODE_RUNS_BREAK=temporary-service-wrapper-unseen  temporary_service_wrapped() 一律交 None（临时 prune-server 的包装 `prune_server_client_selftest.py --with-temporary-service … -- <原命令>`
      「样本表：包装 --server-binary -- 里那条带 --selftest，不当包装自己的自证」
  LIB_PROJECT_CODE_RUNS_BREAK=text-tools-refused  TEXT_TOOLS_RUN_DIRECTLY 不认（名单上的文本工具也要经 task-server）
      「名单：replace-once.py 定点改」「名单：仓副本里逐字节相同的 replace-once.py」「名单：认领草稿目录」「名单：gate-overlap.py --list」「名单：提交客户端的 wait」「名单：task-server/control check」「名单：改标签的 relabel-item.py」「名单：备份原文件的 copy-originals.py」
      「名单：建仓副本的 minimal-copy.py」「名单：打副本闭包的 lib_repository_copy.py entries」（这两格照判法推的、还没复跑）
  LIB_PROJECT_CODE_RUNS_BREAK=two-logs-opaque  command_under_two_logs 一律交 None，two-logs.sh <名> -- <命令> 不剥（里面那条看不见）
      「two-logs.sh 日志出两份的包装」「two-logs.sh --quiet 日志出两份的包装」
  LIB_PROJECT_CODE_RUNS_BREAK=unrecognized-launches-allowed  unrecognized_launch() 一律交 None
      「认不出：」「脚本正文：cargo test 的参数是解不出的变量，照认不出」
  project-code-guard.sh --selftest 原样转出上面这几格的失败行，另加它自己的格（写在它的文件头）。
"""
import ast, contextlib, filecmp, fnmatch, functools, glob, hashlib, importlib.util, os, re, shlex, shutil, stat, subprocess, sys, tempfile
from typing import NamedTuple


def load_sibling_module(module_name):
    spec = importlib.util.spec_from_file_location(module_name, os.path.join(os.path.dirname(os.path.abspath(__file__)), module_name + ".py"))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


shell_words = load_sibling_module("lib_shell_words")


class ProgramUse(NamedTuple):
    kind: str      # 细分的用法（external-program、pattern-unresolved、unrecognized），project-code-guard.sh 按它写拒绝说明
    category: str  # 报给人看的类名，KIND_CATEGORY 的值
    detail: str    # 这一条为什么算：认出的是什么


# 要经 task-server/task 起、本仓的代码判法（project_code_use、starts_a_build、runs_compiled_code）认不出的三种写法：kind → 类名
KIND_CATEGORY = {
    "external-program": "本仓之外的外部程序", "project-code": "展开出来的本仓的代码", "pattern-unresolved": "判不出的通配", "unrecognized": "认不出起什么的写法",
}
# 这几个目录下的脚本带 --selftest 跑是跑它的自证；.claude/gate.d/lib/ 下的共用库直接执行就是跑自证，带不带 --selftest 都算。认不认按登记表（registered_selftest）
# scripts/ 下按用途分的子目录（scripts/README.md）：脚本都住在这一层
SCRIPT_CATEGORY_DIRECTORIES = tuple(f"scripts/{category}" for category in ("agents", "crash", "devices", "experiment-apparatus", "gate", "hosts", "kb", "mutation", "prune-server"))
# 统一任务服务的根目录（records/2026-10-30-编译绕过内存总上限.md 批 3b 新加）：服务的脚本住在这一层，自证同样登记在登记表里（selftest_directories）
TASK_SERVER_DIRECTORY = "task-server"
# 判定库服务的根目录：control 脚本（装、起、停、查、监听，--selftest 只起假的 ip、ssh、systemctl）住在这一层，自证同样登记在登记表里
PRUNE_SERVER_DIRECTORY = "prune-server"
# scripts/mutation/tests：变异那几份脚本的回归单测（test_audit_regressions.py，单独一个 --selftest 跑 unittest），登记表里那一行落在这一层
MUTATION_REGRESSION_TESTS_DIRECTORY = "scripts/mutation/tests"
SELFTEST_DIRECTORIES = SCRIPT_CATEGORY_DIRECTORIES + (MUTATION_REGRESSION_TESTS_DIRECTORY, TASK_SERVER_DIRECTORY, PRUNE_SERVER_DIRECTORY, ".claude/hooks", ".claude/gate.d/lib")
# 实验装置那一层（文件头「实验装置那一层」）：这个目录下正好一层的目录，登记了的单独 --selftest 才当自证
DEVICE_SELFTEST_PARENT = "scripts/experiment-apparatus"
GATE_LIBRARY_DIRECTORY = ".claude/gate.d/lib"
# 自证登记表：一行 `<相对仓根的路径><制表符><种类><制表符># <理由>`，写法在表头。放在这份文件旁边；自证换成样本表时改这个变量
SELFTEST_REGISTRY_NAME = "selftest-registry.tsv"
SELFTEST_REGISTRY_PATH = os.path.join(os.path.dirname(os.path.abspath(__file__)), SELFTEST_REGISTRY_NAME)
SELFTEST_KIND = "selftest"
SELFTEST_REASON_MINIMUM = 4   # 理由（# 之后）至少几个字
# 门禁阶段的样本档（文件头「门禁样本」）：登记表里键是 .claude/gate.d/<阶段>.sh 的行，种类写这两种之一
GATE_STAGE_DIRECTORY = ".claude/gate.d"
# 门禁目录顶层以它起头的文件是共用库、不是阶段（lib-lag-baseline.py 这类）：gate_stage_name 不当阶段认，登记表里它那一行照脚本的自证判（gate_top_library_key）
GATE_TOP_LIBRARY_PREFIX = "lib-"
# samples：样本只读文本、里面的工具都是假的；samples:<目录>：样本根里 <目录> 放着假工具（selftest-process-guard.py 不把那一层的工具当真的）
STAGE_SAMPLES, STAGE_SAMPLES_WITH_TOOLS = "samples", "samples:"
STAGE_SELFTEST_SCRIPT = "stage-selftest.sh"
# stage-selftest.sh 自己认的两个参数（它文件头「用法」）：--item <阶段文件名> 只喂点名那几道（给几次取并集），--list-items 只列可点名的阶段、不喂样本；别的词是阶段目录
STAGE_SELFTEST_ITEM_OPTION, STAGE_SELFTEST_LIST_OPTION = "--item", "--list-items"
# 阶段目录写成 $PWD/… 或 ${PWD}/…（拒绝说明里给的写法）：当前目录认得出、命令文本里没另设 PWD 时换成当前目录（stage_selftest_item_problems）
WORKING_DIRECTORY_VARIABLE = re.compile(r"^\$(?:PWD|\{PWD\})(?=/|$)")
# 命令词里带这几个通配字符的不当自证（registered_selftest）：bash 展开之后才知道起的是哪一份；case 的模式行被切成命令时也是这样的词
SELFTEST_PATTERN_CHARACTERS = set("*?[")
# 阶段参数里认不出是哪个目录的词：没展开的变量、命令替换、通配
UNRESOLVED_WORD_CHARACTERS = set("$`*?[]{}")
# 按名字判的仓内脚本：名字 → 它在仓里的位置。命令词是这个名字、又落在这个位置上，judged_by_name 为真
KNOWN_SCRIPT_LOCATIONS = {
    "gate.sh": ".claude/scripts/gate.sh", "check.sh": ".claude/scripts/check.sh", "lkmm.sh": ".claude/scripts/lkmm.sh",
    "gate-staged.sh": "scripts/gate/gate-staged.sh", "mutate.sh": "scripts/mutation/mutate.sh",
    "vm-bench.sh": "scripts/devices/vm-bench.sh", "e152-run.sh": "scripts/devices/e152-run.sh",
    "capped.sh": "scripts/hosts/capped.sh", "run-with-memory-cap.sh": "scripts/hosts/run-with-memory-cap.sh",
    "crash-point-replay-shard-run.sh": "scripts/crash/crash-point-replay-shard-run.sh", "mutation-shard-run.sh": "scripts/mutation/mutation-shard-run.sh",
    "broad-mutation.py": "scripts/mutation/broad-mutation.py",
    "crash-amplification.sh": "scripts/crash/crash-amplification.sh", "crash-amplification-item.sh": "scripts/crash/crash-amplification-item.sh",
    "crash-amplification-two-hosts.sh": "scripts/crash/crash-amplification-two-hosts.sh",
    "build-patched-qemu.sh": "scripts/devices/build-patched-qemu.sh",
    "crash-point-replay.sh": "scripts/crash/crash-point-replay.sh", "replay.sh": "scripts/experiment-apparatus/replay.sh",
    STAGE_SELFTEST_SCRIPT: ".claude/singlefs-ai-sop/scripts/stage-selftest.sh",
}
# 广谱变异的逐条驱动（scripts/mutation/broad-mutation.py）：带这几个参数之一的只读（列项、汇总、帮助），不起 cargo、不改库目录
BROAD_MUTATION_SCRIPT_NAME = "broad-mutation.py"
# Verus 那一道（formal-verification.sh）的两份判法脚本：直接起时干活的用法算 verus-stage（verus_judge_use）
VERUS_PROOF_CHECK_SCRIPT = "verus-proof-check.py"
# 要经 task-server/task 起的外部程序（PATH 上直接起、本仓之外的）：本仓的脚本由 project_code_use 认，会编的 cargo 与编出来的二进制由 starts_a_build、
# runs_compiled_code 认，这几个程序它们认不出；命中就和直接起本仓的代码一样拒。名单不分档，名字取 basename：
# 前缀 qemu-system（qemu-system-x86_64、qemu-system-aarch64……）、herd7（只问版本的放行）、e152-file-system-benchmark（E152 装置的二进制，光名字从 PATH 起）
EXTERNAL_PROGRAM_PREFIXES = ("qemu-system",)
EXTERNAL_PROGRAM_NAMES = frozenset({"herd7", "e152-file-system-benchmark"})
# 只问版本、不起程序干活的参数写法（只带这一个参数）：程序名 → 参数表
EXTERNAL_PROGRAM_VERSION_ONLY_ARGUMENTS = {"herd7": (["-version"], ["--version"])}
E165_PIN_SCRIPT = "e165-pin-red-examples.py"
# 入口脚本之一 scripts/mutation/prove-red.sh（它从 python 里起 cargo，闸看不见）；用法只有 `--copy <仓副本> <crate> <变异名…>`
PROVE_RED_SCRIPT = "prove-red.sh"
# 命令词里没展开的变量名（`$b`、`${b}`、`${b:-x}`）；往它的定义里找 验证包测试目标名时最多往下点几层
UNEXPANDED_VARIABLE_NAME = re.compile(r"\$\{?([A-Za-z_][A-Za-z0-9_]*)")
UNEXPANDED_DEFINITION_DEPTH = 6


def program_use(kind, detail):
    return ProgramUse(kind, KIND_CATEGORY[kind], detail)



# libtest 的只列用例参数与带一个值的选项：lib_run_scope.py 的 cargo_run_shape 判「cargo test -- --list 只列不跑」用
LIST_ONLY_TEST_ARGUMENT = "--list"
LIBTEST_OPTIONS_WITH_VALUE = {"--logfile", "--skip", "--test-threads", "--format", "--color", "-Z", "--shuffle-seed"}



def checked_cells(arguments):
    """阶段参数里 --check 点名的格（--check <格>[,<格>…] 与 --check=<…>，给几次取并集）；没给 --check 交 None（全跑）。lib_run_scope.py 用它数范围声明里的格数。
    弄坏开关 checked-cells-last-only 下给几次只认最后一次（不取并集）。"""
    cells, given, position = set(), False, 0
    while position < len(arguments):
        argument = arguments[position]
        if argument == "--check" and position + 1 < len(arguments):
            if break_is_set("checked-cells-last-only"):
                cells.clear()
            cells.update(filter(None, arguments[position + 1].split(",")))
            given, position = True, position + 2
            continue
        if argument.startswith("--check="):
            if break_is_set("checked-cells-last-only"):
                cells.clear()
            cells.update(filter(None, argument[len("--check="):].split(",")))
            given = True
        position += 1
    return cells if given else None


def libtest_lists_only(libtest_arguments):
    """libtest 的参数里 --list 当选项出现（只列用例、一条都不跑）：跳过带一个值的选项的那个值再找，`--skip --list` 里的 --list 不算；
    见到 `--` 就停、交「不是只列」：libtest 在 `--` 处停止认选项，之后的 --list 是过滤词（`-- --ignored -- --list` 一条都不跑，照样算不是只列，接受的误拒）。
    弄坏开关 libtest-list-by-presence 下见到 --list 这个词就算只列（不跳过带值选项的值、不在 -- 处停）。"""
    if break_is_set("libtest-list-by-presence"):
        return LIST_ONLY_TEST_ARGUMENT in libtest_arguments
    position = 0
    while position < len(libtest_arguments):
        argument = libtest_arguments[position]
        if argument == "--":
            return False
        if argument == LIST_ONLY_TEST_ARGUMENT:
            return True
        position += 2 if argument in LIBTEST_OPTIONS_WITH_VALUE else 1
    return False


def break_is_set(switch_name):
    return switch_name in os.environ.get(BREAK_VARIABLE, "").split(",")


HOOK_REPOSITORY = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
# 自证登记表的键相对哪个仓根解成真实路径（registered_selftest 只认解到这一份的命令词）；自检的样本表那几格换成临时目录
SELFTEST_REGISTRY_ROOT = HOOK_REPOSITORY
# stage-selftest.sh 喂的样本拿哪个仓的 HEAD 加暂存区比（sample_directory_changes）；自检换成临时建的样本仓
SAMPLE_INDEX_REPOSITORY = HOOK_REPOSITORY
# 弄坏开关的环境变量（写法见文件头）
BREAK_VARIABLE = "LIB_PROJECT_CODE_RUNS_BREAK"


CARGO_GLOBAL_OPTIONS_WITH_VALUE = {"--color", "--config", "-Z"}
# cargo test 与 cargo nextest run 带一个值的选项（nextest 的 --run-ignored、-E 过滤式这一类也在里面：cargo test 不认它们，放一张表不误读）
CARGO_TEST_OPTIONS_WITH_VALUE = {"-p", "--package", "--exclude", "--test", "--bin", "--example", "--bench", "-F", "--features",
                                 "--target", "--target-dir", "--manifest-path", "-j", "--jobs", "--profile", "--color",
                                 "--message-format", "--config", "-Z", "--lockfile-path",
                                 "--run-ignored", "-E", "--filterset", "-P", "--retries", "--partition", "--test-threads", "--max-fail"}
GLOB_CHARACTERS = set("*?[")
# 命令词带通配（classify 先照 bash 展开）：每一份单独当命令判，一次最多判这么多份；bash 真起的那一条（头一份当命令、其余当参数）另外总判
PATTERN_EXPANSION_LIMIT = 256
# bash 的 [^…] 与 [!…] 同义，python 的 glob 只认 [!…]
CARET_NEGATION = re.compile(r"\[\^")
# 方括号里的 POSIX 字符类、等价类、整理符号（[:alpha:]、[=h=]、[.h.]）：python glob 不认，带它的词交给 bash 展开（bash_pattern_expansions）
BASH_BRACKET_PATTERN = re.compile(r"\[[:=.]")
# 交给 bash 展开时最多等几秒；起 bash 用的程序（自证换成不存在的路径，演 bash 起不来）
BASH_PATTERN_TIMEOUT = 5
BASH_FOR_PATTERNS = "bash"
# scripts/crash/gpu-quiesce-run.sh -- <命令…>：清场之后原样起那条命令（command_under_gpu_quiesce 剥掉它）
GPU_QUIESCE_SCRIPT = "gpu-quiesce-run.sh"
GPU_QUIESCE_DIRECTORY = "scripts/crash"
# scripts/hosts/two-logs.sh [--quiet] <名> -- <命令…>：日志出两份的包装（转打、写 <名>.log 与 <名>.problems.log，退出码是那条命令的）：command_under_two_logs 剥掉它
TWO_LOGS_SCRIPT = "two-logs.sh"
# scripts/hosts/run-build-directory.sh [--] <命令…>：包一层起、这一趟一个自己的编译目录（跑完删）：command_under_run_build_directory 剥掉它
RUN_BUILD_DIRECTORY_SCRIPT = "run-build-directory.sh"
RUN_BUILD_DIRECTORY_LOCATION = "scripts/hosts"
TWO_LOGS_DIRECTORY = "scripts/hosts"
# 派发起法：scripts/hosts/multi-host-run.sh --peer [选项…] -- <命令…>（command_under_dispatch_launcher 剥掉它）。host-dispatch.py 命令行只认 --selftest，不在这张表里。
# 名字 → (不带值的选项, 带值的选项 → 值的个数)，照脚本文件头「用法」与它切参数的那一段（multi-host-run.sh 的 multi_host_main）
DISPATCH_LAUNCHER_DIRECTORY = "scripts/hosts"
MULTI_HOST_RUN_SCRIPT = "multi-host-run.sh"
DISPATCH_LAUNCHER_OPTIONS = {
    MULTI_HOST_RUN_SCRIPT: (frozenset({"--peer"}), {"--memory-cap": 1, "--label": 1}),
}
# 临时 prune-server 的包装写法（temporary_service_wrapped）：scripts/prune-server/prune_server_client_selftest.py --with-temporary-service [--server-binary <二进制>] -- <原命令…>
TEMPORARY_SERVICE_WRAPPER_DIRECTORY = "scripts/prune-server"
TEMPORARY_SERVICE_WRAPPER_SCRIPT = "prune_server_client_selftest.py"
TEMPORARY_SERVICE_WRAPPER_MODE = "--with-temporary-service"
TEMPORARY_SERVICE_BINARY_OPTION = "--server-binary"
# 要在临时 prune-server 底下起的自证登记表（同目录，写法在表头；temporary_service_bare_selftest 读它）：一行 `<相对仓根的路径><空白><required 或 own-server><空白># <理由>`。自证换成样本表时改这个变量
TEMPORARY_SERVICE_TABLE_NAME = "selftest-temporary-service.tsv"
TEMPORARY_SERVICE_TABLE_PATH = os.path.join(os.path.dirname(os.path.abspath(__file__)), TEMPORARY_SERVICE_TABLE_NAME)
TEMPORARY_SERVICE_REQUIRED, TEMPORARY_SERVICE_OWN_SERVER = "required", "own-server"
# 包装给里面那条原命令设的标记（scripts/prune-server/prune_server_client.py 的 TEMPORARY_SERVICE_VARIABLE）：自证开跑先核它，没有就退 2
TEMPORARY_SERVICE_VARIABLE = "PRUNE_SERVER_TEMPORARY_SERVICE_SOCKET"
# 自证源码里调客户端的临时服务核对时出现的那个函数名（门禁 code-tooling 认包装写法用的是同一个记号）。拆成两段写：这份文件自己也登记在自证登记表里，
# 整段写出来会被 temporary_service_table_problems 的第二条对一遍当成「调了核对、表里漏登」
TEMPORARY_SERVICE_SOURCE_MARK = "selftest_service_" + "problem"
# multi-host-run.sh 头一个参数是这几样时不是 --peer 那一种派发用法（自证），`--` 之后的命令不跑
MULTI_HOST_RUN_OTHER_MODES = frozenset({"--selftest"})
# 花括号展开与参数里的通配（classify 先照 bash 展开，文件头「花括号与参数里的通配」）：一个词最多展开成这么多份，超出的不判
WORD_EXPANSION_LIMIT = 1024
# 展开出来的命令再切一遍、再判时又带花括号或通配的（文件名里本身带这几个字符），最多再套这么多层
WORD_EXPANSION_DEPTH_LIMIT = 4
# 花括号展开只看这几种记号（brace_gobble）；一个词最多看这么多个，配不上的花括号多到超过的这个词不展开
BRACE_TOKENS = re.compile(r"\$\{|[{},]|\.\.")
BRACE_SCAN_LIMIT = 200000
# 花括号里的序列：{<起>..<止>[..<步长>]}，起止同是整数或同是一个字母；步长是整数（0 当 1，正负只看起止的方向）
BRACE_SEQUENCE = re.compile(r"([+-]?[0-9]+|[A-Za-z])\.\.([+-]?[0-9]+|[A-Za-z])(?:\.\.([+-]?[0-9]+))?")
# cargo llvm-cov 不起测试的子命令；cargo hack 后面接的、起测试或跑程序的子命令
LLVM_COV_NON_RUNNING_SUBCOMMANDS = {"report", "clean", "show-env"}
HACK_INNER_SUBCOMMANDS = {"test", "t", "run", "r", "nextest", "miri", "llvm-cov", "mutants"}
# cargo mutants 只打帮助或版本的参数：只带这几个（一个或几个）时不起 cargo test
MUTANTS_QUERY_ARGUMENTS = frozenset({"--help", "-h", "--version", "-V"})
ALIAS_CONFIGURATION = re.compile(r"^\s*alias\.([A-Za-z0-9_-]+)\s*=")
ALIAS_ENVIRONMENT_PREFIX = "CARGO_ALIAS_"


class CargoCommand(NamedTuple):
    subcommand: str
    rest: list                  # 子命令后面的参数
    directory: str | None       # 按 -C 换过之后的目录
    configuration_values: list  # 全局选项 --config 的值（--config=<值> 与 --config <值> 两种写法）


def cargo_subcommand(arguments, directory):
    """cargo 的全局选项（+工具链、--config、-C 目录……）跳过之后的子命令：交 CargoCommand；没有子命令交 None。"""
    index, configuration_values = 0, []
    while index < len(arguments):
        argument = arguments[index]
        if argument.startswith("+"):
            index += 1
        elif argument in CARGO_GLOBAL_OPTIONS_WITH_VALUE:
            if argument == "--config" and index + 1 < len(arguments):
                configuration_values.append(arguments[index + 1])
            index += 2
        elif argument == "-C":
            directory = shell_words.resolve_path(directory, arguments[index + 1]) if index + 1 < len(arguments) else directory
            index += 2
        elif argument.startswith("-"):
            if argument.startswith("--config="):
                configuration_values.append(argument[len("--config="):])
            index += 1
        else:
            break
    if index >= len(arguments):
        return None
    rest = arguments[index + 1:]
    if not break_is_set("configuration-before-subcommand-only"):
        configuration_values += configuration_values_after_subcommand(rest)
    return CargoCommand(arguments[index], rest, directory, configuration_values)


def configuration_values_after_subcommand(rest):
    """子命令之后、`--` 之前的 --config 值（--config 是 cargo 的全局选项，写在 test 之后照认）：--config <值> 与 --config=<值> 两种写法；
    按 cargo test 的带值选项表跳过别的选项的值（`--features --config` 里的 --config 是 --features 的值）。"""
    values, position = [], 0
    while position < len(rest):
        word = rest[position]
        if word == "--":
            break
        if word == "--config":
            if position + 1 < len(rest):
                values.append(rest[position + 1])
            position += 2
        elif word.startswith("--config="):
            values.append(word[len("--config="):])
            position += 1
        else:
            position += 2 if word in CARGO_TEST_OPTIONS_WITH_VALUE else 1
    return values


def test_invocation(subcommand, rest):
    """cargo 的子命令与它后面的参数 → (怎么起的：'test'、'nextest'、'run'、'bench' 或 None, 照 cargo test 的写法读的参数)。
    bench 与 test 同判（--no-run 照 test 的；开关 bench-unjudged 下不判）；nextest 只认 run（r）；miri、llvm-cov、hack 剥掉自己那一层再看里面那个子命令（llvm-cov 不带子命令时就是跑测试）；mutants 反复起 cargo test。"""
    if subcommand in ("test", "t"):
        return "test", rest
    if subcommand in ("run", "r"):
        return "run", rest
    if subcommand == "bench" and not break_is_set("bench-unjudged"):
        return "bench", rest
    if subcommand == "nextest":
        return ("nextest", rest[1:]) if rest[:1] in (["run"], ["r"]) else (None, [])
    if subcommand == "miri":
        return test_invocation(rest[0], rest[1:]) if rest else (None, [])
    if subcommand == "llvm-cov":
        if rest[:1] == ["nextest"]:
            return "nextest", rest[1:]
        if rest and rest[0] in LLVM_COV_NON_RUNNING_SUBCOMMANDS:
            return None, []
        if rest and not rest[0].startswith("-"):
            return test_invocation(rest[0], rest[1:])
        return "test", rest
    if subcommand == "hack":
        options = rest[:rest.index("--")] if "--" in rest else rest
        for index, word in enumerate(options):
            if word in HACK_INNER_SUBCOMMANDS:
                return test_invocation(word, rest[:index] + rest[index + 1:])
        return None, []
    if subcommand == "mutants":
        if rest and set(rest) <= MUTANTS_QUERY_ARGUMENTS and not break_is_set("mutants-help-counted"):
            return None, []   # 只打帮助或版本，不起 cargo test
        return "test", rest
    return None, []


def alias_of_subcommand(command, environment):
    """子命令是 --config 的 alias.<名> 或环境变量 CARGO_ALIAS_<名> 定的别名时交一句（展开之后是什么、带什么参数都看不见，当 cargo test 判）；不是交 None。"""
    aliases = {match.group(1) for value in command.configuration_values for match in [ALIAS_CONFIGURATION.match(value)] if match}
    if command.subcommand in aliases:
        return f"--config 里定的别名 {command.subcommand}（展开之后带什么参数看不见）"
    alias_variable = ALIAS_ENVIRONMENT_PREFIX + command.subcommand.upper().replace("-", "_")
    if alias_variable in environment:
        return f"环境变量 {alias_variable} 定的别名 {command.subcommand}（展开之后带什么参数看不见）"
    return None


# 门禁阶段：.claude/gate.d/ 顶层的 *.sh（lib-*.sh 是共用库、lib/ 底下的不是阶段），按文件名认，不按编号认
GATE_STAGE_PATH = re.compile(r"(?:^|/)\.claude/gate\.d/([^/]+\.sh)$")
# cargo 编出来的测试二进制：<target 目录>/[<目标三元组>/]<profile>/deps/<名字>-<16 位十六进制的元数据哈希>，名字里的 - 已换成 _
TEST_BINARY_PATH = re.compile(r"(?:^|/)deps/([A-Za-z0-9_]+)-[0-9a-f]{16}$")


def gate_stage_name(word, directory):
    """命令词是 .claude/gate.d/ 顶层的一道阶段时交回它的文件名，不是交 None。"""
    for candidate in (word, shell_words.resolve_path(directory, word) or ""):
        match = GATE_STAGE_PATH.search(candidate)
        if match and not match.group(1).startswith(GATE_TOP_LIBRARY_PREFIX):
            return match.group(1)
    return None


def located_in(word, directory, relative_directory):
    """命令词（照写的样子，或按当前目录解开之后）是不是正好落在 relative_directory 这一层（相对仓根的写法，按路径尾部比）。"""
    for candidate in (os.path.normpath(word), shell_words.resolve_path(directory, word) or ""):
        parent = os.path.dirname(candidate)
        if parent == relative_directory or parent.endswith("/" + relative_directory):
            return True
    return False


def command_word_expanded(words, variables):
    """命令词带 `$名` / `${名}` 的，用这段命令文本里前面赋过的值与 $HOME 展开（lib_shell_words.py 的 expand_known_variables）；展开得出、不带空白的换上，
    展开不了的原样交回（文件头「命令词写在变量里」）。弄坏开关 command-word-variables-unexpanded 下原样交回。"""
    if not words or "$" not in words[0] or break_is_set("command-word-variables-unexpanded"):
        return list(words)
    value = shell_words.expand_known_variables(words[0], variables or {})
    if not value or any(character.isspace() for character in value):
        return list(words)
    return [value] + list(words[1:])


class SelftestRow(NamedTuple):
    kind: str             # selftest（脚本、钩子、门禁共用库的自证）或 samples（门禁阶段的样本档）
    tools_directory: str  # samples:<目录> 里的目录（样本根里放假工具的那一层，相对样本根）；没写交空串
    reason: str           # # 之后那段
    line: int             # 在表里第几行


def selftest_directories():
    """自证认的那几个目录（SELFTEST_DIRECTORIES）。弄坏开关 task-server-directory-unrecognized 下去掉 task-server、
    prune-server-directory-unrecognized 下去掉 prune-server（登记表里那一层的行算写坏的行，那一层的 --selftest 不当自证）。"""
    if break_is_set("task-server-directory-unrecognized"):
        return tuple(place for place in SELFTEST_DIRECTORIES if place != TASK_SERVER_DIRECTORY)
    if break_is_set("prune-server-directory-unrecognized"):
        return tuple(place for place in SELFTEST_DIRECTORIES if place != PRUNE_SERVER_DIRECTORY)
    return SELFTEST_DIRECTORIES


def device_selftest_layer(relative_directory):
    """相对仓根的目录是不是实验装置那一层（DEVICE_SELFTEST_PARENT 下正好一层）。弄坏开关 device-layer-unrecognized 下一律为假（照旧不认这一层）。"""
    if break_is_set("device-layer-unrecognized"):
        return False
    return os.path.dirname(relative_directory) == DEVICE_SELFTEST_PARENT and os.path.basename(relative_directory) not in ("", ".", "..")


def stage_samples_kind_is_valid(kind):
    """门禁阶段那一行的登记认不认得：samples，或 samples: 后面跟一个相对样本根、不带 .. 与空白的假工具目录。"""
    if kind == STAGE_SAMPLES:
        return True
    if not kind.startswith(STAGE_SAMPLES_WITH_TOOLS):
        return False
    tools = kind[len(STAGE_SAMPLES_WITH_TOOLS):]
    return bool(tools) and not os.path.isabs(tools) and ".." not in tools.split("/") and not any(character.isspace() for character in tools)


@functools.lru_cache(maxsize=8)
def selftest_registry_at(path, modified, size):
    """登记表 → ({键: SelftestRow}, [写坏的行，逐条一句])。modified、size 只当缓存的键（看门狗常驻，表改了要重读）。
    认不出的种类、没有 # 理由或理由太短、不在登记目录里的、重复的（留第一行）不收；弄坏开关 selftest-registry-lenient 下照收、不报。"""
    rows, problems = {}, []
    try:
        with open(path, encoding="utf-8") as handle:
            lines = handle.read().split("\n")
    except (OSError, UnicodeDecodeError) as error:
        return {}, [f"读不了登记表 {path}：{error}"]
    lenient = break_is_set("selftest-registry-lenient")
    for number, line in enumerate(lines, start=1):
        if not line.strip() or line.lstrip().startswith("#"):
            continue
        fields = line.split(None, 2)
        key, kind = fields[0], fields[1] if len(fields) > 1 else ""
        reason = fields[2][1:].strip() if len(fields) > 2 and fields[2].startswith("#") else ""
        stage_row = os.path.dirname(key) == GATE_STAGE_DIRECTORY and not gate_top_library_key(key)
        wrong = []
        if stage_row:
            if not stage_samples_kind_is_valid(kind):
                wrong.append(f"门禁阶段的登记「{kind}」认不出（只认 {STAGE_SAMPLES}、{STAGE_SAMPLES_WITH_TOOLS}<相对样本根、不带 .. 的假工具目录>）")
        elif kind != SELFTEST_KIND:
            wrong.append(f"登记「{kind}」认不出（只认 {SELFTEST_KIND}；门禁阶段的样本档写在 .claude/gate.d/<阶段>.sh 那一行）")
        if len(reason) < SELFTEST_REASON_MINIMUM:
            wrong.append(f"# 之后的理由不到 {SELFTEST_REASON_MINIMUM} 个字")
        if (not stage_row and os.path.dirname(key) not in selftest_directories() and not device_selftest_layer(os.path.dirname(key))
                and not gate_top_library_key(key)):
            wrong.append(f"不在 {'、'.join(selftest_directories())} 之一的这一层，也不在实验装置那一层 {DEVICE_SELFTEST_PARENT}/<装置>"
                         f"（门禁阶段的样本档写在 {GATE_STAGE_DIRECTORY} 这一层，那一层以 {GATE_TOP_LIBRARY_PREFIX} 起头的共用库照脚本登记）")
        if key in rows:
            wrong.append(f"与第 {rows[key].line} 行重复")
        if wrong and not lenient:
            problems.append(f"第 {number} 行 {key}：{'；'.join(wrong)}")
            continue
        if key not in rows:
            tools = kind[len(STAGE_SAMPLES_WITH_TOOLS):] if stage_row and kind.startswith(STAGE_SAMPLES_WITH_TOOLS) else ""
            rows[key] = SelftestRow(STAGE_SAMPLES if stage_row else SELFTEST_KIND, tools, reason, number)
    return rows, problems


def gate_top_library_key(key):
    """登记表的键（相对仓根）是不是门禁目录顶层以 lib- 起头的共用库：是就照脚本的自证判，不当门禁阶段的样本档行。
    弄坏开关 gate-top-library-as-stage 下一律为假（这一层的键都当样本档行）。"""
    if break_is_set("gate-top-library-as-stage"):
        return False
    return os.path.dirname(key) == GATE_STAGE_DIRECTORY and os.path.basename(key).startswith(GATE_TOP_LIBRARY_PREFIX)


def read_selftest_registry(path):
    """登记表此刻的内容（selftest_registry_at 按修改时刻与大小缓存）；文件不在交 ({}, [一句])。"""
    try:
        status = os.stat(path)
    except OSError as error:
        return {}, [f"读不了登记表 {path}：{error.strerror}"]
    return selftest_registry_at(path, status.st_mtime_ns, status.st_size)


def selftest_registry_problems(path, root):
    """登记表写坏的行，加上指到 root 下不存在的文件的行，逐条一句；空表是没毛病。弄坏开关 selftest-registry-lenient 下只报读不了。"""
    rows, problems = read_selftest_registry(path)
    problems = list(problems)
    for key, row in sorted(rows.items(), key=lambda item: item[1].line):
        if not os.path.isfile(os.path.join(root, key)) and not break_is_set("selftest-registry-lenient"):
            problems.append(f"第 {row.line} 行 {key}：{root} 下没有这个文件")
    return problems


def selftest_registry_key(word, directory):
    """命令词正好落在登记目录之一（照写的样子，或按当前目录解开）时交登记表里的键「目录/文件名」；落在实验装置那一层的交
    「<DEVICE_SELFTEST_PARENT>/<装置>/文件名」（按路径尾部比，同 located_in）；都不是交 None。"""
    for place in selftest_directories():
        if located_in(word, directory, place):
            return f"{place}/{os.path.basename(os.path.normpath(word))}"
    if located_in(word, directory, GATE_STAGE_DIRECTORY):
        key = f"{GATE_STAGE_DIRECTORY}/{os.path.basename(os.path.normpath(word))}"
        if gate_top_library_key(key):
            return key   # 门禁目录顶层以 lib- 起头的共用库（文件头「自证」）
    for candidate in (os.path.normpath(word), shell_words.resolve_path(directory, word) or ""):
        parent = os.path.dirname(candidate)
        layer = f"{DEVICE_SELFTEST_PARENT}/{os.path.basename(parent)}"
        grandparent = os.path.dirname(parent)
        if (grandparent == DEVICE_SELFTEST_PARENT or grandparent.endswith("/" + DEVICE_SELFTEST_PARENT)) and device_selftest_layer(layer):
            return f"{layer}/{os.path.basename(candidate)}"
    return None


def file_sha256(path):
    """文件的 sha256（十六进制）；不是普通文件、读不了交 None。"""
    try:
        digest = hashlib.sha256()
        with open(path, "rb") as handle:
            for block in iter(lambda: handle.read(1 << 20), b""):
                digest.update(block)
        return digest.hexdigest()
    except OSError:
        return None


def selftest_registered_file(word, directory, key):
    """命令词解到的真实路径（符号链接按指到的算）是不是登记表所在仓（SELFTEST_REGISTRY_ROOT）里键指的那一份文件；相对路径而当前目录认不出的交 False。
    弄坏开关 selftest-registry-by-path-tail 下一律为真（照旧只按路径尾部认）。"""
    if break_is_set("selftest-registry-by-path-tail"):
        return True
    resolved = shell_words.resolve_path(directory, os.path.expanduser(word))
    if resolved is None:
        return False
    return os.path.realpath(resolved) == os.path.realpath(os.path.join(SELFTEST_REGISTRY_ROOT, key))


def registered_selftest(word, arguments, directory):
    """命令是不是登记表里登记过的自证：是交那一行（SelftestRow），不是交 None。
    脚本、钩子、门禁目录顶层 lib- 起头的共用库只认参数正好是 ["--selftest"]，.claude/gate.d/lib/ 下的共用库直接执行就是跑自证，不带参数或单独一个 --selftest 都算；命令词要解到登记表所在仓里那一份文件
    （selftest_registered_file）；命令词带通配的不当自证（弄坏开关 selftest-pattern-words-judged 下照判）；派发起法与 task-server/task submit 的 `--`
    之后那条里的 --selftest 是里面那条的，不是外面这一条自己的自证；实验装置那一层只认登记了的单独 --selftest。
    弄坏开关 selftest-extra-arguments-allowed 下带了别的参数也算单独。"""
    if SELFTEST_PATTERN_CHARACTERS & set(word) and not break_is_set("selftest-pattern-words-judged"):
        return None   # 通配写出的命令词：展开之前判不出起的是哪一份（case 的模式行切成命令时也是这样的词），不当自证
    key = selftest_registry_key(word, directory)
    if key is None:
        return None
    if command_under_dispatch_launcher([word] + list(arguments), directory) is not None:
        return None   # 派发起法 `… -- <命令…>`：--selftest 落在里面那条命令里，不是派发器自己的自证（里面那条由调用方剥开另判）
    if not break_is_set("task-server-submit-as-selftest") and task_server_submission([word] + list(arguments), directory) is not None:
        return None   # task-server/task submit -- <命令…>：--selftest 落在交给服务的那条里，不是客户端自己的自证
    gate_library = os.path.dirname(key) == GATE_LIBRARY_DIRECTORY
    if not gate_library and "--selftest" not in arguments:
        return None
    alone = arguments in ([], ["--selftest"]) if gate_library else arguments == ["--selftest"]
    if break_is_set("selftest-extra-arguments-allowed"):
        alone = True
    row = read_selftest_registry(SELFTEST_REGISTRY_PATH)[0].get(key)
    if row is None or not alone or not selftest_registered_file(word, directory, key):
        return None
    if device_selftest_layer(os.path.dirname(key)) and arguments != ["--selftest"]:
        return None   # 实验装置那一层只认登记了的单独 --selftest（文件头「实验装置那一层」）
    return row


def stage_samples_row(stage):
    """登记表里 .claude/gate.d/<阶段> 那一行（SelftestRow）；没登记交 None。"""
    return read_selftest_registry(SELFTEST_REGISTRY_PATH)[0].get(f"{GATE_STAGE_DIRECTORY}/{stage}")


def sample_index_entries(repository, relative_directory):
    """repository 的暂存区（HEAD 加暂存）里 relative_directory 底下的每个文件：({相对它的路径: (模式, 对象号)}, "")；读不了交 (None, 一句原因)。
    清掉 GIT_ 开头的环境变量再起 git（GIT_DIR、GIT_INDEX_FILE 漏进来会读到别的仓或别的暂存区）。"""
    environment = {name: value for name, value in os.environ.items() if not name.startswith("GIT_")}
    try:
        done = subprocess.run(["git", "--literal-pathspecs", "-C", repository, "ls-files", "-s", "-z", "--", relative_directory + "/"],
                              capture_output=True, env=environment, timeout=60)
    except (OSError, subprocess.SubprocessError) as error:
        return None, f"读不了 {repository} 的暂存区（{error}）"
    if done.returncode != 0:
        return None, f"读不了 {repository} 的暂存区（git ls-files 退 {done.returncode}：{done.stderr.decode(errors='replace').strip()[:200]}）"
    entries, prefix = {}, relative_directory + "/"
    for record in done.stdout.split(b"\0"):
        if not record:
            continue
        metadata, _, path = record.partition(b"\t")
        mode, object_name, stage_number = metadata.decode().split()
        name = os.fsdecode(path)
        if stage_number != "0":
            return None, f"{repository} 的暂存区里 {name} 在合并冲突里"
        entries[name[len(prefix):]] = (mode, object_name)
    return entries, ""


def git_blob_name(data, object_name_length):
    """一段内容当 git 的 blob 时的对象号（对象号 64 位的是 sha256 仓，40 位的是 sha1 仓）。"""
    algorithm = hashlib.sha256 if object_name_length == 64 else hashlib.sha1
    return algorithm(b"blob %d\0" % len(data) + data).hexdigest()


def sample_directory_changes(sample_directory, stage):
    """喂的样本目录（<阶段目录>/fixtures/<阶段>）与 SAMPLE_INDEX_REPOSITORY 的 HEAD 加暂存区里那一道的样本比：多出的（没跟踪的新文件，连同 setup.sh）、
    少了的、内容或执行位（符号链接的指向）不同的、不是普通文件的，逐个交一句（相对样本目录的路径与哪里不同）；读不了暂存区交 [一句]。
    弄坏开关 stage-samples-fixtures-unchecked 下一律交 []。"""
    if break_is_set("stage-samples-fixtures-unchecked"):
        return []
    entries, problem = sample_index_entries(SAMPLE_INDEX_REPOSITORY, f"{GATE_STAGE_DIRECTORY}/fixtures/{stage}")
    if entries is None:
        return [problem]
    changed, seen = [], set()
    for current, directories, files in os.walk(sample_directory):
        links = [name for name in directories if os.path.islink(os.path.join(current, name))]
        directories[:] = sorted(name for name in directories if name not in links)
        for name in sorted(files + links):
            path = os.path.join(current, name)
            relative = os.path.relpath(path, sample_directory)
            seen.add(relative)
            entry = entries.get(relative)
            if entry is None:
                changed.append(f"{relative}（暂存区里没有）")
                continue
            try:
                information = os.lstat(path)
                if stat.S_ISLNK(information.st_mode):
                    data, actual_mode = os.fsencode(os.readlink(path)), "120000"
                elif stat.S_ISREG(information.st_mode):
                    with open(path, "rb") as handle:
                        data = handle.read()
                    actual_mode = "100755" if information.st_mode & stat.S_IXUSR else "100644"
                else:
                    changed.append(f"{relative}（不是普通文件）")
                    continue
            except OSError as error:
                changed.append(f"{relative}（读不了：{error.strerror}）")
                continue
            if actual_mode != entry[0]:
                changed.append(f"{relative}（模式 {actual_mode}，暂存区里是 {entry[0]}）")
            elif git_blob_name(data, len(entry[1])) != entry[1]:
                changed.append(f"{relative}（内容与暂存区不同）")
    changed += [f"{relative}（暂存区里有、喂的样本里没有）" for relative in sorted(set(entries) - seen)]
    return changed


class StageSelftestArguments(NamedTuple):
    directories: list   # 当阶段目录的词，照写的样子（stage-selftest.sh 取最后一个）
    items: list | None  # --item 的值，照写的样子（缺值记空串）；一个 --item 都没写是 None
    list_only: bool     # 带了 --list-items


def stage_selftest_arguments(arguments):
    """照 stage-selftest.sh 自己的参数循环切：先照它开头的 preflight 摘掉每一个整词 --force；--item 后面还有词、又不是空串的，取它当值（`--item --list-items` 的值
    是 --list-items），缺值或值是空串记一个空名（它自己退 2）；--list-items 只列；别的词都当阶段目录。"""
    words = [argument for argument in arguments if argument != "--force"]
    directories, items, list_only, position = [], [], False, 0
    while position < len(words):
        word = words[position]
        if word == STAGE_SELFTEST_ITEM_OPTION:
            if position + 1 < len(words) and words[position + 1]:
                items.append(words[position + 1])
                position += 2
            else:
                items.append("")
                position += 1
            continue
        if word == STAGE_SELFTEST_LIST_OPTION:
            list_only = True
        else:
            directories.append(word)
        position += 1
    return StageSelftestArguments(directories, items or None, list_only)


def stage_selftest_directory(word, written, directory):
    """stage-selftest.sh 喂的阶段目录：(实际路径, "")；判不出交 (None, 一句原因)。written 是参数里当阶段目录的那个词，没写交 None（它自己算的那一个：<脚本所在包的根>/.claude/gate.d）。"""
    if written is not None and UNRESOLVED_WORD_CHARACTERS & set(written):
        return None, f"阶段目录 {written} 写在变量或通配里，判不出它下面有哪些阶段"
    if written is not None:
        gate_directory = shell_words.resolve_path(directory, os.path.expanduser(written))
    else:
        script = shell_words.resolve_path(directory, word)
        gate_directory = os.path.join(os.path.dirname(os.path.dirname(os.path.realpath(script))), ".claude", "gate.d") if script else None
    if gate_directory is None or not os.path.isdir(gate_directory):
        return None, f"阶段目录 {written if written is not None else '（它默认的那一个）'} 此刻不在，判不出它下面有哪些阶段"
    return gate_directory, ""


def stage_feed_problem(gate_directory, stage):
    """stage-selftest.sh 喂的阶段目录里配了样本的一道放不放行（文件头「门禁样本」）：不放行交一句，放行交 ""。带不带 --item 都用这一份。
    要登记在表里，样本要与 SAMPLE_INDEX_REPOSITORY 的 HEAD 加暂存区逐字节相同。"""
    if stage_samples_row(stage) is None:
        return f"{stage} 的样本档没在登记表里"
    changes = sample_directory_changes(os.path.join(gate_directory, "fixtures", stage), stage)
    if changes:
        return (f"{stage} 的样本与 {SAMPLE_INDEX_REPOSITORY} 的 HEAD 加暂存区不同（{len(changes)} 处：{'、'.join(changes[:3])}"
                f"{' 等' if len(changes) > 3 else ''}）")
    return ""


def stage_selftest_is_own_copy(word, directory):
    """命令词解到的真实路径正是本仓那一份 stage-selftest.sh（--item / --list-items 信的是它只喂点名的几道、只列不喂）。
    弄坏开关 stage-items-script-by-name 下一律为真（照旧只按名字与位置认）。"""
    if break_is_set("stage-items-script-by-name"):
        return True
    resolved = shell_words.resolve_path(directory, os.path.expanduser(word))
    return resolved is not None and os.path.realpath(resolved) == os.path.realpath(os.path.join(HOOK_REPOSITORY, KNOWN_SCRIPT_LOCATIONS[STAGE_SELFTEST_SCRIPT]))


def stage_selftest_item_problems(word, parsed, directory, environment=None):
    """stage-selftest.sh 带 --item 或 --list-items（文件头「门禁样本」）：不放行的原因一句，放行交 ""。只判点名后真喂的那几道；点名里没有一道是配了样本的阶段，
    它自己退 2、不喂样本，放行。弄坏开关 stage-items-unjudged 下点名的几道不判，stage-directory-pwd-unexpanded 下 $PWD 不换。"""
    if not stage_selftest_is_own_copy(word, directory):
        return (f"{word} 解到的不是本仓那一份 {os.path.join(HOOK_REPOSITORY, KNOWN_SCRIPT_LOCATIONS[STAGE_SELFTEST_SCRIPT])}：带 --item / --list-items 时信的是"
                "它只喂点名的几道、只列不喂，别处路径尾部相同的拷贝不信；跑本仓那一份，写它在本仓的字面路径")
    if parsed.list_only:
        return ""   # 只列可点名的阶段，不喂样本
    if len(parsed.directories) > 1:
        return f"参数里有 {len(parsed.directories)} 个当阶段目录的词（{'、'.join(parsed.directories)}），认不出喂的是哪一个"
    written = parsed.directories[0] if parsed.directories else ""
    if directory is not None and "PWD" not in (environment or {}) and not break_is_set("stage-directory-pwd-unexpanded"):
        written = WORKING_DIRECTORY_VARIABLE.sub(lambda _: directory, written, count=1)
    gate_directory, problem = stage_selftest_directory(word, written or None, directory)
    if problem:
        return problem
    if break_is_set("stage-items-unjudged"):
        return ""
    unresolved = [item for item in parsed.items if UNRESOLVED_WORD_CHARACTERS & set(item)]
    if unresolved:
        return f"--item {unresolved[0]} 写在变量或通配里，判不出点的是哪几道"
    # 照 stage-selftest.sh 的切法：一个值里用空白隔开的几个名字各算一道，每个名字摘掉结尾的 .sh 再比；
    # 只认带 .sh 的会把不带的点名当成「没这一道」放过去
    requested_stage_names = {word[:-len(".sh")] if word.endswith(".sh") else word for item in parsed.items for word in item.split()}
    fed = [stage for stage in sorted(os.listdir(gate_directory))
           if stage.endswith(".sh") and not stage.startswith(".") and os.path.isdir(os.path.join(gate_directory, "fixtures", stage))
           and stage[:-len(".sh")] in requested_stage_names]
    return "；".join(problem for problem in (stage_feed_problem(gate_directory, stage) for stage in fed) if problem)


def stage_selftest_problems(word, arguments, directory, environment=None):
    """stage-selftest.sh 喂的样本不盯的原因（文件头「门禁样本」），一句；盯交 ""。带 --item / --list-items 的走 stage_selftest_item_problems
    （弄坏开关 stage-items-as-directory 下不认这两个参数，照不带的判）；不带的，命令词要解到本仓那一份 stage-selftest.sh（stage_selftest_is_own_copy，
    弄坏开关 stage-feed-script-by-name 下不核），阶段目录里配了样本的每一道都判。"""
    if break_is_set("stage-samples-ignored"):
        return "弄坏开关 stage-samples-ignored：门禁样本一律不认"
    parsed = stage_selftest_arguments(arguments)
    if (parsed.items is not None or parsed.list_only) and not break_is_set("stage-items-as-directory"):
        return stage_selftest_item_problems(word, parsed, directory, environment)
    if not break_is_set("stage-feed-script-by-name") and not stage_selftest_is_own_copy(word, directory):
        return (f"{word} 解到的不是本仓那一份 {os.path.join(HOOK_REPOSITORY, KNOWN_SCRIPT_LOCATIONS[STAGE_SELFTEST_SCRIPT])}：盯的信的是它只喂样本、"
                "别处路径尾部相同的拷贝不信；跑本仓那一份，写它在本仓的字面路径")
    positional = [argument for argument in arguments if not argument.startswith("-")]
    if len(positional) > 1:
        return f"参数里有 {len(positional)} 个像阶段目录的词，认不出喂的是哪一个"
    gate_directory, problem = stage_selftest_directory(word, positional[0] if positional else None, directory)
    if problem:
        return problem
    return "；".join(problem for problem in (stage_feed_problem(gate_directory, stage) for stage in sorted(os.listdir(gate_directory))
                                              if stage.endswith(".sh") and os.path.isdir(os.path.join(gate_directory, "fixtures", stage))) if problem)


# 跑编译出来的代码的 cargo 子命令（t、r 是 test、run 的内置简写）；只编不跑的 build、check、clippy 不在内
COMPILED_CODE_SUBCOMMANDS = {"test", "t", "run", "r", "bench"}
# cargo 编出来的二进制直接执行：<名字里带 target 的目录>/[<目标三元组>/]<debug|release>/[deps/]<名字>；名字里不许有点（排掉 .d、.rlib、.so）。
# CARGO_TARGET_DIR 设成名字里不带 target 的目录、再直接执行里面的二进制，认不出（测试二进制按 TEST_BINARY_PATH 照认）
BUILT_BINARY_PATH = re.compile(r"(?:^|/)[^/]*target[^/]*/(?:[^/]+/)?(?:debug|release)/(?:deps/)?[A-Za-z0-9_][A-Za-z0-9_-]*$")


def runs_compiled_code(words, directory):
    """一条剥掉前缀与包装的命令会不会跑编译出来的代码（只许经 task-server/task 起的一类）：会就交一句说明，不会交 None。
    cargo 的子命令按 test_invocation 认（nextest run、miri、llvm-cov、hack、mutants 同样算），另认 bench 与别名（展开之后看不见，当会跑）。"""
    if os.path.basename(words[0]) == "cargo":
        found = cargo_subcommand(words[1:], directory)
        if found is None:
            return None
        invocation, rest = test_invocation(found.subcommand, found.rest)
        if found.subcommand not in COMPILED_CODE_SUBCOMMANDS and invocation is None and not alias_of_subcommand(found, {}):
            return None
        options = rest[:rest.index("--")] if "--" in rest else rest
        if "--no-run" in options:
            return None
        return f"cargo {found.subcommand}"
    if TEST_BINARY_PATH.search(words[0]) or BUILT_BINARY_PATH.search(words[0]):
        return f"直接执行 cargo 编出来的二进制 {words[0]}"
    return None


# 不编、不链接的 cargo 子命令（只查版本、清单、帮助，改源码格式，动依赖与锁文件，删编译目录）；别的子命令、别名与插件一律当会编（展开之后、插件里起什么看不见）
CARGO_NON_BUILDING_SUBCOMMANDS = {"fmt", "metadata", "tree", "clean", "version", "help", "locate-project", "pkgid", "read-manifest", "verify-project",
                                  "search", "update", "generate-lockfile", "fetch", "vendor", "new", "init", "add", "remove", "info", "config", "report",
                                  "owner", "login", "logout", "yank", "uninstall"}
# 自己在里面起会编的 cargo、里面不套内存包装的仓内脚本（按文件名认：别处树里的拷贝起的同样是 cargo）；单独一个 --selftest 用的是假 cargo，不算
SCRIPTS_STARTING_BUILDS = {"compile-then-swap.py"}


def starts_a_build(words, directory):
    """一条剥掉前缀与包装的命令会不会起编译：会编的 cargo（CARGO_NON_BUILDING_SUBCOMMANDS 之外的子命令，test 带 --no-run 的也算），
    或 SCRIPTS_STARTING_BUILDS 里的脚本（python 起的、直接执行的；单独一个 --selftest 不算）。会就交一句说明，不会交 None。
    project-code-guard.sh 拿它认「跑项目代码」的一种（不经 task-server/task 起的拒，经没经内存包装都一样），主 agent 与子 agent 一样判、不看目录。
    弄坏开关 build-subcommands-unjudged 下 cargo 一律交 None；build-scripts-unjudged 下那几份脚本交 None。"""
    name = os.path.basename(words[0])
    if name == "cargo":
        found = cargo_subcommand(words[1:], directory)
        if found is None or found.subcommand in CARGO_NON_BUILDING_SUBCOMMANDS or break_is_set("build-subcommands-unjudged"):
            return None
        return f"cargo {found.subcommand}"
    script_index = 0
    if shell_words.PYTHON_NAME.match(name):
        invocation = shell_words.python_invocation(words[1:])
        if invocation.code_string is not None or invocation.module is not None or invocation.script_index is None:
            return None
        script_index = invocation.script_index + 1
    script = os.path.basename(words[script_index])
    if script not in SCRIPTS_STARTING_BUILDS or words[script_index + 1:] == ["--selftest"] or break_is_set("build-scripts-unjudged"):
        return None
    return f"{script}（它在里面起会编的 cargo、里面不套内存包装）"


# ── 只认经 task-server/task 起（文件头「经 task-server 起」；records/2026-10-30-编译绕过内存总上限.md 批 3b ① ③） ──
TASK_SERVER_CLIENT = "task-server/task"
TASK_SERVER_CLIENT_SUBCOMMANDS = frozenset({"submit", "confirm", "wait", "status", "cancel", "log", "clean"})   # confirm：交任务的二次确认（task-server/task_server.py 文件头 ⑦），只给主 agent
# 子 agent 起提交客户端只许这两个只读的子命令（文件头「经 task-server 起」subagent_task_server_refusal）
SUBAGENT_TASK_SERVER_SUBCOMMANDS = frozenset({"status", "log"})
# 子 agent 被拒时的出路：project-code-guard.sh 与 run-scope-guard.sh 的拒绝说明都照这一句写
SUBAGENT_HAND_BACK_WAY_OUT = ("子 agent 不往统一任务服务交任务，task-server/task 的 submit、wait、cancel、clean 都不起，run_in_background 也不起（只读的 status、log 照用）："
                              "把要跑的命令照共用约束 .claude/agent-common.md「不做」一节「子 agent 不交任务」那一条拼成 submit 的整条（范围声明写在 submit 前面），"
                              "写进报告「要主 agent 跑的命令」一节交回，写明在哪棵树里起、跑完看哪几行，由主 agent 交服务")
TASK_SERVER_SUBMIT_DIRECTORY_OPTION = "--cwd"
TASK_SERVER_LIBRARY_PATH = os.path.join(HOOK_REPOSITORY, "task-server", "task_server_lib.py")
# 直接跑、不经 task-server 的本仓脚本只有这两张表（钩子与看门狗都引这一处，别处不另抄）。
# 相对仓根的路径 → 放行的参数形态："any" 是除带 --selftest 之外都放；("first", {词…}) 是第一个参数是其中之一；("exact", (词…)) 是参数恰好这几个
# 谁都能直接跑的文本工具：「文本工具直接跑」（records/2026-10-30-编译绕过内存总上限.md 第 85 行），另两样见同一份记录问题清单 #7，
# 加提交客户端自己与 control check
TEXT_TOOLS_RUN_DIRECTLY = {
    "scripts/kb/replace-once.py": "any",
    "scripts/kb/replace-batch.py": "any",
    "scripts/kb/insert-row.py": "any",
    "scripts/hosts/work-directory-ownership.sh": ("first", frozenset({"claim"})),
    "scripts/kb/kb-spec-check.py": "any",
    "scripts/kb/stale-candidates.py": "any",
    "scripts/kb/kb-sections.py": "any",
    "scripts/agents/checklist-specs.py": "any",
    "scripts/agents/cite-check.py": "any",
    "scripts/kb/claim-experiment.sh": "any",
    "scripts/kb/relabel-item.py": "any",          # 主 agent 定的（records/2026-10-30-编译绕过内存总上限.md 问题清单 #7）：kb-scribe 定义点名的改标签工具
    "scripts/agents/copy-originals.py": "any",    # 同上：共用约束「写」一节教的备份原文件写法
    "scripts/kb/quote-kb.py": "any",              # 与 checklist-specs.py 同类的 kb 抽取工具
    "scripts/kb/append-record.py": "any",          # records/2026-10-29-subagent拆分提案.md 一百七十六节 ⑭：bash-command-detector.sh ⑲ 拒 Bash 往 records/ 追加、出路就是它，与 insert-row.py 同类的写前复核工具，主 agent 与子 agent 都直接跑
    "scripts/kb/ledger-append-section.py": "any",  # 主 agent 定的（records/2026-10-29-一百三十七节那一轮调度记录.md 批次 #19）：往现行拆分提案账本追加一节，取号与写入一步做完（records/2026-10-29-subagent拆分提案.md 第一百五十六节），主 agent 与子 agent 都直接跑
    "scripts/hosts/minimal-copy.py": "any",        # 副本拷贝不全的问题必须根治（records/2026-10-29-subagent拆分提案.md 第一百九十九节）：建仓副本只用它（范围按闭包现算），只 rsync、不编，主 agent 与子 agent 都直接跑
    "scripts/hosts/lib_repository_copy.py": ("first", frozenset({"entries"})),  # 同上：只打算出的副本闭包，不拷不编
    ".claude/singlefs-ai-sop/scripts/gate-overlap.py": ("exact", ("--list",)),
    TASK_SERVER_CLIENT: ("first", TASK_SERVER_CLIENT_SUBCOMMANDS),
    "task-server/control": ("first", frozenset({"check"})),
}
# 只给主 agent 直接跑的会话工具：起在服务的单元里就丢了会话身份（停进程按会话认、定时提醒与看门狗按会话号认）或停不了服务自己（control 的装与起停）
MAIN_AGENT_TOOLS_RUN_DIRECTLY = {
    ".claude/singlefs-ai-sop/scripts/proc.py": "any",
    "scripts/agents/remind.sh": "any",
    "scripts/agents/watch.sh": "any",
    "scripts/agents/agent-watch.py": "any",
    "scripts/agents/lib_session_registry.py": "any",  # 会话登记表现场查（by-session、by-name）：读本机 ~/.claude/sessions 与 /proc，起在服务里会派到第二台、查到的是那一台的登记表
    "scripts/agents/dispatch-prompt.py": "any",
    "task-server/control": ("first", frozenset({"install", "start", "stop", "watch", "clear-queue", "pause", "resume"})),
    # 判定库服务的装与起停、监听：装的是本机用户级 systemd 单元，起在任务服务里就装到别处或丢了会话
    "prune-server/control": ("first", frozenset({"install", "start", "stop", "check", "watch"})),
    # 用户「模拟任务来跑」：服务自证起的是临时目录里的模拟服务与假主机，不进真队列等真任务
    "task-server/task_server_lib.py": ("exact", ("--selftest",)),
}
# 本仓脚本文件头的准入声明（.claude/singlefs-ai-sop/rules/preflight-discipline.md）：改名拷走的照认是本仓的代码
PROJECT_SCRIPT_HEADER = re.compile(rb"^#\s*(?:admission|run-condition):", re.MULTILINE)
PROJECT_HEAD_BYTES = 4096
# python3 -m 起单测的模块（lib_shell_words 交出来的是 <模块>.py）
TEST_RUNNER_MODULE_FILES = frozenset({"pytest.py", "unittest.py", "doctest.py", "nose2.py"})
# systemctl 只读的子命令：别的子命令（stop、kill、disable、mask、edit、restart、reset-failed、set-property 这类）点到 task-server 的单元就拒
SYSTEMCTL_READ_ONLY_VERBS = frozenset({"status", "show", "cat", "is-active", "is-enabled", "is-failed", "list-units", "list-unit-files",
                                       "list-dependencies", "list-timers", "list-sockets", "list-jobs", "help", "show-environment"})
# 不点名单元也停掉整个用户级服务管理器（连同 task-server）的子命令
SYSTEMCTL_MANAGER_VERBS = frozenset({"exit", "halt", "poweroff", "reboot", "kexec", "soft-reboot", "daemon-reexec", "isolate", "emergency", "rescue", "default", "switch-root"})
SYSTEMCTL_OPTIONS_WITH_VALUE = frozenset({"-t", "--type", "-p", "--property", "-H", "--host", "-M", "--machine", "-n", "--lines", "-o", "--output", "-s", "--signal",
                                          "--kill-whom", "--kill-value", "--state", "--root", "--image", "--job-mode", "--what", "--timestamp", "--message",
                                          "--drop-in", "--when", "--preset-mode", "--legend", "--check-inhibitors", "--boot-loader-menu", "--boot-loader-entry",
                                          "--reboot-argument"})
# 服务状态目录下放编译目录的那一层（派发记录点名它；task_server_lib.py 此刻把编译目录放在 TASK_SERVER_BUILD_DIRECTORY，两处都只经 task clean 删）
TASK_SERVER_STATE_BUILDS_NAME = "builds"
# 删文件与目录的命令词（命令位置上的）、find 里删东西的写法、python 代码里删的写法：点到服务的编译目录就拒（build_directory_removal）
REMOVAL_COMMAND_NAMES = frozenset({"rm", "rmdir", "unlink", "shred"})
FIND_EXECUTION_ACTIONS = frozenset({"-exec", "-execdir", "-ok", "-okdir"})
FIND_LEADING_OPTIONS = frozenset({"-H", "-L", "-P"})
PYTHON_REMOVAL_CALL = re.compile(r"\b(?:shutil\.rmtree|os\.(?:remove|unlink|rmdir|removedirs)|\.unlink|\.rmdir)\s*\(")
# python 代码里载入 task-server 的共用库（拿它的密钥函数、直接开库）
TASK_SERVER_LIBRARY_IMPORT = re.compile(r"\bimport\s+task_server_lib\b|\bfrom\s+task_server_lib\s+import\b|import_module\(\s*['\"]task_server_lib|task_server_lib\.py['\"]")


class ProjectRun(NamedTuple):
    detail: str       # 起的是哪一份（相对仓根的路径，认不出相对路径的照写的样子）
    allowed: str      # "nobody"：要经 task-server 起；"main-agent"：主 agent 能直接跑（MAIN_AGENT_TOOLS_RUN_DIRECTLY）；"anyone"：谁都能直接跑（TEXT_TOOLS_RUN_DIRECTLY）
    real_path: str    # 起的那份文件的真实路径（调用方拿它认名单上的脚本正文里的那几行）


@functools.lru_cache(maxsize=1)
def project_top_directories():
    """本仓仓根下的目录名（target、.git 除外）：认仓副本里的文件用（project_relative_path）。"""
    try:
        return frozenset(name for name in os.listdir(HOOK_REPOSITORY)
                         if name not in ("target", ".git") and os.path.isdir(os.path.join(HOOK_REPOSITORY, name)))
    except OSError:
        return frozenset()


def project_relative_path(real_path):
    """真实路径落在本仓里的交相对仓根的路径；落在别处、而从某一层上级往下的那一段（头一段是本仓仓根下的目录）在本仓里有这份文件的
    （仓副本、git worktree、拷走的整棵树）交那一段；都不是交 None。"""
    if real_path == HOOK_REPOSITORY or real_path.startswith(HOOK_REPOSITORY + os.sep):
        return os.path.relpath(real_path, HOOK_REPOSITORY)
    parent = os.path.dirname(real_path)
    while parent and parent != os.path.dirname(parent):
        relative = os.path.relpath(real_path, parent)
        if os.sep in relative and relative.split(os.sep, 1)[0] in project_top_directories() and os.path.isfile(os.path.join(HOOK_REPOSITORY, relative)):
            return relative
        parent = os.path.dirname(parent)
    return None


def has_project_header(real_path):
    try:
        with open(real_path, "rb") as handle:
            return bool(PROJECT_SCRIPT_HEADER.search(handle.read(PROJECT_HEAD_BYTES)))
    except OSError:
        return False


def argument_form_allows(form, arguments):
    if form == "any":
        return "--selftest" not in arguments
    kind, values = form
    if kind == "first":
        return bool(arguments) and arguments[0] in values
    return tuple(arguments) == tuple(values)


def same_as_own_copy(relative, real_path):
    """真实路径是本仓那一份（relative 相对仓根），或与它逐字节相同的拷贝。"""
    own = os.path.join(HOOK_REPOSITORY, relative)
    return real_path == os.path.realpath(own) or (os.path.isfile(own) and filecmp.cmp(real_path, own, shallow=False))


def directly_run_level(relative, real_path, arguments):
    """名单上直接跑的脚本：谁都能跑交 "anyone"，只给主 agent 交 "main-agent"，不在名单上（或参数不合、内容与本仓那一份不同）交 "nobody"。
    弄坏开关 text-tools-refused 下 TEXT_TOOLS_RUN_DIRECTLY 不认，main-agent-tools-refused 下 MAIN_AGENT_TOOLS_RUN_DIRECTLY 不认。"""
    if relative is None or not same_as_own_copy(relative, real_path):
        return "nobody"
    form = TEXT_TOOLS_RUN_DIRECTLY.get(relative)
    if form is not None and argument_form_allows(form, arguments) and not break_is_set("text-tools-refused"):
        return "anyone"
    form = MAIN_AGENT_TOOLS_RUN_DIRECTLY.get(relative)
    if form is not None and argument_form_allows(form, arguments) and not break_is_set("main-agent-tools-refused"):
        return "main-agent"
    return "nobody"


def launched_file(words, directory, launcher):
    """一条剥掉前缀与包装的命令（lib_shell_words 的 CommandAtPosition 的词与 launcher）起的文件的真实路径：bash / sh / source / python 起的、
    直接执行的（命令词带 /）；盘上不在、读不了、认不出当前目录的，shell 启动文件与 awk -f 这类程序文件，PATH 上找的命令交 None。"""
    if not words or launcher in ("startup", "program"):
        return None
    word = words[0]
    if launcher is None and "/" not in word:
        return None
    if not os.path.isabs(os.path.expanduser(word)) and not directory:
        return None
    path = shell_words.resolve_path(directory, os.path.expanduser(word))
    if path is None or not os.path.isfile(path):
        return None
    return os.path.realpath(path)


def project_code_use(words, directory, launcher=None):
    """一条剥掉前缀与包装的命令是不是在跑本仓的代码（只许经 task-server/task 起的那一类，文件头「经 task-server 起」）：是交 ProjectRun，不是交 None。
    认的：起的文件（launched_file）真实路径落在本仓里、或是本仓某份文件的拷贝（project_relative_path）、或开头 4 KiB 里有行首 `# admission:` / `# run-condition:`；
    `python3 -m pytest`、`-m unittest` 这类起单测的。名单上直接跑的（TEXT_TOOLS_RUN_DIRECTLY、MAIN_AGENT_TOOLS_RUN_DIRECTLY）照交，allowed 写它能给谁直接跑。
    会编的 cargo 由 starts_a_build 认，编出来的二进制由 runs_compiled_code 认，这里不认。弄坏开关 project-scripts-unjudged 下一律交 None。"""
    if break_is_set("project-scripts-unjudged") or not words:
        return None
    if launcher == "python" and "/" not in words[0] and words[0] in TEST_RUNNER_MODULE_FILES:
        return ProjectRun(f"python3 -m {words[0][:-len('.py')]} 起单测", "nobody", "")
    real_path = launched_file(words, directory, launcher)
    if real_path is None:
        return None
    relative = project_relative_path(real_path)
    if relative is None and not has_project_header(real_path):
        return None
    return ProjectRun(relative or words[0], directly_run_level(relative, real_path, list(words[1:])), real_path)


def task_server_client_subcommand(words, directory, launcher=None):
    """命令词是本仓的 task-server/task（仓副本里与它逐字节相同的拷贝同算；python3 起的、直接执行的都认）：交它的子命令（没写的交空串）；不是交 None。"""
    if not words:
        return None
    real_path = launched_file(words, directory, launcher)
    if real_path is None or project_relative_path(real_path) != TASK_SERVER_CLIENT or not same_as_own_copy(TASK_SERVER_CLIENT, real_path):
        return None
    return words[1] if len(words) > 1 else ""


def subagent_task_server_refusal(words, directory, launcher=None):
    """子 agent 起这一条要拒就交一句说明（起的是哪个子命令），不拒交 None：命令是本仓的提交客户端（task_server_client_subcommand），
    子命令不是 SUBAGENT_TASK_SERVER_SUBCOMMANDS 里的（submit、wait、cancel、clean 与认不出的都拒）。
    project-code-guard.sh 判子 agent 的每一条命令（读进来的脚本正文里的照判）、run-scope-guard.sh 判子 agent 经 submit 交的验证命令，都只调这一处。"""
    subcommand = task_server_client_subcommand(words, directory, launcher)
    if subcommand is None or subcommand in SUBAGENT_TASK_SERVER_SUBCOMMANDS:
        return None
    return f"{TASK_SERVER_CLIENT} {subcommand}" if subcommand else f"{TASK_SERVER_CLIENT}（没写子命令）"


def task_server_submission(words, directory, launcher=None):
    """命令词是本仓的 task-server/task（仓副本里与它逐字节相同的拷贝同算；python3 起的、直接执行的都认）而子命令是 submit：
    交 (`--` 之后那条命令的词, 任务的当前目录：--cwd 给了的取它照这一条的目录解开，没给是 None)；不是交 None。
    认不出的选项、没写 `--`、`--` 之后没有词的，客户端按用法错退 2、什么都不起，交 None（这一条照名单上的客户端放行）。
    弄坏开关 task-server-submission-unparsed 下一律交 None（submit 那一条只当名单上的客户端放行，后面那条一个字都不判）。"""
    if break_is_set("task-server-submission-unparsed") or len(words) < 2 or words[1] != "submit":
        return None
    if task_server_client_subcommand(words, directory, launcher) != "submit":
        return None
    arguments, position, task_directory = list(words[2:]), 0, None
    while position < len(arguments):
        argument = arguments[position]
        if argument == "--":
            inner = arguments[position + 1:]
            if not inner:
                return None
            resolved = shell_words.resolve_path(directory, os.path.expanduser(task_directory)) if task_directory and directory else task_directory
            return inner, resolved
        if argument == TASK_SERVER_SUBMIT_DIRECTORY_OPTION and position + 1 < len(arguments):
            task_directory, position = arguments[position + 1], position + 2
            continue
        if argument.startswith(TASK_SERVER_SUBMIT_DIRECTORY_OPTION + "="):
            task_directory, position = argument.split("=", 1)[1], position + 1
            continue
        return None
    return None


TASK_SERVER_LIBRARY_LOADED = []   # 空表是还没载；载过的放 (模块, None)，载不进放 (None, 原因)


def task_server_library():
    """按文件路径载入 task-server/task_server_lib.py（只用它的名字、缺省值与读配置的函数，不起服务）：交 (模块, None)，载不进交 (None, 原因)。
    自证换样本库时改 TASK_SERVER_LIBRARY_PATH 并清 TASK_SERVER_LIBRARY_LOADED。"""
    if not TASK_SERVER_LIBRARY_LOADED:
        try:
            specification = importlib.util.spec_from_file_location("task_server_lib_for_hooks", TASK_SERVER_LIBRARY_PATH)
            module = importlib.util.module_from_spec(specification)
            specification.loader.exec_module(module)
            TASK_SERVER_LIBRARY_LOADED.append((module, None))
        except Exception as error:   # 文件不在、语法错、它载入的库出错
            TASK_SERVER_LIBRARY_LOADED.append((None, f"载不进 {TASK_SERVER_LIBRARY_PATH}（{error!r}）"))
    return TASK_SERVER_LIBRARY_LOADED[0]


class TaskServerGuarded(NamedTuple):
    paths: tuple            # 库文件、签名密钥、socket 的路径（取 read_configuration；读不了是空的）
    names: tuple            # 同几样的文件名（库文件名、密钥文件名、「socket 所在目录名/socket 名」），路径写成变量拼的也认
    admitted_variable: str  # run-with-memory-cap.sh 的「已由 task-server 准入」开关变量（只由服务设）
    server_unit: str        # 服务自己的用户级单元
    task_unit_prefix: str   # 每一次尝试的单元名的头一段（单元名是 <它>-<库标签>-<号>-<第几次>.service）
    process_scripts: tuple  # 只由服务与它起的单元起的脚本（相对仓根）：服务本体与每次尝试的那一层
    build_directories: tuple  # 服务的编译目录（TASK_SERVER_BUILD_DIRECTORY 与状态目录下的 builds/，真实路径）：只经 task-server/task clean <号> 删


@functools.lru_cache(maxsize=1)
def task_server_guarded():
    """不经 task-server/task 不许碰的那几样，全取自 task_server_lib.py（不另抄一份）：交 (TaskServerGuarded, None)，载不进交 (None, 原因)。"""
    library, problem = task_server_library()
    if library is None:
        return None, problem
    try:
        configuration = library.read_configuration()
    except Exception:   # 配置不在、读不了：路径认不全，只认文件名
        configuration = None
    paths, names = [], [library.DATABASE_NAME]
    if configuration is not None:
        paths = [os.path.realpath(configuration.database), os.path.realpath(configuration.signing_key_file), os.path.realpath(configuration.socket)]
        key_file, runtime_directory = configuration.signing_key_file, configuration.runtime_directory
        build_roots = [configuration.build_directory, os.path.join(configuration.state_directory, TASK_SERVER_STATE_BUILDS_NAME)]
    else:   # 配置读不了：文件名取它的缺省值（default_values；没有这个函数或键的，只认库文件名）
        try:
            defaults = library.default_values(dict(os.environ))
        except Exception:
            defaults = {}
        key_file, runtime_directory = defaults.get("TASK_SERVER_SIGNING_KEY_FILE"), defaults.get("TASK_SERVER_RUNTIME_DIRECTORY")
        build_roots = [defaults.get("TASK_SERVER_BUILD_DIRECTORY")]
        if defaults.get("TASK_SERVER_STATE_DIRECTORY"):
            build_roots.append(os.path.join(defaults["TASK_SERVER_STATE_DIRECTORY"], TASK_SERVER_STATE_BUILDS_NAME))
    if key_file:
        names.append(os.path.basename(key_file))
    if runtime_directory:
        names.append(os.path.join(os.path.basename(runtime_directory), library.SOCKET_NAME))
    unit = library.UNIT_NAME
    prefix = unit[:unit.rindex("-") + 1] if "-" in unit else unit   # singlefs-task-server.service → singlefs-task-（服务起的单元名同一个头）
    scripts = tuple(os.path.relpath(path, HOOK_REPOSITORY) for path in (library.SERVER_SCRIPT, library.RUNNER_SCRIPT))
    builds = tuple(dict.fromkeys(os.path.realpath(root) for root in build_roots if root))
    return TaskServerGuarded(tuple(paths), tuple(names), library.ADMITTED_VARIABLE, unit, prefix, scripts, builds), None


def path_mentioned(text, path):
    """命令文本里点到这条路径：照写的、~/ 起头的、$HOME / ${HOME} 起头的、库文件的 -wal / -shm / -journal 跟着算（前缀相同）。"""
    home = os.path.expanduser("~")
    forms = [path]
    if path.startswith(home + os.sep):
        tail = path[len(home):]
        forms += ["~" + tail, "$HOME" + tail, "${HOME}" + tail]
    return any(form in text for form in forms)


def expanded_target(word, directory):
    """rm / find 的一个路径参数 → 真实路径：~ 与 $HOME、${HOME} 起头的换成家目录，相对路径从命令的当前目录算；别的变量与命令替换照字面算（判不到，不拒）。"""
    home = os.path.expanduser("~")
    for prefix in ("${HOME}", "$HOME", "~"):
        if word == prefix or word.startswith(prefix + "/"):
            word = home + word[len(prefix):]
            break
    return os.path.realpath(os.path.join(directory or os.getcwd(), word))


def build_directory_removal(words, directory, guarded):
    """一条剥掉前缀与包装的命令直接删服务的编译目录：rm / rmdir / unlink / shred 的目标、find 带 -delete 或 -exec rm 的起点，
    落在 guarded.build_directories 里、等于它们或是它们的上级（连同编译目录一起删）。交说明，不是交 None。"""
    name = os.path.basename(words[0])
    if name in REMOVAL_COMMAND_NAMES:
        targets, options_done = [], False
        for word in words[1:]:
            if word == "--" and not options_done:
                options_done = True
            elif options_done or not word.startswith("-"):
                targets.append(word)
    elif name == "find":
        deleting = "-delete" in words[1:] or any(word in FIND_EXECUTION_ACTIONS and position + 1 < len(words)
                                                 and os.path.basename(words[position + 1]) in REMOVAL_COMMAND_NAMES
                                                 for position, word in enumerate(words) if position)
        if not deleting:
            return None
        targets = []
        for word in words[1:]:
            if word in FIND_LEADING_OPTIONS and not targets:
                continue
            if word.startswith(("-", "(", ")", "!")):
                break
            targets.append(word)
    else:
        return None
    for target in targets:
        real = expanded_target(target, directory)
        for root in guarded.build_directories:
            if real == root or real.startswith(root + os.sep) or root.startswith(real.rstrip(os.sep) + os.sep):
                return (f"直接删 task-server 的编译目录（{target}，落在 {root} 里或是它的上级；编译目录只经 `task-server/task clean <号>` 删，"
                        "号是 wait 末行与 task-server/task status 里那个任务的）")
    return None


def task_server_text_bypass(text):
    """③ 整段命令文本里绕开 task-server 的写法：点到库文件、签名密钥、socket（路径或文件名），python 代码里载入 task_server_lib，
    在命令文本里设「已由 task-server 准入」开关（赋值、export、environ 写、setenv / putenv），python 代码里删服务的编译目录。交 ([说明…], 判不了的原因或 None)。
    弄坏开关 task-server-paths-unjudged 下不认前两样，admitted-variable-unjudged 下不认第三样，task-server-builds-unjudged 下不认第四样。"""
    if not text:
        return [], None
    guarded, problem = task_server_guarded()
    if guarded is None:
        return [], problem
    found = []
    if not break_is_set("task-server-paths-unjudged"):
        for path in guarded.paths:
            if path_mentioned(text, path):
                found.append(f"点到 task-server 的 {os.path.basename(path)}（{path}）")
        for name in guarded.names:
            if name in text and not any(name in entry for entry in found):
                found.append(f"点到 task-server 的 {name}")
        if TASK_SERVER_LIBRARY_IMPORT.search(text):
            found.append("在代码里载入 task-server/task_server_lib.py（它管库与签名密钥）")
    variable = re.escape(guarded.admitted_variable)
    setting = re.compile(variable + r"""['"]?\s*\]?\s*=(?!=)|\b(?:export|declare\s+-[A-Za-z]*x[A-Za-z]*|typeset\s+-[A-Za-z]*x[A-Za-z]*)\b[^;&|\n]*\b""" + variable
                         + r"""|(?:putenv|setenv|setdefault|update)\s*\([^)]*""" + variable)
    if not break_is_set("admitted-variable-unjudged") and setting.search(text):
        found.append(f"在 task-server 之外设 {guarded.admitted_variable}（它让内存包装跳过排队与记账，只由服务设）")
    if not break_is_set("task-server-builds-unjudged") and PYTHON_REMOVAL_CALL.search(text) and (
            any(path_mentioned(text, root) for root in guarded.build_directories) or any(os.path.basename(root) in text for root in guarded.build_directories[:1])):
        found.append("python 代码里删 task-server 的编译目录（编译目录只经 `task-server/task clean <号>` 删，号是 wait 末行与 task-server/task status 里那个任务的）")
    return found, None


def systemctl_unit_touch(words, guarded):
    """交说明、不交 None 的情形：systemctl 停、禁、改 task-server 的单元（服务自己的与每一次尝试的），或停整个用户级服务管理器。"""
    position, positional = 1, []
    while position < len(words):
        word = words[position]
        if word.startswith("-"):
            position += 2 if word in SYSTEMCTL_OPTIONS_WITH_VALUE else 1
            continue
        positional.append(word)
        position += 1
    if not positional:
        return None
    verb, units = positional[0], positional[1:]
    if verb in SYSTEMCTL_READ_ONLY_VERBS:
        return None
    if verb in SYSTEMCTL_MANAGER_VERBS and "--user" in words:
        return f"systemctl --user {verb} 停掉整个用户级服务管理器（连同 {guarded.server_unit}）"
    sample_task_unit = guarded.task_unit_prefix + "x-1-1.service"
    for unit in units:
        if unit.startswith(guarded.task_unit_prefix) or fnmatch.fnmatchcase(guarded.server_unit, unit) or fnmatch.fnmatchcase(sample_task_unit, unit):
            return f"systemctl {verb} {unit}（task-server 的单元只经 task-server/control 起停、经 task-server/task cancel 取消）"
    return None


def task_server_command_bypass(words, environment, directory, launcher=None):
    """③ 一条剥掉前缀与包装的命令绕开 task-server：环境里设了「已由 task-server 准入」开关、直接起服务本体或每次尝试的那一层、
    systemctl 停、禁、改它的单元（systemctl_unit_touch）、直接删服务的编译目录（build_directory_removal）。交 ([说明…], 判不了的原因或 None)。
    弄坏开关 admitted-variable-unjudged 下不认第一样，task-server-scripts-unjudged 下不认第二样，task-server-units-unjudged 下不认第三样，task-server-builds-unjudged 下不认第四样。"""
    if not words:
        return [], None
    guarded, problem = task_server_guarded()
    if guarded is None:
        return [], problem
    found = []
    if guarded.admitted_variable in (environment or {}) and not break_is_set("admitted-variable-unjudged"):
        found.append(f"在 task-server 之外设 {guarded.admitted_variable}（它让内存包装跳过排队与记账，只由服务设）")
    real_path = launched_file(words, directory, launcher)
    if real_path is not None and not break_is_set("task-server-scripts-unjudged") and project_relative_path(real_path) in guarded.process_scripts:
        found.append(f"直接起 {project_relative_path(real_path)}（服务由 task-server/control 起停，每一次尝试的那一层只由服务起）")
    if os.path.basename(words[0]) == "systemctl" and not break_is_set("task-server-units-unjudged"):
        touched = systemctl_unit_touch(words, guarded)
        if touched:
            found.append(touched)
    if not break_is_set("task-server-builds-unjudged"):
        removal = build_directory_removal(words, directory, guarded)
        if removal:
            found.append(removal)
    return found, None


# 包在命令外面、之后照样起那条命令的程序（lib_shell_words 那张前缀表之外的）：名字 → 它带一个值的选项。
# 这张表不进 lib_shell_words 的前缀表：bash-command-detector.sh 要按命令词认出 systemd-run，判它等不等结束。
LAUNCHER_OPTIONS_WITH_VALUE = {
    "time": {"-f", "--format", "-o", "--output"},
    "flock": {"-w", "--wait", "--timeout", "-E", "--conflict-exit-code"},
    "chrt": {"-T", "--sched-runtime", "-P", "--sched-period", "-D", "--sched-deadline"},
    "prlimit": {"-o", "--output", "-p", "--pid"},
    "systemd-run": {"-u", "--unit", "-p", "--property", "--description", "--slice", "-E", "--setenv", "-M", "--machine", "-H", "--host",
                    "--uid", "--gid", "--nice", "--working-directory", "--on-active", "--on-boot", "--on-startup", "--on-unit-active",
                    "--on-unit-inactive", "--on-calendar", "--path-property", "--socket-property", "--timer-property", "--service-type",
                    "-C", "--capsule", "--expand-environment"},
    "strace": {"-a", "-b", "-e", "-E", "--env", "-I", "-o", "-O", "-p", "-P", "-s", "-S", "-u", "-U", "-X",
               "--columns", "--detach-on", "--interruptible", "--stack-trace-frame-limit", "--output", "--summary-syscall-overhead", "--attach",
               "--trace-path", "--string-limit", "--summary-sort-by", "--user", "--summary-columns", "--const-print-style", "--syscall-limit",
               "--argv0", "--trace", "--trace-fds", "--abbrev", "--verbose", "--raw", "--signal", "--signals", "--status", "--read", "--write",
               "--fault", "--inject", "--kvm", "--decode-pids"},
    "perf": {"-e", "--event", "-o", "--output", "-p", "--pid", "-t", "--tid", "-C", "--cpu", "-r", "--repeat", "-I", "--interval-print",
             "-G", "--cgroup", "-x", "--field-separator", "-F", "--freq", "-c", "--count", "-m", "--mmap-pages", "-u", "--uid", "-j",
             "--branch-filter", "--call-graph", "-D", "--delay", "-M", "--metrics"},
    "rustup": set(),
}
# 各包装程序不带值（或值只能用 = 贴着写）的长选项：长选项可以只写一个前缀（getopt_long 认），前缀对不上这里的、却是某个要值的长选项的前缀时，
# 按要值算（下一个词是它的值）。两张表照各自的 --help（systemd-run 照 systemd 255 的 man 页）列；strace 那几个「要不要值」照 strace -o /dev/null --<选项> true 量过
LAUNCHER_LONG_OPTIONS_WITHOUT_VALUE = {
    "time": {"--append", "--portability", "--quiet", "--verbose", "--help", "--version"},
    "flock": {"--shared", "--exclusive", "--unlock", "--nonblock", "--nb", "--close", "--no-fork", "--verbose", "--help", "--version"},
    "chrt": {"--batch", "--deadline", "--fifo", "--idle", "--other", "--rr", "--reset-on-fork", "--all-tasks", "--max", "--pid", "--verbose",
             "--help", "--version"},
    "prlimit": {"--noheadings", "--raw", "--verbose", "--help", "--version", "--core", "--data", "--nice", "--fsize", "--sigpending", "--memlock",
                "--rss", "--nofile", "--msgqueue", "--rtprio", "--stack", "--cpu", "--nproc", "--as", "--locks", "--rttime"},
    "systemd-run": {"--collect", "--help", "--no-ask-password", "--no-block", "--on-clock-change", "--on-timezone-change", "--pipe", "--pty",
                    "--quiet", "--remain-after-exit", "--same-dir", "--scope", "--send-sighup", "--shell", "--slice-inherit", "--system", "--user",
                    "--version", "--wait"},
    "strace": {"--output-append-mode", "--summary-only", "--summary", "--debug", "--daemonize", "--follow-forks", "--output-separately", "--help",
               "--instruction-pointer", "--kill-on-exit", "--stack-trace", "--syscall-number", "--relative-timestamps", "--absolute-timestamps",
               "--timestamps", "--syscall-times", "--no-abbrev", "--version", "--summary-wall-clock", "--strings-in-hex", "--pidns-translation",
               "--successful-only", "--failed-only", "--failing-only", "--seccomp-bpf", "--tips", "--quiet", "--silent", "--silence", "--decode-fds"},
    "perf": set(),
    "rustup": set(),
}
# 弄坏开关 launcher-long-options-partial 下从要值的那张表里拿掉的（补全之前的表），同时不认长选项的前缀
LAUNCHER_LONG_OPTIONS_COMPLETED = {
    "systemd-run": {"--expand-environment"},
    "strace": {"--columns", "--detach-on", "--interruptible", "--stack-trace-frame-limit", "--output", "--summary-syscall-overhead", "--attach",
               "--trace-path", "--string-limit", "--summary-sort-by", "--user", "--summary-columns", "--const-print-style", "--syscall-limit",
               "--argv0", "--trace", "--trace-fds", "--abbrev", "--verbose", "--raw", "--signal", "--signals", "--status", "--read", "--write",
               "--fault", "--inject", "--kvm", "--decode-pids"},
}
# 选项之后、那条命令之前还有几个位置参数：flock 的锁文件、chrt 的优先级、rustup run 的工具链
LAUNCHER_POSITIONAL_COUNT = {"flock": 1, "chrt": 1, "rustup": 1}
# 给里面那条命令设环境变量的选项（值是 NAME=VALUE）：设的变量写在交回那条命令的最前面
LAUNCHER_ENVIRONMENT_OPTIONS = {"systemd-run": {"-E", "--setenv"}, "strace": {"-E", "--env"}}
# 交回的命令以 NAME=VALUE 打头时，那是设的变量、不是程序（值里的路径 basename 碰巧叫 strace 也不当包装剥）
ENVIRONMENT_ASSIGNMENT_WORD = re.compile(r"^[A-Za-z_][A-Za-z0-9_]*=")
# perf 起一条命令的子命令；后面还要再跟一个 record 的那几组
PERF_SUBCOMMANDS_RUNNING_A_COMMAND = {"stat", "record", "trace"}
PERF_GROUPS_WITH_RECORD = {"mem", "c2c", "sched", "lock", "kmem", "kwork"}


def resolved_long_option(option, options_with_value, options_without_value):
    """一个长选项词（`=` 之前那一段）：交回 (认成的选项名, 要不要值)。整词在两张表里的照表；都不在时按前缀认（getopt_long 认唯一的前缀）：
    恰好是一个已知长选项的前缀就当那一个；是几个的前缀时程序自己报「有歧义」、一条都不起，其中有要值的就按要值算（怎么算都一样）。
    弄坏开关 launcher-long-options-partial 下只按整词查表。"""
    if option in options_with_value:
        return option, True
    if option in options_without_value or break_is_set("launcher-long-options-partial") or not option.startswith("--"):
        return option, False
    matches = sorted(candidate for candidate in options_with_value | options_without_value if candidate.startswith("--") and candidate.startswith(option))
    if len(matches) == 1:
        return matches[0], matches[0] in options_with_value
    return option, any(candidate in options_with_value for candidate in matches)


def launcher_option(word, next_word, options_with_value, options_without_value=frozenset()):
    """包装程序的一个选项词：交回 (选项名, 它的值, 这个选项占几个词)。长选项 `--名=值` 与 `--名 值` 查两张表（resolved_long_option，认前缀，
    交回的选项名是认成的那一个）；短选项合写（`-fo 文件`、`-xw 10`、`-o文件`）逐个字母查表，头一个要值的字母之后剩下的是它的值，没剩下就取下一个词；
    一个要值的字母都没有的只占自己。"""
    if word.startswith("--") or break_is_set("launcher-whole-word-options"):
        option, has_attached_value, attached_value = word.partition("=")
        option, takes_value = resolved_long_option(option, options_with_value, options_without_value)
        if has_attached_value:
            return option, attached_value, 1
        return (option, next_word, 2) if takes_value else (option, "", 1)
    for index, letter in enumerate(word[1:], start=1):
        if "-" + letter in options_with_value:
            attached_value = word[index + 1:]
            return ("-" + letter, attached_value, 1) if attached_value else ("-" + letter, next_word, 2)
    return word, "", 1


# systemd-run -p EnvironmentFile= 读不了时替它交的那一个变量：名字照 CARGO_TARGET_*_RUNNER 的形状，判法按「环境变量里定了 runner」算（判不出，按宽）
UNREADABLE_ENVIRONMENT_FILE_RUNNER = "CARGO_TARGET_UNREADABLE_ENVIRONMENT_FILE_RUNNER"


def assignments_of_environment_property(value):
    """systemd-run -p / --property 的一个值设给里面那条命令的 NAME=VALUE：`Environment=A=1 "B=2 3"` 按 systemd 的引号规矩切；
    `EnvironmentFile=[-]<文件>` 按内容读（一行一个 NAME=VALUE，# 与 ; 起头的行、空行不算，行首的 export 去掉）；
    文件读不了（不在、没权限）交一个 UNREADABLE_ENVIRONMENT_FILE_RUNNER=<文件>：里面设了什么判不出，按设了 runner 算。别的属性交空表。"""
    name, _equals, rest = value.partition("=")
    if name == "Environment":
        try:
            words = shlex.split(rest)
        except ValueError:
            words = rest.split()
        return [word for word in words if ENVIRONMENT_ASSIGNMENT_WORD.match(word)]
    if name == "EnvironmentFile":
        path = rest[1:] if rest.startswith("-") else rest
        try:
            with open(path, encoding="utf-8", errors="replace") as handle:
                lines = handle.read().split("\n")
        except OSError:
            return [f"{UNREADABLE_ENVIRONMENT_FILE_RUNNER}={path}"]
        assignments = []
        for line in lines:
            line = line.strip()
            if not line or line.startswith(("#", ";")):
                continue
            line = line[len("export "):].lstrip() if line.startswith("export ") else line
            if ENVIRONMENT_ASSIGNMENT_WORD.match(line):
                key, _separator, assigned = line.partition("=")
                assignments.append(f"{key}={assigned.strip().strip(chr(34)).strip(chr(39))}")
        return assignments
    return []


def command_under_gpu_quiesce(words, directory=None):
    """words[0] 是 scripts 这一层的 gpu-quiesce-run.sh（名字与位置，located_in；别处的同名拷贝不剥）而参数照它开头的 preflight 摘掉每一个 --force 之后
    是 `-- <命令…>` 时交回 (<命令…> 的词, None)：当前目录与环境照外面那一条；--selftest、没写 `--` 的交 None。弄坏开关 gpu-quiesce-opaque 下一律交 None。"""
    if break_is_set("gpu-quiesce-opaque") or not words or os.path.basename(words[0]) != GPU_QUIESCE_SCRIPT:
        return None
    if ENVIRONMENT_ASSIGNMENT_WORD.match(words[0]) or not located_in(words[0], directory, GPU_QUIESCE_DIRECTORY):
        return None
    rest = [word for word in words[1:] if word != "--force"]
    if rest[:1] != ["--"] or len(rest) < 2:
        return None
    return rest[1:], None


def command_under_two_logs(words, directory=None):
    """words[0] 是 scripts/hosts 这一层的 two-logs.sh（名字与位置，located_in；别处的同名拷贝不剥）而参数是 `[--quiet] <名> -- <命令…>` 时交回 (<命令…> 的词, None)：
    当前目录与环境照外面那一条；它不摘参数里的 --force（只对 python3 做准入），所以 `--` 之后原样交回。--selftest、--from-log 与没写 `--` 的交 None。
    弄坏开关 two-logs-opaque 下一律交 None。"""
    if break_is_set("two-logs-opaque") or not words or os.path.basename(words[0]) != TWO_LOGS_SCRIPT:
        return None
    if ENVIRONMENT_ASSIGNMENT_WORD.match(words[0]) or not located_in(words[0], directory, TWO_LOGS_DIRECTORY):
        return None
    rest = words[1:]
    if rest[:1] == ["--quiet"]:
        rest = rest[1:]
    if len(rest) < 3 or rest[0].startswith("-") or rest[1] != "--":
        return None
    return rest[2:], None


def command_under_run_build_directory(words, directory=None):
    """words[0] 是 scripts/hosts 这一层的 run-build-directory.sh（名字与位置，located_in；别处的同名拷贝不剥）而参数是 `[--] <命令…>`（包一层起：
    这一趟一个自己的编译目录、跑完删，那份脚本文件末尾「包一层起」一段）时交回 (<命令…> 的词, None)：当前目录与环境照外面那一条。
    --selftest、没给命令、命令前写了别的选项的交 None。弄坏开关 run-build-directory-opaque 下一律交 None。"""
    if break_is_set("run-build-directory-opaque") or not words or os.path.basename(words[0]) != RUN_BUILD_DIRECTORY_SCRIPT:
        return None
    if ENVIRONMENT_ASSIGNMENT_WORD.match(words[0]) or not located_in(words[0], directory, RUN_BUILD_DIRECTORY_LOCATION):
        return None
    rest = words[1:]
    if rest[:1] == ["--"]:
        rest = rest[1:]
    elif rest[:1] and rest[0].startswith("-"):
        return None
    return (rest, None) if rest else None


def dispatched_command(words):
    """words[0] 是派发起法（DISPATCH_LAUNCHER_OPTIONS 的键）时，照它切参数的写法取 `--` 之后那条命令的词；没有命令交 None。
    `--` 之前有认不出的参数的，派发器按用法错退 2、什么都不跑，这里照严的一侧取它后面第一个 `--` 之后的那条照判。"""
    flags, arity = DISPATCH_LAUNCHER_OPTIONS[os.path.basename(words[0])]
    position = 1
    while position < len(words):
        word = words[position]
        if word == "--":
            return words[position + 1:] if position + 1 < len(words) else None
        if word in flags:
            position += 1
        elif word in arity:
            position += 1 + arity[word]
        else:
            break
    rest = words[position:]
    if "--" not in rest or rest.index("--") + 1 >= len(rest):
        return None
    return rest[rest.index("--") + 1:]


def command_under_dispatch_launcher(words, directory=None):
    """words[0] 是 scripts/hosts 这一层的 multi-host-run.sh（名字与位置，located_in；别处的同名拷贝不剥）而参数是 `--peer [选项…] -- <命令…>` 时
    交回 (<命令…> 的词, None)（--peer 写没写不另分：不带 --peer 它退 2、什么都不跑，照样剥开判里面那条）：当前目录与环境照外面那一条。头一个参数是 MULTI_HOST_RUN_OTHER_MODES 之一的、--selftest 与没写 `--` 的交 None。
    不并进 command_under_launcher：task-server/task_server.py 借 command_under_launcher 剥启动器；project-code-guard.sh 在 command_under_launcher、
    command_under_gpu_quiesce 之后再试它，里面那条算经内存包装（dispatched_under_memory_cap）。
    弄坏开关 dispatch-launchers-unseen 下一律交 None。"""
    if break_is_set("dispatch-launchers-unseen") or not words or ENVIRONMENT_ASSIGNMENT_WORD.match(words[0]):
        return None
    name = os.path.basename(words[0])
    if name not in DISPATCH_LAUNCHER_OPTIONS or not located_in(words[0], directory, DISPATCH_LAUNCHER_DIRECTORY):
        return None
    if name == MULTI_HOST_RUN_SCRIPT and words[1:2] and words[1] in MULTI_HOST_RUN_OTHER_MODES:
        return None
    dispatched = dispatched_command(words)
    return None if dispatched is None else (dispatched, None)


def temporary_service_wrapped(words, directory=None):
    """words[0] 是 scripts/prune-server 这一层的 prune_server_client_selftest.py（名字与位置，located_in；别处的同名拷贝不认）、头一个参数是
    --with-temporary-service 时交 (<-- 之后那条原命令的词>, 给没给 --server-binary <二进制>)；别的、-- 后面没有命令的交 None（文件头 temporary_service_wrapped）。
    -- 之前有认不出的参数的，包装按用法错退 2、什么都不跑，这里照严的一侧取第一个 -- 之后那条照判（同 dispatched_command）。
    弄坏开关 temporary-service-wrapper-unseen 下一律交 None。"""
    if break_is_set("temporary-service-wrapper-unseen") or len(words) < 2 or ENVIRONMENT_ASSIGNMENT_WORD.match(words[0]):
        return None
    if os.path.basename(words[0]) != TEMPORARY_SERVICE_WRAPPER_SCRIPT or words[1] != TEMPORARY_SERVICE_WRAPPER_MODE \
            or not located_in(words[0], directory, TEMPORARY_SERVICE_WRAPPER_DIRECTORY):
        return None
    binary_given, position = False, 2
    while position < len(words):
        if words[position] == "--":
            return (words[position + 1:], binary_given) if position + 1 < len(words) else None
        if words[position] == TEMPORARY_SERVICE_BINARY_OPTION and position + 1 < len(words) and words[position + 1] != "--":
            binary_given, position = True, position + 2
            continue
        position += 1
    return None


def command_under_temporary_service(words, directory=None):
    """临时 prune-server 的包装给了 --server-binary 时交回 (<原命令…> 的词, None)：当前目录与环境照外面那一条，包装不编、只起那份二进制与原命令；
    没给 --server-binary（包装不现编退 2，或在服务给的编译目录里现编）与别的交 None（文件头 temporary_service_wrapped）。"""
    wrapped = temporary_service_wrapped(words, directory)
    return (wrapped[0], None) if wrapped is not None and wrapped[1] else None


@functools.lru_cache(maxsize=8)
def temporary_service_table_at(path, modified, size):
    """要临时服务的自证登记表 → ({键: 判定}, [写坏的行，逐条一句])。modified、size 只当缓存的键（看门狗常驻，表改了要重读）。
    判定认不出、没有 # 理由或理由太短、键重复（留第一行）的不收；弄坏开关 temporary-service-table-lenient 下照收、不报。"""
    rows, problems = {}, []
    try:
        with open(path, encoding="utf-8") as handle:
            lines = handle.read().split("\n")
    except (OSError, UnicodeDecodeError) as error:
        return {}, [f"读不了要临时服务的自证登记表 {path}：{error}"]
    lenient = break_is_set("temporary-service-table-lenient")
    for number, line in enumerate(lines, start=1):
        if not line.strip() or line.lstrip().startswith("#"):
            continue
        fields = line.split(None, 2)
        key, judgement = fields[0], fields[1] if len(fields) > 1 else ""
        reason = fields[2][1:].strip() if len(fields) > 2 and fields[2].startswith("#") else ""
        wrong = []
        if judgement not in (TEMPORARY_SERVICE_REQUIRED, TEMPORARY_SERVICE_OWN_SERVER):
            wrong.append(f"判定「{judgement}」认不出（只认 {TEMPORARY_SERVICE_REQUIRED}、{TEMPORARY_SERVICE_OWN_SERVER}）")
        if len(reason) < SELFTEST_REASON_MINIMUM:
            wrong.append(f"# 之后的理由不到 {SELFTEST_REASON_MINIMUM} 个字")
        if key in rows:
            wrong.append("键重复")
        if wrong and not lenient:
            problems.append(f"第 {number} 行 {key}：{'；'.join(wrong)}")
            continue
        rows.setdefault(key, judgement)
    return rows, problems


def read_temporary_service_table(path):
    """要临时服务的自证登记表此刻的内容（temporary_service_table_at 按修改时刻与大小缓存）；文件不在交 ({}, [一句])。"""
    try:
        status = os.stat(path)
    except OSError as error:
        return {}, [f"读不了要临时服务的自证登记表 {path}：{error.strerror}"]
    return temporary_service_table_at(path, status.st_mtime_ns, status.st_size)


def temporary_service_table_problems(path, registry_path, root):
    """要临时服务的自证登记表与自证登记表、脚本源码「两边对一遍」（表头写了两条），逐条一句；空表（读得了）是没毛病：
    写坏的行；行里的文件在 root 下不在、没登记在 registry_path 里；required 行的源码里没有 TEMPORARY_SERVICE_SOURCE_MARK；
    registry_path 里非门禁阶段的行，源码里有记号而这张表里没有这一行。"""
    rows, problems = read_temporary_service_table(path)
    problems = list(problems)
    registry = read_selftest_registry(registry_path)[0]

    def source_has_mark(key):
        try:
            with open(os.path.join(root, key), encoding="utf-8", errors="replace") as handle:
                return TEMPORARY_SERVICE_SOURCE_MARK in handle.read()
        except OSError:
            return None

    for key, judgement in sorted(rows.items()):
        marked = source_has_mark(key)
        if marked is None:
            problems.append(f"{key}：{root} 下没有这个文件")
        elif key not in registry:
            problems.append(f"{key}：没登记在自证登记表 {registry_path} 里（先在那里登记，再来要临时服务的表加一行）")
        elif judgement == TEMPORARY_SERVICE_REQUIRED and not marked:
            problems.append(f"{key}：登记了 {TEMPORARY_SERVICE_REQUIRED}，源码里却没有 {TEMPORARY_SERVICE_SOURCE_MARK}（裸跑不会因为没有临时服务退 2，登记写错了？）")
    for key in sorted(registry):
        if key in rows or (os.path.dirname(key) == GATE_STAGE_DIRECTORY and not gate_top_library_key(key)):
            continue
        if source_has_mark(key):
            problems.append(f"{key}：源码里调了 {TEMPORARY_SERVICE_SOURCE_MARK}，要临时服务的自证登记表里没有这一行（{TEMPORARY_SERVICE_REQUIRED} 或 {TEMPORARY_SERVICE_OWN_SERVER}）")
    return problems


def temporary_service_command(key):
    """要临时服务的自证的整条写法（不含 task-server/task submit -- 前缀）：包装写法套着 <解释器> <脚本> --selftest（.py 用 python3，别的用 bash）。
    包装不带 --server-binary：任务服务把这条认成会编的、给编译目录，包装现编一遍 prune-server（与源码同一版）；自己编了、两台都读得到的二进制才另加 --server-binary，不写进这一条。
    弄坏开关 temporary-service-command-bare 下是裸写的样子，temporary-service-command-binary-placeholder 下带 --server-binary 与尖括号占位
    （照抄进 shell 时尖括号被当成输入重定向、命令不执行）。"""
    bare = f"{'python3' if key.endswith('.py') else 'bash'} {key} --selftest"
    if break_is_set("temporary-service-command-bare"):
        return bare
    binary = f"{TEMPORARY_SERVICE_BINARY_OPTION} <prune-server 二进制> " if break_is_set("temporary-service-command-binary-placeholder") else ""
    return f"python3 {TEMPORARY_SERVICE_WRAPPER_DIRECTORY}/{TEMPORARY_SERVICE_WRAPPER_SCRIPT} {TEMPORARY_SERVICE_WRAPPER_MODE} {binary}-- {bare}"


def environment_under_temporary_service(environment):
    """包装 `--with-temporary-service … -- <原命令>` 给里面那条原命令设的环境：外面的环境加上临时服务的标记（project-code-guard.sh 剥开包装重判里面那条时用）。
    弄坏开关 temporary-service-environment-dropped 下原样交回（包装里的原命令看起来没有标记）。"""
    if break_is_set("temporary-service-environment-dropped"):
        return dict(environment or {})
    return dict(environment or {}, **{TEMPORARY_SERVICE_VARIABLE: "temporary-service"})


def temporary_service_bare_selftest(words, directory, environment=None):
    """words 是一条已经剥掉前缀与包装的命令（命令词是脚本）：它起的是要临时服务的自证——命令词解到自证登记表所在仓里那一份、要临时服务的自证登记表里判定是 required——
    写成了裸的单独一个 --selftest，环境里又没有临时服务的标记（命令前自己设了 TEMPORARY_SERVICE_VARIABLE，或包装剥开重判时 environment_under_temporary_service 补的），
    交一句说明（为什么、整条包装写法），不是交 None。这种自证裸跑退 2、什么都不判，所以执行前拒，免得白交一趟。
    不并进 registered_selftest：它也被 scripts/gate/selftest-process-guard.py 与 command_under_launcher 剥开后的重判共用，
    改了它，包装里的原命令就盯不上；project-code-guard.sh 在命令位置上单独调这一个。
    弄坏开关 temporary-service-bare-unjudged 下一律交 None；temporary-service-own-server-as-required 下 own-server 的行也算 required；
    temporary-service-marker-ignored 下不看环境里的标记。"""
    if break_is_set("temporary-service-bare-unjudged") or not words or ENVIRONMENT_ASSIGNMENT_WORD.match(words[0]) or list(words[1:]) != ["--selftest"]:
        return None
    key = selftest_registry_key(words[0], directory)
    if key is None or not selftest_registered_file(words[0], directory, key):
        return None
    judgement = read_temporary_service_table(TEMPORARY_SERVICE_TABLE_PATH)[0].get(key)
    if judgement != TEMPORARY_SERVICE_REQUIRED and not (judgement == TEMPORARY_SERVICE_OWN_SERVER and break_is_set("temporary-service-own-server-as-required")):
        return None
    if (environment or {}).get(TEMPORARY_SERVICE_VARIABLE) and not break_is_set("temporary-service-marker-ignored"):
        return None
    return (f"{key} --selftest 只在临时 prune-server 底下跑（开跑先核临时服务，环境里没有 {TEMPORARY_SERVICE_VARIABLE} 就退 2、什么都不判；"
            f"登记在 .claude/hooks/{TEMPORARY_SERVICE_TABLE_NAME}）：整条写成 `task-server/task submit -- {temporary_service_command(key)}`"
            f"（整条不带 {TEMPORARY_SERVICE_BINARY_OPTION}：交给任务服务的这一条由服务认成会编的、给编译目录，包装现编一遍 prune-server；不经服务起、环境里没有 CARGO_TARGET_DIR 时包装不现编、退 2；"
            f"自己编了、两台都读得到的二进制才在 {TEMPORARY_SERVICE_WRAPPER_MODE} 后面另加 {TEMPORARY_SERVICE_BINARY_OPTION} 与它的绝对路径，写在整条之外）")


def dispatched_under_memory_cap(words, directory=None):
    """派发起法剥出来的那条算不算经内存包装：multi-host-run.sh --peer 在第二台经仓里的 run-with-memory-cap.sh 起（不带 --peer 退 2、什么都不跑）。
    命令词解到盘上的文件而不是本仓那一份、内容（连同它 source / import 的库）又与本仓那一份不同的不算（同 existing_copy_elsewhere 的口径）；当前目录认不出的不算；盘上没有的算（起不来，什么都不跑）。"""
    location = os.path.join(DISPATCH_LAUNCHER_DIRECTORY, os.path.basename(words[0]))
    resolved = shell_words.resolve_path(directory, os.path.expanduser(words[0]))
    if resolved is None:
        return False
    return not os.path.isfile(resolved) or family_own_copy(words[0], directory, location) or same_content_as_own(resolved, location)


def command_under_launcher(words, directory=None):
    """words[0] 是 LAUNCHER_OPTIONS_WITH_VALUE 里的程序时，剥掉它（连同它的选项与位置参数），交回 (它起的那条命令的词, 那条命令的当前目录：
    None 是不变，systemd-run --working-directory 给的是相对当前目录的写法)；不是这几样、或后面没有要起的命令（strace -p、flock 只给 fd）交 None。
    选项词怎么切见 launcher_option；LAUNCHER_ENVIRONMENT_OPTIONS 里的选项（systemd-run 的 -E / --setenv、strace 的 -E / --env）设的 NAME=VALUE，
    与 systemd-run -p / --property 的 Environment= / EnvironmentFile= 设的（assignments_of_environment_property）写在交回那条命令的最前面。
    flock 的 -c <字符串> 交回成 sh -c <字符串>。rustup 只认 rustup run，perf 只认 stat / record / trace 与 <组> record。
    scripts/hosts/two-logs.sh 的 `[--quiet] <名> -- <命令…>` 照 command_under_two_logs 剥。
    scripts/hosts/run-build-directory.sh 的 `[--] <命令…>` 照 command_under_run_build_directory 剥。
    scripts/prune-server/prune_server_client_selftest.py 的 `--with-temporary-service --server-binary <二进制> -- <命令…>` 照 command_under_temporary_service 剥。
    scripts/crash/gpu-quiesce-run.sh 不在这里剥（command_under_gpu_quiesce，调用方另试）。"""
    two_logs = command_under_two_logs(words, directory)
    if two_logs is not None:
        return two_logs
    own_build_directory = command_under_run_build_directory(words, directory)
    if own_build_directory is not None:
        return own_build_directory
    temporary_service = command_under_temporary_service(words, directory)
    if temporary_service is not None:
        return temporary_service
    name = os.path.basename(words[0])
    if name not in LAUNCHER_OPTIONS_WITH_VALUE or ENVIRONMENT_ASSIGNMENT_WORD.match(words[0]):
        return command_under_extra_launcher(words)
    position = 1
    if name == "rustup":
        if words[1:2] != ["run"]:
            return None
        position = 2
    elif name == "perf":
        if words[1:2] and words[1] in PERF_SUBCOMMANDS_RUNNING_A_COMMAND:
            position = 2
        elif words[1:2] and words[1] in PERF_GROUPS_WITH_RECORD and words[2:3] == ["record"]:
            position = 3
        else:
            return None
    options_with_value, working_directory, assignments = LAUNCHER_OPTIONS_WITH_VALUE[name], None, []
    if break_is_set("launcher-long-options-partial"):
        options_with_value = options_with_value - LAUNCHER_LONG_OPTIONS_COMPLETED.get(name, set())
    options_without_value = LAUNCHER_LONG_OPTIONS_WITHOUT_VALUE.get(name, set())
    environment_options = set() if name == "strace" and break_is_set("strace-drops-env") else LAUNCHER_ENVIRONMENT_OPTIONS.get(name, set())
    while position < len(words):
        word = words[position]
        if word == "--":
            position += 1
            break
        if not word.startswith("-") or word == "-":
            break
        option, value, word_count = launcher_option(word, words[position + 1] if position + 1 < len(words) else "", options_with_value,
                                                    options_without_value)
        if name == "systemd-run" and option == "--working-directory":
            working_directory = value
        if option in environment_options and "=" in value and not break_is_set("launcher-drops-setenv"):
            assignments.append(value)
        if name == "systemd-run" and option in ("-p", "--property") and not break_is_set("launcher-drops-property-environment"):
            assignments += assignments_of_environment_property(value)
        position += word_count
    if name == "flock" and position + 1 < len(words) and words[position + 1] in ("-c", "--command"):
        return (["sh", "-c", words[position + 2]], working_directory) if position + 2 < len(words) else None
    position += LAUNCHER_POSITIONAL_COUNT.get(name, 0)
    return (assignments + words[position:], working_directory) if position < len(words) else None


# 名单外启动器（research/prompts/shell-words-r1-main-verification.md 报的 25 个、research/prompts/shell-words-r2-main-verification.md 第二节第 11 条、research/prompts/shell-words-r3-sonnet-s2-output.md 第八节 A3、A4）：
# 包在命令外面照样起那条命令的程序，剥掉之后里面那条照判（command_under_extra_launcher）。名字 → (带值的短选项字母, 带值的长选项, 选项之后、命令之前的位置参数个数)。
# 值只能贴着写的选项（`-r目录`、`--mount=文件`）不进带值的表。弄坏开关 LIB_PROJECT_CODE_RUNS_BREAK=extra-launchers-unseen 下这几样都不剥（命令词当普通程序，参数里的命令不判）。
EXTRA_LAUNCHER_OPTIONS = {
    "doas": ("aCu", set(), 0),
    "numactl": ("NmCipP", {"--cpunodebind", "--membind", "--physcpubind", "--interleave", "--preferred", "--preferred-many", "--cpubind"}, 0),
    "unshare": ("SGRw", {"--setuid", "--setgid", "--root", "--wd", "--map-user", "--map-group", "--map-users", "--map-groups", "--propagation", "--setgroups"}, 0),
    "setpriv": ("", {"--ruid", "--euid", "--rgid", "--egid", "--reuid", "--regid", "--groups", "--inh-caps", "--ambient-caps", "--bounding-set", "--securebits",
                     "--pdeathsig", "--selinux-label", "--apparmor-profile", "--landlock-access", "--landlock-rule"}, 0),
    "nsenter": ("tSG", {"--target", "--setuid", "--setgid"}, 0),
    "ltrace": ("aADeFlnopsuwx", {"--align", "--output", "--indent", "--library"}, 0),
    "valgrind": ("", set(), 0),
    "faketime": ("p", {"--date-prog"}, 1),
    "unbuffer": ("", set(), 0),
    "cpulimit": ("lpe", {"--limit", "--pid", "--exe"}, 0),
    "dbus-run-session": ("", {"--config-file", "--dbus-daemon"}, 0),
    "firejail": ("", set(), 0),
    "proot": ("rbmqwki", {"--rootfs", "--bind", "--mount", "--qemu", "--pwd", "--cwd", "--kernel-release", "--change-id"}, 0),
    "chroot": ("", {"--userspec", "--groups"}, 1),
    "cgexec": ("g", set(), 0),
}
# 起的命令不是逐词交的（ssh 把后面的词用空格连起来交给远端 shell、su -c 交一段代码）或要单独切的几样，各有自己的剥法（command_under_extra_launcher）
EXTRA_LAUNCHER_SPECIAL = frozenset({"xargs", "find", "ssh", "su", "runuser", "sg", "script", "watch", "gdb", "scl", "parallel", "bwrap"})
EXTRA_LAUNCHER_NAMES = frozenset(EXTRA_LAUNCHER_OPTIONS) | EXTRA_LAUNCHER_SPECIAL
# xargs 从标准输入读进来、接在命令后面（或换掉 -I 的替换串）的词，find -exec 换掉 {} 的找到的路径，parallel 的参数：执行前算不出，用一个没展开的变量词占位，
# 调用方照「认不出」判（命令词、脚本路径、cargo 参数写成它的，project-code-guard.sh 照认不出的写法拒）
XARGS_INPUT_WORD = "$XARGS_INPUT_FROM_STANDARD_INPUT"
FIND_PATH_WORD = "$FIND_PATH_FOUND"
PARALLEL_INPUT_WORD = "$PARALLEL_ARGUMENTS"
XARGS_SHORT_OPTIONS_WITH_VALUE = "adEILnPs"          # -e、-i、-l 的值只能贴着写
XARGS_LONG_OPTIONS_WITH_VALUE = {"--arg-file", "--delimiter", "--max-args", "--max-procs", "--max-chars", "--process-slot-var"}
SSH_SHORT_OPTIONS_WITH_VALUE = "BbcDEeFIiJLlmOopQRSWw"
SCRIPT_SHORT_OPTIONS_WITH_VALUE = "cEIOBTmo"
SCRIPT_LONG_OPTIONS_WITH_VALUE = {"--command", "--echo", "--log-in", "--log-out", "--log-io", "--log-timing", "--logging-format", "--output-limit"}
SU_SHORT_OPTIONS_WITH_VALUE = "cgGswu"
SU_LONG_OPTIONS_WITH_VALUE = {"--command", "--session-command", "--group", "--supp-group", "--shell", "--whitelist-environment", "--user"}
WATCH_SHORT_OPTIONS_WITH_VALUE = "nq"
WATCH_LONG_OPTIONS_WITH_VALUE = {"--interval", "--equexit"}
PARALLEL_SHORT_OPTIONS_WITH_VALUE = "jSaIdNnEXsC"
PARALLEL_LONG_OPTIONS_WITH_VALUE = {"--jobs", "--sshlogin", "--arg-file", "--colsep", "--delimiter", "--joblog", "--results", "--max-args", "--timeout",
                                    "--tmpdir", "--workdir", "--env", "--basefile", "--return", "--transfer-file", "--retries", "--load", "--memfree"}
BWRAP_OPTION_ARITY = {option: 2 for option in ("--bind", "--bind-try", "--dev-bind", "--dev-bind-try", "--ro-bind", "--ro-bind-try", "--symlink", "--file",
                                               "--bind-data", "--ro-bind-data", "--setenv", "--chmod")}
BWRAP_OPTION_ARITY.update({option: 1 for option in ("--uid", "--gid", "--hostname", "--chdir", "--unsetenv", "--lock-file", "--sync-fd", "--remount-ro", "--exec-label",
                                                    "--file-label", "--proc", "--dev", "--tmpfs", "--mqueue", "--dir", "--seccomp", "--add-seccomp-fd", "--block-fd",
                                                    "--userns-block-fd", "--info-fd", "--json-status-fd", "--cap-add", "--cap-drop", "--perms", "--size", "--argv0",
                                                    "--userns", "--userns2", "--pidns", "--args", "--overlay-src", "--overlay", "--tmp-overlay", "--ro-overlay")})


def skipped_launcher_options(words, position, short_with_value, long_with_value):
    """从 position 起跳过一串选项（`--` 收尾也跳过），交回第一个不是选项的词的下标：短选项簇里头一个带值的字母之后贴着的是值、没有就取下一个词；
    长选项不带 `=` 而在带值表里的取下一个词。"""
    while position < len(words):
        word = words[position]
        if word == "--":
            return position + 1
        if not word.startswith("-") or word == "-":
            return position
        if word.startswith("--"):
            position += 2 if "=" not in word and word in long_with_value else 1
            continue
        consumed = 1
        for index, letter in enumerate(word[1:]):
            if letter in short_with_value:
                if index == len(word) - 2:
                    consumed = 2
                break
        position += consumed
    return position


def launcher_option_value(words, short_letter, long_name, short_with_value, long_with_value):
    """选项里 `-<字母> 值` / `-<字母>值` / `<长名>=值` / `<长名> 值` 的值（最后一个）；没有交 None。只看命令之前的选项段。"""
    found, position = None, 1
    while position < len(words):
        word = words[position]
        if word == "--" or not word.startswith("-") or word == "-":
            break
        if word.startswith("--"):
            name, equals, value = word.partition("=")
            if name == long_name:
                found = value if equals else (words[position + 1] if position + 1 < len(words) else None)
            position += 2 if not equals and name in long_with_value else 1
            continue
        consumed = 1
        for index, letter in enumerate(word[1:]):
            if letter in short_with_value:
                rest = word[index + 2:]
                value = rest or (words[position + 1] if position + 1 < len(words) else None)
                if letter == short_letter:
                    found = value
                consumed = 1 if rest else 2
                break
        position += consumed
    return found


def command_under_extra_launcher(words):
    """words[0] 是名单外启动器（EXTRA_LAUNCHER_NAMES）时交回 (它起的那条命令的词, 那条命令的当前目录或 None)；不是、或后面没有要起的命令交 None。
    逐词起的（doas、numactl、unshare、setpriv、nsenter、ltrace、valgrind、faketime、unbuffer、cpulimit、dbus-run-session、firejail、proot、chroot、cgexec、bwrap）剥掉选项与位置参数；
    xargs 交它起的命令，后面接一个占位词 XARGS_INPUT_WORD（-I / -i / --replace 的把替换串换成它），不给命令时是 echo；
    find 的 -exec / -execdir / -ok / -okdir 交那几条命令（{} 换成 FIND_PATH_WORD，几条用 ` ; ` 连成 sh -c）；
    ssh 交 `sh -c <host 之后的词用空格连起来>`；su / runuser / sg / script 的 -c 代码与 watch 的命令交 `sh -c <代码>`；runuser -u 用户交后面那条命令；
    gdb 只认 --args；scl 认 `scl enable <集合…> -- <命令>` 与末尾一段代码；parallel 交它起的命令再接 PARALLEL_INPUT_WORD（不给命令时整条是那个占位词）。"""
    if break_is_set("extra-launchers-unseen") or not words:
        return None
    name = os.path.basename(words[0])
    if name not in EXTRA_LAUNCHER_NAMES or ENVIRONMENT_ASSIGNMENT_WORD.match(words[0]):
        return None
    if name in EXTRA_LAUNCHER_OPTIONS:
        short_with_value, long_with_value, positional = EXTRA_LAUNCHER_OPTIONS[name]
        position = skipped_launcher_options(words, 1, short_with_value, long_with_value) + positional
        working_directory = None
        if name == "unshare":
            working_directory = launcher_option_value(words, "w", "--wd", short_with_value, long_with_value)
        elif name == "proot":
            working_directory = (launcher_option_value(words, "w", "--pwd", short_with_value, long_with_value)
                                 or launcher_option_value(words, "", "--cwd", short_with_value, long_with_value))
        return (words[position:], working_directory) if position < len(words) else None
    if name == "bwrap":
        position, working_directory = 1, None
        while position < len(words) and words[position].startswith("--"):
            option = words[position]
            if option == "--":
                position += 1
                break
            if option == "--chdir" and position + 1 < len(words):
                working_directory = words[position + 1]
            position += 1 + BWRAP_OPTION_ARITY.get(option, 0)
        return (words[position:], working_directory) if position < len(words) else None
    if name == "xargs":
        position = skipped_launcher_options(words, 1, XARGS_SHORT_OPTIONS_WITH_VALUE, XARGS_LONG_OPTIONS_WITH_VALUE)
        replacement = launcher_option_value(words, "I", "", XARGS_SHORT_OPTIONS_WITH_VALUE, XARGS_LONG_OPTIONS_WITH_VALUE)
        for word in words[1:position]:
            if word.startswith("--replace"):
                replacement = word.partition("=")[2] or "{}"
            elif word.startswith("-i") and not word.startswith("--"):
                replacement = word[2:] or "{}"
        inner = words[position:] or ["echo"]
        if replacement:
            return ([word.replace(replacement, XARGS_INPUT_WORD) for word in inner], None)
        return (inner + [XARGS_INPUT_WORD], None)
    if name == "find":
        commands, position = [], 1
        while position < len(words):
            if words[position] in ("-exec", "-execdir", "-ok", "-okdir"):
                end = position + 1
                while end < len(words) and words[end] not in (";", "+"):
                    end += 1
                if end > position + 1 and end < len(words):   # 没有收尾的 `;` / `+`：find 报 missing argument to -exec，什么都不起
                    commands.append([word.replace("{}", FIND_PATH_WORD) for word in words[position + 1:end]])
                position = end
            position += 1
        if not commands:
            return None
        return (commands[0], None) if len(commands) == 1 else (["sh", "-c", " ; ".join(shlex.join(command) for command in commands)], None)
    if name == "ssh":
        position = skipped_launcher_options(words, 1, SSH_SHORT_OPTIONS_WITH_VALUE, set())
        remote = words[position + 1:]
        return (["sh", "-c", " ".join(remote)], None) if remote else None
    if name in ("su", "runuser"):
        user = launcher_option_value(words, "u", "--user", SU_SHORT_OPTIONS_WITH_VALUE, SU_LONG_OPTIONS_WITH_VALUE) if name == "runuser" else None
        if user is not None:
            position = skipped_launcher_options(words, 1, SU_SHORT_OPTIONS_WITH_VALUE, SU_LONG_OPTIONS_WITH_VALUE)
            return (words[position:], None) if position < len(words) else None
        code = (launcher_option_value(words, "c", "--command", SU_SHORT_OPTIONS_WITH_VALUE, SU_LONG_OPTIONS_WITH_VALUE)
                or launcher_option_value(words, "", "--session-command", SU_SHORT_OPTIONS_WITH_VALUE, SU_LONG_OPTIONS_WITH_VALUE))
        return (["sh", "-c", code], None) if code else None
    if name == "sg":
        rest = words[1:]
        if rest[:1] == ["-"]:
            rest = rest[1:]
        rest = rest[1:]
        if rest[:1] == ["-c"]:
            rest = rest[1:]
        return (["sh", "-c", " ".join(rest)], None) if rest else None
    if name == "script":
        code = launcher_option_value(words, "c", "--command", SCRIPT_SHORT_OPTIONS_WITH_VALUE, SCRIPT_LONG_OPTIONS_WITH_VALUE)
        return (["sh", "-c", code], None) if code else None
    if name == "watch":
        position = skipped_launcher_options(words, 1, WATCH_SHORT_OPTIONS_WITH_VALUE, WATCH_LONG_OPTIONS_WITH_VALUE)
        return (["sh", "-c", " ".join(words[position:])], None) if position < len(words) else None
    if name == "gdb":
        for marker in ("--args", "-args"):
            if marker in words:
                inner = words[words.index(marker) + 1:]
                return (inner, None) if inner else None
        return None
    if name == "scl":
        if words[1:2] != ["enable"]:
            return None
        if "--" in words:
            inner = words[words.index("--") + 1:]
            return (inner, None) if inner else None
        return (["sh", "-c", words[-1]], None) if len(words) > 3 else None
    if name == "parallel":
        position = skipped_launcher_options(words, 1, PARALLEL_SHORT_OPTIONS_WITH_VALUE, PARALLEL_LONG_OPTIONS_WITH_VALUE)
        end = position
        while end < len(words) and words[end] not in (":::", "::::", ":::+", "::::+"):
            end += 1
        inner = words[position:end]
        if not inner:
            return ([PARALLEL_INPUT_WORD], None)
        if any("{" in word for word in inner):
            return ([re.sub(r"\{[^{}]*\}", PARALLEL_INPUT_WORD, word) for word in inner], None)
        return (inner + [PARALLEL_INPUT_WORD], None)
    return None


def launcher_names():
    """command_under_launcher 剥的全部启动器名：LAUNCHER_OPTIONS_WITH_VALUE 的键与名单外的那几个（EXTRA_LAUNCHER_NAMES；bash-command-detector.sh 的自证钉住它那一份与这里相同）。"""
    return frozenset(LAUNCHER_OPTIONS_WITH_VALUE) | EXTRA_LAUNCHER_NAMES


# 认不出起的是什么的写法（认不出的一律拒（偏严），records/2026-10-29-一百三十七节那一轮调度记录.md）：project-code-guard.sh 在命令文本这一层（不是读进去的脚本正文）判，交 UNRECOGNIZED_KIND 的 ProgramUse，
# 主 agent 与子 agent 一样拒。弄坏开关 LIB_PROJECT_CODE_RUNS_BREAK=unrecognized-launches-allowed 下一律交 None。
UNRECOGNIZED_KIND = "unrecognized"
UNRESOLVED_LAUNCH_MARKS = ("$", "`", "<(", ">(")
# cargo 起编出来的代码、范围由参数定的子命令：参数里有展开不了的词就认不出跑的是哪个包、哪些目标
CARGO_RUNNING_SUBCOMMANDS = frozenset({"test", "t", "run", "r", "bench", "nextest", "miri", "llvm-cov", "hack", "mutants"})
# 这几个选项的值写成变量不改跑的范围（编译目录、并行度、输出样子）
CARGO_HARMLESS_VALUE_OPTIONS = frozenset({"--target-dir", "-j", "--jobs", "--color", "--message-format"})
CARGO_GLOBAL_FLAGS_WITH_VALUE = frozenset({"--color", "--config", "-Z", "-C"})


# 脚本路径是怎么起的（lib_shell_words.CommandAtPosition.launcher）→ 认不出时说明里怎么称呼它
LAUNCHED_FILE_NAMES = {"shell": "bash / sh 起的脚本路径", "source": "source 读进来的文件", "python": "python 起的脚本路径",
                       "startup": "shell 启动文件（BASH_ENV / ENV、--rcfile、HOME 与 ZDOTDIR 下的）路径"}


def word_unresolved(word):
    """词里还有展开不了的东西：变量、命令替换、反引号、进程替换（引号已去掉；占位词 XARGS_INPUT_WORD 这类同样算）。"""
    return any(mark in word for mark in UNRESOLVED_LAUNCH_MARKS)


def cargo_arguments_expanded(words, variables):
    """cargo 这一条的参数里写成 `$名字`、`${名字}` 的，用这段命令文本里前面赋过的值换掉（lib_shell_words.expand_known_variables；整个词就是一个变量引用的，值按空白切成几个词）；
    命令词不是 cargo、展开不了的照原样留（交给 unrecognized_launch 判）。例如 `F=--workspace; cargo test $F` 要按展开后的参数判。
    弄坏开关 LIB_PROJECT_CODE_RUNS_BREAK=cargo-arguments-unexpanded 下原样交回。"""
    if not words or os.path.basename(words[0]) != "cargo" or break_is_set("cargo-arguments-unexpanded") or not variables:
        return words
    expanded = [words[0]]
    for word in words[1:]:
        value = shell_words.expand_known_variables(word, variables) if "$" in word and "`" not in word else None
        if value is None:
            expanded.append(word)
        elif re.fullmatch(r"\$(?:[A-Za-z_][A-Za-z0-9_]*|\{[A-Za-z_][A-Za-z0-9_]*\})", word):
            expanded += value.split()
        else:
            expanded.append(value)
    return expanded


# 会话自己那份可执行文件（与 bash-command-detector.sh 的 CLAUDE_EXECUTABLE_PATH_WORDS 同一组，自检核两边相同）：命令词是它的交检出钩子 ⑬ 判起 claude 会话
CLAUDE_EXECUTABLE_PATH_WORDS = frozenset({"$CLAUDE_CODE_EXECPATH", "${CLAUDE_CODE_EXECPATH}"})


def unrecognized_launch(words, launcher=None):
    """命令文本里这一条（已剥掉前缀与包装、命令词里赋过字面值的变量已展开）认不认得出起的是什么：认不出交一句说明，认得出交 None。认不出的三种：
    命令词（直接执行的命令、bash / source 起的脚本，或 shell 启动时先读的文件：BASH_ENV / ENV、--rcfile、HOME 与 ZDOTDIR 下的）带展开不了的变量、命令替换、位置参数、进程替换，或展开出来夹着空白（`c='cargo test'; $c` 照 bash 切词另成几个词）；
    source / . 后面的文件写成这样（`source <(…)`）；cargo 起编出来的代码的子命令（CARGO_RUNNING_SUBCOMMANDS）或它的参数（`--` 之后给测试二进制的、
    CARGO_HARMLESS_VALUE_OPTIONS 的值除外）带这样的词（`cargo $(echo test --workspace)`、`bash -c 'cargo "$@"' _ test --workspace` 里层的 `$@`）。
    命令词是 CLAUDE_EXECUTABLE_PATH_WORDS 的（会话自己那份可执行文件，检出钩子 ⑬ 认它是起 claude 会话、交那一道判）不算认不出（复跑分类报告 d137-replay-classify「四」口径；
    弄坏开关 LIB_PROJECT_CODE_RUNS_BREAK=claude-executable-path-unrecognized 下照认不出）。"""
    if break_is_set("unrecognized-launches-allowed") or not words:
        return None
    word = words[0]
    if launcher is None and word in CLAUDE_EXECUTABLE_PATH_WORDS and not break_is_set("claude-executable-path-unrecognized"):
        return None
    if word_unresolved(word) or any(blank in word for blank in " \t\n"):
        how = LAUNCHED_FILE_NAMES.get(launcher, "命令词")
        return f"{how} `{word}` 由变量、命令替换、位置参数或进程替换拼出，执行前展开不了，认不出起的是什么"
    name = os.path.basename(word)
    if launcher is None and name in ("source", ".") and len(words) > 1 and word_unresolved(words[1]):
        return f"source 读进来的 `{words[1]}` 是命令替换、进程替换或变量拼出的，执行前认不出读进来的是什么"
    if launcher is not None or name != "cargo":
        return None
    position = 1
    while position < len(words) and words[position].startswith(("-", "+")) and not word_unresolved(words[position]):
        position += 2 if words[position] in CARGO_GLOBAL_FLAGS_WITH_VALUE else 1
    if position >= len(words):
        return None
    subcommand = words[position]
    if word_unresolved(subcommand):
        return f"cargo 的子命令 `{subcommand}` 由变量或命令替换拼出，认不出跑的是什么"
    if subcommand not in CARGO_RUNNING_SUBCOMMANDS:
        return None
    rest = words[position + 1:]
    index = 0
    while index < len(rest):
        argument = rest[index]
        if argument == "--":
            break
        if argument in CARGO_HARMLESS_VALUE_OPTIONS:
            index += 2
            continue
        if any(argument.startswith(option + "=") for option in CARGO_HARMLESS_VALUE_OPTIONS):
            index += 1
            continue
        if word_unresolved(argument):
            return (f"cargo {subcommand} 的参数 `{argument}` 由变量、命令替换或位置参数拼出，执行前展开不了，认不出跑的是哪个包、哪些目标"
                    "（--workspace、-p singlefs-verification 都可能藏在里面）")
        index += 1
    return None


# 读进来的脚本正文里，命令词或脚本路径由变量拼出而解不出时，要看它起的是不是这几样（一百三十七节件 1b 第 3 条）：自证、门禁阶段、入口脚本（按名字判的那几份与
# Verus 两份判法、广谱变异驱动、prove-red.sh）、cargo。名字按词认（前后不接字母、数字、点、连字符、下划线）
SCRIPT_BODY_ENTRY_NAMES = tuple(sorted(set(KNOWN_SCRIPT_LOCATIONS) | {BROAD_MUTATION_SCRIPT_NAME, VERUS_PROOF_CHECK_SCRIPT, E165_PIN_SCRIPT, PROVE_RED_SCRIPT}))
SCRIPT_BODY_LAUNCH_SIGNATURES = (
    ("自证", re.compile(r"(?<![\w-])--selftest(?![\w-])")),
    ("门禁阶段", re.compile(r"\.claude/gate\.d/[\w.-]+\.sh(?![\w.-])")),
    ("入口脚本", re.compile(r"(?<![\w.-])(?:" + "|".join(re.escape(name) for name in SCRIPT_BODY_ENTRY_NAMES) + r")(?![\w.-])")),
    ("cargo", re.compile(r"(?<![\w.-])cargo(?![\w.-])")),
)
POSITIONAL_REFERENCE = re.compile(r"\$(?:[@*1-9]|\{[@*1-9][^}]*\})")
SCRIPT_SELF_REFERENCES = frozenset({"$0", "${0}", "$BASH_SOURCE", "${BASH_SOURCE}", "${BASH_SOURCE[0]}"})


def script_in_own_repository(path):
    """脚本的整条路径逐级都不是符号链接、落在本仓（HOOK_REPOSITORY，自证里另认 FAMILY_OWN_REPOSITORY）里：仓里的脚本受写范围闸与提交前的门禁管，
    script_body_unresolved_launch 不判它们（仓里 multi-host-run.sh、merge-and-verify.sh 这类本来就经变量起包装与命令）。
    别处的拷贝（仓副本里没改过的那一份）与本仓同一相对位置那一份逐字节相同的同样算：跑的就是本仓那一份的代码（同 same_content_as_own 的口径）。"""
    if not path:
        return False
    repositories = [os.path.normpath(repository) for repository in (HOOK_REPOSITORY, FAMILY_OWN_REPOSITORY) if repository]
    absolute = os.path.abspath(os.path.expanduser(path))
    walked = path_without_symbolic_links(absolute, "/")
    if walked is not None and any(walked.startswith(repository + os.sep) for repository in repositories):
        return True
    parts = absolute.strip("/").split("/")
    for start in range(1, len(parts) - 1):   # 至少两段（目录/文件），不拿仓根下的单个文件名去对
        suffix = os.path.join(*parts[start:])
        for repository in repositories:
            own = os.path.join(repository, suffix)
            if os.path.isfile(own):
                try:
                    return filecmp.cmp(own, absolute, shallow=False)
                except OSError:
                    return False
    return False


def script_body_unresolved_launch(words, launcher, definitions, positional=(), script_path=None):
    """读进来的脚本正文里这一条（命令词里赋过字面值的变量已展开）：命令词、bash / sh / source / python 起的脚本路径由变量、命令替换、位置参数拼出而解不出，
    而它连同它点到的变量在这份正文里的定义（variable_definitions，往下点到的接着收，最多 UNEXPANDED_DEFINITION_DEPTH 层）、点到位置参数时连同起这份脚本的那一条的参数
    （positional），里面写着自证（--selftest）、门禁阶段（.claude/gate.d/<名>.sh）、入口脚本（SCRIPT_BODY_ENTRY_NAMES）或 cargo 的：交一句说明（照认不出起什么的写法办）；
    cargo 是字面命令词而起编出来代码的子命令或参数解不出的，照 unrecognized_launch 的 cargo 那一半交说明。别的交 None。
    边界：定义里与位置参数里都点不到这几样的（`"$@"` 转发、`$cmd` 的定义里没有它们），照旧不判；正文所在的脚本（script_path）落在本仓里的不判
    （script_in_own_repository），本仓的脚本照常起（文件头「认不出的写法」）。
    弄坏开关 LIB_PROJECT_CODE_RUNS_BREAK=script-body-launches-unjudged 下一律交 None（脚本正文里经变量拼出的起法看不见）。"""
    if break_is_set("script-body-launches-unjudged") or not words:
        return None
    if words[0] in SCRIPT_SELF_REFERENCES or script_in_own_repository(script_path):
        return None   # 脚本起它自己（`bash "$0" --selftest`）：起的就是正在读的这一份，它的正文已经在判
    if not word_unresolved(words[0]):
        if launcher is None and os.path.basename(words[0]) == "cargo":
            found = unrecognized_launch(words, None)
            return f"脚本正文里 {found}" if found else None
        return None
    pool, pending, seen = list(words), set(), set()
    for word in words:
        pending.update(UNEXPANDED_VARIABLE_NAME.findall(word))
        if POSITIONAL_REFERENCE.search(word):
            pool += list(positional)
    for _ in range(UNEXPANDED_DEFINITION_DEPTH):
        reached = set()
        for name in pending - seen:
            seen.add(name)
            for value in (definitions or {}).get(name, ()):
                pool.append(value)
                reached.update(UNEXPANDED_VARIABLE_NAME.findall(value))
                if POSITIONAL_REFERENCE.search(value):
                    pool += list(positional)
        pending = reached - seen
        if not pending:
            break
    text = "\n".join(pool)
    for what, pattern in SCRIPT_BODY_LAUNCH_SIGNATURES:
        match = pattern.search(text)
        if match:
            how = LAUNCHED_FILE_NAMES.get(launcher, "命令词")
            return (f"脚本正文里这一条的{how} `{words[0]}` 由变量、命令替换或位置参数拼出，执行前解不出，而它与它点到的变量的定义"
                    f"{'、起这份脚本时给的参数' if positional and POSITIONAL_REFERENCE.search(text) else ''}里写着{what}（`{match.group(0)}`）："
                    "认不出起的是哪一份、经没经改过（改过的钩子或脚本的自证不在副本里起），照认不出起什么的写法办")
    return None


# 读脚本正文时，入口按自己所在的目录 source 的库（`source "$(dirname "${BASH_SOURCE[0]}")/lib.sh"`、`"${BASH_SOURCE[0]%/*}/…"`、`$(cd "$(dirname "$0")" && pwd)`）：
# 把这几种写法换成脚本所在的目录（照起它的那个路径，不解符号链接：bash 的 BASH_SOURCE 就是这个路径），source 那一行就成了字面路径，照读进来判（三方 m3-close-gov-b17 第一百一十八节跟进）。
SCRIPT_DIRECTORY_FORMS = re.compile(
    r'\$\(\s*cd\s+"?\$\(\s*dirname\s+(?:--\s+)?"?\$\{?(?:BASH_SOURCE(?:\[0\])?|0)\}?"?\s*\)"?\s*(?:&&|;)\s*pwd(?:\s+-P)?\s*\)'
    r'|\$\(\s*dirname\s+(?:--\s+)?"?\$\{?(?:BASH_SOURCE(?:\[0\])?|0)\}?"?\s*\)'
    r'|\$\{(?:BASH_SOURCE(?:\[0\])?|0)%/\*\}')


def script_directory_expanded(text, script_path):
    """脚本正文 text 里按自己所在目录取路径的写法（SCRIPT_DIRECTORY_FORMS）换成 script_path 所在的目录（照起它的路径，不解符号链接）；
    弄坏开关 LIB_PROJECT_CODE_RUNS_BREAK=sourced-libraries-unread 下原样交回（这种 source 的库读正文时看不见）。"""
    if break_is_set("sourced-libraries-unread") or not script_path:
        return text
    directory = os.path.dirname(os.path.abspath(script_path))
    return SCRIPT_DIRECTORY_FORMS.sub(lambda match: directory, text)


SCRIPT_DIRECTORY_VARIABLE = re.compile(r'^\s*(?:local\s+|readonly\s+|declare\s+(?:-\w+\s+)?)?([A-Za-z_][A-Za-z0-9_]*)=\S*(?:BASH_SOURCE|dirname\s+"?\$0)', re.MULTILINE)
SHELL_SOURCED_LIBRARY = re.compile(r'(?:^|[;&|\s])(?:source|\.)\s+"?(?:\$\(\s*dirname\s+(?:--\s+)?"?\$\{?(?:BASH_SOURCE(?:\[0\])?|0)\}?"?\s*\)'
                                   r'|\$\{(?:BASH_SOURCE(?:\[0\])?|0)%/\*\}|\$\{?(?P<variable>[A-Za-z_][A-Za-z0-9_]*)\}?)/(?P<path>[^"\s;&|)]+)', re.MULTILINE)
PYTHON_SIBLING_REFERENCE = re.compile(r"^\s*(?:import\s+([A-Za-z_][A-Za-z0-9_]*)|from\s+([A-Za-z_][A-Za-z0-9_]*)\s+import)|[\"']([\w.-]+\.(?:py|sh))[\"']", re.MULTILINE)
SOURCED_LIBRARY_DEPTH = 4


def script_sourced_libraries(path, depth=0, seen=None):
    """一份脚本按自己所在目录 source（shell）或 import / 点名（python：同目录下有这个 .py 的 import，字符串里写的同目录 .py / .sh）的库，连同那些库自己再引的：
    交回 [相对这份脚本所在目录的路径]（照写的样子，没解符号链接）。认不出的写法不在里面。"""
    seen = set() if seen is None else seen
    try:
        text = open(path, encoding="utf-8", errors="replace").read(MAXIMUM_LIBRARY_SCAN_BYTES)
    except OSError:
        return []
    found = []
    directory = os.path.dirname(path)
    if path.endswith(".py"):
        for match in PYTHON_SIBLING_REFERENCE.finditer(text):
            name = match.group(1) or match.group(2)
            relative = f"{name}.py" if name else match.group(3)
            if relative and os.path.isfile(os.path.join(directory, relative)):
                found.append(relative)
    else:
        variables = set(SCRIPT_DIRECTORY_VARIABLE.findall(text))
        for match in SHELL_SOURCED_LIBRARY.finditer(text):
            if match.group("variable") and match.group("variable") not in variables:
                continue
            found.append(match.group("path"))
    libraries = []
    for relative in found:
        full = os.path.normpath(os.path.join(directory, relative))
        if full in seen:
            continue
        seen.add(full)
        libraries.append(relative)
        if depth + 1 < SOURCED_LIBRARY_DEPTH:
            libraries += [os.path.normpath(os.path.join(os.path.dirname(relative), inner)) for inner in script_sourced_libraries(full, depth + 1, seen)]
    return libraries


MAXIMUM_LIBRARY_SCAN_BYTES = 1 << 20


def variable_definitions(text, depth=0):
    """一段命令文本里每个变量可能取到的值（照写的样子，没展开）：{名字: [值…]}。收 `名=值` 的词（命令前的、单独一条的、export / local 这类的参数）
    与 `for 名 in 词…` 的每个词，命令替换里面的同样收；同一个名字几处定义都留。切词照 lib_shell_words（引号去掉，命令替换原样留在所在的词里）。"""
    definitions = {}
    try:
        tokens = shell_words.shell_tokens(shell_words.strip_data_heredocs(text))
        simple, _targets, substitutions, _redirections = shell_words.simple_commands(tokens)
    except Exception:   # 切不动的文本：一个定义都不收
        return definitions
    for words, inner_texts in zip(simple, substitutions):
        if depth < UNEXPANDED_DEFINITION_DEPTH:
            for inner in inner_texts:
                for name, values in variable_definitions(inner, depth + 1).items():
                    definitions.setdefault(name, []).extend(values)
        if len(words) >= 3 and words[0] == "for" and words[2] == "in":
            definitions.setdefault(words[1], []).extend(words[3:])
        for word in words:
            assignment = shell_words.ASSIGNMENT.match(word)
            if assignment:
                definitions.setdefault(assignment.group(1), []).append(assignment.group(2))
    return definitions


def command_word_expansions(word, directory):
    """命令词带通配时 bash 展开成的那几份路径（相对的按 directory 解、交回仍是相对的写法），排好序；零命中、相对路径而当前目录认不出的交 []
    （bash 照字面起）。只是 `[` / `[[` 的不展开（test 的内建命令）。bash 的 [^…] 换成 python glob 认的 [!…]（弄坏开关 pattern-caret-negation-literal 下不换）。
    带 POSIX 字符类、等价类、整理符号的（BASH_BRACKET_PATTERN）交给 bash 展开（bash_pattern_expansions，次序照 bash 的），判不出交 None
    （弄坏开关 posix-brackets-by-python-glob 下照旧交给 python glob）。"""
    if word in ("[", "[["):
        return []
    pattern = os.path.expanduser(word)
    if BASH_BRACKET_PATTERN.search(pattern) and not break_is_set("posix-brackets-by-python-glob"):
        return bash_pattern_expansions(pattern, directory)
    if not break_is_set("pattern-caret-negation-literal"):
        pattern = CARET_NEGATION.sub("[!", pattern)
    if os.path.isabs(pattern):
        return sorted(glob.glob(pattern))
    if directory is None:
        return []
    return sorted(glob.glob(pattern, root_dir=directory))


def bash_pattern_expansions(pattern, directory):
    """带 POSIX 字符类、等价类、整理符号的词交给 bash 展开成的那几份（次序照 bash 的，相对的仍是相对的写法）；零命中交 []。
    不起被判的命令：词经 $1 传进去、IFS 设空，只做路径展开；不读 rc 与 BASH_ENV。展不开或判不出的（相对路径而当前目录认不出、
    bash 起不来、退出码不是 0、超过 BASH_PATTERN_TIMEOUT 秒）交 None；弄坏开关 posix-brackets-unresolved-literal 下这几种也交 []（照字面判）。"""
    unresolved = [] if break_is_set("posix-brackets-unresolved-literal") else None
    absolute = os.path.isabs(pattern)
    if not absolute and directory is None:
        return unresolved
    environment = {key: value for key, value in os.environ.items() if key not in ("BASH_ENV", "ENV")}
    try:
        finished = subprocess.run([BASH_FOR_PATTERNS, "--norc", "--noprofile", "-c", 'shopt -s nullglob; IFS=; for f in $1; do printf "%s\\0" "$f"; done', "bash", pattern],
                                  cwd="/" if absolute else directory, env=environment, capture_output=True, timeout=BASH_PATTERN_TIMEOUT)
    except (OSError, subprocess.SubprocessError):
        return unresolved
    if finished.returncode != 0:
        return unresolved
    return [os.fsdecode(item) for item in finished.stdout.split(b"\0") if item]


def pattern_expansion_use(words, directory, environment=None):
    """命令词带通配（文件头「命令词带通配」）：按 directory 展开，bash 真起的那一条（头一份当命令、其余几份接在写的参数前面）与每一份单独当命令
    （带写的参数，最多 PATTERN_EXPANSION_LIMIT 份）各判一遍；有一条判成哪一类就交它（说明里写展开成了什么），都不是交 None（调用方再照字面判）。
    展开出来的那条当一段命令文本再切一遍、剥包装（写成通配的 capped.sh、nice、/usr/bin/time 这一类也认）；弄坏开关 pattern-expansions-bare 下直接 classify。"""
    matches = command_word_expansions(words[0], directory)
    if matches is None:
        where = "当前目录认不出" if directory is None and not os.path.isabs(os.path.expanduser(words[0])) else "交给 bash 展开没展成"
        return program_use("pattern-unresolved", f"命令词 {words[0]} 带方括号里的 POSIX 字符类、等价类或整理符号，{where}，判不出 bash 起的是哪一份"
                                                "（写成展开之后的字面路径再起）")
    if not matches:
        return None
    candidates = ([[*matches, *words[1:]]] if len(matches) > 1 else []) + [[match, *words[1:]] for match in matches[:PATTERN_EXPANSION_LIMIT]]
    global pattern_expansion_depth
    for candidate in candidates:
        pattern_expansion_depth += 1
        try:
            if GLOB_CHARACTERS & set(candidate[0]) or break_is_set("pattern-expansions-bare"):
                literal = classify(candidate, directory, environment, expand_patterns=False)
                tests = [literal] if literal else []
            else:
                tests = program_uses_in_command_text(shlex.join(candidate), directory, environment)
        finally:
            pattern_expansion_depth -= 1
        if tests:
            more = f" 等 {len(matches)} 份" if len(matches) > 1 else ""
            where = "" if os.path.isabs(os.path.expanduser(words[0])) else f"按 {directory} "
            return tests[0]._replace(detail=f"{tests[0].detail}（命令词 {words[0]} {where}展开成 {candidate[0]}{more}，照展开的判）")
    return None


class BraceScanTooLong(Exception):
    """一个词里配不上的花括号太多，扫描步数超过 BRACE_SCAN_LIMIT：这个词不展开。"""


def brace_gobble(word, position, wanted, budget):
    """照 bash braces.c 的 brace_gobbler：从 position 起找最外层的 wanted（`{`、`}` 或 `,`），交下标，找不到交 None。
    `${` 算进一层、它的 `{` 不算开头；里层的 `{`、`}` 按层数配，最外层多出来的 `}` 当普通字符；
    找 `}` 时要先见过最外层的逗号或 `..`（后头紧跟的不是 `}`），之前见到的最外层 `}` 当普通字符（{}a,b} 配的是后头那个）。
    只看这几种记号（BRACE_TOKENS），每看一个扣 budget[0] 一步，扣完抛 BraceScanTooLong。"""
    level, seen = 0, wanted != "}"
    for found in BRACE_TOKENS.finditer(word, position):
        budget[0] -= 1
        if budget[0] < 0:
            raise BraceScanTooLong()
        token, at = found.group(), found.start()
        if token == "${":
            level += 1
        elif token == wanted and level == 0 and seen:
            if token == "{" and at == 0 and word[1:2] in ("}", ""):
                continue   # 打头的 `{}`（find -exec 的那种）与单独一个 `{` 照 bash 不算开头
            return at
        elif token == "{":
            level += 1
        elif token == "}":
            level -= 1 if level else 0
        elif wanted == "}" and level == 0 and (token == "," or word[at + 2:at + 3] != "}"):
            seen = True
    return None


def brace_sequence(amble):
    """{<起>..<止>[..<步长>]} 里面那一段照 bash 展开成的几份（最多 WORD_EXPANSION_LIMIT 份）；不是合法的序列交 None。
    整数照 bash 补零：起或止写成 0 起头的两位以上（或 -0 起头的三位以上）时，宽度取这样写的那几边的长度里大的那个（与 lib_shell_words.py 共用 brace_sequence_width）。
    弄坏开关 brace-sequence-literal 下一律交 None。"""
    found = BRACE_SEQUENCE.fullmatch(amble)
    if found is None or break_is_set("brace-sequence-literal"):
        return None
    first, last, step = found.group(1), found.group(2), abs(int(found.group(3) or 1)) or 1
    if first.isalpha() != last.isalpha():
        return None
    if first.isalpha():
        begin, end, render = ord(first), ord(last), chr
    else:
        width = shell_words.brace_sequence_width(first, last)
        begin, end = int(first), int(last)
        render = (lambda value: f"{value:0{width}d}") if width else str
    step = step if begin <= end else -step
    return [render(value) for value in range(begin, end + (1 if step > 0 else -1), step)[:WORD_EXPANSION_LIMIT]]


def brace_expansion(word):
    """一个词照 bash 的花括号展开成的几份（最多 WORD_EXPANSION_LIMIT 份），没有能展开的交 [word]；展开出来的空串照 bash 去掉。
    认的写法：{a,b}（最外层至少一个逗号，各份再各自展开）与序列 {1..3}、{a..e}、{01..10..3}；前后缀照接，几处花括号照乘。
    配不上的 `{`、里面最外层没有逗号又不是序列的（{a}、{}、{1...3}）照字面，接着往后找；`${…}` 是变量，不展开、里面也不找。
    引号里的花括号不展开：切词交来的 ShellWord 带着引号信息，unquoted_braces 为假的照字面交 [word]（函数体外引号里的花括号：放，
    `'{cargo,test,--all}'` 在命令词位置照字面、不当 cargo test）；不是 ShellWord 的普通 str（没有引号信息）照展。
    弄坏开关 brace-expansion-literal 下一律交 [word]，quoted-braces-expanded 下引号里的花括号也照展。"""
    if break_is_set("brace-expansion-literal") or "{" not in word or "}" not in word or ("," not in word and ".." not in word):
        return [word]
    if isinstance(word, shell_words.ShellWord) and not word.unquoted_braces and not break_is_set("quoted-braces-expanded"):
        return [word]
    try:
        results = [result for result in brace_expansion_with_empties(word, [BRACE_SCAN_LIMIT]) if result]
    except BraceScanTooLong:
        return [word]
    return results[:WORD_EXPANSION_LIMIT]


def brace_expansion_with_empties(word, budget):
    """照 bash braces.c 的 brace_expand（空串留着，brace_expansion 再去掉）；budget 见 brace_gobble。"""
    opening = brace_gobble(word, 0, "{", budget)
    if opening is None:
        return [word]
    prefix, start = word[:opening], opening + 1
    close = brace_gobble(word, start, "}", budget)
    if close is None:   # 配不上：这个 `{` 照字面，接着往后找
        return [prefix + "{" + rest for rest in brace_expansion_with_empties(word[start:], budget)][:WORD_EXPANSION_LIMIT]
    amble = word[start:close]
    if "," in amble:   # 照 bash：里面哪一层有逗号都按逗号切（只切最外层的），各份再各自展开
        middles, position = [], 0
        while True:
            comma = brace_gobble(amble, position, ",", budget)
            middles += brace_expansion_with_empties(amble[position:len(amble) if comma is None else comma], budget)
            if comma is None:
                break
            position = comma + 1
    else:
        middles = brace_sequence(amble)
        if middles is None:
            if close + 1 >= len(word):
                return [word]
            middles = [word[opening:close + 1]]   # 不是序列（{1...3}）：照字面，后头接着展开
    suffixes = brace_expansion_with_empties(word[close + 1:], budget)
    return [prefix + middle + suffix for middle in middles[:WORD_EXPANSION_LIMIT] for suffix in suffixes][:WORD_EXPANSION_LIMIT]


def expanded_words(words):
    """bash 对这一条命令的词做的花括号展开（每个词各展开；命令词展开成几份时，头一份是命令、其余接在参数前面）。
    参数里的通配不在这里展开：判成哪一类只看命令词和只问版本的参数写法，参数里的通配展开与否判不出差别；命令词的通配交 pattern_expansion_use。"""
    return [expanded for word in words for expanded in brace_expansion(word)] or words


word_expansion_depth = 0   # word_expansion_use 套了几层（展开出来的文件名里本身带花括号时，再展开有个头）
pattern_expansion_depth = 0   # pattern_expansion_use 套了几层：花括号、通配展开出来的那条命令里本仓的代码、编译也要认（project_code_program_use）


def project_code_program_use(words, directory, launcher, wrappers=()):
    """展开（花括号、通配）出来的那条命令是本仓的代码、会编的 cargo 或编出来的二进制：交 ProgramUse（kind project-code），不是交 None。
    字面写的由 project_code_use、starts_a_build、runs_compiled_code 在命令位置上直接认；写成 bash 才展得开的写法（`bash .claude/gate.d/x.s?`、`{cargo,test}`）
    绕过那三样，所以展开之后在这里再认一遍。名单上谁都能直接跑的、只给主 agent 的会话工具不算。展开出来的是 capped.sh、run-with-memory-cap.sh 这类
    切词剥掉的包装（wrappers，交给服务的命令不带它们）的，包装本身就是本仓的代码，同样算。弄坏开关 expanded-project-code-unjudged 下一律交 None。"""
    if break_is_set("expanded-project-code-unjudged"):
        return None
    if wrappers:
        return program_use("project-code", f"经 {os.path.basename(wrappers[0][0])} 直接起（交给服务的命令不带包装）")
    project = project_code_use(words, directory, launcher)
    if project is not None and project.allowed == "nobody":
        return program_use("project-code", f"本仓的 {project.detail}")
    compiled = runs_compiled_code(words, directory) or starts_a_build(words, directory)
    if compiled:
        return program_use("project-code", compiled)
    return None


def word_expansion_use(words, directory, environment=None):
    """花括号（文件头「入口」classify 一段）：照 bash 展开（expanded_words），与写的不同就判 bash 真起的那一条；
    命令词自己花括号展开成几份的，再把每一份单独当命令、带展开过的参数各判一遍（最多 PATTERN_EXPANSION_LIMIT 份，与命令词带通配同一个口径）。
    命令词展开过的，展开出来的那条当一段命令文本再切一遍、剥包装（花括号写在 bash、nice 这类包装上的也认；弄坏开关 brace-command-words-bare 下直接 classify）。
    有一条判成哪一类就交它（说明里写展开成了什么），都不是、或没有能展开的交 None（调用方再照字面判）。"""
    global word_expansion_depth
    expanded = expanded_words(words)
    if expanded == words or word_expansion_depth >= WORD_EXPANSION_DEPTH_LIMIT:
        return None
    alternatives = brace_expansion(words[0])
    candidates = [expanded]
    if len(alternatives) > 1:
        arguments = [braced for word in words[1:] for braced in brace_expansion(word)]
        candidates += [[alternative, *arguments] for alternative in alternatives[:PATTERN_EXPANSION_LIMIT]]
    resplit = alternatives != [words[0]] and not break_is_set("brace-command-words-bare")
    found = []   # (判出的那一类, 那一条)
    word_expansion_depth += 1
    try:
        for candidate in candidates:
            if resplit:
                tests = program_uses_in_command_text(shlex.join(candidate), directory, environment)
            else:
                literal = classify(candidate, directory, environment)
                tests = [literal] if literal else []
            found += [(test, candidate) for test in tests]
            if tests:
                break
    finally:
        word_expansion_depth -= 1
    if not found:
        return None
    test, candidate = found[0]
    shown = shlex.join(candidate)
    shown = shown if len(shown) <= 200 else shown[:200] + " …"
    how = "bash 真起的那一条" if candidate is expanded else f"命令词 {words[0]} 展开出来的一份单独当命令"
    return test._replace(detail=f"{test.detail}（照 bash 的花括号展开与参数里的通配展开，{how}是 {shown}，照展开的判）")


def classify(words, directory, environment=None, expand_patterns=True):
    """一条已经剥掉前缀与包装的命令：返回 ProgramUse 或 None。environment 是这条命令看得到的变量。
    先照 bash 做花括号展开、展开出来的那一条判成哪一类就交它（word_expansion_use）；
    命令词带通配（*、?、[）的再照 bash 展开、逐个判（pattern_expansion_use），都判不出再照字面判；expand_patterns 为假时只照字面判
    （展开出来的文件名里本身带通配字符时用，免得再展开一遍）。弄坏开关 pattern-command-words-literal 下命令词的通配不展开。
    字面判的只有一样：外部程序名单（external_program_use）里的程序，名单外的一律交 None——本仓的代码由 project_code_use 认，会编的与编出来的由
    starts_a_build、runs_compiled_code 认，认不出起什么的写法由 unrecognized_launch 认。"""
    if expand_patterns:
        expanded = word_expansion_use(words, directory, environment)
        if expanded is not None:
            return expanded
    if expand_patterns and GLOB_CHARACTERS & set(words[0]) and not break_is_set("pattern-command-words-literal"):
        expanded = pattern_expansion_use(words, directory, environment)
        if expanded is not None:
            return expanded
    return external_program_use(os.path.basename(words[0]), words[1:])


def external_program_use(name, arguments):
    """命令词（取 basename）是 EXTERNAL_PROGRAM_NAMES 或 EXTERNAL_PROGRAM_PREFIXES 里的程序：交 ProgramUse，不是、或只问版本交 None。
    弄坏开关 external-programs-unlisted 下一律交 None；herd7-version-flag-counted 下 herd7 只问版本也算。"""
    if break_is_set("external-programs-unlisted"):
        return None
    if not (name in EXTERNAL_PROGRAM_NAMES or name.startswith(EXTERNAL_PROGRAM_PREFIXES)):
        return None
    if arguments in EXTERNAL_PROGRAM_VERSION_ONLY_ARGUMENTS.get(name, ()) and not break_is_set("herd7-version-flag-counted"):
        return None   # 只打程序的版本号
    return program_use("external-program", f"{name}（本仓之外的程序，PATH 上直接起；跑它和跑本仓的代码一样经 task-server/task submit --）")


def program_uses_in_command_text(text, directory, environment=None):
    """一段命令文本里命令位置上的每一处外部程序用法：lib_shell_words 剥掉包装，command_under_launcher 剥掉 /usr/bin/time 这一类（剥完再切一遍），逐条 classify。"""
    found = []
    for command in shell_words.commands_at_command_position(text, directory, environment).commands:
        command = command._replace(words=command_word_expanded(command.words, command.variables))
        launched = command_under_launcher(command.words, command.directory) or command_under_gpu_quiesce(command.words, command.directory)
        if launched is not None:
            inner_words, inner_directory = launched
            inner_start = shell_words.resolve_path(command.directory, inner_directory) if inner_directory else command.directory
            found += program_uses_in_command_text(shlex.join(inner_words), inner_start, command.environment)
            continue
        test = classify(command.words, command.directory, command.environment)
        if not test and (word_expansion_depth or pattern_expansion_depth):
            test = project_code_program_use(command.words, command.directory, command.launcher, command.wrappers)
        if test:
            found.append(test)
    return found


def known_script_at_location(word, directory):
    """命令词（照写的样子，或按当前目录解开之后）是 KNOWN_SCRIPT_LOCATIONS 登记的名字、路径又以它的仓内位置收尾。只看路径尾部，不核是不是本仓那一份。"""
    location = KNOWN_SCRIPT_LOCATIONS.get(os.path.basename(word))
    if location is None:
        return False
    return any(candidate == location or candidate.endswith("/" + location)
               for candidate in (os.path.normpath(word), shell_words.resolve_path(directory, word) or ""))


def path_without_symbolic_links(path, directory):
    """路径（相对路径接在 directory 后面）从根一级一级走下去，每一级都不是符号链接时交走到的绝对路径，否则交 None；
    相对路径而 directory 认不出（None、不是绝对路径）的也交 None。.. 取父目录，走到它时前面各级已确认不是符号链接，所以字面上的父目录就是真实的父目录。"""
    if os.path.isabs(path):
        text = path
    elif directory is not None and os.path.isabs(directory):
        text = os.path.join(directory, path)
    else:
        return None
    walked = "/"
    for part in text.split("/"):
        if part in ("", "."):
            continue
        if part == "..":
            walked = os.path.dirname(walked)
            continue
        walked = os.path.join(walked, part)
        if os.path.islink(walked):
            return None
    return walked


def judged_by_name(word, directory):
    """按名字认的仓内入口脚本（门禁阶段、KNOWN_SCRIPT_LOCATIONS 里的脚本在仓内位置上）：起它算跑本仓的代码（project_code_use 认），里面起什么由它们自己的自证与门禁管，
    读脚本正文的一方不再读进去。盘上有这份文件、又不是本仓那一份（路径尾部相同的拷贝、经符号链接起的、内容连同它 source / import 的库与本仓那一份不逐字节相同的）为假、照读正文；
    盘上没有的照旧按路径尾部为真（bash 起不来，没有正文可读）。弄坏开关 same-family-by-path-tail 下只看路径尾部。"""
    stage = gate_stage_name(word, directory)
    if stage is not None:
        # 门禁阶段同一口径：盘上有、又不是本仓 .claude/gate.d/ 里那一份的拷贝照读正文（第一百一十八节「门禁阶段与包装脚本同一口径」）；弄坏开关 same-family-by-path-tail 下照旧只看文件名
        if break_is_set("same-family-by-path-tail"):
            return True
        resolved = shell_words.resolve_path(directory, os.path.expanduser(word))
        if resolved is None or not os.path.isfile(resolved):
            return True
        return (family_own_copy(word, directory, os.path.join(GATE_STAGE_DIRECTORY, stage))
                or same_content_as_own(resolved, os.path.join(GATE_STAGE_DIRECTORY, stage)))
    name = os.path.basename(word)
    if KNOWN_SCRIPT_LOCATIONS.get(name) is None or not known_script_at_location(word, directory):
        return False
    # 同族的别的几份（gate.sh、mutate.sh、capped.sh、run-with-memory-cap.sh、崩溃放量另两份……，第九十八节 ②）同一口径：盘上有这份文件、又不是本仓那一份（路径尾部相同的拷贝、
    # 经符号链接起的）照读正文；盘上没有的照旧按名字判（bash 起不来，没有正文可读）。弄坏开关 same-family-by-path-tail 下退回只按路径尾部认
    if not break_is_set("same-family-by-path-tail") and existing_copy_elsewhere(word, directory):
        return False
    return True


# 自证里换成样本工作区：样本里的同族脚本照「本仓那一份」认（judged_by_name 那几格测的是名字与参数，不是在哪棵树里）；平常是 None，只认 HOOK_REPOSITORY
FAMILY_OWN_REPOSITORY = None


def family_own_copy(word, directory, location):
    """命令词的整条路径逐级都不是符号链接、走到的正是本仓（HOOK_REPOSITORY，自证里另认 FAMILY_OWN_REPOSITORY）里 location 那一份（同 own_repository_script）。"""
    walked = path_without_symbolic_links(os.path.expanduser(word), directory)
    return walked is not None and any(walked == os.path.normpath(os.path.join(repository, location))
                                      for repository in (HOOK_REPOSITORY, FAMILY_OWN_REPOSITORY) if repository)


def existing_copy_elsewhere(word, directory):
    """命令词解到的文件在盘上、却不是本仓那一份（路径里有符号链接，或落在别的树里）；名字不在 KNOWN_SCRIPT_LOCATIONS 里的交 False。"""
    location = KNOWN_SCRIPT_LOCATIONS.get(os.path.basename(word))
    if location is None:
        return False
    resolved = shell_words.resolve_path(directory, os.path.expanduser(word))
    return (resolved is not None and os.path.isfile(resolved) and not family_own_copy(word, directory, location)
            and not same_content_as_own(resolved, location))


def same_content_as_own(path, location):
    """别处那一份与本仓 location 那一份逐字节相同、它按自己所在目录 source / import 的库（script_sourced_libraries）在别处那边有的也都相同：
    跑的就是本仓那一份的代码，照本仓那一份认。弄坏开关 family-copy-content-unchecked 下一律交 False。"""
    if break_is_set("family-copy-content-unchecked"):
        return False
    own = os.path.join(HOOK_REPOSITORY, location)
    digest = file_sha256(path)
    if digest is None or digest != file_sha256(own):
        return False
    for relative in script_sourced_libraries(path):
        copy_library = os.path.normpath(os.path.join(os.path.dirname(path), relative))
        if os.path.lexists(copy_library) and file_sha256(copy_library) != file_sha256(os.path.normpath(os.path.join(os.path.dirname(own), relative))):
            return False
    return True


def build_sample_workspace(work):
    """自证与 project-code-guard.sh 自证共用的虚工作区：仓根的工作区清单与两个成员（core、验证包），launcher 自证读的环境文件，
    门禁样本（喂给 stage-selftest.sh 的几份阶段目录与它们比的样本仓 index-reference），别处树里路径尾部相同的拷贝。"""
    def write(relative, text):
        path = os.path.join(work, relative)
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "w", encoding="utf-8") as handle:
            handle.write(text)
    write("Cargo.toml", '[workspace]\nresolver = "2"\nmembers = ["crates/singlefs-core", "crates/singlefs-verification"]\n')
    write("crates/singlefs-core/Cargo.toml", '[package]\nname = "singlefs-core"\nversion = "0.0.0"\n')
    write("crates/singlefs-core/tests/core_contract.rs", "")
    write("crates/singlefs-verification/Cargo.toml", '[package]\nname = "singlefs-verification"\nversion = "0.0.0"\n')
    write("crates/singlefs-verification/tests/overwrite_in_one_instance.rs", "#[test]\nfn a_daily_case() {}\n")
    write("alias.env", '# 样本：systemd-run -p EnvironmentFile= 读的\nCARGO_ALIAS_XT="test -p singlefs-verification"\n')
    write("plain.env", "# 样本\nRUST_BACKTRACE=1\n")
    # 门禁样本（文件头「门禁样本」）：这份文件所在仓里那几道阶段的原样拷贝，与喂给 stage-selftest.sh 的几份阶段目录
    def registered_stage(stage):
        try:
            with open(os.path.join(HOOK_REPOSITORY, GATE_STAGE_DIRECTORY, stage), encoding="utf-8") as handle:
                return handle.read()
        except OSError:
            return f"# 这份文件所在仓里没有 {stage}（门禁样本那几格会红）\n"
    for stage in ("doc-text.sh", "doc-decisions.sh"):
        write(f".claude/gate.d/{stage}", registered_stage(stage))
    for gate_directory, stages in (("gd-registered", {"doc-text.sh": registered_stage("doc-text.sh"), "doc-decisions.sh": registered_stage("doc-decisions.sh")}),
                                   ("gd-unregistered", {"brand-new-stage.sh": "#!/usr/bin/env bash\n# 样本：登记表外的阶段\n"})):
        for stage, body in stages.items():
            write(f"{gate_directory}/.claude/gate.d/{stage}", body)
            write(f"{gate_directory}/.claude/gate.d/fixtures/{stage}/red/expect", "exit=1\n")
    write("gd-registered/.claude/gate.d/code-tooling.sh", "#!/usr/bin/env bash\n# 样本：没有样本目录的阶段，stage-selftest.sh 不喂它\n")
    # stage-selftest.sh 喂的样本与暂存区比（文件头「门禁样本」）：index-reference 是临时建的样本仓（自检把 SAMPLE_INDEX_REPOSITORY 换到它），
    # 暂存区里是 gd-registered 那两道的样本；另造几份与它不同的阶段目录：样本里多一份 setup.sh、内容改过、执行位改过、少了文件，与样本仓自己的阶段目录（暂存之后才新加的一份样本）
    reference = os.path.join(work, SAMPLE_INDEX_REFERENCE_NAME)
    for stage in ("doc-text.sh", "doc-decisions.sh"):
        write(f"{SAMPLE_INDEX_REFERENCE_NAME}/.claude/gate.d/fixtures/{stage}/red/expect", "exit=1\n")
    git_environment = {name: value for name, value in os.environ.items() if not name.startswith("GIT_")}
    for git_arguments in (["init", "-q"], ["add", "-A"]):
        subprocess.run(["git", "-C", reference, *git_arguments], check=True, capture_output=True, env=git_environment)
    write(f"{SAMPLE_INDEX_REFERENCE_NAME}/.claude/gate.d/doc-text.sh", registered_stage("doc-text.sh"))
    write(f"{SAMPLE_INDEX_REFERENCE_NAME}/.claude/gate.d/fixtures/doc-text.sh/green/expect", "exit=0\n")
    for gate_directory, extra in (("gd-sample-setup", ("red/setup.sh", "#!/usr/bin/env bash\ncargo test --release -p singlefs-verification\n")),
                                  ("gd-sample-edited", ("red/expect", "exit=0\n")), ("gd-sample-mode", None), ("gd-sample-missing", None)):
        write(f"{gate_directory}/.claude/gate.d/doc-text.sh", registered_stage("doc-text.sh"))
        if gate_directory == "gd-sample-missing":
            os.makedirs(os.path.join(work, gate_directory, ".claude", "gate.d", "fixtures", "doc-text.sh", "green"))   # 只剩一个空的样本目录
        elif extra is None or extra[0] != "red/expect":
            write(f"{gate_directory}/.claude/gate.d/fixtures/doc-text.sh/red/expect", "exit=1\n")
        if extra:
            write(f"{gate_directory}/.claude/gate.d/fixtures/doc-text.sh/{extra[0]}", extra[1])
    os.chmod(os.path.join(work, "gd-sample-mode", ".claude", "gate.d", "fixtures", "doc-text.sh", "red", "expect"), 0o755)
    # 自证登记表按真实路径认：别处树里路径尾部相同的拷贝、指到本仓那一份的符号链接
    write("evil-tree/scripts/kb/claim-experiment.sh", "#!/usr/bin/env bash\n# 改过的自证\nherd7 x.litmus\n")
    write("evil-tree/.claude/singlefs-ai-sop/scripts/stage-selftest.sh", "#!/usr/bin/env bash\n# 样本：改过的 stage-selftest.sh，不认点名\n")
    os.makedirs(os.path.join(work, "linked-tree", "scripts", "gate"))
    os.symlink(os.path.join(HOOK_REPOSITORY, "scripts", "gate", "admission.py"), os.path.join(work, "linked-tree", "scripts", "gate", "admission.py"))
    # case 的模式行（攻方扫描脚本里 case 的那一行原样）：切词把模式切成一条命令
    write("case-pattern.sh", "#!/usr/bin/env bash\nkey=\"$1\"; root=\"$2\"\ncase \"$key\" in\n  .claude/gate.d/lib/*.py) cmd=(python3 \"$root/$key\") ;;\n"
                             "  *) cmd=(bash \"$root/$key\" --selftest) ;;\nesac\necho \"${cmd[@]}\"\n")


# 自检临时建的样本仓（build_sample_workspace 造在临时工作区里，自检把 SAMPLE_INDEX_REPOSITORY 换到它）
SAMPLE_INDEX_REFERENCE_NAME = "index-reference"
SAMPLE_SELFTEST_REGISTRY = (
    "# 样本登记表（lib_project_code_runs 自证造的）\n"
    "scripts/gate/sample-selftest.sh\tselftest\t# 样本：假驱动\n"
    "scripts/gate/sample-bad-kind.sh\tmedium\t# 样本：种类写坏了\n"
    "scripts/gate/sample-short-reason.sh\tselftest\t# 短\n"
    "scripts/gate/sample-pat*.sh\tselftest\t# 样本：文件名里带星号，命令词带通配时不当登记过的自证\n"
    "scripts/gate/sample-missing.sh\tselftest\t# 样本：文件不在\n"
    "elsewhere/sample-outside.sh\tselftest\t# 样本：不在登记目录里\n"
    "scripts/gate/sample-selftest.sh\tselftest\t# 样本：重复的一行\n"
    ".claude/gate.d/sample-stage.sh\tsamples:.sample-tools\t# 样本：样本根放了假工具\n"
    ".claude/gate.d/sample-stage-plain.sh\tsamples\t# 样本：样本只读文本\n"
    ".claude/gate.d/sample-stage-wrong.sh\tselftest\t# 样本：门禁阶段写成自证的种类\n"
    ".claude/gate.d/sample-stage-escape.sh\tsamples:../outside\t# 样本：假工具目录跳出样本根\n"
    "scripts/gate/sample-samples-misplaced.sh\tsamples\t# 样本：样本档的种类写在研究脚本上\n"
    "scripts/experiment-apparatus/sample_device/sample_orchestrator.py\tselftest\t# 样本：实验装置那一层的编排脚本，自证用假驱动\n"
    "scripts/experiment-apparatus/sample_device/deeper/sample_too_deep.py\tselftest\t# 样本：装置目录下面再深一层\n"
    "scripts/sample_top.py\tselftest\t# 样本：直接放在 scripts/ 下、不在哪个分类目录里\n"
    "task-server/sample_server_lib.py\tselftest\t# 样本：统一任务服务那一层的脚本，自证用假驱动\n"
    "prune-server/sample_control\tselftest\t# 样本：判定库服务那一层的脚本，自证只起假的 ip、ssh、systemctl\n"
    ".claude/gate.d/lib-sample-library.py\tselftest\t# 样本：门禁目录顶层以 lib- 起头的共用库，自证用假驱动\n"
    ".claude/gate.d/lib/sample-library.py\tselftest\t# 样本：门禁共用库目录里的库，直接执行就是跑自证\n")
# 样本表里该报出来的行（键）：种类认不出、理由太短、不在登记目录里、重复、文件不在
SAMPLE_SELFTEST_REGISTRY_PROBLEM_KEYS = ["scripts/gate/sample-bad-kind.sh", "scripts/gate/sample-short-reason.sh", "elsewhere/sample-outside.sh",
                                         "scripts/gate/sample-selftest.sh", "scripts/gate/sample-missing.sh", ".claude/gate.d/sample-stage-wrong.sh",
                                         ".claude/gate.d/sample-stage-escape.sh", "scripts/gate/sample-samples-misplaced.sh",
                                         "scripts/experiment-apparatus/sample_device/deeper/sample_too_deep.py", "scripts/sample_top.py"]
# 样本登记表里实验装置那一层的那个装置目录（文件头「实验装置那一层」）
SAMPLE_DEVICE_LAYER = "scripts/experiment-apparatus/sample_device/"


@contextlib.contextmanager
def selftest_registry_swapped(registry, root):
    """只给自证用（平常跑的判法不经它，读的是这份文件旁边那份表）：这一段里登记表换成 registry、键相对 root 认，出来换回。"""
    global SELFTEST_REGISTRY_PATH, SELFTEST_REGISTRY_ROOT
    saved = SELFTEST_REGISTRY_PATH, SELFTEST_REGISTRY_ROOT
    SELFTEST_REGISTRY_PATH, SELFTEST_REGISTRY_ROOT = registry, root
    try:
        yield
    finally:
        SELFTEST_REGISTRY_PATH, SELFTEST_REGISTRY_ROOT = saved


def temporary_service_cells(work, results):
    """要临时服务的自证登记表的读法与裸 --selftest 的判法（文件头 temporary_service_bare_selftest）：在 work 里写样本的自证登记表、要临时服务的表与它们点名的脚本，
    把 SELFTEST_REGISTRY_PATH、SELFTEST_REGISTRY_ROOT、TEMPORARY_SERVICE_TABLE_PATH 换过去判几条命令，判完换回；再核仓里那张表与自证登记表、脚本源码对得上。
    格记进 results，交回判了几格。弄坏开关红的格写在文件头开关表。"""
    global SELFTEST_REGISTRY_PATH, SELFTEST_REGISTRY_ROOT, TEMPORARY_SERVICE_TABLE_PATH
    saved = SELFTEST_REGISTRY_PATH, SELFTEST_REGISTRY_ROOT, TEMPORARY_SERVICE_TABLE_PATH
    sample_registry = os.path.join(work, "sample-temporary-service-registry.tsv")
    sample_table = os.path.join(work, "sample-temporary-service-table.tsv")
    marked = f"# 样本：开跑先调 {TEMPORARY_SERVICE_SOURCE_MARK}\n"
    for relative, text in (("scripts/gate/sample-needs-service.sh", marked), ("scripts/gate/sample-own-server.py", marked), ("scripts/gate/sample-short-reason.sh", marked),
                           ("scripts/gate/sample-marked-unlisted.py", marked), ("scripts/gate/sample-listed-unmarked.sh", "# 样本：没有核对\n"),
                           ("scripts/gate/sample-not-registered.sh", marked), ("scripts/gate/sample-plain.sh", "# 样本：不要临时服务\n"),
                           ("elsewhere-tree/scripts/gate/sample-needs-service.sh", marked)):
        os.makedirs(os.path.dirname(os.path.join(work, relative)), exist_ok=True)
        with open(os.path.join(work, relative), "w", encoding="utf-8") as handle:
            handle.write(text)
    with open(sample_registry, "w", encoding="utf-8") as handle:
        handle.write("".join(f"scripts/gate/{name}\tselftest\t# 样本：自证用假的驱动\n" for name in (
            "sample-needs-service.sh", "sample-own-server.py", "sample-short-reason.sh", "sample-marked-unlisted.py", "sample-listed-unmarked.sh", "sample-plain.sh")))
    with open(sample_table, "w", encoding="utf-8") as handle:
        handle.write("# 样本：要临时服务的自证登记表\n"
                     "scripts/gate/sample-needs-service.sh\trequired\t# 样本：开跑先核临时服务，裸跑退 2\n"
                     "scripts/gate/sample-own-server.py\town-server\t# 样本：自己起服务，裸跑不退 2\n"
                     "scripts/gate/sample-short-reason.sh\trequired\t# 短\n"
                     "scripts/gate/sample-bad-judgement.sh\tmaybe\t# 样本：判定写坏的一行\n"
                     "scripts/gate/sample-needs-service.sh\trequired\t# 样本：键重复的一行\n"
                     "scripts/gate/sample-listed-unmarked.sh\trequired\t# 样本：登记了 required 而源码里没有核对\n"
                     "scripts/gate/sample-not-registered.sh\trequired\t# 样本：自证登记表里没有这一份\n"
                     "scripts/gate/sample-missing.sh\trequired\t# 样本：文件不在\n")
    before = len(results)
    needs_service = "scripts/gate/sample-needs-service.sh"
    wrapped = f"{TEMPORARY_SERVICE_WRAPPER_MODE} -- bash {needs_service} --selftest"
    try:
        SELFTEST_REGISTRY_PATH, SELFTEST_REGISTRY_ROOT, TEMPORARY_SERVICE_TABLE_PATH = sample_registry, work, sample_table
        bare = temporary_service_bare_selftest([needs_service, "--selftest"], work, {})
        results.append(("要临时服务：required 行的裸 --selftest 交整条包装写法", (True, True),
                        (bare is not None, bare is not None and f"task-server/task submit -- python3 {TEMPORARY_SERVICE_WRAPPER_DIRECTORY}/{TEMPORARY_SERVICE_WRAPPER_SCRIPT} {wrapped}" in bare)))
        results.append(("要临时服务：命令前设了临时服务的标记，裸 --selftest 放行", None,
                        temporary_service_bare_selftest([needs_service, "--selftest"], work, {TEMPORARY_SERVICE_VARIABLE: "/sample/server.sock"})))
        marked_environment = environment_under_temporary_service({"PATH": "/sample"})
        results.append(("要临时服务：包装剥开重判时补上的标记让里面那条放行", (True, None),
                        (marked_environment.get(TEMPORARY_SERVICE_VARIABLE, "") != "" and marked_environment.get("PATH") == "/sample",
                         temporary_service_bare_selftest([needs_service, "--selftest"], work, marked_environment))))
        results.append(("要临时服务：own-server 行的裸 --selftest 不拒", None, temporary_service_bare_selftest(["scripts/gate/sample-own-server.py", "--selftest"], work, {})))
        results.append(("要临时服务：带了 --selftest 之外参数的不拒", None, temporary_service_bare_selftest([needs_service, "--selftest", "--cells", "x"], work, {})))
        results.append(("要临时服务：表里没有的脚本不拒", None, temporary_service_bare_selftest(["scripts/gate/sample-plain.sh", "--selftest"], work, {})))
        results.append(("要临时服务：别的树里路径尾部相同的不拒", None,
                        temporary_service_bare_selftest([os.path.join(work, "elsewhere-tree", needs_service), "--selftest"], work, {})))
        results.append(("要临时服务：理由太短的一行不算登记", None, temporary_service_bare_selftest(["scripts/gate/sample-short-reason.sh", "--selftest"], work, {})))
        found_keys = sorted({match.group(1) for problem in temporary_service_table_problems(sample_table, sample_registry, work)
                             for match in [re.match(r"^(?:第 \d+ 行 )?(\S+)：", problem)] if match})
        results.append(("要临时服务：样本表写坏的、两边对不上的几行逐条报出来（判定、理由、键重复、文件不在、没登记、源码里没有核对、漏登）",
                        sorted(f"scripts/gate/sample-{name}" for name in ("bad-judgement.sh", "listed-unmarked.sh", "marked-unlisted.py", "missing.sh", "needs-service.sh",
                                                                          "not-registered.sh", "short-reason.sh")), found_keys))
        results.append(("要临时服务：python 脚本的整条写法起头是 python3", f"python3 {TEMPORARY_SERVICE_WRAPPER_DIRECTORY}/{TEMPORARY_SERVICE_WRAPPER_SCRIPT} {TEMPORARY_SERVICE_WRAPPER_MODE} -- python3 scripts/gate/sample-own-server.py --selftest",
                        temporary_service_command("scripts/gate/sample-own-server.py")))
        whole_command = temporary_service_command(needs_service)
        results.append(("要临时服务：整条写法不带 --server-binary 与尖括号占位", (False, False),
                        (TEMPORARY_SERVICE_BINARY_OPTION in whole_command, "<" in whole_command or ">" in whole_command)))
        refusal_text = temporary_service_bare_selftest([needs_service, "--selftest"], work, {}) or ""
        results.append(("要临时服务：裸 --selftest 的拒绝说明里没有尖括号占位", False, "<" in refusal_text or ">" in refusal_text))
    finally:
        SELFTEST_REGISTRY_PATH, SELFTEST_REGISTRY_ROOT, TEMPORARY_SERVICE_TABLE_PATH = saved
    results.append((f"要临时服务：仓里的表（{TEMPORARY_SERVICE_TABLE_PATH}）与自证登记表、脚本源码对得上", [],
                    temporary_service_table_problems(TEMPORARY_SERVICE_TABLE_PATH, SELFTEST_REGISTRY_PATH, HOOK_REPOSITORY)))
    incident = "scripts/mutation/crates-mutation-rows.py"
    real_bare = temporary_service_bare_selftest([incident, "--selftest"], HOOK_REPOSITORY, {})
    results.append(("要临时服务：仓里 crates-mutation-rows.py 的裸 --selftest 被点名、给整条包装写法", (True, True),
                    (real_bare is not None, real_bare is not None
                     and f"{TEMPORARY_SERVICE_WRAPPER_MODE} -- python3 {incident} --selftest" in real_bare)))
    results.append(("要临时服务：仓里 crates-mutation-rows.py 的 --selftest 在包装底下放行", None,
                    temporary_service_bare_selftest([incident, "--selftest"], HOOK_REPOSITORY, environment_under_temporary_service({}))))
    return len(results) - before


def argument_reading_cells(results):
    """阶段参数里 --check 点名的格（checked_cells）与 libtest 参数里的 --list（libtest_lists_only）怎么读：lib_run_scope.py 的范围声明闸靠这两个判法数格数、认「只列不跑」。
    格记进 results，交判了几格。弄坏开关 checked-cells-last-only、libtest-list-by-presence 红的格写在文件头开关表。"""
    before = len(results)
    for label, arguments, want in (
            ("阶段参数 --check：--check a,b 点名两格", ["--check", "a,b"], {"a", "b"}),
            ("阶段参数 --check：--check=a 的写法", ["--check=a"], {"a"}),
            ("阶段参数 --check：--check=a 与 --check b 给两次取并集", ["--check=a", "--check", "b"], {"a", "b"}),
            ("阶段参数 --check：--check 给三次（含 --check= 写法）取并集", ["--check", "a", "--check", "b,c", "--check=d"], {"a", "b", "c", "d"}),
            ("阶段参数 --check：逗号之间的空段不算一格", ["--check", "a,,b,"], {"a", "b"}),
            ("阶段参数 --check：没给 --check 交 None（全跑）", ["--item", "x", "/repo"], None),
            ("阶段参数 --check：--check 写在参数表末尾、后面缺值，交 None", ["/repo", "--check"], None),
            ("阶段参数 --check：--check 给了空串，给了但一格都没点名，交空集", ["--check", ""], set())):
        results.append((label, want, checked_cells(arguments)))
    for label, arguments, want in (
            ("libtest 参数：--list 单独一个是只列", ["--list"], True),
            ("libtest 参数：--ignored --test-threads 4 --list 是只列", ["--ignored", "--test-threads", "4", "--list"], True),
            ("libtest 参数：--skip --list（--list 是 --skip 的值）不是只列", ["--include-ignored", "--skip", "--list"], False),
            ("libtest 参数：--logfile --list（--list 是日志文件名）不是只列", ["--logfile", "--list"], False),
            ("libtest 参数：-- 之后的 --list 是过滤词，不是只列", ["--", "--list"], False),
            ("libtest 参数：-- 隔开两段之后的 --list 是过滤词，不是只列", ["--include-ignored", "the_full_case", "--", "--list"], False),
            ("libtest 参数：--skip=--list 是一个词、没有 --list 选项，不是只列", ["--ignored", "--skip=--list"], False),
            ("libtest 参数：没有 --list 不是只列", ["--include-ignored", "the_full_case"], False),
            ("libtest 参数：一个参数都没有不是只列", [], False)):
        results.append((label, want, libtest_lists_only(arguments)))
    return len(results) - before


def selftest_registry_cells(work, results):
    """自证登记表的读法：在 work 里写样本表与它点名的文件（sample-missing.sh 不写），把 SELFTEST_REGISTRY_PATH 换过去判几条命令，判完换回；
    再核仓里那份表（这份文件旁边）没有写坏的行、指到的文件在这份文件所在的仓里都在。格记进 results，交回判了几格。"""
    global SELFTEST_REGISTRY_PATH, SELFTEST_REGISTRY_ROOT
    real_registry, real_root = SELFTEST_REGISTRY_PATH, SELFTEST_REGISTRY_ROOT
    sample_registry = os.path.join(work, "sample-selftest-registry.tsv")
    with open(sample_registry, "w", encoding="utf-8") as handle:
        handle.write(SAMPLE_SELFTEST_REGISTRY)
    for relative in ("scripts/gate/sample-selftest.sh", "scripts/gate/sample-bad-kind.sh", "scripts/gate/sample-short-reason.sh", "scripts/gate/sample-pat*.sh", "elsewhere/sample-outside.sh",
                     ".claude/gate.d/sample-stage.sh", ".claude/gate.d/sample-stage-plain.sh", ".claude/gate.d/lib/sample-library.py", "elsewhere-tree/scripts/gate/sample-selftest.sh",
                     *(SAMPLE_DEVICE_LAYER + name for name in ("sample_orchestrator.py", "deeper/sample_too_deep.py", "sample_unregistered.py")),
                     "scripts/sample_top.py", ".claude/gate.d/lib-sample-library.py", ".claude/gate.d/lib-sample-unregistered.py", "task-server/sample_server_lib.py",
                     "prune-server/sample_control", f"{TEMPORARY_SERVICE_WRAPPER_DIRECTORY}/{TEMPORARY_SERVICE_WRAPPER_SCRIPT}",
                     "elsewhere-tree/" + SAMPLE_DEVICE_LAYER + "sample_orchestrator.py"):
        os.makedirs(os.path.dirname(os.path.join(work, relative)), exist_ok=True)
        with open(os.path.join(work, relative), "w", encoding="utf-8") as handle:
            handle.write("# 样本\n")
    before = len(results)
    # (说明, 命令词, 参数, 该交的行的种类；None 是不当登记过的自证)
    sample_cases = [
        ("样本表：登记过的单独 --selftest 认", "scripts/gate/sample-selftest.sh", ["--selftest"], "selftest"),
        ("样本表：带了 --selftest 之外参数的不认", "scripts/gate/sample-selftest.sh", ["--selftest", "--cells", "x"], None),
        ("样本表：不带 --selftest 的不认", "scripts/gate/sample-selftest.sh", [], None),
        ("样本表：命令词带通配的不当登记过的自证", "scripts/gate/sample-pat*.sh", ["--selftest"], None),
        ("样本表：种类写坏的一行不算登记", "scripts/gate/sample-bad-kind.sh", ["--selftest"], None),
        ("样本表：理由太短的一行不算登记", "scripts/gate/sample-short-reason.sh", ["--selftest"], None),
        ("样本表：别的树里路径尾部相同的脚本不算登记", os.path.join(work, "elsewhere-tree", "scripts", "gate", "sample-selftest.sh"), ["--selftest"], None),
        ("样本表：门禁目录顶层 lib- 起头的共用库登记过，单独 --selftest 认", ".claude/gate.d/lib-sample-library.py", ["--selftest"], "selftest"),
        ("样本表：门禁目录顶层 lib- 起头的共用库不带参数不认，只认单独一个 --selftest", ".claude/gate.d/lib-sample-library.py", [], None),
        ("样本表：门禁共用库目录里的库登记过，不带参数直接执行也认", ".claude/gate.d/lib/sample-library.py", [], "selftest"),
        ("样本表：门禁共用库目录里的库登记过，单独 --selftest 认", ".claude/gate.d/lib/sample-library.py", ["--selftest"], "selftest"),
        ("样本表：门禁共用库目录里的库带 --selftest 之外的参数不认", ".claude/gate.d/lib/sample-library.py", ["show", "HEAD", "x.tsv"], None),
        ("样本表：门禁目录顶层 lib- 起头的共用库没登记的不认", ".claude/gate.d/lib-sample-unregistered.py", ["--selftest"], None),
        ("样本表：门禁目录顶层 lib- 起头的共用库带 --selftest 之外的参数不认", ".claude/gate.d/lib-sample-library.py", ["show", "HEAD", "x.tsv"], None),
        ("样本表：装置那一层登记过的单独 --selftest 认", SAMPLE_DEVICE_LAYER + "sample_orchestrator.py", ["--selftest"], "selftest"),
        ("样本表：装置那一层没登记的 --selftest 不认", SAMPLE_DEVICE_LAYER + "sample_unregistered.py", ["--selftest"], None),
        ("样本表：装置那一层登记了而带 --cells 的不认", SAMPLE_DEVICE_LAYER + "sample_orchestrator.py", ["--selftest", "--cells", "x"], None),
        ("样本表：别的树里装置那一层路径尾部相同的不认", os.path.join(work, "elsewhere-tree", SAMPLE_DEVICE_LAYER + "sample_orchestrator.py"), ["--selftest"], None),
        # 统一任务服务那一层（task-server/）：登记了的单独 --selftest 认；`task-server/task submit -- …` 里那条带的 --selftest 不当客户端自己的自证
        # （LIB_PROJECT_CODE_RUNS_BREAK=task-server-directory-unrecognized 下头一格红，task-server-submit-as-selftest 与 task-server-submission-unparsed 下第二格红）
        ("样本表：task-server 那一层登记过的单独 --selftest 认", "task-server/sample_server_lib.py", ["--selftest"], "selftest"),
        ("样本表：task-server/task submit -- 里那条带 --selftest，不当客户端自己的自证", TASK_SERVER_CLIENT, ["submit", "--", "python3", "task-server/sample_server_lib.py", "--selftest"], None),
        # 判定库服务那一层（prune-server/）：同上（LIB_PROJECT_CODE_RUNS_BREAK=prune-server-directory-unrecognized 下这一格红）
        ("样本表：prune-server 那一层登记过的单独 --selftest 认", "prune-server/sample_control", ["--selftest"], "selftest"),
        ("样本表：包装 --server-binary -- 里那条带 --selftest，不当包装自己的自证", f"{TEMPORARY_SERVICE_WRAPPER_DIRECTORY}/{TEMPORARY_SERVICE_WRAPPER_SCRIPT}",
         [TEMPORARY_SERVICE_WRAPPER_MODE, TEMPORARY_SERVICE_BINARY_OPTION, "/sample/prune-server", "--", "bash", "scripts/gate/sample-selftest.sh", "--selftest"], None),
    ]
    try:
        SELFTEST_REGISTRY_PATH, SELFTEST_REGISTRY_ROOT = sample_registry, work
        for label, word, arguments, want in sample_cases:
            row = registered_selftest(word, arguments, work)
            results.append((label, want, row.kind if row else None))
        found_keys = sorted(match.group(1) for problem in selftest_registry_problems(sample_registry, work)
                            for match in [re.match(r"^第 \d+ 行 (\S+)：", problem)] if match)
        results.append(("样本登记表里写坏的几行逐条报出来（种类、理由、目录、重复、文件不在、门禁阶段那几行的种类与假工具目录）", sorted(SAMPLE_SELFTEST_REGISTRY_PROBLEM_KEYS), found_keys))
        rows = read_selftest_registry(sample_registry)[0]
        stage_row, plain_row = rows.get(".claude/gate.d/sample-stage.sh"), rows.get(".claude/gate.d/sample-stage-plain.sh")
        results.append(("样本表：门禁阶段的样本档那一行带假工具目录照收", ("samples", ".sample-tools"), (stage_row.kind, stage_row.tools_directory) if stage_row else None))
        results.append(("样本表：门禁阶段的样本档那一行不带假工具目录照收", ("samples", ""), (plain_row.kind, plain_row.tools_directory) if plain_row else None))
        SELFTEST_REGISTRY_PATH = os.path.join(work, "no-such-registry.tsv")
        results.append(("登记表读不了：一律按没登记", None, registered_selftest("scripts/gate/sample-selftest.sh", ["--selftest"], work)))
    finally:
        SELFTEST_REGISTRY_PATH, SELFTEST_REGISTRY_ROOT = real_registry, real_root
    temporary_service_cells(work, results)
    results.append((f"仓里的登记表（{real_registry}）没有写坏的行、指到的文件在 {HOOK_REPOSITORY} 里都在", [],
                    selftest_registry_problems(real_registry, HOOK_REPOSITORY)))
    return len(results) - before


def selftest():
    """在临时工作区里判外部程序名单、包在命令外面的启动器、花括号与通配展开、本仓代码与编译的认法、认不出起什么的写法、登记表与门禁样本、经 task-server 起的几样；返回退出码。"""
    global SAMPLE_INDEX_REPOSITORY, BASH_FOR_PATTERNS, FAMILY_OWN_REPOSITORY
    work = tempfile.mkdtemp(prefix="lib-project-code-runs-")
    results = []
    real_sample_index = SAMPLE_INDEX_REPOSITORY
    # 登记表那几格判的是本仓里那几份脚本：当前目录写本仓（登记表的键按解析后的真实路径认，临时工作区里同名的不算）
    repository = HOOK_REPOSITORY
    try:
        build_sample_workspace(work)
        FAMILY_OWN_REPOSITORY = work
        SAMPLE_INDEX_REPOSITORY = os.path.join(work, SAMPLE_INDEX_REFERENCE_NAME)
        harness = os.path.join(work, "crates", "singlefs-verification")
        alias_to_harness = "CARGO_ALIAS_XT=test -p singlefs-verification"
        inner = ["herd7", "x.litmus"]

        def uses_in_process(argv, cwd):
            """一个在跑的进程（argv 与 cwd）：包装（/usr/bin/time、flock、systemd-run 这一类，gpu-quiesce-run.sh）剥掉之后里面认出的外部程序用法。"""
            launched = command_under_launcher(argv, cwd) or command_under_gpu_quiesce(argv, cwd)
            if launched is not None:
                inner_words, inner_directory = launched
                return uses_in_process(inner_words, shell_words.resolve_path(cwd, inner_directory) if inner_directory else cwd)
            return program_uses_in_command_text(shlex.join(argv), cwd)

        # 外部程序名单（文件头「外部程序名单」）：命中的算 external-program，只问版本的 herd7 放行；本仓的代码、会编的 cargo 由别的判法认，这里交 None。
        # 弄坏开关 external-programs-unlisted 下除「不在名单」的几格之外全红（应当是 external-program，实际 None）；herd7-version-flag-counted 下「只问版本」的两格红
        external_cases = [
            ("外部程序：qemu-system-x86_64", ["qemu-system-x86_64", "-nographic"], work, "external-program"),
            ("外部程序：qemu-system-aarch64 写绝对路径", ["/usr/bin/qemu-system-aarch64", "-M", "virt"], work, "external-program"),
            ("外部程序：herd7 跑 litmus", ["herd7", "x.litmus"], work, "external-program"),
            ("外部程序：herd7 带 -conf 选项", ["herd7", "-conf", "linux-kernel.cfg", "x.litmus"], work, "external-program"),
            ("外部程序：e152-file-system-benchmark 光名字从 PATH 起", ["e152-file-system-benchmark", "--device", "x"], work, "external-program"),
            ("外部程序：herd7 -version 只问版本，放行", ["herd7", "-version"], work, None),
            ("外部程序：herd7 --version 只问版本，放行", ["herd7", "--version"], work, None),
            ("外部程序：herd7 -version 之后还带别的参数不算只问版本", ["herd7", "-version", "x.litmus"], work, "external-program"),
            ("不在名单：qemu-img 查镜像信息", ["qemu-img", "info", "disk.img"], work, None),
            ("不在名单：名字只是以 herd7 起头的别的程序", ["herd7-helper", "x"], work, None),
            ("不在名单：cargo test 由 starts_a_build 与 runs_compiled_code 认", ["cargo", "test", "-p", "singlefs-core"], work, None),
            ("不在名单：echo 字面写 herd7", ["echo", "herd7", "x.litmus"], work, None),
        ]
        for label, argv, cwd, want in external_cases:
            found = classify(argv, cwd)
            results.append((label, want, found.kind if found else None))
        # lib_shell_words 前缀表之外、包在命令外面照样起那条命令的程序（command_under_launcher）：里面那条外部程序照认
        launcher_cases = [
            ("/usr/bin/time -v 包一层", ["/usr/bin/time", "-v", *inner], work, "external-program"),
            ("time -f 格式 -o 文件包一层", ["time", "-f", "%M", "-o", "/tmp/t", *inner], work, "external-program"),
            ("systemd-run --working-directory= 包一层", ["systemd-run", "--user", "--scope", "--working-directory=crates/singlefs-verification", *inner], work, "external-program"),
            ("flock -w 秒数 锁文件 包一层", ["flock", "-w", "5", "/tmp/lock", *inner], work, "external-program"),
            ("flock 锁文件 -c 字符串", ["flock", "/tmp/lock", "-c", "herd7 x.litmus"], work, "external-program"),
            ("rustup run --install stable 包一层", ["rustup", "run", "--install", "stable", *inner], work, "external-program"),
            ("chrt -i 0 包一层", ["chrt", "-i", "0", *inner], work, "external-program"),
            ("prlimit --as= 包一层", ["prlimit", "--as=8000000000", *inner], work, "external-program"),
            ("systemd-run --user --scope -p MemoryMax=8G 包一层", ["systemd-run", "--user", "--scope", "-q", "-p", "MemoryMax=8G", *inner], work, "external-program"),
            ("strace -f -o 文件包一层", ["strace", "-f", "-o", "/tmp/trace", *inner], work, "external-program"),
            ("perf stat -e 事件包一层", ["perf", "stat", "-e", "cycles", *inner], work, "external-program"),
            ("perf record -g -- 之后", ["perf", "record", "-g", "--", *inner], work, "external-program"),
            ("perf mem record 包一层", ["perf", "mem", "record", *inner], work, "external-program"),
            ("nice 里 /usr/bin/time 里 flock 一层套一层", ["nice", "-n", "19", "/usr/bin/time", "-v", "flock", "/tmp/l", *inner], work, "external-program"),
            ("strace -p 进程号：没有要起的命令", ["strace", "-p", "1234"], work, None),
            ("chrt -p 查优先级：没有要起的命令", ["chrt", "-p", "1234"], work, None),
            ("rustup show 不是 rustup run", ["rustup", "show"], work, None),
            ("perf report 不起命令", ["perf", "report", "-i", "perf.data"], work, None),
            ("/usr/bin/time 包一个不在名单的命令", ["/usr/bin/time", "-v", "true"], work, None),
            # 短选项合写：逐个字母查表，头一个要值的字母之后剩下的是值、没剩下取下一个词（开关 launcher-whole-word-options 下前四格红）
            ("strace -fo 文件（短选项合写）", ["strace", "-fo", "/tmp/s", *inner], work, "external-program"),
            ("flock -xw 10 锁文件（短选项合写）", ["flock", "-xw", "10", "/tmp/l", *inner], work, "external-program"),
            ("systemd-run --user --scope -qu 名字（短选项合写）", ["systemd-run", "--user", "--scope", "-qu", "k1", *inner], work, "external-program"),
            ("/usr/bin/time -ao 文件（短选项合写）", ["/usr/bin/time", "-ao", "/tmp/t", *inner], work, "external-program"),
            ("strace -o文件（值贴着写）", ["strace", "-o/tmp/s", *inner], work, "external-program"),
            # 要值的长选项、值另起一个词：跳过那个值（开关 launcher-long-options-partial 下前四格红）
            ("systemd-run --user --scope --expand-environment no（值另起一个词）包一层", ["systemd-run", "--user", "--scope", "--expand-environment", "no", *inner], work, "external-program"),
            ("strace --output 文件（值另起一个词）包一层", ["strace", "--output", "/tmp/s", *inner], work, "external-program"),
            ("strace --string-limit 200 包一层", ["strace", "--string-limit", "200", *inner], work, "external-program"),
            ("strace -c --summary-col calls（长选项只写前缀）包一层", ["strace", "-c", "--summary-col", "calls", *inner], work, "external-program"),
            ("strace --quiet（不带值的长选项）包一层", ["strace", "--quiet", *inner], work, "external-program"),
            ("systemd-run --user --scope --collect --same-dir（不带值的长选项）包一层", ["systemd-run", "--user", "--scope", "--collect", "--same-dir", *inner], work, "external-program"),
            ("two-logs.sh 日志出两份的包装", ["scripts/hosts/two-logs.sh", "run-one", "--", *inner], work, "external-program"),
            ("two-logs.sh --quiet 日志出两份的包装", ["scripts/hosts/two-logs.sh", "--quiet", "run-one", "--", *inner], work, "external-program"),
            ("run-build-directory.sh 这一趟自己的编译目录的包装", ["scripts/hosts/run-build-directory.sh", "--", *inner], work, "external-program"),
            ("gpu-quiesce-run.sh 清场之后起命令的包装", ["scripts/crash/gpu-quiesce-run.sh", "--", *inner], work, "external-program"),
            ("systemd-run -E 设的不是别名（值的末段叫 strace 也不当包装剥）", ["systemd-run", "--user", "--scope", "-E", "TRACER=/usr/bin/strace", *inner], work, "external-program"),
        ]
        for label, argv, cwd, want in launcher_cases:
            found = uses_in_process(argv, cwd)
            results.append((label, want, found[0].kind if found else None))
        # 启动器设的环境变量带进里面那条命令（交回那条命令的最前面写 名=值，再切一遍时当它的环境变量）：
        # systemd-run -E / --setenv= / -qE 与 -p Environment= / EnvironmentFile=（开关 launcher-drops-setenv、launcher-drops-property-environment 下「True」的格红），
        # strace -E / --env（开关 strace-drops-env 下「True」的格红）
        environment_cases = [
            ("systemd-run -E 设别名", ["systemd-run", "--user", "--scope", "-E", alias_to_harness, "cargo", "xt"], True),
            ("systemd-run --setenv= 设别名", ["systemd-run", "--user", "--scope", "--setenv=" + alias_to_harness, "cargo", "xt"], True),
            ("systemd-run -qE 合写设别名", ["systemd-run", "--user", "--scope", "-qE", alias_to_harness, "cargo", "xt"], True),
            ("strace -E 设别名", ["strace", "-f", "-E", alias_to_harness, "cargo", "xt"], True),
            ("strace --env= 设别名", ["strace", "--env=" + alias_to_harness, "cargo", "xt"], True),
            ("strace --env 值另起一个词设别名", ["strace", "--env", alias_to_harness, "cargo", "xt"], True),
            ("strace -fE 合写设别名", ["strace", "-fE", alias_to_harness, "cargo", "xt"], True),
            ("strace -E 只清变量（不带 =）", ["strace", "-E", "CARGO_ALIAS_XT", "cargo", "xt"], False),
            ("systemd-run -p Environment= 设别名", ["systemd-run", "--user", "--wait", "--pipe", "-p", 'Environment="' + alias_to_harness + '"', "cargo", "xt"], True),
            ("systemd-run --property=Environment= 设两个变量、第二个是别名",
             ["systemd-run", "--user", "--wait", "--pipe", '--property=Environment=A=1 "' + alias_to_harness + '"', "cargo", "xt"], True),
            ("systemd-run --prop Environment=（长选项只写前缀）设别名", ["systemd-run", "--user", "--wait", "--pipe", "--prop", 'Environment="' + alias_to_harness + '"', "cargo", "xt"], True),
            ("systemd-run -p EnvironmentFile= 文件里设别名", ["systemd-run", "--user", "--wait", "--pipe", "-p", "EnvironmentFile=" + os.path.join(work, "alias.env"), "cargo", "xt"], True),
            ("systemd-run -p Environment= 设的不是别名", ["systemd-run", "--user", "--wait", "--pipe", "-p", "Environment=RUST_BACKTRACE=1", "cargo", "xt"], False),
            ("systemd-run -p EnvironmentFile= 文件里设的不是别名", ["systemd-run", "--user", "--wait", "--pipe", "-p", "EnvironmentFile=" + os.path.join(work, "plain.env"), "cargo", "xt"], False),
        ]
        for label, argv, want in environment_cases:
            launched = command_under_launcher(argv, work)
            results.append((f"启动器环境：{label}", want, bool(launched) and alias_to_harness in launched[0]))
        # 花括号起法照命令文本判（lib_shell_words.py 照 bash 展开引号外的花括号、命令词展开成几份时 bash 起的是头一份、其余是它的参数）：
        # 开关 brace-expansion-literal 下「花括号」开头照拒的各格红，brace-sequence-literal 下序列那两格红，brace-command-words-bare 下「花括号写在包装上」一格红，
        # quoted-braces-expanded 下引号里的两格红（应当是 None，实际 external-program）
        os.makedirs(os.path.join(work, "tools"), exist_ok=True)
        for tool in ("herd7", "herd7-helper", "nice"):
            with open(os.path.join(work, "tools", tool), "w", encoding="utf-8") as handle:
                handle.write("#!/bin/sh\n")
        for label, argv, want in (
                ("花括号：命令词里的花括号展开（herd{7,x} 起的是 herd7）", ["bash", "-c", "herd{7,x} x.litmus"], "external-program"),
                ("花括号：命令词整个写成花括号（{herd7,x.litmus}）", ["bash", "-c", "{herd7,x.litmus}"], "external-program"),
                ("花括号：序列展开（qemu-system-{x86_64,aarch64}）", ["bash", "-c", "qemu-system-{x86_64,aarch64} -nographic"], "external-program"),
                ("花括号：花括号写在包装上（{nice,-n,5} herd7 x）", ["bash", "-c", "{nice,-n,5} herd7 x.litmus"], "external-program"),
                ("花括号：命令词位置单引号 {herd7,x.litmus} 照字面放行", ["bash", "-c", "'{herd7,x.litmus}'"], None),
                ("花括号：命令词位置双引号 {herd7,x.litmus} 照字面放行", ["bash", "-c", '"{herd7,x.litmus}"'], None),
                ("花括号：参数里的花括号展开不影响放行（echo herd{7,x}）", ["bash", "-c", "echo herd{7,x}"], None)):
            found = uses_in_process(argv, work)
            results.append((label, want, found[0].kind if found else None))
        # 展开出来的本仓的代码、编译（project_code_program_use；开关 expanded-project-code-unjudged 下「展开」开头照拒的各格红）
        for label, argv, want in (
                ("展开：通配起本仓的脚本经 capped.sh（scripts/hosts/capped.s? 4 true）", ["bash", "-c", "bash scripts/hosts/capped.s? 4 true"], "project-code"),
                ("展开：花括号拼出的是只读命令不拒（{ls,-l}）", ["bash", "-c", "{ls,-l}"], None),
                ("展开：通配起文本工具 scripts/kb/replace-onc?.py 不拒（谁都能直接跑）", ["bash", "-c", "python3 scripts/kb/replace-onc?.py --selftest-never"], None)):
            found = uses_in_process(argv, repository)
            results.append((label, want, found[0].kind if found else None))
        # 没展开的词直接交 classify（命令文本里引号外的花括号 lib_shell_words.py 先展开成头一份当命令，其余是参数，到不了 classify）：
        # 命令词展开成几份的，每一份单独当命令判。开关 brace-sequence-literal 下头一格红（应当是 external-program，实际 None），expanded-project-code-unjudged 下第二格红
        for label, words, cwd, want in (
                ("花括号：数字序列展开（herd{1..9} 里有 herd7）", ["herd{1..9}", "x.litmus"], work, "external-program"),
                ("展开：花括号拼出 cargo test（{cargo,test,-p,singlefs-core}）", ["{cargo,test,-p,singlefs-core}"], repository, "project-code")):
            found = classify(words, cwd)
            results.append((label, want, found.kind if found else None))
        # 命令词带通配：按 directory 展开、按展开出来的那几份判（开关 pattern-command-words-literal 下「通配」开头照拒的各格红）
        for label, words, cwd, want in (
                ("通配：tools/her?7 展开成 tools/herd7", ["tools/her?7", "x.litmus"], work, "external-program"),
                ("通配：tools/herd* 展开成 herd7 与 herd7-helper，其中有 herd7", ["tools/herd*", "x.litmus"], work, "external-program"),
                ("通配：零命中按字面判", ["tools/no-such-*", "x.litmus"], work, None),
                ("通配：展开成不在名单的程序", ["tools/herd7-h*", "x"], work, None),
                ("通配：tools/herd[^x] 的 [^…] 照 bash 当取反", ["tools/herd[^x]", "x.litmus"], work, "external-program"),
                ("通配：POSIX 字符类 tools/her[[:alpha:]]7 交给 bash 展开", ["tools/her[[:alpha:]]7", "x.litmus"], work, "external-program"),
                ("通配：展开出来的是 nice 包装，包装里的 herd7 照认", ["tools/ni?e", "-n", "5", "herd7", "x.litmus"], work, "external-program")):
            found = classify(words, cwd)
            results.append((label, want, found.kind if found else None))
        # here-string 与管道喂给从标准输入读命令的 shell 的字面文本当命令判（lib_shell_words.py 的 shell_standard_input；
        # LIB_SHELL_WORDS_BREAK=shell-standard-input-unread 下前三格红：应当是 external-program，实际 None）
        standard_input_cases = [
            ("标准输入：here-string 喂给 bash 跑 herd7", "bash <<< 'herd7 x.litmus'", "external-program"),
            ("标准输入：printf 经管道喂给 bash 跑 herd7", "printf 'herd7 x.litmus\\n' | bash", "external-program"),
            ("标准输入：echo 经管道喂给 sh 跑 herd7", "echo 'herd7 x.litmus' | sh", "external-program"),
            ("标准输入：printf 经管道喂给 bash -c（代码不从标准输入读）不算", "printf 'herd7 x.litmus' | bash -c 'cat'", None),
        ]
        for label, text, want in standard_input_cases:
            found = program_uses_in_command_text(text, work)
            results.append((label, want, found[0].kind if found else None))
        # 命令词写在变量里：前面赋过值的换成值再判（command_word_expanded；开关 command-word-variables-unexpanded 下这一格红）
        for label, text, want in (("命令词变量：S=herd7 之后 $S 起 herd7", "S=herd7; $S x.litmus", "external-program"),
                                  ("命令词变量：没赋过值的 $T 看不出起的是什么，不算外部程序", "$T x.litmus", None)):
            found = program_uses_in_command_text(text, work)
            results.append((label, want, found[0].kind if found else None))
        # bash 起不来时带 POSIX 字符类的命令词照拒（开关 posix-brackets-unresolved-literal 与 posix-brackets-by-python-glob 下这一格红）
        real_bash, BASH_FOR_PATTERNS = BASH_FOR_PATTERNS, os.path.join(work, "no-such-bash")
        try:
            bash_missing_cases = [("bash 起不来时 POSIX 字符类起外部程序照拒", ["bash", "tools/her[[:alpha:]]7", "x.litmus"], repository, "pattern-unresolved")]
            for label, argv, cwd, want in bash_missing_cases:
                found = uses_in_process(argv, cwd)
                results.append((label, want, found[0].kind if found else None))
        finally:
            BASH_FOR_PATTERNS = real_bash
        # 花括号展开与 bash 一致：同一批词交给 bash（set -f 关掉通配，只剩花括号展开）逐个比（开关 brace-expansion-literal 下大半红，brace-sequence-literal 下序列那几格红）
        brace_words = ["{a,b}", "a{b,c}d", "{a{b,c}}", "{a,b{c,d}}e", "{a,b", "{{a,b}", "{a,b}}", "{a}", "{}", "{,}", "x{,}", "{,a}", "herd{7,x}",
                       "{a,{b}}", "a{{b}}", "a{b{c,d}", "{a,b}{", "}{a,b}", "{}a,b}", "..{},aa,}a.", "ab{}1..,}{.{", "{1..3}", "{3..1}", "{01..10}", "{1..10..3}",
                       "{a..e}", "{a..e..2}", "{1..a}", "{-2..2}", "{-01..3}", "{1..-03}", "{0..00}", "{1..05..2}", "{01..100..33}", "{100..00..50}", "{+1..3}", "{1..3..0}",
                       "{1...3}", "{a..}", "x{1..3}y{a,b}", "{1..2}{3..4}", "{a,b}{c,d}{e,f}"]
        script = "set -f\n" + "".join(f"set -- {word}\nprintf '%s\\0' \"$#\" \"$@\"\n" for word in brace_words)
        answer = subprocess.run(["bash"], input=script, capture_output=True, text=True).stdout.split("\0")
        position = 0
        try:
            for word in brace_words:
                count = int(answer[position])
                results.append((f"花括号展开与 bash 一致：{word}", answer[position + 1:position + 1 + count], brace_expansion(word)))
                position += 1 + count
        except (ValueError, IndexError):
            results.append(("花括号展开与 bash 一致：bash 的输出读得出", True, False))
        # 自证登记表的读法：换成样本表（写坏的几种行都有）判；仓里那份表另核没有写坏的行、指到的文件都在
        argument_cases = argument_reading_cells(results)
        registry_cases = selftest_registry_cells(work, results)
        # 门禁样本（文件头「门禁样本」）：stage-selftest.sh 喂的样本要登记在表里、与 SAMPLE_INDEX_REPOSITORY 的 HEAD 加暂存区逐字节相同才盯；只信本仓那一份 stage-selftest.sh。
        # 弄坏开关 stage-samples-fixtures-unchecked 下「样本」开头改过的五格红，stage-items-as-directory 下 --item / --list-items 的格红，stage-items-unjudged 下点名的几道不判的格红，
        # stage-feed-script-by-name 下别处树里拷贝不带 --item 的格红，stage-items-script-by-name 下别处树里拷贝带 --item 的格红，stage-directory-pwd-unexpanded 下 $PWD 的格红
        own_gate_selftest = os.path.join(repository, KNOWN_SCRIPT_LOCATIONS[STAGE_SELFTEST_SCRIPT])
        evil_gate_selftest = os.path.join(work, "evil-tree", KNOWN_SCRIPT_LOCATIONS[STAGE_SELFTEST_SCRIPT])
        fed = lambda name: os.path.join(work, name, ".claude", "gate.d")
        stage_sample_cases = [
            # (说明, 命令词, 参数, 当前目录, 环境变量, 该不该盯)
            ("样本：喂的样本多一份 setup.sh", own_gate_selftest, [fed("gd-sample-setup")], work, None, False),
            ("样本：内容改过", own_gate_selftest, [fed("gd-sample-edited")], work, None, False),
            ("样本：执行位改过", own_gate_selftest, [fed("gd-sample-mode")], work, None, False),
            ("样本：少了文件", own_gate_selftest, [fed("gd-sample-missing")], work, None, False),
            ("样本：暂存之前新加的样本（样本仓自己的阶段目录）", own_gate_selftest, [fed(SAMPLE_INDEX_REFERENCE_NAME)], work, None, False),
            ("样本档全登记、与暂存区相同的阶段目录", own_gate_selftest, [fed("gd-registered")], work, None, True),
            ("有登记表外阶段的目录", own_gate_selftest, [fed("gd-unregistered")], work, None, False),
            ("阶段目录写在变量里", own_gate_selftest, ["$GD"], work, None, False),
            ("--item 点名一道登记过的", own_gate_selftest, [fed("gd-unregistered"), "--item", "doc-text.sh"], work, None, True),
            ("--list-items 只列、不喂样本", own_gate_selftest, [fed("gd-unregistered"), "--list-items"], work, None, True),
            ("--item 点名不存在的阶段（它自己退 2、一份样本都不喂）", own_gate_selftest, [fed("gd-unregistered"), "--item", "no-such-stage.sh"], work, None, True),
            ("阶段目录写成 $PWD/…、--item 点名一道登记过的", own_gate_selftest, ["$PWD/gd-registered/.claude/gate.d", "--item", "doc-text.sh"], work, None, True),
            ("--item 与值之间夹的 --force 照 preflight 摘掉", own_gate_selftest, [fed("gd-registered"), "--item", "--force", "doc-text.sh"], work, None, True),
            ("--item 点名里夹一道登记表外的", own_gate_selftest, [fed("gd-unregistered"), "--item", "doc-text.sh", "--item", "brand-new-stage.sh"], work, None, False),
            ("--item 一个值里带空格点到登记表外的", own_gate_selftest, [fed("gd-unregistered"), "--item", "brand-new-stage.sh doc-text.sh"], work, None, False),
            ("--item 点名的样本多一份 setup.sh", own_gate_selftest, [fed("gd-sample-setup"), "--item", "doc-text.sh"], work, None, False),
            ("--item 点名写在变量里", own_gate_selftest, [fed("gd-registered"), "--item", "$STAGE"], work, None, False),
            ("阶段目录写成 $PWD/…、命令里另设了 PWD 的不换", own_gate_selftest, ["$PWD/gd-registered/.claude/gate.d", "--item", "doc-text.sh"], work, {"PWD": "/elsewhere"}, False),
            ("别处树里的 stage-selftest.sh 拷贝带 --list-items", evil_gate_selftest, [fed("gd-registered"), "--list-items"], work, None, False),
            ("别处树里的 stage-selftest.sh 拷贝 --item 点名一道", evil_gate_selftest, [fed("gd-registered"), "--item", "doc-text.sh"], work, None, False),
            ("别处树里的 stage-selftest.sh 拷贝不带 --item", evil_gate_selftest, [fed("gd-registered")], work, None, False),
            ("相对着写、当前目录不在本仓的 stage-selftest.sh 不带 --item", ".claude/singlefs-ai-sop/scripts/stage-selftest.sh", [fed("gd-registered")], work, None, False),
        ]
        for label, word, arguments, cwd, environment, want in stage_sample_cases:
            results.append((f"门禁样本：{label}", want, stage_selftest_problems(word, arguments, cwd, environment) == ""))
        refused_setup = stage_selftest_problems(own_gate_selftest, [fed("gd-sample-setup")], work)
        results.append(("门禁样本：样本与暂存区不同的说明点名那份文件", True, "red/setup.sh（暂存区里没有）" in refused_setup))
        refused_item = stage_selftest_problems(own_gate_selftest, [fed("gd-unregistered"), "--item", "doc-text.sh", "--item", "brand-new-stage.sh"], work)
        results.append(("门禁样本：--item 点名里夹登记表外的，说明点名那一道、不点没毛病的那一道", True,
                        "brand-new-stage.sh 的样本档没在登记表里" in refused_item and "doc-text.sh 的" not in refused_item))
        name_cases = [
            ("门禁阶段按名字判", ".claude/gate.d/doc-text.sh", work, True),
            ("去了编号的门禁阶段按名字判", ".claude/gate.d/lkmm.sh", work, True),
            ("门禁目录顶层的共用库 lib-*.sh 不是阶段", ".claude/gate.d/lib-sample.sh", work, False),
            ("lib/ 底下的共用库不是阶段", ".claude/gate.d/lib/stage-cells.sh", work, False),
            ("仓内位置上的 gate-staged.sh 按名字判", "scripts/gate/gate-staged.sh", work, True),
            ("仓内位置上的 crash-point-replay-shard-run.sh 按名字判", "scripts/crash/crash-point-replay-shard-run.sh", work, True),
            ("仓内位置上的 mutation-shard-run.sh 按名字判", "scripts/mutation/mutation-shard-run.sh", work, True),
            ("仓内位置上的 broad-mutation.py 按名字判", "scripts/mutation/broad-mutation.py", work, True),
            ("规范副本里的 stage-selftest.sh 按名字判（放行时不读正文）", ".claude/singlefs-ai-sop/scripts/stage-selftest.sh", work, True),
            ("仓外的同名 broad-mutation.py 不按名字判", "/tmp/claude-1000/somewhere/broad-mutation.py", work, False),
            ("从 research 里相对着写的 mutate.sh", "scripts/mutation/mutate.sh", os.path.join(work, "research"), True),
            ("仓外的同名 mutate.sh 不按名字判", "/tmp/claude-1000/somewhere/mutate.sh", work, False),
            ("仓内位置上的 crash-amplification.sh 按名字判（不读正文）", "scripts/crash/crash-amplification.sh", repository, True),
            ("仓内位置上的 build-patched-qemu.sh 按名字判（不读正文）", "scripts/devices/build-patched-qemu.sh", work, True),
            ("仓内位置上的 crash-point-replay.sh 按名字判（不读正文）", "scripts/crash/crash-point-replay.sh", repository, True),
            ("没登记的脚本", "run-chain.sh", work, False),
            ("路径尾部相同的别处 crash-point-replay.sh 拷贝不按名字判（读正文）", os.path.join(work, "copy-tree", KNOWN_SCRIPT_LOCATIONS["crash-point-replay.sh"]), work, False),
        ]
        os.makedirs(os.path.join(work, "copy-tree", os.path.dirname(KNOWN_SCRIPT_LOCATIONS["crash-point-replay.sh"])), exist_ok=True)
        with open(os.path.join(work, "copy-tree", KNOWN_SCRIPT_LOCATIONS["crash-point-replay.sh"]), "w", encoding="utf-8") as handle:
            handle.write("#!/usr/bin/env bash\necho 样本拷贝\n")
        for label, word, directory, want in name_cases:
            results.append((label, want, judged_by_name(word, directory)))
        # 跑不跑编译出来的代码：真 / 假
        compiled_cases = [
            ("cargo test 一个测试目标", ["cargo", "test", "-p", "singlefs-core", "--test", "core_contract"], True),
            ("cargo nextest run", ["cargo", "nextest", "run", "-p", "singlefs-core"], True),
            ("cargo miri test", ["cargo", "miri", "test", "-p", "singlefs-core"], True),
            ("cargo llvm-cov 不带子命令", ["cargo", "llvm-cov", "-p", "singlefs-core"], True),
            ("cargo mutants", ["cargo", "mutants", "-p", "singlefs-core"], True),
            ("--config 定的别名", ["cargo", "--config", 'alias.xt="test"', "xt"], True),
            ("cargo nextest list 不跑", ["cargo", "nextest", "list"], False),
            ("cargo llvm-cov report 不跑", ["cargo", "llvm-cov", "report"], False),
            ("cargo t 简写", ["cargo", "t", "--lib"], True),
            ("cargo run 实验二进制", ["cargo", "run", "--release", "--bin", "e160-random-small-read-share"], True),
            ("cargo r 简写", ["cargo", "r"], True),
            ("cargo bench", ["cargo", "bench"], True),
            ("cargo bench --no-run 只编不跑", ["cargo", "bench", "--no-run"], False),
            ("cargo mutants --help 只打帮助，不起 cargo test", ["cargo", "mutants", "--help"], False),
            ("cargo mutants -V -h 只打版本与帮助，不起 cargo test", ["cargo", "mutants", "-V", "-h"], False),
            ("--config 写在子命令之后的别名", ["cargo", "xt", "--config", 'alias.xt="test"'], True),
            ("全局选项在子命令前", ["cargo", "+stable", "--offline", "-C", "crates/singlefs-core", "test"], True),
            ("cargo test --no-run 只编不跑", ["cargo", "test", "--no-run", "-p", "singlefs-core"], False),
            ("-- 之后的 --no-run 是测试二进制的参数", ["cargo", "test", "--", "--no-run"], True),
            ("cargo build", ["cargo", "build", "--release"], False),
            ("cargo b 简写是 build", ["cargo", "b"], False),
            ("cargo clippy", ["cargo", "clippy", "--all-targets"], False),
            ("cargo 只带全局选项", ["cargo", "--version"], False),
            ("直接执行测试二进制", [os.path.join("target", "debug", "deps", "overwrite_in_one_instance-0123456789abcdef")], True),
            ("直接执行 research 里的实验二进制", ["scripts/experiment-apparatus/target/release/e142_region_diff_independent"], True),
            ("直接执行名字里带 target 的自定编译目录里的二进制", ["/tmp/singlefs-crates-mutation-target/release/tiny"], True),
            ("带目标三元组的", ["./target/x86_64-unknown-linux-musl/release/new_pool_file_creation_on_device"], True),
            ("deps 下的 .d 依赖文件", [os.path.join("target", "release", "deps", "tiny.d")], False),
            ("名字只当参数", ["ls", "target/release/tiny"], False),
            ("不在 target 底下的程序", ["./prog"], False),
        ]
        for label, words, want in compiled_cases:
            results.append((f"跑编译出来的代码：{label}", want, runs_compiled_code(words, work) is not None))
        # 起不起编译（与谁起、在哪都无关，starts_a_build）：开关 build-subcommands-unjudged 下 cargo 会编的那几格红（应当是 True，实际 False），
        # build-scripts-unjudged 下 compile-then-swap.py 起编译的两格红
        build_cases = [
            ("cargo build", ["cargo", "build", "--offline", "--all-targets"], True),
            ("cargo b 简写", ["cargo", "b"], True),
            ("cargo check", ["cargo", "check", "-p", "singlefs-core"], True),
            ("cargo clippy", ["cargo", "clippy", "--all-targets"], True),
            ("cargo test --no-run 只编不跑也算", ["cargo", "test", "--offline", "--no-run", "-p", "singlefs-verification", "--release"], True),
            ("cargo doc", ["cargo", "doc", "--no-deps"], True),
            ("cargo run", ["cargo", "run", "--bin", "e160-random-small-read-share"], True),
            ("全局选项在子命令前", ["cargo", "+stable", "--offline", "-C", "crates/singlefs-core", "build"], True),
            ("--config 定的别名当会编", ["cargo", "--config", 'alias.xb="build"', "xb"], True),
            ("cargo fmt 不编", ["cargo", "fmt", "--all"], False),
            ("cargo metadata 不编", ["cargo", "metadata", "--format-version", "1", "--no-deps"], False),
            ("cargo tree 不编", ["cargo", "tree"], False),
            ("cargo clean 不编", ["cargo", "clean"], False),
            ("cargo 只带全局选项", ["cargo", "--version"], False),
            ("python3 起 compile-then-swap.py", ["python3", "-B", "scripts/hosts/compile-then-swap.py", "draft.rs", "target.rs", "--scratch", "s"], True),
            ("直接执行别处树里的 compile-then-swap.py", ["/elsewhere/scripts/hosts/compile-then-swap.py", "draft.rs", "target.rs", "--scratch", "s"], True),
            ("compile-then-swap.py 单独一个 --selftest 用假 cargo，不算", ["python3", "scripts/hosts/compile-then-swap.py", "--selftest"], False),
            ("python3 -c 的代码里写着那份脚本的名字不算", ["python3", "-c", "print('compile-then-swap.py')"], False),
            ("不起 cargo 的脚本", ["python3", "scripts/hosts/heavy_threads.py"], False),
        ]
        for label, words, want in build_cases:
            results.append((f"起编译：{label}", want, starts_a_build(words, work) is not None))

        # 一百三十七节那一批（records/2026-10-29-一百三十七节那一轮调度记录.md 件 1）：名单外启动器（开关 extra-launchers-unseen 下「启动器：」开头的格红，应当是起的那条命令，实际 None）
        extra_launcher_cases = [
            ("启动器：xargs 接在命令后面的读进来的词", ["xargs", "cargo", "test", "-p", "singlefs-verification"], ["cargo", "test", "-p", "singlefs-verification", XARGS_INPUT_WORD]),
            ("启动器：xargs -I{} 换掉替换串", ["xargs", "-I{}", "sh", "-c", "cargo test {}"], ["sh", "-c", f"cargo test {XARGS_INPUT_WORD}"]),
            ("启动器：xargs 不给命令是 echo", ["xargs", "-0"], ["echo", XARGS_INPUT_WORD]),
            ("启动器：find -exec 的命令、{} 换成找到的路径", ["find", ".", "-name", "s.sh", "-exec", "bash", "{}", ";"], ["bash", FIND_PATH_WORD]),
            ("启动器：find 两个 -exec 连成 sh -c", ["find", ".", "-exec", "true", ";", "-execdir", "cargo", "test", "+"], ["sh", "-c", "true ; cargo test"]),
            ("启动器：find -exec 没有收尾的 ; 什么都不起", ["find", ".", "-exec", "cargo", "test"], None),
            ("启动器：ssh 把 host 之后的词交给远端 shell", ["ssh", "-p", "22", "localhost", "cargo test --workspace"], ["sh", "-c", "cargo test --workspace"]),
            ("启动器：doas -u 用户", ["doas", "-u", "root", "cargo", "test"], ["cargo", "test"]),
            ("启动器：numactl -N0 贴着写", ["numactl", "-N0", "cargo", "test"], ["cargo", "test"]),
            ("启动器：numactl --membind 值另起一个词", ["numactl", "--membind", "0", "cargo", "test"], ["cargo", "test"]),
            ("启动器：script -qec 的代码", ["script", "-qec", "cargo test --workspace", "/dev/null"], ["sh", "-c", "cargo test --workspace"]),
            ("启动器：unshare --user", ["unshare", "--user", "cargo", "test"], ["cargo", "test"]),
            ("启动器：setpriv --reuid=", ["setpriv", "--reuid=1000", "cargo", "test"], ["cargo", "test"]),
            ("启动器：nsenter -t 进程号", ["nsenter", "-t", "1", "-m", "cargo", "test"], ["cargo", "test"]),
            ("启动器：su -c 的代码", ["su", "-c", "cargo test", "me"], ["sh", "-c", "cargo test"]),
            ("启动器：runuser -u 用户 --", ["runuser", "-u", "me", "--", "cargo", "test"], ["cargo", "test"]),
            ("启动器：sg 组 -c", ["sg", "users", "-c", "cargo test"], ["sh", "-c", "cargo test"]),
            ("启动器：watch -n 1", ["watch", "-n", "1", "cargo", "test"], ["sh", "-c", "cargo test"]),
            ("启动器：gdb --args", ["gdb", "-batch", "--args", "cargo", "test"], ["cargo", "test"]),
            ("启动器：scl enable 集合 --", ["scl", "enable", "x", "--", "cargo", "test"], ["cargo", "test"]),
            ("启动器：parallel ::: 之前是命令", ["parallel", "-j", "2", "cargo", "test", ":::", "a"], ["cargo", "test", PARALLEL_INPUT_WORD]),
            ("启动器：bwrap --bind 带两个值", ["bwrap", "--bind", "/", "/", "--dev", "/dev", "cargo", "test"], ["cargo", "test"]),
            ("启动器：ltrace -o 文件", ["ltrace", "-o", "t.log", "cargo", "test"], ["cargo", "test"]),
            ("启动器：valgrind --tool=", ["valgrind", "--tool=memcheck", "cargo", "test"], ["cargo", "test"]),
            ("启动器：faketime 时间戳", ["faketime", "now", "cargo", "test"], ["cargo", "test"]),
            ("启动器：unbuffer -p", ["unbuffer", "-p", "cargo", "test"], ["cargo", "test"]),
            ("启动器：cpulimit -l 50 --", ["cpulimit", "-l", "50", "--", "cargo", "test"], ["cargo", "test"]),
            ("启动器：dbus-run-session --", ["dbus-run-session", "--", "cargo", "test"], ["cargo", "test"]),
            ("启动器：firejail --quiet", ["firejail", "--quiet", "cargo", "test"], ["cargo", "test"]),
            ("启动器：proot -r 根", ["proot", "-r", "/", "cargo", "test"], ["cargo", "test"]),
            ("启动器：chroot 新根", ["chroot", "/", "cargo", "test"], ["cargo", "test"]),
            ("启动器：cgexec -g", ["cgexec", "-g", "cpu:x", "cargo", "test"], ["cargo", "test"]),
        ]
        for label, words, want in extra_launcher_cases:
            got = command_under_launcher(words, work)
            results.append((label, want, got[0] if got else None))
        results.append(("启动器：名单外的几个与 launcher_names() 一致", True, EXTRA_LAUNCHER_NAMES <= launcher_names()))
        # 派发起法（records/2026-10-29-一百三十七节那一轮调度记录.md#8；开关 dispatch-launchers-unseen 下交回命令的那一格与「派发起法：multi-host-run.sh --peer -- <脚本> --selftest」一格红）
        dispatch, multi_host = "scripts/hosts/host-dispatch.py", "scripts/hosts/multi-host-run.sh"
        dispatch_launcher_cases = [
            ("启动器：multi-host-run.sh --peer 带 --memory-cap、--label 各一个值", [multi_host, *"--peer --memory-cap 2G --label x --".split(), "cargo", "test"],
             ["cargo", "test"]),
            ("启动器：multi-host-run.sh --selftest 是它自己的自证，后面写了 -- 也不剥", [multi_host, "--selftest", "--", "cargo", "test", "--workspace"], None),
            ("启动器：别的目录里的 multi-host-run.sh --peer 不剥", ["tools/multi-host-run.sh", "--peer", "--", "cargo", "test", "--workspace"], None),
        ]
        for label, words, want in dispatch_launcher_cases:
            got = command_under_dispatch_launcher(words, work)
            results.append((label, want, got[0] if got else None))
        foreign_dispatch = os.path.join(work, "foreign-tree", "scripts", "hosts", "multi-host-run.sh")
        os.makedirs(os.path.dirname(foreign_dispatch), exist_ok=True)
        with open(foreign_dispatch, "w", encoding="utf-8") as handle:
            handle.write('#!/usr/bin/env bash\nwhile [[ $# -gt 0 && $1 != -- ]]; do shift; done; shift; exec "$@"\n')
        for label, want, got in (
                ("派发起法：multi-host-run.sh --peer -- <脚本> --selftest 不当派发器自己的自证", None,
                 getattr(registered_selftest(os.path.join(HOOK_REPOSITORY, multi_host), ["--peer", "--label", "x", "--", "python3", "scripts/agents/dispatch-prompt.py", "--selftest"], work),
                         "kind", None)),
                ("派发起法：host-dispatch.py --selftest 照它自己的自证认（登记过）", "selftest", getattr(registered_selftest(os.path.join(HOOK_REPOSITORY, dispatch), ["--selftest"], work), "kind", None)),
                ("派发起法：--peer 剥开的那条算经内存包装", True, dispatched_under_memory_cap([multi_host, "--peer", "--", "cargo", "test"], work)),
                ("派发起法：别处树里内容不同的 multi-host-run.sh --peer 拷贝剥开的不算经内存包装", False,
                 dispatched_under_memory_cap([foreign_dispatch, "--peer", "--", "cargo", "test"], work))):
            results.append((label, want, got))
        # 认不出起什么的写法（开关 unrecognized-launches-allowed 下「认不出：」开头照拒的格红，应当是 True，实际 False）
        unrecognized_cases = [
            ("认不出：命令词是变量", ["$c", "-p", "hi"], None, True),
            ("认不出：命令词是命令替换", ["$(which cargo)", "test", "--workspace"], None, True),
            ("认不出：命令词展开出来夹着空白", ["cargo test --workspace"], None, True),
            ("认不出：bash 起的脚本是进程替换", ["<(echo cargo test)"], "shell", True),
            ("认不出：source 进程替换", ["source", "<(cat s.sh)"], None, True),
            ("认不出：cargo 子命令是命令替换", ["cargo", "$(echo test)", "--workspace"], None, True),
            ("认不出：cargo test 的参数是变量", ["cargo", "test", "$F"], None, True),
            ("认不出：cargo test 的参数是位置参数", ["cargo", "test", "$@"], None, True),
            ("认得出：cargo build 的参数是变量（不跑编出来的代码）", ["cargo", "build", "$F"], None, False),
            ("认得出：cargo test 的 --target-dir 写成变量", ["cargo", "test", "-p", "singlefs-verification", "--target-dir", "$T"], None, False),
            ("认得出：cargo test 的 -- 之后写成变量", ["cargo", "test", "-p", "singlefs-verification", "--", "$FILTER"], None, False),
            ("认得出：字面命令", ["ls", "$HOME"], None, False),
            ("认得出：命令词是 $CLAUDE_CODE_EXECPATH（检出钩子 ⑬ 判起 claude）", ["$CLAUDE_CODE_EXECPATH", "-p", "hi"], None, False),
            ("认得出：命令词是 ${CLAUDE_CODE_EXECPATH}（检出钩子 ⑬ 判起 claude）", ["${CLAUDE_CODE_EXECPATH}", "--print", "hi"], None, False),
        ]
        for label, words, launcher, want in unrecognized_cases:
            results.append((label, want, unrecognized_launch(words, launcher) is not None))
        # 起 claude 的那一组与检出钩子那份同一组（读它文件里那一行的字面集合）
        detector_text = open(os.path.join(os.path.dirname(os.path.abspath(__file__)), "bash-command-detector.sh"), encoding="utf-8").read()
        detector_words = re.search(r"^CLAUDE_EXECUTABLE_PATH_WORDS = (\{.*\})$", detector_text, re.MULTILINE)
        results.append(("认得出：起 claude 的命令词与 bash-command-detector.sh 的 CLAUDE_EXECUTABLE_PATH_WORDS 同一组", set(CLAUDE_EXECUTABLE_PATH_WORDS),
                        set(ast.literal_eval(detector_words.group(1))) if detector_words else None))
        # 读进来的脚本正文里经变量起的自证、门禁阶段、入口脚本、cargo（一百三十七节件 1b 第 3 条；开关 script-body-launches-unjudged 下「照认不出」的三格红，应当是 True，实际 False）
        draft_script = os.path.join(os.sep, "draft-outside-the-repositories", "runs.sh")   # work 在这一段照「本仓」认（FAMILY_OWN_REPOSITORY），草稿脚本取在它外面
        body_cases = [
            ("脚本正文：数组变量的定义里写着 --selftest，照认不出", ["${cmds[$i]}"], None, {"cmds": ["(bash $R/.claude/hooks/x.sh --selftest)"], "i": ["${!cmds[@]}"]}, (), draft_script, True),
            ("脚本正文：\"$@\" 转发、起它的参数里写着 --selftest，照认不出", ["$@"], None, {}, ("bash", "/tree/.claude/hooks/x.sh", "--selftest"), draft_script, True),
            ("脚本正文：cargo test 的参数是解不出的变量，照认不出", ["cargo", "test", "$F"], None, {"F": ["$(cat f)"]}, (), draft_script, True),
            ("脚本正文：\"$@\" 转发、参数里没有这几样，不判", ["$@"], None, {}, ("ls", "-l"), draft_script, False),
            ("脚本正文：bash 起的 $X 定义里只有普通脚本，不判", ["$X"], "shell", {"X": ["/tree/report.sh"]}, (), draft_script, False),
            ("脚本正文：脚本起它自己（$0 --selftest），不判", ["$0", "--selftest"], "shell", {}, (), draft_script, False),
            ("脚本正文：本仓里的脚本不判", ["${cmds[$i]}"], None, {"cmds": ["(bash $R/x.sh --selftest)"]}, (), os.path.join(HOOK_REPOSITORY, "scripts", "hosts", "multi-host-run.sh"), False),
        ]
        for label, words, launcher, definitions, positional, script_path, want in body_cases:
            results.append((label, want, script_body_unresolved_launch(words, launcher, definitions, positional, script_path) is not None))
        # cargo 参数写成前面赋过值的变量（开关 cargo-arguments-unexpanded 下这格红）
        results.append(("cargo 参数展开：F=--workspace 之后 cargo test $F", ["cargo", "test", "--workspace"],
                        cargo_arguments_expanded(["cargo", "test", "$F"], {"F": "--workspace"})))
        # 读正文时按脚本所在目录 source 的库换成字面路径（开关 sourced-libraries-unread 下这格红）
        results.append(("source 的库：$(dirname \"${BASH_SOURCE[0]}\") 换成脚本所在目录", 'source "/tree/scripts/../lib.sh"',
                        script_directory_expanded('source "$(dirname "${BASH_SOURCE[0]}")/../lib.sh"', "/tree/scripts/x.sh")))
        # 同族脚本只认本仓那一份（第九十八节 ②；开关 same-family-by-path-tail 下「同族：」开头三格红，应当是 False / broad-mutation，实际 True / None）
        FAMILY_OWN_REPOSITORY = None
        try:
            elsewhere = os.path.join(work, "family-elsewhere")
            for relative in (".claude/gate.d/doc-text.sh", "scripts/mutation/mutate.sh", "scripts/mutation/broad-mutation.py"):
                os.makedirs(os.path.dirname(os.path.join(elsewhere, relative)), exist_ok=True)
                with open(os.path.join(elsewhere, relative), "w", encoding="utf-8") as handle:
                    handle.write("cargo test --workspace\n")
            results.append(("同族：别处树里的门禁阶段拷贝不按名字判（读正文）", False, judged_by_name(".claude/gate.d/doc-text.sh", elsewhere)))
            results.append(("同族：别处树里的 mutate.sh 拷贝不按名字判（读正文）", False, judged_by_name("scripts/mutation/mutate.sh", elsewhere)))
            results.append(("同族：本仓那一份 mutate.sh 照旧按名字判", True, judged_by_name("scripts/mutation/mutate.sh", repository)))
            # 与本仓那一份逐字节相同的拷贝照本仓那一份认（family-copy-content-unchecked 下这格红）
            identical = os.path.join(work, "family-identical")
            os.makedirs(os.path.join(identical, "scripts", "mutation"), exist_ok=True)
            shutil.copyfile(os.path.join(HOOK_REPOSITORY, KNOWN_SCRIPT_LOCATIONS["mutate.sh"]), os.path.join(identical, "scripts", "mutation", "mutate.sh"))
            results.append(("同族：与本仓那一份逐字节相同的 mutate.sh 拷贝照按名字判", True, judged_by_name("scripts/mutation/mutate.sh", identical)))
        finally:
            FAMILY_OWN_REPOSITORY = work
        # 经 task-server 起（文件头「经 task-server 起」）：本仓的代码怎么认、名单上直接跑的、submit 后面那条、绕开服务的写法。
        # 开关 project-scripts-unjudged 下「本仓代码：」照拒的那几格红（应当是 nobody，实际 None）；text-tools-refused 下「名单：」谁都能跑的几格红（应当是 anyone，实际 nobody）；
        # main-agent-tools-refused 下「名单：主 agent」四格红；task-server-submission-unparsed 下「提交：」认出的三格红（实际 None）；
        # task-server-paths-unjudged 下「绕开：」点到库、密钥、socket 与载入共用库的四格红（应当是 True，实际 False）；admitted-variable-unjudged 下「绕开：」设准入开关的两格红；
        # task-server-scripts-unjudged 下「绕开：直接起服务本体」一格红；task-server-units-unjudged 下「绕开：systemctl」照拒的三格红；
        # task-server-builds-unjudged 下「绕开：」删编译目录的七格红（应当是 True，实际 False）；text-tools-refused 下「名单：提交客户端的 clean」一格红
        copy_root = os.path.join(work, "task-server-copy")
        for relative in ("scripts/kb/replace-once.py", TASK_SERVER_CLIENT):
            os.makedirs(os.path.dirname(os.path.join(copy_root, relative)), exist_ok=True)
            if os.path.isfile(os.path.join(HOOK_REPOSITORY, relative)):
                shutil.copyfile(os.path.join(HOOK_REPOSITORY, relative), os.path.join(copy_root, relative))
        changed_root = os.path.join(work, "task-server-changed-copy")
        os.makedirs(os.path.join(changed_root, "scripts", "kb"), exist_ok=True)
        with open(os.path.join(changed_root, "scripts", "kb", "replace-once.py"), "w", encoding="utf-8") as handle:
            handle.write("import subprocess\nsubprocess.run(['cargo', 'test', '--all'])\n")
        os.makedirs(os.path.join(changed_root, "task-server"), exist_ok=True)
        with open(os.path.join(changed_root, TASK_SERVER_CLIENT), "w", encoding="utf-8") as handle:
            handle.write("#!/usr/bin/env python3\nimport subprocess, sys\nsubprocess.run(sys.argv[sys.argv.index('--') + 1:])\n")
        with open(os.path.join(work, "own-draft-helper.sh"), "w", encoding="utf-8") as handle:
            handle.write("#!/usr/bin/env bash\n# 草稿里自己的辅助脚本\necho hi\n")
        with open(os.path.join(work, "renamed-with-header.py"), "w", encoding="utf-8") as handle:
            handle.write("#!/usr/bin/env python3\n# admission: always 样本\n# run-condition: none 样本\nprint(1)\n")
        def project_level(words, directory, launcher):
            found = project_code_use(words, directory, launcher)
            return None if found is None else found.allowed
        for label, words, directory, launcher, want in (
                ("本仓代码：python3 起本仓的脚本", ["scripts/gate/hook-swap.py", "--dry-run"], HOOK_REPOSITORY, "python", "nobody"),
                ("本仓代码：bash 起本仓的钩子（喂钩子）", [".claude/hooks/write-guard.sh"], HOOK_REPOSITORY, "shell", "nobody"),
                ("本仓代码：仓副本里改过正文的文本工具不算名单上的", ["scripts/kb/replace-once.py", "a", "b", "c"], changed_root, "python", "nobody"),
                ("本仓代码：改名拷走、文件头带准入声明", ["renamed-with-header.py"], work, "python", "nobody"),
                ("本仓代码：python3 -m pytest 起单测", ["pytest.py", "scripts/hosts/tests"], HOOK_REPOSITORY, "python", "nobody"),
                ("本仓代码：文本工具带 --selftest 不在名单的参数里", ["scripts/kb/replace-once.py", "--selftest"], HOOK_REPOSITORY, "python", "nobody"),
                ("本仓代码：work-directory-ownership.sh 的 sweep 不在名单的参数里", ["scripts/hosts/work-directory-ownership.sh", "sweep"], HOOK_REPOSITORY, "shell", "nobody"),
                ("本仓代码：gate-overlap.py 不带 --list", [".claude/singlefs-ai-sop/scripts/gate-overlap.py", "."], HOOK_REPOSITORY, "python", "nobody"),
                ("本仓代码：minimal-copy.py 带 --selftest 不在名单的参数里", ["scripts/hosts/minimal-copy.py", "--selftest"], HOOK_REPOSITORY, "python", "nobody"),
                ("本仓代码：lib_repository_copy.py 的 link-into 不在名单的参数里",
                 ["scripts/hosts/lib_repository_copy.py", "link-into", "/x", "--root", HOOK_REPOSITORY, "--workspace", "scripts/experiment-apparatus"],
                 HOOK_REPOSITORY, "python", "nobody"),
                ("名单：replace-once.py 定点改", ["scripts/kb/replace-once.py", "a", "b", "c"], HOOK_REPOSITORY, "python", "anyone"),
                ("名单：仓副本里逐字节相同的 replace-once.py", ["scripts/kb/replace-once.py", "a", "b", "c"], copy_root, "python", "anyone"),
                ("名单：认领草稿目录", ["scripts/hosts/work-directory-ownership.sh", "claim", "/x"], HOOK_REPOSITORY, "shell", "anyone"),
                ("名单：gate-overlap.py --list", [".claude/singlefs-ai-sop/scripts/gate-overlap.py", "--list"], HOOK_REPOSITORY, "python", "anyone"),
                ("名单：提交客户端的 wait", [TASK_SERVER_CLIENT, "wait", "7"], HOOK_REPOSITORY, None, "anyone"),
                ("名单：提交客户端的 clean", [TASK_SERVER_CLIENT, "clean", "7"], HOOK_REPOSITORY, None, "anyone"),
                ("名单：task-server/control check", ["task-server/control", "check"], HOOK_REPOSITORY, "python", "anyone"),
                ("名单：改标签的 relabel-item.py", ["scripts/kb/relabel-item.py", "D19", "6", "--dry-run"], HOOK_REPOSITORY, "python", "anyone"),
                ("名单：备份原文件的 copy-originals.py", ["scripts/agents/copy-originals.py", "copy", "/x", "a.md"], HOOK_REPOSITORY, "python", "anyone"),
                ("名单：直接起 ledger-append-section.py 不拒", ["scripts/kb/ledger-append-section.py", "--title", "x", "--body-file", "-", "--dry-run"],
                 HOOK_REPOSITORY, "python", "anyone"),
                ("名单：直接起 append-record.py 不拒", ["scripts/kb/append-record.py", "records/x.md", "-", "--dry-run"], HOOK_REPOSITORY, "python", "anyone"),
                ("名单：建仓副本的 minimal-copy.py", ["scripts/hosts/minimal-copy.py", HOOK_REPOSITORY, "/x/copy", "--with-test-inputs"], HOOK_REPOSITORY, "python", "anyone"),
                ("名单：打副本闭包的 lib_repository_copy.py entries", ["scripts/hosts/lib_repository_copy.py", "entries", HOOK_REPOSITORY, "--test-inputs"],
                 HOOK_REPOSITORY, "python", "anyone"),
                ("名单：主 agent 停进程的 proc.py", [".claude/singlefs-ai-sop/scripts/proc.py", "stop", "1"], HOOK_REPOSITORY, "python", "main-agent"),
                ("名单：主 agent 的 task-server/control stop", ["task-server/control", "stop"], HOOK_REPOSITORY, "python", "main-agent"),
                ("名单：主 agent 的 task-server/control pause", ["task-server/control", "pause", "--reason", "x"], HOOK_REPOSITORY, "python", "main-agent"),
                ("名单：主 agent 的 task-server/control resume", ["task-server/control", "resume"], HOOK_REPOSITORY, "python", "main-agent"),
                ("名单：主 agent 直接起服务自证", ["task-server/task_server_lib.py", "--selftest"], HOOK_REPOSITORY, "python", "main-agent"),
                ("名单：服务自证带别的参数不认", ["task-server/task_server_lib.py", "--selftest", "x"], HOOK_REPOSITORY, "python", "nobody"),
                ("名单：主 agent 起定时提醒的 remind.sh", ["scripts/agents/remind.sh", "30", "x"], HOOK_REPOSITORY, "shell", "main-agent"),
                ("不是本仓代码：草稿里自己的辅助脚本", ["own-draft-helper.sh"], work, "shell", None),
                ("不是本仓代码：PATH 上的 cargo（由 starts_a_build 认）", ["cargo", "build"], HOOK_REPOSITORY, None, None),
                ("不是本仓代码：盘上不在的脚本", ["not-here-137.sh"], work, "shell", None),
                ("不是本仓代码：python3 -m py_compile（语法检查）", ["py_compile.py", "x.py"], work, "python", None)):
            results.append((label, want, project_level(words, directory, launcher)))
        for label, words, directory, want in (
                ("提交：submit -- 后面那条", [TASK_SERVER_CLIENT, "submit", "--", "cargo", "build"], HOOK_REPOSITORY, (["cargo", "build"], None)),
                ("提交：--cwd 照这一条的目录解开", [TASK_SERVER_CLIENT, "submit", "--cwd", "crates", "--", "ls"], HOOK_REPOSITORY,
                 (["ls"], os.path.join(HOOK_REPOSITORY, "crates"))),
                ("提交：仓副本里逐字节相同的客户端", [TASK_SERVER_CLIENT, "submit", "--", "ls"], copy_root, (["ls"], None)),
                ("提交：没写 -- 的是用法错，不认", [TASK_SERVER_CLIENT, "submit", "cargo", "build"], HOOK_REPOSITORY, None),
                ("提交：wait 不是提交", [TASK_SERVER_CLIENT, "wait", "7"], HOOK_REPOSITORY, None),
                ("提交：别处同名而内容不同的 task 不认", ["task-server/task", "submit", "--", "ls"], changed_root, None)):
            results.append((label, want, task_server_submission(words, directory)))
        # 子 agent 起提交客户端只许 status、log（subagent_task_server_refusal；project-code-guard.sh 与 run-scope-guard.sh 都调它）
        for label, words, directory, launcher, want in (
                ("子 agent 起客户端：submit 拒", [TASK_SERVER_CLIENT, "submit", "--", "ls"], HOOK_REPOSITORY, None, f"{TASK_SERVER_CLIENT} submit"),
                ("子 agent 起客户端：python3 起的 submit 拒", [TASK_SERVER_CLIENT, "submit", "--", "ls"], HOOK_REPOSITORY, "python", f"{TASK_SERVER_CLIENT} submit"),
                ("子 agent 起客户端：仓副本里逐字节相同的客户端 wait 拒", [TASK_SERVER_CLIENT, "wait", "7"], copy_root, None, f"{TASK_SERVER_CLIENT} wait"),
                ("子 agent 起客户端：cancel 拒", [TASK_SERVER_CLIENT, "cancel", "7"], HOOK_REPOSITORY, None, f"{TASK_SERVER_CLIENT} cancel"),
                ("子 agent 起客户端：clean 拒", [TASK_SERVER_CLIENT, "clean", "7"], HOOK_REPOSITORY, None, f"{TASK_SERVER_CLIENT} clean"),
                ("子 agent 起客户端：没写子命令拒", [TASK_SERVER_CLIENT], HOOK_REPOSITORY, None, f"{TASK_SERVER_CLIENT}（没写子命令）"),
                ("子 agent 起客户端：status 放", [TASK_SERVER_CLIENT, "status", "7"], HOOK_REPOSITORY, None, None),
                ("子 agent 起客户端：log 放", [TASK_SERVER_CLIENT, "log", "7", "--all"], HOOK_REPOSITORY, None, None),
                ("子 agent 起客户端：别处同名而内容不同的 task 不认（由 project_code_use 照本仓的代码判）", ["task-server/task", "submit", "--", "ls"], changed_root, None, None),
                ("子 agent 起客户端：control check 不是提交客户端", ["task-server/control", "check"], HOOK_REPOSITORY, "python", None)):
            results.append((label, want, subagent_task_server_refusal(words, directory, launcher)))
        # 确认单打的确认命令（task_server_lib.confirm_command，出路本身要过得了别的闸）：主 agent 直接起不拒，子 agent 起照 submit 那样拒
        library, library_problem = task_server_library()
        confirm_words = shlex.split(library.confirm_command(7, 2, 1)) if library is not None else None
        results.append(("名单：确认单打的 task confirm 主 agent 直接起", "anyone",
                        project_level(confirm_words, HOOK_REPOSITORY, None) if confirm_words else library_problem))
        results.append(("子 agent 起客户端：确认单打的 task confirm 拒", f"{TASK_SERVER_CLIENT} confirm",
                        subagent_task_server_refusal(confirm_words, HOOK_REPOSITORY, None) if confirm_words else library_problem))
        guarded, guarded_problem = task_server_guarded()
        results.append(("绕开：读得到 task-server/task_server_lib.py 的名字与缺省值", None, guarded_problem))
        if guarded is not None:
            for label, text, want in (
                    ("绕开：点到库文件名", "sqlite3 ~/.local/state/singlefs/task-server/task-server.sqlite3 .tables", True),
                    ("绕开：点到签名密钥文件名", "cat ~/.config/singlefs/task-server-signing.key", True),
                    ("绕开：点到 socket", "curl --unix-socket /run/user/1000/singlefs-task-server/server.sock http://x/health", True),
                    ("绕开：python 代码载入 task_server_lib", "python3 -c 'import task_server_lib'", True),
                    ("绕开：命令文本里 export 准入开关", f"export {guarded.admitted_variable}=1.1", True),
                    ("绕开：python 代码里写 environ 的准入开关", f"python3 -c \"import os; os.environ['{guarded.admitted_variable}'] = '1'\"", True),
                    ("绕开：只读日志目录不算", "ls ~/.local/state/singlefs/task-server/logs", False),
                    ("绕开：提交客户端本身不算", "task-server/task submit -- cargo build", False),
                    ("绕开：python 代码 shutil.rmtree 服务的编译目录", f"python3 -c \"import shutil; shutil.rmtree('{guarded.build_directories[0]}/task-x-7-1')\"", True),
                    ("绕开：python 代码里删别处的目录不算", "python3 -c \"import shutil; shutil.rmtree('/tmp/not-a-task-server-directory')\"", False),
                    ("绕开：task-server/task clean 不算", "task-server/task clean 7", False)):
                results.append((label, want, bool(task_server_text_bypass(text)[0])))
            build_root = guarded.build_directories[0]
            home = os.path.expanduser("~")
            tilde_root = "~" + build_root[len(home):] if build_root.startswith(home + os.sep) else build_root
            state_builds = guarded.build_directories[-1]
            for label, words, environment, want in (
                    ("绕开：命令前设准入开关", ["ls"], {guarded.admitted_variable: "1"}, True),
                    ("绕开：直接起服务本体", ["task-server/task_server.py", "serve"], {}, True),
                    ("绕开：systemctl --user stop 服务单元", ["systemctl", "--user", "stop", guarded.server_unit], {}, True),
                    ("绕开：systemctl --user kill 一次尝试的单元", ["systemctl", "--user", "kill", guarded.task_unit_prefix + "main-3-1.service"], {}, True),
                    ("绕开：systemctl --user stop 通配", ["systemctl", "--user", "stop", guarded.task_unit_prefix + "*"], {}, True),
                    ("绕开：systemctl --user status 服务单元不算", ["systemctl", "--user", "status", guarded.server_unit], {}, False),
                    ("绕开：systemctl stop 别的单元不算", ["systemctl", "--user", "stop", "other.service"], {}, False),
                    ("绕开：rm -rf 服务的编译目录", ["rm", "-rf", "--", os.path.join(build_root, "task-x-7-1")], {}, True),
                    ("绕开：rm 写 ~ 起头的编译目录", ["rm", "-r", tilde_root + "/task-x-7-1/target"], {}, True),
                    ("绕开：rm 状态目录下的 builds", ["rm", "-rf", os.path.join(state_builds, "task-x-7-1")], {}, True),
                    ("绕开：find -delete 编译目录", ["find", build_root, "-name", "*.rlib", "-delete"], {}, True),
                    ("绕开：find -exec rm 编译目录", ["find", "-L", build_root, "-type", "d", "-exec", "rm", "-rf", "{}", "+"], {}, True),
                    ("绕开：rm -rf 编译目录的上级", ["rm", "-rf", os.path.dirname(build_root)], {}, True),
                    ("绕开：du 编译目录不算", ["du", "-sh", build_root], {}, False),
                    ("绕开：find 编译目录不删不算", ["find", build_root, "-maxdepth", "1"], {}, False),
                    ("绕开：rm 别处的目录不算", ["rm", "-rf", "/tmp/not-a-task-server-directory"], {}, False)):
                launcher = "python" if words[0].endswith(".py") else None
                results.append((label, want, bool(task_server_command_bypass(words, environment, HOOK_REPOSITORY, launcher)[0])))
    finally:
        SAMPLE_INDEX_REPOSITORY = real_sample_index
        FAMILY_OWN_REPOSITORY = None
        shutil.rmtree(work)
    failures = [item for item in results if item[1] != item[2]]
    for label, want, got in failures:
        print(f"  ✗ lib_project_code_runs 自检：{label} 应当是 {want}，实际 {got}")  # gate-lint:detail
    if failures:
        print("    → 看 classify()、external_program_use()、word_expansion_use()、pattern_expansion_use()、brace_expansion()、command_under_launcher()、launcher_option()、"
              "runs_compiled_code()、starts_a_build()、test_invocation()、cargo_subcommand()、judged_by_name()、path_without_symbolic_links()、registered_selftest()、"
              "selftest_registry_at()、selftest_registry_problems()、stage_selftest_problems()、stage_selftest_arguments()、stage_selftest_item_problems()、stage_feed_problem()、"
              "project_code_use()、project_relative_path()、directly_run_level()、task_server_submission()、task_server_text_bypass()、task_server_command_bypass()、"
              "task_server_guarded()、unrecognized_launch()、script_body_unresolved_launch()、temporary_service_bare_selftest()、"
              "lib_shell_words.py 的 python_invocation() 与 TEST_BINARY_PATH / BUILT_BINARY_PATH / COMPILED_CODE_SUBCOMMANDS / TEXT_TOOLS_RUN_DIRECTLY / MAIN_AGENT_TOOLS_RUN_DIRECTLY / "
              f"KNOWN_SCRIPT_LOCATIONS / LAUNCHER_OPTIONS_WITH_VALUE / EXTERNAL_PROGRAM_NAMES 几张表与登记表 {SELFTEST_REGISTRY_NAME}；{BREAK_VARIABLE} 设着的话这里本来就该红")
        return 1
    print(f"  ✓ lib_project_code_runs 自检通过（查了 {len(results)} 种：外部程序名单 {len(external_cases)} 种、"
          f"/usr/bin/time、flock、systemd-run 这一类包在外面的 {len(launcher_cases)} 种与它们设的环境变量 {len(environment_cases)} 种、"
          f"bash 起不来时的 POSIX 字符类 {len(bash_missing_cases)} 种、喂给 shell 的标准输入 {len(standard_input_cases)} 种、花括号展开与 bash 对照 {len(brace_words)} 种、"
          f"门禁样本 {len(stage_sample_cases) + 2} 种、样本登记表与仓里那份登记表 {registry_cases} 种、"
          f"阶段参数 --check 与 libtest 参数 --list 的读法 {argument_cases} 种、"
          f"跑不跑编译出来的代码 {len(compiled_cases)} 种）")
    return 0


if __name__ == "__main__":
    if sys.argv[1:] == ["--selftest"]:
        sys.exit(selftest())
    if os.environ.get(BREAK_VARIABLE):   # 弄坏开关只在自证里认：命令行不是单独一个 --selftest 而变量非空，一切之前就拒，不管值是什么
        print(f"  ✗ {BREAK_VARIABLE}={os.environ[BREAK_VARIABLE]} 是弄坏开关，只在自证（lib_project_code_runs.py --selftest）里认，平常跑设了就拒，这一趟什么都没做", file=sys.stderr)
        print(f"  → 怎么办：去掉 {BREAK_VARIABLE} 再跑；要证红就跑 {BREAK_VARIABLE}=<项> python3 .claude/hooks/lib_project_code_runs.py --selftest", file=sys.stderr)
        sys.exit(2)
    print("用法：python3 .claude/hooks/lib_project_code_runs.py --selftest（判定入口按文件路径导入这份模块再调，见文件头）", file=sys.stderr)
    sys.exit(2)
