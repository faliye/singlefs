"""task_server_lib.py 拆出的一份：路径、阈值、名字、表这些常量（总说明在 task_server_lib.py 文件头）。"""
import os
import re


SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
REPOSITORY_ROOT = os.path.dirname(SCRIPT_DIRECTORY)
HOSTS_DIRECTORY = os.path.join(REPOSITORY_ROOT, "scripts", "hosts")
HOOKS_DIRECTORY = os.path.join(REPOSITORY_ROOT, ".claude", "hooks")
CLIENT_SCRIPT = os.path.join(SCRIPT_DIRECTORY, "task")
CONTROL_SCRIPT = os.path.join(SCRIPT_DIRECTORY, "control")
SERVER_SCRIPT = os.path.join(SCRIPT_DIRECTORY, "task_server.py")
RUNNER_SCRIPT = os.path.join(SCRIPT_DIRECTORY, "attempt_runner.py")
UNIT_TEMPLATE = os.path.join(SCRIPT_DIRECTORY, "singlefs-task-server.service.in")
MULTI_HOST_EXAMPLE = os.path.join(REPOSITORY_ROOT, "multi-host.env.example")
MEMORY_WRAPPER = os.path.join(HOSTS_DIRECTORY, "run-with-memory-cap.sh")
PEER_RUNNER = os.path.join(HOSTS_DIRECTORY, "multi-host-run.sh")
GPU_SERVICE_CONTROL = os.path.join(REPOSITORY_ROOT, "gpu-verification-server", "control")   # 服务健康读卡坏时起它的 cards-failed（task_server.py 的 gpu_card_failures）
PRUNE_SERVER_CONTROL = os.path.join(REPOSITORY_ROOT, "prune-server", "control")   # 命令退 75 时起一次它的 check（文件头「错误类型」prune-server-unavailable）
THREAD_WRAPPER = os.path.join(HOSTS_DIRECTORY, "capped.sh")

MULTI_HOST_CONFIG_NAME = "multi-host.env"
MULTI_HOST_CONFIG_VARIABLE = "SINGLEFS_MULTI_HOST_CONFIG"
OPTIONAL_KEY_PREFIX = "TASK_SERVER_"
OPTIONAL_KEYS = ("TASK_SERVER_STATE_DIRECTORY", "TASK_SERVER_RUNTIME_DIRECTORY", "TASK_SERVER_SIGNING_KEY_FILE", "TASK_SERVER_BUILD_DIRECTORY")
IO_PRESSURE_LIMIT_KEY = "TASK_SERVER_IO_PRESSURE_LIMIT"   # 多机配置里磁盘压力阈值那一个键（文件头「路径」）：值不是路径，不在 OPTIONAL_KEYS 里（那几个逐个展开成路径）
IO_PRESSURE_SELFTEST_DEFAULT = 50.0   # 只在弄坏开关 io-pressure-limit-defaulted 下用：没写阈值时补的数
EXPANDED_VARIABLES = ("HOME", "XDG_RUNTIME_DIR", "XDG_STATE_HOME", "XDG_CONFIG_HOME", "XDG_DATA_HOME")
UNIT_NAME = "singlefs-task-server.service"
SOCKET_NAME = "server.sock"
DATABASE_NAME = "task-server.sqlite3"
REQUIRED_MODULES = ("fastapi", "uvicorn")
INSTALL_PACKAGES = "sudo apt install python3-fastapi python3-uvicorn（要 sudo，由用户跑）"
INSTALL_COMMAND = "python3 task-server/control install（先 --dry-run 看一遍要写什么）"
START_COMMAND = "python3 task-server/control start"
CONFIGURATION_STEP = "cp multi-host.env.example multi-host.env（在主工作树根上），照模板里的说明填好（双机、显卡那几个键按这台的实况写）"
SECRET_PATTERN = re.compile(r"[0-9a-f]{64}")
RUN_CONDITION_EXIT = 78
USAGE_EXIT = 2
VOLATILE_FILESYSTEMS = ("tmpfs", "ramfs")
SESSION_VARIABLE = "CLAUDE_CODE_SESSION_ID"
# 会话的收件 socket 与会话令牌（Claude Code 导出给 hook 与 Bash 命令；文件头「发消息给会话」）：task submit 读这两个交给服务，服务记在任务上，出事往那个 socket 发消息
INBOX_SOCKET_VARIABLE = "CLAUDE_CODE_MESSAGING_SOCKET"
INBOX_TOKEN_VARIABLE = "CLAUDE_CODE_MESSAGING_TOKEN"
INBOX_LOG_MARK = "TASK_SERVER_INBOX"   # 发消息没发出去时往那条任务最后一次尝试的日志里接的一行的开头（不算日志在涨：REASON_LOG_NOISE 与 task_server.py 的 log_content）
LONG_RUN_LOG_MARK = "TASK_SERVER_LONG_RUN"   # 服务往在跑那一次尝试的日志里接的长跑那两种行（没有会话的待确认、确认了）的开头（同样不算日志在涨）
HEARTBEAT_LOG_MARK = "TASK_SERVER_HEARTBEAT"   # 命令自己按固定间隔打的心跳行的开头（文件头「命令自己打的两种登记行」）：不算日志在涨（task_server.py 的 log_content），不进理由行的最后一行（REASON_LOG_NOISE）
VERDICT_LIBRARY_PROGRESS_LOG_MARK = "PRUNE_SERVER_CLIENT_PROGRESS"   # 判定库客户端在跟 prune-server 一问一答时按固定间隔打的进度行的开头（文件头「命令自己打的两种登记行」）：算日志在涨（与心跳行相反），不进理由行的最后一行（REASON_LOG_NOISE）
ADMITTED_VARIABLE = "RUN_WITH_MEMORY_CAP_ADMITTED_BY_TASK_SERVER"   # 内存包装只认服务起的那一层的准入标记（run-with-memory-cap.sh 文件头「只认服务直接起的那一层」）
THREAD_ADMITTED_VARIABLE = "CAPPED_ADMITTED_BY_TASK_SERVER"   # capped.sh 只认服务套的那一层（capped.sh 文件头；task_server.py 文件头 ③「起」）：两台都设
PEER_ADMITTED_VARIABLE = "MULTI_HOST_RUN_ADMITTED_BY_TASK_SERVER"   # 派第二台的那一次：multi-host-run.sh --peer 只收服务直接起的这一层（那份文件头 ①；task_server.py 文件头 ③「第二台」）
ATTEMPT_LABEL_VARIABLE = "TASK_SERVER_ATTEMPT_LABEL"   # 每次尝试交给命令的尝试标签（两台都给）：memory_peaks_lib 照 scope 里进程环境的它把 scope 对到尝试上（那份文件头「在跑的 scope」）
KEY_VARIABLE = "RUN_WITH_MEMORY_CAP_KEY"
TARGET_DIRECTORY_VARIABLES = ("CARGO_TARGET_DIR", "CARGO_BUILD_TARGET_DIR")
RESULT_MARK = "TASK_SERVER_RESULT"
SUSPECT_MARK = "TASK_SERVER_SUSPECT"   # wait 报疑似卡住的那一行（文件头「错误类型」表后的「疑似卡住」）
SUSPECT_EXIT = 231                      # wait 报疑似卡住退这个码：错误类型表里没用过；任务照跑，再起 wait 接着等
ATTENTION_MARK = "TASK_SERVER_ATTENTION"   # wait 报要处置的提醒的那一行（文件头「错误类型」表后的「需要处置」）
ATTENTION_TYPE = "needs-attention"         # 那一行的 type=；不是结局，不进 OUTCOME_TYPES
ATTENTION_EXIT = 230                       # 任务还没结局时 wait 报提醒退这个码：错误类型表与 SUSPECT_EXIT 里没用过；任务照旧在排或在跑，再起 wait 接着等
ATTENTION_WHY_LIMIT = 300                  # 那一行的 why= 至多这么多个字符（attention_line）：排队过久的原因把两台的内存、CPU 全写进去，动辄上千字；全文在发给会话的「原因：」一行
# 十八种提醒（task_server.py 的 attention_items 判，文件头「需要处置」）：排队过久、日志长期不涨而服务没停它、卡在 IO 上而服务没停它、
# 清理没收尾、分片父任务推不动、改主工作区的命令没正常结束、预算贴着整台、在每一台上都起不来、跑得久而核没用起来、按给的上限开并行、
# 连着三次撞上限还在重排、跑满两小时没确认、本机读不到在跑尝试的单元状态、
# 出结局时用时超过 7200 秒要拆、换钩子被上一趟没写完成的挡住、换钩子时上一趟还在跑、线程总数远超登记数、出结局时日志里有用例本次未跑
ATTENTION_KINDS = ("queued-too-long", "quiet-log-busy", "io-wait", "cleanup-pending", "unit-unreadable", "shard-advance-failed",
                   "workspace-write-interrupted", "oversized-budget", "long-run-idle-cores", "fills-any-cap", "repeated-cap-hits", "long-run-unconfirmed",
                   "long-run-split", "workspace-write-blocked", "previous-swap-running", "threads-over-registered", "gpu-cases-not-run", "booking-beyond-idle-machine")
BOUNCING_ATTENTION_KINDS = ()   # 提醒里动作是「打回」的（此刻没有：别的提醒都是「提醒照跑」；文件头「异常表」）
# 异常表（ANOMALY_TABLE_TEXT）：那几栏只写在那一个常量里，anomaly_table 照它读，代码里不另抄一份；下面几组是代码里分别处置的种类，自证 anomaly-table-matches-kinds 拿它们与表比
ANOMALY_ACTIONS = ("提交时拒", "打回", "提醒照跑", "服务自己修")
ANOMALY_TABLE_HEAD = "异常表（"
ANOMALY_ROW = re.compile(r"^  ([a-z][a-z-]*)｜([^｜]+)｜([^｜]+)｜([^｜]+)｜([^｜]+)$")
SUBMIT_REFUSAL_KINDS = ("environment-refused", "nested-wrapper", "target-directory-in-repository", "outside-shared-path", "shared-roots-outside",
                        "verdict-library-unserved", "temporary-service-heavy", "whole-workspace-cargo", "inbox-missing", "manual-from-session", "sessionless-unmarked", "full-gate-not-commit-entry", "confirm-counts-mismatch",
                        "whole-package-unconfirmed", "unnamed-run", "card-declaration", "copy-under-proof")
