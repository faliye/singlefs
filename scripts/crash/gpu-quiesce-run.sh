#!/usr/bin/env bash
# admission: always 每次调都在这一刻的卡与服务上清场、跑，上一次的结局不替这一次作保
# run-condition: command bash python3 flock timeout
# 凡是起 GPU 的跑法都经这一份：照多机配置清场并回读、跑那条命令、交回那条命令的退出码。跑完、跑红或被打断都不复原：
#   清场停下的服务一直停着，要复原由用户手动跑私有复原脚本（多机配置的 QUIESCE_START_COMMAND，再拿 QUIESCE_STARTED_CHECK_COMMAND 回读到它退 0）。
# 多机配置里没写清场命令的不在文件头的 run-condition 里判（--selftest 不读多机配置，要在没有配置的机器上也跑得了），
#   在运行中照 .claude/singlefs-ai-sop/rules/preflight-discipline.md 的退出码退 78、写出路。
#
#   gpu-quiesce-run.sh -- <命令…>     清场 → 跑 <命令…>（不复原）；<命令…> 原样起（不经 sh -c），标准输入输出与环境原样交给它
#   gpu-quiesce-run.sh --selftest     拿假的清场命令（换掉多机配置）与假的命令走一遍各格，末尾一轮逐个打开弄坏开关、核它红在点名的格
#
# 只认服务直接起的那一层：task-server 给要卡的任务起尝试时，在最外层套这一份，并给它设 GPU_QUIESCE_RUN_ADMITTED_BY_TASK_SERVER=<任务号>.<第几次尝试>
#   （task-server/task_server_lib.py 的 QUIESCE_ADMITTED_VARIABLE；非空就算设了，值不打、不记）。-- <命令…> 这一种在参数之后先读它、马上 unset，
#   命令与清场那两条命令都看不到它；没有就拒、退 2，清场命令一条没起、命令没跑。于是任务里面的脚本再套一层清场当场被拒：卡与清场由服务定。
#   --selftest 不看它。弄坏开关 admission-mark-kept 下不 unset（标记随环境传给命令，里面再套一层照跑）。
#
# 清场：读多机配置（${SINGLEFS_MULTI_HOST_CONFIG:-<主工作树的根>/multi-host.env}，模板仓根 multi-host.env.example；主工作树的根按 git common-dir 取，
#   同 scripts/crash/crash-point-replay-shard-configuration-check.sh）的两个键 QUIESCE_STOP_COMMAND、QUIESCE_STOPPED_CHECK_COMMAND，两个都要写；
#   QUIESCE_START_COMMAND、QUIESCE_STARTED_CHECK_COMMAND 留给手动复原，这一份不读、写没写都照跑。
#   两条命令各用 bash -c 在调用方的工作目录里跑，命令与输出都不打到标准错误上（里面有私有的服务名）。
#   输出逐条追加进这一趟的本机日志 <日志目录>/<日期>-<登记名>.log（目录 700、文件 600，每条前后各一行「== <键> 起 <时刻>」「== <键> 退 <码>」）：
#   清场回读没过、或这一趟跑的清场命令退非 0 时留下，路径写进 GPU_QUIESCE 那一行的 log=；都过了就删掉。
# 上一趟清了场没复原、服务已经停着：这一趟照样跑清场命令（私有清场脚本见已经停干净就只回读、退 0），回读过了就照跑。
# 两台对称（两台用同一份多机配置，带过去的写法在 scripts/hosts/peer-host-lib.sh 文件头「两台用同一份多机配置」）：清场四个键写的是「在起活的那一台跑、两台都管」的命令
#   （multi-host.env.example：例子是停掉占着两台的服务；清场回读两台都查，下面「回读看哪几张卡、跳过哪些进程」），哪一台起活都照同一份配置跑，不读角色变量。
#   起活那一方照配置清过场、回读过了，往另一台起命令时给它设 GPU_QUIESCE_ORIGIN_READ_BACK=1（变异分片在开跑之前 peer_host_quiesce 回读过了，或 multi-host-run.sh --peer 正经这一份起）：
#   这时这一份不再跑清场与回读那两条命令、照跑，清场那一行写 result=quiesced-on-origin；没设就在这一台照配置清场、回读。
#   锁、登记、本机日志与出去时那一行照旧（同一台上几趟同时用卡照样分得清）；配置里清场那两个键照样要写全（同一份配置，没写全两台一样退 78）。
#   弄坏开关 origin-read-back-ignored 下不看 GPU_QUIESCE_ORIGIN_READ_BACK，一律在这一台清场、回读。
#   日志目录 ${GPU_QUIESCE_LOG_DIRECTORY:-<工作根>/gpu-quiesce-logs}：开机不清，工作根只在 .claude/hooks/lib_work_root.py 里定义（SINGLEFS_WORK_ROOT），
#   这里经 python3 那份库取，取不到照「状态目录建不了」退 78；--selftest 把它指到自己的临时目录，不落进真工作根。
#   清场不停、不杀任何进程，也不按模式找进程：清场只停清场命令白名单里的服务单元；这一趟要用的卡上有不归白名单、也不在别的活着的登记之下的
#   计算进程（别的会话的 GPU 活）时清场回读本来就不过，照拒（退 78），出路是等那个进程跑完再起。命令起了之后，只有它自己的进程组会收到信号（下面「被打断」）。
# 回读看哪几张卡、跳过哪些进程：这一份在进来时算好两样，经环境只交给清场那两条命令（QUIESCE_STOP_COMMAND、QUIESCE_STOPPED_CHECK_COMMAND），
#   私有回读脚本认它们；调用方带进来的同名变量进来时先清掉，不认、也不传给 <命令…>：
#   GPU_QUIESCE_CHECK_LOCAL_CARDS=<本机卡的 nvidia-smi 序号，逗号隔开>  这一趟要用的本机卡，回读只看这几张上的计算进程。
#     取法照用例那一侧（crates/singlefs-verification/src/gpu_card_configuration.rs 的 cards_by_priority_from，清过场那一支）：环境变量 GPU_CARDS_BY_PRIORITY
#     设了照它（空串也算设了），没设读多机配置的 GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED；每项写 <HOSTS 里的地址>:<卡号>:<额度>（从右切两刀，地址可以是 IPv6），
#     取其中地址是这一台的那几项，另一台的不算。哪一项是这一台只照多机配置判法 crash-point-replay-shard-configuration-check.sh --emit-addresses 认
#     （共用库 crash-amplification-lib.sh 的 crash_amplification_host_addresses），写法照同一份库的 crash_amplification_card_list_problem 判。
#     环境变量 GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED 与多机配置的 GPU_CARDS_BY_PRIORITY 不读（清场之下的用例不照它们取卡）。
#     两处都没写时不设：回读照旧查本机所有卡。有一项写坏（写法不对、地址不在 HOSTS）或认不出这一台（多机配置判法判不过）时
#     命令不起、退 78（用例那一侧照样报错不跑）。每一项写法都对、点了卡而一项这一台的卡都没有时写 none：本机的卡一张都不看。
#   GPU_QUIESCE_CHECK_PEER_CARDS=<另一台卡的 nvidia-smi 序号，逗号隔开，或 none>  同一份显卡先后里地址是另一台的那几项，回读在另一台只看这几张；
#     每一项写法都对、点了卡而一项另一台的卡都没有时写 none（另一台的卡一张都不看）；两处都没写时不设：另一台照旧查所有卡。
#   GPU_QUIESCE_CHECK_EXEMPT_PIDS=<进程号，空格隔开>  别的活着的登记（holders/ 下）各自的进程号与它们这一刻的全部子孙：在锁里、清场命令起之前，
#     按 /proc/<进程号>/stat 的父进程号一层层往下找（不按名字或模式找进程），回读不把它们算「没清干净」。没有别的活着的登记时不设。
#     子孙的父进程先退了、被收养到别处的，不再算子孙，照外人办。
#   跳过的进程只管本机；另一台只按 GPU_QUIESCE_CHECK_PEER_CARDS 挑卡。私有回读脚本这几个变量都没给时与没有它们时一样查。标准错误上另打一句「回读看的卡：…；跳过 N 个进程」。
# 几趟同时用卡：每一趟进来时在状态目录（${GPU_QUIESCE_STATE_DIRECTORY:-${XDG_RUNTIME_DIR:-/tmp}/singlefs-gpu-quiesce}）的 holders/ 下登记一份
#   <进程号>-<进程的开机后起始时刻>，进出都在同一把 flock 锁里（至多等 GPU_QUIESCE_LOCK_SECONDS 秒，默认 900）。进来时没有别的活着的登记才跑清场命令，
#   有就只回读；出去时删掉自己的登记。登记的进程号不在了、或起始时刻对不上（进程号被重用）的登记是死的，进出时删掉；被 SIGKILL 的那一趟留下的登记由下一趟删。
# 被打断（欠账 拆分提案第一百二十三节）：命令起在自己的新会话里（setsid --wait，会话号与进程组号都是命令的进程号），标准输入输出与环境照旧交给它。
#   ① 收到 INT、TERM、HUP：把同一个信号转给命令的整个进程组，限时等它退出，到点进程组里还有成员就发 KILL，再做出去时的收尾（删登记、不复原）。
#      限时 GPU_QUIESCE_FORWARD_WAIT_DEFAULT_SECONDS（8 秒；调用方可用 GPU_QUIESCE_FORWARD_WAIT_SECONDS 改，取值限在 1 到 GPU_QUIESCE_FORWARD_WAIT_CEILING_SECONDS（10 秒，
#      即 .claude/singlefs-ai-sop/scripts/proc.py stop 的默认宽限）之间，写得不对取默认）。清场途中收到的，命令不起。都不复原。
#      命令退 0 而这一份收到过信号时仍退 128+信号号（见「退出码」）。
#   ② 这一份被 KILL（没有 trap 可跑）：起命令时同时起一个守卫（一个忽略 INT、HUP、TERM 的子 shell，只用内建睡，不起 sleep 子进程，输出丢掉），每 GPU_QUIESCE_GUARD_POLL_SECONDS 秒（默认 0.5）按
#      「进程号加开机后起始时刻」看这一份还在不在（僵尸算不在）；不在了就对命令的进程组发 TERM、等同一个限时、再发 KILL。命令退出之后这一份先停掉守卫再出去
#      （不停的话守卫之后会对一个已经空了的进程组发信号，进程号重用时就打到别人头上）。
#      为什么是守卫而不是 PR_SET_PDEATHSIG：后者只通知命令这一个进程、不管它的子孙，bash 里也设不了（要先起 python 再 exec，多一层）；
#      systemd scope 要用户级 systemd，而内存包装自己已经开 scope，再套一层是嵌套。进程组加守卫在 bash 里就落得了，覆盖命令的整个进程组。
#   够不到的：命令的子孙自己 setsid 出去的、或内存包装开的 systemd scope 里的（那一支由内存包装收到 TERM 时自己 systemctl --user stop，见它的文件头）；
#      起命令到守卫起来之间（几毫秒）这一份被 KILL，命令成孤儿。命令在新会话里没有控制终端，读不到终端：这一份起的命令都不读终端，标准输入是管道、文件或 /dev/null。
#   要停这一条链，对这一份发 TERM 就够（命令的进程组随之收到）；不用再从最底层的命令起逐个停。
# 查卡（需要检测卡状态，卡状态不对要弹窗提醒；records/2026-10-28-崩溃轻型提速.md「教训：卡坏了没人知道」）：进来之后、拿清场锁与跑清场命令之前，
#   先跑 scripts/hosts/gpu-card-health.py（环境原样带给它），只查这一次分到的卡：回读挑卡用的那一份显卡先后（下面「回读看哪几张卡」：环境变量 GPU_CARDS_BY_PRIORITY，
#   没设取多机配置的 GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED）点了卡的，原样交给它 --cards <那一份>；两处都没点卡的不带参数，它查多机配置点名的全部卡。
#   没分到的卡坏了不挡这一次（查全部卡时，一张没分到的坏卡让每一条要卡的任务清场都判 card-fault）。弄坏开关 card-health-every-named-card 下一律不带 --cards。
#   至多等 GPU_QUIESCE_CARD_HEALTH_SECONDS 秒（默认 GPU_QUIESCE_CARD_HEALTH_DEFAULT_SECONDS），它的输出不原样打，结论另打：
#   退 0（查的卡都好）、退 77（没带 --cards 而配置一张卡都没点名）照往下走，标准错误打一句「· 查卡：…」；
#   退 1 且汇总是 state=fault（有坏卡）：不清场、命令不起、退 78，清场那一行写 result=card-fault、坏卡清单与坏卡的依据，标准错误转打它逐卡的「✗ 显卡状态不对」与它的弹窗出路
#   （坏卡吐出的 [N/A] 计算进程不再落到回读没过那一句「等那个进程跑完」）；
#   别的结局（退 78、到点、起不来、退 1 而汇总不是 fault）：同样不清场、命令不起、退 78，清场那一行写 result=card-health-unchecked，报「卡状态查不成」与它的出路。
#   状态目录不是默认那一份（设了 GPU_QUIESCE_STATE_DIRECTORY：别的脚本的自证与假世界）不查、什么都不打，除非显式给了 GPU_QUIESCE_CARD_HEALTH_PROGRAM
#   （查卡程序的路径：可执行文件，直接起，参数与真的那一份相同（点了卡时 --cards <显卡先后>）；只给自证换成假的用）。查卡途中收到 INT/TERM/HUP：停掉它、不清场、命令不起，退 128+信号号。
# 卡与额度：不读显卡配置、不定额度、不按余量拒或排队：用哪几张卡、各给多少由调用方定（例：task-server 挑好卡、经 GPU_CARDS_BY_PRIORITY 交给它套着的命令），
#   这一份只管清场、回读、跑。调用方给的环境（GPU_CARDS_BY_PRIORITY 这类）原样传给命令。
# 给调用方与派发器的接口（选哪台、查余量、定卡与额度都在调用方；这一份不读也不改它们）：
#   参数只有 -- <命令…>；命令的环境原样带下去（调用方定的卡与额度经环境交给命令，例 GPU_CARDS_BY_PRIORITY），另加一个 GPU_QUIESCE_HOLDER=<这一趟的登记文件>：
#   命令要核自己在清场之下，看这份文件在、文件名以它父进程的进程号加「-」起头（被包着的命令的父进程就是这一份）。
#   交给清场那两条命令的另有 GPU_QUIESCE_CHECK_LOCAL_CARDS、GPU_QUIESCE_CHECK_PEER_CARDS、GPU_QUIESCE_CHECK_EXEMPT_PIDS（见「回读看哪几张卡、跳过哪些进程」），<命令…> 拿不到它们。
#   状态目录 GPU_QUIESCE_STATE_DIRECTORY、本机日志目录 GPU_QUIESCE_LOG_DIRECTORY 与锁的等待 GPU_QUIESCE_LOCK_SECONDS 可由调用方给；同一台机器上同时用卡的各趟要用同一个状态目录；
#   调用方的自证用假清场命令跑这一份时，把日志目录也指到自己的临时目录，不然失败那几趟的日志落进真工作根。
#   结局读退出码与下面的 GPU_QUIESCE 行：命令自己退 78 与这一份退 78 由 step=command 那一行分（not-started 是这一份拒的）。
# 输出：标准错误上逐步一行，给调用方与派发器读（字段 key=value，空格隔开）：
#   GPU_QUIESCE step=quiesce result=stopped-and-read-back|joined-and-read-back|read-back-failed|quiesced-on-origin holders=<进来时别的活着的登记数>[ stop-exit=<码>][ log=<本机日志>]
#     quiesced-on-origin 只在起活那一方清过场、给了 GPU_QUIESCE_ORIGIN_READ_BACK=1 时（文件头「两台对称」）
#     stop-exit 是这一趟跑的清场命令的退出码，回读没过或它退非 0 时打（回读过了而它退非 0，那一行是 stopped-and-read-back 带非 0 的 stop-exit，照跑）；
#     log 在留下本机日志时打
#   GPU_QUIESCE step=quiesce result=card-fault faulty-cards=<台>:<序号>[,<台>:<序号>…] cards=<台>/<序号>/<总线>/<依据>[;…]
#     查卡判出坏卡（文件头「查卡」）：台照查卡脚本机读行的 host=（HOSTS 里的地址），序号照它的 card=（总线读不出、归不到哪张卡的写 unknown），依据照它的 evidence=。
#     faulty-cards= 是给 task-server 读的坏卡清单（逗号隔开，地址可以是 IPv6、从右边切一刀；服务停用的就是这几张，task-server/task_server_lib.py 文件头「错误类型」card-fault 那一行）
#   GPU_QUIESCE step=quiesce result=card-health-unchecked exit=<查卡脚本的退出码>      查卡查不成（124 是到点）
#   GPU_QUIESCE step=command result=exit-<码>|not-started
#   GPU_QUIESCE step=restore result=not-restored holders=<出去时别的活着的登记数>
#     清了场、没复原（清场回读没过、命令没起的那一趟也打这一行）；标准错误另打一句「清了场、没复原」与手动复原的写法
#   GPU_QUIESCE step=exit code=<这一份的退出码>
#   出错时另打「✗ <哪一步>」与下一行「→ 怎么办：…」。
# 退出码：
#   <命令的>  命令起了：交回命令的退出码；命令退 0 而这一份收到过 INT/TERM/HUP 时退 128+信号号
#   2         用法错：没给 -- 或 -- 后面没有命令；环境里没有服务给的 GPU_QUIESCE_RUN_ADMITTED_BY_TASK_SERVER（文件头「只认服务直接起的那一层」）
#   78        命令没起：多机配置里 QUIESCE_STOP_COMMAND、QUIESCE_STOPPED_CHECK_COMMAND 没写全、状态目录或本机日志建不了（含取不到工作根）、锁等不到、清场回读没过、
#             显卡先后写坏或认不出这一台（文件头「回读看哪几张卡」）、查卡判出坏卡或查不成（文件头「查卡」）；
#             收到 INT/TERM/HUP 而命令还没起时退 128+信号号
# 弄坏开关 GPU_QUIESCE_BREAK=<项>（只在自证里认：命令行第一个参数不是 --selftest、又不是自证起的子进程时设了它，在参数解析之前退 2、点名它只在自证里认、清场命令一条不起；逗号连几个；--selftest 末尾那一轮从这张表现读，写法同 scripts/crash/crash-amplification.sh 文件头「弄坏开关」一段，
#   格名是 run_selftest 里「✗ 自证 <格名>：」的那一段）：
#   GPU_QUIESCE_BREAK=run-despite-failed-quiesce  清场回读没过也起命令
#       quiesce-read-back-fails（「清场回读没过却起了命令」）
#   GPU_QUIESCE_BREAK=restore-after-run  出去时（命令起没起、退几都算）又照多机配置跑 QUIESCE_START_COMMAND 与 QUIESCE_STARTED_CHECK_COMMAND 复原，restore 那一行写 restored
#       no-restore（「跑完调了复原命令」）
#   GPU_QUIESCE_BREAK=joined-stale-quiesce-trusted  跟着别的登记用卡而回读没过时不再清场、当场判回读不过
#       rejoin-stops-again（「后来的那一趟没再清一次场」）
#   GPU_QUIESCE_BREAK=trust-stale-holders  死了的登记也当活着的
#       stale-holder（「死了的登记挡住了清场」）
#   GPU_QUIESCE_BREAK=accept-missing-quiesce-commands  多机配置里 QUIESCE_STOP_COMMAND、QUIESCE_STOPPED_CHECK_COMMAND 没写全也不清场照起命令
#       missing-commands（「多机配置没写全清场命令却没退 78」）
#   GPU_QUIESCE_BREAK=no-failure-log  回读没过也不留本机日志、GPU_QUIESCE 行不带 log=
#       failure-log（「✗ 自证 failure-log：…没留本机日志」：回读没过没留本机日志）
#   GPU_QUIESCE_BREAK=failure-log-on-stderr  清场命令的输出打到标准错误上，不进本机日志
#       failure-log（「✗ 自证 failure-log：…私有输出打到了标准错误上」：清场命令的输出打到了标准错误上）
#   GPU_QUIESCE_BREAK=failure-log-in-runtime-dir  没设 GPU_QUIESCE_LOG_DIRECTORY 时日志落进 ${XDG_RUNTIME_DIR:-/tmp}（重启就清），不落工作根
#       failure-log（「✗ 自证 failure-log：…默认位置不在工作根下」：本机日志没落在工作根下）
#   GPU_QUIESCE_BREAK=drop-stop-exit  GPU_QUIESCE 清场那一行不带 stop-exit=（清场命令退非 0 而回读过了时看不出它退了几）
#       stop-exit（「✗ 自证 stop-exit：清场命令退…而回读过了，清场那一行没带 stop-exit」）
#   GPU_QUIESCE_BREAK=no-holder-descendants  不给回读 GPU_QUIESCE_CHECK_EXEMPT_PIDS：别的活着的登记底下的进程也当外人
#       holder-descendants（「前一趟登记下的进程在卡上，后一趟进来回读没过」）
#   GPU_QUIESCE_BREAK=exempt-holder-session  跳过的进程按会话号认：与别的活着的登记同一个会话的进程都跳过（不在任何登记之下的外人也跳过了）
#       outsider-not-exempt（「不在任何登记之下的进程在卡上，回读却过了」）
#   GPU_QUIESCE_BREAK=check-every-card  不给回读 GPU_QUIESCE_CHECK_LOCAL_CARDS：这一趟不用的卡也查
#       unused-card-outsider（「这一趟只用卡 1，卡 0 上的外人挡住了回读」）、used-card-outsider（「✗ 自证 used-card-outsider：这一趟用卡 0，卡 0 上的外人却没挡住回读」：回读拿到的卡不是只有卡 0）、
#       this-host-by-address（「这一台是 2001:db8::7…」：不挑卡，回读拿到的是 unset）、
#       peer-only-cards（「这一趟只用另一台的卡，本机卡 0 上的外人挡住了回读」）、local-only-cards（「这一趟只用本机卡 1，回读没拿到 GPU_QUIESCE_CHECK_PEER_CARDS=none」）
#   GPU_QUIESCE_BREAK=one-host-cards-mean-every-card  只点了一台的卡时，一张都没点的那一台照旧查所有卡（不给 none，也不给 GPU_QUIESCE_CHECK_PEER_CARDS）；两台都点了卡的不动
#       peer-only-cards（「这一趟只用另一台的卡，本机卡 0 上的外人挡住了回读」）、local-only-cards（「这一趟只用本机卡 1，回读没拿到 GPU_QUIESCE_CHECK_PEER_CARDS=none」）
#   GPU_QUIESCE_BREAK=cards-from-unquiesced-key  没设 GPU_CARDS_BY_PRIORITY 时读多机配置的 GPU_CARDS_BY_PRIORITY（不清场那一份），不读 GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED
#       used-card-outsider（「这一趟用卡 0，卡 0 上的外人却没挡住回读」）
#   GPU_QUIESCE_BREAK=keep-inherited-check-variables  调用方带进来的 GPU_QUIESCE_CHECK_LOCAL_CARDS、GPU_QUIESCE_CHECK_PEER_CARDS、GPU_QUIESCE_CHECK_EXEMPT_PIDS 不清掉，这一份没设的那一样照它们传给回读
#       used-card-outsider（「这一趟用卡 0，卡 0 上的外人却没挡住回读」）
#   GPU_QUIESCE_BREAK=origin-read-back-ignored  不看 GPU_QUIESCE_ORIGIN_READ_BACK：起活那一方清过场也在这一台再跑配置的清场与回读
#       origin-read-back-quiesce（「起活那一方清过场，这一台却跑了配置的清场命令…」）
#   GPU_QUIESCE_BREAK=this-host-from-hosts-order  不问多机配置判法 --emit-addresses，一律当 HOSTS 头一项是这一台（写死次序的样子）
#       this-host-by-address（「这一台是 2001:db8::7…」：回读拿到的是头一项 127.0.0.1 的卡 0）
#   GPU_QUIESCE_BREAK=card-address-outside-hosts-accepted  显卡先后里地址不在 HOSTS 的项照认，当成另一台的卡
#       card-address-outside-hosts-refused（「显卡先后的地址 192.0.2.99 不在 HOSTS 里…」）
#   GPU_QUIESCE_BREAK=admission-mark-kept  -- <命令…> 这一种读了 GPU_QUIESCE_RUN_ADMITTED_BY_TASK_SERVER 之后不 unset，标记随环境传给命令
#       nested-call-refused（「被服务起的命令里面再套一层清场…」）
#   这几个开关只在这一份的自证里红：崩溃放量入口、分项驱动与门禁 crash-point-replay 都不再经这一份起（清场归 task-server，服务把这一份套在要卡的命令外面）。
#   下面七个开关只有这一份的 --selftest 认（被打断时转发与守卫那几格、查卡那几格）：
#   GPU_QUIESCE_BREAK=no-forward-signal  收到 INT/TERM/HUP 只记一笔、不转给命令的进程组，也不起到点的 KILL（等命令自己结束）
#       interrupted（「✗ 自证 interrupted：被信号打断之后…」）、forward-term（「✗ 自证 forward-term：…」）、forward-interrupt（「✗ 自证 forward-interrupt：…」）、forward-limit（「✗ 自证 forward-limit：…」）
#   GPU_QUIESCE_BREAK=inherited-ignored-int  起命令不还原放后台时被置成忽略的 INT、QUIT（命令收不到转给它的 INT）
#       forward-interrupt（「✗ 自证 forward-interrupt：…」）
#   GPU_QUIESCE_BREAK=no-kill-after-limit  转发之后限时到了也不对命令的进程组发 KILL（忽略 TERM 的命令一直跑）
#       forward-limit（「✗ 自证 forward-limit：…」）
#   GPU_QUIESCE_BREAK=no-orphan-guard  起命令时不起守卫（这一份被 KILL 之后命令成孤儿接着跑）
#       kill-wrapper（「✗ 自证 kill-wrapper：…」）
#   GPU_QUIESCE_BREAK=leave-guardian-running  命令退出之后不停守卫
#       guardian-stopped（「✗ 自证 guardian-stopped：…」）
#   GPU_QUIESCE_BREAK=skip-card-health  清场之前不查卡（坏卡吐出的 [N/A] 计算进程挡住回读，出路落到「等那个进程跑完」）
#       card-fault（「✗ 自证 card-fault：…」）、card-ok（「✗ 自证 card-ok：…」）、card-health-unchecked（「✗ 自证 card-health-unchecked：…」）、
#       card-health-origin-read-back（「✗ 自证 card-health-origin-read-back：…」）、assigned-card-health（「✗ 自证 assigned-card-health：…」）
#   GPU_QUIESCE_BREAK=card-health-every-named-card  查卡不带 --cards，照查多机配置点名的全部卡（没分到的坏卡也判 card-fault）
#       assigned-card-health（「✗ 自证 assigned-card-health：分到好卡…」）
# 自证末尾那一轮走 scripts/crash/crash-amplification-lib.sh 的 crash_amplification_switch_wheel（并行度 CRASH_AMPLIFICATION_SELFTEST_JOBS，没给取 SINGLEFS_THREAD_CAP，再没给取登记的重活默认并行度）。
#
# gate-similar: peer-host-lib.sh 它的 peer_host_quiesce / peer_host_quiesce_left 只在双机配置开着时读得到清场命令，也不管几趟同时用卡；改它会废掉崩溃点重放双机分片三条用例的全绿标记（admission.py 的 SHARD_DRIVER_FILES），所以这里另写清场与登记
# gate-similar: crash-amplification-two-hosts.sh 它的 --quiesce 经 peer-host-lib.sh 清场，只给单跑双机驱动时用；分项驱动与崩溃放量入口经这一份
# gate-similar: crash-point-replay-shard-run.sh 它在崩溃点重放双机分片（CPU）开跑前经 peer-host-lib.sh 清场；不上 GPU，不归这一份
# gate-similar: run-with-memory-cap.sh 同是「包装 -- 命令」的写法，它管内存上限，这里管清场；两样各自套，次序由调用方定
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}

