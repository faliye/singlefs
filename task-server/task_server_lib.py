#!/usr/bin/env python3
"""singlefs 的统一任务服务（task-server）共用的那一份：路径、依赖、错误类型、放大重试、环境变量名单、编译目录、分片、身份、经 unix socket 调服务，连同整套的自证。

一切跑项目代码的任务（轻、重）都经这个服务起：同一个队、按到达次序、不另设延迟（records/2026-10-30-编译绕过内存总上限.md 的用户定案与形态）。
任务只由主 agent 交（客户端 task-server/task；人手同样用它）；子 agent 只用 status、log 看结局（用户：「现在需要发给 task-server的任务 都不能自己启动」）；挑机器、定内存额度、线程数、分不分片都由服务定，提交时不收这几样；卡归 GPU 批处理验证服务，这个服务不分卡（文件头「卡」）。

这一个目录里的文件：
  task-server/task                   客户端（submit、wait、status、cancel、log、clean；用法与退出码写在它的文件头）
  task-server/control                装与查：install [--dry-run]、check、start、stop、clear-queue、pin-cap、unpin-cap、pause、resume（同上）
  task-server/task_server.py         服务：FastAPI 跑在 uvicorn 上，只开 unix socket；队列、起任务、重试、分片、通知、资源记录都在这一个进程里（判法写在它的文件头）
  task-server/attempt_runner.py      每一次尝试的单元里起命令、记退出码与资源（编译目录不在这里删，下面「编译目录」；服务起它，人不起）
  task-server/singlefs-task-server.service.in   用户级 systemd 单元的模板（control install 照它填）
  task-server/task_server_selftest.py           自证的各格（python3 task-server/task_server_lib.py --selftest 起，单独不起）
  task-server/task_server_lib_constants.py      task_server_lib.py 拆出的一份：路径、阈值、名字、表这些常量
  task-server/task_server_lib_configuration.py  task_server_lib.py 拆出的一份：自证开关的读法（broken）、cargo 与选条判法、多机配置、数据根、NFS 共享、ssh 复用、读 GPU 服务的报告
  task-server/task_server_lib_commands.py       task_server_lib.py 拆出的一份：签名、环境变量与命令词的拆分、套娃派发、对仓内目标的绕过、分片声明与计划、会话号
  task-server/task_server_lib_readings.py       task_server_lib.py 拆出的一份：经 unix socket 调服务、尝试的读数、结局说明与回话的行、异常表、进程表、会话登记
  task-server/task_server_lib_scopes.py         task_server_lib.py 拆出的一份：key-stats、整包测试与未点名跑法的判据、重试决定
                                                这一份 import 它们、再导出同名：别处照旧 import 这一份、取 <名字>；改哪个名字的定义去它所在的那一份
                                                （定义在这一份自己里的只有 SELFTEST_PARALLELISM_TEXT：门禁 selftest-parallelism 只读登记的这一份文件找成功句里的「并行 N 路」）
  task-server/task_server_constants.py、task_server_store.py、task_server_machines.py、task_server_scheduler_<部分>.py（lifecycle、notifying、building、admission、starting）
                                                task_server.py 拆出的各份：常量与小函数、库（Store）、本机与第二台与资源读数、Scheduler 的方法分成五份（Scheduler 类在 task_server.py，继承它们）
  task-server/task_server_selftest_<部分>.py    自证的各格按讲的那一块分成的各份（fakes、world、client、queueing、admission、memory、limits、stalls、notifying、shards、shared_roots、refusals、lifecycle）；
                                                task_server_selftest.py 汇总成 CELLS 并起各格
  task-server/README.md
  task-server 自己没有配置文件：调度资源照仓根 multi-host.env（多机配置，模板 multi-host.env.example）读，路径用下面的固定缺省值。

路径（固定缺省值；多机配置里写了同名的可选键就照键，没写或写空串取缺省）：
  TASK_SERVER_STATE_DIRECTORY     ${XDG_STATE_HOME:-~/.local/state}/singlefs/task-server
                                  库 task-server.sqlite3、logs/（每次尝试的日志）、attempts/（每次尝试的说明与结果）、
                                  alerts.jsonl（告警）、operations.jsonl（增加与取消的操作日志）。不许落在仓里、${XDG_RUNTIME_DIR} 下、tmpfs / ramfs 上（日志要留着，这几处会被清）。
                                  分片的计划与各片的产物不在这里：在共享工作根下的 SHARDS_ROOT_NAME（下面「分片」），两台照同一个绝对路径读写；
                                  判红那一趟的现场同样在共享工作根下的 SCENES_ROOT_NAME（下面「现场」）
  TASK_SERVER_RUNTIME_DIRECTORY   ${XDG_RUNTIME_DIR}/singlefs-task-server：unix socket server.sock 所在目录（700）。socket 的全路径（这个目录接 /server.sock）
                                  不许长于 SOCKET_PATH_MAXIMUM_BYTES（107）字节：Linux 的 unix socket 地址 sun_path 只有 108 字节、留一个结尾的零，长了服务绑不上、
                                  客户端连不上。超了服务、客户端、control 读路径时就退 78，点名那个路径、它有多少字节、上限多少，出路是把这个键写短（socket_path_problem）
  TASK_SERVER_SIGNING_KEY_FILE    ${XDG_CONFIG_HOME:-~/.config}/singlefs/task-server-signing.key：给库里每一行算 HMAC 的密钥（64 位小写十六进制，600，所在目录 700；只有服务读）
  TASK_SERVER_BUILD_DIRECTORY     ${XDG_DATA_HOME:-~/.local/share}/singlefs/task-server-builds：本机各棵树的编译目录组放在它下面，一个一层 tree-<树路径的 sha256 前 8 位>-<随机 6 位十六进制>/target（下面「编译目录」）；不许落在仓里
  值里认的写法只有开头的 ~/ 与 ${HOME}、${XDG_RUNTIME_DIR}、${XDG_STATE_HOME}、${XDG_CONFIG_HOME}、${XDG_DATA_HOME} 五个变量（没设的 XDG 照上面的缺省），展开之后要是绝对路径。
  TASK_SERVER_IO_PRESSURE_LIMIT   不是路径、没有缺省：磁盘压力阈值，大于 0、不超过 100 的数（/proc/pressure/io 的 full avg300 的百分数，两台同一个数）；没写这一行就不按磁盘压力排，
                                  写坏了（不是数、不在范围内）服务、客户端、control 读多机配置时退 78（parse_optional_keys）；怎么用见 task_server.py 文件头 ②「磁盘压力」（io_pressure_problem），读法是 io_pressure_limit
  多机配置在哪：SINGLEFS_MULTI_HOST_CONFIG 给了用它；否则主工作树根（git common-dir 的上一层）的 multi-host.env；不在 git 仓里取这一棵树根上的（与 scripts/crash/crash-amplification-lib.sh
  的 crash_amplification_multi_host_configuration_file 同一个取法）。这里只读 TASK_SERVER_ 起头的那几个键（写了认不出的 TASK_SERVER_ 键判坏），别的键归多机配置的判法
  scripts/crash/crash-point-replay-shard-configuration-check.sh；那份判法认不出的键一律判不过，可选键要等它认了 TASK_SERVER_ 起头的键才写得进去（此刻只写缺省）。
  另读多机配置的 NFS_SHARED_ROOTS、HOSTS 与按台的 SLICE_MEMORY_CAP_BY_HOST（下面「NFS 共享」「主机」与「资源记录」的钉值；读法 configured_value：最后一行算数），
  共享数据根 SHARED_DATA_ROOT 照 .claude/hooks/lib_work_root.py 读（下面「共享数据根」）。
  按台的三个键（CARGO_BIN_DIRECTORY_BY_HOST、BUILD_ROOT_BY_HOST、SLICE_MEMORY_CAP_BY_HOST；写法与意思在仓根 multi-host.env.example）：值是空白分隔的
  <地址>=<值>，每项从第一个 = 切，HOSTS 里每一台恰好一项、每个地址都在 HOSTS 里；切法与这两条判法只在共用取法 scripts/hosts/multi_host_addresses.py 的
  by_host_values（configured_by_host 照它，再按 host_addresses 把地址映到服务记尝试的 local / peer）；少一台判写坏、点名缺的那一台，多一个不在 HOSTS 里的地址判写坏、
  点名那一项。弄坏开关 by-host-missing-accepted、by-host-outside-accepted 下照转给共用取法的同名开关不判那一条。
主机（用户 2026-11-02：「不要设置这种东西 多机就设置 一个数组 把多机的ip写进去」）：多机配置一行 HOSTS=<IP> <IP>（各台有线直连段的地址，空格分隔）；
  第二台的地址只经共用取法 scripts/hosts/multi_host_addresses.py 的 peer_ssh_address 取（peer_ssh_address 照文件路径载入它，不另写一份判法）：它起多机配置判法
  crash-point-replay-shard-configuration-check.sh --emit-addresses --configuration <多机配置>，整份配置判过才给地址（缺键、认不得的键、写坏的行都判不过）；
  哪一台是自己照这一台网卡上的地址认，第二台是数组里不是自己的那一项，ssh 直接连那个地址（用户名靠各台的 ~/.ssh/config）。
  两台对称：不读认台的角色变量；服务照 host_addresses（同一份共用取法的 resolve）认出自己这一台，把 HOSTS 里的地址映到它记尝试的 local / peer
  （分片交给命令的那一台地址、按台的钉值都照它）。
  取不到的（整份判不过、自己认不出、没写 HOSTS）交原因与出路：NFS 共享的「挂载」一项判不过，control install、start、check 判多机配置时退 78。
  服务起的 ssh 一律复用连接（第二台 sshd 的 MaxStartups 被一条条新连接打满过）：起服务时（enable_ssh_multiplexing）控制目录是服务状态目录下的 ssh/（700），
  服务自己起的 ssh（peer_shell_timed）接 ControlMaster=auto、ControlPath=<控制目录>/%C（%C 按本机、对端、端口、用户名分，每台一份）、ControlPersist=SSH_CONTROL_PERSIST_SECONDS
  三项（ssh_multiplex_options）；host-dispatch.py 经 peer-host-lib.sh 起的（余量探测、核进程、进展读数）与服务自己起的 multi-host-run.sh --recover-cleanup、
  --remove-build-directory 照 ssh_multiplexed_environment 在 PATH 前面接控制目录下
  bin/ssh（ssh_shim_text：接上同三项再起 PATH 上下一个 ssh）。主机名不在这一层：照调用方给的，只从多机配置取。控制目录接上 %C 与 ssh 的临时后缀放不下 socket 路径的
  （ssh_control_path_problem）这一次不复用、服务照起。尝试的环境照提交时的白名单另建，不带这一层（主连接留在服务的单元里）。停服务时 close_ssh_masters 逐个关主连接
  （服务单元 KillMode=process，不关的留到 ControlPersist 到点，control stop 回读 cgroup.procs 不空）。弄坏开关 ssh-unmultiplexed 下两处都不接。
共享数据根（.claude/hooks/lib_work_root.py 文件头「数据根」）：环境里的 SINGLEFS_DATA_ROOT，没设取多机配置的 SHARED_DATA_ROOT（绝对路径，落在 NFS_SHARED_ROOTS 某一段下）；
  不补缺省（shared_data_root、work_root）。服务的单元环境里只有 SINGLEFS_MULTI_HOST_CONFIG，起来时取的就是那一行：control install、start、check 判多机配置时只看这一行
  （require_configured_data_root），没写、不是绝对路径、不在共享根下都退 78，出路点名往 multi-host.env 加 SHARED_DATA_ROOT=<数据根的绝对路径>；
  服务取不到同样退 78。弄坏开关 data-root-key-unchecked 下 control 不判这一行。

必须依赖（服务、客户端、control check 开跑先判，缺一样退 78、命令一行不跑，不退回直接执行命令）：
  python 模块 fastapi、uvicorn（本机写法照 pydantic v1）；不在时出路写 sudo apt install python3-fastapi python3-uvicorn（要 sudo，由用户跑）。
  多机配置 multi-host.env 在、TASK_SERVER_ 那几个键读得通；服务通（socket 连得上、/health 回 ok）。服务那一侧另要 systemd-run 与 systemctl 在 PATH 上、签名密钥读得到、
  每台默认重活线程数取得到（.claude/heavy-thread-default）。
  NFS 共享判得过（下面「NFS 共享」）：服务起来之前、control check、control install 与 control start 判，不过退 78。
  多机配置写了 HOSTS（上面「主机」），写了 SHARED_DATA_ROOT（上面「共享数据根」）：control check、install、start 判，服务起来之前取，不过退 78。

NFS 共享（用户：「所有的脚本必须多机跑。抛出的结果需要多机都能读取。如果不行，要么复制资源，要么共享文件。」「没有 nfs 不启动。要求用户配置。
  配置相关来自多机配置的配置文件中 不允许增加额外的配置文件。」；配法在 task-server/README.md「## NFS 共享」）：
  多机配置的 NFS_SHARED_ROOTS=<绝对路径>[:<绝对路径>…]（冒号分隔）：本机导出、第二台挂在同一绝对路径的目录。每个根判四项（nfs_share_report）：
    目录      本机有这个目录；
    导出      NFS_EXPORTS_FILE（/var/lib/nfs/etab）里有一行，第一列逐字等于这个根；
    挂载      经多机配置 HOSTS 里第二台那一项起 ssh（peer_ssh_address、NFS_SSH_OPTIONS）：第二台上 stat -f -c %T -- <根>/. 读到 nfs（nfs_mount_verdict）——走进目录会触发
              x-systemd.automount 挂上它；自动挂载闲着时那里只是一层 autofs 占位，mountpoint -q 照样退 0，不说明 NFS 挂上了，所以不看它。
              不过时出路写在第二台上 systemctl reset-failed 那个挂载单元、ls <根> 触发再挂（问题 84；弄坏开关 nfs-mount-placeholder-accepted 下照改前只看 mountpoint -q）；
              这一次 ssh 超时（NFS_SSH_SECONDS）先当「慢」：进复查期（NFS_SLOW_GRACE_SECONDS 秒），期里每隔 NFS_RECHECK_PAUSE_SECONDS 秒复查一回（peer_shell_rechecked），
              哪一回通了照通算，慢的那一句交给 nfs_share_report 的 slow（服务只记进服务日志与 /health 的 nfs 一栏：不推提醒、不当第二台用不了）；
              复查期满还超时或连不上才算不过，那一句写「连不上第二台（ssh 超时 N 秒，复查期 G 秒里隔 M 秒复查了 K 回仍不通…）」；
              头一次就是连不上（ssh 退 255 而不是超时）的不复查，照写「连不上第二台（ssh 退 255：…）」（第二台 load average 约 60 时一次 30 秒超时、同一刻手跑 stat -f 秒回 nfs；
              弄坏开关 peer-health-single-timeout-reported 下头一次超时就算连不上，peer-recheck-once-only 下复查期里只复查一回）；
    两向探针  本机往 <探针目录>/.task-server-nfs-probe 写一个随机记号、第二台 cat 回来逐字相同；第二台往 <探针目录>/.task-server-nfs-probe-peer 写一个随机记号、本机读回逐字相同；
              读完两个文件都删。探针目录：根下的 .git 是目录（根是 git 工作树）时是 <根>/.git，不在项目根下留条目（仓根新条目钩子会报）；别的根是根本身（nfs_probe_directory）。
  没写这一键、写法不对、哪个根哪一项不过：服务不起（Scheduler 建起之前判，退 78），一句话写明哪个根、哪一项、为什么，出路指 task-server/README.md「## NFS 共享」。
  control check 默认读服务 /health 的 nfs 一栏（写明多久以前判的），--probe 现判四项（每一项开始、结束各打一行，总时长不超过 NFS_LIVE_PROBE_LIMIT_SECONDS 或 --limit，到点打出卡在哪一项、退 1；NfsProbeBudget）；control install 起之前判前三项、回读到 /health 回 ok 之后判四项；control start 起之前判前三项、回读之后判四项；不过都退 78。
  服务起来之后每 NFS_RECHECK_SECONDS（task_server.py）复核前三项（两向探针只在起来时做）：第二台没挂上的根，当第二台此刻用不了（task_server.py 文件头 ②「机器」）、写告警；
  /health 的 nfs 一栏是每个根的四项结论（两向探针取起来时那一次）；服务自己盯健康那一路（task_server.py 文件头 ⑥）每一轮读这一份，前三项不过就发消息给有任务在排、在跑的会话。
  第二台照同一绝对路径读写共享根（仓也在共享根下）：第二台在同一棵树、同一个绝对路径上跑，工作目录与命令里的路径有不在共享根下的提交时就拒（下面「挑机器」）。

库（${状态目录}/task-server.sqlite3，标准库 sqlite3、WAL，文件 600）：只有服务读写它。表 meta、tasks、attempts、history、operations、alerts，
  每一行的正文是一段 JSON（record 列），另有一列 signature = HMAC-SHA256(签名密钥, 表名、行号、state、session、command_key、record 用换行连起来)，小写十六进制。
  服务每次读回都核：核不过的行不起、不认、写告警（kind=record-tampered），任务记成 record-tampered 收尾。会话号只进库与仓外的日志，不进仓。
  决定关联的那一列 task_id（attempts_of、children_of 照它挑尝试与子任务）不在签名里：读的时候与正文的 task_id 比、子任务另比 parent_id，对不上就是核不过（TS-05，
  records/2026-11-01-task-server深入审核.md）；停单元、删编译目录、推分片遇到核不过的行一律不动、回 409 或记 record-tampered。
  记 record-tampered 时重签写回的行自己核得过（关联列照正文写，正文的 task_id 照 parent_id 写，判不出归哪一件的写 0）：同一行不再判出校验不过、不再告警。
  在跑的尝试里核不过的（这一行或它那条任务的行）由对账记 ended、结局 record-tampered（正文只留认得出是哪一次的几样），它的单元由服务停（名字合这个服务的单元名、又不是校验过的在跑尝试的才停，结果报 tampered-unit-stop）；内存、盘与线程的记账
  只取校验过的在跑尝试（task_server.py 的 Store.running_attempts、Scheduler.end_tampered_attempt）。
  在跑尝试读一遍（loop 审核发现 8：一轮里各处要读几百遍）：Store.running_attempts 每个线程照自己那条连接记一份，本进程没写过（write_generation）、别的连接没提交过
    （PRAGMA data_version）就交那一份的拷贝。索引（Store.LOOKUP_INDEXES，启动时建）：attempts(task_id)、tasks(task_id)、attempts(command_key, state, id)、tasks(session, state)，
    按关联列取尝试与子任务、按命令键取结束了的尝试、按会话取任务都走索引，不整表扫。
  状态转换（TS-01、TS-03）：任务行只经 Store.change 改——一个事务里读当前那一行、核、判、只并进要改的那几栏，不拿调用方手里的旧字典整行覆盖；结了局的任务不回到别的状态。
    尝试行只改一两栏的（删完编译目录记「已删」）同样经 Store.change；整行写尝试（Store.update）时库里已是 ended 的不写回别的状态。
  起一次尝试是一个事务（Store.claim_attempt）。业务唯一：attempts 的 (task_id, 正文里的 number)、tasks 的 (正文里的 parent_id, role, shard_index)；
  已有的库违反它服务不起、退 78，出路是停着服务时 control clear-queue（这一版不认那种库）。
  history 表放钉值（下面「资源记录」）；导进来的峰值与用时那几行定额度不读。

命令自己打的登记行（去掉首尾空白之后整行照写法，别的写法不认；打完要刷出去；第二台的日志经 multi-host-run.sh 接回本机，同样读得到）：
  心跳行「TASK_SERVER_HEARTBEAT <说明>」（HEARTBEAT_LOG_MARK）：命令在编译、等锁这类一阵子不出输出的时候按固定间隔打的「我还活着」。以它起头的行不算日志在涨
    （task_server.py 的 log_content），所以判卡住看的「日志不涨」不被它盖住；它也不进理由行里的「日志最后一行」（REASON_LOG_NOISE）。
    命令真卡死时只剩心跳行、CPU 也不涨，满 STALL_SECONDS 照卡住停；夹着真输出的不停。弄坏开关 heartbeat-counted-as-log 下心跳行照算日志在涨。
  判定库进度行「PRUNE_SERVER_CLIENT_PROGRESS <说明>」（VERDICT_LIBRARY_PROGRESS_LOG_MARK）：判定库客户端（scripts/prune-server/prune_server_client.py、prune-server/src/reliable_client.rs）
    在命令跟 prune-server 一问一答、自己没有别的输出的时候按固定间隔打的「还在跟判定库一问一答」。与心跳行相反：以它起头的行算日志在涨（task_server.py 的 log_content），
    命令只在等判定库回话、CPU 也不涨时靠它不被判卡住（服务读不到客户端此刻在不在等回话：判定库服务只登记连着的客户端，不记哪条连接有请求在等）；它不进理由行里的「日志最后一行」（REASON_LOG_NOISE）。
    客户端自己有等回应的时限与重连总时长上限（那份文件头「容错」），等不到回话它自己退 75，所以这一行不会把真卡住的命令一直留着。
    弄坏开关 verdict-library-progress-uncounted 下这一行不算日志在涨。
  脚本永远不报自己要多少内存（用户 2026-11-13 定「脚本永远不要报自己要多少」）：上限只照命令键的记录定、撞了乘 1.5，命令打的任何「要多少」行一律不认。
    一次尝试跑了几个小时才撞上限、整次重来，是这套机制本来的样子，不是要修的缺陷：不为它改起步上限、不借别的键的记录、不让脚本预估；
    要省的是任务自己的内存与用时（算法、重复计算、重复编译），峰值降下来之后记录跟着降，撞的次数自然少。
    服务自己的尝试只用跑完的（失败的、提前退出的尝试，它的峰值不拿来定下一次的额度；RECORD_COMPLETED_OUTCOMES、RECORD_CAP_HIT_OUTCOMES）：
      退 0 的（succeeded）记它量到的峰值；撞了这一条自己上限的（memory-cap-hit，到顶收尾的那一次同样）记「至少要到这个上限」，按它的上限算（真峰值不小于它；
      峰值远低于上限的取峰值；cap_hit_record_bytes 一处算，task_server.py 的 cap_hit_measurement 与 control drop-history --attempt-records-over-gib 都经它）。
      所以下一次同一命令从撞过的最大上限起，不从小的再爬一遍。
      别的结局（command-failed、cancelled、launcher-lost、squeezed-out，连同错误类型表里别的几种）的峰值不进窗口、不占名额；尝试记录照存，status 照显示。
      被停、判红的那几次量到的峰值只当下限（task_server.py 的 stopped_peak_floor；admission 审核 F6）：最近 RECORD_WINDOW 次里最大的比窗口里最大的还大才抬上去，
      窗口空着的比没有记录的缺省（4G）大才照它起；偏小的不拉低、不占名额。下一次同键起步不再从撞过的低档重撞。弄坏开关 stopped-peaks-not-counted 下不当下限。
      理由：提前失败的那一次峰值偏小，不代表这条命令要多少（records/2026-10-30-编译绕过内存总上限.md「服务修复两件合起来验」#8：脚本提前退 1 的那一次峰值 42 MiB，
      修好之后同一条命令照它定 63M，一路撞上限放大到 728M）。
    没有记录的命令（用户：「没记录的先记小，撞了再放大」）：库里这把键一次跑完的都没有的（只有没跑完的尝试也算），
    预测、上限与记账都取 UNRECORDED_MEMORY_DEFAULT_BYTES（4G，上限写成 4096M；不乘 1.5；task_server.py 的 unrecorded_prediction）。撞了 250 照「重试」乘 1.5 重排（4096M → 6144M → 9216M …），乘 1.5 会到那台总上限的换总上限更大的一台、没有就记
    memory-cap-at-maximum 打回，同一套；撞过的那几次进了服务的记录，同一条命令下一回按记录定。缺省取 4G 的理由（主 agent 定的，records/2026-10-30-编译绕过内存总上限.md
    「问题清单的去向」#6）：经服务跑过的任务 1–8 实测峰值 10 MiB 到 1.71 GiB；常见命令已从峰值表导入 3321 条记录，没有记录的多是小命令；
    按 8G 的缺省乘 1.5（12288M）记账时，几条没记录的命令就把本机 slice 记满，后到的被挤去第二台（records 同一份「批 3b 交回与验证」任务 5、6）。
  这把键（连同片数）自己没有记录的不借别的键、不借同一趟的兄弟片：照上面从 4G 起，撞了上限照「重试」放大，同一套。
  钉值（用户：「变异的内存上限直接按照 根目录设置的50gb走」「你这边也直接 将数据库设置为50gb 让他自己算一次多少个并发」「50 这个是读取的配置 不是随便给的」）：
    history 表里 state=kind=pin 的一行按命令键记「这条命令的上限取多机配置 SLICE_MEMORY_CAP_BY_HOST 里派到的那一台那一项」（PIN_STATE、PIN_SOURCE），不记字面的数；
    定额度先认它，预测、上限与记账都取每次现读的配置值减那一台此刻的余量：判哪一台放不放得下就取哪一台那一项（task_server.py 的 booked_at_cap、pinned_on_host 按台换），
    减完不大于 0 的那一台不派（control pin-cap 当场拒，task_server.py 文件头 ②「内存」的钉值），尝试记录 prediction 记
    PINNED_HISTORY；学到的峰值不顶掉它；撞了上限不放大，照 memory-cap-at-maximum 打回。那一项减余量超过这台 slice 总上限减余量的（host_capacity），当那个数用，control pin-cap 回读时报（task_server.py 的 pin_over_limit_lines）。
    内存包装退 254（被 slice 挤掉：任务 10289 钉值等于 slice 总上限，包装写「这一条的结果不算数，重跑」）的钉了也记 squeezed-out、照那一行重排，不算撞上限（task_server.py 的 finish_attempt）。
    配置读不到（没有这个键、写法不对、少一台、多了不在 HOSTS 里的地址、认不出 HOSTS 里哪一项是这一台、文件读不了）的不起、不退回学到的记录：结局 run-condition-unmet（退 78），
    TASK_SERVER_REASON 的 why= 写读不到的原因与出路（改对 SLICE_MEMORY_CAP_BY_HOST，或 unpin-cap 撤掉钉值，再重交；task_server.py 的 pinned_prediction，预测记 PIN_UNREADABLE_HISTORY）。
    写法：python3 task-server/control pin-cap <命令…> / unpin-cap <命令…>（经服务的 POST /pins，服务在跑时也能用；task_server.py 文件头 ②「内存」的钉值）。
  每次尝试经环境 TASK_SERVER_MEMORY_CAP_BYTES（MEMORY_CAP_VARIABLE）把这一次的内存上限（字节，十进制）交给命令，本机与第二台都给。
  每次尝试另经 TASK_SERVER_SOURCE_REPOSITORY（SOURCE_REPOSITORY_VARIABLE）交提交方那一份仓的根（服务所在那棵仓的绝对路径），本机与第二台都给：第二台在同一棵共享的树里跑，
  它就是脚本自己所在的仓；要拼两台一致的路径（~/.claude/projects/<仓路径改写> 这类）的脚本照它拼（任务 2653、2654：拷树那时 agent-cost.py 在第二台拼出了拷贝的路径、退 2）。
  共享根（用户 2026-11-01「先删空间 然后修改 【第二台的 ~ 不是两台共享的那个目录】的bug」；records/2026-10-31-task-server巡检.md 问题 73：第二台上经 ~ 取的工作根与共享数据
    落在第二台本地盘，两台读不到同一份）：每次尝试另经环境交两个变量，本机与第二台都给，第二台原样带本机的值（同 TASK_SERVER_SOURCE_REPOSITORY，从「值是本机绝对路径的不带」里摘出来）：
    SINGLEFS_DATA_ROOT（DATA_ROOT_VARIABLE）是服务这一侧共享数据根的真路径（上面「共享数据根」：服务自己的环境里设了 SINGLEFS_DATA_ROOT 照它，没设取多机配置的 SHARED_DATA_ROOT），
    SINGLEFS_WORK_ROOT（WORK_ROOT_VARIABLE）是 .claude/hooks/lib_work_root.py 的 work_root() 在服务这一侧现算的真路径；任务自己带了这两个（白名单里的）照任务带的。
    提交时核两个值都落在多机配置 NFS_SHARED_ROOTS 的某一段下（task_server.py 的 shared_roots_problem）：有一个不在就当场拒，点名是哪个变量、值是什么，出路是配多机配置
    （task_server.py 文件头 ②「提交时拒」②）。
  有记录的照记录、不设 4G 底：没有记录的照上面先记 4G；有记录的（prediction 记 RECORD_HISTORY）上限与记账照记录，不往 4G 抬；
    撞了上限照「重试」乘 1.5 重排，重排那一次照乘出来的上限，不兜到 4G；撞过上限的命令再交一遍，撞过的那一次的上限进记录窗口，头一次照它起。
    钉值照旧。弄坏开关 recorded-cap-floored 下把下限抬到 MEMORY_FLOOR_RAISED_BYTES（4G）：钉值之外低于 4G 的上限与记账都抬到 4G（有记录的头一次、重排都兜；
    task_server.py 的 memory_floor、memory_floor_applied）。
  内存上限的下限（MEMORY_FLOOR_BYTES，1 GiB：新任务起步不低于 1G；任务 11127 只读一个文件的 python3 脚本，峰值 5 MiB，上限 3M、5M、8M 都退 250，12M 才退 0）：记录的峰值只算进程内存
    （anon 加 shmem），上限还要装读文件带进来的页缓存，峰值很小的命令照记录定出的几 MiB 装不下。起步的上限与重排的上限（computed_prediction 里定出来之后、往上抬之前，
    task_server.py 的 memory_floor_applied）都不低于它，记账照这个上限记（booked_at_cap 取上限为账）；记录比下限大的照记录，没有记录的 4G 与撞了上限的重排一档本来就比它大，不受影响。
    钉值不经这里：钉值是多机配置那一台那一项减余量，对这条命令单独设的就是这个数，低于下限也照钉值；上限到不了的台照旧收到那台能给的最大上限为止（booked_at_cap），
    下限不超过台的上限。弄坏开关 cap-floor-skipped 下不设下限。
  上次运行时长（task_server.py 的 last_run_seconds；只给插队规则 jump_refusal 用）：服务不预计用时（用户：「没有预计用时这个东西。我们整个项目现在不预计用时。仅仅预测内存。」），
    只记「这把命令键上次真跑完用了多久」：服务自己库里同一命令键（与内存记录同一把键：路径、条目照词折过，--check 的格名照留进键）、命令自己跑到退出的尝试
    （结局是 RUN_COMPLETED_OUTCOMES：succeeded、command-failed；被信号杀的 command-failed 不算，撞上限、卡住被停、被挤掉、取消、满四小时被停这些服务判出或打断的不在里面）。
    判据：退 0 的取最近一次的用时；退非 0 的取近 RECORD_WINDOW 次跑到退出的尝试里最长的一次，比退 0 的那个数长才取它；没有退 0 的记录时，退非 0 的最长一次
    跑满 FAILED_RUN_RECORD_MINIMUM_SECONDS（600 秒）才取它，没跑满的交 None（一开始就退 1 的不说明这条命令要跑多久）。分片的计划、各片与合并各照自己那一步的命令键取。
    弄坏开关 estimate-only-succeeded 下退非 0 的都不算，estimate-short-failure-counted 下没有退 0 的记录时早退的也算，estimate-latest-any-outcome 下不分退码取最近一次，
    estimate-failure-outside-window 下退非 0 的不限近几次，estimate-longest-succeeded 下退 0 的取最长一次，estimate-by-exact-command 下只认命令逐词相同的。
  退出时的内存峰值（任务 5012：跑 1 秒左右的片本机记 null，第二台的计划、合并与一片也是 null；任务 10045 这一族在第二台跑约 12 秒，每 5 秒一轮只读到外壳，记 18 MiB，本机 10270 量到 556 MiB，预测跟着掉到 18 MiB）：
    每一次尝试另经环境给内存包装 RUN_WITH_MEMORY_CAP_PEAK_OUTPUT（PEAK_OUTPUT_VARIABLE）= <尝试目录>/EXIT_PEAK_FILE_NAME，包装退出之前把这一条扣页缓存的峰值（它每 0.1 秒采的回收不掉的用量，
    scripts/hosts/run-with-memory-cap.sh 文件头「峰值表」）写进去；两台同一个口径：本机 attempt_runner.py 照说明里的 exit_peak_file 读、第二台 multi-host-run.sh 读了打回本机，都与按轮采的取大的记进 memory_peak；
    第二台读不到这一个的打不出 MULTI_HOST_RUN_PEER_MEMORY_PEAK，记「没量到」（null，不进定额度）。撞上限的那一次 memory_peak 照读数记，定额度按它的上限算（task_server.py 的 recorded_measurements）。
    弄坏开关 peak-read-only-by-sampling 下不给、不读（只靠按轮采），exit-peak-only-when-unsampled 下本机按轮采到了就不看它。
  峰值的口径（两台同一种量）：按轮采的是进程用的内存——内存包装那一层 scope 的 memory.stat 的 anon 加 shmem（scripts/hosts/memory_peaks_lib.py 的 process_memory_bytes），
    留见过的最大；页缓存与内核挂在 scope 上的不算（任务 8853 编验证包：anon 921M、file 22163M，memory.peak 顶到上限 50176M，照 memory.peak 记，照它起步就记满一台）。
    本机在 attempt_runner.py 的采样里读（它文件头 ③），第二台在 multi-host-run.sh 每轮那一次 ssh 里读（它文件头 ⑤「峰值」）。退出时那一份是 scope 的 memory.peak（进程退了之后
    读不到进程内存），包装退出时交的那一份另把按轮之间冒出来的补上。弄坏开关 memory-peak-with-page-cache 下本机按轮读 memory.peak。
  峰值口径一栏（records/2026-10-31-task-server巡检.md 批 93：批 92 之前记的 memory_peak 含页缓存，起步被抬到 45–50G）：服务记下结局的每一次尝试另写一栏
    peak_metric = PROCESS_MEMORY_PEAK_METRIC（「进程内存」，上面那一种口径）。定额度（recorded_measurements）与 fills-any-cap（task_server.py 文件头 ①「要处置的提醒」）只读这一栏是它的行（task_server.py 的 current_metric）；
    没有这一栏的旧行不读、不改、不补（不迁移）。弄坏开关 old-metric-peaks-read 下照改前连旧行一起读。
  第二台的内存峰值（问题 86）：第二台那一层的 cgroup 本机量不到，multi-host-run.sh 在第二台那条命令结束时打一行「MULTI_HOST_RUN_PEER_MEMORY_PEAK bytes=<十进制字节数>」
  （按轮采到的进程内存的最大，按轮一次都没采到才照第二台那一层包装退出时读的，都没有不打；那份文件头 ⑤「峰值」），服务把它记进那一次尝试的 memory_peak，同本机一样进定额度；
  不打的照旧记「没量到」（null）。
  编译目录占盘（问题 76）：本机 attempt_runner.py 量服务给的编译目录（CARGO_TARGET_DIR 所在）的占盘峰值，第二台照 multi-host-run.sh 打的
  「MULTI_HOST_RUN_PEER_BUILD_PEAK bytes=<十进制字节数>」，记进尝试的 build_peak；预测同内存的键与取法（最近 RECORD_WINDOW 次跑成的最大值），派活时那一台盘余量
  要放得下预测加 task_server.py 的 BUILD_DISK_FLOOR_BYTES（task_server.py 文件头 ②「编译目录占盘」）。
  命令键另剥开临时 prune-server 的包装：python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service [--server-binary <路径>] -- <原命令>
  照 -- 后面那条原命令算（unwrapped_temporary_service；--server-binary 给的路径不进键；认登记表行（提交时拒的判定库那一种）同样照原命令认，包装本身照旧整条执行），
  键前面另接 TEMPORARY_SERVICE_KEY_MARK：包着的那一次在同一个 scope 里多起一份 prune-server，峰值比不包的大，不与不包的那一条共用记录
  （任务 3013 包着跑、照记录取到 955M 撞顶，推的：记录来自不包的那一种）。弄坏开关 temporary-service-wrapper-kept 下不剥，temporary-service-key-shared 下不接记号（包着的与不包的共键）。
  这层包装只给脚本自证用：剥出的原命令有会编的 cargo 的，提交时就拒，出路是去掉包装重交（task_server.py 文件头 ②「提交时拒」④）；编译目录只照外层那一条给。
  不带 --server-binary 的包装自己要现编 prune-server（prune_server_client_selftest.py 的 resolve_binary 只在环境里有任务服务给的编译目录 CARGO_TARGET_DIR 时现编），
  所以外层这一条按会编的算（temporary_service_builds_server，task_server.py 的 compiles）：两台都由服务给编译目录；带了 --server-binary、或命令前的赋值与任务环境里
  给了 PRUNE_SERVER_BINARY 的不编、不给。弄坏开关 temporary-service-build-unrecognized 下不认（改前的样子：派本机不给编译目录、包装退 2，派第二台由 multi-host-run.sh 照例给）。
  自带临时服务的两份脚本（OWN_SERVER_SELFTESTS）单独起的自证同理：prune_server_client_selftest.py 的 --selftest 与 --selftest-families、prune-server-load-simulation.py 的 --selftest
  不带 --server-binary、又没有 PRUNE_SERVER_BINARY 时，自己走同一个 resolve_binary 现编，也按会编的算（own_server_selftest_split）：两台都由服务给编译目录，带了二进制的照旧不编。
  任务 7069、7070、7071 都记成 "compiles": false 而退 0（别的会话读任务记录量到的），应是因为它们落在第二台、第二台照例由 multi-host-run.sh 给了编译目录；派本机不给编译目录、resolve_binary 退 2 是按源码推的，没有任务实测过。
  弄坏开关 temporary-service-own-server-unrecognized 下这两份不认。
  线程固定（用户：「每次都开31 按照内存查。 现在核心的问题是内存不足」）：CPU 闸照在跑尝试的线程之和与负载取大判（task_server.py 文件头 ②「CPU 闸」），准入的账只记内存。
  准入记账：新起的按上限记（记账 = 上限；预测峰值、上限与记账都收到判的那一台总上限减那台此刻的余量为止，钉值、记录、缺省、撞上限重排四种来历一样收，task_server.py 的 booked_at_cap；
    预测峰值比某一台大的，别处装得下就派别处，哪台都装不下就照收到的数起跑、不判永远放不下，task_server.py 的 hosts_short_of_record）；在跑的只照报告里它的 scope 此刻回收不掉的用量记、不超过它的上限
    （task_server.py 的 running_booking：不加固定余量、不看任何预先估的数；报告里没有它的 scope 的记整份上限），
    各自的上限（memory.max）照旧，超了只杀它自己；
    slice 兜底：那台 slice 实测余量不到 SLICE_ROOM_FLOOR_BYTES（4 GiB）这一轮不派新任务（task_server.py 的 slice_floor_problem）；
    分片的片：记满上限放不下、预测放得下时，上限与记账一起压到那台此刻放得下的量（用户：「不能超过 内存上限」；task_server.py 的 shard_cap_fitted_to_room）：记账之和不超过总上限，超了只杀它自己。
    余量每台现量、按 1 GiB 向上取整，不是常量（task_server.py 文件头 ②「余量」，量法在 scripts/hosts/memory_peaks_lib.py 文件头「余量」）；
    还没涨到量的那一截照「记账 − 它此刻回收不掉的用量」算（task_server.py 的 running_booked、server_pending；有 scope 的是 0，只剩刚起还没读数的那几条），两台同一段：用量照这一轮读那一台余量的那一份报告
    （host-dispatch.py 的 probe_local / probe_peer，第二台是同一次 ssh）按 scope 量回来、照 scope 里进程环境的 ATTEMPT_LABEL_VARIABLE（每次尝试两台都给）对到尝试上，
    报告里没有那一条的 scope 记整份上限；
    第二台那一层内存包装不排队、不记账，记账只在服务这一处。

挑机器（task_server.py 的 candidate_hosts 与 place；本机的环境、库与配置第二台那一趟带不全，records/2026-10-30-编译绕过内存总上限.md 批 3b）：
  按余量均衡（用户：「两台或者多台的负载均衡也是一个重点」，选「三条都改，按余量均衡」）：任务在派得过去的几台里挑放得下的
    （照 host-dispatch.py 的 judge_host，减去还没涨到量的那一截（排着的一律不留账；排队先后照 task_server.py 文件头 ②「排队先后」那两条规则）；预测用这一条存的那一份、余量用这一轮读的那一份，起之前现读复核一次，
    task_server.py 文件头 ②），放得下的不止一台时挑放下之后剩余占那台总上限比例大的，一样大取本机；
    都放不下的照排队，哪台都永远放不下记 never-fits。撞上限重排不钉台（上面「重试」）。
  派不过去的三种提交时就拒（task_server.py 文件头 ②「提交时拒」）：工作目录与命令里有不在 NFS_SHARED_ROOTS 某一段下的绝对路径（/tmp 下的同样算）、
    交给命令的两个共享根不在共享根下、带判定库而登记表 .claude/gate.d/verdict-library-scripts.tsv 第三列还没写 prune-server（或判不了带不带）。
    第二台在同一棵共享的树里、同一个绝对路径上跑（用户 2026-11-01「要是你倾向于把仓本身也做成 NFS 共享、第二台直接在活树上跑。 我认为这么做比较好。」）：
    路径的读法（task_server.py 的 find_outside_shared_path）只认整个 shell 词：以 / 起头的整个词、「名字=绝对路径」「--选项=绝对路径」的值、shell -c 脚本词照 shell 词切开之后
    以 / 起头的每一个（shell_word_paths）；别的词中间夹着的 /（参数值里的「（接的是 /）」）不算。scripts/hosts/multi-host-run.sh ③ 照它自己的读法同样拒（退 255）。
    两台相同的系统路径（HOST_INDEPENDENT_PATHS：/dev/null、/dev/std*、/dev/fd/、/usr/、/bin/）不判；命令里的相对路径照工作目录解。
    不拷树、不送、不取回、不改写路径；写出的文件就在共享的树与共享根下，两台都读得到。
    编译目录在第二台本地盘上：这棵树在第二台的组里那一个，multi-host-run.sh 在多机配置 BUILD_ROOT_BY_HOST 里第二台那一项下的 builds/<标签>/ 建、带 --keep-build-directory 留着（下面「编译目录」）。
  没有只派一台的名单（用户：「我再重申一次 【只能在本机跑】 是不能接受的。必须尽快解决 让错误暴露错来」）：点到哪个目录、哪个入口的命令都派得到两台，哪台由余量定；
    派到第二台跑红的照红，由脚本那一侧修，服务不替它挡（弄坏开关 host-lists-restored 下照改前放回 task_server.py 的 RESTORED_HOST_LISTS）。派哪台只看第二台此刻用不用得了（连得上、挂着共享根、
    盘余量够，下面两段）与余量。改主工作区的工具（hook-swap.py、apply-writer-patch.py、stage-mine.py、scripts/kb/ 下的）
    与带 --write 的派得到第二台（改的就是同一棵树）；它们撞上限这类本会重排的结局照旧不重排（task_server.py 的 modifies_workspace）。
    包在里面的照认（task_server_scheduler_building.py 的 wrapped_workspace_commands）：bash / sh -c 的串照 shell 词法切成几条、每一条照命令那一层同一种折法认；
    命令位置上起的脚本（bash <脚本>、python3 <脚本>、直接起的）在仓里或工作根下的，读正文、认其中命令位置上写着的；只读参数与 --feed-only 只免它自己那一条。
    一条命令里认到的写工具一条不落（task_server_scheduler_building.py 的 workspace_writer_commands），另有三处共用：换钩子退 7、9、10 的两种提醒认不认它是换钩子
    （包着的换钩子带 --dry-run 这类只读参数的也认，同命令那一层），退 7 找上一趟疑似是哪条被杀的换钩子任务，被杀时 workspace-write-interrupted 逐条判、点名写工具与要核的路径
    （apply-writer-patch.py 照它的结局文件判：补丁目录照这一条命令里认得出的 cd 与赋值算，task_server_scheduler_building.py 的 shell_text_runs；算不出的照可能写了一半报，
    花括号组、heredoc 正文里有 cd 的整段与串里那一条自己带 env -C 的都算不出；hook-swap.py 照它的 --read-outcome 判，换完了、已退回的与结局文件归同一条命令里另一条换钩子的不报）。
    合入后验证（scripts/gate/merge-and-verify.sh）只在自己拷的快照上打补丁与验、主工作区一个字节不写，不算改主工作区，撞上限照常放大重排。
    不认合入锁（task_server.py 文件头 ②）：主工作树正在合入时照常派。
  第二台用不了要告警（task_server.py 的 note_peer）：连不上、余量查不成、多机配置判不过、挂载复核没过、盘余量不到下限（下一段）、那一趟没跑成（peer-run-failed），
  写告警 peer-unavailable，/health 的 peer 一栏写明原因，用不了满一段时间服务发「服务健康」给会话（task_server.py 文件头 ⑥）；哪一台此刻用不了，排队原因带上那一句。
  盘余量（records/2026-10-31-task-server巡检.md 问题 72：第二台根分区剩 353M 时任务 peer-run-failed；两台同一个下限）：服务读每台余量的那一次
    （task_server.py 的 Resources.snapshot，同一个缓存周期）另判那台编译目录所在盘（本机是 TASK_SERVER_BUILD_DIRECTORY 所在盘，第二台是 BUILD_ROOT_BY_HOST 里它那一项）的余量：
    报告里有 disk_available（字节，task_server.py 的 DISK_REPORT_KEY）照它，没有就照那一台的接口现量（本机读盘，第二台经同一条连法起 df -B1 --output=avail -- <编译根>）；
    不到 DISK_FLOOR_BYTES（50G）或查不成，那台这一轮用不了，原因写「<台>盘只剩 X（下限 Y）」，第二台的照上一段告警（盘不够这一种只写一条，余量的数变了不另写）、
    /health 的 peer 一栏写明；够了下一轮自己认回。
  分片的计划、各片与合并都不钉台，哪台由服务挑（各片第二台在前）。提交时不收挑机器的参数，没有「点名要第二台」的写法。
  同一件的兄弟片不避开彼此（用户：内存够就多跑）：放不放得下只看那一台的内存额度，同一台能同时跑几片；不按片号预先定台。
线程（同上）：每次尝试的命令外面套 scripts/hosts/capped.sh <N>，N 是登记的每台默认重活线程数（.claude/heavy-thread-default，经 scripts/hosts/heavy_threads.py 取）；
  一律这一个数，提交时不收（SINGLEFS_THREAD_CAP 不在白名单里，带了当场拒）。第二台那边同一个数（双机两台各取这一个数）。
卡（.claude/rules/verification.md「卡只归 GPU 服务，任务不拿卡」）：卡归 GPU 批处理验证服务（gpu-verification-server/），它照多机配置开卡、一直持着；这个服务只调配 CPU，
  不认谁要卡、不分卡、不清场、不收卡，交给它的任务只跑录入、往判定库写待判行、读结论。命令位置上起的脚本（bash -c 里命令位置上的每一条同样认）文件头前
  DECLARED_HEADER_LINES（60）行里写了领卡、要卡的声明（CARD_DECLARATION_MARKS：# task-server-card-lease:、# task-server-gpu-cards:、# task-server-gpu-arguments:）的，
  提交时当场拒，拒绝说明头一句是 CARD_DECLARATION_REFUSAL「卡归 GPU 批处理验证服务，任务不领卡」，出路写删掉那一行、要卡的那一段交 GPU 服务判
  （card_declaration_problem；task_server.py 文件头 ②「提交时拒」⑧；异常表 card-declaration）。弄坏开关 card-declarations-accepted 下照收。
  选卡的变量（CARD_CHOICE_VARIABLES：GPU_CARDS_BY_PRIORITY、CUDA_VISIBLE_DEVICES）任务带了提交时拒（TAKEN_OVER_VARIABLES），服务自己环境里带着的不交给尝试
  （task_server.py 的 start_attempt；弄坏开关 card-variables-passed-to-attempt 下照交）。
续跑（用户 2026-09-26：跑超过 30 分钟的测试、装置、门禁阶段与脚本要能中断续跑）：脚本自己能续跑的，在文件头声明「# task-server-resumable: <词> [<词>…]」
  （DECLARED_RESUMABLE_MARK，读法：前 60 行、第一个非选项参数是其中一个词或写了 *，只看第一个；resumable_command，起的脚本照 command_word_index 剥掉
  env、nice、timeout、setsid、bash / sh / python3、capped.sh N 之后认）。被打断后重跑同一条命令就接着跑的才写这一行。
  一次尝试跑超过 LONG_RUN_SECONDS（1800 秒）、任务的结局不是 succeeded（取消的不算）、命令又没声明续跑的，结局说明前面写「这条跑了 N 分钟、没有续跑：重交会从头跑」
  （task_server.py 的 finished_preface）。弄坏开关 long-run-not-noted 下不写，resumable-declaration-ignored 下声明了也当没声明。

编译目录（用户：「我们说的不共用编译 指的是以前每个任务，测试时分开的副本」「如果每一次改代码都要全量编译，改一点就要编译 这是不能接受的时间浪费」）：
  按「哪一棵树 × 哪一台」给：树是任务 --cwd 解开符号链接后往上第一层有 .git 的那一层（build_tree_root；哪一层都没有就是解开后的 --cwd 本身）。
    同一棵树在同一台上先后交的会编任务接着用这棵树在这台的那一组编译目录，cargo 照指纹只重编改过的；不同的树各给各的组，从不共用一个
    （几棵不同的树共用编译目录、拷贝保留旧修改时刻时 cargo 判成不用重编：records/2026-09-16-subagent拆分提案.md 第五十二、六十九节）。
  组（task_server.py 的 build_slots 表、take_build_slot）：每棵树每台至多 BUILD_POOL_SLOTS_PER_TREE（task_server.py，3）个；一次尝试拿一个空着的（占着，同一时刻只给一次尝试），
    尝试不在起、在跑了就放回。拿法依次是：这条任务上一次尝试在同一台用的那一个（撞内存上限照大一档重来，接着编）、空着的里最近用过的那一个、组没满就新开一个；
    都占着又满了的排着（task_server.py 文件头 ②「编译目录组」）。组的上限与编译目录占盘那一层一起定，写在 task_server.py 的 BUILD_POOL_SLOTS_PER_TREE 那一段与 ②「编译目录占盘」。
    接着用组里那一个的，占盘那一判从预测里扣掉它已占的盘（组里每一个的占盘在每次尝试收尾时照量到的峰值记：task_server.py 的 settle_build_slot、free_build_slot_bytes）。
    弄坏开关 build-slot-per-attempt 下每次尝试新开一个（不接着用、不看组满），build-slot-tree-ignored 下不分树（这台上各棵树的组当一组），build-slot-held-shared 下占着的也给出去。
  两台同一段（Machine.build_directory）：本机那一个在 <TASK_SERVER_BUILD_DIRECTORY>/tree-<树路径的 sha256 前 8 位>-<随机 6 位十六进制>/target（task_server_machines.py 的 build_directory_name；
    名字不带任务号与尝试标签：组里的目录由同一棵树上之后交的任务接着用，照标签起名会让接着用它的每条任务的进程在 ps 里都挂着开它那条任务的号），经 CARGO_TARGET_DIR 交给命令；
    弄坏开关 build-directory-named-by-label 下本机的照开它的那一次尝试的标签起名。
    第二台那一个是 multi-host-run.sh 在多机配置 BUILD_ROOT_BY_HOST 里它那一项下的 builds/<开它的那一次尝试的标签>/target（名字的唯一来源是 --label，所以带开它那一次的标签）：服务给编译目录的尝试派过去都带 --keep-build-directory，
    组里已有的另带 --reuse-build-directory <那一个的标签>（multi-host-run.sh 文件头「留着的编译目录」）；它跑完打了 PEER_BUILD_KEPT_MARK 的组里记成可接着用，
    没打的（中途失败、被停，⑥ 连它一起删了）组里删掉那一个（task_server.py 的 settle_build_slot）。两台的组互不相干。
    弄坏开关 peer-retry-build-directory-fresh 下第二台不叫留着、不点名接着用。
  入口声明（lifecycle 审核 F1：门禁入口里面才起 cargo，命令那一层认不出）：命令位置上起的脚本文件头前 DECLARED_HEADER_LINES 行里写「# task-server-compiles: yes」
    （DECLARED_COMPILES_MARK、COMPILES_DECLARED_WORD；declared_compiles）的，服务当它会编（task_server.py 的 compiles、compiles_declared），照上面给编译目录；
    单独 --selftest、带 --list-items、--help、-h 的不算；写了别的词的打回 run-condition-unmet，说明写那一句。分片的计划、各片与合并照父任务（create_child）。
    命令拿到的是 CARGO_TARGET_DIR（组里那一个的 target）：入口照它编、不删。弄坏开关 compiles-declaration-ignored 下不读这一行。
  命令里命令位置上有会编的 cargo（.claude/hooks/lib_workspace_cargo_lock.py 的 cargo_invocations 与 BUILDING_SUBCOMMANDS 认，包装剥掉之后照认）、调用方没给编译目录的，同样照上面给。
  删（「服务build之后必须删除历史」「自己用完自己删」照树算）：树删了编译目录跟着删——组里那一行的树此刻不是目录了、那一个空着的，服务删它
    （本机就地删、第二台经 multi-host-run.sh --remove-build-directory；有进程开着里面的文件就不删、写告警 build-directory-left、下一回再扫），删了、回读不在了才从组里删掉那一行；
    服务起来时扫一遍，之后排队线程至多每隔 BUILD_POOL_SWEEP_SECONDS 在后台再扫一遍（task_server.py 的 sweep_vanished_tree_pools）。弄坏开关 vanished-tree-pool-kept 下不删。
    同一扫里另删本机编译根（TASK_SERVER_BUILD_DIRECTORY）下不在任何组里、名字是尝试标签那一种（task-<这一个库的标签>-<任务号>-<第几次>）的目录
    （task_server.py 的 sweep_unpooled_attempt_label_directories）：本机组里新开的一律照 build_directory_name 起 tree-… 的名字，这种名字的目录不在组里就没有哪一次尝试接着用它；
    组里任何一行记着的与在起、在跑的尝试记着的编译目录不动，别的名字、别的库标签的不动；删法同上（有进程开着里面的文件就不删、写告警 build-directory-left、下一回再扫），
    删了、回读不在了才算删了。只扫本机：第二台的 builds/ 下照标签起名的是 multi-host-run.sh 那一侧的名字，留着的都在组里，没留的 ⑥ 当场删。
    弄坏开关 unpooled-label-directories-kept 下一个都不删，unpooled-label-directory-open-unchecked 下删之前不核有没有进程开着，unpooled-label-directories-pool-unread 下组里记着的也删，
    unpooled-label-directories-running-unread 下在起、在跑的尝试记着的也删。
    任务记下结局、task clean <号>、取消都不删本机组里的：clean 只删那一条任务自己的现场（下面「现场」）与分片父任务的测试二进制；取消的照旧删它自己的现场，本机组里的编译目录放回组里；
    派到第二台、中途失败或被停的那一趟（没等到退出码），multi-host-run.sh 的 ⑥ 连它用的组里那一个一起删，收尾时组里删掉那一行（task_server.py 的 settle_build_slot）。
    wait 末行不带 build=（编译目录归树、不归任务）。弄坏开关 clean-removes-build-slot 下 task clean 照删各次尝试用过的组里那一个。
  调用方给了编译目录：环境里的 CARGO_TARGET_DIR / CARGO_BUILD_TARGET_DIR、命令里的 CARGO_TARGET_DIR=… 赋值词与 --target-dir；落在仓里（这一棵树或主工作树里）的提交时拒（退 3），
  落在仓外的照它、不删（归调用方），不进组。只认写在词上的，bash -c 字符串里的另由钩子那一层判。

现场（别的会话报：判红那一趟内存包装留在 /tmp 的临时目录被后来的包装按「只留最近 3 个」轮掉，panic 原文取不到；用户：「用完就要删掉。不要残留了」）：
  每一次尝试经环境给内存包装 RUN_WITH_MEMORY_CAP_SCENE_DIRECTORY（SCENE_VARIABLE）= <工作根>/SCENES_ROOT_NAME/<尝试标签>（Configuration.scenes_directory 下；共享工作根，
  两台同一个绝对路径；服务起来时建 SCENES_ROOT_NAME 那一层，每一次的目录不预建）。命令退非 0 而它的临时目录里有东西、或设了 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1 的，
  包装把那一份整个搬成这个目录、不进 /tmp 下的轮换，日志里那一句点名新位置（scripts/hosts/run-with-memory-cap.sh 文件头「task-server 给这一层的两样」）；
  派第二台的由 multi-host-run.sh 原样设给第二台那一层包装（那份文件头 ⑤「现场目录」）。尝试记录的 scene_directory 记这个路径（status 的 JSON 看得见）。
  什么时候删：提交它的会话 task wait 取走结局（collected_at）之后过 SCENE_KEEP_AFTER_COLLECT_SECONDS（task_server.py，1 小时）由排队线程每一轮扫一遍删；没有会话的（人手 --manual，不一定起 wait）照结局时刻算
  （sweep_scenes；分片的子任务照父任务取走的时刻）；task clean <号> 当场删。设了 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1 的只由 task clean 删。
  取消的不等：task cancel 回读到单元停了就在同一次取消里删（上面「编译目录」的取消的不留产物）。
  这一个库的标签之外的目录不动；任务已不在库里的（clear-queue 清过）当场删。
  取走之后再留一段、不在「有结局」那一刻删的理由：结局一出来提交方还没读日志，现场就没了等于白留；wait 取走之后提交方才读日志、照日志里那一句去读现场，
  留一小时够读完。没人 wait 的任务（列在 GET /sessions/self 的「完成了没人取」里）一直留到 task clean。
  弄坏开关 scene-removed-at-finish 下照「有结局就删」那一刻删，scene-kept-after-collect 下取走之后也不删，scene-keep-ignored 下设了 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1
  的取走之后照删，scene-left-by-clean 下 task clean 不删现场。

分片（用户：「现在就把分片收进服务」）：
  服务认得「这件要分片」：命令（剥掉 env、nice、timeout、setsid、bash / sh / python3、capped.sh N 之后）起的是 SHARDED_ENTRIES 里的四份入口之一而参数对得上，
    或起的脚本文件头前 DECLARED_HEADER_LINES 行里有一行「# task-server-shards: <第一个参数> …」（写 * 是除 --selftest、--list-items、--help、-h 之外都分），第一个参数对得上。
  一件拆成几个子任务，各自是普通任务：各自按记录定额度（子任务的命令键是这件的键加「〈task-server 分片 <步> 第<片号>片〉」，只带片号、不带片数：计划切出来的片数变了同一片号落在同一把键上，但定额度只认片数与这一趟相同的记录（上面「资源记录」的定额度）；计划那一步的键另带这一步编不编（「〈task-server 分片 plan 编〉」「〈task-server 分片 plan 不编〉」，
    shard_child_key），编与不编各记各的；计划那一步的父命令另把缩域变量 SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO 的赋值剥掉（key_words 的 plan_step；缩域深度只改各片判多少崩溃状态，
    计划只编、只切片，不随它变），各片的键仍带它；经 RUN_WITH_MEMORY_CAP_KEY 交给内存包装；
    加的那一段不带 /，键的换算不折它，各片各一个键；这件的键：父任务给了 RUN_WITH_MEMORY_CAP_KEY 的照它，不然父命令照词折叠（memory_peaks_lib.py 的 command_key 收词表，
    带空格的条目名、路径仍是一个词，不再用空格接起来再切），截到给加的那一段留出长度为止，加的那一段不被键长上限截掉（task_server.py 的 create_child；
    弄坏开关 shard-key-from-joined-command 下照改前用空格接父命令的词）；各片只照自己的键定额度，不借计划那一步与兄弟片的峰值，上面「资源记录」）、
    各自挑机器、各自重试；这件（父任务）自己不起，整件的结局等子任务定。
    ① 计划（plan）：服务挑台起这条命令，环境里 TASK_SERVER_SHARD_PHASE=plan、TASK_SERVER_SHARD_PLAN_DIRECTORY=<计划目录>、TASK_SERVER_SHARD_MAXIMUM=<MAXIMUM_SHARDS>；命令往计划目录写 plan.json：
       {"shards": [{"label": 一句}…], "note": 一句}，片数 0 到 MAXIMUM_SHARDS；每片只认 label 一个键：写了 pin 的记 shard-plan-refused，出路写「片不钉台，哪台由服务挑」，
       写了别的键同样记 shard-plan-refused；计划目录里别的文件由命令自己定（各片读）。
    ② 每片一个子任务（片号 0 起）：环境里 TASK_SERVER_SHARD_PHASE=shard、TASK_SERVER_SHARD=<片号>/<片数>、TASK_SERVER_SHARD_PLAN_DIRECTORY、TASK_SERVER_SHARD_OUTPUT_DIRECTORY、
       TASK_SERVER_SHARD_HOST=<这一片所在那一台在 HOSTS 里的地址>（两台对称：给的是地址，不是 local / peer 这类相对的身份；服务照 host_addresses 认出自己这一台，
       派本机的给这一台的地址、派第二台的给另一台的地址；认不出的这一片不起、记 run-condition-unmet，说明写认不出的原因。弄坏开关 shard-host-role-passed 下照改前给 local / peer）。
       哪台照记录与余量挑。上限与记账照子任务自己的记录定（上面「资源记录」），撞了照「重试」放大。
       计划目录 <工作根>/SHARDS_ROOT_NAME/<库标签>-<号>/plan 与产物目录 …/output-<片号>-attempt-<第几次> 都是绝对路径、都在共享工作根下
       （Configuration.shards_directory；工作根照 .claude/hooks/lib_work_root.py 的 work_root()，在 NFS 共享根下）：本机那一片与第二台那一片照同一个绝对路径读写，
       不送、不取回。片退 0 就要建好产物目录（空的也算）。
    ③ 合并（merge）：每一片都有了结局、都是 succeeded 或人手单独取消（下面「只取消一片」）之后起，哪台照记录与余量挑（同计划与各片，两台都可能），
       环境里 TASK_SERVER_SHARD_PHASE=merge、TASK_SERVER_SHARD_PLAN_DIRECTORY、
       TASK_SERVER_SHARD_OUTPUTS=<一份 TSV：每片一行「片号<TAB>台<TAB>产物目录」>（那份 TSV 与各片的产物目录都在共享工作根下的分片根里；只列 succeeded 的那几片）、
       TASK_SERVER_SHARD_MISSING=<缺的片号，逗号连；一片不缺是空串>（合并那一步总设它；缺的是人手单独取消、没有产物的那几片，TSV 里没有它们的行）。
       缺片时合并该判「本次未跑」（退 77）或红，点名缺的片，由入口判；不当成全了判绿。派第二台的那一次
       这几个变量同样带过去：task_server.py 的 PeerMachine.command 拼第二台环境时的 carried_names 里有 TASK_SERVER_SHARD_OUTPUTS 与 TASK_SERVER_SHARD_MISSING
       （弄坏开关 shard-outputs-not-carried 下不带前一个，shard-missing-unnamed 下合并那一步一律不设后一个）。
       整件的结局就是合并这一步的结局。
  有一步最终没成：整件的结局是那一步的错误类型与退出码，wait 末行另带 shard=<片号>/<片数>（计划、合并那一步带 step=plan、step=merge）。
    一片没成（有了结局、不是 succeeded，人手单独取消的不算）：兄弟片照排、照跑到底，结局照收
    （判过的块照写进各自的库），各片全部收齐之后父任务才记结局（取最先没成的那一片），不合并（task_server.py 的 drive_parent_phase；
    弄坏开关 shard-siblings-abandoned 下照改前当场记整件的结局，在跑的没人收）。计划写坏记 shard-plan-refused。整件成了之后删 <库标签>-<号>/ 下各片的产物目录（计划目录留着）。
  计划那一步往计划目录下放的测试二进制（SHARD_PLAN_BINARIES_NAME 那个子目录，test-binaries/）：父任务有了结局（成、败、取消、打回都算）与 task clean <父任务号> 时服务删，
    计划目录别的文件留着；它的真路径不是这一条父任务自己的计划目录下的那一份（符号链接指到别处这类）的不删、服务日志打一行；删了几份、多大与不删的原因写进操作日志
    （task_server.py 的 shard_binaries_removal；弄坏开关 shard-binaries-kept、shard-binaries-path-unchecked）。
  只取消一片：提交它的会话 task cancel <那一片的子任务号>（不是父任务号）：那一片照不分片的任务取消（在排的当场记 cancelled，在跑的停单元、回读），记 cancel_origin=session；
    别的片照跑，合并照常起、TASK_SERVER_SHARD_MISSING 点名缺它（弄坏开关 shard-cancel-ends-whole 下照改前当成一片没成，整件收尾、不合并）。
    服务自己撤掉的片（兄弟片没成那种，cancel_origin=sibling）与整件被取消时撤掉的片（cancel_origin=parent）不算人手取消一片。
  取消整件：父任务先改记 cancelling，在排的子任务当场收尾，在跑的停单元、回读，都停了才记整件 cancelled；没停干净的再取消照样重停（task_server.py 的 cancel_sharded；
    弄坏开关 parent-cancel-spares-running 下照兄弟片没成那一路只撤在排的，在跑的不停）。
  父任务的阶段与片的计划都在库里，子任务收尾之后与每一轮对账都照库里的样子推下一步，服务在推的半途没了的重启之后接着推（task_server.py 文件头 ④）。

环境变量（客户端提交时照调用方此刻的环境分三种，每一个名字都落在其中一种，不静默丢）：
  带进去（白名单）：CARRIED_VARIABLES 与 CARRIED_PREFIXES 那几个（SINGLEFS_RUN_SCOPE、SINGLEFS_MULTI_HOST_CONFIG、PATH、
    CARGO_TARGET_DIR（仓外的）、ROCKSDB_LIB_DIR 这类，名单在下面的常量）。
  会话自带、不带进去（服务那边用它自己的同名值）：AMBIENT_VARIABLES 与 AMBIENT_PREFIXES（HOME、USER、CLAUDE_*、VSCODE_*、XDG_* 这类）。
  当场拒并点名：排队与挑机器归服务的那几个（TAKEN_OVER_VARIABLES 与 RUN_WITH_MEMORY_CAP_*、HOST_DISPATCH_* 里不在白名单的；例 RUN_WITH_MEMORY_CAP_WAIT_SECONDS）、
    选卡的 GPU_CARDS_BY_PRIORITY 与 CUDA_VISIBLE_DEVICES（卡归 GPU 批处理验证服务，文件头「卡」）、名字以 _BREAK 结尾的弄坏开关、指到仓里的编译目录、其余一切名单外的。出路：unset 它，
    或写进命令里 -- env <名>=<值> <命令…>（那样它是命令的一部分，原样跑），或往白名单里加。服务收到时照同一份名单再核一遍。
  退役的名字（RETIRED_VARIABLES：SINGLEFS_HEAVY_TESTS、GATE_MUTATION_START_OVER）：交给服务的命令不带它们
    （各自为什么写在那张表里）；环境里设了（split_environment）、命令外层写了「名=值」（env 后面的与命令位置前的裸赋值，retired_assignment；服务另照 bash / sh -c 引号里每一条命令判）
    都当场拒，说明点名它、写出路（unset 它、命令里也不写；各自的出路在那张表里）。弄坏开关 retired-variables-carried、retired-variable-assignments-accepted。

身份：服务经 unix socket 的 SO_PEERCRED 拿到调用进程号与 uid（uid 不是服务自己的拒），沿进程树往上读各祖先的 /proc/<进程号>/environ 里的 CLAUDE_CODE_SESSION_ID，
  取最外层（离 init 最近）的那一个；调用进程自己的环境不算（伪造的那一份就在那里）。读不到的记空串（人手）。
  子 agent 号：harness 给子 agent 起的 shell 的环境与进程树上都读不到能分开子 agent 的标识（批 3b 现查过：CLAUDE_CODE_SESSION_ID、CLAUDE_CODE_CHILD_SESSION、CLAUDE_PID、
  CLAUDE_EFFORT、CLAUDE_CODE_ENTRYPOINT 这几个在同一个 Claude 进程底下的各个 shell 里逐个相同；父进程就是那个 Claude 进程，命令行里没有 agent、session 一类的参数），不记。
  取消只许同一会话做。

自证（python3 task-server/task_server_lib.py --selftest；只在自证里认的变量与弄坏开关在下面）：每格一个子进程（--selftest-cell <格名> <目录>，只认自证起的），
  各自在临时目录里写一份多机配置（多机配置判法要的键写全：第二台的地址经共用取法整份判过才取得到，HOSTS 是 127.0.0.1 与一个文档用的假第二台地址；
  NFS_SHARED_ROOTS 就是这一格的临时目录，SHARED_DATA_ROOT 在它下面；TASK_SERVER_ 那几个键指到这一格的状态、运行时、密钥与编译目录）与密钥、
  起一份真的 uvicorn 挂在这一格自己的 socket 上；
  各格的 socket 目录不放在格的临时目录里（那里套几层就超了 socket 路径的上限），放在一个短的公共目录下各自的子目录里：系统临时目录（$TMPDIR）接上最长那一条
  socket 路径放得下就用它，放不下用 /tmp（task_server_selftest.py 的 short_temporary_base），经 TASK_SERVER_SELFTEST_SOCKET_ROOT 交给各格，收尾连同临时目录一起删；
  内存包装、multi-host-run.sh、cargo 换成临时目录里的假脚本（照命令里写的 fake-wrapper-exits=、fake-wrapper-exit-peak=、fake-wrapper-scene=、fake-peer-exit= 演结局，每次被起往调用记录里写一行；
  假包装拒起真的 cargo、rustc，只放行假 cargo；假第二台只认 --peer、--memory-cap、--label 与 --（照真的那样，送与取回的参数认不出、退 2），
  起自证的假分片入口或命令里有记号词 fake-peer-run 时照真的那样在这一刻的工作目录里跑（第二台在同一棵共享的树里），编译目录给这一格临时目录旁边的假第二台本地目录下的、跑完删，带 --keep-build-directory 的留着），
  两台的余量换成一份 JSON；各格的假仓根放在假共享根下（仓也在共享根下），里面有一份只有表头的登记表 .claude/gate.d/verdict-library-scripts.tsv；线程包装 capped.sh 是真的。
  尝试的 scope 一律换成假的（TASK_SERVER_FAKE_SCOPE_DIRECTORY；格里不另给的是一个空目录：峰值量不到、回收不掉的用量算 0）：尝试不在自己的单元里时
  （弄坏开关 no-detach）、自证本身套在真内存包装的 scope 里跑时（经 task-server 提交），尝试读到的都不是自证自己那一层的 scope，判定不随起法变。
  尝试单元照旧经真的 systemd-run --user 起（单元名带这一份库的随机标签），跑的只有 bash 与 python 小脚本；等放行文件的那几条至多等 60 秒自己退出。
  不起编译、不起重活、不连第二台、不碰卡。收尾停掉这一格的服务（自己起的进程号）与它起过、还在跑的单元（按库里记的单元名逐个停），删临时目录。
  各格并行，并行度取每台默认重活线程数（scripts/hosts/heavy_threads.py 的 default_heavy_threads 现读），取不出登记值（登记文件不在、写坏，SINGLEFS_THREAD_CAP 不是正整数）整份判红、不退回按核数，每格的输出与退出码各写自己的文件，按格名次序回读、数份数。
  只跑几格：TASK_SERVER_SELFTEST_CELLS=<格名,…>（单跑一格与它在全部格里跑判定相同；门禁与提交前跑的都不设）。

只在自证里认的变量（TASK_SERVER_SELFTEST_MARK 指到自证建的标记文件、内容是 task-server-selftest 才认；别处设了，服务、客户端与 control 一律拒、退 2、什么都不做）：
  TASK_SERVER_BREAK 弄坏开关；TASK_SERVER_SELFTEST_FAST 定时缩短；TASK_SERVER_SELFTEST_WORLD 假脚本写调用记录的目录（服务另在它下面的 scheduler-reads 记排队读预测与余量的每一次，与停单元那一侧停了哪几次尝试；它下面有 unit-states-unreadable 时服务读单元状态一律当读不到，task_server.py 的 unit_states；有 unit-cgroups-unreadable 时本机不经 systemd 核进程的 cgroup 根换成它下面不在的一处，
  task_server.py 的 LocalMachine.attempt_presence_readings）；
  TASK_SERVER_FAKE_RESOURCES 两台余量的 JSON（写了 recheck 的，起之前复核读的那一台照它；每台的报告里写了 disk_available 的、或 df 里写了那一台（{台: 演 df 的输出}）的
    照它判盘余量，都没写的当盘够；down 里写了那一台（{台: 一句原因}）的演它此刻用不了；memory_caps（{台: 字节}）是那台总上限另收到的数；
    每台报告的 memory 里写着那台此刻的余量 slice_margin（自证喂的定值，缺省 2G；写 null 演量不到），服务照它判；报告里写了 slice_scopes 的
    （{单元名: {unreclaimable, run_label}}，memory_peaks_lib.py 文件头「在跑的 scope」），服务照它算那台在跑的待涨量，没写的那几条记整份上限）；
  TASK_SERVER_MEMORY_WRAPPER、TASK_SERVER_PEER_RUNNER、TASK_SERVER_SYSTEMD_RUN、TASK_SERVER_CONTROL_SYSTEMCTL 换掉对应的程序；
  TASK_SERVER_FAKE_SCOPE_DIRECTORY 尝试的 scope 换成一份假 cgroup 目录（上面「自证」）；TASK_SERVER_FAKE_REPOSITORY_ROOT 判能不能派第二台时当仓根的目录；
  TASK_SERVER_SESSION_BOUNDARY_PID 往上读会话号读到这个进程号为止（不含它）；TASK_SERVER_SELFTEST_MISSING_MODULES 当这几个模块不在；
  TASK_SERVER_SELFTEST_CRASH_POINT=<before-unit|after-unit>:<文件> 那份文件在时服务在写了单元名之后、起单元之前（或起了之后）当场退出；
  TASK_SERVER_SELFTEST_FAIL_HEALTH_WRITE=<文件> 那份文件在时 /health 的试写当失败；TASK_SERVER_SELFTEST_ALLOW_TMPFS 状态目录在 tmpfs 上也不拒（临时目录可能在 tmpfs 上）；
  TASK_SERVER_SELFTEST_AGED_SECONDS 提交满这么多秒算高优先；TASK_SERVER_SELFTEST_STALL_SECONDS 日志这么多秒没涨算卡住；TASK_SERVER_SELFTEST_STALL_GRACE_SECONDS 起来后先宽限这么多秒；TASK_SERVER_SELFTEST_PLACEMENT_PAUSED=<文件> 那份文件在时排队线程照判、判能起的不起（记成判能起却没起成）；TASK_SERVER_SELFTEST_PLACEMENT_REFUSED=<文件> 那份文件在时排队线程把每条都判成不能起（演派活判法判错，看进展那一道该报）；
  TASK_SERVER_SELFTEST_SOCKET_ROOT 各格的 socket 目录建在它下面（上面「自证」）；TASK_SERVER_SELFTEST_SLOW_RESOURCES_SECONDS 读假余量每次先睡这么多秒（演余量读得慢）；
  TASK_SERVER_NFS_EXPORTS 当 /var/lib/nfs/etab 读的那份文件、TASK_SERVER_PEER_SSH 连第二台起的 ssh（上面「NFS 共享」：自证里换成假的，各格的多机配置写一行 NFS_SHARED_ROOTS
  指到格里的假共享根）；TASK_SERVER_SELFTEST_PAUSE_POINT=<start-checked|claimed|cancel-unlocked|attempt-finishing|launch-recheck|stop-attempts-read|confirm-counted>:<目录>
  目录里 arm-<点> 在时服务在那一点建 reached-<点>、等 release-<点>
  （start-checked 是判过 queued、记尝试之前，claimed 是记了尝试、起单元之前，cancel-unlocked 是取消记了 cancel_requested、放了锁、停单元之前，
  attempt-finishing 是对账收尾一次尝试的开头，
  launch-recheck 是起之前复核、现读那一台之前，stop-attempts-read 是停单元那一侧读这条任务的尝试之前，confirm-counted 是 task confirm 核过两个数、拿着会话锁改状态之前；task_server.py 的 selftest_pause）；TASK_SERVER_SELFTEST_STOP_FAILS=<文件> 那份文件在时
  下一次停单元不发 systemctl stop、删掉它（演停失败一次）；TASK_SERVER_SELFTEST_START_TIMEOUT_SECONDS 起单元的时限；TASK_SERVER_PRUNE_SERVER_CHECK 命令退 75 时
  当 prune-server/control check 起的那一份可执行文件；TASK_SERVER_GPU_SERVICE_CONTROL 开了 GPU 时 control start、check 当 gpu-verification-server/control 起的那一份可执行文件。TASK_SERVER_SELFTEST_MEMORY_FLOOR_BYTES 内存上限的下限（字节，换掉 MEMORY_FLOOR_BYTES 与弄坏开关 recorded-cap-floored 下的 MEMORY_FLOOR_RAISED_BYTES；各格的小世界写 0，
  下限与开关只红点名的几格；retry-cap-not-floored、recorded-cap-not-floored、cap-hit-rerun-from-record 三格写空串、照真的下限；
  shard-cap-fitted-to-room 写 30G：把两片的上限与记账抬到预测之上，造出记满上限放不下、预测放得下的片）。
  TASK_SERVER_SELFTEST_SLICE_ROOM_FLOOR_BYTES slice 兜底的下限（字节，换掉 task_server.py 的 SLICE_ROOM_FLOOR_BYTES；各格的小世界写 0，不判）。
  TASK_SERVER_SELFTEST_UNNAMED_RUN_REFUSAL 提交时拒 ⑦ 启动测试与验证没点名（task_server.py 的 unnamed_run_refusal）：写 off 不判。各格的小世界写 off：各格交的命令演的是别的规矩，
  照真的判会被 ⑦ 先拒；判 ⑦ 的 unnamed-* 那几格写空串、照真的判。
  TASK_SERVER_SELFTEST_LONG_RUN_SECONDS「跑得久」的界（秒，换掉 LONG_RUN_SECONDS；上面「续跑」）。
  TASK_SERVER_SELFTEST_RUN_LIMIT_SECONDS 跑满两小时提醒的界（秒，换掉 task_server.py 的 RUN_LIMIT_SECONDS；上面「异常表」long-run-unconfirmed：自证不真等两小时）；
  TASK_SERVER_SELFTEST_RUN_HARD_LIMIT_SECONDS 四小时硬上限（秒，换掉 task_server.py 的 RUN_HARD_LIMIT_SECONDS；上面「异常表」over-four-hours：自证不真等四小时）；
  TASK_SERVER_SELFTEST_GPU_CARDS_FAILED 服务健康读 GPU 服务卡坏时当 gpu-verification-server/control cards-failed 起的那一份可执行文件，给了就每一轮读
  （task_server.py 的 gpu_card_failures；自证里没给就不读，不起真 control）；
  TASK_SERVER_SELFTEST_NFS_SSH_SECONDS 连第二台那一次 ssh 的时限（秒，换掉 NFS_SSH_SECONDS）、TASK_SERVER_SELFTEST_NFS_RECHECK_PAUSE_SECONDS 超时之后隔多久复查（秒，换掉 NFS_RECHECK_PAUSE_SECONDS）、TASK_SERVER_SELFTEST_NFS_SLOW_GRACE_SECONDS 复查期多长（秒，换掉 NFS_SLOW_GRACE_SECONDS；上面「NFS 共享」）。
  TASK_SERVER_SELFTEST_QUEUE_ATTENTION_SECONDS 排队过久的提醒门槛（秒，换掉 QUEUE_ATTENTION_MINUTES × 60）；
  TASK_SERVER_SELFTEST_QUIET_ATTENTION_SECONDS 日志长期不涨的提醒门槛（秒，换掉 QUIET_ATTENTION_MINUTES × 60）；
  TASK_SERVER_SELFTEST_IO_WAIT_ATTENTION_SECONDS 卡在 IO 上的提醒门槛（秒，换掉 IO_WAIT_ATTENTION_MINUTES × 60）；
  TASK_SERVER_SELFTEST_PROC_ROOT 本机读进程状态与父进程号的目录
  换成它（当 /proc 读；task_server.py 的 scope_blocked_processes：假 scope 的 cgroup.procs 里点名的进程号在它下面有一份假的 stat，演 D 状态）；TASK_SERVER_SELFTEST_ADVANCE_FAILS=<文件> 那份文件在时每一轮推分片父任务都抛一个错（演分片父任务推不动；task_server.py 的 selftest_advance_fault）。
  TASK_SERVER_SELFTEST_PROGRESS_PUSH_SECONDS 在跑的任务每满这么多秒给提交它的会话发一条进度（换掉 PROGRESS_PUSH_MINUTES × 60）；
  TASK_SERVER_SELFTEST_UNCLAIMED_WARNING_SECONDS 交任务时会话名下结局满这么多秒没人取就打严厉警告、结局没人取再推的门槛（换掉 UNCLAIMED_WARNING_MINUTES × 60）；
  TASK_SERVER_CONTROL_SYSTEMCTL 另换服务盯自己的单元在不在时起的 systemctl（task_server.py 的 own_unit_state）。
  TASK_SERVER_JOURNALCTL 认被外部信号杀掉的尝试时读本机 earlyoom 与内核日志起的 journalctl（task_server.py 文件头 ①「被外部信号杀掉」）。
  TASK_SERVER_SELFTEST_LONG_RUN_ATTENTION_SECONDS 跑得久而核没用起来的界（秒，换掉 task_server.py 的 LONG_RUN_ATTENTION_SECONDS）。
  TASK_SERVER_SELFTEST_PROCESS_LIST 任务之外的进程空转读的进程表文件（当本机的；task_server.py 的 process_listings）
  TASK_SERVER_SELFTEST_RUNAWAY_RENOTIFY_SECONDS 任务之外的进程空转再报的间隔（秒，换掉 task_server.py 的 RUNAWAY_RENOTIFY_SECONDS）
  TASK_SERVER_SELFTEST_SESSIONS_DIRECTORY 服务按会话号找收件 socket 时读的会话登记目录（换掉 ~/.claude/sessions；task_server.py 文件头 ⑥「收件地址」）；
  TASK_SERVER_SELFTEST_CONFIRM_TIMEOUT_SECONDS 交了没确认满这么多秒服务取消（换掉 task_server.py 的 CONFIRM_TIMEOUT_SECONDS）；
  TASK_SERVER_SELFTEST_CONFIRM_RULE 写 all：不分长短一律待确认（task_server.py 的 long_task_judgement；自证的小世界缺省写它，各格演确认单），写空串：只有判成长任务的待确认（真服务的规矩，long-task-*、short-task-*、confirm-on-queued-task-accepted 几格这样起服务）；
  TASK_SERVER_SELFTEST_ANOMALY_REPUSH_SECONDS 门槛不随时长涨的提醒同一道门槛推过之后再推的门槛（秒，换掉 task_server.py 的 ANOMALY_REPUSH_SECONDS）。
  TASK_SERVER_SELFTEST_CRASH_POINT 的点另有 attempt-claimed（起尝试的事务里、插了尝试、改任务之前）与分片推进的
  parent-inserted、child-finished-<plan|shard|merge>、shard-inserted、merge-outputs-written、merge-inserted。

汇总行声明（门禁 code-tooling 的 break-switch-replay 格照它认汇总行，整份复跑与只点名几条开关复跑都认得出；写法在那一格文件头「汇总行的认法」）：
break-summary-rows：「✗ task_server_lib.py 自证 … 格里 … 格不对（逐格在上面）」

弄坏开关 TASK_SERVER_BREAK=<项>（逗号连几个；只给 --selftest 证红，认不出的项自证退 2）。读开关只在 broken（在 task_server_lib_configuration.py，这一份再导出）。下面表里「task_server.py 的 X」「task_server_lib.py 的 X」点名的函数与类，按名字在同目录各份里找：task_server_constants.py、task_server_store.py、task_server_machines.py、task_server_scheduler_<部分>.py 是 task_server.py 拆出的，task_server_lib_<部分>.py 是这一份拆出的；每个名字只在一份里定义。打开后自证红在哪一格
（失败行「✗ task_server_lib.py 自证：<格名>：…」；末尾「✗ task_server_lib.py 自证 N 格里 M 格不对」是汇总）：
  TASK_SERVER_BREAK=fallback-run-when-unreachable  客户端 submit 连不上服务时退回直接执行命令（task 的 submit）
      「✗ task_server_lib.py 自证：server-unreachable：」
  TASK_SERVER_BREAK=exit-code-dropped  客户端 wait 一律退 0（task 的 wait）
      「✗ task_server_lib.py 自证：exit-code-passthrough：」、「✗ task_server_lib.py 自证：memory-cap-at-maximum：」、「✗ task_server_lib.py 自证：three-duties-two-sessions：」、「✗ task_server_lib.py 自证：shard-to-peer-by-plan：」、「✗ task_server_lib.py 自证：run-hard-limit-stopped：」
      「✗ task_server_lib.py 自证：shard-retry-isolated：」、「✗ task_server_lib.py 自证：shard-plan-failed：」、「✗ task_server_lib.py 自证：shard-merge-failed：」、「✗ task_server_lib.py 自证：pinned-cap-unreadable-refused：」、
      「✗ task_server_lib.py 自证：prune-server-unavailable-outcome：」、「✗ task_server_lib.py 自证：shard-failure-waits-for-running：」、「✗ task_server_lib.py 自证：shard-cancel-alone-merges：」、「✗ task_server_lib.py 自证：workspace-write-interrupted-reported：」
  TASK_SERVER_BREAK=wait-gives-up-on-disconnect  客户端 wait 连上过之后断线就退 78，不重连（task 的 wait）
      「✗ task_server_lib.py 自证：restart-keeps-running：」
  TASK_SERVER_BREAK=environment-unfiltered  白名单外、会话自带之外的变量也照带进去，不拒（split_environment，客户端与服务都用它）
      「✗ task_server_lib.py 自证：environment-whitelist：」
      「✗ task_server_lib.py 自证：threads-fixed：」
  TASK_SERVER_BREAK=queue-variables-carried  排队与挑机器归服务的那几个变量照带进去，不拒（split_environment）
      「✗ task_server_lib.py 自证：environment-whitelist：」
  TASK_SERVER_BREAK=retired-variables-carried  环境里设了退役的名字（RETIRED_VARIABLES）照带进去，不拒（split_environment，改前 SINGLEFS_HEAVY_TESTS 在白名单里的样子）
      「✗ task_server_lib.py 自证：retired-variables-refused：」
  TASK_SERVER_BREAK=retired-variable-assignments-accepted  命令外层写了退役名字的「名=值」照收（nested_dispatch 不调 retired_assignment）
      「✗ task_server_lib.py 自证：retired-variables-refused：」
  TASK_SERVER_BREAK=nested-dispatch-allowed  命令自己套了内存包装或派发器也照收（nested_dispatch，客户端与服务都用它）
      「✗ task_server_lib.py 自证：queued-revalidated-on-restart：」、「✗ task_server_lib.py 自证：nested-dispatch-refused：」
  TASK_SERVER_BREAK=session-from-client  会话号取调用进程自己的环境（session_of_peer）
      「✗ task_server_lib.py 自证：forged-session：」
  TASK_SERVER_BREAK=cancel-session-unchecked  服务不判取消的会话是不是提交它的会话（task_server.py 的 cancel）
      「✗ task_server_lib.py 自证：cancel-own-session-only：」、「✗ task_server_lib.py 自证：operation-log：」
  TASK_SERVER_BREAK=cancel-no-readback  服务记了取消就回，不停单元、不回读（task_server.py 的 cancel）
      「✗ task_server_lib.py 自证：cancel-own-session-only：」、「✗ task_server_lib.py 自证：stalled-attempt-stopped：」、「✗ task_server_lib.py 自证：peer-stall-judged-by-peer-cpu：」、「✗ task_server_lib.py 自证：run-hard-limit-stopped：」、「✗ task_server_lib.py 自证：run-hard-limit-why-still-stops：」、「✗ task_server_lib.py 自证：run-hard-limit-writer-stopped：」、「✗ task_server_lib.py 自证：run-hard-limit-shard-stopped：」、「✗ task_server_lib.py 自证：run-hard-limit-applies-after-restart：」、「✗ task_server_lib.py 自证：run-hard-limit-stop-retried：」
      「✗ task_server_lib.py 自证：stalled-at-cap-retried：」、「✗ task_server_lib.py 自证：cancel-sharded-running：」、「✗ task_server_lib.py 自证：cancel-start-race：」
      「✗ task_server_lib.py 自证：sharded-cancel-retried：」、「✗ task_server_lib.py 自证：start-timeout-read-back：」、「✗ task_server_lib.py 自证：parent-cancel-stops-every-shard：」、「✗ task_server_lib.py 自证：cancel-after-natural-end：」
      「✗ task_server_lib.py 自证：busy-cpu-pinned-at-cap-retried：」、「✗ task_server_lib.py 自证：peer-pinned-at-cap-busy-cpu-retried：」（这两格推的，没复跑：同 stalled-at-cap-retried，停卡住那一条走同一段停法）
      「✗ task_server_lib.py 自证：io-wait-peer-not-stalled：」（推的，没复跑：同 peer-stall-judged-by-peer-cpu，没有 D 状态与读不到的那两条照停，走同一段停法）
      「✗ task_server_lib.py 自证：cancelled-before-peer-trip-built：」（单跑这一格复跑过：任务 8993 红在这一格；取消不停单元，回执停在 not-started 的那一条照样没停下来）
      「✗ task_server_lib.py 自证：parent-cancel-waits-for-children：」、「✗ task_server_lib.py 自证：stall-stop-spares-next-attempt：」（新格，推的，没复跑）
  TASK_SERVER_BREAK=queued-cancel-left-queued  取消排着的任务只记要取消、不当场收尾，任务留在队里（task_server.py 的 cancel）
      「✗ task_server_lib.py 自证：cancel-queued：」
      「✗ task_server_lib.py 自证：cancel-start-race：」、「✗ task_server_lib.py 自证：shard-cancel-alone-merges：」
  TASK_SERVER_BREAK=finished-cancel-overwrites  取消已有结局的任务时把结局改记 cancelled（task_server.py 的 cancel）
      「✗ task_server_lib.py 自证：cancel-finished：」
  TASK_SERVER_BREAK=sharded-cancel-children-left  取消分片的父任务只给父任务记 cancelled，排着的子任务不收尾、在跑的不停（task_server.py 的 cancel）
      「✗ task_server_lib.py 自证：cancel-sharded-in-plan：」、「✗ task_server_lib.py 自证：cancel-sharded-running：」
      「✗ task_server_lib.py 自证：sharded-cancel-retried：」、「✗ task_server_lib.py 自证：parent-cancel-stops-every-shard：」
      「✗ task_server_lib.py 自证：parent-cancel-waits-for-children：」（新格，推的，没复跑）
  TASK_SERVER_BREAK=signature-unchecked  服务读回不核校验列（task_server.py 的 Store.verified）
      「✗ task_server_lib.py 自证：tampered-row-not-started：」
      「✗ task_server_lib.py 自证：association-column-verified：」、「✗ task_server_lib.py 自证：child-rows-verified：」、
      「✗ task_server_lib.py 自证：tampered-attempt-ended-once：」（新格，推的，没复跑）、「✗ task_server_lib.py 自证：running-attempts-verified-only：」（新格，推的，没复跑）
  TASK_SERVER_BREAK=tamper-unalerted  核出改过的行不写告警（task_server.py 的 Scheduler.tampered）
      「✗ task_server_lib.py 自证：tampered-row-not-started：」、「✗ task_server_lib.py 自证：tampered-attempt-ended-once：」（新格，推的，没复跑）
  TASK_SERVER_BREAK=operations-unlogged  增加与取消不写操作日志（task_server.py 的 Scheduler.operation）
      「✗ task_server_lib.py 自证：operation-log：」、「✗ task_server_lib.py 自证：pause-reported：」、「✗ task_server_lib.py 自证：pause-allow-switches-session：」
      「✗ task_server_lib.py 自证：clean-failure-explained：」、「✗ task_server_lib.py 自证：shard-binaries-removed-at-outcome：」（操作日志里没有 shard-binaries 那几行）
  TASK_SERVER_BREAK=unlisted-exit-swallowed  classify_attempt 把表外的退出码记成 succeeded
      「✗ task_server_lib.py 自证：exit-code-passthrough：」、「✗ task_server_lib.py 自证：error-types：」、「✗ task_server_lib.py 自证：three-duties-two-sessions：」、
      「✗ task_server_lib.py 自证：unfinished-peak-ignored：」、「✗ task_server_lib.py 自证：shard-retry-isolated：」、「✗ task_server_lib.py 自证：shard-plan-failed：」、「✗ task_server_lib.py 自证：shard-merge-failed：」、
      「✗ task_server_lib.py 自证：long-run-failure-noted：」、「✗ task_server_lib.py 自证：reason-line-per-type：」
      「✗ task_server_lib.py 自证：peer-command-exit-split：」、「✗ task_server_lib.py 自证：failure-scene-kept-until-collected：」、「✗ task_server_lib.py 自证：shard-failure-waits-for-running：」、「✗ task_server_lib.py 自证：workspace-write-interrupted-reported：」
  TASK_SERVER_BREAK=peer-exit-unsplit  classify_attempt 不分第二台，multi-host-run.sh 的 255 记成 command-failed
      「✗ task_server_lib.py 自证：error-types：」、「✗ task_server_lib.py 自证：peer-run-failed-retried：」、「✗ task_server_lib.py 自证：peer-unavailable-alerted：」、「✗ task_server_lib.py 自证：peer-command-exit-split：」
  TASK_SERVER_BREAK=lost-not-recorded  对不回来的尝试一直当在跑，不记 launcher-lost（task_server.py 的 reconcile）
      「✗ task_server_lib.py 自证：error-types：」
  TASK_SERVER_BREAK=cap-growth-skipped  retry_decision 撞了上限照原上限重排（不乘 1.5）
      「✗ task_server_lib.py 自证：memory-cap-at-maximum：」、「✗ task_server_lib.py 自证：loop-leak-check-on-bounce：」（后一格推的，没单跑：它靠第三次撞上限乘 1.5 到顶打回那一判）
      （memory-cap-growth、unrecorded-cap-growth、completed-after-cap-hit、stalled-at-cap-retried、ended-attempt-settled-after-restart、explicit-wait-not-starved、
      busy-cpu-pinned-at-cap-retried、peer-pinned-at-cap-busy-cpu-retried 这几格不红：retry_decision 的乘 1.5 只在到不到总上限那一判里看得出，别把这几格点进来）
  TASK_SERVER_BREAK=cap-unclamped  retry_decision 乘了 1.5 到或超过那台的总上限也照重排、不打回
      「✗ task_server_lib.py 自证：memory-cap-at-maximum：」
      「✗ task_server_lib.py 自证：squeezed-at-total-is-cap-hit：」、「✗ task_server_lib.py 自证：cap-hit-starts-above：」、「✗ task_server_lib.py 自证：pinned-peer-unavailable：」、「✗ task_server_lib.py 自证：loop-leak-check-on-bounce：」（新格，推的，没复跑）
  TASK_SERVER_BREAK=host-switch-skipped  retry_decision 在总上限上还撞时不换总上限更大的另一台，直接记 memory-cap-at-maximum
      「✗ task_server_lib.py 自证：memory-cap-at-maximum：」
      「✗ task_server_lib.py 自证：pinned-peer-unavailable：」
  TASK_SERVER_BREAK=retry-overwrites-attempts  重试那一次起的时候把这个任务前面几次尝试的记录删掉，只留这一次（覆盖前一次；重试次数另记在 outcome_counts，不受它影响）
      「✗ task_server_lib.py 自证：error-types：」、「✗ task_server_lib.py 自证：memory-cap-growth：」、「✗ task_server_lib.py 自证：run-hard-limit-cap-hit-resets：」
      「✗ task_server_lib.py 自证：memory-cap-at-maximum：」、「✗ task_server_lib.py 自证：unrecorded-cap-growth：」、「✗ task_server_lib.py 自证：retry-kinds：」、「✗ task_server_lib.py 自证：retry-keeps-earlier-attempts：」、
      「✗ task_server_lib.py 自证：retry-build-directory-reused：」、「✗ task_server_lib.py 自证：retry-goes-where-it-fits：」、「✗ task_server_lib.py 自证：stalled-at-cap-retried：」、「✗ task_server_lib.py 自证：peer-run-failed-retried：」、
      「✗ task_server_lib.py 自证：completed-after-cap-hit：」、「✗ task_server_lib.py 自证：task-log：」、「✗ task_server_lib.py 自证：ended-attempt-settled-after-restart：」
      「✗ task_server_lib.py 自证：workspace-writer-not-retried：」、「✗ task_server_lib.py 自证：retry-cap-not-floored：」、「✗ task_server_lib.py 自证：peer-command-exit-split：」
      「✗ task_server_lib.py 自证：busy-cpu-pinned-at-cap-retried：」、「✗ task_server_lib.py 自证：peer-pinned-at-cap-busy-cpu-retried：」（这两格推的，没复跑：前一次尝试的记录被删）
      「✗ task_server_lib.py 自证：stall-stop-spares-next-attempt：」（新格，推的，没复跑）、「✗ task_server_lib.py 自证：loop-leak-check-on-bounce：」（新格，推的，没复跑）、「✗ task_server_lib.py 自证：repeated-cap-hits-reminded-once：」（新格，推的，没复跑）
  TASK_SERVER_BREAK=squeeze-retry-skipped  retry_decision 被总上限挤掉（254）不重排
      「✗ task_server_lib.py 自证：retry-kinds：」、「✗ task_server_lib.py 自证：retry-keeps-earlier-attempts：」、「✗ task_server_lib.py 自证：task-log：」、「✗ task_server_lib.py 自证：pause-lets-running-finish：」、「✗ task_server_lib.py 自证：pause-allows-one-session：」
  TASK_SERVER_BREAK=server-pending-ignored  服务判放不放得下时不减自己账上在跑、还没涨到量的那一截（task_server.py 的 server_pending）：
      在跑的任务进 judge_host 的那一判只经这一处（公式 verdict_within_the_formula 另算在跑的记账，不经它，所以按公式封顶的 two-hosts-booked-within-capacity 不红）；
      凡是要“后来的被报告里的空闲减去在跑的占着、排着”“按在跑的记账均衡挑台”的格都连带红
      「✗ task_server_lib.py 自证：queue-holds-unplaceable：」、「✗ task_server_lib.py 自证：pass-booking-carried：」、「✗ task_server_lib.py 自证：unrecorded-burst-balanced：」、「✗ task_server_lib.py 自证：hundred-five-minute-tasks-balanced：」、
      「✗ task_server_lib.py 自证：peer-scope-missing-booked-at-cap：」
  TASK_SERVER_BREAK=launch-when-unplaceable  任务此刻放不下（连同哪台都放不下）也照起，起在本机（task_server.py 的 place）：
      “放不下就排着”只在 place 这一处判，凡是要后来的排着、按先后起、等某一样腾出来的格都连带红
      「✗ task_server_lib.py 自证：attention-queued-too-long：」、「✗ task_server_lib.py 自证：attention-reported-once：」、「✗ task_server_lib.py 自证：queue-holds-unplaceable：」、「✗ task_server_lib.py 自证：prediction-computed-once：」、「✗ task_server_lib.py 自证：run-hard-limit-shard-stopped：」、
      「✗ task_server_lib.py 自证：pass-reads-resources-once：」、「✗ task_server_lib.py 自证：pass-booking-carried：」、「✗ task_server_lib.py 自证：launch-rechecked：」
      「✗ task_server_lib.py 自证：starved-head-judged-by-the-formula：」、「✗ task_server_lib.py 自证：aged-task-judged-first：」
      「✗ task_server_lib.py 自证：three-duties-two-sessions：」、「✗ task_server_lib.py 自证：pinned-peer-unavailable：」、「✗ task_server_lib.py 自证：peer-disk-floor：」、「✗ task_server_lib.py 自证：admission-booked-at-cap：」、
      「✗ task_server_lib.py 自证：cpu-gate-holds-busy-host：」
      「✗ task_server_lib.py 自证：down-host-waits-not-never：」
      「✗ task_server_lib.py 自证：peer-booked-like-local：」、「✗ task_server_lib.py 自证：peer-scope-missing-booked-at-cap：」
      「✗ task_server_lib.py 自证：two-hosts-booked-within-capacity：」、「✗ task_server_lib.py 自证：build-disk-predicted：」
      「✗ task_server_lib.py 自证：shard-failure-waits-for-running：」、「✗ task_server_lib.py 自证：shard-cancel-alone-merges：」、「✗ task_server_lib.py 自证：parent-cancel-stops-every-shard：」
      「✗ task_server_lib.py 自证：recheck-judges-disk：」、「✗ task_server_lib.py 自证：recheck-judges-nfs：」（新格，推的，没复跑）
      （retry-build-directory-reused 不红，量的：它换到第二台的那一段第二台放得下，place 的 chosen 不是 None，走不到这条开关；别把这一格点进来）
  TASK_SERVER_BREAK=queue-reversed  从后到的往先到的取（task_server.py 的 place_queued）
      「✗ task_server_lib.py 自证：arrival-order：」、「✗ task_server_lib.py 自证：pass-booking-carried：」
      「✗ task_server_lib.py 自证：failed-start-not-counted：」
  TASK_SERVER_BREAK=backfill-disabled  排头放不下时后面放得下的也不起（退回严格排队；task_server.py 的 place_queued）
      「✗ task_server_lib.py 自证：three-duties-two-sessions：」
  TASK_SERVER_BREAK=inner-commands-unchecked  bash / sh -c 引号里的命令不照提交规则判（task_server.py 的 nested_problem）
      「✗ task_server_lib.py 自证：nested-dispatch-refused：」
      「✗ task_server_lib.py 自证：retired-variables-refused：」
  TASK_SERVER_BREAK=retry-pinned-to-host  撞上限重排钉在同一台（task_server_lib.py 的 retry_decision）
      「✗ task_server_lib.py 自证：retry-goes-where-it-fits：」
  TASK_SERVER_BREAK=host-specific-variables-sent-to-peer  派第二台时把预编库这类按台取值的、值是本机绝对路径而不在共享根下的变量也带过去（task_server.py 的 attempt_command）
      「✗ task_server_lib.py 自证：peer-runs-in-shared-tree：」
  TASK_SERVER_BREAK=cpu-ignored-for-stall  日志满 STALL_SECONDS 没涨就停，不看 CPU 在不在涨（task_server.py 的 check_stall）
      「✗ task_server_lib.py 自证：computing-quietly-not-stalled：」
      「✗ task_server_lib.py 自证：peer-stall-judged-by-peer-cpu：」
      「✗ task_server_lib.py 自证：busy-cpu-below-cap-keeps-running：」、「✗ task_server_lib.py 自证：attention-quiet-busy-log：」（这两格推的，没复跑：CPU 在涨也照停，wait 等到的是结局不是提醒）
  TASK_SERVER_BREAK=queued-not-revalidated  服务起来时不再核还在排的任务（task_server.py 的 revalidate_queued）
      「✗ task_server_lib.py 自证：queued-revalidated-on-restart：」、「✗ task_server_lib.py 自证：queued-whole-package-rechecked：」（新格，推的，没复跑）
  TASK_SERVER_BREAK=outer-scope-cpu-only  判「在算」只读外层那一个 scope 的 CPU，里层包装各开的 scope 不算（task_server.py 的 scope_cpu_usec）
      「✗ task_server_lib.py 自证：computing-quietly-not-stalled：」
  TASK_SERVER_BREAK=cpu-unread-counted-as-idle  窗口开头或此刻读不到 CPU 也当成 CPU 没涨（task_server.py 的 check_stall）
      「✗ task_server_lib.py 自证：peer-stall-judged-by-peer-cpu：」、「✗ task_server_lib.py 自证：cpu-unread-not-stalled：」
  TASK_SERVER_BREAK=stall-grace-skipped  尝试一起来就判卡住，不先宽限（task_server.py 的 check_stall）
      「✗ task_server_lib.py 自证：stall-waits-for-grace：」
  TASK_SERVER_BREAK=squeeze-at-total-not-cap-hit  上限已到那台总上限的那一条被内核在 slice 那一层杀，照旧记 squeezed-out 原样重排（task_server.py 的 finish）
      「✗ task_server_lib.py 自证：squeezed-at-total-is-cap-hit：」
  TASK_SERVER_BREAK=peer-starts-unthrottled  一轮在第二台起多少条都行，不按 PEER_STARTS_PER_PASS 收（task_server.py 的 place_queued_into）
      「✗ task_server_lib.py 自证：peer-starts-throttled：」
  TASK_SERVER_BREAK=peer-run-retry-skipped  第二台没跑成（peer-run-failed）不重排、直接记结局（task_server_lib.py 的 retry_decision）
      「✗ task_server_lib.py 自证：peer-run-failed-retried：」
      「✗ task_server_lib.py 自证：peer-command-exit-split：」
  TASK_SERVER_BREAK=stall-ignored  日志满 STALL_SECONDS 没涨也不停（task_server.py 的 check_stall）
      「✗ task_server_lib.py 自证：stalled-attempt-stopped：」、「✗ task_server_lib.py 自证：peer-stall-judged-by-peer-cpu：」、「✗ task_server_lib.py 自证：stalled-at-cap-retried：」
      「✗ task_server_lib.py 自证：stall-clock-from-log-mtime：」
      「✗ task_server_lib.py 自证：busy-cpu-pinned-at-cap-retried：」、「✗ task_server_lib.py 自证：peer-pinned-at-cap-busy-cpu-retried：」、「✗ task_server_lib.py 自证：attention-quiet-busy-log：」（这三格推的，没复跑：不停就一直跑到命令自己退；
      日志多久没涨照判卡住记的那一份，开关下不记，提醒也报不出）
      「✗ task_server_lib.py 自证：io-wait-local-not-stalled：」、「✗ task_server_lib.py 自证：io-wait-peer-not-stalled：」（这两格推的，没复跑：D 状态只在判卡住那一段里读，
      开关下不读、io-wait 提醒报不出；没有 D 状态与读不到的那两条不停）
      「✗ task_server_lib.py 自证：stall-stop-spares-next-attempt：」（新格，推的，没复跑）
  TASK_SERVER_BREAK=running-progress-unreported  在跑的那一次不报已跑多久、日志多久没涨、在用核数（task_server.py 的 summary）
      「✗ task_server_lib.py 自证：running-status-shows-progress：」、「✗ task_server_lib.py 自证：stall-clock-from-log-mtime：」、「✗ task_server_lib.py 自证：suspected-stall-notified：」
  TASK_SERVER_BREAK=wrapper-prefixes-allowed  命令里套 nice、timeout、capped.sh、two-logs.sh、gpu-quiesce-run.sh 照收（task_server_lib.py 的 nested_dispatch）
      「✗ task_server_lib.py 自证：nested-dispatch-refused：」
  TASK_SERVER_BREAK=resource-arguments-allowed  命令里写 --memory、--threads、--peer 这类参数照收（task_server_lib.py 的 nested_dispatch）
      「✗ task_server_lib.py 自证：queued-revalidated-on-restart：」、「✗ task_server_lib.py 自证：nested-dispatch-refused：」

  TASK_SERVER_BREAK=waiting-reason-unreported  排着的任务不报在等什么（task_server.py 的 summary 与 session_tasks）
      「✗ task_server_lib.py 自证：pause-allow-waiting-text：」、「✗ task_server_lib.py 自证：pause-reported：」、「✗ task_server_lib.py 自证：launch-rechecked：」、「✗ task_server_lib.py 自证：starved-head-judged-by-the-formula：」、
      「✗ task_server_lib.py 自证：queued-shows-waiting-reason：」、「✗ task_server_lib.py 自证：peer-disk-floor：」、「✗ task_server_lib.py 自证：cpu-gate-holds-busy-host：」
      「✗ task_server_lib.py 自证：peer-booked-like-local：」、「✗ task_server_lib.py 自证：judge-error-skips-only-that-task：」、「✗ task_server_lib.py 自证：build-disk-predicted：」
      「✗ task_server_lib.py 自证：recheck-judges-disk：」、「✗ task_server_lib.py 自证：recheck-judges-nfs：」（新格，推的，没复跑）
  TASK_SERVER_BREAK=nofile-limit-dropped  起尝试单元不带 LimitNOFILE，软上限落回用户级 systemd 的 1024（task_server.py 的 start_unit）
      「✗ task_server_lib.py 自证：attempt-unit-nofile-raised：」
  TASK_SERVER_BREAK=no-detach  尝试不起成脱离服务的单元，起成服务自己的子进程，服务停下时跟着停（task_server.py 的 start_attempt_with_build_directory、stop）
      「✗ task_server_lib.py 自证：restart-keeps-running：」、「✗ task_server_lib.py 自证：unit-start-failed：」、「✗ task_server_lib.py 自证：cancel-own-session-only：」、「✗ task_server_lib.py 自证：stall-clock-from-log-mtime：」、「✗ task_server_lib.py 自证：run-hard-limit-applies-after-restart：」、
      「✗ task_server_lib.py 自证：leftover-processes-stopped：」、「✗ task_server_lib.py 自证：sharded-cancel-retried：」、「✗ task_server_lib.py 自证：association-column-verified：」
      「✗ task_server_lib.py 自证：start-timeout-read-back：」、「✗ task_server_lib.py 自证：failed-start-not-counted：」
      「✗ task_server_lib.py 自证：stall-stop-spares-next-attempt：」（新格，推的，没复跑）
  TASK_SERVER_BREAK=start-failure-ignored  systemd-run 退非 0 也当起成了（task_server.py 的 start_attempt）
      「✗ task_server_lib.py 自证：unit-start-failed：」
      「✗ task_server_lib.py 自证：failed-start-not-counted：」
  TASK_SERVER_BREAK=start-twice-after-restart  重启之后不看单元在不在、起跑标记有没有，「起单元之前」的那一次一律当没起、再起一份（task_server.py 的 reconcile）
      「✗ task_server_lib.py 自证：start-once：」
  TASK_SERVER_BREAK=resources-unrecorded  尝试结束时不把用时之外的资源（内存峰值、核数）写进库（task_server.py 的 finish_attempt）
      「✗ task_server_lib.py 自证：resource-record：」、「✗ task_server_lib.py 自证：unmeasured-not-zero：」、「✗ task_server_lib.py 自证：unfinished-peak-ignored：」、「✗ task_server_lib.py 自证：completed-after-cap-hit：」、
      「✗ task_server_lib.py 自证：build-disk-predicted：」、「✗ task_server_lib.py 自证：peer-memory-peak-recorded：」、「✗ task_server_lib.py 自证：temporary-service-wrapper-unwrapped：」、
      「✗ task_server_lib.py 自证：short-command-peak-recorded：」、「✗ task_server_lib.py 自证：loop-leak-check-on-bounce：」（新格，推的，没复跑）
  TASK_SERVER_BREAK=record-ignored-for-cap  定上限不看服务自己的资源记录，只照没记录的打底（task_server.py 的 computed_prediction）
      「✗ task_server_lib.py 自证：attention-queued-too-long：」、「✗ task_server_lib.py 自证：attention-reported-once：」、「✗ task_server_lib.py 自证：run-hard-limit-shard-stopped：」
      「✗ task_server_lib.py 自证：memory-cap-growth：」、「✗ task_server_lib.py 自证：memory-cap-at-maximum：」、「✗ task_server_lib.py 自证：resource-record：」、
      「✗ task_server_lib.py 自证：starved-head-judged-by-the-formula：」、「✗ task_server_lib.py 自证：aged-task-judged-first：」、「✗ task_server_lib.py 自证：unmeasured-not-zero：」
      「✗ task_server_lib.py 自证：three-duties-two-sessions：」、「✗ task_server_lib.py 自证：stalled-at-cap-retried：」、「✗ task_server_lib.py 自证：aged-head-at-slice-total-starts：」、「✗ task_server_lib.py 自证：squeezed-at-total-is-cap-hit：」、
      「✗ task_server_lib.py 自证：completed-after-cap-hit：」、「✗ task_server_lib.py 自证：cap-hit-starts-above：」、「✗ task_server_lib.py 自证：shard-cap-fitted-to-room：」、「✗ task_server_lib.py 自证：pinned-peer-unavailable：」、
      「✗ task_server_lib.py 自证：pinned-cap-from-configuration：」、「✗ task_server_lib.py 自证：admission-booked-at-cap：」、「✗ task_server_lib.py 自证：whole-starts-beside-running-fragment：」
      「✗ task_server_lib.py 自证：down-host-waits-not-never：」
      「✗ task_server_lib.py 自证：retry-cap-not-floored：」、「✗ task_server_lib.py 自证：recorded-cap-not-floored：」
      「✗ task_server_lib.py 自证：margin-measured-per-host：」
      「✗ task_server_lib.py 自证：peer-booked-like-local：」、「✗ task_server_lib.py 自证：peer-scope-missing-booked-at-cap：」、「✗ task_server_lib.py 自证：judge-error-skips-only-that-task：」
      「✗ task_server_lib.py 自证：build-disk-predicted：」、「✗ task_server_lib.py 自证：peer-memory-peak-recorded：」、「✗ task_server_lib.py 自证：temporary-service-wrapper-unwrapped：」、
      「✗ task_server_lib.py 自证：shard-failure-waits-for-running：」、「✗ task_server_lib.py 自证：shard-cancel-alone-merges：」、「✗ task_server_lib.py 自证：parent-cancel-stops-every-shard：」
      「✗ task_server_lib.py 自证：busy-cpu-pinned-at-cap-retried：」、「✗ task_server_lib.py 自证：busy-cpu-below-cap-keeps-running：」
      （peer-pinned-at-cap-busy-cpu-retried 这一格不红，量的：没看记录头一次的上限 4096M，第二台报的此刻内存 15G 照样不低于它的 98%，停掉重排跑完）、「✗ task_server_lib.py 自证：loop-leak-check-on-bounce：」（新格，推的，没复跑）、
      「✗ task_server_lib.py 自证：shard-start-borrows-nothing：」（推的，没复跑：有记录的那一种也照 4096M）
      「✗ task_server_lib.py 自证：cap-hit-record-over-every-host-starts-clamped：」（推的，没复跑）
  TASK_SERVER_BREAK=unclaimed-not-listed  status 不列完成了没人取的任务（task_server.py 的 session_tasks）
      「✗ task_server_lib.py 自证：unclaimed-finished：」
      「✗ task_server_lib.py 自证：three-duties-two-sessions：」
  TASK_SERVER_BREAK=check-deps-skipped  control check 不判 fastapi、uvicorn 在不在（control 的 check）
      「✗ task_server_lib.py 自证：check-missing：」
  TASK_SERVER_BREAK=config-missing-ignored  多机配置 multi-host.env 不在时不拒、照缺省路径接着判（read_configuration）
      「✗ task_server_lib.py 自证：check-missing：」、「✗ task_server_lib.py 自证：install：」
  TASK_SERVER_BREAK=state-placement-unchecked  状态目录落在仓里、运行时目录下或 tmpfs 上也不拒（read_configuration）
      「✗ task_server_lib.py 自证：state-placement：」
  TASK_SERVER_BREAK=dry-run-writes  install --dry-run 照真装那样写文件（control 的 install）
      「✗ task_server_lib.py 自证：install：」
  TASK_SERVER_BREAK=secrets-overwritten  install 时签名密钥已在也重写（create_signing_key）
      「✗ task_server_lib.py 自证：install：」
  TASK_SERVER_BREAK=install-readback-skipped  install 起了服务不回读 /health 就退 0（control 的 install）
      「✗ task_server_lib.py 自证：install：」
  TASK_SERVER_BREAK=stop-exit-status-only  control stop 只看 systemctl stop 的退出码，不读 is-active 的输出（control 的 stop）
      「✗ task_server_lib.py 自证：control-readback：」
  TASK_SERVER_BREAK=start-readback-skipped  control start 在 systemctl start 退 0 之后就退 0，不回读 is-active 与 /health（control 的 start）
      「✗ task_server_lib.py 自证：control-start：」
  TASK_SERVER_BREAK=start-exit-status-ignored  control start 不看 systemctl start 的退出码，退非 0 也照回读的结论退（control 的 start）
      「✗ task_server_lib.py 自证：control-start：」
  TASK_SERVER_BREAK=gpu-switch-taken-as-on  control start、check 不看多机配置的 ENABLE_GPU、一律当开了 GPU，照样起 gpu-verification-server/control（control 的 gpu_enabled）
      「✗ task_server_lib.py 自证：gpu-service-untouched-when-gpu-off：」（新格，推的，没复跑）
  TASK_SERVER_BREAK=gpu-service-skipped  control start、check 开了 GPU 也不起 gpu-verification-server/control（control 的 gpu_service_follows）
      「✗ task_server_lib.py 自证：gpu-service-required-when-gpu-on：」（新格，推的，没复跑）
  TASK_SERVER_BREAK=log-tail-unbounded  客户端 log 不带 --all 也打整份，不截最后 LOG_TAIL_LINES 行（task 的 log）
      「✗ task_server_lib.py 自证：task-log：」
  TASK_SERVER_BREAK=log-all-ignored  客户端 log 带了 --all 也只打最后 LOG_TAIL_LINES 行（task 的 log）
      「✗ task_server_lib.py 自证：task-log：」
  TASK_SERVER_BREAK=log-attempt-ignored  客户端 log 不认 --attempt，一律打最后一次尝试的日志（task 的 log）
      「✗ task_server_lib.py 自证：task-log：」
  TASK_SERVER_BREAK=log-missing-exit-zero  客户端 log 在任务不在、或要的那一次尝试不在时退 0（task 的 log）
      「✗ task_server_lib.py 自证：task-log：」
  TASK_SERVER_BREAK=target-directory-unset  会编的 cargo 不另给这一次尝试的编译目录（task_server.py 的 start_attempt；命令照 cargo 默认往起它的目录下的 target 编）
      「✗ task_server_lib.py 自证：build-directory-outside-repository：」、「✗ task_server_lib.py 自证：retry-build-directory-reused：」
      「✗ task_server_lib.py 自证：temporary-service-wrapper-compiles：」、「✗ task_server_lib.py 自证：build-disk-predicted：」
      「✗ task_server_lib.py 自证：build-slot-reused-on-same-tree：」、「✗ task_server_lib.py 自证：build-slot-separate-per-tree：」、「✗ task_server_lib.py 自证：build-slot-concurrent-distinct：」、「✗ task_server_lib.py 自证：build-pool-removed-with-tree：」、「✗ task_server_lib.py 自证：clean-keeps-build-pool：」、「✗ task_server_lib.py 自证：build-disk-credits-free-slot：」（新格，推的，没复跑：都要尝试记着组里那一个编译目录）
  TASK_SERVER_BREAK=build-slot-per-attempt  每次尝试在组里新开一个编译目录：不接着用空着的、不看组满（task_server.py 的 take_build_slot、build_pool_problem）
      「✗ task_server_lib.py 自证：build-slot-reused-on-same-tree：」、「✗ task_server_lib.py 自证：retry-build-directory-reused：」、「✗ task_server_lib.py 自证：peer-retry-build-directory-reused：」、「✗ task_server_lib.py 自证：build-disk-credits-free-slot：」（新开关，推的，没复跑）
  TASK_SERVER_BREAK=build-slot-tree-ignored  组不分树：这台上各棵树的编译目录当一组拿（task_server.py 的 tree_build_slots）
      「✗ task_server_lib.py 自证：build-slot-separate-per-tree：」（新开关，推的，没复跑）
  TASK_SERVER_BREAK=build-slot-held-shared  组里占着的也照空着的给出去，同一棵树同时两条拿到同一个（task_server.py 的 held_build_slots）
      「✗ task_server_lib.py 自证：build-slot-concurrent-distinct：」（新开关，推的，没复跑）
  TASK_SERVER_BREAK=vanished-tree-pool-kept  树不在了也不删它的编译目录组，服务起来时与到点在后台那一扫都不删（task_server.py 的 sweep_vanished_tree_pools）
      「✗ task_server_lib.py 自证：build-pool-removed-with-tree：」
  TASK_SERVER_BREAK=unpooled-label-directories-kept  本机编译根下不在任何组里、照尝试标签起名的目录不删（task_server.py 的 sweep_unpooled_attempt_label_directories）
      「✗ task_server_lib.py 自证：unpooled-label-directories-removed：」、「✗ task_server_lib.py 自证：unpooled-label-directory-of-running-attempt-kept：」
  TASK_SERVER_BREAK=unpooled-label-directory-open-unchecked  删那种目录之前不核有没有进程开着里面的文件（同上）
      「✗ task_server_lib.py 自证：unpooled-label-directories-removed：」
  TASK_SERVER_BREAK=unpooled-label-directories-pool-unread  不看组：组里记着的、照尝试标签起名的目录也当不在组里删（同上）
      「✗ task_server_lib.py 自证：unpooled-label-directories-removed：」（新开关，推的，没复跑）
  TASK_SERVER_BREAK=unpooled-label-directories-running-unread  不看在起、在跑的尝试：它们记着的、照尝试标签起名而组里没有那一行的目录也当没人用删（同上）
      「✗ task_server_lib.py 自证：unpooled-label-directory-of-running-attempt-kept：」
  TASK_SERVER_BREAK=clean-removes-build-slot  task clean 照删各次尝试用过的组里那一个编译目录（task_server.py 的 clean、clean_build_slots_of）
      「✗ task_server_lib.py 自证：clean-keeps-build-pool：」（新开关，推的，没复跑）
  TASK_SERVER_BREAK=clean-session-unchecked  task clean 不核提交它的会话（task_server.py 的 clean）
      「✗ task_server_lib.py 自证：build-directory-outside-repository：」
  TASK_SERVER_BREAK=runner-wake-skipped  attempt_runner.py 写完结果不调 /wake，结局要等主循环下一轮才记
      「✗ task_server_lib.py 自证：thirty-second-tick-prompt：」
  TASK_SERVER_BREAK=aged-task-not-prioritized  提交满两小时的不升高优先，照任务号排（task_server.py 的 aged）
      「✗ task_server_lib.py 自证：aged-task-judged-first：」
  TASK_SERVER_BREAK=starved-head-unreported  派活判排头能起却一直没起成 /health 照回 ok（task_server.py 的 starved_problem）
      「✗ task_server_lib.py 自证：starved-head-reported：」
      「✗ task_server_lib.py 自证：starved-head-judged-by-the-formula：」、「✗ task_server_lib.py 自证：health-answers-while-resources-slow：」
  TASK_SERVER_BREAK=blocked-head-counted-as-starved  派活判不能起的排头也当成判能起却没起成（task_server.py 的 starved_problem）
      「✗ task_server_lib.py 自证：blocked-head-not-starved：」、「✗ task_server_lib.py 自证：starved-head-judged-by-the-formula：」、
      「✗ task_server_lib.py 自证：health-head-cpu-gate-not-starved：」
      「✗ task_server_lib.py 自证：progress-stall-reported：」
  TASK_SERVER_BREAK=health-head-judged-apart  /health 照改前另判一套排头放不放得下（只看内存），不读派活给排头的结论（task_server.py 的 head_judged_apart；
      任务 7568、7615、2365 那样被误报）
      「✗ task_server_lib.py 自证：health-head-cpu-gate-not-starved：」
      「✗ task_server_lib.py 自证：progress-stall-reported：」
  TASK_SERVER_BREAK=progress-unwatched  不看进展：某台照公式放得下排着的最小那条却一直一条都没起，/health 照回 ok（task_server.py 的 progress_problem）
      「✗ task_server_lib.py 自证：progress-stall-reported：」
  TASK_SERVER_BREAK=progress-gates-ignored  看进展照公式放得下就算，不看 CPU 闸与编译目录占盘（task_server.py 的 progress_smallest_fitting）
      「✗ task_server_lib.py 自证：health-head-cpu-gate-not-starved：」
  TASK_SERVER_BREAK=wait-notify-skipped  记下结局时不叫醒在等的 wait（task_server.py 的 notify_finished），wait 只能等兜底那一轮
      「✗ task_server_lib.py 自证：thirty-second-tick-prompt：」
  TASK_SERVER_BREAK=in-repository-target-accepted  调用方给的编译目录指到仓里也收（target_directory_refusal，客户端与服务都用它）
      「✗ task_server_lib.py 自证：build-directory-outside-repository：」
  TASK_SERVER_BREAK=unmeasured-recorded-as-zero  量不到峰值的那几次当成 0 记进记录、占近几次的名额（task_server.py 的 finish_attempt 与 recorded_prediction）
      「✗ task_server_lib.py 自证：unmeasured-not-zero：」、「✗ task_server_lib.py 自证：short-command-peak-recorded：」（两样都没有的那一条记成 0）
      「✗ task_server_lib.py 自证：peer-memory-peak-recorded：」
  TASK_SERVER_BREAK=balance-ignored  两台都放得下时取排在最前的那台，不比剩余占比（task_server.py 的 place）
      「✗ task_server_lib.py 自证：balanced-when-both-fit：」、「✗ task_server_lib.py 自证：unrecorded-burst-balanced：」
      「✗ task_server_lib.py 自证：hundred-five-minute-tasks-balanced：」
  TASK_SERVER_BREAK=unrecorded-default-grown  没有记录的命令上限与记账都取 UNRECORDED_MEMORY_DEFAULT_BYTES 乘 CAP_GROWTH_FACTOR（6144M；task_server.py 的 unrecorded_prediction）
      「✗ task_server_lib.py 自证：unrecorded-default-small：」、「✗ task_server_lib.py 自证：unrecorded-cap-growth：」、「✗ task_server_lib.py 自证：unfinished-peak-ignored：」、
      「✗ task_server_lib.py 自证：recorded-cap-not-floored：」、「✗ task_server_lib.py 自证：temporary-service-wrapper-unwrapped：」
      「✗ task_server_lib.py 自证：history-peak-unrecorded-default：」、
  TASK_SERVER_BREAK=unrecorded-booked-above-cap  没有记录的命令上限取小缺省、记账取小缺省乘 CAP_GROWTH_FACTOR（6144M；task_server.py 的 unrecorded_prediction）
      「✗ task_server_lib.py 自证：unrecorded-default-small：」
      「✗ task_server_lib.py 自证：recorded-cap-not-floored：」
      （gap-fill-by-room-skips-length 不红，量的：整片之外空着的 28G 里每条按上限 4G 减、记账 6G 也都放得下，五条照内存那一支起；别把这一格点进来）
  TASK_SERVER_BREAK=unfinished-peaks-counted  服务自己的尝试不分结局，量到的峰值都进定额度（改前的样子；task_server.py 的 recorded_prediction）
      「✗ task_server_lib.py 自证：unfinished-peak-ignored：」
  TASK_SERVER_BREAK=cap-hit-low-peak-counted-at-cap  撞上限那一次峰值远低于上限也按上限进近几次最高（cap_hit_record_bytes：task_server.py 的 cap_hit_measurement 与 control drop-history 共用）
      「✗ task_server_lib.py 自证：cap-hit-low-peak-not-recorded-at-cap：」、「✗ task_server_lib.py 自证：control-drop-history-oversized-records：」
  TASK_SERVER_BREAK=cap-hit-not-counted  撞了这一条上限的尝试不算跑完、它的上限不进定额度（task_server.py 的 recorded_prediction）
      「✗ task_server_lib.py 自证：cap-hit-starts-above：」、「✗ task_server_lib.py 自证：judge-error-skips-only-that-task：」
      「✗ task_server_lib.py 自证：history-peak-window-raises：」
      「✗ task_server_lib.py 自证：cap-hit-record-over-every-host-starts-clamped：」（推的，没复跑）
  TASK_SERVER_BREAK=completed-peak-ignored  退 0 的尝试量到的峰值不进定额度（task_server.py 的 recorded_measurements）
      「✗ task_server_lib.py 自证：attention-queued-too-long：」、「✗ task_server_lib.py 自证：attention-reported-once：」、「✗ task_server_lib.py 自证：run-hard-limit-shard-stopped：」
      「✗ task_server_lib.py 自证：memory-cap-growth：」、「✗ task_server_lib.py 自证：memory-cap-at-maximum：」、「✗ task_server_lib.py 自证：resource-record：」、
      「✗ task_server_lib.py 自证：starved-head-judged-by-the-formula：」、「✗ task_server_lib.py 自证：aged-task-judged-first：」、「✗ task_server_lib.py 自证：unmeasured-not-zero：」
      「✗ task_server_lib.py 自证：three-duties-two-sessions：」、「✗ task_server_lib.py 自证：stalled-at-cap-retried：」、「✗ task_server_lib.py 自证：aged-head-at-slice-total-starts：」、「✗ task_server_lib.py 自证：squeezed-at-total-is-cap-hit：」、
      「✗ task_server_lib.py 自证：completed-after-cap-hit：」、「✗ task_server_lib.py 自证：cap-hit-starts-above：」、「✗ task_server_lib.py 自证：shard-cap-fitted-to-room：」、「✗ task_server_lib.py 自证：pinned-peer-unavailable：」、
      「✗ task_server_lib.py 自证：pinned-cap-from-configuration：」、「✗ task_server_lib.py 自证：admission-booked-at-cap：」、「✗ task_server_lib.py 自证：whole-starts-beside-running-fragment：」
      「✗ task_server_lib.py 自证：down-host-waits-not-never：」、「✗ task_server_lib.py 自证：retry-cap-not-floored：」
      「✗ task_server_lib.py 自证：recorded-cap-not-floored：」
      「✗ task_server_lib.py 自证：margin-measured-per-host：」
      「✗ task_server_lib.py 自证：peer-booked-like-local：」
      「✗ task_server_lib.py 自证：peer-scope-missing-booked-at-cap：」、「✗ task_server_lib.py 自证：build-disk-predicted：」
      「✗ task_server_lib.py 自证：peer-memory-peak-recorded：」、「✗ task_server_lib.py 自证：temporary-service-wrapper-unwrapped：」、「✗ task_server_lib.py 自证：shard-failure-waits-for-running：」、「✗ task_server_lib.py 自证：shard-cancel-alone-merges：」、
      「✗ task_server_lib.py 自证：parent-cancel-stops-every-shard：」
      「✗ task_server_lib.py 自证：busy-cpu-pinned-at-cap-retried：」、「✗ task_server_lib.py 自证：busy-cpu-below-cap-keeps-running：」
      （peer-pinned-at-cap-busy-cpu-retried 这一格不红，量的：记不下峰值时头一次的上限 4096M，第二台报的此刻内存 15G 照样不低于它的 98%，停掉重排跑完）、「✗ task_server_lib.py 自证：loop-leak-check-on-bounce：」（新格，推的，没复跑）、
      「✗ task_server_lib.py 自证：shard-start-borrows-nothing：」（推的，没复跑：有记录的那一种也照 4096M）
  TASK_SERVER_BREAK=peer-overflow-blocked  本机在前的任务本机此刻放不下（还不到永远放不下）时不派第二台，留在队里等本机（task_server.py 的 judge_placement）
      「✗ task_server_lib.py 自证：attention-cleanup-pending：」、「✗ task_server_lib.py 自证：temporary-service-wrapper-compiles：」、「✗ task_server_lib.py 自证：temporary-service-own-server-compiles：」、「✗ task_server_lib.py 自证：every-command-reaches-peer：」、
      「✗ task_server_lib.py 自证：peer-runs-in-shared-tree：」、「✗ task_server_lib.py 自证：peer-stall-judged-by-peer-cpu：」、「✗ task_server_lib.py 自证：peer-when-local-full：」
      「✗ task_server_lib.py 自证：source-repository-passed-to-command：」、「✗ task_server_lib.py 自证：shared-roots-passed-to-command：」、「✗ task_server_lib.py 自证：peer-admission-marked：」、
      「✗ task_server_lib.py 自证：peer-disk-floor：」、「✗ task_server_lib.py 自证：pinned-cap-by-host：」、「✗ task_server_lib.py 自证：gate-stages-sent-to-peer：」、「✗ task_server_lib.py 自证：attempt-label-passed-to-command：」、
      「✗ task_server_lib.py 自证：verdict-library-family-refused：」、「✗ task_server_lib.py 自证：peer-unavailable-alerted：」、「✗ task_server_lib.py 自证：build-disk-predicted：」、
      「✗ task_server_lib.py 自证：peer-memory-peak-recorded：」、「✗ task_server_lib.py 自证：outside-shared-path-refused：」、「✗ task_server_lib.py 自证：shared-roots-outside-refused：」、「✗ task_server_lib.py 自证：merge-on-peer-reads-shard-outputs：」、
      「✗ task_server_lib.py 自证：retry-build-directory-reused：」
      「✗ task_server_lib.py 自证：recheck-judges-disk：」、「✗ task_server_lib.py 自证：recheck-judges-nfs：」
  TASK_SERVER_BREAK=pinned-peer-unavailable-waits  钉在第二台的任务第二台此刻用不了时不判 never-fits，留在队里等（task_server.py 的 place）
      「✗ task_server_lib.py 自证：pinned-peer-unavailable：」
  TASK_SERVER_BREAK=shard-not-sent-to-peer  分片的各片也照本机在前挑（task_server.py 的 candidate_hosts）
      「✗ task_server_lib.py 自证：shard-to-peer-by-plan：」
  TASK_SERVER_BREAK=socket-path-unchecked  读路径与绑 socket 时不核 socket 路径的长度（socket_path_problem，服务、客户端、control 都用它）
      「✗ task_server_lib.py 自证：socket-path-too-long：」
  TASK_SERVER_BREAK=threads-unset  尝试的命令外面不套 capped.sh <N>（task_server.py 的 attempt_command）
      「✗ task_server_lib.py 自证：threads-fixed：」
  TASK_SERVER_BREAK=maximum-line-vague  memory-cap-at-maximum 的 wait 末行不写撞了哪台、到了多少、改什么再重投（task 的 wait）
      「✗ task_server_lib.py 自证：memory-cap-at-maximum：」
  TASK_SERVER_BREAK=sharding-unrecognized  不认要分片的命令，整件当一条普通任务起（sharded_entry）
      「✗ task_server_lib.py 自证：run-hard-limit-shard-stopped：」
      「✗ task_server_lib.py 自证：pause-allows-one-session：」、「✗ task_server_lib.py 自证：pause-holds-new-attempts：」、「✗ task_server_lib.py 自证：attention-shard-advance-failed：」、「✗ task_server_lib.py 自证：shard-to-peer-by-plan：」、
      「✗ task_server_lib.py 自证：shard-siblings-share-a-host：」、「✗ task_server_lib.py 自证：shard-cap-fitted-to-room：」、「✗ task_server_lib.py 自证：shard-retry-isolated：」、「✗ task_server_lib.py 自证：cancel-sharded-in-plan：」、
      「✗ task_server_lib.py 自证：cancel-sharded-running：」、「✗ task_server_lib.py 自证：shard-plan-failed：」、「✗ task_server_lib.py 自证：shard-merge-failed：」、「✗ task_server_lib.py 自证：unsharded-arguments：」、
      「✗ task_server_lib.py 自证：shard-host-is-address：」、「✗ task_server_lib.py 自证：sharded-progress-recovered：」
      「✗ task_server_lib.py 自证：sharded-cancel-retried：」、「✗ task_server_lib.py 自证：association-column-verified：」、「✗ task_server_lib.py 自证：merge-on-peer-reads-shard-outputs：」、「✗ task_server_lib.py 自证：shard-failure-waits-for-running：」、
      「✗ task_server_lib.py 自证：shard-cancel-alone-merges：」、「✗ task_server_lib.py 自证：parent-cancel-stops-every-shard：」、「✗ task_server_lib.py 自证：child-rows-verified：」、
      「✗ task_server_lib.py 自证：shard-child-key-folded-by-words：」
      「✗ task_server_lib.py 自证：parent-cancel-waits-for-children：」（新格，推的，没复跑）、「✗ task_server_lib.py 自证：long-task-sharded：」（新格，推的，没复跑）
  TASK_SERVER_BREAK=unsharded-arguments-sharded  第一个参数是 --selftest、--list-items、--help、-h（或参数里有 --selftest）的也照分片（first_argument_matches）
      「✗ task_server_lib.py 自证：unsharded-arguments：」、「✗ task_server_lib.py 自证：long-task-sharded：」（后一格新格，推的，没复跑：不分片的对照被当成要分片的）
  TASK_SERVER_BREAK=shard-retry-restarts-all  一片重排时把已经成了的兄弟片也重排、再跑一遍（task_server.py 的 finish_attempt）
      「✗ task_server_lib.py 自证：shard-retry-isolated：」
  TASK_SERVER_BREAK=merge-before-all-green  有一片成了就起合并，不等别的片（task_server.py 的 drive_parent_phase）
      「✗ task_server_lib.py 自证：shard-to-peer-by-plan：」、「✗ task_server_lib.py 自证：shard-siblings-share-a-host：」、「✗ task_server_lib.py 自证：shard-cap-fitted-to-room：」、「✗ task_server_lib.py 自证：shard-retry-isolated：」、
      「✗ task_server_lib.py 自证：shard-host-is-address：」、「✗ task_server_lib.py 自证：sharded-progress-recovered：」、「✗ task_server_lib.py 自证：merge-on-peer-reads-shard-outputs：」
      「✗ task_server_lib.py 自证：shard-cancel-alone-merges：」
  TASK_SERVER_BREAK=outputs-temporary-name-shared  合并读的 outputs.tsv 各驱动都写同一个固定的临时文件名再改名：两轮同时推到写 outputs.tsv 那一步时，后改名的那一轮找不到临时文件、抛 FileNotFoundError（task_server.py 的 write_outputs）
      「✗ task_server_lib.py 自证：shard-outputs-written-by-two-drivers：」
  TASK_SERVER_BREAK=shard-failure-unnamed  有一步最终没成时整件的末行不点名是哪一片、哪一步（task_server.py 的 finalize_parent）
      「✗ task_server_lib.py 自证：shard-retry-isolated：」、「✗ task_server_lib.py 自证：shard-plan-failed：」、「✗ task_server_lib.py 自证：shard-merge-failed：」
      「✗ task_server_lib.py 自证：shard-failure-waits-for-running：」、「✗ task_server_lib.py 自证：shard-cancel-alone-merges：」
  TASK_SERVER_BREAK=plan-failure-proceeds  计划那一步退非 0 也照它写的 plan.json 拆片、接着跑（task_server.py 的 drive_parent）
      「✗ task_server_lib.py 自证：shard-plan-failed：」
  TASK_SERVER_BREAK=merge-failure-succeeds  合并那一步没成，整件也记 succeeded（task_server.py 的 drive_parent）
      「✗ task_server_lib.py 自证：shard-merge-failed：」
      「✗ task_server_lib.py 自证：shard-cancel-alone-merges：」
  TASK_SERVER_BREAK=prediction-recomputed-every-pass  在排任务的内存预测每一轮都现算（扫记录），不用存的那一份（task_server.py 的 prediction_of）
      「✗ task_server_lib.py 自证：prediction-computed-once：」
  TASK_SERVER_BREAK=pending-read-per-task  余量与待涨量每排一条都现读一次，不是一轮读一次（task_server.py 的 PassView）
      「✗ task_server_lib.py 自证：pass-reads-resources-once：」
  TASK_SERVER_BREAK=pass-booking-not-carried  这一轮起了的那一条的记账不加进这一轮那台的待涨量，后面的照没加的判（task_server.py 的 PassView.book）
      「✗ task_server_lib.py 自证：pass-booking-carried：」
  TASK_SERVER_BREAK=launch-recheck-skipped  起之前不现读复核，照判的时候那一份起（task_server.py 的 launch_recheck）
      「✗ task_server_lib.py 自证：launch-rechecked：」
      「✗ task_server_lib.py 自证：recheck-judges-disk：」、「✗ task_server_lib.py 自证：recheck-judges-nfs：」（新格，推的，没复跑）
  TASK_SERVER_BREAK=stalled-at-cap-not-retried  判卡住那一刻峰值或此刻内存已到上限的也照记 stalled、不重排（task_server.py 的 judge_stall_stop）
      「✗ task_server_lib.py 自证：stalled-at-cap-retried：」、「✗ task_server_lib.py 自证：busy-cpu-pinned-at-cap-retried：」、「✗ task_server_lib.py 自证：peer-pinned-at-cap-busy-cpu-retried：」（后两格推的，没复跑）
  TASK_SERVER_BREAK=booked-up-to-slice-total  上限与记账照改前收到那台的总上限、不减余量（task_server.py 的 host_capacity）：slice 里有一点回收不掉的用量时，记满一整台的永远起不来
      squeezed-at-total-is-cap-hit 那一格里“上限已到那台总上限”照 host_capacity 比：开关下比的是总上限本身，48G 的上限比不到、照挤掉原样重排
      「✗ task_server_lib.py 自证：squeezed-at-total-is-cap-hit：」、「✗ task_server_lib.py 自证：booking-within-host-limit-every-history：」、
      「✗ task_server_lib.py 自证：cap-hit-record-over-every-host-starts-clamped：」
  TASK_SERVER_BREAK=stall-clock-reset-on-restart  服务起来对账在跑的尝试时「日志上一次涨」取此刻，不取日志的修改时刻（改前的样子；task_server.py 的 log_changed_at）
      「✗ task_server_lib.py 自证：stall-clock-from-log-mtime：」
  TASK_SERVER_BREAK=memory-cap-not-passed  不经环境 TASK_SERVER_MEMORY_CAP_BYTES 把这一次的上限交给命令（task_server.py 的 start_attempt）
      「✗ task_server_lib.py 自证：memory-cap-passed-to-command：」
  TASK_SERVER_BREAK=source-repository-not-passed  不经环境 TASK_SERVER_SOURCE_REPOSITORY 把提交方那一份仓的根交给命令、派第二台时也不带（task_server.py 的 start_attempt、attempt_command）
      「✗ task_server_lib.py 自证：source-repository-passed-to-command：」
  TASK_SERVER_BREAK=suspect-unreported  CPU 满窗口不涨也不报疑似卡住（task_server.py 的 check_suspect）
      「✗ task_server_lib.py 自证：suspected-stall-notified：」
  TASK_SERVER_BREAK=suspect-reported-every-wait  同一段疑似卡住每起一次 wait 都报（task_server.py 的 unreported_suspect）
      「✗ task_server_lib.py 自证：suspected-stall-notified：」
  TASK_SERVER_BREAK=pinned-cap-ignored  定额度不认钉值，照学到的记录定（task_server.py 的 pinned_prediction）
      「✗ task_server_lib.py 自证：pinned-cap-from-configuration：」、「✗ task_server_lib.py 自证：pinned-cap-unreadable-refused：」、「✗ task_server_lib.py 自证：pinned-cap-by-host：」、「✗ task_server_lib.py 自证：by-host-missing-host-refused：」、
      「✗ task_server_lib.py 自证：by-host-outside-address-refused：」、「✗ task_server_lib.py 自证：environment-prefix-stripped-from-key：」、「✗ task_server_lib.py 自证：retry-cap-not-floored：」、「✗ task_server_lib.py 自证：pin-cap-refused-within-margin：」
  TASK_SERVER_BREAK=pinned-cap-grown  钉值撞了上限照撞上限乘 1.5 重排，不照 memory-cap-at-maximum 打回（task_server.py 的 finish_attempt 与 pinned_prediction）
      「✗ task_server_lib.py 自证：pinned-cap-from-configuration：」
  TASK_SERVER_BREAK=pinned-cap-literal  钉值记成钉的那一刻配置里的字面数，改了配置不跟着变（task_server.py 的 pin_cap 与 pin_value）
      「✗ task_server_lib.py 自证：pinned-cap-from-configuration：」、「✗ task_server_lib.py 自证：pinned-cap-unreadable-refused：」、「✗ task_server_lib.py 自证：by-host-missing-host-refused：」、「✗ task_server_lib.py 自证：by-host-outside-address-refused：」、
      「✗ task_server_lib.py 自证：pin-cap-refused-within-margin：」
  TASK_SERVER_BREAK=pin-unreadable-predicted  钉了而多机配置的 SLICE_MEMORY_CAP_BY_HOST 读不到时照改前这一回照常预测、照起（task_server.py 的 pinned_prediction）
      「✗ task_server_lib.py 自证：pinned-cap-unreadable-refused：」
      「✗ task_server_lib.py 自证：by-host-missing-host-refused：」、「✗ task_server_lib.py 自证：by-host-outside-address-refused：」
  TASK_SERVER_BREAK=pinned-cap-this-host-only  钉值不按派到的那一台换，两台都取服务所在这一台那一项（task_server.py 的 pinned_on_host）
      「✗ task_server_lib.py 自证：pinned-cap-by-host：」
  TASK_SERVER_BREAK=by-host-missing-accepted  按台的键少了一台也照收（configured_by_host 给共用取法 multi_host_addresses.py 设 by-host-missing-unchecked）
      「✗ task_server_lib.py 自证：by-host-missing-host-refused：」
  TASK_SERVER_BREAK=by-host-outside-accepted  按台的键多一项不在 HOSTS 里的地址也照收（configured_by_host 给共用取法设 by-host-outside-unchecked）
      「✗ task_server_lib.py 自证：by-host-outside-address-refused：」
  TASK_SERVER_BREAK=shard-host-role-passed  分片的 TASK_SERVER_SHARD_HOST 照改前给 local / peer，不给地址（task_server.py 的 shard_environment）
      「✗ task_server_lib.py 自证：shard-host-is-address：」

  TASK_SERVER_BREAK=shard-cap-not-fitted  分片的片不把上限压到放得下的量（task_server.py 的 shard_cap_fitted_to_room），第二片排着
      「✗ task_server_lib.py 自证：shard-cap-fitted-to-room：」
  TASK_SERVER_BREAK=shard-key-position-folded  分片子任务的命令键照改前把片号写成 <片号>/<片数>（shard_child_key）：键的换算把它折成〈文件〉，各片落一个键、记过的峰值读不到
      「✗ task_server_lib.py 自证：shard-keys-kept-apart：」、「✗ task_server_lib.py 自证：shard-cap-fitted-to-room：」、「✗ task_server_lib.py 自证：run-hard-limit-shard-stopped：」
      「✗ task_server_lib.py 自证：shard-failure-waits-for-running：」、「✗ task_server_lib.py 自证：shard-cancel-alone-merges：」、
      「✗ task_server_lib.py 自证：parent-cancel-stops-every-shard：」
  TASK_SERVER_BREAK=booked-at-peak  准入照改前按峰值记账、在跑的待涨量按记的峰值算（task_server.py 的 booked_at_cap 与 server_pending）
      「✗ task_server_lib.py 自证：pinned-cap-from-configuration：」、「✗ task_server_lib.py 自证：pinned-cap-by-host：」、「✗ task_server_lib.py 自证：environment-prefix-stripped-from-key：」、
      「✗ task_server_lib.py 自证：retry-cap-not-floored：」、「✗ task_server_lib.py 自证：peer-scope-missing-booked-at-cap：」
      「✗ task_server_lib.py 自证：booking-within-host-limit-every-history：」、「✗ task_server_lib.py 自证：cap-hit-record-over-every-host-starts-clamped：」
  TASK_SERVER_BREAK=health-judges-head  /health 照改前现判排头（现读库与余量、另判一套），不读排队线程算好的那一句（task_server.py 的 health 与 head_judged_apart）
      「✗ task_server_lib.py 自证：health-answers-while-resources-slow：」、「✗ task_server_lib.py 自证：health-head-cpu-gate-not-starved：」
      「✗ task_server_lib.py 自证：progress-stall-reported：」

  TASK_SERVER_BREAK=pinned-cap-at-slice-total  钉值照那一台的 slice 总上限记满、不减余量（顶格的钉值比那一台单条能给的多；task_server.py 的 pin_value、pinned_on_host）
      「✗ task_server_lib.py 自证：pinned-cap-by-host：」、「✗ task_server_lib.py 自证：pin-cap-refused-within-margin：」
      （“钉值不大于 0”判的就是那一项减余量：pin_refusal 读 pin_value、host_limit 读 pinned_on_host，开关下两处都看到 2G，拒不了也打不回）
  TASK_SERVER_BREAK=margin-fallback-fixed  哪一台量不到余量时退回固定的 2G 照派（改前的样子；task_server.py 的 host_margin）
      「✗ task_server_lib.py 自证：margin-unmeasured-not-placed：」
  TASK_SERVER_BREAK=margin-unmeasured-waits  能派的每一台这一轮都不派（量不到余量、总上限不比余量大、钉值不大于 0 走同一条路）时照排着等，不记 run-condition-unmet
      （task_server.py 的 place）
      「✗ task_server_lib.py 自证：margin-unmeasured-not-placed：」、「✗ task_server_lib.py 自证：pin-cap-refused-within-margin：」（两台钉值都不大于 0 的那一条走的就是这条路）
  TASK_SERVER_BREAK=margin-local-for-both  第二台也用本机报告里的余量（不分台；task_server.py 的 host_margin）
      （“本机量不到、第二台量得到就派第二台”要两台各读各的：开关下第二台也读成量不到，两台都不派、记 run-condition-unmet）
      「✗ task_server_lib.py 自证：margin-measured-per-host：」、「✗ task_server_lib.py 自证：margin-unmeasured-not-placed：」
  TASK_SERVER_BREAK=pinned-nonpositive-accepted  钉值（那一项减那台此刻的余量）不大于 0 也照钉、照派（task_server.py 的 pin_refusal、host_limit）
      「✗ task_server_lib.py 自证：pin-cap-refused-within-margin：」
  TASK_SERVER_BREAK=ended-attempts-unsettled  对账不补收尾「最新尝试已 ended、任务还 running」的孤儿（task_server.py 的 settle_ended_attempts）
      「✗ task_server_lib.py 自证：ended-attempt-settled-after-restart：」
  TASK_SERVER_BREAK=feed-only-counted-as-writer  hook-swap.py --feed-only 也当成改主工作区、撞上限不重排（task_server.py 的 modifies_workspace；bash -c 串里那一条同样）
      「✗ task_server_lib.py 自证：workspace-writer-not-retried：」、「✗ task_server_lib.py 自证：wrapped-writer-read-only-retried：」（新格，推的，没复跑）
      「✗ task_server_lib.py 自证：wrapped-hook-swap-reminded：」、「✗ task_server_lib.py 自证：wrapped-previous-swap-named：」
  TASK_SERVER_BREAK=merge-and-verify-counted-as-writer  合入后验证 merge-and-verify.sh 也当成改主工作区、撞上限不重排（task_server.py 的 modifies_workspace）
      「✗ task_server_lib.py 自证：workspace-writer-not-retried：」（合入后验证那一条撞了上限只跑一次、打回）
  TASK_SERVER_BREAK=registry-service-column-ignored  登记表 verdict-library-scripts.tsv 第三列写 prune-server 的族也不认、提交时照拒（task_server.py 的 verdict_library_refusal）
      「✗ task_server_lib.py 自证：verdict-library-family-refused：」
  TASK_SERVER_BREAK=cpu-gate-ignored  不判 CPU 闸：负载超过核数 × CPU_OVERSUBSCRIPTION 的台照样派（task_server.py 的 cpu_judgement）
      「✗ task_server_lib.py 自证：cpu-gate-holds-busy-host：」
  TASK_SERVER_BREAK=unstarted-slice-estimated  slice 还没起的台照报告里估的总上限判，不换成多机配置里那台那一项（task_server.py 的 unstarted_slices_filled）
      「✗ task_server_lib.py 自证：unstarted-slice-uses-configured-total：」
  TASK_SERVER_BREAK=down-host-judged-never  多机配置里有一台此刻连不上时，只有它放得下的任务照改前判永远放不下（task_server.py 的 place）
      「✗ task_server_lib.py 自证：attention-queued-too-long：」、「✗ task_server_lib.py 自证：attention-reported-once：」、「✗ task_server_lib.py 自证：down-host-waits-not-never：」、「✗ task_server_lib.py 自证：shard-failure-waits-for-running：」、「✗ task_server_lib.py 自证：run-hard-limit-shard-stopped：」、
      「✗ task_server_lib.py 自证：shard-cancel-alone-merges：」、「✗ task_server_lib.py 自证：parent-cancel-stops-every-shard：」
  TASK_SERVER_BREAK=environment-prefix-kept  命令键不剥 env 与不改跑法的环境变量前缀（key_words）
      「✗ task_server_lib.py 自证：environment-prefix-stripped-from-key：」
  TASK_SERVER_BREAK=workspace-writers-retried  改主工作区的命令撞了上限、被挤掉这类本会重排的结局照原样重排（task_server.py 的 finish_attempt）
      「✗ task_server_lib.py 自证：workspace-writer-not-retried：」、「✗ task_server_lib.py 自证：wrapped-writer-shell-string-not-retried：」（新格，推的，没复跑）、
      「✗ task_server_lib.py 自证：wrapped-writer-script-body-not-retried：」（新格，推的，没复跑）
  TASK_SERVER_BREAK=recorded-cap-floored  下限抬到 4G（MEMORY_FLOOR_RAISED_BYTES）：钉值之外低于 4G 的上限与记账都兜到 4G，有记录的头一次、重排、撞过上限再交、改工作区的首趟都算
      （task_server.py 的 memory_floor、memory_floor_applied）
      「✗ task_server_lib.py 自证：recorded-cap-not-floored：」（有记录的 1300M 那一条开关下是 4096M）、
      「✗ task_server_lib.py 自证：retry-cap-not-floored：」（那一格头一次照记录取 1371M、重排 2057M，开关下都是 4096M）、
      「✗ task_server_lib.py 自证：workspace-writer-cap-has-headroom：」（别的脚本、只读打补丁、合入后验证、只喂不换钩子那几样只取下限 1024M，开关下兜到 4096M）
  TASK_SERVER_BREAK=cap-floor-skipped  不设内存上限的下限（MEMORY_FLOOR_BYTES，1024M）：记录的峰值很小的命令照记录定出几 MiB 的上限，撞了重排也从那个数爬（task_server.py 的 memory_floor）
      「✗ task_server_lib.py 自证：recorded-cap-not-floored：」（峰值 5M 的那一条开关下上限是 8M）、
      「✗ task_server_lib.py 自证：retry-cap-not-floored：」（峰值 5M 的那一条头一次 1024M、重排 1536M，开关下是 8M、12M）
  TASK_SERVER_BREAK=runaway-process-unreported  任务之外的进程空转不报（task_server.py 的 watch_runaway_processes）
      「✗ task_server_lib.py 自证：runaway-process-notified：」
  TASK_SERVER_BREAK=runaway-slice-counted  任务 slice 里的进程也当空转报（task_server_lib.py 的 runaway_processes）
      「✗ task_server_lib.py 自证：runaway-process-notified：」
  TASK_SERVER_BREAK=runaway-process-not-stopped  报满次数还在的也不停（task_server.py 的 watch_runaway_processes）
      「✗ task_server_lib.py 自证：runaway-process-stopped-after-reports：」
  TASK_SERVER_BREAK=runaway-unowned-not-stopped  认不出发起者的不终止（task_server.py 的 watch_runaway_processes）
      「✗ task_server_lib.py 自证：runaway-process-stopped-after-reports：」
  TASK_SERVER_BREAK=runaway-protected-stopped  不碰的那几类也终止（task_server.py 的 watch_runaway_processes）
      「✗ task_server_lib.py 自证：runaway-process-stopped-after-reports：」
  TASK_SERVER_BREAK=runaway-sent-to-every-session  不管是谁的进程都发给服务健康的那几个会话（task_server.py 的 runaway_targets）
      「✗ task_server_lib.py 自证：runaway-process-notified：」
  TASK_SERVER_BREAK=attention-queue-unreported  排队过久不记成要处置的提醒，wait 一直排下去不叫醒（task_server.py 的 attention_items）
      「✗ task_server_lib.py 自证：attention-queued-too-long：」、「✗ task_server_lib.py 自证：attention-reported-once：」
  TASK_SERVER_BREAK=attention-finished-unreported  任务已有结局时 wait 不带提醒（出结局时才判的那几种没人叫；task_server.py 的 wait 接口）
      「✗ task_server_lib.py 自证：workspace-write-interrupted-reported：」（后一格推的，没复跑）
  TASK_SERVER_BREAK=attention-cleanup-unreported  清理没收尾（cleanup_pending）不记成要处置的提醒（task_server.py 的 attention_items）
      「✗ task_server_lib.py 自证：attention-cleanup-pending：」
  TASK_SERVER_BREAK=attention-shard-advance-unreported  分片父任务推不动不记成要处置的提醒（task_server.py 的 drive_sharded_parents 与 attention_items）
      「✗ task_server_lib.py 自证：attention-shard-advance-failed：」
  TASK_SERVER_BREAK=attention-reported-every-wait  同一条任务同一种、同一道提醒门槛的提醒每起一次 wait 都叫（task_server.py 的 unreported_attention）
      「✗ task_server_lib.py 自证：attention-reported-once：」、「✗ task_server_lib.py 自证：reminder-thresholds-every-doubling：」（新格，推的，没复跑）、「✗ task_server_lib.py 自证：reminder-ledger-other-shape-read-afresh：」（新格，推的，没复跑）
  TASK_SERVER_BREAK=temporary-service-build-unrecognized  不带 --server-binary 的临时 prune-server 包装不当会编的算，本机不给编译目录（改前的样子；temporary_service_builds_server）
      「✗ task_server_lib.py 自证：temporary-service-wrapper-compiles：」
  TASK_SERVER_BREAK=judge-error-blocks-queue  一条这一轮判不了放不放得下时照改前挡住它后面的整队（task_server.py 的 place_queued_into）
      「✗ task_server_lib.py 自证：judge-error-skips-only-that-task：」
  TASK_SERVER_BREAK=nfs-unchecked  不判 NFS 共享，没写 NFS_SHARED_ROOTS、导出或挂载不过也照起（nfs_share_report）
      「✗ task_server_lib.py 自证：pause-allows-one-session：」、「✗ task_server_lib.py 自证：pause-allow-waiting-text：」、「✗ task_server_lib.py 自证：pause-allow-switches-session：」、「✗ task_server_lib.py 自证：resume-clears-allowed-session：」、
      「✗ task_server_lib.py 自证：pause-allow-survives-restart：」、「✗ task_server_lib.py 自证：pause-allow-reported：」、「✗ task_server_lib.py 自证：pause-holds-new-attempts：」、「✗ task_server_lib.py 自证：pause-lets-running-finish：」、
      「✗ task_server_lib.py 自证：resume-releases-queue：」、「✗ task_server_lib.py 自证：pause-reported：」、「✗ task_server_lib.py 自证：pause-not-starved：」、「✗ task_server_lib.py 自证：pause-waits-for-starting-attempt：」、
      「✗ task_server_lib.py 自证：attention-queued-too-long：」
      「✗ task_server_lib.py 自证：attention-cleanup-pending：」、「✗ task_server_lib.py 自证：attention-shard-advance-failed：」、「✗ task_server_lib.py 自证：attention-reported-once：」
      「✗ task_server_lib.py 自证：temporary-service-wrapper-compiles：」、「✗ task_server_lib.py 自证：temporary-service-own-server-compiles：」、「✗ task_server_lib.py 自证：exit-code-passthrough：」
      「✗ task_server_lib.py 自证：restart-keeps-running：」、「✗ task_server_lib.py 自证：start-once：」、「✗ task_server_lib.py 自证：queue-holds-unplaceable：」、「✗ task_server_lib.py 自证：arrival-order：」、
      「✗ task_server_lib.py 自证：prediction-computed-once：」、「✗ task_server_lib.py 自证：pass-reads-resources-once：」、「✗ task_server_lib.py 自证：pass-booking-carried：」、「✗ task_server_lib.py 自证：launch-rechecked：」、
      「✗ task_server_lib.py 自证：every-command-reaches-peer：」、「✗ task_server_lib.py 自证：error-types：」、「✗ task_server_lib.py 自证：unit-start-failed：」
      「✗ task_server_lib.py 自证：memory-cap-growth：」、「✗ task_server_lib.py 自证：memory-cap-at-maximum：」、「✗ task_server_lib.py 自证：unrecorded-default-small：」、「✗ task_server_lib.py 自证：unrecorded-cap-growth：」、
      「✗ task_server_lib.py 自证：unrecorded-burst-balanced：」、「✗ task_server_lib.py 自证：retry-kinds：」
      「✗ task_server_lib.py 自证：cancel-own-session-only：」、「✗ task_server_lib.py 自证：queued-revalidated-on-restart：」、「✗ task_server_lib.py 自证：forged-session：」、「✗ task_server_lib.py 自证：environment-whitelist：」、
      「✗ task_server_lib.py 自证：nested-dispatch-refused：」、「✗ task_server_lib.py 自证：resource-record：」、「✗ task_server_lib.py 自证：operation-log：」、「✗ task_server_lib.py 自证：unclaimed-finished：」、
      「✗ task_server_lib.py 自证：retry-keeps-earlier-attempts：」、「✗ task_server_lib.py 自证：build-directory-outside-repository：」、「✗ task_server_lib.py 自证：retry-build-directory-reused：」、「✗ task_server_lib.py 自证：thirty-second-tick-prompt：」、
      「✗ task_server_lib.py 自证：starved-head-reported：」、「✗ task_server_lib.py 自证：blocked-head-not-starved：」、「✗ task_server_lib.py 自证：starved-head-judged-by-the-formula：」、「✗ task_server_lib.py 自证：aged-task-judged-first：」、
      「✗ task_server_lib.py 自证：unmeasured-not-zero：」、「✗ task_server_lib.py 自证：retry-goes-where-it-fits：」、「✗ task_server_lib.py 自证：peer-runs-in-shared-tree：」
      「✗ task_server_lib.py 自证：clean-failure-explained：」、「✗ task_server_lib.py 自证：stalled-attempt-stopped：」、「✗ task_server_lib.py 自证：cpu-unread-not-stalled：」、「✗ task_server_lib.py 自证：running-status-shows-progress：」、
      「✗ task_server_lib.py 自证：stall-waits-for-grace：」、「✗ task_server_lib.py 自证：computing-quietly-not-stalled：」、「✗ task_server_lib.py 自证：peer-stall-judged-by-peer-cpu：」
      「✗ task_server_lib.py 自证：three-duties-two-sessions：」、「✗ task_server_lib.py 自证：hundred-five-minute-tasks-balanced：」、「✗ task_server_lib.py 自证：stalled-at-cap-retried：」
      「✗ task_server_lib.py 自证：aged-head-at-slice-total-starts：」、「✗ task_server_lib.py 自证：peer-run-failed-retried：」、「✗ task_server_lib.py 自证：squeezed-at-total-is-cap-hit：」、「✗ task_server_lib.py 自证：peer-starts-throttled：」、
      「✗ task_server_lib.py 自证：attempt-unit-nofile-raised：」、「✗ task_server_lib.py 自证：queued-shows-waiting-reason：」、「✗ task_server_lib.py 自证：unfinished-peak-ignored：」
      「✗ task_server_lib.py 自证：completed-after-cap-hit：」、「✗ task_server_lib.py 自证：cap-hit-starts-above：」、「✗ task_server_lib.py 自证：balanced-when-both-fit：」、「✗ task_server_lib.py 自证：peer-when-local-full：」、
      「✗ task_server_lib.py 自证：shard-to-peer-by-plan：」、「✗ task_server_lib.py 自证：shard-siblings-share-a-host：」、「✗ task_server_lib.py 自证：shard-cap-fitted-to-room：」、「✗ task_server_lib.py 自证：threads-fixed：」、
      「✗ task_server_lib.py 自证：shard-retry-isolated：」、「✗ task_server_lib.py 自证：task-log：」、「✗ task_server_lib.py 自证：cancel-queued：」、「✗ task_server_lib.py 自证：cancel-finished：」、
      「✗ task_server_lib.py 自证：cancel-sharded-in-plan：」、「✗ task_server_lib.py 自证：cancel-sharded-running：」、「✗ task_server_lib.py 自证：shard-plan-failed：」、「✗ task_server_lib.py 自证：shard-merge-failed：」、
      「✗ task_server_lib.py 自证：unsharded-arguments：」、「✗ task_server_lib.py 自证：pinned-peer-unavailable：」
      「✗ task_server_lib.py 自证：stall-clock-from-log-mtime：」、「✗ task_server_lib.py 自证：memory-cap-passed-to-command：」、「✗ task_server_lib.py 自证：source-repository-passed-to-command：」、「✗ task_server_lib.py 自证：shared-roots-passed-to-command：」、
      「✗ task_server_lib.py 自证：peer-admission-marked：」、「✗ task_server_lib.py 自证：peer-disk-floor：」、「✗ task_server_lib.py 自证：suspected-stall-notified：」
      「✗ task_server_lib.py 自证：pinned-cap-from-configuration：」、「✗ task_server_lib.py 自证：pinned-cap-unreadable-refused：」、「✗ task_server_lib.py 自证：pinned-cap-by-host：」
      「✗ task_server_lib.py 自证：by-host-missing-host-refused：」、「✗ task_server_lib.py 自证：by-host-outside-address-refused：」、「✗ task_server_lib.py 自证：shard-host-is-address：」、「✗ task_server_lib.py 自证：gate-stages-sent-to-peer：」、
      「✗ task_server_lib.py 自证：admission-booked-at-cap：」、「✗ task_server_lib.py 自证：health-answers-while-resources-slow：」
      「✗ task_server_lib.py 自证：leftover-processes-stopped：」、「✗ task_server_lib.py 自证：long-run-failure-noted：」、「✗ task_server_lib.py 自证：attempt-label-passed-to-command：」
      「✗ task_server_lib.py 自证：whole-starts-beside-running-fragment：」、「✗ task_server_lib.py 自证：ended-attempt-settled-after-restart：」
      「✗ task_server_lib.py 自证：cpu-gate-holds-busy-host：」、「✗ task_server_lib.py 自证：unstarted-slice-uses-configured-total：」、「✗ task_server_lib.py 自证：down-host-waits-not-never：」
      「✗ task_server_lib.py 自证：environment-prefix-stripped-from-key：」
      「✗ task_server_lib.py 自证：verdict-library-family-refused：」、「✗ task_server_lib.py 自证：workspace-writer-not-retried：」、「✗ task_server_lib.py 自证：retry-cap-not-floored：」、「✗ task_server_lib.py 自证：recorded-cap-not-floored：」、
      「✗ task_server_lib.py 自证：margin-measured-per-host：」
      「✗ task_server_lib.py 自证：margin-unmeasured-not-placed：」、「✗ task_server_lib.py 自证：pin-cap-refused-within-margin：」、「✗ task_server_lib.py 自证：peer-booked-like-local：」
      「✗ task_server_lib.py 自证：peer-scope-missing-booked-at-cap：」、「✗ task_server_lib.py 自证：two-hosts-booked-within-capacity：」、「✗ task_server_lib.py 自证：judge-error-skips-only-that-task：」
      「✗ task_server_lib.py 自证：nfs-share-required：」、「✗ task_server_lib.py 自证：nfs-share-verified：」、「✗ task_server_lib.py 自证：peer-unavailable-alerted：」、「✗ task_server_lib.py 自证：reason-line-per-type：」、
      「✗ task_server_lib.py 自证：cancel-start-race：」、「✗ task_server_lib.py 自证：sharded-progress-recovered：」、「✗ task_server_lib.py 自证：attempt-claim-atomic：」、「✗ task_server_lib.py 自证：sharded-cancel-retried：」、
      「✗ task_server_lib.py 自证：association-column-verified：」、「✗ task_server_lib.py 自证：start-timeout-read-back：」、「✗ task_server_lib.py 自证：build-disk-predicted：」、「✗ task_server_lib.py 自证：peer-command-exit-split：」、
      「✗ task_server_lib.py 自证：peer-memory-peak-recorded：」、「✗ task_server_lib.py 自证：prune-server-unavailable-outcome：」、「✗ task_server_lib.py 自证：temporary-service-wrapper-unwrapped：」、「✗ task_server_lib.py 自证：temporary-service-heavy-refused：」、
      「✗ task_server_lib.py 自证：outside-shared-path-refused：」、「✗ task_server_lib.py 自证：shared-roots-outside-refused：」、「✗ task_server_lib.py 自证：nfs-probe-concurrent-runs：」、
      「✗ task_server_lib.py 自证：short-command-peak-recorded：」、「✗ task_server_lib.py 自证：merge-on-peer-reads-shard-outputs：」、「✗ task_server_lib.py 自证：stop-ends-probe-children：」
      「✗ task_server_lib.py 自证：failure-scene-kept-until-collected：」、「✗ task_server_lib.py 自证：shard-failure-waits-for-running：」、「✗ task_server_lib.py 自证：shard-cancel-alone-merges：」、「✗ task_server_lib.py 自证：parent-cancel-stops-every-shard：」、
      「✗ task_server_lib.py 自证：cancel-after-natural-end：」、「✗ task_server_lib.py 自证：failed-start-not-counted：」、「✗ task_server_lib.py 自证：child-rows-verified：」、
      「✗ task_server_lib.py 自证：shard-child-key-folded-by-words：」、「✗ task_server_lib.py 自证：workspace-write-interrupted-reported：」、
      「✗ task_server_lib.py 自证：workspace-write-complete-not-reported：」
      「✗ task_server_lib.py 自证：busy-cpu-pinned-at-cap-retried：」、「✗ task_server_lib.py 自证：busy-cpu-below-cap-keeps-running：」、「✗ task_server_lib.py 自证：peer-pinned-at-cap-busy-cpu-retried：」、
      「✗ task_server_lib.py 自证：attention-quiet-busy-log：」、（最后一格推的，没复跑：
      （retired-outcome-status-generic 不红，量的：它往库里直接插一行旧任务、只跑 task status，不交任务，走不到 NFS 共享那一判；别把这一格点进来）
      量的那一趟它因别的原因红着，看不出这条开关红不红它；起服务提交任务的格在这条开关下都红）
      「✗ task_server_lib.py 自证：whole-workspace-cargo-refused：」、
      「✗ task_server_lib.py 自证：inbox-outcome-succeeded：」、「✗ task_server_lib.py 自证：inbox-outcome-failed：」、「✗ task_server_lib.py 自证：inbox-queued-too-long-once：」、「✗ task_server_lib.py 自证：inbox-unreachable-task-runs：」、「✗ task_server_lib.py 自证：unclaimed-results-warned：」、
      「✗ task_server_lib.py 自证：health-database-unwritable-notified：」、「✗ task_server_lib.py 自证：health-unit-missing-notified：」、「✗ task_server_lib.py 自证：health-nfs-notified：」、「✗ task_server_lib.py 自证：health-peer-unavailable-notified：」、「✗ task_server_lib.py 自证：health-gpu-card-failed-notified：」（新格，推的，没复跑）
      「✗ task_server_lib.py 自证：cancelled-before-peer-trip-built：」（推的，没复跑：同上，起服务提交任务）
      「✗ task_server_lib.py 自证：peer-health-slow-rechecked：」（新格，推的，没复跑）
      「✗ task_server_lib.py 自证：parent-cancel-waits-for-children：」、「✗ task_server_lib.py 自证：recheck-judges-disk：」、「✗ task_server_lib.py 自证：recheck-judges-nfs：」、「✗ task_server_lib.py 自证：stall-stop-spares-next-attempt：」（新格，推的，没复跑）、「✗ task_server_lib.py 自证：loop-leak-check-on-bounce：」（新格，推的，没复跑）、「✗ task_server_lib.py 自证：repeated-cap-hits-reminded-once：」（新格，推的，没复跑）
      「✗ task_server_lib.py 自证：build-slot-reused-on-same-tree：」、「✗ task_server_lib.py 自证：build-slot-separate-per-tree：」、「✗ task_server_lib.py 自证：build-slot-concurrent-distinct：」、「✗ task_server_lib.py 自证：build-pool-removed-with-tree：」、「✗ task_server_lib.py 自证：clean-keeps-build-pool：」、「✗ task_server_lib.py 自证：build-disk-credits-free-slot：」（新格，推的，没复跑：起服务提交任务）
  TASK_SERVER_BREAK=nfs-probe-one-way  两向探针只做本机→第二台那一向，第二台写的本机读不回也照过（nfs_share_report）
      「✗ task_server_lib.py 自证：nfs-share-verified：」
  TASK_SERVER_BREAK=nfs-probe-at-root  根是 git 工作树也把探针写在根下（nfs_probe_directory），红在 nfs-share-verified 一格
      「✗ task_server_lib.py 自证：nfs-share-verified：」
  TASK_SERVER_BREAK=nfs-probe-shared-name  两向探针照改前用固定文件名（几趟 control check 同时跑时一趟收尾删掉另一趟要读的那一份，nfs_probe）
      「✗ task_server_lib.py 自证：nfs-probe-concurrent-runs：」
  TASK_SERVER_BREAK=quoted-word-paths-ignored  bash -c 这类整条写在一个词里的不认词里的绝对路径，词里点到共享根外的也照收（任务 2420 在第二台退 127 的那一种；task_server.py 的 find_outside_shared_path）
      「✗ task_server_lib.py 自证：outside-shared-path-refused：」
  TASK_SERVER_BREAK=expansion-tail-as-path  bash -c 词里紧跟在 ${…}、$(…) 后面的 /… 也当写死的绝对路径（改前的样子，别的会话报的误拒；task_server.py 的 shell_word_paths）
      「✗ task_server_lib.py 自证：outside-shared-path-refused：」
  TASK_SERVER_BREAK=peer-unavailable-silent  第二台用不了不记、不写告警，/health 的 peer 一栏照报用得了（task_server.py 的 note_peer）
      「✗ task_server_lib.py 自证：peer-unavailable-alerted：」
      「✗ task_server_lib.py 自证：peer-disk-floor：」
  TASK_SERVER_BREAK=reason-line-omitted  wait 末行之前不打 TASK_SERVER_REASON 那一行（task 的 wait）
      「✗ task_server_lib.py 自证：pinned-cap-unreadable-refused：」
      「✗ task_server_lib.py 自证：reason-line-per-type：」、「✗ task_server_lib.py 自证：loop-leak-check-on-bounce：」（新格，推的，没复跑）
  TASK_SERVER_BREAK=clean-failure-unexplained  task clean 没删成时只说没删干净，不写是哪一份、为什么，操作日志也不写原因（task 的 clean、task_server.py 的 clean）
      「✗ task_server_lib.py 自证：clean-failure-explained：」
  TASK_SERVER_BREAK=shared-roots-not-passed  不经环境 SINGLEFS_DATA_ROOT、SINGLEFS_WORK_ROOT 把两个共享根交给命令，派第二台时也不带（改前的样子；task_server.py 的 start_attempt、attempt_command）
      「✗ task_server_lib.py 自证：shared-roots-passed-to-command：」
  TASK_SERVER_BREAK=peer-admission-unmarked  派第二台的那一次不给 multi-host-run.sh --peer 设 PEER_ADMITTED_VARIABLE（改前的样子；task_server.py 的 start_attempt）
      「✗ task_server_lib.py 自证：peer-admission-marked：」
  TASK_SERVER_BREAK=disk-floor-unchecked  不看各台盘余量，不到下限也照派（改前的样子；task_server.py 的 Resources.disk_problem）
      「✗ task_server_lib.py 自证：peer-disk-floor：」
      「✗ task_server_lib.py 自证：recheck-judges-disk：」（新格，推的，没复跑）
  TASK_SERVER_BREAK=peer-tree-copied  派第二台照改前拷树：加 --send <工作目录> <树里的落点>、命令换到落点里起（task_server.py 的 PeerMachine.command；假第二台照真的那样认不出 --send、退 2）
      「✗ task_server_lib.py 自证：attention-queued-too-long：」、「✗ task_server_lib.py 自证：attention-cleanup-pending：」、「✗ task_server_lib.py 自证：temporary-service-wrapper-compiles：」、「✗ task_server_lib.py 自证：temporary-service-own-server-compiles：」、
      「✗ task_server_lib.py 自证：every-command-reaches-peer：」、「✗ task_server_lib.py 自证：error-types：」、「✗ task_server_lib.py 自证：memory-cap-at-maximum：」、「✗ task_server_lib.py 自证：retry-goes-where-it-fits：」、
      「✗ task_server_lib.py 自证：peer-runs-in-shared-tree：」、「✗ task_server_lib.py 自证：peer-stall-judged-by-peer-cpu：」
      「✗ task_server_lib.py 自证：hundred-five-minute-tasks-balanced：」、「✗ task_server_lib.py 自证：peer-run-failed-retried：」、「✗ task_server_lib.py 自证：peer-starts-throttled：」、「✗ task_server_lib.py 自证：peer-when-local-full：」、
      「✗ task_server_lib.py 自证：shard-to-peer-by-plan：」、「✗ task_server_lib.py 自证：shard-siblings-share-a-host：」、「✗ task_server_lib.py 自证：cancel-sharded-running：」、「✗ task_server_lib.py 自证：memory-cap-passed-to-command：」、
      「✗ task_server_lib.py 自证：source-repository-passed-to-command：」、「✗ task_server_lib.py 自证：shared-roots-passed-to-command：」、「✗ task_server_lib.py 自证：peer-admission-marked：」
      「✗ task_server_lib.py 自证：peer-disk-floor：」、「✗ task_server_lib.py 自证：shard-host-is-address：」、「✗ task_server_lib.py 自证：gate-stages-sent-to-peer：」、「✗ task_server_lib.py 自证：attempt-label-passed-to-command：」、
      「✗ task_server_lib.py 自证：down-host-waits-not-never：」、「✗ task_server_lib.py 自证：verdict-library-family-refused：」、「✗ task_server_lib.py 自证：margin-measured-per-host：」、「✗ task_server_lib.py 自证：margin-unmeasured-not-placed：」、
      「✗ task_server_lib.py 自证：peer-booked-like-local：」、「✗ task_server_lib.py 自证：peer-scope-missing-booked-at-cap：」、「✗ task_server_lib.py 自证：two-hosts-booked-within-capacity：」、「✗ task_server_lib.py 自证：peer-unavailable-alerted：」、
      「✗ task_server_lib.py 自证：sharded-cancel-retried：」、「✗ task_server_lib.py 自证：build-disk-predicted：」、「✗ task_server_lib.py 自证：peer-command-exit-split：」、「✗ task_server_lib.py 自证：peer-memory-peak-recorded：」、
      「✗ task_server_lib.py 自证：temporary-service-wrapper-unwrapped：」、「✗ task_server_lib.py 自证：outside-shared-path-refused：」、「✗ task_server_lib.py 自证：shared-roots-outside-refused：」、「✗ task_server_lib.py 自证：merge-on-peer-reads-shard-outputs：」、
      「✗ task_server_lib.py 自证：failed-start-not-counted：」、「✗ task_server_lib.py 自证：retry-build-directory-reused：」
      「✗ task_server_lib.py 自证：recheck-judges-disk：」、「✗ task_server_lib.py 自证：recheck-judges-nfs：」
  TASK_SERVER_BREAK=workspace-writers-local-only  改主工作区的工具照改前只在本机（task_server.py 的 host_eligible）
      「✗ task_server_lib.py 自证：peer-runs-in-shared-tree：」、「✗ task_server_lib.py 自证：every-command-reaches-peer：」
  TASK_SERVER_BREAK=cancel-race-unguarded  起单元不拿任务的锁、开头判一次 queued 之后拿调用方那份旧字典整行写回任务（改前的样子，TS-01；task_server.py 的 start_attempt）
      「✗ task_server_lib.py 自证：cancel-start-race：」、「✗ task_server_lib.py 自证：attempt-claim-atomic：」
  TASK_SERVER_BREAK=sharded-parents-unreconciled  对账不现推还没结局的分片父任务，只在子任务收尾那一刻推一次（改前的样子，TS-02；task_server.py 的 drive_sharded_parents）
      「✗ task_server_lib.py 自证：attention-shard-advance-failed：」、「✗ task_server_lib.py 自证：sharded-progress-recovered：」
  TASK_SERVER_BREAK=attempt-claim-split  插尝试与改任务分两次提交（改前的样子，TS-03；task_server.py 的 claim_attempt）
      「✗ task_server_lib.py 自证：attempt-claim-atomic：」
  TASK_SERVER_BREAK=sharded-cancel-finished-first  取消分片的整件先给父任务记结局再停子任务，再次取消见已有结局就回停了（改前的样子，TS-04；task_server.py 的 cancel）
      「✗ task_server_lib.py 自证：sharded-cancel-retried：」
      「✗ task_server_lib.py 自证：parent-cancel-waits-for-children：」（新格，推的，没复跑）
  TASK_SERVER_BREAK=association-column-unchecked  读回不比关联列 task_id 与正文（改前的样子，TS-05；task_server.py 的 Store.verified）
      「✗ task_server_lib.py 自证：association-column-verified：」、「✗ task_server_lib.py 自证：tampered-attempt-ended-once：」（新格，推的，没复跑）、
      「✗ task_server_lib.py 自证：running-attempts-verified-only：」（新格，推的，没复跑）
  TASK_SERVER_BREAK=child-rows-unverified  children_of 丢掉校验位、名下校验不过的子任务行照收（task_server.py 的 children_of：status、clean、取消分片的整件、推父任务都经它读）
      「✗ task_server_lib.py 自证：child-rows-verified：」
  TASK_SERVER_BREAK=start-timeout-as-failure  systemd-run 超时当确定没起、记 unit-start-failed（改前的样子，TS-06；task_server.py 的 start_unit）
      「✗ task_server_lib.py 自证：start-timeout-read-back：」
  TASK_SERVER_BREAK=build-disk-unpredicted  派活不看编译目录占盘的预测（改前的样子；task_server.py 的 build_disk_problem）
      「✗ task_server_lib.py 自证：build-disk-predicted：」
  TASK_SERVER_BREAK=build-disk-reuse-uncredited  编译目录占盘那一判不扣接着用的组里那一个已占的盘，照从零编算（task_server.py 的 free_build_slot_bytes）
      「✗ task_server_lib.py 自证：build-disk-credits-free-slot：」
  TASK_SERVER_BREAK=peer-step-exit-ignored  不看第二台那条命令的结束行，multi-host-run.sh 退 255 一律记 peer-run-failed（改前的样子；peer_command_exit）
      「✗ task_server_lib.py 自证：peer-command-exit-split：」
  TASK_SERVER_BREAK=nfs-mount-placeholder-accepted  「挂载」那一项照改前只看 mountpoint -q，自动挂载的 autofs 占位也算挂上（nfs_mount_verdict）
      「✗ task_server_lib.py 自证：nfs-automount-recognized：」
      「✗ task_server_lib.py 自证：stop-ends-probe-children：」
  TASK_SERVER_BREAK=peer-memory-peak-ignored  不读 MULTI_HOST_RUN_PEER_MEMORY_PEAK 那一行，第二台的尝试一律记没量到（改前的样子；task_server.py 的 finish_attempt）
      「✗ task_server_lib.py 自证：peer-memory-peak-recorded：」

  TASK_SERVER_BREAK=prune-server-exit-unsplit  命令退 75 照改前记 command-failed，不查 prune-server（classify_attempt）
      「✗ task_server_lib.py 自证：prune-server-unavailable-outcome：」
  TASK_SERVER_BREAK=temporary-service-wrapper-kept  临时 prune-server 的包装不剥，命令键与认登记表行照整条包着的命令认（改前的样子；unwrapped_temporary_service）
      「✗ task_server_lib.py 自证：temporary-service-wrapper-unwrapped：」
      「✗ task_server_lib.py 自证：temporary-service-heavy-refused：」
  TASK_SERVER_BREAK=temporary-service-key-shared  包着临时 prune-server 的命令键不接 TEMPORARY_SERVICE_KEY_MARK，与不包的那一条共用记录（第十二批甲的样子；temporary_service_wrapped）
      「✗ task_server_lib.py 自证：temporary-service-wrapper-unwrapped：」
  TASK_SERVER_BREAK=host-lists-restored  照改前放回只派一台的名单（task_server.py 的 RESTORED_HOST_LISTS：scripts/hosts/、.claude/gate.d/ 与读本机东西的入口只在本机，
      e185-r2-segment-run.sh 只在第二台），计划里写 pin 的也收（task_server.py 的 restored_local_only、peer_only_entry 与 read_shard_plan）
      「✗ task_server_lib.py 自证：every-command-reaches-peer：」、「✗ task_server_lib.py 自证：shard-to-peer-by-plan：」、「✗ task_server_lib.py 自证：gate-stages-sent-to-peer：」
  TASK_SERVER_BREAK=data-root-key-unchecked  control install、start、check 不判多机配置的 SHARED_DATA_ROOT（control 的 require_configured_data_root）
      「✗ task_server_lib.py 自证：data-root-key-required：」
  TASK_SERVER_BREAK=outside-path-accepted  提交时不判工作目录与命令里的共享根外路径（/tmp 下的同样），照收、照派（task_server.py 的 placement_refusal 与 find_outside_shared_path）
      「✗ task_server_lib.py 自证：outside-shared-path-refused：」
  TASK_SERVER_BREAK=shared-roots-outside-accepted  提交时不判交给命令的两个共享根落不落在 NFS_SHARED_ROOTS 下，照收（task_server.py 的 placement_refusal 与 shared_roots_problem）
      「✗ task_server_lib.py 自证：shared-roots-outside-refused：」
  TASK_SERVER_BREAK=verdict-library-family-accepted  提交时不判带判定库的命令，第三列没写 prune-server 的族与判不了的也照收（task_server.py 的 placement_refusal 与 verdict_library_refusal）
      「✗ task_server_lib.py 自证：verdict-library-family-refused：」
  TASK_SERVER_BREAK=temporary-service-heavy-accepted  提交时不判临时 prune-server 包装里的原命令，包着会编的 cargo 的照收（task_server.py 的 placement_refusal 与 temporary_service_heavy_refusal）
      「✗ task_server_lib.py 自证：temporary-service-heavy-refused：」
  TASK_SERVER_BREAK=leftover-processes-kept  尝试正常结束之后不回读单元与外层 scope 的 cgroup，命令留下的孙进程不停、结局说明不写（task_server.py 的 stop_leftovers）
      「✗ task_server_lib.py 自证：leftover-processes-stopped：」
  TASK_SERVER_BREAK=leftovers-outside-own-unit  attempt_runner 停留下的进程时不核自己待在这一次尝试的单元里（改前的样子：没起成单元时连起服务的那一方一起杀，任务 5662；attempt_runner.py 的 stop_unit_leftovers）
      「✗ task_server_lib.py 自证：leftovers-only-in-own-unit：」
  TASK_SERVER_BREAK=long-run-not-noted  跑超过 LONG_RUN_SECONDS 而失败、没登记续跑的，结局说明不写重交会从头跑（task_server.py 的 finished_preface）
      「✗ task_server_lib.py 自证：long-run-failure-noted：」
  TASK_SERVER_BREAK=resumable-declaration-ignored  不读脚本文件头的 # task-server-resumable:，登记了续跑的也当没登记（resumable_command）
      「✗ task_server_lib.py 自证：long-run-failure-noted：」
  TASK_SERVER_BREAK=attempt-label-not-passed  不经环境 TASK_SERVER_ATTEMPT_LABEL 把尝试标签交给命令，两台的 scope 对不到尝试上（task_server.py 的 start_attempt）
      「✗ task_server_lib.py 自证：attempt-label-passed-to-command：」
  TASK_SERVER_BREAK=peak-read-only-by-sampling  不给内存包装 RUN_WITH_MEMORY_CAP_PEAK_OUTPUT、说明里不写 exit_peak_file，attempt_runner.py 照改前只靠按轮采（文件头「资源记录」的退出时峰值）
      「✗ task_server_lib.py 自证：short-command-peak-recorded：」
      「✗ task_server_lib.py 自证：loop-leak-check-on-bounce：」（红是量的）
  TASK_SERVER_BREAK=shard-outputs-not-carried  派第二台的合并那一次不带 TASK_SERVER_SHARD_OUTPUTS（改前的样子；task_server.py 的 PeerMachine.command 的 carried_names）
      「✗ task_server_lib.py 自证：merge-on-peer-reads-shard-outputs：」
  TASK_SERVER_BREAK=probe-children-left  停服务时不等、不停自己起的子进程（去第二台探测余量与复核挂载的 ssh 这类），留在服务单元的 cgroup 里（改前的样子；task_server.py 的 Scheduler.stop）
      「✗ task_server_lib.py 自证：stop-ends-probe-children：」
  TASK_SERVER_BREAK=scene-removed-at-finish  任务记下结局那一刻就删各次尝试的现场（文件头「现场」；task_server.py 的 finalize）
      「✗ task_server_lib.py 自证：failure-scene-kept-until-collected：」
  TASK_SERVER_BREAK=scene-kept-after-collect  task wait 取走结局之后也不删现场（task_server.py 的 sweep_scenes）
      「✗ task_server_lib.py 自证：failure-scene-kept-until-collected：」
  TASK_SERVER_BREAK=scene-keep-ignored  提交时设了 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1 的现场取走之后照删（task_server.py 的 sweep_scenes）
      「✗ task_server_lib.py 自证：failure-scene-kept-until-collected：」
  TASK_SERVER_BREAK=scene-left-by-clean  task clean 不删现场（task_server.py 的 clean）
      「✗ task_server_lib.py 自证：failure-scene-kept-until-collected：」
  TASK_SERVER_BREAK=shard-siblings-abandoned  一片没成时照改前当场记整件的结局、撤掉在排的兄弟片，在跑的兄弟片没人收（任务 5850 的样子；task_server.py 的 drive_parent_phase）
      「✗ task_server_lib.py 自证：shard-failure-waits-for-running：」
  TASK_SERVER_BREAK=shard-cancel-ends-whole  人手只取消一片时照改前当成一片没成，整件收尾、不合并（task_server.py 的 cancel 不记 cancel_origin）
      「✗ task_server_lib.py 自证：shard-cancel-alone-merges：」
  TASK_SERVER_BREAK=shard-missing-unnamed  合并那一步不设 TASK_SERVER_SHARD_MISSING，缺了哪几片不交给合并（task_server.py 的 shard_environment）
      「✗ task_server_lib.py 自证：shard-cancel-alone-merges：」
  TASK_SERVER_BREAK=parent-cancel-spares-running  取消分片的整件照兄弟片没成那一路只撤在排的子任务，在跑的不停（task_server.py 的 cancel_children）
      「✗ task_server_lib.py 自证：parent-cancel-stops-every-shard：」
      「✗ task_server_lib.py 自证：cancel-sharded-running：」、「✗ task_server_lib.py 自证：sharded-cancel-retried：」
      「✗ task_server_lib.py 自证：parent-cancel-waits-for-children：」（新格，推的，没复跑）
  TASK_SERVER_BREAK=cancel-outcome-unchecked  取消停了单元、回读过就回 stopped，不等结局写回、不核是不是 cancelled（改前的样子；task_server.py 的 cancel_outcome_reply）
      「✗ task_server_lib.py 自证：cancel-after-natural-end：」
  TASK_SERVER_BREAK=failed-start-counted  没起成的尝试也算进这一轮在那台起了几条（改前的样子；task_server.py 的 place_queued_into 的 started_on）
      「✗ task_server_lib.py 自证：failed-start-not-counted：」
  TASK_SERVER_BREAK=task-locks-kept  任务的锁用完不从 Store.task_locks 删，表随跑过的任务数涨（改前的样子；task_server.py 的 Store.task_lock）
      「✗ task_server_lib.py 自证：task-locks-released：」
  TASK_SERVER_BREAK=pause-queue-ignored  服务暂停着排队线程当没暂停，照起排着的任务（task_server.py 的 pause_in_force）
      「✗ task_server_lib.py 自证：pause-holds-new-attempts：」、「✗ task_server_lib.py 自证：pause-lets-running-finish：」、「✗ task_server_lib.py 自证：pause-allows-one-session：」、
      「✗ task_server_lib.py 自证：pause-allow-switches-session：」、「✗ task_server_lib.py 自证：resume-clears-allowed-session：」、「✗ task_server_lib.py 自证：pause-allow-survives-restart：」
  TASK_SERVER_BREAK=pause-start-ungated  pause 与起尝试不共用 start_gate：pause 返回时一条正在起的还没记下，control check 数到的在跑数不准（task_server.py 的 pause_service、start_attempt_unless_paused）
      「✗ task_server_lib.py 自证：pause-waits-for-starting-attempt：」
  TASK_SERVER_BREAK=pause-halts-reconcile  暂停着对账也停，在跑的跑完了结局也不写回（task_server.py 的 loop）
      「✗ task_server_lib.py 自证：pause-lets-running-finish：」、「✗ task_server_lib.py 自证：pause-allows-one-session：」、「✗ task_server_lib.py 自证：pause-allow-switches-session：」、
      「✗ task_server_lib.py 自证：pause-allow-survives-restart：」、「✗ task_server_lib.py 自证：pause-allow-reported：」
  TASK_SERVER_BREAK=pause-state-memory-only  暂停态只记在内存，不写库，服务重启就成了没暂停（task_server.py 的 pause_service）
      「✗ task_server_lib.py 自证：pause-survives-restart：」、「✗ task_server_lib.py 自证：pause-allow-survives-restart：」
  TASK_SERVER_BREAK=resume-leaves-cache-paused  resume 只清库里的暂停态，排队线程读的那一份还是暂停，排着的不起（task_server.py 的 resume_service）
      「✗ task_server_lib.py 自证：resume-releases-queue：」、「✗ task_server_lib.py 自证：pause-reported：」、「✗ task_server_lib.py 自证：resume-clears-allowed-session：」
  TASK_SERVER_BREAK=pause-health-unreported  /health 不带 paused 与 pause，control check 看不出暂停（task_server.py 的 health）
      「✗ task_server_lib.py 自证：pause-reported：」、「✗ task_server_lib.py 自证：pause-allow-switches-session：」、「✗ task_server_lib.py 自证：pause-allow-survives-restart：」、
      「✗ task_server_lib.py 自证：pause-allow-reported：」
  TASK_SERVER_BREAK=pause-waiting-unreported  排着的任务 status 不写「服务暂停中（原因：…）」（task_server.py 的 waiting_text）
      「✗ task_server_lib.py 自证：pause-reported：」、「✗ task_server_lib.py 自证：pause-allow-waiting-text：」
  TASK_SERVER_BREAK=pause-judges-starved  暂停中仍照改前另判排头放不放得下（暂停时排队线程不判），/health 回不健康、control check 退 78（task_server.py 的 starved_problem）
      「✗ task_server_lib.py 自证：pause-not-starved：」
  TASK_SERVER_BREAK=pause-allow-ignored  暂停着连放行的会话交的任务也不排不起，放行形同虚设（task_server.py 的 allowed_during_pause）
      「✗ task_server_lib.py 自证：pause-allows-one-session：」、「✗ task_server_lib.py 自证：pause-allow-waiting-text：」、「✗ task_server_lib.py 自证：pause-allow-switches-session：」、
      「✗ task_server_lib.py 自证：pause-allow-survives-restart：」
  TASK_SERVER_BREAK=pause-allow-matches-any-session  放行了一个会话，暂停着别的会话交的任务也照起（task_server.py 的 allowed_during_pause）
      「✗ task_server_lib.py 自证：pause-allows-one-session：」、「✗ task_server_lib.py 自证：pause-allow-waiting-text：」、「✗ task_server_lib.py 自证：pause-allow-switches-session：」、
      「✗ task_server_lib.py 自证：pause-allow-survives-restart：」
  TASK_SERVER_BREAK=pause-allow-waiting-paused  放行会话排着的任务 status 也写「服务暂停中」，不写它真在等什么（task_server.py 的 waiting_text）
      「✗ task_server_lib.py 自证：pause-allow-waiting-text：」
  TASK_SERVER_BREAK=pause-allow-change-ignored  已经暂停着再 pause 带另一个 --allow-session 不换放行的会话（task_server.py 的 pause_service）
      「✗ task_server_lib.py 自证：pause-allow-switches-session：」
  TASK_SERVER_BREAK=resume-keeps-allowed-session  resume 之后下一次不带 --allow-session 的 pause 还沿用上一次放行的会话（task_server.py 的 resume_service、pause_service）
      「✗ task_server_lib.py 自证：resume-clears-allowed-session：」
  TASK_SERVER_BREAK=pause-allow-memory-only  放行的会话只记在内存，不写进库，服务重启就没人放行了（task_server.py 的 Store.write_pause）
      「✗ task_server_lib.py 自证：pause-allow-survives-restart：」
  TASK_SERVER_BREAK=pause-allow-unsigned  放行的会话不进暂停态的签名，绕过服务改库里的放行会话读不出校验不过（task_server.py 的 Store.pause_signature）
      「✗ task_server_lib.py 自证：pause-allow-survives-restart：」
  TASK_SERVER_BREAK=pause-allow-unreported  /health 与 control check 不写放行的是哪个会话、别的会话在跑几个（task_server.py 的 health、pause_text）
      「✗ task_server_lib.py 自证：pause-allow-reported：」、「✗ task_server_lib.py 自证：pause-allow-switches-session：」、「✗ task_server_lib.py 自证：pause-allow-survives-restart：」
  TASK_SERVER_BREAK=pause-other-running-miscounted  「别的会话在跑 M 个」把放行会话的也数进去，等不到 0（task_server.py 的 running_outside_allowed）
      「✗ task_server_lib.py 自证：pause-allow-reported：」
  TASK_SERVER_BREAK=wait-limit-fixed  自证里各格的等待退回写死的时限：从开等起满安静时限就判红，不看世界还在不在变、没有绝对上限的说法（task_server_selftest.py 的 World.keep_waiting）
      「✗ task_server_lib.py 自证：slow-world-patient-wait：」
  TASK_SERVER_BREAK=temporary-service-own-server-unrecognized  prune_server_client_selftest.py、prune-server-load-simulation.py 不带二进制的 --selftest 不当会编的算（改前的样子；own_server_selftest_split）
      「✗ task_server_lib.py 自证：temporary-service-own-server-compiles：」
  TASK_SERVER_BREAK=shard-key-from-joined-command  分片子任务的键照改前用空格接父命令的词再按空格切，条目名里带空格的后半截漏进键里（task_server.py 的 create_child）
      「✗ task_server_lib.py 自证：shard-child-key-folded-by-words：」
  TASK_SERVER_BREAK=workspace-write-interrupted-unreported  改主工作区的命令没正常结束也不报（task_server.py 的 attention_of_record）
      「✗ task_server_lib.py 自证：workspace-write-interrupted-reported：」、「✗ task_server_lib.py 自证：run-hard-limit-writer-stopped：」、「✗ task_server_lib.py 自证：wrapped-writer-interrupted-judged：」、
      「✗ task_server_lib.py 自证：wrapped-writers-each-judged：」、「✗ task_server_lib.py 自证：wrapped-patch-directory-resolved：」、
      「✗ task_server_lib.py 自证：wrapped-patch-directory-unresolvable：」、「✗ task_server_lib.py 自证：hook-swap-outcome-judged：」
  TASK_SERVER_BREAK=apply-outcome-ignored  不读 apply-writer-patch.py 的结局文件，打全了的也照可能写了一半报（task_server_scheduler_lifecycle.py 的 writer_interruption）
      「✗ task_server_lib.py 自证：workspace-write-complete-not-reported：」、「✗ task_server_lib.py 自证：wrapped-writer-interrupted-judged：」、
      「✗ task_server_lib.py 自证：wrapped-writers-each-judged：」、「✗ task_server_lib.py 自证：wrapped-patch-directory-resolved：」
  TASK_SERVER_BREAK=swap-outcome-unread  被杀的换钩子不读 hook-swap.py 的结局（--read-outcome），换完了的、同一条命令里结局归另一条的也照可能写了一半点名
      （task_server_scheduler_lifecycle.py 的 hook_swap_read_back）
      「✗ task_server_lib.py 自证：hook-swap-outcome-judged：」
  TASK_SERVER_BREAK=swap-outcome-owner-ignored  被杀的换钩子读出没换完时不看输出点名的反向规格在哪条换钩子的 --scratch 下，同一条命令里的换钩子一律点名
      （task_server_scheduler_lifecycle.py 的 workspace_write_judged）
      「✗ task_server_lib.py 自证：hook-swap-outcome-judged：」
  TASK_SERVER_BREAK=at-cap-busy-cpu-kept-running  日志满窗口不涨、此刻内存贴着上限而 CPU 在涨的照改前当在算、照跑（任务 7723；task_server.py 的 judge_stall_stop）
      「✗ task_server_lib.py 自证：busy-cpu-pinned-at-cap-retried：」、「✗ task_server_lib.py 自证：peer-pinned-at-cap-busy-cpu-retried：」
  TASK_SERVER_BREAK=at-cap-fraction-ignored  判贴着上限不比上限：读得到此刻内存就当贴着上限（task_server.py 的 judge_stall_stop）
      「✗ task_server_lib.py 自证：busy-cpu-below-cap-keeps-running：」
  TASK_SERVER_BREAK=peer-memory-line-unread  第二台不读日志里的 MULTI_HOST_RUN_PEER_MEMORY 那一行，此刻内存与峰值都交 None（那一行照旧不算日志在涨；task_server.py 的 PeerMachine.memory_now）
      「✗ task_server_lib.py 自证：peer-pinned-at-cap-busy-cpu-retried：」
  TASK_SERVER_BREAK=attention-quiet-busy-unreported  日志长期不涨而服务没停它不记成要处置的提醒（task_server.py 的 quiet_log_items）
      「✗ task_server_lib.py 自证：attention-quiet-busy-log：」
  TASK_SERVER_BREAK=io-wait-ignored  判卡住不看 D 状态：日志与 CPU 都满窗口不涨、每一轮都有进程在等 IO 的也停掉、记 stalled（改前的样子；任务 8608、8609、8611、8619；
      task_server.py 的 judge_stall_stop）
      「✗ task_server_lib.py 自证：io-wait-local-not-stalled：」、「✗ task_server_lib.py 自证：io-wait-peer-not-stalled：」
  TASK_SERVER_BREAK=attention-io-wait-unreported  卡在 IO 上而服务没停它不记成要处置的提醒（task_server.py 的 io_wait_items）
      「✗ task_server_lib.py 自证：io-wait-local-not-stalled：」
  TASK_SERVER_BREAK=quiet-log-busy-any-exemption  日志长期不涨的提醒不看服务为什么没停它：按 D 状态免停的也报 quiet-log-busy（task_server.py 的 quiet_log_items）
      「✗ task_server_lib.py 自证：io-wait-local-not-stalled：」
  TASK_SERVER_BREAK=read-only-arguments-counted-as-writer  改工作区的工具带了只读的参数（apply-writer-patch.py --dry-run、--selftest 这类，task_server.py 的 WORKSPACE_READ_ONLY_ARGUMENTS）
      也当成改主工作区、撞上限不重排、被杀照可能写了一半报（task_server.py 的 modifies_workspace；bash -c 串与脚本正文里那一条同样）
      「✗ task_server_lib.py 自证：writer-dry-run-retried：」、「✗ task_server_lib.py 自证：wrapped-writer-read-only-retried：」、「✗ task_server_lib.py 自证：wrapped-read-only-hook-swap-reminded：」
  TASK_SERVER_BREAK=peer-retry-build-directory-fresh  派到第二台的尝试不叫 multi-host-run.sh 留着组里那一个编译目录、组里已有的也不点名接着用（第二台每一趟新建、跑完删；
      task_server.py 的 PeerMachine.command）
      「✗ task_server_lib.py 自证：peer-retry-build-directory-reused：」
  TASK_SERVER_BREAK=parallelism-falls-back-to-cores  各格并行的路数取不出登记的每台默认重活线程数就按本机核数（改前的样子；task_server_selftest.py 的 parallelism）：
      登记文件不在、写坏、SINGLEFS_THREAD_CAP 不是正整数时本该判红
      「✗ task_server_lib.py 自证：parallelism-registration-refused：」
  TASK_SERVER_BREAK=build-directory-named-by-label  本机组里新开的编译目录照开它的那一次尝试的标签起名（task_server_machines.py 的 build_directory）
      「✗ task_server_lib.py 自证：build-slot-reused-on-same-tree：」
  TASK_SERVER_BREAK=whole-workspace-cargo-accepted  提交时不判不点名包的全仓编译，不带 -p 的 cargo build、带 --workspace 的 cargo clippy 照收（改前的样子；任务 8105；
      task_server.py 的 placement_refusal 与 whole_workspace_cargo_refusal）
      「✗ task_server_lib.py 自证：whole-workspace-cargo-refused：」
  TASK_SERVER_BREAK=copy-proof-build-accepted  副本上有证红（在排在跑、留着还原日志、持着副本锁持有者记录）时，会编译的命令照收（task_server.py 的 copy_proof_refusal 不判这一头）
      「✗ task_server_lib.py 自证：copy-proof-build-refused：」
  TASK_SERVER_BREAK=copy-proof-start-accepted  副本里已有会编译的任务在排在跑时，对这份副本的证红照收（task_server.py 的 copy_proof_refusal 不判这一头）
      「✗ task_server_lib.py 自证：copy-proof-start-refused：」
  TASK_SERVER_BREAK=copy-proof-every-copy-judged  不比副本：只要有证红在排在跑，别的副本里会编译的命令也拒，只要有会编译的任务在排在跑，别的副本的证红也拒
      「✗ task_server_lib.py 自证：copy-proof-build-refused：」、「✗ task_server_lib.py 自证：copy-proof-start-refused：」
  TASK_SERVER_BREAK=copy-proof-every-command-judged  不看会不会编译：副本上有证红时同副本里不编译的命令也拒，副本里有任务在排在跑时不编译的也挡住对它的证红
      「✗ task_server_lib.py 自证：copy-proof-build-refused：」、「✗ task_server_lib.py 自证：copy-proof-start-refused：」
  TASK_SERVER_BREAK=repository-target-bypass-unwarned  提交时不判命令与入口脚本取仓根 target/ 下的二进制，提交回复、日志头与 status 都不打严厉警告
      （改前的样子：任务 7937–7944 取了仓根 target/ 下合并前的旧二进制；task_server.py 的 repository_target_bypass）
      「✗ task_server_lib.py 自证：repository-target-bypass-warned：」
  TASK_SERVER_BREAK=cancelled-artifacts-kept  取消之后不删各次尝试的现场目录、不核第二台那一趟的本地目录，取消的回复与日志里不报删前删后
      （task_server.py 的 remove_cancelled_artifacts）
      「✗ task_server_lib.py 自证：cancelled-artifacts-removed：」
  TASK_SERVER_BREAK=cancel-not-started-receipt-unread  取消之后核第二台那一趟的本地目录不读清理回执：回执还是 not-started、目录还没建的，日志里没有 ⑥ 那一行也算没删成、
      cancel 退 5（改前的样子；task_server.py 的 cancelled_attempt_artifacts）
      「✗ task_server_lib.py 自证：cancelled-before-peer-trip-built：」
      （这一格红的同时，task cancel 自己打的那一句也是失败行）「✗ 任务 … 停了，但取消之后有 1 份产物没删掉（逐份在上面）」
  TASK_SERVER_BREAK=inbox-not-sent  服务一条消息都不往会话的收件 socket 发（task_server.py 的 send_to_session），发不出去的告警也不写
      「✗ task_server_lib.py 自证：inbox-outcome-succeeded：」、「✗ task_server_lib.py 自证：inbox-outcome-failed：」、「✗ task_server_lib.py 自证：inbox-queued-too-long-once：」、「✗ task_server_lib.py 自证：inbox-unreachable-task-runs：」、「✗ task_server_lib.py 自证：run-hard-limit-stopped：」、
      「✗ task_server_lib.py 自证：health-database-unwritable-notified：」、「✗ task_server_lib.py 自证：health-unit-missing-notified：」、「✗ task_server_lib.py 自证：health-nfs-notified：」、「✗ task_server_lib.py 自证：health-peer-unavailable-notified：」、「✗ task_server_lib.py 自证：health-gpu-card-failed-notified：」（新格，推的，没复跑）
      「✗ task_server_lib.py 自证：peer-health-slow-rechecked：」、「✗ task_server_lib.py 自证：long-run-reminded-not-stopped：」（新格，推的，没复跑）、「✗ task_server_lib.py 自证：long-run-confirm-silences：」（新格，推的，没复跑）、「✗ task_server_lib.py 自证：loop-leak-check-on-bounce：」（新格，推的，没复跑）、「✗ task_server_lib.py 自证：repeated-cap-hits-reminded-once：」（新格，推的，没复跑）
      「✗ task_server_lib.py 自证：health-gpu-block-not-judgeable-notified：」（新格，推的，没复跑）
  TASK_SERVER_BREAK=inbox-auth-omitted  发消息不发认证行、只发 user 那一行（task_server.py 的 post_to_inbox）
      「✗ task_server_lib.py 自证：inbox-outcome-succeeded：」、「✗ task_server_lib.py 自证：inbox-outcome-failed：」、「✗ task_server_lib.py 自证：inbox-queued-too-long-once：」、「✗ task_server_lib.py 自证：run-hard-limit-stopped：」、
      「✗ task_server_lib.py 自证：health-database-unwritable-notified：」、「✗ task_server_lib.py 自证：health-unit-missing-notified：」、「✗ task_server_lib.py 自证：health-nfs-notified：」、「✗ task_server_lib.py 自证：health-peer-unavailable-notified：」、「✗ task_server_lib.py 自证：health-gpu-card-failed-notified：」（新格，推的，没复跑）
      「✗ task_server_lib.py 自证：peer-health-slow-rechecked：」、「✗ task_server_lib.py 自证：long-run-confirm-silences：」（新格，推的，没复跑）、「✗ task_server_lib.py 自证：loop-leak-check-on-bounce：」（新格，推的，没复跑）、「✗ task_server_lib.py 自证：repeated-cap-hits-reminded-once：」（新格，推的，没复跑）
      （long-run-reminded-not-stopped 不红，量的：它只按第一行取消息、核正文里的字，不看认证行、也不核令牌；别把这一格点进来）
      「✗ task_server_lib.py 自证：health-gpu-block-not-judgeable-notified：」（新格，推的，没复跑）
  TASK_SERVER_BREAK=inbox-reported-every-round  发消息那一路不看报到的提醒门槛与段，每一轮照发（task_server.py 的 push_attention、push_suspects）
      「✗ task_server_lib.py 自证：inbox-queued-too-long-once：」、「✗ task_server_lib.py 自证：repeated-cap-hits-reminded-once：」（新格，推的，没复跑）、「✗ task_server_lib.py 自证：long-run-reminded-not-stopped：」（新格，推的，没复跑）、「✗ task_server_lib.py 自证：long-run-confirm-silences：」（新格，推的，没复跑）
  TASK_SERVER_BREAK=pushed-outcome-not-collected  结局发出去了也不记已取（task_server.py 的 push_outcomes）
      「✗ task_server_lib.py 自证：inbox-outcome-succeeded：」、「✗ task_server_lib.py 自证：inbox-outcomes-resent-after-error：」
  TASK_SERVER_BREAK=inbox-pending-dropped-on-error  发结局那一轮处理到某一条抛错时不把它与后面还没处理的号放回待发，下一轮不补发（改前的样子；task_server.py 的 push_outcomes）
      「✗ task_server_lib.py 自证：inbox-outcomes-resent-after-error：」
  TASK_SERVER_BREAK=health-unwatched  服务不盯自己的健康、不发「服务健康」（task_server.py 的 watch_health）
      「✗ task_server_lib.py 自证：health-database-unwritable-notified：」、「✗ task_server_lib.py 自证：health-unit-missing-notified：」、「✗ task_server_lib.py 自证：health-nfs-notified：」、「✗ task_server_lib.py 自证：health-peer-unavailable-notified：」、「✗ task_server_lib.py 自证：health-gpu-card-failed-notified：」（新格，推的，没复跑）
      「✗ task_server_lib.py 自证：peer-health-slow-rechecked：」（新格，推的，没复跑）
      「✗ task_server_lib.py 自证：health-gpu-block-not-judgeable-notified：」、「✗ task_server_lib.py 自证：gpu-health-survives-restart：」（新格，推的，没复跑）
      「✗ task_server_lib.py 自证：gpu-health-unread-host-survives-restart：」、「✗ task_server_lib.py 自证：gpu-block-not-judgeable-outlet-follows-claim：」、「✗ task_server_lib.py 自证：gpu-block-not-judgeable-one-per-stream：」（新格，推的，没复跑）
  TASK_SERVER_BREAK=health-alerted-every-round  同一样服务健康对同一个会话每一轮照发，不记发过的（task_server.py 的 watch_health）
      「✗ task_server_lib.py 自证：health-database-unwritable-notified：」、「✗ task_server_lib.py 自证：health-unit-missing-notified：」、「✗ task_server_lib.py 自证：health-nfs-notified：」、「✗ task_server_lib.py 自证：health-peer-unavailable-notified：」、「✗ task_server_lib.py 自证：health-gpu-card-failed-notified：」（新格，推的，没复跑）
      （peer-health-slow-rechecked 不红，量的：它等到头一条服务健康消息就取、不再等下一轮，核的是慢一次先复查，不是每一轮重发；别把这一格点进来）
      「✗ task_server_lib.py 自证：health-gpu-block-not-judgeable-notified：」、「✗ task_server_lib.py 自证：gpu-health-survives-restart：」（新格，推的，没复跑）
      「✗ task_server_lib.py 自证：gpu-health-unread-host-survives-restart：」、「✗ task_server_lib.py 自证：gpu-block-not-judgeable-one-per-stream：」（新格，推的，没复跑）
  TASK_SERVER_BREAK=gpu-card-failure-unwatched  服务健康不读 GPU 服务的卡坏行、不发「GPU 服务卡坏」、不写 gpu-card-failed 告警（task_server.py 的 gpu_card_failures）
      「✗ task_server_lib.py 自证：health-gpu-card-failed-notified：」
      「✗ task_server_lib.py 自证：health-gpu-block-not-judgeable-notified：」（新格，推的，没复跑）
      「✗ task_server_lib.py 自证：gpu-health-unread-host-survives-restart：」、「✗ task_server_lib.py 自证：gpu-block-not-judgeable-outlet-follows-claim：」、「✗ task_server_lib.py 自证：gpu-block-not-judgeable-one-per-stream：」（新格，推的，没复跑）
  TASK_SERVER_BREAK=gpu-block-not-judgeable-unwatched  cards-failed 转来的判不了的行不认：不发「GPU 服务判不了的块」、不写 gpu-block-not-judgeable 告警（gpu_service_report）
      「✗ task_server_lib.py 自证：health-gpu-block-not-judgeable-notified：」、「✗ task_server_lib.py 自证：gpu-card-lines-match-the-service-fields：」（新格，推的，没复跑）
      「✗ task_server_lib.py 自证：gpu-block-not-judgeable-outlet-follows-claim：」、「✗ task_server_lib.py 自证：gpu-block-not-judgeable-one-per-stream：」（新格，推的，没复跑）
  TASK_SERVER_BREAK=gpu-cards-unread-taken-as-healed  cards-failed 说某一台没读成时不留那一台上一回读到的，当成好了（gpu_service_problems_merged）
      「✗ task_server_lib.py 自证：gpu-service-report-keeps-unread-hosts：」（新格，推的，没复跑）
  TASK_SERVER_BREAK=gpu-card-number-defaulted  卡坏行里取不到卡号时卡号记成「?」照收，不报取不到（gpu_service_report）
      「✗ task_server_lib.py 自证：gpu-card-lines-match-the-service-fields：」（新格，推的，没复跑）
  TASK_SERVER_BREAK=gpu-health-forgotten-before-the-first-read  记着发过的 GPU 那几样这一回没读到就一律当成好了忘掉，服务起来之后头一回读 GPU 服务之前也忘（task_server.py 的 gpu_health_settled）
      「✗ task_server_lib.py 自证：gpu-health-survives-restart：」（新格，推的，没复跑）
      「✗ task_server_lib.py 自证：gpu-health-unread-host-survives-restart：」、「✗ task_server_lib.py 自证：gpu-block-not-judgeable-one-per-stream：」（新格，推的，没复跑）
  TASK_SERVER_BREAK=gpu-alerted-memory-only  GPU 服务卡坏、判不了写过告警的那几样只记在内存里，服务重启后同一样再写一回告警（task_server.py 的 persist_restart_surviving_state）
      「✗ task_server_lib.py 自证：gpu-health-survives-restart：」（新格，推的，没复跑）
      「✗ task_server_lib.py 自证：gpu-health-unread-host-survives-restart：」（新格，推的，没复跑）
  TASK_SERVER_BREAK=gpu-health-unread-host-taken-as-healed  记着发过的卡坏这一回没读到，不看它那一台自服务起来读成过没有就当成好了忘掉（task_server.py 的 gpu_health_settled；
      服务重起后头一回读那一台没读成，读成之后同一张坏卡再发一回）
      「✗ task_server_lib.py 自证：gpu-health-unread-host-survives-restart：」（新格，推的，没复跑）
  TASK_SERVER_BREAK=gpu-block-not-judgeable-claim-ignored  判不了的块那一条的出路不看各块的 claim=，没放认领的也写成放了、另一台会去试（task_server.py 的 gpu_card_failures）
      「✗ task_server_lib.py 自证：gpu-block-not-judgeable-outlet-follows-claim：」（新格，推的，没复跑）
  TASK_SERVER_BREAK=gpu-block-not-judgeable-keyed-per-block  判不了的照每块各算一样、种类带进程号：同一条流几块各发一条，GPU 服务换了进程再发一回（gpu_service_report）
      「✗ task_server_lib.py 自证：gpu-block-not-judgeable-one-per-stream：」、「✗ task_server_lib.py 自证：gpu-card-lines-match-the-service-fields：」（新格，推的，没复跑）
  TASK_SERVER_BREAK=gpu-block-not-judgeable-forgotten-between-reads  判不了的流这一回没读到就照卡坏的办当成好了忘掉，GPU 服务重起之后同一条流再判不了再发一回
      （task_server.py 的 gpu_health_settled）
      「✗ task_server_lib.py 自证：gpu-block-not-judgeable-one-per-stream：」（新格，推的，没复跑）
  TASK_SERVER_BREAK=unclaimed-unwarned  交任务时不判会话名下有没有结局满时限没人取的，不打严厉警告（task_server.py 的 unclaimed_before）
      「✗ task_server_lib.py 自证：unclaimed-results-warned：」
  TASK_SERVER_BREAK=leftovers-error-uncaught  attempt_runner 停单元里留下的进程时抛的异常照改前不隔离，result.json 写不出、成功的任务被判 launcher-lost
      （records/2026-11-03-task-server与prune-server深入对攻审核.md 的 TS-02；说明里的 leftovers_error_uncaught，attempt_runner.py 的 main）
      「✗ task_server_lib.py 自证：leftovers-error-isolated：」
  TASK_SERVER_BREAK=inbox-attention-per-shard  发给会话的要处置的提醒照改前逐片各发一条、不合、不压一轮（task_server.py 的 attention_groups、push_attention）
      「✗ task_server_lib.py 自证：inbox-shard-attention-merged：」
  TASK_SERVER_BREAK=memory-peak-with-page-cache  本机记的峰值与判贴着上限照改前读 scope 的 memory.peak、memory.current（含页缓存；任务 8853；
      attempt_runner.py 的 Sampler 照说明里的 memory_peak_with_page_cache，task_server.py 的 scope_memory_peak、scope_memory_current）
      「✗ task_server_lib.py 自证：page-cache-not-counted-as-process-memory：」
  TASK_SERVER_BREAK=attention-reported-memory-only  报到的提醒门槛照改前只记在服务内存里，服务重启后同一道门槛再报一次（会话 46 的任务 8819；task_server.py 的 note_attention_reported、
      stored_reminded_thresholds）
      「✗ task_server_lib.py 自证：attention-reported-survives-restart：」、「✗ task_server_lib.py 自证：reminder-ledger-other-shape-read-afresh：」（新格，推的，没复跑）
  TASK_SERVER_BREAK=attention-queue-reasonless-sent  排队原因还没判出来的那一轮照改前照报排队过久、原因写读不到（task_server.py 的 attention_of_record）
      「✗ task_server_lib.py 自证：attention-queue-waits-for-reason：」
  TASK_SERVER_BREAK=booking-over-host-limit  钉值之外预测峰值比判的那一台可用上限大的，预测峰值与记账不收到可用上限、照原样交给 judge_host，那一台判永远放不下（task_server.py 的 booked_at_cap）
      「✗ task_server_lib.py 自证：booking-within-host-limit-every-history：」、「✗ task_server_lib.py 自证：cap-hit-record-over-every-host-starts-clamped：」
  TASK_SERVER_BREAK=record-over-every-host-never-fits  预测峰值比能派的每一台可用上限都大、别处也没有可能装得下的台时，每一台照永远放不下判、记 never-fits（task_server.py 的 hosts_short_of_record）
      「✗ task_server_lib.py 自证：cap-hit-record-over-every-host-starts-clamped：」
  TASK_SERVER_BREAK=drop-history-cap-hits-ignored  control drop-history --attempt-records-over-gib 不看撞上限那一支，只比量到的峰值（control 的 drop_history）
      「✗ task_server_lib.py 自证：control-drop-history-oversized-records：」
  TASK_SERVER_BREAK=oversized-budget-silent  记账不小于整片界的不记成要处置的提醒 oversized-budget（task_server.py 的 budget_check、attention_of_record）
      「✗ task_server_lib.py 自证：oversized-budget-reported：」
  TASK_SERVER_BREAK=idle-short-unjudged  不判「在每一台上都起不来」（就算服务在那台跑的任务全收尾、整机空闲也不够要的）：一律照一般放不下的排着，挡后面的、不记提醒（task_server.py 的 idle_short_text）
      「✗ task_server_lib.py 自证：booking-beyond-idle-machine-not-blocking：」（wait 等不到 booking-beyond-idle-machine 那一行）
  TASK_SERVER_BREAK=idle-short-silent  在每一台上都起不来的不记成要处置的提醒 booking-beyond-idle-machine（task_server.py 的 note_idle_short、attention_of_record）
      「✗ task_server_lib.py 自证：booking-beyond-idle-machine-not-blocking：」（wait 等不到那一行）
  TASK_SERVER_BREAK=idle-short-blocks-later  在每一台上都起不来的照一般放不下的那样算「排在前面还没起的」、排满时限挡后面的（task_server.py 的排队那一轮 blocked 那一支）
      「✗ task_server_lib.py 自证：booking-beyond-idle-machine-not-blocking：」（后面没有上次运行时长的小任务被判不插队、起不来）
  TASK_SERVER_BREAK=oversized-under-idle-short-reported  同一条在排、booking-beyond-idle-machine 开着时，oversized-budget 也照报（task_server.py 的 attention_of_record 不罩住它）
      「✗ task_server_lib.py 自证：booking-beyond-idle-machine-not-blocking：」（wait 打了两行提醒）
  TASK_SERVER_BREAK=oversized-reported-before-queue-judged  「在每一台上都起不来」那一判还没对它判过的在排任务（idle_short_judged 里没有它）也报 oversized-budget，抢在 booking-beyond-idle-machine 之前（task_server.py 的 attention_of_record）
      「✗ task_server_lib.py 自证：booking-beyond-idle-machine-not-blocking：」（还没判过的在排任务不该报 oversized-budget）
  TASK_SERVER_BREAK=long-run-idle-silent  跑满 LONG_RUN_ATTENTION_SECONDS 而平均在用核数不到给的线程数 24/31 的不记成要处置的提醒 long-run-idle-cores（task_server.py 的 long_run_idle_items）
      「✗ task_server_lib.py 自证：long-run-idle-cores-reported：」
  TASK_SERVER_BREAK=fills-any-cap-silent  同一命令键最近三次峰值都贴着各自上限的不记成要处置的提醒 fills-any-cap（task_server.py 的 fills_any_cap_items）
      「✗ task_server_lib.py 自证：fills-any-cap-reported：」
  TASK_SERVER_BREAK=fills-any-cap-counts-cap-hits  fills-any-cap 的那三次连撞了上限的尝试也算（照上限一档档撞上去的也报成按上限开并行；task_server.py 的 fills_any_cap_items）
      「✗ task_server_lib.py 自证：fills-any-cap-cap-hits-not-reported：」
  TASK_SERVER_BREAK=fills-any-cap-counts-record-caps  fills-any-cap 的那三次里上限照记下的峰值定的（尝试的 prediction 一栏是 record）也算（task_server.py 的 fills_any_cap_items）
      「✗ task_server_lib.py 自证：fills-any-cap-record-caps-not-reported：」
  TASK_SERVER_BREAK=old-metric-peaks-read  定额度照改前连没写峰值口径的旧尝试行一起读（task_server.py 的 current_metric）
      「✗ task_server_lib.py 自证：old-metric-peaks-not-counted：」
  TASK_SERVER_BREAK=attention-next-generic  anomaly_advice 不照异常表取，每一种都交一句通用的「看 status、等不起就取消改小重交」（提醒、打回的 next= 与发给会话的正文都退回它）
      「✗ task_server_lib.py 自证：long-run-sessionless-noted：」（红是量的）
      （long-run-reminded-not-stopped 不红，量的：推给会话的正文里 task cancel 与 task confirm 那两条出路来自 flag_lines 的长跑待确认那一行，不经 anomaly_advice；别把这一格点进来）
      「✗ task_server_lib.py 自证：anomaly-advice-in-texts：」、「✗ task_server_lib.py 自证：attention-queued-too-long：」、「✗ task_server_lib.py 自证：attention-quiet-busy-log：」、「✗ task_server_lib.py 自证：io-wait-local-not-stalled：」、「✗ task_server_lib.py 自证：attention-cleanup-pending：」、「✗ task_server_lib.py 自证：attention-shard-advance-failed：」、「✗ task_server_lib.py 自证：attention-reported-once：」、「✗ task_server_lib.py 自证：attention-reported-survives-restart：」、「✗ task_server_lib.py 自证：attention-queue-waits-for-reason：」、「✗ task_server_lib.py 自证：oversized-budget-reported：」、「✗ task_server_lib.py 自证：long-run-idle-cores-reported：」、「✗ task_server_lib.py 自证：fills-any-cap-reported：」、「✗ task_server_lib.py 自证：memory-cap-at-maximum：」、「✗ task_server_lib.py 自证：anomaly-repushed-with-history：」、「✗ task_server_lib.py 自证：unclaimed-outcome-repushed：」（头两格之后推的，没复跑：各格核 next= 以异常表那一行起头、或核那一句修改意见在不在）、「✗ task_server_lib.py 自证：repeated-cap-hits-reminded-once：」（新格，推的，没复跑）
  TASK_SERVER_BREAK=progress-not-pushed  在跑满 58 分钟的任务不给提交它的会话发进度（task_server.py 的 push_progress）
      「✗ task_server_lib.py 自证：running-progress-pushed：」（在跑满时限两回、一条进度都没收到）
  TASK_SERVER_BREAK=unclaimed-outcome-not-repushed  结局满 30 分钟没交给会话的不再推（task_server.py 的 repush_unclaimed_outcomes）
      「✗ task_server_lib.py 自证：unclaimed-outcome-repushed：」
  TASK_SERVER_BREAK=anomaly-not-repushed  门槛不随时长涨的提醒同一道门槛推过之后一直还在的不再推（task_server.py 的 attention_repush）
      「✗ task_server_lib.py 自证：anomaly-repushed-with-history：」、「✗ task_server_lib.py 自证：long-run-confirm-silences：」
  TASK_SERVER_BREAK=anomaly-acknowledgement-ignored  confirm --why 记了照跑的提醒（oversized-budget、fills-any-cap）照样再推（task_server.py 的 anomaly_acknowledged）
      「✗ task_server_lib.py 自证：anomaly-acknowledged-not-repushed：」
  TASK_SERVER_BREAK=anomaly-acknowledge-open-only  confirm --why 只认此刻开着的照跑提醒，不认库里记着报过的（task_server.py 的 acknowledge_anomalies）
      「✗ task_server_lib.py 自证：oversized-budget-acknowledged-after-restart：」
  TASK_SERVER_BREAK=runaway-reports-memory-only  任务之外的空转进程报过几次只记在内存里，换服务后从零数、要再报两次才停（task_server.py 的 persist_restart_surviving_state、restore_restart_surviving_state）
      「✗ task_server_lib.py 自证：runaway-reports-survive-restart：」
  TASK_SERVER_BREAK=runaway-reused-pid-carried  上次报它之后才起的同号进程照号接着原来那个的报数，一露面就被停（task_server.py 的 watch_runaway_processes）
      「✗ task_server_lib.py 自证：runaway-reused-pid-counted-afresh：」
  TASK_SERVER_BREAK=advance-episodes-memory-only  分片父任务推不动的这一回与开始过几回只记在内存里，换服务后从第 1 回数起（task_server.py 的 persist_restart_surviving_state、restore_restart_surviving_state）
      「✗ task_server_lib.py 自证：advance-episodes-survive-restart：」
  TASK_SERVER_BREAK=restart-surviving-signature-unchecked  库里换服务不丢的那几份签名对不上的也认，绕过服务改过的停用、报数照样读回（task_server.py 的 Store.restart_surviving_state）
      「✗ task_server_lib.py 自证：restart-surviving-tampered-not-adopted：」
  TASK_SERVER_BREAK=folded-key-unnamed  记录来源只写「这把命令键」，不写键原文、不写键里折掉了点名项（task_server.py 的 folded_key_note）
      「✗ task_server_lib.py 自证：budget-source-names-folded-key：」
  TASK_SERVER_BREAK=inbox-socket-from-submit-only  发消息照交任务时记的收件 socket，不按会话号读会话登记（task_server.py 的 inbox_socket_of）
      「✗ task_server_lib.py 自证：inbox-socket-follows-session：」
  TASK_SERVER_BREAK=inbox-missing-accepted  客户端读不到会话的收件地址也照交（task 的 submit）
      「✗ task_server_lib.py 自证：inbox-unreachable-task-runs：」
  TASK_SERVER_BREAK=submit-trace-silent  提交那一路不打进出日志：回话时那一行与一步慢时当场那一行都不打（task_server.py 的 SubmitTrace）
      「✗ task_server_lib.py 自证：submit-trace-logged：」（推的，没复跑）
  TASK_SERVER_BREAK=confirm-skipped  判成长任务的也交了就进队、不等确认（task_server.py 的 submit）
      「✗ task_server_lib.py 自证：submit-awaits-confirmation：」、「✗ task_server_lib.py 自证：unconfirmed-cancelled：」、「✗ task_server_lib.py 自证：operation-log：」、「✗ task_server_lib.py 自证：confirmation-scope-text：」（后两格推的，没复跑：不等确认就跑完、操作日志里没有 confirm 那一条）、
      「✗ task_server_lib.py 自证：long-task-by-record：」、「✗ task_server_lib.py 自证：long-task-whole-run：」、「✗ task_server_lib.py 自证：long-task-sharded：」、「✗ task_server_lib.py 自证：long-task-unrecorded-test-or-item：」、
      「✗ task_server_lib.py 自证：confirm-on-queued-task-accepted：」（后五格新格，推的，没复跑）
  TASK_SERVER_BREAK=long-task-record-unread  判长任务不看库里记过的用时（task_server.py 的 long_task_judgement 里按记过的用时判的那一条）
      「✗ task_server_lib.py 自证：long-task-by-record：」（新格，推的，没复跑）
  TASK_SERVER_BREAK=long-task-latest-only  判长任务的记录只看最近一次的用时，不取近几次最大的（long_task_judgement）
      「✗ task_server_lib.py 自证：long-task-by-record：」（新格，推的，没复跑）
  TASK_SERVER_BREAK=long-task-whole-unjudged  不指定范围的整趟（范围声明种类 whole、--full-gate、--whole-package-confirmed）不判成长任务（long_task_judgement 里判整趟的那一条）
      「✗ task_server_lib.py 自证：long-task-whole-run：」（新格，推的，没复跑）
  TASK_SERVER_BREAK=long-task-shard-unjudged  要分片的不判成长任务（long_task_judgement 里判分片的那一条）
      「✗ task_server_lib.py 自证：long-task-sharded：」（新格，推的，没复跑）
  TASK_SERVER_BREAK=long-task-unrecorded-unjudged  没有用时记录的 cargo 测试与按项跑的门禁（--item）不判成长任务（long_task_judgement 里判没有记录的那一条）
      「✗ task_server_lib.py 自证：long-task-unrecorded-test-or-item：」、「✗ task_server_lib.py 自证：confirm-on-queued-task-accepted：」（新格，推的，没复跑：后一格拿没记录的 --item 命令当待确认的对照）
  TASK_SERVER_BREAK=short-task-held  不分长短一律待确认，没判成长任务的也不直接进队（long_task_judgement）
      「✗ task_server_lib.py 自证：long-task-by-record：」、「✗ task_server_lib.py 自证：long-task-whole-run：」、「✗ task_server_lib.py 自证：long-task-sharded：」、「✗ task_server_lib.py 自证：long-task-unrecorded-test-or-item：」、
      「✗ task_server_lib.py 自证：short-task-queued-directly：」、「✗ task_server_lib.py 自证：confirm-on-queued-task-accepted：」（新格，推的，没复跑：各格的对照那一条直接进队的变成待确认）
  TASK_SERVER_BREAK=confirm-direct-refused  task confirm 对直接进队的任务回 409「不在待确认」、退 3（task_server.py 的 confirm）
      「✗ task_server_lib.py 自证：confirm-on-queued-task-accepted：」（新格，推的，没复跑）
  TASK_SERVER_BREAK=direct-reason-unreported  提交回话不带直接进队的依据，客户端不打「直接进队：」那一行（task_server.py 的 submit_in_thread）
      「✗ task_server_lib.py 自证：long-task-by-record：」、「✗ task_server_lib.py 自证：long-task-whole-run：」、「✗ task_server_lib.py 自证：long-task-sharded：」、「✗ task_server_lib.py 自证：long-task-unrecorded-test-or-item：」、
      「✗ task_server_lib.py 自证：short-task-queued-directly：」（新格，推的，没复跑：各格的对照那一条不打那一行）
  TASK_SERVER_BREAK=long-task-reason-unsaid  确认单不写「判成长任务」那一行（task_server.py 的 confirmation）
      「✗ task_server_lib.py 自证：long-task-by-record：」、「✗ task_server_lib.py 自证：long-task-whole-run：」、「✗ task_server_lib.py 自证：long-task-sharded：」、「✗ task_server_lib.py 自证：long-task-unrecorded-test-or-item：」（新格，推的，没复跑）
  TASK_SERVER_BREAK=confirm-counts-unchecked  task confirm 不核项数与在排在跑的条数（task_server.py 的 confirm）
      「✗ task_server_lib.py 自证：submit-awaits-confirmation：」、「✗ task_server_lib.py 自证：concurrent-confirms-serialized：」（新格，推的，没复跑）
  TASK_SERVER_BREAK=unconfirmed-kept  交了没确认的满时限也不取消（task_server.py 的 expire_unconfirmed）
      「✗ task_server_lib.py 自证：unconfirmed-cancelled：」
  TASK_SERVER_BREAK=path-check-substring  提交时判共享根外的路径，每个带空白的词都按子串认，参数值中间夹着的 /（「（接的是 /）」）也当绝对路径、整条拒（task_server.py 的 find_outside_shared_path）
      「✗ task_server_lib.py 自证：path-check-whole-words：」
  TASK_SERVER_BREAK=cleanup-tests-tables-hand-listed  清理协议单测的假调度器照手抄的名单建表、漏 activity_windows 这类后加的表（tests/test_cleanup_protocol.py 的 scheduler）
      「✗ task_server_lib.py 自证：cleanup-protocol-unit-tests：」
  TASK_SERVER_BREAK=clear-queue-active-state-only  control clear-queue 判停没停只看 is-active 的输出，单元 cgroup 里还有进程或读不到也照清库（control 的 clear_queue）
      「✗ task_server_lib.py 自证：cleanup-protocol-unit-tests：」
  TASK_SERVER_BREAK=clear-queue-running-attempts-ignored  control clear-queue 不看库里有没有 state='running' 的尝试、照清库（control 的 clear_queue）
      「✗ task_server_lib.py 自证：cleanup-protocol-unit-tests：」
  TASK_SERVER_BREAK=peer-health-single-timeout-reported  连第二台的 ssh 头一次超时就算连不上、不复查（peer_shell_rechecked）
      「✗ task_server_lib.py 自证：peer-health-slow-rechecked：」
  TASK_SERVER_BREAK=check-nfs-probed-live-by-default  control check 不带 --probe 也现探 NFS 共享四项，不读服务 /health 的 nfs 一栏（task-server/control 的 check）
      「✗ task_server_lib.py 自证：control-check-reads-service-nfs-report：」
  TASK_SERVER_BREAK=check-probe-unbounded  control check --probe 不设总时长，慢的那一项一直等（task-server/control 的 probed_nfs）
      「✗ task_server_lib.py 自证：control-check-probe-bounded：」
  TASK_SERVER_BREAK=peer-recheck-once-only  复查期里只复查一回（头两次超时、第三次才回了的也算连不上并群发；peer_shell_rechecked）
      「✗ task_server_lib.py 自证：peer-health-slow-within-recheck-period：」
  TASK_SERVER_BREAK=cap-hit-stat-unreported  内存最高那一刻的明细不抄进撞上限那一次的结局说明与推给会话的正文（task_server.py 的 finish_attempt）
      「✗ task_server_lib.py 自证：peak-memory-stat-in-note：」、「✗ task_server_lib.py 自证：peer-peak-memory-stat-in-note：」、「✗ task_server_lib.py 自证：loop-leak-check-on-bounce：」（新格，推的，没复跑）
  TASK_SERVER_BREAK=peak-memory-stat-read-at-exit  内存明细两台都留最后读到的那一份（进程被杀之后读到的，堆已释放；attempt_runner.py 的 Sampler 照说明里的 memory_stat_at_exit，task_server.py 的 log_content）
      「✗ task_server_lib.py 自证：peak-memory-stat-in-note：」、「✗ task_server_lib.py 自证：peer-peak-memory-stat-in-note：」
  TASK_SERVER_BREAK=exit-peak-only-when-unsampled  本机按轮采到了就不看内存包装退出时交的那一份，按轮之间冒出来的峰值记不到（attempt_runner.py 照说明里的 exit_peak_only_when_unsampled）
      「✗ task_server_lib.py 自证：short-command-peak-recorded：」、「✗ task_server_lib.py 自证：loop-leak-check-on-bounce：」
  TASK_SERVER_BREAK=loop-leak-equal-peaks-growing  连着撞上限的正文里峰值全部相等也写「峰值一次比一次大」（loop_leak_check_text）
      「✗ task_server_lib.py 自证：loop-leak-trend-sentences：」
  TASK_SERVER_BREAK=loop-leak-reading-mismatch-silent  每一次都没顶到上限又撞上限的不写「峰值远低于上限」那一句（loop_leak_check_text）
      「✗ task_server_lib.py 自证：loop-leak-trend-sentences：」
  TASK_SERVER_BREAK=pinned-squeeze-as-cap-hit  钉了的命令退 254（被 slice 挤掉）照撞上限办、打回 memory-cap-at-maximum（task_server.py 的 finish_attempt）
      「✗ task_server_lib.py 自证：pinned-squeezed-requeued：」
  TASK_SERVER_BREAK=pinned-over-limit-unreported  钉值超过 slice 总上限减余量也照那一项减余量报，回读不说（task_server.py 的 pin_value、pin_over_limit_lines）
      「✗ task_server_lib.py 自证：pinned-cap-over-limit-reported：」
  TASK_SERVER_BREAK=pin-cap-help-pinned  control pin-cap --help 把 --help 当命令钉一个值（control 的 main）
      「✗ task_server_lib.py 自证：pin-cap-help-not-pinned：」
  TASK_SERVER_BREAK=scene-sessionless-kept  没有会话的任务的现场也要等结局被取走才删，人手交的不起 wait 就一直不删（task_server.py 的 scene_due）
      「✗ task_server_lib.py 自证：scene-sessionless-swept：」
  TASK_SERVER_BREAK=long-run-unconfirmed-silent  累计跑满两小时的不提醒（task_server.py 的 run_limit_exempt）
      「✗ task_server_lib.py 自证：long-run-reminded-not-stopped：」、「✗ task_server_lib.py 自证：long-run-check-not-exempt：」、
      「✗ task_server_lib.py 自证：long-run-merge-and-verify-not-exempt：」、「✗ task_server_lib.py 自证：long-run-confirm-silences：」、
      「✗ task_server_lib.py 自证：long-run-sessionless-noted：」、「✗ task_server_lib.py 自证：whole-package-confirmed-still-reminded：」（后三格推的，没复跑：要一条被提醒的任务才往下判）
  TASK_SERVER_BREAK=long-run-cap-hits-counted  撞了内存上限的那几次尝试的用时也累加，重来的那一次不归零（task_server.py 的 run_limit_accumulated）
      「✗ task_server_lib.py 自证：long-run-reminder-across-cap-hits：」、「✗ task_server_lib.py 自证：run-hard-limit-cap-hit-resets：」
  TASK_SERVER_BREAK=full-gate-flag-ignored  带了 --full-gate 的照样提醒（task_server.py 的 run_limit_exempt）
      「✗ task_server_lib.py 自证：long-run-full-gate-not-reminded：」
  TASK_SERVER_BREAK=full-gate-any-command-accepted  --full-gate 带在不是提交时门禁的命令上也照收（task_server.py 的 full_gate_refusal）
      「✗ task_server_lib.py 自证：full-gate-flag：」
  TASK_SERVER_BREAK=bounce-new-id-reminder-omitted  打回结局的修改意见不带「不要使用原来的任务号，用 task-server/task submit 创建新的任务号」（anomaly_advice）
      「✗ task_server_lib.py 自证：anomaly-advice-in-texts：」
  TASK_SERVER_BREAK=long-run-short-why-accepted  task confirm <号> --why 去掉空白不到 LONG_RUN_WHY_MINIMUM_CHARACTERS 个字也照收（long_run_confirmation_problem）
      「✗ task_server_lib.py 自证：long-run-confirm-refused：」
  TASK_SERVER_BREAK=long-run-confirm-unchecked  task confirm <号> --why 不核会话、在不在跑与满没满界（task_server.py 的 confirm_long_run）
      「✗ task_server_lib.py 自证：long-run-confirm-refused：」
  TASK_SERVER_BREAK=long-run-sessionless-unnoted  没有会话的任务跑满两小时不往日志写那一行（task_server.py 的 long_run_sessionless_note）
      「✗ task_server_lib.py 自证：long-run-sessionless-noted：」
  TASK_SERVER_BREAK=long-run-confirmed-still-reminded  确认过的照样提醒（task_server.py 的 run_limit_exempt）
      「✗ task_server_lib.py 自证：long-run-confirm-silences：」
  TASK_SERVER_BREAK=long-run-confirmed-flag-hidden  「长跑已确认」那一行不进 task status、列表、推给会话的正文与日志头（task_server.py 的 long_run_lines）
      「✗ task_server_lib.py 自证：long-run-confirm-silences：」、「✗ task_server_lib.py 自证：long-run-sessionless-noted：」、「✗ task_server_lib.py 自证：whole-package-confirmed-still-reminded：」
  TASK_SERVER_BREAK=whole-package-unconfirmed-accepted  整包测试与不指定范围的跑法没带确认标识也照收（task_server.py 的 whole_package_refusal）
      「✗ task_server_lib.py 自证：whole-package-unconfirmed-refused：」、「✗ task_server_lib.py 自证：whole-package-single-cells-refused：」（新格，推的，没复跑）、
      「✗ task_server_lib.py 自证：unscoped-runs-refused：」（新格，推的，没复跑）、「✗ task_server_lib.py 自证：queued-whole-package-rechecked：」（新格，推的，没复跑）、
      「✗ task_server_lib.py 自证：whole-package-feature-items-others-refused：」
  TASK_SERVER_BREAK=whole-package-confirmed-exempts-two-hours  带「整包已确认」标识的跑满两小时不提醒（task_server.py 的 run_limit_exempt）
      「✗ task_server_lib.py 自证：whole-package-confirmed-still-reminded：」
  TASK_SERVER_BREAK=unnamed-run-refusal-skipped  提交时不判启动测试与验证点没点名（task_server.py 的 unnamed_run_refusal 不调 unnamed_run_problem）
      「✗ task_server_lib.py 自证：unnamed-merge-unnamed-refused：」、「✗ task_server_lib.py 自证：unnamed-cargo-target-refused：」、
      「✗ task_server_lib.py 自证：unnamed-prove-red-unnamed-refused：」、「✗ task_server_lib.py 自证：unnamed-gate-cells-judged：」、
      「✗ task_server_lib.py 自证：unnamed-crash-gate-whole-items-refused：」、
      「✗ task_server_lib.py 自证：unnamed-exemptions-accepted：」（那一格的对照：两样例外标识都不带的合入后验证应当被拒）
  TASK_SERVER_BREAK=unnamed-run-exemptions-ignored  带 --full-gate、--whole-package-confirmed 的也照判点没点名（unnamed_run_problem）
      「✗ task_server_lib.py 自证：unnamed-exemptions-accepted：」
  TASK_SERVER_BREAK=unnamed-entry-item-unchecked  merge-and-verify.sh 与 scripts/crash/ 两份入口不带 --item 也算点名（unnamed_run_problem）
      「✗ task_server_lib.py 自证：unnamed-merge-unnamed-refused：」、「✗ task_server_lib.py 自证：unnamed-exemptions-accepted：」（那一格的对照同上）
  TASK_SERVER_BREAK=unnamed-entry-item-uncounted  那三份入口带了 --item 也不算点名（unnamed_run_problem）
      「✗ task_server_lib.py 自证：unnamed-merge-named-accepted：」
  TASK_SERVER_BREAK=unnamed-cargo-filter-accepted  cargo test 点了测试目标或筛选词、没带 --exact 也算点名（unnamed_cargo_test_problem）
      「✗ task_server_lib.py 自证：unnamed-cargo-target-refused：」
  TASK_SERVER_BREAK=unnamed-cargo-exact-uncounted  cargo test `--` 之后 --exact 的测试名不算点名（unnamed_cargo_test_problem）
      「✗ task_server_lib.py 自证：unnamed-cargo-exact-accepted：」
  TASK_SERVER_BREAK=unnamed-prove-red-unchecked  证红不给变异名也算点名（unnamed_run_problem）
      「✗ task_server_lib.py 自证：unnamed-prove-red-unnamed-refused：」
  TASK_SERVER_BREAK=unnamed-prove-red-name-offset  证红的变异名从第二个起数，只点一个的算没点（unnamed_run_problem）
      「✗ task_server_lib.py 自证：unnamed-prove-red-named-accepted：」
  TASK_SERVER_BREAK=unnamed-stage-whole-accepted  门禁阶段 --item、--check 都不带也算点名（unnamed_run_problem）
      「✗ task_server_lib.py 自证：unnamed-gate-cells-judged：」
  TASK_SERVER_BREAK=unnamed-check-cells-accepted  门禁阶段带 --check 就算点名，不看点的格起不起测试（unnamed_run_problem）
      「✗ task_server_lib.py 自证：unnamed-gate-cells-judged：」
  TASK_SERVER_BREAK=unnamed-crash-gate-item-whole-accepted  门禁 crash-point-replay 带了 --item 就算点名，不看 --check 有没有把起整个验证包的格排掉（crash_gate_whole_package_cells）
      「✗ task_server_lib.py 自证：unnamed-crash-gate-whole-items-refused：」
  TASK_SERVER_BREAK=unnamed-crash-gate-cases-only-uncounted  门禁 crash-point-replay 的 --crash-cases-only 不算数，crash-case-replay 一律当整格（crash_gate_whole_package_cells）
      「✗ task_server_lib.py 自证：unnamed-crash-gate-cases-only-accepted：」
  TASK_SERVER_BREAK=test-starting-cells-misnamed  起测试的格表 lkmm 那一行的格名写错一个字（test_starting_gate_cells）
      「✗ task_server_lib.py 自证：test-starting-cells-listed：」
  TASK_SERVER_BREAK=attempt-row-overwritten  Store.update 不守库里已 ended 的尝试：拿手里先读的那份 running 的尝试整行写回，对账刚记了 ended 的那一次被写回 running、
      结局与峰值抹掉（task_server.py 的 Store.update）
      「✗ task_server_lib.py 自证：attempt-row-not-overwritten：」
  TASK_SERVER_BREAK=parent-finalized-before-children  取消分片的整件停了子任务的单元就给父任务记结局、删产物、回 stopped，不等子任务在库里记成 finished
      （task_server.py 的 cancel_children）
      「✗ task_server_lib.py 自证：parent-cancel-waits-for-children：」
  TASK_SERVER_BREAK=recheck-disk-unchecked  起之前复核不判现读那一台的盘余量：第二台的报告不带盘余量，下限与编译目录占盘都放过（task_server.py 的 Resources.recheck）
      「✗ task_server_lib.py 自证：recheck-judges-disk：」
  TASK_SERVER_BREAK=recheck-nfs-unchecked  起之前复核现读的那份不过 peer_view，复核这一刻第二台没挂上共享根也照派（task_server.py 的 launch_recheck）
      「✗ task_server_lib.py 自证：recheck-judges-nfs：」
  TASK_SERVER_BREAK=stall-stop-by-task  判卡住起的后台线程按任务号停在跑的全部，重排起的下一次也停（task_server.py 的 judged_attempt_only）
      「✗ task_server_lib.py 自证：stall-stop-spares-next-attempt：」
  TASK_SERVER_BREAK=peer-cleanup-recovered-concurrently  同一次尝试的第二台清理恢复不互斥，几处同时读到 pending 各起一趟 --recover-cleanup（task_server.py 的 confirm_peer_cleanup）
      「✗ task_server_lib.py 自证：peer-cleanup-recovered-once：」
  TASK_SERVER_BREAK=manual-from-session-accepted  会话里带 --manual 也照收（task_server.py 的 manual_refusal）
      「✗ task_server_lib.py 自证：manual-submit-path：」
  TASK_SERVER_BREAK=sessionless-unmarked-accepted  没有会话又没带 --manual 也照收（task_server.py 的 manual_refusal）
      「✗ task_server_lib.py 自证：manual-submit-path：」
  TASK_SERVER_BREAK=tampered-unit-left-running  核不过的在跑尝试记 ended 之后不停它的单元（task_server.py 的 end_tampered_attempt）
      「✗ task_server_lib.py 自证：tampered-attempt-unit-stopped：」
  TASK_SERVER_BREAK=tampered-rows-left-unrepaired  重签不写关联列、tampered 不照 parent_id 写正文的 task_id、对账判出核不过的在跑尝试只给任务记 record-tampered、尝试行留在在跑里
      （task_server.py 的 Store.write_row、Scheduler.tampered、Scheduler.reconcile）
      「✗ task_server_lib.py 自证：tampered-attempt-ended-once：」（新格，推的，没复跑）
  TASK_SERVER_BREAK=running-attempts-unverified  取在跑尝试连校验不过的一起交，内存、盘与线程的记账都算它（task_server.py 的 Store.running_attempts）
      「✗ task_server_lib.py 自证：running-attempts-verified-only：」（新格，推的，没复跑）
  TASK_SERVER_BREAK=confirm-counts-outside-session-lock  task confirm 不拿会话锁，同会话并发确认各自照对方进队之前的条数核（task_server.py 的 session_lock）
      「✗ task_server_lib.py 自证：concurrent-confirms-serialized：」（新格，推的，没复跑）
  TASK_SERVER_BREAK=resident-entries-kept  按任务号记在内存里的那几张表一项都不删（task_server.py 的 forget_settled_entries）
      「✗ task_server_lib.py 自证：resident-entries-forgotten：」（新格，推的，没复跑）
  TASK_SERVER_BREAK=whole-package-cells-all-three-only  crash-point-replay.sh 的 --check 要三格全点才算整包，单点一格的照收（whole_package_script）
      「✗ task_server_lib.py 自证：whole-package-single-cells-refused：」（新格，推的，没复跑）、「✗ task_server_lib.py 自证：whole-package-feature-items-others-refused：」
  TASK_SERVER_BREAK=merge-all-packages-accepted  merge-and-verify.sh 带 --all-packages 的不算整包（whole_package_script）
      「✗ task_server_lib.py 自证：whole-package-single-cells-refused：」（新格，推的，没复跑）
  TASK_SERVER_BREAK=whole-package-feature-items-uncounted  crash-point-replay.sh --check feature-gated-tests 点了 feature-gated:<文件名> 项也算整包（whole_package_script）
      「✗ task_server_lib.py 自证：whole-package-feature-items-accepted：」
  TASK_SERVER_BREAK=whole-package-any-item-exempts  crash-point-replay.sh --check feature-gated-tests 带了任何一个 --item 都不算整包，崩溃枚举用例的项也算（whole_package_script）
      「✗ task_server_lib.py 自证：whole-package-feature-items-others-refused：」
  TASK_SERVER_BREAK=card-declarations-accepted  入口文件头写了领卡、要卡的声明照收（card_declaration_problem 一律交 None）
      「✗ task_server_lib.py 自证：card-declarations-refused：」
  TASK_SERVER_BREAK=card-variables-passed-to-attempt  服务自己环境里的选卡变量（CARD_CHOICE_VARIABLES）照交给命令（task_server.py 的 start_attempt）
      「✗ task_server_lib.py 自证：attempt-gets-no-card：」
  TASK_SERVER_BREAK=unscoped-gate-accepted  不带选条参数的门禁与入口在提交之外交也照收（unscoped_run_entry）
      「✗ task_server_lib.py 自证：unscoped-runs-refused：」（新格，推的，没复跑）
  TASK_SERVER_BREAK=queued-refusals-not-rechecked  服务起来时重核在排的只核环境变量名单、包装前缀与编译目录三样，别的提交时拒不重核（task_server.py 的 revalidate_queued）
      「✗ task_server_lib.py 自证：queued-whole-package-rechecked：」（新格，推的，没复跑）
  TASK_SERVER_BREAK=loop-leak-check-omitted  连着撞上限之后打回的理由行不带 loop_leak=，推给会话的结局正文不写「检查代码有没有逻辑循环或内存泄漏」那一行（reason_fields）
      「✗ task_server_lib.py 自证：loop-leak-check-on-bounce：」
  TASK_SERVER_BREAK=repeated-cap-hits-silent  末尾连着三次撞上限、还在重排的不记成要处置的提醒 repeated-cap-hits（task_server.py 的 attention_of_record）
      「✗ task_server_lib.py 自证：repeated-cap-hits-reminded-once：」
  TASK_SERVER_BREAK=long-run-idle-quarter-threshold  核数提醒的界是线程数的四分之一，不是 24/31（task_server.py 的 cores_short_of_threads）
      「✗ task_server_lib.py 自证：long-run-idle-cores-reported：」
  TASK_SERVER_BREAK=resume-declaration-any-word  续跑声明后面的非选项参数对上声明的词也算续跑（resumable_command）
      「✗ task_server_lib.py 自证：resume-declaration-first-word-only：」
  TASK_SERVER_BREAK=threads-over-registered-silent  不报 threads-over-registered（task_server.py 的 threads_over_registered_items）
      「✗ task_server_lib.py 自证：threads-over-registered-reported：」（推的那几种没推）、「✗ task_server_lib.py 自证：threads-over-registered-peer-reported：」（可跑的 50000 的那一条没推）
  TASK_SERVER_BREAK=threads-over-registered-many-processes-counted  线程总数远超登记数时不看占线程最多的那一个进程自己超没超（编译时一堆 rustc 加起来多的也报；task_server.py 的 note_threads）
      「✗ task_server_lib.py 自证：threads-over-registered-reported：」（占线程最多的进程只有 34 个线程的那一种也推了）
  TASK_SERVER_BREAK=threads-over-registered-idle-counted  线程总数远超登记数时不看其中可跑的超没超（只起一堆在睡的线程的编译器也报；task_server.py 的 note_threads）
      「✗ task_server_lib.py 自证：threads-over-registered-reported：」（可跑的只有 5 个的那一种也推了）、「✗ task_server_lib.py 自证：threads-over-registered-peer-reported：」（可跑的 0 的那一条也推了；推的，没复跑）
  TASK_SERVER_BREAK=cpu-load-io-unsplit  CPU 闸的排队原因不拆开写等 IO 的线程（余量报告 CPU 一项的 blocked_threads），只写「负载 N」（task_server.py 的 cpu_problem）
      「✗ task_server_lib.py 自证：cpu-reason-splits-io-wait：」
  TASK_SERVER_BREAK=booking-audit-skipped  记账对账不做：在跑的内存记账之和远大于实测的不判、不报、不推（task_server.py 的 audit_bookings）
      「✗ task_server_lib.py 自证：booking-drift-reported：」
  TASK_SERVER_BREAK=plan-key-compile-unmarked  分片计划那一步的命令键不带「这一步编不编」，编与不编的计划步落同一把键、互相取对方的记录（shard_child_key）
      「✗ task_server_lib.py 自证：plan-key-by-compiles：」（编的那一种取了不编的 2G 记录）
  TASK_SERVER_BREAK=booking-audit-young-counted  记账对账把起跑不满两分钟、记整份上限的尝试照整份上限算，不照实占算（task_server.py 的 audit_memory_bookings）
      「✗ task_server_lib.py 自证：booking-audit-young-not-drift：」（起跑 10 秒、各记 12G 的两条连三轮判成了记账漂移）
  TASK_SERVER_BREAK=aged-blocked-lets-others-start  排满时限还起不来的任务不挡排在它后面的：后面的照常判、照常起（task_server.py 的排队那一轮）
      「✗ task_server_lib.py 自证：aged-blocked-task-blocks-later：」
  TASK_SERVER_BREAK=jump-length-unchecked  排在前面的任务还没起时，后面的任务不看有没有上次运行时长、超没超过在跑的剩余时长，放得下就起（task_server.py 的 jump_refusal）
      「✗ task_server_lib.py 自证：jump-only-shorter-than-running：」（没有上次运行时长的与比在跑的剩余长的也先起了）
  TASK_SERVER_BREAK=jump-refused-any-resource  排在前面的任务只缺某一棵树的编译目录名额时，后面的不要编译目录的与别的树的也照缺内存、CPU 那一种判：没有上次运行时长的不插队（task_server.py 的 jump_refusal、place_queued_into）
      「✗ task_server_lib.py 自证：jump-only-same-resource：」（队头只缺编译目录名额时，不编译的、别的树的没有上次运行时长的也被判了不插队）
  TASK_SERVER_BREAK=running-young-booked-by-usage  起跑不满两分钟的在跑尝试不照整份上限记、照此刻实测记（task_server.py 的 running_booking）
      「✗ task_server_lib.py 自证：running-young-booked-at-cap：」（起跑 10 秒的记成了实测的 2G）
  TASK_SERVER_BREAK=machine-memory-line-ignored  整机可用内存与 swap 空闲都低于线也照派（task_server.py 的 machine_memory_problem）
      「✗ task_server_lib.py 自证：machine-memory-line-holds-dispatch：」
  TASK_SERVER_BREAK=external-kill-unrecognized  命令被不是服务发的 TERM、KILL 杀掉也照退出码记 command-failed，不读 earlyoom 与内核日志（task_server.py 的 external_kill_note）
      「✗ task_server_lib.py 自证：external-kill-recorded-and-retried：」
  TASK_SERVER_BREAK=bookings-ledger-unreported  /health 不报在跑记账的逐条明细（task_server.py 的 refresh_booking_ledger）
      「✗ task_server_lib.py 自证：booking-ledger-rows：」
  TASK_SERVER_BREAK=threads-record-succeeded-only  CPU 闸里新任务照历史核数记时只认跑成的几次，退非 0 跑到头的不算（task_server.py 的 threads_by_record）
      「✗ task_server_lib.py 自证：cpu-gate-new-task-counts-failed-runs：」
  TASK_SERVER_BREAK=confirm-retries-fixed  自证里确认单数对不上时只重试 CONFIRM_RETRIES 回，不随同会话在排在跑的条数放宽（task_server_selftest.py 的 World.confirm_submitted）
      「✗ task_server_lib.py 自证：confirm-retries-follow-churn：」
  TASK_SERVER_BREAK=promote-session-unchecked  task promote 不看是不是提交它的会话，别的会话也提前得了（task_server.py 的 promote）
      「✗ task_server_lib.py 自证：promote-skips-cpu-gate：」
  TASK_SERVER_BREAK=promote-cpu-gate-applied  task promote 提前过的照过 CPU 闸（task_server.py 的 cpu_judgement）
      「✗ task_server_lib.py 自证：promote-skips-cpu-gate：」
  TASK_SERVER_BREAK=promote-queue-order-ignored  排队的先后不看提前过没有（task_server.py 的 queue_order）
      「✗ task_server_lib.py 自证：promote-queue-order：」
  TASK_SERVER_BREAK=progress-line-progress-unnamed  在跑进度那一行不写日志不涨时读到的进展（哪份文件在涨、哪张卡在忙；task_server.py 的 progress_attempt_line）
      「✗ task_server_lib.py 自证：progress-line-names-progress：」
  TASK_SERVER_BREAK=open-file-write-mode-unchecked  进展读数把只读打开的文件也算写着的（task_server_lib.py 的 attempt_progress_readings）
      「✗ task_server_lib.py 自证：attempt-progress-readings：」
  TASK_SERVER_BREAK=open-file-growth-ignored  日志不涨时不看子进程写着的文件涨没涨（task_server.py 的 note_attempt_progress）
      「✗ task_server_lib.py 自证：quiet-log-busy-spared-by-progress：」
  TASK_SERVER_BREAK=quiet-log-busy-progress-ignored  日志长期不涨的提醒不看读到的进展，只从日志上一次涨起算（task_server.py 的 quiet_log_items）
      「✗ task_server_lib.py 自证：quiet-log-busy-spared-by-progress：」
  TASK_SERVER_BREAK=ssh-unmultiplexed  服务起的 ssh 不接复用连接的三项，host-dispatch.py 那一路的 PATH 前面不接复用层（task_server_lib.py 的 ssh_multiplex_options、ssh_multiplexed_environment）
      「✗ task_server_lib.py 自证：peer-ssh-multiplexed：」
  TASK_SERVER_BREAK=store-transaction-left-open  库的写事务出事不回滚，连接留在写事务里（task_server.py 的 Store.write_transaction）
      「✗ task_server_lib.py 自证：store-write-rolls-back-on-error：」
  TASK_SERVER_BREAK=disk-floor-host-judged-never  盘余量不到下限的台不算「回来可能放得下」，别的台都放不下就判 never-fits（task_server.py 的 judge_placement）
      「✗ task_server_lib.py 自证：disk-floor-peer-not-never-fits：」
  TASK_SERVER_BREAK=stall-cpu-read-before-log  判卡住每一轮开头读 CPU 之前不先把日志读到底（第二台交上一回解析的旧读数；task_server.py 的 fresh_cpu_usec）
      「✗ task_server_lib.py 自证：stall-reads-log-before-cpu：」
  TASK_SERVER_BREAK=peer-readings-overwritten  第二台的 IO 等待与线程读数每读一回日志就换成这一回读到的几行（前一回读到、还没被读走的丢了；task_server.py 的 read_log_content）
      「✗ task_server_lib.py 自证：peer-readings-merged-across-reads：」
  TASK_SERVER_BREAK=log-content-unlocked  读日志不拿按尝试的锁，两个线程同时从同一个位置读（task_server.py 的 log_content）
      「✗ task_server_lib.py 自证：log-content-serialized：」
  TASK_SERVER_BREAK=log-noise-marks-incomplete  第二台收尾打的峰值、留着编译目录那几行算日志在涨（task_server.py 的 read_log_content）
      「✗ task_server_lib.py 自证：log-noise-marks-not-growth：」
  TASK_SERVER_BREAK=runaway-targets-counting-children  空转进程的提醒找会话时分片的子任务也占最近 20 行（task_server.py 的 runaway_targets）
      「✗ task_server_lib.py 自证：runaway-targets-skip-children：」
  TASK_SERVER_BREAK=promote-unwoken  task promote 改完库不叫醒排队线程（task_server.py 的 promote）
      「✗ task_server_lib.py 自证：promote-wakes-and-promotes-shards：」
  TASK_SERVER_BREAK=promote-shards-refused  task promote 对分片的父任务照「不在排」拒，子任务不继承提前（task_server.py 的 promote、create_child）
      「✗ task_server_lib.py 自证：promote-wakes-and-promotes-shards：」
  TASK_SERVER_BREAK=promote-aged-blocks-queue  提前过的排满时限照算高优先，起不来就挡住后面全部（task_server.py 的 aged）
      「✗ task_server_lib.py 自证：promote-placement-rules：」
  TASK_SERVER_BREAK=promote-blocked-counts-as-earlier  提前过的起不来照算「排在前面还没起的」，后面的照插队判（task_server.py 的 blocks_later）
      「✗ task_server_lib.py 自证：promote-placement-rules：」
  TASK_SERVER_BREAK=pause-queue-positions-filtered  暂停放行一个会话时排位与留着的预测只照放行的那几条算（别的会话排着的排位丢了；task_server.py 的 place_queued_into）
      「✗ task_server_lib.py 自证：pause-keeps-queue-positions：」
  TASK_SERVER_BREAK=peer-probe-thread-unjoined  停服务时不等第二台的余量探测线程（task_server.py 的 Scheduler.stop）
      「✗ task_server_lib.py 自证：stop-joins-peer-probe：」
  TASK_SERVER_BREAK=peer-cooldown-ignored-off-scheduler  第二台上一回读不成、还在冷却里，收尾、核进程、删编译目录、空转那几处照样去连（task_server.py 的 Resources.peer_cooling_down）
      「✗ task_server_lib.py 自证：peer-cooldown-skips-connect：」
  TASK_SERVER_BREAK=command-wrapper-values-unskipped  剥外层包装时只认 env 的 -u、nice 的 -n 两种分开写的值（停在选项值上；task_server_lib.py 的 command_word_index）
      「✗ task_server_lib.py 自证：command-word-index-skips-option-values：」
  TASK_SERVER_BREAK=cargo-exclude-value-unskipped  整包测试的判法把 --exclude 的值当测试名（task_server_lib.py 的 whole_package_cargo_test）
      「✗ task_server_lib.py 自证：command-word-index-skips-option-values：」
  TASK_SERVER_BREAK=runner-start-error-uncaught  命令起不来时 attempt_runner 照抛出去、result.json 写不出（attempt_runner.py 的 main）
      「✗ task_server_lib.py 自证：runner-start-error-recorded：」
  TASK_SERVER_BREAK=runner-final-sample-skipped  命令退了之后只在一次都没采到时补采（attempt_runner.py 的 main）
      「✗ task_server_lib.py 自证：runner-final-sample-after-exit：」
  TASK_SERVER_BREAK=unit-stopped-still-booked  单元确认停了之后收尾没做完的尝试仍进 CPU 与内存的在跑记账（task_server.py 的 release_stopped_booking、booking_attempts）
      「✗ task_server_lib.py 自证：unit-stopped-booking-released：」
  TASK_SERVER_BREAK=cpu-running-cores-over-registered  CPU 闸里实测到的在跑尝试峰值段那一支不限登记的线程数（task_server.py 的 peak_segment_running_cores、latest_running_cores）
      「✗ task_server_lib.py 自证：cpu-gate-running-average-capped-by-registered：」
  TASK_SERVER_BREAK=cpu-cores-by-average  CPU 闸里在跑的照到此刻的平均在用核数记、记录照尝试的平均在用核数（cores）取平均，不记峰值段核数（task_server.py 的 measured_running_cores、threads_by_record）
      「✗ task_server_lib.py 自证：cpu-gate-peak-segment-cores：」
      「✗ task_server_lib.py 自证：cpu-gate-runnable-threads：」（平均那一支不看可跑线程数、记录读 cores 一栏，推的，没复跑）
  TASK_SERVER_BREAK=cpu-oversubscription-three  CPU 闸的线照核数 × CPU_OVERSUBSCRIPTION_LOOSE（6）判，不照 × CPU_OVERSUBSCRIPTION（3.4375；自证小世界各格设的倍数 2 也不认）（task_server.py 的 cpu_judgement）
      「✗ task_server_lib.py 自证：cpu-gate-line-twice-cores：」、「✗ task_server_lib.py 自证：cpu-gate-counts-running-threads：」、「✗ task_server_lib.py 自证：cpu-gate-new-task-by-recorded-cores：」、
      「✗ task_server_lib.py 自证：cpu-gate-new-task-counts-failed-runs：」
  TASK_SERVER_BREAK=peak-segment-reading-unchecked  收尾时累计 CPU 的接口交来的东西不核、照数用，不是数的抛错（task_server.py 的 finish_peak_segment_cores）
      「✗ task_server_lib.py 自证：finish-peak-segment-cpu-unread：」
  TASK_SERVER_BREAK=cpu-runnable-threads-unread  CPU 闸不记可跑线程数，只看在用核数（task_server.py 的 note_threads；机器挤满时在用核数被压低，重任务照低的数一条条放进来）
      「✗ task_server_lib.py 自证：cpu-gate-runnable-threads：」（在用 8 核、可跑线程 31 的那一种记 8）
  TASK_SERVER_BREAK=suspect-reported-memory-only  疑似卡住报过的是哪一段只记在内存里，服务重启后同一段再发一回（task_server.py 的 persist_restart_surviving_state、restore_restart_surviving_state）
      「✗ task_server_lib.py 自证：reminders-not-resent-after-restart：」
  TASK_SERVER_BREAK=health-notified-memory-only  服务健康每一样发给了哪几个会话只记在内存里，服务重启后同一样再发一回（同上）
      「✗ task_server_lib.py 自证：reminders-not-resent-after-restart：」
      「✗ task_server_lib.py 自证：gpu-health-survives-restart：」（新格，推的，没复跑）
      「✗ task_server_lib.py 自证：gpu-health-unread-host-survives-restart：」（新格，推的，没复跑）
  TASK_SERVER_BREAK=booking-drift-notified-memory-only  记账漂移每一件推给每个会话推到了哪道提醒门槛只记在内存里，服务重启后同一道门槛再推一回（同上）
      「✗ task_server_lib.py 自证：reminders-not-resent-after-restart：」
  TASK_SERVER_BREAK=long-run-split-silent  出结局时用时超过 7200 秒的不推 long-run-split（task_server.py 的 attention_of_record）
      「✗ task_server_lib.py 自证：long-run-split-reminded-at-outcome：」
  TASK_SERVER_BREAK=long-run-split-cap-hits-counted  long-run-split 的用时把撞了内存上限的那几次也累加（task_server.py 的 settled_run_seconds）
      「✗ task_server_lib.py 自证：long-run-split-reminded-at-outcome：」
  TASK_SERVER_BREAK=long-run-split-exempt-ignored  long-run-split 不看 .claude/long-run-split-exempt（task_server.py 的 long_run_split_exempt）
      「✗ task_server_lib.py 自证：long-run-split-reminded-at-outcome：」
  TASK_SERVER_BREAK=workspace-write-blocked-unreported  换钩子退 7（被上一趟没写完成的挡住）不推 workspace-write-blocked（task_server.py 的 attention_of_record）
      「✗ task_server_lib.py 自证：workspace-write-blocked-reminded：」、「✗ task_server_lib.py 自证：wrapped-hook-swap-reminded：」、「✗ task_server_lib.py 自证：wrapped-previous-swap-named：」、
      「✗ task_server_lib.py 自证：wrapped-writers-each-judged：」、「✗ task_server_lib.py 自证：wrapped-read-only-hook-swap-reminded：」
  TASK_SERVER_BREAK=previous-swap-running-unreported  换钩子退 9（上一趟还没写完成、起它的进程还在）与退 10 都不推 previous-swap-running（task_server.py 的 attention_of_record）
      「✗ task_server_lib.py 自证：previous-swap-running-reminded：」、「✗ task_server_lib.py 自证：wrapped-hook-swap-reminded：」、
      「✗ task_server_lib.py 自证：wrapped-writers-each-judged：」、「✗ task_server_lib.py 自证：wrapped-read-only-hook-swap-reminded：」
  TASK_SERVER_BREAK=swap-scratch-busy-unreported  换钩子退 10（同一个 --scratch 或这个仓的换上正被另一趟占着）不推 previous-swap-running，退 9 的照推（task_server.py 的 previous_swap_running_items）
      「✗ task_server_lib.py 自证：previous-swap-running-reminded：」、「✗ task_server_lib.py 自证：wrapped-hook-swap-reminded：」、
      「✗ task_server_lib.py 自证：wrapped-writers-each-judged：」、「✗ task_server_lib.py 自证：wrapped-read-only-hook-swap-reminded：」
  TASK_SERVER_BREAK=swap-busy-cause-unread  换钩子退 10 不读日志认占着的是哪一种，一律按同一个 --scratch 被占写（出路叫换 --scratch；task_server_scheduler_lifecycle.py 的 swap_busy_cause）
      「✗ task_server_lib.py 自证：previous-swap-running-reminded：」
  TASK_SERVER_BREAK=swap-busy-unknown-guessed  换钩子退 10 日志里认不出占着的是哪一种（读不到日志、两种拒绝说明都没有）时按同一个 --scratch 被占写，不写两种都可能（swap_busy_cause）
      「✗ task_server_lib.py 自证：previous-swap-running-reminded：」
  TASK_SERVER_BREAK=wrapped-hook-swap-unread  换钩子退 7、9、10 的两种提醒只认命令那一层的 hook-swap.py，包在 bash -c 串、命令位置上起的脚本正文里的不报
      （task_server_scheduler_lifecycle.py 的 hook_swap_command）
      「✗ task_server_lib.py 自证：wrapped-hook-swap-reminded：」、「✗ task_server_lib.py 自证：wrapped-writers-each-judged：」、
      「✗ task_server_lib.py 自证：wrapped-read-only-hook-swap-reminded：」
  TASK_SERVER_BREAK=wrapped-previous-swap-unnamed  换钩子退 7 找上一趟疑似是哪条任务只认命令那一层的 hook-swap.py，包着起的被杀的上一趟点不出
      （task_server_scheduler_lifecycle.py 的 workspace_write_blocked_items）
      「✗ task_server_lib.py 自证：wrapped-previous-swap-named：」、「✗ task_server_lib.py 自证：wrapped-writers-each-judged：」
  TASK_SERVER_BREAK=wrapped-writer-words-unused  改主工作区的命令被杀时写工具与点名要核的路径照命令那一层的词取：包着的写成 bash、点名 -c 的整串，打补丁那一支不读结局文件
      （task_server_scheduler_lifecycle.py 的 workspace_write_judged）
      「✗ task_server_lib.py 自证：wrapped-writer-interrupted-judged：」、「✗ task_server_lib.py 自证：wrapped-writers-each-judged：」、
      「✗ task_server_lib.py 自证：wrapped-patch-directory-resolved：」、「✗ task_server_lib.py 自证：wrapped-patch-directory-unresolvable：」、
      「✗ task_server_lib.py 自证：hook-swap-outcome-judged：」
  TASK_SERVER_BREAK=wrapped-writers-first-only  包在 bash -c 串、命令位置上起的脚本正文里的写工作区工具认到头一条写的就停，后面的不认
      （task_server_scheduler_building.py 的 wrapped_workspace_commands）：先打补丁后换钩子的串里，换钩子退 7、9、10 不报、被杀不报、当上一趟点不出；
      两条换钩子的串里只认前一条，结局归后一条时点错
      「✗ task_server_lib.py 自证：wrapped-writers-each-judged：」、「✗ task_server_lib.py 自证：hook-swap-outcome-judged：」
  TASK_SERVER_BREAK=wrapped-writer-paths-literal  包着的写工作区工具的词不代入赋值、起时的当前目录不跟 cd，一律照切出的原样与任务的工作目录取
      （task_server_scheduler_building.py 的 shell_runs）：补丁目录写成变量、位置参数或 cd 之后的相对路径的，结局文件拼错、当成还没开始换或打全了，被杀不报；
      当前目录推不出的那几形照任务的工作目录读结局文件、不报算不出
      「✗ task_server_lib.py 自证：wrapped-patch-directory-resolved：」、「✗ task_server_lib.py 自证：wrapped-patch-directory-unresolvable：」
  TASK_SERVER_BREAK=brace-group-not-layered  花括号组的 { } 照词两头的边缘字符剥掉、不当一层数（task_server_scheduler_building.py 的 shell_text_pieces）：
      接在管道上的组里 cd 了，之后的补丁目录照 cd 过去的目录拼、读不到结局文件，被杀不报
      「✗ task_server_lib.py 自证：wrapped-patch-directory-unresolvable：」
  TASK_SERVER_BREAK=heredoc-body-as-commands  不认 heredoc，正文的行照这一层平铺着跑的命令切（task_server_scheduler_building.py 的 shell_text_pieces）：
      正文里一行 cd 之后的补丁目录照 cd 过去的目录拼、读不到结局文件，被杀不报
      「✗ task_server_lib.py 自证：wrapped-patch-directory-unresolvable：」
  TASK_SERVER_BREAK=string-env-chdir-unread  串里那一条自己带 env -C 的照前面算出的当前目录交（task_server_scheduler_building.py 的 shell_text_runs）：
      env -C 起的打补丁的相对补丁目录照任务的工作目录拼、读不到结局文件，被杀不报
      「✗ task_server_lib.py 自证：wrapped-patch-directory-unresolvable：」
  TASK_SERVER_BREAK=wrapped-read-only-hook-swap-unread  换钩子退 7、9、10 的两种提醒认包着的换钩子只认写的那几条，带 --dry-run 这类只读参数的不报
      （task_server_scheduler_lifecycle.py 的 wrapped_hook_swap）
      「✗ task_server_lib.py 自证：wrapped-read-only-hook-swap-reminded：」
  TASK_SERVER_BREAK=attention-why-unclipped  提醒那一行的 why= 超过 ATTENTION_WHY_LIMIT 个字符的也照全文写，不截（attention_line）
      「✗ task_server_lib.py 自证：attention-line-why-clipped：」
  TASK_SERVER_BREAK=estimate-longest-succeeded  上次运行时长：退 0 的取最长一次，不取最近一次（task_server.py 的 last_run_seconds）
      「✗ task_server_lib.py 自证：last-run-seconds-from-records：」
  TASK_SERVER_BREAK=estimate-only-succeeded  上次运行时长：退非 0 的都不算（last_run_seconds）
      「✗ task_server_lib.py 自证：last-run-seconds-from-records：」
  TASK_SERVER_BREAK=estimate-latest-any-outcome  上次运行时长：不分退码取最近一次（last_run_seconds）
      「✗ task_server_lib.py 自证：last-run-seconds-from-records：」
  TASK_SERVER_BREAK=estimate-short-failure-counted  上次运行时长：没有退 0 的记录时，没跑满 FAILED_RUN_RECORD_MINIMUM_SECONDS 的失败也取（last_run_seconds）
      「✗ task_server_lib.py 自证：last-run-seconds-from-records：」
  TASK_SERVER_BREAK=estimate-failure-outside-window  上次运行时长：退非 0 的不限近 RECORD_WINDOW 次（last_run_seconds）
      「✗ task_server_lib.py 自证：last-run-seconds-from-records：」
  TASK_SERVER_BREAK=estimate-by-exact-command  上次运行时长只认命令逐词相同的尝试，同一把命令键、命令不同的不算（completed_runs）
      「✗ task_server_lib.py 自证：last-run-seconds-from-records：」
  TASK_SERVER_BREAK=io-pressure-ignored  放行会编译的任务不看那一台的磁盘压力：读数超过阈值也放行（task_server.py 的 io_pressure_problem）
      「✗ task_server_lib.py 自证：io-pressure-gate-by-host：」、「✗ task_server_lib.py 自证：io-pressure-queues-compiling-task：」
  TASK_SERVER_BREAK=io-pressure-applies-to-all  不会编译的任务也按磁盘压力排（io_pressure_problem）
      「✗ task_server_lib.py 自证：io-pressure-gate-by-host：」、「✗ task_server_lib.py 自证：io-pressure-queues-compiling-task：」
  TASK_SERVER_BREAK=io-pressure-limit-defaulted  多机配置没写 TASK_SERVER_IO_PRESSURE_LIMIT 时补缺省 50，照它读、照它排（lib.io_pressure_limit）
      「✗ task_server_lib.py 自证：io-pressure-gate-by-host：」、「✗ task_server_lib.py 自证：io-pressure-reading-recorded：」、「✗ task_server_lib.py 自证：io-pressure-queues-compiling-task：」
  TASK_SERVER_BREAK=io-pressure-peer-reads-local  第二台的磁盘压力判法读本机的读数，不读第二台自己的（io_pressure_problem）
      「✗ task_server_lib.py 自证：io-pressure-gate-by-host：」
  TASK_SERVER_BREAK=io-pressure-wait-unnamed  磁盘压力的排队原因不点名磁盘压力、哪一台、读数与阈值（io_pressure_problem；status 的「在等：」读它）
      「✗ task_server_lib.py 自证：io-pressure-gate-by-host：」、「✗ task_server_lib.py 自证：io-pressure-queues-compiling-task：」
  TASK_SERVER_BREAK=io-pressure-limit-unchecked  TASK_SERVER_IO_PRESSURE_LIMIT 写成不是数、不在 0 到 100 之间的也不判坏（lib.parse_optional_keys）
      「✗ task_server_lib.py 自证：io-pressure-reading-recorded：」
  TASK_SERVER_BREAK=cpu-small-slots-withheld  CPU 闸不给要核数少的名额：超线的一律排着（task_server.py 的 cpu_judgement）
      「✗ task_server_lib.py 自证：cpu-gate-small-task-slots：」
  TASK_SERVER_BREAK=cpu-small-slots-unlimited  要核数少的名额不设上限：占满 CPU_SMALL_TASK_SLOTS 条的也放行（cpu_judgement）
      「✗ task_server_lib.py 自证：cpu-gate-small-task-slots：」
  TASK_SERVER_BREAK=cpu-small-slot-heavy-still-counted  靠名额起的、实测超过 CPU_SMALL_TASK_CORES 核的也照占名额，不让出（task_server.py 的 small_slot_released）
      「✗ task_server_lib.py 自证：cpu-gate-small-task-slots：」
  TASK_SERVER_BREAK=entry-script-absolute-path-kept  本仓门禁与入口脚本写成绝对路径的照原样进键，不折成仓内相对路径：同一份脚本两种写法各是一个键（repository_script_word）
      「✗ task_server_lib.py 自证：entry-script-key-by-repository-path：」、「✗ task_server_lib.py 自证：selection-key-by-script：」
  TASK_SERVER_BREAK=gpu-cases-not-run-unread  日志里带 NOT_RUN_NO_GPU_ADAPTER 记号的行不认：结局说明不写本次未跑的用例、不推 gpu-cases-not-run（task_server.py 的 read_log_content）
      「✗ task_server_lib.py 自证：gpu-cases-not-run-reported：」、「✗ task_server_lib.py 自证：gpu-not-run-line-forms：」、「✗ task_server_lib.py 自证：gpu-not-run-marks-per-line：」
  TASK_SERVER_BREAK=gpu-not-run-line-start-only  标记行只认行首：行首带空白、接在别的字后面（结果行起头后面、别的输出后面、颜色码后面）的不认（task_server.py 的 gpu_not_run_test）
      「✗ task_server_lib.py 自证：gpu-not-run-line-forms：」、「✗ task_server_lib.py 自证：gpu-not-run-marks-per-line：」
  TASK_SERVER_BREAK=gpu-not-run-first-mark-only  一行里只算先出现的那一处记号：同一行里排在它后面的记号不数、不点名（task_server.py 的 gpu_not_run_test）
      「✗ task_server_lib.py 自证：gpu-not-run-marks-per-line：」
  TASK_SERVER_BREAK=gpu-not-run-word-edges-unchecked  标记不判两侧词界：更长的词里含着的 NOT_RUN_NO_GPU_ADAPTER（CANNOT_RUN_…、NOT_RUN_NO_GPU_ADAPTER_X、后面紧贴冒号）也当标记行（task_server.py 的 gpu_not_run_test）
      「✗ task_server_lib.py 自证：gpu-not-run-line-forms：」、「✗ task_server_lib.py 自证：gpu-cases-not-run-reported：」
  TASK_SERVER_BREAK=gpu-not-run-color-codes-kept  不先去 ANSI 颜色码：颜色码结尾的 m 紧贴在记号前面，前面有颜色码的那一行认不出（task_server.py 的 gpu_not_run_test）
      「✗ task_server_lib.py 自证：gpu-not-run-line-forms：」
  TASK_SERVER_BREAK=inbox-headline-state-omitted  推给会话的消息首行不写类别与任务此刻的状态（task_server.py 的 headline_class）
      「✗ task_server_lib.py 自证：inbox-headlines-tell-kind-state-round：」、
      「✗ task_server_lib.py 自证：anomaly-repushed-with-history：」、「✗ task_server_lib.py 自证：inbox-shard-attention-merged：」（这两格照首行里类别与状态那一段（「提醒（任务还在排，不是结局）」这类）认收到的那一条）
  TASK_SERVER_BREAK=inbox-headline-round-omitted  首行不写第几回与上一回在哪一天（task_server.py 的 push_round_tail）
      「✗ task_server_lib.py 自证：inbox-headlines-tell-kind-state-round：」、「✗ task_server_lib.py 自证：anomaly-repushed-with-history：」（首行钉着「；」「第 2 回」）
  TASK_SERVER_BREAK=inbox-headline-name-omitted  首行只写英文种类名、不写中文名（task_server.py 的 kind_label）
      「✗ task_server_lib.py 自证：inbox-headlines-tell-kind-state-round：」
  TASK_SERVER_BREAK=inbox-flag-lines-before-evidence  确认标识那几行（长跑已确认等）排在依据与出路之前（task_server.py 的 attention_text、outcome_text）
      「✗ task_server_lib.py 自证：inbox-headlines-tell-kind-state-round：」
  TASK_SERVER_BREAK=peer-external-kill-line-ignored  第二台日志里的 MULTI_HOST_RUN_PEER_EXTERNAL_KILL 那一行不认（task_server.py 的 peer_external_kill_note）
      「✗ task_server_lib.py 自证：external-kill-peer-line-recorded-and-retried：」
  TASK_SERVER_BREAK=shell-inner-scope-unread  确认单与提交时拒 ⑦ 不读 bash -c 里命令位置上的范围（task_server_lib.py 的 shell_inner_commands）
      「✗ task_server_lib.py 自证：confirm-sheet-reads-bash-c-inner-commands：」、「✗ task_server_lib.py 自证：unnamed-cargo-target-refused：」
  TASK_SERVER_BREAK=workspace-writer-cap-unpadded  换钩子、打补丁的工具内存上限不留余量（task_server.py 的 workspace_writer_floor）
      「✗ task_server_lib.py 自证：workspace-writer-cap-has-headroom：」
  TASK_SERVER_BREAK=shell-string-writers-unread  认改主工作区的工具不切 bash -c 的串，只看命令那一层的词（task_server_scheduler_building.py 的 wrapped_workspace_commands）：
      串里写着 hook-swap.py、stage-mine.py 的撞上限照原样重排
      「✗ task_server_lib.py 自证：wrapped-writer-shell-string-not-retried：」
      「✗ task_server_lib.py 自证：wrapped-hook-swap-reminded：」、「✗ task_server_lib.py 自证：wrapped-previous-swap-named：」、「✗ task_server_lib.py 自证：wrapped-writer-interrupted-judged：」、
      「✗ task_server_lib.py 自证：wrapped-writers-each-judged：」、「✗ task_server_lib.py 自证：wrapped-patch-directory-resolved：」、
      「✗ task_server_lib.py 自证：wrapped-read-only-hook-swap-reminded：」、「✗ task_server_lib.py 自证：wrapped-patch-directory-unresolvable：」、
      「✗ task_server_lib.py 自证：hook-swap-outcome-judged：」
  TASK_SERVER_BREAK=script-body-writers-unread  认改主工作区的工具不读命令位置上起的脚本正文（task_server_scheduler_building.py 的 script_body_workspace_commands）：
      bash、python3 起与直接起的串行脚本里写着换钩子、打补丁的撞上限照原样重排
      「✗ task_server_lib.py 自证：wrapped-writer-script-body-not-retried：」
      「✗ task_server_lib.py 自证：wrapped-hook-swap-reminded：」、「✗ task_server_lib.py 自证：wrapped-previous-swap-named：」、「✗ task_server_lib.py 自证：wrapped-writer-interrupted-judged：」、
      「✗ task_server_lib.py 自证：wrapped-patch-directory-resolved：」、「✗ task_server_lib.py 自证：wrapped-read-only-hook-swap-reminded：」
  TASK_SERVER_BREAK=key-stats-alternation-ignored  control key-stats 按任务数交替的键一律数 0（task_server_lib.py 的 key_stats）
      「✗ task_server_lib.py 自证：control-key-stats-reported：」、「✗ task_server_lib.py 自证：control-key-stats-by-task：」
  TASK_SERVER_BREAK=key-stats-metric-unfiltered  control key-stats 不筛峰值口径：老口径的尝试也进统计（task_server_lib.py 的 key_stats）
      「✗ task_server_lib.py 自证：control-key-stats-by-task：」
  TASK_SERVER_BREAK=key-stats-retries-unfolded  control key-stats 按任务数的交替与「撞过、最近一个任务跑成」也逐次尝试数，同一任务的重试不折成一条（task_server_lib.py 的 key_stats）
      「✗ task_server_lib.py 自证：control-key-stats-by-task：」
  TASK_SERVER_BREAK=key-stats-since-ignored  control key-stats --since 不筛起跑时刻，统计与导出照全部尝试（task_server_lib.py 的 key_stats_after）
      「✗ task_server_lib.py 自证：control-key-stats-by-task：」、「✗ task_server_lib.py 自证：control-key-stats-dump：」
  TASK_SERVER_BREAK=key-stats-dump-overwrites  control key-stats --dump 不排他新建：文件已在照覆盖、退 0（task-server/control 的 key_stats_dump）
      「✗ task_server_lib.py 自证：control-key-stats-dump：」
  TASK_SERVER_BREAK=hard-limit-stop-once  判停的那一次只停一回，没停成的不再停（task_server.py 的 stop_over_limit_attempt）
      「✗ task_server_lib.py 自证：run-hard-limit-stop-retried：」
  TASK_SERVER_BREAK=run-hard-limit-skipped  跑着的判点（各次尝试，与弄坏开关 shard-whole-by-wall-clock 下分片的父任务）都不判四小时硬上限（task_server.py 的 run_hard_limit_verdict、parent_run_hard_limit）
      「✗ task_server_lib.py 自证：shard-hard-limit-by-each-piece：」、「✗ task_server_lib.py 自证：run-hard-limit-stopped：」、「✗ task_server_lib.py 自证：run-hard-limit-why-still-stops：」、「✗ task_server_lib.py 自证：run-hard-limit-writer-stopped：」、「✗ task_server_lib.py 自证：run-hard-limit-shard-stopped：」、「✗ task_server_lib.py 自证：run-hard-limit-applies-after-restart：」、「✗ task_server_lib.py 自证：run-hard-limit-stop-retried：」
  TASK_SERVER_BREAK=run-hard-limit-full-gate-judged  带 --full-gate 的也判四小时硬上限（task_server.py 的 run_hard_limit_verdict、parent_run_hard_limit）
      「✗ task_server_lib.py 自证：run-hard-limit-full-gate-not-stopped：」
  TASK_SERVER_BREAK=run-hard-limit-confirmed-spared  确认过的（--why、long_run_confirmed）满四小时不停（task_server.py 的 run_hard_limit_verdict）
      「✗ task_server_lib.py 自证：run-hard-limit-why-still-stops：」
  TASK_SERVER_BREAK=run-hard-limit-writers-spared  改主工作区的命令满四小时不停（task_server.py 的 run_hard_limit_verdict）
      「✗ task_server_lib.py 自证：run-hard-limit-writer-stopped：」
  TASK_SERVER_BREAK=run-hard-limit-children-spared  分片的子任务不按自己的累计判四小时硬上限（task_server.py 的 run_hard_limit_verdict）
      「✗ task_server_lib.py 自证：shard-hard-limit-by-each-piece：」
  TASK_SERVER_BREAK=run-hard-limit-parent-summed  父任务整件（弄坏开关 shard-whole-by-wall-clock 那一支）取片之和，不取挂钟（covered_seconds）
      「✗ task_server_lib.py 自证：run-hard-limit-arithmetic：」
  TASK_SERVER_BREAK=run-hard-limit-outcome-unmarked  照四小时硬上限停的那一次尝试不记 RUN_HARD_LIMIT_OUTCOME，照被信号杀的结局走（task_server.py 的 finish_attempt）
      「✗ task_server_lib.py 自证：run-hard-limit-stopped：」、「✗ task_server_lib.py 自证：run-hard-limit-why-still-stops：」、「✗ task_server_lib.py 自证：run-hard-limit-writer-stopped：」、「✗ task_server_lib.py 自证：run-hard-limit-shard-stopped：」、「✗ task_server_lib.py 自证：run-hard-limit-applies-after-restart：」、「✗ task_server_lib.py 自证：run-hard-limit-stop-retried：」
  TASK_SERVER_BREAK=run-hard-limit-writer-unreminded  over-four-hours 不报 workspace-write-interrupted（task_server.py 的 workspace_write_item）
      「✗ task_server_lib.py 自证：run-hard-limit-writer-stopped：」
  TASK_SERVER_BREAK=run-hard-limit-exclusive  恰好等于四小时不算到（over_run_hard_limit）
      「✗ task_server_lib.py 自证：run-hard-limit-arithmetic：」
  TASK_SERVER_BREAK=run-hard-limit-loose  比四小时少一秒也算到（over_run_hard_limit）
      「✗ task_server_lib.py 自证：run-hard-limit-arithmetic：」
  TASK_SERVER_BREAK=scope-inside-script-unnamed  确认单里脚本自己定范围的也写「整包、不指定范围」（named_scope_text）
      「✗ task_server_lib.py 自证：confirmation-scope-text：」
  TASK_SERVER_BREAK=crash-scope-unreported  确认单不打崩溃用例与域（crash_scope_text）
      「✗ task_server_lib.py 自证：confirmation-scope-text：」
  TASK_SERVER_BREAK=retired-outcome-kinds-unknown  anomaly_advice 不认 RETIRED_OUTCOME_KINDS，库里记着退役结局的旧任务 status 抛 KeyError
      「✗ task_server_lib.py 自证：retired-outcome-status-generic：」
  TASK_SERVER_BREAK=shard-binaries-kept  分片父任务有了结局与 task clean 时都不删计划目录下的 test-binaries/（task_server.py 的 shard_binaries_removal）
      「✗ task_server_lib.py 自证：shard-binaries-removed-at-outcome：」（取消整件之后那一份还在，task clean 之后也还在）
  TASK_SERVER_BREAK=shard-binaries-path-unchecked  删计划目录下的 test-binaries/ 时不核真路径，是指到别处的符号链接也照真路径删（task_server.py 的 shard_binaries_removal）
      「✗ task_server_lib.py 自证：shard-binaries-removed-at-outcome：」（计划目录外那一份被删了）
  TASK_SERVER_BREAK=runner-threads-state-unread  attempt_runner 数线程不看状态字段，每个线程都算可跑的（attempt_runner.py 的 note_threads；说明里的 threads_state_unread）
      「✗ task_server_lib.py 自证：runner-threads-file-counts：」（可跑的数成线程总数）
  TASK_SERVER_BREAK=nested-units-unbound  套在外层单元里的服务起尝试单元不带 BindsTo、After，外层停了尝试单元照跑（task_server.py 的 nested_outer_unit）
      「✗ task_server_lib.py 自证：selftest-units-stop-with-outer：」
  TASK_SERVER_BREAK=running-booked-at-cap  在跑的尝试一律照它的上限记账，不看此刻实测的用量（task_server.py 的 running_booking）
      「✗ task_server_lib.py 自证：running-booked-at-measured-usage：」、「✗ task_server_lib.py 自证：booking-ledger-rows：」、
      「✗ task_server_lib.py 自证：admission-by-measured-bookings：」、「✗ task_server_lib.py 自证：memory-wait-names-both-numbers：」、
      「✗ task_server_lib.py 自证：peer-scope-missing-booked-at-cap：」（末一格推的，没复跑）
  TASK_SERVER_BREAK=running-booked-with-margin  有 scope 的在跑尝试在此刻实测上再加 1 GiB 记（RUNNING_BOOKING_MARGIN_BREAK_BYTES，不超过上限）（task_server.py 的 running_booking）
      「✗ task_server_lib.py 自证：running-booked-at-measured-usage：」、「✗ task_server_lib.py 自证：booking-ledger-rows：」、
      「✗ task_server_lib.py 自证：admission-by-measured-bookings：」、「✗ task_server_lib.py 自证：memory-wait-names-both-numbers：」
  TASK_SERVER_BREAK=cargo-test-key-whole-command  cargo test 不归同类，整条命令当键：点名不同测试目标的同包同档命令各是一个键、各从没记录的 4096M 起（cargo_test_class_words）
      「✗ task_server_lib.py 自证：cargo-test-key-by-package：」
  TASK_SERVER_BREAK=commit-gate-without-full-gate-accepted  不指定范围的提交时门禁没带 --full-gate 也照收（带整包确认标识或什么都不带都放过；task_server.py 的 whole_package_refusal）
      「✗ task_server_lib.py 自证：full-gate-flag：」
  TASK_SERVER_BREAK=commit-gate-query-forms-judged  提交时门禁只查不跑的写法（--selftest、--list、--list-items、--help 这类）也照不指定范围的那一趟判（query_only_entry 一律交 False：commit_gate_entry、unscoped_run_entry）
      「✗ task_server_lib.py 自证：commit-gate-query-forms-accepted：」（新格，推的，没复跑）
  TASK_SERVER_BREAK=selection-key-whole-command  本仓门禁与入口脚本的选法（--check、--item、--only 与它们的值）照原样进键：点名不同格、不同项的同一道各是一个键、各从没记录的起（selection_class_words）
      「✗ task_server_lib.py 自证：selection-key-by-script：」
  TASK_SERVER_BREAK=selection-positionals-in-place  本仓门禁与入口脚本命令里不以 - 起头的位置参数（仓根）留在原位进键：仓根写在选项前与写在选项后各是一个键（selection_class_words）
      「✗ task_server_lib.py 自证：selection-key-positionals-after-options：」
  TASK_SERVER_BREAK=plan-key-shrunk-domain-kept  分片计划那一步的命令键照留缩域变量 SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO 的赋值：缩域深度不同的两条父命令各一把计划键（key_words 的 plan_step、task_server.py 的 child_key_text）
      「✗ task_server_lib.py 自证：plan-key-without-shrunk-domain：」
  TASK_SERVER_BREAK=progress-ignored-for-stall  判卡住不看进程内存在涨（task_server.py 的 judge_stall_stop、activity_text）
      「✗ task_server_lib.py 自证：memory-growth-not-stalled：」
  TASK_SERVER_BREAK=stall-window-head-unread  判卡住开新窗口的那一轮不读头一份进程内存与峰值，下一轮才记：一轮排队比窗口还长时窗口里只有一份读数，在涨的也当没进展（task_server.py 的 note_window_head）
      「✗ task_server_lib.py 自证：memory-window-head-read-at-open：」
  TASK_SERVER_BREAK=cpu-restart-unmeasured-by-record  CPU 闸里起跑已满两分钟、服务还没量满一段的在跑尝试（服务刚重启）不照累计 CPU ÷ 已跑秒数记，退回命令键的记录或登记数（改前的样子；task_server.py 的 running_threads）
      「✗ task_server_lib.py 自证：cpu-gate-running-after-restart-by-average：」（没有记录的两条各记成了 31 线程）
  TASK_SERVER_BREAK=cpu-unrecorded-by-one-core  CPU 闸里没有核数记录的新命令先按 1 核记，不按登记的线程数（改前的样子；task_server.py 的 threads_by_record）
      「✗ task_server_lib.py 自证：cpu-gate-unrecorded-by-registered-threads：」（没有记录的记成了 1 线程）
  TASK_SERVER_BREAK=cpu-latest-runnable-ignored  CPU 闸里在跑的那一条只照最近一段的在用核数记，不与最近一回的可跑线程数取大（task_server.py 的 latest_running_cores）
      「✗ task_server_lib.py 自证：cpu-gate-running-by-latest-reading：」（可跑 25 个的记成了 18 线程）
  TASK_SERVER_BREAK=cpu-wanted-by-registered-threads  CPU 闸里新起的这一条一律照登记的线程数记，不看同一命令键记下的峰值段核数（task_server.py 的 admission_threads）
      「✗ task_server_lib.py 自证：cpu-gate-new-task-by-recorded-cores：」
      「✗ task_server_lib.py 自证：cpu-gate-runnable-threads：」（新任务一律记 31，推的，没复跑）
  TASK_SERVER_BREAK=cpu-running-unmeasured-registered  CPU 闸里在跑的那几条没实测到核数的（起跑未满 CPU_MEASURED_AFTER_SECONDS、量不到 CPU），一律照登记线程数记，不取同一命令键的记录
      （task_server.py 的 running_threads、unmeasured_threads、threads_by_record）
      「✗ task_server_lib.py 自证：cpu-gate-running-started-by-recorded-cores：」
  TASK_SERVER_BREAK=heartbeat-counted-as-log  心跳行（TASK_SERVER_HEARTBEAT 起头）照算日志在涨，判卡住的「日志不涨」被它盖住（task_server.py 的 log_content）
      「✗ task_server_lib.py 自证：heartbeat-lines-not-log-growth：」
  TASK_SERVER_BREAK=verdict-library-progress-uncounted  判定库进度行（PRUNE_SERVER_CLIENT_PROGRESS 起头）也不算日志在涨：只等判定库回话、没有别的输出的命令被当成日志不涨判卡住（task_server.py 的 log_content）
      「✗ task_server_lib.py 自证：verdict-library-progress-not-stalled：」
  TASK_SERVER_BREAK=long-run-why-children-reminded  分片的子任务不看父任务确认过没有：父任务确认了跑过两小时，子任务满界照样提醒（task_server.py 的 long_run_confirmed_of、run_limit_exempt）
      「✗ task_server_lib.py 自证：long-run-confirmed-covers-shards：」
  TASK_SERVER_BREAK=record-shard-count-unchecked  定额度读片的记录时不核片数：片数不同的那几趟的同一片号也拿来取近 RECORD_WINDOW 次最大（task_server.py 的 record_fits_task）
      「✗ task_server_lib.py 自证：shard-peak-record-count-matched：」
  TASK_SERVER_BREAK=signal-exit-unnamed  被信号终止的不写信号名、不写服务发没发过停的信号（task_server.py 的 signal_note）
      「✗ task_server_lib.py 自证：signal-exit-named：」
  TASK_SERVER_BREAK=slice-floor-ignored  不判 slice 兜底：slice 实测余量不到下限照样派（task_server.py 的 slice_floor_problem）
      「✗ task_server_lib.py 自证：running-booked-unscoped-at-cap：」
  TASK_SERVER_BREAK=shard-whole-by-wall-clock  分片的父任务整件按有片在跑的挂钟算四小时硬上限，满界撤在排的片（task_server.py 的 parent_run_hard_limit）
      「✗ task_server_lib.py 自证：shard-hard-limit-by-each-piece：」、「✗ task_server_lib.py 自证：run-hard-limit-shard-stopped：」
  TASK_SERVER_BREAK=shard-key-count-kept  片的命令键带片数「第<片号>片共<片数>片」，片数一变各片都换新键（shard_child_key）
      「✗ task_server_lib.py 自证：shard-key-without-count：」
  TASK_SERVER_BREAK=stopped-peaks-not-counted  被停、判红的那几次量到的峰值不当下限（task_server.py 的 stopped_peak_floor）
      「✗ task_server_lib.py 自证：stopped-peaks-floor-record：」
  TASK_SERVER_BREAK=compiles-declaration-ignored  不读入口会编的声明：声明了的入口照没声明认，不给编译目录（declared_compiles）
      「✗ task_server_lib.py 自证：compiles-declared-entry：」
  TASK_SERVER_BREAK=parent-cancel-unchecked-at-claim  起子任务那一个事务不判父任务的 cancel_requested，整件正在取消时排着的片照起（task_server.py 的 Store.claim_attempt）
      「✗ task_server_lib.py 自证：parent-cancelling-child-not-claimed：」
  TASK_SERVER_BREAK=cancel-origin-unread  结局 cancelled 的理由行不看 cancel_origin，服务撤的也写提交它的会话（reason_fields）
      「✗ task_server_lib.py 自证：parent-cancelling-child-not-claimed：」
  TASK_SERVER_BREAK=cpu-threads-settling-only  CPU 闸只记起了不到 5 秒的尝试的线程、与负载相加，起了一会儿的不记（task_server.py 的 cpu_problem）
      「✗ task_server_lib.py 自证：cpu-gate-counts-running-threads：」
  TASK_SERVER_BREAK=cpu-unread-passed  报告里读不到核数或负载时 CPU 闸放行（task_server.py 的 cpu_problem）
      「✗ task_server_lib.py 自证：cpu-gate-counts-running-threads：」
  TASK_SERVER_BREAK=lookup-indexes-skipped  库不建按 task_id、命令键、会话的索引（task_server.py 的 Store.initialize）
      「✗ task_server_lib.py 自证：lookup-indexes-used：」
  TASK_SERVER_BREAK=running-attempts-reread  在跑的尝试每次现读，库没变也重读（task_server.py 的 Store.running_attempts）
      「✗ task_server_lib.py 自证：running-attempts-read-once：」
  TASK_SERVER_BREAK=budget-source-every-pass  预算那一判每一轮先算记账来源（按命令键扫库），不记提醒的也算（task_server.py 的 budget_check）
      「✗ task_server_lib.py 自证：budget-source-only-when-reported：」
  TASK_SERVER_BREAK=peer-probe-in-scheduler  第二台的余量在调用方的线程里现连，读不成不记、下一轮照样再连（task_server.py 的 Resources.peer_snapshot_entry）
      「✗ task_server_lib.py 自证：peer-probe-off-scheduler：」
  TASK_SERVER_BREAK=repass-kept-when-idle  队空、暂停着提前回的那一轮不清 repass_soon（task_server.py 的 place_queued_into）
      「✗ task_server_lib.py 自证：repass-cleared-when-idle：」
  TASK_SERVER_BREAK=finish-in-scheduler  收尾（第二台清理回执、留下的进程回读、第二台那一个编译目录留没留着）在排队线程里当场做，第二台卡住时排队线程跟着卡住（task_server.py 的 finish_off_loop）
      「✗ task_server_lib.py 自证：peer-cleanup-off-scheduler：」
  TASK_SERVER_BREAK=cpu-threads-registered-only  CPU 闸里在跑的一律照登记线程数记，起跑满了 CPU_MEASURED_AFTER_SECONDS 的也不照实测核数（task_server.py 的 running_threads）
      「✗ task_server_lib.py 自证：cpu-gate-measured-cores：」、「✗ task_server_lib.py 自证：cpu-gate-runnable-threads：」（后两格推的，没复跑）
  TASK_SERVER_BREAK=unit-unreadable-exit-file-unread  读不到单元状态的那一段不认 result.json，命令退了、核到没了也照没留结果记 launcher-lost（task_server.py 的 reconcile）
      「✗ task_server_lib.py 自证：unit-unreadable-exit-file-settles：」、「✗ task_server_lib.py 自证：unit-unreadable-running-reminded：」
  TASK_SERVER_BREAK=unit-unreadable-absent-cgroup-unconfirmed  不经 systemd 核进程时 cgroup 目录不在不算空、算核不出，进程都没了的那一次一直照记 running（attempt_presence_readings）
      「✗ task_server_lib.py 自证：unit-unreadable-gone-recorded-lost：」
  TASK_SERVER_BREAK=unit-unreadable-lost-unverified  读不到单元状态就当场照没留结果收尾、记 launcher-lost，不核进程，收尾照常经 systemctl 回读、停掉单元里的进程（task_server.py 的 reconcile）
      「✗ task_server_lib.py 自证：unit-unreadable-running-reminded：」、「✗ task_server_lib.py 自证：unit-unreadable-unverified-health-reported：」
  TASK_SERVER_BREAK=unit-unreadable-unreminded  读不到单元状态不记成要处置的提醒，会话收不到 unit-unreadable（task_server.py 的 attention_of_record）
      「✗ task_server_lib.py 自证：unit-unreadable-running-reminded：」
  TASK_SERVER_BREAK=unit-unreadable-health-unreported  /health 不报读不到单元状态，problem 里没有那一句、unit_unreadable 栏是空表，control check 照退 0（task_server.py 的 health）
      「✗ task_server_lib.py 自证：unit-unreadable-unverified-health-reported：」
  TASK_SERVER_BREAK=memory-wait-text-unnamed  内存放不下的排队原因照 judge_host 与公式各自原来那一句写，不写账面与 slice 实测两个数各是什么、不点名记账最多的在跑任务（task_server.py 的 memory_wait_text）
      「✗ task_server_lib.py 自证：memory-wait-names-both-numbers：」、「✗ task_server_lib.py 自证：admission-by-measured-bookings：」
  TASK_SERVER_BREAK=memory-cgroup-kill-counted  内核「Memory cgroup out of memory」那一行（某个内存组到了它自己的上限）也当被外部信号杀掉（task_server_lib.py 的 external_kill_pid、task_server.py 的 peer_external_kill_note）
      「✗ task_server_lib.py 自证：external-kill-peer-line-recorded-and-retried：」
自证另有几格不配开关：selftest-variables-refused（判的守卫在读开关之前，开关在别处设了本身就被它拒）；type-table-covered（静态核错误类型表里每一种都有格喂到）；control-watch-removed（control watch 只剩退 2 打出路句这一支，没有能弄坏的判法）；
anomaly-table-matches-kinds（静态核异常表 ANOMALY_TABLE_TEXT 的种类与 ATTENTION_KINDS、OUTCOME_TYPES、SUBMIT_REFUSAL_KINDS 逐行对得上）。
"""
import os
import sys

sys.dont_write_bytecode = True

import base64  # noqa: E402
import calendar  # noqa: E402
import contextlib  # noqa: E402
import hashlib  # noqa: E402
import hmac  # noqa: E402
import http.client  # noqa: E402
import importlib.util  # noqa: E402
import inspect  # noqa: E402
import json  # noqa: E402
import math  # noqa: E402
import re  # noqa: E402
import secrets  # noqa: E402
import shlex  # noqa: E402
import signal  # noqa: E402
import socket  # noqa: E402
import stat  # noqa: E402
import subprocess  # noqa: E402
import time  # noqa: E402
import tomllib  # noqa: E402
from dataclasses import dataclass  # noqa: E402


if os.path.dirname(os.path.realpath(__file__)) not in sys.path:   # 拆出的几份模块与这一份同目录；按文件路径载入这一份的调用方也靠它找到它们
    sys.path.append(os.path.dirname(os.path.realpath(__file__)))
from task_server_lib_constants import (
    ADDRESSES_LIBRARY, ADMITTED_VARIABLE, AMBIENT_PREFIXES, AMBIENT_VARIABLES, ANOMALY_ACTIONS, ANOMALY_GENERIC_ADVICE, ANOMALY_ROW,
    ANOMALY_TABLE_HEAD, ANOMALY_TABLE_LOADED, ANOMALY_TABLE_TEXT, ANOMALY_TASK_PLACEHOLDER, ASSIGNMENT_WORD, ATTEMPT_LABEL_VARIABLE,
    ATTEMPT_PRESENCE_PEER_SECONDS, ATTEMPT_PRESENCE_PROBE_MARK, ATTEMPT_PRESENCE_TREE_DEPTH, ATTEMPT_PROGRESS_FILE_LIMIT,
    ATTEMPT_PROGRESS_PEER_SECONDS, ATTEMPT_PROGRESS_PROBE_MARK, ATTENTION_EXIT, ATTENTION_KINDS, ATTENTION_MARK, ATTENTION_TYPE, ATTENTION_WHY_LIMIT,
    AWAITING_CONFIRMATION_STATE, BOUNCE_NEW_TASK_REMINDER, BOUNCING_ATTENTION_KINDS, BREAK_VARIABLE, BY_HOST_FORWARDED_BREAKS, CANCEL_ORIGIN_TEXTS,
    CAP_GROWTH_FACTOR, CAP_HIT_REAL_PEAK_FRACTION, CARD_CHOICE_VARIABLES, CARD_DECLARATION_MARKS, CARD_DECLARATION_REFUSAL, CARGO_TEST_SUBCOMMANDS,
    CARGO_VALUE_OPTIONS, CARRIED_PREFIXES, CARRIED_VARIABLES, CLIENT_SCRIPT, COMMIT_GATE_ENTRIES, COMPILES_DECLARED_WORD, CONFIGURATION_STEP,
    CONFIRM_NEEDED_LINE, CONFIRM_NEEDED_MARK, CONFIRM_RULE_SELFTEST_VARIABLE, CONTROL_SCRIPT, COPY_PROOF_OWNER_NAME, COPY_PROOF_OWNER_STALE_SECONDS,
    COPY_PROOF_PENDING_NAME, CPU_OVERSUBSCRIPTION_VARIABLE, CRASH_CASES_ONLY_ARGUMENT, CRASH_CASE_ITEM_PREFIX, CRASH_DOMAIN_ARGUMENT,
    CRASH_GATE_CASES_ONLY_FORM, CRASH_GATE_FEATURE_ITEM_PREFIX, CRASH_SCOPE_ENTRIES, DATABASE_NAME, DATA_ROOT_KEY, DATA_ROOT_VARIABLE,
    DECLARED_COMPILES_MARK, DECLARED_HEADER_LINES, DECLARED_RESUMABLE_MARK, DECLARED_SHARDS_MARK, DIRECT_QUEUE_MARK, ENTRY_QUERY_ARGUMENTS,
    EXIT_PEAK_FILE_NAME, EXIT_TYPES, EXPANDED_VARIABLES, EXTERNAL_KILL_LINE, EXTERNAL_KILL_OUTCOME, EXTERNAL_KILL_RETRIES,
    FAILED_RUN_RECORD_MINIMUM_SECONDS, GIBIBYTE, GPU_BLOCK_CLAIMS_KEPT, GPU_BLOCK_CLAIMS_LET_GO, GPU_BLOCK_CLAIM_FIELD,
    GPU_BLOCK_NOT_JUDGEABLE_ALERT_KIND, GPU_BLOCK_NOT_JUDGEABLE_KEY_FIELDS, GPU_BLOCK_NOT_JUDGEABLE_OF_A_HOST_PREFIX, GPU_BLOCK_NUMBERS_SHOWN,
    GPU_CARDS_FAILED_READ_PREFIX, GPU_CARDS_FAILED_READ_STATES, GPU_CARDS_FAILED_UNREAD_PREFIX, GPU_CARD_FAILED_ALERT_KIND, GPU_CARD_FIELD,
    GPU_FAILED_CARD_OF_A_HOST_PREFIX, GPU_SERVICE_CONTROL, GPU_SERVICE_SOURCE, GPU_THIS_HOST_LABEL, HEARTBEAT_LOG_MARK, HOOKS_DIRECTORY,
    HOSTS_DIRECTORY, HOSTS_KEY, HOST_NAMES, INBOX_LOG_MARK, INBOX_SOCKET_VARIABLE, INBOX_TOKEN_VARIABLE, INSTALL_COMMAND, INSTALL_PACKAGES,
    IO_PRESSURE_LIMIT_KEY, IO_PRESSURE_SELFTEST_DEFAULT, KEY_NEUTRAL_VARIABLES, KEY_VARIABLE, KIND_NAMES, KNOWN_BREAK_ITEMS, LIBTEST_VALUE_OPTIONS,
    LONG_RUN_LOG_MARK, LONG_RUN_SECONDS, LONG_RUN_SPLIT_SECONDS, LONG_TASK_HELD_REASON, LONG_TASK_RECORD_REASON, LONG_TASK_SECONDS,
    LONG_TASK_SHARD_REASON, LONG_TASK_UNRECORDED_ITEMS, LONG_TASK_UNRECORDED_REASON, LONG_TASK_UNRECORDED_TESTS, LONG_TASK_WHOLE_REASON,
    LOOP_LEAK_BOUNCE_CAP_HITS, LOOP_LEAK_CHECK_HEAD, LOOP_LEAK_KINDS, LOOP_LEAK_OUTCOMES, LOOP_LEAK_REMINDER_CAP_HITS, MAXIMUM_SHARDS, MEBIBYTE,
    MEMORY_CAP_RETRIES, MEMORY_CAP_SCOPE_PREFIX, MEMORY_CAP_VARIABLE, MEMORY_CGROUP_KILL_TEXT, MEMORY_FLOOR_BYTES, MEMORY_FLOOR_RAISED_BYTES,
    MEMORY_FLOOR_VARIABLE, MEMORY_PEAK_FILE_NAME, MEMORY_STAT_NAMES, MEMORY_WRAPPER, MERGE_ALL_PACKAGES_ARGUMENT, MERGE_VALUE_OPTIONS,
    MULTI_HOST_ADDRESSES_SCRIPT, MULTI_HOST_CONFIG_NAME, MULTI_HOST_CONFIG_VARIABLE, MULTI_HOST_EXAMPLE, NAMED_SCOPE_OPTIONS,
    NESTED_DISPATCH_SCRIPTS, NFS_EXPORTS_FILE, NFS_FILESYSTEM_TYPES, NFS_ITEMS, NFS_LIVE_PROBE_LIMIT_SECONDS, NFS_MOUNT_PROBE_TEXT,
    NFS_MOUNT_RECOVERY, NFS_PASSED, NFS_PEER_PROBE_NAME, NFS_PROBE_NAME, NFS_PROBE_READ_TRIES, NFS_README, NFS_RECHECK_PAUSE_SECONDS, NFS_ROOTS_KEY,
    NFS_ROOTS_MISSING_TEXT, NFS_SLOW_GRACE_SECONDS, NFS_SSH_OPTIONS, NFS_SSH_SECONDS, NOT_SHARDED_ARGUMENTS, OPTIONAL_KEYS, OPTIONAL_KEY_PREFIX,
    OUTCOME_TYPES, OWN_SERVER_SELFTESTS, PEAK_OUTPUT_VARIABLE, PEER_ADMITTED_VARIABLE, PEER_BUILD_KEPT_MARK, PEER_BUILD_PEAK_MARK,
    PEER_COMMAND_END_LINE, PEER_COMMAND_HOST, PEER_MEMORY_PEAK_MARK, PEER_RUNNER, PEER_RUN_RETRIES, PINNED_HISTORY, PIN_SOURCE, PIN_STATE,
    PIN_UNREADABLE_HISTORY, PLAN_FILE, PLAN_KEY_NEUTRAL_VARIABLES, PRESENCE_ALIVE, PRESENCE_GONE, PRESENCE_UNPROBED, PRESENCE_UNPROBED_TEXT,
    PRESENCE_UNVERIFIED, PRESENCE_WORDS, PROCESS_MEMORY_PEAK_METRIC, PRUNE_SERVER_ALERT_KIND, PRUNE_SERVER_CONTROL, PRUNE_SERVER_UNAVAILABLE_OUTCOME,
    REASON_ELEMENTS, REASON_LOG_NOISE, REASON_MARK, RECORD_CAP_HIT_OUTCOMES, RECORD_COMPLETED_OUTCOMES, RECORD_HISTORY, REFUSED_PREFIXES,
    REPOSITORY_ROOT, REQUIRED_MODULES, RESOURCE_ARGUMENTS, RESULT_MARK, RETIRED_VARIABLES, RETRY_LIMITS, RUNNER_SCRIPT, RUNNING_UNIT_STATES,
    RUN_COMPLETED_OUTCOMES, RUN_CONDITION_EXIT, RUN_HARD_LIMIT_OUTCOME, RUN_HARD_LIMIT_PHASE, SCENES_ROOT_NAME, SCENE_VARIABLE,
    SCOPE_INSIDE_SCRIPT_TEXT, SCOPE_KIND_PREFIX, SCOPE_RANGE_ARGUMENTS, SCRIPT_DIRECTORY, SECRET_PATTERN, SELFTEST_MARK_TEXT, SELFTEST_MARK_VARIABLE,
    SELFTEST_ONLY_VARIABLES, SERVER_SCRIPT, SERVICE_ANOMALY_KINDS, SESSIONS_DIRECTORY_OVERRIDE, SESSION_REGISTRY_LOADED,
    SESSION_REGISTRY_SCRIPT, SESSION_VARIABLE, SHARDED_ENTRIES, SHARDS_ROOT_NAME, SHARD_HOST_VARIABLE, SHARD_MAXIMUM_VARIABLE,
    SHARD_MISSING_VARIABLE, SHARD_OUTPUTS_VARIABLE, SHARD_OUTPUT_VARIABLE, SHARD_PHASE_VARIABLE, SHARD_PLAN_BINARIES_NAME, SHARD_PLAN_KEYS,
    SHARD_PLAN_VARIABLE, SHARD_ROLES, SHARD_VARIABLE, SHARED_ROOT_VARIABLES, SHORT_TASK_RECORD_REASON, SHORT_TASK_UNRECORDED_REASON,
    SLICE_ROOM_FLOOR_VARIABLE, SOCKET_NAME, SOCKET_PATH_MAXIMUM_BYTES, SOURCE_REPOSITORY_VARIABLE, SQUEEZE_RETRIES, SSH_CONTROL_DIRECTORY_NAME,
    SSH_CONTROL_HASH_CHARACTERS, SSH_CONTROL_PERSIST_SECONDS, SSH_CONTROL_TEMPORARY_SUFFIX_CHARACTERS, SSH_CONTROL_VARIABLE, SSH_SHIM_DIRECTORY_NAME,
    STALL_AT_CAP_FRACTION, START_COMMAND, SUBMIT_MARK, SUBMIT_REFUSAL_KINDS, SUBMIT_WARNING_KINDS, SUSPECT_EXIT, SUSPECT_MARK, TAKEN_OVER_PREFIXES,
    TAKEN_OVER_VARIABLES, TARGET_DIRECTORY_VARIABLES, TEMPORARY_SERVICE_BINARY_VARIABLE, TEMPORARY_SERVICE_FLAG, TEMPORARY_SERVICE_KEY_MARK,
    TEMPORARY_SERVICE_OPTIONS, TEMPORARY_SERVICE_SCRIPT, TEST_STARTING_GATE_CELLS, THREADS_FILE_NAME, THREAD_ADMITTED_VARIABLE, THREAD_WRAPPER,
    UNIT_NAME, UNIT_TEMPLATE, UNNAMED_RUN_CARGO_QUERIES, UNNAMED_RUN_EXEMPTION_TEXT, UNNAMED_RUN_ITEM_ENTRIES, UNNAMED_RUN_PROBLEM_HEAD,
    UNNAMED_RUN_PROVE_RED_ENTRY, UNNAMED_RUN_PROVE_RED_QUERIES, UNNAMED_RUN_SELFTEST_VARIABLE, UNNAMED_RUN_STAGE_QUERIES, UNRECORDED_HISTORY,
    UNRECORDED_MEMORY_DEFAULT_BYTES, UNSCOPED_NEXT_STEP, UNSCOPED_RUN_ENTRIES, USAGE_EXIT, VERDICT_LIBRARY_PROGRESS_LOG_MARK, VOLATILE_FILESYSTEMS,
    WHOLE_PACKAGE_CHECK_CELLS, WHOLE_PACKAGE_CRASH_GATE, WHOLE_PACKAGE_MARKS, WHOLE_PACKAGE_MERGE_ENTRY, WHOLE_PACKAGE_NEXT_STEP,
    WHOLE_PACKAGE_SECTION_MINIMUM_CHARACTERS, WHOLE_SCOPE_KIND, WHOLE_SCOPE_TEXT, WORK_ROOT_VARIABLE, WRAPPER_OWN_ARGUMENTS, YOUNG_BOOKING_VARIABLE,
)
from task_server_lib_configuration import (
    CARGO_TEST_CLASS_FLAGS, CARGO_TEST_CLASS_VALUE_OPTIONS, Configuration, NfsProbeBudget, NfsProbeTimedOut, SELECTION_ATTACHED_VALUE_OPTIONS,
    SELECTION_SCRIPT_PREFIXES, SELECTION_VALUE_OPTIONS, Unavailable, broken, budgeted_item, build_tree_root, cap_hit_record_bytes,
    cargo_test_class_words, close_ssh_masters, command_word_index, configuration_key_missing_text, configured_by_host, configured_value,
    data_root_unavailable, default_values, enable_ssh_multiplexing, ensure_private_directory, expand_value, exported_roots, fast, filesystem_type,
    gpu_block_number_order, gpu_line_fields, gpu_service_hosts_read, gpu_service_problems_merged, gpu_service_report, host_addresses, in_selftest, inside, inside_repository, io_pressure_limit,
    io_pressure_limit_problem, key_words, last_line, load_module, main_worktree_root, missing_modules, multi_host_addresses_library,
    multi_host_configuration_path, nfs_mount_verdict, nfs_probe, nfs_probe_directory, nfs_share_report, own_server_selftest_split,
    parse_optional_keys, peer_shell, peer_shell_rechecked, peer_shell_timed, peer_ssh_address, read_configuration,
    refuse_selftest_variables_or_continue, report_unavailable_and_exit, repository_roots, repository_script_word, require_configured_data_root,
    require_modules, require_multi_host_keys, require_nfs_share, selection_class_words, selftest_override, selftest_variable_refusals, shards_root,
    shared_data_root, socket_path_problem, ssh_control_path_problem, ssh_multiplex_options, ssh_multiplexed_environment, ssh_shim_text,
    state_placement_problem, temporary_service_builds_server, temporary_service_split, temporary_service_wrapped, unwrapped_temporary_service,
    work_root, work_root_library,
)
from task_server_lib_commands import (
    BARE_TARGET_LITERAL, BYPASS_HITS_SHOWN, BYPASS_SCRIPT_MAXIMUM_BYTES, BYPASS_SKIPPED_LINE, BYPASS_WORD_PATH, MANIFEST_NONE, MANIFEST_PACKAGE,
    MANIFEST_UNREADABLE, MANIFEST_VIRTUAL, PYTHON_SELFTEST_DEFINITION, REPOSITORY_TARGET_NAME, ROOTED_TARGET_LITERAL, SHELL_SELFTEST_DEFINITION,
    canonical_json, card_declaration_problem, cargo_manifest_scope, cargo_option_values, covered_seconds, create_signing_key, declaration_matches,
    declared_compiles, declared_header_words, declared_shards, enclosing_user_unit, environment_refusal_lines, first_argument_matches,
    first_non_option, given_target_directories, nested_dispatch, over_run_hard_limit, process_environment, process_parent, read_shard_plan,
    read_signing_key, repository_target_references, resumable_command, retired_assignment, row_signature, script_target_literals, session_of_peer,
    shard_child_key, sharded_entry, split_environment, target_directory_refusal,
)
from task_server_lib_readings import (
    ATTEMPT_PRESENCE_READINGS_SOURCE, ATTEMPT_PROGRESS_READINGS_SOURCE, AnomalyRow, COMMAND_CLASS_INTERPRETERS, LOG_REASON_BYTES,
    LONG_RUN_WHY_MINIMUM_CHARACTERS, LONG_RUN_WHY_PLACEHOLDER, LOOP_LEAK_READING_MISMATCH, PROCESS_LIST_COLUMNS, RETIRED_OUTCOME_KINDS, ROLE_NAMES,
    RUNAWAY_PROTECTED_COMMANDS, STOP_PROCESS_POLL_SECONDS, ServerUnreachable, TASK_SLICE_MARK, UnixSocketConnection, anomaly_advice, anomaly_rows,
    anomaly_table, attempt_presence_probe_reply, attempt_presence_probe_text, attempt_presence_readings, attempt_presence_verdict,
    attempt_progress_probe_reply, attempt_progress_probe_text, attempt_progress_readings, attention_line, call_server, child_prefix,
    classify_attempt, command_classes, confirm_command, confirm_needed_fields, confirm_needed_line, crash_scope_text, entry_is, entry_word,
    external_kill_pid, final_exit_code, grown_cap, log_last_line, log_lines, log_tail_text, long_run_confirm_command, long_run_confirmation_problem,
    loop_leak_check_text, maximum_next_step, mebibytes_text, memory_cgroup_kill_line, named_scope, named_scope_entry, named_scope_text,
    named_scope_words, outcome_attempts, parsed_process_list, peer_command_exit, peer_kept_build_label, peer_marked_bytes, pinned_caps_text,
    process_gone_here, process_parents, reason_fields, reason_line, reply_attempts, reply_reason_fields, reply_reason_line, reply_result_line,
    require_server, runaway_processes, runaway_unowned_stoppable, session_inbox_socket, session_registry_library, shell_inner_commands, size_text,
    stop_process_here, stop_process_script, suspect_line, trailing_cap_hits, unreachable_error,
)
from task_server_lib_scopes import (
    KEY_STATS_COPY_MARK, KEY_STATS_DUMP_COLUMNS, KEY_STATS_RECENT, KEY_STATS_SINCE_DATE, RetryDecision, checked_cells, commit_gate_entry,
    copy_proof_marker, crash_gate_whole_package_cells, current_peak_metric, declared_scope_kind, gate_stage_of_entry, inside_directory, iso_epoch,
    item_option_given, item_option_values, key_stats, key_stats_after, key_stats_dump_rows, key_stats_flips, key_stats_lines, key_stats_mark,
    key_stats_since, peak_memory_note, prove_red_copies, query_only_entry, retry_decision, scope_declaration_segments, test_starting_gate_cells,
    unnamed_cargo_test_problem, unnamed_run_problem, unscoped_run_entry, whole_package_cargo_test, whole_package_confirmation_problem,
    whole_package_script,
)

# 自证成功句里报的并行度（task_server_selftest.py 的 run_all 填 workers）。写在这一份入口里、不放进拆出去的常量那一份：门禁 selftest-parallelism 只读 selftest-registry.tsv
# 登记的这一份文件找「并行 N 路」这一类字样，拆出去的模块它不读
SELFTEST_PARALLELISM_TEXT = "并行 {workers} 路"


def main(arguments):
    if arguments == ["--selftest"]:
        import task_server_selftest
        return task_server_selftest.run_all(arguments)
    if len(arguments) == 3 and arguments[0] == "--selftest-cell":
        import task_server_selftest
        return task_server_selftest.run_cell(arguments[1], arguments[2])
    print("  ✗ task_server_lib.py 用法：--selftest（单独跑别的参数没有意义：这一份是服务、客户端与 control 共用的库）", file=sys.stderr)
    print("  → 怎么办：提交任务用 task-server/task submit -- <命令…>；装与查用 python3 task-server/control", file=sys.stderr)
    return USAGE_EXIT


if __name__ == "__main__":
    sys.path.insert(0, SCRIPT_DIRECTORY)
    sys.exit(main(sys.argv[1:]))