SUBMIT_WARNING_KINDS = ("repository-target-bypass", "unclaimed-results")
SERVICE_ANOMALY_KINDS = ("unconfirmed", "suspected-stall", "unclaimed-outcome", "tampered-unit-stop")
# 只给弄坏开关 attention-next-generic：每一种都退回这一句通用的出路
ANOMALY_GENERIC_ADVICE = "task-server/task status {task} 看在等什么，等不起 task-server/task cancel {task}、改小命令重交"
ANOMALY_TASK_PLACEHOLDER = "{task}"
BOUNCE_NEW_TASK_REMINDER = "不要使用原来的任务号，用 task-server/task submit 创建新的任务号"
ANOMALY_TABLE_LOADED = []   # 读过的那一份（anomaly_table）
ANOMALY_TABLE_TEXT = """异常表（封闭的表；一行一种，栏用「｜」隔开：种类｜动作｜判据｜修改意见｜没处置时）：服务判出的每一种异常——提交时拒与提交时警告、交了待确认、
  要处置的提醒（ATTENTION_KINDS）的类型是 needs-attention、wait 退 230、那一行以 TASK_SERVER_ATTENTION 起头（文件头「错误类型」）。
  错误类型表里除 succeeded 之外的每一种结局、要处置的提醒（ATTENTION_KINDS）、疑似卡住、结局没人取——都在这里有一行，怎么判、怎么办、没人管时怎么办只照这一行。
  动作是 ANOMALY_ACTIONS 四样之一：提交时拒（不进库、不跑）、打回（记结局、交给提交它的会话改了重交）、提醒照跑（服务不停、不改任务，只把这一行的修改意见交给会话）、
  服务自己修（重排、换卡、取消这类，修不成的照打回办）。要处置的提醒、结局（打回与服务自己修）、疑似卡住、结局没人取、没确认被取消、确认的数对不上与客户端读不到收件地址，
  修改意见就是 wait 打的 next= 那一句与发给会话的正文「出路：」那一行的头一段（{task} 换成任务号，后面另接这一回的具体数）：服务照 anomaly_advice 从这张表取，代码里不另写一份。
  提交时拒的其余九种与两种提交时警告，拒绝说明在判它的那一处写（要点名这一回的变量、路径、包或那几条结局），表里那一行写的是同一条出路，给人查；
  每一句点名这一种要核的东西，不写「看 status、等不起就取消改小重交」这种通用句。
  没处置时：时长判的提醒每翻一倍再推一回；门槛不随时长涨的（oversized-budget、fills-any-cap、long-run-unconfirmed、cleanup-pending、shard-advance-failed，
  ANOMALY_REPUSHED_KINDS）同一道门槛推过之后满 ANOMALY_REPUSH_SECONDS（30 分钟）还在的再推一回、之后每翻一倍再推，正文写前几次推了什么、此刻还差什么；
  repeated-cap-hits 每条任务只推一回、不再推（它不在 ANOMALY_REPUSHED_KINDS 里）；long-run-idle-cores、threads-over-registered 每次尝试只推一回，都不再推；
  long-run-split、workspace-write-blocked、previous-swap-running、gpu-cases-not-run、booking-beyond-idle-machine 只推一回、不再推；
  结局满 30 分钟没交给会话的照 unclaimed-outcome 那一行再推。自证 anomaly-table-matches-kinds 读这一段与 ATTENTION_KINDS、BOUNCING_ATTENTION_KINDS、OUTCOME_TYPES、SUBMIT_REFUSAL_KINDS 比，对不上判红。
  environment-refused｜提交时拒｜调用方环境里有白名单外的变量，或环境里、命令里设了退役的变量（RETIRED_VARIABLES）｜去掉拒绝说明点名的那个变量再交；命令真要它的写成 task-server/task submit -- env <名>=<值> <命令…>，退役的照说明里的新写法改｜不进库，不再推
  nested-wrapper｜提交时拒｜命令外层或 bash -c 里套了内存包装、派发器、清场或线程包装，或写了 --memory、--memory-cap、--threads、--peer、--local-only、SINGLEFS_THREAD_CAP｜删掉拒绝说明点名的那一层或那个参数再交：内存、线程、机器都由服务定，卡与清场归 GPU 批处理验证服务｜不进库，不再推
  target-directory-in-repository｜提交时拒｜调用方给的 CARGO_TARGET_DIR（环境里或命令里）指到仓里｜去掉它再交，编译目录由服务给；真要自己的给仓外、共享根下的路径｜不进库，不再推
  outside-shared-path｜提交时拒｜工作目录或命令里的绝对路径有不在多机配置 NFS_SHARED_ROOTS 任何一段下的（/tmp 下的同样算）｜把拒绝说明点名的那个路径改放共享根下（草稿放工作根 ~/.local/share/singlefs/work/ 下），再交｜不进库，不再推
  shared-roots-outside｜提交时拒｜交给命令的 SINGLEFS_DATA_ROOT、SINGLEFS_WORK_ROOT 有不在共享根下的｜照点名的那个变量改多机配置的 SHARED_DATA_ROOT，或去掉任务里带的那个变量，再交｜不进库，不再推
  verdict-library-unserved｜提交时拒｜命令点到 .claude/gate.d/verdict-library-scripts.tsv 里带判定库而第三列还没写 prune-server 的一行，或那张表读不了、写坏了｜把点名的那一族接进 prune-server、第三列翻成 prune-server（表写坏的先改对那一行），再交｜不进库，不再推
  temporary-service-heavy｜提交时拒｜临时 prune-server 包装里剥出的原命令会编 cargo｜去掉包装重交：验证包的 cargo test 对真服务判｜不进库，不再推
  whole-workspace-cargo｜提交时拒｜命令位置上会编的 cargo 带 --workspace / --all，或不带 -p 而照清单的走法落到虚拟清单｜点名要编的包 -p <包>（验证包写 -p singlefs-verification）；整仓要跑的走 scripts/gate/merge-and-verify.sh 或提交时的门禁｜不进库，不再推
  inbox-missing｜提交时拒｜客户端读不到会话的收件地址（环境里没有 CLAUDE_CODE_MESSAGING_SOCKET 与 CLAUDE_CODE_MESSAGING_TOKEN）｜从 Claude Code 会话的 Bash 里交（那里的环境带收件地址）；人手交的带 --manual；子 agent 不交任务，写进报告交回主 agent｜不进库，不再推
  full-gate-not-commit-entry｜提交时拒｜带了 --full-gate，而命令不是提交时的六道门禁、scripts/crash/crash-amplification.sh 或 scripts/gate/gate-staged.sh，或带了 --check、--item，或是只查不跑的写法（--selftest、--list、--list-items、--help 这类，query_only_entry）｜全量标识只给不指定范围的提交时门禁：去掉 --full-gate，指定范围的照常交；跑过两小时的照提醒 task-server/task confirm <号> --why 确认｜不进库，不再推
  manual-from-session｜提交时拒｜带了 --manual，而进程树上读得到提交它的会话｜去掉 --manual 再交：会话交的任务结局由服务推给会话｜不进库，不再推
  sessionless-unmarked｜提交时拒｜进程树上读不到会话，也没带 --manual（客户端环境里的收件地址不算数）｜人手交的写成 task-server/task submit --manual -- <命令…>，结局用 task-server/task wait <号> 取；不要填假的收件地址｜不进库，不再推
  whole-package-unconfirmed｜提交时拒｜命令是整包测试（cargo test 不点名 --test、--bin、带测试名的 --lib，或带 --lib --tests、--workspace；crash-point-replay.sh 的 --check 点了 feature-gated-tests、product-gpu-real-card-tests 或不带 --crash-cases-only 的 crash-case-replay；不给补丁目录或带 --all-packages 的 merge-and-verify.sh），或是不带选条参数的门禁与入口（UNSCOPED_RUN_ENTRIES，不指定范围），而没带 --whole-package-confirmed，或带了而确认文件的形式不合（不是共享根下读得到的绝对路径、缺 ①②③ 哪一段、哪一段不到 12 个字）；带 --full-gate 的提交时门禁（COMMIT_GATE_ENTRIES）不走这一条，其中不指定范围而没带 --full-gate 的（含崩溃放量）另拒，整包确认标识顶替不了｜改成只点名改到的测试目标与测试函数（不指定范围的加上 --check、--item 或实验号）；不指定范围的只在提交时跑，提交之外要跑先问用户；确实要整包或不指定范围的照 .claude/agent-common.md「日常开发的测试不跑整个包」确认三遍（①改到的文件与函数、②引到它们的测试目标与函数及 grep 命中、③为什么只点名②不够）写进共享根下的一份文件，带 --whole-package-confirmed <确认文件> 重交；带了照样受两小时提醒与四小时硬上限｜不进库，不再推
  unnamed-run｜提交时拒｜启动测试与验证没点名（unnamed_run_problem）：cargo test 没在 `--` 之后带 --exact 与测试全名；scripts/gate/merge-and-verify.sh、scripts/crash/crash-point-replay.sh、scripts/crash/crash-amplification.sh 没带 --item；scripts/mutation/prove-red.sh 没给变异名；门禁阶段 --item、--check 都没带，或 --check 点了起测试的格（TEST_STARTING_GATE_CELLS）而没带 --item，或门禁 crash-point-replay 带了 --item 而 --check 没把起整个验证包的格排掉（crash_gate_whole_package_cells）；带 --full-gate 或 --whole-package-confirmed 的不走这一条｜照拒绝说明给的样子改成点名改到的、红了的那几项（cargo test 写 -- <测试全名> --exact，入口与门禁写 --item <项>，门禁 crash-point-replay 点崩溃枚举用例再加 --check crash-case-replay --crash-cases-only，证红写变异名）再交；确实要不指定范围或整包跑的，提交时带 --full-gate，提交之外先问用户、照 .claude/agent-common.md「日常开发的测试不跑整个包」确认三遍、带 --whole-package-confirmed <确认文件> 重交｜不进库，不再推
  card-declaration｜提交时拒｜命令位置上起的脚本（bash -c 里命令位置上的每一条同样认）文件头写了领卡、要卡的声明（# task-server-card-lease:、# task-server-gpu-cards:、# task-server-gpu-arguments:；card_declaration_problem）｜卡归 GPU 批处理验证服务，任务不领卡：删掉那一行，脚本里要卡的那一段改成往判定库写待判行、交 GPU 服务判（.claude/rules/verification.md「卡只归 GPU 服务，任务不拿卡」），再照原命令重交｜不进库，不再推
  copy-under-proof｜提交时拒｜一份副本上有证红（scripts/mutation/prove-red.sh --copy <副本>）在排、在跑、留着还原日志 .prove-red-pending 或持着副本锁持有者记录 .prove-red.owner（300 秒内刷新过），而提交的命令会编译、工作目录或 cargo 起的目录落在这份副本里；或提交的是对这份副本的证红，而副本里已有会编译的任务在排、在跑（copy_proof_refusal）｜等那条证红或那条会编译的任务出结局（task-server/task wait <号>）再交，留着还原日志的重跑同一条证红命令让它写回；等不起的另拷一份副本（python3 scripts/hosts/minimal-copy.py <仓根> <草稿目录>/copy --with-test-inputs）在新副本里编｜不进库，不再推
  confirm-counts-mismatch｜提交时拒｜task confirm 给的项数或在排在跑的条数与服务此刻算的不同｜照拒绝说明打的确认单核这条命令的范围（点的项、测试目标）与这个会话在排在跑的那几条：范围对了照服务算的两个数再 task-server/task confirm {task}，不对的 task-server/task cancel {task}、改了重交｜照 unconfirmed 那一行
  repository-target-bypass｜提醒照跑｜命令里的词、或命令位置上起的本仓脚本正文指向仓根 target/ 下的二进制，绕过服务给的编译目录｜编译交给服务给的编译目录，二进制取这一趟编出来的位置（CARGO_TARGET_DIR 下）；入口脚本里删掉仓根 target/ 的路径，改完 task-server/task cancel {task}、重交｜提交回复、日志头与 status 三处照打，不另推
  unclaimed-results｜提醒照跑｜交任务时会话名下有父任务的结局满 UNCLAIMED_WARNING_MINUTES（30 分钟）没交给它｜task-server/task wait <号> 把点名的几条结局取走、照结局那一行的出路办完，再 task-server/task clean <号>｜每交一条照打；那几条结局照 unclaimed-outcome 那一行再推
  unconfirmed｜服务自己修｜交了而满 CONFIRM_TIMEOUT_SECONDS（30 分钟）没 task-server/task confirm（状态 awaiting-confirmation）｜核确认单（task-server/task status {task}）里的范围与项数：对的重交、照确认单 task-server/task confirm；不要了不用管｜服务取消（结局 cancelled）、推一条「没确认，已取消」；那一条结局没人取照 unclaimed-outcome 再推
  queued-too-long｜提醒照跑｜在排、从提交（重排的从上一次尝试结束）起排满 QUEUE_ATTENTION_MINUTES（30 分钟），排队原因已判出来｜task-server/task status {task} 核「在等：…」点名的是哪一台、哪一样（余量、排在哪条任务后面、编译目录占盘、第二台用不了）：等余量的核这一条的上限是不是比它该要的大，第二台用不了的看 python3 task-server/control check｜每翻一倍（60、120 分钟…）再推一回
  quiet-log-busy｜提醒照跑｜在跑、日志满 QUIET_ATTENTION_MINUTES（30 分钟）没涨，服务因 CPU 在涨或读不到 CPU 没停它，这段时间也没读到进展（子进程写着的文件在涨、持着的卡在忙）｜task-server/task log {task} 核最后一行停在哪一步，task-server/task status {task} 核在用核数：一直满核而日志不动的多半在回收里打转或死循环，核那一步是不是把输出攒着不打｜每翻一倍再推一回
  io-wait｜提醒照跑｜在跑、日志没涨的每一轮都有进程在 D 状态，连着满 IO_WAIT_ATTENTION_MINUTES（30 分钟）｜核这一次尝试所在那台的盘与 NFS：/proc/pressure/io 的 some avg60、谁在大量写盘（同台在跑的全目标编译这类）、第二台上共享根的 NFS 挂载在不在、rpc 等不等得回；IO 缓过来它自己接着跑｜每翻一倍再推一回
  cleanup-pending｜提醒照跑｜命令已退出而收尾回读还有进程或读不了（本机单元与 scope、第二台那一趟的清理回执），服务保留这一次的资源占用｜task-server/task log {task} 看留下的是哪几条命令行，核实脚本有没有不等子进程就退（后台 &、setsid 出去的没等齐）；改好之后重交，那几个进程停掉之后服务自己重查收尾，不要删进程号、退出码或清理状态文件来放行｜每次尝试推一回；推过满 30 分钟还在的再推，之后每翻一倍再推
  unit-unreadable｜提醒照跑｜在跑的尝试本机连着读不到它的单元状态（systemctl --user show 起不来、超时或没打 ActiveState）满 UNIT_UNREADABLE_ATTENTION_SECONDS（60 秒）；服务不杀、不收尾、不记结局，照记 running，不经 systemd 核到 cgroup 读得到且空、记下的进程号都不在才记 launcher-lost，核不出就一直等｜在本机核用户级 systemd：systemctl --user show -p LoadState -p ActiveState <依据里点名的单元> 答不答得上（用户管理器卡住、D-Bus 忙、XDG_RUNTIME_DIR 不对这类），修好之后服务照常判；进程还在不在照依据里点名的 cat <cgroup>/cgroup.procs 与 ls /proc/<进程号> 核，命令还在跑的等它跑完，核不出的先修好 cgroup 的读法；不要删尝试目录、起跑标记或改库来放行｜每翻一倍再推一回
  shard-advance-failed｜提醒照跑｜分片的父任务推下一步连着出错｜看 journalctl --user -u singlefs-task-server.service 里「推下一步时出错」那一句前后的栈，核计划 plan.json 与各片的产物目录读不读得通；修好之前任务 {task} 不往下走，等不起 task-server/task cancel {task}｜推不动的每一回推一回；推过满 30 分钟还在的再推，之后每翻一倍再推
  workspace-write-interrupted｜提醒照跑｜改主工作区的命令以撞上限、被挤掉、取消、卡住被停、满四小时硬上限被停、对不回来这类结局收尾或被信号杀，可能写了一半｜核这条命令改的文件或补丁目录（依据里点名）与 git status --short、git diff --stat：没写全的照原样补齐或退回，核过再重交｜随结局推一回；结局没人取照 unclaimed-outcome 再推
  oversized-budget｜提醒照跑｜排队那一刻在某一台上的记账不小于那台的整片界（可用上限的 WHOLE_FRACTION，85%）而不超过可用上限；在排的要排队那一轮对它判过「在每一台上都起不来」之后才报，那一判判出的 booking-beyond-idle-machine 开着的不报（起不来，起得来之后照报）｜核实这一趟的范围（点的项、测试目标）、分片与同时起的个数；确要这么多就照跑、task-server/task confirm {task} --why '<为什么>' 记照跑（记过不再再推），不要的 task-server/task cancel {task}、收小或分片再交｜推过满 30 分钟还在排、在跑的再推，之后每翻一倍再推
  booking-beyond-idle-machine｜提醒照跑｜在排，排队那一轮判出这一条在每一台上都起不来：每一台此刻放不下，而且就算服务在那台跑的任务全收尾、放出它们此刻占的内存，整机空闲或 slice 放得下的也不够它要的（记账收到可用上限，钉了上限、照记录定到整台的都算），或那台本来就判永远放不下；有一台量不到余量、连不上的不判，在跑的尝试读不到用量的按它的上限算放出多少；服务不让它挡排在它后面的任务，后面放得下的照起；它开着的这一轮同一条的 oversized-budget 不报（要的就是整台，这一项已包住它，起得来之后照报）｜核这一条的上限为什么贴着整台：钉了上限的看 python3 task-server/control pin-cap 报的钉值与多机配置的 SLICE_MEMORY_CAP_BY_HOST，不要这么大的 python3 task-server/control unpin-cap <命令…> 撤掉钉值；靠记录的核这把命令键记下的峰值是不是真这么大、范围是不是点多了；不要的 task-server/task cancel {task}、收小或分片再交；等整机空闲回到它要的量它自己会起｜只推一回、不再推
  long-run-unconfirmed｜提醒照跑｜不是带 --full-gate 的提交时门禁，各次尝试在跑的时长累计满 RUN_LIMIT_SECONDS（两小时；撞了内存上限重来的计时归零）而没确认，服务不停、不改任务｜核两件事：范围对不对（点名范围：--item、--check 少几格、点名测试目标；给补丁目录；或照服务分片；整轮的留到提交时），这条是不是确实要跑这么久；不对的 task-server/task cancel {task}、改了范围用 task-server/task submit 重交；对的照整行确认：task-server/task confirm {task} --why '<为什么要跑这么久、范围为什么是这些>'（--why 去掉空白至少 12 个字，只许提交它的会话）；确认放不过四小时硬上限，累计满四小时服务照停（over-four-hours）｜推过满 30 分钟还没确认的再推，之后每翻一倍再推；确认了不再推；没有会话的不推，写进任务日志与 status 那一行
  long-run-split｜提醒照跑｜任务（分片的按片）出结局，服务记的用时超过 LONG_RUN_SPLIT_SECONDS（7200 秒；撞了内存上限、照大一档重来的，从最后一次撞上限之后那一次起算），又不在 .claude/long-run-split-exempt 里｜照 .claude/rules/long-run-split.md「怎么做」办：这一件让它跑完、结局照常用，不为拆而取消；出结局的同一轮里把这一项拆成几项、每一项能单独点名、各自预计不超过 7200 秒，拆不开的优化那段算法、量到不超过；缩域、少点几项、填 --why、确认长跑都不算拆；那一项的代码归别的会话管的，照 .claude/main-agent.md「一轮怎么开、怎么收」第 15 条发给那个会话；调度记录写一行｜只推一回、不再推
  workspace-write-blocked｜提醒照跑｜换钩子的命令（scripts/gate/hook-swap.py，不带 --feed-only）退 7：上一趟换上没写完成（被杀或回滚不成，几份文件停在半截），拒绝再换、一个字节没写；报给被挡的这个会话｜先照这条任务日志里拒绝说明点名的状态核：python3 scripts/gate/hook-swap.py --read-outcome 读结局文件，起上一趟的会话在的就请它补写完成（--seal-outcome）或照反向规格退回；等不到它的主 agent 手工退回到换前再重交（不要使用原来的任务号，用 task-server/task submit 创建新的任务号）；上一趟是哪条任务看依据里点名的号｜只推一回、不再推
  previous-swap-running｜提醒照跑｜换钩子的命令（scripts/gate/hook-swap.py，不带 --feed-only）退 9：上一趟换上还没写完成，起它的进程还在这一台上跑（还在换或在验），拒绝再换、一个字节没写；或退 10：同一个 --scratch，或这个仓的真换（占的是结局文件，按仓只有一份）正被另一趟 hook-swap.py 占着，没建树、没写文件；报给被挡的这个会话｜这一趟一个字节没写，不用退回、不用补写，等占着的那一趟完了再把同一条命令原样交一遍（不要使用原来的任务号，用 task-server/task submit 创建新的任务号）；退 9 的等上一趟写完成：这条任务日志里的拒绝说明点名了那一趟的进程号与起了多少秒，等那个进程退出、python3 scripts/gate/hook-swap.py --read-outcome 读得到它写了完成再交，那个进程退了而结局文件还没写完成的，重交会退 7，照 workspace-write-blocked 那一行办；退 10 的等占着的那一趟结束再交；占着的是哪一种、能不能换草稿目录（只有草稿目录被占、两趟里又有一趟是干跑的——这一条自己是，或占着的那一趟是——才能换：干跑不占这个仓，真换先占这个仓），服务照这条任务日志里拒绝说明的行首认出来写在这一句后面（认不出的两种都写，照拒绝说明办）｜只推一回、不再推
  gpu-cases-not-run｜提醒照跑｜任务（分片的按片；取消的不判）出结局，最后那一次尝试的日志里有带 NOT_RUN_NO_GPU_ADAPTER 记号的行（每处点名一条判段没执行、拿占位结局就返回的用例：不带 gpu 特性而配置开了 GPU，或卡、显存借不到）：记号是整词、一行里每一处都认、一处一条（前面是行首、空白或别的非标识符字符，后面是空白或行尾，ANSI 颜色码先去掉；更长或更短的词不认），点名取这一处记号自己那一截里 test= 那一段；测试不带 --nocapture 时通过的测试打的字被测试框架收走、日志里没有这一行，服务读不到、不出这一项（没提醒不等于用例都跑了）；退出码与结局种类照命令的，命令退 0 仍是成功｜依据里点名的这几条用例判段没执行、不算过，这一趟报的 ok 不含它们：结局里别的用例照常用；要这几条的结论，带 gpu 特性重新提交（cargo test 加 --features gpu，用 task-server/task submit 创建新的任务号），或读 GPU 批处理验证服务对它们的结论（.claude/rules/verification.md「卡只归 GPU 服务，任务不拿卡」）｜只推一回、不再推
  long-run-idle-cores｜提醒照跑｜在跑满 LONG_RUN_ATTENTION_SECONDS（600 秒），到此刻平均在用核数不到给的线程数的 LONG_RUN_IDLE_CORE_FRACTION（24/31，线程数 31 时即不到 24 核）｜核实资源运用是否合理、脚本能不能并行或切片（该切片并行的照 .claude/rules/implementation-workflow.md「测试与崩溃检测优先多线程」）、范围是否点多（点的项、测试目标）；要改就 task-server/task cancel {task}、改了重交｜每次尝试只推一回，不随时长再推
  threads-over-registered｜提醒照跑｜在跑的尝试，服务每一轮读的线程读数里线程总数超过登记数的 THREADS_OVER_REGISTERED_FACTOR（2）倍、其中状态 R 的也超过登记数、占线程最多的那一个进程自己的线程数也超过登记数的 2 倍，连着 THREADS_OVER_REGISTERED_ROUNDS（3）轮；编译时一堆 rustc 进程加起来多的不算；只起一堆在睡的线程的编译器、链接器可跑的到不了登记数、不算；服务照跑、不停它｜测试或脚本该照登记的线程数开线程（.claude/rules/implementation-workflow.md「测试与崩溃检测优先多线程」），核是不是哪一处没读线程上限｜每次尝试只推一回，不随时长再推
  fills-any-cap｜提醒照跑｜同一命令键最近 FILLS_CAP_RUNS（3）次跑成的峰值都不低于各自上限的 FILLS_CAP_FRACTION（90%）；撞了上限的那几次不算（撞的那一次峰值本来就贴着上限），上限照记下的峰值定的那几次也不算（尝试的 prediction 一栏是 record：峰值贴着上限是定法的结果）｜这条命令按给的上限开并行，核实：同时起的个数要以这一趟的范围为界（不超过要判的单位数），不照服务交的上限（TASK_SERVER_MEMORY_CAP_BYTES）定｜推过满 30 分钟还在排、在跑的再推，之后每翻一倍再推
  repeated-cap-hits｜提醒照跑｜同一条任务末尾连着 LOOP_LEAK_REMINDER_CAP_HITS（3）次尝试结局都是 memory-cap-hit，服务还在乘 1.5 重排（在排，或下一次在跑）｜照 loop_leak= 那一段（正文「检查代码有没有逻辑循环或内存泄漏」那一行）逐类核代码：各次的峰值是不是一次比一次大、每次是不是都顶满、内存最高那一刻大头是堆还是页缓存；有循环或泄漏的 task-server/task cancel {task}、改了代码再交，规模本来就这么大的不用管，服务接着放大｜同一条任务只推一回；之后照撞上限的规矩重排，打回时随结局再推一回
  suspected-stall｜提醒照跑｜宽限过后整个单元的 CPU 满 STALL_SECONDS（10 分钟）平均不到一个核的 1%，要卡的这段时间卡上也没有它的计算负载｜task-server/task log {task} 核停在哪一步、在等什么（锁、子进程、网络）；还在算就不用管，真卡住了 task-server/task cancel {task}｜一段只推一回；CPU 又涨了算这一段结束，再静下来是下一段
  unclaimed-outcome｜服务自己修｜父任务的结局满 UNCLAIMED_WARNING_MINUTES（30 分钟）没交给提交它的会话（消息没送到，也没起 wait）｜task-server/task wait {task} 取走结局、照结局那一行的出路办完，再 task-server/task clean {task}｜每翻一倍（60、120 分钟…）再推一回，推到取走为止
  tampered-unit-stop｜服务自己修｜在跑的尝试核不过、对账记 ended 之后，服务停它的单元（名字合这个服务的单元名、又不是校验过的在跑尝试的才停）｜回读没停干净或单元名不认的，照告警点名的单元 systemctl --user status 核过再停；查是谁直接写了库；任务 {task} 要跑就用 task-server/task submit 创建新的任务号｜写告警；任务行核得过的另推一回给提交它的会话
  command-failed｜打回｜命令自己退非 0（表外的退出码都在这里）｜task-server/task log {task} 核最后一屏是哪一条断言、哪一步报错，改了命令或代码再重交（不要使用原来的任务号，用 task-server/task submit 创建新的任务号）｜随结局推一回；没人取照 unclaimed-outcome 再推
  memory-cap-hit｜服务自己修｜撞了这一条自己的上限（内存包装退 250），或判卡住那一刻顶在上限上｜服务乘 1.5 重排、至多 MEMORY_CAP_RETRIES 次，连着撞满 LOOP_LEAK_REMINDER_CAP_HITS（3）次推一回 repeated-cap-hits；次数用完打回的，照理由行 loop_leak= 那一段（推给会话的正文「检查代码有没有逻辑循环或内存泄漏」那一行）逐类核代码：各次的峰值是不是一次比一次大、每次是不是都顶满、内存最高那一刻大头是堆还是页缓存；有循环或泄漏的改了代码再重交，规模本来就这么大的把要的内存降下来（收小范围或分片）再重交（不要使用原来的任务号，用 task-server/task submit 创建新的任务号）｜重排的不推；打回的随结局推一回，没人取照 unclaimed-outcome 再推
  memory-cap-at-maximum｜打回｜撞上限而再乘 1.5 就到能派的每台的可用上限｜核实这一趟的范围（点的项、测试目标）、分片与同时起的个数，收小或分片再交（不要使用原来的任务号，用 task-server/task submit 创建新的任务号）；连着撞了 LOOP_LEAK_BOUNCE_CAP_HITS（2）次及以上的先照理由行 loop_leak= 那一段（推给会话的正文「检查代码有没有逻辑循环或内存泄漏」那一行）核代码有没有逻辑循环或内存泄漏：峰值一次比一次大、每次都顶满的多半不是范围大，有就改了代码再交｜随结局推一回；没人取照 unclaimed-outcome 再推
  squeezed-out｜服务自己修｜被 slice 的总上限挤掉（内存包装退 254）｜服务同一上限重排、至多 SQUEEZE_RETRIES 次；用完打回的，核 by= 点名的同台那几条是不是记账偏小，等它们跑完再重交（不要使用原来的任务号，用 task-server/task submit 创建新的任务号）｜重排的不推；打回的随结局推一回，没人取照 unclaimed-outcome 再推
  timed-out｜打回｜内存包装的限时到了（253）｜task-server/task log {task} 核卡在哪一步，把这一趟的范围拆小再重交（不要使用原来的任务号，用 task-server/task submit 创建新的任务号）｜随结局推一回；没人取照 unclaimed-outcome 再推
  wrapper-refused｜打回｜内存包装用法错或 scope 起不来（251）｜task-server/task log {task} 看内存包装打的最后那一句，照它修好（systemd --user 在不在、slice 配了没有）再重交（不要使用原来的任务号，用 task-server/task submit 创建新的任务号）｜随结局推一回；没人取照 unclaimed-outcome 再推
  run-condition-unmet｜打回｜运行条件不满足：命令退 78、入口会编的声明写坏了、钉了上限而配置读不到、能派的每一台都量不到余量｜照 why= 那一句点名的设备或配置（多机配置那一项、入口文件头）修好再重交（不要使用原来的任务号，用 task-server/task submit 创建新的任务号）｜随结局推一回；没人取照 unclaimed-outcome 再推
  not-run｜打回｜命令报本次未跑（77）｜task-server/task log {task} 核它报缺的是哪一项前提（设备、工具、输入），补齐再重交（不要使用原来的任务号，用 task-server/task submit 创建新的任务号）｜随结局推一回；没人取照 unclaimed-outcome 再推
  killed-externally｜服务自己修｜命令被不是服务发的 TERM 或 KILL 终止（earlyoom、内核 OOM 杀进程），这一台同一时段的 earlyoom 或内核日志里有这一次尝试进程树里某个进程的号｜服务同一上限重排、至多 EXTERNAL_KILL_RETRIES 次；用完打回的看 kill= 那一段（journalctl 里杀进程那一行）与这一台此刻的可用内存、swap，等机器空下来再重交（不要使用原来的任务号，用 task-server/task submit 创建新的任务号）｜重排的不推；打回的随结局推一回，没人取照 unclaimed-outcome 再推
  peer-run-failed｜服务自己修｜第二台那一趟没跑成（连不上、ssh 被重置、那边的准备没过）｜服务同一上限重排、至多 PEER_RUN_RETRIES 次；用完打回的看 python3 task-server/control check 与 /health 的 peer 一栏，修好第二台再重交（不要使用原来的任务号，用 task-server/task submit 创建新的任务号）｜重排的不推；打回的随结局推一回，没人取照 unclaimed-outcome 再推
  prune-server-unavailable｜打回｜命令自己退 75：prune-server 不可用｜看 python3 prune-server/control check 那一句修好 prune-server 再重交（不要使用原来的任务号，用 task-server/task submit 创建新的任务号）；不要改成走本地库｜随结局推一回；没人取照 unclaimed-outcome 再推
  launcher-lost｜打回｜尝试的单元不在了、又没留结果；或本机读不到单元状态时不经 systemd 核到这一次的 cgroup 读得到且空、记下的进程号都不在，又没留结果｜看 journalctl --user -u <unit= 点名的单元> 核单元是谁停的、有没有被 OOM 杀，照原样重交（不要使用原来的任务号，用 task-server/task submit 创建新的任务号）｜随结局推一回；没人取照 unclaimed-outcome 再推
  cancelled｜打回｜提交它的会话取消了，或交了满 30 分钟没确认被服务取消｜要跑就重交（不要使用原来的任务号，用 task-server/task submit 创建新的任务号）；没确认被取消的照确认单核过范围再 task-server/task confirm｜随结局推一回；没人取照 unclaimed-outcome 再推
  record-tampered｜打回｜库里这条记录校验不过｜查是谁绕过服务直接写了库（告警记录点名哪一行），不要使用原来的任务号，用 task-server/task submit 创建新的任务号｜随结局推一回；没人取照 unclaimed-outcome 再推
  never-fits｜打回｜哪台都等多久也放不下，或点名第二台而它用不了｜核 judged= 那一句点名的那一台为什么放不下（点名的那一台用不了、内存哪台都放不下），把命令要的内存收小再重交（不要使用原来的任务号，用 task-server/task submit 创建新的任务号）｜随结局推一回；没人取照 unclaimed-outcome 再推
  unit-start-failed｜打回｜尝试的单元起不来（systemd-run 退非 0）｜看 systemctl --user status 与 journalctl --user，核用户级 systemd 在不在、单元名有没有撞上，修好再重交（不要使用原来的任务号，用 task-server/task submit 创建新的任务号）｜随结局推一回；没人取照 unclaimed-outcome 再推
  shard-plan-refused｜打回｜分片的计划那一步退 0 而它写的计划读不通｜核 plan.json 在不在、是不是 JSON、片数超没超 MAXIMUM_SHARDS、键写没写错（文件头「分片」），改了计划那一步再重交（不要使用原来的任务号，用 task-server/task submit 创建新的任务号）｜随结局推一回；没人取照 unclaimed-outcome 再推
  stalled｜打回｜宽限过后日志与 CPU 都满 STALL_SECONDS（10 分钟）不涨，服务停了它｜task-server/task log {task} 核最后一行停在哪一步、那一步在等什么（锁、子进程、网络）、是不是把输出攒着不打；改了再重交（不要使用原来的任务号，用 task-server/task submit 创建新的任务号）｜随结局推一回；没人取照 unclaimed-outcome 再推
  over-four-hours｜打回｜没带 --full-gate 的任务，各次尝试在跑的时长累计（撞了内存上限重来的计时归零）满 RUN_HARD_LIMIT_SECONDS（四小时），服务停这一次尝试、不重排；分片的各片各按自己的累计算、整件不另判（排队串起来的挂钟不算），一片满界停那一片、兄弟片照跑，收齐后整件照那一片收尾；--why 与长跑已确认都放不过；改主工作区的命令同样停｜这一趟里有一项跑得太久：照 .claude/rules/long-run-split.md「怎么做」把它拆成几项、每一项能单独点名、各自预计不超过 7200 秒（一片超了的多分几片，拆不开的优化那段算法）；缩域、少点几项、填 --why、确认长跑都不算拆；拆好之后重交（不要使用原来的任务号，用 task-server/task submit 创建新的任务号）；task-server/task log {task} 看它走到哪一步，判定落了盘的命令重交只补没判的；改主工作区的命令先照同时推的 workspace-write-interrupted 那一条核现场，核过再重交｜随结局推一回；没人取照 unclaimed-outcome 再推
  resubmit-required｜打回｜服务规则改了，还在排的这一条照现行提交规则不再收｜照 rule= 那一条（提交时拒的哪一种与它的出路）改了再重交（不要使用原来的任务号，用 task-server/task submit 创建新的任务号）｜随结局推一回；没人取照 unclaimed-outcome 再推
"""
# 交任务的二次确认（task_server.py 文件头 ⑦）：交了的任务先在这个状态、不进队，task confirm 两个数对得上才进队
AWAITING_CONFIRMATION_STATE = "awaiting-confirmation"
RUN_HARD_LIMIT_OUTCOME = "over-four-hours"   # 四小时硬上限打回的结局（异常表 over-four-hours；task_server.py 文件头 ⑦「四小时硬上限」）
RUN_HARD_LIMIT_PHASE = "over-four-hours"     # 分片的父任务整件满了四小时硬上限的阶段（task_server.py 的 drive_parent_phase）：在排的撤、在跑的停，都收尾了记结局
LONG_TASK_SECONDS = 1800   # 只有长任务要确认（task_server.py 文件头 ⑦）：同一命令键库里近几次跑到退出的用时，最大的超过它就判成长任务
LONG_TASK_RECORD_REASON = "同一命令键库里记过的用时近 {count} 次最长 {seconds:.0f} 秒，超过 {limit:.0f} 秒"   # 确认单：判成长任务的依据之一
LONG_TASK_WHOLE_REASON = "不指定范围的整趟（{marks}）"                          # 确认单：判成长任务的依据之一，marks 是范围声明种类 whole、--full-gate、--whole-package-confirmed 里命中的
LONG_TASK_SHARD_REASON = "服务要拆成分片（入口 {entry}）"                        # 确认单：判成长任务的依据之一
LONG_TASK_UNRECORDED_REASON = "没有用时记录、又是{what}"                         # 确认单：判成长任务的依据之一，what 是下面两种之一
LONG_TASK_UNRECORDED_TESTS = "会编译的 cargo 测试"
LONG_TASK_UNRECORDED_ITEMS = "按项跑的门禁（--item）"
LONG_TASK_HELD_REASON = "{note}不分长短一律待确认"                                  # 判成长任务的依据：note 写是自证的小世界（CONFIRM_RULE_SELFTEST_VARIABLE）还是弄坏开关 short-task-held
SHORT_TASK_RECORD_REASON = "这把命令键记过的用时近 {count} 次最长 {seconds:.0f} 秒，不超过 {limit:.0f} 秒"   # 提交输出「直接进队：」那一行：不是长任务的依据之一
SHORT_TASK_UNRECORDED_REASON = "没有用时记录，但不是会编译的 cargo 测试，也不是按项跑的门禁（--item）"          # 提交输出「直接进队：」那一行：不是长任务的依据之一
DIRECT_QUEUE_MARK = "直接进队："                                                  # 客户端提交输出里不是长任务的那一行的开头（后面接 SHORT_TASK_*_REASON 之一）
SCOPE_KIND_PREFIX = "种类="                                                      # 范围声明 SINGLEFS_RUN_SCOPE 里点种类的那一段的开头（.claude/hooks/run-scope-guard.sh 文件头）
WHOLE_SCOPE_KIND = "whole"                                                       # 范围声明种类：不指定范围的整趟
CARGO_TEST_SUBCOMMANDS = ("test", "t")                                           # 判成会编译的 cargo 测试的子命令（cargo 的内建别名一并算）
CONFIRM_RULE_SELFTEST_VARIABLE = "TASK_SERVER_SELFTEST_CONFIRM_RULE"             # 只在自证里认：各格的小世界写 all（不分长短一律待确认，那几格演确认单）；判长短的几格写空串照真的判
CONFIRM_NEEDED_MARK = "TASK_SERVER confirm-needed"   # 客户端打确认单之后那一行（机器读的：自证的帮手照它确认）
CONFIRM_NEEDED_LINE = re.compile(r"^TASK_SERVER confirm-needed task=([0-9]+) items=([0-9]+) tasks=([0-9]+)(?: |$)", re.MULTILINE)
SCOPE_INSIDE_SCRIPT_TEXT = "范围由脚本里面定，服务算不出"   # 确认单：命令里没点、脚本里面才展开的范围（task_server.py 文件头 ⑦）
WHOLE_SCOPE_TEXT = "整包、不指定范围"                        # 确认单：一项都没点的整包测试、不带选条参数的门禁与入口、别的 cargo
CRASH_CASE_ITEM_PREFIX = "crash-case:"                       # --item 点的崩溃枚举用例的前缀（确认单数崩溃用例）
CRASH_DOMAIN_ARGUMENT = "--crash-expand-up-to"               # 崩溃域的缩法（scripts/gate/merge-and-verify.sh、scripts/crash/crash-point-replay.sh 文件头）
CRASH_SCOPE_ENTRIES = ("scripts/gate/merge-and-verify.sh", "scripts/crash/crash-point-replay.sh", ".claude/gate.d/crash-point-replay.sh")   # 确认单打崩溃用例与域的入口：没点的写算不出
NAMED_SCOPE_OPTIONS = ("--item", "--check", "--test", "--bin")   # 后面跟一个值的点名选项（--check 的值照逗号拆）；--lib 不带值，--exact 数 `--` 之后的测试名
LIBTEST_VALUE_OPTIONS = ("--skip", "--test-threads", "--format", "--color", "--logfile", "-Z")   # `--` 之后带值的 libtest 选项：值不算点名的测试
# 按会话号找收件 socket（task_server.py 文件头 ⑥「收件地址」）：会话登记的唯一取法是 scripts/agents/lib_session_registry.py，这里只载它
SESSION_REGISTRY_SCRIPT = os.path.join(REPOSITORY_ROOT, "scripts", "agents", "lib_session_registry.py")
SESSIONS_DIRECTORY_OVERRIDE = "TASK_SERVER_SELFTEST_SESSIONS_DIRECTORY"
SESSION_REGISTRY_LOADED = []   # 载过的那一份（session_registry_library）
MEMORY_CAP_VARIABLE = "TASK_SERVER_MEMORY_CAP_BYTES"   # 每次尝试交给命令的这一次内存上限（字节，十进制；名字已告诉别的会话，不改）
SOURCE_REPOSITORY_VARIABLE = "TASK_SERVER_SOURCE_REPOSITORY"   # 每次尝试交给命令的提交方那一份仓的根（本机绝对路径；第二台照它拼两台一致的路径，例会话记录目录）
# 每次尝试交给命令的两个共享根（文件头「资源记录」后的「共享根」）：第二台的 ~ 是另一个目录，经 ~ 取的工作根与共享数据落在第二台本地盘、两台读不到同一份
DATA_ROOT_VARIABLE = "SINGLEFS_DATA_ROOT"       # 共享数据根：照 .claude/hooks/lib_work_root.py 取的真路径（它照同一个名字读，没设取多机配置的 SHARED_DATA_ROOT）
DATA_ROOT_KEY = "SHARED_DATA_ROOT"              # 多机配置里写共享数据根的那一个键（lib_work_root.py 的 DATA_ROOT_CONFIGURATION_KEY；文件头「共享数据根」）
WORK_ROOT_VARIABLE = "SINGLEFS_WORK_ROOT"       # 工作根：lib_work_root.py 的 work_root() 在服务这一侧现算的真路径
SHARED_ROOT_VARIABLES = (DATA_ROOT_VARIABLE, WORK_ROOT_VARIABLE)
PIN_STATE = "pin"                       # history 表里钉值那一种的 state 与 kind（文件头「资源记录」的钉值）
PIN_SOURCE = "SLICE_MEMORY_CAP_BY_HOST"   # 钉值取多机配置里这个按台的键里派到的那一台那一项（每次定额度现读，不记字面的数）
PINNED_HISTORY = "pinned"               # 尝试记录 prediction 一栏：这一次的上限是钉值
PIN_UNREADABLE_HISTORY = "pin-unreadable"   # 钉了而配置读不到的那一份预测（不起，只在排队里认；文件头「资源记录」的钉值）
SUBMIT_MARK = "TASK_SERVER submit"
# 读不到单元状态时不经 systemd 核进程（task_server.py 文件头 ①「读不到单元状态」；attempt_presence_readings、attempt_presence_verdict）
PRESENCE_ALIVE = "alive"             # 有一个记下的进程号在，或有一个这一次的 cgroup 里有进程
PRESENCE_GONE = "gone"               # 都读得到：这一次的 cgroup 都空或已收掉、记下的进程号都不在
PRESENCE_UNVERIFIED = "unverified"   # 没有在的，又有一样核不出（cgroup 读不了、起跑标记不在、清理回执读不到、第二台连不上这类）
PRESENCE_UNPROBED = "unprobed"       # 后台线程还没核完头一回
PRESENCE_UNPROBED_TEXT = "还没核完头一回"
PRESENCE_WORDS = {PRESENCE_ALIVE: "还有进程在", PRESENCE_GONE: "进程都不在", PRESENCE_UNVERIFIED: "核不出", PRESENCE_UNPROBED: "还没核完"}
MEMORY_CAP_SCOPE_PREFIX = "singlefs-memory-cap-"   # 两台的内存包装给每条命令开的 scope 的名字起头（attempt_runner.py 的 SCOPE_PREFIX、peer-host-lib.sh 停的那一种）
# 在用户级管理器的 cgroup 树里往下找几层：单元在 app.slice/<单元>、内存包装的 scope 在 <slice>/<scope>，都在第二层；套在外层单元里跑的服务起的也一样，多留几层（推的）
ATTEMPT_PRESENCE_TREE_DEPTH = 6
# 第二台那一趟（ssh 起一份 python：读两份小文件、走一遍用户级管理器的 cgroup 树、读几份 cgroup.procs 与进程环境）的时限（推的：本机走一遍不到一秒，留足 ssh 建连）
ATTEMPT_PRESENCE_PEER_SECONDS = 60
ATTEMPT_PRESENCE_PROBE_MARK = "TASK_SERVER_ATTEMPT_PRESENCE "   # 第二台那一份读数打回来的那一行的行首（attempt_presence_probe_reply）
# 日志不涨时的进展读数（task_server.py 文件头 ①「要处置的提醒」的 quiet-log-busy；attempt_progress_readings）：
# 第二台那一趟（ssh 起一份 python：走一遍 /proc 里这一趟的进程树与各进程的 fd）的时限（推的：本机走一遍不到一秒，留足 ssh 建连）
ATTEMPT_PROGRESS_PEER_SECONDS = 30
ATTEMPT_PROGRESS_PROBE_MARK = "TASK_SERVER_ATTEMPT_PROGRESS "   # 第二台那一份读数打回来的那一行的行首（attempt_progress_probe_reply）
ATTEMPT_PROGRESS_FILE_LIMIT = 256   # 一趟至多记这么多份写着的文件（记满了余下的进程不再读，problems 里写一句；推的：测试与编译一趟写着的文件在几十份以内）
RUNNING_UNIT_STATES = ("active", "activating", "deactivating", "reloading", "refreshing")
HOST_NAMES = {"local": "本机", "peer": "第二台"}
# NFS 共享（文件头「NFS 共享」）
NFS_ROOTS_KEY = "NFS_SHARED_ROOTS"
NFS_ROOTS_MISSING_TEXT = "没有 NFS 共享 task-server 不起"   # 没写 NFS_SHARED_ROOTS 时 nfs_share_report 那一句（nfs-share-required 照它认出是 NFS 判法拒的）
HOSTS_KEY = "HOSTS"                    # 多机配置里各台地址的那一行（文件头「主机」）
# 服务的弄坏开关 → 转给共用取法 multi_host_addresses.py 的开关（文件头「路径」按台的三个键）
BY_HOST_FORWARDED_BREAKS = (("by-host-missing-accepted", "by-host-missing-unchecked"), ("by-host-outside-accepted", "by-host-outside-unchecked"))
MULTI_HOST_ADDRESSES_SCRIPT = os.path.join(HOSTS_DIRECTORY, "multi_host_addresses.py")   # 第二台地址的共用取法（文件头「主机」；peer_ssh_address）
ADDRESSES_LIBRARY = []                 # 载入过的那一份（multi_host_addresses_library）
NFS_EXPORTS_FILE = "/var/lib/nfs/etab"
NFS_PROBE_NAME = ".task-server-nfs-probe"
NFS_PEER_PROBE_NAME = ".task-server-nfs-probe-peer"
NFS_SSH_OPTIONS = ("-o", "BatchMode=yes", "-o", "ConnectTimeout=10")   # 与 scripts/crash/crash-point-replay-shard-configuration-check.sh 连第二台的同一组
# 服务起的 ssh 复用连接（文件头「主机」；enable_ssh_multiplexing）：控制 socket 放服务状态目录下的这个子目录，按 %C（本机、对端、端口、用户名的哈希，每台一份）分；
# 主连接最后一条会话关掉之后再留这么多秒（推的：服务的探测一两分钟一回，留五分钟够接上下一回，又不让停着服务时主连接留太久）
SSH_CONTROL_DIRECTORY_NAME = "ssh"
SSH_CONTROL_VARIABLE = "TASK_SERVER_SSH_CONTROL_DIRECTORY"   # 服务自己的环境里记控制目录（peer_shell_timed 照它接复用的三项；不传给尝试）
SSH_CONTROL_PERSIST_SECONDS = 300
SSH_CONTROL_HASH_CHARACTERS = 40   # ControlPath 里 %C 展开的长度（十六进制的 SHA1）
SSH_CONTROL_TEMPORARY_SUFFIX_CHARACTERS = 17   # ssh 建主连接时先在 ControlPath 后面接「.」加 16 个随机字符建临时 socket，再改名
SSH_SHIM_DIRECTORY_NAME = "bin"   # 控制目录下放复用层那一份 ssh 的子目录（enable_ssh_multiplexing 写；host-dispatch.py 那一路的 PATH 前面接它）
NFS_SSH_SECONDS = 30
NFS_RECHECK_PAUSE_SECONDS = 5.0   # 连第二台的 ssh 一次超时之后隔这么久复查（文件头「NFS 共享」的挂载）：第二台负载高时一次慢不当连不上
# 复查期：头一次 ssh 超时之后这么久之内复查连得上的记「慢」（只进服务日志与 /health），期满仍不通才算连不上（peer_shell_rechecked）；
# 两台的盘被写满那一阵握手超过 30 秒、几分钟后又回来，不群发「连不上」。推的：两分钟（四个 30 秒的 ssh）
NFS_SLOW_GRACE_SECONDS = 120.0
# control check --probe 现探 NFS 共享的总时长上限（秒；--limit 可改）：到点打出卡在哪一项、退 1。推的：一分钟（两台都堵时一趟现探 100 秒以上不出结论，各会话以为服务卡死）
NFS_LIVE_PROBE_LIMIT_SECONDS = 60.0
NFS_PROBE_READ_TRIES = 3   # 第二台 cat 本机刚写的探针：那边 NFS 客户端的目录缓存可能还没看见它，隔一秒再读，至多这么多次
NFS_README = "task-server/README.md「## NFS 共享」"
NFS_ITEMS = (("directory", "目录"), ("export", "导出"), ("mount", "挂载"), ("probe", "两向探针"))
NFS_PASSED = "过"
# 「挂载」那一项在第二台上起的判法（nfs_mount_verdict）：stat -f 打文件系统类型，NFS 是 nfs（v4 也是）
NFS_MOUNT_PROBE_TEXT = "stat -f -c %T --"
NFS_FILESYSTEM_TYPES = ("nfs", "nfs4")
NFS_MOUNT_RECOVERY = ("在第二台上 sudo systemctl reset-failed \"$(systemd-escape -p --suffix=mount {root})\"、ls {root} 触发自动挂载再挂一次；"
                      "fstab 那一行要带 x-systemd.automount、不带 x-systemd.idle-timeout（task-server/README.md「## NFS 共享」），改过之后 sudo systemctl daemon-reload")
