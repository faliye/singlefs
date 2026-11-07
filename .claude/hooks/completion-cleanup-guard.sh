#!/usr/bin/env bash
# 完工清理闸（Claude Code 钩子）：主 agent 结束一轮、子 agent 交回与收工时，自己认领、自己建的仓副本、编译目录与运行目录里的库拷贝还在，
# 或自己的 Bash 命令点到过的迁库遗留（数据根、判定库服务状态目录与没有归属文件的工作根目录里的旧库与库备份）还在，就拦下，
# 列出路径与 du 的大小，要它删完再收工（用完就删：.claude/main-agent.md「一轮怎么开、怎么收」第 11 条，.claude/agent-common.md「写」一节）。
# hook-events: Stop SubagentStop PreToolUse:SubagentHandback
# gate-similar: handback-scratch-check.sh 上游的交回尾门禁，同挂 PreToolUse:SubagentHandback 与 SubagentStop、同样判自己建的编译目录与仓副本，但只看临时目录（/tmp、$TMPDIR）、按调用时间窗认归属、交回报告写一行「没删」就放行，也不管主 agent；这里看工作根里本会话认领的目录与崩溃放量的运行目录、按归属文件认、只认交付物声明、不收「没删」说明、还挂主 agent 的 Stop；它在上游副本里，不许就地改，并不进去
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
#   主 agent 另判服务给的编译目录（task-server 每次尝试的 CARGO_TARGET_DIR，跑完留给提交它的 agent 用 task-server/task clean <号> 删；用户「你通知完成，
#     编译资源和生成物让对应的agent或者subagent去删」，records/2026-10-30-编译绕过内存总上限.md「补测试那件交回」那一节）：算本会话的全部任务；
#     子 agent 不判这一样（子 agent 一律不往统一任务服务交任务、task clean 也不起，交任务、删编译目录的只剩主 agent）。
#     那个任务（分片的按父任务）已有结局、尝试记的编译目录还在盘上的列进单子，删法 `task-server/task clean <号>`，不直接 rm（项目代码闸照拒直接删）。在排、在跑的不列。
#     读法：服务库只读打开（sqlite3 的 mode=ro；库的位置取 task-server/task_server_lib.py 的 read_configuration，读不了取 default_values），
#     按会话号取 tasks 与 attempts 两张表。不用 task-server/task status --json：不带号只列在排、在跑与完成了没人取的，wait 取过结局的正是要列的那几个却不在里面；
#     按号逐个问要先知道号、一个号一次往返；服务停着时收工照样要判得了。库不在（服务没装过）的不判也不记；库在而读不了、读库的共用模块载不进的照「没判全」记。
#     库的位置可由 COMPLETION_CLEANUP_TASK_SERVER_DATABASE 换，只给自证造样本库用，平常不设。
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
#     种类名现读三处，不在这里另抄：scripts/prune-server/prune_server_client.py 文件头「键空间」一节里「例 / 种类固定 / 种类是」后面的名字，
#     .claude/gate.d/verdict-library-scripts.tsv 第三列的「键空间 <族>/<种类>/」与「<族> 族 <种类>/」，.claude/gate.d/retired-names.tsv 种类是「目录名」的那几行。
#     哪一处读不了或一个名字都读不出，记一处没判全（旧文件库与带种类名的库备份就认不全）。
#   库备份：文件名以 .tar、.tar.gz、.tgz、.tar.zst 结尾，名字里（不分大小写）带 verdict、library 或上面的种类名之一的普通文件；不跟符号链接。
#   旧文件库与库备份在认领目录、草稿目录与上面迁库遗留那几处都认；迁库遗留那几处只认库拷贝、旧文件库与库备份三样，编译目录与仓副本不归这一类。
#   在用的不列，判法调 scripts/hosts/run_build_directory.py 的 open_holders（与它的 holders 同一份：本进程以外的进程的工作目录、根目录、可执行文件、
#     打开的文件描述符或内存映射落在里面；只看得到本用户的；库备份是文件，看有没有进程的文件描述符或内存映射指着它这一份）。认领目录与草稿目录看列出的那一处（含它底下各层）；运行目录看整个运行目录：
#     那一趟的日志或别的子目录有进程开着，就是那一趟还在跑，里面的库拷贝与编译目录都不列。
# 要留的交付物：报告、补丁与打补丁前的备份是普通文件与目录的，本来就不在单上。要留的副本写进它所在那个认领目录的 .deliverables
#   （与归属文件同一层；子 agent 的草稿目录本身也认），一行一条：
#     <相对那个目录的路径>  <种类>  <为什么留>      种类四选一：报告、补丁、备份、交接；为什么留至少 4 个字；# 起头的行是注释
#   交接：子 agent 交给主 agent 接着用的副本（主 agent 照它的报告在里面跑命令）。子 agent 交回、收工时不列；主 agent 结束一轮时照列，用完就删。
#   声明了的那一棵里仓副本、库拷贝、旧文件库与库备份不列，编译目录照列（编译目录不能声明成交付物）；路径落到那个目录外、种类不认得、理由太短的行不算，列进拦下的说明。
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
# 退役判据：工作根与崩溃放量的运行目录不再放副本与库拷贝（驱动与脚本都改成每趟自己的目录、跑完自己删）、服务给的编译目录改回由服务跑完就删，连续四周这道闸一次都没拦；
#   迁库遗留一类另判：判定库不再迁库、不再翻族的布局（库格式锁定），数据根、判定库服务的状态目录与没有归属文件的工作根目录连续四周一份迁库遗留都没认出（「认不出是谁的」里也没有）。
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
#   COMPLETION_CLEANUP_BREAK=repository-copy-unrecognized  不认仓副本（.git、Cargo.toml 加 crates）
#       「✗ 自证 main-repository-copy-left：」
#       「✗ 自证 main-handover-copy-left：」
#       「✗ 自证 subagent-assigned-directory-left：」
#       「✗ 自证 subagent-assigned-relative-by-repository-root：」
#       「✗ 自证 subagent-assigned-shared-reading：」
#       「✗ 自证 main-scratchpad-copy-left：」
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
#   COMPLETION_CLEANUP_BREAK=all-sessions  归属是别的会话的目录也当本会话的判
#       「✗ 自证 other-session-directory-ignored：」
#       「✗ 自证 other-session-leftover-ignored：」
#   COMPLETION_CLEANUP_BREAK=deliverables-ignored  不读 .deliverables
#       「✗ 自证 deliverables-only-passes：」
#       「✗ 自证 subagent-handover-passes：」
#       「✗ 自证 declared-archive-passes：」
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
#   COMPLETION_CLEANUP_BREAK=subagent-sees-whole-session  子 agent 也判本会话认领的全部目录
#       「✗ 自证 subagent-ignores-siblings：」
#       「✗ 自证 subagent-assigned-relative-not-by-cwd：」
#       「✗ 自证 subagent-assigned-relay-ignored：」
#   COMPLETION_CLEANUP_BREAK=prompt-scratch-ignored  不认派发提示里「草稿目录：」点名的那一个
#       「✗ 自证 subagent-assigned-directory-left：」
#       「✗ 自证 subagent-assigned-relative-by-repository-root：」
#       「✗ 自证 subagent-assigned-shared-reading：」
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
#   COMPLETION_CLEANUP_BREAK=service-builds-ignored  不认服务给的编译目录
#       「✗ 自证 main-service-build-left：」
#   COMPLETION_CLEANUP_BREAK=service-build-removal-by-rm  服务给的编译目录删法用 rm -rf（照别的编译目录写）
#       「✗ 自证 main-service-build-left：」
#   COMPLETION_CLEANUP_BREAK=service-builds-session-unchecked  别的会话提交的任务的编译目录也列
#       「✗ 自证 other-session-service-build-ignored：」
#   COMPLETION_CLEANUP_BREAK=service-builds-unfinished-listed  在排、在跑的任务的编译目录也列
#       「✗ 自证 running-service-build-not-listed：」
#   COMPLETION_CLEANUP_BREAK=subagent-service-builds-judged  子 agent 交回、收工时也判本会话的服务编译目录（子 agent 不交任务、不起 task clean，列给它也删不了）
#       「✗ 自证 subagent-service-builds-not-listed：」
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
set -uo pipefail
HOOK_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# python 程序从文件描述符 3 读，标准输入留给钩子的 JSON。
python3 /dev/fd/3 "$HOOK_DIR" "$@" 3<<'PY'
import concurrent.futures, fnmatch, glob, hashlib, importlib.machinery, importlib.util, io, json, os, re, shlex, shutil, sqlite3, subprocess, sys, tempfile, time, traceback, urllib.parse
from datetime import datetime, timezone

sys.dont_write_bytecode = True
HOOK_DIR = sys.argv[1]
ARGUMENTS = sys.argv[2:]
HOOK_PATH = os.path.join(HOOK_DIR, "completion-cleanup-guard.sh")
REPOSITORY_ROOT = os.path.dirname(os.path.dirname(HOOK_DIR))
BREAK_VARIABLE = "COMPLETION_CLEANUP_BREAK"
BREAK_TABLE = [
    ("build-directory-unrecognized", "main-build-directory-left"),
    ("repository-copy-unrecognized", "main-repository-copy-left"),
    ("library-copy-unrecognized", "main-run-library-copy-left"),
    ("all-sessions", "other-session-directory-ignored"),
    ("deliverables-ignored", "deliverables-only-passes"),
    ("build-directory-declarable", "build-directory-never-deliverable"),
    ("handover-exempt-for-main", "main-handover-copy-left"),
    ("background-ignored", "background-running-passes"),
    ("reminder-counts-as-running", "reminder-only-still-judged"),
    ("subagents-ignored", "subagent-running-passes"),
    ("notifications-ignored", "subagent-finished-still-judged"),
    ("open-files-ignored", "held-copy-not-listed"),
    ("subagent-sees-whole-session", "subagent-ignores-siblings"),
    ("prompt-scratch-ignored", "subagent-assigned-directory-left"),
    ("prompt-scratch-relative-by-cwd", "subagent-assigned-relative-by-repository-root"),
    ("run-windows-ignored", "foreign-run-directory-ignored"),
    ("attribution-by-window", "foreign-session-run-in-window-not-listed"),
    ("ledger-ignored", "own-run-blocked-after-creator-exits"),
    ("creator-alive-ignored", "run-in-progress-not-listed"),
    ("subagent-run-session-level", "subagent-ignores-sibling-runs"),
    ("run-holders-leaf-only", "run-held-deep-not-listed"),
    ("release-never", "unchanged-after-block-releases"),
    ("release-on-any-continuation", "changed-after-block-blocks-again"),
    ("size-omitted", "block-lists-size"),
    ("scratchpad-ignored", "main-scratchpad-copy-left"),
    ("unjudged-pass-silent", "truncated-scan-pass-reported"),
    ("unjudged-skip-silent", "unjudged-skip-recorded"),
    ("unreadable-directory-unnoted", "unreadable-directory-pass-reported"),
    ("work-root-unlisted-unnoted", "work-root-unlisted-pass-reported"),
    ("transcript-unreadable-unnoted", "transcript-unreadable-skip-reported"),
    ("service-builds-ignored", "main-service-build-left"),
    ("service-build-removal-by-rm", "main-service-build-left"),
    ("service-builds-session-unchecked", "other-session-service-build-ignored"),
    ("service-builds-unfinished-listed", "running-service-build-not-listed"),
    ("subagent-service-builds-judged", "subagent-service-builds-not-listed"),
    ("guard-off", "break-switch-refused-outside-selftest"),
    ("guard-off-with-others", "guard-off-combined-refused-in-process"),
    ("run-root-default-unseen", "run-root-default-under-work-root"),
    ("data-root-leftovers-unseen", "data-root-family-library-blocked"),
    ("prune-state-directory-unseen", "prune-state-library-copy-blocked"),
    ("service-library-listed", "service-library-not-listed"),
    ("unit-file-ignored", "unit-file-state-directory-read"),
    ("old-library-unrecognized", "data-root-family-library-blocked"),
    ("leftover-unjudged-unnoted", "species-unreadable-pass-reported"),
    ("no-owner-directories-skipped", "no-owner-mentioned-copy-blocked"),
    ("owned-directories-leftover-scanned", "other-session-leftover-ignored"),
    ("registered-entries-listed", "registered-entry-not-listed"),
    ("mention-ancestors-ignored", "data-root-family-library-blocked"),
    ("cd-ignored", "relative-after-cd-attributed"),
    ("anchor-mentions-counted", "anchor-mentions-not-attributed"),
    ("unmentioned-counted-own", "no-owner-unmentioned-reported-not-blocked"),
    ("read-only-mentions-counted", "read-only-mentions-not-attributed"),
    ("leftover-mentions-session-level", "subagent-leftover-by-own-commands"),
    ("archives-unrecognized", "library-archive-blocked"),
    ("archive-any-name", "library-archive-blocked"),
    ("declaration-outside-accepted", "data-root-leftover-not-declarable"),
]
BREAK_ITEMS = tuple(item for item, _ in BREAK_TABLE)
SELFTEST_ENTRIES = (["--selftest"], ["--selftest-cells"])
NAME_PATTERN = re.compile(r"^[A-Za-z0-9_-]+$")
HANDBACK_TOOL = "SubagentHandback"
SUBAGENT_IDLE_SECONDS = 1800
NOTIFICATION_SLACK_SECONDS = 30
WINDOW_SLACK_SECONDS = 1.0
SCAN_MAX_DEPTH = 6
RUN_SCAN_MAX_DEPTH = 3
SCAN_MAX_DIRECTORIES = 50000
SCAN_LIMIT_VARIABLE = "COMPLETION_CLEANUP_SCAN_MAX_DIRECTORIES"   # 只给自证造截断用：设成正整数时上限照它
DETECTIONS_VARIABLE = "AGENT_HOOK_DETECTIONS"
UNJUDGED_FINDING_PREFIX = "完工清理闸没判全、照常放行"
UNJUDGED_SKIP_PREFIX = "完工清理闸没判、照常放行"   # 判不了（report_unjudged）那一类检出的 findings 起头
INPUT_PREFIX_BYTES = 200
DU_TIMEOUT_SECONDS = 30
DECLARATION_FILE = ".deliverables"
DELIVERABLE_KINDS = ("报告", "补丁", "备份", "交接")
HANDOVER_KIND = "交接"
REASON_MINIMUM_CHARACTERS = 4
RUN_COMMAND_KEYWORDS = re.compile(r"crash-amplification|crash-point-replay|broad-mutation|mutation-shard-run|qemu-device-streams|lkmm\.sh|crates-mutation-replay"
                                  r"|formal-verification|research-build-and-replay|model-differential-and-scenarios|gate-staged|gate\.sh")
TIMER_AND_WATCHDOG_SCRIPTS = ("remind.sh", "watch.sh", "agent-watch.py")
LAUNCHER_WORDS = ("bash", "sh", "python3", "python", "nice", "env", "exec")
CLAIM_PATTERN = re.compile(r"work-directory-ownership\.sh['\"]?\s+claim\s+(['\"]?)([^\s'\"]+)\1")
STATE_DIRECTORY_NAME = "completion-cleanup-guard"
SCRATCHPAD_ROOT_VARIABLE = "COMPLETION_CLEANUP_SCRATCHPAD_ROOT"
KIND_BUILD, KIND_WORKTREE, KIND_REPOSITORY, KIND_LIBRARY = "编译目录", "工作树", "仓副本", "库拷贝"
KIND_SERVICE_BUILD = "服务编译目录"   # task-server 给的编译目录：只经 task-server/task clean <号> 删
TASK_SERVER_DATABASE_VARIABLE = "COMPLETION_CLEANUP_TASK_SERVER_DATABASE"   # 只给自证造样本库用：设了就读它，平常不设
TASK_SERVER_CLIENT = "task-server/task"
TASK_SERVER_FINISHED_STATE = "finished"
SERVICE_BUILD_TASKS = {}   # 这一次判出的服务编译目录 → 删它用的任务号（分片的子任务记父任务号）
RUN_NAME_PATTERN = re.compile(r"(?:^|-)([0-9]{8}T[0-9]{6}Z)-([0-9]+)(?=-|$)")
SESSION_VARIABLES = ("WORK_DIRECTORY_OWNER_SESSION", "CLAUDE_CODE_SESSION_ID", "CLAUDE_SESSION_ID")
CLAUDE_PID_VARIABLE = "CLAUDE_PID"
CREATOR_START_SLACK_SECONDS = 1.0
ANCESTRY_MAX_DEPTH = 64
RUN_LEDGER_NAME = "run-owners"
RUN_OWNER_FILE = ".owner"
CRASH_AMPLIFICATION_HOME_VARIABLE = "SINGLEFS_CRASH_AMPLIFICATION_HOME"
CRASH_AMPLIFICATION_DIRECTORY_NAME = "crash-amplification"   # 没设上面那个变量时，工作根下的这一层是崩溃放量的家目录
UNKNOWN_SECTION_TITLE = "认不出是谁的"
UNKNOWN_SHOWN_MAX = 30   # 「认不出是谁的」一节最多列几处（每处要核一遍在不在用），其余只报个数
KIND_OLD_LIBRARY, KIND_LIBRARY_ARCHIVE = "旧文件库", "库备份"
LEFTOVER_KINDS = frozenset((KIND_LIBRARY, KIND_OLD_LIBRARY, KIND_LIBRARY_ARCHIVE))   # 迁库遗留那几处只认这三样
LEFTOVER_SCAN_MAX_DEPTH = 4
PRUNE_STATE_SCAN_MAX_DEPTH = 2
ARCHIVE_SUFFIXES = (".tar", ".tar.gz", ".tgz", ".tar.zst")
ARCHIVE_WORDS = ("verdict", "library")
SPECIES_TOKEN = r"[a-z][a-z0-9]*(?:-[a-z0-9]+)+"   # 种类名的样子：小写、带连字符的目录名
# 种类名现读的三处（文件头「判什么」旧文件库那一段）；自证换成样本文件
SPECIES_SOURCES = {
    "client": os.path.join(REPOSITORY_ROOT, "scripts", "prune-server", "prune_server_client.py"),
    "registry": os.path.join(REPOSITORY_ROOT, ".claude", "gate.d", "verdict-library-scripts.tsv"),
    "retired": os.path.join(REPOSITORY_ROOT, ".claude", "gate.d", "retired-names.tsv"),
}
CLIENT_KEY_SPACE_HEADING = "键空间"
RETIRED_DIRECTORY_KIND = "目录名"
GLOB_CHARACTERS = re.compile(r"[*?\[]")
VARIABLE_ASSIGNMENT = re.compile(r"^([A-Za-z_][A-Za-z0-9_]*)=(.*)$")
VARIABLE_REFERENCE = re.compile(r"\$(?:\{([A-Za-z_][A-Za-z0-9_]*)\}|([A-Za-z_][A-Za-z0-9_]*))")
WORK_ROOT_SUBSTITUTION = re.compile(r"\$\(\s*python3\s+[^\s()]*lib_work_root\.py(?:\s+(data-root|shared\s+([A-Za-z0-9._-]+)))?\s*\)")
SHELL_PUNCTUATION = "();<>|&\n"   # 换行也切出来：命令按段判只读，换行隔开的是两段
LOOSE_WORDS = re.compile(r"([;&|()\n]+)|[ \t\r\f\v<>`'\"]+")   # 拆不开时的切法：段的分隔留作一个词，别的符号丢掉
SEGMENT_SEPARATOR = re.compile(r"[;&|()\n]+")
DIRECTORY_CHANGERS = ("cd", "pushd")
# 命令词是这几样的那一段里的路径不算点到（看一眼的不算本会话的）
READ_ONLY_COMMANDS = frozenset(("ls", "du", "stat", "find", "cat", "head", "tail", "less", "grep", "rg", "wc", "sha256sum", "md5sum",
                                "diff", "cmp", "file", "readlink", "realpath", "tree", "df"))