gpu_quiesce_script_directory="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# 共用库：读配置的一个键（crash_amplification_configuration_value）与弄坏开关轮（--selftest 末尾那一轮）
source "$gpu_quiesce_script_directory/crash-amplification-lib.sh"

GPU_QUIESCE_KEYS=(QUIESCE_STOP_COMMAND QUIESCE_STOPPED_CHECK_COMMAND)
# 复原那一对只在弄坏开关 restore-after-run 下读（文件头「清场」：复原由用户手动做）
GPU_QUIESCE_RESTORE_KEYS=(QUIESCE_START_COMMAND QUIESCE_STARTED_CHECK_COMMAND)
# 转发信号之后等命令退出的限时（文件头「被打断」）：默认值与上限；上限不长于 proc.py stop 的默认宽限（10 秒）
GPU_QUIESCE_FORWARD_WAIT_DEFAULT_SECONDS=8
GPU_QUIESCE_FORWARD_WAIT_CEILING_SECONDS=10
# 查卡至多等几秒（文件头「查卡」）：两台头一次整份读这次开机的内核日志各约 40 秒（本机量的），查卡脚本等第二台的上限 420 秒，再留余量；推的
GPU_QUIESCE_CARD_HEALTH_DEFAULT_SECONDS=600

gpu_quiesce_break_is() { [[ ",${GPU_QUIESCE_BREAK:-}," == *",$1,"* ]]; }
gpu_quiesce_selftest_mark_content="gpu-quiesce-run.sh --selftest 起的子进程"   # 自证起的子进程的标记内容（下面 case 之前那一段认它）
gpu_quiesce_line() { echo "GPU_QUIESCE $*" >&2; }

usage() {
  echo "  ✗ 用法：$0 -- <命令…>，或 $0 --selftest" >&2
  echo "  → 怎么办：要清场再跑的命令写在 -- 后面，例 $0 -- cargo test --release -p singlefs-verification --features gpu --test <测试目标>" >&2
  exit 2
}

gpu_quiesce_configuration_file() { # 打多机配置的路径：SINGLEFS_MULTI_HOST_CONFIG，没设取主工作树的根（git common-dir 的上一层）下的 multi-host.env
  if [[ -n "${SINGLEFS_MULTI_HOST_CONFIG:-}" ]]; then printf '%s' "$SINGLEFS_MULTI_HOST_CONFIG"; return 0; fi
  local common_directory
  if common_directory="$(git -C "$gpu_quiesce_script_directory" rev-parse --path-format=absolute --git-common-dir 2>/dev/null)" && [[ -n "$common_directory" ]]; then
    printf '%s/multi-host.env' "${common_directory%/*}"
  else
    printf '%s/multi-host.env' "$(cd "$gpu_quiesce_script_directory/../.." && pwd)"
  fi
}

gpu_quiesce_process_start_time() { # <进程号>：打 /proc/<进程号>/stat 的第 22 段（开机后起始时刻）；进程不在交回 1
  local stat_line rest fields=()
  stat_line="$(cat "/proc/$1/stat" 2>/dev/null)" || return 1
  rest="${stat_line##*) }"
  read -r -a fields <<<"$rest"
  printf '%s' "${fields[19]:-}"
}

gpu_quiesce_other_live_holders() { # 打 holders/ 下别的活着的登记数；死了的登记删掉（在锁里调）
  local holder_file holder_entry holder_pid holder_start live=0
  for holder_file in "$gpu_quiesce_state_directory/holders"/*; do
    [[ -e "$holder_file" ]] || continue
    holder_entry="${holder_file##*/}"
    [[ "$holder_entry" == "$gpu_quiesce_holder_name" ]] && continue
    holder_pid="${holder_entry%%-*}"; holder_start="${holder_entry#*-}"
    if [[ "$(gpu_quiesce_process_start_time "$holder_pid")" == "$holder_start" ]] || gpu_quiesce_break_is trust-stale-holders; then
      live=$((live + 1))
    else
      rm -f -- "$holder_file"
    fi
  done
  printf '%s' "$live"
}

