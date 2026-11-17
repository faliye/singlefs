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
  TASK_SERVER_BUILD_DIRECTORY     ${XDG_DATA_HOME:-~/.local/share}/singlefs/task-server-builds：每次尝试自己的编译目录 <尝试标签>/target 放在它下面（下面「编译目录」）；不许落在仓里
  值里认的写法只有开头的 ~/ 与 ${HOME}、${XDG_RUNTIME_DIR}、${XDG_STATE_HOME}、${XDG_CONFIG_HOME}、${XDG_DATA_HOME} 五个变量（没设的 XDG 照上面的缺省），展开之后要是绝对路径。
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
    编译目录在第二台本地盘上：multi-host-run.sh 在多机配置 BUILD_ROOT_BY_HOST 里第二台那一项下的 builds/<标签>/ 这一趟新建、跑完删（服务不设，task clean 不管它）。
  没有只派一台的名单（用户：「我再重申一次 【只能在本机跑】 是不能接受的。必须尽快解决 让错误暴露错来」）：点到哪个目录、哪个入口的命令都派得到两台，哪台由余量定；
    派到第二台跑红的照红，由脚本那一侧修，服务不替它挡（弄坏开关 host-lists-restored 下照改前放回 task_server.py 的 RESTORED_HOST_LISTS）。派哪台只看第二台此刻用不用得了（连得上、挂着共享根、
    盘余量够，下面两段）与余量。改主工作区的工具（hook-swap.py、apply-writer-patch.py、stage-mine.py、scripts/kb/ 下的）
    与带 --write 的派得到第二台（改的就是同一棵树）；它们撞上限这类本会重排的结局照旧不重排（task_server.py 的 modifies_workspace）。
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

编译目录（用户 2026-10-23 定案「自己用完自己删」；records/2026-10-30-编译绕过内存总上限.md）：
  入口声明（lifecycle 审核 F1：门禁入口里面才起 cargo，命令那一层认不出）：命令位置上起的脚本文件头前 DECLARED_HEADER_LINES 行里写「# task-server-compiles: yes」
    （DECLARED_COMPILES_MARK、COMPILES_DECLARED_WORD；declared_compiles）的，服务当它会编（task_server.py 的 compiles、compiles_declared），照下面给编译目录、撞上限重排接着用上一次那一份；
    单独 --selftest、带 --list-items、--help、-h 的不算；写了别的词的打回 run-condition-unmet，说明写那一句。分片的计划、各片与合并照父任务（create_child）。
    命令拿到的是 CARGO_TARGET_DIR（本机 <TASK_SERVER_BUILD_DIRECTORY>/<尝试标签>/target，第二台 multi-host-run.sh 给的 builds/<标签>/target）：入口照它编、不删。
    弄坏开关 compiles-declaration-ignored 下不读这一行。
  命令里命令位置上有会编的 cargo（.claude/hooks/lib_workspace_cargo_lock.py 的 cargo_invocations 与 BUILDING_SUBCOMMANDS 认，包装剥掉之后照认）、
  调用方没给编译目录的，本机这一次尝试设 CARGO_TARGET_DIR=<TASK_SERVER_BUILD_DIRECTORY>/<尝试标签>/target；任务记下结局就删它各次尝试的编译目录
  （用户：「服务build之后必须删除历史」；task_server.py 的 remove_finished_build_directories，服务起来时 sweep_finished_build_directories 把已有结局还留着的再扫一遍），
  wait 末行不带 build=。提交时设了 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1 的留着现场：wait 末行带 build=<编译目录>，由提交它的会话用 task-server/task clean <号> 删。
  弄坏开关 finished-build-kept 下照改前一律留着等 task clean（起来时也不扫）。
  取消的不留产物（用户：「取消的任务要自己删掉产物 不要残留」「这个加入流程中 让task server去做」）：task cancel 回读到单元停了，服务在同一次取消里删这条任务
  （分片的连同子任务）每一次尝试的编译目录与现场目录（下面「现场」），提交时设了 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1 的也删，不再等 task clean；派到第二台的，这一趟第二台本地的 builds/<标签>/ 与 runs/ 下进程号、退出码、输出三个文件由 multi-host-run.sh 的 ⑥ 删，
  服务照那一次尝试日志里它打的「⑥ 删了第二台这一趟的本地目录 …（回读不在了）」认（task_server.py 的 PEER_TRIP_REMOVED_MARK），上一次带 --keep-build-directory 留着的经 --remove-build-directory 删。
  删前删后的大小（第二台本地盘上的那一份服务这一侧量不到，照写量不到）写进每条任务最后一次尝试的日志与取消的回复；回读不在了才算删了，没删成的逐份点名、task cancel 退 5
  （task_server.py 的 remove_cancelled_artifacts）。弄坏开关 cancelled-artifacts-kept 下取消之后不删、不报。
  重试复用（records/2026-10-31-task-server巡检.md 批 60：撞了上限重排时从零重编）：重排时服务在本机建的那一份不删，下一次尝试起在同一台就接着用那一份
  （CARGO_TARGET_DIR 指同一个目录；推的：cargo 照指纹复用已编完的 crate，省多少没量过；尝试记 build_directory_reused），换到第二台的在那一次起的时候删
  （删法 scripts/hosts/run_build_directory.py 的 remove_run_build_directory：删之前核没有进程开着里面的文件，有就不删、写告警 build-directory-left；task_server.py 的 kept_build_directory）。
  第二台同一段（批 65）：服务给编译目录的尝试派到第二台时带 --keep-build-directory，multi-host-run.sh 跑完留着这一趟的 builds/<标签>/、打一行 PEER_BUILD_KEPT_MARK
  （它文件头「留着的编译目录」），服务照这一行记成没删（build_directory_kept）；重排的那一次还在第二台就带 --reuse-build-directory <上一次的标签> 接着用那一份
  （multi-host-run.sh 在第二台上核它在不在，不在退 255），换到本机的在那一次起的时候、有了结局的在记下结局时经 multi-host-run.sh --remove-build-directory <标签> 删
  （Machine.build_directory_exists、remove_build_directory：本机就地删，第二台经它；设了 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1 的也删第二台那一份，task clean 不连第二台）。
  弄坏开关 peer-retry-build-directory-fresh 下第二台照改前不留、不接着用。
  弄坏开关 retry-build-directory-fresh 下照改前重排时当场删、下一次新建；retry-build-directory-kept 下换了台也不删上一次那一份。两台同一段（task_server.py 的 Machine.build_directory）：派到第二台的服务不设，由 multi-host-run.sh 在第二台本地盘上 BUILD_ROOT_BY_HOST 里它那一项下的 builds/<尝试标签>/ 这一趟新建、跑完删（multi-host-run.sh 文件头 ⑤），
  服务把那个路径记进尝试的编译目录（status 看得见）、记成「已删」（wait 末行不带 build=，task clean 不管它）。
  调用方给了编译目录：环境里的 CARGO_TARGET_DIR / CARGO_BUILD_TARGET_DIR、命令里的 CARGO_TARGET_DIR=… 赋值词与 --target-dir；落在仓里（这一棵树或主工作树里）的提交时拒（退 3），
  落在仓外的照它、不删（归调用方）。只认写在词上的，bash -c 字符串里的另由钩子那一层判。

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
    shard_child_key），编与不编各记各的；经 RUN_WITH_MEMORY_CAP_KEY 交给内存包装；
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
  起自证的假分片入口或命令里有记号词 fake-peer-run 时照真的那样在这一刻的工作目录里跑（第二台在同一棵共享的树里），编译目录给这一格临时目录旁边的假第二台本地目录下的、跑完删），
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
  指到格里的假共享根）；TASK_SERVER_SELFTEST_PAUSE_POINT=<start-checked|claimed|cancel-unlocked|attempt-finishing|finished-build-removal|launch-recheck|stop-attempts-read|confirm-counted>:<目录>
  目录里 arm-<点> 在时服务在那一点建 reached-<点>、等 release-<点>
  （start-checked 是判过 queued、记尝试之前，claimed 是记了尝试、起单元之前，cancel-unlocked 是取消记了 cancel_requested、放了锁、停单元之前，
  attempt-finishing 是对账收尾一次尝试的开头，finished-build-removal 是记下结局时拿着 build_removal_lock、删一份编译目录之前，
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
  TASK_SERVER_SELFTEST_ANOMALY_REPUSH_SECONDS 门槛不随时长涨的提醒同一道门槛推过之后再推的门槛（秒，换掉 task_server.py 的 ANOMALY_REPUSH_SECONDS）。
  TASK_SERVER_SELFTEST_CRASH_POINT 的点另有 attempt-claimed（起尝试的事务里、插了尝试、改任务之前）与分片推进的
  parent-inserted、child-finished-<plan|shard|merge>、shard-inserted、merge-outputs-written、merge-inserted。

弄坏开关 TASK_SERVER_BREAK=<项>（逗号连几个；只给 --selftest 证红，认不出的项自证退 2）。读开关只在这一份的 broken。打开后自证红在哪一格
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
  TASK_SERVER_BREAK=no-detach  尝试不起成脱离服务的单元，起成服务自己的子进程，服务停下时跟着停（task_server.py 的 start_unit）
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
  TASK_SERVER_BREAK=record-ignored-for-cap  定上限不看服务自己的资源记录，只照没记录的打底（task_server.py 的 prediction_of）
      「✗ task_server_lib.py 自证：attention-queued-too-long：」、「✗ task_server_lib.py 自证：attention-reported-once：」、「✗ task_server_lib.py 自证：run-hard-limit-shard-stopped：」
      「✗ task_server_lib.py 自证：error-types：」、「✗ task_server_lib.py 自证：memory-cap-growth：」、「✗ task_server_lib.py 自证：memory-cap-at-maximum：」、「✗ task_server_lib.py 自证：resource-record：」、
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
      「✗ task_server_lib.py 自证：build-directory-outside-repository：」、「✗ task_server_lib.py 自证：retry-build-directory-reused：」、
      「✗ task_server_lib.py 自证：clean-failure-explained：」、「✗ task_server_lib.py 自证：finished-build-removed：」（后三格要尝试记着编译目录，推的）
      「✗ task_server_lib.py 自证：temporary-service-wrapper-compiles：」、「✗ task_server_lib.py 自证：build-disk-predicted：」
      「✗ task_server_lib.py 自证：clean-waits-for-finished-removal：」（新格，推的，没复跑）
  TASK_SERVER_BREAK=retry-build-directory-kept  重排换了台，上一次在本机留着的那一份编译目录不删（task_server.py 的 start_attempt 换台那一支）
      「✗ task_server_lib.py 自证：retry-build-directory-reused：」
  TASK_SERVER_BREAK=clean-removes-nothing  task clean 不删编译目录（task_server.py 的 clean）
      「✗ task_server_lib.py 自证：build-directory-outside-repository：」、「✗ task_server_lib.py 自证：clean-failure-explained：」
      （这两格红的同时，task clean 自己打的那一句也是失败行）「✗ 任务 … 没删干净：删了 0 份、没删成 1 份（逐份在上面）」
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
  TASK_SERVER_BREAK=completed-peak-ignored  退 0 的尝试量到的峰值不进定额度（task_server.py 的 recorded_prediction）
      「✗ task_server_lib.py 自证：attention-queued-too-long：」、「✗ task_server_lib.py 自证：attention-reported-once：」、「✗ task_server_lib.py 自证：run-hard-limit-shard-stopped：」
      「✗ task_server_lib.py 自证：error-types：」、「✗ task_server_lib.py 自证：memory-cap-growth：」、「✗ task_server_lib.py 自证：memory-cap-at-maximum：」、「✗ task_server_lib.py 自证：resource-record：」、
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
  TASK_SERVER_BREAK=peer-overflow-blocked  本机在前的任务本机此刻放不下（还不到永远放不下）时不派第二台，留在队里等本机（task_server.py 的 place）
      「✗ task_server_lib.py 自证：attention-cleanup-pending：」、「✗ task_server_lib.py 自证：temporary-service-wrapper-compiles：」、「✗ task_server_lib.py 自证：temporary-service-own-server-compiles：」、「✗ task_server_lib.py 自证：every-command-reaches-peer：」、
      「✗ task_server_lib.py 自证：peer-runs-in-shared-tree：」、「✗ task_server_lib.py 自证：peer-stall-judged-by-peer-cpu：」、「✗ task_server_lib.py 自证：peer-when-local-full：」
      「✗ task_server_lib.py 自证：memory-cap-passed-to-command：」、「✗ task_server_lib.py 自证：source-repository-passed-to-command：」、「✗ task_server_lib.py 自证：shared-roots-passed-to-command：」、「✗ task_server_lib.py 自证：peer-admission-marked：」、
      「✗ task_server_lib.py 自证：peer-disk-floor：」、「✗ task_server_lib.py 自证：pinned-cap-by-host：」、「✗ task_server_lib.py 自证：gate-stages-sent-to-peer：」、「✗ task_server_lib.py 自证：attempt-label-passed-to-command：」、
      「✗ task_server_lib.py 自证：verdict-library-family-refused：」、「✗ task_server_lib.py 自证：peer-unavailable-alerted：」、「✗ task_server_lib.py 自证：build-disk-predicted：」、
      「✗ task_server_lib.py 自证：peer-memory-peak-recorded：」、「✗ task_server_lib.py 自证：outside-shared-path-refused：」、「✗ task_server_lib.py 自证：shared-roots-outside-refused：」、「✗ task_server_lib.py 自证：merge-on-peer-reads-shard-outputs：」、
      「✗ task_server_lib.py 自证：retry-build-directory-reused：」
      「✗ task_server_lib.py 自证：recheck-judges-disk：」、「✗ task_server_lib.py 自证：recheck-judges-nfs：」（新格，推的，没复跑）
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
      「✗ task_server_lib.py 自证：parent-cancel-waits-for-children：」（新格，推的，没复跑）
  TASK_SERVER_BREAK=unsharded-arguments-sharded  第一个参数是 --selftest、--list-items、--help、-h（或参数里有 --selftest）的也照分片（first_argument_matches）
      「✗ task_server_lib.py 自证：unsharded-arguments：」
  TASK_SERVER_BREAK=shard-retry-restarts-all  一片重排时把已经成了的兄弟片也重排、再跑一遍（task_server.py 的 finish_attempt）
      「✗ task_server_lib.py 自证：shard-retry-isolated：」
  TASK_SERVER_BREAK=merge-before-all-green  有一片成了就起合并，不等别的片（task_server.py 的 drive_parent）
      「✗ task_server_lib.py 自证：shard-to-peer-by-plan：」、「✗ task_server_lib.py 自证：shard-siblings-share-a-host：」、「✗ task_server_lib.py 自证：shard-cap-fitted-to-room：」、「✗ task_server_lib.py 自证：shard-retry-isolated：」、
      「✗ task_server_lib.py 自证：shard-host-is-address：」、「✗ task_server_lib.py 自证：sharded-progress-recovered：」、「✗ task_server_lib.py 自证：merge-on-peer-reads-shard-outputs：」
      「✗ task_server_lib.py 自证：shard-cancel-alone-merges：」
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
      「✗ task_server_lib.py 自证：aged-task-judged-first：」、「✗ task_server_lib.py 自证：three-duties-two-sessions：」
      「✗ task_server_lib.py 自证：squeezed-at-total-is-cap-hit：」、「✗ task_server_lib.py 自证：pinned-peer-unavailable：」、「✗ task_server_lib.py 自证：unstarted-slice-uses-configured-total：」、「✗ task_server_lib.py 自证：margin-measured-per-host：」、
      「✗ task_server_lib.py 自证：booking-within-host-limit-every-history：」、「✗ task_server_lib.py 自证：cap-hit-record-over-every-host-starts-clamped：」（推的，没复跑）
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
      「✗ task_server_lib.py 自证：starved-head-judged-by-the-formula：」、「✗ task_server_lib.py 自证：shard-cap-fitted-to-room：」、「✗ task_server_lib.py 自证：pinned-cap-from-configuration：」
      「✗ task_server_lib.py 自证：pinned-cap-by-host：」、「✗ task_server_lib.py 自证：admission-booked-at-cap：」、「✗ task_server_lib.py 自证：whole-starts-beside-running-fragment：」
      「✗ task_server_lib.py 自证：environment-prefix-stripped-from-key：」、「✗ task_server_lib.py 自证：retry-cap-not-floored：」
      「✗ task_server_lib.py 自证：recorded-cap-not-floored：」
      「✗ task_server_lib.py 自证：margin-measured-per-host：」、「✗ task_server_lib.py 自证：peer-scope-missing-booked-at-cap：」
      「✗ task_server_lib.py 自证：two-hosts-booked-within-capacity：」
      「✗ task_server_lib.py 自证：booking-within-host-limit-every-history：」、「✗ task_server_lib.py 自证：cap-hit-record-over-every-host-starts-clamped：」（推的，没复跑）
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
  TASK_SERVER_BREAK=feed-only-counted-as-writer  hook-swap.py --feed-only 也当成改主工作区、撞上限不重排（task_server.py 的 modifies_workspace）
      「✗ task_server_lib.py 自证：workspace-writer-not-retried：」
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
      「✗ task_server_lib.py 自证：workspace-writer-not-retried：」
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
      「✗ task_server_lib.py 自证：outside-shared-path-refused：」、「✗ task_server_lib.py 自证：shared-roots-outside-refused：」、「✗ task_server_lib.py 自证：finished-build-removed：」、「✗ task_server_lib.py 自证：nfs-probe-concurrent-runs：」、
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
      「✗ task_server_lib.py 自证：health-database-unwritable-notified：」、「✗ task_server_lib.py 自证：health-unit-missing-notified：」、「✗ task_server_lib.py 自证：health-nfs-notified：」、「✗ task_server_lib.py 自证：health-peer-unavailable-notified：」
      「✗ task_server_lib.py 自证：cancelled-before-peer-trip-built：」（推的，没复跑：同上，起服务提交任务）
      「✗ task_server_lib.py 自证：peer-health-slow-rechecked：」（新格，推的，没复跑）
      「✗ task_server_lib.py 自证：parent-cancel-waits-for-children：」、「✗ task_server_lib.py 自证：clean-waits-for-finished-removal：」、「✗ task_server_lib.py 自证：recheck-judges-disk：」、「✗ task_server_lib.py 自证：recheck-judges-nfs：」、「✗ task_server_lib.py 自证：stall-stop-spares-next-attempt：」（新格，推的，没复跑）、「✗ task_server_lib.py 自证：loop-leak-check-on-bounce：」（新格，推的，没复跑）、「✗ task_server_lib.py 自证：repeated-cap-hits-reminded-once：」（新格，推的，没复跑）
  TASK_SERVER_BREAK=nfs-probe-one-way  两向探针只做本机→第二台那一向，第二台写的本机读不回也照过（nfs_share_report）
      「✗ task_server_lib.py 自证：nfs-share-verified：」
  TASK_SERVER_BREAK=nfs-probe-at-root  根是 git 工作树也把探针写在根下（nfs_probe_directory），红在 nfs-share-verified 一格
      「✗ task_server_lib.py 自证：nfs-share-verified：」
  TASK_SERVER_BREAK=finished-build-kept  任务记下结局之后照改前留着各次尝试的编译目录、等提交方 task clean（服务起来时也不扫；task_server.py 的 remove_finished_build_directories）
      「✗ task_server_lib.py 自证：finished-build-removed：」
      「✗ task_server_lib.py 自证：clean-waits-for-finished-removal：」（新格，推的，没复跑）
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
  TASK_SERVER_BREAK=peer-tree-copied  派第二台照改前拷树：加 --send <工作目录> <树里的落点>、命令换到落点里起（task_server.py 的 attempt_command；假第二台照真的那样认不出 --send、退 2）
      「✗ task_server_lib.py 自证：attention-queued-too-long：」、「✗ task_server_lib.py 自证：attention-cleanup-pending：」、「✗ task_server_lib.py 自证：temporary-service-wrapper-compiles：」、「✗ task_server_lib.py 自证：temporary-service-own-server-compiles：」、
      「✗ task_server_lib.py 自证：every-command-reaches-peer：」、「✗ task_server_lib.py 自证：error-types：」、「✗ task_server_lib.py 自证：memory-cap-at-maximum：」、「✗ task_server_lib.py 自证：retry-goes-where-it-fits：」、
      「✗ task_server_lib.py 自证：peer-runs-in-shared-tree：」、「✗ task_server_lib.py 自证：peer-stall-judged-by-peer-cpu：」、「✗ task_server_lib.py 自证：three-duties-two-sessions：」
      「✗ task_server_lib.py 自证：hundred-five-minute-tasks-balanced：」、「✗ task_server_lib.py 自证：peer-run-failed-retried：」、「✗ task_server_lib.py 自证：peer-starts-throttled：」、「✗ task_server_lib.py 自证：peer-when-local-full：」、
      「✗ task_server_lib.py 自证：shard-to-peer-by-plan：」、「✗ task_server_lib.py 自证：shard-siblings-share-a-host：」、「✗ task_server_lib.py 自证：cancel-sharded-running：」、「✗ task_server_lib.py 自证：memory-cap-passed-to-command：」、
      「✗ task_server_lib.py 自证：source-repository-passed-to-command：」、「✗ task_server_lib.py 自证：shared-roots-passed-to-command：」、「✗ task_server_lib.py 自证：peer-admission-marked：」
      「✗ task_server_lib.py 自证：peer-disk-floor：」、「✗ task_server_lib.py 自证：shard-host-is-address：」、「✗ task_server_lib.py 自证：gate-stages-sent-to-peer：」、「✗ task_server_lib.py 自证：attempt-label-passed-to-command：」、
      「✗ task_server_lib.py 自证：down-host-waits-not-never：」、「✗ task_server_lib.py 自证：verdict-library-family-refused：」、「✗ task_server_lib.py 自证：margin-measured-per-host：」、「✗ task_server_lib.py 自证：margin-unmeasured-not-placed：」、
      「✗ task_server_lib.py 自证：peer-booked-like-local：」、「✗ task_server_lib.py 自证：peer-scope-missing-booked-at-cap：」、「✗ task_server_lib.py 自证：two-hosts-booked-within-capacity：」、「✗ task_server_lib.py 自证：peer-unavailable-alerted：」、
      「✗ task_server_lib.py 自证：sharded-cancel-retried：」、「✗ task_server_lib.py 自证：build-disk-predicted：」、「✗ task_server_lib.py 自证：peer-command-exit-split：」、「✗ task_server_lib.py 自证：peer-memory-peak-recorded：」、
      「✗ task_server_lib.py 自证：temporary-service-wrapper-unwrapped：」、「✗ task_server_lib.py 自证：outside-shared-path-refused：」、「✗ task_server_lib.py 自证：shared-roots-outside-refused：」、「✗ task_server_lib.py 自证：merge-on-peer-reads-shard-outputs：」、
      「✗ task_server_lib.py 自证：failed-start-not-counted：」、「✗ task_server_lib.py 自证：retry-build-directory-reused：」
      「✗ task_server_lib.py 自证：recheck-judges-disk：」、「✗ task_server_lib.py 自证：recheck-judges-nfs：」（新格，推的，没复跑）
  TASK_SERVER_BREAK=workspace-writers-local-only  改主工作区的工具照改前只在本机（task_server.py 的 peer_eligible）
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
      「✗ task_server_lib.py 自证：recheck-judges-disk：」（新格，推的，没复跑）
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
  TASK_SERVER_BREAK=retry-build-directory-fresh  撞上限重排时当场删这一次的编译目录，下一次新建、从零编（改前的样子；task_server.py 的 settle_attempt、start_attempt）
      「✗ task_server_lib.py 自证：retry-build-directory-reused：」、「✗ task_server_lib.py 自证：peer-retry-build-directory-reused：」（后一格推的，没复跑）
  TASK_SERVER_BREAK=shard-key-from-joined-command  分片子任务的键照改前用空格接父命令的词再按空格切，条目名里带空格的后半截漏进键里（task_server.py 的 create_child）
      「✗ task_server_lib.py 自证：shard-child-key-folded-by-words：」
  TASK_SERVER_BREAK=workspace-write-interrupted-unreported  改主工作区的命令没正常结束也不报（改前的样子；task_server.py 的 attention_of_record）
      「✗ task_server_lib.py 自证：workspace-write-interrupted-reported：」、「✗ task_server_lib.py 自证：run-hard-limit-writer-stopped：」
  TASK_SERVER_BREAK=apply-outcome-ignored  不读 apply-writer-patch.py 的结局文件，打全了的也照可能写了一半报（task_server.py 的 workspace_write_judged）
      「✗ task_server_lib.py 自证：workspace-write-complete-not-reported：」
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
      也当成改主工作区、撞上限不重排（改前的样子；task_server.py 的 modifies_workspace）
      「✗ task_server_lib.py 自证：writer-dry-run-retried：」
  TASK_SERVER_BREAK=peer-retry-build-directory-fresh  派到第二台的尝试不叫 multi-host-run.sh 留着编译目录、重排到第二台的那一次也不接着用（改前的样子：第二台每一趟新建、跑完删；
      task_server.py 的 start_attempt、PeerMachine.command）
      「✗ task_server_lib.py 自证：peer-retry-build-directory-reused：」
  TASK_SERVER_BREAK=parallelism-falls-back-to-cores  各格并行的路数取不出登记的每台默认重活线程数就按本机核数（改前的样子；task_server_selftest.py 的 parallelism）：
      登记文件不在、写坏、SINGLEFS_THREAD_CAP 不是正整数时本该判红
      「✗ task_server_lib.py 自证：parallelism-registration-refused：」
  TASK_SERVER_BREAK=whole-workspace-cargo-accepted  提交时不判不点名包的全仓编译，不带 -p 的 cargo build、带 --workspace 的 cargo clippy 照收（改前的样子；任务 8105；
      task_server.py 的 placement_refusal 与 whole_workspace_cargo_refusal）
      「✗ task_server_lib.py 自证：whole-workspace-cargo-refused：」
  TASK_SERVER_BREAK=repository-target-bypass-unwarned  提交时不判命令与入口脚本取仓根 target/ 下的二进制，提交回复、日志头与 status 都不打严厉警告
      （改前的样子：任务 7937–7944 取了仓根 target/ 下合并前的旧二进制；task_server.py 的 repository_target_bypass）
      「✗ task_server_lib.py 自证：repository-target-bypass-warned：」
  TASK_SERVER_BREAK=cancelled-artifacts-kept  取消之后不删各次尝试的编译目录与现场目录、不核第二台那一趟的本地目录，取消的回复与日志里不报删前删后（改前的样子：
      取消之后编译目录与现场等提交它的会话另下删除指令；task_server.py 的 remove_cancelled_artifacts）
      「✗ task_server_lib.py 自证：cancelled-artifacts-removed：」
  TASK_SERVER_BREAK=cancel-not-started-receipt-unread  取消之后核第二台那一趟的本地目录不读清理回执：回执还是 not-started、目录还没建的，日志里没有 ⑥ 那一行也算没删成、
      cancel 退 5（改前的样子；task_server.py 的 cancelled_attempt_artifacts）
      「✗ task_server_lib.py 自证：cancelled-before-peer-trip-built：」
      （这一格红的同时，task cancel 自己打的那一句也是失败行）「✗ 任务 … 停了，但取消之后有 1 份产物没删掉（逐份在上面）」
  TASK_SERVER_BREAK=inbox-not-sent  服务一条消息都不往会话的收件 socket 发（task_server.py 的 send_to_session），发不出去的告警也不写
      「✗ task_server_lib.py 自证：inbox-outcome-succeeded：」、「✗ task_server_lib.py 自证：inbox-outcome-failed：」、「✗ task_server_lib.py 自证：inbox-queued-too-long-once：」、「✗ task_server_lib.py 自证：inbox-unreachable-task-runs：」、「✗ task_server_lib.py 自证：run-hard-limit-stopped：」、
      「✗ task_server_lib.py 自证：health-database-unwritable-notified：」、「✗ task_server_lib.py 自证：health-unit-missing-notified：」、「✗ task_server_lib.py 自证：health-nfs-notified：」、「✗ task_server_lib.py 自证：health-peer-unavailable-notified：」
      「✗ task_server_lib.py 自证：peer-health-slow-rechecked：」、「✗ task_server_lib.py 自证：long-run-reminded-not-stopped：」（新格，推的，没复跑）、「✗ task_server_lib.py 自证：long-run-confirm-silences：」（新格，推的，没复跑）、「✗ task_server_lib.py 自证：loop-leak-check-on-bounce：」（新格，推的，没复跑）、「✗ task_server_lib.py 自证：repeated-cap-hits-reminded-once：」（新格，推的，没复跑）
  TASK_SERVER_BREAK=inbox-auth-omitted  发消息不发认证行、只发 user 那一行（task_server.py 的 post_to_inbox）
      「✗ task_server_lib.py 自证：inbox-outcome-succeeded：」、「✗ task_server_lib.py 自证：inbox-outcome-failed：」、「✗ task_server_lib.py 自证：inbox-queued-too-long-once：」、「✗ task_server_lib.py 自证：run-hard-limit-stopped：」、
      「✗ task_server_lib.py 自证：health-database-unwritable-notified：」、「✗ task_server_lib.py 自证：health-unit-missing-notified：」、「✗ task_server_lib.py 自证：health-nfs-notified：」、「✗ task_server_lib.py 自证：health-peer-unavailable-notified：」
      「✗ task_server_lib.py 自证：peer-health-slow-rechecked：」、「✗ task_server_lib.py 自证：long-run-confirm-silences：」（新格，推的，没复跑）、「✗ task_server_lib.py 自证：loop-leak-check-on-bounce：」（新格，推的，没复跑）、「✗ task_server_lib.py 自证：repeated-cap-hits-reminded-once：」（新格，推的，没复跑）
      （long-run-reminded-not-stopped 不红，量的：它只按第一行取消息、核正文里的字，不看认证行、也不核令牌；别把这一格点进来）
  TASK_SERVER_BREAK=inbox-reported-every-round  发消息那一路不看报到的提醒门槛与段，每一轮照发（task_server.py 的 push_attention、push_suspects）
      「✗ task_server_lib.py 自证：inbox-queued-too-long-once：」、「✗ task_server_lib.py 自证：repeated-cap-hits-reminded-once：」（新格，推的，没复跑）、「✗ task_server_lib.py 自证：long-run-reminded-not-stopped：」（新格，推的，没复跑）、「✗ task_server_lib.py 自证：long-run-confirm-silences：」（新格，推的，没复跑）
  TASK_SERVER_BREAK=pushed-outcome-not-collected  结局发出去了也不记已取（task_server.py 的 push_outcomes）
      「✗ task_server_lib.py 自证：inbox-outcome-succeeded：」、「✗ task_server_lib.py 自证：inbox-outcomes-resent-after-error：」
  TASK_SERVER_BREAK=inbox-pending-dropped-on-error  发结局那一轮处理到某一条抛错时不把它与后面还没处理的号放回待发，下一轮不补发（改前的样子；task_server.py 的 push_outcomes）
      「✗ task_server_lib.py 自证：inbox-outcomes-resent-after-error：」
  TASK_SERVER_BREAK=health-unwatched  服务不盯自己的健康、不发「服务健康」（task_server.py 的 watch_health）
      「✗ task_server_lib.py 自证：health-database-unwritable-notified：」、「✗ task_server_lib.py 自证：health-unit-missing-notified：」、「✗ task_server_lib.py 自证：health-nfs-notified：」、「✗ task_server_lib.py 自证：health-peer-unavailable-notified：」
      「✗ task_server_lib.py 自证：peer-health-slow-rechecked：」（新格，推的，没复跑）
  TASK_SERVER_BREAK=health-alerted-every-round  同一样服务健康对同一个会话每一轮照发，不记发过的（task_server.py 的 watch_health）
      「✗ task_server_lib.py 自证：health-database-unwritable-notified：」、「✗ task_server_lib.py 自证：health-unit-missing-notified：」、「✗ task_server_lib.py 自证：health-nfs-notified：」、「✗ task_server_lib.py 自证：health-peer-unavailable-notified：」
      （peer-health-slow-rechecked 不红，量的：它等到头一条服务健康消息就取、不再等下一轮，核的是慢一次先复查，不是每一轮重发；别把这一格点进来）
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
  TASK_SERVER_BREAK=confirm-skipped  交任务不等确认就进队（task_server.py 的 submit）
      「✗ task_server_lib.py 自证：submit-awaits-confirmation：」、「✗ task_server_lib.py 自证：unconfirmed-cancelled：」、「✗ task_server_lib.py 自证：operation-log：」、「✗ task_server_lib.py 自证：confirmation-scope-text：」（后两格推的，没复跑：不等确认就跑完、操作日志里没有 confirm 那一条）
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
  TASK_SERVER_BREAK=loop-leak-reading-mismatch-silent  每一次都没顶到上限又撞上限的不写「峰值读数与撞上限对不上」那一句（loop_leak_check_text）
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
  TASK_SERVER_BREAK=attempt-row-overwritten  删完编译目录记「已删」拿手里先读的那份尝试整行写回，Store.update 不守库里已 ended 的尝试
      （task_server.py 的 mark_build_directory_removed、Store.update）：对账刚记了 ended 的那一次被写回 running、结局与峰值抹掉
      「✗ task_server_lib.py 自证：attempt-row-not-overwritten：」
  TASK_SERVER_BREAK=parent-finalized-before-children  取消分片的整件停了子任务的单元就给父任务记结局、删产物、回 stopped，不等子任务在库里记成 finished
      （task_server.py 的 cancel_children）
      「✗ task_server_lib.py 自证：parent-cancel-waits-for-children：」
  TASK_SERVER_BREAK=clean-races-finished-removal  记下结局时先叫醒 wait 再删编译目录，task clean 不拿 build_removal_lock（task_server.py 的 finalize、clean）
      「✗ task_server_lib.py 自证：clean-waits-for-finished-removal：」
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
  TASK_SERVER_BREAK=clean-peer-build-left  task clean 不删第二台留着的编译目录（task_server.py 的 clean）
      「✗ task_server_lib.py 自证：clean-removes-peer-kept-build：」
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
  TASK_SERVER_BREAK=cpu-running-cores-over-registered  CPU 闸里实测到的在跑尝试峰值段那一支不限登记的线程数（task_server.py 的 measured_running_cores）
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
  TASK_SERVER_BREAK=booking-drift-notified-memory-only  记账漂移每一件推给每个会话推到了哪道提醒门槛只记在内存里，服务重启后同一道门槛再推一回（同上）
      「✗ task_server_lib.py 自证：reminders-not-resent-after-restart：」
  TASK_SERVER_BREAK=long-run-split-silent  出结局时用时超过 7200 秒的不推 long-run-split（task_server.py 的 attention_of_record）
      「✗ task_server_lib.py 自证：long-run-split-reminded-at-outcome：」
  TASK_SERVER_BREAK=long-run-split-cap-hits-counted  long-run-split 的用时把撞了内存上限的那几次也累加（task_server.py 的 settled_run_seconds）
      「✗ task_server_lib.py 自证：long-run-split-reminded-at-outcome：」
  TASK_SERVER_BREAK=long-run-split-exempt-ignored  long-run-split 不看 .claude/long-run-split-exempt（task_server.py 的 long_run_split_exempt）
      「✗ task_server_lib.py 自证：long-run-split-reminded-at-outcome：」
  TASK_SERVER_BREAK=workspace-write-blocked-unreported  换钩子退 7（被上一趟没写完成的挡住）不推 workspace-write-blocked（task_server.py 的 attention_of_record）
      「✗ task_server_lib.py 自证：workspace-write-blocked-reminded：」
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
  TASK_SERVER_BREAK=selection-key-whole-command  本仓门禁与入口脚本的选法（--check、--item、--only 与它们的值）照原样进键：点名不同格、不同项的同一道各是一个键、各从没记录的起（selection_class_words）
      「✗ task_server_lib.py 自证：selection-key-by-script：」
  TASK_SERVER_BREAK=progress-ignored-for-stall  判卡住不看进程内存在涨（task_server.py 的 judge_stall_stop、activity_text）
      「✗ task_server_lib.py 自证：memory-growth-not-stalled：」
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
  TASK_SERVER_BREAK=finish-in-scheduler  收尾（第二台清理回执、留下的进程回读、删编译目录）在排队线程里当场做，第二台卡住时排队线程跟着卡住（task_server.py 的 finish_off_loop）
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
PRUNE_SERVER_CONTROL = os.path.join(REPOSITORY_ROOT, "prune-server", "control")   # 命令退 75 时起一次它的 check（文件头「错误类型」prune-server-unavailable）
THREAD_WRAPPER = os.path.join(HOSTS_DIRECTORY, "capped.sh")