FIND_MODIFYING_PREDICATES = frozenset(("-delete", "-exec", "-execdir"))   # find 带这几样会删、会起别的命令，不算只读


def broken(item):
    return item in os.environ.get(BREAK_VARIABLE, "").split(",")


GUARD_OFF_ITEM = "guard-off"                         # 关守卫项：只在它是唯一一项时认
GUARD_OFF_LENIENT_ITEM = "guard-off-with-others"     # 弄坏开关：自证进程本身里调守卫时，关守卫项与别的项同写也照认


def refuse_break_outside_selftest(value=None, arguments=None):
    """弄坏开关只在自证里认：平常跑设了就退 2，点名变量；关守卫项 guard-off 只在它是唯一一项时认，与别的项同写照拒并点名它（records/2026-10-23-subagent拆分提案.md
    第一百二十二节 ②）；不认识的项名一样拒。value、arguments 不给时取环境里的开关与这一次的参数；自证在进程里给值探同写那一格，
    guard-off-with-others 只在自证进程本身（这一次的参数是自证入口）里认，平常跑同写着它照拒。"""
    value = os.environ.get(BREAK_VARIABLE, "") if value is None else value
    arguments = ARGUMENTS if arguments is None else arguments
    if not value:
        return
    items = [item for item in value.split(",") if item]
    if arguments not in SELFTEST_ENTRIES:
        others = [item for item in items if item != GUARD_OFF_ITEM]
        if GUARD_OFF_ITEM in items and others and not (ARGUMENTS in SELFTEST_ENTRIES and broken(GUARD_OFF_LENIENT_ITEM)):
            print(f"✗ 完工清理闸：{BREAK_VARIABLE}={value}：关守卫项 {GUARD_OFF_ITEM} 只在它是唯一一项时认，与别的项（{'、'.join(others)}）同写照拒；"
                  "弄坏开关只在自证（--selftest）里认，这一次没判", file=sys.stderr)
            print(f"→ 怎么办：unset {BREAK_VARIABLE} 再来；要看某一项红在哪一格，跑 {BREAK_VARIABLE}=<项> bash {HOOK_PATH} --selftest，一次只设那一项", file=sys.stderr)
            sys.exit(2)
        if GUARD_OFF_ITEM not in items:
            print(f"✗ 完工清理闸：{BREAK_VARIABLE}={value} 是弄坏开关，只在自证（--selftest）里认；平常跑设了就拒，这一次没判", file=sys.stderr)
            print(f"→ 怎么办：unset {BREAK_VARIABLE} 再来；要看开关红在哪一格，跑 bash {HOOK_PATH} --selftest", file=sys.stderr)
            sys.exit(2)
    unknown = [item for item in items if item not in BREAK_ITEMS]
    if unknown:
        print(f"✗ 完工清理闸：{BREAK_VARIABLE} 里的 {'、'.join(unknown)} 不认识（只认 {'、'.join(BREAK_ITEMS)}）", file=sys.stderr)
        print(f"→ 怎么办：项名照 {HOOK_PATH} 文件头「弄坏开关」一段写", file=sys.stderr)
        sys.exit(2)


MODULE_PATHS = {
    "lib_work_root": os.path.join(HOOK_DIR, "lib_work_root.py"),
    # 「草稿目录：」一行怎么认（只认行首那一行，与派发闸 ⑯ ⑰、交回闸 ⑥ 共用一份；判决 research/prompts/landing-audit-r4-main-verification.md K2-4）
    "lib_prompt_scratch_line": os.path.join(HOOK_DIR, "lib_prompt_scratch_line.py"),
    "agent_watch": os.path.join(REPOSITORY_ROOT, "scripts", "agents", "agent-watch.py"),
    "session_transcript": os.path.join(REPOSITORY_ROOT, ".claude", "singlefs-ai-sop", "scripts", "session-transcript.py"),
    "run_build_directory": os.path.join(REPOSITORY_ROOT, "scripts", "hosts", "run_build_directory.py"),
    # 服务库在哪：只用它的 read_configuration 与 default_values，不起服务
    "task_server_lib": os.path.join(REPOSITORY_ROOT, "task-server", "task_server_lib.py"),
    # 判定库服务的状态目录与服务自己那份库在哪：只用它的 Paths、unit_library 与 LIBRARY_DIRECTORY_NAME，不起服务（文件没有 .py 后缀，照源码载入）
    "prune_server_control": os.path.join(REPOSITORY_ROOT, "prune-server", "control"),
}
LOADED_MODULES = {}


def module(name):
    if name not in LOADED_MODULES:
        path = MODULE_PATHS[name]
        loader = None if path.endswith(".py") else importlib.machinery.SourceFileLoader(name, path)
        specification = importlib.util.spec_from_file_location(name, path, loader=loader)
        loaded = importlib.util.module_from_spec(specification)
        specification.loader.exec_module(loaded)
        LOADED_MODULES[name] = loaded
    return LOADED_MODULES[name]


class CannotJudge(Exception):
    pass


def is_strictly_under(path, directory):
    return path.startswith(directory.rstrip("/") + "/")


def is_project_root_or_above(path):
    real = os.path.realpath(path)
    project = os.path.realpath(REPOSITORY_ROOT)
    return real == project or is_strictly_under(project, real)


# ── 归属与判哪几处
def owner_session(directory):
    """<目录>/.owner 三行（session、created、purpose）都在才认，交 session 的值；没有或写得不对交 None。"""
    path = os.path.join(directory, ".owner")
    if os.path.islink(path) or not os.path.isfile(path):
        return None
    try:
        lines = open(path, encoding="utf-8").read().splitlines()
    except OSError:
        return None
    values = dict(line.split(": ", 1) for line in lines if ": " in line)
    if len(lines) != 3 or not all(values.get(key) for key in ("session", "created", "purpose")):
        return None
    return values["session"]


def work_root():
    return module("lib_work_root").work_root()


def scratchpad_root():
    return os.environ.get(SCRATCHPAD_ROOT_VARIABLE) or module("lib_work_root").claude_session_temporary_root()


def main_roots(session, main_transcript, notes=None):
    """主 agent 判的目录：[(目录, 它的认领目录或 None, 下一层不看的名字)]。工作根在而列不出（读不了）时本会话认领的目录一个都没判，
    给了 notes 的往里记一句（照「放行但必记检出」；弄坏开关 work-root-unlisted-unnoted 下不记）；工作根不在的照旧什么都不记。"""
    roots = []
    base = work_root()
    try:
        names = sorted(os.listdir(base))
    except FileNotFoundError:
        names = []
    except OSError as error:
        names = []
        if notes is not None and not broken("work-root-unlisted-unnoted"):
            notes.append(f"工作根 {base} 列不出（{type(error).__name__}: {error}），本会话认领的目录一个都没判；出路：ls -ld {base} 看权限，修好再收工")
    for name in names:
        path = os.path.join(base, name)
        if os.path.islink(path) or not os.path.isdir(path):
            continue
        owner = owner_session(path)
        if owner == session or (owner is not None and broken("all-sessions")):
            roots.append((path, path, set()))
    if not broken("scratchpad-ignored"):
        scratchpad = os.path.join(scratchpad_root(), os.path.basename(os.path.dirname(main_transcript)), session)
        if os.path.isdir(scratchpad) and not os.path.islink(scratchpad):
            roots.append((scratchpad, None, {"tasks"}))
    return roots


def first_user_text(transcript_path):
    """会话记录里第一条 user 记录的文字（子 agent 的派发提示）。"""
    try:
        with open(transcript_path, encoding="utf-8", errors="replace") as handle:
            for line in handle:
                if '"user"' not in line:
                    continue
                try:
                    record = json.loads(line)
                except ValueError:
                    continue
                message = record.get("message") if isinstance(record, dict) else None
                if not isinstance(message, dict) or message.get("role") != "user":
                    continue
                content = message.get("content")
                if isinstance(content, str):
                    return content
                if isinstance(content, list):
                    return "\n".join(block.get("text", "") for block in content if isinstance(block, dict) and block.get("type") == "text")
    except OSError:
        return ""
    return ""


def subagent_roots(session, agent_transcript, calls):
    """子 agent 判的目录：它自己 claim 的工作根下一层目录，与派发提示「草稿目录：」点名的那一个。"""
    base = work_root()
    candidates = []
    for call in calls:
        if call["name"] != "Bash":
            continue
        for match in CLAIM_PATTERN.finditer(str(call["input"].get("command") or "")):
            text = os.path.expanduser(match.group(2)).rstrip("/")
            candidates.append((os.path.normpath(text if "/" in text else os.path.join(base, text)), "claim"))
    if not broken("prompt-scratch-ignored"):
        reading = module("lib_prompt_scratch_line")
        for value in reading.scratch_line_values(first_user_text(agent_transcript)):
            written = module("lib_work_root").expand_written_path(value.rstrip("/"))
            if not os.path.isabs(written) and not broken("prompt-scratch-relative-by-cwd"):   # 写相对路径的按仓根解（与那份库的 scratch_directories 同一种解法），不按钩子此刻的当前目录
                written = os.path.join(reading.repository_root(), written)
            candidates.append((os.path.normpath(os.path.abspath(written)), "assigned"))
    roots, seen = [], set()
    for path, origin in candidates:
        if path in seen or not is_strictly_under(path, base) or os.path.islink(path) or not os.path.isdir(path):
            continue
        top = os.path.join(base, os.path.relpath(path, base).split(os.sep)[0])
        owner = owner_session(top)
        if origin == "claim" and (path != top or owner != session):
            continue
        if owner is not None and owner != session:
            continue
        seen.add(path)
        roots.append((path, top, set()))
    return [root for root in roots if not any(is_strictly_under(root[0], other[0]) for other in roots)]


# ── 交付物声明
def declarations(directories, role):
    """读这几个目录里的 .deliverables：交 ({声明了的绝对路径: 种类}, [写坏的行])；只收这个角色免列的种类。
    路径落到那个目录外的行不算（数据根层的旧库靠声明放不过）；弄坏开关 declaration-outside-accepted 下照收。"""
    exempt_kinds = set() if broken("deliverables-ignored") else {"报告", "补丁", "备份"}
    if exempt_kinds and (role == "subagent" or broken("handover-exempt-for-main")):
        exempt_kinds.add(HANDOVER_KIND)
    exempt, problems = {}, []
    for directory in dict.fromkeys(directory for directory in directories if directory):
        path = os.path.join(directory, DECLARATION_FILE)
        if os.path.islink(path) or not os.path.isfile(path):
            continue
        try:
            lines = open(path, encoding="utf-8", errors="replace").read().splitlines()
        except OSError:
            problems.append(f"{path} 读不了，里面的声明不算")
            continue
        for number, line in enumerate(lines, 1):
            text = line.strip()
            if not text or text.startswith("#"):
                continue
            parts = text.split(None, 2)
            if len(parts) < 3 or parts[1] not in DELIVERABLE_KINDS or len(parts[2].strip()) < REASON_MINIMUM_CHARACTERS:
                problems.append(f"{path}:{number} 不是「<相对路径> <种类> <为什么留>」（种类：{'、'.join(DELIVERABLE_KINDS)}；理由至少 {REASON_MINIMUM_CHARACTERS} 个字），这一行不算")
                continue
            relative = parts[0].strip("`'\"").rstrip("/")
            target = os.path.normpath(os.path.join(directory, relative))
            if (os.path.isabs(relative) or not is_strictly_under(target, directory)) and not broken("declaration-outside-accepted"):
                problems.append(f"{path}:{number} 的路径 {relative} 不在 {directory} 里面，这一行不算")
                continue
            if parts[1] in exempt_kinds:
                exempt[target] = parts[1]
    return exempt, problems


# ── 认副本
def species_in_name(name, species):
    """名字里带的那个种类名（整个名字，或前后是 . - _ 与名字两头）；没有交 None。"""
    for candidate in species:
        if re.search(rf"(?:^|[._-]){re.escape(candidate)}(?:$|[._-])", name):
            return candidate
    return None


def is_library_archive(name, species):
    """库备份压缩包（文件头「判什么」库备份那一段）。弄坏开关 archives-unrecognized 下一个都不认，archive-any-name 下压缩包都认。"""
    if broken("archives-unrecognized"):
        return False
    lowered = name.lower()
    if not lowered.endswith(ARCHIVE_SUFFIXES):
        return False
    if broken("archive-any-name"):
        return True
    return any(word in lowered for word in ARCHIVE_WORDS) or species_in_name(lowered, species) is not None


def kind_of(path, names, species=(), by_name=False):
    """这个目录是什么（文件头「判什么」）；by_name 为真时（不是起头那一处）另按种类名认旧文件库。"""
    kind = module("agent_watch").scratch_hog_kind(path, names)
    if kind is None and ".rustc_info.json" in names:
        kind = KIND_BUILD
    if kind == KIND_BUILD and broken("build-directory-unrecognized"):
        kind = None
    if kind == KIND_REPOSITORY:
        if broken("repository-copy-unrecognized"):
            kind = None
        elif os.path.isfile(os.path.join(path, ".git")):
            kind = KIND_WORKTREE
    if (kind is None and "CURRENT" in names and any(name.startswith("MANIFEST-") for name in names)
            and not broken("library-copy-unrecognized")):
        kind = KIND_LIBRARY
    if (kind is None and by_name and names and not broken("old-library-unrecognized")
            and species_in_name(os.path.basename(path), species) is not None):
        kind = KIND_OLD_LIBRARY
    return kind


def scan_limit():
    """扫多少个目录到顶：COMPLETION_CLEANUP_SCAN_MAX_DIRECTORIES 设成正整数的照它（只给自证造截断用），别的取 SCAN_MAX_DIRECTORIES。"""
    text = os.environ.get(SCAN_LIMIT_VARIABLE, "")
    return int(text) if text.isdigit() and int(text) > 0 else SCAN_MAX_DIRECTORIES


def scan(root, max_depth, exempt, skipped_names, budget, wanted=None, skip=None):
    """从 root 往下找副本：交 [(路径, 种类)]；认出一样就不再往里找，声明了的那一棵只找编译目录。
    wanted 给了只交这几种（认出别的种类照样不再往里找）；skip(路径, 层数) 为真的下层目录不进。种类名取 budget["species"]，
    budget["not_by_name"] 里的目录不按名字认旧文件库（数据根表里登记的在用项）。"""
    found = []
    species = budget.get("species", ())
    stack = [(root, 0, False)]
    while stack:
        path, depth, inside_exempt = stack.pop()
        if budget["visited"] >= budget["limit"]:
            budget["truncated"].add(root)
            break
        budget["visited"] += 1
        if is_project_root_or_above(path):
            continue
        try:
            entries = list(os.scandir(path))
        except OSError as error:
            if not broken("unreadable-directory-unnoted"):   # 读不了的目录里有没有副本没判：记下来，放行时照「放行但必记检出」报（静默跳过）
                budget.setdefault("unreadable", []).append(f"{path}（{type(error).__name__}: {error.strerror or error}）")
            continue
        here_exempt = inside_exempt or path in exempt
        kind = kind_of(path, {entry.name for entry in entries}, species, by_name=depth > 0 and path not in budget.get("not_by_name", ()))
        if kind is not None:
            if wanted is not None and kind not in wanted:
                continue
            if kind == KIND_BUILD:
                if not (here_exempt and broken("build-directory-declarable")):
                    found.append((path, kind))
                continue
            if not here_exempt:
                found.append((path, kind))
                continue
        for entry in entries:   # 库备份是文件：这一层的文件逐个认，声明了的不列
            if wanted is not None and KIND_LIBRARY_ARCHIVE not in wanted:
                break
            try:
                is_file = entry.is_file(follow_symlinks=False)
            except OSError:
                continue
            if is_file and is_library_archive(entry.name, species) and not (here_exempt or entry.path in exempt):
                found.append((entry.path, KIND_LIBRARY_ARCHIVE))
        if depth >= max_depth:
            continue
        for entry in sorted(entries, key=lambda item: item.name, reverse=True):
            if depth == 0 and entry.name in skipped_names:
                continue
            try:
                is_directory = entry.is_dir(follow_symlinks=False)
            except OSError:
                continue
            if is_directory and not (skip is not None and skip(entry.path, depth + 1)):
                stack.append((entry.path, depth + 1, here_exempt))
    return found


# ── 运行目录
def run_roots():
    """崩溃放量的运行目录根：SINGLEFS_CRASH_AMPLIFICATION_HOME 设了是它下面的 runs，没设是工作根下 crash-amplification/runs
    （同 scripts/crash/crash-amplification-lib.sh 的 crash_amplification_home_directory）。弄坏开关 run-root-default-unseen 下没设时交空表。"""
    home = os.environ.get(CRASH_AMPLIFICATION_HOME_VARIABLE)
    if home:
        return [os.path.join(home, "runs")]
    if broken("run-root-default-unseen"):
        return []
    return [os.path.join(work_root(), CRASH_AMPLIFICATION_DIRECTORY_NAME, "runs")]


def birth_times(paths):
    ordered = sorted(paths)
    if not ordered:
        return {}
    completed = subprocess.run(["stat", "--printf=%.9W\\t%n\\0", "--", *ordered], capture_output=True, text=True, check=False)
    reported = {}
    for record in completed.stdout.split("\0"):
        seconds_text, separator, path = record.partition("\t")
        if separator:
            try:
                reported[path] = float(seconds_text)
            except ValueError:
                pass
    return {path: (reported[path] if reported.get(path, 0.0) > 0 else None) for path in ordered}


BOOT_SECONDS = []


def boot_seconds():
    if not BOOT_SECONDS:
        value = None
        try:
            for line in open("/proc/stat", encoding="ascii", errors="replace"):
                if line.startswith("btime "):
                    value = float(line.split()[1])
        except (OSError, ValueError):
            value = None
        BOOT_SECONDS.append(value)
    return BOOT_SECONDS[0]


def process_status(pid):
    """(父进程号, 开跑时刻)：进程不在、是僵尸或读不了交 None。"""
    try:
        text = open(f"/proc/{pid}/stat", encoding="utf-8", errors="replace").read()
    except OSError:
        return None
    fields = text.rpartition(")")[2].split()
    boot = boot_seconds()
    if len(fields) < 20 or fields[0] in ("Z", "X") or boot is None:
        return None
    try:
        return int(fields[1]), boot + int(fields[19]) / os.sysconf("SC_CLK_TCK")
    except ValueError:
        return None


def process_environment(pid):
    try:
        data = open(f"/proc/{pid}/environ", "rb").read()
    except OSError:
        return {}
    wanted = {name.encode() for name in (*SESSION_VARIABLES, CLAUDE_PID_VARIABLE)}
    environment = {}
    for item in data.split(b"\0"):
        key, separator, value = item.partition(b"=")
        if separator and key in wanted:
            environment[key.decode()] = value.decode(errors="replace")
    return environment


def session_of(environment):
    for name in SESSION_VARIABLES:
        value = environment.get(name, "")
        if value and NAME_PATTERN.match(value):
            return value
    return None


def evaluated_command(pid):
    """Bash 工具起的那层 shell（bash -c '… && eval <命令> && …'）里 eval 的那条命令；认不出交 None。"""
    try:
        words = [word.decode(errors="replace") for word in open(f"/proc/{pid}/cmdline", "rb").read().split(b"\0")]
    except OSError:
        return None
    if len(words) < 3 or words[1] != "-c":
        return None
    try:
        tokens = shlex.split(words[2])
    except ValueError:
        return None
    for index, token in enumerate(tokens[:-1]):
        if token == "eval":
            return tokens[index + 1]
    return None


def command_digest(command):
    return hashlib.sha256(command.strip().encode()).hexdigest()