gpu_quiesce_holder_descendants() { # 打别的活着的登记各自的进程号与它们这一刻的全部子孙（/proc 的父进程号链），空格隔开；不按名字找进程（在锁里调）
  local holder_file holder_entry holder_pid stat_file stat_line rest pid child root index fields=() roots=() found=()
  local -A children=() session_of=() seen=()
  for holder_file in "$gpu_quiesce_state_directory/holders"/*; do
    [[ -e "$holder_file" ]] || continue
    holder_entry="${holder_file##*/}"
    [[ "$holder_entry" == "$gpu_quiesce_holder_name" ]] && continue
    holder_pid="${holder_entry%%-*}"
    [[ "$holder_pid" =~ ^[0-9]+$ ]] || continue
    if [[ "$(gpu_quiesce_process_start_time "$holder_pid")" == "${holder_entry#*-}" ]] || gpu_quiesce_break_is trust-stale-holders; then roots+=("$holder_pid"); fi
  done
  (( ${#roots[@]} > 0 )) || return 0
  for stat_file in /proc/[0-9]*/stat; do
    { read -r stat_line <"$stat_file"; } 2>/dev/null || continue
    pid="${stat_file#/proc/}"; pid="${pid%/stat}"
    rest="${stat_line##*) }"
    read -r -a fields <<<"$rest"
    # 去掉「进程号 (名字) 」之后：0 状态、1 父进程号、2 进程组、3 会话号
    children[${fields[1]:-0}]+=" $pid"
    session_of[$pid]="${fields[3]:-}"
  done
  found=("${roots[@]}")
  for ((index = 0; index < ${#found[@]}; index++)); do
    pid="${found[index]}"
    [[ -n "${seen[$pid]:-}" ]] && continue
    seen[$pid]=1
    for child in ${children[$pid]:-}; do found+=("$child"); done
  done
  if gpu_quiesce_break_is exempt-holder-session; then
    for pid in "${!session_of[@]}"; do
      for root in "${roots[@]}"; do
        [[ -n "${session_of[$root]:-}" && "${session_of[$pid]}" == "${session_of[$root]}" ]] && seen[$pid]=1
      done
    done
  fi
  printf '%s\n' "${!seen[@]}" | LC_ALL=C sort -n | tr '\n' ' ' | sed 's/ $//'
}

# 回读看的卡（文件头「回读看哪几张卡、跳过哪些进程」）：置 gpu_quiesce_local_cards、gpu_quiesce_peer_cards（不设的为空）、gpu_quiesce_cards_said（给标准错误那一句）
#   与 gpu_quiesce_cards_value（取到的那一份显卡先后原样，查卡交给 --cards；没点卡为空）。
#   显卡先后写坏、或认不出这一台：置 gpu_quiesce_cards_problem、交回 1（命令不起、退 78）。哪一项是这一台照共用库 crash_amplification_host_addresses
#   （多机配置判法 --emit-addresses），写法照 crash_amplification_card_list_problem；弄坏开关 this-host-from-hosts-order、card-address-outside-hosts-accepted 原样交给这两处
gpu_quiesce_choose_cards() {
  local key=GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED value source first_is_this=0 outside_accepted=0
  gpu_quiesce_local_cards="" gpu_quiesce_peer_cards="" gpu_quiesce_cards_problem="" gpu_quiesce_cards_value=""
  if [[ -n "${GPU_CARDS_BY_PRIORITY+set}" ]]; then
    value="$GPU_CARDS_BY_PRIORITY"; source="环境变量 GPU_CARDS_BY_PRIORITY"
  else
    gpu_quiesce_break_is cards-from-unquiesced-key && key=GPU_CARDS_BY_PRIORITY
    value="$(crash_amplification_configuration_value "$gpu_quiesce_configuration" "$key")"; source="多机配置的 $key"
  fi
  # 查卡也只查这一份点名的卡（文件头「查卡」）：写坏的照下面判，判不过命令不起、轮不到查卡
  gpu_quiesce_cards_value="$value"
  if [[ -z "${value//[[:space:]]/}" ]]; then
    gpu_quiesce_cards_said="本机所有卡（环境变量 GPU_CARDS_BY_PRIORITY 与多机配置的 GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED 都没点名卡）"
    return 0
  fi
  gpu_quiesce_break_is this-host-from-hosts-order && first_is_this=1
  if ! crash_amplification_host_addresses "$gpu_quiesce_configuration" "$first_is_this"; then
    gpu_quiesce_cards_problem="$source 点名了卡，却认不出这一台是多机配置 HOSTS 里哪一项（上面是多机配置判法的原话）"
    return 1
  fi
  gpu_quiesce_break_is card-address-outside-hosts-accepted && outside_accepted=1
  if ! gpu_quiesce_cards_problem="$(crash_amplification_card_list_problem "$value" "$crash_amplification_hosts" "$outside_accepted")"; then
    gpu_quiesce_cards_problem="$source 的$gpu_quiesce_cards_problem"
    return 1
  fi
  gpu_quiesce_cards_problem=""
  # 弄坏开关 check-every-card：写法照判，只是不给回读挑卡
  if gpu_quiesce_break_is check-every-card; then gpu_quiesce_cards_said="本机所有卡"; return 0; fi
  gpu_quiesce_local_cards="$(crash_amplification_cards_on_address "$value" "$crash_amplification_this_host_address")"
  if [[ "$gpu_quiesce_local_cards" != none ]]; then
    gpu_quiesce_cards_said="本机卡 $gpu_quiesce_local_cards（$source）"
  elif gpu_quiesce_break_is one-host-cards-mean-every-card; then
    gpu_quiesce_local_cards=""
    gpu_quiesce_cards_said="本机所有卡（$source 一张本机卡都没点名）"
  else
    # 每一项写法都对、只点了另一台的卡：这一台的卡一张都不看（文件头「回读看哪几张卡」）
    gpu_quiesce_cards_said="本机一张都不看（$source 只点名了另一台的卡）"
  fi
  # 另一台那一侧（文件头「回读看哪几张卡」GPU_QUIESCE_CHECK_PEER_CARDS）；弄坏开关 one-host-cards-mean-every-card 只动一张都没点的那一台：
  #   这一台没点卡在上面置空，另一台没点卡在这里置空（不给 GPU_QUIESCE_CHECK_PEER_CARDS，照旧查所有卡）；两台都点了卡的照常
  gpu_quiesce_peer_cards="$(crash_amplification_cards_on_address "$value" "$crash_amplification_peer_host_address")"
  if [[ "$gpu_quiesce_peer_cards" == none ]] && gpu_quiesce_break_is one-host-cards-mean-every-card; then
    gpu_quiesce_peer_cards=""
  else
    gpu_quiesce_cards_said+="；另一台$([[ "$gpu_quiesce_peer_cards" == none ]] && echo "一张都不看" || echo "卡 $gpu_quiesce_peer_cards")"
  fi
  return 0
}

gpu_quiesce_enter_lock() {
  exec {gpu_quiesce_lock_fd}>>"$gpu_quiesce_state_directory/lock" || return 1
  flock -w "${GPU_QUIESCE_LOCK_SECONDS:-900}" "$gpu_quiesce_lock_fd"
}
gpu_quiesce_leave_lock() {
  flock -u "$gpu_quiesce_lock_fd" 2>/dev/null
  exec {gpu_quiesce_lock_fd}>&-
}

gpu_quiesce_run_configured() { # <键>：bash -c 跑配置里那条命令，命令与输出不打到标准错误，输出追加进本机日志；交回它的退出码
  #   环境另加回读看的卡与跳过的进程号（gpu_quiesce_check_environment，进来时在锁里置；没置的那一样不设）
  local code
  if gpu_quiesce_break_is failure-log-on-stderr; then env ${gpu_quiesce_check_environment[@]+"${gpu_quiesce_check_environment[@]}"} bash -c "${gpu_quiesce_commands[$1]}" </dev/null >&2; return; fi
  printf '== %s 起 %s\n' "$1" "$(date -u +%FT%TZ)" >>"$gpu_quiesce_log_file"
  env ${gpu_quiesce_check_environment[@]+"${gpu_quiesce_check_environment[@]}"} bash -c "${gpu_quiesce_commands[$1]}" </dev/null >>"$gpu_quiesce_log_file" 2>&1; code=$?
  printf '== %s 退 %s\n' "$1" "$code" >>"$gpu_quiesce_log_file"
  return "$code"
}

gpu_quiesce_keep_log() { # 留下本机日志：gpu_quiesce_log_field 置成 GPU_QUIESCE 行末尾的「 log=<路径>」（开关 no-failure-log 打开时不留、置空）
  gpu_quiesce_log_field=""
  gpu_quiesce_break_is no-failure-log && return 0
  gpu_quiesce_log_kept=1
  gpu_quiesce_log_field=" log=$gpu_quiesce_log_file"
}
gpu_quiesce_finish_log() { # 出口：没留下的本机日志删掉
  [[ -n "${gpu_quiesce_log_file:-}" ]] || return 0
  (( gpu_quiesce_log_kept )) || rm -f -- "$gpu_quiesce_log_file"
}

# 出去：删自己的登记，不复原（文件头「清场」）：打一行 step=restore result=not-restored 与手动复原的写法。
#   弄坏开关 restore-after-run 下在出去时复原并回读（只给证红用）
gpu_quiesce_leave() {
  (( gpu_quiesce_registered )) || return 0
  gpu_quiesce_registered=0
  local others="unknown" restore_key
  if gpu_quiesce_enter_lock; then
    rm -f -- "$gpu_quiesce_state_directory/holders/$gpu_quiesce_holder_name"
    others="$(gpu_quiesce_other_live_holders)"
    gpu_quiesce_leave_lock
  else
    echo "  · ${GPU_QUIESCE_LOCK_SECONDS:-900} 秒没拿到清场锁 $gpu_quiesce_state_directory/lock，自己的登记没删：下一趟进来时认出它是死的、删掉" >&2
  fi
  if gpu_quiesce_break_is restore-after-run; then
    for restore_key in "${GPU_QUIESCE_RESTORE_KEYS[@]}"; do
      gpu_quiesce_commands[$restore_key]="$(crash_amplification_configuration_value "$gpu_quiesce_configuration" "$restore_key")"
      [[ -n "${gpu_quiesce_commands[$restore_key]}" ]] && gpu_quiesce_run_configured "$restore_key"
    done
    gpu_quiesce_line "step=restore result=restored holders=$others"
    return 0
  fi
  gpu_quiesce_line "step=restore result=not-restored holders=$others"
  echo "  · 清了场、没复原：清场停下的服务一直停着；要复原手动跑私有复原脚本（多机配置的 QUIESCE_START_COMMAND），再跑 QUIESCE_STARTED_CHECK_COMMAND 回读到它退 0" >&2
}

# 被打断时的转发、限时与守卫（文件头「被打断」）。命令起在自己的新会话里（gpu_quiesce_start_command 的 setsid --wait）：gpu_quiesce_command_pid 同时是它的会话号与进程组号
gpu_quiesce_forward_limit() { # 打限时秒数：GPU_QUIESCE_FORWARD_WAIT_SECONDS 是 1 到上限之间的整数就取它，否则取默认
  local seconds="${GPU_QUIESCE_FORWARD_WAIT_SECONDS:-}"
  if [[ "$seconds" =~ ^[0-9]{1,3}$ ]] && (( 10#$seconds >= 1 && 10#$seconds <= GPU_QUIESCE_FORWARD_WAIT_CEILING_SECONDS )); then
    printf '%s' "$((10#$seconds))"
  else
    printf '%s' "$GPU_QUIESCE_FORWARD_WAIT_DEFAULT_SECONDS"
  fi
}

gpu_quiesce_process_alive() { # <进程号> <开机后起始时刻>：进程在、起始时刻对得上、不是僵尸；只用内建（守卫每个间隔调一次）
  local line rest fields=()
  { read -r line <"/proc/$1/stat"; } 2>/dev/null || return 1
  rest="${line##*) }"
  read -r -a fields <<<"$rest"
  [[ "${fields[0]:-Z}" != Z && "${fields[19]:-}" == "$2" ]]
}

gpu_quiesce_pause() { # <秒>：只用内建睡一会儿（read -t 读一根没有人写的管道）。限时与守卫是子 shell：用 sleep 的话，停子 shell 时睡着的 sleep 留在外面
  read -r -t "$1" <> <(:)
}

# 起命令：setsid --wait 把它放进自己的新会话。这一份没开作业控制，起的后台任务不是进程组组长，setsid 不 fork，$! 就是命令的进程号，也是它的会话号与进程组号
#   （下面的 kill 都对 "-$gpu_quiesce_command_pid" 发，这个变量只在这里赋）。没开作业控制时放后台起的命令，标准输入默认换成 /dev/null、INT 与 QUIT 被置成忽略：
#   <&0 把标准输入原样交给命令，env --default-signal 把 INT、QUIT 还原成默认（命令照常认转发给它的 INT）。标准输出、标准错误与环境原样
gpu_quiesce_start_command() { # <命令…>：置 gpu_quiesce_command_pid。弄坏开关 inherited-ignored-int 下不还原 INT、QUIT（命令收不到转给它的 INT）
  if gpu_quiesce_break_is inherited-ignored-int; then
    GPU_QUIESCE_HOLDER="$gpu_quiesce_state_directory/holders/$gpu_quiesce_holder_name" setsid --wait "$@" <&0 &
  else
    GPU_QUIESCE_HOLDER="$gpu_quiesce_state_directory/holders/$gpu_quiesce_holder_name" setsid --wait env --default-signal=INT,QUIT -- "$@" <&0 &
  fi
  gpu_quiesce_command_pid=$!
}

gpu_quiesce_on_signal() { # <信号名> <退出码>：INT/TERM/HUP 的 trap；记下，命令起过了就转给它的进程组
  gpu_quiesce_interrupted="$2"
  gpu_quiesce_last_signal="$1"
  gpu_quiesce_forward_signal
}

gpu_quiesce_forward_signal() { # 把 gpu_quiesce_last_signal 转给命令的进程组（进程组不在就发给命令自己），起到点的 KILL；命令还没起就什么也不做
  [[ -n "$gpu_quiesce_command_pid" && -n "$gpu_quiesce_last_signal" ]] || return 0
  gpu_quiesce_break_is no-forward-signal && return 0
  gpu_quiesce_forwarded=1
  kill -s "$gpu_quiesce_last_signal" -- -"$gpu_quiesce_command_pid" 2>/dev/null || kill -s "$gpu_quiesce_last_signal" "$gpu_quiesce_command_pid" 2>/dev/null
  gpu_quiesce_arm_limit
}

gpu_quiesce_arm_limit() { # 起一个限时的 KILL：一个子 shell 睡满限时就对命令的进程组发 KILL（命令先退了，gpu_quiesce_disarm_limit 停掉它）；已经起过的不再起
  [[ -z "$gpu_quiesce_limit_pid" ]] || return 0
  gpu_quiesce_break_is no-kill-after-limit && return 0
  { trap '' HUP TERM; gpu_quiesce_pause "$gpu_quiesce_limit_seconds"; kill -s KILL -- -"$gpu_quiesce_command_pid" 2>/dev/null; } </dev/null >/dev/null 2>&1 &
  gpu_quiesce_limit_pid=$!
}

gpu_quiesce_disarm_limit() { # 命令退了：停掉没到点的 KILL。用 KILL（它忽略 TERM），再 wait 收尸、丢掉它的标准错误（不然 bash 打「Killed …」和整段命令正文）
  [[ -n "$gpu_quiesce_limit_pid" ]] || return 0
  kill -s KILL "$gpu_quiesce_limit_pid" 2>/dev/null
  wait "$gpu_quiesce_limit_pid" 2>/dev/null
  gpu_quiesce_limit_pid=""
}

gpu_quiesce_guard_start() { # 起守卫：这一份没了（被 KILL）就停命令的进程组；守卫是个子 shell，忽略 INT、HUP、TERM（终端的 ^C、挂断和发给整组的 TERM 都不让它先走），输出丢掉
  gpu_quiesce_guard_pid=""
  gpu_quiesce_break_is no-orphan-guard && return 0
  local wrapper_start="${gpu_quiesce_holder_name#*-}"
  (
    trap - EXIT
    trap '' INT HUP TERM
    exec </dev/null >/dev/null 2>&1
    while gpu_quiesce_process_alive "$$" "$wrapper_start"; do gpu_quiesce_pause "$gpu_quiesce_guard_poll"; done
    kill -s TERM -- -"$gpu_quiesce_command_pid" 2>/dev/null
    for ((guard_tick = 0; guard_tick < gpu_quiesce_limit_seconds * 10; guard_tick++)); do
      kill -0 -- -"$gpu_quiesce_command_pid" 2>/dev/null || exit 0
      gpu_quiesce_pause 0.1
    done
    kill -s KILL -- -"$gpu_quiesce_command_pid" 2>/dev/null
  ) &
  gpu_quiesce_guard_pid=$!
}

gpu_quiesce_guard_stop() { # 命令退了、这一份还在：停守卫（它之后会对一个空了的进程组发信号，进程号重用时打到别人头上）
  [[ -n "$gpu_quiesce_guard_pid" ]] || return 0
  gpu_quiesce_break_is leave-guardian-running && return 0
  kill -s KILL "$gpu_quiesce_guard_pid" 2>/dev/null   # KILL 不是 TERM：守卫忽略 TERM
  wait "$gpu_quiesce_guard_pid" 2>/dev/null
  gpu_quiesce_guard_pid=""
}

gpu_quiesce_wait_command() { # 等命令退出，交回它的退出码；wait 被 trap 打断（命令还在）就接着等
  local code again
  while :; do
    # 信号在命令起好、进程号记下之前到了的，trap 那一刻没处转，这里补转
    if [[ -n "$gpu_quiesce_interrupted" ]] && (( ! gpu_quiesce_forwarded )); then gpu_quiesce_forward_signal; fi
    wait "$gpu_quiesce_command_pid" 2>/dev/null; code=$?   # 2>/dev/null：命令被 KILL 时 bash 在这里打「Killed …」和整段命令正文，退出码不丢
    if (( code > 128 )) && kill -0 "$gpu_quiesce_command_pid" 2>/dev/null; then continue; fi
    if (( code > 128 )); then
      # 命令已经退了：被 trap 打断的那一次 wait 没取到它的退出码时，再取一次（取过了是 127）
      wait "$gpu_quiesce_command_pid" 2>/dev/null; again=$?
      (( again != 127 )) && code="$again"
    fi
    return "$code"
  done
}

# 查卡（文件头「查卡」）：好或没点名卡交回 0；有坏卡或查不成打清场那一行、✗ 与出路，交回 78；途中被打断停掉它、交回 128+信号号。弄坏开关 skip-card-health 不查
gpu_quiesce_card_health() {
  local output_file seconds code=0 health_pid mark host card bus state evidence rest cards="" faulty="" next_step checked_said="多机配置点名的卡"
  local health_arguments=()
  gpu_quiesce_break_is skip-card-health && return 0
  [[ -z "${GPU_QUIESCE_CARD_HEALTH_PROGRAM:-}" && -n "${GPU_QUIESCE_STATE_DIRECTORY:-}" ]] && return 0
  # 只查这一次分到的卡（文件头「查卡」）；两处都没点卡的不带参数、查多机配置点名的全部卡。弄坏开关 card-health-every-named-card 下一律不带
  if [[ -n "${gpu_quiesce_cards_value//[[:space:]]/}" ]] && ! gpu_quiesce_break_is card-health-every-named-card; then
    health_arguments=(--cards "$gpu_quiesce_cards_value")
    checked_said="这一次分到的卡"
  fi
  seconds="${GPU_QUIESCE_CARD_HEALTH_SECONDS:-$GPU_QUIESCE_CARD_HEALTH_DEFAULT_SECONDS}"
  [[ "$seconds" =~ ^[1-9][0-9]{0,5}$ ]] || seconds="$GPU_QUIESCE_CARD_HEALTH_DEFAULT_SECONDS"
  output_file="$gpu_quiesce_state_directory/card-health-$gpu_quiesce_holder_name.out"
  if [[ -n "${GPU_QUIESCE_CARD_HEALTH_PROGRAM:-}" ]]; then
    timeout "$seconds" "$GPU_QUIESCE_CARD_HEALTH_PROGRAM" ${health_arguments[@]+"${health_arguments[@]}"} </dev/null >"$output_file" 2>&1 &
  else
    timeout "$seconds" python3 "$gpu_quiesce_script_directory/../hosts/gpu-card-health.py" ${health_arguments[@]+"${health_arguments[@]}"} </dev/null >"$output_file" 2>&1 &
  fi
  health_pid=$!
  wait "$health_pid" 2>/dev/null; code=$?
  # wait 被 trap 打断（它还在）：收到过 INT/TERM/HUP 就停掉它（自己起的这一个，只发这一次），再接着等它退
  if (( code > 128 )) && kill -0 "$health_pid" 2>/dev/null && [[ -n "$gpu_quiesce_interrupted" ]]; then
    kill -TERM "$health_pid" 2>/dev/null
  fi
  while (( code > 128 )) && kill -0 "$health_pid" 2>/dev/null; do
    wait "$health_pid" 2>/dev/null; code=$?
  done
  if [[ -n "$gpu_quiesce_interrupted" ]]; then rm -f -- "$output_file"; return "$gpu_quiesce_interrupted"; fi
  if (( code == 0 )); then
    echo "  · 查卡：${checked_said}都好（scripts/hosts/gpu-card-health.py 退 0）" >&2
  elif (( code == 77 )); then
    echo "  · 查卡：多机配置一张卡都没点名，没查（scripts/hosts/gpu-card-health.py 退 77）" >&2
  elif (( code == 1 )) && grep -q '^GPU_CARD_HEALTH_SUMMARY state=fault ' "$output_file"; then
    while read -r mark host card bus state evidence rest; do
      [[ "$mark" == GPU_CARD_HEALTH && "$state" == state=fault ]] || continue
      faulty+="${faulty:+,}${host#host=}:${card#card=}"
      cards+="${cards:+;}${host#host=}/${card#card=}/${bus#bus=}/${evidence#evidence=}"
    done <"$output_file"
    # faulty-cards= 是 task-server 停用哪几张的依据（文件头「输出」）
    gpu_quiesce_line "step=quiesce result=card-fault faulty-cards=$faulty cards=$cards"
    grep '^  ✗ 显卡状态不对' "$output_file" >&2
    next_step="$(sed -n 's/^  → 怎么办：//p' "$output_file" | head -n 1)"
    echo "  ✗ 显卡状态不对：没清场，命令没起（退 78）" >&2
    echo "  → 怎么办：${next_step:-照上面逐卡那几行办；单跑 python3 scripts/hosts/gpu-card-health.py 看全文}" >&2
    code=78
  else
    gpu_quiesce_line "step=quiesce result=card-health-unchecked exit=$code"
    grep '^  ✗ 卡状态查不成' "$output_file" >&2
    next_step="$(sed -n 's/^  → 怎么办：//p' "$output_file" | head -n 1)"
    echo "  ✗ 卡状态查不成（scripts/hosts/gpu-card-health.py 退 $code$( ((code == 124)) && printf '，%s 秒没查完' "$seconds" )）：没清场，命令没起（退 78）" >&2
    echo "  → 怎么办：${next_step:-单跑 python3 scripts/hosts/gpu-card-health.py 看原因，修好再起}" >&2
    code=78
  fi
  rm -f -- "$output_file"
  return "$code"
}

run_selftest() {
  local work failures=0 cells=0 nested=0
  crash_amplification_switch_wheel_inner && nested=1
  work="$(mktemp -d "${TMPDIR:-/tmp}/gpu-quiesce-run-selftest.XXXXXX")"
  trap 'rm -rf -- "${work:?}"' EXIT
  # 自证起的子进程的标记：只有这里建、这里设（下面 case 之前那一段认它）；各格起的里层凭它认 GPU_QUIESCE_BREAK
  printf '%s\n' "$gpu_quiesce_selftest_mark_content" >"$work/mark"
  export GPU_QUIESCE_SELFTEST_MARK="$work/mark"
  # 各格起的这一份都当服务直接起的那一层（文件头「只认服务直接起的那一层」）；格 admission-refused 自己去掉
  export GPU_QUIESCE_RUN_ADMITTED_BY_TASK_SERVER=selftest-task.1
  # 本机日志一律落进这里的临时目录（run_wrapper 再按状态目录各给一份），不落进真工作根；格 failure-log 里查默认位置的那一趟另把工作根指到临时目录
  export GPU_QUIESCE_LOG_DIRECTORY="$work/logs"
  # 转发之后的限时与守卫的间隔取短（文件头「被打断」），各格里被停的假命令几秒内就见分晓；调用方带进来的这两个值不认
  export GPU_QUIESCE_FORWARD_WAIT_SECONDS=2 GPU_QUIESCE_GUARD_POLL_SECONDS=0.2
  # 起活那一方清没清场、显卡先后与换掉这一台网卡上的地址的变量（多机配置判法只在自证里认它）不从调用方带进来：要的格自己给
  unset GPU_QUIESCE_ORIGIN_READ_BACK GPU_CARDS_BY_PRIORITY CONFIGURATION_SELFTEST_LOCAL_ADDRESSES
  # 查卡程序与它的限时不从调用方带进来：查卡那几格自己给（没给时各格的状态目录都不是默认那一份，这一份不查卡）
  unset GPU_QUIESCE_CARD_HEALTH_PROGRAM GPU_QUIESCE_CARD_HEALTH_SECONDS
  # 假的清场命令：每一步记一笔进 FAKE_CALLS；FAKE_STOPPED_CHECK / FAKE_STARTED_CHECK 写成 fail 时那一步回读不过；FAKE_STOP_EXIT 是清场命令的退出码（默认 0）；
  #   设了 FAKE_GPU_PROCESSES 的，回读再过一道假的「卡上的计算进程」（gpu-check.sh）。
  #   必写键写全（点名了卡的那几格经多机配置判法 --emit-addresses 认这一台，整份要判得过）：主机数组写 127.0.0.1（每一台的网卡上都有）与文档用的地址段 192.0.2.1（不在任何一台上），
  #   两台判得一样
  #   按台的三个键两台各一项（多机配置判法文件头「按台的三个键」）
  printf '%s\n' "HOSTS=127.0.0.1 192.0.2.1" "CARGO_BIN_DIRECTORY_BY_HOST=127.0.0.1=/selftest/this-cargo 192.0.2.1=/selftest/peer-cargo" \
    "BUILD_ROOT_BY_HOST=127.0.0.1=/selftest/this-builds 192.0.2.1=/selftest/peer-builds" "SLICE_MEMORY_CAP_BY_HOST=127.0.0.1=2G 192.0.2.1=1G" \
    ENABLE_ACROSS_MACHINES=0 ENABLE_GPU=0 "NFS_SHARED_ROOTS=$work" \
    "QUIESCE_STOP_COMMAND='echo stop >> \"\$FAKE_CALLS\"; echo fake-private-stop-output; rm -f -- \"\${FAKE_SERVICES_UP:-/nonexistent}\"; exit \"\${FAKE_STOP_EXIT:-0}\"'" \
    "QUIESCE_STOPPED_CHECK_COMMAND='echo stopped-check >> \"\$FAKE_CALLS\"; echo fake-private-unit-stdout; echo fake-private-unit-stderr >&2; [[ ! -e \"\${FAKE_SERVICES_UP:-/nonexistent}\" ]] && [[ \"\${FAKE_STOPPED_CHECK:-ok}\" == ok ]] && { [[ -z \"\${FAKE_GPU_PROCESSES:-}\" ]] || bash \"$work/gpu-check.sh\"; }'" \
    "QUIESCE_START_COMMAND='echo start >> \"\$FAKE_CALLS\"'" \
    "QUIESCE_STARTED_CHECK_COMMAND='echo started-check >> \"\$FAKE_CALLS\"; [[ \"\${FAKE_STARTED_CHECK:-ok}\" == ok ]]'" >"$work/multi-host.env"
  grep -v '^QUIESCE_STOPPED_CHECK_COMMAND=' "$work/multi-host.env" >"$work/multi-host-missing.env"
  # 假的「卡上的计算进程」回读（各格 holder-descendants、outsider-not-exempt、unused-card-outsider、used-card-outsider）：FAKE_GPU_PROCESSES 每行「<本机卡序号> <进程号>」，
  #   照这一份给的 GPU_QUIESCE_CHECK_LOCAL_CARDS 只看点名的卡（none 一张都不看）、跳过 GPU_QUIESCE_CHECK_EXEMPT_PIDS 里的进程号，剩下一个就退 1；
  #   那一刻看到的两个变量记进 FAKE_CHECK_SEEN，GPU_QUIESCE_CHECK_PEER_CARDS 另记进 FAKE_CHECK_SEEN.peer
  cat >"$work/gpu-check.sh" <<'STUB'
#!/usr/bin/env bash
echo "cards=${GPU_QUIESCE_CHECK_LOCAL_CARDS-unset} exempt=${GPU_QUIESCE_CHECK_EXEMPT_PIDS-unset}" >> "${FAKE_CHECK_SEEN:-/dev/null}"
[[ -n "${FAKE_CHECK_SEEN:-}" ]] && echo "peer=${GPU_QUIESCE_CHECK_PEER_CARDS-unset}" >> "$FAKE_CHECK_SEEN.peer"
[[ "${GPU_QUIESCE_CHECK_LOCAL_CARDS:-}" == none ]] && exit 0
residue=0
while read -r card pid; do
  [[ -n "$card" ]] || continue
  [[ -n "${GPU_QUIESCE_CHECK_LOCAL_CARDS:-}" && ",$GPU_QUIESCE_CHECK_LOCAL_CARDS," != *",$card,"* ]] && continue
  [[ " ${GPU_QUIESCE_CHECK_EXEMPT_PIDS:-} " == *" $pid "* ]] && continue
  echo "fake-gpu-residue card=$card pid=$pid"; residue=1
done <"${FAKE_GPU_PROCESSES:?}"
exit "$residue"
STUB
  # 显卡先后写在多机配置里的一份（格 used-card-outsider）：清场之后那一份点名本机卡 0，不清场那一份点名本机卡 1
  { cat "$work/multi-host.env"; printf '%s\n' "GPU_CARDS_BY_PRIORITY='127.0.0.1:1:8G'" "GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED='192.0.2.1:1:16G 127.0.0.1:0:24G'"; } >"$work/multi-host-cards.env"
  # 主机数组里另一项是 IPv6 的一份（格 this-host-by-address）：自证把这一台网卡上的地址换成 2001:db8::7，这一台就是那一项
  #   按台的三个键跟着主机数组换成这两项
  { grep -vE '^(HOSTS|CARGO_BIN_DIRECTORY_BY_HOST|BUILD_ROOT_BY_HOST|SLICE_MEMORY_CAP_BY_HOST)=' "$work/multi-host.env"
    printf '%s\n' "HOSTS=127.0.0.1 2001:db8::7" "CARGO_BIN_DIRECTORY_BY_HOST=127.0.0.1=/selftest/this-cargo 2001:db8::7=/selftest/peer-cargo" \
      "BUILD_ROOT_BY_HOST=127.0.0.1=/selftest/this-builds 2001:db8::7=/selftest/peer-builds" "SLICE_MEMORY_CAP_BY_HOST=127.0.0.1=2G 2001:db8::7=1G"; } >"$work/multi-host-ipv6.env"
  # 复原那一对没写的多机配置（格 no-restore）：这一份不读它们，照跑
  grep -vE '^QUIESCE_START(ED_CHECK)?_COMMAND=' "$work/multi-host.env" >"$work/multi-host-without-restore.env"
  # 假的命令：<模式> 记开始与结束；fail 退 7，self-term 自己收 TERM 退出，wait-for:<文件> 等那份文件出现（至多 10 秒），sleep:<秒> 睡一会儿，
  #   term-aware:<秒> / ignore-term:<秒> 起一个孙辈 sleep、进程号记进 $FAKE_PIDS.command 与 .child、起好了记 command-ready，再等：前者收到 TERM 记 command-got-term 就退，后者不理 TERM，
  #   hold <进程号文件> <放行文件> 起一个子进程（这一份的孙辈）、把它的进程号写进 <进程号文件>，等它见到 <放行文件>（至多 300 秒：放行由格子做，
  #     这个时限只防格子半路出错时不收；机器忙时 30 秒不够，后一趟还没进来它就到点退了、登记跟着没了）退出
  cat >"$work/command.sh" <<'STUB'
#!/usr/bin/env bash
echo "command-$1-start" >> "${FAKE_CALLS:?}"
# 核 GPU_QUIESCE_HOLDER：文件在、文件名以父进程（包装）的进程号加「-」起头
if [[ "$1" == ok && -e "${GPU_QUIESCE_HOLDER:-}" && "${GPU_QUIESCE_HOLDER##*/}" == "$PPID-"* ]]; then echo holder-of-parent >> "$FAKE_CALLS"; fi
case "$1" in
  fail) echo "command-$1-end" >> "$FAKE_CALLS"; exit 7 ;;
  self-term) kill -s TERM "$$" ;;
  wait-for:*) for ((round = 0; round < 200; round++)); do [[ -e "${1#wait-for:}" ]] && break; sleep 0.05; done ;;
  sleep:*) sleep "${1#sleep:}" ;;
  hold) bash -c 'for ((round = 0; round < 6000; round++)); do [[ -e "$1" ]] && break; sleep 0.05; done' hold-child "$3" &
        echo "$!" >"$2"; wait "$!" ;;
  term-aware:*|ignore-term:*|int-aware:*)
        # 收到 TERM 记一笔就退（term-aware），或不理 TERM（ignore-term，孙辈 sleep 也继承了忽略），或收到 INT 记一笔、停孙辈再退（int-aware：放后台起的孙辈忽略 INT，要自己停）；
        #   孙辈与自己的进程号记进 $FAKE_PIDS.child、.command，起好了记 command-ready
        case "$1" in
          term-aware:*) trap 'echo command-got-term >> "$FAKE_CALLS"; exit 0' TERM ;;
          int-aware:*) trap 'echo command-got-interrupt >> "$FAKE_CALLS"; kill "$grandchild"; exit 0' INT ;;
          *) trap '' TERM ;;
        esac
        sleep "${1#*:}" & grandchild=$!; echo "$grandchild" >"${FAKE_PIDS:?}.child"; echo "$$" >"$FAKE_PIDS.command"
        echo command-ready >> "$FAKE_CALLS"; wait "$grandchild" ;;