MULTI_HOST_CONFIG_NAME = "multi-host.env"
MULTI_HOST_CONFIG_VARIABLE = "SINGLEFS_MULTI_HOST_CONFIG"
OPTIONAL_KEY_PREFIX = "TASK_SERVER_"
OPTIONAL_KEYS = ("TASK_SERVER_STATE_DIRECTORY", "TASK_SERVER_RUNTIME_DIRECTORY", "TASK_SERVER_SIGNING_KEY_FILE", "TASK_SERVER_BUILD_DIRECTORY")
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
# 十五种提醒（task_server.py 的 attention_items 判，文件头「需要处置」）：排队过久、日志长期不涨而服务没停它、卡在 IO 上而服务没停它、
# 清理没收尾、分片父任务推不动、改主工作区的命令没正常结束、预算贴着整台、跑得久而核没用起来、按给的上限开并行、
# 连着三次撞上限还在重排、跑满两小时没确认、本机读不到在跑尝试的单元状态、
# 出结局时用时超过 7200 秒要拆、换钩子被上一趟没写完成的挡住、线程总数远超登记数
ATTENTION_KINDS = ("queued-too-long", "quiet-log-busy", "io-wait", "cleanup-pending", "unit-unreadable", "shard-advance-failed",
                   "workspace-write-interrupted", "oversized-budget", "long-run-idle-cores", "fills-any-cap", "repeated-cap-hits", "long-run-unconfirmed",
                   "long-run-split", "workspace-write-blocked", "threads-over-registered")
BOUNCING_ATTENTION_KINDS = ()   # 提醒里动作是「打回」的（此刻没有：别的提醒都是「提醒照跑」；文件头「异常表」）
# 异常表（ANOMALY_TABLE_TEXT）：那几栏只写在那一个常量里，anomaly_table 照它读，代码里不另抄一份；下面几组是代码里分别处置的种类，自证 anomaly-table-matches-kinds 拿它们与表比
ANOMALY_ACTIONS = ("提交时拒", "打回", "提醒照跑", "服务自己修")
ANOMALY_TABLE_HEAD = "异常表（"
ANOMALY_ROW = re.compile(r"^  ([a-z][a-z-]*)｜([^｜]+)｜([^｜]+)｜([^｜]+)｜([^｜]+)$")
SUBMIT_REFUSAL_KINDS = ("environment-refused", "nested-wrapper", "target-directory-in-repository", "outside-shared-path", "shared-roots-outside",
                        "verdict-library-unserved", "temporary-service-heavy", "whole-workspace-cargo", "inbox-missing", "manual-from-session", "sessionless-unmarked", "full-gate-not-commit-entry", "confirm-counts-mismatch",
                        "whole-package-unconfirmed", "unnamed-run", "card-declaration")
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
  long-run-split、workspace-write-blocked 只推一回、不再推；
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
  full-gate-not-commit-entry｜提交时拒｜带了 --full-gate，而命令不是提交时的六道门禁、scripts/crash/crash-amplification.sh 或 scripts/gate/gate-staged.sh，或带了 --check、--item｜全量标识只给不指定范围的提交时门禁：去掉 --full-gate，指定范围的照常交；跑过两小时的照提醒 task-server/task confirm <号> --why 确认｜不进库，不再推
  manual-from-session｜提交时拒｜带了 --manual，而进程树上读得到提交它的会话｜去掉 --manual 再交：会话交的任务结局由服务推给会话｜不进库，不再推
  sessionless-unmarked｜提交时拒｜进程树上读不到会话，也没带 --manual（客户端环境里的收件地址不算数）｜人手交的写成 task-server/task submit --manual -- <命令…>，结局用 task-server/task wait <号> 取；不要填假的收件地址｜不进库，不再推
  whole-package-unconfirmed｜提交时拒｜命令是整包测试（cargo test 不点名 --test、--bin、带测试名的 --lib，或带 --lib --tests、--workspace；crash-point-replay.sh 的 --check 点了 feature-gated-tests、product-gpu-real-card-tests 或不带 --crash-cases-only 的 crash-case-replay；不给补丁目录或带 --all-packages 的 merge-and-verify.sh），或是不带选条参数的门禁与入口（UNSCOPED_RUN_ENTRIES，不指定范围），而没带 --whole-package-confirmed，或带了而确认文件的形式不合（不是共享根下读得到的绝对路径、缺 ①②③ 哪一段、哪一段不到 12 个字）；带 --full-gate 的提交时门禁（COMMIT_GATE_ENTRIES）不走这一条，其中不指定范围而没带 --full-gate 的（含崩溃放量）另拒，整包确认标识顶替不了｜改成只点名改到的测试目标与测试函数（不指定范围的加上 --check、--item 或实验号）；不指定范围的只在提交时跑，提交之外要跑先问用户；确实要整包或不指定范围的照 .claude/agent-common.md「日常开发的测试不跑整个包」确认三遍（①改到的文件与函数、②引到它们的测试目标与函数及 grep 命中、③为什么只点名②不够）写进共享根下的一份文件，带 --whole-package-confirmed <确认文件> 重交；带了照样受两小时提醒与四小时硬上限｜不进库，不再推
  unnamed-run｜提交时拒｜启动测试与验证没点名（unnamed_run_problem）：cargo test 没在 `--` 之后带 --exact 与测试全名；scripts/gate/merge-and-verify.sh、scripts/crash/crash-point-replay.sh、scripts/crash/crash-amplification.sh 没带 --item；scripts/mutation/prove-red.sh 没给变异名；门禁阶段 --item、--check 都没带，或 --check 点了起测试的格（TEST_STARTING_GATE_CELLS）而没带 --item，或门禁 crash-point-replay 带了 --item 而 --check 没把起整个验证包的格排掉（crash_gate_whole_package_cells）；带 --full-gate 或 --whole-package-confirmed 的不走这一条｜照拒绝说明给的样子改成点名改到的、红了的那几项（cargo test 写 -- <测试全名> --exact，入口与门禁写 --item <项>，门禁 crash-point-replay 点崩溃枚举用例再加 --check crash-case-replay --crash-cases-only，证红写变异名）再交；确实要不指定范围或整包跑的，提交时带 --full-gate，提交之外先问用户、照 .claude/agent-common.md「日常开发的测试不跑整个包」确认三遍、带 --whole-package-confirmed <确认文件> 重交｜不进库，不再推
  card-declaration｜提交时拒｜命令位置上起的脚本（bash -c 里命令位置上的每一条同样认）文件头写了领卡、要卡的声明（# task-server-card-lease:、# task-server-gpu-cards:、# task-server-gpu-arguments:；card_declaration_problem）｜卡归 GPU 批处理验证服务，任务不领卡：删掉那一行，脚本里要卡的那一段改成往判定库写待判行、交 GPU 服务判（.claude/rules/verification.md「卡只归 GPU 服务，任务不拿卡」），再照原命令重交｜不进库，不再推
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
  oversized-budget｜提醒照跑｜排队那一刻在某一台上的记账不小于那台的整片界（可用上限的 WHOLE_FRACTION，85%）而不超过可用上限｜核实这一趟的范围（点的项、测试目标）、分片与同时起的个数；确要这么多就照跑、task-server/task confirm {task} --why '<为什么>' 记照跑（记过不再再推），不要的 task-server/task cancel {task}、收小或分片再交｜推过满 30 分钟还在排、在跑的再推，之后每翻一倍再推
  long-run-unconfirmed｜提醒照跑｜不是带 --full-gate 的提交时门禁，各次尝试在跑的时长累计满 RUN_LIMIT_SECONDS（两小时；撞了内存上限重来的计时归零）而没确认，服务不停、不改任务｜核两件事：范围对不对（点名范围：--item、--check 少几格、点名测试目标；给补丁目录；或照服务分片；整轮的留到提交时），这条是不是确实要跑这么久；不对的 task-server/task cancel {task}、改了范围用 task-server/task submit 重交；对的照整行确认：task-server/task confirm {task} --why '<为什么要跑这么久、范围为什么是这些>'（--why 去掉空白至少 12 个字，只许提交它的会话）；确认放不过四小时硬上限，累计满四小时服务照停（over-four-hours）｜推过满 30 分钟还没确认的再推，之后每翻一倍再推；确认了不再推；没有会话的不推，写进任务日志与 status 那一行
  long-run-split｜提醒照跑｜任务（分片的按片）出结局，服务记的用时超过 LONG_RUN_SPLIT_SECONDS（7200 秒；撞了内存上限、照大一档重来的，从最后一次撞上限之后那一次起算），又不在 .claude/long-run-split-exempt 里｜照 .claude/rules/long-run-split.md「怎么做」办：这一件让它跑完、结局照常用，不为拆而取消；出结局的同一轮里把这一项拆成几项、每一项能单独点名、各自预计不超过 7200 秒，拆不开的优化那段算法、量到不超过；缩域、少点几项、填 --why、确认长跑都不算拆；那一项的代码归别的会话管的，照 .claude/main-agent.md「一轮怎么开、怎么收」第 15 条发给那个会话；调度记录写一行｜只推一回、不再推
  workspace-write-blocked｜提醒照跑｜换钩子的命令（scripts/gate/hook-swap.py，不带 --feed-only）退 7：上一趟换上没写完成（被杀或回滚不成，几份文件停在半截），拒绝再换、一个字节没写；报给被挡的这个会话｜先照这条任务日志里拒绝说明点名的状态核：python3 scripts/gate/hook-swap.py --read-outcome 读结局文件，起上一趟的会话在的就请它补写完成（--seal-outcome）或照反向规格退回；等不到它的主 agent 手工退回到换前再重交（不要使用原来的任务号，用 task-server/task submit 创建新的任务号）；上一趟是哪条任务看依据里点名的号｜只推一回、不再推
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
UNNAMED_RUN_STAGE_QUERIES = frozenset({"--list", "--list-items", "--selftest", "--help", "-h", "--tool-versions"})   # 门禁阶段只打格名、项名、自证、帮助、工具版本
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
                    INBOX_LOG_MARK + " ", LONG_RUN_LOG_MARK + " ", HEARTBEAT_LOG_MARK + " ")   # 取日志最后一行时跳过的进度行（task_server.py 判日志涨不涨同样不算它们）
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

ITEM_LINE_PREFIX = b"ITEM_"

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
    "shard-advance-failed": "分片推不动", "workspace-write-interrupted": "改主工作区的命令没正常结束", "oversized-budget": "记账贴着整台",
    "long-run-idle-cores": "跑得久而核没用起来", "fills-any-cap": "按给的上限开并行", "repeated-cap-hits": "连着撞上限", "long-run-unconfirmed": "跑满两小时没确认",
    "long-run-split": "跑过两小时要拆", "workspace-write-blocked": "换钩子被上一趟没写完成的挡住", "threads-over-registered": "线程远超登记数",
    "suspected-stall": "疑似卡住", "unclaimed-outcome": "结局还没人取",
}
LONG_RUN_SPLIT_SECONDS = 7200   # .claude/rules/long-run-split.md「判据」：一项一次运行的用时超过这么多秒，跑完就要拆
EXIT_TYPES = {0: "succeeded", 75: "prune-server-unavailable", 77: "not-run", 78: "run-condition-unmet", 250: "memory-cap-hit",
              251: "wrapper-refused", 253: "timed-out", 254: "squeezed-out"}
PRUNE_SERVER_UNAVAILABLE_OUTCOME = "prune-server-unavailable"   # 75：scripts/prune-server/prune_server_client.py 的 EXIT_UNAVAILABLE
PRUNE_SERVER_ALERT_KIND = "prune-server-unavailable"            # 命令退 75 而 prune-server/control check 不通时写的告警
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
    "TASK_SERVER_JOURNALCTL", CPU_OVERSUBSCRIPTION_VARIABLE, YOUNG_BOOKING_VARIABLE, UNNAMED_RUN_SELFTEST_VARIABLE)
SELFTEST_PARALLELISM_TEXT = "并行 {workers} 路"   # 自证成功句里报的并行度（task_server_selftest.py 的 run_all 填 workers）；门禁 selftest-parallelism 只读登记的这一份文件，字样放在这里它才认
KNOWN_BREAK_ITEMS = (
    "run-hard-limit-skipped", "run-hard-limit-full-gate-judged", "run-hard-limit-confirmed-spared", "run-hard-limit-writers-spared", "run-hard-limit-children-spared", "run-hard-limit-parent-summed", "run-hard-limit-outcome-unmarked", "run-hard-limit-writer-unreminded", "run-hard-limit-exclusive", "run-hard-limit-loose", "scope-inside-script-unnamed", "crash-scope-unreported",
    "fallback-run-when-unreachable", "exit-code-dropped", "wait-gives-up-on-disconnect", "environment-unfiltered", "queue-variables-carried",
    "retired-variables-carried", "retired-variable-assignments-accepted", "nested-dispatch-allowed", "session-from-client", "cancel-session-unchecked", "cancel-no-readback", "signature-unchecked", "tamper-unalerted",
    "operations-unlogged", "unlisted-exit-swallowed", "peer-exit-unsplit", "lost-not-recorded",
    "cap-growth-skipped", "cap-unclamped", "host-switch-skipped", "retry-overwrites-attempts", "squeeze-retry-skipped",     "server-pending-ignored", "launch-when-unplaceable", "queue-reversed", "nofile-limit-dropped", "waiting-reason-unreported", "backfill-disabled",     "no-detach", "start-failure-ignored", "start-twice-after-restart",
    "resources-unrecorded", "record-ignored-for-cap", "unclaimed-not-listed", "check-deps-skipped", "config-missing-ignored",
    "state-placement-unchecked", "dry-run-writes", "secrets-overwritten", "install-readback-skipped", "stop-exit-status-only",
    "target-directory-unset", "retry-build-directory-kept", "clean-removes-nothing", "clean-session-unchecked", "runner-wake-skipped", "wait-notify-skipped", "aged-task-not-prioritized", "starved-head-unreported", "blocked-head-counted-as-starved", "in-repository-target-accepted",
    "unmeasured-recorded-as-zero", "threads-unset", "maximum-line-vague", "sharding-unrecognized",
    "shard-retry-restarts-all", "merge-before-all-green", "shard-failure-unnamed", "balance-ignored", "retry-pinned-to-host", "stall-ignored", "stall-grace-skipped", "cpu-unread-counted-as-idle", "outer-scope-cpu-only", "inner-commands-unchecked", "queued-not-revalidated", "cpu-ignored-for-stall", "peer-run-retry-skipped", "peer-starts-unthrottled", "squeeze-at-total-not-cap-hit", "host-specific-variables-sent-to-peer", "running-progress-unreported", "wrapper-prefixes-allowed", "resource-arguments-allowed", "peer-overflow-blocked",
    "shard-not-sent-to-peer", "shard-cap-not-fitted", "shard-key-position-folded", "socket-path-unchecked", "unrecorded-default-grown", "unrecorded-booked-above-cap",
    "unfinished-peaks-counted", "cap-hit-not-counted", "cap-hit-low-peak-counted-at-cap",
    "completed-peak-ignored", "log-tail-unbounded", "log-all-ignored", "log-attempt-ignored", "log-missing-exit-zero", "queued-cancel-left-queued",
    "finished-cancel-overwrites", "sharded-cancel-children-left", "plan-failure-proceeds", "merge-failure-succeeds", "unsharded-arguments-sharded",
    "start-readback-skipped", "start-exit-status-ignored", "gpu-switch-taken-as-on", "gpu-service-skipped", "pinned-peer-unavailable-waits",
    "prediction-recomputed-every-pass", "pending-read-per-task", "pass-booking-not-carried", "launch-recheck-skipped",
        "stall-clock-reset-on-restart", "workspace-writers-retried", "judge-error-blocks-queue", "stalled-at-cap-not-retried", "booked-up-to-slice-total",
    "memory-cap-not-passed", "source-repository-not-passed", "suspect-unreported", "suspect-reported-every-wait", "pinned-cap-ignored", "pinned-cap-grown",
    "pinned-cap-literal", "pin-unreadable-predicted", "booked-at-peak", "health-judges-head",     "pinned-cap-at-slice-total", "margin-fallback-fixed", "margin-unmeasured-waits",
    "margin-local-for-both", "pinned-nonpositive-accepted", "ended-attempts-unsettled", "feed-only-counted-as-writer", "registry-service-column-ignored", "cpu-gate-ignored", "unstarted-slice-estimated", "down-host-judged-never",     "environment-prefix-kept", "nfs-unchecked", "nfs-probe-one-way", "nfs-probe-at-root", "nfs-probe-shared-name", "finished-build-kept", "quoted-word-paths-ignored", "expansion-tail-as-path",
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
    "retry-build-directory-fresh", "shard-key-from-joined-command",
    "workspace-write-interrupted-unreported", "apply-outcome-ignored",     "read-only-arguments-counted-as-writer", "peer-retry-build-directory-fresh",
    "parallelism-falls-back-to-cores",
    "at-cap-busy-cpu-kept-running", "at-cap-fraction-ignored", "peer-memory-line-unread", 
    "attention-quiet-busy-unreported", "io-wait-ignored", "attention-io-wait-unreported",         "health-head-judged-apart", "progress-unwatched", "progress-gates-ignored", 
    "whole-workspace-cargo-accepted", 
    "repository-target-bypass-unwarned", "cancelled-artifacts-kept",
    "inbox-not-sent", "inbox-auth-omitted", "inbox-reported-every-round", "pushed-outcome-not-collected",
    "health-unwatched", "health-alerted-every-round", "unclaimed-unwarned",
    "cancel-not-started-receipt-unread", "quiet-log-busy-any-exemption", "inbox-pending-dropped-on-error",
    "leftovers-error-uncaught", "inbox-attention-per-shard",
    "memory-peak-with-page-cache", "attention-reported-memory-only", "attention-queue-reasonless-sent",
    "booking-over-host-limit", "record-over-every-host-never-fits", "drop-history-cap-hits-ignored", "oversized-budget-silent", "long-run-idle-silent",
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
    "attempt-row-overwritten", "parent-finalized-before-children", "clean-races-finished-removal", "recheck-disk-unchecked",     "recheck-nfs-unchecked", "stall-stop-by-task", "peer-cleanup-recovered-concurrently",
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
    "unit-unreadable-health-unreported",         "cargo-test-key-whole-command", "selection-key-whole-command", "commit-gate-without-full-gate-accepted", "progress-ignored-for-stall", 
    "cpu-wanted-by-registered-threads", "cpu-unrecorded-by-one-core", "cpu-restart-unmeasured-by-record", "cpu-latest-runnable-ignored", "signal-exit-unnamed", "cpu-running-unmeasured-registered",
    "heartbeat-counted-as-log", "record-shard-count-unchecked",
        "fills-any-cap-counts-record-caps", "unit-stopped-still-booked", "cpu-load-io-unsplit", "peer-recheck-once-only",
    "check-nfs-probed-live-by-default", "check-probe-unbounded", "booking-audit-skipped", "booking-audit-young-counted", "plan-key-compile-unmarked", "merge-and-verify-counted-as-writer",
        "promote-session-unchecked", "promote-cpu-gate-applied", "promote-queue-order-ignored",     "progress-line-progress-unnamed", "open-file-write-mode-unchecked", "open-file-growth-ignored",
    "quiet-log-busy-progress-ignored", "ssh-unmultiplexed",
        "store-transaction-left-open", "disk-floor-host-judged-never",     "stall-cpu-read-before-log",
    "peer-readings-overwritten", "log-content-unlocked", "promote-unwoken", "promote-shards-refused", "promote-aged-blocks-queue", "promote-blocked-counts-as-earlier",
    "pause-queue-positions-filtered",     "peer-probe-thread-unjoined", "peer-cooldown-ignored-off-scheduler", "clean-peer-build-left",     "command-wrapper-values-unskipped", "cargo-exclude-value-unskipped", "runner-start-error-uncaught", "runner-final-sample-skipped", "log-noise-marks-incomplete",
    "runaway-targets-counting-children",
    "aged-blocked-lets-others-start", "jump-length-unchecked", "running-young-booked-by-usage", "machine-memory-line-ignored", "external-kill-unrecognized", "bookings-ledger-unreported",
    "threads-record-succeeded-only", "confirm-retries-fixed",     "cpu-running-cores-over-registered",
    "suspect-reported-memory-only", "health-notified-memory-only", "booking-drift-notified-memory-only",     "long-run-split-silent", "long-run-split-cap-hits-counted", "long-run-split-exempt-ignored", "workspace-write-blocked-unreported",
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
    "whole-package-feature-items-uncounted", "whole-package-any-item-exempts", "card-declarations-accepted", "card-variables-passed-to-attempt")


# ── 只在自证里认的变量 ──
def in_selftest(environment=None):
    environment = os.environ if environment is None else environment
    mark = environment.get(SELFTEST_MARK_VARIABLE, "")
    if not mark:
        return False
    try:
        with open(mark, encoding="utf-8") as handle:
            return handle.read().strip() == SELFTEST_MARK_TEXT
    except OSError:
        return False


def selftest_override(name, environment=None):
    """只在自证里认的变量：自证里交它的值，别处交空串（别处设了由 refuse_selftest_variables_or_continue 拒）。"""
    environment = os.environ if environment is None else environment
    return environment.get(name, "") if in_selftest(environment) else ""


def broken(item, environment=None):
    return item in selftest_override(BREAK_VARIABLE, environment).split(",")


def cap_hit_record_bytes(cap, peak, unresolved, environment=None):
    """撞了上限的那一次尝试进定额度的量（字节；文件头「资源记录」）：task_server.py 的 cap_hit_measurement 与 control drop-history --attempt-records-over-gib
    都只经这一处算，两处是同一个数。峰值到了上限的 CAP_HIT_REAL_PEAK_FRACTION 或量不到的取上限（至少要到这个上限）；之后这把键还没有一次跑完的（unresolved）
    取上限乘 CAP_GROWTH_FACTOR（撞了说明要的比这个上限多，同键下一条新任务从放大过的那一档起；任务 15161 撞 6144M，照上限本身记时重交的 15185 还是 6144M）；
    峰值远低于上限的取峰值（脚本开跑前预判装不下就退 250，任务 10731：上限 30.4G、峰值 1.0G，上限不代表它要这么多）。
    弄坏开关 cap-hit-low-peak-counted-at-cap 下峰值远低于上限的也取上限。"""
    measured = int(cap * CAP_GROWTH_FACTOR) if unresolved else int(cap)
    if (isinstance(peak, int) and not isinstance(peak, bool) and 0 < peak < cap * CAP_HIT_REAL_PEAK_FRACTION
            and not broken("cap-hit-low-peak-counted-at-cap", environment)):
        measured = peak
    return measured


def key_words(command, environment=None):
    """算命令键用的词（文件头「资源记录」的命令键）：起头的 env 与紧跟的「名=值」里名在 KEY_NEUTRAL_VARIABLES 的剥掉，别的赋值照留（留了的连 env 一起留）；
    一个都没剥掉的（不是 env 起头、env 后面先是选项）照原样。弄坏开关 environment-prefix-kept 下照原样。"""
    words = unwrapped_temporary_service(list(command), environment)
    if broken("environment-prefix-kept", environment) or not words or os.path.basename(words[0]) != "env":
        return words
    index, kept = 1, []
    while index < len(words) and ASSIGNMENT_WORD.match(words[index]):
        if words[index].split("=", 1)[0] not in KEY_NEUTRAL_VARIABLES:
            kept.append(words[index])
        index += 1
    if len(kept) == index - 1:
        return words
    return ([words[0]] + kept if kept else []) + words[index:]


# cargo test 的同类键（cargo_test_class_words）照留的几样：带值的选项 → 键里写成哪个名字；不带值的开关 → 键里写成哪个名字
CARGO_TEST_CLASS_VALUE_OPTIONS = {"-p": "-p", "--package": "-p", "--exclude": "--exclude", "--manifest-path": "--manifest-path",
                                  "--features": "--features", "-F": "--features", "--profile": "--profile"}
CARGO_TEST_CLASS_FLAGS = {"--workspace": "--workspace", "--all": "--workspace", "--all-features": "--all-features",
                          "--no-default-features": "--no-default-features", "--release": "--release", "-r": "--release"}