def launcher_digest(pid, claude_pid):
    """从 pid 顺着父进程往上，找父进程是 Claude 进程的那一层，交它 eval 的命令的 sha256；找不到交 None。"""
    current = pid
    for _ in range(ANCESTRY_MAX_DEPTH):
        status = process_status(current)
        if status is None:
            return None
        if status[0] == claude_pid:
            command = evaluated_command(current)
            return command_digest(command) if command is not None else None
        if status[0] <= 1:
            return None
        current = status[0]
    return None


def live_creator(path, born):
    """运行目录名里的进程号此刻还在、开跑不晚于目录的创建时间：交 {"pid", "session", "launcher_sha256"}，否则 None。"""
    match = RUN_NAME_PATTERN.search(os.path.basename(path))
    if match is None or born is None:
        return None
    pid = int(match.group(2))
    status = process_status(pid)
    if status is None or status[1] > born + CREATOR_START_SLACK_SECONDS:
        return None
    environment = process_environment(pid)
    claude = environment.get(CLAUDE_PID_VARIABLE, "")
    return {"pid": pid, "session": session_of(environment),
            "launcher_sha256": launcher_digest(pid, int(claude)) if claude.isdigit() else None}


def ledger_directory():
    return os.path.join(module("lib_work_root").state_directory(STATE_DIRECTORY_NAME), RUN_LEDGER_NAME)


def ledger_path(path):
    return os.path.join(ledger_directory(), f"{os.path.basename(path)}-{hashlib.sha256(path.encode()).hexdigest()[:12]}.json")


def record_ledger(path, born, creator):
    """排他记账：先写临时文件，再 link 到记账名（已有就不动）。"""
    if broken("ledger-ignored") or not creator.get("session"):
        return
    target = ledger_path(path)
    temporary = f"{target}.{os.getpid()}.tmp"
    try:
        os.makedirs(os.path.dirname(target), exist_ok=True)
        with open(temporary, "w", encoding="utf-8") as handle:
            json.dump({"path": path, "born": born, **creator}, handle, ensure_ascii=False)
        os.link(temporary, target)
    except OSError:
        pass
    finally:
        try:
            os.remove(temporary)
        except OSError:
            pass


def ledger_entry(path, born):
    if broken("ledger-ignored") or born is None:
        return None
    try:
        entry = json.load(open(ledger_path(path), encoding="utf-8"))
    except (OSError, ValueError):
        return None
    if not isinstance(entry, dict) or entry.get("path") != path or not isinstance(entry.get("born"), (int, float)) or abs(entry["born"] - born) > 0.001:
        return None
    return entry


def prune_ledger():
    """运行目录不在了的记账删掉。"""
    directory = ledger_directory()
    try:
        names = os.listdir(directory)
    except OSError:
        return
    for name in names:
        if not name.endswith(".json"):
            continue
        try:
            recorded = json.load(open(os.path.join(directory, name), encoding="utf-8")).get("path")
        except (OSError, ValueError, AttributeError):
            continue
        if isinstance(recorded, str) and recorded and not os.path.lexists(recorded):
            try:
                os.remove(os.path.join(directory, name))
            except OSError:
                pass


def run_owner(path, born):
    """交 (状态, 归属)：状态是 running（起它的进程还在）、known 或 unknown；归属 {"session", "launcher_sha256"}。"""
    creator = live_creator(path, born)
    if creator is not None:
        record_ledger(path, born, creator)
        if not broken("creator-alive-ignored"):
            return "running", creator
    entry = ledger_entry(path, born) or (creator if creator and creator.get("session") else None)
    recorded = owner_session(path)
    if recorded:
        same = entry is not None and entry.get("session") == recorded
        return "known", {"session": recorded, "launcher_sha256": entry.get("launcher_sha256") if same else None}
    if entry is not None:
        return "known", {"session": entry.get("session"), "launcher_sha256": entry.get("launcher_sha256")}
    return "unknown", None


def call_windows(calls, now):
    return [(call["started_at"] - WINDOW_SLACK_SECONDS, (call["ended_at"] if call["ended_at"] is not None else now) + WINDOW_SLACK_SECONDS)
            for call in calls if call["name"] == "Bash" and RUN_COMMAND_KEYWORDS.search(str(call["input"].get("command") or ""))]


def classify_runs(role, session, own_transcript, calls, now, notes):
    """交 (自己的运行目录, 认不出是谁、创建时间落在自己 Bash 调用时间窗里的运行目录)。起它的进程还在的两边都不放。"""
    directories = []
    for root in run_roots():
        try:
            names = sorted(os.listdir(root))
        except OSError:
            continue
        directories += [os.path.join(root, name) for name in names
                        if os.path.isdir(os.path.join(root, name)) and not os.path.islink(os.path.join(root, name))]
    if not directories:
        return [], []
    prune_ledger()
    if broken("run-windows-ignored"):
        return directories, []
    transcripts = module("session_transcript")
    started = transcripts.epoch_seconds(transcripts.first_timestamp(own_transcript))
    windows = call_windows(calls, now)
    own_digests = {command_digest(str(call["input"].get("command") or "")) for call in calls if call["name"] == "Bash"}

    def recent(path):
        try:
            return started is None or os.lstat(path).st_mtime >= started - WINDOW_SLACK_SECONDS
        except OSError:
            return False

    def creator_alive(path):
        match = RUN_NAME_PATTERN.search(os.path.basename(path))
        return match is not None and os.path.exists(f"/proc/{match.group(2)}")

    births = birth_times([path for path in directories if recent(path) or creator_alive(path)])
    missing = sorted(path for path, born in births.items() if born is None)
    if missing:
        notes.append(f"取不到创建时间的运行目录 {len(missing)} 个没判（认不出起它的进程）：{' '.join(missing[:5])}{' …' if len(missing) > 5 else ''}；"
                     "出路：逐个看是不是你起的那一趟，是的用完就删（rm -rf -- <路径>），不是的别动")
    own, unknown = [], []
    for path, born in sorted(births.items()):
        if born is None:
            continue
        state, owner = run_owner(path, born)
        if state == "running":
            continue
        in_window = (started is None or born >= started - WINDOW_SLACK_SECONDS) and any(start <= born <= end for start, end in windows)
        if broken("attribution-by-window"):
            if in_window:
                own.append(path)
            continue
        if owner is not None and owner.get("session") == session:
            if role == "main" or broken("subagent-run-session-level") or owner.get("launcher_sha256") in own_digests:
                own.append(path)
            continue
        if owner is None and in_window:
            unknown.append(path)
    return own, unknown


# ── 迁库遗留（文件头「判哪几处」迁库遗留那一段）
def client_species(path):
    """prune_server_client.py 文件头「键空间」一节里「例 / 种类固定 / 种类是」后面的名字（一节从行首是「键空间」的那一行起，到下一行顶格的为止）。"""
    with open(path, encoding="utf-8") as handle:
        lines = handle.read().splitlines()
    section, inside = [], False
    for line in lines:
        if line.startswith(CLIENT_KEY_SPACE_HEADING):
            inside = True
        elif inside and line and not line[0].isspace():
            break
        if inside:
            section.append(line)
    names = []
    for match in re.finditer(rf"(?:例|种类固定|种类是)\s*({SPECIES_TOKEN}(?:/[^\s、，）]*)?(?:\s*、\s*{SPECIES_TOKEN})*)", "\n".join(section)):
        names += [part.split("/")[0] for part in re.split(r"\s*、\s*", match.group(1))]
    return names


def registry_species(path):
    """verdict-library-scripts.tsv 第三列里「键空间 <族>/<种类>/」与「<族> 族 <种类>/」的种类。"""
    names = []
    with open(path, encoding="utf-8") as handle:
        for line in handle:
            columns = line.rstrip("\n").split("\t")
            if line.startswith("#") or len(columns) < 3:
                continue
            for match in re.finditer(rf"键空间 {SPECIES_TOKEN}/({SPECIES_TOKEN})/|{SPECIES_TOKEN} 族 ({SPECIES_TOKEN})/", columns[2]):
                names.append(match.group(1) or match.group(2))
    return names


def retired_species(path):
    """retired-names.tsv 里种类是「目录名」的那几行的名字（退役的旧库目录名）。"""
    names = []
    with open(path, encoding="utf-8") as handle:
        for line in handle:
            columns = line.rstrip("\n").split("\t")
            if not line.startswith("#") and len(columns) >= 2 and columns[0] and columns[1] == RETIRED_DIRECTORY_KIND:
                names.append(columns[0])
    return names


def library_species(notes):
    """旧文件库与库备份认的种类名：SPECIES_SOURCES 三处现读、合起来去重。哪一处读不了或一个都读不出，往 notes 记一处没判全
    （弄坏开关 leftover-unjudged-unnoted 下不记）。"""
    readers = (("client", client_species, f"文件头「{CLIENT_KEY_SPACE_HEADING}」一节"), ("registry", registry_species, "第三列的键空间"),
               ("retired", retired_species, f"种类是「{RETIRED_DIRECTORY_KIND}」的行"))
    names = []
    for key, reader, where in readers:
        path = SPECIES_SOURCES[key]
        try:
            found, reason = reader(path), "一个种类名都没读出"
        except (OSError, UnicodeDecodeError) as error:
            found, reason = [], f"读不了（{type(error).__name__}: {error}）"
        if not found and not broken("leftover-unjudged-unnoted"):
            notes.append(f"种类名读不出：{path} {where}{reason}，旧文件库与带种类名的库备份认不全；出路：把那份文件放回仓里、照它原来的写法写，再收工")
        names += found
    return tuple(dict.fromkeys(names))


def prune_service_directories(notes):
    """判定库服务的状态目录与服务自己那份库：交 ([状态目录…], 服务那份库或 None)。取法照 prune-server/control：单元文件的 --state-directory
    （unit_library），读不到单元文件时取它的缺省状态目录；两处不同的都交。control 载不进、单元文件在而读不了往 notes 记一处没判全；
    弄坏开关 unit-file-ignored 下不读单元文件。"""
    noting = not broken("leftover-unjudged-unnoted")
    try:
        control = module("prune_server_control")
        paths = control.Paths()
    except Exception as error:  # noqa: BLE001 —— 载不进就记一处没判全，别的照判
        if noting:
            notes.append(f"读不到 {MODULE_PATHS['prune_server_control']}（{error!r}），判定库服务状态目录里的迁库遗留没判；出路：把 prune-server/ 放回仓里再收工")
        return [], None
    library = None
    if not broken("unit-file-ignored"):
        library, origin = control.unit_library(paths)
        if library is None and os.path.lexists(paths.unit_file) and noting:
            notes.append(f"判定库服务的单元文件读不了（{origin}），只看了缺省的状态目录 {paths.state_directory}；出路：ls -l {paths.unit_file} 看权限，修好再收工")
    if library is None:
        library = os.path.join(paths.state_directory, control.LIBRARY_DIRECTORY_NAME)
    library = os.path.normpath(library)
    return list(dict.fromkeys([os.path.dirname(library), os.path.normpath(paths.state_directory)])), library


def with_ancestors(paths):
    every = set()
    for path in paths:
        current = os.path.normpath(path)
        while True:
            every.add(current)
            parent = os.path.dirname(current)
            if parent == current:
                break
            current = parent
    return every


def leftover_places(judged_roots, notes):
    """迁库遗留扫哪几处：交 ([(根, 最深几层, 是不是工作根)], 不进的目录, 服务那份库或 None, 锚点与它们的上层, 数据根或 None)。
    弄坏开关 data-root-leftovers-unseen 下不看数据根与崩溃放量的家目录，prune-state-directory-unseen 下不看判定库服务的状态目录，
    no-owner-directories-skipped 下不看工作根下没写归属文件的目录，service-library-listed 下服务那份库照进。"""
    library = module("lib_work_root")
    base = os.path.normpath(work_root())
    runs = [os.path.normpath(path) for path in run_roots()]
    home = os.path.normpath(os.environ.get(CRASH_AMPLIFICATION_HOME_VARIABLE) or os.path.join(base, CRASH_AMPLIFICATION_DIRECTORY_NAME))
    places, anchors, data = [], [library.home_directory(), base, home, *runs], None
    if not broken("data-root-leftovers-unseen"):
        try:
            data = os.path.normpath(library.shared_data_root())
        except ValueError as error:
            if not broken("leftover-unjudged-unnoted"):
                notes.append(f"数据根取不到（{error}），数据根下的迁库遗留没判；出路：{library.DATA_ROOT_HOWTO}")
        if data is not None:
            places.append((data, LEFTOVER_SCAN_MAX_DEPTH, False))
            anchors.append(data)
        if not any(home == root or is_strictly_under(home, root) for root in (data, base) if root):
            places.append((home, LEFTOVER_SCAN_MAX_DEPTH, False))
    state_directories, service = prune_service_directories(notes)
    anchors += state_directories
    if not broken("prune-state-directory-unseen"):
        places += [(directory, PRUNE_STATE_SCAN_MAX_DEPTH, False) for directory in state_directories]
    if not broken("no-owner-directories-skipped"):
        places.append((base, LEFTOVER_SCAN_MAX_DEPTH, True))
    skipped = {base, *runs, *(root for root, _, _ in places), *(os.path.normpath(root[0]) for root in judged_roots)}
    if service is not None and not broken("service-library-listed"):
        skipped.add(service)
    return places, skipped, service, with_ancestors(anchors), data


def substituted_work_roots(command):
    """$(python3 …/lib_work_root.py [data-root | shared <名>]) 换成它打的那一处；取不到的原样留着（后面认不出、不算点到）。"""
    library = module("lib_work_root")

    def replace(match):
        try:
            if match.group(1) is None:
                return library.work_root()
            if match.group(2):
                return library.shared_data_path(match.group(2))
            return library.shared_data_root()
        except ValueError:
            return match.group(0)
    return WORK_ROOT_SUBSTITUTION.sub(replace, command)


def command_words(command):
    """一条 Bash 命令拆成词：shlex（带 shell 符号）拆得开照它，拆不开（引号不成对、here-doc 里的散字）照空白与 shell 符号切。
    段的分隔（SEGMENT_SEPARATOR：| || && ; & 括号与换行）两种拆法都留作单独的词。"""
    try:
        lexer = shlex.shlex(command, posix=True, punctuation_chars=SHELL_PUNCTUATION)
        lexer.whitespace = " \t\r\f\v"
        lexer.whitespace_split = True
        return list(lexer)
    except ValueError:
        return [word for word in LOOSE_WORDS.split(command) if word]


def read_only_flags(words):
    """每个词在不在只读的那一段里：段照 SEGMENT_SEPARATOR 切，命令词是起头的 NAME=值 之后那个词的最后一节文件名，
    在 READ_ONLY_COMMANDS 里（find 另要不带 FIND_MODIFYING_PREDICATES）就是只读。弄坏开关 read-only-mentions-counted 下一段都不算只读。"""
    flags, start = [False] * len(words), 0
    for end in [*(index for index, word in enumerate(words) if SEGMENT_SEPARATOR.fullmatch(word)), len(words)]:
        segment = words[start:end]
        command = next((word for word in segment if not VARIABLE_ASSIGNMENT.match(word)), None)
        name = os.path.basename(command) if command is not None else None
        if (name in READ_ONLY_COMMANDS and not (name == "find" and FIND_MODIFYING_PREDICATES.intersection(segment))
                and not broken("read-only-mentions-counted")):
            flags[start:end] = [True] * len(segment)
        start = end + 1
    return flags


def resolved_word(word, variables, directory):
    """一个词 → 它点的绝对路径（normpath）；认不出交 None。相对路径照 directory（同一条命令里前面 cd 到的）解，弄坏开关 cd-ignored 下不解。"""
    text = VARIABLE_REFERENCE.sub(lambda match: variables.get(match.group(1) or match.group(2), match.group(0)), word)
    text = module("lib_work_root").expand_written_path(text)
    if not text or "$" in text:
        return None
    if not os.path.isabs(text):
        if directory is None or broken("cd-ignored"):
            return None
        text = os.path.join(directory, text)
    return os.path.normpath(text)


def mentioned_paths(calls, names=None):
    """自己的 Bash 命令点到的路径：(点到的绝对路径集合, 带通配的列表)。names 给了只拆文字里带其中一个名字的命令（别的命令点不到要找的那几处）。
    只读的那一段（read_only_flags）里的路径不收，赋值与 cd 照记（后面的段要用）。"""
    exact, patterns = set(), []
    for call in calls:
        if call["name"] != "Bash":
            continue
        command = str(call["input"].get("command") or "")
        if names is not None and not any(name in command for name in names):
            continue
        words = command_words(substituted_work_roots(command))
        read_only = read_only_flags(words)
        variables, directory = {}, None
        for index, word in enumerate(words):
            assignment = VARIABLE_ASSIGNMENT.match(word)
            values = [assignment.group(2)] if assignment else ([] if word.startswith("-") or SEGMENT_SEPARATOR.fullmatch(word) else [word])
            if not assignment and "=" in word:
                values.append(word.split("=", 1)[1])
            if read_only[index]:
                values = []
            for value in values:
                path = resolved_word(value, variables, directory)
                if path is None:
                    continue
                if GLOB_CHARACTERS.search(path):
                    patterns.append(path)
                else:
                    exact.add(path)
            if assignment:
                value = resolved_word(assignment.group(2), variables, directory)
                if value is not None:
                    variables[assignment.group(1)] = value
            if index > 0 and words[index - 1] in DIRECTORY_CHANGERS:
                value = resolved_word(word, variables, directory)
                if value is not None and not GLOB_CHARACTERS.search(value):
                    directory = value
    return exact, patterns


def glob_matches(path, pattern):
    path_parts, pattern_parts = path.split("/"), pattern.split("/")
    return len(path_parts) == len(pattern_parts) and all(fnmatch.fnmatchcase(part, wanted) for part, wanted in zip(path_parts, pattern_parts))


def literal_prefix(pattern):
    """通配里头一个带 * ? [ 的那一层之前的那一截。"""
    parts = pattern.split("/")
    for index, part in enumerate(parts):
        if GLOB_CHARACTERS.search(part):
            return "/".join(parts[:index]) or "/"
    return pattern


def point_attributed(path, exact, patterns, anchors):
    """path 本身或它的上层（到锚点为止，锚点本身不算）被自己的 Bash 命令点到过。弄坏开关 unmentioned-counted-own 下都算，
    anchor-mentions-counted 下锚点与字面前缀是锚点的通配也算，mention-ancestors-ignored 下只认 path 本身。"""
    if broken("unmentioned-counted-own"):
        return True
    anchors_counted = broken("anchor-mentions-counted")
    usable = [pattern for pattern in patterns if anchors_counted or literal_prefix(pattern) not in anchors]
    current = path
    while True:
        if current in anchors and not anchors_counted:
            return False
        if current in exact or any(glob_matches(current, pattern) for pattern in usable):
            return True
        parent = os.path.dirname(current)
        if broken("mention-ancestors-ignored") or parent == current:
            return False
        current = parent