esac
echo "command-$1-end" >> "$FAKE_CALLS"
STUB
  selftest_cell_failed() { # <格名> <这一格判错的一句> <出路>
    echo "  ✗ 自证 $1：$2" >&2
    echo "     → 怎么办：$3" >&2
    failures=$((failures + 1))
  }
  run_wrapper() { # <calls 文件> <状态目录> [<环境 键=值>…] -- <命令…>：起一趟这一份，输出写进 <calls 文件>.out，本机日志进 <状态目录>/logs
    local calls="$1" state="$2"; shift 2
    local environment=()
    while (($#)) && [[ "$1" != -- ]]; do environment+=("$1"); shift; done
    # 调用方环境里的显卡先后与回读握手变量不带进来，要的格自己给
    env -u GPU_CARDS_BY_PRIORITY -u GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED -u GPU_QUIESCE_CHECK_LOCAL_CARDS -u GPU_QUIESCE_CHECK_PEER_CARDS -u GPU_QUIESCE_CHECK_EXEMPT_PIDS \
      -u CONFIGURATION_SELFTEST_LOCAL_ADDRESSES -u GPU_QUIESCE_ORIGIN_READ_BACK \
      SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host.env" GPU_QUIESCE_STATE_DIRECTORY="$state" GPU_QUIESCE_LOG_DIRECTORY="$state/logs" FAKE_CALLS="$calls" ${environment[@]+"${environment[@]}"} \
      bash "$0" "$@" >"$calls.out" 2>&1
  }
  # 调用次序（复原那两笔 start / started-check 不列：跑完调没调复原命令只由格 no-restore 判）
  calls_of() { { grep -vxE 'start|started-check' "$1" || true; } | tr '\n' ' '; }
  wait_for_line() { # <文件> <行> [<至多等几秒，默认 10>]：那一行出现（按挂钟算，不按轮数）
    local deadline=$((SECONDS + ${3:-10}))
    while (( SECONDS < deadline )); do grep -qx -- "$2" "$1" 2>/dev/null && return 0; sleep 0.05; done
    grep -qx -- "$2" "$1" 2>/dev/null
  }
  local cell_exit calls no_restore_output

  # 格 quiesce-run：清场、回读、命令，次序对，退命令的 0；输出里不带配置里的命令
  cells=$((cells + 1)); calls="$work/green.calls"; : >"$calls"
  run_wrapper "$calls" "$work/state-green" -- bash "$work/command.sh" ok; cell_exit=$?
  if [[ "$cell_exit" != 0 || "$(calls_of "$calls")" != "stop stopped-check command-ok-start holder-of-parent command-ok-end " ]] \
     || ! grep -q '^GPU_QUIESCE step=quiesce result=stopped-and-read-back holders=0$' "$calls.out" \
     || grep -q 'FAKE_CALLS' "$calls.out"; then
    selftest_cell_failed quiesce-run "没有照「清场、回读、命令」走、没退 0，或输出里带了配置里的命令（退 $cell_exit，次序「$(calls_of "$calls")」）" "看主流程 gpu_quiesce_enter / gpu_quiesce_leave 与 gpu_quiesce_run_configured（命令与输出都不打）"
  fi
  # 格 missing-commands：多机配置里少 QUIESCE_STOPPED_CHECK_COMMAND ⇒ 退 78，命令没起、清场命令一条都没跑
  cells=$((cells + 1)); calls="$work/missing.calls"; : >"$calls"
  run_wrapper "$calls" "$work/state-missing" SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-missing.env" -- bash "$work/command.sh" ok; cell_exit=$?
  if [[ "$cell_exit" != 78 || -s "$calls" ]] || ! grep -q 'QUIESCE_STOPPED_CHECK_COMMAND' "$calls.out"; then
    selftest_cell_failed missing-commands "多机配置没写全清场命令却没退 78、点名缺的键，或照跑了（退 $cell_exit，次序「$(calls_of "$calls")」）" "看读配置那一段：清场那两个键缺一个就退 78、不起命令"
  fi
  # 格 quiesce-read-back-fails：清场回读不过 ⇒ 命令不起、退 78；清场命令跑过
  cells=$((cells + 1)); calls="$work/stop-fails.calls"; : >"$calls"
  run_wrapper "$calls" "$work/state-stop-fails" FAKE_STOPPED_CHECK=fail -- bash "$work/command.sh" ok; cell_exit=$?
  if [[ "$cell_exit" != 78 || "$(calls_of "$calls")" != "stop stopped-check " ]] || ! grep -q '^GPU_QUIESCE step=command result=not-started$' "$calls.out"; then
    selftest_cell_failed quiesce-read-back-fails "清场回读没过却起了命令、没退 78（退 $cell_exit，次序「$(calls_of "$calls")」）" "看 gpu_quiesce_enter 回读那一支：不过就不登记命令、照样走 gpu_quiesce_leave"
  fi
  # 格 stop-exit：清场命令退 1 而清场回读过了 ⇒ 照跑、退命令的 0，
  #   清场那一行是 stopped-and-read-back 带 stop-exit=1；清场回读不过那一趟（格 quiesce-read-back-fails）带 stop-exit=0
  cells=$((cells + 1)); calls="$work/stop-refused.calls"; : >"$calls"
  run_wrapper "$calls" "$work/state-stop-refused" FAKE_STOP_EXIT=1 -- bash "$work/command.sh" ok; cell_exit=$?
  if [[ "$cell_exit" != 0 || "$(calls_of "$calls")" != "stop stopped-check command-ok-start holder-of-parent command-ok-end " ]] \
     || ! grep -Eq '^GPU_QUIESCE step=quiesce result=stopped-and-read-back holders=0 stop-exit=1( |$)' "$calls.out" \
     || ! grep -Eq '^GPU_QUIESCE step=quiesce result=read-back-failed holders=0 stop-exit=0( |$)' "$work/stop-fails.calls.out"; then
    selftest_cell_failed stop-exit "清场命令退 1 而回读过了，清场那一行没带 stop-exit=1、没照跑或没退 0，或清场回读不过那一行没带 stop-exit=0（退 $cell_exit，次序「$(calls_of "$calls")」，清场那一行「$(grep '^GPU_QUIESCE step=quiesce ' "$calls.out")」）" \
      "看清场那一段的 gpu_quiesce_stop_exit 与 gpu_quiesce_stop_field：这一趟跑了清场命令、而回读没过或它退非 0 时，那一行带 stop-exit=<码>"
  fi
  # 格 failure-log：清场回读不过 ⇒ GPU_QUIESCE 清场那一行带 log=<本机日志>；日志 600、所在目录 700，里面有回读命令的两路输出与「退 1」那一行；
  #   清场命令退非 0 而回读过了（格 stop-exit 那一趟）同样留下、带 log=；私有输出不上标准错误；全绿那一趟（格 quiesce-run 的状态目录）不留日志；
  #   不设 GPU_QUIESCE_LOG_DIRECTORY 时日志在 <工作根>/gpu-quiesce-logs/ 下（工作根指到这里的临时目录），不在 XDG_RUNTIME_DIR 下
  cells=$((cells + 1))
  local failure_log failure_mode="" failure_directory_mode="" refused_log default_log runtime_files
  failure_log="$(sed -nE 's/^GPU_QUIESCE step=quiesce result=read-back-failed .* log=([^ ]+)$/\1/p' "$work/stop-fails.calls.out")"
  if [[ -n "$failure_log" && -f "$failure_log" ]]; then
    failure_mode="$(stat -c %a -- "$failure_log")"; failure_directory_mode="$(stat -c %a -- "${failure_log%/*}")"
  fi
  refused_log="$(sed -nE 's/^GPU_QUIESCE step=quiesce result=stopped-and-read-back .* log=([^ ]+)$/\1/p' "$work/stop-refused.calls.out")"
  mkdir -p "$work/runtime"
  env -u GPU_QUIESCE_LOG_DIRECTORY SINGLEFS_WORK_ROOT="$work/work-root" XDG_RUNTIME_DIR="$work/runtime" SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host.env" \
    GPU_QUIESCE_STATE_DIRECTORY="$work/state-default-log" FAKE_CALLS="$work/default-log.calls" FAKE_STOPPED_CHECK=fail \
    bash "$0" -- bash "$work/command.sh" ok >"$work/default-log.calls.out" 2>&1
  default_log="$(sed -nE 's/^GPU_QUIESCE step=quiesce result=read-back-failed .* log=([^ ]+)$/\1/p' "$work/default-log.calls.out")"
  runtime_files="$(find "$work/runtime" -type f | wc -l)"
  if [[ "$failure_mode" != 600 || "$failure_directory_mode" != 700 ]] || ! grep -q '^fake-private-unit-stdout$' "$failure_log" || ! grep -q '^fake-private-unit-stderr$' "$failure_log" \
     || ! grep -q '^fake-private-stop-output$' "$failure_log" || ! grep -q '^== QUIESCE_STOPPED_CHECK_COMMAND 退 1$' "$failure_log" \
     || [[ -z "$refused_log" || ! -f "$refused_log" ]] || ! grep -q '^== QUIESCE_STOP_COMMAND 退 1$' "$refused_log" \
     || grep -q 'fake-private-' "$work/stop-fails.calls.out" "$work/green.calls.out" "$work/stop-refused.calls.out" "$work/default-log.calls.out" \
     || compgen -G "$work/state-green/logs/*" >/dev/null \
     || [[ "$default_log" != "$work/work-root/gpu-quiesce-logs/"* || ! -f "$default_log" || "$runtime_files" != 0 ]]; then
    selftest_cell_failed failure-log "回读没过（或清场命令退非 0）没留本机日志、日志不是 600 或目录不是 700、缺清场与回读命令的输出、私有输出打到了标准错误上、全绿那一趟留了日志，或默认位置不在工作根下（清场回读不过的日志「${failure_log:-没打 log=}」权限「${failure_mode:-无}」目录「${failure_directory_mode:-无}」；清场退 1 的日志「${refused_log:-没打 log=}」；默认位置「${default_log:-没打 log=}」，XDG_RUNTIME_DIR 下 $runtime_files 份）" \
      "看 gpu_quiesce_run_configured（输出进本机日志）、gpu_quiesce_keep_log、gpu_quiesce_finish_log 与取日志目录那一段（默认 <工作根>/gpu-quiesce-logs，工作根经 .claude/hooks/lib_work_root.py 取）"
  fi
  # 格 command-fails：命令退 7 ⇒ 交回 7
  cells=$((cells + 1)); calls="$work/fail.calls"; : >"$calls"
  run_wrapper "$calls" "$work/state-fail" -- bash "$work/command.sh" fail; cell_exit=$?
  if [[ "$cell_exit" != 7 || "$(calls_of "$calls")" != "stop stopped-check command-fail-start command-fail-end " ]]; then
    selftest_cell_failed command-fails "命令失败之后没交回命令的退出码 7（退 $cell_exit，次序「$(calls_of "$calls")」）" "看末尾的退出码：命令起了就交回它的"
  fi
  # 格 interrupted：① 命令跑着时这一份收到 TERM（kill 自己起的那一份的进程号）⇒ 转给命令（默认处理，它当场停）、退 143，命令没跑到结束；② 命令自己被 TERM 打断 ⇒ 退 143
  cells=$((cells + 1)); calls="$work/term.calls"; : >"$calls"
  # 直接起这一份（不经 run_wrapper 那层函数），$! 才是这一份自己的进程号
  env SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host.env" GPU_QUIESCE_STATE_DIRECTORY="$work/state-term" FAKE_CALLS="$calls" \
    bash "$0" -- bash "$work/command.sh" sleep:1 >"$calls.out" 2>&1 &
  local wrapper_pid=$! interrupted_exit=""
  if wait_for_line "$calls" "command-sleep:1-start"; then kill -TERM "$wrapper_pid"; fi
  wait "$wrapper_pid"; interrupted_exit=$?
  calls="$work/self-term.calls"; : >"$calls"
  run_wrapper "$calls" "$work/state-self-term" -- bash "$work/command.sh" self-term; cell_exit=$?
  if [[ "$interrupted_exit" != 143 || "$(calls_of "$work/term.calls")" != "stop stopped-check command-sleep:1-start " ]] \
     || [[ "$cell_exit" != 143 || "$(calls_of "$calls")" != "stop stopped-check command-self-term-start " ]]; then
    selftest_cell_failed interrupted "被信号打断之后没把信号转给命令（命令跑到了结束），或没退 143（这一份收到 TERM：退 $interrupted_exit，次序「$(calls_of "$work/term.calls")」；命令被 TERM：退 $cell_exit，次序「$(calls_of "$calls")」）" \
      "看 INT/TERM/HUP 的 trap（gpu_quiesce_on_signal：记下信号、转给命令的进程组）、gpu_quiesce_wait_command 与末尾的退出码"
  fi
  # 被打断时的转发、限时与守卫（文件头「被打断」）五格。假命令 term-aware / ignore-term / int-aware 起一个孙辈 sleep（同一个进程组），把两个进程号记进 FAKE_PIDS.command / .child，
  #   带一个只此一趟的记号参数：扫 /proc/*/cmdline 看这一份、守卫、命令有没有留下（孙辈 sleep 的命令行里没有记号，用记下的进程号认）
  pid_running() { # <进程号>：在跑（僵尸不算）
    local stat_line
    [[ -n "${1:-}" ]] && { read -r stat_line <"/proc/$1/stat"; } 2>/dev/null || return 1
    stat_line="${stat_line##*) }"
    [[ "${stat_line%% *}" != Z ]]
  }
  leftover_count() { # <记号>：命令行里带记号的进程数（模式写成 [l]eak，grep 自己的命令行不匹配）
    grep -la -- "[l]eak-token-$1" /proc/[0-9]*/cmdline 2>/dev/null | wc -l
  }
  wait_no_leftover() { # <记号> <至多等几秒>
    local deadline=$((SECONDS + $2))   # 按挂钟算，不按轮数：机器忙时扫一遍 /proc 要好几秒
    while (( SECONDS < deadline )); do [[ "$(leftover_count "$1")" == 0 ]] && return 0; sleep 0.05; done
    [[ "$(leftover_count "$1")" == 0 ]]
  }
  start_forward_wrapper() { # <格名> <假命令的模式> <记号> [<环境 键=值>…]：直接起这一份（$! 才是它自己的进程号），置 forward_wrapper_pid、forward_calls、forward_pids
    local cell="$1" mode="$2" token="$3"; shift 3
    forward_calls="$work/$cell.calls"; forward_pids="$work/$cell.pids"; : >"$forward_calls"
    # env --default-signal：放后台起的这一份没有作业控制，INT、QUIT 被置成忽略，这一份的 INT 的 trap 就设不上（格 forward-interrupt 要它收得到 INT）
    env --default-signal=INT,QUIT SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host.env" GPU_QUIESCE_STATE_DIRECTORY="$work/state-$cell" FAKE_CALLS="$forward_calls" FAKE_PIDS="$forward_pids" "$@" \
      bash "$0" -- bash "$work/command.sh" "$mode" "leak-token-$token" >"$forward_calls.out" 2>&1 &
    forward_wrapper_pid=$!
  }
  await_forward_wrapper() { # <至多等几秒>：等 forward_wrapper_pid 退出，置 forward_exit；到点还没退就 KILL 它、forward_exit 置 stuck，并按记下的进程号收掉假命令（不留进程）
    local deadline=$((SECONDS + $1))
    while (( SECONDS < deadline )); do kill -0 "$forward_wrapper_pid" 2>/dev/null || break; sleep 0.05; done
    if kill -0 "$forward_wrapper_pid" 2>/dev/null; then
      kill -KILL "$forward_wrapper_pid" 2>/dev/null; wait "$forward_wrapper_pid" 2>/dev/null; forward_exit=stuck
    else
      wait "$forward_wrapper_pid"; forward_exit=$?
    fi
  }
  reap_recorded_process() { # <记下进程号的文件>：文件里是假命令或它的孙辈（这一格自己起的）写下的一个进程号；还在跑就停这一个
    local recorded_pid
    [[ -s "$1" ]] || return 0
    recorded_pid="$(cat -- "$1")"
    pid_running "$recorded_pid" && kill -KILL "$recorded_pid" 2>/dev/null
    return 0
  }
  reap_forward_command() { # 按记下的进程号收掉假命令和它的孙辈（只这两个自己起的进程号，一次一个）
    reap_recorded_process "$forward_pids.command"
    reap_recorded_process "$forward_pids.child"
  }
  # 机器忙的时候（开关轮一批起很多份）进程收尾要几秒：各格等「停了没有」的上限放宽到这个数；停对了的格不会等满，只有该红的格等满
  local forward_settle_seconds=15
  local round forward_ready forward_deadline forward_wrapper_pid forward_calls forward_pids forward_exit started_microseconds elapsed_milliseconds forward_token command_running child_running leftovers_state
  # 格 forward-term：命令跑着时这一份收到 TERM ⇒ TERM 转给命令的进程组（命令记 command-got-term、孙辈 sleep 也停），这一份退 143，不留进程
  cells=$((cells + 1)); forward_token="$$-term"
  start_forward_wrapper forward-term "term-aware:60" "$forward_token"
  forward_ready=no
  if wait_for_line "$forward_calls" command-ready "$forward_settle_seconds"; then forward_ready=yes; kill -TERM "$forward_wrapper_pid"; fi
  await_forward_wrapper $((GPU_QUIESCE_FORWARD_WAIT_SECONDS + forward_settle_seconds))
  command_running=no; child_running=no
  pid_running "$(cat -- "$forward_pids.command" 2>/dev/null)" && command_running=yes
  pid_running "$(cat -- "$forward_pids.child" 2>/dev/null)" && child_running=yes
  wait_no_leftover "$forward_token" "$forward_settle_seconds" && leftovers_state=none || leftovers_state="$(leftover_count "$forward_token") 个"
  if [[ "$forward_ready" != yes || "$forward_exit" != 143 ]] || ! grep -qx command-got-term "$forward_calls" || [[ "$command_running" != no || "$child_running" != no || "$leftovers_state" != none ]]; then
    selftest_cell_failed forward-term "这一份收到 TERM 没把它转给命令的进程组、没退 143，或命令、孙辈、这一份、守卫有没停的（假命令在 ${forward_settle_seconds} 秒内起好了：$forward_ready，退 ${forward_exit}，命令收到 TERM「$(grep -cx command-got-term "$forward_calls")」次，命令还在跑：$command_running，孙辈还在跑：$child_running，命令行带记号的进程留下 $leftovers_state）" \
      "看 gpu_quiesce_on_signal / gpu_quiesce_forward_signal（kill -s <信号> -- -<命令的进程号>）、起命令那一段（gpu_quiesce_start_command 的 setsid --wait）与 gpu_quiesce_guard_stop"
  fi
  reap_forward_command
  # 格 forward-interrupt：命令跑着时这一份收到 INT ⇒ INT 转给命令的进程组（命令记 command-got-interrupt、停孙辈再退），这一份退 130，不留进程。
  #   起命令放后台时 INT 被置成忽略（没开作业控制），起命令那一步要还原成默认（gpu_quiesce_start_command 的 env --default-signal）；开关 inherited-ignored-int 下不还原，命令收不到 INT，等限时之后被 KILL（退 137）
  cells=$((cells + 1)); forward_token="$$-interrupt"
  start_forward_wrapper forward-interrupt "int-aware:60" "$forward_token"
  forward_ready=no
  if wait_for_line "$forward_calls" command-ready "$forward_settle_seconds"; then forward_ready=yes; kill -INT "$forward_wrapper_pid"; fi
  await_forward_wrapper $((GPU_QUIESCE_FORWARD_WAIT_SECONDS + forward_settle_seconds))
  command_running=no; child_running=no
  pid_running "$(cat -- "$forward_pids.command" 2>/dev/null)" && command_running=yes
  pid_running "$(cat -- "$forward_pids.child" 2>/dev/null)" && child_running=yes
  wait_no_leftover "$forward_token" "$forward_settle_seconds" && leftovers_state=none || leftovers_state="$(leftover_count "$forward_token") 个"
  if [[ "$forward_ready" != yes || "$forward_exit" != 130 ]] || ! grep -qx command-got-interrupt "$forward_calls" || [[ "$command_running" != no || "$child_running" != no || "$leftovers_state" != none ]]; then
    selftest_cell_failed forward-interrupt "这一份收到 INT 没把它转给命令（命令没记到 command-got-interrupt）、没退 130，或命令、孙辈、这一份、守卫有没停的（假命令在 ${forward_settle_seconds} 秒内起好了：$forward_ready，退 ${forward_exit}，命令收到 INT「$(grep -cx command-got-interrupt "$forward_calls")」次，命令还在跑：$command_running，孙辈还在跑：$child_running，命令行带记号的进程留下 $leftovers_state）" \
      "看 gpu_quiesce_start_command：放后台起的命令要用 env --default-signal=INT,QUIT 还原 INT、QUIT，再看 gpu_quiesce_forward_signal"
  fi
  reap_forward_command
  # 格 forward-limit：命令不理 TERM ⇒ 转发之后等满限时（GPU_QUIESCE_FORWARD_WAIT_SECONDS=2）才对它的进程组发 KILL，这一份退命令被 KILL 的 137，不留进程；没等够限时就杀也红
  cells=$((cells + 1)); forward_token="$$-limit"
  start_forward_wrapper forward-limit "ignore-term:60" "$forward_token"
  started_microseconds=0; forward_ready=no
  if wait_for_line "$forward_calls" command-ready "$forward_settle_seconds"; then forward_ready=yes; started_microseconds="${EPOCHREALTIME/./}"; kill -TERM "$forward_wrapper_pid"; fi
  await_forward_wrapper $((GPU_QUIESCE_FORWARD_WAIT_SECONDS + forward_settle_seconds))
  elapsed_milliseconds=$(( (${EPOCHREALTIME/./} - started_microseconds) / 1000 ))
  command_running=no; child_running=no
  pid_running "$(cat -- "$forward_pids.command" 2>/dev/null)" && command_running=yes
  pid_running "$(cat -- "$forward_pids.child" 2>/dev/null)" && child_running=yes
  wait_no_leftover "$forward_token" "$forward_settle_seconds" && leftovers_state=none || leftovers_state="$(leftover_count "$forward_token") 个"
  if [[ "$forward_ready" != yes || "$forward_exit" != 137 ]] || grep -qx command-got-term "$forward_calls" || (( elapsed_milliseconds < GPU_QUIESCE_FORWARD_WAIT_SECONDS * 1000 - 300 )) \
     || [[ "$command_running" != no || "$child_running" != no || "$leftovers_state" != none ]]; then
    selftest_cell_failed forward-limit "命令不理 TERM，这一份没在限时（${GPU_QUIESCE_FORWARD_WAIT_SECONDS} 秒）到点之后对它的进程组发 KILL、没退 137、等不够限时就动了手，或命令、孙辈、这一份、守卫有没停的（假命令在 ${forward_settle_seconds} 秒内起好了：$forward_ready，退 ${forward_exit}，从发 TERM 到这一份退出 ${elapsed_milliseconds} 毫秒，命令还在跑：$command_running，孙辈还在跑：$child_running，命令行带记号的进程留下 $leftovers_state）" \
      "看 gpu_quiesce_arm_limit（转发之后起一个睡满限时再 kill -s KILL -- -<命令的进程号> 的子 shell）与 gpu_quiesce_disarm_limit"
  fi
  reap_forward_command
  # 格 kill-wrapper：这一份被 KILL（没有 trap 可跑）⇒ 守卫发现它没了，对命令的进程组发 TERM、限时之后发 KILL，命令、孙辈、守卫都不留（命令不理 TERM，两步都走到）
  cells=$((cells + 1)); forward_token="$$-kill"
  start_forward_wrapper kill-wrapper "ignore-term:60" "$forward_token"
  forward_ready=no
  if wait_for_line "$forward_calls" command-ready "$forward_settle_seconds"; then forward_ready=yes; kill -KILL "$forward_wrapper_pid"; fi
  wait "$forward_wrapper_pid" 2>/dev/null
  forward_deadline=$((SECONDS + GPU_QUIESCE_FORWARD_WAIT_SECONDS + forward_settle_seconds))
  while (( SECONDS < forward_deadline )); do
    pid_running "$(cat -- "$forward_pids.command" 2>/dev/null)" || pid_running "$(cat -- "$forward_pids.child" 2>/dev/null)" || break
    sleep 0.05
  done
  command_running=no; child_running=no
  pid_running "$(cat -- "$forward_pids.command" 2>/dev/null)" && command_running=yes
  pid_running "$(cat -- "$forward_pids.child" 2>/dev/null)" && child_running=yes
  wait_no_leftover "$forward_token" "$forward_settle_seconds" && leftovers_state=none || leftovers_state="$(leftover_count "$forward_token") 个"
  if [[ "$forward_ready" != yes || "$command_running" != no || "$child_running" != no || "$leftovers_state" != none ]]; then
    selftest_cell_failed kill-wrapper "这一份被 KILL 之后命令的进程组没停（假命令在 ${forward_settle_seconds} 秒内起好了：$forward_ready，命令还在跑：$command_running，孙辈还在跑：$child_running，命令行带记号的进程留下 $leftovers_state）" \
      "看 gpu_quiesce_guard_start（守卫按进程号加起始时刻看这一份还在不在，没了就 kill -s TERM、限时后 kill -s KILL -- -<命令的进程号>）与起命令那一段有没有调它"
  fi
  reap_forward_command
  # 格 guardian-stopped：命令正常退出之后守卫不留（守卫的间隔取比等的上限还长的 30 秒，留着的话等满上限也还在；开关 leave-guardian-running 下它留到 30 秒后自己发现这一份没了、退出）
  cells=$((cells + 1)); forward_token="$$-guard"; calls="$work/guardian.calls"; : >"$calls"
  run_wrapper "$calls" "$work/state-guardian" GPU_QUIESCE_GUARD_POLL_SECONDS=30 -- bash "$work/command.sh" ok "leak-token-$forward_token"; cell_exit=$?
  wait_no_leftover "$forward_token" "$forward_settle_seconds" && leftovers_state=none || leftovers_state="$(leftover_count "$forward_token") 个"
  if [[ "$cell_exit" != 0 || "$leftovers_state" != none ]]; then
    selftest_cell_failed guardian-stopped "命令正常退出之后守卫还在（退 $cell_exit，命令行带记号的进程留下 $leftovers_state）" "看 gpu_quiesce_guard_stop：命令退出之后、出去之前 KILL 守卫"
  fi
  # 格 overlapping-runs：A 先进来清场，B 跟着进来只回读（joined-and-read-back holders=1）；清场命令只跑一次，两趟都退 0
  cells=$((cells + 1)); calls="$work/overlap.calls"; : >"$calls"
  run_wrapper "$calls" "$work/state-overlap" -- bash "$work/command.sh" "wait-for:$work/b-started" &
  local first_pid=$! first_exit second_exit
  wait_for_line "$calls" "command-wait-for:$work/b-started-start"
  cp -- "$calls" "$work/overlap.before-b"
  env SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host.env" GPU_QUIESCE_STATE_DIRECTORY="$work/state-overlap" FAKE_CALLS="$calls" \
    bash "$0" -- bash -c "touch '$work/b-started'; for ((round = 0; round < 200; round++)); do [[ -e '$work/a-done' ]] && break; sleep 0.05; done; echo command-b-end >> '$calls'" \
    >"$work/overlap-b.out" 2>&1 &
  local second_pid=$!
  wait "$first_pid"; first_exit=$?
  touch "$work/a-done"
  wait "$second_pid"; second_exit=$?
  local overlap_calls; overlap_calls="$(calls_of "$calls")"
  if [[ "$first_exit" != 0 || "$second_exit" != 0 || "$(grep -cx stop "$calls")" != 1 ]] \
     || [[ "$overlap_calls" != *"command-b-end " ]] || ! grep -q '^GPU_QUIESCE step=quiesce result=joined-and-read-back holders=1$' "$work/overlap-b.out"; then
    selftest_cell_failed overlapping-runs "两趟重叠时没有只清一次、后来的那一趟没跟着用（A 退 $first_exit、B 退 $second_exit，次序「$overlap_calls」）" "看 gpu_quiesce_enter 里 gpu_quiesce_other_live_holders 那一处"
  fi
  # 格 rejoin-stops-again：A 清过场还在用卡，之后服务又起来了（FAKE_SERVICES_UP 标记在，演另一台重启拉起了被停的服务）⇒ B 跟着进来回读没过，
  #   再清一次场（标记被删）、回读过了、退 0（stopped-and-read-back holders=1）；开关 joined-stale-quiesce-trusted 下 B 回读不过、退 78
  cells=$((cells + 1)); : >"$work/rejoin-holder.calls"
  run_wrapper "$work/rejoin-holder.calls" "$work/state-rejoin" -- bash "$work/command.sh" hold "$work/rejoin-child.pid" "$work/rejoin-release" &
  local rejoin_holder_pid=$! rejoin_exit rejoin_holder_exit
  for ((round = 0; round < 2000; round++)); do [[ -s "$work/rejoin-child.pid" ]] && break; sleep 0.05; done
  : >"$work/services-up"
  calls="$work/rejoin.calls"; : >"$calls"
  run_wrapper "$calls" "$work/state-rejoin" FAKE_SERVICES_UP="$work/services-up" -- bash "$work/command.sh" ok; rejoin_exit=$?
  touch "$work/rejoin-release"
  wait "$rejoin_holder_pid"; rejoin_holder_exit=$?
  if [[ "$rejoin_exit" != 0 || "$rejoin_holder_exit" != 0 || -e "$work/services-up" ]] \
     || ! grep -q '^GPU_QUIESCE step=quiesce result=stopped-and-read-back holders=1$' "$calls.out"; then
    selftest_cell_failed rejoin-stops-again "跟着别的登记用卡、服务又起来了时，后来的那一趟没再清一次场（B 退 $rejoin_exit、A 退 $rejoin_holder_exit，服务标记还在：$([[ -e "$work/services-up" ]] && echo 在 || echo 不在)，清场那一行「$(grep '^GPU_QUIESCE step=quiesce ' "$calls.out")」）" \
      "看进来那一段：跟着别的登记用卡而回读没过时再跑一次 QUIESCE_STOP_COMMAND、再回读"
  fi
  # 格 stale-holder：holders/ 下有一份死了的登记（进程号已经不在）⇒ 不当它在用，照常清场，并删掉它
  cells=$((cells + 1)); calls="$work/stale.calls"; : >"$calls"
  mkdir -p "$work/state-stale/holders"
  bash -c 'exit 0' & local dead_pid=$!; wait "$dead_pid"
  : >"$work/state-stale/holders/$dead_pid-1"
  run_wrapper "$calls" "$work/state-stale" -- bash "$work/command.sh" ok; cell_exit=$?
  if [[ "$cell_exit" != 0 || "$(calls_of "$calls")" != "stop stopped-check command-ok-start holder-of-parent command-ok-end " || -e "$work/state-stale/holders/$dead_pid-1" ]]; then
    selftest_cell_failed stale-holder "死了的登记挡住了清场、或没删掉（退 $cell_exit，次序「$(calls_of "$calls")」）" "看 gpu_quiesce_other_live_holders：进程不在或起始时刻对不上的登记删掉、不计数"
  fi
  # 格 no-restore：跑完、跑红、清场回读没过、被打断都不调复原命令：上面各格那几趟的调用记录里一笔 start / started-check 都没有，
  #   出去时那一行是 step=restore result=not-restored；多机配置里没写复原那一对也照跑；同一个状态目录上一趟清了场没复原、
  #   接着再起一趟照常清场、回读、跑（服务已经停着时私有清场脚本只回读、退 0，这里的假清场命令照样退 0）
  cells=$((cells + 1)); calls="$work/no-restore.calls"; : >"$calls"
  local no_restore_first_exit restore_calls restore_lines_missing=""
  run_wrapper "$calls" "$work/state-no-restore" -- bash "$work/command.sh" ok; no_restore_first_exit=$?
  cp -- "$calls.out" "$work/no-restore-first.out"
  run_wrapper "$calls" "$work/state-no-restore" SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-without-restore.env" -- bash "$work/command.sh" ok; cell_exit=$?
  restore_calls="$(cat -- "$work"/*.calls | grep -cxE 'start|started-check')"
  for no_restore_output in green.calls.out fail.calls.out stop-fails.calls.out term.calls.out self-term.calls.out no-restore-first.out no-restore.calls.out; do
    grep -q '^GPU_QUIESCE step=restore result=not-restored holders=0$' "$work/$no_restore_output" || restore_lines_missing+="$no_restore_output "
  done
  if [[ "$no_restore_first_exit" != 0 || "$cell_exit" != 0 || "$restore_calls" != 0 || -n "$restore_lines_missing" ]] \
     || [[ "$(tr '\n' ' ' <"$calls")" != "stop stopped-check command-ok-start holder-of-parent command-ok-end stop stopped-check command-ok-start holder-of-parent command-ok-end " ]] \
     || ! grep -q '清了场、没复原' "$work/no-restore.calls.out"; then
    selftest_cell_failed no-restore "跑完调了复原命令（各格的调用记录里 start / started-check 共 $restore_calls 笔），或出去时那一行不是 result=not-restored（缺的：${restore_lines_missing:-无}），或没写复原那一对的配置、接着上一趟再起一趟没照跑（两趟退 $no_restore_first_exit、$cell_exit，次序「$(tr '\n' ' ' <"$calls")」）" \
      "看 gpu_quiesce_leave：只删自己的登记、打 step=restore result=not-restored，不跑 QUIESCE_START_COMMAND；读配置那一段只要清场那两个键"
  fi
  # 格 usage：没给 -- 或 -- 后面没有命令 ⇒ 退 2，什么都不跑
  cells=$((cells + 1)); calls="$work/usage.calls"; : >"$calls"
  run_wrapper "$calls" "$work/state-usage" -- ; cell_exit=$?
  local usage_second_exit
  env SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host.env" GPU_QUIESCE_STATE_DIRECTORY="$work/state-usage" FAKE_CALLS="$calls" \
    bash "$0" bash "$work/command.sh" ok >"$calls.out" 2>&1; usage_second_exit=$?
  if [[ "$cell_exit" != 2 || "$usage_second_exit" != 2 || -s "$calls" ]]; then
    selftest_cell_failed usage "用法错没退 2（-- 后面空着退 $cell_exit，没给 -- 退 $usage_second_exit），或跑了东西" "看参数那一段"
  fi
  # 格 admission-refused：环境里没有服务给的准入标记 ⇒ 退 2、清场命令一条没起、命令没跑，出路写「任务里面不再套包装」
  cells=$((cells + 1)); calls="$work/unadmitted.calls"; : >"$calls"
  env -u GPU_QUIESCE_RUN_ADMITTED_BY_TASK_SERVER SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host.env" GPU_QUIESCE_STATE_DIRECTORY="$work/state-unadmitted" \
    GPU_QUIESCE_LOG_DIRECTORY="$work/state-unadmitted/logs" FAKE_CALLS="$calls" bash "$0" -- bash "$work/command.sh" ok >"$calls.out" 2>&1; cell_exit=$?
  if [[ "$cell_exit" != 2 || -s "$calls" ]] || ! grep -q '任务里面不再套包装：内存、线程、机器、卡由 task-server 定；删掉这一层' "$calls.out"; then
    selftest_cell_failed admission-refused "没有服务给的准入标记却没拒（退 $cell_exit，调用次序「$(calls_of "$calls")」），或出路没写「任务里面不再套包装」" "看 case 之后读 GPU_QUIESCE_RUN_ADMITTED_BY_TASK_SERVER 那一段"
  fi
  # 格 nested-call-refused：服务起的这一层里面，命令再套一层清场 ⇒ 里层拒、退 2，命令没跑；外层照常清场、交回里层的 2
  cells=$((cells + 1)); calls="$work/nested.calls"; : >"$calls"
  run_wrapper "$calls" "$work/state-nested" -- bash "$0" -- bash "$work/command.sh" ok; cell_exit=$?
  if [[ "$cell_exit" != 2 || "$(calls_of "$calls")" != "stop stopped-check " ]]; then
    selftest_cell_failed nested-call-refused "被服务起的命令里面再套一层清场，里层应当拒、命令不跑，实际外层退 $cell_exit、调用次序「$(calls_of "$calls")」" \
      "看 case 之后那一段：读了 GPU_QUIESCE_RUN_ADMITTED_BY_TASK_SERVER 要 unset，命令里看不到它"
  fi

  # 回读看哪几张卡、跳过哪些进程（文件头那一段）四格。外人是这里起的一个进程：不在任何登记之下、与登记同一个会话，见到 outsider-release 退出（至多 600 秒：放行由格子做，时限只防格子半路出错时不收）。
  #   回读过没过只看 GPU_QUIESCE 清场那一行（回读不过还起不起命令归格 quiesce-read-back-fails）
  local round outsider_pid holder_child first_holder_pid holder_exit descendants_exit outsider_seen
  seen_exempt() { printf ' %s ' "$(sed -n 's/^cards=[^ ]* exempt=//p' "$1" 2>/dev/null | head -n 1)"; }
  bash -c 'for ((round = 0; round < 12000; round++)); do [[ -e "$1" ]] && break; sleep 0.05; done' outsider "$work/outsider-release" &
  outsider_pid=$!
  # 格 holder-descendants：前一趟 A 在用卡（登记活着），它的孙辈进程在卡 0 上；B 进来只回读 ⇒ 回读过（joined-and-read-back holders=1）、退 0，
  #   回读拿到的 GPU_QUIESCE_CHECK_EXEMPT_PIDS 里有那个孙辈；A 放行之后退 0
  # 格 outsider-not-exempt：A 还在用卡，卡 0 上另有外人 ⇒ B 回读不过（read-back-failed），跳过的进程号里没有外人
  cells=$((cells + 2)); : >"$work/holder.calls"
  run_wrapper "$work/holder.calls" "$work/state-holders" -- bash "$work/command.sh" hold "$work/holder-child.pid" "$work/holder-release" &
  first_holder_pid=$!
  for ((round = 0; round < 2000; round++)); do [[ -s "$work/holder-child.pid" ]] && break; sleep 0.05; done
  holder_child="$(cat -- "$work/holder-child.pid" 2>/dev/null)"
  printf '0 %s\n' "$holder_child" >"$work/gpu-descendant.list"
  printf '0 %s\n0 %s\n' "$holder_child" "$outsider_pid" >"$work/gpu-outsider.list"
  calls="$work/descendants.calls"; : >"$calls"
  run_wrapper "$calls" "$work/state-holders" FAKE_GPU_PROCESSES="$work/gpu-descendant.list" FAKE_CHECK_SEEN="$work/descendants.seen" -- bash "$work/command.sh" ok; descendants_exit=$?
  calls="$work/outsider.calls"; : >"$calls"
  run_wrapper "$calls" "$work/state-holders" FAKE_GPU_PROCESSES="$work/gpu-outsider.list" FAKE_CHECK_SEEN="$work/outsider.seen" -- bash "$work/command.sh" ok
  touch "$work/holder-release"
  wait "$first_holder_pid"; holder_exit=$?
  if [[ -z "$holder_child" || "$holder_exit" != 0 || "$descendants_exit" != 0 ]] \
     || ! grep -q '^GPU_QUIESCE step=quiesce result=joined-and-read-back holders=1$' "$work/descendants.calls.out" \
     || [[ "$(seen_exempt "$work/descendants.seen")" != *" $holder_child "* ]]; then
    selftest_cell_failed holder-descendants "前一趟登记下的进程在卡上，后一趟进来回读没过（A 的孙辈「${holder_child:-没起来}」，A 退 $holder_exit，B 退 $descendants_exit，B 的清场那一行「$(grep '^GPU_QUIESCE step=quiesce ' "$work/descendants.calls.out")」，回读看到的「$(head -n 1 "$work/descendants.seen" 2>/dev/null)」）" \
      "看 gpu_quiesce_holder_descendants（别的活着的登记与它们的子孙，按 /proc 的父进程号往下找）与进来那一段：它们经 GPU_QUIESCE_CHECK_EXEMPT_PIDS 交给回读"
  fi
  outsider_seen="$(seen_exempt "$work/outsider.seen")"
  if ! grep -Eq '^GPU_QUIESCE step=quiesce result=read-back-failed holders=1( |$)' "$work/outsider.calls.out" \
     || [[ ! -s "$work/outsider.seen" || "$outsider_seen" == *" $outsider_pid "* ]]; then
    selftest_cell_failed outsider-not-exempt "不在任何登记之下的进程在卡上，回读却过了（外人 $outsider_pid，清场那一行「$(grep '^GPU_QUIESCE step=quiesce ' "$work/outsider.calls.out")」，回读看到的「$(head -n 1 "$work/outsider.seen" 2>/dev/null)」）" \
      "看 gpu_quiesce_holder_descendants：只跳过别的活着的登记自己与它们的子孙，不按会话、进程组或名字认自家的"
  fi
  # 格 unused-card-outsider：没有别的登记，这一趟的 GPU_CARDS_BY_PRIORITY 是「192.0.2.1:0:8G 127.0.0.1:1:8G」，外人在本机卡 0 上 ⇒ 回读只看本机卡 1、过了，照跑、退 0
  printf '0 %s\n' "$outsider_pid" >"$work/gpu-card0.list"
  cells=$((cells + 1)); calls="$work/unused-card.calls"; : >"$calls"
  run_wrapper "$calls" "$work/state-unused-card" GPU_CARDS_BY_PRIORITY='192.0.2.1:0:8G 127.0.0.1:1:8G' FAKE_GPU_PROCESSES="$work/gpu-card0.list" FAKE_CHECK_SEEN="$work/unused-card.seen" -- bash "$work/command.sh" ok; cell_exit=$?
  if [[ "$cell_exit" != 0 || "$(head -n 1 "$work/unused-card.seen" 2>/dev/null)" != "cards=1 exempt=unset" ]] \
     || ! grep -q '^GPU_QUIESCE step=quiesce result=stopped-and-read-back holders=0$' "$calls.out" \
     || ! grep -q '回读看的卡：本机卡 1（环境变量 GPU_CARDS_BY_PRIORITY）' "$calls.out"; then
    selftest_cell_failed unused-card-outsider "这一趟只用卡 1，卡 0 上的外人挡住了回读（退 $cell_exit，清场那一行「$(grep '^GPU_QUIESCE step=quiesce ' "$calls.out")」，回读看到的「$(head -n 1 "$work/unused-card.seen" 2>/dev/null)」）" \
      "看 gpu_quiesce_choose_cards 与共用库 crash_amplification_cards_on_address：GPU_CARDS_BY_PRIORITY 里地址是这一台的那几项的卡号经 GPU_QUIESCE_CHECK_LOCAL_CARDS 交给回读，另一台的不算"
  fi
  # 格 used-card-outsider：没设 GPU_CARDS_BY_PRIORITY，多机配置清场之后那一份点名本机卡 0（不清场那一份点名卡 1），外人在卡 0 上；
  #   调用方另带进来 GPU_QUIESCE_CHECK_LOCAL_CARDS=1 与 GPU_QUIESCE_CHECK_EXEMPT_PIDS=<外人> ⇒ 不认它们，回读只看卡 0、不跳过外人、不过
  cells=$((cells + 1)); calls="$work/used-card.calls"; : >"$calls"
  run_wrapper "$calls" "$work/state-used-card" SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-cards.env" GPU_QUIESCE_CHECK_LOCAL_CARDS=1 GPU_QUIESCE_CHECK_EXEMPT_PIDS="$outsider_pid" \
    FAKE_GPU_PROCESSES="$work/gpu-card0.list" FAKE_CHECK_SEEN="$work/used-card.seen" -- bash "$work/command.sh" ok
  if [[ "$(head -n 1 "$work/used-card.seen" 2>/dev/null)" != "cards=0 exempt=unset" ]] \
     || ! grep -Eq '^GPU_QUIESCE step=quiesce result=read-back-failed holders=0( |$)' "$calls.out"; then
    selftest_cell_failed used-card-outsider "这一趟用卡 0，卡 0 上的外人却没挡住回读（清场那一行「$(grep '^GPU_QUIESCE step=quiesce ' "$calls.out")」，回读看到的「$(head -n 1 "$work/used-card.seen" 2>/dev/null)」）" \
      "看 gpu_quiesce_choose_cards（没设 GPU_CARDS_BY_PRIORITY 读多机配置的 GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED）与参数之后清掉调用方带进来的 GPU_QUIESCE_CHECK_* 那一行"
  fi
  # 格 peer-only-cards：这一趟的 GPU_CARDS_BY_PRIORITY 只有「192.0.2.1:0:8G」，外人在本机卡 0 上 ⇒ 本机一张都不看（none）、另一台只看卡 0，过了，照跑、退 0
  cells=$((cells + 1)); calls="$work/peer-only.calls"; : >"$calls"
  run_wrapper "$calls" "$work/state-peer-only" GPU_CARDS_BY_PRIORITY='192.0.2.1:0:8G' FAKE_GPU_PROCESSES="$work/gpu-card0.list" FAKE_CHECK_SEEN="$work/peer-only.seen" -- bash "$work/command.sh" ok; cell_exit=$?
  if [[ "$cell_exit" != 0 || "$(head -n 1 "$work/peer-only.seen" 2>/dev/null)" != "cards=none exempt=unset" || "$(head -n 1 "$work/peer-only.seen.peer" 2>/dev/null)" != "peer=0" ]] \
     || ! grep -q '回读看的卡：本机一张都不看（环境变量 GPU_CARDS_BY_PRIORITY 只点名了另一台的卡）；另一台卡 0' "$calls.out"; then
    selftest_cell_failed peer-only-cards "这一趟只用另一台的卡，本机卡 0 上的外人挡住了回读（退 $cell_exit，清场那一行「$(grep '^GPU_QUIESCE step=quiesce ' "$calls.out")」，回读看到的「$(head -n 1 "$work/peer-only.seen" 2>/dev/null)」「$(head -n 1 "$work/peer-only.seen.peer" 2>/dev/null)」）" \
      "看 gpu_quiesce_choose_cards：每一项写法都对、只点了另一台的卡时 GPU_QUIESCE_CHECK_LOCAL_CARDS=none，另一台那几项的卡号经 GPU_QUIESCE_CHECK_PEER_CARDS 交给回读"
  fi
  # 格 local-only-cards：这一趟只有「127.0.0.1:1:8G」⇒ 本机只看卡 1，另一台一张都不看（GPU_QUIESCE_CHECK_PEER_CARDS=none），照跑、退 0
  cells=$((cells + 1)); calls="$work/local-only.calls"; : >"$calls"
  run_wrapper "$calls" "$work/state-local-only" GPU_CARDS_BY_PRIORITY='127.0.0.1:1:8G' FAKE_GPU_PROCESSES="$work/gpu-card0.list" FAKE_CHECK_SEEN="$work/local-only.seen" -- bash "$work/command.sh" ok; cell_exit=$?
  if [[ "$cell_exit" != 0 || "$(head -n 1 "$work/local-only.seen" 2>/dev/null)" != "cards=1 exempt=unset" || "$(head -n 1 "$work/local-only.seen.peer" 2>/dev/null)" != "peer=none" ]]; then
    selftest_cell_failed local-only-cards "这一趟只用本机卡 1，回读没拿到 GPU_QUIESCE_CHECK_PEER_CARDS=none（退 $cell_exit，回读看到的「$(head -n 1 "$work/local-only.seen" 2>/dev/null)」「$(head -n 1 "$work/local-only.seen.peer" 2>/dev/null)」）" \
      "看 gpu_quiesce_choose_cards 末尾另一台那一段：每一项写法都对而没点另一台的卡时写 none"
  fi
  touch "$work/outsider-release"
  wait "$outsider_pid"

  # 格 this-host-by-address：主机数组「127.0.0.1 2001:db8::7」、这一台网卡上的地址换成 2001:db8::7（CONFIGURATION_SELFTEST_LOCAL_ADDRESSES，多机配置判法只在自证里认），
  #   显卡先后「127.0.0.1:0:8G 2001:db8::7:1:8G」⇒ 这一台是 IPv6 那一项（不看 HOSTS 次序）：回读本机只看卡 1、另一台只看卡 0，照跑、退 0
  cells=$((cells + 1)); calls="$work/this-host.calls"; : >"$calls"
  run_wrapper "$calls" "$work/state-this-host" SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-ipv6.env" CONFIGURATION_SELFTEST_LOCAL_ADDRESSES=2001:db8::7 \
    GPU_CARDS_BY_PRIORITY='127.0.0.1:0:8G 2001:db8::7:1:8G' FAKE_GPU_PROCESSES="$work/gpu-card0.list" FAKE_CHECK_SEEN="$work/this-host.seen" -- bash "$work/command.sh" ok; cell_exit=$?
  if [[ "$cell_exit" != 0 || "$(head -n 1 "$work/this-host.seen" 2>/dev/null)" != "cards=1 exempt=unset" || "$(head -n 1 "$work/this-host.seen.peer" 2>/dev/null)" != "peer=0" ]] \
     || ! grep -q '回读看的卡：本机卡 1（环境变量 GPU_CARDS_BY_PRIORITY）；另一台卡 0' "$calls.out"; then
    selftest_cell_failed this-host-by-address "这一台是 2001:db8::7、显卡先后「127.0.0.1:0:8G 2001:db8::7:1:8G」，回读应当本机只看卡 1、另一台只看卡 0（退 $cell_exit，回读看到的「$(head -n 1 "$work/this-host.seen" 2>/dev/null)」「$(head -n 1 "$work/this-host.seen.peer" 2>/dev/null)」，那一句「$(grep '回读看的卡' "$calls.out")」）" \
      "看 gpu_quiesce_choose_cards 与共用库 crash_amplification_host_addresses：这一台照多机配置判法 --emit-addresses 认，从右切两刀取地址"
  fi
  # 格 card-address-outside-hosts-refused：显卡先后「192.0.2.99:0:8G」的地址不在 HOSTS 里 ⇒ 清场命令一条都没跑、命令没起、退 78，点名那个地址
  cells=$((cells + 1)); calls="$work/outside-hosts.calls"; : >"$calls"
  run_wrapper "$calls" "$work/state-outside-hosts" GPU_CARDS_BY_PRIORITY='192.0.2.99:0:8G' -- bash "$work/command.sh" ok; cell_exit=$?
  if [[ "$cell_exit" != 78 || -s "$calls" ]] || ! grep -q '的地址「192.0.2.99」不在 HOSTS' "$calls.out" || ! grep -q '^GPU_QUIESCE step=command result=not-started$' "$calls.out"; then
    selftest_cell_failed card-address-outside-hosts-refused "显卡先后的地址 192.0.2.99 不在 HOSTS 里，应当不清场、命令不起、退 78（退 $cell_exit，次序「$(calls_of "$calls")」，输出 $(tr '\n' ' ' <"$calls.out" | cut -c1-200)）" \
      "看共用库 crash_amplification_card_list_problem：从右切两刀之后地址要是 HOSTS 里的一项"
  fi
  # 格 origin-read-back-quiesce：起活那一方清过场（GPU_QUIESCE_ORIGIN_READ_BACK=1）⇒ 清场命令一条都没跑、照起命令、退 0，清场那一行是 quiesced-on-origin
  cells=$((cells + 1)); calls="$work/origin.calls"; : >"$calls"
  run_wrapper "$calls" "$work/state-origin" GPU_QUIESCE_ORIGIN_READ_BACK=1 -- bash "$work/command.sh" ok; cell_exit=$?
  if [[ "$cell_exit" != 0 || "$(calls_of "$calls")" != "command-ok-start holder-of-parent command-ok-end " ]] \
     || ! grep -q '^GPU_QUIESCE step=quiesce result=quiesced-on-origin holders=0$' "$calls.out"; then
    selftest_cell_failed origin-read-back-quiesce "起活那一方清过场，这一台却跑了配置的清场命令、没照起命令或清场那一行不是 quiesced-on-origin（退 $cell_exit，次序「$(calls_of "$calls")」，清场那一行「$(grep '^GPU_QUIESCE step=quiesce ' "$calls.out")」）" \
      "看进来那一段的 gpu_quiesce_on_origin：GPU_QUIESCE_ORIGIN_READ_BACK=1 时不调 gpu_quiesce_run_configured，就算回读过了"
  fi
  # 查卡那几格（文件头「查卡」）：假的查卡程序记一笔「card-health」；FAKE_CARD_HEALTH 选 ok（都好）、fault（第二台 0 号卡坏）、unchecked（第二台连不上）
  cat >"$work/card-health.sh" <<'STUB'
#!/usr/bin/env bash
echo "card-health" >> "${FAKE_CALLS:?}"
case "${FAKE_CARD_HEALTH:-ok}" in
  ok) echo "GPU_CARD_HEALTH host=local card=0 bus=00000000:01:00.0 state=ok evidence=temperature=30,power=17.66,recovery-action=None counted=none"
      echo "GPU_CARD_HEALTH_SUMMARY state=ok checked=1 fault=0 unchecked=none"; echo "  ✓ 卡状态：查了 1 张点名的卡，都好"; exit 0 ;;
  fault) echo "GPU_CARD_HEALTH host=local card=0 bus=00000000:01:00.0 state=ok evidence=temperature=30,power=17.66,recovery-action=None counted=none"
         echo "GPU_CARD_HEALTH host=peer card=0 bus=00000000:01:00.0 state=fault evidence=temperature=ERR!,xid-62=1 counted=xid-109:4549"
         echo "GPU_CARD_HEALTH_SUMMARY state=fault checked=2 fault=1 unchecked=none"
         echo "  ✗ 显卡状态不对：第二台 0 号卡（总线 00000000:01:00.0）：温度读成 ERR!；这次开机后内核报的致命类 Xid 62=1"
         echo "  → 怎么办：主 agent 弹窗告诉用户哪台哪张卡与依据（上面逐卡那几行），请用户在那台用 root 重置：第二台 sudo nvidia-smi -r -i 0；重置不了就重启那台，重启后先跑多机配置 QUIESCE_START_COMMAND 那条私有复原脚本再清场"
         exit 1 ;;
  unchecked) echo "GPU_CARD_HEALTH_SUMMARY state=unchecked checked=0 fault=0 unchecked=第二台:连不上"
             echo "  ✗ 卡状态查不成（第二台）：第二台连不上或没回读数（退 255）"; echo "  → 怎么办：第二台连不上就先修 ssh 免密登录"; exit 78 ;;