def cargo_test_class_words(words, environment=None):
    """命令键用的词里 cargo test 归同类（文件头「资源记录」的命令键：同包同特性同档就是同类）：命令词之前的前缀照留，cargo test 后面只留包（-p / --package、
    --workspace、--exclude、--manifest-path）、特性（--features / -F 的值按逗号与空格拆开、去重排序合成一个，--all-features、--no-default-features）与档（--release、--profile），
    次序固定；点名的测试目标（--test、--lib、--bin、--example、--bench 与它们的值）、位置参数里的测试名、`--` 之后的整段都不进键：点名不同目标的同类命令共用一份记录。
    别的命令照原样。弄坏开关 cargo-test-key-whole-command 下照原样（点名不同目标就各是一个键）。"""
    words = list(words)
    index = command_word_index(words)
    if (broken("cargo-test-key-whole-command", environment) or index + 1 >= len(words)
            or os.path.basename(words[index]) != "cargo" or words[index + 1] != "test"):
        return words
    arguments = words[index + 2:]
    if "--" in arguments:
        arguments = arguments[:arguments.index("--")]
    values, flags, position = {}, set(), 0
    while position < len(arguments):
        name, joined, value = arguments[position].partition("=")
        if name in CARGO_TEST_CLASS_VALUE_OPTIONS:
            if not joined:
                value = arguments[position + 1] if position + 1 < len(arguments) else ""
                position += 1
            values.setdefault(CARGO_TEST_CLASS_VALUE_OPTIONS[name], set()).update(
                piece for piece in (re.split(r"[,\s]+", value) if name in ("--features", "-F") else [value]) if piece)
        elif arguments[position] in CARGO_TEST_CLASS_FLAGS:
            flags.add(CARGO_TEST_CLASS_FLAGS[arguments[position]])
        position += 1
    kept = sorted(flags)
    for option in sorted(values):
        if option == "--features":
            kept += [option, ",".join(sorted(values[option]))]
        else:
            for value in sorted(values[option]):
                kept += [option, value]
    return words[:index + 2] + kept


# 本仓门禁与入口脚本归同类键（selection_class_words）时连值去掉的选法：点名哪几格、哪几项不同的同一道共用一份记录（用户定「类似这样的参数都应该加入」）
SELECTION_VALUE_OPTIONS = ("--check", "--item", "--only")
SELECTION_SCRIPT_PREFIXES = (".claude/gate.d/", "scripts/", ".claude/scripts/")


def selection_class_words(words, environment=None):
    """命令键用的词里本仓门禁与入口脚本归同类（文件头「资源记录」的命令键）：命令词是 SELECTION_SCRIPT_PREFIXES 下的脚本（相对路径，或绝对路径里含 /<前缀>）时，
    SELECTION_VALUE_OPTIONS 里的选项（--x <值> 与 --x=<值> 两种写法）连同值都去掉，在末尾按选项名排序各接一个「--x *」；一个都没有的照原样（整道一个键，与点名的分开）。
    别的命令照原样。弄坏开关 selection-key-whole-command 下照原样（点名不同格、不同项就各是一个键）。"""
    words = list(words)
    index = command_word_index(words)
    if broken("selection-key-whole-command", environment) or index >= len(words):
        return words
    script = words[index]
    if not any(script.startswith(prefix) or f"/{prefix}" in script for prefix in SELECTION_SCRIPT_PREFIXES):
        return words
    kept, seen, position = words[:index + 1], set(), index + 1
    while position < len(words):
        name, joined, _ = words[position].partition("=")
        if name in SELECTION_VALUE_OPTIONS:
            seen.add(name)
            position += 1 if joined else 2
            continue
        kept.append(words[position])
        position += 1
    if not seen:
        return words
    for name in sorted(seen):
        kept += [name, "*"]
    return kept


def temporary_service_wrapped(command, environment=None):
    """命令包着临时 prune-server 的那一层（unwrapped_temporary_service 剥得开）交 True：命令键前面另接 TEMPORARY_SERVICE_KEY_MARK（task_server.py 的 command_key）。
    弄坏开关 temporary-service-key-shared 下一律 False（包着的与不包的共键，第十二批甲的样子）。"""
    if broken("temporary-service-key-shared", environment):
        return False
    return unwrapped_temporary_service(list(command), environment) != list(command)


def unwrapped_temporary_service(command, environment=None):
    """剥开临时 prune-server 的包装（TEMPORARY_SERVICE_SCRIPT --with-temporary-service [--server-binary <路径>] -- <原命令>，前面的 python3 一起剥；
    包装前面的 env、nice 这类前缀照留）：交前缀加原命令；不是这一层包装的照原样交。算命令键（key_words）、认登记表行
    （task_server.py 的 verdict_library_refusal）、判包装里会编的（task_server.py 的 temporary_service_heavy_refusal）用它；包装本身照旧整条执行。
    弄坏开关 temporary-service-wrapper-kept 下照原样交。"""
    words = list(command)
    if broken("temporary-service-wrapper-kept", environment):
        return words
    split = temporary_service_split(words)
    if split is None:
        return words
    start, inner, _ = split
    return words[:start] + words[inner:]


def temporary_service_split(words):
    """认临时 prune-server 的包装（TEMPORARY_SERVICE_SCRIPT --with-temporary-service [--server-binary <路径>] -- <原命令>）：不是这一层交 None；
    是交 (包装起头的位置：前面的 python3 算在包装里, 原命令起头的位置, 给了 --server-binary 没有)。unwrapped_temporary_service 与 temporary_service_builds_server 共用。"""
    index = command_word_index(words)
    if index >= len(words) - 1 or os.path.basename(words[index]) != TEMPORARY_SERVICE_SCRIPT or words[index + 1] != TEMPORARY_SERVICE_FLAG:
        return None
    start = index - 1 if index > 0 and os.path.basename(words[index - 1]) in ("python3", "python") else index
    position = index + 2
    binary_given = False
    while position < len(words) and words[position] in TEMPORARY_SERVICE_OPTIONS:
        binary_given = True
        position += 2
    if position >= len(words) - 1 or words[position] != "--":
        return None
    return start, position + 1, binary_given


def own_server_selftest_split(words):
    """自带临时服务的那两份脚本（OWN_SERVER_SELFTESTS）的自证：脚本词后面紧跟它认的自证模式词的交 (起头的位置：前面的 python3 算在里面, 给了 --server-binary 没有)，
    不是这种起法交 None。temporary_service_builds_server 用它；包装那一层另由 temporary_service_split 认。"""
    index = command_word_index(words)
    if index >= len(words) - 1:
        return None
    modes = OWN_SERVER_SELFTESTS.get(os.path.basename(words[index]))
    if modes is None or words[index + 1] not in modes:
        return None
    start = index - 1 if index > 0 and os.path.basename(words[index - 1]) in ("python3", "python") else index
    return start, "--server-binary" in words[index + 2:]


def temporary_service_builds_server(command, task_environment=None, environment=None):
    """命令会让 prune-server 现编一份、又没给二进制（--server-binary，命令前的赋值与任务环境里的 TEMPORARY_SERVICE_BINARY_VARIABLE）交 True：这一层包装
    （temporary_service_split）与自带临时服务的两份脚本的自证（own_server_selftest_split）不带二进制起时，都在服务给的编译目录里现编一份 prune-server
    （prune_server_client_selftest.py 的 resolve_binary），外层这一条按会编的算（task_server.py 的 compiles），两台都由服务给编译目录；不是这两种起法、或给了二进制的交 False。
    弄坏开关 temporary-service-build-unrecognized 下一律 False，temporary-service-own-server-unrecognized 下自带临时服务的那两份不认。"""
    if broken("temporary-service-build-unrecognized", environment):
        return False
    words = list(command)
    split = temporary_service_split(words)
    if split is not None:
        start, binary_given = split[0], split[2]
    else:
        own = None if broken("temporary-service-own-server-unrecognized", environment) else own_server_selftest_split(words)
        if own is None:
            return False
        start, binary_given = own
    if binary_given:
        return False
    given_before = any(word.startswith(TEMPORARY_SERVICE_BINARY_VARIABLE + "=") for word in words[:start])
    return not given_before and not (task_environment or {}).get(TEMPORARY_SERVICE_BINARY_VARIABLE)


def selftest_variable_refusals(environment=None):
    environment = os.environ if environment is None else environment
    if in_selftest(environment):
        return []
    named = [name for name in SELFTEST_ONLY_VARIABLES if environment.get(name)]
    if not named:
        return []
    return ([f"  ✗ {name}={environment[name]} 只在自证（python3 task-server/task_server_lib.py --selftest）里认，平常跑设了它就拒，什么都没做" for name in named]
            + [f"  → 怎么办：unset {' '.join(named)} 再跑；要看弄坏开关弄红哪几格，设着 {BREAK_VARIABLE} 跑 --selftest（开关表在 task_server_lib.py 文件头）"])


def refuse_selftest_variables_or_continue(environment=None):
    refusals = selftest_variable_refusals(environment)
    if refusals:
        for line in refusals:
            print(line, file=sys.stderr)
        sys.exit(USAGE_EXIT)


def fast(environment=None):
    return selftest_override("TASK_SERVER_SELFTEST_FAST", environment) == "1"


# ── 运行条件不满足（退 78）──
class Unavailable(Exception):
    """运行条件不满足：str 是卡在哪，next_step 是出路。"""

    def __init__(self, reason, next_step):
        super().__init__(reason)
        self.next_step = next_step


def report_unavailable_and_exit(error, who):
    print(f"  ✗ {who}：{error}", file=sys.stderr)
    print(f"  → 怎么办：{error.next_step}", file=sys.stderr)
    sys.exit(RUN_CONDITION_EXIT)


def missing_modules(environment=None):
    """必须依赖里此刻导入不了的 python 模块（只查找、不导入）。"""
    forced = selftest_override("TASK_SERVER_SELFTEST_MISSING_MODULES", environment).split(",")
    return [name for name in REQUIRED_MODULES if name in forced or importlib.util.find_spec(name) is None]


def require_modules(environment=None):
    missing = missing_modules(environment)
    if missing:
        raise Unavailable(f"必须依赖的 python 模块不在：{'、'.join(missing)}（task-server 没有它们起不来，不退回直接执行命令）",
                          f"{INSTALL_PACKAGES}；装好之后 {INSTALL_COMMAND}，再 python3 task-server/control check 回读到退 0")


# ── 路径（文件头「路径」）──
@dataclass
class Configuration:
    path: str
    state_directory: str
    runtime_directory: str
    signing_key_file: str
    build_directory: str

    @property
    def database(self):
        return os.path.join(self.state_directory, DATABASE_NAME)

    @property
    def logs_directory(self):
        return os.path.join(self.state_directory, "logs")

    @property
    def attempts_directory(self):
        return os.path.join(self.state_directory, "attempts")

    @property
    def shards_directory(self):
        """分片的计划与各片产物（文件头「分片」）：共享工作根下的 SHARDS_ROOT_NAME，两台照同一个绝对路径读写。"""
        return shards_root()

    @property
    def scenes_directory(self):
        """判红那一趟的现场（文件头「现场」）：共享工作根下的 SCENES_ROOT_NAME，两台照同一个绝对路径读写。"""
        return os.path.join(work_root(), SCENES_ROOT_NAME)

    @property
    def alerts_log(self):
        return os.path.join(self.state_directory, "alerts.jsonl")

    @property
    def operations_log(self):
        return os.path.join(self.state_directory, "operations.jsonl")

    @property
    def socket(self):
        return os.path.join(self.runtime_directory, SOCKET_NAME)


def shards_root():
    """分片的计划与各片产物所在的目录（文件头「分片」）：工作根下的 SHARDS_ROOT_NAME，不归多机配置管。"""
    return os.path.join(work_root(), SHARDS_ROOT_NAME)


def work_root_library():
    return load_module("lib_work_root", os.path.join(HOOKS_DIRECTORY, "lib_work_root.py"))


def data_root_unavailable(error, configuration_path):
    """取不到共享数据根（lib_work_root.py 的 DataRootUnconfigured）→ Unavailable（退 78），出路点名往多机配置加 SHARED_DATA_ROOT 那一行（文件头「共享数据根」）。"""
    return Unavailable(f"取不到共享数据根：{error}",
                       f"往多机配置 {configuration_path}（主工作树根的 multi-host.env）加一行 {DATA_ROOT_KEY}=<数据根的绝对路径>，要落在同一份配置 {NFS_ROOTS_KEY} 的某一段下"
                       f"（{NFS_README}）；改好再起")


def work_root():
    """工作根的真路径（.claude/hooks/lib_work_root.py 的 work_root()，SINGLEFS_WORK_ROOT 可换，没设是共享数据根下的 work；在 NFS 共享根下，两台同一个绝对路径）。
    取不到（共享数据根取不到）抛 Unavailable，不补缺省。"""
    library = work_root_library()
    try:
        return os.path.realpath(library.work_root())
    except library.DataRootUnconfigured as error:
        raise data_root_unavailable(error, multi_host_configuration_path()) from None


def shared_data_root(environment=None):
    """共享数据根的真路径（文件头「共享数据根」）：照 lib_work_root.py 的 shared_data_root 取（SINGLEFS_DATA_ROOT，没设取多机配置的 SHARED_DATA_ROOT），不另算、不补缺省；
    多机配置照这里的取法（SINGLEFS_MULTI_HOST_CONFIG 或主工作树根的 multi-host.env）交给它。取不到抛 Unavailable。"""
    environment = os.environ if environment is None else environment
    path = multi_host_configuration_path(environment)
    library = work_root_library()
    try:
        return os.path.realpath(library.shared_data_root(dict(environment, **{MULTI_HOST_CONFIG_VARIABLE: path})))
    except library.DataRootUnconfigured as error:
        raise data_root_unavailable(error, path) from None


def require_configured_data_root(configuration, environment=None):
    """control install、start、check 判多机配置（文件头「共享数据根」）：只看配置里 SHARED_DATA_ROOT 那一行（lib_work_root.py 的 configured_data_root：写了、绝对路径、
    落在 NFS_SHARED_ROOTS 某一段下），不看环境：服务的单元环境里没有 SINGLEFS_DATA_ROOT，起来时取的就是这一行。→ 数据根；不合抛 Unavailable。
    弄坏开关 data-root-key-unchecked 下不判、交空串。"""
    if broken("data-root-key-unchecked", environment):
        return ""
    library = work_root_library()
    try:
        return library.configured_data_root({MULTI_HOST_CONFIG_VARIABLE: configuration.path})
    except library.DataRootUnconfigured as error:
        raise data_root_unavailable(error, configuration.path) from None


def multi_host_addresses_library():
    """第二台地址的共用取法（文件头「主机」）：照文件路径载入 scripts/hosts/multi_host_addresses.py，一个进程只载入一次。"""
    if not ADDRESSES_LIBRARY:
        ADDRESSES_LIBRARY.append(load_module("multi_host_addresses", MULTI_HOST_ADDRESSES_SCRIPT))
    return ADDRESSES_LIBRARY[0]


def peer_ssh_address(configuration_path, environment=None):
    """第二台的 ssh 地址（文件头「主机」）：→ (地址或 None, 取不到的原因连出路)。调共用取法 multi_host_addresses.py 的 peer_ssh_address（整份配置判过才给地址），
    不另写一份判法。"""
    try:
        library = multi_host_addresses_library()
    except (OSError, ImportError, SyntaxError) as error:
        return None, f"载入不了第二台地址的共用取法 {MULTI_HOST_ADDRESSES_SCRIPT}（{type(error).__name__}: {error}）：看那份文件在不在、改坏没有"
    return library.peer_ssh_address(configuration_path, None if environment is None else dict(environment))


def host_addresses(configuration_path, environment=None):
    """这一台与另一台在 HOSTS 里的地址（文件头「主机」）：→ (Addresses(this, peer) 或 None, 取不到的原因连出路)。调共用取法 multi_host_addresses.py 的 resolve，
    不另写一份判法；task_server.py 照它把 HOSTS 里的地址映到 local / peer（分片交给命令的那一台地址、按台的钉值）。"""
    try:
        library = multi_host_addresses_library()
    except (OSError, ImportError, SyntaxError) as error:
        return None, f"载入不了第二台地址的共用取法 {MULTI_HOST_ADDRESSES_SCRIPT}（{type(error).__name__}: {error}）：看那份文件在不在、改坏没有"
    try:
        return library.resolve(configuration_path, None if environment is None else dict(environment)), ""
    except library.AddressesUnavailable as error:
        return None, f"{error}；{library.HOWTO}"


def require_multi_host_keys(configuration, environment=None):
    """control install、start、check 判多机配置的 HOSTS 与 SHARED_DATA_ROOT（文件头「主机」「共享数据根」）：→ (第二台的地址, 数据根)；不合抛 Unavailable（退 78）。"""
    peer, why = peer_ssh_address(configuration.path, environment)
    if peer is None:
        raise Unavailable(f"取不到第二台的地址（多机配置的 {HOSTS_KEY}）：{why}", f"照 task-server/README.md「## NFS 共享」的「多机配置写哪几键」改 {configuration.path}；改好再起")
    return peer, require_configured_data_root(configuration, environment)


def main_worktree_root(directory=REPOSITORY_ROOT):
    """directory 所在的主工作树的根（git common-dir 的上一层）；不在 git 仓里或查不了交空串。"""
    try:
        completed = subprocess.run(["git", "-C", directory, "rev-parse", "--path-format=absolute", "--git-common-dir"],
                                   capture_output=True, text=True, stdin=subprocess.DEVNULL, timeout=10)
    except (OSError, subprocess.TimeoutExpired):
        return ""
    common = completed.stdout.strip()
    return os.path.dirname(common) if completed.returncode == 0 and common else ""


def repository_roots():
    """「仓里」指哪几棵树：这一份脚本所在的树与它的主工作树（去重）。"""
    roots = [REPOSITORY_ROOT]
    main_root = main_worktree_root()
    if main_root and os.path.realpath(main_root) != os.path.realpath(REPOSITORY_ROOT):
        roots.append(main_root)
    return roots


def multi_host_configuration_path(environment=None):
    """多机配置在哪（文件头「路径」）：SINGLEFS_MULTI_HOST_CONFIG，否则主工作树根的 multi-host.env，不在 git 仓里取这一棵树根上的。"""
    environment = os.environ if environment is None else environment
    explicit = environment.get(MULTI_HOST_CONFIG_VARIABLE, "")
    if explicit:
        return explicit
    return os.path.join(main_worktree_root() or REPOSITORY_ROOT, MULTI_HOST_CONFIG_NAME)


def default_values(environment):
    home = environment.get("HOME") or os.path.expanduser("~")
    runtime = environment.get("XDG_RUNTIME_DIR") or f"/run/user/{os.getuid()}"
    state_home = environment.get("XDG_STATE_HOME") or os.path.join(home, ".local", "state")
    config_home = environment.get("XDG_CONFIG_HOME") or os.path.join(home, ".config")
    data_home = environment.get("XDG_DATA_HOME") or os.path.join(home, ".local", "share")
    return {"HOME": home, "XDG_RUNTIME_DIR": runtime, "XDG_STATE_HOME": state_home, "XDG_CONFIG_HOME": config_home, "XDG_DATA_HOME": data_home,
            "TASK_SERVER_STATE_DIRECTORY": os.path.join(state_home, "singlefs", "task-server"),
            "TASK_SERVER_RUNTIME_DIRECTORY": os.path.join(runtime, "singlefs-task-server"),
            "TASK_SERVER_SIGNING_KEY_FILE": os.path.join(config_home, "singlefs", "task-server-signing.key"),
            "TASK_SERVER_BUILD_DIRECTORY": os.path.join(data_home, "singlefs", "task-server-builds")}


def expand_value(text, defaults):
    if text.startswith("~/"):
        text = defaults["HOME"] + text[1:]
    for name in EXPANDED_VARIABLES:
        text = text.replace("${" + name + "}", defaults[name])
    return text


def parse_optional_keys(text, path):
    """多机配置里 TASK_SERVER_ 起头的那几行 → ({键: 值}, [问题])；别的行归多机配置的判法，这里不看。"""
    values, problems = {}, []
    for number, line in enumerate(text.splitlines(), 1):
        stripped = line.strip()
        if not stripped.startswith(OPTIONAL_KEY_PREFIX):
            continue
        name, separator, value = stripped.partition("=")
        name, value = name.strip(), value.strip()
        if len(value) >= 2 and value[0] == value[-1] and value[0] in "'\"":
            value = value[1:-1]
        if not separator or name not in OPTIONAL_KEYS:
            problems.append(f"{path}:{number} 认不出（TASK_SERVER_ 起头的键只认 {'、'.join(OPTIONAL_KEYS)}，一行一个 键=值）")
            continue
        if name in values:
            problems.append(f"{path}:{number} {name} 写了第二遍")
            continue
        values[name] = value
    return values, problems


def filesystem_type(path):
    """path（或它最近一层在的祖先）所在的文件系统类型；/proc/self/mounts 读不了交空串。"""
    probe = os.path.realpath(path)
    while not os.path.exists(probe) and probe != "/":
        probe = os.path.dirname(probe)
    best, kind = "", ""
    try:
        with open("/proc/self/mounts", encoding="utf-8") as handle:
            for line in handle:
                fields = line.split()
                if len(fields) < 3:
                    continue
                mount_point = fields[1].replace("\\040", " ")
                if (probe == mount_point or probe.startswith(mount_point.rstrip("/") + "/")) and len(mount_point) >= len(best):
                    best, kind = mount_point, fields[2]
    except OSError:
        return ""
    return kind


def inside(path, directory):
    path, directory = os.path.realpath(path), os.path.realpath(directory)
    return path == directory or path.startswith(directory.rstrip("/") + "/")


def inside_repository(path):
    """落在仓里的话交那一棵树的根，不在交空串。"""
    return next((root for root in repository_roots() if inside(path, root)), "")


def state_placement_problem(state_directory, runtime_directory, build_directory, defaults, environment):
    """几个目录放得对不对（文件头「路径」）：状态目录落在仓里、运行时目录下或 tmpfs / ramfs 上，编译目录落在仓里，交原因；放得对交空串。"""
    if broken("state-placement-unchecked", environment):
        return ""
    root = inside_repository(state_directory)
    if root:
        return f"状态目录 {state_directory} 落在仓里（{root}）：日志与库不进仓，仓根的日志会被清"
    for runtime in (runtime_directory, defaults["XDG_RUNTIME_DIR"]):
        if inside(state_directory, runtime):
            return f"状态目录 {state_directory} 落在运行时目录 {runtime} 下：那里是 tmpfs，重启就清"
    kind = filesystem_type(state_directory)
    if kind in VOLATILE_FILESYSTEMS and not selftest_override("TASK_SERVER_SELFTEST_ALLOW_TMPFS", environment):
        return f"状态目录 {state_directory} 在 {kind} 上：重启就清，日志留不住"
    root = inside_repository(build_directory)
    if root:
        return f"编译目录 {build_directory} 落在仓里（{root}）：每次尝试的编译目录不许长在仓里"
    return ""


def socket_path_problem(socket_path, environment=None):
    """socket 的全路径放不放得进 unix socket 的地址（文件头「路径」）：太长交原因（点名路径、字节数与上限），放得下交空串。"""
    if broken("socket-path-unchecked", environment):
        return ""
    length = len(os.fsencode(socket_path))
    if length <= SOCKET_PATH_MAXIMUM_BYTES:
        return ""
    return (f"socket 路径 {socket_path} 有 {length} 字节，超过 unix socket 地址放得下的 {SOCKET_PATH_MAXIMUM_BYTES} 字节"
            "（Linux 的 sun_path 只有 108 字节、含结尾的零）：服务绑不上、客户端连不上")


def read_configuration(environment=None):
    """读路径（文件头「路径」）：交 Configuration；多机配置不在、TASK_SERVER_ 那几个键写坏、目录放错都抛 Unavailable。"""
    environment = os.environ if environment is None else environment
    path = multi_host_configuration_path(environment)
    defaults = default_values(environment)
    try:
        with open(path, encoding="utf-8") as handle:
            text = handle.read()
    except FileNotFoundError:
        if not broken("config-missing-ignored", environment):
            raise Unavailable(f"多机配置 {path} 不在（task-server 照它调度资源；模板是仓根 multi-host.env.example）",
                              f"{CONFIGURATION_STEP}；不在主工作树里跑的设 {MULTI_HOST_CONFIG_VARIABLE}=<主工作树根>/{MULTI_HOST_CONFIG_NAME}") from None
        text = ""
    except OSError as error:
        raise Unavailable(f"多机配置 {path} 读不了（{error}）", "看它的属主与权限，修好再跑") from None
    values, problems = parse_optional_keys(text, path)
    if problems:
        raise Unavailable("多机配置里 TASK_SERVER_ 那几个键写坏了：" + "；".join(problems), f"照 task-server/task_server_lib.py 文件头「路径」改 {path}，或删掉那几行取缺省")
    resolved = {}
    for name in OPTIONAL_KEYS:
        value = expand_value(values[name], defaults) if values.get(name) else defaults[name]
        if not os.path.isabs(value):
            raise Unavailable(f"多机配置 {path} 里 {name} 展开之后是「{value}」，不是绝对路径", f"写成绝对路径，或删掉这一行取缺省（{defaults[name]}）")
        resolved[name] = os.path.normpath(value)
    problem = state_placement_problem(resolved["TASK_SERVER_STATE_DIRECTORY"], resolved["TASK_SERVER_RUNTIME_DIRECTORY"], resolved["TASK_SERVER_BUILD_DIRECTORY"],
                                      defaults, environment)
    if problem:
        raise Unavailable(problem, f"把 {path} 里那个键改到仓外、不会被清的目录，或删掉那一行取缺省（文件头「路径」）")
    problem = socket_path_problem(os.path.join(resolved["TASK_SERVER_RUNTIME_DIRECTORY"], SOCKET_NAME), environment)
    if problem:
        raise Unavailable(problem, f"把 {path} 里的 TASK_SERVER_RUNTIME_DIRECTORY 改到短一点的目录（接上 /{SOCKET_NAME} 不超过 {SOCKET_PATH_MAXIMUM_BYTES} 字节），"
                          f"或删掉那一行取缺省 {defaults['TASK_SERVER_RUNTIME_DIRECTORY']}（缺省也太长的，把 XDG_RUNTIME_DIR 设成短的目录）")
    return Configuration(path, resolved["TASK_SERVER_STATE_DIRECTORY"], resolved["TASK_SERVER_RUNTIME_DIRECTORY"], resolved["TASK_SERVER_SIGNING_KEY_FILE"],
                         resolved["TASK_SERVER_BUILD_DIRECTORY"])


# ── NFS 共享（文件头「NFS 共享」）──
def configured_value(path, name):
    """多机配置里一个键的值：→ (值或 None, 读不到的原因)。NFS_SHARED_ROOTS、HOSTS 与按台的键（configured_by_host）都照这一套读：
    以「名字=」起头的行里最后一行算数；去掉行尾「 # …」注释与成对的引号。"""
    try:
        with open(path, encoding="utf-8") as handle:
            lines = [line for line in handle.read().splitlines() if re.match(rf"^\s*{re.escape(name)}=", line)]
    except OSError as error:
        return None, f"多机配置 {path} 读不了（{error}）"
    if not lines:
        return None, f"多机配置 {path} 里没有 {name}"
    value = re.sub(r"\s+#.*$", "", lines[-1].split("=", 1)[1]).strip()
    if len(value) >= 2 and value[0] == value[-1] and value[0] in "'\"":
        value = value[1:-1]
    return value, ""


def configured_by_host(configuration_path, key, environment=None):
    """多机配置里一个按台的键（文件头「路径」按台的三个键）：→ ({地址: 值} 或 None, 读不到或写坏的原因)。HOSTS 与这个键照 configured_value 读，
    切法与「每台恰好一项、地址都在 HOSTS 里」两条判法交给共用取法 multi_host_addresses.py 的 by_host_values，不另写一份。
    弄坏开关 by-host-missing-accepted、by-host-outside-accepted 照 BY_HOST_FORWARDED_BREAKS 转给共用取法的同名开关。"""
    try:
        library = multi_host_addresses_library()
    except (OSError, ImportError, SyntaxError) as error:
        return None, f"载入不了按台的键的共用取法 {MULTI_HOST_ADDRESSES_SCRIPT}（{type(error).__name__}: {error}）：看那份文件在不在、改坏没有"
    hosts_text, why = configured_value(configuration_path, HOSTS_KEY)
    if hosts_text is None:
        return None, why
    text, why = configured_value(configuration_path, key)
    if text is None:
        return None, f"{why}（三个按台的键都要写，仓根 multi-host.env.example）"
    forwarded = ",".join(item for name, item in BY_HOST_FORWARDED_BREAKS if broken(name, environment))
    try:
        return library.by_host_values(text, hosts_text.split(), key, {library.BREAK_VARIABLE: forwarded}), ""
    except library.ByHostValueUnavailable as error:
        return None, f"多机配置 {configuration_path} 写坏了：{error}"


class NfsProbeTimedOut(Exception):
    """control check --probe 现探的总时长到点：str 写卡在哪一项（NfsProbeBudget.stuck_text）。"""


class NfsProbeBudget:
    """control check --probe 现探 NFS 共享用的预算：总时长上限 limit 秒（NFS_LIVE_PROBE_LIMIT_SECONDS 或 --limit），每一项开始、结束各经 progress 打一行（带这一项用了几秒），
    ssh 每一次的时限取 NFS_SSH_SECONDS 与剩下的时间里小的那个；到点抛 NfsProbeTimedOut，写卡在哪一项、已多久。clock 只给自证换。"""

    def __init__(self, limit, progress=None, clock=time.monotonic):
        self.limit, self.progress, self.clock = float(limit), progress, clock
        self.deadline = clock() + self.limit
        self.item, self.item_started = "", None

    def remaining(self):
        return self.deadline - self.clock()

    def check(self):
        if self.remaining() <= 0:
            raise NfsProbeTimedOut(self.stuck_text())

    def seconds_for(self, wanted):
        """一次 ssh 最多等多久：wanted 与剩下的时间里小的；已到点的抛 NfsProbeTimedOut。"""
        self.check()
        return min(float(wanted), self.remaining())

    def begin(self, root, name):
        self.item, self.item_started = f"共享根 {root} 的「{name}」", self.clock()
        self.say(f"  … {self.item}开始")

    def end(self, verdict):
        self.say(f"  … {self.item}结束（{self.clock() - self.item_started:.1f} 秒）：{'过' if verdict == NFS_PASSED else '不过或没判：' + str(verdict)}")
        self.item, self.item_started = "", None

    def say(self, line):
        if self.progress is not None:
            self.progress(line)

    def stuck_text(self):
        where = f"卡在 {self.item}（这一项已 {self.clock() - self.item_started:.0f} 秒）" if self.item else "还没进任何一项"
        return f"NFS 共享现探总时长 {self.limit:g} 秒到点：{where}"