# 打回的理由（文件头「错误类型」表后的「打回的理由」）：wait 在末行之前打的那一行与每种类型要带的要素（reason_fields 按这张表给值，自证 reason-line-per-type 逐种核）
REASON_MARK = "TASK_SERVER_REASON"
REASON_ELEMENTS = {
    "succeeded": ("exit",), "command-failed": ("exit", "host", "last_line"), "memory-cap-hit": ("host", "cap", "peak"),
    "memory-cap-at-maximum": ("host", "reached", "peak"), "squeezed-out": ("host", "cap", "by"),
    "timed-out": ("cap", "last_line"), "wrapper-refused": ("last_line",), "run-condition-unmet": ("last_line",), "not-run": ("last_line",),
    "peer-run-failed": ("host", "last_line"), "prune-server-unavailable": ("host", "last_line"), "launcher-lost": ("unit",),
    "cancelled": ("by",), "record-tampered": ("what",),
    "never-fits": ("judged",), "unit-start-failed": ("unit", "judged"), "resubmit-required": ("rule",), "stalled": ("judged",), "shard-plan-refused": ("judged",),
    "over-four-hours": ("judged",), "killed-externally": ("host", "kill"),
}
MEMORY_STAT_NAMES = ("anon", "file", "file_dirty", "file_writeback", "shmem")   # 内存最高那一刻存的 memory.stat 那几样（attempt_runner.py、multi-host-run.sh 的 MULTI_HOST_RUN_PEER_MEMORY_STAT、peak_memory_note）
# 跑满两小时提醒的豁免（文件头「异常表」long-run-unconfirmed）：提交时的六道门禁、每次提交必跑的崩溃放量全部分项与提交前整轮（仓根下的相对路径），交的时候带 task submit --full-gate 才豁免
COMMIT_GATE_ENTRIES = (".claude/gate.d/crash-point-replay.sh", ".claude/gate.d/qemu-device-streams.sh", ".claude/gate.d/lkmm.sh",
                       ".claude/gate.d/crates-mutation-replay.sh", ".claude/gate.d/formal-verification.sh", ".claude/gate.d/broad-mutation.sh",
                       "scripts/crash/crash-amplification.sh", "scripts/gate/gate-staged.sh")