esac
STUB
  chmod +x "$work/card-health.sh"
  # 格 card-fault：坏卡 ⇒ 清场命令一条都没跑、命令没起、退 78；清场那一行是 card-fault 带坏卡的依据，标准错误转打逐卡那一句与弹窗出路
  cells=$((cells + 1)); calls="$work/card-fault.calls"; : >"$calls"
  run_wrapper "$calls" "$work/state-card-fault" GPU_QUIESCE_CARD_HEALTH_PROGRAM="$work/card-health.sh" FAKE_CARD_HEALTH=fault -- bash "$work/command.sh" ok; cell_exit=$?
  if [[ "$cell_exit" != 78 || "$(calls_of "$calls")" != "card-health " ]] \
     || ! grep -qxF 'GPU_QUIESCE step=quiesce result=card-fault faulty-cards=peer:0 cards=peer/0/00000000:01:00.0/temperature=ERR!,xid-62=1' "$calls.out" \
     || ! grep -q '^GPU_QUIESCE step=command result=not-started$' "$calls.out" || ! grep -q '显卡状态不对：第二台 0 号卡' "$calls.out" \
     || ! grep -q '→ 怎么办：主 agent 弹窗.*第二台 sudo nvidia-smi -r -i 0.*QUIESCE_START_COMMAND' "$calls.out"; then
    selftest_cell_failed card-fault "坏卡那一趟应当不清场、命令不起、退 78，清场那一行带 card-fault 与依据、转打弹窗出路（退 $cell_exit，次序「$(calls_of "$calls")」，清场那一行「$(grep '^GPU_QUIESCE step=quiesce ' "$calls.out")」）" \
      "看 gpu_quiesce_card_health：查卡脚本退 1 且汇总是 state=fault 就打 card-fault、交回 78，主流程不拿锁、不清场"
  fi
  # 格 card-ok：卡都好 ⇒ 先查卡、再照旧清场、回读、命令，退 0
  cells=$((cells + 1)); calls="$work/card-ok.calls"; : >"$calls"
  run_wrapper "$calls" "$work/state-card-ok" GPU_QUIESCE_CARD_HEALTH_PROGRAM="$work/card-health.sh" FAKE_CARD_HEALTH=ok -- bash "$work/command.sh" ok; cell_exit=$?
  if [[ "$cell_exit" != 0 || "$(calls_of "$calls")" != "card-health stop stopped-check command-ok-start holder-of-parent command-ok-end " ]] \
     || ! grep -q '^GPU_QUIESCE step=quiesce result=stopped-and-read-back holders=0$' "$calls.out" || ! grep -q '查卡：多机配置点名的卡都好' "$calls.out"; then
    selftest_cell_failed card-ok "卡都好那一趟应当先查卡、再照旧清场回读起命令、退 0（退 $cell_exit，次序「$(calls_of "$calls")」）" \
      "看主流程：gpu_quiesce_card_health 在拿锁之前调、退 0 照往下走"
  fi
  # 格 card-health-unchecked：查卡查不成（退 78）⇒ 不清场、命令不起、退 78，清场那一行是 card-health-unchecked，报「卡状态查不成」
  cells=$((cells + 1)); calls="$work/card-unchecked.calls"; : >"$calls"
  run_wrapper "$calls" "$work/state-card-unchecked" GPU_QUIESCE_CARD_HEALTH_PROGRAM="$work/card-health.sh" FAKE_CARD_HEALTH=unchecked -- bash "$work/command.sh" ok; cell_exit=$?
  if [[ "$cell_exit" != 78 || "$(calls_of "$calls")" != "card-health " ]] \
     || ! grep -qx 'GPU_QUIESCE step=quiesce result=card-health-unchecked exit=78' "$calls.out" || ! grep -q '卡状态查不成（scripts/hosts/gpu-card-health.py 退 78）' "$calls.out" \
     || ! grep -q '^GPU_QUIESCE step=command result=not-started$' "$calls.out"; then
    selftest_cell_failed card-health-unchecked "查卡查不成那一趟应当不清场、命令不起、退 78、报卡状态查不成（退 $cell_exit，次序「$(calls_of "$calls")」，清场那一行「$(grep '^GPU_QUIESCE step=quiesce ' "$calls.out")」）" \
      "看 gpu_quiesce_card_health：退 0、77 之外（退 1 而汇总不是 fault 也算）打 card-health-unchecked、交回 78"
  fi
  # 格 card-health-origin-read-back：起活那一方清过场（GPU_QUIESCE_ORIGIN_READ_BACK=1）照样先查卡，卡都好照起命令
  cells=$((cells + 1)); calls="$work/card-origin.calls"; : >"$calls"
  run_wrapper "$calls" "$work/state-card-origin" GPU_QUIESCE_ORIGIN_READ_BACK=1 GPU_QUIESCE_CARD_HEALTH_PROGRAM="$work/card-health.sh" \
    FAKE_CARD_HEALTH=ok -- bash "$work/command.sh" ok; cell_exit=$?
  if [[ "$cell_exit" != 0 || "$(head -n 1 "$calls")" != "card-health" ]] || ! grep -qx 'command-ok-end' "$calls"; then
    selftest_cell_failed card-health-origin-read-back "起活那一方清过场时也应当先查卡、卡都好照起命令（退 $cell_exit，次序「$(calls_of "$calls")」）" \
      "看主流程：gpu_quiesce_card_health 在拿锁之前调，GPU_QUIESCE_ORIGIN_READ_BACK=1 时照调"
  fi
  # 格 assigned-card-health 的假查卡程序：不带 --cards 时演查到两张卡（本机 127.0.0.1 的 0 号好、另一台 192.0.2.1 的 0 号坏），带了只查点名的那几张；
  #   记一笔「card-health」进 FAKE_CALLS，交来的参数另记进 $FAKE_CALLS.health-arguments
  cat >"$work/card-health-assigned.sh" <<'STUB'