def migration_leftovers(role, main_transcript, calls, judged_roots, exempt, budget, notes):
    """迁库遗留：交 (自己的 [(路径, 种类)], 认不出是谁的 [(路径, 种类)])，在不在用留给调用方判。归属只看自己的会话记录
    （弄坏开关 leftover-mentions-session-level 下子 agent 另算主会话记录里的命令）。"""
    places, skipped, _service, anchors, data = leftover_places(judged_roots, notes)
    if data is not None and not broken("registered-entries-listed"):
        library = module("lib_work_root")
        budget["not_by_name"] = {os.path.join(data, name) for name in (*library.SHARED_DATA_ENTRIES, *library.PER_HOST_DATA_ENTRIES)}
    found = {}
    for root, depth, is_work_root in places:
        if os.path.islink(root) or not os.path.isdir(root):
            continue

        def skip(path, level, is_work_root=is_work_root):
            if path in skipped:
                return True
            return (is_work_root and level == 1 and owner_session(path) is not None
                    and not broken("owned-directories-leftover-scanned"))
        for path, kind in scan(root, depth, exempt, set(), budget, LEFTOVER_KINDS, skip):
            found.setdefault(path, kind)
    if not found:
        return [], []
    names = None
    if not broken("anchor-mentions-counted"):
        names = set()
        for path in found:
            current = path
            while current not in anchors and os.path.dirname(current) != current:
                names.add(os.path.basename(current))
                current = os.path.dirname(current)
    own_calls = list(calls)
    if role == "subagent" and broken("leftover-mentions-session-level"):
        own_calls += module("session_transcript").tool_calls(main_transcript)[0]
    exact, patterns = mentioned_paths(own_calls, names)
    own, unknown = [], []
    for path, kind in sorted(found.items()):
        (own if point_attributed(path, exact, patterns, anchors) else unknown).append((path, kind))
    return own, unknown


def shown_unknown(items):
    """「认不出是谁的」里列哪几处迁库遗留：在用的去掉，最多 UNKNOWN_SHOWN_MAX 处；交 (列的, 没核、没列的个数)。"""
    kept = []
    for index, item in enumerate(items):
        if len(kept) >= UNKNOWN_SHOWN_MAX:
            return kept, len(items) - index
        if broken("open-files-ignored") or not in_use(item[0]):
            kept.append(item)
    return kept, 0


# ── 在跑的活
def launched_script(command):
    """Bash 工具外包装去掉之后，这条命令跑的是哪个脚本（basename）；是复合命令或认不出交 None。"""
    shown = module("agent_watch").command_for_display(command)
    if re.search(r"&&|\|\||[;|&]", shown):
        return None
    try:
        words = shlex.split(shown)
    except ValueError:
        words = shown.split()
    while words and (re.match(r"^[A-Za-z_][A-Za-z0-9_]*=", words[0]) or os.path.basename(words[0]) in LAUNCHER_WORDS or words[0].startswith("-")):
        words = words[1:]
    return os.path.basename(words[0]) if words else None


def background_work(transcript):
    """自己起的、还有进程开着输出文件的后台任务（循环提醒与看门狗不算）：[(任务号, 命令)]。"""
    if broken("background-ignored"):
        return []
    work = []
    for task_id, _, tops in module("agent_watch").own_background_tasks_still_running(transcript):
        commands = [command for _, _, command, _ in tops]
        if (commands and not broken("reminder-counts-as-running")
                and all(launched_script(command) in TIMER_AND_WATCHDOG_SCRIPTS for command in commands)):
            continue
        work.append((task_id, commands[0] if commands else ""))
    return work


def running_subagents(main_transcript, now):
    if broken("subagents-ignored"):
        return []
    paths = glob.glob(os.path.join(main_transcript[:-len(".jsonl")], "subagents", "agent-*.jsonl"))
    recent = []
    for path in paths:
        try:
            modified = os.stat(path).st_mtime
        except OSError:
            continue
        if now - modified <= SUBAGENT_IDLE_SECONDS:
            recent.append((path, modified))
    if not recent:
        return []
    stops, notifications, _ = module("agent_watch").session_task_events(main_transcript)
    running = []
    for path, modified in recent:
        agent_id = os.path.basename(path)[len("agent-"):-len(".jsonl")]
        _, notified_at = notifications.get(agent_id, (None, None))
        stopped_at = stops.get(agent_id)
        ended = any(moment is not None and modified <= moment.timestamp() + NOTIFICATION_SLACK_SECONDS for moment in (notified_at, stopped_at))
        if ended and not broken("notifications-ignored"):
            continue
        running.append(agent_id)
    return running


# ── 在用
def task_server_database(notes):
    """服务库的路径：COMPLETION_CLEANUP_TASK_SERVER_DATABASE 设了照它（自证用）；别的取 task_server_lib.py 的 read_configuration，读不了取 default_values。
    共用模块载不进记一处没判全、交 None。"""
    override = os.environ.get(TASK_SERVER_DATABASE_VARIABLE)
    if override:
        return override
    try:
        library = module("task_server_lib")
    except Exception as error:
        notes.append(f"读不到 {MODULE_PATHS['task_server_lib']}（{error!r}），服务给的编译目录没判；出路：把 task-server/ 放回仓里，"
                     f"自己提交过的任务逐个 {TASK_SERVER_CLIENT} status <号> 看末行 build= 再 {TASK_SERVER_CLIENT} clean <号>")
        return None
    try:
        return library.read_configuration().database
    except Exception:
        try:
            defaults = library.default_values(dict(os.environ))
            return os.path.join(defaults["TASK_SERVER_STATE_DIRECTORY"], library.DATABASE_NAME)
        except Exception as error:
            notes.append(f"算不出服务库的位置（{error!r}），服务给的编译目录没判；出路：{TASK_SERVER_CLIENT} status 看自己的任务，逐个 {TASK_SERVER_CLIENT} clean <号>")
            return None


def service_builds(role, session, notes):
    """本会话已有结局、编译目录还在盘上的 task-server 任务：[(编译目录, 删它用的任务号)]（读法在文件头「判哪几处」服务给的编译目录那一段）。
    只判主 agent，算本会话的全部；子 agent 不交任务、不起 task clean，交一张空表。弄坏开关 service-builds-ignored 下一个都不认，service-builds-session-unchecked 下
    别的会话的也算，service-builds-unfinished-listed 下在排、在跑的也算，subagent-service-builds-judged 下子 agent 也算本会话的全部。"""
    if broken("service-builds-ignored"):
        return []
    if role != "main" and not broken("subagent-service-builds-judged"):
        return []
    database = task_server_database(notes)
    if database is None or not os.path.exists(database):
        return []
    any_session = broken("service-builds-session-unchecked")
    where, parameters = ("1", ()) if any_session else ("session = ?", (session,))
    try:
        connection = sqlite3.connect(f"file:{urllib.parse.quote(os.path.abspath(database))}?mode=ro", uri=True, timeout=5)
        try:
            tasks = {row[0]: (row[1], row[2]) for row in connection.execute(f"SELECT id, state, record FROM tasks WHERE {where}", parameters)}
            attempts = connection.execute(f"SELECT task_id, record FROM attempts WHERE {where}", parameters).fetchall()
        finally:
            connection.close()
    except sqlite3.Error as error:
        notes.append(f"服务库 {database} 读不了（{error}），服务给的编译目录没判；出路：{TASK_SERVER_CLIENT} status 看自己的任务，"
                     f"逐个 {TASK_SERVER_CLIENT} clean <号>；库读不了是服务那一侧的事，交主 agent 看 python3 task-server/control check")
        return []

    def record_of(text):
        try:
            decoded = json.loads(text)
        except (TypeError, ValueError):
            return {}
        return decoded if isinstance(decoded, dict) else {}

    owner = {}   # 任务号 → 删它用的任务号（分片的子任务交父任务号：task clean <父任务号> 连子任务一起删）
    for task_id, (_state, text) in tasks.items():
        parent = record_of(text).get("parent_id")
        owner[task_id] = parent if isinstance(parent, int) and parent in tasks else task_id
    found = {}
    for task_id, text in attempts:
        top = owner.get(task_id)
        if top is None:
            continue
        if tasks[top][0] != TASK_SERVER_FINISHED_STATE and not broken("service-builds-unfinished-listed"):
            continue
        directory = record_of(text).get("build_directory")
        if isinstance(directory, str) and directory and os.path.isdir(directory):
            found.setdefault(os.path.abspath(directory), top)
    return sorted(found.items())


def file_held(path):
    """有本进程以外的进程开着这一份文件：/proc/<pid>/fd 或 maps 指着它（库备份是文件，open_holders 只认目录；只看得到本用户的）。"""
    real = os.path.realpath(path)
    own = os.getpid()
    for entry in os.listdir("/proc"):
        if not entry.isdigit() or int(entry) == own:
            continue
        base = os.path.join("/proc", entry)
        try:
            descriptors = os.listdir(os.path.join(base, "fd"))
        except OSError:
            descriptors = []
        for descriptor in descriptors:
            try:
                if os.readlink(os.path.join(base, "fd", descriptor)) == real:
                    return True
            except OSError:
                pass
        try:
            with open(os.path.join(base, "maps"), encoding="utf-8", errors="replace") as handle:
                if any(len(line.split(None, 5)) == 6 and line.split(None, 5)[5].strip() == real for line in handle):
                    return True
        except OSError:
            pass
    return False


def in_use(path):
    """有进程开着 path 里的文件（目录照 scripts/hosts/run_build_directory.py 的 open_holders，与它的 holders 同一份判法；文件照 file_held）。"""
    if not os.path.isdir(path) and os.path.lexists(path):
        return file_held(path)
    return bool(module("run_build_directory").open_holders(os.path.abspath(path)))


def not_in_use(found_in_roots, found_in_runs):
    """认领目录与草稿目录里看列出的那一处；运行目录看整个运行目录（弄坏开关 run-holders-leaf-only 时只看列出的那一处）。"""
    if broken("open-files-ignored"):
        return found_in_roots + [item for items in found_in_runs.values() for item in items]
    kept = [(path, kind) for path, kind in found_in_roots if not in_use(path)]
    for run_directory, items in found_in_runs.items():
        if broken("run-holders-leaf-only"):
            kept += [(path, kind) for path, kind in items if not in_use(path)]
        elif items and not in_use(run_directory):
            kept += items
    return kept


# ── 单子
def sizes(paths):
    if not paths:
        return {}
    try:
        completed = subprocess.run(["timeout", str(DU_TIMEOUT_SECONDS), "du", "-sh", "--", *paths], capture_output=True, text=True, check=False)
    except OSError:
        return {}
    measured = {}
    for line in completed.stdout.splitlines():
        size, separator, path = line.partition("\t")
        if separator:
            measured[path] = size
    return measured


def removal_command(path, kind):
    if kind == KIND_SERVICE_BUILD and not broken("service-build-removal-by-rm"):
        return f"{TASK_SERVER_CLIENT} clean {SERVICE_BUILD_TASKS.get(path, '<号>')}"
    if kind == KIND_WORKTREE:
        try:
            pointer = open(os.path.join(path, ".git"), encoding="utf-8").read().strip()
        except OSError:
            pointer = ""
        source = pointer[len("gitdir: "):].split("/.git/worktrees/")[0] if pointer.startswith("gitdir: ") and "/.git/worktrees/" in pointer else ""
        if source:
            return f"git -C {shlex.quote(source)} worktree remove --force {shlex.quote(path)}"
    return f"rm -rf -- {shlex.quote(path)}"


def leftover_lines(leftovers):
    measured = {} if broken("size-omitted") else sizes([path for path, _ in leftovers])
    lines = []
    for path, kind in leftovers:
        lines.append(f"  {kind}  {path}" if broken("size-omitted") else f"  {kind}  {path}  {measured.get(path, '?')}")
    return lines


def unknown_lines(unknown, omitted=0):
    if not unknown and not omitted:
        return []
    lines = [f"「{UNKNOWN_SECTION_TITLE}」（只报不拦：认不出是不是你建的。运行目录：建在你某次 Bash 调用的时间窗里，但起它的进程已经不在、没记账、也没有归属记录；"
             "迁库遗留：在数据根、判定库服务的状态目录或没有归属文件的工作根目录里，你的 Bash 命令没点到它和它的上层目录；"
             "删之前先核是谁的，不是你的别动）：", *leftover_lines(unknown)]
    if omitted:
        lines.append(f"  另有 {omitted} 处迁库遗留没列（这一节最多列 {UNKNOWN_SHOWN_MAX} 处，没核在不在用）")
    return lines


def block_text(role, leftovers, problems, notes, unknown=(), omitted=0, place_paths=()):
    if role == "main":
        head = ("✗ 完工清理：这一轮做完了（没有在跑的后台任务与子 agent），本会话认领的目录与它起的运行目录里还剩下面这些仓副本、编译目录与库拷贝，"
                "或本会话点到过的迁库遗留还在；用完就删，删完再收工。")
        rule = ".claude/main-agent.md「一轮怎么开、怎么收」第 11 条"
        kinds = "报告、补丁、备份"
    else:
        head = "✗ 完工清理：交回之前先删掉你自己认领、自己建的仓副本、编译目录与库拷贝，与你的命令点到过的迁库遗留（下面这些还在）；用完就删，删完再交回。"
        rule = ".claude/agent-common.md「写」一节"
        kinds = "报告、补丁、备份、交接（交给主 agent 接着用的副本，报告里写明主 agent 在里面跑什么）"
    lines = [head, *leftover_lines(leftovers),
             "→ 怎么办：逐个删（删之前 du -sh 记下的大小写进报告），删法：",
             *[f"    {command}" for command in dict.fromkeys(removal_command(path, kind) for path, kind in leftovers)],
             f"   不留给以后的趟复用，也不等别的活来用（{rule}）。",
             f"   报告、补丁与打补丁前的备份是普通文件的不在单上；要留的副本写进它所在认领目录的 {DECLARATION_FILE}：一行「<相对路径> <种类> <为什么留>」，"
             f"种类是 {kinds}；编译目录声明了也照列。写法在 {HOOK_PATH} 文件头「要留的交付物」。"]
    if any(kind == KIND_SERVICE_BUILD for _, kind in leftovers):
        lines.append(f"   {KIND_SERVICE_BUILD}是 task-server 给的（wait 末行 build=）：只经 `{TASK_SERVER_CLIENT} clean <号>` 删，不直接 rm（项目代码闸照拒直接删）；"
                     "它删前删后报大小，记进报告。")
    if any(path in place_paths for path, _ in leftovers):
        lines.append(f"   数据根、判定库服务的状态目录与没有归属文件的工作根目录里的迁库遗留（{KIND_LIBRARY}、{KIND_OLD_LIBRARY}、{KIND_LIBRARY_ARCHIVE}）"
                     f"是你的命令点到过的：判定已在 prune-server 那一份里，这些不靠声明留；真要留，先问用户。")
    lines += [f"   ! {problem}" for problem in problems + notes]
    lines += unknown_lines(list(unknown), omitted)
    return "\n".join(lines)


def release_text(leftovers, unknown=(), omitted=0):
    return "\n".join(["完工清理闸：拦下之后这几项一个没删，这一次放行收工，交用户定：", *leftover_lines(leftovers), *unknown_lines(list(unknown), omitted)])


def state_path(session, actor):
    return os.path.join(module("lib_work_root").state_directory(STATE_DIRECTORY_NAME), f"{session}-{actor}.json")


def load_state(path):
    try:
        return json.load(open(path, encoding="utf-8"))
    except (OSError, ValueError):
        return {}


def save_state(path, paths):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    temporary = f"{path}.{os.getpid()}.tmp"
    with open(temporary, "w", encoding="utf-8") as handle:
        json.dump({"paths": paths}, handle, ensure_ascii=False)
    os.replace(temporary, path)


def clear_state(path):
    try:
        os.remove(path)
    except OSError:
        pass


# ── 判
def decide(hook_input):
    """钩子输入 → {"action": pass / block / release / skip, "text": …, "paths": […]}。"""
    if not isinstance(hook_input, dict):
        return {"action": "skip", "text": "钩子输入不是 JSON 对象"}
    event = hook_input.get("hook_event_name")
    session = str(hook_input.get("session_id") or "")
    main_transcript = str(hook_input.get("transcript_path") or "")
    if event == "Stop":
        role, actor, own_transcript = "main", "main", main_transcript
    elif event == "SubagentStop":
        role, actor, own_transcript = "subagent", str(hook_input.get("agent_id") or ""), str(hook_input.get("agent_transcript_path") or "")
        if not actor and own_transcript:
            actor = os.path.basename(own_transcript)[len("agent-"):-len(".jsonl")]
    elif event == "PreToolUse":
        actor = str(hook_input.get("agent_id") or "")
        if hook_input.get("tool_name") != HANDBACK_TOOL or not actor or actor == "main":
            return {"action": "pass"}
        role = "subagent"
        own_transcript = f"{main_transcript[:-len('.jsonl')]}/subagents/agent-{actor}.jsonl" if main_transcript.endswith(".jsonl") else ""
    else:
        return {"action": "pass"}
    if not NAME_PATTERN.match(session) or not NAME_PATTERN.match(actor):
        return {"action": "skip", "text": "认不出会话号或子 agent 号"}
    if not own_transcript or not os.path.isfile(own_transcript) or (role == "main" and not main_transcript.endswith(".jsonl")):
        return {"action": "skip", "text": f"找不到会话记录（{own_transcript or '输入里没有'}）"}
    if not os.access(own_transcript, os.R_OK) and not broken("transcript-unreadable-unnoted"):   # 在而读不了：读它的共用模块静默交空表，认领与草稿目录都认不出（当没有、照常放行）
        return {"action": "skip", "text": f"会话记录 {own_transcript} 在而读不了，认领与派发提示点名的草稿目录都认不出"}
    now = time.time()
    calls, _ = module("session_transcript").tool_calls(own_transcript)
    notes = []
    roots = main_roots(session, main_transcript, notes) if role == "main" or broken("subagent-sees-whole-session") else subagent_roots(session, own_transcript, calls)
    exempt, problems = declarations([directory for root in roots for directory in (root[0], root[1])], role)
    species = library_species(notes)
    budget = {"visited": 0, "truncated": set(), "limit": scan_limit(), "unreadable": [], "species": species}
    found_in_roots = []
    for path, _, skipped in roots:
        found_in_roots += scan(path, SCAN_MAX_DEPTH, exempt, skipped, budget)
    own_runs, unknown_runs = classify_runs(role, session, own_transcript, calls, now, notes)
    found_in_runs = {path: scan(path, RUN_SCAN_MAX_DEPTH, {}, set(), budget) for path in own_runs}
    found_unknown = {path: scan(path, RUN_SCAN_MAX_DEPTH, {}, set(), budget) for path in unknown_runs}
    if budget["truncated"]:
        notes.append(f"看了 {budget['limit']} 个目录到顶，这几处没看完：{' '.join(sorted(budget['truncated']))}；"
                     f"出路：这几处里不用的删掉（目录少了下一次才看得完），要留的在它所在认领目录的 {DECLARATION_FILE} 里登记（写法在 {HOOK_PATH} 文件头「要留的交付物」）")
    if budget["unreadable"]:
        shown = budget["unreadable"]
        notes.append(f"读不了的目录 {len(shown)} 个没看（里面有没有副本、编译目录与库拷贝没判）：{' '.join(shown[:5])}{' …' if len(shown) > 5 else ''}；"
                     "出路：ls -ld 看权限，不用的删掉，要留的在它所在认领目录的 .deliverables 里登记")
    # 迁库遗留另用一份扫描上限：上面那几处的没判全照旧单独一句，不与这一类混在一起
    leftover_budget = {"visited": 0, "truncated": set(), "limit": scan_limit(), "unreadable": [], "species": species}
    own_leftovers, unknown_leftovers = migration_leftovers(role, main_transcript, calls, roots, exempt, leftover_budget, notes)
    if leftover_budget["truncated"]:
        notes.append(f"迁库遗留看了 {leftover_budget['limit']} 个目录到顶，这几处没看完：{' '.join(sorted(leftover_budget['truncated']))}；"
                     "出路：这几处里不用的旧库与备份删掉（目录少了下一次才看得完）")
    if leftover_budget["unreadable"]:
        shown = leftover_budget["unreadable"]
        notes.append(f"迁库遗留那几处有 {len(shown)} 个目录读不了、没看（里面有没有旧库与备份没判）：{' '.join(shown[:5])}{' …' if len(shown) > 5 else ''}；"
                     "出路：ls -ld 看权限，不用的删掉")
    service = service_builds(role, session, notes)
    SERVICE_BUILD_TASKS.update(service)
    leftovers = sorted(dict(not_in_use(found_in_roots + own_leftovers, found_in_runs) + [(path, KIND_SERVICE_BUILD) for path, _ in service]).items())
    listed = {path for path, _ in leftovers}
    leftover_unknown, omitted = shown_unknown([item for item in unknown_leftovers if item[0] not in listed])
    unknown = sorted(dict(not_in_use([], found_unknown) + leftover_unknown).items())
    state_file = state_path(session, actor)
    if not leftovers:
        clear_state(state_file)
        return {"action": "pass", "notes": notes, "unknown": unknown, "unknown_omitted": omitted}
    if background_work(own_transcript) or (role == "main" and running_subagents(main_transcript, now)):
        return {"action": "pass", "running": True}
    paths = [path for path, _ in leftovers]
    place_paths = {path for path, _ in own_leftovers}
    if event != "PreToolUse" and hook_input.get("stop_hook_active") is True and not broken("release-never"):
        if load_state(state_file).get("paths") == paths or broken("release-on-any-continuation"):
            return {"action": "release", "text": release_text(leftovers, unknown, omitted), "paths": paths, "unknown": unknown}
    save_state(state_file, paths)
    return {"action": "block", "text": block_text(role, leftovers, problems, notes, unknown, omitted, place_paths), "paths": paths, "unknown": unknown}