SCOPE_RANGE_ARGUMENTS = ("--check", "--item")   # 带了就是指定范围，不是提交时那一趟
# 整包测试（task_server.py 文件头 ②「提交时拒」⑥；.claude/agent-common.md「日常开发的测试不跑整个包」）：crash-point-replay.sh 的 --check 点了这几格里的哪一格都算整包
# （crash-case-replay 带 CRASH_CASES_ONLY_ARGUMENT 的只判崩溃枚举用例，不算）
WHOLE_PACKAGE_CHECK_CELLS = ("crash-case-replay", "feature-gated-tests", "product-gpu-real-card-tests")
CRASH_CASES_ONLY_ARGUMENT = "--crash-cases-only"
WHOLE_PACKAGE_CRASH_GATE = ".claude/gate.d/crash-point-replay.sh"
WHOLE_PACKAGE_MERGE_ENTRY = "scripts/gate/merge-and-verify.sh"
MERGE_VALUE_OPTIONS = ("--crash-expand-up-to", "--item")   # merge-and-verify.sh 带值的选项（它文件头的用法行）：值不当补丁目录
MERGE_ALL_PACKAGES_ARGUMENT = "--all-packages"   # merge-and-verify.sh 带它的算整包（那份脚本自己也退 2、一样都不做；提交时先拒）
# 不带选条参数就跑全部项的门禁与入口（task_server.py 文件头 ②「提交时拒」⑥；.claude/main-agent.md「一轮怎么开、怎么收」：不指定范围的提交之外要跑先问用户）：
# {仓根下的相对路径: (带了哪一个就算指定了范围的选项, 给了不以 - 起头的词算不算指定了范围, 带了就不跑的选项)}。提交时的门禁由调用方先照 commit_gate_exempt 豁免。
UNSCOPED_RUN_ENTRIES = {
    **{entry: (SCOPE_RANGE_ARGUMENTS, False, ()) for entry in COMMIT_GATE_ENTRIES},
    "scripts/crash/crash-point-replay.sh": (("--item",), False, ()),
    "scripts/crash/crash-amplification.sh": (("--item",), False, ()),
    "scripts/experiment-apparatus/replay.sh": ((), True, ("--admission-only",)),   # 给实验号是指定范围；--admission-only 只问准入、不编不跑
}
UNSCOPED_NEXT_STEP = ("加上选条参数只跑要的那几项（--check <格>、--item <项>、实验号）；不指定范围的在提交时由提交流程跑，提交之外要跑先问用户，"
                      "用户点头了照 .claude/agent-common.md「日常开发的测试不跑整个包」确认三遍，带 --whole-package-confirmed <确认文件> 重交（守两小时）；"
                      "提交时的六道门禁、崩溃放量全部分项（scripts/crash/crash-amplification.sh）与 scripts/gate/gate-staged.sh 带 --full-gate 交")