#!/usr/bin/env bash
echo "card-health" >> "${FAKE_CALLS:?}"
echo "$*" >> "$FAKE_CALLS.health-arguments"
named="127.0.0.1:0:8G 192.0.2.1:0:8G"
[[ "${1:-}" == --cards ]] && named="${2:-}"
checked=0; fault=0
for entry in $named; do
  address_index="${entry%:*}"; checked=$((checked + 1))
  if [[ "$address_index" == 192.0.2.1:0 ]]; then
    echo "GPU_CARD_HEALTH host=${address_index%:*} card=${address_index##*:} bus=00000000:01:00.0 state=fault evidence=recovery-action=Reset,xid-62=1 counted=none"
    fault=$((fault + 1))
  else
    echo "GPU_CARD_HEALTH host=${address_index%:*} card=${address_index##*:} bus=00000000:01:00.0 state=ok evidence=temperature=30,power=17.66,recovery-action=None counted=none"
  fi
done
if (( fault > 0 )); then
  echo "GPU_CARD_HEALTH_SUMMARY state=fault checked=$checked fault=$fault unchecked=none"
  echo "  ✗ 显卡状态不对：第二台 0 号卡（总线 00000000:01:00.0）：Recovery Action 是 Reset"
  echo "  → 怎么办：主 agent 弹窗告诉用户（假的出路）"
  exit 1
