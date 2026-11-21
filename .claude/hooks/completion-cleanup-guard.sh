#!/usr/bin/env bash
# 完工清理闸（Claude Code 钩子）：主 agent 结束一轮、子 agent 交回与收工时，自己认领、自己建的仓副本、编译目录与运行目录里的库拷贝还在，
# 或自己的 Bash 命令点到过的迁库遗留（数据根、判定库服务状态目录与没有归属文件的工作根目录里的旧库与库备份）还在，
# 或主 agent 本会话合入后验证留下的快照（<工作根>/merge-and-verify/<一层>/repo）还在，
# 或主 agent 本会话交的任务已有结局、结局没取走或提交时设了 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1、而现场目录还在，就拦下，
# 列出路径与 du 的大小，要它删完再收工（用完就删：.claude/main-agent.md「一轮怎么开、怎么收」第 11 条，.claude/agent-common.md「写」一节）。
# hook-events: Stop SubagentStop PreToolUse:SubagentHandback
# gate-similar: handback-scratch-check.sh 上游的交回尾门禁，同挂 PreToolUse:SubagentHandback 与 SubagentStop、同样判自己建的编译目录与仓副本，但只看临时目录（/tmp、$TMPDIR）、按调用时间窗认归属、交回报告写一行「没删」就放行，也不管主 agent；这里看工作根里本会话认领的目录与崩溃放量的运行目录、按归属文件认、只认交付物声明与子 agent 交回正文里「交主 agent 删 <全路径>：<原因>」一行、不收「没删」说明、还挂主 agent 的 Stop；它在上游副本里，不许就地改，并不进去
# gate-similar: handback-guard.sh 同挂 PreToolUse:SubagentHandback，判交回正文、报告与自己起的后台任务还在跑；这里判磁盘上留下的副本。后台任务在跑的认法两边都从 scripts/agents/agent-watch.py 导入 own_background_tasks_still_running，不另写一份；判的对象与出路不同，并进去要让交回正文闸兼管归属文件与运行目录
# gate-similar: remind-rearm-stop-guard.sh 同挂 Stop，判循环提醒的登记有没有进程在等、没在等就交出重起命令；这里判磁盘上的副本，读的输入（提醒登记目录对归属文件、会话记录与运行目录）与放行条件都不同
# gate-similar: gate-reuse-check.sh 上游的 Stop / SubagentStop 收工自检，判这一轮新写的门禁与钩子写没写 gate-similar，本仓没注册；对象不同
#
# 什么时候判：
#   Stop（主 agent）：手里没有在跑的后台任务与子 agent 时才判；有就放行、什么都不打。
#     在跑的后台任务：会话记录里起过的后台任务此刻还有进程开着它的输出文件（从 scripts/agents/agent-watch.py 导入 own_background_tasks_still_running，与交回闸 ④ 同一份认法）；
#       开着输出文件的最上层进程，命令（去掉 Bash 工具的外包装之后）跑的全是循环提醒 remind.sh 或看门狗 watch.sh、agent-watch.py 的，不算在跑的活。
#     在跑的子 agent：主会话目录 subagents/ 下的会话记录，最后修改在 SUBAGENT_IDLE_SECONDS（30 分钟）以内，而主会话记录里没有它最后一次写之后（放 NOTIFICATION_SLACK_SECONDS）
#       的任务通知或 TaskStop（从 agent-watch.py 导入 session_task_events）。30 分钟没写过的不算在跑。
#   SubagentStop 与交回工具 SubagentHandback 的 PreToolUse（子 agent）：它自己起的后台任务还有进程开着输出文件时放行（那一条归交回闸 ④），别的时候都判。
#
# 判哪几处：
#   主 agent：工作根（.claude/hooks/lib_work_root.py）下一层、归属文件 .owner 的 session 是本会话的每个目录（scripts/hosts/work-directory-ownership.sh 写的那份；
#     子 agent 认领的会话号与主 agent 相同，也归本会话），加本会话的临时草稿目录 <会话临时目录根>/<项目目录名>/<会话号>/（会话临时目录根 /tmp/claude-<uid> 取 lib_work_root.py 的 claude_session_temporary_root；下一层的 tasks/ 不看）。
#   子 agent：它自己的 Bash 调用里 work-directory-ownership.sh claim 认领的目录（归属文件要是本会话的），与派发提示里「草稿目录：」点名的那一个
#     （只认行首那一行，句中与列表里转述的不算；认法在同目录 lib_prompt_scratch_line.py，与派发闸 ⑯ ⑰、交回闸 ⑥ 共用；写相对路径的按仓根解，
#     用那份库的 repository_root，不按钩子此刻的当前目录）
#     （在工作根下；它所在的那个工作根下一层目录有归属文件的，要是本会话的）。主 agent 的别的目录、兄弟子 agent 的目录不判。
#   服务给的编译目录（TASK_SERVER_BUILD_DIRECTORY 下按树分的组）谁都不判：它归树、不归任务。树是任务 --cwd 解开符号链接后往上第一层有 .git 的那一层
#     （哪一层都没有就是解开后的 --cwd 本身），树删了服务跟着删它那一整组（task-server/README.md「编译目录」、task-server/task_server_lib.py 文件头「编译目录」）；
#     收工时要删的是认领目录与临时草稿目录里自己建的仓副本与编译目录，归上面那几处判。
#   主 agent 另判任务现场（task-server/task_server_lib.py 文件头「现场」：每一次尝试的现场目录 <工作根>/task-server-scenes/<尝试标签> 记在尝试记录的 scene_directory；
#     提交它的会话 task wait 取走结局之后过一小时由服务删，task clean <号> 当场删，提交时设了 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1 的只由 task clean 删）：
#     服务库只读打开（路径取 task-server/task_server_lib.py 的 read_configuration().database，读不了配置取 default_values 的 TASK_SERVER_STATE_DIRECTORY 下 DATABASE_NAME；
#     sqlite 只读模式；tasks 按会话列取，attempts 按任务号接 tasks、用任务表的会话判：服务写尝试行时会话列是空串，task-server/task_server_scheduler_starting.py 起尝试那一处），
#     本会话的任务（分片的子任务按父任务）已有结局、尝试记的现场目录还在盘上，又占着服务两个不到期条件之一的（与 task-server/task_server_scheduler_building.py 的 scene_due 同一口径，两个都判），
#     按「任务现场」列进单子：
#       只是结局没取走（父任务记录的 collected 不为真）：删法 task-server/task wait <号>（取走结局，服务过一小时自己删）；
#       设了留现场（父任务记录的 environment 里 SINGLEFS_KEEP_TEST_TEMPORARY_FILES 是 1）、结局已取走：删法 task-server/task clean <号>（取走了也只由 task clean 删）；
#       两样都占：删法「先 task-server/task wait <号>，再 task-server/task clean <号>」。task clean 只删现场、不算取走结局，结局没取走的照旧算没人取，
#       服务之后每次收这个会话的任务都打严厉警告（task-server/task_server.py 文件头「提交时警告」②），所以结局没取走的出路都先写 wait。
#     分片的写父任务号（task clean 连子任务的现场一起删）；不直接 rm，还要读的读完再删。
#     在排、在跑的不列；结局已取走、没设留现场的不列（服务过一小时自己删）。子 agent 不判这一样（子 agent 不往统一任务服务交任务、不起 task clean）。
#     服务库不在（服务没装过）不判；载不进 task_server_lib.py、算不出库的位置、库读不了的记一处没判全。
#   主 agent 另判合入后验证留下的快照（scripts/gate/merge-and-verify.sh 带 MERGE_AND_VERIFY_KEEP_COPY=1 留下的 <工作根>/merge-and-verify/<一层>/repo；
#     那一趟在 <一层>/.owner 写三行归属记录，写法同 scripts/hosts/work-directory-ownership.sh 的归属文件）：
#     <工作根>/merge-and-verify/ 下一层、.owner 的 session 是本会话的（三行写全才认，与上面认领目录同一个 owner_session），同层 repo/ 还在（是目录、不是符号链接）的，
#     按「合入后验证快照」列 repo 的路径与 du 的大小，删法 rm -rf -- <repo>（只删 repo/，日志与 .owner 留着）；
#     repo/ 已删的、别的会话的、没有 .owner（或写得不对）的一层不列，子 agent 不判（子 agent 不跑合入后验证）。
#     那一层里有进程开着（那一趟还在跑，或有进程的工作目录、打开的文件落在这一层里）的整层不列。
#     要留的（验证段后面还要在它上面验、修补要以它为基线建副本的）在那一层的 .deliverables 写一行「repo 基线 <为什么留>」（下面「要留的交付物」）。
#     <工作根>/merge-and-verify/ 本身在而列不出时记一处没判全。
#   两者都另判崩溃放量的运行目录：$SINGLEFS_CRASH_AMPLIFICATION_HOME/runs/ 下一层（没设时是工作根下的 crash-amplification/runs/ 下一层，
#     与 scripts/crash/crash-amplification-lib.sh 的 crash_amplification_home_directory 同一取法），
#     只判认得出确凿归属的；创建时间与 Bash 调用的时间窗重叠一条不算。确凿归属这样认：
#     起它的进程：运行目录名里的「<时刻>-<进程号>」（入口脚本拿自己的 $$ 起名，建目录的是它或它的子进程）那个进程此刻还在、不是僵尸、
#       开跑时刻（/proc/<pid>/stat 的 starttime）不晚于目录的创建时间（statx 的 btime）加 CREATOR_START_SLACK_SECONDS（晚于的是进程号被重用了）。
#       它还在就是那一趟还在跑：整个运行目录不列。它环境里的会话号（依次取 SESSION_VARIABLES，同 scripts/hosts/work-directory-ownership.sh）是归属；
#       顺着父进程往上找父进程是 Claude 进程（它环境里的 CLAUDE_PID）的那一层，就是起它的那次 Bash 调用的 shell，取它 -c 里 eval 的那条命令，记下 sha256。
#       看到一次就排他记账：<工作根>/completion-cleanup-guard/run-owners/<运行目录名>-<路径摘要>.json（每个会话的这道闸都记看到的全部运行目录，先看到的记，记了不改；
#       运行目录不在了的那几份顺手删）。
#     那个进程不在了：照记账认；运行目录里有归属记录 .owner（三行，写法同 work-directory-ownership.sh 的归属文件，入口脚本建运行目录时写）的，会话号以它为准。
#     归属是本会话的：主 agent 算自己的；子 agent 另要记账里那条命令的 sha256 等于它自己会话记录里某条 Bash 命令的（只有归属记录、没有记账的归主 agent）。
#     认不出的（进程不在、没记账、没归属记录）不列；其中创建时间落在它自己某次 Bash 调用的时间窗里（发起到结果，后台跑的到完成通知，没有的到现在，
#       两头各放 WINDOW_SLACK_SECONDS；那条命令点了 RUN_COMMAND_KEYWORDS 之一：崩溃放量、崩溃点重放、广谱变异、门禁）、里面又留了副本的，
#       拦下时在单子后面另起一节「认不出是谁的」列出来，只报不拦；不拦的时候不打。
#   两者都另判迁库遗留（迁库、族翻列之后留下的旧库与备份）：
#     数据根（lib_work_root.py 的 shared_data_root；取不到记一处没判全）下 LEFTOVER_SCAN_MAX_DEPTH 层，工作根与运行目录根不进；
#     崩溃放量的家目录（同上 crash_amplification_home_directory 的取法）不在数据根与工作根下时另看，同样 LEFTOVER_SCAN_MAX_DEPTH 层、运行目录根不进；
#     判定库服务的状态目录 PRUNE_STATE_SCAN_MAX_DEPTH 层：取法照 prune-server/control（载入它，不另写）：装好的单元文件 ExecStart 的 --state-directory（unit_library），
#       与它的缺省状态目录（Paths().state_directory）不同时两处都看；单元文件不在取缺省；单元文件在而读不了、control 载不进记一处没判全。
#       服务自己那份库（unit_library 交的那一份，读不到单元文件时是缺省状态目录下的 library）不列、不进。
#     工作根下一层没写归属文件（.owner 不在或写得不对）的每个目录，LEFTOVER_SCAN_MAX_DEPTH 层（从工作根算）；有归属文件的照上面的认领规则，这里不进。
#     上面认领目录、草稿目录与运行目录判过的那几处不重判。数据根表里登记的在用项（lib_work_root.py 的 SHARED_DATA_ENTRIES 与 PER_HOST_DATA_ENTRIES，
#       数据根下一层那一个）本身不按名字认成旧文件库，里面照看。
#     归属按点名认：路径本身或它的上层目录（到锚点为止，锚点本身不算）出现在自己会话记录里某条 Bash 命令的文字里，就是自己的
#       （主 agent 看主会话记录，子 agent 看它自己的；读法与上面认领、运行目录同一份 session-transcript.py 的 tool_calls；
#       创建时间与调用时间窗重叠不算）。只读命令不算点到：命令按 |、||、&&、;、&、括号与换行切成段，
#       一段的命令词（跳过起头的 NAME=值，取最后一节文件名）在 READ_ONLY_COMMANDS 里的（ls、du、stat、find、cat、head、tail、less、grep、rg、wc、
#       sha256sum、md5sum、diff、cmp、file、readlink、realpath、tree、df；find 带 FIND_MODIFYING_PREDICATES 之一（-delete、-exec、-execdir）的不算只读），
#       那一段里的路径不算点到；mv、cp、tar、rsync、ln、mkdir、rm 这类改动命令，与把路径当参数交给脚本或程序的，照算。命令拆成词（shlex，拆不开照空白与 shell 符号切），
#       ~、$HOME、$SINGLEFS_WORK_ROOT 起头的照 lib_work_root.py 的 expand_written_path 展开，同一条命令里 NAME=<路径> 赋过的 $NAME 换成它，
#       $(python3 …/lib_work_root.py [data-root | shared <名>]) 换成它打的那一处；相对路径照同一条命令里前面 cd 到的目录解，没 cd 过的不认；
#       --x=<路径> 取等号后面；带 * ? [ 的按层通配。锚点：家目录、数据根、工作根、崩溃放量的家目录、运行目录根、判定库服务的状态目录与它们的上层；
#       只点到锚点（mkdir -p <数据根>、rsync -a <数据根>/* … 这种字面前缀是锚点的通配）不认作点到下面的东西。
#     自己的拦；认不出的（没点到）不拦，拦下时列进「认不出是谁的」一节（最多 UNKNOWN_SHOWN_MAX 处，其余报个数）。在用的不列（下面「判什么」的 open_holders）。
#     这几处没有交付物声明：.deliverables 只认在它所在那个认领目录之内的路径（上面「要留的交付物」），数据根层与没有归属文件的目录里的旧库靠声明放不过。
# 判什么（逐个目录往下找，认出一样就不再往里找；认领目录与草稿目录最深 SCAN_MAX_DEPTH 层，运行目录 RUN_SCAN_MAX_DEPTH 层；不跟符号链接；仓根本身与它的上层不判）：
#   编译目录：带签名对得上的 CACHEDIR.TAG，或带 .rustc_info.json（cargo 的 target 与 CARGO_TARGET_DIR 指的目录都带）；
#   仓副本：有 .git（目录；.git 是文件的报成工作树），或同时有 Cargo.toml 与 crates（这两样的认法从 agent-watch.py 导入 scratch_hog_kind）；
#   库拷贝：同时有 CURRENT 与 MANIFEST-*（RocksDB 的整份拷贝：判定库只在 prune-server 里一份，草稿与运行目录里不该有本地的库拷贝）。
#   旧文件库：目录名里带判定库的种类名（整个名字，或前后是 . - _ 与名字两头），里面不空；起头那一处（认领目录、草稿目录、扫的根）本身不按名字认。
#     种类名现读两处，不在这里另抄：scripts/prune-server/prune_server_client.py 文件头「键空间」一节里「例 / 种类固定 / 种类是」后面的名字，
#     .claude/gate.d/verdict-library-scripts.tsv 第三列的「键空间 <族>/<种类>/」与「<族> 族 <种类>/」。
#     哪一处读不了或一个名字都读不出，记一处没判全（旧文件库与带种类名的库备份就认不全）。
#   库备份：文件名以 .tar、.tar.gz、.tgz、.tar.zst 结尾，名字里（不分大小写）带 verdict、library 或上面的种类名之一的普通文件；不跟符号链接。
#   旧文件库与库备份在认领目录、草稿目录与上面迁库遗留那几处都认；迁库遗留那几处只认库拷贝、旧文件库与库备份三样，编译目录与仓副本不归这一类。
#   在用的不列，判法调 scripts/hosts/run_build_directory.py 的 open_holders（与它的 holders 同一份：本进程以外的进程的工作目录、根目录、可执行文件、
#     打开的文件描述符或内存映射落在里面；只看得到本用户的；库备份是文件，看有没有进程的文件描述符或内存映射指着它这一份）。认领目录与草稿目录看列出的那一处（含它底下各层）；运行目录看整个运行目录：
#     那一趟的日志或别的子目录有进程开着，就是那一趟还在跑，里面的库拷贝与编译目录都不列。
#   换上流程的草稿树：路径是 <认领目录或草稿目录>/hook-swap/tree 这一形（scripts/gate/hook-swap.py 在 --scratch 点的目录下建，那一趟自己跑完或出错就删；
#     它在建树、编译、喂钩子的间隙里没有进程的工作目录或文件描述符落在树里）的行，决定拦之前（后台任务与子 agent 判过之后）最后再看一眼，下面两种不列：
#     有进程在用：in_use 的判法（open_holders）看得到的，或有进程的命令行点到这棵树、点到它的草稿目录本身（hook-swap.py 那一趟命令行里 --scratch 点的是草稿目录；
#       点到草稿目录底下别的文件不算）；
#     这一刻已经没了（那一趟刚删掉树，单子里会打成 du 量不出的「?」）。
#     没有进程在用、还在的照列，删法照旧 rm -rf -- <树>。
# 要留的交付物：报告、补丁与打补丁前的备份是普通文件与目录的，本来就不在单上。要留的副本写进它所在那个认领目录的 .deliverables
#   （与归属文件同一层；子 agent 的草稿目录本身也认；合入后验证的那一层也认，快照写 repo），一行一条：
#     <相对那个目录的路径>  <种类>  <为什么留>      种类五选一：报告、补丁、备份、交接、基线；为什么留至少 4 个字；# 起头的行是注释
#   路径带空格的，整个路径用反引号、单引号或双引号包着：先按引号切出路径，再按空白切种类与理由。.deliverables 本身要是普通文件，符号链接不读、判写法不合并点名。
#   只读这几层的 .deliverables：主 agent 是本会话认领的目录、临时草稿目录与合入后验证的每一层；子 agent 是它自己认领的目录与派发提示点名的草稿目录，
#     加上它们所在的工作根下一层（只为读那一层里免列的声明）。写在这几层底下更深处（例：副本目录里）的 .deliverables 不读，里面的声明不算、写坏的行也不拦。
#     例外：子 agent 交回、收工时，它认领的目录与草稿目录底下的下层目录（嵌套派发里下层子 agent 的草稿目录；不是副本，副本里面的不进）里的 .deliverables 也读，
#     只取声明：下层声明了的副本（路径相对那一层，种类取这个角色免列的）算已声明，那一份里写坏的行不判、不拦这一次交回；主 agent 不读下层的。
#     上层子 agent 被拦时，单子上落在下层目录里的副本逐个点名下层的声明为什么没认上：从认领目录或草稿目录往下到它的那几层里，.deliverables 是符号链接（写出指向哪）、
#     不是普通文件、读不了、哪一行写坏（文件:行号与不合在哪），或这几层都不在（没写，或写了又被删了）；都好好的就点名它们里没有声明这一份的行。
#   交接：子 agent 交给主 agent 接着用的副本（主 agent 照它的报告在里面跑命令）。子 agent 交回、收工时不列；主 agent 结束一轮时照列，用完就删。
#   基线：主 agent 自己留着、后面要在它上面跑命令或以它为基线建副本的树（写段下一批的段树、合入后验证留下的快照）。只认主 agent 的声明：
#     主 agent 结束一轮时不列，用完就删；子 agent 交回、收工时写了也照列（要交给主 agent 的用交接）；编译目录不能声明成基线。
#   声明了的那一棵里仓副本、库拷贝、旧文件库与库备份不列，编译目录照列（编译目录不能声明成交付物）；路径落到那个目录外、种类不认得、理由太短的行不算，列进拦下的说明。
# 删不掉的（只子 agent，只编译目录、工作树与仓副本三种，只认认领目录与草稿目录里单子上列出的那几处）：子 agent 在交回正文里逐个写一行
#   「交主 agent 删 <全路径>：<删不掉的原因，至少 HAND_TO_MAIN_REASON_MINIMUM 个字>」（写法见 HAND_TO_MAIN_LINE：行首可带列表符号，路径可用反引号或引号包着；
#   路径要与单子上列的一字不差，上层目录不替下层）的，交回、收工时不列。交回工具那一次看这一次交回的 message；SubagentStop 看已送达的交回正文与收工前的回复；
#   被交回闸拒掉（is_error）的那次交回正文不算。库拷贝、旧文件库、库备份与迁库遗留不认这种写法；主 agent 结束一轮时照旧列这些副本，由主 agent 删。
# 交回时（PreToolUse）这个子 agent 自己写的那几层（它认领的目录、派发提示点名的草稿目录；它们所在的工作根下一层是别人写的，那一层的坏行不拦这一次交回）
#   的 .deliverables 里有写法不合的行（读不了、是符号链接、不是三段、种类不在五种里、理由太短、路径落在目录外）：不看有没有副本留着，
#   当场拒，说明逐行点名文件、行号与不合在哪，写出五种种类、理由的下限与一行例子；改对再交回。Stop 与 SubagentStop 不因它拦（拦下的单子里照旧带「!」那几行）。
# 拦下：Stop 与 SubagentStop 打 {"decision": "block", "reason": …}；PreToolUse 退 2、说明写 stderr。说明逐个列种类、路径与 du -sh 的大小
#   （du 合起来最多 DU_TIMEOUT_SECONDS 秒，量不出写「?」）和删法。
# 拦过一次、续跑回来（stop_hook_active 为真）还是同一张单（一个都没删）：放行，把单子作 systemMessage 交给用户处置（免得卡死）；单子变了照拦。PreToolUse 每次都拦。
#   拦下的单记在 <工作根>/completion-cleanup-guard/<会话号>-<main 或子 agent 号>.json。
# 判不了（输入不是 JSON 对象、认不出会话号、找不到会话记录、会话记录在而读不了、导入不了共用模块、自己出错）：照「放行但必记检出」放行，stderr 写一句「没判」（输入读不出的带输入的前 200 字节），
#   检出记录（AGENT_HOOK_DETECTIONS，没设取 lib_work_root.py 的 default_detections_path）追加一行、结局「只记」，findings 以「完工清理闸没判、照常放行」起头，
#   看门狗据此报给主 agent（report_unjudged）。不归这道闸判的事件与工具（别的 PreToolUse、主 agent 的交回）照旧什么都不打。
# 判了而没判全、又没有残留的（扫到 SCAN_MAX_DIRECTORIES 个目录的上限、几处没看完；读不了的目录没看；主 agent 的工作根在而列不出；崩溃放量的运行目录取不到创建时间、没判；
#   迁库遗留那几处另算一份上限、扫到上限或读不了的另起一句；种类名读不出；数据根取不到；prune-server/control 载不进；判定库服务的单元文件在而读不了）：照「放行但必记检出」放行，
#   stderr 逐条打出没判全的几处与出路（「看了 N 个目录到顶，这几处没看完：…」），检出记录（AGENT_HOOK_DETECTIONS，没设取 lib_work_root.py 的 default_detections_path）
#   追加一行、结局「只记」，看门狗据此报给主 agent（report_unjudged_pass）；判全了的放行照旧什么都不打。拦下时这几处照旧写在单子后面。
#   上限可由 COMPLETION_CLEANUP_SCAN_MAX_DIRECTORIES（正整数）调小，只给自证造截断用，平常不设。
# 认不出的（照样要删）：没认领、也不是派发提示点名的草稿目录里建的；起它的进程这道闸一次都没看到还在跑、入口脚本又没写归属记录的运行目录
#   （从开跑到跑完之间没有哪个会话收过工、交过回；只在「认不出是谁的」里报）；第二台上的副本；
#   迁库遗留里自己的 Bash 命令一次都没点到（经脚本里拼出来的路径、命令替换里算出来的、cd 到别处再写相对路径的、只在只读命令那一段里出现的）的旧库与备份（只在「认不出是谁的」里报）；
#   第二台本地盘上的旧库与备份（判定库服务的状态目录与按台的那几项照那一台的家目录取，这道闸只在本机跑、只看本机的）；超过最深层数的。
# 退役判据：工作根与崩溃放量的运行目录不再放副本与库拷贝（驱动与脚本都改成每趟自己的目录、跑完自己删），连续四周这道闸一次都没拦；
#   迁库遗留一类另判：判定库不再迁库、不再翻族的布局（库格式锁定），数据根、判定库服务的状态目录与没有归属文件的工作根目录连续四周一份迁库遗留都没认出（「认不出是谁的」里也没有）；
#   任务现场一类另判：服务对结局没取走与设了 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1 的任务现场也到时自己删（task-server/task_server_lib.py 文件头「现场」不再交给 task wait 与 task clean）。
#
#   completion-cleanup-guard.sh              从 stdin 读钩子的 JSON
#   completion-cleanup-guard.sh --selftest   临时目录里造工作根、数据根、运行目录、会话记录、prune-server/control 的自证根（状态目录与单元文件）与占着文件的假进程，逐格核；没设弄坏开关时末尾逐个打开每个开关（并行），核它红在点名的格
# 弄坏开关（只在自证里认，平常跑设了就退 2；几个用逗号连）。逐条写开关名与值、弄坏哪一处，下一行是打开后红的那一格（「」里是失败行的开头）：
#   COMPLETION_CLEANUP_BREAK=build-directory-unrecognized  不认编译目录（CACHEDIR.TAG 与 .rustc_info.json）
#       「✗ 自证 main-build-directory-left：」
#       「✗ 自证 own-run-blocked-after-creator-exits：」
#       「✗ 自证 unknown-run-reported-not-blocked：」
#       「✗ 自证 build-directory-never-deliverable：」
#       「✗ 自证 reminder-only-still-judged：」
#       「✗ 自证 subagent-finished-still-judged：」
#       「✗ 自证 held-copy-not-listed：」
#       「✗ 自证 subagent-build-directory-left：」
#       「✗ 自证 unchanged-after-block-releases：」
#       「✗ 自证 changed-after-block-blocks-again：」
#       「✗ 自证 block-lists-size：」
#       「✗ 自证 subagent-hand-short-reason-blocks：」
#       「✗ 自证 subagent-hand-ancestor-blocks：」
#       「✗ 自证 main-hand-declaration-ignored：」
#   COMPLETION_CLEANUP_BREAK=repository-copy-unrecognized  不认仓副本（.git、Cargo.toml 加 crates）
#       「✗ 自证 main-repository-copy-left：」
#       「✗ 自证 main-handover-copy-left：」
#       「✗ 自证 subagent-assigned-directory-left：」
#       「✗ 自证 subagent-assigned-relative-by-repository-root：」
#       「✗ 自证 subagent-assigned-shared-reading：」
#       「✗ 自证 main-scratchpad-copy-left：」
#       「✗ 自证 swap-tree-idle-listed：」
#       「✗ 自证 swap-tree-gone-not-listed：」
#       「✗ 自证 subagent-hand-undeclared-blocks：」
#       「✗ 自证 subagent-hand-short-reason-blocks：」
#       「✗ 自证 subagent-hand-ancestor-blocks：」
#       「✗ 自证 main-hand-declaration-ignored：」
#       「✗ 自证 nested-lower-handover-main-listed：」
#       「✗ 自证 nested-undeclared-copy-blocks：」
#       「✗ 自证 nested-lower-broken-line-named：」
#       「✗ 自证 nested-lower-declaration-missing-named：」
#       「✗ 自证 nested-lower-declaration-symlink-named：」
#   COMPLETION_CLEANUP_BREAK=library-copy-unrecognized  不认库拷贝（CURRENT 加 MANIFEST-*）
#       「✗ 自证 main-run-library-copy-left：」
#       「✗ 自证 own-run-blocked-after-creator-exits：」
#       「✗ 自证 unknown-run-reported-not-blocked：」
#       「✗ 自证 subagent-own-run-left：」
#       「✗ 自证 run-root-default-under-work-root：」
#       「✗ 自证 prune-state-library-copy-blocked：」
#       「✗ 自证 unit-file-state-directory-read：」
#       「✗ 自证 relative-after-cd-attributed：」
#       「✗ 自证 no-owner-mentioned-copy-blocked：」
#       「✗ 自证 no-owner-unmentioned-reported-not-blocked：」
#       「✗ 自证 subagent-hand-library-copy-blocks：」
#   COMPLETION_CLEANUP_BREAK=all-sessions  归属是别的会话的目录也当本会话的判
#       「✗ 自证 other-session-directory-ignored：」
#       「✗ 自证 other-session-leftover-ignored：」
#   COMPLETION_CLEANUP_BREAK=deliverables-ignored  不读 .deliverables
#       「✗ 自证 deliverables-only-passes：」
#       「✗ 自证 subagent-handover-passes：」
#       「✗ 自证 declared-archive-passes：」
#       「✗ 自证 merge-snapshot-declared-baseline-passes：」
#       「✗ 自证 segment-tree-declared-baseline-passes：」
#       「✗ 自证 quoted-declaration-with-space-accepted：」
#       「✗ 自证 nested-lower-handover-passes：」
#   COMPLETION_CLEANUP_BREAK=build-directory-declarable  声明成交付物的那一棵里编译目录也不列
#       「✗ 自证 build-directory-never-deliverable：」
#   COMPLETION_CLEANUP_BREAK=handover-exempt-for-main  主 agent 也不列声明成交接的副本
#       「✗ 自证 main-handover-copy-left：」
#   COMPLETION_CLEANUP_BREAK=background-ignored  不看后台任务在不在跑
#       「✗ 自证 background-running-passes：」
#   COMPLETION_CLEANUP_BREAK=reminder-counts-as-running  循环提醒的后台任务也算在跑的活
#       「✗ 自证 reminder-only-still-judged：」
#   COMPLETION_CLEANUP_BREAK=subagents-ignored  不看子 agent 在不在跑
#       「✗ 自证 subagent-running-passes：」
#   COMPLETION_CLEANUP_BREAK=notifications-ignored  不看任务通知，写过不到 30 分钟的子 agent 都当在跑
#       「✗ 自证 subagent-finished-still-judged：」
#   COMPLETION_CLEANUP_BREAK=open-files-ignored  不看 /proc，有进程在用的也列
#       「✗ 自证 held-copy-not-listed：」
#       「✗ 自证 run-held-deep-not-listed：」
#       「✗ 自证 mapped-copy-not-listed：」
#       「✗ 自证 held-leftover-not-listed：」
#       「✗ 自证 merge-snapshot-held-not-listed：」
#   COMPLETION_CLEANUP_BREAK=subagent-sees-whole-session  子 agent 也判本会话认领的全部目录
#       「✗ 自证 subagent-ignores-siblings：」
#       「✗ 自证 subagent-assigned-relative-not-by-cwd：」
#       「✗ 自证 subagent-assigned-relay-ignored：」
#       「✗ 自证 handback-parent-declaration-passes：」
#   COMPLETION_CLEANUP_BREAK=prompt-scratch-ignored  不认派发提示里「草稿目录：」点名的那一个
#       「✗ 自证 subagent-assigned-directory-left：」
#       「✗ 自证 subagent-assigned-relative-by-repository-root：」
#       「✗ 自证 subagent-assigned-shared-reading：」
#       「✗ 自证 handback-parent-declaration-passes：」
#       「✗ 自证 nested-undeclared-copy-blocks：」
#       「✗ 自证 nested-lower-broken-line-named：」
#       「✗ 自证 nested-lower-declaration-missing-named：」
#       「✗ 自证 nested-lower-declaration-symlink-named：」
#   COMPLETION_CLEANUP_BREAK=prompt-scratch-relative-by-cwd  「草稿目录：」写相对路径的按钩子此刻的当前目录解，不按仓根
#       「✗ 自证 subagent-assigned-relative-by-repository-root：」
#       「✗ 自证 subagent-assigned-relative-not-by-cwd：」
#   COMPLETION_CLEANUP_BREAK=run-windows-ignored  运行目录不认归属，全算自己的
#       「✗ 自证 foreign-run-directory-ignored：」
#       「✗ 自证 run-in-progress-not-listed：」
#       「✗ 自证 foreign-session-run-in-window-not-listed：」
#       「✗ 自证 unknown-run-reported-not-blocked：」
#       「✗ 自证 subagent-ignores-sibling-runs：」
#       「✗ 自证 birth-unknown-pass-reported：」
#   COMPLETION_CLEANUP_BREAK=attribution-by-window  运行目录退回按时间窗认归属（创建时间落在自己 Bash 调用的时间窗里就算自己的）
#       「✗ 自证 foreign-session-run-in-window-not-listed：」
#       「✗ 自证 main-run-library-copy-left：」
#       「✗ 自证 own-run-blocked-after-creator-exits：」
#       「✗ 自证 unknown-run-reported-not-blocked：」
#       「✗ 自证 subagent-ignores-sibling-runs：」
#       「✗ 自证 run-root-default-under-work-root：」
#   COMPLETION_CLEANUP_BREAK=ledger-ignored  不记、不读起它的进程的记账
#       「✗ 自证 own-run-blocked-after-creator-exits：」
#       「✗ 自证 foreign-session-run-in-window-not-listed：」
#       「✗ 自证 subagent-own-run-left：」
#   COMPLETION_CLEANUP_BREAK=creator-alive-ignored  起它的进程还在也照判
#       「✗ 自证 run-in-progress-not-listed：」
#   COMPLETION_CLEANUP_BREAK=subagent-run-session-level  子 agent 把本会话别的 Bash 调用起的运行目录也算自己的
#       「✗ 自证 subagent-ignores-sibling-runs：」
#   COMPLETION_CLEANUP_BREAK=run-holders-leaf-only  运行目录里只看列出的那一处有没有进程开着，不看整个运行目录
#       「✗ 自证 run-held-deep-not-listed：」
#   COMPLETION_CLEANUP_BREAK=release-never  续跑回来还是同一张单也照拦
#       「✗ 自证 unchanged-after-block-releases：」
#   COMPLETION_CLEANUP_BREAK=release-on-any-continuation  续跑回来单子变了也放行
#       「✗ 自证 changed-after-block-blocks-again：」
#   COMPLETION_CLEANUP_BREAK=size-omitted  单上不写 du 的大小
#       「✗ 自证 block-lists-size：」
#       「✗ 自证 prune-state-library-copy-blocked：」
#   COMPLETION_CLEANUP_BREAK=scratchpad-ignored  主 agent 不判本会话的临时草稿目录
#       「✗ 自证 main-scratchpad-copy-left：」
#   COMPLETION_CLEANUP_BREAK=unjudged-pass-silent  没判全的放行不打、不记检出（没扫完的几处只进拦下时的单子）
#       「✗ 自证 truncated-scan-pass-reported：」
#       「✗ 自证 birth-unknown-pass-reported：」
#       「✗ 自证 unreadable-directory-pass-reported：」
#       「✗ 自证 work-root-unlisted-pass-reported：」
#   COMPLETION_CLEANUP_BREAK=unjudged-skip-silent  判不了的放行只在 stderr 写那一句、不记检出
#       「✗ 自证 unjudged-skip-recorded：」
#       「✗ 自证 self-error-pass-recorded：」
#   COMPLETION_CLEANUP_BREAK=unreadable-directory-unnoted  扫的时候读不了的目录静默跳过，不进没判全的几处
#       「✗ 自证 unreadable-directory-pass-reported：」
#   COMPLETION_CLEANUP_BREAK=work-root-unlisted-unnoted  工作根在而列不出时当作本会话没认领任何目录，不进没判全的几处
#       「✗ 自证 work-root-unlisted-pass-reported：」
#   COMPLETION_CLEANUP_BREAK=transcript-unreadable-unnoted  会话记录在而读不了时不当判不了，照常判（读它的共用模块交空表，什么都认不出、放行）
#       「✗ 自证 transcript-unreadable-skip-reported：」
#   COMPLETION_CLEANUP_BREAK=run-root-default-unseen  没设 SINGLEFS_CRASH_AMPLIFICATION_HOME 时一个运行目录根都不看（工作根下的 crash-amplification/runs 不判）
#       「✗ 自证 run-root-default-under-work-root：」
#   COMPLETION_CLEANUP_BREAK=guard-off  平常跑设了弄坏开关也不拒；它是关守卫项，只在它是唯一一项时认，与别的项同写照拒并点名它
#       「✗ 自证 break-switch-refused-outside-selftest：」
#   COMPLETION_CLEANUP_BREAK=guard-off-with-others  关守卫项与别的项同写也照认（guard-off 写在哪一项都认；只在自证进程本身里调守卫时认，平常跑同写照拒）
#       「✗ 自证 guard-off-combined-refused-in-process：」
#   COMPLETION_CLEANUP_BREAK=data-root-leftovers-unseen  迁库遗留不看数据根与崩溃放量的家目录（数据根下的旧库与备份不在射程里）
#       「✗ 自证 data-root-family-library-blocked：」
#       「✗ 自证 relative-after-cd-attributed：」
#       「✗ 自证 library-archive-blocked：」
#       「✗ 自证 species-read-from-registries：」
#       「✗ 自证 data-root-unconfigured-pass-reported：」
#       「✗ 自证 data-root-leftover-not-declarable：」
#       「✗ 自证 subagent-leftover-by-own-commands：」
#       「✗ 自证 read-only-mentions-not-attributed：」
#   COMPLETION_CLEANUP_BREAK=prune-state-directory-unseen  迁库遗留不看判定库服务的状态目录
#       「✗ 自证 prune-state-library-copy-blocked：」
#       「✗ 自证 unit-file-state-directory-read：」
#   COMPLETION_CLEANUP_BREAK=service-library-listed  服务自己那份库也照迁库遗留判
#       「✗ 自证 service-library-not-listed：」
#       「✗ 自证 unit-file-state-directory-read：」
#   COMPLETION_CLEANUP_BREAK=unit-file-ignored  状态目录不读单元文件的 --state-directory，只取 control 的缺省状态目录
#       「✗ 自证 unit-file-state-directory-read：」
#   COMPLETION_CLEANUP_BREAK=old-library-unrecognized  不按种类名认旧文件库
#       「✗ 自证 data-root-family-library-blocked：」
#       「✗ 自证 species-read-from-registries：」
#       「✗ 自证 data-root-leftover-not-declarable：」
#       「✗ 自证 subagent-leftover-by-own-commands：」
#       「✗ 自证 read-only-mentions-not-attributed：」
#   COMPLETION_CLEANUP_BREAK=leftover-unjudged-unnoted  种类名读不出、数据根取不到、control 载不进、单元文件读不了时不记没判全
#       「✗ 自证 species-unreadable-pass-reported：」
#       「✗ 自证 data-root-unconfigured-pass-reported：」
#   COMPLETION_CLEANUP_BREAK=no-owner-directories-skipped  工作根下没写归属文件的目录不判
#       「✗ 自证 no-owner-mentioned-copy-blocked：」
#       「✗ 自证 no-owner-unmentioned-reported-not-blocked：」
#   COMPLETION_CLEANUP_BREAK=owned-directories-leftover-scanned  工作根下有归属文件的目录（别的会话的也算）也按迁库遗留判
#       「✗ 自证 other-session-leftover-ignored：」
#   COMPLETION_CLEANUP_BREAK=registered-entries-listed  数据根表里登记的在用项名字对得上种类名的也认成旧文件库
#       「✗ 自证 registered-entry-not-listed：」
#   COMPLETION_CLEANUP_BREAK=mention-ancestors-ignored  归属只认路径本身被点到，上层目录被点到不算
#       「✗ 自证 data-root-family-library-blocked：」
#       「✗ 自证 subagent-leftover-by-own-commands：」
#   COMPLETION_CLEANUP_BREAK=cd-ignored  相对路径不照同一条命令里前面 cd 到的目录解
#       「✗ 自证 relative-after-cd-attributed：」
#   COMPLETION_CLEANUP_BREAK=anchor-mentions-counted  只点到锚点（数据根本身、字面前缀是锚点的通配）也算点到下面的东西
#       「✗ 自证 anchor-mentions-not-attributed：」
#   COMPLETION_CLEANUP_BREAK=unmentioned-counted-own  迁库遗留不认归属，没点到的也算本会话的
#       「✗ 自证 no-owner-unmentioned-reported-not-blocked：」
#       「✗ 自证 anchor-mentions-not-attributed：」
#       「✗ 自证 subagent-leftover-by-own-commands：」
#       「✗ 自证 read-only-mentions-not-attributed：」
#   COMPLETION_CLEANUP_BREAK=read-only-mentions-counted  只读命令（READ_ONLY_COMMANDS）那一段里的路径也算点到（ls、du 读一眼也算本会话的）
#       「✗ 自证 read-only-mentions-not-attributed：」
#   COMPLETION_CLEANUP_BREAK=leftover-mentions-session-level  子 agent 把主会话记录里的命令点到的也算自己的
#       「✗ 自证 subagent-leftover-by-own-commands：」
#   COMPLETION_CLEANUP_BREAK=archives-unrecognized  不认库备份压缩包
#       「✗ 自证 library-archive-blocked：」
#   COMPLETION_CLEANUP_BREAK=archive-any-name  压缩包不看名字都认成库备份
#       「✗ 自证 library-archive-blocked：」
#   COMPLETION_CLEANUP_BREAK=declaration-outside-accepted  .deliverables 里落到认领目录外的路径也照收（数据根层的旧库靠声明放得过）
#       「✗ 自证 data-root-leftover-not-declarable：」
#   COMPLETION_CLEANUP_BREAK=hand-declaration-ignored  不读交回正文与收工前回复里的「交主 agent 删」行
#       「✗ 自证 subagent-hand-to-main-passes：」
#       「✗ 自证 subagent-hand-undeclared-blocks：」
#       「✗ 自证 subagent-hand-stop-delivered-passes：」
#       「✗ 自证 subagent-hand-stop-reply-passes：」
#   COMPLETION_CLEANUP_BREAK=hand-reason-unchecked  原因不到 HAND_TO_MAIN_REASON_MINIMUM 个字的「交主 agent 删」行也认
#       「✗ 自证 subagent-hand-short-reason-blocks：」
#   COMPLETION_CLEANUP_BREAK=hand-ancestor-accepted  上层目录写了「交主 agent 删」就连它底下的副本一起不列
#       「✗ 自证 subagent-hand-ancestor-blocks：」
#   COMPLETION_CLEANUP_BREAK=hand-all-kinds  库拷贝、旧文件库与库备份也认「交主 agent 删」行
#       「✗ 自证 subagent-hand-library-copy-blocks：」
#   COMPLETION_CLEANUP_BREAK=hand-exempt-for-main  主 agent 结束一轮时也认自己会话里写的「交主 agent 删」行
#       「✗ 自证 main-hand-declaration-ignored：」
#   COMPLETION_CLEANUP_BREAK=hand-way-unlisted  拦下的单子里不写「交主 agent 删」这一条出路
#       「✗ 自证 subagent-block-names-hand-way：」
#       「✗ 自证 subagent-hand-way-passes-handback-gates：」
#   COMPLETION_CLEANUP_BREAK=hand-way-example-oversized  单子里的例子行拖到超过交回正文闸的字数上限
#       「✗ 自证 subagent-hand-way-passes-handback-gates：」
#   COMPLETION_CLEANUP_BREAK=handback-declaration-unchecked  交回（PreToolUse）时 .deliverables 里写法不合的行不拦
#       「✗ 自证 handback-bad-declaration-blocked：」
#       「✗ 自证 handback-parent-declaration-passes：」
#       「✗ 自证 handback-symlinked-declaration-blocked：」
#   COMPLETION_CLEANUP_BREAK=handback-parent-declaration-judged  交回（PreToolUse）时子 agent 草稿目录所在的工作根下一层里别人写坏的行也拦
#       「✗ 自证 handback-parent-declaration-passes：」
#   COMPLETION_CLEANUP_BREAK=declaration-quotes-unsplit  .deliverables 里带引号的路径不先按引号切、整行按空白切三段（带空格的路径判写法不合）
#       「✗ 自证 quoted-declaration-with-space-accepted：」
#   COMPLETION_CLEANUP_BREAK=declaration-symlink-skipped  符号链接的 .deliverables 整份跳过、不判写法不合
#       「✗ 自证 handback-symlinked-declaration-blocked：」
#   COMPLETION_CLEANUP_BREAK=kept-scenes-ignored  不看任务现场
#       「✗ 自证 main-kept-scene-left：」
#       「✗ 自证 kept-collected-scene-left：」
#       「✗ 自证 uncollected-scene-left：」
#   COMPLETION_CLEANUP_BREAK=kept-scene-removal-by-rm  任务现场的删法写成 rm -rf（照别的副本写），不写 task-server/task clean <号> 与 task wait <号>
#       「✗ 自证 main-kept-scene-left：」
#       「✗ 自证 kept-collected-scene-left：」
#       「✗ 自证 uncollected-scene-left：」
#   COMPLETION_CLEANUP_BREAK=kept-scenes-keep-unchecked  两个不到期条件都不看：提交时没设 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1、结局已取走的任务的现场也列
#       「✗ 自证 unkept-scene-not-listed：」
#   COMPLETION_CLEANUP_BREAK=kept-scenes-keep-variable-unchecked  列不列只看结局取没取走，不看设没设留现场：设了 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1、结局已取走的任务的现场不列
#       「✗ 自证 kept-collected-scene-left：」
#   COMPLETION_CLEANUP_BREAK=kept-scenes-uncollected-unchecked  不看结局取没取走：没设留现场、结局没取走的任务的现场不列，两样都占的删法只写 task-server/task clean <号>
#       「✗ 自证 main-kept-scene-left：」
#       「✗ 自证 uncollected-scene-left：」
#   COMPLETION_CLEANUP_BREAK=uncollected-scene-removal-by-clean  结局没取走、没设留现场的任务现场删法写成 task-server/task clean <号>，不写 task wait <号>
#       「✗ 自证 uncollected-scene-left：」
#   COMPLETION_CLEANUP_BREAK=kept-scene-removal-uncollected-first  设了留现场、结局也没取走的任务现场删法只写 task-server/task wait <号>（取走结局删不掉它），不接着写 task clean <号>
#       「✗ 自证 main-kept-scene-left：」
#   COMPLETION_CLEANUP_BREAK=kept-scene-removal-wait-skipped  设了留现场、结局也没取走的任务现场删法只写 task-server/task clean <号>，不先写 task wait <号>（结局照旧没人取）
#       「✗ 自证 main-kept-scene-left：」
#   COMPLETION_CLEANUP_BREAK=kept-scene-attempts-by-session-column  尝试按尝试表自己的会话列取（服务写的是空串），不按任务号接任务表
#       「✗ 自证 main-kept-scene-left：」
#       「✗ 自证 kept-collected-scene-left：」
#       「✗ 自证 uncollected-scene-left：」
#   COMPLETION_CLEANUP_BREAK=kept-scenes-session-unchecked  别的会话提交的任务的现场也列
#       「✗ 自证 other-session-kept-scene-ignored：」
#   COMPLETION_CLEANUP_BREAK=kept-scenes-unfinished-listed  在排、在跑的任务的现场也列
#       「✗ 自证 running-kept-scene-not-listed：」
#   COMPLETION_CLEANUP_BREAK=subagent-kept-scenes-judged  子 agent 交回、收工时也判本会话的任务现场（子 agent 不交任务、不起 task clean，列给它也删不了）
#       「✗ 自证 subagent-kept-scenes-not-listed：」
#   COMPLETION_CLEANUP_BREAK=merge-snapshots-unseen  不看合入后验证留下的快照
#       「✗ 自证 main-merge-snapshot-left：」
#   COMPLETION_CLEANUP_BREAK=snapshot-session-unchecked  别的会话的合入后验证快照也列
#       「✗ 自证 other-session-merge-snapshot-ignored：」
#   COMPLETION_CLEANUP_BREAK=snapshot-owner-unchecked  合入后验证的一层没有归属文件（或写得不对）也当本会话的
#       「✗ 自证 merge-snapshot-without-owner-ignored：」
#   COMPLETION_CLEANUP_BREAK=snapshot-removed-listed  快照 repo/ 已删的一层也列（列那一层的目录）
#       「✗ 自证 merge-snapshot-removed-not-listed：」
#   COMPLETION_CLEANUP_BREAK=baseline-declaration-ignored  主 agent 声明的「基线」不认
#       「✗ 自证 merge-snapshot-declared-baseline-passes：」
#       「✗ 自证 segment-tree-declared-baseline-passes：」
#   COMPLETION_CLEANUP_BREAK=baseline-exempt-for-subagent  子 agent 声明的「基线」也认
#       「✗ 自证 subagent-baseline-declaration-listed：」
#   COMPLETION_CLEANUP_BREAK=subagent-snapshots-judged  子 agent 交回、收工时也判本会话的合入后验证快照
#       「✗ 自证 subagent-merge-snapshots-not-listed：」
#   COMPLETION_CLEANUP_BREAK=merge-snapshot-root-unlisted-unnoted  <工作根>/merge-and-verify/ 在而列不出时当作没有快照，不进没判全的几处
#       「✗ 自证 merge-snapshot-root-unlisted-pass-reported：」
#   COMPLETION_CLEANUP_BREAK=swap-tree-always-listed  <草稿目录>/hook-swap/tree 这一形的行不做最后一看：命令行点到的、已经没了的也列
#       「✗ 自证 swap-tree-in-use-not-listed：」
#       「✗ 自证 swap-tree-gone-not-listed：」
#   COMPLETION_CLEANUP_BREAK=nested-declaration-ignored  子 agent 不读它认领目录与草稿目录底下的下层目录里的 .deliverables（下层声明了交接的副本照列）
#       「✗ 自证 nested-lower-handover-passes：」
#   COMPLETION_CLEANUP_BREAK=nested-declaration-overtrusted  认下层声明认过了头：主 agent 也读下层目录里的 .deliverables，下层那一份在（不论写了哪几行、是不是符号链接）就把它那一层整棵算已声明
#       「✗ 自证 nested-lower-handover-main-listed：」
#       「✗ 自证 nested-undeclared-copy-blocks：」
#       「✗ 自证 nested-lower-broken-line-named：」
#       「✗ 自证 nested-lower-declaration-symlink-named：」
#   COMPLETION_CLEANUP_BREAK=nested-declaration-problems-unnamed  上层子 agent 被拦时，说明里不点名下层的声明为什么没认上
#       「✗ 自证 nested-lower-broken-line-named：」
#       「✗ 自证 nested-lower-declaration-missing-named：」
#       「✗ 自证 nested-lower-declaration-symlink-named：」
#   COMPLETION_CLEANUP_BREAK=part-unreadable-passes  引导（下面读部分文件的那一段）载不全部分文件（读不到、编不过、编得过而 import 抛错或入口要的名字缺）时不按「读不到」处置：照原样抛错、进程退 1（Claude Code 照常放行、什么都不拦）；只在自证起的子进程里认（引导读它，不经 python 部分文件）
#       「✗ 自证 bootstrap-part-unreadable-refused：」「✗ 自证 bootstrap-first-part-emptied-refused：」「✗ 自证 bootstrap-main-part-emptied-refused：」「✗ 自证 bootstrap-part-unreadable-stop-hook-active-released：」「✗ 自证 bootstrap-part-unreadable-subagent-stop-released：」
#   COMPLETION_CLEANUP_BREAK=part-unreadable-always-refuses  引导载不全部分文件时不先读 hook 输入、一律退 2（主 agent 第二回 Stop、子 agent 的 SubagentStop 与交回也拦）
#       「✗ 自证 bootstrap-part-unreadable-stop-hook-active-released：」「✗ 自证 bootstrap-part-unreadable-subagent-stop-released：」
# python 代码分在同目录的这几份（本文件是入口：bash 这一层与下面的引导；引导先把每一份读进来、核顶层名字、语法解析过（只读语法树、不执行），再按次序 import 它们（每一份是普通 python 模块，用到别份的名字写显式 import、不留环），不单独执行；
# 载不全任何一份（读不到、编不过、编得过而不完整：顶层没有入口要的名字——COMPLETION_CLEANUP_PART_REQUIRED_NAMES 一份一组，import 之前用 ast 静态核、点名缺哪个——或 import 抛错），照缺依赖的处置：先读 hook 输入，
# 只在主 agent 第一回的 Stop（没有 stop_hook_active、不带 agent_id）上退 2、stderr 点名是哪一份、不抛错退 1；主 agent 续跑回来的 Stop（stop_hook_active 为真）、SubagentStop、交回的 PreToolUse 与读不出的输入放行，
# 记一条检出（findings 以「完工清理闸没判、照常放行」起头），免得子 agent 与主 agent 卡死在收工上；主函数 main 在引导核完之后由引导调用，部分文件末尾不自己退出）：
# completion_cleanup_guard_decisions.py：弄坏开关表、认领与运行目录与仓副本与编译目录与库拷贝的扫法、任务现场的读法、声明（.deliverables）、后台任务与子 agent 的认法、decide 与 hook_main
# completion_cleanup_guard_selftest.py：自证（Scenario、selftest_cells、run_switch、selftest）与 main
# 弄坏开关的名单、读开关的代码在这几份 python 里（引导里那一条在引导里），开关表与红的格写在这个文件头。
set -uo pipefail
HOOK_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# python 程序从文件描述符 3 读，标准输入留给钩子的 JSON。
python3 /dev/fd/3 "$HOOK_DIR" "$@" 3<<'PY'
import ast, os, sys
COMPLETION_CLEANUP_PART_NAMES = ("completion_cleanup_guard_decisions.py", "completion_cleanup_guard_selftest.py",)   # 按次序 import（后一份 import 前一份、没有环）；路径按钩子自己所在的目录（argv[1]）取
COMPLETION_CLEANUP_PART_REQUIRED_NAMES = (   # 与上一行同次序、一份部分文件一组：入口要的名字（这一份第一个、最后一个绑定的名字与别份、入口引到的名字）；import 之前用 ast 核这一份顶层的 def、class 与赋值绑定了这些名字，缺一个就是不完整（截空、截短）
    ("HOOK_DIR", "BREAK_TABLE", "decide", "record_detection", "report_unjudged", "hook_main"),
    ("iso", "selftest_cells", "run_switch", "selftest", "main"),
)
UNJUDGED_SKIP_PREFIX = "完工清理闸没判、照常放行"   # 判不了（引导载不全部分文件、report_unjudged）那一类检出的 findings 起头
# 引导里的弄坏开关只在自证起的子进程里认：自证（completion_cleanup_guard_selftest.py 的 bootstrap-part-unreadable-refused 等格）起拷出来的钩子时带上这两个值
SELFTEST_MARK_VARIABLE = "COMPLETION_CLEANUP_SELFTEST_MARK"
SELFTEST_MARK_CONTENT = "completion-cleanup-guard.sh --selftest-cells 起的子进程"