def report_unjudged_pass(hook_input, notes, stream, detections_path=None):
    """放行而没判全（扫到上限没看完、运行目录取不到创建时间）：照「放行但必记检出」往 stream 逐条打出、检出记录追加一行（结局「只记」），看门狗据此报给主 agent。
    detections_path 不给取 AGENT_HOOK_DETECTIONS，没设取 lib_work_root.py 的 default_detections_path。交打出的行数；弄坏开关 unjudged-pass-silent 下不打、不记。"""
    if not notes or broken("unjudged-pass-silent"):
        return 0
    recorded = record_detection(hook_input, [f"{UNJUDGED_FINDING_PREFIX}：{note}" for note in notes], detections_path)
    lines = ["! 完工清理闸：这一次没判全，照「放行但必记检出」照常放行：", *[f"   ! {note}" for note in notes], f"   {recorded}"]
    print("\n".join(lines), file=stream)
    return len(lines)


def record_detection(hook_input, findings, detections_path=None):
    """检出记录追加一行（结局「只记」）：report_unjudged_pass 与 report_unjudged 共用。detections_path 不给取 AGENT_HOOK_DETECTIONS，
    没设取 lib_work_root.py 的 default_detections_path。交一句给人看的说明：记进了哪里，或为什么没记成。"""
    hook_input = hook_input if isinstance(hook_input, dict) else {}
    event = str(hook_input.get("hook_event_name") or "")
    path = detections_path or os.environ.get(DETECTIONS_VARIABLE)
    if not path:
        try:
            path = module("lib_work_root").default_detections_path()
        except Exception:  # noqa: BLE001 —— 读不到工作根的定义：不知道往哪写，下面照打一句
            path = None
    entry = {"time": datetime.now(timezone.utc).isoformat(), "session_id": hook_input.get("session_id") or "读不到", "agent_id": hook_input.get("agent_id"),
             "agent_type": hook_input.get("agent_type") or ("主 agent" if event == "Stop" else "子 agent"),
             "transcript_path": hook_input.get("transcript_path"), "command": f"完工清理闸（{event or '事件读不到'}）",
             "findings": list(findings), "outcome": "只记"}
    if not path:
        return f"检出没记成：取不到检出记录的位置（{DETECTIONS_VARIABLE} 没设，lib_work_root.py 也读不到）"
    try:
        os.makedirs(os.path.dirname(path) or ".", exist_ok=True)
        with open(path, "a", encoding="utf-8") as handle:
            handle.write(json.dumps(entry, ensure_ascii=False) + "\n")
    except OSError as error:
        return f"检出没记成：写不了 {path}（{error}）"
    return f"检出记进 {path}"


def report_unjudged(hook_input, text, stream, detections_path=None):
    """判不了（输入不是 JSON 对象、认不出会话号、找不到会话记录、导入不了共用模块、自己出错）：照「放行但必记检出」往 stream 打一句「没判」、
    检出记录追加一行（结局「只记」，findings 以 UNJUDGED_SKIP_PREFIX 起头），看门狗据此报给主 agent。弄坏开关 unjudged-skip-silent 下只打那一句、不记。"""
    if broken("unjudged-skip-silent"):
        print(f"! 完工清理闸：{text}，这一次没判（放行）", file=stream)
        return
    recorded = record_detection(hook_input, [f"{UNJUDGED_SKIP_PREFIX}：{text}"], detections_path)
    print(f"! 完工清理闸：{text}，这一次没判（放行），照「放行但必记检出」{recorded}", file=stream)


def hook_main():
    raw = sys.stdin.buffer.read().decode("utf-8", "replace")
    try:
        hook_input = json.loads(raw)
    except ValueError as error:
        report_unjudged({}, f"钩子输入不是合法的 JSON（{type(error).__name__}: {error}；读到 {len(raw)} 个字符，前 {INPUT_PREFIX_BYTES} 个：{raw[:INPUT_PREFIX_BYTES]!r}）",
                        sys.stderr)
        return 0
    try:
        outcome = decide(hook_input)
    except Exception as error:  # noqa: BLE001 —— 自己出错不拦，免得一个缺陷挡住所有收工
        print("! 完工清理闸自己出错了，这一次没判（放行）：", file=sys.stderr)
        traceback.print_exc(file=sys.stderr)
        report_unjudged(hook_input, f"自己出错（{type(error).__name__}: {error}）", sys.stderr)
        return 0
    action = outcome["action"]
    if action == "pass" and outcome.get("notes"):
        report_unjudged_pass(hook_input, outcome["notes"], sys.stderr)
        return 0
    if action == "skip":
        report_unjudged(hook_input, outcome["text"], sys.stderr)
        return 0
    if action == "release":
        print(json.dumps({"systemMessage": outcome["text"]}, ensure_ascii=False))
        return 0
    if action == "block":
        if hook_input.get("hook_event_name") == "PreToolUse":
            print(outcome["text"], file=sys.stderr)
            return 2
        print(json.dumps({"decision": "block", "reason": outcome["text"]}, ensure_ascii=False))
    return 0


# ── 自证：每一格造一个答案已知的局面，在本进程里调 decide（包装那几格起钩子本身）
def iso(seconds):
    return datetime.fromtimestamp(seconds, timezone.utc).isoformat().replace("+00:00", "Z")


class Scenario:
    """一格一个：临时目录里的工作根、崩溃放量的家目录、会话临时目录根、项目目录与主会话记录。"""

    def __init__(self, base, name):
        self.root = os.path.join(base, name)
        self.work = os.path.join(self.root, "work")
        self.home = os.path.join(self.root, "crash-home")
        self.runs = os.path.join(self.home, "runs")
        self.scratch = os.path.join(self.root, "session-temporary-root")
        self.project = os.path.join(self.root, "projects", "project-directory")
        self.session = "s-" + re.sub(r"[^A-Za-z0-9_-]", "-", name)
        self.main_transcript = os.path.join(self.project, self.session + ".jsonl")
        self.service_database_path = os.path.join(self.root, "task-server.sqlite3")   # 不建就是服务没装过：不判服务编译目录
        self.data = os.path.join(self.root, "data-root")             # 数据根：与工作根分开，迁库遗留那几格往里放
        self.prune_control = os.path.join(self.root, "prune-control")   # prune-server/control 的自证根：状态目录与单元文件都在它底下
        self.processes = []
        self.call_count = 0
        for directory in (self.work, self.runs, self.scratch, self.project, self.data):
            os.makedirs(directory, exist_ok=True)
        self.now = time.time()
        self.write(self.main_transcript, [self.user_text(self.now - 100, "开工")])

    def environment(self):
        return {"SINGLEFS_WORK_ROOT": self.work, CRASH_AMPLIFICATION_HOME_VARIABLE: self.home, SCRATCHPAD_ROOT_VARIABLE: self.scratch,
                DETECTIONS_VARIABLE: os.path.join(self.root, "detections.jsonl"), TASK_SERVER_DATABASE_VARIABLE: self.service_database_path,
                module("lib_work_root").DATA_ROOT_VARIABLE: self.data, module("prune_server_control").SELFTEST_ROOT_VARIABLE: self.prune_control}

    def prune_paths(self):
        """prune-server/control 在这一格的自证根底下取的路径（状态目录、单元文件、二进制）。"""
        control = module("prune_server_control")
        saved = os.environ.get(control.SELFTEST_ROOT_VARIABLE)
        os.environ[control.SELFTEST_ROOT_VARIABLE] = self.prune_control
        try:
            return control.Paths()
        finally:
            if saved is None:
                os.environ.pop(control.SELFTEST_ROOT_VARIABLE, None)
            else:
                os.environ[control.SELFTEST_ROOT_VARIABLE] = saved

    def bash(self, command):
        """主会话记录里记一次做完了的 Bash 调用。"""
        self.call_count += 1
        self.write(self.main_transcript, self.bash_call(f"toolu_leftover_{self.call_count}", self.now - 40, self.now - 39, command), append=True)

    @staticmethod
    def filled(path):
        """一个不空的目录（旧文件库的样子：里面一份键文件）。"""
        os.makedirs(path, exist_ok=True)
        with open(os.path.join(path, "keys.json"), "w", encoding="utf-8") as handle:
            handle.write("{}\n")
        return path

    @staticmethod
    def archive(path):
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "wb") as handle:
            handle.write(b"x" * 512)
        return path

    @staticmethod
    def write(path, records, append=False):
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "a" if append else "w", encoding="utf-8") as handle:
            for record in records:
                handle.write(json.dumps(record, ensure_ascii=False) + "\n")

    @staticmethod
    def user_text(seconds, text):
        return {"type": "user", "timestamp": iso(seconds), "message": {"role": "user", "content": text}}

    @staticmethod
    def bash_call(call_id, started, ended, command, result="完成", background=False):
        tool_input = {"command": command}
        if background:
            tool_input["run_in_background"] = True
        records = [{"type": "assistant", "timestamp": iso(started),
                    "message": {"role": "assistant", "content": [{"type": "tool_use", "id": call_id, "name": "Bash", "input": tool_input}]}}]
        if ended is not None:
            records.append({"type": "user", "timestamp": iso(ended),
                            "message": {"role": "user", "content": [{"type": "tool_result", "tool_use_id": call_id, "content": result}]}})
        return records

    @staticmethod
    def notification(seconds, task_id, status="completed"):
        return {"type": "user", "timestamp": iso(seconds), "origin": {"kind": "task-notification"},
                "message": {"role": "user", "content": f"<task-notification>\n<task-id>{task_id}</task-id>\n<status>{status}</status>\n</task-notification>"}}

    def owned(self, name, session=None):
        path = os.path.join(self.work, name)
        os.makedirs(path, exist_ok=True)
        with open(os.path.join(path, ".owner"), "w", encoding="utf-8") as handle:
            handle.write(f"session: {session or self.session}\ncreated: 2000-01-01\npurpose: 自证\n")
        return path

    @staticmethod
    def build_directory(path):
        os.makedirs(os.path.join(path, "release", "deps"), exist_ok=True)
        with open(os.path.join(path, "CACHEDIR.TAG"), "w") as handle:
            handle.write("Signature: 8a477f597d28d172789f06886806bc55\n")
        with open(os.path.join(path, "release", "deps", "libx.rlib"), "wb") as handle:
            handle.write(b"x" * 4096)
        return path

    @staticmethod
    def repository_copy(path):
        os.makedirs(os.path.join(path, ".git"), exist_ok=True)
        os.makedirs(os.path.join(path, "crates", "core"), exist_ok=True)
        open(os.path.join(path, "Cargo.toml"), "w").close()
        return path

    @staticmethod
    def library_copy(path):
        os.makedirs(path, exist_ok=True)
        for name in ("CURRENT", "MANIFEST-000004", "000005.sst"):
            open(os.path.join(path, name), "w").close()
        return path

    def service_task(self, task_id, state, session=None, parent_id=None, build=True):
        """样本服务库里记一个任务与它的一次尝试（表与列照 task-server/task_server.py 的 Store）；build 为真时尝试的编译目录建在盘上。交编译目录的路径。"""
        connection = sqlite3.connect(self.service_database_path)
        try:
            for table in ("tasks", "attempts"):
                connection.execute(f"CREATE TABLE IF NOT EXISTS {table}(id INTEGER PRIMARY KEY AUTOINCREMENT, task_id INTEGER NOT NULL DEFAULT 0, state TEXT NOT NULL, "
                                   "session TEXT NOT NULL, command_key TEXT NOT NULL, record TEXT NOT NULL, signature TEXT NOT NULL)")
            owner_session = session or self.session
            record = {"id": task_id, "state": state, "session": owner_session}
            if parent_id is not None:
                record["parent_id"] = parent_id
            connection.execute("INSERT INTO tasks(id, task_id, state, session, command_key, record, signature) VALUES(?, ?, ?, ?, '', ?, '')",
                               (task_id, parent_id or 0, state, owner_session, json.dumps(record)))
            directory = os.path.join(self.root, "service-builds", f"task-sample-{task_id}-1")
            connection.execute("INSERT INTO attempts(task_id, state, session, command_key, record, signature) VALUES(?, 'finished', ?, '', ?, '')",
                               (task_id, owner_session, json.dumps({"task_id": task_id, "build_directory": directory})))
            connection.commit()
        finally:
            connection.close()
        if build:
            self.build_directory(os.path.join(directory, "target"))
        return directory

    def run_directory(self, pid, suffix="", owner_session=None):
        """运行目录根下一层：名字照入口脚本的「<前缀>-<时刻>-<进程号>[-<后缀>]」；给了 owner_session 的写归属记录。"""
        path = os.path.join(self.runs, f"crash-amplification-{time.strftime('%Y%m%dT%H%M%SZ', time.gmtime())}-{pid}{suffix}")
        os.makedirs(path)
        if owner_session:
            with open(os.path.join(path, RUN_OWNER_FILE), "w", encoding="utf-8") as handle:
                handle.write(f"session: {owner_session}\ncreated: 2000-01-01\npurpose: 自证的运行目录\n")
        return path

    @staticmethod
    def dead_pid():
        """比 pid_max 大的进程号：一定不在。"""
        try:
            return int(open("/proc/sys/kernel/pid_max").read()) + 1
        except (OSError, ValueError):
            return 99999999

    def creator(self, session, launcher_command=None):
        """起运行目录的假入口：环境里带会话号与 CLAUDE_PID（本进程当 Claude 进程）。给了 launcher_command 的照 Bash 工具那样起
        bash -c 'true && eval <命令> && true'，命令用内建的 read 等在管道上（不起子进程，停它只停这一个进程号）。"""
        environment = {key: value for key, value in os.environ.items() if key not in SESSION_VARIABLES}
        environment.update({"CLAUDE_CODE_SESSION_ID": session, CLAUDE_PID_VARIABLE: str(os.getpid())})
        arguments = ["sleep", "120"] if launcher_command is None else ["bash", "-c", f"true && eval {shlex.quote(launcher_command)} && true"]
        process = subprocess.Popen(arguments, stdin=subprocess.PIPE, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, env=environment)
        self.processes.append(process)
        return process

    def end(self, process):
        process.kill()
        process.wait()
        self.processes.remove(process)

    def subagent(self, agent_id, prompt, records=()):
        path = os.path.join(self.project, self.session, "subagents", f"agent-{agent_id}.jsonl")
        self.write(path, [self.user_text(self.now - 90, prompt), *records])
        return path

    def hold(self, arguments, **options):
        process = subprocess.Popen(arguments, stdin=subprocess.DEVNULL, stderr=subprocess.DEVNULL, **options)
        self.processes.append(process)
        return process

    def background_task(self, task_id, command_arguments):
        """主会话记录里一条还没完成通知的后台任务，它的输出文件由一个活着的进程开着。"""
        output = os.path.join(self.root, "tasks", f"{task_id}.output")
        os.makedirs(os.path.dirname(output), exist_ok=True)
        handle = open(output, "w")
        process = self.hold(command_arguments, stdout=handle)
        handle.close()
        # 等它开着输出文件、exec 换好进程名（bash -c 'exec -a …' 那种，换好之前命令行还是 bash -c …）
        for _ in range(250):
            try:
                holding = os.path.realpath(os.readlink(f"/proc/{process.pid}/fd/1")) == os.path.realpath(output)
                renamed = command_arguments[0] != "bash" or not open(f"/proc/{process.pid}/cmdline", "rb").read().startswith(b"bash")
                if holding and renamed:
                    break
            except OSError:
                pass
            time.sleep(0.02)
        self.write(self.main_transcript, self.bash_call(f"toolu_{task_id}", self.now - 50, self.now - 49, "bash x.sh",
                                                        f"Command running in background with ID: {task_id}. Output is being written to: {output}",
                                                        background=True), append=True)

    def stop(self):
        for process in self.processes:
            process.kill()
            process.wait()
        self.processes = []

    def decide(self, hook_input, extra=None):
        environment = dict(self.environment(), **(extra or {}))
        saved = {key: os.environ.get(key) for key in environment}
        os.environ.update(environment)
        try:
            return decide(hook_input)
        finally:
            for key, value in saved.items():
                if value is None:
                    os.environ.pop(key, None)
                else:
                    os.environ[key] = value

    def stop_input(self, active=False):
        return {"hook_event_name": "Stop", "session_id": self.session, "transcript_path": self.main_transcript, "stop_hook_active": active}

    def subagent_stop_input(self, agent_id, path, active=False):
        return {"hook_event_name": "SubagentStop", "session_id": self.session, "transcript_path": self.main_transcript,
                "agent_id": agent_id, "agent_transcript_path": path, "stop_hook_active": active}

    def handback_input(self, agent_id):
        return {"hook_event_name": "PreToolUse", "tool_name": HANDBACK_TOOL, "session_id": self.session,
                "transcript_path": self.main_transcript, "agent_id": agent_id, "tool_input": {"message": "交回"}}