# cargo 子命令后面带值的选项：值不当测试名（whole_package_cargo_test 认测试名用）
CARGO_VALUE_OPTIONS = ("-p", "--package", "-F", "--features", "--profile", "--target", "--test", "--bin", "--example", "--bench", "-j", "--jobs",
                       "--manifest-path", "--target-dir", "--color", "--message-format", "-Z", "--config", "--exclude")
WHOLE_PACKAGE_MARKS = ("①", "②", "③")   # 整包确认文件的三段（.claude/agent-common.md「日常开发的测试不跑整个包」的三遍确认）各以它起头
WHOLE_PACKAGE_SECTION_MINIMUM_CHARACTERS = 12   # 每段去掉空白（与段首的记号）至少这么多个字
WHOLE_PACKAGE_NEXT_STEP = ("改成只点名改到的测试目标与测试函数；确实要整包的照 .claude/agent-common.md「日常开发的测试不跑整个包」确认三遍，"
                           "带 --whole-package-confirmed <确认文件> 重交（task-server/task submit --whole-package-confirmed <确认文件> -- <命令…>）")
# 启动测试与验证不点名就拒（task_server.py 文件头 ②「提交时拒」⑦；异常表 unnamed-run；records/2026-11-17-subagent账本自上而下清欠.md 第二百二十九节「收口要做的」）：
# 判据只这一份 unnamed_run_problem，服务提交时拒那一处（task_server.py 的 unnamed_run_refusal）与范围声明闸 .claude/hooks/run-scope-guard.sh
# （lib_run_scope.py 经 lib_project_code_runs.task_server_library 载入这一份）都调它。
# 带 --item 才算点名的入口：{仓根下的相对路径: (点名的写法, 只查不跑的参数)}；--item 给几次都行，--item=<项> 也认
UNNAMED_RUN_ITEM_ENTRIES = {
    "scripts/gate/merge-and-verify.sh": ("--item <包>:<目标根文件>（可点的项用 --list-items 列）", frozenset({"--list-items", "--selftest", "--help", "-h"})),
    "scripts/crash/crash-point-replay.sh": ("--item <项>（可点的项用 --list-items 列）", frozenset({"--list-items", "--selftest", "--help", "-h"})),
    "scripts/crash/crash-amplification.sh": ("--item <分项>（可点的项用 --list-items 列）", frozenset({"--list-items", "--selftest", "--help", "-h"})),
}
UNNAMED_RUN_PROVE_RED_ENTRY = "scripts/mutation/prove-red.sh"   # 证红：--copy <副本> <crate> 之后至少一个变异名才算点名（那份文件头的用法行与它 prove_from_arguments 收参数的写法）
UNNAMED_RUN_PROVE_RED_QUERIES = frozenset({"--selftest", "--help", "-h"})
COPY_PROOF_PENDING_NAME = ".prove-red-pending"   # 证红在副本根上落的还原日志（scripts/mutation/prove-red.sh 的 RESTORE_JOURNAL_NAME）：在就是源码此刻带着变异
COPY_PROOF_OWNER_NAME = ".prove-red.owner"   # 证红在副本根上写的副本锁持有者记录（scripts/gate/apply-writer-patch.py 的 COPY_OWNER_NAME），持有期间每 15 秒刷新修改时间
COPY_PROOF_OWNER_STALE_SECONDS = 300   # 记录这么久没刷新就当持有者没了（scripts/gate/apply-writer-patch.py 的 COPY_OWNER_STALE_SECONDS）
UNNAMED_RUN_STAGE_QUERIES = frozenset({"--list", "--list-items", "--selftest", "--help", "-h", "--tool-versions"})   # 门禁阶段只打格名、项名、自证、帮助、工具版本
ENTRY_QUERY_ARGUMENTS = frozenset({"--list-items", "--selftest", "--help", "-h"})   # 门禁阶段之外的入口只列项、跑自证、打帮助（同 UNNAMED_RUN_ITEM_ENTRIES 各行的只查不跑；query_only_entry）
UNNAMED_RUN_CARGO_QUERIES = frozenset({"--no-run", "--help", "-h"})   # cargo test `--` 之前带它们的只编或只打帮助，不起测试（`--` 之后的 --list 只列用例，另判）
# 门禁阶段里起测试的格（起 cargo test、虚机、herd7、Verus 或实验装置；格名照各道文件头的格名表）：--check 点的格全不在这里才算点名，点了其中一格的要带 --item。
# 手列：各道文件头的格名表（`# gate-cell:` 行，--list 打的就是它）只写这一格判什么、不标起不起测试，从那里读不出哪几格起测试。
# 不在表里的阶段与格当不起测试；加了起测试的格照这张表加一行。自证 test-starting-cells-listed 核这里每个格名都在那一道的格名表里（格改名、删格时这一格红）。
# 读这张表一律经 test_starting_gate_cells（弄坏开关在那里）。
TEST_STARTING_GATE_CELLS = {
    ".claude/gate.d/crash-point-replay.sh": WHOLE_PACKAGE_CHECK_CELLS,
    ".claude/gate.d/qemu-device-streams.sh": ("device-streams",),
    ".claude/gate.d/lkmm.sh": ("litmus-verdicts",),
    ".claude/gate.d/crates-mutation-replay.sh": ("crates-mutation-rows",),
    ".claude/gate.d/formal-verification.sh": ("verus-proof", "verus-vacuity", "verus-must-red-worlds", "pinned-red-examples-replay"),
    ".claude/gate.d/model-differential-and-scenarios.sh": ("model-random-histories", "model-reuse-sampling", "model-rollback-sampling", "model-allocation-record-wall",
                                                           "model-unit-area-wall", "model-unit-area-wall-admission-judged"),
    ".claude/gate.d/research-build-and-replay.sh": ("research-unit-tests", "experiment-replay", "mutation-tables"),
}
# 门禁 crash-point-replay（WHOLE_PACKAGE_CRASH_GATE）带了 --item 还要 --check 把起整个验证包的格排掉（crash_gate_whole_package_cells；那份文件头「逐项点名」「--crash-cases-only」）：
# 项分两类，以这个前缀起头的是挂特性的测试文件（只限定 feature-gated-tests 格编、跑哪几个），别的是崩溃枚举用例（crash-case: 可写可不写）
CRASH_GATE_FEATURE_ITEM_PREFIX = "feature-gated:"
CRASH_GATE_CASES_ONLY_FORM = "--check crash-case-replay --crash-cases-only"   # 点崩溃枚举用例只走按块那一路、不起整个验证包的写法（拒绝说明照它给改法）
UNNAMED_RUN_PROBLEM_HEAD = "启动测试与验证没点名"   # 拒绝说明的头一段（服务与范围声明闸同一句）
UNNAMED_RUN_EXEMPTION_TEXT = ("确实要不指定范围或整包跑的：提交时那一趟照 .claude/skills/crash-test/commit-stages.md 带 --full-gate；提交之外先问用户，"
                              "用户点头了照 .claude/agent-common.md「日常开发的测试不跑整个包」确认三遍、带 --whole-package-confirmed <确认文件> 重交")
UNNAMED_RUN_SELFTEST_VARIABLE = "TASK_SERVER_SELFTEST_UNNAMED_RUN_REFUSAL"   # 只在自证里认：各格的小世界写 off（不判 ⑦，那几格的命令照旧收）；判 ⑦ 的几格写空串照真的
# multi-host-run.sh 打进日志的几行（文件头「错误类型」与「资源记录」）：第二台那条命令自己的退出码、第二台那一层的内存峰值与编译目录占盘峰值
PEER_COMMAND_END_LINE = re.compile(r"^MULTI_HOST_RUN_STEP step=peer-command event=end(?: \S+)* exit=([0-9]+)(?: |$)")
PEER_COMMAND_HOST = "peer-command"   # classify_attempt 的 host：第二台那条命令自己的退出码（255 也是 command-failed）
PEER_MEMORY_PEAK_MARK = "MULTI_HOST_RUN_PEER_MEMORY_PEAK"
PEER_BUILD_PEAK_MARK = "MULTI_HOST_RUN_PEER_BUILD_PEAK"
PEER_BUILD_KEPT_MARK = "MULTI_HOST_RUN_PEER_BUILD_KEPT"   # multi-host-run.sh 带 --keep-build-directory 跑完留下这一趟的编译目录时打「<它> label=<标签>」（它文件头「留着的编译目录」）
REASON_LOG_NOISE = ("MULTI_HOST_RUN_STEP ", "MULTI_HOST_RUN_PEER_CPU ", "MULTI_HOST_RUN_PEER_MEMORY ", "MULTI_HOST_RUN_PEER_MEMORY_STAT ", "MULTI_HOST_RUN_PEER_GPU ", "MULTI_HOST_RUN_PEER_IO_WAIT ", "MULTI_HOST_RUN_PEER_THREADS ", PEER_MEMORY_PEAK_MARK + " ", PEER_BUILD_PEAK_MARK + " ", PEER_BUILD_KEPT_MARK + " ",
                    INBOX_LOG_MARK + " ", LONG_RUN_LOG_MARK + " ", HEARTBEAT_LOG_MARK + " ", VERDICT_LIBRARY_PROGRESS_LOG_MARK + " ")   # 取日志最后一行时跳过的进度行（task_server.py 判日志涨不涨同样不算它们，判定库进度行除外：它算日志在涨）
# 临时 prune-server 的包装（scripts/prune-server/prune_server_client_selftest.py --with-temporary-service [--server-binary <路径>] -- <原命令>）：
# 算命令键、认登记表行时剥开、照 -- 后面那条原命令认（unwrapped_temporary_service）；包装本身照旧整条执行
TEMPORARY_SERVICE_SCRIPT = "prune_server_client_selftest.py"
TEMPORARY_SERVICE_KEY_MARK = "〈task-server 临时 prune-server〉"   # 包着这一层的命令键前面接它（文件头「资源记录」的命令键）：不与不包的那一条共用记录
TEMPORARY_SERVICE_FLAG = "--with-temporary-service"
TEMPORARY_SERVICE_OPTIONS = ("--server-binary",)
# 自带临时服务的两份脚本（.claude/hooks/selftest-temporary-service.tsv 里登记 own-server 的）：脚本词 → 它不带二进制就自己现编 prune-server 的自证模式词
# （都走 prune_server_client_selftest.py 的 resolve_binary：没有 --server-binary 与 PRUNE_SERVER_BINARY，有任务服务给的编译目录就现编，没有就退 2）
OWN_SERVER_SELFTESTS = {"prune_server_client_selftest.py": ("--selftest", "--selftest-families"), "prune-server-load-simulation.py": ("--selftest",)}
TEMPORARY_SERVICE_BINARY_VARIABLE = "PRUNE_SERVER_BINARY"   # 包装不带 --server-binary 时读它（scripts/prune-server/prune_server_client.py 的 BINARY_VARIABLE）：给了就不现编