def ssh_control_path_problem(control_directory):
    """控制目录放不放得下 ssh 的控制 socket：<控制目录>/<%C 的 40 个字符> 再接 ssh 建主连接时的临时后缀，不超过 socket 路径的上限（SOCKET_PATH_MAXIMUM_BYTES）。
    → 放得下交空串，放不下交那一句。"""
    longest = len(os.fsencode(control_directory)) + 1 + SSH_CONTROL_HASH_CHARACTERS + SSH_CONTROL_TEMPORARY_SUFFIX_CHARACTERS
    if longest > SOCKET_PATH_MAXIMUM_BYTES:
        return (f"ssh 复用连接的控制目录 {control_directory} 太长：接上 %C 与 ssh 的临时后缀共 {longest} 字节，超过 socket 路径的上限 {SOCKET_PATH_MAXIMUM_BYTES}，"
                "这一次服务起的 ssh 不复用连接")
    return ""


def ssh_multiplex_options(environment=None):
    """服务起的 ssh 复用连接的三项（文件头「主机」）：environment 里记了控制目录（SSH_CONTROL_VARIABLE，enable_ssh_multiplexing 设）的交
    ("-o", "ControlMaster=auto", "-o", "ControlPath=<控制目录>/%C", "-o", "ControlPersist=<SSH_CONTROL_PERSIST_SECONDS>")，没记的交空元组（control 这类不是服务起的）。
    主机名不在这里：照调用方给的（多机配置 HOSTS 里那一台，peer_ssh_address）。弄坏开关 ssh-unmultiplexed 下交空元组。"""
    control = (os.environ if environment is None else environment).get(SSH_CONTROL_VARIABLE, "")
    if not control or broken("ssh-unmultiplexed", environment):
        return ()
    return ("-o", "ControlMaster=auto", "-o", f"ControlPath={control}/%C", "-o", f"ControlPersist={SSH_CONTROL_PERSIST_SECONDS}")


def ssh_shim_text(options):
    """复用层那一份 ssh（enable_ssh_multiplexing 写进 <控制目录>/bin/ssh）的正文：PATH 上它之后头一个可执行的 ssh 接上 options 这几项再起，参数原样（主机名照调用方给的）；
    PATH 上除了它自己没有 ssh 的退 255、标准错误写一句。"""
    words = " ".join(shlex.quote(word) for word in options)
    return ("#!/usr/bin/env bash\n"
            "# task-server 起的 ssh 一律复用连接（task_server_lib.py 的 enable_ssh_multiplexing 每次起服务时照 ssh_shim_text 写这一份）：\n"
            "# PATH 上它之后头一个可执行的 ssh 接上 ControlMaster、ControlPath、ControlPersist 三项再起，参数原样\n"
            'own="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"\n'
            'IFS=: read -r -a entries <<< "${PATH:-}"\n'
            'for entry in "${entries[@]}"; do\n'
            '  [[ -n "$entry" && -f "$entry/ssh" && -x "$entry/ssh" ]] || continue\n'
            '  [[ "$(cd "$entry" 2>/dev/null && pwd -P)" == "$own" ]] && continue\n'
            f'  exec "$entry/ssh" {words} "$@"\n'
            "done\n"
            'echo "task-server 的 ssh 复用层（$own/ssh）：PATH 上除了它自己没有 ssh" >&2\n'
            "exit 255\n")


def enable_ssh_multiplexing(environment, state_directory):
    """服务起的 ssh 一律复用连接（文件头「主机」；serve 起服务时调）：控制目录是状态目录下的 SSH_CONTROL_DIRECTORY_NAME（700），记进 environment 的 SSH_CONTROL_VARIABLE
    （peer_shell_timed 照它接三项），控制目录下 bin/ssh 写复用层（ssh_shim_text；host-dispatch.py 经 peer-host-lib.sh 起的 ssh 由 ssh_multiplexed_environment 把它接在 PATH 前面）。
    控制目录放不下 socket 路径的（ssh_control_path_problem）、建不了目录或写不了复用层的不记、交那一句（这一次不复用，服务照起）；成了交空串。
    尝试的环境照提交时的白名单另建（CARRIED_VARIABLES），不带这一层：主连接留在服务的单元里，不进尝试的单元。"""
    control = os.path.join(state_directory, SSH_CONTROL_DIRECTORY_NAME)
    problem = ssh_control_path_problem(control)
    if problem:
        return problem
    try:
        ensure_private_directory(control)
        shim_directory = os.path.join(control, SSH_SHIM_DIRECTORY_NAME)
        ensure_private_directory(shim_directory)
        shim = os.path.join(shim_directory, "ssh")
        temporary = f"{shim}.{os.getpid()}.partial"
        with open(temporary, "w", encoding="utf-8") as handle:
            handle.write(ssh_shim_text(ssh_multiplex_options({SSH_CONTROL_VARIABLE: control})))
        os.chmod(temporary, 0o700)
        os.replace(temporary, shim)
    except OSError as error:
        return f"ssh 复用连接的控制目录 {control} 建不了或复用层写不了（{type(error).__name__}: {error}），这一次服务起的 ssh 不复用连接"
    environment[SSH_CONTROL_VARIABLE] = control
    return ""


def ssh_multiplexed_environment(base, environment):
    """host-dispatch.py 那一路（Peer.run → peer-host-lib.sh 的 peer_host_run，命令词是 PATH 上的 ssh）的环境接上复用层：environment 里记了控制目录的，
    交 base 的一份拷贝，PATH 前面接 <控制目录>/bin（enable_ssh_multiplexing 写的那一份 ssh）、SSH_CONTROL_VARIABLE 照记；没记的交 base 的一份拷贝原样。
    弄坏开关 ssh-unmultiplexed 下原样交。"""
    merged = dict(base)
    control = (environment or {}).get(SSH_CONTROL_VARIABLE, "")
    if not control or broken("ssh-unmultiplexed", environment):
        return merged
    merged[SSH_CONTROL_VARIABLE] = control
    merged["PATH"] = os.pathsep.join(part for part in (os.path.join(control, SSH_SHIM_DIRECTORY_NAME), merged.get("PATH", "")) if part)
    return merged


def close_ssh_masters(environment, configuration_path):
    """停服务时关掉复用的主连接（serve 收尾调）：控制目录里每一个 socket 起一次 ssh -o ControlPath=<那个 socket> -O exit <第二台>（第二台的地址只从多机配置取：
    peer_ssh_address，有 socket 才取）。服务单元 KillMode=process，停服务不杀单元里别的进程：不关的主连接留到 ControlPersist 到点，control stop 回读 cgroup.procs 不空。
    → 关了几个。控制目录没记、列不出、没有 socket、第二台的地址取不到的交 0。"""
    control = (environment or {}).get(SSH_CONTROL_VARIABLE, "")
    if not control:
        return 0
    try:
        sockets = [path for path in (os.path.join(control, name) for name in sorted(os.listdir(control))) if stat.S_ISSOCK(os.lstat(path).st_mode)]
    except OSError:
        return 0
    host = peer_ssh_address(configuration_path, environment)[0] if sockets else None
    if not host:
        return 0
    ssh = selftest_override("TASK_SERVER_PEER_SSH", environment) or "ssh"
    closed = 0
    for path in sockets:
        try:
            subprocess.run([ssh, "-o", f"ControlPath={path}", "-O", "exit", host], capture_output=True, text=True, stdin=subprocess.DEVNULL, timeout=10)
        except (OSError, subprocess.TimeoutExpired):
            continue
        closed += 1
    return closed


def peer_shell_timed(host, script, environment=None, budget=None):
    """经 ssh 在第二台上跑一段 shell（NFS_SSH_OPTIONS，服务起的另接复用连接的三项 ssh_multiplex_options）：→ (退出码, 标准输出, 标准错误, 是不是超时)；ssh 起不来或超时退 255。
    自证里 ssh 换成 TASK_SERVER_PEER_SSH 那一份、时限换成 TASK_SERVER_SELFTEST_NFS_SSH_SECONDS。给了 budget（现探）的，时限取它剩下的时间里小的那个，已到点的抛 NfsProbeTimedOut。"""
    ssh = selftest_override("TASK_SERVER_PEER_SSH", environment) or "ssh"
    seconds = float(selftest_override("TASK_SERVER_SELFTEST_NFS_SSH_SECONDS", environment) or NFS_SSH_SECONDS)
    if budget is not None:
        seconds = budget.seconds_for(seconds)
    try:
        completed = subprocess.run([ssh, *NFS_SSH_OPTIONS, *ssh_multiplex_options(environment), host, script], capture_output=True, text=True,
                                   stdin=subprocess.DEVNULL, timeout=seconds)
    except subprocess.TimeoutExpired as error:
        return 255, "", f"{type(error).__name__}: {error}", True
    except OSError as error:
        return 255, "", f"{type(error).__name__}: {error}", False
    return completed.returncode, completed.stdout, completed.stderr, False


def peer_shell(host, script, environment=None, budget=None):
    """peer_shell_timed 去掉是不是超时那一样：→ (退出码, 标准输出, 标准错误)。"""
    code, output, error, _ = peer_shell_timed(host, script, environment, budget)
    return code, output, error


def peer_shell_rechecked(host, script, environment=None, budget=None):
    """连第二台一次超时先当慢（文件头「NFS 共享」的挂载）：→ (退出码, 标准输出, 标准错误, 慢的那一句)。头一次超时的进复查期（NFS_SLOW_GRACE_SECONDS 秒）：每隔
    NFS_RECHECK_PAUSE_SECONDS 复查一回，哪一回回了照那一回的交，慢的那一句写「慢：头一次 ssh 超时 …，隔 … 秒复查到第 N 回回了」；期满（开始下一回之前算）还超时或连不上的
    交 255、标准错误写复查了几回、最后一回的原因。头一次没超时的照它交、不复查。弄坏开关 peer-health-single-timeout-reported 下不复查（头一次超时就照连不上交），
    peer-recheck-once-only 下复查期里只复查一回。给了 budget（control check --probe 的现探）的，每一回 ssh 的时限与回与回之间的等待都不超过它剩下的时间，到点抛 NfsProbeTimedOut。"""
    code, output, error, timed_out = peer_shell_timed(host, script, environment, budget)
    if not timed_out or broken("peer-health-single-timeout-reported", environment):
        return code, output, error, ""
    seconds = float(selftest_override("TASK_SERVER_SELFTEST_NFS_SSH_SECONDS", environment) or NFS_SSH_SECONDS)
    pause = float(selftest_override("TASK_SERVER_SELFTEST_NFS_RECHECK_PAUSE_SECONDS", environment) or NFS_RECHECK_PAUSE_SECONDS)
    grace = float(selftest_override("TASK_SERVER_SELFTEST_NFS_SLOW_GRACE_SECONDS", environment) or NFS_SLOW_GRACE_SECONDS)
    started, rechecks = time.monotonic(), 0
    while True:
        if budget is not None:
            budget.check()
        time.sleep(pause if budget is None else min(pause, max(0.0, budget.remaining())))
        again, output, second_error, second_timed_out = peer_shell_timed(host, script, environment, budget)
        rechecks += 1
        if again != 255:
            return again, output, second_error, (f"慢：头一次 ssh 超时 {seconds:g} 秒（{last_line(error, '没有输出')}），隔 {pause:g} 秒复查到第 {rechecks} 回回了"
                                                f"（复查了 {time.monotonic() - started:.0f} 秒）")
        last = "又超时" if second_timed_out else f"连不上（{last_line(second_error, '没有输出')}）"
        if time.monotonic() - started >= grace or broken("peer-recheck-once-only", environment):
            return 255, output, f"ssh 超时 {seconds:g} 秒，复查期 {grace:g} 秒里隔 {pause:g} 秒复查了 {rechecks} 回仍不通（最后一回{last}）", ""


def last_line(text, fallback):
    lines = [line.strip() for line in (text or "").splitlines() if line.strip()]
    return lines[-1] if lines else fallback


def exported_roots(environment=None):
    """NFS_EXPORTS_FILE 里各行的第一列（本机此刻导出的目录）：→ (集合, 读不到的原因)。自证里读 TASK_SERVER_NFS_EXPORTS 那一份。"""
    path = selftest_override("TASK_SERVER_NFS_EXPORTS", environment) or NFS_EXPORTS_FILE
    try:
        with open(path, encoding="utf-8") as handle:
            return {line.split()[0] for line in handle if line.split()}, ""
    except OSError as error:
        return set(), f"{path} 读不了（{error}）：本机没装 nfs-kernel-server 或没导出"


def nfs_probe_directory(root, environment=None):
    """两向探针写在哪（文件头「NFS 共享」的两向探针）：根下的 .git 是目录时写进它，不在项目根下留条目；别的根写在根下。弄坏开关 nfs-probe-at-root 下一律写在根下。"""
    git_directory = os.path.join(root, ".git")
    return git_directory if os.path.isdir(git_directory) and not broken("nfs-probe-at-root", environment) else root


def nfs_probe(root, host, environment=None, budget=None):
    """两向探针（文件头「NFS 共享」）：→「过」或一句为什么不过。读完两个探针文件都删（共享根在本机盘上，本机删就是两边都删）。
    弄坏开关 nfs-probe-one-way 下只做本机→第二台那一向。"""
    probe_directory = nfs_probe_directory(root, environment)
    # 每一趟的探针文件名各带一个随机后缀：几个会话的看门狗同时起 control check 时，一趟收尾删文件不会删到另一趟正要读的那一份
    suffix = "" if broken("nfs-probe-shared-name", environment) else f"-{secrets.token_hex(8)}"
    local_file = os.path.join(probe_directory, NFS_PROBE_NAME + suffix)
    peer_file = os.path.join(probe_directory, NFS_PEER_PROBE_NAME + suffix)
    try:
        token = secrets.token_hex(16)
        with open(local_file, "w", encoding="utf-8") as handle:
            handle.write(token)
        reader = (f"for try in $(seq 1 {NFS_PROBE_READ_TRIES}); do cat -- {shlex.quote(local_file)} 2>/dev/null && exit 0; sleep 1; done; exit 1")
        code, seen, error = peer_shell(host, reader, environment, budget)
        if code != 0:
            return f"本机→第二台不过：本机写的 {local_file}，第二台读不到（退 {code}：{last_line(error, '没有输出')}）"
        if seen != token:
            return f"本机→第二台不过：本机写的 {local_file}，第二台读回的不是同一个记号"
        if broken("nfs-probe-one-way", environment):
            return NFS_PASSED
        token = secrets.token_hex(16)
        code, _, error = peer_shell(host, f"printf %s {token} > {shlex.quote(peer_file)}", environment, budget)
        if code != 0:
            return f"第二台→本机不过：第二台写不进 {peer_file}（退 {code}：{last_line(error, '没有输出')}）"
        try:
            with open(peer_file, encoding="utf-8") as handle:
                seen = handle.read()
        except OSError as read_error:
            return f"第二台→本机不过：第二台写的 {peer_file} 本机读不到（{read_error}）"
        return NFS_PASSED if seen == token else f"第二台→本机不过：第二台写的 {peer_file}，本机读回的不是同一个记号"
    except OSError as error:
        return f"本机写探针 {local_file} 出错（{error}）"
    finally:
        for path in (local_file, peer_file):
            with contextlib.suppress(OSError):
                os.remove(path)


def nfs_mount_verdict(root, host, environment=None, slow=None, budget=None):
    """「挂载」那一项（文件头「NFS 共享」）：经 ssh 在第二台上 stat -f 这个根下面的「.」——走进目录会触发 x-systemd.automount 挂上它（自动挂载点闲着时
    只是一层 autofs 的占位，mountpoint -q 照样退 0，不说明 NFS 挂上了），读到的文件系统类型是 nfs 才过。→「过」或一句为什么不过、怎么在第二台恢复。
    ssh 一次超时照 peer_shell_rechecked 复查一次：复查回了照回的判，慢的那一句（带上根）追加进 slow（给了的话）；复查仍不通的写连不上。
    弄坏开关 nfs-mount-placeholder-accepted 下照改前只看 mountpoint -q（占位也算挂上）。"""
    if broken("nfs-mount-placeholder-accepted", environment):
        code, _, error, slowly = peer_shell_rechecked(host, f"mountpoint -q -- {shlex.quote(root)}", environment, budget)
        if slowly and slow is not None:
            slow.append(f"共享根 {root} 的「挂载」{slowly}")
        return NFS_PASSED if code == 0 else (f"第二台 mountpoint -q 退 {code}：没把它挂在同一路径" if code != 255
                                             else f"连不上第二台（{last_line(error, '没有输出')}）")
    code, output, error, slowly = peer_shell_rechecked(host, f"{NFS_MOUNT_PROBE_TEXT} {shlex.quote(root + '/.')}", environment, budget)
    if slowly and slow is not None:
        slow.append(f"共享根 {root} 的「挂载」{slowly}")
    if code == 255:
        return f"连不上第二台（{last_line(error, 'ssh 退 255，没有输出')}）"
    kind = output.strip()
    if code == 0 and kind in NFS_FILESYSTEM_TYPES:
        return NFS_PASSED
    found = f"stat -f 退 {code}：{last_line(error, '没有输出')}" if code != 0 else f"那里的文件系统是 {kind or '（空）'}、不是 NFS"
    return f"第二台读不到挂上的 NFS（{found}）：{NFS_MOUNT_RECOVERY.format(root=root)}"


def budgeted_item(budget, root, key, compute):
    """NFS 共享的一项（NFS_ITEMS 里 key 那一项）：没给 budget 的直接算；给了的开始、结束各经 budget 打一行。"""
    if budget is None:
        return compute()
    budget.begin(root, dict(NFS_ITEMS)[key])
    verdict = compute()
    budget.end(verdict)
    return verdict


def nfs_share_report(configuration_path, environment=None, probe=True, budget=None):
    """NFS 共享判得过没有（文件头「NFS 共享」）：→ {ok, problem, next_step, roots: [{root, directory, export, mount, probe}…], checked_at, slow}。
    slow 是「挂载」那一项 ssh 头一次超时、复查回了的那几句（nfs_mount_verdict；服务只记进服务日志）。
    每一项是「过」或一句为什么不过（没判的写「没判：…」）；problem 是第一处不过的那一句（哪个根、哪一项、为什么）。probe=False 不做两向探针。
    budget（NfsProbeBudget，control check --probe 的现探）给了的，每一项开始、结束各打一行、ssh 受它的总时长管，到点抛 NfsProbeTimedOut。
    弄坏开关 nfs-unchecked 下一项都不判、照过。"""
    environment = os.environ if environment is None else environment
    report = {"ok": False, "problem": "", "next_step": f"照 {NFS_README} 在本机导出、第二台挂在同一绝对路径，修好再起", "roots": [], "checked_at": round(time.time(), 3),
              "slow": []}
    if broken("nfs-unchecked", environment):
        return dict(report, ok=True)
    text, why = configured_value(configuration_path, NFS_ROOTS_KEY)
    if not text:
        report.update(problem=f"{why or f'多机配置 {configuration_path} 里 {NFS_ROOTS_KEY} 是空的'}：{NFS_ROOTS_MISSING_TEXT}（结果要两台都读得到）",
                      next_step=f"照 {NFS_README} 在本机导出、第二台挂在同一绝对路径，再往 {configuration_path} 写一行 {NFS_ROOTS_KEY}=<绝对路径>[:<绝对路径>…]（冒号分隔）")
        return report
    roots = [part.strip() for part in text.split(":")]
    if any(not root or not os.path.isabs(root) for root in roots):
        report.update(problem=f"多机配置 {configuration_path} 的 {NFS_ROOTS_KEY}「{text}」写法不对：要是冒号分隔的绝对路径",
                      next_step=f"改成 {NFS_ROOTS_KEY}=<绝对路径>[:<绝对路径>…]（{NFS_README}）")
        return report
    host, host_why = peer_ssh_address(configuration_path, environment)
    exports, exports_why = exported_roots(environment)
    for root in roots:
        entry = {"root": root}
        entry["directory"] = budgeted_item(budget, root, "directory", lambda: NFS_PASSED if os.path.isdir(root) else "本机没有这个目录")
        entry["export"] = budgeted_item(budget, root, "export", lambda: NFS_PASSED if root in exports else (
            exports_why or f"{selftest_override('TASK_SERVER_NFS_EXPORTS', environment) or NFS_EXPORTS_FILE} 里没有第一列是它的导出行"))
        if not host:
            entry["mount"] = f"{host_why}：连不了第二台"
        else:
            entry["mount"] = budgeted_item(budget, root, "mount", lambda: nfs_mount_verdict(root, host, environment, report["slow"], budget))
        if not probe:
            entry["probe"] = "没判：只在起来时与 control check --probe、install、start 做"
        elif entry["directory"] != NFS_PASSED or entry["mount"] != NFS_PASSED:
            entry["probe"] = "没判：本机目录或第二台挂载没过"
        else:
            entry["probe"] = budgeted_item(budget, root, "probe", lambda: nfs_probe(root, host, environment, budget))
        report["roots"].append(entry)
    for entry in report["roots"]:
        failed = next(((key, name) for key, name in NFS_ITEMS if entry.get(key) != NFS_PASSED and not str(entry.get(key)).startswith("没判")), None)
        if failed:
            report["problem"] = f"共享根 {entry['root']} 的「{failed[1]}」不过：{entry[failed[0]]}"
            return report
    report["ok"] = True
    return report


def require_nfs_share(configuration, environment=None, probe=True):
    """NFS 共享判不过抛 Unavailable（退 78）；判得过交那份结论。"""
    report = nfs_share_report(configuration.path, environment, probe)
    if not report["ok"]:
        raise Unavailable(f"NFS 共享判不过：{report['problem']}", report["next_step"])
    return report


# ── 私有目录与签名密钥 ──
def ensure_private_directory(path):
    os.makedirs(path, mode=0o700, exist_ok=True)
    os.chmod(path, 0o700)


def read_signing_key(path):
    """读签名密钥（64 位小写十六进制、属主自己、600、所在目录 700）；不对就抛 Unavailable。只有服务读它。"""
    try:
        status = os.stat(path)
    except FileNotFoundError:
        raise Unavailable(f"签名密钥 {path} 不在", f"{INSTALL_COMMAND}（它排他新建密钥，已在的不动）") from None
    except OSError as error:
        raise Unavailable(f"签名密钥 {path} 读不了（{error}）", "看它的属主与权限，修好再跑") from None
    if status.st_uid != os.getuid() or status.st_mode & 0o077:
        raise Unavailable(f"签名密钥 {path} 的属主不是自己或权限不是 600（{oct(status.st_mode & 0o777)}）", f"chmod 600 {path}（属主不对就删掉它重装）")
    if os.stat(os.path.dirname(path)).st_mode & 0o077:
        raise Unavailable(f"签名密钥所在目录 {os.path.dirname(path)} 的权限不是 700", f"chmod 700 {os.path.dirname(path)}")
    with open(path, encoding="utf-8") as handle:
        text = handle.read().strip()
    if not SECRET_PATTERN.fullmatch(text):
        raise Unavailable(f"签名密钥 {path} 写坏了（要 64 位小写十六进制）", "删掉它重装；密钥换了，库里旧的行都会核不过，要连库一起清掉")
    return bytes.fromhex(text)


def create_signing_key(path, environment=None):
    """排他新建签名密钥（32 字节随机数的十六进制，600）；已在就不动、交 False。弄坏开关 secrets-overwritten 下已在也重写。"""
    ensure_private_directory(os.path.dirname(path))
    if broken("secrets-overwritten", environment) and os.path.exists(path):
        os.remove(path)
    try:
        descriptor = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    except FileExistsError:
        return False
    with os.fdopen(descriptor, "w", encoding="utf-8") as handle:
        handle.write(secrets.token_hex(32) + "\n")
    return True


def row_signature(key, table, row_id, state, session, command_key, record_text):
    message = "\n".join([table, str(row_id), state or "", session or "", command_key or "", record_text])
    return hmac.new(key, message.encode("utf-8"), hashlib.sha256).hexdigest()


def canonical_json(value):
    return json.dumps(value, sort_keys=True, ensure_ascii=False, separators=(",", ":"))


# ── 环境变量名单（文件头「环境变量」）──
def split_environment(environment, broken_environment=None):
    """→ (带进去的 {名: 值}, 拒的 [(名, 为什么)])；会话自带的两边都不进。"""
    carried, refused = {}, []
    for name in sorted(environment):
        if name in AMBIENT_VARIABLES or name.startswith(AMBIENT_PREFIXES):
            continue
        if name in RETIRED_VARIABLES:   # 弄坏开关 retired-variables-carried：照改前带进去
            if broken("retired-variables-carried", broken_environment):
                carried[name] = environment[name]
            else:
                refused.append((name, RETIRED_VARIABLES[name]))
            continue
        if name in CARRIED_VARIABLES or name.startswith(CARRIED_PREFIXES):
            carried[name] = environment[name]
            continue
        if name in TAKEN_OVER_VARIABLES or name.startswith(TAKEN_OVER_PREFIXES):
            if broken("queue-variables-carried", broken_environment):
                carried[name] = environment[name]
            else:
                refused.append((name, TAKEN_OVER_VARIABLES.get(name, "排队、记账与挑机器归 task-server，内存包装与派发器的这一类参数不收")))
            continue
        if broken("environment-unfiltered", broken_environment):
            carried[name] = environment[name]
            continue
        if name.endswith("_BREAK"):
            refused.append((name, "弄坏开关只在各脚本自己的 --selftest 里认"))
            continue
        refused.append((name, "白名单外：不知道它该不该带进任务"))
    return carried, refused


def environment_refusal_lines(refused):
    lines = [f"  ✗ 这几个环境变量 task-server 不收（不静默丢，当场拒）：" + "、".join(name for name, _ in refused)]
    lines += [f"      {name}：{why}" for name, why in refused]
    retired = [name for name, _ in refused if name in RETIRED_VARIABLES]
    lines.append("  → 怎么办：unset 它们再提交；命令真要它的，写进命令里：task-server/task submit -- env <名>=<值> <命令…>（原样跑）；"
                 "该常带的，往 task-server/task_server_lib.py 的 CARRIED_VARIABLES 里加"
                 + (f"；{'、'.join(retired)} 是退役的名字，命令里也不写（照上面它那一行的出路办）" if retired else ""))
    return lines


def command_word_index(argv, environment=None):
    """剥掉 env（连同 名=值、-u 名、-C 目录、-S 串）、nice（-n N、-nN、--adjustment N、-N）、timeout（-k 时长、-s 信号与别的开关之后的 <时长>）、setsid、
    bash / sh（-o、-O 带一个值）/ python3（-W、-X 带一个值）起脚本、capped.sh N 之后命令词在 argv 里的位置；剥完了没有交 len(argv)。
    带值的选项值写在下一个词里的连同值一起跳过（不停在选项值上，把值当命令词）。弄坏开关 command-wrapper-values-unskipped 下只认 env 的 -u、nice 的 -n 两种分开写的值。"""
    unskipped = broken("command-wrapper-values-unskipped", environment)
    env_values = ("-u", "--unset") if unskipped else ("-u", "--unset", "-C", "--chdir", "-S", "--split-string")
    index = 0
    while index < len(argv):
        word = argv[index]
        name = os.path.basename(word)
        if name == "env":
            index += 1
            while index < len(argv) and ("=" in argv[index] or argv[index].startswith("-")):
                index += 2 if argv[index] in env_values else 1
            continue
        if name == "nice":
            index += 1
            if unskipped:
                index += 2 if index < len(argv) and argv[index] == "-n" else 0
                continue
            while index < len(argv) and argv[index].startswith("-"):
                index += 2 if argv[index] in ("-n", "--adjustment") else 1
            continue
        if name == "timeout":
            index += 1
            while index < len(argv) and argv[index].startswith("-"):
                index += 2 if argv[index] in ("-k", "--kill-after", "-s", "--signal") and not unskipped else 1
            index += 1
            continue
        if name == "setsid":
            index += 1
            while index < len(argv) and argv[index].startswith("-"):
                index += 1
            continue
        if name in ("bash", "sh", "python3", "python"):
            index += 1
            valued = () if unskipped else (("-W", "-X") if name.startswith("python") else ("-o", "-O"))
            while index < len(argv) and argv[index].startswith("-") and argv[index] != "-c":
                index += 2 if argv[index] in valued else 1
            continue
        if name == "capped.sh":
            index += 2
            continue
        break
    return index


def retired_assignment(argv):
    """命令外层写了退役变量的「名=值」（env 后面的，与命令位置前的裸赋值；文件头「环境变量」）：交那个词；没有交空串。"""
    index = command_word_index(argv)
    words = list(argv[:index])
    while index < len(argv) and ASSIGNMENT_WORD.match(argv[index]):
        words.append(argv[index])
        index += 1
    return next((word for word in words if "=" in word and word.split("=", 1)[0] in RETIRED_VARIABLES), "")


def nested_dispatch(argv, environment=None):
    """命令自己在命令位置上套了内存包装、派发器或 multi-host-run.sh 的跑法，或在外层写了退役变量：交说明；没有交空串（剥法见 command_word_index）。"""
    if not broken("retired-variable-assignments-accepted", environment):
        retired = retired_assignment(argv)
        if retired:
            return f"{retired}：{RETIRED_VARIABLES[retired.split('=', 1)[0]]}（去掉这个赋值）"
    if broken("nested-dispatch-allowed", environment):
        return ""
    index = command_word_index(argv)
    if index >= len(argv):
        return ""
    name = os.path.basename(argv[index])
    rest = argv[index + 1:]
    if not broken("wrapper-prefixes-allowed", environment):
        for word in argv[:index]:
            if os.path.basename(word) in REFUSED_PREFIXES:
                return f"{word} …：{REFUSED_PREFIXES[os.path.basename(word)]}由服务定，命令里不套它（去掉这一层）"
            if word.startswith("SINGLEFS_THREAD_CAP="):
                return f"{word}：线程一律是登记的每台默认重活线程数，命令里不写它（去掉这个赋值）"
        if name in REFUSED_PREFIXES and rest not in WRAPPER_OWN_ARGUMENTS:
            return f"{argv[index]} …：{REFUSED_PREFIXES[name]}由服务定，命令里不套它（去掉这一层）"
    if not broken("resource-arguments-allowed", environment):
        for word in rest:
            if word in RESOURCE_ARGUMENTS or word.split("=", 1)[0] in RESOURCE_ARGUMENTS and "=" in word:
                return f"{word}：上限、线程与挑哪台由服务定，命令里不写它（去掉这个参数）"
    if name == "run-with-memory-cap.sh" and rest and (re.fullmatch(r"[1-9][0-9]*[KMGT]", rest[0]) or rest[0] == "auto"):
        return f"{argv[index]} {rest[0]} …：内存上限由服务定，命令里不套内存包装"
    if name in ("host-dispatch.py", "multi-host-run.sh") and "--" in rest:
        return f"{argv[index]} … -- …：挑机器、派第二台由服务定，命令里不套派发器"
    return ""