def selftest_cells():
    base = tempfile.mkdtemp(prefix="completion-cleanup-selftest-")
    failures, checked = [], [0]

    def check(cell, ok, detail):
        checked[0] += 1
        if not ok:
            failures.append(cell)
            print(f"✗ 自证 {cell}：{detail}")
            print(f"→ 怎么办：看 completion-cleanup-guard.sh 里 {cell} 对应的判定；{BREAK_VARIABLE} 设着的话这里本来就该红")

    def blocked_with(outcome, *paths):
        return outcome.get("action") == "block" and all(path in outcome.get("text", "") for path in paths)

    def detection_entries(path):
        entries = []
        if os.path.exists(path):
            with open(path, encoding="utf-8") as handle:
                for line in handle:
                    try:
                        entries.append(json.loads(line))
                    except ValueError:
                        entries.append({"坏行": line})
        return entries

    scenarios = []

    def scenario(name):
        made = Scenario(base, name)
        scenarios.append(made)
        return made

    try:
        # 主 agent：认领目录里留了编译目录
        s = scenario("main-build")
        target = s.build_directory(os.path.join(s.owned("w-build"), "copy", "target"))
        outcome = s.decide(s.stop_input())
        check("main-build-directory-left", blocked_with(outcome, target), f"留了编译目录 {target} 应当拦下并列出，实际 {outcome}")
        # 主 agent：认领目录里留了仓副本
        s = scenario("main-repository")
        copy = s.repository_copy(os.path.join(s.owned("w-copy"), "repository"))
        outcome = s.decide(s.stop_input())
        check("main-repository-copy-left", blocked_with(outcome, copy, f"rm -rf -- {copy}"), f"留了仓副本 {copy} 应当拦下、列出删法，实际 {outcome}")
        # 主 agent：归属记录是本会话的崩溃放量运行目录里留了库拷贝（起它的进程已经不在）
        s = scenario("main-run")
        library = s.library_copy(os.path.join(s.run_directory(s.dead_pid(), "-two-hosts", owner_session=s.session), "library-copy"))
        outcome = s.decide(s.stop_input())
        check("main-run-library-copy-left", blocked_with(outcome, library), f"运行目录里留了库拷贝 {library} 应当拦下，实际 {outcome}")
        # 没设 SINGLEFS_CRASH_AMPLIFICATION_HOME：运行目录根是工作根下的 crash-amplification/runs，那里本会话的运行目录留了库拷贝照拦
        s = scenario("run-root-default")
        s.runs = os.path.join(s.work, CRASH_AMPLIFICATION_DIRECTORY_NAME, "runs")
        library = s.library_copy(os.path.join(s.run_directory(s.dead_pid(), owner_session=s.session), "library-copy"))
        outcome = s.decide(s.stop_input(), {CRASH_AMPLIFICATION_HOME_VARIABLE: ""})
        check("run-root-default-under-work-root", blocked_with(outcome, library),
              f"没设 {CRASH_AMPLIFICATION_HOME_VARIABLE} 时工作根下 crash-amplification/runs 里本会话的运行目录留了库拷贝 {library} 应当拦下，实际 {outcome}")
        # 本会话的进程起的运行目录：起它的进程还在时整个不列（那一趟还在跑），看到一次就记账；进程退了照记账拦
        s = scenario("own-run")
        process = s.creator(s.session)
        run = s.run_directory(process.pid)
        library = s.library_copy(os.path.join(run, "library-copy"))
        target = s.build_directory(os.path.join(run, "target"))
        running = s.decide(s.stop_input())
        check("run-in-progress-not-listed", running.get("action") == "pass" and library not in running.get("text", ""),
              f"起运行目录的进程 {process.pid} 还在（那一趟还在跑），里面的 {library} 不该列，实际 {running}")
        s.end(process)
        outcome = s.decide(s.stop_input())
        check("own-run-blocked-after-creator-exits", blocked_with(outcome, library, target),
              f"本会话的进程起的运行目录（看到它还在跑时记过账），进程退了之后留下的 {library}、{target} 应当拦下，实际 {outcome}")
        # 别的会话的进程起的运行目录，创建时间落在本会话的 Bash 调用时间窗里：不拦（记账认的、归属记录认的各一份）
        s = scenario("foreign-session-run")
        s.write(s.main_transcript, s.bash_call("toolu_run", s.now - 30, s.now + 600, "bash scripts/crash/crash-amplification.sh --item x"), append=True)
        process = s.creator("s-somebody-else")
        seen = s.library_copy(os.path.join(s.run_directory(process.pid), "library-copy"))
        s.decide(s.stop_input())
        s.end(process)
        recorded = s.library_copy(os.path.join(s.run_directory(s.dead_pid(), "-two-hosts", owner_session="s-somebody-else"), "library-copy"))
        outcome = s.decide(s.stop_input())
        check("foreign-session-run-in-window-not-listed",
              outcome.get("action") == "pass" and not any(path in str(outcome) for path in (seen, recorded)),
              f"别的会话的进程起的运行目录（{seen}、{recorded}）时间窗与本会话重叠也不该拦、不该报，实际 {outcome}")
        # 认不出是谁的（进程不在、没记账、没归属记录）、建在本会话 Bash 调用的时间窗里：只在拦下时另起一节报，不进单子
        s = scenario("unknown-run")
        s.write(s.main_transcript, s.bash_call("toolu_run", s.now - 30, s.now + 600, "bash scripts/crash/crash-amplification.sh --item x"), append=True)
        unknown = s.library_copy(os.path.join(s.run_directory(s.dead_pid()), "library-copy"))
        alone = s.decide(s.stop_input())
        target = s.build_directory(os.path.join(s.owned("w-unknown"), "target"))
        outcome = s.decide(s.stop_input())
        text = outcome.get("text", "")
        check("unknown-run-reported-not-blocked",
              alone.get("action") == "pass" and blocked_with(outcome, target) and unknown not in outcome.get("paths", [])
              and UNKNOWN_SECTION_TITLE in text and text.index(UNKNOWN_SECTION_TITLE) < text.index(unknown),
              f"认不出是谁的 {unknown} 不该拦，只在拦下时列进「{UNKNOWN_SECTION_TITLE}」一节，实际单独时 {alone}，有别的要拦时 {outcome}")
        # 运行目录里深处有文件被开着（那一趟还在写日志），库拷贝本身没人开着：整个运行目录不列
        s = scenario("run-held-deep")
        run = s.run_directory(s.dead_pid(), owner_session=s.session)
        library = s.library_copy(os.path.join(run, "library-copy"))
        os.makedirs(os.path.join(run, "logs", "deeper"))
        with open(os.path.join(run, "logs", "deeper", "share.log"), "w") as handle:
            s.hold(["sleep", "120"], stdout=handle)
        outcome = s.decide(s.stop_input())
        check("run-held-deep-not-listed", outcome.get("action") == "pass" and library not in outcome.get("text", ""),
              f"运行目录 {run} 深处的日志有进程开着，里面的 {library} 不该列，实际 {outcome}")
        s.stop()
        # 只经内存映射开着（文件描述符已关）的编译目录不列（run_build_directory.py 的判法看 /proc/<pid>/maps）
        s = scenario("mapped")
        mapped = s.build_directory(os.path.join(s.owned("w-mapped"), "target"))
        mapped_file = os.path.join(mapped, "release", "deps", "libx.rlib")
        ready = os.path.join(s.root, "mapped.ready")
        s.hold([sys.executable, "-c", "import mmap, sys, time\nhandle = open(sys.argv[1], 'rb')\nmapping = mmap.mmap(handle.fileno(), 0, access=mmap.ACCESS_READ)\n"
                "handle.close()\nopen(sys.argv[2], 'w').close()\ntime.sleep(120)\n", mapped_file, ready], stdout=subprocess.DEVNULL)
        for _ in range(250):
            if os.path.exists(ready):
                break
            time.sleep(0.02)
        outcome = s.decide(s.stop_input())
        check("mapped-copy-not-listed", os.path.exists(ready) and outcome.get("action") == "pass",
              f"编译目录 {mapped} 里的文件只经内存映射开着也算在用，不该列，实际 {outcome}")
        s.stop()
        # 子 agent：只认它自己的 Bash 调用起的运行目录；同一会话别的调用（主 agent、兄弟子 agent）起的不算它的
        s = scenario("subagent-run")
        mine_command, sibling_command = ": crash-amplification mine; read -r _", ": crash-amplification sibling; read -r _"
        agent = s.subagent("arun", "派发", s.bash_call("toolu_mine", s.now - 20, s.now + 600, mine_command))
        mine_process, sibling_process = s.creator(s.session, mine_command), s.creator(s.session, sibling_command)
        mine = s.library_copy(os.path.join(s.run_directory(mine_process.pid), "library-copy"))
        sibling = s.library_copy(os.path.join(s.run_directory(sibling_process.pid), "library-copy"))
        s.decide(s.subagent_stop_input("arun", agent))
        s.end(mine_process)
        s.end(sibling_process)
        outcome = s.decide(s.subagent_stop_input("arun", agent))
        check("subagent-own-run-left", blocked_with(outcome, mine), f"子 agent 自己的 Bash 调用起的运行目录里的 {mine} 应当拦下，实际 {outcome}")
        check("subagent-ignores-sibling-runs", sibling not in outcome.get("text", ""), f"同一会话别的调用起的 {sibling} 不该算到这个子 agent 头上，实际 {outcome}")
        # 服务给的编译目录（文件头「判哪几处」服务给的编译目录那一段）：本会话已有结局、编译目录还在的拦下，删法是 task-server/task clean <号>（不写 rm）
        s = scenario("service-build")
        finished = s.service_task(7, "finished")
        child = s.service_task(8, "finished", parent_id=7)
        removed = s.service_task(9, "finished", build=False)
        outcome = s.decide(s.stop_input())
        check("main-service-build-left", blocked_with(outcome, finished, child, f"{TASK_SERVER_CLIENT} clean 7")
              and f"rm -rf -- {finished}" not in outcome.get("text", "") and f"{TASK_SERVER_CLIENT} clean 8" not in outcome.get("text", "")
              and removed not in outcome.get("text", ""),
              f"本会话已有结局的任务 7（连同分片子任务 8）的编译目录 {finished}、{child} 应当拦下、删法只写 {TASK_SERVER_CLIENT} clean 7，盘上已经没了的 {removed} 不列，实际 {outcome}")
        s = scenario("service-build-running")
        running = s.service_task(10, "running")
        queued = s.service_task(11, "queued")
        outcome = s.decide(s.stop_input())
        check("running-service-build-not-listed", outcome.get("action") == "pass" and running not in str(outcome) and queued not in str(outcome),
              f"在跑、在排的任务的编译目录（{running}、{queued}）不该列，实际 {outcome}")
        s = scenario("service-build-other-session")
        theirs = s.service_task(12, "finished", session="s-somebody-else")
        outcome = s.decide(s.stop_input())
        check("other-session-service-build-ignored", outcome.get("action") == "pass" and theirs not in str(outcome),
              f"别的会话提交的任务的编译目录 {theirs} 不该列，实际 {outcome}")
        # 子 agent 不交任务、不起 task clean：交回时本会话已有结局、编译目录还在的任务一个都不列（归主 agent 收工时判）
        s = scenario("service-build-subagent")
        agent = s.subagent("abuild", "派发", s.bash_call("toolu_status", s.now - 20, s.now - 19, "task-server/task status 13", "task 13 finished"))
        session_build = s.service_task(13, "finished")
        outcome = s.decide(s.handback_input("abuild"))
        check("subagent-service-builds-not-listed", outcome.get("action") == "pass" and session_build not in str(outcome),
              f"子 agent 交回时本会话任务 13 的编译目录 {session_build} 不该列（子 agent 不交任务、不起 {TASK_SERVER_CLIENT} clean），实际 {outcome}")
        # 别的会话认领的目录不判
        s = scenario("other-session")
        s.build_directory(os.path.join(s.owned("w-theirs", session="s-somebody-else"), "target"))
        outcome = s.decide(s.stop_input())
        check("other-session-directory-ignored", outcome.get("action") == "pass", f"别的会话认领的目录不该判，实际 {outcome}")
        # 只剩交付物：报告、补丁、声明成备份的副本
        s = scenario("deliverables")
        directory = s.owned("w-deliver")
        open(os.path.join(directory, "result.md"), "w").close()
        os.makedirs(os.path.join(directory, "patch"))
        open(os.path.join(directory, "patch", "crates.patch"), "w").close()
        s.repository_copy(os.path.join(directory, "before-patch"))
        with open(os.path.join(directory, DECLARATION_FILE), "w", encoding="utf-8") as handle:
            handle.write("# 交付物\nbefore-patch 备份 打补丁之前的原样\nbroken-line 留着\n")
        outcome = s.decide(s.stop_input())
        check("deliverables-only-passes", outcome.get("action") == "pass", f"只剩报告、补丁与声明了的备份应当放行，实际 {outcome}")
        # 声明成交付物的编译目录照列
        s = scenario("declared-build")
        directory = s.owned("w-declared")
        kept = s.build_directory(os.path.join(directory, "kept-target"))
        with open(os.path.join(directory, DECLARATION_FILE), "w", encoding="utf-8") as handle:
            handle.write("kept-target 备份 想留着下一趟接着编\n")
        outcome = s.decide(s.stop_input())
        check("build-directory-never-deliverable", blocked_with(outcome, kept), f"声明成备份的编译目录 {kept} 照样要列，实际 {outcome}")
        # 交接副本：主 agent 照列，子 agent 交回时不列
        s = scenario("handover")
        directory = s.owned("w-handover")
        handed = s.repository_copy(os.path.join(directory, "copy"))
        with open(os.path.join(directory, DECLARATION_FILE), "w", encoding="utf-8") as handle:
            handle.write("copy 交接 主 agent 在里面交带 --item 的\n")
        outcome = s.decide(s.stop_input())
        check("main-handover-copy-left", blocked_with(outcome, handed), f"主 agent 结束一轮时交接副本 {handed} 照列，实际 {outcome}")
        agent = s.subagent("ahandover", "派发", s.bash_call("toolu_claim", s.now - 80, s.now - 79,
                                                              "bash scripts/hosts/work-directory-ownership.sh claim w-handover --purpose 自证"))
        outcome = s.decide(s.subagent_stop_input("ahandover", agent))
        check("subagent-handover-passes", outcome.get("action") == "pass", f"子 agent 交回时交接副本不列，实际 {outcome}")
        # 有在跑的后台任务：放行
        s = scenario("background")
        s.build_directory(os.path.join(s.owned("w-busy"), "target"))
        s.background_task("bwork", ["sleep", "120"])
        outcome = s.decide(s.stop_input())
        check("background-running-passes", outcome.get("action") == "pass", f"手里有在跑的后台任务应当放行，实际 {outcome}")
        s.stop()
        # 后台只有循环提醒在等：照判
        s = scenario("reminder")
        target = s.build_directory(os.path.join(s.owned("w-remind"), "target"))
        s.background_task("bremind", ["bash", "-c", "exec -a remind.sh sleep 120"])
        outcome = s.decide(s.stop_input())
        check("reminder-only-still-judged", blocked_with(outcome, target), f"后台只有循环提醒在等不算在跑的活，应当照拦，实际 {outcome}")
        s.stop()
        # 有在跑的子 agent：放行；子 agent 收到完成通知之后：照判
        s = scenario("subagent-running")
        s.build_directory(os.path.join(s.owned("w-sub"), "target"))
        s.subagent("arunning", "派发", s.bash_call("toolu_pending", s.now - 5, None, "sleep 5"))
        outcome = s.decide(s.stop_input())
        check("subagent-running-passes", outcome.get("action") == "pass", f"手里有在跑的子 agent 应当放行，实际 {outcome}")
        s = scenario("subagent-finished")
        target = s.build_directory(os.path.join(s.owned("w-sub"), "target"))
        s.subagent("afinished", "派发")
        s.write(s.main_transcript, [s.notification(time.time() + 5, "afinished")], append=True)
        outcome = s.decide(s.stop_input())
        check("subagent-finished-still-judged", blocked_with(outcome, target), f"子 agent 已收到完成通知、不算在跑，应当照拦，实际 {outcome}")
        # 有进程在用的不列
        s = scenario("held")
        directory = s.owned("w-held")
        held = s.build_directory(os.path.join(directory, "held-target"))
        idle = s.build_directory(os.path.join(directory, "idle-target"))
        s.hold(["sleep", "120"], cwd=held)
        outcome = s.decide(s.stop_input())
        check("held-copy-not-listed", blocked_with(outcome, idle) and held not in outcome.get("text", ""),
              f"有进程在用的 {held} 不该列、没人用的 {idle} 照列，实际 {outcome}")
        s.stop()
        # 子 agent：只判自己认领的，主 agent 的别的目录不判
        s = scenario("subagent-scope")
        mine = s.build_directory(os.path.join(s.owned("sub-claimed"), "target"))
        sibling = s.build_directory(os.path.join(s.owned("main-own"), "target"))
        agent = s.subagent("ascope", "派发", s.bash_call("toolu_claim", s.now - 80, s.now - 79,
                                                           "bash scripts/hosts/work-directory-ownership.sh claim sub-claimed --purpose '自证'"))
        outcome = s.decide(s.subagent_stop_input("ascope", agent))
        check("subagent-build-directory-left", blocked_with(outcome, mine), f"子 agent 自己认领的目录里的编译目录 {mine} 应当拦下，实际 {outcome}")
        check("subagent-ignores-siblings", sibling not in outcome.get("text", ""), f"主 agent 的别的目录 {sibling} 不该算到子 agent 头上，实际 {outcome}")
        # 子 agent：派发提示点名的草稿目录
        s = scenario("subagent-assigned")
        directory = s.owned("assigned")
        assigned_copy = s.repository_copy(os.path.join(directory, "copy"))
        agent = s.subagent("aassigned", f"# 派发\n草稿目录：{directory}\n报告：{directory}/result.md\n")
        outcome = s.decide(s.subagent_stop_input("aassigned", agent))
        check("subagent-assigned-directory-left", blocked_with(outcome, assigned_copy), f"派发提示点名的草稿目录里的仓副本 {assigned_copy} 应当拦下，实际 {outcome}")
        # 子 agent：「草稿目录：」写相对路径的按仓根解（lib_prompt_scratch_line.py 的 repository_root），不按钩子此刻的当前目录
        # （b12-scratch-lib.md「要主 agent 判的」第二条；开关 prompt-scratch-relative-by-cwd 下这两格红）
        s = scenario("subagent-assigned-relative")
        directory = s.owned("assigned-relative")
        relative_copy = s.repository_copy(os.path.join(directory, "copy"))
        relative_written = os.path.relpath(directory, module("lib_prompt_scratch_line").repository_root())
        deep_elsewhere = os.path.join(s.root, "cwd-elsewhere", *(["deeper"] * 8))
        os.makedirs(deep_elsewhere)
        cwd_named = s.owned("assigned-by-cwd")
        cwd_copy = s.repository_copy(os.path.join(cwd_named, "copy"))
        here = os.getcwd()
        try:
            os.chdir(deep_elsewhere)   # 当前目录在仓外、层数比仓根深：按当前目录解，那几个 .. 落不回工作根
            agent = s.subagent("arelative", f"# 派发\n草稿目录：{relative_written}\n报告：{relative_written}/result.md\n")
            by_root = s.decide(s.subagent_stop_input("arelative", agent))
            os.chdir(s.work)   # 当前目录是工作根：只写目录名的，按当前目录解就落到工作根下那个目录，按仓根解不在工作根下
            agent = s.subagent("acwd", "# 派发\n草稿目录：assigned-by-cwd\n报告：assigned-by-cwd/result.md\n")
            by_cwd = s.decide(s.subagent_stop_input("acwd", agent))
        finally:
            os.chdir(here)
        check("subagent-assigned-relative-by-repository-root", blocked_with(by_root, relative_copy),
              f"「草稿目录：{relative_written}」按仓根解是 {directory}，里面的仓副本 {relative_copy} 应当拦下（当前目录在 {deep_elsewhere}），实际 {by_root}")
        check("subagent-assigned-relative-not-by-cwd", cwd_copy not in by_cwd.get("text", ""),
              f"「草稿目录：assigned-by-cwd」按仓根解不在工作根下，不该按当前目录 {s.work} 解成 {cwd_named}、把 {cwd_copy} 算到这个子 agent 头上，实际 {by_cwd}")
        # 子 agent：提示里转述别的子 agent 的草稿目录（句中「，草稿目录 <别处>」、列表「- 草稿目录 <别处>」）不算它的；那一行的认法调的是 lib_prompt_scratch_line.py
        s = scenario("subagent-assigned-relay")
        mine, previous, listed = s.owned("assigned-mine"), s.owned("assigned-previous"), s.owned("assigned-listed")
        previous_copy = s.repository_copy(os.path.join(previous, "copy"))
        listed_copy = s.repository_copy(os.path.join(listed, "copy"))
        agent = s.subagent("arelay", f"# 派发\n草稿目录：{mine}\n报告：{mine}/result.md\n前一段交回在先，草稿目录 {previous}/ 里有它的副本。\n要读的：\n- 草稿目录 {listed}/ 下的 logs/\n")
        outcome = s.decide(s.subagent_stop_input("arelay", agent))
        check("subagent-assigned-relay-ignored", previous_copy not in outcome.get("text", "") and listed_copy not in outcome.get("text", ""),
              f"转述里别的子 agent 的草稿目录 {previous}、{listed} 不该算到这个子 agent 头上，实际 {outcome}")
        shared_reading = module("lib_prompt_scratch_line")
        reading = shared_reading.scratch_line_values
        shared_reading.scratch_line_values = lambda text: [previous]
        try:
            outcome = s.decide(s.subagent_stop_input("arelay", agent))
        finally:
            shared_reading.scratch_line_values = reading
        check("subagent-assigned-shared-reading", blocked_with(outcome, previous_copy),
              f"草稿目录一行的认法换成替身（只交 {previous}）之后应当拦下 {previous_copy}：调的要是 lib_prompt_scratch_line.py 那一份，实际 {outcome}")
        # 别的时间窗里建的运行目录不算这里的
        s = scenario("foreign-run")
        s.library_copy(os.path.join(s.runs, "crash-amplification-foreign", "library-copy"))
        s.write(s.main_transcript, [s.user_text(s.now - 7300, "早先")] + s.bash_call("toolu_old", s.now - 7200, s.now - 7100, "bash scripts/crash/crash-amplification.sh"), append=False)
        outcome = s.decide(s.stop_input())
        check("foreign-run-directory-ignored", outcome.get("action") == "pass", f"不在自己 Bash 调用时间窗里建的运行目录不该判，实际 {outcome}")
        # 续跑回来同一张单：放行交用户；单子变了：照拦
        s = scenario("release")
        directory = s.owned("w-release")
        target = s.build_directory(os.path.join(directory, "target"))
        first = s.decide(s.stop_input())
        second = s.decide(s.stop_input(active=True))
        check("unchanged-after-block-releases", first.get("action") == "block" and second.get("action") == "release" and target in second.get("text", ""),
              f"拦过一次、续跑回来一个没删应当放行并把单子交用户，实际 {first.get('action')} 之后 {second}")
        s.build_directory(os.path.join(directory, "another-target"))
        third = s.decide(s.stop_input(active=True))
        check("changed-after-block-blocks-again", third.get("action") == "block", f"续跑回来单子变了应当照拦，实际 {third}")
        # 单上写 du 的大小
        s = scenario("size")
        target = s.build_directory(os.path.join(s.owned("w-size"), "target"))
        outcome = s.decide(s.stop_input())
        check("block-lists-size", re.search(rf"  {KIND_BUILD}  {re.escape(target)}  [0-9.,]+[KMGTP]?\n", outcome.get("text", "")) is not None,
              f"单上每一行要写 du -sh 的大小，实际 {outcome}")
        # 主 agent：本会话的临时草稿目录（tasks/ 不看）
        s = scenario("scratchpad")
        pad = os.path.join(s.scratch, "project-directory", s.session)
        pad_copy = s.repository_copy(os.path.join(pad, "copy"))
        task_copy = s.build_directory(os.path.join(pad, "tasks", "target"))
        outcome = s.decide(s.stop_input())
        check("main-scratchpad-copy-left", blocked_with(outcome, pad_copy) and task_copy not in outcome.get("text", ""),
              f"本会话临时草稿目录里的仓副本 {pad_copy} 应当拦下、tasks/ 下的不看，实际 {outcome}")
        # 什么都没留：放行
        s = scenario("clean")
        open(os.path.join(s.owned("w-clean"), "result.md"), "w").close()
        outcome = s.decide(s.stop_input())
        check("nothing-left-passes", outcome.get("action") == "pass", f"只剩报告应当放行，实际 {outcome}")
        # 扫到上限没看完、没有残留：照常放行，没判全的几处与出路打出来、检出记录多一行（结局只记）；上限经 COMPLETION_CLEANUP_SCAN_MAX_DIRECTORIES 调成 3
        s = scenario("truncated-scan")
        deep = s.owned("w-deep")
        for name in ("a", "b", "c", "d", "e", "f"):
            os.makedirs(os.path.join(deep, name, "inner"))
        outcome = s.decide(s.stop_input(), {SCAN_LIMIT_VARIABLE: "3"})
        detections = os.path.join(s.root, "detections-truncated.jsonl")
        printed = io.StringIO()
        report_unjudged_pass(s.stop_input(), outcome.get("notes") or [], printed, detections)
        entries, text = detection_entries(detections), printed.getvalue()
        check("truncated-scan-pass-reported",
              outcome.get("action") == "pass" and f"看了 3 个目录到顶，这几处没看完：{deep}" in text and "出路" in text
              and len(entries) == 1 and entries[0].get("outcome") == "只记" and entries[0].get("session_id") == s.session,
              f"扫到上限（3 个）没看完 {deep}、没有残留应当放行，打出没看完的几处与出路、检出记录多一行，实际 {outcome}，打出 {text!r}，检出 {entries}")
        # 运行目录取不到创建时间（birth_times 换成替身，全交 None）、没有残留：照常放行，「取不到创建时间」那一句打出来、检出记录多一行
        s = scenario("birth-unknown")
        run = s.run_directory(s.dead_pid(), owner_session=s.session)
        real_birth_times = birth_times
        globals()["birth_times"] = lambda paths: {path: None for path in sorted(paths)}
        try:
            outcome = s.decide(s.stop_input())
        finally:
            globals()["birth_times"] = real_birth_times
        detections = os.path.join(s.root, "detections-birth.jsonl")
        printed = io.StringIO()
        report_unjudged_pass(s.stop_input(), outcome.get("notes") or [], printed, detections)
        entries, text = detection_entries(detections), printed.getvalue()
        check("birth-unknown-pass-reported",
              outcome.get("action") == "pass" and "取不到创建时间的运行目录 1 个没判" in text and run in text and len(entries) == 1 and entries[0].get("outcome") == "只记",
              f"运行目录 {run} 取不到创建时间、没有残留应当放行，打出那一句、检出记录多一行，实际 {outcome}，打出 {text!r}，检出 {entries}")
        # 读不了的目录（这里把认领目录里的一层 chmod 成 000）、没有残留：照常放行，「读不了的目录」那一句打出来、检出记录多一行
        s = scenario("unreadable-directory")
        locked = os.path.join(s.owned("w-locked"), "locked")
        os.makedirs(locked)
        os.chmod(locked, 0)
        try:
            outcome = s.decide(s.stop_input())
        finally:
            os.chmod(locked, 0o700)
        detections = os.path.join(s.root, "detections-unreadable.jsonl")
        printed = io.StringIO()
        report_unjudged_pass(s.stop_input(), outcome.get("notes") or [], printed, detections)
        entries, text = detection_entries(detections), printed.getvalue()
        check("unreadable-directory-pass-reported",
              outcome.get("action") == "pass" and "读不了的目录 1 个没看" in text and locked in text and len(entries) == 1 and entries[0].get("outcome") == "只记",
              f"认领目录里的 {locked} 读不了、没有残留应当放行，打出那一句、检出记录多一行，实际 {outcome}，打出 {text!r}，检出 {entries}")
        # 主 agent 的工作根在而列不出（chmod 成 300：进得去、列不出）：照常放行，「工作根列不出」那一句打出来、检出记录多一行
        s = scenario("work-root-unlisted")
        s.owned("w-hidden")
        os.chmod(s.work, 0o300)
        try:
            outcome = s.decide(s.stop_input())
        finally:
            os.chmod(s.work, 0o700)
        detections = os.path.join(s.root, "detections-unlisted.jsonl")
        printed = io.StringIO()
        report_unjudged_pass(s.stop_input(), outcome.get("notes") or [], printed, detections)
        entries, text = detection_entries(detections), printed.getvalue()
        check("work-root-unlisted-pass-reported",
              outcome.get("action") == "pass" and f"工作根 {s.work} 列不出" in text and len(entries) == 1 and entries[0].get("outcome") == "只记",
              f"工作根 {s.work} 列不出应当放行，打出那一句、检出记录多一行，实际 {outcome}，打出 {text!r}，检出 {entries}")
        # 会话记录在而读不了（chmod 成 000）：判不了，照「放行但必记检出」走 skip（入口记检出那一步由 unjudged-skip-recorded 那一格核）
        s = scenario("transcript-unreadable")   # 没有残留：弄坏开关下读不了的会话记录被当成空的，照常放行、不记（有残留时后面读后台任务那一步抛异常，走「自己出错」那一支）
        s.owned("w-unreadable-transcript")
        os.chmod(s.main_transcript, 0)
        try:
            outcome = s.decide(s.stop_input())
        finally:
            os.chmod(s.main_transcript, 0o600)
        check("transcript-unreadable-skip-reported", outcome.get("action") == "skip" and "读不了" in outcome.get("text", ""),
              f"会话记录 {s.main_transcript} 读不了应当判不了（skip、说明写读不了），实际 {outcome}")
        # 迁库遗留（文件头「判哪几处」迁库遗留那一段）：判定库服务的状态目录里服务那份库不列、旁边的整份库拷贝列（写出大小）；
        # 数据根下按种类名认的旧文件库列，命令只点到它的上层目录也算
        s = scenario("leftover-data-root")
        state = s.prune_paths().state_directory
        service_library = s.library_copy(os.path.join(state, "library"))
        old_layout = s.library_copy(os.path.join(state, "library.old-layout-2000-01-01"))
        family = s.filled(os.path.join(s.data, "legacy", "singlefs-unit-verdicts"))
        s.bash(f"mv {service_library} {old_layout}")
        s.bash(f"mkdir -p {s.data}/legacy/ && mv {s.data}/singlefs-unit-verdicts {s.data}/legacy/")
        outcome = s.decide(s.stop_input())
        check("prune-state-library-copy-blocked", blocked_with(outcome, old_layout)
              and re.search(rf"  {KIND_LIBRARY}  {re.escape(old_layout)}  [0-9.,]+[KMGTP]?\n", outcome.get("text", "")) is not None,
              f"判定库服务状态目录里的整份库拷贝 {old_layout} 应当拦下并写出 du 的大小，实际 {outcome}")
        check("service-library-not-listed", service_library not in outcome.get("paths", []) and service_library not in [path for path, _ in outcome.get("unknown", [])],
              f"服务自己那份库 {service_library} 不该列（拦的单与「{UNKNOWN_SECTION_TITLE}」都不该有），实际 {outcome}")
        # 只看拦的单（paths）：拦下时「认不出是谁的」一节也写进说明，只查说明里有没有这条路径，没认作本会话的也照样过
        check("data-root-family-library-blocked", blocked_with(outcome, family) and family in outcome.get("paths", []),
              f"数据根下按种类名认的旧文件库 {family}（命令点到它的上层 {s.data}/legacy）应当拦下，实际 {outcome}")
        # 单元文件 ExecStart 的 --state-directory 指到别处：那里的 library 是服务那份、不列，旁边的拷贝列
        s = scenario("leftover-unit-file")
        unit_state = os.path.join(s.root, "unit-state")
        paths = s.prune_paths()
        os.makedirs(os.path.dirname(paths.unit_file), exist_ok=True)
        with open(paths.unit_file, "w", encoding="utf-8") as handle:
            handle.write(f"[Service]\nExecStart={paths.binary} serve --spawned-by-parent --state-directory {unit_state}\n")
        unit_library = s.library_copy(os.path.join(unit_state, "library"))
        before_migration = s.library_copy(os.path.join(unit_state, "library.before-migration"))
        s.bash(f"cp -a {unit_library} {before_migration}")
        outcome = s.decide(s.stop_input())
        check("unit-file-state-directory-read", blocked_with(outcome, before_migration) and unit_library not in outcome.get("paths", [])
              and unit_library not in [path for path, _ in outcome.get("unknown", [])],
              f"单元文件的 --state-directory 是 {unit_state}：里面的拷贝 {before_migration} 应当拦下、服务那份 {unit_library} 不列，实际 {outcome}")
        # 相对路径照同一条命令里前面 cd 到的目录解
        s = scenario("leftover-relative")
        moved = s.library_copy(os.path.join(s.data, "rocks-old"))
        s.bash(f"cd {s.data} && mv library-copy rocks-old")
        outcome = s.decide(s.stop_input())
        check("relative-after-cd-attributed", blocked_with(outcome, moved), f"cd {s.data} 之后写相对路径点到的 {moved} 应当算本会话的、拦下，实际 {outcome}")
        # 库备份压缩包：名字里带 verdict 的列，别的压缩包不列
        s = scenario("leftover-archive")
        flips = os.path.join(s.data, "family-flips")
        kept_archive = s.archive(os.path.join(flips, "unit-verdicts-old-dirs-2000-01-01.tar.gz"))
        report_archive = s.archive(os.path.join(flips, "report-2000-01-01.tar.gz"))
        s.bash(f"tar -czf {kept_archive} -C {s.data} old-dirs && tar -czf {report_archive} notes")
        outcome = s.decide(s.stop_input())
        check("library-archive-blocked", blocked_with(outcome, kept_archive) and report_archive not in str(outcome),
              f"库备份 {kept_archive} 应当拦下，名字里不带种类名、verdict、library 的 {report_archive} 不列，实际 {outcome}")
        # 工作根下没写归属文件的目录：本会话命令点到过的库拷贝拦，没点到的只进「认不出是谁的」
        s = scenario("leftover-no-owner")
        loose = s.library_copy(os.path.join(s.work, "w-loose", "service-library-before-migration"))
        s.bash(f"cp -a /nowhere/library {loose}")
        outcome = s.decide(s.stop_input())
        check("no-owner-mentioned-copy-blocked", blocked_with(outcome, loose), f"没写归属文件的工作根目录里、本会话命令点到过的库拷贝 {loose} 应当拦下，实际 {outcome}")
        s = scenario("leftover-no-owner-unmentioned")
        stray = s.library_copy(os.path.join(s.work, "w-stray", "library-copy"))
        alone = s.decide(s.stop_input())
        listed = "\n".join(unknown_lines(alone.get("unknown", [])))
        check("no-owner-unmentioned-reported-not-blocked",
              alone.get("action") == "pass" and stray in [path for path, _ in alone.get("unknown", [])]
              and stray in listed and listed.index(UNKNOWN_SECTION_TITLE) < listed.index(stray),
              f"没写归属文件的工作根目录里、没被点到的库拷贝 {stray} 不该拦，只进「{UNKNOWN_SECTION_TITLE}」一节，实际 {alone}")
        # 只点到锚点（数据根本身、字面前缀是数据根的通配、判定库服务的状态目录）不算点到下面的东西；用改动命令点，只读命令本来就不算
        s = scenario("leftover-anchor")
        stale = s.library_copy(os.path.join(s.data, "stale-library"))
        s.bash(f"mkdir -p {s.data} {s.prune_paths().state_directory} && rsync -a {s.data}/* /nowhere/backup/")
        outcome = s.decide(s.stop_input())
        check("anchor-mentions-not-attributed", outcome.get("action") == "pass" and stale not in outcome.get("paths", []),
              f"命令只点到数据根与 {s.data}/* 这种通配，{stale} 不该算本会话的，实际 {outcome}")
        # 只读命令（du、ls、wc、不带 -delete 的 find）那一段里的路径不算点到：只进「认不出是谁的」、不拦；
        # 同一条命令里 mv 那一段点到的、find 带 -delete 点到的照拦
        s = scenario("leftover-read-only")
        looked = s.filled(os.path.join(s.data, "legacy-looked", "singlefs-unit-verdicts"))
        s.bash(f"du -sh {looked} && ls {looked} | wc -l; find {looked} -name keys.json")
        looked_outcome = s.decide(s.stop_input())
        s = scenario("leftover-read-only-then-moved")
        moved = s.filled(os.path.join(s.data, "legacy-moved", "singlefs-unit-verdicts"))
        purged = s.filled(os.path.join(s.data, "legacy-purged", "singlefs-mutate-library"))
        s.bash(f"ls {moved} && mv {s.data}/staging {moved}")
        s.bash(f"find {purged} -name '*.tmp' -delete")
        moved_outcome = s.decide(s.stop_input())
        check("read-only-mentions-not-attributed",
              looked_outcome.get("action") == "pass" and looked in [path for path, _ in looked_outcome.get("unknown", [])]
              and looked not in looked_outcome.get("paths", [])
              and moved_outcome.get("action") == "block" and moved in moved_outcome.get("paths", []) and purged in moved_outcome.get("paths", []),
              f"只用 du、ls、find 点过的 {looked} 应当只进「{UNKNOWN_SECTION_TITLE}」、不拦，实际 {looked_outcome}；"
              f"同一条命令里 mv 点到的 {moved}、find -delete 点到的 {purged} 应当拦下，实际 {moved_outcome}")
        # 种类名现读三处（这里换成样本文件）：样本里的名字认，样本里没有的不认（不是钩子里抄的一份）
        s = scenario("leftover-species")
        samples = os.path.join(s.root, "species-samples")
        os.makedirs(samples)
        sample_sources = {"client": os.path.join(samples, "client.py"), "registry": os.path.join(samples, "registry.tsv"), "retired": os.path.join(samples, "retired.tsv")}
        with open(sample_sources["client"], "w", encoding="utf-8") as handle:
            handle.write('"""样本\n\n键空间（样本）：\n  族名：unit-verdicts（种类是 singlefs-sample-verdicts/ 下那一层）\n连哪一处：样本\n"""\n')
        with open(sample_sources["registry"], "w", encoding="utf-8") as handle:
            handle.write("# 样本\nscripts/sample.py\t-\tprune-server（unit-verdicts 族，键空间 unit-verdicts/singlefs-sample-ledger/…）\treuse-three-questions\n")
        with open(sample_sources["retired"], "w", encoding="utf-8") as handle:
            handle.write(f"# 样本\nsample-retired-store\t{RETIRED_DIRECTORY_KIND}\t删掉：样本\n")
        sampled = [s.filled(os.path.join(s.data, "old", name)) for name in ("singlefs-sample-verdicts", "singlefs-sample-ledger", "sample-retired-store")]
        unlisted = s.filled(os.path.join(s.data, "old", "singlefs-unit-verdicts"))
        s.bash("mkdir -p " + " ".join(sampled + [unlisted]))
        saved_sources = dict(SPECIES_SOURCES)
        SPECIES_SOURCES.update(sample_sources)
        try:
            outcome = s.decide(s.stop_input())
        finally:
            SPECIES_SOURCES.update(saved_sources)
        check("species-read-from-registries", blocked_with(outcome, *sampled) and unlisted not in outcome.get("text", ""),
              f"种类名照样本三处读：{sampled} 应当拦下，样本里没有的 {unlisted} 不该认，实际 {outcome}")
        # 种类名一处都读不出：照常放行，记一处没判全
        s = scenario("leftover-species-unreadable")
        SPECIES_SOURCES.update({key: os.path.join(s.root, f"missing-{key}") for key in SPECIES_SOURCES})
        try:
            outcome = s.decide(s.stop_input())
        finally:
            SPECIES_SOURCES.update(saved_sources)
        check("species-unreadable-pass-reported", outcome.get("action") == "pass" and any("种类名读不出" in note for note in outcome.get("notes") or []),
              f"种类名的三处都读不了应当放行并记「种类名读不出」一处没判全，实际 {outcome}")
        # 数据根取不到（变量没设、多机配置不在）：照常放行，记一处没判全
        s = scenario("leftover-data-root-unconfigured")
        outcome = s.decide(s.stop_input(), {module("lib_work_root").DATA_ROOT_VARIABLE: "",
                                            module("lib_work_root").MULTI_HOST_CONFIGURATION_VARIABLE: os.path.join(s.root, "missing-multi-host.env")})
        check("data-root-unconfigured-pass-reported", outcome.get("action") == "pass" and any("数据根取不到" in note for note in outcome.get("notes") or []),
              f"数据根取不到应当放行并记「数据根取不到」一处没判全，实际 {outcome}")
        # 在用的迁库遗留不列：旧文件库里的文件、库备份本身各有进程开着
        s = scenario("leftover-held")
        held_library = s.filled(os.path.join(s.data, "legacy-held", "singlefs-unit-verdicts"))
        held_archive = s.archive(os.path.join(s.data, "library-backup-held.tar.gz"))
        with open(os.path.join(held_library, "keys.log"), "w") as handle:
            s.hold(["sleep", "120"], stdout=handle)
        with open(held_archive, "ab") as handle:
            s.hold(["sleep", "120"], stdout=handle)
        s.bash(f"cp -a /nowhere/library {held_library} && cp /nowhere/backup.tar.gz {held_archive}")
        outcome = s.decide(s.stop_input())
        check("held-leftover-not-listed", outcome.get("action") == "pass" and held_library not in str(outcome) and held_archive not in str(outcome),
              f"有进程开着的 {held_library}、{held_archive} 不该列，实际 {outcome}")
        s.stop()
        # 数据根层的旧库靠认领目录里的 .deliverables 声明放不过（路径落到认领目录外的行不算）
        s = scenario("leftover-declared-outside")
        directory = s.owned("w-declare")
        kept_family = s.filled(os.path.join(s.data, "legacy-kept", "singlefs-mutate-library"))
        with open(os.path.join(directory, DECLARATION_FILE), "w", encoding="utf-8") as handle:
            handle.write(f"{os.path.relpath(os.path.join(s.data, 'legacy-kept'), directory)} 备份 想留着这份旧库\n")
        s.bash(f"mv /nowhere/old-library {kept_family}")
        outcome = s.decide(s.stop_input())
        check("data-root-leftover-not-declarable", blocked_with(outcome, kept_family),
              f"认领目录外的 {kept_family} 声明成备份也照拦，实际 {outcome}")
        # 认领目录里声明成备份的库备份照放行
        s = scenario("leftover-declared-archive")
        directory = s.owned("w-archive")
        s.archive(os.path.join(directory, "library-backup-2000-01-01.tar.gz"))
        with open(os.path.join(directory, DECLARATION_FILE), "w", encoding="utf-8") as handle:
            handle.write("library-backup-2000-01-01.tar.gz 备份 打补丁之前的旧库\n")
        outcome = s.decide(s.stop_input())
        check("declared-archive-passes", outcome.get("action") == "pass", f"认领目录里声明成备份的库备份应当放行，实际 {outcome}")
        # 数据根是工作根的上一层（本机的样子）：别的会话认领的目录照认领规则不判，命令点到过也不算
        s = scenario("leftover-other-session")
        theirs = s.library_copy(os.path.join(s.owned("w-theirs-leftover", session="s-somebody-else"), "copy"))
        s.bash(f"cp -a /nowhere/library {theirs}")
        outcome = s.decide(s.stop_input(), {module("lib_work_root").DATA_ROOT_VARIABLE: s.root})
        check("other-session-leftover-ignored", outcome.get("action") == "pass" and theirs not in str(outcome),
              f"别的会话认领的目录里的 {theirs} 不归迁库遗留判，实际 {outcome}")
        # 子 agent 只按它自己会话记录里的命令认：主会话记录点到的不算它的
        s = scenario("leftover-subagent")
        mine = s.filled(os.path.join(s.data, "legacy-mine", "singlefs-unit-verdicts"))
        main_side = s.filled(os.path.join(s.data, "legacy-main", "singlefs-mutate-library"))
        agent = s.subagent("aleftover", "派发", s.bash_call("toolu_leftover_mine", s.now - 20, s.now - 19,
                                                             f"mv {s.data}/singlefs-unit-verdicts {s.data}/legacy-mine/"))
        s.bash(f"mv {s.data}/singlefs-mutate-library {s.data}/legacy-main/")
        outcome = s.decide(s.subagent_stop_input("aleftover", agent))
        check("subagent-leftover-by-own-commands", blocked_with(outcome, mine) and main_side not in outcome.get("paths", []),
              f"子 agent 自己的命令点到的 {mine} 应当拦下，主会话记录点到的 {main_side} 不算它的，实际 {outcome}")
        # 数据根表里登记的在用项（下一层那一个）本身不按名字认成旧文件库
        s = scenario("leftover-registered-entry")
        entry = next(name for name in module("lib_work_root").SHARED_DATA_ENTRIES if re.fullmatch(SPECIES_TOKEN, name))
        live = s.filled(os.path.join(s.data, entry))
        registry_sample = os.path.join(s.root, "registry.tsv")
        with open(registry_sample, "w", encoding="utf-8") as handle:
            handle.write(f"# 样本\nscripts/sample.py\t-\tprune-server（unit-verdicts 族，键空间 unit-verdicts/{entry}/…）\treuse-three-questions\n")
        s.bash(f"cp -a /nowhere/library {live}")
        SPECIES_SOURCES["registry"] = registry_sample
        try:
            outcome = s.decide(s.stop_input())
        finally:
            SPECIES_SOURCES.update(saved_sources)
        check("registered-entry-not-listed", outcome.get("action") == "pass" and live not in str(outcome),
              f"数据根下登记的在用项 {live} 名字与种类名相同也不该列，实际 {outcome}")
        # 包装：起钩子本身（不带弄坏开关）
        clean_environment = {key: value for key, value in os.environ.items() if key != BREAK_VARIABLE}
        s = scenario("wrapper")
        target = s.build_directory(os.path.join(s.owned("w-wrap"), "target"))
        completed = subprocess.run(["bash", HOOK_PATH], input=json.dumps(s.stop_input()), capture_output=True, text=True,
                                   env=dict(clean_environment, **s.environment()))
        try:
            printed = json.loads(completed.stdout)
        except ValueError:
            printed = {}
        check("wrapper-stop-json", completed.returncode == 0 and printed.get("decision") == "block" and target in printed.get("reason", ""),
              f"Stop 上拦下应当退 0、打 decision=block 的 JSON，实际退 {completed.returncode}，输出 {completed.stdout!r}{completed.stderr!r}")
        mine = s.build_directory(os.path.join(s.owned("w-wrap-sub"), "target"))
        s.subagent("awrap", "派发", s.bash_call("toolu_claim", s.now - 80, s.now - 79, "bash scripts/hosts/work-directory-ownership.sh claim w-wrap-sub --purpose x"))
        completed = subprocess.run(["bash", HOOK_PATH], input=json.dumps(s.handback_input("awrap")), capture_output=True, text=True,
                                   env=dict(clean_environment, **s.environment()))
        check("wrapper-handback-exit-2", completed.returncode == 2 and mine in completed.stderr and completed.stdout == "",
              f"交回之前拦下应当退 2、清单写 stderr，实际退 {completed.returncode}，stderr {completed.stderr!r}")
        state = True
        for raw in ("", "不是 JSON", "[]", json.dumps({}), json.dumps({"hook_event_name": "Stop", "session_id": "../x", "transcript_path": "/nowhere.jsonl"})):
            completed = subprocess.run(["bash", HOOK_PATH], input=raw, capture_output=True, text=True, env=dict(clean_environment, **s.environment()))
            state = state and completed.returncode == 0 and completed.stdout == ""
        check("bad-input-passes", state, "读不出的输入、没有会话号、会话号带路径符号都应当放行（退 0、标准输出为空）")
        # 判不了的放行照「放行但必记检出」：读不出的输入、不是 JSON 对象、会话号带路径符号、找不到会话记录，各照常放行、stderr 写「没判」、检出记录各多一行（结局只记）。
        # 在进程里起 hook_main（弄坏开关 unjudged-skip-silent 只在自证里认，起钩子本身时带不进去）；另一格起钩子本身核退出码与 stderr
        import contextlib
        def hook_main_in_process(raw_bytes, environment):
            saved_stdin, saved_environment = sys.stdin, {key: os.environ.get(key) for key in environment}
            os.environ.update(environment)
            sys.stdin = io.TextIOWrapper(io.BytesIO(raw_bytes), encoding="utf-8")
            printed_out, printed_err = io.StringIO(), io.StringIO()
            try:
                with contextlib.redirect_stdout(printed_out), contextlib.redirect_stderr(printed_err):
                    code = hook_main()
            finally:
                sys.stdin = saved_stdin
                for key, value in saved_environment.items():
                    if value is None:
                        os.environ.pop(key, None)
                    else:
                        os.environ[key] = value
            return code, printed_out.getvalue(), printed_err.getvalue()
        s = scenario("unjudged-skip")
        detections = s.environment()[DETECTIONS_VARIABLE]
        unjudged_inputs = ("不是 JSON", "[]", json.dumps({"hook_event_name": "Stop", "session_id": "../x", "transcript_path": "/nowhere.jsonl"}),
                           json.dumps({"hook_event_name": "Stop", "session_id": s.session, "transcript_path": os.path.join(s.project, "missing.jsonl")}))
        outcomes = [hook_main_in_process(raw.encode("utf-8"), s.environment()) for raw in unjudged_inputs]
        entries = detection_entries(detections)
        check("unjudged-skip-recorded",
              all(code == 0 and out == "" and "没判" in err and "放行但必记检出" in err for code, out, err in outcomes) and len(entries) == len(unjudged_inputs)
              and all(entry.get("outcome") == "只记" and all(str(finding).startswith(UNJUDGED_SKIP_PREFIX) for finding in entry.get("findings") or [None])
                      for entry in entries)
              and entries[-1].get("session_id") == s.session,
              f"判不了的放行应当退 0、标准输出为空、stderr 写没判与放行但必记检出、检出记录每次多一行（结局只记），实际 {outcomes}，检出 {entries}")
        # 自己出错（decide 换成抛异常的替身）：照常放行，stderr 写出错与没判，检出记录多一行
        s = scenario("self-error")
        detections = s.environment()[DETECTIONS_VARIABLE]
        real_decide = decide
        def exploding_decide(hook_input):
            raise RuntimeError("自证造的出错")
        globals()["decide"] = exploding_decide
        try:
            code, out, err = hook_main_in_process(json.dumps(s.stop_input()).encode("utf-8"), s.environment())
        finally:
            globals()["decide"] = real_decide
        entries = detection_entries(detections)
        check("self-error-pass-recorded",
              code == 0 and out == "" and "自己出错" in err and len(entries) == 1 and entries[0].get("outcome") == "只记"
              and "自证造的出错" in " ".join(entries[0].get("findings") or []),
              f"自己出错应当照常放行、stderr 写出错、检出记录多一行（结局只记），实际退 {code}，stdout {out!r}，stderr {err[-600:]!r}，检出 {entries}")
        # 起钩子本身（不带弄坏开关）：判不了的照常退 0、标准输出为空，stderr 写没判与检出记进哪里，检出记录多一行
        s = scenario("wrapper-unjudged")
        detections = s.environment()[DETECTIONS_VARIABLE]
        completed = subprocess.run(["bash", HOOK_PATH], input="不是 JSON", capture_output=True, text=True, env=dict(clean_environment, **s.environment()))
        entries = detection_entries(detections)
        check("wrapper-unjudged-skip-recorded",
              completed.returncode == 0 and completed.stdout == "" and "没判" in completed.stderr and "检出记进" in completed.stderr
              and "不是 JSON" in completed.stderr and len(entries) == 1 and entries[0].get("session_id") == "读不到",
              f"读不出的输入应当退 0、标准输出为空、stderr 写没判、输入开头与检出记进哪里、检出记录多一行（会话号读不到），实际退 {completed.returncode}，"
              f"stdout {completed.stdout!r}，stderr {completed.stderr!r}，检出 {entries}")
        # 包装：扫到上限没看完、没有残留时钩子本身退 0、标准输出为空、stderr 打出那一句，检出记录多一行；不设上限（判全了）的放行什么都不打、不记
        s = scenario("wrapper-truncated")
        deep = s.owned("w-wrap-deep")
        for name in ("a", "b", "c", "d", "e", "f"):
            os.makedirs(os.path.join(deep, name, "inner"))
        detections = s.environment()[DETECTIONS_VARIABLE]
        completed = subprocess.run(["bash", HOOK_PATH], input=json.dumps(s.stop_input()), capture_output=True, text=True,
                                   env=dict(clean_environment, **s.environment(), **{SCAN_LIMIT_VARIABLE: "3"}))
        entries = detection_entries(detections)
        check("wrapper-truncated-pass-stderr",
              completed.returncode == 0 and completed.stdout == "" and f"看了 3 个目录到顶，这几处没看完：{deep}" in completed.stderr
              and len(entries) == 1 and entries[0].get("outcome") == "只记",
              f"扫到上限没看完、没有残留时应当退 0、stderr 打出没看完的几处、检出记录多一行，实际退 {completed.returncode}，stdout {completed.stdout!r}，"
              f"stderr {completed.stderr!r}，检出 {entries}")
        plain_environment = {key: value for key, value in clean_environment.items() if key != SCAN_LIMIT_VARIABLE}
        completed = subprocess.run(["bash", HOOK_PATH], input=json.dumps(s.stop_input()), capture_output=True, text=True,
                                   env=dict(plain_environment, **s.environment()))
        check("wrapper-plain-pass-silent",
              completed.returncode == 0 and completed.stdout == "" and completed.stderr == "" and len(detection_entries(detections)) == 1,
              f"判全了、没有残留的放行应当什么都不打、不记检出，实际退 {completed.returncode}，stdout {completed.stdout!r}，stderr {completed.stderr!r}，"
              f"检出 {detection_entries(detections)}")
        # 平常跑设了弄坏开关：退 2、点名变量只在自证里认（guard-off 设着时探的值只写关守卫项一项：同写会被守卫拒，单独一项才照认，这一格红）
        probe = GUARD_OFF_ITEM if broken(GUARD_OFF_ITEM) else "build-directory-unrecognized"
        completed = subprocess.run(["bash", HOOK_PATH, "--force", "--break-guard-probe-unknown-option"], input="", capture_output=True, text=True,
                                   env=dict(clean_environment, **{BREAK_VARIABLE: probe}))
        check("break-switch-refused-outside-selftest", completed.returncode == 2 and BREAK_VARIABLE in completed.stderr and "自证" in completed.stderr,
              f"平常跑设了 {BREAK_VARIABLE} 应当退 2 并点名它只在自证里认，实际退 {completed.returncode}，stderr {completed.stderr!r}")
        # 关守卫项与别的项同写：起钩子本身平常跑退 2、点名关守卫项只在唯一一项时认；自证进程里调守卫探同写也拒（guard-off-with-others 下照认，进程里那一格红）
        combined = f"{GUARD_OFF_ITEM},build-directory-unrecognized"
        completed = subprocess.run(["bash", HOOK_PATH, "--force", "--break-guard-probe-unknown-option"], input="", capture_output=True, text=True,
                                   env=dict(clean_environment, **{BREAK_VARIABLE: combined}))
        check("guard-off-combined-refused-outside-selftest",
              completed.returncode == 2 and GUARD_OFF_ITEM in completed.stderr and "唯一一项" in completed.stderr and BREAK_VARIABLE in completed.stderr,
              f"平常跑设了 {BREAK_VARIABLE}={combined} 应当退 2 并点名关守卫项 {GUARD_OFF_ITEM} 只在它是唯一一项时认，实际退 {completed.returncode}，stderr {completed.stderr!r}")
        refused_in_process = io.StringIO()
        try:
            with contextlib.redirect_stderr(refused_in_process):
                refuse_break_outside_selftest(combined, ["--force", "--break-guard-probe-unknown-option"])
            stop_code = None
        except SystemExit as stop:
            stop_code = stop.code
        check("guard-off-combined-refused-in-process", stop_code == 2 and "唯一一项" in refused_in_process.getvalue(),
              f"自证进程里拿 {combined} 与平常跑的参数调守卫应当退 2 并点名关守卫项只在唯一一项时认，实际 {stop_code}，stderr {refused_in_process.getvalue()!r}")
    finally:
        for made in scenarios:
            made.stop()
        shutil.rmtree(base, ignore_errors=True)
    return failures, checked[0]