fi
echo "GPU_CARD_HEALTH_SUMMARY state=ok checked=$checked fault=0 unchecked=none"
STUB
  chmod +x "$work/card-health-assigned.sh"
  # 格 assigned-card-health：查到的两张卡里另一台 0 号坏。分到本机 0 号好卡（GPU_CARDS_BY_PRIORITY='127.0.0.1:0:8G'）⇒ 查卡拿到的参数只有它、
  #   不判 card-fault、照常清场起命令、退 0；分到坏的那一张（'192.0.2.1:0:8G'）⇒ 不清场、命令不起、退 78，清场那一行 faulty-cards= 点名它
  cells=$((cells + 1))
  local good_calls="$work/assigned-good.calls" bad_calls="$work/assigned-bad.calls" good_exit bad_exit
  : >"$good_calls"; : >"$bad_calls"
  run_wrapper "$good_calls" "$work/state-assigned-good" GPU_CARDS_BY_PRIORITY='127.0.0.1:0:8G' GPU_QUIESCE_CARD_HEALTH_PROGRAM="$work/card-health-assigned.sh" \
    -- bash "$work/command.sh" ok; good_exit=$?
  run_wrapper "$bad_calls" "$work/state-assigned-bad" GPU_CARDS_BY_PRIORITY='192.0.2.1:0:8G' GPU_QUIESCE_CARD_HEALTH_PROGRAM="$work/card-health-assigned.sh" \
    -- bash "$work/command.sh" ok; bad_exit=$?
  if [[ "$good_exit" != 0 || "$(calls_of "$good_calls")" != "card-health stop stopped-check command-ok-start holder-of-parent command-ok-end " ]] \
     || [[ "$(cat "$good_calls.health-arguments" 2>/dev/null)" != "--cards 127.0.0.1:0:8G" ]] || grep -q 'result=card-fault' "$good_calls.out"; then
    selftest_cell_failed assigned-card-health "分到好卡（本机 0 号）那一趟应当只查它、不判 card-fault、照常起命令退 0（退 $good_exit，次序「$(calls_of "$good_calls")」，查卡拿到的参数「$(cat "$good_calls.health-arguments" 2>/dev/null)」，清场那一行「$(grep '^GPU_QUIESCE step=quiesce ' "$good_calls.out")」）" \
      "看 gpu_quiesce_card_health：gpu_quiesce_choose_cards 取到的显卡先后原样交给查卡程序 --cards，没分到的卡不查"
  fi
  if [[ "$bad_exit" != 78 || "$(calls_of "$bad_calls")" != "card-health " ]] \
     || ! grep -qxF 'GPU_QUIESCE step=quiesce result=card-fault faulty-cards=192.0.2.1:0 cards=192.0.2.1/0/00000000:01:00.0/recovery-action=Reset,xid-62=1' "$bad_calls.out"; then
    selftest_cell_failed assigned-card-health "分到坏卡（另一台 0 号）那一趟应当不清场、命令不起、退 78，清场那一行 faulty-cards= 点名它（退 $bad_exit，次序「$(calls_of "$bad_calls")」，清场那一行「$(grep '^GPU_QUIESCE step=quiesce ' "$bad_calls.out")」）" \
      "看 gpu_quiesce_card_health：坏卡那几行的台与序号拼进清场那一行的 faulty-cards="
  fi

  if (( failures > 0 )); then
    echo "  ✗ --selftest：$cells 格里 $failures 格判错" >&2
    echo "     → 怎么办：照上面每一格给的出路改，再跑 $0 --selftest" >&2
    exit 1
  fi
  if (( nested )); then echo "  ✓ --selftest（开关轮里起的一份）：$cells 格都对"; exit 0; fi
  # 开关轮：文件头「弄坏开关」那张表现读，表里的开关与代码里认的开关要相等，逐个打开要红在点名的那一格
  local table assignment named round_failures=0 in_code in_table
  switch_wheel_assignments=(); switch_wheel_expected=()
  table="$(crash_amplification_read_switch_table "$0" GPU_QUIESCE_BREAK)"
  while IFS=$'\t' read -r assignment named; do
    [[ -n "$assignment" ]] || continue
    switch_wheel_assignments+=("$assignment"); switch_wheel_expected+=("$named")
  done <<<"$table"
  in_code="$(grep -oE 'gpu_quiesce_break_is [a-z0-9-]+' "$0" | awk '{ print $2 }' | LC_ALL=C sort -u)"
  in_table="$(printf '%s\n' ${switch_wheel_assignments[@]+"${switch_wheel_assignments[@]#*=}"} | LC_ALL=C sort -u)"
  if [[ -z "$table" || "$in_code" != "$in_table" ]]; then
    echo "  ✗ 开关轮：代码里认的开关与文件头表里列的不一样（代码里：$(tr '\n' ' ' <<<"$in_code")；表里：$(tr '\n' ' ' <<<"$in_table")）" >&2
    echo "     → 怎么办：代码里每一处 gpu_quiesce_break_is 认的开关都在文件头「弄坏开关」表里写一行，表里每一行都在代码里有对应的一处" >&2
    round_failures=$((round_failures + 1))
  fi
  crash_amplification_switch_wheel "$work" '✗ 自证 ([a-z-]+)：' 0 -- bash "$0" --selftest
  round_failures=$((round_failures + switch_wheel_failures))
  if (( round_failures > 0 )); then
    echo "  ✗ --selftest：$cells 格都对，开关轮有 $round_failures 处对不上" >&2
    echo "     → 怎么办：红的格与表里写的不同，要么改那一处让开关红在点名的格，要么照实改文件头那一行；单看一个开关：GPU_QUIESCE_BREAK=<开关> $0 --selftest" >&2
    exit 1
  fi
  echo "  ✓ --selftest：清场回读命令的次序、没写全清场命令退 78、清场回读不过不起命令、清场命令退非 0 而回读过了那一行带 stop-exit、回读没过或清场命令退非 0 时在工作根下留 600 的本机日志且输出不上标准错误、命令失败交回退出码、被信号打断时把信号（TERM、INT）转给命令的进程组、限时之后补 KILL、退 143 / 130，这一份被 KILL 时守卫停命令的进程组，命令退出后守卫不留、两趟重叠只清一次、死了的登记不挡、跑完不调复原命令且报 not-restored（没写复原那一对、接着上一趟再起都照跑）、用法错退 2、没有服务给的准入标记退 2、被服务起的命令里面再套一层清场里层退 2、前一趟登记下的进程不挡后一趟回读而外人照挡、回读只看这一趟要用的本机卡（GPU_CARDS_BY_PRIORITY，没设取多机配置清场之后那一份；调用方带进来的回读变量不认；只点了一台的卡时另一台一张都不看、另一台只看地址是它的那几张）、这一台照多机配置判法 --emit-addresses 认（主机数组里 IPv6 那一项是这一台时取它的卡）、显卡先后的地址不在 HOSTS 里不清场不起命令退 78、起活那一方清过场时不再跑配置的清场与回读、清场之前查卡（坏卡不清场不起命令退 78 且清场那一行带 card-fault 与依据、查不成同样退 78、卡好照跑、起活那一方清过场时照查；只查这一次分到的卡：没分到的坏卡不挡、分到的坏卡在清场那一行 faulty-cards= 点名），$cells 格都对；开关轮 ${#switch_wheel_assignments[@]} 个开关（文件头现读）逐个打开，收回 $switch_wheel_collected 个，各红在点名的那一格"
  exit 0
}