# ── 编译目录（文件头「编译目录」）──
def given_target_directories(environment, argv, cwd):
    """调用方给的编译目录：→ [(从哪来, 绝对路径)]：环境里的两个变量、命令词里的 CARGO_TARGET_DIR=… 赋值与 --target-dir。"""
    found = []
    for name in TARGET_DIRECTORY_VARIABLES:
        if environment.get(name):
            found.append((f"环境里的 {name}", environment[name]))
    for index, word in enumerate(argv):
        for name in TARGET_DIRECTORY_VARIABLES:
            if word.startswith(name + "="):
                found.append((f"命令里的 {name}=", word[len(name) + 1:]))
        if word.startswith("--target-dir="):
            found.append(("命令里的 --target-dir", word[len("--target-dir="):]))
        elif word == "--target-dir" and index + 1 < len(argv):
            found.append(("命令里的 --target-dir", argv[index + 1]))
    return [(source, os.path.normpath(os.path.join(cwd, os.path.expanduser(value)))) for source, value in found if value]


def target_directory_refusal(environment, argv, cwd, broken_environment=None):
    """调用方给的编译目录落在仓里：交说明；没给或在仓外交空串。弄坏开关 in-repository-target-accepted 下一律交空串。"""
    if broken("in-repository-target-accepted", broken_environment):
        return ""
    for source, path in given_target_directories(environment, argv, cwd):
        root = inside_repository(path)
        if root:
            return f"{source}指到仓里（{path}，在 {root} 里）：编译目录不许长在仓里"
    return ""


# ── 取仓根 target/ 下的二进制（task_server.py 文件头 ②「提交时警告」）──
REPOSITORY_TARGET_NAME = "target"
BYPASS_SCRIPT_MAXIMUM_BYTES = 4 * 1024 * 1024   # 读入口脚本正文只读这么大的（推的：本仓的脚本都远小于它，读不全的不判）
BYPASS_HITS_SHOWN = 3   # 警告那一句至多点名几处
# 词里的路径段：引号、命令分隔、重定向与「名=」的等号都切开
BYPASS_WORD_PATH = re.compile(r"[^\s'\"`;|&()<>=]+")
# 入口脚本正文里指向仓根 target/ 的路径字面：前面接仓根变量（shell 的 $<名>、${<名>}、${<名>:-…}，python f 串的 {<名>}；名里带 repo、root 或 toplevel）、
# $(dirname …) 之后往上跳的 /..、$(git rev-parse --show-toplevel)，接 /target/release/ 或 /target/debug/（${CARGO_TARGET_DIR:-$<仓根>/target}/release/ 这种退回仓根的同样算）
ROOTED_TARGET_LITERAL = re.compile(
    r"(?:\$\{?\w*(?:repo|root|toplevel)\w*\}?|\{\w*(?:repo|root|toplevel)\w*\}|dirname[^\n]*?\)\"?(?:/\.\.)+|\$\(git rev-parse --show-toplevel\))"
    r"/target\}?/(?:release|debug)/", re.IGNORECASE)
# 在仓根起的命令，正文里裸写的 target/release/、target/debug/：赋值的值（NAME=、python 的 name = ）、缺省值（${…:-target/…}）与命令位置（行首不带引号）
BARE_TARGET_LITERAL = re.compile(r"(?:^\s*|\w\s*=\s*[\"']?|:-[\"']?)(?:\./)?target/(?:release|debug)/")
# 正文里不读的行：注释、只在弄坏开关下走的那一支（break_is、*_BREAK、broken(）
BYPASS_SKIPPED_LINE = re.compile(r"^\s*(?:#|//)|\bbreak_is\b|_BREAK\b|\bbroken\(")
# 自证那一支：python 名字里带 selftest 或以 cells 收尾的函数体、shell 名字里带 selftest 的函数体（样本里造的假路径，如 memory_peaks_lib.py 的 key_fold_cells）
PYTHON_SELFTEST_DEFINITION = re.compile(r"^(\s*)def\s+(\w*selftest\w*|\w*cells)\s*\(")
SHELL_SELFTEST_DEFINITION = re.compile(r"^\s*(?:function\s+)?[\w-]*selftest[\w-]*\s*\(\)")


def script_target_literals(script_path, bare_counts):
    """入口脚本正文里指向仓根 target/ 的路径字面（ROOTED_TARGET_LITERAL；bare_counts 为真时另认 BARE_TARGET_LITERAL）：→ [(行号, 那一段字面)]。
    注释、弄坏开关那一支（BYPASS_SKIPPED_LINE）与自证那一支的函数体不读；读不了、不是普通文件、超过 BYPASS_SCRIPT_MAXIMUM_BYTES 的交 []。"""
    try:
        if not os.path.isfile(script_path) or os.path.getsize(script_path) > BYPASS_SCRIPT_MAXIMUM_BYTES:
            return []
        with open(script_path, encoding="utf-8", errors="replace") as handle:
            lines = handle.read().splitlines()
    except OSError:
        return []
    found, python_indent, in_shell_selftest = [], None, False
    for number, line in enumerate(lines, 1):
        if in_shell_selftest:
            in_shell_selftest = not line.startswith("}")
            continue
        if python_indent is not None:
            if not line.strip() or len(line) - len(line.lstrip()) > python_indent:
                continue
            python_indent = None
        definition = PYTHON_SELFTEST_DEFINITION.match(line)
        if definition:
            python_indent = len(definition.group(1))
            continue
        if SHELL_SELFTEST_DEFINITION.match(line):
            in_shell_selftest = not line.rstrip().endswith("}")
            continue
        if BYPASS_SKIPPED_LINE.search(line):
            continue
        match = ROOTED_TARGET_LITERAL.search(line) or (BARE_TARGET_LITERAL.search(line) if bare_counts else None)
        if match:
            found.append((number, match.group(0).strip()))
    return found


def repository_target_references(argv, cwd, roots, environment):
    """这一条命令取仓根 target/ 下的东西（task_server.py 文件头 ②「提交时警告」）：→ [那一处的说明…]，没有交 []。
    roots 是「仓根」那几棵树（服务的仓根与主工作树；草稿与临时目录下的仓副本不在里面）。认两样：
    ① 命令里的词（整个词、「名=值」的值、bash -c 词里的每一段；相对路径照工作目录解）落在某个仓根的 target/ 下；调用方给的编译目录（given_target_directories）下的不算；
    ② 命令位置上起的本仓脚本（command_word_index 剥过之后那一个词；不是 -c）正文里有指向仓根 target/ 的路径字面（script_target_literals；工作目录就是仓根时另认裸写的）。
    带 --selftest 起的脚本走自证那一支（样本里的假路径），不读正文。"""
    targets = [os.path.join(os.path.realpath(root), REPOSITORY_TARGET_NAME) for root in roots]
    given = [path for _, path in given_target_directories(environment, argv, cwd)]
    found = []
    for word in argv:
        for piece in BYPASS_WORD_PATH.findall(word):
            if REPOSITORY_TARGET_NAME not in piece or "$" in piece:
                continue
            path = os.path.normpath(os.path.join(cwd, piece))
            if (any(inside(path, target) and os.path.realpath(path) != target for target in targets)
                    and not any(inside(path, directory) for directory in given)):
                found.append(f"命令里的 {piece}（{path}）")
                break
    index = command_word_index(argv)
    if index < len(argv) and argv[index] != "-c" and "--selftest" not in argv[index + 1:]:
        script = os.path.realpath(os.path.join(cwd, argv[index]))
        root = next((os.path.realpath(root) for root in roots if inside(script, root)), "")
        if root and not inside(script, os.path.join(root, REPOSITORY_TARGET_NAME)):
            bare = os.path.realpath(cwd) in (os.path.realpath(each) for each in roots)
            for number, literal in script_target_literals(script, bare):
                found.append(f"入口脚本 {os.path.relpath(script, root)}:{number} 的 {literal}")
    return found


# ── 分片（文件头「分片」）──
def declared_header_words(script_path, mark):
    """脚本文件头前 DECLARED_HEADER_LINES 行里以 mark 起头的那一行写的词（分片、会编、续跑与卡的声明共用）；文件读不了、不是普通文件或没写交 None。"""
    if not os.path.isfile(script_path):
        return None
    try:
        with open(script_path, encoding="utf-8", errors="replace") as handle:
            for _ in range(DECLARED_HEADER_LINES):
                line = handle.readline()
                if not line:
                    break
                if line.startswith(mark):
                    return tuple(line[len(mark):].split())
    except OSError:
        return None
    return None


def declared_shards(script_path):
    """「# task-server-shards: …」写的第一个参数（* 是都分）；没写交 None。"""
    return declared_header_words(script_path, DECLARED_SHARDS_MARK)


def first_non_option(arguments):
    """参数里第一个不以 - 起头的词；没有交空串。"""
    return next((argument for argument in arguments if not argument.startswith("-")), "")


def declaration_matches(arguments, declared):
    """续跑声明的判法：声明了 * 就是任何参数都算，不然第一个非选项参数（first_non_option）要是声明的词之一。"""
    if not declared:
        return False
    return "*" in declared or first_non_option(arguments) in declared


def declared_compiles(script_path, environment=None):
    """命令位置上起的脚本声明了会编没有（文件头「编译目录」的入口声明）：→ (声明了没有, 写坏的那一句)。只认「# task-server-compiles: yes」；写了别的不猜，
    交那一句（服务照 run-condition-unmet 打回）。文件不在、没有那一行交 (False, "")。弄坏开关 compiles-declaration-ignored 下一律当没声明。"""
    if broken("compiles-declaration-ignored", environment):
        return False, ""
    declared = declared_header_words(script_path, DECLARED_COMPILES_MARK)
    if declared is None:
        return False, ""
    if declared == (COMPILES_DECLARED_WORD,):
        return True, ""
    return False, f"{script_path} 文件头的会编声明写成「{' '.join(declared)}」，要写 {COMPILES_DECLARED_WORD}"


def card_declaration_problem(argv, cwd, environment=None):
    """命令位置上起的脚本（command_word_index 剥过之后那一个词，照工作目录解成真路径）文件头前 DECLARED_HEADER_LINES 行里写了 CARD_DECLARATION_MARKS 之一
    （task_server.py 文件头 ②「提交时拒」⑧；异常表 card-declaration）→ {problem, next_step}（点名那份脚本与写了哪几种）；没写、文件不在、不是普通文件交 None。
    只查不跑的写法不另放行：声明写在文件头，跟这一次怎么起无关。弄坏开关 card-declarations-accepted 下一律交 None。"""
    if broken("card-declarations-accepted", environment):
        return None
    index = command_word_index(argv)
    if index >= len(argv):
        return None
    script = os.path.realpath(os.path.join(cwd or REPOSITORY_ROOT, argv[index]))
    found = [mark for mark in CARD_DECLARATION_MARKS if declared_header_words(script, mark) is not None]
    if not found:
        return None
    return {"problem": f"{CARD_DECLARATION_REFUSAL}：{script} 文件头写了 {'、'.join(found)}，服务不认领卡、要卡的声明，这一条不收、不跑",
            "next_step": (f"删掉那份脚本文件头的 {'、'.join(found)} 那一行，脚本里要卡的那一段改成往判定库写待判行、交 GPU 批处理验证服务判"
                          "（.claude/rules/verification.md「卡只归 GPU 服务，任务不拿卡」），再照原命令重交")}


def enclosing_user_unit(cgroup_text):
    """进程的 cgroup（/proc/<进程号>/cgroup 里 cgroup v2 那一行「0::<路径>」）→ 它所在的用户级 systemd 单元名：路径里 user@<uid>.service 之下、离进程最近的那一段
    .service 或 .scope（task_server.py 文件头 ③「套在外层单元里的服务」）。不在用户级管理器之下、或认不出的交空串。"""
    for line in (cgroup_text or "").splitlines():
        if not line.startswith("0::"):
            continue
        parts = [part for part in line[3:].strip().split("/") if part]
        manager = next((index for index, part in enumerate(parts) if part.startswith("user@") and part.endswith(".service")), None)
        if manager is None:
            return ""
        return next((part for part in reversed(parts[manager + 1:]) if part.endswith((".service", ".scope"))), "")
    return ""


def cargo_option_values(arguments):
    """cargo 子命令后面、`--` 之前的参数里认这几样：→ {"-p": [包…], "--features": [特性…], "--profile": 档, "--target": 目标, "--test": [测试目标…], 别的开关: True}。
    -p / --package、-F / --features、--profile、--target、--test 认「选项 值」「--选项=值」「-p值」三种写法；--release（-r）记成 --profile release。"""
    found = {"-p": [], "--features": [], "--test": [], "--profile": "dev", "--target": ""}
    aliases = {"--package": "-p", "-F": "--features"}
    index = 0
    while index < len(arguments) and arguments[index] != "--":
        word = arguments[index]
        name, value = word, None
        if "=" in word and word.startswith("--"):
            name, value = word.split("=", 1)
        elif word[:2] in ("-p", "-F") and len(word) > 2:
            name, value = word[:2], word[2:]
        name = aliases.get(name, name)
        if name in ("-p", "--features", "--test", "--profile", "--target"):
            if value is None:
                index += 1
                value = arguments[index] if index < len(arguments) else ""
            if name == "--features":
                found[name] += [part for part in re.split(r"[,\s]+", value) if part]
            elif name in ("-p", "--test"):
                found[name].append(value)
            else:
                found[name] = value
        elif word in ("--release", "-r"):
            found["--profile"] = "release"
        elif word.startswith("-"):
            found[word] = True
        index += 1
    return found


MANIFEST_VIRTUAL = "virtual"          # cargo_manifest_scope：只有 [workspace]、没有 [package] 的清单，不点名包就编每一个成员
MANIFEST_PACKAGE = "package"          # 包自己的清单：不点名包只编这一个包
MANIFEST_NONE = "none"                # 往上一份 Cargo.toml 都没有：cargo 自己就报错
MANIFEST_UNREADABLE = "unreadable"    # 有清单而读不了、不是合法的 TOML


def cargo_manifest_scope(directory):
    """cargo 在 directory 起、不点名包时编的范围（task_server.py 文件头 ②「提交时拒」⑤）：照 cargo 找清单的走法，directory 起往上第一份 Cargo.toml。
    → (那份清单的路径或 None, MANIFEST_VIRTUAL / MANIFEST_PACKAGE / MANIFEST_NONE / MANIFEST_UNREADABLE 之一, 读不了的原因或空串)。"""
    current = os.path.abspath(directory)
    while True:
        manifest = os.path.join(current, "Cargo.toml")
        if os.path.isfile(manifest):
            try:
                with open(manifest, "rb") as handle:
                    parsed = tomllib.load(handle)
            except (OSError, tomllib.TOMLDecodeError) as error:
                return manifest, MANIFEST_UNREADABLE, f"{type(error).__name__}: {error}"
            return manifest, MANIFEST_VIRTUAL if "workspace" in parsed and "package" not in parsed else MANIFEST_PACKAGE, ""
        parent = os.path.dirname(current)
        if parent == current:
            return None, MANIFEST_NONE, ""
        current = parent


def resumable_command(argv, cwd, environment=None):
    """这条命令登记了续跑没有（文件头「续跑」）：命令位置上起的脚本（剥掉 env、nice、timeout、setsid、bash / sh / python3、capped.sh N 之后，同 sharded_entry）
    文件头写了「# task-server-resumable: <词>…」、第一个非选项参数是其中一个（或写了 *）。弄坏开关 resumable-declaration-ignored 下一律当没登记，
    resume-declaration-any-word 下非选项参数有一个是声明的词就算（不只看第一个）。"""
    if broken("resumable-declaration-ignored", environment):
        return False
    index = command_word_index(argv)
    if index >= len(argv):
        return False
    script = os.path.realpath(os.path.join(cwd or REPOSITORY_ROOT, argv[index]))
    declared = declared_header_words(script, DECLARED_RESUMABLE_MARK)
    if broken("resume-declaration-any-word", environment):
        return bool(declared) and ("*" in declared or any(argument in declared for argument in argv[index + 1:] if not argument.startswith("-")))
    return declaration_matches(argv[index + 1:], declared)


def first_argument_matches(arguments, accepted, environment=None):
    first = arguments[0] if arguments else ""
    unsharded = first in NOT_SHARDED_ARGUMENTS or any(argument in ("--selftest",) for argument in arguments)
    if unsharded and not broken("unsharded-arguments-sharded", environment):
        return False
    if accepted is None or "*" in accepted:
        return True
    return first in accepted


def sharded_entry(argv, cwd, environment=None):
    """这件要不要分片：交起的那份脚本（仓根下的相对路径，或声明了分片的脚本的绝对路径）；不分交空串。弄坏开关 sharding-unrecognized 下一律交空串。"""
    if broken("sharding-unrecognized", environment):
        return ""
    index = command_word_index(argv)
    if index >= len(argv):
        return ""
    word, arguments = argv[index], argv[index + 1:]
    script = os.path.realpath(os.path.join(cwd, word))
    for relative, accepted in SHARDED_ENTRIES.items():
        if (script.endswith("/" + relative) or word == relative) and first_argument_matches(arguments, accepted, environment):
            return relative
    declared = declared_shards(script) if os.path.isfile(script) else None
    if declared and first_argument_matches(arguments, declared, environment):
        return script
    return ""










def over_run_hard_limit(seconds, limit, environment=None):
    """累计用时到没到四小时硬上限（task_server.py 文件头 ⑦「四小时硬上限」）：恰好等于上限算到。弄坏开关 run-hard-limit-exclusive 下等于上限不算，run-hard-limit-loose 下比上限少一秒也算。"""
    if broken("run-hard-limit-exclusive", environment):
        return seconds > limit
    return seconds >= limit - (1 if broken("run-hard-limit-loose", environment) else 0)


def covered_seconds(intervals, environment=None):
    """几段时段 [(起, 止)…] 的并集有多长（秒）：分片的父任务整件的挂钟时长（task_server.py 的 parent_run_hard_limit，只在弄坏开关 shard-whole-by-wall-clock 下算），同时在跑的不重复算，都不在跑的空档不算。
    弄坏开关 run-hard-limit-parent-summed 下取各段之和。"""
    if broken("run-hard-limit-parent-summed", environment):
        return sum(max(0.0, end - start) for start, end in intervals)
    total, reached = 0.0, None
    for start, end in sorted(intervals):
        start = start if reached is None else max(start, reached)
        if end > start:
            total += end - start
            reached = end
    return total








def shard_child_key(base_key_text, role, index, count, environment=None, compiles=False):
    """子任务的命令键原文（交给内存包装的 RUN_WITH_MEMORY_CAP_KEY）：这件的键加「〈task-server 分片 <步> 第<片号>片〉」，只带片号、不带片数：
    计划切出来的片数变了（2 片、64 片），同一片号还是这把键，但读记录时只认片数相同的那几趟（task_server.py 的 record_fits_task；相当的记录一条都没有的片照没有记录的命令定，同一趟的兄弟片与别的键都不借；文件头「资源记录」）。
    计划那一步的键带「这一步编不编」（compiles：服务给不给这一步编译目录，task_server.py 的 child_compiles）：「〈task-server 分片 plan 编〉」或「〈task-server 分片 plan 不编〉」，
    编与不编的计划步各记各的（计划这一步预编测试二进制时的峰值比只算键、核账的大得多，不编的记录不当编的那一种的起点）。
    加的那一段不带 /：键的换算（memory_peaks_lib.py 文件头「键」③）把带 / 的位置参数折成〈文件〉，带了各片就落一个键。
    弄坏开关 shard-key-position-folded 下写成 <片号>/<片数>，shard-key-count-kept 下写成「第<片号>片共<片数>片」（片数一变各片都换新键），
    plan-key-compile-unmarked 下计划那一步不带编不编（编与不编落同一把键）。"""
    if role == "plan" and not broken("plan-key-compile-unmarked", environment):
        position = " 编" if compiles else " 不编"
    elif role != "shard":
        position = ""
    elif broken("shard-key-position-folded", environment):
        position = f" {index}/{count}"
    elif broken("shard-key-count-kept", environment):
        position = f" 第{index}片共{count}片"
    else:
        position = f" 第{index}片"
    return f"{base_key_text} 〈task-server 分片 {role}{position}〉"


def read_shard_plan(path):
    """读计划（文件头「分片」①）：→ ([{"label"}…], 问题)；读得通问题是空串。写了 pin 的拒（片不钉台，哪台由服务挑）；弄坏开关 host-lists-restored 下照改前收
    pin（空串与 local，读了不用）。"""
    restored = broken("host-lists-restored")
    try:
        with open(path, encoding="utf-8") as handle:
            value = json.load(handle)
    except FileNotFoundError:
        return [], f"计划那一步退 0 而没写 {path}"
    except (OSError, ValueError) as error:
        return [], f"{path} 读不通（{type(error).__name__}: {error}）"
    shards = value.get("shards") if isinstance(value, dict) else None
    if not isinstance(shards, list):
        return [], f"{path} 里没有 shards 列表"
    if len(shards) > MAXIMUM_SHARDS:
        return [], f"{path} 写了 {len(shards)} 片，多于 {MAXIMUM_SHARDS} 片"
    parsed = []
    for number, shard in enumerate(shards):
        if isinstance(shard, dict) and "pin" in shard and not restored:
            return [], f"{path} 第 {number} 片写了 pin（{shard['pin']!r}）：片不钉台，哪台由服务挑；计划里去掉 pin"
        if restored and isinstance(shard, dict) and (shard.get("pin") or "") in ("", "local"):
            shard = {name: value for name, value in shard.items() if name != "pin"}
        if not isinstance(shard, dict) or set(shard) - set(SHARD_PLAN_KEYS):
            return [], f"{path} 第 {number} 片不是只带 {'、'.join(SHARD_PLAN_KEYS)} 的对象"
        if not isinstance(shard.get("label", ""), str):
            return [], f"{path} 第 {number} 片写坏了（label 是一句话）"
        parsed.append({"label": shard.get("label", "")})
    return parsed, ""


# ── 身份（文件头「身份」）──
def process_parent(process_id):
    try:
        with open(f"/proc/{process_id}/stat", encoding="utf-8") as handle:
            return int(handle.read().rsplit(")", 1)[1].split()[1])
    except (OSError, IndexError, ValueError):
        return 0


def process_environment(process_id):
    try:
        with open(f"/proc/{process_id}/environ", "rb") as handle:
            raw = handle.read()
    except OSError:
        return {}
    pairs = (item.split(b"=", 1) for item in raw.split(b"\0") if b"=" in item)
    return {name.decode("utf-8", "replace"): value.decode("utf-8", "replace") for name, value in pairs}


def session_of_peer(process_id, boundary_process_id=1, environment=None):
    """调用进程的会话号：从它的父进程起往上，各祖先 /proc 环境里的 CLAUDE_CODE_SESSION_ID 取最外层的那一个（不含调用进程自己；到 init 或 boundary 为止，不含它）。
    读不到交空串。弄坏开关 session-from-client 下取调用进程自己环境里的。"""
    if broken("session-from-client", environment):
        return process_environment(process_id).get(SESSION_VARIABLE, "")
    found, seen = "", set()
    current = process_parent(process_id)
    while current > 1 and current != boundary_process_id and current not in seen:
        seen.add(current)
        value = process_environment(current).get(SESSION_VARIABLE, "")
        if value:
            found = value
        current = process_parent(current)
    return found


# ── 经 unix socket 调服务 ──
class ServerUnreachable(Exception):
    pass


class UnixSocketConnection(http.client.HTTPConnection):
    def __init__(self, socket_path, timeout):
        super().__init__("task-server", timeout=timeout)
        self.socket_path = socket_path

    def connect(self):
        connection = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        connection.settimeout(self.timeout)
        connection.connect(self.socket_path)
        self.sock = connection


def call_server(socket_path, method, path, body=None, timeout=30.0):
    """→ (HTTP 状态码, 解开的 JSON)；连不上、读不完抛 ServerUnreachable。"""
    connection = UnixSocketConnection(socket_path, timeout)
    try:
        payload = None if body is None else json.dumps(body, ensure_ascii=False).encode("utf-8")
        connection.request(method, path, body=payload, headers={"Content-Type": "application/json"} if payload is not None else {})
        response = connection.getresponse()
        data = response.read()
        status = response.status
    except (OSError, http.client.HTTPException) as error:
        raise ServerUnreachable(f"{type(error).__name__}: {error}") from None
    finally:
        connection.close()
    try:
        decoded = json.loads(data.decode("utf-8")) if data else {}
    except ValueError:
        decoded = {"detail": data.decode("utf-8", "replace")[:500]}
    return status, decoded


def unreachable_error(configuration, reason):
    return Unavailable(f"task-server 不通（{configuration.socket}：{reason}）",
                       f"看服务在不在：systemctl --user status {UNIT_NAME}；没起就 {START_COMMAND}；没装就 {INSTALL_COMMAND}；"
                       "不退回直接执行命令，服务好了再提交")


def require_server(configuration, timeout=10.0):
    """服务通不通：/health 回 ok 交它的 JSON，不通抛 Unavailable。"""
    try:
        status, health = call_server(configuration.socket, "GET", "/health", timeout=timeout)
    except ServerUnreachable as error:
        raise unreachable_error(configuration, str(error)) from None
    if status != 200 or not health.get("ok"):
        raise Unavailable(f"task-server 回话了但不健康（HTTP {status}：{health.get('problem') or health}）",
                          f"看服务的日志：journalctl --user -u {UNIT_NAME} -n 100；修好之后 python3 task-server/control check 回读到退 0")
    return health


# ── 错误类型与重试（纯函数）──
def classify_attempt(exit_code, host, log_text, cancelled, environment=None):
    """一次尝试的错误类型（文件头「错误类型」）。exit_code 是 None：单元不在了又没留结果。"""
    if cancelled:
        return "cancelled"
    if exit_code is None:
        return "launcher-lost"
    if exit_code == 255 and host == "peer" and not broken("peer-exit-unsplit", environment):
        return "peer-run-failed"
    if exit_code == 75 and broken("prune-server-exit-unsplit", environment):
        return "command-failed"   # 弄坏开关：改前的样子，75 不单列
    if exit_code in EXIT_TYPES:
        return EXIT_TYPES[exit_code]
    if broken("unlisted-exit-swallowed", environment):
        return "succeeded"
    return "command-failed"


def memory_cgroup_kill_line(line, environment=None):
    """这一行是内核在某个内存组到了它自己的上限时杀进程的记录（带 MEMORY_CGROUP_KILL_TEXT）：不算被外部信号杀掉。弄坏开关 memory-cgroup-kill-counted 下一律交 False。"""
    return MEMORY_CGROUP_KILL_TEXT in line and not broken("memory-cgroup-kill-counted", environment)


def external_kill_pid(line, environment=None):
    """journalctl 里一行是 earlyoom（「sending SIGTERM to process <号> …」）或整机内存不够时内核 OOM（「Killed process <号> (…」）杀进程的记录，交被杀的进程号；
    不是这两种行、或是内存组到了它自己上限的那一种（memory_cgroup_kill_line）交 None。"""
    if memory_cgroup_kill_line(line, environment):
        return None
    found = EXTERNAL_KILL_LINE.search(line)
    return int(found.group(1) or found.group(2)) if found else None