def selftest_cells_main():
    failures, checked = selftest_cells()
    if failures:
        print(f"✗ 自证：{len(failures)} 格红（共判了 {checked} 格）")
        print("→ 怎么办：照上面每一格的下一步改，再单独跑 completion-cleanup-guard.sh --selftest")
        return 1
    print(f"✓ 各格都对（判了 {checked} 格）")
    return 0


def run_switch(switch):
    clean = {key: value for key, value in os.environ.items() if key != BREAK_VARIABLE}
    completed = subprocess.run(["bash", HOOK_PATH, "--selftest-cells"], capture_output=True, text=True, env=dict(clean, **{BREAK_VARIABLE: switch}), timeout=300)
    return completed.returncode, completed.stdout + completed.stderr


def selftest():
    if os.environ.get(BREAK_VARIABLE):
        failures, checked = selftest_cells()
        if not failures:
            print(f"✗ 自证：打开弄坏开关 {os.environ[BREAK_VARIABLE]} 之后 {checked} 格仍全绿")
            print("→ 怎么办：那个开关弄坏的那一处要真影响它点名的格；两边对不上就改开关或改格")
        return 1
    clean = {key: value for key, value in os.environ.items() if key != BREAK_VARIABLE}
    completed = subprocess.run(["bash", HOOK_PATH, "--selftest-cells"], capture_output=True, text=True, env=clean, timeout=300)
    if completed.returncode != 0:
        print("✗ 自证：不带弄坏开关时各格应当全绿，实际：")
        print(completed.stdout + completed.stderr)
        print("→ 怎么办：先让 --selftest-cells 不带弄坏开关跑绿，再回来跑 --selftest")
        return 1
    sys.path.insert(0, os.path.join(os.path.dirname(os.path.dirname(HOOK_DIR)), "scripts", "hosts"))
    import heavy_threads   # 开关轮的并行度取每台默认重活线程数（.claude/heavy-thread-default），不按核数另定
    try:
        workers = heavy_threads.default_heavy_threads()
    except heavy_threads.HeavyThreadDefaultError as error:
        print(f"✗ 自证：取不到每台默认重活线程数，开关轮没起：{error}")
        print(f"→ 怎么办：{error.howto}")
        return 1
    with concurrent.futures.ThreadPoolExecutor(max_workers=workers) as pool:
        results = list(pool.map(lambda pair: (pair, run_switch(pair[0])), BREAK_TABLE))
    failed = 0
    if len(results) != len(BREAK_TABLE):
        print(f"✗ 自证：派出去 {len(BREAK_TABLE)} 个开关、收回 {len(results)} 个")
        print("→ 怎么办：整轮作废，重跑 --selftest")
        return 1
    for (switch, cell), (code, output) in results:
        if code == 0 or f"✗ 自证 {cell}：" not in output:
            failed += 1
            print(f"✗ 自证：打开弄坏开关 {switch} 之后应当红在「{cell}」格，实际退 {code}，输出：{output[-1500:]!r}")
            print(f"→ 怎么办：{switch} 弄坏的那一处要真影响「{cell}」格的判定；两边对不上就改开关或改格")
    if failed:
        print(f"✗ 自证：{len(BREAK_TABLE)} 个弄坏开关里 {failed} 个没红在点名的格")
        print("→ 怎么办：照上面逐个改，再跑 --selftest")
        return 1
    summary = completed.stdout.strip().splitlines()[-1] if completed.stdout.strip() else ""
    print(f"✓ completion-cleanup-guard.sh 自证：{summary.lstrip('✓ ')}；{len(BREAK_TABLE)} 个弄坏开关各红在点名的格")
    return 0


def main():
    refuse_break_outside_selftest()
    if ARGUMENTS == ["--selftest"]:
        return selftest()
    if ARGUMENTS == ["--selftest-cells"]:
        return selftest_cells_main()
    return hook_main()


sys.exit(main())
PY