def bootstrap_break_is_set(item):
    return os.environ.get(SELFTEST_MARK_VARIABLE) == SELFTEST_MARK_CONTENT and item in os.environ.get("COMPLETION_CLEANUP_BREAK", "").split(",")

def refuses_stop(hook_input):
    """这一次是不是主 agent 第一回的 Stop：没有 stop_hook_active、不带 agent_id。别的事件、读不出的输入都不是。"""
    return isinstance(hook_input, dict) and hook_input.get("hook_event_name") == "Stop" and hook_input.get("stop_hook_active") is not True and not hook_input.get("agent_id")

def record_part_unusable_release(hook_input, problem):
    """部分文件载不全而放行的那一次，照「放行但必记检出」往检出记录追加一行：decisions 的 record_detection 此刻用不了，借同目录 lib_work_root.py 的 record_unjudged_pass。交一句给人看的说明。"""
    sys.dont_write_bytecode = True
    event = str(hook_input.get("hook_event_name") or "事件读不到") if isinstance(hook_input, dict) else "输入读不出"
    try:
        import importlib.util
        specification = importlib.util.spec_from_file_location("lib_work_root", os.path.join(sys.argv[1], "lib_work_root.py"))
        library = importlib.util.module_from_spec(specification)
        specification.loader.exec_module(library)
        return library.record_unjudged_pass("completion-cleanup-guard.sh", hook_input, f"完工清理闸（{event}）", [f"{UNJUDGED_SKIP_PREFIX}：{problem}"])[1]
    except Exception as error:   # 文件不在、语法错都算读不到
        return f"检出没记成：读不到同目录的 lib_work_root.py（{error!r}）"