# ── 读不到单元状态时不经 systemd 核进程（task_server.py 文件头 ①「读不到单元状态」）──
def attempt_presence_readings(request):
    """不经 systemd 核一次尝试在这一台上还有没有进程：只读 /proc 与 cgroupfs，不起 systemctl、不发信号（第二台经 attempt_presence_probe_text 把这一段送过去跑，两台同一段）。
    request：process_ids（记下的进程号）、process_id_files（存着一个进程号的文件：第二台 multi-host-run.sh 记的 runs/<标签>.pid；文件不在算没起过那一层）、
    cgroup_directories（记下的 cgroup 目录：attempt_runner 写的 scope 文件里那一个）、cgroup_names（照名字在这一台用户级管理器的 cgroup 树里找的目录：尝试的单元）、
    cgroup_name_files（存着一个 cgroup 名字的文件，读出的名字接上 cgroup_name_suffix 照 cgroup_names 找：第二台 multi-host-run.sh 记的 runs/<标签>.scope）、
    label_entry（「<尝试标签变量>=<标签>」：树里名字以 label_scope_prefix 起头、以 .scope 收尾的 cgroup，里面有进程的环境带这一项的也算这一次的）、
    tree_depth（在树里往下找几层）、cgroup_root、proc_root（不给是 /sys/fs/cgroup、/proc）、absent_cgroup_unconfirmed（只给弄坏开关：cgroup 目录不在不算空、算读不了）。
    用户级管理器的 cgroup 树是 <cgroup_root>/user.slice/user-<uid>.slice/user@<uid>.service（uid 是起这一段的那个用户的）。
    交回 {"processes": {进程号: 在不在（僵尸算不在；进程表列不出是 None）}, "cgroups": {目录或名字: [进程号…]（目录不在而往上一层列得出算空）| None（读不了）},
    "files": {文件: 读到的内容（不在是空串）}, "problems": [核不动的那几句]}。"""
    import os as os_module

    proc_root = request.get("proc_root") or "/proc"
    cgroup_root = (request.get("cgroup_root") or "/sys/fs/cgroup").rstrip("/")
    absent_unconfirmed = bool(request.get("absent_cgroup_unconfirmed"))
    label_entry = (request.get("label_entry") or "").encode("utf-8")
    prefix = request.get("label_scope_prefix") or ""
    readings = {"processes": {}, "cgroups": {}, "files": {}, "problems": []}

    def read_small_file(path):
        """→ 去掉首尾空白的内容；文件不在而所在目录列得出交空串（没写过），读不了交 None、记一句。"""
        try:
            with open(path, encoding="utf-8", errors="replace") as handle:
                return handle.read().strip()
        except FileNotFoundError:
            try:
                os_module.listdir(os_module.path.dirname(path) or ".")
            except OSError as error:
                readings["problems"].append(f"{path} 不在，所在的目录也列不出（{type(error).__name__}）")
                return None
            return ""
        except OSError as error:
            readings["problems"].append(f"{path} 读不了（{type(error).__name__}）")
            return None

    def members(directory):
        """cgroup 目录此刻的进程号；目录不在而往上第一层在的目录列得出交 []（cgroup 已收掉），读不了交 None、记一句。"""
        try:
            with open(os_module.path.join(directory, "cgroup.procs"), encoding="utf-8") as handle:
                return handle.read().split()
        except FileNotFoundError:
            pass
        except OSError as error:
            readings["problems"].append(f"{directory}/cgroup.procs 读不了（{type(error).__name__}）")
            return None
        if os_module.path.isdir(directory):
            readings["problems"].append(f"{directory} 在而没有 cgroup.procs")
            return None
        if absent_unconfirmed:
            readings["problems"].append(f"{directory} 不在（弄坏开关 unit-unreadable-absent-cgroup-unconfirmed：不算空）")
            return None
        ancestor = os_module.path.dirname(directory.rstrip("/"))
        while ancestor == cgroup_root or ancestor.startswith(cgroup_root + "/"):
            if os_module.path.isdir(ancestor):
                try:
                    os_module.listdir(ancestor)
                except OSError as error:
                    readings["problems"].append(f"{directory} 不在，上一层 {ancestor} 列不出（{type(error).__name__}）")
                    return None
                return []
            ancestor = os_module.path.dirname(ancestor)
        readings["problems"].append(f"{directory} 不在，往上到 cgroup 根 {cgroup_root} 都不在")
        return None

    def labelled(processes):
        """→ (有一个进程的环境带 label_entry, 有读不了环境的)；读的时候刚退出的跳过。"""
        unreadable = False
        for process in processes:
            try:
                with open(os_module.path.join(proc_root, process, "environ"), "rb") as handle:
                    if label_entry in handle.read().split(b"\0"):
                        return True, False
            except FileNotFoundError:
                continue
            except OSError:
                unreadable = True
        return False, unreadable

    try:
        os_module.listdir(proc_root)
        proc_readable = True
    except OSError as error:
        proc_readable = False
        readings["problems"].append(f"进程表 {proc_root} 列不出（{type(error).__name__}）")
    process_ids = [str(process) for process in request.get("process_ids") or []]
    for path in request.get("process_id_files") or []:
        text = read_small_file(path)
        if text is None:
            continue
        readings["files"][path] = text
        if text.isdigit():
            process_ids.append(text)
        elif text:
            readings["problems"].append(f"{path} 里不是进程号（{text[:40]}）")
    for process in process_ids:
        if not proc_readable:
            readings["processes"][process] = None
            continue
        try:
            with open(os_module.path.join(proc_root, process, "stat"), encoding="utf-8", errors="replace") as handle:
                readings["processes"][process] = handle.read().rpartition(")")[2].split()[:1] != ["Z"]
        except FileNotFoundError:
            readings["processes"][process] = False
        except OSError:
            readings["processes"][process] = True   # 目录在、stat 读不了：当在
    names = list(request.get("cgroup_names") or [])
    for path in request.get("cgroup_name_files") or []:
        text = read_small_file(path)
        if text is None:
            continue
        readings["files"][path] = text
        if text:
            names.append(text + (request.get("cgroup_name_suffix") or ""))
    directories = list(request.get("cgroup_directories") or [])
    if names or (label_entry and prefix):
        uid = os_module.getuid()
        manager = f"{cgroup_root}/user.slice/user-{uid}.slice/user@{uid}.service"
        found, candidates, errors = {}, [], []

        def walk_error(error):
            if not isinstance(error, FileNotFoundError):   # 走的时候刚收掉的目录不算读不了
                errors.append(f"{getattr(error, 'filename', '') or manager} 列不出（{type(error).__name__}）")

        if not os_module.path.isdir(manager):
            errors.append(f"用户级管理器的 cgroup 目录 {manager} 不在")
        else:
            for current, children, _ in os_module.walk(manager, onerror=walk_error):
                for name in children:
                    if name in names:
                        found.setdefault(name, os_module.path.join(current, name))
                    elif label_entry and prefix and name.startswith(prefix) and name.endswith(".scope"):
                        candidates.append(os_module.path.join(current, name))
                if current[len(manager):].count("/") + 1 >= int(request.get("tree_depth") or 1):
                    children[:] = []
        readings["problems"] += errors
        for name in names:
            if errors:
                readings["cgroups"][name] = None
            elif name in found:
                directories.append(found[name])
            elif absent_unconfirmed:
                readings["problems"].append(f"cgroup {name} 在 {manager} 下找不到（弄坏开关 unit-unreadable-absent-cgroup-unconfirmed：不算空）")
                readings["cgroups"][name] = None
            else:
                readings["cgroups"][name] = []   # 树走全了也没有：这个单元或 scope 的 cgroup 已收掉
        for path in candidates:
            processes = members(path)
            if processes is None:
                readings["cgroups"][path] = None
                continue
            ours, unreadable = labelled(processes) if processes else (False, False)
            if ours:
                readings["cgroups"][path] = processes
            elif unreadable:
                readings["problems"].append(f"{path} 里有进程的环境读不了，认不出是不是这一次的")
                readings["cgroups"][path] = None
    for directory in directories:
        readings["cgroups"][directory] = members(directory)
    return readings


ATTEMPT_PRESENCE_READINGS_SOURCE = inspect.getsource(attempt_presence_readings)   # 载入时取一次：运行时再按行号读磁盘上的源码，文件被改过就取到别的函数


def attempt_presence_probe_text(request):
    """在第二台上起 attempt_presence_readings 的那一条 bash 命令（经 peer-host-lib.sh 的 peer_host_run 起）：
    源码与请求各 base64 一份，读数打成一行 ATTEMPT_PRESENCE_PROBE_MARK 加 JSON。"""
    program = (ATTEMPT_PRESENCE_READINGS_SOURCE
               + "\nimport base64 as probe_base64, json as probe_json, sys as probe_sys\n"
               + "probe_request = probe_json.loads(probe_base64.b64decode(probe_sys.argv[1]).decode('utf-8'))\n"
               + f"print({ATTEMPT_PRESENCE_PROBE_MARK!r} + probe_json.dumps(attempt_presence_readings(probe_request)))\n")
    encoded = base64.b64encode(program.encode("utf-8")).decode("ascii")
    argument = base64.b64encode(json.dumps(request).encode("utf-8")).decode("ascii")
    return f'python3 -B -c "$(printf %s {encoded} | base64 -d)" {argument}'


def attempt_presence_probe_reply(stdout):
    """第二台那一趟的标准输出 → 读数；没有那一行或解不开交 None。"""
    line = next((line for line in reversed((stdout or "").splitlines()) if line.startswith(ATTEMPT_PRESENCE_PROBE_MARK)), None)
    if line is None:
        return None
    try:
        readings = json.loads(line[len(ATTEMPT_PRESENCE_PROBE_MARK):])
    except ValueError:
        return None
    return readings if isinstance(readings, dict) else None


# ── 日志不涨时的进展读数（task_server.py 文件头 ①「要处置的提醒」的 quiet-log-busy）──
def attempt_progress_readings(request):
    """一次尝试在这一台上此刻的进展读数：这一趟进程树里各进程打开写着的普通文件与大小。只读 /proc，不发信号
    （第二台经 attempt_progress_probe_text 把这一段送过去跑，两台同一段）。
    request：root_process_ids（进程树的根）、root_process_id_files（存着一个根进程号的文件：第二台 multi-host-run.sh 记的 runs/<标签>.pid）、
    excluded_files（不算的文件：尝试的顶层日志，它的大小里有起法那一层每轮打的进度行，服务另照 log_content 判）、proc_root（不给是 /proc）、
    file_limit、
    write_mode_unchecked（只给弄坏开关 open-file-write-mode-unchecked：只读打开的也算）。
    进程树照 <proc_root>/<号>/stat 的父进程号从根往下；写着的文件：<号>/fd/<fd> 指到的绝对路径（不是已删的），<号>/fdinfo/<fd> 的 flags 里访问方式是只写或读写，
    指到的是普通文件。→ {"files": {路径: 字节数}, "processes": 进程树里几个进程, "problems": [读不了的那几句]}。"""
    import os as os_module
    import stat as stat_module

    proc_root = request.get("proc_root") or "/proc"
    limit = int(request.get("file_limit") or 256)
    readings = {"files": {}, "processes": 0, "problems": []}
    excluded = set()
    for path in request.get("excluded_files") or []:
        excluded.add(os_module.path.realpath(path))
    roots = [int(process) for process in request.get("root_process_ids") or [] if str(process).isdigit()]
    for path in request.get("root_process_id_files") or []:
        try:
            with open(path, encoding="utf-8") as handle:
                text = handle.read().strip()
        except OSError as error:
            readings["problems"].append(f"{path} 读不了（{type(error).__name__}）")
            continue
        if text.isdigit():
            roots.append(int(text))
        elif text:
            readings["problems"].append(f"{path} 里不是进程号（{text[:40]}）")
    children = {}
    try:
        names = os_module.listdir(proc_root)
    except OSError as error:
        readings["problems"].append(f"进程表 {proc_root} 列不出（{type(error).__name__}）")
        names = []
    for name in names:
        if not name.isdigit():
            continue
        try:
            with open(os_module.path.join(proc_root, name, "stat"), encoding="utf-8", errors="replace") as handle:
                parent = int(handle.read().rsplit(")", 1)[1].split()[1])
        except (OSError, IndexError, ValueError):
            continue
        children.setdefault(parent, []).append(int(name))
    family, frontier = set(), [root for root in roots if os_module.path.isdir(os_module.path.join(proc_root, str(root)))]
    family.update(frontier)
    while frontier:
        for child in children.get(frontier.pop(), []):
            if child not in family:
                family.add(child)
                frontier.append(child)
    readings["processes"] = len(family)
    for process in sorted(family):
        if len(readings["files"]) >= limit:
            readings["problems"].append(f"写着的文件记满 {limit} 份，余下的进程不再读")
            break
        directory = os_module.path.join(proc_root, str(process), "fd")
        try:
            descriptors = os_module.listdir(directory)
        except OSError:
            continue   # 刚退出、或读不了别人的进程：这一个不算
        for descriptor in descriptors:
            try:
                target = os_module.readlink(os_module.path.join(directory, descriptor))
            except OSError:
                continue
            if not target.startswith("/") or target.endswith(" (deleted)"):
                continue
            if not request.get("write_mode_unchecked"):
                try:
                    with open(os_module.path.join(proc_root, str(process), "fdinfo", descriptor), encoding="utf-8") as handle:
                        flags = next((int(line.split()[1], 8) for line in handle if line.startswith("flags:")), None)
                except (OSError, ValueError, IndexError):
                    continue
                if flags is None or flags & 3 not in (1, 2):   # O_ACCMODE：1 只写、2 读写
                    continue
            real = os_module.path.realpath(target)
            if real in excluded or real in readings["files"]:
                continue
            try:
                found = os_module.stat(real)
            except OSError:
                continue
            if not stat_module.S_ISREG(found.st_mode):
                continue
            if len(readings["files"]) >= limit:
                break
            readings["files"][real] = found.st_size
    return readings


ATTEMPT_PROGRESS_READINGS_SOURCE = inspect.getsource(attempt_progress_readings)   # 载入时取一次（同 ATTEMPT_PRESENCE_READINGS_SOURCE）


def attempt_progress_probe_text(request):
    """在第二台上起 attempt_progress_readings 的那一条 bash 命令（经 peer-host-lib.sh 的 peer_host_run 起，同 attempt_presence_probe_text 的送法）：
    源码与请求各 base64 一份，读数打成一行 ATTEMPT_PROGRESS_PROBE_MARK 加 JSON。"""
    program = (ATTEMPT_PROGRESS_READINGS_SOURCE
               + "\nimport base64 as probe_base64, json as probe_json, sys as probe_sys\n"
               + "probe_request = probe_json.loads(probe_base64.b64decode(probe_sys.argv[1]).decode('utf-8'))\n"
               + f"print({ATTEMPT_PROGRESS_PROBE_MARK!r} + probe_json.dumps(attempt_progress_readings(probe_request)))\n")
    encoded = base64.b64encode(program.encode("utf-8")).decode("ascii")
    argument = base64.b64encode(json.dumps(request).encode("utf-8")).decode("ascii")
    return f'python3 -B -c "$(printf %s {encoded} | base64 -d)" {argument}'


def attempt_progress_probe_reply(stdout):
    """第二台那一趟的标准输出 → 读数；没有那一行或解不开交 None。"""
    line = next((line for line in reversed((stdout or "").splitlines()) if line.startswith(ATTEMPT_PROGRESS_PROBE_MARK)), None)
    if line is None:
        return None
    try:
        readings = json.loads(line[len(ATTEMPT_PROGRESS_PROBE_MARK):])
    except ValueError:
        return None
    return readings if isinstance(readings, dict) else None


def attempt_presence_verdict(parts):
    """一次尝试在各台的读数合起来判（task_server.py 文件头 ①「读不到单元状态」）。parts：[(台（HOST_NAMES 的键）, attempt_presence_readings 交的读数或 None, 读不成的那一句)]。
    → (PRESENCE_ALIVE：有一个记下的进程号在、或有一个 cgroup 里有进程；PRESENCE_GONE：都读得到、cgroup 都空或已收掉、记下的进程号都不在；
    PRESENCE_UNVERIFIED：没有在的、又有一样读不成或核不动, 说明：逐台写核到的进程号与 cgroup)。"""
    alive, unverified, pieces = False, False, []
    for host, readings, problem in parts:
        name = HOST_NAMES.get(host, host)
        if readings is None:
            unverified = True
            pieces.append(f"{name}：{problem or '读不成'}")
            continue
        facts = []
        for process, present in sorted((readings.get("processes") or {}).items()):
            if present is None:
                unverified = True
                facts.append(f"进程号 {process} 读不了")
            elif present:
                alive = True
                facts.append(f"进程号 {process} 在（/proc/{process}）")
            else:
                facts.append(f"进程号 {process} 不在")
        for group, processes in sorted((readings.get("cgroups") or {}).items()):
            if processes is None:
                unverified = True
                facts.append(f"cgroup {group} 读不了")
            elif processes:
                alive = True
                facts.append(f"cgroup {group} 里 {len(processes)} 个进程（{' '.join(processes[:5])}{' …' if len(processes) > 5 else ''}）")
            else:
                facts.append(f"cgroup {group} 空或已收掉")
        if readings.get("problems"):
            unverified = True
            facts.append("核不动：" + "；".join(str(line) for line in readings["problems"][:3]))
        if readings.get("note"):
            facts.append(str(readings["note"]))
        pieces.append(f"{name}：{'，'.join(facts) or '没有要核的'}")
    verdict = PRESENCE_ALIVE if alive else PRESENCE_UNVERIFIED if unverified else PRESENCE_GONE
    return verdict, "；".join(pieces)


def peer_command_exit(log_text, environment=None):
    """第二台那条命令自己的退出码：日志里最后一行「MULTI_HOST_RUN_STEP step=peer-command event=end … exit=<码>」（multi-host-run.sh 在第二台那条命令
    等到结局时打）的码；没有这一行交 None（multi-host-run.sh 那一层没跑到命令结束）。弄坏开关 peer-step-exit-ignored 下一律 None（改前的样子）。"""
    if broken("peer-step-exit-ignored", environment):
        return None
    found = None
    for line in (log_text or "").splitlines():
        matched = PEER_COMMAND_END_LINE.match(line.strip())
        if matched:
            found = int(matched.group(1))
    return found


def peer_marked_bytes(log_text, mark):
    """日志里最后一行「<mark> bytes=<十进制字节数>」的字节数（multi-host-run.sh 打的第二台那一层的峰值）；没有交 None。"""
    pattern = re.compile(rf"^{re.escape(mark)} bytes=([0-9]+)$")
    found = None
    for line in (log_text or "").splitlines():
        matched = pattern.match(line.strip())
        if matched:
            found = int(matched.group(1))
    return found


def peer_kept_build_label(log_text):
    """日志里最后一行「PEER_BUILD_KEPT_MARK label=<标签>」的标签（multi-host-run.sh 留下了第二台这一趟的编译目录）；没有交 None。"""
    pattern = re.compile(rf"^{re.escape(PEER_BUILD_KEPT_MARK)} label=([A-Za-z0-9][A-Za-z0-9._-]*)$")
    found = None
    for line in (log_text or "").splitlines():
        matched = pattern.match(line.strip())
        if matched:
            found = matched.group(1)
    return found


def final_exit_code(outcome, attempt_exit):
    code, _ = OUTCOME_TYPES[outcome]
    if code is not None:
        return code
    return attempt_exit if isinstance(attempt_exit, int) and 0 < attempt_exit < 256 else 1


def grown_cap(cap_bytes):
    return math.ceil(cap_bytes * CAP_GROWTH_FACTOR / MEBIBYTE) * MEBIBYTE


def mebibytes_text(byte_count):
    return f"{math.ceil(byte_count / MEBIBYTE)}M"


def maximum_next_step(host, cap_text, peak_text, environment=None):
    """memory-cap-at-maximum 的出路（文件头「撞顶到两台最大」）：撞了哪台、到了多少、峰值多少，后面接异常表里这一种的修改意见（核实范围、分片与同时起的个数）。
    wait 的末行、打回的理由与发给会话的正文都用这一句。"""
    where = HOST_NAMES.get(host, host or "本机")
    return (f"撞到动态上限、打回：{where}已给到 {cap_text}（峰值 {peak_text}），再乘 1.5 就到能派的每一台的可用上限；"
            + anomaly_advice("memory-cap-at-maximum", "", environment))


def pinned_caps_text(reply):
    """钉值回话（task_server.py 的 pin_cap 交的那一份）里两台此刻的钉值：「本机 20480M、第二台 30720M」；读不到写「此刻读不到（原因）」（task status 与 control pin-cap 打）。"""
    caps = reply.get("bytes_by_host")
    if isinstance(caps, dict) and caps and all(isinstance(value, int) for value in caps.values()):
        shown = "、".join(f"{HOST_NAMES.get(host, host)} {mebibytes_text(value)}" for host, value in sorted(caps.items()))
        return f"{shown}；{reply['problem']}" if reply.get("problem") else shown   # 哪一台量不到余量：给得出的照给，量不到的写原因（task_server.py 的 pin_value）
    return f"此刻读不到（{reply.get('problem')}）"


def size_text(byte_count):
    if not isinstance(byte_count, int):
        return "没量到"
    return f"{byte_count / GIBIBYTE:.2f} GiB" if byte_count >= GIBIBYTE else f"{byte_count / MEBIBYTE:.0f} MiB"


def log_lines(log_text):
    """日志里人读的那几行（去掉空行与 REASON_LOG_NOISE 那几种进度行）。"""
    return [line.strip() for line in (log_text or "").splitlines() if line.strip() and not line.strip().startswith(REASON_LOG_NOISE)]


def log_last_line(log_text, fallback="日志里没有输出"):
    lines = log_lines(log_text)
    return lines[-1][:300] if lines else fallback


def trailing_cap_hits(attempts):
    """尝试记录里末尾连着结局是 memory-cap-hit 的那几次（文件头「打回的理由」的 loop_leak=、「异常表」repeated-cap-hits）→ [尝试…]，照第几次从小到大。
    只数已经收尾的（state 是 ended）；在跑、在起的那一次不数、也不断开；中间夹着别的结局（被挤掉、第二台没跑成这类）的从那之后数起。"""
    ended = sorted((attempt for attempt in attempts if attempt.get("state") == "ended"), key=lambda attempt: int(attempt.get("number") or 0))
    run = []
    for attempt in ended:
        run = run + [attempt] if attempt.get("outcome") == "memory-cap-hit" else []
    return run


LOOP_LEAK_READING_MISMATCH = "峰值读数与撞上限对不上，先当读数不准，不要据此判泄漏"


def loop_leak_check_text(cap_hits):
    """「检查代码有没有逻辑循环或内存泄漏」那一段（文件头「打回的理由」）：cap_hits 是 trailing_cap_hits 交的那几次（至少一次）。
    写要查的两类（LOOP_LEAK_KINDS），再写依据：各次的上限与峰值逐次列出、峰值的走势（严格变大、全部相等，或都不是）、每次是不是都顶到上限（峰值不低于上限的 STALL_AT_CAP_FRACTION）、
    最后一次内存最高那一刻的明细（task_server.py 的 finish_attempt 记的 peak_memory_note）；每一次都没顶到的另写 LOOP_LEAK_READING_MISMATCH。理由行的 loop_leak=、结局正文与 repeated-cap-hits 的提醒都用这一句。
    弄坏开关 loop-leak-equal-peaks-growing 下相等也写一次比一次大，loop-leak-reading-mismatch-silent 下不写那一句。"""
    shown = "、".join(f"第 {attempt.get('number')} 次上限 {attempt.get('memory_cap_text') or '没记'} 峰值 {size_text(attempt.get('memory_peak'))}" for attempt in cap_hits)
    peaks = [attempt.get("memory_peak") for attempt in cap_hits]
    if not all(isinstance(peak, int) and peak > 0 for peak in peaks):
        trend = "有几次峰值没量到，比不出是不是一次比一次大"
    elif all(later >= earlier if broken("loop-leak-equal-peaks-growing") else later > earlier for earlier, later in zip(peaks, peaks[1:])):
        trend = "峰值一次比一次大"
    elif all(later == earlier for earlier, later in zip(peaks, peaks[1:])):
        trend = "峰值没变"
    else:
        trend = "峰值不是一次比一次大"
    short = [str(attempt.get("number")) for attempt in cap_hits
             if not (isinstance(attempt.get("memory_peak"), int) and isinstance(attempt.get("memory_cap"), int) and attempt["memory_cap"] > 0
                                                 and attempt["memory_peak"] >= STALL_AT_CAP_FRACTION * attempt["memory_cap"])]
    topped = (f"每次都顶到上限（峰值不低于上限的 {STALL_AT_CAP_FRACTION:.0%}）" if not short
              else f"第 {'、'.join(short)} 次没顶到上限的 {STALL_AT_CAP_FRACTION:.0%}（或峰值没量到）")
    stat = cap_hits[-1].get("peak_memory_note") or "最后一次内存最高那一刻的明细没采到（命令比一轮还短，或读不了 scope 的 memory.current、memory.stat）"
    mismatch = f"；{LOOP_LEAK_READING_MISMATCH}" if len(short) == len(cap_hits) and not broken("loop-leak-reading-mismatch-silent") else ""
    return f"{LOOP_LEAK_CHECK_HEAD}连着撞了 {len(cap_hits)} 次内存上限，要查两类：{LOOP_LEAK_KINDS}。依据：{shown}；{trend}；{topped}；{stat}{mismatch}"


def reason_fields(task, attempts, log_text):
    """打回的理由（文件头「错误类型」表后的「打回的理由」）：→ [(栏, 值)]，type 在头、why 与 next 在尾，中间是 REASON_ELEMENTS 给这一种的要素。
    attempts 是结局所在的那几次尝试（分片的是没成那一步的子任务的），log_text 是最后那一次日志的尾巴。"""
    outcome = task.get("outcome") or ""
    task_id = task.get("id")
    note = (task.get("note") or "").strip()
    last = attempts[-1] if attempts else {}
    facts = {
        "exit": str(task.get("exit_code")),
        "host": HOST_NAMES.get(last.get("host") or task.get("maximum_host") or "", last.get("host") or "本机"),
        "last_line": log_last_line(log_text),
        "cap": last.get("memory_cap_text") or "没起",
        "peak": size_text(last.get("memory_peak")) if attempts else "没量到",
        "reached": task.get("maximum_cap_text") or last.get("memory_cap_text") or "?",
        "by": last.get("squeezed_by") or "同台没有别的任务在跑（slice 里回收不掉的用量把它挤掉）",
        "unit": last.get("unit") or "没起单元",
        "judged": note or "没有说明",
        "kill": last.get("external_kill_note") or "没有说明",
        "rule": note or "没有说明",
        "what": note or "任务记录",
    }
    if outcome == "cancelled":   # 谁撤的照子任务的 cancel_origin 写（CANCEL_ORIGIN_TEXTS；弄坏开关 cancel-origin-unread 下一律写提交它的会话）
        origin = "" if broken("cancel-origin-unread") else str(task.get("cancel_origin") or "")
        facts["by"] = CANCEL_ORIGIN_TEXTS.get(origin, CANCEL_ORIGIN_TEXTS[""])
    counts = task.get("outcome_counts") or {}
    whys = {
        "succeeded": "命令退 0",
        "command-failed": f"命令自己退 {facts['exit']}（不是服务判的；日志最后一行见 last_line）" + (f"；{last['signal_note']}" if last.get("signal_note") else ""),
        "memory-cap-hit": f"撞了这一次的内存上限 {facts['cap']}（量到的峰值 {facts['peak']}），撞了 {counts.get('memory-cap-hit', 1)} 次" + (f"：{note}" if note else ""),
        "memory-cap-at-maximum": ((f"{facts['host']}给到 {facts['reached']} 还撞（峰值 {facts['peak']}），再乘 1.5 就到能派的每台的总上限" + (f"：{note}" if note else ""))
                                  if attempts else (note or "撞上限而再乘 1.5 就到能派的每台的总上限")),
        "squeezed-out": f"{facts['host']}的 slice 总上限满了，这一条（上限 {facts['cap']}）被内核挤掉 {counts.get('squeezed-out', 1)} 次；同台在跑的：{facts['by']}",
        "timed-out": f"内存包装的限时到了（253，上限 {facts['cap']}）",
        "wrapper-refused": "内存包装用法错或 scope 起不来（251）",
        "run-condition-unmet": note or f"命令退 78（运行条件不满足）：{facts['last_line']}",
        "not-run": "命令报本次未跑（77）",
        "peer-run-failed": "第二台那一趟没跑成（multi-host-run.sh 退 255、日志里没有第二台那条命令的结束行：连不上、ssh 被重置或那边的准备没过），重排过也没成",
        "prune-server-unavailable": "命令自己退 75：prune-server（判定库服务）不可用；服务另起了一次 python3 prune-server/control check，不通的写了告警",
        "launcher-lost": f"尝试的单元 {facts['unit']} 不在了、又没留结果，服务对不回来",
        "cancelled": note or "提交它的会话取消、单元已停",
        "record-tampered": f"库里这条记录校验不过（{facts['what']}），一次都不起",
        "never-fits": f"哪台都等多久也放不下，或点名第二台而它用不了：{facts['judged']}",
        "unit-start-failed": f"尝试的单元起不来：{facts['judged']}",
        "resubmit-required": f"服务规则改了，还在排的这一条照现行提交规则不再收：{facts['rule']}",
        "stalled": f"卡住，服务停了它：{facts['judged']}",
        "shard-plan-refused": f"分片的计划那一步退 0 而它写的计划读不通：{facts['judged']}",
        "over-four-hours": f"到了四小时硬上限，服务打回：{facts['judged']}",
        "killed-externally": f"命令被不是服务发的信号杀掉 {counts.get(EXTERNAL_KILL_OUTCOME, 1)} 次（{facts['kill']}），重排过也没成",
    }
    why = whys.get(outcome, note or f"结局 {outcome}")
    # 出路照异常表这一种那一行的修改意见（文件头「异常表」）；memory-cap-at-maximum 前面另接撞了哪台、到了多少（maximum_next_step）
    if outcome == "succeeded":
        next_step = "无"
    elif outcome == "memory-cap-at-maximum":
        next_step = maximum_next_step(task.get("maximum_host") or last.get("host") or "", facts["reached"], facts["peak"])
    elif outcome == "launcher-lost":
        next_step = anomaly_advice(outcome, task_id).replace("<unit= 点名的单元>", facts["unit"])
    else:
        next_step = anomaly_advice(outcome, task_id)
    # 连着撞上限之后打回的另带 loop_leak=（文件头「打回的理由」）：末尾连着撞上限不少于 LOOP_LEAK_BOUNCE_CAP_HITS 次才带，排队那一刻就打回的（一次都没起）不带
    cap_hits = trailing_cap_hits(attempts) if outcome in LOOP_LEAK_OUTCOMES and not broken("loop-leak-check-omitted") else []
    loop_leak = [("loop_leak", loop_leak_check_text(cap_hits))] if len(cap_hits) >= LOOP_LEAK_BOUNCE_CAP_HITS else []
    return [("type", outcome)] + [(name, facts[name]) for name in REASON_ELEMENTS.get(outcome, ())] + loop_leak + [("why", why), ("next", next_step)]


def reason_line(task, attempts, log_text):
    """REASON_MARK 那一行（值照 shlex 引，shlex.split 拆得回）。"""
    return " ".join([REASON_MARK, f"task={task.get('id')}"] + [f"{name}={shlex.quote(str(value))}" for name, value in reason_fields(task, attempts, log_text)])


# ── 一条任务的回话（GET /tasks/<号> 与 wait 的 summary）照 wait 的写法拼成几行：客户端打它，服务发消息给会话（task_server.py 文件头 ⑥）照同一份拼 ──
LOG_REASON_BYTES = 65536   # 打回的理由取日志最后这么多字节（与 task_server.py 的 LOG_TAIL_BYTES 同一个数）
ROLE_NAMES = {"plan": "计划", "merge": "合并"}


def child_prefix(child_task):
    role = child_task.get("role")
    if role == "shard":
        return f"第 {child_task.get('shard_index')}/{child_task.get('shard_count')} 片（子任务 {child_task.get('id')}）"
    return f"{ROLE_NAMES.get(role, role)}（子任务 {child_task.get('id')}）"


def reply_attempts(reply):
    """[(前缀, 尝试)]：普通任务是它自己的；分片的父任务是各子任务的，按子任务号。"""
    if reply.get("children"):
        return [(child_prefix(child["task"]), attempt) for child in reply["children"] for attempt in child.get("attempts", [])]
    return [("", attempt) for attempt in reply.get("attempts", [])]


def outcome_attempts(reply):
    """结局所在的那几次尝试：分片的有一步没成的是那一步的子任务的，别的是全部（末行的 log= 与打回的理由取同一份）。"""
    failed = reply["task"].get("failed_step") or {}
    attempts = [attempt for _, attempt in reply_attempts(reply)]
    if failed and reply.get("children"):
        attempts = next((child.get("attempts", []) for child in reply["children"] if child["task"].get("id") == failed.get("task")), attempts)
    return attempts


def log_tail_text(path):
    """日志最后 LOG_REASON_BYTES 字节（读不了交空串）。"""
    if not path:
        return ""
    try:
        with open(path, "rb") as handle:
            handle.seek(0, os.SEEK_END)
            handle.seek(max(0, handle.tell() - LOG_REASON_BYTES))
            return handle.read().decode("utf-8", "replace")
    except OSError:
        return ""