# 弄坏开关只在自证里认（文件头「弄坏开关」）：命令行第一个参数不是 --selftest、又不是自证起的子进程而 GPU_QUIESCE_BREAK 非空（不管值是什么），
# 在参数解析之前退 2、一条清场命令也不起。自证起的子进程有两种：这份脚本自己的自证（GPU_QUIESCE_SELFTEST_MARK 指到 run_selftest 建的标记文件）、
# crash-amplification.sh 的自证经这一份清场重起自己时带过来的它的标记（CRASH_AMPLIFICATION_SELFTEST_MARK，内容是它的 SELFTEST_MARK_CONTENT 那一行）。
gpu_quiesce_under_selftest=0
if [[ -n "${GPU_QUIESCE_SELFTEST_MARK:-}" && -f "${GPU_QUIESCE_SELFTEST_MARK:-}" ]] \
   && [[ "$(cat -- "$GPU_QUIESCE_SELFTEST_MARK")" == "$gpu_quiesce_selftest_mark_content" ]]; then
  gpu_quiesce_under_selftest=1
elif [[ -n "${CRASH_AMPLIFICATION_SELFTEST_MARK:-}" && -f "${CRASH_AMPLIFICATION_SELFTEST_MARK:-}" ]] \
   && [[ "$(cat -- "$CRASH_AMPLIFICATION_SELFTEST_MARK")" == "crash-amplification.sh --selftest 起的子进程" ]]; then
  gpu_quiesce_under_selftest=1
fi
if [[ -n "${GPU_QUIESCE_BREAK:-}" && "${1:-}" != --selftest ]] && (( ! gpu_quiesce_under_selftest )); then
  echo "  ✗ GPU_QUIESCE_BREAK 只在自证里认（弄坏开关只给 --selftest 证红用），这一趟不是自证；清场命令一条没起" >&2
  echo "  → 怎么办：去掉 GPU_QUIESCE_BREAK 再跑；要证红就跑 GPU_QUIESCE_BREAK=<项> bash $0 --selftest" >&2
  exit 2
fi

case "${1:-}" in
  --selftest) (($# == 1)) || usage; run_selftest ;;
  --) shift; (($# > 0)) || usage ;;
  *) usage ;;
esac

# 只认服务直接起的那一层（文件头）：没有服务给的准入标记就拒、退 2，清场命令一条不起；有就 unset，命令与清场命令都看不到它（弄坏开关 admission-mark-kept 下照留）
gpu_quiesce_admitted_by_task_server="${GPU_QUIESCE_RUN_ADMITTED_BY_TASK_SERVER:-}"
gpu_quiesce_break_is admission-mark-kept || unset GPU_QUIESCE_RUN_ADMITTED_BY_TASK_SERVER
if [[ -z "$gpu_quiesce_admitted_by_task_server" ]]; then
  echo "  ✗ gpu-quiesce-run.sh 只认 task-server 起尝试时直接套的那一层，环境里没有服务给的 GPU_QUIESCE_RUN_ADMITTED_BY_TASK_SERVER（任务里面又套了一层，或不经服务直接起）；清场命令一条没起，命令没跑" >&2
  echo "  → 怎么办：任务里面不再套包装：内存、线程、机器、卡由 task-server 定；删掉这一层（用哪张卡照读服务交的 GPU_CARDS_BY_PRIORITY），整条经 task-server/task submit -- <命令…> 交，服务认出要卡就在最外层套清场" >&2
  exit 2
fi

# 回读的三个握手变量只由这一份置、只给清场那两条命令：调用方带进来的清掉，不认、也不传给 <命令…>
gpu_quiesce_break_is keep-inherited-check-variables || unset GPU_QUIESCE_CHECK_LOCAL_CARDS GPU_QUIESCE_CHECK_PEER_CARDS GPU_QUIESCE_CHECK_EXEMPT_PIDS
gpu_quiesce_check_environment=()

# 读配置里清场那两个键：缺一个就不清场、不起命令，退 78
gpu_quiesce_configuration="$(gpu_quiesce_configuration_file)"
declare -A gpu_quiesce_commands=()
gpu_quiesce_missing=()
for gpu_quiesce_key in "${GPU_QUIESCE_KEYS[@]}"; do
  gpu_quiesce_commands[$gpu_quiesce_key]="$(crash_amplification_configuration_value "$gpu_quiesce_configuration" "$gpu_quiesce_key")"
  [[ -n "${gpu_quiesce_commands[$gpu_quiesce_key]}" ]] || gpu_quiesce_missing+=("$gpu_quiesce_key")
done
if (( ${#gpu_quiesce_missing[@]} > 0 )); then
  if gpu_quiesce_break_is accept-missing-quiesce-commands; then
    "$@"; exit $?
  fi
  gpu_quiesce_line "step=command result=not-started"
  gpu_quiesce_line "step=exit code=78"
  echo "  ✗ 多机配置 $gpu_quiesce_configuration 里没写 ${gpu_quiesce_missing[*]}（文件不在也算）：清不了场，命令没起" >&2
  echo "  → 怎么办：照仓根 multi-host.env.example 写全 QUIESCE_STOP_COMMAND 与 QUIESCE_STOPPED_CHECK_COMMAND（清场、清场回读；复原那一对这里不读）；配置在别处的设 SINGLEFS_MULTI_HOST_CONFIG 指到它（退 78）" >&2
  exit 78
fi
# 回读看的卡（文件头「回读看哪几张卡、跳过哪些进程」）：显卡先后写坏或认不出这一台就不清场、命令不起，退 78
if ! gpu_quiesce_choose_cards; then
  gpu_quiesce_line "step=command result=not-started"
  gpu_quiesce_line "step=exit code=78"
  echo "  ✗ $gpu_quiesce_cards_problem：认不出这一趟要用这一台的哪几张卡，没清场，命令没起" >&2
  echo "  → 怎么办：改显卡先后（task-server 交的 GPU_CARDS_BY_PRIORITY，或多机配置 $gpu_quiesce_configuration 的 GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED）：$CRASH_AMPLIFICATION_CARD_ENTRY_HOW_TO_WRITE；多机配置判不过的照上面那一句改（退 78）" >&2
  exit 78
fi

gpu_quiesce_state_directory="${GPU_QUIESCE_STATE_DIRECTORY:-${XDG_RUNTIME_DIR:-/tmp}/singlefs-gpu-quiesce}"
if ! mkdir -p -- "$gpu_quiesce_state_directory/holders"; then
  gpu_quiesce_line "step=command result=not-started"
  gpu_quiesce_line "step=exit code=78"
  echo "  ✗ 建不了清场的状态目录 $gpu_quiesce_state_directory/holders：命令没起" >&2
  echo "  → 怎么办：看 XDG_RUNTIME_DIR 可不可写，或设 GPU_QUIESCE_STATE_DIRECTORY 指到可写的目录；几趟同时用卡的要指同一个（退 78）" >&2
  exit 78
fi
gpu_quiesce_holder_name="$$-$(gpu_quiesce_process_start_time "$$")"
# 本机日志：清场那两条命令的输出进这里（目录 700、文件 600，日志目录开机不清）；取不到工作根、建不了就不起
gpu_quiesce_log_kept=0; gpu_quiesce_log_file=""
if [[ -n "${GPU_QUIESCE_LOG_DIRECTORY:-}" ]]; then gpu_quiesce_log_directory="$GPU_QUIESCE_LOG_DIRECTORY"
elif gpu_quiesce_break_is failure-log-in-runtime-dir; then gpu_quiesce_log_directory="${XDG_RUNTIME_DIR:-/tmp}/singlefs-gpu-quiesce/logs"
elif gpu_quiesce_work_root="$(python3 "$gpu_quiesce_script_directory/../../.claude/hooks/lib_work_root.py")" && [[ -n "$gpu_quiesce_work_root" ]]; then
  gpu_quiesce_log_directory="$gpu_quiesce_work_root/gpu-quiesce-logs"
else
  gpu_quiesce_line "step=command result=not-started"
  gpu_quiesce_line "step=exit code=78"
  echo "  ✗ 取不到工作根（.claude/hooks/lib_work_root.py 跑不起来），清场的本机日志没处放：命令没起" >&2
  echo "  → 怎么办：恢复那份库，或设 GPU_QUIESCE_LOG_DIRECTORY 指到开机不清的可写目录（退 78）" >&2
  exit 78
fi
gpu_quiesce_log_file="$gpu_quiesce_log_directory/$(TZ=Asia/Tokyo date +%F)-$gpu_quiesce_holder_name.log"
if ! { mkdir -p -m 700 -- "$gpu_quiesce_log_directory" && chmod 700 -- "$gpu_quiesce_log_directory" && ( umask 077 && : >"$gpu_quiesce_log_file" ); }; then
  gpu_quiesce_line "step=command result=not-started"
  gpu_quiesce_line "step=exit code=78"
  echo "  ✗ 建不了清场的本机日志 $gpu_quiesce_log_file：命令没起" >&2
  echo "  → 怎么办：看日志目录 $gpu_quiesce_log_directory 可不可写，或设 GPU_QUIESCE_LOG_DIRECTORY 指到开机不清的可写目录（退 78）" >&2
  exit 78
fi
gpu_quiesce_registered=0; gpu_quiesce_command_started=0; gpu_quiesce_command_exit=0; gpu_quiesce_interrupted=""
gpu_quiesce_command_pid=""; gpu_quiesce_last_signal=""; gpu_quiesce_forwarded=0; gpu_quiesce_limit_pid=""; gpu_quiesce_guard_pid=""
gpu_quiesce_limit_seconds="$(gpu_quiesce_forward_limit)"
gpu_quiesce_guard_poll="${GPU_QUIESCE_GUARD_POLL_SECONDS:-0.5}"
[[ "$gpu_quiesce_guard_poll" =~ ^[0-9]+(\.[0-9]+)?$ ]] || gpu_quiesce_guard_poll=0.5
trap 'gpu_quiesce_on_signal INT 130' INT
trap 'gpu_quiesce_on_signal TERM 143' TERM
trap 'gpu_quiesce_on_signal HUP 129' HUP
trap 'gpu_quiesce_leave; gpu_quiesce_finish_log' EXIT

# 进来：锁里登记、没有别的活着的登记才清场，回读
# 查卡（文件头「查卡」）：拿锁、清场之前；坏卡或查不成就不清场、命令不起、退 78，途中被打断退 128+信号号
gpu_quiesce_card_health; gpu_quiesce_card_health_exit=$?
if (( gpu_quiesce_card_health_exit != 0 )); then
  gpu_quiesce_line "step=command result=not-started"
  gpu_quiesce_line "step=exit code=$gpu_quiesce_card_health_exit"
  exit "$gpu_quiesce_card_health_exit"
fi
if ! gpu_quiesce_enter_lock; then
  gpu_quiesce_line "step=command result=not-started"
  gpu_quiesce_line "step=exit code=78"
  echo "  ✗ ${GPU_QUIESCE_LOCK_SECONDS:-900} 秒没拿到清场锁 $gpu_quiesce_state_directory/lock：命令没起" >&2
  echo "  → 怎么办：别的经 gpu-quiesce-run.sh 的那一趟卡在清场命令上；等它结束再起（退 78）" >&2
  exit 78
fi
gpu_quiesce_others="$(gpu_quiesce_other_live_holders)"
: >"$gpu_quiesce_state_directory/holders/$gpu_quiesce_holder_name"
gpu_quiesce_registered=1
# 回读看的卡（进来之前已选好）与跳过的进程号（文件头「回读看哪几张卡、跳过哪些进程」）：清场那一支与只回读那一支都算，在锁里、清场命令起之前
[[ -n "$gpu_quiesce_local_cards" ]] && gpu_quiesce_check_environment+=("GPU_QUIESCE_CHECK_LOCAL_CARDS=$gpu_quiesce_local_cards")
[[ -n "$gpu_quiesce_peer_cards" ]] && gpu_quiesce_check_environment+=("GPU_QUIESCE_CHECK_PEER_CARDS=$gpu_quiesce_peer_cards")
gpu_quiesce_exempt_pids=""
gpu_quiesce_break_is no-holder-descendants || gpu_quiesce_exempt_pids="$(gpu_quiesce_holder_descendants)"
[[ -n "$gpu_quiesce_exempt_pids" ]] && gpu_quiesce_check_environment+=("GPU_QUIESCE_CHECK_EXEMPT_PIDS=$gpu_quiesce_exempt_pids")
gpu_quiesce_exempt_count=0
[[ -n "$gpu_quiesce_exempt_pids" ]] && gpu_quiesce_exempt_count="$(wc -w <<<"$gpu_quiesce_exempt_pids")"
# 起活那一方清过场（文件头「两台对称」）：给了 GPU_QUIESCE_ORIGIN_READ_BACK=1 就不再跑配置的清场与回读。弄坏开关 origin-read-back-ignored 下照样在这一台跑
gpu_quiesce_on_origin=0
if [[ "${GPU_QUIESCE_ORIGIN_READ_BACK:-}" == 1 ]] && ! gpu_quiesce_break_is origin-read-back-ignored; then gpu_quiesce_on_origin=1; fi
if (( gpu_quiesce_on_origin )); then
  echo "  · 回读看的卡：起活那一方已照多机配置清场并回读（两台都管），这里不跑；这一趟用的是$gpu_quiesce_cards_said" >&2
else
  echo "  · 回读看的卡：$gpu_quiesce_cards_said；跳过别的活着的登记下的 $gpu_quiesce_exempt_count 个进程" >&2
fi
gpu_quiesce_stopped_here=0
gpu_quiesce_stop_exit=0
gpu_quiesce_read_back=0
if (( gpu_quiesce_on_origin )); then
  gpu_quiesce_read_back=1
else
  if (( gpu_quiesce_others == 0 )); then gpu_quiesce_run_configured QUIESCE_STOP_COMMAND; gpu_quiesce_stop_exit=$?; gpu_quiesce_stopped_here=1; fi
  gpu_quiesce_run_configured QUIESCE_STOPPED_CHECK_COMMAND && gpu_quiesce_read_back=1
  # 跟着别的登记用卡而回读没过（另一台重启过、清场停下的服务被开机拉起这类：登记还在，场已经不清了）：自己再清一次场、再回读一次。
  # 别的登记下的进程照旧经 GPU_QUIESCE_CHECK_EXEMPT_PIDS 跳过、不停；清场命令只停配置里那几个服务。弄坏开关 joined-stale-quiesce-trusted 下不再清
  if (( ! gpu_quiesce_read_back && gpu_quiesce_others > 0 )) && ! gpu_quiesce_break_is joined-stale-quiesce-trusted; then
    echo "  · 已有 $gpu_quiesce_others 趟清过场在用卡，回读却没过（另一台重启过、清场停下的服务又起来了这类）：再清一次场" >&2
    gpu_quiesce_run_configured QUIESCE_STOP_COMMAND; gpu_quiesce_stop_exit=$?; gpu_quiesce_stopped_here=1
    gpu_quiesce_run_configured QUIESCE_STOPPED_CHECK_COMMAND && gpu_quiesce_read_back=1
  fi
fi
gpu_quiesce_leave_lock
gpu_quiesce_quiesce_failed=0
gpu_quiesce_stop_field=""
if (( gpu_quiesce_stopped_here )) && { (( ! gpu_quiesce_read_back )) || (( gpu_quiesce_stop_exit != 0 )); }; then
  gpu_quiesce_keep_log
  gpu_quiesce_break_is drop-stop-exit || gpu_quiesce_stop_field=" stop-exit=$gpu_quiesce_stop_exit"
  gpu_quiesce_stop_field+="$gpu_quiesce_log_field"
fi
if (( gpu_quiesce_on_origin )); then
  gpu_quiesce_line "step=quiesce result=quiesced-on-origin holders=$gpu_quiesce_others"
  echo "  · 照多机配置清了场：起活那一方已清场并回读过（两台都管，GPU_QUIESCE_ORIGIN_READ_BACK=1），这里不再跑" >&2
elif (( gpu_quiesce_read_back )); then
  if (( gpu_quiesce_stopped_here )); then
    gpu_quiesce_line "step=quiesce result=stopped-and-read-back holders=$gpu_quiesce_others$gpu_quiesce_stop_field"
    if (( gpu_quiesce_stop_exit != 0 )); then
      echo "  · 照多机配置清了场，回读过了，照跑；清场命令自己退了 $gpu_quiesce_stop_exit，它报的原因在 log= 那份本机日志里" >&2
    else
      echo "  · 照多机配置清了场，回读过了" >&2
    fi
  else
    gpu_quiesce_line "step=quiesce result=joined-and-read-back holders=$gpu_quiesce_others"
    echo "  · 已有 $gpu_quiesce_others 趟清过场在用卡，回读过了，跟着用" >&2
  fi
else
  if [[ -z "$gpu_quiesce_stop_field" ]]; then gpu_quiesce_keep_log; gpu_quiesce_stop_field="$gpu_quiesce_log_field"; fi
  gpu_quiesce_line "step=quiesce result=read-back-failed holders=$gpu_quiesce_others$gpu_quiesce_stop_field"
  echo "  ✗ 照多机配置清场之后回读没过（QUIESCE_STOPPED_CHECK_COMMAND 退非 0）；清场与回读命令自己报的原因在 log= 那份本机日志里（不打到这里：里面有私有服务名）" >&2
  echo "  → 怎么办：卡上还有不归清场白名单的计算进程（别的会话的 GPU 活）时回读本来就不过：等那个进程跑完再起，不杀它；不是这样的，手动跑配置里的清场命令、回读到它退 0 为止（退 78）" >&2
  gpu_quiesce_break_is run-despite-failed-quiesce || gpu_quiesce_quiesce_failed=1
fi

# 跑：命令起在自己的新会话里（setsid --wait，标准输入输出与环境原样），这一份等它；收到 INT/TERM/HUP 转给它的进程组、限时之后补 KILL，被 KILL 时守卫停它的进程组（文件头「被打断」）
if (( ! gpu_quiesce_quiesce_failed )) && [[ -z "$gpu_quiesce_interrupted" ]]; then
  gpu_quiesce_command_started=1
  gpu_quiesce_start_command "$@"
  gpu_quiesce_guard_start
  gpu_quiesce_wait_command; gpu_quiesce_command_exit=$?
  gpu_quiesce_guard_stop
  gpu_quiesce_disarm_limit
  gpu_quiesce_line "step=command result=exit-$gpu_quiesce_command_exit"
else
  gpu_quiesce_line "step=command result=not-started"
fi

# 出去：删自己的登记，不复原
gpu_quiesce_leave
if (( gpu_quiesce_quiesce_failed )); then gpu_quiesce_final=78
elif [[ -n "$gpu_quiesce_interrupted" ]] && { (( ! gpu_quiesce_command_started )) || (( gpu_quiesce_command_exit == 0 )); }; then gpu_quiesce_final="$gpu_quiesce_interrupted"
else gpu_quiesce_final="$gpu_quiesce_command_exit"; fi
gpu_quiesce_line "step=exit code=$gpu_quiesce_final"
exit "$gpu_quiesce_final"