def hook_part_unusable(hook_part_path, hook_part_error):
    """一份部分文件载不全：先读 hook 输入（钩子那种起法），只在主 agent 第一回的 Stop 上按拒判（退 2、点名那一份）；别的事件放行并记检出，免得卡死。"""
    if bootstrap_break_is_set("part-unreadable-passes"):
        raise hook_part_error   # 弄坏开关 part-unreadable-passes：照原样抛错、进程退 1（只在自证起的子进程里认）
    problem = f"读不到或编不过部分文件 {hook_part_path}（{type(hook_part_error).__name__}：{hook_part_error}）"
    if len(sys.argv) == 2 and not bootstrap_break_is_set("part-unreadable-always-refuses"):
        import json
        try:
            hook_input = json.loads(sys.stdin.buffer.read().decode("utf-8"))
        except ValueError:   # 读不出：不知道是谁在收工，不拦
            hook_input = None
        if not refuses_stop(hook_input):
            print(f"! 完工清理闸：{problem}，这一次没判（放行；只有主 agent 第一回的 Stop 才拦，续跑回来的 Stop、子 agent 的收工与交回不拦），{record_part_unusable_release(hook_input, problem)}", file=sys.stderr)
            sys.exit(0)
    print(f"✗ 完工清理闸：{problem}，这一次没法判，按拒判", file=sys.stderr)
    print("→ 怎么办：把这一份恢复成编得过、完整的版本（git 里取，或让主 agent 换回）再收工（这一份恢复之前，完工清理闸拦主 agent 第一回的 Stop；续跑回来的 Stop、子 agent 的收工与交回放行并记检出）", file=sys.stderr)
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
for hook_part_name, hook_part_required in zip(COMPLETION_CLEANUP_PART_NAMES, COMPLETION_CLEANUP_PART_REQUIRED_NAMES, strict=True):   # 每一份先读进来、核顶层绑定了入口要的名字、语法解析过（只读语法树、不执行），全核完全过了才 import：读不到、语法错或缺名字的，按缺依赖的处置（点名是哪一份、缺哪个名字），不让 python 在 main 之前抛错退 1 而被照常放行
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
    import completion_cleanup_guard_decisions
except Exception as hook_part_error:
    hook_part_unusable(os.path.join(sys.argv[1], "completion_cleanup_guard_decisions.py"), hook_part_error)
try:
    from completion_cleanup_guard_selftest import main
except Exception as hook_part_error:
    hook_part_unusable(os.path.join(sys.argv[1], "completion_cleanup_guard_selftest.py"), hook_part_error)
sys.exit(main())   # 主函数由引导在核完之后调，不靠部分文件末行自己退出
PY