def reply_reason_fields(reply):
    """打回的理由那几栏（reason_fields）：日志取结局那一次的最后 LOG_REASON_BYTES 字节。"""
    attempts = outcome_attempts(reply)
    return reason_fields(reply["task"], attempts, log_tail_text(attempts[-1].get("log")) if attempts else "")


def reply_reason_line(reply):
    """末行之前那一行 REASON_MARK（task 文件头 wait 那一段）。"""
    return " ".join([REASON_MARK, f"task={reply['task'].get('id')}"] + [f"{name}={shlex.quote(str(value))}" for name, value in reply_reason_fields(reply)])


def reply_result_line(reply, environment=None):
    """末行 RESULT_MARK（task 文件头 wait 那一段）。弄坏开关 maximum-line-vague 下撞顶的不写撞了哪台、到了多少、改什么。"""
    task = reply["task"]
    pairs = reply_attempts(reply)
    failed = task.get("failed_step") or {}
    log_attempts = outcome_attempts(reply)
    log = log_attempts[-1].get("log", "") if log_attempts else ""
    line = (f"{RESULT_MARK} task={task.get('id')} type={task.get('outcome')} exit={task.get('exit_code')} attempts={len(pairs)} "
            f"log={log or '无（一次都没起）'}")
    if task.get("sharded_entry"):
        line += f" shards={task.get('shard_count', 0)}"
    if failed.get("role") == "shard":
        line += f" shard={failed.get('index')}/{failed.get('count')}"
    elif failed.get("role") in ("plan", "merge"):
        line += f" step={failed['role']}"
    build = next((attempt.get("build_directory") for _, attempt in reversed(pairs) if attempt.get("build_directory") and not attempt.get("build_directory_removed")), "")
    if build:   # 编译目录留给提交它的 agent：用完 task-server/task clean <号> 删（文件头「编译目录」）
        line += f" build={build}"
    if task.get("outcome") == "memory-cap-at-maximum" and not broken("maximum-line-vague", environment):
        host = failed.get("maximum_host") or task.get("maximum_host", "")
        cap_text = failed.get("maximum_cap_text") or task.get("maximum_cap_text", "")
        peak_text = size_text(log_attempts[-1].get("memory_peak")) if log_attempts else "没量到"
        line += f" host={HOST_NAMES.get(host, host or '本机')} reached={cap_text or '?'} next={maximum_next_step(host, cap_text or '?', peak_text)}"
    return line


def suspect_line(task_id, suspect):
    """报疑似卡住的那一行 SUSPECT_MARK（task 文件头 wait 那一段）。"""
    return (f"{SUSPECT_MARK} task={task_id} attempt={suspect.get('attempt')} host={HOST_NAMES.get(suspect.get('host'), suspect.get('host') or '本机')} "
            f"window={suspect.get('window_seconds')} cores={suspect.get('cores')} log={suspect.get('log') or '无'} why={suspect.get('why')}")


def attention_line(task_id, item):
    """报要处置的提醒的那一行 ATTENTION_MARK（task 文件头 wait 那一段）：值照 shlex 引（同 REASON_MARK，shlex.split 拆得回）；part= 只在提醒说的是分片的子任务时写它的任务号；
    带 loop_leak 的那一种（repeated-cap-hits，文件头「错误类型」）在 basis= 之后另写 loop_leak=；时长判的那几种（项里有 reminded_after_seconds）在 kind= 之后写 reminded_after_seconds=（这一回跨过的那道提醒门槛，秒）。why= 超过 ATTENTION_WHY_LIMIT 个字符的按字符截到那么长、尾巴接「…」
    （全文在发给会话的那一条的「原因：」一行）；弄坏开关 attention-why-unclipped 下不截。"""
    part = f" part={item.get('task')}" if item.get("task") not in (None, task_id) else ""
    values = [(name, item.get(name)) for name in ("basis", "loop_leak", "why", "next") if name != "loop_leak" or item.get(name)]
    why = str(item.get("why"))
    if len(why) > ATTENTION_WHY_LIMIT and not broken("attention-why-unclipped"):
        values = [(name, why[:ATTENTION_WHY_LIMIT - 1] + "…" if name == "why" else value) for name, value in values]
    after = f" reminded_after_seconds={item['reminded_after_seconds']}" if item.get("reminded_after_seconds") is not None else ""
    return (f"{ATTENTION_MARK} task={task_id}{part} type={ATTENTION_TYPE} kind={item.get('kind')}{after} "
            + " ".join(f"{name}={shlex.quote(str(value))}" for name, value in values))


@dataclass(frozen=True)
class AnomalyRow:
    kind: str
    action: str
    criterion: str
    advice: str
    unhandled: str


def anomaly_rows(text):
    """异常表那一段（text 里从 ANOMALY_TABLE_HEAD 起头的那一行起、到下一个空行为止）逐行读成 [AnomalyRow]，次序照表；
    行首两个空格、照「｜」切出五栏的才算一行，栏的前后不留空白。没有这一段交 []。"""
    rows, inside = [], False
    for line in (text or "").splitlines():
        if not inside:
            inside = line.startswith(ANOMALY_TABLE_HEAD)
            continue
        if not line.strip():
            break
        found = ANOMALY_ROW.match(line)
        if found:
            rows.append(AnomalyRow(*(part.strip() for part in found.groups())))
    return rows


def anomaly_table():
    """{种类: AnomalyRow}：ANOMALY_TABLE_TEXT 那张表（anomaly_rows 读，读过一次就留着）。"""
    if not ANOMALY_TABLE_LOADED:
        ANOMALY_TABLE_LOADED.append({row.kind: row for row in anomaly_rows(ANOMALY_TABLE_TEXT)})
    return ANOMALY_TABLE_LOADED[0]


RETIRED_OUTCOME_KINDS = ("over-two-hours",)   # 库里的旧任务还记着、服务不再判出的结局：表里没有这一行，修改意见交通用那一句

def anomaly_advice(kind, task, environment=None):
    """异常表里这一种的修改意见（next= 那一句的头一段），{task} 换成任务号；表里没有这一种抛 KeyError（种类与表对不上是代码错，不悄悄退回一句通用的）。
    弄坏开关 attention-next-generic 下每一种都交 ANOMALY_GENERIC_ADVICE；retired-outcome-kinds-unknown 下退役的结局照表里没有抛 KeyError；弄坏开关 bounce-new-id-reminder-omitted 下抹掉「不要使用原来的任务号，用 task-server/task submit 创建新的任务号」。"""
    retired = kind in RETIRED_OUTCOME_KINDS and not broken("retired-outcome-kinds-unknown", environment)
    template = ANOMALY_GENERIC_ADVICE if broken("attention-next-generic", environment) or retired else anomaly_table()[kind].advice
    text = template.replace(ANOMALY_TASK_PLACEHOLDER, str(task))
    if broken("bounce-new-id-reminder-omitted", environment):
        text = text.replace(f"（{BOUNCE_NEW_TASK_REMINDER}）", "").replace(BOUNCE_NEW_TASK_REMINDER, "")
    return text


PROCESS_LIST_COLUMNS = "pid=,ppid=,etimes=,times=,stat=,cgroup:300=,comm="   # 任务之外的进程空转读的进程表（task_server.py 的 process_listings）
TASK_SLICE_MARK = "singlefs.slice"   # cgroup 里带这一段的是任务 slice 里的进程


def parsed_process_list(listing):
    """PROCESS_LIST_COLUMNS 那几列的输出 → [{pid, ppid, elapsed, cpu, stat, cgroup, command}…]；列不够、数不对的行不要。"""
    rows = []
    for line in listing.splitlines():
        fields = line.split(None, 6)
        if len(fields) == 7 and all(field.isdigit() for field in fields[:4]):
            rows.append({"pid": int(fields[0]), "ppid": int(fields[1]), "elapsed": int(fields[2]), "cpu": int(fields[3]), "stat": fields[4], "cgroup": fields[5], "command": fields[6]})
    return rows


def process_parents(listing):
    """进程表 → {进程号: 父进程号}。"""
    return {row["pid"]: row["ppid"] for row in parsed_process_list(listing)}


def runaway_processes(listing, minimum_cpu_seconds, minimum_fraction, environment=None):
    """任务之外的进程空转的判据（task_server.py 文件头 ⑥）：不是内核线程、不在任务 slice 里、累计 CPU 满 minimum_cpu_seconds、累计 CPU 占已跑时长不低于 minimum_fraction。
    弄坏开关 runaway-slice-counted 下任务 slice 里的也算。"""
    found = []
    for row in parsed_process_list(listing):
        if row["pid"] == 2 or row["ppid"] == 2:
            continue
        if TASK_SLICE_MARK in row["cgroup"] and not broken("runaway-slice-counted", environment):
            continue
        if row["cpu"] >= minimum_cpu_seconds and row["cpu"] >= minimum_fraction * max(row["elapsed"], 1):
            found.append(row)
    return found


STOP_PROCESS_POLL_SECONDS = 0.1   # stop_process_here 回读 /proc 的间隔，同 stop_process_script 的 sleep 0.1
RUNAWAY_PROTECTED_COMMANDS = ("claude", "MainThread", "node", "code", "sshd", "systemd", "prune-server", "Xorg", "gnome-shell", "tmux: server")   # 认不出发起者也不碰的命令名（会话、编辑器、登录与桌面、判定库服务）


def runaway_unowned_stoppable(row, environment=None):
    """认不出发起者的空转进程能不能直接终止：只碰登录会话 scope 里的（systemd 单元与系统服务不碰），RUNAWAY_PROTECTED_COMMANDS 点名的不碰。"""
    return "/session-" in row["cgroup"] and row["cgroup"].endswith(".scope") and row["command"] not in RUNAWAY_PROTECTED_COMMANDS


def stop_process_script(pid, grace_seconds):
    """按进程号停的 shell（第二台经 ssh 跑；本机的同一个次序与判法是 stop_process_here）：先 TERM，过 grace_seconds 秒还在发 KILL，再等一会儿；退 0 是回读已不在（僵尸算不在）。"""
    number = int(pid)
    gone = f'gone() {{ s=$(sed -E "s/.*\\) (.).*/\\1/" /proc/{number}/stat 2>/dev/null); [ -z "$s" ] || [ "$s" = Z ]; }}'
    wait_gone = f"for i in $(seq 1 {int(grace_seconds) * 10}); do gone && exit 0; sleep 0.1; done"
    return f"{gone}; kill -TERM {number} 2>/dev/null; {wait_gone}; kill -KILL {number} 2>/dev/null; {wait_gone}; gone"


def process_gone_here(pid):
    """本机这个进程号回读已不在：/proc/<号>/stat 读不到，或状态字是 Z（僵尸算不在，同 stop_process_script 的 gone）。"""
    try:
        with open(f"/proc/{int(pid)}/stat", encoding="utf-8", errors="replace") as handle:
            return handle.read().rsplit(")", 1)[1].split()[0] == "Z"
    except (OSError, IndexError):
        return True


def stop_process_here(pid, grace_seconds):
    """本机按进程号停，与 stop_process_script（第二台跑的那一段）同一个次序与判法：先 TERM，过 grace_seconds 秒还在发 KILL，再等同样久；→ 回读已不在。发信号本身的错不算数，只看回读。"""
    number = int(pid)
    for signal_number in (signal.SIGTERM, signal.SIGKILL):
        with contextlib.suppress(OSError):
            os.kill(number, signal_number)
        deadline = time.monotonic() + grace_seconds
        while time.monotonic() < deadline:
            if process_gone_here(number):
                return True
            time.sleep(STOP_PROCESS_POLL_SECONDS)
    return process_gone_here(number)


COMMAND_CLASS_INTERPRETERS = ("bash", "sh", "python3", "python")


def command_classes(argv):
    """一条命令的同类（task_server.py 的 class_average_seconds）→ 从窄到宽的名字：同一脚本（cargo 是 cargo test 这样带子命令）、同一程序。开头的 env 与变量赋值不算；`bash -c` 这种没有脚本的交空表。"""
    words = list(argv)
    if words and os.path.basename(words[0]) == "env":
        words = words[1:]
    while words and ("=" in words[0] and not words[0].startswith("-") or words[0].startswith("-")):
        words = words[1:]
    if not words:
        return []
    program = os.path.basename(words[0])
    if program in COMMAND_CLASS_INTERPRETERS:
        if len(words) < 2 or words[1].startswith("-"):
            return []
        return [f"{program} {os.path.basename(words[1])}", program]
    if program == "cargo" and len(words) > 1:
        return [f"cargo {words[1]}", program]
    return [program]


def session_registry_library():
    """载 scripts/agents/lib_session_registry.py（会话登记的唯一取法）：交模块；载不进抛它的错（服务把这一回当找不到收件 socket）。"""
    if not SESSION_REGISTRY_LOADED:
        SESSION_REGISTRY_LOADED.append(load_module("lib_session_registry_for_task_server", SESSION_REGISTRY_SCRIPT))
    return SESSION_REGISTRY_LOADED[0]


def session_inbox_socket(session, environment=None):
    """按会话号找这个会话此刻的收件 socket（task_server.py 文件头 ⑥「收件地址」）→ (路径, 找不到时的那一句)：读会话登记目录
    （~/.claude/sessions，自证换成 SESSIONS_DIRECTORY_OVERRIDE 点名的）里会话号相同、进程还活着、带 messagingSocketPath 的那几份，取启动最晚的一份；
    读法与判活照 lib_session_registry.py（不另读 .key 文件，不另判别的字段）。没有会话（人手）、目录读不了、没有这样的一份，交 ("", 那一句)。"""
    if not session:
        return "", "这条任务没有会话（人手交的）"
    try:
        registry = session_registry_library()
        directory = selftest_override(SESSIONS_DIRECTORY_OVERRIDE, environment) or registry.registry_directory({})
        entries = registry.session_registry_entries(directory)
    except Exception as error:   # 载不进、读的时候抛错：这一回找不到
        return "", f"读会话登记出了错（{type(error).__name__}: {error}）"
    if entries is None:
        return "", f"会话登记目录 {directory} 读不了，或里面一份能解的登记都没有"
    alive = [entry for entry in entries if entry.session_id == session and entry.is_alive and entry.messaging_socket_path]
    if not alive:
        return "", f"会话登记目录 {directory} 里没有会话 {session} 进程还活着、带收件 socket 的那一份（会话已关）"
    return max(alive, key=registry.started_or_earliest).messaging_socket_path, ""


def confirm_command(task_id, items, tasks):
    """确认单打的确认命令（task_server.py 文件头 ⑦）：客户端照它打，钩子自证照它喂（.claude/hooks/lib_project_code_runs.py）。"""
    return f"task-server/task confirm {task_id} --items {items} --tasks {tasks}"


LONG_RUN_WHY_PLACEHOLDER = "<为什么要跑这么久、范围为什么是这些>"
LONG_RUN_WHY_MINIMUM_CHARACTERS = 12   # task confirm <号> --why 去掉空白至少这么多个字（task_server.py 文件头 ⑦「跑满两小时」）


def long_run_confirm_command(task_id, why=LONG_RUN_WHY_PLACEHOLDER):
    """跑满两小时提醒那一条「对」的出路整行（task_server.py 文件头 ⑦「跑满两小时」）：status 与拒绝说明照它打，自证照它喂。"""
    return f"task-server/task confirm {task_id} --why {shlex.quote(why)}"


def long_run_confirmation_problem(why, environment=None):
    """长跑确认的 --why（task confirm <号> --why '<…>'；task_server.py 文件头 ⑦「跑满两小时」）：去掉空白至少 LONG_RUN_WHY_MINIMUM_CHARACTERS 个字。
    → 不合的那一句，合的交空串。弄坏开关 long-run-short-why-accepted 下不判字数。"""
    if broken("long-run-short-why-accepted", environment):
        return ""
    stated = "".join((why or "").split())
    if len(stated) < LONG_RUN_WHY_MINIMUM_CHARACTERS:
        return f"--why 去掉空白只有 {len(stated)} 个字，至少 {LONG_RUN_WHY_MINIMUM_CHARACTERS} 个：写为什么要跑这么久、范围为什么是这些"
    return ""


def confirm_needed_line(task_id, items, tasks, timeout_seconds):
    """确认单末尾那一行 CONFIRM_NEEDED_MARK（机器读的；数是服务此刻算的）。"""
    return f"{CONFIRM_NEEDED_MARK} task={task_id} items={items} tasks={tasks} timeout={float(timeout_seconds):.0f}"


def confirm_needed_fields(text):
    """text 里最后一行 CONFIRM_NEEDED_MARK 的 (任务号, 项数, 条数)；没有交 None。"""
    found = list(CONFIRM_NEEDED_LINE.finditer(text or ""))
    return tuple(int(value) for value in found[-1].groups()) if found else None


def shell_inner_commands(command, environment=None):
    """命令是 bash / sh -c '<脚本>'（key_words 剥过前缀之后）时，脚本里按 && || ; | & 与括号切出的每一条命令（词表，重定向和它的目标去掉）→ [[词…]…]；不是 -c 的、引号不配对拆不开的交 []。
    确认单（named_scope、crash_scope_text、task_server.py 的 scope_is_whole）用它读 bash -c 里命令位置上的脚本与参数，不读被起脚本的正文。
    弄坏开关 shell-inner-scope-unread 下交 []（bash -c 里的范围一律认不出）。"""
    words = key_words(command, environment)
    index = command_word_index(words)
    if broken("shell-inner-scope-unread", environment) or index >= len(words) - 1 or words[index] != "-c":
        return []
    try:
        lexer = shlex.shlex(words[index + 1], posix=True, punctuation_chars=True)
        lexer.whitespace_split = True
        tokens = list(lexer)
    except ValueError:
        return []
    commands, current, skip_target = [], [], False
    for token in tokens:
        if skip_target:   # 重定向的目标不是命令的参数
            skip_target = False
            continue
        if token and set(token) <= set("&|;()"):
            commands.append(current)
            current = []
        elif token and set(token) <= set("<>&") and ("<" in token or ">" in token):
            skip_target = True
            if current and current[-1].isdigit():   # 2>&1 的 2 是文件描述符，不是参数
                current.pop()
        else:
            current.append(token)
    commands.append(current)
    return [inner for inner in commands if inner]


def named_scope(command, environment=None):
    """确认单的项数（task_server.py 文件头 ⑦）：命令（key_words 剥过不改跑法的前缀与临时 prune-server 包装）里点名的 --item、--check（值照逗号拆）、
    --test、--bin、--lib 与 `--` 之后带了 --exact 时的测试名（LIBTEST_VALUE_OPTIONS 的值不算）各几个 → [(选项, 值)…]，次序照命令；一个都没点的交 []。
    命令是 bash / sh -c 的，外层一个都没点时读脚本里每一条命令位置上点的（shell_inner_commands）。"""
    named = named_scope_words(command, environment)
    if not named:
        for inner in shell_inner_commands(command, environment):
            named += named_scope_words(inner, environment)
    return named


def named_scope_words(command, environment=None):
    """named_scope 对一条命令（不钻 bash -c）点名的项。"""
    words = key_words(command, environment)
    before, after = (words[:words.index("--")], words[words.index("--") + 1:]) if "--" in words else (words, [])
    named, position = [], 0
    while position < len(before):
        name, joined, value = before[position].partition("=")
        if name in NAMED_SCOPE_OPTIONS:
            if not joined:
                value = before[position + 1] if position + 1 < len(before) else ""
                position += 1
            named += [(name, piece) for piece in (value.split(",") if name == "--check" else [value]) if piece]
        elif before[position] == "--lib":
            named.append(("--lib", ""))
        position += 1
    if "--exact" in after:
        skip = False
        for word in after:
            if skip or word in LIBTEST_VALUE_OPTIONS:
                skip = not skip
                continue
            if not word.startswith("-"):
                named.append(("--exact", word))
    return named


def named_scope_text(named, whole, environment=None):
    """named_scope 交的那几项写成「--item 3、--test 1」（各选项照头一次出现的次序）；一个都没有的，whole 为真（task_server.py 的 scope_is_whole）写 WHOLE_SCOPE_TEXT，
    别的写 SCOPE_INSIDE_SCRIPT_TEXT。弄坏开关 scope-inside-script-unnamed 下一律写 WHOLE_SCOPE_TEXT。"""
    counts = {}
    for name, _ in named:
        counts[name] = counts.get(name, 0) + 1
    if counts:
        return "、".join(f"{name} {count}" for name, count in counts.items())
    return WHOLE_SCOPE_TEXT if whole or broken("scope-inside-script-unnamed", environment) else SCOPE_INSIDE_SCRIPT_TEXT


def crash_scope_text(command, cwd, environment=None):
    """确认单「崩溃用例与域」那一行（task_server.py 文件头 ⑦）：命令里点名的 crash-case 项数与 --crash-expand-up-to 的值；CRASH_SCOPE_ENTRIES 起的、命令里都没写的写 SCOPE_INSIDE_SCRIPT_TEXT；
    别的命令一样都没写的交空串（不打这一行）。弄坏开关 crash-scope-unreported 下一律空串。"""
    if broken("crash-scope-unreported", environment):
        return ""
    inner_results = [found for found in (crash_scope_text(inner, cwd, environment) for inner in shell_inner_commands(command, environment)) if found]
    if inner_results:   # bash -c 的脚本里命令位置上点了崩溃用例或域的：读那一条（脚本里有几条，取头一条给出确实点了的）
        pointed = [found for found in inner_results if found != SCOPE_INSIDE_SCRIPT_TEXT]
        return (pointed or inner_results)[0]
    words = key_words(command, environment)
    cases = [value for name, value in named_scope(command, environment) if name == "--item" and value.startswith(CRASH_CASE_ITEM_PREFIX)]
    domain = ""
    for position, word in enumerate(words):
        if word == CRASH_DOMAIN_ARGUMENT and position + 1 < len(words):
            domain = words[position + 1]
        elif word.startswith(CRASH_DOMAIN_ARGUMENT + "="):
            domain = word[len(CRASH_DOMAIN_ARGUMENT) + 1:]
    found = entry_word(words, cwd)
    entry = found is not None and any(entry_is(found, relative) for relative in CRASH_SCOPE_ENTRIES)
    if not cases and not domain:
        return SCOPE_INSIDE_SCRIPT_TEXT if entry else ""
    case_text = f"{len(cases)} 条（命令里点名）" if cases else SCOPE_INSIDE_SCRIPT_TEXT
    domain_text = f"{CRASH_DOMAIN_ARGUMENT} {domain}" if domain else f"命令里没给 {CRASH_DOMAIN_ARGUMENT}，由脚本定"
    return f"用例 {case_text}；域 {domain_text}"


def named_scope_entry(command, environment=None):
    """点的项属于哪一条入口（比两条命令点的项交不交叠只在同一条入口里比）：剥过前缀之后命令位置上那个词，cargo 另接 -p / --package 点的包（排过序）。"""
    words = key_words(command, environment)
    index = command_word_index(words)
    if index >= len(words):
        return ""
    if os.path.basename(words[index]) != "cargo":
        return words[index]
    packages = sorted(words[position + 1] for position in range(index, len(words) - 1) if words[position] in ("-p", "--package"))
    return " ".join(["cargo", *packages])


KEY_STATS_RECENT = 5   # control key-stats 看每把命令键的近几次（同 task_server.py 的 RECORD_WINDOW：定额度读的近几次）
KEY_STATS_COPY_MARK = "singlefs/work/"   # 命令键里带这一段的是工作根下副本或草稿目录的路径（同一条命令换一份副本就是一个新键）
# control key-stats --dump 逐行写的列（research/prompts/ledger80-key-records-r1-main-verification.md「三、改法」第 3 条）：id 是尝试的行号，别的取尝试记录里同名的一栏，没有这一栏的写 null
KEY_STATS_DUMP_COLUMNS = ("id", "task_id", "number", "command_key", "command", "started_at", "ended_at", "outcome", "exit_code", "memory_cap", "memory_peak",
                          "peak_metric", "stalled_at_cap", "prediction", "shard_count", "host", "item_plan")
KEY_STATS_SINCE_DATE = re.compile(r"\d{4}-\d{2}-\d{2}")   # --since 的日期写法（取那一天开头，日期照 time.gmtime 算：与 control records 打的起跑日期同一种算法）


def current_peak_metric(attempt):
    """尝试记录的峰值口径是不是这一版的（文件头「资源记录」的峰值口径一栏）：peak_metric 是 PROCESS_MEMORY_PEAK_METRIC 的才算。
    定额度（task_server.py 的 current_metric）与 control key-stats（key_stats）共用这一个判法。"""
    return attempt.get("peak_metric") == PROCESS_MEMORY_PEAK_METRIC


def key_stats_since(text):
    """control key-stats --since 的值 → (epoch 秒, 认不出的原因)：写成 epoch 秒（整数或小数）或 YYYY-MM-DD（那一天开头，日期照 time.gmtime 算、calendar.timegm 换回 epoch 秒）。认不出交 (None, 原因)。"""
    with contextlib.suppress(ValueError):
        value = float(text)
        if math.isfinite(value) and value >= 0:
            return value, ""
    if KEY_STATS_SINCE_DATE.fullmatch(text):
        with contextlib.suppress(ValueError):
            return float(calendar.timegm(time.strptime(text, "%Y-%m-%d"))), ""
    return None, f"--since 的值「{text}」认不出：写成 epoch 秒或 YYYY-MM-DD"


def key_stats_after(attempts, since):
    """起跑时刻（started_at）不早于 since（epoch 秒）的尝试；since 是 None 的照原样交。弄坏开关 key-stats-since-ignored 下不筛。"""
    if since is None or broken("key-stats-since-ignored"):
        return list(attempts)
    return [attempt for attempt in attempts
            if isinstance(attempt.get("started_at"), (int, float)) and not isinstance(attempt.get("started_at"), bool) and float(attempt["started_at"]) >= since]


def key_stats_mark(attempt):
    """key_stats 里一次尝试的标记：撞上限（RECORD_CAP_HIT_OUTCOMES）记 hit，跑成记 clean，别的结局交空串（不进交替的数）。"""
    if attempt.get("outcome") in RECORD_CAP_HIT_OUTCOMES:
        return "hit"
    return "clean" if attempt.get("outcome") == "succeeded" else ""


def key_stats_flips(marks):
    """一串标记（空串去掉）里前后不同的回数。"""
    marks = [mark for mark in marks if mark]
    return sum(1 for before, after in zip(marks, marks[1:]) if before != after), marks