# 分片（文件头「分片」）
SHARDED_ENTRIES = {   # 仓根下的相对路径 → 第一个参数认哪几个（None：除 NOT_SHARDED_ARGUMENTS 之外都分）
    "scripts/crash/crash-amplification-two-hosts.sh": None,
    "scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/rows_orchestrator.py": ("run",),
}
NOT_SHARDED_ARGUMENTS = ("--selftest", "--list-items", "--help", "-h")
DECLARED_SHARDS_MARK = "# task-server-shards:"
DECLARED_HEADER_LINES = 60   # 脚本文件头里的声明（分片、会编、续跑、卡的声明）只读前这么多行
# 入口会编（文件头「编译目录」的入口声明）：命令位置上起的脚本文件头写「# task-server-compiles: yes」，服务当它会编、照 cargo 那样给编译目录（撞上限重排接着用上一次那一份）
DECLARED_COMPILES_MARK = "# task-server-compiles:"
COMPILES_DECLARED_WORD = "yes"   # 那一行唯一认的词
# 卡归 GPU 批处理验证服务（.claude/rules/verification.md「卡只归 GPU 服务，任务不拿卡」）：命令位置上起的脚本文件头写了这几种领卡、要卡的声明之一，提交时当场拒
# （card_declaration_problem；task_server.py 文件头 ②「提交时拒」⑧），服务不认它们、不分卡、不清场
CARD_DECLARATION_MARKS = ("# task-server-card-lease:", "# task-server-gpu-cards:", "# task-server-gpu-arguments:")
CARD_DECLARATION_REFUSAL = "卡归 GPU 批处理验证服务，任务不领卡"   # 拒绝说明的头一句
CARD_CHOICE_VARIABLES = ("GPU_CARDS_BY_PRIORITY", "CUDA_VISIBLE_DEVICES")   # 选卡的变量：任务带的提交时拒（TAKEN_OVER_VARIABLES），服务自己环境里的不交给尝试
# 续跑（文件头「续跑」）：命令位置上起的脚本文件头写「# task-server-resumable: <词> [<词>…]」，第一个非选项参数是其中一个，就算这条命令登记了续跑（* 是任何参数）
DECLARED_RESUMABLE_MARK = "# task-server-resumable:"
LONG_RUN_SECONDS = 30 * 60   # 用户 2026-09-26：跑超过 30 分钟的要能中断续跑；跑这么久而失败、又没登记续跑的，结局说明写一句重交会从头跑
MAXIMUM_SHARDS = 64   # 按项分片的入口一项一片（崩溃放量全量 29 项，scripts/crash/crash-amplification.sh 文件头「经 task-server 按项分片」）
SHARD_PHASE_VARIABLE = "TASK_SERVER_SHARD_PHASE"
SHARD_MAXIMUM_VARIABLE = "TASK_SERVER_SHARD_MAXIMUM"   # 计划那一步：最多写几片（MAXIMUM_SHARDS），件多于它的由计划自己合进这几片（文件头「分片」①）
SHARD_VARIABLE = "TASK_SERVER_SHARD"
SHARD_PLAN_VARIABLE = "TASK_SERVER_SHARD_PLAN_DIRECTORY"
SHARD_OUTPUT_VARIABLE = "TASK_SERVER_SHARD_OUTPUT_DIRECTORY"
SHARD_OUTPUTS_VARIABLE = "TASK_SERVER_SHARD_OUTPUTS"
SHARD_HOST_VARIABLE = "TASK_SERVER_SHARD_HOST"
SHARD_MISSING_VARIABLE = "TASK_SERVER_SHARD_MISSING"   # 合并那一步：人手单独取消、没有产物的那几片的片号（文件头「分片」③）
SHARDS_ROOT_NAME = "task-server-shards"   # 工作根下放分片计划与各片产物的目录（文件头「分片」：两台照同一个绝对路径读写）
SHARD_PLAN_BINARIES_NAME = "test-binaries"   # 计划目录下计划那一步预先编好的测试二进制所在的子目录（文件头「分片」；同 .claude/gate.d/crash-point-replay.sh 的计划目录下那一份），父任务有了结局服务删
SCENES_ROOT_NAME = "task-server-scenes"   # 工作根下放判红那一趟现场的目录，每一次尝试一个 <尝试标签>（文件头「现场」）
SCENE_VARIABLE = "RUN_WITH_MEMORY_CAP_SCENE_DIRECTORY"   # 交给内存包装的这一次尝试的现场目录（文件头「现场」）
PEAK_OUTPUT_VARIABLE = "RUN_WITH_MEMORY_CAP_PEAK_OUTPUT"   # 交给内存包装的退出时峰值那一份（文件头「资源记录」的退出时峰值）
EXIT_PEAK_FILE_NAME = "exit_memory_peak"   # 尝试目录里内存包装写退出时峰值的那一份
MEMORY_PEAK_FILE_NAME = "memory_peak"   # 尝试目录里 attempt_runner 每轮写进程内存见过的最大的那一份（文件头「资源记录」的峰值的口径；task_server.py 的 scope_memory_peak 读它）
THREADS_FILE_NAME = "threads"   # 尝试目录里 attempt_runner 每轮写这一轮线程读数的那一份（JSON；attempt_runner.py 的同名常量与这里一起改；task_server.py 的 scope_thread_reading 读它）
PLAN_FILE = "plan.json"
SHARD_PLAN_KEYS = ("label",)
SHARD_ROLES = ("plan", "shard", "merge")   # 分片那三步的子任务（文件头「分片」）

# 环境变量名单（文件头「环境变量」）
CARRIED_VARIABLES = (
    "SINGLEFS_RUN_SCOPE", WORK_ROOT_VARIABLE, DATA_ROOT_VARIABLE, MULTI_HOST_CONFIG_VARIABLE, "SINGLEFS_KEEP_TEST_TEMPORARY_FILES",
    "PATH", "CARGO_HOME", "RUSTUP_HOME", "RUSTUP_TOOLCHAIN", "RUST_BACKTRACE", "RUST_LOG", "ROCKSDB_LIB_DIR", "ROCKSDB_STATIC", "SNAPPY_LIB_DIR", "SNAPPY_STATIC",
    "LANG", "LANGUAGE", "TMPDIR", "TZ", *TARGET_DIRECTORY_VARIABLES,
    KEY_VARIABLE, "RUN_WITH_MEMORY_CAP_TIME_LIMIT")
CARRIED_PREFIXES = ("LC_",)
AMBIENT_VARIABLES = (
    "_", "HOME", "USER", "LOGNAME", "SHELL", "PWD", "OLDPWD", "SHLVL", "TERM", "COLORTERM", "LS_COLORS", "LESSOPEN", "LESSCLOSE", "MAIL", "EDITOR", "VISUAL",
    "PAGER", "BROWSER", "GIT_EDITOR", "DISPLAY", "WAYLAND_DISPLAY", "XAUTHORITY", "DBUS_SESSION_BUS_ADDRESS", "DEBUGINFOD_URLS", "MOTD_SHOWN", "AI_AGENT",
    "APPLICATION_INSIGHTS_NO_STATSBEAT", "CODEX_HOME", "ELECTRON_RUN_AS_NODE", "NODE_EXTRA_CA_CERTS", "NoDefaultCurrentDirectoryInExePath", "SSL_CERT_FILE",
    "CLAUDECODE", "HOSTNAME", "HOSTTYPE", "OSTYPE", "MACHTYPE", "COLUMNS", "LINES", "INVOCATION_ID", "JOURNAL_STREAM", "MANAGERPID", "SYSTEMD_EXEC_PID",
    "PYTHONDONTWRITEBYTECODE", "DESKTOP_SESSION", "SESSION_MANAGER", "GPG_AGENT_INFO", "STY", "WINDOW")
AMBIENT_PREFIXES = ("CLAUDE_", "COPILOT_", "COREPACK_", "MCP_", "VSCODE_", "XDG_", "SSH_", "TERM_PROGRAM", "TMUX", "OTEL_", "ANTHROPIC_", "npm_",
                    "GNOME_", "QT_", "GTK_", "KRB5", "BASH_FUNC_", "TASK_SERVER_")
TAKEN_OVER_VARIABLES = {
    "RUN_WITH_MEMORY_CAP_WAIT_SECONDS": "排队归 task-server：任务在服务的库里排，内存包装不排队",
    "HOST_DISPATCH_WAIT_SECONDS": "挑机器归 task-server",
    ADMITTED_VARIABLE: "这个变量只由 task-server 设给内存包装",
    THREAD_ADMITTED_VARIABLE: "这个变量只由 task-server 设给它套的那一层 capped.sh",
    PEER_ADMITTED_VARIABLE: "这个变量只由 task-server 设给派第二台的那一层 multi-host-run.sh --peer",
    "RUN_WITH_MEMORY_CAP_OUTER_UNIT": "提交客户端是在别的内存包装的 scope 里起的：不在内存包装里面提交，在外面提交",
    "RUN_WITH_MEMORY_CAP_OUTER_CAP": "提交客户端是在别的内存包装的 scope 里起的：不在内存包装里面提交，在外面提交",
    **{name: f"{CARD_DECLARATION_REFUSAL}：任务不挑卡，卡上的活交 GPU 服务代跑" for name in CARD_CHOICE_VARIABLES},
}
TAKEN_OVER_PREFIXES = ("RUN_WITH_MEMORY_CAP_", "HOST_DISPATCH_")
# 退役的名字（文件头「环境变量」）：名字 → 为什么不收与出路；环境里设了、命令外层写了「名=值」都当场拒
RETIRED_VARIABLES = {
    "SINGLEFS_HEAVY_TESTS": "退役了：交给服务的命令不写这个前缀，不指定范围的重型测试照 .claude/rules/verification.md「一个门禁：两种跑法」由主 agent 在提交时交（提交之外要跑先问用户）：unset 它，命令里也不写",
    "GATE_MUTATION_START_OVER": "已删：crates 变异门禁全量只认 SINGLEFS_GATE_FULL=1：unset 它，要全量写 -- env SINGLEFS_GATE_FULL=1 <命令…>",
}
NESTED_DISPATCH_SCRIPTS = ("run-with-memory-cap.sh", "host-dispatch.py", "multi-host-run.sh")
# 用户定 M06：提交的命令不带包装前缀与脚本自己的上限、线程、挑台参数，由服务一处拒（records/2026-10-30-编译绕过内存总上限.md）
REFUSED_PREFIXES = {"nice": "优先级", "timeout": "限时", "capped.sh": "线程", "two-logs.sh": "日志", "gpu-quiesce-run.sh": "清场（清场与卡都归 GPU 批处理验证服务）"}
RESOURCE_ARGUMENTS = ("--memory", "--memory-cap", "--threads", "--peer", "--local-only")
WRAPPER_OWN_ARGUMENTS = (["--selftest"], ["--list-items"], ["--help"], ["-h"])

# 错误类型：名字 → (wait 的退出码，None 是照命令自己的；说明)。文件头「错误类型」是同一张表
OUTCOME_TYPES = {
    "succeeded": (0, "命令退 0"),
    "command-failed": (None, "命令自己退非 0（表外的退出码都在这里）"),
    "memory-cap-hit": (250, "撞了这一条自己的上限"),
    "memory-cap-at-maximum": (236, "撞上限、已到两台最大"),
    "squeezed-out": (254, "被总上限挤掉"),
    "timed-out": (253, "超时"),
    "wrapper-refused": (251, "内存包装用法错或 scope 起不来"),
    "run-condition-unmet": (78, "运行条件不满足"),
    "not-run": (77, "本次未跑"),
    "peer-run-failed": (255, "第二台没跑成"),
    "prune-server-unavailable": (75, "prune-server 不可用（命令自己退 75）"),
    "launcher-lost": (238, "对不回来"),
    "cancelled": (239, "被取消"),
    "record-tampered": (240, "库被改过（校验不过）"),
    "never-fits": (241, "哪台都等多久也放不下"),
    "unit-start-failed": (235, "尝试的单元起不来"),
    "shard-plan-refused": (234, "分片的计划读不通"),
    "stalled": (233, "日志满 10 分钟没涨，服务停了它"),
    "resubmit-required": (232, "服务规则改了，照现行规则不再收"),
    "over-four-hours": (229, "累计到四小时硬上限，服务打回"),
    "killed-externally": (228, "被不是服务发的 TERM 或 KILL 杀掉（earlyoom 或内核日志里有它的进程号）"),
}
# 推给会话的消息首行给每个种类的中文名，英文种类名留在括号里（task_server.py 的 outcome_headline、reminder_headline、suspect_headline）：
# 结局种类（OUTCOME_TYPES）、要处置的提醒（ATTENTION_KINDS）、疑似卡住与结局没人取各一项；自证 headline-names-cover-kinds 核这些种类一个不缺
KIND_NAMES = {
    "succeeded": "成功", "command-failed": "命令自己失败", "memory-cap-hit": "撞了内存上限", "memory-cap-at-maximum": "撞内存上限且已到最大",
    "squeezed-out": "被总上限挤掉", "timed-out": "超时", "wrapper-refused": "内存包装拒绝",
    "run-condition-unmet": "运行条件不满足", "not-run": "本次未跑", "peer-run-failed": "第二台没跑成", "prune-server-unavailable": "判定库服务不可用",
    "launcher-lost": "对不回来", "cancelled": "已取消", "record-tampered": "库被改过", "never-fits": "哪台都放不下",
    "unit-start-failed": "单元起不来", "shard-plan-refused": "分片计划读不通", "stalled": "卡住被停", "resubmit-required": "要重交",
    "over-four-hours": "超四小时被打回", "killed-externally": "被外部信号杀掉",
    "queued-too-long": "排队过久", "quiet-log-busy": "日志长期不涨但在忙", "io-wait": "卡在 IO 上",
    "cleanup-pending": "清理没收尾", "unit-unreadable": "读不到单元状态",
    "shard-advance-failed": "分片推不动", "workspace-write-interrupted": "改主工作区的命令没正常结束", "oversized-budget": "记账贴着整台", "booking-beyond-idle-machine": "在每一台上都起不来",
    "long-run-idle-cores": "跑得久而核没用起来", "fills-any-cap": "按给的上限开并行", "repeated-cap-hits": "连着撞上限", "long-run-unconfirmed": "跑满两小时没确认",
    "long-run-split": "跑过两小时要拆", "workspace-write-blocked": "换钩子被上一趟没写完成的挡住", "previous-swap-running": "换钩子时上一趟还在跑",
    "threads-over-registered": "线程远超登记数",
    "gpu-cases-not-run": "有用例本次未跑",
    "suspected-stall": "疑似卡住", "unclaimed-outcome": "结局还没人取",
}
LONG_RUN_SPLIT_SECONDS = 7200   # .claude/rules/long-run-split.md「判据」：一项一次运行的用时超过这么多秒，跑完就要拆
EXIT_TYPES = {0: "succeeded", 75: "prune-server-unavailable", 77: "not-run", 78: "run-condition-unmet", 250: "memory-cap-hit",
              251: "wrapper-refused", 253: "timed-out", 254: "squeezed-out"}
PRUNE_SERVER_UNAVAILABLE_OUTCOME = "prune-server-unavailable"   # 75：scripts/prune-server/prune_server_client.py 的 EXIT_UNAVAILABLE
PRUNE_SERVER_ALERT_KIND = "prune-server-unavailable"            # 命令退 75 而 prune-server/control check 不通时写的告警
GPU_CARD_FAILED_ALERT_KIND = "gpu-card-failed"                   # GPU 服务报了卡坏时写的告警（task_server.py 的 watch_health；同一张卡同一个服务进程只写一回）
GPU_BLOCK_NOT_JUDGEABLE_ALERT_KIND = "gpu-block-not-judgeable"   # GPU 服务报了判不了的块时写的告警（同上；同一台同一条流只写一回，不分服务进程）
# gpu-verification-server/control cards-failed 打的几种行（那份文件的 CARDS_FAILED_READ_PREFIX、FAILED_CARD_OF_A_HOST_PREFIX、BLOCK_NOT_JUDGEABLE_OF_A_HOST_PREFIX、
# CARDS_FAILED_UNREAD_PREFIX，逐字相同）：每台一行 <开头> host=<台> state=<状态> lines=<行数>；一行卡坏 <开头> host=<台> process=<进程号> <卡坏行原样>；
# 一行判不了 <开头> host=<台> process=<进程号> <判不了那一行原样>；一台没读成 <开头> host=<台> state=<状态>
GPU_CARDS_FAILED_READ_PREFIX = "GPU_VERIFICATION_SERVER_CARDS_FAILED_READ"
GPU_FAILED_CARD_OF_A_HOST_PREFIX = "GPU_VERIFICATION_SERVER_FAILED_CARD_OF_A_HOST"
GPU_BLOCK_NOT_JUDGEABLE_OF_A_HOST_PREFIX = "GPU_VERIFICATION_SERVER_BLOCK_NOT_JUDGEABLE_OF_A_HOST"
GPU_CARDS_FAILED_UNREAD_PREFIX = "GPU_VERIFICATION_SERVER_CARDS_FAILED_UNREAD"
GPU_CARDS_FAILED_READ_STATES = ("read", "not-running")   # 每台那一行里算读成了的状态（unreadable、unreachable 两种没读成，control 文件头 cards-failed）
GPU_THIS_HOST_LABEL = "this-host"   # cards-failed 里这一台（任务服务所在的这一台）写的台名
# 卡坏行里卡号那一栏、判不了那一行里认块的三栏与认领那一栏（gpu-verification-server/src/serving_from_the_library.rs 的 service_notice_line 卡坏那一支、
# block_not_judgeable_line；自证 gpu-card-lines-match-the-service-fields 从那份源码读字段名与认领的几种值来核）
GPU_CARD_FIELD = "card"
GPU_BLOCK_NOT_JUDGEABLE_KEY_FIELDS = ("family", "stream", "block")
GPU_BLOCK_CLAIM_FIELD = "claim"
GPU_BLOCK_CLAIMS_LET_GO = ("released", "not-held")   # claim= 里算放了的值（那份源码的 CLAIM_RELEASED、CLAIM_NOT_HELD）：另一台取得到
GPU_BLOCK_CLAIMS_KEPT = ("kept",)                     # claim= 里算没放的值（CLAIM_KEPT）：留在库里到认领租期过了
GPU_BLOCK_NUMBERS_SHOWN = 5   # 判不了那一条的正文列这条流头这么多块的块号，多的写还有几块
GPU_SERVICE_SOURCE = os.path.join(REPOSITORY_ROOT, "gpu-verification-server", "src", "serving_from_the_library.rs")

CAP_GROWTH_FACTOR = 1.5          # 用户定：撞上限就把额度乘 1.5 重排（records/2026-10-30-编译绕过内存总上限.md）；定额度照记录不乘它（近五次最大本身）
MEMORY_CAP_RETRIES = 16
STALL_AT_CAP_FRACTION = 0.98     # 判卡住那一刻 scope 的内存此刻或峰值到了上限的这么多：顶在上限上卡在回收里，按撞上限办（任务 1866 第 4 次：上限 13824M、峰值 13.50 GiB；
                                 # 任务 7723 第 2 次：memory.current 6442393600、memory.max 6442450944，CPU 在回收里打转；task_server.py 文件头 ①「顶在上限上卡住」）；
                                 # loop_leak_check_text 拿同一档判每一次撞上限是不是顶满（客户端拼打回理由也读它，所以定义在这一份）
LOOP_LEAK_BOUNCE_CAP_HITS = 2    # 连着撞上限之后打回（memory-cap-at-maximum、次数用完的 memory-cap-hit）：结局所在那几次尝试末尾连着撞上限不少于这么多次，
                                 # 理由行另带 loop_leak=、推给会话的结局正文另起一行（reason_fields）；只撞过一次的、排队那一刻记账就超上限一次都没起的不带
LOOP_LEAK_REMINDER_CAP_HITS = 3  # 还在重排的：同一条任务末尾连着这么多次尝试结局都是 memory-cap-hit，推一回要处置的提醒 repeated-cap-hits（task_server.py 的 repeated_cap_hits_item）
LOOP_LEAK_OUTCOMES = ("memory-cap-hit", "memory-cap-at-maximum")   # 理由行带 loop_leak= 的两种结局
LOOP_LEAK_CHECK_HEAD = "检查代码有没有逻辑循环或内存泄漏："   # loop_leak= 那一段的开头：结局正文、提醒正文与理由行同一句（loop_leak_check_text）
LOOP_LEAK_KINDS = ("逻辑循环（死循环、无界递归、反复往同一集合里加、每轮都重读全量）；"
                   "内存泄漏（只增不减的缓存或表、把大文件或全量结果整份读进内存、用完不释放的大对象）")   # 那一段写的要查的两类
SQUEEZE_RETRIES = 3
PEER_RUN_RETRIES = 2             # 第二台没跑成（连不上、ssh 被重置、那边的准备没过）不是任务的错：同一上限重排、不钉台
EXTERNAL_KILL_RETRIES = 2        # 被 earlyoom 或内核杀掉（killed-externally）不是任务的错：同一上限重排；机器还缺内存时再被杀，用完打回
EXTERNAL_KILL_OUTCOME = "killed-externally"
EXTERNAL_KILL_LINE = re.compile(r"sending SIG(?:TERM|KILL) to process (\d+)\b|Killed process (\d+) \(")   # earlyoom 与内核 OOM 杀进程时打的两种行
# 内核在某个内存组到了它自己的上限（memory.max）时打的那一行里的字样（「Memory cgroup out of memory: Killed process <号> (…」）：不是整机内存紧张，不算被外部信号杀掉
# （整机那一种是「Out of memory: Killed process」）。照原来的路判：命令自己退非 0，或这一条自己的 scope 撞了上限的照撞上限（external_kill_pid、task_server.py 的 peer_external_kill_note）
MEMORY_CGROUP_KILL_TEXT = "Memory cgroup out of memory"
RETRY_LIMITS = {"memory-cap-hit": MEMORY_CAP_RETRIES, "squeezed-out": SQUEEZE_RETRIES, "peer-run-failed": PEER_RUN_RETRIES, EXTERNAL_KILL_OUTCOME: EXTERNAL_KILL_RETRIES}
MEBIBYTE = 1024 * 1024
GIBIBYTE = 1024 * MEBIBYTE
# 没有记录的命令的预测、上限与记账（文件头「资源记录」）：用户选「没记录的先记小，撞了再放大」，主 agent 定 4G
# （records/2026-10-30-编译绕过内存总上限.md「问题清单的去向」#6）：任务 1–8 实测峰值 10 MiB 到 1.71 GiB；常见命令已从峰值表导入 3321 条记录
UNRECORDED_MEMORY_DEFAULT_BYTES = 4 * GIBIBYTE
# 内存上限的下限（文件头「资源记录」的内存上限下限）：记录的峰值只算进程内存（anon 加 shmem），上限还要装页缓存，读一个文件就撞顶；钉值不受它管
MEMORY_FLOOR_BYTES = 1024 * MEBIBYTE
MEMORY_FLOOR_RAISED_BYTES = UNRECORDED_MEMORY_DEFAULT_BYTES   # 只在弄坏开关 recorded-cap-floored 下当下限：有记录的上限也被抬到没记录的缺省
CPU_OVERSUBSCRIPTION_VARIABLE = "TASK_SERVER_SELFTEST_CPU_OVERSUBSCRIPTION"   # 只在自证里认：各格的小世界换 CPU 闸的倍数（task_server.py 的 cpu_judgement；各格照 2 搭的）
YOUNG_BOOKING_VARIABLE = "TASK_SERVER_SELFTEST_YOUNG_BOOKING_SECONDS"       # 只在自证里认：各格的小世界换「起跑不满这么久照整份上限记」的秒数（running_booking；0 是不看）
SLICE_ROOM_FLOOR_VARIABLE = "TASK_SERVER_SELFTEST_SLICE_ROOM_FLOOR_BYTES"   # 只在自证里认：各格的小世界换 slice 兜底的下限（task_server.py 的 slice_floor_problem；0 是不判）
MEMORY_FLOOR_VARIABLE = "TASK_SERVER_SELFTEST_MEMORY_FLOOR_BYTES"   # 只在自证里认：各格的小世界换这个下限（文件头「只在自证里认的变量」）
UNRECORDED_HISTORY = "unrecorded-default"   # 尝试记录 prediction 一栏：这一次是按没记录的小缺省定的
RECORD_HISTORY = "record"                   # 尝试记录 prediction 一栏：这一次的上限照服务自己记下的近几次最大定（文件头「资源记录」的定额度）
PROCESS_MEMORY_PEAK_METRIC = "进程内存"   # 尝试记录 peak_metric 一栏：memory_peak 是进程用的内存（anon 加 shmem；文件头「资源记录」的峰值口径一栏）；没有这一栏的旧行不读
# 命令键换算之前剥掉的环境变量前缀（文件头「资源记录」的命令键）：这几个只改配置路径、范围声明、语言这类，不改命令要多少内存
KEY_NEUTRAL_VARIABLES = ("SINGLEFS_MULTI_HOST_CONFIG", "SINGLEFS_RUN_SCOPE", "BINDGEN_EXTRA_CLANG_ARGS", "LANG")
# 分片计划那一步的命令键另外剥掉的环境变量前缀（key_words 的 plan_step）：缩域变量的值只改各片判多少崩溃状态，计划那一步只编、只切片，不随它变；各片的键仍带它
PLAN_KEY_NEUTRAL_VARIABLES = ("SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO",)
ASSIGNMENT_WORD = re.compile(r"[A-Za-z_][A-Za-z0-9_]*=", re.DOTALL)
# 定额度只用跑完的尝试（文件头「资源记录」）：退 0 的记量到的峰值，撞了这一条自己上限的记「至少要到这个上限」；别的结局的峰值偏小或不作数，不进
# 结局 cancelled 的理由行里「谁撤的」（reason_fields 的 by；文件头「分片」的只取消一片）：照子任务记的 cancel_origin 取；没记的是提交它的会话取消的
CANCEL_ORIGIN_TEXTS = {
    "": "提交它的会话（task-server/task cancel）",
    "session": "提交它的会话（task-server/task cancel 这一片）",
    "parent": "服务撤的：整件被取消（提交它的会话 task-server/task cancel 父任务），这一条没起或停了",
    "sibling": "服务撤的：同一件的兄弟片没成，这一片没起",
    "hard-limit": "服务撤的：整件到了四小时硬上限，这一片没起",
}
RECORD_COMPLETED_OUTCOMES = ("succeeded",)
# 上次运行时长取的尝试（文件头「资源记录」的上次运行时长）：命令自己跑到退出的两种结局；被信号杀的 command-failed 由 task_server.py 的 completed_runs 另排掉
RUN_COMPLETED_OUTCOMES = ("succeeded", "command-failed")
FAILED_RUN_RECORD_MINIMUM_SECONDS = 600.0   # 没有退 0 的记录时，退非 0 的尝试跑满这么久才当「上次跑了多久」的记录（task_server.py 的 last_run_seconds，插队规则用）：一开始就退 1 的不说明这条命令要跑多久
RECORD_CAP_HIT_OUTCOMES = ("memory-cap-hit", "memory-cap-at-maximum")
CAP_HIT_REAL_PEAK_FRACTION = 0.5   # 撞上限那一次峰值到了上限的这么多，才算真要这么多、按上限进定额度；不到的（脚本开跑前预判装不下就退 250）按量到的峰值进（cap_hit_record_bytes）
# Linux 的 sockaddr_un.sun_path 是 108 字节；留一个结尾的零，路径本身至多 107 字节（文件头「路径」）
SOCKET_PATH_MAXIMUM_BYTES = 107

SELFTEST_MARK_VARIABLE = "TASK_SERVER_SELFTEST_MARK"
SELFTEST_MARK_TEXT = "task-server-selftest"
BREAK_VARIABLE = "TASK_SERVER_BREAK"
SELFTEST_ONLY_VARIABLES = (
    BREAK_VARIABLE, "TASK_SERVER_SELFTEST_FAST", "TASK_SERVER_SELFTEST_WORLD", "TASK_SERVER_FAKE_RESOURCES", "TASK_SERVER_MEMORY_WRAPPER",
    "TASK_SERVER_PEER_RUNNER", "TASK_SERVER_SYSTEMD_RUN", "TASK_SERVER_CONTROL_SYSTEMCTL",
    "TASK_SERVER_FAKE_SCOPE_DIRECTORY", "TASK_SERVER_FAKE_REPOSITORY_ROOT", "TASK_SERVER_SESSION_BOUNDARY_PID", "TASK_SERVER_SELFTEST_MISSING_MODULES",
    "TASK_SERVER_SELFTEST_CRASH_POINT", "TASK_SERVER_SELFTEST_FAIL_HEALTH_WRITE", "TASK_SERVER_SELFTEST_ALLOW_TMPFS", "TASK_SERVER_SELFTEST_SOCKET_ROOT",
    "TASK_SERVER_SELFTEST_AGED_SECONDS", "TASK_SERVER_SELFTEST_PLACEMENT_PAUSED", "TASK_SERVER_SELFTEST_PLACEMENT_REFUSED", "TASK_SERVER_SELFTEST_STALL_SECONDS", "TASK_SERVER_SELFTEST_STALL_GRACE_SECONDS",
    "TASK_SERVER_SELFTEST_SLOW_RESOURCES_SECONDS", "TASK_SERVER_NFS_EXPORTS", "TASK_SERVER_PEER_SSH", "TASK_SERVER_SELFTEST_PAUSE_POINT",
    "TASK_SERVER_SELFTEST_STOP_FAILS", "TASK_SERVER_SELFTEST_START_TIMEOUT_SECONDS", "TASK_SERVER_PRUNE_SERVER_CHECK", "TASK_SERVER_GPU_SERVICE_CONTROL", MEMORY_FLOOR_VARIABLE,
    "TASK_SERVER_SELFTEST_LONG_RUN_SECONDS", "TASK_SERVER_SELFTEST_QUEUE_ATTENTION_SECONDS",
    "TASK_SERVER_SELFTEST_QUIET_ATTENTION_SECONDS", "TASK_SERVER_SELFTEST_ADVANCE_FAILS", "TASK_SERVER_SELFTEST_IO_WAIT_ATTENTION_SECONDS",
    "TASK_SERVER_SELFTEST_PROC_ROOT", "TASK_SERVER_SELFTEST_PROGRESS_PUSH_SECONDS", "TASK_SERVER_SELFTEST_UNCLAIMED_WARNING_SECONDS",
    "TASK_SERVER_SELFTEST_LONG_RUN_ATTENTION_SECONDS", "TASK_SERVER_SELFTEST_SESSIONS_DIRECTORY", "TASK_SERVER_SELFTEST_PROCESS_LIST", "TASK_SERVER_SELFTEST_RUNAWAY_RENOTIFY_SECONDS", "TASK_SERVER_SELFTEST_CONFIRM_TIMEOUT_SECONDS",
    "TASK_SERVER_SELFTEST_ANOMALY_REPUSH_SECONDS", "TASK_SERVER_SELFTEST_NFS_SSH_SECONDS", "TASK_SERVER_SELFTEST_NFS_RECHECK_PAUSE_SECONDS", "TASK_SERVER_SELFTEST_NFS_SLOW_GRACE_SECONDS",
    "TASK_SERVER_SELFTEST_RUN_LIMIT_SECONDS", "TASK_SERVER_SELFTEST_RUN_HARD_LIMIT_SECONDS",
    "TASK_SERVER_JOURNALCTL", "TASK_SERVER_SELFTEST_GPU_CARDS_FAILED", CPU_OVERSUBSCRIPTION_VARIABLE, YOUNG_BOOKING_VARIABLE, UNNAMED_RUN_SELFTEST_VARIABLE, CONFIRM_RULE_SELFTEST_VARIABLE)