def key_stats(attempts, recent=KEY_STATS_RECENT):
    """control key-stats 的统计（records/2026-11-17-subagent账本自上而下清欠.md 八十节 ①④ 回看要数的几样；research/prompts/ledger80-key-records-r1-main-verification.md「三、改法」第 2 条；只读、不改任何判法）：
    attempts 是尝试记录的列表（命令键、任务号、第几次、state、结局、峰值、峰值口径、起跑时刻），只看收尾了的、有命令键的、峰值口径是这一版的（current_peak_metric，与定额度读的同一个判法），
    按命令键分组，组里按起跑先后排。→ {used, keys, single, copy_keys, alternating: [键…], alternating_attempts: [键…], hit_then_clean, dispersion: [(键, 最大峰值÷最小峰值)…], median, p90, over_one_and_half}：
    used 是进统计的尝试数；alternating 按任务数：同一任务（task_id）的几次尝试折成一条，标记取这个任务第一次尝试（number 最小的）的结局（key_stats_mark），
    任务按第一次尝试的起跑先后排，近 recent 个任务里撞上限与跑成翻面至少两回的键；hit_then_clean 同样按任务数：近 recent 个任务里有撞过上限的、最近一个任务跑成的键数；
    alternating_attempts 按尝试数：近 recent 次尝试逐次的标记翻面至少两回的键；dispersion 是近 recent 次尝试里跑成并量到峰值至少两次的键的最大峰值比最小峰值（中位数与第 90 百分位取最近排名法）。
    弄坏开关 key-stats-alternation-ignored 下 alternating 交空，key-stats-metric-unfiltered 下不筛峰值口径，key-stats-retries-unfolded 下按任务数的两样也逐次尝试数。"""
    used = [attempt for attempt in attempts if attempt.get("state") == "ended" and attempt.get("command_key")
            and (current_peak_metric(attempt) or broken("key-stats-metric-unfiltered"))]
    by_key = {}
    for attempt in sorted(used, key=lambda a: (float(a.get("started_at") or 0), int(a.get("id") or 0))):
        by_key.setdefault(attempt["command_key"], []).append(attempt)
    alternating, alternating_attempts, hit_then_clean, dispersion = [], [], 0, []
    for key, runs in by_key.items():
        latest = runs[-recent:]
        if key_stats_flips(key_stats_mark(run) for run in latest)[0] >= 2:
            alternating_attempts.append(key)
        by_task = {}
        for run in runs:
            by_task.setdefault(run.get("task_id"), []).append(run)
        firsts = [min(group, key=lambda a: (int(a.get("number") or 0), float(a.get("started_at") or 0), int(a.get("id") or 0))) for group in by_task.values()]
        if broken("key-stats-retries-unfolded"):
            firsts = runs
        flips, marks = key_stats_flips(key_stats_mark(run) for run in firsts[-recent:])
        if flips >= 2 and not broken("key-stats-alternation-ignored"):
            alternating.append(key)
        if "hit" in marks and marks[-1:] == ["clean"]:
            hit_then_clean += 1
        peaks = [run["memory_peak"] for run in latest if run.get("outcome") == "succeeded" and isinstance(run.get("memory_peak"), int) and not isinstance(run.get("memory_peak"), bool)
                 and run["memory_peak"] > 0]
        if len(peaks) >= 2:
            dispersion.append((key, max(peaks) / min(peaks)))
    ratios = sorted(ratio for _, ratio in dispersion)
    return {"used": len(used), "keys": len(by_key), "single": sum(1 for runs in by_key.values() if len(runs) == 1), "copy_keys": sum(1 for key in by_key if KEY_STATS_COPY_MARK in key),
            "alternating": alternating, "alternating_attempts": alternating_attempts, "hit_then_clean": hit_then_clean, "dispersion": dispersion,
            "median": ratios[(len(ratios) - 1) // 2] if ratios else None, "p90": ratios[max(0, math.ceil(0.9 * len(ratios)) - 1)] if ratios else None,
            "over_one_and_half": sum(1 for ratio in ratios if ratio >= 1.5)}


def key_stats_lines(stats):
    """key_stats 的结果写成几行给人看（control key-stats 打）。"""
    def named(keys):
        return "：" + "；".join(key[:80] for key in keys[:5]) if keys else ""
    lines = [f"命令键 {stats['keys']} 个（只跑过一次的 {stats['single']} 个），其中带工作根下副本或草稿目录路径的 {stats['copy_keys']} 个",
             f"近 {KEY_STATS_RECENT} 个任务里撞上限与跑成交替至少两回的键 {len(stats['alternating'])} 个（按任务数：同一任务的几次尝试折成一条，取第一次尝试的结局）" + named(stats["alternating"]),
             f"近 {KEY_STATS_RECENT} 个任务里撞过上限、最近一个任务跑成的键 {stats['hit_then_clean']} 个（按任务数，同上）",
             f"按尝试数：近 {KEY_STATS_RECENT} 次尝试里撞上限与跑成交替至少两回的键 {len(stats['alternating_attempts'])} 个" + named(stats["alternating_attempts"])]
    if stats["dispersion"]:
        lines.append(f"近 {KEY_STATS_RECENT} 次里跑成并量到峰值至少两次的键 {len(stats['dispersion'])} 个，各键最大峰值比最小峰值：中位 {stats['median']:.2f} 倍、第 90 百分位 {stats['p90']:.2f} 倍，"
                     f"到 1.5 倍以上的 {stats['over_one_and_half']} 个")
    else:
        lines.append(f"近 {KEY_STATS_RECENT} 次里跑成并量到峰值至少两次的键 0 个")
    return lines


def key_stats_dump_rows(attempts):
    """control key-stats --dump 逐行写的内容：收尾了的尝试按行号排，每条取 KEY_STATS_DUMP_COLUMNS 那几列（尝试记录没有那一栏的交 None，写出来是 null）。"""
    ended = sorted((attempt for attempt in attempts if attempt.get("state") == "ended"), key=lambda a: int(a.get("id") or 0))
    return [{column: attempt.get(column) for column in KEY_STATS_DUMP_COLUMNS} for attempt in ended]


def peak_memory_note(stat):
    """内存最高那一刻的 memory.stat（task_server.py 文件头 ①「内存最高那一刻」：attempt_runner.py 的 memory_stat_at_peak，第二台是 MULTI_HOST_RUN_PEER_MEMORY_STAT 里 current_bytes 最大的那一行）
    换成给人看的一句（读不到的那一项写读不到）；没有交空串。大头在 file（页缓存，含没写回的脏页）的另写一句「大头是页缓存」，在 anon 的写「大头是进程自己的内存（堆）」：只记、不改判法。"""
    if not isinstance(stat, dict):
        return ""

    def shown(name):
        return mebibytes_text(stat[name]) if isinstance(stat.get(name), int) else "读不到"
    text = (f"内存最高那一刻（memory.current {shown('current')}）scope 的内存明细：anon {shown('anon')}、file {shown('file')}（其中没写回的脏页 file_dirty {shown('file_dirty')}、"
            f"正在写回 file_writeback {shown('file_writeback')}）、shmem {shown('shmem')}")
    if isinstance(stat.get("anon"), int) and isinstance(stat.get("file"), int):
        text += "：大头是页缓存（file）" if stat["file"] > stat["anon"] else "：大头是进程自己的内存（anon，堆）"
    return text


def whole_package_cargo_test(rest):
    """一条 cargo test 后面的参数（rest）是不是整包测试（task_server.py 文件头 ②「提交时拒」⑥）：→ 一句是哪一种，或空串（点名了测试目标）。
    带 --workspace / --all 的是整包；点名了 --test、--bin、或带测试名（`--` 前后不以 - 起头的词、选项的值不算：CARGO_VALUE_OPTIONS 里的选项值写在下一个词里的连同值跳过，
    `--名=值` 与连写的 `-p包` 不吞下一个词）的不是；--lib 与 --tests 都带、没带测试名的是整包；别的都是。
    弄坏开关 cargo-exclude-value-unskipped 下 --exclude 的值当测试名（`-p 包 --exclude 别的` 当点名了测试）。"""
    before, after = (rest[:rest.index("--")], rest[rest.index("--") + 1:]) if "--" in rest else (list(rest), [])
    value_options = tuple(option for option in CARGO_VALUE_OPTIONS if option != "--exclude") if broken("cargo-exclude-value-unskipped") else CARGO_VALUE_OPTIONS
    options, names, index = set(), [], 0
    while index < len(before):
        word = before[index]
        name = word.split("=", 1)[0] if word.startswith("--") else word
        if word.startswith("-"):
            options.add(name)
            if name in value_options and "=" not in word:   # 连写的 -p包 name 是整个词、不在表里，不吞下一个词
                index += 1
        else:
            names.append(word)
        index += 1
    skip = False
    for word in after:
        if skip:
            skip = False
            continue
        if word in LIBTEST_VALUE_OPTIONS:
            skip = True
        elif not word.startswith("-"):
            names.append(word)
    if options & {"--workspace", "--all"}:
        return "cargo test 带 --workspace（整个工作区的每一个包）"
    if names:
        return ""
    if {"--lib", "--tests"} <= options:
        return "cargo test 带 --lib --tests、没带测试名（包里全部测试目标）"
    if options & {"--test", "--bin"}:
        return ""
    return "cargo test 没点名 --test、--bin，也没带测试名（整个包的测试）"


def entry_word(argv, cwd):
    """命令位置上起的那一个词（剥法同 sharded_entry）：→ (词, 真路径, 后面的参数)；没有命令词交 None。"""
    index = command_word_index(argv)
    if index >= len(argv):
        return None
    return argv[index], os.path.realpath(os.path.join(cwd or REPOSITORY_ROOT, argv[index])), argv[index + 1:]


def entry_is(found, relative):
    """entry_word 交的那个词起的是不是仓根下的 relative 那一份。"""
    word, script, _ = found
    return script.endswith("/" + relative) or word == relative


def checked_cells(arguments):
    """--check <格,…> 与 --check=<格,…> 点的格（给几次取并集）。"""
    cells = set()
    for position, argument in enumerate(arguments):
        if argument == "--check" and position + 1 < len(arguments):
            cells.update(arguments[position + 1].split(","))
        elif argument.startswith("--check="):
            cells.update(argument[len("--check="):].split(","))
    return cells


def whole_package_script(argv, cwd, environment=None):
    """命令位置上起的入口是不是整包测试（task_server.py 文件头 ②「提交时拒」⑥）：→ 一句或空串。带 --selftest、--list-items 这类的不是。
    crash-point-replay.sh 门禁的 --check 点了 WHOLE_PACKAGE_CHECK_CELLS 里的哪一格、几格都是（crash-case-replay 带 --crash-cases-only 的那一格不算；
    feature-gated-tests 带了 CRASH_GATE_FEATURE_ITEM_PREFIX 起头的 --item 的那一格不算：只编、只跑点名的挂特性文件，同 crash_gate_whole_package_cells）；
    merge-and-verify.sh 没给补丁目录、或带 --all-packages 的是。弄坏开关 whole-package-cells-all-three-only 下 --check 要三格全点才算，
    merge-all-packages-accepted 下 --all-packages 不算，whole-package-feature-items-uncounted 下点了挂特性文件的 feature-gated-tests 照算整包，
    whole-package-any-item-exempts 下带了任何一个 --item 的 feature-gated-tests 都不算（崩溃枚举用例的项也算点了挂特性文件）。"""
    found = entry_word(argv, cwd)
    if found is None or any(argument in NOT_SHARDED_ARGUMENTS for argument in found[2]):
        return ""
    arguments = found[2]
    if entry_is(found, WHOLE_PACKAGE_CRASH_GATE):
        cells = checked_cells(arguments)
        if broken("whole-package-cells-all-three-only", environment):
            named = list(WHOLE_PACKAGE_CHECK_CELLS) if set(WHOLE_PACKAGE_CHECK_CELLS) <= cells else []
        else:
            crash_cases_only = CRASH_CASES_ONLY_ARGUMENT in arguments
            items = item_option_values(arguments)
            if broken("whole-package-any-item-exempts", environment):
                feature_items = items
            else:
                feature_items = [] if broken("whole-package-feature-items-uncounted", environment) else [
                    item for item in items if item.startswith(CRASH_GATE_FEATURE_ITEM_PREFIX)]
            named = [cell for cell in WHOLE_PACKAGE_CHECK_CELLS if cell in cells and not (cell == "crash-case-replay" and crash_cases_only)
                     and not (cell == "feature-gated-tests" and feature_items)]
        if not named:
            return ""
        alone = "（crash-case-replay 没带 --crash-cases-only）" if "crash-case-replay" in named else ""
        if "feature-gated-tests" in named:
            alone += f"（feature-gated-tests 没点 {CRASH_GATE_FEATURE_ITEM_PREFIX}<文件名> 项）"
        return f"{WHOLE_PACKAGE_CRASH_GATE} --check 点了 {'、'.join(named)}{alone}：验证包那一步的格是整包的测试"
    if entry_is(found, WHOLE_PACKAGE_MERGE_ENTRY):
        if MERGE_ALL_PACKAGES_ARGUMENT in arguments and not broken("merge-all-packages-accepted", environment):
            return f"{WHOLE_PACKAGE_MERGE_ENTRY} 带 {MERGE_ALL_PACKAGES_ARGUMENT}（每个包都跑）"
        position, given = 0, []
        while position < len(arguments):
            argument = arguments[position]
            if argument in MERGE_VALUE_OPTIONS:
                position += 2
                continue
            if not argument.startswith("-"):
                given.append(argument)
            position += 1
        return "" if given else f"{WHOLE_PACKAGE_MERGE_ENTRY} 没给补丁目录（合入验证整包跑一遍）"
    return ""


def unscoped_run_entry(argv, cwd, environment=None):
    """命令位置上起的是 UNSCOPED_RUN_ENTRIES 里的门禁或入口、又没带选条参数（不指定范围，跑全部项）：→ 一句或空串。带 --selftest、--list-items 这类的、
    带了那一份「不跑」的选项的不是。带全量门禁标识（task submit --full-gate）的由调用方先放行。弄坏开关 unscoped-gate-accepted 下一律交空串。"""
    found = entry_word(argv, cwd)
    if found is None or broken("unscoped-gate-accepted", environment) or any(argument in NOT_SHARDED_ARGUMENTS for argument in found[2]):
        return ""
    arguments = found[2]
    for relative, (options, positional, not_runs) in UNSCOPED_RUN_ENTRIES.items():
        if not entry_is(found, relative):
            continue
        if any(argument in not_runs for argument in arguments):
            return ""
        if any(argument in options or argument.startswith(tuple(option + "=" for option in options)) for argument in arguments):
            return ""
        if positional and any(not argument.startswith("-") for argument in arguments):
            return ""
        wanted = "、".join(list(options) + (["实验号"] if positional else []))
        return f"{relative} 没带选条参数（{wanted}）：不指定范围、跑全部项"
    return ""


def whole_package_confirmation_problem(path, shared_root_of):
    """整包确认文件（task submit --whole-package-confirmed <文件>；task_server.py 文件头 ②「提交时拒」⑥）的形式：绝对路径、真路径落在某一段共享根下（shared_root_of 交那一段，
    不在交空串）、读得到、有以 WHOLE_PACKAGE_MARKS 三个记号起头的三段，每段去掉空白与记号至少 WHOLE_PACKAGE_SECTION_MINIMUM_CHARACTERS 个字。
    → 不合的那一句（点名缺哪一段、哪一段太短），合的交空串。只判形式，不判写得对不对。"""
    if not path or not os.path.isabs(path):
        return f"--whole-package-confirmed 给的「{path}」不是绝对路径"
    real = os.path.realpath(path)
    if not shared_root_of(real):
        return f"整包确认文件 {path} 不在多机配置 NFS_SHARED_ROOTS 的共享根下（两台都要读得到）"
    try:
        with open(real, encoding="utf-8", errors="replace") as handle:
            text = handle.read()
    except OSError as error:
        return f"整包确认文件 {path} 读不了（{error}）"
    sections, current = {}, None
    for line in text.splitlines():
        stripped = line.strip().lstrip("-*#> ").strip()
        mark = next((mark for mark in WHOLE_PACKAGE_MARKS if stripped.startswith(mark)), None)
        if mark is not None:
            current = mark
            sections[mark] = sections.get(mark, "") + stripped[len(mark):]
        elif current is not None:
            sections[current] += line
    missing = [mark for mark in WHOLE_PACKAGE_MARKS if mark not in sections]
    if missing:
        return f"整包确认文件 {path} 缺以 {'、'.join(missing)} 起头的那一段（三段：①改到的文件与函数、②引到它们的测试目标与函数及 grep 命中、③为什么只点名②不够）"
    short = [mark for mark in WHOLE_PACKAGE_MARKS if len("".join(sections[mark].split())) < WHOLE_PACKAGE_SECTION_MINIMUM_CHARACTERS]
    if short:
        return f"整包确认文件 {path} 的 {'、'.join(short)} 那一段去掉空白不到 {WHOLE_PACKAGE_SECTION_MINIMUM_CHARACTERS} 个字"
    return ""


def commit_gate_entry(argv, cwd):
    """全量门禁标识（task submit --full-gate；文件头「异常表」long-run-unconfirmed）认的命令：起的是 COMMIT_GATE_ENTRIES 之一（剥法同 sharded_entry）、
    参数里没有 --check 与 --item（「--check=…」这种写法同样算）。"""
    index = command_word_index(argv)
    if index >= len(argv):
        return False
    word, arguments = argv[index], argv[index + 1:]
    script = os.path.realpath(os.path.join(cwd or REPOSITORY_ROOT, word))
    if not any(script.endswith("/" + relative) or word == relative for relative in COMMIT_GATE_ENTRIES):
        return False
    return not any(argument in SCOPE_RANGE_ARGUMENTS or argument.startswith(tuple(name + "=" for name in SCOPE_RANGE_ARGUMENTS)) for argument in arguments)


def item_option_values(arguments):
    """--item <项> 与 --item=<项> 点的项（值不空的，次序照命令）。"""
    values = []
    for position, argument in enumerate(arguments):
        if argument == "--item" and position + 1 < len(arguments) and arguments[position + 1]:
            values.append(arguments[position + 1])
        elif argument.startswith("--item=") and len(argument) > len("--item="):
            values.append(argument[len("--item="):])
    return values


def item_option_given(arguments):
    """参数里有没有 --item <项> 或 --item=<项>（值不空）。"""
    return bool(item_option_values(arguments))


def test_starting_gate_cells(environment=None):
    """起测试的格表 TEST_STARTING_GATE_CELLS（unnamed_run_problem 与自证 test-starting-cells-listed 都经这里读）。
    弄坏开关 test-starting-cells-misnamed 下 lkmm 那一行的格名写错一个字（那一道的格名表里没有它）。"""
    if broken("test-starting-cells-misnamed", environment):
        return {**TEST_STARTING_GATE_CELLS, ".claude/gate.d/lkmm.sh": ("litmus-verdict",)}
    return TEST_STARTING_GATE_CELLS


def crash_gate_whole_package_cells(arguments, environment=None):
    """门禁 crash-point-replay 带了 --item 的这一条（arguments 是阶段文件后面的参数）照它文件头「逐项点名」「--crash-cases-only」实际要整格跑的
    WHOLE_PACKAGE_CHECK_CELLS 里的格 → 格名列表（次序照那张表），空的是只跑点名的项。
    要跑的格：给了 --check 的照点的格；没给 --check 的，点了崩溃枚举用例（不以 CRASH_GATE_FEATURE_ITEM_PREFIX 起头的项）照旧九格，只点挂特性的文件的只跑 feature-gated-tests 一格。
    其中不整格跑的：crash-case-replay 带 --crash-cases-only（只走按块那一路、按条复用那一趟不起）；feature-gated-tests 点了挂特性的文件（只编、只跑点名的那几个）。
    product-gpu-real-card-tests 不收点名，跑到就是整格。
    弄坏开关 unnamed-crash-gate-item-whole-accepted 下一律交空（带 --item 就算点名），unnamed-crash-gate-cases-only-uncounted 下 --crash-cases-only 不算数。"""
    if broken("unnamed-crash-gate-item-whole-accepted", environment):
        return []
    items = item_option_values(arguments)
    feature_items = [item for item in items if item.startswith(CRASH_GATE_FEATURE_ITEM_PREFIX)]
    cells = checked_cells(arguments) or (set(WHOLE_PACKAGE_CHECK_CELLS) if len(feature_items) < len(items) else {"feature-gated-tests"})
    cases_only = CRASH_CASES_ONLY_ARGUMENT in arguments and not broken("unnamed-crash-gate-cases-only-uncounted", environment)
    return [cell for cell in WHOLE_PACKAGE_CHECK_CELLS
            if cell in cells and not (cell == "crash-case-replay" and cases_only) and not (cell == "feature-gated-tests" and feature_items)]


def gate_stage_of_entry(found):
    """entry_word 交的那个词起的是不是 .claude/gate.d/ 顶层的门禁阶段（lib/ 下的不算）：是交「.claude/gate.d/<文件名>」，不是交空串。"""
    word, real, _ = found
    for path in (real, os.path.normpath(word)):
        parent = os.path.dirname(path)
        if path.endswith(".sh") and os.path.basename(parent) == "gate.d" and os.path.basename(os.path.dirname(parent)) == ".claude":
            return ".claude/gate.d/" + os.path.basename(path)
    return ""


def unnamed_cargo_test_problem(after_test, shown, environment=None):
    """cargo test / cargo t 后面的参数（after_test）点没点到 --exact 的单条测试函数：点了、只编不跑、只列用例交 None，没点交 {problem, next_step}。
    shown 是这条命令照原样写成的一串（拒绝说明里改成点名的样子照它接）。
    弄坏开关 unnamed-cargo-filter-accepted 下带了测试目标或筛选词就算点名，unnamed-cargo-exact-uncounted 下 --exact 的测试名不算点名。"""
    before, after = (after_test[:after_test.index("--")], after_test[after_test.index("--") + 1:]) if "--" in after_test else (list(after_test), [])
    if any(word in UNNAMED_RUN_CARGO_QUERIES for word in before) or "--list" in after:
        return None
    names, skip = [], False
    for word in after:
        if skip:
            skip = False
        elif word in LIBTEST_VALUE_OPTIONS:
            skip = True
        elif not word.startswith("-"):
            names.append(word)
    if "--exact" in after and names and not broken("unnamed-cargo-exact-uncounted", environment):
        return None
    if broken("unnamed-cargo-filter-accepted", environment) and (names or any(word in ("--test", "--bin", "--lib") or not word.startswith("-") for word in before)):
        return None
    head = shown.split(" -- ", 1)[0] if " -- " in shown else shown
    example = f"{head} -- {' '.join(names) if names else '<测试全名>'} --exact"
    return {"problem": f"{UNNAMED_RUN_PROBLEM_HEAD}：{shown[:240]} 没点名到单条测试函数（`--` 之后要带 --exact 与测试全名；只点测试目标、只给筛选词的都跑整个目标或一批测试）",
            "next_step": f"改成点名改到的那几条测试函数：{example}（几条就写几个全名，模块路径写全）；{UNNAMED_RUN_EXEMPTION_TEXT}"}


def unnamed_run_problem(argv, cwd, full_gate=False, whole_package_confirmed=False, environment=None):
    """启动测试与验证的一条命令（命令位置上那一条，前缀照 command_word_index 剥）点没点名（task_server.py 文件头 ②「提交时拒」⑦；异常表 unnamed-run）：
    → None（点了名、只查不跑、不是这几种、带了例外的标识），或 {problem: 哪一种没点名, next_step: 改成点名的样子}。
    例外两种：交它的那一条带 --full-gate（提交时那一趟；服务的 full_gate_refusal 先核过它只给不指定范围的提交时门禁），带 --whole-package-confirmed（用户确认过的整包，服务先核过确认文件的形式）。
    算点名的：cargo test / cargo t 的 `--` 之后带 --exact 与至少一个测试全名（LIBTEST_VALUE_OPTIONS 的值不算；--no-run、--help 与 `-- --list` 不起测试，不判）；
      UNNAMED_RUN_ITEM_ENTRIES 那三份带 --item；证红 prove-red.sh --copy <副本> <crate> 之后至少一个变异名；
      .claude/gate.d/ 顶层的门禁阶段带 --item，或带 --check 而点的格全不在 TEST_STARTING_GATE_CELLS 这一道那一行；门禁 crash-point-replay 带了 --item 还要
      crash_gate_whole_package_cells 交空（--check 把起整个验证包的格排掉）。只查不跑的参数（各自那一张 *_QUERIES）不判。
    bash -c 的脚本由调用方逐条拆开再问（服务用 shell_inner_commands，范围声明闸用它自己的切词），这里只判一条。
    弄坏开关：unnamed-run-exemptions-ignored 下带了例外标识照判；unnamed-entry-item-unchecked 下三份入口不带 --item 也算点名，unnamed-entry-item-uncounted 下带了也不算；
      unnamed-prove-red-unchecked 下证红不给变异名也算点名，unnamed-prove-red-name-offset 下变异名从第二个起数（只点一个的算没点）；
      unnamed-stage-whole-accepted 下门禁阶段 --item、--check 都不带也算点名，unnamed-check-cells-accepted 下带 --check 就算点名、不看点的格起不起测试；
      cargo test 的两个见 unnamed_cargo_test_problem，门禁 crash-point-replay 带 --item 的两个见 crash_gate_whole_package_cells，起测试的格表的见 test_starting_gate_cells。"""
    if (full_gate or whole_package_confirmed) and not broken("unnamed-run-exemptions-ignored", environment):
        return None
    argv = list(argv)
    index = command_word_index(argv)
    if index >= len(argv):
        return None
    shown = shlex.join(argv[index:])
    if os.path.basename(argv[index]) == "cargo":
        position = index + 1
        while position < len(argv) and (argv[position].startswith("+") or argv[position].startswith("-")):
            position += 2 if argv[position] in ("-Z", "--config", "--color", "-C") else 1
        if position < len(argv) and argv[position] in ("test", "t"):
            return unnamed_cargo_test_problem(argv[position + 1:], shown, environment)
        return None
    found = entry_word(argv, cwd)
    word, _, arguments = found
    arguments = [argument for argument in arguments if argument != "--force"]

    def named_form(addition):
        return " ".join([shlex.quote(word), addition] + ([shlex.join(arguments)] if arguments else []))
    for relative, (named, queries) in UNNAMED_RUN_ITEM_ENTRIES.items():
        if not entry_is(found, relative):
            continue
        if any(argument in queries for argument in arguments):
            return None
        given = item_option_given(arguments) and not broken("unnamed-entry-item-uncounted", environment)
        if given or broken("unnamed-entry-item-unchecked", environment):
            return None
        return {"problem": f"{UNNAMED_RUN_PROBLEM_HEAD}：{shown[:240]} 没带 --item（{relative} 不带 --item 就跑它自己挑的全部项或全部目标）",
                "next_step": f"改成只跑改到的、红了的那几项：{named_form(named)}；{UNNAMED_RUN_EXEMPTION_TEXT}"}
    if entry_is(found, UNNAMED_RUN_PROVE_RED_ENTRY):
        if any(argument in UNNAMED_RUN_PROVE_RED_QUERIES for argument in arguments):
            return None
        names_start = 4 if broken("unnamed-prove-red-name-offset", environment) else 3   # --copy <副本> <crate> 之后是变异名
        if (len(arguments) > names_start and arguments[0] == "--copy") or broken("unnamed-prove-red-unchecked", environment):
            return None
        return {"problem": f"{UNNAMED_RUN_PROBLEM_HEAD}：{shown[:240]} 没点名变异（证红要在 --copy <副本> <crate> 之后写至少一个变异名）",
                "next_step": f"改成点名这一次新写的变异行：bash {UNNAMED_RUN_PROVE_RED_ENTRY} --copy <副本> <crate> <变异名…>（变异名是 crates/mutations.tsv 第一段）；{UNNAMED_RUN_EXEMPTION_TEXT}"}
    stage = gate_stage_of_entry(found)
    if not stage or any(argument in UNNAMED_RUN_STAGE_QUERIES for argument in arguments):
        return None
    if item_option_given(arguments):
        whole = crash_gate_whole_package_cells(arguments, environment) if stage == WHOLE_PACKAGE_CRASH_GATE else []
        if not whole:
            return None
        rest, position = [], 0
        while position < len(arguments):   # 改法照原命令去掉 --check 与 --crash-cases-only，接上 CRASH_GATE_CASES_ONLY_FORM
            if arguments[position] == "--check":
                position += 2
                continue
            if not arguments[position].startswith("--check=") and arguments[position] != CRASH_CASES_ONLY_ARGUMENT:
                rest.append(arguments[position])
            position += 1
        rewritten = " ".join([shlex.quote(word), CRASH_GATE_CASES_ONLY_FORM, shlex.join(rest)])
        return {"problem": (f"{UNNAMED_RUN_PROBLEM_HEAD}：{shown[:240]} 带了 --item，--check 却没把起整个验证包的格排掉，{'、'.join(whole)} 照跑整格"
                            f"（门禁 {stage} 文件头「逐项点名」：没给 --check 而点了崩溃枚举用例的照旧九格；crash-case-replay 不带 --crash-cases-only 的连按条复用那一趟一起起）"),
                "next_step": (f"点崩溃枚举用例的加 {CRASH_GATE_CASES_ONLY_FORM}：{rewritten}；只点 {CRASH_GATE_FEATURE_ITEM_PREFIX}<文件名> 的不带 --check、"
                              f"不与崩溃枚举用例写在同一条（只跑 feature-gated-tests 一格、只编点名的文件）；{UNNAMED_RUN_EXEMPTION_TEXT}")}
    cells = checked_cells(arguments)
    if not cells:
        if broken("unnamed-stage-whole-accepted", environment):
            return None
        return {"problem": f"{UNNAMED_RUN_PROBLEM_HEAD}：{shown[:240]} 没带 --item 也没带 --check（门禁 {stage} 的全部格、全部项）",
                "next_step": f"改成点名：{named_form('--item <项>')}（可点的项用 --list-items 列），或只点不起测试的格 --check <格>（格名用 --list 列）；{UNNAMED_RUN_EXEMPTION_TEXT}"}
    starting = sorted(cells & set(test_starting_gate_cells(environment).get(stage, ())))
    if not starting or broken("unnamed-check-cells-accepted", environment):
        return None
    return {"problem": f"{UNNAMED_RUN_PROBLEM_HEAD}：{shown[:240]} 的 --check 点了起测试的格 {'、'.join(starting)}，又没带 --item（起测试的格只点格不算点名）",
            "next_step": f"改成点名项：{named_form('--item <项>')}（可点的项用 --list-items 列；崩溃枚举用例写 --check crash-case-replay --crash-cases-only --item crash-case:<用例名>）；{UNNAMED_RUN_EXEMPTION_TEXT}"}


def scope_declaration_segments(environment, command):
    """范围声明 SINGLEFS_RUN_SCOPE 的原文照「;」切成的几段（确认单逐段打）：任务带的环境里的，或命令里起头的 env 后面写的；都没有交 []。"""
    text = (environment or {}).get("SINGLEFS_RUN_SCOPE", "")
    if not text and command and os.path.basename(command[0]) == "env":
        text = next((word.split("=", 1)[1] for word in command[1:] if word.startswith("SINGLEFS_RUN_SCOPE=")), "")
    return [segment.strip() for segment in text.split(";") if segment.strip()]


def iso_epoch(text):
    """「2024-01-30T12:34:56Z」这类时刻 → 秒；认不出交 0.0。"""  # clock-times:allow 格式示例说明
    try:
        return float(calendar.timegm(time.strptime(text[:19], "%Y-%m-%dT%H:%M:%S")))
    except (TypeError, ValueError):
        return 0.0


@dataclass
class RetryDecision:
    retry: bool
    outcome: str = ""      # 不重试时的结局
    cap_bytes: int = 0     # 重试的上限
    host: str = ""         # 重试钉在哪台（空串是照原来的取法）
    why: str = ""


def retry_decision(outcome_counts, outcome, cap_bytes, host, pinned_host, maxima, environment=None):
    """outcome_counts 是含这一次在内每种类型出过几次；maxima 是 {台: 那台的总上限字节或 None（查不到）}。交 RetryDecision（文件头「重试」）。
    脚本永远不报自己要多少内存（用户 2026-11-13 定「脚本永远不要报自己要多少」）：上限只照命令键的记录定、撞了乘 1.5，命令打的任何「要多少」行一律不认。"""
    used = outcome_counts.get(outcome, 0)
    if outcome not in RETRY_LIMITS:
        return RetryDecision(False, outcome=outcome)
    if outcome == "squeezed-out" and broken("squeeze-retry-skipped", environment):
        return RetryDecision(False, outcome=outcome)
    if outcome == "peer-run-failed" and broken("peer-run-retry-skipped", environment):
        return RetryDecision(False, outcome=outcome)
    if used > RETRY_LIMITS[outcome]:
        return RetryDecision(False, outcome=outcome, why=f"{outcome} 已重试 {RETRY_LIMITS[outcome]} 次")
    if outcome != "memory-cap-hit":
        return RetryDecision(True, cap_bytes=cap_bytes, host=pinned_host, why=f"{outcome}：同一上限重排")
    grown = cap_bytes if broken("cap-growth-skipped", environment) else grown_cap(cap_bytes)
    maximum = maxima.get(host)
    if maximum is None:
        return RetryDecision(True, cap_bytes=grown, host=pinned_host or (host if broken("retry-pinned-to-host", environment) else ""),
                             why="上限乘 1.5（这台的总上限查不到），哪台装得下去哪台")
    # 用户：「涨到总上限就打回」：乘 1.5 还在总上限以内的才重排；到或超过总上限的不重排（slice 里总挂着一点回收不掉的内核内存，记满一整台永远凑不齐）
    if grown < maximum or broken("cap-unclamped", environment):
        return RetryDecision(True, cap_bytes=grown, host=pinned_host or (host if broken("retry-pinned-to-host", environment) else ""),
                             why="上限乘 1.5（还在这台的总上限以内）、哪台装得下去哪台")
    if not broken("host-switch-skipped", environment):
        others = [(other_maximum, other) for other, other_maximum in maxima.items() if other != host and other_maximum and other_maximum > grown]
        if others:
            other_maximum, other = max(others)
            return RetryDecision(True, cap_bytes=grown, host=other,
                                 why=f"乘 1.5 就到这台的总上限，换总上限更大的{'第二台' if other == 'peer' else '本机'}")
    return RetryDecision(False, outcome="memory-cap-at-maximum",
                         why=f"撞上限、再乘 1.5 就到两台的总上限（{HOST_NAMES.get(host, host)} 撞在 {mebibytes_text(cap_bytes)}）")


def load_module(name, path):
    specification = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module


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