KNOWN_BREAK_ITEMS = (
    "run-hard-limit-skipped", "run-hard-limit-full-gate-judged", "run-hard-limit-confirmed-spared", "run-hard-limit-writers-spared", "run-hard-limit-children-spared", "run-hard-limit-parent-summed", "run-hard-limit-outcome-unmarked", "run-hard-limit-writer-unreminded", "run-hard-limit-exclusive", "run-hard-limit-loose", "scope-inside-script-unnamed", "crash-scope-unreported",
    "fallback-run-when-unreachable", "exit-code-dropped", "wait-gives-up-on-disconnect", "environment-unfiltered", "queue-variables-carried",
    "retired-variables-carried", "retired-variable-assignments-accepted", "nested-dispatch-allowed", "session-from-client", "cancel-session-unchecked", "cancel-no-readback", "signature-unchecked", "tamper-unalerted",
    "operations-unlogged", "unlisted-exit-swallowed", "peer-exit-unsplit", "lost-not-recorded",
    "cap-growth-skipped", "cap-unclamped", "host-switch-skipped", "retry-overwrites-attempts", "squeeze-retry-skipped",     "server-pending-ignored", "launch-when-unplaceable", "queue-reversed", "nofile-limit-dropped", "waiting-reason-unreported", "backfill-disabled",     "no-detach", "start-failure-ignored", "start-twice-after-restart",
    "resources-unrecorded", "record-ignored-for-cap", "unclaimed-not-listed", "check-deps-skipped", "config-missing-ignored",
    "state-placement-unchecked", "dry-run-writes", "secrets-overwritten", "install-readback-skipped", "stop-exit-status-only",
    "target-directory-unset", "clean-session-unchecked", "runner-wake-skipped", "wait-notify-skipped", "aged-task-not-prioritized", "starved-head-unreported", "blocked-head-counted-as-starved", "in-repository-target-accepted",
    "unmeasured-recorded-as-zero", "threads-unset", "maximum-line-vague", "sharding-unrecognized",
    "shard-retry-restarts-all", "merge-before-all-green", "outputs-temporary-name-shared", "shard-failure-unnamed", "balance-ignored", "retry-pinned-to-host", "stall-ignored", "stall-grace-skipped", "cpu-unread-counted-as-idle", "outer-scope-cpu-only", "inner-commands-unchecked", "queued-not-revalidated", "cpu-ignored-for-stall", "peer-run-retry-skipped", "peer-starts-unthrottled", "squeeze-at-total-not-cap-hit", "host-specific-variables-sent-to-peer", "running-progress-unreported", "wrapper-prefixes-allowed", "resource-arguments-allowed", "peer-overflow-blocked",
    "shard-not-sent-to-peer", "shard-cap-not-fitted", "shard-key-position-folded", "socket-path-unchecked", "unrecorded-default-grown", "unrecorded-booked-above-cap",
    "unfinished-peaks-counted", "cap-hit-not-counted", "cap-hit-low-peak-counted-at-cap",
    "completed-peak-ignored", "log-tail-unbounded", "log-all-ignored", "log-attempt-ignored", "log-missing-exit-zero", "queued-cancel-left-queued",
    "finished-cancel-overwrites", "sharded-cancel-children-left", "plan-failure-proceeds", "merge-failure-succeeds", "unsharded-arguments-sharded",
    "start-readback-skipped", "start-exit-status-ignored", "gpu-switch-taken-as-on", "gpu-service-skipped", "pinned-peer-unavailable-waits",
    "prediction-recomputed-every-pass", "pending-read-per-task", "pass-booking-not-carried", "launch-recheck-skipped",
        "stall-clock-reset-on-restart", "workspace-writers-retried", "judge-error-blocks-queue", "stalled-at-cap-not-retried", "booked-up-to-slice-total",
    "memory-cap-not-passed", "source-repository-not-passed", "suspect-unreported", "suspect-reported-every-wait", "pinned-cap-ignored", "pinned-cap-grown",
    "pinned-cap-literal", "pin-unreadable-predicted", "booked-at-peak", "health-judges-head",     "pinned-cap-at-slice-total", "margin-fallback-fixed", "margin-unmeasured-waits",
    "margin-local-for-both", "pinned-nonpositive-accepted", "ended-attempts-unsettled", "feed-only-counted-as-writer", "registry-service-column-ignored", "cpu-gate-ignored", "unstarted-slice-estimated", "down-host-judged-never",     "environment-prefix-kept", "nfs-unchecked", "nfs-probe-one-way", "nfs-probe-at-root", "nfs-probe-shared-name", "quoted-word-paths-ignored", "expansion-tail-as-path",
    "peer-unavailable-silent", "reason-line-omitted", "clean-failure-unexplained",
    "shared-roots-not-passed", "peer-admission-unmarked", "disk-floor-unchecked", "peer-tree-copied", "workspace-writers-local-only",
    "cancel-race-unguarded", "sharded-parents-unreconciled", "attempt-claim-split", "sharded-cancel-finished-first",
    "association-column-unchecked", "start-timeout-as-failure", "build-disk-unpredicted", "peer-step-exit-ignored", "nfs-mount-placeholder-accepted",
    "peer-memory-peak-ignored", "prune-server-exit-unsplit", "temporary-service-wrapper-kept", "temporary-service-key-shared",
    "host-lists-restored", "data-root-key-unchecked", "outside-path-accepted",
    "shared-roots-outside-accepted", "verdict-library-family-accepted", "temporary-service-heavy-accepted",
    "pinned-cap-this-host-only", "by-host-missing-accepted", "by-host-outside-accepted", "shard-host-role-passed",
    "leftover-processes-kept", "long-run-not-noted", "resumable-declaration-ignored",
    "attempt-label-not-passed", "peak-read-only-by-sampling", "shard-outputs-not-carried", "probe-children-left", "scene-removed-at-finish", "scene-kept-after-collect",
    "scene-keep-ignored", "scene-left-by-clean", "leftovers-outside-own-unit",
    "shard-siblings-abandoned", "shard-cancel-ends-whole", "shard-missing-unnamed", "parent-cancel-spares-running", "cancel-outcome-unchecked",
    "failed-start-counted", "child-rows-unverified", "task-locks-kept",     "recorded-cap-floored", "cap-floor-skipped", 
    "attention-queue-unreported", "attention-finished-unreported", "attention-cleanup-unreported",
    "attention-shard-advance-unreported", "attention-reported-every-wait", "temporary-service-build-unrecognized",
    "pause-queue-ignored", "pause-start-ungated", "pause-halts-reconcile", "pause-state-memory-only", "resume-leaves-cache-paused", "pause-health-unreported",
    "pause-waiting-unreported", "pause-judges-starved",
    "pause-allow-ignored", "pause-allow-matches-any-session", "pause-allow-waiting-paused", "pause-allow-change-ignored", "resume-keeps-allowed-session",
    "pause-allow-memory-only", "pause-allow-unsigned", "pause-allow-unreported", "pause-other-running-miscounted",
    "wait-limit-fixed", "temporary-service-own-server-unrecognized",
    "shard-key-from-joined-command",
    "workspace-write-interrupted-unreported", "apply-outcome-ignored",     "read-only-arguments-counted-as-writer", "peer-retry-build-directory-fresh",
    "parallelism-falls-back-to-cores", "build-directory-named-by-label",
    "at-cap-busy-cpu-kept-running", "at-cap-fraction-ignored", "peer-memory-line-unread", 
    "attention-quiet-busy-unreported", "io-wait-ignored", "attention-io-wait-unreported",         "health-head-judged-apart", "progress-unwatched", "progress-gates-ignored", 
    "whole-workspace-cargo-accepted", "copy-proof-build-accepted", "copy-proof-start-accepted", "copy-proof-every-copy-judged", "copy-proof-every-command-judged",
    "repository-target-bypass-unwarned", "cancelled-artifacts-kept",
    "inbox-not-sent", "inbox-auth-omitted", "inbox-reported-every-round", "pushed-outcome-not-collected",
    "health-unwatched", "health-alerted-every-round", "gpu-card-failure-unwatched", "unclaimed-unwarned",
    "gpu-block-not-judgeable-unwatched", "gpu-cards-unread-taken-as-healed", "gpu-card-number-defaulted", "gpu-health-forgotten-before-the-first-read",
    "gpu-alerted-memory-only", "gpu-health-unread-host-taken-as-healed", "gpu-block-not-judgeable-claim-ignored",
    "gpu-block-not-judgeable-keyed-per-block", "gpu-block-not-judgeable-forgotten-between-reads",
    "cancel-not-started-receipt-unread", "quiet-log-busy-any-exemption", "inbox-pending-dropped-on-error",
    "leftovers-error-uncaught", "inbox-attention-per-shard",
    "memory-peak-with-page-cache", "attention-reported-memory-only", "attention-queue-reasonless-sent",
    "booking-over-host-limit", "idle-short-unjudged", "idle-short-silent", "idle-short-blocks-later", "oversized-under-idle-short-reported", "oversized-reported-before-queue-judged", "record-over-every-host-never-fits", "drop-history-cap-hits-ignored", "oversized-budget-silent", "long-run-idle-silent",
    "fills-any-cap-silent", "old-metric-peaks-read",
    "attention-next-generic", "progress-not-pushed", "unclaimed-outcome-not-repushed", "anomaly-not-repushed", "anomaly-acknowledgement-ignored", "anomaly-acknowledge-open-only", "runaway-reports-memory-only", "runaway-reused-pid-carried", "advance-episodes-memory-only", "restart-surviving-signature-unchecked", "folded-key-unnamed", "inbox-socket-from-submit-only", "inbox-missing-accepted",
    "confirm-skipped", "confirm-counts-unchecked", "unconfirmed-kept", "cleanup-tests-tables-hand-listed",
    "clear-queue-active-state-only", "clear-queue-running-attempts-ignored",
    "fills-any-cap-counts-cap-hits", "path-check-substring",
    "peer-health-single-timeout-reported", "cap-hit-stat-unreported",
    "peak-memory-stat-read-at-exit", "exit-peak-only-when-unsampled", "loop-leak-equal-peaks-growing", "loop-leak-reading-mismatch-silent",
    "pinned-squeeze-as-cap-hit", "pinned-over-limit-unreported", "pin-cap-help-pinned", "scene-sessionless-kept",     "long-run-unconfirmed-silent", "long-run-cap-hits-counted", "full-gate-flag-ignored", "full-gate-any-command-accepted",
    "bounce-new-id-reminder-omitted", "long-run-short-why-accepted", "long-run-confirm-unchecked", "long-run-sessionless-unnoted", "long-run-confirmed-still-reminded", "long-run-confirmed-flag-hidden",
    "whole-package-unconfirmed-accepted", "whole-package-confirmed-exempts-two-hours",
    "attempt-row-overwritten", "parent-finalized-before-children", "recheck-disk-unchecked",     "recheck-nfs-unchecked", "stall-stop-by-task", "peer-cleanup-recovered-concurrently",
    "tampered-rows-left-unrepaired", "tampered-unit-left-running", "manual-from-session-accepted", "sessionless-unmarked-accepted", "running-attempts-unverified",
    "confirm-counts-outside-session-lock", "resident-entries-kept",
    "whole-package-cells-all-three-only", "merge-all-packages-accepted", "unscoped-gate-accepted", "queued-refusals-not-rechecked",
    "loop-leak-check-omitted", "repeated-cap-hits-silent",     "long-run-idle-quarter-threshold",
    "retired-outcome-kinds-unknown",
    "resume-declaration-any-word",
    "runaway-process-unreported", "runaway-slice-counted", "runaway-process-not-stopped", "runaway-unowned-not-stopped", "runaway-protected-stopped", "runaway-sent-to-every-session", "hard-limit-stop-once", 
                "nested-units-unbound",
    "running-booked-at-cap", "running-booked-with-margin", "slice-floor-ignored", "shard-whole-by-wall-clock", "shard-key-count-kept",
    "stopped-peaks-not-counted", "compiles-declaration-ignored", "parent-cancel-unchecked-at-claim", "cancel-origin-unread", "cpu-threads-settling-only",
    "cpu-unread-passed", "lookup-indexes-skipped", "running-attempts-reread",
    "budget-source-every-pass", "peer-probe-in-scheduler", "repass-kept-when-idle",     "finish-in-scheduler", "cpu-threads-registered-only",
    "unit-unreadable-exit-file-unread", "unit-unreadable-absent-cgroup-unconfirmed", "unit-unreadable-lost-unverified", "unit-unreadable-unreminded",
    "unit-unreadable-health-unreported",         "cargo-test-key-whole-command", "selection-key-whole-command", "entry-script-absolute-path-kept", "commit-gate-without-full-gate-accepted", "commit-gate-query-forms-judged", "progress-ignored-for-stall", 
    "cpu-wanted-by-registered-threads", "cpu-unrecorded-by-one-core", "cpu-restart-unmeasured-by-record", "cpu-latest-runnable-ignored", "signal-exit-unnamed", "cpu-running-unmeasured-registered",
    "heartbeat-counted-as-log", "record-shard-count-unchecked",
        "fills-any-cap-counts-record-caps", "unit-stopped-still-booked", "cpu-load-io-unsplit", "peer-recheck-once-only",
    "check-nfs-probed-live-by-default", "check-probe-unbounded", "booking-audit-skipped", "booking-audit-young-counted", "plan-key-compile-unmarked", "merge-and-verify-counted-as-writer",
        "promote-session-unchecked", "promote-cpu-gate-applied", "promote-queue-order-ignored",     "progress-line-progress-unnamed", "open-file-write-mode-unchecked", "open-file-growth-ignored",
    "quiet-log-busy-progress-ignored", "ssh-unmultiplexed",
        "store-transaction-left-open", "disk-floor-host-judged-never",     "stall-cpu-read-before-log",
    "peer-readings-overwritten", "log-content-unlocked", "promote-unwoken", "promote-shards-refused", "promote-aged-blocks-queue", "promote-blocked-counts-as-earlier",
    "pause-queue-positions-filtered",     "peer-probe-thread-unjoined", "peer-cooldown-ignored-off-scheduler", "command-wrapper-values-unskipped", "cargo-exclude-value-unskipped", "runner-start-error-uncaught", "runner-final-sample-skipped", "log-noise-marks-incomplete",
    "runaway-targets-counting-children",
    "aged-blocked-lets-others-start", "jump-length-unchecked", "jump-refused-any-resource", "running-young-booked-by-usage", "machine-memory-line-ignored", "external-kill-unrecognized", "bookings-ledger-unreported",
    "threads-record-succeeded-only", "confirm-retries-fixed",     "cpu-running-cores-over-registered",
    "suspect-reported-memory-only", "health-notified-memory-only", "booking-drift-notified-memory-only",     "long-run-split-silent", "long-run-split-cap-hits-counted", "long-run-split-exempt-ignored", "workspace-write-blocked-unreported", "previous-swap-running-unreported", "swap-scratch-busy-unreported", "swap-busy-cause-unread", "swap-busy-unknown-guessed", "attention-why-unclipped",
    "estimate-longest-succeeded", "estimate-only-succeeded", "estimate-latest-any-outcome", "estimate-short-failure-counted", "estimate-failure-outside-window", "estimate-by-exact-command",
    "cpu-small-slots-withheld", "cpu-small-slots-unlimited", "cpu-small-slot-heavy-still-counted",
    "io-pressure-ignored", "io-pressure-applies-to-all", "io-pressure-limit-defaulted", "io-pressure-peer-reads-local", "io-pressure-wait-unnamed", "io-pressure-limit-unchecked", "gpu-cases-not-run-unread", "gpu-not-run-line-start-only", "gpu-not-run-word-edges-unchecked", "gpu-not-run-color-codes-kept", "gpu-not-run-first-mark-only",
    "inbox-headline-state-omitted", "inbox-headline-round-omitted", "inbox-headline-name-omitted", "inbox-flag-lines-before-evidence",
    "peer-external-kill-line-ignored", "shell-inner-scope-unread", "workspace-writer-cap-unpadded", "key-stats-alternation-ignored",
    "key-stats-metric-unfiltered", "key-stats-retries-unfolded", "key-stats-since-ignored", "key-stats-dump-overwrites",
    "cpu-cores-by-average", "cpu-oversubscription-three",
    "peak-segment-reading-unchecked",     "submit-trace-silent",
    "memory-wait-text-unnamed", "memory-cgroup-kill-counted",
    "cpu-runnable-threads-unread", "threads-over-registered-silent",
    "threads-over-registered-idle-counted", "threads-over-registered-many-processes-counted", "shard-binaries-kept", "shard-binaries-path-unchecked", "runner-threads-state-unread",
    "unnamed-run-refusal-skipped", "unnamed-run-exemptions-ignored", "unnamed-entry-item-unchecked", "unnamed-entry-item-uncounted", "unnamed-cargo-filter-accepted",
    "unnamed-cargo-exact-uncounted", "unnamed-prove-red-unchecked", "unnamed-prove-red-name-offset", "unnamed-stage-whole-accepted", "unnamed-check-cells-accepted",
    "unnamed-crash-gate-item-whole-accepted", "unnamed-crash-gate-cases-only-uncounted", "test-starting-cells-misnamed",
    "whole-package-feature-items-uncounted", "whole-package-any-item-exempts", "card-declarations-accepted", "card-variables-passed-to-attempt",
    "build-slot-per-attempt", "build-slot-tree-ignored", "build-slot-held-shared", "vanished-tree-pool-kept", "clean-removes-build-slot", "build-disk-reuse-uncredited",
    "unpooled-label-directories-kept", "unpooled-label-directory-open-unchecked", "unpooled-label-directories-pool-unread",
    "selection-positionals-in-place", "plan-key-shrunk-domain-kept", "stall-window-head-unread", "verdict-library-progress-uncounted", "long-run-why-children-reminded",
    "long-task-record-unread", "long-task-latest-only", "long-task-whole-unjudged", "long-task-shard-unjudged", "long-task-unrecorded-unjudged", "short-task-held",
    "confirm-direct-refused", "direct-reason-unreported", "long-task-reason-unsaid",
    "shell-string-writers-unread", "script-body-writers-unread", "unpooled-label-directories-running-unread",
    "wrapped-hook-swap-unread", "wrapped-previous-swap-unnamed", "wrapped-writer-words-unused",
    "wrapped-writers-first-only", "wrapped-writer-paths-literal", "wrapped-read-only-hook-swap-unread",
    "brace-group-not-layered", "heredoc-body-as-commands", "string-env-chdir-unread", "swap-outcome-unread", "swap-outcome-owner-ignored")
